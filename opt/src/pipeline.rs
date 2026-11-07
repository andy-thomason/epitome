//! The pipeline: inlining, recursion removal and combines, run bottom-up
//! over the call graph on many threads.
//!
//! 1. Build the call graph and find its SCCs (in parallel per function).
//! 2. Declare the intrinsics the passes may call, while the module can
//!    still change.
//! 3. Split the module for parallel work (`ctx::parallel_phase`) and run
//!    each SCC as a task once the SCCs it calls are finished
//!    (`par::run_dag`). A task takes its functions' bodies and:
//!    - inlines calls to functions in other, finished SCCs;
//!    - if the SCC is a cycle, collapses it to one function and turns the
//!      remaining self-calls into loops;
//!    - runs the combiner on each function;
//!    - publishes the bodies, which unblocks the SCCs that call them.
//! 4. Put the module back together, merge identical metadata, and delete
//!    internal functions that nothing refers to any more.
//!
//! The result is the same for any number of threads: a task reads only its
//! own bodies and the published bodies of its callees, which are final.

use std::sync::atomic::{AtomicUsize, Ordering};

use epitome_ir::metadata;
use epitome_ir::module::*;
use epitome_ir::types::Type;

use crate::callgraph::CallGraph;
use crate::ctx::{Intrinsics, parallel_phase};
use crate::inline::{InlineParams, Inliner, OptLevel};
use crate::recursion::{self, RecursionMode};
use crate::{attrs, par};

#[derive(Clone, Debug)]
pub struct Options {
    /// Threads to use; 0 means one per core.
    pub threads: usize,
    pub level: OptLevel,
    pub inline: bool,
    pub combine: bool,
    pub cse: bool,
    /// The combiner's rules and steps about memory: forwarding stored
    /// values, reusing loads, deleting dead stores (memory SSA).
    pub memory: bool,
    /// Marking calls that may reuse the frame `tail` (`tailmark.rs`).
    pub tailmark: bool,
    pub mem2reg: bool,
    pub sroa: bool,
    pub simplifycfg: bool,
    /// Loop rotation, then induction variables and trip counts.
    pub loops: bool,
    pub recursion: RecursionMode,
    /// Autochop: how far elective changes may go (`epitome_ir::chop`).
    pub chop: Option<epitome_ir::chop::Limit>,
    /// Combine rules to try before the standard ones.
    pub extra_rules: Option<String>,
}

impl Default for Options {
    fn default() -> Self {
        Options {
            threads: 0,
            level: OptLevel::O2,
            inline: true,
            combine: true,
            cse: true,
            memory: true,
            tailmark: true,
            mem2reg: true,
            sroa: true,
            simplifycfg: true,
            loops: true,
            recursion: RecursionMode::Tail,
            chop: None,
            extra_rules: None,
        }
    }
}

