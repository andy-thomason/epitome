//! The units of the CVA6 RISC-V core that epitome-sv's netlist compiles,
//! in the cv64a6_imafdc_sv39 configuration: their files in a CVA6
//! checkout, the wrapper each is built under (beside this file), and the
//! stimulus each is run on. Shared by the tests (`sv/tests/cva6.rs`,
//! `netlist.rs`, `lanes.rs`) and the example (`sv/examples/cva6.rs`).
//!
//! Only names of CVA6's files are here, never its source: the checkout is
//! fetched at run time. The wrappers are Epitome's own, written from the
//! units' ports.
#![allow(dead_code)]

use std::path::Path;

use epitome_sv::model::{Io, Sources};
use epitome_sv::netlist::GROUP;
use epitome_sv::stimulus::{Rng, frames, set_bits, stimulus, stimulus_with};

/// The CVA6 commit the units are checked on (2026-10-23). Pinned because
/// the wrappers repeat types of `cva6.sv` (`slice_pkg.sv`) and the stimulus
/// knows the bit offsets of its structs and the numbers of its enums
/// (`fu_op`), any of which a later commit may change.
pub const COMMIT: &str = "742dc82a0c706e0a3bfb98b709fd3ee06d50d12a";

/// The configuration's packages, and the include directories, then `rest`
/// (paths in the checkout).
pub fn cva6_sources(cva6: &Path, rest: &[&str]) -> Sources {
    let mut src = Sources::default();
    for p in [
        "core/include/config_pkg.sv",
        "core/include/cv64a6_imafdc_sv39_config_pkg.sv",
        "core/include/riscv_pkg.sv",
        "core/include/build_config_pkg.sv",
        "core/include/ariane_pkg.sv",
        "vendor/pulp-platform/common_cells/src/cf_math_pkg.sv",
    ]
    .iter()
    .chain(rest)
    {
        src.file(cva6.join(p));
    }
    src.include_dir(cva6.join("core/include")).include_dir(cva6.join("vendor/pulp-platform/common_cells/include"));
    src
}

/// Stimulus of a kind.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mix {
    /// Random, with the edges and rare cases often.
    Random,
    /// As a program's run has them: most arms taken by no instance.
    Typical,
}

/// A unit and how it is built and run.
pub struct Unit {
    pub name: &'static str,
    pub about: &'static str,
    /// The wrapper's module, the top.
    pub top: &'static str,
    /// The wrapper's file name and text.
    pub wrapper: (&'static str, &'static str),
    /// Whether the wrapper imports `slice_pkg`.
    pub slice_pkg: bool,
    /// CVA6's files besides the configuration's packages.
    pub files: &'static [&'static str],
    pub clock: Option<&'static str>,
    /// Cycles the tests run, and instances.
    pub cycles: usize,
    pub instances: usize,
    pub typical: bool,
    pub seed: u64,
}

/// The types of cva6.sv, for wrappers to import.
pub const SLICE_PKG: &str = include_str!("slice_pkg.sv");

const LZC: &str = "vendor/pulp-platform/common_cells/src/lzc.sv";
const POPCOUNT: &str = "vendor/pulp-platform/common_cells/src/popcount.sv";

