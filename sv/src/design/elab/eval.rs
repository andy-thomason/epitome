//! Constant evaluation (11.2.1), including constant functions (13.4.3).
//!
//! Expressions are evaluated on the design IR, after binding, so constant
//! evaluation and simulation share clause 11's rules through `Bits`. A
//! constant function's body is run by a small interpreter over the IR's
//! statements, with its arguments and locals in a frame.

use super::{Elab, R, expr};
use crate::ast::{Kind, NodeId};
use crate::design::types::{Type, TypeId, Types, UDim};
use crate::design::value::{Bits, Logic};
use crate::design::*;
use std::collections::HashMap;

/// Binds and evaluates a constant expression.
pub(crate) fn const_expr<'a>(e: &mut Elab<'a, '_>, sc: ScopeId, n: NodeId) -> R<'a, Const> {
    let x = expr::expr(e, sc, n)?;
    let mut ev = Eval { frames: Vec::new(), steps: 0, node: n };
    match ev.expr(e, x) {
        Ok(v) => Ok(v),
        Err(Stop::Error(err)) => Err(err),
        Err(_) => Err(e.err(n, "this is not a constant expression")),
    }
}

/// The type a parameter takes from its value when declared without one
/// (6.20.2).
pub(crate) fn type_of_const(e: &mut Elab, v: &Const) -> TypeId {
    match v {
        Const::Bits(b) => {
            if b.width() == 32 && b.signed() {
                Types::INT
            } else {
                e.d.types.vector(b.width().max(1), b.signed(), !b.is_known())
            }
        }
        Const::Real(_) => Types::REAL,
        Const::Str(s) => e.d.types.vector((s.len() as u32 * 8).max(8), false, false),
        Const::Type(_) => Types::TYPE,
        Const::Null => Types::NULL,
        Const::Unbounded => Types::INT,
        Const::Aggregate(_) => Types::UNKNOWN,
    }
}

/// A constant converted to type `t`, as assignment converts (10.7).
pub(crate) fn convert_const(e: &Elab, v: &Const, t: TypeId) -> Const {
    let ty = &e.d.types;
    if ty.is_integral(t) {
        let w = ty.width(t);
        let s = ty.is_signed(t);
        let four = ty.is_four_state(t);
        let b = match v {
            Const::Bits(b) => b.resize(w, b.signed()).with_signed(s),
            Const::Real(r) => Bits::from_f64(w, s, *r),
            Const::Str(st) => Bits::from_str_bytes(st).resize(w, false).with_signed(s),
            _ => return v.clone(),
        };
        let b = if four { b } else { two_state(&b) };
        return Const::Bits(b);
    }
    if ty.is_real(t) {
        return match v {
            Const::Bits(b) => Const::Real(b.to_f64()),
            Const::Real(r) => {
                if matches!(ty.get(t), Type::Real(crate::design::types::RealKind::ShortReal)) {
                    Const::Real(*r as f32 as f64)
                } else {
                    Const::Real(*r)
                }
            }
            _ => v.clone(),
        };
    }
    if t == Types::STRING {
        return match v {
            Const::Bits(b) => Const::Str(b.to_str_bytes()),
            _ => v.clone(),
        };
    }
    v.clone()
}

/// x and z become 0 (6.3.2.1).
pub(crate) fn two_state(b: &Bits) -> Bits {
    let mut r = b.clone();
    for i in 0..b.width() {
        if !b.bit(i).is_known() {
            r.set_bit(i, Logic::Zero);
        }
    }
    r
}

/// Equality for a generate `case` item: bit for bit (27.5).
pub(crate) fn const_eq(a: &Const, b: &Const) -> bool {
    match (a, b) {
        (Const::Bits(x), Const::Bits(y)) => x.case_eq(y),
        (Const::Real(x), Const::Real(y)) => x == y,
        (Const::Bits(x), Const::Real(y)) | (Const::Real(y), Const::Bits(x)) => x.to_f64() == *y,
        (Const::Str(x), Const::Str(y)) => x == y,
        (Const::Type(x), Const::Type(y)) => x == y,
        _ => a == b,
    }
}

/// A time literal's value in the time unit of the scope it is written in
/// (3.14).
pub(crate) fn time_literal(e: &Elab, _sc: ScopeId, n: NodeId) -> f64 {
    let t = e.text(n);
    let split = t.find(|c: char| c.is_ascii_alphabetic()).unwrap_or(t.len());
    let v: f64 = t[..split].replace('_', "").parse().unwrap_or(0.0);
    let exp = match &t[split..] {
        "s" => 0,
        "ms" => -3,
        "us" => -6,
        "ns" => -9,
        "ps" => -12,
        "fs" => -15,
        _ => return v,
    };
    let unit = e.state(n).timescale.map_or(-9, |ts| ts.unit as i32);
    v * 10f64.powi(exp - unit)
}

/// Why evaluation stopped early.
pub(crate) enum Stop<'a> {
    Error(crate::source::Error<'a>),
    NotConstant,
    Return(Const),
    Break,
    Continue,
}

type V<'a> = Result<Const, Stop<'a>>;

pub(crate) struct Eval {
    pub frames: Vec<HashMap<ObjId, Const>>,
    pub steps: u64,
    pub node: NodeId,
}

fn bits_of(c: &Const) -> Option<Bits> {
    match c {
        Const::Bits(b) => Some(b.clone()),
        Const::Real(r) => Some(Bits::from_f64(64, true, *r)),
        Const::Str(s) => Some(Bits::from_str_bytes(s)),
        _ => None,
    }
}

/// The default value of a variable of type `t` (6.8, table 6-7).
pub(crate) fn default_value(e: &Elab, t: TypeId) -> Const {
    let ty = &e.d.types;
    if ty.is_integral(t) {
        let w = ty.width(t);
        let s = ty.is_signed(t);
        return Const::Bits(if ty.is_four_state(t) { Bits::x(w).with_signed(s) } else { Bits::zero(w, s) });
    }
    match ty.get(t) {
        Type::Real(_) => Const::Real(0.0),
        Type::String => Const::Str(vec![]),
        Type::Unpacked { elem, dim: UDim::Range(l, r) } => {
            let n = (l - r).unsigned_abs() as usize + 1;
            let d = default_value(e, *elem);
            Const::Aggregate(vec![d; n.min(1 << 20)])
        }
        Type::Unpacked { .. } => Const::Aggregate(vec![]),
        Type::Struct { members, .. } => Const::Aggregate(members.iter().map(|m| default_value(e, m.ty)).collect()),
        Type::Class(_) | Type::Null | Type::Chandle => Const::Null,
        _ => Const::Bits(Bits::x(1)),
    }
}

impl Eval {
    fn err<'a>(&self, e: &Elab<'a, '_>, n: NodeId, msg: impl Into<String>) -> Stop<'a> {
        let _ = self.node;
        Stop::Error(e.err(n, msg))
    }

