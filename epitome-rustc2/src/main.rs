//! `epitome-rustc2 [options] file.rs`: for now, parses a file and reports
//! the first error with its position.
//!
//! Options:
//!
//! - `--edition 2015|2018|2021|2024` (default 2024), also `--edition=N`.
//! - `--dump-ast`: print the syntax tree, one node per line.
//! - `--sexp`: print the syntax tree on one line.
//! - `--tokens`: print the tokens, one per line.
//! - `--parse-tree dir`: parse every `.rs` file under `dir`, and report
//!   each that fails and a count.

use epitome_rustc2::lexer::{Edition, lex};
use epitome_rustc2::parse::parse_str;
use epitome_rustc2::source::{Source, SourceMap};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

fn main() -> ExitCode {
    // The parser recurses as deeply as expressions nest.
    std::thread::Builder::new().stack_size(1 << 28).spawn(run).unwrap().join().unwrap()
}

fn run() -> ExitCode {
    let mut edition = Edition::default();
    let mut dump = false;
    let mut sexp = false;
    let mut tokens = false;
    let mut tree: Option<PathBuf> = None;
    let mut files = Vec::new();
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        let value = |args: &mut dyn Iterator<Item = String>, name: &str| {
            arg.strip_prefix(&format!("{name}=")).map(str::to_string).or_else(|| args.next())
        };
        match arg.as_str() {
            "--dump-ast" => dump = true,
            "--sexp" => sexp = true,
            "--tokens" => tokens = true,
            _ if arg.starts_with("--edition") => {
                let v = value(&mut args, "--edition").unwrap_or_default();
                match Edition::parse(&v) {
                    Some(e) => edition = e,
                    None => {
                        eprintln!("error: unknown edition `{v}`");
                        return ExitCode::FAILURE;
                    }
                }
            }
            _ if arg.starts_with("--parse-tree") => tree = value(&mut args, "--parse-tree").map(PathBuf::from),
            _ if arg.starts_with('-') => {
                eprintln!("error: unknown option `{arg}`");
                return ExitCode::FAILURE;
            }
            _ => files.push(PathBuf::from(arg)),
        }
    }
    if let Some(dir) = tree {
        return parse_tree(&dir, edition);
    }
    if files.is_empty() {
        eprintln!("usage: epitome-rustc2 [--edition N] [--dump-ast | --sexp | --tokens] file.rs");
        return ExitCode::FAILURE;
    }
    let map = SourceMap::new();
    for file in &files {
        let text = match std::fs::read_to_string(file) {
            Ok(t) => t,
            Err(e) => {
                eprintln!("error: {}: {e}", file.display());
                return ExitCode::FAILURE;
            }
        };
        let src = map.add(Source::new(file.display().to_string(), text));
        if tokens {
            match lex(&src.text) {
                Ok(ts) => ts.iter().for_each(|t| println!("{}", t.0)),
                Err(e) => {
                    eprint!("{}", e.render(src));
                    return ExitCode::FAILURE;
                }
            }
            continue;
        }
        match parse_str(&src.text, edition) {
            Ok(ast) => {
                if dump {
                    print!("{}", ast.dump(ast.root));
                }
                if sexp {
                    println!("{}", ast.sexp(ast.root));
                }
            }
            Err(e) => {
                eprint!("{}", e.render(src));
                return ExitCode::FAILURE;
            }
        }
    }
    ExitCode::SUCCESS
}

/// Parses every `.rs` file under `dir`.
fn parse_tree(dir: &Path, edition: Edition) -> ExitCode {
    let mut files = Vec::new();
    collect(dir, &mut files);
    files.sort();
    let mut failed = 0;
    for file in &files {
        let Ok(text) = std::fs::read_to_string(file) else { continue };
        let src = Source::new(file.display().to_string(), text);
        if let Err(e) = parse_str(&src.text, edition) {
            failed += 1;
            eprint!("{}", e.render(&src));
        }
    }
    println!("{} files, {} failed", files.len(), failed);
    if failed == 0 { ExitCode::SUCCESS } else { ExitCode::FAILURE }
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
