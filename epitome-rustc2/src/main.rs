//! `epitome-rustc2 [options] root.rs...`: for now, loads each crate (its
//! root file and the files of its modules, all parsed in parallel) and
//! reports every error with its position.
//!
//! Options:
//!
//! - `--edition 2015|2018|2021|2024` (default 2024), also `--edition=N`.
//! - `--threads N`: worker threads (default one per core).
//! - `--cfg name` or `--cfg 'name="value"'`: set a configuration option,
//!   on top of the host's (`target_os`, `unix`, `debug_assertions`, ...).
//! - `--no-cfg`: do not strip `cfg`; load every module whose file exists.
//! - `--extern name=lib.so`: a proc-macro library (built by rustc) that the
//!   crates use, as `name`.
//! - `--proc-macro-server PATH`: the server that runs proc macros; by
//!   default the one beside the `rustc` on `PATH`.
//! - `--dump-ast`: print each file's syntax tree, one node per line.
//! - `--sexp`: print each file's syntax tree on one line.
//! - `--pretty`: print each file's syntax tree back as Rust source.
//! - `--tokens file.rs`: print the tokens of one file, one per line.
//! - `--list-files`: print the path of each file loaded, one per line.
//! - `--no-expand`: stop before macro expansion (it runs by default, so
//!   `--pretty` shows the expanded tree).
//! - `--expand-stats`: print what expansion did, and the macros most often
//!   not found.
//! - `--resolve-stats`: resolve names, and print how many paths resolved
//!   and the first segments of those that did not.
//! - `--dump-res`: resolve names, and print every path with what it
//!   resolved to.
//! - `--dump-sigs`: work out every item's signature, and print them.
//! - `--types-stats`: count signatures, and why any have error types.
//! - `--dump-types`: type check every body, and print each expression's
//!   and pattern's type.
//! - `--typeck-stats`: type check every body, and count what failed and why.
//! - `--dump-mono`: monomorphise from the last crate's `main` (or its
//!   non-generic functions), and print every item needed.
//! - `--mono-stats`: monomorphise, and count the items and what failed.
//! - `--dump-ir`: monomorphise, lower every function instance to Epitome
//!   IR, and print the module, with the source of each statement as a
//!   comment.
//! - `--lower-stats`: lower, and list each function that could not be
//!   lowered (its body traps) with why.
//! - `-o PATH`: monomorphise from the last crate's `main`, lower, optimise
//!   with Epitome's optimiser, compile for the host with Epitome's code
//!   generator, and link with `cc` (and libm) into the executable `PATH`. It fails if
//!   any function of the last crate could not be lowered.
//! - `-O0`: with `-o`, do not optimise the crates given (`-C opt-level=0`).
//! - `-C opt-level=0|1|2|3|s|z` (also `-Copt-level=..`): how much to
//!   optimise the crates given (default 2). The standard library's crates
//!   (those from rust-src) are optimised at 2 whatever this is, and built
//!   without `debug_assertions`, as rustc's prebuilt `std` is; a generic
//!   function is optimised as the crate it is written in is.
//! - `--crate-opt name=level`: crate `name`'s own optimisation level, in
//!   place of the above (`--crate-opt std=0`, `--crate-opt prog=3`).
//! - `-C panic=unwind|abort|immediate-abort` (also `-Cpanic=...`): the
//!   panic strategy `std` and `core` are built for (`cfg(panic = ..)`;
//!   default `unwind`). With `immediate-abort` a panic aborts at once,
//!   without formatting or printing its message, which leaves the
//!   message-formatting code out of the program.
//! - `--emit=obj`: with `-o`, write the object file to `PATH`, unlinked.
//! - Kernels: a function of the last crate marked
//!   `#[epitome::target("riscv64")]` is compiled for that machine too (see
//!   `device`), into the object `PATH.riscv64.o`, linked into an image the
//!   program carries. A call of a kernel launches it: `__epitome_launch`,
//!   which an RV64 interpreter compiled into the program gives
//!   (`runtime/rv64.rs`), or the program itself with `--launch-runtime
//!   none`. `--emit=kernels` writes only the objects.
//! - `--lib`: compile the last crate as a library: monomorphise from all
//!   its non-generic functions (even if it has a `main`), and make them
//!   external, so that optimising keeps them. With `-o`, use `--emit=obj`.
//! - `--std`: load `std` and the crates it is built from (from the
//!   toolchain's rust-src, as `sysroot::std_crates` lists them) before
//!   the crates given, with the feature and environment they expect.
//! - `--dump-layout`: the layout of every non-generic struct, enum, union
//!   and type alias of the last crate given.
//! - `--parse-tree dir`: parse every `.rs` file under `dir` (each on its
//!   own, not following modules), and report each that fails and a count.
//!   Each file is read in the edition of the nearest `Cargo.toml` above it
//!   (2015 if it names none, as for Cargo), or `--edition` if there is no
//!   `Cargo.toml`. With `--roundtrip`, each file is also printed and
//!   parsed again, and must give the same tree.

