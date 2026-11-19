//! Arithmetic for combines: reading constants, evaluating the constant
//! expressions and conditions of rules, and folding operations whose
//! operands are all constant.
//!
//! Integers up to 64 bits are handled; of wider ones, only comparisons
//! fold (`fold_icmp_wide`). The semantics follow the Language Reference: shifting by the width or more
//! gives poison, dividing by zero (and signed division overflow) is
//! undefined behaviour and is never folded, and `nuw`, `nsw`, `exact` and
//! `disjoint` give poison when their promise is broken.

use epitome_ir::apint::ApInt;
use epitome_ir::float::{self, FloatKind};
use epitome_ir::module::*;
use epitome_ir::types::{Type, TypeId};

use crate::table::{ConstTable as Constants, TypeTable as Types};

use super::dsl::{BinExprOp, CmpOp, Cond, Expr, FloatLit, Lit, Subject, UnOp};

/// An integer constant (scalar, or a vector with every element the same),
/// with its value truncated to its width.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct IntConst {
    pub bits: u32,
    pub value: u64,
}

pub fn mask(bits: u32) -> u64 {
    if bits >= 64 { u64::MAX } else { (1u64 << bits) - 1 }
}

/// Sign-extends a `bits`-wide value to 64 bits.
pub fn sext(value: u64, bits: u32) -> i64 {
    if bits >= 64 { value as i64 } else { ((value << (64 - bits)) as i64) >> (64 - bits) }
}

/// The integer width of a type, or of its elements for a vector.
pub fn int_bits(types: &Types, ty: TypeId) -> Option<u32> {
    match types.get(types.scalar(ty)) {
        Type::Int(b) => Some(*b),
        _ => None,
    }
}

pub fn int_const(consts: &Constants, types: &Types, c: ConstId) -> Option<IntConst> {
    let k = consts.get(c);
    match &k.kind {
        ConstKind::Int(a) if a.bits() <= 64 => Some(IntConst { bits: a.bits(), value: a.low_u64() }),
        ConstKind::Zero => {
            let bits = int_bits(types, k.ty).filter(|&b| b <= 64)?;
            Some(IntConst { bits, value: 0 })
        }
        ConstKind::Splat(e) => int_const(consts, types, *e),
        ConstKind::Vector(es) => {
            let first = int_const(consts, types, *es.first()?)?;
            es[1..].iter().all(|&e| int_const(consts, types, e) == Some(first)).then_some(first)
        }
        _ => None,
    }
}

/// Which of the float literals a constant is, if any.
pub fn float_lit(consts: &Constants, types: &Types, c: ConstId) -> Option<FloatLit> {
    let k = consts.get(c);
    match &k.kind {
        ConstKind::Zero => FloatKind::of(types.get(types.scalar(k.ty))).map(|_| FloatLit::PosZero),
        ConstKind::Float(bits) => {
            let kind = FloatKind::of(types.get(k.ty))?;
            let v = float::to_f64(kind, *bits)?;
            if v == 0.0 {
                Some(if v.is_sign_negative() { FloatLit::NegZero } else { FloatLit::PosZero })
            } else if v == 1.0 {
                Some(FloatLit::One)
            } else {
                None
            }
        }
        ConstKind::Splat(e) => float_lit(consts, types, *e),
        ConstKind::Vector(es) => {
            let first = float_lit(consts, types, *es.first()?)?;
            es[1..].iter().all(|&e| float_lit(consts, types, e) == Some(first)).then_some(first)
        }
        _ => None,
    }
}

/// Every literal a constant equals: for an integer, its value read as
/// unsigned and as signed.
pub fn lits_of(consts: &Constants, types: &Types, c: ConstId) -> Vec<Lit> {
    if matches!(consts.get(c).kind, ConstKind::Poison) {
        return vec![Lit::Poison];
    }
    if let Some(k) = int_const(consts, types, c) {
        let unsigned = Lit::Int(k.value as i64);
        let signed = Lit::Int(sext(k.value, k.bits));
        return if unsigned == signed { vec![unsigned] } else { vec![unsigned, signed] };
    }
    float_lit(consts, types, c).map(Lit::Float).into_iter().collect()
}

