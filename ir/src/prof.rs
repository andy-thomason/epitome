//! Branch weights and `llvm.expect`.
//!
//! A conditional branch or `switch` may carry `!prof` metadata giving a
//! weight to each successor, in operand order (LangRef, "Branch Weight
//! Metadata"):
//!
//! ```text
//! br i1 %c, label %a, label %b, !prof !0
//! !0 = !{!"branch_weights", i32 2000, i32 1}
//! ```
//!
//! (`!"expected"` may follow the name, saying the weights came from
//! `llvm.expect`; it changes nothing here.) `llvm.expect(v, c)` says `v` is
//! most likely `c`, and `llvm.expect.with.probability(v, c, p)` that it is
//! `c` with probability `p`. `lower_expect` turns either into weights on
//! the branches that test it, as LLVM does early in its pipeline, and
//! replaces it with its value: after that nothing but the weights says it
//! was there. Weights are only hints; a pass that cannot keep them right
//! drops them.

use crate::module::*;
use crate::types::TypeId;

/// The weight `llvm.expect` gives the side it expects, and the other.
pub const LIKELY: u64 = 2000;
pub const UNLIKELY: u64 = 1;

/// What reading weights needs from the module the function is in (the
/// optimiser and the code generator keep their tables differently).
pub trait Tables {
    /// The name of the function a call calls directly.
    fn callee_name(&self, call: &Call) -> Option<String>;
    /// An integer constant's value (its low 64 bits).
    fn int_const(&self, c: ConstId) -> Option<u64>;
    /// A `double` constant's value.
    fn double_const(&self, c: ConstId) -> Option<f64>;
    /// An integer type's width.
    fn int_bits(&self, ty: TypeId) -> Option<u32>;
    /// A metadata node's contents.
    fn md(&self, id: MdId) -> Metadata;
}

/// And what writing them needs.
pub trait Env: Tables {
    fn add_md(&mut self, data: Metadata) -> MdId;
    /// An `i32` constant.
    fn i32(&mut self, v: u32) -> Value;
}

/// The `!prof` node of instruction `i`.
pub fn prof_of(inst: &Inst) -> Option<MdId> {
    inst.attachments.iter().find(|a| a.kind == "prof").map(|a| a.node)
}

/// The weights of `id`, if it is a `branch_weights` node: one per
/// successor.
pub fn branch_weights(env: &impl Tables, id: MdId) -> Option<Vec<u64>> {
    let Metadata::Tuple { ops, .. } = env.md(id) else { return None };
    let mut ops = ops.into_iter();
    let name = ops.next()??;
    if env.md(name) != Metadata::String(b"branch_weights".to_vec()) {
        return None;
    }
    let mut out = Vec::new();
    for op in ops {
        match env.md(op?) {
            Metadata::String(_) if out.is_empty() => {}
            Metadata::Value(Value::Const(c)) => out.push(env.int_const(c)?),
            _ => return None,
        }
    }
    Some(out)
}

/// The weights on terminator `inst`, if it has as many as successors.
pub fn weights_of(env: &impl Tables, inst: &Inst) -> Option<Vec<u64>> {
    let w = branch_weights(env, prof_of(inst)?)?;
    (w.len() == inst.op.successors().len()).then_some(w)
}

/// Whether node `id` says its weights came from `llvm.expect`.
fn is_expected(env: &impl Tables, id: MdId) -> bool {
    let Metadata::Tuple { ops, .. } = env.md(id) else { return false };
    ops.get(1).copied().flatten().is_some_and(|x| env.md(x) == Metadata::String(b"expected".to_vec()))
}

