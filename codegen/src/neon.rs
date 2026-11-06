//! AArch64's own intrinsics (`llvm.aarch64.*`) that generic IR says as
//! well, as that IR (after `prepare::neon_intrinsics`, which renames those
//! LLVM has generic intrinsics for):
//!
//! - SHA-3's logic (`eor3`, `bcax`, `rax1`, `xar`) as exclusive ors, ands
//!   and rotations.
//! - Conversions to and from fixed point (`vcvtfp2fxs`...) as products by
//!   a power of two, which are exact, and the saturating conversions.
//! - Widening products (`smull`, `pmull`...) as products of the operands
//!   extended; saturating doubling ones (`sqdmull`, `sqdmulh` of
//!   scalars...) as saturating sums of a wide product with itself.
//! - By-lane forms (`sqdmulh.lane`) as of a splat of the lane; lanes
//!   copied (`vcopy.lane`) as an extraction and an insertion.
//! - Tables of two to four registers (`tbl2`, `tbx3`...) as lookups in
//!   each, ored, out-of-range bytes the old value's for `tbx`.
//! - Structured loads and stores (`ld2`, `st3lane`, `ld1x4`, `ld4r`...)
//!   as loads and stores of the elements, and shuffles.
//! - Divisions (`llvm.aarch64.sdiv`) as the instruction's: zero for a
//!   divisor of zero, the least number for it divided by -1.
//! - Scalar saturating narrowings (`scalar.sqxtn`) as clamps; `cls` as a
//!   count of leading zeros; prefetches as nothing.

use epitome_ir::module::*;
use epitome_ir::types::{Type, TypeId};

use crate::intrinsics::mangle;
use crate::prepare::{Ctx, Seq, call_intrinsic, callee_name};

pub fn lower(cx: &mut Ctx, f: &mut Function) {
    let blocks: Vec<BlockId> = f.layout.clone();
    for b in blocks {
        let mut k = 0;
        while k < f.blocks[b.index()].insts.len() {
            let i = f.blocks[b.index()].insts[k];
            let Op::Call(call) = &f.insts[i.index()].op else {
                k += 1;
                continue;
            };
            let Some(name) = callee_name(cx.m, call).filter(|n| n.starts_with("llvm.aarch64.")) else {
                k += 1;
                continue;
            };
            let args: Vec<Value> = call.args.iter().map(|a| a.value).collect();
            let result = f.insts[i.index()].result;
            let ty = result.map(|r| f.locals[r.index()].ty);
            let mut s = Seq { f, new: Vec::new() };
            let Some(v) = rewrite(cx, &mut s, &name, &args, ty) else {
                k += 1;
                continue;
            };
            let new = std::mem::take(&mut s.new);
            let n = new.len();
            f.blocks[b.index()].insts.splice(k..=k, new);
            if let (Some(v), Some(r)) = (v, result) {
                f.replace_uses(Value::Local(r), v);
            }
            k += n;
        }
    }
}

