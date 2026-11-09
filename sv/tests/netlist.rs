//! The compiled netlist (`src/netlist.rs`), built by Epitome, loaded and
//! run, and checked instance by instance: a module of operators against
//! values worked out here, and CVA6's `pmp_entry` (when `CVA6_DIR` points
//! at a CVA6 checkout) against a scalar model of it.
//!
//! On an AArch64 Mac or x86-64 Linux with `cc`; elsewhere the tests skip.
//! x86-64 uses AVX2 unless `NETLIST_MATTR` names other features.

mod common;

use std::path::PathBuf;
use std::time::Instant;

use common::*;
use epitome_sv::netlist::{self, GROUP};

const OPS: &str = r#"
module ops (
    input  logic [55:0] a,
    input  logic [55:0] b,
    input  logic [5:0]  s,
    input  logic [1:0]  m,
    input  logic signed [15:0] sa,
    input  logic signed [15:0] sb,
    output logic [55:0] sum,
    output logic [55:0] diff,
    output logic [55:0] bits,
    output logic [55:0] shl,
    output logic [55:0] shr,
    output logic signed [15:0] sar,
    output logic        lt,
    output logic        eq,
    output logic        slt,
    output logic        red,
    output logic [7:0]  cat,
    output logic [5:0]  sel,
    output logic [55:0] mux,
    output logic [55:0] each,
    output logic [7:0]  pri
);
  function automatic logic odd(input logic [55:0] x);
    return x[0];
  endfunction
  assign sum = a + b;
  assign diff = a - b;
  assign bits = (a & b) | (~a ^ b);
  assign shl = a << s;
  assign shr = a >> s;
  assign sar = sa >>> s[3:0];
  assign lt = a < b;
  assign eq = a == b;
  assign slt = sa < sb;
  assign red = ^a;
  assign cat = {a[3:0], b[55:52]};
  assign sel = {a[s], b[s +: 5]};
  always_comb begin
    case (m)
      2'd0: mux = a;
      2'd1: mux = b;
      2'd2: mux = a ^ b;
      default: mux = '0;
    endcase
    each = '0;
    for (int i = 0; i < 8; i++) each[i*7 +: 7] = a[i*7 +: 7] + 7'd1;
    // A chain whose conditions are made after its arms, one of them a
    // function's result.
    pri = 8'd0;
    if (a[3:0] == 4'd1) pri = 8'd1;
    else if (odd(b)) pri = 8'd2;
    else if (m == 2'd3) pri = {2'b0, s};
  end
endmodule
"#;

#[test]
fn operators_lane_by_lane() {
    let Some((t, triple, mattr)) = host() else {
        eprintln!("needs cc on an AArch64 Mac or x86-64 Linux; skipping");
        return;
    };
    let tmp = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("netlist");
    std::fs::create_dir_all(&tmp).unwrap();
    let src = tmp.join("ops.sv");
    std::fs::write(&src, OPS).unwrap();
    let b = build("ops", netlist_of(vec![src], Vec::new(), "ops"), &t, triple, &mattr);
    report("ops", &b);
    let net = &b.net;
    let m56 = (1u64 << 56) - 1;
    let mut rng = Rng(0x2545_F491_4F6C_DD1D);
    let mut st = State::new(net.size);
    for round in 0..4 {
        let mut ins = Vec::with_capacity(GROUP);
        for n in 0..GROUP {
            let mut a = rng.next() & m56;
            let mut b = rng.next() & m56;
            match (round + n) % 5 {
                0 => b = a,
                1 => b = a ^ (1 << (rng.next() % 56)),
                2 => a = m56,
                _ => {}
            }
            let (s, m) = (rng.next() % 64, rng.next() % 4);
            let (sa, sb) = (rng.next() as u16, rng.next() as u16);
            for (p, v) in [("a", a), ("b", b), ("s", s), ("m", m), ("sa", sa as u64), ("sb", sb as u64)] {
                st.put(net, p, n, v);
            }
            ins.push((a, b, s, m, sa as i16, sb as i16));
        }
        unsafe { (b.eval)(st.ptr) };
        for (n, &(a, b2, s, m, sa, sb)) in ins.iter().enumerate() {
            let mut each = 0;
            for i in 0..8 {
                each |= ((((a >> (7 * i)) & 0x7F) + 1) & 0x7F) << (7 * i);
            }
            let want = [
                ("sum", (a + b2) & m56),
                ("diff", a.wrapping_sub(b2) & m56),
                ("bits", ((a & b2) | (!a ^ b2)) & m56),
                ("shl", if s < 56 { (a << s) & m56 } else { 0 }),
                ("shr", if s < 56 { a >> s } else { 0 }),
                ("sar", (sa >> (s & 15)) as u16 as u64),
                ("lt", (a < b2) as u64),
                ("eq", (a == b2) as u64),
                ("slt", (sa < sb) as u64),
                ("red", (a.count_ones() & 1) as u64),
                ("cat", ((a & 0xF) << 4) | ((b2 >> 52) & 0xF)),
                ("sel", (if s < 56 { (a >> s) & 1 } else { 0 }) << 5 | if s < 56 { (b2 >> s) & 0x1F } else { 0 }),
                ("mux", [a, b2, a ^ b2, 0][m as usize]),
                ("each", each),
                ("pri", if a & 0xF == 1 { 1 } else if b2 & 1 == 1 { 2 } else if m == 3 { s } else { 0 }),
            ];
            for (port, w) in want {
                let got = st.get(net, port, n);
                assert_eq!(got, w, "instance {n}, {port}: a {a:x} b {b2:x} s {s} m {m} sa {sa} sb {sb}");
            }
        }
    }
}

/// `pmp_entry`'s `always_comb`, written out by hand from its source.
fn pmp_reference(addr: u64, conf: u64, prev: u64, mode: u64) -> bool {
    const PLEN: u32 = 56;
    const CW: u32 = PLEN - 2;
    let m56 = (1u64 << PLEN) - 1;
    match mode {
        1 => addr >= ((prev & !1) << 2) & m56 && addr < ((conf & !1) << 2) & m56,
        3 => {
            let conf_n = (0b11 << CW) | (!conf & ((1 << CW) - 1));
            let size = conf_n.trailing_zeros() + 3;
            let mask = (m56 << size) & m56;
            (addr & mask) == ((conf << 2) & mask)
        }
        _ => false,
    }
}

const PMP_TOP: &str = r#"
// pmp_entry with the cv64a6_imafdc_sv39 configuration (PLEN = 56).
module pmp_entry_top
  import cva6_config_pkg::*;
#(
    parameter config_pkg::cva6_cfg_t CVA6Cfg = build_config_pkg::build_config(cva6_config_pkg::cva6_cfg)
) (
    input logic [CVA6Cfg.PLEN-1:0] addr_i,
    input logic [CVA6Cfg.PLEN-3:0] conf_addr_i,
    input logic [CVA6Cfg.PLEN-3:0] conf_addr_prev_i,
    input riscv::pmp_addr_mode_t conf_addr_mode_i,
    output logic match_o
);
  pmp_entry #(.CVA6Cfg(CVA6Cfg)) i_entry (.*);