use epitome_rustc2::ast::Ast;
use epitome_rustc2::cfg::CfgSet;
use epitome_rustc2::hash::FastMap;
use epitome_rustc2::lexer::{Edition, lex};
use epitome_rustc2::parse::parse_str;
use epitome_rustc2::pretty::{first_difference, pretty};
use epitome_rustc2::session::{CrateRoot, Options, Session};
use epitome_rustc2::source::Source;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

fn main() -> ExitCode {
    let mut edition = Edition::default();
    let mut options = Options::default();
    let (mut dump, mut sexp, mut tokens, mut print, mut roundtrip) = (false, false, false, false, false);
    let mut list_files = false;
    let (mut no_expand, mut expand_stats) = (false, false);
    let (mut resolve_stats, mut dump_res) = (false, false);
    let (mut dump_sigs, mut types_stats) = (false, false);
    let (mut dump_types, mut typeck_stats) = (false, false);
    let mut dump_layout = false;
    let (mut dump_mono, mut mono_stats) = (false, false);
    let (mut dump_ir, mut lower_stats) = (false, false);
    let mut output: Option<PathBuf> = None;
    let (mut emit_obj, mut lib, mut emit_kernels) = (false, false, false);
    let mut launch_runtime = true;
    // The optimisation level of the crates given, and of crates by name.
    let mut opt_level = "2".to_string();
    let mut crate_opt: Vec<(String, String)> = Vec::new();
    let level_ok = |l: &str| matches!(l, "0" | "1" | "2" | "3" | "s" | "z");
    let mut tree: Option<PathBuf> = None;
    let mut cfg = CfgSet::host();
    let mut externs: Vec<(String, PathBuf)> = Vec::new();
    let mut no_cfg = false;
    let mut with_std = false;
    let mut files = Vec::new();
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        let mut value = |name: &str| arg.strip_prefix(&format!("{name}=")).map(str::to_string).or_else(|| args.next());
        match arg.as_str() {
            "--dump-ast" => dump = true,
            "--sexp" => sexp = true,
            "--tokens" => tokens = true,
            "--pretty" => print = true,
            "--roundtrip" => roundtrip = true,
            "--no-cfg" => no_cfg = true,
            "--list-files" => list_files = true,
            "--no-expand" => no_expand = true,
            "--expand-stats" => expand_stats = true,
            "--resolve-stats" => resolve_stats = true,
            "--dump-res" => dump_res = true,
            "--dump-sigs" => dump_sigs = true,
            "--types-stats" => types_stats = true,
            "--dump-types" => dump_types = true,
            "--typeck-stats" => typeck_stats = true,
            "--dump-layout" => dump_layout = true,
            "--dump-mono" => dump_mono = true,
            "--mono-stats" => mono_stats = true,
            "--dump-ir" => dump_ir = true,
            "-O0" => opt_level = "0".to_string(),
            _ if arg.starts_with("--crate-opt") => {
                let v = value("--crate-opt").unwrap_or_default();
                match v.split_once('=') {
                    Some((n, l)) if level_ok(l) => crate_opt.push((n.to_string(), l.to_string())),
                    _ => {
                        eprintln!("error: `--crate-opt` takes `name=level` (level 0, 1, 2, 3, s or z), not `{v}`");
                        return ExitCode::FAILURE;
                    }
                }
            }
            "--emit=obj" => emit_obj = true,
            "--emit=kernels" => emit_kernels = true,
            _ if arg.starts_with("--launch-runtime") => match value("--launch-runtime").as_deref() {
                Some("rv64") => launch_runtime = true,
                Some("none") => launch_runtime = false,
                v => {
                    eprintln!("error: `--launch-runtime` is `rv64` or `none`, not {v:?}");
                    return ExitCode::FAILURE;
                }
            },
            "--lib" => lib = true,
            "--std" => with_std = true,
            "-o" => output = value("-o").map(PathBuf::from),
            "--lower-stats" => lower_stats = true,
            _ if arg.starts_with("-C") => {
                let v = if arg == "-C" { value("-C").unwrap_or_default() } else { arg[2..].to_string() };
                match v.strip_prefix("panic=") {
                    Some(p @ ("unwind" | "abort" | "immediate-abort")) => {
                        cfg.replace_value("panic", p);
                    }
                    _ if v.strip_prefix("opt-level=").is_some_and(level_ok) => opt_level = v["opt-level=".len()..].to_string(),
                    _ => {
                        eprintln!("error: unknown codegen option `{v}`");
                        return ExitCode::FAILURE;
                    }
                }
            }
            _ if arg.starts_with("--cfg") => {
                let v = value("--cfg").unwrap_or_default();
                match v.split_once('=') {
                    Some((n, val)) => cfg.set_value(n.trim(), val.trim().trim_matches('"')),
                    None => cfg.set(v.trim()),
                };
            }
            _ if arg.starts_with("--edition") => {
                let v = value("--edition").unwrap_or_default();
                let Some(e) = Edition::parse(&v) else {
                    eprintln!("error: unknown edition `{v}`");
                    return ExitCode::FAILURE;
                };
                edition = e;
            }
            _ if arg.starts_with("--threads") => {
                let v = value("--threads").unwrap_or_default();
                let Ok(n) = v.parse() else {
                    eprintln!("error: bad thread count `{v}`");
                    return ExitCode::FAILURE;
                };
                options.threads = n;
            }
            _ if arg.starts_with("--parse-tree") => tree = value("--parse-tree").map(PathBuf::from),
            _ if arg.starts_with("--extern") => {
                let v = value("--extern").unwrap_or_default();
                let Some((name, path)) = v.split_once('=') else {
                    eprintln!("error: expected `--extern name=path`");
                    return ExitCode::FAILURE;
                };
                externs.push((name.to_string(), PathBuf::from(path)));
            }
            _ if arg.starts_with("--proc-macro-server") => {
                options.proc_macro_server = value("--proc-macro-server").map(PathBuf::from);
            }
            _ if arg.starts_with('-') => {
                eprintln!("error: unknown option `{arg}`");
                return ExitCode::FAILURE;
            }
            _ => files.push(PathBuf::from(arg)),
        }
    }
    if let Some(dir) = tree {
        return parse_tree(&dir, edition, roundtrip, options);
    }
    if files.is_empty() {
        eprintln!("usage: epitome-rustc2 [--edition N] [--threads N] [--dump-ast | --sexp | --pretty] root.rs...");
        return ExitCode::FAILURE;
    }
    if tokens {
        return print_tokens(&files);
    }

    // `std` and its crates first, as rust-src has them.
    let mut std_roots = Vec::new();
    if with_std {
        let roots = epitome_rustc2::sysroot::rust_src()
            .ok_or_else(|| "no rust-src: `rustup component add rust-src`".to_string())
            .and_then(|lib| epitome_rustc2::sysroot::std_crates(&lib));
        match roots {
            Ok(r) => std_roots = r,
            Err(e) => {
                eprintln!("error: {e}");
                return ExitCode::FAILURE;
            }
        }
        for (n, v) in epitome_rustc2::sysroot::std_cfg() {
            match v {
                Some(v) => cfg.set_value(n, v),
                None => cfg.set(n),
            };
        }
        options.env.extend(epitome_rustc2::sysroot::std_env(std::env::consts::ARCH));
    }
    // The standard library's crates (from rust-src) are built as rustc's
    // prebuilt `std` is: without `debug_assertions`.
    let is_library = |p: &Path| p.to_string_lossy().contains("/lib/rustlib/src/rust/library/");
    let library_cfg = (!no_cfg).then(|| {
        let mut c = cfg.clone();
        c.unset("debug_assertions");
        c
    });
    if !no_cfg {
        options.cfg = Some(cfg);
    }
    if !externs.is_empty() && options.proc_macro_server.is_none() {
        options.proc_macro_server = epitome_rustc2::proc_macro::Server::from_rustc();
    }
    let session = Session::new(options);
    // A program with kernels gets the launch runtime, as a crate before
    // its own (written out of the compiler, where the session can read it).
    let mut runtime: Vec<CrateRoot> = Vec::new();
    if launch_runtime
        && !emit_kernels
        && output.is_some()
        && let Some(last) = files.last()
        && std::fs::read_to_string(last).is_ok_and(|t| t.contains("epitome::target"))
    {
        let text = include_str!("../runtime/rv64.rs");
        let path = std::env::temp_dir().join(format!("epitome-launch-rv64-{:016x}.rs", epitome_rustc2::hash::fnv(text.as_bytes())));
        if std::fs::read_to_string(&path).ok().as_deref() != Some(text)
            && let Err(e) = std::fs::write(&path, text)
        {
            eprintln!("error: writing the launch runtime to {}: {e}", path.display());
            return ExitCode::FAILURE;
        }
        runtime.push(CrateRoot::new("epitome_launch_rv64", path).edition(Edition::E2024));
    }
    let runtime_names: Vec<String> = runtime.iter().map(|r| r.name.clone()).collect();
    let roots: Vec<CrateRoot> = std_roots
        .into_iter()
        .chain(runtime)
        .chain(files.iter().map(|f| {
            let mut root = CrateRoot::new(crate_name(f), f).edition(edition);
            for (name, lib) in &externs {
                root = root.proc_macro(name.clone(), lib.clone());
            }
            root
        }))
        .map(|root| match &library_cfg {
            Some(c) if is_library(&root.root) || runtime_names.contains(&root.name) => root.cfg(c.clone()),
            _ => root,
        })
        .collect();
    // Each crate's optimisation level: its own (`--crate-opt`), else 2 for
    // the standard library's and `-C opt-level` for the others.
    let levels: Vec<String> = roots
        .iter()
        .map(|root| match crate_opt.iter().rev().find(|(n, _)| *n == root.name) {
            Some((_, l)) => l.clone(),
            None if is_library(&root.root) || runtime_names.contains(&root.name) => "2".to_string(),
            None => opt_level.clone(),
        })
        .collect();
    let mut crates = session.parse_crates(roots);
    // A crate whose root file could not be read or parsed has no files,
    // and what comes after parsing needs every crate's root (the first
    // crate's is where trait selection starts): report and stop.
    if crates.iter().any(|c| c.files.is_empty()) {
        for e in crates.iter().flat_map(|c| &c.errors) {
            eprint!("{}", e.render_in(&session.sources));
        }
        return ExitCode::FAILURE;
    }
    if !no_expand {
        let stats = session.expand(&mut crates);
        if expand_stats {
            eprintln!(
                "expansion: {} rounds, {} calls expanded, {} built-in calls left, {} files loaded",
                stats.rounds, stats.expanded, stats.builtins_left, stats.files_loaded
            );
            eprintln!("per round: {:?}", stats.per_round);
            eprintln!("found only by name (not by scope, path or import): {:?}", stats.found_by_name);
            if stats.per_round.last().is_some_and(|&n| n > 0) {
                eprintln!("still expanding in the last round: {:?}", stats.last_round);
            }
            let mut missing: Vec<_> = stats.unresolved.iter().collect();
            missing.sort_by(|a, b| b.1.cmp(a.1).then(a.0.cmp(b.0)));
            let total: usize = missing.iter().map(|m| m.1).sum();
            eprintln!("not found: {total} calls to {} macros", missing.len());
            for (name, n) in missing.iter().take(20) {
                eprintln!("  {n:6} {name}!");
            }
        }
    }
    if dump_sigs || types_stats || dump_types || typeck_stats || dump_layout || dump_mono || mono_stats || dump_ir || lower_stats || output.is_some() {
        let r = session.resolve(&crates);
        let t = std::time::Instant::now();
        let tcx = session.collect_types(&crates, &r);
        let elapsed = t.elapsed();
        if dump_sigs {
            print!("{}", epitome_rustc2::ty::dump_signatures(&tcx, &r));
        }
        if types_stats {
            let s = tcx.stats();
            eprintln!(
                "types: {} signatures ({} with error types), {} impls, {} distinct types, in {:.3} s",
                s.signatures,
                s.with_errors,
                s.impls,
                s.distinct_types,
                elapsed.as_secs_f64()
            );
            for (why, n) in s.reasons.iter().take(20) {
                eprintln!("  {n:6} {why}");
            }
        }
        if dump_layout {
            let index = epitome_rustc2::typeck::Index::new(&tcx, &r);
            let cx = epitome_rustc2::layout::LayoutCx::new(&tcx, &r, &crates, &index, Default::default());
            print!("{}", cx.dump(Some(crates.len() as u32 - 1)));
        }
        // The kernels, each machine's in a module of its own, linked into
        // an image for the host to carry.
        let mut images: Vec<(String, Vec<u8>)> = Vec::new();
        if let Some(out) = &output {
            let last = crates.len() as u32 - 1;
            let mut machines: Vec<String> = epitome_rustc2::device::kernels(&r, &crates, last).into_iter().map(|(_, t)| t).collect();
            machines.sort();
            machines.dedup();
            let program = files.last().expect("a crate root");
            for name in machines {
                let Some(machine) = epitome_rustc2::device::machine(&name) else {
                    eprintln!("error: no machine `{name}` for `#[epitome::target]` (there is `riscv64`)");
                    return ExitCode::FAILURE;
                };
                let level = levels.last().cloned().unwrap_or_else(|| "2".to_string());
                let mut lowered = match epitome_rustc2::device::compile(program, &crate_name(program), edition, &machine, session.options.threads, &level) {
                    Ok(l) => l,
                    Err(e) => {
                        eprintln!("error: {e}");
                        return ExitCode::FAILURE;
                    }
                };
                let path = PathBuf::from(format!("{}.{name}.o", out.display()));
                if let Err(e) = build(&mut lowered.module, machine.target, &path, Some(epitome_opt::inline::OptLevel::O2), true) {
                    eprintln!("error: {name}: {e}");
                    return ExitCode::FAILURE;
                }
                if !emit_kernels {
                    let object = std::fs::read(&path).unwrap_or_default();
                    let stubs = ["putchar", "memcpy", "memmove", "memset", "memcmp", "bcmp"];
                    match epitome_codegen::flat::link_riscv64(&[object], 0x1_0000, 0x1000, &stubs) {
                        Ok(image) => images.push((name.clone(), image.encode(&name))),
                        Err(e) => {
                            eprintln!("error: linking the {name} kernels: {e}");
                            return ExitCode::FAILURE;
                        }
                    }
                }
            }
            if emit_kernels {
                return ExitCode::SUCCESS;
            }
        }
        if dump_mono || mono_stats || dump_ir || lower_stats || output.is_some() {
            let index = epitome_rustc2::typeck::Index::new(&tcx, &r);
            // Only the bodies the program reaches are type checked, as
            // monomorphisation finds them.
            // (`EPITOME_TYPECK_ALL` checks every body first, as before.)
            let mut res = epitome_rustc2::typeck::TypeckResults::empty(&crates);
            let t = std::time::Instant::now();
            let roots = epitome_rustc2::mono::roots(&r, &tcx, &crates, crates.len() as u32 - 1, lib);
            let mono = if std::env::var_os("EPITOME_TYPECK_ALL").is_some() {
                res = session.typeck_with(&crates, &r, &tcx, &index);
                session.collect_mono_items(&crates, &r, &tcx, &index, &res, roots.clone())
            } else {
                session.collect_mono_items_lazily(&crates, &r, &tcx, &index, &mut res, roots.clone())
            };
            let elapsed = t.elapsed();
            if dump_mono {
                print!("{}", mono.dump(&r, &tcx));
            }
            if mono_stats {
                let missed: usize = mono.unresolved.iter().map(|x| x.1).sum();
                eprintln!(
                    "mono: {} items from {} uses ({missed} not resolved), {} bodies type checked, in {:.3} s",
                    mono.items.len(),
                    mono.uses,
                    res.stats.bodies,
                    elapsed.as_secs_f64()
                );
                // All of them with EPITOME_MONO_ALL set.
                let shown = if std::env::var_os("EPITOME_MONO_ALL").is_some() { usize::MAX } else { 20 };
                for (why, n) in mono.unresolved.iter().take(shown) {
                    eprintln!("  {n:6} {why}");
                }
            }
            if dump_ir || lower_stats || output.is_some() {
                let t = std::time::Instant::now();
                let target = epitome_rustc2::lower::Target::host();
                let mut lowered = session.lower(&crates, &r, &tcx, &index, &res, &mono, target);
                if lib {
                    lowered.export(&roots);
                }
                let elapsed = t.elapsed();
                if dump_ir {
                    print!("{}", lowered.dump());
                }
                if lower_stats {
                    eprintln!(
                        "lower: {} functions, {} not lowered, {} that trap where they use something not supported, in {:.3} s",
                        lowered.functions,
                        lowered.failures.len(),
                        lowered.partial.len(),
                        elapsed.as_secs_f64()
                    );
                    for f in lowered.failures.iter().chain(&lowered.partial) {
                        eprintln!("  {}: {}", f.name, f.why);
                    }
                }
                if let Some(out) = &output {
                    let last = crates.len() as u32 - 1;
                    let own: Vec<_> = lowered.failures.iter().filter(|f| item_crate(&r, f.item) == Some(last)).collect();
                    if !own.is_empty() {
                        for f in own {
                            eprintln!("error: could not lower {}: {}", f.name, f.why);
                        }
                        return ExitCode::FAILURE;
                    }
                    // Each function at its crate's level; what lowering made
                    // itself at the given crates' (2 if that is 0).
                    let level = levels.iter().any(|l| l != "0").then(|| {
                        lowered.set_opt_levels(&r, |k| levels.get(k as usize).cloned());
                        epitome_opt::inline::OptLevel::parse(&opt_level).unwrap_or(epitome_opt::inline::OptLevel::O2)
                    });
                    // Each machine's image, where the program launches it.
                    for (name, bytes) in &images {
                        let m = &mut lowered.module;
                        if let Some(g) = m.find_global(&format!("epitome.image.{name}"))
                            && let epitome_ir::module::Value::Const(c) = m.const_bytes(bytes)
                        {
                            let ty = m.consts.get(c).ty;
                            if let epitome_ir::module::GlobalKind::Variable(v) = &mut m.globals[g.index()].kind {
                                v.init = Some(c);
                                v.ty = ty;
                            }
                        }
                    }
                    if let Err(e) = build(&mut lowered.module, target, out, level, emit_obj) {
                        eprintln!("error: {e}");
                        return ExitCode::FAILURE;
                    }
                }
            }
        }
        if dump_types || typeck_stats {
            let t = std::time::Instant::now();
            let res = session.typeck(&crates, &r, &tcx);
            let elapsed = t.elapsed();
            if dump_types {
                print!("{}", res.dump(&crates, &r, &tcx, &session.sources));
            }
            if typeck_stats {
                let s = &res.stats;
                eprintln!(
                    "typeck: {} bodies ({} clean), {} nodes typed, {} with error types, {} unknown, in {:.3} s",
                    s.bodies,
                    s.clean_bodies,
                    s.nodes,
                    s.error_nodes,
                    s.unknown_nodes,
                    elapsed.as_secs_f64()
                );
                for (why, n) in s.reasons.iter().take(25) {
                    eprintln!("  {n:6} {why}");
                }
            }
        }
    }
    if resolve_stats || dump_res {
        let t = std::time::Instant::now();
        let r = session.resolve(&crates);
        let elapsed = t.elapsed();
        if dump_res {
            print!("{}", r.dump(&crates, &session.sources));
        }
        if resolve_stats {
            let s = r.stats(&crates);
            eprintln!(
                "resolution: {} defs, {} modules, {} imports ({} failed), in {:.3} s",
                s.defs,
                s.modules,
                s.imports,
                s.imports_failed,
                elapsed.as_secs_f64()
            );
            eprintln!(
                "paths: {} resolved, {} partial (left to type checking), {} unresolved, of {}",
                s.resolved, s.partial, s.unresolved, s.paths
            );
            for (name, n) in s.unresolved_names.iter().take(25) {
                eprintln!("  {n:6} {name}");
            }
            eprintln!("imports that did not resolve (first 40):");
            for line in r.failed_import_lines().iter().take(40) {
                eprintln!("  {line}");
            }
        }
    }
    if list_files {
        let mut included = session.included.lock().unwrap().clone();
        included.sort();
        included.dedup();
        for f in included {
            println!("{}", f.display());
        }
    }
    let mut failed = false;
    for c in &crates {
        for f in &c.files {
            if list_files {
                println!("{}", f.source.name);
            }
            let several = crates.len() > 1 || c.files.len() > 1;
            if several && (dump || sexp || print) {
                println!("// {}", f.source.name);
            }
            if dump {
                print!("{}", f.ast.dump(f.ast.root));
            }
            if sexp {
                println!("{}", f.ast.sexp(f.ast.root));
            }
            if print {
                print!("{}", pretty(&f.ast));
            }
        }
        for e in &c.errors {
            eprint!("{}", e.render_in(&session.sources));
            failed = true;
        }
    }
    if failed { ExitCode::FAILURE } else { ExitCode::SUCCESS }
}