#[derive(Debug, Default)]
pub struct Report {
    pub threads: usize,
    pub functions: usize,
    pub sccs: usize,
    /// One line per call-graph cycle, as found before the passes ran.
    pub cycles: Vec<String>,
    pub inlined: usize,
    /// How often each combine rule fired, over the whole module.
    pub combines: crate::combine::Stats,
    /// What CSE removed, over the whole module.
    pub cse: crate::cse::Stats,
    /// Runs of stores made memsets.
    pub memcpyopt: crate::memcpyopt::Stats,
    /// What moved out of loops.
    pub licm: crate::licm::Stats,
    /// What mem2reg promoted, over the whole module.
    pub mem2reg: crate::mem2reg::Stats,
    /// What SROA split, over the whole module.
    pub sroa: crate::sroa::Stats,
    /// What CFG simplification did, over the whole module.
    pub simplifycfg: crate::simplifycfg::Stats,
    pub rotated: usize,
    pub indvars: crate::indvars::Stats,
    pub tail_calls: usize,
    /// Calls marked `tail` (`tailmark.rs`).
    pub tail_marked: usize,
    pub accumulator_calls: usize,
    pub frame_calls: usize,
    /// Remarks per function, in module order.
    pub remarks: Vec<(String, Vec<String>)>,
    /// Autochop: the functions counted, in their order, each one's count,
    /// and where the limit stopped (function, steps made, the last step).
    pub chop_names: Vec<String>,
    pub chop_counts: Vec<epitome_ir::chop::Counter>,
    pub chop_stopped: Option<(usize, u64, Option<String>)>,
    /// The first step refused there.
    pub chop_next: Option<String>,
    /// Functions deleted because nothing used them any more.
    pub deleted: Vec<String>,
    /// The size of all functions, and of the longest chain of SCCs that
    /// must run one after another, in instructions. Their ratio bounds the
    /// speed-up from threads.
    pub total_work: u64,
    pub critical_path: u64,
    /// The same two measures in time: the sum of every task's run time, and
    /// the longest chain of dependent tasks by run time.
    pub task_time: std::time::Duration,
    pub task_critical_path: std::time::Duration,
    /// Time spent in each step, in order.
    pub timings: Vec<(&'static str, std::time::Duration)>,
}

/// Optimises `m`. Functions marked `"epitome-opaque"` are for another tool
/// (Epitome's `docs/design/partial-machines.md`): their bodies are out of
/// the module while it is optimised, so that no pass reads, changes or
/// inlines them, and a call to one is a call to a declaration, which may
/// read and write any memory its own attributes allow.
pub fn run(m: &mut Module, opts: &Options) -> Report {
    let fenced = take_opaque_bodies(m);
    let report = optimise(m, opts);
    for (g, body) in fenced {
        if let GlobalKind::Function(f) = &mut m.globals[g.index()].kind {
            f.locals = body.locals;
            f.blocks = body.blocks;
            f.layout = body.layout;
            f.insts = body.insts;
            f.has_body = true;
        }
    }
    report
}

/// The bodies of `m`'s `"epitome-opaque"` functions, taken out, which
/// leaves each a declaration (its parameters stay).
fn take_opaque_bodies(m: &mut Module) -> Vec<(GlobalId, Function)> {
    let opaque: Vec<GlobalId> = m
        .global_order
        .iter()
        .copied()
        .filter(|&g| match &m.globals[g.index()].kind {
            GlobalKind::Function(f) => f.has_body && epitome_ir::fits::opaque(m, &f.fn_attrs),
            _ => false,
        })
        .collect();
    let mut out = Vec::new();
    for g in opaque {
        let GlobalKind::Function(f) = &mut m.globals[g.index()].kind else { continue };
        let mut body = Function { locals: f.locals[..f.params.len()].to_vec(), ..Default::default() };
        std::mem::swap(&mut body.locals, &mut f.locals);
        body.blocks = std::mem::take(&mut f.blocks);
        body.layout = std::mem::take(&mut f.layout);
        body.insts = std::mem::take(&mut f.insts);
        f.has_body = false;
        out.push((g, body));
    }
    out
}

fn optimise(m: &mut Module, opts: &Options) -> Report {
    let threads = if opts.threads == 0 { par::default_threads() } else { opts.threads };
    let mut clock = std::time::Instant::now();
    let mut timings = Vec::new();
    let mut lap = |name: &'static str| {
        let now = std::time::Instant::now();
        timings.push((name, now - clock));
        clock = now;
    };
    let cg = CallGraph::build(m, threads);
    let mut report = Report {
        threads,
        functions: cg.nodes.len(),
        sccs: cg.sccs.len(),
        cycles: cg.describe_cycles(m),
        ..Default::default()
    };
    let intrinsics = declare_intrinsics(m);
    let deps = cg.scc_deps();
    let priority = cg.scc_priorities(&deps);
    report.total_work = cg.size.iter().map(|&s| s.max(1) as u64).sum();
    report.critical_path = priority.iter().copied().max().unwrap_or(0);
    lap("call graph");
    let inliner = Inliner::new(InlineParams::for_level(opts.level), &cg, m.globals.len());
    let inlined = AtomicUsize::new(0);
    let combines = std::sync::Mutex::new(crate::combine::Stats::default());
    let cse_stats = std::sync::Mutex::new(crate::cse::Stats::default());
    let memcpy_stats = std::sync::Mutex::new(crate::memcpyopt::Stats::default());
    let licm_stats = std::sync::Mutex::new(crate::licm::Stats::default());
    let m2r_stats = std::sync::Mutex::new(crate::mem2reg::Stats::default());
    let sroa_stats = std::sync::Mutex::new(crate::sroa::Stats::default());
    let cfg_stats = std::sync::Mutex::new(crate::simplifycfg::Stats::default());
    let extra = opts.extra_rules.as_deref().map(|r| crate::combine::with_extra(r).unwrap_or_else(|e| panic!("{e}")));
    let matcher = extra.as_ref().unwrap_or_else(|| crate::combine::standard());
    let rotated = AtomicUsize::new(0);
    let iv_stats = std::sync::Mutex::new(crate::indvars::Stats::default());
    let tail_calls = AtomicUsize::new(0);
    let tail_marked = AtomicUsize::new(0);
    let accumulator_calls = AtomicUsize::new(0);
    let frame_calls = AtomicUsize::new(0);
    let task_nanos: Vec<std::sync::atomic::AtomicU64> = (0..cg.sccs.len()).map(|_| Default::default()).collect();
    // Autochop: functions with bodies are numbered in module order, and
    // each one's elective changes are counted while its passes run.
    let chop_order: Vec<GlobalId> = m
        .global_order
        .iter()
        .copied()
        .filter(|&g| matches!(&m.global(g).kind, GlobalKind::Function(f) if f.has_body))
        .collect();
    let mut chop_index = vec![usize::MAX; m.globals.len()];
    for (k, g) in chop_order.iter().enumerate() {
        chop_index[g.index()] = k;
    }
    let chop = epitome_ir::chop::Session::new(chop_order.len(), opts.chop);
    let counted = |g: GlobalId| chop.enter(chop_index[g.index()]);

