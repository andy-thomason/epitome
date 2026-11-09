//! The compiled netlist (`src/netlist.rs`), built by Epitome, loaded and
//! run, and checked instance by instance: a module of operators against
//! values worked out here, and CVA6's `pmp_entry` (when `CVA6_DIR` points
//! at a CVA6 checkout) against a scalar model of it.
//!
//! The code is compiled and loaded in this process (`epitome_codegen::load`):
//! no assembly, object file or linker. With `NETLIST_CC=1` it is also
//! written as an object, linked by the system's `cc -shared` and `dlopen`ed,
//! and each run of the loaded code checked against a run of that on the
//! same state. `NETLIST_ASM=1` writes the assembly beside the IR.
//!
//! On an AArch64 Mac or x86-64 Linux; elsewhere the tests skip. x86-64 uses
//! AVX2 unless `NETLIST_MATTR` names other features.

use std::ffi::{CString, c_char, c_int, c_void};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Instant;

use epitome_codegen::load::{self, Loaded};
use epitome_codegen::object::SectionKind;
use epitome_codegen::target::Target;
use epitome_sv::netlist::{self, GROUP, Netlist};
use epitome_sv::preprocess::{Options, preprocess_files};
use epitome_sv::source::SourceMap;

unsafe extern "C" {
    fn dlopen(path: *const c_char, flags: c_int) -> *mut c_void;
    fn dlsym(handle: *mut c_void, name: *const c_char) -> *mut c_void;
    fn dlerror() -> *const c_char;
}

const RTLD_NOW: c_int = 2;

/// This machine's target, triple and features, if the tests can run here.
fn host() -> Option<(Target, &'static str, Vec<String>)> {
    if cfg!(all(target_arch = "x86_64", target_os = "linux")) {
        // NETLIST_MATTR picks the features, as `+avx512f,+avx512bw`.
        let mattr = std::env::var("NETLIST_MATTR").unwrap_or_else(|_| "+avx2".into());
        Some((Target::x86_64(), "x86_64-unknown-linux-gnu", mattr.split(',').map(String::from).collect()))
    } else if cfg!(all(target_arch = "aarch64", target_os = "macos")) {
        Some((Target::aarch64(), "arm64-apple-macosx14.0.0", Vec::new()))
    } else {
        None
    }
}

/// Bit planes where the target has mask registers: AArch64, and x86-64
/// with AVX-512.
fn netlist_options() -> netlist::Options {
    let x86_without_masks =
        cfg!(target_arch = "x86_64") && !std::env::var("NETLIST_MATTR").is_ok_and(|m| m.contains("avx512f"));
    let recognise = std::env::var("NETLIST_RECOGNISE").map_or(true, |v| v != "0");
    netlist::Options { bit_planes: !x86_without_masks, recognise }
}

/// Elaborates `files` with `top` and compiles its netlist, on a thread with
/// room for the elaborator's recursion.
fn netlist_of(files: Vec<PathBuf>, include_dirs: Vec<PathBuf>, top: &str) -> Netlist {
    netlist_with(files, include_dirs, top, netlist_options())
}

fn netlist_with(files: Vec<PathBuf>, include_dirs: Vec<PathBuf>, top: &str, layout: netlist::Options) -> Netlist {
    let top = top.to_string();
    std::thread::Builder::new()
        .stack_size(256 << 20)
        .spawn(move || {
            let map = SourceMap::new();
            let opts = Options { include_dirs, ..Default::default() };
            let out = preprocess_files(&map, &files, &opts).unwrap_or_else(|e| panic!("{e}"));
            let ast = epitome_sv::parse::parse(&out).unwrap_or_else(|e| panic!("{}", map.render(&e)));
            let d = epitome_sv::design::elab::elaborate_tops(&ast, &out, &[top])
                .unwrap_or_else(|e| panic!("{}", map.render(&e)));
            netlist::compile_with(&d, d.tops[0], &layout).unwrap_or_else(|e| panic!("{e}"))
        })
        .unwrap()
        .join()
        .unwrap()
}

/// A compiled and loaded netlist.
struct Built {
    net: Netlist,
    eval: unsafe extern "C" fn(*mut u8),
    /// The code `eval` is in.
    _code: Loaded,
    /// With `NETLIST_CC`: the same code linked by `cc` and `dlopen`ed.
    cc_eval: Option<unsafe extern "C" fn(*mut u8)>,
    /// Each function's name and size in bytes of machine code.
    sizes: Vec<(String, u64)>,
}

