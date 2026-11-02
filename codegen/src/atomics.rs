//! Atomics in selection form: read-modify-write, compare-and-swap, and
//! ordered loads and stores, as the machine has them.
//!
//! - An `atomicrmw` the description has an instruction for (AArch64's
//!   `ldadd` with FEAT_LSE, RISC-V's `amoadd`) stays as it is; `sub` is
//!   first an `add` of the negation.
//! - A `cmpxchg` becomes `llvm.epitome.cmpxchg.iN(ptr, expected, new,
//!   rank)`, which returns the old value (AArch64's `cas`), its `rank` what
//!   `target::cmpxchg_rank` makes of the orderings. Success is that value
//!   equal to the one expected.
//! - Without those, where the description names outline helpers
//!   (`atomic-helpers`: libgcc's and compiler-rt's `__aarch64_ldadd4_acq`),
//!   each is a call of one.
//! - Where it has load-reserved and store-conditional instead (RISC-V's
//!   `lr` and `sc`, as `llvm.epitome.lr.iN` and `llvm.epitome.sc.iN`), the
//!   rest are loops of those; 8 and 16-bit ones work on the aligned 32-bit
//!   word holding them, the other bytes written back as they were.
//! - Anything else (`nand`, the floating-point operations, `uinc_wrap`) is
//!   a loop of compare-and-swap.
//! - Pointers and floating-point values go through integers of their
//!   width.
//! - A sequentially consistent store, where the description says so
//!   (`seq-cst-stores` = `swap`: x86), is an `atomicrmw xchg`.
//! - Ordered loads and stores the machine has no instruction for are
//!   plain ones between fences (the RISC-V psABI's mapping): a load
//!   acquires with `fence r, rw` after it, and is sequentially consistent
//!   with `fence rw, rw` before it too; a store releases with a fence
//!   before it.
//!
//! - Atomics of widths the machine has none of (`i128`), or of memory not
//!   aligned to their size, are calls of the C library's generic ones
//!   (`__atomic_load`, `__atomic_store`, `__atomic_compare_exchange`, with
//!   the size, the value through memory), as clang's are; a
//!   read-modify-write is a loop of compare-and-swaps of them.
//!
//! What each emits is never paired with or moved past other memory
//! accesses (`MInst::barrier`).

use epitome_ir::module::*;
use epitome_ir::types::{Type, TypeId};

use crate::prepare::{Ctx, Seq, call_intrinsic, replace_pair};
use crate::target::{Head, cmpxchg_rank, order_rank};

/// What happened at one instruction: `n` instructions in its place (1 if
/// it stayed), or a loop made, and the block after it to go on in.
enum Step {
    Next(usize),
    Split(BlockId),
}

pub fn expand_atomics(cx: &mut Ctx, f: &mut Function) -> Result<(), String> {
    let mut b = 0;
    'blocks: while b < f.layout.len() {
        let blk = f.layout[b];
        let mut k = 0;
        while k < f.blocks[blk.index()].insts.len() {
            match expand_one(cx, f, blk, k)? {
                Step::Next(n) => k += n,
                Step::Split(done) => {
                    b = f.layout.iter().position(|&x| x == done).unwrap();
                    continue 'blocks;
                }
            }
        }
        b += 1;
    }
    Ok(())
}

/// `new` in place of the instruction at `k`.
fn replace(f: &mut Function, blk: BlockId, k: usize, new: Vec<InstId>) -> Step {
    let n = new.len();
    f.blocks[blk.index()].insts.splice(k..=k, new);
    Step::Next(n)
}

