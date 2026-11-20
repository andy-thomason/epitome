//! The inliner: deciding which calls to inline, and doing it.
//!
//! The inliner works bottom-up over the call graph (see `pipeline`). When
//! it looks at a function, every callee outside the function's own SCC is
//! already final and published, so:
//!
//! - the decision for a call uses the callee's final size, which does not
//!   depend on which other functions happen to be finished, and
//! - calls inside a cycle are never inlined here. The recursion pass deals
//!   with cycles.
//!
//! # The cost model
//!
//! Each call compares what inlining costs in code size with what it saves:
//!
//! - **cost**: the callee's size in weighted instructions (`inst_cost`).
//!   Instructions that disappear or fold into addressing count 0.
//! - **bonus**, the things inlining removes or enables:
//!   - the call itself: `CALL_COST` plus one per argument;
//!   - a constant argument: `CONST_COND_BONUS` for each use of the
//!     parameter in a comparison, branch, switch or select (the branch
//!     folds away), and `CONST_ARITH_BONUS` for each other use;
//!   - a pointer to a caller alloca: `ALLOCA_ARG_BONUS` for each load,
//!     store or address computation through the parameter (the memory can
//!     then be promoted to registers);
//!   - the only call of an internal function whose address is never taken:
//!     the whole callee cost plus `FUNCTION_OVERHEAD`, because the original
//!     is deleted afterwards and code size does not grow.
//!
//! A call is inlined when `cost - bonus <= threshold`. The threshold comes
//! from the optimisation level and is lowered for size-optimised callers
//! (`optsize`, `minsize`), cold callees, calls on paths that end in
//! `unreachable` and callers that only run on the way to one (the call
//! graph's `cold`), and raised for `inlinehint`. At `-Os` and `-Oz`, a
//! callee that stays in the program because it has other callers is only
//! inlined where the copy is no bigger than the call. `alwaysinline` skips the
//! comparison; `noinline` and `optnone` callees are never inlined.
//!
//! As a guard against blow-up, a caller may grow to at most
//! `growth_factor` times its starting size plus `growth_slack`
//! instructions, whatever the individual decisions say. Inlining the only
//! call of a function that is then deleted is exempt, because the module
//! does not grow; it is only held to `HARD_SIZE_LIMIT`.

use crate::fx::FxHashMap as HashMap;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicU32, Ordering};

use epitome_ir::edit::Names;
use epitome_ir::module::*;
use epitome_ir::types::Type;

use crate::attrs;
use crate::callgraph::CallGraph;
use crate::ctx::Shared;

const CALL_COST: i64 = 5;
const CONST_COND_BONUS: i64 = 10;
const CONST_ARITH_BONUS: i64 = 2;
const ALLOCA_ARG_BONUS: i64 = 3;
const FUNCTION_OVERHEAD: i64 = 5;
/// No function grows past this many instructions by inlining, even when
/// inlining is free in module size.
const HARD_SIZE_LIMIT: usize = 50_000;

/// How hard to optimise. Maps onto `-O1` to `-O3`, `-Os` and `-Oz`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OptLevel {
    O1,
    O2,
    O3,
    Os,
    Oz,
}

impl OptLevel {
    pub fn parse(s: &str) -> Option<OptLevel> {
        Some(match s {
            "1" => OptLevel::O1,
            "2" => OptLevel::O2,
            "3" => OptLevel::O3,
            "s" => OptLevel::Os,
            "z" => OptLevel::Oz,
            _ => return None,
        })
    }
}

#[derive(Clone, Copy, Debug)]
pub struct InlineParams {
    pub threshold: i64,
    /// Threshold for size-optimised callers, cold callees and cold calls.
    pub cold_threshold: i64,
    /// Threshold for a callee that is called from more than one place (or
    /// whose address is taken), and so stays in the program: each copy
    /// inlined adds to its size. Lower than `threshold` only at `-Os` and
    /// `-Oz`.
    pub shared_threshold: i64,
    /// Bonus added to the threshold for `inlinehint` callees.
    pub hint_bonus: i64,
    pub growth_factor: usize,
    pub growth_slack: usize,
}

impl InlineParams {
    /// The parameters for a caller with attributes `attrs`: its own
    /// `"epitome-opt-level"` (`1`, `2`, `3`, `s`, `z`), where it has one
    /// (a crate built at its own level), else `default`.
    pub fn for_caller(m: &Module, attrs: &[Attr], default: InlineParams) -> InlineParams {
        match crate::attrs::string(m, attrs, "epitome-opt-level").and_then(|v| std::str::from_utf8(v).ok()).and_then(OptLevel::parse) {
            Some(level) => InlineParams::for_level(level),
            None => default,
        }
    }

    pub fn for_level(level: OptLevel) -> InlineParams {
        let (threshold, growth_factor, growth_slack) = match level {
            OptLevel::O1 => (40, 2, 100),
            OptLevel::O2 => (100, 4, 400),
            OptLevel::O3 => (250, 8, 1000),
            OptLevel::Os => (25, 2, 50),
            OptLevel::Oz => (0, 1, 20),
        };
        let cold_threshold = threshold.min(0);
        let shared_threshold = match level {
            OptLevel::Os | OptLevel::Oz => cold_threshold,
            _ => threshold,
        };
        InlineParams { threshold, cold_threshold, shared_threshold, hint_bonus: threshold / 2, growth_factor, growth_slack }
    }
}

