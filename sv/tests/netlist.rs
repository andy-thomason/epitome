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
    output logic [7:0]  pri,
    output logic [16:0] decl
);
  function automatic logic odd(input logic [55:0] x);
    return x[0];
  endfunction
  // Net declaration assignments, each one continuous assignment.
  wire p = a[0] ^ b[0];
  wire [7:0] lo = a[7:0], hi = ~b[7:0];
  assign decl = {p, lo, hi};
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
                ("decl", ((a ^ b2) & 1) << 16 | (a & 0xFF) << 8 | (!b2 & 0xFF)),
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

const UNPACKED: &str = r#"
module recs (
    input  logic [7:0]  a,
    input  logic [12:0] b,
    input  logic [1:0]  k,
    output logic [7:0]  first,
    output logic [12:0] second,
    output logic [20:0] whole,
    output logic [7:0]  picked,
    output logic        flag
);
  typedef struct { logic [2:0] tag; logic flag; } inner_t;
  typedef struct { logic [7:0] x; inner_t in; logic [12:0] y; } rec_t;
  rec_t r;
  rec_t rs [4];
  always_comb begin
    r = '{x: a, in: '{tag: a[2:0], flag: b[0]}, y: b};
    r.x = r.x ^ 8'h0f;
    for (int i = 0; i < 4; i++) begin
      rs[i] = r;
      rs[i].x = r.x + 8'(i);
    end
  end
  assign first = r.x;
  assign second = r.y;
  assign whole = {r.x, r.y};
  assign picked = rs[k].x;
  assign flag = r.in.flag;
endmodule
"#;

#[test]
fn unpacked_structs_lane_by_lane() {
    // Unpacked structs, nested, in an array, built by a pattern and written
    // member by member, are kept as their members concatenated.
    let Some(host) = host() else {
        eprintln!("needs an AArch64 Mac or x86-64 Linux; skipping");
        return;
    };
    let b = build("recs", netlist_of(&text("recs.sv", UNPACKED), "recs"), &host);
    report("recs", &b);
    let mut rng = Rng(0x9E37_79B9_7F4A_7C15);
    by_states(
        &b,
        4 * b.net.lanes,
        |st, k, _| {
            let (a, bb, kk) = (rng.next() & 0xFF, rng.next() & 0x1FFF, rng.next() & 3);
            for (p, v) in [("a", a), ("b", bb), ("k", kk)] {
                st.put(&b.net, p, k, v);
            }
            (a, bb, kk)
        },
        |st, k, &(a, bb, kk)| {
            let x = a ^ 0x0F;
            for (port, w) in [("first", x), ("second", bb), ("whole", x << 13 | bb), ("picked", (x + kk) & 0xFF), ("flag", bb & 1)] {
                assert_eq!(st.get(&b.net, port, k), w, "{port}: a {a:x} b {bb:x} k {kk}");
            }
        },
    );
}

const DIVMOD: &str = r#"
module divmod (
    input  logic [15:0] a,
    input  logic [15:0] b,
    input  logic [39:0] c,
    input  logic [39:0] d,
    output logic [15:0] q,
    output logic [15:0] r,
    output logic signed [15:0] sq,
    output logic signed [15:0] sr,
    output logic [39:0] wq,
    output logic [39:0] wr
);
  assign q = a / b;
  assign r = a % b;
  assign sq = $signed(a) / $signed(b);
  assign sr = $signed(a) % $signed(b);
  assign wq = c / d;
  assign wr = c % d;
endmodule
"#;

