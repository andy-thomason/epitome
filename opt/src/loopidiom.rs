//! Loops that only set memory to one byte become `llvm.memset`: the memset
//! half of LLVM's loop idiom recognition.
//!
//! A loop of one block (rotated: it tests at the bottom), with a counter
//! that steps by 1 and an exit test whose trip count is known, whose only
//! effect is a store of a constant whose bytes are all one byte (`0`, `-1`,
//! `0.0`), or a `memset` of a constant length, to consecutive addresses, an
//! iteration's bytes on from the last's, is replaced by one `memset` in its
//! preheader, and no longer loops. A loop around such a loop then has a
//! `memset` of a row an iteration, and becomes one too, so a fill of a
//! matrix is one call.
//!
//! An address is a base that does not change in the loop plus a byte
//! offset linear in the counter: `getelementptr`s, `add`, `sub`, and `mul`
//! and `shl` by constants. The trip count is `bound - start` for an exact
//! test (`i.next != bound`, which `indvars` makes when a guard proves the
//! loop runs), and that or 1, whichever is more, for `i.next < bound` with
//! an increment that cannot wrap. Nothing the loop computes may be used after
//! it.

use epitome_ir::apint::ApInt;
use epitome_ir::module::*;
use epitome_ir::types::{Type, TypeId};

use crate::combine::eval;
use crate::ctx::Shared;
use crate::loops::{self, Loop};

#[derive(Clone, Copy, Debug, Default)]
pub struct Stats {
    /// Loops made memsets.
    pub memsets: usize,
}

/// A value linear in the loop's counter: `iv` times the counter, plus
/// invariant values times coefficients, plus `k`.
#[derive(Clone, Debug, Default)]
struct Lin {
    iv: i64,
    terms: Vec<(Value, i64)>,
    k: i64,
}

impl Lin {
    fn scaled(mut self, by: i64) -> Option<Lin> {
        self.iv = self.iv.checked_mul(by)?;
        self.k = self.k.checked_mul(by)?;
        for t in &mut self.terms {
            t.1 = t.1.checked_mul(by)?;
        }
        Some(self)
    }

    fn plus(mut self, other: Lin) -> Option<Lin> {
        self.iv = self.iv.checked_add(other.iv)?;
        self.k = self.k.checked_add(other.k)?;
        self.terms.extend(other.terms);
        Some(self)
    }
}

pub fn run(sh: &Shared, f: &mut Function) -> Stats {
    let mut stats = Stats::default();
    if !f.has_body || sh.intrinsics.memset.is_none() {
        return stats;
    }
    // One loop at a time, the CFG found afresh after each.
    let mut names = epitome_ir::edit::Names::new(f);
    'again: loop {
        for l in loops::find(f) {
            if !matches!(&l.blocks[..], [h] if l.latches == [*h]) {
                continue;
            }
            // The loop's one way in; a preheader made for it only once the
            // loop is known to be one to replace.
            let preds = f.predecessors();
            let outside: Vec<BlockId> = preds[l.header.index()].iter().copied().filter(|p| !l.contains(*p)).collect();
            let [from] = outside[..] else { continue };
            if l.preheader(f, &preds).is_none() {
                if !try_loop(sh, f, &l, from, true) || loops::ensure_preheader(f, &mut names, &l).is_none() {
                    continue;
                }
            }
            let preds = f.predecessors();
            let Some(pre) = l.preheader(f, &preds) else { continue };
            if try_loop(sh, f, &l, pre, false) {
                stats.memsets += 1;
                continue 'again;
            }
        }
        return stats;
    }
}

fn def_inst(f: &Function, v: Value) -> Option<InstId> {
    match v {
        Value::Local(x) => match f.locals[x.index()].def {
            LocalDef::Inst(d) => Some(d),
            _ => None,
        },
        _ => None,
    }
}

fn invariant(f: &Function, l: &Loop, v: Value) -> bool {
    match v {
        Value::Const(_) => true,
        Value::Local(x) => match f.locals[x.index()].def {
            LocalDef::Param(_) => true,
            LocalDef::Inst(d) => !l.blocks.iter().any(|b| f.blocks[b.index()].insts.contains(&d)),
            _ => false,
        },
        Value::Metadata(_) => false,
    }
}

fn int_const(sh: &Shared, v: Value) -> Option<i64> {
    let Value::Const(c) = v else { return None };
    let k = eval::int_const(sh.consts(), sh.types(), c)?;
    Some(eval::sext(k.value, k.bits))
}

