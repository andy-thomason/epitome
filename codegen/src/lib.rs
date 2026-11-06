//! Code generation for Epitome: instruction selection from machine
//! descriptions written in LLVM IR, register allocation, frames, and
//! assembly, binary and disassembly output. See `docs/design/isel.md`.

pub mod abi;
pub mod aggregates;
pub mod asm;
pub mod asm_text;
pub mod assembler;
pub mod atomics;
pub mod bitmaps;
pub mod check;
pub mod divconst;
pub mod eh;
pub mod elf;
pub mod enc;
pub mod features;
pub mod frame;
pub mod huge;
pub mod intrinsics;
pub mod isel;
pub mod lanewise;
pub mod layout;
pub mod legalize;
pub mod machine;
pub mod macho;
pub mod mir;
pub mod neon;
pub mod objdump;
pub mod object;
pub mod peephole;
pub mod prepare;
pub mod regalloc;
pub mod runtime;
pub mod target;
pub mod unwind;
pub mod wide;
pub mod wide_vectors;
pub mod x87;

use epitome_ir::Module;
use epitome_ir::layout::DataLayout;
use epitome_ir::module::*;

use crate::mir::{MFunc, Opcode};
use crate::prepare::Ctx;
use crate::target::Target;

/// The default data layout for arm64 Darwin, when a module has none.
pub const DARWIN_ARM64_LAYOUT: &str = "e-m:o-i64:64-i128:128-n32:64-S128-Fn32";

pub struct Compiled {
    pub funcs: Vec<MFunc>,
    /// The globals with initial values.
    pub globals: Vec<object::GlobalData>,
    pub aliases: Vec<object::Alias>,
    pub abi: abi::Abi,
    /// For Mach-O: the least macOS version the code is for, from the
    /// module's triple (`arm64-apple-macosx26.0.0`), else 11.0.
    pub min_os: (u32, u32, u32),
    pub asm: String,
    /// Whether the module had assembly of its own, which `asm` ends with.
    pub module_asm: bool,
    /// With `keep_going`: the functions that could not be compiled, and why.
    pub errors: Vec<String>,
    /// Autochop: the functions counted (in module order), each one's count
    /// of elective choices, and where a limit stopped.
    pub chop_names: Vec<String>,
    pub chop_counts: Vec<epitome_ir::chop::Counter>,
    pub chop_stopped: Option<(usize, u64, Option<String>)>,
}

/// Which stages to stop after, for looking at the machine IR.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Stop {
    #[default]
    Done,
    AfterSelection,
    AfterAllocation,
}

#[derive(Clone, Debug, Default)]
pub struct Options {
    pub stop: Stop,
    /// The calling convention and platform, by name; by default, the one
    /// the module's target triple names, or else the description's first.
    pub abi: Option<String>,
    /// Further registers to keep out of allocation.
    pub reserve: Vec<u8>,
    /// The assembler dialect, if not the convention's own.
    pub syntax: Option<abi::Syntax>,
    /// Report each function that cannot be compiled, and carry on with the
    /// rest, instead of stopping at the first.
    pub keep_going: bool,
    /// The CPU, as the description names it; by default the one each
    /// function's IR names, or the description's.
    pub mcpu: Option<String>,
    /// Features turned on or off: `+neon`, `-lse`, or lists of them.
    pub mattr: Vec<String>,
    /// Threads to compile functions on; 0 means one per core.
    pub threads: usize,
    /// Autochop: how far elective choices may go (`epitome_ir::chop`).
    pub chop: Option<epitome_ir::chop::Limit>,
}

/// Time spent in each stage of compiling functions, summed over every
/// function and thread, in nanoseconds (for `--stats`).
pub mod timing {
    use std::sync::atomic::{AtomicU64, Ordering};

    pub static PREPARE: AtomicU64 = AtomicU64::new(0);
    pub static SELECT: AtomicU64 = AtomicU64::new(0);
    pub static ALLOCATE: AtomicU64 = AtomicU64::new(0);
    pub static FRAME: AtomicU64 = AtomicU64::new(0);
    pub static LAYOUT: AtomicU64 = AtomicU64::new(0);

