//! `epitome-rustc`: Epitome as the code generator for Cargo builds.
//!
//! ```sh
//! RUSTC_WRAPPER=/path/to/epitome-rustc cargo test
//! ```
//!
//! Cargo runs a `RUSTC_WRAPPER` with rustc's command line. For each crate
//! it builds, this runs rustc as asked, also writing the crate's LLVM IR
//! (`--emit=llvm-ir`, one codegen unit), and compiles that IR itself:
//!
//! - **A library** (`rlib`): after rustc, our object replaces rustc's in
//!   the archive, under the same member name.
//! - **A binary or test harness**: rustc is given this program as its
//!   linker (`-C linker`), which compiles the IR, already written by then,
//!   and links with our object in place of rustc's for the crate, then
//!   runs the real linker (`cc`, or `EPITOME_RUSTC_LINKER`).
//!
//! What stays rustc's (LLVM's): the standard library, which comes built;
//! the allocator shim; proc-macro crates and build scripts, which run
//! inside the build (`EPITOME_RUSTC_BUILD_SCRIPTS=1` takes build scripts
//! too); crates built with link-time optimisation, and libraries built
//! with bitcode for it (without `-C embed-bitcode=no`), as it is LLVM's
//! of the bitcode our objects do not have; and any crate whose IR we
//! cannot compile, which keeps rustc's object. Every crate's outcome is a line in `EPITOME_RUSTC_LOG`
//! (default `epitome-rustc.log` beside the target directory's `deps`).
//!
//! `EPITOME_RUSTC_OPT=1` asks rustc for its IR before LLVM's passes
//! (`-C no-prepopulate-passes`) and runs Epitome's own pipeline on it;
//! otherwise the IR is LLVM's, optimised as the profile says, and only
//! code generation is ours. `EPITOME_RUSTC_SKIP=a,b` leaves those crates
//! alone; `EPITOME_RUSTC_ONLY=a,b` takes only those. `EPITOME_LLC_CHOP`
//! limits elective changes as for epitome-llc (`epitome_ir::chop`), to
//! search for the change that miscompiles a crate.
//!
//! Cargo already builds several crates at once, so each compilation here
//! uses `EPITOME_RUSTC_THREADS` threads (default 2), not one per core:
//! with one per core, a dozen crates at once each running a dozen threads,
//! each thread with its own copy of the module, can run a machine out of
//! memory.
//!
//! Each crate's line in the log ends with numbers for comparing: rustc's
//! time for the crate (`rustc_ms`, its whole compilation, LLVM included),
//! ours (`ours_ms`, reading the IR and compiling it), and the size of the
//! machine code (`__text`) in rustc's object and in ours.

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};
use std::time::Instant;

use epitome_codegen::target::Target;
use epitome_codegen::{Options, compile};

/// Set in rustc's environment for the link: the crate's IR, and the start
/// of the names of rustc's objects for the crate.
const LL_VAR: &str = "EPITOME_RUSTC_LL";
const PREFIX_VAR: &str = "EPITOME_RUSTC_PREFIX";
const LOG_VAR: &str = "EPITOME_RUSTC_LOG_FILE";

/// A runaway compilation stops at a memory limit (`epitome_ir::memlimit`;
/// `EPITOME_MEMORY_LIMIT_MB`, default 4096) rather than taking the machine
/// down.
#[global_allocator]
static MEMORY: epitome_ir::memlimit::Limited = epitome_ir::memlimit::Limited;

fn main() -> ExitCode {
    epitome_ir::memlimit::set_from_env(4096);
    let args: Vec<String> = std::env::args().collect();
    if let Ok(ll) = std::env::var(LL_VAR) {
        return link(&ll, &args[1..]);
    }
    if args.len() < 2 {
        eprintln!("usage: RUSTC_WRAPPER=epitome-rustc cargo build (or test)");
        return ExitCode::FAILURE;
    }
    wrap(&args[1], &args[2..])
}

/// The value of an option given as `--name value` or `--name=value`.
fn option<'a>(args: &'a [String], name: &str) -> Option<&'a str> {
    let eq = format!("{name}=");
    args.iter().enumerate().find_map(
        |(k, a)| {
            if a == name { args.get(k + 1).map(String::as_str) } else { a.strip_prefix(&eq) }
        },
    )
}

/// The value of a codegen option, `-C key=value` or `-Ckey=value`.
fn codegen_option<'a>(args: &'a [String], key: &str) -> Option<&'a str> {
    let eq = format!("{key}=");
    args.iter().enumerate().find_map(|(k, a)| {
        let v = if a == "-C" { args.get(k + 1).map(String::as_str)? } else { a.strip_prefix("-C")? };
        v.strip_prefix(&eq)
    })
}

