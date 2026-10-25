//! `epitome-sv [options] file.sv...`: for now, preprocesses and parses the
//! files as one compilation unit and reports the first error with its
//! position.
//!
//! Options:
//!
//! - `-E`: print the preprocessed text, and stop.
//! - `--ast`: print the syntax tree.
//! - `-I dir`, `-Idir`, `+incdir+dir[+dir...]`: search `dir` for
//!   `` `include `` files.
//! - `-D name[=text]`, `-Dname[=text]`, `+define+name[=text][+...]`: define
//!   a macro.
//!
//! Other options are ignored, so that test runners can pass their own.

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
    let mut options = Options::default();
    let mut files = Vec::new();
    let mut print = false;
    let mut ast = false;
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        if arg == "-E" {
            print = true;
        } else if arg == "--ast" {
            ast = true;
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
        } else if !arg.starts_with('-') && !arg.starts_with('+') {
            files.push(PathBuf::from(arg));
        }
    }
    if files.is_empty() {
        eprintln!("usage: epitome-sv [-E] [--ast] [-I dir] [-D name[=text]] file.sv...");
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
                ExitCode::SUCCESS
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