endmodule
"#;

#[test]
fn cva6_pmp_entry() {
    let Some(cva6) = std::env::var_os("CVA6_DIR").map(PathBuf::from) else {
        eprintln!("set CVA6_DIR to a CVA6 checkout to run this; skipping");
        return;
    };
    let Some((t, triple, mattr)) = host() else {
        eprintln!("needs cc on an AArch64 Mac or x86-64 Linux; skipping");
        return;
    };
    let tmp = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("netlist");
    std::fs::create_dir_all(&tmp).unwrap();
    std::fs::write(tmp.join("pmp_entry_top.sv"), PMP_TOP).unwrap();
    let c = |p: &str| cva6.join(p);
    let files = vec![
        c("core/include/config_pkg.sv"),
        c("core/include/cv64a6_imafdc_sv39_config_pkg.sv"),
        c("core/include/riscv_pkg.sv"),
        c("core/include/build_config_pkg.sv"),
        c("vendor/pulp-platform/common_cells/src/cf_math_pkg.sv"),
        c("vendor/pulp-platform/common_cells/src/lzc.sv"),
        c("core/pmp/src/pmp_entry.sv"),
        tmp.join("pmp_entry_top.sv"),
    ];
    let b = build("pmp_entry", netlist_of(files, vec![c("core/include")], "pmp_entry_top"), &t, triple, &mattr);
    report("pmp_entry", &b);
    let net = &b.net;
    // Random configurations, with addresses often inside the region, as in
    // examples/soa_pmp_entry.rs.
    let (m56, mcw) = ((1u64 << 56) - 1, (1u64 << 54) - 1);
    let mut rng = Rng(0x9E37_79B9_7F4A_7C15);
    let mut st = State::new(net.size);
    let mut ins = Vec::with_capacity(GROUP);
    for n in 0..GROUP {
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
        for (p, v) in [("addr_i", addr), ("conf_addr_i", conf), ("conf_addr_prev_i", prev), ("conf_addr_mode_i", mode)] {
            st.put(net, p, n, v);
        }
        ins.push((addr, conf, prev, mode));
    }
    unsafe { (b.eval)(st.ptr) };
    let mut matches = 0;
    for (n, &(addr, conf, prev, mode)) in ins.iter().enumerate() {
        let want = pmp_reference(addr, conf, prev, mode);
        assert_eq!(st.get(net, "match_o", n) == 1, want, "instance {n}: addr {addr:x} conf {conf:x} prev {prev:x} mode {mode}");
        matches += want as usize;
    }
    let iters = 2000;
    let t0 = Instant::now();
    for _ in 0..iters {
        unsafe { (b.eval)(std::hint::black_box(st.ptr)) };
    }
    let ns = t0.elapsed().as_secs_f64() * 1e9 / (iters * GROUP) as f64;
    eprintln!("pmp_entry: {GROUP} instances agree with the model ({matches} match); {ns:.2} ns an instance");
}

