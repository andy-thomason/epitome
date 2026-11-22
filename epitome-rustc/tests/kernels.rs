//! Kernels: each program in `tests/kernels/` has functions marked
//! `#[epitome::target("riscv64")]`. One that says `//! run on: riscv64`
//! is compiled for RISC-V only (`--emit=kernels`) and run on codegen's RV64
//! machine (`codegen/tests/riscv/emu.rs`) from its kernel `main`; one that
//! says `//! run on: host` is built whole (optimised, and with `-O0`) and
//! run here, its calls of kernels launching them on the RV64 interpreter
//! compiled into it. What each prints must be what its `//! prints:` lines
//! say (`{host_arch}` being the machine the test runs on).

#[path = "../../codegen/tests/riscv/emu.rs"]
#[allow(dead_code)]
mod emu;

use std::path::{Path, PathBuf};
use std::process::Command;

#[test]
fn kernels_run_on_riscv() {
    let Ok(out) = Command::new("rustc").args(["--print", "sysroot"]).output() else {
        eprintln!("no rustc: skipped");
        return;
    };
    let root = PathBuf::from(String::from_utf8_lossy(&out.stdout).trim());
    if !root.join("lib/rustlib/src/rust/library/core/src/lib.rs").exists() {
        eprintln!("no rust-src: skipped");
        return;
    }
    let dir = std::env::temp_dir().join(format!("epitome-rustc-kernels-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let programs = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/kernels");
    let mut files: Vec<PathBuf> = std::fs::read_dir(&programs).unwrap().flatten().map(|e| e.path()).filter(|p| p.extension().is_some_and(|x| x == "rs")).collect();
    files.sort();
    assert!(!files.is_empty());
    let mut failed = Vec::new();
    for p in &files {
        let name = p.file_stem().unwrap().to_string_lossy().into_owned();
        let text = std::fs::read_to_string(p).unwrap();
        // `{host_arch}` is the architecture of the machine the test runs on.
        let want: String = text
            .lines()
            .filter_map(|l| l.strip_prefix("//! prints: "))
            .map(|l| format!("{}\n", l.replace("{host_arch}", std::env::consts::ARCH)))
            .collect();
        let out = dir.join(&name);
        if text.contains("//! run on: host") {
            for opt in [&[][..], &["-O0"][..]] {
                let built = Command::new(env!("CARGO_BIN_EXE_epitome-rustc")).arg("--std").args(opt).arg("-o").arg(&out).arg(p).output().unwrap();
                if !built.status.success() {
                    failed.push(format!("{name} {opt:?}: not built:\n{}", String::from_utf8_lossy(&built.stderr)));
                    continue;
                }
                let ran = Command::new(&out).output().unwrap();
                let got = String::from_utf8_lossy(&ran.stdout).into_owned();
                if !ran.status.success() || got != want {
                    failed.push(format!("{name} {opt:?}: got {got:?} ({}), stderr {}, want {want:?}", ran.status, String::from_utf8_lossy(&ran.stderr)));
                }
            }
            continue;
        }
        let built = Command::new(env!("CARGO_BIN_EXE_epitome-rustc")).arg("--emit=kernels").arg("-o").arg(&out).arg(p).output().unwrap();
        if !built.status.success() {
            failed.push(format!("{name}: not built:\n{}", String::from_utf8_lossy(&built.stderr)));
            continue;
        }
        let object = std::fs::read(dir.join(format!("{name}.riscv64.o"))).unwrap();
        let got = emu::Machine::link(&[object]).and_then(|mut m| m.run(100_000_000).map(|code| (code, m.out.clone())));
        match got {
            Ok((0, printed)) if printed == want => {}
            other => failed.push(format!("{name}: got {other:?}, want {want:?}")),
        }
    }
    let _ = std::fs::remove_dir_all(&dir);
    assert!(failed.is_empty(), "{}", failed.join("\n\n"));
}
