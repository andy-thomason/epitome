//! A clocked step's code ([`Steps`]) cut into tasks for threads, and how
//! fast those tasks would run: the plan's second step
//! (`docs/design/sv-threads.md`), with no runtime yet.
//!
//! Each piece of code is costed, from a profile where there is one; the
//! calls are cut top down against a budget, a call whose cost fits being
//! one task; a task waits on each earlier task whose bytes of the state it
//! reads or writes, or that reads what it writes; and a list scheduler,
//! simulated, says how long the tasks take on so many threads.

use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};
use std::ops::Range;

use crate::model::Profiled;
use crate::netlist::Steps;

/// Each piece's cost. With a profile ([`crate::model::Timed::profile`]),
/// each call's own cycles a call (its counter's, less the calls within it
/// that have counters), spread over the pieces it runs itself, and those
/// of its calls without counters, by their instructions; without one, the
/// instructions.
pub fn costs(steps: &Steps, profile: &[Profiled]) -> Vec<f64> {
    let insts = |k: usize| steps.items[k].insts as f64;
    if profile.is_empty() {
        return (0..steps.items.len()).map(insts).collect();
    }
    let per_call: HashMap<(&str, &str), f64> =
        profile.iter().filter(|p| p.calls > 0.0).map(|p| ((p.counter.path.as_str(), p.counter.func.as_str()), p.ticks / p.calls)).collect();
    let calls = &steps.calls;
    let counted: Vec<Option<f64>> = calls.iter().map(|c| per_call.get(&(c.path.as_str(), c.func.as_str())).copied()).collect();
    // Each call's nearest call, itself or above it, with a counter.
    let mut owner: Vec<Option<usize>> = vec![None; calls.len()];
    for (k, c) in calls.iter().enumerate() {
        owner[k] = if counted[k].is_some() { Some(k) } else { c.parent.and_then(|p| owner[p]) };
    }
    // Each counted call's own: less the counted calls just below it.
    let mut own: Vec<f64> = counted.iter().map(|c| c.unwrap_or(0.0)).collect();
    for (k, c) in calls.iter().enumerate() {
        if let (Some(t), Some(p)) = (counted[k], c.parent.and_then(|p| owner[p])) {
            own[p] -= t;
        }
    }
    // Spread by instructions over the pieces each owns.
    let mut owned = vec![0.0; calls.len()];
    for it in &steps.items {
        if let Some(o) = owner[it.call] {
            owned[o] += it.insts as f64;
        }
    }
    (0..steps.items.len())
        .map(|k| match owner[steps.items[k].call] {
            Some(o) if owned[o] > 0.0 => own[o].max(0.0) * insts(k) / owned[o],
            _ => 0.0,
        })
        .collect()
}

/// Tasks and what each waits on.
#[derive(Clone, Debug, Default)]
pub struct Graph {
    /// Each task's pieces, runs of [`Steps::items`], in order.
    pub tasks: Vec<Vec<Range<usize>>>,
    pub cost: Vec<f64>,
    /// The earlier tasks each waits on.
    pub deps: Vec<Vec<u32>>,
}

impl Graph {
    pub fn edges(&self) -> usize {
        self.deps.iter().map(|d| d.len()).sum()
    }

    /// Whether every task waits only on tasks before it, and every piece
    /// is in exactly one task.
    pub fn well_formed(&self, pieces: usize) -> bool {
        let mut seen = vec![false; pieces];
        for r in self.tasks.iter().flatten() {
            for k in r.clone() {
                if std::mem::replace(&mut seen[k], true) {
                    return false;
                }
            }
        }
        seen.into_iter().all(|x| x) && self.deps.iter().enumerate().all(|(t, d)| d.iter().all(|&u| (u as usize) < t))
    }
}

