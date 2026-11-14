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
        if args.iter().any(|a| a == "--ram-modes") {
            return ram_modes(&ip, &serial, &d, &db, &sem, &dir, &args);
        }
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

/// Maps the block RAM's other read modes on the board (see
/// `board::ram_probe` and `board::ram_locate`), appending each finding to
/// `record-ram-modes.txt` as it is made. `--points MODE:ADDR:PIN,...` maps
/// those points instead of the screen and address 0; with `--in-word`,
/// searching only the word the address's low eight bits name.
fn ram_modes(ip: &Icesprog, serial: &PathBuf, d: &Device, db: &Chipdb, sem: &Semantics, dir: &std::path::Path, args: &[String]) -> Result<(), String> {
    use epitome_sv::arch::ice40::icebox::Graph;
    use std::io::Write as _;
    let raw = db.routing.raw();
    let g = Graph::new(&raw);
    let probe_bits: Vec<u8> = (0..64u8).map(|i| 0x20 + i).collect();
    let mut runs = 0usize;
    let mut probe = |mode: usize, addr: usize, pin: usize, words: &[u16]| -> Result<bool, String> {
        let bits = board::ram_probe(d, db, sem, &g, mode, addr, pin, words)?;
        ip.program(&bits)?;
        runs += 1;
        Ok(Serial::open(serial)?.echo(&probe_bits)? == "echo")
    };
    let path = dir.join("record-ram-modes.txt");
    let mut out = std::fs::OpenOptions::new().create(true).append(true).open(&path).map_err(|e| e.to_string())?;
    let mut note = |line: String| -> Result<(), String> {
        eprintln!("  {line}");
        writeln!(out, "{line}").map_err(|e| e.to_string())
    };
    let points: Option<Vec<(usize, usize, usize)>> = args.iter().position(|a| a == "--points").and_then(|i| args.get(i + 1)).map(|s| {
        s.split(',').map(|p| {
            let v: Vec<usize> = p.split(':').map(|x| x.parse().unwrap()).collect();
            (v[0], v[1], v[2])
        }).collect()
    });
    let locate = |probe: &mut dyn FnMut(usize, usize, usize, &[u16]) -> Result<bool, String>, mode: usize, addr: usize, pin: usize| {
        board::ram_locate(&mut |w| probe(mode, addr, pin, w))
    };
    if let Some(points) = points {
        let in_word = args.iter().any(|a| a == "--in-word");
        for (mode, addr, pin) in points {
            let p = if in_word {
                board::ram_locate_in_word(addr & 255, &mut |w| probe(mode, addr, pin, w))?
            } else {
                locate(&mut probe, mode, addr, pin)?
            };
            note(format!("bit {mode} {addr} {pin} {}", p.map_or("none".into(), |p| p.to_string())))?;
        }
        return Ok(());
    }
    // The method on the mode the earlier experiments know: address 1, pin 5
    // of 256x16 is word 1 bit 5, stored bit 21.
    let p = locate(&mut probe, 0, 1, 5)?;
    note(format!("bit 0 1 5 {}", p.map_or("none".into(), |p| p.to_string())))?;
    if p != Some(21) {
        return Err("the method does not find word 1 bit 5 in 256x16".into());
    }
    let ones = vec![0xffffu16; 256];
    for mode in 1..4 {
        let mut used = Vec::new();
        for pin in 0..16 {
            let e = probe(mode, 0, pin, &ones)?;
            note(format!("pin {mode} {pin} {}", if e { "data" } else { "none" }))?;
            if e {
                used.push(pin);
            }
        }
        for pin in used {
            let p = locate(&mut probe, mode, 0, pin)?;
            note(format!("bit {mode} 0 {pin} {}", p.map_or("none".into(), |p| p.to_string())))?;
        }
    }
    Ok(())
}
