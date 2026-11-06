//! Compiles the C programs in `tests/programs/` for RISC-V RV64, with the
//! description in `targets/riscv64/` read at run time (it is not built in),
//! and runs them on the RV64 machine in `riscv/emu.rs`.
//!
//! clang here has no RISC-V back end, so the IR is clang's for
//! `aarch64-linux-gnu` (LP64, as RISC-V's LP64D is, and with unsigned
//! `char`), against a few declarations of the C library's
//! (`riscv/include`). The reference is the program built natively by clang
//! with unsigned `char`. For each program, built two ways (clang `-O2`
//! without vectorising, and `-O2` vectorised, whose vectors RISC-V without
//! V does lane by lane), the test:
//!
//! 1. compiles both files to RISC-V ELF objects, links them on the RV64
//!    machine, runs `main`, and checks it prints what the reference does;
//! 2. checks epitome-as makes the same object of our assembly;
//! 3. checks epitome-objdump reads each object back as our assembly;
//! 4. does 1 again with Zbb on (`--mattr=+zbb`), whose instructions replace
//!    recipes for minimum, maximum and more, and `bits.c` with M off
//!    (`--mattr=-m`).
//!
//! `varargs.c` is left out: its IR's `va_list` is AArch64's. The test is
//! skipped when `clang` is not installed.

#[path = "riscv/emu.rs"]
mod emu;

use std::path::{Path, PathBuf};
use std::process::Command;

use epitome_codegen::assembler::Assembler;
use epitome_codegen::objdump::{self, Image};
use epitome_codegen::target::Target;
use epitome_codegen::{Options, compile};

/// RISC-V's aliases: mnemonics the disassembler may read back for an
/// instruction we wrote another way, or the other way round.
const ALIASES: &[(&str, &str)] = &[
    ("mv", "addi"),
    ("li", "addi"),
    ("neg", "sub"),
    ("negw", "subw"),
    ("sext.w", "addiw"),
    ("seqz", "sltiu"),
    ("snez", "sltu"),
    ("j", "jal"),
];

fn run(cmd: &mut Command) -> Result<String, String> {
    let out = cmd.output().map_err(|e| format!("{cmd:?}: {e}"))?;
    if !out.status.success() {
        return Err(format!("{cmd:?} failed:\n{}", String::from_utf8_lossy(&out.stderr)));
    }
    Ok(String::from_utf8_lossy(&out.stdout).into_owned())
}

/// Our RISC-V object for C file `src`, built by clang with `flags`.
fn object(t: &Target, src: &Path, flags: &[&str], tmp: &Path, mattr: &[String]) -> Result<(Vec<u8>, String), String> {
    let include = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/riscv/include");
    let stem = src.file_stem().unwrap().to_string_lossy().into_owned();
    let ll = tmp.join(format!("{stem}.{}.ll", flags.join("")));
    run(Command::new("clang")
        .args(["--target=aarch64-linux-gnu", "-fno-math-errno", "-S", "-emit-llvm"])
        .args(flags)
        .arg("-I")
        .arg(&include)
        .arg(src)
        .arg("-o")
        .arg(&ll))?;
    let text = std::fs::read_to_string(&ll).unwrap();
    let mut m = epitome_ir::parse_module(&text).map_err(|e| format!("{stem}: {e:?}"))?;
    let opts = Options { mattr: mattr.to_vec(), ..Options::default() };
    let compiled = compile(t, &mut m, &opts).map_err(|e| format!("{stem}: {e}"))?;
    let bytes = compiled.object_file(t).map_err(|e| format!("{stem}: {e}"))?;
    std::fs::write(tmp.join(format!("{stem}.s")), &compiled.asm).unwrap();
    Ok((bytes, compiled.asm))
}