/// The crate an item is defined in.
fn item_crate(r: &epitome_rustc2::resolve::Resolver, it: epitome_rustc2::mono::Item) -> Option<u32> {
    use epitome_rustc2::mono::Item;
    let d = match it {
        Item::Fn(d, _) | Item::Closure(_, d, _) | Item::Static(d) | Item::Foreign(d) | Item::Intrinsic(d, _) | Item::Const(d, _) => d,
        Item::VTable(..) | Item::DropGlue(_) => return None,
    };
    Some(r.defs[d.0 as usize].krate)
}

/// Optimises `m` (at `level`, for the functions without a level of their
/// own; not at all if `None`), compiles it for `target` and writes the
/// object file, then links it with `cc` into `out` (unless `emit_obj`, when
/// the object file is `out`).
fn build(
    m: &mut epitome_ir::Module,
    target: epitome_rustc2::lower::Target,
    out: &Path,
    level: Option<epitome_opt::inline::OptLevel>,
    emit_obj: bool,
) -> Result<(), String> {
    if let Some(level) = level {
        epitome_opt::pipeline::run(m, &epitome_opt::pipeline::Options { level, ..Default::default() });
    }
    let t = if target.triple.starts_with("x86_64") {
        epitome_codegen::target::Target::x86_64()
    } else if target.triple.starts_with("riscv64") {
        epitome_codegen::target::Target::riscv64()
    } else {
        epitome_codegen::target::Target::aarch64()
    };
    let compiled = epitome_codegen::compile(&t, m, &epitome_codegen::Options::default())?;
    let object = compiled.object_file(&t)?;
    let obj_path = if emit_obj { out.to_path_buf() } else { out.with_extension("o") };
    std::fs::write(&obj_path, object).map_err(|e| format!("{}: {e}", obj_path.display()))?;
    if emit_obj {
        return Ok(());
    }
    // libm: Epitome's code generator calls `floorf`, `fma` and their
    // kin where the machine has no instruction for them.
    let status = std::process::Command::new("cc").arg(&obj_path).arg("-o").arg(out).arg("-lm").status();
    let _ = std::fs::remove_file(&obj_path);
    match status {
        Ok(s) if s.success() => Ok(()),
        Ok(s) => Err(format!("linking with cc failed: {s}")),
        Err(e) => Err(format!("cannot run cc: {e}")),
    }
}

