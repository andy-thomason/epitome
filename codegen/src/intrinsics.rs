//! Generic intrinsics no instruction is for, as other IR (before the rest
//! of `prepare`, which then lowers that):
//!
//! - The constrained floating-point ones (`llvm.experimental.constrained.*`)
//!   as the plain operations: code here runs in the default environment
//!   (rounding to nearest, no traps), where they are the same.
//! - What tells the optimiser things (`llvm.expect`, `llvm.annotation`,
//!   `llvm.launder.invariant.group`...) as the value it passes on, or
//!   nothing.
//! - `llvm.canonicalize` as `llvm.minnum` of the value with itself, which
//!   quiets a signalling NaN and changes nothing else.
//! - Integers' fixed-point products and quotients (`llvm.smul.fix`...),
//!   shifts that saturate, carry-less products (`llvm.clmul`), bit
//!   deposits and extractions (`llvm.pdep`, `llvm.pext`), by operations on
//!   integers of twice the width, or bit by bit.
//! - Conversions rounding as the environment does, or away from zero
//!   (`llvm.lrint`, `llvm.lround`...), as the rounding then the conversion.
//! - Element-wise intrinsics of vectors with no vector instruction for
//!   them, lane by lane.
//! - Vectors' lanes moved (`llvm.vector.reverse`, `.splice.left`,
//!   `.extract`, `.insert`, `.interleave2`...): shuffles with constant
//!   masks, or for interleavings of other than two, lane by lane;
//!   `llvm.stepvector` as its constant.
//! - Intrinsics of vectors of one lane, as of the lane.
//! - Integers' minimums and maximums of widths no instruction has (`i33`)
//!   as a comparison and a select; products that may overflow, of up to
//!   64 bits, in 128.
//! - `llvm.frexp` and `llvm.modf` as the C library's functions, the second
//!   result through a stack slot; `llvm.sincos` (and `sincospi`) as `sin`
//!   and `cos` (Apple's `__sinpi` and `__cospi`).
//! - Reductions of halves, lane by lane in halves (each step rounded as
//!   in half precision), and conversions of halves that saturate, from
//!   their exact single-precision values.
//! - Masked memory (`llvm.masked.load`, `.store`, `.gather`, `.scatter`,
//!   `.expandload`, `.compressstore`) lane by lane, each lane's access
//!   made to a stack slot where its lane is off: no lane off touches
//!   memory, and no branches are made. Compressions of vectors
//!   (`llvm.experimental.vector.compress`) through a stack slot so.
//! - Masked division (`llvm.masked.udiv`...) by 1 where lanes are off;
//!   the last lane on (`llvm.experimental.vector.extract.last.active`), as
//!   selects; `llvm.experimental.vector.match` as comparisons with each
//!   lane; `llvm.get.active.lane.mask` as a comparison with the lanes'
//!   numbers; reductions of booleans as their logic.
//! - Funnel shifts of widths no register is, by the amount modulo the
//!   width (not masked by a power of two).
//! - Comparisons of vectors of halves (`fcmp olt <8 x half>`) as of
//!   their single-precision values, which hold them exactly: one vector
//!   comparison, whose mask legalisation knows the lanes of.
//! - The floating-point environment (`llvm.get.rounding`,
//!   `llvm.set.fpmode`...) through AArch64's FPCR (`llvm.aarch64.get.fpcr`,
//!   where the description has it), or the C library's `fegetenv` and
//!   `fesetenv`; traps with a kind (`llvm.debugtrap`, `llvm.ubsantrap`) as
//!   AArch64's breakpoints; `llvm.ptrauth.blend` and `.resign` as
//!   arithmetic and the instructions for the others.
//! - `llvm.minnum` and `llvm.maxnum` of halves, which are done in single
//!   precision (`prepare::half_as_int`), where the conversion quiets a
//!   signalling NaN: NaN where either operand is one, as the half
//!   instructions give.
//! - `llvm.is.fpclass` of halves (in single precision a signalling NaN
//!   would be quiet) and of vectors, by their bits: comparisons of
//!   integers, lane by lane at once.
//! - Floating-point maximum and minimum reductions of a number of lanes
//!   other than a power of two, of the vector padded with what changes
//!   nothing (a quiet NaN for `fmax`, an infinity for `fmaximum`): as
//!   LLVM's back end does, so that the order of NEON's pairwise reductions,
//!   which decides which NaN comes out, is the same.

use epitome_ir::module::*;
use epitome_ir::types::{Type, TypeId};

use crate::prepare::{Ctx, Seq, call_intrinsic, callee_name};

/// What a rewrite made of a call: the value it is now (`Final`: made of
/// calls not to be rewritten again), or nothing (a call without a result,
/// removed).
enum Done {
    Value(Value),
    Final(Value),
    Removed,
}

pub fn lower(cx: &mut Ctx, f: &mut Function) {
    let blocks: Vec<BlockId> = f.layout.clone();
    // Stack slots the rewrites need, for the entry block's start.
    let mut slots = Vec::new();
    for b in blocks {
        let mut k = 0;
        while k < f.blocks[b.index()].insts.len() {
            let i = f.blocks[b.index()].insts[k];
            if let Op::FCmp { pred, fmf, lhs, rhs } = f.insts[i.index()].op
                && let Some(fty) = half_vector_as_float(cx, cx.value_ty(f, lhs))
            {
                let r = f.insts[i.index()].result.unwrap();
                let mty = f.locals[r.index()].ty;
                let mut s = Seq { f, new: Vec::new() };
                let a = s.cast(CastOp::FPExt, lhs, fty);
                let c = s.cast(CastOp::FPExt, rhs, fty);
                let v = s.op(Op::FCmp { pred, fmf, lhs: a, rhs: c }, mty);
                let new = std::mem::take(&mut s.new);
                let n = new.len();
                f.blocks[b.index()].insts.splice(k..=k, new);
                f.replace_uses(Value::Local(r), v);
                k += n;
                continue;
            }
            let Op::Call(call) = &f.insts[i.index()].op else {
                k += 1;
                continue;
            };
            let Some(name) = callee_name(cx.m, call).filter(|n| n.starts_with("llvm.")) else {
                k += 1;
                continue;
            };
            let args: Vec<Value> = call.args.iter().map(|a| a.value).collect();
            let result = f.insts[i.index()].result;
            let ty = result.map(|r| f.locals[r.index()].ty);
            let mut s = Seq { f, new: Vec::new() };
            let Some(done) = rewrite(cx, &mut s, &name, &args, ty, &mut slots) else {
                k += 1;
                continue;
            };
            let new = std::mem::take(&mut s.new);
            let n = new.len();
            f.blocks[b.index()].insts.splice(k..=k, new);
            match (done, result) {
                // The new instructions are looked at again: a rewrite may
                // make calls that are rewritten in their turn, lane by lane.
                (Done::Value(v), Some(r)) => f.replace_uses(Value::Local(r), v),
                (Done::Final(v), Some(r)) => {
                    f.replace_uses(Value::Local(r), v);
                    k += n;
                }
                _ => {}
            }
        }
    }
    if let Some(&entry) = f.layout.first() {
        f.blocks[entry.index()].insts.splice(0..0, slots);
    }
}

