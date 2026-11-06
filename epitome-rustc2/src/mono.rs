//! Monomorphisation: from the root functions, every function (at the
//! generic arguments it is used with), closure body, vtable, static and
//! constant the program needs.
//!
//! An item's body is walked with its arguments substituted for its
//! generic parameters, and the types it then mentions normalised. The
//! calls in it come from type checking's tables:
//!
//! - a path to a function (called, or made a function pointer) is a
//!   `FnDef` with its arguments;
//! - method calls, overloaded operators, `*` and `[]`, and calls through
//!   the `Fn` traits are in `methods`; `for` and `?` in `implicit`;
//! - autoderef through `Deref` is in `adjustments`, and so is unsizing to
//!   `dyn Trait`, which needs the concrete type's vtable: every method of
//!   the trait (and its supertraits) for that type.
//!
//! A call of a trait method is resolved to the impl's method (or the
//! trait's default) by selecting the impl for the concrete `Self`. Calls
//! the compiler implements (a closure through `Fn::call`, a function
//! pointer, a `dyn` method) need nothing more here. Each item is a job on
//! the pool, and finding a new item adds a job, so the walk runs in
//! parallel.

use crate::ast::{Kind, NodeId};
use crate::hash::{FastMap, FastSet};
use crate::pool;
use crate::resolve::{DefId, DefKind, Ns, Res, Resolver};
use crate::session::{ParsedCrate, Session};
use crate::ty::collect::subst;
use crate::ty::{ClosureId, List, Mutbl, Ty, TyCtxt, TyKind};
use crate::typeck::{Adjust, Index, Solver, TypeckResults};
use std::sync::Mutex;

/// Something the program needs code or data for.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, PartialOrd, Ord)]
pub enum Item {
    /// A function with a body, at its arguments (its parents' parameters
    /// first, as `TyCtxt::all_params` orders them).
    Fn(DefId, List),
    /// A closure's body, in its function at the function's arguments.
    Closure(ClosureId, DefId, List),
    /// A compiler intrinsic, at its arguments.
    Intrinsic(DefId, List),
    /// A function from an `extern` block (or with no body), by name.
    Foreign(DefId),
    /// A `static`.
    Static(DefId),
    /// A `const` (or associated constant) at its arguments, computed where
    /// it is used.
    Const(DefId, List),
    /// The vtable of a concrete type for a `dyn` trait.
    VTable(Ty, DefId),
}

/// What monomorphisation found.
pub struct MonoItems {
    /// Every item, sorted.
    pub items: Vec<Item>,
    /// Calls and uses found, over all bodies.
    pub uses: usize,
    /// What could not be resolved, with counts, most common first.
    pub unresolved: Vec<(String, usize)>,
}

impl Session {
    /// The items the `roots` need, transitively.
    pub fn collect_mono_items<'s>(
        &'s self,
        crates: &[ParsedCrate<'s>],
        r: &Resolver<'s>,
        tcx: &TyCtxt<'s>,
        index: &Index<'s>,
        typeck: &TypeckResults,
        roots: Vec<Item>,
    ) -> MonoItems {
        let seen: Mutex<FastSet<Item>> = Mutex::new(roots.iter().copied().collect());
        let unresolved: Mutex<FastMap<String, usize>> = Mutex::new(FastMap::default());
        let uses = std::sync::atomic::AtomicUsize::new(0);
        pool::run(self.threads(), roots, |item, q| {
            let mut w = Walk {
                tcx,
                r,
                crates,
                typeck,
                solver: Solver::new(tcx, r, crates, index),
                found: Vec::new(),
                unresolved: FastMap::default(),
            };
            w.item(item);
            uses.fetch_add(w.found.len(), std::sync::atomic::Ordering::Relaxed);
            {
                let mut seen = seen.lock().unwrap();
                for f in w.found {
                    if seen.insert(f) {
                        q.push(f);
                    }
                }
            }
            let mut u = unresolved.lock().unwrap();
            for (k, v) in w.unresolved {
                *u.entry(k).or_default() += v;
            }
        });
        let mut items: Vec<Item> = seen.into_inner().unwrap().into_iter().collect();
        items.sort();
        let mut unresolved: Vec<(String, usize)> = unresolved.into_inner().unwrap().into_iter().collect();
        unresolved.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
        MonoItems { items, uses: uses.into_inner(), unresolved }
    }
}

