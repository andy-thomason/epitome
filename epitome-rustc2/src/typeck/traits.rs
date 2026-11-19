//! Traits in a body: which impl (or bound) makes a type implement a
//! trait, what an associated type is for a type, and what is built on
//! those (trait methods, operators, `Deref`, `Fn`, `IntoIterator`, `?`).
//!
//! Selection looks, in order, at:
//!
//! - what the body may assume: its item's (and parents') `where` clauses,
//!   with supertraits added, and the bounds of a `dyn` or `impl Trait`;
//! - obligations made while checking (a generic function's bounds on the
//!   variables standing for its parameters, an `impl Trait` result's
//!   bounds), which only say something about that same variable;
//! - closures, function items and pointers for the `Fn` traits, and
//!   `Sized` for everything;
//! - impls of the trait, found by the head of the type (and blanket impls),
//!   each tried in a snapshot, with its own `where` clauses checked a few
//!   levels deep.
//!
//! One impl matching is a result; several is ambiguous, which is not an
//! error: the variables may be settled later, and anything waiting on an
//! associated type is kept as a pending obligation, tried again later.

use super::check::{FnCtxt, Head, head};
use super::Adjust;
use super::infer::VarKind;
use crate::ast::{Kind, NodeId};
use crate::hash::FastMap;
use crate::resolve::{DefId, DefKind, Ns, Res};
use crate::ty::collect::subst;
use crate::ty::{Mutbl, Pred, TraitRef, Ty, TyKind};

/// A trait's impls, found by the head of their self type; `blanket` are
/// those whose self type is a parameter (or has no head).
#[derive(Default)]
pub struct TraitImpls {
    pub by_head: FastMap<Head, Vec<usize>>,
    pub blanket: Vec<usize>,
    pub all: Vec<usize>,
}

/// What makes a type implement a trait.
#[derive(Clone, Debug)]
pub(super) enum Source {
    /// An impl, with its arguments.
    Impl(usize, Vec<Ty>),
    /// Something the body may assume (or an obligation on a variable).
    Bound,
    /// Built in; for the `Fn` traits, the result type, and for `Future`
    /// (of an `async` block) the output.
    Builtin(Option<Ty>),
}

#[derive(Clone, Debug)]
pub(super) enum Select {
    No,
    Yes(Source),
    Ambiguous,
}

impl Select {
    pub(super) fn holds(&self) -> bool {
        !matches!(self, Select::No)
    }
}

/// How deep impls' `where` clauses are checked, and projections followed.
const MAX_DEPTH: u32 = 4;

/// How deep impls' `where` clauses are checked when a projection's impl is
/// ambiguous at a body's usual depth.
const DEEP_WHERE: u32 = 4;

impl<'a, 's> FnCtxt<'a, 's> {
    // ---- the predicates a body starts with ----

    /// What the body of `owner` may assume: its and its parents'
    /// predicates, `Self: Trait` in a trait, and all their supertraits.
    pub(super) fn env_predicates(&mut self) {
        let tcx = self.tcx;
        let mut preds = Vec::new();
        let mut o = Some(self.owner);
        while let Some(d) = o {
            preds.extend(tcx.predicates[d.0 as usize].iter().cloned());
            if self.r.defs[d.0 as usize].kind == DefKind::Trait {
                let params = tcx.all_params(d);
                if let Some((&s, rest)) = params.split_first() {
                    let self_ty = self.mk(TyKind::Param(s));
                    let args: Vec<Ty> = rest.iter().map(|&p| self.mk(TyKind::Param(p))).collect();
                    preds.push(Pred::Trait(self_ty, TraitRef { def: d, args: tcx.list(args), bindings: Vec::new(), assoc_bounds: Vec::new() }));
                }
            }
            o = tcx.generics[d.0 as usize].parent;
        }
        let mut out = Vec::new();
        for p in preds {
            match p {
                Pred::Trait(t, tr) => self.elaborate(t, &tr, &mut out),
                p => {
                    if !out.contains(&p) {
                        out.push(p)
                    }
                }
            }
        }
        self.env_preds = out;
        // `where <S as Identity<Q>>::T: Clone` is `S: Clone` when the
        // projection can be worked out: that too.
        let n = self.env_preds.len();
        for i in 0..n {
            let Pred::Trait(t, tr) = self.env_preds[i].clone() else { continue };
            if !matches!(self.tcx.kind(t), TyKind::Projection(..)) {
                continue;
            }
            let nt = self.normalize(t, 1);
            let nt = self.infer.resolve(nt);
            if nt != t && !self.tcx.has_infer(nt) && !matches!(self.tcx.kind(nt), TyKind::Projection(..)) {
                let mut more = Vec::new();
                self.elaborate(nt, &tr, &mut more);
                for p in more {
                    if !self.env_preds.contains(&p) {
                        self.env_preds.push(p);
                    }
                }
            }
        }
    }

    /// `t: tr`, its bindings as projections, and the same for each
    /// supertrait, into `out` (once each).
    pub(super) fn elaborate(&self, t: Ty, tr: &TraitRef, out: &mut Vec<Pred>) {
        let tcx = self.tcx;
        let p = Pred::Trait(t, tr.clone());
        if out.contains(&p) || out.len() > 256 {
            return;
        }
        out.push(p);
        for &(assoc, b) in &tr.bindings {
            let mut args = vec![t];
            args.extend(tcx.tys(tr.args));
            out.push(Pred::Projection(t, assoc, tcx.list(args), b));
        }
        for (assoc, b) in &tr.assoc_bounds {
            let mut args = vec![t];
            args.extend(tcx.tys(tr.args));
            let p = self.mk(TyKind::Projection(*assoc, tcx.list(args)));
            self.elaborate(p, b, out);
        }
        let Some(def) = tcx.traits.get(&tr.def) else { return };
        let params = tcx.all_params(tr.def);
        let mut args = vec![t];
        args.extend(tcx.tys(tr.args));
        // Arguments not written take their defaults (`T: BytewiseEq` is
        // `BytewiseEq<T>`, so its supertrait `PartialEq<Rhs>` is
        // `PartialEq<T>`).
        let defaults = &tcx.generics[tr.def.0 as usize].defaults;
        while args.len() < params.len() {
            let Some(d) = defaults.get(args.len()).copied().flatten() else { break };
            let a = subst(tcx, d, &params[..args.len()], &args);
            args.push(a);
        }
        for sup in &def.supertraits {
            let sup = subst_trait_ref(tcx, sup, &params, &args);
            self.elaborate(t, &sup, out);
        }
    }

    /// The predicates a `dyn` or `impl Trait` type `t` brings.
    fn type_bounds(&self, t: Ty) -> Vec<Pred> {
        let tcx = self.tcx;
        let mut out = Vec::new();
        match tcx.kind(t) {
            TyKind::Dyn(d) => {
                for tr in tcx.dyn_of(d).traits {
                    self.elaborate(t, &tr, &mut out);
                }
            }
            TyKind::Opaque(o, l) => {
                let def = tcx.opaque(o);
                let params = tcx.all_params(def.owner);
                let args = tcx.tys(l);
                for tr in def.bounds {
                    let tr = subst_trait_ref(tcx, &tr, &params, &args);
                    self.elaborate(t, &tr, &mut out);
                }
            }
            // `<T as Trait>::A`: what the trait declares of `A`.
            TyKind::Projection(a, l) => {
                let params = tcx.all_params(a);
                let args = tcx.tys(l);
                for p in &tcx.predicates[a.0 as usize] {
                    match p {
                        Pred::Trait(pt, tr) if args.len() >= params.len() || params.len() == args.len() + tcx.generics[a.0 as usize].own.len() => {
                            let pt = subst(tcx, *pt, &params, &args);
                            if pt == t {
                                let tr = subst_trait_ref(tcx, tr, &params, &args);
                                self.elaborate(t, &tr, &mut out);
                            }
                        }
                        _ => {}
                    }
                }
                // And the `where` clauses on it of the trait it, or a
                // projection it is of, belongs to (`where Self::Iter:
                // Iterator, <Self::Iter as Iterator>::Item: Bar`), which
                // hold as its bounds do.
                let mut cur = t;
                while let TyKind::Projection(a, l) = tcx.kind(cur) {
                    let args = tcx.tys(l);
                    if let Some(tr) = self.trait_of(a) {
                        let tparams = tcx.all_params(tr);
                        if args.len() >= tparams.len() {
                            let targs = &args[..tparams.len()];
                            for p in &tcx.predicates[tr.0 as usize] {
                                if let Pred::Trait(pt, ptr) = p
                                    && subst(tcx, *pt, &tparams, targs) == t
                                {
                                    let ptr = subst_trait_ref(tcx, ptr, &tparams, targs);
                                    self.elaborate(t, &ptr, &mut out);
                                }
                            }
                        }
                    }
                    let Some(&base) = args.first() else { break };
                    cur = base;
                }
            }
            _ => {}
        }
        out
    }

    /// `pred`, from a generic item used at arguments that are variables:
    /// kept to say what those variables must be.
    pub(super) fn oblige(&mut self, pred: Pred) {
        let self_ty = match &pred {
            Pred::Trait(t, _) | Pred::Projection(t, ..) => *t,
        };
        // Checked against impls once known: the arguments' unknowns, or
        // the type's (an opaque type's hidden type, bound by its bounds).
        if let Pred::Trait(t, tr) = &pred
            && self.pending_traits.len() < 4096
            && (self.tcx.tys(tr.args).into_iter().any(|a| self.tcx.has_infer(self.infer.resolve(a)))
                || self.infer.var_kind(self.infer.shallow(*t)) == Some(VarKind::General))
        {
            self.pending_traits.push((*t, tr.def, tr.args));
        }
        // `S: R<E = A>` with `S` known and `A` not: `A` is `S`'s `E`.
        if let Pred::Projection(_, a, l, t) = &pred
            && !self.tcx.has_infer(self.infer.resolve(self_ty))
            && self.tcx.has_infer(self.infer.resolve(*t))
        {
            let args = self.tcx.tys(*l);
            let v = self.project_or_var(*a, &args);
            self.infer.unify(v, *t);
            return;
        }
        if !self.tcx.has_infer(self.infer.resolve(self_ty)) || self.obligations.len() > 4096 {
            return;
        }
        if let Pred::Projection(_, a, l, t) = &pred {
            self.pending.push((*a, *l, *t));
        }
        self.obligations.push(pred);
    }