impl Built {
    /// Runs the entry on `st`; with `NETLIST_CC`, also the linked code on a
    /// copy of it, and checks that the two leave the same state.
    fn run(&self, st: &mut State) {
        let Some(cc_eval) = self.cc_eval else {
            unsafe { (self.eval)(st.ptr) };
            return;
        };
        let theirs = State::new(self.net.size);
        unsafe {
            std::ptr::copy_nonoverlapping(st.ptr, theirs.ptr, self.net.size);
            (self.eval)(st.ptr);
            cc_eval(theirs.ptr);
            let a = std::slice::from_raw_parts(st.ptr, self.net.size);
            let b = std::slice::from_raw_parts(theirs.ptr, self.net.size);
            if let Some(k) = (0..a.len()).find(|&k| a[k] != b[k]) {
                panic!("the loaded code and cc's link of it differ, first at byte {k} of the state");
            }
        }
    }
}

/// Optimises and compiles the netlist's IR, and loads it.
fn build(name: &str, net: Netlist, t: &Target, triple: &str, mattr: &[String]) -> Built {
    let tmp = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("netlist");
    std::fs::create_dir_all(&tmp).unwrap();
    let ir = format!("target triple = \"{triple}\"\n{}", net.ir);
    std::fs::write(tmp.join(format!("{name}.ll")), &ir).unwrap();
    let started = Instant::now();
    let mut m = epitome_ir::parse_module(&ir).unwrap_or_else(|e| panic!("{name}.ll: {e:?}"));
    epitome_opt::pipeline::run(&mut m, &Default::default());
    let optimised = started.elapsed();
    let opts = epitome_codegen::Options { mattr: mattr.to_vec(), no_asm: true, ..Default::default() };
    let c = epitome_codegen::compile(t, &mut m, &opts).unwrap_or_else(|e| panic!("{name}: {e}"));
    let compiled = started.elapsed();
    let code = load::load(t, &c, load::process_symbol).unwrap_or_else(|e| panic!("{name}: loading: {e}"));
    let loaded = started.elapsed();
    eprintln!(
        "{name}: parsed and optimised in {:.1} ms, compiled in {:.1} ms, loaded in {:.1} ms",
        optimised.as_secs_f64() * 1e3,
        (compiled - optimised).as_secs_f64() * 1e3,
        (loaded - compiled).as_secs_f64() * 1e3
    );
    if std::env::var_os("NETLIST_ASM").is_some() {
        std::fs::write(tmp.join(format!("{name}.s")), c.assembly(t).unwrap().as_bytes()).unwrap();
    }
    let object = c.object(t).unwrap();
    let mut sizes: Vec<(String, u64)> = object
        .symbols
        .iter()
        .filter(|s| {
            s.name.contains("sv_eval_") && s.section.is_some_and(|i| object.sections[i].kind == SectionKind::Text)
        })
        .map(|s| (s.name.trim_start_matches('_').to_string(), s.size))
        .collect();
    sizes.sort();
    let eval = unsafe { code.function(&net.entry) }.unwrap_or_else(|| panic!("no {}", net.entry));
    let cc_eval = std::env::var_os("NETLIST_CC").is_some().then(|| linked(name, &c, t, &net.entry, &tmp));
    Built { net, eval, _code: code, cc_eval, sizes }
}

/// The compiled code written as an object, linked by `cc -shared` and
/// `dlopen`ed: function `entry`.
fn linked(
    name: &str,
    c: &epitome_codegen::Compiled,
    t: &Target,
    entry: &str,
    tmp: &Path,
) -> unsafe extern "C" fn(*mut u8) {
    let obj = tmp.join(format!("{name}.o"));
    std::fs::write(&obj, c.object_file(t).unwrap()).unwrap();
    let lib = tmp.join(format!("lib{name}.so"));
    let st = Command::new("cc").arg("-shared").arg("-o").arg(&lib).arg(&obj).status().unwrap();
    assert!(st.success(), "linking {name}");
    unsafe {
        let path = CString::new(lib.to_str().unwrap()).unwrap();
        let h = dlopen(path.as_ptr(), RTLD_NOW);
        assert!(!h.is_null(), "{:?}", std::ffi::CStr::from_ptr(dlerror()));
        let sym = CString::new(entry).unwrap();
        let f = dlsym(h, sym.as_ptr());
        assert!(!f.is_null(), "no {entry}");
        std::mem::transmute::<*mut c_void, unsafe extern "C" fn(*mut u8)>(f)
    }
}