    let ((), remarks) = parallel_phase(m, intrinsics, |sh| {
        par::run_dag(&deps, &priority, threads, |s| {
            let started = std::time::Instant::now();
            let mut bodies: Vec<(GlobalId, Box<Function>)> = cg.sccs[s]
                .iter()
                .map(|&v| cg.nodes[v])
                .filter(|&g| sh.published(g).is_none())
                .map(|g| (g, sh.take(g)))
                .collect();
            // Locals in memory become SSA values first, so every later pass
            // sees them; and again after inlining, which often makes an
            // alloca promotable by bringing in the code it was passed to.
            let promote = |g: GlobalId, f: &mut Function| {
                if opts.sroa {
                    let s = crate::sroa::run(sh, f);
                    if s.split > 0 {
                        sh.remark(g, format!("sroa: {} allocas split into {} pieces", s.split, s.pieces));
                        let mut all = sroa_stats.lock().unwrap();
                        all.split += s.split;
                        all.pieces += s.pieces;
                    }
                }
                if !opts.mem2reg {
                    return;
                }
                let s = crate::mem2reg::run(sh, f);
                if s.promoted > 0 {
                    sh.remark(
                        g,
                        format!(
                            "mem2reg: {} allocas promoted ({} loads, {} stores removed)",
                            s.promoted, s.loads, s.stores
                        ),
                    );
                    let mut all = m2r_stats.lock().unwrap();
                    all.promoted += s.promoted;
                    all.loads += s.loads;
                    all.stores += s.stores;
                }
            };
            if opts.mem2reg || opts.sroa {
                for (g, f) in &mut bodies {
                    let _counted = counted(*g);
                    promote(*g, f);
                }
            }
            if opts.inline {
                for (g, f) in &mut bodies {
                    let _counted = counted(*g);
                    let n = inliner.run_on(sh, *g, f);
                    inlined.fetch_add(n, Ordering::Relaxed);
                    if n > 0 && (opts.mem2reg || opts.sroa) {
                        promote(*g, f);
                    }
                }
            }
            if opts.recursion != RecursionMode::Off && cg.is_recursive(s) && !bodies.is_empty() {
                let remove = |g: GlobalId, f: &mut Function| {
                    let m = sh.module;
                    if attrs::has(m, &f.fn_attrs, "optnone") {
                        return;
                    }
                    let mode = if attrs::has(m, &f.fn_attrs, "minsize") { RecursionMode::Tail } else { opts.recursion };
                    if !epitome_ir::chop::allow(|| format!("recursion: remove @{}'s recursion", sh.name(g))) {
                        return;
                    }
                    let block_addresses = cg.node(g).is_some_and(|n| cg.has_blockaddress[n]);
                    match recursion::eliminate(sh, g, f, mode, block_addresses) {
                        Ok(o) => {
                            tail_calls.fetch_add(o.tail_calls, Ordering::Relaxed);
                            accumulator_calls.fetch_add(o.accumulator_calls, Ordering::Relaxed);
                            frame_calls.fetch_add(o.frame_calls.len(), Ordering::Relaxed);
                            if o.accumulator_calls > 0 {
                                let op = o.accumulator_op.map_or("", |op| op.keyword());
                                sh.remark(
                                    g,
                                    format!(
                                        "recursion: {} accumulator call(s) now jump back to the start, folding with {op}",
                                        o.accumulator_calls
                                    ),
                                );
                            }
                            if o.tail_calls > 0 {
                                sh.remark(
                                    g,
                                    format!("recursion: {} tail call(s) now jump back to the start", o.tail_calls),
                                );
                            }
                            if !o.frame_calls.is_empty() {
                                let sizes: Vec<String> = o.frame_calls.iter().map(|n| n.to_string()).collect();
                                sh.remark(
                                    g,
                                    format!(
                                        "recursion: {} call(s) now push a frame on an explicit stack (values saved per frame: {})",
                                        o.frame_calls.len(),
                                        sizes.join(", ")
                                    ),
                                );
                            }
                            if o.kept > 0 {
                                sh.remark(g, format!("recursion: {} recursive call(s) kept", o.kept));
                            }
                        }
                        Err(e) => sh.remark(g, format!("recursion: not removed: {e}")),
                    }
                };
                // Self-calls first, so that inlining members into each other
                // below terminates.
                for (g, f) in &mut bodies {
                    let _counted = counted(*g);
                    remove(*g, f);
                }
                // Collapsing the cycle is one step, of its first function.
                let collapse = bodies.len() > 1 && {
                    let _counted = counted(bodies[0].0);
                    epitome_ir::chop::allow(|| format!("recursion: collapse the cycle of @{}", sh.name(bodies[0].0)))
                };
                if collapse {
                    for (e, result) in recursion::collapse_scc(sh, &cg, &mut bodies, 64) {
                        let g = bodies[e].0;
                        match result {
                            Ok(()) => {
                                let others: Vec<String> = bodies
                                    .iter()
                                    .filter(|(x, _)| *x != g)
                                    .map(|(x, _)| format!("@{}", sh.name(*x)))
                                    .collect();
                                sh.remark(g, format!("recursion: inlined {} to break the cycle", others.join(", ")));
                                let (g, f) = &mut bodies[e];
                                let _counted = counted(*g);
                                remove(*g, f);
                            }
                            Err(err) => sh.remark(g, format!("recursion: cycle not collapsed: {err}")),
                        }
                    }
                }
            }
            // Simplify before publishing, so callers see the simpler body:
            // combine, then CSE, then combine again if CSE found anything,
            // since merged values can make new patterns (`x - y` with
            // `y` now known to be `x`).
            let combine = |g: GlobalId, f: &mut Function, seeds: Option<&[InstId]>| {
                let stats = crate::combine::run_with(sh, f, matcher, seeds, opts.memory);
                if stats.total() > 0 {
                    let list: Vec<String> = stats.fired.iter().map(|(k, v)| format!("{k} x{v}")).collect();
                    sh.remark(g, format!("combine: {}", list.join(", ")));
                    combines.lock().unwrap().merge(&stats);
                }
            };
            // A few rounds of CFG simplification, each followed by the
            // combiner on what it touched, which may fold more branches.
            let simplify_cfg = |g: GlobalId, f: &mut Function| {
                if !opts.simplifycfg {
                    return;
                }
                let block_addresses = cg.node(g).is_some_and(|n| cg.has_blockaddress[n]);
                for _ in 0..3 {
                    let out = crate::simplifycfg::run(sh, f, block_addresses);
                    let s = out.stats;
                    if s.total() == 0 {
                        break;
                    }
                    sh.remark(
                            g,
                            format!(
                                "simplifycfg: {} unreachable blocks removed, {} empty blocks bypassed, {} selects made, {} blocks merged",
                                s.unreachable, s.bypassed, s.selects, s.merged
                            ),
                        );
                    {
                        let mut all = cfg_stats.lock().unwrap();
                        all.unreachable += s.unreachable;
                        all.bypassed += s.bypassed;
                        all.selects += s.selects;
                        all.speculated += s.speculated;
                        all.merged += s.merged;
                    }
                    if !opts.combine || out.touched.is_empty() {
                        break;
                    }
                    combine(g, f, Some(&out.touched));
                }
            };
            for (g, f) in &mut bodies {
                let _counted = counted(*g);
                if opts.combine {
                    combine(*g, f, None);
                    // Calls on constants that can be worked out, now that
                    // the arguments are folded.
                    let n = crate::constcall::run(sh, *g, f);
                    if n > 0 {
                        sh.remark(*g, format!("constcall: {n} calls worked out"));
                        combine(*g, f, None);
                    }
                }
                // Then the CFG, and the combiner again on what that touched,
                // which may fold more branches.
                simplify_cfg(*g, f);
                // Loops: rotate them to test at the bottom, tidy what that
                // copied, then work out counters and trip counts.
                if opts.loops {
                    let r = crate::rotate::run(sh, f);
                    if r.rotated > 0 {
                        sh.remark(*g, format!("rotate: {} loops rotated", r.rotated));
                        rotated.fetch_add(r.rotated, Ordering::Relaxed);
                        if opts.combine {
                            combine(*g, f, None);
                        }
                        simplify_cfg(*g, f);
                    }
                    // What does not change round a loop, out of it.
                    let h = crate::licm::run(sh, f);
                    if h.hoisted > 0 {
                        sh.remark(*g, format!("licm: {} moved out of loops ({} loads)", h.hoisted, h.loads));
                        let mut all = licm_stats.lock().unwrap();
                        all.hoisted += h.hoisted;
                        all.loads += h.loads;
                    }
                    let s = crate::indvars::run(sh, f);
                    for (h, n) in &s.trip_counts {
                        sh.remark(*g, format!("indvars: loop {h} runs {n} times when entered"));
                    }
                    if s.changed() {
                        sh.remark(
                            *g,
                            format!(
                                "indvars: {} exact exit tests, {} exit values, {} counters widened",
                                s.exact_exits, s.exit_values, s.widened
                            ),
                        );
                        if opts.combine {
                            combine(*g, f, None);
                        }
                    }
                    // Loops that only set memory to a byte, as memsets; then
                    // the loop around them, once what is left of the inner one
                    // is folded away.
                    if opts.memory {
                        for _ in 0..4 {
                            let s = crate::loopidiom::run(sh, f);
                            if s.memsets == 0 {
                                break;
                            }
                            sh.remark(*g, format!("loopidiom: {} loops made memsets", s.memsets));
                            if opts.combine {
                                combine(*g, f, None);
                            }
                            simplify_cfg(*g, f);
                        }
                    }
                    let r = crate::lsr::run(sh, f);
                    if r.changed() {
                        sh.remark(
                            *g,
                            format!(
                                "lsr: {} arrays stepped by pointer, {} counters counting down",
                                r.pointers, r.countdowns
                            ),
                        );
                        if opts.combine {
                            combine(*g, f, None);
                        }
                    }
                    let mut all = iv_stats.lock().unwrap();
                    all.trip_counts.extend(s.trip_counts);
                    all.exact_exits += s.exact_exits;
                    all.exit_values += s.exit_values;
                    all.widened += s.widened;
                }
                if opts.cse {
                    let crate::cse::Outcome { stats: s, changed } = crate::cse::run(sh, f);
                    if s.total() > 0 {
                        sh.remark(
                            *g,
                            format!(
                                "cse: {} merged, {} loads reused, {} stores forwarded",
                                s.merged, s.loads, s.forwarded
                            ),
                        );
                        let mut all = cse_stats.lock().unwrap();
                        all.merged += s.merged;
                        all.loads += s.loads;
                        all.forwarded += s.forwarded;
                        drop(all);
                        if opts.combine {
                            combine(*g, f, Some(&changed));
                        }
                    }
                }
                // Runs of stores as memsets, once the stores that were dead
                // or forwarded are gone.
                if opts.memory {
                    let s = crate::memcpyopt::run(sh, f);
                    if s.memsets > 0 {
                        sh.remark(*g, format!("memcpyopt: {} stores made {} memsets", s.stores, s.memsets));
                        // The stores' addresses may now be unused.
                        if opts.combine {
                            combine(*g, f, None);
                        }
                        let mut all = memcpy_stats.lock().unwrap();
                        all.memsets += s.memsets;
                        all.stores += s.stores;
                    }
                }
                // Last, once the allocas that could go are gone: which calls
                // may reuse the frame.
                if opts.tailmark {
                    let n = crate::tailmark::run(sh, f);
                    tail_marked.fetch_add(n, Ordering::Relaxed);
                }
            }
            for (g, f) in bodies {
                sh.publish(g, f);
            }
            task_nanos[s].store(started.elapsed().as_nanos() as u64, Ordering::Relaxed);
        });
    });

