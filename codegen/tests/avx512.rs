//! AVX-512's 512-bit vectors, with masks in k registers:
//! `tests/avx512/kernels.ll` (arithmetic, shifts, every comparison, selects,
//! masks as integers and in memory, minimum and maximum, leading and
//! trailing zeros, over `<16 x i32>`, `<8 x i64>`, `<32 x i16>` and
//! `<64 x i8>`; floats' comparisons and selects; and AVX2's shifts by
//! vectors).
//!
//! - Anywhere: each kernel compiles to code that never touches the stack,
//!   so none of it was done a lane at a time (which goes through stack
//!   slots), and its masks are in k registers (but the `novl_` kernels',
//!   for avx512f without avx512vl, whose 256-bit masks stay in ymm).
//! - On x86-64 Linux with AVX-512 and clang: built by us and by clang,
//!   each linked with `main.c` and run, the two print the same.
//! - Chains of bitwise logic (`ternlog` kernels) are each one `vpternlog`,
//!   and each of the 256 tables of `llvm.epitome.bitfn3`, run on 0xf0,
//!   0xcc and 0xaa, gives itself back.

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
    assert!(fs.len() >= 100, "{} functions", fs.len());
    for (name, body) in &fs {
        assert!(!body.contains("%rsp"), "{name} uses the stack:\n{body}");
        assert!(["%zmm", "%ymm", "%xmm"].iter().any(|r| body.contains(r)), "{name} has no vector registers:\n{body}");
        let masks = ["compares", "fsel", "selects", "mask_memory", "mixed"].iter().any(|k| name.contains(k));
        if name.starts_with("ternlog") {
            assert!(body.contains("vpternlog"), "{name} has no vpternlog:\n{body}");
            for op in ["vpand", "vpor", "vpxor"] {
                assert!(!body.contains(op), "{name} still has {op}:\n{body}");
            }
        }
        if name == "ternlog_adder" {
            assert_eq!(body.matches("vpternlog").count(), 2, "the full adder:\n{body}");
        }
        if name.starts_with("novl_") {
            assert!(!body.contains("%k"), "{name} has k registers without avx512vl:\n{body}");
        } else if masks {
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

/// Every table of `llvm.epitome.bitfn3`, at 512, 256 and 128 bits, of
/// doublewords and quadwords, on 0xf0, 0xcc and 0xaa: each must give its
/// table back in every byte.
#[test]
fn bitfn3_tables() {
    let ok = cfg!(target_arch = "x86_64")
        && cfg!(target_os = "linux")
        && has_avx512()
        && Command::new("clang").arg("--version").output().is_ok_and(|o| o.status.success());
    if !ok {
        eprintln!("needs clang on x86-64 Linux with AVX-512; skipping");
        return;
    }
    let tmp = std::path::PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("bitfn3");
    std::fs::create_dir_all(&tmp).unwrap();
    let types = [
        ("v16i32", "<16 x i32>", 64),
        ("v8i64", "<8 x i64>", 64),
        ("v8i32", "<8 x i32>", 32),
        ("v2i64", "<2 x i64>", 16),
    ];
    let mut ll = String::from("target triple = \"x86_64-unknown-linux-gnu\"\n");
    for (name, ty, _) in types {
        ll.push_str(&format!("declare {ty} @llvm.epitome.bitfn3.{name}(i8, {ty}, {ty}, {ty})\n"));
        ll.push_str(&format!("define void @tables_{name}(ptr %pa, ptr %pb, ptr %pc, ptr %o) #0 {{\n"));
        ll.push_str(&format!("  %a = load {ty}, ptr %pa\n  %b = load {ty}, ptr %pb\n  %c = load {ty}, ptr %pc\n"));
        for k in 0..256 {
            ll.push_str(&format!(
                "  %r{k} = call {ty} @llvm.epitome.bitfn3.{name}(i8 {}, {ty} %a, {ty} %b, {ty} %c)\n  \
                 %p{k} = getelementptr {ty}, ptr %o, i64 {k}\n  store {ty} %r{k}, ptr %p{k}\n",
                k as u8 as i8
            ));
        }
        ll.push_str("  ret void\n}\n");
    }
    ll.push_str("attributes #0 = { nounwind \"target-features\"=\"+avx512f,+avx512vl\" }\n");
    let t = Target::x86_64();
    let mut m = epitome_ir::parse_module(&ll).unwrap();
    std::fs::write(tmp.join("tables.o"), compile(&t, &mut m, &Options::default()).unwrap().object_file(&t).unwrap())
        .unwrap();
    let mut main = String::from("#include <stdio.h>\n#include <string.h>\n");
    for (name, _, _) in types {
        main.push_str(&format!("void tables_{name}(const void *, const void *, const void *, void *);\n"));
    }
    main.push_str(
        "static unsigned char a[64] __attribute__((aligned(64))), b[64] __attribute__((aligned(64))), \
         c[64] __attribute__((aligned(64))), out[256 * 64] __attribute__((aligned(64)));\n\
         static int check(const char *name, int size) {\n  int bad = 0;\n  \
         for (int k = 0; k < 256; k++) for (int j = 0; j < size; j++) if (out[k * size + j] != k) { \
         if (!bad) printf(\"%s: table %d gives %d\\n\", name, k, out[k * size + j]); bad = 1; }\n  return bad;\n}\n\
         int main(void) {\n  memset(a, 0xf0, 64); memset(b, 0xcc, 64); memset(c, 0xaa, 64);\n  int bad = 0;\n",
    );
    for (name, _, size) in types {
        main.push_str(&format!(
            "  memset(out, 0x55, sizeof out); tables_{name}(a, b, c, out); bad |= check(\"{name}\", {size});\n"
        ));
    }
    main.push_str("  if (!bad) printf(\"ok\\n\");\n  return bad;\n}\n");
    std::fs::write(tmp.join("main.c"), main).unwrap();
    let exe = tmp.join("tables");
    let st =
        Command::new("clang").arg(tmp.join("main.c")).arg(tmp.join("tables.o")).arg("-o").arg(&exe).status().unwrap();
    assert!(st.success());
    let o = Command::new(&exe).output().unwrap();
    assert_eq!(String::from_utf8_lossy(&o.stdout), "ok\n");
}
