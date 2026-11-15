//! Units of the CVA6 RISC-V core (when `CVA6_DIR` points at a CVA6
//! checkout), in the cv64a6_imafdc_sv39 configuration (`cva6/units.rs`):
//! each compiled as a netlist, run on random stimulus for many instances,
//! and checked word for word against Verilator's model of the same source
//! (when Verilator is on the path, `VERILATOR` names it or
//! `VERILATOR_ROOT` holds it), which is also timed.
//!
//! On an AArch64 Mac or x86-64 Linux; elsewhere the tests skip.

mod common;
#[path = "cva6/units.rs"]
mod units;

use std::path::PathBuf;

use common::*;
use epitome_sv::netlist::GROUP;

/// CVA6's checkout and this machine, or why the test skips.
fn setup() -> Option<(PathBuf, Host)> {
    let Some(cva6) = std::env::var_os("CVA6_DIR").map(PathBuf::from) else {
        eprintln!("set CVA6_DIR to a CVA6 checkout to run this; skipping");
        return None;
    };
    let Some(host) = host() else {
        eprintln!("needs an AArch64 Mac or x86-64 Linux; skipping");
        return None;
    };
    Some((cva6, host))
}

/// Builds unit `name`, checks each kind of its stimulus against Verilator,
/// and times both, the stimulus run `reps` times.
fn check_unit(name: &str, reps: usize) {
    let Some((cva6, host)) = setup() else { return };
    let u = units::unit(name);
    let src = u.sources(&cva6);
    let b = build(name, netlist_of(&src, u.top), &host);
    report(name, &b);
    let io = Io::new(&b.net, u.clock).unwrap();
    let Some(v) = verilate(name, &src, u.top, &io) else { return };
    let (cycles, n) = (u.cycles, u.instances);
    let mut rng = Rng(u.seed);
    for &mix in u.mixes() {
        let stim = u.stimulus(mix, &io, cycles, n, &mut rng);
        check_against(&b, &io, &v, cycles, n, &stim, &|w| format!("{w:x?}"));
        let ns = time_replay(&b, &io, cycles, n, &stim, reps);
        eprintln!(
            "{name} ({mix:?}): {n} instances, {cycles} cycles agree with Verilator; {ns:.2} ns an instance a cycle; Verilator {}",
            bench(&v, cycles, n, &stim, reps)
        );
    }
}

#[test]
fn cva6_alu() {
    check_unit("alu", 100);
}

#[test]
fn cva6_exec() {
    check_unit("exec", 100);
}

#[test]
fn cva6_compressed_decoder() {
    check_unit("cdec", 100);
}

#[test]
fn cva6_decoder() {
    check_unit("decoder", 100);
}

#[test]
fn cva6_id_stage() {
    check_unit("id_stage", 3);
}

#[test]
fn cva6_issue_stage() {
    check_unit("issue_stage", 3);
}

#[test]
fn cva6_mult() {
    check_unit("mult", 3);
}

#[test]
fn cva6_serdiv() {
    check_unit("serdiv", 3);
}

/// common_cells' popcount, a tree of instances of itself, at several
/// widths.
#[test]
fn popcount_tree() {
    let Some((cva6, host)) = setup() else { return };
    for w in [64u32, 13, 32, 5] {
        let name = format!("popcount_{w}");
        let cw = 32 - w.leading_zeros();
        let mut src = Sources::default();
        src.file(cva6.join("vendor/pulp-platform/common_cells/src/popcount.sv")).text(
            format!("{name}.sv"),
            format!(
                "module {name} (input logic [{}:0] d, output logic [{}:0] c);\n  popcount #(.INPUT_WIDTH({w})) i (.data_i(d), .popcount_o(c));\nendmodule\n",
                w - 1,
                cw - 1
            ),
        );
        let net = netlist_of(&src, &name);
        // The tree is proved to count ones: at most the top and the count,
        // none of the instances below it.
        assert!(net.functions.len() <= 2, "{name}: the popcount tree is not recognised");
        let b = build(&name, net, &host);
        let io = Io::new(&b.net, None).unwrap();
        let mut rng = Rng(0x5EED ^ w as u64);
        let mask = if w == 64 { !0 } else { (1u64 << w) - 1 };
        let stim = stimulus(&io, 1, GROUP, &mut rng, &mut |_, _, rng| Some((rng.next() >> (rng.next() % 64)) & mask));
        let Some(v) = verilate(&name, &src, &name, &io) else { return };
        check_against(&b, &io, &v, 1, GROUP, &stim, &|x| format!("{x:x?}"));
    }
}

