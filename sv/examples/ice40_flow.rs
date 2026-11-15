//! SystemVerilog to an iCE40 on one of MuseLab's boards, every stage
//! Epitome's own (`epitome_sv::arch::ice40::flow`):
//!
//!     cargo run --release -p epitome-sv --example ice40_flow -- DESIGN.sv [--top NAME] \
//!         [--board nano|icesugar] [--part ice40-lp1k|ice40-up5k] [--package PKG] \
//!         [--pins FILE] [-D NAME=VALUE]... [--out DIR] [--check CYCLES] [--program]
//!
//! `--board` picks the board (`board::BOARDS`): `nano`, the iCESugar-nano's
//! iCE40LP1K-CM36 (the default), or `icesugar`, the iCESugar's
//! iCE40UP5K-SG48; `--part` picks it by its part instead. It synthesises,
//! maps, places and routes the design and writes `NAME.asc`, `NAME.bin`
//! and `NAME.svg` (the placed and routed design over the die) to `--out`
//! (by default `target/ice40_flow`). `--package` and `--pins` (a
//! constraints file of `port pin` lines) are by default the board's, such
//! as `sv/arch/ice40-up5k/flow/icesugar.pins`. `-D` defines a preprocessor
//! macro. `--check` first runs the source through Epitome's interpreter and the
//! mapped cells on random stimulus, which must agree. `--program` writes
//! the bitstream to the board with `icesprog`, once `icesprog -p` has
//! reported the board chosen.

use epitome_sv::arch::ice40::board::{BOARDS, Icesprog, board};
use epitome_sv::arch::ice40::flow::{Options, build, parse_pins};
use epitome_sv::arch::ice40::map::mapped_lines;
use epitome_sv::arch::ice40::svg::{SvgOptions, render_svg};
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
    let which = arg("--board").or_else(|| arg("--part")).unwrap_or_else(|| "nano".into());
    let brd = board(&which).ok_or_else(|| {
        let names: Vec<String> = BOARDS.iter().map(|b| format!("{} ({})", b.name, b.part)).collect();
        format!("no board or part {which}: there are {}", names.join(", "))
    })?;
    let arch = brd.arch();
    let package = arg("--package").unwrap_or_else(|| brd.package.into());
    let pins_file = arg("--pins").map(PathBuf::from).unwrap_or_else(|| brd.pins_file());
    let top = arg("--top").unwrap_or_else(|| PathBuf::from(file).file_stem().unwrap().to_string_lossy().into_owned());
    let out = arg("--out")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../target/ice40_flow"));
    let mut src = Sources::default();
    src.file(file);
    for (i, a) in args.iter().enumerate() {
        if a == "-D" {
            let d = args.get(i + 1).ok_or("-D needs NAME=VALUE")?;
            let (k, v) = d.split_once('=').unwrap_or((d, "1"));
            src.define(k, v);
        }
    }
    let (d, db, sem) = brd.records()?;
    let pins = parse_pins(&std::fs::read_to_string(&pins_file).map_err(|e| format!("{}: {e}", pins_file.display()))?)?;

    let t = std::time::Instant::now();
    let netlist = epitome_sv::synth::lower::synthesise(&src, &top)?;
    if let Some(cycles) = arg("--check") {
        let cycles: usize = cycles.parse().map_err(|_| "--check takes a number of cycles")?;
        let st = random_stimulus(&netlist, cycles, 0x5eed)?;
        let want = reference(&src, &top, &netlist, &st)?;
        let m = epitome_sv::arch::ice40::map::map(&netlist)?;
        compare(&want, &mapped_lines(&m, &st))
            .map_err(|e| format!("the mapped cells differ from the interpreter: {e}"))?;
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
    let b = build(
        &d,
        &db,
        &sem,
        netlist,
        &Options { package, pins, seed: 1, fabric_clocks: args.iter().any(|a| a == "--fabric-clocks") },
    )?;
    eprintln!("{}\n({:.2} s)", b.report(), t.elapsed().as_secs_f64());
    let model = std::fs::read_to_string(arch.join("record-timing.txt")).ok().map_or_else(
        || Ok(epitome_sv::arch::ice40::timing::Model::guess()),
        |t| epitome_sv::arch::ice40::timing::Model::from_record(&t),
    )?;
    eprintln!("{}", epitome_sv::arch::ice40::timing::analyse(&b, &d, &db, &model).text(&model));

    std::fs::create_dir_all(&out).map_err(|e| e.to_string())?;
    let write = |name: String, data: &[u8]| -> Result<PathBuf, String> {
        let p = out.join(name);
        std::fs::write(&p, data).map_err(|e| format!("{}: {e}", p.display()))?;
        Ok(p)
    };
    write(format!("{top}.asc"), b.asc.to_text().as_bytes())?;
    let bin = write(format!("{top}.bin"), &b.bitstream)?;
    let rams: Vec<(usize, usize)> = b.placement.rams.iter().flatten().copied().collect();
    let svg = render_svg(
        &d,
        &db,
        &sem,
        &SvgOptions {
            overlay: Some(&b.asc),
            board: None,
            caption: Some(format!("{top}, placed and routed by Epitome")),
            rams: Some(&rams),
        },
    );
    let svg_path = write(format!("{top}.svg"), svg.as_bytes())?;
    eprintln!("wrote {} and {}", bin.display(), svg_path.display());
    if args.iter().any(|a| a == "--program") {
        let ip = Icesprog::find().ok_or("icesprog is not built (see sv/arch/README.md)")?;
        ip.expect(brd)?;
        ip.program(&b.bitstream)?;
        eprintln!("programmed");
    }
    Ok(())
}