    pub(crate) fn expr<'a>(&mut self, e: &mut Elab<'a, '_>, x: ExprId) -> V<'a> {
        self.steps += 1;
        if self.steps > 50_000_000 {
            let n = e.ex(x).node;
            return Err(self.err(e, n, "constant evaluation takes too long; is there an endless loop?"));
        }
        let ex = e.ex(x).clone();
        let n = ex.node;
        let ty = ex.ty;
        use ExprKind as K;
        match &ex.kind {
            K::Const(c) => Ok(e.d.consts[c.0 as usize].clone()),
            K::Object(o) => self.read(e, *o, n),
            K::Hier { object: Some(o), .. } => {
                let obj = e.obj(*o);
                if matches!(obj.kind, ObjKind::Param { .. } | ObjKind::EnumConst) {
                    return self.read(e, *o, n);
                }
                Err(self.err(e, n, "a hierarchical reference is not a constant expression (11.2.1)"))
            }
            K::Hier { .. } => Err(self.err(e, n, "a hierarchical reference is not a constant expression (11.2.1)")),
            K::TypeRef(t) => Ok(Const::Type(*t)),
            K::Convert => {
                let v = self.expr(e, ex.args[0])?;
                Ok(convert_const(e, &v, ty))
            }
            K::Unary(op @ (UnOp::PreInc | UnOp::PreDec | UnOp::PostInc | UnOp::PostDec)) => {
                let old = self.expr(e, ex.args[0])?;
                let one = Const::Bits(Bits::from_i64(32, true, 1));
                let bop = if matches!(op, UnOp::PreInc | UnOp::PostInc) { BinOp::Add } else { BinOp::Sub };
                let new = self.binary(e, bop, old.clone(), one, ty, n)?;
                self.store(e, ex.args[0], new.clone())?;
                Ok(if matches!(op, UnOp::PreInc | UnOp::PreDec) { new } else { old })
            }
            K::Unary(op) => {
                let v = self.expr(e, ex.args[0])?;
                self.unary(e, *op, v, ty, ex.args[0], n)
            }
            K::Binary(op) => {
                // Short-circuit `&&` and `||` (11.4.7).
                if matches!(op, BinOp::LogAnd | BinOp::LogOr) {
                    let a = self.expr(e, ex.args[0])?;
                    let at = truth(&a);
                    if *op == BinOp::LogAnd && at == Some(false) {
                        return Ok(Const::Bits(Bits::from_bool(false)));
                    }
                    if *op == BinOp::LogOr && at == Some(true) {
                        return Ok(Const::Bits(Bits::from_bool(true)));
                    }
                    let b = self.expr(e, ex.args[1])?;
                    let bt = truth(&b);
                    let r = match (op, at, bt) {
                        (BinOp::LogAnd, _, Some(false)) => Logic::Zero,
                        (BinOp::LogAnd, Some(true), Some(true)) => Logic::One,
                        (BinOp::LogOr, _, Some(true)) => Logic::One,
                        (BinOp::LogOr, Some(false), Some(false)) => Logic::Zero,
                        _ => Logic::X,
                    };
                    return Ok(Const::Bits(Bits::of_logic(r)));
                }
                let a = self.expr(e, ex.args[0])?;
                let b = self.expr(e, ex.args[1])?;
                self.binary(e, *op, a, b, ty, n)
            }
            K::Cond => {
                let c = self.expr(e, ex.args[0])?;
                match truth(&c) {
                    Some(true) => self.expr(e, ex.args[1]),
                    Some(false) => self.expr(e, ex.args[2]),
                    None => {
                        let a = self.expr(e, ex.args[1])?;
                        let b = self.expr(e, ex.args[2])?;
                        match (bits_of(&a), bits_of(&b)) {
                            (Some(a), Some(b)) => {
                                // Bits that agree are kept, others are x (11.4.11).
                                let mut r = a.clone();
                                for i in 0..r.width() {
                                    if a.bit(i) != b.bit(i) || !a.bit(i).is_known() {
                                        r.set_bit(i, Logic::X);
                                    }
                                }
                                Ok(Const::Bits(r))
                            }
                            _ => Ok(a),
                        }
                    }
                }
            }
            K::Concat => {
                if ty == Types::STRING {
                    let mut s = Vec::new();
                    for &a in &ex.args {
                        match self.expr(e, a)? {
                            Const::Str(x) => s.extend(x),
                            Const::Bits(b) => s.extend(b.to_str_bytes()),
                            _ => return Err(Stop::NotConstant),
                        }
                    }
                    return Ok(Const::Str(s));
                }
                if e.d.types.is_unpacked_array(ty) {
                    let mut out = Vec::new();
                    for &a in &ex.args {
                        match self.expr(e, a)? {
                            Const::Aggregate(v) if e.d.types.is_unpacked_array(e.ex(a).ty) => out.extend(v),
                            v => out.push(v),
                        }
                    }
                    return Ok(Const::Aggregate(out));
                }
                let mut parts = Vec::new();
                for &a in &ex.args {
                    let v = self.expr(e, a)?;
                    let at = e.ex(a).ty;
                    let w = e.d.types.width(at);
                    let b = bits_of(&v).ok_or(Stop::NotConstant)?;
                    parts.push(if w > 0 { b.resize(w, b.signed()) } else { b });
                }
                Ok(Const::Bits(Bits::concat(&parts)))
            }
            K::Replicate(k) => {
                let v = self.expr(e, ex.args[0])?;
                if let Const::Str(s) = &v {
                    return Ok(Const::Str(s.repeat(*k as usize)));
                }
                let b = bits_of(&v).ok_or(Stop::NotConstant)?;
                let parts = vec![b; *k as usize];
                Ok(Const::Bits(Bits::concat(&parts)))
            }
            K::Index => {
                let base = self.expr(e, ex.args[0])?;
                let i = self.expr(e, ex.args[1])?;
                let bt = e.ex(ex.args[0]).ty;
                self.index(e, &base, bt, &i, ty, n)
            }
            K::Part { kind, width } => {
                let base = self.expr(e, ex.args[0])?;
                let a = self.expr(e, ex.args[1])?.to_i64();
                let bt = e.ex(ex.args[0]).ty;
                let Some(a) = a else { return Ok(Const::Bits(Bits::x(*width))) };
                let (hi, lo) = match kind {
                    PartKind::Range => {
                        let b = self.expr(e, ex.args[2])?.to_i64().unwrap_or(0);
                        (a, b)
                    }
                    PartKind::Up => (a + *width as i64 - 1, a),
                    PartKind::Down => (a, a - *width as i64 + 1),
                };
                if let Const::Aggregate(items) = &base {
                    let (l, r) = match e.d.types.get(bt) {
                        Type::Unpacked { dim: UDim::Range(l, r), .. } => (*l, *r),
                        _ => (0, items.len() as i64 - 1),
                    };
                    let pos = |i: i64| if l >= r { i - r } else { r - i };
                    let _ = pos;
                    let off = |i: i64| if l <= r { i - l } else { l - i };
                    let (a2, b2) = (off(hi).min(off(lo)), off(hi).max(off(lo)));
                    let slice: Vec<Const> = (a2..=b2).filter_map(|i| items.get(i as usize).cloned()).collect();
                    return Ok(Const::Aggregate(slice));
                }
                let b = bits_of(&base).ok_or(Stop::NotConstant)?;
                let ew = e.d.types.element(bt).map_or(1, |t| e.d.types.width(t).max(1)) as i64;
                let lo_off = lsb_offset(e, bt, lo.min(hi), hi.max(lo));
                match lo_off {
                    Some(off) => Ok(Const::Bits(b.slice((off * ew).max(0) as u32, width * ew as u32 / ew as u32 * ew as u32))),
                    None => Ok(Const::Bits(Bits::x(*width))),
                }
            }
            K::Member(i) => {
                let base = self.expr(e, ex.args[0])?;
                let bt = e.ex(ex.args[0]).ty;
                match base {
                    Const::Aggregate(v) => v.get(*i as usize).cloned().ok_or(Stop::NotConstant),
                    Const::Bits(b) => {
                        // A packed struct: members from the most significant.
                        let Type::Struct { members, union, .. } = e.d.types.get(bt).clone() else {
                            return Err(Stop::NotConstant);
                        };
                        let w = e.d.types.width(members[*i as usize].ty);
                        if union {
                            return Ok(Const::Bits(b.slice(0, w)));
                        }
                        let below: u32 = members[*i as usize + 1..].iter().map(|m| e.d.types.width(m.ty)).sum();
                        let s = e.d.types.is_signed(members[*i as usize].ty);
                        Ok(Const::Bits(b.slice(below, w).with_signed(s)))
                    }
                    _ => Err(Stop::NotConstant),
                }
            }
            K::Pattern => {
                let mut out = Vec::new();
                for &a in &ex.args {
                    out.push(self.expr(e, a)?);
                }
                if e.d.types.is_integral(ty) {
                    // A packed target: concatenate the elements.
                    let mut parts = Vec::new();
                    let elem_tys: Vec<TypeId> = match e.d.types.get(ty).clone() {
                        Type::Struct { members, .. } => members.iter().map(|m| m.ty).collect(),
                        _ => vec![],
                    };
                    for (i, v) in out.iter().enumerate() {
                        let b = bits_of(v).ok_or(Stop::NotConstant)?;
                        let w = elem_tys.get(i).map(|&t| e.d.types.width(t)).unwrap_or(b.width());
                        parts.push(b.resize(w, b.signed()));
                    }
                    return Ok(Const::Bits(Bits::concat(&parts).resize(e.d.types.width(ty), false)));
                }
                Ok(Const::Aggregate(out))
            }
            K::Call(sub) => self.call(e, *sub, &ex.args, n),
            K::SysCall(s) => self.syscall(e, e.name(*s), &ex.args, ty, n),
            K::Method(m) => self.method(e, e.name(*m), &ex.args, n),
            K::Inside => {
                let v = self.expr(e, ex.args[0])?;
                let vb = bits_of(&v);
                let mut any_x = false;
                for &r in &ex.args[1..] {
                    let rr = e.ex(r).clone();
                    if rr.kind == K::Range {
                        let lo = self.expr(e, rr.args[0])?;
                        let hi = self.expr(e, rr.args[1])?;
                        if let (Some(v), Some(lo), Some(hi)) = (vb.as_ref(), bits_of(&lo), bits_of(&hi)) {
                            let ge = v.cmp_op(&lo, |o| o.is_ge());
                            let le = v.cmp_op(&hi, |o| o.is_le());
                            if ge == Logic::One && le == Logic::One {
                                return Ok(Const::Bits(Bits::from_bool(true)));
                            }
                            any_x |= ge == Logic::X || le == Logic::X;
                        }
                    } else {
                        let c = self.expr(e, r)?;
                        if let Const::Aggregate(items) = &c {
                            if items.iter().any(|i| const_eq(i, &v)) {
                                return Ok(Const::Bits(Bits::from_bool(true)));
                            }
                            continue;
                        }
                        match (vb.as_ref(), bits_of(&c)) {
                            (Some(v), Some(c)) => match v.wildcard_eq(&c) {
                                Logic::One => return Ok(Const::Bits(Bits::from_bool(true))),
                                Logic::X => any_x = true,
                                _ => {}
                            },
                            _ => {
                                if const_eq(&v, &c) {
                                    return Ok(Const::Bits(Bits::from_bool(true)));
                                }
                            }
                        }
                    }
                }
                Ok(Const::Bits(if any_x { Bits::x(1) } else { Bits::from_bool(false) }))
            }
            K::Null => Ok(Const::Null),
            K::Assign => {
                let v = self.expr(e, ex.args[1])?;
                self.store(e, ex.args[0], v.clone())?;
                Ok(v)
            }
            // `$` as a parameter's value: unbounded (6.20.2, 20.6.3).
            K::Dollar => Ok(Const::Unbounded),
            K::Field(_) | K::New | K::This | K::Streaming { .. } | K::Range | K::Tagged(_) | K::Opaque => {
                Err(Stop::NotConstant)
            }
        }
    }

