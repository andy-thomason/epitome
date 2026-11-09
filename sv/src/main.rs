//! `epitome-sv [options] file.sv...`: for now, preprocesses, parses and
//! elaborates the files as one compilation unit and reports the first
//! error with its position.
//!
//! Options:
//!
//! - `-E`: print the preprocessed text, and stop.
//! - `--ast`: print the syntax tree.
//! - `--design`: elaborate, and print the design.
//! - `--parse-only`: stop after parsing; `--elab-only`: after elaborating,
//!   with a count of what the elaborator has not done yet.
//! - `--gaps`: elaborate, and list those gaps by kind of syntax.
//! - `--top name`: elaborate `name` as a top-level instance (repeatable);
//!   by default, every design element nothing instantiates.
//! - `+name` or `+name=value`: plusargs for the simulation (21.6).
//!
//! Without these, the design is simulated by the interpreter. The exit
//! status is non-zero if `$fatal` was called or the run hit a limit.
//! - `-I dir`, `-Idir`, `+incdir+dir[+dir...]`: search `dir` for
//!   `` `include `` files.
//! - `-D name[=text]`, `-Dname[=text]`, `+define+name[=text][+...]`: define
//!   a macro.
//!
//! Other options are ignored, so that test runners can pass their own.

use epitome_sv::design::print::print as print_design;
use epitome_sv::parse::parse;
use epitome_sv::preprocess::{Options, preprocess_files};
use epitome_sv::source::SourceMap;
use std::path::PathBuf;
use std::process::ExitCode;

fn define(options: &mut Options, d: &str) {
    let (name, text) = d.split_once('=').unwrap_or((d, ""));
    options.defines.push((name.to_string(), text.to_string()));
}

fn main() -> ExitCode {
    // Elaboration recurses as deeply as the design's hierarchy.
    std::thread::Builder::new().stack_size(1 << 30).spawn(run).unwrap().join().unwrap()
}

fn run() -> ExitCode {
    let mut options = Options::default();
    let mut files = Vec::new();
    let mut print = false;
    let mut ast = false;
    let mut design = false;
    let mut parse_only = false;
    let mut elab_only = false;
    let mut gaps = false;
    let mut netlist = false;
    let mut plusargs = Vec::new();
    let mut tops = Vec::new();
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        if arg == "-E" {
            print = true;
        } else if arg == "--ast" {
            ast = true;
        } else if arg == "--design" {
            design = true;
        } else if arg == "--parse-only" {
            parse_only = true;
        } else if arg == "--elab-only" {
            elab_only = true;
        } else if arg == "--netlist" {
            netlist = true;
        } else if arg == "--gaps" {
            (elab_only, gaps) = (true, true);
        } else if arg == "--top" {
            tops.push(args.next().unwrap_or_default());
        } else if let Some(dir) = arg.strip_prefix("-I") {
            let dir = if dir.is_empty() { args.next().unwrap_or_default() } else { dir.to_string() };
            options.include_dirs.push(dir.into());
        } else if let Some(d) = arg.strip_prefix("-D") {
            let d = if d.is_empty() { args.next().unwrap_or_default() } else { d.to_string() };
            define(&mut options, &d);
        } else if let Some(dirs) = arg.strip_prefix("+incdir+") {
            options.include_dirs.extend(dirs.split('+').filter(|d| !d.is_empty()).map(PathBuf::from));
        } else if let Some(ds) = arg.strip_prefix("+define+") {
            ds.split('+').filter(|d| !d.is_empty()).for_each(|d| define(&mut options, d));
        } else if arg.starts_with('+') && !arg.starts_with("+incdir+") && !arg.starts_with("+define+") {
            plusargs.push(arg.clone());
        } else if !arg.starts_with('-') && !arg.starts_with('+') {
            files.push(PathBuf::from(arg));
        }
    }
    if files.is_empty() {
        eprintln!("usage: epitome-sv [-E] [--ast] [--design] [--netlist] [-I dir] [-D name[=text]] file.sv...");
        return ExitCode::from(2);
    }
    let map = SourceMap::new();
    match preprocess_files(&map, &files, &options) {
        Ok(out) if print => {
            print!("{}", out.text(&map));
            ExitCode::SUCCESS
        }
        Ok(out) => match parse(&out) {
            Ok(tree) => {
                if ast {
                    print!("{}", tree.dump(tree.root));
                }
                if parse_only {
                    return ExitCode::SUCCESS;
                }
                match epitome_sv::design::elab::elaborate_tops(&tree, &out, &tops) {
                    Ok(d) => {
                        if design {
                            print!("{}", print_design(&d));
                        }
                        if netlist {
                            return print_netlist(&d);
                        }
                        if design || elab_only {
                            let (unknown, opaque) = d.gaps();
                            eprintln!("epitome-sv: {unknown} expressions of unknown type, {opaque} items kept as syntax");
                            if gaps {
                                print_gaps(&d, &tree);
                            }
                            return ExitCode::SUCCESS;
                        }
                        let opts = epitome_sv::sim::Options { plusargs, ..Default::default() };
                        let mut stdout = std::io::stdout();
                        let r = epitome_sv::sim::simulate(&d, &tree, &mut stdout, &opts);
                        for u in &r.unsupported {
                            eprintln!("epitome-sv: not simulated: {u}");
                        }
                        if r.failed { ExitCode::FAILURE } else { ExitCode::SUCCESS }
                    }
                    Err(e) => {
                        eprint!("{}", map.render(&e));
                        ExitCode::FAILURE
                    }
                }
            }
            Err(e) => {
                eprint!("{}", map.render(&e));
                ExitCode::FAILURE
            }
        },
        Err(e) => {
            eprint!("{e}");
            ExitCode::FAILURE
        }
    }
}

