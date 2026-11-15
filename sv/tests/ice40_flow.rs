//! The FPGA flow (`epitome_sv::synth`, `epitome_sv::arch::ice40::flow`):
//! each stage checked on small designs, against Epitome's interpreter on
//! random stimulus. The board tests run only with `ICE40_BOARD` set to the
//! board plugged in: `nano` (or `1`) for the iCESugar-nano, `icesugar` for
//! the iCESugar.

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
            "module t(input logic c1, c2, d, output logic y);\n logic m [4];\n always_ff @(posedge c1) m[d] <= d;\n always_ff @(posedge c2) y <= m[d];\nendmodule\n",
            "two clocks",
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

/// A net declaration assignment is one continuous assignment (10.3.1),
/// not two drivers; also where the net is declared a port after it.
#[test]
fn net_declaration_assignments() {
    let text = r#"
module t(input logic clk, input logic [7:0] b, input logic [7:0] d, output logic [7:0] y, output logic x);
  logic [1:0] s = 0;
  always_ff @(posedge clk) s <= {s[0], b[0]};
  wire r = s[1];
  wire [7:0] a = b, c = d ^ 8'h5a;
  assign y = a + c;
  assign x = r;
endmodule
"#;
    check_flow(text, "t", 200);
    // Not ANSI: the port declared after its net, and before.
    check_flow(
        "module t(a, y, z);\n input logic [3:0] a;\n wire y = ^a;\n output y;\n output z;\n wire [1:0] z = a[1:0] & a[3:2];\nendmodule\n",
        "t",
        50,
    );
    let e = synthesise(
        &src("module t(input logic a, output logic y);\n wire w = a;\n assign w = ~a;\n assign y = w;\nendmodule\n"),
        "t",
    )
    .unwrap_err();
    assert!(e.contains("more than one driver"), "{e}");
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
use epitome_sv::arch::ice40::board::{Board, board};
use epitome_sv::arch::ice40::chipdb::Chipdb;
use epitome_sv::arch::ice40::flow::{Built, Options, build, parse_pins};
use epitome_sv::arch::ice40::icebox::Semantics;
use std::path::PathBuf;

fn arch() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("arch/ice40-lp1k")
}

fn records() -> (Device, Chipdb, Semantics) {
    nano().records().unwrap()
}

fn nano() -> &'static Board {
    board("nano").unwrap()
}

/// The board `ICE40_BOARD` names (`1` is the nano), if it is set.
fn live_board() -> Option<&'static Board> {
    let v = std::env::var("ICE40_BOARD").ok()?;
    let name = if v == "1" { "nano" } else { v.as_str() };
    Some(board(name).unwrap_or_else(|| panic!("ICE40_BOARD={v}: no such board (nano, icesugar)")))
}

/// A board design: the board's part's own (`flow/NAME.sv` in its
/// description), or else one of the designs every board runs, kept with
/// the nano's.
fn design_file(b: &Board, name: &str) -> PathBuf {
    let own = b.arch().join(format!("flow/{name}.sv"));
    if own.is_file() { own } else { arch().join(format!("flow/{name}.sv")) }
}

/// Builds `flow/NAME.sv` for the nano, checking the mapped cells against
/// the interpreter first.
fn build_design(name: &str, defines: &[&str]) -> Built {
    build_design_with(name, defines, false)
}

/// [`build_design`], with a clock the design makes routed through the
/// fabric if `fabric_clocks`.
fn build_design_with(name: &str, defines: &[&str], fabric_clocks: bool) -> Built {
    build_design_for(nano(), name, defines, fabric_clocks)
}

/// [`build_design_with`] for board `brd`.
fn build_design_for(brd: &Board, name: &str, defines: &[&str], fabric_clocks: bool) -> Built {
    let (d, db, sem) = brd.records().unwrap();
    let mut s = Sources::default();
    s.file(design_file(brd, name));
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
    let pins = board_pins(brd, &n);
    let b = build(&d, &db, &sem, n, &Options { package: brd.package.into(), pins, seed: 1, fabric_clocks })
        .unwrap_or_else(|e| panic!("{name}: {e}"));
    eprintln!("{name} on the {}:\n{}", brd.title, b.report());
    b
}

/// The board's pins for the ports the design has.
fn board_pins(brd: &Board, n: &epitome_sv::synth::Netlist) -> Vec<(String, String)> {
    let pins = parse_pins(&std::fs::read_to_string(brd.pins_file()).unwrap()).unwrap();
    let ports: Vec<String> = n
        .inputs
        .iter()
        .chain(&n.outputs)
        .flat_map(|p| (0..p.width).map(|b| epitome_sv::arch::ice40::place::bit_name(&p.name, p.width, b)))
        .collect();
    pins.into_iter().filter(|(p, _)| ports.contains(p)).collect()
}

