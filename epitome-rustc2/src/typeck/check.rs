//! Checking one body.

use super::BodyResult;
use super::infer::{Infer, VarKind};
use crate::ast::{Ast, Kind, NodeId};
use crate::hash::{FastMap, FastSet};
use crate::resolve::{DefId, DefKind, Ns, Res, Resolver};
use crate::session::ParsedCrate;
use crate::ty::collect::subst;
use crate::ty::lower::{Env, Lower, is_type};
use crate::ty::{Const, CtorKind, FloatTy, IntTy, Mutbl, Ty, TyCtxt, TyKind, UintTy};

/// The first type constructor of a type, to find impls by.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Head {
    Adt(DefId),
    Prim(TyKind),
    Str,
    Slice,
    Array,
    Ref(Mutbl),
    Ptr(Mutbl),
    Dyn(DefId),
    Foreign(DefId),
    Tuple,
    FnPtr,
    Never,
}

pub fn head(tcx: &TyCtxt, t: Ty) -> Option<Head> {
    Some(match tcx.kind(t) {
        TyKind::Adt(d, _) => Head::Adt(d),
        k @ (TyKind::Bool | TyKind::Char | TyKind::Int(_) | TyKind::Uint(_) | TyKind::Float(_)) => Head::Prim(k),
        TyKind::Str => Head::Str,
        TyKind::Slice(_) => Head::Slice,
        TyKind::Array(..) => Head::Array,
        TyKind::Ref(m, _) => Head::Ref(m),
        TyKind::Ptr(m, _) => Head::Ptr(m),
        TyKind::Dyn(d) => Head::Dyn(tcx.dyn_of(d).traits.first()?.def),
        TyKind::Foreign(d) => Head::Foreign(d),
        TyKind::Tuple(_) => Head::Tuple,
        TyKind::FnPtr(_) => Head::FnPtr,
        TyKind::Never => Head::Never,
        _ => return None,
    })
}

/// Impls indexed for lookup: inherent impls by their self type's head.
pub struct Index {
    pub inherent: FastMap<Head, Vec<usize>>,
    /// Trait impls by trait (for trait selection, next).
    #[allow(dead_code)]
    pub by_trait: FastMap<DefId, Vec<usize>>,
}

impl Index {
    pub fn new(tcx: &TyCtxt, _r: &Resolver) -> Self {
        let mut inherent: FastMap<Head, Vec<usize>> = FastMap::default();
        let mut by_trait: FastMap<DefId, Vec<usize>> = FastMap::default();
        for (i, imp) in tcx.impls.iter().enumerate() {
            match &imp.trait_ref {
                None => {
                    if let Some(h) = head(tcx, imp.self_ty) {
                        inherent.entry(h).or_default().push(i);
                    }
                }
                Some(t) => by_trait.entry(t.def).or_default().push(i),
            }
        }
        Index { inherent, by_trait }
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn check_body<'s>(
    tcx: &TyCtxt<'s>,
    r: &Resolver<'s>,
    crates: &[ParsedCrate<'s>],
    index: &Index,
    owner: DefId,
    k: u32,
    file: u32,
    item: NodeId,
) -> Option<BodyResult> {
    let ast = &crates[k as usize].files[file as usize].ast;
    let kind = r.defs[owner.0 as usize].kind;
    let mut cx = FnCtxt {
        tcx,
        r,
        env: Env { r, crates },
        infer: Infer::new(tcx),
        index,
        k,
        file,
        ast,
        owner,
        types: FastMap::default(),
        methods: Vec::new(),
        ret: tcx.unit(),
        loops: Vec::new(),
        errors: FastMap::default(),
        reported: FastSet::default(),
    };
    match kind {
        DefKind::Fn | DefKind::AssocFn => {
            let body = ast.children(item).iter().copied().find(|&c| ast.kind(c) == Kind::Block)?;
            let sig = tcx.sig_of(tcx.fn_sig[owner.0 as usize]?);
            let inputs = tcx.tys(sig.inputs);
            let inputs: Vec<Ty> = inputs.into_iter().map(|t| cx.fill_holes(t)).collect();
            let out = cx.fill_holes(sig.output);
            cx.ret = cx.opaque_to_var(out);
            // Parameters: their patterns, in order.
            let params = ast.child(item, Kind::Params)?;
            let mut i = 0;
            for &p in ast.children(params) {
                let t = inputs.get(i).copied().unwrap_or_else(|| tcx.error());
                match ast.kind(p) {
                    Kind::SelfParam => {
                        cx.types.insert(p, t);
                        i += 1;
                    }
                    Kind::Param => {
                        let kids: Vec<NodeId> =
                            ast.children(p).iter().copied().filter(|&c| !matches!(ast.kind(c), Kind::Attribute | Kind::DocComment)).collect();
                        if kids.len() == 2 {
                            cx.pat(kids[0], t, Mutbl::Not, false);
                        }
                        i += 1;
                    }
                    _ => {}
                }
            }
            let ret = cx.ret;
            let got = cx.expr(body, Some(ret));
            cx.coerce(body, got, ret);
        }
        _ => {
            // A constant or static: its value against its type.
            let value = ast.children(item).iter().rev().copied().find(|&c| is_expr(ast.kind(c)))?;
            let t = tcx.type_of(owner).unwrap_or_else(|| tcx.error());
            let t = cx.fill_holes(t);
            let got = cx.expr(value, Some(t));
            cx.coerce(value, got, t);
        }
    }
    Some(cx.finish())
}

/// Whether a node kind is an expression.
fn is_expr(k: Kind) -> bool {
    use Kind::*;
    matches!(
        k,
        Literal
            | PathExpr
            | Binary
            | Unary
            | Borrow
            | Cast
            | Range
            | Call
            | MethodCall
            | Field
            | Index
            | Try
            | Await
            | Tuple
            | Paren
            | Array
            | ArrayRepeat
            | StructExpr
            | Block
            | If
            | Let
            | While
            | Loop
            | For
            | Match
            | Closure
            | Break
            | Continue
            | Return
            | Yield
            | Become
            | Underscore
            | ConstBlock
            | MacroCall
            | AttrExpr
            | Builtin
            | UseExpr
            | Yeet
    )
}

struct Loop<'s> {
    label: Option<&'s str>,
    /// What `break value` gives; `None` for a `while` or `for`.
    ty: Option<Ty>,
    broke: bool,
}

struct FnCtxt<'a, 's> {
    tcx: &'a TyCtxt<'s>,
    r: &'a Resolver<'s>,
    env: Env<'a, 's>,
    infer: Infer<'a, 's>,
    index: &'a Index,
    k: u32,
    file: u32,
    ast: &'a Ast<'s>,
    owner: DefId,
    /// Types of expressions and patterns, unresolved until the end.
    types: FastMap<NodeId, Ty>,
    methods: Vec<(NodeId, DefId)>,
    ret: Ty,
    loops: Vec<Loop<'s>>,
    errors: FastMap<String, usize>,
    /// Nodes already counted as errors.
    reported: FastSet<NodeId>,
}

impl<'a, 's> FnCtxt<'a, 's> {
    fn finish(mut self) -> BodyResult {
        self.infer.fallback();
        let mut types = Vec::with_capacity(self.types.len());
        let (mut error_nodes, mut unknown_nodes) = (0, 0);
        let mut nodes: Vec<(NodeId, Ty)> = self.types.iter().map(|(&n, &t)| (n, t)).collect();
        nodes.sort_by_key(|x| x.0);
        for (n, t) in nodes {
            let t = self.infer.resolve(t);
            if self.tcx.has_error(t) {
                error_nodes += 1;
            } else if self.tcx.has_infer(t) {
                unknown_nodes += 1;
            }
            types.push((n, t));
        }
        BodyResult {
            k: self.k,
            file: self.file,
            types,
            methods: self.methods,
            errors: self.errors,
            error_nodes,
            unknown_nodes,
        }
    }

    /// Counts why a type is an error (once per node), and gives `Error`.
    fn err(&mut self, n: NodeId, why: &str) -> Ty {
        if self.reported.insert(n) {
            *self.errors.entry(why.to_string()).or_default() += 1;
        }
        self.tcx.error()
    }

    fn mk(&self, k: TyKind) -> Ty {
        self.tcx.mk(k)
    }

    fn unit(&self) -> Ty {
        self.tcx.unit()
    }

    fn kids(&self, n: NodeId) -> Vec<NodeId> {
        self.ast.children(n).iter().copied().filter(|&c| !matches!(self.ast.kind(c), Kind::Attribute | Kind::DocComment)).collect()
    }