/// What the inliner needs to know about a callee, computed once per
/// published function.
#[derive(Debug)]
pub struct Summary {
    pub cost: i64,
    /// Why this function can never be inlined, if it cannot.
    pub blocker: Option<&'static str>,
    /// Allocas outside the entry block or with a variable size.
    pub dynamic_alloca: bool,
    /// The body still calls a function in its own SCC, itself included, so
    /// inlining it would only unroll a cycle.
    pub in_cycle: bool,
    /// Landing pads, `invoke` or `resume`.
    pub has_eh: bool,
    pub nounwind: bool,
    pub param_cond_uses: Vec<u32>,
    pub param_arith_uses: Vec<u32>,
    pub param_mem_uses: Vec<u32>,
}

/// Per-module inliner state shared by every task.
pub struct Inliner<'a> {
    pub params: InlineParams,
    pub cg: &'a CallGraph,
    summaries: Vec<OnceLock<Summary>>,
    /// Which piece of its SCC each node is in, once the SCC's task has
    /// found that dead calls held it together and split it (see
    /// `set_piece`); 0 until then.
    pieces: Vec<AtomicU32>,
}

impl<'a> Inliner<'a> {
    pub fn new(params: InlineParams, cg: &'a CallGraph, globals: usize) -> Self {
        Inliner {
            params,
            cg,
            summaries: (0..globals).map(|_| OnceLock::new()).collect(),
            pieces: (0..cg.nodes.len()).map(|_| AtomicU32::new(0)).collect(),
        }
    }

    fn summary(&self, sh: &Shared, g: GlobalId, f: &Function) -> &Summary {
        self.summaries[g.index()].get_or_init(|| summarize(sh, self, g, f))
    }

    /// Puts `g` in piece `piece` of its SCC. Only the task of `g`'s SCC
    /// calls this, before it publishes `g`, and with a number no other
    /// piece of that SCC has had: from then on `g` and the members left in
    /// other pieces count as in different SCCs, so calls between them may
    /// be inlined.
    pub fn set_piece(&self, g: GlobalId, piece: u32) {
        if let Some(n) = self.cg.node(g) {
            self.pieces[n].store(piece, Ordering::Relaxed);
        }
    }

    /// True if `a` and `b` are in the same SCC (and the same piece of it).
    pub fn same_scc(&self, a: GlobalId, b: GlobalId) -> bool {
        match (self.cg.node(a), self.cg.node(b)) {
            (Some(x), Some(y)) => {
                self.cg.scc_of[x] == self.cg.scc_of[y]
                    && self.pieces[x].load(Ordering::Relaxed) == self.pieces[y].load(Ordering::Relaxed)
            }
            _ => false,
        }
    }

