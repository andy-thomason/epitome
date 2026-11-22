//! The `epitome-rustc` program itself: how it fails on bad input.

use std::path::{Path, PathBuf};
use std::process::Command;

/// A fresh directory under the system's temporary directory.
fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("epitome-rustc-driver-{name}-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

/// Builds `roots` to an object file: the exit code and what went to stderr.
fn build(dir: &Path, roots: &[PathBuf]) -> (Option<i32>, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_epitome-rustc"))
        .arg("--emit=obj")
        .arg("-o")
        .arg(dir.join("out.o"))
        .args(roots)
        .output()
        .unwrap();
    (out.status.code(), String::from_utf8_lossy(&out.stderr).into_owned())
}

/// A crate root that cannot be read or parsed is an error, whichever
/// crate it is, not a crate of no files for later stages to trip over (the
/// first crate's root is where trait selection starts).
#[test]
fn bad_crate_roots() {
    let dir = scratch("roots");
    let main = dir.join("main.rs");
    std::fs::write(&main, "fn main() {}\n").unwrap();
    let bad = dir.join("bad.rs");
    std::fs::write(&bad, "fn (\n").unwrap();
    let missing = dir.join("missing.rs");
    for roots in [[missing.clone(), main.clone()], [main.clone(), missing.clone()], [bad.clone(), main.clone()]] {
        let (code, stderr) = build(&dir, &roots);
        assert_eq!(code, Some(1), "{roots:?}:\n{stderr}");
        assert!(stderr.contains("error"), "{roots:?}:\n{stderr}");
        assert!(!stderr.contains("panicked"), "{roots:?}:\n{stderr}");
    }
    let (_, stderr) = build(&dir, &[missing.clone(), main.clone()]);
    assert!(stderr.contains("cannot read"), "{stderr}");
    std::fs::remove_dir_all(&dir).ok();
}
