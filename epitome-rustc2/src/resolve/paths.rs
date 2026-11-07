//! Resolving every path in every file, in parallel over files.
//!
//! A walk of each file keeps a stack of scopes ("ribs") for what is
//! visible lexically: generic parameters and `Self`, function parameters,
//! `let` bindings, match arms, `if let` and `while let` conditions,
//! `for` patterns and closure parameters. An item inside a function starts
//! afresh: it cannot see the function's locals or generics.
//!
//! A one-segment name is looked up in the ribs, then in the module's items
//! (from an enclosing block outward to its module), then the extern
//! prelude (crate names, for types), the std prelude, and the built-in
//! types. A longer path follows modules (and enums and traits, which hold
//! names too) from its first segment; when it reaches a type it stops and
//! records the rest for type checking.

use super::{DefKind, ModId, Ns, Res, ResolveStats, Resolver};
use crate::ast::{Ast, Kind, NodeId};
use crate::hash::FastMap;
use crate::pool;
use crate::session::{ParsedCrate, Session};
use std::sync::Mutex;

use super::PRIMITIVES;

/// The syntax context of a name seen from every context.
const ANY_CTX: u32 = u32::MAX;

pub(super) fn resolve_paths<'s>(session: &Session, r: &mut Resolver<'s>, crates: &[ParsedCrate<'s>]) {
    let jobs: Vec<(usize, usize)> =
        crates.iter().enumerate().flat_map(|(k, c)| (0..c.files.len()).map(move |i| (k, i))).collect();
    type FileRes = (usize, usize, Vec<Option<Res>>);
    let out: Mutex<Vec<FileRes>> = Mutex::new(Vec::new());
    let rr: &Resolver<'s> = r;
    pool::run(session.threads(), jobs, |(k, i), _| {
        let f = &crates[k].files[i];
        let mut w = Walker {
            r: rr,
            krate: k as u32,
            file: i as u32,
            ast: &f.ast,
            module: rr.file_modules[k][i],
            ribs: Vec::new(),
            out: vec![None; f.ast.nodes.len()],
        };
        w.walk(f.ast.root);
        out.lock().unwrap().push((k, i, w.out));
    });
    let mut res: Vec<Vec<Vec<Option<Res>>>> =
        crates.iter().map(|c| (0..c.files.len()).map(|_| Vec::new()).collect()).collect();
    for (k, i, m) in out.into_inner().unwrap() {
        res[k][i] = m;
    }
    r.res = res;
}