#[test]
fn division_lane_by_lane() {
    // Division and modulo of values not known at compile time, unsigned and
    // signed, in one piece and across pieces; by zero, 0.
    let Some(host) = host() else {
        eprintln!("needs an AArch64 Mac or x86-64 Linux; skipping");
        return;
    };
    let b = build("divmod", netlist_of(&text("divmod.sv", DIVMOD), "divmod"), &host);
    report("divmod", &b);
    let mut rng = Rng(0x6A09_E667_F3BC_C908);
    let m40 = (1u64 << 40) - 1;
    by_states(
        &b,
        16 * b.net.lanes,
        |st, k, i| {
            let (a, mut bb) = (rng.next() & 0xFFFF, rng.next() & 0xFFFF);
            let (c, mut d) = (rng.next() & m40, rng.next() & m40);
            match i % 4 {
                0 => bb = 0,
                1 => bb &= 0xF,
                2 => d &= 0xFFFFF,
                _ => {}
            }
            if i % 7 == 0 {
                d = 0;
            }
            for (p, v) in [("a", a), ("b", bb), ("c", c), ("d", d)] {
                st.put(&b.net, p, k, v);
            }
            (a, bb, c, d)
        },
        |st, k, &(a, bb, c, d)| {
            let (sa, sb) = (a as u16 as i16, bb as u16 as i16);
            let sdiv = |x: i16, y: i16| if y == 0 { 0 } else { x.wrapping_div(y) as u16 as u64 };
            let srem = |x: i16, y: i16| if y == 0 { 0 } else { x.wrapping_rem(y) as u16 as u64 };
            let want = [
                ("q", if bb == 0 { 0 } else { a / bb }),
                ("r", if bb == 0 { 0 } else { a % bb }),
                ("sq", sdiv(sa, sb)),
                ("sr", srem(sa, sb)),
                ("wq", if d == 0 { 0 } else { c / d }),
                ("wr", if d == 0 { 0 } else { c % d }),
            ];
            for (port, w) in want {
                assert_eq!(st.get(&b.net, port, k), w, "{port}: a {a} b {bb} c {c} d {d}");
            }
        },
    );
}

const COUNTS: &str = r#"
module counts (
    input  logic [6:0]  n,
    input  logic [44:0] v,
    output logic [5:0]  ones,
    output logic [3:0]  few,
    output logic        hot,
    output logic        hot0,
    output logic        unk
);
  assign ones = $countones(v);
  assign few = $countones(n);
  assign hot = $onehot(v);
  assign hot0 = $onehot0(n);
  assign unk = $isunknown(v);
endmodule
"#;

#[test]
fn bit_counts_lane_by_lane() {
    // $countones, $onehot and $onehot0 on values in one piece and across
    // pieces; $isunknown is 0, as the netlist holds no x.
    let Some(host) = host() else {
        eprintln!("needs an AArch64 Mac or x86-64 Linux; skipping");
        return;
    };
    let b = build("counts", netlist_of(&text("counts.sv", COUNTS), "counts"), &host);
    report("counts", &b);
    let mut rng = Rng(0xBB67_AE85_84CA_A73B);
    let m45 = (1u64 << 45) - 1;
    by_states(
        &b,
        16 * b.net.lanes,
        |st, k, i| {
            let mut v = rng.next() & m45;
            let mut n = rng.next() & 0x7F;
            match i % 4 {
                0 => v = 1 << (rng.next() % 45),
                1 => n = 0,
                2 => n = 1 << (rng.next() % 7),
                _ => {}
            }
            st.put(&b.net, "v", k, v);
            st.put(&b.net, "n", k, n);
            (v, n)
        },
        |st, k, &(v, n)| {
            let want = [
                ("ones", v.count_ones() as u64),
                ("few", n.count_ones() as u64),
                ("hot", (v.count_ones() == 1) as u64),
                ("hot0", (n.count_ones() <= 1) as u64),
                ("unk", 0),
            ];
            for (port, w) in want {
                assert_eq!(st.get(&b.net, port, k), w, "{port}: v {v:x} n {n:x}");
            }
        },
    );
}

const LATCH: &str = r#"
module lat (
    input  logic       en,
    input  logic [7:0] d,
    output logic [7:0] q,
    output logic [7:0] q2
);
  always_latch begin
    if (en) q = d;
  end
  always_latch begin
    if (!en) q2 = d + 8'd1;
  end
endmodule
"#;

