//! Folding branches on constants before the call graph is built.
//!
//! The call graph is built from the bodies as they come in, before any
//! pass runs, and a call behind a branch that can never be taken (`if
//! cfg!(debug_assertions) { assert!(..) }`, lowered as `icmp ne i8 0, 0`)
//! is an edge like any other. Through the panic machinery, such edges
//! close cycles, and the inliner never inlines between members of one
//! cycle. So, per function and before the graph is built:
//!
//! - a conditional branch on a constant, or on an `icmp` of two integer
//!   constants, becomes a plain branch;
//! - a `switch` on a constant becomes a plain branch to its case;
//! - blocks that can no longer be reached are deleted.
//!
//! This is only what can be seen without looking at memory or at other
//! functions; it is cheap, and the passes after it do the rest. A call
//! whose result is a constant only once its callee is optimised (as
//! `ub_checks()` is) is left for the SCC's task to delete, which splits
//! the cycle there (`pipeline.rs`). Functions whose blocks have their
//! address taken, or that are `optnone`, are left alone.

use epitome_ir::module::*;
use epitome_ir::types::Type;

use crate::{attrs, cleanup, combine::eval, par};

#[derive(Default, Debug, Clone, Copy)]
pub struct Stats {
    /// Branches and switches made plain branches.
    pub branches: usize,
    /// Blocks deleted because nothing reaches them any more.
    pub blocks: usize,
}

/// Folds `m`'s branches on constants (see the module's notes). Each
/// function's changes are elective changes of that function for autochop:
/// `counted` makes them so. Returns the counts and a remark for each
/// function changed.
pub fn run<G>(m: &mut Module, threads: usize, counted: impl Fn(GlobalId) -> G) -> (Stats, Vec<(GlobalId, String)>) {
    let mut named = vec![false; m.globals.len()];
    for i in 0..m.consts.len() {
        if let ConstKind::BlockAddress { func, .. } = m.consts.get(ConstId(i as u32)).kind {
            named[func.index()] = true;
        }
    }
    let fns: Vec<GlobalId> = m
        .functions()
        .filter(|(g, f)| f.has_body && !named[g.index()] && !attrs::has(m, &f.fn_attrs, "optnone"))
        .map(|(g, _)| g)
        .collect();
    // What to fold, found in parallel; most functions have nothing.
    let folds = par::map(&fns, threads, |_, &g| {
        let GlobalKind::Function(f) = &m.global(g).kind else { unreachable!() };
        find(m, f)
    });
    let mut stats = Stats::default();
    let mut remarks = Vec::new();
    for (&g, folds) in fns.iter().zip(folds) {
        if folds.is_empty() {
            continue;
        }
        let _counted = counted(g);
        let GlobalKind::Function(f) = &mut m.globals[g.index()].kind else { unreachable!() };
        let mut folded = 0;
        for (b, t, target) in folds {
            if !epitome_ir::chop::allow(|| {
                format!(
                    "prefold: the branch on a constant in {}",
                    epitome_ir::printer::local_name(f, f.blocks[b.index()].local)
                )
            }) {
                continue;
            }
            // Each other target loses this block's phi entries; the kept
            // one keeps one entry, for the one edge left.
            let succs = f.insts[t.index()].op.successors();
            let mut seen = Vec::new();
            for s in succs {
                if seen.contains(&s) {
                    continue;
                }
                seen.push(s);
                let keep = usize::from(s == target);
                for k in 0..f.blocks[s.index()].insts.len() {
                    let i = f.blocks[s.index()].insts[k];
                    let Op::Phi { incoming, .. } = &mut f.insts[i.index()].op else { break };
                    let mut n = 0;
                    incoming.retain(|(_, from)| {
                        if *from != b {
                            return true;
                        }
                        n += 1;
                        n <= keep
                    });
                }
            }
            f.insts[t.index()].op = Op::Br(target);
            // (Branch weights are per successor: a plain branch has none.)
            f.insts[t.index()].attachments.retain(|a| a.kind != "prof");
            folded += 1;
        }
        if folded > 0 {
            let n = f.layout.len();
            cleanup::remove_unreachable(f);
            let removed = n - f.layout.len();
            stats.branches += folded;
            stats.blocks += removed;
            remarks.push((g, format!("prefold: {folded} branches on constants folded, {removed} blocks removed")));
        }
    }
    (stats, remarks)
}

/// The branches of `f` on constants: each one's block, terminator, and the
/// block it always goes to.
fn find(m: &Module, f: &Function) -> Vec<(BlockId, InstId, BlockId)> {
    let mut out = Vec::new();
    for &b in &f.layout {
        let Some(t) = f.terminator(b) else { continue };
        let target = match &f.insts[t.index()].op {
            Op::CondBr { cond, then_dest, else_dest } => match condition(m, f, *cond) {
                Some(true) => *then_dest,
                Some(false) => *else_dest,
                None => continue,
            },
            Op::Switch { value, default, cases } => {
                let Some(v) = int(m, *value) else { continue };
                cases.iter().find(|&&(c, _)| int(m, Value::Const(c)) == Some(v)).map_or(*default, |&(_, d)| d)
            }
            _ => continue,
        };
        out.push((b, t, target));
    }
    out
}

/// The value of an `i1` condition that is a constant, or an `icmp` of two
/// integer constants.
fn condition(m: &Module, f: &Function, cond: Value) -> Option<bool> {
    if let Some((v, _)) = int(m, cond) {
        return Some(v & 1 == 1);
    }
    let Value::Local(l) = cond else { return None };
    let LocalDef::Inst(d) = f.locals[l.index()].def else { return None };
    let Op::ICmp { pred, lhs, rhs, .. } = f.insts[d.index()].op else { return None };
    let ((a, bits), (b, _)) = (int(m, lhs)?, int(m, rhs)?);
    Some(eval::fold_icmp(pred, a, b, bits))
}

/// An integer constant of at most 64 bits, and its width.
fn int(m: &Module, v: Value) -> Option<(u64, u32)> {
    let Value::Const(c) = v else { return None };
    let k = m.consts.get(c);
    match &k.kind {
        ConstKind::Int(a) if a.bits() <= 64 => Some((a.low_u64(), a.bits())),
        ConstKind::Zero => match m.types.get(k.ty) {
            Type::Int(bits) if *bits <= 64 => Some((0, *bits)),
            _ => None,
        },
        _ => None,
    }
}
