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
