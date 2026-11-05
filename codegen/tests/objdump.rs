//! Checks epitome-objdump's disassembler on code we did not generate:
//! clang's own builds of the C programs in `tests/programs/` (and of
//! SQLite, when `EPITOME_SQLITE` names the amalgamation, as in
//! `tests/sqlite.rs`), at `-O2` with vectorising. Every word the
//! description decodes must be, to the system's objdump, the same
//! instruction: the same mnemonic, or an alias of it. Words the
//! description does not have are counted, not checked.
//!
//! A wrong decode (bits read as some other instruction) is worse than none,
//! and code from another compiler uses encodings ours never makes. This
//! test found shifts by immediate claiming `movi` words, and `adrp`s with
//! their low page bits dropped.
//!
//! On a host that is not AArch64, the programs are built for AArch64 Linux
//! (with the few headers of `tests/riscv/include`; those needing more are
//! skipped), and `llvm-objdump` reads them. The test is skipped when `clang` or an objdump is not
//! installed.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;

use epitome_codegen::objdump::{self, Image};
use epitome_codegen::target::Target;

/// Mnemonics objdump prints for instructions the description writes
/// another way (an alias, or the instruction the alias stands for).
const ALIASES: &[(&str, &str)] = &[
    ("add", "mov"),
    ("mov", "orr"),
    ("mov", "movz"),
    ("mov", "movn"),
    ("mov", "ins"),
    ("mov", "umov"),
    ("mov", "dup"),
    ("neg", "sub"),
    ("sshll", "sxtl"),
    ("ushll", "uxtl"),
    ("sshll2", "sxtl2"),
    ("ushll2", "uxtl2"),
    ("ror", "extr"),
];

const NATIVE: bool = cfg!(target_arch = "aarch64");

/// The objdump that reads AArch64 here: the system's on an AArch64 host,
/// LLVM's elsewhere.
const OBJDUMP: &str = if NATIVE { "objdump" } else { "llvm-objdump" };

fn have_tools() -> bool {
    let ok = |c: &str, a: &str| Command::new(c).arg(a).output().is_ok_and(|o| o.status.success());
    ok("clang", "--version") && ok(OBJDUMP, "--version")
}

/// objdump writes a vector's arrangement into the mnemonic (`add.4s`);
/// the description writes it on the registers.
fn plain_mnemonic(m: &str) -> &str {
    match m.rsplit_once('.') {
        Some((base, arr))
            if !base.is_empty() && (arr.len() == 1 || arr.chars().next().is_some_and(|c| c.is_ascii_digit())) =>
        {
            base
        }
        _ => m,
    }
}

/// Disassembles `obj` both ways and compares; returns (decoded, words).
fn compare(t: &Target, obj: &Path) -> Result<(usize, usize), String> {
    let out = Command::new(OBJDUMP).arg("-d").arg(obj).output().map_err(|e| e.to_string())?;
    let sys = String::from_utf8_lossy(&out.stdout);
    // The system's: address to mnemonic, for each instruction.
    let mut theirs: HashMap<u64, String> = HashMap::new();
    for l in sys.lines() {
        let mut parts = l.split('\t');
        let (Some(head), Some(mn)) = (parts.next(), parts.next()) else { continue };
        let Some(addr) = head.trim().split(':').next().and_then(|a| u64::from_str_radix(a, 16).ok()) else { continue };
        theirs.insert(addr, mn.trim().to_string());
    }
    let img = Image::read(&std::fs::read(obj).unwrap(), &t.machine)?;
    let ours = objdump::disassemble(t, &img, false);
    let (mut decoded, mut words) = (0, 0);
    let mut wrong = Vec::new();
    for l in ours.lines() {
        let Some((addr, rest)) = l.split_once(": ") else { continue };
        let Ok(addr) = u64::from_str_radix(addr.trim(), 16) else { continue };
        let Some((_, text)) = rest.split_once('\t') else { continue };
        words += 1;
        let mine = text.split_whitespace().next().unwrap_or("");
        if mine == ".word" {
            continue;
        }
        decoded += 1;
        let Some(sys_mn) = theirs.get(&addr) else {
            wrong.push(format!("{addr:#x}: `{text}`, which objdump does not list"));
            continue;
        };
        let other = plain_mnemonic(sys_mn);
        // An alias Arm's specification gives for the instruction we read.
        let word = img
            .sections
            .iter()
            .find(|s| s.code && addr >= s.addr && addr + 4 <= s.addr + s.data.len() as u64)
            .map(|s| &s.data[(addr - s.addr) as usize..][..4]);
        let arm_alias = word
            .and_then(|w| epitome_codegen::mir::decode(t, w))
            .is_some_and(|(i, _, _)| t.instrs[i].aliases.iter().any(|a| a == other));
        if mine != other && !arm_alias && !ALIASES.contains(&(mine, other)) && !ALIASES.contains(&(other, mine)) {
            wrong.push(format!("{addr:#x}: we read `{text}`, objdump `{sys_mn}`"));
        }
    }
    if wrong.is_empty() {
        Ok((decoded, words))
    } else {
        let n = wrong.len();
        wrong.truncate(10);
        Err(format!("{}: {n} words read wrongly:\n{}", obj.display(), wrong.join("\n")))
    }
}

#[test]
fn disassembly_agrees_with_objdump() {
    if !have_tools() {
        eprintln!("clang or objdump not found; skipping");
        return;
    }
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/programs");
    let tmp = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("objdump");
    std::fs::create_dir_all(&tmp).unwrap();
    let t = Target::aarch64();
    let mut sources: Vec<PathBuf> = std::fs::read_dir(&dir)
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|x| x == "c"))
        .collect();
    sources.sort();
    if let Ok(src) = std::env::var("EPITOME_SQLITE") {
        sources.push(PathBuf::from(src).join("sqlite3.c"));
    }
    let mut failures = Vec::new();
    for src in &sources {
        let stem = src.file_stem().unwrap().to_string_lossy().into_owned();
        let obj = tmp.join(format!("{stem}.clang.o"));
        let mut clang = Command::new("clang");
        if !NATIVE {
            clang.arg("--target=aarch64-linux-gnu").arg("-I").arg(dir.join("../riscv/include"));
        }
        let ok = clang.args(["-O2", "-c"]).arg(src).arg("-o").arg(&obj).output();
        if !ok.is_ok_and(|o| o.status.success()) {
            if !NATIVE {
                // (Headers this host has only for itself, such as pthread.h.)
                println!("skipped {stem}: clang cannot build it for AArch64 here");
                continue;
            }
            failures.push(format!("{stem}: clang failed"));
            continue;
        }
        match compare(&t, &obj) {
            Ok((decoded, words)) => {
                println!("ok {stem}: {decoded} of {words} words decoded, all as objdump reads them")
            }
            Err(e) => failures.push(e),
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
