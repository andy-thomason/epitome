//! Function signatures with their regions.
//!
//! `ty::collect` gives a function `fn(&T) -> &T` and no lifetimes. Here the
//! same declaration is read again from the source, keeping each lifetime
//! where it is written (and where it is not: elision):
//!
//! - the lifetime parameters, with those of the impl or trait around, and
//!   whether each is early bound (named in a bound, or not in an argument:
//!   it is a parameter of the item) or late bound (chosen at each call);
//! - each reference, lifetime argument and `dyn Trait + 'a` in the
//!   parameter and return types as a [`Region`], with elided ones numbered:
//!   each in an argument is its own, the return type's is the one of
//!   `&self`, or the only one in the arguments;
//! - `'a: 'b` and `T: 'a` bounds, and trait bounds with their arguments
//!   (`F: for<'x> Fn(&'x T) -> &'x U`).
//!
//! A type is an [`RTy`], the structure of the `Ty` with a region in each
//! place one can be; a type alias is expanded, so that its parameters are
//! the arguments given.

use crate::ast::{Ast, Kind, NodeId};
use crate::hash::FastMap;
use crate::resolve::{DefId, DefKind, Res, Resolver};
use crate::session::ParsedCrate;
use crate::ty::lower::is_type;
use crate::ty::{Mutbl, ParamId, TyCtxt};

/// A lifetime's name without its tick or `r#`: `'r#a` is `'a`.
pub(crate) fn lname(s: &str) -> &str {
    let s = s.strip_prefix('\'').unwrap_or(s);
    s.strip_prefix("r#").unwrap_or(s)
}

/// A lifetime.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum Region {
    Static,
    /// A lifetime parameter of the item (or of its impl or trait): its
    /// index in `FnRegions::lifetimes`.
    Param(u32),
    /// An elided lifetime in an argument: numbered in order of appearance.
    Elided(u32),
    /// Bound by `for<'a>` or in a function pointer type, numbered in
    /// the signature.
    Bound(u32),
    /// An elided lifetime in an impl's header (`impl Tr for &T`): a
    /// parameter of the impl, numbered in the header.
    ImplAnon(u32),
    /// An elided lifetime of the return type that has nothing to be: an
    /// error (E0106), or `'_` in a place that does not say.
    Unknown,
}

/// A generic argument.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum RArg {
    Region(Region),
    Ty(RTy),
    Const,
}

/// A trait bound: `subject: Trait<args, Assoc = ty>`.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct RBound {
    pub trait_def: DefId,
    /// The trait's arguments after the self type; for `Fn(A, B) -> C`, the
    /// tuple `(A, B)` and `Output = C`.
    pub args: Vec<RArg>,
    pub assoc: Vec<(String, RTy)>,
    /// The regions of a `for<...>` before it.
    pub binder: Vec<u32>,
}

/// A type with its regions.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum RTy {
    /// No regions in it: a primitive, `!`, a type parameter's bounds.
    Plain,
    Ref(Region, Mutbl, Box<RTy>),
    Ptr(Mutbl, Box<RTy>),
    Slice(Box<RTy>),
    Array(Box<RTy>),
    Tuple(Vec<RTy>),
    Adt(DefId, Vec<RArg>),
    /// A type parameter.
    Param(ParamId),
    /// `dyn Trait + 'r`.
    Dyn(Region, Vec<RBound>),
    /// A function pointer: the regions it binds, argument and return types.
    FnPtr(Vec<u32>, Vec<RTy>, Box<RTy>),
    /// `<T as Trait<'a>>::Assoc`: everything it mentions (invariant).
    Proj(Vec<RArg>),
    /// `impl Trait`: the regions and types its bounds mention.
    Opaque(Vec<RArg>, Vec<RBound>),
}

impl RTy {
    /// Each region in the type, in order.
    pub fn regions(&self, out: &mut Vec<Region>) {
        let args = |a: &[RArg], out: &mut Vec<Region>| {
            for x in a {
                match x {
                    RArg::Region(r) => out.push(*r),
                    RArg::Ty(t) => t.regions(out),
                    RArg::Const => {}
                }
            }
        };
        match self {
            RTy::Plain | RTy::Param(_) => {}
            RTy::Ref(r, _, t) => {
                out.push(*r);
                t.regions(out);
            }
            RTy::Ptr(_, t) | RTy::Slice(t) | RTy::Array(t) => t.regions(out),
            RTy::Tuple(ts) => ts.iter().for_each(|t| t.regions(out)),
            RTy::Adt(_, a) | RTy::Proj(a) => args(a, out),
            RTy::Dyn(r, bs) => {
                out.push(*r);
                for b in bs {
                    args(&b.args, out);
                    b.assoc.iter().for_each(|(_, t)| t.regions(out));
                }
            }
            RTy::FnPtr(_, i, o) => {
                i.iter().for_each(|t| t.regions(out));
                o.regions(out);
            }
            RTy::Opaque(a, bs) => {
                args(a, out);
                for b in bs {
                    args(&b.args, out);
                    b.assoc.iter().for_each(|(_, t)| t.regions(out));
                }
            }
        }
    }
}

/// A lifetime parameter.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LifetimeParam {
    pub name: String,
    /// Chosen at each call (it is in an argument and in no bound), not a
    /// parameter of the function's type.
    pub late: bool,
    /// Declared by the impl or trait the function is in.
    pub parent: bool,
}

