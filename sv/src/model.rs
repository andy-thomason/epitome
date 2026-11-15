//! From SystemVerilog sources to machine code running in this process: the
//! library's way to build a design and run it.
//!
//! - [`Sources`] names what to read: files, and text held in memory (a
//!   wrapper, say), with include directories and macros.
//! - [`elaborate`] preprocesses, parses and elaborates them as one
//!   compilation unit, and [`netlist`] lowers a top-level module to its
//!   netlist ([`crate::netlist`]), an Epitome IR `Module`.
//! - [`Model::build`] optimises that with `epitome-opt`, compiles it with
//!   `epitome-codegen` for this machine ([`Host`]), and loads the code into
//!   this process (`epitome_codegen::load`).
//!
//! Every step is in memory: no IR or assembly text, no object file, no
//! linker.
//!
//! A [`State`] holds the values of a group of independent simulations, the
//! netlist's lanes. [`Io`] says where the top's ports are in it, in the
//! order stimulus is given: for each cycle, for each instance, each input's
//! 32-bit words, low first (the clock aside). [`Model::run`] runs many
//! instances on such stimulus and gives their outputs in the same order;
//! [`Model::time`] times it. [`crate::verilator`] builds Verilator's model
//! of the same sources to check those outputs against.
//!
//! ```no_run
//! use epitome_sv::model::{Host, Io, Model, Sources, netlist};
//! let host = Host::native().expect("an x86-64 Linux or AArch64 macOS machine");
//! let mut src = Sources::default();
//! src.text("add.sv", "module add(input logic [7:0] a, b, output logic [7:0] y); assign y = a + b; endmodule");
//! let model = Model::build(netlist(&src, "add", &host.options(1)).unwrap(), &host).unwrap();
//! let io = Io::new(&model.net, None).unwrap();
//! assert_eq!(model.run(&io, 1, 1, &[2, 3]), [5]);
//! ```

use std::path::PathBuf;
use std::time::{Duration, Instant};

use epitome_codegen::Compiled;
use epitome_codegen::load::{self, Loaded};
use epitome_codegen::object::SectionKind;
use epitome_codegen::target::Target;

use crate::design::{Design, Dir};
use crate::netlist::{self, Field, Netlist};
use crate::preprocess::{Options as PpOptions, preprocess};
use crate::source::{Source as Text, SourceMap};

/// What to elaborate: files and in-memory sources, read in order as one
/// compilation unit, with the directories `` `include `` searches and the
/// macros defined before the first.
#[derive(Clone, Debug, Default)]
pub struct Sources {
    pub items: Vec<Source>,
    pub include_dirs: Vec<PathBuf>,
    pub defines: Vec<(String, String)>,
}

/// One source of a compilation unit.
#[derive(Clone, Debug)]
pub enum Source {
    File(PathBuf),
    /// Text, and the name errors give it (an include in it is searched for
    /// in the name's directory first, as for a file).
    Text { name: String, text: String },
}

impl Sources {
    pub fn file(&mut self, path: impl Into<PathBuf>) -> &mut Self {
        self.items.push(Source::File(path.into()));
        self
    }

    pub fn text(&mut self, name: impl Into<String>, text: impl Into<String>) -> &mut Self {
        self.items.push(Source::Text { name: name.into(), text: text.into() });
        self
    }

    pub fn include_dir(&mut self, dir: impl Into<PathBuf>) -> &mut Self {
        self.include_dirs.push(dir.into());
        self
    }

    pub fn define(&mut self, name: impl Into<String>, text: impl Into<String>) -> &mut Self {
        self.defines.push((name.into(), text.into()));
        self
    }

    /// Lines in all of them, files read from disk.
    pub fn lines(&self) -> usize {
        self.items
            .iter()
            .map(|s| match s {
                Source::File(p) => std::fs::read_to_string(p).map_or(0, |t| t.lines().count()),
                Source::Text { text, .. } => text.lines().count(),
            })
            .sum()
    }
}

