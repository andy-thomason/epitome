//! Units of the CVA6 RISC-V core (when `CVA6_DIR` points at a CVA6
//! checkout), in the cv64a6_imafdc_sv39 configuration: each compiled as a
//! netlist, run on random stimulus for many instances, and checked word
//! for word against Verilator's model of the same source (when Verilator
//! is on the path, or `VERILATOR` names it), which is also timed.
//!
//! On an AArch64 Mac or x86-64 Linux with `cc`; elsewhere the tests skip.

mod common;

use std::path::PathBuf;
use std::time::Instant;

use common::*;
use epitome_sv::netlist::GROUP;

/// CVA6's checkout and this machine's target, or why the test skips.
fn setup() -> Option<(PathBuf, (epitome_codegen::target::Target, &'static str, Vec<String>))> {
    let Some(cva6) = std::env::var_os("CVA6_DIR").map(PathBuf::from) else {
        eprintln!("set CVA6_DIR to a CVA6 checkout to run this; skipping");
        return None;
    };
    let Some(host) = host() else {
        eprintln!("needs cc on an AArch64 Mac or x86-64 Linux; skipping");
        return None;
    };
    Some((cva6, host))
}

/// The netlist's time an instance a cycle (two evaluations with a clock).
fn time_netlist(b: &Built, clock: Option<&str>, iters: usize) -> f64 {
    let st = State::new(b.net.size);
    let clk = clock.map(|c| b.net.port(c).unwrap().clone());
    let t0 = Instant::now();
    for _ in 0..iters {
        match &clk {
            Some(f) => {
                for level in [0, 1] {
                    for r in 0..GROUP / 16 {
                        unsafe { (st.ptr.add(f.offset + 2 * r) as *mut u16).write(if level == 1 { !0 } else { 0 }) };
                    }
                    unsafe { (b.eval)(std::hint::black_box(st.ptr)) };
                }
            }
            None => unsafe { (b.eval)(std::hint::black_box(st.ptr)) },
        }
    }
    t0.elapsed().as_secs_f64() * 1e9 / (iters * GROUP) as f64
}

const ALU_TOP: &str = r#"
// CVA6's ALU, its fu_data_t as cva6.sv declares it.
module alu_top
  import ariane_pkg::*;
#(
    parameter config_pkg::cva6_cfg_t CVA6Cfg = build_config_pkg::build_config(cva6_config_pkg::cva6_cfg),
    localparam type fu_data_t = struct packed {
      fu_t                              fu;
      fu_op                             operation;
      logic [CVA6Cfg.XLEN-1:0]          operand_a;
      logic [CVA6Cfg.XLEN-1:0]          operand_b;
      logic [CVA6Cfg.XLEN-1:0]          imm;
      logic [CVA6Cfg.TRANS_ID_BITS-1:0] trans_id;
    }
) (
    input fu_op operation_i,
    input logic [CVA6Cfg.XLEN-1:0] a_i,
    input logic [CVA6Cfg.XLEN-1:0] b_i,
    output logic [CVA6Cfg.XLEN-1:0] result_o,
    output logic branch_o
);
  fu_data_t d;
  assign d.fu = ALU;
  assign d.operation = operation_i;
  assign d.operand_a = a_i;
  assign d.operand_b = b_i;
  assign d.imm = '0;
  assign d.trans_id = '0;
  alu #(.CVA6Cfg(CVA6Cfg), .fu_data_t(fu_data_t)) i_alu (
      .clk_i(1'b0), .rst_ni(1'b1), .fu_data_i(d), .fu_data_cpop_i(d), .result_o, .alu_branch_res_o(branch_o));
endmodule
"#;

/// A 64-bit operand: random, small, or at an edge.
fn operand(rng: &mut Rng) -> u64 {
    match rng.next() % 8 {
        0 => rng.next() % 70,
        1 => [0, 1, !0, 1 << 63, (1 << 63) - 1, 0xFFFF_FFFF, 0x8000_0000, 0x7FFF_FFFF][rng.next() as usize % 8],
        2 => rng.next() >> (rng.next() % 64),
        3 => !(rng.next() >> (rng.next() % 64)),
        _ => rng.next(),
    }
}

