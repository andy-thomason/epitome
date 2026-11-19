//! What dominating branches imply: comparisons known on a path because a
//! branch on the way to it established them.
//!
//! A block reached only along the `true` edge of `br i1 %c, ...` knows
//! `%c` is true, and if `%c = icmp ult %i, %n`, that `%i < %n`. Every
//! block the edge's target dominates knows the same. This pass walks the
//! dominator tree with the facts in force at each block and uses them to:
//!
//! - fold comparisons the facts decide: a slice bounds check
//!   `icmp ult %i, %len` under a loop test `%i < %len`, or
//!   `%i < min(%a.len, %b.len)`, or `%i < %len / 2`;
//! - pick the arm of a `select` whose condition is known where it is used
//!   (front ends build `Option` that way: `select %c, %i, undef`);
//! - mark `add` and `sub` `nuw` when the facts show they cannot wrap.
//!
//! The facts are comparisons `a rel b` with `rel` one of `<`, `<=`
//! (unsigned and signed), `==` and `!=`. A query `a < b` is proved by a
//! small search with a depth and step limit: transitivity through the facts
//! (`a < y` and `y <= b`), and the shape of the operands (`x / 2 <= x`,
//! `min(x, y) <= x`, `x - y <= x` when `y <= x`, `x - 1 < x` when
//! `x != 0`). When a fact is added, the bounds its right-hand side implies
//! are added with it, so `i < len / 2` also gives `i < len`.
//!
//! It does not know facts that hold round a loop rather than down the
//! dominator tree (an induction variable stays below its bound); it runs
//! before rotation, while a loop's test still dominates its body. See
//! `docs/design/ranges.md`.

use epitome_ir::module::*;
use epitome_ir::types::Type;

use crate::cfg::DomTree;
use crate::combine::eval;
use crate::ctx::Shared;
use crate::fx::FxHashMap as HashMap;

#[derive(Default, Debug, Clone, Copy)]
pub struct Stats {
    /// Comparisons folded to a constant.
    pub folded: usize,
    /// Uses of a `select` replaced by the arm its known condition picks.
    pub selects: usize,
    /// `add`s and `sub`s marked `nuw`.
    pub flags: usize,
}

impl Stats {
    pub fn changed(&self) -> bool {
        self.folded + self.selects + self.flags > 0
    }
}

/// How deep a proof may go, and how many steps one question may take.
const DEPTH: u32 = 6;
const STEPS: u32 = 96;
/// Bounds added with a fact, through the shape of its right-hand side.
const DERIVED: usize = 6;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Rel {
    Ult,
    Ule,
    Slt,
    Sle,
    Eq,
    Ne,
}

/// `a rel b`.
#[derive(Clone, Copy, Debug)]
struct Fact {
    rel: Rel,
    a: Value,
    b: Value,
}

/// A comparison as a relation, with `>` and `>=` turned round.
fn relation(pred: ICmpPred, a: Value, b: Value) -> Fact {
    use ICmpPred::*;
    let (rel, a, b) = match pred {
        Eq => (Rel::Eq, a, b),
        Ne => (Rel::Ne, a, b),
        Ult => (Rel::Ult, a, b),
        Ule => (Rel::Ule, a, b),
        Ugt => (Rel::Ult, b, a),
        Uge => (Rel::Ule, b, a),
        Slt => (Rel::Slt, a, b),
        Sle => (Rel::Sle, a, b),
        Sgt => (Rel::Slt, b, a),
        Sge => (Rel::Sle, b, a),
    };
    Fact { rel, a, b }
}

/// The relation that holds when `f` does not.
fn negation(f: Fact) -> Fact {
    let (rel, a, b) = match f.rel {
        Rel::Ult => (Rel::Ule, f.b, f.a),
        Rel::Ule => (Rel::Ult, f.b, f.a),
        Rel::Slt => (Rel::Sle, f.b, f.a),
        Rel::Sle => (Rel::Slt, f.b, f.a),
        Rel::Eq => (Rel::Ne, f.a, f.b),
        Rel::Ne => (Rel::Eq, f.a, f.b),
    };
    Fact { rel, a, b }
}