    fn res(&self, path: NodeId) -> Option<&'a Res> {
        let r: &'a Resolver<'s> = self.r;
        r.res_of(self.k as usize, self.file as usize, path)
    }

    /// A type written in the body (`let x: T`, `as T`, `::<T>`), with `_`
    /// as a fresh variable.
    fn written_ty(&mut self, n: NodeId) -> Ty {
        let mut l = Lower {
            tcx: self.tcx,
            env: &self.env,
            expanding: FastSet::default(),
            impl_self: None,
            holes: true,
            k: self.k,
            file: self.file,
            owner: self.owner,
            failed: false,
        };
        let t = l.ty(n);
        self.fill_holes(t)
    }

    /// `t` with each hole (`Infer(u32::MAX)`) replaced by a fresh variable.
    fn fill_holes(&mut self, t: Ty) -> Ty {
        let tcx = self.tcx;
        match tcx.kind(t) {
            TyKind::Infer(u32::MAX) => self.infer.fresh(),
            TyKind::Tuple(l) => {
                let v = tcx.tys(l).into_iter().map(|x| self.fill_holes(x)).collect();
                self.mk(TyKind::Tuple(tcx.list(v)))
            }
            TyKind::Adt(d, l) => {
                let v = tcx.tys(l).into_iter().map(|x| self.fill_holes(x)).collect();
                self.mk(TyKind::Adt(d, tcx.list(v)))
            }
            TyKind::Ref(m, x) => {
                let x = self.fill_holes(x);
                self.mk(TyKind::Ref(m, x))
            }
            TyKind::Ptr(m, x) => {
                let x = self.fill_holes(x);
                self.mk(TyKind::Ptr(m, x))
            }
            TyKind::Slice(x) => {
                let x = self.fill_holes(x);
                self.mk(TyKind::Slice(x))
            }
            TyKind::Array(x, c) => {
                let x = self.fill_holes(x);
                self.mk(TyKind::Array(x, c))
            }
            _ => t,
        }
    }

    /// `impl Trait` in a return type: checked as a variable for now.
    fn opaque_to_var(&mut self, t: Ty) -> Ty {
        match self.tcx.kind(t) {
            TyKind::Opaque(_) => self.infer.fresh(),
            _ => t,
        }
    }

    fn lang_adt(&mut self, name: &str, args: Vec<Ty>) -> Option<Ty> {
        let d = *self.tcx.lang.get(name)?;
        let l = self.tcx.list(args);
        Some(self.mk(TyKind::Adt(d, l)))
    }

    // ---- coercion ----

    /// Makes `got` (the type of `n`) fit `want`: equal, or coerced (`!` to
    /// anything, `&mut T` to `&T`, `&[T; N]` to `&[T]`, a function item to a
    /// function pointer).
    fn coerce(&mut self, n: NodeId, got: Ty, want: Ty) -> bool {
        let g = self.infer.shallow(got);
        let w = self.infer.shallow(want);
        let tcx = self.tcx;
        match (tcx.kind(g), tcx.kind(w)) {
            (TyKind::Never, _) => return true,
            (TyKind::Ref(Mutbl::Mut, x), TyKind::Ref(Mutbl::Not, y)) => {
                let s = self.infer.snapshot();
                if self.infer.unify(x, y) {
                    return true;
                }
                self.infer.rollback(s);
                if let (TyKind::Array(e, _), TyKind::Slice(f)) = (tcx.kind(self.infer.shallow(x)), tcx.kind(self.infer.shallow(y))) {
                    return self.infer.unify(e, f);
                }
            }
            (TyKind::Ref(m, x), TyKind::Ref(n2, y)) if m == n2 || m == Mutbl::Mut => {
                if let (TyKind::Array(e, _), TyKind::Slice(f)) = (tcx.kind(self.infer.shallow(x)), tcx.kind(self.infer.shallow(y))) {
                    return self.infer.unify(e, f);
                }
            }
            (TyKind::FnDef(d, l), TyKind::FnPtr(s)) => {
                if let Some((inputs, output)) = self.callable_sig(d, l) {
                    let s = tcx.sig_of(s);
                    let want_in = tcx.tys(s.inputs);
                    return inputs.len() == want_in.len()
                        && inputs.iter().zip(&want_in).all(|(&a, &b)| self.infer.unify(a, b))
                        && self.infer.unify(output, s.output);
                }
            }
            (TyKind::Closure(_, cs), TyKind::FnPtr(s)) => {
                let (c, s) = (tcx.sig_of(cs), tcx.sig_of(s));
                return self.infer.unify(self.mk(TyKind::FnPtr(tcx.sig(crate::ty::FnSig { unsafety: s.unsafety, abi: s.abi, ..c }))), w);
            }
            _ => {}
        }
        if self.infer.unify(g, w) {
            return true;
        }
        self.err(n, "mismatched types");
        false
    }

    /// The common type of two branches (`if`/`else`, match arms).
    fn join(&mut self, n: NodeId, a: Ty, b: Ty) -> Ty {
        let (sa, sb) = (self.infer.shallow(a), self.infer.shallow(b));
        if self.tcx.kind(sa) == TyKind::Never {
            return b;
        }
        if self.tcx.kind(sb) == TyKind::Never {
            return a;
        }
        if !self.coerce(n, b, a) {
            return self.tcx.error();
        }
        a
    }

    // ---- items as values ----

    /// The parameter types and result of calling item `d` (a function, or
    /// a tuple struct's or variant's constructor) with arguments `l`.
    fn callable_sig(&mut self, d: DefId, l: crate::ty::List) -> Option<(Vec<Ty>, Ty)> {
        let tcx = self.tcx;
        let args = tcx.tys(l);
        let params = tcx.all_params(d);
        match self.r.defs[d.0 as usize].kind {
            DefKind::Fn | DefKind::AssocFn | DefKind::ForeignFn => {
                let sig = tcx.sig_of(tcx.fn_sig[d.0 as usize]?);
                let inputs = tcx.tys(sig.inputs).into_iter().map(|t| subst(tcx, t, &params, &args)).collect();
                let out = subst(tcx, sig.output, &params, &args);
                let out = self.opaque_to_var(out);
                Some((inputs, out))
            }
            DefKind::Struct => {
                let v = tcx.adts.get(&d)?.variants.first()?.clone();
                let inputs = v.fields.iter().map(|f| subst(tcx, f.ty, &params, &args)).collect();
                Some((inputs, self.mk(TyKind::Adt(d, l))))
            }
            DefKind::Variant => {
                let e = tcx.generics[d.0 as usize].parent?;
                let v = tcx.adts.get(&e)?.variants.iter().find(|v| v.def == Some(d))?.clone();
                let inputs = v.fields.iter().map(|f| subst(tcx, f.ty, &params, &args)).collect();
                Some((inputs, self.mk(TyKind::Adt(e, l))))
            }
            _ => None,
        }
    }

    /// Arguments for item `d`'s parameters: fresh variables, with those
    /// written in the path's last segment (`f::<u8>`) for its own.
    fn item_args(&mut self, d: DefId, path: NodeId) -> Vec<Ty> {
        let tcx = self.tcx;
        let all = tcx.all_params(d);
        let mut args = self.infer.instantiate(&all);
        let own = tcx.generics[d.0 as usize].own.len();
        let written: Vec<NodeId> = self
            .ast
            .children(path)
            .iter()
            .rev()
            .filter(|&&s| self.ast.kind(s) == Kind::Segment)
            .find_map(|&s| self.ast.child(s, Kind::GenericArgs))
            .map(|g| self.ast.children(g).iter().copied().filter(|&a| is_type(self.ast.kind(a))).collect())
            .unwrap_or_default();
        let own_start = all.len() - own;
        // A variant's (or an assoc item's) written arguments may be its
        // parent's: `Option::<u8>::None`, `Vec::<u8>::new`.
        let start = if own == 0 || self.r.defs[d.0 as usize].kind == DefKind::Variant { 0 } else { own_start };
        for (i, a) in written.into_iter().enumerate() {
            if start + i < args.len() {
                let t = self.written_ty(a);
                args[start + i] = t;
            }
        }
        args
    }

