//! Integers wider than 128 bits (`i1088`, as SROA makes of a struct it
//! moves whole): each value becomes 64-bit limbs, low first (the last
//! narrower when the width is not a multiple of 64), and each operation on
//! it operations on the limbs. It runs before `wide.rs`, which then sees
//! only `i128`s and narrower.
//!
//! What such integers are used for is moving memory, and taking parts of
//! it: loads and stores (volatile ones too), phis, selects, freezes,
//! `and`/`or`/`xor`, shifts by constants (of widths a multiple of 64),
//! extensions to them, truncations of them, and comparisons for equality.
//! Anything else of them is an error naming it.

use std::collections::HashMap;

use epitome_ir::module::*;
use epitome_ir::types::{Type, TypeId};

use crate::prepare::{Ctx, Seq};

fn huge_bits(cx: &Ctx, ty: TypeId) -> Option<u32> {
    match cx.m.types.get(ty) {
        Type::Int(b) if *b > 128 => Some(*b),
        _ => None,
    }
}

/// The types of the limbs of an integer of `bits` bits.
fn limb_types(cx: &mut Ctx, bits: u32) -> Vec<TypeId> {
    let n = bits.div_ceil(64);
    (0..n).map(|k| cx.m.types.int(if k + 1 < n { 64 } else { bits - 64 * (n - 1) })).collect()
}

/// Whether instruction `i` has a huge integer as its result or an operand.
fn touches(cx: &Ctx, f: &Function, i: InstId) -> bool {
    let inst = &f.insts[i.index()];
    let mut found = inst.result.is_some_and(|r| huge_bits(cx, f.locals[r.index()].ty).is_some());
    inst.op.for_each_value(|v| {
        if !matches!(v, Value::Metadata(_)) {
            found |= huge_bits(cx, cx.value_ty(f, *v)).is_some();
        }
    });
    found
}

pub fn split_huge(cx: &mut Ctx, f: &mut Function) -> Result<(), String> {
    let all: Vec<InstId> = f.insts_in_order().collect();
    if !all.iter().any(|&i| touches(cx, f, i)) {
        return Ok(());
    }
    let mut limbs: HashMap<LocalId, Vec<Value>> = HashMap::new();
    let mut pending: Vec<(Vec<InstId>, InstId)> = Vec::new();
    let order = crate::wide::reverse_postorder(f);
    for b in order {
        let insts = f.blocks[b.index()].insts.clone();
        let mut out = Vec::new();
        for i in insts {
            if !touches(cx, f, i) {
                out.push(i);
                continue;
            }
            let mut s = Seq { f, new: Vec::new() };
            let result = split_inst(cx, &mut s, &mut limbs, &mut pending, i)?;
            let new = std::mem::take(&mut s.new);
            out.extend(new);
            // A narrow result (a truncation, a comparison) takes the old
            // one's place.
            if let (Some(v), Some(r)) = (result, f.insts[i.index()].result) {
                f.replace_uses(Value::Local(r), v);
            }
        }
        f.blocks[b.index()].insts = out;
    }
    // The phis' incoming limbs, now that each value has them.
    for (phis, orig) in pending {
        let Op::Phi { incoming, .. } = f.insts[orig.index()].op.clone() else { unreachable!() };
        let bits = huge_bits(cx, f.locals[f.insts[orig.index()].result.unwrap().index()].ty).unwrap();
        for (v, pred) in incoming {
            let ls = limbs_of(cx, &limbs, f, v, bits)?;
            for (k, &p) in phis.iter().enumerate() {
                if let Op::Phi { incoming, .. } = &mut f.insts[p.index()].op {
                    incoming.push((ls[k], pred));
                }
            }
        }
    }
    Ok(())
}