/// What an item declares about regions.
#[derive(Clone, Debug, Default)]
pub struct FnRegions {
    pub lifetimes: Vec<LifetimeParam>,
    /// Argument types (`self` first, if any).
    pub inputs: Vec<RTy>,
    pub output: RTy,
    /// How many elided lifetimes the arguments have.
    pub elided: u32,
    /// How many regions `for<...>` and function pointer types bind.
    pub bound: u32,
    /// `'a: 'b`: `(a, b)`.
    pub outlives: Vec<(Region, Region)>,
    /// `T: 'a`.
    pub ty_outlives: Vec<(RTy, Region)>,
    /// `T: Trait<...>`, the bounds with a region or a type argument in them.
    pub bounds: Vec<(RTy, RBound)>,
}

impl Default for RTy {
    fn default() -> RTy {
        RTy::Plain
    }
}

struct B<'c, 's> {
    r: &'c Resolver<'s>,
    crates: &'c [ParsedCrate<'s>],
    tcx: &'c TyCtxt<'s>,
    k: usize,
    file: usize,
    /// Lifetimes in scope, innermost last.
    scope: Vec<(&'s str, Region)>,
    /// Type parameters an alias being expanded has been given.
    subst: FastMap<ParamId, RTy>,
    /// What an elided lifetime is: a fresh one (in an argument), or this.
    elide_to: Option<Region>,
    elided: u32,
    bound: u32,
    /// Regions seen in the arguments, counted for the return type's
    /// elision.
    seen: Vec<Region>,
    /// In the arguments.
    inputs: bool,
    /// Reading an impl's header: elided lifetimes are the impl's.
    header: Option<u32>,
}

impl<'c, 's> B<'c, 's> {
    fn ast(&self) -> &'c Ast<'s> {
        &self.crates[self.k].files[self.file].ast
    }

    fn fresh_bound(&mut self) -> u32 {
        self.bound += 1;
        self.bound - 1
    }

    fn elide(&mut self) -> Region {
        if let Some(n) = self.header {
            self.header = Some(n + 1);
            return Region::ImplAnon(n);
        }
        if self.inputs {
            self.elided += 1;
            let e = Region::Elided(self.elided - 1);
            self.seen.push(e);
            e
        } else {
            self.elide_to.unwrap_or(Region::Unknown)
        }
    }

    fn lifetime(&mut self, l: NodeId) -> Region {
        let name = self.ast().token(l).0;
        match name {
            "'static" => Region::Static,
            "'_" => self.elide(),
            _ => {
                let found = self.scope.iter().rev().find(|(n, _)| lname(n) == lname(name)).map(|x| x.1);
                match found {
                    Some(r) => {
                        if self.inputs {
                            self.seen.push(r);
                        }
                        r
                    }
                    None => Region::Unknown,
                }
            }
        }
    }

    /// Binds the `for<...>` lifetimes under `n`, returning their ids and
    /// how many scope entries were added.
    fn binder(&mut self, n: NodeId) -> Vec<u32> {
        let ast = self.ast();
        let mut ids = Vec::new();
        if let Some(f) = ast.child(n, Kind::ForLifetimes) {
            for &p in ast.children(f) {
                if let Some(l) = ast.child(p, Kind::Lifetime) {
                    let id = self.fresh_bound();
                    self.scope.push((ast.token(l).0, Region::Bound(id)));
                    ids.push(id);
                }
            }
        }
        ids
    }

    fn unbind(&mut self, ids: &[u32]) {
        let n = self.scope.len() - ids.len();
        self.scope.truncate(n);
    }

    fn ty(&mut self, n: NodeId, default_region: Option<Region>) -> RTy {
        let ast = self.ast();
        let kids = ast.children(n);
        match ast.kind(n) {
            Kind::PathType => match kids.first() {
                Some(&p) => self.path(p, default_region),
                None => RTy::Plain,
            },
            Kind::RefType => {
                let lt = kids.iter().copied().find(|&c| ast.kind(c) == Kind::Lifetime);
                let is_mut = kids.iter().any(|&c| ast.kind(c) == Kind::Keyword && ast.token(c).0 == "mut");
                let r = match lt {
                    Some(l) => self.lifetime(l),
                    None => self.elide(),
                };
                let inner = self.ty(*kids.last().unwrap(), Some(r));
                RTy::Ref(r, if is_mut { Mutbl::Mut } else { Mutbl::Not }, Box::new(inner))
            }
            Kind::PtrType => {
                let m = if ast.token(kids[0]).0 == "mut" { Mutbl::Mut } else { Mutbl::Not };
                RTy::Ptr(m, Box::new(self.ty(kids[1], None)))
            }
            Kind::ArrayType => RTy::Array(Box::new(self.ty(kids[0], None))),
            Kind::SliceType => RTy::Slice(Box::new(self.ty(kids[0], None))),
            Kind::PatternType => self.ty(kids[0], None),
            Kind::TupleType => RTy::Tuple(kids.iter().map(|&c| self.ty(c, None)).collect()),
            Kind::ParenType => self.ty(kids[0], default_region),
            Kind::FnPtrType => {
                let ids = self.binder(n);
                let mut inputs = Vec::new();
                let mut ret = None;
                for &c in kids {
                    match ast.kind(c) {
                        Kind::Params => {
                            inputs.extend(ast.children(c).iter().filter_map(|&p| ast.children(p).last().copied()));
                        }
                        Kind::RetType => ret = Some(ast.children(c)[0]),
                        _ => {}
                    }
                }
                let (anon, ins, out) = self.fn_like(&inputs, ret);
                self.unbind(&ids);
                let mut all = ids;
                all.extend(anon);
                RTy::FnPtr(all, ins, Box::new(out))
            }
            Kind::DynTraitType | Kind::ImplTraitType => {
                let Some(b) = ast.child(n, Kind::Bounds) else { return RTy::Plain };
                let mut region = None;
                let mut bounds = Vec::new();
                let mut regions = Vec::new();
                for &c in ast.children(b) {
                    match ast.kind(c) {
                        Kind::Lifetime => {
                            let r = self.lifetime(c);
                            region = Some(r);
                            regions.push(RArg::Region(r));
                        }
                        Kind::TraitBound => {
                            if let Some(x) = self.bound_of(c) {
                                bounds.push(x);
                            }
                        }
                        _ => {}
                    }
                }
                if ast.kind(n) == Kind::DynTraitType {
                    // `&'a dyn Trait` is `&'a (dyn Trait + 'a)`; elsewhere `'static`.
                    RTy::Dyn(region.or(default_region).unwrap_or(Region::Static), bounds)
                } else {
                    RTy::Opaque(regions, bounds)
                }
            }
            Kind::UnsafeBinderType => {
                // `unsafe<'a, 'b> T` binds its lifetimes.
                let ids: Vec<u32> = kids
                    .iter()
                    .filter(|&&c| ast.kind(c) == Kind::Lifetime)
                    .map(|&c| {
                        let id = self.fresh_bound();
                        self.scope.push((ast.token(c).0, Region::Bound(id)));
                        id
                    })
                    .collect();
                let t = self.ty(*kids.last().unwrap(), None);
                self.unbind(&ids);
                t
            }
            _ => RTy::Plain,
        }
    }