pub fn run(sh: &Shared, f: &mut Function) -> Stats {
    let mut stats = Stats::default();
    if !f.has_body || f.blocks.is_empty() {
        return stats;
    }
    let edits = {
        let mut w = Walker::new(sh, f);
        w.walk();
        w.edits
    };
    // Uses of selects first: they name instructions by id, which folding
    // comparisons does not change.
    for (i, from, to) in edits.uses {
        f.insts[i.index()].op.for_each_value_mut(|v| {
            if *v == from {
                *v = to;
            }
        });
        stats.selects += 1;
    }
    for (phi, pred, to) in edits.phi_uses {
        if let Op::Phi { incoming, .. } = &mut f.insts[phi.index()].op {
            for (v, b) in incoming.iter_mut() {
                if *b == pred {
                    *v = to;
                }
            }
        }
        stats.selects += 1;
    }
    for i in edits.nuw {
        if let Op::Binary { flags, .. } = &mut f.insts[i.index()].op {
            *flags = *flags | IntFlags::NUW;
            stats.flags += 1;
        }
    }
    if !edits.folded.is_empty() {
        stats.folded = edits.folded.len();
        let map: std::collections::HashMap<LocalId, Value> = edits.folded.into_iter().collect();
        f.replace_many(&map);
    }
    stats
}

#[derive(Default)]
struct Edits {
    /// In instruction `i`, uses of the first value become the second.
    uses: Vec<(InstId, Value, Value)>,
    /// The entry of phi `.0` from block `.1` becomes `.2`.
    phi_uses: Vec<(InstId, BlockId, Value)>,
    /// Comparisons with a known result.
    folded: Vec<(LocalId, Value)>,
    nuw: Vec<InstId>,
}

struct Walker<'a, 'm> {
    sh: &'a Shared<'m>,
    f: &'a Function,
    dom: DomTree,
    preds: Vec<Vec<BlockId>>,
    /// Every fact in force at the current block, oldest first.
    facts: Vec<Fact>,
    /// For each value, the facts (by index) that mention it.
    index: HashMap<Value, Vec<u32>>,
    /// Facts assumed for the question being asked (a select's arm).
    assumed: Vec<Fact>,
    steps: u32,
    edits: Edits,
    t: Value,
    fl: Value,
}

impl<'a, 'm> Walker<'a, 'm> {
    fn new(sh: &'a Shared<'m>, f: &'a Function) -> Self {
        Walker {
            sh,
            f,
            dom: DomTree::new(f),
            preds: f.predecessors(),
            facts: Vec::new(),
            index: HashMap::default(),
            assumed: Vec::new(),
            steps: 0,
            edits: Edits::default(),
            t: sh.int(1, 1),
            fl: sh.int(1, 0),
        }
    }

    fn walk(&mut self) {
        let children = self.dom.children();
        let entry = self.f.entry();
        // (block, facts in force before its own, next child to visit)
        let mut stack: Vec<(BlockId, usize, usize)> = Vec::new();
        let mark = self.facts.len();
        self.visit(entry);
        stack.push((entry, mark, 0));
        while let Some(&mut (b, _, ref mut next)) = stack.last_mut() {
            if let Some(&c) = children[b.index()].get(*next) {
                *next += 1;
                let mark = self.facts.len();
                self.enter(b, c);
                self.visit(c);
                stack.push((c, mark, 0));
            } else {
                let (_, mark, _) = stack.pop().unwrap();
                self.truncate(mark);
            }
        }
    }

    // ---------------------------------------------------------------------
    // Facts

    /// Adds the facts of the edge from `p` into its dominator-tree child
    /// `c`, if `c` is reached only along that edge.
    fn enter(&mut self, p: BlockId, c: BlockId) {
        if self.preds[c.index()].as_slice() != [p] {
            return;
        }
        let mut new = Vec::new();
        self.edge_facts(p, c, &mut new);
        for fact in new {
            self.add(fact);
        }
    }

    /// What holds on the edge from `p` to `c`: `p`'s branch condition, if
    /// the edge is one of its two different targets; for a switch case
    /// reached by only one value, the value.
    fn edge_facts(&self, p: BlockId, c: BlockId, out: &mut Vec<Fact>) {
        let Some(t) = self.f.terminator(p) else { return };
        match &self.f.insts[t.index()].op {
            Op::CondBr { cond, then_dest, else_dest } if then_dest != else_dest => {
                self.condition(*cond, c == *then_dest, out, 3);
            }
            Op::Switch { value, default, cases } if *default != c => {
                let mut hit = cases.iter().filter(|(_, b)| *b == c);
                if let (Some((k, _)), None) = (hit.next(), hit.next()) {
                    out.push(Fact { rel: Rel::Eq, a: *value, b: Value::Const(*k) });
                }
            }
            _ => {}
        }
    }

