//! What the netlist tests share: building a netlist's IR with Epitome and
//! loading it, a group's state, and Verilator's model of the same design,
//! run as a program, as the reference to check it against.
#![allow(dead_code)]

use std::ffi::{CString, c_char, c_int, c_void};
use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::process::Command;

use epitome_codegen::object::SectionKind;
use epitome_codegen::target::Target;
use epitome_sv::design::Dir;
use epitome_sv::netlist::{self, Field, GROUP, Lane, Netlist};
use epitome_sv::preprocess::{Options, preprocess_files};
use epitome_sv::source::SourceMap;

unsafe extern "C" {
    fn dlopen(path: *const c_char, flags: c_int) -> *mut c_void;
    fn dlsym(handle: *mut c_void, name: *const c_char) -> *mut c_void;
    fn dlerror() -> *const c_char;
}

const RTLD_NOW: c_int = 2;

/// This machine's target, triple and features, if the tests can run here.
pub fn host() -> Option<(Target, &'static str, Vec<String>)> {
    let cc = Command::new("cc").arg("--version").output().is_ok_and(|o| o.status.success());
    if !cc {
        return None;
    }
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
pub fn netlist_options() -> netlist::Options {
    let x86_without_masks =
        cfg!(target_arch = "x86_64") && !std::env::var("NETLIST_MATTR").is_ok_and(|m| m.contains("avx512f"));
    let recognise = std::env::var("NETLIST_RECOGNISE").map_or(true, |v| v != "0");
    netlist::Options { bit_planes: !x86_without_masks, recognise }
}

/// Elaborates `files` with `top` and compiles its netlist, on a thread with
/// room for the elaborator's recursion.
pub fn netlist_of(files: Vec<PathBuf>, include_dirs: Vec<PathBuf>, top: &str) -> Netlist {
    netlist_with(files, include_dirs, top, netlist_options())
}

pub fn netlist_with(files: Vec<PathBuf>, include_dirs: Vec<PathBuf>, top: &str, layout: netlist::Options) -> Netlist {
    let top = top.to_string();
    std::thread::Builder::new()
        .stack_size(256 << 20)
        .spawn(move || {
            let map = SourceMap::new();
            let opts = Options { include_dirs, ..Default::default() };
            let out = preprocess_files(&map, &files, &opts).unwrap_or_else(|e| panic!("{e}"));
            let ast = epitome_sv::parse::parse(&out).unwrap_or_else(|e| panic!("{}", map.render(&e)));
            let d = epitome_sv::design::elab::elaborate_tops(&ast, &out, &[top]).unwrap_or_else(|e| panic!("{}", map.render(&e)));
            netlist::compile_with(&d, d.tops[0], &layout).unwrap_or_else(|e| panic!("{e}"))
        })
        .unwrap()
        .join()
        .unwrap()
}

/// A compiled and loaded netlist.
pub struct Built {
    pub net: Netlist,
    pub eval: unsafe extern "C" fn(*mut u8),
    /// Each function's name and size in bytes of machine code.
    pub sizes: Vec<(String, u64)>,
}

/// Optimises, compiles and links the netlist's IR, and loads it.
pub fn build(name: &str, net: Netlist, t: &Target, triple: &str, mattr: &[String]) -> Built {
    let tmp = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("netlist");
    std::fs::create_dir_all(&tmp).unwrap();
    let ir = format!("target triple = \"{triple}\"\n{}", net.ir);
    std::fs::write(tmp.join(format!("{name}.ll")), &ir).unwrap();
    let mut m = epitome_ir::parse_module(&ir).unwrap_or_else(|e| panic!("{name}.ll: {e:?}"));
    epitome_opt::pipeline::run(&mut m, &Default::default());
    let opts = epitome_codegen::Options { mattr: mattr.to_vec(), ..Default::default() };
    let c = epitome_codegen::compile(t, &mut m, &opts).unwrap_or_else(|e| panic!("{name}: {e}"));
    std::fs::write(tmp.join(format!("{name}.s")), &c.asm).unwrap();
    let object = c.object(t).unwrap();
    let mut sizes: Vec<(String, u64)> = object
        .symbols
        .iter()
        .filter(|s| s.name.contains("sv_eval_") && s.section.is_some_and(|i| object.sections[i].kind == SectionKind::Text))
        .map(|s| (s.name.trim_start_matches('_').to_string(), s.size))
        .collect();
    sizes.sort();
    let obj = tmp.join(format!("{name}.o"));
    std::fs::write(&obj, c.object_file(t).unwrap()).unwrap();
    let lib = tmp.join(format!("lib{name}.so"));
    let st = Command::new("cc").arg("-shared").arg("-o").arg(&lib).arg(&obj).status().unwrap();
    assert!(st.success(), "linking {name}");
    let eval = unsafe {
        let path = CString::new(lib.to_str().unwrap()).unwrap();
        let h = dlopen(path.as_ptr(), RTLD_NOW);
        assert!(!h.is_null(), "{:?}", std::ffi::CStr::from_ptr(dlerror()));
        let sym = CString::new(net.entry.as_str()).unwrap();
        let f = dlsym(h, sym.as_ptr());
        assert!(!f.is_null(), "no {}", net.entry);
        std::mem::transmute::<*mut c_void, unsafe extern "C" fn(*mut u8)>(f)
    };
    Built { net, eval, sizes }
}

/// A group's state, aligned for 512-bit loads.
pub struct State {
    pub ptr: *mut u8,
    layout: std::alloc::Layout,
}

impl State {
    pub fn new(size: usize) -> State {
        let layout = std::alloc::Layout::from_size_align(size.max(64), 64).unwrap();
        State { ptr: unsafe { std::alloc::alloc_zeroed(layout) }, layout }
    }

    pub fn put(&mut self, net: &Netlist, port: &str, n: usize, v: u64) {
        let f = net.port(port).unwrap_or_else(|| panic!("no port {port}"));
        unsafe { f.put(self.ptr, n, v) };
    }

    /// A value wider than 64 bits, as 64-bit words, low first.
    pub fn put_wide(&mut self, net: &Netlist, port: &str, n: usize, v: &[u64]) {
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

    pub fn get(&self, net: &Netlist, port: &str, n: usize) -> u64 {
        let f = net.port(port).unwrap_or_else(|| panic!("no port {port}"));
        unsafe { f.get(self.ptr, n) }
    }
}

impl State {
    /// Instance `n`'s value of `f`, as 32-bit words, low first.
    pub fn put_words(&mut self, f: &Field, n: usize, v: &[u32]) {
        match f.lane {
            Lane::W32 => {
                for k in 0..f.pieces() {
                    let mut piece = v[k];
                    let top = f.width as usize - 32 * k;
                    if top < 32 {
                        piece &= (1u32 << top) - 1;
                    }
                    unsafe { (self.ptr.add(f.at(k, n)) as *mut u32).write(piece) };
                }
            }
            _ => unsafe { f.put(self.ptr, n, v[0] as u64) },
        }
    }

    pub fn get_words(&self, f: &Field, n: usize, out: &mut Vec<u32>) {
        match f.lane {
            Lane::W32 => {
                for k in 0..f.pieces() {
                    out.push(unsafe { (self.ptr.add(f.at(k, n)) as *const u32).read() });
                }
            }
            _ => out.push(unsafe { f.get(self.ptr, n) } as u32),
        }
    }
}

impl Drop for State {
    fn drop(&mut self) {
        unsafe { std::alloc::dealloc(self.ptr, self.layout) };
    }
}

pub struct Rng(pub u64);

impl Rng {
    pub fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
}

pub fn report(name: &str, b: &Built) {
    let sizes: Vec<String> = b.sizes.iter().map(|(n, s)| format!("{n} {s} bytes")).collect();
    eprintln!("{name}: {} bytes of state a group; {}", b.net.size, sizes.join(", "));
}


/// Words in a `w`-bit value.
pub fn words(w: u32) -> usize {
    (w as usize).div_ceil(32).max(1)
}

/// The tests' temporary directory.
pub fn tmp_dir() -> PathBuf {
    let tmp = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("netlist");
    std::fs::create_dir_all(&tmp).unwrap();
    tmp
}

/// Verilator's model of a top-level module, built as a program that reads
/// stimulus for many instances from a file and writes what each gives.
pub struct Verilated {
    bin: PathBuf,
    /// The inputs the stimulus holds (not the clock), and the outputs, with
    /// their widths, in the netlist's port order.
    pub ins: Vec<(String, u32)>,
    pub outs: Vec<(String, u32)>,
    pub clock: Option<String>,
}

/// Verilator, if it runs here: `$VERILATOR`, or `verilator` on the path.
fn verilator() -> Option<String> {
    let v = std::env::var("VERILATOR").unwrap_or_else(|_| "verilator".into());
    Command::new(&v).arg("--version").output().ok().filter(|o| o.status.success()).map(|_| v)
}

/// Builds Verilator's model of `top` (whose ports are `ports`), with a C++
/// driver written for those ports. With `clock`, each cycle sets the
/// inputs with the clock low, evaluates, raises the clock and evaluates
/// again; without, it sets the inputs and evaluates. `None` if Verilator
/// is not here.
pub fn verilate(name: &str, files: &[PathBuf], incs: &[PathBuf], top: &str, ports: &[Field], clock: Option<&str>) -> Option<Verilated> {
    let Some(vl) = verilator() else {
        eprintln!("{name}: no verilator; not checked against it");
        return None;
    };
    let dir = tmp_dir().join("vl").join(name);
    std::fs::create_dir_all(&dir).unwrap();
    let ins: Vec<(String, u32)> =
        ports.iter().filter(|p| p.dir == Dir::Input && Some(p.name.as_str()) != clock).map(|p| (p.name.clone(), p.width)).collect();
    let outs: Vec<(String, u32)> = ports.iter().filter(|p| p.dir == Dir::Output).map(|p| (p.name.clone(), p.width)).collect();
    let mut put = String::new();
    let mut at = 0;
    for (p, w) in &ins {
        let _ = match w {
            0..=32 => writeln!(put, "    t->{p} = v[{at}];"),
            33..=64 => writeln!(put, "    t->{p} = (uint64_t)v[{at}] | (uint64_t)v[{}] << 32;", at + 1),
            _ => writeln!(put, "    for (int j = 0; j < {}; j++) t->{p}[j] = v[{at} + j];", words(*w)),
        };
        at += words(*w);
    }
    let in_words = at;
    let mut get = String::new();
    at = 0;
    for (p, w) in &outs {
        let _ = match w {
            0..=32 => writeln!(get, "    v[{at}] = t->{p};"),
            33..=64 => writeln!(get, "    v[{at}] = (uint32_t)t->{p}; v[{}] = (uint32_t)(t->{p} >> 32);", at + 1),
            _ => writeln!(get, "    for (int j = 0; j < {}; j++) v[{at} + j] = t->{p}[j];", words(*w)),
        };
        at += words(*w);
    }
    let out_words = at;
    let step = match clock {
        Some(c) => format!("t->{c} = 0; t->eval(); t->{c} = 1; t->eval();"),
        None => "t->eval();".into(),
    };
    let driver = format!(
        r#"// Written by epitome-sv's netlist tests: Verilator's model of {top},
// run on stimulus for many instances.
#include <chrono>
#include <cstdint>
#include <cstdio>
#include <cstdlib>
#include <vector>
#include "verilated.h"
#include "V{top}.h"
typedef V{top} Top;
static const int IN = {in_words}, OUT = {out_words};
static void put(Top *t, const uint32_t *v) {{
{put}}}
static void get(Top *t, uint32_t *v) {{
{get}}}
// driver run|bench CYCLES INSTANCES IN [OUT|REPS]
int main(int argc, char **argv) {{
    bool bench = argv[1][0] == 'b';
    int cycles = atoi(argv[2]), n = atoi(argv[3]);
    std::vector<uint32_t> in((size_t)cycles * n * IN), out((size_t)cycles * n * OUT);
    FILE *f = fopen(argv[4], "rb");
    if (!f || fread(in.data(), 4, in.size(), f) != in.size()) return 2;
    fclose(f);
    VerilatedContext ctx;
    std::vector<Top *> m(n);
    for (auto &t : m) t = new Top(&ctx);
    int reps = bench ? atoi(argv[5]) : 1;
    auto t0 = std::chrono::steady_clock::now();
    for (int r = 0; r < reps; r++)
        for (int c = 0; c < cycles; c++)
            for (int i = 0; i < n; i++) {{
                Top *t = m[i];
                put(t, &in[((size_t)c * n + i) * IN]);
                {step}
                get(t, &out[((size_t)c * n + i) * OUT]);
            }}
    double ns = std::chrono::duration<double, std::nano>(std::chrono::steady_clock::now() - t0).count();
    if (bench) {{
        printf("%.2f ns an instance a cycle; %zu bytes a model\n", ns / ((double)reps * cycles * n), sizeof(Top));
        return 0;
    }}
    f = fopen(argv[5], "wb");
    if (!f || fwrite(out.data(), 4, out.size(), f) != out.size()) return 3;
    fclose(f);
    return 0;
}}
"#
    );
    let cpp = dir.join("driver.cpp");
    let mut cmd: Vec<String> = [vl.as_str(), "--cc", "--exe", "--build", "-j", "8", "-O3", "--x-assign", "fast", "--x-initial", "fast", "--noassert"]
        .iter()
        .map(|s| s.to_string())
        .collect();
    cmd.extend(["-Wno-fatal", "-Wno-lint", "-Wno-style", "-CFLAGS", "-O3", "--top-module", top, "-o", "vmodel"].iter().map(|s| s.to_string()));
    cmd.push("-Mdir".into());
    cmd.push(dir.join("obj").display().to_string());
    cmd.extend(incs.iter().map(|i| format!("-I{}", i.display())));
    cmd.extend(files.iter().map(|f| f.display().to_string()));
    cmd.push(cpp.display().to_string());
    // Built again only when the driver, the command or a source changed.
    let mut stamp = format!("{driver}\n{}\n", cmd.join(" "));
    for f in files {
        let m = std::fs::metadata(f).unwrap_or_else(|e| panic!("{}: {e}", f.display()));
        let _ = writeln!(stamp, "{} {} {:?}", f.display(), m.len(), m.modified().ok());
    }
    let bin = dir.join("obj").join("vmodel");
    let stamp_file = dir.join("stamp");
    if !bin.exists() || std::fs::read_to_string(&stamp_file).ok().as_deref() != Some(stamp.as_str()) {
        std::fs::write(&cpp, &driver).unwrap();
        let out = Command::new(&cmd[0]).args(&cmd[1..]).output().unwrap();
        assert!(out.status.success(), "verilator for {name}:\n{}{}", String::from_utf8_lossy(&out.stdout), String::from_utf8_lossy(&out.stderr));
        std::fs::write(&stamp_file, &stamp).unwrap();
    }
    Some(Verilated { bin, ins, outs, clock: clock.map(String::from) })
}

impl Verilated {
    pub fn in_words(&self) -> usize {
        self.ins.iter().map(|p| words(p.1)).sum()
    }

    pub fn out_words(&self) -> usize {
        self.outs.iter().map(|p| words(p.1)).sum()
    }

    fn write_stimulus(&self, stim: &[u32]) -> PathBuf {
        let path = self.bin.with_file_name("stimulus.bin");
        let bytes: Vec<u8> = stim.iter().flat_map(|w| w.to_le_bytes()).collect();
        std::fs::write(&path, bytes).unwrap();
        path
    }

    /// What each instance's outputs are at the end of each cycle, for
    /// `stim`: for each cycle, for each instance, its inputs' words.
    pub fn run(&self, cycles: usize, n: usize, stim: &[u32]) -> Vec<u32> {
        assert_eq!(stim.len(), cycles * n * self.in_words());
        let input = self.write_stimulus(stim);
        let output = self.bin.with_file_name("response.bin");
        let st = Command::new(&self.bin).args(["run", &cycles.to_string(), &n.to_string()]).arg(&input).arg(&output).status().unwrap();
        assert!(st.success(), "{} failed", self.bin.display());
        let bytes = std::fs::read(&output).unwrap();
        bytes.chunks_exact(4).map(|c| u32::from_le_bytes(c.try_into().unwrap())).collect()
    }

    /// The model's time an instance a cycle, on `stim` run `reps` times.
    pub fn bench(&self, cycles: usize, n: usize, stim: &[u32], reps: usize) -> String {
        let input = self.write_stimulus(stim);
        let out = Command::new(&self.bin).args(["bench", &cycles.to_string(), &n.to_string()]).arg(&input).arg(reps.to_string()).output().unwrap();
        String::from_utf8_lossy(&out.stdout).trim().to_string()
    }
}

/// Runs the netlist on the same stimulus as [`Verilated::run`], one group
/// at a time, and checks each output word against Verilator's; `name`
/// labels a mismatch, and `show` turns an instance's inputs into text.
pub fn check_against(b: &Built, v: &Verilated, cycles: usize, n: usize, stim: &[u32], show: &dyn Fn(&[u32]) -> String) {
    assert_eq!(n % GROUP, 0);
    let want = v.run(cycles, n, stim);
    let net = &b.net;
    let field = |name: &str| net.port(name).unwrap_or_else(|| panic!("no port {name}")).clone();
    let ins: Vec<Field> = v.ins.iter().map(|p| field(&p.0)).collect();
    let outs: Vec<Field> = v.outs.iter().map(|p| field(&p.0)).collect();
    let clock = v.clock.as_deref().map(field);
    let (iw, ow) = (v.in_words(), v.out_words());
    let mut bad = 0;
    for g in 0..n / GROUP {
        let mut st = State::new(net.size);
        for c in 0..cycles {
            for k in 0..GROUP {
                let i = g * GROUP + k;
                let mut at = (c * n + i) * iw;
                for f in &ins {
                    st.put_words(f, k, &stim[at..at + words(f.width)]);
                    at += words(f.width);
                }
            }
            match &clock {
                Some(clk) => {
                    for level in [0, 1] {
                        for k in 0..GROUP {
                            unsafe { clk.put(st.ptr, k, level) };
                        }
                        unsafe { (b.eval)(st.ptr) };
                    }
                }
                None => unsafe { (b.eval)(st.ptr) },
            }
            for k in 0..GROUP {
                let i = g * GROUP + k;
                let mut got = Vec::with_capacity(ow);
                for f in &outs {
                    st.get_words(f, k, &mut got);
                }
                let w = &want[(c * n + i) * ow..(c * n + i + 1) * ow];
                if got != w {
                    let mut at = 0;
                    for f in &outs {
                        let r = at..at + words(f.width);
                        if got[r.clone()] != w[r.clone()] && bad < 10 {
                            let base = (c * n + i) * iw;
                            eprintln!(
                                "cycle {c}, instance {i}: {} is {:x?}, Verilator {:x?}; {}",
                                f.name,
                                &got[r.clone()],
                                &w[r.clone()],
                                show(&stim[base..base + iw])
                            );
                        }
                        at = r.end;
                    }
                    bad += 1;
                }
            }
        }
    }
    assert_eq!(bad, 0, "{bad} instance-cycles differ from Verilator");
}

/// The common files of CVA6's cv64a6_imafdc_sv39 configuration, and the
/// include directories.
pub fn cva6_files(cva6: &Path, rest: &[&str]) -> (Vec<PathBuf>, Vec<PathBuf>) {
    let c = |p: &str| cva6.join(p);
    let mut files: Vec<PathBuf> = [
        "core/include/config_pkg.sv",
        "core/include/cv64a6_imafdc_sv39_config_pkg.sv",
        "core/include/riscv_pkg.sv",
        "core/include/build_config_pkg.sv",
        "core/include/ariane_pkg.sv",
        "vendor/pulp-platform/common_cells/src/cf_math_pkg.sv",
    ]
    .iter()
    .map(|p| c(p))
    .collect();
    files.extend(rest.iter().map(|p| if p.starts_with('/') { PathBuf::from(p) } else { c(p) }));
    (files, vec![c("core/include"), c("vendor/pulp-platform/common_cells/include")])
}