#[test]
fn latches_hold() {
    // An always_latch writes its values while enabled and holds them when
    // not.
    let Some(host) = host() else {
        eprintln!("needs an AArch64 Mac or x86-64 Linux; skipping");
        return;
    };
    let b = build("lat", netlist_of(&text("lat.sv", LATCH), "lat"), &host);
    report("lat", &b);
    let net = &b.net;
    let mut st = b.state();
    let mut rng = Rng(0x3C6E_F372_FE94_F82B);
    let (mut q, mut q2) = (vec![0u64; net.lanes], vec![0u64; net.lanes]);
    for _ in 0..12 {
        let ins: Vec<(u64, u64)> = (0..net.lanes).map(|_| (rng.next() & 1, rng.next() & 0xFF)).collect();
        for (k, &(en, d)) in ins.iter().enumerate() {
            st.put(net, "en", k, en);
            st.put(net, "d", k, d);
        }
        b.eval_state(&mut st);
        for (k, &(en, d)) in ins.iter().enumerate() {
            if en == 1 {
                q[k] = d;
            } else {
                q2[k] = (d + 1) & 0xFF;
            }
            assert_eq!(st.get(net, "q", k), q[k], "q, lane {k}: en {en} d {d}");
            assert_eq!(st.get(net, "q2", k), q2[k], "q2, lane {k}: en {en} d {d}");
        }
    }
}

const FALSE_LOOP: &str = r#"
module inc (input logic [7:0] a, input logic [7:0] b, output logic [7:0] ya, output logic [7:0] yb);
  assign ya = a + 8'd1;
  assign yb = b ^ 8'h5a;
endmodule
module floop (
    input  logic [7:0] x,
    output logic [7:0] w,
    output logic [7:0] z
);
  logic [7:0] u, v, cb, ca, cya, cyb;
  // Each process waits on the other, though no bit does: u -> v -> w.
  always_comb begin
    u = x + 8'd3;
    w = v ^ 8'hff;
  end
  always_comb begin
    v = u + 8'd7;
  end
  // Through a child: its b input is made from its ya output.
  inc c (.a(ca), .b(cb), .ya(cya), .yb(cyb));
  always_comb begin
    ca = x;
    cb = cya + w;
    z = cyb;
  end
endmodule
"#;

#[test]
fn false_loops_settle() {
    // Code that waits on itself as whole processes or calls, though not
    // through any bit, is run over until it settles.
    let Some(host) = host() else {
        eprintln!("needs an AArch64 Mac or x86-64 Linux; skipping");
        return;
    };
    let b = build("floop", netlist_of(&text("floop.sv", FALSE_LOOP), "floop"), &host);
    report("floop", &b);
    let mut rng = Rng(0xA54F_F53A_5F1D_36F1);
    by_states(
        &b,
        4 * b.net.lanes,
        |st, k, _| {
            let x = rng.next() & 0xFF;
            st.put(&b.net, "x", k, x);
            x
        },
        |st, k, &x| {
            let u = (x + 3) & 0xFF;
            let v = (u + 7) & 0xFF;
            let w = v ^ 0xFF;
            let cya = (x + 1) & 0xFF;
            let cb = (cya + w) & 0xFF;
            let z = cb ^ 0x5A;
            assert_eq!(st.get(&b.net, "w", k), w, "w: x {x}");
            assert_eq!(st.get(&b.net, "z", k), z, "z: x {x}");
        },
    );
}

