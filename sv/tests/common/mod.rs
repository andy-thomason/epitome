//! What the netlist tests share, beyond the library's own way of building
//! and running a design (`epitome_sv::model`), Verilator's model of it
//! (`epitome_sv::verilator`) and stimulus (`epitome_sv::stimulus`): the
//! switches the tests take from the environment, and checks of
//! Epitome's code generation against the system's C compiler.
//!
//! The netlist's `Module` is optimised, compiled and loaded in this
//! process: no IR text, assembly, object file or linker. With
//! `NETLIST_CC=1` it is also written as an object, linked by the system's
//! `cc -shared` and `dlopen`ed, and each run of the loaded code checked
//! against a run of that on the same state. `NETLIST_IR=1` and
//! `NETLIST_ASM=1` write the IR and the assembly as text, for reading.
#![allow(dead_code)]

use std::ffi::{CString, c_char, c_int, c_void};
use std::path::{Path, PathBuf};
use std::process::Command;

#[allow(unused_imports)]
pub use epitome_sv::model::{Host, Io, Model, Sources, State, words};
pub use epitome_sv::stimulus::*;
use epitome_sv::netlist::{self, GROUP, Netlist};
use epitome_sv::verilator::{Verilated, Verilator};

unsafe extern "C" {
    fn dlopen(path: *const c_char, flags: c_int) -> *mut c_void;
    fn dlsym(handle: *mut c_void, name: *const c_char) -> *mut c_void;
    fn dlerror() -> *const c_char;
}

const RTLD_NOW: c_int = 2;

/// This machine, if the tests can run here. On x86-64, AVX2 unless
/// `NETLIST_MATTR` names other features, as `+avx512f,+avx512bw`.
pub fn host() -> Option<Host> {
    Host::with_mattr(&std::env::var("NETLIST_MATTR").unwrap_or_else(|_| if cfg!(target_arch = "x86_64") { "+avx2".into() } else { String::new() }))
}

/// Bit planes where the target has mask registers: AArch64, and x86-64
/// with AVX-512; a group of simulations ([`GROUP`]) at once, or as many as
/// `NETLIST_SIMS` says.
pub fn netlist_options() -> netlist::Options {
    let masks = host().is_some_and(|h| h.has_masks());
    let recognise = std::env::var("NETLIST_RECOGNISE").map_or(true, |v| v != "0");
    // NETLIST_GUARD_MIN: the smallest arm guarded, 0 for none.
    let guard_min = std::env::var("NETLIST_GUARD_MIN").ok().and_then(|v| v.parse().ok()).unwrap_or(netlist::GUARD_MIN);
    // NETLIST_PROFILE=1: each child's call timed (see `time_replay`).
    // (On x86-64, a function, `sv_ticks`: the object writer does not
    // assemble the inline assembly rdtsc becomes.)
    let profile = std::env::var("NETLIST_PROFILE").is_ok_and(|v| v == "1").then_some(if cfg!(target_arch = "x86_64") {
        "sv_ticks"
    } else {
        "llvm.readcyclecounter"
    });
    // NETLIST_BRANCH_MIN: the smallest arm of scalar code behind a branch.
    let branch_min = std::env::var("NETLIST_BRANCH_MIN").ok().and_then(|v| v.parse().ok()).unwrap_or(netlist::BRANCH_MIN);
    // NETLIST_SIMS: simulations at once, the lanes of a state (fewer where
    // the state would not fit `netlist::STATE_MAX`).
    let sims = std::env::var("NETLIST_SIMS").ok().and_then(|v| v.parse().ok()).unwrap_or(GROUP);
    netlist::Options { bit_planes: masks, recognise, guard_min, branch_min, sims, profile }
}

/// [`netlist_options`] for `sims` simulations at once.
pub fn options_for(sims: usize) -> netlist::Options {
    netlist::Options { sims, ..netlist_options() }
}