    /// The arguments and result of a function pointer type or `Fn(A) -> B`
    /// sugar: each elided lifetime in the arguments is bound by it, and the
    /// result's is the only one there is. Returns the regions it binds.
    fn fn_like(&mut self, inputs: &[NodeId], ret: Option<NodeId>) -> (Vec<u32>, Vec<RTy>, RTy) {
        let (before, seen0) = (self.elided, self.seen.len());
        let (was_inputs, was_to) = (std::mem::replace(&mut self.inputs, true), self.elide_to);
        let ins: Vec<RTy> = inputs.iter().map(|&t| self.ty(t, None)).collect();
        let made = self.elided - before;
        let ids: Vec<u32> = (0..made).map(|_| self.fresh_bound()).collect();
        self.elided = before;
        let rn = |r: Region| match r {
            Region::Elided(i) if i >= before && ((i - before) as usize) < ids.len() => Region::Bound(ids[(i - before) as usize]),
            r => r,
        };
        let mut distinct: Vec<Region> = Vec::new();
        for &s in &self.seen[seen0..] {
            let s = rn(s);
            if s != Region::Static && !distinct.contains(&s) {
                distinct.push(s);
            }
        }
        self.seen.truncate(seen0);
        let ins: Vec<RTy> = ins.into_iter().map(|t| rename(t, before, &ids)).collect();
        self.inputs = false;
        self.elide_to = (distinct.len() == 1).then(|| distinct[0]);
        let out = match ret {
            Some(t) => self.ty(t, None),
            None => RTy::Tuple(Vec::new()),
        };
        self.inputs = was_inputs;
        self.elide_to = was_to;
        (ids, ins, out)
    }

    fn args(&mut self, g: Option<NodeId>, d: DefId) -> Vec<RArg> {
        let ast = self.ast();
        let (mut lts, mut tys) = (Vec::new(), Vec::new());
        for &a in g.map(|g| ast.children(g)).unwrap_or(&[]) {
            match ast.kind(a) {
                Kind::Lifetime | Kind::AssocConstraint => {
                    if ast.kind(a) == Kind::Lifetime {
                        lts.push(a);
                    }
                }
                _ => tys.push(a),
            }
        }
        let decl = self.declared(d);
        let (mut li, mut ti) = (0, 0);
        let mut out = Vec::new();
        for kind in decl {
            match kind {
                Kind::LifetimeParam => {
                    let r = match lts.get(li) {
                        Some(&l) => self.lifetime(l),
                        // A lifetime argument left out is elided.
                        None => self.elide(),
                    };
                    li += 1;
                    out.push(RArg::Region(r));
                }
                Kind::TypeParam => {
                    out.push(match tys.get(ti) {
                        Some(&t) => RArg::Ty(self.ty(t, None)),
                        None => RArg::Ty(RTy::Plain),
                    });
                    ti += 1;
                }
                _ => {
                    ti += 1;
                    out.push(RArg::Const);
                }
            }
        }
        out
    }

    /// The kinds of the parameters an item declares.
    fn declared(&self, d: DefId) -> Vec<Kind> {
        let def = &self.r.defs[d.0 as usize];
        let ast = &self.crates[def.krate as usize].files[def.file as usize].ast;
        let Some(g) = ast.child(def.node, Kind::Generics) else { return Vec::new() };
        ast.children(g).iter().map(|&p| ast.kind(p)).filter(|k| matches!(k, Kind::LifetimeParam | Kind::TypeParam | Kind::ConstParam)).collect()
    }