/// Puts `weights` on `inst` as its `!prof` (replacing any), marked as
/// from `llvm.expect` if `expected`.
pub fn set_weights(env: &mut impl Env, inst: &mut Inst, weights: &[u64], expected: bool) {
    let name = env.add_md(Metadata::String(b"branch_weights".to_vec()));
    let mut ops = vec![Some(name)];
    if expected {
        ops.push(Some(env.add_md(Metadata::String(b"expected".to_vec()))));
    }
    for &w in weights {
        let v = env.i32(w.min(u32::MAX as u64) as u32);
        ops.push(Some(env.add_md(Metadata::Value(v))));
    }
    let node = env.add_md(Metadata::Tuple { distinct: false, ops });
    inst.attachments.retain(|a| a.kind != "prof");
    inst.attachments.push(Attachment { kind: "prof".into(), node });
}

/// Keeps `inst`'s weights right for its new op, which replaced `old`: a
/// conditional branch whose targets were swapped swaps them; any other
/// change but an unchanged list of successors drops them.
pub fn retarget(env: &mut impl Env, inst: &mut Inst, old: &Op) {
    let id = prof_of(inst);
    let Some(w) = id.and_then(|id| branch_weights(env, id)) else {
        inst.attachments.retain(|a| a.kind != "prof");
        return;
    };
    let (was, now) = (old.successors(), inst.op.successors());
    if was == now && now.len() == w.len() {
        return;
    }
    if let (Op::CondBr { then_dest: a, else_dest: b, .. }, Op::CondBr { then_dest: c, else_dest: d, .. }) =
        (old, &inst.op)
        && a == d
        && b == c
        && w.len() == 2
    {
        let expected = is_expected(env, id.unwrap());
        set_weights(env, inst, &[w[1], w[0]], expected);
        return;
    }
    inst.attachments.retain(|a| a.kind != "prof");
}

/// An `llvm.expect` call: its value, what it expects and with what
/// probability (none for plain `llvm.expect`).
struct Expect {
    value: Value,
    expected: u64,
    probability: Option<f64>,
}

fn expect_of(env: &impl Tables, op: &Op) -> Option<Expect> {
    let Op::Call(call) = op else { return None };
    let name = env.callee_name(call)?;
    let rest = name.strip_prefix("llvm.expect.")?;
    let Value::Const(c) = call.args.get(1)?.value else { return None };
    let expected = env.int_const(c)?;
    let probability = if rest.starts_with("with.probability") {
        let Value::Const(p) = call.args.get(2)?.value else { return None };
        Some(env.double_const(p)?.clamp(0.0, 1.0))
    } else {
        None
    };
    Some(Expect { value: call.args[0].value, expected, probability })
}

fn mask(bits: u32) -> u64 {
    if bits >= 64 { u64::MAX } else { (1u64 << bits) - 1 }
}

/// The value `v` most likely has, if it follows from an `llvm.expect`
/// through casts, an `icmp eq` or `ne` with a constant, or an `xor` with
/// one; and the probability, if one was given.
fn likely_value(env: &impl Tables, f: &Function, v: Value, depth: u32) -> Option<(u64, Option<f64>)> {
    let Value::Local(l) = v else { return None };
    let LocalDef::Inst(i) = f.locals[l.index()].def else { return None };
    if depth > 8 {
        return None;
    }
    let bits = env.int_bits(f.locals[l.index()].ty)?;
    let konst = |v: &Value| match v {
        Value::Const(c) => env.int_const(*c),
        _ => None,
    };
    let op = &f.insts[i.index()].op;
    if let Some(e) = expect_of(env, op) {
        return Some((e.expected & mask(bits), e.probability));
    }
    match op {
        Op::Cast { op: CastOp::ZExt | CastOp::Trunc, value, .. } => {
            let (x, p) = likely_value(env, f, *value, depth + 1)?;
            Some((x & mask(bits), p))
        }
        Op::Cast { op: CastOp::SExt, value, .. } => {
            let (x, p) = likely_value(env, f, *value, depth + 1)?;
            let from = match value {
                Value::Local(s) => env.int_bits(f.locals[s.index()].ty)?,
                _ => return None,
            };
            let neg = from < 64 && x >> (from - 1) & 1 == 1;
            Some((if neg { (x | !mask(from)) & mask(bits) } else { x }, p))
        }
        Op::ICmp { pred: pred @ (ICmpPred::Eq | ICmpPred::Ne), lhs, rhs, .. } => {
            let (x, k) = match (konst(lhs), konst(rhs)) {
                (None, Some(k)) => (*lhs, k),
                (Some(k), None) => (*rhs, k),
                _ => return None,
            };
            let (y, p) = likely_value(env, f, x, depth + 1)?;
            let w = match x {
                Value::Local(s) => env.int_bits(f.locals[s.index()].ty)?,
                _ => return None,
            };
            let same = y == k & mask(w);
            Some(((same == (*pred == ICmpPred::Eq)) as u64, p))
        }
        Op::Binary { op: BinOp::Xor, lhs, rhs, .. } => {
            let (x, k) = match (konst(lhs), konst(rhs)) {
                (None, Some(k)) => (*lhs, k),
                (Some(k), None) => (*rhs, k),
                _ => return None,
            };
            let (y, p) = likely_value(env, f, x, depth + 1)?;
            Some(((y ^ k) & mask(bits), p))
        }
        _ => None,
    }
}

