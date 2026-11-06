//! Compiles C programs with epitome-llc and checks they behave as clang's
//! build does, and that the machine code is right.
//!
//! For each `tests/programs/NAME.c` (functions) and `NAME.main.c` (a main
//! that calls them and prints the results), in four configurations of the
//! IR given to epitome-llc:
//!
//! - `O0`: clang without optimisation, so every variable is in memory;
//! - `opt`: that, through epitome-opt's pipeline;
//! - `O2`: clang `-O2`, without vectorising or unrolling;
//! - `vec`: clang `-O2` as it is, vectorised and unrolled;
//!
//! the test:
//!
//! 1. builds our assembly with the main and checks the output is the same
//!    as clang's build of both C files;
//! 2. assembles our assembly with clang, and checks each word of the object
//!    code against our own encoder, except where we leave a fixup;
//! 3. disassembles each word with the description and checks it encodes
//!    back to the same word, and that the mnemonic is objdump's, or an
//!    alias of it; and reads the assembler's Mach-O object with
//!    epitome-objdump's code, which must give back our assembly (see 6);
//! 4. does the same for Linux: IR from clang for `aarch64-linux-gnu`,
//!    compiled with the AAPCS64 convention to ELF assembly, assembled by
//!    clang for Linux and checked word by word. (It is not run: there is
//!    no Linux here. `tests/abi.rs` runs AAPCS64 code against clang's.)
//! 5. writes our own ELF object file for it, and checks it says what the
//!    assembler's says: the same sections with the same bytes, the same
//!    relocations against the same symbols, and the same symbols (see
//!    `ElfFile::canonical`);
//! 6. disassembles our object with epitome-objdump's code and checks it
//!    reads back as our assembly: every instruction's mnemonic, and where
//!    a relocation fills an instruction in, the whole instruction, symbol
//!    and all;
//! 7. writes our own Mach-O object file for this machine, checks it says
//!    what the assembler's says (as 5), links it with the main, and checks
//!    the program's output is clang's build's;
//! 8. assembles our assembly with epitome-as, for both formats, and
//!    checks the object is byte for byte the one written directly in 5 and
//!    7. So three ways to the object file (clang's assembler, ours, and
//!    none) agree.
//!
//! The test is skipped when `clang` is not installed.

use std::path::{Path, PathBuf};
use std::process::Command;

use epitome_codegen::assembler::Assembler;
use epitome_codegen::objdump::{self, Image};
use epitome_codegen::target::Target;
use epitome_codegen::{Options, compile, elf, macho, mir};

fn have_tools() -> bool {
    let ok = |c: &str, a: &str| Command::new(c).arg(a).output().is_ok_and(|o| o.status.success());
    ok("clang", "--version") && ok("objdump", "--version")
}