    fn read<'a>(&mut self, e: &mut Elab<'a, '_>, o: ObjId, n: NodeId) -> V<'a> {
        for f in self.frames.iter().rev() {
            if let Some(v) = f.get(&o) {
                return Ok(v.clone());
            }
        }
        let obj = e.obj(o);
        match obj.kind {
            ObjKind::Param { .. } | ObjKind::EnumConst | ObjKind::Genvar => match obj.value {
                Some(c) if c.0 != u32::MAX => Ok(e.d.consts[c.0 as usize].clone()),
                _ => Err(self.err(e, n, format!("`{}` has no value yet", e.name(obj.name)))),
            },
            _ => {
                if !self.frames.is_empty() {
                    let name = e.name(obj.name);
                    return Err(self.err(e, n, format!("a constant function cannot read `{name}`, which is not its own (13.4.3)")));
                }
                // `const` variables with constant initialisers are not
                // constant expressions either (6.20.6).
                Err(self.err(e, n, format!("`{}` is not a constant", e.name(obj.name))))
            }
        }
    }

    fn unary<'a>(&mut self, e: &Elab<'a, '_>, op: UnOp, v: Const, ty: TypeId, _a: ExprId, n: NodeId) -> V<'a> {
        if let Const::Real(r) = v {
            return Ok(match op {
                UnOp::Plus => Const::Real(r),
                UnOp::Neg => Const::Real(-r),
                UnOp::LogNot => Const::Bits(Bits::from_bool(r == 0.0)),
                _ => return Err(self.err(e, n, "this operator cannot take a real")),
            });
        }
        let b = bits_of(&v).ok_or(Stop::NotConstant)?;
        let w = e.d.types.width(ty).max(b.width());
        let s = e.d.types.is_signed(ty);
        let r = match op {
            UnOp::Plus => b,
            UnOp::Neg => b.resize(w, s).with_signed(s).neg(),
            UnOp::Not => b.not(),
            UnOp::LogNot => Bits::of_logic(match b.truth() {
                Some(t) => {
                    if t {
                        Logic::Zero
                    } else {
                        Logic::One
                    }
                }
                None => Logic::X,
            }),
            UnOp::RedAnd => Bits::of_logic(b.reduce_and()),
            UnOp::RedNand => Bits::of_logic(b.reduce_and()).not(),
            UnOp::RedOr => Bits::of_logic(b.reduce_or()),
            UnOp::RedNor => Bits::of_logic(b.reduce_or()).not(),
            UnOp::RedXor => Bits::of_logic(b.reduce_xor()),
            UnOp::RedXnor => Bits::of_logic(b.reduce_xor()).not(),
            _ => return Err(Stop::NotConstant),
        };
        Ok(Const::Bits(r))
    }

