//! The FPGA flow (`epitome_sv::synth`, `epitome_sv::arch::ice40::flow`):
//! each stage checked on small designs, against Epitome's interpreter on
//! random stimulus. The board tests run only with `ICE40_BOARD=1` and
//! the iCESugar-nano plugged in.

use epitome_sv::arch::ice40::map::{map, mapped_lines};
use epitome_sv::model::Sources;
use epitome_sv::synth::check::{compare, netlist_lines, random_stimulus, reference};
use epitome_sv::synth::lower::synthesise;

fn src(text: &str) -> Sources {
    let mut s = Sources::default();
    s.text("design.sv", text);
    s
}

pub const MIXED: &str = r#"
module sub(input logic a, input logic [3:0] b, output logic y);
  assign y = a ^ b[2];
endmodule
module top(input logic clk, input logic rst, input logic [3:0] a, input logic [7:0] b,
           output logic [3:0] q, output logic lt, output logic y, output logic [7:0] s, output logic [7:0] m);
  logic [3:0] cnt = 0;
  always_ff @(posedge clk or posedge rst)
    if (rst) cnt <= 4'd3;
    else if (a[0]) cnt <= cnt + 1;
  assign q = cnt;
  always_comb begin
    lt = 0;
    case (a[2:1])
      2'd0: lt = a < cnt;
      2'd1: lt = cnt[a[3:2]];
      default: lt = &a;
    endcase
  end
  sub u(.a(a[1]), .b(cnt), .y(y));
  logic [7:0] acc;
  always_ff @(posedge clk)
    if (rst) acc <= 0;
    else acc <= acc - b + {4'b0, a};
  assign s = acc;
  always_comb begin
    m = b;
    for (int i = 0; i < 4; i++)
      if (a[i]) m[i +: 2] = ~b[i*2 +: 2];
    m = m >> a[1:0];
  end
endmodule
"#;

pub const COUNTER: &str = r#"
module counter(input logic clk, output logic led, output logic [7:0] low);
  logic [23:0] n = 0;
  always_ff @(posedge clk) n <= n + 1;
  assign led = n[23];
  assign low = n[7:0];
endmodule
"#;

pub const ALU: &str = r#"
module alu(input logic clk, input logic [1:0] op, input logic [7:0] a, input logic [7:0] b, input logic en,
           output logic [7:0] y, output logic lt, output logic ge, output logic eq);
  always_ff @(posedge clk)
    if (en)
      case (op)
        2'd0: y <= a + b;
        2'd1: y <= a - b;
        2'd2: y <= a & ~b;
        default: y <= {a[3:0], b[7:4]};
      endcase
  assign lt = $signed(a) < $signed(b);
  assign ge = a >= b;
  assign eq = a == y;
endmodule
"#;

pub const RAMS: &str = r#"
module rams(input logic clk, input logic we, input logic re, input logic [3:0] wa, input logic [3:0] ra,
            input logic [15:0] wd, output logic [15:0] q, output logic [7:0] q2, output logic [5:0] q3);
  logic [15:0] mem [0:15];
  always_ff @(posedge clk) begin
    if (we) mem[wa] <= wd;
    if (re) q <= mem[ra];
  end
  // A second read port, unconditional.
  logic [15:0] m2;
  always_ff @(posedge clk) m2 <= mem[wa ^ ra];
  assign q2 = m2[7:0] ^ m2[15:8];
  // Indexed from 16, written under a case.
  logic [5:0] hi [16:31];
  always_ff @(posedge clk) begin
    case (wd[1:0])
      2'd0: hi[{1'b1, wa}] <= wd[7:2];
      2'd1: if (we) hi[{1'b1, ra}] <= ~wd[7:2];
      default: ;
    endcase
    q3 <= hi[{1'b1, ra}];
  end
endmodule
"#;

pub const TABLES: &str = r#"
module tables(input logic clk, input logic [3:0] a, output logic [7:0] y, output logic [7:0] z, output logic [3:0] p);
  logic [7:0] t [16];
  initial for (int i = 0; i < 16; i++) t[i] = i * 7 + 3;
  assign y = t[a];
  always_ff @(posedge clk) z <= t[~a];
  logic [3:0] pat [0:3] = '{4'h1, 4'h2, 4'h4, 4'h8};
  assign p = pat[a[1:0]] ^ pat[a[3:2]];
endmodule
"#;

/// Synthesises `text` and checks the netlist alone against the
/// interpreter.
fn check_netlist(text: &str, top: &str, cycles: usize) -> epitome_sv::synth::Netlist {
    let s = src(text);
    let n = synthesise(&s, top).unwrap_or_else(|e| panic!("{e}"));
    eprintln!("{top}: {}", n.stats());
    let st = random_stimulus(&n, cycles, 0x1234_5678).unwrap();
    let want = reference(&s, top, &n, &st).unwrap();
    compare(&want, &netlist_lines(&n, &st)).unwrap_or_else(|e| panic!("netlist: {e}"));
    n
}

#[test]
fn memories_match_interpreter() {
    let n = check_netlist(RAMS, "rams", 400);
    assert_eq!(n.mems.len(), 2);
    assert_eq!(n.mems[0].reads.len(), 2);
    let n = check_netlist(TABLES, "tables", 200);
    // t read by a register is a block RAM read port; read by `assign`, and
    // pat, logic.
    assert_eq!(n.mems.len(), 1);
}

/// Synthesises `text`, checks the netlist and its mapping against the
/// interpreter on `cycles` cycles of random stimulus.
fn check_flow(text: &str, top: &str, cycles: usize) -> epitome_sv::arch::ice40::map::Mapped {
    let s = src(text);
    let n = synthesise(&s, top).unwrap_or_else(|e| panic!("{e}"));
    eprintln!("{top}: {}", n.stats());
    let st = random_stimulus(&n, cycles, 0x1234_5678).unwrap();
    let want = reference(&s, top, &n, &st).unwrap();
    compare(&want, &netlist_lines(&n, &st)).unwrap_or_else(|e| panic!("netlist: {e}"));
    let m = map(&n).unwrap_or_else(|e| panic!("{e}"));
    eprintln!("{top}: {}", m.stats());
    compare(&want, &mapped_lines(&m, &st)).unwrap_or_else(|e| panic!("mapped: {e}"));
    m
}

#[test]
fn flow_matches_interpreter() {
    check_flow(MIXED, "top", 300);
    check_flow(ALU, "alu", 300);
}

#[test]
fn counter_on_the_carry_chain() {
    let m = check_flow(COUNTER, "counter", 300);
    // A cell for each bit, its sum in the LUT, its carry in the chain.
    assert_eq!(m.chains.len(), 1);
    assert!(m.lcs.len() <= 26, "{}", m.stats());
}

/// What synthesis does not do yet is an error at the source.
#[test]
fn synth_rejects() {
    let cases = [
        ("module t(input logic a, b, output logic y);\n always_comb if (a) y = b;\nendmodule\n", "latch", 2),
        (
            "module t(input logic c1, c2, d, output logic y, z);\n always_ff @(posedge c1) y <= d;\n always_ff @(posedge c2) z <= d;\nendmodule\n",
            "one clock",
            0,
        ),
        (
            "module t(input logic clk, input logic [9:0] a, input logic [7:0] d, output logic [7:0] y);\n logic [7:0] m [1024];\n always_ff @(posedge clk) m[a] <= d;\n assign y = m[a];\nendmodule\n",
            "too big to build from flip-flops",
            4,
        ),
        (
            "module t(input logic [1:0] a, input logic d, output logic y);\n logic m [4];\n always_comb m[a] = d;\n assign y = m[0];\nendmodule\n",
            "only on a clock edge",
            3,
        ),
        (
            "module t(input logic a, output wire y);\n assign y = a;\n assign y = ~a;\nendmodule\n",
            "more than one driver",
            3,
        ),
        (
            "module t(input logic a, output logic y);\n logic p, q;\n assign p = q ^ a;\n assign q = p;\n assign y = q;\nendmodule\n",
            "combinational loop",
            0,
        ),
        (
            "module t(input logic [7:0] a, b, output logic [7:0] y);\n assign y = a / b;\nendmodule\n",
            "not synthesised",
            2,
        ),
    ];
    for (text, what, line) in cases {
        let e = synthesise(&src(text), "t").unwrap_err();
        assert!(e.contains(what), "{e}");
        if line > 0 {
            assert!(e.contains(&format!("design.sv:{line}:")), "{e}");
        }
    }
}

/// Flip-flops on different enables go to different tiles; a chain longer
/// than a tile carries on in the tile above.
#[test]
fn placement_is_legal() {
    let text = r#"
module t(input logic clk, input logic [3:0] en, input logic [11:0] d, output logic [11:0] q, output logic [19:0] n);
  always_ff @(posedge clk)
    for (int i = 0; i < 12; i++)
      if (en[i % 4]) q[i] <= d[i];
  logic [19:0] c = 0;
  always_ff @(posedge clk) if (en == 4'hf) c <= c + d[3:0];
  assign n = c;
endmodule
"#;
    check_flow(text, "t", 200);
    let b = build_tq144(text, "t");
    // Four enables: at least four tiles of flip-flops besides the chain's.
    let mut tiles: Vec<(usize, usize)> = b
        .mapped
        .lcs
        .iter()
        .enumerate()
        .filter(|(_, l)| l.ff.is_some())
        .map(|(i, _)| (b.placement.at[i].0, b.placement.at[i].1))
        .collect();
    tiles.sort();
    tiles.dedup();
    assert!(tiles.len() >= 4, "{tiles:?}");
    assert!(b.mapped.chains.iter().any(|c| c.lcs.len() > 8));
    epitome_sv::arch::ice40::place::check(&records().1, &b.mapped, &b.placement).unwrap();
}

// ---- the flow on the board's designs ----

use epitome_sv::arch::ice40::Device;
use epitome_sv::arch::ice40::chipdb::Chipdb;
use epitome_sv::arch::ice40::flow::{Built, Options, build, parse_pins};
use epitome_sv::arch::ice40::icebox::Semantics;
use std::path::PathBuf;

fn arch() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("arch/ice40-lp1k")
}

fn records() -> (Device, Chipdb, Semantics) {
    let read = |n: &str| std::fs::read_to_string(arch().join(n)).unwrap();
    (
        Device::from_record(&read("record-icepack.txt")).unwrap(),
        Chipdb::from_text(&read("record-chipdb.txt")).unwrap(),
        Semantics::from_text(&read("record-icebox.txt")).unwrap(),
    )
}

/// Builds `flow/NAME.sv` for the board, checking the mapped cells against
/// the interpreter first.
fn build_design(name: &str, defines: &[&str]) -> Built {
    let (d, db, sem) = records();
    let mut s = Sources::default();
    s.file(arch().join(format!("flow/{name}.sv")));
    for &k in defines {
        let (k, v) = k.split_once('=').unwrap_or((k, "1"));
        s.define(k, v);
    }
    let n = synthesise(&s, name).unwrap_or_else(|e| panic!("{e}"));
    // Enough cycles for a RAM delay line to come round.
    let depth = n.mems.iter().map(|m| m.depth as usize).max().unwrap_or(0);
    let st = random_stimulus(&n, if depth == 0 { 200 } else { depth + 300 }, 7).unwrap();
    let want = reference(&s, name, &n, &st).unwrap();
    let m = map(&n).unwrap();
    compare(&want, &mapped_lines(&m, &st)).unwrap_or_else(|e| panic!("{name}: {e}"));
    let pins = parse_pins(&std::fs::read_to_string(arch().join("flow/nano.pins")).unwrap()).unwrap();
    let ports: Vec<&str> = n.inputs.iter().chain(&n.outputs).map(|p| p.name.as_str()).collect();
    let pins = pins.into_iter().filter(|(p, _)| ports.contains(&p.as_str())).collect();
    let b = build(&d, &db, &sem, n, &Options { package: "cm36".into(), pins, seed: 1 })
        .unwrap_or_else(|e| panic!("{name}: {e}"));
    eprintln!("{name}:\n{}", b.report());
    b
}

#[test]
fn board_designs_build() {
    let echo = build_design("echo", &[]);
    assert_eq!(echo.mapped.lcs.len(), 0);
    let inv = build_design("echo", &["INVERT"]);
    assert_eq!(inv.mapped.lcs.len(), 1);
    let delay = build_design("delay", &[]);
    assert_eq!(delay.mapped.lcs.iter().filter(|l| l.ff.is_some()).count(), 4);
    assert_eq!(delay.clock_glb, Some(1));
    let blink = build_design("blink", &[]);
    assert_eq!(blink.mapped.chains.len(), 1);
    build_design("hello", &[]);
    for w in ["16", "8", "4", "2", "1"] {
        let b = build_design("ramdelay", &[&format!("WIDTH={w}")]);
        assert_eq!(b.mapped.rams.len(), 1);
    }
    let b = build_design("hello_ram", &[]);
    assert_eq!(b.mapped.rams.len(), 1);
    // The delay line's read never meets its write: no delayed write.
    let b = build_design("ramdelay", &["WIDTH=8"]);
    assert!(!b.mapped.lcs.iter().any(|l| l.name.contains("bypass")), "{}", b.mapped.stats());
    // Here they always meet.
    let b = build_design("rdw", &[]);
    assert!(b.mapped.lcs.iter().any(|l| l.name.contains("bypass")), "{}", b.mapped.stats());
    // Deeper than a block: banks.
    let b = build_design("ramdelay", &["WIDTH=1", "DEPTH=8192"]);
    assert_eq!(b.mapped.rams[0].blocks, 1);
    assert_eq!(b.mapped.rams.len(), 4);
    let b = build_design("ramdelay", &["WIDTH=4", "DEPTH=16384"]);
    assert_eq!(b.mapped.rams.iter().map(|r| r.blocks).sum::<u32>(), 16);
}

/// Programs the board with each design and watches the serial port; only
/// with `ICE40_BOARD=1` (it reprograms an iCESugar-nano).
#[test]
fn board_live() {
    use epitome_sv::arch::ice40::board::{Icesprog, Serial};
    use std::time::{Duration, Instant};
    if std::env::var_os("ICE40_BOARD").is_none() {
        eprintln!("skipped: set ICE40_BOARD=1 to reprogram an iCESugar-nano");
        return;
    }
    let ip = Icesprog::find().expect("icesprog");
    let port = Serial::find().expect("serial port");
    let probe: Vec<u8> = (0..64u8).map(|i| 0x20 + i).collect();
    let run = |b: &Built| -> String {
        ip.program(&b.bitstream).unwrap();
        Serial::open(&port).unwrap().echo(&probe).unwrap()
    };
    // (a) combinational: rx routed to tx echoes; ~rx from a LUT does not.
    assert_eq!(run(&build_design("echo", &[])), "echo", "(a) rx to tx");
    assert_ne!(run(&build_design("echo", &["INVERT"])), "echo", "(a) ~rx to tx");
    // (b) registered: rx through four flip-flops on the 12 MHz clock.
    assert_eq!(run(&build_design("delay", &[])), "echo", "(b) delayed rx");
    // (c) the counter: echoes come and go with its top bit, every
    // 2^23 / 12 MHz = 0.70 s.
    let b = build_design("blink", &[]);
    ip.program(&b.bitstream).unwrap();
    let mut s = Serial::open(&port).unwrap();
    s.read_quiet(200);
    let t0 = Instant::now();
    let mut samples = Vec::new();
    while t0.elapsed() < Duration::from_secs(5) {
        s.send(b"U").unwrap();
        let back = s.read_quiet(20);
        let seen = match back.as_slice() {
            b"U" => Some(true),
            [] => Some(false),
            _ => None,
        };
        samples.push((t0.elapsed().as_secs_f64(), seen));
    }
    let known: Vec<(f64, bool)> = samples.iter().filter_map(|&(t, s)| s.map(|s| (t, s))).collect();
    let edges: Vec<f64> = known.windows(2).filter(|w| w[0].1 != w[1].1).map(|w| (w[0].0 + w[1].0) / 2.0).collect();
    eprintln!("(c) {} samples, echo changes at {edges:.2?}", samples.len());
    assert!(edges.len() >= 4, "(c) too few changes: {edges:?}");
    let half = (edges[edges.len() - 1] - edges[0]) / (edges.len() - 1) as f64;
    assert!((half - 0.699).abs() < 0.1, "(c) half period {half:.3} s, not 0.70 s");
    // (d) the UART transmitter: the Mac reads its string.
    let b = build_design("hello", &[]);
    ip.program(&b.bitstream).unwrap();
    let mut s = Serial::open(&port).unwrap();
    s.read_for(100);
    let got = s.read_for(500);
    let text = String::from_utf8_lossy(&got);
    let whole = text.matches("Hello from Epitome\r\n").count();
    eprintln!("(d) {} bytes, {whole} whole strings: {:?}...", got.len(), &text[..text.len().min(60)]);
    assert!(whole >= 100, "(d) {whole} strings in 0.5 s");
    // (e) rx through a block RAM in each of its modes.
    for w in ["16", "8", "4", "2", "1"] {
        let seen = run(&build_design("ramdelay", &[&format!("WIDTH={w}")]));
        eprintln!("(e) through a RAM {w} bits wide: {seen}");
        assert_eq!(seen, "echo", "(e) WIDTH={w}");
    }
    // (g) deeper than a block: banks, up to the whole chip's RAM.
    for (w, depth) in [("1", "8192"), ("4", "16384")] {
        let seen = run(&build_design("ramdelay", &[&format!("WIDTH={w}"), &format!("DEPTH={depth}")]));
        eprintln!("(g) through {depth} words of {w} bits: {seen}");
        assert_eq!(seen, "echo", "(g) WIDTH={w} DEPTH={depth}");
    }
    // (h) a read of the word being written on the same edge sees the old
    // word, as SystemVerilog says, in each mode.
    for w in ["16", "8", "4", "2"] {
        let seen = run(&build_design("rdw", &[&format!("W={w}")]));
        eprintln!("(h) read while writing, {w} bits wide: {seen}");
        assert_eq!(seen, "echo", "(h) W={w}");
    }
    // (f) the string from a RAM's initial contents.
    let b = build_design("hello_ram", &[]);
    ip.program(&b.bitstream).unwrap();
    let mut s = Serial::open(&port).unwrap();
    s.read_for(100);
    let got = s.read_for(500);
    let text = String::from_utf8_lossy(&got);
    let whole = text.matches("Hello from Epitome\r\n").count();
    eprintln!("(f) {} bytes, {whole} whole strings: {:?}...", got.len(), &text[..text.len().min(60)]);
    assert!(whole >= 100, "(f) {whole} strings in 0.5 s");
}

/// A `case` table becomes a decision tree, and few LUTs: hello's 20
/// characters of 7 bits.
#[test]
fn case_table_is_small() {
    let text = std::fs::read_to_string(arch().join("flow/hello.sv")).unwrap();
    let a = text.find("  always_comb").unwrap();
    let b = text.find("endcase").unwrap() + 8;
    let rom = format!("module rom(input logic [4:0] idx, output logic [7:0] ch);\n{}\nendmodule\n", &text[a..b]);
    let n = synthesise(&src(&rom), "rom").unwrap();
    let m = map(&n).unwrap();
    eprintln!("{}\n{}", n.stats(), m.stats());
    assert!(m.lcs.len() <= 20, "{}", m.stats());
}

/// Builds `top` on the LP1K in its TQ144 package (the CM36 has too few
/// pins for most test designs), the clock on a pin with a global buffer,
/// the other ports on pins in order.
fn build_tq144(text: &str, top: &str) -> Built {
    let (d, db, sem) = records();
    let n = synthesise(&src(text), top).unwrap();
    let pkg = &db.pins.iter().find(|p| p.0 == "tq144").unwrap().1;
    let gb = db.gbufpin.iter().find_map(|g| pkg.iter().find(|p| (p.1, p.2, p.3) == (g.0, g.1, g.2))).unwrap();
    let mut free = pkg.iter().filter(|p| p.0 != gb.0).map(|p| p.0.clone());
    let mut pins = vec![("clk".to_string(), gb.0.clone())];
    for p in n.inputs.iter().chain(&n.outputs).filter(|p| p.name != "clk") {
        for b in 0..p.width {
            pins.push((epitome_sv::arch::ice40::place::bit_name(&p.name, p.width, b), free.next().unwrap()));
        }
    }
    let b =
        build(&d, &db, &sem, n, &Options { package: "tq144".into(), pins, seed: 3 }).unwrap_or_else(|e| panic!("{e}"));
    eprintln!("{top}:\n{}", b.report());
    b
}

#[test]
fn memories_map_place_route() {
    let m = check_flow(RAMS, "rams", 400);
    // mem has two read ports: two copies; hi one.
    assert_eq!(m.rams.len(), 3);
    let b = build_tq144(RAMS, "rams");
    assert_eq!(b.placement.rams.iter().flatten().count(), 3);
    let m = check_flow(TABLES, "tables", 200);
    assert_eq!(m.rams.len(), 1);
    build_tq144(TABLES, "tables");
}

#[test]
fn ram_modes() {
    use epitome_sv::arch::ice40::map::ram_mode;
    assert_eq!(ram_mode(16, 256), Some((0, 1)));
    assert_eq!(ram_mode(8, 512), Some((1, 1)));
    assert_eq!(ram_mode(8, 20), Some((0, 1)));
    assert_eq!(ram_mode(1, 2048), Some((3, 1)));
    assert_eq!(ram_mode(32, 256), Some((0, 2)));
    assert_eq!(ram_mode(32, 300), Some((1, 4)));
    assert_eq!(ram_mode(1, 4096), None);
}

/// Programs `$TRY_SV` (top: the file's name; `$TRY_D` macros, comma
/// separated) and says what the serial port sees: for trying designs on
/// the board by hand.
#[test]
#[ignore]
fn board_try() {
    use epitome_sv::arch::ice40::board::{Icesprog, Serial};
    let Some(path) = std::env::var_os("TRY_SV") else { return };
    let path = PathBuf::from(path);
    let top = path.file_stem().unwrap().to_string_lossy().into_owned();
    let (d, db, sem) = records();
    let mut s = Sources::default();
    s.file(&path);
    for k in std::env::var("TRY_D").unwrap_or_default().split(',').filter(|k| !k.is_empty()) {
        let (k, v) = k.split_once('=').unwrap_or((k, "1"));
        s.define(k, v);
    }
    let n = synthesise(&s, &top).unwrap_or_else(|e| panic!("{e}"));
    let pins = parse_pins(&std::fs::read_to_string(arch().join("flow/nano.pins")).unwrap()).unwrap();
    let ports: Vec<&str> = n.inputs.iter().chain(&n.outputs).map(|p| p.name.as_str()).collect();
    let pins = pins.into_iter().filter(|(p, _)| ports.contains(&p.as_str())).collect();
    let b = build(&d, &db, &sem, n, &Options { package: "cm36".into(), pins, seed: 1 }).unwrap();
    eprintln!("{}", b.report());
    for (i, r) in b.mapped.rams.iter().enumerate() {
        eprintln!("ram {} mode {} at {:?}", r.name, r.mode, b.placement.rams[i]);
    }
    Icesprog::find().unwrap().program(&b.bitstream).unwrap();
    let mut ser = Serial::open(&Serial::find().unwrap()).unwrap();
    let probe: Vec<u8> = (0..64u8).map(|i| 0x20 + i).collect();
    eprintln!("TRY: {}", ser.echo(&probe).unwrap());
    ser.read_quiet(200);
    ser.send(&probe).unwrap();
    let back = ser.read_quiet(400);
    let diff = back.iter().zip(&probe).filter(|(a, b)| a != b).count();
    eprintln!("TRY bytes: {} back, {diff} differ: {:02x?}", back.len(), &back[..back.len().min(24)]);
}

pub const DEEP: &str = r#"
module deep(input logic clk, input logic we, input logic [3:0] a, input logic [3:0] b, input logic [3:0] d,
            output logic [3:0] q, output logic [3:0] r);
  // 5000 words: banks of 1024 (mode 1024x4), the last 904 deep. Addresses
  // a[3:2] * 2048 + a[1:0] land in banks 0, 2 and 4, and past the end.
  logic [3:0] mem [5000];
  always_ff @(posedge clk) begin
    if (we) mem[{a[3:2], 9'd0, a[1:0]}] <= d;
    q <= mem[{b[3:2], 9'd0, b[1:0]}];
  end
  always_ff @(posedge clk) if (d[0]) r <= mem[{a[1:0], 9'd0, b[1:0]}];
endmodule
"#;

#[test]
fn deep_memories_split_into_banks() {
    use epitome_sv::arch::ice40::map::bank_depth;
    assert_eq!(bank_depth(4, 5000), 1024);
    assert_eq!(bank_depth(8, 2048), 2048);
    assert_eq!(bank_depth(4, 16384), 2048);
    assert_eq!(bank_depth(1, 32768), 2048);
    assert_eq!(bank_depth(16, 1024), 1024);
    let n = check_netlist(DEEP, "deep", 600);
    let split = n.split_mems(&bank_depth).unwrap();
    eprintln!("split: {}", split.stats());
    assert_eq!(split.mems.len(), 5);
    let s = src(DEEP);
    let st = random_stimulus(&n, 600, 0x1234_5678).unwrap();
    let want = reference(&s, "deep", &n, &st).unwrap();
    compare(&want, &netlist_lines(&split, &st)).unwrap_or_else(|e| panic!("split: {e}"));
    let m = check_flow(DEEP, "deep", 600);
    assert_eq!(m.rams.iter().map(|r| r.blocks).sum::<u32>(), 10);
    build_tq144(DEEP, "deep");
}

pub const CALLS: &str = r#"
module calls(input logic clk, input logic [7:0] a, input logic [7:0] b, output logic [7:0] y, output logic [7:0] z,
             output logic [3:0] first, output logic [7:0] acc);
  function automatic logic [7:0] rev(input logic [7:0] x);
    for (int i = 0; i < 8; i++) rev[i] = x[7-i];
  endfunction
  function logic [7:0] sat(input logic [7:0] x, input logic [7:0] lim);
    if (x > lim) return lim;
    return x;
  endfunction
  // The first set bit, by an early return from a loop.
  function automatic logic [3:0] lowest(input logic [7:0] x);
    for (int i = 0; i < 8; i++)
      if (x[i]) return i[3:0];
    return 4'd15;
  endfunction
  task automatic swap(input logic [7:0] p, output logic [7:0] q);
    q = {p[3:0], p[7:4]};
  endtask
  always_comb begin
    y = sat(rev(a), b);
    swap(a ^ b, z);
    first = lowest(a & b);
  end
  // break and continue: add the bytes of b's set bits up to the first
  // clear bit past bit 1.
  always_ff @(posedge clk) begin
    logic [7:0] s;
    s = 0;
    for (int i = 0; i < 8; i++) begin
      if (i < 2) continue;
      if (!b[i]) break;
      s = s + a;
    end
    acc <= s;
  end
endmodule
"#;

#[test]
fn calls_match_interpreter() {
    check_flow(CALLS, "calls", 300);
}

pub const MEMWRITES: &str = r#"
module memwrites(input logic clk, input logic [3:0] wa, input logic [3:0] ra, input logic [1:0] be, input logic [15:0] wd,
                 input logic sel, output logic [15:0] q, output logic [7:0] rf_a, output logic [7:0] rf_b);
  // Byte writes: a lane of block RAM for each byte.
  logic [15:0] mem [16];
  always_ff @(posedge clk) begin
    if (be[0]) mem[wa][7:0] <= wd[7:0];
    if (be[1]) mem[wa][15:8] <= wd[15:8];
    q <= mem[ra];
  end
  // Two writes never enabled together share block RAM's one port.
  logic [7:0] ex [32];
  logic [7:0] exq;
  always_ff @(posedge clk) begin
    if (sel && be == 2'b11) ex[{1'b0, wa}] <= wd[7:0];
    if (!sel && be == 2'b11) ex[{1'b1, ra}] <= wd[15:8];
    exq <= ex[{be[0], wa}];
  end
  // A register file: two write ports and two reads at once: flip-flops.
  logic [7:0] rf [8];
  always_ff @(posedge clk) begin
    if (be[0]) rf[wa[2:0]] <= wd[7:0];
    if (be[1]) rf[ra[2:0]] <= wd[15:8] ^ exq;
  end
  assign rf_a = rf[wa[2:0] ^ 3'd1];
  assign rf_b = rf[ra[2:0]];
endmodule
"#;

#[test]
fn memory_writes_and_flip_flop_memories() {
    let n = check_netlist(MEMWRITES, "memwrites", 600);
    // mem and ex in block RAM; rf in flip-flops.
    assert_eq!(n.mems.len(), 2);
    assert!(n.mems[0].write.as_ref().unwrap().en.iter().any(|&e| e != n.mems[0].write.as_ref().unwrap().en[0]));
    let m = check_flow(MEMWRITES, "memwrites", 600);
    // mem in two lanes (bytes) and ex: three RAMs.
    assert_eq!(m.rams.len(), 3);
    build_tq144(MEMWRITES, "memwrites");
    // Initial contents of a memory of flip-flops, some ones.
    check_flow(
        "module t(input logic clk, input logic [1:0] a, input logic [3:0] d, input logic we, output logic [3:0] y);\n logic [3:0] m [4] = '{4'h9, 4'hf, 4'h0, 4'h6};\n always_ff @(posedge clk) if (we) m[a] <= d;\n assign y = m[a];\nendmodule\n",
        "t",
        200,
    );
}
