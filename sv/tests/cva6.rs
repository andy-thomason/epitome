//! Units of the CVA6 RISC-V core (when `CVA6_DIR` points at a CVA6
//! checkout), in the cv64a6_imafdc_sv39 configuration: each compiled as a
//! netlist, run on random stimulus for many instances, and checked word
//! for word against Verilator's model of the same source (when Verilator
//! is on the path, or `VERILATOR` names it), which is also timed.
//!
//! On an AArch64 Mac or x86-64 Linux with `cc`; elsewhere the tests skip.

mod common;

use std::path::PathBuf;

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
    let ns = time_replay(&b, &v, 1, n, &stim, 100);
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
        let net = netlist_of(files.clone(), Vec::new(), &name);
        // The tree is proved to count ones: at most the top and the count,
        // none of the instances below it.
        assert!(net.ir.matches("\ndefine void @sv_eval_").count() <= 2, "{name}: the popcount tree is not recognised");
        let b = build(&name, net, &t, triple, &mattr);
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
    let ns = time_replay(&b, &v, 1, n, &stim, 100);
    eprintln!("exec: {n} instances agree with Verilator; {ns:.2} ns an instance; Verilator {}", v.bench(1, n, &stim, 200));
}

/// The types of cva6.sv, for wrappers to import.
const SLICE_PKG: &str = include_str!("cva6/slice_pkg.sv");