    fn binary<'a>(&mut self, e: &Elab<'a, '_>, op: BinOp, a: Const, b: Const, ty: TypeId, n: NodeId) -> V<'a> {
        use BinOp::*;
        // Reals.
        if matches!(a, Const::Real(_)) || matches!(b, Const::Real(_)) {
            let x = match &a {
                Const::Real(r) => *r,
                Const::Bits(b) => b.to_f64(),
                _ => return Err(Stop::NotConstant),
            };
            let y = match &b {
                Const::Real(r) => *r,
                Const::Bits(b) => b.to_f64(),
                _ => return Err(Stop::NotConstant),
            };
            let bl = |v: bool| Const::Bits(Bits::from_bool(v));
            return Ok(match op {
                Add => Const::Real(x + y),
                Sub => Const::Real(x - y),
                Mul => Const::Real(x * y),
                Div => Const::Real(x / y),
                Pow => Const::Real(x.powf(y)),
                Eq | CaseEq => bl(x == y),
                Ne | CaseNe => bl(x != y),
                Lt => bl(x < y),
                Le => bl(x <= y),
                Gt => bl(x > y),
                Ge => bl(x >= y),
                _ => return Err(self.err(e, n, "this operator cannot take a real")),
            });
        }
        // Strings compare as strings.
        if let (Const::Str(x), Const::Str(y)) = (&a, &b)
            && (matches!(op, Eq | Ne | CaseEq | CaseNe) || (ty != Types::UNKNOWN && !e.d.types.is_integral(ty)))
        {
            let bl = |v: bool| Const::Bits(Bits::from_bool(v));
            return Ok(match op {
                Eq | CaseEq => bl(x == y),
                Ne | CaseNe => bl(x != y),
                Lt => bl(x < y),
                Le => bl(x <= y),
                Gt => bl(x > y),
                _ => bl(x >= y),
            });
        }
        if matches!(a, Const::Unbounded) || matches!(b, Const::Unbounded) {
            return Err(Stop::NotConstant);
        }
        if matches!((&a, &b), (Const::Type(_), Const::Type(_))) {
            let same = a == b;
            return Ok(Const::Bits(Bits::from_bool(if matches!(op, Eq | CaseEq) { same } else { !same })));
        }
        let x = bits_of(&a).ok_or(Stop::NotConstant)?;
        let y = bits_of(&b).ok_or(Stop::NotConstant)?;
        let w = e.d.types.width(ty).max(1);
        let s = e.d.types.is_signed(ty);
        let fit = |v: &Bits| v.resize(w, v.signed()).with_signed(s);
        let r = match op {
            Add => fit(&x).add(&fit(&y)),
            Sub => fit(&x).sub(&fit(&y)),
            Mul => fit(&x).mul(&fit(&y)),
            Div => fit(&x).div(&fit(&y)),
            Mod => fit(&x).rem(&fit(&y)),
            And => fit(&x).and(&fit(&y)),
            Or => fit(&x).or(&fit(&y)),
            Xor => fit(&x).xor(&fit(&y)),
            Xnor => fit(&x).xnor(&fit(&y)),
            Pow => fit(&x).pow(&y),
            Shl | AShl | Shr | AShr => {
                let v = fit(&x);
                match y.to_u64() {
                    Some(k) => match op {
                        Shl | AShl => v.shl(k),
                        Shr => v.lshr(k),
                        _ => v.ashr(k),
                    },
                    None => Bits::x(w),
                }
            }
            Eq => Bits::of_logic(x.logic_eq(&y)),
            Ne => Bits::of_logic(x.logic_eq(&y)).not(),
            CaseEq => Bits::from_bool(x.case_eq(&y)),
            CaseNe => Bits::from_bool(!x.case_eq(&y)),
            WildEq => Bits::of_logic(x.wildcard_eq(&y)),
            WildNe => Bits::of_logic(x.wildcard_eq(&y)).not(),
            Lt => Bits::of_logic(x.cmp_op(&y, |o| o.is_lt())),
            Le => Bits::of_logic(x.cmp_op(&y, |o| o.is_le())),
            Gt => Bits::of_logic(x.cmp_op(&y, |o| o.is_gt())),
            Ge => Bits::of_logic(x.cmp_op(&y, |o| o.is_ge())),
            Implies => {
                let (p, q) = (x.truth(), y.truth());
                Bits::of_logic(match (p, q) {
                    (Some(false), _) | (_, Some(true)) => Logic::One,
                    (Some(true), Some(false)) => Logic::Zero,
                    _ => Logic::X,
                })
            }
            Equiv => match (x.truth(), y.truth()) {
                (Some(p), Some(q)) => Bits::from_bool(p == q),
                _ => Bits::x(1),
            },
            LogAnd | LogOr => unreachable!(),
        };
        Ok(Const::Bits(r))
    }

    fn index<'a>(&mut self, e: &Elab<'a, '_>, base: &Const, bt: TypeId, i: &Const, ty: TypeId, _n: NodeId) -> V<'a> {
        let Some(idx) = i.to_i64() else {
            return Ok(default_value(e, ty));
        };
        match base {
            Const::Aggregate(items) => {
                let (l, r) = match e.d.types.get(bt) {
                    Type::Unpacked { dim: UDim::Range(l, r), .. } => (*l, *r),
                    _ => (0, items.len() as i64 - 1),
                };
                let off = if l <= r { idx - l } else { l - idx };
                if off < 0 || off as usize >= items.len() {
                    return Ok(default_value(e, ty));
                }
                Ok(items[off as usize].clone())
            }
            Const::Str(s) => {
                if idx < 0 || idx as usize >= s.len() {
                    return Ok(Const::Bits(Bits::zero(8, false)));
                }
                Ok(Const::Bits(Bits::from_u64(8, false, s[idx as usize] as u64)))
            }
            _ => {
                let b = bits_of(base).ok_or(Stop::NotConstant)?;
                let ew = e.d.types.width(ty).max(1);
                match lsb_offset(e, bt, idx, idx) {
                    Some(off) => Ok(Const::Bits(b.slice(off as u32 * ew, ew))),
                    None => Ok(Const::Bits(Bits::x(ew))),
                }
            }
        }
    }