/// The roots of crate `k`: its `main` if it has one, else every function
/// that is not generic.
pub fn roots(r: &Resolver, tcx: &TyCtxt, k: u32) -> Vec<Item> {
    let fns: Vec<DefId> = (0..r.defs.len() as u32)
        .map(DefId)
        .filter(|&d| {
            let def = &r.defs[d.0 as usize];
            def.krate == k && matches!(def.kind, DefKind::Fn | DefKind::AssocFn) && tcx.all_params(d).is_empty()
        })
        .collect();
    let empty = tcx.list(Vec::new());
    match fns.iter().find(|&&d| r.defs[d.0 as usize].name == "main" && r.defs[d.0 as usize].kind == DefKind::Fn) {
        Some(&m) => vec![Item::Fn(m, empty)],
        None => fns.into_iter().map(|d| Item::Fn(d, empty)).collect(),
    }
}

struct Walk<'a, 's> {
    tcx: &'a TyCtxt<'s>,
    r: &'a Resolver<'s>,
    crates: &'a [ParsedCrate<'s>],
    typeck: &'a TypeckResults,
    solver: Solver<'a, 's>,
    found: Vec<Item>,
    unresolved: FastMap<String, usize>,
}

impl<'a, 's> Walk<'a, 's> {
    fn miss(&mut self, why: String) {
        *self.unresolved.entry(why).or_default() += 1;
    }

    fn item(&mut self, item: Item) {
        match item {
            Item::Fn(d, args) => self.body(d, args),
            Item::Const(d, args) => self.body(d, args),
            Item::Static(d) => {
                let empty = self.tcx.list(Vec::new());
                self.body(d, empty)
            }
            Item::VTable(t, tr) => self.vtable(t, tr),
            Item::Closure(..) | Item::Intrinsic(..) | Item::Foreign(_) => {}
        }
    }

    /// Walks the body of `d` at `args`.
    fn body(&mut self, d: DefId, args: List) {
        let tcx = self.tcx;
        let def = &self.r.defs[d.0 as usize];
        let (k, f) = (def.krate as usize, def.file as usize);
        let ast = &self.crates[k].files[f].ast;
        let types = &self.typeck.types[k][f];
        let tables = &self.typeck.tables[k][f];
        let params = tcx.all_params(d);
        let args = tcx.tys(args);
        let mut nodes = Vec::new();
        crate::typeck::collect_nodes(ast, def.node, &mut nodes);
        for n in nodes {
            match ast.kind(n) {
                Kind::PathExpr | Kind::IdentPat | Kind::PathPat => self.path_use(k, f, n, &params, &args),
                Kind::Closure => {
                    if let Some(Some(t)) = types.get(n.0 as usize)
                        && let TyKind::Closure(c, _, _) = tcx.kind(*t)
                    {
                        let l = tcx.list(args.clone());
                        self.found.push(Item::Closure(c, d, l));
                    }
                }
                _ => {}
            }
            if let Some(&(m, margs)) = tables.methods.get(&n) {
                let margs = self.sub_list(margs, &params, &args);
                self.call(m, margs);
            }
            if let Some(&(c, cargs)) = tables.consts.get(&n) {
                let cargs = self.sub_list(cargs, &params, &args);
                self.call(c, cargs);
            }
            if let Some(calls) = tables.implicit.get(&n) {
                for &(m, margs) in calls {
                    let margs = self.sub_list(margs, &params, &args);
                    self.call(m, margs);
                }
            }
            if let Some(adj) = tables.adjustments.get(&n)
                && let Some(Some(t)) = types.get(n.0 as usize)
            {
                let adj = adj.clone();
                let start = self.sub(*t, &params, &args);
                let end = tables.adjusted.get(&n).map(|&t| self.sub(t, &params, &args));
                self.adjustments(start, &adj, end);
            }
        }
    }

    fn sub(&mut self, t: Ty, params: &[crate::ty::ParamId], args: &[Ty]) -> Ty {
        sub(self.tcx, &self.typeck.hidden, &mut self.solver, t, params, args)
    }

