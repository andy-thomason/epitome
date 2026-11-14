//! SystemVerilog to the iCESugar-nano's FPGA (iCE40LP1K-CM36), every
//! stage Epitome's own (`epitome_sv::arch::ice40::flow`):
//!
//!     cargo run --release -p epitome-sv --example ice40_flow -- DESIGN.sv [--top NAME] \
//!         [--pins FILE] [-D NAME=VALUE]... [--out DIR] [--check CYCLES] [--program]
//!
//! It synthesises, maps, places and routes the design and writes
//! `NAME.asc`, `NAME.bin` and `NAME.svg` (the placed and routed design over
//! the die) to `--out` (by default `target/ice40_flow`). `--pins` is a
//! constraints file (`port pin` lines; by default the board's,
//! `sv/arch/ice40-lp1k/flow/nano.pins`). `-D` defines a preprocessor
//! macro. `--check` first runs the source through Epitome's interpreter and the
//! mapped cells on random stimulus, which must agree. `--program` writes
//! the bitstream to the board with `icesprog`.

use epitome_sv::arch::ice40::board::Icesprog;
use epitome_sv::arch::ice40::chipdb::Chipdb;
use epitome_sv::arch::ice40::flow::{Options, build, parse_pins};
use epitome_sv::arch::ice40::icebox::Semantics;
use epitome_sv::arch::ice40::map::mapped_lines;
use epitome_sv::arch::ice40::svg::{SvgOptions, render_svg};
use epitome_sv::arch::ice40::Device;
use epitome_sv::model::Sources;
use epitome_sv::synth::check::{compare, random_stimulus, reference};
use std::path::PathBuf;
use std::process::ExitCode;

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), String> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let arg = |name: &str| args.iter().position(|a| a == name).and_then(|i| args.get(i + 1)).cloned();
    let file = args.iter().find(|a| a.ends_with(".sv") || a.ends_with(".v")).ok_or("no design (.sv) given")?;
    let arch = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("arch/ice40-lp1k");
    let pins_file = arg("--pins").map(PathBuf::from).unwrap_or_else(|| arch.join("flow/nano.pins"));
    let top = arg("--top").unwrap_or_else(|| PathBuf::from(file).file_stem().unwrap().to_string_lossy().into_owned());
    let out = arg("--out").map(PathBuf::from).unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../target/ice40_flow"));
    let mut src = Sources::default();
    src.file(file);
    for (i, a) in args.iter().enumerate() {
        if a == "-D" {
            let d = args.get(i + 1).ok_or("-D needs NAME=VALUE")?;
            let (k, v) = d.split_once('=').unwrap_or((d, "1"));
            src.define(k, v);
        }
    }
    let read = |n: &str| std::fs::read_to_string(arch.join(n)).map_err(|e| format!("{n}: {e}"));
    let d = Device::from_record(&read("record-icepack.txt")?)?;
    let db = Chipdb::from_text(&read("record-chipdb.txt")?)?;
    let sem = Semantics::from_text(&read("record-icebox.txt")?)?;
    let pins = parse_pins(&std::fs::read_to_string(&pins_file).map_err(|e| format!("{}: {e}", pins_file.display()))?)?;

    let t = std::time::Instant::now();
    let netlist = epitome_sv::synth::lower::synthesise(&src, &top)?;
    if let Some(cycles) = arg("--check") {
        let cycles: usize = cycles.parse().map_err(|_| "--check takes a number of cycles")?;
        let st = random_stimulus(&netlist, cycles, 0x5eed)?;
        let want = reference(&src, &top, &netlist, &st)?;
        let m = epitome_sv::arch::ice40::map::map(&netlist)?;
        compare(&want, &mapped_lines(&m, &st)).map_err(|e| format!("the mapped cells differ from the interpreter: {e}"))?;
        eprintln!("check: the mapped cells agree with the interpreter for {cycles} cycles");
    }
    // Only the ports the design has need pins.
    let names: Vec<String> = netlist
        .inputs
        .iter()
        .chain(&netlist.outputs)
        .flat_map(|p| (0..p.width).map(|b| epitome_sv::arch::ice40::place::bit_name(&p.name, p.width, b)))
        .collect();
    let pins: Vec<(String, String)> = pins.into_iter().filter(|(n, _)| names.contains(n)).collect();
    let b = build(&d, &db, &sem, netlist, &Options { package: "cm36".into(), pins, seed: 1 })?;
    eprintln!("{}\n({:.2} s)", b.report(), t.elapsed().as_secs_f64());

    std::fs::create_dir_all(&out).map_err(|e| e.to_string())?;
    let write = |name: String, data: &[u8]| -> Result<PathBuf, String> {
        let p = out.join(name);
        std::fs::write(&p, data).map_err(|e| format!("{}: {e}", p.display()))?;
        Ok(p)
    };
    write(format!("{top}.asc"), b.asc.to_text().as_bytes())?;
    let bin = write(format!("{top}.bin"), &b.bitstream)?;
    let svg = render_svg(&d, &db, &sem, &SvgOptions { overlay: Some(&b.asc), board: None, caption: Some(format!("{top}, placed and routed by Epitome")) });
    let svg_path = write(format!("{top}.svg"), svg.as_bytes())?;
    eprintln!("wrote {} and {}", bin.display(), svg_path.display());
    if args.iter().any(|a| a == "--program") {
        let ip = Icesprog::find().ok_or("icesprog is not built (see sv/arch/README.md)")?;
        ip.program(&b.bitstream)?;
        eprintln!("programmed");
    }
    Ok(())
}
