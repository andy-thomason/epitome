//! Narrow vectors with AVX-512VL, as a netlist of few lanes has them:
//! 8 and 4 lanes of 32 bits narrowed to bytes and halves
//! (`tests/avx512/narrow.ll`), one vpmovdb or vpmovdw each, not lane by
//! lane through the stack; and products of 64-bit lanes in a ymm and an xmm
//! (`tests/avx512/mul.ll`), one vpmullq each, and of 32-bit lanes in an xmm,
//! one pmulld. On a machine with AVX-512 and clang, our builds are run
//! against clang's.

use std::process::Command;

use epitome_codegen::target::Target;
use epitome_codegen::{Options, compile};

#[test]
fn narrowing_8_and_4_lanes() {
    check("narrow", &["vpmovdb %ymm", "vpmovdw %ymm", "vpmovdw %xmm"]);
}

#[test]
fn products_in_ymm_and_xmm() {
    check("mul", &["vpmullq %ymm", "vpmullq %xmm", "pmulld %xmm"]);
}

/// Compiles `tests/avx512/{name}.ll`, checks its assembly has each of
/// `insns` and no lanes through the stack, and, where it can, runs it
/// against clang's build with `{name}_main.c`.
fn check(name: &str, insns: &[&str]) {
    let dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/avx512");
    let ll = std::fs::read_to_string(dir.join(format!("{name}.ll"))).unwrap();
    let t = Target::x86_64();
    let mut m = epitome_ir::parse_module(&ll).unwrap();
    let c = compile(&t, &mut m, &Options::default()).unwrap();
    for i in insns {
        assert!(c.asm.contains(i), "no {i}:\n{}", c.asm);
    }
    assert!(!c.asm.contains("(%rsp)"), "lanes through the stack:\n{}", c.asm);
    let avx512 = std::fs::read_to_string("/proc/cpuinfo").is_ok_and(|s| s.contains(" avx512vl"));
    let clang = Command::new("clang").arg("--version").output().is_ok_and(|o| o.status.success());
    if !(cfg!(target_arch = "x86_64") && cfg!(target_os = "linux") && avx512 && clang) {
        eprintln!("needs clang on x86-64 Linux with AVX-512 to run; checked the assembly only");
        return;
    }
    let tmp = std::path::PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!("avx512-{name}"));
    std::fs::create_dir_all(&tmp).unwrap();
    let ours = tmp.join("ours.o");
    std::fs::write(&ours, c.object_file(&t).unwrap()).unwrap();
    let theirs = tmp.join("theirs.o");
    let st = Command::new("clang").args(["-O1", "-c"]).arg(dir.join(format!("{name}.ll"))).arg("-o").arg(&theirs).status().unwrap();
    assert!(st.success());
    let mut out = Vec::new();
    for (who, obj) in [("ours", &ours), ("theirs", &theirs)] {
        let exe = tmp.join(who);
        let st = Command::new("clang").arg(obj).arg(dir.join(format!("{name}_main.c"))).arg("-o").arg(&exe).status().unwrap();
        assert!(st.success());
        let o = Command::new(&exe).output().unwrap();
        assert!(o.status.success());
        out.push(String::from_utf8_lossy(&o.stdout).into_owned());
    }
    assert_eq!(out[0], out[1], "our build of {name}.ll differs from clang's");
}