    /// The facts that hold when `cond` is `value`.
    fn condition(&self, cond: Value, value: bool, out: &mut Vec<Fact>, depth: u32) {
        out.push(Fact { rel: Rel::Eq, a: cond, b: if value { self.t } else { self.fl } });
        if depth == 0 {
            return;
        }
        let Some(op) = self.op(cond) else { return };
        match *op {
            Op::ICmp { pred, lhs, rhs, .. } if self.is_int(lhs) => {
                let fact = relation(pred, lhs, rhs);
                out.push(if value { fact } else { negation(fact) });
            }
            // Both sides of an `and` hold when it does; neither side of an
            // `or` when it does not.
            Op::Binary { op: BinOp::And, lhs, rhs, .. } if value => {
                self.condition(lhs, true, out, depth - 1);
                self.condition(rhs, true, out, depth - 1);
            }
            Op::Binary { op: BinOp::Or, lhs, rhs, .. } if !value => {
                self.condition(lhs, false, out, depth - 1);
                self.condition(rhs, false, out, depth - 1);
            }
            Op::Binary { op: BinOp::Xor, lhs, rhs, .. } if rhs == self.t => {
                self.condition(lhs, !value, out, depth - 1);
            }
            // `select a, b, false` is `a && b`; `select a, true, b` is `a || b`.
            Op::Select { cond: a, then_value, else_value, .. } if value && else_value == self.fl => {
                self.condition(a, true, out, depth - 1);
                self.condition(then_value, true, out, depth - 1);
            }
            Op::Select { cond: a, then_value, else_value, .. } if !value && then_value == self.t => {
                self.condition(a, false, out, depth - 1);
                self.condition(else_value, false, out, depth - 1);
            }
            _ => {}
        }
    }

    /// Adds a fact, and the bounds it implies through the shape of its
    /// right-hand side: `a < len / 2` gives `a < len`, `a < min(x, y)`
    /// gives `a < x` and `a < y`.
    fn add(&mut self, fact: Fact) {
        self.push(fact);
        if !matches!(fact.rel, Rel::Ult | Rel::Ule) {
            return;
        }
        let mut work = vec![fact.b];
        let mut added = 0;
        while let Some(b) = work.pop() {
            for up in self.upper_bounds(b) {
                if added == DERIVED {
                    return;
                }
                added += 1;
                self.push(Fact { rel: fact.rel, a: fact.a, b: up });
                work.push(up);
            }
        }
    }

    fn push(&mut self, fact: Fact) {
        let k = self.facts.len() as u32;
        self.facts.push(fact);
        self.index.entry(fact.a).or_default().push(k);
        if fact.b != fact.a {
            self.index.entry(fact.b).or_default().push(k);
        }
    }

    fn truncate(&mut self, mark: usize) {
        while self.facts.len() > mark {
            let fact = self.facts.pop().unwrap();
            for v in [fact.a, fact.b] {
                if let Some(list) = self.index.get_mut(&v) {
                    if list.last() == Some(&(self.facts.len() as u32)) {
                        list.pop();
                    }
                    if list.is_empty() {
                        self.index.remove(&v);
                    }
                }
            }
        }
    }

    /// Values `w` with `v <= w` (unsigned) by `v`'s shape alone.
    fn upper_bounds(&self, v: Value) -> Vec<Value> {
        let Some(op) = self.op(v) else { return vec![] };
        match *op {
            Op::Binary { op: BinOp::LShr | BinOp::UDiv | BinOp::URem, lhs, .. } => vec![lhs],
            Op::Binary { op: BinOp::And, lhs, rhs, .. } => vec![lhs, rhs],
            Op::Binary { op: BinOp::Sub, flags, lhs, .. } if flags.contains(IntFlags::NUW) => vec![lhs],
            _ => match self.min_max(v) {
                Some((false, x, y)) => vec![x, y],
                _ => vec![],
            },
        }
    }

