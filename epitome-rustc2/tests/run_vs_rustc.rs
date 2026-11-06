//! Each program in `tests/run/` is compiled by `epitome-rustc2` (with
//! `core` from rust-src, through Epitome's optimiser and code generator,
//! and without the optimiser) and by rustc, and run: the outputs and exit
//! statuses must be the same.
//!
//! The programs are `#![no_std]` with a C `main`, and write through
//! libc's `putchar`; `floats.rs` uses `#![feature(f16)]`, which
//! `RUSTC_BOOTSTRAP=1` lets rustc take. rustc links them against libc, with a stub for
//! `rust_eh_personality`, which the precompiled `core` refers to although
//! nothing unwinds. Only on x86-64 Linux, with rust-src and `cc`.

use std::path::{Path, PathBuf};
use std::process::Command;

fn sysroot() -> Option<PathBuf> {
    let out = Command::new("rustc").args(["--print", "sysroot"]).output().ok()?;
    Some(PathBuf::from(String::from_utf8(out.stdout).ok()?.trim()))
}

/// Runs `exe` for at most ten seconds: its output and exit status.
fn run(exe: &Path) -> (String, Option<i32>) {
    let out = Command::new("timeout").arg("10").arg(exe).output().expect("timeout runs");
    (String::from_utf8_lossy(&out.stdout).into_owned(), out.status.code())
}

#[test]
fn programs_match_rustc() {
    if !cfg!(all(target_os = "linux", target_arch = "x86_64")) {
        eprintln!("not x86-64 Linux: skipped");
        return;
    }
    let Some(root) = sysroot() else {
        eprintln!("no rustc: skipped");
        return;
    };
    let core = root.join("lib/rustlib/src/rust/library/core/src/lib.rs");
    if !core.exists() {
        eprintln!("no rust-src: skipped");
        return;
    }
    let dir = std::env::temp_dir().join(format!("epitome-rustc2-run-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let stub = dir.join("eh.c");
    std::fs::write(&stub, "void rust_eh_personality(void) {}\n").unwrap();
    let eh = dir.join("eh.o");
    assert!(Command::new("cc").arg("-c").arg(&stub).arg("-o").arg(&eh).status().unwrap().success(), "cc failed");

    let programs_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/run");
    let mut programs: Vec<PathBuf> =
        std::fs::read_dir(&programs_dir).unwrap().flatten().map(|e| e.path()).filter(|p| p.extension().is_some_and(|x| x == "rs")).collect();
    programs.sort();
    assert!(programs.len() >= 5, "programs: {programs:?}");
    let mut failed = Vec::new();
    for p in &programs {
        let name = p.file_stem().unwrap().to_string_lossy().into_owned();
        let rc = dir.join(format!("{name}.rustc"));
        // `RUSTC_BOOTSTRAP` lets this stable rustc take the features a
        // program uses (`f16`).
        let built = Command::new("rustc")
            .env("RUSTC_BOOTSTRAP", "1")
            .args(["--edition", "2024", "-C", "panic=abort", "-C", "link-arg=-lc"])
            .arg(format!("-Clink-arg={}", eh.display()))
            .arg(p)
            .arg("-o")
            .arg(&rc)
            .output()
            .unwrap();
        assert!(built.status.success(), "rustc failed on {name}:\n{}", String::from_utf8_lossy(&built.stderr));
        let want = run(&rc);
        for opt in [true, false] {
            let ep = dir.join(format!("{name}.{}", if opt { "ep" } else { "ep0" }));
            let mut cmd = Command::new(env!("CARGO_BIN_EXE_epitome-rustc2"));
            if !opt {
                cmd.arg("-O0");
            }
            let built = cmd.arg("-o").arg(&ep).arg(&core).arg(p).output().unwrap();
            if !built.status.success() {
                failed.push(format!("{name} (optimised: {opt}): not built:\n{}", String::from_utf8_lossy(&built.stderr)));
                continue;
            }
            let got = run(&ep);
            if got != want {
                failed.push(format!("{name} (optimised: {opt}): got {got:?}\nrustc's gives {want:?}"));
            }
        }
    }
    let _ = std::fs::remove_dir_all(&dir);
    assert!(failed.is_empty(), "{}", failed.join("\n\n"));
}