/// The step's calls cut into tasks: a call that costs at most `budget`
/// is one task, a dearer one its own pieces one a task and its calls cut
/// in turn. Then each task cheaper than `min` joins one it waits on, if
/// together they cost at most `min`: the one that starts last, so long as
/// every other it waits on ends before that one starts (tasks wait only on
/// tasks before them, so none of those can come to wait on the merged
/// one); with nothing to wait on, it joins the last such task that waits on
/// nothing.
pub fn cut(steps: &Steps, costs: &[f64], budget: f64, min: f64) -> Graph {
    let calls = &steps.calls;
    let mut kids: Vec<Vec<usize>> = vec![Vec::new(); calls.len()];
    for (k, c) in calls.iter().enumerate() {
        if let Some(p) = c.parent {
            kids[p].push(k);
        }
    }
    let cost_of = |r: &Range<usize>| costs[r.clone()].iter().sum::<f64>();
    let mut runs: Vec<Range<usize>> = Vec::new();
    fn open(k: usize, steps: &Steps, kids: &[Vec<usize>], budget: f64, cost_of: &dyn Fn(&Range<usize>) -> f64, runs: &mut Vec<Range<usize>>) {
        let c = &steps.calls[k];
        if cost_of(&c.items) <= budget || c.items.len() <= 1 {
            if !c.items.is_empty() {
                runs.push(c.items.clone());
            }
            return;
        }
        let mut at = c.items.start;
        let mut next = kids[k].iter().peekable();
        while at < c.items.end {
            match next.peek() {
                Some(&&kid) if steps.calls[kid].items.start == at => {
                    next.next();
                    open(kid, steps, kids, budget, cost_of, runs);
                    at = steps.calls[kid].items.end.max(at + usize::from(steps.calls[kid].items.is_empty()));
                }
                _ => {
                    runs.push(at..at + 1);
                    at += 1;
                }
            }
        }
    }
    for k in (0..calls.len()).filter(|&k| calls[k].parent.is_none()) {
        open(k, steps, &kids, budget, &cost_of, &mut runs);
    }
    let fine: Vec<Vec<Range<usize>>> = runs.into_iter().map(|r| vec![r]).collect();
    let deps0 = deps(steps, &fine);
    // Each fine task's cluster, by the cluster's first task.
    let n = fine.len();
    let mut cluster: Vec<usize> = (0..n).collect();
    let mut total: Vec<f64> = fine.iter().map(|t| cost_of(&t[0])).collect();
    // (A cluster's first task is its own index; its last, `last`.)
    let mut last: Vec<usize> = (0..n).collect();
    let mut free: Option<usize> = None;
    for t in 0..n {
        let c = total[t];
        if c >= min {
            continue;
        }
        let target = match deps0[t].iter().map(|&u| cluster[u as usize]).max() {
            Some(u) => deps0[t].iter().map(|&v| cluster[v as usize]).all(|v| v == u || last[v] < u).then_some(u),
            None => {
                let f = free.filter(|&f| total[f] + c <= min);
                if f.is_none() {
                    free = Some(t);
                }
                f
            }
        };
        if let Some(u) = target
            && total[u] + c <= min
        {
            cluster[t] = u;
            total[u] += c;
            last[u] = t;
        }
    }
    // The clusters, and what each waits on: what its tasks wait on.
    let mut index = vec![usize::MAX; n];
    let mut runs: Vec<Vec<Range<usize>>> = Vec::new();
    for t in 0..n {
        let c = cluster[t];
        if index[c] == usize::MAX {
            index[c] = runs.len();
            runs.push(Vec::new());
        }
        let rs = &mut runs[index[c]];
        let r = fine[t][0].clone();
        match rs.last_mut() {
            Some(l) if l.end == r.start => l.end = r.end,
            _ => rs.push(r),
        }
    }
    let m = runs.len();
    let mut waits: Vec<Vec<u32>> = vec![Vec::new(); m];
    for t in 0..n {
        let c = index[cluster[t]];
        for &u in &deps0[t] {
            let d = index[cluster[u as usize]];
            if d != c {
                waits[c].push(d as u32);
            }
        }
    }
    for w in &mut waits {
        w.sort_unstable();
        w.dedup();
    }
    // In an order where each comes after all it waits on.
    let mut succs: Vec<Vec<u32>> = vec![Vec::new(); m];
    let mut left: Vec<usize> = waits.iter().map(|w| w.len()).collect();
    for (c, w) in waits.iter().enumerate() {
        for &d in w {
            succs[d as usize].push(c as u32);
        }
    }
    let mut ready: std::collections::BTreeSet<usize> = (0..m).filter(|&c| left[c] == 0).collect();
    let mut order = Vec::with_capacity(m);
    while let Some(c) = ready.pop_first() {
        order.push(c);
        for &s in &succs[c] {
            left[s as usize] -= 1;
            if left[s as usize] == 0 {
                ready.insert(s as usize);
            }
        }
    }
    assert_eq!(order.len(), m, "tasks that wait on each other");
    let mut at = vec![0u32; m];
    for (k, &c) in order.iter().enumerate() {
        at[c] = k as u32;
    }
    let tasks: Vec<Vec<Range<usize>>> = order.iter().map(|&c| runs[c].clone()).collect();
    let deps: Vec<Vec<u32>> = order
        .iter()
        .map(|&c| {
            let mut d: Vec<u32> = waits[c].iter().map(|&x| at[x as usize]).collect();
            d.sort_unstable();
            d
        })
        .collect();
    let cost: Vec<f64> = tasks.iter().map(|t| t.iter().map(cost_of).sum()).collect();
    Graph { tasks, cost, deps }
}