    lap("inline and recursion (parallel)");
    // Longest chain by measured time. Callees come first in `sccs`.
    let nanos: Vec<u64> = task_nanos.iter().map(|t| t.load(Ordering::Relaxed)).collect();
    let mut finish = vec![0u64; nanos.len()];
    for s in 0..nanos.len() {
        finish[s] = nanos[s] + deps[s].iter().map(|&d| finish[d]).max().unwrap_or(0);
    }
    report.task_time = std::time::Duration::from_nanos(nanos.iter().sum());
    report.task_critical_path = std::time::Duration::from_nanos(finish.iter().copied().max().unwrap_or(0));
    report.inlined = inlined.into_inner();
    report.combines = combines.into_inner().unwrap();
    report.cse = cse_stats.into_inner().unwrap();
    report.memcpyopt = memcpy_stats.into_inner().unwrap();
    report.licm = licm_stats.into_inner().unwrap();
    report.mem2reg = m2r_stats.into_inner().unwrap();
    report.sroa = sroa_stats.into_inner().unwrap();
    report.simplifycfg = cfg_stats.into_inner().unwrap();
    report.rotated = rotated.into_inner();
    report.indvars = iv_stats.into_inner().unwrap();
    report.tail_calls = tail_calls.into_inner();
    report.tail_marked = tail_marked.into_inner();
    report.accumulator_calls = accumulator_calls.into_inner();
    report.frame_calls = frame_calls.into_inner();
    report.remarks = remarks
        .into_iter()
        .map(|(g, list)| (m.global(g).name.clone().unwrap_or_else(|| g.0.to_string()), list))
        .collect();
    metadata::unique_metadata(m);
    lap("merge metadata");
    report.deleted = delete_dead_functions(m, threads, &chop, &chop_index);
    lap("delete dead functions");
    report.chop_names =
        chop_order.iter().map(|&g| m.global(g).name.clone().unwrap_or_else(|| g.0.to_string())).collect();
    report.chop_counts = chop.counts();
    report.chop_stopped = chop.stopped_at();
    report.chop_next = chop.refused_first();
    report.timings = timings;
    report
}