    fn path(&mut self, p: NodeId, default_region: Option<Region>) -> RTy {
        let ast = self.ast();
        let mentioned = |this: &mut Self| -> RTy { RTy::Proj(this.mentioned(p)) };
        if ast.child(p, Kind::QSelf).is_some() {
            return mentioned(self);
        }
        let Some(res) = self.r.res_of(self.k, self.file, p).cloned() else { return RTy::Plain };
        let last = ast.children(p).iter().rev().copied().find(|&s| ast.kind(s) == Kind::Segment);
        let g = last.and_then(|s| ast.child(s, Kind::GenericArgs));
        match res {
            Res::Generic(n) => match self.tcx.param_at.get(&(self.k as u32, self.file as u32, n)) {
                Some(pid) if self.tcx.params[pid.0 as usize].is_const => RTy::Plain,
                Some(pid) => self.subst.get(pid).cloned().unwrap_or(RTy::Param(*pid)),
                None => RTy::Plain,
            },
            Res::SelfTy(f, n) => self.self_ty(f as usize, n),
            Res::Partial(..) => mentioned(self),
            Res::Def(d) => match self.r.defs[d.0 as usize].kind {
                DefKind::Struct | DefKind::Enum | DefKind::Union => RTy::Adt(d, self.args(g, d)),
                DefKind::TypeAlias => self.alias(d, g, default_region),
                DefKind::Trait | DefKind::TraitAlias => {
                    // A bare trait object.
                    match self.bound_of_path(p) {
                        Some(b) => RTy::Dyn(default_region.unwrap_or(Region::Static), vec![b]),
                        None => RTy::Plain,
                    }
                }
                _ => RTy::Plain,
            },
            _ => RTy::Plain,
        }
    }

    /// An alias, expanded: its aliased type with its parameters the
    /// arguments given.
    fn alias(&mut self, d: DefId, g: Option<NodeId>, default_region: Option<Region>) -> RTy {
        let args = self.args(g, d);
        let def = &self.r.defs[d.0 as usize];
        let (k, f) = (def.krate as usize, def.file as usize);
        let ast = &self.crates[k].files[f].ast;
        let Some(&target) = ast.children(def.node).iter().find(|&&t| is_type(ast.kind(t))) else { return RTy::Plain };
        let Some(gn) = ast.child(def.node, Kind::Generics) else { return self.in_file(k, f, |s| s.ty(target, default_region)) };
        // Its parameters: the lifetimes by name, the types by parameter.
        let mut scope = Vec::new();
        let mut subst = FastMap::default();
        let mut it = args.into_iter();
        for &p in ast.children(gn) {
            let kind = ast.kind(p);
            if !matches!(kind, Kind::LifetimeParam | Kind::TypeParam | Kind::ConstParam) {
                continue;
            }
            let a = it.next();
            match (kind, a) {
                (Kind::LifetimeParam, Some(RArg::Region(r))) => {
                    if let Some(l) = ast.child(p, Kind::Lifetime) {
                        scope.push((ast.token(l).0, r));
                    }
                }
                (Kind::TypeParam, Some(RArg::Ty(t))) => {
                    if let Some(&pid) = self.tcx.param_at.get(&(k as u32, f as u32, p)) {
                        subst.insert(pid, t);
                    }
                }
                _ => {}
            }
        }
        let saved = (std::mem::replace(&mut self.scope, scope), std::mem::replace(&mut self.subst, subst));
        // (The alias's own elided lifetimes are the use's.)
        let t = self.in_file(k, f, |s| s.ty(target, default_region));
        self.scope = saved.0;
        self.subst = saved.1;
        t
    }

    fn in_file<T>(&mut self, k: usize, f: usize, run: impl FnOnce(&mut Self) -> T) -> T {
        let was = (self.k, self.file);
        (self.k, self.file) = (k, f);
        let t = run(self);
        (self.k, self.file) = was;
        t
    }

    /// `Self` in an impl is the type it is for; in a trait, its parameter.
    fn self_ty(&mut self, f: usize, n: NodeId) -> RTy {
        let ast = &self.crates[self.k].files[f].ast;
        match ast.kind(n) {
            Kind::Impl | Kind::Delegation => {
                let t = ast.children(n).iter().rev().copied().find(|&c| is_type(ast.kind(c)));
                match t {
                    // Written in the impl's own scope: its lifetimes.
                    Some(t) => {
                        // (Its elided lifetimes are the impl's own.)
                        let was = self.header.replace(0);
                        let r = self.in_file(self.k, f, |s| s.ty(t, None));
                        self.header = was;
                        r
                    }
                    None => RTy::Plain,
                }
            }
            _ => {
                // A trait's or an ADT's `Self`: the parameter or the type
                // with its own parameters.
                match self.tcx.param_at.get(&(self.k as u32, f as u32, n)) {
                    Some(&p) => RTy::Param(p),
                    None => RTy::Plain,
                }
            }
        }
    }

    /// Everything the path names: its lifetimes, and the types it is given.
    fn mentioned(&mut self, n: NodeId) -> Vec<RArg> {
        let ast = self.ast();
        let mut out = Vec::new();
        self.walk_mentioned(n, &mut out);
        let _ = ast;
        out
    }

