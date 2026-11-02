//! `x86_fp80` (C's `long double` on x86) on the x87 unit, in selection
//! form.
//!
//! The x87 registers are a stack, which the register allocator does not
//! model; so each `x86_fp80` value lives in a 16-byte stack slot of its
//! own, and each operation on such values loads its operands onto the x87
//! stack, works on them there, and stores the result to the result's slot,
//! leaving the stack empty. The x87 instructions are intrinsics
//! (`llvm.epitome.x87.fldt(ptr)`, `llvm.epitome.x87.fadd()`, ...) that the
//! description has one instruction each for, called in order; comparisons
//! are recipes (`llvm.epitome.x87.cmp.ogt(ptr, ptr)`: both loaded,
//! compared, one popped, the other popped, the condition set in a
//! register).
//!
//! - Loads and stores of `x86_fp80` copy its 10 bytes (an `i64` and an
//!   `i16`); constants are stored to their slots in the entry block.
//! - A phi's incoming values are copied to a slot of the phi's at the end
//!   of each predecessor, and from it to the phi's own slot after the
//!   phis: so phis that read each other read the values from before.
//! - Conversions to integers set the rounding to truncation for the store
//!   (`fnstcw`, `fldcw`), as C's conversions truncate; unsigned 64-bit
//!   ones go through `2^63`. Conversions from unsigned 64-bit integers add
//!   `2^64` to those the signed load reads as negative.
//! - Parameters, results and calls' arguments of `x86_fp80` (passed in
//!   memory, returned in `st(0)`) are not supported yet.

use std::collections::HashMap;

use epitome_ir::module::*;
use epitome_ir::types::{Type, TypeId};

use crate::prepare::{Ctx, Seq, call_intrinsic, callee_name};

fn is_fp80(cx: &Ctx, ty: TypeId) -> bool {
    matches!(cx.m.types.get(ty), Type::X86Fp80)
}

/// The bits of `2^e` as an `x86_fp80`.
fn pow2(e: u32) -> u128 {
    (u128::from(0x3fff + e) << 64) | (1u128 << 63)
}

struct Lower<'a, 'c> {
    cx: &'a mut Ctx<'c>,
    /// Each `x86_fp80` value's slot.
    slots: HashMap<LocalId, Value>,
    /// Constants' slots, filled in the entry block.
    consts: HashMap<u128, Value>,
    /// What goes at the start of the entry block: the slots, and the
    /// constants stored in them.
    prologue: Vec<InstId>,
}

impl Lower<'_, '_> {
    fn ptr(&mut self) -> TypeId {
        self.cx.m.types.ptr(0)
    }

    /// A new slot of `size` bytes, in the entry block.
    fn slot(&mut self, f: &mut Function, size: u64) -> Value {
        let i8t = self.cx.m.types.int(8);
        let i64t = self.cx.m.types.int(64);
        let n = self.cx.int(i64t, size as i64);
        let ptr = self.ptr();
        let op = Op::Alloca { inalloca: false, ty: i8t, count: Some(n), align: Some(Align::of(16)), addrspace: None };
        let i = f.add_inst(op, Some((None, ptr)));
        self.prologue.push(i);
        f.result(i)
    }

    /// The slot holding the constant whose bits are `bits`.
    fn constant(&mut self, f: &mut Function, bits: u128) -> Value {
        if let Some(&v) = self.consts.get(&bits) {
            return v;
        }
        let p = self.slot(f, 16);
        let (i64t, i16t) = (self.cx.m.types.int(64), self.cx.m.types.int(16));
        let lo = self.cx.int(i64t, bits as u64 as i64);
        let hi = self.cx.int(i16t, (bits >> 64) as u16 as i16 as i64);
        let mut s = Seq { f, new: Vec::new() };
        let ptr = self.ptr();
        let at8 = gep(self.cx, &mut s, p, 8, ptr);
        s.op_void(store(lo, p));
        s.op_void(store(hi, at8));
        self.prologue.extend(s.new);
        self.consts.insert(bits, p);
        p
    }