    /// Inlines the calls in `f`, which is function `g`, that are worth it.
    /// Calls to functions in `g`'s own SCC are left alone. Returns the
    /// number of calls inlined.
    pub fn run_on(&self, sh: &Shared, g: GlobalId, f: &mut Function) -> usize {
        let m = sh.module;
        let caller_attrs = f.fn_attrs.clone();
        if attrs::has(m, &caller_attrs, "optnone") {
            return 0;
        }
        let start_size = live_size(f);
        let params = InlineParams::for_caller(m, &caller_attrs, self.params);
        let limit = start_size * params.growth_factor + params.growth_slack;
        let mut size = start_size;
        // Built on first use: most functions inline nothing.
        let mut names: Option<Names> = None;
        let mut count = 0;
        let mut memo = DebugMemo::default();
        // Only reached on the way to a panic or an error report.
        let caller_cold = self.cg.node(g).is_some_and(|n| self.cg.cold[n]);

        // The calls to consider, in layout order. Calls exposed by inlining
        // are not revisited: the callee's own inliner already decided them.
        let sites: Vec<InstId> = f
            .insts_in_order()
            .filter(|&i| {
                let Some(call) = f.insts[i.index()].op.call() else { return false };
                let Some(callee) = sh.direct_callee(call) else { return false };
                // Within the SCC, only calls the cut left (`cut.rs`) to
                // members already finished.
                (!self.same_scc(g, callee) || !crate::cut::is_cut(call)) && sh.published(callee).is_some_and(|c| c.has_body)
            })
            .collect();

        // Each site's block: where it started, and where the rest of that
        // block has moved to as calls before it were inlined (sites come in
        // order, and inlining a call moves what follows it to a new block).
        let start_blocks = f.inst_blocks();
        let mut tail: Vec<Option<BlockId>> = vec![None; f.blocks.len()];
        // Results of inlined calls, and what replaces them: applied in one
        // sweep at the end. A site's own operands are brought up to date
        // first, so that it is judged on what it will really be passed.
        let mut pending: Vec<Option<Value>> = vec![None; f.locals.len()];
        let mut order = LinkedLayout::new(f);
        let resolve = |pending: &[Option<Value>], mut v: Value| {
            while let Value::Local(l) = v
                && let Some(Some(n)) = pending.get(l.index())
            {
                v = *n;
            }
            v
        };
        for site in sites {
            if count > 0 {
                f.insts[site.index()].op.for_each_value_mut(|v| *v = resolve(&pending, *v));
            }
            let call = f.insts[site.index()].op.call().unwrap();
            let callee_g = sh.direct_callee(call).unwrap();
            let callee = sh.published(callee_g).unwrap();
            let summary = self.summary(sh, callee_g, callee);
            let callee_name = sh.name(callee_g);
            let first = start_blocks[site.index()].expect("call site not in a block");
            let block = tail[first.index()].unwrap_or(first);

            let own = self.same_scc(g, callee_g);
            let decision = self.decide(sh, f, site, block, &caller_attrs, caller_cold, callee_g, callee, summary, own);
            let Decision::Inline { cost, bonus, threshold, last_call } = decision else {
                if let Decision::No(reason) = decision {
                    sh.remark(g, format!("not inlined @{callee_name}: {reason}"));
                }
                continue;
            };
            let new_size = size + summary.cost.max(0) as usize;
            let limit = if last_call { HARD_SIZE_LIMIT } else { limit };
            if new_size > limit && threshold != i64::MAX {
                sh.remark(g, format!("not inlined @{callee_name}: caller would grow past {limit} instructions"));
                continue;
            }
            if !epitome_ir::chop::allow(|| {
                format!("inline @{callee_name} at {}", epitome_ir::printer::inst_name(f, site))
            }) {
                continue;
            }
            let names = names.get_or_insert_with(|| Names::new(f));
            let done = inline_call_deferred(
                sh,
                f,
                Some(&mut order),
                names,
                &mut memo,
                site,
                block,
                callee,
                &callee_name,
                summary.dynamic_alloca,
            );
            if matches!(f.insts[site.index()].op, Op::Call(_)) {
                tail[first.index()] = Some(done.cont);
            }
            if let Some((r, v)) = done.replace {
                if pending.len() < f.locals.len() {
                    pending.resize(f.locals.len(), None);
                }
                pending[r.index()] = Some(v);
            }
            size = (size as isize + done.added) as usize;
            count += 1;
            let why = if threshold == i64::MAX {
                "alwaysinline".to_string()
            } else {
                format!("cost {cost}, bonus {bonus}, threshold {threshold}")
            };
            sh.remark(g, format!("inlined @{callee_name}: {why}"));
        }
        if count > 0 {
            order.finish(f);
            debug_assert_eq!(size, live_size(f), "the size was kept up to date");
            if pending.len() < f.locals.len() {
                pending.resize(f.locals.len(), None);
            }
            let order: Vec<InstId> = f.insts_in_order().collect();
            for i in order {
                f.insts[i.index()].op.for_each_value_mut(|v| *v = resolve(&pending, *v));
            }
            let lmap: Vec<Option<Value>> =
                (0..f.locals.len()).map(|k| pending[k].map(|v| resolve(&pending, v))).collect();
            replace_all_in_debug_records(sh, f, &lmap);
        }
        // A merged block would vanish from under a `blockaddress`.
        let block_addresses = self.cg.node(g).is_some_and(|n| self.cg.has_blockaddress[n]);
        if count > 0 && !block_addresses {
            crate::cleanup::merge_blocks(f);
        }
        count
    }