fn rewrite(
    cx: &mut Ctx,
    s: &mut Seq,
    name: &str,
    args: &[Value],
    ty: Option<TypeId>,
    slots: &mut Vec<InstId>,
) -> Option<Done> {
    if let Some(rest) = name.strip_prefix("llvm.experimental.constrained.") {
        return constrained(cx, s, rest, args, ty?).map(Done::Value);
    }
    let base = base_name(name);
    match base {
        "llvm.expect"
        | "llvm.expect.with.probability"
        | "llvm.annotation"
        | "llvm.ptr.annotation"
        | "llvm.launder.invariant.group"
        | "llvm.strip.invariant.group"
        | "llvm.arithmetic.fence" => {
            return Some(Done::Value(args[0]));
        }
        "llvm.donothing"
        | "llvm.sideeffect"
        | "llvm.licm.disable"
        | "llvm.fake.use"
        | "llvm.var.annotation"
        | "llvm.invariant.end" => return Some(Done::Removed),
        // Run-time checks are kept (as when no pass has chosen to drop
        // them).
        "llvm.allow.runtime.check" | "llvm.allow.ubsan.check" => {
            let i1 = cx.m.types.int(1);
            return Some(Done::Value(cx.int(i1, 1)));
        }
        _ => {}
    }
    // Those without a result.
    let void = cx.m.types.intern(Type::Void);
    match base {
        "llvm.masked.store" | "llvm.masked.scatter" | "llvm.masked.compressstore" => {
            return masked_memory(cx, s, base, args, void, slots);
        }
        "llvm.prefetch" => return Some(Done::Removed),
        // (When it does not loop.)
        "llvm.cond.loop" if cx.const_int(args[0]) == Some(0) => return Some(Done::Removed),
        "llvm.clear_cache" => {
            let _ = call_intrinsic(cx, s, "__clear_cache", vec![args[0], args[1]], void);
            return Some(Done::Removed);
        }
        // Breakpoints of Apple's kinds, as LLVM's back end makes them.
        "llvm.debugtrap" | "llvm.ubsantrap" if cx.selects_intrinsic("llvm.aarch64.break") => {
            let kind = if base == "llvm.debugtrap" { 0xf000 } else { 0x5500 | (cx.const_int(args[0])? & 0xff) };
            let i32t = cx.m.types.int(32);
            let k = cx.int(i32t, kind);
            let _ = call_intrinsic(cx, s, "llvm.aarch64.break", vec![k], void);
            return Some(Done::Removed);
        }
        "llvm.set.rounding" | "llvm.set.fpmode" | "llvm.reset.fpmode" if has_fpcr(cx) => {
            set_fpcr(cx, s, base, args.first().copied());
            return Some(Done::Removed);
        }
        "llvm.set.fpenv" | "llvm.reset.fpenv" => {
            let ptr = cx.m.types.ptr(0);
            let p = if base == "llvm.reset.fpenv" {
                // FE_DFL_ENV: Apple's is the address -1.
                let i64t = cx.m.types.int(64);
                let m1 = cx.int(i64t, -1);
                s.cast(CastOp::IntToPtr, m1, ptr)
            } else {
                let ety = cx.value_ty(s.f, args[0]);
                let p = slot(cx, s, ety, slots);
                s.op_void(Op::Store { volatile: false, atomic: None, value: args[0], ptr: p, align: None });
                p
            };
            let i32t = cx.m.types.int(32);
            let _ = call_intrinsic(cx, s, "fesetenv", vec![p], i32t);
            return Some(Done::Removed);
        }
        _ => {}
    }
    let ty = ty?;
    // Of one lane: of that lane (a reduction, the lane itself).
    let one_lane_arg = args.iter().any(|&a| !matches!(a, Value::Metadata(_)) && one_lane(cx, cx.value_ty(s.f, a)));
    if one_lane(cx, ty) || (!is_vector(cx, ty) && one_lane_arg) {
        return one_lane_call(cx, s, base, name, args, ty).map(Done::Value);
    }
    if is_vector(cx, ty) && lane_by_lane(base) && !cx.selects_intrinsic(name) {
        return Some(Done::Value(lanes(cx, s, name, args, ty)));
    }
    match base {
        "llvm.vector.reduce.fmax"
        | "llvm.vector.reduce.fmin"
        | "llvm.vector.reduce.fmaximum"
        | "llvm.vector.reduce.fminimum"
            if !half_elems(cx, ty) =>
        {
            pad_reduction(cx, s, base, args[0], ty).map(Done::Value)
        }
        "llvm.is.fpclass" if half_elems(cx, cx.value_ty(s.f, args[0])) || is_vector(cx, ty) => {
            let mask = cx.const_int(args[1])?;
            Some(Done::Value(fp_class(cx, s, args[0], mask as u32, ty)))
        }
        "llvm.minnum" | "llvm.maxnum" if half_elems(cx, ty) => Some(Done::Final(half_min_max(cx, s, name, args, ty))),
        "llvm.vector.reverse"
        | "llvm.vector.splice.left"
        | "llvm.vector.splice.right"
        | "llvm.vector.extract"
        | "llvm.vector.insert"
        | "llvm.stepvector" => moved_lanes(cx, s, base, args, ty).map(Done::Value),
        b if b.starts_with("llvm.vector.interleave") || b.starts_with("llvm.vector.deinterleave") => {
            let k: u32 = b
                .trim_start_matches("llvm.vector.")
                .trim_start_matches("de")
                .strip_prefix("interleave")?
                .parse()
                .ok()?;
            Some(Done::Value(if b.contains("deinterleave") {
                deinterleave(cx, s, args[0], k, ty)
            } else {
                interleave(cx, s, args, ty)
            }))
        }
        "llvm.smax" | "llvm.smin" | "llvm.umax" | "llvm.umin" if odd_width(cx, ty) => {
            let pred = match base {
                "llvm.smax" => ICmpPred::Sgt,
                "llvm.smin" => ICmpPred::Slt,
                "llvm.umax" => ICmpPred::Ugt,
                _ => ICmpPred::Ult,
            };
            let i1 = cx.m.types.int(1);
            let c = s.op(Op::ICmp { pred, samesign: false, lhs: args[0], rhs: args[1] }, i1);
            Some(Done::Value(
                s.op(Op::Select { cond: c, then_value: args[0], else_value: args[1], fmf: FastMath::default() }, ty),
            ))
        }
        "llvm.smul.with.overflow" | "llvm.umul.with.overflow" => {
            let Type::Struct { fields, .. } = cx.m.types.get(ty).clone() else { return None };
            if !odd_width(cx, fields[0]) || int_bits(cx, fields[0]) > 64 {
                return None;
            }
            Some(Done::Value(mul_overflow(cx, s, args[0], args[1], ty, fields[0], base.starts_with("llvm.s"))))
        }
        "llvm.frexp" | "llvm.modf" => libm_pair(cx, s, base, args[0], ty, slots).map(Done::Value),
        "llvm.sincos" | "llvm.sincospi" => {
            let Type::Struct { fields, .. } = cx.m.types.get(ty).clone() else { return None };
            let fty = fields[0];
            let float = match cx.m.types.get(fty) {
                Type::Float => true,
                Type::Double => false,
                _ => return None,
            };
            // Apple's: both at once (whose results are not always those of
            // sin and cos alone), a struct of the two.
            if darwin(cx) {
                let stret = match (base, float) {
                    ("llvm.sincos", true) => "__sincosf_stret",
                    ("llvm.sincos", false) => "__sincos_stret",
                    (_, true) => "__sincospif_stret",
                    (_, false) => "__sincospi_stret",
                };
                return Some(Done::Final(call_intrinsic(cx, s, stret, vec![args[0]], ty)));
            }
            let names = if base == "llvm.sincos" { ["sin", "cos"] } else { ["__sinpi", "__cospi"] };
            let mut agg = Value::Const(cx.m.consts.intern(Constant { ty, kind: ConstKind::Poison }));
            for (q, n) in names.into_iter().enumerate() {
                let n = if float { format!("{n}f") } else { n.to_string() };
                let v = call_intrinsic(cx, s, &n, vec![args[0]], fty);
                agg = s.op(Op::InsertValue { aggregate: agg, element: v, indices: vec![q as u32].into() }, ty);
            }
            Some(Done::Final(agg))
        }
        b if b.starts_with("llvm.vector.reduce.") && half_elems(cx, cx.value_ty(s.f, *args.last().unwrap())) => {
            half_reduction(cx, s, b, args, ty).map(Done::Value)
        }
        "llvm.powi" | "llvm.ldexp" if half_elems(cx, ty) && !is_vector(cx, ty) => {
            // In single precision (exact for ldexp, and the one rounding
            // to half last), as LLVM's back end does.
            let f32t = cx.m.types.intern(Type::Float);
            let x = s.cast(CastOp::FPExt, args[0], f32t);
            let ity = cx.value_ty(s.f, args[1]);
            let r = call_intrinsic(cx, s, &format!("{base}.f32.{}", mangle(cx, ity)), vec![x, args[1]], f32t);
            Some(Done::Value(s.cast(CastOp::FPTrunc, r, ty)))
        }
        "llvm.get.rounding" | "llvm.get.fpmode" if has_fpcr(cx) => {
            let i64t = cx.m.types.int(64);
            let fpcr = call_intrinsic(cx, s, "llvm.aarch64.get.fpcr", vec![], i64t);
            if base == "llvm.get.fpmode" {
                return Some(Done::Value(s.cast(CastOp::Trunc, fpcr, ty)));
            }
            // FPCR's RMode (bits 22 and 23: nearest, +inf, -inf, zero) as
            // LLVM's numbers (zero, nearest, +inf, -inf): one more, mod 4.
            let one = cx.int(i64t, 1 << 22);
            let v = s.bin(BinOp::Add, fpcr, one, i64t);
            let sh = cx.int(i64t, 22);
            let v = s.bin(BinOp::LShr, v, sh, i64t);
            let three = cx.int(i64t, 3);
            let v = s.bin(BinOp::And, v, three, i64t);
            Some(Done::Value(s.cast(CastOp::Trunc, v, ty)))
        }
        "llvm.get.fpenv" => {
            let p = slot(cx, s, ty, slots);
            let i32t = cx.m.types.int(32);
            let _ = call_intrinsic(cx, s, "fegetenv", vec![p], i32t);
            Some(Done::Value(s.op(Op::Load { ptr: p, align: None, volatile: false, atomic: None }, ty)))
        }
        "llvm.readcyclecounter" if darwin(cx) => {
            Some(Done::Value(call_intrinsic(cx, s, "llvm.readsteadycounter", vec![], ty)))
        }
        "llvm.ptrauth.blend" => {
            // The address's low 48 bits, the small discriminator's above.
            let low = cx.int(ty, (1i64 << 48) - 1);
            let a = s.bin(BinOp::And, args[0], low, ty);
            let sh = cx.int(ty, 48);
            let b = s.bin(BinOp::Shl, args[1], sh, ty);
            Some(Done::Value(s.bin(BinOp::Or, a, b, ty)))
        }
        "llvm.ptrauth.resign" => {
            let v = call_intrinsic(cx, s, "llvm.ptrauth.auth", vec![args[0], args[1], args[2]], ty);
            Some(Done::Value(call_intrinsic(cx, s, "llvm.ptrauth.sign", vec![v, args[3], args[4]], ty)))
        }
        "llvm.invariant.start" => {
            Some(Done::Value(Value::Const(cx.m.consts.intern(Constant { ty, kind: ConstKind::Null }))))
        }
        "llvm.stackaddress" => {
            Some(Done::Value(call_intrinsic(cx, s, &format!("llvm.stacksave.{}", mangle(cx, ty)), vec![], ty)))
        }
        "llvm.fptosi.sat" | "llvm.fptoui.sat" if half_elems(cx, cx.value_ty(s.f, args[0])) => {
            let hty = cx.value_ty(s.f, args[0]);
            let f32t = cx.m.types.intern(Type::Float);
            let fty = match *cx.m.types.get(hty) {
                Type::Vector { len, scalable, .. } => cx.m.types.intern(Type::Vector { len, elem: f32t, scalable }),
                _ => f32t,
            };
            let x = s.cast(CastOp::FPExt, args[0], fty);
            let n = format!("{base}.{}.{}", mangle(cx, ty), mangle(cx, fty));
            Some(Done::Value(call_intrinsic(cx, s, &n, vec![x], ty)))
        }
        "llvm.masked.load"
        | "llvm.masked.store"
        | "llvm.masked.gather"
        | "llvm.masked.scatter"
        | "llvm.masked.expandload"
        | "llvm.masked.compressstore" => masked_memory(cx, s, base, args, ty, slots),
        "llvm.experimental.vector.compress" => Some(Done::Value(compress(cx, s, args, ty, slots))),
        "llvm.masked.udiv" | "llvm.masked.urem" | "llvm.masked.sdiv" | "llvm.masked.srem" => {
            let one = splat_int(cx, ty, 1);
            let d =
                s.op(Op::Select { cond: args[2], then_value: args[1], else_value: one, fmf: FastMath::default() }, ty);
            let op = match base {
                "llvm.masked.udiv" => BinOp::UDiv,
                "llvm.masked.urem" => BinOp::URem,
                "llvm.masked.sdiv" => BinOp::SDiv,
                _ => BinOp::SRem,
            };
            Some(Done::Value(s.bin(op, args[0], d, ty)))
        }
        "llvm.experimental.vector.extract.last.active" => {
            let (n, elem) = lanes_of(cx, cx.value_ty(s.f, args[0]))?;
            let i64t = cx.m.types.int(64);
            let i1 = cx.m.types.int(1);
            let mut r = args[2];
            for k in 0..n {
                let idx = cx.int(i64t, k as i64);
                let on = s.op(Op::ExtractElement { vector: args[1], index: idx }, i1);
                let x = s.op(Op::ExtractElement { vector: args[0], index: idx }, elem);
                r = s.op(Op::Select { cond: on, then_value: x, else_value: r, fmf: FastMath::default() }, elem);
            }
            Some(Done::Value(r))
        }
        "llvm.experimental.vector.match" => {
            let aty = cx.value_ty(s.f, args[0]);
            let (n, elem) = lanes_of(cx, aty)?;
            let (m, _) = lanes_of(cx, cx.value_ty(s.f, args[1]))?;
            let i64t = cx.m.types.int(64);
            let mut any: Option<Value> = None;
            for k in 0..m {
                let idx = cx.int(i64t, k as i64);
                let x = s.op(Op::ExtractElement { vector: args[1], index: idx }, elem);
                let lanes: Vec<u32> = vec![0; n as usize];
                let one = s.op(
                    Op::InsertElement {
                        vector: Value::Const(cx.m.consts.intern(Constant { ty: aty, kind: ConstKind::Poison })),
                        element: x,
                        index: cx.int(i64t, 0),
                    },
                    aty,
                );
                let splat = shuffle(cx, s, one, one, &lanes, aty);
                let eq = s.op(Op::ICmp { pred: ICmpPred::Eq, samesign: false, lhs: args[0], rhs: splat }, ty);
                any = Some(match any {
                    Some(a) => s.bin(BinOp::Or, a, eq, ty),
                    None => eq,
                });
            }
            Some(Done::Value(s.bin(BinOp::And, any?, args[2], ty)))
        }
        "llvm.get.active.lane.mask" => {
            // Lane k is on where base + k < n: where n > base, and k is
            // below n - base.
            let (n, _) = lanes_of(cx, ty)?;
            let ity = cx.value_ty(s.f, args[0]);
            let vty = cx.m.types.intern(Type::Vector { len: n, elem: ity, scalable: false });
            let i1 = cx.m.types.int(1);
            let above = s.op(Op::ICmp { pred: ICmpPred::Ugt, samesign: false, lhs: args[1], rhs: args[0] }, i1);
            let left = s.bin(BinOp::Sub, args[1], args[0], ity);
            let i64t = cx.m.types.int(64);
            let poison = Value::Const(cx.m.consts.intern(Constant { ty: vty, kind: ConstKind::Poison }));
            let one = s.op(Op::InsertElement { vector: poison, element: left, index: cx.int(i64t, 0) }, vty);
            let splat = shuffle(cx, s, one, one, &vec![0; n as usize], vty);
            let ids: Vec<ConstId> = (0..n)
                .map(|k| {
                    let Value::Const(c) = cx.int(ity, k as i64) else { unreachable!() };
                    c
                })
                .collect();
            let step = Value::Const(cx.m.consts.intern(Constant { ty: vty, kind: ConstKind::Vector(ids) }));
            let below = s.op(Op::ICmp { pred: ICmpPred::Ult, samesign: false, lhs: step, rhs: splat }, ty);
            // (And with n > base splatted, a mask of one boolean.)
            let mpoison = Value::Const(cx.m.consts.intern(Constant { ty, kind: ConstKind::Poison }));
            let zero = cx.int(i64t, 0);
            let one_b = s.op(Op::InsertElement { vector: mpoison, element: above, index: zero }, ty);
            let above_all = shuffle(cx, s, one_b, one_b, &vec![0; n as usize], ty);
            Some(Done::Value(s.bin(BinOp::And, below, above_all, ty)))
        }
        "llvm.vector.reduce.add"
        | "llvm.vector.reduce.umax"
        | "llvm.vector.reduce.umin"
        | "llvm.vector.reduce.smax"
        | "llvm.vector.reduce.smin"
        | "llvm.vector.reduce.mul"
            if matches!(cx.m.types.get(ty), Type::Int(1)) =>
        {
            // Of booleans: add is xor; umax and smin are or (-1 is the
            // least signed); umin, smax and mul are and.
            let red = match base {
                "llvm.vector.reduce.add" => "xor",
                "llvm.vector.reduce.umax" | "llvm.vector.reduce.smin" => "or",
                _ => "and",
            };
            let vty = cx.value_ty(s.f, args[0]);
            Some(Done::Value(call_intrinsic(
                cx,
                s,
                &format!("llvm.vector.reduce.{red}.{}", mangle(cx, vty)),
                vec![args[0]],
                ty,
            )))
        }
        "llvm.vector.partial.reduce.fadd" => {
            // The accumulator, then each part of the input as wide as it,
            // added in order.
            let (n, _) = lanes_of(cx, ty)?;
            let (m, _) = lanes_of(cx, cx.value_ty(s.f, args[1]))?;
            let mut acc = args[0];
            for part in 0..m / n {
                let l: Vec<u32> = (part * n..(part + 1) * n).collect();
                let x = shuffle(cx, s, args[1], args[1], &l, ty);
                acc = s.bin(BinOp::FAdd, acc, x, ty);
            }
            Some(Done::Value(acc))
        }
        "llvm.fshl" | "llvm.fshr" if int_bits(cx, ty) == 128 && cx.const_int(args[2]).is_none() => {
            Some(Done::Value(odd_funnel(cx, s, args, ty, base == "llvm.fshl")))
        }
        "llvm.bitreverse" if int_bits(cx, ty) == 128 && matches!(cx.m.types.get(ty), Type::Int(_)) => {
            // Each half reversed, and the halves swapped.
            let i64t = cx.m.types.int(64);
            let lo = s.cast(CastOp::Trunc, args[0], i64t);
            let sh = cx.int(ty, 64);
            let hi = s.bin(BinOp::LShr, args[0], sh, ty);
            let hi = s.cast(CastOp::Trunc, hi, i64t);
            let rlo = call_intrinsic(cx, s, "llvm.bitreverse.i64", vec![lo], i64t);
            let rhi = call_intrinsic(cx, s, "llvm.bitreverse.i64", vec![hi], i64t);
            let a = s.cast(CastOp::ZExt, rlo, ty);
            let a = s.bin(BinOp::Shl, a, sh, ty);
            let b = s.cast(CastOp::ZExt, rhi, ty);
            Some(Done::Value(s.bin(BinOp::Or, a, b, ty)))
        }
        "llvm.fshl" | "llvm.fshr" if odd_width(cx, ty) => {
            Some(Done::Value(odd_funnel(cx, s, args, ty, base == "llvm.fshl")))
        }
        "llvm.get.dynamic.area.offset" => Some(Done::Value(cx.int(ty, 0))),
        "llvm.adjust.trampoline" => Some(Done::Value(args[0])),
        "llvm.load.relative" => {
            // ptr + its i32 at ptr + offset, sign-extended.
            let i8t = cx.m.types.int(8);
            let i32t = cx.m.types.int(32);
            let i64t = cx.m.types.int(64);
            let gep = |s: &mut Seq, p: Value, off: Value| {
                s.op(Op::Gep { flags: GepFlags(0), source_ty: i8t, ptr: p, indices: vec![off].into() }, ty)
            };
            let at = gep(s, args[0], args[1]);
            let rel = s.op(Op::Load { ptr: at, align: None, volatile: false, atomic: None }, i32t);
            let rel = s.cast(CastOp::SExt, rel, i64t);
            Some(Done::Value(gep(s, args[0], rel)))
        }
        "llvm.canonicalize" => {
            let suffix = &name["llvm.canonicalize.".len()..];
            Some(Done::Value(call_intrinsic(cx, s, &format!("llvm.minnum.{suffix}"), vec![args[0], args[0]], ty)))
        }
        "llvm.sshl.sat" | "llvm.ushl.sat" if !cx.selects_intrinsic(name) => {
            Some(Done::Value(shl_sat(cx, s, args[0], args[1], ty, base == "llvm.sshl.sat")))
        }
        "llvm.smul.fix" | "llvm.umul.fix" | "llvm.smul.fix.sat" | "llvm.umul.fix.sat" | "llvm.sdiv.fix"
        | "llvm.udiv.fix" | "llvm.sdiv.fix.sat" | "llvm.udiv.fix.sat" => {
            let scale = cx.const_int(args[2])? as u32;
            let signed = base.starts_with("llvm.s");
            let sat = base.ends_with(".sat");
            let v = if base.contains("mul") {
                mul_fix(cx, s, args[0], args[1], scale, ty, signed, sat)
            } else {
                div_fix(cx, s, args[0], args[1], scale, ty, signed, sat)
            };
            Some(Done::Value(v))
        }
        "llvm.clmul" if !cx.selects_intrinsic(name) => Some(Done::Value(clmul(cx, s, args[0], args[1], ty))),
        "llvm.pdep" | "llvm.pext" if !cx.selects_intrinsic(name) => {
            Some(Done::Value(deposit(cx, s, args[0], args[1], ty, base == "llvm.pdep")))
        }
        "llvm.lrint" | "llvm.llrint" | "llvm.lround" | "llvm.llround" if !cx.selects_intrinsic(name) => {
            let fty = cx.value_ty(s.f, args[0]);
            let round = if base.ends_with("rint") { "rint" } else { "round" };
            let r = call_intrinsic(cx, s, &format!("llvm.{round}.{}", mangle(cx, fty)), vec![args[0]], fty);
            Some(Done::Value(s.cast(CastOp::FPToSI, r, ty)))
        }
        _ => None,
    }
}