const PMP_UNIT_TOP: &str = r#"
// The PMP unit with the cv64a6_imafdc_sv39 configuration: 8 entries.
module pmp_top
  import ariane_pkg::*;
#(
    parameter config_pkg::cva6_cfg_t CVA6Cfg = build_config_pkg::build_config(cva6_config_pkg::cva6_cfg)
) (
    input logic [CVA6Cfg.PLEN-1:0] addr_i,
    input riscv::pmp_access_t access_type_i,
    input riscv::priv_lvl_t priv_lvl_i,
    input logic [CVA6Cfg.NrPMPEntries-1:0][CVA6Cfg.PLEN-3:0] conf_addr_i,
    input riscv::pmpcfg_t [CVA6Cfg.NrPMPEntries-1:0] conf_i,
    output logic allow_o
);
  pmp #(.CVA6Cfg(CVA6Cfg)) i_pmp (.*);
endmodule
"#;

/// `pmp`'s `always_comb`: the first entry that applies and matches decides;
/// with none, only machine mode may access.
fn pmp_unit_reference(addr: u64, access: u64, priv_lvl: u64, conf_addr: &[u64; 8], conf: &[u64; 8]) -> bool {
    for i in 0..8 {
        let (locked, mode, perm) = (conf[i] >> 7 & 1, conf[i] >> 3 & 3, conf[i] & 7);
        let prev = if i == 0 { 0 } else { conf_addr[i - 1] };
        if (priv_lvl != 3 || locked == 1) && pmp_reference(addr, conf_addr[i], prev, mode) {
            return access & perm == access;
        }
    }
    priv_lvl == 3
}