    #[allow(clippy::too_many_arguments)]
    fn decide(
        &self,
        sh: &Shared,
        caller: &Function,
        site: InstId,
        block: BlockId,
        caller_attrs: &[Attr],
        caller_cold: bool,
        callee_g: GlobalId,
        callee: &Function,
        s: &Summary,
        own_scc: bool,
    ) -> Decision {
        let m = sh.module;
        let inst = &caller.insts[site.index()];
        let call = inst.op.call().unwrap();
        if let Some(r) = s.blocker {
            return Decision::No(r.into());
        }
        // Inside the callee's own SCC, the calls that close the cycle are
        // cut and stay calls, so inlining it unrolls nothing.
        if s.in_cycle && !own_scc {
            return Decision::No("callee is part of a call-graph cycle".into());
        }
        match &inst.op {
            Op::Call(_) => {}
            Op::Invoke { .. } if s.nounwind && !s.has_eh => {}
            Op::Invoke { .. } => return Decision::No("invoke of a callee that may unwind".into()),
            _ => return Decision::No("callbr".into()),
        }
        if call.tail == TailKind::MustTail {
            return Decision::No("musttail call".into());
        }
        if !call.bundles.is_empty() {
            return Decision::No("operand bundles".into());
        }
        if call.cconv != callee.cconv {
            return Decision::No("calling convention mismatch".into());
        }
        if call.fn_ty != callee.ty || call.args.len() != callee.params.len() {
            return Decision::No("call type does not match the callee".into());
        }
        if attrs::has(m, &call.fn_attrs, "noinline") {
            return Decision::No("call site is noinline".into());
        }
        if s.has_eh && caller.personality.is_some() && caller.personality != callee.personality {
            return Decision::No("different personality functions".into());
        }
        if callee.gc.is_some() && callee.gc != caller.gc {
            return Decision::No("different garbage collectors".into());
        }
        for key in ["target-cpu", "target-features"] {
            let theirs = attrs::string(m, &callee.fn_attrs, key);
            if theirs.is_some() && theirs != attrs::string(m, caller_attrs, key) {
                return Decision::No("different target features".into());
            }
        }
        if attrs::has(m, &callee.fn_attrs, "alwaysinline") {
            return Decision::Inline { cost: s.cost, bonus: 0, threshold: i64::MAX, last_call: false };
        }
        if attrs::has(m, caller_attrs, "optnone") {
            return Decision::No("caller is optnone".into());
        }

        // Threshold.
        let p = &InlineParams::for_caller(m, caller_attrs, self.params);
        let cold_site = ends_in_unreachable(caller, block);
        let mut threshold = p.threshold;
        let mut why = if caller_cold { " (caller only runs on the way to a panic)" } else { "" };
        if attrs::has(m, caller_attrs, "minsize")
            || attrs::has(m, caller_attrs, "optsize")
            || attrs::has(m, &callee.fn_attrs, "cold")
            || attrs::has(m, &call.fn_attrs, "cold")
            || cold_site
            || caller_cold
        {
            threshold = threshold.min(p.cold_threshold);
        } else if attrs::has(m, &callee.fn_attrs, "inlinehint") {
            threshold += p.hint_bonus;
        }

        // Bonus.
        let mut bonus = CALL_COST + call.args.len() as i64;
        let consts = sh.consts();
        for (i, a) in call.args.iter().enumerate() {
            match a.value {
                Value::Const(c) if !matches!(consts.get(c).kind, ConstKind::Undef | ConstKind::Poison) => {
                    bonus += CONST_COND_BONUS * s.param_cond_uses[i] as i64
                        + CONST_ARITH_BONUS * s.param_arith_uses[i] as i64;
                }
                Value::Local(l) if is_alloca(caller, l) => {
                    bonus += ALLOCA_ARG_BONUS * s.param_mem_uses[i].min(8) as i64
                }
                _ => {}
            }
        }
        let mut last_call = false;
        if let Some(n) = self.cg.node(callee_g) {
            let local = matches!(sh.module.global(callee_g).props.linkage, Linkage::Internal | Linkage::Private);
            if local && !self.cg.address_taken[n] && self.cg.call_sites[n] == 1 {
                bonus += s.cost + FUNCTION_OVERHEAD;
                last_call = true;
            }
        }

        // A callee that stays in the program anyway, because something else
        // calls it or takes its address, is copied into every caller that
        // inlines it.
        if !last_call
            && let Some(n) = self.cg.node(callee_g)
            && self.cg.call_sites[n] as usize + self.cg.address_taken[n] as usize > 1
            && p.shared_threshold < threshold
        {
            threshold = p.shared_threshold;
            why = " (callee is called from elsewhere too)";
        }
        if s.cost - bonus <= threshold {
            Decision::Inline { cost: s.cost, bonus, threshold, last_call }
        } else {
            Decision::No(format!("cost {} - bonus {} > threshold {}{why}", s.cost, bonus, threshold))
        }
    }
}

enum Decision {
    Inline { cost: i64, bonus: i64, threshold: i64, last_call: bool },
    No(String),
}

/// The number of instructions placed in the body.
fn live_size(f: &Function) -> usize {
    f.layout.iter().map(|b| f.blocks[b.index()].insts.len()).sum()
}

fn is_alloca(f: &Function, l: LocalId) -> bool {
    matches!(f.locals[l.index()].def, LocalDef::Inst(i) if matches!(f.insts[i.index()].op, Op::Alloca { .. }))
}

/// True if every path from `b` reaches `unreachable` without branching:
/// a cheap stand-in for profile data saying the call is cold.
pub(crate) fn ends_in_unreachable(f: &Function, mut b: BlockId) -> bool {
    for _ in 0..8 {
        let Some(t) = f.terminator(b) else { return false };
        match f.insts[t.index()].op {
            Op::Unreachable => return true,
            Op::Br(next) => b = next,
            _ => return false,
        }
    }
    false
}

/// The size weight of one instruction. Instructions that vanish when
/// inlined, or that fold into addressing modes, count 0.
fn inst_cost(sh: &Shared, entry: BlockId, block: BlockId, inst: &Inst) -> i64 {
    match &inst.op {
        Op::Phi { .. } | Op::Br(_) | Op::Ret(_) | Op::Unreachable => 0,
        Op::Alloca { count, .. } if block == entry && count.is_none_or(|c| matches!(c, Value::Const(_))) => 0,
        Op::Cast { op: CastOp::BitCast, .. } => 0,
        Op::Gep { indices, .. } if indices.iter().all(|v| matches!(v, Value::Const(_))) => 0,
        Op::Switch { cases, .. } => 1 + cases.len() as i64 / 2,
        Op::Call(call) | Op::Invoke { call, .. } | Op::CallBr { call, .. } => {
            match sh.direct_callee(call).and_then(|g| sh.module.global(g).name.as_deref()) {
                Some(n) if is_free_intrinsic(n) => 0,
                Some(n) if n.starts_with("llvm.") => 1,
                _ => CALL_COST + call.args.len() as i64,
            }
        }
        _ => 1,
    }
}

fn is_free_intrinsic(name: &str) -> bool {
    ["llvm.dbg.", "llvm.lifetime.", "llvm.assume", "llvm.experimental.noalias.scope.decl", "llvm.pseudoprobe"]
        .iter()
        .any(|p| name.starts_with(p))
}

