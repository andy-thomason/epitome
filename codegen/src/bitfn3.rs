//! Chains of bitwise logic over three values as one function of three bits
//! (docs/design/combine.md, "Three-input bit functions"): a tree of `and`,
//! `or` and `xor` on integer vectors (a `not` is an `xor` with all ones),
//! reading at most three values besides constants of all zeros or all
//! ones, becomes `llvm.epitome.bitfn3.vNiW(i8 table, a, b, c)`, where the
//! description has an instruction for it at that type (x86's
//! `vpternlogd`).
//!
//! - The table is the tree evaluated on the bytes 0xf0, 0xcc and 0xaa in
//!   place of the three values: bit k is the function's value where the
//!   inputs are bits k of those.
//! - An operation with other uses may be taken in too: computed again
//!   within the tree, it stays for those. A tree is made one call if that
//!   saves at least one instruction: two of its operations go, not
//!   counting a `not` that `and` takes (`vpandn` does the two in one).
//!   Use counts are kept as trees are made, last first, so that an
//!   operation two trees shared goes with the second (a full adder's
//!   `a ^ b`).
//! - A constant leaf is the last argument, which x86 can read from
//!   memory (a broadcast from read-only data, `12-pool.ll`).
//! - Each is an elective change (autochop).

use std::collections::{HashMap, HashSet};

use epitome_ir::module::*;
use epitome_ir::types::{Type, TypeId};

use crate::prepare::{Ctx, Seq};

/// The name of the intrinsic for vectors of type `ty`, if they are of
/// integers.
fn name_of(cx: &Ctx, ty: TypeId) -> Option<String> {
    let Type::Vector { len, elem, scalable: false } = *cx.m.types.get(ty) else { return None };
    let Type::Int(b) = *cx.m.types.get(elem) else { return None };
    (b >= 8).then(|| format!("llvm.epitome.bitfn3.v{len}i{b}"))
}

/// All zeros (`Some(0x00)`) or all ones (`Some(0xff)`), as a byte of a
/// table.
fn const_byte(cx: &Ctx, v: Value) -> Option<u8> {
    let Value::Const(c) = v else { return None };
    let lane = |e: ConstId| match &cx.m.constant(e).kind {
        ConstKind::Int(a) if a.is_zero() => Some(0x00u8),
        ConstKind::Int(a) if a.bits() <= 64 && a.low_u64() == u64::MAX >> (64 - a.bits()) => Some(0xff),
        ConstKind::Zero => Some(0x00),
        _ => None,
    };
    match &cx.m.constant(c).kind {
        ConstKind::Zero => Some(0x00),
        ConstKind::Splat(e) => lane(*e),
        ConstKind::Vector(es) => {
            let first = lane(*es.first()?)?;
            es.iter().all(|&e| lane(e) == Some(first)).then_some(first)
        }
        _ => None,
    }
}

/// A tree being gathered: its leaves (at most three) and its operations.
struct Tree {
    leaves: Vec<Value>,
    ops: Vec<InstId>,
}

fn logic(f: &Function, v: Value) -> Option<(InstId, BinOp, Value, Value)> {
    let Value::Local(x) = v else { return None };
    let LocalDef::Inst(d) = f.locals[x.index()].def else { return None };
    match f.insts[d.index()].op {
        Op::Binary { op: op @ (BinOp::And | BinOp::Or | BinOp::Xor), lhs, rhs, .. } => Some((d, op, lhs, rhs)),
        _ => None,
    }
}

/// Gathers the tree under `v` (an operand of the tree), taking in an
/// operation if its leaves still fit. (One with other uses is computed
/// again within the tree, and stays for those.)
fn gather(cx: &Ctx, f: &Function, v: Value, t: &mut Tree) {
    if const_byte(cx, v).is_some() {
        return;
    }
    if let Some((d, _, lhs, rhs)) = logic(f, v)
        && t.ops.len() < 16
    {
        // Taken in if the leaves it adds still fit (tried on a copy).
        let mut inner = Tree { leaves: t.leaves.clone(), ops: t.ops.clone() };
        inner.ops.push(d);
        gather(cx, f, lhs, &mut inner);
        gather(cx, f, rhs, &mut inner);
        if inner.leaves.len() <= 3 {
            *t = inner;
            return;
        }
    }
    if !t.leaves.contains(&v) {
        t.leaves.push(v);
    }
}

/// The operations of the tree rooted at `root` that would go: the root,
/// and those whose every use is by one that goes.
fn dying(f: &Function, uses: &HashMap<LocalId, usize>, t: &Tree) -> HashSet<InstId> {
    let mut dead: HashSet<InstId> = HashSet::from([t.ops[0]]);
    loop {
        let mut by_dead: HashMap<LocalId, usize> = HashMap::new();
        for &d in &dead {
            f.insts[d.index()].op.for_each_value(|v| {
                if let Value::Local(x) = v {
                    *by_dead.entry(*x).or_insert(0) += 1;
                }
            });
        }
        let more: Vec<InstId> = t.ops[1..]
            .iter()
            .copied()
            .filter(|d| !dead.contains(d))
            .filter(|&d| {
                let r = f.insts[d.index()].result.unwrap();
                by_dead.get(&r) == uses.get(&r)
            })
            .collect();
        if more.is_empty() {
            return dead;
        }
        dead.extend(more);
    }
}