/// What a rule's expressions can see.
pub struct Env<'a> {
    /// The value of each constant variable.
    pub consts: &'a [Option<IntConst>],
    /// The integer width of each variable.
    pub widths: &'a [Option<u32>],
    /// `bw`.
    pub bw: Option<u32>,
}

/// Evaluates an expression exactly (modulo 2^128). `None` if it uses
/// something unknown, such as the width of a non-integer.
pub fn eval(e: &Expr, env: &Env) -> Option<u128> {
    Some(match e {
        Expr::Var(v) => env.consts[*v]?.value as u128,
        Expr::Num(n) => *n as i128 as u128,
        Expr::Bw => env.bw? as u128,
        Expr::BwOf(v) => env.widths[*v]? as u128,
        Expr::Log2(x) => {
            let x = eval(x, env)?;
            if x == 0 {
                return None;
            }
            127 - x.leading_zeros() as u128
        }
        Expr::Un(UnOp::Neg, x) => eval(x, env)?.wrapping_neg(),
        Expr::Un(UnOp::Not, x) => !eval(x, env)?,
        Expr::Bin(op, a, b) => {
            let (a, b) = (eval(a, env)?, eval(b, env)?);
            match op {
                BinExprOp::Add => a.wrapping_add(b),
                BinExprOp::Sub => a.wrapping_sub(b),
                BinExprOp::Mul => a.wrapping_mul(b),
                BinExprOp::And => a & b,
                BinExprOp::Or => a | b,
                BinExprOp::Xor => a ^ b,
                BinExprOp::Shl => a.checked_shl(u32::try_from(b).ok()?)?,
                BinExprOp::Shr => a.checked_shr(u32::try_from(b).ok()?)?,
            }
        }
    })
}

/// Checks a condition. `same_type` answers `sametype(a, b)`.
pub fn check(c: &Cond, env: &Env, same_type: &dyn Fn(Subject, Subject) -> bool) -> bool {
    match c {
        Cond::Pow2(e) => eval(e, env).is_some_and(|v| {
            // A power of two at the width it is used: the constant's own.
            let v = v as u64 & env.bw.map_or(u64::MAX, mask);
            v.is_power_of_two()
        }),
        Cond::SameType(a, b) => same_type(*a, *b),
        // Checked by the combiner, which knows the uses and has alias
        // analysis.
        Cond::OneUse | Cond::MustAlias(..) => true,
        Cond::Cmp(op, a, b) => {
            let (Some(a), Some(b)) = (eval(a, env), eval(b, env)) else { return false };
            match op {
                CmpOp::Eq => a == b,
                CmpOp::Ne => a != b,
                CmpOp::Lt => a < b,
                CmpOp::Le => a <= b,
                CmpOp::Gt => a > b,
                CmpOp::Ge => a >= b,
            }
        }
    }
}

/// The result of folding.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Folded {
    Int(u64),
    Poison,
}

