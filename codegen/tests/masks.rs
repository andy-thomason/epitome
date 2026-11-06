//! Vector comparison masks as integers (`bitcast <N x i1> to iN`), at
//! widths that fit a register and ones that must be split, built by us,
//! linked with a C main, run, and checked against the bits worked out here.
//! On an AArch64 Mac with clang only.

use std::process::Command;

use epitome_codegen::target::Target;
use epitome_codegen::{Options, compile};

/// This machine's target, and its triple.
fn host_target() -> Target {
    if cfg!(target_arch = "x86_64") { Target::x86_64() } else { Target::aarch64() }
}

const TRIPLE: &str = if cfg!(target_arch = "x86_64") { "x86_64-unknown-linux-gnu" } else { "arm64-apple-macosx14.0.0" };

#[test]
fn comparison_masks_as_integers() {
    let ok = (cfg!(target_arch = "aarch64") && cfg!(target_os = "macos")
        || cfg!(target_arch = "x86_64") && cfg!(target_os = "linux"))
        && Command::new("clang").arg("--version").output().is_ok_and(|o| o.status.success());
    if !ok {
        eprintln!("needs clang on an AArch64 Mac or x86-64 Linux; skipping");
        return;
    }
    let tmp = std::path::PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("masks");
    std::fs::create_dir_all(&tmp).unwrap();
    // mask_N(a, b): bit k set where a[k] == b[k], for N bytes.
    let mut ll = format!("target triple = \"{TRIPLE}\"\n");
    for n in [8, 16, 32, 64] {
        // pick_N(bits, a, b, out): out[k] = bit k ? a[k] : b[k].
        ll.push_str(&format!(
            "define void @pick_{n}(i{n} %bits, ptr %a, ptr %b, ptr %out) {{\n  %m = bitcast i{n} %bits to <{n} x i1>\n  \
             %x = load <{n} x i8>, ptr %a\n  %y = load <{n} x i8>, ptr %b\n  \
             %s = select <{n} x i1> %m, <{n} x i8> %x, <{n} x i8> %y\n  store <{n} x i8> %s, ptr %out\n  ret void\n}}\n"
        ));
        ll.push_str(&format!(
            "define i{n} @mask_{n}(ptr %a, ptr %b) {{\n  %x = load <{n} x i8>, ptr %a\n  %y = load <{n} x i8>, ptr %b\n  \
             %c = icmp eq <{n} x i8> %x, %y\n  %m = bitcast <{n} x i1> %c to i{n}\n  ret i{n} %m\n}}\n"
        ));
    }
    let t = host_target();
    let mut m = epitome_ir::parse_module(&ll).unwrap();
    let compiled = compile(&t, &mut m, &Options::default()).unwrap();
    std::fs::write(tmp.join("masks.o"), compiled.object_file(&t).unwrap()).unwrap();
    let main = r#"
#include <stdio.h>
#include <stdint.h>
uint8_t mask_8(const char *, const char *);
uint16_t mask_16(const char *, const char *);
uint32_t mask_32(const char *, const char *);
uint64_t mask_64(const char *, const char *);
void pick_8(uint8_t, const char *, const char *, char *);
void pick_16(uint16_t, const char *, const char *, char *);
void pick_32(uint32_t, const char *, const char *, char *);
void pick_64(uint64_t, const char *, const char *, char *);
int main(void) {
    char a[64], b[64];
    for (int i = 0; i < 64; i++) { a[i] = (char)i; b[i] = (char)((i * 7) % 5 == 0 ? i : i + 1); }
    printf("%llx %llx %llx %llx\n", (unsigned long long)mask_8(a, b), (unsigned long long)mask_16(a, b),
           (unsigned long long)mask_32(a, b), (unsigned long long)mask_64(a, b));
    char out[64];
    uint64_t pattern = 0x9a3c5f0e17d2b648ull;
    pick_8((uint8_t)pattern, a, b, out);
    for (int i = 0; i < 8; i++) printf("%d,", out[i]);
    pick_16((uint16_t)pattern, a, b, out);
    for (int i = 0; i < 16; i++) printf("%d,", out[i]);
    pick_32((uint32_t)pattern, a, b, out);
    for (int i = 0; i < 32; i++) printf("%d,", out[i]);
    pick_64(pattern, a, b, out);
    for (int i = 0; i < 64; i++) printf("%d,", out[i]);
    printf("\n");
    return 0;
}
"#;
    std::fs::write(tmp.join("main.c"), main).unwrap();
    let exe = tmp.join("masks");
    let built =
        Command::new("clang").arg(tmp.join("main.c")).arg(tmp.join("masks.o")).arg("-o").arg(&exe).output().unwrap();
    assert!(built.status.success(), "{}", String::from_utf8_lossy(&built.stderr));
    let out = Command::new(&exe).output().unwrap();
    let got = String::from_utf8_lossy(&out.stdout).trim().to_string();
    let bits = |n: u32| -> u64 { (0..n).filter(|i| (i * 7) % 5 == 0).fold(0u64, |m, i| m | 1 << i) };
    let a = |i: u64| i as i64;
    let b = |i: u64| if (i * 7) % 5 == 0 { i as i64 } else { i as i64 + 1 };
    let pattern: u64 = 0x9a3c5f0e17d2b648;
    let mut picks = String::new();
    for n in [8u64, 16, 32, 64] {
        for i in 0..n {
            picks.push_str(&format!("{},", if pattern >> i & 1 == 1 { a(i) } else { b(i) }));
        }
    }
    let want = format!("{:x} {:x} {:x} {:x}\n{picks}", bits(8), bits(16), bits(32), bits(64));
    assert_eq!(got, want);
}