/// A stack slot of type `ty`, for the entry block.
fn slot(cx: &mut Ctx, s: &mut Seq, ty: TypeId, slots: &mut Vec<InstId>) -> Value {
    let ptr = cx.m.types.ptr(0);
    let i =
        s.f.add_inst(Op::Alloca { inalloca: false, ty, count: None, align: None, addrspace: None }, Some((None, ptr)));
    slots.push(i);
    s.f.result(i)
}

fn gep(cx: &mut Ctx, s: &mut Seq, elem: TypeId, p: Value, k: Value) -> Value {
    let ptr = cx.m.types.ptr(0);
    s.op(Op::Gep { flags: GepFlags(0), source_ty: elem, ptr: p, indices: vec![k].into() }, ptr)
}

/// Masked loads and stores, lane by lane: each lane's address that of its
/// element (consecutive, or its pointer for gathers and scatters, or the
/// next of those on for expansions and compressions), or a stack slot's
/// where the lane is off; loaded lanes that are off the pass-through's.
fn masked_memory(
    cx: &mut Ctx,
    s: &mut Seq,
    base: &str,
    args: &[Value],
    ty: TypeId,
    slots: &mut Vec<InstId>,
) -> Option<Done> {
    let store = matches!(base, "llvm.masked.store" | "llvm.masked.scatter" | "llvm.masked.compressstore");
    // (value, address(es), mask, pass-through)
    let (value, at, mask, pass) =
        if store { (Some(args[0]), args[1], args[2], None) } else { (None, args[0], args[1], Some(args[2])) };
    let vty = if store { cx.value_ty(s.f, args[0]) } else { ty };
    let (n, elem) = lanes_of(cx, vty)?;
    let i64t = cx.m.types.int(64);
    let i1 = cx.m.types.int(1);
    let ptr = cx.m.types.ptr(0);
    let dummy = slot(cx, s, elem, slots);
    let packed = matches!(base, "llvm.masked.expandload" | "llvm.masked.compressstore");
    let mut count = cx.int(i64t, 0);
    let mut out = pass;
    for k in 0..n {
        let idx = cx.int(i64t, k as i64);
        let on = s.op(Op::ExtractElement { vector: mask, index: idx }, i1);
        let addr = match base {
            "llvm.masked.gather" | "llvm.masked.scatter" => s.op(Op::ExtractElement { vector: at, index: idx }, ptr),
            _ if packed => gep(cx, s, elem, at, count),
            _ => gep(cx, s, elem, at, idx),
        };
        if packed {
            let one = s.cast(CastOp::ZExt, on, i64t);
            count = s.bin(BinOp::Add, count, one, i64t);
        }
        let a = s.op(Op::Select { cond: on, then_value: addr, else_value: dummy, fmf: FastMath::default() }, ptr);
        if let Some(v) = value {
            let x = s.op(Op::ExtractElement { vector: v, index: idx }, elem);
            s.op_void(Op::Store { volatile: false, atomic: None, value: x, ptr: a, align: Some(Align::of(1)) });
        } else {
            let x = s.op(Op::Load { ptr: a, align: Some(Align::of(1)), volatile: false, atomic: None }, elem);
            let p = s.op(Op::ExtractElement { vector: pass?, index: idx }, elem);
            let lane = s.op(Op::Select { cond: on, then_value: x, else_value: p, fmf: FastMath::default() }, elem);
            out = Some(s.op(Op::InsertElement { vector: out?, element: lane, index: idx }, ty));
        }
    }
    Some(match out {
        Some(v) if !store => Done::Value(v),
        _ => Done::Removed,
    })
}

