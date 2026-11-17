//! VPI (`src/sim/vpi.rs`) through the launcher pair, as a test runner uses
//! them: `tests/vpi/app.rs`, a VPI application, is built as a shared
//! library and loaded by `epitome-vvp` into a simulation of
//! `tests/vpi/design.sv` that `epitome-iverilog` set up. The application
//! walks the design, reads and writes values, uses every kind of callback
//! and registers a system task and function; what it prints is compared
//! with what it should print.

use std::path::{Path, PathBuf};
use std::process::Command;

fn here(p: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(p)
}

/// Builds the application with the compiler that built this test.
fn build_app(dir: &Path) -> PathBuf {
    let out = dir.join("app.vpi");
    let rustc = std::env::var("RUSTC").unwrap_or_else(|_| "rustc".into());
    let mut c = Command::new(rustc);
    c.args(["--edition", "2024", "-O", "--crate-type", "cdylib", "-o"]).arg(&out).arg(here("tests/vpi/app.rs"));
    // Its VPI functions are found in the executable that loads it.
    if cfg!(target_os = "macos") {
        c.args(["-C", "link-arg=-undefined", "-C", "link-arg=dynamic_lookup"]);
    }
    let r = c.output().expect("run rustc");
    assert!(r.status.success(), "building the VPI application failed:\n{}", String::from_utf8_lossy(&r.stderr));
    out
}

#[test]
fn application() {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("vpi-application");
    std::fs::create_dir_all(&dir).unwrap();
    let app = build_app(&dir);
    assert!(app.is_file());
    let design = dir.join("design.evvp");
    let r = Command::new(env!("CARGO_BIN_EXE_epitome-iverilog"))
        .args(["-g2012", "-o"])
        .arg(&design)
        .arg(here("tests/vpi/design.sv"))
        .output()
        .unwrap();
    assert!(r.status.success(), "{}", String::from_utf8_lossy(&r.stderr));
    let r = Command::new(env!("CARGO_BIN_EXE_epitome-vvp"))
        .arg("-M")
        .arg(&dir)
        .args(["-m", "app"])
        .arg(&design)
        .arg("+app_flag")
        .env("APP_ERRORS", "1")
        .output()
        .unwrap();
    let out = String::from_utf8_lossy(&r.stdout);
    assert!(r.status.success(), "{out}{}", String::from_utf8_lossy(&r.stderr));
    if out != EXPECTED {
        for (i, (a, b)) in out.lines().zip(EXPECTED.lines()).enumerate() {
            if a != b {
                panic!("line {}: got\n  {a}\nexpected\n  {b}\nwhole output:\n{out}", i + 1);
            }
        }
        panic!("got {} lines, expected {}:\n{out}", out.lines().count(), EXPECTED.lines().count());
    }
}

/// What the application prints. Times are in the design's precision (ps);
/// the clock, which the application drives, has a 10ns period.
const EXPECTED: &str = "\
app: start; +app_flag given: true
app: precision -12
app: top top (top), unit -9
app: net: inv:8
app: reg: clk:1 data:8 nib:4 q:8
app: int: edges:32
app: real: ratio:1
app: string: label:5
app: bit: pairs:8
app: array: mem:4
app: parameter: NAME:24 R:1 W:32
app: module: u:-1
app: scope: g[0]:-1 g[1]:-1 u:-1
app: W=8 NAME=top R=1.5 u.N=3
app: top.g[1].v in top.g[1] (134), type 48
app: top.u.q is a 48 output of 8 bits, in u
app: nib=10zx [3:0] vector=1
app: nib[1] scalar=2 nib[3]=1
app: pairs[2]=10 pairs[2][1]=1
app: mem has 4 words, [0:3], type 116
app: label=hello ratio=0
app: removed: 1
app: 0ps $app_show start 0 8
app: 5000ps q=03 at 5ns
app: 5000ps edges=1
app: 7000ps wrote data; it reads 00
app: 7000ps read-write: data=41 inv=be
app: 7000ps read-only: data=41 inv=be
app: 12000ps mem[1]=99 forced inv=55
app: 15000ps next time: data=43 inv=55
app: 15000ps q=46 at 15ns
app: 15000ps edges=2
app: 18000ps released inv, now bc
design: 23000 mem[1]=99 inv=bd q=46 twice(21)=42
app: 25000ps q=45 at 25ns
app: 25000ps edges=3
app: 30000ps label is bye, ratio 0.25
app: no handle: true, error level after a good call 0
app: a freed handle: error level 3: vpi_get_value: bad handle
app: 35000ps edges=4
app: 40000ps finishing
app: 40000ps end
";