fn expand_one(cx: &mut Ctx, f: &mut Function, blk: BlockId, k: usize) -> Result<Step, String> {
    let i = f.blocks[blk.index()].insts[k];
    if let Some(size) = library_size(cx, f, i) {
        return library_atomic(cx, f, blk, k, size);
    }
    match f.insts[i.index()].op.clone() {
        Op::CmpXchg { ptr, cmp, new, success, failure, align, .. } => {
            let r = f.insts[i.index()].result.ok_or("a cmpxchg without a result")?;
            let sty = f.locals[r.index()].ty;
            let ty = cx.value_ty(f, cmp);
            let (ity, bits) = int_of(cx, ty, align)?;
            let rank = cmpxchg_rank(success, failure);
            let mut s = Seq { f, new: Vec::new() };
            let (c, n) = (to_int(cx, &mut s, cmp, ty, ity), to_int(cx, &mut s, new, ty, ity));
            let i1 = cx.m.types.int(1);
            if has_cas(cx, bits, rank) {
                let old = cas(cx, &mut s, ptr, c, n, bits, rank);
                let ok = s.op(Op::ICmp { pred: ICmpPred::Eq, samesign: false, lhs: old, rhs: c }, i1);
                let old = from_int(cx, &mut s, old, ty, ity);
                replace_pair(cx, &mut s, r, sty, old, ok);
                let new = std::mem::take(&mut s.new);
                return Ok(replace(f, blk, k, new));
            }
            let Some(word) = Word::new(cx, &mut s, ptr, bits) else {
                return Err(format!("this target has no {bits}-bit compare-and-swap"));
            };
            // lp: the old value reserved; unless it is the one expected,
            // done. tr: the new one stored, or lp again.
            let pre = std::mem::take(&mut s.new);
            let (lp, done) = open_loop(f, blk, k, pre);
            let tr = f.add_block(None);
            let at = f.layout.iter().position(|&x| x == lp).unwrap();
            f.layout.insert(at + 1, tr);
            let mut s = Seq { f, new: Vec::new() };
            let w = word.reserve(cx, &mut s, rank);
            let old = word.extract(cx, &mut s, w);
            let ne = s.op(Op::ICmp { pred: ICmpPred::Ne, samesign: false, lhs: old, rhs: c }, i1);
            let body = std::mem::take(&mut s.new);
            let nw = word.insert(cx, &mut s, w, n);
            let again = word.store(cx, &mut s, nw, rank);
            let retry = std::mem::take(&mut s.new);
            let ok = s.op(Op::ICmp { pred: ICmpPred::Eq, samesign: false, lhs: old, rhs: c }, i1);
            let old_v = from_int(cx, &mut s, old, ty, ity);
            replace_pair(cx, &mut s, r, sty, old_v, ok);
            let after = std::mem::take(&mut s.new);
            f.blocks[lp.index()].insts.extend(body);
            f.push_inst(lp, Op::CondBr { cond: ne, then_dest: done, else_dest: tr }, None);
            f.blocks[tr.index()].insts.extend(retry);
            f.push_inst(tr, Op::CondBr { cond: again, then_dest: lp, else_dest: done }, None);
            f.blocks[done.index()].insts.splice(0..0, after);
            Ok(Step::Split(done))
        }
        Op::AtomicRmw { op, elementwise, ptr, value, ordering, align, .. } => {
            if elementwise {
                return Err("elementwise atomicrmw is not supported".into());
            }
            let r = f.insts[i.index()].result.ok_or("an atomicrmw without a result")?;
            let ty = f.locals[r.index()].ty;
            let (ity, bits) = int_of(cx, ty, align)?;
            let float = is_float(cx, ty);
            let mut s = Seq { f, new: Vec::new() };
            let (orig, mut op) = (op, op);
            let mut v = if float { value } else { to_int(cx, &mut s, value, ty, ity) };
            if op == RmwOp::Sub && has_rmw(cx, RmwOp::Add, bits, ordering) {
                let zero = cx.int(ity, 0);
                v = s.bin(BinOp::Sub, zero, v, ity);
                op = RmwOp::Add;
            }
            // An instruction for it, or a helper.
            let direct = !float && has_rmw(cx, op, bits, ordering);
            if direct && op == orig && v == value && ity == ty {
                return Ok(Step::Next(1));
            }
            let old = if direct {
                Some(s.op(
                    Op::AtomicRmw {
                        op,
                        volatile: false,
                        elementwise: false,
                        ptr,
                        value: v,
                        scope: None,
                        ordering,
                        align,
                    },
                    ity,
                ))
            } else if !float {
                rmw_helper(cx, &mut s, op, ptr, v, bits, ordering)
            } else {
                None
            };
            if let Some(old) = old {
                let old = from_int(cx, &mut s, old, ty, ity);
                s.f.replace_uses(Value::Local(r), old);
                let new = std::mem::take(&mut s.new);
                return Ok(replace(f, blk, k, new));
            }
            // The new value, from the old one: of a float's bits, as the
            // float.
            let next_of = |cx: &mut Ctx, s: &mut Seq, old: Value| -> Result<Value, String> {
                let cur = if float { s.cast(CastOp::BitCast, old, ty) } else { old };
                let next = combine(cx, s, op, cur, v, ty)?;
                Ok(if float { s.cast(CastOp::BitCast, next, ity) } else { next })
            };
            let cas_rank = cmpxchg_rank(ordering, failure_of(ordering));
            if has_cas(cx, bits, cas_rank) {
                // A loop: the value seen, the new one made from it, and a
                // compare-and-swap of one for the other, until nothing
                // came between.
                let first = s.op(
                    Op::Load {
                        volatile: false,
                        atomic: Some(Atomic { elementwise: false, scope: None, ordering: Ordering::Monotonic }),
                        ptr,
                        align: Some(Align::of(u64::from(bits / 8))),
                    },
                    ity,
                );
                let pre = std::mem::take(&mut s.new);
                let (lp, done) = open_loop(f, blk, k, pre);
                let mut s = Seq { f, new: Vec::new() };
                let phi = s.f.add_inst(Op::Phi { fmf: FastMath::default(), incoming: vec![] }, Some((None, ity)));
                let seen = s.f.result(phi);
                let next = next_of(cx, &mut s, seen)?;
                let old = cas(cx, &mut s, ptr, seen, next, bits, cas_rank);
                let i1 = cx.m.types.int(1);
                let ok = s.op(Op::ICmp { pred: ICmpPred::Eq, samesign: false, lhs: old, rhs: seen }, i1);
                let body = std::mem::take(&mut s.new);
                let result = from_int(cx, &mut s, seen, ty, ity);
                let after = std::mem::take(&mut s.new);
                f.insts[phi.index()].op = Op::Phi { fmf: FastMath::default(), incoming: vec![(first, blk), (old, lp)] };
                f.blocks[lp.index()].insts.push(phi);
                f.blocks[lp.index()].insts.extend(body);
                f.push_inst(lp, Op::CondBr { cond: ok, then_dest: done, else_dest: lp }, None);
                f.replace_uses(Value::Local(r), result);
                f.blocks[done.index()].insts.splice(0..0, after);
                return Ok(Step::Split(done));
            }
            let Some(word) = Word::new(cx, &mut s, ptr, bits) else {
                return Err(format!("this target has no {bits}-bit atomic read-modify-write"));
            };
            let rank = order_rank(ordering);
            if float {
                // Floating-point instructions between lr and sc could
                // keep it from ever storing (a loop of lr and sc is sure
                // to finish only of base integer instructions): the new
                // value is made outside, from the one seen, and the loop
                // of lr and sc is a compare-and-swap of it.
                let first = s.op(
                    Op::Load {
                        volatile: false,
                        atomic: Some(Atomic { elementwise: false, scope: None, ordering: Ordering::Monotonic }),
                        ptr,
                        align: Some(Align::of(u64::from(bits / 8))),
                    },
                    ity,
                );
                let pre = std::mem::take(&mut s.new);
                let (lp, done) = open_loop(f, blk, k, pre);
                let at = f.layout.iter().position(|&x| x == lp).unwrap();
                let (inner, tr) = (f.add_block(None), f.add_block(None));
                f.layout.splice(at + 1..at + 1, [inner, tr]);
                let mut s = Seq { f, new: Vec::new() };
                let phi = s.f.add_inst(Op::Phi { fmf: FastMath::default(), incoming: vec![] }, Some((None, ity)));
                let seen = s.f.result(phi);
                let next = next_of(cx, &mut s, seen)?;
                let body = std::mem::take(&mut s.new);
                let w = word.reserve(cx, &mut s, rank);
                let i1 = cx.m.types.int(1);
                let ne = s.op(Op::ICmp { pred: ICmpPred::Ne, samesign: false, lhs: w, rhs: seen }, i1);
                let check = std::mem::take(&mut s.new);
                let again = word.store(cx, &mut s, next, rank);
                let retry = std::mem::take(&mut s.new);
                let result = from_int(cx, &mut s, seen, ty, ity);
                let after = std::mem::take(&mut s.new);
                f.insts[phi.index()].op =
                    Op::Phi { fmf: FastMath::default(), incoming: vec![(first, blk), (w, inner)] };
                f.blocks[lp.index()].insts.push(phi);
                f.blocks[lp.index()].insts.extend(body);
                f.push_inst(lp, Op::Br(inner), None);
                f.blocks[inner.index()].insts.extend(check);
                f.push_inst(inner, Op::CondBr { cond: ne, then_dest: lp, else_dest: tr }, None);
                f.blocks[tr.index()].insts.extend(retry);
                f.push_inst(tr, Op::CondBr { cond: again, then_dest: inner, else_dest: done }, None);
                f.replace_uses(Value::Local(r), result);
                f.blocks[done.index()].insts.splice(0..0, after);
                return Ok(Step::Split(done));
            }
            // A loop: the old value reserved, the new one made from it and
            // stored, or again.
            let pre = std::mem::take(&mut s.new);
            let (lp, done) = open_loop(f, blk, k, pre);
            let mut s = Seq { f, new: Vec::new() };
            let w = word.reserve(cx, &mut s, rank);
            let old = word.extract(cx, &mut s, w);
            let next = next_of(cx, &mut s, old)?;
            let nw = word.insert(cx, &mut s, w, next);
            let again = word.store(cx, &mut s, nw, rank);
            let body = std::mem::take(&mut s.new);
            let result = from_int(cx, &mut s, old, ty, ity);
            let after = std::mem::take(&mut s.new);
            f.blocks[lp.index()].insts.extend(body);
            f.push_inst(lp, Op::CondBr { cond: again, then_dest: lp, else_dest: done }, None);
            f.replace_uses(Value::Local(r), result);
            f.blocks[done.index()].insts.splice(0..0, after);
            Ok(Step::Split(done))
        }
        Op::Load { atomic: Some(at), ptr, align, volatile } => {
            let r = f.insts[i.index()].result.ok_or("a load without a result")?;
            let ty = f.locals[r.index()].ty;
            let (ity, bits) = int_of(cx, ty, align)?;
            if !ordered(at.ordering) {
                return Ok(Step::Next(1));
            }
            let mut s = Seq { f, new: Vec::new() };
            if cx.selects_head(|h| matches!(h, Head::LoadAcq(b) if *b == bits)) {
                // Of the integer of a float's bits (`ldar` has no
                // floating-point registers).
                if !is_float(cx, ty) {
                    return Ok(Step::Next(1));
                }
                let v = s.op(Op::Load { atomic: Some(at), ptr, align, volatile }, ity);
                let v = s.cast(CastOp::BitCast, v, ty);
                s.f.replace_uses(Value::Local(r), v);
            } else {
                if at.ordering == Ordering::SeqCst {
                    s.op_void(Op::Fence { scope: None, ordering: Ordering::SeqCst });
                }
                let relaxed = Atomic { ordering: Ordering::Monotonic, ..at };
                let v = s.op(Op::Load { atomic: Some(relaxed), ptr, align, volatile }, ty);
                s.op_void(Op::Fence { scope: None, ordering: Ordering::Acquire });
                s.f.replace_uses(Value::Local(r), v);
            }
            let new = std::mem::take(&mut s.new);
            Ok(replace(f, blk, k, new))
        }
        Op::Store { atomic: Some(at), value, ptr, align, volatile } => {
            let ty = cx.value_ty(f, value);
            let (ity, bits) = int_of(cx, ty, align)?;
            if !ordered(at.ordering) {
                return Ok(Step::Next(1));
            }
            let mut s = Seq { f, new: Vec::new() };
            if at.ordering == Ordering::SeqCst && cx.t.machine.seq_cst_swap {
                let v = to_int(cx, &mut s, value, ty, ity);
                let rmw = Op::AtomicRmw {
                    op: RmwOp::Xchg,
                    volatile,
                    elementwise: false,
                    ptr,
                    value: v,
                    scope: None,
                    ordering: Ordering::SeqCst,
                    align,
                };
                s.op(rmw, ity);
            } else if cx.selects_head(|h| matches!(h, Head::StoreRel(b) if *b == bits)) {
                if !is_float(cx, ty) {
                    return Ok(Step::Next(1));
                }
                let v = s.cast(CastOp::BitCast, value, ity);
                s.op_void(Op::Store { atomic: Some(at), value: v, ptr, align, volatile });
            } else {
                s.op_void(Op::Fence { scope: None, ordering: Ordering::Release });
                let relaxed = Atomic { ordering: Ordering::Monotonic, ..at };
                s.op_void(Op::Store { atomic: Some(relaxed), value, ptr, align, volatile });
            }
            let new = std::mem::take(&mut s.new);
            Ok(replace(f, blk, k, new))
        }
        _ => Ok(Step::Next(1)),
    }
}