#[derive(Default)]
struct Rib<'s> {
    /// Local variables and const parameters: name, syntax context, what.
    values: Vec<(&'s str, u32, Res)>,
    types: Vec<(&'s str, Res)>,
}

struct Walker<'w, 's> {
    r: &'w Resolver<'s>,
    krate: u32,
    file: u32,
    ast: &'w Ast<'s>,
    module: ModId,
    ribs: Vec<Rib<'s>>,
    /// What each path (or identifier pattern naming a constant) resolved
    /// to, by node; `None` for a path that did not resolve.
    out: Vec<Option<Res>>,
}

/// One segment of a path: its name.
fn segments<'s>(ast: &Ast<'s>, path: NodeId) -> (bool, Option<NodeId>, Vec<(&'s str, NodeId)>) {
    let mut rooted = false;
    let mut qself = None;
    let mut segs = Vec::new();
    for &c in ast.children(path) {
        match ast.kind(c) {
            Kind::PathRoot => rooted = true,
            Kind::QSelf => qself = Some(c),
            Kind::Segment => {
                let name = ast.children(c).first().map_or("", |&n| ast.token(n).ident_name());
                segs.push((name, c));
            }
            _ => {}
        }
    }
    (rooted, qself, segs)
}

impl<'w, 's> Walker<'w, 's> {
    // ---- lookup ----

    /// A one-segment name in `ns`.
    fn lookup_name(&self, name: &str, ns: Ns, with_ribs: bool) -> Option<Res> {
        if with_ribs {
            if ns == Ns::Value {
                // A local in the same syntax context; for a name written in
                // a macro's body, then one written outside any macro (the
                // locals at the macro's definition, approximately).
                let ctx = self.r.ctx_of(name);
                let find = |c: u32| {
                    self.ribs.iter().rev().find_map(|rib| {
                        rib.values
                            .iter()
                            .rev()
                            .find(|(n, x, _)| *n == name && (*x == c || *x == ANY_CTX))
                            .map(|v| v.2.clone())
                    })
                };
                if let Some(res) = find(ctx).or_else(|| if ctx != 0 { find(0) } else { None }) {
                    return Some(res);
                }
            } else if ns == Ns::Type {
                for rib in self.ribs.iter().rev() {
                    if let Some((_, res)) = rib.types.iter().rev().find(|(n, _)| *n == name) {
                        return Some(res.clone());
                    }
                }
            }
        }
        let r = self.r;
        // The enclosing blocks, then the module.
        let mut m = self.module;
        loop {
            if let Some(res) = r.lookup(m, name, ns, self.module) {
                return Some(res);
            }
            let module = &r.modules[m.0 as usize];
            match module.parent {
                Some(p) if module.block => m = p,
                _ => break,
            }
        }
        if ns == Ns::Type
            && let Some(root) = r.extern_prelude(self.krate, name)
        {
            return Some(Res::Def(r.modules[root.0 as usize].def.unwrap()));
        }
        if let Some(p) = r.preludes[self.krate as usize]
            && let Some(res) = r.lookup(p, name, ns, self.module)
        {
            return Some(res);
        }
        if ns == Ns::Type
            && let Some(&p) = PRIMITIVES.iter().find(|&&p| p == name)
        {
            return Some(Res::PrimTy(p));
        }
        if ns == Ns::Macro {
            // Exported macros of every crate (the `#[macro_use]` prelude).
            for &root in &r.roots {
                if let Some(res) = r.lookup(root, name, Ns::Macro, root) {
                    return Some(res);
                }
            }
        }
        None
    }

    /// A whole path in `ns`.
    fn resolve_path(&self, path: NodeId, ns: Ns) -> Option<Res> {
        let (rooted, qself, segs) = segments(self.ast, path);
        if qself.is_some() {
            return Some(Res::Partial(Box::new(Res::PrimTy("<qualified>")), segs.len() as u32));
        }
        let names: Vec<&str> = segs.iter().map(|s| s.0).collect();
        if names.is_empty() {
            return None;
        }
        if names.len() == 1 && !rooted {
            if names[0] == "Self" {
                return self.lookup_name("Self", Ns::Type, true);
            }
            // A generic argument `N` is read as a type, but may be a
            // constant (a const parameter or item), as rustc also tries.
            // A module is never a type: `u8` in a type is the primitive
            // even where a module `u8` is in scope (as in `core`).
            if ns == Ns::Type
                && let Some(&p) = PRIMITIVES.iter().find(|&&p| p == names[0])
                && let Some(Res::Def(d)) = self.lookup_name(names[0], ns, true)
                && self.r.defs[d.0 as usize].kind == DefKind::Mod
            {
                return Some(Res::PrimTy(p));
            }
            return self
                .lookup_name(names[0], ns, true)
                .or_else(|| if ns == Ns::Type { self.lookup_name(names[0], Ns::Value, true) } else { None });
        }
        // The first segment: a keyword, `Self`, or a name in type scope.
        let first = names[0];
        let (cur, used) = if rooted || matches!(first, "crate" | "self" | "super") {
            let edition = self.r.editions[self.krate as usize];
            self.r.path_start(&names, rooted, self.module, self.krate, edition)?
        } else {
            let res = self.lookup_name(first, Ns::Type, true)?;
            match self.r.module_of_res(&res) {
                Some(m) => (m, 1),
                None => return Some(Res::Partial(Box::new(res), (names.len() - 1) as u32)),
            }
        };
        let last = names.len() - 1;
        let start = cur;
        let res = self.follow_path(&names, cur, used, ns);
        // `char::is_whitespace` where a module `char` is in scope: the
        // primitive type, as rustc tries when the module has no such name.
        if res.is_none()
            && used == 1
            && start != self.module
            && let Some(&p) = PRIMITIVES.iter().find(|&&p| p == first)
        {
            return Some(Res::Partial(Box::new(Res::PrimTy(p)), last as u32));
        }
        res
    }

    /// Follows `names[used..]` from module `cur`, the last in `ns`.
    fn follow_path(&self, names: &[&str], mut cur: ModId, used: usize, ns: Ns) -> Option<Res> {
        let last = names.len() - 1;
        for (i, &name) in names.iter().enumerate().skip(used) {
            let want = if i == last { ns } else { Ns::Type };
            let found = self.r.lookup(cur, name, want, self.module).or_else(|| {
                // The last segment of a path in one namespace may name the
                // other: `Struct { }` and `Enum::Unit`.
                if i == last && ns != Ns::Macro {
                    let other = if ns == Ns::Value { Ns::Type } else { Ns::Value };
                    self.r.lookup(cur, name, other, self.module)
                } else {
                    None
                }
            });
            let res = match found {
                Some(r) => r,
                None => {
                    // A name an enum does not define (`Ordering::is_lt`)
                    // is an associated item, for type checking to find.
                    let def = self.r.modules[cur.0 as usize].def?;
                    if self.r.defs[def.0 as usize].kind == DefKind::Enum {
                        return Some(Res::Partial(Box::new(Res::Def(def)), (last - i + 1) as u32));
                    }
                    return None;
                }
            };
            if i == last {
                return Some(res);
            }
            match self.r.module_of_res(&res) {
                Some(m) => cur = m,
                None => return Some(Res::Partial(Box::new(res), (last - i) as u32)),
            }
        }
        // The path is only keywords: `crate`, `super`.
        self.r.modules[cur.0 as usize].def.map(Res::Def)
    }

    fn record(&mut self, path: NodeId, ns: Ns) {
        // A path written in a macro 2.0's body resolves where the macro is;
        // failing that (an item the expansion itself defines, which full
        // hygiene would see), where it was called.
        let saved = self.module;
        let mut res = None;
        if let Some(m) = self.r.def_site_module(self.ast.token(path).0) {
            self.module = m;
            res = self.resolve_path(path, ns);
            self.module = saved;
        }
        let res = res.or_else(|| self.resolve_path(path, ns));
        if let Some(res) = res {
            self.out[path.0 as usize] = Some(res);
        }
    }

    // ---- scopes ----

    fn push(&mut self) {
        self.ribs.push(Rib::default());
    }

    fn pop(&mut self) {
        self.ribs.pop();
    }

    fn rib(&mut self) -> &mut Rib<'s> {
        if self.ribs.is_empty() {
            self.push();
        }
        self.ribs.last_mut().unwrap()
    }

    /// Generic parameters of an item, into the current rib.
    fn generics(&mut self, item: NodeId) {
        let Some(g) = self.ast.child(item, Kind::Generics) else { return };
        for &p in self.ast.children(g) {
            let Some(n) = self.ast.child(p, Kind::Name) else { continue };
            let name = self.ast.token(n).ident_name();
            match self.ast.kind(p) {
                Kind::TypeParam => self.rib().types.push((name, Res::Generic(p))),
                // Generic parameters are not hygienic, as locals are.
                Kind::ConstParam => self.rib().values.push((name, ANY_CTX, Res::Generic(p))),
                _ => {}
            }
        }
    }

    /// The names a pattern binds, resolving the paths in it as it goes.
    /// An identifier that names a unit struct, unit variant or constant in
    /// scope is that, not a new binding.
    fn pattern(&mut self, pat: NodeId, binds: &mut Vec<(&'s str, u32, Res)>) {
        match self.ast.kind(pat) {
            Kind::IdentPat => {
                let Some(n) = self.ast.child(pat, Kind::Name) else { return };
                let name = self.ast.token(n).ident_name();
                let plain = !self.ast.children(pat).iter().any(|&c| matches!(self.ast.kind(c), Kind::Keyword))
                    && self.ast.children(pat).len() == 1;
                if plain
                    && let Some(res) = self.lookup_name(name, Ns::Value, false)
                    && let Res::Def(d) = &res
                    && matches!(
                        self.r.defs[d.0 as usize].kind,
                        DefKind::Struct | DefKind::Variant | DefKind::Const | DefKind::AssocConst
                    )
                {
                    self.out[pat.0 as usize] = Some(res);
                    return;
                }
                binds.push((name, self.r.ctx_of(name), Res::Local(pat)));
                for &c in self.ast.children(pat) {
                    if !matches!(self.ast.kind(c), Kind::Name | Kind::Keyword) {
                        self.pattern(c, binds);
                    }
                }
            }
            Kind::PathPat | Kind::TupleStructPat => {
                let kids = self.ast.children(pat).to_vec();
                if let Some(&p) = kids.first() {
                    self.record(p, Ns::Value);
                    self.generic_args_of(p);
                }
                for &c in &kids[1..] {
                    self.pattern(c, binds);
                }
            }
            Kind::StructPat => {
                let kids = self.ast.children(pat).to_vec();
                if let Some(&p) = kids.first() {
                    self.record(p, Ns::Type);
                    self.generic_args_of(p);
                }
                for &c in &kids[1..] {
                    self.pattern(c, binds);
                }
            }
            Kind::FieldPat => {
                for &c in self.ast.children(pat).to_vec().iter() {
                    if !matches!(self.ast.kind(c), Kind::Name | Kind::Literal | Kind::Attribute | Kind::DocComment) {
                        self.pattern(c, binds);
                    }
                }
            }
            Kind::PatMacro => {
                if let Some(p) = self.ast.child(pat, Kind::Path) {
                    self.record(p, Ns::Macro);
                }
            }
            // Range ends may be paths (constants); literals have none.
            Kind::LitPat | Kind::RangePat | Kind::ConstBlock => {
                for c in self.ast.children(pat).to_vec() {
                    match self.ast.kind(c) {
                        Kind::PathPat => {
                            if let Some(&p) = self.ast.children(c).first() {
                                self.path(p, Ns::Value);
                            }
                        }
                        Kind::Op | Kind::Literal => {}
                        Kind::LitPat => {}
                        _ => self.walk(c),
                    }
                }
            }
            _ => {
                for c in self.ast.children(pat).to_vec() {
                    self.pattern(c, binds);
                }
            }
        }
    }

    /// Binds a pattern's names in the current rib.
    fn bind_pattern(&mut self, pat: NodeId) {
        let mut binds = Vec::new();
        self.pattern(pat, &mut binds);
        self.rib().values.extend(binds);
    }

    /// Walks the generic arguments inside a path's segments.
    fn generic_args_of(&mut self, path: NodeId) {
        for c in self.ast.children(path).to_vec() {
            match self.ast.kind(c) {
                Kind::Segment => {
                    for a in self.ast.children(c).to_vec().into_iter().skip(1) {
                        self.walk(a);
                    }
                }
                Kind::QSelf => self.walk(c),
                _ => {}
            }
        }
    }

    /// A path in namespace `ns`, and the types inside it.
    fn path(&mut self, path: NodeId, ns: Ns) {
        self.record(path, ns);
        self.generic_args_of(path);
    }

    /// An `if` or `while` condition: `let` bindings in it are visible in
    /// the rest of the condition and in the body (the caller's rib).
    fn condition(&mut self, e: NodeId) {
        match self.ast.kind(e) {
            Kind::Let => {
                let kids = self.ast.children(e).to_vec();
                if let [pat, value] = kids.as_slice() {
                    self.walk(*value);
                    self.bind_pattern(*pat);
                }
            }
            Kind::Binary if self.ast.text(self.ast.children(e)[1]) == "&&" => {
                let kids = self.ast.children(e).to_vec();
                self.condition(kids[0]);
                self.condition(kids[2]);
            }
            _ => self.walk(e),
        }
    }

    /// An item: it starts with no locals in scope, and its own generics.
    fn item(&mut self, item: NodeId) {
        let saved = std::mem::take(&mut self.ribs);
        self.item_body(item);
        self.ribs = saved;
    }

    /// An item's insides, with the scopes it makes.
    fn item_body(&mut self, item: NodeId) {
        let kind = self.ast.kind(item);
        self.push();
        match kind {
            Kind::Impl | Kind::Trait | Kind::Struct | Kind::Enum | Kind::Union => {
                let file = self.file;
                self.rib().types.push(("Self", Res::SelfTy(file, item)));
            }
            _ => {}
        }
        self.generics(item);
        let saved_module = self.module;
        if kind == Kind::Mod
            && let Some(&m) = self.r.module_of.get(&(self.krate, self.file, item))
        {
            self.module = m;
        }
        for c in self.ast.children(item).to_vec() {
            match self.ast.kind(c) {
                Kind::Params => {
                    for p in self.ast.children(c).to_vec() {
                        match self.ast.kind(p) {
                            Kind::SelfParam => {
                                let tok = self.ast.child(p, Kind::Name).map_or("self", |n| self.ast.token(n).0);
                                let ctx = self.r.ctx_of(tok);
                                self.rib().values.push(("self", ctx, Res::Local(p)));
                                if let Some(t) =
                                    self.ast.children(p).iter().copied().find(|&t| is_type(self.ast.kind(t)))
                                {
                                    self.walk(t);
                                }
                            }
                            Kind::Param | Kind::Variadic => {
                                let kids = self.ast.children(p).to_vec();
                                let kids: Vec<NodeId> = kids
                                    .into_iter()
                                    .filter(|&k| !matches!(self.ast.kind(k), Kind::Attribute | Kind::DocComment))
                                    .collect();
                                if kids.len() == 2 || self.ast.kind(p) == Kind::Variadic && kids.len() == 1 {
                                    if let Some(&t) = kids.get(1) {
                                        self.walk(t);
                                    }
                                    self.bind_pattern(kids[0]);
                                } else if let Some(&t) = kids.first() {
                                    self.walk(t);
                                }
                            }
                            _ => {}
                        }
                    }
                }
                // Items of impls and traits keep the impl's or trait's
                // generics and `Self`.
                Kind::Fn | Kind::Const | Kind::TypeAlias | Kind::MacroCall
                    if matches!(kind, Kind::Impl | Kind::Trait) =>
                {
                    self.push();
                    self.item_body(c);
                    self.pop();
                }
                k if is_item(k) => self.item(c),
                _ => self.walk(c),
            }
        }
        self.module = saved_module;
        self.pop();
    }

    // ---- the walk ----

    fn walk(&mut self, n: NodeId) {
        let kind = self.ast.kind(n);
        match kind {
            Kind::TokenTree
            | Kind::Attribute
            | Kind::InnerAttribute
            | Kind::DocComment
            | Kind::Visibility
            | Kind::ImplRestriction
            | Kind::Use
            | Kind::MacroRules
            | Kind::MacroDef
            | Kind::ExternCrate => {}
            _ if kind.is_leaf() => {}
            Kind::Crate => {
                for c in self.ast.children(n).to_vec() {
                    self.walk(c);
                }
            }
            _ if is_item(kind) => self.item(n),
            Kind::PathExpr => {
                if let Some(&p) = self.ast.children(n).first() {
                    self.path(p, Ns::Value);
                }
            }
            Kind::PathType | Kind::TraitBound => {
                for c in self.ast.children(n).to_vec() {
                    if self.ast.kind(c) == Kind::Path {
                        self.path(c, Ns::Type);
                    } else {
                        self.walk(c);
                    }
                }
            }
            Kind::StructExpr => {
                let kids = self.ast.children(n).to_vec();
                if let Some(&p) = kids.first() {
                    self.path(p, Ns::Type);
                }
                for &c in &kids[1..] {
                    self.walk(c);
                }
            }
            Kind::FieldInit => {
                let kids = self.ast.children(n).to_vec();
                match kids.iter().copied().find(|&c| !matches!(self.ast.kind(c), Kind::Attribute | Kind::DocComment)) {
                    // `S { a }`: the field's name is also a value in scope.
                    Some(name) if kids.len() == 1 && self.ast.kind(name) == Kind::Name => {
                        let s = self.ast.token(name).ident_name();
                        if let Some(res) = self.lookup_name(s, Ns::Value, true) {
                            self.out[name.0 as usize] = Some(res);
                        }
                    }
                    _ => {
                        for c in kids {
                            if !matches!(self.ast.kind(c), Kind::Name | Kind::Literal) {
                                self.walk(c);
                            }
                        }
                    }
                }
            }
            Kind::MacroCall | Kind::TypeMacro | Kind::PatMacro => {
                if let Some(p) = self.ast.child(n, Kind::Path) {
                    self.record(p, Ns::Macro);
                }
                // What expansion added to a call it left (`asm!`'s
                // operands): expressions like any other.
                for c in self.ast.children(n).to_vec() {
                    if !matches!(self.ast.kind(c), Kind::Path | Kind::TokenTree | Kind::Attribute | Kind::DocComment) {
                        self.walk(c);
                    }
                }
            }
            Kind::Block => {
                let saved = self.module;
                if let Some(&m) = self.r.module_of.get(&(self.krate, self.file, n)) {
                    self.module = m;
                }
                self.push();
                for c in self.ast.children(n).to_vec() {
                    match self.ast.kind(c) {
                        Kind::LetStmt => {
                            // The value and type first, then the bindings,
                            // which are visible after the statement.
                            let kids = self.ast.children(c).to_vec();
                            let mut pat = None;
                            for k in kids {
                                match self.ast.kind(k) {
                                    Kind::Attribute | Kind::DocComment | Kind::Keyword => {}
                                    _ if pat.is_none() => pat = Some(k),
                                    _ => self.walk(k),
                                }
                            }
                            if let Some(p) = pat {
                                self.bind_pattern(p);
                            }
                        }
                        _ => self.walk(c),
                    }
                }
                self.pop();
                self.module = saved;
            }
            Kind::If | Kind::While => {
                let kids = self.ast.children(n).to_vec();
                let start = kids.iter().position(|&k| self.ast.kind(k) != Kind::Lifetime).unwrap_or(0);
                self.push();
                self.condition(kids[start]);
                self.walk(kids[start + 1]);
                self.pop();
                for &k in &kids[start + 2..] {
                    self.walk(k);
                }
            }
            Kind::For => {
                let kids: Vec<NodeId> = self
                    .ast
                    .children(n)
                    .iter()
                    .copied()
                    .filter(|&k| !matches!(self.ast.kind(k), Kind::Lifetime | Kind::Keyword))
                    .collect();
                if let [pat, iter, body] = kids.as_slice() {
                    self.walk(*iter);
                    self.push();
                    self.bind_pattern(*pat);
                    self.walk(*body);
                    self.pop();
                }
            }
            Kind::MatchArm => {
                self.push();
                let kids = self.ast.children(n).to_vec();
                let mut seen_pat = false;
                for k in kids {
                    match self.ast.kind(k) {
                        Kind::Attribute | Kind::DocComment => {}
                        _ if !seen_pat => {
                            self.bind_pattern(k);
                            seen_pat = true;
                        }
                        Kind::Guard => {
                            let g = self.ast.children(k)[0];
                            self.condition(g);
                        }
                        _ => self.walk(k),
                    }
                }
                self.pop();
            }
            Kind::Closure => {
                self.push();
                for c in self.ast.children(n).to_vec() {
                    if self.ast.kind(c) == Kind::ClosureParams {
                        for p in self.ast.children(c).to_vec() {
                            let kids: Vec<NodeId> = self
                                .ast
                                .children(p)
                                .iter()
                                .copied()
                                .filter(|&k| !matches!(self.ast.kind(k), Kind::Attribute | Kind::DocComment))
                                .collect();
                            if let Some(&t) = kids.get(1) {
                                self.walk(t);
                            }
                            if let Some(&pat) = kids.first() {
                                self.bind_pattern(pat);
                            }
                        }
                    } else {
                        self.walk(c);
                    }
                }
                self.pop();
            }
            Kind::Let => self.condition(n),
            Kind::Field => {
                // `a.b`: only the receiver has names.
                if let Some(&e) = self.ast.children(n).first() {
                    self.walk(e);
                }
            }
            Kind::MethodCall => {
                let kids = self.ast.children(n).to_vec();
                self.walk(kids[0]);
                // The method's generic arguments, then the arguments.
                for a in self.ast.children(kids[1]).to_vec().into_iter().skip(1) {
                    self.walk(a);
                }
                self.walk(kids[2]);
            }
            _ if is_pattern(kind) => {
                let mut binds = Vec::new();
                self.pattern(n, &mut binds);
            }
            _ => {
                for c in self.ast.children(n).to_vec() {
                    self.walk(c);
                }
            }
        }
    }
}

fn is_item(k: Kind) -> bool {
    use Kind::*;
    matches!(k, Fn | Struct | Union | Enum | Trait | TraitAlias | Impl | Mod | Const | Static | TypeAlias | ExternBlock)
}

fn is_type(k: Kind) -> bool {
    use Kind::*;
    matches!(
        k,
        PathType
            | RefType
            | PtrType
            | ArrayType
            | SliceType
            | TupleType
            | ParenType
            | PatternType
            | NeverType
            | InferType
            | FnPtrType
            | ImplTraitType
            | DynTraitType
            | TypeMacro
    )
}

fn is_pattern(k: Kind) -> bool {
    use Kind::*;
    matches!(
        k,
        WildPat
            | RestPat
            | IdentPat
            | LitPat
            | RangePat
            | RefPat
            | TuplePat
            | ParenPat
            | SlicePat
            | PathPat
            | TupleStructPat
            | StructPat
            | OrPat
            | BoxPat
            | NeverPat
            | PatMacro
    )
}

impl Resolver<'_> {
    /// Every path in expressions, types and patterns, with what it resolved
    /// to (`??` if nothing): `file:line:col path -> resolution`.
    pub fn dump(&self, crates: &[ParsedCrate], sources: &crate::source::SourceMap) -> String {
        let mut out = String::new();
        for (k, c) in crates.iter().enumerate() {
            for (i, f) in c.files.iter().enumerate() {
                let mut paths = Vec::new();
                collect_path_nodes(&f.ast, f.ast.root, &mut paths);
                for p in paths {
                    let t = f.ast.token(p);
                    let at = sources.find(t.0).map_or_else(|| "<expansion>".to_string(), |s| s.locate(t.0).to_string());
                    let what = self.res_of(k, i, p).map_or_else(|| "??".to_string(), |r| self.describe(r));
                    out.push_str(&format!("{at} {} -> {what}\n", f.ast.text(p)));
                }
            }
        }
        out
    }

    /// Counts what the paths of `crates` resolved to.
    pub fn stats(&self, crates: &[ParsedCrate]) -> ResolveStats {
        let mut s = ResolveStats {
            defs: self.defs.len(),
            modules: self.modules.len(),
            imports: self.imports.len(),
            imports_failed: self.failed_imports.len(),
            ..Default::default()
        };
        let mut missing: FastMap<String, usize> = FastMap::default();
        for (k, c) in crates.iter().enumerate() {
            for (i, f) in c.files.iter().enumerate() {
                let res = &self.res[k][i];
                count_paths(&f.ast, f.ast.root, res, &mut s, &mut missing);
            }
        }
        let mut names: Vec<(String, usize)> = missing.into_iter().collect();
        names.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
        s.unresolved_names = names;
        s
    }
}

