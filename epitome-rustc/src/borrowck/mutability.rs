//! Mutability: what is borrowed with `&mut` or assigned to, and is not
//! mutable (E0596, E0594).
//!
//! rustc's rule for whether a place can change:
//!
//! - a local only if it is declared `mut` (a temporary always is);
//! - a field, an array element: if what it is part of is;
//! - `*r` of a `&mut T`: yes, however `r` is declared (it need not be `mut` to
//!   be written through), but not if `r` itself is reached through a `&`;
//! - `*r` of a `&T` or a `*const T`: never;
//! - `*p` of a `*mut T`: always;
//! - the contents of a `Box`: if the box is.
//!
//! A variable assigned *as a whole* is E0384, which `moves` has: whether it
//! has a value already is what matters there.
//!
//! What a closure has captured is not asked (the graph does not say whether
//! it is changed through), so a closure body is not checked for it.

use super::cfg::*;
use super::moves::Raw;
use crate::ast::NodeId;
use crate::hash::FastSet;
use crate::ty::{ClosureId, Mutbl, Ty, TyCtxt, TyKind};

struct An<'a, 's> {
    cfg: &'a Cfg,
    tcx: &'a TyCtxt<'s>,
    fn_mut: &'a FastSet<ClosureId>,
    out: Vec<Raw>,
}

/// The closures whose bodies change what they captured: `FnMut` (or `FnOnce`),
/// so that calling one needs it to be mutable.
pub fn mutating_closures(cfgs: &[Cfg]) -> FastSet<ClosureId> {
    let mut out = FastSet::default();
    for cfg in cfgs {
        let Some(c) = cfg.closure else { continue };
        let in_env = |p: &Place| p.local == Local(1) && matches!(p.proj.first(), Some(Proj::Field(..)));
        let changes = cfg.reachable().into_iter().any(|b| {
            cfg.blocks[b.0 as usize].stmts.iter().any(|s| match &s.kind {
                StmtKind::Assign(dest, rv) => {
                    (!dest.proj.is_empty() && in_env(dest)) || matches!(rv, Rvalue::Ref(Mutbl::Mut, p) | Rvalue::AddrOf(Mutbl::Mut, p) if in_env(p))
                }
                _ => false,
            })
        });
        if changes {
            out.insert(c);
        }
    }
    out
}

/// The mutability errors of a body.
pub fn check(cfg: &Cfg, tcx: &TyCtxt, fn_mut: &FastSet<ClosureId>) -> Vec<Raw> {
    let mut an = An { cfg, tcx, fn_mut, out: Vec::new() };
    for b in cfg.reachable() {
        let data = &cfg.blocks[b.0 as usize];
        for s in &data.stmts {
            if let StmtKind::Assign(dest, rv) = &s.kind {
                an.stmt(dest, rv, s.node);
            }
        }
        // A closure that changes what it captured, called: it is borrowed
        // mutably to be.
        if let Some((Term::Call { func: Callee::Value(Operand::Use(p)), .. }, node)) = &data.term
            && let TyKind::Closure(c, _, _) = tcx.kind(p.ty)
            && an.fn_mut.contains(&c)
            && !an.can_change(p)
        {
            an.err("E0596", p.at.unwrap_or(*node), "cannot borrow as mutable");
        }
    }
    let mut seen = std::collections::HashSet::new();
    an.out.retain(|r| seen.insert((r.code, r.node)));
    an.out
}

impl<'a, 's> An<'a, 's> {
    fn pointee(&self, t: Ty) -> Option<Ty> {
        match self.tcx.kind(t) {
            TyKind::Ref(_, x) | TyKind::Ptr(_, x) => Some(x),
            TyKind::Adt(d, l) if self.tcx.lang.get("owned_box") == Some(&d) => self.tcx.tys(l).first().copied(),
            _ => None,
        }
    }