    fn walk_mentioned(&mut self, n: NodeId, out: &mut Vec<RArg>) {
        let ast = self.ast();
        match ast.kind(n) {
            Kind::Lifetime => {
                let r = self.lifetime(n);
                out.push(RArg::Region(r));
            }
            k if is_type(k) && k != Kind::PathType => {
                let t = self.ty(n, None);
                out.push(RArg::Ty(t));
            }
            Kind::PathType => {
                let is_trait = ast.children(n).first().is_some_and(|&p| {
                    matches!(self.r.res_of(self.k, self.file, p), Some(Res::Def(d)) if matches!(self.r.defs[d.0 as usize].kind, DefKind::Trait | DefKind::TraitAlias))
                });
                if is_trait {
                    // `<T as Trait<'a>>::X`: the trait's arguments.
                    for &c in ast.children(n).to_vec().iter() {
                        self.walk_mentioned(c, out);
                    }
                } else {
                    let t = self.ty(n, None);
                    out.push(RArg::Ty(t));
                }
            }
            _ => {
                for &c in ast.children(n).to_vec().iter() {
                    self.walk_mentioned(c, out);
                }
            }
        }
    }

    fn bound_of(&mut self, tb: NodeId) -> Option<RBound> {
        let ast = self.ast();
        let path = ast.child(tb, Kind::Path)?;
        let ids = self.binder(tb);
        let mut b = self.bound_of_path(path);
        if let Some(x) = &mut b {
            x.binder = ids.clone();
        }
        self.unbind(&ids);
        b
    }

    fn bound_of_path(&mut self, path: NodeId) -> Option<RBound> {
        let ast = self.ast();
        let Some(Res::Def(d)) = self.r.res_of(self.k, self.file, path).cloned() else { return None };
        if !matches!(self.r.defs[d.0 as usize].kind, DefKind::Trait | DefKind::TraitAlias) {
            return None;
        }
        let seg = ast.children(path).iter().rev().copied().find(|&s| ast.kind(s) == Kind::Segment)?;
        let mut out = RBound { trait_def: d, args: Vec::new(), assoc: Vec::new(), binder: Vec::new() };
        for &c in ast.children(seg) {
            match ast.kind(c) {
                Kind::GenericArgs => {
                    for &a in ast.children(c) {
                        match ast.kind(a) {
                            Kind::Lifetime => {
                                let r = self.lifetime(a);
                                out.args.push(RArg::Region(r));
                            }
                            Kind::AssocConstraint => {
                                let kids = ast.children(a);
                                let name = ast.token(kids[0]).0.to_string();
                                if let Some(&v) = kids.iter().rev().find(|&&v| is_type(ast.kind(v))) {
                                    let t = self.ty(v, None);
                                    out.assoc.push((name, t));
                                }
                            }
                            Kind::Literal | Kind::Unary | Kind::Block => out.args.push(RArg::Const),
                            _ => {
                                let t = self.ty(a, None);
                                out.args.push(RArg::Ty(t));
                            }
                        }
                    }
                }
                // `Fn(A, B) -> C`: arguments are a tuple; the output is
                // the elided lifetimes' own (they are bound by the bound).
                Kind::FnSugar => {
                    let ins: Vec<NodeId> = ast.children(c).iter().copied().filter(|&x| is_type(ast.kind(x))).collect();
                    let ret = ast.child(c, Kind::RetType).map(|x| ast.children(x)[0]);
                    let (anon, ins, ret) = self.fn_like(&ins, ret);
                    out.binder.extend(anon);
                    out.args.push(RArg::Ty(RTy::Tuple(ins)));
                    out.assoc.push(("Output".to_string(), ret));
                }
                _ => {}
            }
        }
        Some(out)
    }
}

/// `ty` with its elided regions numbered from `first` bound to `ids`.
fn rename(t: RTy, first: u32, ids: &[u32]) -> RTy {
    let reg = |r: Region| match r {
        Region::Elided(i) if i >= first && ((i - first) as usize) < ids.len() => Region::Bound(ids[(i - first) as usize]),
        r => r,
    };
    let arg = |a: RArg| match a {
        RArg::Region(r) => RArg::Region(reg(r)),
        RArg::Ty(t) => RArg::Ty(rename(t, first, ids)),
        RArg::Const => RArg::Const,
    };
    let b = |x: RBound| RBound {
        args: x.args.into_iter().map(arg).collect(),
        assoc: x.assoc.into_iter().map(|(n, t)| (n, rename(t, first, ids))).collect(),
        ..x
    };
    match t {
        RTy::Ref(r, m, t) => RTy::Ref(reg(r), m, Box::new(rename(*t, first, ids))),
        RTy::Ptr(m, t) => RTy::Ptr(m, Box::new(rename(*t, first, ids))),
        RTy::Slice(t) => RTy::Slice(Box::new(rename(*t, first, ids))),
        RTy::Array(t) => RTy::Array(Box::new(rename(*t, first, ids))),
        RTy::Tuple(ts) => RTy::Tuple(ts.into_iter().map(|t| rename(t, first, ids)).collect()),
        RTy::Adt(d, a) => RTy::Adt(d, a.into_iter().map(arg).collect()),
        RTy::Proj(a) => RTy::Proj(a.into_iter().map(arg).collect()),
        RTy::Dyn(r, bs) => RTy::Dyn(reg(r), bs.into_iter().map(b).collect()),
        RTy::FnPtr(bd, i, o) => {
            RTy::FnPtr(bd, i.into_iter().map(|t| rename(t, first, ids)).collect(), Box::new(rename(*o, first, ids)))
        }
        RTy::Opaque(a, bs) => RTy::Opaque(a.into_iter().map(arg).collect(), bs.into_iter().map(b).collect()),
        t => t,
    }
}

