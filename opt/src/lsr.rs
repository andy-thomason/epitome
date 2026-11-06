//! Loop strength reduction: array indexing by a counter becomes pointers
//! stepped through the arrays, and the counter counts down to zero.
//!
//! In a loop that tests at the bottom (see `rotate`), with a counter
//! `i = phi [start, preheader], [i + s, latch]` for a constant step `s`
//! that `indvars` has given an exact exit test (`i.next != bound`):
//!
//! - each `getelementptr T, ptr %base, i64 %i`, for a `%base` that does not
//!   change in the loop, becomes a pointer of its own,
//!   `p = phi [&base[start], preheader], [p + s, latch]`; so does one
//!   indexed by `%o + %i`, for an `%o` that does not change in the loop
//!   (a row of a matrix: `&base[o + start]` to start);
//! - the counter becomes `bound - i`, so the exit test is a comparison with
//!   zero: `c.next = c - s; c.next != 0`.
//!
//! On machines with post-increment addressing (AArch64's `ldr x0, [x1],
//! #8`), stepping a pointer costs nothing beyond the load or store, and a
//! comparison with zero folds into the branch (`cbnz`), so the loop loses
//! its index arithmetic and its comparison.
//!
//! The loop is changed only when every use of the counter is its increment,
//! the exit test, or an address used directly by loads and stores, and at
//! most `MAX_POINTERS` arrays are indexed: each becomes a register live
//! across the loop.

use epitome_ir::edit::Names;
use epitome_ir::module::*;
use epitome_ir::types::{Type, TypeId};

use crate::ctx::Shared;
use crate::loops::{self, Loop};

/// Most arrays turned into pointers in one loop: two inputs and an output
/// of an element-wise loop, and one more.
const MAX_POINTERS: usize = 4;

#[derive(Default, Debug, Clone, Copy)]
pub struct Stats {
    pub pointers: usize,
    pub countdowns: usize,
}

impl Stats {
    pub fn changed(&self) -> bool {
        self.pointers + self.countdowns > 0
    }
}

pub fn run(sh: &Shared, f: &mut Function) -> Stats {
    let mut stats = Stats::default();
    if !f.has_body {
        return stats;
    }
    let mut names = Names::new(f);
    // (Nothing here changes the CFG, so the predecessors stay right.)
    let preds = f.predecessors();
    for l in loops::find(f) {
        reduce(sh, f, &mut names, &l, &preds, &mut stats);
    }
    stats
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
    match def_inst(f, v) {
        Some(d) => !l.blocks.iter().any(|b| f.blocks[b.index()].insts.contains(&d)),
        None => !matches!(v, Value::Metadata(_)),
    }
}

/// Whether `a` and `b` are the same value: the same, or the same integer
/// operation on the same values (two copies of a row's offset that LICM took
/// out of a loop, which CSE has not yet merged), looking `depth` deep.
fn same(f: &Function, a: Value, b: Value, depth: u32) -> bool {
    if a == b {
        return true;
    }
    if depth == 0 {
        return false;
    }
    let (Some(x), Some(y)) = (def_inst(f, a), def_inst(f, b)) else { return false };
    match (&f.insts[x.index()].op, &f.insts[y.index()].op) {
        (
            &Op::Binary { op: o1, lhs: l1, rhs: r1, flags: f1, .. },
            &Op::Binary { op: o2, lhs: l2, rhs: r2, flags: f2, .. },
        ) => o1 == o2 && f1 == f2 && same(f, l1, l2, depth - 1) && same(f, r1, r2, depth - 1),
        _ => false,
    }
}

/// The instructions that use `v`.
fn users(f: &Function, v: Value) -> Vec<InstId> {
    f.insts_in_order()
        .filter(|&i| {
            let mut uses = false;
            f.insts[i.index()].op.for_each_value(|x| uses |= *x == v);
            uses
        })
        .collect()
}