/// The lanes of a vector that are on, in order, at the start; the rest
/// the pass-through's: through a stack slot holding the pass-through, each
/// lane stored at the next place (or to a slot of its own when off).
fn compress(cx: &mut Ctx, s: &mut Seq, args: &[Value], ty: TypeId, slots: &mut Vec<InstId>) -> Value {
    let (n, elem) = lanes_of(cx, ty).unwrap();
    let i64t = cx.m.types.int(64);
    let i1 = cx.m.types.int(1);
    let ptr = cx.m.types.ptr(0);
    let buf = slot(cx, s, ty, slots);
    let dummy = slot(cx, s, elem, slots);
    s.op_void(Op::Store { volatile: false, atomic: None, value: args[2], ptr: buf, align: Some(Align::of(1)) });
    let mut count = cx.int(i64t, 0);
    for k in 0..n {
        let idx = cx.int(i64t, k as i64);
        let on = s.op(Op::ExtractElement { vector: args[1], index: idx }, i1);
        let x = s.op(Op::ExtractElement { vector: args[0], index: idx }, elem);
        let place = gep(cx, s, elem, buf, count);
        let a = s.op(Op::Select { cond: on, then_value: place, else_value: dummy, fmf: FastMath::default() }, ptr);
        s.op_void(Op::Store { volatile: false, atomic: None, value: x, ptr: a, align: Some(Align::of(1)) });
        let one = s.cast(CastOp::ZExt, on, i64t);
        count = s.bin(BinOp::Add, count, one, i64t);
    }
    s.op(Op::Load { ptr: buf, align: Some(Align::of(1)), volatile: false, atomic: None }, ty)
}

/// A funnel shift of a width no register is (or of 128 bits, by a
/// variable amount), by the amount modulo the width: the two shifts, the
/// second done in two steps so that it is never by the whole width.
fn odd_funnel(cx: &mut Ctx, s: &mut Seq, args: &[Value], ty: TypeId, left: bool) -> Value {
    let w = int_bits(cx, ty) as i64;
    let wv = cx.int(ty, w);
    let k = s.bin(BinOp::URem, args[2], wv, ty);
    let one = cx.int(ty, 1);
    let top = cx.int(ty, w - 1);
    let rest = s.bin(BinOp::Sub, top, k, ty);
    let (first, second, op1, op2) =
        if left { (args[0], args[1], BinOp::Shl, BinOp::LShr) } else { (args[1], args[0], BinOp::LShr, BinOp::Shl) };
    let a = s.bin(op1, first, k, ty);
    let b = s.bin(op2, second, one, ty);
    let b = s.bin(op2, b, rest, ty);
    s.bin(BinOp::Or, a, b, ty)
}

/// Whether the description reads and writes AArch64's FPCR.
fn has_fpcr(cx: &Ctx) -> bool {
    cx.selects_intrinsic("llvm.aarch64.get.fpcr") && cx.selects_intrinsic("llvm.aarch64.set.fpcr")
}

/// FPCR written for `llvm.set.rounding` (its RMode from LLVM's number:
/// one less, mod 4), `llvm.set.fpmode` (all of it) or `llvm.reset.fpmode`
/// (the mode's fields cleared: rounding to nearest, no traps, no flushing).
fn set_fpcr(cx: &mut Ctx, s: &mut Seq, base: &str, arg: Option<Value>) {
    let i64t = cx.m.types.int(64);
    let void = cx.m.types.intern(Type::Void);
    let v = match base {
        "llvm.set.fpmode" => {
            let x = arg.unwrap();
            s.cast(CastOp::ZExt, x, i64t)
        }
        _ => {
            let fpcr = call_intrinsic(cx, s, "llvm.aarch64.get.fpcr", vec![], i64t);
            if base == "llvm.reset.fpmode" {
                // (The mode's bits: AHP, DN, FZ, RMode, Stride, FZ16, Len,
                // and the trap enables; those LLVM's back end clears.)
                let keep = cx.int(i64t, 0xffff_ffff_f800_40f8_u64 as i64);
                s.bin(BinOp::And, fpcr, keep, i64t)
            } else {
                let x = arg.unwrap();
                let x64 = s.cast(CastOp::ZExt, x, i64t);
                let three = cx.int(i64t, 3);
                let m1 = cx.int(i64t, 3);
                let r = s.bin(BinOp::Add, x64, m1, i64t);
                let r = s.bin(BinOp::And, r, three, i64t);
                let sh = cx.int(i64t, 22);
                let r = s.bin(BinOp::Shl, r, sh, i64t);
                let clear = cx.int(i64t, !(3i64 << 22));
                let f = s.bin(BinOp::And, fpcr, clear, i64t);
                s.bin(BinOp::Or, f, r, i64t)
            }
        }
    };
    let _ = call_intrinsic(cx, s, "llvm.aarch64.set.fpcr", vec![v], void);
}

/// Whether the module is for an Apple platform.
fn darwin(cx: &Ctx) -> bool {
    cx.m.triple.as_deref().is_some_and(|t| {
        let t = String::from_utf8_lossy(t);
        t.contains("apple") || t.contains("darwin") || t.contains("macos")
    })
}

/// Whether `ty` is a vector of one lane (or a struct of them).
fn one_lane(cx: &Ctx, ty: TypeId) -> bool {
    match cx.m.types.get(ty) {
        Type::Vector { len: 1, scalable: false, .. } => true,
        Type::Struct { fields, .. } => {
            fields.first().is_some_and(|&f| matches!(cx.m.types.get(f), Type::Vector { len: 1, scalable: false, .. }))
        }
        _ => false,
    }
}

