//! Loop rotation: turning a loop that tests at the top into one that tests
//! at the bottom.
//!
//! Front ends write `for (i = 0; i < n; i++) body` as
//!
//! ```text
//! preheader:  br header
//! header:     i = phi [0, preheader], [i.next, latch]
//!             c = icmp slt i, n
//!             br c, body, exit
//! body ...    latch: i.next = add i, 1;  br header
//! ```
//!
//! which takes two branches per iteration and keeps the test apart from the
//! work. Rotation copies the header's instructions twice:
//!
//! - into the preheader, as a *guard*, with each header phi replaced by its
//!   value from the preheader: does the loop run at all?
//! - into a new block after the latch, as the *bottom test*, with each phi
//!   replaced by its value from the latch: does it run again?
//!
//! The old header then has no predecessors and goes; `body` is the new
//! header. Every run of the old header corresponds to exactly one run of
//! one of the copies, so nothing is executed that was not before, and any
//! instruction can be copied except those whose identity matters (allocas,
//! calls marked `convergent` or `noduplicate`).
//!
//! Values the header defined now have two definitions, the guard's and the
//! bottom test's, so SSA form is rebuilt with `ssa::SsaUpdater`, which puts
//! phis in the new header where the two meet.
//!
//! A loop is rotated when it has one latch ending in an unconditional
//! branch to the header, a header of at most `MAX_HEADER` instructions
//! ending in a conditional branch with one target in the loop and one out,
//! and an in-loop target reached only from the header.

use epitome_ir::edit::Names;
use epitome_ir::module::*;

use crate::ctx::Shared;
use crate::fx::FxHashMap as HashMap;
use crate::loops::{self, Loop};
use crate::ssa::{self, SsaUpdater};

/// Most instructions copied from a header.
const MAX_HEADER: usize = 16;

#[derive(Default, Debug, Clone, Copy)]
pub struct Stats {
    pub rotated: usize,
}

pub fn run(sh: &Shared, f: &mut Function) -> Stats {
    let mut stats = Stats::default();
    if !f.has_body {
        return stats;
    }
    let mut names = Names::new(f);
    sink_next_values(f);
    // Each rotation changes the blocks, so loops are found again after one.
    for _ in 0..64 {
        let found = loops::find(f);
        let Some(l) = found.iter().find(|l| can_rotate(sh, f, l)) else { break };
        let l = l.clone();
        if !epitome_ir::chop::allow(|| {
            format!("rotate the loop at {}", epitome_ir::printer::local_name(f, f.blocks[l.header.index()].local))
        }) {
            break;
        }
        if rotate(sh, f, &mut names, &l) {
            stats.rotated += 1;
        } else {
            break;
        }
    }
    stats
}

/// Moves a header instruction computing a header phi's next value into the
/// latch, when nothing else uses it: `i.next = add i, 1` left in the header
/// by an inlined `Range::next`, which picks the next value before testing.
/// It then runs only when the loop goes round again, and the loop can be
/// rotated (`can_rotate` refuses a next value made in the header).
fn sink_next_values(f: &mut Function) {
    let found = loops::find(f);
    if found.is_empty() {
        return;
    }
    // Uses of each local, outside the phi entries that carry it round.
    let mut uses = vec![0u32; f.locals.len()];
    for i in f.insts_in_order() {
        f.insts[i.index()].op.for_each_value(|v| {
            if let Value::Local(x) = v {
                uses[x.index()] += 1;
            }
        });
    }
    for l in &found {
        let h = l.header;
        let [latch] = l.latches[..] else { continue };
        if latch == h || !f.terminator(latch).is_some_and(|t| matches!(f.insts[t.index()].op, Op::Br(x) if x == h)) {
            continue;
        }
        for phi in phis(f, h) {
            let Some(Value::Local(x)) = incoming(f, phi, latch) else { continue };
            let LocalDef::Inst(i) = f.locals[x.index()].def else { continue };
            let pure = matches!(
                f.insts[i.index()].op,
                Op::Binary { .. } | Op::Cast { .. } | Op::ICmp { .. } | Op::Select { .. } | Op::Gep { .. }
            );
            // Used once (by this phi), with no debug records of its own to move.
            if !pure || uses[x.index()] != 1 || !f.insts[i.index()].debug_records.is_empty() {
                continue;
            }
            let Some(at) = f.blocks[h.index()].insts.iter().position(|&y| y == i) else { continue };
            f.blocks[h.index()].insts.remove(at);
            let list = &mut f.blocks[latch.index()].insts;
            list.insert(list.len() - 1, i);
        }
    }
}

fn phis(f: &Function, b: BlockId) -> Vec<InstId> {
    f.blocks[b.index()].insts.iter().copied().take_while(|&i| f.insts[i.index()].op.is_phi()).collect()
}