/// The IR for a call to `name`: `Some(Some(v))` its value, `Some(None)`
/// done (a call without a result), `None` left as it is.
fn rewrite(cx: &mut Ctx, s: &mut Seq, name: &str, args: &[Value], ty: Option<TypeId>) -> Option<Option<Value>> {
    let rest = name.strip_prefix("llvm.aarch64.")?;
    // The operation's name: `sqxtn` of `neon.sqxtn.v8i8`, `scalar.sqxtn` of
    // `neon.scalar.sqxtn.i32.i64`.
    let body = rest.strip_prefix("neon.").or(rest.strip_prefix("crypto.")).unwrap_or(rest);
    let op = match body.strip_prefix("scalar.") {
        Some(x) => format!("scalar.{}", x.split('.').next().unwrap_or("")),
        None => body.split('.').next().unwrap_or("").to_string(),
    };
    let op = op.as_str();
    // Those without a result.
    match op {
        "prefetch" => return Some(None),
        "st1x2" | "st1x3" | "st1x4" | "st2" | "st3" | "st4" | "st2lane" | "st3lane" | "st4lane" => {
            store_structure(cx, s, op, args);
            return Some(None);
        }
        _ => {}
    }
    let ty = ty?;
    let bin = |s: &mut Seq, op, a, b, ty| s.bin(op, a, b, ty);
    Some(Some(match op {
        "eor3s" | "eor3u" => {
            let x = bin(s, BinOp::Xor, args[0], args[1], ty);
            bin(s, BinOp::Xor, x, args[2], ty)
        }
        "bcaxs" | "bcaxu" => {
            let ones = splat_int(cx, ty, -1);
            let nc = bin(s, BinOp::Xor, args[2], ones, ty);
            let x = bin(s, BinOp::And, args[1], nc, ty);
            bin(s, BinOp::Xor, args[0], x, ty)
        }
        "rax1" => {
            let one = splat_int(cx, ty, 1);
            let r = call_intrinsic(cx, s, &format!("llvm.fshl.{}", mangle(cx, ty)), vec![args[1], args[1], one], ty);
            bin(s, BinOp::Xor, args[0], r, ty)
        }
        "xar" => {
            let x = bin(s, BinOp::Xor, args[0], args[1], ty);
            let k = cx.const_int(args[2])?;
            let amount = splat_int(cx, ty, k);
            call_intrinsic(cx, s, &format!("llvm.fshr.{}", mangle(cx, ty)), vec![x, x, amount], ty)
        }
        "vcvtfp2fxs" | "vcvtfp2fxu" => {
            let fbits = cx.const_int(args[1])?;
            let fty = cx.value_ty(s.f, args[0]);
            let scale = power_of_two(cx, fty, fbits)?;
            let x = s.bin(BinOp::FMul, args[0], scale, fty);
            let sat = if op == "vcvtfp2fxs" { "fptosi" } else { "fptoui" };
            call_intrinsic(cx, s, &format!("llvm.{sat}.sat.{}.{}", mangle(cx, ty), mangle(cx, fty)), vec![x], ty)
        }
        "vcvtfxs2fp" | "vcvtfxu2fp" => {
            let fbits = cx.const_int(args[1])?;
            let c = if op == "vcvtfxs2fp" { CastOp::SIToFP } else { CastOp::UIToFP };
            let x = s.cast(c, args[0], ty);
            let scale = power_of_two(cx, ty, -fbits)?;
            s.bin(BinOp::FMul, x, scale, ty)
        }
        "smull" | "umull" => {
            let ext = if op == "smull" { CastOp::SExt } else { CastOp::ZExt };
            let (a, b) = (s.cast(ext, args[0], ty), s.cast(ext, args[1], ty));
            s.bin(BinOp::Mul, a, b, ty)
        }
        // Doubled, saturating: the product doubled by a saturating sum
        // with itself (only the least number squared overflows).
        "sqdmull" | "sqdmulls" => {
            let (a, b) = (s.cast(CastOp::SExt, args[0], ty), s.cast(CastOp::SExt, args[1], ty));
            let p = s.bin(BinOp::Mul, a, b, ty);
            call_intrinsic(cx, s, &format!("llvm.sadd.sat.{}", mangle(cx, ty)), vec![p, p], ty)
        }
        // Of scalars: the doubled product's high half (rounded: plus half
        // of its low half's range first).
        "sqdmulh" | "sqrdmulh" if matches!(cx.m.types.get(ty), Type::Int(32)) => {
            let i64t = cx.m.types.int(64);
            let (a, b) = (s.cast(CastOp::SExt, args[0], i64t), s.cast(CastOp::SExt, args[1], i64t));
            let p = s.bin(BinOp::Mul, a, b, i64t);
            let mut d = call_intrinsic(cx, s, "llvm.sadd.sat.i64", vec![p, p], i64t);
            if op == "sqrdmulh" {
                let half = cx.int(i64t, 1 << 31);
                d = call_intrinsic(cx, s, "llvm.sadd.sat.i64", vec![d, half], i64t);
            }
            let sh = cx.int(i64t, 32);
            let hi = s.bin(BinOp::AShr, d, sh, i64t);
            s.cast(CastOp::Trunc, hi, ty)
        }
        "sqdmulh" | "sqrdmulh" if rest.contains(".lane") => {
            // (`sqdmulh.lane` and `.laneq`: of the lane splatted.)
            let k = cx.const_int(args[2])? as u32;
            let (n, _) = lanes(cx, ty)?;
            let splat = shuffle(cx, s, args[1], args[1], &vec![k; n as usize], ty);
            let base = if op == "sqdmulh" { "sqdmulh" } else { "sqrdmulh" };
            call_intrinsic(cx, s, &format!("llvm.aarch64.neon.{base}.{}", mangle(cx, ty)), vec![args[0], splat], ty)
        }
        "pmul" | "pmull" => {
            let x = if op == "pmull" { s.cast(CastOp::ZExt, args[0], ty) } else { args[0] };
            let y = if op == "pmull" { s.cast(CastOp::ZExt, args[1], ty) } else { args[1] };
            call_intrinsic(cx, s, &format!("llvm.clmul.{}", mangle(cx, ty)), vec![x, y], ty)
        }
        "pmull64" => {
            let i128t = cx.m.types.int(128);
            let (x, y) = (s.cast(CastOp::ZExt, args[0], i128t), s.cast(CastOp::ZExt, args[1], i128t));
            let p = call_intrinsic(cx, s, "llvm.clmul.i128", vec![x, y], i128t);
            s.cast(CastOp::BitCast, p, ty)
        }
        "vcopy" if rest.starts_with("neon.vcopy.lane") => {
            let (_, elem) = lanes(cx, ty)?;
            let x = s.op(Op::ExtractElement { vector: args[2], index: args[3] }, elem);
            s.op(Op::InsertElement { vector: args[0], element: x, index: args[1] }, ty)
        }
        "tbl2" | "tbl3" | "tbl4" | "tbx1" | "tbx2" | "tbx3" | "tbx4" => table(cx, s, op, args, ty),
        "ld1x2" | "ld1x3" | "ld1x4" | "ld2" | "ld3" | "ld4" | "ld2r" | "ld3r" | "ld4r" | "ld2lane" | "ld3lane"
        | "ld4lane" => load_structure(cx, s, op, args, ty)?,
        // Of floats, addp is faddp.
        "addp" if lanes(cx, ty).is_some_and(|(_, e)| matches!(cx.m.types.get(e), Type::Float | Type::Double)) => {
            call_intrinsic(cx, s, &format!("llvm.aarch64.neon.faddp.{}", mangle(cx, ty)), vec![args[0], args[1]], ty)
        }
        // Of a scalar: of a value not negative, shifted unsigned with
        // saturation; of a negative one, 0.
        "sqshlu" if matches!(cx.m.types.get(ty), Type::Int(_)) => {
            let i1 = cx.m.types.int(1);
            let zero = cx.int(ty, 0);
            let neg = s.op(Op::ICmp { pred: ICmpPred::Slt, samesign: false, lhs: args[0], rhs: zero }, i1);
            let sh = call_intrinsic(cx, s, &format!("llvm.ushl.sat.{}", mangle(cx, ty)), vec![args[0], args[1]], ty);
            s.op(Op::Select { cond: neg, then_value: zero, else_value: sh, fmf: FastMath::default() }, ty)
        }
        // Of 64 bits (as one lane): the shifted bits, the old value's
        // others.
        "vsli" | "vsri" if lanes(cx, ty).is_some_and(|(n, _)| n == 1) => {
            let i64t = cx.m.types.int(64);
            let k = cx.const_int(args[2])?;
            let i0 = cx.int(i64t, 0);
            let old = s.op(Op::ExtractElement { vector: args[0], index: i0 }, i64t);
            let x = s.op(Op::ExtractElement { vector: args[1], index: i0 }, i64t);
            let kk = cx.int(i64t, k);
            let (shifted, keep) = if op == "vsli" {
                (s.bin(BinOp::Shl, x, kk, i64t), cx.int(i64t, ((1u64 << k) - 1) as i64))
            } else {
                (s.bin(BinOp::LShr, x, kk, i64t), cx.int(i64t, !(u64::MAX >> k) as i64))
            };
            let kept = s.bin(BinOp::And, old, keep, i64t);
            let r = s.bin(BinOp::Or, shifted, kept, i64t);
            let poison = Value::Const(cx.m.consts.intern(Constant { ty, kind: ConstKind::Poison }));
            s.op(Op::InsertElement { vector: poison, element: r, index: i0 }, ty)
        }
        // Into one lane of 64 bits: the sum of the two lanes, extended.
        "saddlp" | "uaddlp" if lanes(cx, ty).is_some_and(|(n, _)| n == 1) => {
            let i64t = cx.m.types.int(64);
            let (_, elem) = lanes(cx, cx.value_ty(s.f, args[0]))?;
            let ext = if op == "saddlp" { CastOp::SExt } else { CastOp::ZExt };
            let (i0, i1v) = (cx.int(i64t, 0), cx.int(i64t, 1));
            let a = s.op(Op::ExtractElement { vector: args[0], index: i0 }, elem);
            let b = s.op(Op::ExtractElement { vector: args[0], index: i1v }, elem);
            let (a, b) = (s.cast(ext, a, i64t), s.cast(ext, b, i64t));
            let sum = s.bin(BinOp::Add, a, b, i64t);
            let poison = Value::Const(cx.m.consts.intern(Constant { ty, kind: ConstKind::Poison }));
            s.op(Op::InsertElement { vector: poison, element: sum, index: i0 }, ty)
        }
        "sdiv" | "udiv" => {
            // The divisor 1 where it is 0 (and the result 0), or where it
            // is -1 and the dividend the least (whose quotient is itself).
            let i1 = cx.m.types.int(1);
            let zero = cx.int(ty, 0);
            let one = cx.int(ty, 1);
            let is_zero = s.op(Op::ICmp { pred: ICmpPred::Eq, samesign: false, lhs: args[1], rhs: zero }, i1);
            let mut bad = is_zero;
            if op == "sdiv" {
                let bits = match cx.m.types.get(ty) {
                    Type::Int(b) => *b,
                    _ => 64,
                };
                let min = cx.int(ty, i64::MIN >> (64 - bits));
                let m1 = cx.int(ty, -1);
                let a_min = s.op(Op::ICmp { pred: ICmpPred::Eq, samesign: false, lhs: args[0], rhs: min }, i1);
                let b_m1 = s.op(Op::ICmp { pred: ICmpPred::Eq, samesign: false, lhs: args[1], rhs: m1 }, i1);
                let over = s.bin(BinOp::And, a_min, b_m1, i1);
                bad = s.bin(BinOp::Or, is_zero, over, i1);
            }
            let d = s.op(Op::Select { cond: bad, then_value: one, else_value: args[1], fmf: FastMath::default() }, ty);
            let q = s.bin(if op == "sdiv" { BinOp::SDiv } else { BinOp::UDiv }, args[0], d, ty);
            s.op(Op::Select { cond: is_zero, then_value: zero, else_value: q, fmf: FastMath::default() }, ty)
        }
        "scalar.sqxtn" | "scalar.uqxtn" | "scalar.sqxtun" => {
            let wty = cx.value_ty(s.f, args[0]);
            let bits = match cx.m.types.get(ty) {
                Type::Int(b) => *b,
                _ => 32,
            };
            let wm = mangle(cx, wty);
            let x = match op {
                "scalar.sqxtn" => {
                    let hi = cx.int(wty, (1i64 << (bits - 1)) - 1);
                    let lo = cx.int(wty, -(1i64 << (bits - 1)));
                    let y = call_intrinsic(cx, s, &format!("llvm.smin.{wm}"), vec![args[0], hi], wty);
                    call_intrinsic(cx, s, &format!("llvm.smax.{wm}"), vec![y, lo], wty)
                }
                "scalar.uqxtn" => {
                    let hi = cx.int(wty, ((1u64 << bits) - 1) as i64);
                    call_intrinsic(cx, s, &format!("llvm.umin.{wm}"), vec![args[0], hi], wty)
                }
                _ => {
                    let hi = cx.int(wty, ((1u64 << bits) - 1) as i64);
                    let zero = cx.int(wty, 0);
                    let y = call_intrinsic(cx, s, &format!("llvm.smin.{wm}"), vec![args[0], hi], wty);
                    call_intrinsic(cx, s, &format!("llvm.smax.{wm}"), vec![y, zero], wty)
                }
            };
            s.cast(CastOp::Trunc, x, ty)
        }
        // Leading bits like the sign, below it: the leading zeros of the
        // value exclusive-ored with its sign, less one.
        "cls" | "cls64" if matches!(cx.m.types.get(ty), Type::Int(_)) => {
            let xty = cx.value_ty(s.f, args[0]);
            let bits = match cx.m.types.get(xty) {
                Type::Int(b) => *b,
                _ => 64,
            };
            let sh = cx.int(xty, bits as i64 - 1);
            let sign = s.bin(BinOp::AShr, args[0], sh, xty);
            let x = s.bin(BinOp::Xor, args[0], sign, xty);
            let i1 = cx.m.types.int(1);
            let f = cx.int(i1, 0);
            let z = call_intrinsic(cx, s, &format!("llvm.ctlz.{}", mangle(cx, xty)), vec![x, f], xty);
            let one = cx.int(xty, 1);
            let c = s.bin(BinOp::Sub, z, one, xty);
            if xty == ty { c } else { s.cast(CastOp::Trunc, c, ty) }
        }
        _ => return None,
    }))
}

