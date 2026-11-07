//! The combiner: rewrites instructions into simpler equivalents, driven by
//! the rules in `opt/rules/combine.rules`.
//!
//! - `dsl`: the rule language and its parser.
//! - `tree`: the discrimination tree that finds the rules matching an
//!   instruction without trying each one.
//! - `eval`: constants, rule expressions and conditions, and folding.
//!
//! The pass works through a list of instructions, starting with all of
//! them. For each it:
//!
//! 1. deletes it if nothing uses it and it has no side effects;
//! 2. puts it in canonical form: a constant operand of a commutative
//!    operation or comparison moves to the right;
//! 3. folds it if all its operands are constant, or if it is a phi whose
//!    entries are all the same value;
//! 4. otherwise tries the rules that match, best first.
//!
//! When an instruction is replaced, its users go back on the list, since
//! they may now match something, and so do the instructions it used,
//! which may now be unused. Use lists make each replacement cost the
//! number of users, not the size of the function. New instructions are
//! queued to go before the one they replace, and the blocks are rebuilt
//! once at the end.

pub mod constload;
pub mod dsl;
pub mod eval;
pub mod tree;

use std::collections::BTreeMap;

use crate::fx::FxHashMap as HashMap;
use std::sync::OnceLock;

use epitome_ir::apint::ApInt;
use epitome_ir::float::FloatKind;
use epitome_ir::module::*;
use epitome_ir::types::{Type, TypeId};

use crate::table::{ConstTable as Constants, TypeTable as Types};

use crate::ctx::Shared;
use dsl::{Cond, FloatLit, Head, Lit, Repl, Subject as Who};
use eval::{Env, Folded, IntConst};
use tree::{Match, Matcher, Shape, Subject};

/// The most instructions the search for a dead store looks at.
const DSE_BUDGET: u32 = 256;

/// The most leaves a chain is reassociated over.
const MAX_CHAIN: usize = 256;

/// The standard rules, compiled once.
pub fn standard() -> &'static Matcher {
    static M: OnceLock<Matcher> = OnceLock::new();
    M.get_or_init(|| {
        let rules = dsl::parse_rules(include_str!("../../rules/combine.rules"))
            .unwrap_or_else(|e| panic!("opt/rules/combine.rules: {e}"));
        Matcher::new(rules)
    })
}

/// The standard rules with `extra` rules before them (tried first), for
/// experiments and for tests that plant a rule.
pub fn with_extra(extra: &str) -> Result<Matcher, String> {
    let mut rules = dsl::parse_rules(extra).map_err(|e| format!("extra rules: {e}"))?;
    rules.extend(
        dsl::parse_rules(include_str!("../../rules/combine.rules")).map_err(|e| format!("combine.rules: {e}"))?,
    );
    Ok(Matcher::new(rules))
}

/// How often each rule (or built-in step) fired.
#[derive(Default, Debug, Clone)]
pub struct Stats {
    pub fired: BTreeMap<String, usize>,
    /// A branch was changed, so blocks may now be unreachable.
    pub cfg_changed: bool,
}

impl Stats {
    fn hit(&mut self, name: &str) {
        *self.fired.entry(name.to_string()).or_default() += 1;
    }

    pub fn total(&self) -> usize {
        self.fired.values().sum()
    }

    pub fn merge(&mut self, other: &Stats) {
        for (k, v) in &other.fired {
            *self.fired.entry(k.clone()).or_default() += v;
        }
        self.cfg_changed |= other.cfg_changed;
    }
}

/// Runs the combiner on one function.
pub fn run(sh: &Shared, f: &mut Function, matcher: &Matcher) -> Stats {
    run_from(sh, f, matcher, None)
}

/// Runs the combiner starting from only the instructions in `seeds`, or
/// from every instruction for `None`. Changes still spread from there to
/// users and operands as usual.
pub fn run_from(sh: &Shared, f: &mut Function, matcher: &Matcher, seeds: Option<&[InstId]>) -> Stats {
    run_with(sh, f, matcher, seeds, false)
}

/// `run_from`, and with `memory`, the rules and steps about memory too:
/// forwarding a stored value to a load (`@` rules), reusing an earlier
/// load, and deleting dead stores. They ask memory SSA's walker
/// (`memssa.rs`), built for the run.
pub fn run_with(sh: &Shared, f: &mut Function, matcher: &Matcher, seeds: Option<&[InstId]>, memory: bool) -> Stats {
    if !f.has_body {
        return Stats::default();
    }
    let mem = memory.then(|| Memory::new(sh, f));
    let mut c = Combiner::new(sh, f, matcher, seeds);
    c.mem = mem;
    c.run();
    c.finish();
    c.stats
}

/// What the memory rules and steps ask: alias analysis, memory SSA and its
/// walker, made when the run starts. The combiner's edits leave them right
/// enough: replacing values with equal ones, and taking out loads and
/// pure instructions, change no write; folding branches only takes paths
/// away, which can only make a walk more cautious. A dead store taken out
/// is told to the walker (`WalkState::remove`).
struct Memory<'s, 'm> {
    aa: crate::aa::BasicAa<'s, 'm>,
    ssa: crate::memssa::MemorySsa,
    walk: crate::memssa::WalkState,
    /// Loads by address and type, for reusing one.
    loads: HashMap<(Value, TypeId), Vec<InstId>>,
    /// Each instruction's place in its block, when the run started.
    pos: Vec<u32>,
}

impl<'s, 'm> Memory<'s, 'm> {
    fn new(sh: &'s Shared<'m>, f: &Function) -> Self {
        let aa = crate::aa::BasicAa::new(sh, f);
        let ssa = crate::memssa::MemorySsa::build(&aa, f);
        let walk = crate::memssa::WalkState::new(f);
        let mut loads: HashMap<(Value, TypeId), Vec<InstId>> = HashMap::default();
        let mut pos = vec![0u32; f.insts.len()];
        for &b in &f.layout {
            for (k, &i) in f.blocks[b.index()].insts.iter().enumerate() {
                pos[i.index()] = k as u32;
                if let Op::Load { volatile: false, atomic: None, ptr, .. } = f.insts[i.index()].op
                    && let Some(r) = f.insts[i.index()].result
                {
                    loads.entry((ptr, f.locals[r.index()].ty)).or_default().push(i);
                }
            }
        }
        Memory { aa, ssa, walk, loads, pos }
    }

    /// Whether instruction `a` comes before `b` on every path to `b`.
    fn dominates(&self, a: InstId, b: InstId) -> bool {
        match (self.walk.block_of(a), self.walk.block_of(b)) {
            (Some(x), Some(y)) if x == y => self.pos[a.index()] < self.pos[b.index()],
            (Some(x), Some(y)) => self.walk.dominates(x, y),
            _ => false,
        }
    }
}

struct Combiner<'s, 'm, 'f> {
    sh: &'s Shared<'m>,
    f: &'f mut Function,
    matcher: &'s Matcher,
    /// The instructions using each local, once per operand.
    uses: Vec<Vec<InstId>>,
    erased: Vec<bool>,
    block_of: Vec<Option<BlockId>>,
    /// New instructions to place before an existing one.
    before: HashMap<InstId, Vec<InstId>>,
    work: Vec<InstId>,
    queued: Vec<bool>,
    /// What replaced each local, for debug records.
    replaced: HashMap<LocalId, Value>,
    /// Debug records of erased instructions, waiting for the next one.
    pending_records: Vec<DebugRecord>,
    stats: Stats,
    /// For the memory rules and steps, when asked for.
    mem: Option<Memory<'s, 'm>>,
}

fn inst_has_result(f: &Function, i: InstId) -> bool {
    f.insts[i.index()].result.is_some()
}

/// Instructions that can be deleted when nothing uses them.
fn is_pure(op: &Op) -> bool {
    matches!(
        op,
        Op::Binary { .. }
            | Op::ICmp { .. }
            | Op::FCmp { .. }
            | Op::Select { .. }
            | Op::Cast { .. }
            | Op::FNeg { .. }
            | Op::Freeze(_)
            | Op::Gep { .. }
            | Op::Phi { .. }
            | Op::ExtractValue { .. }
            | Op::InsertValue { .. }
            | Op::ExtractElement { .. }
            | Op::InsertElement { .. }
            | Op::ShuffleVector { .. }
            | Op::BitExtract { .. }
            | Op::BitInsert { .. }
            | Op::Alloca { .. }
            | Op::Load { volatile: false, atomic: None, .. }
    )
}

/// The operation and operands the matcher sees for an instruction.
/// A block's label, as the matcher sees a block operand.
fn label(f: &Function, b: BlockId) -> Value {
    Value::Local(f.blocks[b.index()].local)
}

fn head_of(f: &Function, op: &Op) -> Option<(Head, [Value; 3])> {
    let pad = Value::Const(ConstId(0));
    Some(match *op {
        Op::Br(b) => (Head::Br, [label(f, b), pad, pad]),
        Op::CondBr { cond, then_dest, else_dest } => (Head::CondBr, [cond, label(f, then_dest), label(f, else_dest)]),
        Op::Binary { op, lhs, rhs, .. } => (Head::Bin(op), [lhs, rhs, pad]),
        Op::ICmp { pred, lhs, rhs, .. } => (Head::ICmp(pred), [lhs, rhs, pad]),
        Op::Select { cond, then_value, else_value, .. } => (Head::Select, [cond, then_value, else_value]),
        Op::Cast { op, value, .. } => (Head::Cast(op), [value, pad, pad]),
        Op::FNeg { value, .. } => (Head::FNeg, [value, pad, pad]),
        Op::Freeze(v) => (Head::Freeze, [v, pad, pad]),
        Op::Load { volatile: false, atomic: None, ptr, .. } => (Head::Load, [ptr, pad, pad]),
        _ => return None,
    })
}