    pub(crate) fn call<'a>(&mut self, e: &mut Elab<'a, '_>, sub: SubId, args: &[ExprId], n: NodeId) -> V<'a> {
        super::checks::constant_function(e, sub, n).map_err(Stop::Error)?;
        if !e.d.subroutines[sub.0 as usize].elaborated {
            super::stmt::sub_body(e, sub).map_err(Stop::Error)?;
        }
        if self.frames.len() > 256 {
            return Err(self.err(e, n, "constant function calls are nested too deeply"));
        }
        let s = &e.d.subroutines[sub.0 as usize];
        let formals = s.args.clone();
        let body = s.body.clone();
        let scope = s.scope;
        let name = s.name;
        let ret = s.ret;
        let mut frame = HashMap::new();
        for (i, &f) in formals.iter().enumerate() {
            let v = match args.get(i) {
                Some(&a) => self.expr(e, a)?,
                None => default_value(e, e.obj(f).ty),
            };
            frame.insert(f, v);
        }
        // Locals start at their defaults.
        let locals: Vec<ObjId> = (0..e.d.objects.len() as u32)
            .map(ObjId)
            .filter(|&o| {
                let obj = e.obj(o);
                obj.kind == ObjKind::Variable && within(e, obj.scope, scope) && !formals.contains(&o)
            })
            .collect();
        for o in locals {
            let t = e.obj(o).ty;
            frame.insert(o, default_value(e, t));
        }
        if let Some(Binding::Object(r)) = e.scope(scope).names.get(&name).copied() {
            frame.insert(r, default_value(e, ret));
        }
        // Initialisers of locals.
        self.frames.push(frame);
        let inits: Vec<(ObjId, ExprId)> = (0..e.d.objects.len() as u32)
            .map(ObjId)
            .filter_map(|o| {
                let obj = e.obj(o);
                (within(e, obj.scope, scope) && obj.kind == ObjKind::Variable).then_some(()).and(obj.init.map(|i| (o, i)))
            })
            .collect();
        for (o, i) in inits {
            let v = self.expr(e, i)?;
            self.frames.last_mut().unwrap().insert(o, v);
        }
        let mut result = None;
        for st in body {
            match self.stmt(e, st) {
                Ok(()) => {}
                Err(Stop::Return(v)) => {
                    result = Some(v);
                    break;
                }
                Err(err) => {
                    self.frames.pop();
                    return Err(err);
                }
            }
        }
        let frame = self.frames.pop().unwrap();
        let v = match result {
            Some(v) => v,
            None => match e.scope(scope).names.get(&name).copied() {
                Some(Binding::Object(r)) => frame.get(&r).cloned().unwrap_or(Const::Null),
                _ => Const::Null,
            },
        };
        Ok(convert_const(e, &v, ret))
    }

    fn stmt<'a>(&mut self, e: &mut Elab<'a, '_>, s: StmtId) -> Result<(), Stop<'a>> {
        self.steps += 1;
        if self.steps > 50_000_000 {
            let n = e.d.stmts[s.0 as usize].node;
            return Err(self.err(e, n, "constant evaluation takes too long; is there an endless loop?"));
        }
        let st = e.d.stmts[s.0 as usize].clone();
        match st.kind {
            StmtKind::Null => Ok(()),
            StmtKind::Block { stmts, scope, .. } => {
                if let Some(bs) = scope {
                    // Block locals start at their defaults.
                    let locals: Vec<ObjId> = (0..e.d.objects.len() as u32)
                        .map(ObjId)
                        .filter(|&o| e.obj(o).scope == bs && e.obj(o).kind == ObjKind::Variable)
                        .collect();
                    for o in locals {
                        let v = match e.obj(o).init {
                            Some(i) => self.expr(e, i)?,
                            None => default_value(e, e.obj(o).ty),
                        };
                        self.frames.last_mut().ok_or(Stop::NotConstant)?.insert(o, v);
                    }
                }
                for x in stmts {
                    self.stmt(e, x)?;
                }
                Ok(())
            }
            StmtKind::Assign { kind, lhs, rhs, timing } => {
                if timing.is_some() {
                    return Err(self.err(e, st.node, "a constant function cannot wait (13.4.3)"));
                }
                let v = self.expr(e, rhs)?;
                let v = match kind {
                    AssignKind::Compound(op) => {
                        let old = self.expr(e, lhs)?;
                        let lt = e.ex(lhs).ty;
                        self.binary(e, op, old, v, lt, st.node)?
                    }
                    _ => v,
                };
                self.store(e, lhs, v)
            }
            StmtKind::Expr(x) => {
                self.expr(e, x)?;
                Ok(())
            }
            StmtKind::If { cond, then, else_, .. } => {
                let c = self.expr(e, cond)?;
                if truth(&c) == Some(true) {
                    self.stmt(e, then)
                } else if let Some(el) = else_ {
                    self.stmt(e, el)
                } else {
                    Ok(())
                }
            }
            StmtKind::Case { kind, expr: v, items, default, .. } => {
                let val = self.expr(e, v)?;
                for (xs, body) in items {
                    for x in xs {
                        let c = self.expr(e, x)?;
                        let hit = match (bits_of(&val), bits_of(&c)) {
                            (Some(a), Some(b)) => match kind {
                                CaseKind::Case => a.case_eq(&b),
                                CaseKind::CaseZ | CaseKind::CaseX => {
                                    let w = a.width().max(b.width());
                                    let (a, b) = (a.extend(w), b.extend(w));
                                    (0..w).all(|i| {
                                        let (p, q) = (a.bit(i), b.bit(i));
                                        let dont = |l: Logic| l == Logic::Z || (kind == CaseKind::CaseX && l == Logic::X);
                                        dont(p) || dont(q) || p == q
                                    })
                                }
                                CaseKind::Inside => a.wildcard_eq(&b) == Logic::One,
                            },
                            _ => const_eq(&val, &c),
                        };
                        if hit {
                            return self.stmt(e, body);
                        }
                    }
                }
                match default {
                    Some(d) => self.stmt(e, d),
                    None => Ok(()),
                }
            }
            StmtKind::For { init, cond, step, body, scope } => {
                if let Some(bs) = scope {
                    let locals: Vec<ObjId> =
                        (0..e.d.objects.len() as u32).map(ObjId).filter(|&o| e.obj(o).scope == bs).collect();
                    for o in locals {
                        let t = e.obj(o).ty;
                        self.frames.last_mut().ok_or(Stop::NotConstant)?.insert(o, default_value(e, t));
                    }
                }
                for s in init {
                    self.stmt(e, s)?;
                }
                loop {
                    if let Some(c) = cond {
                        let v = self.expr(e, c)?;
                        if truth(&v) != Some(true) {
                            break;
                        }
                    }
                    match self.stmt(e, body) {
                        Ok(()) | Err(Stop::Continue) => {}
                        Err(Stop::Break) => break,
                        Err(x) => return Err(x),
                    }
                    for s in &step {
                        self.stmt(e, *s)?;
                    }
                    self.steps += 1;
                    if self.steps > 50_000_000 {
                        return Err(self.err(e, st.node, "constant evaluation takes too long"));
                    }
                }
                Ok(())
            }
            StmtKind::While(c, body) => {
                loop {
                    let v = self.expr(e, c)?;
                    if truth(&v) != Some(true) {
                        break;
                    }
                    match self.stmt(e, body) {
                        Ok(()) | Err(Stop::Continue) => {}
                        Err(Stop::Break) => break,
                        Err(x) => return Err(x),
                    }
                }
                Ok(())
            }
            StmtKind::DoWhile(body, c) => {
                loop {
                    match self.stmt(e, body) {
                        Ok(()) | Err(Stop::Continue) => {}
                        Err(Stop::Break) => break,
                        Err(x) => return Err(x),
                    }
                    let v = self.expr(e, c)?;
                    if truth(&v) != Some(true) {
                        break;
                    }
                }
                Ok(())
            }
            StmtKind::Repeat(c, body) => {
                let k = self.expr(e, c)?.to_i64().unwrap_or(0);
                for _ in 0..k.max(0) {
                    match self.stmt(e, body) {
                        Ok(()) | Err(Stop::Continue) => {}
                        Err(Stop::Break) => break,
                        Err(x) => return Err(x),
                    }
                }
                Ok(())
            }
            StmtKind::Forever(body) => loop {
                match self.stmt(e, body) {
                    Ok(()) | Err(Stop::Continue) => {}
                    Err(Stop::Break) => return Ok(()),
                    Err(x) => return Err(x),
                }
            },
            StmtKind::Foreach { array, vars, body, .. } => {
                let a = self.expr(e, array)?;
                let n = match &a {
                    Const::Aggregate(v) => v.len() as i64,
                    Const::Str(s) => s.len() as i64,
                    Const::Bits(b) => b.width() as i64,
                    _ => 0,
                };
                let at = e.ex(array).ty;
                let (l, r) = match e.d.types.get(at) {
                    Type::Unpacked { dim: UDim::Range(l, r), .. } => (*l, *r),
                    _ => e.d.types.packed_range(at).unwrap_or((0, n - 1)),
                };
                let step = if l <= r { 1 } else { -1 };
                let mut i = l;
                for _ in 0..n {
                    if let Some(Some(v)) = vars.first() {
                        let t = e.obj(*v).ty;
                        let c = convert_const(e, &Const::Bits(Bits::from_i64(32, true, i)), t);
                        self.frames.last_mut().ok_or(Stop::NotConstant)?.insert(*v, c);
                    }
                    match self.stmt(e, body) {
                        Ok(()) | Err(Stop::Continue) => {}
                        Err(Stop::Break) => break,
                        Err(x) => return Err(x),
                    }
                    i += step;
                }
                Ok(())
            }
            StmtKind::Return(v) => {
                let r = match v {
                    Some(x) => self.expr(e, x)?,
                    None => Const::Null,
                };
                Err(Stop::Return(r))
            }
            StmtKind::Break => Err(Stop::Break),
            StmtKind::Continue => Err(Stop::Continue),
            StmtKind::Timed(..) | StmtKind::Wait(..) | StmtKind::WaitFork => {
                Err(self.err(e, st.node, "a constant function cannot wait (13.4.3)"))
            }
            StmtKind::Assert { cond, .. } => {
                self.expr(e, cond)?;
                Ok(())
            }
            _ => Err(self.err(e, st.node, "this statement cannot be run in a constant function (13.4.3)")),
        }
    }

