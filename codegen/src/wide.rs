//! Integers wider than a register (`i65` to `i128`): each value becomes
//! two `i64` halves, low and high, and each operation on them operations on
//! the halves. It runs before anything else in `prepare`, so that the rest
//! of the code generator sees no wider integers, but for one shape: the
//! high half of a product, `trunc (lshr (mul (zext a), (zext b)), 64)`,
//! which descriptions have instructions for (`umulh`, `mulhu`).
//!
//! An `iN` narrower than 128 keeps its high half's bits above `N - 64`
//! undefined, as an odd width below 64 does: they are put right (zeroed or
//! sign-copied) only where they matter, before comparisons, right shifts,
//! divisions and extensions.
//!
//! At a call, a wide argument or result is a `[2 x i64]`, which the
//! conventions pass in two registers as they do a small struct, marked
//! `epitome.wide` so that a convention can align the pair (AAPCS64 puts
//! one in an even register and the next). Divisions and remainders call the
//! runtime's `__udivti3`, `__divti3`, `__umodti3` and `__modti3`.

use std::collections::HashMap;

use epitome_ir::apint::ApInt;
use epitome_ir::module::*;
use epitome_ir::types::{Type, TypeId};

use crate::prepare::Ctx;

/// The attribute that marks a wide value passed as `[2 x i64]`.
pub const WIDE: &str = "epitome.wide";

fn wide_bits(cx: &Ctx, ty: TypeId) -> Option<u32> {
    match cx.m.types.get(ty) {
        Type::Int(b) if *b > 64 && *b <= 128 => Some(*b),
        _ => None,
    }
}

type Pair = (Value, Value);

struct B<'f> {
    f: &'f mut Function,
    new: Vec<InstId>,
}

impl B<'_> {
    fn op(&mut self, op: Op, ty: TypeId) -> Value {
        let i = self.f.add_inst(op, Some((None, ty)));
        self.new.push(i);
        self.f.result(i)
    }

    fn bin(&mut self, op: BinOp, lhs: Value, rhs: Value, ty: TypeId) -> Value {
        self.op(Op::Binary { op, flags: IntFlags::NONE, fmf: FastMath::default(), lhs, rhs }, ty)
    }

    fn cast(&mut self, op: CastOp, value: Value, ty: TypeId) -> Value {
        self.op(Op::Cast { op, flags: IntFlags::NONE, fmf: FastMath::default(), value }, ty)
    }

    fn cmp(&mut self, pred: ICmpPred, lhs: Value, rhs: Value, i1: TypeId) -> Value {
        self.op(Op::ICmp { pred, samesign: false, lhs, rhs }, i1)
    }

    fn select(&mut self, cond: Value, a: Value, b: Value, ty: TypeId) -> Value {
        self.op(Op::Select { fmf: FastMath::default(), cond, then_value: a, else_value: b }, ty)
    }
}

struct Types {
    i1: TypeId,
    i64: TypeId,
    i128: TypeId,
    pair: TypeId,
}