    /// The slot holding `x86_fp80` value `v`.
    fn of(&mut self, f: &mut Function, v: Value) -> Result<Value, String> {
        match v {
            Value::Local(x) => self.slots.get(&x).copied().ok_or_else(|| "an x86_fp80 value without a slot".into()),
            Value::Const(c) => match self.cx.m.constant(c).kind.clone() {
                ConstKind::Float(bits) => Ok(self.constant(f, bits)),
                ConstKind::Zero | ConstKind::Undef | ConstKind::Poison => Ok(self.constant(f, 0)),
                k => Err(format!("x86_fp80 constant {k:?} is not supported yet")),
            },
            Value::Metadata(_) => Err("metadata as an x86_fp80 value".into()),
        }
    }
}

fn store(value: Value, ptr: Value) -> Op {
    Op::Store { volatile: false, atomic: None, value, ptr, align: None }
}

fn load(ptr: Value) -> Op {
    Op::Load { volatile: false, atomic: None, ptr, align: None }
}

fn gep(cx: &mut Ctx, s: &mut Seq, p: Value, off: i64, ptr: TypeId) -> Value {
    let i8t = cx.m.types.int(8);
    let i64t = cx.m.types.int(64);
    let k = cx.int(i64t, off);
    s.op(Op::Gep { flags: Default::default(), source_ty: i8t, ptr: p, indices: vec![k].into() }, ptr)
}

/// An x87 instruction: `llvm.epitome.x87.NAME(args)`.
fn x87(cx: &mut Ctx, s: &mut Seq, name: &str, args: Vec<Value>) {
    let void = cx.m.types.intern(Type::Void);
    let params: Vec<TypeId> = args.iter().map(|&a| cx.value_ty(s.f, a)).collect();
    let callee = crate::legalize::declare(cx, &format!("llvm.epitome.x87.{name}"), void, &params);
    let fn_ty = cx.m.types.intern(Type::Function { ret: void, params, varargs: false });
    s.op_void(Op::Call(Box::new(Call {
        tail: TailKind::None,
        fmf: FastMath::default(),
        cconv: CallConv::default(),
        ret_attrs: vec![],
        addrspace: None,
        fn_ty,
        callee,
        args: args.into_iter().map(|value| CallArg { value, attrs: vec![] }).collect(),
        fn_attrs: vec![],
        bundles: vec![],
    })));
}

/// Copies an `x86_fp80`'s 10 bytes from `src` to `dst`.
fn copy(cx: &mut Ctx, s: &mut Seq, dst: Value, src: Value) {
    let ptr = cx.m.types.ptr(0);
    let (i64t, i16t) = (cx.m.types.int(64), cx.m.types.int(16));
    let lo = s.op(load(src), i64t);
    let src8 = gep(cx, s, src, 8, ptr);
    let hi = s.op(load(src8), i16t);
    s.op_void(store(lo, dst));
    let dst8 = gep(cx, s, dst, 8, ptr);
    s.op_void(store(hi, dst8));
}

/// `pred(*a, *b)` as an `i1`.
fn compare(cx: &mut Ctx, s: &mut Seq, pred: FCmpPred, a: Value, b: Value) -> Value {
    use FCmpPred::*;
    let i1 = cx.m.types.int(1);
    let one = |cx: &mut Ctx, s: &mut Seq, p: &str, a: Value, b: Value| {
        call_intrinsic(cx, s, &format!("llvm.epitome.x87.cmp.{p}"), vec![a, b], i1)
    };
    match pred {
        Ogt => one(cx, s, "ogt", a, b),
        Oge => one(cx, s, "oge", a, b),
        Ult => one(cx, s, "ult", a, b),
        Ule => one(cx, s, "ule", a, b),
        Ueq => one(cx, s, "ueq", a, b),
        One => one(cx, s, "one", a, b),
        Uno => one(cx, s, "uno", a, b),
        Ord => one(cx, s, "ord", a, b),
        Olt => one(cx, s, "ogt", b, a),
        Ole => one(cx, s, "oge", b, a),
        Ugt => one(cx, s, "ult", b, a),
        Uge => one(cx, s, "ule", b, a),
        Oeq => {
            let (e, o) = (one(cx, s, "ueq", a, b), one(cx, s, "ord", a, b));
            s.bin(BinOp::And, e, o, i1)
        }
        Une => {
            let (n, u) = (one(cx, s, "one", a, b), one(cx, s, "uno", a, b));
            s.bin(BinOp::Or, n, u, i1)
        }
        True => cx.int(i1, 1),
        False => cx.int(i1, 0),
    }
}