    /// The type of a path used as a value.
    fn path_value(&mut self, n: NodeId, path: NodeId) -> Ty {
        let tcx = self.tcx;
        let Some(res) = self.res(path) else { return self.err(n, "unresolved path") };
        match res {
            Res::Local(p) => match self.types.get(p) {
                Some(&t) => t,
                None => self.err(n, "local without a type"),
            },
            Res::Def(d) => {
                let d = *d;
                match self.r.defs[d.0 as usize].kind {
                    DefKind::Fn | DefKind::AssocFn | DefKind::ForeignFn => {
                        let args = self.item_args(d, path);
                        self.mk(TyKind::FnDef(d, tcx.list(args)))
                    }
                    DefKind::Struct => {
                        let args = self.item_args(d, path);
                        let kind = tcx.adts.get(&d).and_then(|a| a.variants.first()).map(|v| v.kind);
                        match kind {
                            Some(CtorKind::Tuple) => self.mk(TyKind::FnDef(d, tcx.list(args))),
                            Some(CtorKind::Unit) => self.mk(TyKind::Adt(d, tcx.list(args))),
                            _ => self.err(n, "struct used as a value"),
                        }
                    }
                    DefKind::Variant => {
                        let args = self.item_args(d, path);
                        let e = tcx.generics[d.0 as usize].parent;
                        let kind = e.and_then(|e| tcx.adts.get(&e)).and_then(|a| a.variants.iter().find(|v| v.def == Some(d))).map(|v| v.kind);
                        match (kind, e) {
                            (Some(CtorKind::Tuple), _) => self.mk(TyKind::FnDef(d, tcx.list(args))),
                            (Some(CtorKind::Unit), Some(e)) => self.mk(TyKind::Adt(e, tcx.list(args))),
                            _ => self.err(n, "variant used as a value"),
                        }
                    }
                    DefKind::Const | DefKind::Static | DefKind::AssocConst | DefKind::ForeignStatic => {
                        let args = self.item_args(d, path);
                        match tcx.type_of(d) {
                            Some(t) => subst(tcx, t, &tcx.all_params(d), &args),
                            None => self.err(n, "constant without a type"),
                        }
                    }
                    _ => self.err(n, "path to something that is not a value"),
                }
            }
            Res::Generic(p) => {
                // A const parameter: its declared type.
                let p = *p;
                match self.ast.children(p).iter().copied().find(|&c| is_type(self.ast.kind(c))) {
                    Some(t) => self.written_ty(t),
                    None => self.err(n, "generic parameter used as a value"),
                }
            }
            Res::SelfTy(f, s) => {
                // `Self` as a unit struct value or tuple constructor.
                let Some(&st) = tcx.self_ty.get(&(self.k, *f, *s)) else { return self.err(n, "`Self` has no type") };
                match tcx.kind(st) {
                    TyKind::Adt(d, l) => match tcx.adts.get(&d).and_then(|a| a.variants.first()).map(|v| v.kind) {
                        Some(CtorKind::Tuple) => self.mk(TyKind::FnDef(d, l)),
                        Some(CtorKind::Unit) => st,
                        _ => self.err(n, "`Self` used as a value"),
                    },
                    _ => self.err(n, "`Self` used as a value"),
                }
            }
            Res::Partial(base, 1) => {
                let name = self.last_segment_name(path);
                let base = (**base).clone();
                self.assoc_value(n, path, &base, name)
            }
            Res::Partial(..) => self.err(n, "path through an associated item"),
            Res::PrimTy(_) => self.err(n, "a type used as a value"),
        }
    }