pub const UNITS: &[Unit] = &[
    Unit {
        name: "pmp_entry",
        about: "one physical memory protection entry",
        top: "pmp_entry_top",
        wrapper: ("pmp_entry_top.sv", include_str!("pmp_entry_top.sv")),
        slice_pkg: false,
        files: &[LZC, "core/pmp/src/pmp_entry.sv"],
        clock: None,
        cycles: 1,
        instances: GROUP,
        typical: false,
        seed: 0x9E37_79B9_7F4A_7C15,
    },
    Unit {
        name: "pmp",
        about: "the PMP unit: eight pmp_entry",
        top: "pmp_top",
        wrapper: ("pmp_top.sv", include_str!("pmp_top.sv")),
        slice_pkg: false,
        files: &[LZC, "core/pmp/src/pmp_entry.sv", "core/pmp/src/pmp.sv"],
        clock: None,
        cycles: 1,
        instances: GROUP,
        typical: false,
        seed: 0x0DDB_1A5E_5BAD_5EED,
    },
    Unit {
        name: "alu",
        about: "the ALU, with common_cells' popcount and lzc",
        top: "alu_top",
        wrapper: ("alu_top.sv", include_str!("alu_top.sv")),
        slice_pkg: false,
        files: &[LZC, POPCOUNT, "core/alu.sv"],
        clock: None,
        cycles: 1,
        instances: 4 * GROUP,
        typical: false,
        seed: 0xA1B2_C3D4_E5F6_0718,
    },
    Unit {
        name: "exec",
        about: "alu_wrapper and branch_unit, wired as ex_stage wires them",
        top: "exec_top",
        wrapper: ("exec_top.sv", include_str!("exec_top.sv")),
        slice_pkg: false,
        files: &[LZC, POPCOUNT, "core/alu.sv", "core/alu_wrapper.sv", "core/branch_unit.sv"],
        clock: None,
        cycles: 1,
        instances: 4 * GROUP,
        typical: false,
        seed: 0x0E1E_C0DE_5EED_0001,
    },
    Unit {
        name: "cdec",
        about: "the compressed decoder",
        top: "cdec_top",
        wrapper: ("cdec_top.sv", include_str!("cdec_top.sv")),
        slice_pkg: true,
        files: &["core/compressed_decoder.sv"],
        clock: None,
        cycles: 1,
        instances: 8 * GROUP,
        typical: false,
        seed: 0xC0DE_C0DE_0000_0016,
    },
    Unit {
        name: "decoder",
        about: "the decoder",
        top: "decoder_top",
        wrapper: ("decoder_top.sv", include_str!("decoder_top.sv")),
        slice_pkg: true,
        files: &["core/decoder.sv"],
        clock: None,
        cycles: 1,
        instances: 8 * GROUP,
        typical: true,
        seed: 0xDEC0_DE00_1234_5678,
    },
    Unit {
        name: "id_stage",
        about: "the ID stage: compressed decoder, CV-X-IF driver, decoder, issue register",
        top: "id_top",
        wrapper: ("id_top.sv", include_str!("id_top.sv")),
        slice_pkg: true,
        files: &["core/compressed_decoder.sv", "core/decoder.sv", "core/cvxif_compressed_if_driver.sv", "core/id_stage.sv"],
        clock: Some("clk_i"),
        cycles: 60,
        instances: GROUP,
        typical: true,
        seed: 0x1D57_A6E0_0000_0001,
    },
    Unit {
        name: "issue_stage",
        about: "the issue stage: scoreboard, operand read, register files",
        top: "issue_top",
        wrapper: ("issue_top.sv", include_str!("issue_top.sv")),
        slice_pkg: true,
        files: &[
            "core/cvxif_issue_register_commit_if_driver.sv",
            "core/raw_checker.sv",
            "core/ariane_regfile_ff.sv",
            "core/scoreboard.sv",
            "core/issue_read_operands.sv",
            "core/issue_stage.sv",
        ],
        clock: Some("clk_i"),
        cycles: 200,
        instances: GROUP,
        typical: true,
        seed: 0x155E_57A6_E000_0001,
    },
    Unit {
        name: "mult",
        about: "the multiply/divide unit: multiplier and serdiv",
        top: "mult_top",
        wrapper: ("mult_top.sv", include_str!("mult_top.sv")),
        slice_pkg: true,
        files: &[LZC, "core/serdiv.sv", "core/multiplier.sv", "core/mult.sv"],
        clock: Some("clk_i"),
        cycles: 200,
        instances: GROUP,
        typical: true,
        seed: 0x3017_0000_0000_0001,
    },
    Unit {
        name: "serdiv",
        about: "the serial divider",
        top: "serdiv_top",
        wrapper: ("serdiv_top.sv", include_str!("serdiv_top.sv")),
        slice_pkg: true,
        files: &[LZC, "core/serdiv.sv"],
        clock: Some("clk_i"),
        cycles: 300,
        instances: GROUP,
        typical: true,
        seed: 0x5E2D_1700_0000_0001,
    },
];