/// The path nodes under `n` that resolution looks at, in tree order.
fn collect_path_nodes(ast: &Ast, n: NodeId, out: &mut Vec<NodeId>) {
    let kind = ast.kind(n);
    if matches!(
        kind,
        Kind::Use
            | Kind::Visibility
            | Kind::ImplRestriction
            | Kind::Attribute
            | Kind::InnerAttribute
            | Kind::TokenTree
            | Kind::MacroRules
            | Kind::MacroDef
    ) {
        return;
    }
    if kind == Kind::Path {
        out.push(n);
    }
    for &c in ast.children(n) {
        collect_path_nodes(ast, c, out);
    }
}

/// Counts the paths under `n` in expressions, types and patterns (not in
/// `use`, visibilities, attributes or macro input).
fn count_paths(ast: &Ast, n: NodeId, res: &[Option<Res>], s: &mut ResolveStats, missing: &mut FastMap<String, usize>) {
    let kind = ast.kind(n);
    if matches!(
        kind,
        Kind::Use
            | Kind::Visibility
            | Kind::ImplRestriction
            | Kind::Attribute
            | Kind::InnerAttribute
            | Kind::TokenTree
            | Kind::MacroRules
            | Kind::MacroDef
    ) {
        return;
    }
    if kind == Kind::Path {
        s.paths += 1;
        match res.get(n.0 as usize).and_then(Option::as_ref) {
            Some(Res::Partial(..)) => s.partial += 1,
            Some(_) => s.resolved += 1,
            None => {
                s.unresolved += 1;
                let (_, _, segs) = segments(ast, n);
                let first = segs.first().map_or("", |x| x.0);
                *missing.entry(first.to_string()).or_default() += 1;
            }
        }
        // Paths inside generic arguments are counted on their own.
        for &c in ast.children(n) {
            count_paths(ast, c, res, s, missing);
        }
        return;
    }
    for &c in ast.children(n) {
        count_paths(ast, c, res, s, missing);
    }
}