/// An intrinsic of vectors of one lane: its scalar form of the lanes; a
/// reduction (whose result is a scalar), the lane (after the start value,
/// for those with one).
fn one_lane_call(cx: &mut Ctx, s: &mut Seq, base: &str, name: &str, args: &[Value], ty: TypeId) -> Option<Value> {
    // (A lane's place among one: where it is.)
    if base == "llvm.vector.reverse" {
        return Some(args[0]);
    }
    if !one_lane(cx, ty) {
        let rest = base.strip_prefix("llvm.vector.reduce.")?;
        let v = *args.last()?;
        let elem = lanes_of(cx, cx.value_ty(s.f, v))?.1;
        let i64t = cx.m.types.int(64);
        let zero = cx.int(i64t, 0);
        let x = s.op(Op::ExtractElement { vector: v, index: zero }, elem);
        return Some(match rest {
            "fadd" => s.bin(BinOp::FAdd, args[0], x, ty),
            "fmul" => s.bin(BinOp::FMul, args[0], x, ty),
            _ => x,
        });
    }
    Some(lanes(cx, s, name, args, ty))
}

/// Whether integer type `ty` is of a width no register is (not 8, 16, 32,
/// 64 or 128).
fn odd_width(cx: &Ctx, ty: TypeId) -> bool {
    matches!(cx.m.types.get(ty), Type::Int(b) if ![8, 16, 32, 64, 128].contains(b))
}

/// `{ a * b, overflowed }` of integers of type `ity` (of up to 64 bits): the
/// product of the operands extended to 128 bits, and whether its
/// truncation, extended back, is another.
fn mul_overflow(cx: &mut Ctx, s: &mut Seq, a: Value, b: Value, ty: TypeId, ity: TypeId, signed: bool) -> Value {
    let i128t = cx.m.types.int(128);
    let i1 = cx.m.types.int(1);
    let ext = if signed { CastOp::SExt } else { CastOp::ZExt };
    let (x, y) = (s.cast(ext, a, i128t), s.cast(ext, b, i128t));
    let p = s.bin(BinOp::Mul, x, y, i128t);
    let r = s.cast(CastOp::Trunc, p, ity);
    let back = s.cast(ext, r, i128t);
    let over = s.op(Op::ICmp { pred: ICmpPred::Ne, samesign: false, lhs: back, rhs: p }, i1);
    let undef = Value::Const(cx.m.consts.intern(Constant { ty, kind: ConstKind::Poison }));
    let agg = s.op(Op::InsertValue { aggregate: undef, element: r, indices: vec![0].into() }, ty);
    s.op(Op::InsertValue { aggregate: agg, element: over, indices: vec![1].into() }, ty)
}

/// `llvm.frexp` or `llvm.modf` of a float or a double: the C library's
/// function, its second result stored through a pointer to a stack slot
/// and loaded back. (Of halves, in single precision, which holds them
/// exactly.)
fn libm_pair(cx: &mut Ctx, s: &mut Seq, base: &str, x: Value, ty: TypeId, slots: &mut Vec<InstId>) -> Option<Value> {
    let Type::Struct { fields, .. } = cx.m.types.get(ty).clone() else { return None };
    let (fty, second) = (fields[0], fields[1]);
    let (x, cty, half) = match cx.m.types.get(fty) {
        Type::Float | Type::Double => (x, fty, false),
        Type::Half => {
            let f32t = cx.m.types.intern(Type::Float);
            (s.cast(CastOp::FPExt, x, f32t), f32t, true)
        }
        _ => return None,
    };
    let float = matches!(cx.m.types.get(cty), Type::Float);
    let c = &base["llvm.".len()..];
    let fname = if float { format!("{c}f") } else { c.to_string() };
    let sty = if base == "llvm.frexp" { second } else { cty };
    let ptr = cx.m.types.ptr(0);
    let slot = s.f.add_inst(
        Op::Alloca { inalloca: false, ty: sty, count: None, align: None, addrspace: None },
        Some((None, ptr)),
    );
    slots.push(slot);
    let p = s.f.result(slot);
    let r = call_intrinsic(cx, s, &fname, vec![x, p], cty);
    let other = s.op(Op::Load { ptr: p, align: None, volatile: false, atomic: None }, sty);
    let (r, other) = if half {
        let a = s.cast(CastOp::FPTrunc, r, fty);
        let b = if base == "llvm.modf" { s.cast(CastOp::FPTrunc, other, fty) } else { other };
        (a, b)
    } else {
        (r, other)
    };
    let undef = Value::Const(cx.m.consts.intern(Constant { ty, kind: ConstKind::Poison }));
    let agg = s.op(Op::InsertValue { aggregate: undef, element: r, indices: vec![0].into() }, ty);
    Some(s.op(Op::InsertValue { aggregate: agg, element: other, indices: vec![1].into() }, ty))
}

/// A reduction of halves: lane by lane, each step an operation on halves
/// (rounded as half precision is). The maximums and minimums in NEON's
/// order, pairs then pairs of those; sums and products in order, after the
/// start value.
fn half_reduction(cx: &mut Ctx, s: &mut Seq, base: &str, args: &[Value], ty: TypeId) -> Option<Value> {
    let v = *args.last()?;
    let (n, elem) = lanes_of(cx, cx.value_ty(s.f, v))?;
    let i64t = cx.m.types.int(64);
    let lane: Vec<Value> = (0..n)
        .map(|k| {
            let idx = cx.int(i64t, k as i64);
            s.op(Op::ExtractElement { vector: v, index: idx }, elem)
        })
        .collect();
    let rest = base.strip_prefix("llvm.vector.reduce.")?;
    let ordered = match rest {
        "fadd" => Some(BinOp::FAdd),
        "fmul" => Some(BinOp::FMul),
        _ => None,
    };
    if let Some(op) = ordered {
        let mut acc = args[0];
        for x in lane {
            acc = s.bin(op, acc, x, ty);
        }
        return Some(acc);
    }
    let intr = match rest {
        "fmax" => "maxnum",
        "fmin" => "minnum",
        "fmaximum" => "maximum",
        "fminimum" => "minimum",
        _ => return None,
    };
    let name = format!("llvm.{intr}.f16");
    let mut level = lane;
    while level.len() > 1 {
        let mut next = Vec::new();
        for pair in level.chunks(2) {
            next.push(if pair.len() == 2 { call_intrinsic(cx, s, &name, vec![pair[0], pair[1]], ty) } else { pair[0] });
        }
        level = next;
    }
    Some(level[0])
}

/// For a vector of halves, the vector of floats of its lanes.
fn half_vector_as_float(cx: &mut Ctx, ty: TypeId) -> Option<TypeId> {
    let Type::Vector { len, elem, scalable: false } = *cx.m.types.get(ty) else { return None };
    if !matches!(cx.m.types.get(elem), Type::Half) {
        return None;
    }
    let f32t = cx.m.types.intern(Type::Float);
    Some(cx.m.types.intern(Type::Vector { len, elem: f32t, scalable: false }))
}

/// Whether `ty` is `half` or a vector of them.
fn half_elems(cx: &Ctx, ty: TypeId) -> bool {
    let e = match cx.m.types.get(ty) {
        Type::Vector { elem, .. } => *elem,
        _ => ty,
    };
    matches!(cx.m.types.get(e), Type::Half)
}

/// A constant of integer type `ity` (or splatted, of a vector of them).
fn splat_int(cx: &mut Ctx, ity: TypeId, v: i64) -> Value {
    let elem = match cx.m.types.get(ity) {
        Type::Vector { elem, .. } => *elem,
        _ => ity,
    };
    let Value::Const(c) = cx.int(elem, v) else { unreachable!() };
    if elem == ity {
        Value::Const(c)
    } else {
        Value::Const(cx.m.consts.intern(Constant { ty: ity, kind: ConstKind::Splat(c) }))
    }
}

/// `llvm.is.fpclass` of halves, floats or doubles (or vectors of them),
/// `mask` choosing the classes (bit 0 a signalling NaN, 1 a quiet one, then
/// -infinity, -normal, -subnormal, -0, +0, +subnormal, +normal,
/// +infinity), from the bits: the magnitude against the exponent's limits,
/// the quiet bit, the sign.
fn fp_class(cx: &mut Ctx, s: &mut Seq, x: Value, mask: u32, bty: TypeId) -> Value {
    let vty = cx.value_ty(s.f, x);
    let elem = match cx.m.types.get(vty) {
        Type::Vector { elem, .. } => *elem,
        _ => vty,
    };
    let (w, mbits) = match cx.m.types.get(elem) {
        Type::Half => (16, 10),
        Type::Float => (32, 23),
        _ => (64, 52),
    };
    let iel = cx.m.types.int(w);
    let ity = match *cx.m.types.get(vty) {
        Type::Vector { len, scalable, .. } => cx.m.types.intern(Type::Vector { len, elem: iel, scalable }),
        _ => iel,
    };
    let bits = s.cast(CastOp::BitCast, x, ity);
    let c = |cx: &mut Ctx, v: i64| splat_int(cx, ity, v);
    let abs_mask = c(cx, ((1u64 << (w - 1)) - 1) as i64);
    let inf_bits = ((1u64 << (w - 1)) - 1) & !((1u64 << mbits) - 1);
    let (inf, min_normal, quiet, zero) =
        (c(cx, inf_bits as i64), c(cx, 1i64 << mbits), c(cx, 1i64 << (mbits - 1)), c(cx, 0));
    let cmp = |s: &mut Seq, pred, a, b| s.op(Op::ICmp { pred, samesign: false, lhs: a, rhs: b }, bty);
    let abs = s.bin(BinOp::And, bits, abs_mask, ity);
    let neg = cmp(s, ICmpPred::Slt, bits, zero);
    let pos = cmp(s, ICmpPred::Sge, bits, zero);
    let nan = cmp(s, ICmpPred::Ugt, abs, inf);
    let q = s.bin(BinOp::And, bits, quiet, ity);
    let is_q = cmp(s, ICmpPred::Ne, q, zero);
    let is_s = cmp(s, ICmpPred::Eq, q, zero);
    let is_inf = cmp(s, ICmpPred::Eq, abs, inf);
    let is_zero = cmp(s, ICmpPred::Eq, abs, zero);
    let below_normal = cmp(s, ICmpPred::Ult, abs, min_normal);
    let not_zero = cmp(s, ICmpPred::Ne, abs, zero);
    let sub = s.bin(BinOp::And, below_normal, not_zero, bty);
    let at_least_normal = cmp(s, ICmpPred::Uge, abs, min_normal);
    let below_inf = cmp(s, ICmpPred::Ult, abs, inf);
    let normal = s.bin(BinOp::And, at_least_normal, below_inf, bty);
    let classes = [
        (nan, Some(is_s), None),
        (nan, Some(is_q), None),
        (is_inf, None, Some(neg)),
        (normal, None, Some(neg)),
        (sub, None, Some(neg)),
        (is_zero, None, Some(neg)),
        (is_zero, None, Some(pos)),
        (sub, None, Some(pos)),
        (normal, None, Some(pos)),
        (is_inf, None, Some(pos)),
    ];
    let mut acc: Option<Value> = None;
    for (k, (base, extra, sign)) in classes.into_iter().enumerate() {
        if mask & (1 << k) == 0 {
            continue;
        }
        let mut v = base;
        for e in [extra, sign].into_iter().flatten() {
            v = s.bin(BinOp::And, v, e, bty);
        }
        acc = Some(match acc {
            Some(a) => s.bin(BinOp::Or, a, v, bty),
            None => v,
        });
    }
    match acc {
        Some(a) => a,
        None => {
            let i1 = cx.m.types.int(1);
            let Value::Const(f) = cx.int(i1, 0) else { unreachable!() };
            if bty == i1 {
                Value::Const(f)
            } else {
                Value::Const(cx.m.consts.intern(Constant { ty: bty, kind: ConstKind::Splat(f) }))
            }
        }
    }
}