/// Splits the function's wide integers into halves.
pub fn split_wide(cx: &mut Ctx, f: &mut Function) -> Result<(), String> {
    let (ret, params, varargs) = match cx.m.types.get(f.ty).clone() {
        Type::Function { ret, params, varargs } => (ret, params, varargs),
        _ => return Ok(()),
    };
    let any = f.locals.iter().any(|l| wide_bits(cx, l.ty).is_some())
        || wide_bits(cx, ret).is_some()
        || f.insts.iter().any(|i| {
            let mut found = false;
            i.op.for_each_value(|v| {
                if let Value::Const(c) = v {
                    found |= wide_bits(cx, cx.m.constant(*c).ty).is_some();
                }
            });
            found
        });
    if !any {
        return Ok(());
    }
    let i64t = cx.m.types.int(64);
    let ty = Types {
        i1: cx.m.types.int(1),
        i64: i64t,
        i128: cx.m.types.int(128),
        pair: cx.m.types.intern(Type::Array { len: 2, elem: i64t }),
    };
    let mut parts: HashMap<LocalId, Pair> = HashMap::new();
    let mut bits_of: HashMap<LocalId, u32> = HashMap::new();
    let entry = f.layout[0];

    // An invoke of a wide result gets a block of its own on its normal
    // edge, where the result's halves are taken out once it has returned.
    for b in f.layout.clone() {
        let Some(t) = f.terminator(b) else { continue };
        let Op::Invoke { normal, .. } = f.insts[t.index()].op else { continue };
        if !f.insts[t.index()].result.is_some_and(|r| wide_bits(cx, f.locals[r.index()].ty).is_some()) {
            continue;
        }
        let e = f.add_block(None);
        let at = f.layout.iter().position(|&x| x == b).unwrap();
        f.layout.insert(at + 1, e);
        f.push_inst(e, Op::Br(normal), None);
        if let Op::Invoke { normal: n, .. } = &mut f.insts[t.index()].op {
            *n = e;
        }
        f.retarget_phis(normal, b, e);
    }

    // Parameters: a wide one comes as a [2 x i64].
    let mut entry_new = Vec::new();
    let mut new_params = params.clone();
    for (k, p) in f.params.clone().iter().enumerate() {
        let Some(b) = wide_bits(cx, f.locals[p.local.index()].ty) else { continue };
        bits_of.insert(p.local, b);
        f.locals[p.local.index()].ty = ty.pair;
        f.params[k].attrs.push(Attr::Word(WIDE.into()));
        new_params[k] = ty.pair;
        let mut s = B { f, new: Vec::new() };
        let agg = Value::Local(p.local);
        let lo = s.op(Op::ExtractValue { aggregate: agg, indices: vec![0].into() }, ty.i64);
        let hi = s.op(Op::ExtractValue { aggregate: agg, indices: vec![1].into() }, ty.i64);
        entry_new.extend(s.new);
        parts.insert(p.local, (lo, hi));
    }
    let ret_bits = wide_bits(cx, ret);
    if ret_bits.is_some() {
        f.ret_attrs.push(Attr::Word(WIDE.into()));
    }
    let new_ret = if ret_bits.is_some() { ty.pair } else { ret };
    f.ty = cx.m.types.intern(Type::Function { ret: new_ret, params: new_params, varargs });

    // Phis first, so that a value may come round a loop.
    let mut phis: Vec<(InstId, InstId, InstId, u32)> = Vec::new();
    for &b in &f.layout.clone() {
        for k in 0..f.blocks[b.index()].insts.len() {
            let i = f.blocks[b.index()].insts[k];
            let Op::Phi { .. } = &f.insts[i.index()].op else { continue };
            let r = f.insts[i.index()].result.unwrap();
            let Some(bits) = wide_bits(cx, f.locals[r.index()].ty) else { continue };
            let lo = f.add_inst(Op::Phi { fmf: FastMath::default(), incoming: vec![] }, Some((None, ty.i64)));
            let hi = f.add_inst(Op::Phi { fmf: FastMath::default(), incoming: vec![] }, Some((None, ty.i64)));
            parts.insert(r, (f.result(lo), f.result(hi)));
            bits_of.insert(r, bits);
            phis.push((i, lo, hi, bits));
        }
    }

    // In reverse postorder, so that each value is split before its uses
    // (the layout need not have definitions first); unreachable blocks
    // after.
    let mut blocks = reverse_postorder(f);
    for &b in &f.layout {
        if !blocks.contains(&b) {
            blocks.push(b);
        }
    }
    for b in blocks {
        let old = std::mem::take(&mut f.blocks[b.index()].insts);
        let mut out: Vec<InstId> = if b == entry { std::mem::take(&mut entry_new) } else { Vec::new() };
        for i in old {
            // A wide phi: its halves' phis instead.
            if let Some(&(_, lo, hi, _)) = phis.iter().find(|p| p.0 == i) {
                // Phis stay at the top of the block.
                let at = out.iter().take_while(|&&x| matches!(f.insts[x.index()].op, Op::Phi { .. })).count();
                out.splice(at..at, [lo, hi]);
                continue;
            }
            let mut s = B { f, new: Vec::new() };
            let keep = rewrite(cx, &mut s, &ty, &mut parts, &mut bits_of, i, ret_bits.is_some())?;
            let new = s.new;
            out.extend(new);
            if keep {
                out.push(i);
            }
            out.extend(AFTER.with(|a| std::mem::take(&mut *a.borrow_mut())));
        }
        f.blocks[b.index()].insts = out;
    }
    // The phis' incoming halves.
    for (i, lo, hi, _) in phis {
        let Op::Phi { incoming, .. } = f.insts[i.index()].op.clone() else { unreachable!() };
        let mut ilo = Vec::new();
        let mut ihi = Vec::new();
        for (v, from) in incoming {
            let (l, h) = pair_of(cx, &ty, &parts, v)?;
            ilo.push((l, from));
            ihi.push((h, from));
        }
        f.insts[lo.index()].op = Op::Phi { fmf: FastMath::default(), incoming: ilo };
        f.insts[hi.index()].op = Op::Phi { fmf: FastMath::default(), incoming: ihi };
    }
    Ok(())
}

/// The halves of a wide value.
fn pair_of(cx: &mut Ctx, ty: &Types, parts: &HashMap<LocalId, Pair>, v: Value) -> Result<Pair, String> {
    match v {
        Value::Local(x) => parts.get(&x).copied().ok_or_else(|| "a wide value used before it is made".to_string()),
        Value::Const(c) => {
            let k = cx.m.constant(c).clone();
            let (lo, hi) = match &k.kind {
                ConstKind::Int(a) => {
                    let w = a.words();
                    let lo = w.first().copied().unwrap_or(0);
                    let hi = w.get(1).copied().unwrap_or(if a.is_negative() && w.len() < 2 { u64::MAX } else { 0 });
                    (lo, hi)
                }
                ConstKind::Zero | ConstKind::Undef | ConstKind::Poison => (0, 0),
                other => return Err(format!("a wide constant {other:?}")),
            };
            let mk = |cx: &mut Ctx, x: u64| {
                Value::Const(cx.m.consts.intern(Constant { ty: ty.i64, kind: ConstKind::Int(ApInt::from_u64(64, x)) }))
            };
            Ok((mk(cx, lo), mk(cx, hi)))
        }
        Value::Metadata(_) => Err("metadata as a wide value".into()),
    }
}

fn int(cx: &mut Ctx, ty: TypeId, v: i64) -> Value {
    cx.int(ty, v)
}

/// The high half with its bits above the type's width put right: zeroed,
/// or copies of its sign bit.
fn normal(cx: &mut Ctx, s: &mut B, ty: &Types, hi: Value, bits: u32, signed: bool) -> Value {
    if bits >= 128 {
        return hi;
    }
    let up = int(cx, ty.i64, (128 - bits) as i64);
    if signed {
        let t = s.bin(BinOp::Shl, hi, up, ty.i64);
        s.bin(BinOp::AShr, t, up, ty.i64)
    } else {
        let m = int(cx, ty.i64, ((1u64 << (bits - 64)) - 1) as i64);
        s.bin(BinOp::And, hi, m, ty.i64)
    }
}

/// The width of a wide operand.
fn width(cx: &Ctx, f: &Function, bits_of: &HashMap<LocalId, u32>, v: Value) -> Option<u32> {
    match v {
        Value::Local(x) => bits_of.get(&x).copied().or_else(|| wide_bits(cx, f.locals[x.index()].ty)),
        Value::Const(c) => wide_bits(cx, cx.m.constant(c).ty),
        Value::Metadata(_) => None,
    }
}