/// The unit named `name`.
pub fn unit(name: &str) -> &'static Unit {
    UNITS.iter().find(|u| u.name == name).unwrap_or_else(|| panic!("no unit {name}"))
}

impl Unit {
    /// Its sources: CVA6's packages and files, `slice_pkg` if it wants it,
    /// and its wrapper.
    pub fn sources(&self, cva6: &Path) -> Sources {
        let mut src = cva6_sources(cva6, self.files);
        if self.slice_pkg {
            // (Before the files: the wrapper's types are the package's.)
            let at = 6;
            src.items.insert(at, epitome_sv::model::Source::Text { name: "slice_pkg.sv".into(), text: SLICE_PKG.into() });
        }
        src.text(self.wrapper.0, self.wrapper.1);
        src
    }

    /// The kinds of stimulus it has.
    pub fn mixes(&self) -> &'static [Mix] {
        if self.typical { &[Mix::Random, Mix::Typical] } else { &[Mix::Random] }
    }

    /// Stimulus of kind `mix` for `cycles` cycles of `n` instances.
    pub fn stimulus(&self, mix: Mix, io: &Io, cycles: usize, n: usize, rng: &mut Rng) -> Vec<u32> {
        let typical = mix == Mix::Typical;
        match self.name {
            "pmp_entry" => frames(io, cycles, n, &mut |_, _, f| {
                let (addr, conf, prev, mode) = pmp_entry_case(rng);
                f.set("addr_i", addr);
                f.set("conf_addr_i", conf);
                f.set("conf_addr_prev_i", prev);
                f.set("conf_addr_mode_i", mode);
            }),
            "pmp" => frames(io, cycles, n, &mut |_, _, f| {
                let p = pmp_case(rng);
                f.set("addr_i", p.addr);
                f.set("access_type_i", p.access);
                f.set("priv_lvl_i", p.priv_lvl);
                // conf_addr_i[i] is bits 54i up.
                let mut wide = vec![0u32; 14];
                for (i, &a) in p.conf_addr.iter().enumerate() {
                    set_bits(&mut wide, 54 * i, 54, a);
                }
                let wide: Vec<u64> = wide.chunks(2).map(|w| w[0] as u64 | (w[1] as u64) << 32).collect();
                f.set_wide("conf_addr_i", &wide);
                f.set("conf_i", p.conf.iter().enumerate().fold(0, |w, (i, &c)| w | c << (8 * i)));
            }),
            "alu" => stimulus(io, cycles, n, rng, &mut |name, _, rng| match name {
                // Mostly the operations there are (the enum has about 225).
                "operation_i" => Some(if rng.next() % 16 == 0 { rng.next() % 256 } else { rng.next() % 226 }),
                "a_i" | "b_i" => Some(operand(rng)),
                _ => None,
            }),
            "exec" => frames(io, cycles, n, &mut |_, _, f| {
                // Half branches and jumps (LTS..JALR are 13..19).
                let op = if rng.next() % 2 == 0 { 13 + rng.next() % 7 } else { rng.next() % 226 };
                let (a, b) = (operand(rng), operand(rng));
                let imm = ((rng.next() as i64) >> (40 + rng.next() % 24)) as u64;
                let pc = rng.next() & !1;
                let target = if op == 19 { a.wrapping_add(imm) & !1 } else { pc.wrapping_add(imm) };
                let predicted = if rng.next() % 2 == 0 { target } else { rng.next() };
                for (p, v) in [("operation_i", op), ("a_i", a), ("b_i", b), ("imm_i", imm), ("pc_i", pc)] {
                    f.set(p, v);
                }
                f.set("is_compressed_i", rng.next() % 2);
                f.set("branch_valid_i", (rng.next() % 4 != 0) as u64);
                f.set("predict_valid_i", rng.next() % 2);
                f.set("predict_cf_i", rng.next() % 5);
                f.set("predict_address_i", predicted);
            }),
            // Mostly compressed: the low two bits not 11.
            "cdec" => stimulus(io, cycles, n, rng, &mut |_, _, rng| {
                let x = rng.next();
                Some(if x % 8 == 0 { x | 3 } else if x & 3 == 3 { x & !1 } else { x })
            }),
            "decoder" if typical => stimulus(io, cycles, n, rng, &mut |name, _, rng| match name {
                "instruction_i" => Some(rv_instruction_typical(rng) as u64),
                "is_compressed_i" | "is_illegal_i" | "debug_req_i" | "debug_mode_i" | "ex_i" | "irq_i" => Some(0),
                _ => None,
            }),
            "decoder" => stimulus(io, cycles, n, rng, &mut |name, _, rng| match name {
                "instruction_i" => Some(rv_instruction(rng) as u64),
                "is_compressed_i" | "is_illegal_i" | "debug_req_i" | "debug_mode_i" => Some((rng.next() % 8 == 0) as u64),
                // No exception from fetch, mostly.
                "ex_i" => Some(rng.next() & !(((rng.next() % 4 != 0) as u64) << 1)),
                _ => None,
            }),
            "id_stage" => {
                // fetch_entry_t: address, instruction, branch prediction (68
                // bits), then the exception, whose valid bit is bit 1.
                let fe = io.ins.iter().find(|p| p.name == "fetch_entry_i").unwrap().width as usize;
                let ex = fe - 64 - 32 - 68;
                stimulus_with(io, cycles, n, rng, &mut |name, c, rng, w| match name {
                    "rst_ni" if typical => w[0] = (c > 0) as u32,
                    "rst_ni" => w[0] = (c > 0 && rng.next() % 64 != 0) as u32,
                    // A program's mix, uncompressed, with no flushes, debug
                    // or exceptions.
                    "flush_i" | "debug_req_i" | "debug_mode_i" | "irq_i" | "priv_lvl_i" if typical => w[0] = 0,
                    "fetch_entry_valid_i" | "issue_instr_ack_i" | "compressed_ready_i" if typical => {
                        w[0] = (rng.next() % 8 != 0) as u32
                    }
                    "fetch_entry_i" if typical => {
                        set_bits(w, ex + 68, 32, rv_instruction_typical(rng) as u64);
                        set_bits(w, 0, ex, 0);
                    }
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
                })
            }
            "issue_stage" => issue_stimulus(typical, io, cycles, n, rng),
            "mult" if typical => stimulus(io, cycles, n, rng, &mut |name, c, rng| match name {
                // A request now and then, mostly MUL, divisions rare.
                "rst_ni" => Some((c > 0) as u64),
                "flush_i" => Some(0),
                "mult_valid_i" => Some((rng.next() % 16 == 0) as u64),
                "operation_i" => Some(if rng.next() % 32 == 0 { 92 + rng.next() % 8 } else { 87 }),
                "a_i" | "b_i" => Some(operand(rng)),
                _ => None,
            }),
            // MUL..REMUW are 87..99, CLMUL..CLMULR 159..161.
            "mult" => stimulus(io, cycles, n, rng, &mut |name, c, rng| match name {
                "rst_ni" => Some((c > 0 && rng.next() % 512 != 0) as u64),
                "flush_i" => Some((rng.next() % 64 == 0) as u64),
                "mult_valid_i" => Some((rng.next() % 3 == 0) as u64),
                "operation_i" => Some(if rng.next() % 4 == 0 { 159 + rng.next() % 3 } else { 87 + rng.next() % 13 }),
                "a_i" | "b_i" => Some(operand(rng)),
                _ => None,
            }),
            "serdiv" if typical => stimulus(io, cycles, n, rng, &mut |name, c, rng| match name {
                // A division now and then, results taken at once.
                "rst_ni" => Some((c > 0) as u64),
                "flush_i" => Some(0),
                "in_vld_i" => Some((rng.next() % 64 == 0) as u64),
                "out_rdy_i" => Some(1),
                "op_a_i" | "op_b_i" => Some(operand(rng)),
                _ => None,
            }),
            "serdiv" => stimulus(io, cycles, n, rng, &mut |name, c, rng| match name {
                "rst_ni" => Some((c > 0 && rng.next() % 512 != 0) as u64),
                "in_vld_i" => Some((rng.next() % 4 == 0) as u64),
                "out_rdy_i" => Some((rng.next() % 8 != 0) as u64),
                "flush_i" => Some((rng.next() % 128 == 0) as u64),
                "op_a_i" | "op_b_i" => Some(operand(rng)),
                _ => None,
            }),
            other => panic!("no stimulus for {other}"),
        }
    }
}

