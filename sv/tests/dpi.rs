//! DPI-C (`src/sim/dpi.rs`): `tests/dpi/lib.rs`, C functions written in
//! Rust against Annex H, is built as a shared library and loaded into a
//! simulation of `tests/dpi/design.sv`, which imports them: every kind of
//! argument and result in each direction, open arrays, context imports
//! calling exports in the scopes they set, user data, and imported tasks
//! that wait in exported tasks, one of them disabled. What the design
//! prints is compared with what it should print, through `epitome-sv`
//! and through the `epitome-iverilog` and `epitome-vvp` pair.
//!
//! When Verilator can be run, the design is also built with it, without
//! what it does not take, and the two outputs compared.

use std::path::{Path, PathBuf};
use std::process::Command;

fn here(p: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(p)
}

const EXT: &str = if cfg!(target_os = "macos") { "dylib" } else { "so" };

/// Builds the library with the compiler that built this test; returns its
/// path without the extension, as `-sv_lib` takes it.
fn build_lib(dir: &Path) -> PathBuf {
    let stem = dir.join("libdpitest");
    let rustc = std::env::var("RUSTC").unwrap_or_else(|_| "rustc".into());
    let mut c = Command::new(rustc);
    c.args(["--edition", "2024", "-O", "--crate-type", "cdylib", "-o"]).arg(stem.with_extension(EXT)).arg(here("tests/dpi/lib.rs"));
    // The svdpi.h functions and the exports are found in the process.
    if cfg!(target_os = "macos") {
        c.args(["-C", "link-arg=-undefined", "-C", "link-arg=dynamic_lookup"]);
    }
    let r = c.output().expect("run rustc");
    assert!(r.status.success(), "building the DPI library failed:\n{}", String::from_utf8_lossy(&r.stderr));
    stem
}

fn compare(what: &str, out: &str, want: &str) {
    if out == want {
        return;
    }
    for (i, (a, b)) in out.lines().zip(want.lines()).enumerate() {
        if a != b {
            panic!("{what}, line {}: got\n  {a}\nexpected\n  {b}\nwhole output:\n{out}", i + 1);
        }
    }
    panic!("{what}: got {} lines, expected {}:\n{out}", out.lines().count(), want.lines().count());
}

fn tmp(name: &str) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(name);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn imports_and_exports() {
    let dir = tmp("dpi");
    let lib = build_lib(&dir);
    let design = here("tests/dpi/design.sv");

    let r = Command::new(env!("CARGO_BIN_EXE_epitome-sv")).arg(&design).arg("-sv_lib").arg(&lib).output().unwrap();
    let out = String::from_utf8_lossy(&r.stdout);
    let err = String::from_utf8_lossy(&r.stderr);
    assert!(r.status.success(), "{out}{err}");
    assert!(err.is_empty(), "{err}");
    compare("epitome-sv", &out, EXPECTED);

    // As a test runner would: the library named at the run step, or in
    // EPITOME_SV_LIBS.
    let file = dir.join("design.evvp");
    let r = Command::new(env!("CARGO_BIN_EXE_epitome-iverilog")).args(["-g2012", "-o"]).arg(&file).arg(&design).output().unwrap();
    assert!(r.status.success(), "{}", String::from_utf8_lossy(&r.stderr));
    for env in [false, true] {
        let mut c = Command::new(env!("CARGO_BIN_EXE_epitome-vvp"));
        if env {
            c.env("EPITOME_SV_LIBS", lib.with_extension(EXT));
        } else {
            c.arg("-sv_root").arg(&dir).args(["-sv_lib", "libdpitest"]);
        }
        let r = c.arg(&file).output().unwrap();
        let out = String::from_utf8_lossy(&r.stdout);
        assert!(r.status.success(), "{out}{}", String::from_utf8_lossy(&r.stderr));
        compare(if env { "epitome-vvp with EPITOME_SV_LIBS" } else { "epitome-vvp" }, &out, EXPECTED);
    }
}

/// A library loaded at the compile step, as the design file records it.
#[test]
fn library_in_the_design_file() {
    let dir = tmp("dpi-design-file");
    let lib = build_lib(&dir);
    let file = dir.join("design.evvp");
    let r = Command::new(env!("CARGO_BIN_EXE_epitome-iverilog"))
        .arg("-o")
        .arg(&file)
        .arg("-sv_lib")
        .arg(&lib)
        .arg(here("tests/dpi/design.sv"))
        .output()
        .unwrap();
    assert!(r.status.success(), "{}", String::from_utf8_lossy(&r.stderr));
    let r = Command::new(env!("CARGO_BIN_EXE_epitome-vvp")).arg(&file).output().unwrap();
    compare("epitome-vvp", &String::from_utf8_lossy(&r.stdout), EXPECTED);
}

/// A missing C function stops the run with an error.
#[test]
fn missing_function() {
    let dir = tmp("dpi-missing");
    let sv = dir.join("missing.sv");
    std::fs::write(&sv, "module top;\n  import \"DPI-C\" function int no_such_function_here(input int x);\n  initial $display(\"%0d\", no_such_function_here(1));\nendmodule\n").unwrap();
    let r = Command::new(env!("CARGO_BIN_EXE_epitome-sv")).arg(&sv).output().unwrap();
    assert!(!r.status.success());
    assert!(String::from_utf8_lossy(&r.stderr).contains("no C function `no_such_function_here`"));
}

