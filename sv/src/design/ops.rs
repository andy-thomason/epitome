//! Operations on values, shared by constant evaluation and simulation:
//! clause 11's operators and conversions on `Const`, and the facts about
//! types they need. A `None` result means the operation does not apply
//! to these values (a real given to `%`, say).

use super::types::{Type, TypeId, Types, UDim};
use super::value::{Bits, Logic};
use super::{BinOp, Const, UnOp};

/// A constant converted to type `t`, as assignment converts (10.7).
pub fn convert(tys: &Types, v: &Const, t: TypeId) -> Const {
    let ty = tys;
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
pub fn two_state(b: &Bits) -> Bits {
    let mut r = b.clone();
    for i in 0..b.width() {
        if !b.bit(i).is_known() {
            r.set_bit(i, Logic::Zero);
        }
    }
    r
}

/// Equality for a generate `case` item: bit for bit (27.5).
pub fn const_eq(a: &Const, b: &Const) -> bool {
    match (a, b) {
        (Const::Bits(x), Const::Bits(y)) => x.case_eq(y),
        (Const::Real(x), Const::Real(y)) => x == y,
        (Const::Bits(x), Const::Real(y)) | (Const::Real(y), Const::Bits(x)) => x.to_f64() == *y,
        (Const::Str(x), Const::Str(y)) => x == y,
        (Const::Type(x), Const::Type(y)) => x == y,
        _ => a == b,
    }
}

pub fn bits_of(c: &Const) -> Option<Bits> {
    match c {
        Const::Bits(b) => Some(b.clone()),
        Const::Real(r) => Some(Bits::from_f64(64, true, *r)),
        Const::Str(s) => Some(Bits::from_str_bytes(s)),
        _ => None,
    }
}

/// The default value of a variable of type `t` (6.8, table 6-7).
pub fn default_value(tys: &Types, t: TypeId) -> Const {
    let ty = tys;
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
            let d = default_value(tys, *elem);
            Const::Aggregate(vec![d; n.min(1 << 20)])
        }
        Type::Unpacked { .. } => Const::Aggregate(vec![]),
        Type::Struct { members, .. } => Const::Aggregate(members.iter().map(|m| default_value(tys, m.ty)).collect()),
        Type::Class(_) | Type::Null | Type::Chandle => Const::Null,
        _ => Const::Bits(Bits::x(1)),
    }
}

pub fn truth(c: &Const) -> Option<bool> {
    match c {
        Const::Bits(b) => b.truth(),
        Const::Real(r) => Some(*r != 0.0),
        Const::Str(s) => Some(!s.is_empty()),
        Const::Null => Some(false),
        _ => None,
    }
}

/// The offset, in elements from the least significant end, of the indices
/// `lo..=hi` of an integral type's outermost dimension; `None` if out of
/// range (11.5.1).
pub fn lsb_offset(tys: &Types, t: TypeId, lo: i64, hi: i64) -> Option<i64> {
    let (l, r) = tys.packed_range(t).unwrap_or((tys.width(t) as i64 - 1, 0));
    let (min, max) = (l.min(r), l.max(r));
    if lo < min || hi > max {
        return None;
    }
    Some(if l >= r { lo - r } else { r - hi })
}