/// A 64-bit operand: random, small, or at an edge.
pub fn operand(rng: &mut Rng) -> u64 {
    match rng.next() % 8 {
        0 => rng.next() % 70,
        1 => [0, 1, !0, 1 << 63, (1 << 63) - 1, 0xFFFF_FFFF, 0x8000_0000, 0x7FFF_FFFF][rng.next() as usize % 8],
        2 => rng.next() >> (rng.next() % 64),
        3 => !(rng.next() >> (rng.next() % 64)),
        _ => rng.next(),
    }
}

/// A random RISC-V instruction word: mostly the major opcodes there are,
/// with the usual function codes.
pub fn rv_instruction(rng: &mut Rng) -> u32 {
    const OPS: [u32; 24] = [
        0x03, 0x07, 0x0F, 0x13, 0x17, 0x1B, 0x23, 0x27, 0x2F, 0x33, 0x37, 0x3B, 0x43, 0x47, 0x4B, 0x4F, 0x53, 0x63, 0x67, 0x6F, 0x73,
        0x33, 0x13, 0x73,
    ];
    let mut x = rng.next() as u32;
    if rng.next() % 8 != 0 {
        x = (x & !0x7F) | OPS[rng.next() as usize % OPS.len()];
    }
    match rng.next() % 4 {
        // funct7 of the base, M, and bit-manipulation instructions.
        0 => x = (x & 0x01FF_FFFF) | [0x00, 0x20, 0x01, 0x04, 0x05, 0x14, 0x24, 0x30, 0x34][rng.next() as usize % 9] << 25,
        1 => x &= !(0x1F << 20),
        _ => {}
    }
    x
}

