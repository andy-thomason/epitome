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