    fn is_box(&self, t: Ty) -> bool {
        matches!(self.tcx.kind(t), TyKind::Adt(d, _) if self.tcx.lang.get("owned_box") == Some(&d))
    }

    /// The type after `projs`, starting from `ty`.
    fn type_after(&self, mut ty: Ty, projs: &[Proj]) -> Ty {
        for pr in projs {
            ty = match pr {
                Proj::Field(_, t) => *t,
                Proj::Deref => self.pointee(ty).unwrap_or(ty),
                Proj::Index(_, e) | Proj::ConstIndex(_, _, e) => *e,
                Proj::Subslice(_, _, _, e) => self.tcx.mk(TyKind::Slice(*e)),
                Proj::Downcast(_) => ty,
            };
        }
        ty
    }

    /// Whether what `projs` pick out of a variable of type `ty` (`var_mut`:
    /// declared `mut`, or a temporary) can be changed. `local_ok`: the
    /// variable need not be `mut` (the place is only being written through).
    fn mutable(&self, ty: Ty, var_mut: bool, projs: &[Proj], local_ok: bool) -> bool {
        let Some((last, init)) = projs.split_last() else { return var_mut || local_ok };
        match last {
            Proj::Field(..) | Proj::Downcast(_) | Proj::Index(..) | Proj::ConstIndex(..) | Proj::Subslice(..) => self.mutable(ty, var_mut, init, local_ok),
            Proj::Deref => {
                let base = self.type_after(ty, init);
                match self.tcx.kind(base) {
                    TyKind::Ref(Mutbl::Mut, _) => self.mutable(ty, var_mut, init, true),
                    TyKind::Ref(Mutbl::Not, _) | TyKind::Ptr(Mutbl::Not, _) => false,
                    TyKind::Ptr(Mutbl::Mut, _) => true,
                    _ if self.is_box(base) => self.mutable(ty, var_mut, init, local_ok),
                    // Not known: not an error.
                    _ => true,
                }
            }
        }
    }

    /// What a closure captured is the variable it came from: the first field
    /// of the environment, and (if held by reference) the `*` after it, are
    /// the variable; whether it is `mut` is what it was declared.
    fn can_change(&self, p: &Place) -> bool {
        if self.cfg.closure.is_some()
            && p.local == Local(1)
            && let Some(Proj::Field(i, _)) = p.proj.first()
            && let Some(&(var_mut, by_value, ty)) = self.cfg.upvars.get(*i as usize)
        {
            let rest = if by_value { &p.proj[1..] } else { p.proj.get(2..).unwrap_or(&[]) };
            return self.mutable(ty, var_mut, rest, false);
        }
        let d = &self.cfg.locals[p.local.0 as usize];
        let var_mut = match d.kind {
            LocalKind::Temp | LocalKind::Return => true,
            LocalKind::Var | LocalKind::Arg | LocalKind::Static => d.mutable,
        };
        self.mutable(d.ty, var_mut, &p.proj, false)
    }

    fn err(&mut self, code: &'static str, node: NodeId, message: &str) {
        self.out.push(Raw { code, node, message: message.to_string(), local: None });
    }

    fn stmt(&mut self, dest: &Place, rv: &Rvalue, node: NodeId) {
        // `&mut p`, `&raw mut p`: p must be able to change.
        if let Rvalue::Ref(Mutbl::Mut, p) | Rvalue::AddrOf(Mutbl::Mut, p) = rv
            && !self.can_change(p)
        {
            self.err("E0596", p.at.unwrap_or(node), "cannot borrow as mutable");
        }
        // `p.f = v`, `*r = v`, `a[i] = v`: p must be able to change. (A whole
        // variable is E0384, in `moves`; a whole static is here.)
        let whole_static = dest.proj.is_empty() && self.cfg.locals[dest.local.0 as usize].kind == LocalKind::Static;
        if (!dest.proj.is_empty() || whole_static) && !self.can_change(dest) {
            self.err("E0594", dest.at.unwrap_or(node), "cannot assign");
        }
    }
}
