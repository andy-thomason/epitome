//! The call graph, its strongly connected components (SCCs), and the order
//! for working through them.
//!
//! A node is a function; an edge is a direct call. Calls through pointers
//! are not edges: they cannot be inlined, and a cycle that only closes
//! through a pointer is not one the recursion pass can remove.
//!
//! The SCCs form a DAG. A function's SCC depends on the SCCs of its
//! callees, so working bottom-up means a function is only processed once
//! every callee outside its own SCC is final. Independent SCCs run in
//! parallel; see `par::run_dag`.

use epitome_ir::module::*;

use crate::fx::FxHashMap as HashMap;
use crate::par;

pub struct CallGraph {
    /// The functions, in module order. Node `i` is `nodes[i]`.
    pub nodes: Vec<GlobalId>,
    /// The node of each global, indexed by `GlobalId`.
    pub node_of: Vec<Option<usize>>,
    /// The distinct direct callees of each node, sorted.
    pub callees: Vec<Vec<usize>>,
    /// The direct callee of every call site in each node, repeats included.
    pub sites: Vec<Vec<usize>>,
    /// The number of call instructions that call each node directly.
    pub call_sites: Vec<u32>,
    /// How many of `call_sites` are in functions of the node's own SCC,
    /// itself included.
    pub sites_in_scc: Vec<u32>,
    /// The node's address is used other than as a direct callee, so it may
    /// be called from anywhere.
    pub address_taken: Vec<bool>,
    /// The globals each node's body mentions other than as a direct
    /// callee (an address taken, a variable used), sorted.
    pub refs: Vec<Vec<GlobalId>>,
    /// A `blockaddress` constant names one of the node's blocks.
    pub has_blockaddress: Vec<bool>,
    /// The instruction count of each node.
    pub size: Vec<usize>,
    /// SCCs, each a list of nodes in module order. Callees come before
    /// callers: an SCC appears after every SCC it calls.
    pub sccs: Vec<Vec<usize>>,
    /// The SCC of each node.
    pub scc_of: Vec<usize>,
}

/// What one function body calls and refers to.
struct Scan {
    calls: Vec<GlobalId>,
    refs: Vec<GlobalId>,
    size: usize,
}

impl CallGraph {
    pub fn build(m: &Module, threads: usize) -> CallGraph {
        let mentions = const_mentions(&m.consts);
        let nodes: Vec<GlobalId> = m.functions().map(|(g, _)| g).collect();
        let mut node_of = vec![None; m.globals.len()];
        for (i, &g) in nodes.iter().enumerate() {
            node_of[g.index()] = Some(i);
        }
        let is_fn = |g: GlobalId| node_of[g.index()].is_some();

        let scans = par::map(&nodes, threads, |_, &g| {
            let GlobalKind::Function(f) = &m.global(g).kind else { unreachable!() };
            scan(m, &mentions, f, &is_fn)
        });

        let n = nodes.len();
        let mut callees = vec![Vec::new(); n];
        let mut sites = vec![Vec::new(); n];
        let mut call_sites = vec![0; n];
        let mut address_taken = vec![false; n];
        let mut refs = vec![Vec::new(); n];
        let mut size = vec![0; n];
        let mut take = |g: GlobalId| {
            if let Some(i) = node_of[g.index()] {
                address_taken[i] = true;
            }
        };
        for (i, s) in scans.iter().enumerate() {
            size[i] = s.size;
            for &g in &s.calls {
                let j = node_of[g.index()].unwrap();
                call_sites[j] += 1;
                callees[i].push(j);
                sites[i].push(j);
            }
            callees[i].sort_unstable();
            callees[i].dedup();
            s.refs.iter().for_each(|&g| take(g));
            refs[i] = s.refs.clone();
            refs[i].sort_unstable();
            refs[i].dedup();
        }
        // References from outside function bodies.
        for &g in &m.global_order {
            let refs: Vec<ConstId> = match &m.global(g).kind {
                GlobalKind::Variable(v) => v.init.into_iter().collect(),
                GlobalKind::Alias { aliasee, .. } => vec![*aliasee],
                GlobalKind::IFunc { resolver, .. } => vec![*resolver],
                GlobalKind::Function(f) => [f.prefix, f.prologue, f.personality].into_iter().flatten().collect(),
                GlobalKind::Forward => vec![],
            };
            refs.iter().for_each(|c| mentions[c.index()].iter().for_each(|&g| take(g)));
        }
        for node in &m.metadata {
            let mut values = Vec::new();
            match &node.data {
                Metadata::Value(v) => values.push(*v),
                Metadata::Node { fields, .. } => fields.iter().for_each(|f| {
                    if let MdFieldValue::Value(v) = f.value {
                        values.push(v)
                    }
                }),
                _ => {}
            }
            for v in values {
                if let Value::Const(c) = v {
                    mentions[c.index()].iter().for_each(|&g| take(g));
                }
            }
        }
        let mut has_blockaddress = vec![false; n];
        for i in 0..m.consts.len() {
            if let ConstKind::BlockAddress { func, .. } = m.consts.get(ConstId(i as u32)).kind
                && let Some(j) = node_of[func.index()]
            {
                has_blockaddress[j] = true;
            }
        }

        let sccs = tarjan(&callees);
        let mut scc_of = vec![0; n];
        for (s, members) in sccs.iter().enumerate() {
            for &v in members {
                scc_of[v] = s;
            }
        }
        let mut sites_in_scc = vec![0; n];
        for (i, s) in scans.iter().enumerate() {
            for &g in &s.calls {
                let j = node_of[g.index()].unwrap();
                if scc_of[i] == scc_of[j] {
                    sites_in_scc[j] += 1;
                }
            }
        }
        CallGraph {
            nodes,
            node_of,
            callees,
            sites,
            call_sites,
            sites_in_scc,
            address_taken,
            refs,
            has_blockaddress,
            size,
            sccs,
            scc_of,
        }
    }

