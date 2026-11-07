//! `epitome-llc`: compiles a `.ll` file to AArch64 assembly, or to an
//! object file (Mach-O or ELF, as the convention uses).

use std::process::ExitCode;

use epitome_codegen::target::Target;
use epitome_codegen::{Options, Stop, compile, mir};

const USAGE: &str = "usage: epitome-llc [options] <file.ll>

  -j <n>                  threads to compile functions on (default: one
                          per core); the output is the same for any number
  --chop=F[:T]            autochop: functions before F (in module order,
                          from 0) get every elective choice, F its first T,
                          the rest none; also EPITOME_LLC_CHOP
  --chop-counts           print each function's count of elective choices
  -o <file>               write the output here instead of stdout (- is
                          stdout; an input of - is stdin); a name ending in
                          .o asks for an object file; functions with the
                          attribute epitome-opaque (for another tool) go to
                          <file>.opaque.ll as IR, and need -o
  --filetype=asm|obj      assembly (the default) or an object file, Mach-O
                          or ELF as the convention uses
  --target=<name|path>    the machine description: a .ll file, a directory
                          of them (read in name order), or a name: aarch64
                          and x86_64 are built in, others are directories of that name
                          in EPITOME_TARGET_PATH (a list separated by
                          colons); by default, the one the module's triple names
                          (riscv64-... is riscv64), else aarch64
  --target-add=<path>     read these .ll files too, as more of the
                          description (may be given again)
  --abi=darwin|aapcs64    the calling convention and platform (default:
                          from the module's target triple, else darwin)
  --mcpu=<name>           the CPU, as the description names it (default:
                          the one each function's IR names, or the
                          description's)
  --mattr=+f,-g,...       turn features on or off
  --reserve=x18,...       keep these registers out of allocation
  --syntax=macho|elf      the assembler dialect, if not the convention's
  --stop-after=isel|ra    print machine IR after selection or allocation
  --print-after-prepare   print each function's IR, as selection gets it,
                          to standard error
  --hex                   print each function's machine code in hex
  --disasm                disassemble each function's machine code
  --patterns              list the selection patterns and exit
  --check-target          check the description (--target, --target-add)
                          and exit: encodings, assembly, recipes, and a
                          smoke program compiled under each convention
  --verbose               with --check-target, every message
  --sample-asm            print assembly with sample lines of every
                          instruction of the description, and exit
  --stats                 print matching counts and timings to stderr
  --keep-going            report every function that cannot be compiled,
                          and compile the rest";

/// A runaway compilation stops at a memory limit (`epitome_ir::memlimit`;
/// `EPITOME_MEMORY_LIMIT_MB`, default 8192) rather than taking the machine
/// down.
#[global_allocator]
static MEMORY: epitome_ir::memlimit::Limited = epitome_ir::memlimit::Limited;

fn main() -> ExitCode {
    epitome_ir::memlimit::set_from_env(8192);
    let args: Vec<String> = std::env::args().skip(1).collect();
    let (mut input, mut output, mut target_file) = (None, None, None);
    let mut adds: Vec<String> = Vec::new();
    let (mut hex, mut disasm, mut patterns, mut stats) = (false, false, false, false);
    let (mut check, mut verbose, mut sample) = (false, false, false);
    let mut reserve: Vec<String> = Vec::new();
    let mut filetype = None;
    let mut stop = Stop::Done;
    let mut print_prepared = false;
    let mut opts = Options::default();
    let (mut chop, mut chop_counts) = (None, false);
    let mut it = args.iter();
    while let Some(a) = it.next() {
        match a.as_str() {
            "-o" => output = it.next().cloned(),
            "-j" => match it.next().and_then(|n| n.parse().ok()) {
                Some(n) => opts.threads = n,
                None => {
                    eprintln!("epitome-llc: -j needs a number\n\n{USAGE}");
                    return ExitCode::FAILURE;
                }
            },
            "--hex" => hex = true,
            "--disasm" => disasm = true,
            "--patterns" => patterns = true,
            "--check-target" => check = true,
            "--sample-asm" => sample = true,
            "--verbose" => verbose = true,
            "--stats" => stats = true,
            "--chop-counts" => chop_counts = true,
            s if s.starts_with("--chop=") => chop = Some(s["--chop=".len()..].to_string()),
            "--keep-going" => opts.keep_going = true,
            "--filetype=asm" => filetype = Some(false),
            "--filetype=obj" => filetype = Some(true),
            "--stop-after=isel" => stop = Stop::AfterSelection,
            "--stop-after=ra" => stop = Stop::AfterAllocation,
            "--print-after-prepare" => print_prepared = true,
            s if s.starts_with("--target=") => target_file = Some(s["--target=".len()..].to_string()),
            s if s.starts_with("--target-add=") => adds.push(s["--target-add=".len()..].to_string()),
            s if s.starts_with("--abi=") => opts.abi = Some(s["--abi=".len()..].to_string()),
            s if s.starts_with("--mcpu=") => opts.mcpu = Some(s["--mcpu=".len()..].to_string()),
            s if s.starts_with("--mattr=") => opts.mattr.push(s["--mattr=".len()..].to_string()),
            "--syntax=macho" => opts.syntax = Some(epitome_codegen::abi::Syntax::MachO),
            "--syntax=elf" => opts.syntax = Some(epitome_codegen::abi::Syntax::Elf),
            s if s.starts_with("--reserve=") => reserve.extend(s["--reserve=".len()..].split(',').map(str::to_string)),
            s if s.starts_with('-') && s != "-" => {
                eprintln!("epitome-llc: unknown option {s}\n\n{USAGE}");
                return ExitCode::FAILURE;
            }
            s => input = Some(s.to_string()),
        }
    }
    let load = |triple: &str| match Target::resolve(target_file.as_deref(), triple, &adds) {
        Ok(t) => Some(t),
        Err(e) => {
            eprintln!("epitome-llc: {e}");
            None
        }
    };
    if sample {
        let Some(target) = load("") else { return ExitCode::FAILURE };
        let syntax = opts.syntax.unwrap_or(epitome_codegen::abi::Syntax::Elf);
        print!("{}", epitome_codegen::check::sample_assembly(&target, syntax, 3));
        return ExitCode::SUCCESS;
    }
    if check {
        let Some(target) = load("") else { return ExitCode::FAILURE };
        let report = epitome_codegen::check::check(&target);
        print!("{}", report.render(verbose));
        return if report.errors() == 0 { ExitCode::SUCCESS } else { ExitCode::FAILURE };
    }
    if patterns {
        let Some(target) = load("") else { return ExitCode::FAILURE };
        for p in &target.patterns {
            let emitted: Vec<&str> = p.emit.iter().map(|e| target.instrs[e.instr].name.as_str()).collect();
            println!("{:<40} {} IR ops -> {}", p.name, p.nodes, emitted.join(", "));
        }
        println!("{} patterns, {} trie nodes", target.patterns.len(), target.trie.nodes.len());
        return ExitCode::SUCCESS;
    }
    let Some(input) = input else {
        eprintln!("{USAGE}");
        return ExitCode::FAILURE;
    };
    let src = match epitome_ir::io::read_to_string(&input) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("epitome-llc: {input}: {e}");
            return ExitCode::FAILURE;
        }
    };
    let mut module = match epitome_ir::parse_module(&src) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("epitome-llc: {input}: {e:?}");
            return ExitCode::FAILURE;
        }
    };
    let triple = module.triple.as_ref().map(|s| String::from_utf8_lossy(s).into_owned()).unwrap_or_default();
    let Some(target) = load(&triple) else { return ExitCode::FAILURE };
    match target.machine.regs(&reserve) {
        Ok(r) => opts.reserve.extend(r),
        Err(e) => {
            eprintln!("epitome-llc: --reserve: {e}");
            return ExitCode::FAILURE;
        }
    }
    let started = std::time::Instant::now();
    opts.stop = stop;
    opts.print_prepared = print_prepared;
    opts.chop = match epitome_ir::chop::limit_from(chop.as_deref(), "EPITOME_LLC_CHOP") {
        Ok(l) => l,
        Err(e) => {
            eprintln!("epitome-llc: {e}");
            return ExitCode::FAILURE;
        }
    };
    let compiled = match compile(&target, &mut module, &opts) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("epitome-llc: {e}");
            return ExitCode::FAILURE;
        }
    };
    for e in &compiled.errors {
        eprintln!("epitome-llc: {e}");
    }
    // Autochop's reports, as epitome-opt's.
    if let Some((k, steps, last)) = &compiled.chop_stopped {
        let name = compiled.chop_names.get(*k).map_or("?", String::as_str);
        match last {
            Some(d) => eprintln!("; chop: stopped in @{name} (function {k}) after step {steps}: {d}"),
            None => eprintln!("; chop: stopped in @{name} (function {k}) after {steps} steps"),
        }
    }
    if chop_counts {
        for (k, c) in compiled.chop_counts.iter().enumerate() {
            eprintln!("; chop: function {k} @{}: {} elective choices", compiled.chop_names[k], c.steps);
        }
    }
    {
        let counts: Vec<u64> = compiled.chop_counts.iter().map(|c| c.steps).collect();
        if let Err(e) = epitome_ir::chop::report_counts("llc", &compiled.chop_names, &counts) {
            eprintln!("EPITOME_CHOP_COUNTS: {e}");
        }
        if let Ok(dump) = std::env::var("EPITOME_CHOP_DUMP")
            && let Err(e) = std::fs::write(&dump, &compiled.asm)
        {
            eprintln!("{dump}: {e}");
        }
    }
    if stats {
        use epitome_codegen::isel::stats::*;
        let total = started.elapsed();
        let (roots, steps, cands, valid) = (get(&ROOTS), get(&STEPS), get(&CANDIDATES), get(&VALID));
        let insts: usize = compiled.funcs.iter().map(|f| f.blocks.iter().map(|b| b.insts.len()).sum::<usize>()).sum();
        eprintln!(
            "; {} functions, {insts} machine instructions; {roots} roots matched: {:.1} trie steps, {:.1} candidates, {:.1} valid per root",
            compiled.funcs.len(),
            steps as f64 / roots.max(1) as f64,
            cands as f64 / roots.max(1) as f64,
            valid as f64 / roots.max(1) as f64
        );
        let m = get(&NANOS) as f64 / 1e6;
        eprintln!(
            "; matching {m:.1} ms ({:.0} ns per root: {:.0} walking the trie, {:.0} checking candidates) of {:.1} ms compiling",
            get(&NANOS) as f64 / roots.max(1) as f64,
            get(&WALK_NANOS) as f64 / roots.max(1) as f64,
            (get(&NANOS) - get(&WALK_NANOS)) as f64 / roots.max(1) as f64,
            total.as_secs_f64() * 1e3
        );
        use epitome_codegen::timing;
        let ms = |c| timing::get(c) as f64 / 1e6;
        eprintln!(
            "; stages, summed over functions: prepare {:.1} ms, selection {:.1} ms, allocation {:.1} ms, frames {:.1} ms, layout {:.1} ms",
            ms(&timing::PREPARE),
            ms(&timing::SELECT),
            ms(&timing::ALLOCATE),
            ms(&timing::FRAME),
            ms(&timing::LAYOUT)
        );
    }
    // The opaque functions, beside the output.
    if let Some(ir) = &compiled.opaque {
        let Some(path) = output.as_deref().filter(|p| *p != "-") else {
            eprintln!("epitome-llc: the module has epitome-opaque functions, which need -o <file> (<file>.opaque.ll)");
            return ExitCode::FAILURE;
        };
        let path = format!("{path}.opaque.ll");
        if let Err(e) = epitome_ir::io::write(&path, ir.as_bytes()) {
            eprintln!("epitome-llc: {path}: {e}");
            return ExitCode::FAILURE;
        }
    }
    let object = filetype.unwrap_or_else(|| output.as_deref().is_some_and(|o| o.ends_with(".o")));
    if object {
        let bytes = match compiled.object_file(&target) {
            Ok(b) => b,
            Err(e) => {
                eprintln!("epitome-llc: {e}");
                return ExitCode::FAILURE;
            }
        };
        let Some(path) = output else {
            eprintln!("epitome-llc: an object file needs -o");
            return ExitCode::FAILURE;
        };
        if let Err(e) = epitome_ir::io::write(&path, bytes) {
            eprintln!("epitome-llc: {path}: {e}");
            return ExitCode::FAILURE;
        }
        return if compiled.errors.is_empty() { ExitCode::SUCCESS } else { ExitCode::FAILURE };
    }
    let mut text = compiled.asm.clone();
    if hex || disasm {
        text.clear();
        for f in &compiled.funcs {
            let code = match mir::encode_function(&target, f) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("epitome-llc: {e}");
                    return ExitCode::FAILURE;
                }
            };
            text.push_str(&format!("{}:\n", f.name));
            for (k, &start) in code.starts.iter().enumerate() {
                let end = code.starts.get(k + 1).copied().unwrap_or(code.bytes.len());
                let bytes = &code.bytes[start..end];
                // A word as a word; other lengths as their bytes.
                let hex = if bytes.len() == 4 {
                    format!("{:08x}", epitome_codegen::enc::word(bytes))
                } else {
                    bytes.iter().map(|b| format!("{b:02x}")).collect::<Vec<_>>().join(" ")
                };
                let fix = code
                    .fixups
                    .iter()
                    .find(|x| x.offset - x.at == start)
                    .map(|x| format!("  ; fixup {}", x.symbol))
                    .unwrap_or_default();
                if disasm {
                    let (asm, _) = mir::disassemble(&target, bytes, start as u64);
                    text.push_str(&format!("{start:6x}: {hex}  {asm}{fix}\n"));
                } else {
                    text.push_str(&format!("{hex}\n"));
                }
            }
        }
    }
    match output {
        Some(path) => {
            if let Err(e) = epitome_ir::io::write(&path, text) {
                eprintln!("epitome-llc: {path}: {e}");
                return ExitCode::FAILURE;
            }
        }
        None => print!("{text}"),
    }
    ExitCode::SUCCESS
}