/// `llvm.minnum` or `llvm.maxnum` of halves: where an operand is a
/// signalling NaN, it made quiet (the first that is, as the half
/// instructions give), else the operation.
fn half_min_max(cx: &mut Ctx, s: &mut Seq, name: &str, args: &[Value], ty: TypeId) -> Value {
    let bty = match *cx.m.types.get(ty) {
        Type::Vector { len, scalable, .. } => {
            let i1 = cx.m.types.int(1);
            cx.m.types.intern(Type::Vector { len, elem: i1, scalable })
        }
        _ => cx.m.types.int(1),
    };
    let i16t = cx.m.types.int(16);
    let ity = match *cx.m.types.get(ty) {
        Type::Vector { len, scalable, .. } => cx.m.types.intern(Type::Vector { len, elem: i16t, scalable }),
        _ => i16t,
    };
    let quiet_bit = splat_int(cx, ity, 0x200);
    let quiet = |s: &mut Seq, x: Value| {
        let b = s.cast(CastOp::BitCast, x, ity);
        let q = s.bin(BinOp::Or, b, quiet_bit, ity);
        s.cast(CastOp::BitCast, q, ty)
    };
    // (Converted to single precision, a signalling NaN would be quiet.)
    let a = fp_class(cx, s, args[0], 1, bty);
    let b = fp_class(cx, s, args[1], 1, bty);
    let r = call_intrinsic(cx, s, name, vec![args[0], args[1]], ty);
    let qb = quiet(s, args[1]);
    let r = s.op(Op::Select { cond: b, then_value: qb, else_value: r, fmf: FastMath::default() }, ty);
    let qa = quiet(s, args[0]);
    s.op(Op::Select { cond: a, then_value: qa, else_value: r, fmf: FastMath::default() }, ty)
}

/// A shuffle of `a` and `b` (of one type), lane `k` of the result `lanes[k]`
/// of the two (`b`'s after `a`'s), of type `ty`.
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

fn lanes_of(cx: &Ctx, ty: TypeId) -> Option<(u32, TypeId)> {
    match cx.m.types.get(ty) {
        Type::Vector { len, elem, scalable: false } => Some((*len, *elem)),
        _ => None,
    }
}

/// The intrinsics that move lanes, as shuffles.
fn moved_lanes(cx: &mut Ctx, s: &mut Seq, base: &str, args: &[Value], ty: TypeId) -> Option<Value> {
    let (n, elem) = lanes_of(cx, ty)?;
    match base {
        "llvm.stepvector" => {
            let ids: Vec<ConstId> = (0..n)
                .map(|k| {
                    let Value::Const(c) = cx.int(elem, k as i64) else { unreachable!() };
                    c
                })
                .collect();
            Some(Value::Const(cx.m.consts.intern(Constant { ty, kind: ConstKind::Vector(ids) })))
        }
        "llvm.vector.reverse" => {
            let lanes: Vec<u32> = (0..n).rev().collect();
            Some(shuffle(cx, s, args[0], args[0], &lanes, ty))
        }
        // The lanes of the two together from lane `k` on (left), or the
        // last `k` of the first then the second's (right).
        "llvm.vector.splice.left" | "llvm.vector.splice.right" => {
            let k = cx.const_int(args[2])? as u32;
            let start = if base.ends_with("left") { k } else { n - k };
            let lanes: Vec<u32> = (start..start + n).collect();
            Some(shuffle(cx, s, args[0], args[1], &lanes, ty))
        }
        "llvm.vector.extract" => {
            let at = cx.const_int(args[1])? as u32;
            let lanes: Vec<u32> = (at..at + n).collect();
            Some(shuffle(cx, s, args[0], args[0], &lanes, ty))
        }
        "llvm.vector.insert" => {
            let at = cx.const_int(args[2])? as u32;
            let (m, _) = lanes_of(cx, cx.value_ty(s.f, args[1]))?;
            // The part widened to the whole's lanes, then the lanes chosen.
            let widen: Vec<u32> = (0..n).map(|l| l.min(m - 1)).collect();
            let wide = shuffle(cx, s, args[1], args[1], &widen, ty);
            let lanes: Vec<u32> = (0..n).map(|l| if l >= at && l < at + m { n + l - at } else { l }).collect();
            Some(shuffle(cx, s, args[0], wide, &lanes, ty))
        }
        _ => None,
    }
}

/// Vectors interleaved: lane `j` of the `k`-th, lane `j * K + k` of the
/// result. Two by one shuffle; more lane by lane.
fn interleave(cx: &mut Ctx, s: &mut Seq, args: &[Value], ty: TypeId) -> Value {
    let (n, elem) = lanes_of(cx, cx.value_ty(s.f, args[0])).unwrap();
    let k = args.len() as u32;
    if k == 2 {
        let lanes: Vec<u32> = (0..2 * n).map(|l| (l % 2) * n + l / 2).collect();
        return shuffle(cx, s, args[0], args[1], &lanes, ty);
    }
    let i64t = cx.m.types.int(64);
    let mut r = Value::Const(cx.m.consts.intern(Constant { ty, kind: ConstKind::Poison }));
    for j in 0..n {
        for (q, &v) in args.iter().enumerate() {
            let from = cx.int(i64t, j as i64);
            let e = s.op(Op::ExtractElement { vector: v, index: from }, elem);
            let to = cx.int(i64t, (j * k + q as u32) as i64);
            r = s.op(Op::InsertElement { vector: r, element: e, index: to }, ty);
        }
    }
    r
}

/// A vector deinterleaved into `k` (a struct of them): lane `j * K + q` of
/// it, lane `j` of the `q`-th. Each by a shuffle.
fn deinterleave(cx: &mut Ctx, s: &mut Seq, v: Value, k: u32, ty: TypeId) -> Value {
    let Type::Struct { fields, .. } = cx.m.types.get(ty).clone() else { unreachable!() };
    let (n, _) = lanes_of(cx, fields[0]).unwrap();
    let mut agg = Value::Const(cx.m.consts.intern(Constant { ty, kind: ConstKind::Poison }));
    for q in 0..k {
        let lanes: Vec<u32> = (0..n).map(|j| j * k + q).collect();
        let part = shuffle(cx, s, v, v, &lanes, fields[q as usize]);
        agg = s.op(Op::InsertValue { aggregate: agg, element: part, indices: vec![q].into() }, ty);
    }
    agg
}

/// A maximum or minimum reduction of a vector of `len` lanes, not a
/// power of two, as of the vector padded to one with the reduction's
/// identity; of one wider than a register, as of the maximums (or
/// minimums) of its halves.
fn pad_reduction(cx: &mut Ctx, s: &mut Seq, base: &str, v: Value, ty: TypeId) -> Option<Value> {
    let vty = cx.value_ty(s.f, v);
    let Type::Vector { len, elem, scalable: false } = *cx.m.types.get(vty) else { return None };
    if len.is_power_of_two() {
        // Wider than a register: the halves' lanes taken pairwise first
        // (as LLVM's back end does), then the reduction of the result.
        let bits = cx.size_of(vty).ok()? * 8;
        if bits <= 128 || len < 2 {
            return None;
        }
        let half = len / 2;
        let hty = cx.m.types.intern(Type::Vector { len: half, elem, scalable: false });
        let lo = shuffle(cx, s, v, v, &(0..half).collect::<Vec<_>>(), hty);
        let hi = shuffle(cx, s, v, v, &(half..len).collect::<Vec<_>>(), hty);
        let op = match base {
            "llvm.vector.reduce.fmax" => "maxnum",
            "llvm.vector.reduce.fmin" => "minnum",
            "llvm.vector.reduce.fmaximum" => "maximum",
            _ => "minimum",
        };
        let m = call_intrinsic(cx, s, &format!("llvm.{op}.{}", mangle(cx, hty)), vec![lo, hi], hty);
        return Some(call_intrinsic(cx, s, &format!("{base}.{}", mangle(cx, hty)), vec![m], ty));
    }
    let (sign, exp, mbits) = match cx.m.types.get(elem) {
        Type::Float => (31, 0xffu128 << 23, 23),
        Type::Double => (63, 0x7ffu128 << 52, 52),
        _ => return None,
    };
    let bits = match base {
        // A quiet NaN (negative, as LLVM's back end has it).
        "llvm.vector.reduce.fmax" | "llvm.vector.reduce.fmin" => (1u128 << sign) | exp | (1u128 << (mbits - 1)),
        "llvm.vector.reduce.fmaximum" => (1u128 << sign) | exp,
        _ => exp,
    };
    let wide = len.next_power_of_two();
    let pad = Value::Const(cx.m.consts.intern(Constant { ty: elem, kind: ConstKind::Float(bits) }));
    // The lanes there are, then the pad in each one more.
    let i32t = cx.m.types.int(32);
    let lanes: Vec<ConstId> = (0..wide)
        .map(|k| {
            let Value::Const(c) = cx.int(i32t, k.min(len - 1) as i64) else { unreachable!() };
            c
        })
        .collect();
    let mty = cx.m.types.intern(Type::Vector { len: wide, elem: i32t, scalable: false });
    let mask = cx.m.consts.intern(Constant { ty: mty, kind: ConstKind::Vector(lanes) });
    let wty = cx.m.types.intern(Type::Vector { len: wide, elem, scalable: false });
    let mut padded = s.op(Op::ShuffleVector { a: v, b: v, mask }, wty);
    let i64t = cx.m.types.int(64);
    for k in len..wide {
        let index = cx.int(i64t, k as i64);
        padded = s.op(Op::InsertElement { vector: padded, element: pad, index }, wty);
    }
    Some(call_intrinsic(cx, s, &format!("{base}.{}", mangle(cx, wty)), vec![padded], ty))
}

