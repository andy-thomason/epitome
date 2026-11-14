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
            "module t(input logic [1:0] a, input logic d, output logic y);\n logic m [4];\n always_comb y = m[a];\nendmodule\n",
            "memories",
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
    let (d, db, sem) = records();
    let n = synthesise(&src(text), "t").unwrap();
    // The CM36 has too few pins: the same die in its TQ144 package, the
    // clock on a pin with a global buffer.
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
    eprintln!("{}", b.report());
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
    epitome_sv::arch::ice40::place::check(&db, &b.mapped, &b.placement).unwrap();
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
        s.define(k, "1");
    }
    let n = synthesise(&s, name).unwrap_or_else(|e| panic!("{e}"));
    let st = random_stimulus(&n, 200, 7).unwrap();
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
