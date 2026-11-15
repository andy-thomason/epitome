//! CVA6, the OpenHW Group's 64-bit RISC-V core, built with epitome-sv used
//! as a library, and optionally with Verilator to compare.
//!
//! ```text
//! cargo run --release -p epitome-sv --example cva6 -- [options] [unit...]
//! ```
//!
//! 1. **Fetch.** CVA6 at the commit the tests use ([`units::COMMIT`]) is
//!    fetched with `git` into `target/cva6-example/cva6` and reused after
//!    (`--cva6 DIR` or `CVA6_DIR` for a checkout of your own). The commit
//!    is pinned because the wrappers repeat types of `cva6.sv` and the
//!    stimulus knows the layout of its structs and the numbers of its
//!    enums, which later commits change. None of CVA6 is part of Epitome.
//! 2. **Build.** Each unit of the core that the netlist compiles today, in
//!    the cv64a6_imafdc_sv39 configuration, under a wrapper of Epitome's
//!    own (`sv/tests/cva6/`), is preprocessed, parsed, elaborated, lowered
//!    to Epitome IR with the IR builder, optimised by epitome-opt,
//!    compiled by epitome-codegen and loaded into this process
//!    (`epitome_sv::model`), all in memory: no IR or assembly text, no
//!    files, no linker.
//! 3. **Run.** Each unit runs on random stimulus (and, where the tests have
//!    one, a typical mix) for one simulation, and with `--sims N` for N at
//!    once as lanes of the same code; the time is per instance and cycle.
//! 4. **The whole core.** `cva6` itself is elaborated, and each of its
//!    specialisations tried as a netlist of its own: the netlist does not
//!    compile all of the core yet, and this says what it lacks. Its files
//!    are those CVA6's own list (`core/Flist.cva6`) names, read when this
//!    runs. It needs CVA6's FPU (`core/cvfpu`, a submodule, and its own
//!    submodule `src/fpu_div_sqrt_mvp`): where the checkout does not have
//!    them, they are fetched at the commits it names into the cache.
//! 5. **Verilator** (`--verilator`; `$VERILATOR`, `$VERILATOR_ROOT/bin` or
//!    the path): each unit is also built as a Verilator model, run as a
//!    program on the same stimulus, its outputs checked word for word
//!    against epitome-sv's, and its time, code, state and build time put
//!    beside them.
//!
//! Options:
//!
//! - `--cva6 DIR`: a CVA6 checkout to use (as `CVA6_DIR`).
//! - `--cache DIR`: where to fetch to and build Verilator's models
//!   (default `target/cva6-example`).
//! - `--sims N`: also run N simulations at once.
//! - `--cycles N`: cycles for each unit (by default 4096 for a
//!   combinational unit's single simulation, and the tests' for others).
//! - `--verilator`: compare with Verilator.
//! - `--mattr LIST`: x86-64 features to compile for (by default those of
//!   this processor: AVX-512 if it has it, else AVX2).
//! - `--no-core`: leave out the whole core.
//! - `unit...`: only these units (`pmp_entry pmp alu exec cdec decoder
//!   id_stage issue_stage mult serdiv`).
//!
//! On x86-64, a unit whose code epitome-codegen cannot yet select for
//! AVX-512 is compiled for AVX2 instead, and the output says so.
//! Elaboration errors are shown by their first line only (the position and
//! message), not the source line under it. `docs/design/sv.md` section 8.6
//! has the numbers on one machine.

#[path = "../tests/cva6/units.rs"]
mod units;

use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};
use std::time::{Duration, Instant};

use epitome_sv::model::{self, Host, Io, Model, Sources, describe_shape};
use epitome_sv::netlist::{self, Netlist};
use epitome_sv::stimulus::Rng;
use epitome_sv::verilator::{Verilated, Verilator};
use units::Unit;

const CVA6_URL: &str = "https://github.com/openhwgroup/cva6";
/// The configuration, as CVA6's file list names it.
const TARGET_CFG: &str = "cv64a6_imafdc_sv39";

struct Args {
    cva6: Option<PathBuf>,
    cache: PathBuf,
    sims: Option<usize>,
    cycles: Option<usize>,
    verilator: bool,
    mattr: Option<String>,
    core: bool,
    units: Vec<String>,
}

