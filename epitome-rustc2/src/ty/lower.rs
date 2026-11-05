//! Turning types written in the source into `Ty`s.
//!
//! A path type's meaning comes from name resolution (`Resolver::res_of`):
//! a struct, enum or union with its generic arguments (missing ones take
//! their defaults); a type alias, expanded; a primitive; a generic
//! parameter; `Self`. A path that went past a type (`T::Item`, `Self::Item`,
//! `<T as Tr>::Item`) is an associated type of a trait that bounds the type:
//! a projection.

use super::{Abi, FloatTy, IntTy, UintTy};
use super::{Const, DynData, FnSig, Mutbl, OpaqueDef, ParamId, TraitRef, Ty, TyCtxt, TyKind};
use crate::ast::{Ast, Kind, NodeId};
use crate::hash::FastSet;
use crate::resolve::{DefId, DefKind, Ns, Res, Resolver};
use crate::session::ParsedCrate;

/// Where a node is: crate, file, node.
pub(crate) type At = (u32, u32, NodeId);

/// What lowering needs to know besides the tree.
pub(crate) struct Env<'c, 's> {
    pub r: &'c Resolver<'s>,
    pub crates: &'c [ParsedCrate<'s>],
}

impl<'c, 's> Env<'c, 's> {
    pub fn ast(&self, k: u32, file: u32) -> &'c Ast<'s> {
        &self.crates[k as usize].files[file as usize].ast
    }

    pub fn at_of(&self, d: DefId) -> At {
        let def = &self.r.defs[d.0 as usize];
        (def.krate, def.file, def.node)
    }
}

/// Lowering types in one place: an item of one file.
pub(crate) struct Lower<'l, 'c, 's> {
    pub tcx: &'l TyCtxt<'s>,
    pub env: &'l Env<'c, 's>,
    /// Aliases being expanded, to stop a cycle.
    pub expanding: FastSet<DefId>,
    /// `Self` of an impl whose header is being lowered (its trait may name
    /// `Self`), before it is in `Env::self_ty`.
    pub impl_self: Option<(At, Ty)>,
    /// In a body, `_` is a hole for inference (`Infer(u32::MAX)`, which
    /// the checker replaces); in a signature it is an error.
    pub holes: bool,
    pub k: u32,
    pub file: u32,
    /// The item whose signature this is (for `impl Trait`).
    pub owner: DefId,
    /// Whether a type came out as an error.
    pub failed: bool,
}

impl<'l, 'c, 's> Lower<'l, 'c, 's> {
    fn ast(&self) -> &'c Ast<'s> {
        self.env.ast(self.k, self.file)
    }

    fn err(&mut self, why: &str) -> Ty {
        self.failed = true;
        self.tcx.note_error(why);
        self.tcx.error()
    }

