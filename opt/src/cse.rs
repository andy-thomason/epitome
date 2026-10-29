//! Common-subexpression elimination: an instruction that computes what a
//! dominating instruction already computed is replaced by it.
//!
//! Each instruction gets a key that equal computations share: its opcode,
//! the attributes that change its result (predicate, cast kind, indices),
//! its result type, and its operands. Operands are looked up through the
//! replacements made so far, so equivalence carries through chains: once
//! `%b` is known to be `%a`, `add %b, 1` has the same key as `add %a, 1`.
//! Keys are put in a canonical form, so that
//!
//! - commutative operations sort their operands: `a + b` and `b + a` match;
//! - comparisons put the smaller operand first and swap the predicate to
//!   suit: `icmp sgt %x, %y` and `icmp slt %y, %x` match.
//!
//! Flags such as `nsw`, `exact`, `inbounds` and fast-math are not part of
//! the key. When two instructions merge, the one kept keeps only the flags
//! both had: its own stronger flags might make poison where the other did
//! not.
//!
//! The table of keys is scoped by the dominator tree, walked in preorder:
//! an entry is visible only in the blocks its instruction dominates, so a
//! replacement always dominates what it replaces.
//!
//! Loads are handled too. A load can reuse an earlier load of the same
//! address and type, or the value an earlier store put there, if nothing in
//! between may have changed those bytes. The pass keeps a log of the
//! instructions on the current dominator path that may write memory, and
//! asks alias analysis (`aa`) whether any written since the value was
//! remembered may modify the load's location. Entering a block that can be
//! reached other than from its dominator adds an "unknown" entry to the
//! log, which nothing gets past, since another path in may have written.
//!
//! `freeze` is never merged: two freezes of the same poison may give
//! different values.

use crate::fx::FxHashMap as HashMap;
use std::hash::Hash;

use epitome_ir::module::*;
use epitome_ir::types::TypeId;

use crate::aa::{BasicAa, Location};
use crate::cfg::DomTree;
use crate::ctx::Shared;

#[derive(Default, Debug, Clone, Copy)]
pub struct Stats {
    /// Instructions replaced by an earlier equal one.
    pub merged: usize,
    /// Loads replaced by an earlier load.
    pub loads: usize,
    /// Loads replaced by the value an earlier store wrote.
    pub forwarded: usize,
}

/// What `run` did: counts, and the instructions whose operands it changed,
/// which are the only ones that can now combine further.
pub struct Outcome {
    pub stats: Stats,
    pub changed: Vec<InstId>,
}

impl Stats {
    pub fn total(&self) -> usize {
        self.merged + self.loads + self.forwarded
    }
}

/// What makes two instructions compute the same value.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
enum Key {
    Bin(BinOp, TypeId, Value, Value),
    ICmp(ICmpPred, TypeId, Value, Value),
    FCmp(FCmpPred, TypeId, Value, Value),
    Select(TypeId, Value, Value, Value),
    Cast(CastOp, TypeId, Value),
    FNeg(TypeId, Value),
    Gep {
        source: TypeId,
        ty: TypeId,
        operands: Vec<Value>,
    },
    ExtractValue(TypeId, Value, Box<[u32]>),
    InsertValue(TypeId, Value, Value, Box<[u32]>),
    ExtractElement(TypeId, Value, Value),
    InsertElement(TypeId, Value, Value, Value),
    Shuffle(TypeId, Value, Value, ConstId),
    BitExtract(TypeId, Value, Value),
    BitInsert(TypeId, Value, Value, Value),
    /// Phis in the same block with the same entries.
    Phi(BlockId, TypeId, Vec<(BlockId, Value)>),
}

/// An order on values, for putting operands in canonical order.
fn rank(v: Value) -> (u8, u32) {
    match v {
        Value::Local(l) => (0, l.0),
        Value::Const(c) => (1, c.0),
        Value::Metadata(m) => (2, m.0),
    }
}

fn commutative(op: BinOp) -> bool {
    matches!(op, BinOp::Add | BinOp::Mul | BinOp::And | BinOp::Or | BinOp::Xor | BinOp::FAdd | BinOp::FMul)
}

fn swapped_fcmp(p: FCmpPred) -> FCmpPred {
    use FCmpPred::*;
    match p {
        Ogt => Olt,
        Oge => Ole,
        Olt => Ogt,
        Ole => Oge,
        Ugt => Ult,
        Uge => Ule,
        Ult => Ugt,
        Ule => Uge,
        p => p,
    }
}