#[test]
fn board_designs_build() {
    let echo = build_design("echo", &[]);
    assert_eq!(echo.mapped.lcs.len(), 0);
    let inv = build_design("echo", &["INVERT"]);
    assert_eq!(inv.mapped.lcs.len(), 1);
    let delay = build_design("delay", &[]);
    assert_eq!(delay.mapped.lcs.iter().filter(|l| l.ff.is_some()).count(), 4);
    assert_eq!(delay.clock_glb, vec![1]);
    let blink = build_design("blink", &[]);
    assert_eq!(blink.mapped.chains.len(), 1);
    build_design("hello", &[]);
    for w in ["16", "8", "4", "2", "1"] {
        let b = build_design("ramdelay", &[&format!("WIDTH={w}")]);
        assert_eq!(b.mapped.rams.len(), 1);
    }
    let b = build_design("hello_ram", &[]);
    assert_eq!(b.mapped.rams.len(), 1);
    // A clock the design makes: on a global network of its own (through a
    // global buffer input), or through the fabric.
    let b = build_design("hello_div", &[]);
    assert_eq!(b.clock_glb.len(), 2, "{:?}", b.clock_glb);
    let b = build_design_with("hello_div", &[], true);
    assert_eq!(b.clock_glb, vec![1]);
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
    // A line echo through a 512-byte RAM.
    let b = build_design("echo_ram", &[]);
    assert_eq!(b.mapped.rams.len(), 1);
}

/// Clocks a UART bit at 115200 baud from the board's 12 MHz.
const BIT: usize = 104;

/// The frames of `bytes` on a UART line, a level for each clock.
fn uart_frames(bytes: &[u8]) -> Vec<bool> {
    let mut v = Vec::new();
    for &b in bytes {
        let bits = std::iter::once(false).chain((0..8).map(|i| b >> i & 1 != 0)).chain(std::iter::once(true));
        for bit in bits {
            v.extend(std::iter::repeat_n(bit, BIT));
        }
    }
    v
}

/// The bytes on a UART line, a level for each clock: each start bit's
/// fall, then each bit sampled in its middle.
fn uart_bytes(line: &[bool]) -> Vec<u8> {
    let mut out = Vec::new();
    let mut t = 1;
    while t < line.len() {
        if line[t - 1] && !line[t] && t + 9 * BIT + BIT / 2 < line.len() {
            let mid = t + BIT / 2;
            assert!(!line[mid], "a start bit shorter than half a bit at {t}");
            let byte = (0..8).fold(0u8, |a, i| a | (line[mid + (i + 1) * BIT] as u8) << i);
            assert!(line[mid + 9 * BIT], "no stop bit at {}", mid + 9 * BIT);
            out.push(byte);
            t = mid + 9 * BIT;
        }
        t += 1;
    }
    out
}

/// echo_ram on UART frames: the interpreter, the netlist and the mapped
/// cells agree, and the line comes back reversed. A line feed, and bytes
/// sent while the line is going back, are dropped.
#[test]
fn echo_ram_reverses_a_line() {
    use epitome_sv::synth::check::Stimulus;
    let mut s = Sources::default();
    s.file(arch().join("flow/echo_ram.sv"));
    let n = synthesise(&s, "echo_ram").unwrap_or_else(|e| panic!("{e}"));
    let mut rx = vec![true; 200];
    rx.extend(uart_frames(b"ab\nc\r"));
    // The reply (five bytes) starts as the CR's stop bit is sampled.
    rx.extend(vec![true; 4 * BIT]);
    rx.extend(uart_frames(b"zz"));
    rx.extend(vec![true; 6 * 10 * BIT]);
    rx.extend(uart_frames(b"\r"));
    rx.extend(vec![true; 3 * 10 * BIT]);
    let clocks = n.clock_inputs().unwrap();
    let at = |name: &str| n.inputs.iter().position(|p| p.name == name).unwrap();
    let (clk, rxi) = (at("clk"), at("rx"));
    assert_eq!(clocks, vec![(clk, false)]);
    let cycles = rx.iter().map(|&r| (0..n.inputs.len()).map(|i| (i == rxi && r) as u64).collect()).collect();
    let st = Stimulus { clocks, cycles };
    let want = reference(&s, "echo_ram", &n, &st).unwrap();
    compare(&want, &netlist_lines(&n, &st)).unwrap_or_else(|e| panic!("netlist: {e}"));
    let m = map(&n).unwrap();
    eprintln!("echo_ram: {}", m.stats());
    assert_eq!(m.rams.len(), 1);
    let got = mapped_lines(&m, &st);
    compare(&want, &got).unwrap_or_else(|e| panic!("mapped: {e}"));
    let txi = n.outputs.iter().position(|p| p.name == "tx").unwrap();
    let tx: Vec<bool> = got.iter().map(|l| l.split(' ').nth(txi).unwrap() == "1").collect();
    assert_eq!(String::from_utf8_lossy(&uart_bytes(&tx)), "cba\r\n\r\n");
}