    fn sub_list(&mut self, l: List, params: &[crate::ty::ParamId], args: &[Ty]) -> Vec<Ty> {
        self.tcx.tys(l).into_iter().map(|t| self.sub(t, params, args)).collect()
    }

    /// A path used as a value (or in a pattern): a function, `static` or
    /// constant.
    fn path_use(&mut self, k: usize, f: usize, n: NodeId, params: &[crate::ty::ParamId], args: &[Ty]) {
        let tcx = self.tcx;
        let types = &self.typeck.types[k][f];
        let ast = &self.crates[k].files[f].ast;
        if let Some(Some(t)) = types.get(n.0 as usize)
            && let TyKind::FnDef(fd, fargs) = tcx.kind(*t)
            && ast.kind(n) == Kind::PathExpr
        {
            let fargs = self.sub_list(fargs, params, args);
            self.call(fd, fargs);
            return;
        }
        let path = if ast.kind(n) == Kind::IdentPat { n } else { ast.children(n).first().copied().unwrap_or(n) };
        if let Some(Res::Def(d)) = self.r.res_of(k, f, path) {
            let d = *d;
            match self.r.defs[d.0 as usize].kind {
                DefKind::Static => self.found.push(Item::Static(d)),
                DefKind::ForeignStatic => self.found.push(Item::Foreign(d)),
                _ => {}
            }
        }
    }

    /// A call (or use) of function `d` at `args`.
    fn call(&mut self, d: DefId, args: Vec<Ty>) {
        let tcx = self.tcx;
        if args.iter().any(|&a| has_param_or_infer(tcx, a)) {
            let a: Vec<String> = args.iter().map(|&t| tcx.display(self.r, t)).collect();
            self.miss(format!("arguments not concrete: {}<{}>", self.r.def_path(d), a.join(", ")));
            return;
        }
        match instance(self.r, self.crates, tcx, &mut self.solver, d, args.clone()) {
            Resolved::Item(it) => self.found.push(it),
            Resolved::Builtin => self.builtin_call(d, &args),
            Resolved::NoImpl => self.miss("associated constant without an impl".into()),
            Resolved::None => {}
        }
    }

    /// A trait method with a built-in impl: a closure, function item or
    /// pointer called through `Fn`, a method of a `dyn` (through its
    /// vtable), or `Clone::clone` of a tuple or pattern type. A function
    /// item's own body is what runs; a tuple's clone clones each element.
    fn builtin_call(&mut self, d: DefId, args: &[Ty]) {
        let tcx = self.tcx;
        let self_ty = args[0];
        if trait_of(self.r, d).is_some_and(|tr| tcx.lang.get("clone") == Some(&tr)) {
            match tcx.kind(self_ty) {
                TyKind::Tuple(l) => {
                    for t in tcx.tys(l) {
                        self.call(d, vec![t]);
                    }
                    return;
                }
                TyKind::Pat(..)
                | TyKind::Bool
                | TyKind::Char
                | TyKind::Int(_)
                | TyKind::Uint(_)
                | TyKind::Float(_)
                | TyKind::Never
                | TyKind::Ref(Mutbl::Not, _)
                | TyKind::Ptr(..)
                | TyKind::FnDef(..)
                | TyKind::FnPtr(_)
                | TyKind::Closure(..) => return,
                _ => {}
            }
        }
        let inner = match tcx.kind(self_ty) {
            TyKind::Ref(_, x) => x,
            _ => self_ty,
        };
        match tcx.kind(inner) {
            TyKind::FnDef(f, fargs) => self.call(f, tcx.tys(fargs)),
            TyKind::Closure(..) | TyKind::FnPtr(_) | TyKind::Dyn(_) => {}
            _ => {
                let name = self.r.def_path(d);
                self.miss(format!("no impl for {name} on {}", tcx.display(self.r, self_ty)));
            }
        }
    }