/// The limbs of `v`, an integer of `bits` bits: a value's, or a constant's.
fn limbs_of(
    cx: &mut Ctx,
    limbs: &HashMap<LocalId, Vec<Value>>,
    f: &Function,
    v: Value,
    bits: u32,
) -> Result<Vec<Value>, String> {
    match v {
        Value::Local(l) => limbs.get(&l).cloned().ok_or_else(|| "a huge integer used before it is made".into()),
        Value::Const(c) => {
            let k = cx.m.constant(c).clone();
            let tys = limb_types(cx, bits);
            Ok(tys
                .iter()
                .enumerate()
                .map(|(n, &t)| {
                    let kind = match &k.kind {
                        ConstKind::Int(a) => {
                            let w = a.words().get(n).copied().unwrap_or(0);
                            let lb = match cx.m.types.get(t) {
                                Type::Int(b) => *b,
                                _ => 64,
                            };
                            ConstKind::Int(epitome_ir::apint::ApInt::from_u64(
                                lb,
                                if lb < 64 { w & ((1u64 << lb) - 1) } else { w },
                            ))
                        }
                        ConstKind::Undef => ConstKind::Undef,
                        ConstKind::Poison => ConstKind::Poison,
                        _ => ConstKind::Zero,
                    };
                    Value::Const(cx.m.consts.intern(Constant { ty: t, kind }))
                })
                .collect())
        }
        Value::Metadata(_) => {
            let _ = f;
            Err("metadata as a huge integer".into())
        }
    }
}