    /// Writes `v` to the place `lhs` names, in the current frame.
    fn store<'a>(&mut self, e: &mut Elab<'a, '_>, lhs: ExprId, v: Const) -> Result<(), Stop<'a>> {
        let lx = e.ex(lhs).clone();
        let lt = lx.ty;
        match lx.kind {
            ExprKind::Object(o) => {
                let v = convert_const(e, &v, lt);
                let Some(f) = self.frames.iter_mut().rev().find(|f| f.contains_key(&o)) else {
                    let name = e.name(e.obj(o).name);
                    return Err(self.err(e, lx.node, format!("a constant function cannot write `{name}` (13.4.3)")));
                };
                f.insert(o, v);
                Ok(())
            }
            ExprKind::Index => {
                let base = self.expr(e, lx.args[0])?;
                let idx = self.expr(e, lx.args[1])?.to_i64();
                let bt = e.ex(lx.args[0]).ty;
                let new = match (base, idx) {
                    (Const::Aggregate(mut items), Some(i)) => {
                        let (l, r) = match e.d.types.get(bt) {
                            Type::Unpacked { dim: UDim::Range(l, r), .. } => (*l, *r),
                            _ => (0, items.len() as i64 - 1),
                        };
                        let off = if l <= r { i - l } else { l - i };
                        if matches!(e.d.types.get(bt), Type::Unpacked { dim: UDim::Dynamic | UDim::Queue(_) | UDim::Assoc(_), .. })
                            && off >= items.len() as i64
                        {
                            items.resize(off as usize + 1, default_value(e, lt));
                        }
                        if off >= 0 && (off as usize) < items.len() {
                            items[off as usize] = convert_const(e, &v, lt);
                        }
                        Const::Aggregate(items)
                    }
                    (Const::Bits(mut b), Some(i)) => {
                        let ew = e.d.types.width(lt).max(1);
                        if let Some(off) = lsb_offset(e, bt, i, i) {
                            let nv = bits_of(&convert_const(e, &v, lt)).unwrap_or(Bits::x(ew));
                            b.set_slice(off as u32 * ew, &nv.resize(ew, false));
                        }
                        Const::Bits(b)
                    }
                    (Const::Str(mut s), Some(i)) => {
                        if i >= 0 && (i as usize) < s.len() {
                            s[i as usize] = v.to_i64().unwrap_or(0) as u8;
                        }
                        Const::Str(s)
                    }
                    (b, _) => b,
                };
                self.store(e, lx.args[0], new)
            }
            ExprKind::Part { kind, width } => {
                let base = self.expr(e, lx.args[0])?;
                let a = self.expr(e, lx.args[1])?.to_i64();
                let bt = e.ex(lx.args[0]).ty;
                let (Const::Bits(mut b), Some(a)) = (base, a) else { return Ok(()) };
                let (hi, lo) = match kind {
                    PartKind::Range => (a, self.expr(e, lx.args[2])?.to_i64().unwrap_or(0)),
                    PartKind::Up => (a + width as i64 - 1, a),
                    PartKind::Down => (a, a - width as i64 + 1),
                };
                if let Some(off) = lsb_offset(e, bt, lo.min(hi), hi.max(lo)) {
                    let nv = bits_of(&v).unwrap_or(Bits::x(width));
                    b.set_slice(off as u32, &nv.resize(width, false));
                }
                self.store(e, lx.args[0], Const::Bits(b))
            }
            ExprKind::Member(i) => {
                let base = self.expr(e, lx.args[0])?;
                let bt = e.ex(lx.args[0]).ty;
                let new = match base {
                    Const::Aggregate(mut items) => {
                        if (i as usize) < items.len() {
                            items[i as usize] = v;
                        }
                        Const::Aggregate(items)
                    }
                    Const::Bits(mut b) => {
                        if let Type::Struct { members, .. } = e.d.types.get(bt).clone() {
                            let w = e.d.types.width(members[i as usize].ty);
                            let below: u32 = members[i as usize + 1..].iter().map(|m| e.d.types.width(m.ty)).sum();
                            let nv = bits_of(&v).unwrap_or(Bits::x(w));
                            b.set_slice(below, &nv.resize(w, false));
                        }
                        Const::Bits(b)
                    }
                    other => other,
                };
                self.store(e, lx.args[0], new)
            }
            ExprKind::Concat => {
                // `{a, b} = v`: split from the most significant end.
                let b = bits_of(&v).ok_or(Stop::NotConstant)?;
                let mut hi = b.width();
                for &p in &lx.args {
                    let w = e.d.types.width(e.ex(p).ty);
                    hi = hi.saturating_sub(w);
                    self.store(e, p, Const::Bits(b.slice(hi, w)))?;
                }
                Ok(())
            }
            ExprKind::Convert => self.store(e, lx.args[0], v),
            _ => Err(self.err(e, lx.node, "a constant function cannot write here (13.4.3)")),
        }
    }

