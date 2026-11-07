//! Narrowing, for a machine whose integers are at most `W` bits (one that
//! lacks `i64`; Epitome's `docs/design/partial-machines.md`, section 3).
//! Index arithmetic is often wider than the values in it: Rust's `usize`
//! is 64 bits on a 64-bit target, and JAX's and PyTorch's indices are
//! `i64`, even when every value fits in 32 bits. Such a computation is
//! done in `W` bits where only the low `W` bits of each value are needed.
//!
//! First, a bound on each wide value: how many bits it needs as an
//! unsigned number (it is below 2^u) and as a signed one (it is the sign
//! extension of its low s bits), from what it is made of. A loop counter
//! is bounded by its exit test: one that counts up by one until it equals
//! or passes `n` is at most `n`, and one that counts down by one to zero is
//! at most where it started.
//!
//! Then the values that can be computed in `W` bits: those made by
//! arithmetic, logic, shifts by a constant, extensions from `W` bits or
//! fewer, phis and selects, whose every use needs only their low `W` bits
//! (a truncation to `W` bits or fewer, or another such value), or compares
//! them with another such value where both fit in `W` bits (both unsigned,
//! or both signed). A shift right or a division also needs its operand to
//! fit. Each is given type `iW` in place: an extension from `iW` goes, a
//! truncation to `iW` goes, and a compare becomes a compare of the narrow
//! values (unsigned where both fit unsigned).
//!
//! What cannot be narrowed is left as it is, for `fits` to report.

use crate::apint::ApInt;
use crate::module::*;
use crate::types::{Type, TypeId};