/// The key of an instruction, or `None` if it must not be merged.
fn key(op: &Op, ty: TypeId, block: BlockId) -> Option<Key> {
    Some(match op {
        Op::Binary { op, lhs, rhs, .. } => {
            let (mut a, mut b) = (*lhs, *rhs);
            if commutative(*op) && rank(b) < rank(a) {
                std::mem::swap(&mut a, &mut b);
            }
            Key::Bin(*op, ty, a, b)
        }
        Op::ICmp { pred, lhs, rhs, .. } => {
            if rank(*rhs) < rank(*lhs) {
                Key::ICmp(crate::combine::eval::swapped(*pred), ty, *rhs, *lhs)
            } else {
                Key::ICmp(*pred, ty, *lhs, *rhs)
            }
        }
        Op::FCmp { pred, lhs, rhs, .. } => {
            if rank(*rhs) < rank(*lhs) {
                Key::FCmp(swapped_fcmp(*pred), ty, *rhs, *lhs)
            } else {
                Key::FCmp(*pred, ty, *lhs, *rhs)
            }
        }
        Op::Select { cond, then_value, else_value, .. } => Key::Select(ty, *cond, *then_value, *else_value),
        Op::Cast { op, value, .. } => Key::Cast(*op, ty, *value),
        Op::FNeg { value, .. } => Key::FNeg(ty, *value),
        Op::Gep { source_ty, ptr, indices, .. } => Key::Gep {
            source: *source_ty,
            ty,
            operands: std::iter::once(*ptr).chain(indices.iter().copied()).collect(),
        },
        Op::ExtractValue { aggregate, indices } => Key::ExtractValue(ty, *aggregate, indices.clone()),
        Op::InsertValue { aggregate, element, indices } => Key::InsertValue(ty, *aggregate, *element, indices.clone()),
        Op::ExtractElement { vector, index } => Key::ExtractElement(ty, *vector, *index),
        Op::InsertElement { vector, element, index } => Key::InsertElement(ty, *vector, *element, *index),
        Op::ShuffleVector { a, b, mask } => Key::Shuffle(ty, *a, *b, *mask),
        Op::BitExtract { value, offset } => Key::BitExtract(ty, *value, *offset),
        Op::BitInsert { base, value, offset } => Key::BitInsert(ty, *base, *value, *offset),
        Op::Phi { incoming, .. } => {
            let mut entries: Vec<(BlockId, Value)> = incoming.iter().map(|&(v, b)| (b, v)).collect();
            entries.sort_by_key(|&(b, v)| (b, rank(v)));
            Key::Phi(block, ty, entries)
        }
        _ => return None,
    })
}

/// Leaves the flags of `keep` with only those `other` has too.
fn intersect_flags(keep: &mut Op, other: &Op) {
    match (keep, other) {
        (Op::Binary { flags, fmf, .. }, Op::Binary { flags: f2, fmf: m2, .. })
        | (Op::Cast { flags, fmf, .. }, Op::Cast { flags: f2, fmf: m2, .. }) => {
            *flags = IntFlags(flags.0 & f2.0);
            *fmf = FastMath(fmf.0 & m2.0);
        }
        (Op::ICmp { samesign, .. }, Op::ICmp { samesign: s2, .. }) => *samesign &= *s2,
        (Op::FCmp { fmf, .. }, Op::FCmp { fmf: m2, .. })
        | (Op::Select { fmf, .. }, Op::Select { fmf: m2, .. })
        | (Op::FNeg { fmf, .. }, Op::FNeg { fmf: m2, .. })
        | (Op::Phi { fmf, .. }, Op::Phi { fmf: m2, .. }) => *fmf = FastMath(fmf.0 & m2.0),
        (Op::Gep { flags, .. }, Op::Gep { flags: f2, .. }) => *flags = GepFlags(flags.0 & f2.0),
        _ => {}
    }
}

/// A hash map whose insertions can be undone back to a mark.
struct Scoped<K, V> {
    map: HashMap<K, V>,
    log: Vec<(K, Option<V>)>,
}

impl<K: Hash + Eq + Clone, V> Scoped<K, V> {
    fn new() -> Self {
        Scoped { map: HashMap::default(), log: Vec::new() }
    }

    fn get(&self, k: &K) -> Option<&V> {
        self.map.get(k)
    }

