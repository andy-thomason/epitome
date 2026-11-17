//! `epitome-iverilog`: takes the command line test runners give Icarus
//! Verilog's `iverilog` and writes a design file for `epitome-vvp`
//! (`epitome_sv::launch`). The sources are parsed now, so that syntax
//! errors show at the compile step; elaboration waits for the run.

use epitome_sv::launch::DesignFile;
use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.iter().any(|a| a == "-V" || a == "--version") {
        println!("epitome-iverilog {} (epitome-sv)", env!("CARGO_PKG_VERSION"));
        return ExitCode::SUCCESS;
    }
    let (d, out) = match DesignFile::from_iverilog_args(&args) {
        Ok(x) => x,
        Err(e) => {
            eprintln!("epitome-iverilog: {e}");
            return ExitCode::from(2);
        }
    };
    let check = std::thread::Builder::new().stack_size(1 << 30).spawn({
        let d = d.clone();
        move || -> Result<(), String> {
            let map = epitome_sv::source::SourceMap::new();
            let pp = epitome_sv::preprocess::preprocess_files(&map, &d.files, &d.options()?)?;
            epitome_sv::parse::parse(&pp).map_err(|e| map.render(&e))?;
            d.param_values()?;
            Ok(())
        }
    });
    if let Err(e) = check.unwrap().join().unwrap() {
        eprint!("{e}");
        if !e.ends_with('\n') {
            eprintln!();
        }
        return ExitCode::FAILURE;
    }
    if let Err(e) = std::fs::write(&out, d.write()) {
        eprintln!("epitome-iverilog: {}: {e}", out.display());
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}