/// The size in bytes of the atomic at `i`, if it is one for the C
/// library's generic functions: a width the machine has no atomics of, or
/// memory not aligned to its size.
fn library_size(cx: &Ctx, f: &Function, i: InstId) -> Option<u64> {
    let (ty, align) = match &f.insts[i.index()].op {
        Op::CmpXchg { cmp, align, .. } => (cx.value_ty(f, *cmp), *align),
        Op::AtomicRmw { elementwise: false, align, .. } | Op::Load { atomic: Some(_), align, .. } => {
            (f.locals[f.insts[i.index()].result?.index()].ty, *align)
        }
        Op::Store { atomic: Some(_), value, align, .. } => (cx.value_ty(f, *value), *align),
        _ => return None,
    };
    let bits = match cx.m.types.get(ty) {
        Type::Int(b) => *b,
        Type::Ptr(0) | Type::Double => 64,
        Type::Float => 32,
        Type::Half => 16,
        _ => return None,
    };
    let bytes = u64::from(bits.div_ceil(8));
    let odd = !matches!(bits, 8 | 16 | 32 | 64);
    let misaligned = align.is_some_and(|a| a.bytes() < bytes);
    (bits % 8 == 0 && (odd || misaligned)).then_some(bytes)
}

/// C's number for ordering `o` (`__ATOMIC_RELAXED` 0 to `__ATOMIC_SEQ_CST` 5).
fn c_order(o: Ordering) -> i64 {
    match o {
        Ordering::Acquire => 2,
        Ordering::Release => 3,
        Ordering::AcqRel => 4,
        Ordering::SeqCst => 5,
        _ => 0,
    }
}