fn summarize(sh: &Shared, inliner: &Inliner, g: GlobalId, f: &Function) -> Summary {
    let cg = inliner.cg;
    let m = sh.module;
    let n_params = f.params.len();
    let mut s = Summary {
        cost: 0,
        blocker: None,
        dynamic_alloca: false,
        in_cycle: false,
        has_eh: false,
        nounwind: attrs::has(m, &f.fn_attrs, "nounwind"),
        param_cond_uses: vec![0; n_params],
        param_arith_uses: vec![0; n_params],
        param_mem_uses: vec![0; n_params],
    };
    let mut block = |reason: &'static str| {
        s.blocker.get_or_insert(reason);
    };
    if !f.has_body {
        block("no body");
    }
    let Type::Function { varargs, .. } = sh.ty(f.ty) else { unreachable!() };
    if varargs {
        block("variadic");
    }
    if attrs::has(m, &f.fn_attrs, "noinline") || attrs::has(m, &f.fn_attrs, "optnone") {
        block("noinline");
    }
    if cg.node(g).is_some_and(|n| cg.has_blockaddress[n]) {
        block("its blocks have their address taken");
    }
    if !attrs::definition_is_final(m.global(g).props.linkage) {
        block("its definition may be replaced at link time");
    }
    for p in &f.params {
        for a in ["byval", "inalloca", "preallocated"] {
            if attrs::has_named(m, &p.attrs, a) {
                block("byval, inalloca or preallocated parameter");
            }
        }
    }
    if !f.has_body {
        return s;
    }
    let param_index: HashMap<LocalId, usize> = f.params.iter().enumerate().map(|(i, p)| (p.local, i)).collect();
    let entry = f.entry();
    for &b in &f.layout {
        for &i in &f.blocks[b.index()].insts {
            let inst = &f.insts[i.index()];
            s.cost += inst_cost(sh, entry, b, inst);
            match &inst.op {
                Op::IndirectBr { .. } => block("indirectbr"),
                Op::CallBr { .. } => block("callbr"),
                Op::CatchSwitch { .. } | Op::CatchPad { .. } | Op::CleanupPad { .. } => {
                    block("funclet-based exception handling")
                }
                Op::LandingPad { .. } | Op::Invoke { .. } | Op::Resume(_) => s.has_eh = true,
                Op::Alloca { inalloca: true, .. } => block("inalloca"),
                Op::Alloca { count, .. } if b != entry || count.is_some_and(|c| !matches!(c, Value::Const(_))) => {
                    s.dynamic_alloca = true
                }
                _ => {}
            }
            if let Some(call) = inst.op.call() {
                if call.tail == TailKind::MustTail {
                    block("musttail call");
                }
                let target = sh.direct_callee(call);
                if target.is_some_and(|t| inliner.same_scc(g, t)) {
                    s.in_cycle = true;
                }
                let twice = attrs::has(m, &call.fn_attrs, "returns_twice")
                    || target
                        .and_then(|t| sh.published(t))
                        .is_some_and(|t| attrs::has(m, &t.fn_attrs, "returns_twice"));
                if twice {
                    block("calls a returns_twice function");
                }
                if target.and_then(|t| m.global(t).name.as_deref()).is_some_and(|n| n.starts_with("llvm.localescape")) {
                    block("llvm.localescape");
                }
            }
            // Parameter uses, for the constant and alloca bonuses.
            let kind = match &inst.op {
                Op::ICmp { .. } | Op::FCmp { .. } | Op::Switch { .. } | Op::CondBr { .. } | Op::Select { .. } => 0,
                Op::Load { .. } | Op::Store { .. } | Op::Gep { .. } => 2,
                _ => 1,
            };
            inst.op.for_each_value(|v| {
                if let Value::Local(l) = v
                    && let Some(&p) = param_index.get(l)
                {
                    match kind {
                        0 => s.param_cond_uses[p] += 1,
                        2 => s.param_mem_uses[p] += 1,
                        _ => s.param_arith_uses[p] += 1,
                    }
                }
            });
        }
    }
    s
}

/// Memo tables for debug information while inlining into one function.
#[derive(Default)]
pub struct DebugMemo {
    /// (callee location, call-site location) -> location with `inlinedAt`.
    locations: HashMap<(MdId, MdId), MdId>,
}

/// Replaces call `site` in `block` with a copy of `callee`'s body.
///
/// The block is split after the call. The copied entry block follows the
/// first half, each `ret` becomes a branch to the second half, and the
/// returned values meet in a phi there. Static allocas of the callee move
/// to the caller's entry block, so inlining into a loop does not grow the
/// stack on each iteration; if the callee also has dynamic allocas, the
/// stack is saved before the body and restored after it.
#[allow(clippy::too_many_arguments)]
#[allow(clippy::too_many_arguments)]
pub fn inline_call(
    sh: &Shared,
    caller: &mut Function,
    names: &mut Names,
    memo: &mut DebugMemo,
    site: InstId,
    block: BlockId,
    callee: &Function,
    callee_name: &str,
    dynamic_alloca: bool,
) {
    let done = inline_call_deferred(sh, caller, None, names, memo, site, block, callee, callee_name, dynamic_alloca);
    if let Some((r, value)) = done.replace {
        caller.replace_uses(Value::Local(r), value);
        replace_in_debug_records(sh, caller, r, value);
    }
}