    /// `v` as an unsigned `min` (false) or `max` (true) of two values: a
    /// call to `llvm.umin`/`llvm.umax`, or a select of the two picking by
    /// comparing them.
    fn min_max(&self, v: Value) -> Option<(bool, Value, Value)> {
        match self.op(v)? {
            Op::Select { cond, then_value, else_value, .. } => {
                let Some(&Op::ICmp { pred, lhs, rhs, .. }) = self.op(*cond) else { return None };
                let (x, y) = (*then_value, *else_value);
                let picks_lhs = if (lhs, rhs) == (x, y) {
                    true
                } else if (lhs, rhs) == (y, x) {
                    false
                } else {
                    return None;
                };
                // `select (x < y), x, y` is min; `select (y < x), x, y` is max.
                match (pred, picks_lhs) {
                    (ICmpPred::Ult | ICmpPred::Ule, true) | (ICmpPred::Ugt | ICmpPred::Uge, false) => Some((false, x, y)),
                    (ICmpPred::Ugt | ICmpPred::Uge, true) | (ICmpPred::Ult | ICmpPred::Ule, false) => Some((true, x, y)),
                    _ => None,
                }
            }
            Op::Call(call) if call.args.len() == 2 => {
                let g = self.sh.direct_callee(call)?;
                let name = self.sh.name(g);
                let max = if name.starts_with("llvm.umin.") {
                    false
                } else if name.starts_with("llvm.umax.") {
                    true
                } else {
                    return None;
                };
                Some((max, call.args[0].value, call.args[1].value))
            }
            _ => None,
        }
    }

    // ---------------------------------------------------------------------
    // Rewriting

    fn visit(&mut self, b: BlockId) {
        let f = self.f;
        for &i in &f.blocks[b.index()].insts {
            let op = &f.insts[i.index()].op;
            if op.is_phi() {
                continue;
            }
            // Selects whose condition is known here.
            let mut operands = Vec::new();
            op.for_each_value(|v| operands.push(*v));
            for v in operands {
                if let Some(to) = self.select_arm(v) {
                    self.edits.uses.push((i, v, to));
                } else if matches!(v, Value::Local(_))
                    && let Some(k) = self.fact_bool(v)
                {
                    // A condition tested again below its own branch.
                    self.edits.uses.push((i, v, if k { self.t } else { self.fl }));
                }
            }
            match *op {
                Op::ICmp { pred, lhs, rhs, .. } if self.is_int(lhs) => {
                    let fact = relation(pred, lhs, rhs);
                    let known = if self.prove(fact) {
                        Some(self.t)
                    } else if self.prove(negation(fact)) {
                        Some(self.fl)
                    } else {
                        None
                    };
                    if let (Some(k), Some(r)) = (known, f.insts[i.index()].result) {
                        self.edits.folded.push((r, k));
                    }
                }
                Op::Binary { op: BinOp::Add, flags, lhs, rhs, .. }
                    if !flags.contains(IntFlags::NUW) && self.is_int(lhs) =>
                {
                    // `x + 1` cannot wrap when something is above `x`.
                    let (x, k) = match (self.int(lhs), self.int(rhs)) {
                        (None, Some(k)) => (lhs, k),
                        (Some(k), None) => (rhs, k),
                        _ => continue,
                    };
                    if k.value == 1 && self.prove(Fact { rel: Rel::Ult, a: x, b: self.all_ones(k.bits) }) {
                        self.edits.nuw.push(i);
                    }
                }
                Op::Binary { op: BinOp::Sub, flags, lhs, rhs, .. }
                    if !flags.contains(IntFlags::NUW)
                        && self.is_int(lhs)
                        && self.int(rhs).is_none()
                        && self.prove(Fact { rel: Rel::Ule, a: rhs, b: lhs }) =>
                {
                    self.edits.nuw.push(i);
                }
                _ => {}
            }
        }
        // Phi entries from this block, with what the edge adds.
        for s in f.successors(b) {
            let phis: Vec<InstId> =
                f.blocks[s.index()].insts.iter().copied().take_while(|&i| f.insts[i.index()].op.is_phi()).collect();
            if phis.is_empty() {
                continue;
            }
            let mut edge = Vec::new();
            if f.successors(b).iter().filter(|&&x| x == s).count() == 1 {
                self.edge_facts(b, s, &mut edge);
            }
            for p in phis {
                let Op::Phi { incoming, .. } = &f.insts[p.index()].op else { continue };
                for &(v, from) in incoming {
                    if from != b {
                        continue;
                    }
                    self.assumed.clone_from(&edge);
                    if let Some(to) = self.select_arm(v) {
                        self.edits.phi_uses.push((p, b, to));
                    }
                    self.assumed.clear();
                }
            }
        }
    }