/// A stack slot of type `ty`, made at the function's entry.
fn slot(cx: &mut Ctx, f: &mut Function, ty: TypeId) -> Value {
    let ptr = cx.m.types.ptr(0);
    let a = f.add_inst(
        Op::Alloca { inalloca: false, ty, count: None, align: Some(Align::of(16)), addrspace: None },
        Some((None, ptr)),
    );
    let entry = f.layout[0];
    f.blocks[entry.index()].insts.insert(0, a);
    f.result(a)
}

/// The atomic at `k` of `blk` (of `size` bytes), as the C library's
/// generic functions do it (see the top of this file).
fn library_atomic(cx: &mut Ctx, f: &mut Function, blk: BlockId, k: usize, size: u64) -> Result<Step, String> {
    // (Its slots go at the entry, moving `k` if it is there.)
    let i = f.blocks[blk.index()].insts[k];
    let entry = f.layout[0];
    let op = f.insts[i.index()].op.clone();
    let (i64t, i32t, i1) = (cx.m.types.int(64), cx.m.types.int(32), cx.m.types.int(1));
    let void = cx.m.types.intern(Type::Void);
    let n = cx.int(i64t, size as i64);
    let mut slots = 0;
    let mut make = |cx: &mut Ctx, f: &mut Function, ty| {
        slots += 1;
        slot(cx, f, ty)
    };
    let (step, new) = match op {
        Op::Load { ptr, atomic: Some(at), .. } => {
            let r = f.insts[i.index()].result.unwrap();
            let ty = f.locals[r.index()].ty;
            let out = make(cx, f, ty);
            let mut s = Seq { f, new: Vec::new() };
            let o = cx.int(i32t, c_order(at.ordering));
            call_intrinsic(cx, &mut s, "__atomic_load", vec![n, ptr, out, o], void);
            let v = s.op(Op::Load { volatile: false, atomic: None, ptr: out, align: None }, ty);
            s.f.replace_uses(Value::Local(r), v);
            (None, s.new)
        }
        Op::Store { value, ptr, atomic: Some(at), .. } => {
            let ty = cx.value_ty(f, value);
            let from = make(cx, f, ty);
            let mut s = Seq { f, new: Vec::new() };
            s.op_void(Op::Store { volatile: false, atomic: None, value, ptr: from, align: None });
            let o = cx.int(i32t, c_order(at.ordering));
            call_intrinsic(cx, &mut s, "__atomic_store", vec![n, ptr, from, o], void);
            (None, s.new)
        }
        Op::CmpXchg { ptr, cmp, new, success, failure, .. } => {
            let r = f.insts[i.index()].result.unwrap();
            let sty = f.locals[r.index()].ty;
            let ty = cx.value_ty(f, cmp);
            let (e, d) = (make(cx, f, ty), make(cx, f, ty));
            let mut s = Seq { f, new: Vec::new() };
            s.op_void(Op::Store { volatile: false, atomic: None, value: cmp, ptr: e, align: None });
            s.op_void(Op::Store { volatile: false, atomic: None, value: new, ptr: d, align: None });
            let (so, fo) = (cx.int(i32t, c_order(success)), cx.int(i32t, c_order(failure)));
            let ok = call_intrinsic(cx, &mut s, "__atomic_compare_exchange", vec![n, ptr, e, d, so, fo], i1);
            // Where it failed, the value it found is where the one expected was.
            let old = s.op(Op::Load { volatile: false, atomic: None, ptr: e, align: None }, ty);
            replace_pair(cx, &mut s, r, sty, old, ok);
            (None, s.new)
        }
        Op::AtomicRmw { op, ptr, value, ordering, .. } => {
            let r = f.insts[i.index()].result.unwrap();
            let ty = f.locals[r.index()].ty;
            let (e, d) = (make(cx, f, ty), make(cx, f, ty));
            let k = if blk == entry { k + slots } else { k };
            // The value seen (read plainly: a torn read is only a failed
            // compare-and-swap); then a loop of compare-and-swaps of the
            // new value made from it, until nothing came between.
            let mut s = Seq { f, new: Vec::new() };
            let first = s.op(Op::Load { volatile: false, atomic: None, ptr, align: Some(Align::of(1)) }, ty);
            let pre = std::mem::take(&mut s.new);
            let (lp, done) = open_loop(f, blk, k, pre);
            let mut s = Seq { f, new: Vec::new() };
            let phi = s.f.add_inst(Op::Phi { fmf: FastMath::default(), incoming: vec![] }, Some((None, ty)));
            let seen = s.f.result(phi);
            let next = combine(cx, &mut s, op, seen, value, ty)?;
            s.op_void(Op::Store { volatile: false, atomic: None, value: seen, ptr: e, align: None });
            s.op_void(Op::Store { volatile: false, atomic: None, value: next, ptr: d, align: None });
            let so = cx.int(i32t, c_order(ordering));
            let fo = cx.int(i32t, c_order(failure_of(ordering)));
            let ok = call_intrinsic(cx, &mut s, "__atomic_compare_exchange", vec![n, ptr, e, d, so, fo], i1);
            let cur = s.op(Op::Load { volatile: false, atomic: None, ptr: e, align: None }, ty);
            let body = std::mem::take(&mut s.new);
            f.insts[phi.index()].op = Op::Phi { fmf: FastMath::default(), incoming: vec![(first, blk), (cur, lp)] };
            f.blocks[lp.index()].insts.push(phi);
            f.blocks[lp.index()].insts.extend(body);
            f.push_inst(lp, Op::CondBr { cond: ok, then_dest: done, else_dest: lp }, None);
            f.replace_uses(Value::Local(r), seen);
            (Some(done), Vec::new())
        }
        _ => unreachable!(),
    };
    if let Some(done) = step {
        return Ok(Step::Split(done));
    }
    let k = if blk == entry { k + slots } else { k };
    Ok(replace(f, blk, k, new))
}