/// A function's block order as a linked list, for inlining many calls:
/// putting a block after another is then constant time, where inserting
/// into the layout costs its length. The layout is rebuilt once at the end.
pub struct LinkedLayout {
    next: Vec<Option<BlockId>>,
    first: Option<BlockId>,
}

impl LinkedLayout {
    pub fn new(f: &Function) -> Self {
        let mut next = vec![None; f.blocks.len()];
        for w in f.layout.windows(2) {
            next[w[0].index()] = Some(w[1]);
        }
        LinkedLayout { next, first: f.layout.first().copied() }
    }

    /// Puts `x` right after `y`.
    pub fn insert_after(&mut self, y: BlockId, x: BlockId) {
        let n = x.index().max(y.index()) + 1;
        if self.next.len() < n {
            self.next.resize(n, None);
        }
        self.next[x.index()] = self.next[y.index()];
        self.next[y.index()] = Some(x);
    }

    /// Writes the order back to the function's layout.
    pub fn finish(self, f: &mut Function) {
        let mut layout = Vec::with_capacity(f.layout.len());
        let mut b = self.first;
        while let Some(x) = b {
            layout.push(x);
            b = self.next.get(x.index()).copied().flatten();
        }
        f.layout = layout;
    }
}

/// What `inline_call_deferred` did, and what it left to the caller.
pub struct Inlined {
    /// The block the rest of the call's block moved to (for a call), where
    /// the body returns.
    pub cont: BlockId,
    /// The call's result and the value that replaces it, for the caller to
    /// replace (with others, in one sweep).
    pub replace: Option<(LocalId, Value)>,
    /// Instructions added to the function, less the call.
    pub added: isize,
}