/// `args` without the codegen options named in `keys`.
fn without_codegen(args: &[String], keys: &[&str]) -> Vec<String> {
    let named = |v: &str| keys.iter().any(|k| v == *k || v.starts_with(&format!("{k}=")));
    let mut out = Vec::new();
    let mut k = 0;
    while k < args.len() {
        let a = &args[k];
        if a == "-C" && args.get(k + 1).is_some_and(|v| named(v)) {
            k += 2;
            continue;
        }
        if a.strip_prefix("-C").is_some_and(|v| !v.is_empty() && named(v)) {
            k += 1;
            continue;
        }
        out.push(a.clone());
        k += 1;
    }
    out
}

fn list(var: &str) -> Option<Vec<String>> {
    std::env::var(var).ok().map(|v| v.split(',').map(str::to_string).collect())
}

fn wrap(rustc: &str, args: &[String]) -> ExitCode {
    let pass = |reason: &str, log: Option<&Path>, krate: &str| {
        if let Some(log) = log {
            note(log, &format!("{krate}: rustc's ({reason})"));
        }
        status(Command::new(rustc).args(args))
    };
    let Some(krate) = option(args, "--crate-name").map(str::to_string) else {
        return pass("", None, "");
    };
    if krate == "___" {
        return pass("", None, "");
    }
    let Some(out_dir) = option(args, "--out-dir").map(PathBuf::from) else {
        return pass("", None, &krate);
    };
    let log = std::env::var("EPITOME_RUSTC_LOG")
        .map(PathBuf::from)
        .unwrap_or_else(|_| out_dir.parent().unwrap_or(&out_dir).join("epitome-rustc.log"));
    let emit = option(args, "--emit").unwrap_or("link").to_string();
    if !emit.split(',').any(|e| e == "link") {
        return pass("no code asked for", Some(&log), &krate);
    }
    let types: Vec<&str> = args
        .iter()
        .enumerate()
        .filter(|(_, a)| *a == "--crate-type")
        .filter_map(|(k, _)| args.get(k + 1).map(String::as_str))
        .collect();
    if let Some(t) = types.iter().find(|t| matches!(**t, "proc-macro" | "dylib" | "cdylib" | "staticlib")) {
        return pass(&format!("a {t}"), Some(&log), &krate);
    }
    // Link-time optimisation is LLVM's, of the bitcode in each crate's
    // object, which ours do not have: a crate built with it, or a library
    // that a build with it may link (Cargo asks for no bitcode, with
    // `-C embed-bitcode=no`, only when none will), keeps rustc's.
    let lto = args.iter().enumerate().any(|(k, a)| {
        let v = if a == "-C" {
            args.get(k + 1).map(String::as_str).unwrap_or("")
        } else {
            a.strip_prefix("-C").unwrap_or("")
        };
        v == "lto" || v.strip_prefix("lto=").is_some_and(|x| !matches!(x, "off" | "no" | "n" | "false"))
    });
    if lto {
        return pass("link-time optimisation", Some(&log), &krate);
    }
    let library = !args.iter().any(|a| a == "--test") && !types.contains(&"bin");
    if library && codegen_option(args, "embed-bitcode") != Some("no") {
        return pass("bitcode for link-time optimisation", Some(&log), &krate);
    }
    if krate.starts_with("build_script_") && std::env::var("EPITOME_RUSTC_BUILD_SCRIPTS").is_err() {
        return pass("a build script", Some(&log), &krate);
    }
    if list("EPITOME_RUSTC_SKIP").is_some_and(|l| l.contains(&krate))
        || list("EPITOME_RUSTC_ONLY").is_some_and(|l| !l.contains(&krate))
    {
        return pass("skipped", Some(&log), &krate);
    }
    let extra = codegen_option(args, "extra-filename").unwrap_or("").to_string();
    let binary = args.iter().any(|a| a == "--test") || types.contains(&"bin");
    let ours_opt = std::env::var("EPITOME_RUSTC_OPT").is_ok_and(|v| v == "1");

    // rustc as asked, with the IR too, in one codegen unit, not incremental.
    let mut new = without_codegen(args, &["codegen-units", "incremental"]);
    for a in &mut new {
        if let Some(e) = a.strip_prefix("--emit=") {
            *a = format!("--emit={e},llvm-ir");
        }
    }
    if !args.iter().any(|a| a.starts_with("--emit")) {
        new.push("--emit=link,llvm-ir".into());
    }
    new.extend(["-C".into(), "codegen-units=1".into()]);
    if ours_opt {
        new.extend(["-C".into(), "no-prepopulate-passes".into()]);
    }
    let ll = out_dir.join(format!("{krate}{extra}.ll"));
    let prefix = format!("{krate}{extra}.");
    let mut cmd = Command::new(rustc);
    if binary {
        let me = std::env::current_exe().expect("this program's path");
        if let Some(l) = codegen_option(args, "linker") {
            cmd.env("EPITOME_RUSTC_LINKER", l);
        }
        new = without_codegen(&new, &["linker", "linker-flavor"]);
        new.extend(["-C".into(), format!("linker={}", me.display()), "-C".into(), "linker-flavor=gcc".into()]);
        cmd.env(LL_VAR, &ll).env(PREFIX_VAR, &prefix).env(LOG_VAR, &log);
    }
    let started = Instant::now();
    let code = status(cmd.args(&new));
    let rustc_ms = started.elapsed().as_millis();
    if binary {
        // (Its link, and so our compilation, is inside this time.)
        note(&log, &format!("{krate}: rustc time (binary) | rustc_ms={rustc_ms}"));
    }
    if code != ExitCode::SUCCESS || binary {
        return code;
    }
    // A library: our object in the archive, in place of rustc's.
    if types.is_empty() || types.iter().any(|t| matches!(*t, "lib" | "rlib")) {
        let rlib = out_dir.join(format!("lib{krate}{extra}.rlib"));
        match replace_in_rlib(&rlib, &ll, &prefix, ours_opt) {
            Ok(r) => note(
                &log,
                &format!(
                    "{krate}: ours (library) | rustc_ms={rustc_ms} ours_ms={} text_rustc={} text_ours={}",
                    r.ms, r.text_rustc, r.text_ours
                ),
            ),
            Err(e) => note(&log, &format!("{krate}: rustc's (ours failed: {e}) | rustc_ms={rustc_ms}")),
        }
    }
    code
}

