//! Lanes that follow instance counts (docs/design/sv.md section 8.3): the
//! netlist built for one simulation, a few and many, and arrays of
//! instances run as lanes of one call, each checked word for word against
//! Verilator's model of the same source (when Verilator is on the path, or
//! `VERILATOR` names it).
//!
//! On an AArch64 Mac or x86-64 Linux with `cc`; elsewhere the tests skip.

mod common;
#[path = "cva6/units.rs"]
mod units;

use std::path::PathBuf;

use common::*;
use epitome_sv::netlist::GROUP;

/// Arrays of instances of every kind the netlist meets: one of clocked
/// cells with ports stored every way (a bit, a byte, a half, pieces), one
/// whose iterations feed each other (called an instance at a time), and
/// one nested in another generate loop.
const ARRAYS: &str = r#"
module acc_cell #(parameter int W = 40) (
  input  logic          clk_i,
  input  logic          rst_ni,
  input  logic          en_i,
  input  logic [6:0]    k_i,
  input  logic [12:0]   h_i,
  input  logic [W-1:0]  d_i,
  input  logic [99:0]   wide_i,
  output logic [W-1:0]  q_o,
  output logic [12:0]   s_o,
  output logic          odd_o,
  output logic [99:0]   wide_o
);
  logic [W-1:0] acc;
  always_ff @(posedge clk_i or negedge rst_ni)
    if (!rst_ni) acc <= '0;
    else if (en_i) acc <= acc + d_i + W'(k_i);
  assign q_o = acc ^ {(W / 8){8'h5a}};
  assign s_o = h_i + 13'(k_i) * 13'd3;
  assign odd_o = ^d_i;
  assign wide_o = {wide_i[49:0], wide_i[99:50]} + 100'(k_i);
endmodule

module chain_link (input logic [15:0] a_i, input logic [3:0] k_i, output logic [15:0] y_o);
  assign y_o = (a_i << 1) ^ {12'b0, k_i};
endmodule

module leaf (input logic [7:0] x_i, input logic [2:0] sel_i, output logic [7:0] y_o);
  always_comb begin
    case (sel_i)
      3'd0: y_o = x_i;
      3'd1: y_o = ~x_i;
      3'd2: y_o = x_i + 8'd1;
      3'd3: y_o = {x_i[3:0], x_i[7:4]};
      default: y_o = x_i ^ {5'b0, sel_i};
    endcase
  end
endmodule

module arrays_top (
  input  logic          clk_i,
  input  logic          rst_ni,
  input  logic [4:0]    en_i,
  input  logic [34:0]   k_i,
  input  logic [64:0]   h_i,
  input  logic [199:0]  d_i,
  input  logic [99:0]   wide_i,
  input  logic [15:0]   seed_i,
  input  logic [23:0]   ksel_i,
  input  logic [95:0]   x_i,
  input  logic [35:0]   sel_i,
  output logic [199:0]  q_o,
  output logic [64:0]   s_o,
  output logic [4:0]    odd_o,
  output logic [99:0]   wide_o,
  output logic [15:0]   chain_o,
  output logic [95:0]   y_o
);
  logic [99:0] w [5];
  for (genvar i = 0; i < 5; i++) begin : g_cell
    acc_cell #(.W(40)) u (
      .clk_i, .rst_ni,
      .en_i(en_i[i]),
      .k_i(k_i[i*7 +: 7]),
      .h_i(h_i[i*13 +: 13]),
      .d_i(d_i[i*40 +: 40]),
      .wide_i(i == 2 ? ~wide_i : wide_i),
      .q_o(q_o[i*40 +: 40]),
      .s_o(s_o[i*13 +: 13]),
      .odd_o(odd_o[i]),
      .wide_o(w[i])
    );
  end
  assign wide_o = w[0] ^ w[1] ^ w[2] ^ w[3] ^ w[4];

  // Each link's input is the last one's output.
  logic [15:0] c [7];
  assign c[0] = seed_i;
  for (genvar i = 0; i < 6; i++) begin : g_chain
    chain_link u (.a_i(c[i]), .k_i(ksel_i[i*4 +: 4]), .y_o(c[i+1]));
  end
  assign chain_o = c[6];

  // Three rows of four leaves.
  for (genvar r = 0; r < 3; r++) begin : g_row
    for (genvar j = 0; j < 4; j++) begin : g_leaf
      leaf u (.x_i(x_i[(r*4+j)*8 +: 8]), .sel_i(sel_i[(r*4+j)*3 +: 3]), .y_o(y_o[(r*4+j)*8 +: 8]));
    end
  end
endmodule
"#;

/// The arrays design for one simulation, a few, and many, against
/// Verilator: arrays of instances are lanes of one call (the cells, the
/// leaves of each row) below parents narrower than 16 lanes, a chain of
/// them an instance at a time.
#[test]
fn arrays_of_instances() {
    let Some(host) = host() else {
        eprintln!("needs an AArch64 Mac or x86-64 Linux; skipping");
        return;
    };
    let src = text("arrays_top.sv", ARRAYS);
    let mut v = None;
    for sims in [1, 2, 5, 16, 33] {
        let name = format!("arrays_{sims}");
        let net = netlist_with(&src, "arrays_top", options_for(sims));
        if sims == 1 {
            // The cells one call of five lanes, each row's leaves one of
            // four, the chain's links one lane each, six calls.
            for (what, lanes) in [("acc_cell", 5), ("leaf", 4), ("chain_link", 1)] {
                assert!(net.functions.iter().any(|f| f.spec == what && f.lanes == lanes), "{what} is not lowered for {lanes} lanes");
            }
            let ir = net.ir();
            let calls = |what: &str| {
                let f = net.functions.iter().find(|f| f.spec == what).unwrap();
                ir.matches(&format!("call void @{}(", f.name)).count()
            };
            assert_eq!(calls("acc_cell"), 1, "the cells are not one call");
            assert_eq!(calls("leaf"), 3, "each row's leaves are not one call");
            assert_eq!(calls("chain_link"), 6, "the chain's links are not a call each");
        }
        let b = build(&name, net, &host);
        report(&name, &b);
        let io = Io::new(&b.net, Some("clk_i")).unwrap();
        if v.is_none() {
            v = verilate("arrays", &src, "arrays_top", &io);
        }
        let Some(v) = &v else { return };
        let (cycles, n) = (24, 99);
        let mut rng = Rng(0xA77A_5000_0000_0001 ^ sims as u64);
        let stim = stimulus_with(&io, cycles, n, &mut rng, &mut |name, c, rng, w| match name {
            "rst_ni" => w[0] = (c > 0 && rng.next() % 16 != 0) as u32,
            _ => {}
        });
        check_against(&b, &io, v, cycles, n, &stim, &|w| format!("{w:x?}"));
        let ns = time_replay(&b, &io, cycles, n, &stim, 20);
        eprintln!("{name}: {n} instances, {cycles} cycles agree with Verilator; {ns:.2} ns an instance a cycle; Verilator {}", bench(v, cycles, n, &stim, 20));
    }
}

/// CVA6's PMP unit for one simulation, a few and many, against Verilator:
/// its eight `pmp_entry`s are one call with eight lanes for each lane of
/// the unit, or with as many simulations as fill registers of 16, eight.
#[test]
fn pmp_entries_as_lanes() {
    let Some(cva6) = std::env::var_os("CVA6_DIR").map(PathBuf::from) else {
        eprintln!("set CVA6_DIR to a CVA6 checkout to run this; skipping");
        return;
    };
    let Some(host) = host() else {
        eprintln!("needs an AArch64 Mac or x86-64 Linux; skipping");
        return;
    };
    let u = units::unit("pmp");
    let src = u.sources(&cva6);
    let mut v = None;
    for sims in [1, 3, 16, GROUP] {
        let name = format!("pmp_lanes_{sims}");
        let net = netlist_with(&src, u.top, options_for(sims));
        // Below a unit of one register, pmp_entry for eight lanes for each
        // of the unit's (its register's width of them, padding included),
        // one call; below a wider one, eight calls with the unit's lanes.
        let unit = net.functions.iter().find(|f| f.spec == "pmp").expect("pmp is lowered");
        let sh = unit.shape;
        let grouped = sh.chunks == 1 && sh.n < 16;
        let lanes = if grouped { 8 * sh.width() as usize } else { net.lanes };
        let entries: Vec<_> = net.functions.iter().filter(|f| f.spec == "pmp_entry").collect();
        assert_eq!(entries.len(), 1, "pmp_entry lowered more than once");
        assert_eq!(entries[0].lanes as usize, lanes, "pmp_entry's lanes");
        let calls = net.ir().matches(&format!("call void @{}(", entries[0].name)).count();
        assert_eq!(calls, if grouped { 1 } else { 8 }, "pmp_entry's calls");
        let b = build(&name, net, &host);
        report(&name, &b);
        let io = Io::new(&b.net, None).unwrap();
        if v.is_none() {
            v = verilate("pmp_lanes", &src, u.top, &io);
        }
        let Some(v) = &v else { return };
        let n = GROUP;
        let stim = u.stimulus(units::Mix::Random, &io, 1, n, &mut Rng(u.seed ^ sims as u64));
        check_against(&b, &io, v, 1, n, &stim, &|w| format!("{w:x?}"));
        let ns = time_replay(&b, &io, 1, n, &stim, 200);
        eprintln!("{name}: {n} instances agree with Verilator; {ns:.2} ns an instance; Verilator {}", bench(v, 1, n, &stim, 200));
    }
}