fn incoming(f: &Function, phi: InstId, from: BlockId) -> Option<Value> {
    let Op::Phi { incoming, .. } = &f.insts[phi.index()].op else { return None };
    incoming.iter().find(|(_, b)| *b == from).map(|(v, _)| *v)
}

/// The in-loop and exit targets of the header's branch.
fn header_targets(f: &Function, l: &Loop) -> Option<(Value, BlockId, BlockId)> {
    let t = f.terminator(l.header)?;
    let Op::CondBr { cond, then_dest, else_dest } = f.insts[t.index()].op else { return None };
    match (l.contains(then_dest), l.contains(else_dest)) {
        (true, false) => Some((cond, then_dest, else_dest)),
        (false, true) => Some((cond, else_dest, then_dest)),
        _ => None,
    }
}

fn can_rotate(sh: &Shared, f: &Function, l: &Loop) -> bool {
    let h = l.header;
    let [latch] = l.latches[..] else { return false };
    if latch == h {
        return false;
    }
    if !f.terminator(latch).is_some_and(|t| matches!(f.insts[t.index()].op, Op::Br(x) if x == h)) {
        return false;
    }
    let Some((_, inside, _)) = header_targets(f, l) else { return false };
    let preds = f.predecessors();
    if inside == h || preds[inside.index()].as_slice() != [h] || !phis(f, inside).is_empty() {
        return false;
    }
    let insts = &f.blocks[h.index()].insts;
    let body: Vec<InstId> =
        insts[..insts.len() - 1].iter().copied().filter(|&i| !f.insts[i.index()].op.is_phi()).collect();
    if body.len() > MAX_HEADER {
        return false;
    }
    // A phi whose next value is computed in the header itself would need
    // this iteration's value and the copy's in the same block; not handled.
    let defined_in_header = |v: Value| match v {
        Value::Local(x) => matches!(f.locals[x.index()].def, LocalDef::Inst(d) if insts.contains(&d)),
        _ => false,
    };
    if phis(f, h).iter().any(|&p| incoming(f, p, latch).is_some_and(defined_in_header)) {
        return false;
    }
    body.iter().all(|&i| {
        let op = &f.insts[i.index()].op;
        match op {
            Op::Alloca { .. } | Op::LandingPad { .. } | Op::CatchPad { .. } | Op::CleanupPad { .. } => false,
            Op::Call(call) => {
                let m = sh.module;
                let attrs = |list: &[Attr]| {
                    crate::attrs::has(m, list, "convergent") || crate::attrs::has(m, list, "noduplicate")
                };
                !attrs(&call.fn_attrs)
                    && !sh.direct_callee(call).and_then(|g| sh.published(g)).is_some_and(|c| attrs(&c.fn_attrs))
            }
            op => !op.is_terminator(),
        }
    })
}

/// Copies the header's non-phi instructions to the end of block `to`
/// (before its terminator, if `to` has one), with header values looked up
/// in `map`, which gains the copies.
fn copy_header(f: &mut Function, names: &mut Names, h: BlockId, to: BlockId, map: &mut HashMap<LocalId, Value>) {
    let insts = f.blocks[h.index()].insts.clone();
    let body: Vec<InstId> =
        insts[..insts.len() - 1].iter().copied().filter(|&i| !f.insts[i.index()].op.is_phi()).collect();
    for i in body {
        let mut op = f.insts[i.index()].op.clone();
        op.for_each_value_mut(|v| {
            if let Value::Local(l) = v
                && let Some(&m) = map.get(l)
            {
                *v = m;
            }
        });
        let result = f.insts[i.index()].result.map(|r| {
            let name = f.locals[r.index()].name.clone().map(|n| names.fresh(&n));
            (name, f.locals[r.index()].ty)
        });
        let copy = f.add_inst(op, result);
        f.insts[copy.index()].attachments = f.insts[i.index()].attachments.clone();
        let list = &mut f.blocks[to.index()].insts;
        let at = if list.last().is_some_and(|&t| f.insts[t.index()].op.is_terminator()) {
            list.len() - 1
        } else {
            list.len()
        };
        f.blocks[to.index()].insts.insert(at, copy);
        if let (Some(r), Some(c)) = (f.insts[i.index()].result, f.insts[copy.index()].result) {
            map.insert(r, Value::Local(c));
        }
    }
}