/// A group's state, aligned for 512-bit loads.
struct State {
    ptr: *mut u8,
    layout: std::alloc::Layout,
}

impl State {
    fn new(size: usize) -> State {
        let layout = std::alloc::Layout::from_size_align(size.max(64), 64).unwrap();
        State { ptr: unsafe { std::alloc::alloc_zeroed(layout) }, layout }
    }

    fn put(&mut self, net: &Netlist, port: &str, n: usize, v: u64) {
        let f = net.port(port).unwrap_or_else(|| panic!("no port {port}"));
        unsafe { f.put(self.ptr, n, v) };
    }

    /// A value wider than 64 bits, as 64-bit words, low first.
    fn put_wide(&mut self, net: &Netlist, port: &str, n: usize, v: &[u64]) {
        let f = net.port(port).unwrap_or_else(|| panic!("no port {port}"));
        for k in 0..f.pieces() {
            let word = v.get(k / 2).copied().unwrap_or(0);
            let mut piece = (word >> (32 * (k % 2))) as u32;
            let top = f.width as usize - 32 * k;
            if top < 32 {
                piece &= (1u32 << top) - 1;
            }
            unsafe { (self.ptr.add(f.at(k, n)) as *mut u32).write(piece) };
        }
    }

    fn get(&self, net: &Netlist, port: &str, n: usize) -> u64 {
        let f = net.port(port).unwrap_or_else(|| panic!("no port {port}"));
        unsafe { f.get(self.ptr, n) }
    }
}

impl Drop for State {
    fn drop(&mut self) {
        unsafe { std::alloc::dealloc(self.ptr, self.layout) };
    }
}

struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
}

fn report(name: &str, b: &Built) {
    let sizes: Vec<String> = b.sizes.iter().map(|(n, s)| format!("{n} {s} bytes")).collect();
    eprintln!("{name}: {} bytes of state a group; {}", b.net.size, sizes.join(", "));
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
    output logic [55:0] each
);
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
  end
endmodule
"#;

#[test]
fn operators_lane_by_lane() {
    let Some((t, triple, mattr)) = host() else {
        eprintln!("needs an AArch64 Mac or x86-64 Linux; skipping");
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
        b.run(&mut st);
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
        eprintln!("needs an AArch64 Mac or x86-64 Linux; skipping");
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
        for (p, v) in [("addr_i", addr), ("conf_addr_i", conf), ("conf_addr_prev_i", prev), ("conf_addr_mode_i", mode)]
        {
            st.put(net, p, n, v);
        }
        ins.push((addr, conf, prev, mode));
    }
    b.run(&mut st);
    let mut matches = 0;
    for (n, &(addr, conf, prev, mode)) in ins.iter().enumerate() {
        let want = pmp_reference(addr, conf, prev, mode);
        assert_eq!(
            st.get(net, "match_o", n) == 1,
            want,
            "instance {n}: addr {addr:x} conf {conf:x} prev {prev:x} mode {mode}"
        );
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
        eprintln!("needs an AArch64 Mac or x86-64 Linux; skipping");
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
    b.run(&mut st);
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
        eprintln!("needs an AArch64 Mac or x86-64 Linux; skipping");
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
        let files = vec![
            cva6.join("vendor/pulp-platform/common_cells/src/cf_math_pkg.sv"),
            cva6.join("vendor/pulp-platform/common_cells/src/lzc.sv"),
            src,
        ];
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
            b.run(&mut st);
            outs.push(
                (0..GROUP).map(|n| (st.get(&b.net, "cnt_o", n), st.get(&b.net, "empty_o", n))).collect::<Vec<_>>(),
            );
        }
        for (n, &v) in ins.iter().enumerate() {
            assert_eq!(
                outs[0][n], outs[1][n],
                "{name}: in {v:x}: recognised gives {:?}, its logic {:?}",
                outs[0][n], outs[1][n]
            );
            if v != 0 {
                let want = if mode == 0 { v.trailing_zeros() } else { (v << (64 - w)).leading_zeros() } as u64;
                assert_eq!(outs[0][n].0, want, "{name}: in {v:x}");
            }
        }
        eprintln!(
            "{name}: recognised and its logic agree on {GROUP} inputs; code {} bytes against {}",
            built[0].sizes.iter().map(|s| s.1).sum::<u64>(),
            built[1].sizes.iter().map(|s| s.1).sum::<u64>()
        );
    }
}