/// `inline_call`, leaving the replacement of the call's result to the
/// caller, so that many inlined calls cost one sweep over the function
/// rather than one each.
#[allow(clippy::too_many_arguments)]
///
/// With `order`, new blocks are placed in it, and the function's layout is
/// left for `LinkedLayout::finish` to rebuild.
pub fn inline_call_deferred(
    sh: &Shared,
    caller: &mut Function,
    mut order: Option<&mut LinkedLayout>,
    names: &mut Names,
    memo: &mut DebugMemo,
    site: InstId,
    block: BlockId,
    callee: &Function,
    callee_name: &str,
    dynamic_alloca: bool,
) -> Inlined {
    let inst = caller.insts[site.index()].clone();
    let mut added: isize = -1;
    let call = inst.op.call().unwrap().clone();
    let pos = caller.blocks[block.index()].insts.iter().position(|&i| i == site).unwrap();
    let exit_name = names.fresh(&format!("{callee_name}.exit"));

    // 1. The block the body returns to.
    let cont = match &inst.op {
        Op::Call(_) => match order.as_deref_mut() {
            Some(o) => {
                let cont = caller.split_block_unplaced(block, pos + 1, Some(exit_name));
                o.insert_after(block, cont);
                cont
            }
            None => caller.split_block(block, pos + 1, Some(exit_name)),
        },
        Op::Invoke { normal, unwind, .. } => {
            let cont = caller.add_block(Some(exit_name));
            match order.as_deref_mut() {
                Some(o) => o.insert_after(block, cont),
                None => {
                    let at = caller.layout.iter().position(|&b| b == block).unwrap();
                    caller.layout.insert(at + 1, cont);
                }
            }
            caller.push_inst(cont, Op::Br(*normal), None);
            added += 1;
            caller.retarget_phis(*normal, block, cont);
            caller.remove_phi_entries(*unwind, block);
            cont
        }
        _ => unreachable!("only call and invoke sites are inlined"),
    };
    caller.blocks[block.index()].insts.retain(|&i| i != site);
    // Debug records before the call stay before the copied body.
    let site_records = inst.debug_records.clone();

    // 2. Map the callee's locals and blocks into the caller.
    let mut lmap: Vec<Option<Value>> = vec![None; callee.locals.len()];
    for (p, a) in callee.params.iter().zip(&call.args) {
        lmap[p.local.index()] = Some(a.value);
    }
    let mut bmap: Vec<Option<BlockId>> = vec![None; callee.blocks.len()];
    // The body goes between the call's block and the block it returns to.
    match order.as_deref_mut() {
        Some(o) => {
            let mut after = block;
            for &cb in &callee.layout {
                let name = callee.block_name(cb).map(|n| names.fresh(&format!("{n}.i")));
                let nb = caller.add_block(name);
                o.insert_after(after, nb);
                after = nb;
                bmap[cb.index()] = Some(nb);
            }
        }
        None => {
            let at = caller.layout.iter().position(|&b| b == cont).unwrap();
            for (k, &cb) in callee.layout.iter().enumerate() {
                let name = callee.block_name(cb).map(|n| names.fresh(&format!("{n}.i")));
                let nb = caller.add_block(name);
                caller.layout.insert(at + k, nb);
                bmap[cb.index()] = Some(nb);
            }
        }
    }
    let mut copies: Vec<(BlockId, InstId)> = Vec::new();
    for &cb in &callee.layout {
        for &ci in &callee.blocks[cb.index()].insts {
            let src = &callee.insts[ci.index()];
            let result = src.result.map(|r| {
                let l = &callee.locals[r.index()];
                (l.name.as_ref().map(|n| names.fresh(&format!("{n}.i"))), l.ty)
            });
            let ni = caller.add_inst(src.op.clone(), result);
            caller.insts[ni.index()].attachments = src.attachments.clone();
            caller.insts[ni.index()].debug_records = src.debug_records.clone();
            if let Some(r) = src.result {
                lmap[r.index()] = Some(caller.result(ni));
            }
            copies.push((cb, ni));
        }
    }
    added += copies.len() as isize;

    // 3. Rewrite the copies' operands, and place them.
    let site_loc = inst.attachments.iter().find(|a| a.kind == "dbg").map(|a| a.node);
    let callee_entry = callee.entry();
    let caller_entry = caller.entry();
    let mut alloca_at = 0;
    let mut returns: Vec<(Option<Value>, BlockId)> = Vec::new();
    let mut loops: HashMap<MdId, MdId> = HashMap::default();
    for (cb, ni) in copies {
        let nb = bmap[cb.index()].unwrap();
        let copy = &mut caller.insts[ni.index()];
        copy.op.for_each_value_mut(|v| match *v {
            Value::Local(l) => *v = lmap[l.index()].expect("callee value with no definition"),
            Value::Metadata(md) => *v = Value::Metadata(remap_md_locals(sh, md, &lmap)),
            Value::Const(_) => {}
        });
        copy.op.for_each_block_mut(|b| *b = bmap[b.index()].unwrap());
        if let Some(c) = copy.op.call_mut()
            && c.tail == TailKind::Tail
        {
            // `tail` promised not to touch the callee's allocas. Those are now
            // the caller's, and the caller's allocas may be in the arguments.
            c.tail = TailKind::None;
        }
        for a in &mut copy.attachments {
            if a.kind == "llvm.loop" {
                a.node = inlined_loop(sh, memo, &mut loops, a.node, site_loc);
            }
        }
        match site_loc {
            Some(loc) => {
                for a in &mut copy.attachments {
                    if a.kind == "dbg" {
                        a.node = inlined_location(sh, memo, a.node, loc);
                    }
                }
                for r in &mut copy.debug_records {
                    for arg in &mut r.args {
                        *arg = remap_md_locals(sh, *arg, &lmap);
                    }
                    if let Some(last) = r.args.last_mut() {
                        *last = inlined_location(sh, memo, *last, loc);
                    }
                }
            }
            None => {
                copy.attachments.retain(|a| a.kind != "dbg");
                copy.debug_records.clear();
            }
        }
        if let Op::Ret(v) = copy.op {
            returns.push((v, nb));
            copy.op = Op::Br(cont);
        }
        let static_alloca = cb == callee_entry
            && matches!(&copy.op, Op::Alloca { count, .. } if count.is_none_or(|c| matches!(c, Value::Const(_))));
        if static_alloca {
            caller.blocks[caller_entry.index()].insts.insert(alloca_at, ni);
            alloca_at += 1;
        } else {
            caller.blocks[nb.index()].insts.push(ni);
        }
    }
    if s_has_eh(callee) && caller.personality.is_none() {
        caller.personality = callee.personality;
    }

    // 4. Enter the body.
    let saved_sp = if dynamic_alloca {
        let save = sh.global_ref(sh.intrinsics.stacksave.expect("llvm.stacksave not declared"));
        let ptr = sh.intern_type(Type::Ptr(0));
        let fn_ty = sh.intern_type(Type::Function { ret: ptr, params: vec![], varargs: false });
        let i = caller.push_inst(
            block,
            Op::Call(Box::new(plain_call(fn_ty, save, vec![]))),
            Some((Some(names.fresh("savedstack")), ptr)),
        );
        added += 1;
        Some(caller.result(i))
    } else {
        None
    };
    let enter = caller.push_inst(block, Op::Br(bmap[callee_entry.index()].unwrap()), None);
    added += 1;
    caller.insts[enter.index()].debug_records = site_records;

    // 5. The returned value.
    let mut phis = 0;
    let mut replace = None;
    if let Some(r) = inst.result {
        let ty = caller.locals[r.index()].ty;
        let value = match returns.as_slice() {
            [] => sh.poison(ty),
            [(v, _)] => v.unwrap_or(sh.poison(ty)),
            many => {
                let incoming = many.iter().map(|(v, b)| (v.unwrap_or(sh.poison(ty)), *b)).collect();
                let name = caller.locals[r.index()].name.clone().map(|n| names.fresh(&n));
                let phi = caller.add_inst(Op::Phi { fmf: FastMath::default(), incoming }, Some((name, ty)));
                caller.blocks[cont.index()].insts.insert(0, phi);
                phis = 1;
                added += 1;
                caller.result(phi)
            }
        };
        replace = Some((r, value));
    }
    if let Some(sp) = saved_sp {
        let restore = sh.global_ref(sh.intrinsics.stackrestore.expect("llvm.stackrestore not declared"));
        let ptr = sh.intern_type(Type::Ptr(0));
        let void = sh.intern_type(Type::Void);
        let fn_ty = sh.intern_type(Type::Function { ret: void, params: vec![ptr], varargs: false });
        let i = caller.add_inst(Op::Call(Box::new(plain_call(fn_ty, restore, vec![sp]))), None);
        caller.blocks[cont.index()].insts.insert(phis, i);
        added += 1;
    }
    Inlined { cont, replace, added }
}