    /// The predicates of item `d` (and its parents) at arguments `args`,
    /// as obligations.
    pub(super) fn oblige_item(&mut self, d: DefId, args: &[Ty]) {
        let tcx = self.tcx;
        let params = tcx.all_params(d);
        let mut o = Some(d);
        while let Some(x) = o {
            // An inherent impl's `where` clauses are its methods' too
            // (`impl<T, F: FnOnce(T)> DropGuard<T, F>`: what `new`'s closure
            // takes); a trait's or a trait impl's are not obligations here.
            let kind = self.r.defs[x.0 as usize].kind;
            let inherent = kind == DefKind::Impl && tcx.impl_index.get(&x).is_some_and(|&i| tcx.impls[i].trait_ref.is_none());
            if matches!(kind, DefKind::Impl | DefKind::Trait) && !inherent {
                break;
            }
            for p in tcx.predicates[x.0 as usize].clone() {
                // Substituted, with projections in them normalised (or
                // variables standing for them), so that a closure checked
                // against them sees through `<Self as Iterator>::Item`.
                let sub = |cx: &mut Self, t: Ty| {
                    let t = subst(tcx, t, &params, args);
                    cx.normalize(t, 0)
                };
                let p = match p {
                    Pred::Trait(t, tr) => {
                        let tr = subst_trait_ref(tcx, &tr, &params, args);
                        let targs: Vec<Ty> = tcx.tys(tr.args).into_iter().map(|x| self.normalize(x, 0)).collect();
                        let bindings = tr.bindings.iter().map(|&(d, b)| (d, self.normalize(b, 0))).collect();
                        let tr = TraitRef { args: tcx.list(targs), bindings, ..tr };
                        Pred::Trait(sub(self, t), tr)
                    }
                    Pred::Projection(t, a, l, b) => {
                        let l2: Vec<Ty> = tcx.tys(l).into_iter().map(|x| sub(self, x)).collect();
                        Pred::Projection(sub(self, t), a, tcx.list(l2), sub(self, b))
                    }
                };
                // Unknowns in it (`C<?N>: Foo<3>`): the one impl that fits
                // may settle them.
                if let Pred::Trait(t, tr) = &p
                    && Some(&tr.def) != tcx.lang.get("sized")
                    && (tcx.has_infer(self.infer.resolve(*t))
                        || tcx.tys(tr.args).iter().any(|&a| tcx.has_infer(self.infer.resolve(a))))
                {
                    self.pending_traits.push((*t, tr.def, tr.args));
                }
                self.oblige(p);
            }
            o = tcx.generics[x.0 as usize].parent;
        }
    }

    /// A snapshot of inference and of the obligations.
    pub(super) fn probe(&self) -> (super::infer::Snapshot, usize, usize, usize) {
        (self.infer.snapshot(), self.obligations.len(), self.pending.len(), self.pending_traits.len())
    }

    pub(super) fn rollback(&mut self, s: (super::infer::Snapshot, usize, usize, usize)) {
        self.infer.rollback(s.0);
        self.obligations.truncate(s.1);
        self.pending.truncate(s.2);
        self.pending_traits.truncate(s.3);
    }

    /// Whether a predicate's self type `pt` is `t`: by unifying, unless
    /// `pt` is still a variable (an obligation), which only matches itself.
    fn self_matches(&mut self, pt: Ty, t: Ty) -> bool {
        let ps = self.infer.shallow(pt);
        if self.infer.var_kind(ps).is_some() || self.infer.var_kind(t).is_some() {
            return ps == t;
        }
        self.infer.unify(ps, t)
    }

    fn args_unify(&mut self, a: &[Ty], b: &[Ty]) -> bool {
        a.len() == b.len() && a.iter().zip(b).all(|(&x, &y)| self.infer.unify(x, y))
    }

    /// Whether `t: tr<targs>` is something the body may assume (or an
    /// obligation on `t`), unifying `targs` with it if so.
    fn assumed(&mut self, t: Ty, tr: DefId, targs: &[Ty]) -> bool {
        let tcx = self.tcx;
        let bounds = self.type_bounds(t);
        let n = self.env_preds.len() + self.obligations.len();
        for i in 0..n + bounds.len() {
            let p = if i < self.env_preds.len() {
                &self.env_preds[i]
            } else if i < n {
                &self.obligations[i - self.env_preds.len()]
            } else {
                &bounds[i - n]
            };
            let Pred::Trait(pt, ptr) = p else { continue };
            if ptr.def != tr {
                continue;
            }
            let (pt, pargs) = (*pt, ptr.args);
            // An obligation says something only while its type is a
            // variable; once known, the impl that satisfies it is needed.
            let obligation = i >= self.env_preds.len() && i < n;
            if obligation && self.infer.var_kind(self.infer.shallow(pt)).is_none() {
                continue;
            }
            let s = self.probe();
            let pargs = self.full_trait_args(tr, pt, tcx.tys(pargs));
            if self.self_matches(pt, t) && self.args_unify(&pargs, targs) {
                return true;
            }
            self.rollback(s);
            // A supertrait's arguments may name the bound's own
            // projections (`Module: Mul<<Self as Module>::Ring>` with
            // `Ring = R`): worked out, shallowly.
            if pargs.iter().any(|&x| has_projection(tcx, x)) {
                let pargs: Vec<Ty> = pargs.iter().map(|&x| self.normalize(x, MAX_DEPTH - 1)).collect();
                if self.self_matches(pt, t) && self.args_unify(&pargs, targs) {
                    return true;
                }
                self.rollback(s);
            }
        }
        false
    }

    // ---- selection ----

    /// Traits the compiler implements itself (or that hold for every type
    /// as far as type checking cares): `Sized` and its kin, `Tuple` for
    /// tuples, auto traits, and `Copy`/`Clone` for closures and function
    /// items and pointers.
    fn is_builtin_marker(&self, tr: DefId, t: Ty) -> bool {
        let tcx = self.tcx;
        if tcx.traits.get(&tr).is_some_and(|d| d.is_auto) {
            return true;
        }
        let is = |name: &str| tcx.lang.get(name) == Some(&tr);
        if ["sized", "meta_sized", "pointee_sized", "destruct", "freeze", "unsize", "discriminant_kind", "pointee_trait", "transmute_trait", "bikeshed_guaranteed_no_drop"]
            .iter()
            .any(|n| is(n))
        {
            return true;
        }
        match tcx.kind(t) {
            TyKind::Tuple(_) if is("tuple_trait") => true,
            TyKind::FnPtr(_) if is("fn_ptr_trait") => true,
            TyKind::Closure(..) | TyKind::FnDef(..) | TyKind::FnPtr(_) | TyKind::Never => is("copy") || is("clone"),
            _ => false,
        }
    }

    /// Whether, and how, `t: tr<targs>`.
    pub(super) fn select(&mut self, t: Ty, tr: DefId, targs: &[Ty], depth: u32) -> Select {
        let tcx = self.tcx;
        let mut t = self.infer.shallow(t);
        // `<Map<I, F> as Iterator>::Item: Tr`: what the projection is.
        if let TyKind::Projection(..) = tcx.kind(t)
            && depth < MAX_DEPTH
        {
            let n = self.normalize(t, depth + 1);
            t = self.infer.shallow(n);
        }
        if depth > MAX_DEPTH || tcx.kind(t) == TyKind::Error {
            return Select::Ambiguous;
        }
        if self.assumed(t, tr, targs) {
            return Select::Yes(Source::Bound);
        }
        let var = self.infer.var_kind(t);
        if var == Some(VarKind::General) {
            return Select::Ambiguous;
        }
        if self.is_builtin_marker(tr, t) {
            return Select::Yes(Source::Builtin(None));
        }
        // Tuples are `Clone` and `Copy` when their elements are, built in.
        if let TyKind::Tuple(l) = tcx.kind(t)
            && ["clone", "copy"].iter().any(|n| tcx.lang.get(n) == Some(&tr))
        {
            let mut ambiguous = false;
            for e in tcx.tys(l) {
                match self.select(e, tr, &[], depth + 1) {
                    Select::Yes(_) => {}
                    Select::No => return Select::No,
                    Select::Ambiguous => ambiguous = true,
                }
            }
            return if ambiguous { Select::Ambiguous } else { Select::Yes(Source::Builtin(None)) };
        }
        // Closures, function items and pointers implement the `Fn` traits.
        if ["fn", "fn_mut", "fn_once"].iter().any(|l| tcx.lang.get(l) == Some(&tr)) {
            let sig = match tcx.kind(t) {
                TyKind::FnDef(d, l) => self.callable_sig(d, l),
                TyKind::FnPtr(s) | TyKind::Closure(_, s, _) => {
                    let s = tcx.sig_of(s);
                    Some((tcx.tys(s.inputs), s.output))
                }
                _ => None,
            };
            if let Some((inputs, output)) = sig {
                let tuple = self.mk(TyKind::Tuple(tcx.list(inputs)));
                return match targs.first() {
                    Some(&a) if self.infer.unify(a, tuple) => Select::Yes(Source::Builtin(Some(output))),
                    _ => Select::No,
                };
            }
        }
        // Closures, function items and pointers whose result is a future
        // implement the `AsyncFn` traits, with that future as what a call
        // gives (`async || {}`).
        if ["async_fn", "async_fn_mut", "async_fn_once"].iter().any(|l| tcx.lang.get(l) == Some(&tr)) {
            let sig = match tcx.kind(t) {
                TyKind::FnDef(d, l) => self.callable_sig(d, l),
                TyKind::FnPtr(s) | TyKind::Closure(_, s, _) => {
                    let s = tcx.sig_of(s);
                    Some((tcx.tys(s.inputs), s.output))
                }
                _ => None,
            };
            if let Some((inputs, output)) = sig {
                let fut = self.lang("future_trait");
                let is_future = fut.is_some_and(|f| self.select(self.infer.shallow(output), f, &[], depth + 1).holds());
                let tuple = self.mk(TyKind::Tuple(tcx.list(inputs)));
                return match targs.first() {
                    Some(&a) if is_future && self.infer.unify(a, tuple) => Select::Yes(Source::Builtin(Some(output))),
                    _ => Select::No,
                };
            }
        }
        // Coroutines implement `Coroutine<R>` for their resume type `R`.
        if let TyKind::Coroutine(_, parts, _) = tcx.kind(t)
            && tcx.lang.get("coroutine") == Some(&tr)
        {
            let resume = tcx.tys(match tcx.kind(parts) {
                TyKind::Tuple(l) => l,
                _ => return Select::No,
            })[0];
            let r = targs.first().copied().unwrap_or_else(|| self.unit());
            return if self.infer.unify(r, resume) { Select::Yes(Source::Builtin(None)) } else { Select::No };
        }
        // `gen` blocks implement `Iterator`, `async gen` ones
        // `AsyncIterator`, their `Item` the type they yield.
        if let TyKind::GenBlock(_, _, _, a) = tcx.kind(t)
            && tcx.lang.get(if a { "async_iterator" } else { "iterator" }) == Some(&tr)
        {
            return Select::Yes(Source::Builtin(None));
        }
        // `async` blocks implement `Future`, with their value as `Output`.
        if let TyKind::AsyncBlock(_, out, _) = tcx.kind(t)
            && tcx.lang.get("future_trait") == Some(&tr)
        {
            return Select::Yes(Source::Builtin(Some(out)));
        }
        let Some(ti) = self.index.trait_impls.get(&tr) else { return Select::No };
        let cands: Vec<usize> = match head(tcx, t) {
            Some(h) => ti.by_head.get(&h).into_iter().flatten().chain(&ti.blanket).copied().collect(),
            None if var.is_some() => ti.all.clone(),
            None => ti.blanket.clone(),
        };
        let mut found: Option<usize> = None;
        for i in cands {
            let s = self.probe();
            let ok = self.impl_matches(i, t, targs, depth).is_some();
            self.rollback(s);
            if ok {
                if found.is_some() {
                    return Select::Ambiguous;
                }
                found = Some(i);
            }
        }
        match found {
            Some(i) => match self.impl_matches(i, t, targs, depth) {
                Some(args) => Select::Yes(Source::Impl(i, args)),
                None => Select::No,
            },
            None => Select::No,
        }
    }