    fn res(&self, path: NodeId) -> Option<&'c Res> {
        let r: &'c Resolver<'s> = self.env.r;
        r.res_of(self.k as usize, self.file as usize, path)
    }

    /// The type a type node stands for.
    pub fn ty(&mut self, n: NodeId) -> Ty {
        let ast = self.ast();
        let kids: Vec<NodeId> = ast.children(n).to_vec();
        match ast.kind(n) {
            Kind::PathType => match kids.first() {
                Some(&p) => self.path_ty(p),
                None => self.err("empty path type"),
            },
            Kind::RefType => {
                let mutbl = if kids.iter().any(|&c| ast.kind(c) == Kind::Keyword) { Mutbl::Mut } else { Mutbl::Not };
                let inner = *kids.last().unwrap();
                let t = self.ty(inner);
                self.tcx.mk(TyKind::Ref(mutbl, t))
            }
            Kind::PtrType => {
                let mutbl = if ast.token(kids[0]).0 == "mut" { Mutbl::Mut } else { Mutbl::Not };
                let t = self.ty(kids[1]);
                self.tcx.mk(TyKind::Ptr(mutbl, t))
            }
            Kind::ArrayType => {
                let t = self.ty(kids[0]);
                let c = self.konst(kids[1]);
                self.tcx.mk(TyKind::Array(t, c))
            }
            Kind::SliceType => {
                let t = self.ty(kids[0]);
                self.tcx.mk(TyKind::Slice(t))
            }
            Kind::TupleType => {
                let tys = kids.iter().map(|&c| self.ty(c)).collect();
                let l = self.tcx.list(tys);
                self.tcx.mk(TyKind::Tuple(l))
            }
            Kind::ParenType => self.ty(kids[0]),
            Kind::NeverType => self.tcx.mk(TyKind::Never),
            Kind::InferType if self.holes => self.tcx.mk(TyKind::Infer(u32::MAX)),
            Kind::InferType => self.err("`_` in a signature"),
            Kind::FnPtrType => self.fn_ptr(n),
            Kind::ImplTraitType => {
                let bounds = kids.first().map(|&b| self.bounds(b)).unwrap_or_default();
                let id = self.tcx.new_opaque(OpaqueDef { owner: self.owner, bounds });
                self.tcx.mk(TyKind::Opaque(id))
            }
            Kind::DynTraitType => {
                let b = kids.iter().copied().find(|&c| ast.kind(c) == Kind::Bounds);
                let traits = b.map(|b| self.bounds(b)).unwrap_or_default();
                if traits.is_empty() {
                    return self.err("`dyn` with no trait");
                }
                let d = self.tcx.dyn_(DynData { traits });
                self.tcx.mk(TyKind::Dyn(d))
            }
            Kind::TypeMacro => self.err("unexpanded macro in a type"),
            _ => self.err("not a type"),
        }
    }

    /// A constant in a type: an array length.
    pub fn konst(&mut self, e: NodeId) -> super::ConstId {
        let ast = self.ast();
        let c = match ast.kind(e) {
            Kind::Literal => {
                let t = ast.token(e).0;
                let digits: String = t
                    .chars()
                    .take_while(|c| c.is_ascii_hexdigit() || *c == '_' || *c == 'x')
                    .filter(|&c| c != '_')
                    .collect();
                let v = if let Some(h) = digits.strip_prefix("0x") {
                    u128::from_str_radix(h, 16).ok()
                } else {
                    digits.parse().ok()
                };
                v.map_or(Const::Expr(self.k, self.file, e), Const::Value)
            }
            Kind::PathExpr => match ast.children(e).first().and_then(|&p| self.res(p)) {
                Some(Res::Generic(n)) => match self.tcx.param_at.get(&(self.k, self.file, *n)) {
                    Some(&p) => Const::Param(p),
                    None => Const::Expr(self.k, self.file, e),
                },
                _ => Const::Expr(self.k, self.file, e),
            },
            Kind::Block if ast.children(e).len() == 1 => return self.konst(ast.children(e)[0]),
            _ => Const::Expr(self.k, self.file, e),
        };
        self.tcx.konst(c)
    }

    fn fn_ptr(&mut self, n: NodeId) -> Ty {
        let ast = self.ast();
        let mut inputs = Vec::new();
        let mut output = None;
        let mut variadic = false;
        let mut unsafety = false;
        let mut abi = Abi::Rust;
        for &c in ast.children(n) {
            match ast.kind(c) {
                Kind::Keyword if ast.token(c).0 == "unsafe" => unsafety = true,
                Kind::Keyword if ast.token(c).0 == "extern" => abi = Abi::C,
                Kind::Abi => abi = if crate::cfg::string_value(ast.token(c).0) == "C" { Abi::C } else { Abi::Other },
                Kind::Params => {
                    for &p in ast.children(c) {
                        match ast.kind(p) {
                            Kind::Variadic => variadic = true,
                            Kind::Param => {
                                let t = *ast.children(p).last().unwrap();
                                inputs.push(self.ty(t));
                            }
                            _ => {}
                        }
                    }
                }
                Kind::RetType => output = Some(self.ty(ast.children(c)[0])),
                _ => {}
            }
        }
        let output = output.unwrap_or_else(|| self.tcx.unit());
        let inputs = self.tcx.list(inputs);
        let s = self.tcx.sig(FnSig { inputs, output, unsafety, variadic, abi });
        self.tcx.mk(TyKind::FnPtr(s))
    }

    /// The type a path in a type stands for.
    pub fn path_ty(&mut self, path: NodeId) -> Ty {
        let ast = self.ast();
        if let Some(q) = ast.child(path, Kind::QSelf) {
            return self.qualified(path, q);
        }
        let Some(res) = self.res(path) else {
            let first =
                ast.children(path).iter().find(|&&s| ast.kind(s) == Kind::Segment).map_or("", |&s| ast.token(s).0);
            return self.err(&format!("unresolved path in a type: {first}"));
        };
        let last = ast.children(path).iter().rev().copied().find(|&s| ast.kind(s) == Kind::Segment);
        match res {
            Res::Def(d) => {
                let d = *d;
                match self.env.r.defs[d.0 as usize].kind {
                    DefKind::Struct | DefKind::Enum | DefKind::Union => {
                        let args = self.args(d, last);
                        self.tcx.mk(TyKind::Adt(d, args))
                    }
                    DefKind::TypeAlias => {
                        let target = super::collect::alias_ty(self.tcx, self.env, &mut self.expanding, d);
                        let args = self.args(d, last);
                        let args = self.tcx.tys(args);
                        let params = self.tcx.all_params(d);
                        super::collect::subst(self.tcx, target, &params, &args)
                    }
                    DefKind::Trait | DefKind::TraitAlias => {
                        // A bare trait as a type, from before `dyn`.
                        match self.trait_ref(path) {
                            Some(t) => {
                                let d = self.tcx.dyn_(DynData { traits: vec![t] });
                                self.tcx.mk(TyKind::Dyn(d))
                            }
                            None => self.err("bad trait object"),
                        }
                    }
                    DefKind::ForeignType => self.tcx.mk(TyKind::Foreign(d)),
                    DefKind::Const | DefKind::AssocConst => {
                        let c = self.tcx.konst(Const::Expr(self.k, self.file, path));
                        self.tcx.mk(TyKind::ConstArg(c))
                    }
                    DefKind::AssocType => self.err("associated type named without its self type"),
                    other => self.err(&format!("a path to a {other:?} in a type")),
                }
            }
            Res::PrimTy(p) => prim(self.tcx, p),
            Res::Generic(n) => match self.tcx.param_at.get(&(self.k, self.file, *n)) {
                Some(&p) => {
                    if self.tcx.params[p.0 as usize].is_const {
                        let c = self.tcx.konst(Const::Param(p));
                        self.tcx.mk(TyKind::ConstArg(c))
                    } else {
                        self.tcx.mk(TyKind::Param(p))
                    }
                }
                None => self.err("generic parameter not found"),
            },
            Res::SelfTy(f, n) => match self.self_ty_at(&(self.k, *f, *n)) {
                Some(t) => t,
                None => {
                    let what = self
                        .tcx
                        .def_at
                        .get(&(self.k, *f, *n))
                        .map(|d| format!("{:?}", self.env.r.defs[d.0 as usize].kind));
                    let node_kind = format!("{:?}", self.env.ast(self.k, *f).kind(*n));
                    self.err(&format!("`Self` here has no type ({} {node_kind})", what.unwrap_or_default()))
                }
            },
            Res::Partial(base, rest) if *rest == 1 => {
                let name = last.map_or("", |s| ast.token(s).0);
                self.assoc_of(base, name)
            }
            Res::Partial(..) => self.err("associated item path of more than one segment"),
            Res::Local(_) => self.err("a local variable in a type"),
        }
    }

    /// `<T as Trait>::Assoc` or `<T>::Assoc`.
    fn qualified(&mut self, path: NodeId, q: NodeId) -> Ty {
        let ast = self.ast();
        let qk = ast.children(q).to_vec();
        let self_ty = self.ty(qk[0]);
        let name = ast.children(path).iter().find(|&&s| ast.kind(s) == Kind::Segment).map_or("", |&s| ast.token(s).0);
        if ast.children(path).iter().filter(|&&s| ast.kind(s) == Kind::Segment).count() != 1 {
            return self.err("qualified path of more than one segment");
        }
        match qk.get(1) {
            Some(&tr) => {
                let Some(&tp) = ast.children(tr).first() else { return self.err("bad trait in qualified path") };
                let Some(t) = self.trait_ref(tp) else { return self.err("trait in qualified path not found") };
                self.projection(self_ty, &t, name)
            }
            None => {
                // `<T>::Assoc`: as `T::Assoc`.
                match self.tcx.kind(self_ty) {
                    TyKind::Param(p) => self.assoc_of_param(p, self_ty, name),
                    _ => self.err("associated type of a concrete type"),
                }
            }
        }
    }

    /// `base::name` for an associated type.
    fn assoc_of(&mut self, base: &Res, name: &str) -> Ty {
        match base {
            Res::Generic(n) => match self.tcx.param_at.get(&(self.k, self.file, *n)) {
                Some(&p) => {
                    let t = self.tcx.mk(TyKind::Param(p));
                    self.assoc_of_param(p, t, name)
                }
                None => self.err("generic parameter not found"),
            },
            Res::SelfTy(f, n) => {
                let at = (self.k, *f, *n);
                let Some(self_ty) = self.self_ty_at(&at) else { return self.err("`Self` here has no type") };
                let Some(&d) = self.tcx.def_at.get(&at) else { return self.err("`Self` of nothing") };
                match self.env.r.defs[d.0 as usize].kind {
                    DefKind::Trait => {
                        let t = TraitRef { def: d, args: self.tcx.list(Vec::new()), bindings: Vec::new(), assoc_bounds: Vec::new() };
                        let t = TraitRef { args: self.trait_identity(d), ..t };
                        self.projection(self_ty, &t, name)
                    }
                    DefKind::Impl => {
                        // The impl's own definition, if it has one.
                        if let Some(m) = self.env.r.defs[d.0 as usize].module.or_else(|| self.impl_module(d))
                            && let Some(Res::Def(a)) = self.env.r.lookup(m, name, Ns::Type, m)
                        {
                            return super::collect::alias_ty(self.tcx, self.env, &mut self.expanding, a);
                        }
                        match self.tcx.impl_index.get(&d).map(|&i| self.tcx.impls[i].trait_ref.clone()) {
                            Some(Some(t)) => self.projection(self_ty, &t, name),
                            _ => self.err("associated type of an inherent impl's `Self`"),
                        }
                    }
                    _ => self.err("associated type of a concrete type"),
                }
            }
            _ => self.err("associated type of a concrete type"),
        }
    }

    fn self_ty_at(&self, at: &At) -> Option<Ty> {
        match self.impl_self {
            Some((a, t)) if a == *at => Some(t),
            _ => self.tcx.self_ty.get(at).copied(),
        }
    }

    fn impl_module(&self, d: DefId) -> Option<crate::resolve::ModId> {
        self.env.r.module_of.get(&self.env.at_of(d)).copied()
    }

    /// `P::name` for a generic parameter: a trait that bounds `P` and has
    /// (or whose supertraits have) an associated type `name`.
    fn assoc_of_param(&mut self, p: ParamId, self_ty: Ty, name: &str) -> Ty {
        let owner = self.tcx.params[p.0 as usize].owner;
        // `Self` of a trait: that trait.
        if self.tcx.params[p.0 as usize].name == "Self" && self.env.r.defs[owner.0 as usize].kind == DefKind::Trait {
            let t = TraitRef { def: owner, args: self.trait_identity(owner), bindings: Vec::new(), assoc_bounds: Vec::new() };
            return self.projection(self_ty, &t, name);
        }
        for t in self.param_bound_traits(p) {
            if super::collect::find_assoc(self.env, t.def, name).is_some() {
                return self.projection(self_ty, &t, name);
            }
        }
        self.err("associated type not found in the parameter's bounds")
    }

    /// The trait refs written as bounds on parameter `p` (on it and in
    /// `where` clauses of its owner and the owner's parents).
    fn param_bound_traits(&mut self, p: ParamId) -> Vec<TraitRef> {
        let pd = self.tcx.params[p.0 as usize].clone();
        let mut out = Vec::new();
        // The item being lowered and its parents (which include the
        // parameter's owner): a method's `where T: Tr` bounds its impl's
        // `T`.
        let mut owner = Some(self.owner);
        while let Some(o) = owner {
            let (k, f, node) = self.env.at_of(o);
            let ast = self.env.ast(k, f);
            let saved = (self.k, self.file);
            (self.k, self.file) = (k, f);
            // `T: Bound` in the parameter list.
            if pd.at.1 == f
                && let Some(b) = ast.child(pd.at.2, Kind::Bounds)
                && pd.at.0 == k
            {
                out.extend(self.bounds(b));
            }
            // `where T: Bound`.
            if let Some(w) = ast.child(node, Kind::WhereClause) {
                for &pred in ast.children(w) {
                    let kids = ast.children(pred);
                    let (Some(&t), Some(&b)) = (
                        kids.iter().find(|&&c| ast.kind(c) == Kind::PathType),
                        kids.iter().find(|&&c| ast.kind(c) == Kind::Bounds),
                    ) else {
                        continue;
                    };
                    let is_p = ast.children(t).first().and_then(|&path| self.res(path)).is_some_and(
                        |r| matches!(r, Res::Generic(n) if self.tcx.param_at.get(&(k, f, *n)) == Some(&p)),
                    );
                    if is_p {
                        out.extend(self.bounds(b));
                    }
                }
            }
            (self.k, self.file) = saved;
            owner = self.tcx.generics[o.0 as usize].parent;
        }
        out
    }

    /// A trait's own parameters as arguments (without `Self`).
    fn trait_identity(&self, t: DefId) -> super::List {
        let tys: Vec<Ty> = self.tcx.all_params(t).into_iter().skip(1).map(|p| self.tcx.mk(TyKind::Param(p))).collect();
        self.tcx.list(tys)
    }

    /// `<self_ty as t>::name`.
    fn projection(&mut self, self_ty: Ty, t: &TraitRef, name: &str) -> Ty {
        match super::collect::find_assoc(self.env, t.def, name) {
            Some((assoc, _)) => {
                let mut args = vec![self_ty];
                args.extend(self.tcx.tys(t.args));
                let l = self.tcx.list(args);
                self.tcx.mk(TyKind::Projection(assoc, l))
            }
            None => self.err("associated type not found in the trait"),
        }
    }

    /// The generic arguments a path segment gives item `d`: written ones,
    /// then defaults for the rest.
    fn args(&mut self, d: DefId, seg: Option<NodeId>) -> super::List {
        let ast = self.ast();
        let mut tys = Vec::new();
        if let Some(seg) = seg
            && let Some(g) = ast.child(seg, Kind::GenericArgs)
        {
            for &a in ast.children(g) {
                match ast.kind(a) {
                    Kind::Lifetime | Kind::AssocConstraint => {}
                    Kind::Literal | Kind::Unary | Kind::Block => {
                        let c = self.konst(a);
                        tys.push(self.tcx.mk(TyKind::ConstArg(c)));
                    }
                    _ => tys.push(self.ty(a)),
                }
            }
        }
        let g = self.tcx.generics[d.0 as usize].clone();
        let parent_args = g.parent_count as usize;
        let want = g.own.len();
        let _ = parent_args;
        while tys.len() < want {
            match g.defaults.get(tys.len()).copied().flatten() {
                Some(t) => {
                    // Defaults may name earlier parameters.
                    let params = g.own[..tys.len()].to_vec();
                    let sofar = tys.clone();
                    tys.push(super::collect::subst(self.tcx, t, &params, &sofar));
                }
                None => {
                    let e = self.err("missing generic argument");
                    tys.push(e);
                }
            }
        }
        tys.truncate(want);
        self.tcx.list(tys)
    }

    /// The trait a path in a bound names, its arguments and bindings.
    pub fn trait_ref(&mut self, path: NodeId) -> Option<TraitRef> {
        let ast = self.ast();
        let def = match self.res(path)? {
            Res::Def(d) if matches!(self.env.r.defs[d.0 as usize].kind, DefKind::Trait | DefKind::TraitAlias) => *d,
            _ => return None,
        };
        let seg = ast.children(path).iter().rev().copied().find(|&s| ast.kind(s) == Kind::Segment)?;
        let mut tys = Vec::new();
        let mut bindings = Vec::new();
        let mut assoc_bounds = Vec::new();
        for &c in ast.children(seg) {
            match ast.kind(c) {
                Kind::GenericArgs => {
                    for &a in ast.children(c) {
                        match ast.kind(a) {
                            Kind::Lifetime => {}
                            Kind::AssocConstraint => {
                                let kids = ast.children(a);
                                let name = ast.token(kids[0]).0;
                                let value = kids.iter().rev().copied().find(|&v| is_type(ast.kind(v)));
                                let found = super::collect::find_assoc(self.env, def, name);
                                if let (Some(v), Some((assoc, _))) = (value, found) {
                                    let t = self.ty(v);
                                    bindings.push((assoc, t));
                                } else if let (Some(b), Some((assoc, _))) =
                                    (kids.iter().copied().find(|&v| ast.kind(v) == Kind::Bounds), found)
                                {
                                    // Lowering these can need the bounds of
                                    // other parameters, whose bounds can
                                    // need these: only so deep.
                                    thread_local!(static DEPTH: std::cell::Cell<u32> = const { std::cell::Cell::new(0) });
                                    if DEPTH.get() < 2 {
                                        DEPTH.set(DEPTH.get() + 1);
                                        for tr in self.bounds(b) {
                                            assoc_bounds.push((assoc, tr));
                                        }
                                        DEPTH.set(DEPTH.get() - 1);
                                    }
                                }
                            }
                            Kind::Literal | Kind::Unary | Kind::Block => {
                                let c = self.konst(a);
                                tys.push(self.tcx.mk(TyKind::ConstArg(c)));
                            }
                            _ => tys.push(self.ty(a)),
                        }
                    }
                }
                Kind::FnSugar => {
                    // `Fn(A, B) -> C`: `Fn<(A, B)>` with `Output = C`.
                    let mut inputs = Vec::new();
                    let mut output = None;
                    for &x in ast.children(c) {
                        match ast.kind(x) {
                            Kind::RetType => output = Some(self.ty(ast.children(x)[0])),
                            Kind::Op => {}
                            _ => inputs.push(self.ty(x)),
                        }
                    }
                    let l = self.tcx.list(inputs);
                    tys.push(self.tcx.mk(TyKind::Tuple(l)));
                    let out = output.unwrap_or_else(|| self.tcx.unit());
                    if let Some((assoc, _)) = super::collect::find_assoc(self.env, def, "Output") {
                        bindings.push((assoc, out));
                    }
                }
                _ => {}
            }
        }
        Some(TraitRef { def, args: self.tcx.list(tys), bindings, assoc_bounds })
    }

    /// The trait bounds in a `Bounds` node (lifetimes, `?Sized` and
    /// `use<..>` are not kept).
    pub fn bounds(&mut self, b: NodeId) -> Vec<TraitRef> {
        let ast = self.ast();
        let mut out = Vec::new();
        for &tb in ast.children(b) {
            if ast.kind(tb) != Kind::TraitBound {
                continue;
            }
            if ast.children(tb).iter().any(|&c| ast.kind(c) == Kind::Op && ast.token(c).0 == "?") {
                continue;
            }
            let Some(path) = ast.child(tb, Kind::Path) else { continue };
            match self.trait_ref(path) {
                Some(t) => out.push(t),
                None => {
                    self.err("bound is not a trait");
                }
            }
        }
        out
    }
}

