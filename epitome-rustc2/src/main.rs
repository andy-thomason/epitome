//! `epitome-rustc2 [options] root.rs...`: for now, loads each crate (its
//! root file and the files of its modules, all parsed in parallel) and
//! reports every error with its position.
//!
//! Options:
//!
//! - `--edition 2015|2018|2021|2024` (default 2024), also `--edition=N`.
//! - `--threads N`: worker threads (default one per core).
//! - `--dump-ast`: print each file's syntax tree, one node per line.
//! - `--sexp`: print each file's syntax tree on one line.
//! - `--pretty`: print each file's syntax tree back as Rust source.
//! - `--tokens file.rs`: print the tokens of one file, one per line.
//! - `--parse-tree dir`: parse every `.rs` file under `dir` (each on its
//!   own, not following modules), and report each that fails and a count.
//!   Each file is read in the edition of the nearest `Cargo.toml` above it
//!   (2015 if it names none, as for Cargo), or `--edition` if there is no
//!   `Cargo.toml`. With `--roundtrip`, each file is also printed and
//!   parsed again, and must give the same tree.

use epitome_rustc2::ast::Ast;
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
    let mut tree: Option<PathBuf> = None;
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

    let session = Session::new(options);
    let roots = files
        .iter()
        .map(|f| {
            let name = f.file_stem().map_or("main".into(), |s| s.to_string_lossy().into_owned());
            CrateRoot::new(name, f).edition(edition)
        })
        .collect();
    let crates = session.parse_crates(roots);
    let mut failed = false;
    for c in &crates {
        for f in &c.files {
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
