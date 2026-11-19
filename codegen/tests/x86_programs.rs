//! Compiles C programs for x86-64 with epitome-llc and runs them, on an
//! x86-64 Linux machine (elsewhere the test is skipped, as it is without
//! clang).
//!
//! For each `tests/programs/NAME.c` of `PROGRAMS` (and `tests/x86/NAME.c` of
//! `X86_PROGRAMS`) and its `NAME.main.c`, in
//! the configurations of `tests/programs.rs` (clang -O0, that through
//! epitome-opt, -O2 without vectorising, and -O2), `-O2 -fPIC` (linked
//! into a shared library the main calls), and `-O2 -mavx2 -mfma` (256-bit
//! vectors), the test:
//!
//! 1. compiles clang's IR for `x86_64-linux-gnu` to assembly, assembles it
//!    with epitome-as, links it with the main (built by clang), runs it,
//!    and checks it prints what clang's build of both prints;
//! 2. assembles the assembly with clang too, and checks the code and the
//!    relocations are ours;
//! 3. reads our object back with epitome-objdump's code, which must give
//!    our assembly.

use std::path::Path;
use std::process::Command;

use epitome_codegen::abi::Syntax;
use epitome_codegen::assembler::Assembler;
use epitome_codegen::objdump::{self, Image};
use epitome_codegen::target::Target;
use epitome_codegen::{Options, compile};

/// The programs that run on x86-64 so far.
const PROGRAMS: &[&str] = &[
    "atomics",
    "atomics_lib",
    "basic",
    "bits",
    "calls",
    "divs",
    "floats",
    "frames",
    "half",
    "hfa",
    "loops",
    "structs",
    "tails",
    "tls",
    "varargs",
    "vectors",
    "vla",
    "wide",
];

/// And those of `tests/x86`, for x86 alone (`long double` is x87's).
const X86_PROGRAMS: &[&str] = &["fminmax", "ldouble", "vecops", "lanes"];

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

/// The code of an object's `.text`, and its relocations.
fn text_of(t: &Target, bytes: &[u8]) -> Result<(Vec<u8>, Vec<(u64, String, String, i64)>), String> {
    let img = Image::read(bytes, &t.machine)?;
    let s = img.sections.iter().find(|s| s.name == ".text").ok_or("no .text")?;
    let mut relocs: Vec<_> =
        s.relocs.iter().map(|r| (r.offset, r.type_name.clone(), r.target.clone(), r.addend)).collect();
    relocs.sort();
    Ok((s.data.clone(), relocs))
}