    pub fn node(&self, g: GlobalId) -> Option<usize> {
        self.node_of.get(g.index()).copied().flatten()
    }

    /// True if SCC `s` is a cycle: more than one function, or a function
    /// that calls itself.
    pub fn is_recursive(&self, s: usize) -> bool {
        let members = &self.sccs[s];
        members.len() > 1 || self.callees[members[0]].contains(&members[0])
    }

    /// For each SCC, the SCCs it calls, which must be finished first.
    pub fn scc_deps(&self) -> Vec<Vec<usize>> {
        self.sccs
            .iter()
            .enumerate()
            .map(|(s, members)| {
                let mut deps: Vec<usize> = members
                    .iter()
                    .flat_map(|&v| self.callees[v].iter().map(|&c| self.scc_of[c]))
                    .filter(|&d| d != s)
                    .collect();
                deps.sort_unstable();
                deps.dedup();
                deps
            })
            .collect()
    }

    /// A scheduling priority for each SCC: its size plus the largest
    /// priority of any SCC that calls it. This is the length of the longest
    /// chain of work waiting on it, so starting high priorities first
    /// shortens the critical path.
    pub fn scc_priorities(&self, deps: &[Vec<usize>]) -> Vec<u64> {
        let mut prio = vec![0u64; self.sccs.len()];
        // Callers come after callees, so walk backwards to see callers first.
        for s in (0..self.sccs.len()).rev() {
            let own: usize = self.sccs[s].iter().map(|&v| self.size[v].max(1)).sum();
            prio[s] += own as u64;
            for &d in &deps[s] {
                prio[d] = prio[d].max(prio[s]);
            }
        }
        prio
    }

    /// One line per cycle, such as `@even -> @odd -> @even`.
    pub fn describe_cycles(&self, m: &Module) -> Vec<String> {
        let name = |v: usize| match &m.global(self.nodes[v]).name {
            Some(n) => format!("@{n}"),
            None => format!("@{}", self.nodes[v].0),
        };
        let mut lines = Vec::new();
        for (s, members) in self.sccs.iter().enumerate() {
            if !self.is_recursive(s) {
                continue;
            }
            let start = members[0];
            let path = self.cycle_through(start, s);
            let text: Vec<String> = path.iter().map(|&v| name(v)).collect();
            let mut line = text.join(" -> ");
            if members.len() > 2 {
                let others: Vec<String> = members.iter().map(|&v| name(v)).collect();
                line.push_str(&format!("   (SCC of {}: {})", members.len(), others.join(", ")));
            }
            lines.push(line);
        }
        lines
    }

    /// A shortest cycle from `start` back to itself inside SCC `s`.
    fn cycle_through(&self, start: usize, s: usize) -> Vec<usize> {
        if self.callees[start].contains(&start) {
            return vec![start, start];
        }
        let mut prev: HashMap<usize, usize> = HashMap::default();
        let mut queue = std::collections::VecDeque::from([start]);
        while let Some(v) = queue.pop_front() {
            for &c in &self.callees[v] {
                if self.scc_of[c] != s {
                    continue;
                }
                if c == start {
                    // Walk back from `v` to `start`, then put it in call order.
                    let mut path = vec![v];
                    while *path.last().unwrap() != start {
                        path.push(prev[path.last().unwrap()]);
                    }
                    path.reverse();
                    path.push(start);
                    return path;
                }
                if let std::collections::hash_map::Entry::Vacant(e) = prev.entry(c) {
                    e.insert(v);
                    queue.push_back(c);
                }
            }
        }
        vec![start]
    }
}