/// Compiles the IR at `ll` to an object. A panic in the compiler is an
/// error like any other: the crate keeps rustc's object.
fn object_of(ll: &Path, ours_opt: bool) -> Result<Vec<u8>, String> {
    let quiet = std::env::var("EPITOME_RUSTC_PANICS").is_err();
    if quiet {
        std::panic::set_hook(Box::new(|_| {}));
    }
    std::panic::catch_unwind(|| object_of_inner(ll, ours_opt)).unwrap_or_else(|p| {
        let what = p
            .downcast_ref::<String>()
            .cloned()
            .or_else(|| p.downcast_ref::<&str>().map(|s| s.to_string()))
            .unwrap_or_default();
        Err(format!("the compiler panicked: {what}"))
    })
}

fn object_of_inner(ll: &Path, ours_opt: bool) -> Result<Vec<u8>, String> {
    let text = std::fs::read_to_string(ll).map_err(|e| format!("{}: {e}", ll.display()))?;
    let mut m = epitome_ir::parse_module(&text).map_err(|e| format!("parsing: {e:?}"))?;
    let threads = std::env::var("EPITOME_RUSTC_THREADS").ok().and_then(|v| v.parse().ok()).unwrap_or(2);
    if ours_opt {
        epitome_opt::pipeline::run(&mut m, &epitome_opt::pipeline::Options { threads, ..Default::default() });
    }
    let t = if cfg!(target_arch = "x86_64") { Target::x86_64() } else { Target::aarch64() };
    // A chop limit, as epitome-llc takes, to search a miscompiled crate
    // (with `EPITOME_RUSTC_ONLY` naming it).
    let chop = epitome_ir::chop::limit_from(None, "EPITOME_LLC_CHOP")?;
    let compiled = compile(&t, &mut m, &Options { threads, chop, ..Options::default() })?;
    if !compiled.errors.is_empty() {
        return Err(compiled.errors.join("; "));
    }
    // Each function's count of elective changes, for epitome-chop.
    let counts: Vec<u64> = compiled.chop_counts.iter().map(|c| c.steps).collect();
    epitome_ir::chop::report_counts("llc", &compiled.chop_names, &counts)
        .map_err(|e| format!("EPITOME_CHOP_COUNTS: {e}"))?;
    compiled.object_file(&t)
}

/// The size of an object's machine code: its `__text` sections (Mach-O),
/// or `.text` ones (ELF).
fn text_size(object: &[u8]) -> u64 {
    if object.starts_with(b"\x7fELF") {
        return epitome_codegen::elf::read(object)
            .map(|e| e.sections.iter().filter(|s| s.name.starts_with(".text")).map(|s| s.size).sum())
            .unwrap_or(0);
    }
    epitome_codegen::macho::read(object)
        .map(|m| m.sections.iter().filter(|s| s.name == "__text").map(|s| s.size).sum())
        .unwrap_or(0)
}