fn rotate(sh: &Shared, f: &mut Function, names: &mut Names, l: &Loop) -> bool {
    let Some(pre) = loops::ensure_preheader(f, names, l) else { return false };
    let h = l.header;
    let latch = l.latches[0];
    let (cond, inside, exit) = header_targets(f, l).unwrap();
    let t = f.terminator(h).unwrap();
    let then_is_inside = matches!(f.insts[t.index()].op, Op::CondBr { then_dest, .. } if then_dest == inside);
    let branch = |c: Value| {
        if then_is_inside {
            Op::CondBr { cond: c, then_dest: inside, else_dest: exit }
        } else {
            Op::CondBr { cond: c, then_dest: exit, else_dest: inside }
        }
    };
    let look = |m: &HashMap<LocalId, Value>, v: Value| match v {
        Value::Local(x) => m.get(&x).copied().unwrap_or(v),
        v => v,
    };

    // What each header value is on entry, and after an iteration.
    let header_phis = phis(f, h);
    let mut on_entry: HashMap<LocalId, Value> = HashMap::default();
    let mut on_repeat: HashMap<LocalId, Value> = HashMap::default();
    for &phi in &header_phis {
        let r = f.insts[phi.index()].result.unwrap();
        on_entry.insert(r, incoming(f, phi, pre).expect("header phi has a preheader entry"));
        on_repeat.insert(r, incoming(f, phi, latch).expect("header phi has a latch entry"));
    }

    // The guard, in the preheader.
    copy_header(f, names, h, pre, &mut on_entry);
    let pre_t = f.terminator(pre).unwrap();
    f.insts[pre_t.index()].op = branch(look(&on_entry, cond));

    // The header is going: its values' names pass to the bottom test's
    // copies and to the phis that will join the two copies, which are the
    // values the loop body sees, as the header's were.
    let header_insts = f.blocks[h.index()].insts.clone();
    for &i in &header_insts {
        if let Some(r) = f.insts[i.index()].result
            && let Some(n) = f.locals[r.index()].name.take()
        {
            names.release(&n);
            f.locals[r.index()].name = Some(n);
        }
    }

    // The bottom test, in a new block after the latch.
    let name = Some(names.fresh(&format!("{}.next", f.block_name(h).unwrap_or("loop"))));
    let bottom = f.add_block(name);
    let at = f.layout.iter().position(|&b| b == latch).unwrap();
    f.layout.insert(at + 1, bottom);
    let latch_t = f.terminator(latch).unwrap();
    f.insts[latch_t.index()].op = Op::Br(bottom);
    copy_header(f, names, h, bottom, &mut on_repeat);
    f.push_inst(bottom, branch(look(&on_repeat, cond)), None);

    // The exit's phis: one entry from each copy instead of the header's.
    for phi in phis(f, exit) {
        let Some(v) = incoming(f, phi, h) else { continue };
        let (a, b) = (look(&on_entry, v), look(&on_repeat, v));
        if let Op::Phi { incoming, .. } = &mut f.insts[phi.index()].op {
            incoming.retain(|(_, from)| *from != h);
            incoming.push((a, pre));
            incoming.push((b, bottom));
        }
    }
    f.layout.retain(|&b| b != h);

    // Rebuild SSA form for header values used elsewhere: each now has a
    // definition in the preheader and one in the bottom block.
    let mut up = SsaUpdater::new(f.predecessors());
    let mut var_of: HashMap<LocalId, usize> = HashMap::default();
    // In header order, so that the phis the updater makes come out in it.
    let defined: Vec<LocalId> = header_insts.iter().filter_map(|&i| f.insts[i.index()].result).collect();
    for r in defined {
        let (Some(&v), Some(&w)) = (on_entry.get(&r), on_repeat.get(&r)) else { continue };
        let ty = f.locals[r.index()].ty;
        let var = up.add_var(ty, f.locals[r.index()].name.clone(), sh.poison(ty));
        up.def(var, pre, v);
        up.def(var, bottom, w);
        var_of.insert(r, var);
    }
    // Keep variable numbering independent of hash order: the updater's
    // phis come out in the order of the uses below, which is layout order.
    let snapshot: Vec<(BlockId, InstId)> =
        f.layout.iter().flat_map(|&b| f.blocks[b.index()].insts.iter().map(move |&i| (b, i))).collect();
    for &(b, i) in &snapshot {
        let mut wanted: Vec<Option<(usize, BlockId)>> = Vec::new();
        match &f.insts[i.index()].op {
            Op::Phi { incoming, .. } => {
                for (v, p) in incoming {
                    wanted.push(match v {
                        Value::Local(x) => var_of.get(x).map(|&k| (k, *p)),
                        _ => None,
                    });
                }
            }
            op => op.for_each_value(|v| {
                wanted.push(match v {
                    Value::Local(x) => var_of.get(x).map(|&k| (k, b)),
                    _ => None,
                })
            }),
        }
        if wanted.iter().all(Option::is_none) {
            continue;
        }
        let values: Vec<Option<Value>> = wanted.iter().map(|w| w.map(|(k, at)| up.value_at(f, names, k, at))).collect();
        let mut it = values.into_iter();
        f.insts[i.index()].op.for_each_value_mut(|v| {
            if let Some(Some(new)) = it.next() {
                *v = new;
            }
        });
    }
    let replaced = ssa::remove_trivial_phis(f, &up.phis);
    crate::recursion::fix_debug_records(sh, f, &up, &var_of, &replaced);
    true
}