fn args() -> Result<Args, String> {
    let target = std::env::var_os("CARGO_TARGET_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap().join("target"));
    let mut a = Args {
        cva6: std::env::var_os("CVA6_DIR").map(PathBuf::from),
        cache: target.join("cva6-example"),
        sims: None,
        cycles: None,
        verilator: false,
        mattr: None,
        core: true,
        units: Vec::new(),
    };
    let mut it = std::env::args().skip(1);
    let num = |v: Option<String>, what: &str| v.and_then(|v| v.parse::<usize>().ok()).filter(|&n| n > 0).ok_or(format!("{what} wants a number"));
    while let Some(arg) = it.next() {
        match arg.as_str() {
            "--cva6" => a.cva6 = Some(it.next().ok_or("--cva6 wants a directory")?.into()),
            "--cache" => a.cache = it.next().ok_or("--cache wants a directory")?.into(),
            "--sims" => a.sims = Some(num(it.next(), "--sims")?),
            "--cycles" => a.cycles = Some(num(it.next(), "--cycles")?),
            "--verilator" => a.verilator = true,
            "--mattr" => a.mattr = Some(it.next().ok_or("--mattr wants a list")?),
            "--no-core" => a.core = false,
            "-h" | "--help" => return Err(String::new()),
            u if !u.starts_with('-') && units::UNITS.iter().any(|x| x.name == u) => a.units.push(u.to_string()),
            other => return Err(format!("unknown argument {other}")),
        }
    }
    Ok(a)
}

const USAGE: &str = "usage: cargo run --release -p epitome-sv --example cva6 -- [--cva6 DIR] [--cache DIR] [--sims N] [--cycles N] [--verilator] [--mattr LIST] [--no-core] [unit...]";

fn main() -> ExitCode {
    let a = match args() {
        Ok(a) => a,
        Err(e) => {
            if !e.is_empty() {
                eprintln!("{e}");
            }
            eprintln!("{USAGE}\nunits: {}", units::UNITS.iter().map(|u| u.name).collect::<Vec<_>>().join(" "));
            return ExitCode::from(2);
        }
    };
    match run(&a) {
        Ok(true) => ExitCode::SUCCESS,
        Ok(false) => ExitCode::FAILURE,
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::FAILURE
        }
    }
}

/// `git` in `dir` with `args`, its output if it succeeds.
fn git(dir: &Path, args: &[&str]) -> Result<String, String> {
    let out = Command::new("git").arg("-C").arg(dir).args(args).output().map_err(|e| format!("git: {e}"))?;
    if !out.status.success() {
        return Err(format!("git {} in {}: {}", args.join(" "), dir.display(), String::from_utf8_lossy(&out.stderr).trim()));
    }
    Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
}

/// `url` at `commit` in `dir`: fetched (that commit alone) unless it is
/// there already.
fn fetch(dir: &Path, url: &str, commit: &str) -> Result<(), String> {
    if git(dir, &["rev-parse", "HEAD"]).is_ok_and(|h| h == commit) {
        return Ok(());
    }
    std::fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    if !dir.join(".git").exists() {
        git(dir, &["init", "--quiet"])?;
        git(dir, &["remote", "add", "origin", url])?;
    }
    println!("fetching {url} at {} into {}", &commit[..12], dir.display());
    let t0 = Instant::now();
    git(dir, &["fetch", "--quiet", "--depth", "1", "origin", commit])?;
    git(dir, &["checkout", "--quiet", "--force", "FETCH_HEAD"])?;
    println!("  fetched in {:.1} s", t0.elapsed().as_secs_f64());
    Ok(())
}

/// CVA6's checkout: the one named, or one fetched into the cache.
fn cva6_checkout(a: &Args) -> Result<PathBuf, String> {
    if let Some(dir) = &a.cva6 {
        match git(dir, &["rev-parse", "HEAD"]) {
            Ok(h) if h == units::COMMIT => {}
            Ok(h) => println!("note: {} is at {}, not {}; the units are checked at the latter", dir.display(), &h[..h.len().min(12)], &units::COMMIT[..12]),
            Err(_) => println!("note: {} is not a git checkout; using it as it is", dir.display()),
        }
        return Ok(dir.clone());
    }
    let dir = a.cache.join("cva6");
    fetch(&dir, CVA6_URL, units::COMMIT)?;
    Ok(dir)
}