    /// What `adj` calls, from `start` (the expression's type); `end` is
    /// the type after them.
    fn adjustments(&mut self, start: Ty, adj: &[Adjust], end: Option<Ty>) {
        let tcx = self.tcx;
        let mut cur = start;
        for (i, a) in adj.iter().enumerate() {
            match a {
                Adjust::Deref => {
                    cur = match tcx.kind(cur) {
                        TyKind::Ref(_, x) | TyKind::Ptr(_, x) => x,
                        TyKind::Adt(_, l) => tcx.tys(l).first().copied().unwrap_or(cur),
                        _ => cur,
                    }
                }
                Adjust::OverloadedDeref => {
                    let mutable = matches!(adj.get(i + 1), Some(Adjust::Borrow(Mutbl::Mut)));
                    let Some(&deref) = tcx.lang.get("deref") else { return };
                    if let Some(m) = self.method_of(deref, "deref") {
                        self.call(m, vec![cur]);
                    }
                    if mutable
                        && let Some(&dm) = tcx.lang.get("deref_mut")
                        && let Some(m) = self.method_of(dm, "deref_mut")
                    {
                        self.call(m, vec![cur]);
                    }
                    let Some(&target) = tcx.lang.get("deref_target") else { return };
                    let p = tcx.mk(TyKind::Projection(target, tcx.list(vec![cur])));
                    cur = self.solver.normalize(p);
                }
                Adjust::Borrow(m) => cur = tcx.mk(TyKind::Ref(*m, cur)),
                Adjust::Unsize => {
                    let Some(to) = end else { return };
                    let pointee = |t: Ty| match tcx.kind(t) {
                        TyKind::Ref(_, x) | TyKind::Ptr(_, x) => Some(x),
                        TyKind::Adt(_, l) => tcx.tys(l).first().copied(),
                        _ => None,
                    };
                    if let (Some(src), Some(dst)) = (pointee(cur), pointee(to))
                        && let TyKind::Dyn(dy) = tcx.kind(dst)
                        && !matches!(tcx.kind(src), TyKind::Dyn(_))
                        && let Some(tr) = tcx.dyn_of(dy).traits.first()
                    {
                        self.found.push(Item::VTable(src, tr.def));
                    }
                    cur = to;
                }
                _ => {}
            }
        }
    }

    fn method_of(&self, tr: DefId, name: &str) -> Option<DefId> {
        let m = self.r.defs[tr.0 as usize].module?;
        match self.r.modules[m.0 as usize].names.get(&(name, Ns::Value)).map(|b| &b.res) {
            Some(Res::Def(d)) => Some(*d),
            _ => None,
        }
    }

    /// A vtable of `t` for trait `tr`: each method of `tr` and its
    /// supertraits, for `t`.
    fn vtable(&mut self, t: Ty, tr: DefId) {
        let tcx = self.tcx;
        let mut traits = vec![tr];
        let mut i = 0;
        while i < traits.len() {
            if let Some(td) = tcx.traits.get(&traits[i]) {
                for s in &td.supertraits {
                    if !traits.contains(&s.def) {
                        traits.push(s.def);
                    }
                }
            }
            i += 1;
        }
        for tr in traits {
            let Some(m) = self.r.defs[tr.0 as usize].module else { continue };
            let methods: Vec<DefId> = self.r.modules[m.0 as usize]
                .names
                .values()
                .filter_map(|b| match b.res {
                    Res::Def(d) if self.r.defs[d.0 as usize].kind == DefKind::AssocFn => Some(d),
                    _ => None,
                })
                .collect();
            for method in methods {
                // Methods with their own generic parameters are not in a
                // vtable (they cannot be called through `dyn`).
                let n = tcx.all_params(method).len();
                let ntrait = tcx.all_params(tr).len();
                if n != ntrait {
                    continue;
                }
                let mut args = vec![t];
                // The trait's other arguments are not known here; traits
                // with them are rare as `dyn`.
                if ntrait > 1 {
                    continue;
                }
                args.truncate(n);
                self.call(method, args);
            }
        }
    }
}

/// `t` at an instance: `args` put for `params`, `impl Trait` revealed,
/// projections normalised.
pub(crate) fn sub(
    tcx: &TyCtxt,
    hidden: &FastMap<crate::ty::OpaqueId, Ty>,
    solver: &mut Solver,
    t: Ty,
    params: &[crate::ty::ParamId],
    args: &[Ty],
) -> Ty {
    let t = subst(tcx, t, params, args);
    let t = reveal(tcx, hidden, t);
    let t = solver.normalize(t);
    reveal(tcx, hidden, t)
}