fn scan(m: &Module, mentions: &[Vec<GlobalId>], f: &Function, is_fn: &dyn Fn(GlobalId) -> bool) -> Scan {
    let mut out = Scan { calls: Vec::new(), refs: Vec::new(), size: 0 };
    let refer = |v: &Value, out: &mut Scan| {
        if let Value::Const(c) = v {
            out.refs.extend_from_slice(&mentions[c.index()]);
        }
    };
    for i in f.insts_in_order() {
        out.size += 1;
        let op = &f.insts[i.index()].op;
        let Some(call) = op.call() else {
            op.for_each_value(|v| refer(v, &mut out));
            continue;
        };
        let direct = match call.callee {
            Value::Const(c) => match m.consts.get(c).kind {
                ConstKind::Global(g) if is_fn(g) => Some(g),
                _ => None,
            },
            _ => None,
        };
        match direct {
            Some(g) => out.calls.push(g),
            None => refer(&call.callee, &mut out),
        }
        call.args.iter().for_each(|a| refer(&a.value, &mut out));
        call.bundles.iter().flat_map(|b| &b.inputs).for_each(|v| refer(v, &mut out));
    }
    out
}

/// For each constant, the globals it mentions, directly or through its
/// operands.
pub(crate) fn const_mentions(consts: &Constants) -> Vec<Vec<GlobalId>> {
    let n = consts.len();
    let mut memo: Vec<Option<Vec<GlobalId>>> = vec![None; n];
    fn visit(consts: &Constants, memo: &mut Vec<Option<Vec<GlobalId>>>, c: ConstId) -> Vec<GlobalId> {
        if let Some(v) = &memo[c.index()] {
            return v.clone();
        }
        // Mark as in progress so a malformed cycle cannot recurse forever.
        memo[c.index()] = Some(Vec::new());
        let (kids, mut globals): (Vec<ConstId>, Vec<GlobalId>) = match &consts.get(c).kind {
            ConstKind::Array(v) | ConstKind::Struct(v) | ConstKind::Vector(v) | ConstKind::PtrAuth(v) => {
                (v.clone(), vec![])
            }
            ConstKind::Splat(x) => (vec![*x], vec![]),
            ConstKind::Global(g) | ConstKind::DsoLocalEquivalent(g) | ConstKind::NoCfi(g) => (vec![], vec![*g]),
            ConstKind::BlockAddress { func, .. } => (vec![], vec![*func]),
            ConstKind::Expr(e) => (
                match e {
                    ConstExpr::Cast { value, .. } => vec![*value],
                    ConstExpr::Gep { ptr, indices, .. } => {
                        std::iter::once(*ptr).chain(indices.iter().copied()).collect()
                    }
                    ConstExpr::Binary { lhs, rhs, .. } => vec![*lhs, *rhs],
                    ConstExpr::ExtractElement { vector, index } => vec![*vector, *index],
                    ConstExpr::InsertElement { vector, element, index } => vec![*vector, *element, *index],
                    ConstExpr::ShuffleVector { a, b, mask } => vec![*a, *b, *mask],
                },
                vec![],
            ),
            _ => (vec![], vec![]),
        };
        for k in kids {
            globals.extend(visit(consts, memo, k));
        }
        globals.sort_unstable();
        globals.dedup();
        memo[c.index()] = Some(globals.clone());
        globals
    }
    for i in 0..n {
        visit(consts, &mut memo, ConstId(i as u32));
    }
    memo.into_iter().map(|v| v.unwrap()).collect()
}

/// Tarjan's algorithm, without recursion. SCCs come out callees first.
pub(crate) fn tarjan(succ: &[Vec<usize>]) -> Vec<Vec<usize>> {
    let n = succ.len();
    let mut index = vec![u32::MAX; n];
    let mut low = vec![0u32; n];
    let mut on_stack = vec![false; n];
    let mut stack = Vec::new();
    let mut sccs = Vec::new();
    let mut next = 0u32;
    for root in 0..n {
        if index[root] != u32::MAX {
            continue;
        }
        let mut calls: Vec<(usize, usize)> = vec![(root, 0)];
        index[root] = next;
        low[root] = next;
        next += 1;
        stack.push(root);
        on_stack[root] = true;
        while let Some(&mut (v, ref mut edge)) = calls.last_mut() {
            if let Some(&w) = succ[v].get(*edge) {
                *edge += 1;
                if index[w] == u32::MAX {
                    index[w] = next;
                    low[w] = next;
                    next += 1;
                    stack.push(w);
                    on_stack[w] = true;
                    calls.push((w, 0));
                } else if on_stack[w] {
                    low[v] = low[v].min(index[w]);
                }
                continue;
            }
            calls.pop();
            if let Some(&(parent, _)) = calls.last() {
                low[parent] = low[parent].min(low[v]);
            }
            if low[v] == index[v] {
                let mut scc = Vec::new();
                loop {
                    let w = stack.pop().unwrap();
                    on_stack[w] = false;
                    scc.push(w);
                    if w == v {
                        break;
                    }
                }
                scc.sort_unstable();
                sccs.push(scc);
            }
        }
    }
    sccs
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tarjan_finds_cycles_callees_first() {
        // 0 -> 1 -> 2 -> 1, 0 -> 3
        let succ = vec![vec![1, 3], vec![2], vec![1], vec![]];
        let sccs = tarjan(&succ);
        assert_eq!(sccs, vec![vec![1, 2], vec![3], vec![0]]);
    }
}