/// The matcher's view of a function: instructions erased by the combiner
/// are invisible.
struct View<'a> {
    f: &'a Function,
    erased: &'a [bool],
    consts: &'a Constants,
    types: &'a Types,
}

impl Subject for View<'_> {
    fn shape(&self, v: Value) -> Shape {
        match v {
            Value::Local(l) => match self.f.locals[l.index()].def {
                LocalDef::Inst(i) if !self.erased.get(i.index()).copied().unwrap_or(false) => {
                    match head_of(self.f, &self.f.insts[i.index()].op) {
                        Some((h, ops)) => Shape::Op(h, ops),
                        None => Shape::Opaque,
                    }
                }
                _ => Shape::Opaque,
            },
            Value::Const(c) => Shape::Const(c),
            Value::Metadata(_) => Shape::Opaque,
        }
    }

    fn lits(&self, c: ConstId) -> Vec<Lit> {
        eval::lits_of(self.consts, self.types, c)
    }

    fn is_int_const(&self, c: ConstId) -> bool {
        eval::int_const(self.consts, self.types, c).is_some()
    }
}

impl<'s, 'm, 'f> Combiner<'s, 'm, 'f> {
    fn new(sh: &'s Shared<'m>, f: &'f mut Function, matcher: &'s Matcher, seeds: Option<&[InstId]>) -> Self {
        let mut c = Combiner {
            sh,
            uses: vec![Vec::new(); f.locals.len()],
            erased: vec![false; f.insts.len()],
            block_of: f.inst_blocks(),
            before: HashMap::default(),
            work: Vec::new(),
            queued: vec![false; f.insts.len()],
            replaced: HashMap::default(),
            pending_records: Vec::new(),
            stats: Stats::default(),
            f,
            matcher,
            mem: None,
        };
        let order: Vec<InstId> = c.f.insts_in_order().collect();
        for &i in &order {
            c.add_uses(i);
        }
        // Visit in program order: the list is a stack.
        match seeds {
            Some(list) => list.iter().rev().for_each(|&i| c.push(i)),
            None => order.iter().rev().for_each(|&i| c.push(i)),
        }
        c
    }

    fn grow(&mut self) {
        self.uses.resize(self.f.locals.len(), Vec::new());
        self.erased.resize(self.f.insts.len(), false);
        self.queued.resize(self.f.insts.len(), false);
        self.block_of.resize(self.f.insts.len(), None);
    }

    fn add_uses(&mut self, i: InstId) {
        let mut ls = Vec::new();
        self.f.insts[i.index()].op.for_each_value(|v| {
            if let Value::Local(l) = v {
                ls.push(*l);
            }
        });
        for l in ls {
            self.uses[l.index()].push(i);
        }
    }

    fn push(&mut self, i: InstId) {
        if !self.queued[i.index()] && !self.erased[i.index()] {
            self.queued[i.index()] = true;
            self.work.push(i);
        }
    }

    /// Queues the instruction defining `v`, if there is one.
    fn push_def(&mut self, v: Value) {
        if let Value::Local(l) = v
            && let LocalDef::Inst(d) = self.f.locals[l.index()].def
        {
            self.push(d);
        }
    }

    fn value_type(&self, v: Value) -> Option<TypeId> {
        match v {
            Value::Local(l) => Some(self.f.locals[l.index()].ty),
            Value::Const(c) => Some(self.sh.consts().get(c).ty),
            Value::Metadata(_) => None,
        }
    }

    fn run(&mut self) {
        // A bound on the work, in case a set of rules undoes itself.
        let mut budget = 20 * self.f.insts.len() + 1000;
        while let Some(i) = self.work.pop() {
            self.queued[i.index()] = false;
            if self.erased[i.index()] {
                continue;
            }
            if budget == 0 {
                self.stats.hit("(stopped: work limit reached)");
                break;
            }
            budget -= 1;
            self.visit(i);
        }
    }

    /// Reassociation, as LLVM's `reassociate` pass does it: a chain of one
    /// associative, commutative integer operation (`add`, `mul`, `and`,
    /// `or`, `xor`), through operands that are the same operation with no
    /// other use, is rebuilt as a chain leaning left with its leaves sorted
    /// by `rank`. Constants then gather at the outside, where the constant
    /// rules fold them (`(x + 3) + y + 5` becomes `(x + y) + 8`), and chains
    /// with the same leaves come out the same, so CSE finds them. Only the
    /// chain's top is rebuilt, once; a chain already in order is left
    /// alone, so this ends. Flags such as `nsw` are dropped, being true
    /// only of the order written.
    fn reassociate(&mut self, i: InstId) -> Option<Value> {
        let Op::Binary { op, lhs, rhs, .. } = self.f.insts[i.index()].op else { return None };
        if !matches!(op, BinOp::Add | BinOp::Mul | BinOp::And | BinOp::Or | BinOp::Xor) {
            return None;
        }
        let r = self.f.insts[i.index()].result?;
        let ty = self.f.locals[r.index()].ty;
        // Part of a longer chain: its top does the work.
        let inner = |me: &Self, v: Value| -> Option<(Value, Value)> {
            let Value::Local(l) = v else { return None };
            let LocalDef::Inst(d) = me.f.locals[l.index()].def else { return None };
            if me.erased[d.index()] || me.uses[l.index()].len() != 1 || me.f.locals[l.index()].ty != ty {
                return None;
            }
            match me.f.insts[d.index()].op {
                Op::Binary { op: o, lhs, rhs, .. } if o == op => Some((lhs, rhs)),
                _ => None,
            }
        };
        if let [u] = self.uses[r.index()][..]
            && let Some(ur) = self.f.insts[u.index()].result
            && inner(self, Value::Local(ur)).is_some_and(|(a, b)| a == Value::Local(r) || b == Value::Local(r))
        {
            return None;
        }
        // The leaves, left to right, and whether the chain leans left.
        let mut leaves = Vec::new();
        let mut leans_left = true;
        let mut interior = 0;
        // (value, whether it is a right operand)
        let mut stack = vec![(rhs, true), (lhs, false)];
        while let Some((v, right)) = stack.pop() {
            match inner(self, v) {
                Some((a, b)) if leaves.len() + stack.len() < MAX_CHAIN => {
                    // An inner node on the right is not leaning left.
                    leans_left &= !right;
                    interior += 1;
                    stack.push((b, true));
                    stack.push((a, false));
                }
                _ => leaves.push(v),
            }
        }
        if interior == 0 {
            return None;
        }
        let mut sorted = leaves.clone();
        sorted.sort_by_key(|&v| self.rank(v));
        if leans_left && sorted == leaves {
            return None;
        }
        if !self.allowed("(reassociated)", i) {
            return None;
        }
        let mut acc = sorted[0];
        let mut last = None;
        for &leaf in &sorted[1..] {
            let new = self.f.add_inst(
                Op::Binary { op, flags: IntFlags::NONE, fmf: FastMath::default(), lhs: acc, rhs: leaf },
                Some((None, ty)),
            );
            self.grow();
            self.block_of[new.index()] = self.block_of[i.index()];
            self.before.entry(i).or_default().push(new);
            self.add_uses(new);
            self.push(new);
            acc = self.f.result(new);
            last = Some(new);
        }
        // The new top takes the old one's name.
        if let Some(last) = last {
            let name = self.f.locals[r.index()].name.take();
            let new = self.f.insts[last.index()].result.unwrap();
            self.f.locals[new.index()].name = name;
        }
        Some(acc)
    }