/// The same design and library in Verilator, where it can be run: the
/// outputs agree but on the lines Verilator computes otherwise, by
/// design or by not following Annex H. It passes `shortreal` as `double`
/// (reals, weigh, outs), has no x or z (outs, open fill, selects), clears
/// the bits above a part-select it gets (selects; H.11.5 keeps them), and
/// gives no scaled real time, or no time functions at all in older
/// versions (time; H.13).
#[test]
fn against_verilator() {
    // "Verilator 5.053 ...": older versions depart further, and are not
    // compared.
    let version = Command::new("verilator").arg("--version").output().ok().map(|r| String::from_utf8_lossy(&r.stdout).into_owned());
    let number = version.as_deref().and_then(|v| v.split_whitespace().nth(1)).and_then(|n| {
        let (major, minor) = n.split_once('.')?;
        Some((major.parse::<u32>().ok()?, minor.parse::<u32>().ok()?))
    });
    match number {
        Some(n) if n >= (5, 50) => {}
        _ => {
            eprintln!("verilator 5.050 or later not found; skipped");
            return;
        }
    }
    let dir = tmp("dpi-verilator");
    let lib = build_lib(&dir).with_extension(EXT);
    let defines = ["-DTWO_STATE", "-DNO_OPEN_PACKED", "-DNO_STRUCT_ARGS", "-DNO_DYNAMIC_OPEN", "-DNO_TIMED_EXPORTS"];
    let obj = dir.join("obj");
    let _ = std::fs::remove_dir_all(&obj);
    let ldflags = format!("{}{}", lib.display(), if cfg!(target_os = "linux") { " -rdynamic" } else { "" });
    let r = Command::new("verilator")
        .args(["--binary", "--timing", "-Wno-fatal", "-Wno-lint", "-Wno-style", "--top-module", "top", "-Mdir"])
        .arg(&obj)
        .args(defines)
        .arg("-LDFLAGS")
        .arg(&ldflags)
        .arg(here("tests/dpi/design.sv"))
        .output()
        .unwrap();
    assert!(r.status.success(), "verilator failed:\n{}{}", String::from_utf8_lossy(&r.stdout), String::from_utf8_lossy(&r.stderr));
    let r = Command::new(obj.join("Vtop")).output().unwrap();
    let theirs = String::from_utf8_lossy(&r.stdout).into_owned();
    let r = Command::new(env!("CARGO_BIN_EXE_epitome-sv")).args(defines).arg(here("tests/dpi/design.sv")).arg("-sv_lib").arg(&lib).output().unwrap();
    let ours = String::from_utf8_lossy(&r.stdout).into_owned();
    // Verilator's report and warnings aside.
    let theirs: Vec<&str> = theirs.lines().filter(|l| !l.starts_with("- ") && !l.starts_with('%')).collect();
    let ours: Vec<&str> = ours.lines().collect();
    assert_eq!(theirs.len(), ours.len(), "Verilator:\n{}\nepitome-sv:\n{}", theirs.join("\n"), ours.join("\n"));
    let differ = ["reals ", "weigh ", "outs ", "open fill ", "selects ", "time "];
    for (a, b) in theirs.iter().zip(&ours) {
        if a != b && !differ.iter().any(|d| b.starts_with(d)) {
            panic!("Verilator printed\n  {a}\nepitome-sv\n  {b}");
        }
    }
}

/// What the design prints. Times are in the design's precision (ps).
const EXPECTED: &str = "\
bytes -5 -128 0
ints 300 39998 7
long -9000000000 unsigned 1
reals 2.750 4.500
scalars 1 0 1 x z
string hello, dpi 12345
chandle 12 null 0
enum 7
weigh 356.50
package 42
split 7 -2 swapped -2.00 1.25
outs -7 30000 1 x 0.75 200
out string from C
inout string QUIET!
vec wide a5a5a5a5000000c500005000 l70 01xz10_89abcdef01234567
popcount 20
integer -15 time 1234567890124
packed struct 1436 q a
rec sum 23.50
rec made 9 0.125 -1 abdeadbeef made
arr sum 174
arr rev 33 23 11
open sum 30010
open fill a002 a00X a004 a00X
open dynamic 400100 a000 a00X
open packed 1205 7007
open 2d dims 2 [2:5] low 0 high 1 inc -1 size 4 packed 4
open 2d elems 7 10 9 1
open scalars 10zx
open recs 444.25
selects Z000fffd_Z000000X 00000017 00000001 fffffff0 0000000c 0000fffd 80000000
version 1800-2005
scope top
cells 1051 2062 1
ask 1: 1107
recall 1: 11
ask 2: 2208
recall 2: 22
time 0 -9 -12 6000 6.0
trace run_cell 110 at 14000
trace run_cell 213 at 20000
run_cell at 20000: 14 20
trace nested 3 at 20000
trace nested 2 at 21000
trace nested 1 at 22000
trace nested 0 at 23000
nested at 24000:'{3, 2, 1, 0}
plain task 42
disabled at 30000: 11
done
";