/// Finds or declares `llvm.stacksave.p0` and `llvm.stackrestore.p0`.
/// Unused declarations are deleted at the end of the pipeline.
fn declare_intrinsics(m: &mut Module) -> Intrinsics {
    let ptr = m.types.ptr(0);
    let void = m.types.void();
    let save_ty = m.types.intern(Type::Function { ret: ptr, params: vec![], varargs: false });
    let restore_ty = m.types.intern(Type::Function { ret: void, params: vec![ptr], varargs: false });
    let (i8t, i64t, i1t) = (m.types.int(8), m.types.int(64), m.types.int(1));
    let memset_ty = m.types.intern(Type::Function { ret: void, params: vec![ptr, i8t, i64t, i1t], varargs: false });
    let fsh = [8u32, 16, 32, 64].map(|w| {
        let t = m.types.int(w);
        (w, t, m.types.intern(Type::Function { ret: t, params: vec![t, t, t], varargs: false }))
    });
    let mut declare = |name: &str, ty, params: &[epitome_ir::types::TypeId]| -> GlobalId {
        if let Some(i) = m.globals.iter().position(|g| g.name.as_deref() == Some(name)) {
            return GlobalId(i as u32);
        }
        let mut f = Function { ty, ..Default::default() };
        for (k, &t) in params.iter().enumerate() {
            let local = f.add_local(None, t, LocalDef::Param(k as u32));
            f.params.push(Param { local, attrs: vec![] });
        }
        let g = GlobalId(m.globals.len() as u32);
        m.globals.push(Global {
            name: Some(name.to_string()),
            props: GlobalProps::default(),
            attachments: vec![],
            kind: GlobalKind::Function(Box::new(f)),
        });
        m.global_order.push(g);
        g
    };
    Intrinsics {
        stacksave: Some(declare("llvm.stacksave.p0", save_ty, &[])),
        stackrestore: Some(declare("llvm.stackrestore.p0", restore_ty, &[ptr])),
        memset: Some(declare("llvm.memset.p0.i64", memset_ty, &[ptr, i8t, i64t, i1t])),
        fshl: fsh.map(|(w, t, ty)| Some(declare(&format!("llvm.fshl.i{w}"), ty, &[t, t, t]))),
    }
}