fn in_loop(f: &Function, l: &Loop, i: InstId) -> bool {
    l.blocks.iter().any(|b| f.blocks[b.index()].insts.contains(&i))
}

fn int_const(sh: &Shared, v: Value) -> Option<i64> {
    let Value::Const(c) = v else { return None };
    let k = sh.consts().get(c);
    match (&k.kind, sh.ty(k.ty)) {
        (ConstKind::Int(a), Type::Int(64)) => Some(a.low_u64() as i64),
        _ => None,
    }
}

fn reduce(sh: &Shared, f: &mut Function, names: &mut Names, l: &Loop, preds: &[Vec<BlockId>], stats: &mut Stats) {
    let h = l.header;
    let [latch] = l.latches[..] else { return };
    let outside: Vec<BlockId> = preds[h.index()].iter().copied().filter(|p| !l.contains(*p)).collect();
    let [pre] = outside[..] else { return };
    let i64t = sh.intern_type(Type::Int(64));
    let phis: Vec<InstId> =
        f.blocks[h.index()].insts.iter().copied().take_while(|&i| f.insts[i.index()].op.is_phi()).collect();
    for phi in phis {
        let Some(r) = f.insts[phi.index()].result else { continue };
        if f.locals[r.index()].ty != i64t {
            continue;
        }
        let Op::Phi { incoming, .. } = &f.insts[phi.index()].op else { continue };
        if incoming.len() != 2 {
            continue;
        }
        let start = incoming.iter().find(|(_, b)| *b == pre).map(|(v, _)| *v);
        let next = incoming.iter().find(|(_, b)| *b == latch).map(|(v, _)| *v);
        let (Some(start), Some(next)) = (start, next) else { continue };
        let Some(inc) = def_inst(f, next) else { continue };
        let i_v = Value::Local(r);
        // The step: a positive constant.
        let step = match f.insts[inc.index()].op {
            Op::Binary { op: BinOp::Add, lhs, rhs, .. } if lhs == i_v => int_const(sh, rhs),
            Op::Binary { op: BinOp::Add, lhs, rhs, .. } if rhs == i_v => int_const(sh, lhs),
            _ => None,
        };
        let Some(step) = step.filter(|&s| s > 0) else { continue };
        let step_v = sh.int(64, step);
        // The exit test: the latch's branch on `i` or `i.next` against a
        // bound that does not change, for equality.
        let Some(t) = f.terminator(latch) else { continue };
        let Op::CondBr { cond, .. } = f.insts[t.index()].op else { continue };
        let Some(cmp) = def_inst(f, cond) else { continue };
        let Op::ICmp { pred: pred @ (ICmpPred::Eq | ICmpPred::Ne), lhs, rhs, .. } = f.insts[cmp.index()].op else {
            continue;
        };
        let (tested, bound) = if (lhs == i_v || lhs == next) && invariant(f, l, rhs) {
            (lhs, rhs)
        } else if (rhs == i_v || rhs == next) && invariant(f, l, lhs) {
            (rhs, lhs)
        } else {
            continue;
        };
        if users(f, cond) != vec![t] {
            continue;
        }
        // Every use of the counter: its increment, the test, or an index:
        // the counter plus, perhaps, something that does not change in the
        // loop, plus a constant (a vectorized loop unrolled reads
        // `o + i + 4`). Each address: (the gep, its base, the offset that is
        // not constant, the constant, its type).
        let mut geps: Vec<(InstId, Value, Option<Value>, i64, TypeId)> = Vec::new();
        let mut offsets: Vec<InstId> = Vec::new();
        let mut work: Vec<(Value, Option<Value>, i64)> = vec![(i_v, None, 0)];
        let mut ok_users = true;
        'walk: while let Some((v, off, c)) = work.pop() {
            for u in users(f, v) {
                if v == i_v && (u == inc || u == cmp || u == phi) {
                    continue;
                }
                match &f.insts[u.index()].op {
                    Op::Gep { ptr, indices, source_ty, .. }
                        if indices[..] == [v] && *ptr != v && invariant(f, l, *ptr) && in_loop(f, l, u) =>
                    {
                        let g = f.result(u);
                        let addr_only = users(f, g).into_iter().all(|x| match &f.insts[x.index()].op {
                            Op::Load { ptr, .. } => *ptr == g,
                            Op::Store { ptr, value, .. } => *ptr == g && *value != g,
                            _ => false,
                        });
                        if !addr_only {
                            ok_users = false;
                            break 'walk;
                        }
                        geps.push((u, *ptr, off, c, *source_ty));
                    }
                    &Op::Binary { op: BinOp::Add, lhs, rhs, .. } if in_loop(f, l, u) => {
                        let x = if lhs == v { rhs } else { lhs };
                        let a = f.result(u);
                        match int_const(sh, x) {
                            Some(k) if x != v => work.push((a, off, c.wrapping_add(k))),
                            None if x != v && off.is_none() && invariant(f, l, x) => work.push((a, Some(x), c)),
                            _ => {
                                ok_users = false;
                                break 'walk;
                            }
                        }
                        offsets.push(u);
                    }
                    _ => {
                        ok_users = false;
                        break 'walk;
                    }
                }
            }
        }
        let next_users = users(f, next);
        if !ok_users || !next_users.iter().all(|&u| u == phi || u == cmp) {
            continue;
        }
        // One pointer for each base and offset, offsets that are the same
        // value counting as one.
        let same_offset = |f: &Function, x: Option<Value>, y: Option<Value>| match (x, y) {
            (None, None) => true,
            (Some(x), Some(y)) => same(f, x, y, 4),
            _ => false,
        };
        let mut bases: Vec<(Value, Option<Value>, TypeId)> = Vec::new();
        for g in geps.iter_mut() {
            match bases.iter().find(|&&(b, o, ty)| b == g.1 && ty == g.4 && same_offset(f, o, g.2)) {
                Some(&(_, o, _)) => g.2 = o,
                None => bases.push((g.1, g.2, g.4)),
            }
        }
        if bases.len() > MAX_POINTERS {
            continue;
        }
        if !epitome_ir::chop::allow(|| {
            format!("lsr: the counter {} of the loop at {}", epitome_ir::printer::local_name(f, r), {
                let l = f.blocks[h.index()].local;
                epitome_ir::printer::local_name(f, l)
            })
        }) {
            continue;
        }

        // The counter's debug records no longer describe it.
        let poison = sh.poison(i64t);
        crate::inline::replace_in_debug_records(sh, f, r, poison);
        if let Value::Local(n) = next {
            crate::inline::replace_in_debug_records(sh, f, n, poison);
        }
        let pre_t = f.terminator(pre).unwrap();
        let mut before_pre: Vec<InstId> = Vec::new();
        let latch_t = f.terminator(latch).unwrap();
        let mut before_latch: Vec<InstId> = Vec::new();

        // A pointer for each array.
        for (base, off, ty) in bases {
            let ptr_ty = f.locals[f.insts
                [geps.iter().find(|g| g.1 == base && g.2 == off && g.4 == ty).unwrap().0.index()]
            .result
            .unwrap()
            .index()]
            .ty;
            // Where it starts: base[start], or base[o + start].
            let first_index = match off {
                None => start,
                Some(o) if int_const(sh, start) == Some(0) => o,
                Some(o) => {
                    let a = f.add_inst(
                        Op::Binary {
                            op: BinOp::Add,
                            flags: IntFlags::NONE,
                            fmf: FastMath::default(),
                            lhs: o,
                            rhs: start,
                        },
                        Some((Some(names.fresh("lsr.offset")), i64t)),
                    );
                    before_pre.push(a);
                    f.result(a)
                }
            };
            let first = if off.is_none() && int_const(sh, start) == Some(0) {
                base
            } else {
                let g = f.add_inst(
                    Op::Gep { flags: GepFlags::default(), source_ty: ty, ptr: base, indices: vec![first_index].into() },
                    Some((Some(names.fresh("lsr.start")), ptr_ty)),
                );
                before_pre.push(g);
                f.result(g)
            };
            let p = f.add_inst(
                Op::Phi { fmf: FastMath::default(), incoming: vec![] },
                Some((Some(names.fresh("lsr.ptr")), ptr_ty)),
            );
            f.blocks[h.index()].insts.insert(0, p);
            let pv = f.result(p);
            let step = f.add_inst(
                Op::Gep { flags: GepFlags::default(), source_ty: ty, ptr: pv, indices: vec![step_v].into() },
                Some((Some(names.fresh("lsr.ptr.next")), ptr_ty)),
            );
            before_latch.push(step);
            let sv = f.result(step);
            if let Op::Phi { incoming, .. } = &mut f.insts[p.index()].op {
                incoming.push((first, pre));
                incoming.push((sv, latch));
            }
            for &(g, b, o, c, t2) in &geps {
                if b == base && o == off && t2 == ty {
                    if c == 0 {
                        let gv = f.result(g);
                        f.replace_uses(gv, pv);
                        for blk in &l.blocks {
                            f.blocks[blk.index()].insts.retain(|&x| x != g);
                        }
                    } else {
                        // A constant distance from the pointer: an
                        // immediate offset in the load or store.
                        f.insts[g.index()].op = Op::Gep {
                            flags: GepFlags::default(),
                            source_ty: ty,
                            ptr: pv,
                            indices: vec![sh.int(64, c)].into(),
                        };
                    }
                }
            }
            stats.pointers += 1;
        }

        // The counter counts down to zero: c = bound - i.
        let count_start = match (int_const(sh, bound), int_const(sh, start)) {
            (Some(b), Some(s)) => sh.int(64, b.wrapping_sub(s)),
            (_, Some(0)) => bound,
            _ => {
                let s = f.add_inst(
                    Op::Binary {
                        op: BinOp::Sub,
                        flags: IntFlags::NONE,
                        fmf: FastMath::default(),
                        lhs: bound,
                        rhs: start,
                    },
                    Some((Some(names.fresh("lsr.count.start")), i64t)),
                );
                before_pre.push(s);
                f.result(s)
            }
        };
        if let Op::Phi { incoming, .. } = &mut f.insts[phi.index()].op {
            for (v, b) in incoming.iter_mut() {
                if *b == pre {
                    *v = count_start;
                }
            }
        }
        // The offsets added to the counter are no longer used.
        for &a in &offsets {
            for blk in &l.blocks {
                f.blocks[blk.index()].insts.retain(|&x| x != a);
            }
        }
        let minus_step = sh.int(64, -step);
        f.insts[inc.index()].op =
            Op::Binary { op: BinOp::Add, flags: IntFlags::NONE, fmf: FastMath::default(), lhs: i_v, rhs: minus_step };
        let zero = sh.int(64, 0);
        f.insts[cmp.index()].op = Op::ICmp { pred, samesign: false, lhs: tested, rhs: zero };
        f.locals[r.index()].name = Some(names.fresh("lsr.count"));
        if let Value::Local(n) = next {
            f.locals[n.index()].name = Some(names.fresh("lsr.count.next"));
        }
        stats.countdowns += 1;

        let at = f.blocks[pre.index()].insts.iter().position(|&x| x == pre_t).unwrap();
        f.blocks[pre.index()].insts.splice(at..at, before_pre);
        let at = f.blocks[latch.index()].insts.iter().position(|&x| x == latch_t).unwrap();
        f.blocks[latch.index()].insts.splice(at..at, before_latch);
        // One counter per loop.
        return;
    }
}