/// What a use of item `d` at concrete `args` is.
pub(crate) enum Resolved {
    /// A function, intrinsic, foreign function or constant.
    Item(Item),
    /// A trait method with a built-in impl (`Fn` of a closure, `Clone` of
    /// a `Copy` type, a `dyn` method).
    Builtin,
    /// An associated constant with no impl.
    NoImpl,
    /// Not code: a constructor, say.
    None,
}

/// Whether `d` is a compiler intrinsic.
pub(crate) fn is_intrinsic(r: &Resolver, crates: &[ParsedCrate], d: DefId) -> bool {
    let def = &r.defs[d.0 as usize];
    let ast = &crates[def.krate as usize].files[def.file as usize].ast;
    crate::session::has_attribute(ast, def.node, "rustc_intrinsic")
}

/// The trait an associated item is in, if it is in one.
pub(crate) fn trait_of(r: &Resolver, item: DefId) -> Option<DefId> {
    let m = r.defs[item.0 as usize].parent?;
    let d = r.modules[m.0 as usize].def?;
    (r.defs[d.0 as usize].kind == DefKind::Trait).then_some(d)
}

/// Resolves a use of `d` at `args` (all concrete): a trait method to the
/// impl's method (or the trait's default), an associated constant to the
/// impl's.
pub(crate) fn instance(r: &Resolver, crates: &[ParsedCrate], tcx: &TyCtxt, solver: &mut Solver, d: DefId, args: Vec<Ty>) -> Resolved {
    let kind = r.defs[d.0 as usize].kind;
    match kind {
        DefKind::ForeignFn => Resolved::Item(Item::Foreign(d)),
        DefKind::Const => Resolved::Item(Item::Const(d, tcx.list(args))),
        DefKind::AssocConst => match solver.resolve_method(d, &args) {
            Some((c, a)) => Resolved::Item(Item::Const(c, tcx.list(a))),
            None => Resolved::NoImpl,
        },
        DefKind::Fn | DefKind::AssocFn => {
            let (f, a) = if kind == DefKind::AssocFn && trait_of(r, d).is_some() {
                match solver.resolve_method(d, &args) {
                    Some(x) => x,
                    None => return Resolved::Builtin,
                }
            } else {
                (d, args)
            };
            let l = tcx.list(a);
            let def = &r.defs[f.0 as usize];
            let ast = &crates[def.krate as usize].files[def.file as usize].ast;
            Resolved::Item(if is_intrinsic(r, crates, f) {
                Item::Intrinsic(f, l)
            } else if ast.child(def.node, Kind::Block).is_none() {
                Item::Foreign(f)
            } else {
                Item::Fn(f, l)
            })
        }
        _ => Resolved::None,
    }
}

/// Whether a type still has generic parameters or variables in it.
pub(crate) fn has_param_or_infer(tcx: &TyCtxt, t: Ty) -> bool {
    match tcx.kind(t) {
        TyKind::Param(_) | TyKind::Infer(_) | TyKind::Error => true,
        TyKind::Tuple(l) | TyKind::Adt(_, l) | TyKind::FnDef(_, l) | TyKind::Projection(_, l) => {
            tcx.tys(l).into_iter().any(|x| has_param_or_infer(tcx, x))
        }
        TyKind::Ref(_, x) | TyKind::Ptr(_, x) | TyKind::Slice(x) | TyKind::Pat(x, ..) => has_param_or_infer(tcx, x),
        TyKind::Array(x, c) => {
            has_param_or_infer(tcx, x) || matches!(tcx.const_of(c), crate::ty::Const::Param(_) | crate::ty::Const::Infer(_) | crate::ty::Const::Error)
        }
        TyKind::FnPtr(s) => {
            let s = tcx.sig_of(s);
            has_param_or_infer(tcx, s.output) || tcx.tys(s.inputs).into_iter().any(|x| has_param_or_infer(tcx, x))
        }
        TyKind::Closure(_, s, l) => {
            let s = tcx.sig_of(s);
            has_param_or_infer(tcx, s.output)
                || tcx.tys(s.inputs).into_iter().any(|x| has_param_or_infer(tcx, x))
                || tcx.tys(l).into_iter().any(|x| has_param_or_infer(tcx, x))
        }
        // Not revealed: its owner's body did not say what it is.
        TyKind::Opaque(..) => true,
        TyKind::ConstArg(c) => matches!(tcx.const_of(c), crate::ty::Const::Param(_) | crate::ty::Const::Infer(_) | crate::ty::Const::Error),
        _ => false,
    }
}

