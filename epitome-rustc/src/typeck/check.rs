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
    // A function a `reuse` defines has no body of its own.
    if matches!(ast.kind(item), Kind::Delegation | Kind::UseTree) {
        return None;
    }
    match kind {
        DefKind::Fn | DefKind::AssocFn => {
            let body = ast.children(item).iter().copied().find(|&c| ast.kind(c) == Kind::Block)?;
            let sig = tcx.sig_of(tcx.fn_sig[owner.0 as usize]?);
            let inputs = tcx.tys(sig.inputs);
            let inputs: Vec<Ty> = inputs.into_iter().map(|t| cx.fill_holes(t)).collect();
            let inputs: Vec<Ty> = inputs.into_iter().map(|t| cx.reveal_defined(t)).collect();
            // An `async fn`'s body is checked against the declared return
            // type, the output of the future a call gives.
            let out = match tcx.kind(sig.output) {
                TyKind::AsyncBlock(c, o, _) if tcx.closure_at(c) == (k, file, item) => o,
                // A `gen fn`'s body yields the declared type and returns
                // `()`.
                TyKind::GenBlock(c, y, _, _) if tcx.closure_at(c) == (k, file, item) => {
                    let unit = tcx.unit();
                    cx.coroutine = Some((unit, y));
                    unit
                }
                _ => sig.output,
            };
            let out = cx.fill_holes(out);
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
            | MoveExpr
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
    /// Selecting for a projection's trait: a bound that could be any of
    /// several is not taken (see `select`).
    pub(super) projecting: bool,
    /// The next `select` leaves out the bounds the body assumes.
    pub(super) skip_assumed: bool,
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
    /// In a coroutine: its resume and yield types, for `yield`.
    coroutine: Option<(Ty, Ty)>,
    /// The next closure checked is marked `#[coroutine]`.
    coroutine_attr: bool,
    /// The expression a call being checked calls (its path's `where`
    /// clauses are the call's to oblige).
    callee: Option<NodeId>,
    /// The types of the `try` blocks being checked, innermost last: where
    /// a `?` in them goes.
    pub(super) try_blocks: Vec<Ty>,
    /// The variable standing for each opaque type this body defines (its
    /// hidden type), one for all its uses, and the opaque type.
    defined: FastMap<crate::ty::OpaqueId, (Ty, Ty)>,
    /// Variables a `!` was coerced to: unknown at the end, they are `!`
    /// (2024) or `()` (before), as for rustc's diverging variables.
    diverging: Vec<Ty>,
    loops: Vec<Loop<'s>>,
    errors: FastMap<String, usize>,
    /// Nodes already counted as errors.
    reported: FastSet<NodeId>,
    /// What the body may assume (`traits.rs`).
    pub(super) env_preds: Vec<Pred>,
    /// While ranking specialising impls: a generic parameter has only the
    /// auto traits and `Freeze` its bounds give it, so that `impl<T:
    /// FreezeMarker>` is more specific than `impl<T>` (`traits.rs`).
    pub(super) params_strict: bool,
    /// Predicates on variables, from generic items used in the body.
    pub(super) obligations: Vec<Pred>,
    /// Projections waiting for their types to be known: `<args>::assoc`
    /// is the variable.
    pub(super) pending: Vec<(DefId, crate::ty::List, Ty)>,
    /// Trait predicates on a known type whose trait arguments are still
    /// variables (`NonNull<u8>: From<Unique<?T>>`): selecting the impl
    /// once only one matches says what the variables are.
    pub(super) pending_traits: Vec<(Ty, DefId, crate::ty::List)>,
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
            projecting: false,
            skip_assumed: false,
            ret: tcx.unit(),
            coroutine: None,
            coroutine_attr: false,
            callee: None,
            diverging: Vec::new(),
            try_blocks: Vec::new(),
            defined: FastMap::default(),
            loops: Vec::new(),
            errors: FastMap::default(),
            reported: FastSet::default(),
            env_preds: Vec::new(),
            params_strict: false,
            obligations: Vec::new(),
            pending: Vec::new(),
            pending_traits: Vec::new(),
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
                    // A method's receiver is a place only if it is
                    // borrowed (`borrowed`, from its adjustments): one taken
                    // by value (`hooks.run()`) is moved.
                    Kind::Field | Kind::Index | Kind::Match => first,
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
        // An opaque type this body could define but did not settle: a use
        // of it, not a definition.
        let defined: Vec<(Ty, Ty)> = self.defined.values().copied().collect();
        for (v, t) in defined {
            if self.infer.var_kind(self.infer.shallow(v)) == Some(VarKind::General) {
                self.infer.unify(v, t);
            }
        }
        let edition = self.env.crates[self.k as usize].edition;
        for v in std::mem::take(&mut self.diverging) {
            if self.infer.var_kind(self.infer.shallow(v)) == Some(VarKind::General) {
                let t = if edition >= crate::lexer::Edition::E2024 { self.mk(TyKind::Never) } else { self.unit() };
                self.infer.unify(v, t);
            }
        }
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
                // With `EPITOME_TYPECK_WHY` set to a reason with
                // "unknown" in it, the first node in each body whose type
                // was left unknown.
                if unknown_nodes == 0 && why_filter().is_some_and(|w| "a type left unknown".contains(w)) {
                    let mut text = self.ast.text(n).replace('\n', " ");
                    text.truncate((0..=80.min(text.len())).rev().find(|&i| text.is_char_boundary(i)).unwrap_or(0));
                    eprintln!("a type left unknown: {text} in {}", self.r.def_path(self.owner));
                }
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
            owner: self.owner,
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
            if let Some(w) = why_filter()
                && why.contains(w)
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
        let t = self.fill_holes(t);
        self.reveal_defined(t)
    }

    /// `t` with each type-alias `impl Trait` this body defines replaced by
    /// its hidden type's variable.
    fn reveal_defined(&mut self, t: Ty) -> Ty {
        let tcx = self.tcx;
        let map = |cx: &mut Self, l: crate::ty::List| -> crate::ty::List {
            let v: Vec<Ty> = tcx.tys(l).into_iter().map(|x| cx.reveal_defined(x)).collect();
            tcx.list(v)
        };
        match tcx.kind(t) {
            TyKind::Opaque(o, _) if self.defines(o) => self.opaque_vars(t, true),
            TyKind::Ref(m, x) => {
                let x = self.reveal_defined(x);
                self.mk(TyKind::Ref(m, x))
            }
            TyKind::Ptr(m, x) => {
                let x = self.reveal_defined(x);
                self.mk(TyKind::Ptr(m, x))
            }
            TyKind::Slice(x) => {
                let x = self.reveal_defined(x);
                self.mk(TyKind::Slice(x))
            }
            TyKind::Array(x, c) => {
                let x = self.reveal_defined(x);
                self.mk(TyKind::Array(x, c))
            }
            TyKind::Adt(d, l) if tcx.tys(l).iter().any(|&x| has_opaque(tcx, x)) => {
                let l = map(self, l);
                self.mk(TyKind::Adt(d, l))
            }
            TyKind::Tuple(l) if tcx.tys(l).iter().any(|&x| has_opaque(tcx, x)) => {
                let l = map(self, l);
                self.mk(TyKind::Tuple(l))
            }
            _ => t,
        }
    }

    /// `t` with the `impl Trait`s whose hidden types this body is finding
    /// replaced by their variables (inside a type's arguments too).
    fn reveal_own(&self, t: Ty) -> Ty {
        let tcx = self.tcx;
        match tcx.kind(t) {
            TyKind::Opaque(o, _) => self.defined.get(&o).map_or(t, |&(v, _)| v),
            TyKind::Ref(m, x) => self.mk(TyKind::Ref(m, self.reveal_own(x))),
            TyKind::Adt(d, l) if tcx.tys(l).iter().any(|&x| has_opaque(tcx, x)) => {
                let v: Vec<Ty> = tcx.tys(l).into_iter().map(|x| self.reveal_own(x)).collect();
                self.mk(TyKind::Adt(d, tcx.list(v)))
            }
            TyKind::Tuple(l) if tcx.tys(l).iter().any(|&x| has_opaque(tcx, x)) => {
                let v: Vec<Ty> = tcx.tys(l).into_iter().map(|x| self.reveal_own(x)).collect();
                self.mk(TyKind::Tuple(tcx.list(v)))
            }
            _ => t,
        }
    }

    /// Whether this body defines type-alias `impl Trait` `o`: it lists the
    /// alias in `#[define_opaque(..)]`, or the alias is declared inside it.
    fn defines(&self, o: crate::ty::OpaqueId) -> bool {
        let alias = self.tcx.opaque(o).owner;
        let a = &self.r.defs[alias.0 as usize];
        if a.kind != DefKind::TypeAlias {
            return false;
        }
        let me = &self.r.defs[self.owner.0 as usize];
        let ast = self.ast;
        let item = ast.node(me.node);
        if a.krate == me.krate && a.file == me.file {
            let n = ast.node(a.node);
            if n.first >= item.first && n.end <= item.end && a.node != me.node {
                return true;
            }
        }
        // `#[define_opaque(Tait, other::Tait2)]`, by the aliases' names.
        ast.children(me.node).iter().any(|&c| {
            ast.kind(c) == Kind::Attribute && {
                let text = ast.text(c);
                text.trim_start_matches("#[").starts_with("define_opaque")
                    && text.split(|ch: char| !(ch.is_alphanumeric() || ch == '_')).any(|w| w == a.name)
            }
        })
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
            TyKind::Projection(d, l) => {
                let v = tcx.tys(l).into_iter().map(|x| self.fill_holes(x)).collect();
                self.mk(TyKind::Projection(d, tcx.list(v)))
            }
            // `<X<_, _> as Tr>` with `type X<A, B> = impl Tr`.
            TyKind::Opaque(o, l) => {
                let v = tcx.tys(l).into_iter().map(|x| self.fill_holes(x)).collect();
                self.mk(TyKind::Opaque(o, tcx.list(v)))
            }
            // `fn(_) -> _`.
            TyKind::FnPtr(sig) => {
                let mut sig = tcx.sig_of(sig);
                let v = tcx.tys(sig.inputs).into_iter().map(|x| self.fill_holes(x)).collect();
                sig.inputs = tcx.list(v);
                sig.output = self.fill_holes(sig.output);
                self.mk(TyKind::FnPtr(tcx.sig(sig)))
            }
            // `dyn Iterator<Item = _>`.
            TyKind::Dyn(d) => {
                let mut data = tcx.dyn_of(d);
                for tr in &mut data.traits {
                    let v = tcx.tys(tr.args).into_iter().map(|x| self.fill_holes(x)).collect();
                    tr.args = tcx.list(v);
                    for b in &mut tr.bindings {
                        b.1 = self.fill_holes(b.1);
                    }
                }
                self.mk(TyKind::Dyn(tcx.dyn_(data)))
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
        // So may a box's (`Box::new(v.into_iter())` as a `Box<dyn Iterator>`).
        if !self.pending.is_empty()
            && let (TyKind::Adt(a, x), TyKind::Adt(b, y)) =
                (self.tcx.kind(self.infer.shallow(got)), self.tcx.kind(self.infer.shallow(want)))
            && a == b
            && Some(a) == self.lang("owned_box")
            && self.infer.var_kind(self.infer.shallow(self.tcx.tys(x)[0])).is_some()
            && self.may_be_unsized(self.tcx.tys(y)[0])
        {
            self.process_pending();
        }
        // An unknown made a pointer (`ptr::read(x)`, `x` waiting on a
        // projection) may be a reference once it is known.
        if !self.pending.is_empty()
            && self.infer.var_kind(self.infer.shallow(got)) == Some(VarKind::General)
            && matches!(self.tcx.kind(self.infer.shallow(want)), TyKind::Ptr(..))
        {
            self.process_pending();
        }
        let g = self.infer.shallow(got);
        let w = self.infer.shallow(want);
        let tcx = self.tcx;
        match (tcx.kind(g), tcx.kind(w)) {
            (TyKind::Never, TyKind::Never) => return true,
            (TyKind::Never, _) => {
                self.diverges_into(w);
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
            // `Pin<&mut T>` as `Pin<&T>` (`pin_ergonomics`).
            (TyKind::Adt(a, x), TyKind::Adt(b, y))
                if a == b
                    && Some(a) == self.lang("pin")
                    && let (TyKind::Ref(Mutbl::Mut, p), TyKind::Ref(Mutbl::Not, q)) =
                        (tcx.kind(self.infer.shallow(tcx.tys(x)[0])), tcx.kind(self.infer.shallow(tcx.tys(y)[0]))) =>
            {
                let s = self.probe();
                if self.infer.unify(p, q) {
                    return true;
                }
                self.rollback(s);
            }
            (TyKind::Adt(a, x), TyKind::Adt(b, y)) if a == b && Some(a) == self.lang("owned_box") => {
                // The allocators are the same; the pointees the same, or
                // the one unsizing to the other.
                let (xs, ys) = (tcx.tys(x), tcx.tys(y));
                let p = self.probe();
                if xs[1..].iter().zip(&ys[1..]).all(|(&a, &b)| self.infer.unify(a, b)) {
                    let (x, y) = (xs[0], ys[0]);
                    let q = self.probe();
                    if self.infer.unify(x, y) {
                        return true;
                    }
                    self.rollback(q);
                    if self.unsizes_to(x, y) {
                        self.adjust(n, vec![Adjust::Unsize], w);
                        return true;
                    }
                }
                self.rollback(p);
            }
            (TyKind::FnDef(d, l), TyKind::FnPtr(s)) => {
                if let Some((inputs, output)) = self.callable_sig(d, l) {
                    let s = tcx.sig_of(s);
                    let want_in = tcx.tys(s.inputs);
                    let ok = inputs.len() == want_in.len()
                        && inputs.iter().zip(&want_in).all(|(&a, &b)| self.unify_alias(a, b))
                        && self.unify_alias(output, s.output);
                    if ok {
                        self.adjust(n, vec![Adjust::ReifyFnPointer], w);
                    }
                    return ok;
                }
            }
            // `fn(T)` to `unsafe fn(T)`: the same pointer.
            (TyKind::FnPtr(a), TyKind::FnPtr(b)) if !tcx.sig_of(a).unsafety && tcx.sig_of(b).unsafety => {
                let s = crate::ty::FnSig { unsafety: true, ..tcx.sig_of(a) };
                let unsafe_ptr = self.mk(TyKind::FnPtr(tcx.sig(s)));
                if self.infer.unify(unsafe_ptr, w) {
                    return true;
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
        // An `impl Trait` this body defines, met as another type (`let x:
        // Vec<i32> = foo(false)` in `foo`, or the hidden type itself, as in
        // `match b { true => baz(false), false => Some(bar()) }`): its
        // hidden type, rather than the opaque type standing for itself.
        let revealed = self.reveal_own(g);
        let wanted = self.reveal_own(w);
        if revealed != g || wanted != w {
            let p = self.probe();
            if self.infer.unify(revealed, wanted) {
                return true;
            }
            self.rollback(p);
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

    /// A branch (an `if`'s, a match arm) whose type is a reference that is
    /// not the expected reference type: deref-coerced to it, as each
    /// branch is coerced to what is wanted (`Owned(s) => &s` as a
    /// `&str`).
    fn branch_to_expected(&mut self, n: NodeId, t: Ty, expect: Option<Ty>) -> Ty {
        let tcx = self.tcx;
        let Some(e) = expect.map(|e| self.infer.shallow(e)) else { return t };
        let (TyKind::Ref(..), TyKind::Ref(_, x)) = (tcx.kind(self.infer.shallow(t)), tcx.kind(e)) else { return t };
        if tcx.has_infer(self.infer.resolve(x)) {
            return t;
        }
        let p = self.probe();
        if self.infer.unify(t, e) {
            return e;
        }
        self.rollback(p);
        if let TyKind::Ref(m, y) = tcx.kind(self.infer.shallow(t))
            && let TyKind::Ref(m2, _) = tcx.kind(e)
            && (m == m2 || m == Mutbl::Mut)
        {
            // `&mut T` as `&T`.
            if m != m2 && self.infer.unify(y, x) {
                return e;
            }
            self.rollback(p);
            if let Some(adj) = self.deref_coerces(y, x, m2) {
                self.adjust(n, adj, e);
                return e;
            }
        }
        self.rollback(p);
        t
    }

    /// The common type of two branches (`if`/`else`, match arms).
    fn join(&mut self, n: NodeId, a: Ty, b: Ty) -> Ty {
        let (sa, sb) = (self.infer.shallow(a), self.infer.shallow(b));
        // A `!` arm: the other's type, which, if nothing else settles it,
        // falls back as a diverging variable does.
        if self.tcx.kind(sa) == TyKind::Never {
            self.diverges_into(b);
            return b;
        }
        if self.tcx.kind(sb) == TyKind::Never {
            self.diverges_into(a);
            return a;
        }
        // `*mut T` and `*const T`: both `*const T`.
        if let (TyKind::Ptr(ma, x), TyKind::Ptr(mb, y)) = (self.tcx.kind(sa), self.tcx.kind(sb))
            && ma != mb
            && self.infer.unify(x, y)
        {
            return self.mk(TyKind::Ptr(Mutbl::Not, x));
        }
        // Two different functions or non-capturing closures (`if c { f }
        // else { g }`): both as a function pointer.
        let fn_like = |k: TyKind| matches!(k, TyKind::FnDef(..) | TyKind::Closure(..) | TyKind::FnPtr(_));
        if fn_like(self.tcx.kind(sa)) && fn_like(self.tcx.kind(sb)) && sa != sb {
            let p = self.probe();
            if !self.infer.unify(sa, sb) {
                self.rollback(p);
                if let Some(ptr) = self.fn_pointer_of(sa)
                    && self.coerce(n, a, ptr)
                    && self.coerce(n, b, ptr)
                {
                    return ptr;
                }
            }
        }
        if !self.coerce(n, b, a) {
            return self.tcx.error();
        }
        a
    }

    /// `a` and `b` made the same, where `a` may be a variable standing for
    /// a projection not yet worked out (`<?T as Foo>::Assoc`) and `b` that
    /// projection with other arguments (`<T as Foo>::Assoc`): the
    /// arguments are made the same too.
    fn unify_alias(&mut self, a: Ty, b: Ty) -> bool {
        let tcx = self.tcx;
        let va = self.infer.shallow(a);
        if self.infer.var_kind(va).is_some()
            && let TyKind::Projection(item, l) = tcx.kind(self.infer.shallow(b))
            && let Some(&(_, pl, _)) = self.pending.iter().find(|p| p.0 == item && self.infer.shallow(p.2) == va)
        {
            let (xs, ys) = (tcx.tys(pl), tcx.tys(l));
            let p = self.probe();
            if xs.len() == ys.len() && xs.iter().zip(&ys).all(|(&x, &y)| self.infer.unify(x, y)) {
                return self.infer.unify(a, b);
            }
            self.rollback(p);
        }
        self.infer.unify(a, b)
    }

    /// The function pointer type a function item, closure or pointer
    /// coerces to.
    fn fn_pointer_of(&mut self, t: Ty) -> Option<Ty> {
        let tcx = self.tcx;
        match tcx.kind(t) {
            TyKind::FnPtr(_) => Some(t),
            TyKind::Closure(_, s, _) => Some(self.mk(TyKind::FnPtr(s))),
            TyKind::FnDef(d, l) => {
                let (inputs, output) = self.callable_sig(d, l)?;
                // A tuple struct's or variant's constructor has no
                // signature of its own: a safe Rust one.
                let (unsafety, abi) = match tcx.fn_sig[d.0 as usize] {
                    Some(s) => (tcx.sig_of(s).unsafety, tcx.sig_of(s).abi),
                    None => (false, crate::ty::Abi::Rust),
                };
                let sig = crate::ty::FnSig { inputs: tcx.list(inputs), output, unsafety, variadic: false, abi };
                Some(self.mk(TyKind::FnPtr(tcx.sig(sig))))
            }
            _ => None,
        }
    }

    /// Marks `t`, if a variable not yet known, as one a `!` was coerced to
    /// (see `diverging`).
    fn diverges_into(&mut self, t: Ty) {
        let t = self.infer.shallow(t);
        if self.infer.var_kind(t) == Some(VarKind::General) {
            self.diverging.push(t);
        }
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
                // A trait's method by path (`From::from(x)`): its trait's
                // arguments may be settled by the one impl that fits.
                if let Some(tr) = self.trait_of(d) {
                    let n = tcx.all_params(tr).len().min(args.len());
                    if n > 1 && args[1..n].iter().any(|&a| tcx.has_infer(self.infer.resolve(a))) {
                        self.pending_traits.push((args[0], tr, tcx.list(args[1..n].to_vec())));
                    }
                }
                // `impl Trait` for a parameter: a variable, with the bounds
                // as obligations. For the result, the opaque type itself.
                let inst = |cx: &mut Self, t: Ty, input: bool| -> Ty {
                    // (Only the signature's own: an `impl Trait` that came
                    // in through `Self` is a type like any other.)
                    let input = input && has_opaque(tcx, t);
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

    /// `t` with each `impl Trait` in it (at the top, behind a reference,
    /// or among a type's arguments) replaced by a variable bound by its
    /// bounds. In the body that defines them, the variables are what they
    /// stand for (their hidden types), which are recorded.
    fn opaque_vars(&mut self, t: Ty, defining: bool) -> Ty {
        let tcx = self.tcx;
        match tcx.kind(t) {
            TyKind::Opaque(o, l) => {
                if defining && let Some(&(v, _)) = self.defined.get(&o) {
                    return v;
                }
                let v = self.infer.fresh();
                let def = tcx.opaque(o);
                let (params, args) = (tcx.all_params(def.owner), tcx.tys(l));
                let mut preds = Vec::new();
                for tr in def.bounds {
                    let mut tr = super::traits::subst_trait_ref(tcx, &tr, &params, &args);
                    // `impl IntoIterator<Item = (u8, impl Into<T>)>`: the
                    // inner one's hidden type too, where it is found.
                    if defining {
                        for b in &mut tr.bindings {
                            b.1 = self.opaque_vars(b.1, true);
                        }
                    }
                    self.elaborate(v, &tr, &mut preds);
                }
                for p in preds {
                    self.oblige(p);
                }
                if defining {
                    self.hidden.push((o, v));
                    self.defined.insert(o, (v, t));
                }
                v
            }
            TyKind::Ref(m, x) => {
                let x = self.opaque_vars(x, defining);
                self.mk(TyKind::Ref(m, x))
            }
            // Inside other types too (`Option<impl Write>`).
            TyKind::Adt(d, l) if tcx.tys(l).iter().any(|&x| has_opaque(tcx, x)) => {
                let args: Vec<Ty> = tcx.tys(l).into_iter().map(|x| self.opaque_vars(x, defining)).collect();
                self.mk(TyKind::Adt(d, tcx.list(args)))
            }
            TyKind::Tuple(l) if tcx.tys(l).iter().any(|&x| has_opaque(tcx, x)) => {
                let args: Vec<Ty> = tcx.tys(l).into_iter().map(|x| self.opaque_vars(x, defining)).collect();
                self.mk(TyKind::Tuple(tcx.list(args)))
            }
            // `dyn Future<Output = impl IntoIterator<Item = u32>>`.
            TyKind::Dyn(d) if has_opaque(tcx, t) => {
                let mut data = tcx.dyn_of(d);
                for tr in &mut data.traits {
                    for b in &mut tr.bindings {
                        b.1 = self.opaque_vars(b.1, defining);
                    }
                }
                self.mk(TyKind::Dyn(tcx.dyn_(data)))
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
                        // Not called (`f::<T, fn(_)>;`): its `where` clauses
                        // still say what unknowns in its arguments are.
                        if self.callee != Some(n) && args.iter().any(|&a| tcx.has_infer(self.infer.resolve(a))) {
                            self.oblige_item(d, &args);
                        }
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
                // `<T as Tr>::C::<u8>` (a generic const item).
                if let Some((m, item, args)) = self.consts.last().cloned()
                    && m == n
                {
                    self.own_turbofish(path, item, &args);
                }
                self.path_turbofish(path, t)
            }
            Res::Partial(base, 2) => {
                // `T::Assoc::item`: the associated type, then its item.
                let base = (**base).clone();
                let segs: Vec<NodeId> = self.ast.children(path).iter().copied().filter(|&s| self.ast.kind(s) == Kind::Segment).collect();
                let mid = self.ast.token(segs[segs.len() - 2]).0;
                let name = self.last_segment_name(path);
                let Some(bt) = self.base_ty(path, &base) else { return self.err(n, "path through an associated item") };
                // `<T as Trait>::Assoc::item`: `Assoc` of that trait.
                let at = if let Some(q) = self.ast.child(path, Kind::QSelf)
                    && let Some(&tr) = self.ast.children(q).get(1)
                    && let Some(&tp) = self.ast.children(tr).first()
                {
                    let Some((self_ty, trait_ref)) = self.qualified_trait_ref(q, tp) else {
                        return self.err(n, "trait in qualified path not found");
                    };
                    let Some(a) = self.trait_item(trait_ref.def, mid, Ns::Type) else {
                        return self.err(n, "associated type not found");
                    };
                    let mut args = vec![self_ty];
                    for t in tcx.tys(trait_ref.args) {
                        args.push(self.fill_holes(t));
                    }
                    self.project_or_var(a, &args)
                } else if let Some((item, mut args)) = self.inherent_item(bt, mid, Ns::Type)
                    && let Some(ty) = tcx.type_of(item)
                {
                    // `Self::Item::default()` with an inherent `type Item`.
                    let params = tcx.all_params(item);
                    let start = args.len();
                    while args.len() < params.len() {
                        args.push(self.infer.fresh());
                    }
                    // `Foo::Bar::<10>::new()`: its own arguments.
                    if let Some(g) = self.ast.child(segs[segs.len() - 2], Kind::GenericArgs) {
                        let ast = self.ast;
                        let written: Vec<NodeId> =
                            ast.children(g).iter().copied().filter(|&a| is_generic_arg(ast.kind(a))).collect();
                        for (i, a) in written.into_iter().enumerate() {
                            if let Some(&v) = args.get(start + i) {
                                let w = self.written_arg(a);
                                self.infer.unify(v, w);
                            }
                        }
                    }
                    subst(tcx, ty, &params, &args)
                } else {
                    let Some(at) = self.assoc_type_of(bt, mid) else { return self.err(n, "associated type not found") };
                    at
                };
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
        let TyKind::FnDef(item, l) = tcx.kind(t) else { return t };
        self.own_turbofish(path, item, &tcx.tys(l));
        t
    }

    /// The written arguments of `path`'s last segment made `item`'s own
    /// parameters (the last ones of `args`).
    fn own_turbofish(&mut self, path: NodeId, item: DefId, args: &[Ty]) {
        let ast = self.ast;
        let Some(&seg) = ast.children(path).iter().rev().find(|&&s| ast.kind(s) == Kind::Segment) else { return };
        let Some(g) = ast.child(seg, Kind::GenericArgs) else { return };
        let written: Vec<NodeId> = ast.children(g).iter().copied().filter(|&a| is_generic_arg(ast.kind(a))).collect();
        let own = self.tcx.generics[item.0 as usize].own.len();
        let start = args.len().saturating_sub(own);
        for (i, a) in written.into_iter().enumerate().take(own) {
            let w = self.written_arg(a);
            self.infer.unify(args[start + i], w);
        }
    }

    fn last_segment_name(&self, path: NodeId) -> &'s str {
        let ast = self.ast;
        ast.children(path).iter().rev().find(|&&s| ast.kind(s) == Kind::Segment).map_or("", |&s| ast.token(s).0)
    }

    /// `<T as Trait>::name` as a value.
    fn qualified_trait_value(&mut self, n: NodeId, q: NodeId, tr: NodeId, name: &str) -> Ty {
        let tcx = self.tcx;
        let Some(&tp) = self.ast.children(tr).first() else { return self.err(n, "bad qualified path") };
        let Some((self_ty, trait_ref)) = self.qualified_trait_ref(q, tp) else {
            return self.err(n, "trait in qualified path not found");
        };
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

    /// The self type and trait of a qualified path's `<T as Trait>` (`tp`
    /// the trait's path).
    fn qualified_trait_ref(&mut self, q: NodeId, tp: NodeId) -> Option<(Ty, crate::ty::TraitRef)> {
        let self_ty = self.written_ty(self.ast.children(q)[0]);
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
        Some((self_ty, l.trait_ref(tp)?))
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
                        // Types and constants (`F::<3>::t()`), not lifetimes.
                        let written: Vec<NodeId> = self
                            .ast
                            .children(g)
                            .iter()
                            .copied()
                            .filter(|&a| {
                                is_type(self.ast.kind(a))
                                    || matches!(self.ast.kind(a), Kind::Literal | Kind::Unary | Kind::Block)
                            })
                            .collect();
                        let count = written.len();
                        for (i, a) in written.into_iter().enumerate() {
                            if i < args.len() {
                                args[i] = self.written_arg(a);
                            }
                        }
                        // Those not written take their defaults:
                        // `RawVec::<T>` is `RawVec<T, Global>`.
                        let defaults = tcx.generics[d.0 as usize].defaults.clone();
                        for i in count..args.len() {
                            if let Some(Some(t)) = defaults.get(i) {
                                args[i] = subst(tcx, *t, &all, &args);
                            }
                        }
                    }
                    self.mk(TyKind::Adt(*d, tcx.list(args)))
                }
                DefKind::TypeAlias => {
                    let t = tcx.type_of(*d)?;
                    let all = tcx.all_params(*d);
                    let mut args = self.infer.instantiate(&all);
                    // `Foo::<i32>::None`: the alias's written arguments.
                    let ast = self.ast;
                    let segs: Vec<NodeId> =
                        ast.children(path).iter().copied().filter(|&s| ast.kind(s) == Kind::Segment).collect();
                    if segs.len() >= 2
                        && let Some(g) = ast.child(segs[segs.len() - 2], Kind::GenericArgs)
                    {
                        let arg = |a: NodeId| {
                            is_type(ast.kind(a)) || matches!(ast.kind(a), Kind::Literal | Kind::Unary | Kind::Block)
                        };
                        let written: Vec<NodeId> = ast.children(g).iter().copied().filter(|&a| arg(a)).collect();
                        for (i, a) in written.into_iter().enumerate() {
                            if i < args.len() {
                                args[i] = self.written_arg(a);
                            }
                        }
                    }
                    subst(tcx, t, &all, &args)
                }
                // `extern { type Local; }`: `Local::foo(x)`.
                DefKind::ForeignType => self.mk(TyKind::Foreign(*d)),
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
            // `Bar<impl Sized>` in the body finding that type: `Bar<?hidden>`
            // (`impl Bar<u32>` settles it).
            let revealed = self.reveal_own(t);
            let st_revealed = if has_opaque(tcx, st) { self.reveal_defined(st) } else { st };
            if revealed != t || st_revealed != st {
                if self.infer.unify(st_revealed, revealed) {
                    return Some((item, args));
                }
                self.infer.rollback(fresh);
            }
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
    /// Traits in scope are tried along the whole chain first, then (in
    /// case scope was missed) any trait.
    fn method_call(&mut self, n: NodeId, recv_node: NodeId, recv_ty: Ty, name: &str) -> Option<(DefId, Vec<Ty>, Ty)> {
        let s = self.probe();
        let steps = self.autoderef(recv_ty);
        let found = self.method_call_in(n, recv_node, &steps, name, true).or_else(|| self.method_call_in(n, recv_node, &steps, name, false));
        if found.is_some() || steps.len() < 2 && !steps.iter().any(|&(t, _)| self.reveal_own(t) != t) {
            return found;
        }
        // Not found. Working out the deref chain may itself have settled
        // unknowns (`Foo<?T, _>` through `impl Deref for Foo<u32, U>`
        // makes `?T` `u32`), which a method of the receiver itself
        // (`Foo<i32, i32>`) does not want: try that without the chain.
        // (And with the `impl Trait` this body is finding the hidden type
        // of, in the receiver, as its variable.)
        self.rollback(s);
        let first = self.norm(recv_ty);
        for t in [first, self.reveal_own(first)] {
            let steps = [(t, None)];
            if let Some(f) = self.method_call_in(n, recv_node, &steps, name, true).or_else(|| self.method_call_in(n, recv_node, &steps, name, false)) {
                return Some(f);
            }
        }
        None
    }

    fn method_call_in(&mut self, n: NodeId, recv_node: NodeId, steps: &[(Ty, Option<Adjust>)], name: &str, scoped: bool) -> Option<(DefId, Vec<Ty>, Ty)> {
        let tcx = self.tcx;
        for k in 0..steps.len() {
            let t = steps[k].0;
            if self.infer.var_kind(t) == Some(VarKind::General) {
                return None;
            }
            // `Pin<&mut T>` also as `Pin<&T>` (`pin_ergonomics`).
            let pin_shared = match tcx.kind(t) {
                TyKind::Adt(d, l) if Some(d) == self.lang("pin") => match tcx.kind(self.infer.shallow(tcx.tys(l)[0])) {
                    TyKind::Ref(Mutbl::Mut, x) => Some(tcx.pin(self.mk_ref(Mutbl::Not, x))),
                    _ => None,
                },
                _ => None,
            };
            for form in 0..4 {
                let (recv, borrow) = match form {
                    0 => (t, None),
                    1 => (self.mk_ref(Mutbl::Not, t), Some(Mutbl::Not)),
                    2 => (self.mk_ref(Mutbl::Mut, t), Some(Mutbl::Mut)),
                    _ => match pin_shared {
                        Some(p) => (p, None),
                        None => continue,
                    },
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
                            && (self.infer.unify(sp, recv) || {
                                let r = self.reveal_own(recv);
                                let sr = if has_opaque(tcx, sp) { self.reveal_defined(sp) } else { sp };
                                (r != recv || sr != sp) && self.infer.unify(sr, r)
                            })
                        {
                            // The impl's `where` clauses may settle its
                            // parameters (`impl<A, S> B<S> where S: R<E =
                            // A>`).
                            self.oblige_item(item, &args);
                            self.methods.push((n, item, args));
                            let adj = Self::step_adjustments(steps, k, borrow);
                            self.adjust(recv_node, adj, recv);
                            return Some((item, inputs[1..].to_vec(), output));
                        }
                    }
                    self.rollback(s);
                }
                if let Some((item, args, inputs, output)) = self.trait_method(recv, name, scoped) {
                    self.methods.push((n, item, args));
                    let adj = Self::step_adjustments(steps, k, borrow);
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
                // `#[coroutine] || ...`.
                self.coroutine_attr = ast.kind(n) == Kind::AttrExpr
                    && ast.kind(e) == Kind::Closure
                    && ast.children(n).iter().any(|&a| {
                        ast.kind(a) == Kind::Attribute && ast.text(a).trim_start_matches("#[").starts_with("coroutine")
                    });
                self.expr(e, expect)
            }
            Kind::Tuple => {
                let want: Vec<Option<Ty>> = match expect.map(|e| tcx.kind(self.infer.shallow(e))) {
                    Some(TyKind::Tuple(l)) if tcx.tys(l).len() == kids.len() => tcx.tys(l).into_iter().map(Some).collect(),
                    _ => vec![None; kids.len()],
                };
                let mut tys = Vec::new();
                for (&e, w) in kids.iter().zip(want) {
                    let t = self.expr(e, w);
                    // Each field is coerced to its expected type where a
                    // pointer's can change (an array behind it unsizing,
                    // `*mut` to `*const`: `(self, &[], &[])` as `(&[T],
                    // &[U], &[T])`), or it is `!`.
                    let pointer = |t: Ty| matches!(tcx.kind(self.infer.shallow(t)), TyKind::Ref(..) | TyKind::Ptr(..));
                    if let Some(w) = w
                        && (pointer(t) && pointer(w) || tcx.kind(self.infer.shallow(t)) == TyKind::Never)
                        && self.coerce(e, t, w)
                    {
                        tys.push(w);
                    } else {
                        tys.push(t);
                    }
                }
                self.mk(TyKind::Tuple(tcx.list(tys)))
            }
            Kind::Array => {
                let elem = match expect.map(|e| tcx.kind(self.infer.shallow(e))) {
                    Some(TyKind::Array(t, _) | TyKind::Slice(t)) => {
                        for &e in &kids {
                            let et = self.expr(e, Some(t));
                            self.coerce(e, et, t);
                        }
                        t
                    }
                    // The elements' common type, as match arms have
                    // (`[foo, |a| 2]`: function pointers).
                    _ => {
                        let mut acc: Option<Ty> = None;
                        for &e in &kids {
                            let et = self.expr(e, acc);
                            acc = Some(match acc {
                                None => et,
                                Some(a) => self.join(e, a, et),
                            });
                        }
                        acc.unwrap_or_else(|| self.infer.fresh())
                    }
                };
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
                // `[e; n]` copies `e` (unless it is a constant, or `n` is 0
                // or 1): `T: Copy` may settle unknowns in it.
                if !matches!(ast.text(kids[1]).trim(), "0" | "1")
                    && !matches!(ast.kind(kids[0]), Kind::PathExpr | Kind::ConstBlock)
                    && tcx.has_infer(self.infer.resolve(elem))
                    && let Some(copy) = self.lang("copy")
                {
                    self.pending_traits.push((elem, copy, tcx.list(vec![])));
                }
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
                if raw {
                    self.mk(TyKind::Ptr(mutbl, t))
                } else if kids.iter().any(|&c| ast.kind(c) == Kind::Keyword && ast.token(c).0 == "pin") {
                    // `&pin mut x`: `Pin<&mut T>`.
                    let r = self.mk(TyKind::Ref(mutbl, t));
                    tcx.pin(r)
                } else {
                    self.mk(TyKind::Ref(mutbl, t))
                }
            }
            Kind::Cast => {
                let target = self.written_ty(kids[1]);
                // A cast to `&dyn Fn(A)`: what a closure in it takes.
                // So does `(|x| x) as fn(usize) -> usize`.
                let mut op = kids[0];
                while ast.kind(op) == Kind::Paren {
                    op = *ast.children(op).last().unwrap();
                }
                let fn_ptr = ast.kind(op) == Kind::Closure && matches!(tcx.kind(target), TyKind::FnPtr(_));
                let src_expect = (fn_ptr || has_dyn(tcx, target)).then_some(target);
                let src = self.expr(kids[0], src_expect);
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
                        if let (TyKind::Ptr(_, x), TyKind::Ptr(_, y)) = (tcx.kind(s), tcx.kind(self.infer.shallow(target))) {
                            let p = self.probe();
                            if !self.infer.unify(y, x) {
                                self.rollback(p);
                            }
                        }
                    }
                }
                // `&[T; N] as &[U]`, `Box<[T; N]> as Box<[U]>`: `T` is `U`.
                let s = self.norm(src);
                let pointee = |k: TyKind| match k {
                    TyKind::Ref(_, x) | TyKind::Ptr(_, x) => Some(x),
                    TyKind::Adt(_, l) => tcx.tys(l).first().copied(),
                    _ => None,
                };
                let same_ctor = match (tcx.kind(s), tcx.kind(self.infer.shallow(target))) {
                    (TyKind::Ref(..), TyKind::Ref(..)) | (TyKind::Ptr(..), TyKind::Ptr(..)) => true,
                    (TyKind::Adt(a, _), TyKind::Adt(b, _)) => a == b,
                    _ => false,
                };
                if same_ctor
                    && let (Some(x), Some(y)) = (pointee(tcx.kind(s)), pointee(tcx.kind(self.infer.shallow(target))))
                    && let (TyKind::Array(e, _), TyKind::Slice(f)) =
                        (tcx.kind(self.infer.shallow(x)), tcx.kind(self.infer.shallow(y)))
                {
                    self.infer.unify(e, f);
                }
                // `&T as *const T`: a reference becomes a pointer to what
                // it refers to (unless it is to an array, `&[T; N] as
                // *const T`), which settles either side's unknowns.
                if let (TyKind::Ref(_, x), TyKind::Ptr(_, y)) = (tcx.kind(s), tcx.kind(self.infer.shallow(target))) {
                    let p = self.probe();
                    if !self.infer.unify(y, x) {
                        self.rollback(p);
                    }
                }
                // A pointer to an unknown cast to a pointer is tried as a
                // coercion first: `p.cast() as *const T` makes it `T`.
                if let (TyKind::Ptr(_, x), TyKind::Ptr(_, y)) =
                    (tcx.kind(self.norm(src)), tcx.kind(self.infer.shallow(target)))
                    && self.infer.var_kind(self.infer.shallow(x)) == Some(VarKind::General)
                {
                    let p = self.probe();
                    if !self.infer.unify(x, y) {
                        self.rollback(p);
                    }
                }
                // A cast to a struct or enum is a coercion: `Err(()) as
                // Result<(), _>` settles both sides.
                if let TyKind::Adt(..) = tcx.kind(self.infer.shallow(target)) {
                    let p = self.probe();
                    if !self.infer.unify(src, target) {
                        self.rollback(p);
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
                // A method of a variable a `!` was coerced to (`let x =
                // loop {}; x.m()`) is looked up on `!`, as rustc does.
                // An unknown receiver may be settled by what is pending (`T:
                // Tr` with one impl for `Option<_>`).
                if self.infer.var_kind(self.infer.shallow(recv)) == Some(VarKind::General) {
                    self.process_pending();
                }
                let r = self.infer.shallow(recv);
                if self.infer.var_kind(r) == Some(VarKind::General)
                    && self.diverging.iter().any(|&v| self.infer.shallow(v) == r)
                {
                    let never = self.mk(TyKind::Never);
                    self.infer.unify(r, never);
                }
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
                self.index(n, kids[0], kids[1], base, idx)
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
                        // Two closures (or functions) where a function
                        // pointer is wanted: each made one.
                        let callable = |t: Ty| matches!(tcx.kind(t), TyKind::Closure(..) | TyKind::FnDef(..));
                        let (st, se) = (self.infer.shallow(then), self.infer.shallow(et));
                        if callable(st) && callable(se) && st != se {
                            // The pointer wanted, or with no expectation two
                            // function items' signature as one.
                            let want = match expect.map(|w| self.infer.shallow(w)) {
                                Some(w) if matches!(tcx.kind(w), TyKind::FnPtr(_)) => Some(w),
                                _ => match tcx.kind(st) {
                                    TyKind::FnDef(d, l) if matches!(tcx.kind(se), TyKind::FnDef(..)) => self.callable_sig(d, l).and_then(|(inputs, output)| {
                                        let base = tcx.sig_of(tcx.fn_sig[d.0 as usize]?);
                                        Some(self.mk(TyKind::FnPtr(tcx.sig(crate::ty::FnSig { inputs: tcx.list(inputs), output, ..base }))))
                                    }),
                                    _ => None,
                                },
                            };
                            if let Some(want) = want {
                                self.coerce(kids[1], then, want);
                                self.coerce(e, et, want);
                                return want;
                            }
                        }
                        let then = self.branch_to_expected(kids[1], then, expect);
                        let et = self.branch_to_expected(e, et, expect);
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
                // Every arm's pattern first, then guards and bodies, as
                // rustc does: a later arm's `&S` says what the value is.
                for &arm in &kids[1..] {
                    if ast.kind(arm) == Kind::MatchArm {
                        let ak = self.kids(arm);
                        self.pat(ak[0], scrut, Mutbl::Not, false);
                    }
                }
                for &arm in &kids[1..] {
                    if ast.kind(arm) != Kind::MatchArm {
                        continue;
                    }
                    let ak = self.kids(arm);
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
                        let t = self.branch_to_expected(body, t, expect);
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
                // In a `try` block of a type not yet known: the same kind
                // as this operand's (homogeneous `try` blocks).
                if let Some(&r) = self.try_blocks.last()
                    && self.infer.var_kind(self.infer.shallow(r)).is_some()
                    && let TyKind::Adt(d, l) = tcx.kind(self.norm(t))
                {
                    let mut args = tcx.tys(l);
                    if let Some(first) = args.first_mut() {
                        *first = self.infer.fresh();
                    }
                    let same = self.mk(TyKind::Adt(d, tcx.list(args)));
                    self.infer.unify(r, same);
                }
                let o = self.try_output(n, t);
                self.record_try(n, t);
                // `x?` is a match one of whose arms returns.
                self.diverges_into(o);
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
            // `yield e`: `e` is what the coroutine yields, and the value is
            // what it is resumed with.
            Kind::Yield if self.coroutine.is_some() => {
                let (resume, yields) = self.coroutine.unwrap();
                match kids.iter().copied().find(|&c| is_expr(ast.kind(c))) {
                    Some(e) => {
                        let t = self.expr(e, Some(yields));
                        self.coerce(e, t, yields);
                    }
                    None => {
                        let u = self.unit();
                        if !self.infer.unify(u, yields) {
                            self.err(n, "mismatched types");
                        }
                    }
                }
                resume
            }
            // `become f(x)`: a tail call, which returns its value.
            Kind::Become if kids.len() == 1 && is_expr(ast.kind(kids[0])) => {
                let ret = self.ret;
                let t = self.expr(kids[0], Some(ret));
                self.coerce(kids[0], t, ret);
                self.mk(TyKind::Never)
            }
            // `x.use`: a clone (or copy) of `x`.
            Kind::UseExpr if kids.len() == 1 && is_expr(ast.kind(kids[0])) => self.expr(kids[0], expect),
            // `move(x)`: `x`, moved into the closure.
            Kind::MoveExpr => self.expr(kids[0], expect),
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
            // `c"..."`: `&CStr`.
            b'c' => {
                return match self.lang("CStr") {
                    Some(d) => {
                        let s = self.mk(TyKind::Adt(d, tcx.list(vec![])));
                        self.mk(TyKind::Ref(Mutbl::Not, s))
                    }
                    None => self.infer.fresh(),
                };
            }
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
                Some((tr, super::traits::Select::Yes(_))) => {
                    self.record_trait_method(n, tr, method, sa, &[sb]);
                    // Unknowns on the right (`v == []`): the impl's
                    // `where` clauses may settle them.
                    if tcx.has_infer(self.infer.resolve(sb)) {
                        self.pending_traits.push((sa, tr, tcx.list(vec![sb])));
                    }
                }
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
                Some(tr) => match self.op_rhs_by_coercion(kids[2], tr, sa, b) {
                    Some(w) => {
                        self.record_trait_method(n, tr, &name, sa, &[w]);
                        true
                    }
                    None => false,
                },
                _ => false,
            };
            return if ok { self.unit() } else { self.err(n, "operator on types that do not implement it") };
        }
        if let Some(o) = self.op_trait(n, lang, lang, sa, Some(sb)) {
            return o;
        }
        if let Some(tr) = self.lang(lang)
            && let Some(w) = self.op_rhs_by_coercion(kids[2], tr, sa, b)
            && let Some(o) = self.op_trait(n, lang, lang, sa, Some(w))
        {
            return o;
        }
        self.err(n, "operator on types that do not implement it")
    }

    /// `s += &string`, `s + &string`: the right operand, a reference,
    /// deref-coerced to the right side of the one impl of `tr` for `a`
    /// (`String: AddAssign<&str>`).
    fn op_rhs_by_coercion(&mut self, rhs: NodeId, tr: DefId, a: Ty, b: Ty) -> Option<Ty> {
        let tcx = self.tcx;
        if !matches!(tcx.kind(self.norm(b)), TyKind::Ref(..)) {
            return None;
        }
        let p = self.probe();
        let v = self.infer.fresh();
        let want = match self.select(a, tr, &[v], 0) {
            super::traits::Select::Yes(super::traits::Source::Impl(..)) => self.infer.resolve(v),
            _ => {
                self.rollback(p);
                return None;
            }
        };
        if !matches!(tcx.kind(want), TyKind::Ref(..)) || tcx.has_infer(want) {
            self.rollback(p);
            return None;
        }
        self.coerce(rhs, b, want).then_some(want)
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
        self.check_args_hinted(n, args, inputs, None);
    }

    /// `check_args`, each argument checked against `hints` (what the
    /// expected result says it should be) where given, then coerced to its
    /// parameter's type.
    fn check_args_hinted(&mut self, n: NodeId, args: &[NodeId], inputs: &[Ty], hints: Option<&[Ty]>) {
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
                    // A hint that is only a variable (an `impl Trait` of
                    // the expected result, in the body that defines it)
                    // says less than the parameter's own type: a closure
                    // takes its signature from the `Fn` bound on that.
                    let hint = hints
                        .and_then(|h| h.get(i))
                        .copied()
                        .filter(|&h| self.infer.var_kind(self.infer.shallow(h)) != Some(VarKind::General))
                        .unwrap_or(want);
                    let t = self.expr(a, Some(hint));
                    // A parameter that is still a variable (`Some(run)`'s)
                    // takes the hint's type, coerced to (a function item
                    // made the pointer wanted), as rustc does.
                    // (Not an unsized one, `dyn Future` of `Box::new(async {
                    // .. })` as a `Box<dyn Future>`: no argument is that.)
                    if hint != want
                        && self.infer.var_kind(self.infer.shallow(want)).is_some()
                        && !self.tcx.has_infer(self.infer.resolve(hint))
                        && !matches!(
                            self.tcx.kind(self.infer.shallow(hint)),
                            TyKind::Dyn(_) | TyKind::Slice(_) | TyKind::Str
                        )
                    {
                        let p = self.probe();
                        if self.coerce(a, t, hint) && self.infer.unify(want, hint) {
                            continue;
                        }
                        self.rollback(p);
                    }
                    self.coerce(a, t, want);
                }
                None => {
                    self.expr(a, None);
                }
            }
        }
    }

    fn call(&mut self, n: NodeId, kids: &[NodeId], expect: Option<Ty>) -> Ty {
        let tcx = self.tcx;
        let mut c = kids[0];
        while self.ast.kind(c) == Kind::Paren {
            c = *self.ast.children(c).last().unwrap();
        }
        self.callee = Some(c);
        let callee = self.expr(kids[0], None);
        self.callee = None;
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
            None if self.infer.var_kind(s).is_none() => self.call_fn_traits(n, s, args.len()).or_else(|| {
                // Through `Deref` (`Box<dyn Fn()>`, `impl Deref<Target =
                // fn()>`): the first type on the way that can be called.
                let steps = self.autoderef(s);
                steps.into_iter().skip(1).find_map(|(t, _)| {
                    let t = self.norm(t);
                    match tcx.kind(t) {
                        TyKind::FnDef(d, l) => self.callable_sig(d, l),
                        TyKind::FnPtr(sig) | TyKind::Closure(_, sig, _) => {
                            let sig = tcx.sig_of(sig);
                            Some((tcx.tys(sig.inputs), sig.output))
                        }
                        _ if self.infer.var_kind(t).is_none() => self.call_fn_traits(n, t, args.len()),
                        _ => None,
                    }
                })
            }),
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
                    // What the expected result says the arguments are
                    // (`Vec<Box<dyn Shape>>` from `vec![...]` makes its
                    // array's elements `Box<dyn Shape>`): hints only.
                    let hints = expect.and_then(|e| {
                        let p = self.probe();
                        let h = self.infer.unify(output, e).then(|| inputs.iter().map(|&t| self.infer.resolve(t)).collect::<Vec<Ty>>());
                        self.rollback(p);
                        h
                    });
                    self.check_args_hinted(n, &args, &inputs, hints.as_deref());
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
        // A field this body cannot see (`Box`'s own, outside `alloc`) is
        // passed over for one further along, if there is one.
        let mut hidden = None;
        for k in 0..steps.len() {
            let t = steps[k].0;
            match tcx.kind(t) {
                TyKind::Adt(d, l) => {
                    if let Some(adt) = tcx.adts.get(&d)
                        && adt.kind != crate::ty::AdtKind::Enum
                        && let Some(i) = adt.variants[0].fields.iter().position(|f| f.name == name)
                    {
                        if k + 1 < steps.len() && self.field_hidden(d, i) {
                            hidden.get_or_insert(k);
                            continue;
                        }
                        let ft = subst(tcx, adt.variants[0].fields[i].ty, &tcx.all_params(d), &tcx.tys(l));
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
        if let Some(k) = hidden
            && let TyKind::Adt(d, l) = tcx.kind(steps[k].0)
            && let Some(f) = tcx.adts.get(&d).and_then(|a| a.variants[0].fields.iter().find(|f| f.name == name))
        {
            let ft = subst(tcx, f.ty, &tcx.all_params(d), &tcx.tys(l));
            self.adjust(base_node, Self::step_adjustments(&steps, k, None), steps[k].0);
            return self.normalize(ft, 0);
        }
        self.err(n, "field not found")
    }

    /// Whether field `i` of struct `d` is private to a module this body is
    /// not in (or `pub(...)` within another crate).
    fn field_hidden(&self, d: DefId, i: usize) -> bool {
        let r = self.r;
        let def = &r.defs[d.0 as usize];
        let ast = &self.env.crates[def.krate as usize].files[def.file as usize].ast;
        let Some(list) = ast.children(def.node).iter().copied().find(|&c| matches!(ast.kind(c), Kind::NamedFields | Kind::TupleFields)) else {
            return false;
        };
        let fields: Vec<NodeId> = ast.children(list).iter().copied().filter(|&c| matches!(ast.kind(c), Kind::FieldDef | Kind::TupleField)).collect();
        let Some(&f) = fields.get(i) else { return false };
        let here = r.defs[self.owner.0 as usize].krate;
        match ast.child(f, Kind::Visibility) {
            Some(v) if ast.child(v, Kind::Path).is_none() => false,
            Some(_) => def.krate != here,
            None => {
                let (Some(m), Some(from)) = (def.parent, r.defs[self.owner.0 as usize].parent) else { return false };
                let m = r.modules[m.0 as usize].normal;
                !r.visible(crate::resolve::Vis::Restricted(m), from)
            }
        }
    }

    /// `base[idx]`: built in for arrays, slices and `str`, else `Index`,
    /// through autoderef.
    /// The dereferences (and an array's unsizing, for `Index` on a slice)
    /// are adjustments of `base_node`; an overloaded index records
    /// `Index::index` on `n`.
    fn index(&mut self, n: NodeId, base_node: NodeId, idx_node: NodeId, base: Ty, idx: Ty) -> Ty {
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
        // `v[&string]` with `Index<&str>`: the index deref-coerced.
        if let Some(tr) = self.lang("index")
            && let Some(out) = self.trait_item(tr, "Output", Ns::Type)
        {
            for k in 0..steps.len() {
                let t = steps[k].0;
                if let Some(w) = self.op_rhs_by_coercion(idx_node, tr, t, idx) {
                    self.adjust(base_node, Self::step_adjustments(&steps, k, None), steps[k].0);
                    self.record_trait_method(n, tr, "index", t, &[w]);
                    return self.project_or_var(out, &[t, w]);
                }
            }
        }
        self.err(n, "indexing something that does not implement `Index`")
    }

    fn struct_expr(&mut self, n: NodeId, kids: &[NodeId]) -> Ty {
        let tcx = self.tcx;
        let path = kids[0];
        // `<Foo as A>::Assoc { br: 2 }`: the struct the type is (failing
        // that, as below: `<T as Tr>::Variant { .. }`).
        let qualified = if matches!(self.res(path), Some(Res::Partial(..)))
            && self.ast.child(path, Kind::QSelf).is_some()
        {
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
            let pt = l.path_ty(path);
            let pt = self.fill_holes(pt);
            let pt = self.norm(pt);
            match tcx.kind(pt) {
                TyKind::Adt(a, _) => tcx.adts.get(&a).and_then(|x| x.variants.first().cloned()).map(|v| (pt, Some(v))),
                _ => None,
            }
        } else {
            None
        };
        let (adt_ty, variant) = if let Some(q) = qualified {
            q
        } else {
            match self.res(path) {
                Some(Res::Def(d)) => {
                    let d = *d;
                    match self.r.defs[d.0 as usize].kind {
                        DefKind::Struct | DefKind::Union => {
                            let args = self.item_args(d, path);
                            (
                                self.mk(TyKind::Adt(d, tcx.list(args))),
                                tcx.adts.get(&d).and_then(|a| a.variants.first().cloned()),
                            )
                        }
                        DefKind::Variant => {
                            let args = self.item_args(d, path);
                            let e = tcx.generics[d.0 as usize].parent;
                            let v = e
                                .and_then(|e| tcx.adts.get(&e))
                                .and_then(|a| a.variants.iter().find(|v| v.def == Some(d)).cloned());
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
            }
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
                    // (A field declared through a projection: what it is.)
                    let want = want.map(|w| if has_projection_ty(tcx, w) { self.normalize(w, 0) } else { w });
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
        if std::mem::take(&mut self.coroutine_attr) {
            return self.coroutine_closure(n, kids);
        }
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
            // `dyn Fn(A) -> B` (a closure cast to `&dyn Fn(A)`); a `dyn` of
            // another trait through its bounds, as for a variable.
            Some((e, TyKind::Dyn(d))) => {
                let fns: Vec<DefId> = ["fn", "fn_mut", "fn_once"].iter().filter_map(|l| self.lang(l)).collect();
                let direct = tcx.dyn_of(d).traits.iter().find_map(|tr| {
                    let a = *tcx.tys(tr.args).first().filter(|_| fns.contains(&tr.def))?;
                    let TyKind::Tuple(l) = tcx.kind(a) else { return None };
                    Some((tcx.tys(l), tr.bindings.first().map(|b| b.1)))
                });
                match direct.or_else(|| self.closure_expectation(e)) {
                    Some((i, r)) => {
                        expected_ret = r;
                        Some(i)
                    }
                    None => None,
                }
            }
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
        // `async |x| body`: a call gives a future of the body's value.
        // What it takes and that value may come from `AsyncFn` bounds;
        // `Fn() -> Fut` ones say the future.
        let is_async = kids.iter().any(|&c| ast.kind(c) == Kind::Keyword && ast.token(c).0 == "async");
        let mut expected_fut = None;
        if is_async {
            expected_fut = expected_ret.take();
            if let Some(e) = expect
                && let Some((i, r)) = self.closure_expectation_of(e, true)
            {
                for (&t, &w) in inputs.iter().zip(&i) {
                    self.infer.unify(t, w);
                }
                expected_ret = r;
            }
        }
        let ret = match kids.iter().copied().find(|&c| ast.kind(c) == Kind::RetType) {
            Some(r) => self.written_ty(self.ast.children(r)[0]),
            None => expected_ret.unwrap_or_else(|| self.infer.fresh()),
        };
        let body = *kids.last().unwrap();
        let saved_ret = std::mem::replace(&mut self.ret, ret);
        let saved_co = self.coroutine.take();
        let saved_try = std::mem::take(&mut self.try_blocks);
        let saved_loops = std::mem::take(&mut self.loops);
        let bt = self.expr(body, Some(ret));
        self.coerce(body, bt, ret);
        self.ret = saved_ret;
        self.coroutine = saved_co;
        self.try_blocks = saved_try;
        self.loops = saved_loops;
        let id = tcx.closure_id((self.k, self.file, n));
        self.closures.push(id);
        let ret = if is_async {
            let body_id = tcx.closure_id((self.k, self.file, body));
            let fut = self.mk(TyKind::AsyncBlock(body_id, ret, tcx.identity_args(self.owner)));
            if let Some(e) = expected_fut {
                self.infer.unify(fut, e);
            }
            fut
        } else {
            ret
        };
        let sig = tcx.sig(crate::ty::FnSig { inputs: tcx.list(inputs), output: ret, unsafety: false, variadic: false, abi: crate::ty::Abi::Rust });
        self.mk(TyKind::Closure(id, sig, tcx.identity_args(self.owner)))
    }

    /// Whether a slice pattern's element is `..` (or `rest @ ..`).
    fn has_rest(&self, k: NodeId) -> bool {
        let ast = self.ast;
        ast.kind(k) == Kind::RestPat || ast.kind(k) == Kind::IdentPat && self.kids(k).iter().any(|&c| ast.kind(c) == Kind::RestPat)
    }

    /// The names a checked pattern binds, with their types.
    fn binding_types(&self, pat: NodeId) -> Vec<(&'s str, Ty)> {
        let mut out = Vec::new();
        let mut stack = vec![pat];
        while let Some(p) = stack.pop() {
            if self.ast.kind(p) == Kind::IdentPat
                && self.bindings.contains_key(&p)
                && let (Some(n), Some(&t)) = (self.ast.child(p, Kind::Name), self.types.get(&p))
            {
                out.push((self.ast.token(n).ident_name(), t));
            }
            stack.extend(self.ast.children(p).iter().copied());
        }
        out
    }

    /// `#[coroutine] |arg| body`: a coroutine resumed with `arg` (`()` if
    /// it takes none), yielding what its `yield`s give, and returning the
    /// body's value.
    fn coroutine_closure(&mut self, n: NodeId, kids: &[NodeId]) -> Ty {
        let tcx = self.tcx;
        let ast = self.ast;
        let params = kids.iter().copied().find(|&c| ast.kind(c) == Kind::ClosureParams).map(|p| self.kids(p)).unwrap_or_default();
        let resume = match params.first() {
            Some(&p) => {
                let pk = self.kids(p);
                let t = match pk.get(1) {
                    Some(&ty) => self.written_ty(ty),
                    None => self.infer.fresh(),
                };
                self.pat(pk[0], t, Mutbl::Not, false);
                t
            }
            None => self.unit(),
        };
        let ret = match kids.iter().copied().find(|&c| ast.kind(c) == Kind::RetType) {
            Some(r) => self.written_ty(self.ast.children(r)[0]),
            None => self.infer.fresh(),
        };
        let yields = self.infer.fresh();
        let body = *kids.last().unwrap();
        let saved_ret = std::mem::replace(&mut self.ret, ret);
        let saved_co = self.coroutine.replace((resume, yields));
        let saved_try = std::mem::take(&mut self.try_blocks);
        let saved_loops = std::mem::take(&mut self.loops);
        let bt = self.expr(body, Some(ret));
        self.coerce(body, bt, ret);
        self.ret = saved_ret;
        self.coroutine = saved_co;
        self.try_blocks = saved_try;
        self.loops = saved_loops;
        let id = tcx.closure_id((self.k, self.file, n));
        self.closures.push(id);
        let parts = self.mk(TyKind::Tuple(tcx.list(vec![resume, yields, ret])));
        self.mk(TyKind::Coroutine(id, parts, tcx.identity_args(self.owner)))
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
            "core_panic" | "std_panic" | "unreachable_2015" | "unreachable_2021" | "unreachable" | "panic" => {
                self.mk(TyKind::Never)
            }
            _ => self.infer.fresh(),
        }
    }

    // ---- statements ----

    fn block(&mut self, n: NodeId, expect: Option<Ty>) -> Ty {
        let ast = self.ast;
        let kids = self.kids(n);
        let stmts: Vec<NodeId> = kids
            .iter()
            .copied()
            .filter(|&c| {
                !matches!(ast.kind(c), Kind::Lifetime | Kind::Keyword | Kind::InnerAttribute) && !is_type(ast.kind(c))
            })
            .collect();
        if ast.kind(n) == Kind::ConstBlock {
            return self.expr(kids[0], expect);
        }
        let keyword = |k: &str| kids.iter().any(|&c| ast.kind(c) == Kind::Keyword && ast.token(c).0 == k);
        // `try { e }`: some `R: Try<Output = typeof e>`, which the
        // context says.
        if keyword("try") {
            // `try bikeshed T { .. }`: of type `T`; otherwise what the
            // context says, or the first `?` in it (`Option<_>` for an
            // `Option`'s).
            let written = kids.iter().copied().find(|&c| is_type(ast.kind(c)));
            let r = match (written, expect) {
                (Some(w), _) => self.written_ty(w),
                (None, Some(e)) => e,
                (None, None) => self.infer.fresh(),
            };
            self.try_blocks.push(r);
            let out = self.stmts(&stmts, None);
            self.try_blocks.pop();
            if let Some(tr) = self.lang("Try")
                && let Some(o) = self.trait_item(tr, "Output", Ns::Type)
            {
                let rs = self.norm(r);
                if self.infer.var_kind(rs).is_some() {
                    let tcx = self.tcx;
                    self.obligations.push(Pred::Projection(rs, o, tcx.list(vec![rs]), out));
                    // Worked out once something says what `R` is
                    // (`if let Err(()) = try {}`).
                    self.pending.push((o, tcx.list(vec![rs]), out));
                } else {
                    let ro = self.project_or_var(o, &[rs]);
                    self.infer.unify(ro, out);
                }
            }
            return r;
        }
        // `async` blocks: a future of their own, whose output is the
        // block's value (or what a `return` in it gives).
        if keyword("async") && !keyword("gen") {
            let tcx = self.tcx;
            let out = self.infer.fresh();
            let saved_ret = std::mem::replace(&mut self.ret, out);
            let saved_co = self.coroutine.take();
            let saved_try = std::mem::take(&mut self.try_blocks);
            let saved_loops = std::mem::take(&mut self.loops);
            let bt = self.stmts(&stmts, Some(out));
            self.ret = saved_ret;
            self.coroutine = saved_co;
            self.try_blocks = saved_try;
            self.loops = saved_loops;
            if tcx.kind(self.infer.shallow(bt)) == TyKind::Never {
                // (`async { unimplemented!() }`: falls back as `!` does.)
                self.diverges_into(out);
            } else if !self.infer.unify(bt, out) {
                self.err(n, "mismatched types");
            }
            let id = tcx.closure_id((self.k, self.file, n));
            return self.mk(TyKind::AsyncBlock(id, out, tcx.identity_args(self.owner)));
        }
        // `gen` blocks (and `async gen`): an iterator of their own, of
        // what their `yield`s give.
        if keyword("gen") {
            let tcx = self.tcx;
            let yields = self.infer.fresh();
            let unit = self.unit();
            let saved_ret = std::mem::replace(&mut self.ret, unit);
            let saved_co = self.coroutine.replace((unit, yields));
            let saved_try = std::mem::take(&mut self.try_blocks);
            let saved_loops = std::mem::take(&mut self.loops);
            self.stmts(&stmts, Some(unit));
            self.ret = saved_ret;
            self.coroutine = saved_co;
            self.try_blocks = saved_try;
            self.loops = saved_loops;
            let id = tcx.closure_id((self.k, self.file, n));
            return self.mk(TyKind::GenBlock(id, yields, tcx.identity_args(self.owner), keyword("async")));
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
                    // `#[coroutine] || { .. };` as a statement.
                    let ast = self.ast;
                    self.coroutine_attr = ast.kind(e) == Kind::Closure
                        && ast.children(s).iter().any(|&a| {
                            ast.kind(a) == Kind::Attribute
                                && ast.text(a).trim_start_matches("#[").starts_with("coroutine")
                        });
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
        let peels = !matches!(
            kind,
            Kind::IdentPat
                | Kind::WildPat
                | Kind::RefPat
                | Kind::RestPat
                | Kind::OrPat
                | Kind::ParenPat
                | Kind::GuardPat
        ) || kind == Kind::IdentPat && self.r.res_of(self.k as usize, self.file as usize, p).is_some();
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
                let words: Vec<&str> =
                    kids.iter().filter(|&&c| ast.kind(c) == Kind::Keyword).map(|&c| ast.token(c).0).collect();
                let has_ref = words.contains(&"ref");
                // `ref mut x`, not `mut ref x` (a mutable binding).
                let is_mut = words.windows(2).any(|w| w == ["ref", "mut"]) || !has_ref && words.contains(&"mut");
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
            // `p if e`: `e` is a `bool`, checked once `p` binds its names.
            Kind::GuardPat => {
                self.pat(kids[0], t, bm, by_ref);
                let b = self.mk(TyKind::Bool);
                let g = self.expr(kids[1], Some(b));
                if !self.infer.unify(g, b) {
                    self.err(kids[1], "mismatched types");
                }
            }
            Kind::OrPat => {
                for &k in &kids {
                    self.pat(k, t, bm, by_ref);
                }
                // A name bound in each alternative has one type
                // (`Ok(a) | Err(a)`).
                let first = self.binding_types(kids[0]);
                for &k in &kids[1..] {
                    for (name, bt) in self.binding_types(k) {
                        if let Some(&(_, ft)) = first.iter().find(|f| f.0 == name) {
                            self.infer.unify(ft, bt);
                        }
                    }
                }
            }
            Kind::RefPat => {
                let inner = *kids.last().unwrap();
                let keyword = |k: &str| kids.iter().any(|&c| ast.kind(c) == Kind::Keyword && ast.token(c).0 == k);
                let m = if keyword("mut") { Mutbl::Mut } else { Mutbl::Not };
                let it = self.infer.fresh();
                let rt = self.mk(TyKind::Ref(m, it));
                // `&pin mut p`: through a `Pin<&mut T>`.
                let rt = if keyword("pin") { tcx.pin(rt) } else { rt };
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
                    // `r#move` names the field `move`.
                    let (name, sub) = if fk.len() == 2 {
                        (ast.token(fk[0]).ident_name(), fk[1])
                    } else {
                        let sub = fk[0];
                        (ast.child(sub, Kind::Name).map_or("", |n| ast.token(n).ident_name()), sub)
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
                    // `[_, _]` with no `..`: an array of that many.
                    TyKind::Array(e, _) => {
                        if !kids.iter().any(|&k| self.has_rest(k)) {
                            let len = self.mk(TyKind::Array(e, tcx.konst(crate::ty::Const::Value(kids.len() as u128))));
                            self.infer.unify(st, len);
                        }
                        e
                    }
                    TyKind::Slice(e) => e,
                    // Not known yet: with no `..`, an array of that many
                    // (`let [_, _] = a.into();`), as rustc infers it.
                    _ => {
                        let e = self.infer.fresh();
                        let sl = if kids.iter().any(|&k| self.has_rest(k)) {
                            self.mk(TyKind::Slice(e))
                        } else {
                            self.mk(TyKind::Array(e, tcx.konst(crate::ty::Const::Value(kids.len() as u128))))
                        };
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
        // Through a type alias (`Variant { span, .. }`) or an associated
        // type (`<Foo as A>::Assoc { br }`): the struct it names, with its
        // arguments.
        let aliased = match self.res(path) {
            Some(Res::Def(d)) => self.r.defs[d.0 as usize].kind == DefKind::TypeAlias,
            Some(Res::Partial(..)) => self.ast.child(path, Kind::QSelf).is_some(),
            _ => false,
        };
        if aliased {
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
            let pt = l.path_ty(path);
            let pt = self.fill_holes(pt);
            let pt = self.norm(pt);
            if let TyKind::Adt(d, l) = tcx.kind(pt)
                && let Some(v) = tcx.adts.get(&d).and_then(|a| a.variants.first().cloned())
            {
                self.infer.unify(pt, t);
                let (params, args) = (tcx.all_params(d), tcx.tys(l));
                let fields = v.fields.iter().map(|f| (f.name, subst(tcx, f.ty, &params, &args))).collect();
                return (pt, fields);
            }
        }
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

/// Whether `t` has an `impl Trait` in it (at the top, behind references,
/// or among type arguments).
fn has_projection_ty(tcx: &crate::ty::TyCtxt, t: Ty) -> bool {
    match tcx.kind(t) {
        TyKind::Projection(..) => true,
        TyKind::Ref(_, x) | TyKind::Ptr(_, x) | TyKind::Slice(x) | TyKind::Array(x, _) => has_projection_ty(tcx, x),
        TyKind::Adt(_, l) | TyKind::Tuple(l) => tcx.tys(l).into_iter().any(|x| has_projection_ty(tcx, x)),
        _ => false,
    }
}

fn has_opaque(tcx: &crate::ty::TyCtxt, t: Ty) -> bool {
    match tcx.kind(t) {
        TyKind::Opaque(..) => true,
        TyKind::Ref(_, x) => has_opaque(tcx, x),
        TyKind::Adt(_, l) | TyKind::Tuple(l) => tcx.tys(l).into_iter().any(|x| has_opaque(tcx, x)),
        TyKind::Dyn(d) => tcx.dyn_of(d).traits.iter().any(|tr| tr.bindings.iter().any(|b| has_opaque(tcx, b.1))),
        _ => false,
    }
}

/// `EPITOME_TYPECK_WHY`: print the errors whose reason has this in it.
fn why_filter() -> Option<&'static str> {
    static WHY: std::sync::OnceLock<Option<String>> = std::sync::OnceLock::new();
    WHY.get_or_init(|| std::env::var("EPITOME_TYPECK_WHY").ok()).as_deref()
}

/// Whether a type has a `dyn` in it.
fn has_dyn(tcx: &TyCtxt, t: Ty) -> bool {
    match tcx.kind(t) {
        TyKind::Dyn(_) => true,
        TyKind::Ref(_, x) | TyKind::Ptr(_, x) => has_dyn(tcx, x),
        TyKind::Adt(_, l) => tcx.tys(l).into_iter().any(|x| has_dyn(tcx, x)),
        _ => false,
    }
}
