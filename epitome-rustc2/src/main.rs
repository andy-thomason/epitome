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
//! - `--parse-tree dir`: parse every `.rs` file under `dir` (each on its
//!   own, not following modules), and report each that fails and a count.
//!   Each file is read in the edition of the nearest `Cargo.toml` above it
//!   (2015 if it names none, as for Cargo), or `--edition` if there is no
//!   `Cargo.toml`. With `--roundtrip`, each file is also printed and
//!   parsed again, and must give the same tree.

use epitome_rustc2::ast::Ast;
use epitome_rustc2::cfg::CfgSet;
use epitome_rustc2::lexer::{Edition, lex};
use epitome_rustc2::parse::parse_str;
use epitome_rustc2::pretty::{first_difference, pretty};
use epitome_rustc2::session::{CrateRoot, Options, Session};
use epitome_rustc2::source::Source;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

fn main() -> ExitCode {
    let mut edition = Edition::default();
    let mut options = Options::default();
    let (mut dump, mut sexp, mut tokens, mut print, mut roundtrip) = (false, false, false, false, false);
    let mut list_files = false;
    let (mut no_expand, mut expand_stats) = (false, false);
    let (mut resolve_stats, mut dump_res) = (false, false);
    let mut tree: Option<PathBuf> = None;
    let mut cfg = CfgSet::host();
    let mut externs: Vec<(String, PathBuf)> = Vec::new();
    let mut no_cfg = false;
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

    if !no_cfg {
        options.cfg = Some(cfg);
    }
    if !externs.is_empty() && options.proc_macro_server.is_none() {
        options.proc_macro_server = epitome_rustc2::proc_macro::Server::from_rustc();
    }
    let session = Session::new(options);
    let roots = files
        .iter()
        .map(|f| {
            let mut root = CrateRoot::new(crate_name(f), f).edition(edition);
            for (name, lib) in &externs {
                root = root.proc_macro(name.clone(), lib.clone());
            }
            root
        })
        .collect();
    let mut crates = session.parse_crates(roots);
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
    let mut editions = HashMap::new();
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
fn crate_edition(dir: &Path, cache: &mut HashMap<PathBuf, Option<Edition>>) -> Option<Edition> {
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
