//! 8 and 4 lanes of 32 bits narrowed to bytes and halves with AVX-512VL
//! (`tests/avx512/narrow.ll`): one vpmovdb or vpmovdw each, not lane by
//! lane through the stack. On a machine with AVX-512 and clang, our build
//! is run against clang's.

use std::process::Command;

use epitome_codegen::target::Target;
use epitome_codegen::{Options, compile};

#[test]
fn narrowing_8_and_4_lanes() {
    let dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/avx512");
    let ll = std::fs::read_to_string(dir.join("narrow.ll")).unwrap();
    let t = Target::x86_64();
    let mut m = epitome_ir::parse_module(&ll).unwrap();
    let c = compile(&t, &mut m, &Options::default()).unwrap();
    for i in ["vpmovdb %ymm", "vpmovdw %ymm", "vpmovdw %xmm"] {
        assert!(c.asm.contains(i), "no {i}:\n{}", c.asm);
    }
    assert!(!c.asm.contains("(%rsp)"), "lanes through the stack:\n{}", c.asm);
    let avx512 = std::fs::read_to_string("/proc/cpuinfo").is_ok_and(|s| s.contains(" avx512vl"));
    let clang = Command::new("clang").arg("--version").output().is_ok_and(|o| o.status.success());
    if !(cfg!(target_arch = "x86_64") && cfg!(target_os = "linux") && avx512 && clang) {
        eprintln!("needs clang on x86-64 Linux with AVX-512 to run; checked the assembly only");
        return;
    }
    let tmp = std::path::PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("avx512-narrow");
    std::fs::create_dir_all(&tmp).unwrap();
    let ours = tmp.join("ours.o");
    std::fs::write(&ours, c.object_file(&t).unwrap()).unwrap();
    let theirs = tmp.join("theirs.o");
    let st = Command::new("clang").args(["-O1", "-c"]).arg(dir.join("narrow.ll")).arg("-o").arg(&theirs).status().unwrap();
    assert!(st.success());
    let mut out = Vec::new();
    for (name, obj) in [("ours", &ours), ("theirs", &theirs)] {
        let exe = tmp.join(name);
        let st = Command::new("clang").arg(obj).arg(dir.join("narrow_main.c")).arg("-o").arg(&exe).status().unwrap();
        assert!(st.success());
        let o = Command::new(&exe).output().unwrap();
        assert!(o.status.success());
        out.push(String::from_utf8_lossy(&o.stdout).into_owned());
    }
    assert_eq!(out[0], out[1], "our build of narrow.ll differs from clang's");
}