/// Rewrites instruction `i` if it touches a wide value; true if it is to
/// stay (its result is not wide, and the new instructions feed it).
fn rewrite(
    cx: &mut Ctx,
    s: &mut B,
    ty: &Types,
    parts: &mut HashMap<LocalId, Pair>,
    bits_of: &mut HashMap<LocalId, u32>,
    i: InstId,
    wide_ret: bool,
) -> Result<bool, String> {
    let op = s.f.insts[i.index()].op.clone();
    let result = s.f.insts[i.index()].result;
    // A half of a wide result, taken out already (after an invoke).
    if let (Op::ExtractValue { aggregate: Value::Local(a), .. }, Some(r)) = (&op, result)
        && parts.get(a).is_some_and(|&(lo, hi)| lo == Value::Local(r) || hi == Value::Local(r))
    {
        return Ok(true);
    }
    let res_bits = result.and_then(|r| wide_bits(cx, s.f.locals[r.index()].ty));
    let mut wide_operand = false;
    op.for_each_value(|v| wide_operand |= width(cx, s.f, bits_of, *v).is_some());
    if res_bits.is_none() && !wide_operand {
        return Ok(true);
    }
    let i64t = ty.i64;
    let k64 =
        Value::Const(cx.m.consts.intern(Constant { ty: ty.i128, kind: ConstKind::Int(ApInt::from_u64(128, 64)) }));
    let set = |parts: &mut HashMap<LocalId, Pair>, bits_of: &mut HashMap<LocalId, u32>, p: Pair| {
        if let (Some(r), Some(b)) = (result, res_bits) {
            parts.insert(r, p);
            bits_of.insert(r, b);
        }
    };
    let bits = res_bits.unwrap_or(128);
    match op {
        Op::Binary { op: bop, lhs, rhs, .. } if res_bits.is_some() => {
            let (al, ah) = pair_of(cx, ty, parts, lhs)?;
            let (bl, bh) = pair_of(cx, ty, parts, rhs)?;
            let p = match bop {
                BinOp::And | BinOp::Or | BinOp::Xor => (s.bin(bop, al, bl, i64t), s.bin(bop, ah, bh, i64t)),
                BinOp::Add => {
                    let l = s.bin(BinOp::Add, al, bl, i64t);
                    let c = s.cmp(ICmpPred::Ult, l, al, ty.i1);
                    let c = s.cast(CastOp::ZExt, c, i64t);
                    let h = s.bin(BinOp::Add, ah, bh, i64t);
                    (l, s.bin(BinOp::Add, h, c, i64t))
                }
                BinOp::Sub => {
                    let l = s.bin(BinOp::Sub, al, bl, i64t);
                    let c = s.cmp(ICmpPred::Ult, al, bl, ty.i1);
                    let c = s.cast(CastOp::ZExt, c, i64t);
                    let h = s.bin(BinOp::Sub, ah, bh, i64t);
                    (l, s.bin(BinOp::Sub, h, c, i64t))
                }
                BinOp::Mul => {
                    let l = s.bin(BinOp::Mul, al, bl, i64t);
                    let za = s.cast(CastOp::ZExt, al, ty.i128);
                    let zb = s.cast(CastOp::ZExt, bl, ty.i128);
                    let p = s.bin(BinOp::Mul, za, zb, ty.i128);
                    let ph = s.bin(BinOp::LShr, p, k64, ty.i128);
                    let carry = s.cast(CastOp::Trunc, ph, i64t);
                    let x = s.bin(BinOp::Mul, al, bh, i64t);
                    let y = s.bin(BinOp::Mul, ah, bl, i64t);
                    let h = s.bin(BinOp::Add, carry, x, i64t);
                    (l, s.bin(BinOp::Add, h, y, i64t))
                }
                BinOp::Shl | BinOp::LShr | BinOp::AShr => {
                    let signed = bop == BinOp::AShr;
                    let ah = if bop == BinOp::Shl { ah } else { normal(cx, s, ty, ah, bits, signed) };
                    shift(cx, s, ty, bop, (al, ah), bl)
                }
                BinOp::UDiv | BinOp::SDiv | BinOp::URem | BinOp::SRem => {
                    let signed = matches!(bop, BinOp::SDiv | BinOp::SRem);
                    let ah = normal(cx, s, ty, ah, bits, signed);
                    let bh = normal(cx, s, ty, bh, bits, signed);
                    let name = match bop {
                        BinOp::UDiv => "__udivti3",
                        BinOp::SDiv => "__divti3",
                        BinOp::URem => "__umodti3",
                        _ => "__modti3",
                    };
                    runtime(cx, s, ty, name, &[(al, ah), (bl, bh)])
                }
                other => return Err(format!("wide integers: {other:?} is not supported yet")),
            };
            set(parts, bits_of, p);
            Ok(false)
        }
        Op::ICmp { pred, lhs, rhs, .. } => {
            let bits = width(cx, s.f, bits_of, lhs).unwrap_or(128);
            let (al, ah) = pair_of(cx, ty, parts, lhs)?;
            let (bl, bh) = pair_of(cx, ty, parts, rhs)?;
            let signed = matches!(pred, ICmpPred::Slt | ICmpPred::Sle | ICmpPred::Sgt | ICmpPred::Sge);
            let ah = normal(cx, s, ty, ah, bits, signed);
            let bh = normal(cx, s, ty, bh, bits, signed);
            let new = match pred {
                ICmpPred::Eq | ICmpPred::Ne => {
                    let x = s.bin(BinOp::Xor, al, bl, i64t);
                    let y = s.bin(BinOp::Xor, ah, bh, i64t);
                    let z = s.bin(BinOp::Or, x, y, i64t);
                    let zero = int(cx, i64t, 0);
                    Op::ICmp { pred, samesign: false, lhs: z, rhs: zero }
                }
                _ => {
                    // The high halves decide, unless they are equal; then
                    // the low halves, unsigned.
                    let low = match pred {
                        ICmpPred::Slt | ICmpPred::Ult => ICmpPred::Ult,
                        ICmpPred::Sle | ICmpPred::Ule => ICmpPred::Ule,
                        ICmpPred::Sgt | ICmpPred::Ugt => ICmpPred::Ugt,
                        _ => ICmpPred::Uge,
                    };
                    let strict = match pred {
                        ICmpPred::Sle => ICmpPred::Slt,
                        ICmpPred::Sge => ICmpPred::Sgt,
                        ICmpPred::Ule => ICmpPred::Ult,
                        ICmpPred::Uge => ICmpPred::Ugt,
                        p => p,
                    };
                    let eq = s.cmp(ICmpPred::Eq, ah, bh, ty.i1);
                    let l = s.cmp(low, al, bl, ty.i1);
                    let h = s.cmp(strict, ah, bh, ty.i1);
                    Op::Select { fmf: FastMath::default(), cond: eq, then_value: l, else_value: h }
                }
            };
            s.f.insts[i.index()].op = new;
            Ok(true)
        }
        Op::Select { cond, then_value, else_value, .. } if res_bits.is_some() => {
            let (al, ah) = pair_of(cx, ty, parts, then_value)?;
            let (bl, bh) = pair_of(cx, ty, parts, else_value)?;
            let p = (s.select(cond, al, bl, i64t), s.select(cond, ah, bh, i64t));
            set(parts, bits_of, p);
            Ok(false)
        }
        Op::Freeze(v) if res_bits.is_some() => {
            let p = pair_of(cx, ty, parts, v)?;
            set(parts, bits_of, p);
            Ok(false)
        }
        Op::Cast { op: cop, value, .. } => {
            let from = width(cx, s.f, bits_of, value);
            match (cop, from, res_bits) {
                // Narrow to wide.
                (CastOp::ZExt | CastOp::SExt, None, Some(_)) => {
                    let vb = match cx.m.types.get(cx.value_ty(s.f, value)) {
                        Type::Int(b) => *b,
                        _ => 64,
                    };
                    let lo = if vb < 64 { s.cast(cop, value, i64t) } else { value };
                    let hi = if cop == CastOp::SExt {
                        let k = int(cx, i64t, 63);
                        s.bin(BinOp::AShr, lo, k, i64t)
                    } else {
                        int(cx, i64t, 0)
                    };
                    set(parts, bits_of, (lo, hi));
                    Ok(false)
                }
                // Wide to wide.
                (CastOp::ZExt | CastOp::SExt | CastOp::Trunc, Some(fb), Some(_)) => {
                    let (lo, hi) = pair_of(cx, ty, parts, value)?;
                    let hi = if cop == CastOp::Trunc { hi } else { normal(cx, s, ty, hi, fb, cop == CastOp::SExt) };
                    set(parts, bits_of, (lo, hi));
                    Ok(false)
                }
                // Wide to narrow: the low half.
                (CastOp::Trunc, Some(_), None) => {
                    let (lo, _) = pair_of(cx, ty, parts, value)?;
                    let r = result.unwrap();
                    let rt = s.f.locals[r.index()].ty;
                    s.f.insts[i.index()].op = if matches!(cx.m.types.get(rt), Type::Int(64)) {
                        Op::Freeze(lo)
                    } else {
                        Op::Cast { op: CastOp::Trunc, flags: IntFlags::NONE, fmf: FastMath::default(), value: lo }
                    };
                    Ok(true)
                }
                // To floating point: the runtime's (`__floattidf`).
                (CastOp::SIToFP | CastOp::UIToFP, Some(fb), None) => {
                    let signed = cop == CastOp::SIToFP;
                    let (lo, hi) = pair_of(cx, ty, parts, value)?;
                    let hi = normal(cx, s, ty, hi, fb, signed);
                    let r = result.unwrap();
                    let rt = s.f.locals[r.index()].ty;
                    let f = match cx.m.types.get(rt) {
                        Type::Double => "df",
                        Type::Float => "sf",
                        _ => return Err("wide integers: a conversion to this type is not supported yet".into()),
                    };
                    let name = format!("__float{}ti{f}", if signed { "" } else { "un" });
                    let v = runtime_to_scalar(cx, s, ty, &name, (lo, hi), rt);
                    s.f.insts[i.index()].op = Op::Freeze(v);
                    Ok(true)
                }
                // From floating point: the runtime's (`__fixdfti`).
                (CastOp::FPToSI | CastOp::FPToUI, None, Some(_)) => {
                    let ft = cx.value_ty(s.f, value);
                    let f = match cx.m.types.get(ft) {
                        Type::Double => "df",
                        Type::Float => "sf",
                        _ => return Err("wide integers: a conversion from this type is not supported yet".into()),
                    };
                    let name = format!("__fix{}{f}ti", if cop == CastOp::FPToSI { "" } else { "uns" });
                    let p = runtime_from_scalar(cx, s, ty, &name, value, ft);
                    set(parts, bits_of, p);
                    Ok(false)
                }
                (c, _, _) => Err(format!("wide integers: a {c:?} cast is not supported yet")),
            }
        }
        Op::Load { ptr, align, volatile, atomic } if res_bits.is_some() && atomic.is_none() => {
            let lo = s.op(Op::Load { volatile, atomic: None, ptr, align }, i64t);
            let i8t = cx.m.types.int(8);
            let eight = int(cx, i64t, 8);
            let p8 = s.op(
                Op::Gep { flags: GepFlags::default(), source_ty: i8t, ptr, indices: vec![eight].into() },
                cx.m.types.ptr(0),
            );
            // The high half is as many bytes as it takes, and no more (an
            // `i104`'s five: an `i40`, which `promote_odd` loads in pieces).
            let hb = (bits - 64).div_ceil(8) * 8;
            let hi = if hb == 64 {
                s.op(Op::Load { volatile, atomic: None, ptr: p8, align: align.map(|a| a.min(Align::of(8))) }, i64t)
            } else {
                let narrow = cx.m.types.int(hb);
                let v = s.op(Op::Load { volatile, atomic: None, ptr: p8, align: Some(Align::of(1)) }, narrow);
                s.cast(CastOp::ZExt, v, i64t)
            };
            set(parts, bits_of, (lo, hi));
            Ok(false)
        }
        Op::Store { value, ptr, align, volatile, atomic }
            if width(cx, s.f, bits_of, value).is_some() && atomic.is_none() =>
        {
            let b = width(cx, s.f, bits_of, value).unwrap();
            let (lo, hi) = pair_of(cx, ty, parts, value)?;
            let i8t = cx.m.types.int(8);
            let eight = int(cx, i64t, 8);
            let p8 = s.op(
                Op::Gep { flags: GepFlags::default(), source_ty: i8t, ptr, indices: vec![eight].into() },
                cx.m.types.ptr(0),
            );
            let st = s.f.add_inst(Op::Store { volatile, atomic: None, value: lo, ptr, align }, None);
            s.new.push(st);
            // As many bytes as the value takes, and no more (see the load);
            // bits past the value's in its last byte written as zeros, as
            // LLVM writes them.
            let hb = (b - 64).div_ceil(8) * 8;
            let hi = if b % 8 != 0 {
                let mask = int(cx, i64t, ((1u64 << (b - 64)) - 1) as i64);
                s.bin(BinOp::And, hi, mask, i64t)
            } else {
                hi
            };
            let hi = if hb == 64 {
                hi
            } else {
                let narrow = cx.m.types.int(hb);
                s.cast(CastOp::Trunc, hi, narrow)
            };
            let st =
                s.f.add_inst(Op::Store { volatile, atomic: None, value: hi, ptr: p8, align: Some(Align::of(1)) }, None);
            s.new.push(st);
            Ok(false)
        }
        Op::Ret(Some(v)) if wide_ret => {
            let agg = aggregate(cx, s, ty, parts, v)?;
            s.f.insts[i.index()].op = Op::Ret(Some(agg));
            Ok(true)
        }
        Op::Call(call)
            if crate::prepare::callee_name(cx.m, &call).is_some_and(|n| n.starts_with("llvm."))
                && (res_bits.is_some() || wide_operand) =>
        {
            let name = crate::prepare::callee_name(cx.m, &call).unwrap();
            let base: String = name.split('.').take(2).collect::<Vec<_>>().join(".");
            let args: Vec<Value> = call.args.iter().map(|a| a.value).collect();
            let p = intrinsic(cx, s, ty, &base, &args, parts, bits)
                .ok_or_else(|| format!("wide integers: {name} is not supported yet"))??;
            set(parts, bits_of, p);
            Ok(false)
        }
        Op::Invoke { mut call, normal, unwind } => {
            let wide_result = widen_call(cx, s, ty, parts, bits_of, &mut call, res_bits.is_some())?;
            s.f.insts[i.index()].op = Op::Invoke { call, normal, unwind };
            if let Some(r) = result.filter(|_| wide_result) {
                // Its halves, in the block of its normal edge (made above).
                s.f.locals[r.index()].ty = ty.pair;
                let agg = Value::Local(r);
                let lo = s.f.add_inst(Op::ExtractValue { aggregate: agg, indices: vec![0].into() }, Some((None, i64t)));
                let hi = s.f.add_inst(Op::ExtractValue { aggregate: agg, indices: vec![1].into() }, Some((None, i64t)));
                s.f.blocks[normal.index()].insts.splice(0..0, [lo, hi]);
                parts.insert(r, (s.f.result(lo), s.f.result(hi)));
                bits_of.insert(r, bits);
            }
            Ok(true)
        }
        Op::Call(mut call) => {
            // Wide arguments as [2 x i64]s; a wide result as one.
            let Type::Function { ret, params, varargs } = cx.m.types.get(call.fn_ty).clone() else { unreachable!() };
            let mut params = params;
            for (k, a) in call.args.iter_mut().enumerate() {
                if width(cx, s.f, bits_of, a.value).is_some() {
                    a.value = aggregate(cx, s, ty, parts, a.value)?;
                    a.attrs.push(Attr::Word(WIDE.into()));
                    if k < params.len() {
                        params[k] = ty.pair;
                    }
                }
            }
            let wide_result = res_bits.is_some();
            if wide_result {
                call.ret_attrs.push(Attr::Word(WIDE.into()));
            }
            call.fn_ty =
                cx.m.types.intern(Type::Function { ret: if wide_result { ty.pair } else { ret }, params, varargs });
            if let Some(r) = result.filter(|_| wide_result) {
                s.f.locals[r.index()].ty = ty.pair;
                s.f.insts[i.index()].op = Op::Call(call);
                // The call stays; its halves come after it.
                let agg = Value::Local(r);
                let lo = s.f.add_inst(Op::ExtractValue { aggregate: agg, indices: vec![0].into() }, Some((None, i64t)));
                let hi = s.f.add_inst(Op::ExtractValue { aggregate: agg, indices: vec![1].into() }, Some((None, i64t)));
                let (lo, hi) = (s.f.result(lo), s.f.result(hi));
                let (Value::Local(l), Value::Local(h)) = (lo, hi) else { unreachable!() };
                let (LocalDef::Inst(li), LocalDef::Inst(hi_i)) = (s.f.locals[l.index()].def, s.f.locals[h.index()].def)
                else {
                    unreachable!()
                };
                AFTER.with(|a| a.borrow_mut().extend([li, hi_i]));
                parts.insert(r, (lo, hi));
                bits_of.insert(r, bits);
                return Ok(true);
            }
            s.f.insts[i.index()].op = Op::Call(call);
            Ok(true)
        }
        other => Err(format!("wide integers: {} is not supported yet", op_name(&other))),
    }
}