/// `f` (of module `m`) with what can be computed in `widest` bits
/// computed so. `f` itself is not changed.
pub fn narrow(m: &mut Module, f: &Function, widest: u32) -> Function {
    let mut f = f.clone();
    let w = widest;
    if w == 0 || w > 64 {
        return f;
    }
    let width = |m: &Module, ty: TypeId| match m.types.get(ty) {
        Type::Int(b) => *b,
        _ => 0,
    };
    let nl = f.locals.len();
    let wide: Vec<bool> = (0..nl).map(|l| (w + 1..=64).contains(&width(m, f.locals[l].ty))).collect();
    if !wide.iter().any(|&x| x) {
        return f;
    }
    let mut def: Vec<Option<InstId>> = vec![None; nl];
    let mut block_of: Vec<Option<BlockId>> = vec![None; f.insts.len()];
    for &b in &f.layout {
        for &i in &f.blocks[b.index()].insts {
            block_of[i.index()] = Some(b);
            if let Some(r) = f.insts[i.index()].result {
                def[r.index()] = Some(i);
            }
        }
    }
    let order: Vec<InstId> = f.insts_in_order().collect();
    let bits_of = |m: &Module, f: &Function, v: Value| match v {
        Value::Local(l) => width(m, f.locals[l.index()].ty),
        Value::Const(c) => width(m, m.constant(c).ty),
        Value::Metadata(_) => 0,
    };

    // ---------------------------------------------------------------- bounds
    let mut ub: Vec<u32> = (0..nl).map(|l| if def[l].is_some() { 0 } else { width(m, f.locals[l].ty) }).collect();
    let mut sb: Vec<u32> = (0..nl).map(|l| if def[l].is_some() { 1 } else { width(m, f.locals[l].ty) }).collect();
    let loops = counters(m, &f, &def);
    // Each value's bounds can only grow, and are at most its width.
    for _round in 0..200 {
        let mut changed = false;
        for &i in &order {
            let Some(r) = f.insts[i.index()].result else { continue };
            if !wide[r.index()] {
                continue;
            }
            let n = width(m, f.locals[r.index()].ty);
            let get = |v: Value| -> (u32, u32) { value_bounds(m, &ub, &sb, v) };
            let (mut u, mut s) = match &f.insts[i.index()].op {
                Op::Cast { op: CastOp::ZExt, value, .. } => {
                    let (vu, _) = get(*value);
                    (vu, (vu + 1).min(n))
                }
                Op::Cast { op: CastOp::SExt, value, .. } => {
                    let (vu, vs) = get(*value);
                    let k = bits_of(m, &f, *value);
                    (if vu < k { vu } else { n }, vs)
                }
                Op::Cast { op: CastOp::Trunc, value, .. } => {
                    let (vu, vs) = get(*value);
                    (vu.min(n), vs.min(n))
                }
                Op::Freeze(v) => get(*v),
                Op::Select { then_value, else_value, .. } => {
                    let (a, b) = (get(*then_value), get(*else_value));
                    (a.0.max(b.0), a.1.max(b.1))
                }
                Op::Phi { incoming, .. } => incoming.iter().fold((0, 1), |acc, (v, _)| {
                    let x = get(*v);
                    (acc.0.max(x.0), acc.1.max(x.1))
                }),
                Op::Binary { op, flags, lhs, rhs, .. } => {
                    let ((au, as_), (bu, bs)) = (get(*lhs), get(*rhs));
                    let amount = const_u64(m, *rhs);
                    match op {
                        BinOp::Add => ((au.max(bu) + 1).min(n), (as_.max(bs) + 1).min(n)),
                        BinOp::Sub => (if flags.contains(IntFlags::NUW) { au } else { n }, (as_.max(bs) + 1).min(n)),
                        BinOp::Mul => ((au + bu).min(n), (as_ + bs).min(n)),
                        BinOp::And => (au.min(bu), as_.max(bs)),
                        BinOp::Or | BinOp::Xor => (au.max(bu), as_.max(bs)),
                        BinOp::Shl => match amount {
                            Some(c) if c < u64::from(n) => ((au + c as u32).min(n), (as_ + c as u32).min(n)),
                            _ => (n, n),
                        },
                        BinOp::LShr => match amount {
                            Some(c) if c >= 1 && c < u64::from(n) => {
                                let u = au.saturating_sub(c as u32).min(n - c as u32);
                                (u, (u + 1).min(n))
                            }
                            _ => (au, if au < n { au + 1 } else { n }),
                        },
                        BinOp::AShr => match amount {
                            Some(c) if c < u64::from(n) => {
                                let s = as_.saturating_sub(c as u32).max(1);
                                (if au < n { au.saturating_sub(c as u32) } else { n }, s)
                            }
                            _ => (n, as_),
                        },
                        BinOp::UDiv => (au, (au + 1).min(n)),
                        BinOp::URem => (au.min(bu), (au.min(bu) + 1).min(n)),
                        _ => (n, n),
                    }
                }
                _ => (n, n),
            };
            // A loop counter, by its exit test.
            if let Some(&(phi, next, ref how)) = loops.iter().find(|(p, x, _)| *p == r || *x == r) {
                let (start, end) = (how.start.iter().map(|&v| get(v).0).max().unwrap_or(n), how.end.map(|v| get(v).0));
                let bound = match (how.kind, end) {
                    (Kind::UpTo, Some(e)) if r == phi => Some(start.max(e)),
                    (Kind::UpTo, Some(e)) => Some(((start + 1).min(n)).max(e)),
                    (Kind::UpToEqual, Some(e)) if r == phi => Some(start.max(e)),
                    (Kind::UpToEqual, Some(e)) => Some(e),
                    (Kind::DownToZero, _) => Some(start),
                    _ => None,
                };
                let _ = next;
                if let Some(b) = bound {
                    u = u.min(b);
                    s = s.min((u + 1).min(n));
                }
            }
            let (u, s) = (u.min(n), s.clamp(1, n));
            if u > ub[r.index()] || s > sb[r.index()] {
                ub[r.index()] = ub[r.index()].max(u);
                sb[r.index()] = sb[r.index()].max(s);
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }

    // ------------------------------------------------- what can be narrowed
    let fits_u = |m: &Module, v: Value| value_bounds(m, &ub, &sb, v).0 <= w;
    let fits_s = |m: &Module, v: Value| value_bounds(m, &ub, &sb, v).1 <= w;
    let mut users: Vec<Vec<InstId>> = vec![Vec::new(); nl];
    for &i in &order {
        f.insts[i.index()].op.for_each_value(|v| {
            if let Value::Local(l) = v {
                users[l.index()].push(i);
            }
        });
    }
    let mut keep: Vec<bool> = (0..nl).map(|l| wide[l] && def[l].is_some()).collect();
    loop {
        let mut changed = false;
        for l in 0..nl {
            if !keep[l] {
                continue;
            }
            let ok_operand = |v: Value, keep: &[bool]| match v {
                Value::Local(x) => keep[x.index()],
                Value::Const(c) => matches!(
                    m.constant(c).kind,
                    ConstKind::Int(_) | ConstKind::Zero | ConstKind::Undef | ConstKind::Poison
                ),
                Value::Metadata(_) => false,
            };
            let small_amount = |v: Value| const_u64(m, v).is_some_and(|c| c < u64::from(w));
            let i = def[l].unwrap();
            let computable = match &f.insts[i.index()].op {
                Op::Cast { op: CastOp::ZExt | CastOp::SExt, value, .. } => bits_of(m, &f, *value) <= w,
                Op::Cast { op: CastOp::Trunc, value, .. } => ok_operand(*value, &keep),
                Op::Freeze(v) => ok_operand(*v, &keep),
                Op::Select { then_value, else_value, .. } => {
                    ok_operand(*then_value, &keep) && ok_operand(*else_value, &keep)
                }
                Op::Phi { incoming, .. } => incoming.iter().all(|(v, _)| ok_operand(*v, &keep)),
                Op::Binary { op, lhs, rhs, .. } => match op {
                    BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::And | BinOp::Or | BinOp::Xor => {
                        ok_operand(*lhs, &keep) && ok_operand(*rhs, &keep)
                    }
                    BinOp::Shl => ok_operand(*lhs, &keep) && small_amount(*rhs),
                    BinOp::LShr => ok_operand(*lhs, &keep) && fits_u(m, *lhs) && small_amount(*rhs),
                    BinOp::AShr => ok_operand(*lhs, &keep) && fits_s(m, *lhs) && small_amount(*rhs),
                    BinOp::UDiv | BinOp::URem => {
                        ok_operand(*lhs, &keep) && ok_operand(*rhs, &keep) && fits_u(m, *lhs) && fits_u(m, *rhs)
                    }
                    _ => false,
                },
                _ => false,
            };
            let me = Value::Local(LocalId(l as u32));
            let used_well = users[l].iter().all(|&u| {
                let user = &f.insts[u.index()];
                let narrowed = user.result.is_some_and(|r| keep[r.index()]);
                match &user.op {
                    Op::Cast { op: CastOp::Trunc, .. } => {
                        narrowed || user.result.is_some_and(|r| width(m, f.locals[r.index()].ty) <= w)
                    }
                    Op::ICmp { lhs, rhs, .. } => {
                        ok_operand(*lhs, &keep)
                            && ok_operand(*rhs, &keep)
                            && ((fits_u(m, *lhs) && fits_u(m, *rhs)) || (fits_s(m, *lhs) && fits_s(m, *rhs)))
                    }
                    // Not as a shift's amount.
                    Op::Binary { op: BinOp::Shl | BinOp::LShr | BinOp::AShr, rhs, .. } if *rhs == me => false,
                    Op::Select { cond, .. } if *cond == me => false,
                    Op::Binary { .. } | Op::Phi { .. } | Op::Select { .. } | Op::Freeze(_) | Op::Cast { .. } => {
                        narrowed
                    }
                    _ => false,
                }
            });
            if !computable || !used_well {
                keep[l] = false;
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }
    if !keep.iter().any(|&k| k) {
        return f;
    }

    // ---------------------------------------------------------------- rewrite
    let iw = m.types.int(w);
    let low = |m: &mut Module, v: Value| -> Value {
        match v {
            Value::Const(c) => {
                let kind = match &m.constant(c).kind {
                    ConstKind::Int(a) => ConstKind::Int(ApInt::from_u64(w, a.low_u64())),
                    k => k.clone(),
                };
                Value::Const(m.consts.intern(Constant { ty: iw, kind }))
            }
            v => v,
        }
    };
    let mut alias: Vec<(Value, Value)> = Vec::new();
    let mut gone: Vec<InstId> = Vec::new();
    for &i in &order {
        let res = f.insts[i.index()].result;
        if let Some(r) = res
            && keep[r.index()]
        {
            let me = Value::Local(r);
            let mut op = f.insts[i.index()].op.clone();
            match &mut op {
                Op::Cast { op: CastOp::ZExt | CastOp::SExt, value, .. } if bits_of(m, &f, *value) == w => {
                    alias.push((me, *value));
                    gone.push(i);
                }
                Op::Cast { op: CastOp::Trunc, value, .. } => {
                    alias.push((me, low(m, *value)));
                    gone.push(i);
                }
                Op::Freeze(v) => *v = low(m, *v),
                Op::Binary { flags, lhs, rhs, .. } => {
                    *flags = IntFlags::NONE;
                    *lhs = low(m, *lhs);
                    *rhs = low(m, *rhs);
                }
                Op::Phi { incoming, .. } => {
                    for (v, _) in incoming.iter_mut() {
                        *v = low(m, *v);
                    }
                }
                Op::Select { then_value, else_value, .. } => {
                    *then_value = low(m, *then_value);
                    *else_value = low(m, *else_value);
                }
                _ => {}
            }
            f.insts[i.index()].op = op;
            f.locals[r.index()].ty = iw;
            continue;
        }
        // Uses of narrowed values by what stays as it is.
        let narrowed = |v: &Value| matches!(v, Value::Local(l) if keep[l.index()]);
        let mut op = f.insts[i.index()].op.clone();
        match &mut op {
            Op::Cast { op: CastOp::Trunc, value, .. } if narrowed(value) => {
                let r = res.unwrap();
                if width(m, f.locals[r.index()].ty) == w {
                    alias.push((Value::Local(r), *value));
                    gone.push(i);
                }
            }
            Op::ICmp { pred, lhs, rhs, .. } if narrowed(lhs) || narrowed(rhs) => {
                // Both fit unsigned: an unsigned compare, whatever it was.
                if fits_u(m, *lhs) && fits_u(m, *rhs) {
                    *pred = match *pred {
                        ICmpPred::Slt => ICmpPred::Ult,
                        ICmpPred::Sle => ICmpPred::Ule,
                        ICmpPred::Sgt => ICmpPred::Ugt,
                        ICmpPred::Sge => ICmpPred::Uge,
                        p => p,
                    };
                }
                *lhs = low(m, *lhs);
                *rhs = low(m, *rhs);
            }
            _ => {}
        }
        f.insts[i.index()].op = op;
    }
    for &b in &f.layout.clone() {
        f.blocks[b.index()].insts.retain(|i| !gone.contains(i));
    }
    // An alias may be of another (a truncation of an extension).
    let resolve = |v: Value| {
        let mut v = v;
        while let Some(&(_, to)) = alias.iter().find(|(from, _)| *from == v) {
            v = to;
        }
        v
    };
    for &(from, _) in &alias {
        let to = resolve(from);
        f.replace_uses(from, to);
    }
    f
}

/// A value's bounds (unsigned bits, signed bits), as far as they are known.
fn value_bounds(m: &Module, ub: &[u32], sb: &[u32], v: Value) -> (u32, u32) {
    match v {
        Value::Local(l) => (ub[l.index()], sb[l.index()]),
        Value::Const(c) => {
            let k = m.constant(c);
            let n = match m.types.get(k.ty) {
                Type::Int(b) => *b,
                _ => return (64, 64),
            };
            match &k.kind {
                ConstKind::Int(a) if n <= 64 => {
                    let x = a.low_u64() & if n == 64 { u64::MAX } else { (1 << n) - 1 };
                    let u = 64 - x.leading_zeros();
                    let sx = ((x << (64 - n)) as i64) >> (64 - n);
                    let s = if sx >= 0 { 65 - sx.leading_zeros() } else { 65 - (!sx).leading_zeros() };
                    (u, s.clamp(1, n))
                }
                ConstKind::Zero | ConstKind::Undef | ConstKind::Poison => (0, 1),
                _ => (n, n),
            }
        }
        Value::Metadata(_) => (64, 64),
    }
}

fn const_u64(m: &Module, v: Value) -> Option<u64> {
    match v {
        Value::Const(c) => match &m.constant(c).kind {
            ConstKind::Int(a) => Some(a.low_u64()),
            ConstKind::Zero => Some(0),
            _ => None,
        },
        _ => None,
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum Kind {
    /// Up by one while below `end` (an unsigned less-than test).
    UpTo,
    /// Up by one until equal to `end`, without wrapping (`add nuw`, or
    /// from 0 with `end` known not to be 0).
    UpToEqual,
    /// Down by one to zero, from a start known not to be zero.
    DownToZero,
}

#[derive(Clone, Debug)]
struct Counter {
    kind: Kind,
    /// What it starts at, on each way into the loop.
    start: Vec<Value>,
    /// What it counts to (none for down to zero).
    end: Option<Value>,
}

/// The loop counters of `f`: (phi, next, how), where `phi` is the
/// counter, `next` = phi ± 1 is what the loop's back edges carry, and every
/// back edge is taken only while the exit test says so.
fn counters(m: &Module, f: &Function, def: &[Option<InstId>]) -> Vec<(LocalId, LocalId, Counter)> {
    let mut out = Vec::new();
    let preds = f.predecessors();
    let local = |v: Value| match v {
        Value::Local(l) => Some(l),
        _ => None,
    };
    let op_of = |l: LocalId| def[l.index()].map(|i| &f.insts[i.index()].op);
    for &h in &f.layout {
        for &i in &f.blocks[h.index()].insts {
            let Op::Phi { incoming, .. } = &f.insts[i.index()].op else { break };
            let Some(p) = f.insts[i.index()].result else { continue };
            // The value the back edges carry: p + 1 or p - 1.
            let step = |v: Value| -> Option<(LocalId, i64)> {
                let n = local(v)?;
                match op_of(n)? {
                    Op::Binary { op: BinOp::Add, lhs, rhs, flags, .. } if local(*lhs) == Some(p) => {
                        let c = const_u64(m, *rhs)?;
                        let d = if c == 1 {
                            1
                        } else if c == u64::MAX || (c as i64) == -1 {
                            -1
                        } else {
                            return None;
                        };
                        Some((n, d * if flags.contains(IntFlags::NUW) { 2 } else { 1 }))
                    }
                    Op::Binary { op: BinOp::Sub, lhs, rhs, .. }
                        if local(*lhs) == Some(p) && const_u64(m, *rhs) == Some(1) =>
                    {
                        Some((n, -1))
                    }
                    _ => None,
                }
            };
            let Some(&(nextv, _)) = incoming.iter().find(|(v, _)| step(*v).is_some()) else { continue };
            let (next, sign) = step(nextv).unwrap();
            let nuw = sign == 2;
            let up = sign > 0;
            let latches: Vec<BlockId> = incoming.iter().filter(|(v, _)| *v == nextv).map(|(_, b)| *b).collect();
            let start: Vec<(Value, BlockId)> = incoming.iter().filter(|(v, _)| *v != nextv).copied().collect();
            if start.is_empty() {
                continue;
            }
            // Each latch's test: the back edge is taken only when it holds.
            let mut kind = None;
            let mut end = None;
            let mut ok = true;
            for &l in &latches {
                let Some(t) = f.terminator(l) else {
                    ok = false;
                    break;
                };
                let Op::CondBr { cond, then_dest, else_dest } = &f.insts[t.index()].op else {
                    ok = false;
                    break;
                };
                if (*then_dest == h) == (*else_dest == h) {
                    ok = false;
                    break;
                }
                let Some(Op::ICmp { pred, lhs, rhs, .. }) = local(*cond).and_then(op_of) else {
                    ok = false;
                    break;
                };
                let mut pred = *pred;
                if *else_dest == h {
                    pred = inverse(pred);
                }
                let (mut x, mut y) = (*lhs, *rhs);
                if local(y) == Some(next) || local(y) == Some(p) {
                    std::mem::swap(&mut x, &mut y);
                    pred = swapped(pred);
                }
                let on_next = local(x) == Some(next);
                let on_phi = local(x) == Some(p);
                let this = match (up, pred) {
                    (true, ICmpPred::Ult) if on_next || on_phi => Some((Kind::UpTo, Some(y))),
                    (true, ICmpPred::Ne) if on_next => Some((Kind::UpToEqual, Some(y))),
                    (false, ICmpPred::Ne | ICmpPred::Ugt) if on_next && const_u64(m, y) == Some(0) => {
                        Some((Kind::DownToZero, None))
                    }
                    _ => None,
                };
                let Some((k, e)) = this else {
                    ok = false;
                    break;
                };
                if kind.is_some_and(|x| x != k) || (end.is_some() && end != e) {
                    ok = false;
                    break;
                }
                kind = Some(k);
                end = e;
            }
            let Some(kind) = kind.filter(|_| ok) else { continue };
            // Counting until equal must not pass the end: `add nuw` (passing
            // it would wrap, which nuw makes undefined), or from 0 with the
            // end known not to be 0 on the way in. Counting down to zero must
            // not start there.
            let known_nonzero = |v: Value, from: BlockId| {
                const_u64(m, v).is_some_and(|c| c != 0) || entry_guard_nonzero(m, f, &preds, def, v, from, h)
            };
            let fine = match kind {
                Kind::UpTo => true,
                Kind::UpToEqual => {
                    nuw || start.iter().all(|&(s, b)| const_u64(m, s) == Some(0) && known_nonzero(end.unwrap(), b))
                }
                Kind::DownToZero => start.iter().all(|&(s, b)| known_nonzero(s, b)),
            };
            if fine {
                out.push((p, next, Counter { kind, start: start.iter().map(|s| s.0).collect(), end }));
            }
        }
    }
    out
}

/// Whether the edge from `from` into loop header `h` (or into a block
/// that only goes on to `h`) is taken only when `v` is not zero: `from`
/// ends `br (icmp eq x, 0), exit, h` (or `ne`, the other way round), where
/// `x` is `v`, or what `v` is an extension of.
fn entry_guard_nonzero(
    m: &Module,
    f: &Function,
    preds: &[Vec<BlockId>],
    def: &[Option<InstId>],
    v: Value,
    from: BlockId,
    h: BlockId,
) -> bool {
    let source = match v {
        Value::Local(l) => match def[l.index()].map(|i| &f.insts[i.index()].op) {
            Some(Op::Cast { op: CastOp::ZExt | CastOp::SExt, value, .. }) => Some(*value),
            _ => None,
        },
        _ => None,
    };
    let guards = |b: BlockId, into: BlockId| -> bool {
        let Some(t) = f.terminator(b) else { return false };
        let Op::CondBr { cond, then_dest, else_dest } = &f.insts[t.index()].op else { return false };
        let Value::Local(c) = cond else { return false };
        let Some(Op::ICmp { pred, lhs, rhs, .. }) = def[c.index()].map(|i| &f.insts[i.index()].op) else {
            return false;
        };
        let (x, zero) =
            if const_u64(m, *rhs) == Some(0) { (*lhs, true) } else { (*rhs, const_u64(m, *lhs) == Some(0)) };
        if !zero || (x != v && Some(x) != source) {
            return false;
        }
        match pred {
            ICmpPred::Eq => *else_dest == into && *then_dest != into,
            ICmpPred::Ne | ICmpPred::Ugt => *then_dest == into && *else_dest != into,
            _ => false,
        }
    };
    if guards(from, h) {
        return true;
    }
    // A block between: its only predecessor guards it, and it goes on to h.
    let only_on = f.terminator(from).is_some_and(|t| matches!(f.insts[t.index()].op, Op::Br(d) if d == h));
    only_on && preds[from.index()].len() == 1 && guards(preds[from.index()][0], from)
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

fn swapped(p: ICmpPred) -> ICmpPred {
    use ICmpPred::*;
    match p {
        Ugt => Ult,
        Uge => Ule,
        Ult => Ugt,
        Ule => Uge,
        Sgt => Slt,
        Sge => Sle,
        Slt => Sgt,
        Sle => Sge,
        p => p,
    }
}