/// Preprocesses, parses and elaborates `src` with `tops` as the top-level
/// instances (none: every module nothing instantiates), and gives `f` the
/// design. Errors come back rendered, with their positions. It runs on a
/// thread of its own with room for the elaborator's recursion, which goes
/// as deep as the design's hierarchy.
pub fn elaborate<R: Send>(src: &Sources, tops: &[&str], f: impl FnOnce(&Design) -> R + Send) -> Result<R, String> {
    elaborate_with_ast(src, tops, |d, _| Ok(f(d)))
}

/// [`elaborate`], with the syntax tree too, and errors from `f` rendered
/// with their positions as the elaborator's are.
pub fn elaborate_with_ast<R: Send>(
    src: &Sources,
    tops: &[&str],
    f: impl for<'a> FnOnce(&Design<'a>, &crate::ast::Ast<'a>) -> Result<R, crate::source::Error<'a>> + Send,
) -> Result<R, String> {
    let tops: Vec<String> = tops.iter().map(|t| t.to_string()).collect();
    std::thread::scope(|s| {
        std::thread::Builder::new()
            .stack_size(1 << 30)
            .spawn_scoped(s, move || {
                let map = SourceMap::new();
                let mut ids = Vec::new();
                for item in &src.items {
                    let text = match item {
                        Source::File(p) => {
                            let t = std::fs::read_to_string(p).map_err(|e| format!("{}: {e}", p.display()))?;
                            Text::new(p.display().to_string(), t)
                        }
                        Source::Text { name, text } => Text::new(name.clone(), text.clone()),
                    };
                    map.add(text);
                    ids.push(map.len() - 1);
                }
                let opts = PpOptions { include_dirs: src.include_dirs.clone(), defines: src.defines.clone(), ..Default::default() };
                let out = preprocess(&map, &ids, &opts).map_err(|e| map.render(&e))?;
                let ast = crate::parse::parse(&out).map_err(|e| map.render(&e))?;
                let d = crate::design::elab::elaborate_tops(&ast, &out, &tops).map_err(|e| map.render(&e))?;
                f(&d, &ast).map_err(|e| map.render(&e))
            })
            .expect("a thread to elaborate on")
            .join()
            .unwrap_or_else(|p| std::panic::resume_unwind(p))
    })
}

/// Elaborates `src` with `top` as the top-level module and lowers its
/// netlist.
pub fn netlist(src: &Sources, top: &str, opts: &netlist::Options) -> Result<Netlist, String> {
    elaborate(src, &[top], |d| netlist::compile_with(d, d.tops[0], opts))?
}

/// What the netlist makes of one specialisation of a design.
#[derive(Clone, Debug)]
pub struct Lowering {
    /// The module's name.
    pub module: String,
    /// Its instances in the design.
    pub instances: u32,
    /// Lowered, with everything below it, as a top of its own; or why not.
    pub result: Result<(), String>,
}

/// Each specialisation of `d`, tried as the top of a netlist of its own
/// (one simulation): which of a design the netlist can
/// compile yet, and why not the rest. In the order the elaborator made
/// them.
pub fn coverage(d: &Design) -> Vec<Lowering> {
    let opts = netlist::Options::default();
    let mut out = Vec::new();
    for (k, spec) in d.specs.iter().enumerate() {
        let Some(inst) = d.instances.iter().position(|i| i.spec.0 as usize == k) else { continue };
        let result = netlist::compile_with(d, crate::design::InstId(inst as u32), &opts).map(|_| ());
        out.push(Lowering { module: d.name(spec.name).to_string(), instances: spec.instances, result });
    }
    out
}

/// A machine to compile for: this one, as the tests and examples run on
/// it (x86-64 Linux, AArch64 macOS).
pub struct Host {
    pub target: Target,
    pub triple: &'static str,
    /// Features turned on, as `+avx512f` (`epitome_codegen::Options::mattr`).
    pub mattr: Vec<String>,
}

/// The AVX-512 features the netlist's code is selected for on x86-64
/// (those of x86-64-v4).
pub const AVX512: &str = "+avx512f,+avx512bw,+avx512cd,+avx512dq,+avx512vl";