    fn last_segment_name(&self, path: NodeId) -> &'s str {
        let ast = self.ast;
        ast.children(path).iter().rev().find(|&&s| ast.kind(s) == Kind::Segment).map_or("", |&s| ast.token(s).0)
    }

    /// The type a path's base names, for `Type::item`.
    fn base_ty(&mut self, path: NodeId, base: &Res) -> Option<Ty> {
        let tcx = self.tcx;
        Some(match base {
            Res::Def(d) => match self.r.defs[d.0 as usize].kind {
                DefKind::Struct | DefKind::Enum | DefKind::Union => {
                    // Arguments written on the type's segment, if any.
                    let segs: Vec<NodeId> = self.ast.children(path).iter().copied().filter(|&s| self.ast.kind(s) == Kind::Segment).collect();
                    let all = tcx.all_params(*d);
                    let mut args = self.infer.instantiate(&all);
                    if segs.len() >= 2
                        && let Some(g) = self.ast.child(segs[segs.len() - 2], Kind::GenericArgs)
                    {
                        let written: Vec<NodeId> = self.ast.children(g).iter().copied().filter(|&a| is_type(self.ast.kind(a))).collect();
                        for (i, a) in written.into_iter().enumerate() {
                            if i < args.len() {
                                args[i] = self.written_ty(a);
                            }
                        }
                    }
                    self.mk(TyKind::Adt(*d, tcx.list(args)))
                }
                DefKind::TypeAlias => {
                    let t = tcx.type_of(*d)?;
                    let all = tcx.all_params(*d);
                    let args = self.infer.instantiate(&all);
                    subst(tcx, t, &all, &args)
                }
                _ => return None,
            },
            Res::PrimTy(p) => crate::ty::lower::prim(tcx, p),
            Res::SelfTy(f, s) => *tcx.self_ty.get(&(self.k, *f, *s))?,
            Res::Generic(p) => self.mk(TyKind::Param(*tcx.param_at.get(&(self.k, self.file, *p))?)),
            _ => return None,
        })
    }

    /// `Type::name` as a value: an associated function or constant of an
    /// inherent impl of the type.
    fn assoc_value(&mut self, n: NodeId, path: NodeId, base: &Res, name: &str) -> Ty {
        let Some(bt) = self.base_ty(path, base) else { return self.err(n, "associated item of something that is not a type") };
        let bt = self.infer.shallow(bt);
        match self.inherent_item(bt, name, Ns::Value) {
            Some((item, impl_args)) => {
                let tcx = self.tcx;
                let own = tcx.generics[item.0 as usize].own.len();
                let mut args = impl_args;
                let fresh = (0..own).map(|_| self.infer.fresh()).collect::<Vec<_>>();
                args.extend(fresh);
                match self.r.defs[item.0 as usize].kind {
                    DefKind::AssocFn => self.mk(TyKind::FnDef(item, tcx.list(args))),
                    DefKind::AssocConst => match tcx.type_of(item) {
                        Some(t) => subst(tcx, t, &tcx.all_params(item), &args),
                        None => self.err(n, "constant without a type"),
                    },
                    _ => self.err(n, "associated item is not a value"),
                }
            }
            None => self.err(n, "associated item not found in an inherent impl"),
        }
    }

    /// An item `name` of an inherent impl whose self type unifies with
    /// `t`: the item, and the impl's arguments.
    fn inherent_item(&mut self, t: Ty, name: &str, ns: Ns) -> Option<(DefId, Vec<Ty>)> {
        let tcx = self.tcx;
        let h = head(tcx, t)?;
        let impls = self.index.inherent.get(&h)?.clone();
        for i in impls {
            let imp = &tcx.impls[i];
            let Some(m) = self.r.defs[imp.def.0 as usize].module else { continue };
            let Some(Res::Def(item)) = self.r.modules[m.0 as usize].names.get(&(name, ns)).map(|b| b.res.clone()) else { continue };
            let s = self.infer.snapshot();
            let params = tcx.all_params(imp.def);
            let args = self.infer.instantiate(&params);
            let st = subst(tcx, imp.self_ty, &params, &args);
            if self.infer.unify(st, t) {
                return Some((item, args));
            }
            self.infer.rollback(s);
        }
        None
    }

    // ---- method calls ----

    /// `recv.name(args)`: inherent methods, through references and `Box`.
    fn method_call(&mut self, n: NodeId, recv_ty: Ty, name: &str) -> Option<(DefId, Vec<Ty>, Ty)> {
        let tcx = self.tcx;
        let mut t = self.infer.shallow(recv_ty);
        for _ in 0..8 {
            if let Some((item, impl_args)) = self.inherent_item(t, name, Ns::Value)
                && self.r.defs[item.0 as usize].kind == DefKind::AssocFn
            {
                let own = tcx.generics[item.0 as usize].own.len();
                let mut args = impl_args;
                let fresh = (0..own).map(|_| self.infer.fresh()).collect::<Vec<_>>();
                args.extend(fresh);
                let l = tcx.list(args);
                let (inputs, output) = self.callable_sig(item, l)?;
                let Some(&self_param) = inputs.first() else { continue };
                // The receiver by value, or borrowed to fit `&self`.
                let s = self.infer.snapshot();
                if self.infer.unify(self_param, t) {
                    self.methods.push((n, item));
                    return Some((item, inputs[1..].to_vec(), output));
                }
                self.infer.rollback(s);
                if let TyKind::Ref(_, inner) = tcx.kind(self.infer.shallow(self_param))
                    && self.infer.unify(inner, t)
                {
                    self.methods.push((n, item));
                    return Some((item, inputs[1..].to_vec(), output));
                }
            }
            // Through a reference, a raw pointer's not, or a `Box`.
            t = match tcx.kind(t) {
                TyKind::Ref(_, x) => self.infer.shallow(x),
                TyKind::Adt(d, l) if Some(&d) == tcx.lang.get("owned_box") => self.infer.shallow(tcx.tys(l)[0]),
                _ => return None,
            };
        }
        None
    }

    // ---- expressions ----

    /// The type of expression `n`, checked against `expect` where that
    /// helps inference (the caller still coerces).
    fn expr(&mut self, n: NodeId, expect: Option<Ty>) -> Ty {
        let t = self.expr_inner(n, expect);
        self.types.insert(n, t);
        t
    }

    fn expr_inner(&mut self, n: NodeId, expect: Option<Ty>) -> Ty {
        let tcx = self.tcx;
        let ast = self.ast;
        let kids = self.kids(n);
        match ast.kind(n) {
            Kind::Literal => self.literal(n, expect),
            Kind::PathExpr => self.path_value(n, kids[0]),
            Kind::Paren | Kind::AttrExpr => {
                let e = *kids.last().unwrap();
                self.expr(e, expect)
            }
            Kind::Tuple => {
                let want: Vec<Option<Ty>> = match expect.map(|e| tcx.kind(self.infer.shallow(e))) {
                    Some(TyKind::Tuple(l)) if tcx.tys(l).len() == kids.len() => tcx.tys(l).into_iter().map(Some).collect(),
                    _ => vec![None; kids.len()],
                };
                let tys: Vec<Ty> = kids.iter().zip(want).map(|(&e, w)| self.expr(e, w)).collect();
                self.mk(TyKind::Tuple(tcx.list(tys)))
            }
            Kind::Array => {
                let elem = match expect.map(|e| tcx.kind(self.infer.shallow(e))) {
                    Some(TyKind::Array(t, _) | TyKind::Slice(t)) => t,
                    _ => self.infer.fresh(),
                };
                for &e in &kids {
                    let t = self.expr(e, Some(elem));
                    self.coerce(e, t, elem);
                }
                let c = tcx.konst(Const::Value(kids.len() as u128));
                self.mk(TyKind::Array(elem, c))
            }
            Kind::ArrayRepeat => {
                let elem = self.infer.fresh();
                let t = self.expr(kids[0], Some(elem));
                self.coerce(kids[0], t, elem);
                let usize_ = self.mk(TyKind::Uint(UintTy::Usize));
                let lt = self.expr(kids[1], Some(usize_));
                self.coerce(kids[1], lt, usize_);
                let mut l = Lower { tcx, env: &self.env, expanding: FastSet::default(), impl_self: None, holes: true, k: self.k, file: self.file, owner: self.owner, failed: false };
                let c = l.konst(kids[1]);
                self.mk(TyKind::Array(elem, c))
            }
            Kind::Block | Kind::ConstBlock => self.block(n, expect),
            Kind::Unary => self.unary(n, &kids, expect),
            Kind::Borrow => {
                let raw = kids.iter().any(|&c| ast.kind(c) == Kind::Keyword && ast.token(c).0 == "raw");
                let mutbl = if kids.iter().any(|&c| ast.kind(c) == Kind::Keyword && ast.token(c).0 == "mut") { Mutbl::Mut } else { Mutbl::Not };
                let e = *kids.last().unwrap();
                let inner_expect = expect.and_then(|x| match tcx.kind(self.infer.shallow(x)) {
                    TyKind::Ref(_, i) | TyKind::Ptr(_, i) => Some(i),
                    _ => None,
                });
                let t = self.expr(e, inner_expect);
                if raw { self.mk(TyKind::Ptr(mutbl, t)) } else { self.mk(TyKind::Ref(mutbl, t)) }
            }
            Kind::Cast => {
                let target = self.written_ty(kids[1]);
                let _ = self.expr(kids[0], None);
                target
            }
            Kind::Binary => self.binary(n, &kids),
            Kind::Range => self.range(n, &kids),
            Kind::Call => self.call(n, &kids, expect),
            Kind::MethodCall => {
                let recv = self.expr(kids[0], None);
                let name = ast.token(kids[1]).0;
                let args = self.kids(kids[2]);
                match self.method_call(n, recv, name) {
                    Some((_, inputs, output)) => {
                        self.check_args(n, &args, &inputs);
                        output
                    }
                    None => {
                        for &a in &args {
                            self.expr(a, None);
                        }
                        self.err(n, "method not found (inherent methods only)")
                    }
                }
            }
            Kind::Field => {
                let base = self.expr(kids[0], None);
                let name = ast.token(kids[1]).0;
                self.field(n, base, name)
            }
            Kind::Index => {
                let base = self.expr(kids[0], None);
                let idx = self.expr(kids[1], None);
                self.index(n, base, idx)
            }
            Kind::StructExpr => self.struct_expr(n, &kids),
            Kind::If => {
                let bool_ = self.mk(TyKind::Bool);
                let ct = self.expr(kids[0], Some(bool_));
                self.coerce(kids[0], ct, bool_);
                let then = self.expr(kids[1], expect);
                match kids.get(2) {
                    Some(&e) => {
                        let et = self.expr(e, expect.or(Some(then)));
                        self.join(n, then, et)
                    }
                    None => {
                        let unit = self.unit();
                        self.coerce(kids[1], then, unit);
                        unit
                    }
                }
            }
            Kind::Let => {
                let t = self.expr(kids[1], None);
                self.pat(kids[0], t, Mutbl::Not, false);
                self.mk(TyKind::Bool)
            }
            Kind::Match => {
                let scrut = self.expr(kids[0], None);
                let mut result: Option<Ty> = None;
                for &arm in &kids[1..] {
                    if ast.kind(arm) != Kind::MatchArm {
                        continue;
                    }
                    let ak = self.kids(arm);
                    self.pat(ak[0], scrut, Mutbl::Not, false);
                    let mut i = 1;
                    if let Some(&g) = ak.get(1)
                        && ast.kind(g) == Kind::Guard
                    {
                        let bool_ = self.mk(TyKind::Bool);
                        let ge = self.ast.children(g)[0];
                        let gt = self.expr(ge, Some(bool_));
                        self.coerce(ge, gt, bool_);
                        i = 2;
                    }
                    if let Some(&body) = ak.get(i) {
                        let t = self.expr(body, expect.or(result));
                        result = Some(match result {
                            None => t,
                            Some(r) => self.join(body, r, t),
                        });
                    }
                }
                result.unwrap_or_else(|| self.mk(TyKind::Never))
            }
            Kind::Loop => {
                let label = kids.first().filter(|&&c| ast.kind(c) == Kind::Lifetime).map(|&c| ast.token(c).0);
                let ty = self.infer.fresh();
                self.loops.push(Loop { label, ty: Some(ty), broke: false });
                let unit = self.unit();
                let body = *kids.last().unwrap();
                let bt = self.expr(body, Some(unit));
                self.coerce(body, bt, unit);
                let l = self.loops.pop().unwrap();
                if l.broke { ty } else { self.mk(TyKind::Never) }
            }
            Kind::While => {
                let rest: Vec<NodeId> = kids.iter().copied().filter(|&c| ast.kind(c) != Kind::Lifetime).collect();
                let label = kids.first().filter(|&&c| ast.kind(c) == Kind::Lifetime).map(|&c| ast.token(c).0);
                let bool_ = self.mk(TyKind::Bool);
                let ct = self.expr(rest[0], Some(bool_));
                self.coerce(rest[0], ct, bool_);
                self.loops.push(Loop { label, ty: None, broke: false });
                let unit = self.unit();
                let bt = self.expr(rest[1], Some(unit));
                self.coerce(rest[1], bt, unit);
                self.loops.pop();
                unit
            }
            Kind::For => {
                let rest: Vec<NodeId> = kids.iter().copied().filter(|&c| !matches!(ast.kind(c), Kind::Lifetime | Kind::Keyword)).collect();
                let label = kids.first().filter(|&&c| ast.kind(c) == Kind::Lifetime).map(|&c| ast.token(c).0);
                let it = self.expr(rest[1], None);
                let item = self.iter_item(rest[1], it);
                self.pat(rest[0], item, Mutbl::Not, false);
                self.loops.push(Loop { label, ty: None, broke: false });
                let unit = self.unit();
                let bt = self.expr(rest[2], Some(unit));
                self.coerce(rest[2], bt, unit);
                self.loops.pop();
                unit
            }
            Kind::Break => {
                let label = kids.first().filter(|&&c| ast.kind(c) == Kind::Lifetime).map(|&c| ast.token(c).0);
                let value = kids.iter().copied().find(|&c| ast.kind(c) != Kind::Lifetime);
                let target = match label {
                    Some(l) => self.loops.iter().rposition(|x| x.label == Some(l)),
                    None => self.loops.len().checked_sub(1),
                };
                let want = target.and_then(|i| {
                    self.loops[i].broke = true;
                    self.loops[i].ty
                });
                match (value, want) {
                    (Some(v), Some(w)) => {
                        let t = self.expr(v, Some(w));
                        self.coerce(v, t, w);
                    }
                    (Some(v), None) => {
                        self.expr(v, None);
                    }
                    (None, Some(w)) => {
                        let unit = self.unit();
                        self.infer.unify(w, unit);
                    }
                    (None, None) => {}
                }
                self.mk(TyKind::Never)
            }
            Kind::Continue => self.mk(TyKind::Never),
            Kind::Return => {
                let ret = self.ret;
                match kids.first() {
                    Some(&e) => {
                        let t = self.expr(e, Some(ret));
                        self.coerce(e, t, ret);
                    }
                    None => {
                        let unit = self.unit();
                        self.coerce(n, unit, ret);
                    }
                }
                self.mk(TyKind::Never)
            }
            Kind::Underscore => self.infer.fresh(),
            Kind::Try => {
                let t = self.expr(kids[0], None);
                self.try_output(n, t)
            }
            Kind::Closure => self.closure(n, &kids, expect),
            Kind::MacroCall => self.builtin_call(n, &kids),
            Kind::Await | Kind::Yield | Kind::Become | Kind::Yeet | Kind::Builtin | Kind::UseExpr => {
                for &c in &kids {
                    if is_expr(ast.kind(c)) {
                        self.expr(c, None);
                    }
                }
                self.err(n, "unsupported expression")
            }
            _ => self.err(n, "not an expression"),
        }
    }

    fn literal(&mut self, n: NodeId, expect: Option<Ty>) -> Ty {
        let tcx = self.tcx;
        let t = self.ast.token(n).0;
        if t == "true" || t == "false" {
            return self.mk(TyKind::Bool);
        }
        let b = t.as_bytes();
        match b[0] {
            b'\'' => return self.mk(TyKind::Char),
            b'"' | b'r' => {
                let s = self.mk(TyKind::Str);
                return self.mk(TyKind::Ref(Mutbl::Not, s));
            }
            b'b' if b.get(1) == Some(&b'\'') => return self.mk(TyKind::Uint(UintTy::U8)),
            b'b' => {
                let u8_ = self.mk(TyKind::Uint(UintTy::U8));
                let c = tcx.konst(Const::Expr(self.k, self.file, n));
                let a = self.mk(TyKind::Array(u8_, c));
                return self.mk(TyKind::Ref(Mutbl::Not, a));
            }
            b'c' => return self.infer.fresh(),
            _ => {}
        }
        // A number: its suffix, or a variable of the right kind.
        let hex = t.starts_with("0x") || t.starts_with("0b") || t.starts_with("0o");
        let body_end = if hex {
            ["i8", "i16", "i32", "i64", "i128", "isize", "u8", "u16", "u32", "u64", "u128", "usize"]
                .iter()
                .find(|s| t.ends_with(*s))
                .map_or(t.len(), |s| t.len() - s.len())
        } else {
            t.find(|c: char| c.is_ascii_alphabetic() && c != 'e' && c != 'E').unwrap_or(t.len())
        };
        let suffix = &t[body_end..];
        let digits = &t[..body_end];
        let float = !hex && (digits.contains('.') || digits.contains(['e', 'E']));
        let k = match suffix {
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
            _ => {
                let v = self.infer.var(if float { VarKind::Float } else { VarKind::Int });
                // The expected type settles it, if it is a number.
                if let Some(e) = expect
                    && self.infer.is_numeric(e)
                {
                    let s = self.infer.snapshot();
                    if !self.infer.unify(v, e) {
                        self.infer.rollback(s);
                    }
                }
                return v;
            }
        };
        self.mk(k)
    }

    fn unary(&mut self, n: NodeId, kids: &[NodeId], expect: Option<Ty>) -> Ty {
        let tcx = self.tcx;
        let op = self.ast.token(kids[0]).0;
        if self.ast.kind(kids[0]) == Kind::Keyword {
            // `box e`.
            let t = self.expr(kids[1], None);
            return self.lang_adt("owned_box", vec![t]).unwrap_or_else(|| self.err(n, "`box` without `Box`"));
        }
        let t = self.expr(kids[1], if op == "*" { None } else { expect });
        let s = self.infer.shallow(t);
        match op {
            "*" => match tcx.kind(s) {
                TyKind::Ref(_, x) | TyKind::Ptr(_, x) => x,
                TyKind::Adt(d, l) if Some(&d) == tcx.lang.get("owned_box") => tcx.tys(l)[0],
                _ => self.err(n, "dereference of something that is not a reference (needs `Deref`)"),
            },
            "-" if self.infer.is_numeric(s) => t,
            "!" if self.infer.is_integral(s) || tcx.kind(s) == TyKind::Bool => t,
            _ => self.err(n, "operator on a type that is not a primitive (needs operator traits)"),
        }
    }

    fn binary(&mut self, n: NodeId, kids: &[NodeId]) -> Ty {
        let tcx = self.tcx;
        let op = self.ast.text(kids[1]);
        let bool_ = self.mk(TyKind::Bool);
        match op.as_str() {
            "&&" | "||" => {
                let a = self.expr(kids[0], Some(bool_));
                self.coerce(kids[0], a, bool_);
                let b = self.expr(kids[2], Some(bool_));
                self.coerce(kids[2], b, bool_);
                return bool_;
            }
            "=" => {
                let a = self.expr(kids[0], None);
                let b = self.expr(kids[2], Some(a));
                self.coerce(kids[2], b, a);
                return self.unit();
            }
            _ => {}
        }
        let a = self.expr(kids[0], None);
        let shift = matches!(op.as_str(), "<<" | ">>" | "<<=" | ">>=");
        let b = self.expr(kids[2], if shift { None } else { Some(a) });
        let (sa, sb) = (self.infer.shallow(a), self.infer.shallow(b));
        let prim = |s: Ty| self.infer.is_numeric(s) || matches!(tcx.kind(s), TyKind::Bool | TyKind::Char);
        let both_prim = prim(sa) && prim(sb) || prim(sa) && self.infer.var_kind(sb).is_some() || self.infer.var_kind(sa).is_some() && prim(sb);
        let assign = op.ends_with('=') && !matches!(op.as_str(), "==" | "!=" | "<=" | ">=");
        let compare = matches!(op.as_str(), "==" | "!=" | "<" | ">" | "<=" | ">=");
        if compare {
            // Comparing two values of one type (any type, for now).
            self.coerce(kids[2], b, a);
            return bool_;
        }
        if both_prim || self.infer.var_kind(sa).is_some() && self.infer.var_kind(sb).is_some() {
            if !shift {
                self.coerce(kids[2], b, a);
            }
            return if assign { self.unit() } else { a };
        }
        if matches!(tcx.kind(sa), TyKind::Ptr(..)) && !assign {
            return a;
        }
        self.err(n, "operator on a type that is not a primitive (needs operator traits)")
    }

    fn range(&mut self, n: NodeId, kids: &[NodeId]) -> Ty {
        let op_at = kids.iter().position(|&c| self.ast.kind(c) == Kind::Op).unwrap_or(0);
        let inclusive = self.ast.text(kids[op_at]) == "..=";
        let start = kids[..op_at].first().copied();
        let end = kids.get(op_at + 1).copied();
        let elem = self.infer.fresh();
        for e in [start, end].into_iter().flatten() {
            let t = self.expr(e, Some(elem));
            self.coerce(e, t, elem);
        }
        let name = match (start.is_some(), end.is_some(), inclusive) {
            (true, true, false) => "Range",
            (true, false, _) => "RangeFrom",
            (false, true, false) => "RangeTo",
            (false, false, _) => "RangeFull",
            (true, true, true) => "RangeInclusive",
            (false, true, true) => "RangeToInclusive",
        };
        let args = if name == "RangeFull" { vec![] } else { vec![elem] };
        self.lang_adt(name, args).unwrap_or_else(|| self.err(n, "range type not found"))
    }

    fn check_args(&mut self, n: NodeId, args: &[NodeId], inputs: &[Ty]) {
        if args.len() != inputs.len() {
            self.err(n, "wrong number of arguments");
        }
        for (i, &a) in args.iter().enumerate() {
            match inputs.get(i) {
                Some(&want) => {
                    let t = self.expr(a, Some(want));
                    self.coerce(a, t, want);
                }
                None => {
                    self.expr(a, None);
                }
            }
        }
    }

    fn call(&mut self, n: NodeId, kids: &[NodeId], _expect: Option<Ty>) -> Ty {
        let tcx = self.tcx;
        let callee = self.expr(kids[0], None);
        let args = self.kids(kids[1]);
        let s = self.infer.shallow(callee);
        let sig = match tcx.kind(s) {
            TyKind::FnDef(d, l) => self.callable_sig(d, l),
            TyKind::FnPtr(sig) | TyKind::Closure(_, sig) => {
                let sig = tcx.sig_of(sig);
                Some((tcx.tys(sig.inputs), sig.output))
            }
            TyKind::Ref(_, x) => match tcx.kind(self.infer.shallow(x)) {
                TyKind::FnPtr(sig) | TyKind::Closure(_, sig) => {
                    let sig = tcx.sig_of(sig);
                    Some((tcx.tys(sig.inputs), sig.output))
                }
                _ => None,
            },
            _ => None,
        };
        match sig {
            Some((inputs, output)) => {
                let variadic = matches!(tcx.kind(s), TyKind::FnDef(d, _) if tcx.fn_sig[d.0 as usize].is_some_and(|x| tcx.sig_of(x).variadic));
                if variadic && args.len() >= inputs.len() {
                    for (i, &a) in args.iter().enumerate() {
                        let t = self.expr(a, inputs.get(i).copied());
                        if let Some(&w) = inputs.get(i) {
                            self.coerce(a, t, w);
                        }
                    }
                } else {
                    self.check_args(n, &args, &inputs);
                }
                output
            }
            None => {
                for &a in &args {
                    self.expr(a, None);
                }
                self.err(n, "call of something that is not a function (needs `Fn` traits)")
            }
        }
    }

    /// `base.name`, through references and `Box`.
    fn field(&mut self, n: NodeId, base: Ty, name: &str) -> Ty {
        let tcx = self.tcx;
        let mut t = self.infer.shallow(base);
        for _ in 0..8 {
            match tcx.kind(t) {
                TyKind::Adt(d, l) => {
                    if let Some(adt) = tcx.adts.get(&d)
                        && adt.kind != crate::ty::AdtKind::Enum
                        && let Some(f) = adt.variants[0].fields.iter().find(|f| f.name == name)
                    {
                        return subst(tcx, f.ty, &tcx.all_params(d), &tcx.tys(l));
                    }
                    if Some(&d) == tcx.lang.get("owned_box") {
                        t = self.infer.shallow(tcx.tys(l)[0]);
                        continue;
                    }
                    return self.err(n, "field not found (or behind `Deref`)");
                }
                TyKind::Tuple(l) => {
                    return match name.parse::<usize>().ok().and_then(|i| tcx.tys(l).get(i).copied()) {
                        Some(x) => x,
                        None => self.err(n, "tuple field out of range"),
                    };
                }
                TyKind::Ref(_, x) | TyKind::Ptr(_, x) => t = self.infer.shallow(x),
                TyKind::Infer(_) => return self.err(n, "field of a value whose type is not yet known"),
                _ => return self.err(n, "field of something that has none"),
            }
        }
        self.err(n, "field through too many references")
    }

    fn index(&mut self, n: NodeId, base: Ty, idx: Ty) -> Ty {
        let tcx = self.tcx;
        let mut t = self.infer.shallow(base);
        loop {
            match tcx.kind(t) {
                TyKind::Ref(_, x) => t = self.infer.shallow(x),
                TyKind::Array(e, _) | TyKind::Slice(e) => {
                    let i = self.infer.shallow(idx);
                    if self.infer.is_integral(i) {
                        let usize_ = self.mk(TyKind::Uint(UintTy::Usize));
                        self.infer.unify(i, usize_);
                        return e;
                    }
                    // A range gives a slice.
                    if let TyKind::Adt(..) = tcx.kind(i) {
                        return self.mk(TyKind::Slice(e));
                    }
                    return self.err(n, "index that is not an integer or range");
                }
                TyKind::Str => return t,
                _ => return self.err(n, "indexing something that is not an array or slice (needs `Index`)"),
            }
        }
    }

    fn struct_expr(&mut self, n: NodeId, kids: &[NodeId]) -> Ty {
        let tcx = self.tcx;
        let path = kids[0];
        let (adt_ty, variant) = match self.res(path) {
            Some(Res::Def(d)) => {
                let d = *d;
                match self.r.defs[d.0 as usize].kind {
                    DefKind::Struct | DefKind::Union => {
                        let args = self.item_args(d, path);
                        (self.mk(TyKind::Adt(d, tcx.list(args))), tcx.adts.get(&d).and_then(|a| a.variants.first().cloned()))
                    }
                    DefKind::Variant => {
                        let args = self.item_args(d, path);
                        let e = tcx.generics[d.0 as usize].parent;
                        let v = e.and_then(|e| tcx.adts.get(&e)).and_then(|a| a.variants.iter().find(|v| v.def == Some(d)).cloned());
                        match e {
                            Some(e) => (self.mk(TyKind::Adt(e, tcx.list(args))), v),
                            None => (self.tcx.error(), None),
                        }
                    }
                    DefKind::TypeAlias => match tcx.type_of(d) {
                        Some(t) => {
                            let all = tcx.all_params(d);
                            let args = self.infer.instantiate(&all);
                            let t = subst(tcx, t, &all, &args);
                            let v = match tcx.kind(t) {
                                TyKind::Adt(a, _) => tcx.adts.get(&a).and_then(|x| x.variants.first().cloned()),
                                _ => None,
                            };
                            (t, v)
                        }
                        None => (self.tcx.error(), None),
                    },
                    _ => (self.tcx.error(), None),
                }
            }
            Some(Res::SelfTy(f, s)) => match tcx.self_ty.get(&(self.k, *f, *s)) {
                Some(&t) => {
                    let v = match tcx.kind(t) {
                        TyKind::Adt(a, _) => tcx.adts.get(&a).and_then(|x| x.variants.first().cloned()),
                        _ => None,
                    };
                    (t, v)
                }
                None => (self.tcx.error(), None),
            },
            Some(Res::Partial(base, 1)) => {
                // `Self::Variant { .. }` and `Alias::Variant { .. }`.
                let name = self.last_segment_name(path);
                let base = (**base).clone();
                match self.base_ty(path, &base).map(|t| self.infer.shallow(t)).map(|t| (t, tcx.kind(t))) {
                    Some((t, TyKind::Adt(e, _))) => {
                        let v = tcx.adts.get(&e).and_then(|a| a.variants.iter().find(|v| v.name == name).cloned());
                        (t, v)
                    }
                    _ => (self.tcx.error(), None),
                }
            }
            _ => (self.tcx.error(), None),
        };
        let (d, args) = match tcx.kind(adt_ty) {
            TyKind::Adt(d, l) => (Some(d), tcx.tys(l)),
            _ => (None, Vec::new()),
        };
        for &f in &kids[1..] {
            let fk = self.kids(f);
            match self.ast.kind(f) {
                Kind::FieldInit => {
                    let name = self.ast.token(fk[0]).0;
                    let want = variant.as_ref().and_then(|v| v.fields.iter().find(|x| x.name == name)).map(|x| {
                        let params = d.map(|d| tcx.all_params(d)).unwrap_or_default();
                        subst(tcx, x.ty, &params, &args)
                    });
                    let value = fk.get(1).copied();
                    match (value, want) {
                        (Some(v), Some(w)) => {
                            let t = self.expr(v, Some(w));
                            self.coerce(v, t, w);
                        }
                        (Some(v), None) => {
                            self.expr(v, None);
                        }
                        // `S { a }`: the local `a`.
                        (None, Some(w)) => {
                            if let Some(res) = self.r.res_of(self.k as usize, self.file as usize, fk[0])
                                && let Res::Local(p) = res
                                && let Some(&t) = self.types.get(p)
                            {
                                self.coerce(fk[0], t, w);
                            }
                        }
                        (None, None) => {}
                    }
                }
                Kind::StructBase => {
                    if let Some(&b) = fk.first() {
                        let t = self.expr(b, Some(adt_ty));
                        self.coerce(b, t, adt_ty);
                    }
                }
                _ => {}
            }
        }
        if variant.is_none() {
            return self.err(n, "struct literal of something that is not a struct or variant");
        }
        adt_ty
    }

    /// What `for` gets from iterating a value: built in for ranges,
    /// arrays and slices by reference, for now.
    fn iter_item(&mut self, n: NodeId, t: Ty) -> Ty {
        let tcx = self.tcx;
        let s = self.infer.shallow(t);
        match tcx.kind(s) {
            TyKind::Adt(d, l) if ["Range", "RangeFrom", "RangeInclusive"].iter().any(|r| tcx.lang.get(r) == Some(&d)) => tcx.tys(l)[0],
            TyKind::Ref(m, x) => match tcx.kind(self.infer.shallow(x)) {
                TyKind::Array(e, _) | TyKind::Slice(e) => self.mk(TyKind::Ref(m, e)),
                _ => self.err(n, "iterating needs `IntoIterator`"),
            },
            _ => self.err(n, "iterating needs `IntoIterator`"),
        }
    }

    /// `e?` on `Option<T>` or `Result<T, E>`: `T`.
    fn try_output(&mut self, n: NodeId, t: Ty) -> Ty {
        let tcx = self.tcx;
        match tcx.kind(self.infer.shallow(t)) {
            TyKind::Adt(d, l) if tcx.lang.get("Option") == Some(&d) || self.r.defs[d.0 as usize].name == "Result" => {
                tcx.tys(l)[0]
            }
            _ => self.err(n, "`?` on something other than `Option` or `Result` (needs `Try`)"),
        }
    }

    fn closure(&mut self, n: NodeId, kids: &[NodeId], expect: Option<Ty>) -> Ty {
        let tcx = self.tcx;
        let ast = self.ast;
        // Parameter types written, or taken from an expected closure or
        // function pointer type.
        let expected_inputs: Option<Vec<Ty>> = expect.and_then(|e| match tcx.kind(self.infer.shallow(e)) {
            TyKind::FnPtr(s) | TyKind::Closure(_, s) => Some(tcx.tys(tcx.sig_of(s).inputs)),
            _ => None,
        });
        let params = kids.iter().copied().find(|&c| ast.kind(c) == Kind::ClosureParams).map(|p| self.kids(p)).unwrap_or_default();
        let mut inputs = Vec::new();
        for (i, &p) in params.iter().enumerate() {
            let pk = self.kids(p);
            let t = match pk.get(1) {
                Some(&ty) => self.written_ty(ty),
                None => expected_inputs.as_ref().and_then(|v| v.get(i).copied()).unwrap_or_else(|| self.infer.fresh()),
            };
            self.pat(pk[0], t, Mutbl::Not, false);
            inputs.push(t);
        }
        let ret = match kids.iter().copied().find(|&c| ast.kind(c) == Kind::RetType) {
            Some(r) => self.written_ty(self.ast.children(r)[0]),
            None => self.infer.fresh(),
        };
        let body = *kids.last().unwrap();
        let saved_ret = std::mem::replace(&mut self.ret, ret);
        let saved_loops = std::mem::take(&mut self.loops);
        let bt = self.expr(body, Some(ret));
        self.coerce(body, bt, ret);
        self.ret = saved_ret;
        self.loops = saved_loops;
        let id = tcx.closure_id((self.k, self.file, n));
        let sig = tcx.sig(crate::ty::FnSig { inputs: tcx.list(inputs), output: ret, unsafety: false, variadic: false, abi: crate::ty::Abi::Rust });
        self.mk(TyKind::Closure(id, sig))
    }

    /// Built-in macros left as calls.
    fn builtin_call(&mut self, n: NodeId, kids: &[NodeId]) -> Ty {
        let Some(&path) = kids.first() else { return self.err(n, "macro call") };
        let name = self.last_segment_name(path);
        match name {
            "format_args" | "const_format_args" | "format_args_nl" => {
                self.lang_adt("format_arguments", Vec::new()).unwrap_or_else(|| self.err(n, "`Arguments` not found"))
            }
            "include_str" => {
                let s = self.mk(TyKind::Str);
                self.mk(TyKind::Ref(Mutbl::Not, s))
            }
            "offset_of" => self.mk(TyKind::Uint(UintTy::Usize)),
            "asm" | "assert" | "global_asm" | "naked_asm" | "trace_macros" | "log_syntax" => self.unit(),
            "core_panic" | "std_panic" | "unreachable_2015" | "unreachable_2021" | "panic" => self.mk(TyKind::Never),
            _ => self.infer.fresh(),
        }
    }

    // ---- statements ----

    fn block(&mut self, n: NodeId, expect: Option<Ty>) -> Ty {
        let ast = self.ast;
        let kids = self.kids(n);
        let stmts: Vec<NodeId> =
            kids.iter().copied().filter(|&c| !matches!(ast.kind(c), Kind::Lifetime | Kind::Keyword | Kind::InnerAttribute)).collect();
        if ast.kind(n) == Kind::ConstBlock {
            return self.expr(kids[0], expect);
        }
        let mut diverges = false;
        let last = stmts.len().checked_sub(1);
        for (i, &s) in stmts.iter().enumerate() {
            match ast.kind(s) {
                Kind::LetStmt => {
                    let sk: Vec<NodeId> = self.kids(s).into_iter().filter(|&c| ast.kind(c) != Kind::Keyword).collect();
                    let pat = sk[0];
                    let mut rest = &sk[1..];
                    let ty = match rest.first() {
                        Some(&t) if is_type(ast.kind(t)) => {
                            rest = &rest[1..];
                            Some(self.written_ty(t))
                        }
                        _ => None,
                    };
                    let t = match rest.first() {
                        Some(&init) => {
                            let got = self.expr(init, ty);
                            match ty {
                                Some(w) => {
                                    self.coerce(init, got, w);
                                    w
                                }
                                None => got,
                            }
                        }
                        None => ty.unwrap_or_else(|| self.infer.fresh()),
                    };
                    if let Some(&els) = rest.get(1) {
                        let never = self.mk(TyKind::Never);
                        let et = self.expr(els, Some(never));
                        self.coerce(els, et, never);
                    }
                    self.pat(pat, t, Mutbl::Not, false);
                }
                Kind::ExprStmt => {
                    let Some(&e) = self.kids(s).last() else { continue };
                    let t = self.expr(e, None);
                    if self.tcx.kind(self.infer.shallow(t)) == TyKind::Never {
                        diverges = true;
                    }
                }
                Kind::EmptyStmt => {}
                k if is_expr(k) => {
                    let is_tail = Some(i) == last;
                    let t = self.expr(s, if is_tail { expect } else { None });
                    if is_tail {
                        return t;
                    }
                    if self.tcx.kind(self.infer.shallow(t)) == TyKind::Never {
                        diverges = true;
                    }
                }
                _ => {}
            }
        }
        if diverges { self.mk(TyKind::Never) } else { self.unit() }
    }

    // ---- patterns ----

    /// Checks pattern `p` against `t`, binding its names. `by_ref` and
    /// `bm` are the default binding mode (`ref` or `ref mut` after the
    /// pattern passed through a reference).
    fn pat(&mut self, p: NodeId, t: Ty, bm: Mutbl, by_ref: bool) {
        self.types.insert(p, t);
        let tcx = self.tcx;
        let ast = self.ast;
        let kind = ast.kind(p);
        // A pattern that is not a binding or a reference pattern matches
        // through references, binding by reference.
        let peels = !matches!(kind, Kind::IdentPat | Kind::WildPat | Kind::RefPat | Kind::RestPat | Kind::OrPat | Kind::ParenPat)
            || kind == Kind::IdentPat && self.r.res_of(self.k as usize, self.file as usize, p).is_some();
        if peels && let TyKind::Ref(m, inner) = tcx.kind(self.infer.shallow(t)) {
            let bm = if by_ref && bm == Mutbl::Not { Mutbl::Not } else { m };
            return self.pat(p, inner, bm, true);
        }
        let kids = self.kids(p);
        match kind {
            Kind::IdentPat => {
                if let Some(res) = self.r.res_of(self.k as usize, self.file as usize, p) {
                    // A constant or unit struct or variant.
                    let res = res.clone();
                    let vt = match res {
                        Res::Def(d) => match self.r.defs[d.0 as usize].kind {
                            DefKind::Variant | DefKind::Struct => {
                                let args = self.infer.instantiate(&tcx.all_params(d));
                                let owner = if self.r.defs[d.0 as usize].kind == DefKind::Variant { tcx.generics[d.0 as usize].parent.unwrap_or(d) } else { d };
                                self.mk(TyKind::Adt(owner, tcx.list(args)))
                            }
                            _ => tcx.type_of(d).unwrap_or_else(|| tcx.error()),
                        },
                        _ => tcx.error(),
                    };
                    self.infer.unify(vt, t);
                    return;
                }
                let has_ref = kids.iter().any(|&c| ast.kind(c) == Kind::Keyword && ast.token(c).0 == "ref");
                let is_mut = kids.iter().any(|&c| ast.kind(c) == Kind::Keyword && ast.token(c).0 == "mut");
                let bt = if has_ref {
                    self.mk(TyKind::Ref(if is_mut { Mutbl::Mut } else { Mutbl::Not }, t))
                } else if by_ref {
                    self.mk(TyKind::Ref(bm, t))
                } else {
                    t
                };
                self.types.insert(p, bt);
                if let Some(&sub) = kids.iter().find(|&&c| !matches!(ast.kind(c), Kind::Keyword | Kind::Name)) {
                    self.pat(sub, t, bm, by_ref);
                }
            }
            Kind::WildPat | Kind::RestPat | Kind::NeverPat | Kind::PatMacro => {}
            Kind::ParenPat => self.pat(kids[0], t, bm, by_ref),
            Kind::OrPat => {
                for &k in &kids {
                    self.pat(k, t, bm, by_ref);
                }
            }
            Kind::RefPat => {
                let inner = *kids.last().unwrap();
                let m = if kids.iter().any(|&c| ast.kind(c) == Kind::Keyword) { Mutbl::Mut } else { Mutbl::Not };
                let it = self.infer.fresh();
                let rt = self.mk(TyKind::Ref(m, it));
                if !self.infer.unify(rt, t) {
                    self.err(p, "reference pattern on a value that is not a reference");
                }
                self.pat(inner, it, Mutbl::Not, false);
            }
            Kind::TuplePat => {
                let rest = kids.iter().position(|&c| ast.kind(c) == Kind::RestPat);
                let st = self.infer.shallow(t);
                let elems: Vec<Ty> = match tcx.kind(st) {
                    TyKind::Tuple(l) => tcx.tys(l),
                    _ if rest.is_none() => {
                        let v: Vec<Ty> = kids.iter().map(|_| self.infer.fresh()).collect();
                        let tt = self.mk(TyKind::Tuple(tcx.list(v.clone())));
                        self.infer.unify(tt, t);
                        v
                    }
                    _ => {
                        self.err(p, "tuple pattern with `..` on a value whose type is not yet known");
                        return;
                    }
                };
                self.positional(&kids, &elems, rest, bm, by_ref);
            }
            Kind::TupleStructPat => {
                let fields = self.ctor_fields(p, kids[0], t);
                let pats = &kids[1..];
                let rest = pats.iter().position(|&c| ast.kind(c) == Kind::RestPat);
                self.positional(pats, &fields, rest, bm, by_ref);
            }
            Kind::StructPat => {
                let (adt, fields) = self.named_fields(kids[0], t);
                let _ = adt;
                for &f in &kids[1..] {
                    if ast.kind(f) != Kind::FieldPat {
                        continue;
                    }
                    let fk = self.kids(f);
                    let (name, sub) = if fk.len() == 2 {
                        (ast.token(fk[0]).0, fk[1])
                    } else {
                        let sub = fk[0];
                        (ast.child(sub, Kind::Name).map_or("", |n| ast.token(n).0), sub)
                    };
                    let ft = fields.iter().find(|x| x.0 == name).map(|x| x.1).unwrap_or_else(|| self.err(f, "no such field in pattern"));
                    self.pat(sub, ft, bm, by_ref);
                }
            }
            Kind::PathPat => {
                let vt = self.path_value(p, kids[0]);
                self.infer.unify(vt, t);
            }
            Kind::LitPat | Kind::RangePat => {
                for &k in &kids {
                    match ast.kind(k) {
                        Kind::Literal => {
                            let lt = self.literal(k, Some(t));
                            self.infer.unify(lt, t);
                        }
                        Kind::Unary => {
                            let e = self.ast.children(k)[1];
                            let lt = self.literal(e, Some(t));
                            self.infer.unify(lt, t);
                        }
                        Kind::LitPat | Kind::PathPat => self.pat(k, t, bm, by_ref),
                        _ => {}
                    }
                }
            }
            Kind::SlicePat => {
                let st = self.infer.shallow(t);
                let elem = match tcx.kind(st) {
                    TyKind::Array(e, _) | TyKind::Slice(e) => e,
                    _ => {
                        let e = self.infer.fresh();
                        let sl = self.mk(TyKind::Slice(e));
                        self.infer.unify(sl, t);
                        e
                    }
                };
                for &k in &kids {
                    if ast.kind(k) == Kind::RestPat {
                        continue;
                    }
                    // `rest @ ..` binds the rest as a slice.
                    if ast.kind(k) == Kind::IdentPat && self.kids(k).iter().any(|&c| ast.kind(c) == Kind::RestPat) {
                        let sl = self.mk(TyKind::Slice(elem));
                        self.pat(k, sl, bm, by_ref);
                        continue;
                    }
                    self.pat(k, elem, bm, by_ref);
                }
            }
            Kind::BoxPat => {
                let inner = self.infer.fresh();
                if let Some(b) = self.lang_adt("owned_box", vec![inner]) {
                    self.infer.unify(b, t);
                }
                self.pat(kids[0], inner, bm, by_ref);
            }
            Kind::ConstBlock => {
                let ct = self.expr(kids[0], Some(t));
                self.infer.unify(ct, t);
            }
            _ => {
                self.err(p, "unsupported pattern");
            }
        }
    }

    /// Patterns against positional fields, with `..` at `rest`.
    fn positional(&mut self, pats: &[NodeId], fields: &[Ty], rest: Option<usize>, bm: Mutbl, by_ref: bool) {
        for (i, &sp) in pats.iter().enumerate() {
            if Some(i) == rest {
                continue;
            }
            let fi = match rest {
                Some(r) if i > r => fields.len().saturating_sub(pats.len() - i),
                _ => i,
            };
            let ft = fields.get(fi).copied().unwrap_or_else(|| self.tcx.error());
            self.pat(sp, ft, bm, by_ref);
        }
    }

    /// The field types of a tuple struct or variant named by `path`, made
    /// to match `t`.
    fn ctor_fields(&mut self, p: NodeId, path: NodeId, t: Ty) -> Vec<Ty> {
        let ct = self.path_value(p, path);
        match self.tcx.kind(self.infer.shallow(ct)) {
            TyKind::FnDef(d, l) => match self.callable_sig(d, l) {
                Some((inputs, output)) => {
                    if !self.infer.unify(output, t) {
                        self.err(p, "pattern of another type");
                    }
                    inputs
                }
                None => Vec::new(),
            },
            _ => Vec::new(),
        }
    }

    /// The named fields of the struct or variant `path` names, made to
    /// match `t`.
    fn named_fields(&mut self, path: NodeId, t: Ty) -> (Ty, Vec<(&'s str, Ty)>) {
        let tcx = self.tcx;
        let (owner, variant) = match self.res(path) {
            Some(Res::Def(d)) => match self.r.defs[d.0 as usize].kind {
                DefKind::Struct | DefKind::Union => (Some(*d), tcx.adts.get(d).and_then(|a| a.variants.first().cloned())),
                DefKind::Variant => {
                    let e = tcx.generics[d.0 as usize].parent;
                    (e, e.and_then(|e| tcx.adts.get(&e)).and_then(|a| a.variants.iter().find(|v| v.def == Some(*d)).cloned()))
                }
                _ => (None, None),
            },
            Some(Res::SelfTy(f, s)) => match tcx.self_ty.get(&(self.k, *f, *s)).map(|&t| tcx.kind(t)) {
                Some(TyKind::Adt(a, _)) => (Some(a), tcx.adts.get(&a).and_then(|x| x.variants.first().cloned())),
                _ => (None, None),
            },
            Some(Res::Partial(base, 1)) => {
                let name = self.last_segment_name(path);
                let base = (**base).clone();
                match self.base_ty(path, &base).map(|b| tcx.kind(self.infer.shallow(b))) {
                    Some(TyKind::Adt(e, _)) => (Some(e), tcx.adts.get(&e).and_then(|a| a.variants.iter().find(|v| v.name == name).cloned())),
                    _ => (None, None),
                }
            }
            _ => (None, None),
        };
        let Some(owner) = owner else { return (tcx.error(), Vec::new()) };
        let params = tcx.all_params(owner);
        let args = self.infer.instantiate(&params);
        let at = self.mk(TyKind::Adt(owner, tcx.list(args.clone())));
        self.infer.unify(at, t);
        let fields = variant.map(|v| v.fields.iter().map(|f| (f.name, subst(tcx, f.ty, &params, &args))).collect()).unwrap_or_default();
        (at, fields)
    }
}