/// Folds an integer binary operation on `bits`-wide values. `None` when
/// the operation is undefined behaviour or not an integer operation.
pub fn fold_binop(op: BinOp, flags: IntFlags, a: u64, b: u64, bits: u32) -> Option<Folded> {
    let m = mask(bits);
    let (a, b) = (a & m, b & m);
    let (sa, sb) = (sext(a, bits), sext(b, bits));
    let smin = sext(1u64 << (bits - 1), bits);
    let in_signed = |v: i128| v >= smin as i128 && v < -(smin as i128);
    let poison_if = |bad: bool, v: u64| Some(if bad { Folded::Poison } else { Folded::Int(v & m) });
    let nuw = flags.contains(IntFlags::NUW);
    let nsw = flags.contains(IntFlags::NSW);
    let exact = flags.contains(IntFlags::EXACT);
    match op {
        BinOp::Add => {
            let r = a.wrapping_add(b) & m;
            poison_if((nuw && (a as u128 + b as u128) > m as u128) || (nsw && !in_signed(sa as i128 + sb as i128)), r)
        }
        BinOp::Sub => {
            let r = a.wrapping_sub(b) & m;
            poison_if((nuw && a < b) || (nsw && !in_signed(sa as i128 - sb as i128)), r)
        }
        BinOp::Mul => {
            let r = a.wrapping_mul(b) & m;
            poison_if((nuw && (a as u128 * b as u128) > m as u128) || (nsw && !in_signed(sa as i128 * sb as i128)), r)
        }
        BinOp::UDiv => {
            if b == 0 {
                return None;
            }
            poison_if(exact && a % b != 0, a / b)
        }
        BinOp::URem => (b != 0).then(|| Folded::Int(a % b)),
        BinOp::SDiv => {
            if sb == 0 || (sa == smin && sb == -1) {
                return None;
            }
            poison_if(exact && sa % sb != 0, (sa / sb) as u64)
        }
        BinOp::SRem => {
            if sb == 0 || (sa == smin && sb == -1) {
                return None;
            }
            Some(Folded::Int((sa % sb) as u64 & m))
        }
        BinOp::Shl | BinOp::LShr | BinOp::AShr if b >= bits as u64 => Some(Folded::Poison),
        BinOp::Shl => {
            let r = (a << b) & m;
            let lost_unsigned = (r >> b) != a;
            let lost_signed = sext(r, bits) >> b != sa;
            poison_if((nuw && lost_unsigned) || (nsw && lost_signed), r)
        }
        BinOp::LShr => poison_if(exact && (a & ((1u64 << b) - 1)) != 0, a >> b),
        BinOp::AShr => poison_if(exact && (a & ((1u64 << b) - 1)) != 0, (sa >> b) as u64),
        BinOp::And => Some(Folded::Int(a & b)),
        BinOp::Or => poison_if(flags.contains(IntFlags::DISJOINT) && a & b != 0, a | b),
        BinOp::Xor => Some(Folded::Int(a ^ b)),
        _ => None,
    }
}

pub fn fold_icmp(pred: ICmpPred, a: u64, b: u64, bits: u32) -> bool {
    let m = mask(bits);
    let (a, b) = (a & m, b & m);
    let (sa, sb) = (sext(a, bits), sext(b, bits));
    match pred {
        ICmpPred::Eq => a == b,
        ICmpPred::Ne => a != b,
        ICmpPred::Ugt => a > b,
        ICmpPred::Uge => a >= b,
        ICmpPred::Ult => a < b,
        ICmpPred::Ule => a <= b,
        ICmpPred::Sgt => sa > sb,
        ICmpPred::Sge => sa >= sb,
        ICmpPred::Slt => sa < sb,
        ICmpPred::Sle => sa <= sb,
    }
}

/// An integer constant of any width (scalar, or a splat).
pub fn wide_int(consts: &Constants, types: &Types, c: ConstId) -> Option<ApInt> {
    let k = consts.get(c);
    match &k.kind {
        ConstKind::Int(a) => Some(a.clone()),
        ConstKind::Zero => Some(ApInt::zero(int_bits(types, k.ty)?)),
        ConstKind::Splat(e) => wide_int(consts, types, *e),
        _ => None,
    }
}

/// `fold_icmp` for integers of any width, such as the `i128` compares of
/// `u16::MAX as u128 <= usize::MAX as u128`. `None` if the widths differ.
pub fn fold_icmp_wide(pred: ICmpPred, a: &ApInt, b: &ApInt) -> Option<bool> {
    use std::cmp::Ordering::*;
    if a.bits() != b.bits() {
        return None;
    }
    // Words are little-endian with the bits above the width zero, so the
    // unsigned order is the order of the words from the top.
    let unsigned = a.words().iter().rev().cmp(b.words().iter().rev());
    let signed = match (a.is_negative(), b.is_negative()) {
        (true, false) => Less,
        (false, true) => Greater,
        _ => unsigned,
    };
    Some(match pred {
        ICmpPred::Eq => unsigned == Equal,
        ICmpPred::Ne => unsigned != Equal,
        ICmpPred::Ugt => unsigned == Greater,
        ICmpPred::Uge => unsigned != Less,
        ICmpPred::Ult => unsigned == Less,
        ICmpPred::Ule => unsigned != Greater,
        ICmpPred::Sgt => signed == Greater,
        ICmpPred::Sge => signed != Less,
        ICmpPred::Slt => signed == Less,
        ICmpPred::Sle => signed != Greater,
    })
}