/// A crate's name from its root file: the file's name, or for `lib.rs`,
/// `main.rs` and `mod.rs` the directory's (above `src`).
fn crate_name(root: &Path) -> String {
    let stem = root.file_stem().map_or(String::new(), |s| s.to_string_lossy().into_owned());
    if !matches!(stem.as_str(), "lib" | "main" | "mod") {
        return stem;
    }
    let mut dir = root.parent();
    if dir.is_some_and(|d| d.file_name().is_some_and(|n| n == "src")) {
        dir = dir.and_then(Path::parent);
    }
    dir.and_then(|d| d.file_name()).map_or(stem, |n| n.to_string_lossy().replace('-', "_"))
}

fn print_tokens(files: &[PathBuf]) -> ExitCode {
    for file in files {
        let src = match std::fs::read_to_string(file) {
            Ok(t) => Source::new(file.display().to_string(), t),
            Err(e) => {
                eprintln!("error: {}: {e}", file.display());
                return ExitCode::FAILURE;
            }
        };
        match lex(&src.text) {
            Ok(ts) => ts.iter().for_each(|t| println!("{}", t.0)),
            Err(e) => {
                eprint!("{}", e.render(&src));
                return ExitCode::FAILURE;
            }
        }
    }
    ExitCode::SUCCESS
}