    pub fn add(c: &AtomicU64, since: std::time::Instant) {
        c.fetch_add(since.elapsed().as_nanos() as u64, Ordering::Relaxed);
    }

    pub fn get(c: &AtomicU64) -> u64 {
        c.load(Ordering::Relaxed)
    }
}

/// `work(i)` for every `i` in `order`, on up to `threads` threads, each
/// thread starting from its own `init()`; the results come back by index
/// (`0..n`), whatever order they ran in. `done` runs on each thread when it
/// has no more work.
pub(crate) fn par_run<S, R: Send>(
    n: usize,
    order: &[usize],
    threads: usize,
    init: impl Fn() -> S + Sync,
    work: impl Fn(&mut S, usize) -> R + Sync,
    done: impl Fn() + Sync,
) -> Vec<R> {
    let results: Vec<std::sync::Mutex<Option<R>>> = (0..n).map(|_| std::sync::Mutex::new(None)).collect();
    let next = std::sync::atomic::AtomicUsize::new(0);
    let worker = || {
        let mut state = init();
        while let Some(&i) = order.get(next.fetch_add(1, std::sync::atomic::Ordering::Relaxed)) {
            let r = work(&mut state, i);
            *results[i].lock().unwrap() = Some(r);
        }
        done();
    };
    if threads <= 1 {
        worker();
    } else {
        std::thread::scope(|s| {
            for _ in 0..threads {
                s.spawn(worker);
            }
        });
    }
    results.into_iter().map(|r| r.into_inner().unwrap().expect("a task did not run")).collect()
}

/// A function to compile: its body, taken out of the module.
struct Job {
    name: String,
    global: bool,
    body: Function,
}

/// The convention `opts` asks for, for module `m`.
pub fn choose_abi(t: &Target, m: &Module, opts: &Options) -> Result<abi::Abi, String> {
    let triple = m.triple.as_ref().map(|s| String::from_utf8_lossy(s).into_owned()).unwrap_or_default();
    let mut abi = match &opts.abi {
        Some(name) => t.abi(name).ok_or_else(|| format!("no calling convention {name}"))?,
        None => t.abi_for_triple(&triple).or_else(|| t.abis.first()).ok_or("no calling convention")?,
    }
    .clone();
    abi.reserved.extend(&opts.reserve);
    if let Some(s) = opts.syntax {
        abi.syntax = s;
    }
    Ok(abi)
}

/// Compiles function `f` of module `m` (as `cx` has it).
fn compile_function(t: &Target, abi: &abi::Abi, cx: &mut Ctx, job: &Job, opts: &Options) -> Result<MFunc, String> {
    use std::time::Instant;
    cx.features = features::of_function(t, cx.m, &job.body.fn_attrs, opts.mcpu.as_deref(), &opts.mattr)?;
    let now = Instant::now();
    let prepared = prepare::prepare(cx, &job.body, &job.name)?;
    timing::add(&timing::PREPARE, now);
    let now = Instant::now();
    let mut mf = isel::Sel::new(t, abi, cx, &prepared, &job.name, job.global).run()?;
    timing::add(&timing::SELECT, now);
    // What the description says instructions clobber.
    for b in &mut mf.blocks {
        for inst in &mut b.insts {
            if let Opcode::Instr(i) = inst.op
                && !t.instrs[i].clobbers.is_empty()
            {
                inst.imp_defs = t.instrs[i].clobbers.clone();
            }
        }
    }
    peephole::redundant_compares(t, &mut mf);
    // Until nothing changes: an address made from one by adding to it
    // becomes one itself.
    for _ in 0..4 {
        if peephole::frame_addresses(t, &mut mf) == 0 {
            break;
        }
    }
    if opts.stop != Stop::AfterSelection {
        let now = Instant::now();
        let selected = (opts.stop != Stop::AfterAllocation).then(|| mf.clone());
        regalloc::allocate(t, &mut mf)?;
        timing::add(&timing::ALLOCATE, now);
        if opts.stop != Stop::AfterAllocation {
            let now = Instant::now();
            match frame::lower(t, &mut mf) {
                // The frame needed a scratch register the allocator had
                // given a value: allocate again, keeping them free.
                Err(e) if e.ends_with(frame::NEEDS_SCRATCH) => {
                    mf = selected.expect("kept for this");
                    regalloc::allocate_keeping_scratch(t, &mut mf)?;
                    frame::lower(t, &mut mf)?;
                }
                r => r?,
            }
            peephole::pair_memory(t, &mut mf);
            peephole::three_address(t, &mut mf);
            peephole::short_forms(t, &mut mf);
            timing::add(&timing::FRAME, now);
            let now = Instant::now();
            layout::place(t, &mut mf);
            layout::relax(t, &mut mf);
            mir::set_sizes(t, &mut mf);
            timing::add(&timing::LAYOUT, now);
        }
    }
    Ok(mf)
}