    /// If `v` is a select whose condition is known here, the arm it picks.
    fn select_arm(&mut self, v: Value) -> Option<Value> {
        let &Op::Select { cond, then_value, else_value, .. } = self.op(v)? else { return None };
        match self.known_bool(cond) {
            Some(true) => Some(then_value),
            Some(false) => Some(else_value),
            None => None,
        }
    }

    /// The value of an `i1` a fact gives directly.
    fn fact_bool(&self, c: Value) -> Option<bool> {
        if c == self.t {
            return Some(true);
        }
        if c == self.fl {
            return Some(false);
        }
        for fact in self.facts_about(c) {
            if fact.rel == Rel::Eq && (fact.a == c || fact.b == c) {
                let other = if fact.a == c { fact.b } else { fact.a };
                if other == self.t {
                    return Some(true);
                }
                if other == self.fl {
                    return Some(false);
                }
            }
        }
        None
    }

    /// The value of an `i1` the facts decide.
    fn known_bool(&mut self, c: Value) -> Option<bool> {
        if let Some(k) = self.fact_bool(c) {
            return Some(k);
        }
        let &Op::ICmp { pred, lhs, rhs, .. } = self.op(c)? else { return None };
        if !self.is_int(lhs) {
            return None;
        }
        let fact = relation(pred, lhs, rhs);
        if self.prove(fact) {
            Some(true)
        } else if self.prove(negation(fact)) {
            Some(false)
        } else {
            None
        }
    }

    // ---------------------------------------------------------------------
    // Proving

    fn prove(&mut self, fact: Fact) -> bool {
        self.steps = 0;
        self.holds(fact.rel, fact.a, fact.b, DEPTH)
    }

    fn facts_about(&self, v: Value) -> Vec<Fact> {
        let mut out: Vec<Fact> = self.assumed.iter().copied().filter(|f| f.a == v || f.b == v).collect();
        if let Some(list) = self.index.get(&v) {
            // Nearest first: the most recent branch is the likeliest to
            // be about this.
            out.extend(list.iter().rev().map(|&k| self.facts[k as usize]));
        }
        out
    }

    fn holds(&mut self, rel: Rel, a: Value, b: Value, depth: u32) -> bool {
        if depth == 0 || self.steps >= STEPS {
            return false;
        }
        self.steps += 1;
        if a == b {
            return matches!(rel, Rel::Ule | Rel::Sle | Rel::Eq);
        }
        if let (Some(x), Some(y)) = (self.int(a), self.int(b))
            && x.bits == y.bits
        {
            let pred = match rel {
                Rel::Ult => ICmpPred::Ult,
                Rel::Ule => ICmpPred::Ule,
                Rel::Slt => ICmpPred::Slt,
                Rel::Sle => ICmpPred::Sle,
                Rel::Eq => ICmpPred::Eq,
                Rel::Ne => ICmpPred::Ne,
            };
            return eval::fold_icmp(pred, x.value, y.value, x.bits);
        }
        match rel {
            Rel::Ult => self.ult(a, b, depth),
            Rel::Ule => self.ule(a, b, depth),
            Rel::Slt | Rel::Sle => self.signed(rel, a, b, depth),
            Rel::Eq => self.facts_about(a).iter().any(|f| f.rel == Rel::Eq && (f.a == b || f.b == b)),
            Rel::Ne => {
                if self.facts_about(a).iter().any(|f| f.rel == Rel::Ne && (f.a == b || f.b == b)) {
                    return true;
                }
                let zero = |s: &Self, v: Value| s.int(v).is_some_and(|k| k.value == 0);
                if zero(self, b) {
                    return self.nonzero(a, depth);
                }
                if zero(self, a) {
                    return self.nonzero(b, depth);
                }
                self.ult(a, b, depth - 1) || self.ult(b, a, depth - 1)
            }
        }
    }

    /// `x != 0`: something is below it, or it is above something nonzero.
    fn nonzero(&mut self, x: Value, depth: u32) -> bool {
        if let Some(k) = self.int(x) {
            return k.value != 0;
        }
        for fact in self.facts_about(x) {
            if fact.b == x && fact.rel == Rel::Ult {
                return true;
            }
            if fact.rel == Rel::Ne && self.int(if fact.a == x { fact.b } else { fact.a }).is_some_and(|k| k.value == 0) {
                return true;
            }
        }
        if let Some(Op::Binary { op: BinOp::Or, lhs, rhs, .. }) = self.op(x) {
            let (lhs, rhs) = (*lhs, *rhs);
            return self.int(rhs).is_some_and(|k| k.value != 0) || self.int(lhs).is_some_and(|k| k.value != 0);
        }
        let _ = depth;
        false
    }