/// Makes room for a loop at instruction `k` of `blk`, which goes: `blk`
/// keeps what was before it, then `pre`; the loop block (empty, for the
/// caller to fill and end) follows; then the block of what came after.
fn open_loop(f: &mut Function, blk: BlockId, k: usize, pre: Vec<InstId>) -> (BlockId, BlockId) {
    let lp = f.split_block(blk, k, None);
    let done = f.split_block(lp, 1, None);
    f.blocks[lp.index()].insts.clear();
    f.blocks[blk.index()].insts.extend(pre);
    f.push_inst(blk, Op::Br(lp), None);
    (lp, done)
}

/// The failure ordering for a compare-and-swap of success ordering `o`:
/// no release, and no stronger.
fn failure_of(o: Ordering) -> Ordering {
    match o {
        Ordering::AcqRel | Ordering::Acquire => Ordering::Acquire,
        Ordering::SeqCst => Ordering::SeqCst,
        _ => Ordering::Monotonic,
    }
}

fn is_float(cx: &Ctx, ty: TypeId) -> bool {
    matches!(cx.m.types.get(ty), Type::Float | Type::Double | Type::Half)
}

/// Does `o` order other accesses (more than `monotonic`)?
fn ordered(o: Ordering) -> bool {
    !matches!(o, Ordering::Unordered | Ordering::Monotonic)
}