/// What each task waits on: the last earlier task to write each byte it
/// reads or writes, and the earlier tasks that read a byte it writes since
/// it was last written.
fn deps(steps: &Steps, tasks: &[Vec<Range<usize>>]) -> Vec<Vec<u32>> {
    // The state cut into runs no range starts or ends inside.
    let mut cuts: Vec<u64> = Vec::new();
    for it in &steps.items {
        for &(a, n) in it.reads.iter().chain(&it.writes) {
            cuts.push(a);
            cuts.push(a + n);
        }
    }
    cuts.sort_unstable();
    cuts.dedup();
    let atoms = |&(a, n): &(u64, u64)| cuts.partition_point(|&c| c < a)..cuts.partition_point(|&c| c < a + n);
    let mut writer: Vec<u32> = vec![u32::MAX; cuts.len()];
    let mut readers: Vec<Vec<u32>> = vec![Vec::new(); cuts.len()];
    let mut seen: Vec<u32> = vec![u32::MAX; tasks.len()];
    let mut out = Vec::with_capacity(tasks.len());
    for (t, r) in tasks.iter().enumerate() {
        let t = t as u32;
        let mut d = Vec::new();
        let mut wait = |u: u32, d: &mut Vec<u32>| {
            if u != t && u != u32::MAX && seen[u as usize] != t {
                seen[u as usize] = t;
                d.push(u);
            }
        };
        for it in r.iter().flat_map(|r| &steps.items[r.clone()]) {
            for rr in &it.reads {
                for a in atoms(rr) {
                    wait(writer[a], &mut d);
                }
            }
            for ww in &it.writes {
                for a in atoms(ww) {
                    wait(writer[a], &mut d);
                    for &u in &readers[a] {
                        wait(u, &mut d);
                    }
                }
            }
        }
        for it in r.iter().flat_map(|r| &steps.items[r.clone()]) {
            for rr in &it.reads {
                for a in atoms(rr) {
                    if readers[a].last() != Some(&t) {
                        readers[a].push(t);
                    }
                }
            }
        }
        for it in r.iter().flat_map(|r| &steps.items[r.clone()]) {
            for ww in &it.writes {
                for a in atoms(ww) {
                    writer[a] = t;
                    readers[a].clear();
                }
            }
        }
        d.sort_unstable();
        out.push(d);
    }
    out
}