    /// Where several impls match `t: tr<args>` (a specialising family, such
    /// as `SpecExtend`'s), but every one that does gives the variables in
    /// `args` the same types: those types. Whether it settled them.
    fn settle_by_agreement(&mut self, t: Ty, tr: DefId, args: &[Ty]) -> bool {
        let tcx = self.tcx;
        let t = self.infer.shallow(t);
        let Some(ti) = self.index.trait_impls.get(&tr) else { return false };
        let cands: Vec<usize> = match head(tcx, t) {
            Some(h) => ti.by_head.get(&h).into_iter().flatten().chain(&ti.blanket).copied().collect(),
            None => return false,
        };
        let mut agreed: Option<Vec<Ty>> = None;
        for i in cands {
            let s = self.probe();
            let got = self.impl_matches(i, t, args, 1).map(|_| args.iter().map(|&a| self.infer.resolve(a)).collect::<Vec<Ty>>());
            self.rollback(s);
            let Some(got) = got else { continue };
            if got.iter().any(|&a| tcx.has_infer(a)) {
                return false;
            }
            match &agreed {
                None => agreed = Some(got),
                Some(a) if *a == got => {}
                Some(_) => return false,
            }
        }
        let Some(agreed) = agreed else { return false };
        args.iter().zip(agreed).all(|(&a, b)| self.infer.unify(a, b))
    }

    /// For a concrete `t` that several impls match (specialisation): the
    /// one that is more specific than each of the others, which match its
    /// header. Its arguments, as `select` gives them. The impls in `skip`
    /// are left out (to find the one an item not in them comes from).
    pub(super) fn select_specific(&mut self, t: Ty, tr: DefId, targs: &[Ty], skip: &[usize]) -> Option<(usize, Vec<Ty>)> {
        let tcx = self.tcx;
        let ti = self.index.trait_impls.get(&tr)?;
        let cands: Vec<usize> = match head(tcx, t) {
            Some(h) => ti.by_head.get(&h).into_iter().flatten().chain(&ti.blanket).copied().collect(),
            None => ti.blanket.clone(),
        };
        let matching: Vec<usize> = cands
            .into_iter()
            .filter(|&i| {
                if skip.contains(&i) {
                    return false;
                }
                let s = self.probe();
                let ok = self.impl_matches(i, t, targs, 0).is_some();
                self.rollback(s);
                ok
            })
            .collect();
        let best: Vec<usize> = matching
            .iter()
            .copied()
            .filter(|&i| {
                // Each other impl matches impl `i`'s header, its parameters
                // left as they are and its `where` clauses assumed.
                let imp = &tcx.impls[i];
                let Some(itr) = imp.trait_ref.as_ref() else { return false };
                let iargs = tcx.tys(itr.args);
                let mut env = Vec::new();
                for p in &tcx.predicates[imp.def.0 as usize] {
                    match p {
                        Pred::Trait(pt, ptr) => self.elaborate(*pt, ptr, &mut env),
                        _ => env.push(p.clone()),
                    }
                }
                let saved = std::mem::replace(&mut self.env_preds, env);
                let ok = matching.iter().all(|&j| {
                    if j == i {
                        return true;
                    }
                    let s = self.probe();
                    let ok = self.impl_matches(j, imp.self_ty, &iargs, 0).is_some();
                    self.rollback(s);
                    ok
                });
                self.env_preds = saved;
                ok
            })
            .collect();
        // Impls whose headers match each other may differ only in `Sized`
        // (`impl<W: Write + ?Sized> S for &mut W` and `impl<W: Write> S for
        // &mut W`): the one that does not relax it is the more specific, by
        // the implicit `Sized` bounds the predicates do not record.
        let fewest = best.iter().map(|&i| self.relaxes_sized(i)).min();
        let best: Vec<usize> = best.into_iter().filter(|&i| Some(self.relaxes_sized(i)) == fewest).collect();
        let [i] = best[..] else { return None };
        let args = self.impl_matches(i, t, targs, 0)?;
        Some((i, args))
    }

    /// How many `?Sized` bounds impl `i` writes, in its generics and
    /// `where` clause.
    fn relaxes_sized(&self, i: usize) -> usize {
        let def = &self.r.defs[self.tcx.impls[i].def.0 as usize];
        let ast = &self.env.crates[def.krate as usize].files[def.file as usize].ast;
        let mut n = 0;
        let mut stack: Vec<NodeId> = ast
            .children(def.node)
            .iter()
            .copied()
            .filter(|&c| matches!(ast.kind(c), Kind::Generics | Kind::WhereClause))
            .collect();
        while let Some(c) = stack.pop() {
            if ast.kind(c) == Kind::TraitBound && ast.children(c).first().is_some_and(|&o| ast.kind(o) == Kind::Op && ast.token(o).0 == "?") {
                n += 1;
            }
            stack.extend(ast.children(c).iter().copied());
        }
        n
    }

    /// A trait's arguments (after `Self`) with defaults filled in for those
    /// not written (`Add` is `Add<Self>`), or variables if none.
    pub(super) fn full_trait_args(&mut self, tr: DefId, self_ty: Ty, mut args: Vec<Ty>) -> Vec<Ty> {
        let tcx = self.tcx;
        let g = &tcx.generics[tr.0 as usize];
        let n = g.own.len().saturating_sub(1);
        if args.len() >= n {
            return args;
        }
        let params = tcx.all_params(tr);
        for i in args.len()..n {
            let t = match g.defaults.get(i + 1).copied().flatten() {
                Some(d) => {
                    let mut all = vec![self_ty];
                    all.extend(args.iter().copied());
                    while all.len() < params.len() {
                        all.push(self.infer.fresh());
                    }
                    subst(tcx, d, &params, &all)
                }
                None => self.infer.fresh(),
            };
            args.push(t);
        }
        args
    }

    /// Whether impl `i` makes `t: tr<targs>`: its arguments if so. Binds
    /// variables; the caller probes.
    fn impl_matches(&mut self, i: usize, t: Ty, targs: &[Ty], depth: u32) -> Option<Vec<Ty>> {
        let tcx = self.tcx;
        let imp = &tcx.impls[i];
        let params = tcx.all_params(imp.def);
        let preds = expand_assoc_bounds(tcx, &tcx.predicates[imp.def.0 as usize]);
        let args = self.infer.instantiate(&params);
        let mut st = subst(tcx, imp.self_ty, &params, &args);
        // `(C, T, <C as Trait<T>>::Assoc)`: the plain parts first, which
        // say what the projections' are.
        let parts = match (tcx.kind(st), tcx.kind(self.infer.shallow(t))) {
            (TyKind::Tuple(x), TyKind::Tuple(y)) => Some((x, y)),
            (TyKind::Adt(a, x), TyKind::Adt(b, y)) if a == b => Some((x, y)),
            _ => None,
        };
        if has_projection(tcx, st)
            && let Some((x, y)) = parts
            && tcx.tys(x).len() == tcx.tys(y).len()
        {
            let (xs, ys) = (tcx.tys(x), tcx.tys(y));
            for (&a, &b) in xs.iter().zip(&ys) {
                if !has_projection(tcx, a) && !self.infer.unify(a, b) {
                    return None;
                }
            }
        }
        // A self type through a projection (`Matrix<R, <A as Alloc<R>>::
        // Buffer>`): worked out where it can be.
        if has_projection(tcx, st) && depth < MAX_DEPTH {
            st = self.normalize(st, depth + 1);
        }
        if !self.infer.unify(st, t) {
            return None;
        }
        let tr = imp.trait_ref.as_ref()?;
        let iargs: Vec<Ty> = tcx.tys(tr.args).into_iter().map(|x| subst(tcx, x, &params, &args)).collect();
        let iargs = self.full_trait_args(tr.def, st, iargs);
        // A default may be a projection (`FromResidual<R = <Self as
        // Try>::Residual>`).
        let iargs: Vec<Ty> = iargs.into_iter().map(|x| self.normalize(x, depth + 1)).collect();
        if !self.args_unify(&iargs, targs) {
            return None;
        }
        // `where I: Iterator<Item = &'a T>`: what settles `T`; deeper than
        // the other `where` clauses are checked, as a parameter only these
        // name (`B` of `Map<I, F>`'s `F: FnMut(I::Item) -> B`) would be
        // left unknown.
        if depth < MAX_DEPTH {
            for p in &preds {
                if let Pred::Projection(_, a, l, ty) = p {
                    let pl: Vec<Ty> = tcx.tys(*l).into_iter().map(|x| subst(tcx, x, &params, &args)).collect();
                    let pl: Vec<Ty> = pl.into_iter().map(|x| self.normalize(x, depth + 1)).collect();
                    let ty = subst(tcx, *ty, &params, &args);
                    if let Some(got) = self.project(*a, &pl, depth + 1)
                        && !self.infer.unify(got, ty)
                    {
                        // `Item = U::Item`: `U` may be known by now; while
                        // it cannot be worked out it says nothing.
                        let n = self.normalize(ty, depth + 1);
                        if !has_projection(tcx, self.infer.resolve(n)) && !self.infer.unify(got, n) {
                            return None;
                        }
                    }
                }
            }
        }
        if depth < self.where_depth {
            for p in &preds {
                if let Pred::Trait(pt, ptr) = p {
                    if Some(&ptr.def) == tcx.lang.get("sized") {
                        continue;
                    }
                    let pt = subst(tcx, *pt, &params, &args);
                    let pt = self.normalize(pt, depth + 1);
                    let pa: Vec<Ty> = tcx.tys(ptr.args).into_iter().map(|x| subst(tcx, x, &params, &args)).collect();
                    let pa: Vec<Ty> = pa.into_iter().map(|x| self.normalize(x, depth + 1)).collect();
                    // `T: PartialEq` is `T: PartialEq<T>`.
                    let pa = self.full_trait_args(ptr.def, pt, pa);
                    if !self.select(pt, ptr.def, &pa, depth + 1).holds() {
                        return None;
                    }
                }
            }
        }
        Some(args)
    }