/// A RISC-V instruction word as a program's mix has them: mostly integer
/// arithmetic, loads, stores and branches; floating point, atomics and
/// system instructions one in a hundred.
pub fn rv_instruction_typical(rng: &mut Rng) -> u32 {
    let r = rng.next() % 100;
    let op = match r {
        0..30 => 0x13,  // OP-IMM
        30..45 => 0x03, // LOAD
        45..55 => 0x23, // STORE
        55..70 => 0x63, // BRANCH
        70..85 => 0x33, // OP
        85..90 => [0x37, 0x17][rng.next() as usize % 2],
        90..95 => [0x6F, 0x67][rng.next() as usize % 2],
        95..99 => [0x1B, 0x3B][rng.next() as usize % 2],
        _ => [0x07, 0x27, 0x2F, 0x53, 0x73, 0x0F, 0x43][rng.next() as usize % 7],
    };
    let mut x = (rng.next() as u32 & !0x7F) | op;
    // funct7 of the base instructions mostly; M and B now and then.
    if matches!(op, 0x33 | 0x3B) {
        x = (x & 0x01FF_FFFF) | if rng.next() % 8 == 0 { [0x01, 0x05, 0x20][rng.next() as usize % 3] } else { 0 } << 25;
    }
    x
}

/// A `pmp_entry` case: an address, the entry's address and the one before
/// it, and the mode; configurations random, addresses often inside the
/// region (as in `examples/soa_pmp_entry.rs`).
pub fn pmp_entry_case(rng: &mut Rng) -> (u64, u64, u64, u64) {
    let (m56, mcw) = ((1u64 << 56) - 1, (1u64 << 54) - 1);
    let mode = rng.next() % 4;
    let t = rng.next() % 55;
    let conf = if mode == 3 { (rng.next() << (t + 1) | ((1 << t) - 1)) & mcw } else { rng.next() & mcw };
    let prev = rng.next() & mcw;
    let (lo, hi) = (((prev & !1) << 2) & m56, ((conf & !1) << 2) & m56);
    let addr = match rng.next() % 3 {
        0 => rng.next() & m56,
        1 if mode == 3 => ((conf << 2) & m56) ^ (rng.next() & ((1u64 << (t + 3).min(56)) - 1)),
        _ if hi > lo => lo + rng.next() % (hi - lo),
        _ => rng.next() & m56,
    };
    (addr, conf, prev, mode)
}