/// The blocks reachable from the entry, in reverse postorder.
pub(crate) fn reverse_postorder(f: &Function) -> Vec<BlockId> {
    let mut seen = vec![false; f.blocks.len()];
    let mut post = Vec::new();
    let entry = f.layout[0];
    seen[entry.index()] = true;
    let mut stack = vec![(entry, f.successors(entry), 0usize)];
    while let Some((b, succs, next)) = stack.last_mut() {
        if let Some(&s) = succs.get(*next) {
            *next += 1;
            if !seen[s.index()] {
                seen[s.index()] = true;
                let ss = f.successors(s);
                stack.push((s, ss, 0));
            }
        } else {
            post.push(*b);
            stack.pop();
        }
    }
    post.reverse();
    post
}

/// A call's (or an invoke's) wide arguments as [2 x i64]s, and, if
/// `wide_result`, its result as one: whether it has one.
fn widen_call(
    cx: &mut Ctx,
    s: &mut B,
    ty: &Types,
    parts: &HashMap<LocalId, Pair>,
    bits_of: &HashMap<LocalId, u32>,
    call: &mut Call,
    wide_result: bool,
) -> Result<bool, String> {
    let Type::Function { ret, params, varargs } = cx.m.types.get(call.fn_ty).clone() else { unreachable!() };
    let mut params = params;
    for (k, a) in call.args.iter_mut().enumerate() {
        if width(cx, s.f, bits_of, a.value).is_some() {
            a.value = aggregate(cx, s, ty, parts, a.value)?;
            a.attrs.push(Attr::Word(WIDE.into()));
            if k < params.len() {
                params[k] = ty.pair;
            }
        }
    }
    if wide_result {
        call.ret_attrs.push(Attr::Word(WIDE.into()));
    }
    call.fn_ty = cx.m.types.intern(Type::Function { ret: if wide_result { ty.pair } else { ret }, params, varargs });
    Ok(wide_result)
}