/// The predicate that gives the same answer with the operands swapped.
pub fn swapped(pred: ICmpPred) -> ICmpPred {
    match pred {
        ICmpPred::Ugt => ICmpPred::Ult,
        ICmpPred::Uge => ICmpPred::Ule,
        ICmpPred::Ult => ICmpPred::Ugt,
        ICmpPred::Ule => ICmpPred::Uge,
        ICmpPred::Sgt => ICmpPred::Slt,
        ICmpPred::Sge => ICmpPred::Sle,
        ICmpPred::Slt => ICmpPred::Sgt,
        ICmpPred::Sle => ICmpPred::Sge,
        p => p,
    }
}

/// Folds an integer-to-integer cast.
pub fn fold_cast(op: CastOp, flags: IntFlags, v: u64, from: u32, to: u32) -> Option<Folded> {
    let v = v & mask(from);
    let r = match op {
        CastOp::Trunc => {
            let r = v & mask(to);
            let nuw = flags.contains(IntFlags::NUW) && r != v;
            let nsw = flags.contains(IntFlags::NSW) && sext(r, to) != sext(v, from);
            return Some(if nuw || nsw { Folded::Poison } else { Folded::Int(r) });
        }
        CastOp::ZExt => {
            if flags.contains(IntFlags::NNEG) && sext(v, from) < 0 {
                return Some(Folded::Poison);
            }
            v
        }
        CastOp::SExt => sext(v, from) as u64 & mask(to),
        CastOp::BitCast if from == to => v,
        _ => return None,
    };
    Some(Folded::Int(r))
}

/// Folds a binary float operation on `float` or `double` constants, given
/// their bits. Fast-math flags are the caller's concern.
pub fn fold_float(op: BinOp, kind: FloatKind, a: u128, b: u128) -> Option<u128> {
    match kind {
        FloatKind::Float => {
            let (x, y) = (f32::from_bits(a as u32), f32::from_bits(b as u32));
            let r = match op {
                BinOp::FAdd => x + y,
                BinOp::FSub => x - y,
                BinOp::FMul => x * y,
                BinOp::FDiv => x / y,
                BinOp::FRem => x % y,
                _ => return None,
            };
            Some(r.to_bits() as u128)
        }
        FloatKind::Double => {
            let (x, y) = (f64::from_bits(a as u64), f64::from_bits(b as u64));
            let r = match op {
                BinOp::FAdd => x + y,
                BinOp::FSub => x - y,
                BinOp::FMul => x * y,
                BinOp::FDiv => x / y,
                BinOp::FRem => x % y,
                _ => return None,
            };
            Some(r.to_bits() as u128)
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn folding_follows_the_language_reference() {
        let f = |op, flags, a, b| fold_binop(op, flags, a, b, 8);
        assert_eq!(f(BinOp::Add, IntFlags::NONE, 200, 100), Some(Folded::Int(44)));
        assert_eq!(f(BinOp::Add, IntFlags::NUW, 200, 100), Some(Folded::Poison));
        assert_eq!(f(BinOp::Add, IntFlags::NSW, 100, 100), Some(Folded::Poison));
        assert_eq!(f(BinOp::Shl, IntFlags::NONE, 1, 8), Some(Folded::Poison));
        assert_eq!(f(BinOp::UDiv, IntFlags::NONE, 1, 0), None);
        assert_eq!(f(BinOp::SDiv, IntFlags::NONE, 0x80, 0xff), None);
        assert_eq!(f(BinOp::SRem, IntFlags::NONE, 0xf9, 2), Some(Folded::Int(0xff)));
        assert_eq!(f(BinOp::AShr, IntFlags::NONE, 0x80, 7), Some(Folded::Int(0xff)));
        assert!(fold_icmp(ICmpPred::Slt, 0xff, 0, 8));
        assert!(!fold_icmp(ICmpPred::Ult, 0xff, 0, 8));
        assert_eq!(fold_cast(CastOp::SExt, IntFlags::NONE, 0x80, 8, 16), Some(Folded::Int(0xff80)));
    }
}