/// The range of dimension `dim` (1 is the outermost), unpacked first, for
/// `$left` and the like (20.7).
pub fn nth_range(tys: &Types, t: TypeId, dim: i64) -> Option<(i64, i64)> {
    let mut ranges = Vec::new();
    let mut cur = t;
    loop {
        match tys.get(cur) {
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
                let w = tys.width(cur) as i64;
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

pub fn count_dims(tys: &Types, t: TypeId) -> (usize, usize) {
    let (mut p, mut u) = (0, 0);
    let mut cur = t;
    loop {
        match tys.get(cur) {
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
pub fn bits_total(tys: &Types, t: TypeId) -> Option<u64> {
    match tys.get(t) {
        Type::Unpacked { elem, dim: UDim::Range(l, r) } => Some(((l - r).unsigned_abs() + 1) * bits_total(tys, *elem)?),
        Type::Struct { members, union, .. } => {
            let ws: Option<Vec<u64>> = members.iter().map(|m| bits_total(tys, m.ty)).collect();
            let ws = ws?;
            Some(if *union { ws.into_iter().max().unwrap_or(0) } else { ws.into_iter().sum() })
        }
        _ => {
            let w = tys.width(t);
            if w > 0 { Some(w as u64) } else { None }
        }
    }
}

pub fn unary(tys: &Types, op: UnOp, v: Const, ty: TypeId) -> Option<Const> {
    if let Const::Real(r) = v {
        return Some(match op {
            UnOp::Plus => Const::Real(r),
            UnOp::Neg => Const::Real(-r),
            UnOp::LogNot => Const::Bits(Bits::from_bool(r == 0.0)),
            _ => return None,
        });
    }
    let b = bits_of(&v)?;
    let w = tys.width(ty).max(b.width());
    let s = tys.is_signed(ty);
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
        _ => return None,
    };
    Some(Const::Bits(r))
}


pub fn binary(tys: &Types, op: BinOp, a: Const, b: Const, ty: TypeId) -> Option<Const> {
    use BinOp::*;
    // Reals.
    if matches!(a, Const::Real(_)) || matches!(b, Const::Real(_)) {
        let x = match &a {
            Const::Real(r) => *r,
            Const::Bits(b) => b.to_f64(),
            _ => return None,
        };
        let y = match &b {
            Const::Real(r) => *r,
            Const::Bits(b) => b.to_f64(),
            _ => return None,
        };
        let bl = |v: bool| Const::Bits(Bits::from_bool(v));
        return Some(match op {
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
            _ => return None,
        });
    }
    // Strings compare as strings.
    if let (Const::Str(x), Const::Str(y)) = (&a, &b)
        && (matches!(op, Eq | Ne | CaseEq | CaseNe) || (ty != Types::UNKNOWN && !tys.is_integral(ty)))
    {
        let bl = |v: bool| Const::Bits(Bits::from_bool(v));
        return Some(match op {
            Eq | CaseEq => bl(x == y),
            Ne | CaseNe => bl(x != y),
            Lt => bl(x < y),
            Le => bl(x <= y),
            Gt => bl(x > y),
            _ => bl(x >= y),
        });
    }
    if matches!(a, Const::Unbounded) || matches!(b, Const::Unbounded) {
        return None;
    }
    if matches!((&a, &b), (Const::Type(_), Const::Type(_))) {
        let same = a == b;
        return Some(Const::Bits(Bits::from_bool(if matches!(op, Eq | CaseEq) { same } else { !same })));
    }
    let x = bits_of(&a)?;
    let y = bits_of(&b)?;
    let w = tys.width(ty).max(1);
    let s = tys.is_signed(ty);
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
    Some(Const::Bits(r))
}


pub fn index(tys: &Types, base: &Const, bt: TypeId, i: &Const, ty: TypeId) -> Option<Const> {
    let Some(idx) = i.to_i64() else {
        return Some(default_value(tys, ty));
    };
    match base {
        Const::Aggregate(items) => {
            let (l, r) = match tys.get(bt) {
                Type::Unpacked { dim: UDim::Range(l, r), .. } => (*l, *r),
                _ => (0, items.len() as i64 - 1),
            };
            let off = if l <= r { idx - l } else { l - idx };
            if off < 0 || off as usize >= items.len() {
                return Some(default_value(tys, ty));
            }
            Some(items[off as usize].clone())
        }
        Const::Str(s) => {
            if idx < 0 || idx as usize >= s.len() {
                return Some(Const::Bits(Bits::zero(8, false)));
            }
            Some(Const::Bits(Bits::from_u64(8, false, s[idx as usize] as u64)))
        }
        _ => {
            let b = bits_of(base)?;
            let ew = tys.width(ty).max(1);
            match lsb_offset(tys, bt, idx, idx) {
                Some(off) => Some(Const::Bits(b.slice(off as u32 * ew, ew))),
                None => Some(Const::Bits(Bits::x(ew))),
            }
        }
    }
}