    /// Linear combinations: an `add` or `sub` over operands that are
    /// `add`, `sub`, or `shl` or `mul` by a constant, with no other use, is
    /// a sum of its leaves times constants, plus a constant. Each leaf's
    /// coefficients are added up (modulo the width), and the sum is rebuilt
    /// when that takes fewer instructions than it did: `(x + 1) - (x + 3)`
    /// is `-2`, and `x - (x + 1) + x` is `x - 1`. Positive terms come first,
    /// sorted by `rank`, then the negative ones, then the constant. Flags
    /// are dropped, as for `reassociate`.
    fn linear(&mut self, i: InstId) -> Option<Value> {
        let Op::Binary { op: BinOp::Add | BinOp::Sub, .. } = self.f.insts[i.index()].op else { return None };
        let r = self.f.insts[i.index()].result?;
        let ty = self.f.locals[r.index()].ty;
        let bits = eval::int_bits(self.sh.types(), ty).filter(|&b| b <= 64)?;
        let m = eval::mask(bits);
        // An operand that is part of the sum: (op, lhs, rhs), with the
        // constant of a shift or multiply.
        let node = |me: &Self, d: InstId| -> Option<(BinOp, Value, Value, u64)> {
            let Op::Binary { op, lhs, rhs, .. } = me.f.insts[d.index()].op else { return None };
            match op {
                BinOp::Add | BinOp::Sub => Some((op, lhs, rhs, 0)),
                BinOp::Shl | BinOp::Mul => {
                    let Value::Const(c) = rhs else { return None };
                    let k = me.int(c)?.value;
                    (op == BinOp::Mul || k < bits as u64).then_some((op, lhs, rhs, k))
                }
                _ => None,
            }
        };
        let inner = |me: &Self, v: Value| -> Option<(BinOp, Value, Value, u64)> {
            let Value::Local(l) = v else { return None };
            let LocalDef::Inst(d) = me.f.locals[l.index()].def else { return None };
            if me.erased[d.index()] || me.uses[l.index()].len() != 1 || me.f.locals[l.index()].ty != ty {
                return None;
            }
            node(me, d)
        };
        // Part of a larger sum: its top does the work.
        if let [u] = self.uses[r.index()][..]
            && let Some(ur) = self.f.insts[u.index()].result
            && self.f.locals[ur.index()].ty == ty
            && node(self, u).is_some_and(|(o, a, b, _)| {
                a == Value::Local(r) || (b == Value::Local(r) && matches!(o, BinOp::Add | BinOp::Sub))
            })
        {
            return None;
        }
        let mut terms: Vec<(Value, u64)> = Vec::new();
        let mut index: HashMap<Value, usize> = HashMap::default();
        let mut konst = 0u64;
        let mut interior = 0usize;
        let mut stack = vec![(Value::Local(r), 1u64)];
        // The root is expanded once, at the top. In code that cannot be
        // reached, a value may be its own operand (`%a = sub %a, %x`, after
        // a one-entry phi folds); met again, it is an inner node like any
        // other, under the same bound.
        let mut top = true;
        while let Some((v, c)) = stack.pop() {
            if let Value::Const(k) = v
                && let Some(k) = self.int(k)
            {
                konst = konst.wrapping_add(c.wrapping_mul(k.value)) & m;
                continue;
            }
            let here = if std::mem::take(&mut top) {
                node(self, i)
            } else if interior < MAX_CHAIN {
                inner(self, v)
            } else {
                None
            };
            match here {
                Some((BinOp::Add, a, b, _)) => {
                    stack.push((b, c));
                    stack.push((a, c));
                }
                Some((BinOp::Sub, a, b, _)) => {
                    stack.push((b, c.wrapping_neg() & m));
                    stack.push((a, c));
                }
                Some((BinOp::Shl, a, _, k)) => stack.push((a, (c << k) & m)),
                Some((_, a, _, k)) => stack.push((a, c.wrapping_mul(k) & m)),
                None => {
                    let at = *index.entry(v).or_insert_with(|| {
                        terms.push((v, 0));
                        terms.len() - 1
                    });
                    terms[at].1 = terms[at].1.wrapping_add(c) & m;
                    continue;
                }
            }
            interior += 1;
        }
        terms.retain(|t| t.1 != 0);
        terms.sort_by_key(|t| self.rank(t.0));
        // Positive and negative terms, by their coefficients' signs.
        let (pos, neg): (Vec<(Value, u64)>, Vec<(Value, u64)>) =
            terms.into_iter().partition(|t| eval::sext(t.1, bits) > 0 || t.1 == 1);
        let neg: Vec<(Value, u64)> = neg.into_iter().map(|(v, c)| (v, c.wrapping_neg() & m)).collect();
        let scaled = pos.iter().chain(&neg).filter(|t| t.1 != 1).count();
        let cost = scaled + pos.len().saturating_sub(1) + neg.len() + (konst != 0 && !pos.is_empty()) as usize;
        if cost >= interior || !self.allowed("(linear)", i) {
            return None;
        }
        let mut last = None;
        let mut emit = |me: &mut Self, op: BinOp, lhs: Value, rhs: Value| -> Value {
            let new = me.f.add_inst(
                Op::Binary { op, flags: IntFlags::NONE, fmf: FastMath::default(), lhs, rhs },
                Some((None, ty)),
            );
            me.grow();
            me.block_of[new.index()] = me.block_of[i.index()];
            me.before.entry(i).or_default().push(new);
            me.add_uses(new);
            me.push(new);
            last = Some(new);
            me.f.result(new)
        };
        let mut term = |me: &mut Self, (v, c): (Value, u64)| -> Option<Value> {
            if c == 1 {
                return Some(v);
            }
            Some(if c.is_power_of_two() {
                let k = me.int_value(ty, c.trailing_zeros() as u64)?;
                emit(me, BinOp::Shl, v, k)
            } else {
                let k = me.int_value(ty, c)?;
                emit(me, BinOp::Mul, v, k)
            })
        };
        let pos: Vec<Value> = pos.into_iter().map(|t| term(self, t)).collect::<Option<_>>()?;
        let neg: Vec<Value> = neg.into_iter().map(|t| term(self, t)).collect::<Option<_>>()?;
        drop(term);
        let k = self.int_value(ty, konst)?;
        // With no positive term, the constant (perhaps 0) comes first.
        let mut acc = pos.first().copied().unwrap_or(k);
        for &p in pos.iter().skip(1) {
            acc = emit(self, BinOp::Add, acc, p);
        }
        for &n in &neg {
            acc = emit(self, BinOp::Sub, acc, n);
        }
        if konst != 0 && !pos.is_empty() {
            acc = emit(self, BinOp::Add, acc, k);
        }
        drop(emit);
        if let Some(last) = last {
            let name = self.f.locals[r.index()].name.take();
            let new = self.f.insts[last.index()].result.unwrap();
            self.f.locals[new.index()].name = name;
        }
        Some(acc)
    }

    /// Demanded bits: an instruction that reads only some of its operand's
    /// bits (`trunc` the low ones, `lshr` by a constant the high ones, `shl`
    /// by a constant the low ones, `and` with a constant those set) may read
    /// them from a simpler value that has the same bits there. Through an
    /// `or` one side can go if it cannot set any of those bits, and through
    /// an `and` with a constant if the constant keeps them all. This takes
    /// apart the integers SROA packs small structs into: the low half of
    /// `(x & 0xffffffff00000000) | zext y` is `y`'s. The operand is changed
    /// in place; the instruction's value is the same.
    fn demanded(&mut self, i: InstId) -> bool {
        let (operand, demand) = match &self.f.insts[i.index()].op {
            Op::Cast { op: CastOp::Trunc, value, .. } => {
                let Some(to) = self.f.insts[i.index()].result else { return false };
                let Some(w) = eval::int_bits(self.sh.types(), self.f.locals[to.index()].ty) else { return false };
                (*value, eval::mask(w))
            }
            Op::Binary {
                op: op @ (BinOp::LShr | BinOp::AShr | BinOp::Shl | BinOp::And),
                lhs,
                rhs: Value::Const(c),
                ..
            } => {
                let Some(k) = self.int(*c) else { return false };
                let m = eval::mask(k.bits);
                let d = match op {
                    BinOp::And => k.value,
                    _ if k.value >= k.bits as u64 => return false,
                    BinOp::LShr | BinOp::AShr => m & !eval::mask(k.value as u32),
                    _ => eval::mask(k.bits - k.value as u32),
                };
                (*lhs, d)
            }
            _ => return false,
        };
        let Some(bits) = self.value_type(operand).and_then(|t| eval::int_bits(self.sh.types(), t)).filter(|&b| b <= 64)
        else {
            return false;
        };
        let demand = demand & eval::mask(bits);
        let Some(w) = self.simpler_for(operand, demand, bits, 8) else { return false };
        if w == operand || !self.allowed("(demanded bits)", i) {
            return false;
        }
        self.set_operand(i, w);
        true
    }

    /// Changes the first operand of `i` (the value of a cast, the left of
    /// a binary operation, the aggregate of an `extractvalue`), keeping the
    /// use lists right, and visits it again.
    fn set_operand(&mut self, i: InstId, w: Value) {
        let old = match &mut self.f.insts[i.index()].op {
            Op::Cast { value: x, .. } | Op::Binary { lhs: x, .. } | Op::ExtractValue { aggregate: x, .. } => {
                std::mem::replace(x, w)
            }
            _ => return,
        };
        if let Value::Local(l) = old {
            let list = &mut self.uses[l.index()];
            if let Some(k) = list.iter().position(|&x| x == i) {
                list.swap_remove(k);
            }
            self.push_def(old);
        }
        if let Value::Local(l) = w {
            self.uses[l.index()].push(i);
        }
        self.push(i);
    }

    /// Folds from the bits that may be set (`ones`): an `and` with a
    /// constant that keeps all of them is its operand; a shift left and back
    /// right (or right and back left) by the same amount that loses none of
    /// them is the value shifted; and a value with no bit that may be set is
    /// 0.
    fn known_bits(&self, i: InstId) -> Option<Value> {
        let r = self.f.insts[i.index()].result?;
        let ty = self.f.locals[r.index()].ty;
        if let Op::ICmp { pred, lhs, rhs: Value::Const(c), .. } = self.f.insts[i.index()].op {
            let k = self.int(c)?;
            let b = self.value_type(lhs).and_then(|t| eval::int_bits(self.sh.types(), t)).filter(|&b| b <= 64)?;
            if !matches!(self.sh.types().get(ty), Type::Int(1)) {
                return None;
            }
            return self.compare_known(pred, self.ones(lhs, b, 6), k.value, b).map(|v| self.int_value(ty, v as u64))?;
        }
        let bits = eval::int_bits(self.sh.types(), ty).filter(|&b| b <= 64)?;
        if !matches!(self.sh.types().get(ty), Type::Int(_)) {
            return None;
        }
        let m = eval::mask(bits);
        let shifted = |me: &Self, v: Value, want: BinOp, k: u64| -> Option<Value> {
            let Value::Local(l) = v else { return None };
            let LocalDef::Inst(d) = me.f.locals[l.index()].def else { return None };
            let Op::Binary { op, lhs, rhs: Value::Const(c), .. } = me.f.insts[d.index()].op else { return None };
            (op == want && me.int(c)?.value == k && !me.erased[d.index()]).then_some(lhs)
        };
        let v = match self.f.insts[i.index()].op {
            Op::Binary { op: BinOp::And, lhs, rhs: Value::Const(c), .. } => {
                let k = self.int(c)?.value;
                (self.ones(lhs, bits, 6) & !k & m == 0).then_some(lhs)
            }
            Op::Binary { op: BinOp::LShr, lhs, rhs: Value::Const(c), .. } => {
                let k = self.int(c)?.value;
                if k >= bits as u64 {
                    return None;
                }
                shifted(self, lhs, BinOp::Shl, k).filter(|&x| self.ones(x, bits, 6) & !eval::mask(bits - k as u32) == 0)
            }
            Op::Binary { op: BinOp::Shl, lhs, rhs: Value::Const(c), .. } => {
                let k = self.int(c)?.value;
                if k >= bits as u64 {
                    return None;
                }
                shifted(self, lhs, BinOp::LShr, k).filter(|&x| self.ones(x, bits, 6) & eval::mask(k as u32) == 0)
            }
            _ => None,
        };
        if v.is_some() {
            return v;
        }
        // zext (trunc y) is y, if y fits in the narrow type.
        if let Op::Cast { op: CastOp::ZExt, value, .. } = self.f.insts[i.index()].op
            && let Some((y, narrow)) = self.trunc_of(value)
            && self.value_type(y) == Some(ty)
            && self.ones(y, bits, 6) & !eval::mask(narrow) == 0
        {
            return Some(y);
        }
        match self.f.insts[i.index()].op {
            Op::Binary { op: BinOp::And | BinOp::Or | BinOp::Xor | BinOp::Shl | BinOp::LShr, .. }
            | Op::Cast { op: CastOp::ZExt, .. }
                if self.ones(Value::Local(r), bits, 6) == 0 =>
            {
                self.int_value(ty, 0)
            }
            _ => None,
        }
    }