#[test]
fn cva6_alu() {
    let Some((cva6, (t, triple, mattr))) = setup() else { return };
    let tmp = tmp_dir();
    std::fs::write(tmp.join("alu_top.sv"), ALU_TOP).unwrap();
    let top = tmp.join("alu_top.sv");
    let (files, incs) = cva6_files(
        &cva6,
        &["vendor/pulp-platform/common_cells/src/lzc.sv", "vendor/pulp-platform/common_cells/src/popcount.sv", "core/alu.sv", top.to_str().unwrap()],
    );
    let b = build("alu", netlist_of(files.clone(), incs.clone(), "alu_top"), &t, triple, &mattr);
    report("alu", &b);
    let Some(v) = verilate("alu", &files, &incs, "alu_top", &b.net.ports, None) else { return };
    let n = 4 * GROUP;
    let mut rng = Rng(0xA1B2_C3D4_E5F6_0718);
    let mut stim = Vec::new();
    for _ in 0..n {
        // Mostly the operations there are (the enum has about 225).
        let op = if rng.next() % 16 == 0 { rng.next() % 256 } else { rng.next() % 226 };
        let (a, bb) = (operand(&mut rng), operand(&mut rng));
        stim.extend([op as u32, a as u32, (a >> 32) as u32, bb as u32, (bb >> 32) as u32]);
    }
    check_against(&b, &v, 1, n, &stim, &|w| format!("op {} a {:x} b {:x}", w[0], w[1] as u64 | (w[2] as u64) << 32, w[3] as u64 | (w[4] as u64) << 32));
    let ns = time_netlist(&b, None, 2000);
    eprintln!("alu: {n} instances agree with Verilator; {ns:.2} ns an instance; Verilator {}", v.bench(1, n, &stim, 200));
}

/// common_cells' popcount, a tree of instances of itself, at several
/// widths.
#[test]
fn popcount_tree() {
    let Some((cva6, (t, triple, mattr))) = setup() else { return };
    let tmp = tmp_dir();
    for w in [64u32, 13, 32, 5] {
        let name = format!("popcount_{w}");
        let src = tmp.join(format!("{name}.sv"));
        let cw = 32 - w.leading_zeros();
        std::fs::write(
            &src,
            format!(
                "module {name} (input logic [{}:0] d, output logic [{}:0] c);\n  popcount #(.INPUT_WIDTH({w})) i (.data_i(d), .popcount_o(c));\nendmodule\n",
                w - 1,
                cw - 1
            ),
        )
        .unwrap();
        let files = vec![cva6.join("vendor/pulp-platform/common_cells/src/popcount.sv"), src];
        let b = build(&name, netlist_of(files.clone(), Vec::new(), &name), &t, triple, &mattr);
        let mut rng = Rng(0x5EED ^ w as u64);
        let mask = if w == 64 { !0 } else { (1u64 << w) - 1 };
        let stim: Vec<u32> = (0..GROUP)
            .flat_map(|_| {
                let x = (rng.next() >> (rng.next() % 64)) & mask;
                if w > 32 { vec![x as u32, (x >> 32) as u32] } else { vec![x as u32] }
            })
            .collect();
        let Some(v) = verilate(&name, &files, &[], &name, &b.net.ports, None) else { return };
        check_against(&b, &v, 1, GROUP, &stim, &|x| format!("{x:x?}"));
    }
}

const EXEC_TOP: &str = r#"
// The ALU and the branch unit, connected as ex_stage connects them, with
// the types of cva6.sv.
module exec_top
  import ariane_pkg::*;