    fn insert(&mut self, k: K, v: V) {
        let old = self.map.insert(k.clone(), v);
        self.log.push((k, old));
    }

    fn mark(&self) -> usize {
        self.log.len()
    }

    fn undo(&mut self, mark: usize) {
        while self.log.len() > mark {
            let (k, old) = self.log.pop().unwrap();
            match old {
                Some(v) => self.map.insert(k, v),
                None => self.map.remove(&k),
            };
        }
    }
}

/// True if the instruction may write memory, or must stay ordered with
/// memory accesses as if it did.
fn may_write(sh: &Shared, op: &Op) -> bool {
    match op {
        Op::Store { .. } | Op::AtomicRmw { .. } | Op::CmpXchg { .. } | Op::Fence { .. } | Op::VaArg { .. } => true,
        Op::Load { volatile, atomic, .. } => *volatile || atomic.is_some(),
        Op::Call(call) | Op::Invoke { call, .. } | Op::CallBr { call, .. } => !call_is_read_only(sh, call),
        _ => false,
    }
}

/// Calls that cannot write memory: `memory(none)` or `memory(read)` on the
/// call or the callee, and intrinsics that only describe the program.
fn call_is_read_only(sh: &Shared, call: &Call) -> bool {
    let m = sh.module;
    let read_only = |attrs: &[Attr]| {
        crate::attrs::has(m, attrs, "readnone")
            || crate::attrs::has(m, attrs, "readonly")
            || crate::attrs::paren_text(m, attrs, "memory").is_some_and(|t| t == "none" || t == "read")
    };
    if read_only(&call.fn_attrs) {
        return true;
    }
    let Some(g) = sh.direct_callee(call) else { return false };
    if let Some(name) = m.global(g).name.as_deref()
        && ["llvm.dbg.", "llvm.lifetime.", "llvm.assume", "llvm.experimental.noalias.scope.decl"]
            .iter()
            .any(|p| name.starts_with(p))
    {
        return true;
    }
    sh.published(g).is_some_and(|f| read_only(&f.fn_attrs))
}

/// Most writes to check before giving up on a remembered value.
const MAX_WRITES: usize = 64;

/// True if none of `writes` may change the bytes at `loc`.
fn still_there(aa: &BasicAa, f: &Function, writes: &[Option<InstId>], loc: Location) -> bool {
    writes.len() <= MAX_WRITES
        && writes.iter().all(|w| match w {
            None => false,
            Some(i) => !aa.mod_ref(f, &f.insts[i.index()], loc).mods,
        })
}

