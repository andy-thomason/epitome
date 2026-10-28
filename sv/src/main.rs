//! `epitome-sv [options] file.sv...`: for now, preprocesses, parses and
//! elaborates the files as one compilation unit and reports the first
//! error with its position.
//!
//! Options:
//!
//! - `-E`: print the preprocessed text, and stop.
//! - `--ast`: print the syntax tree.
//! - `--design`: elaborate, and print the design.
//! - `--parse-only`: stop after parsing; `--elab-only`: after elaborating.
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

use epitome_sv::design::elab::elaborate;
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
    let mut plusargs = Vec::new();
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
        eprintln!("usage: epitome-sv [-E] [--ast] [--design] [-I dir] [-D name[=text]] file.sv...");
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
                match elaborate(&tree, &out) {
                    Ok(d) => {
                        if design {
                            print!("{}", print_design(&d));
                        }
                        if design || elab_only {
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