/// Masks split across registers, up to 128 lanes (`prepare::wide_mask_bits`):
/// selects, reductions, bitcasts to and from `i128` and `i64`, loads and
/// stores of them, built by us and by clang from `tests/masks/wide.ll`, each
/// linked with `wide_main.c`: the two must print the same.
#[test]
fn wide_masks_as_clangs() {
    let ok = (cfg!(target_arch = "aarch64") && cfg!(target_os = "macos")
        || cfg!(target_arch = "x86_64") && cfg!(target_os = "linux"))
        && Command::new("clang").arg("--version").output().is_ok_and(|o| o.status.success());
    if !ok {
        eprintln!("needs clang on an AArch64 Mac or x86-64 Linux; skipping");
        return;
    }
    let dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/masks");
    let tmp = std::path::PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("masks-wide");
    std::fs::create_dir_all(&tmp).unwrap();
    // (For this machine's triple.)
    let text = std::fs::read_to_string(dir.join("wide.ll"))
        .unwrap()
        .replace("target triple = \"arm64-apple-macosx14.0.0\"", &format!("target triple = \"{TRIPLE}\""));
    let ll = tmp.join("wide.ll");
    std::fs::write(&ll, &text).unwrap();
    let t = host_target();
    let mut m = epitome_ir::parse_module(&text).unwrap();
    let ours = tmp.join("ours.o");
    std::fs::write(&ours, compile(&t, &mut m, &Options::default()).unwrap().object_file(&t).unwrap()).unwrap();
    let theirs = tmp.join("theirs.o");
    let st = Command::new("clang").arg("-O1").arg("-c").arg(&ll).arg("-o").arg(&theirs).status().unwrap();
    assert!(st.success());
    let mut out = Vec::new();
    for (name, obj) in [("ours", &ours), ("theirs", &theirs)] {
        let exe = tmp.join(name);
        let st = Command::new("clang").arg(obj).arg(dir.join("wide_main.c")).arg("-o").arg(&exe).status().unwrap();
        assert!(st.success());
        let o = Command::new(&exe).output().unwrap();
        assert!(o.status.success());
        out.push(String::from_utf8_lossy(&o.stdout).into_owned());
    }
    assert_eq!(out[0], out[1], "our build of wide.ll differs from clang's");
}

