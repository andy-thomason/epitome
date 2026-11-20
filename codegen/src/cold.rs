//! Which blocks are cold: rarely run, so placed after the rest of the
//! function (`layout.rs`), where they cost the hot code no cache lines
//! and no taken branches.
//!
//! A block is cold when the IR says so:
//!
//! - every path from it ends in `unreachable`, a trap, or a call that does
//!   not return (`noreturn`): a panic, an abort, a failed bounds check;
//! - it calls a function that is `cold`, or makes a call marked `cold`;
//! - it is an exception handling pad (a landing pad and the like);
//!
//! and coldness flows on: a block is also cold when every edge into it is
//! cold, an edge being cold when it comes from a cold block or is unlikely
//! by its branch's weights (`!prof`, which `llvm.expect` becomes first:
//! `epitome_ir::prof`). So a block reached only through cold blocks (one
//! a cold block dominates, say) is cold, and a join that hot code also
//! reaches is not. The entry block never is.

use epitome_ir::float::{FloatKind, to_f64};
use epitome_ir::module::*;
use epitome_ir::prof;
use epitome_ir::types::{Type, TypeId};

use crate::isel::has_word;
use crate::prepare::callee_name;

/// An edge is unlikely when its share of its branch's weight is at most
/// one part in this many.
const UNLIKELY_SHARE: u64 = 64;

/// The module's tables, to read weights from (`&Module`) or write them
/// (`&mut Module`).
struct Env<M>(M);

impl<M: std::ops::Deref<Target = Module>> prof::Tables for Env<M> {
    fn callee_name(&self, call: &Call) -> Option<String> {
        callee_name(&self.0, call)
    }

    fn int_const(&self, c: ConstId) -> Option<u64> {
        match &self.0.constant(c).kind {
            ConstKind::Int(v) => Some(v.low_u64()),
            ConstKind::Zero => Some(0),
            _ => None,
        }
    }

    fn double_const(&self, c: ConstId) -> Option<f64> {
        match self.0.constant(c).kind {
            ConstKind::Float(bits) => to_f64(FloatKind::Double, bits),
            _ => None,
        }
    }

    fn int_bits(&self, ty: TypeId) -> Option<u32> {
        match self.0.types.get(ty) {
            Type::Int(n) => Some(*n),
            _ => None,
        }
    }

    fn md(&self, id: MdId) -> Metadata {
        self.0.metadata[id.index()].data.clone()
    }
}

impl prof::Env for Env<&mut Module> {
    fn add_md(&mut self, data: Metadata) -> MdId {
        self.0.metadata.push(MdNode { slot: None, data });
        MdId(self.0.metadata.len() as u32 - 1)
    }

    fn i32(&mut self, v: u32) -> Value {
        let ty = self.0.types.int(32);
        Value::Const(self.0.consts.intern(Constant { ty, kind: ConstKind::Int(epitome_ir::apint::ApInt::from_u64(32, v as u64)) }))
    }
}

/// Whether a call's attributes, or its callee's, have `word`.
fn call_has(m: &Module, call: &Call, word: &str) -> bool {
    has_word(m, &call.fn_attrs, word)
        || match call.callee {
            Value::Const(c) => match m.constant(c).kind {
                ConstKind::Global(g) => match &m.global(g).kind {
                    GlobalKind::Function(f) => has_word(m, &f.fn_attrs, word),
                    _ => false,
                },
                _ => false,
            },
            _ => false,
        }
}

/// Turns `f`'s `llvm.expect` calls into weights on the branches that test
/// them (`epitome_ir::prof::lower_expect`).
pub fn lower_expect(m: &mut Module, f: &mut Function) {
    prof::lower_expect(&mut Env(m), f);
}

/// Which of `f`'s blocks are cold, by their place in `f.layout`.
pub fn blocks(m: &Module, f: &Function) -> Vec<bool> {
    let n = f.blocks.len();
    let succs: Vec<Vec<BlockId>> = (0..n).map(|b| f.successors(BlockId(b as u32))).collect();
    let mut preds: Vec<Vec<BlockId>> = vec![Vec::new(); n];
    for &b in &f.layout {
        for &s in &succs[b.index()] {
            preds[s.index()].push(b);
        }
    }

    // What each block holds: a way out of the program, a cold call, a pad.
    let mut ends = vec![false; n];
    let mut seed = vec![false; n];
    for &b in &f.layout {
        for &i in &f.blocks[b.index()].insts {
            match &f.insts[i.index()].op {
                Op::Unreachable => ends[b.index()] = true,
                Op::LandingPad { .. } | Op::CatchPad { .. } | Op::CleanupPad { .. } | Op::CatchSwitch { .. } => {
                    seed[b.index()] = true
                }
                op => {
                    if let Some(call) = op.call() {
                        let name = callee_name(m, call);
                        if call_has(m, call, "noreturn") || name.as_deref().is_some_and(|n| n.starts_with("llvm.trap") || n.starts_with("llvm.ubsantrap")) {
                            ends[b.index()] = true;
                        }
                        if call_has(m, call, "cold") {
                            seed[b.index()] = true;
                        }
                    }
                }
            }
        }
    }
    // Blocks every path from which ends the program.
    loop {
        let mut changed = false;
        for &b in f.layout.iter().rev() {
            let s = &succs[b.index()];
            if !ends[b.index()] && !s.is_empty() && s.iter().all(|x| ends[x.index()]) {
                ends[b.index()] = true;
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }
    for b in 0..n {
        seed[b] |= ends[b];
    }

    // Each block's unlikely successors, by its weights.
    let unlikely: Vec<Vec<BlockId>> = (0..n)
        .map(|b| {
            let Some(t) = f.terminator(BlockId(b as u32)) else { return Vec::new() };
            let inst = &f.insts[t.index()];
            let Some(w) = prof::weights_of(&Env(m), inst) else { return Vec::new() };
            let total: u64 = w.iter().sum();
            let s = &succs[b];
            let share = |x: BlockId| s.iter().zip(&w).filter(|(y, _)| **y == x).map(|(_, w)| *w).sum::<u64>();
            s.iter().copied().filter(|&x| total > 0 && share(x) * UNLIKELY_SHARE <= total).collect()
        })
        .collect();

    // Coldness flows on, to blocks all of whose edges in are cold: from
    // everything cold (but the entry), until nothing changes, so a loop
    // only cold code reaches stays cold.
    let entry = f.layout[0].index();
    let mut cold = vec![true; n];
    cold[entry] = false;
    loop {
        let mut changed = false;
        for &b in &f.layout {
            let b = b.index();
            if !cold[b] || seed[b] || preds[b].is_empty() {
                continue;
            }
            let hot_edge = preds[b].iter().any(|p| !cold[p.index()] && !unlikely[p.index()].contains(&BlockId(b as u32)));
            if hot_edge {
                cold[b] = false;
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }
    f.layout.iter().map(|b| cold[b.index()]).collect()
}
