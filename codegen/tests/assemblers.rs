//! epitome-as and clang's assembler agree on every instruction the
//! AArch64 description has: sample lines of each (`epitome-llc
//! --sample-asm`, several per instruction, over a spread of operands) are
//! assembled by both, and the code must be the same word for word. Most of
//! the description's entries (80-asm.ll) were fitted to clang's encodings,
//! so this keeps them, and the rest, honest.
//!
//! RISC-V's are checked the same way against LLVM's RISC-V assembler, as
//! rustc has it (its `riscv64gc-unknown-none-elf` target, and for Zcmp and
//! Zcmt, `riscv64imac-unknown-none-elf`), compressed instructions included.
//!
//! Each is skipped when its assembler is not installed.

use std::process::Command;

use epitome_codegen::abi::Syntax;
use epitome_codegen::assembler::Assembler;
use epitome_codegen::objdump::Image;
use epitome_codegen::target::Target;

/// What clang must be told the machine has, for every extension the
/// description's instructions need.
const MARCH: &str = "-march=armv9.4-a+crc+crypto+sha3+sm4+rcpc+rcpc3+lse+lse128+rdm+dotprod+fp16+fp16fml+bf16+i8mm\
+f64mm+sme+fp8+fp8fma+fp8dot2+fp8dot4+faminmax+lut+cssc+mops+the+d128+gcs+rasv2+ite+flagm+predres+sb+ssbs+tme+ls64+wfxt\
+hbc+brbe+pauth-lr+fprcvt+lsfe+lsui+occmo+cmpbr+pauth+bti+memtag+rng";

/// The sample lines that differ between our object and theirs, their code
/// split into instructions by `split`.
fn differences(t: &Target, text: &str, ours: &[u8], theirs: &[u8], split: fn(&[u8]) -> Vec<u32>) -> Vec<String> {
    let (a, b) = (code(ours, t, split), code(theirs, t, split));
    let lines: Vec<&str> = text.lines().filter(|l| l.contains(":\t")).collect();
    assert_eq!(a.len(), b.len());
    assert_eq!(a.len(), lines.len());
    a.iter()
        .zip(&b)
        .zip(&lines)
        .filter(|((x, y), _)| x != y)
        .take(20)
        .map(|((x, y), l)| format!("{l}: ours {x:08x}, theirs {y:08x}"))
        .collect()
}

fn code(bytes: &[u8], t: &Target, split: fn(&[u8]) -> Vec<u32>) -> Vec<u32> {
    let img = Image::read(bytes, &t.machine).unwrap();
    img.sections.iter().filter(|s| s.code).flat_map(|s| split(&s.data)).collect()
}

/// Code as words.
fn words(data: &[u8]) -> Vec<u32> {
    data.chunks(4).map(|c| u32::from_le_bytes(c.try_into().unwrap())).collect()
}

/// RISC-V code as its instructions: a word where the low two bits are 11,
/// else a halfword.
fn riscv_instructions(data: &[u8]) -> Vec<u32> {
    let mut out = Vec::new();
    let mut at = 0;
    while at + 2 <= data.len() {
        let half = u16::from_le_bytes([data[at], data[at + 1]]) as u32;
        if half & 3 == 3 && at + 4 <= data.len() {
            out.push(half | (u16::from_le_bytes([data[at + 2], data[at + 3]]) as u32) << 16);
            at += 4;
        } else {
            out.push(half);
            at += 2;
        }
    }
    out
}

#[test]
fn epitome_as_and_clang_agree() {
    let ok = Command::new("clang").arg("--version").output().is_ok_and(|o| o.status.success());
    if !ok {
        eprintln!("clang not found; skipping");
        return;
    }
    let t = Target::aarch64();
    let text = epitome_codegen::check::sample_assembly(&t, Syntax::Elf, 3);
    let tmp = std::path::PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("assemblers");
    std::fs::create_dir_all(&tmp).unwrap();
    let s = tmp.join("aarch64.s");
    std::fs::write(&s, &text).unwrap();
    let ours = Assembler::new(&t).assemble(&text, Some(Syntax::Elf)).and_then(|a| a.file(&t)).unwrap();
    let o = tmp.join("aarch64.clang.o");
    let out = Command::new("clang")
        .args(["--target=aarch64-linux-gnu", MARCH, "-ferror-limit=0", "-c"])
        .arg(&s)
        .arg("-o")
        .arg(&o)
        .output()
        .unwrap();
    assert!(out.status.success(), "clang:\n{}", String::from_utf8_lossy(&out.stderr));
    let wrong = differences(&t, &text, &ours, &std::fs::read(&o).unwrap(), words);
    assert!(wrong.is_empty(), "these differ:\n{}", wrong.join("\n"));
}