impl Host {
    /// This machine with the features its processor has: on x86-64,
    /// AVX-512 ([`AVX512`]) where it has it, else AVX2.
    pub fn native() -> Option<Host> {
        #[cfg(target_arch = "x86_64")]
        {
            let avx512 = std::is_x86_feature_detected!("avx512f")
                && std::is_x86_feature_detected!("avx512bw")
                && std::is_x86_feature_detected!("avx512cd")
                && std::is_x86_feature_detected!("avx512dq")
                && std::is_x86_feature_detected!("avx512vl");
            let mut mattr = if avx512 { AVX512.to_string() } else { "+avx2".to_string() };
            if avx512 && std::is_x86_feature_detected!("avx512vpopcntdq") {
                mattr += ",+avx512vpopcntdq";
            }
            Host::with_mattr(&mattr)
        }
        #[cfg(not(target_arch = "x86_64"))]
        Host::with_mattr("")
    }

    /// This machine with the features `mattr` names, comma-separated (as
    /// `+avx2`); none on AArch64.
    pub fn with_mattr(mattr: &str) -> Option<Host> {
        let mattr: Vec<String> = mattr.split(',').filter(|m| !m.is_empty()).map(String::from).collect();
        if cfg!(all(target_arch = "x86_64", target_os = "linux")) {
            Some(Host { target: Target::x86_64(), triple: "x86_64-unknown-linux-gnu", mattr })
        } else if cfg!(all(target_arch = "aarch64", target_os = "macos")) {
            Some(Host { target: Target::aarch64(), triple: "arm64-apple-macosx14.0.0", mattr })
        } else {
            None
        }
    }

    /// Whether the code has mask registers for bit planes: AArch64's
    /// vectors, and x86-64 with AVX-512.
    pub fn has_masks(&self) -> bool {
        self.triple.starts_with("arm64") || self.mattr.iter().any(|m| m == "+avx512f")
    }

    /// The netlist's options for this machine and `sims` simulations at
    /// once.
    pub fn options(&self, sims: usize) -> netlist::Options {
        netlist::Options { bit_planes: self.has_masks(), sims, ..Default::default() }
    }

    /// The features, as `--mattr` would give them; `native` if none.
    pub fn describe(&self) -> String {
        let arch = if self.triple.starts_with("arm64") { "AArch64" } else { "x86-64" };
        if self.mattr.is_empty() { arch.to_string() } else { format!("{arch} {}", self.mattr.join(",")) }
    }
}

/// How long [`Model::build`]'s steps took.
#[derive(Clone, Copy, Debug, Default)]
pub struct BuildTimes {
    pub optimise: Duration,
    pub compile: Duration,
    pub load: Duration,
}

/// A netlist compiled and loaded into this process.
pub struct Model {
    pub net: Netlist,
    /// The top's entry ([`Netlist::entry`]): a step, for a clocked top.
    pub eval: unsafe extern "C" fn(*mut u8),
    /// Each function's name and bytes of machine code.
    pub functions: Vec<(String, u64)>,
    /// With `netlist::Options::profile`, its counters (else null).
    pub prof: *mut i64,
    pub times: BuildTimes,
    /// The code `eval` is in.
    _code: Option<Loaded>,
    /// Run beside `eval` on a copy of the state, to check it.
    cross: Option<unsafe extern "C" fn(*mut u8)>,
}

/// The cycle counter the netlist's profile calls on x86-64 (where it is a
/// function, not inline assembly).
#[cfg(target_arch = "x86_64")]
extern "C" fn sv_ticks() -> i64 {
    unsafe { std::arch::x86_64::_rdtsc() as i64 }
}

/// What the netlist's code calls that it does not define.
pub fn resolve(name: &str) -> Option<*const u8> {
    #[cfg(target_arch = "x86_64")]
    if name == "sv_ticks" {
        return Some(sv_ticks as *const u8);
    }
    load::process_symbol(name)
}