#[test]
fn cva6_pmp() {
    let Some(cva6) = std::env::var_os("CVA6_DIR").map(PathBuf::from) else {
        eprintln!("set CVA6_DIR to a CVA6 checkout to run this; skipping");
        return;
    };
    let Some((t, triple, mattr)) = host() else {
        eprintln!("needs cc on an AArch64 Mac or x86-64 Linux; skipping");
        return;
    };
    let tmp = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("netlist");
    std::fs::create_dir_all(&tmp).unwrap();
    std::fs::write(tmp.join("pmp_top.sv"), PMP_UNIT_TOP).unwrap();
    let c = |p: &str| cva6.join(p);
    let files = vec![
        c("core/include/config_pkg.sv"),
        c("core/include/cv64a6_imafdc_sv39_config_pkg.sv"),
        c("core/include/riscv_pkg.sv"),
        c("core/include/build_config_pkg.sv"),
        c("core/include/ariane_pkg.sv"),
        c("vendor/pulp-platform/common_cells/src/cf_math_pkg.sv"),
        c("vendor/pulp-platform/common_cells/src/lzc.sv"),
        c("core/pmp/src/pmp_entry.sv"),
        c("core/pmp/src/pmp.sv"),
        tmp.join("pmp_top.sv"),
    ];
    let incs = vec![c("core/include"), c("vendor/pulp-platform/common_cells/include")];
    let b = build("pmp", netlist_of(files, incs, "pmp_top"), &t, triple, &mattr);
    report("pmp", &b);
    let net = &b.net;
    let (m56, mcw) = ((1u64 << 56) - 1, (1u64 << 54) - 1);
    let mut rng = Rng(0x0DDB_1A5E_5BAD_5EED);
    let mut st = State::new(net.size);
    let mut ins = Vec::with_capacity(GROUP);
    for n in 0..GROUP {
        // Ascending TOR bounds and NAPOT regions, mixed, so that entries
        // match, overlap and miss.
        let mut conf_addr = [0u64; 8];
        let mut conf = [0u64; 8];
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
        let priv_lvl = rng.next() % 4;
        st.put(net, "addr_i", n, addr);
        st.put(net, "access_type_i", n, access);
        st.put(net, "priv_lvl_i", n, priv_lvl);
        // conf_addr_i[i] is bits 54i up.
        let mut wide = [0u64; 7];
        for (i, &a) in conf_addr.iter().enumerate() {
            let at = 54 * i;
            wide[at / 64] |= a << (at % 64);
            if at % 64 + 54 > 64 {
                wide[at / 64 + 1] |= a >> (64 - at % 64);
            }
        }
        st.put_wide(net, "conf_addr_i", n, &wide);
        st.put(net, "conf_i", n, conf.iter().enumerate().fold(0, |w, (i, &c)| w | c << (8 * i)));
        ins.push((addr, access, priv_lvl, conf_addr, conf));
    }
    unsafe { (b.eval)(st.ptr) };
    let mut allowed = 0;
    for (n, (addr, access, priv_lvl, conf_addr, conf)) in ins.iter().enumerate() {
        let want = pmp_unit_reference(*addr, *access, *priv_lvl, conf_addr, conf);
        assert_eq!(
            st.get(net, "allow_o", n) == 1,
            want,
            "instance {n}: addr {addr:x} access {access} priv {priv_lvl} conf_addr {conf_addr:x?} conf {conf:x?}"
        );
        allowed += want as usize;
    }
    let iters = 200;
    let t0 = Instant::now();
    for _ in 0..iters {
        unsafe { (b.eval)(std::hint::black_box(st.ptr)) };
    }
    let ns = t0.elapsed().as_secs_f64() * 1e9 / (iters * GROUP) as f64;
    eprintln!("pmp: {GROUP} instances agree with the model ({allowed} allowed); {ns:.2} ns an instance");
}

