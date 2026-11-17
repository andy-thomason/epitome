//! `epitome-vvp`: takes the command line test runners give Icarus
//! Verilog's `vvp`, loads the VPI modules it names (`-M dir`, `-m name`)
//! and simulates the design file `epitome-iverilog` wrote
//! (`epitome_sv::launch`). The arguments after the design file, such as
//! plusargs, go to the design and to the modules.
//!
//! The exit status is non-zero if `$fatal` was called or the run hit a
//! limit.

use epitome_sv::launch::{DesignFile, VvpArgs};
use std::process::ExitCode;

// The VPI functions, for the modules to find in this executable.
epitome_sv::export_vpi!();

fn main() -> ExitCode {
    // Elaboration recurses as deeply as the design's hierarchy.
    std::thread::Builder::new().stack_size(1 << 30).spawn(run).unwrap().join().unwrap()
}

fn run() -> ExitCode {
    let argv: Vec<String> = std::env::args().collect();
    if argv.iter().skip(1).any(|a| a == "-V" || a == "--version") {
        println!("epitome-vvp {} (epitome-sv)", env!("CARGO_PKG_VERSION"));
        return ExitCode::SUCCESS;
    }
    let args = match VvpArgs::parse(&argv[1..]) {
        Ok(a) => a,
        Err(e) => {
            eprintln!("epitome-vvp: {e}");
            return ExitCode::from(2);
        }
    };
    let df = match std::fs::read_to_string(&args.design).map_err(|e| e.to_string()).and_then(|t| DesignFile::parse(&t)) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("epitome-vvp: {}: {e}", args.design.display());
            return ExitCode::from(2);
        }
    };
    let plusargs: Vec<String> = args.extra.iter().filter(|a| a.starts_with('+')).cloned().collect();
    let setup = epitome_sv::sim::vpi::Setup { libs: args.libs.clone(), argv: argv.clone() };
    let r = epitome_sv::launch::with_design(&df, |d, tree| {
        let opts = epitome_sv::sim::Options { plusargs, max_steps: u64::MAX, zero_init: epitome_sv::sim::zero_init_from_env(), ..Default::default() };
        let mut stdout = std::io::stdout();
        epitome_sv::sim::vpi::simulate(d, tree, &mut stdout, &opts, &setup)
    });
    match r {
        Ok(Ok(outcome)) => {
            for u in &outcome.unsupported {
                eprintln!("epitome-vvp: not simulated: {u}");
            }
            if outcome.failed { ExitCode::FAILURE } else { ExitCode::SUCCESS }
        }
        Ok(Err(e)) | Err(e) => {
            eprint!("epitome-vvp: {e}");
            if !e.ends_with('\n') {
                eprintln!();
            }
            ExitCode::FAILURE
        }
    }
}