/// One source held in memory.
pub fn text(name: &str, text: &str) -> Sources {
    let mut s = Sources::default();
    s.text(name, text);
    s
}

/// Elaborates `src` with `top` and compiles its netlist.
pub fn netlist_of(src: &Sources, top: &str) -> Netlist {
    netlist_with(src, top, netlist_options())
}

pub fn netlist_with(src: &Sources, top: &str, opts: netlist::Options) -> Netlist {
    epitome_sv::model::netlist(src, top, &opts).unwrap_or_else(|e| panic!("{top}: {e}"))
}

/// The tests' temporary directory.
pub fn tmp_dir() -> PathBuf {
    let tmp = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("netlist");
    std::fs::create_dir_all(&tmp).unwrap();
    tmp
}

/// Optimises and compiles the netlist's IR, and loads it.
pub fn build(name: &str, net: Netlist, host: &Host) -> Model {
    let tmp = tmp_dir();
    // The IR as text only to read (NETLIST_IR=1) or for the system's C
    // compiler (NETLIST_CC_IR=1).
    let cc_ir = std::env::var("NETLIST_CC_IR").is_ok_and(|v| v == "1");
    if cc_ir || std::env::var("NETLIST_IR").is_ok_and(|v| v == "1") {
        let mut m = net.module.clone();
        m.triple = Some(host.triple.as_bytes().to_vec());
        std::fs::write(tmp.join(format!("{name}.ll")), m.to_string()).unwrap();
    }
    // NETLIST_NO_OPT=1 leaves the IR as generated, for telling a bug in
    // the optimiser from one in code generation.
    let optimise = std::env::var("NETLIST_NO_OPT").map_or(true, |v| v != "1");
    let (c, times) = epitome_sv::model::compile(&net, host, optimise).unwrap_or_else(|e| panic!("{name}: {e}"));
    if std::env::var_os("NETLIST_ASM").is_some() {
        std::fs::write(tmp.join(format!("{name}.s")), c.assembly(&host.target).unwrap().as_bytes()).unwrap();
    }
    // NETLIST_CC_IR=1 has the system's C compiler build the IR instead, as
    // a check of Epitome's code generation.
    if cc_ir {
        let functions = epitome_sv::model::functions(&c, host).unwrap();
        let obj = tmp.join(format!("{name}_cc.o"));
        let st = Command::new("cc").args(["-O1", "-c", "-o"]).arg(&obj).arg(tmp.join(format!("{name}.ll"))).status().unwrap();
        assert!(st.success(), "cc on {name}.ll");
        let (eval, prof) = linked(&format!("{name}_cc"), &obj, &net, &tmp);
        return unsafe { Model::from_parts(net, eval, functions, prof) };
    }
    let mut model = Model::load(net, host, &c, times).unwrap_or_else(|e| panic!("{name}: {e}"));
    let t = model.times;
    eprintln!(
        "{name}: optimised in {:.1} ms, compiled in {:.1} ms, loaded in {:.1} ms",
        t.optimise.as_secs_f64() * 1e3,
        t.compile.as_secs_f64() * 1e3,
        t.load.as_secs_f64() * 1e3
    );
    if std::env::var_os("NETLIST_CC").is_some() {
        let obj = tmp.join(format!("{name}.o"));
        std::fs::write(&obj, c.object_file(&host.target).unwrap()).unwrap();
        let other = linked(name, &obj, &model.net, &tmp).0;
        unsafe { model.cross_check(other) };
    }
    model
}