pub(crate) fn is_type(k: Kind) -> bool {
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
            | NeverType
            | InferType
            | FnPtrType
            | ImplTraitType
            | DynTraitType
            | TypeMacro
    )
}

pub(crate) fn prim(tcx: &TyCtxt, p: &str) -> Ty {
    let k = match p {
        "bool" => TyKind::Bool,
        "char" => TyKind::Char,
        "str" => TyKind::Str,
        "i8" => TyKind::Int(IntTy::I8),
        "i16" => TyKind::Int(IntTy::I16),
        "i32" => TyKind::Int(IntTy::I32),
        "i64" => TyKind::Int(IntTy::I64),
        "i128" => TyKind::Int(IntTy::I128),
        "isize" => TyKind::Int(IntTy::Isize),
        "u8" => TyKind::Uint(UintTy::U8),
        "u16" => TyKind::Uint(UintTy::U16),
        "u32" => TyKind::Uint(UintTy::U32),
        "u64" => TyKind::Uint(UintTy::U64),
        "u128" => TyKind::Uint(UintTy::U128),
        "usize" => TyKind::Uint(UintTy::Usize),
        "f16" => TyKind::Float(FloatTy::F16),
        "f32" => TyKind::Float(FloatTy::F32),
        "f64" => TyKind::Float(FloatTy::F64),
        "f128" => TyKind::Float(FloatTy::F128),
        _ => TyKind::Error,
    };
    tcx.mk(k)
}