    // ---- associated types ----

    /// The trait an associated item belongs to.
    pub(super) fn trait_of(&self, item: DefId) -> Option<DefId> {
        let m = self.r.defs[item.0 as usize].parent?;
        let d = self.r.modules[m.0 as usize].def?;
        (self.r.defs[d.0 as usize].kind == DefKind::Trait).then_some(d)
    }

    /// Trait `tr`'s associated item `name` (in `ns`).
    pub(super) fn trait_item(&self, tr: DefId, name: &str, ns: Ns) -> Option<DefId> {
        let m = self.r.defs[tr.0 as usize].module?;
        match self.r.modules[m.0 as usize].names.get(&(name, ns)).map(|b| &b.res) {
            Some(Res::Def(d)) => Some(*d),
            _ => None,
        }
    }

    /// Impl `i`'s item `name` (in `ns`).
    fn impl_item(&self, i: usize, name: &str, ns: Ns) -> Option<DefId> {
        let m = self.r.defs[self.tcx.impls[i].def.0 as usize].module?;
        match self.r.modules[m.0 as usize].names.get(&(name, ns)).map(|b| &b.res) {
            Some(Res::Def(d)) => Some(*d),
            _ => None,
        }
    }

    /// `<args[0] as Trait<args[1..]>>::assoc` if it can be worked out now.
    pub(super) fn project(&mut self, assoc: DefId, args: &[Ty], depth: u32) -> Option<Ty> {
        let tcx = self.tcx;
        if depth > MAX_DEPTH || args.is_empty() {
            return None;
        }
        let t = self.infer.shallow(args[0]);
        // What the body assumes, obligations, and `dyn`/`impl` bindings.
        let bounds = self.type_bounds(t);
        let n = self.env_preds.len() + self.obligations.len();
        for i in 0..n + bounds.len() {
            let p = if i < self.env_preds.len() {
                &self.env_preds[i]
            } else if i < n {
                &self.obligations[i - self.env_preds.len()]
            } else {
                &bounds[i - n]
            };
            let Pred::Projection(pt, a, l, ty) = p else { continue };
            if *a != assoc {
                continue;
            }
            let (pt, l, ty) = (*pt, *l, *ty);
            let obligation = i >= self.env_preds.len() && i < n;
            if obligation && self.infer.var_kind(self.infer.shallow(pt)).is_none() {
                continue;
            }
            let s = self.probe();
            let pl = tcx.tys(l);
            // A binding written without a generic associated type's own
            // arguments (`Tr<Gat<u8> = X>` keeps none): the trait's only.
            let (pl, al) = if pl.len() == args.len() {
                (&pl[1..], &args[1..])
            } else {
                let ntr = self.trait_of(assoc).map_or(1, |tr| tcx.all_params(tr).len());
                (&pl[1..ntr.min(pl.len()).max(1)], &args[1..ntr.min(args.len()).max(1)])
            };
            let (pa, aa) = match self.trait_of(assoc) {
                Some(tr) => (self.full_trait_args(tr, pt, pl.to_vec()), self.full_trait_args(tr, t, al.to_vec())),
                None => (pl.to_vec(), al.to_vec()),
            };
            if self.self_matches(pt, t) && self.args_unify(&pa, &aa) {
                return Some(ty);
            }
            self.rollback(s);
        }
        if tcx.lang.get("metadata_type") == Some(&assoc) {
            return self.pointee_metadata(t, 0);
        }
        // A `gen` block's `Item`.
        if let TyKind::GenBlock(_, y, _, a) = tcx.kind(t)
            && let Some(tr) = self.trait_of(assoc)
            && tcx.lang.get(if a { "async_iterator" } else { "iterator" }) == Some(&tr)
            && self.r.defs[assoc.0 as usize].name == "Item"
        {
            return Some(y);
        }
        // A coroutine's `Yield` and `Return`.
        if let TyKind::Coroutine(_, parts, _) = tcx.kind(t)
            && let TyKind::Tuple(l) = tcx.kind(parts)
        {
            if tcx.lang.get("coroutine_yield") == Some(&assoc) {
                return Some(tcx.tys(l)[1]);
            }
            if tcx.lang.get("coroutine_return") == Some(&assoc) {
                return Some(tcx.tys(l)[2]);
            }
        }
        if tcx.lang.get("discriminant_type") == Some(&assoc) {
            return self.discriminant_type(t);
        }
        let tr = self.trait_of(assoc)?;
        // A generic associated type's own arguments follow the trait's.
        let ntr = tcx.all_params(tr).len().min(args.len());
        let (args, own_args) = args.split_at(ntr);
        let mut sel = self.select(t, tr, &args[1..], depth);
        // An integer variable's own obligation (`{integer}: SliceIndex<[u8]>`)
        // says nothing of the impl: that is looked for.
        if matches!(sel, Select::Yes(Source::Bound))
            && matches!(self.infer.var_kind(t), Some(VarKind::Int | VarKind::Float))
        {
            let saved = std::mem::take(&mut self.obligations);
            sel = self.select(t, tr, &args[1..], depth);
            self.obligations = saved;
        }
        // Two impls that both seem to apply, with every type known: their
        // `where` clauses checked deeper may rule one out (`u8:
        // TryFrom<u16>` has core's impl, and the blanket one through `u16:
        // Into<u8>`, which needs `u8: From<u16>`, which does not hold).
        // With unknowns in them too, when the self type is known at the
        // top (`Vec<?X>: IntoIterator`, not the blanket impl for
        // iterators), if that leaves one.
        if matches!(sel, Select::Ambiguous) && self.where_depth < DEEP_WHERE {
            let known = !args.iter().any(|&x| tcx.has_infer(self.infer.resolve(x)));
            if known || self.infer.var_kind(t).is_none() {
                let s = self.probe();
                let saved = std::mem::replace(&mut self.where_depth, DEEP_WHERE);
                let deeper = self.select(t, tr, &args[1..], depth);
                self.where_depth = saved;
                match deeper {
                    Select::Yes(_) => sel = deeper,
                    _ if known => sel = deeper,
                    _ => self.rollback(s),
                }
            }
        }
        match sel {
            Select::Yes(Source::Impl(i, iargs)) => {
                let name = self.r.defs[assoc.0 as usize].name;
                let item = self.impl_item(i, name, Ns::Type)?;
                let it = tcx.type_of(item)?;
                let params = tcx.all_params(item);
                let mut a = iargs;
                if own_args.len() == tcx.generics[item.0 as usize].own.len() {
                    a.extend_from_slice(own_args);
                }
                while a.len() < params.len() {
                    a.push(self.infer.fresh());
                }
                let it = subst(tcx, it, &params, &a);
                Some(self.normalize(it, depth + 1))
            }
            Select::Yes(Source::Builtin(Some(out)))
                if ["fn_once_output", "future_output", "call_once_future", "call_ref_future"]
                    .iter()
                    .any(|l| tcx.lang.get(l) == Some(&assoc)) =>
            {
                Some(out)
            }
            // An async closure's `Output`: its future's.
            Select::Yes(Source::Builtin(Some(out))) if tcx.lang.get("async_fn_once_output") == Some(&assoc) => {
                self.await_output(out)
            }
            _ => None,
        }
    }

    /// `assoc` projected from `args`, or a variable standing for it until
    /// it can be (a pending obligation), or the projection itself when it
    /// names no variable (`<T as Iterator>::Item` in a generic body).
    pub(super) fn project_or_var(&mut self, assoc: DefId, args: &[Ty]) -> Ty {
        if let Some(t) = self.project(assoc, args, 0) {
            return t;
        }
        let tcx = self.tcx;
        let p = self.mk(TyKind::Projection(assoc, tcx.list(args.to_vec())));
        if !tcx.has_infer(self.infer.resolve(p)) {
            return p;
        }
        // Unknowns only in a generic associated type's own arguments
        // (`T::Assoc<_>`): a rigid projection, whose arguments unify.
        let ntr = self.trait_of(assoc).map_or(args.len(), |tr| tcx.all_params(tr).len());
        if ntr < args.len() && !args[..ntr].iter().any(|&a| tcx.has_infer(self.infer.resolve(a))) {
            return p;
        }
        let v = self.infer.fresh();
        self.pending.push((assoc, tcx.list(args.to_vec()), v));
        self.obligations.push(Pred::Projection(args[0], assoc, tcx.list(args.to_vec()), v));
        v
    }

    /// `t` with the projections in it worked out where they can be.
    /// `<t as DiscriminantKind>::Discriminant`, which the compiler
    /// implements: an enum's `repr` integer (`isize` without one), `u8`
    /// for anything else. `None` while `t` is not known.
    fn discriminant_type(&mut self, t: Ty) -> Option<Ty> {
        let tcx = self.tcx;
        let t = self.infer.shallow(t);
        let named = |name: &str| crate::consteval::prim_named(name).map(|k| tcx.mk(k));
        match tcx.kind(t) {
            TyKind::Param(_) | TyKind::Infer(_) | TyKind::Projection(..) | TyKind::Opaque(..) | TyKind::Error => None,
            TyKind::Adt(d, _) if tcx.adts.get(&d).is_some_and(|a| a.kind == crate::ty::AdtKind::Enum) => {
                let def = &self.r.defs[d.0 as usize];
                let ast = &self.env.crates[def.krate as usize].files[def.file as usize].ast;
                let repr = crate::session::attributes(ast, def.node)
                    .filter(|(n, _)| n == "repr")
                    .flat_map(|(_, a)| ast.tokens_of(a).iter().map(|t| t.0).collect::<Vec<_>>())
                    .find(|w| matches!(*w, "u8" | "u16" | "u32" | "u64" | "u128" | "usize" | "i8" | "i16" | "i32" | "i64" | "i128" | "isize"));
                named(repr.unwrap_or("isize"))
            }
            _ => named("u8"),
        }
    }