const REPORTS: &str = r#"
module reps #(parameter int N = 4) (
    input  logic       clk,
    input  logic [7:0] a,
    output logic [7:0] y
);
  // Checks that only report change nothing, and are left out.
  initial begin
    if (N * 2 != 8) $error("N = %0d", N);
  end
  always_comb begin
    y = a + 8'(N);
    if (a == 8'hff) $warning("a is %0d", a);
  end
  always_ff @(posedge clk) if (a == 8'h00) $display("zero at %0d", a);
endmodule
"#;

#[test]
fn reports_are_left_out() {
    let Some(host) = host() else {
        eprintln!("needs an AArch64 Mac or x86-64 Linux; skipping");
        return;
    };
    let b = build("reps", netlist_of(&text("reps.sv", REPORTS), "reps"), &host);
    report("reps", &b);
    let mut rng = Rng(0x510E_527F_ADE6_82D1);
    by_states(
        &b,
        2 * b.net.lanes,
        |st, k, _| {
            let a = rng.next() & 0xFF;
            st.put(&b.net, "a", k, a);
            a
        },
        |st, k, &a| assert_eq!(st.get(&b.net, "y", k), (a + 4) & 0xFF, "a {a}"),
    );
}

const WIDE_MEM: &str = r#"
module wmem (
    input  logic         clk,
    input  logic         we,
    input  logic         wb,
    input  logic [3:0]   wa,
    input  logic [3:0]   ra,
    input  logic [63:0]  d,
    output logic [199:0] q,
    output logic [7:0]   qb
);
  logic [199:0] mem [16];
  always_ff @(posedge clk) begin
    if (we) mem[wa] <= {d[7:0], d, d, d};
    if (wb) mem[wa][15:8] <= d[15:8];
  end
  assign q = mem[ra];
  assign qb = mem[ra][15:8];
endmodule
"#;

#[test]
fn wide_memories_by_element() {
    // A memory of words wider than a machine word is kept a word to a slot
    // in scalar code: written whole and in part, read, by addresses known
    // only at run time.
    let Some(host) = host() else {
        eprintln!("needs an AArch64 Mac or x86-64 Linux; skipping");
        return;
    };
    let net = netlist_with(&text("wmem.sv", WIDE_MEM), "wmem", host.options(1));
    let b = build("wmem", net, &host);
    report("wmem", &b);
    let io = Io::new(&b.net, Some("clk")).unwrap();
    let mut rng = Rng(0x1F83_D9AB_FB41_BD6B);
    let cycles = 400;
    let mut stim = Vec::new();
    let mut ins = Vec::new();
    for _ in 0..cycles {
        let (we, wb, wa, ra, d) = (rng.next() & 1, rng.next() % 3 == 0, rng.next() & 15, rng.next() & 15, rng.next());
        for p in &io.ins {
            match p.name.as_str() {
                "we" => stim.push(we as u32),
                "wb" => stim.push(wb as u32),
                "wa" => stim.push(wa as u32),
                "ra" => stim.push(ra as u32),
                "d" => stim.extend([d as u32, (d >> 32) as u32]),
                n => panic!("{n}"),
            }
        }
        ins.push((we == 1, wb, wa as usize, ra as usize, d));
    }
    let out = b.run(&io, cycles, 1, &stim);
    // The model: 200-bit words as four u64s, the least significant first.
    let mut mem = [[0u64; 4]; 16];
    let ow = io.out_words();
    for (c, &(we, wb, wa, ra, d)) in ins.iter().enumerate() {
        if we {
            mem[wa] = [d, d, d, d & 0xFF];
        }
        if wb {
            mem[wa][0] = mem[wa][0] & !0xFF00 | d & 0xFF00;
        }
        let words = &out[c * ow..(c + 1) * ow];
        let want = mem[ra];
        let q: Vec<u32> = (0..7).map(|k| (want[k / 2] >> (32 * (k % 2))) as u32 & if k == 6 { 0xFF } else { !0 }).collect();
        assert_eq!(&words[..7], &q[..], "cycle {c}: q of {ra}");
        assert_eq!(words[7], ((want[0] >> 8) & 0xFF) as u32, "cycle {c}: qb of {ra}");
    }
}

const ADDR_MEM: &str = r#"
module amem (
    input  logic         clk,
    input  logic         we,
    input  logic         wb,
    input  logic [5:0]   wa,
    input  logic [5:0]   ra,
    input  logic [63:0]  d,
    output logic [127:0] q,
    output logic [15:0]  qh
);
  logic [127:0] mem [40];
  always_ff @(posedge clk) begin
    if (we) mem[wa] <= {d ^ 64'h5555, d};
    if (wb) mem[wa][79:64] <= d[15:0];
  end
  assign q = mem[ra];
  assign qh = mem[ra][79:64];
endmodule
"#;

#[test]
fn memories_by_address() {
    // A memory kept in the state, of words a whole number of machine words
    // wide, read and written (whole and in part) by its address in scalar
    // code; out of range, a read gives 0 and a write changes nothing.
    let Some(host) = host() else {
        eprintln!("needs an AArch64 Mac or x86-64 Linux; skipping");
        return;
    };
    let net = netlist_with(&text("amem.sv", ADDR_MEM), "amem", host.options(1));
    let b = build("amem", net, &host);
    report("amem", &b);
    let io = Io::new(&b.net, Some("clk")).unwrap();
    let mut rng = Rng(0x5BE0_CD19_137E_2179);
    let cycles = 600;
    let (mut stim, mut ins) = (Vec::new(), Vec::new());
    for _ in 0..cycles {
        let (we, wb, wa, ra, d) = (rng.next() & 1 == 1, rng.next() % 3 == 0, rng.next() % 48, rng.next() % 48, rng.next());
        for p in &io.ins {
            match p.name.as_str() {
                "we" => stim.push(we as u32),
                "wb" => stim.push(wb as u32),
                "wa" => stim.push(wa as u32),
                "ra" => stim.push(ra as u32),
                "d" => stim.extend([d as u32, (d >> 32) as u32]),
                n => panic!("{n}"),
            }
        }
        ins.push((we, wb, wa as usize, ra as usize, d));
    }
    let out = b.run(&io, cycles, 1, &stim);
    let mut mem = [[0u64; 2]; 40];
    let ow = io.out_words();
    for (c, &(we, wb, wa, ra, d)) in ins.iter().enumerate() {
        if wa < 40 {
            if we {
                mem[wa] = [d, d ^ 0x5555];
            }
            if wb {
                mem[wa][1] = mem[wa][1] & !0xFFFF | d & 0xFFFF;
            }
        }
        let want = if ra < 40 { mem[ra] } else { [0, 0] };
        let words = &out[c * ow..(c + 1) * ow];
        let q: Vec<u32> = (0..4).map(|k| (want[k / 2] >> (32 * (k % 2))) as u32).collect();
        assert_eq!(&words[..4], &q[..], "cycle {c}: q of {ra}");
        assert_eq!(words[4], (want[1] & 0xFFFF) as u32, "cycle {c}: qh of {ra}");
    }
}

const ODD_MEM: &str = r#"
module omem (
    input  logic         clk,
    input  logic         we,
    input  logic         wb,
    input  logic [5:0]   wa,
    input  logic [5:0]   ra,
    input  logic [63:0]  d,
    output logic [72:0]  q,
    output logic [7:0]   qb
);
  // Words 73 bits wide: not a whole number of machine words apart.
  logic [72:0] mem [37];
  always_ff @(posedge clk) begin
    if (we) mem[wa] <= {d[8:0], d};
    if (wb) mem[wa][67:60] <= d[7:0];
  end
  assign q = mem[ra];
  assign qb = mem[ra][67:60];
endmodule
"#;

#[test]
fn memories_of_odd_widths_by_address() {
    let Some(host) = host() else {
        eprintln!("needs an AArch64 Mac or x86-64 Linux; skipping");
        return;
    };
    let net = netlist_with(&text("omem.sv", ODD_MEM), "omem", host.options(1));
    let b = build("omem", net, &host);
    report("omem", &b);
    let io = Io::new(&b.net, Some("clk")).unwrap();
    let mut rng = Rng(0xCBBB_9D5D_C105_9ED8);
    let cycles = 600;
    let (mut stim, mut ins) = (Vec::new(), Vec::new());
    for _ in 0..cycles {
        let (we, wb, wa, ra, d) = (rng.next() & 1 == 1, rng.next() % 3 == 0, rng.next() % 44, rng.next() % 44, rng.next());
        for p in &io.ins {
            match p.name.as_str() {
                "we" => stim.push(we as u32),
                "wb" => stim.push(wb as u32),
                "wa" => stim.push(wa as u32),
                "ra" => stim.push(ra as u32),
                "d" => stim.extend([d as u32, (d >> 32) as u32]),
                n => panic!("{n}"),
            }
        }
        ins.push((we, wb, wa as usize, ra as usize, d));
    }
    let out = b.run(&io, cycles, 1, &stim);
    // 73-bit words as u128s.
    let mut mem = [0u128; 37];
    let m73 = (1u128 << 73) - 1;
    let ow = io.out_words();
    for (c, &(we, wb, wa, ra, d)) in ins.iter().enumerate() {
        if wa < 37 {
            if we {
                mem[wa] = ((d as u128 & 0x1FF) << 64 | d as u128) & m73;
            }
            if wb {
                mem[wa] = mem[wa] & !(0xFFu128 << 60) | ((d as u128 & 0xFF) << 60);
            }
        }
        let want = if ra < 37 { mem[ra] } else { 0 };
        let words = &out[c * ow..(c + 1) * ow];
        let q: Vec<u32> = (0..3).map(|k| (want >> (32 * k)) as u32 & if k == 2 { 0x1FF } else { !0 }).collect();
        assert_eq!(&words[..3], &q[..], "cycle {c}: q of {ra}");
        assert_eq!(words[3], ((want >> 60) & 0xFF) as u32, "cycle {c}: qb of {ra}");
    }
}

const QUEUED_MEM: &str = r#"
module qmem (
    input  logic         clk,
    input  logic         we,
    input  logic         wb,
    input  logic         clr,
    input  logic [5:0]   wa,
    input  logic [5:0]   ra,
    input  logic [63:0]  d,
    output logic [127:0] q,
    output logic [72:0]  r
);
  // Memories the process that writes them also reads: every read of the
  // cycle sees them as they were before it.
  logic [127:0] m [40];
  logic [72:0]  n [37];
  always_ff @(posedge clk) begin
    q <= m[ra];
    r <= n[ra];
    if (we) begin
      m[wa] <= {m[ra][63:0], d};
      n[wa] <= {d[8:0], d} ^ n[ra];
    end
    if (wb) begin
      m[wa][71:64] <= d[7:0];
      n[wa][67:60] <= d[7:0];
    end
    if (clr)
      for (int i = 0; i < 3; i++) begin
        m[i] <= '0;
        n[i] <= '0;
      end
  end
endmodule
"#;

#[test]
fn memories_read_where_written() {
    // Nonblocking writes to memories their own process reads take effect
    // after it, in the order they were made; out of range, a read gives 0
    // and a write changes nothing.
    let Some(host) = host() else {
        eprintln!("needs an AArch64 Mac or x86-64 Linux; skipping");
        return;
    };
    let net = netlist_with(&text("qmem.sv", QUEUED_MEM), "qmem", host.options(1));
    let b = build("qmem", net, &host);
    report("qmem", &b);
    let io = Io::new(&b.net, Some("clk")).unwrap();
    let mut rng = Rng(0x1F83_D9AB_FB41_BD6B);
    let cycles = 800;
    let (mut stim, mut ins) = (Vec::new(), Vec::new());
    for _ in 0..cycles {
        let (we, wb, clr) = (rng.next() & 1 == 1, rng.next() % 3 == 0, rng.next() % 17 == 0);
        let (wa, ra, d) = (rng.next() % 44, rng.next() % 44, rng.next());
        for p in &io.ins {
            match p.name.as_str() {
                "we" => stim.push(we as u32),
                "wb" => stim.push(wb as u32),
                "clr" => stim.push(clr as u32),
                "wa" => stim.push(wa as u32),
                "ra" => stim.push(ra as u32),
                "d" => stim.extend([d as u32, (d >> 32) as u32]),
                n => panic!("{n}"),
            }
        }
        ins.push((we, wb, clr, wa as usize, ra as usize, d));
    }
    let out = b.run(&io, cycles, 1, &stim);
    let (mut m, mut n) = ([0u128; 40], [0u128; 37]);
    let m73 = (1u128 << 73) - 1;
    let ow = io.out_words();
    for (c, &(we, wb, clr, wa, ra, d)) in ins.iter().enumerate() {
        let (mr, nr) = (m.get(ra).copied().unwrap_or(0), n.get(ra).copied().unwrap_or(0));
        let d = d as u128;
        if we {
            if wa < 40 {
                m[wa] = mr << 64 | d;
            }
            if wa < 37 {
                n[wa] = ((d & 0x1FF) << 64 | d) ^ nr;
            }
        }
        if wb {
            if wa < 40 {
                m[wa] = m[wa] & !(0xFFu128 << 64) | (d & 0xFF) << 64;
            }
            if wa < 37 {
                n[wa] = n[wa] & !(0xFFu128 << 60) | (d & 0xFF) << 60;
            }
        }
        if clr {
            m[..3].fill(0);
            n[..3].fill(0);
        }
        let words = &out[c * ow..(c + 1) * ow];
        let q: Vec<u32> = (0..4).map(|k| (mr >> (32 * k)) as u32).collect();
        let r: Vec<u32> = (0..3).map(|k| ((nr & m73) >> (32 * k)) as u32).collect();
        assert_eq!(&words[..4], &q[..], "cycle {c}: q of {ra}");
        assert_eq!(&words[4..7], &r[..], "cycle {c}: r of {ra}");
    }
}

const MEM_ARRAY: &str = r#"
module bank #(parameter int W = 72) (
    input  logic         clk,
    input  logic         we,
    input  logic [7:0]   a,
    input  logic [W-1:0] d,
    output logic [W-1:0] q
);
  logic [W-1:0] ram [200];
  always_ff @(posedge clk) begin
    if (we) ram[a] <= d;
    else q <= ram[a];
  end
endmodule

module banks (
    input  logic         clk,
    input  logic [3:0]   we,
    input  logic [7:0]   a,
    input  logic [63:0]  d,
    output logic [287:0] q
);
  // An array of memories: each called on its own, by its address.
  for (genvar k = 0; k < 4; k++) begin : g
    bank u (.clk, .we(we[k]), .a, .d({d[7:0] ^ 8'(k), d}), .q(q[k*72 +: 72]));
  end
endmodule
"#;

#[test]
fn arrays_of_memories() {
    let Some(host) = host() else {
        eprintln!("needs an AArch64 Mac or x86-64 Linux; skipping");
        return;
    };
    let net = netlist_with(&text("banks.sv", MEM_ARRAY), "banks", host.options(1));
    let b = build("banks", net, &host);
    report("banks", &b);
    let io = Io::new(&b.net, Some("clk")).unwrap();
    let mut rng = Rng(0x6A09_E667_F3BC_C908);
    let cycles = 800;
    let (mut stim, mut ins) = (Vec::new(), Vec::new());
    for _ in 0..cycles {
        let (we, a, d) = (rng.next() % 16, rng.next() % 220, rng.next());
        for p in &io.ins {
            match p.name.as_str() {
                "we" => stim.push(we as u32),
                "a" => stim.push(a as u32),
                "d" => stim.extend([d as u32, (d >> 32) as u32]),
                n => panic!("{n}"),
            }
        }
        ins.push((we, a as usize, d));
    }
    let out = b.run(&io, cycles, 1, &stim);
    let mut ram = vec![[0u128; 200]; 4];
    let mut q = [0u128; 4];
    let ow = io.out_words();
    for (c, &(we, a, d)) in ins.iter().enumerate() {
        for k in 0..4 {
            if we >> k & 1 == 1 {
                if a < 200 {
                    ram[k][a] = ((d as u128 & 0xFF) ^ k as u128) << 64 | d as u128;
                }
            } else {
                q[k] = ram[k].get(a).copied().unwrap_or(0);
            }
        }
        let words = &out[c * ow..(c + 1) * ow];
        let mut all = vec![0u32; 9];
        for k in 0..4 {
            for bit in 0..72 {
                if q[k] >> bit & 1 == 1 {
                    let at = k * 72 + bit;
                    all[at / 32] |= 1 << (at % 32);
                }
            }
        }
        assert_eq!(&words[..9], &all[..], "cycle {c}");
    }
}

const PROFILED: &str = r#"
module leaf (input logic clk, input logic [31:0] a, output logic [31:0] q);
  always_ff @(posedge clk) q <= a * 3 + 1;
endmodule

module mid (input logic clk, input logic [31:0] a, output logic [31:0] q);
  logic [31:0] t;
  leaf l0 (.clk, .a, .q(t));
  leaf l1 (.clk, .a(t ^ 32'h55), .q);
endmodule

module ptop (input logic clk, input logic [31:0] a, output logic [127:0] q);
  for (genvar k = 0; k < 4; k++) begin : g
    mid m (.clk, .a(a + 32'(k)), .q(q[k*32 +: 32]));
  end
endmodule
"#;

#[test]
fn profiled_by_instance() {
    // Each instance's calls counted in its own state: a counter for each
    // instance and function, within its caller's, and the same outputs as
    // without a profile.
    let Some(host) = host() else {
        eprintln!("needs an AArch64 Mac or x86-64 Linux; skipping");
        return;
    };
    let src = text("ptop.sv", PROFILED);
    let mut opts = host.options(1);
    opts.profile = Some(epitome_sv::model::TICKS);
    let b = build("ptop", netlist_with(&src, "ptop", opts), &host);
    let plain = build("ptop_plain", netlist_with(&src, "ptop", host.options(1)), &host);
    let io = Io::new(&b.net, Some("clk")).unwrap();
    let mut rng = Rng(0x3C6E_F372_FE94_F82B);
    let cycles = 50;
    let stim: Vec<u32> = (0..cycles).map(|_| rng.next() as u32).collect();
    let out = b.run(&io, cycles, 1, &stim);
    assert_eq!(out, plain.run(&io, cycles, 1, &stim));
    // The model: the inputs reach the leaves' registers though only they
    // read them (copied before the clocked processes, not when settling).
    let (mut t, mut q) = ([0u32; 4], [0u32; 4]);
    for (c, &a) in stim.iter().enumerate() {
        for k in 0..4 {
            (t[k], q[k]) = (a.wrapping_add(k as u32).wrapping_mul(3).wrapping_add(1), (t[k] ^ 0x55).wrapping_mul(3).wrapping_add(1));
        }
        assert_eq!(&out[c * 4..c * 4 + 4], &q[..], "cycle {c}");
    }
    let t = b.time(&io, cycles, 1, &stim, 2);
    for p in &t.profile {
        eprintln!("{:8.1} {:8.1} {:5.2}  {}", p.ticks, p.own, p.calls, p.what);
    }
    let has = |what: &str| t.profile.iter().find(|p| p.what == what).unwrap_or_else(|| panic!("no counter {what}"));
    for what in ["step ff", "step commit", "step settle again"] {
        assert!(has(what).calls > 0.0, "{what} is called");
    }
    // Only clocked processes read the input and the clock: neither a new
    // input nor an edge needs settling, only a commit does.
    assert!(has("step settle").calls < 0.5, "settles {} times a cycle", has("step settle").calls);
    // The array may be called as one or an instance at a time.
    let leaves: Vec<_> = t.profile.iter().filter(|p| p.counter.func == "ff" && p.counter.spec == "leaf").collect();
    assert!(leaves.len() == 2 || leaves.len() == 8, "{} leaves' ff counters", leaves.len());
    for p in &leaves {
        assert!(p.counter.path.starts_with("g[") && p.counter.path.contains(".m.l"), "{}", p.counter.path);
        let within = &t.profile[p.counter.within.expect("a leaf's ff is within its parent's")];
        assert_eq!(within.counter.func, "ff");
        assert_eq!(within.counter.spec, "mid");
        assert!(p.calls > 0.0);
    }
    for p in &t.profile {
        assert!(p.own >= 0.0, "{}: own {} of {}", p.what, p.own, p.ticks);
    }
}