    fn syscall<'a>(&mut self, e: &mut Elab<'a, '_>, name: &str, args: &[ExprId], ty: TypeId, n: NodeId) -> V<'a> {
        let int = |v: i64| Const::Bits(Bits::from_i64(32, true, v));
        let arg_ty = |e: &Elab, i: usize| -> TypeId {
            match args.get(i).map(|&a| &e.ex(a).kind) {
                Some(ExprKind::TypeRef(t)) => *t,
                _ => args.get(i).map_or(Types::UNKNOWN, |&a| e.ex(a).ty),
            }
        };
        match name {
            "$clog2" => {
                let v = self.expr(e, args[0])?;
                let b = bits_of(&v).ok_or(Stop::NotConstant)?;
                if !b.is_known() {
                    return Ok(Const::Bits(Bits::x(32)));
                }
                // The number of bits needed to address v values.
                let mut k = 0u32;
                let mut p = Bits::from_u64(b.width().max(1) + 1, false, 1);
                let target = b.resize(b.width() + 1, false).with_signed(false);
                while p.cmp_op(&target, |o| o.is_lt()) == Logic::One {
                    p = p.shl(1);
                    k += 1;
                    if k > b.width() + 1 {
                        break;
                    }
                }
                Ok(int(k as i64))
            }
            "$bits" => {
                let t = arg_ty(e, 0);
                let w = e.d.types.width(t);
                if w == 0 {
                    // An unpacked array of fixed size, or a value.
                    let v = bits_total(e, t);
                    return match v {
                        Some(v) => Ok(int(v as i64)),
                        None => {
                            if let Some(&a) = args.first() {
                                let c = self.expr(e, a)?;
                                match c {
                                    Const::Str(s) => Ok(int(s.len() as i64 * 8)),
                                    Const::Aggregate(v) => Ok(int(v.len() as i64)),
                                    _ => Err(Stop::NotConstant),
                                }
                            } else {
                                Err(Stop::NotConstant)
                            }
                        }
                    };
                }
                Ok(int(w as i64))
            }
            "$size" | "$left" | "$right" | "$low" | "$high" | "$increment" => {
                let t = arg_ty(e, 0);
                let dim = match args.get(1) {
                    Some(&a) => self.expr(e, a)?.to_i64().unwrap_or(1),
                    None => 1,
                };
                let Some((l, r)) = nth_range(e, t, dim) else { return Err(Stop::NotConstant) };
                Ok(int(match name {
                    "$size" => (l - r).abs() + 1,
                    "$left" => l,
                    "$right" => r,
                    "$low" => l.min(r),
                    "$high" => l.max(r),
                    _ => {
                        if l >= r {
                            1
                        } else {
                            -1
                        }
                    }
                }))
            }
            "$dimensions" | "$unpacked_dimensions" => {
                let t = arg_ty(e, 0);
                let (p, u) = count_dims(e, t);
                Ok(int(if name == "$dimensions" { (p + u) as i64 } else { u as i64 }))
            }
            "$signed" | "$unsigned" => {
                let v = self.expr(e, args[0])?;
                let b = bits_of(&v).ok_or(Stop::NotConstant)?;
                Ok(Const::Bits(b.with_signed(name == "$signed")))
            }
            "$countones" => {
                let b = bits_of(&self.expr(e, args[0])?).ok_or(Stop::NotConstant)?;
                Ok(int((0..b.width()).filter(|&i| b.bit(i) == Logic::One).count() as i64))
            }
            "$onehot" | "$onehot0" => {
                let b = bits_of(&self.expr(e, args[0])?).ok_or(Stop::NotConstant)?;
                let c = (0..b.width()).filter(|&i| b.bit(i) == Logic::One).count();
                Ok(Const::Bits(Bits::from_bool(if name == "$onehot" { c == 1 } else { c <= 1 })))
            }
            "$isunknown" => {
                let b = bits_of(&self.expr(e, args[0])?).ok_or(Stop::NotConstant)?;
                Ok(Const::Bits(Bits::from_bool(!b.is_known())))
            }
            "$itor" => {
                let b = bits_of(&self.expr(e, args[0])?).ok_or(Stop::NotConstant)?;
                Ok(Const::Real(b.to_f64()))
            }
            "$rtoi" => match self.expr(e, args[0])? {
                Const::Real(r) => Ok(int(r.trunc() as i64)),
                v => Ok(v),
            },
            "$realtobits" => match self.expr(e, args[0])? {
                Const::Real(r) => Ok(Const::Bits(Bits::from_u64(64, false, r.to_bits()))),
                _ => Err(Stop::NotConstant),
            },
            "$bitstoreal" => {
                let b = bits_of(&self.expr(e, args[0])?).ok_or(Stop::NotConstant)?;
                Ok(Const::Real(f64::from_bits(b.to_u64().unwrap_or(0))))
            }
            "$sqrt" | "$ln" | "$log10" | "$exp" | "$floor" | "$ceil" | "$sin" | "$cos" | "$tan" | "$asin" | "$acos"
            | "$atan" | "$sinh" | "$cosh" | "$tanh" | "$asinh" | "$acosh" | "$atanh" => {
                let x = match self.expr(e, args[0])? {
                    Const::Real(r) => r,
                    Const::Bits(b) => b.to_f64(),
                    _ => return Err(Stop::NotConstant),
                };
                Ok(Const::Real(match name {
                    "$sqrt" => x.sqrt(),
                    "$ln" => x.ln(),
                    "$log10" => x.log10(),
                    "$exp" => x.exp(),
                    "$floor" => x.floor(),
                    "$ceil" => x.ceil(),
                    "$sin" => x.sin(),
                    "$cos" => x.cos(),
                    "$tan" => x.tan(),
                    "$asin" => x.asin(),
                    "$acos" => x.acos(),
                    "$atan" => x.atan(),
                    "$sinh" => x.sinh(),
                    "$cosh" => x.cosh(),
                    "$tanh" => x.tanh(),
                    "$asinh" => x.asinh(),
                    "$acosh" => x.acosh(),
                    _ => x.atanh(),
                }))
            }
            "$pow" | "$atan2" | "$hypot" => {
                let f = |c: Const| match c {
                    Const::Real(r) => Some(r),
                    Const::Bits(b) => Some(b.to_f64()),
                    _ => None,
                };
                let x = f(self.expr(e, args[0])?).ok_or(Stop::NotConstant)?;
                let y = f(self.expr(e, args[1])?).ok_or(Stop::NotConstant)?;
                Ok(Const::Real(match name {
                    "$pow" => x.powf(y),
                    "$atan2" => x.atan2(y),
                    _ => x.hypot(y),
                }))
            }
            "$typename" => {
                let t = arg_ty(e, 0);
                Ok(Const::Str(e.d.types.describe(t, &e.d.syms).into_bytes()))
            }
            "$sformatf" | "$psprintf" => {
                let mut vals = Vec::new();
                for &a in args {
                    vals.push(self.expr(e, a)?);
                }
                Ok(Const::Str(crate::design::print::format(&vals).into_bytes()))
            }
            "$error" | "$fatal" | "$warning" | "$info" | "$display" | "$write" => Ok(Const::Null),
            "$isunbounded" => {
                let v = self.expr(e, args[0])?;
                Ok(Const::Bits(Bits::from_bool(v == Const::Unbounded)))
            }
            _ => {
                let _ = ty;
                let _ = n;
                Err(Stop::NotConstant)
            }
        }
    }