/// Submodule `path` of `repo` (at the commit `repo` names, from the URL
/// its `.gitmodules` gives): `repo`'s own if it has it checked out, else
/// fetched into `cache`.
fn submodule(repo: &Path, path: &str, cache: &Path) -> Result<PathBuf, String> {
    let own = repo.join(path);
    if std::fs::read_dir(&own).is_ok_and(|mut d| d.next().is_some()) {
        return Ok(own);
    }
    let commit = git(repo, &["rev-parse", &format!("HEAD:{path}")])?;
    // (Its name in .gitmodules need not be its path.)
    let paths = git(repo, &["config", "-f", ".gitmodules", "--get-regexp", r"^submodule\..*\.path$"])?;
    let key = paths
        .lines()
        .find_map(|l| l.split_once(' ').filter(|(_, p)| *p == path).map(|(k, _)| k.trim_end_matches(".path").to_string()))
        .ok_or_else(|| format!("{} names no submodule {path}", repo.join(".gitmodules").display()))?;
    let url = git(repo, &["config", "-f", ".gitmodules", &format!("{key}.url")])?;
    let dir = cache.join(Path::new(path).file_name().unwrap());
    fetch(&dir, &url, &commit)?;
    Ok(dir)
}

fn ms(d: Duration) -> String {
    format!("{:.1} ms", d.as_secs_f64() * 1e3)
}

/// An error's first line: where and what, without the source under it.
fn first_line(e: &str) -> &str {
    e.lines().next().unwrap_or(e)
}

/// What was measured of a unit for one number of simulations and one kind
/// of stimulus.
struct Row {
    unit: &'static str,
    sims: usize,
    ns: f64,
    vl_ns: Option<f64>,
    agree: Option<bool>,
    /// Compiled for AVX2, as AVX-512 failed.
    avx2: bool,
}

/// What was built of a unit.
struct Built {
    unit: &'static str,
    build: Duration,
    code: u64,
    state: f64,
    vl: Option<(Option<Duration>, Option<u64>, usize)>,
}

fn run(a: &Args) -> Result<bool, String> {
    let host = match &a.mattr {
        Some(m) => Host::with_mattr(m),
        None => Host::native(),
    }
    .ok_or("epitome-sv runs code in process on x86-64 Linux and AArch64 macOS only")?;
    println!("epitome-sv's CVA6 example on {}", host.describe());
    let cva6 = cva6_checkout(a)?;
    println!("CVA6 {} at {}", &units::COMMIT[..12], cva6.display());

    let verilator = if a.verilator { find_verilator() } else { None };
    let list: Vec<&'static Unit> =
        units::UNITS.iter().filter(|u| a.units.is_empty() || a.units.iter().any(|n| n == u.name)).collect();
    let mut rows = Vec::new();
    let mut built = Vec::new();
    let mut ok = true;
    for u in list {
        match unit(a, &host, &cva6, u, verilator.as_ref(), &mut rows) {
            Ok(b) => built.push(b),
            Err(e) => {
                println!("  {}: {}", u.name, first_line(&e));
                ok = false;
            }
        }
    }
    summary(&rows, &built, verilator.is_some());
    if a.core {
        if let Err(e) = core(a, &cva6) {
            println!("the whole core: {}", first_line(&e));
        }
    }
    if rows.iter().any(|r| r.agree == Some(false)) {
        println!("\nepitome-sv and Verilator differ on some unit (see above)");
        ok = false;
    }
    Ok(ok)
}

/// Verilator, if it is here and new enough; says so if not.
fn find_verilator() -> Option<Verilator> {
    let Some(v) = Verilator::find() else {
        println!("Verilator is not here ($VERILATOR, $VERILATOR_ROOT/bin, or on the path): carrying on without it");
        return None;
    };
    println!("{} ({})", v.version, v.path.display());
    if !v.new_enough() {
        println!("  older than {}.{:03}, which the driver needs: carrying on without it", verilator_oldest().0, verilator_oldest().1);
        return None;
    }
    if v.number().is_some_and(|n| n < (5, 53)) {
        println!("  note: the published numbers are Verilator 5.053's; older ones, as Debian's 5.020, are much slower on CVA6 (UNOPTFLAT)");
    }
    Some(v)
}