    /// `<t as Pointee>::Metadata`, which the compiler implements: `usize`
    /// for slices and `str`, `DynMetadata<t>` for `dyn Trait`, `()` for a
    /// sized type, and a struct's or tuple's last field's for one ending
    /// in an unsized field. `None` while `t` is not known well enough.
    fn pointee_metadata(&mut self, t: Ty, depth: u32) -> Option<Ty> {
        let tcx = self.tcx;
        if depth > 8 {
            return None;
        }
        let t = self.infer.shallow(t);
        match tcx.kind(t) {
            TyKind::Slice(_) | TyKind::Str => Some(self.mk(TyKind::Uint(crate::ty::UintTy::Usize))),
            TyKind::Dyn(_) => {
                let d = *tcx.lang.get("dyn_metadata")?;
                Some(self.mk(TyKind::Adt(d, tcx.list(vec![t]))))
            }
            TyKind::Param(_) | TyKind::Infer(_) | TyKind::Projection(..) | TyKind::Opaque(..) | TyKind::Error => None,
            TyKind::Adt(d, l) => {
                let adt = tcx.adts.get(&d)?;
                let last = match adt.kind {
                    crate::ty::AdtKind::Struct => adt.variants.first().and_then(|v| v.fields.last()),
                    _ => None,
                };
                match last {
                    Some(f) => {
                        let ft = subst(tcx, f.ty, &tcx.all_params(d), &tcx.tys(l));
                        let ft = self.normalize(ft, depth + 1);
                        self.pointee_metadata(ft, depth + 1)
                    }
                    None => Some(self.unit()),
                }
            }
            TyKind::Tuple(l) => match tcx.tys(l).last() {
                Some(&x) => self.pointee_metadata(x, depth + 1),
                None => Some(self.unit()),
            },
            _ => Some(self.unit()),
        }
    }

    pub(super) fn normalize(&mut self, t: Ty, depth: u32) -> Ty {
        let tcx = self.tcx;
        if !has_projection(tcx, t) {
            return t;
        }
        match tcx.kind(t) {
            TyKind::Projection(a, l) => {
                let args: Vec<Ty> = tcx.tys(l).into_iter().map(|x| self.normalize(x, depth)).collect();
                if depth <= MAX_DEPTH
                    && let Some(n) = self.project(a, &args, depth)
                {
                    return n;
                }
                if depth == 0 {
                    return self.project_or_var(a, &args);
                }
                self.mk(TyKind::Projection(a, tcx.list(args)))
            }
            TyKind::Tuple(l) => {
                let v = tcx.tys(l).into_iter().map(|x| self.normalize(x, depth)).collect();
                self.mk(TyKind::Tuple(tcx.list(v)))
            }
            TyKind::Adt(d, l) => {
                let v = tcx.tys(l).into_iter().map(|x| self.normalize(x, depth)).collect();
                self.mk(TyKind::Adt(d, tcx.list(v)))
            }
            TyKind::FnDef(d, l) => {
                let v = tcx.tys(l).into_iter().map(|x| self.normalize(x, depth)).collect();
                self.mk(TyKind::FnDef(d, tcx.list(v)))
            }
            TyKind::Ref(m, x) => {
                let x = self.normalize(x, depth);
                self.mk(TyKind::Ref(m, x))
            }
            TyKind::Ptr(m, x) => {
                let x = self.normalize(x, depth);
                self.mk(TyKind::Ptr(m, x))
            }
            TyKind::Slice(x) => {
                let x = self.normalize(x, depth);
                self.mk(TyKind::Slice(x))
            }
            TyKind::Array(x, c) => {
                let x = self.normalize(x, depth);
                self.mk(TyKind::Array(x, c))
            }
            // `dyn HttpService<F, S = <F as Framing>::F>`.
            TyKind::Dyn(d) => {
                let mut data = tcx.dyn_of(d);
                for tr in &mut data.traits {
                    let v = tcx.tys(tr.args).into_iter().map(|x| self.normalize(x, depth)).collect();
                    tr.args = tcx.list(v);
                    for b in &mut tr.bindings {
                        b.1 = self.normalize(b.1, depth);
                    }
                }
                self.mk(TyKind::Dyn(tcx.dyn_(data)))
            }
            TyKind::FnPtr(s) | TyKind::Closure(_, s, _) => {
                let mut sig = tcx.sig_of(s);
                let inputs = tcx.tys(sig.inputs).into_iter().map(|x| self.normalize(x, depth)).collect();
                sig.inputs = tcx.list(inputs);
                sig.output = self.normalize(sig.output, depth);
                let sig = tcx.sig(sig);
                match tcx.kind(t) {
                    TyKind::Closure(c, _, l) => {
                        let v = tcx.tys(l).into_iter().map(|x| self.normalize(x, depth)).collect();
                        self.mk(TyKind::Closure(c, sig, tcx.list(v)))
                    }
                    _ => self.mk(TyKind::FnPtr(sig)),
                }
            }
            TyKind::AsyncBlock(c, o, l) => {
                let o = self.normalize(o, depth);
                let v = tcx.tys(l).into_iter().map(|x| self.normalize(x, depth)).collect();
                self.mk(TyKind::AsyncBlock(c, o, tcx.list(v)))
            }
            TyKind::Coroutine(c, o, l) => {
                let o = self.normalize(o, depth);
                let v = tcx.tys(l).into_iter().map(|x| self.normalize(x, depth)).collect();
                self.mk(TyKind::Coroutine(c, o, tcx.list(v)))
            }
            TyKind::GenBlock(c, y, l, a) => {
                let y = self.normalize(y, depth);
                let v = tcx.tys(l).into_iter().map(|x| self.normalize(x, depth)).collect();
                self.mk(TyKind::GenBlock(c, y, tcx.list(v), a))
            }
            TyKind::Opaque(o, l) => {
                let v = tcx.tys(l).into_iter().map(|x| self.normalize(x, depth)).collect();
                self.mk(TyKind::Opaque(o, tcx.list(v)))
            }
            _ => t,
        }
    }

    /// Tries the pending projections again: those whose types are now
    /// known enough give their variable a type.
    pub(super) fn process_pending(&mut self) {
        // A projection settled may settle a trait obligation's arguments,
        // and the other way round: until neither changes.
        loop {
            let before = (self.pending_traits.len(), self.pending.len());
            self.process_pending_once();
            if (self.pending_traits.len(), self.pending.len()) == before {
                break;
            }
        }
    }

    fn process_pending_once(&mut self) {
        let tcx = self.tcx;
        // Trait predicates whose arguments are variables: one impl
        // matching settles them.
        let mut i = 0;
        while i < self.pending_traits.len() {
            let (t, tr, l) = self.pending_traits[i];
            let args: Vec<Ty> = tcx.tys(l).into_iter().map(|x| self.infer.resolve(x)).collect();
            let rt = self.infer.resolve(t);
            if !args.iter().any(|&a| tcx.has_infer(a)) && !tcx.has_infer(rt) {
                self.pending_traits.swap_remove(i);
                continue;
            }
            // Its type not known yet (not even what type it is): later.
            // One known at the top with unknowns inside (`C<?N>: Foo<3>`)
            // is tried: the impl that fits may settle them.
            if self.infer.var_kind(self.infer.shallow(t)).is_some() {
                i += 1;
                continue;
            }
            // (An opaque or `dyn` type's bounds may settle both sides.)
            if tcx.has_infer(rt)
                && args.iter().any(|&a| tcx.has_infer(a))
                && !matches!(tcx.kind(rt), TyKind::Opaque(..) | TyKind::Dyn(_))
            {
                i += 1;
                continue;
            }
            match self.select(t, tr, &args, 1) {
                Select::Ambiguous if !self.settle_by_agreement(t, tr, &args) => i += 1,
                // The impl chosen: its `where` clauses with unknowns in
                // them may settle those in turn (`Result<(), ()>:
                // FromResidual<Result<!, ?E>>` needs `(): From<?E>`).
                Select::Yes(Source::Impl(imp, iargs)) => {
                    self.pending_traits.swap_remove(i);
                    let def = tcx.impls[imp].def;
                    let params = tcx.all_params(def);
                    for p in &tcx.predicates[def.0 as usize] {
                        let Pred::Trait(pt, ptr) = p else { continue };
                        if Some(&ptr.def) == tcx.lang.get("sized") || self.pending_traits.len() > 64 {
                            continue;
                        }
                        let pt = subst(tcx, *pt, &params, &iargs);
                        let pa: Vec<Ty> = tcx.tys(ptr.args).into_iter().map(|x| subst(tcx, x, &params, &iargs)).collect();
                        if !tcx.has_infer(self.infer.resolve(pt)) && pa.iter().any(|&a| tcx.has_infer(self.infer.resolve(a))) {
                            let pa = self.full_trait_args(ptr.def, pt, pa);
                            self.pending_traits.push((pt, ptr.def, tcx.list(pa)));
                        }
                    }
                }
                _ => {
                    self.pending_traits.swap_remove(i);
                }
            }
        }
        let mut i = 0;
        while i < self.pending.len() {
            let (a, l, v) = self.pending[i];
            let args: Vec<Ty> = tcx.tys(l).into_iter().map(|x| self.infer.shallow(x)).collect();
            // A variable is known only by its obligations (an `impl
            // FnMut` result's, say): not from this one itself.
            let v = self.infer.shallow(v);
            let saved = self.obligations.clone();
            // (The obligation's variable may since have been made the
            // same as another: compared as resolved.)
            let infer = &self.infer;
            let others: Vec<Pred> = saved
                .iter()
                .filter(|p| !matches!(p, Pred::Projection(.., t) if infer.shallow(*t) == v))
                .cloned()
                .collect();
            self.obligations = others;
            let got = self.project(a, &args, 1);
            self.obligations = saved;
            match got {
                Some(t) => {
                    self.infer.unify(t, v);
                    self.pending.swap_remove(i);
                }
                None => {
                    // Known now, but not projectable: a projection the
                    // body cannot see through (`<I as Iterator>::Item`).
                    let rigid = args.iter().any(|&x| mentions_rigid(tcx, self.infer.resolve(x)));
                    let p = self.mk(TyKind::Projection(a, tcx.list(args)));
                    let p = self.infer.resolve(p);
                    if rigid && !tcx.has_infer(p) && !tcx.has_error(p) {
                        self.infer.unify(p, v);
                        self.pending.swap_remove(i);
                    } else {
                        i += 1;
                    }
                }
            }
        }
    }

    /// The type at the top of `t` once known: variables followed, and a
    /// projection worked out (trying pending ones if `t` is a variable).
    pub(super) fn norm(&mut self, t: Ty) -> Ty {
        let mut t = self.infer.shallow(t);
        if self.infer.var_kind(t) == Some(VarKind::General) && !self.pending.is_empty() {
            self.process_pending();
            t = self.infer.shallow(t);
        }
        if let TyKind::Projection(..) = self.tcx.kind(t) {
            let n = self.normalize(t, 0);
            return self.infer.shallow(n);
        }
        t
    }