/// Object `obj` linked by `cc -shared` (with `sv_ticks`, if the netlist
/// calls it) and `dlopen`ed: the netlist's entry, and its profile's
/// counters if it has them.
fn linked(name: &str, obj: &Path, net: &Netlist, tmp: &Path) -> (unsafe extern "C" fn(*mut u8), *mut i64) {
    let lib = tmp.join(format!("lib{name}.so"));
    let mut link = Command::new("cc");
    link.arg("-shared").arg("-o").arg(&lib).arg(obj);
    if net.module.find_global("sv_ticks").is_some() {
        let c = tmp.join("sv_ticks.c");
        std::fs::write(&c, "#include <x86intrin.h>\nlong sv_ticks(void) { return (long)__rdtsc(); }\n").unwrap();
        link.arg("-O2").arg(&c);
    }
    let st = link.status().unwrap();
    assert!(st.success(), "linking {name}");
    unsafe {
        let path = CString::new(lib.to_str().unwrap()).unwrap();
        let h = dlopen(path.as_ptr(), RTLD_NOW);
        assert!(!h.is_null(), "{:?}", std::ffi::CStr::from_ptr(dlerror()));
        let sym = CString::new(net.entry.as_str()).unwrap();
        let f = dlsym(h, sym.as_ptr());
        assert!(!f.is_null(), "no {}", net.entry);
        let prof = dlsym(h, c"sv_prof".as_ptr()) as *mut i64;
        (std::mem::transmute::<*mut c_void, unsafe extern "C" fn(*mut u8)>(f), prof)
    }
}

pub fn report(name: &str, b: &Model) {
    let sizes: Vec<String> = b.functions.iter().map(|(n, s)| format!("{n} {s} bytes")).collect();
    eprintln!("{name}: {} bytes of state for {} lanes; {}", b.net.size, b.net.lanes, sizes.join(", "));
}

/// Verilator's model of `top` from `src`, built in the tests' directory as
/// `name`; `None` if Verilator is not here (`$VERILATOR`,
/// `$VERILATOR_ROOT/bin`, or on the path).
pub fn verilate(name: &str, src: &Sources, top: &str, io: &Io) -> Option<Verilated> {
    let Some(vl) = Verilator::find() else {
        eprintln!("{name}: no verilator; not checked against it");
        return None;
    };
    Some(vl.build(&tmp_dir().join("vl").join(name), src, top, io).unwrap_or_else(|e| panic!("{e}")))
}

/// Runs the model on the same stimulus as Verilator's, and checks each
/// output word against Verilator's; `show` turns an instance's inputs
/// into text for a mismatch.
pub fn check_against(b: &Model, io: &Io, v: &Verilated, cycles: usize, n: usize, stim: &[u32], show: &dyn Fn(&[u32]) -> String) {
    let want = v.run(cycles, n, stim).unwrap_or_else(|e| panic!("{e}"));
    let got = b.run(io, cycles, n, stim);
    let (bad, first) = epitome_sv::model::compare(io, n, &got, &want, 10);
    let iw = io.in_words();
    for m in &first {
        let base = (m.cycle * n + m.instance) * iw;
        eprintln!(
            "cycle {}, instance {}: {} is {:x?}, Verilator {:x?}; {}",
            m.cycle,
            m.instance,
            m.port,
            m.ours,
            m.theirs,
            show(&stim[base..base + iw])
        );
    }
    assert_eq!(bad, 0, "{bad} instance-cycles differ from Verilator");
}

/// The model's time an instance a cycle (`Model::time`), with its
/// profile's counters printed if it has one. `NETLIST_REPS` multiplies the
/// repetitions, for a profiler to watch.
pub fn time_replay(b: &Model, io: &Io, cycles: usize, n: usize, stim: &[u32], reps: usize) -> f64 {
    let reps = reps * std::env::var("NETLIST_REPS").ok().and_then(|r| r.parse().ok()).unwrap_or(1);
    let t = b.time(io, cycles, n, stim, reps);
    for p in &t.profile {
        eprintln!("profile: {:10.1} ticks an instance a cycle, {:5.2} calls a cycle  {}", p.ticks, p.calls, p.what);
    }
    t.ns
}

/// Verilator's time on the same stimulus, as text.
pub fn bench(v: &Verilated, cycles: usize, n: usize, stim: &[u32], reps: usize) -> String {
    v.bench(cycles, n, stim, reps).map_or_else(|e| e, |b| b.to_string())
}
