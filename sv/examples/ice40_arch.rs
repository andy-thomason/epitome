//! The iCE40 architecture description generator, for each part of
//! `epitome_sv::arch::ice40::PARTS` (the iCESugar's UP5K, the
//! iCESugar-nano's LP1K).
//!
//!     cargo run --release -p epitome-sv --example ice40_arch -- [--replay] [--part DIR]
//!
//! Each part has three sources, each with its record in `sv/arch/<part>/`:
//!
//! - the `icepack` experiments (`record-icepack.txt`), rerun when
//!   `icepack` is found (`Icepack::find`; `python3
//!   sv/tools/ice40_asc_check.py --install-only` builds it);
//! - the chip database (`record-chipdb.txt`), made again when the `icebox`
//!   tools are found (`Icebox::find`: the same pinned IceStorm checkout);
//! - the `icebox` experiments (`record-icebox.txt`), rerun with the same
//!   tools (they route with the chip database).
//!
//! Any source that cannot be rerun is replayed from its record, as all are
//! with `--replay`. Then `description.json` is written from the three,
//! and `logic_cell.sv`, a model of the logic cell, from the last. `--part
//! ice40-lp1k` (say) does one part.

use epitome_sv::arch::ice40::board;
use epitome_sv::arch::ice40::chipdb::Chipdb;
use epitome_sv::arch::ice40::icebox::{self, Icebox, Semantics};
use epitome_sv::arch::ice40::{Device, PARTS, Part, describe, experiment, icepack::Icepack};
use std::path::PathBuf;
use std::process::ExitCode;

fn generate(part: &Part, replay: bool) -> Result<(), String> {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("arch").join(part.dir);
    let read = |name: &str| std::fs::read_to_string(dir.join(name)).map_err(|e| format!("{}/{name}: {e}", part.dir));
    let write = |name: &str, text: &str| -> Result<(), String> {
        std::fs::write(dir.join(name), text).map_err(|e| format!("{name}: {e}"))?;
        eprintln!("wrote {}", dir.join(name).display());
        Ok(())
    };
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    eprintln!("{} ({}, {})", part.name, part.package, part.board);
    let d = match Icepack::find().filter(|_| !replay) {
        Some(ip) => {
            eprintln!("icepack experiments with {}", ip.path.display());
            let d = experiment::run(&ip, part.asc, 16, &mut |s| eprintln!("  {s}"))?;
            write("record-icepack.txt", &d.to_record())?;
            d
        }
        None => Device::from_record(&read("record-icepack.txt")?)?,
    };
    let ib = Icebox::find().filter(|_| !replay);
    let (db, raw) = match &ib {
        Some(ib) => {
            eprintln!("chip database from icebox_chipdb {}", part.chipdb_flag);
            let (db, raw) = Chipdb::parse(&ib.chipdb(part)?)?;
            write("record-chipdb.txt", &db.to_text())?;
            (db, Some(raw))
        }
        None => (Chipdb::from_text(&read("record-chipdb.txt")?)?, None),
    };
    let sem = match (&ib, &raw) {
        (Some(ib), Some(raw)) => {
            eprintln!("icebox experiments with {}", ib.dir.display());
            let sem = icebox::run(ib, &d, &db, raw, &mut |s| eprintln!("  {s}"))?;
            write("record-icebox.txt", &sem.to_text())?;
            sem
        }
        _ => Semantics::from_text(&read("record-icebox.txt")?)?,
    };
    let failed: Vec<_> = describe::checks(&d, Some(&db), Some(&sem)).into_iter().filter(|c| !c.1).collect();
    for (what, _, detail) in &failed {
        eprintln!("check failed: {what} {detail}");
    }
    // The board experiments, where a board was tried (ice40_board).
    let board = read("record-board.txt").ok().map(|t| board::results_from_text(&t)).transpose()?;
    let points = read("record-ram-modes.txt").map(|t| board::ram_points_from_text(&t)).unwrap_or_default();
    write("description.json", &describe::to_json(&d, Some(&db), Some(&sem), board.as_ref().map(|b| (b, &points[..]))))?;
    write("logic_cell.sv", &describe::logic_cell_sv(part, &sem))?;
    if failed.is_empty() { Ok(()) } else { Err(format!("{}: {} checks failed", part.name, failed.len())) }
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let replay = args.iter().any(|a| a == "--replay");
    let only = args.iter().position(|a| a == "--part").and_then(|i| args.get(i + 1));
    let mut ok = true;
    for part in PARTS.iter().filter(|p| only.is_none_or(|o| o == p.dir)) {
        if let Err(e) = generate(part, replay) {
            eprintln!("error: {e}");
            ok = false;
        }
    }
    if ok { ExitCode::SUCCESS } else { ExitCode::FAILURE }
}
