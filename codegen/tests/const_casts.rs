//! Truncations of constant vectors of a type split across registers
//! (`trunc <16 x i32> splat (i32 0) to <16 x i8>`, which epitome-sv's
//! netlist writes unoptimised): the constant is split into parts as a
//! computed vector is. Built by us, linked with a C main, run, and checked
//! against the values worked out here. On an AArch64 Mac or x86-64 Linux
//! with clang only.

use std::process::Command;

use epitome_codegen::target::Target;
use epitome_codegen::{Options, compile};

const TRIPLE: &str = if cfg!(target_arch = "x86_64") { "x86_64-unknown-linux-gnu" } else { "arm64-apple-macosx14.0.0" };

/// The constants, as IR, and their lanes.
fn lanes() -> Vec<(String, Vec<u64>)> {
    let ramp: Vec<u64> = (0..16).map(|i| 0x1234_5600 + i * 37 + (i << 20)).collect();
    let text = |v: &[u64], ty: &str| format!("<{}>", v.iter().map(|x| format!("{ty} {}", *x as i64)).collect::<Vec<_>>().join(", "));
    vec![
        ("splat (i32 0)".to_string(), vec![0; 16]),
        ("splat (i32 -1234567)".to_string(), vec![(-1234567i32) as u32 as u64; 16]),
        (text(&ramp, "i32"), ramp),
    ]
}

#[test]
fn truncations_of_split_constants() {
    let ok = (cfg!(target_arch = "aarch64") && cfg!(target_os = "macos")
        || cfg!(target_arch = "x86_64") && cfg!(target_os = "linux"))
        && Command::new("clang").arg("--version").output().is_ok_and(|o| o.status.success());
    if !ok {
        eprintln!("needs clang on an AArch64 Mac or x86-64 Linux; skipping");
        return;
    }
    let tmp = std::path::PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("const-casts");
    std::fs::create_dir_all(&tmp).unwrap();
    let mut ll = format!("target triple = \"{TRIPLE}\"\n");
    let mut want = String::new();
    let mut calls = String::new();
    for (k, (c, v)) in lanes().iter().enumerate() {
        for (bits, mask) in [(8, 0xff), (16, 0xffff)] {
            ll.push_str(&format!(
                "define void @t{k}_{bits}(ptr %out) {{\n  %x = trunc <16 x i32> {c} to <16 x i{bits}>\n  \
                 store <16 x i{bits}> %x, ptr %out, align 1\n  ret void\n}}\n"
            ));
            calls.push_str(&format!(
                "    {{ uint{bits}_t o[16]; t{k}_{bits}(o); for (int i = 0; i < 16; i++) printf(\"%llx \", (unsigned long long)o[i]); printf(\"\\n\"); }}\n"
            ));
            want.push_str(&v.iter().map(|x| format!("{:x} ", x & mask)).collect::<String>());
            want.push('\n');
        }
    }
    let decls: String = (0..lanes().len()).map(|k| format!("void t{k}_8(uint8_t *);\nvoid t{k}_16(uint16_t *);\n")).collect();
    let main = format!("#include <stdio.h>\n#include <stdint.h>\n{decls}int main(void) {{\n{calls}    return 0;\n}}\n");
    std::fs::write(tmp.join("main.c"), main).unwrap();
    // The host's vectors, and on x86-64 AVX2's too (two ymm parts).
    let mut mattrs = vec![Vec::new()];
    if cfg!(target_arch = "x86_64") {
        mattrs.push(vec!["+avx2".to_string()]);
    }
    for mattr in mattrs {
        let t = if cfg!(target_arch = "x86_64") { Target::x86_64() } else { Target::aarch64() };
        let mut m = epitome_ir::parse_module(&ll).unwrap();
        let opts = Options { mattr: mattr.clone(), ..Default::default() };
        let c = compile(&t, &mut m, &opts).unwrap_or_else(|e| panic!("{mattr:?}: {e}"));
        let obj = tmp.join("casts.o");
        std::fs::write(&obj, c.object_file(&t).unwrap()).unwrap();
        let exe = tmp.join("casts");
        let st = Command::new("clang").arg(&obj).arg(tmp.join("main.c")).arg("-o").arg(&exe).status().unwrap();
        assert!(st.success());
        let out = Command::new(&exe).output().unwrap();
        assert!(out.status.success());
        assert_eq!(String::from_utf8_lossy(&out.stdout), want, "{mattr:?}");
    }
}