/// Whether any of `ts` still has generic parameters or variables in it.
pub(crate) fn has_param_or_infer_list(tcx: &TyCtxt, ts: &[Ty]) -> bool {
    ts.iter().any(|&t| has_param_or_infer(tcx, t))
}

/// `t` with each `impl Trait` in it replaced by what it is (its hidden
/// type, at its arguments).
pub fn reveal(tcx: &TyCtxt, hidden: &FastMap<crate::ty::OpaqueId, Ty>, t: Ty) -> Ty {
    let list = |l: List| tcx.list(tcx.tys(l).into_iter().map(|x| reveal(tcx, hidden, x)).collect());
    match tcx.kind(t) {
        TyKind::Opaque(o, l) => match hidden.get(&o) {
            Some(&h) => {
                let owner = tcx.opaque(o).owner;
                let h = subst(tcx, h, &tcx.all_params(owner), &tcx.tys(l));
                reveal(tcx, hidden, h)
            }
            None => t,
        },
        TyKind::Tuple(l) => tcx.mk(TyKind::Tuple(list(l))),
        TyKind::Adt(d, l) => tcx.mk(TyKind::Adt(d, list(l))),
        TyKind::FnDef(d, l) => tcx.mk(TyKind::FnDef(d, list(l))),
        TyKind::Projection(d, l) => tcx.mk(TyKind::Projection(d, list(l))),
        TyKind::Ref(m, x) => tcx.mk(TyKind::Ref(m, reveal(tcx, hidden, x))),
        TyKind::Ptr(m, x) => tcx.mk(TyKind::Ptr(m, reveal(tcx, hidden, x))),
        TyKind::Slice(x) => tcx.mk(TyKind::Slice(reveal(tcx, hidden, x))),
        TyKind::Array(x, c) => tcx.mk(TyKind::Array(reveal(tcx, hidden, x), c)),
        TyKind::Pat(x, a, b) => tcx.mk(TyKind::Pat(reveal(tcx, hidden, x), a, b)),
        TyKind::FnPtr(s) => {
            let mut sig = tcx.sig_of(s);
            sig.inputs = list(sig.inputs);
            sig.output = reveal(tcx, hidden, sig.output);
            tcx.mk(TyKind::FnPtr(tcx.sig(sig)))
        }
        TyKind::Closure(c, s, l) => {
            let mut sig = tcx.sig_of(s);
            sig.inputs = list(sig.inputs);
            sig.output = reveal(tcx, hidden, sig.output);
            tcx.mk(TyKind::Closure(c, tcx.sig(sig), list(l)))
        }
        _ => t,
    }
}

/// An item's path, with an impl's item as `<Type as Trait>::name` (or
/// `<Type>::name`) rather than `{impl}::name`.
pub fn item_path(r: &Resolver, tcx: &TyCtxt, d: DefId) -> String {
    let def = &r.defs[d.0 as usize];
    if let Some(m) = def.parent
        && let Some(i) = r.modules[m.0 as usize].def
        && r.defs[i.0 as usize].kind == DefKind::Impl
        && let Some(&ix) = tcx.impl_index.get(&i)
    {
        let imp = &tcx.impls[ix];
        let self_ty = tcx.display(r, imp.self_ty);
        return match &imp.trait_ref {
            Some(t) => format!("<{self_ty} as {}>::{}", r.def_path(t.def), def.name),
            None => format!("<{self_ty}>::{}", def.name),
        };
    }
    r.def_path(d)
}