/// `*a` converted to a signed 64-bit integer, truncating.
fn to_i64(l: &mut Lower, f: &mut Function, a: Value) -> Vec<InstId> {
    let cw = l.slot(f, 4);
    let t = l.slot(f, 8);
    let cx = &mut *l.cx;
    let ptr = cx.m.types.ptr(0);
    let (i64t, i16t) = (cx.m.types.int(64), cx.m.types.int(16));
    let mut s = Seq { f, new: Vec::new() };
    // The control word with rounding toward zero (bits 10 and 11 set).
    x87(cx, &mut s, "fnstcw", vec![cw]);
    let w = s.op(load(cw), i16t);
    let rz = cx.int(i16t, 0xc00);
    let w2 = s.bin(BinOp::Or, w, rz, i16t);
    let cw2 = gep(cx, &mut s, cw, 2, ptr);
    s.op_void(store(w2, cw2));
    x87(cx, &mut s, "fldcw", vec![cw2]);
    x87(cx, &mut s, "fldt", vec![a]);
    x87(cx, &mut s, "fistpll", vec![t]);
    x87(cx, &mut s, "fldcw", vec![cw]);
    s.op(load(t), i64t);
    s.new
}

/// Rewrites `f`'s `x86_fp80` values as described above, if it has any and
/// the description has the x87 instructions.
pub fn lower(cx: &mut Ctx, f: &mut Function) -> Result<(), String> {
    if !f.locals.iter().any(|l| is_fp80(cx, l.ty)) && !uses_fp80_constants(cx, f) {
        return Ok(());
    }
    if !cx.selects_intrinsic("llvm.epitome.x87.fldt") {
        return Ok(());
    }
    if let Type::Function { ret, params, .. } = cx.m.types.get(f.ty)
        && (is_fp80(cx, *ret) || params.iter().any(|&p| is_fp80(cx, p)))
    {
        return Err("parameters and results of x86_fp80 are not supported yet".into());
    }
    let mut l = Lower { cx, slots: HashMap::new(), consts: HashMap::new(), prologue: Vec::new() };
    // A slot for each value (and a second for each phi).
    let mut phi_in: HashMap<LocalId, Value> = HashMap::new();
    for i in f.insts_in_order().collect::<Vec<_>>() {
        let Some(r) = f.insts[i.index()].result else { continue };
        if !is_fp80(l.cx, f.locals[r.index()].ty) {
            continue;
        }
        let p = l.slot(f, 16);
        l.slots.insert(r, p);
        if matches!(f.insts[i.index()].op, Op::Phi { .. }) {
            let q = l.slot(f, 16);
            phi_in.insert(r, q);
        }
    }
    // Copies at the ends of predecessors, for phis.
    let mut at_end: HashMap<BlockId, Vec<InstId>> = HashMap::new();
    let layout = f.layout.clone();
    for &b in &layout {
        let insts = f.blocks[b.index()].insts.clone();
        let mut out = Vec::new();
        let mut after_phis = Vec::new();
        for i in insts {
            let new = lower_one(&mut l, f, i, &phi_in, &mut at_end, &mut after_phis)?;
            match new {
                None => out.push(i),
                Some(v) => out.extend(v),
            }
        }
        // The phis' values, after the phis.
        let at = out.iter().position(|&i| !matches!(f.insts[i.index()].op, Op::Phi { .. })).unwrap_or(out.len());
        out.splice(at..at, after_phis);
        f.blocks[b.index()].insts = out;
    }
    for (b, copies) in at_end {
        let insts = &mut f.blocks[b.index()].insts;
        let at = insts.len().saturating_sub(1);
        insts.splice(at..at, copies);
    }
    let entry = f.entry();
    let prologue = std::mem::take(&mut l.prologue);
    f.blocks[entry.index()].insts.splice(0..0, prologue);
    Ok(())
}