fn lanes(cx: &Ctx, ty: TypeId) -> Option<(u32, TypeId)> {
    match cx.m.types.get(ty) {
        Type::Vector { len, elem, scalable: false } => Some((*len, *elem)),
        _ => None,
    }
}

/// A constant of integer type `ty` (splatted, for a vector).
fn splat_int(cx: &mut Ctx, ty: TypeId, v: i64) -> Value {
    let elem = lanes(cx, ty).map_or(ty, |l| l.1);
    let Value::Const(c) = cx.int(elem, v) else { unreachable!() };
    if elem == ty {
        Value::Const(c)
    } else {
        Value::Const(cx.m.consts.intern(Constant { ty, kind: ConstKind::Splat(c) }))
    }
}

/// 2 to the `k` as a constant of floating-point type `ty` (or a splat).
fn power_of_two(cx: &mut Ctx, ty: TypeId, k: i64) -> Option<Value> {
    let elem = lanes(cx, ty).map_or(ty, |l| l.1);
    let bits = match cx.m.types.get(elem) {
        Type::Float => ((127 + k) as u128) << 23,
        Type::Double => ((1023 + k) as u128) << 52,
        _ => return None,
    };
    let c = cx.m.consts.intern(Constant { ty: elem, kind: ConstKind::Float(bits) });
    Some(if elem == ty {
        Value::Const(c)
    } else {
        Value::Const(cx.m.consts.intern(Constant { ty, kind: ConstKind::Splat(c) }))
    })
}