/// Compiles every function of `m` with a body, on up to `opts.threads`
/// threads, then the globals.
///
/// Functions are compiled independently. Each thread works on its own copy
/// of the module without the function bodies (its types, constants and
/// declarations, which compiling a function may add to), taking functions
/// largest first; the results are put in module order. A function's machine
/// IR names what it refers to by symbol, not by the module's ids, so the
/// output is the same for any number of threads.
pub fn compile(t: &Target, m: &mut Module, opts: &Options) -> Result<Compiled, String> {
    let abi = choose_abi(t, m, opts)?;
    let spec = m.data_layout.as_ref().map(|d| String::from_utf8_lossy(d).into_owned());
    let dl = DataLayout::parse(spec.as_deref().unwrap_or(DARWIN_ARM64_LAYOUT))?;
    // The bodies come out of the module (its parameters stay, which other
    // functions' calls may look at), and go back at the end.
    let mut jobs: Vec<(GlobalId, Job)> = Vec::new();
    for &g in &m.global_order {
        let gl = &mut m.globals[g.index()];
        let name = gl.name.clone().unwrap_or_else(|| format!("f{}", g.0));
        let global = !matches!(gl.props.linkage, Linkage::Private | Linkage::Internal);
        let GlobalKind::Function(f) = &mut gl.kind else { continue };
        if !f.has_body {
            continue;
        }
        let mut body = Function { locals: f.locals[..f.params.len()].to_vec(), ..Default::default() };
        std::mem::swap(&mut body.locals, &mut f.locals);
        body.blocks = std::mem::take(&mut f.blocks);
        body.layout = std::mem::take(&mut f.layout);
        body.insts = std::mem::take(&mut f.insts);
        body.ty = f.ty;
        body.cconv = f.cconv.clone();
        body.ret_attrs = f.ret_attrs.clone();
        body.params = f.params.clone();
        body.fn_attrs = f.fn_attrs.clone();
        body.personality = f.personality;
        body.has_body = true;
        jobs.push((g, Job { name, global, body }));
    }
    let threads =
        if opts.threads == 0 { std::thread::available_parallelism().map_or(1, |n| n.get()) } else { opts.threads };
    let threads = threads.min(jobs.len()).max(1);
    // Largest first, so that a big function does not start last.
    let mut by_size: Vec<usize> = (0..jobs.len()).collect();
    by_size.sort_by_key(|&k| std::cmp::Reverse(jobs[k].1.body.insts.len()));
    let skeleton: &Module = m;
    // Autochop: function `j` (in module order) counts its own choices.
    let chop = epitome_ir::chop::Session::new(jobs.len(), opts.chop);
    let results = par_run(
        jobs.len(),
        &by_size,
        threads,
        || skeleton.clone(),
        |m, j| {
            let mut cx = Ctx { m, dl: &dl, t, features: Default::default(), hfa: abi.hfa };
            let _counted = chop.enter(j);
            compile_function(t, &abi, &mut cx, &jobs[j].1, opts)
        },
        isel::stats::flush,
    );
    let chop_names: Vec<String> = jobs.iter().map(|(_, j)| j.name.clone()).collect();
    // The bodies go back.
    for (g, job) in jobs {
        if let GlobalKind::Function(f) = &mut m.globals[g.index()].kind {
            f.locals = job.body.locals;
            f.blocks = job.body.blocks;
            f.layout = job.body.layout;
            f.insts = job.body.insts;
        }
    }
    let mut funcs = Vec::new();
    let mut errors = Vec::new();
    for r in results {
        let mut mf = match r {
            Ok(mf) => mf,
            Err(e) if opts.keep_going => {
                errors.push(e);
                continue;
            }
            Err(e) => return Err(e),
        };
        let fi = funcs.len();
        for (k, b) in mf.blocks.iter_mut().enumerate() {
            b.label = format!("{}BB{fi}_{k}", abi.local_prefix());
        }
        funcs.push(mf);
    }
    let cx = Ctx { m, dl: &dl, t, features: Default::default(), hfa: abi.hfa };
    let min_os = min_os_version(&String::from_utf8_lossy(cx.m.triple.as_deref().unwrap_or_default()));
    let mut out = String::new();
    if abi.syntax == abi::Syntax::MachO {
        let (a, b, c) = min_os;
        let update = if c == 0 { String::new() } else { format!(", {c}") };
        out.push_str(&format!("\t.build_version macos, {a}, {b}{update}\n"));
    }
    out.push_str(&format!("\t{}\n", abi.section_directive(object::SectionKind::Text)));
    // Each function's text, in parallel; then in order.
    let all: Vec<usize> = (0..funcs.len()).collect();
    let texts = par_run(
        funcs.len(),
        &all,
        threads,
        || (),
        |_, k| {
            let mut s = String::new();
            asm::function(t, &funcs[k], k, &mut s).map(|()| s)
        },
        || (),
    );
    for text in texts {
        out.push_str(&text?);
    }
    let mut globals = asm::collect_globals(&cx, &abi)?;
    globals.extend(asm::eh_stubs(&funcs, &abi));
    asm::print_globals(&abi, &globals, &mut out);
    let aliases = asm::collect_aliases(&cx, &abi)?;
    asm::print_aliases(&aliases, &mut out);
    // The module's own assembly, after ours (which the assembler then
    // assembles, rather than the object being written directly).
    let module_asm = !cx.m.module_asm.is_empty();
    if module_asm {
        out.push_str(&format!("\t{}\n", abi.section_directive(object::SectionKind::Text)));
        for line in &cx.m.module_asm {
            out.push_str(&String::from_utf8_lossy(line));
            out.push('\n');
        }
    }
    if abi.syntax == abi::Syntax::Elf {
        // The stack need not be executable.
        out.push_str("\t.section .note.GNU-stack,\"\",%progbits\n");
    }
    Ok(Compiled {
        module_asm,
        funcs,
        globals,
        aliases,
        abi,
        min_os,
        asm: out,
        errors,
        chop_names,
        chop_counts: chop.counts(),
        chop_stopped: chop.stopped_at(),
    })
}