    /// A funnel shift written out: `(x << a) | (y >> (w - a))` is
    /// `llvm.fshl(x, y, a)`, a rotation when x and y are the same, which
    /// code generation selects as one instruction where there is one, and
    /// writes back as the two shifts where there is not.
    fn funnel(&mut self, i: InstId) -> Option<Value> {
        let Op::Binary { op: BinOp::Or, lhs, rhs, .. } = self.f.insts[i.index()].op else { return None };
        let r = self.f.insts[i.index()].result?;
        let ty = self.f.locals[r.index()].ty;
        let Type::Int(w) = *self.sh.types().get(ty) else { return None };
        let k = [8, 16, 32, 64].iter().position(|&b| b == w)?;
        let g = self.sh.intrinsics.fshl[k]?;
        let shift = |me: &Self, v: Value, want: BinOp| -> Option<(Value, u64)> {
            let Value::Local(l) = v else { return None };
            let LocalDef::Inst(d) = me.f.locals[l.index()].def else { return None };
            let Op::Binary { op, lhs, rhs: Value::Const(c), .. } = me.f.insts[d.index()].op else { return None };
            (op == want && !me.erased[d.index()]).then_some(())?;
            Some((lhs, me.int(c)?.value))
        };
        let (hi, lo) = match (shift(self, lhs, BinOp::Shl), shift(self, rhs, BinOp::LShr)) {
            (Some(h), Some(l)) => (h, l),
            _ => (shift(self, rhs, BinOp::Shl)?, shift(self, lhs, BinOp::LShr)?),
        };
        if hi.1 == 0 || hi.1 >= w as u64 || hi.1 + lo.1 != w as u64 || !self.allowed("(funnel shift)", i) {
            return None;
        }
        let amount = self.int_value(ty, hi.1)?;
        let fn_ty = self.sh.published(g)?.ty;
        let call = crate::inline::plain_call(fn_ty, self.sh.global_ref(g), vec![hi.0, lo.0, amount]);
        let new = self.f.add_inst(Op::Call(Box::new(call)), Some((None, ty)));
        self.grow();
        self.block_of[new.index()] = self.block_of[i.index()];
        self.before.entry(i).or_default().push(new);
        self.add_uses(new);
        self.push(new);
        let name = self.f.locals[r.index()].name.take();
        let v = self.f.insts[new.index()].result.unwrap();
        self.f.locals[v.index()].name = name;
        Some(Value::Local(v))
    }

    /// The answer of `x pred c`, where `x` lies in `[0, ones]` (its bits
    /// outside `ones` are zero), if that range decides it.
    fn compare_known(&self, pred: ICmpPred, ones: u64, c: u64, bits: u32) -> Option<bool> {
        let top = 1u64 << (bits - 1);
        // Within [0, hi] as unsigned; as signed too if hi's sign bit is clear.
        let hi = ones;
        let signed = hi & top == 0;
        let sc = eval::sext(c, bits);
        match pred {
            ICmpPred::Eq if c & !ones != 0 => Some(false),
            ICmpPred::Ne if c & !ones != 0 => Some(true),
            ICmpPred::Ult if hi < c => Some(true),
            ICmpPred::Ult if c == 0 => Some(false),
            ICmpPred::Ule if hi <= c => Some(true),
            ICmpPred::Ugt if hi <= c => Some(false),
            ICmpPred::Uge if hi < c => Some(false),
            ICmpPred::Uge if c == 0 => Some(true),
            ICmpPred::Slt if signed && sc <= 0 => Some(false),
            ICmpPred::Slt if signed && (hi as i64) < sc => Some(true),
            ICmpPred::Sle if signed && sc < 0 => Some(false),
            ICmpPred::Sle if signed && (hi as i64) <= sc => Some(true),
            ICmpPred::Sgt if signed && sc < 0 => Some(true),
            ICmpPred::Sgt if signed && (hi as i64) <= sc => Some(false),
            ICmpPred::Sge if signed && sc <= 0 => Some(true),
            ICmpPred::Sge if signed && (hi as i64) < sc => Some(false),
            _ => None,
        }
    }

    /// `extractvalue` from an `insertvalue`: the element inserted at the
    /// same place, or (`Err`) the aggregate below a chain of insertions at
    /// other places, to extract from instead.
    fn extract_insert(&self, i: InstId) -> Option<Result<Value, Value>> {
        let Op::ExtractValue { aggregate, ref indices } = self.f.insts[i.index()].op else { return None };
        let mut agg = aggregate;
        for _ in 0..64 {
            let Value::Local(l) = agg else { break };
            let LocalDef::Inst(d) = self.f.locals[l.index()].def else { break };
            if self.erased[d.index()] {
                break;
            }
            let Op::InsertValue { aggregate: below, element, indices: ref at } = self.f.insts[d.index()].op else {
                break;
            };
            if at == indices {
                return Some(Ok(element));
            }
            // One a prefix of the other: the insertion overlaps.
            let n = at.len().min(indices.len());
            if at[..n] == indices[..n] {
                break;
            }
            agg = below;
        }
        (agg != aggregate).then_some(Err(agg))
    }

    /// A value, other than `v` and already made, with the same bits as `v`
    /// where `demand` has them set.
    fn simpler_for(&self, v: Value, demand: u64, bits: u32, depth: u32) -> Option<Value> {
        let Value::Local(l) = v else { return None };
        let LocalDef::Inst(d) = self.f.locals[l.index()].def else { return None };
        if depth == 0 || self.erased[d.index()] {
            return None;
        }
        let deeper = |w: Value| Some(self.simpler_for(w, demand, bits, depth - 1).unwrap_or(w));
        match self.f.insts[d.index()].op {
            Op::Binary { op: BinOp::Or, lhs, rhs, .. } => {
                if self.ones(lhs, bits, 6) & demand == 0 {
                    deeper(rhs)
                } else if self.ones(rhs, bits, 6) & demand == 0 {
                    deeper(lhs)
                } else {
                    None
                }
            }
            Op::Binary { op: BinOp::And, lhs, rhs: Value::Const(c), .. } => {
                let k = self.int(c)?;
                if k.value & demand == demand { deeper(lhs) } else { None }
            }
            // zext (trunc y), where only the narrow bits are wanted: y.
            Op::Cast { op: CastOp::ZExt, value, .. } => {
                let (y, narrow) = self.trunc_of(value)?;
                (self.value_type(y) == self.value_type(v) && demand & !eval::mask(narrow) == 0).then(|| deeper(y))?
            }
            _ => None,
        }
    }

    /// `v` as `trunc y`: y and the narrow width.
    fn trunc_of(&self, v: Value) -> Option<(Value, u32)> {
        let Value::Local(l) = v else { return None };
        let LocalDef::Inst(d) = self.f.locals[l.index()].def else { return None };
        let Op::Cast { op: CastOp::Trunc, value, .. } = self.f.insts[d.index()].op else { return None };
        if self.erased[d.index()] {
            return None;
        }
        Some((value, eval::int_bits(self.sh.types(), self.f.locals[l.index()].ty)?))
    }

    /// The bits of `v` that may be set (`bits` wide): a mask the value is
    /// known to fit in.
    fn ones(&self, v: Value, bits: u32, depth: u32) -> u64 {
        let all = eval::mask(bits);
        match v {
            Value::Const(c) => match &self.sh.consts().get(c).kind {
                ConstKind::Int(_) | ConstKind::Zero => self.int(c).map_or(all, |k| k.value),
                _ => all,
            },
            Value::Local(l) if depth > 0 => {
                let LocalDef::Inst(d) = self.f.locals[l.index()].def else { return all };
                match self.f.insts[d.index()].op {
                    Op::Binary { op: BinOp::And, lhs, rhs, .. } => {
                        self.ones(lhs, bits, depth - 1) & self.ones(rhs, bits, depth - 1)
                    }
                    Op::Binary { op: BinOp::Or | BinOp::Xor, lhs, rhs, .. } => {
                        self.ones(lhs, bits, depth - 1) | self.ones(rhs, bits, depth - 1)
                    }
                    Op::Binary { op: BinOp::Shl, lhs, rhs: Value::Const(c), .. } => match self.int(c) {
                        Some(k) if k.value < bits as u64 => (self.ones(lhs, bits, depth - 1) << k.value) & all,
                        _ => all,
                    },
                    Op::Binary { op: BinOp::LShr, lhs, rhs: Value::Const(c), .. } => match self.int(c) {
                        Some(k) if k.value < bits as u64 => self.ones(lhs, bits, depth - 1) >> k.value,
                        _ => all,
                    },
                    Op::Cast { op: CastOp::ZExt, value, .. } => {
                        match self.value_type(value).and_then(|t| eval::int_bits(self.sh.types(), t)) {
                            Some(w) if w < bits => self.ones(value, w, depth - 1),
                            _ => all,
                        }
                    }
                    _ => all,
                }
            }
            _ => all,
        }
    }