fn check_program(t: &Target, dir: &Path, tmp: &Path, name: &str) -> Result<Vec<String>, String> {
    let src = dir.join(format!("{name}.c"));
    let main = dir.join(format!("{name}.main.c"));
    let wrap: &[&str] =
        if std::fs::read_to_string(&src).unwrap().contains("// no-fwrapv") { &[] } else { &["-fwrapv"] };
    let reference = tmp.join(format!("{name}.ref"));
    run(Command::new("clang")
        .args(["-funsigned-char", "-O1"])
        .args(wrap)
        .arg(&src)
        .arg(&main)
        .arg("-lm") // (Linux's libm is not in libc)
        .arg("-o")
        .arg(&reference))?;
    // A NaN's sign is the host's: RISC-V's default NaN is positive, x86's
    // negative (`-nan`), and C does not say which.
    let want = run(&mut Command::new(&reference))?.replace("-nan", "nan");
    let mut report = Vec::new();
    let configs: &[(&str, &[&str])] =
        &[("O2", &["-O2", "-fno-vectorize", "-fno-slp-vectorize", "-fno-unroll-loops"]), ("vec", &["-O2"])];
    for (config, flags) in configs {
        let flags: Vec<&str> = flags.iter().chain(wrap).copied().collect();
        // bits.c also without M, whose population counts then add bytes
        // up without a multiply.
        let mut mattrs = vec![vec![], vec!["+zbb".to_string()]];
        if name == "bits" {
            mattrs.push(vec!["-m".to_string()]);
        }
        for mattr in mattrs {
            let stem = format!("{name}.{config}{}", mattr.first().map_or("", |m| m.as_str()));
            let (a, asm_a) = object(t, &src, &flags, tmp, &mattr)?;
            // main is built with the default features, but for Zbb.
            let main_mattr: Vec<String> = mattr.iter().filter(|m| m.starts_with('+')).cloned().collect();
            let (b, asm_b) = object(t, &main, &flags, tmp, &main_mattr)?;
            // 2, 3. The assembler and the disassembler agree with the
            // object written directly.
            for (bytes, asm) in [(&a, &asm_a), (&b, &asm_b)] {
                let assembled = Assembler::new(t)
                    .assemble(asm, None)
                    .and_then(|x| x.file(t))
                    .map_err(|e| format!("{stem}: epitome-as: {e}"))?;
                if &assembled != bytes {
                    return Err(format!("{stem}: epitome-as makes another object of our assembly"));
                }
                let img = Image::read(bytes, &t.machine).map_err(|e| format!("{stem}: {e}"))?;
                objdump::reads_back(t, &img, asm, ALIASES).map_err(|e| format!("{stem}: epitome-objdump: {e}"))?;
            }
            // 1. It runs, and prints what the reference prints.
            let mut m = emu::Machine::link(&[a, b]).map_err(|e| format!("{stem}: linking: {e}"))?;
            let status =
                m.run(2_000_000_000).map_err(|e| format!("{stem}: running: {e}\nprinted so far:\n{}", m.out))?;
            let out = m.out.replace("-nan", "nan");
            if out != want || status != 0 {
                let first = out.lines().zip(want.lines()).position(|(x, y)| x != y);
                return Err(format!(
                    "{stem}: printed something else (status {status}); first differing line {first:?}:\n  ours:   {:?}\n  theirs: {:?}",
                    first.and_then(|k| out.lines().nth(k)),
                    first.and_then(|k| want.lines().nth(k))
                ));
            }
            report.push(format!("{stem}: ran, {} lines as the reference", want.lines().count()));
        }
    }
    Ok(report)
}

#[test]
fn programs_run_on_riscv() {
    let ok = Command::new("clang").arg("--version").output().is_ok_and(|o| o.status.success());
    if !ok {
        eprintln!("clang not found; skipping");
        return;
    }
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let t = Target::load_path(root.join("targets/riscv64").to_str().unwrap()).unwrap();
    let dir = root.join("tests/programs");
    let tmp = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("riscv");
    std::fs::create_dir_all(&tmp).unwrap();
    let mut failures = Vec::new();
    for name in [
        "atomics", "basic", "bits", "divs", "loops", "calls", "floats", "structs", "tails", "tls", "vectors", "vla",
        "wide",
    ] {
        match check_program(&t, &dir, &tmp, name) {
            Ok(r) => r.iter().for_each(|l| println!("ok {l}")),
            Err(e) => failures.push(e),
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

/// Inline assembly (`asm/riscv.ll`, written as clang would write it from
/// C), compiled with a main built here, linked and run on the RV64
/// machine: it must print what the same functions in C print.
#[test]
fn inline_asm_runs_on_riscv() {
    let ok = Command::new("clang").arg("--version").output().is_ok_and(|o| o.status.success());
    if !ok {
        eprintln!("clang not found; skipping");
        return;
    }
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let t = Target::load_path(root.join("targets/riscv64").to_str().unwrap()).unwrap();
    let dir = root.join("tests/asm");
    let tmp = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("riscv-asm");
    std::fs::create_dir_all(&tmp).unwrap();
    let reference = tmp.join("ref");
    run(Command::new("clang")
        .args(["-O1"])
        .arg(dir.join("riscv.ref.c"))
        .arg(dir.join("riscv.main.c"))
        .arg("-o")
        .arg(&reference))
    .unwrap();
    let want = run(&mut Command::new(&reference)).unwrap();
    let text = std::fs::read_to_string(dir.join("riscv.ll")).unwrap();
    let mut m = epitome_ir::parse_module(&text).unwrap();
    let compiled = compile(&t, &mut m, &Options::default()).unwrap();
    std::fs::write(tmp.join("riscv.s"), &compiled.asm).unwrap();
    let a = compiled.object_file(&t).unwrap_or_else(|e| panic!("{e}\n{}", compiled.asm));
    let (b, _) = object(&t, &dir.join("riscv.main.c"), &["-O2"], &tmp, &[]).unwrap();
    let mut machine = emu::Machine::link(&[a, b]).unwrap();
    let status = machine.run(100_000_000).unwrap_or_else(|e| panic!("{e}\nprinted so far:\n{}", machine.out));
    assert_eq!((machine.out.as_str(), status), (want.as_str(), 0), "\n{}", compiled.asm);
}