/// A case of the PMP unit.
#[derive(Clone, Debug)]
pub struct PmpCase {
    pub addr: u64,
    pub access: u64,
    pub priv_lvl: u64,
    pub conf_addr: [u64; 8],
    pub conf: [u64; 8],
}

/// Ascending TOR bounds and NAPOT regions, mixed, so that entries match,
/// overlap and miss, and addresses near them.
pub fn pmp_case(rng: &mut Rng) -> PmpCase {
    let (m56, mcw) = ((1u64 << 56) - 1, (1u64 << 54) - 1);
    let (mut conf_addr, mut conf) = ([0u64; 8], [0u64; 8]);
    let mut at = rng.next() % (1 << 30);
    for i in 0..8 {
        let mode = rng.next() % 4;
        conf_addr[i] = if mode == 3 {
            let t = rng.next() % 20;
            ((at >> t << t) | ((1 << t) - 1) >> 1) & mcw
        } else {
            at = (at + rng.next() % (1 << 24)) & mcw;
            at
        };
        conf[i] = (rng.next() % 4 == 0) as u64 * 0x80 | mode << 3 | rng.next() % 8;
    }
    let pick = rng.next() as usize % 8;
    let addr = match rng.next() % 3 {
        0 => rng.next() & m56,
        _ => ((conf_addr[pick] << 2) + rng.next() % (1 << 12)) & m56,
    };
    let access = [1u64, 2, 4][rng.next() as usize % 3];
    PmpCase { addr, access, priv_lvl: rng.next() % 4, conf_addr, conf }
}

/// Bit offsets of scoreboard_entry_t's fields (a packed struct, the first
/// member highest), from its width: the exception is what is left.
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

fn issue_stimulus(typical: bool, io: &Io, cycles: usize, n: usize, rng: &mut Rng) -> Vec<u32> {
    let sbe = Sbe::of(io.ins.iter().find(|p| p.name == "decoded_instr_i").unwrap().width);
    let ex = sbe.ex;
    if !typical {
        // Random, with flushes, stalls and exceptions now and then.
        return stimulus_with(io, cycles, n, rng, &mut |name, c, rng, w| match name {
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
    }
    // A program's flow: instructions mostly for the ALU, loads, stores and
    // branches, results written back and committed now and then, no
    // exceptions, flushes rare.
    stimulus_with(io, cycles, n, rng, &mut |name, c, rng, w| match name {
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
        "is_ctrl_flow_i" | "x_issue_resp_i" | "we_fpr_i" => w[0] = 0,
        "hart_id_i" | "ex_ex_i" | "resolved_branch_i" => w.fill(0),
        "wt_valid_i" => w[0] = (0..4).map(|k| ((rng.next() % 4 == 0) as u32) << k).sum(),
        "resolve_branch_i" => w[0] = (rng.next() % 16 == 0) as u32,
        "waddr_i" => w[0] = (reg(rng) | reg(rng) << 5) as u32,
        "we_gpr_i" => w[0] = (rng.next() % 3 == 0) as u32 | ((rng.next() % 8 == 0) as u32) << 1,
        "commit_ack_i" => w[0] = if rng.next() % 2 == 0 { 1 | ((rng.next() % 4 == 0) as u32) << 1 } else { 0 },
        _ => {}
    })
}
