//! Renders the iCE40 descriptions as SVG (`epitome_sv::arch::ice40::svg`).
//!
//!     cargo run --release -p epitome-sv --example ice40_svg -- [--part DIR] [--asc FILE | --bin FILE | --experiment NAME] [--out FILE]
//!
//! By default, each part in `PARTS` to `sv/arch/<part>/device.svg`. With a
//! configuration, an `.asc` file, a bitstream or one of the board
//! experiments (`board::experiments`, LP1K), it is laid over the die;
//! `--out` says where the picture goes (by default beside the part's
//! description, named after the configuration).

use epitome_sv::arch::ice40::asc::Asc;
use epitome_sv::arch::ice40::board;
use epitome_sv::arch::ice40::chipdb::Chipdb;
use epitome_sv::arch::ice40::icebox::Semantics;
use epitome_sv::arch::ice40::svg::{SvgOptions, render_svg};
use epitome_sv::arch::ice40::{Device, PARTS};
use std::path::PathBuf;
use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let arg = |name: &str| args.iter().position(|a| a == name).and_then(|i| args.get(i + 1)).cloned();
    let result = (|| -> Result<(), String> {
        let overlaid = arg("--asc").is_some() || arg("--bin").is_some() || arg("--experiment").is_some();
        let only = arg("--part").or_else(|| arg("--experiment").map(|_| "ice40-lp1k".into()));
        for part in PARTS.iter().filter(|p| only.as_ref().is_none_or(|o| *o == p.dir)) {
            let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("arch").join(part.dir);
            let read = |n: &str| std::fs::read_to_string(dir.join(n)).map_err(|e| format!("{n}: {e}"));
            let d = Device::from_record(&read("record-icepack.txt")?)?;
            let db = Chipdb::from_text(&read("record-chipdb.txt")?)?;
            let sem = Semantics::from_text(&read("record-icebox.txt")?)?;
            let results = read("record-board.txt").ok().map(|t| board::results_from_text(&t)).transpose()?;
            let (overlay, caption, name): (Option<Asc>, Option<String>, String) = if let Some(f) = arg("--asc") {
                (Some(Asc::parse(&std::fs::read_to_string(&f).map_err(|e| format!("{f}: {e}"))?)?), Some(format!("with {f} laid over it")), "asc".into())
            } else if let Some(f) = arg("--bin") {
                (Some(d.unpack(&std::fs::read(&f).map_err(|e| format!("{f}: {e}"))?)?), Some(format!("with the bitstream {f} laid over it")), "bin".into())
            } else if let Some(n) = arg("--experiment") {
                let e = board::experiments(&d, &db, &sem)?.into_iter().find(|e| e.name == n).ok_or_else(|| format!("no board experiment {n}"))?;
                (Some(d.unpack(&e.bits)?), Some(format!("with the board experiment {n} laid over it")), n)
            } else {
                (None, None, String::new())
            };
            let opts = SvgOptions { overlay: overlay.as_ref(), board: results.as_ref(), caption };
            let svg = render_svg(&d, &db, &sem, &opts);
            let out = arg("--out").map(PathBuf::from).unwrap_or_else(|| dir.join(if overlaid { format!("device-{name}.svg") } else { "device.svg".into() }));
            std::fs::write(&out, svg).map_err(|e| format!("{}: {e}", out.display()))?;
            eprintln!("wrote {}", out.display());
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