/// The integer type an atomic value of type `ty` goes through, and its
/// width: one the machine does atomically, naturally aligned.
fn int_of(cx: &mut Ctx, ty: TypeId, align: Option<Align>) -> Result<(TypeId, u32), String> {
    let bits = match cx.m.types.get(ty) {
        Type::Int(b) => *b,
        Type::Ptr(0) | Type::Double => 64,
        Type::Float => 32,
        Type::Half => 16,
        _ => return Err("atomics of this type are not supported".into()),
    };
    if !matches!(bits, 8 | 16 | 32 | 64) {
        return Err(format!("atomics of {bits} bits are not supported yet"));
    }
    if align.is_some_and(|a| a.bytes() < u64::from(bits / 8)) {
        return Err("misaligned atomics are not supported".into());
    }
    Ok((cx.m.types.int(bits), bits))
}

fn to_int(cx: &mut Ctx, s: &mut Seq, v: Value, ty: TypeId, ity: TypeId) -> Value {
    match cx.m.types.get(ty) {
        _ if ty == ity => v,
        Type::Ptr(_) => s.cast(CastOp::PtrToInt, v, ity),
        _ => s.cast(CastOp::BitCast, v, ity),
    }
}

fn from_int(cx: &mut Ctx, s: &mut Seq, v: Value, ty: TypeId, ity: TypeId) -> Value {
    match cx.m.types.get(ty) {
        _ if ty == ity => v,
        Type::Ptr(_) => s.cast(CastOp::IntToPtr, v, ty),
        _ => s.cast(CastOp::BitCast, v, ty),
    }
}

/// Has the description an instruction, that is on, for `op` at `bits`
/// with this ordering?
fn has_rmw(cx: &Ctx, op: RmwOp, bits: u32, ordering: Ordering) -> bool {
    let rank = order_rank(ordering);
    cx.selects_head(|h| matches!(h, Head::Rmw(o, b, r) if *o == op && *b == bits && *r == rank))
}

/// The outline helper's name for this operation, width and ordering
/// (`order_rank`'s), if the description names them (`atomic-helpers`).
fn helper(cx: &Ctx, op: &str, bits: u32, rank: u8) -> Option<String> {
    let template = cx.t.machine.atomic_helpers.as_ref()?;
    let order = ["relax", "acq", "rel", "acq_rel"][rank as usize];
    Some(template.replace("{op}", op).replace("{bytes}", &(bits / 8).to_string()).replace("{order}", order))
}

/// An outline helper's call for `op`, returning the old value.
fn rmw_helper(
    cx: &mut Ctx,
    s: &mut Seq,
    op: RmwOp,
    ptr: Value,
    v: Value,
    bits: u32,
    ordering: Ordering,
) -> Option<Value> {
    let name = match op {
        RmwOp::Xchg => "swp",
        RmwOp::Add | RmwOp::Sub => "ldadd",
        RmwOp::And => "ldclr",
        RmwOp::Or => "ldset",
        RmwOp::Xor => "ldeor",
        _ => return None,
    };
    let name = helper(cx, name, bits, order_rank(ordering))?;
    let ity = cx.m.types.int(bits);
    let v = match op {
        RmwOp::Sub => {
            let zero = cx.int(ity, 0);
            s.bin(BinOp::Sub, zero, v, ity)
        }
        RmwOp::And => {
            let ones = cx.int(ity, -1);
            s.bin(BinOp::Xor, v, ones, ity)
        }
        _ => v,
    };
    Some(call_intrinsic(cx, s, &name, vec![v, ptr], ity))
}

