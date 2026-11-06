//! AVX-512's 512-bit integer vectors, with masks in k registers:
//! `tests/avx512/kernels.ll` (arithmetic, shifts, every comparison, selects,
//! masks as integers and in memory, minimum and maximum, leading and
//! trailing zeros, over `<16 x i32>`, `<8 x i64>`, `<32 x i16>` and
//! `<64 x i8>`).
//!
//! - Anywhere: each kernel compiles to code that never touches the stack,
//!   so none of it was done a lane at a time (which goes through stack
//!   slots), and its masks are in k registers.
//! - On x86-64 Linux with AVX-512 and clang: built by us and by clang,
//!   each linked with `main.c` and run, the two print the same.

use std::process::Command;

use epitome_codegen::target::Target;
use epitome_codegen::{Options, compile};

fn kernels() -> String {
    let dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/avx512");
    std::fs::read_to_string(dir.join("kernels.ll")).unwrap()
}

/// Each function's assembly, by name.
fn functions(asm: &str) -> Vec<(String, String)> {
    let mut out: Vec<(String, String)> = Vec::new();
    for line in asm.lines() {
        if let Some(name) = line.strip_suffix(':').filter(|n| !n.starts_with('.') && !n.contains(' ')) {
            out.push((name.to_string(), String::new()));
        } else if let Some((_, body)) = out.last_mut() {
            body.push_str(line);
            body.push('\n');
        }
    }
    out
}

#[test]
fn avx512_kernels_stay_in_registers() {
    let t = Target::x86_64();
    let mut m = epitome_ir::parse_module(&kernels()).unwrap();
    let asm = compile(&t, &mut m, &Options::default()).unwrap().asm;
    let fs = functions(&asm);
    assert!(fs.len() >= 25, "{} functions", fs.len());
    for (name, body) in &fs {
        assert!(!body.contains("%rsp"), "{name} uses the stack:\n{body}");
        assert!(body.contains("%zmm"), "{name} has no zmm registers:\n{body}");
        if name.starts_with("compares") || name.starts_with("selects") || name.starts_with("mask_memory") {
            assert!(body.contains("%k"), "{name} has no k registers:\n{body}");
        }
    }
}

/// Does this machine have the AVX-512 the kernels need?
fn has_avx512() -> bool {
    let Ok(info) = std::fs::read_to_string("/proc/cpuinfo") else { return false };
    let Some(flags) = info.lines().find(|l| l.starts_with("flags")) else { return false };
    let flags: Vec<&str> = flags.split_whitespace().collect();
    ["avx512f", "avx512bw", "avx512cd", "avx512dq", "avx512vl"].iter().all(|f| flags.contains(f))
}

#[test]
fn avx512_kernels_as_clangs() {
    let ok = cfg!(target_arch = "x86_64")
        && cfg!(target_os = "linux")
        && has_avx512()
        && Command::new("clang").arg("--version").output().is_ok_and(|o| o.status.success());
    if !ok {
        eprintln!("needs clang on x86-64 Linux with AVX-512; skipping");
        return;
    }
    let dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/avx512");
    let tmp = std::path::PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("avx512");
    std::fs::create_dir_all(&tmp).unwrap();
    let t = Target::x86_64();
    let mut m = epitome_ir::parse_module(&kernels()).unwrap();
    let ours = tmp.join("ours.o");
    std::fs::write(&ours, compile(&t, &mut m, &Options::default()).unwrap().object_file(&t).unwrap()).unwrap();
    let theirs = tmp.join("theirs.o");
    let st =
        Command::new("clang").arg("-O1").arg("-c").arg(dir.join("kernels.ll")).arg("-o").arg(&theirs).status().unwrap();
    assert!(st.success());
    let mut out = Vec::new();
    for (name, obj) in [("ours", &ours), ("theirs", &theirs)] {
        let exe = tmp.join(name);
        let st =
            Command::new("clang").arg("-O1").arg(obj).arg(dir.join("main.c")).arg("-o").arg(&exe).status().unwrap();
        assert!(st.success());
        let o = Command::new(&exe).output().unwrap();
        assert!(o.status.success(), "{name}: {}", String::from_utf8_lossy(&o.stderr));
        out.push(String::from_utf8_lossy(&o.stdout).into_owned());
    }
    assert_eq!(out[0].lines().count(), out[1].lines().count());
    let wrong: Vec<String> = out[0]
        .lines()
        .zip(out[1].lines())
        .filter(|(a, b)| a != b)
        .map(|(a, b)| format!("ours:   {a}\ntheirs: {b}"))
        .collect();
    assert!(wrong.is_empty(), "{} kernels differ from clang's:\n{}", wrong.len(), wrong.join("\n"));
}