    /// `a < b`, unsigned.
    fn ult(&mut self, a: Value, b: Value, depth: u32) -> bool {
        if depth == 0 {
            return false;
        }
        let d = depth - 1;
        if self.int(b).is_some_and(|k| k.value == 0) {
            return false;
        }
        if self.int(a).is_some_and(|k| k.value == 0) {
            return self.nonzero(b, d);
        }
        // Through the facts: `a < y <= b`, `a <= y < b`, `a == y < b`.
        for fact in self.facts_about(a) {
            if fact.a == a {
                let ok = match fact.rel {
                    Rel::Ult => self.holds(Rel::Ule, fact.b, b, d),
                    Rel::Ule => self.holds(Rel::Ult, fact.b, b, d),
                    Rel::Eq => self.holds(Rel::Ult, fact.b, b, d),
                    _ => false,
                };
                if ok {
                    return true;
                }
            } else if fact.rel == Rel::Eq && self.holds(Rel::Ult, fact.a, b, d) {
                return true;
            }
        }
        for fact in self.facts_about(b) {
            if fact.b == b {
                let ok = match fact.rel {
                    Rel::Ult => self.holds(Rel::Ule, a, fact.a, d),
                    Rel::Ule => self.holds(Rel::Ult, a, fact.a, d),
                    _ => false,
                };
                if ok {
                    return true;
                }
            }
        }
        // By the shape of `a`.
        if let Some(op) = self.op(a) {
            match *op {
                // `x - y <= x` when `y <= x`.
                Op::Binary { op: BinOp::Sub, lhs, rhs, .. } => {
                    if self.holds(Rel::Ult, lhs, b, d) && self.holds(Rel::Ule, rhs, lhs, d) {
                        return true;
                    }
                }
                // `x - k < x` when `k <= x`, for `k > 0`; for `k == 1`,
                // `x != 0`.
                Op::Binary { op: BinOp::Add, lhs, rhs, .. } => {
                    if let Some(k) = self.int(rhs)
                        && k.value != 0
                        && eval::sext(k.value, k.bits) < 0
                    {
                        let minus = eval::mask(k.bits) & k.value.wrapping_neg();
                        let ok = if minus == 1 {
                            self.nonzero(lhs, d)
                        } else {
                            let kv = self.sh.int(k.bits, minus as i64);
                            self.holds(Rel::Ule, kv, lhs, d)
                        };
                        if ok && self.holds(Rel::Ule, lhs, b, d) {
                            return true;
                        }
                    }
                }
                // Never above what it is computed from.
                Op::Binary { op: BinOp::LShr | BinOp::UDiv | BinOp::And, lhs, .. } => {
                    if self.holds(Rel::Ult, lhs, b, d) {
                        return true;
                    }
                    if let Op::Binary { op: BinOp::And, rhs, .. } = *op
                        && self.holds(Rel::Ult, rhs, b, d)
                    {
                        return true;
                    }
                    // `x / k < x` when `x != 0` and `k > 1`.
                    if let Op::Binary { op: BinOp::LShr | BinOp::UDiv, lhs, rhs, .. } = *op
                        && lhs == b
                        && self.int(rhs).is_some_and(|k| {
                            if matches!(op, Op::Binary { op: BinOp::LShr, .. }) { k.value > 0 } else { k.value > 1 }
                        })
                        && self.nonzero(lhs, d)
                    {
                        return true;
                    }
                }
                // `x % y < y` wherever it is defined.
                Op::Binary { op: BinOp::URem, rhs, .. } => {
                    if self.holds(Rel::Ule, rhs, b, d) {
                        return true;
                    }
                }
                Op::Select { cond, then_value, else_value, .. }
                    if self.both_arms(Rel::Ult, cond, then_value, else_value, b, true, d) =>
                {
                    return true;
                }
                _ => {}
            }
            if let Some((max, x, y)) = self.min_max(a) {
                let ok = if max {
                    self.holds(Rel::Ult, x, b, d) && self.holds(Rel::Ult, y, b, d)
                } else {
                    self.holds(Rel::Ult, x, b, d) || self.holds(Rel::Ult, y, b, d)
                };
                if ok {
                    return true;
                }
            }
        }
        // By the shape of `b`.
        if let Some(&Op::Select { cond, then_value, else_value, .. }) = self.op(b)
            && self.both_arms(Rel::Ult, cond, then_value, else_value, a, false, d)
        {
            return true;
        }
        if let Some((max, x, y)) = self.min_max(b) {
            let ok = if max {
                self.holds(Rel::Ult, a, x, d) || self.holds(Rel::Ult, a, y, d)
            } else {
                self.holds(Rel::Ult, a, x, d) && self.holds(Rel::Ult, a, y, d)
            };
            if ok {
                return true;
            }
        }
        // Signed and not negative is unsigned.
        if self.holds(Rel::Slt, a, b, d) && self.holds(Rel::Sle, self.zero_like(a), a, d) {
            return true;
        }
        false
    }

