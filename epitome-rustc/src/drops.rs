//! Which types need dropping, and what dropping one does: the facts both
//! monomorphisation (to find the drop glue and `Drop::drop` instances a
//! body needs) and lowering (to call them) use.
//!
//! A type needs dropping when dropping it runs code: it has a `Drop` impl,
//! or something it holds by value does. `ManuallyDrop` and unions hold
//! their contents without dropping them; a `dyn` is dropped through its
//! vtable, so it always counts.

use crate::hash::{FastMap, FastSet};
use crate::resolve::DefId;
use crate::ty::{AdtKind, ClosureId, List, Ty, TyCtxt, TyKind};
use crate::typeck::{Solver, TypeckResults};
use std::sync::Mutex;

/// The ADTs with a `Drop` impl, and a cache of `needs_drop`.
pub struct DropInfo {
    /// Every struct, enum or union some `impl Drop` is for.
    impls: FastSet<DefId>,
    cache: Mutex<FastMap<Ty, bool>>,
}

impl DropInfo {
    pub fn new(tcx: &TyCtxt) -> DropInfo {
        let mut impls = FastSet::default();
        if let Some(&drop) = tcx.lang.get("drop") {
            for i in &tcx.impls {
                if !i.negative
                    && i.trait_ref.as_ref().is_some_and(|t| t.def == drop)
                    && let TyKind::Adt(d, _) = tcx.kind(i.self_ty)
                {
                    impls.insert(d);
                }
            }
        }
        DropInfo { impls, cache: Mutex::new(FastMap::default()) }
    }

    /// Whether ADT `d` has a `Drop` impl.
    pub fn has_impl(&self, d: DefId) -> bool {
        self.impls.contains(&d)
    }

    /// Whether dropping a value of `t` (concrete) runs any code.
    pub fn needs_drop(&self, tcx: &TyCtxt, typeck: &TypeckResults, solver: &mut Solver, t: Ty) -> bool {
        self.needs(tcx, typeck, solver, &mut FastSet::default(), t)
    }

    fn needs(&self, tcx: &TyCtxt, typeck: &TypeckResults, solver: &mut Solver, busy: &mut FastSet<Ty>, t: Ty) -> bool {
        if let Some(&b) = self.cache.lock().unwrap().get(&t) {
            return b;
        }
        // A type that holds itself does so through a pointer, which does
        // not look inside; while one is being worked out, it does not count.
        if !busy.insert(t) {
            return false;
        }
        let b = match tcx.kind(t) {
            TyKind::Adt(d, _) if self.impls.contains(&d) => true,
            TyKind::Adt(d, _) if tcx.lang.get("owned_box") == Some(&d) => true,
            TyKind::Adt(d, _) if tcx.lang.get("manually_drop") == Some(&d) => false,
            TyKind::Adt(d, _) if tcx.adts.get(&d).is_some_and(|a| a.kind == AdtKind::Union) => false,
            TyKind::Dyn(_) => true,
            _ => parts(tcx, typeck, solver, t).into_iter().any(|x| self.needs(tcx, typeck, solver, busy, x)),
        };
        busy.remove(&t);
        self.cache.lock().unwrap().insert(t, b);
        b
    }
}

/// The types a value of `t` holds by value and drops with it: each field
/// of each variant, each element, each capture taken by value. A `Box`
/// holds its pointee too. Nothing for types that hold nothing.
pub fn parts(tcx: &TyCtxt, typeck: &TypeckResults, solver: &mut Solver, t: Ty) -> Vec<Ty> {
    match tcx.kind(t) {
        TyKind::Tuple(l) => tcx.tys(l),
        TyKind::Array(e, _) | TyKind::Slice(e) | TyKind::Pat(e, ..) => vec![e],
        TyKind::Adt(d, l) if tcx.lang.get("owned_box") == Some(&d) => {
            let mut v: Vec<Ty> = tcx.tys(l).first().copied().into_iter().collect();
            v.extend(adt_fields(tcx, typeck, solver, d, l).into_iter().flatten());
            v
        }
        TyKind::Adt(d, l) => adt_fields(tcx, typeck, solver, d, l).into_iter().flatten().collect(),
        TyKind::Closure(c, _, l) => upvar_tys(tcx, typeck, solver, c, l).unwrap_or_default(),
        _ => Vec::new(),
    }
}

/// The type of each field of each variant of ADT `d` at `l`.
pub fn adt_fields(tcx: &TyCtxt, typeck: &TypeckResults, solver: &mut Solver, d: DefId, l: List) -> Vec<Vec<Ty>> {
    let Some(adt) = tcx.adts.get(&d) else { return Vec::new() };
    let params = tcx.all_params(d);
    let args = tcx.tys(l);
    adt.variants
        .iter()
        .map(|v| v.fields.iter().map(|f| crate::mono::sub(tcx, &typeck.hidden, solver, f.ty, &params, &args)).collect())
        .collect()
}

/// The types of what closure `c` (with its owner's arguments `l`)
/// captures: a reference to each local captured by reference, the local's
/// type for one captured by value.
pub fn upvar_tys(tcx: &TyCtxt, typeck: &TypeckResults, solver: &mut Solver, c: ClosureId, l: List) -> Option<Vec<Ty>> {
    let caps = typeck.captures.get(&c)?;
    let owner = *typeck.closure_owners.get(&c)?;
    let (k, f, _) = tcx.closure_at(c);
    let params = tcx.all_params(owner);
    let args = tcx.tys(l);
    caps.iter()
        .map(|cap| {
            let t = (*typeck.types.get(k as usize)?.get(f as usize)?.get(cap.local.0 as usize)?)?;
            let t = crate::mono::sub(tcx, &typeck.hidden, solver, t, &params, &args);
            Some(if cap.by_value { t } else { tcx.mk(TyKind::Ref(crate::ty::Mutbl::Mut, t)) })
        })
        .collect()
}
