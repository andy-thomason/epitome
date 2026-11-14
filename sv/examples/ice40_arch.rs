//! The iCE40UP5K architecture description generator.
//!
//!     cargo run --release -p epitome-sv --example ice40_arch -- [--replay] [--chipdb FILE]
//!
//! It has three sources, each with its record in `sv/arch/ice40-up5k/`:
//!
//! - the `icepack` experiments (`record-icepack.txt`), rerun when
//!   `icepack` is found (`Icepack::find`; `python3
//!   sv/tools/ice40_asc_check.py --install-only` builds it);
//! - the chip database (`record-chipdb.txt`), read again when `--chipdb`
//!   names `chipdb-5k.txt` or `$CHIPDB` does;
//! - the `icebox` experiments (`record-icebox.txt`), rerun when the
//!   `icebox` tools are found (`Icebox::find`: the same pinned IceStorm
//!   checkout) and the chip database was read (they route with it).
//!
//! Any source that cannot be rerun is replayed from its record, as all are
//! with `--replay`. Then `description.json` is written from the three,
//! and `logic_cell.sv`, a model of the logic cell, from the last.

use epitome_sv::arch::ice40::chipdb::Chipdb;
use epitome_sv::arch::ice40::icebox::{self, Icebox, Semantics};
use epitome_sv::arch::ice40::{Device, describe, experiment, icepack::Icepack};
use std::path::PathBuf;
use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let replay = args.iter().any(|a| a == "--replay");
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("arch/ice40-up5k");
    let chipdb_path = args.iter().position(|a| a == "--chipdb").and_then(|i| args.get(i + 1).cloned()).or_else(|| std::env::var("CHIPDB").ok());
    let read = |name: &str| std::fs::read_to_string(dir.join(name)).map_err(|e| format!("{name}: {e}"));
    let write = |name: &str, text: &str| -> Result<(), String> {
        std::fs::write(dir.join(name), text).map_err(|e| format!("{name}: {e}"))?;
        eprintln!("wrote {}", dir.join(name).display());
        Ok(())
    };
    let result = (|| -> Result<(), String> {
        std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        let d = match Icepack::find().filter(|_| !replay) {
            Some(ip) => {
                eprintln!("icepack experiments with {}", ip.path.display());
                let d = experiment::run(&ip, 16, &mut |s| eprintln!("  {s}"))?;
                write("record-icepack.txt", &d.to_record())?;
                d
            }
            None => Device::from_record(&read("record-icepack.txt")?)?,
        };
        let (db, raw) = match chipdb_path.filter(|_| !replay) {
            Some(path) => {
                eprintln!("reading {path}");
                let (db, raw) = Chipdb::parse(&std::fs::read_to_string(&path).map_err(|e| format!("{path}: {e}"))?)?;
                write("record-chipdb.txt", &db.to_text())?;
                (db, Some(raw))
            }
            None => (Chipdb::from_text(&read("record-chipdb.txt")?)?, None),
        };
        let sem = match (Icebox::find().filter(|_| !replay), &raw) {
            (Some(ib), Some(raw)) => {
                eprintln!("icebox experiments with {}", ib.dir.display());
                let sem = icebox::run(&ib, &d, &db, raw, &mut |s| eprintln!("  {s}"))?;
                write("record-icebox.txt", &sem.to_text())?;
                sem
            }
            _ => Semantics::from_text(&read("record-icebox.txt")?)?,
        };
        let failed: Vec<_> = describe::checks(&d, Some(&db), Some(&sem)).into_iter().filter(|c| !c.1).collect();
        for (what, _, detail) in &failed {
            eprintln!("check failed: {what} {detail}");
        }
        write("description.json", &describe::to_json(&d, Some(&db), Some(&sem)))?;
        write("logic_cell.sv", &describe::logic_cell_sv(&sem))?;
        if failed.is_empty() { Ok(()) } else { Err(format!("{} checks failed", failed.len())) }
    })();
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::FAILURE
        }
    }
}