fn run(cmd: &mut Command) -> Result<String, String> {
    let out = cmd.output().map_err(|e| format!("{cmd:?}: {e}"))?;
    if !out.status.success() {
        return Err(format!(
            "{cmd:?} failed:\n{}{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        ));
    }
    Ok(String::from_utf8_lossy(&out.stdout).into_owned())
}

/// Mnemonics objdump may print for an instruction we print another way.
const ALIASES: &[(&str, &str)] = &[
    ("add", "mov"),
    ("mov", "orr"),
    ("mov", "movz"),
    ("mov", "movn"),
    ("mov", "ins"),
    ("neg", "sub"),
    ("sshll", "sxtl"),
    ("ushll", "uxtl"),
    ("sshll2", "sxtl2"),
    ("ushll2", "uxtl2"),
    ("ror", "extr"),
];

fn check_program(t: &Target, dir: &Path, tmp: &Path, name: &str) -> Result<Vec<String>, String> {
    let src = dir.join(format!("{name}.c"));
    let main = dir.join(format!("{name}.main.c"));
    let reference_exe = tmp.join(format!("{name}.ref"));
    // Programs whose arithmetic never overflows say so, and are built
    // without -fwrapv, so that overflow flags reach the optimiser.
    let wrap: &[&str] =
        if std::fs::read_to_string(&src).unwrap().contains("// no-fwrapv") { &[] } else { &["-fwrapv"] };
    // Programs that recurse too deep for a stack without tail calls say
    // so: they are built only with optimisation, which marks calls `tail`.
    let tails = std::fs::read_to_string(&src).unwrap().contains("// needs-tail-calls");
    let opt: &[&str] = if tails { &["-O2"] } else { &[] };
    run(Command::new("clang").args(wrap).args(opt).arg(&src).arg(&main).arg("-o").arg(&reference_exe))?;
    let want = run(&mut Command::new(&reference_exe))?;
    let mut report = Vec::new();
    let configs: &[(&str, &[&str])] = &[
        ("O0", &["-O0", "-Xclang", "-disable-O0-optnone"]),
        ("opt", &["-O0", "-Xclang", "-disable-O0-optnone"]),
        ("O2", &["-O2", "-fno-vectorize", "-fno-slp-vectorize", "-fno-unroll-loops"]),
        ("vec", &["-O2"]),
    ];
    for (config, flags) in configs {
        if tails && matches!(*config, "O0" | "opt") {
            continue;
        }
        let stem = format!("{name}.{config}");
        let ll = tmp.join(format!("{stem}.ll"));
        run(Command::new("clang").args(wrap).args(*flags).args(["-S", "-emit-llvm"]).arg(&src).arg("-o").arg(&ll))?;
        let text = std::fs::read_to_string(&ll).unwrap();
        let mut m = epitome_ir::parse_module(&text).map_err(|e| format!("{stem}: {e:?}"))?;
        if *config == "opt" {
            epitome_opt::pipeline::run(&mut m, &epitome_opt::pipeline::Options { threads: 1, ..Default::default() });
            let text = m.to_string();
            m = epitome_ir::parse_module(&text).map_err(|e| format!("{stem}: reparse: {e:?}"))?;
        }
        let compiled = compile(t, &mut m, &Options::default()).map_err(|e| format!("{stem}: {e}"))?;
        let s = tmp.join(format!("{stem}.s"));
        std::fs::write(&s, &compiled.asm).unwrap();

        // 1. It behaves the same.
        let exe = tmp.join(&stem);
        run(Command::new("clang").args(opt).arg(&s).arg(&main).arg("-o").arg(&exe))?;
        let got = run(&mut Command::new(&exe))?;
        if got != want {
            return Err(format!("{stem}: output differs from clang's build; see {}", s.display()));
        }

        // 2. Our encoding is the assembler's.
        let obj = tmp.join(format!("{stem}.o"));
        run(Command::new("clang").arg("-c").arg(&s).arg("-o").arg(&obj))?;
        let dump = run(Command::new("objdump").arg("-d").arg(&obj))?;
        let theirs = words_of(&dump);
        let mut ours: Vec<(u32, bool)> = Vec::new();
        for f in &compiled.funcs {
            let code = mir::encode_function(t, f)?;
            for (k, w) in code.bytes.chunks(4).enumerate() {
                let w = u32::from_le_bytes(w.try_into().unwrap());
                ours.push((w, code.fixups.iter().any(|x| x.offset == 4 * k)));
            }
        }
        if ours.len() != theirs.len() {
            return Err(format!("{stem}: {} words from us, {} from the assembler", ours.len(), theirs.len()));
        }
        for (k, ((w, fixup), (tw, mnemonic))) in ours.iter().zip(&theirs).enumerate() {
            if w != tw && !fixup {
                return Err(format!("{stem}: word {k} is {w:08x}, the assembler's is {tw:08x} ({mnemonic})"));
            }
            // 3. The disassembler reads it back.
            let Some((i, env, _)) = mir::decode(t, &w.to_le_bytes()) else {
                return Err(format!("{stem}: cannot disassemble {w:08x} ({mnemonic})"));
            };
            let ins = &t.instrs[i];
            if ins.encode(&env) != Some(w.to_le_bytes().to_vec()) {
                return Err(format!("{stem}: {w:08x} disassembles as {} but does not encode back", ins.name));
            }
            let ours_mn = ins.asm.split_whitespace().next().unwrap_or("");
            let ours_mn = ours_mn.split('.').next().unwrap();
            let theirs_mn = mnemonic.split('.').next().unwrap();
            if ours_mn != theirs_mn
                && !ALIASES.contains(&(theirs_mn, ours_mn))
                && !ALIASES.contains(&(ours_mn, theirs_mn))
            {
                return Err(format!("{stem}: {w:08x} is {mnemonic} to objdump, {ours_mn} to us"));
            }
        }
        report.push(format!("{stem}: {} instructions", ours.len()));

        // 3b. epitome-objdump reads the assembler's Mach-O object back
        // as our assembly.
        objdump_reads_back(t, &obj, &compiled.asm, &stem)?;

        // 7. Our own Mach-O object says what the assembler's says, and
        // linked with the main, behaves the same.
        let mine = tmp.join(format!("{stem}.ours.o"));
        let bytes = compiled.object_file(t).map_err(|e| format!("{stem}: {e}"))?;
        std::fs::write(&mine, &bytes).unwrap();
        same_object(t, &mine, &obj, &stem)?;
        // 8. epitome-as makes the same object of our assembly.
        same_assembled(t, &compiled.asm, &bytes, &stem)?;
        let exe = tmp.join(format!("{stem}.ours"));
        run(Command::new("clang").args(opt).arg(&mine).arg(&main).arg("-o").arg(&exe))?;
        if run(&mut Command::new(&exe))? != want {
            return Err(format!("{stem}: built from our object file, the output differs from clang's build"));
        }

        // 4. The same for Linux (AAPCS64, ELF): built from clang's IR for
        // that target, assembled by clang for it, checked word by word.
        // Programs that use what only Apple's convention has so far say so.
        if std::fs::read_to_string(&src).unwrap().contains("// darwin-only") {
            continue;
        }
        let elf_stem = format!("{stem}.linux");
        let ll = tmp.join(format!("{elf_stem}.ll"));
        run(Command::new("clang")
            .args(["--target=aarch64-linux-gnu"])
            .args(wrap)
            .args(*flags)
            .args(["-S", "-emit-llvm"])
            .arg(&src)
            .arg("-o")
            .arg(&ll))?;
        let text = std::fs::read_to_string(&ll).unwrap();
        let mut m = epitome_ir::parse_module(&text).map_err(|e| format!("{elf_stem}: {e:?}"))?;
        if *config == "opt" {
            epitome_opt::pipeline::run(&mut m, &epitome_opt::pipeline::Options { threads: 1, ..Default::default() });
            let text = m.to_string();
            m = epitome_ir::parse_module(&text).map_err(|e| format!("{elf_stem}: reparse: {e:?}"))?;
        }
        let compiled = compile(t, &mut m, &Options::default()).map_err(|e| format!("{elf_stem}: {e}"))?;
        if compiled.funcs.first().is_some_and(|f| f.abi.name != "aapcs64") {
            return Err(format!("{elf_stem}: the Linux triple did not select aapcs64"));
        }
        let s = tmp.join(format!("{elf_stem}.s"));
        std::fs::write(&s, &compiled.asm).unwrap();
        let obj = tmp.join(format!("{elf_stem}.o"));
        run(Command::new("clang").args(["--target=aarch64-linux-gnu", "-c"]).arg(&s).arg("-o").arg(&obj))?;
        let dump = run(Command::new("objdump").arg("-d").arg(&obj))?;
        let theirs = words_of(&dump);
        let mut ours: Vec<(u32, bool)> = Vec::new();
        for f in &compiled.funcs {
            let code = mir::encode_function(t, f)?;
            for (k, w) in code.bytes.chunks(4).enumerate() {
                let w = u32::from_le_bytes(w.try_into().unwrap());
                ours.push((w, code.fixups.iter().any(|x| x.offset == 4 * k)));
            }
        }
        if ours.len() != theirs.len() {
            return Err(format!("{elf_stem}: {} words from us, {} from the assembler", ours.len(), theirs.len()));
        }
        for (k, ((w, fixup), (tw, mnemonic))) in ours.iter().zip(&theirs).enumerate() {
            if w != tw && !fixup {
                return Err(format!("{elf_stem}: word {k} is {w:08x}, the assembler's is {tw:08x} ({mnemonic})"));
            }
        }
        report.push(format!("{elf_stem}: {} instructions", ours.len()));

        // 5. Our object file is the assembler's.
        let mine = tmp.join(format!("{elf_stem}.ours.o"));
        let bytes = compiled.object_file(t).map_err(|e| format!("{elf_stem}: {e}"))?;
        std::fs::write(&mine, &bytes).unwrap();
        same_object(t, &mine, &obj, &elf_stem)?;
        same_assembled(t, &compiled.asm, &bytes, &elf_stem)?;

        // 6. epitome-objdump reads our object back as our assembly.
        objdump_reads_back(t, &mine, &compiled.asm, &elf_stem)?;
    }
    Ok(report)
}

/// Checks two object files, ELF or Mach-O, say the same (see
/// `ElfFile::canonical` and `MachFile::canonical`).
fn same_object(t: &Target, ours: &Path, theirs: &Path, stem: &str) -> Result<(), String> {
    let read = |p: &Path| {
        let bytes = std::fs::read(p).unwrap();
        let text = if macho::is_macho(&bytes) {
            macho::read(&bytes).map(|m| macho::without_unwind(&m.canonical(&t.machine)))
        } else {
            elf::read(&bytes).map(|e| e.canonical(&t.machine))
        };
        text.map_err(|e| format!("{stem}: {}: {e}", p.display()))
    };
    let (a, b) = (read(ours)?, read(theirs)?);
    if a == b {
        return Ok(());
    }
    let diff = epitome_codegen::object::differences(&a, &b, 8);
    Err(format!("{stem}: our object differs from the assembler's ({}):\n{}", ours.display(), diff.join("\n")))
}

/// Checks epitome-as makes exactly `object` of `asm`.
fn same_assembled(t: &Target, asm: &str, object: &[u8], stem: &str) -> Result<(), String> {
    let assembled = Assembler::new(t).assemble(asm, None).map_err(|e| format!("{stem}: epitome-as: {e}"))?.file(t)?;
    if assembled == object { Ok(()) } else { Err(format!("{stem}: epitome-as makes another object of our assembly")) }
}

/// Checks epitome-objdump's disassembly of `obj` reads as `asm`.
fn objdump_reads_back(t: &Target, obj: &Path, asm: &str, stem: &str) -> Result<(), String> {
    let img = Image::read(&std::fs::read(obj).unwrap(), &t.machine).map_err(|e| format!("{stem}: {e}"))?;
    objdump::reads_back(t, &img, asm, ALIASES).map(|_| ()).map_err(|e| format!("{stem}: {e}"))
}

/// The words and mnemonics in objdump's disassembly.
fn words_of(dump: &str) -> Vec<(u32, String)> {
    dump.lines()
        .filter_map(|l| {
            let mut it = l.split_whitespace();
            let addr = it.next()?;
            if !addr.ends_with(':') || !addr.trim_end_matches(':').chars().all(|c| c.is_ascii_hexdigit()) {
                return None;
            }
            let word = u32::from_str_radix(it.next()?, 16).ok()?;
            Some((word, it.next().unwrap_or("").to_string()))
        })
        .collect()
}

#[test]
fn programs_behave_the_same() {
    if !cfg!(all(target_arch = "aarch64", target_os = "macos")) {
        eprintln!("runs AArch64 code on an AArch64 Mac (x86_programs is x86-64 Linux's); skipping");
        return;
    }
    if !have_tools() {
        eprintln!("clang or objdump not found; skipping");
        return;
    }
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/programs");
    let tmp = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("codegen-programs");
    std::fs::create_dir_all(&tmp).unwrap();
    let t = Target::aarch64();
    let mut names: Vec<String> = std::fs::read_dir(&dir)
        .unwrap()
        .filter_map(|e| {
            let p = e.unwrap().path();
            let n = p.file_name()?.to_str()?.to_string();
            (n.ends_with(".c") && !n.ends_with(".main.c")).then(|| n.trim_end_matches(".c").to_string())
        })
        .collect();
    names.sort();
    let mut failures = Vec::new();
    for n in &names {
        match check_program(&t, &dir, &tmp, n) {
            Ok(r) => r.iter().for_each(|l| println!("ok {l}")),
            Err(e) => failures.push(e),
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