/// Of `dead`, the `not`s an `and` among them takes, which vpandn does for
/// nothing.
fn free_nots(cx: &Ctx, f: &Function, dead: &HashSet<InstId>) -> usize {
    let not = |v: Value| {
        logic(f, v).is_some_and(|(d, op, l, r)| {
            dead.contains(&d)
                && op == BinOp::Xor
                && (const_byte(cx, l) == Some(0xff) || const_byte(cx, r) == Some(0xff))
        })
    };
    dead.iter()
        .filter_map(|d| match f.insts[d.index()].op {
            Op::Binary { op: BinOp::And, lhs, rhs, .. } => Some(usize::from(not(lhs)) + usize::from(not(rhs))),
            _ => None,
        })
        .sum::<usize>()
        .min(dead.len())
}

/// The tree under `v` evaluated on the leaves' bytes.
fn table(cx: &Ctx, f: &Function, v: Value, t: &Tree) -> u8 {
    if let Some(b) = const_byte(cx, v) {
        return b;
    }
    if let Some(k) = t.leaves.iter().position(|l| *l == v) {
        return [0xf0, 0xcc, 0xaa][k];
    }
    let (_, op, lhs, rhs) = logic(f, v).expect("an operation of the tree");
    let (a, b) = (table(cx, f, lhs, t), table(cx, f, rhs, t));
    match op {
        BinOp::And => a & b,
        BinOp::Or => a | b,
        _ => a ^ b,
    }
}

pub fn form(cx: &mut Ctx, f: &mut Function) {
    // (Nothing to do where no width has the instruction.)
    if cx.vector_bits() == 0
        || !cx.selects_intrinsic("llvm.epitome.bitfn3.v4i32") && !cx.selects_intrinsic("llvm.epitome.bitfn3.v16i32")
    {
        return;
    }
    let mut has: HashMap<TypeId, Option<String>> = HashMap::new();
    let mut uses: HashMap<LocalId, usize> = HashMap::new();
    for i in f.insts_in_order() {
        f.insts[i.index()].op.for_each_value(|v| {
            if let Value::Local(x) = v {
                *uses.entry(*x).or_insert(0) += 1;
            }
        });
    }
    let i8t = cx.m.types.int(8);
    let mut gone: HashSet<InstId> = HashSet::new();
    for b in f.layout.clone() {
        // Last first, so that a tree's root is met before its operations.
        let mut k = f.blocks[b.index()].insts.len();
        while k > 0 {
            k -= 1;
            let i = f.blocks[b.index()].insts[k];
            if gone.contains(&i) {
                continue;
            }
            let Some(r) = f.insts[i.index()].result else { continue };
            let ty = f.locals[r.index()].ty;
            let Some((_, _, lhs, rhs)) = logic(f, Value::Local(r)) else { continue };
            let name = has.entry(ty).or_insert_with(|| name_of(cx, ty).filter(|n| cx.selects_intrinsic(n))).clone();
            let Some(name) = name else { continue };
            let mut t = Tree { leaves: Vec::new(), ops: vec![i] };
            gather(cx, f, lhs, &mut t);
            gather(cx, f, rhs, &mut t);
            // (More than three values: its operations may be roots of
            // their own, met next.)
            if t.leaves.is_empty() || t.leaves.len() > 3 {
                continue;
            }
            // One call in place of what goes: worth it if two go.
            let dead = dying(f, &uses, &t);
            if dead.len() - free_nots(cx, f, &dead) < 2 {
                continue;
            }
            // A constant leaf goes last, into the operand that may be
            // memory (x86's broadcast from read-only data), the others
            // before it: the first again for a missing one.
            if let Some(c) = t.leaves.iter().position(|l| matches!(l, Value::Const(_))) {
                let k = t.leaves.remove(c);
                while t.leaves.len() < 2 {
                    t.leaves.push(t.leaves[0]);
                }
                t.leaves.push(k);
            }
            let bits = table(cx, f, Value::Local(r), &t);
            if !epitome_ir::chop::allow(|| format!("bitfn3 {bits:#04x} of {} operations", t.ops.len())) {
                continue;
            }
            // (Unused inputs read the first.)
            let mut args: Vec<Value> = t.leaves.clone();
            while args.len() < 3 {
                args.push(args[0]);
            }
            let tv = cx.int(i8t, i64::from(bits as i8));
            args.insert(0, tv);
            let mut s = Seq { f, new: Vec::new() };
            let v = crate::prepare::call_intrinsic(cx, &mut s, &name, args, ty);
            let new = std::mem::take(&mut s.new);
            f.replace_uses(Value::Local(r), v);
            f.blocks[b.index()].insts.splice(k + 1..k + 1, new);
            // What the operations that go used, and the call uses.
            for &d in &dead {
                f.insts[d.index()].op.for_each_value(|v| {
                    if let Value::Local(x) = v {
                        *uses.entry(*x).or_insert(1) -= 1;
                    }
                });
            }
            for l in &t.leaves {
                if let Value::Local(x) = l {
                    *uses.entry(*x).or_insert(0) += 1;
                }
            }
            gone.extend(dead);
        }
    }
    // The operations taken in, now unused.
    for blk in &mut f.blocks {
        blk.insts.retain(|i| !gone.contains(i));
    }
}