thread_local! {
    /// Instructions to go after the one being rewritten (a call's result's
    /// halves).
    static AFTER: std::cell::RefCell<Vec<InstId>> = const { std::cell::RefCell::new(Vec::new()) };
}

fn op_name(op: &Op) -> String {
    let s = format!("{op:?}");
    s.split([' ', '(', '{']).next().unwrap_or("?").to_string()
}

/// A wide value as a [2 x i64].
fn aggregate(cx: &mut Ctx, s: &mut B, ty: &Types, parts: &HashMap<LocalId, Pair>, v: Value) -> Result<Value, String> {
    let (lo, hi) = pair_of(cx, ty, parts, v)?;
    let undef = Value::Const(cx.m.consts.intern(Constant { ty: ty.pair, kind: ConstKind::Undef }));
    let a = s.op(Op::InsertValue { aggregate: undef, element: lo, indices: vec![0].into() }, ty.pair);
    Ok(s.op(Op::InsertValue { aggregate: a, element: hi, indices: vec![1].into() }, ty.pair))
}

/// A call to a runtime function of two wide values, giving one.
fn runtime(cx: &mut Ctx, s: &mut B, ty: &Types, name: &str, args: &[Pair]) -> Pair {
    let params = vec![ty.pair; args.len()];
    let callee = crate::legalize::declare(cx, name, ty.pair, &params);
    let fn_ty = cx.m.types.intern(Type::Function { ret: ty.pair, params, varargs: false });
    let undef = Value::Const(cx.m.consts.intern(Constant { ty: ty.pair, kind: ConstKind::Undef }));
    let mut call_args = Vec::new();
    for &(lo, hi) in args {
        let a = s.op(Op::InsertValue { aggregate: undef, element: lo, indices: vec![0].into() }, ty.pair);
        let a = s.op(Op::InsertValue { aggregate: a, element: hi, indices: vec![1].into() }, ty.pair);
        call_args.push(CallArg { value: a, attrs: vec![Attr::Word(WIDE.into())] });
    }
    let r = s.op(
        Op::Call(Box::new(Call {
            tail: TailKind::None,
            fmf: FastMath::default(),
            cconv: CallConv::default(),
            ret_attrs: vec![Attr::Word(WIDE.into())],
            addrspace: None,
            fn_ty,
            callee,
            args: call_args,
            fn_attrs: vec![],
            bundles: vec![],
        })),
        ty.pair,
    );
    let lo = s.op(Op::ExtractValue { aggregate: r, indices: vec![0].into() }, ty.i64);
    let hi = s.op(Op::ExtractValue { aggregate: r, indices: vec![1].into() }, ty.i64);
    (lo, hi)
}