#(
    parameter config_pkg::cva6_cfg_t CVA6Cfg = build_config_pkg::build_config(cva6_config_pkg::cva6_cfg),
    localparam type fu_data_t = struct packed {
      fu_t                              fu;
      fu_op                             operation;
      logic [CVA6Cfg.XLEN-1:0]          operand_a;
      logic [CVA6Cfg.XLEN-1:0]          operand_b;
      logic [CVA6Cfg.XLEN-1:0]          imm;
      logic [CVA6Cfg.TRANS_ID_BITS-1:0] trans_id;
    },
    localparam type branchpredict_sbe_t = struct packed {
      logic                    valid;
      cf_t                     cf;
      logic [CVA6Cfg.VLEN-1:0] predict_address;
    },
    localparam type exception_t = struct packed {
      logic [CVA6Cfg.XLEN-1:0] cause;
      logic [CVA6Cfg.XLEN-1:0] tval;
      logic [CVA6Cfg.GPLEN-1:0] tval2;
      logic [31:0] tinst;
      logic gva;
      logic valid;
      logic timing;
    },
    localparam type bp_resolve_t = struct packed {
      logic                    valid;
      logic [CVA6Cfg.VLEN-1:0] pc;
      logic [CVA6Cfg.VLEN-1:0] target_address;
      logic                    is_mispredict;
      logic                    is_taken;
      cf_t                     cf_type;
    }
) (
    input fu_op operation_i,
    input logic [CVA6Cfg.XLEN-1:0] a_i,
    input logic [CVA6Cfg.XLEN-1:0] b_i,
    input logic [CVA6Cfg.XLEN-1:0] imm_i,
    input logic [CVA6Cfg.VLEN-1:0] pc_i,
    input logic is_compressed_i,
    input logic branch_valid_i,
    input logic predict_valid_i,
    input cf_t predict_cf_i,
    input logic [CVA6Cfg.VLEN-1:0] predict_address_i,
    output logic [CVA6Cfg.XLEN-1:0] alu_result_o,
    output logic [CVA6Cfg.VLEN-1:0] branch_result_o,
    output bp_resolve_t resolved_branch_o,
    output logic resolve_branch_o,
    output exception_t branch_exception_o
);
  fu_data_t d;
  branchpredict_sbe_t bp;
  logic alu_branch_res;
  assign d = '{fu: ALU, operation: operation_i, operand_a: a_i, operand_b: b_i, imm: imm_i, trans_id: '0};
  assign bp = '{valid: predict_valid_i, cf: predict_cf_i, predict_address: predict_address_i};
  alu_wrapper #(.CVA6Cfg(CVA6Cfg), .fu_data_t(fu_data_t)) i_alu (
      .clk_i(1'b0), .rst_ni(1'b1), .alu_bypass_i('0), .fu_data_i(d), .result_o(alu_result_o),
      .alu_branch_res_o(alu_branch_res));
  branch_unit #(
      .CVA6Cfg(CVA6Cfg), .bp_resolve_t(bp_resolve_t), .branchpredict_sbe_t(branchpredict_sbe_t),
      .exception_t(exception_t), .fu_data_t(fu_data_t)
  ) i_branch (
      .clk_i(1'b0), .rst_ni(1'b1), .v_i(1'b0), .debug_mode_i(1'b0), .fu_data_i(d), .pc_i,
      .is_zcmt_i(1'b0), .is_compressed_instr_i(is_compressed_i), .branch_valid_i,
      .branch_comp_res_i(alu_branch_res), .branch_result_o, .branch_predict_i(bp),
      .resolved_branch_o, .resolve_branch_o, .branch_exception_o);
endmodule
"#;

#[test]
fn cva6_exec() {
    let Some((cva6, (t, triple, mattr))) = setup() else { return };
    let tmp = tmp_dir();
    let top = tmp.join("exec_top.sv");
    std::fs::write(&top, EXEC_TOP).unwrap();
    let (files, incs) = cva6_files(
        &cva6,
        &[
            "vendor/pulp-platform/common_cells/src/lzc.sv",
            "vendor/pulp-platform/common_cells/src/popcount.sv",
            "core/alu.sv",
            "core/alu_wrapper.sv",
            "core/branch_unit.sv",
            top.to_str().unwrap(),
        ],
    );
    let b = build("exec", netlist_of(files.clone(), incs.clone(), "exec_top"), &t, triple, &mattr);
    report("exec", &b);
    let Some(v) = verilate("exec", &files, &incs, "exec_top", &b.net.ports, None) else { return };
    let n = 4 * GROUP;
    let mut rng = Rng(0x0E1E_C0DE_5EED_0001);
    let mut stim = Vec::new();
    let w2 = |x: u64| [x as u32, (x >> 32) as u32];
    for _ in 0..n {
        // Half branches and jumps (LTS..JALR are 13..19).
        let op = if rng.next() % 2 == 0 { 13 + rng.next() % 7 } else { rng.next() % 226 };
        let (a, bb) = (operand(&mut rng), operand(&mut rng));
        let imm = ((rng.next() as i64) >> (40 + rng.next() % 24)) as u64;
        let pc = rng.next() & !1;
        let target = if op == 19 { a.wrapping_add(imm) & !1 } else { pc.wrapping_add(imm) };
        let predicted = if rng.next() % 2 == 0 { target } else { rng.next() };
        stim.extend([op as u32]);
        for x in [a, bb, imm, pc] {
            stim.extend(w2(x));
        }
        stim.extend([(rng.next() % 2) as u32, (rng.next() % 4 != 0) as u32, (rng.next() % 2) as u32, (rng.next() % 5) as u32]);
        stim.extend(w2(predicted));
    }
    check_against(&b, &v, 1, n, &stim, &|w| format!("{w:x?}"));
    let ns = time_netlist(&b, None, 2000);
    eprintln!("exec: {n} instances agree with Verilator; {ns:.2} ns an instance; Verilator {}", v.bench(1, n, &stim, 200));
}

/// The types of cva6.sv, for wrappers to import.
const SLICE_PKG: &str = include_str!("cva6/slice_pkg.sv");

/// Writes the package and a wrapper, and the files and include
/// directories of CVA6 with `rest` and those.
fn slice_files(cva6: &std::path::Path, wrapper: &str, sv: &str, rest: &[&str]) -> (Vec<PathBuf>, Vec<PathBuf>) {
    let tmp = tmp_dir();
    let (pkg, top) = (tmp.join("slice_pkg.sv"), tmp.join(format!("{wrapper}.sv")));
    std::fs::write(&pkg, SLICE_PKG).unwrap();
    std::fs::write(&top, sv).unwrap();
    let mut all: Vec<&str> = vec![pkg.to_str().unwrap()];
    all.extend(rest);
    all.push(top.to_str().unwrap());
    cva6_files(cva6, &all)
}

/// The decoder's choices for a 32-bit instruction word.
fn decoder_pick(name: &str, rng: &mut Rng) -> Option<u64> {
    match name {
        "instruction_i" => Some(rv_instruction(rng) as u64),
        "is_compressed_i" | "is_illegal_i" | "debug_req_i" | "debug_mode_i" => Some((rng.next() % 8 == 0) as u64),
        // No exception from fetch, mostly.
        "ex_i" => Some(rng.next() & !(((rng.next() % 4 != 0) as u64) << 1)),
        _ => None,
    }
}

#[test]
fn cva6_decoder() {
    let Some((cva6, (t, triple, mattr))) = setup() else { return };
    let (files, incs) = slice_files(&cva6, "decoder_top", include_str!("cva6/decoder_top.sv"), &["core/decoder.sv"]);
    let b = build("decoder", netlist_of(files.clone(), incs.clone(), "decoder_top"), &t, triple, &mattr);
    report("decoder", &b);
    let Some(v) = verilate("decoder", &files, &incs, "decoder_top", &b.net.ports, None) else { return };
    let n = 8 * GROUP;
    let mut rng = Rng(0xDEC0_DE00_1234_5678);
    let stim = stimulus(&v, 1, n, &mut rng, &mut |name, _, rng| decoder_pick(name, rng));
    check_against(&b, &v, 1, n, &stim, &|w| format!("{w:x?}"));
    let ns = time_netlist(&b, None, 1000);
    eprintln!("decoder: {n} instances agree with Verilator; {ns:.2} ns an instance; Verilator {}", v.bench(1, n, &stim, 100));
}

#[test]
fn cva6_compressed_decoder() {
    let Some((cva6, (t, triple, mattr))) = setup() else { return };
    let (files, incs) = slice_files(&cva6, "cdec_top", include_str!("cva6/cdec_top.sv"), &["core/compressed_decoder.sv"]);
    let b = build("cdec", netlist_of(files.clone(), incs.clone(), "cdec_top"), &t, triple, &mattr);
    report("cdec", &b);
    let Some(v) = verilate("cdec", &files, &incs, "cdec_top", &b.net.ports, None) else { return };
    let n = 8 * GROUP;
    let mut rng = Rng(0xC0DE_C0DE_0000_0016);
    // Mostly compressed: the low two bits not 11.
    let stim = stimulus(&v, 1, n, &mut rng, &mut |_, _, rng| {
        let x = rng.next();
        Some(if x % 8 == 0 { x | 3 } else if x & 3 == 3 { x & !1 } else { x })
    });
    check_against(&b, &v, 1, n, &stim, &|w| format!("{w:x?}"));
    let ns = time_netlist(&b, None, 1000);
    eprintln!("cdec: {n} instances agree with Verilator; {ns:.2} ns an instance; Verilator {}", v.bench(1, n, &stim, 100));
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
    let Some((_, (t, triple, mattr))) = setup() else { return };
    let src = tmp_dir().join("clocked_top.sv");
    std::fs::write(&src, CLOCKED).unwrap();
    let files = vec![src];
    let b = build("clocked", netlist_of(files.clone(), Vec::new(), "clocked_top"), &t, triple, &mattr);
    report("clocked", &b);
    let Some(v) = verilate("clocked", &files, &[], "clocked_top", &b.net.ports, Some("clk_i")) else { return };
    let (cycles, n) = (40, 2 * GROUP);
    let mut rng = Rng(0xC10C_4ED0_0000_0001);
    let stim = stimulus(&v, cycles, n, &mut rng, &mut |name, c, rng| match name {
        "rst_ni" => Some((c > 0 && rng.next() % 16 != 0) as u64),
        "en_i" => Some((rng.next() % 4 != 0) as u64),
        "d_i" => Some(if rng.next() % 8 == 0 { 0xff } else { rng.next() % 256 }),
        _ => None,
    });
    check_against(&b, &v, cycles, n, &stim, &|w| format!("{w:x?}"));
}
