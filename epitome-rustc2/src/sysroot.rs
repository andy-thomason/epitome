//! The standard library's crates, from the toolchain's rust-src: the
//! crates `std` is built from (`core`, `alloc`, `std` and those beside
//! them in `library/`, and the crates.io ones it vendors in
//! `library/vendor/`), with what rustc's bootstrap gives them that their
//! sources expect: the `rustc-dep-of-std` feature, and `STD_ENV_ARCH`.

use crate::lexer::Edition;
use crate::session::CrateRoot;
use std::path::{Path, PathBuf};

/// `library/` of the toolchain's rust-src, if it is installed.
pub fn rust_src() -> Option<PathBuf> {
    let out = std::process::Command::new("rustc").args(["--print", "sysroot"]).output().ok()?;
    let root = PathBuf::from(String::from_utf8(out.stdout).ok()?.trim());
    let lib = root.join("lib/rustlib/src/rust/library");
    lib.join("std/src/lib.rs").exists().then_some(lib)
}

/// The crates `std` needs, each before those that use it, from
/// `library` (rust-src's `library/`).
pub fn std_crates(library: &Path) -> Result<Vec<CrateRoot>, String> {
    let mut out = Vec::new();
    let mut add = |name: &str, dir: PathBuf| {
        let edition = edition(&dir.join("Cargo.toml"));
        out.push(CrateRoot::new(name, dir.join("src/lib.rs")).edition(edition));
    };
    add("core", library.join("core"));
    add("alloc", library.join("alloc"));
    for (name, dir) in [("libc", "libc"), ("cfg_if", "cfg-if"), ("hashbrown", "hashbrown"), ("rustc_demangle", "rustc-demangle")] {
        add(name, vendored(library, dir)?);
    }
    for name in ["std_detect", "unwind", "panic_abort", "std"] {
        add(name, library.join(name));
    }
    Ok(out)
}

/// The `cfg` options `std`'s crates are built with, as `name = value`:
/// the vendored crates' `rustc-dep-of-std` feature.
pub fn std_cfg() -> Vec<(&'static str, &'static str)> {
    vec![("feature", "rustc-dep-of-std")]
}

/// The environment `std`'s sources read with `env!`, for a target whose
/// architecture is `arch` (`x86_64`).
pub fn std_env(arch: &str) -> Vec<(String, String)> {
    vec![("STD_ENV_ARCH".to_string(), arch.to_string())]
}

/// The directory of the vendored crate `name` (`library/vendor/name-x.y.z`).
fn vendored(library: &Path, name: &str) -> Result<PathBuf, String> {
    let vendor = library.join("vendor");
    let prefix = format!("{name}-");
    let mut found: Vec<PathBuf> = std::fs::read_dir(&vendor)
        .map_err(|e| format!("{}: {e}", vendor.display()))?
        .flatten()
        .map(|e| e.path())
        .filter(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .and_then(|n| n.strip_prefix(&prefix))
                .is_some_and(|v| v.starts_with(|c: char| c.is_ascii_digit()))
        })
        .collect();
    found.sort();
    found.pop().ok_or_else(|| format!("no `{name}` in {}", vendor.display()))
}

/// The edition a `Cargo.toml` names (2015 if none).
fn edition(manifest: &Path) -> Edition {
    let text = std::fs::read_to_string(manifest).unwrap_or_default();
    text.lines()
        .find_map(|l| {
            let rest = l.trim().strip_prefix("edition")?.trim_start().strip_prefix('=')?;
            Edition::parse(rest.trim().trim_matches('"'))
        })
        .unwrap_or(Edition::E2015)
}