fn uses_fp80_constants(cx: &Ctx, f: &Function) -> bool {
    f.insts_in_order().any(|i| {
        let mut any = false;
        f.insts[i.index()].op.for_each_value(|v| {
            if let Value::Const(c) = v {
                any |= is_fp80(cx, cx.m.constant(*c).ty);
            }
        });
        any
    })
}

/// The instructions in place of `i`, or `None` to keep it.
fn lower_one(
    l: &mut Lower,
    f: &mut Function,
    i: InstId,
    phi_in: &HashMap<LocalId, Value>,
    at_end: &mut HashMap<BlockId, Vec<InstId>>,
    after_phis: &mut Vec<InstId>,
) -> Result<Option<Vec<InstId>>, String> {
    let op = f.insts[i.index()].op.clone();
    let res = f.insts[i.index()].result;
    let res_fp80 = res.is_some_and(|r| is_fp80(l.cx, f.locals[r.index()].ty));
    let mut uses = false;
    op.for_each_value(|v| {
        if !matches!(v, Value::Metadata(_)) {
            uses |= is_fp80(l.cx, l.cx.value_ty(f, *v));
        }
    });
    if !res_fp80 && !uses {
        return Ok(None);
    }
    let ptr = l.cx.m.types.ptr(0);
    let dst = res.and_then(|r| l.slots.get(&r).copied());
    let mut new = Vec::new();
    match op {
        Op::Phi { incoming, .. } => {
            let r = res.unwrap();
            let tmp = phi_in[&r];
            for (v, pred) in incoming {
                if matches!(v, Value::Const(c) if matches!(l.cx.m.constant(c).kind, ConstKind::Undef | ConstKind::Poison))
                {
                    continue;
                }
                let src = l.of(f, v)?;
                let mut s = Seq { f, new: Vec::new() };
                copy(l.cx, &mut s, tmp, src);
                at_end.entry(pred).or_default().extend(s.new);
            }
            let mut s = Seq { f, new: Vec::new() };
            copy(l.cx, &mut s, dst.unwrap(), tmp);
            after_phis.extend(s.new);
        }
        Op::Load { ptr: p, atomic: None, .. } => {
            let mut s = Seq { f, new: Vec::new() };
            copy(l.cx, &mut s, dst.unwrap(), p);
            new = s.new;
        }
        Op::Store { value, ptr: p, atomic: None, .. } => {
            let src = l.of(f, value)?;
            let mut s = Seq { f, new: Vec::new() };
            copy(l.cx, &mut s, p, src);
            new = s.new;
        }
        Op::Binary { op: bop @ (BinOp::FAdd | BinOp::FSub | BinOp::FMul | BinOp::FDiv), lhs, rhs, .. } => {
            let (a, b) = (l.of(f, lhs)?, l.of(f, rhs)?);
            let name = match bop {
                BinOp::FAdd => "fadd",
                BinOp::FSub => "fsub",
                BinOp::FMul => "fmul",
                _ => "fdiv",
            };
            let mut s = Seq { f, new: Vec::new() };
            x87(l.cx, &mut s, "fldt", vec![a]);
            x87(l.cx, &mut s, "fldt", vec![b]);
            x87(l.cx, &mut s, name, vec![]);
            x87(l.cx, &mut s, "fstpt", vec![dst.unwrap()]);
            new = s.new;
        }
        Op::FNeg { value, .. } => {
            let a = l.of(f, value)?;
            let mut s = Seq { f, new: Vec::new() };
            x87(l.cx, &mut s, "fldt", vec![a]);
            x87(l.cx, &mut s, "fchs", vec![]);
            x87(l.cx, &mut s, "fstpt", vec![dst.unwrap()]);
            new = s.new;
        }
        Op::Freeze(v) => {
            let a = l.of(f, v)?;
            let mut s = Seq { f, new: Vec::new() };
            copy(l.cx, &mut s, dst.unwrap(), a);
            new = s.new;
        }
        Op::Select { cond, then_value, else_value, .. } if res_fp80 => {
            let (a, b) = (l.of(f, then_value)?, l.of(f, else_value)?);
            let mut s = Seq { f, new: Vec::new() };
            let p = s.op(Op::Select { fmf: FastMath::default(), cond, then_value: a, else_value: b }, ptr);
            copy(l.cx, &mut s, dst.unwrap(), p);
            new = s.new;
        }
        Op::FCmp { pred, lhs, rhs, .. } => {
            let (a, b) = (l.of(f, lhs)?, l.of(f, rhs)?);
            let mut s = Seq { f, new: Vec::new() };
            let c = compare(l.cx, &mut s, pred, a, b);
            s.f.replace_uses(Value::Local(res.unwrap()), c);
            new = s.new;
        }
        Op::Cast { op: CastOp::FPExt, value, .. } => {
            let ty = l.cx.value_ty(f, value);
            let double = l.cx.m.types.intern(Type::Double);
            let t = l.slot(f, 8);
            let mut s = Seq { f, new: Vec::new() };
            let v = match l.cx.m.types.get(ty) {
                Type::Double => value,
                Type::Float => s.cast(CastOp::FPExt, value, double),
                _ => return Err("fpext to x86_fp80 from this type is not supported yet".into()),
            };
            s.op_void(store(v, t));
            x87(l.cx, &mut s, "fldl", vec![t]);
            x87(l.cx, &mut s, "fstpt", vec![dst.unwrap()]);
            new = s.new;
        }
        Op::Cast { op: CastOp::FPTrunc, value, .. } => {
            let r = res.unwrap();
            let ty = f.locals[r.index()].ty;
            let a = l.of(f, value)?;
            let (name, size) = match l.cx.m.types.get(ty) {
                Type::Double => ("fstpl", 8),
                Type::Float => ("fstps", 4),
                _ => return Err("fptrunc of x86_fp80 to this type is not supported yet".into()),
            };
            let t = l.slot(f, size);
            let mut s = Seq { f, new: Vec::new() };
            x87(l.cx, &mut s, "fldt", vec![a]);
            x87(l.cx, &mut s, name, vec![t]);
            let v = s.op(load(t), ty);
            s.f.replace_uses(Value::Local(r), v);
            new = s.new;
        }
        Op::Cast { op: cop @ (CastOp::SIToFP | CastOp::UIToFP), value, .. } => {
            let ty = l.cx.value_ty(f, value);
            let Type::Int(bits) = *l.cx.m.types.get(ty) else {
                return Err("conversions of vectors to x86_fp80 are not supported yet".into());
            };
            if bits > 64 {
                return Err("conversions of integers wider than 64 bits to x86_fp80 are not supported yet".into());
            }
            let i64t = l.cx.m.types.int(64);
            let t = l.slot(f, 8);
            let unsigned64 = cop == CastOp::UIToFP && bits == 64;
            let (two64, sum) = if unsigned64 { (l.constant(f, pow2(64)), l.slot(f, 16)) } else { (t, t) };
            let d = dst.unwrap();
            let mut s = Seq { f, new: Vec::new() };
            let v = match (bits, cop) {
                (64, _) => value,
                (_, CastOp::SIToFP) => s.cast(CastOp::SExt, value, i64t),
                _ => s.cast(CastOp::ZExt, value, i64t),
            };
            s.op_void(store(v, t));
            x87(l.cx, &mut s, "fildll", vec![t]);
            x87(l.cx, &mut s, "fstpt", vec![d]);
            if unsigned64 {
                // Read as negative: 2^64 more.
                x87(l.cx, &mut s, "fldt", vec![d]);
                x87(l.cx, &mut s, "fldt", vec![two64]);
                x87(l.cx, &mut s, "fadd", vec![]);
                x87(l.cx, &mut s, "fstpt", vec![sum]);
                let i1 = l.cx.m.types.int(1);
                let zero = l.cx.int(i64t, 0);
                let neg = s.op(Op::ICmp { pred: ICmpPred::Slt, samesign: false, lhs: v, rhs: zero }, i1);
                let p = s.op(Op::Select { fmf: FastMath::default(), cond: neg, then_value: sum, else_value: d }, ptr);
                copy(l.cx, &mut s, d, p);
            }
            new = s.new;
        }
        Op::Cast { op: cop @ (CastOp::FPToSI | CastOp::FPToUI), value, .. } => {
            let r = res.unwrap();
            let ty = f.locals[r.index()].ty;
            let Type::Int(bits) = *l.cx.m.types.get(ty) else {
                return Err("conversions of x86_fp80 to vectors are not supported yet".into());
            };
            if bits > 64 {
                return Err("conversions of x86_fp80 to integers wider than 64 bits are not supported yet".into());
            }
            let a = l.of(f, value)?;
            let i64t = l.cx.m.types.int(64);
            let v = if cop == CastOp::FPToUI && bits == 64 {
                // Below 2^63, as it is; else less 2^63, with the top bit
                // set after.
                let big = l.constant(f, pow2(63));
                let less = l.slot(f, 16);
                let mut s = Seq { f, new: Vec::new() };
                let small = compare(l.cx, &mut s, FCmpPred::Olt, a, big);
                x87(l.cx, &mut s, "fldt", vec![a]);
                x87(l.cx, &mut s, "fldt", vec![big]);
                x87(l.cx, &mut s, "fsub", vec![]);
                x87(l.cx, &mut s, "fstpt", vec![less]);
                let p =
                    s.op(Op::Select { fmf: FastMath::default(), cond: small, then_value: a, else_value: less }, ptr);
                new.extend(s.new);
                let conv = to_i64(l, f, p);
                let k = *f.insts[conv.last().unwrap().index()].result.as_ref().unwrap();
                new.extend(conv);
                let mut s = Seq { f, new: Vec::new() };
                let top = l.cx.int(i64t, i64::MIN);
                let set = s.bin(BinOp::Xor, Value::Local(k), top, i64t);
                let v = s.op(
                    Op::Select { fmf: FastMath::default(), cond: small, then_value: Value::Local(k), else_value: set },
                    i64t,
                );
                new.extend(s.new);
                v
            } else {
                let conv = to_i64(l, f, a);
                let k = *f.insts[conv.last().unwrap().index()].result.as_ref().unwrap();
                new.extend(conv);
                Value::Local(k)
            };
            let v = if bits < 64 {
                let mut s = Seq { f, new: Vec::new() };
                let t = s.cast(CastOp::Trunc, v, ty);
                new.extend(s.new);
                t
            } else {
                v
            };
            f.replace_uses(Value::Local(r), v);
        }
        Op::Call(ref c) if res_fp80 || uses => {
            let name = callee_name(l.cx.m, c).unwrap_or_default();
            let a = c.args.first().map(|a| a.value);
            match (name.as_str(), a) {
                ("llvm.fabs.f80", Some(v)) => {
                    let a = l.of(f, v)?;
                    let mut s = Seq { f, new: Vec::new() };
                    x87(l.cx, &mut s, "fldt", vec![a]);
                    x87(l.cx, &mut s, "fabs", vec![]);
                    x87(l.cx, &mut s, "fstpt", vec![dst.unwrap()]);
                    new = s.new;
                }
                ("llvm.fmuladd.f80", Some(_)) => {
                    let (a, b, c2) = (l.of(f, c.args[0].value)?, l.of(f, c.args[1].value)?, l.of(f, c.args[2].value)?);
                    let mut s = Seq { f, new: Vec::new() };
                    x87(l.cx, &mut s, "fldt", vec![a]);
                    x87(l.cx, &mut s, "fldt", vec![b]);
                    x87(l.cx, &mut s, "fmul", vec![]);
                    x87(l.cx, &mut s, "fldt", vec![c2]);
                    x87(l.cx, &mut s, "fadd", vec![]);
                    x87(l.cx, &mut s, "fstpt", vec![dst.unwrap()]);
                    new = s.new;
                }
                _ => return Err("calls with x86_fp80 arguments or results are not supported yet".into()),
            }
        }
        _ => return Err("this operation on x86_fp80 is not supported yet".into()),
    }
    Ok(Some(new))
}