/// Programs the board with each design and watches the serial port; only
/// with `ICE40_BOARD` set to the board plugged in (`nano` or `icesugar`:
/// it reprograms it).
#[test]
fn board_live() {
    use epitome_sv::arch::ice40::board::{Icesprog, Serial};
    use std::time::{Duration, Instant};
    let Some(brd) = live_board() else {
        eprintln!("skipped: set ICE40_BOARD=nano (or icesugar) to reprogram the board plugged in");
        return;
    };
    let ip = Icesprog::find().expect("icesprog");
    ip.expect(brd).unwrap();
    let port = Serial::find().expect("serial port");
    let build_design = |name: &str, defines: &[&str]| build_design_for(brd, name, defines, false);
    let build_design_with = |name: &str, defines: &[&str], fabric| build_design_for(brd, name, defines, fabric);
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
    // (g) deeper than a block: banks, up to the nano's whole RAM (16
    // blocks), and on the UP5K 28 of its 30 (seven 2048x2 blocks wide,
    // four deep).
    let mut deep = vec![("1", "8192"), ("4", "16384")];
    if brd.name == "icesugar" {
        deep.push(("14", "8192"));
    }
    for (w, depth) in deep {
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
    // (i) a clock the design makes, on a global network of its own; and
    // (j) routed through the fabric instead.
    for (test, fabric) in [("(i)", false), ("(j)", true)] {
        let b = build_design_with("hello_div", &[], fabric);
        ip.program(&b.bitstream).unwrap();
        let mut s = Serial::open(&port).unwrap();
        s.read_for(100);
        let got = s.read_for(500);
        let whole = String::from_utf8_lossy(&got).matches("Hello from Epitome\r\n").count();
        eprintln!("{test} on a divided clock ({:?}): {} bytes, {whole} whole strings", b.clock_glb, got.len());
        assert!(whole >= 100, "{test} {whole} strings in 0.5 s");
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
    // (k) a line echo through block RAM: random printable lines, each sent
    // with a CR, come back reversed (cut to 511 bytes) with CR LF.
    let b = build_design("echo_ram", &[]);
    ip.program(&b.bitstream).unwrap();
    let mut s = Serial::open(&port).unwrap();
    s.read_quiet(200);
    let mut seed = 0x9E37_79B9_7F4A_7C15u64;
    let mut next = move || {
        seed ^= seed << 13;
        seed ^= seed >> 7;
        seed ^= seed << 17;
        seed
    };
    let mut lens = vec![0, 1, 3, 7, 17, 100, 255, 300, 511, 600];
    lens.extend((0..16).map(|_| [1, 17, 255, 300, 511][next() as usize % 5]));
    let mut bad = Vec::new();
    for &len in &lens {
        let line: Vec<u8> = (0..len).map(|_| 0x20 + (next() % 95) as u8).collect();
        let mut want: Vec<u8> = line[..len.min(511)].iter().rev().copied().collect();
        want.extend(b"\r\n");
        s.send(&line).unwrap();
        s.send(b"\r").unwrap();
        let back = s.read_quiet(300);
        if back != want {
            bad.push((len, back.len()));
        }
    }
    eprintln!(
        "(k) {} of {} lines back reversed; wrong (length, bytes back): {bad:?}",
        lens.len() - bad.len(),
        lens.len()
    );
    assert!(bad.is_empty(), "(k) {bad:?}");
    if brd.name == "icesugar" {
        board_rv32i(&ip, &port, brd);
    }
}

/// (l) The RV32I computer: started by a CR, it sends its banner, self-test
/// and primes, then answers lines, as the emulator does.
fn board_rv32i(ip: &epitome_sv::arch::ice40::board::Icesprog, port: &PathBuf, brd: &Board) {
    use epitome_sv::arch::ice40::board::Serial;
    use epitome_sv::arch::ice40::rv32i::{Emulator, assemble};
    let p = assemble(&std::fs::read_to_string(up5k().join("flow/rv32i_demo.s")).unwrap()).unwrap();
    let b = build_design_for(brd, "rv32i", &[], false);
    ip.program(&b.bitstream).unwrap();
    let mut s = Serial::open(port).unwrap();
    s.read_quiet(200);
    let mut seed = 0x2545_F491_4F6C_DD1Du64;
    let mut next = move || {
        seed ^= seed << 13;
        seed ^= seed >> 7;
        seed ^= seed << 17;
        seed
    };
    let mut lines: Vec<Vec<u8>> = vec![b"".to_vec(), b"Hello, iCESugar".to_vec(), b"".to_vec()];
    for len in [1, 7, 40, 199, 200, 230] {
        lines.push((0..len).map(|_| 0x20 + (next() % 95) as u8).collect());
    }
    let refs: Vec<&[u8]> = lines.iter().map(|l| l.as_slice()).collect();
    let want = Emulator::session(&p, &refs).unwrap();
    let mut bad = Vec::new();
    for (i, l) in lines.iter().enumerate() {
        s.send(l).unwrap();
        s.send(b"\r").unwrap();
        let back = s.read_quiet(300);
        if i < 3 {
            eprintln!("(l) {}", String::from_utf8_lossy(&back).trim_end());
        }
        if back != want[i + 1] {
            eprintln!("(l) line {i}: {:?}, not {:?}", String::from_utf8_lossy(&back), String::from_utf8_lossy(&want[i + 1]));
            bad.push(i);
        }
    }
    eprintln!("(l) the RV32I computer: {} of {} replies as the emulator's", lines.len() - bad.len(), lines.len());
    assert!(bad.is_empty(), "(l) {bad:?}");
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
    let b = build(&d, &db, &sem, n, &Options { package: "tq144".into(), pins, seed: 3, fabric_clocks: false })
        .unwrap_or_else(|e| panic!("{e}"));
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
/// separated) on the board `ICE40_BOARD` names (the nano if unset) and
/// says what the serial port sees: for trying designs on the board by
/// hand.
#[test]
#[ignore]
fn board_try() {
    use epitome_sv::arch::ice40::board::{Icesprog, Serial};
    let Some(path) = std::env::var_os("TRY_SV") else { return };
    let path = PathBuf::from(path);
    let top = path.file_stem().unwrap().to_string_lossy().into_owned();
    let brd = live_board().unwrap_or(nano());
    let (d, db, sem) = brd.records().unwrap();
    let mut s = Sources::default();
    s.file(&path);
    for k in std::env::var("TRY_D").unwrap_or_default().split(',').filter(|k| !k.is_empty()) {
        let (k, v) = k.split_once('=').unwrap_or((k, "1"));
        s.define(k, v);
    }
    let n = synthesise(&s, &top).unwrap_or_else(|e| panic!("{e}"));
    let pins = board_pins(brd, &n);
    let b = build(&d, &db, &sem, n, &Options { package: brd.package.into(), pins, seed: 1, fabric_clocks: false })
        .unwrap();
    eprintln!("{}", b.report());
    for (i, r) in b.mapped.rams.iter().enumerate() {
        eprintln!("ram {} mode {} at {:?}", r.name, r.mode, b.placement.rams[i]);
    }
    let ip = Icesprog::find().unwrap();
    ip.expect(brd).unwrap();
    ip.program(&b.bitstream).unwrap();
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

pub const CLOCKS: &str = r#"
module clocks(input logic c1, input logic c2, input logic rst, input logic [3:0] d,
              output logic [3:0] a, output logic [3:0] b, output logic [3:0] s, output logic [3:0] n);
  always_ff @(posedge c1) a <= d;
  always_ff @(posedge c2) b <= a + d;
  // A clock the design makes: c1 divided by two.
  logic div = 0;
  always_ff @(posedge c1) div <= ~div;
  always_ff @(posedge div or posedge rst)
    if (rst) s <= 0;
    else s <= s + 1;
  logic [3:0] nn = 0;
  always_ff @(negedge c1) nn <= nn ^ d;
  assign n = nn;
endmodule
"#;

#[test]
fn several_clocks() {
    let m = check_flow(CLOCKS, "clocks", 300);
    let mut clocks: Vec<_> = m.lcs.iter().filter_map(|l| l.ff.as_ref().map(|f| (f.clk, f.neg))).collect();
    clocks.sort();
    clocks.dedup();
    // c1 both edges, c2, and div.
    assert_eq!(clocks.len(), 4, "{clocks:?}");
    build_tq144(CLOCKS, "clocks");
}

// ---- timing ----

use epitome_sv::arch::ice40::timing::{self, Kind, Model};

fn model() -> Model {
    model_for(nano())
}

/// The board's part's timing record, or the guess.
fn model_for(brd: &Board) -> Model {
    std::fs::read_to_string(brd.arch().join("record-timing.txt"))
        .ok()
        .map_or_else(Model::guess, |t| Model::from_record(&t).unwrap())
}

/// Builds a timing experiment for the nano.
fn build_experiment(kind: Kind, n: usize) -> (Built, usize) {
    build_experiment_for(nano(), kind, n)
}

/// Builds a timing experiment for board `brd`: its clock, rx and tx pins.
fn build_experiment_for(brd: &Board, kind: Kind, n: usize) -> (Built, usize) {
    let (d, db, sem) = brd.records().unwrap();
    let (m, carries) = timing::experiment(kind, n);
    let pins = parse_pins(&std::fs::read_to_string(brd.pins_file()).unwrap()).unwrap();
    let pins = pins.into_iter().filter(|(p, _)| ["clk", "rx", "tx"].contains(&p.as_str())).collect();
    let b = epitome_sv::arch::ice40::flow::build_mapped(
        &d,
        &db,
        &sem,
        epitome_sv::synth::Netlist::new(),
        m,
        &Options { package: brd.package.into(), pins, seed: 1, fabric_clocks: false },
    )
    .unwrap_or_else(|e| panic!("{kind:?} {n}: {e}"));
    (b, carries)
}

/// With no delays (the cells' simulation), every experiment's capture is
/// right, so it echoes for ever.
#[test]
fn timing_experiments_echo_without_delays() {
    use epitome_sv::arch::ice40::map::LcSim;
    for (kind, n) in [(Kind::Luts, 5), (Kind::Spread, 6), (Kind::Carry, 2), (Kind::Ram, 3)] {
        let (m, _) = timing::experiment(kind, n);
        let mut sim = LcSim::new(&m);
        sim.inputs[1][0] = !0;
        sim.settle();
        for cycle in 0..40 {
            sim.inputs[0][0] = !0;
            sim.settle();
            sim.inputs[0][0] = 0;
            sim.settle();
            let tx = sim.get(m.outputs[0].1[0]) & 1;
            assert_eq!(tx, 1, "{kind:?}: the error flag set at cycle {cycle}");
        }
        build_experiment(kind, n);
    }
}

/// The board designs' longest paths, by the model.
#[test]
fn timing_of_board_designs() {
    let (d, db, _) = records();
    let model = model();
    for (name, defs) in
        [("blink", vec![]), ("hello", vec![]), ("hello_ram", vec![]), ("ramdelay", vec!["WIDTH=4", "DEPTH=16384"])]
    {
        let b = build_design(name, &defs);
        let r = timing::analyse(&b, &d, &db, &model);
        eprintln!("{name}: {}", r.text(&model));
        // Each runs at 12 MHz on the board.
        assert!(r.fmax().unwrap() > 12.0, "{name}: {}", r.text(&model));
    }
}

/// Measures the model on the board (`ICE40_BOARD=nano` or `icesugar`, and
/// `ICE40_CALIBRATE=1`): for each experiment, the shortest chain that
/// misses the 12 MHz clock, then the delays that fit, into the part's
/// `record-timing.txt`.
#[test]
fn timing_calibrate() {
    use epitome_sv::arch::ice40::board::{Icesprog, Serial};
    let (Some(brd), Some(_)) = (live_board(), std::env::var_os("ICE40_CALIBRATE")) else {
        eprintln!("skipped: set ICE40_BOARD=nano (or icesugar) and ICE40_CALIBRATE=1 to measure the timing on the board");
        return;
    };
    let ip = Icesprog::find().expect("icesprog");
    ip.expect(brd).unwrap();
    let port = Serial::find().expect("serial port");
    let probe: Vec<u8> = (0..64u8).map(|i| 0x20 + i).collect();
    let passes = |kind: Kind, n: usize| -> (bool, (usize, usize, usize)) {
        let (b, carries) = build_experiment_for(brd, kind, n);
        ip.program(&b.bitstream).unwrap();
        let ok = Serial::open(&port).unwrap().echo(&probe).unwrap() == "echo";
        let counts = timing::path_counts(&b, kind, n, carries);
        eprintln!("{kind:?} n={n}: {} (LUTs, switches, carries {counts:?})", if ok { "pass" } else { "FAIL" });
        (ok, counts)
    };
    // The threshold: the path counts halfway between the longest that
    // passes and the shortest that fails.
    let threshold = |kind: Kind, hi: usize| -> Option<(f64, f64, f64)> {
        let (ok, low) = passes(kind, 0);
        if !ok {
            return None;
        }
        let (ok, _) = passes(kind, hi);
        if ok {
            return None;
        }
        let (mut lo, mut hi, mut lo_c) = (0, hi, low);
        let mut hi_c = (0, 0, 0);
        while hi - lo > 1 {
            let mid = (lo + hi) / 2;
            let (ok, c) = passes(kind, mid);
            if ok { (lo, lo_c) = (mid, c) } else { (hi, hi_c) = (mid, c) }
        }
        if hi_c == (0, 0, 0) {
            hi_c = passes(kind, hi).1;
        }
        let avg = |a: usize, b: usize| (a + b) as f64 / 2.0;
        Some((avg(lo_c.0, hi_c.0), avg(lo_c.1, hi_c.1), avg(lo_c.2, hi_c.2)))
    };
    let t = timing::PERIOD;
    // ICE40_CALIBRATE=ram measures the RAM alone, keeping the rest.
    if std::env::var("ICE40_CALIBRATE").is_ok_and(|v| v == "ram") {
        let mut m = model_for(brd);
        let (l, h, _) = threshold(Kind::Ram, 600).expect("a RAM path that fails");
        // Within a stage of a flip-flop's clock-to-output: not below 0.
        m.ram = (t - l * m.lut - h * m.hop).max(0.0);
        m.source += &format!("; a RAM read then {l} LUTs with {h} switches");
        eprintln!("{}", m.to_record());
        std::fs::write(brd.arch().join("record-timing.txt"), m.to_record()).unwrap();
        return;
    }
    let (l1, h1, _) = threshold(Kind::Luts, 1100).expect("a LUT chain that fails");
    let (l2, h2, _) = threshold(Kind::Spread, 250).expect("a spread chain that fails");
    // l * lut + h * hop = t, twice.
    let det = l1 * h2 - l2 * h1;
    let lut = (t * h2 - t * h1) / det;
    let hop = (l1 * t - l2 * t) / det;
    eprintln!(
        "LUT chain: {l1} LUTs, {h1} switches; spread: {l2} LUTs, {h2} switches; so lut {lut:.3} ns, hop {hop:.3} ns"
    );
    let carry = match threshold(Kind::Carry, 600) {
        Some((l, h, c)) => (t - l * lut - h * hop) / c,
        None => f64::NAN,
    };
    let (ram, ram_note) = match threshold(Kind::Ram, 600) {
        Some((l, h, _)) => ((t - l * lut - h * hop).max(0.0), format!("; a RAM read then {l} LUTs with {h} switches")),
        None => (f64::NAN, String::new()),
    };
    let m = Model {
        lut,
        hop,
        carry,
        ram,
        source: format!(
            "measured on the {}: LUT chains of {l1} and {l2} LUTs with {h1} and {h2} switches at 83.3 ns, a 125-carry chain{ram_note}",
            brd.title
        ),
    };
    eprintln!("{}", m.to_record());
    std::fs::write(brd.arch().join("record-timing.txt"), m.to_record()).unwrap();
}

#[test]
fn spread_experiment_places() {
    for n in [10, 100, 250] {
        build_experiment(Kind::Spread, n);
    }
}

/// Programs one timing experiment (`TRY_KIND`, `TRY_N`) and says whether
/// it echoes.
#[test]
#[ignore]
fn timing_try() {
    use epitome_sv::arch::ice40::board::{Icesprog, Serial};
    let kind = match std::env::var("TRY_KIND").unwrap_or_default().as_str() {
        "spread" => Kind::Spread,
        "carry" => Kind::Carry,
        "ram" => Kind::Ram,
        _ => Kind::Luts,
    };
    let n: usize = std::env::var("TRY_N").ok().and_then(|v| v.parse().ok()).unwrap_or(0);
    let (b, _) = build_experiment_for(live_board().unwrap_or(nano()), kind, n);
    Icesprog::find().unwrap().program(&b.bitstream).unwrap();
    let probe: Vec<u8> = (0..64u8).map(|i| 0x20 + i).collect();
    eprintln!("TRY {kind:?} {n}: {}", Serial::open(&Serial::find().unwrap()).unwrap().echo(&probe).unwrap());
}

/// The RV32I assembler's encodings, against instructions written out by
/// hand from the RISC-V specification's formats.
#[test]
fn rv32i_assembles() {
    use epitome_sv::arch::ice40::rv32i::assemble;
    let p = assemble(
        "nop\nret\necall\nlui a0, 0x12345\naddi sp, sp, -16\nsw ra, 12(sp)\nlw ra, 12(sp)\n\
         x: jal ra, y\nbeq a0, a1, x\nsrai a0, a0, 7\nsub t0, t1, t2\ny: lbu a5, -1(s11)\nsb a5, 2047(a4)\nauipc t0, 1\n",
    )
    .unwrap();
    let want = [
        0x0000_0013u32,
        0x0000_8067,
        0x0000_0073,
        0x1234_5537,
        0xff01_0113,
        0x0011_2623,
        0x00c1_2083,
        0x0100_00ef, // jal ra, +16
        0xfeb5_0ee3, // beq a0, a1, -4
        0x4075_5513,
        0x4073_02b3,
        0xfffd_c783,
        0x7ef7_0fa3,
        0x0000_1297,
    ];
    let got = p.words();
    for (i, (g, w)) in got.iter().zip(&want).enumerate() {
        assert_eq!(g, w, "instruction {i}: {g:#010x}, not {w:#010x}");
    }
}

fn up5k() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("arch/ice40-up5k")
}

/// The demonstration program's image in the design is the assembler's.
#[test]
fn rv32i_image_is_current() {
    use epitome_sv::arch::ice40::rv32i::assemble;
    let dir = up5k().join("flow");
    let p = assemble(&std::fs::read_to_string(dir.join("rv32i_demo.s")).unwrap()).unwrap();
    assert_eq!(
        std::fs::read_to_string(dir.join("rv32i_demo.svh")).unwrap(),
        p.to_sv("rv32i_demo.s"),
        "rv32i_demo.svh is stale: `cargo run --release -p epitome-sv --example ice40_rv32i` remakes it"
    );
}

/// The RV32I computer's mapped cells, with its UART at 8 clocks a bit,
/// send what the emulator does: once a CR has started it, the banner,
/// self-test and primes, then a reply to each line, sent once the prompt
/// has come.
#[test]
fn rv32i_runs_its_program() {
    use epitome_sv::arch::ice40::map::LcSim;
    use epitome_sv::arch::ice40::rv32i::{Emulator, assemble};
    const BIT: usize = 8;
    let dir = up5k().join("flow");
    let p = assemble(&std::fs::read_to_string(dir.join("rv32i_demo.s")).unwrap()).unwrap();
    let lines: [&[u8]; 4] = [b"", b"Hello, iCESugar", b"", b"0123456789 abcdefghijklmnopqrstuvwxyz"];
    let want: Vec<u8> = Emulator::session(&p, &lines).unwrap().concat();
    let mut s = Sources::default();
    s.file(dir.join("rv32i.sv"));
    s.text(
        "fast.sv",
        format!("module fast(input logic clk, rx, output logic tx, led);\n  rv32i #(.BIT({BIT})) cpu(.*);\nendmodule\n"),
    );
    let n = synthesise(&s, "fast").unwrap_or_else(|e| panic!("{e}"));
    let m = map(&n).unwrap();
    eprintln!("rv32i: {}", m.stats());
    let at = |name: &str| m.inputs.iter().position(|p| p.name == name).unwrap();
    let (clk, rxi) = (at("clk"), at("rx"));
    let txo = m.outputs.iter().position(|(p, _)| p == "tx").unwrap();
    let mut sim = LcSim::new(&m);
    let mut sending: std::collections::VecDeque<bool> = Default::default();
    let mut got: Vec<u8> = Vec::new();
    let (mut prev, mut frame_at): (bool, Option<usize>) = (true, None);
    let mut next_line = 0;
    let prompts = |got: &[u8]| got.windows(2).filter(|w| w == b"> ").count();
    let t = std::time::Instant::now();
    let mut cycle = 0usize;
    let mut rx_was = false;
    while !(next_line == lines.len() && prompts(&got) >= lines.len()) {
        assert!(cycle < 2_000_000, "after {cycle} cycles: {:?}", String::from_utf8_lossy(&got));
        if sending.is_empty() && next_line < lines.len() && prompts(&got) >= next_line {
            for &b in lines[next_line].iter().chain(b"\r") {
                let bits = std::iter::once(false).chain((0..8).map(|i| b >> i & 1 != 0)).chain([true, true]);
                for bit in bits {
                    sending.extend(std::iter::repeat_n(bit, BIT));
                }
            }
            next_line += 1;
        }
        let rx = sending.pop_front().unwrap_or(true);
        if rx != rx_was || cycle == 0 {
            sim.inputs[rxi][0] = if rx { !0 } else { 0 };
            sim.settle();
            rx_was = rx;
        }
        let tx = sim.get(m.outputs[txo].1[0]) & 1 != 0;
        // A start bit's fall, then each bit sampled in its middle.
        match frame_at {
            None if prev && !tx => frame_at = Some(cycle),
            Some(f) if cycle == f + BIT / 2 + 9 * BIT => {
                frame_at = None;
                assert!(tx, "no stop bit at {cycle}");
            }
            _ => {}
        }
        if let Some(f) = frame_at {
            let k = cycle - f;
            if k >= BIT + BIT / 2 && (k - BIT / 2) % BIT == 0 && k < 9 * BIT {
                let i = (k - BIT / 2) / BIT - 1;
                if i == 0 {
                    got.push(0);
                }
                *got.last_mut().unwrap() |= (tx as u8) << i;
            }
        }
        prev = tx;
        sim.inputs[clk][0] = !0;
        sim.settle();
        sim.inputs[clk][0] = 0;
        sim.settle();
        cycle += 1;
    }
    eprintln!("{cycle} cycles in {:.1} s:\n{}", t.elapsed().as_secs_f64(), String::from_utf8_lossy(&got));
    assert_eq!(String::from_utf8_lossy(&got), String::from_utf8_lossy(&want));
}

/// The board designs build for the iCESugar's UP5K too, with its pins: a
/// RAM block there needs its PowerUp bit, which the nano's does not.
#[test]
fn up5k_designs_build() {
    let brd = board("icesugar").unwrap();
    let (d, db, sem) = brd.records().unwrap();
    let power_up = db.tile_bits.iter().find(|(k, _)| k == "ramb").unwrap().1.iter().find(|(f, _)| f == "RamConfig.PowerUp").unwrap().1[0];
    for (name, defs) in [("echo", vec![]), ("delay", vec![]), ("blink", vec![]), ("hello", vec![]), ("hello_ram", vec![]), ("echo_ram", vec![]), ("ramdelay", vec!["WIDTH=14", "DEPTH=8192"])] {
        let b = build_design_for(brd, name, &defs, false);
        let rams: Vec<(usize, usize)> = b.placement.rams.iter().flatten().copied().collect();
        for &(x, y) in &rams {
            assert!(b.bits.contains(&(x, y, power_up)), "{name}: the RAM at {x} {y} is not powered up");
        }
        if name == "ramdelay" {
            assert_eq!(rams.len(), 28);
        }
        let r = timing::analyse(&b, &d, &db, &model_for(brd));
        assert!(r.fmax().unwrap_or(1e9) > 12.0, "{name}: {}", r.text(&model_for(brd)));
    }
    let _ = sem;
    let nano_bits = build_design("hello_ram", &[]).bits;
    let (_, ndb, _) = records();
    let nano_power = ndb.tile_bits.iter().find(|(k, _)| k == "ramb").unwrap().1.iter().find(|(f, _)| f == "RamConfig.PowerUp").unwrap().1[0];
    assert!(!nano_bits.iter().any(|&(_, _, b)| b == nano_power), "the nano's RAM must not have PowerUp set");
    for n in [10, 100] {
        build_experiment_for(brd, Kind::Spread, n);
    }
}

/// The RV32I computer builds for the iCESugar: well over a thousand logic
/// cells and 20 block RAMs (16 of memory, 4 of registers), placed, routed
/// and inside the 12 MHz clock by the model.
#[test]
fn rv32i_builds() {
    let brd = board("icesugar").unwrap();
    let (d, db, _) = brd.records().unwrap();
    let t = std::time::Instant::now();
    let b = build_design_for(brd, "rv32i", &[], false);
    let r = timing::analyse(&b, &d, &db, &model_for(brd));
    eprintln!("{}\n{:.1} s", r.text(&model_for(brd)), t.elapsed().as_secs_f64());
    assert!(b.mapped.lcs.len() > 2000, "{}", b.mapped.stats());
    assert_eq!(b.placement.rams.iter().flatten().count(), 20);
    assert!(r.fmax().unwrap() > 12.0, "{}", r.text(&model_for(brd)));
}
