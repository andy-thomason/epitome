//! The compiled netlist (`src/netlist.rs`), built by Epitome, loaded and
//! run, and checked instance by instance: a module of operators against
//! values worked out here, and CVA6's `pmp_entry` (when `CVA6_DIR` points
//! at a CVA6 checkout) against a scalar model of it.
//!
//! On an AArch64 Mac or x86-64 Linux; elsewhere the tests skip.
//! x86-64 uses AVX2 unless `NETLIST_MATTR` names other features.

mod common;
#[path = "cva6/units.rs"]
mod units;

use std::path::PathBuf;

use common::*;
use epitome_sv::netlist::{self, GROUP};

/// Runs `n` instances a state's lanes at a time: `put` sets lane `k`'s
/// inputs for instance `i` (and says what it is), the state is evaluated,
/// and `check` looks at each lane's outputs.
fn by_states<T>(
    b: &Model,
    n: usize,
    mut put: impl FnMut(&mut State, usize, usize) -> T,
    mut check: impl FnMut(&State, usize, &T),
) -> State {
    let mut st = State::new(b.net.size);
    for g in 0..n.div_ceil(b.net.lanes) {
        st = State::new(b.net.size);
        let lanes = b.net.lanes.min(n - g * b.net.lanes);
        let ins: Vec<T> = (0..lanes).map(|k| put(&mut st, k, g * b.net.lanes + k)).collect();
        b.eval_state(&mut st);
        for (k, x) in ins.iter().enumerate() {
            check(&st, k, x);
        }
    }
    st
}

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
    let Some(host) = host() else {
        eprintln!("needs an AArch64 Mac or x86-64 Linux; skipping");
        return;
    };
    let b = build("ops", netlist_of(&text("ops.sv", OPS), "ops"), &host);
    report("ops", &b);
    let net = &b.net;
    let m56 = (1u64 << 56) - 1;
    let mut rng = Rng(0x2545_F491_4F6C_DD1D);
    let mut st = State::new(net.size);
    // GROUP instances a round, a state's lanes at a time.
    for round in 0..4 * GROUP.div_ceil(net.lanes) {
        let mut ins = Vec::with_capacity(net.lanes);
        for n in 0..net.lanes {
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
        b.eval_state(&mut st);
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

/// Builds CVA6 unit `name` (`cva6/units.rs`), runs it on its stimulus for
/// a group of instances, and gives each instance's inputs and its one
/// output to `check`; with the time an instance, the stimulus run `reps`
/// times, and the lanes of a state.
fn run_unit(name: &str, reps: usize, mut check: impl FnMut(&Io, &[u32], u64)) -> Option<(f64, usize)> {
    let (cva6, host) = setup()?;
    let u = units::unit(name);
    let b = build(name, netlist_of(&u.sources(&cva6), u.top), &host);
    report(name, &b);
    let io = Io::new(&b.net, None).unwrap();
    let stim = u.stimulus(units::Mix::Random, &io, 1, GROUP, &mut Rng(u.seed));
    let out = b.run(&io, 1, GROUP, &stim);
    assert_eq!(io.out_words(), 1);
    for (i, w) in stim.chunks(io.in_words()).enumerate() {
        check(&io, w, out[i] as u64);
    }
    Some((b.time(&io, 1, GROUP, &stim, reps).ns, b.net.lanes))
}

#[test]
fn cva6_pmp_entry() {
    let mut matches = 0;
    let Some((ns, lanes)) = run_unit("pmp_entry", 200, |io, w, got| {
        let v = |p| low64(input(io, w, p));
        let (addr, conf, prev, mode) = (v("addr_i"), v("conf_addr_i"), v("conf_addr_prev_i"), v("conf_addr_mode_i"));
        let want = pmp_reference(addr, conf, prev, mode);
        assert_eq!(got == 1, want, "addr {addr:x} conf {conf:x} prev {prev:x} mode {mode}");
        matches += want as usize;
    }) else {
        return;
    };
    eprintln!("pmp_entry: {GROUP} instances agree with the model ({matches} match); {ns:.2} ns an instance ({lanes} lanes)");
}

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

/// Bits `lo..lo + w` (`w` at most 64) of `words`.
fn bits(words: &[u32], lo: usize, w: usize) -> u64 {
    (0..w).map(|i| ((words[(lo + i) / 32] >> ((lo + i) % 32) & 1) as u64) << i).sum()
}

#[test]
fn cva6_pmp() {
    let mut allowed = 0;
    let Some((ns, lanes)) = run_unit("pmp", 20, |io, w, got| {
        let v = |p| low64(input(io, w, p));
        let (addr, access, priv_lvl) = (v("addr_i"), v("access_type_i"), v("priv_lvl_i"));
        let conf_addr: [u64; 8] = std::array::from_fn(|i| bits(input(io, w, "conf_addr_i"), 54 * i, 54));
        let conf: [u64; 8] = std::array::from_fn(|i| bits(input(io, w, "conf_i"), 8 * i, 8));
        let want = pmp_unit_reference(addr, access, priv_lvl, &conf_addr, &conf);
        assert_eq!(got == 1, want, "addr {addr:x} access {access} priv {priv_lvl} conf_addr {conf_addr:x?} conf {conf:x?}");
        allowed += want as usize;
    }) else {
        return;
    };
    eprintln!("pmp: {GROUP} instances agree with the model ({allowed} allowed); {ns:.2} ns an instance ({lanes} lanes)");
}

/// common_cells' lzc at several widths, both modes: recognised as a count
/// of zeros (`netlist::recognise`), it must give what its own logic gives
/// for every input tried, zero and single bits among them.
#[test]
fn lzc_recognised_matches_its_logic() {
    let Some((cva6, host)) = setup() else { return };
    for (w, mode) in [(56u32, 0u32), (56, 1), (13, 0), (13, 1), (37, 0), (64, 1), (32, 0)] {
        let cw = 32 - (w - 1).leading_zeros();
        let name = format!("lzc_{w}_{mode}");
        let mut src = Sources::default();
        src.file(cva6.join("vendor/pulp-platform/common_cells/src/cf_math_pkg.sv"));
        src.file(cva6.join("vendor/pulp-platform/common_cells/src/lzc.sv"));
        src.text(
            format!("{name}.sv"),
            format!(
                "module {name} (input logic [{}:0] in_i, output logic [{}:0] cnt_o, output logic empty_o);\n  \
                 lzc #(.WIDTH({w}), .MODE({mode})) i_lzc (.*);\nendmodule\n",
                w - 1,
                cw - 1
            ),
        );
        let mut built = Vec::new();
        for recognise in [true, false] {
            let opts = netlist::Options { recognise, ..netlist_options() };
            let net = netlist_with(&src, &name, opts);
            let tag = if recognise { "known" } else { "logic" };
            if recognise {
                assert!(net.ir().matches("@llvm.ctlz.").count() > 1, "{name} was not recognised");
            }
            built.push(build(&format!("{name}_{tag}"), net, &host));
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
            let mut got = vec![(0, 0); GROUP];
            by_states(b, GROUP, |st, n, i| {
                st.put(&b.net, "in_i", n, ins[i]);
                i
            }, |st, n, &i| got[i] = (st.get(&b.net, "cnt_o", n), st.get(&b.net, "empty_o", n)));
            outs.push(got);
        }
        for (n, &v) in ins.iter().enumerate() {
            assert_eq!(outs[0][n], outs[1][n], "{name}: in {v:x}: recognised gives {:?}, its logic {:?}", outs[0][n], outs[1][n]);
            if v != 0 {
                let want = if mode == 0 { v.trailing_zeros() } else { (v << (64 - w)).leading_zeros() } as u64;
                assert_eq!(outs[0][n].0, want, "{name}: in {v:x}");
            }
        }
        eprintln!("{name}: recognised and its logic agree on {GROUP} inputs; code {} bytes against {}", built[0].code_bytes(), built[1].code_bytes());
    }
}