/// The netlist's IR, for `host`, optimised (unless `optimise` is false)
/// and compiled; with how long each took.
pub fn compile(net: &Netlist, host: &Host, optimise: bool) -> Result<(Compiled, BuildTimes), String> {
    let started = Instant::now();
    let mut m = net.module.clone();
    m.triple = Some(host.triple.as_bytes().to_vec());
    if optimise {
        epitome_opt::pipeline::run(&mut m, &Default::default());
    }
    let optimised = started.elapsed();
    let opts = epitome_codegen::Options { mattr: host.mattr.clone(), no_asm: true, ..Default::default() };
    let c = epitome_codegen::compile(&host.target, &mut m, &opts)?;
    Ok((c, BuildTimes { optimise: optimised, compile: started.elapsed() - optimised, load: Duration::ZERO }))
}

/// Each of the netlist's functions in `c`, with its bytes of machine code.
pub fn functions(c: &Compiled, host: &Host) -> Result<Vec<(String, u64)>, String> {
    let object = c.object(&host.target)?;
    let mut out: Vec<(String, u64)> = object
        .symbols
        .iter()
        .filter(|s| s.name.contains("sv_eval_") && s.section.is_some_and(|i| object.sections[i].kind == SectionKind::Text))
        .map(|s| (s.name.trim_start_matches('_').to_string(), s.size))
        .collect();
    out.sort();
    Ok(out)
}

impl Model {
    /// `net` optimised, compiled for `host` and loaded.
    pub fn build(net: Netlist, host: &Host) -> Result<Model, String> {
        let (c, times) = compile(&net, host, true)?;
        Model::load(net, host, &c, times)
    }

    /// `c`, compiled from `net`, loaded.
    pub fn load(net: Netlist, host: &Host, c: &Compiled, mut times: BuildTimes) -> Result<Model, String> {
        let started = Instant::now();
        let t = &host.target;
        let functions = functions(c, host)?;
        let code = load::load(t, c, resolve).map_err(|e| format!("loading: {e}"))?;
        let eval = unsafe { code.function(&net.entry) }.ok_or_else(|| format!("no {}", net.entry))?;
        let prof = code.address("sv_prof").map_or(std::ptr::null_mut(), |p| p as *mut i64);
        times.load = started.elapsed();
        Ok(Model { net, eval, functions, prof, times, _code: Some(code), cross: None })
    }

    /// A model of code loaded some other way: `eval` is `net`'s entry,
    /// `prof` its profile's counters or null.
    ///
    /// # Safety
    /// `eval` runs `net`'s code, and stays loaded while the model lives.
    pub unsafe fn from_parts(net: Netlist, eval: unsafe extern "C" fn(*mut u8), functions: Vec<(String, u64)>, prof: *mut i64) -> Model {
        Model { net, eval, functions, prof, times: BuildTimes::default(), _code: None, cross: None }
    }

    /// From now on each [`Model::eval_state`] also runs `other` on a copy
    /// of the state, and panics where the two leave different states: for
    /// checking a second build of the same IR (the tests' `NETLIST_CC`).
    ///
    /// # Safety
    /// As for [`Model::from_parts`].
    pub unsafe fn cross_check(&mut self, other: unsafe extern "C" fn(*mut u8)) {
        self.cross = Some(other);
    }

    /// Bytes of machine code in all its functions.
    pub fn code_bytes(&self) -> u64 {
        self.functions.iter().map(|f| f.1).sum()
    }

    /// Bytes of state for each simulation.
    pub fn state_per_instance(&self) -> f64 {
        self.net.size as f64 / self.net.lanes as f64
    }

    /// A zeroed state.
    pub fn state(&self) -> State {
        State::new(self.net.size)
    }

    /// Evaluates `st` once: settles it, or for a clocked top, a step.
    pub fn eval_state(&self, st: &mut State) {
        let Some(other) = self.cross else {
            unsafe { (self.eval)(st.ptr) };
            return;
        };
        let theirs = State::new(self.net.size);
        unsafe {
            std::ptr::copy_nonoverlapping(st.ptr, theirs.ptr, self.net.size);
            (self.eval)(st.ptr);
            other(theirs.ptr);
        }
        if let Some(k) = (0..self.net.size).find(|&k| st.bytes()[k] != theirs.bytes()[k]) {
            panic!("the loaded code and the other build of it differ, first at byte {k} of the state");
        }
    }