/// Parses every `.rs` file under `dir`, in parallel.
fn parse_tree(dir: &Path, edition: Edition, roundtrip: bool, options: Options) -> ExitCode {
    let mut files = Vec::new();
    collect(dir, &mut files);
    files.sort();
    let mut editions = FastMap::default();
    let roots: Vec<CrateRoot> = files
        .iter()
        .map(|f| {
            let e = f.parent().and_then(|d| crate_edition(d, &mut editions)).unwrap_or(edition);
            CrateRoot::new(f.display().to_string(), f).edition(e).follow_modules(false)
        })
        .collect();
    let session = Session::new(options);
    let crates = session.parse_crates(roots);
    let mut failed = 0;
    for c in &crates {
        for e in &c.errors {
            eprint!("{}", e.render_in(&session.sources));
        }
        let mut bad = !c.errors.is_empty();
        if roundtrip
            && let Some(f) = c.files.first()
            && let Err(message) = check_roundtrip(&f.ast, c.edition)
        {
            eprintln!("{}: roundtrip: {message}", f.source.name);
            bad = true;
        }
        failed += bad as usize;
    }
    println!("{} files, {} failed", files.len(), failed);
    if failed == 0 { ExitCode::SUCCESS } else { ExitCode::FAILURE }
}

/// Prints `ast`, parses the output, and compares the trees.
fn check_roundtrip(ast: &Ast, edition: Edition) -> Result<(), String> {
    let src = Source::new("<pretty>", pretty(ast));
    let again = parse_str(&src.text, edition).map_err(|e| format!("the output does not parse:\n{}", e.render(&src)))?;
    match first_difference(ast, ast.root, &again, again.root) {
        None => Ok(()),
        Some((a, b)) => Err(format!("the tree changed:\n  {a}\n  {b}")),
    }
}

/// The edition of the package whose `Cargo.toml` is nearest above `dir`.
fn crate_edition(dir: &Path, cache: &mut FastMap<PathBuf, Option<Edition>>) -> Option<Edition> {
    if let Some(&e) = cache.get(dir) {
        return e;
    }
    let manifest = dir.join("Cargo.toml");
    let e = if let Ok(text) = std::fs::read_to_string(&manifest) {
        // `edition = "2021"` in `[package]`; a workspace's own manifest
        // may have none, and `edition.workspace = true` is not followed.
        let value = text.lines().find_map(|l| {
            let rest = l.trim().strip_prefix("edition")?.trim_start().strip_prefix('=')?;
            Some(rest.trim().trim_matches('"').to_string())
        });
        Some(value.and_then(|v| Edition::parse(&v)).unwrap_or(Edition::E2015))
    } else {
        dir.parent().and_then(|p| crate_edition(p, cache))
    };
    cache.insert(dir.to_path_buf(), e);
    e
}

fn collect(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else { return };
    for e in entries.flatten() {
        let p = e.path();
        if p.is_dir() {
            collect(&p, out);
        } else if p.extension().is_some_and(|x| x == "rs") {
            out.push(p);
        }
    }
}
