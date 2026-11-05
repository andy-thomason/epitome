//! Type inference for one body: type variables and unification.
//!
//! A variable is a `TyKind::Infer(n)`, numbered within the body. It is
//! unknown (general, or known to be an integer or a float, from a literal)
//! or bound to a type, which may be another variable. Unifying two types
//! makes them equal, binding variables as it must; it fails on a mismatch.

use crate::ty::{Const, List, ParamId, Ty, TyCtxt, TyKind};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum VarKind {
    General,
    Int,
    Float,
}

#[derive(Clone, Copy, Debug)]
enum Var {
    Unknown(VarKind),
    Known(Ty),
}

pub struct Infer<'t, 's> {
    pub tcx: &'t TyCtxt<'s>,
    vars: Vec<Var>,
    /// Bindings made, with what each variable was before, to undo a probe.
    log: Vec<(u32, Var)>,
}

/// A point to roll inference back to.
#[derive(Clone, Copy)]
pub struct Snapshot {
    vars: usize,
    log: usize,
}

impl<'t, 's> Infer<'t, 's> {
    pub fn new(tcx: &'t TyCtxt<'s>) -> Self {
        Infer { tcx, vars: Vec::new(), log: Vec::new() }
    }

    pub fn snapshot(&self) -> Snapshot {
        Snapshot { vars: self.vars.len(), log: self.log.len() }
    }

    /// Undoes every binding (and new variable) since `s`.
    pub fn rollback(&mut self, s: Snapshot) {
        while self.log.len() > s.log {
            let (n, old) = self.log.pop().unwrap();
            self.vars[n as usize] = old;
        }
        self.vars.truncate(s.vars);
    }

    pub fn var(&mut self, kind: VarKind) -> Ty {
        let n = self.vars.len() as u32;
        self.vars.push(Var::Unknown(kind));
        self.tcx.mk(TyKind::Infer(n))
    }

    pub fn fresh(&mut self) -> Ty {
        self.var(VarKind::General)
    }

    /// `t` with any bound variable at its top replaced by what it is bound
    /// to.
    pub fn shallow(&self, mut t: Ty) -> Ty {
        while let TyKind::Infer(n) = self.tcx.kind(t) {
            match self.vars.get(n as usize) {
                Some(Var::Known(u)) => t = *u,
                Some(Var::Unknown(_)) => break,
                // Not one of this body's variables (a `_` left in a
                // signature): treated as an error.
                None => return self.tcx.error(),
            }
        }
        t
    }

    /// The kind of an unbound variable `t` (after `shallow`), if it is one.
    pub fn var_kind(&self, t: Ty) -> Option<VarKind> {
        match self.tcx.kind(t) {
            TyKind::Infer(n) => match self.vars.get(n as usize) {
                Some(Var::Unknown(k)) => Some(*k),
                _ => None,
            },
            _ => None,
        }
    }

    /// `t` with every bound variable inside it replaced.
    pub fn resolve(&self, t: Ty) -> Ty {
        let t = self.shallow(t);
        let tcx = self.tcx;
        let list = |l: List| tcx.list(tcx.tys(l).into_iter().map(|x| self.resolve(x)).collect());
        match tcx.kind(t) {
            TyKind::Tuple(l) => tcx.mk(TyKind::Tuple(list(l))),
            TyKind::Adt(d, l) => tcx.mk(TyKind::Adt(d, list(l))),
            TyKind::FnDef(d, l) => tcx.mk(TyKind::FnDef(d, list(l))),
            TyKind::Projection(d, l) => tcx.mk(TyKind::Projection(d, list(l))),
            TyKind::Closure(c, s) => {
                let mut sig = tcx.sig_of(s);
                sig.inputs = list(sig.inputs);
                sig.output = self.resolve(sig.output);
                tcx.mk(TyKind::Closure(c, tcx.sig(sig)))
            }
            TyKind::Ref(m, x) => tcx.mk(TyKind::Ref(m, self.resolve(x))),
            TyKind::Ptr(m, x) => tcx.mk(TyKind::Ptr(m, self.resolve(x))),
            TyKind::Slice(x) => tcx.mk(TyKind::Slice(self.resolve(x))),
            TyKind::Array(x, c) => tcx.mk(TyKind::Array(self.resolve(x), c)),
            TyKind::FnPtr(s) => {
                let mut sig = tcx.sig_of(s);
                sig.inputs = list(sig.inputs);
                sig.output = self.resolve(sig.output);
                tcx.mk(TyKind::FnPtr(tcx.sig(sig)))
            }
            _ => t,
        }
    }

    /// Integer and float variables left unbound become `i32` and `f64`.
    pub fn fallback(&mut self) {
        for i in 0..self.vars.len() {
            if let Var::Unknown(k) = self.vars[i] {
                let t = match k {
                    VarKind::Int => self.tcx.mk(TyKind::Int(crate::ty::IntTy::I32)),
                    VarKind::Float => self.tcx.mk(TyKind::Float(crate::ty::FloatTy::F64)),
                    VarKind::General => continue,
                };
                self.vars[i] = Var::Known(t);
            }
        }
    }

