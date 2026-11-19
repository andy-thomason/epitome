//! Epitome's optimisation passes, and the machinery that runs them on many
//! functions at once.
//!
//! - `ctx` and `par`: splitting a module so threads can work on different
//!   functions, and scheduling the work.
//! - `callgraph`: the call graph, its cycles (SCCs) and the bottom-up order.
//! - `inline`: the inliner and its cost model.
//! - `recursion`: turning recursion into loops with an explicit stack.
//! - `pipeline`: the passes above, run together.
//! - `combine`: rewriting instructions into simpler ones, from rules in
//!   `opt/rules/combine.rules`, matched with a discrimination tree.
//! - `aa`: basic alias analysis: whether two accesses can overlap, and
//!   whether an instruction can read or write a location.
//! - `sroa`: splitting allocas into pieces `mem2reg` can promote.
//! - `mem2reg`: promoting allocas to SSA values.
//! - `memssa`: memory SSA, and the walker that finds a load's clobbering
//!   write on demand.
//! - `simplifycfg`: unreachable blocks, empty blocks, diamonds to
//!   selects, straight lines; branches themselves fold in the combiner.
//! - `implied`: what dominating branches imply: comparisons folded,
//!   selects resolved and `nuw` set from the conditions on the way to
//!   them (bounds checks in loops).
//! - `ranges`: what each integer can be, as unsigned and signed intervals,
//!   which `implied` narrows by the facts where a value is used.
//! - `cse`: common-subexpression elimination, by hashing a canonical key
//!   for each instruction, scoped by the dominator tree.
//! - `cfg`, `ssa`, `cleanup`: analyses and helpers the passes share.
//!
//! See `docs/design/parallel.md` for the design.

pub mod aa;
pub mod attrs;
pub mod callgraph;
pub mod cfg;
pub mod cleanup;
pub mod combine;
pub mod constcall;
pub mod cse;
pub mod ctx;
pub mod fx;
pub mod implied;
pub mod indvars;
pub mod inline;
pub mod licm;
pub mod loopidiom;
pub mod loops;
pub mod lsr;
pub mod mem2reg;
pub mod memcpyopt;
pub mod memssa;
pub mod par;
pub mod pipeline;
pub mod ranges;
pub mod recursion;
pub mod rotate;
pub mod simplifycfg;
pub mod sroa;
pub mod ssa;
pub mod table;
pub mod tailmark;