    /// An earlier load of the same address and type, before this one on
    /// every path, under the same write (both loads' clobbers are the same
    /// write, found exactly): nothing between them can have changed the
    /// bytes, so this load can take that one's value. Up to 16 candidates
    /// are tried, nearest-made first.
    fn reuse_load(&self, i: InstId) -> Option<Value> {
        let mem = self.mem.as_ref()?;
        let Op::Load { volatile: false, atomic: None, ptr, .. } = self.f.insts[i.index()].op else { return None };
        let r = self.f.insts[i.index()].result?;
        let ty = self.f.locals[r.index()].ty;
        let candidates = mem.loads.get(&(ptr, ty))?;
        let mine = mem.walk.clobber_of(&mem.ssa, &mem.aa, self.f, i).filter(|c| c.exact)?;
        let mut tried = 0;
        for &k in candidates.iter().rev() {
            if k == i || self.erased[k.index()] || !mem.dominates(k, i) {
                continue;
            }
            tried += 1;
            if tried > 16 {
                break;
            }
            let theirs = mem.walk.clobber_of(&mem.ssa, &mem.aa, self.f, k);
            if theirs == Some(mine) && self.allowed("(load reused)", i) {
                return Some(self.f.result(k));
            }
        }
        None
    }

    /// A store that changes nothing anyone sees, and why:
    ///
    /// - it stores what is already there: the value of a load of the same
    ///   bytes, with no write to them between (the load's clobber and the
    ///   store's are the same write);
    /// - on every path from it, the same bytes are written again before
    ///   anything may read them, or the path ends where nothing can (an
    ///   `unreachable`; a return, if the memory is a local of the function
    ///   that never escapes). The search goes forward over the CFG, through
    ///   loops, with a budget of instructions; a call that may unwind stops
    ///   it, unless the memory is local, since a handler could read it.
    fn dead_store(&self, i: InstId) -> Option<&'static str> {
        let mem = self.mem.as_ref()?;
        let Op::Store { volatile: false, atomic: None, value, ptr, .. } = self.f.insts[i.index()].op else {
            return None;
        };
        let loc = crate::memssa::location_of(&mem.aa, self.f, i)?;
        let size = loc.size?;
        // The value already there.
        if let Value::Local(l) = value
            && let LocalDef::Inst(ld) = self.f.locals[l.index()].def
            && !self.erased[ld.index()]
            && let Op::Load { volatile: false, atomic: None, ptr: from, .. } = self.f.insts[ld.index()].op
            && crate::memssa::location_of(&mem.aa, self.f, ld).and_then(|x| x.size) == Some(size)
            && mem.aa.alias(self.f, loc, crate::aa::Location { ptr: from, size: Some(size) })
                == crate::aa::AliasResult::MustAlias
        {
            let a = mem.walk.clobber_of(&mem.ssa, &mem.aa, self.f, ld).filter(|c| c.exact);
            let b = mem.walk.clobber_of(&mem.ssa, &mem.aa, self.f, i).filter(|c| c.exact);
            if a.is_some() && a == b && self.allowed("(store of the value there)", i) {
                return Some("(store of the value there)");
            }
        }
        // Overwritten before it can be read, on every path.
        let local = mem.aa.is_local_object(self.f, ptr);
        let start = mem.walk.block_of(i)?;
        let first = mem.pos[i.index()] as usize + 1;
        let m = self.sh.module;
        let mut budget = DSE_BUDGET;
        let mut work: Vec<(BlockId, usize)> = vec![(start, first)];
        let mut entered: crate::fx::FxHashSet<BlockId> = Default::default();
        while let Some((b, from)) = work.pop() {
            let insts = &self.f.blocks[b.index()].insts;
            let mut ended = false;
            for &x in insts.iter().skip(from) {
                if self.erased[x.index()] {
                    continue;
                }
                // Back round a loop to the store itself: it overwrites.
                if x == i {
                    ended = true;
                    break;
                }
                if budget == 0 {
                    return None;
                }
                budget -= 1;
                let inst = &self.f.insts[x.index()];
                if let Op::Store { volatile: false, atomic: None, ptr: p2, .. } = inst.op
                    && let Some(l2) = crate::memssa::location_of(&mem.aa, self.f, x)
                    && l2.size.is_some_and(|s2| s2 >= size)
                    && mem.aa.alias(
                        self.f,
                        crate::aa::Location { ptr: p2, size: None },
                        crate::aa::Location { ptr, size: None },
                    ) == crate::aa::AliasResult::MustAlias
                {
                    ended = true;
                    break;
                }
                if mem.aa.mod_ref(self.f, inst, loc).refs {
                    return None;
                }
                let unwinds = match &inst.op {
                    Op::Call(call) | Op::Invoke { call, .. } => {
                        !crate::attrs::has(m, &call.fn_attrs, "nounwind")
                            && !self
                                .sh
                                .direct_callee(call)
                                .and_then(|g| match &m.global(g).kind {
                                    GlobalKind::Function(callee) => {
                                        Some(crate::attrs::has(m, &callee.fn_attrs, "nounwind"))
                                    }
                                    _ => None,
                                })
                                .unwrap_or(false)
                    }
                    Op::Resume(_) => true,
                    _ => false,
                };
                if unwinds && !local {
                    return None;
                }
                match &inst.op {
                    Op::Ret(_) => {
                        if !local {
                            return None;
                        }
                        ended = true;
                    }
                    Op::Unreachable => ended = true,
                    op if op.is_terminator() => {
                        for s in op.successors() {
                            if entered.insert(s) {
                                work.push((s, 0));
                            }
                        }
                        ended = true;
                    }
                    _ => {}
                }
                if ended {
                    break;
                }
            }
            if !ended {
                // A block without a terminator: not expected; be careful.
                return None;
            }
        }
        if self.allowed("(dead store)", i) { Some("(dead store)") } else { None }
    }

    /// A value's place in the canonical order reassociation sorts by:
    /// parameters first, in order; then instructions, in the order they were
    /// made; constants last, so that they gather at the outside of a chain,
    /// where the constant rules fold them together.
    fn rank(&self, v: Value) -> u64 {
        match v {
            Value::Local(l) => match self.f.locals[l.index()].def {
                LocalDef::Param(k) => k as u64,
                LocalDef::Inst(i) => self.f.params.len() as u64 + 1 + i.0 as u64,
                _ => self.f.params.len() as u64,
            },
            _ => u64::MAX,
        }
    }

    /// An instruction, for autochop's descriptions: its result's name, or
    /// its number.
    fn what(&self, i: InstId) -> String {
        epitome_ir::printer::inst_name(self.f, i)
    }

    /// Asks autochop whether this step may be made (`epitome_ir::chop`).
    fn allowed(&self, what: &str, i: InstId) -> bool {
        epitome_ir::chop::allow(|| format!("combine {what} at {}", self.what(i)))
    }

    fn visit(&mut self, i: InstId) {
        let inst = &self.f.insts[i.index()];
        if let Some(r) = inst.result
            && self.uses[r.index()].is_empty()
            && is_pure(&inst.op)
            && self.allowed("(dead)", i)
        {
            self.erase(i);
            self.stats.hit("(dead)");
            return;
        }
        if self.canonicalize(i) {
            self.stats.hit("(constant to the right)");
        }
        if let Some(v) = self.fold(i)
            && self.allowed("(folded)", i)
        {
            self.stats.hit("(folded)");
            self.replace(i, v);
            return;
        }
        if inst_has_result(self.f, i)
            && let Some((name, v)) = self.apply_rules(i)
        {
            self.stats.hit(&name);
            self.replace(i, v);
            return;
        }
        if let Some(v) = self.known_bits(i)
            && self.allowed("(known bits)", i)
        {
            self.stats.hit("(known bits)");
            self.replace(i, v);
            return;
        }
        if let Some(v) = self.funnel(i) {
            self.stats.hit("(funnel shift)");
            self.replace(i, v);
            return;
        }
        match self.extract_insert(i) {
            Some(Ok(v)) if self.allowed("(extract of insert)", i) => {
                self.stats.hit("(extract of insert)");
                self.replace(i, v);
                return;
            }
            Some(Err(agg)) if self.allowed("(extract below inserts)", i) => {
                self.stats.hit("(extract below inserts)");
                self.set_operand(i, agg);
                return;
            }
            _ => {}
        }
        if self.demanded(i) {
            self.stats.hit("(demanded bits)");
            return;
        }
        if let Some(v) = self.linear(i) {
            self.stats.hit("(linear)");
            self.replace(i, v);
            return;
        }
        if let Some(v) = self.reassociate(i) {
            self.stats.hit("(reassociated)");
            self.replace(i, v);
            return;
        }
        if self.mem.is_some() {
            if let Some(v) = self.reuse_load(i) {
                self.stats.hit("(load reused)");
                self.replace(i, v);
                return;
            }
            if let Some(what) = self.dead_store(i) {
                self.stats.hit(what);
                self.erase(i);
                if let Some(mem) = self.mem.as_mut() {
                    mem.walk.remove(i);
                }
                return;
            }
        }
        if self.f.insts[i.index()].op.is_terminator() {
            if let Some(new) = self.fold_switch(i) {
                if self.allowed("(switch folded)", i) {
                    self.stats.hit("(switch folded)");
                    self.retarget(i, new);
                }
            } else if let Some((name, new)) = self.apply_branch_rules(i)
                && self.allowed(&name, i)
            {
                self.stats.hit(&name);
                self.retarget(i, new);
            }
        }
    }

    /// A `switch` on a constant, or whose targets are all the same block,
    /// as a plain branch. Written here rather than as a rule because a
    /// switch has any number of operands.
    fn fold_switch(&self, i: InstId) -> Option<Op> {
        let Op::Switch { value, default, cases } = &self.f.insts[i.index()].op else { return None };
        if cases.iter().all(|(_, b)| b == default) {
            return Some(Op::Br(*default));
        }
        let Value::Const(c) = value else { return None };
        let v = self.int(*c)?;
        let target =
            cases.iter().find(|(k, _)| self.int(*k).is_some_and(|k| k.value == v.value)).map_or(*default, |(_, b)| *b);
        Some(Op::Br(target))
    }

    /// Tries the branch rules matching terminator `i`.
    fn apply_branch_rules(&mut self, i: InstId) -> Option<(String, Op)> {
        let (head, ops) = head_of(self.f, &self.f.insts[i.index()].op)?;
        let matches = {
            let consts = self.sh.consts();
            let types = self.sh.types();
            let view = View { f: self.f, erased: &self.erased, consts, types };
            self.matcher.find_op(&view, head, ops)
        };
        for m in matches {
            if let Some(op) = self.try_branch_match(i, &m) {
                let rule = &self.matcher.rules[self.matcher.variants[m.variant].rule];
                return Some((rule.name.clone(), op));
            }
        }
        None
    }

    fn try_branch_match(&mut self, i: InstId, m: &Match) -> Option<Op> {
        let variant = &self.matcher.variants[m.variant];
        let rule = &self.matcher.rules[variant.rule];
        let mut bound: Vec<Option<Value>> = vec![None; rule.vars.len()];
        for (slot, &v) in variant.slots.iter().zip(&m.captures) {
            match bound[*slot] {
                Some(prev) if prev != v => return None,
                _ => bound[*slot] = Some(v),
            }
        }
        let consts: Vec<Option<IntConst>> = {
            let (cs, ts) = (self.sh.consts(), self.sh.types());
            bound
                .iter()
                .map(|b| match b {
                    Some(Value::Const(c)) => eval::int_const(cs, ts, *c),
                    _ => None,
                })
                .collect()
        };
        let widths = vec![None; rule.vars.len()];
        let env = Env { consts: &consts, widths: &widths, bw: None };
        let same_type = |_: Who, _: Who| false;
        if !rule.conds.iter().all(|c| eval::check(c, &env, &same_type)) {
            return None;
        }
        if rule.conds.contains(&Cond::OneUse)
            && m.interior.iter().any(|v| matches!(v, Value::Local(l) if self.uses[l.index()].len() != 1))
        {
            return None;
        }
        // The new branch replaces the old; anything else it makes must be
        // paid for by instructions the match leaves unused.
        let made = count_ops(&rule.repl) - 1;
        let freed =
            m.interior.iter().filter(|v| matches!(v, Value::Local(l) if self.uses[l.index()].len() == 1)).count();
        if made > freed {
            return None;
        }
        let Repl::Op(head, args) = &rule.repl else { return None };
        let block = |v: &Repl| -> Option<BlockId> {
            let Repl::Block(k) = v else { return None };
            match bound[*k]? {
                Value::Local(l) => match self.f.locals[l.index()].def {
                    LocalDef::Block(b) => Some(b),
                    _ => None,
                },
                _ => None,
            }
        };
        match head {
            Head::Br => Some(Op::Br(block(&args[0])?)),
            Head::CondBr => {
                let (then_dest, else_dest) = (block(&args[1])?, block(&args[2])?);
                let i1 = self.sh.intern_type(Type::Int(1));
                let mut created = Vec::new();
                match self.build(&args[0], &bound, &env, i1, i, &mut created) {
                    Some(cond) => {
                        for c in created {
                            self.push(c);
                        }
                        Some(Op::CondBr { cond, then_dest, else_dest })
                    }
                    None => {
                        for c in created.into_iter().rev() {
                            self.erase(c);
                        }
                        None
                    }
                }
            }
            _ => None,
        }
    }

    /// Replaces terminator `i` with `new`. Each edge the new terminator no
    /// longer has takes one entry out of the target's phis, since phis
    /// have one entry per edge; the phis go back on the list, as they may
    /// now fold.
    fn retarget(&mut self, i: InstId, new: Op) {
        let b = self.block_of[i.index()].expect("terminator not in a block");
        let old_succs = self.f.insts[i.index()].op.successors();
        let mut removed = old_succs.clone();
        for s in new.successors() {
            if let Some(k) = removed.iter().position(|&x| x == s) {
                removed.remove(k);
            }
        }
        // Operand uses: the old ones go, the new ones come.
        let mut old_ops = Vec::new();
        self.f.insts[i.index()].op.for_each_value(|v| old_ops.push(*v));
        for v in old_ops {
            if let Value::Local(l) = v {
                let list = &mut self.uses[l.index()];
                if let Some(k) = list.iter().position(|&x| x == i) {
                    list.swap_remove(k);
                }
                self.push_def(v);
            }
        }
        self.f.insts[i.index()].op = new;
        self.add_uses(i);
        for s in removed {
            for k in 0..self.f.blocks[s.index()].insts.len() {
                let p = self.f.blocks[s.index()].insts[k];
                let Op::Phi { incoming, .. } = &mut self.f.insts[p.index()].op else { break };
                if let Some(at) = incoming.iter().position(|(_, from)| *from == b) {
                    let (v, _) = incoming.remove(at);
                    if let Value::Local(l) = v {
                        let list = &mut self.uses[l.index()];
                        if let Some(k) = list.iter().position(|&x| x == p) {
                            list.swap_remove(k);
                        }
                    }
                    self.push_def(v);
                }
            }
        }
        let mut succs = old_succs;
        succs.dedup();
        for s in succs {
            let phis: Vec<InstId> = self.f.blocks[s.index()]
                .insts
                .iter()
                .copied()
                .take_while(|&p| self.f.insts[p.index()].op.is_phi())
                .collect();
            for p in phis {
                self.push(p);
            }
        }
        self.stats.cfg_changed = true;
    }

    /// Moves a constant left operand of a commutative operation or a
    /// comparison to the right.
    fn canonicalize(&mut self, i: InstId) -> bool {
        let commutative = |o: BinOp| {
            matches!(o, BinOp::Add | BinOp::Mul | BinOp::And | BinOp::Or | BinOp::Xor | BinOp::FAdd | BinOp::FMul)
        };
        let swaps = match &self.f.insts[i.index()].op {
            Op::Binary { op, lhs, rhs, .. } => {
                commutative(*op) && matches!(lhs, Value::Const(_)) && !matches!(rhs, Value::Const(_))
            }
            Op::ICmp { lhs, rhs, .. } => matches!(lhs, Value::Const(_)) && !matches!(rhs, Value::Const(_)),
            _ => false,
        };
        if !swaps || !self.allowed("(constant to the right)", i) {
            return false;
        }
        let op = &mut self.f.insts[i.index()].op;
        match op {
            Op::Binary { op, lhs, rhs, .. }
                if commutative(*op) && matches!(lhs, Value::Const(_)) && !matches!(rhs, Value::Const(_)) =>
            {
                std::mem::swap(lhs, rhs);
                true
            }
            Op::ICmp { pred, lhs, rhs, .. } if matches!(lhs, Value::Const(_)) && !matches!(rhs, Value::Const(_)) => {
                std::mem::swap(lhs, rhs);
                *pred = eval::swapped(*pred);
                true
            }
            _ => false,
        }
    }

    /// The value of an instruction whose result is known without running
    /// it: all-constant operands, poison operands, or a phi of one value.
    fn fold(&mut self, i: InstId) -> Option<Value> {
        let sh = self.sh;
        let inst = &self.f.insts[i.index()];
        let r = inst.result?;
        let ty = self.f.locals[r.index()].ty;
        let op = inst.op.clone();
        let is_poison = |v: Value| matches!(v, Value::Const(c) if matches!(sh.consts().get(c).kind, ConstKind::Poison));
        match op {
            // A load from a constant global reads its initialiser.
            Op::Load { volatile: false, atomic: None, ptr, .. } => constload::load_constant(sh, self.f, ptr, ty),
            Op::Phi { incoming, .. } => {
                let me = Value::Local(r);
                let mut same = None;
                for (v, _) in incoming {
                    if v == me || Some(v) == same {
                        continue;
                    }
                    if same.is_some() {
                        return None;
                    }
                    same = Some(v);
                }
                same
            }
            Op::Binary { op, flags, fmf, lhs, rhs } => {
                if is_poison(lhs) || is_poison(rhs) {
                    return Some(sh.poison(ty));
                }
                let (Value::Const(a), Value::Const(b)) = (lhs, rhs) else { return None };
                if op.is_float() {
                    if !fmf.is_empty() {
                        return None;
                    }
                    let (consts, types) = (sh.consts(), sh.types());
                    let (ConstKind::Float(x), ConstKind::Float(y)) = (&consts.get(a).kind, &consts.get(b).kind) else {
                        return None;
                    };
                    let kind = FloatKind::of(types.get(ty))?;
                    let bits = eval::fold_float(op, kind, *x, *y)?;
                    return Some(Value::Const(sh.intern_const(Constant { ty, kind: ConstKind::Float(bits) })));
                }
                let (x, y) = (self.int(a)?, self.int(b)?);
                match eval::fold_binop(op, flags, x.value, y.value, x.bits)? {
                    Folded::Int(v) => self.int_value(ty, v),
                    Folded::Poison => Some(sh.poison(ty)),
                }
            }
            Op::ICmp { pred, lhs, rhs, .. } => {
                if is_poison(lhs) || is_poison(rhs) {
                    return Some(sh.poison(ty));
                }
                let (Value::Const(a), Value::Const(b)) = (lhs, rhs) else { return None };
                let (x, y) = (self.int(a)?, self.int(b)?);
                self.int_value(ty, eval::fold_icmp(pred, x.value, y.value, x.bits) as u64)
            }
            Op::Select { cond, then_value, else_value, .. } => {
                if is_poison(cond) {
                    return Some(sh.poison(ty));
                }
                let Value::Const(c) = cond else { return None };
                let k = self.int(c)?;
                Some(if k.value != 0 { then_value } else { else_value })
            }
            Op::Cast { op, flags, value, .. } => {
                if is_poison(value) {
                    return Some(sh.poison(ty));
                }
                let Value::Const(c) = value else { return None };
                // A float's bits as an integer of its width: the raw bits.
                if op == CastOp::BitCast
                    && let ConstKind::Float(raw) = sh.consts().get(c).kind
                    && let Some(w) = eval::int_bits(sh.types(), ty).filter(|&w| w <= 64)
                    && matches!(sh.types().get(ty), Type::Int(_))
                    && FloatKind::of(sh.types().get(sh.consts().get(c).ty)).is_some_and(|k| k.bits() == w)
                {
                    return self.int_value(ty, raw as u64);
                }
                let x = self.int(c)?;
                let to = eval::int_bits(sh.types(), ty)?;
                // Wider than 64 bits, a sign extension's sign goes on past
                // bit 63, which the 64-bit fold does not carry.
                if op == CastOp::SExt && to > 64 && x.bits <= 64 {
                    let v = eval::sext(x.value & eval::mask(x.bits), x.bits);
                    return self.int_value_signed(ty, v);
                }
                match eval::fold_cast(op, flags, x.value, x.bits, to)? {
                    Folded::Int(v) => self.int_value(ty, v),
                    Folded::Poison => Some(sh.poison(ty)),
                }
            }
            Op::FNeg { fmf, value: Value::Const(c) } if fmf.is_empty() => {
                let bits = {
                    let consts = sh.consts();
                    let types = sh.types();
                    let ConstKind::Float(b) = consts.get(c).kind else { return None };
                    let kind = FloatKind::of(types.get(ty))?;
                    let sign = match kind {
                        FloatKind::Half | FloatKind::BFloat => 15,
                        FloatKind::Float => 31,
                        FloatKind::Double => 63,
                        _ => return None,
                    };
                    b ^ (1u128 << sign)
                };
                Some(Value::Const(sh.intern_const(Constant { ty, kind: ConstKind::Float(bits) })))
            }
            Op::Freeze(Value::Const(c)) => {
                let consts = sh.consts();
                let k = &consts.get(c).kind;
                matches!(k, ConstKind::Int(_) | ConstKind::Float(_) | ConstKind::Zero | ConstKind::Null)
                    .then_some(Value::Const(c))
            }
            _ => None,
        }
    }

    fn int(&self, c: ConstId) -> Option<IntConst> {
        eval::int_const(self.sh.consts(), self.sh.types(), c)
    }

    /// An integer constant of type `ty` (a splat for a vector).
    fn int_value(&self, ty: TypeId, v: u64) -> Option<Value> {
        let sh = self.sh;
        let (bits, elem, vector) = {
            let types = sh.types();
            let elem = types.scalar(ty);
            let Type::Int(bits) = *types.get(elem) else { return None };
            (bits, elem, elem != ty)
        };
        let k =
            sh.intern_const(Constant { ty: elem, kind: ConstKind::Int(ApInt::from_u64(bits, v & eval::mask(bits))) });
        Some(Value::Const(if vector { sh.intern_const(Constant { ty, kind: ConstKind::Splat(k) }) } else { k }))
    }

    /// A signed integer constant of type `ty` (a splat for a vector), sign
    /// extended to its width however wide.
    fn int_value_signed(&self, ty: TypeId, v: i64) -> Option<Value> {
        let sh = self.sh;
        let (bits, elem, vector) = {
            let types = sh.types();
            let elem = types.scalar(ty);
            let Type::Int(bits) = *types.get(elem) else { return None };
            (bits, elem, elem != ty)
        };
        let k = sh.intern_const(Constant { ty: elem, kind: ConstKind::Int(ApInt::from_i64(bits, v)) });
        Some(Value::Const(if vector { sh.intern_const(Constant { ty, kind: ConstKind::Splat(k) }) } else { k }))
    }

    fn float_value(&self, ty: TypeId, lit: FloatLit) -> Option<Value> {
        let sh = self.sh;
        let (elem, kind, vector) = {
            let types = sh.types();
            let elem = types.scalar(ty);
            (elem, FloatKind::of(types.get(elem))?, elem != ty)
        };
        let v = match lit {
            FloatLit::PosZero => 0.0,
            FloatLit::NegZero => -0.0,
            FloatLit::One => 1.0,
        };
        let bits = epitome_ir::float::from_f64(kind, v)?;
        let k = sh.intern_const(Constant { ty: elem, kind: ConstKind::Float(bits) });
        Some(Value::Const(if vector { sh.intern_const(Constant { ty, kind: ConstKind::Splat(k) }) } else { k }))
    }

    /// Tries the rules matching `i`, best first, and builds the
    /// replacement of the first that applies. Returns the rule's name and
    /// the replacement value.
    fn apply_rules(&mut self, i: InstId) -> Option<(String, Value)> {
        let root = self.f.result(i);
        let matches = {
            let consts = self.sh.consts();
            let types = self.sh.types();
            let view = View { f: self.f, erased: &self.erased, consts, types };
            self.matcher.find(&view, root)
        };
        for m in matches {
            if let Some(v) = self.try_match(i, &m) {
                let rule = &self.matcher.rules[self.matcher.variants[m.variant].rule];
                return Some((rule.name.clone(), v));
            }
        }
        None
    }

    fn try_match(&mut self, i: InstId, m: &Match) -> Option<Value> {
        let variant = &self.matcher.variants[m.variant];
        let rule = &self.matcher.rules[variant.rule];
        // Bind the variables; a repeated variable must see the same value.
        let mut bound: Vec<Option<Value>> = vec![None; rule.vars.len()];
        for (slot, &v) in variant.slots.iter().zip(&m.captures) {
            match bound[*slot] {
                Some(prev) if prev != v => return None,
                _ => bound[*slot] = Some(v),
            }
        }
        // A memory rule: the write that reaches the load, as the walker
        // finds it, must be the store the rule names.
        if let Some(dsl::Pat::Op(Head::Store, args)) = &rule.clobber {
            let mem = self.mem.as_ref()?;
            let c = mem.walk.clobber_of(&mem.ssa, &mem.aa, self.f, i)?;
            let crate::memssa::Access::Def { inst: w, .. } = *mem.ssa.get(c.at) else { return None };
            if !c.exact || self.erased[w.index()] {
                return None;
            }
            let Op::Store { volatile: false, atomic: None, value, ptr, .. } = self.f.insts[w.index()].op else {
                return None;
            };
            for (a, v) in args.iter().zip([value, ptr]) {
                let (dsl::Pat::Var(k) | dsl::Pat::Const(k)) = *a else { return None };
                if matches!(a, dsl::Pat::Const(_)) && !matches!(v, Value::Const(_)) {
                    return None;
                }
                match bound[k] {
                    Some(prev) if prev != v => return None,
                    _ => bound[k] = Some(v),
                }
            }
        }
        for c in &rule.conds {
            if let Cond::MustAlias(Who::Var(a), Who::Var(b)) = *c {
                let mem = self.mem.as_ref()?;
                let loc = |v: Value| crate::aa::Location { ptr: v, size: None };
                if mem.aa.alias(self.f, loc(bound[a]?), loc(bound[b]?)) != crate::aa::AliasResult::MustAlias {
                    return None;
                }
            }
        }
        // Constants and widths for the expressions.
        let root_ty = self.f.locals[self.f.insts[i.index()].result?.index()].ty;
        let (consts, widths, bw) = {
            let cs = self.sh.consts();
            let ts = self.sh.types();
            let consts: Vec<Option<IntConst>> = bound
                .iter()
                .map(|b| match b {
                    Some(Value::Const(c)) => eval::int_const(cs, ts, *c),
                    _ => None,
                })
                .collect();
            let ty_of = |v: Value| match v {
                Value::Local(l) => Some(self.f.locals[l.index()].ty),
                Value::Const(c) => Some(cs.get(c).ty),
                Value::Metadata(_) => None,
            };
            let widths: Vec<Option<u32>> =
                bound.iter().map(|b| b.and_then(ty_of).and_then(|t| eval::int_bits(ts, t))).collect();
            // For a comparison, `bw` is the width of what it compares.
            let bw = match &self.f.insts[i.index()].op {
                Op::ICmp { lhs, .. } => ty_of(*lhs).and_then(|t| eval::int_bits(ts, t)),
                _ => eval::int_bits(ts, root_ty),
            };
            (consts, widths, bw)
        };
        let env = Env { consts: &consts, widths: &widths, bw };
        let type_of = |w: Who| -> Option<TypeId> {
            match w {
                Who::Root => Some(root_ty),
                Who::Var(v) => bound[v].and_then(|x| self.value_type(x)),
            }
        };
        let same_type = |a: Who, b: Who| type_of(a).is_some() && type_of(a) == type_of(b);
        if !rule.conds.iter().all(|c| eval::check(c, &env, &same_type)) {
            return None;
        }
        if rule.conds.contains(&Cond::OneUse)
            && m.interior.iter().any(|v| matches!(v, Value::Local(l) if self.uses[l.index()].len() != 1))
        {
            return None;
        }
        // Worth it only if no more instructions are made than become unused.
        let made = count_ops(&rule.repl);
        let freed =
            1 + m.interior.iter().filter(|v| matches!(v, Value::Local(l) if self.uses[l.index()].len() == 1)).count();
        if made > freed {
            return None;
        }
        let mut created = Vec::new();
        let result = self.build(&rule.repl, &bound, &env, root_ty, i, &mut created);
        // Autochop: asked once the replacement is built and checked, so
        // that every step counted is a change made; if refused, it is
        // undone as one that does not type-check is.
        let allowed = matches!(result, Some(v) if self.value_type(v) == Some(root_ty))
            && epitome_ir::chop::allow(|| format!("combine {} at {}", rule.name, self.what(i)));
        match result {
            Some(v) if allowed => {
                // The new instruction in the root's place takes its name.
                if let (Some(&last), Some(r)) = (created.last(), self.f.insts[i.index()].result)
                    && self.f.result(last) == v
                {
                    let name = self.f.locals[r.index()].name.take();
                    let new = self.f.insts[last.index()].result.unwrap();
                    self.f.locals[new.index()].name = name;
                }
                for c in created {
                    self.push(c);
                }
                Some(v)
            }
            _ => {
                // The replacement did not type-check: undo what was built.
                for c in created.into_iter().rev() {
                    self.erase(c);
                }
                None
            }
        }
    }

    /// Builds a replacement of type `ty`, placing new instructions before
    /// `root`.
    fn build(
        &mut self,
        r: &Repl,
        bound: &[Option<Value>],
        env: &Env,
        ty: TypeId,
        root: InstId,
        created: &mut Vec<InstId>,
    ) -> Option<Value> {
        let sh = self.sh;
        match r {
            Repl::Var(v) => {
                let x = bound[*v]?;
                (self.value_type(x) == Some(ty)).then_some(x)
            }
            Repl::Block(_) => None,
            Repl::Lit(Lit::Poison) => Some(sh.poison(ty)),
            Repl::Lit(Lit::Float(f)) => self.float_value(ty, *f),
            Repl::Lit(Lit::Int(n)) => self.int_value(ty, *n as u64),
            Repl::Const(e) => {
                let v = eval::eval(e, env)?;
                self.int_value(ty, v as u64)
            }
            Repl::Op(head, args) => {
                let i1 = sh.intern_type(Type::Int(1));
                let shape = sh.ty(ty);
                let bool_ty = match shape {
                    Type::Vector { scalable, len, .. } => sh.intern_type(Type::Vector { scalable, len, elem: i1 }),
                    _ => i1,
                };
                // The type each operand must have.
                let var_ty = |a: &Repl| match a {
                    Repl::Var(v) => bound[*v].and_then(|x| self.value_type(x)),
                    _ => None,
                };
                let operand_tys: Vec<TypeId> = match head {
                    Head::Bin(_) | Head::FNeg | Head::Freeze => vec![ty; head.arity()],
                    Head::Select => vec![bool_ty, ty, ty],
                    Head::ICmp(_) => {
                        let t = args.iter().find_map(var_ty)?;
                        vec![t, t]
                    }
                    Head::Cast(_) => vec![var_ty(&args[0])?],
                    Head::Br | Head::CondBr | Head::Load | Head::Store => return None,
                };
                let mut ops = Vec::new();
                for (a, &t) in args.iter().zip(&operand_tys) {
                    ops.push(self.build(a, bound, env, t, root, created)?);
                }
                let op = match *head {
                    Head::Bin(op) => {
                        Op::Binary { op, flags: IntFlags::NONE, fmf: FastMath::default(), lhs: ops[0], rhs: ops[1] }
                    }
                    Head::ICmp(pred) => Op::ICmp { pred, samesign: false, lhs: ops[0], rhs: ops[1] },
                    Head::Select => {
                        Op::Select { fmf: FastMath::default(), cond: ops[0], then_value: ops[1], else_value: ops[2] }
                    }
                    Head::Cast(op) => Op::Cast { op, flags: IntFlags::NONE, fmf: FastMath::default(), value: ops[0] },
                    Head::FNeg => Op::FNeg { fmf: FastMath::default(), value: ops[0] },
                    Head::Freeze => Op::Freeze(ops[0]),
                    Head::Br | Head::CondBr | Head::Load | Head::Store => {
                        unreachable!("branches are built by try_branch_match; loads and stores never")
                    }
                };
                let new = self.f.add_inst(op, Some((None, ty)));
                self.grow();
                self.block_of[new.index()] = self.block_of[root.index()];
                self.before.entry(root).or_default().push(new);
                self.add_uses(new);
                created.push(new);
                Some(self.f.result(new))
            }
        }
    }

    /// Replaces every use of `i`'s result with `v`, then erases `i`.
    fn replace(&mut self, i: InstId, v: Value) {
        let r = self.f.insts[i.index()].result.unwrap();
        let from = Value::Local(r);
        let users = std::mem::take(&mut self.uses[r.index()]);
        let mut done = Vec::new();
        for u in users {
            if done.contains(&u) {
                continue;
            }
            done.push(u);
            let mut n = 0;
            self.f.insts[u.index()].op.for_each_value_mut(|x| {
                if *x == from {
                    *x = v;
                    n += 1;
                }
            });
            if let Value::Local(l) = v {
                for _ in 0..n {
                    self.uses[l.index()].push(u);
                }
            }
            self.push(u);
        }
        self.replaced.insert(r, v);
        self.erase(i);
    }

    /// Removes an instruction. Its operands lose a use, and go back on the
    /// list in case that leaves them unused.
    fn erase(&mut self, i: InstId) {
        if self.erased[i.index()] {
            return;
        }
        self.erased[i.index()] = true;
        let mut operands = Vec::new();
        self.f.insts[i.index()].op.for_each_value(|v| operands.push(*v));
        for v in operands {
            if let Value::Local(l) = v {
                let list = &mut self.uses[l.index()];
                if let Some(k) = list.iter().position(|&x| x == i) {
                    list.swap_remove(k);
                }
                self.push_def(v);
            }
        }
    }

    /// Rebuilds the blocks with new instructions in place and erased ones
    /// gone, and points debug records at the values that replaced theirs.
    fn finish(&mut self) {
        for b in self.f.layout.clone() {
            let old = std::mem::take(&mut self.f.blocks[b.index()].insts);
            let mut new = Vec::with_capacity(old.len());
            for i in old {
                self.place(i, &mut new);
            }
            self.f.blocks[b.index()].insts = new;
        }
        self.fix_debug_records();
    }

    fn fix_debug_records(&mut self) {
        let sh = self.sh;
        let resolve = |mut v: Value| {
            for _ in 0..1000 {
                match v {
                    Value::Local(l) => match self.replaced.get(&l) {
                        Some(&next) => v = next,
                        None => break,
                    },
                    _ => break,
                }
            }
            v
        };
        let dead = |l: LocalId| matches!(self.f.locals[l.index()].def, LocalDef::Inst(d) if self.erased[d.index()]);
        let fix = |v: Value| -> Value {
            let v = resolve(v);
            match v {
                Value::Local(l) if dead(l) => sh.poison(self.f.locals[l.index()].ty),
                v => v,
            }
        };
        let mut changes: Vec<(InstId, usize, usize, MdId)> = Vec::new();
        for i in self.f.insts_in_order() {
            for (k, r) in self.f.insts[i.index()].debug_records.iter().enumerate() {
                for (j, &arg) in r.args.iter().enumerate() {
                    let data = sh.metadata()[arg.index()].data.clone();
                    let new = match data {
                        Metadata::Value(v @ Value::Local(_)) if fix(v) != v => Some(Metadata::Value(fix(v))),
                        Metadata::Node { distinct, kind, fields }
                            if fields.iter().any(|x| matches!(x.value, MdFieldValue::Value(v) if fix(v) != v)) =>
                        {
                            let fields = fields
                                .into_iter()
                                .map(|x| match x.value {
                                    MdFieldValue::Value(v) => {
                                        MdField { name: x.name, value: MdFieldValue::Value(fix(v)) }
                                    }
                                    _ => x,
                                })
                                .collect();
                            Some(Metadata::Node { distinct, kind, fields })
                        }
                        _ => None,
                    };
                    if let Some(data) = new {
                        changes.push((i, k, j, sh.add_md(data)));
                    }
                }
            }
        }
        for (i, k, j, md) in changes {
            self.f.insts[i.index()].debug_records[k].args[j] = md;
        }
    }

    fn place(&mut self, i: InstId, out: &mut Vec<InstId>) {
        if let Some(list) = self.before.remove(&i) {
            for n in list {
                self.place(n, out);
            }
        }
        if !self.erased[i.index()] {
            out.push(i);
        } else if !self.f.insts[i.index()].debug_records.is_empty() {
            // The records of an erased instruction move to the next one.
            let records = std::mem::take(&mut self.f.insts[i.index()].debug_records);
            self.pending_records.extend(records);
            return;
        }
        if !self.erased[i.index()] && !self.pending_records.is_empty() {
            let mut records = std::mem::take(&mut self.pending_records);
            records.append(&mut self.f.insts[i.index()].debug_records);
            self.f.insts[i.index()].debug_records = records;
        }
    }
}

/// The number of instructions a replacement makes.
fn count_ops(r: &Repl) -> usize {
    match r {
        Repl::Op(_, args) => 1 + args.iter().map(count_ops).sum::<usize>(),
        _ => 0,
    }
}