/// The lifetime parameters an item declares, in order, with the node of
/// each and its bounds (`'a: 'b + 'c`).
fn lifetime_params<'a>(ast: &'a Ast, node: NodeId) -> Vec<(&'a str, NodeId)> {
    let Some(g) = ast.child(node, Kind::Generics) else { return Vec::new() };
    ast.children(g)
        .iter()
        .filter(|&&p| ast.kind(p) == Kind::LifetimeParam)
        .filter_map(|&p| Some((ast.token(ast.child(p, Kind::Lifetime)?).0, p)))
        .collect()
}

/// The impl or trait an associated item is in.
fn parent_of(r: &Resolver, d: DefId) -> Option<DefId> {
    let def = &r.defs[d.0 as usize];
    if !matches!(def.kind, DefKind::AssocFn | DefKind::AssocConst | DefKind::AssocType) {
        return None;
    }
    let m = def.parent?;
    let p = r.modules[m.0 as usize].def?;
    matches!(r.defs[p.0 as usize].kind, DefKind::Impl | DefKind::Trait).then_some(p)
}

/// Whether the lifetime named `name` appears under `n`.
fn mentions(ast: &Ast, n: NodeId, name: &str) -> bool {
    (ast.kind(n) == Kind::Lifetime && lname(ast.token(n).0) == lname(name)) || ast.children(n).iter().any(|&c| mentions(ast, c, name))
}

/// `'a: 'b`, `T: 'a`, and trait bounds, from the generics and `where`
/// clause of `node`, into `out`.
fn predicates(b: &mut B, node: NodeId, out: &mut FnRegions) {
    let ast = b.ast();
    let region_of = |b: &mut B, l: NodeId| b.lifetime(l);
    if let Some(g) = ast.child(node, Kind::Generics) {
        for &p in ast.children(g) {
            match ast.kind(p) {
                Kind::LifetimeParam => {
                    let Some(l) = ast.child(p, Kind::Lifetime) else { continue };
                    let a = region_of(b, l);
                    if let Some(bs) = ast.child(p, Kind::Bounds) {
                        for &x in ast.children(bs) {
                            if ast.kind(x) == Kind::Lifetime {
                                let o = region_of(b, x);
                                out.outlives.push((a, o));
                            }
                        }
                    }
                }
                Kind::TypeParam => {
                    let Some(&pid) = b.tcx.param_at.get(&(b.k as u32, b.file as u32, p)) else { continue };
                    if let Some(bs) = ast.child(p, Kind::Bounds) {
                        bounds(b, RTy::Param(pid), bs, out);
                    }
                }
                _ => {}
            }
        }
    }
    if let Some(w) = ast.child(node, Kind::WhereClause) {
        for &pred in ast.children(w) {
            let kids = ast.children(pred);
            let Some(bs) = kids.iter().copied().find(|&c| ast.kind(c) == Kind::Bounds) else { continue };
            let ids = b.binder(pred);
            if let Some(l) = kids.iter().copied().find(|&c| ast.kind(c) == Kind::Lifetime) {
                let a = region_of(b, l);
                for &x in ast.children(bs) {
                    if ast.kind(x) == Kind::Lifetime {
                        let o = region_of(b, x);
                        out.outlives.push((a, o));
                    }
                }
            } else if let Some(t) = kids.iter().copied().find(|&c| is_type(ast.kind(c))) {
                let ty = b.ty(t, None);
                bounds(b, ty, bs, out);
            }
            b.unbind(&ids);
        }
    }
}

fn bounds(b: &mut B, subject: RTy, bs: NodeId, out: &mut FnRegions) {
    let ast = b.ast();
    for &x in ast.children(bs) {
        match ast.kind(x) {
            Kind::Lifetime => {
                let r = b.lifetime(x);
                out.ty_outlives.push((subject.clone(), r));
            }
            Kind::TraitBound => {
                if let Some(t) = b.bound_of(x) {
                    out.bounds.push((subject.clone(), t));
                }
            }
            _ => {}
        }
    }
}

/// Whether an item's lifetime is late bound: in an argument, and in no
/// bound or `where` clause.
pub(crate) fn is_late(ast: &Ast, node: NodeId, name: &str, p: NodeId) -> bool {
    let in_args = ast.child(node, Kind::Params).is_some_and(|ps| mentions(ast, ps, name));
    // Bounds on any parameter (`'a: 'b`, `T: 'a`), and the `where` clause.
    let in_bounds = ast.child(node, Kind::Generics).is_some_and(|g| {
        ast.children(g).iter().any(|&q| {
            if q == p {
                ast.child(q, Kind::Bounds).is_some()
            } else {
                matches!(ast.kind(q), Kind::LifetimeParam | Kind::TypeParam | Kind::ConstParam) && mentions(ast, q, name)
            }
        })
    });
    let in_where = ast.child(node, Kind::WhereClause).is_some_and(|w| mentions(ast, w, name));
    in_args && !in_bounds && !in_where
}