/// Runs CSE on one function.
pub fn run(sh: &Shared, f: &mut Function) -> Outcome {
    let mut stats = Stats::default();
    if !f.has_body {
        return Outcome { stats, changed: Vec::new() };
    }
    let dom = DomTree::new(f);
    let children = dom.children();
    let preds = f.predecessors();
    // What each merged value was replaced by.
    let mut map: HashMap<LocalId, Value> = HashMap::default();
    let resolve = |map: &HashMap<LocalId, Value>, mut v: Value| {
        while let Value::Local(l) = v {
            match map.get(&l) {
                Some(&next) => v = next,
                None => break,
            }
        }
        v
    };
    let mut table: Scoped<Key, InstId> = Scoped::new();
    // Remembered memory contents: (address, type) -> (value, position in
    // the write log when it was remembered).
    let mut memory: Scoped<(Value, TypeId), (Value, usize)> = Scoped::new();
    // Every instruction that may write memory on the current dominator
    // path, and `None` where a block can be reached another way.
    let mut writes: Vec<Option<InstId>> = Vec::new();
    let aa = BasicAa::new(sh, f);
    let mut erased = vec![false; f.insts.len()];
    let mut rewritten: Vec<InstId> = Vec::new();

    enum Step {
        Enter(BlockId, bool),
        Exit(usize, usize, usize),
    }
    let mut steps = vec![Step::Enter(f.entry(), false)];
    while let Some(step) = steps.pop() {
        let (b, joined) = match step {
            Step::Exit(t, m, w) => {
                table.undo(t);
                memory.undo(m);
                writes.truncate(w);
                continue;
            }
            Step::Enter(b, joined) => (b, joined),
        };
        steps.push(Step::Exit(table.mark(), memory.mark(), writes.len()));
        if joined {
            writes.push(None);
        }
        for k in 0..f.blocks[b.index()].insts.len() {
            let i = f.blocks[b.index()].insts[k];
            // Operands through the replacements made so far.
            let mut touched = false;
            f.insts[i.index()].op.for_each_value_mut(|v| {
                let new = resolve(&map, *v);
                touched |= new != *v;
                *v = new;
            });
            if touched {
                rewritten.push(i);
            }
            let inst = &f.insts[i.index()];
            if may_write(sh, &inst.op) {
                writes.push(Some(i));
            }
            let Some(r) = inst.result else {
                if let Op::Store { volatile: false, atomic: None, value, ptr, .. } = inst.op {
                    let ty = match value {
                        Value::Local(l) => f.locals[l.index()].ty,
                        Value::Const(c) => sh.consts().get(c).ty,
                        Value::Metadata(_) => continue,
                    };
                    memory.insert((ptr, ty), (value, writes.len()));
                }
                continue;
            };
            let ty = f.locals[r.index()].ty;
            if let Op::Load { volatile: false, atomic: None, ptr, .. } = inst.op {
                let loc = aa.location(ptr, ty);

                match memory.get(&(ptr, ty)) {
                    Some(&(v, since))
                        if still_there(&aa, f, &writes[since..], loc)
                            && epitome_ir::chop::allow(|| {
                                format!("cse: the load {} reuses a value", epitome_ir::printer::inst_name(f, i))
                            }) =>
                    {
                        let from_store = !matches!(v, Value::Local(l) if matches!(f.locals[l.index()].def, LocalDef::Inst(d) if matches!(f.insts[d.index()].op, Op::Load { .. })));
                        if from_store {
                            stats.forwarded += 1;
                        } else {
                            stats.loads += 1;
                        }
                        map.insert(r, v);
                        erased[i.index()] = true;
                    }
                    _ => memory.insert((ptr, ty), (Value::Local(r), writes.len())),
                }
                continue;
            }
            let Some(key) = key(&inst.op, ty, b) else { continue };
            match table.get(&key) {
                Some(&leader)
                    if epitome_ir::chop::allow(|| {
                        let n = |x: InstId| epitome_ir::printer::inst_name(f, x);
                        format!("cse: {} merged into {}", n(i), n(leader))
                    }) =>
                {
                    let dup = inst.op.clone();
                    intersect_flags(&mut f.insts[leader.index()].op, &dup);
                    map.insert(r, f.result(leader));
                    erased[i.index()] = true;
                    stats.merged += 1;
                }
                // Refused by autochop: it stays, and the first stays the
                // leader.
                Some(_) => {}
                None => table.insert(key, i),
            }
        }
        // Children, so that the first child is visited first. A child that
        // can be reached other than from this block may see memory written
        // on another path.
        for &c in children[b.index()].iter().rev() {
            steps.push(Step::Enter(c, preds[c.index()].as_slice() != [b]));
        }
    }
    if stats.total() == 0 {
        return Outcome { stats, changed: Vec::new() };
    }

    // Uses the walk did not reach, such as phi entries from loop back edges
    // and unreachable blocks, also go through the replacements.
    for k in 0..f.layout.len() {
        let b = f.layout[k];
        f.blocks[b.index()].insts.retain(|i| !erased[i.index()]);
    }
    let mut changed = Vec::new();
    for i in f.insts_in_order().collect::<Vec<_>>() {
        let mut touched = false;
        f.insts[i.index()].op.for_each_value_mut(|v| {
            let new = resolve(&map, *v);
            touched |= new != *v;
            *v = new;
        });
        if touched {
            changed.push(i);
        }
    }
    // Debug records follow the replacements.
    let mut changes = Vec::new();
    for i in f.insts_in_order() {
        for (k, rec) in f.insts[i.index()].debug_records.iter().enumerate() {
            for (j, &arg) in rec.args.iter().enumerate() {
                let data = sh.metadata()[arg.index()].data.clone();
                if let Metadata::Value(v @ Value::Local(_)) = data {
                    let new = resolve(&map, v);
                    if new != v {
                        changes.push((i, k, j, new));
                    }
                }
            }
        }
    }
    for (i, k, j, v) in changes {
        let md = sh.md_value(v);
        f.insts[i.index()].debug_records[k].args[j] = md;
    }
    changed.extend(rewritten.into_iter().filter(|i| !erased[i.index()]));
    changed.sort();
    changed.dedup();
    Outcome { stats, changed }
}