/// The macOS version a triple names (`macosx26.0.0`, `macos14.2`), or 11.0,
/// the first for arm64.
pub fn min_os_version(triple: &str) -> (u32, u32, u32) {
    let rest = triple.split('-').find_map(|p| p.strip_prefix("macosx").or_else(|| p.strip_prefix("macos")));
    let mut parts = rest.unwrap_or("").split('.').map(|x| x.parse::<u32>().ok());
    match (parts.next().flatten(), parts.next().flatten(), parts.next().flatten()) {
        (Some(a), b, c) if a > 0 => (a, b.unwrap_or(0), c.unwrap_or(0)),
        _ => (11, 0, 0),
    }
}

impl Compiled {
    /// The object file's contents, in the abstract (see `object`).
    pub fn object(&self, t: &Target) -> Result<object::Object, String> {
        object::build(t, &self.abi, &self.funcs, &self.globals, &self.aliases)
    }

    /// The object file, in the convention's format: ELF or Mach-O. With
    /// inline assembly, whose text only an assembler reads, by assembling
    /// the assembly.
    pub fn object_file(&self, t: &Target) -> Result<Vec<u8>, String> {
        if self.module_asm || self.funcs.iter().any(|f| !f.asms.is_empty()) {
            return assembler::Assembler::new(t).assemble(&self.asm, Some(self.abi.syntax)).and_then(|a| a.file(t));
        }
        let obj = self.object(t)?;
        match self.abi.syntax {
            abi::Syntax::Elf => elf::write(&obj, &t.machine),
            abi::Syntax::MachO => macho::write(&obj, self.min_os, &t.machine),
        }
    }
}