/// Logic on masks of 256-bit comparisons with AVX2: eight lanes of 32-bit
/// comparisons fill a ymm register, so an `or` of two of them is `vpor` on
/// ymm, not `por` on the low halves. On x86-64 Linux with clang only.
#[test]
fn logic_on_masks_of_256_bit_comparisons() {
    let ok = cfg!(target_arch = "x86_64")
        && cfg!(target_os = "linux")
        && Command::new("clang").arg("--version").output().is_ok_and(|o| o.status.success());
    if !ok {
        eprintln!("needs clang on x86-64 Linux; skipping");
        return;
    }
    let tmp = std::path::PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("masks-ymm");
    std::fs::create_dir_all(&tmp).unwrap();
    let mut ll = format!("target triple = \"{TRIPLE}\"\n");
    for op in ["and", "or", "xor"] {
        // logic(a, b, out): out[k] = (a[k] <u b[k]) op (a[k] == b[k]).
        ll.push_str(&format!(
            "define void @{op}8(ptr %a, ptr %b, ptr %out) {{\n  %x = load <8 x i32>, ptr %a, align 32\n  \
             %y = load <8 x i32>, ptr %b, align 32\n  %c = icmp ult <8 x i32> %x, %y\n  %d = icmp eq <8 x i32> %x, %y\n  \
             %e = {op} <8 x i1> %c, %d\n  %z = zext <8 x i1> %e to <8 x i32>\n  store <8 x i32> %z, ptr %out, align 32\n  ret void\n}}\n"
        ));
    }
    let t = host_target();
    let mut m = epitome_ir::parse_module(&ll).unwrap();
    let opts = Options { mattr: vec!["+avx2".into()], ..Default::default() };
    let compiled = compile(&t, &mut m, &opts).unwrap();
    assert!(!compiled.asm.contains("por %xmm"), "{}", compiled.asm);
    std::fs::write(tmp.join("logic.o"), compiled.object_file(&t).unwrap()).unwrap();
    let main = r#"
#include <stdio.h>
#include <stdint.h>
void and8(const uint32_t *, const uint32_t *, uint32_t *);
void or8(const uint32_t *, const uint32_t *, uint32_t *);
void xor8(const uint32_t *, const uint32_t *, uint32_t *);
int main(void) {
    _Alignas(32) uint32_t a[8] = {0, 1, 5, 0xffffffff, 7, 0x80000000, 3, 9};
    _Alignas(32) uint32_t b[8] = {1, 1, 4, 0, 7, 0x7fffffff, 0xffffffff, 9};
    _Alignas(32) uint32_t o[8];
    void (*f[3])(const uint32_t *, const uint32_t *, uint32_t *) = {and8, or8, xor8};
    for (int k = 0; k < 3; k++) {
        f[k](a, b, o);
        for (int i = 0; i < 8; i++) printf("%u", o[i]);
        printf("\n");
    }
    return 0;
}
"#;
    std::fs::write(tmp.join("main.c"), main).unwrap();
    let exe = tmp.join("logic");
    let built = Command::new("clang").arg(tmp.join("main.c")).arg(tmp.join("logic.o")).arg("-o").arg(&exe).output().unwrap();
    assert!(built.status.success(), "{}", String::from_utf8_lossy(&built.stderr));
    let got = String::from_utf8_lossy(&Command::new(&exe).output().unwrap().stdout).into_owned();
    let (a, b) = ([0u32, 1, 5, 0xffffffff, 7, 0x80000000, 3, 9], [1u32, 1, 4, 0, 7, 0x7fffffff, 0xffffffff, 9]);
    let mut want = String::new();
    for f in [|x: bool, y: bool| x & y, |x, y| x | y, |x, y| x ^ y] {
        for i in 0..8 {
            want.push_str(if f(a[i] < b[i], a[i] == b[i]) { "1" } else { "0" });
        }
        want.push('\n');
    }
    assert_eq!(got, want);
}