/// Is there a compare-and-swap of `bits` with orderings `rank`: an
/// instruction or a helper?
fn has_cas(cx: &Ctx, bits: u32, rank: u8) -> bool {
    cx.selects_intrinsic_with(&format!("llvm.epitome.cmpxchg.i{bits}"), i64::from(rank))
        || helper(cx, "cas", bits, rank).is_some()
}

/// A compare-and-swap of `new` for `expected` at `ptr`, returning the
/// value there before (`has_cas` said there is one).
fn cas(cx: &mut Ctx, s: &mut Seq, ptr: Value, expected: Value, new: Value, bits: u32, rank: u8) -> Value {
    let ity = cx.m.types.int(bits);
    let name = format!("llvm.epitome.cmpxchg.i{bits}");
    if cx.selects_intrinsic_with(&name, i64::from(rank)) {
        let i32t = cx.m.types.int(32);
        let rv = cx.int(i32t, i64::from(rank));
        return call_intrinsic(cx, s, &name, vec![ptr, expected, new, rv], ity);
    }
    let h = helper(cx, "cas", bits, rank).unwrap();
    call_intrinsic(cx, s, &h, vec![expected, new, ptr], ity)
}

/// The word a loop of load-reserved and store-conditional works on, for
/// a value of `bits` at `ptr`: the value itself, or the aligned 32-bit
/// word holding it.
struct Word {
    addr: Value,
    /// The width reserved (`bits`, or 32).
    wbits: u32,
    bits: u32,
    /// The value's place in a 32-bit word: its shift, and the mask of the
    /// word's other bits.
    place: Option<(Value, Value)>,
}

impl Word {
    fn new(cx: &mut Ctx, s: &mut Seq, ptr: Value, bits: u32) -> Option<Word> {
        let has = |cx: &Ctx, w: u32| cx.selects_intrinsic_with(&format!("llvm.epitome.lr.i{w}"), 0);
        if has(cx, bits) {
            return Some(Word { addr: ptr, wbits: bits, bits, place: None });
        }
        if bits >= 32 || !has(cx, 32) {
            return None;
        }
        // addr = ptr & ~3, shift = (ptr & 3) * 8, others = ~(ones << shift)
        let (i64t, i32t, pty) = (cx.m.types.int(64), cx.m.types.int(32), cx.m.types.ptr(0));
        let p = s.cast(CastOp::PtrToInt, ptr, i64t);
        let m3 = cx.int(i64t, !3);
        let a = s.bin(BinOp::And, p, m3, i64t);
        let addr = s.cast(CastOp::IntToPtr, a, pty);
        let low = s.cast(CastOp::Trunc, p, i32t);
        let three = cx.int(i32t, 3);
        let byte = s.bin(BinOp::And, low, three, i32t);
        let shift = s.bin(BinOp::Shl, byte, three, i32t);
        let ones = cx.int(i32t, (1i64 << bits) - 1);
        let mask = s.bin(BinOp::Shl, ones, shift, i32t);
        let all = cx.int(i32t, -1);
        let others = s.bin(BinOp::Xor, mask, all, i32t);
        Some(Word { addr, wbits: 32, bits, place: Some((shift, others)) })
    }

    /// The word, reserved, with the acquire of `rank` (`order_rank`'s).
    fn reserve(&self, cx: &mut Ctx, s: &mut Seq, rank: u8) -> Value {
        // lr has .aq and .aqrl; a release alone is sc's.
        let lr_rank = match rank {
            0 | 2 => 0,
            1 => 1,
            _ => 3,
        };
        let (wty, i32t) = (cx.m.types.int(self.wbits), cx.m.types.int(32));
        let rv = cx.int(i32t, lr_rank);
        call_intrinsic(cx, s, &format!("llvm.epitome.lr.i{}", self.wbits), vec![self.addr, rv], wty)
    }

    /// The value, from the word.
    fn extract(&self, cx: &mut Ctx, s: &mut Seq, w: Value) -> Value {
        let Some((shift, _)) = self.place else { return w };
        let (i32t, ty) = (cx.m.types.int(32), cx.m.types.int(self.bits));
        let v = s.bin(BinOp::LShr, w, shift, i32t);
        s.cast(CastOp::Trunc, v, ty)
    }