    /// One cycle of lanes `0..ins.len() / io.in_words()` of `st`: each
    /// lane's inputs set from its words of `ins`, the state evaluated
    /// (for a clocked top, with the clock low and then high), and each
    /// lane's outputs appended to `out`.
    pub fn cycle(&self, io: &Io, st: &mut State, ins: &[u32], out: Option<&mut Vec<u32>>) {
        let iw = io.in_words();
        let lanes = if iw == 0 { self.net.lanes } else { ins.len() / iw };
        assert!(lanes <= self.net.lanes, "{lanes} lanes of inputs for a state of {}", self.net.lanes);
        for k in 0..lanes {
            st.put_all(&io.ins, k, &ins[k * iw..(k + 1) * iw]);
        }
        match &io.clock {
            Some(clk) => {
                for level in [0, 1] {
                    for k in 0..lanes {
                        unsafe { clk.put(st.ptr, k, level) };
                    }
                    self.eval_state(st);
                }
            }
            None => self.eval_state(st),
        }
        if let Some(out) = out {
            for k in 0..lanes {
                for f in &io.outs {
                    st.get_words(f, k, out);
                }
            }
        }
    }

    /// Runs `n` instances for `cycles` cycles on `stim` (for each cycle,
    /// for each instance, its inputs' words, as [`Io`] orders them), a
    /// state's lanes at a time from zeroed states, and gives their outputs
    /// in the same order.
    pub fn run(&self, io: &Io, cycles: usize, n: usize, stim: &[u32]) -> Vec<u32> {
        let (iw, ow, group) = (io.in_words(), io.out_words(), self.net.lanes);
        assert_eq!(stim.len(), cycles * n * iw, "stimulus for {cycles} cycles of {n} instances");
        let mut out = vec![0; cycles * n * ow];
        let mut ins = Vec::with_capacity(group * iw);
        let mut got = Vec::with_capacity(group * ow);
        for g in 0..n.div_ceil(group) {
            // The last state's lanes past the instances run on zeros.
            let (first, here) = (g * group, group.min(n - g * group));
            let mut st = self.state();
            for c in 0..cycles {
                ins.clear();
                ins.extend_from_slice(&stim[(c * n + first) * iw..(c * n + first + here) * iw]);
                got.clear();
                self.cycle(io, &mut st, &ins, Some(&mut got));
                out[(c * n + first) * ow..(c * n + first + here) * ow].copy_from_slice(&got);
            }
        }
        out
    }

    /// Nanoseconds an instance a cycle, on the first instances of `stim`
    /// (as for [`Model::run`]): a state's worth, or for states of few
    /// lanes as many states as make [`netlist::GROUP`] lanes, replayed
    /// `reps` times from zeroed states, timing only the evaluations; and
    /// with a profile, its counters.
    pub fn time(&self, io: &Io, cycles: usize, n: usize, stim: &[u32], reps: usize) -> Timed {
        let net = &self.net;
        let (iw, group) = (io.in_words(), net.lanes);
        // States of few lanes each are timed a pass over many, as one call
        // takes too little time to time on its own.
        let states = (n / group).clamp(1, (netlist::GROUP / group).max(1));
        let lanes = group.min(n);
        let mut spent = Duration::ZERO;
        let counters: &mut [i64] =
            if self.prof.is_null() { &mut [] } else { unsafe { std::slice::from_raw_parts_mut(self.prof, 2 * net.profile.len()) } };
        counters.fill(0);
        for _ in 0..reps {
            let mut sts: Vec<State> = (0..states).map(|_| self.state()).collect();
            for c in 0..cycles {
                for (g, st) in sts.iter_mut().enumerate() {
                    for k in 0..lanes {
                        let at = (c * n + g * group + k) * iw;
                        st.put_all(&io.ins, k, &stim[at..at + iw]);
                    }
                }
                let levels: &[u64] = if io.clock.is_some() { &[0, 1] } else { &[0] };
                for &level in levels {
                    if let Some(clk) = &io.clock {
                        for st in &sts {
                            for k in 0..group {
                                unsafe { clk.put(st.ptr, k, level) };
                            }
                        }
                    }
                    let t0 = Instant::now();
                    for st in &sts {
                        unsafe { (self.eval)(std::hint::black_box(st.ptr)) };
                    }
                    spent += t0.elapsed();
                }
            }
        }
        // The counters, per instance a cycle, largest first, with the
        // calls a state a cycle.
        let per = (reps * cycles * states) as f64;
        let mut profile: Vec<Profiled> = counters
            .chunks(2)
            .zip(&net.profile)
            .map(|(c, what)| Profiled { ticks: c[0] as f64 / per / group as f64, calls: c[1] as f64 / per, what: what.clone() })
            .collect();
        profile.sort_by(|a, b| b.ticks.total_cmp(&a.ticks));
        Timed { ns: spent.as_secs_f64() * 1e9 / (reps * cycles * states * group) as f64, profile }
    }
}