/// A runtime call of one wide argument returning a scalar of type `ret`.
fn runtime_to_scalar(cx: &mut Ctx, s: &mut B, ty: &Types, name: &str, (lo, hi): Pair, ret: TypeId) -> Value {
    let callee = crate::legalize::declare(cx, name, ret, &[ty.pair]);
    let fn_ty = cx.m.types.intern(Type::Function { ret, params: vec![ty.pair], varargs: false });
    let undef = Value::Const(cx.m.consts.intern(Constant { ty: ty.pair, kind: ConstKind::Undef }));
    let a = s.op(Op::InsertValue { aggregate: undef, element: lo, indices: vec![0].into() }, ty.pair);
    let a = s.op(Op::InsertValue { aggregate: a, element: hi, indices: vec![1].into() }, ty.pair);
    s.op(
        Op::Call(Box::new(Call {
            tail: TailKind::None,
            fmf: FastMath::default(),
            cconv: CallConv::default(),
            ret_attrs: vec![],
            addrspace: None,
            fn_ty,
            callee,
            args: vec![CallArg { value: a, attrs: vec![Attr::Word(WIDE.into())] }],
            fn_attrs: vec![],
            bundles: vec![],
        })),
        ret,
    )
}

/// A runtime call of one scalar argument (of type `arg_ty`) returning a
/// wide value, as its halves.
fn runtime_from_scalar(cx: &mut Ctx, s: &mut B, ty: &Types, name: &str, arg: Value, arg_ty: TypeId) -> Pair {
    let callee = crate::legalize::declare(cx, name, ty.pair, &[arg_ty]);
    let fn_ty = cx.m.types.intern(Type::Function { ret: ty.pair, params: vec![arg_ty], varargs: false });
    let r = s.op(
        Op::Call(Box::new(Call {
            tail: TailKind::None,
            fmf: FastMath::default(),
            cconv: CallConv::default(),
            ret_attrs: vec![Attr::Word(WIDE.into())],
            addrspace: None,
            fn_ty,
            callee,
            args: vec![CallArg { value: arg, attrs: vec![] }],
            fn_attrs: vec![],
            bundles: vec![],
        })),
        ty.pair,
    );
    let lo = s.op(Op::ExtractValue { aggregate: r, indices: vec![0].into() }, ty.i64);
    let hi = s.op(Op::ExtractValue { aggregate: r, indices: vec![1].into() }, ty.i64);
    (lo, hi)
}