/// The byte every byte of constant `v` is, if they are all one.
fn splat_byte(sh: &Shared, v: Value) -> Option<(u8, u64)> {
    let Value::Const(c) = v else { return None };
    let k = sh.consts().get(c).clone();
    let size = sh.layout.store_size(sh.types(), k.ty)?;
    if size == 0 || size > 16 {
        return None;
    }
    let bits: u128 = match &k.kind {
        ConstKind::Zero | ConstKind::Null => 0,
        ConstKind::Int(a) if size <= 8 => a.low_u64() as u128,
        ConstKind::Float(b) => *b,
        _ => return None,
    };
    let byte = (bits & 0xff) as u8;
    (0..size).all(|j| ((bits >> (8 * j)) & 0xff) as u8 == byte).then_some((byte, size))
}

/// `v` as a linear form in the loop's counter `iv`.
fn lin_int(sh: &Shared, f: &Function, l: &Loop, iv: Value, v: Value, depth: u32) -> Option<Lin> {
    if v == iv {
        return Some(Lin { iv: 1, ..Lin::default() });
    }
    if let Some(k) = int_const(sh, v) {
        return Some(Lin { k, ..Lin::default() });
    }
    if invariant(f, l, v) {
        return Some(Lin { terms: vec![(v, 1)], ..Lin::default() });
    }
    if depth == 0 {
        return None;
    }
    let d = def_inst(f, v)?;
    let Op::Binary { op, lhs, rhs, .. } = f.insts[d.index()].op else { return None };
    let sub = |x: Value| lin_int(sh, f, l, iv, x, depth - 1);
    match op {
        BinOp::Add => sub(lhs)?.plus(sub(rhs)?),
        BinOp::Sub => sub(lhs)?.plus(sub(rhs)?.scaled(-1)?),
        BinOp::Mul => match (int_const(sh, lhs), int_const(sh, rhs)) {
            (_, Some(k)) => sub(lhs)?.scaled(k),
            (Some(k), _) => sub(rhs)?.scaled(k),
            _ => None,
        },
        BinOp::Shl => {
            let k = int_const(sh, rhs).filter(|k| (0..62).contains(k))?;
            sub(lhs)?.scaled(1 << k)
        }
        _ => None,
    }
}

/// Pointer `v` as an invariant base and a byte offset linear in `iv`.
fn lin_ptr(sh: &Shared, f: &Function, l: &Loop, iv: Value, v: Value, depth: u32) -> Option<(Value, Lin)> {
    if invariant(f, l, v) {
        return Some((v, Lin::default()));
    }
    if depth == 0 {
        return None;
    }
    let d = def_inst(f, v)?;
    let Op::Gep { source_ty, ptr, ref indices, .. } = f.insts[d.index()].op else { return None };
    let (types, dl) = (sh.types(), &sh.layout);
    let (base, mut off) = lin_ptr(sh, f, l, iv, ptr, depth - 1)?;
    let mut ty: TypeId = source_ty;
    for (k, &index) in indices.iter().enumerate() {
        if k > 0 {
            match types.get(ty).clone() {
                Type::Array { elem, .. } => ty = elem,
                Type::Struct { fields, packed } => {
                    let n = usize::try_from(int_const(sh, index)?).ok()?;
                    let layout = dl.struct_layout(types, &fields, packed)?;
                    off.k = off.k.checked_add(*layout.offsets.get(n)? as i64)?;
                    ty = *fields.get(n)?;
                    continue;
                }
                _ => return None,
            }
        }
        let size = dl.alloc_size(types, ty)? as i64;
        off = off.plus(lin_int(sh, f, l, iv, index, 8)?.scaled(size)?)?;
    }
    Some((base, off))
}

/// What a loop sets: the address, an iteration's bytes, the byte, and the
/// instruction that does it.
struct Effect {
    inst: InstId,
    ptr: Value,
    bytes: u64,
    byte: u8,
}

fn effect(sh: &Shared, f: &Function, i: InstId) -> Option<Effect> {
    match &f.insts[i.index()].op {
        Op::Store { volatile: false, atomic: None, value, ptr, .. } => {
            let (byte, bytes) = splat_byte(sh, *value)?;
            Some(Effect { inst: i, ptr: *ptr, bytes, byte })
        }
        Op::Call(call) => {
            let Value::Const(c) = call.callee else { return None };
            let ConstKind::Global(g) = sh.consts().get(c).kind else { return None };
            if Some(g) != sh.intrinsics.memset || call.args.len() != 4 {
                return None;
            }
            let byte = int_const(sh, call.args[1].value)? as u8;
            let bytes = u64::try_from(int_const(sh, call.args[2].value)?).ok()?;
            if int_const(sh, call.args[3].value)? != 0 {
                return None;
            }
            Some(Effect { inst: i, ptr: call.args[0].value, bytes, byte })
        }
        _ => None,
    }
}

