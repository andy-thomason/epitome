//! Collecting definitions, modules and imports from the expanded trees.

use super::{Binding, Def, DefId, DefKind, Import, ImportKind, ModId, Module, Ns, Res, Resolver, Vis};
use crate::ast::{Ast, Kind, NodeId};
use crate::hash::FastMap;
use crate::session::{ParsedCrate, has_attribute};

/// Builds the definitions, modules and imports of every crate.
pub(super) fn collect<'s>(crates: &[ParsedCrate<'s>]) -> Resolver<'s> {
    let mut r = Resolver {
        defs: Vec::new(),
        modules: Vec::new(),
        roots: Vec::new(),
        crate_names: crates.iter().map(|c| c.name.clone()).collect(),
        editions: crates.iter().map(|c| c.edition).collect(),
        preludes: vec![None; crates.len()],
        file_modules: Vec::new(),
        module_of: FastMap::default(),
        imports: Vec::new(),
        failed_imports: Vec::new(),
        prelude_imports: vec![None; crates.len()],
        no_core: vec![false; crates.len()],
        no_std: vec![false; crates.len()],
        res: Vec::new(),
        def_site: Vec::new(),
        macro_bodies: Vec::new(),
    };
    // Crate roots first, so that `extern crate` can name any of them.
    for (k, c) in crates.iter().enumerate() {
        let root = r.new_module(k as u32, None, false);
        let def = r.new_def(DefKind::Mod, "", k as u32, None, 0, NodeId(0), Vis::Public, Some(root));
        r.modules[root.0 as usize].def = Some(def);
        r.roots.push(root);
        if let Some(f) = c.files.first() {
            let ast = &f.ast;
            let inner = |name: &str| {
                ast.children(ast.root).iter().any(|&a| {
                    ast.kind(a) == Kind::InnerAttribute && ast.child(a, Kind::Path).is_some_and(|p| ast.text(p) == name)
                })
            };
            r.no_core[k] = inner("no_core");
            r.no_std[k] = inner("no_std");
        }
    }
    for (k, c) in crates.iter().enumerate() {
        let mut file_modules = vec![r.roots[k]; c.files.len()];
        for (i, f) in c.files.iter().enumerate() {
            let m = if i == 0 {
                r.roots[k]
            } else {
                match f.parent.and_then(|(pf, node)| r.module_of.get(&(k as u32, pf as u32, node)).copied()) {
                    Some(m) => m,
                    None => continue,
                }
            };
            file_modules[i] = m;
            let mut cx = Collector { r: &mut r, krate: k as u32, file: i as u32, ast: &f.ast, crates };
            cx.items(f.ast.root, m);
        }
        r.file_modules.push(file_modules);
    }
    r.def_site.sort_by_key(|x| x.0);
    r.macro_bodies.sort();
    r
}

impl<'s> Resolver<'s> {
    fn new_module(&mut self, krate: u32, parent: Option<ModId>, block: bool) -> ModId {
        let id = ModId(self.modules.len() as u32);
        let normal = match parent {
            Some(p) if block => self.modules[p.0 as usize].normal,
            _ => id,
        };
        self.modules.push(Module {
            krate,
            def: None,
            parent,
            normal,
            block,
            names: FastMap::default(),
            globs: Vec::new(),
        });
        id
    }

    #[allow(clippy::too_many_arguments)]
    fn new_def(
        &mut self,
        kind: DefKind,
        name: &'s str,
        krate: u32,
        parent: Option<ModId>,
        file: u32,
        node: NodeId,
        vis: Vis,
        module: Option<ModId>,
    ) -> DefId {
        let id = DefId(self.defs.len() as u32);
        self.defs.push(Def { kind, name, krate, parent, file, node, vis, module });
        id
    }

    /// Adds a name to a module. An earlier name in the same namespace
    /// stays, except that a later macro shadows an earlier one.
    pub(super) fn bind(&mut self, m: ModId, name: &'s str, ns: Ns, res: Res, vis: Vis) {
        if name == "_" || name.is_empty() {
            return;
        }
        let names = &mut self.modules[m.0 as usize].names;
        if ns == Ns::Macro {
            names.insert((name, ns), Binding { res, vis });
        } else {
            names.entry((name, ns)).or_insert(Binding { res, vis });
        }
    }
}

struct Collector<'c, 'r, 's> {
    r: &'r mut Resolver<'s>,
    krate: u32,
    file: u32,
    ast: &'c Ast<'s>,
    crates: &'c [ParsedCrate<'s>],
}