/// [`Model::time`]'s result.
#[derive(Clone, Debug)]
pub struct Timed {
    /// Nanoseconds an instance a cycle.
    pub ns: f64,
    pub profile: Vec<Profiled>,
}

/// A counter of the profile: cycle counter ticks an instance a cycle, and
/// calls a state a cycle, in what it names.
#[derive(Clone, Debug)]
pub struct Profiled {
    pub ticks: f64,
    pub calls: f64,
    pub what: String,
}

/// The ports a run sets and reads: the inputs (the clock aside) and the
/// outputs, in port order, the order of their words in stimulus and
/// responses.
#[derive(Clone, Debug)]
pub struct Io {
    pub ins: Vec<Field>,
    pub outs: Vec<Field>,
    pub clock: Option<Field>,
}

impl Io {
    /// `net`'s ports, with `clock` raised each cycle if given.
    pub fn new(net: &Netlist, clock: Option<&str>) -> Result<Io, String> {
        let clock = clock.map(|c| net.port(c).cloned().ok_or_else(|| format!("no port {c}"))).transpose()?;
        let is_clock = |p: &Field| clock.as_ref().is_some_and(|c| c.name == p.name);
        Ok(Io {
            ins: net.ports.iter().filter(|p| p.dir == Dir::Input && !is_clock(p)).cloned().collect(),
            outs: net.ports.iter().filter(|p| p.dir == Dir::Output).cloned().collect(),
            clock,
        })
    }

    /// 32-bit words of an instance's inputs.
    pub fn in_words(&self) -> usize {
        self.ins.iter().map(|p| words(p.width)).sum()
    }

    pub fn out_words(&self) -> usize {
        self.outs.iter().map(|p| words(p.width)).sum()
    }

    /// Where input `name`'s words are in an instance's, and its width.
    pub fn input(&self, name: &str) -> Option<(usize, u32)> {
        let mut at = 0;
        for p in &self.ins {
            if p.name == name {
                return Some((at, p.width));
            }
            at += words(p.width);
        }
        None
    }
}

/// 32-bit words in a `w`-bit value.
pub fn words(w: u32) -> usize {
    (w as usize).div_ceil(32).max(1)
}

/// A netlist's state, aligned for 512-bit loads.
pub struct State {
    pub ptr: *mut u8,
    layout: std::alloc::Layout,
}

impl State {
    pub fn new(size: usize) -> State {
        let layout = std::alloc::Layout::from_size_align(size.max(64), 64).unwrap();
        State { ptr: unsafe { std::alloc::alloc_zeroed(layout) }, layout }
    }