fn verilator_oldest() -> (u32, u32) {
    epitome_sv::verilator::OLDEST
}

/// Elaborates `src` with `top` and lowers its netlist: with the time each
/// took.
fn lower(src: &Sources, top: &str, opts: &netlist::Options) -> Result<(Netlist, Duration, Duration), String> {
    let t0 = Instant::now();
    let (net, t1) = model::elaborate(src, &[top], |d| {
        let t1 = Instant::now();
        (netlist::compile_with(d, d.tops[0], opts), t1)
    })?;
    Ok((net?, t1 - t0, t1.elapsed()))
}

/// `u` lowered for `sims` simulations and built for `host`: with the
/// time elaboration, lowering, and the rest took.
fn build(src: &Sources, u: &Unit, host: &Host, sims: usize) -> Result<(Model, Duration, Duration, Duration), String> {
    let (net, elab, lowered) = lower(src, u.top, &host.options(sims))?;
    let started = Instant::now();
    let m = Model::build(net, host)?;
    Ok((m, elab, lowered, started.elapsed()))
}

fn unit(a: &Args, host: &Host, cva6: &Path, u: &'static Unit, vl: Option<&Verilator>, rows: &mut Vec<Row>) -> Result<Built, String> {
    println!("\n{}: {}", u.name, u.about);
    let src = u.sources(cva6);
    let mut sims_list = vec![1];
    sims_list.extend(a.sims.filter(|&s| s > 1));
    let mut first = None;
    let mut vmodel: Option<Verilated> = None;
    for sims in sims_list {
        let mut fallback = false;
        let (m, elab, lowered, built_in) = match build(&src, u, host, sims) {
            Ok(x) => x,
            // Where AVX-512's selection lacks an instruction the unit's code
            // wants, AVX2's, said plainly.
            Err(e) if host.mattr.iter().any(|m| m.starts_with("+avx512")) => {
                println!("  {sims} simulation{}: not compiled for AVX-512: {}", if sims == 1 { "" } else { "s" }, first_line(&e));
                let avx2 = Host::with_mattr("+avx2").unwrap();
                fallback = true;
                build(&src, u, &avx2, sims)?
            }
            Err(e) => return Err(e),
        };
        let build = elab + lowered + built_in;
        let t = m.times;
        println!(
            "  {sims} simulation{}{}: elaborated in {}, lowered in {}, optimised in {}, compiled in {}, loaded in {}",
            if sims == 1 { "" } else { "s" },
            if fallback { " (AVX2)" } else { "" },
            ms(elab),
            ms(lowered),
            ms(t.optimise),
            ms(t.compile),
            ms(t.load)
        );
        println!(
            "    {} bytes of code, {:.1} bytes of state an instance ({} lane{} a state)",
            m.code_bytes(),
            m.state_per_instance(),
            m.net.lanes,
            if m.net.lanes == 1 { "" } else { "s" }
        );
        let mut seen: Vec<String> = Vec::new();
        for f in &m.net.functions {
            let s = format!("{} {} lane{}: {}", f.spec, f.lanes, if f.lanes == 1 { "" } else { "s" }, describe_shape(f.shape));
            if !seen.contains(&s) {
                seen.push(s);
            }
        }
        println!("    lanes: {}", seen.join("; "));
        let io = Io::new(&m.net, u.clock)?;
        if let (Some(v), None) = (vl, &vmodel) {
            match v.build(&a.cache.join("vl").join(u.name), &src, u.top, &io) {
                Ok(b) => {
                    match b.build_time {
                        Some(t) if b.built => println!("  Verilator's model built in {:.1} s", t.as_secs_f64()),
                        Some(t) => println!("  Verilator's model as built before (in {:.1} s)", t.as_secs_f64()),
                        None => println!("  Verilator's model as built before"),
                    }
                    vmodel = Some(b);
                }
                Err(e) => println!("  Verilator: {}", first_line(&e)),
            }
        }
        // As many cycles as make 4096 instance-cycles of a combinational
        // unit, the tests' for a clocked one.
        let cycles = a.cycles.unwrap_or(if u.clock.is_some() { u.cycles } else { (4096 / sims).max(1) });
        let mut rng = Rng(u.seed);
        let mut vl_state = None;
        for &mix in u.mixes() {
            let stim = u.stimulus(mix, &io, cycles, sims, &mut rng);
            // At least a fifth of a second of evaluations.
            let once = m.time(&io, cycles, sims, &stim, 1).ns * (cycles * sims) as f64;
            let reps = ((2e8 / once.max(1.0)) as usize).clamp(1, 100_000);
            let ns = m.time(&io, cycles, sims, &stim, reps).ns;
            let (mut vl_ns, mut agree) = (None, None);
            let mut tail = String::new();
            if let Some(v) = &vmodel {
                let ours = m.run(&io, cycles, sims, &stim);
                let theirs = v.run(cycles, sims, &stim)?;
                let (bad, first) = model::compare(&io, sims, &ours, &theirs, 3);
                agree = Some(bad == 0);
                for x in &first {
                    println!("    differs: cycle {}, instance {}, {} is {:x?}, Verilator {:x?}", x.cycle, x.instance, x.port, x.ours, x.theirs);
                }
                let b = v.bench(cycles, sims, &stim, reps.max(1))?;
                vl_ns = Some(b.ns);
                vl_state = Some(b.state_bytes);
                tail = format!(
                    "; Verilator {:.2} ns ({:.1}x); {}",
                    b.ns,
                    b.ns / ns,
                    if bad == 0 { "outputs agree word for word".to_string() } else { format!("{bad} instance-cycles differ") }
                );
            }
            println!("    {mix:?}: {cycles} cycles x {sims}: {ns:.2} ns an instance-cycle{tail}");
            rows.push(Row { unit: u.name, sims, ns, vl_ns, agree, avx2: fallback });
        }
        if first.is_none() {
            let vl = vmodel.as_ref().map(|v| (v.build_time, v.code_bytes, vl_state.unwrap_or(0)));
            first = Some(Built { unit: u.name, build, code: m.code_bytes(), state: m.state_per_instance(), vl });
        }
    }
    Ok(first.unwrap())
}