    /// Whether a type is (or may become) an integer.
    pub fn is_integral(&self, t: Ty) -> bool {
        let t = self.shallow(t);
        matches!(self.tcx.kind(t), TyKind::Int(_) | TyKind::Uint(_)) || self.var_kind(t) == Some(VarKind::Int)
    }

    /// Whether a type is (or may become) a number.
    pub fn is_numeric(&self, t: Ty) -> bool {
        let t = self.shallow(t);
        self.is_integral(t) || matches!(self.tcx.kind(t), TyKind::Float(_)) || self.var_kind(t) == Some(VarKind::Float)
    }

    fn bind(&mut self, n: u32, t: Ty) -> bool {
        let kind = match self.vars[n as usize] {
            Var::Unknown(k) => k,
            Var::Known(_) => unreachable!(),
        };
        let ok = match (kind, self.tcx.kind(t)) {
            (VarKind::General, _) => true,
            (VarKind::Int, TyKind::Int(_) | TyKind::Uint(_)) => true,
            (VarKind::Float, TyKind::Float(_)) => true,
            (_, TyKind::Error) => true,
            (_, TyKind::Infer(m)) => {
                // Two variables: the kinds must agree; keep the stricter.
                let other = match self.vars[m as usize] {
                    Var::Unknown(k) => k,
                    Var::Known(_) => unreachable!(),
                };
                match (kind, other) {
                    (a, b) if a == b => true,
                    (VarKind::General, _) => true,
                    (k, VarKind::General) => {
                        self.log.push((m, self.vars[m as usize]));
                        self.vars[m as usize] = Var::Unknown(k);
                        true
                    }
                    _ => false,
                }
            }
            _ => false,
        };
        if ok {
            self.log.push((n, self.vars[n as usize]));
            self.vars[n as usize] = Var::Known(t);
        }
        ok
    }

    /// Makes `a` and `b` the same type, or says they cannot be.
    pub fn unify(&mut self, a: Ty, b: Ty) -> bool {
        let a = self.shallow(a);
        let b = self.shallow(b);
        if a == b {
            return true;
        }
        let tcx = self.tcx;
        match (tcx.kind(a), tcx.kind(b)) {
            (TyKind::Error, _) | (_, TyKind::Error) => true,
            (TyKind::Infer(n), _) => self.bind(n, b),
            (_, TyKind::Infer(n)) => self.bind(n, a),
            (TyKind::Tuple(x), TyKind::Tuple(y)) => self.unify_lists(x, y),
            (TyKind::Adt(d, x), TyKind::Adt(e, y)) | (TyKind::FnDef(d, x), TyKind::FnDef(e, y)) => {
                d == e && self.unify_lists(x, y)
            }
            (TyKind::Closure(c, s), TyKind::Closure(d, t)) => {
                let (s, t) = (tcx.sig_of(s), tcx.sig_of(t));
                c == d && self.unify_lists(s.inputs, t.inputs) && self.unify(s.output, t.output)
            }
            (TyKind::Projection(d, x), TyKind::Projection(e, y)) => d == e && self.unify_lists(x, y),
            (TyKind::Ref(m, x), TyKind::Ref(n, y)) | (TyKind::Ptr(m, x), TyKind::Ptr(n, y)) => m == n && self.unify(x, y),
            (TyKind::Slice(x), TyKind::Slice(y)) => self.unify(x, y),
            (TyKind::Array(x, c), TyKind::Array(y, d)) => self.consts_agree(c, d) && self.unify(x, y),
            (TyKind::FnPtr(s), TyKind::FnPtr(t)) => {
                let (s, t) = (tcx.sig_of(s), tcx.sig_of(t));
                s.unsafety == t.unsafety && self.unify_lists(s.inputs, t.inputs) && self.unify(s.output, t.output)
            }
            (TyKind::ConstArg(c), TyKind::ConstArg(d)) => self.consts_agree(c, d),
            _ => false,
        }
    }

    fn unify_lists(&mut self, x: List, y: List) -> bool {
        if x == y {
            return true;
        }
        let (xs, ys) = (self.tcx.tys(x), self.tcx.tys(y));
        xs.len() == ys.len() && xs.into_iter().zip(ys).all(|(a, b)| self.unify(a, b))
    }

    /// Two constants that are not known to differ (an expression not yet
    /// evaluated agrees with anything).
    fn consts_agree(&self, c: crate::ty::ConstId, d: crate::ty::ConstId) -> bool {
        match (self.tcx.const_of(c), self.tcx.const_of(d)) {
            (Const::Value(a), Const::Value(b)) => a == b,
            (Const::Param(a), Const::Param(b)) => a == b,
            _ => true,
        }
    }

    /// Fresh variables for the parameters of an item (its parent's too).
    pub fn instantiate(&mut self, params: &[ParamId]) -> Vec<Ty> {
        params.iter().map(|_| self.fresh()).collect()
    }
}
