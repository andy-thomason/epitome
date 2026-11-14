//! Runs the board experiments (`epitome_sv::arch::ice40::board`) on an
//! iCESugar-nano, and records what it did in
//! `sv/arch/ice40-lp1k/record-board.txt`.
//!
//!     cargo run --release -p epitome-sv --example ice40_board -- [--install] [--only NAME]
//!
//! It reprograms the board's flash for each experiment. `--install` builds
//! `icesprog` first if it is not built (see `Icesprog::install`).

use epitome_sv::arch::ice40::board::{self, Icesprog, Serial};
use epitome_sv::arch::ice40::chipdb::Chipdb;
use epitome_sv::arch::ice40::icebox::Semantics;
use epitome_sv::arch::ice40::{Device, part};
use std::path::PathBuf;
use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let only = args.iter().position(|a| a == "--only").and_then(|i| args.get(i + 1));
    let result = (|| -> Result<(), String> {
        let p = part("1k").unwrap();
        let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("arch").join(p.dir);
        let read = |n: &str| std::fs::read_to_string(dir.join(n)).map_err(|e| format!("{n}: {e}"));
        let d = Device::from_record(&read("record-icepack.txt")?)?;
        let db = Chipdb::from_text(&read("record-chipdb.txt")?)?;
        let sem = Semantics::from_text(&read("record-icebox.txt")?)?;
        let ip = if args.iter().any(|a| a == "--install") { Icesprog::install()? } else { Icesprog::find().ok_or("no icesprog: run with --install")? };
        let (board, flash) = ip.probe()?;
        if board != "iCESugar-Nano" {
            return Err(format!("the board is an {board}, not an iCESugar-Nano"));
        }
        let serial = Serial::find().ok_or("no iCELink serial port")?;
        eprintln!("{board}, flash {flash}, serial {}", serial.display());
        let exps: Vec<_> = board::experiments(&d, &db, &sem)?.into_iter().filter(|e| only.is_none_or(|o| *o == e.name)).collect();
        let res = board::run(&ip, &serial, &exps, &mut |s| eprintln!("  {s}"))?;
        if only.is_none() {
            std::fs::write(dir.join("record-board.txt"), board::results_text(&board, &flash, &res)).map_err(|e| e.to_string())?;
            eprintln!("wrote {}", dir.join("record-board.txt").display());
        }
        Ok(())
    })();
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::FAILURE
        }
    }
}