/// The design's gaps, counted by the kind of syntax they came from.
fn print_gaps(d: &epitome_sv::design::Design, tree: &epitome_sv::ast::Ast) {
    use epitome_sv::design::types::Types;
    use epitome_sv::design::{ExprKind, Item, StmtKind};
    use std::collections::BTreeMap;
    let mut counts: BTreeMap<String, usize> = BTreeMap::new();
    let mut count = |what: &str, n| *counts.entry(format!("{what} {:?}", tree.kind(n))).or_default() += 1;
    for e in d.exprs.iter().filter(|e| e.ty == Types::UNKNOWN) {
        count(if matches!(e.kind, ExprKind::Opaque) { "expression kept as syntax:" } else { "expression of unknown type:" }, e.node);
    }
    for s in d.stmts.iter().filter(|s| matches!(s.kind, StmtKind::Opaque)) {
        count("statement kept as syntax:", s.node);
    }
    for i in d.scopes.iter().flat_map(|s| &s.items) {
        if let Item::Opaque(n) = i {
            count("item kept as syntax:", *n);
        }
    }
    let mut v: Vec<_> = counts.into_iter().collect();
    v.sort_by_key(|(_, n)| std::cmp::Reverse(*n));
    for (k, n) in v {
        eprintln!("{n:6} {k}");
    }
}

/// The compiled netlist of each top-level instance, as Epitome IR, with
/// where its ports are in a group's state.
fn print_netlist(d: &epitome_sv::design::Design) -> ExitCode {
    for &top in &d.tops {
        match epitome_sv::netlist::compile(d, top) {
            Ok(n) => {
                print!("{}", n.ir);
                eprintln!(
                    "epitome-sv: {} lanes, {} bytes of state; entry {}; {} assertions left out",
                    n.lanes,
                    n.size,
                    n.entry,
                    n.dropped
                );
                for p in &n.ports {
                    eprintln!("epitome-sv:   {:?} {} [{}] at {}", p.dir, p.name, p.width, p.offset);
                }
            }
            Err(e) => {
                eprintln!("epitome-sv: {}: {e}", d.name(d.instance(top).name));
                return ExitCode::FAILURE;
            }
        }
    }
    ExitCode::SUCCESS
}