#[test]
fn epitome_as_and_llvm_agree_on_riscv() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let t = Target::load_path(root.join("targets/riscv64").to_str().unwrap()).unwrap();
    let text = epitome_codegen::check::sample_assembly(&t, Syntax::Elf, 3);
    let tmp = std::path::PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("assemblers");
    std::fs::create_dir_all(&tmp).unwrap();
    let head: Vec<&str> = text.lines().filter(|l| !l.contains(":\t")).collect();
    let mnemonic = |l: &str| l.split(":\t").nth(1).unwrap_or("").split_whitespace().next().unwrap_or("").to_string();
    // The sample lines in groups, each assembled for its machine: the
    // 32-bit instructions, the compressed ones, and Zcmp's and Zcmt's, which
    // take the encodings of C's double-precision loads and stores, so are
    // for a machine without D. rustc assembles them as a crate's global
    // assembly, without relaxing branches, and compressing instructions only
    // where they are written compressed.
    let groups: [(&str, &str, &str, &str, fn(&str) -> bool); 3] = [
        ("riscv64", "riscv64gc-unknown-none-elf", RISCV_FEATURES, "norvc", |m| {
            !m.starts_with("c.") && !m.starts_with("cm.")
        }),
        ("riscv64c", "riscv64gc-unknown-none-elf", RISCV_C_FEATURES, "rvc", |m| m.starts_with("c.")),
        ("riscv64zcmp", "riscv64imac-unknown-none-elf", "+zcb,+zcmp,+zcmt", "rvc", |m| m.starts_with("cm.")),
    ];
    for (name, target, features, rvc, pick) in groups {
        let lines: Vec<&str> = text.lines().filter(|l| l.contains(":\t") && pick(&mnemonic(l))).collect();
        assert!(!lines.is_empty(), "no {name} lines");
        let body = format!("{}\n{}\n", head.join("\n"), lines.join("\n"));
        std::fs::write(tmp.join(format!("{name}.s")), format!(".option {rvc}\n.option norelax\n{body}")).unwrap();
        std::fs::write(
            tmp.join(format!("{name}.rs")),
            format!("#![no_std]\ncore::arch::global_asm!(include_str!(\"{name}.s\"), options(raw));\n"),
        )
        .unwrap();
        let o = tmp.join(format!("{name}.o"));
        let out = Command::new("rustc")
            .args(["--target", target, "--crate-type", "lib", "--emit", "obj", "-C"])
            .arg(format!("target-feature={features}"))
            .arg("-o")
            .arg(&o)
            .arg(tmp.join(format!("{name}.rs")))
            .output()
            .unwrap();
        let err = String::from_utf8_lossy(&out.stderr);
        if err.contains("can't find crate for `core`") {
            eprintln!("rustc has no {target} target; skipping {name}");
            continue;
        }
        assert!(out.status.success(), "rustc:\n{err}");
        let ours = Assembler::new(&t).assemble(&body, Some(Syntax::Elf)).and_then(|a| a.file(&t)).unwrap();
        let wrong = differences(&t, &body, &ours, &std::fs::read(&o).unwrap(), riscv_instructions);
        assert!(wrong.is_empty(), "these differ ({name}):\n{}", wrong.join("\n"));
    }
}
/// The extensions the RISC-V description's instructions need.
const RISCV_FEATURES: &str = "+m,+a,+f,+d,+zicsr,+zifencei,+zba,+zbb,+zbs,+zbc,+zbkb,+zbkc,+zbkx,+zicond,+zfa,+zfh,\
+zawrs,+zacas,+zabha,+zicbom,+zicboz,+zicbop,+zihintntl,+zimop,+zknd,+zkne,+zknh,+zksed,+zksh,+svinval,+zicntr,\
+zalasr,+experimental-zicfilp,+experimental-zicfiss,+q,+zfbfmin,+h,+sdext,+smrnmi,+ssctr,+v,+zvbb,+zvbc,\
+zvkg,+zvkned,+zvknha,+zvknhb,+zvksed,+zvksh,+zvfh,+zvfbfmin,+zvfbfwma";
/// And those of its compressed instructions.
const RISCV_C_FEATURES: &str = "+m,+a,+f,+d,+zicsr,+zba,+zbb,+zihintntl,+zimop,+experimental-zicfiss,+zcb,+zcmop";