/// Deletes internal functions with no remaining references, and unused
/// intrinsic declarations. Deleting a function removes its calls, which can
/// leave its callees unused in turn, so this works through a list.
/// Returns the names of the deleted functions, intrinsics aside.
///
/// Deleting a function with a body is an elective change of that function
/// (autochop); declarations go without asking.
fn delete_dead_functions(
    m: &mut Module,
    threads: usize,
    chop: &epitome_ir::chop::Session,
    chop_index: &[usize],
) -> Vec<String> {
    let cg = CallGraph::build(m, threads);
    let mut calls = cg.call_sites.clone();
    let deletable = |n: usize| -> bool {
        let global = m.global(cg.nodes[n]);
        let local = matches!(global.props.linkage, Linkage::Internal | Linkage::Private);
        let intrinsic = global.name.as_deref().is_some_and(|s| s.starts_with("llvm."));
        let GlobalKind::Function(f) = &global.kind else { return false };
        !cg.address_taken[n] && !cg.has_blockaddress[n] && (local || (intrinsic && !f.has_body))
    };
    let mut dead = vec![false; cg.nodes.len()];
    let mut work: Vec<usize> = (0..cg.nodes.len()).filter(|&n| calls[n] == 0 && deletable(n)).collect();
    while let Some(n) = work.pop() {
        if dead[n] {
            continue;
        }
        let g = cg.nodes[n];
        if let GlobalKind::Function(f) = &m.global(g).kind
            && f.has_body
            && chop_index[g.index()] != usize::MAX
        {
            let _counted = chop.enter(chop_index[g.index()]);
            let name = m.global(g).name.clone().unwrap_or_default();
            if !epitome_ir::chop::allow(|| format!("delete @{name}, no longer used")) {
                continue;
            }
        }
        dead[n] = true;
        for &c in &cg.sites[n] {
            calls[c] -= 1;
            if calls[c] == 0 && !dead[c] && deletable(c) {
                work.push(c);
            }
        }
    }
    let mut deleted = Vec::new();
    for (n, &g) in cg.nodes.iter().enumerate() {
        if dead[n] {
            let name = m.global(g).name.clone().unwrap_or_default();
            if !name.starts_with("llvm.") {
                deleted.push(name);
            }
        }
    }
    let mut gone = vec![false; m.globals.len()];
    cg.nodes.iter().enumerate().filter(|&(n, _)| dead[n]).for_each(|(_, g)| gone[g.index()] = true);
    m.global_order.retain(|g| !gone[g.index()]);
    deleted
}