fn s_has_eh(f: &Function) -> bool {
    f.insts_in_order()
        .any(|i| matches!(f.insts[i.index()].op, Op::LandingPad { .. } | Op::Invoke { .. } | Op::Resume(_)))
}

/// A plain `call` with no attributes.
pub fn plain_call(fn_ty: epitome_ir::types::TypeId, callee: Value, args: Vec<Value>) -> Call {
    Call {
        tail: TailKind::None,
        fmf: FastMath::default(),
        cconv: CallConv::C,
        ret_attrs: vec![],
        addrspace: None,
        fn_ty,
        callee,
        args: args.into_iter().map(|value| CallArg { value, attrs: vec![] }).collect(),
        fn_attrs: vec![],
        bundles: vec![],
    }
}

/// A copy of metadata node `md` with the callee's locals replaced by their
/// values in the caller. Nodes that hold no locals are returned unchanged.
fn remap_md_locals(sh: &Shared, md: MdId, lmap: &[Option<Value>]) -> MdId {
    let data = sh.metadata()[md.index()].data.clone();
    let map = |v: Value| match v {
        Value::Local(l) => lmap[l.index()].unwrap_or(v),
        v => v,
    };
    match data {
        Metadata::Value(v @ Value::Local(_)) if map(v) != v => sh.md_value(map(v)),
        Metadata::Node { distinct, kind, fields }
            if fields.iter().any(|f| matches!(f.value, MdFieldValue::Value(v) if map(v) != v)) =>
        {
            let fields = fields
                .into_iter()
                .map(|f| match f.value {
                    MdFieldValue::Value(v) => MdField { name: f.name, value: MdFieldValue::Value(map(v)) },
                    _ => f,
                })
                .collect();
            sh.add_md(Metadata::Node { distinct, kind, fields })
        }
        _ => md,
    }
}

/// Points debug records that refer to local `from` at `to` instead.
pub fn replace_in_debug_records(sh: &Shared, f: &mut Function, from: LocalId, to: Value) {
    let mut lmap: Vec<Option<Value>> = vec![None; f.locals.len()];
    lmap[from.index()] = Some(to);
    replace_all_in_debug_records(sh, f, &lmap);
}

/// Every replacement `lmap` gives (by local), in one walk of the debug
/// records.
pub fn replace_all_in_debug_records(sh: &Shared, f: &mut Function, lmap: &[Option<Value>]) {
    let order: Vec<InstId> = f.insts_in_order().collect();
    for i in order {
        for r in &mut f.insts[i.index()].debug_records {
            for arg in &mut r.args {
                *arg = remap_md_locals(sh, *arg, lmap);
            }
        }
    }
}

/// A copy of loop metadata `id` for one inlined copy of a loop. Loop IDs
/// are distinct, so each copy gets its own; the locations inside get the
/// call site added to their `inlinedAt` chain, or are dropped when the call
/// has no location.
fn inlined_loop(
    sh: &Shared,
    memo: &mut DebugMemo,
    copies: &mut HashMap<MdId, MdId>,
    id: MdId,
    site: Option<MdId>,
) -> MdId {
    if let Some(&done) = copies.get(&id) {
        return done;
    }
    let Metadata::Tuple { distinct, ops } = sh.metadata()[id.index()].data.clone() else { return id };
    let new = sh.add_md(Metadata::Tuple { distinct, ops: vec![] });
    copies.insert(id, new);
    let mut new_ops = Vec::new();
    for op in ops {
        let Some(x) = op else {
            new_ops.push(None);
            continue;
        };
        if x == id {
            new_ops.push(Some(new));
            continue;
        }
        let is_location = matches!(&sh.metadata()[x.index()].data, Metadata::Node { kind, .. } if kind == "DILocation");
        match (is_location, site) {
            (true, Some(loc)) => new_ops.push(Some(inlined_location(sh, memo, x, loc))),
            (true, None) => {}
            (false, _) => new_ops.push(Some(x)),
        }
    }
    sh.set_md(new, Metadata::Tuple { distinct, ops: new_ops });
    new
}

/// The location `loc` from the callee, as seen after inlining at call-site
/// location `site`: the end of its `inlinedAt` chain gets `site` added.
fn inlined_location(sh: &Shared, memo: &mut DebugMemo, loc: MdId, site: MdId) -> MdId {
    if let Some(&done) = memo.locations.get(&(loc, site)) {
        return done;
    }
    let data = sh.metadata()[loc.index()].data.clone();
    let Metadata::Node { kind, mut fields, .. } = data else { return loc };
    if kind != "DILocation" {
        return loc;
    }
    let outer = match fields.iter().find(|f| f.name.as_deref() == Some("inlinedAt")).map(|f| &f.value) {
        Some(MdFieldValue::Md(x)) => inlined_location(sh, memo, *x, site),
        _ => site,
    };
    match fields.iter_mut().find(|f| f.name.as_deref() == Some("inlinedAt")) {
        Some(f) => f.value = MdFieldValue::Md(outer),
        None => {
            let at = fields.iter().position(|f| f.name.as_deref() == Some("scope")).map_or(fields.len(), |p| p + 1);
            fields.insert(at, MdField { name: Some("inlinedAt".into()), value: MdFieldValue::Md(outer) });
        }
    }
    let new = sh.add_md(Metadata::Node { distinct: false, kind, fields });
    memo.locations.insert((loc, site), new);
    new
}