    fn method<'a>(&mut self, e: &mut Elab<'a, '_>, name: &str, args: &[ExprId], _n: NodeId) -> V<'a> {
        let base = self.expr(e, args[0])?;
        let bt = e.ex(args[0]).ty;
        match (name, &base, e.d.types.get(bt).clone()) {
            ("len", Const::Str(s), _) => Ok(Const::Bits(Bits::from_i64(32, true, s.len() as i64))),
            ("size", Const::Aggregate(v), _) => Ok(Const::Bits(Bits::from_i64(32, true, v.len() as i64))),
            ("toupper", Const::Str(s), _) => Ok(Const::Str(s.to_ascii_uppercase())),
            ("tolower", Const::Str(s), _) => Ok(Const::Str(s.to_ascii_lowercase())),
            ("num", _, Type::Enum { items, .. }) => Ok(Const::Bits(Bits::from_i64(32, true, items.len() as i64))),
            ("first", _, Type::Enum { items, .. }) => Ok(items.first().map_or(Const::Null, |(_, v)| Const::Bits(v.clone()))),
            ("last", _, Type::Enum { items, .. }) => Ok(items.last().map_or(Const::Null, |(_, v)| Const::Bits(v.clone()))),
            ("name", Const::Bits(b), Type::Enum { items, .. }) => {
                Ok(Const::Str(items.iter().find(|(_, v)| v.case_eq(b)).map_or(vec![], |(s, _)| e.name(*s).as_bytes().to_vec())))
            }
            ("next" | "prev", Const::Bits(b), Type::Enum { items, .. }) => {
                let k = match args.get(1) {
                    Some(&a) => self.expr(e, a)?.to_i64().unwrap_or(1),
                    None => 1,
                };
                let i = items.iter().position(|(_, v)| v.case_eq(b));
                let n = items.len() as i64;
                Ok(match i {
                    Some(i) => {
                        let j = if name == "next" { (i as i64 + k).rem_euclid(n) } else { (i as i64 - k).rem_euclid(n) };
                        Const::Bits(items[j as usize].1.clone())
                    }
                    None => default_value(e, bt),
                })
            }
            _ => Err(Stop::NotConstant),
        }
    }
}

fn truth(c: &Const) -> Option<bool> {
    match c {
        Const::Bits(b) => b.truth(),
        Const::Real(r) => Some(*r != 0.0),
        Const::Str(s) => Some(!s.is_empty()),
        Const::Null => Some(false),
        _ => None,
    }
}

/// Whether scope `s` is `top` or inside it.
fn within(e: &Elab, s: ScopeId, top: ScopeId) -> bool {
    let mut cur = Some(s);
    while let Some(id) = cur {
        if id == top {
            return true;
        }
        cur = e.scope(id).parent;
    }
    false
}

/// The offset, in elements from the least significant end, of the indices
/// `lo..=hi` of an integral type's outermost dimension; `None` if out of
/// range (11.5.1).
pub(crate) fn lsb_offset(e: &Elab, t: TypeId, lo: i64, hi: i64) -> Option<i64> {
    let (l, r) = e.d.types.packed_range(t).unwrap_or((e.d.types.width(t) as i64 - 1, 0));
    let (min, max) = (l.min(r), l.max(r));
    if lo < min || hi > max {
        return None;
    }
    Some(if l >= r { lo - r } else { r - hi })
}

/// The range of dimension `dim` (1 is the outermost), unpacked first, for
/// `$left` and the like (20.7).
fn nth_range(e: &Elab, t: TypeId, dim: i64) -> Option<(i64, i64)> {
    let mut ranges = Vec::new();
    let mut cur = t;
    loop {
        match e.d.types.get(cur) {
            Type::Unpacked { elem, dim: UDim::Range(l, r) } => {
                ranges.push((*l, *r));
                cur = *elem;
            }
            Type::Vector { atom, dims, .. } => {
                if dims.is_empty() {
                    if !atom.is_scalar() {
                        ranges.push((atom.width() as i64 - 1, 0));
                    }
                } else {
                    ranges.extend(dims.iter().copied());
                }
                break;
            }
            Type::PackedArray { elem, dim, .. } => {
                ranges.push(*dim);
                cur = *elem;
            }
            Type::Struct { .. } | Type::Enum { .. } => {
                let w = e.d.types.width(cur) as i64;
                if w > 0 {
                    ranges.push((w - 1, 0));
                }
                break;
            }
            _ => break,
        }
    }
    ranges.get(dim as usize - 1).copied()
}

fn count_dims(e: &Elab, t: TypeId) -> (usize, usize) {
    let (mut p, mut u) = (0, 0);
    let mut cur = t;
    loop {
        match e.d.types.get(cur) {
            Type::Unpacked { elem, .. } => {
                u += 1;
                cur = *elem;
            }
            Type::Vector { atom, dims, .. } => {
                p += if dims.is_empty() { usize::from(!atom.is_scalar()) } else { dims.len() };
                break;
            }
            Type::PackedArray { elem, .. } => {
                p += 1;
                cur = *elem;
            }
            Type::Struct { packed: true, .. } | Type::Enum { .. } => {
                p += 1;
                break;
            }
            Type::String => {
                u += 1;
                break;
            }
            _ => break,
        }
    }
    (p, u)
}

/// The number of bits in a fixed-size type, unpacked dimensions included.
fn bits_total(e: &Elab, t: TypeId) -> Option<u64> {
    match e.d.types.get(t) {
        Type::Unpacked { elem, dim: UDim::Range(l, r) } => Some(((l - r).unsigned_abs() + 1) * bits_total(e, *elem)?),
        Type::Struct { members, union, .. } => {
            let ws: Option<Vec<u64>> = members.iter().map(|m| bits_total(e, m.ty)).collect();
            let ws = ws?;
            Some(if *union { ws.into_iter().max().unwrap_or(0) } else { ws.into_iter().sum() })
        }
        _ => {
            let w = e.d.types.width(t);
            if w > 0 { Some(w as u64) } else { None }
        }
    }
}

#[allow(dead_code)]
fn is_kind(e: &Elab, n: NodeId, k: Kind) -> bool {
    e.kind(n) == k
}