/// A function's signature with regions; `None` for what is not a function.
pub fn fn_regions<'s>(crates: &[ParsedCrate<'s>], r: &Resolver<'s>, tcx: &TyCtxt<'s>, d: DefId) -> Option<FnRegions> {
    let def = &r.defs[d.0 as usize];
    if !matches!(def.kind, DefKind::Fn | DefKind::AssocFn | DefKind::ForeignFn) {
        return None;
    }
    let (k, f) = (def.krate as usize, def.file as usize);
    let ast = &crates[k].files[f].ast;
    let node = def.node;
    let mut b = B {
        r,
        crates,
        tcx,
        k,
        file: f,
        scope: Vec::new(),
        subst: FastMap::default(),
        elide_to: None,
        elided: 0,
        bound: 0,
        seen: Vec::new(),
        inputs: true,
        header: None,
    };
    let mut out = FnRegions::default();
    // The impl's or trait's lifetimes (always parameters of the item), then its own.
    if let Some(p) = parent_of(r, d) {
        let pd = &r.defs[p.0 as usize];
        let past = &crates[pd.krate as usize].files[pd.file as usize].ast;
        for (name, _) in lifetime_params(past, pd.node) {
            let i = out.lifetimes.len() as u32;
            out.lifetimes.push(LifetimeParam { name: name.to_string(), late: false, parent: true });
            b.scope.push((name, Region::Param(i)));
        }
    }
    for (name, p) in lifetime_params(ast, node) {
        let i = out.lifetimes.len() as u32;
        out.lifetimes.push(LifetimeParam { name: name.to_string(), late: is_late(ast, node, name, p), parent: false });
        b.scope.push((name, Region::Param(i)));
    }
    // The arguments.
    let mut self_region = None;
    if let Some(ps) = ast.child(node, Kind::Params) {
        for &p in ast.children(ps) {
            match ast.kind(p) {
                Kind::SelfParam => {
                    // `&'a self`, `&mut self`, `self: &Self`, or `self`.
                    let kids = ast.children(p);
                    let is_ref = kids.iter().any(|&c| ast.kind(c) == Kind::Op && ast.token(c).0 == "&");
                    let typed = kids.iter().copied().find(|&c| is_type(ast.kind(c)));
                    let before = b.seen.len();
                    let ty = if let Some(t) = typed {
                        b.ty(t, None)
                    } else {
                        let is_mut = kids.iter().any(|&c| ast.kind(c) == Kind::Keyword && ast.token(c).0 == "mut") && is_ref;
                        let me = match parent_of(r, d) {
                            Some(pp) => {
                                let ppd = &r.defs[pp.0 as usize];
                                let (pk, pf) = (ppd.krate as usize, ppd.file as usize);
                                b.self_ty_of(pk, pf, ppd.node)
                            }
                            None => RTy::Plain,
                        };
                        if is_ref {
                            let reg = match kids.iter().copied().find(|&c| ast.kind(c) == Kind::Lifetime) {
                                Some(l) => b.lifetime(l),
                                None => b.elide(),
                            };
                            RTy::Ref(reg, if is_mut { Mutbl::Mut } else { Mutbl::Not }, Box::new(me))
                        } else {
                            me
                        }
                    };
                    // `self` by reference gives the return type's lifetime; its
                    // other lifetimes are not candidates.
                    let gave = self_region;
                    if self_region.is_none() {
                        if let RTy::Ref(reg, _, _) = &ty {
                            self_region = Some(*reg);
                        } else if let Some(&reg) = b.seen.get(before..).and_then(|s| s.first()) {
                            // `self: Pin<&mut Self>`, `Box<&Self>`.
                            if typed.is_some() && matches!(ty, RTy::Adt(..)) {
                                self_region = Some(reg);
                            }
                        }
                    }
                    if self_region == gave {
                        b.seen.truncate(before);
                    }
                    out.inputs.push(ty);
                }
                Kind::Param => {
                    if let Some(&t) = ast.children(p).last() {
                        out.inputs.push(b.ty(t, None));
                    }
                }
                _ => {}
            }
        }
    }
    // What the return type's elided lifetime is.
    b.inputs = false;
    b.elide_to = self_region.or_else(|| {
        let mut distinct: Vec<Region> = Vec::new();
        for &s in &b.seen {
            if !distinct.contains(&s) {
                distinct.push(s);
            }
        }
        // Static ones do not count.
        distinct.retain(|&x| x != Region::Static);
        (distinct.len() == 1).then(|| distinct[0])
    });
    out.output = match ast.child(node, Kind::RetType) {
        Some(rt) => b.ty(ast.children(rt)[0], None),
        None => RTy::Tuple(Vec::new()),
    };
    // Bounds are written in terms of the declared lifetimes.
    b.inputs = false;
    b.elide_to = None;
    predicates(&mut b, node, &mut out);
    if let Some(p) = parent_of(r, d) {
        let pd = &r.defs[p.0 as usize];
        let (saved_k, saved_f) = (b.k, b.file);
        (b.k, b.file) = (pd.krate as usize, pd.file as usize);
        predicates(&mut b, pd.node, &mut out);
        (b.k, b.file) = (saved_k, saved_f);
    }
    out.elided = b.elided;
    out.bound = b.bound;
    Some(out)
}

impl<'c, 's> B<'c, 's> {
    fn self_ty_of(&mut self, k: usize, f: usize, node: NodeId) -> RTy {
        let saved = std::mem::replace(&mut self.k, k);
        let r = self.self_ty(f, node);
        self.k = saved;
        r
    }
}