fn check_program(t: &Target, dir: &Path, tmp: &Path, name: &str) -> Result<(), String> {
    let src = dir.join(format!("{name}.c"));
    let main = dir.join(format!("{name}.main.c"));
    let text = std::fs::read_to_string(&src).unwrap();
    let wrap: &[&str] = if text.contains("// no-fwrapv") { &[] } else { &["-fwrapv"] };
    let tails = text.contains("// needs-tail-calls");
    let target = ["--target=x86_64-linux-gnu"];
    let reference = tmp.join(format!("{name}.ref"));
    // (Without errno, clang's build calls no sqrtf16, which C libraries
    // lack: ours takes it as llvm.sqrt.f16.)
    run(Command::new("clang")
        .args(wrap)
        .arg("-fno-math-errno")
        .args(["-O2", "-no-pie"])
        .arg(&src)
        .arg(&main)
        .args(["-lm", "-latomic", "-o"])
        .arg(&reference))?;
    let want = run(&mut Command::new(&reference))?;
    let configs: &[(&str, &[&str])] = &[
        ("O0", &["-O0", "-Xclang", "-disable-O0-optnone"]),
        ("opt", &["-O0", "-Xclang", "-disable-O0-optnone"]),
        ("O2", &["-O2", "-fno-vectorize", "-fno-slp-vectorize", "-fno-unroll-loops"]),
        ("vec", &["-O2"]),
        ("pic", &["-O2", "-fPIC"]),
        ("avx2", &["-O2", "-mavx2", "-mfma"]),
    ];
    for (config, flags) in configs {
        if tails && matches!(*config, "O0" | "opt") {
            continue;
        }
        let stem = format!("{name}.{config}");
        let ll = tmp.join(format!("{stem}.ll"));
        run(Command::new("clang")
            .args(target)
            .args(wrap)
            .args(*flags)
            .args(["-S", "-emit-llvm"])
            .arg(&src)
            .arg("-o")
            .arg(&ll))?;
        let mut m =
            epitome_ir::parse_module(&std::fs::read_to_string(&ll).unwrap()).map_err(|e| format!("{stem}: {e:?}"))?;
        if *config == "opt" {
            epitome_opt::pipeline::run(&mut m, &epitome_opt::pipeline::Options { threads: 1, ..Default::default() });
            m = epitome_ir::parse_module(&m.to_string()).map_err(|e| format!("{stem}: reparse: {e:?}"))?;
        }
        let compiled = compile(t, &mut m, &Options::default()).map_err(|e| format!("{stem}: {e}"))?;
        let s = tmp.join(format!("{stem}.s"));
        std::fs::write(&s, &compiled.asm).unwrap();

        // 1. Ours, assembled by epitome-as, behaves as clang's build.
        let ours = Assembler::new(t)
            .assemble(&compiled.asm, Some(Syntax::Elf))
            .and_then(|a| a.file(t))
            .map_err(|e| format!("{stem}: epitome-as: {e}"))?;
        let obj = tmp.join(format!("{stem}.o"));
        std::fs::write(&obj, &ours).unwrap();
        let exe = tmp.join(&stem);
        if *config == "pic" {
            // A shared library of ours, which the main is linked with.
            let lib = tmp.join(format!("lib{stem}.so"));
            run(Command::new("clang").arg("-shared").arg(&obj).arg("-o").arg(&lib))?;
            run(Command::new("clang")
                .arg("-O2")
                .arg(&main)
                .arg(&lib)
                .arg(format!("-Wl,-rpath,{}", tmp.display()))
                .args(["-lm", "-latomic", "-o"])
                .arg(&exe))?;
        } else {
            // (The main as the code it calls: 256-bit vectors are passed in
            // ymm registers only with AVX.)
            let main_flags: &[&str] = if *config == "avx2" { &["-mavx2", "-mfma"] } else { &[] };
            run(Command::new("clang")
                .args(["-O2", "-no-pie"])
                .args(main_flags)
                .arg(&obj)
                .arg(&main)
                .args(["-lm", "-latomic", "-o"])
                .arg(&exe))?;
        }
        let got = run(&mut Command::new(&exe))?;
        if got != want {
            return Err(format!("{stem}: output differs from clang's build; see {}", s.display()));
        }

        // 2. clang's assembler makes the same code and relocations.
        let theirs = tmp.join(format!("{stem}.clang.o"));
        run(Command::new("clang").args(target).arg("-c").arg(&s).arg("-o").arg(&theirs))?;
        let (oc, or) = text_of(t, &ours)?;
        let (tc, tr) = text_of(t, &std::fs::read(&theirs).unwrap())?;
        if oc != tc {
            let at = oc.iter().zip(&tc).position(|(a, b)| a != b).unwrap_or(oc.len().min(tc.len()));
            return Err(format!("{stem}: the code differs from clang's assembler's at byte {at:#x}"));
        }
        if or != tr {
            return Err(format!("{stem}: the relocations differ from clang's assembler's"));
        }

        // 3. epitome-objdump reads our object back as our assembly.
        let img = Image::read(&ours, &t.machine)?;
        objdump::reads_back(t, &img, &compiled.asm, &[]).map_err(|e| format!("{stem}: {e}"))?;
    }
    Ok(())
}

#[test]
fn x86_programs_behave_the_same() {
    if !(cfg!(target_arch = "x86_64") && cfg!(target_os = "linux")) {
        eprintln!("not an x86-64 Linux machine: skipped");
        return;
    }
    if !Command::new("clang").arg("--version").output().is_ok_and(|o| o.status.success()) {
        eprintln!("clang is not installed: skipped");
        return;
    }
    let t = Target::x86_64();
    let here = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests");
    let tmp = Path::new(env!("CARGO_TARGET_TMPDIR")).join("x86_programs");
    std::fs::create_dir_all(&tmp).unwrap();
    let all = PROGRAMS.iter().map(|n| ("programs", n)).chain(X86_PROGRAMS.iter().map(|n| ("x86", n)));
    let failed: Vec<String> =
        all.filter_map(|(dir, name)| check_program(&t, &here.join(dir), &tmp, name).err()).collect();
    assert!(failed.is_empty(), "{}", failed.join("\n"));
}
