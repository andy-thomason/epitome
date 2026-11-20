//! Spill code for an instruction that reads three vector registers, all of
//! them spilled (a blend with AVX2, where each mask of 16 lanes takes two
//! ymm registers): x86 keeps three vector scratch registers for it
//! (`tests/x86_spill/three_operands.ll`). On x86-64 Linux with clang, our
//! build is run against clang's.

use std::process::Command;

use epitome_codegen::target::Target;
use epitome_codegen::{Options, compile};

#[test]
fn three_spilled_vector_operands() {
    let dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/x86_spill");
    let ll = std::fs::read_to_string(dir.join("three_operands.ll")).unwrap();
    let t = Target::x86_64();
    let mut m = epitome_ir::parse_module(&ll).unwrap();
    let opts = Options { mattr: vec!["+avx2".into()], ..Default::default() };
    let c = compile(&t, &mut m, &opts).unwrap_or_else(|e| panic!("{e}"));
    let clang = Command::new("clang").arg("--version").output().is_ok_and(|o| o.status.success());
    if !(cfg!(target_arch = "x86_64") && cfg!(target_os = "linux") && clang) {
        eprintln!("needs clang on x86-64 Linux to run; compiled only");
        return;
    }
    let tmp = std::path::PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("x86-spill");
    std::fs::create_dir_all(&tmp).unwrap();
    let ours = tmp.join("ours.o");
    std::fs::write(&ours, c.object_file(&t).unwrap()).unwrap();
    let theirs = tmp.join("theirs.o");
    let st = Command::new("clang").args(["-O1", "-mavx2", "-c"]).arg(dir.join("three_operands.ll")).arg("-o").arg(&theirs).status().unwrap();
    assert!(st.success());
    let mut out = Vec::new();
    for (name, obj) in [("ours", &ours), ("theirs", &theirs)] {
        let exe = tmp.join(name);
        let st = Command::new("clang").arg(obj).arg(dir.join("main.c")).arg("-o").arg(&exe).status().unwrap();
        assert!(st.success());
        let o = Command::new(&exe).output().unwrap();
        assert!(o.status.success());
        out.push(String::from_utf8_lossy(&o.stdout).into_owned());
    }
    assert_eq!(out[0], out[1], "our build of three_operands.ll differs from clang's");
}

/// A compare the same as one before it in its block goes
/// (`peephole::redundant_compares`), and so does an address of a slot
/// made once and used across the function (`frame_addresses`); the block
/// must still say where its terminator starts, as the register allocator
/// puts a spilled value's load for leaving the block there. With the
/// terminator's start left one too far, the load went after the jump and
/// never ran, and a loop that read the value never ended. The allocator
/// now refuses a block whose terminator is not where it says.
#[test]
fn removed_compare_keeps_the_terminator_start() {
    let ll = "
define i64 @f(i64 %a, i64 %b) {
entry:
  %c1 = icmp ult i64 %a, %b
  %s1 = select i1 %c1, i64 %a, i64 %b
  %c2 = icmp ult i64 %a, %b
  %s2 = select i1 %c2, i64 1, i64 2
  %r = add i64 %s1, %s2
  br label %next

next:
  ret i64 %r
}
";
    for (t, triple) in [(Target::x86_64(), "x86_64-unknown-linux-gnu"), (Target::aarch64(), "arm64-apple-macosx14.0.0")] {
        let mut m = epitome_ir::parse_module(&format!("target triple = \"{triple}\"\n{ll}")).unwrap();
        compile(&t, &mut m, &Options::default()).unwrap_or_else(|e| panic!("{triple}: {e}"));
    }
}