    // ---- built on the above ----

    /// The lang item `name`.
    pub(super) fn lang(&self, name: &str) -> Option<DefId> {
        self.tcx.lang.get(name).copied()
    }

    /// `*t`, built in or through `Deref`, if `t` can be dereferenced: the
    /// type, and which.
    pub(super) fn deref_once(&mut self, t: Ty) -> Option<(Ty, Adjust)> {
        let tcx = self.tcx;
        let t = self.norm(t);
        match tcx.kind(t) {
            TyKind::Ref(_, x) => return Some((x, Adjust::Deref)),
            TyKind::Adt(d, l) if Some(d) == self.lang("owned_box") => return Some((tcx.tys(l)[0], Adjust::Deref)),
            TyKind::Infer(_) | TyKind::Error => return None,
            _ => {}
        }
        let deref = self.lang("deref")?;
        let target = self.lang("deref_target")?;
        match self.select(t, deref, &[], 0) {
            Select::Yes(_) => Some((self.project_or_var(target, &[t]), Adjust::OverloadedDeref)),
            _ => None,
        }
    }

    /// `t`, `*t`, `**t`, ...: the types autoderef goes through, each with
    /// how it was reached from the one before (none for `t`), then an
    /// array's slice (reached by `Unsize`).
    pub(super) fn autoderef(&mut self, t: Ty) -> Vec<(Ty, Option<Adjust>)> {
        let tcx = self.tcx;
        let mut steps = vec![(self.norm(t), None)];
        while steps.len() < 8 {
            let last = steps.last().unwrap().0;
            match self.deref_once(last) {
                Some((n, how)) => {
                    let n = self.norm(n);
                    if steps.iter().any(|s| s.0 == n) {
                        break;
                    }
                    steps.push((n, Some(how)));
                }
                None => break,
            }
        }
        if let TyKind::Array(e, _) = tcx.kind(steps.last().unwrap().0) {
            steps.push((self.mk(TyKind::Slice(e)), Some(Adjust::Unsize)));
        }
        steps
    }

    /// The adjustments that reach step `k` of `steps`, then `borrow` (an
    /// autoref) if any. Unsizing an array comes after the borrow: it is
    /// the pointer that is unsized.
    pub(super) fn step_adjustments(steps: &[(Ty, Option<Adjust>)], k: usize, borrow: Option<Mutbl>) -> Vec<Adjust> {
        let mut out = Vec::new();
        let mut unsize = false;
        for s in &steps[1..=k] {
            match s.1 {
                Some(Adjust::Unsize) => unsize = true,
                Some(a) => out.push(a),
                None => {}
            }
        }
        if let Some(m) = borrow {
            out.push(Adjust::Borrow(m));
        }
        if unsize {
            out.push(Adjust::Unsize);
        }
        out
    }

    /// Traits with an item named `name`, those the body is more likely to
    /// mean first: bounds, then traits in scope, then the rest.
    pub(super) fn trait_candidates(&mut self, name: &str) -> Vec<(DefId, DefId)> {
        self.trait_candidates_in(name, false)
    }

    /// The traits with an item `name`, and those items: what the body
    /// assumes first, then those in scope, then (unless `scoped`) the
    /// rest.
    fn trait_candidates_in(&mut self, name: &str, scoped: bool) -> Vec<(DefId, DefId)> {
        let Some(c) = self.index.trait_items.get(name) else { return Vec::new() };
        let mut c = c.clone();
        c.retain(|&(tr, _)| self.can_use(tr));
        if c.len() > 1 || scoped {
            let bound: Vec<DefId> = self
                .env_preds
                .iter()
                .chain(&self.obligations)
                .filter_map(|p| match p {
                    Pred::Trait(_, tr) => Some(tr.def),
                    _ => None,
                })
                .collect();
            let rank: Vec<u8> = c
                .iter()
                .map(|&(tr, _)| {
                    if bound.contains(&tr) {
                        0
                    } else if self.in_scope(tr) {
                        1
                    } else {
                        2
                    }
                })
                .collect();
            let mut ranked: Vec<(u8, (DefId, DefId))> = rank.into_iter().zip(c).collect();
            ranked.sort_by_key(|x| x.0);
            c = ranked.into_iter().filter(|x| !scoped || x.0 < 2).map(|x| x.1).collect();
        }
        c
    }

    /// Whether trait `tr` is in scope in the body: by name in its module
    /// (or the blocks and impls around it), or in the prelude.
    fn in_scope(&mut self, tr: DefId) -> bool {
        if let Some(&b) = self.scope_cache.get(&tr) {
            return b;
        }
        let r = self.r;
        let name = r.defs[tr.0 as usize].name;
        let mut found = false;
        let mut cur = r.defs[self.owner.0 as usize].parent;
        while let Some(m) = cur {
            if r.lookup(m, name, Ns::Type, m) == Some(Res::Def(tr)) {
                found = true;
                break;
            }
            let module = &r.modules[m.0 as usize];
            if module.normal == m {
                break;
            }
            cur = module.parent;
        }
        if !found && let Some(p) = r.preludes[self.k as usize] {
            found = r.lookup(p, name, Ns::Type, p) == Some(Res::Def(tr));
        }
        self.scope_cache.insert(tr, found);
        found
    }

    /// A trait method `name` whose receiver is `recv`: the method, the
    /// other parameters' types and the result.
    /// With `scoped`, only traits the body assumes or has in scope.
    pub(super) fn trait_method(&mut self, recv: Ty, name: &str, scoped: bool) -> Option<(DefId, Vec<Ty>, Vec<Ty>, Ty)> {
        let tcx = self.tcx;
        for (tr, item) in self.trait_candidates_in(name, scoped) {
            if self.r.defs[item.0 as usize].kind != DefKind::AssocFn || !self.has_self(item) {
                continue;
            }
            let s = self.probe();
            let params = tcx.all_params(item);
            let args = self.infer.instantiate(&params);
            let own = tcx.generics[item.0 as usize].own.len();
            let self_ty = args[0];
            let targs = args[1..params.len() - own].to_vec();
            let (inputs, output) = match self.callable_sig(item, tcx.list(args.clone())) {
                Some(x) => x,
                None => {
                    self.rollback(s);
                    continue;
                }
            };
            if inputs.is_empty() || !self.infer.unify(inputs[0], recv) {
                self.rollback(s);
                continue;
            }
            match self.select(self_ty, tr, &targs, 0) {
                Select::No => {
                    self.rollback(s);
                    continue;
                }
                // Its trait's arguments may be settled later.
                Select::Ambiguous if targs.iter().any(|&a| tcx.has_infer(self.infer.resolve(a))) => {
                    self.pending_traits.push((self_ty, tr, tcx.list(targs.clone())));
                }
                _ => {}
            }
            return Some((item, args, inputs[1..].to_vec(), output));
        }
        None
    }

    /// Whether function `item` takes `self`: only those are methods
    /// (`Sum::sum(iter)` is not).
    pub(super) fn has_self(&self, item: DefId) -> bool {
        let def = &self.r.defs[item.0 as usize];
        let ast = &self.env.crates[def.krate as usize].files[def.file as usize].ast;
        ast.child(def.node, crate::ast::Kind::Params).is_some_and(|p| ast.child(p, crate::ast::Kind::SelfParam).is_some())
    }

    /// `T::name` where `T` implements a trait with an item `name` (a
    /// function or constant): its type.
    pub(super) fn trait_assoc_value(&mut self, n: crate::ast::NodeId, t: Ty, name: &str) -> Option<Ty> {
        let tcx = self.tcx;
        for (tr, item) in self.trait_candidates(name) {
            let s = self.probe();
            let params = tcx.all_params(item);
            let mut args = self.infer.instantiate(&params);
            args[0] = t;
            let own = tcx.generics[item.0 as usize].own.len();
            let targs = args[1..params.len() - own].to_vec();
            if !self.select(t, tr, &targs, 0).holds() {
                self.rollback(s);
                continue;
            }
            match self.r.defs[item.0 as usize].kind {
                DefKind::AssocFn => return Some(self.mk(TyKind::FnDef(item, tcx.list(args)))),
                DefKind::AssocConst => {
                    if let Some(ct) = tcx.type_of(item) {
                        self.consts.push((n, item, args.clone()));
                        let ct = subst(tcx, ct, &params, &args);
                        return Some(self.normalize(ct, 0));
                    }
                }
                _ => {}
            }
            self.rollback(s);
        }
        None
    }

    /// An operator through its trait (lang item `lang`): `Output` of
    /// `a: Trait<b>` (or of `a: Trait` for a unary operator). Records that
    /// `n` calls the trait's method `method`.
    pub(super) fn op_trait(&mut self, n: crate::ast::NodeId, lang: &str, method: &str, a: Ty, b: Option<Ty>) -> Option<Ty> {
        let tr = self.lang(lang)?;
        let targs: Vec<Ty> = b.into_iter().collect();
        if !self.select(a, tr, &targs, 0).holds() {
            return None;
        }
        self.record_trait_method(n, tr, method, a, &targs);
        let Some(out) = self.trait_item(tr, "Output", Ns::Type) else { return Some(self.unit()) };
        let mut args = vec![a];
        args.extend(targs);
        Some(self.project_or_var(out, &args))
    }

    /// Records that node `n` calls trait `tr`'s method `method` with
    /// `Self` = `self_ty` and the trait's other arguments `targs`.
    pub(super) fn record_trait_method(&mut self, n: crate::ast::NodeId, tr: DefId, method: &str, self_ty: Ty, targs: &[Ty]) {
        if let Some(m) = self.trait_item(tr, method, Ns::Value) {
            let mut args = vec![self_ty];
            let full = self.full_trait_args(tr, self_ty, targs.to_vec());
            args.extend(full);
            self.methods.push((n, m, args));
        }
    }