const CLOCKED: &str = r#"
// Registers: an asynchronous reset, an enable, a chain through a child
// (whose register the parent samples on the same edge), a counter with a
// variable part select, and a process on the falling edge.
module stage #(parameter int W = 8) (
    input logic clk_i, input logic rst_ni, input logic en_i,
    input logic [W-1:0] d_i, output logic [W-1:0] q_o
);
  always_ff @(posedge clk_i or negedge rst_ni)
    if (!rst_ni) q_o <= '0;
    else if (en_i) q_o <= d_i;
endmodule

module clocked_top (
    input logic clk_i, input logic rst_ni, input logic en_i,
    input logic [7:0] d_i, input logic [2:0] sel_i,
    output logic [7:0] a_o, output logic [7:0] b_o, output logic [7:0] c_o,
    output logic [15:0] count_o, output logic [7:0] fall_o, output logic [3:0] nib_o
);
  logic [7:0] a, b;
  stage #(8) s0 (.clk_i, .rst_ni, .en_i, .d_i, .q_o(a));
  stage #(8) s1 (.clk_i, .rst_ni, .en_i(1'b1), .d_i(a ^ 8'h5a), .q_o(b));
  always_ff @(posedge clk_i) c_o <= b + a;
  always_ff @(posedge clk_i or negedge rst_ni) begin
    if (!rst_ni) begin
      count_o <= 16'd0;
      nib_o <= '0;
    end else begin
      count_o <= count_o + {8'd0, d_i};
      nib_o[sel_i[1:0]] <= d_i[sel_i[2]];
      if (d_i == 8'hff) count_o[3:0] <= 4'hf;
    end
  end
  always_ff @(negedge clk_i) fall_o <= c_o - d_i;
  assign a_o = a;
  assign b_o = b;
endmodule
"#;

#[test]
fn clocked_basics() {
    let Some((_, host)) = setup() else { return };
    let src = text("clocked_top.sv", CLOCKED);
    let b = build("clocked", netlist_of(&src, "clocked_top"), &host);
    report("clocked", &b);
    let io = Io::new(&b.net, Some("clk_i")).unwrap();
    let Some(v) = verilate("clocked", &src, "clocked_top", &io) else { return };
    let (cycles, n) = (40, 2 * GROUP);
    let mut rng = Rng(0xC10C_4ED0_0000_0001);
    let stim = stimulus(&io, cycles, n, &mut rng, &mut |name, c, rng| match name {
        "rst_ni" => Some((c > 0 && rng.next() % 16 != 0) as u64),
        "en_i" => Some((rng.next() % 4 != 0) as u64),
        "d_i" => Some(if rng.next() % 8 == 0 { 0xff } else { rng.next() % 256 }),
        _ => None,
    });
    check_against(&b, &io, &v, cycles, n, &stim, &|w| format!("{w:x?}"));
}

/// For looking into a mismatch: `NETLIST_ADHOC=top[,clock]` and
/// `NETLIST_ADHOC_FILES=a.sv:b.sv:...` (with CVA6's packages before them)
/// check any design against Verilator on random stimulus.
#[test]
fn adhoc() {
    let (Ok(spec), Ok(list)) = (std::env::var("NETLIST_ADHOC"), std::env::var("NETLIST_ADHOC_FILES")) else { return };
    let Some((cva6, host)) = setup() else { return };
    let mut it = spec.split(',');
    let top = it.next().unwrap().to_string();
    let clock = it.next();
    let rest: Vec<&str> = list.split(':').collect();
    let src = units::cva6_sources(&cva6, &rest);
    let b = build("adhoc", netlist_of(&src, &top), &host);
    report("adhoc", &b);
    let io = Io::new(&b.net, clock).unwrap();
    let Some(v) = verilate("adhoc", &src, &top, &io) else { return };
    let cycles = if clock.is_some() { 8 } else { 1 };
    let mut rng = Rng(0xAD40C);
    let stim = stimulus(&io, cycles, GROUP, &mut rng, &mut |name, c, rng| match name {
        "rst_ni" => Some((c > 0) as u64),
        "in_vld_i" | "flush_i" => Some((rng.next() % 4 == 0) as u64),
        "op_a_i" | "op_b_i" => Some(units::operand(rng)),
        _ => None,
    });
    check_against(&b, &io, &v, cycles, GROUP, &stim, &|w| format!("{w:x?}"));
}

const GUARDED: &str = r#"
// Costly arms, each run only where some lane of a register of lanes takes
// it: if and else-if arms, case arms, a costly part of a function (with a
// return in a loop) under an if, and a register with an enable.
module guarded_top (
    input logic clk_i, input logic rst_ni,
    input logic [1:0] sel_i, input logic en_i,
    input logic [63:0] a_i, input logic [63:0] b_i,
    output logic [63:0] comb_o, output logic [63:0] case_o,
    output logic [63:0] acc_o, output logic [7:0] cnt_o, output logic [7:0] first_o
);
  function automatic logic [63:0] mix(input logic [63:0] x, input logic [63:0] y);
    logic [63:0] r;
    r = x;
    for (int i = 0; i < 4; i++) r = (r * y) ^ {r[31:0], r[63:32]} ^ (r >> 7);
    return r;
  endfunction
  function automatic logic [7:0] first_set(input logic [63:0] x, input logic go);
    first_set = 8'hff;
    if (go) begin
      for (int i = 0; i < 64; i++) if (x[i]) return i[7:0];
      return 8'd64;
    end
  endfunction
  always_comb begin
    comb_o = a_i ^ b_i;
    if (sel_i[0]) comb_o = mix(a_i, b_i);
    else if (sel_i[1]) comb_o = mix(b_i, a_i) + 64'd1;
  end
  always_comb begin
    case (sel_i)
      2'd0: case_o = a_i;
      2'd1: case_o = mix(a_i, ~b_i);
      2'd2: case_o = b_i - a_i;
      default: case_o = mix(~a_i, b_i) ^ 64'h5a5a;
    endcase
  end
  assign first_o = first_set(a_i & b_i, sel_i == 2'd3);
  always_ff @(posedge clk_i or negedge rst_ni)
    if (!rst_ni) begin
      acc_o <= '0;
      cnt_o <= '0;
    end else if (en_i) begin
      acc_o <= mix(acc_o ^ a_i, b_i | 64'd1);
      cnt_o <= cnt_o + 8'd1;
    end
endmodule
"#;

/// The guards (`netlist::GUARD_MIN`): registers of lanes where no lane, every
/// lane, one lane in sixteen and lanes at random take the costly arms.
#[test]
fn guarded_arms() {
    let Some((_, host)) = setup() else { return };
    let src = text("guarded_top.sv", GUARDED);
    // (Guarded as by default, whatever NETLIST_GUARD_MIN says; in 32-bit
    // lanes, where the arms' 64-bit products are costly enough to guard.)
    let opts = epitome_sv::netlist::Options { guard_min: epitome_sv::netlist::GUARD_MIN, lane_bits: Some(32), ..netlist_options() };
    let b = build("guarded", netlist_with(&src, "guarded_top", opts), &host);
    report("guarded", &b);
    let io = Io::new(&b.net, Some("clk_i")).unwrap();
    let Some(v) = verilate("guarded", &src, "guarded_top", &io) else { return };
    let (cycles, n) = (24, 2 * GROUP);
    let mut rng = Rng(0x6A2D_ED00_0000_0001);
    // Each register of 16 lanes: none, all, one, or some, changing with the
    // cycle so that a register goes from one to another.
    let stim = stimulus_at(&io, cycles, n, &mut rng, &mut |name, c, i, rng, w| {
        let lane = i % 16;
        let takes = match (i / 16 + c / 6) % 4 {
            0 => false,
            1 => true,
            2 => lane == c % 16,
            _ => rng.next() % 2 == 0,
        };
        match name {
            "rst_ni" => w[0] = (c > 0) as u32,
            "sel_i" => w[0] = if takes { 1 + (rng.next() % 3) as u32 } else { 0 },
            "en_i" => w[0] = takes as u32,
            _ => {}
        }
    });
    check_against(&b, &io, &v, cycles, n, &stim, &|w| format!("{w:x?}"));
    // (Scalar code puts costly arms behind branches of its own.)
    let ir = b.net.ir();
    let guards = ir.matches("label %guard").count() + ir.matches("label %then").count();
    // (The IR as text parses back.)
    epitome_ir::parse_module(&ir).unwrap_or_else(|e| panic!("the netlist's IR as text: {e:?}"));
    assert!(guards >= 4, "the costly arms are not guarded");
    let ns = time_replay(&b, &io, cycles, n, &stim, 3);
    // The same without guards, every arm run and merged by selects.
    let opts = epitome_sv::netlist::Options { guard_min: 0, lane_bits: Some(32), ..netlist_options() };
    let flat = build("guarded_flat", netlist_with(&src, "guarded_top", opts), &host);
    let flat_io = Io::new(&flat.net, Some("clk_i")).unwrap();
    check_against(&flat, &flat_io, &v, cycles, n, &stim, &|w| format!("{w:x?}"));
    let flat_ns = time_replay(&flat, &flat_io, cycles, n, &stim, 3);
    eprintln!(
        "guarded: {n} instances, {cycles} cycles agree with Verilator; {ns:.2} ns an instance a cycle, {flat_ns:.2} without guards; Verilator {}",
        bench(&v, cycles, n, &stim, 3)
    );
}
