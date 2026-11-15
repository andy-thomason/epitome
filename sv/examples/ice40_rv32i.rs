//! The iCESugar's RV32I computer (`sv/arch/ice40-up5k/flow/rv32i.sv`):
//! assembles its program, `rv32i_demo.s`, into `rv32i_demo.svh`, and runs
//! it on the emulator with the lines given (or a few of its own; the first
//! starts it), printing what the board should send.
//!
//!     cargo run --release -p epitome-sv --example ice40_rv32i [-- LINE...]

use epitome_sv::arch::ice40::rv32i::{Emulator, assemble};
use std::path::PathBuf;
use std::process::ExitCode;

fn main() -> ExitCode {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("arch/ice40-up5k/flow");
    let run = || -> Result<(), String> {
        let src = std::fs::read_to_string(dir.join("rv32i_demo.s")).map_err(|e| e.to_string())?;
        let p = assemble(&src)?;
        let svh = dir.join("rv32i_demo.svh");
        std::fs::write(&svh, p.to_sv("rv32i_demo.s")).map_err(|e| e.to_string())?;
        let used = p.labels.get("sieve").copied().unwrap_or(0);
        eprintln!("{} bytes ({} of code and data before the sieve); wrote {}", p.image.len(), used, svh.display());
        let args: Vec<String> = std::env::args().skip(1).collect();
        let lines: Vec<&[u8]> = if args.is_empty() {
            vec![b"", b"Hello, iCESugar", b""]
        } else {
            args.iter().map(|a| a.as_bytes()).collect()
        };
        let mut e = Emulator::new(&p);
        e.rx.push_back(b'\r');
        e.run(50_000_000)?;
        eprintln!("{} instructions to the first prompt", e.steps);
        let out = Emulator::session(&p, &lines)?;
        for o in out {
            print!("{}", String::from_utf8_lossy(&o));
        }
        println!();
        Ok(())
    };
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::FAILURE
        }
    }
}