    /// Calling a value of type `t` through the `Fn` traits with `n`
    /// arguments: the parameter types and result. Records the trait
    /// method the call node `call` calls.
    pub(super) fn call_fn_traits(&mut self, call: crate::ast::NodeId, t: Ty, n: usize) -> Option<(Vec<Ty>, Ty)> {
        let tcx = self.tcx;
        let output = self.lang("fn_once_output")?;
        if let Some(sig) = self.call_fn_traits_sync(call, t, n, output) {
            return Some(sig);
        }
        // `f()` with `f: impl AsyncFnOnce()`: a call gives its future.
        let fut = self.lang("call_once_future")?;
        for (l, method) in
            [("async_fn", "async_call"), ("async_fn_mut", "async_call_mut"), ("async_fn_once", "async_call_once")]
        {
            let Some(tr) = self.lang(l) else { continue };
            let s = self.probe();
            let inputs: Vec<Ty> = (0..n).map(|_| self.infer.fresh()).collect();
            let tuple = self.mk(TyKind::Tuple(tcx.list(inputs.clone())));
            match self.select(t, tr, &[tuple], 0) {
                Select::Yes(Source::Builtin(Some(out))) => {
                    self.record_trait_method(call, tr, method, t, &[tuple]);
                    return Some((inputs, out));
                }
                Select::Yes(_) => {
                    self.record_trait_method(call, tr, method, t, &[tuple]);
                    let out = self.project_or_var(fut, &[t, tuple]);
                    return Some((inputs, out));
                }
                _ => self.rollback(s),
            }
        }
        None
    }

    fn call_fn_traits_sync(
        &mut self,
        call: crate::ast::NodeId,
        t: Ty,
        n: usize,
        output: DefId,
    ) -> Option<(Vec<Ty>, Ty)> {
        let tcx = self.tcx;
        for l in ["fn", "fn_mut", "fn_once"] {
            let Some(tr) = self.lang(l) else { continue };
            let s = self.probe();
            let inputs: Vec<Ty> = (0..n).map(|_| self.infer.fresh()).collect();
            let tuple = self.mk(TyKind::Tuple(tcx.list(inputs.clone())));
            let method = match l {
                "fn" => "call",
                "fn_mut" => "call_mut",
                _ => "call_once",
            };
            match self.select(t, tr, &[tuple], 0) {
                Select::Yes(Source::Builtin(Some(out))) => {
                    self.record_trait_method(call, tr, method, t, &[tuple]);
                    return Some((inputs, out));
                }
                Select::Yes(_) => {
                    self.record_trait_method(call, tr, method, t, &[tuple]);
                    let out = self.project_or_var(output, &[t, tuple]);
                    return Some((inputs, out));
                }
                _ => self.rollback(s),
            }
        }
        None
    }

    /// What iterating over `t` (in a `for`) gives.
    pub(super) fn iterated_item(&mut self, t: Ty) -> Option<Ty> {
        let f = self.lang("into_iter")?;
        let tr = self.trait_of(f)?;
        let item = self.trait_item(tr, "Item", Ns::Type)?;
        if !self.select(t, tr, &[], 0).holds() {
            return None;
        }
        Some(self.project_or_var(item, &[t]))
    }

    /// Records the calls `for` (node `n`) makes on what it iterates, `t`:
    /// `IntoIterator::into_iter(t)`, then `Iterator::next` on the result.
    pub(super) fn record_for(&mut self, n: crate::ast::NodeId, t: Ty) {
        let t = self.norm(t);
        let (Some(f), Some(it)) = (self.lang("into_iter"), self.lang("iterator")) else { return };
        let Some(tr) = self.trait_of(f) else { return };
        self.implicit_call(n, tr, "into_iter", vec![t]);
        if let Some(a) = self.trait_item(tr, "IntoIter", Ns::Type) {
            let iter = self.project_or_var(a, &[t]);
            self.implicit_call(n, it, "next", vec![iter]);
        }
    }

    /// Records the calls `t?` (node `n`) makes: `Try::branch(t)`, and
    /// `FromResidual::from_residual` of the function's result type.
    pub(super) fn record_try(&mut self, n: crate::ast::NodeId, t: Ty) {
        let tcx = self.tcx;
        let t = self.norm(t);
        let Some(tr) = self.lang("Try") else { return };
        self.implicit_call(n, tr, "branch", vec![t]);
        let Some(res) = self.trait_item(tr, "Residual", Ns::Type) else { return };
        let residual = self.project_or_var(res, &[t]);
        let from = tcx.traits.get(&tr).and_then(|d| d.supertraits.iter().find(|s| self.trait_item(s.def, "from_residual", Ns::Value).is_some()).map(|s| s.def));
        if let Some(fr) = from {
            // In a `try` block, the `?` goes to it, not out of the function.
            let ret = self.try_blocks.last().copied().unwrap_or(self.ret);
            self.implicit_call(n, fr, "from_residual", vec![ret, residual]);
            // `R: FromResidual<Result<Infallible, E>>`, where the one impl
            // that fits settles `E` (`Ok(())?` in a `Result<(), ()>`).
            self.pending_traits.push((ret, fr, tcx.list(vec![residual])));
        }
    }

    /// What `t?` gives: `<t as Try>::Output`.
    pub(super) fn try_output_trait(&mut self, t: Ty) -> Option<Ty> {
        let tr = self.lang("Try")?;
        let out = self.trait_item(tr, "Output", Ns::Type)?;
        if !self.select(t, tr, &[], 0).holds() {
            return None;
        }
        Some(self.project_or_var(out, &[t]))
    }

    /// What `t.await` gives: `<t as Future>::Output`.
    pub(super) fn await_output(&mut self, t: Ty) -> Option<Ty> {
        let out = self.lang("future_output")?;
        let tr = self.trait_of(out)?;
        if !self.select(t, tr, &[], 0).holds() {
            return None;
        }
        Some(self.project_or_var(out, &[t]))
    }

    /// What a closure expected as `t` (a variable a generic function's
    /// `F: Fn(A) -> B` bound is on) should take and give.
    pub(super) fn closure_expectation(&mut self, t: Ty) -> Option<(Vec<Ty>, Option<Ty>)> {
        self.closure_expectation_of(t, false)
    }

    /// The same through `AsyncFn` bounds (`asyncness`): what an async
    /// closure takes, and the output of the future it gives.
    pub(super) fn closure_expectation_of(&mut self, t: Ty, asyncness: bool) -> Option<(Vec<Ty>, Option<Ty>)> {
        let tcx = self.tcx;
        let t = self.infer.shallow(t);
        let (traits, out) = if asyncness {
            (["async_fn", "async_fn_mut", "async_fn_once"], "async_fn_once_output")
        } else {
            (["fn", "fn_mut", "fn_once"], "fn_once_output")
        };
        let fns: Vec<DefId> = traits.iter().filter_map(|l| self.lang(l)).collect();
        let output = self.lang(out);
        let mut inputs = None;
        let mut ret = None;
        let mut bounds = self.type_bounds(t);
        // Bounds of other traits, through their supertraits (`F:
        // Receive<T, E>` where `Receive<T, E>: FnOnce(Result<T, E>)`).
        let others: Vec<TraitRef> = self
            .obligations
            .iter()
            .chain(&self.env_preds)
            .filter_map(|p| match p {
                Pred::Trait(pt, tr) if !fns.contains(&tr.def) && self.infer.shallow(*pt) == t => Some(tr.clone()),
                _ => None,
            })
            .collect();
        for tr in others {
            let mut sup = Vec::new();
            self.elaborate(t, &tr, &mut sup);
            bounds.extend(sup);
        }
        // A trait alias's `where` clauses on its parameters (`T:
        // Confusing<F>` with `trait Confusing<F> = Fn(i32) where F:
        // Fn(u32)`).
        let aliases: Vec<(Ty, TraitRef)> = self
            .obligations
            .iter()
            .chain(&self.env_preds)
            .filter_map(|p| match p {
                Pred::Trait(pt, tr) if self.r.defs[tr.def.0 as usize].kind == DefKind::TraitAlias => {
                    Some((*pt, tr.clone()))
                }
                _ => None,
            })
            .collect();
        for (pt, tr) in aliases {
            let params = tcx.all_params(tr.def);
            let mut args = vec![pt];
            args.extend(tcx.tys(tr.args));
            if args.len() != params.len() {
                continue;
            }
            for q in &tcx.predicates[tr.def.0 as usize] {
                match q {
                    Pred::Trait(x, qtr) if fns.contains(&qtr.def) => {
                        let x = subst(tcx, *x, &params, &args);
                        if self.infer.shallow(x) == t {
                            bounds.push(Pred::Trait(x, subst_trait_ref(tcx, qtr, &params, &args)));
                        }
                    }
                    Pred::Projection(x, a, l, ty) if Some(*a) == output => {
                        let sub = |y: Ty| subst(tcx, y, &params, &args);
                        let l = tcx.list(tcx.tys(*l).into_iter().map(sub).collect());
                        bounds.push(Pred::Projection(sub(*x), *a, l, sub(*ty)));
                    }
                    _ => {}
                }
            }
        }
        // Through an impl: `Wrap<F>: Foo` with `impl<T> Foo for Wrap<T>
        // where T: Fn(i32)`, the one impl that applies.
        let around: Vec<(Ty, TraitRef)> = self
            .obligations
            .iter()
            .filter_map(|p| match p {
                Pred::Trait(pt, tr) if self.infer.shallow(*pt) != t && self.mentions(*pt, t) => Some((*pt, tr.clone())),
                _ => None,
            })
            .collect();
        for (pt, tr) in around {
            let args = tcx.tys(tr.args);
            if let Select::Yes(Source::Impl(i, iargs)) = self.select(pt, tr.def, &args, 0) {
                let params = tcx.all_params(tcx.impls[i].def);
                for p in &tcx.predicates[tcx.impls[i].def.0 as usize] {
                    match p {
                        Pred::Trait(x, ptr) if fns.contains(&ptr.def) => {
                            let x = subst(tcx, *x, &params, &iargs);
                            bounds.push(Pred::Trait(x, subst_trait_ref(tcx, ptr, &params, &iargs)));
                        }
                        Pred::Projection(x, a, l, ty) if Some(*a) == output => {
                            let sub = |y: Ty| subst(tcx, y, &params, &iargs);
                            let l = tcx.list(tcx.tys(*l).into_iter().map(sub).collect());
                            bounds.push(Pred::Projection(sub(*x), *a, l, sub(*ty)));
                        }
                        _ => {}
                    }
                }
            }
        }
        for p in self.obligations.iter().chain(&self.env_preds).chain(&bounds) {
            match p {
                Pred::Trait(pt, tr) if fns.contains(&tr.def) && self.infer.shallow(*pt) == t => {
                    if let Some(&a) = tcx.tys(tr.args).first()
                        && let TyKind::Tuple(l) = tcx.kind(self.infer.shallow(a))
                    {
                        inputs = Some(tcx.tys(l));
                    }
                }
                // Not a projection on the closure's own type (`Fn0: Fn(i32)
                // -> Self::Out`), whose value would need the closure's.
                Pred::Projection(pt, a, _, ty)
                    if Some(*a) == output && self.infer.shallow(*pt) == t && !self.projects_from(*ty, t) =>
                {
                    ret = Some(*ty)
                }
                _ => {}
            }
        }
        inputs.map(|i| (i, ret))
    }