impl MonoItems {
    /// One item per line, for `--dump-mono`.
    pub fn dump(&self, r: &Resolver, tcx: &TyCtxt) -> String {
        use std::fmt::Write;
        let args = |l: List| {
            let v = tcx.tys(l);
            if v.is_empty() {
                String::new()
            } else {
                let s: Vec<String> = v.into_iter().map(|t| tcx.display(r, t)).collect();
                format!("<{}>", s.join(", "))
            }
        };
        let mut lines: Vec<String> = self
            .items
            .iter()
            .map(|it| match *it {
                Item::Fn(d, l) => format!("fn {}{}", item_path(r, tcx, d), args(l)),
                Item::Closure(c, d, l) => format!("closure @{} in {}{}", tcx.closure_at(c).2.0, r.def_path(d), args(l)),
                Item::Intrinsic(d, l) => format!("intrinsic {}{}", r.def_path(d), args(l)),
                Item::Foreign(d) => format!("foreign {}", r.def_path(d)),
                Item::Static(d) => format!("static {}", r.def_path(d)),
                Item::Const(d, l) => format!("const {}{}", r.def_path(d), args(l)),
                Item::VTable(t, tr) => format!("vtable {} as {}", tcx.display(r, t), r.def_path(tr)),
            })
            .collect();
        lines.sort();
        let mut out = String::new();
        for l in lines {
            let _ = writeln!(out, "{l}");
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use crate::session::{CrateRoot, Options, Session};

    /// `core` from the toolchain's rust-src, if it is installed.
    fn core_src() -> Option<std::path::PathBuf> {
        let out = std::process::Command::new("rustc").args(["--print", "sysroot"]).output().ok()?;
        let root = std::path::PathBuf::from(String::from_utf8(out.stdout).ok()?.trim());
        let core = root.join("lib/rustlib/src/rust/library/core/src/lib.rs");
        core.exists().then_some(core)
    }

    #[test]
    fn program_items() {
        let Some(core) = core_src() else {
            eprintln!("no rust-src: skipped");
            return;
        };
        let file = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/mono/program.rs");
        let session = Session::new(Options { threads: 4, cfg: Some(crate::cfg::CfgSet::host()), ..Default::default() });
        let mut crates = session.parse_crates(vec![CrateRoot::new("core", core), CrateRoot::new("program", &file)]);
        session.expand(&mut crates);
        let r = session.resolve(&crates);
        let tcx = session.collect_types(&crates, &r);
        let index = crate::typeck::Index::new(&tcx, &r);
        let res = session.typeck_with(&crates, &r, &tcx, &index);
        let roots = super::roots(&r, &tcx, 1);
        let mono = session.collect_mono_items(&crates, &r, &tcx, &index, &res, roots);
        let dump = mono.dump(&r, &tcx);
        assert!(mono.unresolved.is_empty(), "not resolved: {:?}", mono.unresolved);
        for want in [
            // Generic functions at their arguments, and trait methods
            // resolved to the impl (or the trait's default body).
            "fn program::total<Square>",
            "fn program::Area::twice<Rect>",
            "fn <Square as program::Area>::area",
            "fn <u64 as core::convert::From>::from",
            "fn core::cmp::Ord::max<u64>",
            // `?` through `Try` and `FromResidual`.
            "fn <Option<T> as core::ops::try_trait::Try>::branch<u32>",
            "fn <Option<T> as core::ops::try_trait::FromResidual>::from_residual<u32>",
            // `for` through `IntoIterator` and `Iterator::next`.
            "fn <Iter<T> as core::iter::traits::iterator::Iterator>::next<&dyn Area>",
            // Iterators in core, with the closure `map_fold` returns
            // (`impl FnMut`) seen through.
            "fn <Iter<T> as core::iter::traits::iterator::Iterator>::fold<Square, u64, {closure}(u64, &Square) -> u64>",
            // `impl Trait` as a parameter and as a result.
            "fn program::apply<{closure}(u64) -> u64>",
            "fn program::adder",
            // Unsizing to `dyn Area` needs each type's vtable.
            "vtable Rect as program::Area",
            "vtable Square as program::Area",
        ] {
            assert!(dump.lines().any(|l| l == want), "no `{want}` in:\n{dump}");
        }
        assert!(
            dump.lines().any(|l| l.starts_with("closure @") && l.ends_with("in program::adder")),
            "no closure of adder in:\n{dump}"
        );
    }
}