    fn bytes(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts(self.ptr, self.layout.size()) }
    }

    /// Lane `n`'s value of port `port` (its low 64 bits).
    pub fn put(&mut self, net: &Netlist, port: &str, n: usize, v: u64) {
        let f = net.port(port).unwrap_or_else(|| panic!("no port {port}"));
        unsafe { f.put(self.ptr, n, v) };
    }

    /// A value wider than 64 bits, as 64-bit words, low first.
    pub fn put_wide(&mut self, net: &Netlist, port: &str, n: usize, v: &[u64]) {
        let f = net.port(port).unwrap_or_else(|| panic!("no port {port}"));
        let words: Vec<u32> = (0..f.pieces()).map(|k| (v.get(k / 2).copied().unwrap_or(0) >> (32 * (k % 2))) as u32).collect();
        self.put_words(f, n, &words);
    }

    pub fn get(&self, net: &Netlist, port: &str, n: usize) -> u64 {
        let f = net.port(port).unwrap_or_else(|| panic!("no port {port}"));
        unsafe { f.get(self.ptr, n) }
    }

    /// Lane `n`'s value of `f`, as 32-bit words, low first.
    pub fn put_words(&mut self, f: &Field, n: usize, v: &[u32]) {
        unsafe { f.put_words(self.ptr, n, v) }
    }

    /// Appends lane `n`'s value of `f`, as 32-bit words, low first.
    pub fn get_words(&self, f: &Field, n: usize, out: &mut Vec<u32>) {
        unsafe { f.get_words(self.ptr, n, out) }
    }

    /// Lane `n`'s values of `fields`, from their words one after another.
    pub fn put_all(&mut self, fields: &[Field], n: usize, v: &[u32]) {
        let mut at = 0;
        for f in fields {
            self.put_words(f, n, &v[at..at + words(f.width)]);
            at += words(f.width);
        }
    }
}

impl Drop for State {
    fn drop(&mut self) {
        unsafe { std::alloc::dealloc(self.ptr, self.layout) };
    }
}

/// An instance-cycle where two runs' outputs differ.
#[derive(Clone, Debug)]
pub struct Mismatch {
    pub cycle: usize,
    pub instance: usize,
    pub port: String,
    pub ours: Vec<u32>,
    pub theirs: Vec<u32>,
}

/// Where `ours` and `theirs`, the outputs of [`Model::run`] (or of
/// Verilator's model on the same stimulus) for `n` instances, differ:
/// the instance-cycles that do, and the first `keep` ports that do.
pub fn compare(io: &Io, n: usize, ours: &[u32], theirs: &[u32], keep: usize) -> (usize, Vec<Mismatch>) {
    assert_eq!(ours.len(), theirs.len(), "outputs of different runs");
    let ow = io.out_words();
    let mut bad = 0;
    let mut first = Vec::new();
    for (x, (a, b)) in ours.chunks(ow.max(1)).zip(theirs.chunks(ow.max(1))).enumerate() {
        if a == b {
            continue;
        }
        bad += 1;
        let mut at = 0;
        for f in &io.outs {
            let r = at..at + words(f.width);
            if a[r.clone()] != b[r.clone()] && first.len() < keep {
                first.push(Mismatch { cycle: x / n, instance: x % n, port: f.name.clone(), ours: a[r.clone()].to_vec(), theirs: b[r.clone()].to_vec() });
            }
            at = r.end;
        }
    }
    (bad, first)
}

/// Where a cache of things built from `src` should be considered stale: a
/// line for each file with its length and time, and each text's length
/// and a hash of it.
pub(crate) fn fingerprint(src: &Sources) -> String {
    use std::hash::{Hash, Hasher};
    let mut out = String::new();
    for item in &src.items {
        match item {
            Source::File(p) => {
                let m = std::fs::metadata(p);
                let (len, time) = m.map_or((0, None), |m| (m.len(), m.modified().ok()));
                out += &format!("{} {len} {time:?}\n", p.display());
            }
            Source::Text { name, text } => {
                let mut h = crate::hash::FxHasher::default();
                text.hash(&mut h);
                out += &format!("{name} {} {:x}\n", text.len(), h.finish());
            }
        }
    }
    for d in &src.include_dirs {
        out += &format!("-I{}\n", d.display());
    }
    for (n, t) in &src.defines {
        out += &format!("-D{n}={t}\n");
    }
    out
}

/// How a function runs its lanes, in words: `scalar`; SIMD, the lanes of
/// a register and their bits (`16 x 32-bit`); or SWAR, lanes side by side
/// in an `i64` (`SWAR 8 x 8-bit`); with the registers if more than one.
pub fn describe_shape(sh: netlist::Shape) -> String {
    if sh.scalar() {
        return "scalar".into();
    }
    let regs = if sh.chunks > 1 { format!(", {} registers", sh.chunks) } else { String::new() };
    let swar = if sh.swar { "SWAR " } else { "" };
    format!("{swar}{} x {}-bit{regs}", sh.n, sh.pb())
}
