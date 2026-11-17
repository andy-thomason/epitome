//! The cocotb example (`examples/cocotb`): cocotb's own tests of a FIFO,
//! run by cocotb's Icarus runner on `epitome-iverilog` and `epitome-vvp`.
//! Skipped, with a note, when the Python (`COCOTB_PYTHON`, else `python3`)
//! cannot import cocotb 2.

use std::path::Path;
use std::process::Command;

#[test]
fn fifo() {
    let python = std::env::var("COCOTB_PYTHON").unwrap_or_else(|_| "python3".into());
    let check = Command::new(&python)
        .args(["-c", "import cocotb, cocotb_tools.runner; assert int(cocotb.__version__.split('.')[0]) >= 2"])
        .output();
    if !check.is_ok_and(|o| o.status.success()) {
        eprintln!("skipping: {python} cannot import cocotb 2 (set COCOTB_PYTHON to a Python that can)");
        return;
    }
    let bin = Path::new(env!("CARGO_BIN_EXE_epitome-vvp")).parent().unwrap();
    let build = Path::new(env!("CARGO_TARGET_TMPDIR")).join("cocotb-fifo");
    let script = Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/cocotb/run.py");
    let r = Command::new(&python).arg(script).env("EPITOME_SV_BIN", bin).env("EPITOME_COCOTB_BUILD", &build).output().unwrap();
    let out = String::from_utf8_lossy(&r.stdout);
    let err = String::from_utf8_lossy(&r.stderr);
    assert!(r.status.success(), "the cocotb tests failed:\n{out}\n{err}");
    assert!(out.contains("run.py: 5 tests, 0 failed"), "{out}\n{err}");
}