/// The field types of a struct, union or enum, with their regions: one list
/// per variant. The lifetimes are the ADT's own, numbered in order.
pub fn adt_fields<'s>(crates: &[ParsedCrate<'s>], r: &Resolver<'s>, tcx: &TyCtxt<'s>, d: DefId) -> Option<Vec<Vec<RTy>>> {
    let def = &r.defs[d.0 as usize];
    if !matches!(def.kind, DefKind::Struct | DefKind::Enum | DefKind::Union) {
        return None;
    }
    let (k, f) = (def.krate as usize, def.file as usize);
    let ast = &crates[k].files[f].ast;
    let mut b = B {
        r,
        crates,
        tcx,
        k,
        file: f,
        scope: Vec::new(),
        subst: FastMap::default(),
        elide_to: None,
        elided: 0,
        bound: 0,
        seen: Vec::new(),
        inputs: false,
        header: None,
    };
    for (i, (name, _)) in lifetime_params(ast, def.node).into_iter().enumerate() {
        b.scope.push((name, Region::Param(i as u32)));
    }
    let owners: Vec<NodeId> = if def.kind == DefKind::Enum {
        ast.children(def.node).iter().copied().filter(|&v| ast.kind(v) == Kind::Variant).collect()
    } else {
        vec![def.node]
    };
    let mut out = Vec::new();
    for o in owners {
        let mut fs = Vec::new();
        for &c in ast.children(o) {
            if matches!(ast.kind(c), Kind::NamedFields | Kind::TupleFields) {
                for &x in ast.children(c) {
                    if let Some(&t) = ast.children(x).iter().find(|&&t| is_type(ast.kind(t))) {
                        fs.push(b.ty(t, None));
                    }
                }
            }
        }
        out.push(fs);
    }
    Some(out)
}

impl RTy {
    /// The type as Rust writes it, with the regions it has.
    pub fn show(&self, r: &Resolver, tcx: &TyCtxt, names: &[LifetimeParam]) -> String {
        let reg = |x: Region| match x {
            Region::Static => "'static".to_string(),
            Region::Param(i) => names.get(i as usize).map_or("'?p".to_string(), |l| l.name.clone()),
            Region::Elided(i) => format!("'_{i}"),
            Region::Bound(i) => format!("'^{i}"),
            Region::ImplAnon(i) => format!("'_impl{i}"),
            Region::Unknown => "'?".to_string(),
        };
        let args = |a: &[RArg]| -> String {
            a.iter()
                .map(|x| match x {
                    RArg::Region(g) => reg(*g),
                    RArg::Ty(t) => t.show(r, tcx, names),
                    RArg::Const => "N".to_string(),
                })
                .collect::<Vec<_>>()
                .join(", ")
        };
        let bound = |b: &RBound| {
            let mut a = args(&b.args);
            for (n, t) in &b.assoc {
                if !a.is_empty() {
                    a.push_str(", ");
                }
                a.push_str(&format!("{n} = {}", t.show(r, tcx, names)));
            }
            format!("{}<{a}>", r.defs[b.trait_def.0 as usize].name)
        };
        match self {
            RTy::Plain => "_".to_string(),
            RTy::Ref(g, m, t) => {
                format!("&{} {}{}", reg(*g), if *m == Mutbl::Mut { "mut " } else { "" }, t.show(r, tcx, names))
            }
            RTy::Ptr(m, t) => format!("*{} {}", if *m == Mutbl::Mut { "mut" } else { "const" }, t.show(r, tcx, names)),
            RTy::Slice(t) => format!("[{}]", t.show(r, tcx, names)),
            RTy::Array(t) => format!("[{}; N]", t.show(r, tcx, names)),
            RTy::Tuple(ts) => format!("({})", ts.iter().map(|t| t.show(r, tcx, names)).collect::<Vec<_>>().join(", ")),
            RTy::Adt(d, a) => format!("{}<{}>", r.defs[d.0 as usize].name, args(a)),
            RTy::Param(p) => tcx.params[p.0 as usize].name.to_string(),
            RTy::Dyn(g, bs) => format!("dyn {} + {}", bs.iter().map(bound).collect::<Vec<_>>().join(" + "), reg(*g)),
            RTy::FnPtr(bd, i, o) => format!(
                "for<{}> fn({}) -> {}",
                bd.iter().map(|b| format!("'^{b}")).collect::<Vec<_>>().join(", "),
                i.iter().map(|t| t.show(r, tcx, names)).collect::<Vec<_>>().join(", "),
                o.show(r, tcx, names)
            ),
            RTy::Proj(a) => format!("proj<{}>", args(a)),
            RTy::Opaque(a, bs) => {
                format!("impl {} [{}]", bs.iter().map(bound).collect::<Vec<_>>().join(" + "), args(a))
            }
        }
    }
}

impl FnRegions {
    /// The signature as Rust writes it, with the regions it has:
    /// `fn<'a, 'b(late)>(&'a T, &'_0 u8) -> &'a u8 where 'a: 'b`.
    pub fn show(&self, r: &Resolver, tcx: &TyCtxt) -> String {
        let n = &self.lifetimes;
        let own: Vec<String> =
            n.iter().map(|l| format!("{}{}{}", l.name, if l.late { "(late)" } else { "" }, if l.parent { "(parent)" } else { "" })).collect();
        let reg = |x: Region| RTy::Ref(x, Mutbl::Not, Box::new(RTy::Plain)).show(r, tcx, n).trim_start_matches('&').trim_end_matches(" _").to_string();
        let mut s = format!(
            "fn<{}>({}) -> {}",
            own.join(", "),
            self.inputs.iter().map(|t| t.show(r, tcx, n)).collect::<Vec<_>>().join(", "),
            self.output.show(r, tcx, n)
        );
        let mut wh: Vec<String> = self.outlives.iter().map(|(a, b)| format!("{}: {}", reg(*a), reg(*b))).collect();
        wh.extend(self.ty_outlives.iter().map(|(t, g)| format!("{}: {}", t.show(r, tcx, n), reg(*g))));
        if !wh.is_empty() {
            s.push_str(&format!(" where {}", wh.join(", ")));
        }
        s
    }
}