/// The longest path through the tasks, each costing `overhead` more.
pub fn span(g: &Graph, overhead: f64) -> f64 {
    let mut finish = vec![0.0f64; g.tasks.len()];
    for t in 0..g.tasks.len() {
        let start = g.deps[t].iter().map(|&u| finish[u as usize]).fold(0.0, f64::max);
        finish[t] = start + g.cost[t] + overhead;
    }
    finish.into_iter().fold(0.0, f64::max)
}

/// The tasks on the longest path, first to last.
pub fn longest_path(g: &Graph, overhead: f64) -> Vec<usize> {
    let n = g.tasks.len();
    let mut finish = vec![0.0f64; n];
    let mut from = vec![usize::MAX; n];
    for t in 0..n {
        for &u in &g.deps[t] {
            if finish[u as usize] > finish[t] {
                (finish[t], from[t]) = (finish[u as usize], u as usize);
            }
        }
        finish[t] += g.cost[t] + overhead;
    }
    let mut t = (0..n).max_by(|&a, &b| finish[a].total_cmp(&finish[b]));
    let mut path = Vec::new();
    while let Some(k) = t {
        path.push(k);
        t = (from[k] != usize::MAX).then_some(from[k]);
    }
    path.reverse();
    path
}

/// How long the tasks take on `threads` threads, each costing `overhead`
/// more: a list scheduler that, whenever a thread is free, starts the
/// ready task with the longest path after it.
pub fn makespan(g: &Graph, threads: usize, overhead: f64) -> f64 {
    let n = g.tasks.len();
    let mut succs: Vec<Vec<u32>> = vec![Vec::new(); n];
    let mut waits: Vec<usize> = vec![0; n];
    for (t, d) in g.deps.iter().enumerate() {
        waits[t] = d.len();
        for &u in d {
            succs[u as usize].push(t as u32);
        }
    }
    let mut level = vec![0.0f64; n];
    for t in (0..n).rev() {
        level[t] = g.cost[t] + overhead + succs[t].iter().map(|&s| level[s as usize]).fold(0.0, f64::max);
    }
    #[derive(PartialEq)]
    struct By(f64, u32);
    impl Eq for By {}
    impl PartialOrd for By {
        fn partial_cmp(&self, o: &Self) -> Option<Ordering> {
            Some(self.cmp(o))
        }
    }
    impl Ord for By {
        fn cmp(&self, o: &Self) -> Ordering {
            self.0.total_cmp(&o.0).then(o.1.cmp(&self.1))
        }
    }
    let mut ready: BinaryHeap<By> = (0..n).filter(|&t| waits[t] == 0).map(|t| By(level[t], t as u32)).collect();
    // (Running tasks by when they end, soonest first.)
    let mut running: BinaryHeap<By> = BinaryHeap::new();
    let (mut now, mut idle) = (0.0f64, threads.max(1));
    loop {
        while idle > 0
            && let Some(By(_, t)) = ready.pop()
        {
            idle -= 1;
            running.push(By(-(now + g.cost[t as usize] + overhead), t));
        }
        let Some(By(end, t)) = running.pop() else { break };
        now = -end;
        idle += 1;
        for &s in &succs[t as usize] {
            waits[s as usize] -= 1;
            if waits[s as usize] == 0 {
                ready.push(By(level[s as usize], s));
            }
        }
    }
    now
}

/// The step cut for so many threads, and how it would run.
#[derive(Clone, Debug)]
pub struct Plan {
    pub threads: usize,
    pub tasks: usize,
    pub edges: usize,
    /// The longest path, and the time on `threads` threads.
    pub span: f64,
    pub time: f64,
}