/// Our object for a crate, and how it compares with rustc's.
struct Replaced {
    ms: u128,
    text_rustc: u64,
    text_ours: u64,
}

fn replace_in_rlib(rlib: &Path, ll: &Path, prefix: &str, ours_opt: bool) -> Result<Replaced, String> {
    let started = Instant::now();
    let bytes = object_of(ll, ours_opt)?;
    let ms = started.elapsed().as_millis();
    let listing = Command::new("ar").arg("t").arg(rlib).output().map_err(|e| format!("ar: {e}"))?;
    let members = String::from_utf8_lossy(&listing.stdout).into_owned();
    let mine: Vec<&str> = members.lines().filter(|m| m.starts_with(prefix) && m.ends_with("-cgu.0.rcgu.o")).collect();
    let [member] = mine[..] else {
        return Err(format!("{} objects for the crate in the archive, not one", mine.len()));
    };
    let theirs = Command::new("ar").arg("p").arg(rlib).arg(member).output().map_err(|e| format!("ar: {e}"))?;
    let text_rustc = text_size(&theirs.stdout);
    let dir = ll.with_extension("epitome");
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    std::fs::write(dir.join(member), &bytes).map_err(|e| e.to_string())?;
    let ok = Command::new("ar")
        .current_dir(&dir)
        .arg("rs")
        .arg(std::path::absolute(rlib).map_err(|e| e.to_string())?)
        .arg(member)
        .status()
        .map_err(|e| format!("ar: {e}"))?;
    if !ok.success() {
        return Err("ar failed".into());
    }
    Ok(Replaced { ms, text_rustc, text_ours: text_size(&bytes) })
}

/// As the linker: our object for the crate in place of rustc's, then the
/// real linker.
fn link(ll: &str, args: &[String]) -> ExitCode {
    let prefix = std::env::var(PREFIX_VAR).unwrap_or_default();
    let log = std::env::var(LOG_VAR).map(PathBuf::from).ok();
    let krate = prefix.split('-').next().unwrap_or("").trim_end_matches('.').to_string();
    let ours_opt = std::env::var("EPITOME_RUSTC_OPT").is_ok_and(|v| v == "1");
    let is_mine = |a: &str| {
        let name = Path::new(a).file_name().and_then(|n| n.to_str()).unwrap_or("");
        name.starts_with(&prefix) && name.ends_with("-cgu.0.rcgu.o")
    };
    let mut args = args.to_vec();
    let started = Instant::now();
    let ours = object_of(Path::new(ll), ours_opt);
    let ms = started.elapsed().as_millis();
    let text_rustc: u64 =
        args.iter().filter(|a| is_mine(a)).map(|a| std::fs::read(a).map_or(0, |b| text_size(&b))).sum();
    match ours {
        Ok(bytes) => {
            let obj = Path::new(ll).with_extension("epitome.o");
            match std::fs::write(&obj, &bytes) {
                Ok(()) => {
                    let n = args.iter().filter(|a| is_mine(a)).count();
                    for a in &mut args {
                        if is_mine(a) {
                            *a = obj.display().to_string();
                        }
                    }
                    if let Some(log) = &log {
                        let what = if n == 1 { "ours (binary)".to_string() } else { format!("rustc's ({n} objects)") };
                        note(
                            log,
                            &format!(
                                "{krate}: {what} | ours_ms={ms} text_rustc={text_rustc} text_ours={}",
                                text_size(&bytes)
                            ),
                        );
                    }
                }
                Err(e) => {
                    if let Some(log) = &log {
                        note(log, &format!("{krate}: rustc's (writing our object: {e})"));
                    }
                }
            }
        }
        Err(e) => {
            if let Some(log) = &log {
                note(log, &format!("{krate}: rustc's (ours failed: {e})"));
            }
        }
    }
    let linker = std::env::var("EPITOME_RUSTC_LINKER").unwrap_or_else(|_| "cc".into());
    status(Command::new(linker).args(&args).env_remove(LL_VAR))
}

fn status(cmd: &mut Command) -> ExitCode {
    match cmd.status() {
        Ok(s) if s.success() => ExitCode::SUCCESS,
        Ok(s) => ExitCode::from(s.code().unwrap_or(1).clamp(1, 255) as u8),
        Err(e) => {
            eprintln!("epitome-rustc: {cmd:?}: {e}");
            ExitCode::FAILURE
        }
    }
}

fn note(log: &Path, line: &str) {
    let first: String = line.lines().next().unwrap_or("").chars().take(300).collect();
    // One write, so that lines from builds in parallel do not mix.
    if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(log) {
        let _ = f.write_all(format!("{first}\n").as_bytes());
    }
}
