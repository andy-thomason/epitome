//! Cutting a call-graph cycle so the rest of it can be inlined.
//!
//! Within a cycle, the inliner may inline any call except the ones that
//! actually close it. This chooses those calls, a *cut*: a set of calls
//! such that the members' other calls among themselves are acyclic. Its
//! members can then be finished callees first, each one inlining its
//! uncut calls to the members finished before it, as if they were in SCCs
//! of their own.
//!
//! A cut call is marked with the call-site attribute `"epitome-cycle-cut"`,
//! so that it stays cut when the body holding it is inlined somewhere else:
//! the copy is still a call to the same function, and still not inlined,
//! which is what keeps inlining from unrolling the cycle. The marks are
//! removed once the bottom-up passes are done (`strip`).
//!
//! The calls cut are the cheapest to leave as calls:
//!
//! 1. cold calls: in a block that only leads to `unreachable` (a panic), or
//!    to a `noreturn` function;
//! 2. then calls to the biggest functions, which would not be inlined
//!    anyway.
//!
//! The members are ordered by a greedy heuristic for the minimum feedback
//! arc set (Eades, Lin and Smyth), weighted by those costs: functions only
//! called go first and functions that only call go last, and otherwise the
//! one whose outgoing calls outweigh its incoming ones most. The calls
//! that go backwards in that order are the cut. Then each cut call that
//! would close no cycle is put back, the most costly first. A self-call is
//! always cut.

use epitome_ir::module::*;

use crate::attrs;
use crate::ctx::Shared;

/// The call-site attribute marking a cut call.
pub const CUT: &[u8] = b"epitome-cycle-cut";

/// True if `call` is a cut call.
pub fn is_cut(call: &Call) -> bool {
    call.fn_attrs.iter().any(|a| matches!(a, Attr::Str { key, .. } if key == CUT))
}

/// The cost of cutting a cold call; a warm one costs 10, plus `WARM`
/// divided by one more than the callee's size.
const COLD: u64 = 1;
const WARM: u64 = 1000;

#[derive(Debug, Default)]
pub struct Cut {
    /// The members, callees first by their uncut calls.
    pub order: Vec<usize>,
    /// Calls between distinct members that were cut, and of those, how
    /// many were cold.
    pub calls: usize,
    pub cold: usize,
    /// Self-calls (always cut).
    pub self_calls: usize,
}

/// Chooses a cut for the cycle `members` (see the module's notes), marks
/// its calls, and returns the order to finish the members in.
pub fn choose(sh: &Shared, members: &mut [(GlobalId, Box<Function>)]) -> Cut {
    let n = members.len();
    let index: crate::fx::FxHashMap<GlobalId, usize> = members.iter().enumerate().map(|(k, (g, _))| (*g, k)).collect();
    let size: Vec<u64> =
        members.iter().map(|(_, f)| f.layout.iter().map(|b| f.blocks[b.index()].insts.len() as u64).sum()).collect();
    // The cost of cutting each edge, summed over its calls.
    let mut weight: Vec<crate::fx::FxHashMap<usize, u64>> = vec![Default::default(); n];
    let mut cold_calls: Vec<crate::fx::FxHashMap<usize, usize>> = vec![Default::default(); n];
    for (a, (_, f)) in members.iter().enumerate() {
        let blocks = f.inst_blocks();
        for i in f.insts_in_order() {
            let Some(call) = f.insts[i.index()].op.call() else { continue };
            let Some(&b) = sh.direct_callee(call).and_then(|t| index.get(&t)) else { continue };
            let callee = &members[b].1;
            let cold = attrs::has(sh.module, &call.fn_attrs, "noreturn")
                || attrs::has(sh.module, &callee.fn_attrs, "noreturn")
                || blocks[i.index()].is_some_and(|blk| crate::inline::ends_in_unreachable(f, blk));
            let cost = if cold { COLD } else { 10 + WARM / (1 + size[b]) };
            *weight[a].entry(b).or_default() += cost;
            if cold {
                *cold_calls[a].entry(b).or_default() += 1;
            }
        }
    }
    let rank = order_greedy(n, &weight);
    // Callers come first in `rank`: an edge to an earlier node is cut.
    let mut kept: Vec<Vec<usize>> = vec![Vec::new(); n];
    let mut cut: Vec<(u64, usize, usize)> = Vec::new();
    for a in 0..n {
        for (&b, &w) in &weight[a] {
            if a != b && rank[a] < rank[b] {
                kept[a].push(b);
            } else if a != b {
                cut.push((w, a, b));
            }
        }
    }
    // Put back the cut edges that close no cycle, the most costly first.
    cut.sort_by(|x, y| y.0.cmp(&x.0).then(x.1.cmp(&y.1)).then(x.2.cmp(&y.2)));
    let mut really_cut: Vec<(usize, usize)> = Vec::new();
    for &(_, a, b) in &cut {
        if reaches(&kept, b, a) {
            really_cut.push((a, b));
        } else {
            kept[a].push(b);
        }
    }
    let mut out = Cut { order: postorder(&kept), ..Default::default() };
    // Mark the calls.
    let mut cut_of: Vec<Vec<usize>> = vec![Vec::new(); n];
    for &(a, b) in &really_cut {
        cut_of[a].push(b);
        out.cold += cold_calls[a].get(&b).copied().unwrap_or(0);
    }
    for (a, (g, f)) in members.iter_mut().enumerate() {
        let order: Vec<InstId> = f.insts_in_order().collect();
        for i in order {
            let Some(call) = f.insts[i.index()].op.call() else { continue };
            let Some(t) = sh.direct_callee(call) else { continue };
            let Some(&b) = index.get(&t) else { continue };
            let self_call = t == *g;
            if !self_call && !cut_of[a].contains(&b) {
                continue;
            }
            let call = f.insts[i.index()].op.call_mut().unwrap();
            if !is_cut(call) {
                call.fn_attrs.push(Attr::Str { key: CUT.to_vec(), value: None });
            }
            if self_call {
                out.self_calls += 1;
            } else {
                out.calls += 1;
            }
        }
    }
    out
}