/// The weights of the likely side and the others, for probability `p`
/// (`None`: `llvm.expect`'s).
fn split(p: Option<f64>, others: usize) -> (u64, u64) {
    match p {
        None => (LIKELY, UNLIKELY),
        Some(p) => {
            let scale = (1u64 << 20) as f64;
            let rest = (1.0 - p) / others.max(1) as f64;
            (((p * scale).round() as u64).max(1), ((rest * scale).round() as u64).max(1))
        }
    }
}

/// Turns each `llvm.expect` in `f` into weights on the branches and
/// switches that test it, then replaces it with its value. Returns how
/// many calls went.
pub fn lower_expect(env: &mut impl Env, f: &mut Function) -> usize {
    let calls: Vec<(BlockId, InstId)> = f
        .layout
        .iter()
        .flat_map(|&b| f.blocks[b.index()].insts.iter().map(move |&i| (b, i)))
        .filter(|&(_, i)| f.insts[i.index()].result.is_some() && expect_of(env, &f.insts[i.index()].op).is_some())
        .collect();
    if calls.is_empty() {
        return 0;
    }
    for &b in &f.layout.clone() {
        let Some(t) = f.terminator(b) else { continue };
        let weights = match &f.insts[t.index()].op {
            Op::CondBr { cond, then_dest, else_dest } if then_dest != else_dest => {
                likely_value(env, f, *cond, 0).map(|(x, p)| {
                    let (l, u) = split(p, 1);
                    if x & 1 == 1 { vec![l, u] } else { vec![u, l] }
                })
            }
            Op::Switch { value, cases, .. } => likely_value(env, f, *value, 0).map(|(x, p)| {
                let bits = match value {
                    Value::Local(s) => env.int_bits(f.locals[s.index()].ty).unwrap_or(64),
                    _ => 64,
                };
                let hit = cases.iter().position(|(k, _)| env.int_const(*k).is_some_and(|k| k & mask(bits) == x));
                let (l, u) = split(p, cases.len());
                let mut w = vec![u; cases.len() + 1];
                w[hit.map_or(0, |h| h + 1)] = l;
                w
            }),
            _ => None,
        };
        if let Some(w) = weights {
            let mut inst = f.insts[t.index()].clone();
            set_weights(env, &mut inst, &w, true);
            f.insts[t.index()] = inst;
        }
    }
    for &(b, i) in &calls {
        let e = expect_of(env, &f.insts[i.index()].op).unwrap();
        let r = f.result(i);
        f.replace_uses(r, e.value);
        f.blocks[b.index()].insts.retain(|&x| x != i);
    }
    calls.len()
}