/// An intrinsic's name without its manglings (`llvm.smul.fix` of
/// `llvm.smul.fix.v4i32`).
fn base_name(name: &str) -> &str {
    let mut end = name.len();
    for (at, part) in name.match_indices('.').map(|(k, _)| (k, &name[k + 1..])).collect::<Vec<_>>().into_iter().rev() {
        let part = part.split('.').next().unwrap_or("");
        let mangling =
            part.strip_prefix(['i', 'f', 'p']).is_some_and(|d| !d.is_empty() && d.bytes().all(|b| b.is_ascii_digit()))
                || part.strip_prefix('v').is_some_and(|d| d.starts_with(|c: char| c.is_ascii_digit()))
                || part == "bf16";
        if !mangling {
            break;
        }
        end = at;
    }
    &name[..end]
}

/// Those element-wise intrinsics whose vectors are done lane by lane when
/// no instruction is for them.
fn lane_by_lane(base: &str) -> bool {
    matches!(
        base,
        "llvm.sadd.with.overflow"
            | "llvm.uadd.with.overflow"
            | "llvm.ssub.with.overflow"
            | "llvm.usub.with.overflow"
            | "llvm.smul.with.overflow"
            | "llvm.umul.with.overflow"
            | "llvm.smul.fix"
            | "llvm.umul.fix"
            | "llvm.smul.fix.sat"
            | "llvm.umul.fix.sat"
            | "llvm.sdiv.fix"
            | "llvm.udiv.fix"
            | "llvm.sdiv.fix.sat"
            | "llvm.udiv.fix.sat"
            | "llvm.sshl.sat"
            | "llvm.ushl.sat"
            | "llvm.lrint"
            | "llvm.llrint"
            | "llvm.lround"
            | "llvm.llround"
            | "llvm.frexp"
            | "llvm.ldexp"
            | "llvm.powi"
            | "llvm.modf"
            | "llvm.sincos"
            | "llvm.sincospi"
            | "llvm.scmp"
            | "llvm.ucmp"
            | "llvm.clmul"
    )
}

fn is_vector(cx: &Ctx, ty: TypeId) -> bool {
    match cx.m.types.get(ty) {
        Type::Vector { .. } => true,
        Type::Struct { fields, .. } => {
            fields.first().is_some_and(|&f| matches!(cx.m.types.get(f), Type::Vector { .. }))
        }
        _ => false,
    }
}

/// LLVM's mangling of a type in an intrinsic's name.
pub(crate) fn mangle(cx: &Ctx, ty: TypeId) -> String {
    match cx.m.types.get(ty) {
        Type::Int(b) => format!("i{b}"),
        Type::Half => "f16".into(),
        Type::BFloat => "bf16".into(),
        Type::Float => "f32".into(),
        Type::Double => "f64".into(),
        Type::Fp128 => "f128".into(),
        Type::Ptr(a) => format!("p{a}"),
        Type::Vector { len, elem, .. } => format!("v{len}{}", mangle(cx, *elem)),
        _ => "x".into(),
    }
}

/// The lanes' type of a vector type (or of a struct of vectors, each
/// field's), and how many.
fn lane_types(cx: &mut Ctx, ty: TypeId) -> (u32, TypeId) {
    match cx.m.types.get(ty).clone() {
        Type::Vector { len, elem, .. } => (len, elem),
        Type::Struct { fields, packed } => {
            let mut len = 0;
            let mut elems = Vec::new();
            for fld in fields {
                let Type::Vector { len: n, elem, .. } = *cx.m.types.get(fld) else { unreachable!() };
                len = n;
                elems.push(elem);
            }
            (len, cx.m.types.intern(Type::Struct { fields: elems, packed }))
        }
        _ => (1, ty),
    }
}

/// Intrinsic `name` of vectors, one call of its scalar form a lane: each
/// vector argument's lane (others as they are), and the results put
/// together (each field of a struct result as a vector). Booleans are put
/// together as integers as wide as the arguments' lanes, then compared
/// with zero: a mask as from a comparison (whose lanes' width
/// legalisation knows).
fn lanes(cx: &mut Ctx, s: &mut Seq, name: &str, args: &[Value], ty: TypeId) -> Value {
    let (len, ety) = lane_types(cx, ty);
    let i64t = cx.m.types.int(64);
    // The scalar intrinsic: each vector type in the name as its lanes'.
    let scalar_name = {
        let mut parts: Vec<String> = name.split('.').map(str::to_string).collect();
        for p in parts.iter_mut().skip(1) {
            if let Some(rest) = p.strip_prefix('v')
                && let Some(at) = rest.find(|c: char| !c.is_ascii_digit())
                && at > 0
                && rest[..at].parse::<u32>() == Ok(len)
            {
                *p = rest[at..].to_string();
            }
        }
        parts.join(".")
    };
    let fields: Vec<TypeId> = match cx.m.types.get(ty).clone() {
        Type::Struct { fields, .. } => fields,
        _ => vec![ty],
    };
    let i1 = cx.m.types.int(1);
    let arg_bits = args
        .iter()
        .find_map(|&x| match *cx.m.types.get(cx.value_ty(s.f, x)) {
            Type::Vector { elem, .. } => Some(cx.size_of(elem).map_or(8, |b| (8 * b as u32).max(8))),
            _ => None,
        })
        .unwrap_or(8);
    // Each field as built: booleans' as integers.
    let built: Vec<TypeId> = fields
        .iter()
        .map(|&t| match *cx.m.types.get(t) {
            Type::Vector { len, elem, scalable } if elem == i1 => {
                let e = cx.m.types.int(arg_bits);
                cx.m.types.intern(Type::Vector { len, elem: e, scalable })
            }
            _ => t,
        })
        .collect();
    let mut outs: Vec<Value> =
        built.iter().map(|&t| Value::Const(cx.m.consts.intern(Constant { ty: t, kind: ConstKind::Poison }))).collect();
    for lane in 0..len {
        let idx = cx.int(i64t, lane as i64);
        let a: Vec<Value> = args
            .iter()
            .map(|&x| match *cx.m.types.get(cx.value_ty(s.f, x)) {
                Type::Vector { elem, .. } => s.op(Op::ExtractElement { vector: x, index: idx }, elem),
                _ => x,
            })
            .collect();
        let y = call_intrinsic(cx, s, &scalar_name, a, ety);
        for (n, &fty) in fields.iter().enumerate() {
            let Type::Vector { elem, .. } = *cx.m.types.get(fty) else { unreachable!() };
            let mut e = if matches!(cx.m.types.get(ty), Type::Struct { .. }) {
                s.op(Op::ExtractValue { aggregate: y, indices: vec![n as u32].into() }, elem)
            } else {
                y
            };
            let Type::Vector { elem: be, .. } = *cx.m.types.get(built[n]) else { unreachable!() };
            if be != elem {
                e = s.cast(CastOp::SExt, e, be);
            }
            outs[n] = s.op(Op::InsertElement { vector: outs[n], element: e, index: idx }, built[n]);
        }
    }
    for (n, &fty) in fields.iter().enumerate() {
        if built[n] != fty {
            let zero = Value::Const(cx.m.consts.intern(Constant { ty: built[n], kind: ConstKind::Zero }));
            outs[n] = s.op(Op::ICmp { pred: ICmpPred::Ne, samesign: false, lhs: outs[n], rhs: zero }, fty);
        }
    }
    if !matches!(cx.m.types.get(ty), Type::Struct { .. }) {
        return outs[0];
    }
    let mut agg = Value::Const(cx.m.consts.intern(Constant { ty, kind: ConstKind::Poison }));
    for (n, v) in outs.into_iter().enumerate() {
        agg = s.op(Op::InsertValue { aggregate: agg, element: v, indices: vec![n as u32].into() }, ty);
    }
    agg
}

fn int_bits(cx: &Ctx, ty: TypeId) -> u32 {
    match cx.m.types.get(ty) {
        Type::Int(b) => *b,
        _ => 64,
    }
}