/// A rank for each node: callers before callees for as much of the weight
/// as the greedy heuristic finds.
fn order_greedy(n: usize, weight: &[crate::fx::FxHashMap<usize, u64>]) -> Vec<usize> {
    let mut preds: Vec<Vec<(usize, u64)>> = vec![Vec::new(); n];
    for a in 0..n {
        for (&b, &w) in &weight[a] {
            if a != b {
                preds[b].push((a, w));
            }
        }
    }
    let mut gone = vec![false; n];
    let (mut out_w, mut in_w) = (vec![0u64; n], vec![0u64; n]);
    let (mut out_n, mut in_n) = (vec![0usize; n], vec![0usize; n]);
    for a in 0..n {
        for (&b, &w) in &weight[a] {
            if a != b {
                out_w[a] += w;
                in_w[b] += w;
                out_n[a] += 1;
                in_n[b] += 1;
            }
        }
    }
    let (mut front, mut back) = (Vec::new(), Vec::new());
    let mut left = n;
    let remove = |v: usize,
                  gone: &mut Vec<bool>,
                  out_w: &mut Vec<u64>,
                  in_w: &mut Vec<u64>,
                  out_n: &mut Vec<usize>,
                  in_n: &mut Vec<usize>| {
        gone[v] = true;
        for (&b, &w) in &weight[v] {
            if b != v && !gone[b] {
                in_w[b] -= w;
                in_n[b] -= 1;
            }
        }
        for &(a, w) in &preds[v] {
            if !gone[a] {
                out_w[a] -= w;
                out_n[a] -= 1;
            }
        }
    };
    while left > 0 {
        // Sinks to the back, sources to the front, as long as there are.
        let mut moved = true;
        while moved {
            moved = false;
            for v in 0..n {
                if gone[v] {
                    continue;
                }
                if out_n[v] == 0 {
                    back.push(v);
                } else if in_n[v] == 0 {
                    front.push(v);
                } else {
                    continue;
                }
                remove(v, &mut gone, &mut out_w, &mut in_w, &mut out_n, &mut in_n);
                left -= 1;
                moved = true;
            }
        }
        if left == 0 {
            break;
        }
        // Then the node whose outgoing calls outweigh its incoming ones most.
        let v = (0..n)
            .filter(|&v| !gone[v])
            .max_by_key(|&v| (out_w[v] as i64 - in_w[v] as i64, std::cmp::Reverse(v)))
            .unwrap();
        front.push(v);
        remove(v, &mut gone, &mut out_w, &mut in_w, &mut out_n, &mut in_n);
        left -= 1;
    }
    back.reverse();
    front.extend(back);
    let mut rank = vec![0; n];
    for (k, &v) in front.iter().enumerate() {
        rank[v] = k;
    }
    rank
}

/// True if `to` can be reached from `from` along `succ`.
fn reaches(succ: &[Vec<usize>], from: usize, to: usize) -> bool {
    let mut seen = vec![false; succ.len()];
    let mut work = vec![from];
    while let Some(v) = work.pop() {
        if v == to {
            return true;
        }
        if !std::mem::replace(&mut seen[v], true) {
            work.extend(succ[v].iter().copied());
        }
    }
    false
}

/// The nodes of the acyclic graph `succ`, each after all it reaches.
fn postorder(succ: &[Vec<usize>]) -> Vec<usize> {
    let n = succ.len();
    let mut state = vec![0u8; n];
    let mut out = Vec::with_capacity(n);
    for root in 0..n {
        if state[root] != 0 {
            continue;
        }
        let mut stack = vec![(root, 0usize)];
        state[root] = 1;
        while let Some(&mut (v, ref mut k)) = stack.last_mut() {
            if *k < succ[v].len() {
                let w = succ[v][*k];
                *k += 1;
                if state[w] == 0 {
                    state[w] = 1;
                    stack.push((w, 0));
                }
            } else {
                state[v] = 2;
                out.push(v);
                stack.pop();
            }
        }
    }
    out
}

/// Removes the cut marks from every call in `m`.
pub fn strip(m: &mut Module) {
    for g in m.globals.iter_mut() {
        let GlobalKind::Function(f) = &mut g.kind else { continue };
        for inst in f.insts.iter_mut() {
            if let Some(call) = inst.op.call_mut() {
                call.fn_attrs.retain(|a| !matches!(a, Attr::Str { key, .. } if key == CUT));
            }
        }
    }
}