/// Replaces loop `l`, entered from `pre`, by a memset if it is one; with
/// `dry`, only says whether it is.
fn try_loop(sh: &Shared, f: &mut Function, l: &Loop, pre: BlockId, dry: bool) -> bool {
    let h = l.header;
    if l.blocks.len() != 1 || l.latches != [h] {
        return false;
    }

    // The bottom test.
    let Some(branch) = f.terminator(h) else { return false };
    let Op::CondBr { cond, then_dest, else_dest } = f.insts[branch.index()].op else { return false };
    let (repeat_on_true, exit) = match (then_dest == h, else_dest == h) {
        (true, false) => (true, else_dest),
        (false, true) => (false, then_dest),
        _ => return false,
    };
    let Some(cmp) = def_inst(f, cond) else { return false };
    let Op::ICmp { pred, lhs, rhs, .. } = f.insts[cmp.index()].op else { return false };

    // The counter: a phi of start and next = phi + 1, tested as next.
    let insts = f.blocks[h.index()].insts.clone();
    let phis: Vec<InstId> = insts.iter().copied().take_while(|&i| f.insts[i.index()].op.is_phi()).collect();
    let [phi] = phis[..] else { return false };
    let Op::Phi { incoming, .. } = &f.insts[phi.index()].op else { return false };
    let start = incoming.iter().find(|(_, b)| *b == pre).map(|(v, _)| *v);
    let next = incoming.iter().find(|(_, b)| *b == h).map(|(v, _)| *v);
    let (Some(start), Some(next)) = (start, next) else { return false };
    let iv = f.result(phi);
    let Some(next_inst) = def_inst(f, next) else { return false };
    let step_flags = match f.insts[next_inst.index()].op {
        Op::Binary { op: BinOp::Add, flags, lhs: a, rhs: b, .. }
            if (a == iv && int_const(sh, b) == Some(1)) || (b == iv && int_const(sh, a) == Some(1)) =>
        {
            flags
        }
        _ => return false,
    };
    let (pred, bound) = if lhs == next && invariant(f, l, rhs) {
        (pred, rhs)
    } else if rhs == next && invariant(f, l, lhs) {
        (eval::swapped(pred), lhs)
    } else {
        return false;
    };
    let repeat = if repeat_on_true { pred } else { inverse(pred) };
    // The trip count: bound - start for an exact test; for `<`, at least
    // one (the loop tests at the bottom), when the step cannot wrap.
    let constant_trip = matches!((int_const(sh, start), int_const(sh, bound)), (Some(s), Some(b)) if s < b);
    let at_least_one = match repeat {
        ICmpPred::Ne => None,
        ICmpPred::Slt | ICmpPred::Ult if constant_trip => None,
        ICmpPred::Slt if step_flags.contains(IntFlags::NSW) => Some(ICmpPred::Slt),
        ICmpPred::Ult if step_flags.contains(IntFlags::NUW) => Some(ICmpPred::Ult),
        _ => return false,
    };

    // One effect; everything else arithmetic the loop alone uses.
    let mut found: Option<Effect> = None;
    for &i in &insts {
        if i == phi || i == next_inst || i == cmp || i == branch {
            continue;
        }
        if let Some(e) = effect(sh, f, i) {
            if found.is_some() {
                return false;
            }
            found = Some(e);
            continue;
        }
        if !matches!(f.insts[i.index()].op, Op::Binary { .. } | Op::Gep { .. } | Op::Cast { .. }) {
            return false;
        }
    }
    let Some(e) = found else { return false };
    // Nothing from the loop used outside it.
    for user in f.insts_in_order().collect::<Vec<_>>() {
        if insts.contains(&user) {
            continue;
        }
        let mut uses_loop = false;
        f.insts[user.index()].op.for_each_value(|v| {
            if let Some(d) = def_inst(f, *v)
                && insts.contains(&d)
            {
                uses_loop = true;
            }
        });
        if uses_loop {
            return false;
        }
    }
    // Consecutive: an iteration's bytes on from the last's.
    let Some((base, off)) = lin_ptr(sh, f, l, iv, e.ptr, 16) else { return false };
    if off.iv != e.bytes as i64 || e.bytes == 0 {
        return false;
    }
    if dry {
        return true;
    }
    if !epitome_ir::chop::allow(|| format!("loopidiom: the loop at %{} as a memset", f.block_name(h).unwrap_or("?"))) {
        return false;
    }

    // In the preheader: the address at the start, the trip count, the call.
    let i64t = sh.intern_type(Type::Int(64));
    let i8t = sh.intern_type(Type::Int(8));
    let ptr_ty = sh.intern_type(Type::Ptr(0));
    let at = f.blocks[pre.index()].insts.len() - 1;
    let mut new: Vec<InstId> = Vec::new();
    let mut emit = |f: &mut Function, op: Op, ty: TypeId| -> Value {
        let i = f.add_inst(op, Some((None, ty)));
        new.push(i);
        f.result(i)
    };
    let wide = |f: &mut Function, emit: &mut dyn FnMut(&mut Function, Op, TypeId) -> Value, v: Value| -> Value {
        if let Some(k) = int_const(sh, v) {
            return sh.int(64, k);
        }
        let ty = match v {
            Value::Local(x) => f.locals[x.index()].ty,
            Value::Const(c) => sh.consts().get(c).ty,
            Value::Metadata(_) => return v,
        };
        if matches!(sh.types().get(ty), Type::Int(64)) {
            return v;
        }
        emit(f, Op::Cast { op: CastOp::SExt, flags: IntFlags::NONE, fmf: FastMath::default(), value: v }, i64t)
    };
    let binary = |op: BinOp, lhs: Value, rhs: Value| Op::Binary {
        op,
        flags: IntFlags::NONE,
        fmf: FastMath::default(),
        lhs,
        rhs,
    };
    let start64 = wide(f, &mut emit, start);
    let bound64 = wide(f, &mut emit, bound);
    // The offset at the counter's start, its constant parts added up.
    let mut k = off.k;
    let mut offset: Option<Value> = None;
    let mut terms = off.terms.clone();
    terms.push((start64, off.iv));
    for (v, coef) in terms {
        if let Some(c) = int_const(sh, v) {
            k = k.wrapping_add(c.wrapping_mul(coef));
            continue;
        }
        let v = wide(f, &mut emit, v);
        let scaled = if coef == 1 { v } else { emit(f, binary(BinOp::Mul, v, sh.int(64, coef)), i64t) };
        offset = Some(match offset {
            Some(o) => emit(f, binary(BinOp::Add, o, scaled), i64t),
            None => scaled,
        });
    }
    let offset = match (offset, k) {
        (None, k) => sh.int(64, k),
        (Some(o), 0) => o,
        (Some(o), k) => emit(f, binary(BinOp::Add, o, sh.int(64, k)), i64t),
    };
    let address = if int_const(sh, offset) == Some(0) {
        base
    } else {
        emit(f, Op::Gep { flags: GepFlags::default(), source_ty: i8t, ptr: base, indices: vec![offset].into() }, ptr_ty)
    };
    let mut trip = emit(f, binary(BinOp::Sub, bound64, start64), i64t);
    if let Some(pred) = at_least_one {
        let i1t = sh.intern_type(Type::Int(1));
        let enters = emit(f, Op::ICmp { pred, samesign: false, lhs: start64, rhs: bound64 }, i1t);
        trip = emit(
            f,
            Op::Select { fmf: FastMath::default(), cond: enters, then_value: trip, else_value: sh.int(64, 1) },
            i64t,
        );
    }
    let bytes = emit(f, binary(BinOp::Mul, trip, sh.int(64, e.bytes as i64)), i64t);
    let memset = sh.global_ref(sh.intrinsics.memset.unwrap());
    let fn_ty = sh.published(sh.intrinsics.memset.unwrap()).expect("llvm.memset is declared").ty;
    let i1 = sh.intern_type(Type::Int(1));
    let no = Value::Const(sh.intern_const(Constant { ty: i1, kind: ConstKind::Int(ApInt::from_u64(1, 0)) }));
    let call = crate::inline::plain_call(fn_ty, memset, vec![address, sh.int(8, e.byte as i64), bytes, no]);
    let c = f.add_inst(Op::Call(Box::new(call)), None);
    new.push(c);
    for (k, i) in new.into_iter().enumerate() {
        f.blocks[pre.index()].insts.insert(at + k, i);
    }

    // The loop no longer sets anything, and goes round once.
    f.blocks[h.index()].insts.retain(|&i| i != e.inst);
    f.insts[branch.index()].op = Op::Br(exit);
    if let Op::Phi { incoming, .. } = &mut f.insts[phi.index()].op {
        incoming.retain(|(_, b)| *b != h);
    }
    true
}

fn inverse(p: ICmpPred) -> ICmpPred {
    use ICmpPred::*;
    match p {
        Eq => Ne,
        Ne => Eq,
        Ugt => Ule,
        Uge => Ult,
        Ult => Uge,
        Ule => Ugt,
        Sgt => Sle,
        Sge => Slt,
        Slt => Sge,
        Sle => Sgt,
    }
}