/// A wide shift of (lo, hi) by `amt` (the amount's low half): by a
/// constant, the halves' shifts; by a variable, both cases, chosen by
/// whether the amount reaches the high half.
fn shift(cx: &mut Ctx, s: &mut B, ty: &Types, op: BinOp, (lo, hi): Pair, amt: Value) -> Pair {
    let i64t = ty.i64;
    let c = |cx: &mut Ctx, v: i64| int(cx, i64t, v);
    if let Some(k) = cx.const_int(amt) {
        let k = k & 127;
        return match (op, k) {
            (_, 0) => (lo, hi),
            (BinOp::Shl, 64) => (c(cx, 0), lo),
            (BinOp::Shl, k) if k >= 64 => {
                let x = c(cx, k - 64);
                (c(cx, 0), s.bin(BinOp::Shl, lo, x, i64t))
            }
            (BinOp::Shl, k) => {
                let (a, b) = (c(cx, k), c(cx, 64 - k));
                let h = s.bin(BinOp::Shl, hi, a, i64t);
                let carry = s.bin(BinOp::LShr, lo, b, i64t);
                (s.bin(BinOp::Shl, lo, a, i64t), s.bin(BinOp::Or, h, carry, i64t))
            }
            (op, k) if k >= 64 => {
                let x = c(cx, k - 64);
                let l = if k == 64 { hi } else { s.bin(op, hi, x, i64t) };
                let h = if op == BinOp::AShr {
                    let k63 = c(cx, 63);
                    s.bin(BinOp::AShr, hi, k63, i64t)
                } else {
                    c(cx, 0)
                };
                (l, h)
            }
            (op, k) => {
                let (a, b) = (c(cx, k), c(cx, 64 - k));
                let l = s.bin(BinOp::LShr, lo, a, i64t);
                let carry = s.bin(BinOp::Shl, hi, b, i64t);
                (s.bin(BinOp::Or, l, carry, i64t), s.bin(op, hi, a, i64t))
            }
        };
    }
    // amt in 0..128: n = amt & 63; past = amt >= 64. The bits that cross
    // from one half to the other are shifted in two steps, so that no
    // shift is by 64.
    let k63 = c(cx, 63);
    let k64 = c(cx, 64);
    let one = c(cx, 1);
    let n = s.bin(BinOp::And, amt, k63, i64t);
    let back = s.bin(BinOp::Xor, n, k63, i64t);
    let past = s.cmp(ICmpPred::Uge, amt, k64, ty.i1);
    let zero = c(cx, 0);
    if op == BinOp::Shl {
        let l = s.bin(BinOp::Shl, lo, n, i64t);
        let h = s.bin(BinOp::Shl, hi, n, i64t);
        let t = s.bin(BinOp::LShr, lo, one, i64t);
        let carry = s.bin(BinOp::LShr, t, back, i64t);
        let h = s.bin(BinOp::Or, h, carry, i64t);
        (s.select(past, zero, l, i64t), s.select(past, l, h, i64t))
    } else {
        let h = s.bin(op, hi, n, i64t);
        let l = s.bin(BinOp::LShr, lo, n, i64t);
        let t = s.bin(BinOp::Shl, hi, one, i64t);
        let carry = s.bin(BinOp::Shl, t, back, i64t);
        let l = s.bin(BinOp::Or, l, carry, i64t);
        let fill = if op == BinOp::AShr { s.bin(BinOp::AShr, hi, k63, i64t) } else { zero };
        (s.select(past, h, l, i64t), s.select(past, fill, h, i64t))
    }
}