    /// The word, with `v` in the value's place.
    fn insert(&self, cx: &mut Ctx, s: &mut Seq, w: Value, v: Value) -> Value {
        let Some((shift, others)) = self.place else { return v };
        let i32t = cx.m.types.int(32);
        let kept = s.bin(BinOp::And, w, others, i32t);
        let x = s.cast(CastOp::ZExt, v, i32t);
        let x = s.bin(BinOp::Shl, x, shift, i32t);
        s.bin(BinOp::Or, kept, x, i32t)
    }

    /// The store-conditional of the word, with the release of `rank`:
    /// true if it failed.
    fn store(&self, cx: &mut Ctx, s: &mut Seq, w: Value, rank: u8) -> Value {
        let (i1, i32t) = (cx.m.types.int(1), cx.m.types.int(32));
        let rv = cx.int(i32t, if rank & 2 != 0 { 2 } else { 0 });
        let st = call_intrinsic(cx, s, &format!("llvm.epitome.sc.i{}", self.wbits), vec![self.addr, w, rv], i32t);
        let zero = cx.int(i32t, 0);
        s.op(Op::ICmp { pred: ICmpPred::Ne, samesign: false, lhs: st, rhs: zero }, i1)
    }
}

/// What `op` writes, given the old value `old` and the operand `v`.
fn combine(cx: &mut Ctx, s: &mut Seq, op: RmwOp, old: Value, v: Value, ty: TypeId) -> Result<Value, String> {
    let i1 = cx.m.types.int(1);
    let cmp = |s: &mut Seq, pred, l, r| s.op(Op::ICmp { pred, samesign: false, lhs: l, rhs: r }, i1);
    let select =
        |s: &mut Seq, c, t, e| s.op(Op::Select { fmf: FastMath::default(), cond: c, then_value: t, else_value: e }, ty);
    let suffix = match cx.m.types.get(ty) {
        Type::Float => "f32",
        Type::Double => "f64",
        Type::Half => "f16",
        _ => "",
    };
    Ok(match op {
        RmwOp::Xchg => v,
        RmwOp::Add => s.bin(BinOp::Add, old, v, ty),
        RmwOp::Sub => s.bin(BinOp::Sub, old, v, ty),
        RmwOp::And => s.bin(BinOp::And, old, v, ty),
        RmwOp::Or => s.bin(BinOp::Or, old, v, ty),
        RmwOp::Xor => s.bin(BinOp::Xor, old, v, ty),
        RmwOp::Nand => {
            let a = s.bin(BinOp::And, old, v, ty);
            let ones = cx.int(ty, -1);
            s.bin(BinOp::Xor, a, ones, ty)
        }
        RmwOp::Max | RmwOp::Min | RmwOp::UMax | RmwOp::UMin => {
            let pred = match op {
                RmwOp::Max => ICmpPred::Sgt,
                RmwOp::Min => ICmpPred::Slt,
                RmwOp::UMax => ICmpPred::Ugt,
                _ => ICmpPred::Ult,
            };
            let c = cmp(s, pred, old, v);
            select(s, c, old, v)
        }
        RmwOp::FAdd => s.bin(BinOp::FAdd, old, v, ty),
        RmwOp::FSub => s.bin(BinOp::FSub, old, v, ty),
        RmwOp::FMax | RmwOp::FMin | RmwOp::FMaximum | RmwOp::FMinimum | RmwOp::FMaximumNum | RmwOp::FMinimumNum => {
            let base = match op {
                RmwOp::FMax => "maxnum",
                RmwOp::FMin => "minnum",
                RmwOp::FMaximum => "maximum",
                RmwOp::FMinimum => "minimum",
                RmwOp::FMaximumNum => "maximumnum",
                _ => "minimumnum",
            };
            call_intrinsic(cx, s, &format!("llvm.{base}.{suffix}"), vec![old, v], ty)
        }
        // old u>= v ? 0 : old + 1
        RmwOp::UIncWrap => {
            let (one, zero) = (cx.int(ty, 1), cx.int(ty, 0));
            let inc = s.bin(BinOp::Add, old, one, ty);
            let c = cmp(s, ICmpPred::Uge, old, v);
            select(s, c, zero, inc)
        }
        // old == 0 || old u> v ? v : old - 1
        RmwOp::UDecWrap => {
            let (one, zero) = (cx.int(ty, 1), cx.int(ty, 0));
            let dec = s.bin(BinOp::Sub, old, one, ty);
            let z = cmp(s, ICmpPred::Eq, old, zero);
            let gt = cmp(s, ICmpPred::Ugt, old, v);
            let c = s.bin(BinOp::Or, z, gt, i1);
            select(s, c, v, dec)
        }
        // old u>= v ? old - v : old (usub_cond), or 0 (usub_sat)
        RmwOp::USubCond | RmwOp::USubSat => {
            let d = s.bin(BinOp::Sub, old, v, ty);
            let c = cmp(s, ICmpPred::Uge, old, v);
            let other = if op == RmwOp::USubCond { old } else { cx.int(ty, 0) };
            select(s, c, d, other)
        }
    })
}