/// common_cells' lzc at several widths, both modes: recognised as a count
/// of zeros (`netlist::recognise`), it must give what its own logic gives
/// for every input tried, zero and single bits among them.
#[test]
fn lzc_recognised_matches_its_logic() {
    let Some(cva6) = std::env::var_os("CVA6_DIR").map(PathBuf::from) else {
        eprintln!("set CVA6_DIR to a CVA6 checkout to run this; skipping");
        return;
    };
    let Some((t, triple, mattr)) = host() else {
        eprintln!("needs cc on an AArch64 Mac or x86-64 Linux; skipping");
        return;
    };
    let tmp = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("netlist");
    std::fs::create_dir_all(&tmp).unwrap();
    for (w, mode) in [(56u32, 0u32), (56, 1), (13, 0), (13, 1), (37, 0), (64, 1), (32, 0)] {
        let cw = 32 - (w - 1).leading_zeros();
        let name = format!("lzc_{w}_{mode}");
        let src = tmp.join(format!("{name}.sv"));
        std::fs::write(
            &src,
            format!(
                "module {name} (input logic [{}:0] in_i, output logic [{}:0] cnt_o, output logic empty_o);\n  \
                 lzc #(.WIDTH({w}), .MODE({mode})) i_lzc (.*);\nendmodule\n",
                w - 1,
                cw - 1
            ),
        )
        .unwrap();
        let files =
            vec![cva6.join("vendor/pulp-platform/common_cells/src/cf_math_pkg.sv"), cva6.join("vendor/pulp-platform/common_cells/src/lzc.sv"), src];
        let mut built = Vec::new();
        for recognise in [true, false] {
            let opts = netlist::Options { recognise, ..netlist_options() };
            let net = netlist_with(files.clone(), Vec::new(), &name, opts);
            let tag = if recognise { "known" } else { "logic" };
            if recognise {
                assert!(net.ir.matches("@llvm.ctlz.v16i32(").count() > 1, "{name} was not recognised");
            }
            built.push(build(&format!("{name}_{tag}"), net, &t, triple, &mattr));
        }
        let mask = if w == 64 { !0 } else { (1u64 << w) - 1 };
        let mut rng = Rng(0x1234_5678_9ABC_DEF1 ^ w as u64);
        let mut ins = vec![0u64, mask];
        ins.extend((0..w).map(|i| 1u64 << i));
        ins.extend((0..w).map(|i| mask << i & mask));
        while ins.len() < GROUP {
            let r = rng.next();
            ins.push((r >> (rng.next() % 64)) & mask);
        }
        let mut outs = Vec::new();
        for b in &built {
            let mut st = State::new(b.net.size);
            for (n, &v) in ins.iter().enumerate() {
                st.put(&b.net, "in_i", n, v);
            }
            unsafe { (b.eval)(st.ptr) };
            outs.push((0..GROUP).map(|n| (st.get(&b.net, "cnt_o", n), st.get(&b.net, "empty_o", n))).collect::<Vec<_>>());
        }
        for (n, &v) in ins.iter().enumerate() {
            assert_eq!(outs[0][n], outs[1][n], "{name}: in {v:x}: recognised gives {:?}, its logic {:?}", outs[0][n], outs[1][n]);
            if v != 0 {
                let want = if mode == 0 { v.trailing_zeros() } else { (v << (64 - w)).leading_zeros() } as u64;
                assert_eq!(outs[0][n].0, want, "{name}: in {v:x}");
            }
        }
        eprintln!("{name}: recognised and its logic agree on {GROUP} inputs; code {} bytes against {}", built[0].sizes.iter().map(|s| s.1).sum::<u64>(), built[1].sizes.iter().map(|s| s.1).sum::<u64>());
    }
}