/// The constrained operation `op` (`fadd.f32`) as the plain one: its
/// metadata arguments (rounding, exceptions) dropped.
fn constrained(cx: &mut Ctx, s: &mut Seq, op: &str, args: &[Value], ty: TypeId) -> Option<Value> {
    let vals: Vec<Value> = args.iter().copied().filter(|a| !matches!(a, Value::Metadata(_))).collect();
    let (base, suffix) = op.split_once('.').unwrap_or((op, ""));
    let bin = |op| Op::Binary { op, flags: IntFlags::NONE, fmf: FastMath::default(), lhs: vals[0], rhs: vals[1] };
    Some(match base {
        "fadd" => s.op(bin(BinOp::FAdd), ty),
        "fsub" => s.op(bin(BinOp::FSub), ty),
        "fmul" => s.op(bin(BinOp::FMul), ty),
        "fdiv" => s.op(bin(BinOp::FDiv), ty),
        "frem" => s.op(bin(BinOp::FRem), ty),
        "fptosi" => s.cast(CastOp::FPToSI, vals[0], ty),
        "fptoui" => s.cast(CastOp::FPToUI, vals[0], ty),
        "sitofp" => s.cast(CastOp::SIToFP, vals[0], ty),
        "uitofp" => s.cast(CastOp::UIToFP, vals[0], ty),
        "fptrunc" => s.cast(CastOp::FPTrunc, vals[0], ty),
        "fpext" => s.cast(CastOp::FPExt, vals[0], ty),
        "fcmp" | "fcmps" => {
            let Some(&Value::Metadata(md)) = args.get(2) else { return None };
            let Metadata::String(p) = &cx.m.metadata[md.index()].data else { return None };
            let p = String::from_utf8_lossy(p).into_owned();
            let pred = FCmpPred::ALL.iter().find(|(n, _)| *n == p)?.1;
            s.op(Op::FCmp { pred, fmf: FastMath::default(), lhs: vals[0], rhs: vals[1] }, ty)
        }
        // (powi's exponent is in the plain one's name.)
        "powi" => call_intrinsic(cx, s, &format!("llvm.powi.{suffix}.i32"), vals, ty),
        _ => call_intrinsic(cx, s, &format!("llvm.{base}.{suffix}"), vals, ty),
    })
}

/// `x << n` that saturates: where bits would be lost (the value shifted
/// back is another), the largest value of the sign (or unsigned).
fn shl_sat(cx: &mut Ctx, s: &mut Seq, x: Value, n: Value, ty: TypeId, signed: bool) -> Value {
    let bits = int_bits(cx, ty);
    let i1 = cx.m.types.int(1);
    let shifted = s.bin(BinOp::Shl, x, n, ty);
    let back = s.bin(if signed { BinOp::AShr } else { BinOp::LShr }, shifted, n, ty);
    let lost = s.op(Op::ICmp { pred: ICmpPred::Ne, samesign: false, lhs: back, rhs: x }, i1);
    let limit = if signed {
        let zero = cx.int(ty, 0);
        let neg = s.op(Op::ICmp { pred: ICmpPred::Slt, samesign: false, lhs: x, rhs: zero }, i1);
        let min = cx.int(ty, i64::MIN >> (64 - bits));
        let max = cx.int(ty, i64::MAX >> (64 - bits));
        s.op(Op::Select { cond: neg, then_value: min, else_value: max, fmf: FastMath::default() }, ty)
    } else {
        cx.int(ty, -1)
    };
    s.op(Op::Select { cond: lost, then_value: limit, else_value: shifted, fmf: FastMath::default() }, ty)
}

/// `v`, of an integer twice `ty`'s width, clamped to `ty`'s range (signed
/// or, `signed` false, unsigned) and truncated to it.
fn clamp_to(cx: &mut Ctx, s: &mut Seq, v: Value, wide: TypeId, ty: TypeId, signed: bool) -> Value {
    let bits = int_bits(cx, ty);
    let i1 = cx.m.types.int(1);
    let sel = |s: &mut Seq, pred, a: Value, b: Value| {
        let c = s.op(Op::ICmp { pred, samesign: false, lhs: a, rhs: b }, i1);
        s.op(Op::Select { cond: c, then_value: a, else_value: b, fmf: FastMath::default() }, wide)
    };
    let v = if signed {
        let max = wide_int(cx, wide, (1u128 << (bits - 1)) - 1);
        let min = wide_int(cx, wide, (u128::MAX << (bits - 1)) & mask(int_bits(cx, wide)));
        let v = sel(s, ICmpPred::Slt, v, max);
        sel(s, ICmpPred::Sgt, v, min)
    } else {
        let max = wide_int(cx, wide, mask(bits));
        sel(s, ICmpPred::Ult, v, max)
    };
    s.cast(CastOp::Trunc, v, ty)
}

/// The low `bits` bits all ones.
fn mask(bits: u32) -> u128 {
    if bits >= 128 { u128::MAX } else { (1u128 << bits) - 1 }
}

/// An integer constant of type `ty` (up to 128 bits) with bits `v`.
fn wide_int(cx: &mut Ctx, ty: TypeId, v: u128) -> Value {
    let bits = int_bits(cx, ty);
    let ap = epitome_ir::apint::ApInt::parse_hex(bits, &format!("{v:x}"), false).expect("a constant that fits");
    Value::Const(cx.m.consts.intern(Constant { ty, kind: ConstKind::Int(ap) }))
}

/// A fixed-point product with `scale` fraction bits: the product of the
/// operands extended to twice the width, shifted right by `scale`; and
/// saturated, or truncated.
#[allow(clippy::too_many_arguments)]
fn mul_fix(cx: &mut Ctx, s: &mut Seq, a: Value, b: Value, scale: u32, ty: TypeId, signed: bool, sat: bool) -> Value {
    let bits = int_bits(cx, ty);
    let wide = cx.m.types.int(2 * bits);
    let ext = if signed { CastOp::SExt } else { CastOp::ZExt };
    let (x, y) = (s.cast(ext, a, wide), s.cast(ext, b, wide));
    let p = s.bin(BinOp::Mul, x, y, wide);
    let sh = cx.int(wide, scale as i64);
    let q = s.bin(if signed { BinOp::AShr } else { BinOp::LShr }, p, sh, wide);
    if sat { clamp_to(cx, s, q, wide, ty, signed) } else { s.cast(CastOp::Trunc, q, ty) }
}

/// A fixed-point quotient with `scale` fraction bits: the dividend
/// extended to twice the width and shifted left by `scale`, divided by the
/// divisor extended (rounding toward negative infinity, signed); and
/// saturated, or truncated.
#[allow(clippy::too_many_arguments)]
fn div_fix(cx: &mut Ctx, s: &mut Seq, a: Value, b: Value, scale: u32, ty: TypeId, signed: bool, sat: bool) -> Value {
    let bits = int_bits(cx, ty);
    let wide = cx.m.types.int(2 * bits);
    let ext = if signed { CastOp::SExt } else { CastOp::ZExt };
    let (x, y) = (s.cast(ext, a, wide), s.cast(ext, b, wide));
    let sh = cx.int(wide, scale as i64);
    let x = s.bin(BinOp::Shl, x, sh, wide);
    let q = if signed {
        // Toward negative infinity: one less where the remainder is not
        // zero and the signs differ.
        let i1 = cx.m.types.int(1);
        let q = s.bin(BinOp::SDiv, x, y, wide);
        let r = s.bin(BinOp::SRem, x, y, wide);
        let zero = cx.int(wide, 0);
        let rem = s.op(Op::ICmp { pred: ICmpPred::Ne, samesign: false, lhs: r, rhs: zero }, i1);
        let xor = s.bin(BinOp::Xor, x, y, wide);
        let differ = s.op(Op::ICmp { pred: ICmpPred::Slt, samesign: false, lhs: xor, rhs: zero }, i1);
        let both = s.bin(BinOp::And, rem, differ, i1);
        let adj = s.cast(CastOp::ZExt, both, wide);
        s.bin(BinOp::Sub, q, adj, wide)
    } else {
        s.bin(BinOp::UDiv, x, y, wide)
    };
    if sat { clamp_to(cx, s, q, wide, ty, signed) } else { s.cast(CastOp::Trunc, q, ty) }
}

/// The carry-less product: `a << k` exclusive-ored in for each bit `k` of
/// `b` that is set.
fn clmul(cx: &mut Ctx, s: &mut Seq, a: Value, b: Value, ty: TypeId) -> Value {
    let bits = int_bits(cx, ty);
    let mut acc = cx.int(ty, 0);
    let one = cx.int(ty, 1);
    for k in 0..bits {
        let kk = cx.int(ty, k as i64);
        let bit = s.bin(BinOp::LShr, b, kk, ty);
        let bit = s.bin(BinOp::And, bit, one, ty);
        // All ones where the bit is set.
        let zero = cx.int(ty, 0);
        let mask = s.bin(BinOp::Sub, zero, bit, ty);
        let sh = s.bin(BinOp::Shl, a, kk, ty);
        let term = s.bin(BinOp::And, sh, mask, ty);
        acc = s.bin(BinOp::Xor, acc, term, ty);
    }
    acc
}

/// Bits deposited (`pdep`: the low bits of `x`, in order, at the places
/// `mask`'s set bits are) or extracted (`pext`: the bits of `x` where
/// `mask`'s are set, in order, at the bottom): a bit at a time, with the
/// count of the mask's bits so far.
fn deposit(cx: &mut Ctx, s: &mut Seq, x: Value, mask: Value, ty: TypeId, dep: bool) -> Value {
    let bits = int_bits(cx, ty);
    let one = cx.int(ty, 1);
    let mut acc = cx.int(ty, 0);
    let mut count = cx.int(ty, 0);
    for k in 0..bits {
        let kk = cx.int(ty, k as i64);
        let m = s.bin(BinOp::LShr, mask, kk, ty);
        let m = s.bin(BinOp::And, m, one, ty);
        let (from, to) = if dep { (count, kk) } else { (kk, count) };
        let b = s.bin(BinOp::LShr, x, from, ty);
        let b = s.bin(BinOp::And, b, m, ty);
        let b = s.bin(BinOp::Shl, b, to, ty);
        acc = s.bin(BinOp::Or, acc, b, ty);
        count = s.bin(BinOp::Add, count, m, ty);
    }
    acc
}
