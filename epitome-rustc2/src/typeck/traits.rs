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
use super::infer::VarKind;
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
    /// Built in; for the `Fn` traits, the result type.
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
            TyKind::Opaque(o) => {
                for tr in tcx.opaque(o).bounds {
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
            if matches!(self.r.defs[x.0 as usize].kind, DefKind::Impl | DefKind::Trait) {
                break;
            }
            for p in tcx.predicates[x.0 as usize].clone() {
                let p = match p {
                    Pred::Trait(t, tr) => Pred::Trait(subst(tcx, t, &params, args), subst_trait_ref(tcx, &tr, &params, args)),
                    Pred::Projection(t, a, l, b) => {
                        let l2: Vec<Ty> = tcx.tys(l).into_iter().map(|x| subst(tcx, x, &params, args)).collect();
                        Pred::Projection(subst(tcx, t, &params, args), a, tcx.list(l2), subst(tcx, b, &params, args))
                    }
                };
                self.oblige(p);
            }
            o = tcx.generics[x.0 as usize].parent;
        }
    }

    /// A snapshot of inference and of the obligations.
    pub(super) fn probe(&self) -> (super::infer::Snapshot, usize, usize) {
        (self.infer.snapshot(), self.obligations.len(), self.pending.len())
    }

    pub(super) fn rollback(&mut self, s: (super::infer::Snapshot, usize, usize)) {
        self.infer.rollback(s.0);
        self.obligations.truncate(s.1);
        self.pending.truncate(s.2);
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
        if ["sized", "meta_sized", "pointee_sized", "destruct", "freeze", "unsize", "discriminant_kind", "pointee_trait", "fn_ptr_trait", "transmute_trait", "bikeshed_guaranteed_no_drop"]
            .iter()
            .any(|n| is(n))
        {
            return true;
        }
        match tcx.kind(t) {
            TyKind::Tuple(_) if is("tuple_trait") => true,
            TyKind::Closure(..) | TyKind::FnDef(..) | TyKind::FnPtr(_) | TyKind::Never => is("copy") || is("clone"),
            _ => false,
        }
    }

    /// Whether, and how, `t: tr<targs>`.
    pub(super) fn select(&mut self, t: Ty, tr: DefId, targs: &[Ty], depth: u32) -> Select {
        let tcx = self.tcx;
        let t = self.infer.shallow(t);
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
        // Closures, function items and pointers implement the `Fn` traits.
        if ["fn", "fn_mut", "fn_once"].iter().any(|l| tcx.lang.get(l) == Some(&tr)) {
            let sig = match tcx.kind(t) {
                TyKind::FnDef(d, l) => self.callable_sig(d, l),
                TyKind::FnPtr(s) | TyKind::Closure(_, s) => {
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
        let args = self.infer.instantiate(&params);
        let st = subst(tcx, imp.self_ty, &params, &args);
        if !self.infer.unify(st, t) {
            return None;
        }
        let tr = imp.trait_ref.as_ref()?;
        let iargs: Vec<Ty> = tcx.tys(tr.args).into_iter().map(|x| subst(tcx, x, &params, &args)).collect();
        let iargs = self.full_trait_args(tr.def, st, iargs);
        if !self.args_unify(&iargs, targs) {
            return None;
        }
        if depth < 2 {
            for p in &tcx.predicates[imp.def.0 as usize] {
                if let Pred::Trait(pt, ptr) = p {
                    if Some(&ptr.def) == tcx.lang.get("sized") {
                        continue;
                    }
                    let pt = subst(tcx, *pt, &params, &args);
                    let pa: Vec<Ty> = tcx.tys(ptr.args).into_iter().map(|x| subst(tcx, x, &params, &args)).collect();
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
            if self.self_matches(pt, t) && self.args_unify(&pl[1..], &args[1..]) {
                return Some(ty);
            }
            self.rollback(s);
        }
        let tr = self.trait_of(assoc)?;
        match self.select(t, tr, &args[1..], depth) {
            Select::Yes(Source::Impl(i, iargs)) => {
                let name = self.r.defs[assoc.0 as usize].name;
                let item = self.impl_item(i, name, Ns::Type)?;
                let it = tcx.type_of(item)?;
                let params = tcx.all_params(item);
                let mut a = iargs;
                while a.len() < params.len() {
                    a.push(self.infer.fresh());
                }
                let it = subst(tcx, it, &params, &a);
                Some(self.normalize(it, depth + 1))
            }
            Select::Yes(Source::Builtin(Some(out))) if tcx.lang.get("fn_once_output") == Some(&assoc) => Some(out),
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
        let v = self.infer.fresh();
        self.pending.push((assoc, tcx.list(args.to_vec()), v));
        self.obligations.push(Pred::Projection(args[0], assoc, tcx.list(args.to_vec()), v));
        v
    }

    /// `t` with the projections in it worked out where they can be.
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
            TyKind::FnPtr(s) => {
                let mut sig = tcx.sig_of(s);
                let inputs = tcx.tys(sig.inputs).into_iter().map(|x| self.normalize(x, depth)).collect();
                sig.inputs = tcx.list(inputs);
                sig.output = self.normalize(sig.output, depth);
                self.mk(TyKind::FnPtr(tcx.sig(sig)))
            }
            _ => t,
        }
    }

    /// Tries the pending projections again: those whose types are now
    /// known enough give their variable a type.
    pub(super) fn process_pending(&mut self) {
        let tcx = self.tcx;
        let mut i = 0;
        while i < self.pending.len() {
            let (a, l, v) = self.pending[i];
            let args: Vec<Ty> = tcx.tys(l).into_iter().map(|x| self.infer.shallow(x)).collect();
            // A variable is known only by its obligations (an `impl
            // FnMut` result's, say): not from this one itself.
            let v = self.infer.shallow(v);
            let saved = self.obligations.clone();
            self.obligations.retain(|p| !matches!(p, Pred::Projection(.., t) if *t == v));
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
                    let p = self.mk(TyKind::Projection(a, tcx.list(args)));
                    let p = self.infer.resolve(p);
                    if !tcx.has_infer(p) && !tcx.has_error(p) {
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

    /// `*t` through `Deref`, if `t` implements it.
    pub(super) fn deref_once(&mut self, t: Ty) -> Option<Ty> {
        let tcx = self.tcx;
        let t = self.norm(t);
        match tcx.kind(t) {
            TyKind::Ref(_, x) => return Some(x),
            TyKind::Adt(d, l) if Some(d) == self.lang("owned_box") => return Some(tcx.tys(l)[0]),
            TyKind::Infer(_) | TyKind::Error => return None,
            _ => {}
        }
        let deref = self.lang("deref")?;
        let target = self.lang("deref_target")?;
        match self.select(t, deref, &[], 0) {
            Select::Yes(_) => Some(self.project_or_var(target, &[t])),
            _ => None,
        }
    }

    /// `t`, `*t`, `**t`, ...: the types autoderef goes through, then an
    /// array's slice.
    pub(super) fn autoderef(&mut self, t: Ty) -> Vec<Ty> {
        let tcx = self.tcx;
        let mut steps = vec![self.norm(t)];
        while steps.len() < 8 {
            let last = *steps.last().unwrap();
            match self.deref_once(last) {
                Some(n) => {
                    let n = self.norm(n);
                    if steps.contains(&n) {
                        break;
                    }
                    steps.push(n);
                }
                None => break,
            }
        }
        if let TyKind::Array(e, _) = tcx.kind(*steps.last().unwrap()) {
            steps.push(self.mk(TyKind::Slice(e)));
        }
        steps
    }

    /// Traits with an item named `name`, those the body is more likely to
    /// mean first: bounds, then traits in scope, then the rest.
    pub(super) fn trait_candidates(&mut self, name: &str) -> Vec<(DefId, DefId)> {
        let Some(c) = self.index.trait_items.get(name) else { return Vec::new() };
        let mut c = c.clone();
        if c.len() > 1 {
            let bound: Vec<DefId> = self
                .env_preds
                .iter()
                .chain(&self.obligations)
                .filter_map(|p| match p {
                    Pred::Trait(_, tr) => Some(tr.def),
                    _ => None,
                })
                .collect();
            c.sort_by_cached_key(|&(tr, _)| {
                if bound.contains(&tr) {
                    0
                } else if self.in_scope(tr) {
                    1
                } else {
                    2
                }
            });
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
    pub(super) fn trait_method(&mut self, recv: Ty, name: &str) -> Option<(DefId, Vec<Ty>, Ty)> {
        let tcx = self.tcx;
        for (tr, item) in self.trait_candidates(name) {
            if self.r.defs[item.0 as usize].kind != DefKind::AssocFn {
                continue;
            }
            let s = self.probe();
            let params = tcx.all_params(item);
            let args = self.infer.instantiate(&params);
            let own = tcx.generics[item.0 as usize].own.len();
            let self_ty = args[0];
            let targs = args[1..params.len() - own].to_vec();
            let (inputs, output) = match self.callable_sig(item, tcx.list(args)) {
                Some(x) => x,
                None => {
                    self.rollback(s);
                    continue;
                }
            };
            if inputs.is_empty() || !self.infer.unify(inputs[0], recv) || !self.select(self_ty, tr, &targs, 0).holds() {
                self.rollback(s);
                continue;
            }
            return Some((item, inputs[1..].to_vec(), output));
        }
        None
    }

    /// `T::name` where `T` implements a trait with an item `name` (a
    /// function or constant): its type.
    pub(super) fn trait_assoc_value(&mut self, t: Ty, name: &str) -> Option<Ty> {
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
    /// `a: Trait<b>` (or of `a: Trait` for a unary operator).
    pub(super) fn op_trait(&mut self, lang: &str, a: Ty, b: Option<Ty>) -> Option<Ty> {
        let tr = self.lang(lang)?;
        let targs: Vec<Ty> = b.into_iter().collect();
        if !self.select(a, tr, &targs, 0).holds() {
            return None;
        }
        let Some(out) = self.trait_item(tr, "Output", Ns::Type) else { return Some(self.unit()) };
        let mut args = vec![a];
        args.extend(targs);
        Some(self.project_or_var(out, &args))
    }

    /// Calling a value of type `t` through the `Fn` traits with `n`
    /// arguments: the parameter types and result.
    pub(super) fn call_fn_traits(&mut self, t: Ty, n: usize) -> Option<(Vec<Ty>, Ty)> {
        let tcx = self.tcx;
        let output = self.lang("fn_once_output")?;
        for l in ["fn", "fn_mut", "fn_once"] {
            let Some(tr) = self.lang(l) else { continue };
            let s = self.probe();
            let inputs: Vec<Ty> = (0..n).map(|_| self.infer.fresh()).collect();
            let tuple = self.mk(TyKind::Tuple(tcx.list(inputs.clone())));
            match self.select(t, tr, &[tuple], 0) {
                Select::Yes(Source::Builtin(Some(out))) => return Some((inputs, out)),
                Select::Yes(_) => {
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
        let tcx = self.tcx;
        let t = self.infer.shallow(t);
        let fns: Vec<DefId> = ["fn", "fn_mut", "fn_once"].iter().filter_map(|l| self.lang(l)).collect();
        let output = self.lang("fn_once_output");
        let mut inputs = None;
        let mut ret = None;
        let bounds = self.type_bounds(t);
        for p in self.obligations.iter().chain(&self.env_preds).chain(&bounds) {
            match p {
                Pred::Trait(pt, tr) if fns.contains(&tr.def) && self.infer.shallow(*pt) == t => {
                    if let Some(&a) = tcx.tys(tr.args).first()
                        && let TyKind::Tuple(l) = tcx.kind(self.infer.shallow(a))
                    {
                        inputs = Some(tcx.tys(l));
                    }
                }
                Pred::Projection(pt, a, _, ty) if Some(*a) == output && self.infer.shallow(*pt) == t => ret = Some(*ty),
                _ => {}
            }
        }
        inputs.map(|i| (i, ret))
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
            (_, TyKind::Dyn(d)) => match tcx.dyn_of(d).traits.first() {
                Some(tr) => {
                    let args = tcx.tys(tr.args);
                    self.select(t, tr.def, &args, 0).holds()
                }
                None => true,
            },
            _ => false,
        }
    }

    /// Coercing `&x` to `&y` through `Deref`: `&String` to `&str`.
    pub(super) fn deref_coerces(&mut self, x: Ty, y: Ty) -> bool {
        let steps = self.autoderef(x);
        for &s in &steps[1..] {
            let p = self.probe();
            if self.infer.unify(s, y) {
                return true;
            }
            self.rollback(p);
        }
        false
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

/// Whether a type has a projection in it.
fn has_projection(tcx: &crate::ty::TyCtxt, t: Ty) -> bool {
    match tcx.kind(t) {
        TyKind::Projection(..) => true,
        TyKind::Tuple(l) | TyKind::Adt(_, l) => tcx.tys(l).into_iter().any(|x| has_projection(tcx, x)),
        TyKind::Ref(_, x) | TyKind::Ptr(_, x) | TyKind::Slice(x) | TyKind::Array(x, _) => has_projection(tcx, x),
        TyKind::FnPtr(s) => {
            let s = tcx.sig_of(s);
            has_projection(tcx, s.output) || tcx.tys(s.inputs).into_iter().any(|x| has_projection(tcx, x))
        }
        _ => false,
    }
}