fn is_item(k: Kind) -> bool {
    use Kind::*;
    matches!(
        k,
        Fn | Struct
            | Union
            | Enum
            | Trait
            | TraitAlias
            | Impl
            | Mod
            | Use
            | Const
            | Static
            | TypeAlias
            | ExternCrate
            | ExternBlock
            | MacroRules
            | MacroDef
    )
}

impl<'c, 'r, 's> Collector<'c, 'r, 's> {
    fn name(&self, item: NodeId) -> &'s str {
        self.ast.child(item, Kind::Name).map_or("", |n| self.ast.token(n).ident_name())
    }

    /// The visibility an item's `pub` gives, in module `m`.
    fn vis(&self, item: NodeId, m: ModId) -> Vis {
        let normal = self.r.modules[m.0 as usize].normal;
        let Some(v) = self.ast.child(item, Kind::Visibility) else { return Vis::Restricted(normal) };
        let Some(path) = self.ast.child(v, Kind::Path) else { return Vis::Public };
        let segs: Vec<&str> = self.ast.children(path).iter().map(|&s| self.ast.token(s).0).collect();
        let root = self.r.roots[self.krate as usize];
        match segs.as_slice() {
            ["crate"] => Vis::Restricted(root),
            ["self"] => Vis::Restricted(normal),
            ["super"] => {
                let p = self.r.modules[normal.0 as usize].parent.map(|p| self.r.modules[p.0 as usize].normal);
                Vis::Restricted(p.unwrap_or(root))
            }
            // `pub(in path)`: an ancestor module, which already exists.
            _ => Vis::Restricted(self.ancestor(&segs, normal).unwrap_or(root)),
        }
    }

    /// The module `pub(in a::b)` names, from module `normal`: `crate`,
    /// `self` or `super`s, then module names (a 2015 path starts at the
    /// crate root).
    fn ancestor(&self, segs: &[&str], normal: ModId) -> Option<ModId> {
        let root = self.r.roots[self.krate as usize];
        let mut cur = root;
        let mut rest = segs;
        match *segs.first()? {
            "crate" => rest = &segs[1..],
            "self" => {
                cur = normal;
                rest = &segs[1..];
            }
            "super" => {
                cur = normal;
                while rest.first() == Some(&"super") {
                    let p = self.r.modules[cur.0 as usize].parent?;
                    cur = self.r.modules[p.0 as usize].normal;
                    rest = &rest[1..];
                }
            }
            _ => {}
        }
        for &seg in rest {
            let res = self.r.lookup(cur, seg, Ns::Type, cur)?;
            cur = self.r.module_of_res(&res)?;
        }
        Some(cur)
    }

    fn def(&mut self, kind: DefKind, name: &'s str, m: ModId, node: NodeId, vis: Vis, module: Option<ModId>) -> DefId {
        self.r.new_def(kind, name, self.krate, Some(m), self.file, node, vis, module)
    }

    /// The items under `node` (a file's root, a module, or a block), in
    /// module `m`.
    fn items(&mut self, node: NodeId, m: ModId) {
        let kids = self.ast.children(node).to_vec();
        for item in kids {
            let kind = self.ast.kind(item);
            if !is_item(kind) {
                continue;
            }
            self.item(item, kind, m);
        }
    }

    fn item(&mut self, item: NodeId, kind: Kind, m: ModId) {
        let name = self.name(item);
        let vis = self.vis(item, m);
        match kind {
            Kind::Mod => {
                let sub = self.r.new_module(self.krate, Some(m), false);
                let d = self.def(DefKind::Mod, name, m, item, vis, Some(sub));
                self.r.modules[sub.0 as usize].def = Some(d);
                self.r.module_of.insert((self.krate, self.file, item), sub);
                self.r.bind(m, name, Ns::Type, Res::Def(d), vis);
                self.items(item, sub);
            }
            Kind::Struct | Kind::Union => {
                let dk = if kind == Kind::Struct { DefKind::Struct } else { DefKind::Union };
                let d = self.def(dk, name, m, item, vis, None);
                self.r.bind(m, name, Ns::Type, Res::Def(d), vis);
                if kind == Kind::Struct && self.ast.child(item, Kind::NamedFields).is_none() {
                    // A tuple or unit struct is also its constructor.
                    self.r.bind(m, name, Ns::Value, Res::Def(d), vis);
                }
                self.blocks(item, m);
            }
            Kind::Enum => {
                let sub = self.r.new_module(self.krate, Some(m), false);
                self.r.modules[sub.0 as usize].normal = m;
                let d = self.def(DefKind::Enum, name, m, item, vis, Some(sub));
                self.r.modules[sub.0 as usize].def = Some(d);
                self.r.module_of.insert((self.krate, self.file, item), sub);
                self.r.bind(m, name, Ns::Type, Res::Def(d), vis);
                for v in self.ast.children(item).to_vec() {
                    if self.ast.kind(v) != Kind::Variant {
                        continue;
                    }
                    let vn = self.name(v);
                    let vd = self.def(DefKind::Variant, vn, sub, v, Vis::Public, None);
                    self.r.bind(sub, vn, Ns::Type, Res::Def(vd), Vis::Public);
                    if self.ast.child(v, Kind::NamedFields).is_none() {
                        self.r.bind(sub, vn, Ns::Value, Res::Def(vd), Vis::Public);
                    }
                    self.blocks(v, m);
                }
            }
            Kind::Trait => {
                let sub = self.r.new_module(self.krate, Some(m), false);
                self.r.modules[sub.0 as usize].normal = m;
                let d = self.def(DefKind::Trait, name, m, item, vis, Some(sub));
                self.r.modules[sub.0 as usize].def = Some(d);
                self.r.module_of.insert((self.krate, self.file, item), sub);
                self.r.bind(m, name, Ns::Type, Res::Def(d), vis);
                self.assoc_items(item, sub, m);
            }
            Kind::TraitAlias | Kind::TypeAlias => {
                let dk = if kind == Kind::TraitAlias { DefKind::TraitAlias } else { DefKind::TypeAlias };
                let d = self.def(dk, name, m, item, vis, None);
                self.r.bind(m, name, Ns::Type, Res::Def(d), vis);
            }
            Kind::Fn | Kind::Const | Kind::Static => {
                let dk = match kind {
                    Kind::Fn => DefKind::Fn,
                    Kind::Const => DefKind::Const,
                    _ => DefKind::Static,
                };
                let d = self.def(dk, name, m, item, vis, None);
                self.r.bind(m, name, Ns::Value, Res::Def(d), vis);
                self.blocks(item, m);
            }
            Kind::Impl => {
                let sub = self.r.new_module(self.krate, Some(m), false);
                let d = self.def(DefKind::Impl, "", m, item, vis, Some(sub));
                self.r.modules[sub.0 as usize].def = Some(d);
                self.r.modules[sub.0 as usize].normal = m;
                self.r.module_of.insert((self.krate, self.file, item), sub);
                self.assoc_items(item, sub, m);
            }
            Kind::ExternCrate => {
                let target = if name == "self" {
                    Some(self.krate as usize)
                } else {
                    self.crates.iter().position(|c| c.name == name)
                };
                let bound = self
                    .ast
                    .child(item, Kind::Rename)
                    .and_then(|rn| self.ast.child(rn, Kind::Name))
                    .map_or(name, |n| self.ast.token(n).ident_name());
                if let Some(t) = target {
                    let root = self.r.roots[t];
                    let def = self.r.modules[root.0 as usize].def.unwrap();
                    self.r.bind(m, bound, Ns::Type, Res::Def(def), vis);
                }
            }
            Kind::ExternBlock => {
                for f in self.ast.children(item).to_vec() {
                    let (dk, ns) = match self.ast.kind(f) {
                        Kind::Fn => (DefKind::ForeignFn, Ns::Value),
                        Kind::Static => (DefKind::ForeignStatic, Ns::Value),
                        Kind::TypeAlias => (DefKind::ForeignType, Ns::Type),
                        _ => continue,
                    };
                    let fname = self.name(f);
                    let fvis = self.vis(f, m);
                    let d = self.def(dk, fname, m, f, fvis, None);
                    self.r.bind(m, fname, ns, Res::Def(d), fvis);
                }
            }
            Kind::Use => {
                let Some(tree) = self.ast.child(item, Kind::UseTree) else { return };
                self.use_tree(tree, Vec::new(), false, m, vis, item);
                if has_attribute(self.ast, item, "prelude_import") {
                    self.r.prelude_imports[self.krate as usize] = Some(self.r.imports.len() - 1);
                }
            }
            Kind::MacroRules | Kind::MacroDef => {
                let exported = has_attribute(self.ast, item, "macro_export");
                let d = self.def(DefKind::Macro, name, m, item, vis, None);
                if exported {
                    let root = self.r.roots[self.krate as usize];
                    self.r.bind(root, name, Ns::Macro, Res::Def(d), Vis::Public);
                }
                // A `macro_rules!` can be named by `use` in its module (as
                // `pub(crate) use m;` does); a macro 2.0 has its own `pub`.
                let mvis =
                    if kind == Kind::MacroDef { vis } else { Vis::Restricted(self.r.modules[m.0 as usize].normal) };
                self.r.bind(m, name, Ns::Macro, Res::Def(d), mvis);
                // Only macros written in a file have a body of their own:
                // one a macro made is text of that macro's body, whose
                // context it shares (and spans of copied tokens would not
                // be one piece of text).
                let written = self.ast.is_original(item);
                for &tt in self.ast.children(item).iter().filter(|&&c| written && self.ast.kind(c) == Kind::TokenTree) {
                    let toks = self.ast.tokens_of(tt);
                    if let (Some(a), Some(b)) = (toks.first(), toks.last()) {
                        let (start, end) = (a.0.as_ptr() as usize, b.0.as_ptr() as usize + b.0.len());
                        if end > start {
                            self.r.macro_bodies.push((start, end));
                        }
                    }
                }
                if kind == Kind::MacroDef
                    && written
                    && let Some(&body) =
                        self.ast.children(item).iter().rev().find(|&&c| self.ast.kind(c) == Kind::TokenTree)
                {
                    let toks = self.ast.tokens_of(body);
                    if let (Some(a), Some(b)) = (toks.first(), toks.last()) {
                        let start = a.0.as_ptr() as usize;
                        let end = b.0.as_ptr() as usize + b.0.len();
                        if end > start {
                            self.r.def_site.push((start, end, m));
                        }
                    }
                }
            }
            _ => {}
        }
    }

    /// The items of a trait or impl, in `sub`; `m` is the enclosing
    /// module, for blocks in their bodies.
    fn assoc_items(&mut self, item: NodeId, sub: ModId, m: ModId) {
        for a in self.ast.children(item).to_vec() {
            let (dk, ns) = match self.ast.kind(a) {
                Kind::Fn => (DefKind::AssocFn, Ns::Value),
                Kind::Const => (DefKind::AssocConst, Ns::Value),
                Kind::TypeAlias => (DefKind::AssocType, Ns::Type),
                _ => continue,
            };
            let an = self.name(a);
            let d = self.def(dk, an, sub, a, Vis::Public, None);
            self.r.bind(sub, an, ns, Res::Def(d), Vis::Public);
            self.blocks(a, m);
        }
    }

    /// Finds blocks that hold items under `node` (an item's insides), and
    /// makes a module for each.
    fn blocks(&mut self, node: NodeId, m: ModId) {
        for k in self.ast.children(node).to_vec() {
            let kind = self.ast.kind(k);
            if kind.is_leaf() || is_item(kind) {
                continue;
            }
            if kind == Kind::Block && self.ast.children(k).iter().any(|&c| is_item(self.ast.kind(c))) {
                let b = self.r.new_module(self.krate, Some(m), true);
                self.r.module_of.insert((self.krate, self.file, k), b);
                self.items(k, b);
                self.blocks(k, b);
            } else {
                self.blocks(k, m);
            }
        }
    }

    /// Flattens a `use` tree into imports.
    fn use_tree(&mut self, tree: NodeId, prefix: Vec<&'s str>, rooted: bool, m: ModId, vis: Vis, item: NodeId) {
        let mut path = prefix;
        let mut rooted = rooted;
        let mut tail = None;
        for &k in self.ast.children(tree) {
            match self.ast.kind(k) {
                Kind::PathRoot => rooted = true,
                Kind::Path => {
                    for &s in self.ast.children(k) {
                        path.push(self.ast.token(s).ident_name());
                    }
                }
                Kind::UseGlob | Kind::UseGroup | Kind::Rename => tail = Some(k),
                _ => {}
            }
        }
        let edition = self.r.editions[self.krate as usize];
        let (krate, file) = (self.krate, self.file);
        let import = |r: &mut Resolver<'s>, path: Vec<&'s str>, kind: ImportKind<'s>| {
            r.imports.push(Import { module: m, krate, path, rooted, kind, vis, file, node: item, edition });
        };
        match tail.map(|t| (t, self.ast.kind(t))) {
            Some((_, Kind::UseGlob)) => import(self.r, path, ImportKind::Glob),
            Some((g, Kind::UseGroup)) => {
                for sub in self.ast.children(g).to_vec() {
                    self.use_tree(sub, path.clone(), rooted, m, vis, item);
                }
            }
            other => {
                let rename =
                    other.and_then(|(rn, _)| self.ast.child(rn, Kind::Name)).map(|n| self.ast.token(n).ident_name());
                // `a::b::{self}` names `b`.
                if path.last() == Some(&"self") && path.len() > 1 {
                    path.pop();
                }
                let Some(&last) = path.last() else { return };
                import(self.r, path, ImportKind::Single(rename.unwrap_or(last)));
            }
        }
    }
}
