//! `-C panic=immediate-abort`: `std` and `core` built so that a panic
//! aborts at once. A program prints as it does with the default strategy,
//! a panic stops it with nothing on stderr, and the program is smaller
//! (no code to format and print the message). Where `--std` builds (x86-64
//! Linux and AArch64 macOS), with rust-src.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// Builds `src` with `--std` and `opts` into `exe`.
fn build(src: &Path, exe: &Path, opts: &[&str]) {
    let out = Command::new(env!("CARGO_BIN_EXE_epitome-rustc")).arg("--std").args(opts).arg("-o").arg(exe).arg(src).output().unwrap();
    assert!(out.status.success(), "{src:?} {opts:?} not built:\n{}", String::from_utf8_lossy(&out.stderr));
}

fn run(exe: &Path) -> Output {
    Command::new(exe).output().unwrap()
}

#[test]
fn immediate_abort() {
    if !cfg!(any(all(target_os = "linux", target_arch = "x86_64"), all(target_os = "macos", target_arch = "aarch64"))) {
        eprintln!("not a target `--std` builds for: skipped");
        return;
    }
    let Ok(out) = Command::new("rustc").args(["--print", "sysroot"]).output() else {
        eprintln!("no rustc: skipped");
        return;
    };
    let root = PathBuf::from(String::from_utf8_lossy(&out.stdout).trim());
    if !root.join("lib/rustlib/src/rust/library/std/src/lib.rs").exists() {
        eprintln!("no rust-src: skipped");
        return;
    }
    let dir = std::env::temp_dir().join(format!("epitome-rustc-panic-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();

    let hello = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/run/std_hello.rs");
    let (plain, small) = (dir.join("hello"), dir.join("hello-ia"));
    build(&hello, &plain, &[]);
    build(&hello, &small, &["-Cpanic=immediate-abort"]);
    let (want, got) = (run(&plain), run(&small));
    assert!(want.status.success() && got.status.success());
    assert_eq!(String::from_utf8_lossy(&got.stdout), String::from_utf8_lossy(&want.stdout));
    let size = |p: &Path| std::fs::metadata(p).unwrap().len();
    assert!(size(&small) < size(&plain), "{} bytes, not fewer than {}", size(&small), size(&plain));

    let src = dir.join("panics.rs");
    std::fs::write(&src, "fn main() {\n    println!(\"before\");\n    let v: Vec<i32> = Vec::new();\n    println!(\"{}\", v[std::hint::black_box(3)]);\n}\n").unwrap();
    let exe = dir.join("panics");
    build(&src, &exe, &["-C", "panic=immediate-abort"]);
    let out = run(&exe);
    assert_eq!(String::from_utf8_lossy(&out.stdout), "before\n");
    assert!(!out.status.success());
    assert!(out.stderr.is_empty(), "stderr: {}", String::from_utf8_lossy(&out.stderr));
    let _ = std::fs::remove_dir_all(&dir);
}