/// For each of `threads`: the step cut for `per` tasks a thread (a budget
/// of the step's work over `per * threads`, pieces next to each other
/// merged up to the budget, or up to `floor` where that is more), with
/// each task's dispatch costing `overhead`. Also the step's whole work,
/// the time on one thread with no overhead.
pub fn plans(steps: &Steps, costs: &[f64], threads: &[usize], per: f64, floor: f64, overhead: f64) -> (f64, Vec<Plan>) {
    let work: f64 = costs.iter().sum();
    let plans = threads
        .iter()
        .map(|&n| {
            let budget = work / (per * n as f64);
            let g = cut(steps, costs, budget, budget.max(floor));
            Plan { threads: n, tasks: g.tasks.len(), edges: g.edges(), span: span(&g, overhead), time: makespan(&g, n, overhead) }
        })
        .collect();
    (work, plans)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::netlist::{Invocation, Piece};

    fn piece(call: usize, insts: usize, reads: &[(u64, u64)], writes: &[(u64, u64)]) -> Piece {
        Piece { call, what: "settle", name: String::new(), insts, reads: reads.to_vec(), writes: writes.to_vec() }
    }

    /// A top calling `n` children, each one piece; `chain` makes each read
    /// what the one before wrote.
    fn fan(n: usize, chain: bool) -> Steps {
        let mut s = Steps::default();
        s.calls.push(Invocation { path: String::new(), spec: "top".into(), func: "step settle".into(), parent: None, items: 0..n });
        for k in 0..n {
            let w = (100 * k as u64 + 100, 8);
            let r = if chain && k > 0 { vec![(100 * k as u64, 8)] } else { vec![(0, 8)] };
            s.calls.push(Invocation { path: format!("u{k}"), spec: "kid".into(), func: "settle".into(), parent: Some(0), items: k..k + 1 });
            s.items.push(piece(k + 1, 10, &r, &[w]));
        }
        s
    }

    #[test]
    fn independent_children_run_side_by_side() {
        let s = fan(8, false);
        let c = costs(&s, &[]);
        let g = cut(&s, &c, 20.0, 0.0);
        assert!(g.well_formed(8));
        assert_eq!(g.tasks.len(), 8);
        assert_eq!(g.edges(), 0);
        assert_eq!(makespan(&g, 4, 0.0), 20.0);
        assert_eq!(makespan(&g, 8, 0.0), 10.0);
        assert_eq!(span(&g, 0.0), 10.0);
    }

    #[test]
    fn a_chain_waits() {
        let s = fan(8, true);
        let c = costs(&s, &[]);
        let g = cut(&s, &c, 20.0, 0.0);
        assert_eq!(g.edges(), 7);
        assert_eq!(longest_path(&g, 0.0), (0..8).collect::<Vec<_>>());
        assert_eq!(makespan(&g, 8, 0.0), 80.0);
    }

    #[test]
    fn readers_hold_back_a_writer() {
        // Two readers of byte 0, then a writer of it: it waits on both.
        let mut s = Steps::default();
        s.calls.push(Invocation { path: String::new(), spec: "top".into(), func: "step settle".into(), parent: None, items: 0..3 });
        s.items.push(piece(0, 1, &[(0, 4)], &[(8, 4)]));
        s.items.push(piece(0, 1, &[(2, 1)], &[(16, 4)]));
        s.items.push(piece(0, 1, &[], &[(0, 8)]));
        let g = cut(&s, &costs(&s, &[]), 1.0, 0.0);
        assert_eq!(g.deps, vec![vec![], vec![], vec![0, 1]]);
    }

    #[test]
    fn a_cheap_call_is_one_task_and_small_tasks_merge() {
        let s = fan(8, false);
        let c = costs(&s, &[]);
        assert_eq!(cut(&s, &c, 1000.0, 0.0).tasks, vec![vec![0..8]]);
        // (Eight that wait on nothing: merged four at a time.)
        assert_eq!(cut(&s, &c, 20.0, 40.0).tasks.len(), 2);
        // (A chain: each joins the one it waits on.)
        let g = cut(&fan(8, true), &c, 20.0, 40.0);
        assert!(g.well_formed(8));
        assert_eq!(g.tasks.len(), 2);
    }
}