    /// `a <= b`, unsigned.
    fn ule(&mut self, a: Value, b: Value, depth: u32) -> bool {
        if depth == 0 {
            return false;
        }
        let d = depth - 1;
        if self.int(a).is_some_and(|k| k.value == 0) || self.int(b).is_some_and(|k| k.value == eval::mask(k.bits)) {
            return true;
        }
        for fact in self.facts_about(a) {
            if fact.a == a {
                let ok = match fact.rel {
                    Rel::Ult | Rel::Ule | Rel::Eq => self.holds(Rel::Ule, fact.b, b, d),
                    _ => false,
                };
                if ok {
                    return true;
                }
            } else if fact.rel == Rel::Eq && self.holds(Rel::Ule, fact.a, b, d) {
                return true;
            }
        }
        for fact in self.facts_about(b) {
            if fact.b == b && matches!(fact.rel, Rel::Ult | Rel::Ule) && self.holds(Rel::Ule, a, fact.a, d) {
                return true;
            }
        }
        if let Some(op) = self.op(a) {
            match *op {
                Op::Binary { op: BinOp::LShr | BinOp::UDiv | BinOp::URem, lhs, .. } => {
                    if self.holds(Rel::Ule, lhs, b, d) {
                        return true;
                    }
                }
                Op::Binary { op: BinOp::And, lhs, rhs, .. } => {
                    if self.holds(Rel::Ule, lhs, b, d) || self.holds(Rel::Ule, rhs, b, d) {
                        return true;
                    }
                }
                Op::Binary { op: BinOp::Sub, lhs, rhs, .. } => {
                    if self.holds(Rel::Ule, lhs, b, d) && self.holds(Rel::Ule, rhs, lhs, d) {
                        return true;
                    }
                }
                Op::Select { cond, then_value, else_value, .. }
                    if self.both_arms(Rel::Ule, cond, then_value, else_value, b, true, d) =>
                {
                    return true;
                }
                _ => {}
            }
            if let Some((max, x, y)) = self.min_max(a) {
                let ok = if max {
                    self.holds(Rel::Ule, x, b, d) && self.holds(Rel::Ule, y, b, d)
                } else {
                    self.holds(Rel::Ule, x, b, d) || self.holds(Rel::Ule, y, b, d)
                };
                if ok {
                    return true;
                }
            }
        }
        if let Some(op) = self.op(b) {
            match *op {
                // `a <= x - 1` when `a < x`.
                Op::Binary { op: BinOp::Add, lhs, rhs, .. }
                    if self.int(rhs).is_some_and(|k| k.value == eval::mask(k.bits)) =>
                {
                    if self.holds(Rel::Ult, a, lhs, d) {
                        return true;
                    }
                }
                Op::Select { cond, then_value, else_value, .. }
                    if self.both_arms(Rel::Ule, cond, then_value, else_value, a, false, d) =>
                {
                    return true;
                }
                _ => {}
            }
            if let Some((max, x, y)) = self.min_max(b) {
                let ok = if max {
                    self.holds(Rel::Ule, a, x, d) || self.holds(Rel::Ule, a, y, d)
                } else {
                    self.holds(Rel::Ule, a, x, d) && self.holds(Rel::Ule, a, y, d)
                };
                if ok {
                    return true;
                }
            }
        }
        self.ult(a, b, depth)
    }