fn shuffle(cx: &mut Ctx, s: &mut Seq, a: Value, b: Value, lanes: &[u32], ty: TypeId) -> Value {
    let i32t = cx.m.types.int(32);
    let ids: Vec<ConstId> = lanes
        .iter()
        .map(|&l| {
            let Value::Const(c) = cx.int(i32t, l as i64) else { unreachable!() };
            c
        })
        .collect();
    let mty = cx.m.types.intern(Type::Vector { len: lanes.len() as u32, elem: i32t, scalable: false });
    let mask = cx.m.consts.intern(Constant { ty: mty, kind: ConstKind::Vector(ids) });
    s.op(Op::ShuffleVector { a, b, mask }, ty)
}

/// A lookup in tables of one to four registers (the last argument the
/// indices): each table's lookup by the indices less its start (bytes
/// before it wrap to out of range, which looks up 0), ored; for `tbx`,
/// where the index is past all the tables, the old value's byte.
fn table(cx: &mut Ctx, s: &mut Seq, op: &str, args: &[Value], ty: TypeId) -> Value {
    let tbx = op.starts_with("tbx");
    let (old, tables) = if tbx { (Some(args[0]), &args[1..args.len() - 1]) } else { (None, &args[..args.len() - 1]) };
    let idx = *args.last().unwrap();
    let name = format!("llvm.aarch64.neon.tbl1.{}", mangle(cx, ty));
    let mut r: Option<Value> = None;
    for (k, &t) in tables.iter().enumerate() {
        let i = if k == 0 {
            idx
        } else {
            let off = splat_int(cx, ty, 16 * k as i64);
            s.bin(BinOp::Sub, idx, off, ty)
        };
        let x = call_intrinsic(cx, s, &name, vec![t, i], ty);
        r = Some(match r {
            Some(a) => s.bin(BinOp::Or, a, x, ty),
            None => x,
        });
    }
    let r = r.unwrap();
    let Some(old) = old else { return r };
    let (n, _) = lanes(cx, ty).unwrap();
    let i1 = cx.m.types.int(1);
    let mty = cx.m.types.intern(Type::Vector { len: n, elem: i1, scalable: false });
    let end = splat_int(cx, ty, 16 * tables.len() as i64);
    let inside = s.op(Op::ICmp { pred: ICmpPred::Ult, samesign: false, lhs: idx, rhs: end }, mty);
    s.op(Op::Select { cond: inside, then_value: r, else_value: old, fmf: FastMath::default() }, ty)
}