/// An intrinsic on wide integers, on their halves; None if not one of
/// these.
fn intrinsic(
    cx: &mut Ctx,
    s: &mut B,
    ty: &Types,
    base: &str,
    args: &[Value],
    parts: &HashMap<LocalId, Pair>,
    bits: u32,
) -> Option<Result<Pair, String>> {
    let i64t = ty.i64;
    let get = |cx: &mut Ctx, k: usize| pair_of(cx, ty, parts, args[k]);
    let run = |cx: &mut Ctx, s: &mut B| -> Result<Pair, String> {
        Ok(match base {
            // fshl(a, b, c) = (a << c) | (b >> (w - c)), c mod w; fshr is
            // fshl by w - c. By a constant only.
            "llvm.fshl" | "llvm.fshr" => {
                let (a, b) = (get(cx, 0)?, get(cx, 1)?);
                let (cl, _) = get(cx, 2)?;
                let c = cx.const_int(cl).ok_or("a wide funnel shift by a variable amount")?.rem_euclid(bits as i64);
                let c = if base == "llvm.fshr" { (bits as i64 - c) % bits as i64 } else { c };
                if c == 0 {
                    return Ok(if base == "llvm.fshr" { b } else { a });
                }
                let b = (b.0, normal(cx, s, ty, b.1, bits, false));
                let k = int(cx, i64t, c);
                let hi = shift(cx, s, ty, BinOp::Shl, a, k);
                let k2 = int(cx, i64t, bits as i64 - c);
                let lo = shift(cx, s, ty, BinOp::LShr, b, k2);
                (s.bin(BinOp::Or, hi.0, lo.0, i64t), s.bin(BinOp::Or, hi.1, lo.1, i64t))
            }
            "llvm.umin" | "llvm.umax" | "llvm.smin" | "llvm.smax" => {
                let (a, b) = (get(cx, 0)?, get(cx, 1)?);
                let signed = base.starts_with("llvm.s");
                let (ah, bh) = (normal(cx, s, ty, a.1, bits, signed), normal(cx, s, ty, b.1, bits, signed));
                let lt = if signed { ICmpPred::Slt } else { ICmpPred::Ult };
                let eq = s.cmp(ICmpPred::Eq, ah, bh, ty.i1);
                let l = s.cmp(ICmpPred::Ult, a.0, b.0, ty.i1);
                let h = s.cmp(lt, ah, bh, ty.i1);
                let less = s.select(eq, l, h, ty.i1);
                let (x, y) = if base.ends_with("min") { (a, b) } else { (b, a) };
                (s.select(less, x.0, y.0, i64t), s.select(less, x.1, y.1, i64t))
            }
            "llvm.abs" => {
                let (lo, hi) = get(cx, 0)?;
                let hi = normal(cx, s, ty, hi, bits, true);
                let k63 = int(cx, i64t, 63);
                let sign = s.bin(BinOp::AShr, hi, k63, i64t);
                // (x ^ sign) - sign.
                let (xl, xh) = (s.bin(BinOp::Xor, lo, sign, i64t), s.bin(BinOp::Xor, hi, sign, i64t));
                let l = s.bin(BinOp::Sub, xl, sign, i64t);
                let c = s.cmp(ICmpPred::Ult, xl, sign, ty.i1);
                let c = s.cast(CastOp::ZExt, c, i64t);
                let h = s.bin(BinOp::Sub, xh, sign, i64t);
                (l, s.bin(BinOp::Sub, h, c, i64t))
            }
            // A byte swap: the halves' swapped, and exchanged; at fewer
            // than 128 bits, of the value zero-extended, then shifted down.
            "llvm.bswap" if bits % 8 == 0 => {
                let (lo, hi) = get(cx, 0)?;
                let hi = normal(cx, s, ty, hi, bits, false);
                let callee = crate::legalize::declare(cx, "llvm.bswap.i64", i64t, &[i64t]);
                let swap = |cx: &mut Ctx, s: &mut B, v: Value| {
                    let fn_ty = cx.m.types.intern(Type::Function { ret: i64t, params: vec![i64t], varargs: false });
                    s.op(
                        Op::Call(Box::new(Call {
                            tail: TailKind::None,
                            fmf: FastMath::default(),
                            cconv: CallConv::default(),
                            ret_attrs: vec![],
                            addrspace: None,
                            fn_ty,
                            callee,
                            args: vec![CallArg { value: v, attrs: vec![] }],
                            fn_attrs: vec![],
                            bundles: vec![],
                        })),
                        i64t,
                    )
                };
                let swapped = (swap(cx, s, hi), swap(cx, s, lo));
                if bits == 128 {
                    swapped
                } else {
                    let by = int(cx, i64t, i64::from(128 - bits));
                    shift(cx, s, ty, BinOp::LShr, swapped, by)
                }
            }
            "llvm.ctpop" | "llvm.ctlz" | "llvm.cttz" => {
                let (lo, hi) = get(cx, 0)?;
                let hi = normal(cx, s, ty, hi, bits, false);
                let un = |cx: &mut Ctx, s: &mut B, name: &str, v: Value| {
                    let i1 = ty.i1;
                    let sig: Vec<TypeId> = if name == "llvm.ctpop" { vec![i64t] } else { vec![i64t, i1] };
                    let callee = crate::legalize::declare(cx, &format!("{name}.i64"), i64t, &sig);
                    let mut a = vec![CallArg { value: v, attrs: vec![] }];
                    let mut params = vec![i64t];
                    if name != "llvm.ctpop" {
                        a.push(CallArg { value: int(cx, i1, 0), attrs: vec![] });
                        params.push(i1);
                    }
                    let fn_ty = cx.m.types.intern(Type::Function { ret: i64t, params, varargs: false });
                    s.op(
                        Op::Call(Box::new(Call {
                            tail: TailKind::None,
                            fmf: FastMath::default(),
                            cconv: CallConv::default(),
                            ret_attrs: vec![],
                            addrspace: None,
                            fn_ty,
                            callee,
                            args: a,
                            fn_attrs: vec![],
                            bundles: vec![],
                        })),
                        i64t,
                    )
                };
                let zero = int(cx, i64t, 0);
                let n = match base {
                    "llvm.ctpop" => {
                        let (a, b) = (un(cx, s, base, lo), un(cx, s, base, hi));
                        s.bin(BinOp::Add, a, b, i64t)
                    }
                    "llvm.ctlz" => {
                        // From the high half, unless it is zero; counted
                        // at 128 bits, less the bits above the type's.
                        let h = un(cx, s, base, hi);
                        let l = un(cx, s, base, lo);
                        let k64 = int(cx, i64t, 64);
                        let l = s.bin(BinOp::Add, l, k64, i64t);
                        let z = s.cmp(ICmpPred::Eq, hi, zero, ty.i1);
                        let n = s.select(z, l, h, i64t);
                        let extra = int(cx, i64t, 128 - bits as i64);
                        s.bin(BinOp::Sub, n, extra, i64t)
                    }
                    _ => {
                        let l = un(cx, s, base, lo);
                        let h = un(cx, s, base, hi);
                        let k64 = int(cx, i64t, 64);
                        let h = s.bin(BinOp::Add, h, k64, i64t);
                        let z = s.cmp(ICmpPred::Eq, lo, zero, ty.i1);
                        let n = s.select(z, h, l, i64t);
                        let w = int(cx, i64t, bits as i64);
                        let past = s.cmp(ICmpPred::Ugt, n, w, ty.i1);
                        s.select(past, w, n, i64t)
                    }
                };
                (n, zero)
            }
            _ => return Err(String::new()),
        })
    };
    match run(cx, s) {
        Err(e) if e.is_empty() => None,
        r => Some(r),
    }
}