    /// Whether `t` has a projection on `v` in it.
    fn projects_from(&self, t: Ty, v: Ty) -> bool {
        let tcx = self.tcx;
        match tcx.kind(self.infer.shallow(t)) {
            TyKind::Projection(_, l) => tcx.tys(l).first().is_some_and(|&s| self.mentions(s, v)),
            TyKind::Adt(_, l) | TyKind::Tuple(l) => tcx.tys(l).into_iter().any(|x| self.projects_from(x, v)),
            TyKind::Ref(_, x) | TyKind::Ptr(_, x) | TyKind::Slice(x) | TyKind::Array(x, _) => self.projects_from(x, v),
            _ => false,
        }
    }

    /// Whether type `t` has `v` (a variable) in it.
    fn mentions(&self, t: Ty, v: Ty) -> bool {
        let tcx = self.tcx;
        let t = self.infer.shallow(t);
        if t == v {
            return true;
        }
        match tcx.kind(t) {
            TyKind::Adt(_, l) | TyKind::Tuple(l) => tcx.tys(l).into_iter().any(|x| self.mentions(x, v)),
            TyKind::Ref(_, x) | TyKind::Ptr(_, x) | TyKind::Slice(x) | TyKind::Array(x, _) => self.mentions(x, v),
            _ => false,
        }
    }

    /// Whether `t` (a reference's or `Box`'s target) can be coerced to
    /// `want` by unsizing: to a `dyn` it implements, or from an array to a
    /// slice.
    pub(super) fn unsizes_to(&mut self, t: Ty, want: Ty) -> bool {
        let tcx = self.tcx;
        let (t, w) = (self.infer.shallow(t), self.infer.shallow(want));
        match (tcx.kind(t), tcx.kind(w)) {
            (TyKind::Array(e, _), TyKind::Slice(f)) => self.infer.unify(e, f),
            (TyKind::Dyn(_), TyKind::Dyn(_)) => true,
            // A struct whose last field unsizes: `S<[T; N]>` to `S<[T]>`,
            // the other arguments the same.
            (TyKind::Adt(a, x), TyKind::Adt(b, y)) if a == b && tcx.adts.get(&a).is_some_and(|d| d.kind == crate::ty::AdtKind::Struct) => {
                let (xs, ys) = (tcx.tys(x), tcx.tys(y));
                let mut unsized_at = None;
                for (i, (&p, &q)) in xs.iter().zip(&ys).enumerate() {
                    let s = self.probe();
                    if self.infer.unify(p, q) {
                        continue;
                    }
                    self.rollback(s);
                    if unsized_at.is_some() {
                        return false;
                    }
                    unsized_at = Some(i);
                }
                match unsized_at {
                    Some(i) => self.unsizes_to(xs[i], ys[i]),
                    None => false,
                }
            }
            (_, TyKind::Dyn(d)) => match tcx.dyn_of(d).traits.first() {
                Some(tr) => {
                    let args = tcx.tys(tr.args);
                    if !self.select(t, tr.def, &args, 0).holds() {
                        return false;
                    }
                    // Its bindings are the type's (`async { vec![] }` as a
                    // `dyn Future<Output = Vec<u32>>`).
                    for &(assoc, b) in &tr.bindings {
                        let mut pa = vec![t];
                        pa.extend(args.iter().copied());
                        let p = self.project_or_var(assoc, &pa);
                        self.infer.unify(p, b);
                    }
                    true
                }
                None => true,
            },
            _ => false,
        }
    }

    /// Coercing `&x` to `&y` (or `&mut y`, `m`) through `Deref`: `&String`
    /// to `&str`. The adjustments if so: the reference's `*`, the steps to
    /// `y`, and a new borrow.
    pub(super) fn deref_coerces(&mut self, x: Ty, y: Ty, m: Mutbl) -> Option<Vec<Adjust>> {
        let steps = self.autoderef(x);
        for k in 1..steps.len() {
            if steps[k].1 == Some(Adjust::Unsize) {
                break;
            }
            let p = self.probe();
            if self.infer.unify(steps[k].0, y) {
                let mut adj = vec![Adjust::Deref];
                adj.extend(Self::step_adjustments(&steps, k, Some(m)));
                return Some(adj);
            }
            self.rollback(p);
        }
        None
    }

    /// `T::name` for an associated type `name` of a trait `T` implements
    /// (or is bound by): the projection, normalised.
    pub(super) fn assoc_type_of(&mut self, t: Ty, name: &str) -> Option<Ty> {
        let tcx = self.tcx;
        let mut preds = self.env_preds.clone();
        preds.extend(self.type_bounds(t));
        for p in preds {
            let Pred::Trait(pt, tr) = p else { continue };
            if self.infer.shallow(pt) != t {
                continue;
            }
            if let Some(a) = self.trait_item(tr.def, name, Ns::Type) {
                let mut args = vec![t];
                args.extend(tcx.tys(tr.args));
                return Some(self.project_or_var(a, &args));
            }
        }
        // Any trait with such an associated type that `t` implements, in
        // the order they are defined (not the map's, which other crates'
        // traits change).
        let mut traits: Vec<DefId> = tcx
            .traits
            .keys()
            .copied()
            .filter(|&tr| self.can_use(tr) && self.trait_item(tr, name, Ns::Type).is_some())
            .collect();
        traits.sort();
        for tr in traits {
            let n = tcx.generics[tr.0 as usize].own.len().saturating_sub(1);
            let targs: Vec<Ty> = (0..n).map(|_| self.infer.fresh()).collect();
            if let Select::Yes(_) = self.select(t, tr, &targs, 0) {
                let a = self.trait_item(tr, name, Ns::Type)?;
                let mut args = vec![t];
                args.extend(targs);
                return Some(self.project_or_var(a, &args));
            }
        }
        None
    }

    /// Whether the body's crate may use `d`: it is its own, or of a crate
    /// it depends on (`CrateRoot::deps`).
    pub(super) fn can_use(&self, d: DefId) -> bool {
        let crates = self.env.crates;
        let c = self.r.defs[d.0 as usize].krate;
        c == self.k
            || crates[self.k as usize].deps.as_ref().is_none_or(|deps| deps.contains(&crates[c as usize].name))
    }

    /// A `&` or `&mut` of `t`.
    pub(super) fn mk_ref(&self, m: Mutbl, t: Ty) -> Ty {
        self.mk(TyKind::Ref(m, t))
    }
}

/// `tr` with `params` replaced by `args`.
pub(super) fn subst_trait_ref(tcx: &crate::ty::TyCtxt, tr: &TraitRef, params: &[crate::ty::ParamId], args: &[Ty]) -> TraitRef {
    let a: Vec<Ty> = tcx.tys(tr.args).into_iter().map(|x| subst(tcx, x, params, args)).collect();
    TraitRef {
        def: tr.def,
        args: tcx.list(a),
        bindings: tr.bindings.iter().map(|&(d, t)| (d, subst(tcx, t, params, args))).collect(),
        assoc_bounds: tr.assoc_bounds.iter().map(|(d, b)| (*d, subst_trait_ref(tcx, b, params, args))).collect(),
    }
}

/// Whether a type mentions something a body cannot see through: a
/// parameter, `dyn`, `impl Trait`, or a projection on one of those.
fn mentions_rigid(tcx: &crate::ty::TyCtxt, t: Ty) -> bool {
    match tcx.kind(t) {
        TyKind::Param(_) | TyKind::Opaque(..) | TyKind::Dyn(_) | TyKind::Projection(..) => true,
        TyKind::Tuple(l) | TyKind::Adt(_, l) | TyKind::FnDef(_, l) => tcx.tys(l).into_iter().any(|x| mentions_rigid(tcx, x)),
        TyKind::Ref(_, x) | TyKind::Ptr(_, x) | TyKind::Slice(x) | TyKind::Array(x, _) | TyKind::Pat(x, ..) => mentions_rigid(tcx, x),
        _ => false,
    }
}

/// Whether a type has a projection in it.
fn has_projection(tcx: &crate::ty::TyCtxt, t: Ty) -> bool {
    match tcx.kind(t) {
        TyKind::Projection(..) => true,
        TyKind::Tuple(l) | TyKind::Adt(_, l) | TyKind::Opaque(_, l) | TyKind::FnDef(_, l) => tcx.tys(l).into_iter().any(|x| has_projection(tcx, x)),
        TyKind::Ref(_, x) | TyKind::Ptr(_, x) | TyKind::Slice(x) | TyKind::Array(x, _) => has_projection(tcx, x),
        TyKind::FnPtr(s) => {
            let s = tcx.sig_of(s);
            has_projection(tcx, s.output) || tcx.tys(s.inputs).into_iter().any(|x| has_projection(tcx, x))
        }
        TyKind::Closure(_, s, l) => {
            let s = tcx.sig_of(s);
            has_projection(tcx, s.output)
                || tcx.tys(s.inputs).into_iter().any(|x| has_projection(tcx, x))
                || tcx.tys(l).into_iter().any(|x| has_projection(tcx, x))
        }
        TyKind::Dyn(d) => tcx.dyn_of(d).traits.iter().any(|tr| {
            tcx.tys(tr.args).into_iter().any(|x| has_projection(tcx, x))
                || tr.bindings.iter().any(|b| has_projection(tcx, b.1))
        }),
        TyKind::AsyncBlock(_, o, l) | TyKind::Coroutine(_, o, l) | TyKind::GenBlock(_, o, l, _) => {
            has_projection(tcx, o) || tcx.tys(l).into_iter().any(|x| has_projection(tcx, x))
        }
        _ => false,
    }
}

/// `preds` with their associated type bounds spelled out: `I:
/// Iterator<Item: IntoIterator<IntoIter = U>>` also gives `I::Item:
/// IntoIterator` and `<I::Item as IntoIterator>::IntoIter == U`.
fn expand_assoc_bounds(tcx: &crate::ty::TyCtxt, preds: &[Pred]) -> Vec<Pred> {
    let mut out = preds.to_vec();
    let mut i = 0;
    while i < out.len() && out.len() < 64 {
        if let Pred::Trait(t, tr) = out[i].clone() {
            let mut args = vec![t];
            args.extend(tcx.tys(tr.args));
            // A bound's own bindings, for those made here (the written
            // ones are predicates of their own already).
            if i >= preds.len() {
                for &(a, b) in &tr.bindings {
                    out.push(Pred::Projection(t, a, tcx.list(args.clone()), b));
                }
            }
            for (assoc, b) in &tr.assoc_bounds {
                let p = tcx.mk(TyKind::Projection(*assoc, tcx.list(args.clone())));
                out.push(Pred::Trait(p, b.clone()));
            }
        }
        i += 1;
    }
    out
}
