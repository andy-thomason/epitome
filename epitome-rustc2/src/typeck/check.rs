//! Checking one body.

use super::{Adjust, BindingMode, BodyResult};
use super::infer::{Infer, VarKind};
use super::traits::TraitImpls;
use crate::ast::{Ast, Kind, NodeId};
use crate::hash::{FastMap, FastSet};
use crate::resolve::{DefId, DefKind, Ns, Res, Resolver};
use crate::session::ParsedCrate;
use crate::ty::collect::subst;
use crate::ty::lower::{Env, Lower, is_type};
use crate::ty::{Const, CtorKind, FloatTy, IntTy, Mutbl, Pred, Ty, TyCtxt, TyKind, UintTy};

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

/// Impls indexed for lookup: inherent impls by their self type's head,
/// trait impls by trait and head, and traits' items by name.
pub struct Index<'s> {
    pub inherent: FastMap<Head, Vec<usize>>,
    pub trait_impls: FastMap<DefId, TraitImpls>,
    /// Traits' functions and constants by name: (trait, item).
    pub trait_items: FastMap<&'s str, Vec<(DefId, DefId)>>,
}

impl<'s> Index<'s> {
    pub fn new(tcx: &TyCtxt, r: &Resolver<'s>) -> Self {
        let mut inherent: FastMap<Head, Vec<usize>> = FastMap::default();
        let mut trait_impls: FastMap<DefId, TraitImpls> = FastMap::default();
        for (i, imp) in tcx.impls.iter().enumerate() {
            // An impl of something not found would match anything.
            if tcx.has_error(imp.self_ty) {
                continue;
            }
            match &imp.trait_ref {
                None => {
                    if let Some(h) = head(tcx, imp.self_ty) {
                        inherent.entry(h).or_default().push(i);
                    }
                }
                Some(t) => {
                    if tcx.trait_ref_has_error(t) || imp.negative {
                        continue;
                    }
                    let e = trait_impls.entry(t.def).or_default();
                    e.all.push(i);
                    match head(tcx, imp.self_ty) {
                        Some(h) => e.by_head.entry(h).or_default().push(i),
                        None => e.blanket.push(i),
                    }
                }
            }
        }
        let mut trait_items: FastMap<&'s str, Vec<(DefId, DefId)>> = FastMap::default();
        for (i, d) in r.defs.iter().enumerate() {
            if !matches!(d.kind, DefKind::AssocFn | DefKind::AssocConst) {
                continue;
            }
            let Some(m) = d.parent else { continue };
            let Some(t) = r.modules[m.0 as usize].def else { continue };
            if r.defs[t.0 as usize].kind == DefKind::Trait {
                trait_items.entry(d.name).or_default().push((t, DefId(i as u32)));
            }
        }
        Index { inherent, trait_impls, trait_items }
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn check_body<'s>(
    tcx: &TyCtxt<'s>,
    r: &Resolver<'s>,
    crates: &[ParsedCrate<'s>],
    index: &Index<'s>,
    owner: DefId,
    k: u32,
    file: u32,
    item: NodeId,
) -> Option<BodyResult> {
    let ast = &crates[k as usize].files[file as usize].ast;
    let kind = r.defs[owner.0 as usize].kind;
    let mut cx = FnCtxt::new(tcx, r, crates, index, owner, k, file);
    match kind {
        DefKind::Fn | DefKind::AssocFn => {
            let body = ast.children(item).iter().copied().find(|&c| ast.kind(c) == Kind::Block)?;
            let sig = tcx.sig_of(tcx.fn_sig[owner.0 as usize]?);
            let inputs = tcx.tys(sig.inputs);
            let inputs: Vec<Ty> = inputs.into_iter().map(|t| cx.fill_holes(t)).collect();
            let out = cx.fill_holes(sig.output);
            // `impl Trait`: a variable bound by its bounds, so that a
            // closure returned gets its signature from them.
            cx.ret = cx.opaque_vars(out, true);
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

pub(super) struct FnCtxt<'a, 's> {
    pub(super) tcx: &'a TyCtxt<'s>,
    pub(super) r: &'a Resolver<'s>,
    pub(super) env: Env<'a, 's>,
    pub(super) infer: Infer<'a, 's>,
    pub(super) index: &'a Index<'s>,
    pub(super) k: u32,
    file: u32,
    ast: &'a Ast<'s>,
    pub(super) owner: DefId,
    /// How deep impls' `where` clauses are checked in selection: shallow
    /// in bodies, deeper for concrete types after type checking.
    pub(super) where_depth: u32,
    /// Types of expressions and patterns, unresolved until the end.
    types: FastMap<NodeId, Ty>,
    /// What each method call (or overloaded operator) calls, and the
    /// arguments, unresolved until the end.
    pub(super) methods: Vec<(NodeId, DefId, Vec<Ty>)>,
    /// The constant each path to one names, and its arguments.
    pub(super) consts: Vec<(NodeId, DefId, Vec<Ty>)>,
    adjustments: FastMap<NodeId, Vec<Adjust>>,
    pat_derefs: FastMap<NodeId, u32>,
    bindings: FastMap<NodeId, BindingMode>,
    /// The type of an adjusted expression after its adjustments.
    adjusted: FastMap<NodeId, Ty>,
    /// Calls a node makes without writing them.
    implicit: FastMap<NodeId, Vec<(DefId, Vec<Ty>)>>,
    /// The `impl Trait` types this body defines, and the variables that
    /// stand for them.
    hidden: Vec<(crate::ty::OpaqueId, Ty)>,
    /// The closures written in this body.
    closures: Vec<crate::ty::ClosureId>,
    pub(super) ret: Ty,
    loops: Vec<Loop<'s>>,
    errors: FastMap<String, usize>,
    /// Nodes already counted as errors.
    reported: FastSet<NodeId>,
    /// What the body may assume (`traits.rs`).
    pub(super) env_preds: Vec<Pred>,
    /// Predicates on variables, from generic items used in the body.
    pub(super) obligations: Vec<Pred>,
    /// Projections waiting for their types to be known: `<args>::assoc`
    /// is the variable.
    pub(super) pending: Vec<(DefId, crate::ty::List, Ty)>,
    pub(super) scope_cache: FastMap<DefId, bool>,
}

impl<'a, 's> FnCtxt<'a, 's> {
    /// A context for checking the body of `owner` (in crate `k`, `file`),
    /// or, with an owner that has no predicates, for selecting outside
    /// any body.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn new(
        tcx: &'a TyCtxt<'s>,
        r: &'a Resolver<'s>,
        crates: &'a [ParsedCrate<'s>],
        index: &'a Index<'s>,
        owner: DefId,
        k: u32,
        file: u32,
    ) -> Self {
        let ast = &crates[k as usize].files[file as usize].ast;
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
            consts: Vec::new(),
            adjustments: FastMap::default(),
            pat_derefs: FastMap::default(),
            bindings: FastMap::default(),
            adjusted: FastMap::default(),
            implicit: FastMap::default(),
            hidden: Vec::new(),
            closures: Vec::new(),
            where_depth: 2,
            ret: tcx.unit(),
            loops: Vec::new(),
            errors: FastMap::default(),
            reported: FastSet::default(),
            env_preds: Vec::new(),
            obligations: Vec::new(),
            pending: Vec::new(),
            scope_cache: FastMap::default(),
        };
        cx.env_predicates();
        cx
    }

    /// What closure `c` (of this body) captures: every local of an
    /// enclosing body it uses, in order of first use. A capture is by value
    /// in a `move` closure, or where the body moves a value that is not
    /// `Copy` out of it (directly, or into a `move` closure inside it);
    /// otherwise by reference. Whole variables are captured, not the
    /// fields used, which behaves the same while nothing is dropped.
    fn captures(&mut self, c: crate::ty::ClosureId) -> Vec<super::Capture> {
        let ast = self.ast;
        let (_, _, node) = self.tcx.closure_at(c);
        let is_move = |n: NodeId| ast.children(n).iter().any(|&k| ast.kind(k) == Kind::Keyword && ast.token(k).0 == "move");
        // The closure's nodes in source order, each with its parent and
        // the innermost closure it is in.
        let mut nodes: Vec<(NodeId, NodeId, NodeId)> = Vec::new();
        let mut stack = vec![(node, node, node)];
        while let Some((n, parent, closure)) = stack.pop() {
            nodes.push((n, parent, closure));
            let inner = if ast.kind(n) == Kind::Closure { n } else { closure };
            for &k in ast.children(n).iter().rev() {
                stack.push((k, n, inner));
            }
        }
        let inside: FastSet<NodeId> = nodes.iter().map(|x| x.0).collect();
        let parent_of: FastMap<NodeId, NodeId> = nodes.iter().map(|x| (x.0, x.1)).collect();
        let copy = self.lang("copy");
        let mut out: Vec<super::Capture> = Vec::new();
        for &(n, parent, closure) in &nodes {
            let path = match ast.kind(n) {
                Kind::PathExpr => ast.children(n).first().copied(),
                // `S { a }`: its name is a use of the local `a`, by value.
                Kind::FieldInit => match ast.children(n).iter().copied().filter(|&c| ast.kind(c) != Kind::Attribute).collect::<Vec<_>>()[..] {
                    [name] if ast.kind(name) == Kind::Name => Some(name),
                    _ => None,
                },
                _ => None,
            };
            let Some(path) = path else { continue };
            let Some(Res::Local(p)) = self.res(path) else { continue };
            let p = *p;
            if inside.contains(&p) {
                continue;
            }
            // Whether this use takes the value (rather than a place).
            let mut child = n;
            let mut up = parent;
            while ast.kind(up) == Kind::Paren && up != node {
                child = up;
                up = parent_of.get(&up).copied().unwrap_or(node);
            }
            let first = ast.children(up).first() == Some(&child);
            let borrowed = matches!(self.adjustments.get(&child).and_then(|a| a.first()), Some(Adjust::Borrow(_) | Adjust::Deref));
            let place_use = borrowed
                || match ast.kind(up) {
                    Kind::Borrow | Kind::Unary => true,
                    Kind::Field | Kind::Index | Kind::Match | Kind::MethodCall => first,
                    Kind::Let => !first,
                    Kind::Binary => first && ast.children(up).get(1).is_some_and(|&o| ast.text(o).ends_with('=') && !matches!(ast.text(o).as_str(), "==" | "!=" | "<=" | ">=")),
                    _ => false,
                };
            let value_use = !place_use || closure != node && is_move(closure);
            let by_value = is_move(node)
                || value_use && {
                    let t = self.types.get(&p).copied().map(|t| self.infer.resolve(t));
                    match (t, copy) {
                        (Some(t), Some(cp)) => match self.tcx.kind(t) {
                            TyKind::Ref(Mutbl::Not, _) => false,
                            _ => !self.select(t, cp, &[], 0).holds(),
                        },
                        _ => false,
                    }
                };
            match out.iter_mut().find(|x| x.local == p) {
                Some(x) => x.by_value |= by_value,
                None => out.push(super::Capture { local: p, by_value }),
            }
        }
        out
    }

    fn finish(mut self) -> BodyResult {
        self.process_pending();
        self.infer.fallback();
        self.process_pending();
        let captures: Vec<(crate::ty::ClosureId, Vec<super::Capture>)> =
            self.closures.clone().into_iter().map(|c| (c, self.captures(c))).collect();
        let mut types = Vec::with_capacity(self.types.len());
        let (mut error_nodes, mut unknown_nodes) = (0, 0);
        let mut nodes: Vec<(NodeId, Ty)> = self.types.iter().map(|(&n, &t)| (n, t)).collect();
        nodes.sort_by_key(|x| x.0);
        for (n, t) in nodes {
            let t = self.infer.resolve(t);
            // Projections whose types are now known.
            let t = self.normalize(t, 1);
            let t = self.infer.resolve(t);
            if self.tcx.has_error(t) {
                error_nodes += 1;
            } else if self.tcx.has_infer(t) {
                unknown_nodes += 1;
            }
            types.push((n, t));
        }
        let methods = std::mem::take(&mut self.methods);
        let methods = methods
            .into_iter()
            .map(|(n, d, args)| {
                let args: Vec<Ty> = args
                    .into_iter()
                    .map(|t| {
                        let t = self.infer.resolve(t);
                        let t = self.normalize(t, 1);
                        self.infer.resolve(t)
                    })
                    .collect();
                (n, d, self.tcx.list(args))
            })
            .collect();
        let consts = std::mem::take(&mut self.consts);
        let consts = consts
            .into_iter()
            .map(|(n, d, args)| {
                let args: Vec<Ty> = args
                    .into_iter()
                    .map(|t| {
                        let t = self.infer.resolve(t);
                        let t = self.normalize(t, 1);
                        self.infer.resolve(t)
                    })
                    .collect();
                (n, d, self.tcx.list(args))
            })
            .collect();
        let mut adjustments: Vec<(NodeId, Vec<Adjust>)> = std::mem::take(&mut self.adjustments).into_iter().collect();
        adjustments.sort_by_key(|x| x.0);
        let mut pat_derefs: Vec<(NodeId, u32)> = std::mem::take(&mut self.pat_derefs).into_iter().collect();
        pat_derefs.sort_by_key(|x| x.0);
        let mut bindings: Vec<(NodeId, BindingMode)> = std::mem::take(&mut self.bindings).into_iter().collect();
        bindings.sort_by_key(|x| x.0);
        let fin = |cx: &mut Self, t: Ty| {
            let t = cx.infer.resolve(t);
            let t = cx.normalize(t, 1);
            cx.infer.resolve(t)
        };
        let mut adjusted: Vec<(NodeId, Ty)> = std::mem::take(&mut self.adjusted).into_iter().collect();
        adjusted.sort_by_key(|x| x.0);
        for a in &mut adjusted {
            a.1 = fin(&mut self, a.1);
        }
        let mut implicit: Vec<(NodeId, DefId, crate::ty::List)> = Vec::new();
        let mut calls: Vec<_> = std::mem::take(&mut self.implicit).into_iter().collect();
        calls.sort_by_key(|x| x.0);
        for (n, cs) in calls {
            for (d, args) in cs {
                let args: Vec<Ty> = args.into_iter().map(|t| fin(&mut self, t)).collect();
                implicit.push((n, d, self.tcx.list(args)));
            }
        }
        let hidden_vars = std::mem::take(&mut self.hidden);
        let hidden: Vec<(crate::ty::OpaqueId, Ty)> = hidden_vars.into_iter().map(|(o, v)| (o, fin(&mut self, v))).collect();
        BodyResult {
            hidden,
            closures: std::mem::take(&mut self.closures).into_iter().map(|c| (c, self.owner)).collect(),
            captures,
            k: self.k,
            file: self.file,
            types,
            methods,
            consts,
            adjustments,
            pat_derefs,
            bindings,
            adjusted,
            implicit,
            errors: self.errors,
            error_nodes,
            unknown_nodes,
        }
    }

    /// Counts why a type is an error (once per node), and gives `Error`.
    fn err(&mut self, n: NodeId, why: &str) -> Ty {
        if self.reported.insert(n) {
            *self.errors.entry(why.to_string()).or_default() += 1;
            // `EPITOME_TYPECK_WHY=text`: print the errors whose reason
            // has `text` in it, with where they are.
            static WHY: std::sync::OnceLock<Option<String>> = std::sync::OnceLock::new();
            if let Some(w) = WHY.get_or_init(|| std::env::var("EPITOME_TYPECK_WHY").ok())
                && why.contains(w.as_str())
            {
                let mut text = self.ast.text(n).replace('\n', " ");
                text.truncate((0..=80.min(text.len())).rev().find(|&i| text.is_char_boundary(i)).unwrap_or(0));
                eprintln!("{why}: {} in {}", text, self.r.def_path(self.owner));
            }
        }
        self.tcx.error()
    }

    pub(super) fn mk(&self, k: TyKind) -> Ty {
        self.tcx.mk(k)
    }

    pub(super) fn unit(&self) -> Ty {
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

    /// A written generic argument: a type, or a constant (`4`, `{ N + 1
    /// }`, `{ Ordering::Relaxed }`) as a `ConstArg`.
    fn written_arg(&mut self, n: NodeId) -> Ty {
        if !matches!(self.ast.kind(n), Kind::Literal | Kind::Unary | Kind::Block) {
            return self.written_ty(n);
        }
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
        let c = l.konst(n);
        self.mk(TyKind::ConstArg(c))
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
        // A reference to a type waiting on a projection (a method's `&mut
        // Self::Target`) may be known by now, and must be before it is
        // unified with what is wanted, or an unsizing would be missed.
        if !self.pending.is_empty()
            && let (TyKind::Ref(_, x) | TyKind::Ptr(_, x), TyKind::Ref(_, y) | TyKind::Ptr(_, y)) =
                (self.tcx.kind(self.infer.shallow(got)), self.tcx.kind(self.infer.shallow(want)))
            && self.infer.var_kind(self.infer.shallow(x)).is_some()
            && self.may_be_unsized(y)
        {
            self.process_pending();
        }
        let g = self.infer.shallow(got);
        let w = self.infer.shallow(want);
        let tcx = self.tcx;
        match (tcx.kind(g), tcx.kind(w)) {
            (TyKind::Never, TyKind::Never) => return true,
            (TyKind::Never, _) => {
                self.adjust(n, vec![Adjust::NeverToAny], w);
                return true;
            }
            // Reborrowing, unsizing (`&[T; N]` to `&[T]`, `&T` to
            // `&dyn Trait`), and deref coercion (`&String` to `&str`).
            (TyKind::Ref(m, x), TyKind::Ref(m2, y)) if m == m2 || m == Mutbl::Mut => {
                let p = self.probe();
                if self.infer.unify(x, y) {
                    return true;
                }
                self.rollback(p);
                if self.unsizes_to(x, y) {
                    self.adjust(n, vec![Adjust::Unsize], w);
                    return true;
                }
                self.rollback(p);
                if let Some(adj) = self.deref_coerces(x, y, m2) {
                    self.adjust(n, adj, w);
                    return true;
                }
                self.rollback(p);
            }
            (TyKind::Ref(m, x), TyKind::Ptr(m2, y)) | (TyKind::Ptr(m, x), TyKind::Ptr(m2, y)) if m == m2 || m == Mutbl::Mut => {
                let p = self.probe();
                if self.infer.unify(x, y) {
                    return true;
                }
                if self.unsizes_to(x, y) {
                    self.adjust(n, vec![Adjust::Unsize], w);
                    return true;
                }
                self.rollback(p);
            }
            (TyKind::Adt(a, x), TyKind::Adt(b, y)) if a == b && Some(a) == self.lang("owned_box") => {
                let (x, y) = (tcx.tys(x)[0], tcx.tys(y)[0]);
                let p = self.probe();
                if self.infer.unify(x, y) {
                    return true;
                }
                if self.unsizes_to(x, y) {
                    self.adjust(n, vec![Adjust::Unsize], w);
                    return true;
                }
                self.rollback(p);
            }
            (TyKind::FnDef(d, l), TyKind::FnPtr(s)) => {
                if let Some((inputs, output)) = self.callable_sig(d, l) {
                    let s = tcx.sig_of(s);
                    let want_in = tcx.tys(s.inputs);
                    let ok = inputs.len() == want_in.len()
                        && inputs.iter().zip(&want_in).all(|(&a, &b)| self.infer.unify(a, b))
                        && self.infer.unify(output, s.output);
                    if ok {
                        self.adjust(n, vec![Adjust::ReifyFnPointer], w);
                    }
                    return ok;
                }
            }
            (TyKind::Closure(_, cs, _), TyKind::FnPtr(s)) => {
                let (c, s) = (tcx.sig_of(cs), tcx.sig_of(s));
                let ok = self.infer.unify(self.mk(TyKind::FnPtr(tcx.sig(crate::ty::FnSig { unsafety: s.unsafety, abi: s.abi, ..c }))), w);
                if ok {
                    self.adjust(n, vec![Adjust::ClosureFnPointer], w);
                }
                return ok;
            }
            _ => {}
        }
        if self.infer.unify(g, w) {
            return true;
        }
        self.err(n, "mismatched types");
        false
    }

    /// Whether `t` is, or holds as an argument, a slice, `str` or `dyn`:
    /// something a pointer might be unsized to.
    fn may_be_unsized(&self, t: Ty) -> bool {
        let tcx = self.tcx;
        match tcx.kind(self.infer.shallow(t)) {
            TyKind::Slice(_) | TyKind::Str | TyKind::Dyn(_) => true,
            TyKind::Adt(_, l) | TyKind::Tuple(l) => tcx.tys(l).into_iter().any(|x| self.may_be_unsized(x)),
            _ => false,
        }
    }

    /// `recv.name::<A, B>(..)`: the written arguments for the method's own
    /// parameters (the last ones of the arguments recorded for `n`).
    fn method_turbofish(&mut self, n: NodeId, item: DefId, seg: NodeId) {
        let ast = self.ast;
        let Some(g) = ast.child(seg, Kind::GenericArgs) else { return };
        let written: Vec<NodeId> = ast.children(g).iter().copied().filter(|&a| is_generic_arg(ast.kind(a))).collect();
        let own = self.tcx.generics[item.0 as usize].own.len();
        let Some(args) = self.methods.iter().rev().find(|m| m.0 == n).map(|m| m.2.clone()) else { return };
        let start = args.len().saturating_sub(own);
        for (i, a) in written.into_iter().enumerate() {
            if let Some(&v) = args.get(start + i) {
                let t = self.written_arg(a);
                self.infer.unify(v, t);
            }
        }
    }

    /// Records adjustments of the expression `n`, after any it has, and
    /// the type they give it.
    fn adjust(&mut self, n: NodeId, adj: Vec<Adjust>, to: Ty) {
        if !adj.is_empty() {
            self.adjustments.entry(n).or_default().extend(adj);
            self.adjusted.insert(n, to);
        }
    }

    /// Records a call that `n` makes without writing it (`for`'s
    /// `IntoIterator::into_iter` and `Iterator::next`, `?`'s
    /// `Try::branch` and `FromResidual::from_residual`).
    pub(super) fn implicit_call(&mut self, n: NodeId, tr: DefId, method: &str, args: Vec<Ty>) {
        if let Some(m) = self.trait_item(tr, method, Ns::Value) {
            let full = {
                let rest = self.full_trait_args(tr, args[0], args[1..].to_vec());
                let mut a = vec![args[0]];
                a.extend(rest);
                a
            };
            self.implicit.entry(n).or_default().push((m, full));
        }
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
    pub(super) fn callable_sig(&mut self, d: DefId, l: crate::ty::List) -> Option<(Vec<Ty>, Ty)> {
        let tcx = self.tcx;
        let args = tcx.tys(l);
        let params = tcx.all_params(d);
        match self.r.defs[d.0 as usize].kind {
            DefKind::Fn | DefKind::AssocFn | DefKind::ForeignFn => {
                let sig = tcx.sig_of(tcx.fn_sig[d.0 as usize]?);
                self.oblige_item(d, &args);
                // `impl Trait` for a parameter: a variable, with the bounds
                // as obligations. For the result, the opaque type itself.
                let inst = |cx: &mut Self, t: Ty, input: bool| -> Ty {
                    let t = subst(tcx, t, &params, &args);
                    let t = if input { cx.opaque_vars(t, false) } else { t };
                    cx.normalize(t, 0)
                };
                let inputs = tcx.tys(sig.inputs).into_iter().map(|t| inst(self, t, true)).collect();
                let out = inst(self, sig.output, false);
                Some((inputs, out))
            }
            DefKind::Struct => {
                let v = tcx.adts.get(&d)?.variants.first()?.clone();
                let inputs = v.fields.iter().map(|f| subst(tcx, f.ty, &params, &args)).collect::<Vec<_>>();
                let inputs = inputs.into_iter().map(|t| self.normalize(t, 0)).collect();
                Some((inputs, self.mk(TyKind::Adt(d, l))))
            }
            DefKind::Variant => {
                let e = tcx.generics[d.0 as usize].parent?;
                let v = tcx.adts.get(&e)?.variants.iter().find(|v| v.def == Some(d))?.clone();
                let inputs = v.fields.iter().map(|f| subst(tcx, f.ty, &params, &args)).collect::<Vec<_>>();
                let inputs = inputs.into_iter().map(|t| self.normalize(t, 0)).collect();
                Some((inputs, self.mk(TyKind::Adt(e, l))))
            }
            _ => None,
        }
    }

    /// `t` with each `impl Trait` in it (at the top, or behind a
    /// reference) replaced by a variable bound by its bounds. In the body
    /// that defines them, the variables are what they stand for (their
    /// hidden types), which are recorded.
    fn opaque_vars(&mut self, t: Ty, defining: bool) -> Ty {
        let tcx = self.tcx;
        match tcx.kind(t) {
            TyKind::Opaque(o, l) => {
                let v = self.infer.fresh();
                let def = tcx.opaque(o);
                let (params, args) = (tcx.all_params(def.owner), tcx.tys(l));
                let mut preds = Vec::new();
                for tr in def.bounds {
                    let tr = super::traits::subst_trait_ref(tcx, &tr, &params, &args);
                    self.elaborate(v, &tr, &mut preds);
                }
                for p in preds {
                    self.oblige(p);
                }
                if defining {
                    self.hidden.push((o, v));
                }
                v
            }
            TyKind::Ref(m, x) => {
                let x = self.opaque_vars(x, defining);
                self.mk(TyKind::Ref(m, x))
            }
            _ => t,
        }
    }

    /// Arguments for item `d`'s parameters: fresh variables, with those
    /// written in the path's last segment (`f::<u8>`) for its own.
    fn item_args(&mut self, d: DefId, path: NodeId) -> Vec<Ty> {
        let tcx = self.tcx;
        let ast = self.ast;
        let all = tcx.all_params(d);
        let mut args = self.infer.instantiate(&all);
        let own = tcx.generics[d.0 as usize].own.len();
        let own_start = all.len() - own;
        let variant = self.r.defs[d.0 as usize].kind == DefKind::Variant;
        let in_trait = tcx.generics[d.0 as usize].parent.is_some_and(|p| self.r.defs[p.0 as usize].kind == DefKind::Trait);
        let segs: Vec<NodeId> = ast.children(path).iter().copied().filter(|&s| ast.kind(s) == Kind::Segment).collect();
        let written = |s: Option<&NodeId>| -> Vec<NodeId> {
            s.and_then(|&s| ast.child(s, Kind::GenericArgs))
                .map(|g| ast.children(g).iter().copied().filter(|&a| is_generic_arg(ast.kind(a))).collect())
                .unwrap_or_default()
        };
        // On the last segment: the item's own (a variant's are its
        // enum's: `None::<u8>`). On the one before: its parent's
        // (`Option::<u8>::None`, `Trait::<A>::f` after `Self`).
        let last = written(segs.last());
        let parent = if segs.len() >= 2 { written(segs.get(segs.len() - 2)) } else { Vec::new() };
        let mut put = |cx: &mut Self, start: usize, ts: Vec<NodeId>| {
            for (i, a) in ts.into_iter().enumerate() {
                if start + i < args.len() {
                    args[start + i] = cx.written_arg(a);
                }
            }
        };
        put(self, if variant { 0 } else { own_start }, last);
        put(self, if in_trait { 1 } else { 0 }, parent);
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
                        if matches!(self.r.defs[d.0 as usize].kind, DefKind::Const | DefKind::AssocConst) {
                            self.consts.push((n, d, args.clone()));
                        }
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
                let t = if let Some(q) = self.ast.child(path, Kind::QSelf)
                    && let Some(&tr) = self.ast.children(q).get(1)
                {
                    self.qualified_trait_value(n, q, tr, name)
                } else {
                    self.assoc_value(n, path, &base, name)
                };
                self.path_turbofish(path, t)
            }
            Res::Partial(base, 2) => {
                // `T::Assoc::item`: the associated type, then its item.
                let base = (**base).clone();
                let segs: Vec<NodeId> = self.ast.children(path).iter().copied().filter(|&s| self.ast.kind(s) == Kind::Segment).collect();
                let mid = self.ast.token(segs[segs.len() - 2]).0;
                let name = self.last_segment_name(path);
                let Some(bt) = self.base_ty(path, &base) else { return self.err(n, "path through an associated item") };
                let Some(at) = self.assoc_type_of(bt, mid) else { return self.err(n, "associated type not found") };
                let at = self.norm(at);
                if let Some((item, impl_args)) = self.inherent_item(at, name, Ns::Value) {
                    let own = tcx.generics[item.0 as usize].own.len();
                    let mut args = impl_args;
                    let fresh = (0..own).map(|_| self.infer.fresh()).collect::<Vec<_>>();
                    args.extend(fresh);
                    let t = self.mk(TyKind::FnDef(item, tcx.list(args)));
                    return self.path_turbofish(path, t);
                }
                match self.trait_assoc_value(n, at, name) {
                    Some(t) => self.path_turbofish(path, t),
                    None => self.err(n, "associated item not found"),
                }
            }
            Res::Partial(..) => self.err(n, "path through an associated item"),
            Res::PrimTy(_) => self.err(n, "a type used as a value"),
        }
    }

    /// `Type::f::<A, B>` and `T::f::<A, B>`: the arguments written on the
    /// last segment, for the function's own parameters (the last of its
    /// arguments), which the lookup left as variables.
    fn path_turbofish(&mut self, path: NodeId, t: Ty) -> Ty {
        let tcx = self.tcx;
        let ast = self.ast;
        let TyKind::FnDef(item, l) = tcx.kind(t) else { return t };
        let Some(&seg) = ast.children(path).iter().rev().find(|&&s| ast.kind(s) == Kind::Segment) else { return t };
        let Some(g) = ast.child(seg, Kind::GenericArgs) else { return t };
        let written: Vec<NodeId> = ast.children(g).iter().copied().filter(|&a| is_generic_arg(ast.kind(a))).collect();
        let args = tcx.tys(l);
        let own = tcx.generics[item.0 as usize].own.len();
        let start = args.len().saturating_sub(own);
        for (i, a) in written.into_iter().enumerate().take(own) {
            let w = self.written_arg(a);
            self.infer.unify(args[start + i], w);
        }
        t
    }

    fn last_segment_name(&self, path: NodeId) -> &'s str {
        let ast = self.ast;
        ast.children(path).iter().rev().find(|&&s| ast.kind(s) == Kind::Segment).map_or("", |&s| ast.token(s).0)
    }

    /// `<T as Trait>::name` as a value.
    fn qualified_trait_value(&mut self, n: NodeId, q: NodeId, tr: NodeId, name: &str) -> Ty {
        let tcx = self.tcx;
        let qk = self.ast.children(q).to_vec();
        let self_ty = self.written_ty(qk[0]);
        let Some(&tp) = self.ast.children(tr).first() else { return self.err(n, "bad qualified path") };
        let mut l = Lower {
            tcx,
            env: &self.env,
            expanding: FastSet::default(),
            impl_self: None,
            holes: true,
            k: self.k,
            file: self.file,
            owner: self.owner,
            failed: false,
        };
        let Some(trait_ref) = l.trait_ref(tp) else { return self.err(n, "trait in qualified path not found") };
        let Some(item) = self.trait_item(trait_ref.def, name, Ns::Value) else {
            return self.err(n, "no such item in the trait of a qualified path");
        };
        let params = tcx.all_params(item);
        let mut args = vec![self_ty];
        for t in tcx.tys(trait_ref.args) {
            let t = self.fill_holes(t);
            args.push(t);
        }
        while args.len() < params.len() {
            args.push(self.infer.fresh());
        }
        args.truncate(params.len());
        match self.r.defs[item.0 as usize].kind {
            DefKind::AssocFn => self.mk(TyKind::FnDef(item, tcx.list(args))),
            DefKind::AssocConst => match tcx.type_of(item) {
                Some(t) => {
                    self.consts.push((n, item, args.clone()));
                    let t = subst(tcx, t, &params, &args);
                    self.normalize(t, 0)
                }
                None => self.err(n, "constant without a type"),
            },
            _ => self.err(n, "qualified path to something that is not a value"),
        }
    }

    /// The type a path's base names, for `Type::item`.
    fn base_ty(&mut self, path: NodeId, base: &Res) -> Option<Ty> {
        let tcx = self.tcx;
        if let Res::PrimTy("<qualified>") = base
            && let Some(q) = self.ast.child(path, Kind::QSelf)
        {
            let t = self.ast.children(q)[0];
            return Some(self.written_ty(t));
        }
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
        let bt = self.norm(bt);
        let tcx = self.tcx;
        // `Self::Variant` and `Alias::Variant`.
        if let TyKind::Adt(e, l) = tcx.kind(bt)
            && let Some(adt) = tcx.adts.get(&e)
            && adt.kind == crate::ty::AdtKind::Enum
            && let Some(v) = adt.variants.iter().find(|v| v.name == name)
        {
            return match (v.kind, v.def) {
                (CtorKind::Unit, _) => bt,
                (CtorKind::Tuple, Some(vd)) => self.mk(TyKind::FnDef(vd, l)),
                _ => self.err(n, "struct variant used as a value"),
            };
        }
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
                        Some(t) => {
                            self.consts.push((n, item, args.clone()));
                            let t = subst(tcx, t, &tcx.all_params(item), &args);
                            self.normalize(t, 0)
                        }
                        None => self.err(n, "constant without a type"),
                    },
                    _ => self.err(n, "associated item is not a value"),
                }
            }
            None => match self.trait_assoc_value(n, bt, name) {
                Some(t) => t,
                None => self.err(n, "associated item not found"),
            },
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
            let fresh = self.infer.snapshot();
            if self.infer.unify(st, t) {
                return Some((item, args));
            }
            self.infer.rollback(fresh);
            // A self type with a projection among its arguments
            // (`ControlFlow<R, R::Output>`): the other arguments first, which
            // settle the projection's, then the projection, normalised.
            if let (TyKind::Adt(d, l), TyKind::Adt(e, m)) = (tcx.kind(st), tcx.kind(self.infer.shallow(t)))
                && d == e
                && tcx.tys(l).into_iter().any(|x| matches!(tcx.kind(x), TyKind::Projection(..)))
            {
                let (ls, ms) = (tcx.tys(l), tcx.tys(m));
                let plain = ls.iter().zip(&ms).filter(|(a, _)| !matches!(tcx.kind(**a), TyKind::Projection(..))).all(|(&a, &b)| self.infer.unify(a, b));
                let ok = plain
                    && ls.iter().zip(&ms).filter(|(a, _)| matches!(tcx.kind(**a), TyKind::Projection(..))).all(|(&a, &b)| {
                        let n = self.normalize(a, 0);
                        self.infer.unify(n, b)
                    });
                if ok {
                    return Some((item, args));
                }
            }
            self.infer.rollback(s);
        }
        None
    }

    // ---- method calls ----

    /// `recv.name(args)`: for each type autoderef goes through, the
    /// receiver by value, then borrowed, then borrowed mutably; inherent
    /// methods before trait methods at each.
    fn method_call(&mut self, n: NodeId, recv_node: NodeId, recv_ty: Ty, name: &str) -> Option<(DefId, Vec<Ty>, Ty)> {
        let tcx = self.tcx;
        let steps = self.autoderef(recv_ty);
        for k in 0..steps.len() {
            let t = steps[k].0;
            if self.infer.var_kind(t) == Some(VarKind::General) {
                return None;
            }
            for form in 0..3 {
                let (recv, borrow) = match form {
                    0 => (t, None),
                    1 => (self.mk_ref(Mutbl::Not, t), Some(Mutbl::Not)),
                    _ => (self.mk_ref(Mutbl::Mut, t), Some(Mutbl::Mut)),
                };
                // An unsized slice is only reached through a borrow.
                if borrow.is_none() && steps[k].1 == Some(Adjust::Unsize) {
                    continue;
                }
                // Inherent methods first: those of this step's type, or of
                // a type further along whose receiver is this one (`len(&self)`
                // of `IndexRange` takes a `&IndexRange` as it is).
                for j in k..steps.len() {
                    let tj = steps[j].0;
                    if self.infer.var_kind(tj) == Some(VarKind::General) {
                        break;
                    }
                    let s = self.probe();
                    if let Some((item, impl_args)) = self.inherent_item(tj, name, Ns::Value)
                        && self.r.defs[item.0 as usize].kind == DefKind::AssocFn
                        && self.has_self(item)
                    {
                        let own = tcx.generics[item.0 as usize].own.len();
                        let mut args = impl_args;
                        let fresh = (0..own).map(|_| self.infer.fresh()).collect::<Vec<_>>();
                        args.extend(fresh);
                        if let Some((inputs, output)) = self.callable_sig(item, tcx.list(args.clone()))
                            && let Some(&sp) = inputs.first()
                            && self.infer.unify(sp, recv)
                        {
                            self.methods.push((n, item, args));
                            let adj = Self::step_adjustments(&steps, k, borrow);
                            self.adjust(recv_node, adj, recv);
                            return Some((item, inputs[1..].to_vec(), output));
                        }
                    }
                    self.rollback(s);
                }
                if let Some((item, args, inputs, output)) = self.trait_method(recv, name) {
                    self.methods.push((n, item, args));
                    let adj = Self::step_adjustments(&steps, k, borrow);
                    self.adjust(recv_node, adj, recv);
                    return Some((item, inputs, output));
                }
            }
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
                let src = self.expr(kids[0], None);
                // A `_` in the target: a pointer cast keeps the pointee
                // (`r as *const _`); otherwise the context says (`n as _`).
                if tcx.has_infer(self.infer.resolve(target)) {
                    let p = self.probe();
                    let by_context = match expect.map(|e| self.infer.shallow(e)) {
                        Some(e) if self.infer.var_kind(e).is_none() => self.infer.unify(target, e),
                        _ => false,
                    };
                    if !by_context {
                        self.rollback(p);
                        let s = self.norm(src);
                        if let (TyKind::Ref(_, x) | TyKind::Ptr(_, x), TyKind::Ptr(_, y)) = (tcx.kind(s), tcx.kind(self.infer.shallow(target))) {
                            let p = self.probe();
                            if !self.infer.unify(y, x) {
                                self.rollback(p);
                            }
                        }
                    }
                }
                target
            }
            Kind::Binary => self.binary(n, &kids),
            Kind::Range => self.range(n, &kids),
            Kind::Call => self.call(n, &kids, expect),
            Kind::MethodCall => {
                let recv = self.expr(kids[0], None);
                let name = ast.token(kids[1]).0;
                let args = self.kids(kids[2]);
                match self.method_call(n, kids[0], recv, name) {
                    Some((item, inputs, output)) => {
                        self.method_turbofish(n, item, kids[1]);
                        self.check_args(n, &args, &inputs);
                        output
                    }
                    None => {
                        for &a in &args {
                            self.expr(a, None);
                        }
                        self.err(n, "method not found")
                    }
                }
            }
            Kind::Field => {
                let base = self.expr(kids[0], None);
                let name = ast.token(kids[1]).0;
                self.field(n, kids[0], base, name)
            }
            Kind::Index => {
                let base = self.expr(kids[0], None);
                let idx = self.expr(kids[1], None);
                self.index(n, kids[0], base, idx)
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
                self.record_for(n, it);
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
                let o = self.try_output(n, t);
                self.record_try(n, t);
                o
            }
            Kind::Closure => self.closure(n, &kids, expect),
            Kind::MacroCall => self.builtin_call(n, &kids),
            Kind::Await => {
                let t = self.expr(kids[0], None);
                let s = self.norm(t);
                match self.await_output(s) {
                    Some(o) => o,
                    None => self.err(n, "`.await` on something that is not a future"),
                }
            }
            Kind::Builtin => self.builtin_expr(n, &kids, expect),
            Kind::Yield | Kind::Become | Kind::Yeet | Kind::UseExpr => {
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
                // `&[u8; N]`, N its length (an expression if it is bad).
                let c = match crate::lower::parse_str(t) {
                    Some(bytes) => tcx.konst(Const::Value(bytes.len() as u128)),
                    None => tcx.konst(Const::Expr(self.k, self.file, n)),
                };
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
        let s = self.norm(t);
        match op {
            "*" => match tcx.kind(s) {
                TyKind::Ptr(_, x) => x,
                _ => match self.deref_once(s) {
                    Some((x, how)) => {
                        if how == Adjust::OverloadedDeref
                            && let Some(tr) = self.lang("deref")
                        {
                            self.record_trait_method(n, tr, "deref", s, &[]);
                        }
                        x
                    }
                    None => self.err(n, "dereference of something that does not implement `Deref`"),
                },
            },
            "-" if self.infer.is_numeric(s) => t,
            "!" if self.infer.is_integral(s) || tcx.kind(s) == TyKind::Bool => t,
            _ => {
                let (lang, method) = if op == "-" { ("neg", "neg") } else { ("not", "not") };
                match self.op_trait(n, lang, method, s, None) {
                    Some(o) => o,
                    None => self.err(n, "unary operator on a type that does not implement it"),
                }
            }
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
        // An operand waiting on a projection (a closure's parameter) may be
        // known by now; it must be before deciding the operator is built in.
        let (sa, sb) = (self.norm(a), self.norm(b));
        let prim = |s: Ty| self.infer.is_numeric(s) || matches!(tcx.kind(s), TyKind::Bool | TyKind::Char);
        let both_prim = prim(sa) && prim(sb) || prim(sa) && self.infer.var_kind(sb).is_some() || self.infer.var_kind(sa).is_some() && prim(sb);
        let assign = op.ends_with('=') && !matches!(op.as_str(), "==" | "!=" | "<=" | ">=");
        let compare = matches!(op.as_str(), "==" | "!=" | "<" | ">" | "<=" | ">=");
        if compare {
            if both_prim || self.infer.var_kind(sa).is_some() {
                self.coerce(kids[2], b, a);
                return bool_;
            }
            // `PartialEq<B> for A` or `PartialOrd<B> for A`; failing
            // that, the same type.
            let lang = if matches!(op.as_str(), "==" | "!=") { "eq" } else { "partial_ord" };
            let method = match op.as_str() {
                "==" => "eq",
                "!=" => "ne",
                "<" => "lt",
                ">" => "gt",
                "<=" => "le",
                _ => "ge",
            };
            let p = self.probe();
            match self.lang(lang).map(|tr| (tr, self.select(sa, tr, &[sb], 0))) {
                Some((tr, super::traits::Select::Yes(_))) => self.record_trait_method(n, tr, method, sa, &[sb]),
                _ => {
                    self.rollback(p);
                    self.coerce(kids[2], b, a);
                }
            }
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
        // Types waiting on projections (a closure's parameter) may be
        // known by now.
        if !self.pending.is_empty() && (self.tcx.has_infer(self.infer.resolve(a)) || self.tcx.has_infer(self.infer.resolve(b))) {
            self.process_pending();
        }
        let base = op.trim_end_matches('=');
        let lang = match base {
            "+" => "add",
            "-" => "sub",
            "*" => "mul",
            "/" => "div",
            "%" => "rem",
            "&" => "bitand",
            "|" => "bitor",
            "^" => "bitxor",
            "<<" => "shl",
            ">>" => "shr",
            _ => return self.err(n, "unknown operator"),
        };
        let (sa, sb) = (self.norm(a), self.norm(b));
        if assign {
            let name = format!("{lang}_assign");
            let ok = match self.lang(&name) {
                Some(tr) if self.select(sa, tr, &[sb], 0).holds() => {
                    self.record_trait_method(n, tr, &name, sa, &[sb]);
                    true
                }
                _ => false,
            };
            return if ok { self.unit() } else { self.err(n, "operator on types that do not implement it") };
        }
        match self.op_trait(n, lang, lang, sa, Some(sb)) {
            Some(o) => o,
            None => self.err(n, "operator on types that do not implement it"),
        }
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
        // Closures last, so that the other arguments settle what they take.
        let mut order: Vec<usize> = (0..args.len()).filter(|&i| self.ast.kind(args[i]) != Kind::Closure).collect();
        order.extend((0..args.len()).filter(|&i| self.ast.kind(args[i]) == Kind::Closure));
        for i in order {
            let a = args[i];
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
        let s = self.norm(callee);
        let sig = match tcx.kind(s) {
            TyKind::FnDef(d, l) => self.callable_sig(d, l),
            TyKind::FnPtr(sig) | TyKind::Closure(_, sig, _) => {
                let sig = tcx.sig_of(sig);
                Some((tcx.tys(sig.inputs), sig.output))
            }
            TyKind::Ref(_, x) => match tcx.kind(self.infer.shallow(x)) {
                TyKind::FnPtr(sig) | TyKind::Closure(_, sig, _) => {
                    let sig = tcx.sig_of(sig);
                    Some((tcx.tys(sig.inputs), sig.output))
                }
                _ => None,
            },
            _ => None,
        };
        let sig = match sig {
            Some(x) => Some(x),
            None if self.infer.var_kind(s).is_none() => self.call_fn_traits(n, s, args.len()),
            None => None,
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
                self.err(n, "call of something that is not a function")
            }
        }
    }

    /// `base.name`, through references, `Box` and `Deref`; the
    /// dereferences are adjustments of `base_node`.
    fn field(&mut self, n: NodeId, base_node: NodeId, base: Ty, name: &str) -> Ty {
        let tcx = self.tcx;
        let steps = self.autoderef(base);
        for k in 0..steps.len() {
            let t = steps[k].0;
            match tcx.kind(t) {
                TyKind::Adt(d, l) => {
                    if let Some(adt) = tcx.adts.get(&d)
                        && adt.kind != crate::ty::AdtKind::Enum
                        && let Some(f) = adt.variants[0].fields.iter().find(|f| f.name == name)
                    {
                        let ft = subst(tcx, f.ty, &tcx.all_params(d), &tcx.tys(l));
                        self.adjust(base_node, Self::step_adjustments(&steps, k, None), steps[k].0);
                        return self.normalize(ft, 0);
                    }
                }
                TyKind::Tuple(l) => {
                    if let Some(x) = name.parse::<usize>().ok().and_then(|i| tcx.tys(l).get(i).copied()) {
                        self.adjust(base_node, Self::step_adjustments(&steps, k, None), steps[k].0);
                        return x;
                    }
                }
                TyKind::Ptr(_, x) => {
                    // Not autoderef, but `(*p).f` is often written `p.f`
                    // in macro output; follow it once.
                    let x = self.norm(x);
                    if let TyKind::Adt(d, l) = tcx.kind(x)
                        && let Some(adt) = tcx.adts.get(&d)
                        && adt.kind != crate::ty::AdtKind::Enum
                        && let Some(f) = adt.variants[0].fields.iter().find(|f| f.name == name)
                    {
                        let ft = subst(tcx, f.ty, &tcx.all_params(d), &tcx.tys(l));
                        let mut adj = Self::step_adjustments(&steps, k, None);
                        adj.push(Adjust::Deref);
                        self.adjust(base_node, adj, x);
                        return self.normalize(ft, 0);
                    }
                }
                TyKind::Infer(_) => return self.err(n, "field of a value whose type is not yet known"),
                _ => {}
            }
        }
        self.err(n, "field not found")
    }

    /// `base[idx]`: built in for arrays, slices and `str`, else `Index`,
    /// through autoderef.
    /// The dereferences (and an array's unsizing, for `Index` on a slice)
    /// are adjustments of `base_node`; an overloaded index records
    /// `Index::index` on `n`.
    fn index(&mut self, n: NodeId, base_node: NodeId, base: Ty, idx: Ty) -> Ty {
        let tcx = self.tcx;
        let steps = self.autoderef(base);
        let i = self.norm(idx);
        for k in 0..steps.len() {
            let t = steps[k].0;
            match tcx.kind(t) {
                TyKind::Array(e, _) | TyKind::Slice(e) if self.infer.is_integral(i) => {
                    let usize_ = self.mk(TyKind::Uint(UintTy::Usize));
                    self.infer.unify(i, usize_);
                    // An array's own elements, not its slice's.
                    let k = if steps[k].1 == Some(Adjust::Unsize) { k - 1 } else { k };
                    self.adjust(base_node, Self::step_adjustments(&steps, k, None), steps[k].0);
                    return e;
                }
                TyKind::Infer(_) => return self.err(n, "indexing a value whose type is not yet known"),
                _ => {}
            }
            if let Some(tr) = self.lang("index")
                && self.select(t, tr, &[i], 0).holds()
                && let Some(out) = self.trait_item(tr, "Output", Ns::Type)
            {
                self.adjust(base_node, Self::step_adjustments(&steps, k, None), steps[k].0);
                self.record_trait_method(n, tr, "index", t, &[i]);
                return self.project_or_var(out, &[t, i]);
            }
        }
        self.err(n, "indexing something that does not implement `Index`")
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

    /// What `for` gets from iterating a value: `IntoIterator::Item`.
    fn iter_item(&mut self, n: NodeId, t: Ty) -> Ty {
        let tcx = self.tcx;
        let s = self.norm(t);
        if let Some(item) = self.iterated_item(s) {
            return item;
        }
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
        let s = self.norm(t);
        if let Some(o) = self.try_output_trait(s) {
            return o;
        }
        match tcx.kind(s) {
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
        let mut expected_ret = None;
        let expected_inputs: Option<Vec<Ty>> = match expect.map(|e| (e, tcx.kind(self.infer.shallow(e)))) {
            Some((_, TyKind::FnPtr(s) | TyKind::Closure(_, s, _))) => {
                let s = tcx.sig_of(s);
                expected_ret = Some(s.output);
                Some(tcx.tys(s.inputs))
            }
            // A variable or parameter with `Fn` bounds on it.
            Some((e, _)) => self.closure_expectation(e).map(|(i, r)| {
                expected_ret = r;
                i
            }),
            None => None,
        };
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
            None => expected_ret.unwrap_or_else(|| self.infer.fresh()),
        };
        let body = *kids.last().unwrap();
        let saved_ret = std::mem::replace(&mut self.ret, ret);
        let saved_loops = std::mem::take(&mut self.loops);
        let bt = self.expr(body, Some(ret));
        self.coerce(body, bt, ret);
        self.ret = saved_ret;
        self.loops = saved_loops;
        let id = tcx.closure_id((self.k, self.file, n));
        self.closures.push(id);
        let sig = tcx.sig(crate::ty::FnSig { inputs: tcx.list(inputs), output: ret, unsafety: false, variadic: false, abi: crate::ty::Abi::Rust });
        self.mk(TyKind::Closure(id, sig, tcx.identity_args(self.owner)))
    }

    /// `builtin # name(..)`: `offset_of` is a `usize` (its container's
    /// type recorded at the type node, for lowering); `type_ascribe(e, T)`
    /// is `e` as a `T`; and the unsafe binder ones are their operand, as
    /// an unsafe binder's type is the type it binds.
    fn builtin_expr(&mut self, n: NodeId, kids: &[NodeId], expect: Option<Ty>) -> Ty {
        let ast = self.ast;
        match ast.token(kids[0]).0 {
            "offset_of" => {
                let t = self.written_ty(kids[1]);
                self.types.insert(kids[1], t);
                let mut path = crate::ty::lower::field_path(&ast.text(kids[2]));
                let mut cur = self.norm(t);
                while !path.is_empty() {
                    let Some((_, _, ft, used)) = self.tcx.field_step(cur, &path) else {
                        return self.err(n, "`offset_of` of a field not found");
                    };
                    path.drain(..used);
                    cur = self.norm(ft);
                }
                self.mk(TyKind::Uint(UintTy::Usize))
            }
            "type_ascribe" if kids.len() == 3 => {
                let t = self.written_ty(kids[2]);
                let e = self.expr(kids[1], Some(t));
                self.infer.unify(e, t);
                t
            }
            "wrap_binder" | "unwrap_binder" if kids.len() >= 2 => {
                let e = self.expr(kids[1], expect);
                if let Some(&w) = kids.get(2) {
                    let t = self.written_ty(w);
                    self.infer.unify(e, t);
                }
                e
            }
            _ => self.err(n, "unsupported expression"),
        }
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
            "asm" => {
                // Its templates and operands' expressions, which expansion
                // added after its tokens.
                for &c in &kids[2.min(kids.len())..] {
                    self.expr(c, None);
                }
                let noreturn = kids.get(1).is_some_and(|&tt| {
                    self.ast.tokens_of(tt).windows(3).any(|w| w[0].0 == "options" && w[1].0 == "(") && self.ast.tokens_of(tt).iter().any(|t| t.0 == "noreturn")
                });
                if noreturn { self.mk(TyKind::Never) } else { self.unit() }
            }
            "assert" | "global_asm" | "naked_asm" | "trace_macros" | "log_syntax" => self.unit(),
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
        let keyword = |k: &str| kids.iter().any(|&c| ast.kind(c) == Kind::Keyword && ast.token(c).0 == k);
        // `try { e }`: some `R: Try<Output = typeof e>`, which the
        // context says.
        if keyword("try") {
            let out = self.stmts(&stmts, None);
            let r = match expect {
                Some(e) => e,
                None => self.infer.fresh(),
            };
            if let Some(tr) = self.lang("Try")
                && let Some(o) = self.trait_item(tr, "Output", Ns::Type)
            {
                let rs = self.norm(r);
                if self.infer.var_kind(rs).is_some() {
                    let tcx = self.tcx;
                    self.obligations.push(Pred::Projection(rs, o, tcx.list(vec![rs]), out));
                } else {
                    let ro = self.project_or_var(o, &[rs]);
                    self.infer.unify(ro, out);
                }
            }
            return r;
        }
        // `async` and `gen` blocks: a future or iterator of their own.
        if keyword("async") || keyword("gen") {
            self.stmts(&stmts, None);
            return self.infer.fresh();
        }
        self.stmts(&stmts, expect)
    }

    /// A block's statements: the tail's type, `!` if one diverges, or `()`.
    fn stmts(&mut self, stmts: &[NodeId], expect: Option<Ty>) -> Ty {
        let ast = self.ast;
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
        let t = if peels || kind == Kind::RefPat { self.norm(t) } else { t };
        if peels && let TyKind::Ref(m, inner) = tcx.kind(self.infer.shallow(t)) {
            let bm = if by_ref && bm == Mutbl::Not { Mutbl::Not } else { m };
            *self.pat_derefs.entry(p).or_default() += 1;
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
                let mode = if has_ref {
                    BindingMode::Ref(if is_mut { Mutbl::Mut } else { Mutbl::Not })
                } else if by_ref {
                    BindingMode::Ref(bm)
                } else {
                    BindingMode::Value
                };
                let bt = match mode {
                    BindingMode::Ref(m) => self.mk(TyKind::Ref(m, t)),
                    BindingMode::Value => t,
                };
                self.bindings.insert(p, mode);
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
            // `deref!(p)`: `p` matches what the value dereferences to.
            Kind::BuiltinPat if self.ast.token(kids[0]).0 == "deref" => {
                let s = self.norm(t);
                match self.deref_once(s) {
                    Some((x, how)) => {
                        if how == Adjust::OverloadedDeref
                            && let Some(tr) = self.lang("deref")
                        {
                            self.record_trait_method(p, tr, "deref", s, &[]);
                        }
                        self.pat(kids[1], x, bm, by_ref);
                    }
                    None => {
                        self.err(p, "`deref!` of something that does not implement `Deref`");
                    }
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

/// Whether a node in generic arguments is an argument: a type, or a
/// constant written as a literal, a negated one or a block.
fn is_generic_arg(k: Kind) -> bool {
    is_type(k) || matches!(k, Kind::Literal | Kind::Unary | Kind::Block)
}