fn gep(cx: &mut Ctx, s: &mut Seq, elem: TypeId, p: Value, k: i64) -> Value {
    let i64t = cx.m.types.int(64);
    let ptr = cx.m.types.ptr(0);
    let k = cx.int(i64t, k);
    s.op(Op::Gep { flags: GepFlags(0), source_ty: elem, ptr: p, indices: vec![k].into() }, ptr)
}

fn load(s: &mut Seq, p: Value, ty: TypeId) -> Value {
    s.op(Op::Load { ptr: p, align: Some(Align::of(1)), volatile: false, atomic: None }, ty)
}

/// A structured load (a struct of vectors): `ld1xN` consecutive vectors;
/// `ldN` elements interleaved (lane `j` of vector `q` the element `j * N +
/// q`); `ldNr` each vector a splat of the next element; `ldNlane` the next
/// element in each vector's lane.
fn load_structure(cx: &mut Ctx, s: &mut Seq, op: &str, args: &[Value], ty: TypeId) -> Option<Value> {
    let Type::Struct { fields, .. } = cx.m.types.get(ty).clone() else { return None };
    let vty = fields[0];
    let (n, elem) = lanes(cx, vty)?;
    let parts = fields.len() as u32;
    let i64t = cx.m.types.int(64);
    let ptr = *args.last()?;
    let mut vs = Vec::new();
    if op.starts_with("ld1x") {
        for q in 0..parts {
            let p = gep(cx, s, vty, ptr, q as i64);
            vs.push(load(s, p, vty));
        }
    } else if op.ends_with('r') {
        for q in 0..parts {
            let p = gep(cx, s, elem, ptr, q as i64);
            let x = load(s, p, elem);
            let poison = Value::Const(cx.m.consts.intern(Constant { ty: vty, kind: ConstKind::Poison }));
            let zero = cx.int(i64t, 0);
            let one = s.op(Op::InsertElement { vector: poison, element: x, index: zero }, vty);
            vs.push(shuffle(cx, s, one, one, &vec![0; n as usize], vty));
        }
    } else if op.ends_with("lane") {
        let lane = args[parts as usize];
        for q in 0..parts {
            let p = gep(cx, s, elem, ptr, q as i64);
            let x = load(s, p, elem);
            vs.push(s.op(Op::InsertElement { vector: args[q as usize], element: x, index: lane }, vty));
        }
    } else {
        let wide = cx.m.types.intern(Type::Vector { len: n * parts, elem, scalable: false });
        let all = load(s, ptr, wide);
        for q in 0..parts {
            let l: Vec<u32> = (0..n).map(|j| j * parts + q).collect();
            vs.push(shuffle(cx, s, all, all, &l, vty));
        }
    }
    let mut agg = Value::Const(cx.m.consts.intern(Constant { ty, kind: ConstKind::Poison }));
    for (q, v) in vs.into_iter().enumerate() {
        agg = s.op(Op::InsertValue { aggregate: agg, element: v, indices: vec![q as u32].into() }, ty);
    }
    Some(agg)
}