    /// `a < b` or `a <= b`, signed: through the facts only.
    fn signed(&mut self, rel: Rel, a: Value, b: Value, depth: u32) -> bool {
        let d = depth - 1;
        if rel == Rel::Sle {
            // Not negative: a zero-extended value, or shifted right.
            if self.int(a).is_some_and(|k| k.value == 0) {
                match self.op(b) {
                    Some(Op::Cast { op: CastOp::ZExt, .. }) => return true,
                    Some(&Op::Binary { op: BinOp::LShr, rhs, .. }) if self.int(rhs).is_some_and(|k| k.value > 0) => {
                        return true;
                    }
                    _ => {}
                }
            }
            if self.int(b).is_some_and(|k| k.value == eval::mask(k.bits) >> 1) {
                return true;
            }
        }
        for fact in self.facts_about(a) {
            if fact.a != a {
                if fact.rel == Rel::Eq && self.holds(rel, fact.a, b, d) {
                    return true;
                }
                continue;
            }
            let ok = match (fact.rel, rel) {
                (Rel::Slt, _) => self.holds(Rel::Sle, fact.b, b, d),
                (Rel::Sle, Rel::Sle) | (Rel::Eq, _) => self.holds(rel, fact.b, b, d),
                (Rel::Sle, Rel::Slt) => self.holds(Rel::Slt, fact.b, b, d),
                _ => false,
            };
            if ok {
                return true;
            }
        }
        for fact in self.facts_about(b) {
            if fact.b != b {
                continue;
            }
            let ok = match (fact.rel, rel) {
                (Rel::Slt, _) => self.holds(Rel::Sle, a, fact.a, d),
                (Rel::Sle, _) => self.holds(rel, a, fact.a, d),
                _ => false,
            };
            if ok {
                return true;
            }
        }
        rel == Rel::Sle && self.holds(Rel::Slt, a, b, d)
    }

    /// `rel` between `other` and each arm of `select cond, x, y`, with the
    /// arm's condition assumed: the select on the left (`arm rel other`)
    /// or on the right (`other rel arm`).
    #[allow(clippy::too_many_arguments)]
    fn both_arms(&mut self, rel: Rel, cond: Value, x: Value, y: Value, other: Value, left: bool, d: u32) -> bool {
        // A condition already known picks one arm.
        if let Some(k) = self.fact_bool(cond) {
            let arm = if k { x } else { y };
            let (p, q) = if left { (arm, other) } else { (other, arm) };
            return self.holds(rel, p, q, d);
        }
        let mut ok = true;
        for (arm, value) in [(x, true), (y, false)] {
            let mut extra = Vec::new();
            self.condition(cond, value, &mut extra, 1);
            let before = self.assumed.len();
            self.assumed.extend(extra);
            let (p, q) = if left { (arm, other) } else { (other, arm) };
            ok = self.holds(rel, p, q, d);
            self.assumed.truncate(before);
            if !ok {
                break;
            }
        }
        ok
    }

    // ---------------------------------------------------------------------
    // Values

    fn op(&self, v: Value) -> Option<&'a Op> {
        let Value::Local(x) = v else { return None };
        match self.f.locals[x.index()].def {
            LocalDef::Inst(i) => Some(&self.f.insts[i.index()].op),
            _ => None,
        }
    }

    /// True for a scalar integer of at most 64 bits.
    fn is_int(&self, v: Value) -> bool {
        let ty = match v {
            Value::Local(x) => self.f.locals[x.index()].ty,
            Value::Const(c) => self.sh.consts().get(c).ty,
            Value::Metadata(_) => return false,
        };
        matches!(*self.sh.types().get(ty), Type::Int(b) if b <= 64)
    }

    fn int(&self, v: Value) -> Option<eval::IntConst> {
        let Value::Const(c) = v else { return None };
        if !self.is_int(v) {
            return None;
        }
        eval::int_const(self.sh.consts(), self.sh.types(), c)
    }

    fn all_ones(&self, bits: u32) -> Value {
        self.sh.int(bits, -1)
    }

    /// Zero, of `v`'s type.
    fn zero_like(&self, v: Value) -> Value {
        let ty = match v {
            Value::Local(x) => self.f.locals[x.index()].ty,
            Value::Const(c) => self.sh.consts().get(c).ty,
            Value::Metadata(_) => return v,
        };
        match *self.sh.types().get(ty) {
            Type::Int(bits) => self.sh.int(bits, 0),
            _ => v,
        }
    }
}
