//! `epitome-opt`: reads a `.ll` file, runs the interprocedural pipeline
//! (inlining and recursion removal) and prints the result.

use std::process::ExitCode;
use std::time::Instant;

use epitome_ir::module::GlobalKind;
use epitome_opt::inline::OptLevel;
use epitome_opt::pipeline::{self, Options};
use epitome_opt::recursion::RecursionMode;

const USAGE: &str = "usage: epitome-opt [options] <file.ll>

  -O1 -O2 -O3 -Os -Oz     optimisation level (default -O2)
  -j <n>                  threads (default: one per core)
  -o <file>               write the result here instead of stdout (- is
                          stdout; an input of - is stdin)
  --passes=<list>         which passes to run, from sroa, mem2reg, inline,
                          recursion, combine, simplifycfg, loops, cse,
                          memory (the combiner's memory rules) and
                          tailmark (marking calls tail),
                          separated by commas (default: all)
  --no-inline             do not inline
  --no-combine            do not run the combiner
  --no-cse                do not run common-subexpression elimination
  --no-memory             do not forward stores, reuse loads or delete dead
                          stores (the combiner's memory rules)
  --no-mem2reg            do not promote allocas to SSA values
  --no-sroa               do not split allocas into pieces
  --no-simplifycfg        do not simplify the control-flow graph
  --no-loops              do not rotate loops or simplify their counters
  --recursion=<mode>      off; tail (default): tail and accumulator calls
                          become loops, other recursion stays; or stack:
                          all recursion, with an explicit stack
  --remarks               print each decision to stderr
  --print-cycles          print the call graph's cycles to stderr
  --stats                 print counts and timings to stderr
  --verify                check the output is well-formed SSA
  --chop=F[:T]            autochop: functions before F (in module order,
                          from 0) get every elective change, F its first T,
                          the rest none; also EPITOME_OPT_CHOP. Reports
                          where it stopped, and the last change made
  --chop-counts           print each function's count of elective changes
  --rules=<file>          combine rules to try before the standard ones
  --target=<path>         a machine description (a .ll file, or a
                          directory of them): after optimising, warn of
                          each use of what it lacks (!target.lacks), as
                          epitome-llc would refuse it
  --print-memssa          print memory SSA for the result (each access, and
                          the write that reaches each load and store) as
                          comments; with --passes= alone, for the input";

/// What a description says its machine lacks: a `.ll` file, or every one
/// in a directory.
fn read_machine(path: &str) -> Result<epitome_ir::fits::Machine, String> {
    let p = std::path::Path::new(path);
    let mut files: Vec<std::path::PathBuf> = if p.is_dir() {
        std::fs::read_dir(p)
            .map_err(|e| e.to_string())?
            .filter_map(|e| e.ok().map(|e| e.path()))
            .filter(|f| f.extension().is_some_and(|x| x == "ll"))
            .collect()
    } else {
        vec![p.to_path_buf()]
    };
    files.sort();
    let mut mc = epitome_ir::fits::Machine { arg_regs: usize::MAX, ..Default::default() };
    for f in files {
        let text = std::fs::read_to_string(&f).map_err(|e| format!("{}: {e}", f.display()))?;
        let d = epitome_ir::parse_module(&text).map_err(|e| format!("{}:{e}", f.display()))?;
        let one = epitome_ir::fits::Machine::read(&d)?;
        mc.lacks.extend(one.lacks);
        mc.widest = mc.widest.max(one.widest);
        mc.launch |= one.launch;
    }
    Ok(mc)
}

/// A runaway compilation stops at a memory limit (`epitome_ir::memlimit`;
/// `EPITOME_MEMORY_LIMIT_MB`, default 8192) rather than taking the machine
/// down.
#[global_allocator]
static MEMORY: epitome_ir::memlimit::Limited = epitome_ir::memlimit::Limited;

fn main() -> ExitCode {
    epitome_ir::memlimit::set_from_env(8192);
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut opts = Options::default();
    let mut input = None;
    let mut output = None;
    let (mut remarks, mut cycles, mut stats, mut verify) = (false, false, false, false);
    let mut memssa = false;
    let (mut chop, mut chop_counts) = (None, false);
    let mut target: Option<String> = None;
    let mut it = args.iter();
    while let Some(a) = it.next() {
        match a.as_str() {
            "-j" => match it.next().and_then(|n| n.parse().ok()) {
                Some(n) => opts.threads = n,
                None => return usage("-j needs a number"),
            },
            "-o" => output = it.next().cloned(),
            "--no-inline" => opts.inline = false,
            "--no-combine" => opts.combine = false,
            "--no-cse" => opts.cse = false,
            "--no-memory" => opts.memory = false,
            "--no-mem2reg" => opts.mem2reg = false,
            "--no-sroa" => opts.sroa = false,
            "--no-simplifycfg" => opts.simplifycfg = false,
            "--no-loops" => opts.loops = false,
            s if s.starts_with("--passes=") => {
                let list: Vec<&str> = s["--passes=".len()..].split(',').filter(|p| !p.is_empty()).collect();
                if let Some(bad) = list.iter().find(|p| {
                    ![
                        "sroa",
                        "mem2reg",
                        "inline",
                        "recursion",
                        "combine",
                        "simplifycfg",
                        "loops",
                        "cse",
                        "memory",
                        "tailmark",
                    ]
                    .contains(p)
                }) {
                    return usage(&format!("unknown pass {bad}"));
                }
                opts.inline = list.contains(&"inline");
                opts.combine = list.contains(&"combine");
                opts.cse = list.contains(&"cse");
                opts.memory = list.contains(&"memory");
                opts.tailmark = list.contains(&"tailmark");
                opts.mem2reg = list.contains(&"mem2reg");
                opts.sroa = list.contains(&"sroa");
                opts.simplifycfg = list.contains(&"simplifycfg");
                opts.loops = list.contains(&"loops");
                if !list.contains(&"recursion") {
                    opts.recursion = RecursionMode::Off;
                }
            }
            "--remarks" => remarks = true,
            "--print-cycles" => cycles = true,
            "--stats" => stats = true,
            "--verify" => verify = true,
            "--print-memssa" => memssa = true,
            s if s.starts_with("--target=") => target = Some(s["--target=".len()..].to_string()),
            "--chop-counts" => chop_counts = true,
            s if s.starts_with("--chop=") => chop = Some(s["--chop=".len()..].to_string()),
            s if s.starts_with("--rules=") => {
                let path = &s["--rules=".len()..];
                match std::fs::read_to_string(path) {
                    Ok(text) => {
                        if let Err(e) = epitome_opt::combine::with_extra(&text) {
                            return usage(&format!("{path}: {e}"));
                        }
                        opts.extra_rules = Some(text);
                    }
                    Err(e) => return usage(&format!("{path}: {e}")),
                }
            }
            "-h" | "--help" => {
                println!("{USAGE}");
                return ExitCode::SUCCESS;
            }
            s if s.starts_with("-O") => match OptLevel::parse(&s[2..]) {
                Some(l) => opts.level = l,
                None => return usage(&format!("unknown level {s}")),
            },
            s if s.starts_with("--recursion=") => {
                opts.recursion = match &s["--recursion=".len()..] {
                    "off" => RecursionMode::Off,
                    "tail" => RecursionMode::Tail,
                    "stack" => RecursionMode::Stack,
                    other => return usage(&format!("unknown recursion mode {other}")),
                }
            }
            s if s.starts_with('-') && s != "-" => return usage(&format!("unknown option {s}")),
            s => input = Some(s.to_string()),
        }
    }
    let Some(path) = input else { return usage("no input file") };
    opts.chop = match epitome_ir::chop::limit_from(chop.as_deref(), "EPITOME_OPT_CHOP") {
        Ok(l) => l,
        Err(e) => return usage(&e),
    };

    let src = match epitome_ir::io::read_to_string(&path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("{path}: {e}");
            return ExitCode::FAILURE;
        }
    };
    let t0 = Instant::now();
    let mut m = match epitome_ir::parse_module(&src) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("{path}:{e}");
            return ExitCode::FAILURE;
        }
    };
    let t1 = Instant::now();
    let report = pipeline::run(&mut m, &opts);
    let t2 = Instant::now();
    if let Some(t) = &target {
        match read_machine(t) {
            Ok(mc) => {
                for g in m.global_order.clone() {
                    let gl = &m.globals[g.index()];
                    // (An opaque function is for another machine.)
                    if let GlobalKind::Function(f) = &gl.kind
                        && f.has_body
                        && !epitome_ir::fits::opaque(&m, &f.fn_attrs)
                    {
                        let name = gl.name.clone().unwrap_or_default();
                        // As epitome-llc will see it: without i64, narrowed.
                        let f = (**f).clone();
                        let f = if mc.lacks("i64") { epitome_ir::narrow::narrow(&mut m, &f, mc.widest) } else { f };
                        for w in epitome_ir::fits::check(&mc, &m, &f, &name) {
                            eprintln!("warning: {w}");
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("epitome-opt: {t}: {e}");
                return ExitCode::FAILURE;
            }
        }
    }
    let text = if memssa {
        let threads = if opts.threads == 0 { epitome_opt::par::default_threads() } else { opts.threads };
        let reports = epitome_opt::memssa::analyse_module(&mut m, threads, true);
        if stats {
            let (mut d, mut u, mut p, mut q, mut h, mut st, mut g) = (0, 0, 0, 0, 0, 0, 0);
            let (mut build, mut walk) = (std::time::Duration::ZERO, std::time::Duration::ZERO);
            for (_, r) in &reports {
                (d, u, p) = (d + r.defs, u + r.uses, p + r.phis);
                (q, h, st, g) = (q + r.walks.queries, h + r.walks.cache_hits, st + r.walks.steps, g + r.walks.gave_up);
                build += r.build;
                walk += r.walk;
            }
            eprintln!(
                "; memssa: {d} defs, {u} uses, {p} phis; {q} clobber queries ({h} cached), {st} alias checks, {g} gave up; build {:.1} ms, walks {:.1} ms",
                ms(build),
                ms(walk)
            );
        }
        let by: std::collections::HashMap<_, _> = reports.into_iter().collect();
        epitome_ir::printer::print_module_with(&m, &|g, at| by.get(&g).and_then(|r| r.notes.get(&at).cloned()))
    } else {
        m.to_string()
    };
    let t3 = Instant::now();

    // Autochop's reports: where the limit stopped, each function's count,
    // and (for a search) the counts and the output in files.
    if let Some((k, steps, last)) = &report.chop_stopped {
        let name = report.chop_names.get(*k).map_or("?", String::as_str);
        match last {
            Some(d) => eprintln!("; chop: stopped in @{name} (function {k}) after step {steps}: {d}"),
            None => eprintln!("; chop: stopped in @{name} (function {k}) after {steps} steps"),
        }
        if let Some(next) = &report.chop_next {
            eprintln!("; chop: the next step, refused: {next}");
        }
    }
    if chop_counts {
        for (k, c) in report.chop_counts.iter().enumerate() {
            eprintln!("; chop: function {k} @{}: {} elective changes", report.chop_names[k], c.steps);
        }
    }
    {
        let counts: Vec<u64> = report.chop_counts.iter().map(|c| c.steps).collect();
        if let Err(e) = epitome_ir::chop::report_counts("opt", &report.chop_names, &counts) {
            eprintln!("EPITOME_CHOP_COUNTS: {e}");
        }
        if let Ok(dump) = std::env::var("EPITOME_CHOP_DUMP")
            && let Err(e) = std::fs::write(&dump, &text)
        {
            eprintln!("{dump}: {e}");
        }
    }
    if cycles {
        if report.cycles.is_empty() {
            eprintln!("; no call-graph cycles");
        }
        for c in &report.cycles {
            eprintln!("; cycle: {c}");
        }
    }
    if remarks {
        for (f, list) in &report.remarks {
            for r in list {
                eprintln!("; @{f}: {r}");
            }
        }
        for f in &report.deleted {
            eprintln!("; @{f}: deleted, no longer used");
        }
    }
    if stats {
        eprintln!(
            "; {} functions in {} SCCs ({} cycles) on {} threads: {} calls inlined, {} tail, {} accumulator and {} other recursive calls removed, {} functions deleted",
            report.functions,
            report.sccs,
            report.cycles.len(),
            report.threads,
            report.inlined,
            report.tail_calls,
            report.accumulator_calls,
            report.frame_calls,
            report.deleted.len()
        );
        let c = report.simplifycfg;
        if c.total() > 0 {
            eprintln!(
                "; simplifycfg: {} unreachable blocks removed, {} empty blocks bypassed, {} selects made, {} blocks merged",
                c.unreachable, c.bypassed, c.selects, c.merged
            );
        }
        if report.rotated > 0 || report.indvars.changed() {
            let iv = &report.indvars;
            eprintln!(
                "; loops: {} rotated, {} trip counts known, {} exact exit tests, {} exit values, {} counters widened",
                report.rotated,
                iv.trip_counts.len(),
                iv.exact_exits,
                iv.exit_values,
                iv.widened
            );
        }
        if report.sroa.split > 0 {
            eprintln!("; sroa: {} allocas split into {} pieces", report.sroa.split, report.sroa.pieces);
        }
        if report.mem2reg.promoted > 0 {
            eprintln!(
                "; mem2reg: {} allocas promoted, {} loads and {} stores removed",
                report.mem2reg.promoted, report.mem2reg.loads, report.mem2reg.stores
            );
        }
        if report.cse.total() > 0 {
            eprintln!(
                "; cse: {} instructions merged, {} loads reused, {} stores forwarded",
                report.cse.merged, report.cse.loads, report.cse.forwarded
            );
        }
        if report.licm.hoisted > 0 {
            eprintln!("; licm: {} instructions moved out of loops ({} loads)", report.licm.hoisted, report.licm.loads);
        }
        if report.tail_marked > 0 {
            eprintln!("; tailmark: {} calls marked tail", report.tail_marked);
        }
        if report.memcpyopt.memsets > 0 {
            eprintln!("; memcpyopt: {} stores made {} memsets", report.memcpyopt.stores, report.memcpyopt.memsets);
        }
        if report.combines.total() > 0 {
            let mut top: Vec<(&String, &usize)> = report.combines.fired.iter().collect();
            top.sort_by(|a, b| b.1.cmp(a.1).then(a.0.cmp(b.0)));
            let list: Vec<String> = top.iter().take(12).map(|(k, v)| format!("{k} {v}")).collect();
            eprintln!("; combines: {} in all; most used: {}", report.combines.total(), list.join(", "));
        }
        eprintln!(
            "; critical path {} of {} instructions: at most {:.1}x faster with threads",
            report.critical_path,
            report.total_work,
            report.total_work as f64 / report.critical_path.max(1) as f64
        );
        eprintln!(
            "; measured: tasks took {:.1} ms in all, {:.1} ms along the longest chain: at most {:.1}x faster",
            ms(report.task_time),
            ms(report.task_critical_path),
            report.task_time.as_secs_f64() / report.task_critical_path.as_secs_f64().max(1e-9)
        );
        let steps: Vec<String> = report.timings.iter().map(|(n, d)| format!("{n} {:.1} ms", ms(*d))).collect();
        eprintln!("; passes: {}", steps.join(", "));
        eprintln!("; parse {:.1} ms, passes {:.1} ms, print {:.1} ms", ms(t1 - t0), ms(t2 - t1), ms(t3 - t2));
    }
    if verify {
        let mut bad = false;
        for (g, f) in m.functions() {
            let name = m.global(g).name.clone().unwrap_or_default();
            for e in epitome_opt::cfg::verify(f, &name) {
                eprintln!("verify: {e}");
                bad = true;
            }
        }
        if bad {
            return ExitCode::FAILURE;
        }
    }
    match output {
        Some(o) => {
            if let Err(e) = epitome_ir::io::write(&o, text) {
                eprintln!("{o}: {e}");
                return ExitCode::FAILURE;
            }
        }
        None => print!("{text}"),
    }
    ExitCode::SUCCESS
}

fn ms(d: std::time::Duration) -> f64 {
    d.as_secs_f64() * 1000.0
}

fn usage(msg: &str) -> ExitCode {
    eprintln!("epitome-opt: {msg}\n\n{USAGE}");
    ExitCode::FAILURE
}