/// A structured store, as `load_structure`'s loads are: `st1xN`
/// consecutive vectors, `stN` interleaved, `stNlane` each vector's lane
/// at the next element.
fn store_structure(cx: &mut Ctx, s: &mut Seq, op: &str, args: &[Value]) {
    let ptr = *args.last().unwrap();
    let lane_store = op.ends_with("lane");
    let parts = if lane_store { args.len() - 2 } else { args.len() - 1 };
    let vty = cx.value_ty(s.f, args[0]);
    let (n, elem) = lanes(cx, vty).unwrap();
    let store = |s: &mut Seq, v: Value, p: Value| {
        s.op_void(Op::Store { volatile: false, atomic: None, value: v, ptr: p, align: Some(Align::of(1)) })
    };
    let vectors = &args[..parts];
    if op.starts_with("st1x") {
        for (q, &v) in vectors.iter().enumerate() {
            let p = gep(cx, s, vty, ptr, q as i64);
            store(s, v, p);
        }
    } else if lane_store {
        let lane = args[parts];
        for (q, &v) in vectors.iter().enumerate() {
            let x = s.op(Op::ExtractElement { vector: v, index: lane }, elem);
            let p = gep(cx, s, elem, ptr, q as i64);
            store(s, x, p);
        }
    } else {
        // Element j * N + q, lane j of vector q: each element stored.
        let i64t = cx.m.types.int(64);
        for j in 0..n {
            for (q, &v) in vectors.iter().enumerate() {
                let idx = cx.int(i64t, j as i64);
                let x = s.op(Op::ExtractElement { vector: v, index: idx }, elem);
                let p = gep(cx, s, elem, ptr, (j as usize * parts + q) as i64);
                store(s, x, p);
            }
        }
    }
}