/// Instruction `i` on limbs: their values recorded, or the narrow value it
/// now is.
fn split_inst(
    cx: &mut Ctx,
    s: &mut Seq,
    limbs: &mut HashMap<LocalId, Vec<Value>>,
    pending: &mut Vec<(Vec<InstId>, InstId)>,
    i: InstId,
) -> Result<Option<Value>, String> {
    let inst = s.f.insts[i.index()].clone();
    let rty = inst.result.map(|r| s.f.locals[r.index()].ty);
    let rbits = rty.and_then(|t| huge_bits(cx, t));
    let i64t = cx.m.types.int(64);
    let i8t = cx.m.types.int(8);
    let ptr = cx.m.types.ptr(0);
    let set = |limbs: &mut HashMap<LocalId, Vec<Value>>, ls: Vec<Value>| {
        limbs.insert(inst.result.unwrap(), ls);
    };
    let unsupported = |what: &str| Err(format!("{what} of an integer wider than 128 bits is not supported yet"));
    match inst.op {
        Op::Phi { fmf, .. } => {
            let tys = limb_types(cx, rbits.unwrap());
            let phis: Vec<InstId> =
                tys.iter().map(|&t| s.f.add_inst(Op::Phi { fmf, incoming: vec![] }, Some((None, t)))).collect();
            s.new.extend(phis.iter().copied());
            let ls = phis.iter().map(|&p| s.f.result(p)).collect();
            set(limbs, ls);
            pending.push((phis, i));
            Ok(None)
        }
        Op::Load { volatile, atomic: None, ptr: p, align } => {
            let tys = limb_types(cx, rbits.unwrap());
            let mut ls = Vec::new();
            for (k, &t) in tys.iter().enumerate() {
                let off = cx.int(i64t, 8 * k as i64);
                let at = s.op(Op::Gep { flags: GepFlags(0), source_ty: i8t, ptr: p, indices: vec![off].into() }, ptr);
                let a = align.map(|a| a.min(Align::of(8)));
                ls.push(s.op(Op::Load { volatile, atomic: None, ptr: at, align: a }, t));
            }
            set(limbs, ls);
            Ok(None)
        }
        Op::Store { volatile, atomic: None, value, ptr: p, align } => {
            let bits = huge_bits(cx, cx.value_ty(s.f, value)).unwrap();
            let ls = limbs_of(cx, limbs, s.f, value, bits)?;
            for (k, &l) in ls.iter().enumerate() {
                let off = cx.int(i64t, 8 * k as i64);
                let at = s.op(Op::Gep { flags: GepFlags(0), source_ty: i8t, ptr: p, indices: vec![off].into() }, ptr);
                s.op_void(Op::Store {
                    volatile,
                    atomic: None,
                    value: l,
                    ptr: at,
                    align: align.map(|a| a.min(Align::of(8))),
                });
            }
            Ok(None)
        }
        Op::Select { fmf, cond, then_value, else_value } if rbits.is_some() => {
            let bits = rbits.unwrap();
            let (a, b) = (limbs_of(cx, limbs, s.f, then_value, bits)?, limbs_of(cx, limbs, s.f, else_value, bits)?);
            let tys = limb_types(cx, bits);
            let ls = (0..a.len())
                .map(|k| s.op(Op::Select { fmf, cond, then_value: a[k], else_value: b[k] }, tys[k]))
                .collect();
            set(limbs, ls);
            Ok(None)
        }
        Op::Freeze(x) => {
            let bits = rbits.unwrap();
            let a = limbs_of(cx, limbs, s.f, x, bits)?;
            let tys = limb_types(cx, bits);
            let ls = (0..a.len()).map(|k| s.op(Op::Freeze(a[k]), tys[k])).collect();
            set(limbs, ls);
            Ok(None)
        }
        Op::Binary { op: op @ (BinOp::And | BinOp::Or | BinOp::Xor), lhs, rhs, .. } => {
            let bits = rbits.unwrap();
            let (a, b) = (limbs_of(cx, limbs, s.f, lhs, bits)?, limbs_of(cx, limbs, s.f, rhs, bits)?);
            let tys = limb_types(cx, bits);
            let ls = (0..a.len()).map(|k| s.bin(op, a[k], b[k], tys[k])).collect();
            set(limbs, ls);
            Ok(None)
        }
        Op::Binary { op: op @ (BinOp::Shl | BinOp::LShr | BinOp::AShr), lhs, rhs, .. } => {
            let bits = rbits.unwrap();
            let Some(c) = cx.const_int(rhs) else { return unsupported("a shift by a variable amount") };
            if !bits.is_multiple_of(64) {
                return unsupported("a shift");
            }
            let a = limbs_of(cx, limbs, s.f, lhs, bits)?;
            let n = a.len() as i64;
            let (q, r) = (c / 64, c % 64);
            let zero = cx.int(i64t, 0);
            let fill = if op == BinOp::AShr {
                let sh = cx.int(i64t, 63);
                s.bin(BinOp::AShr, a[n as usize - 1], sh, i64t)
            } else {
                zero
            };
            // Limb `j` of the source, or the fill beyond it.
            let src = |j: i64| if (0..n).contains(&j) { a[j as usize] } else { fill };
            let mut ls = Vec::new();
            for j in 0..n {
                let (lo, hi) =
                    if op == BinOp::Shl { (src(j - q - 1), src(j - q)) } else { (src(j + q), src(j + q + 1)) };
                let v = if r == 0 {
                    if op == BinOp::Shl { hi } else { lo }
                } else {
                    let rr = cx.int(i64t, r);
                    let rest = cx.int(i64t, 64 - r);
                    if op == BinOp::Shl {
                        let x = s.bin(BinOp::Shl, hi, rr, i64t);
                        let y = s.bin(BinOp::LShr, lo, rest, i64t);
                        s.bin(BinOp::Or, x, y, i64t)
                    } else {
                        let x = s.bin(BinOp::LShr, lo, rr, i64t);
                        let y = s.bin(BinOp::Shl, hi, rest, i64t);
                        s.bin(BinOp::Or, x, y, i64t)
                    }
                };
                ls.push(v);
            }
            set(limbs, ls);
            Ok(None)
        }
        Op::Cast { op: CastOp::Trunc, value, .. } => {
            let from = huge_bits(cx, cx.value_ty(s.f, value)).unwrap();
            let a = limbs_of(cx, limbs, s.f, value, from)?;
            let to_ty = rty.unwrap();
            let to = match cx.m.types.get(to_ty) {
                Type::Int(b) => *b,
                _ => return unsupported("a truncation to other than an integer"),
            };
            if to > 128 {
                let tys = limb_types(cx, to);
                let ls = (0..tys.len())
                    .map(|k| if cx.value_ty(s.f, a[k]) == tys[k] { a[k] } else { s.cast(CastOp::Trunc, a[k], tys[k]) })
                    .collect();
                set(limbs, ls);
                return Ok(None);
            }
            if to <= 64 {
                return Ok(Some(if to == 64 { a[0] } else { s.cast(CastOp::Trunc, a[0], to_ty) }));
            }
            // An i65 to i128: its two low limbs.
            let i128t = cx.m.types.int(128);
            let lo = s.cast(CastOp::ZExt, a[0], i128t);
            let hi = s.cast(CastOp::ZExt, a[1], i128t);
            let sh = cx.int(i128t, 64);
            let hi = s.bin(BinOp::Shl, hi, sh, i128t);
            let v = s.bin(BinOp::Or, lo, hi, i128t);
            Ok(Some(if to == 128 { v } else { s.cast(CastOp::Trunc, v, to_ty) }))
        }
        Op::Cast { op: op @ (CastOp::ZExt | CastOp::SExt), value, .. } => {
            let bits = rbits.unwrap();
            let tys = limb_types(cx, bits);
            let fty = cx.value_ty(s.f, value);
            let src: Vec<Value> = match huge_bits(cx, fty) {
                Some(fb) => limbs_of(cx, limbs, s.f, value, fb)?,
                None => {
                    let fb = match cx.m.types.get(fty) {
                        Type::Int(b) => *b,
                        _ => return unsupported("an extension of other than an integer"),
                    };
                    if fb <= 64 {
                        vec![if fb == 64 { value } else { s.cast(op, value, i64t) }]
                    } else {
                        let lo = s.cast(CastOp::Trunc, value, i64t);
                        let sh = cx.int(fty, 64);
                        let hi = s.bin(if op == CastOp::SExt { BinOp::AShr } else { BinOp::LShr }, value, sh, fty);
                        let hi = if fb == 128 {
                            s.cast(CastOp::Trunc, hi, i64t)
                        } else {
                            let hn = cx.m.types.int(fb - 64);
                            let h = s.cast(CastOp::Trunc, hi, hn);
                            s.cast(op, h, i64t)
                        };
                        vec![lo, hi]
                    }
                }
            };
            // (A narrower top limb of the source, extended to 64 first.)
            let mut src = src;
            if let Some(&last) = src.last()
                && cx.value_ty(s.f, last) != i64t
            {
                let n = src.len();
                src[n - 1] = s.cast(op, last, i64t);
            }
            let fill = if op == CastOp::SExt {
                let sh = cx.int(i64t, 63);
                s.bin(BinOp::AShr, *src.last().unwrap(), sh, i64t)
            } else {
                cx.int(i64t, 0)
            };
            let ls = (0..tys.len())
                .map(|k| {
                    let v = src.get(k).copied().unwrap_or(fill);
                    if tys[k] == i64t { v } else { s.cast(CastOp::Trunc, v, tys[k]) }
                })
                .collect();
            set(limbs, ls);
            Ok(None)
        }
        Op::ICmp { pred: pred @ (ICmpPred::Eq | ICmpPred::Ne), lhs, rhs, .. } => {
            let bits = huge_bits(cx, cx.value_ty(s.f, lhs)).unwrap();
            let (a, b) = (limbs_of(cx, limbs, s.f, lhs, bits)?, limbs_of(cx, limbs, s.f, rhs, bits)?);
            let tys = limb_types(cx, bits);
            let mut acc: Option<Value> = None;
            for k in 0..a.len() {
                let x = s.bin(BinOp::Xor, a[k], b[k], tys[k]);
                let x = if tys[k] == i64t { x } else { s.cast(CastOp::ZExt, x, i64t) };
                acc = Some(match acc {
                    Some(p) => s.bin(BinOp::Or, p, x, i64t),
                    None => x,
                });
            }
            let zero = cx.int(i64t, 0);
            let i1 = cx.m.types.int(1);
            Ok(Some(s.op(Op::ICmp { pred, samesign: false, lhs: acc.unwrap(), rhs: zero }, i1)))
        }
        ref op => {
            let name = format!("{op:?}");
            unsupported(name.split([' ', '(', '{']).next().unwrap_or("an operation"))
        }
    }
}