/// Writes the package and a wrapper, and the files and include
/// directories of CVA6 with `rest` and those.
fn slice_files(cva6: &std::path::Path, wrapper: &str, sv: &str, rest: &[&str]) -> (Vec<PathBuf>, Vec<PathBuf>) {
    let tmp = tmp_dir();
    // (A package file for each wrapper, as tests run at once.)
    let (pkg, top) = (tmp.join(format!("{wrapper}_pkg.sv")), tmp.join(format!("{wrapper}.sv")));
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
    let ns = time_replay(&b, &v, 1, n, &stim, 100);
    eprintln!("decoder: {n} instances agree with Verilator; {ns:.2} ns an instance; Verilator {}", v.bench(1, n, &stim, 100));
    // As a program's mix, where most arms are taken by no lane.
    let typ = stimulus(&v, 1, n, &mut rng, &mut |name, _, rng| match name {
        "instruction_i" => Some(rv_instruction_typical(rng) as u64),
        "is_compressed_i" | "is_illegal_i" | "debug_req_i" | "debug_mode_i" => Some(0),
        "ex_i" => Some(0),
        "irq_i" => Some(0),
        _ => None,
    });
    check_against(&b, &v, 1, n, &typ, &|w| format!("{w:x?}"));
    let ns = time_replay(&b, &v, 1, n, &typ, 100);
    eprintln!("decoder, typical: {ns:.2} ns an instance; Verilator {}", v.bench(1, n, &typ, 100));
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
    let ns = time_replay(&b, &v, 1, n, &stim, 100);
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

#[test]
fn cva6_serdiv() {
    let Some((cva6, (t, triple, mattr))) = setup() else { return };
    let (files, incs) = slice_files(
        &cva6,
        "serdiv_top",
        include_str!("cva6/serdiv_top.sv"),
        &["vendor/pulp-platform/common_cells/src/lzc.sv", "core/serdiv.sv"],
    );
    let b = build("serdiv", netlist_of(files.clone(), incs.clone(), "serdiv_top"), &t, triple, &mattr);
    report("serdiv", &b);
    let Some(v) = verilate("serdiv", &files, &incs, "serdiv_top", &b.net.ports, Some("clk_i")) else { return };
    let (cycles, n) = (300, GROUP);
    let mut rng = Rng(0x5E2D_1700_0000_0001);
    let stim = stimulus(&v, cycles, n, &mut rng, &mut |name, c, rng| match name {
        "rst_ni" => Some((c > 0 && rng.next() % 512 != 0) as u64),
        "in_vld_i" => Some((rng.next() % 4 == 0) as u64),
        "out_rdy_i" => Some((rng.next() % 8 != 0) as u64),
        "flush_i" => Some((rng.next() % 128 == 0) as u64),
        "op_a_i" | "op_b_i" => Some(operand(rng)),
        _ => None,
    });
    check_against(&b, &v, cycles, n, &stim, &|w| format!("{w:x?}"));
    let ns = time_replay(&b, &v, cycles, n, &stim, 3);
    eprintln!("serdiv: {n} instances, {cycles} cycles agree with Verilator; {ns:.2} ns an instance a cycle; Verilator {}", v.bench(cycles, n, &stim, 3));
    // A division now and then, results taken at once.
    let typ = stimulus(&v, cycles, n, &mut rng, &mut |name, c, rng| match name {
        "rst_ni" => Some((c > 0) as u64),
        "flush_i" => Some(0),
        "in_vld_i" => Some((rng.next() % 64 == 0) as u64),
        "out_rdy_i" => Some(1),
        "op_a_i" | "op_b_i" => Some(operand(rng)),
        _ => None,
    });
    check_against(&b, &v, cycles, n, &typ, &|w| format!("{w:x?}"));
    let ns = time_replay(&b, &v, cycles, n, &typ, 3);
    eprintln!("serdiv, typical: {ns:.2} ns an instance a cycle; Verilator {}", v.bench(cycles, n, &typ, 3));
}

/// For looking into a mismatch: `NETLIST_ADHOC=top[,clock]` and
/// `NETLIST_ADHOC_FILES=a.sv:b.sv:...` (with CVA6's packages before them)
/// check any design against Verilator on random stimulus.
#[test]
fn adhoc() {
    let (Ok(spec), Ok(list)) = (std::env::var("NETLIST_ADHOC"), std::env::var("NETLIST_ADHOC_FILES")) else { return };
    let Some((cva6, (t, triple, mattr))) = setup() else { return };
    let mut it = spec.split(',');
    let top = it.next().unwrap().to_string();
    let clock = it.next();
    let rest: Vec<&str> = list.split(':').collect();
    let (files, incs) = cva6_files(&cva6, &rest);
    let b = build("adhoc", netlist_of(files.clone(), incs.clone(), &top), &t, triple, &mattr);
    report("adhoc", &b);
    let Some(v) = verilate("adhoc", &files, &incs, &top, &b.net.ports, clock) else { return };
    let cycles = if clock.is_some() { 8 } else { 1 };
    let mut rng = Rng(0xAD40C);
    let stim = stimulus(&v, cycles, GROUP, &mut rng, &mut |name, c, rng| match name {
        "rst_ni" => Some((c > 0) as u64),
        "in_vld_i" | "flush_i" => Some((rng.next() % 4 == 0) as u64),
        "op_a_i" | "op_b_i" => Some(operand(rng)),
        _ => None,
    });
    check_against(&b, &v, cycles, GROUP, &stim, &|w| format!("{w:x?}"));
}

#[test]
fn cva6_id_stage() {
    let Some((cva6, (t, triple, mattr))) = setup() else { return };
    let (files, incs) = slice_files(
        &cva6,
        "id_top",
        include_str!("cva6/id_top.sv"),
        &["core/compressed_decoder.sv", "core/decoder.sv", "core/cvxif_compressed_if_driver.sv", "core/id_stage.sv"],
    );
    let b = build("id_stage", netlist_of(files.clone(), incs.clone(), "id_top"), &t, triple, &mattr);
    report("id_stage", &b);
    let Some(v) = verilate("id_stage", &files, &incs, "id_top", &b.net.ports, Some("clk_i")) else { return };
    // fetch_entry_t: address, instruction, branch prediction (68 bits),
    // then the exception, whose valid bit is bit 1.
    let fe = b.net.port("fetch_entry_i").unwrap().width as usize;
    let ex = fe - 64 - 32 - 68;
    let (cycles, n) = (60, GROUP);
    let mut rng = Rng(0x1D57_A6E0_0000_0001);
    let stim = stimulus_with(&v, cycles, n, &mut rng, &mut |name, c, rng, w| match name {
        "rst_ni" => w[0] = (c > 0 && rng.next() % 64 != 0) as u32,
        "flush_i" => w[0] = (rng.next() % 16 == 0) as u32,
        "debug_req_i" | "debug_mode_i" => w[0] = (rng.next() % 16 == 0) as u32,
        "fetch_entry_valid_i" => w[0] = (rng.next() % 4 != 0) as u32,
        "issue_instr_ack_i" => w[0] = (rng.next() % 2 == 0) as u32,
        "compressed_ready_i" => w[0] = (rng.next() % 4 != 0) as u32,
        "fetch_entry_i" => {
            // Compressed instructions a third of the time.
            let i = if rng.next() % 3 == 0 { rng.next() as u32 & !3 | (rng.next() % 3) as u32 } else { rv_instruction(rng) };
            set_bits(w, ex + 68, 32, i as u64);
            set_bits(w, 1, 1, (rng.next() % 8 == 0) as u64);
        }
        _ => {}
    });
    check_against(&b, &v, cycles, n, &stim, &|w| format!("{w:x?}"));
    let ns = time_replay(&b, &v, cycles, n, &stim, 3);
    eprintln!("id_stage: {n} instances, {cycles} cycles agree with Verilator; {ns:.2} ns an instance a cycle; Verilator {}", v.bench(cycles, n, &stim, 3));
    // A program's mix, uncompressed, with no flushes, debug or exceptions.
    let typ = stimulus_with(&v, cycles, n, &mut rng, &mut |name, c, rng, w| match name {
        "rst_ni" => w[0] = (c > 0) as u32,
        "flush_i" | "debug_req_i" | "debug_mode_i" => w[0] = 0,
        "fetch_entry_valid_i" | "issue_instr_ack_i" | "compressed_ready_i" => w[0] = (rng.next() % 8 != 0) as u32,
        "irq_i" => w[0] = 0,
        "priv_lvl_i" => w[0] = 0,
        "fetch_entry_i" => {
            set_bits(w, ex + 68, 32, rv_instruction_typical(rng) as u64);
            set_bits(w, 0, ex, 0);
        }
        _ => {}
    });
    check_against(&b, &v, cycles, n, &typ, &|w| format!("{w:x?}"));
    let ns = time_replay(&b, &v, cycles, n, &typ, 3);
    eprintln!("id_stage, typical: {ns:.2} ns an instance a cycle; Verilator {}", v.bench(cycles, n, &typ, 3));
}

#[test]
fn cva6_mult() {
    let Some((cva6, (t, triple, mattr))) = setup() else { return };
    let (files, incs) = slice_files(
        &cva6,
        "mult_top",
        include_str!("cva6/mult_top.sv"),
        &["vendor/pulp-platform/common_cells/src/lzc.sv", "core/serdiv.sv", "core/multiplier.sv", "core/mult.sv"],
    );
    let b = build("mult", netlist_of(files.clone(), incs.clone(), "mult_top"), &t, triple, &mattr);
    report("mult", &b);
    let Some(v) = verilate("mult", &files, &incs, "mult_top", &b.net.ports, Some("clk_i")) else { return };
    let (cycles, n) = (200, GROUP);
    let mut rng = Rng(0x3017_0000_0000_0001);
    // MUL..REMUW are 87..99, CLMUL..CLMULR 159..161.
    let stim = stimulus(&v, cycles, n, &mut rng, &mut |name, c, rng| match name {
        "rst_ni" => Some((c > 0 && rng.next() % 512 != 0) as u64),
        "flush_i" => Some((rng.next() % 64 == 0) as u64),
        "mult_valid_i" => Some((rng.next() % 3 == 0) as u64),
        "operation_i" => Some(if rng.next() % 4 == 0 { 159 + rng.next() % 3 } else { 87 + rng.next() % 13 }),
        "a_i" | "b_i" => Some(operand(rng)),
        _ => None,
    });
    check_against(&b, &v, cycles, n, &stim, &|w| format!("{w:x?}"));
    let ns = time_replay(&b, &v, cycles, n, &stim, 3);
    eprintln!("mult: {n} instances, {cycles} cycles agree with Verilator; {ns:.2} ns an instance a cycle; Verilator {}", v.bench(cycles, n, &stim, 3));
    // A request now and then, mostly MUL, divisions rare.
    let typ = stimulus(&v, cycles, n, &mut rng, &mut |name, c, rng| match name {
        "rst_ni" => Some((c > 0) as u64),
        "flush_i" => Some(0),
        "mult_valid_i" => Some((rng.next() % 16 == 0) as u64),
        "operation_i" => Some(if rng.next() % 32 == 0 { 92 + rng.next() % 8 } else { 87 }),
        "a_i" | "b_i" => Some(operand(rng)),
        _ => None,
    });
    check_against(&b, &v, cycles, n, &typ, &|w| format!("{w:x?}"));
    let ns = time_replay(&b, &v, cycles, n, &typ, 3);
    eprintln!("mult, typical: {ns:.2} ns an instance a cycle; Verilator {}", v.bench(cycles, n, &typ, 3));
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
    let Some((_, (t, triple, mattr))) = setup() else { return };
    let src = tmp_dir().join("guarded_top.sv");
    std::fs::write(&src, GUARDED).unwrap();
    let files = vec![src];
    // (Guarded as by default, whatever NETLIST_GUARD_MIN says.)
    let opts = epitome_sv::netlist::Options { guard_min: epitome_sv::netlist::GUARD_MIN, ..netlist_options() };
    let b = build("guarded", netlist_with(files.clone(), Vec::new(), "guarded_top", opts), &t, triple, &mattr);
    report("guarded", &b);
    let Some(v) = verilate("guarded", &files, &[], "guarded_top", &b.net.ports, Some("clk_i")) else { return };
    let (cycles, n) = (24, 2 * GROUP);
    let mut rng = Rng(0x6A2D_ED00_0000_0001);
    // Each register of 16 lanes: none, all, one, or some, changing with the
    // cycle so that a register goes from one to another.
    let stim = stimulus_at(&v, cycles, n, &mut rng, &mut |name, c, i, rng, w| {
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
    check_against(&b, &v, cycles, n, &stim, &|w| format!("{w:x?}"));
    // (Scalar code puts costly arms behind branches of its own.)
    let guards = b.net.ir.matches(", label %G").count() + b.net.ir.matches(", label %T").count();
    assert!(guards >= 4, "the costly arms are not guarded");
    let ns = time_replay(&b, &v, cycles, n, &stim, 3);
    // The same without guards, every arm run and merged by selects.
    let opts = epitome_sv::netlist::Options { guard_min: 0, ..netlist_options() };
    let flat = build("guarded_flat", netlist_with(files.clone(), Vec::new(), "guarded_top", opts), &t, triple, &mattr);
    check_against(&flat, &v, cycles, n, &stim, &|w| format!("{w:x?}"));
    let flat_ns = time_replay(&flat, &v, cycles, n, &stim, 3);
    eprintln!(
        "guarded: {n} instances, {cycles} cycles agree with Verilator; {ns:.2} ns an instance a cycle, {flat_ns:.2} without guards; Verilator {}",
        v.bench(cycles, n, &stim, 3)
    );
}

/// Bit offsets of scoreboard_entry_t's fields (a packed struct, the
/// first member highest), from its width: the exception is what is left.
struct Sbe {
    ex: usize,
}

impl Sbe {
    fn of(width: u32) -> Sbe {
        Sbe { ex: width as usize - 186 }
    }
    // (From the least significant: six flags, the branch prediction (43
    // bits: address, cf, valid), the exception, use_pc, use_zimm, use_imm,
    // valid, result, rd, rs2, rs1, op, fu, trans_id, pc.)
    const BP_VALID: usize = 48;
    const EX: usize = 49;
    fn use_pc(&self) -> usize {
        Self::EX + self.ex
    }
    fn result(&self) -> usize {
        self.use_pc() + 4
    }
    fn rd(&self) -> usize {
        self.result() + 64
    }
    fn op(&self) -> usize {
        self.rd() + 15
    }
    fn fu(&self) -> usize {
        self.op() + 8
    }
    fn pc(&self) -> usize {
        self.fu() + 7
    }
}

/// A register number, the low ones more often (so that instructions
/// depend on each other).
fn reg(rng: &mut Rng) -> u64 {
    if rng.next() % 2 == 0 { rng.next() % 6 } else { rng.next() % 32 }
}

#[test]
fn cva6_issue_stage() {
    let Some((cva6, (t, triple, mattr))) = setup() else { return };
    let (files, incs) = slice_files(
        &cva6,
        "issue_top",
        include_str!("cva6/issue_top.sv"),
        &[
            "core/cvxif_issue_register_commit_if_driver.sv",
            "core/raw_checker.sv",
            "core/ariane_regfile_ff.sv",
            "core/scoreboard.sv",
            "core/issue_read_operands.sv",
            "core/issue_stage.sv",
        ],
    );
    let b = build("issue_stage", netlist_of(files.clone(), incs.clone(), "issue_top"), &t, triple, &mattr);
    report("issue_stage", &b);
    let Some(v) = verilate("issue_stage", &files, &incs, "issue_top", &b.net.ports, Some("clk_i")) else { return };
    let sbe = Sbe::of(b.net.port("decoded_instr_i").unwrap().width);
    let ex = sbe.ex;
    let (cycles, n) = (200, GROUP);
    let mut rng = Rng(0x155E_57A6_E000_0001);
    // Random, with flushes, stalls and exceptions now and then.
    let stim = stimulus_with(&v, cycles, n, &mut rng, &mut |name, c, rng, w| match name {
        "rst_ni" => w[0] = (c > 0 && rng.next() % 64 != 0) as u32,
        "flush_i" => w[0] = (rng.next() % 16 == 0) as u32,
        "flush_unissued_instr_i" | "stall_i" => w[0] = (rng.next() % 8 == 0) as u32,
        "decoded_instr_i" => {
            set_bits(w, Sbe::EX + 1, 1, (rng.next() % 8 == 0) as u64);
            set_bits(w, sbe.fu(), 4, rng.next() % 12);
            set_bits(w, sbe.rd(), 15, reg(rng) | reg(rng) << 5 | reg(rng) << 10);
        }
        "ex_ex_i" => {
            for k in 0..5 {
                set_bits(w, k * ex + 1, 1, (rng.next() % 8 == 0) as u64);
            }
        }
        "waddr_i" => w[0] = (reg(rng) | reg(rng) << 5) as u32,
        _ => {}
    });
    check_against(&b, &v, cycles, n, &stim, &|w| format!("{w:x?}"));
    let ns = time_replay(&b, &v, cycles, n, &stim, 3);
    eprintln!("issue_stage: {n} instances, {cycles} cycles agree with Verilator; {ns:.2} ns an instance a cycle; Verilator {}", v.bench(cycles, n, &stim, 3));
    // A program's flow: instructions mostly for the ALU, loads, stores and
    // branches, results written back and committed now and then, no
    // exceptions, flushes rare.
    let typ = stimulus_with(&v, cycles, n, &mut rng, &mut |name, c, rng, w| match name {
        "rst_ni" => w[0] = (c > 0) as u32,
        "flush_i" => w[0] = (rng.next() % 256 == 0) as u32,
        "flush_unissued_instr_i" => w[0] = (rng.next() % 64 == 0) as u32,
        "stall_i" | "fpu_early_valid_i" | "x_we_i" => w[0] = 0,
        "decoded_instr_valid_i" | "lsu_ready_i" => w[0] = (rng.next() % 8 != 0) as u32,
        "flu_ready_i" | "fpu_ready_i" | "xfu_ready_i" | "x_issue_ready_i" | "x_register_ready_i" => w[0] = 1,
        "decoded_instr_i" => {
            w.fill(0);
            let fu = [3, 3, 3, 3, 1, 1, 2, 4, 4, 5, 6, 7, 0][rng.next() as usize % 13];
            set_bits(w, sbe.pc(), 39, rng.next() & 0x7F_FFFF_FFFE);
            set_bits(w, sbe.fu(), 4, fu);
            set_bits(w, sbe.op(), 8, rng.next() % 226);
            set_bits(w, sbe.rd(), 15, reg(rng) | reg(rng) << 5 | reg(rng) << 10);
            set_bits(w, sbe.result(), 64, ((rng.next() as i64) >> 52) as u64);
            set_bits(w, sbe.use_pc(), 3, [0, 0, 4, 4, 1][rng.next() as usize % 5]);
        }
        "orig_instr_i" => w[0] = rv_instruction_typical(rng),
        "is_ctrl_flow_i" => w[0] = 0,
        "x_issue_resp_i" => w[0] = 0,
        "hart_id_i" => w.fill(0),
        "wt_valid_i" => w[0] = (0..4).map(|k| ((rng.next() % 4 == 0) as u32) << k).sum(),
        "ex_ex_i" | "resolved_branch_i" => w.fill(0),
        "resolve_branch_i" => w[0] = (rng.next() % 16 == 0) as u32,
        "waddr_i" => w[0] = (reg(rng) | reg(rng) << 5) as u32,
        "we_gpr_i" => w[0] = (rng.next() % 3 == 0) as u32 | ((rng.next() % 8 == 0) as u32) << 1,
        "we_fpr_i" => w[0] = 0,
        "commit_ack_i" => w[0] = if rng.next() % 2 == 0 { 1 | ((rng.next() % 4 == 0) as u32) << 1 } else { 0 },
        _ => {}
    });
    let _ = Sbe::BP_VALID;
    check_against(&b, &v, cycles, n, &typ, &|w| format!("{w:x?}"));
    let ns = time_replay(&b, &v, cycles, n, &typ, 3);
    eprintln!("issue_stage, typical: {ns:.2} ns an instance a cycle; Verilator {}", v.bench(cycles, n, &typ, 3));
}