fn summary(rows: &[Row], built: &[Built], vl: bool) {
    if rows.is_empty() {
        return;
    }
    println!("\nns an instance-cycle (random / typical stimulus)");
    let sims: Vec<usize> = {
        let mut s: Vec<usize> = rows.iter().map(|r| r.sims).collect();
        s.dedup();
        s.sort();
        s.dedup();
        s
    };
    let mut head = format!("{:<12}", "unit");
    for s in &sims {
        head += &format!(" {:>16}", format!("epitome x{s}"));
        if vl {
            head += &format!(" {:>16}", format!("Verilator x{s}"));
        }
    }
    println!("{head}");
    for b in built {
        let mut line = format!("{:<12}", b.unit);
        for &s in &sims {
            let here: Vec<&Row> = rows.iter().filter(|r| r.unit == b.unit && r.sims == s).collect();
            let fmt = |xs: Vec<f64>| xs.iter().map(|x| format!("{x:.1}")).collect::<Vec<_>>().join(" / ");
            let mark = if here.iter().any(|r| r.avx2) { "*" } else { "" };
            line += &format!(" {:>16}", fmt(here.iter().map(|r| r.ns).collect()) + mark);
            if vl {
                line += &format!(" {:>16}", fmt(here.iter().filter_map(|r| r.vl_ns).collect()));
            }
        }
        println!("{line}");
    }
    if rows.iter().any(|r| r.avx2) {
        println!("* compiled for AVX2: epitome-codegen's AVX-512 selection lacks an instruction this code wants");
    }
    println!("\nbuilt (one simulation)");
    println!(
        "{:<12} {:>12} {:>12} {:>12}{}",
        "unit",
        "build",
        "code bytes",
        "state/inst",
        if vl { format!(" {:>12} {:>12} {:>12}", "VL build", "VL code", "VL state") } else { String::new() }
    );
    for b in built {
        let mut line = format!("{:<12} {:>12} {:>12} {:>12.1}", b.unit, ms(b.build), b.code, b.state);
        if let Some((t, code, state)) = &b.vl {
            line += &format!(
                " {:>12} {:>12} {:>12}",
                t.map_or("?".into(), |t| format!("{:.1} s", t.as_secs_f64())),
                code.map_or("?".into(), |c| c.to_string()),
                state
            );
        }
        println!("{line}");
    }
}

/// CVA6's files and include directories for the configuration, from the
/// file list it keeps (`core/Flist.cva6`), read as it is when this runs:
/// the FPU's files from `cvfpu` and its divider's from `mvp`, and the
/// HPDcache's left out (with its submodule), as this configuration has
/// the write-through cache.
fn core_sources(cva6: &Path, cvfpu: &Path, mvp: &Path) -> Result<Sources, String> {
    let list = cva6.join("core/Flist.cva6");
    let text = std::fs::read_to_string(&list).map_err(|e| format!("{}: {e}", list.display()))?;
    let root = cva6.display().to_string();
    let mut src = Sources::default();
    for line in text.lines().map(str::trim) {
        if line.is_empty() || line.starts_with("//") || line.to_ascii_lowercase().contains("hpdcache") {
            continue;
        }
        let line = line
            .replace("${CVA6_REPO_DIR}/core/cvfpu/src/fpu_div_sqrt_mvp", &mvp.display().to_string())
            .replace("${CVA6_REPO_DIR}/core/cvfpu", &cvfpu.display().to_string())
            .replace("${CVA6_REPO_DIR}", &root)
            .replace("${TARGET_CFG}", TARGET_CFG);
        match line.strip_prefix("+incdir+") {
            Some(dir) => {
                src.include_dir(dir);
            }
            None => {
                src.file(line);
            }
        }
    }
    Ok(src)
}

/// The whole core: elaborated, and each specialisation tried as a netlist
/// of its own.
fn core(a: &Args, cva6: &Path) -> Result<(), String> {
    println!("\nthe whole core (cva6)");
    let cvfpu = submodule(cva6, "core/cvfpu", &a.cache)?;
    let mvp = submodule(&cvfpu, "src/fpu_div_sqrt_mvp", &a.cache)?;
    let src = core_sources(cva6, &cvfpu, &mvp)?;
    println!("  {} files, {} lines", src.items.len(), src.lines());
    let t0 = Instant::now();
    let (specs, instances, gaps, elab, cov) = model::elaborate(&src, &["cva6"], |d| {
        let elab = t0.elapsed();
        let cov = model::coverage(d);
        (d.specs.len(), d.instances.len(), d.gaps(), elab, cov)
    })?;
    println!("  elaborated in {}: {specs} specialisations, {instances} instances", ms(elab));
    println!("  {} expressions of unknown type, {} items kept as syntax (mostly concurrent assertions)", gaps.0, gaps.1);
    let lowered: Vec<&model::Lowering> = cov.iter().filter(|c| c.result.is_ok()).collect();
    println!(
        "  the netlist does not compile all of the core yet: {} of {} specialisations lower, with all below them, in {}",
        lowered.len(),
        cov.len(),
        ms(t0.elapsed() - elab)
    );
    // The rest, by the first thing that stops each (in it or below it),
    // a module's name counted once for all its specialisations.
    let mut why: Vec<(String, Vec<(&str, usize)>)> = Vec::new();
    for c in cov.iter().filter(|c| c.result.is_err()) {
        let e = first_line(c.result.as_ref().unwrap_err());
        // (Without the module and what follows: "a combinational loop in
        // m: ..." is one kind of gap for every m.)
        let e = e.split(": ").next().unwrap_or(e);
        let e = e.split(" in ").next().unwrap_or(e).to_string();
        let i = why.iter().position(|w| w.0 == e).unwrap_or_else(|| {
            why.push((e, Vec::new()));
            why.len() - 1
        });
        match why[i].1.iter_mut().find(|m| m.0 == c.module) {
            Some(m) => m.1 += 1,
            None => why[i].1.push((&c.module, 1)),
        }
    }
    why.sort_by_key(|w| std::cmp::Reverse(w.1.iter().map(|m| m.1).sum::<usize>()));
    for (e, mods) in &why {
        let n: usize = mods.iter().map(|m| m.1).sum();
        let names: Vec<String> = mods.iter().take(6).map(|(m, k)| if *k > 1 { format!("{m} (x{k})") } else { m.to_string() }).collect();
        let more = if mods.len() > 6 { format!(" and {} more", mods.len() - 6) } else { String::new() };
        println!("    {n:3} {e}: {}{more}", names.join(", "));
    }
    Ok(())
}
