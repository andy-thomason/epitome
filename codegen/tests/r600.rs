//! The R600 (Evergreen) description against LLVM's r600 back end. LLVM has
//! no assembler or disassembler for it, but `llc -show-mc-encoding` writes
//! each instruction's encoding beside its assembly: each kernel in
//! `tests/r600/` is compiled so, and each line, assembled by us, must give
//! the same bytes.
//!
//! Skipped when `llc` has no r600 target.

use std::path::PathBuf;
use std::process::Command;

use epitome_codegen::abi::Syntax;
use epitome_codegen::assembler::Assembler;
use epitome_codegen::check;
use epitome_codegen::target::Target;

fn r600() -> Target {
    Target::load_path(&format!("{}/targets/r600", env!("CARGO_MANIFEST_DIR"))).unwrap()
}

/// The description's own checks: every entry encodes, decodes and
/// disassembles as itself, and its assembly reads back.
#[test]
fn description_passes_its_checks() {
    let report = check::check(&r600());
    for name in ["encodings", "assembly"] {
        let s = report.sections.iter().find(|s| s.name == name).unwrap();
        assert!(s.errors.is_empty(), "{}", report.render(true));
    }
}

/// llc's line as we read it: its columns of spaces made one, and a literal
/// constant's value as a float, which llc writes beside it, left out
/// (`2(2.802597e-45), 0(0.000000e+00)` is `LITERALS 2, 0`).
fn ours(line: &str) -> String {
    let line = line.split_whitespace().collect::<Vec<_>>().join(" ");
    let literal = |s: &str| s.split_once('(').map(|(n, _)| n.trim().to_string());
    match line.split_once("), ") {
        Some((a, b)) if line.ends_with(')') => match (literal(a), literal(b)) {
            (Some(x), Some(y)) if x.parse::<i64>().is_ok() && y.parse::<i64>().is_ok() => {
                format!("LITERALS {x}, {y}")
            }
            _ => line,
        },
        _ => line,
    }
}

/// Each line of llc's assembly with an encoding: the line, and the bytes.
fn encoded(asm: &str) -> Vec<(String, Vec<u8>)> {
    asm.lines()
        .filter_map(|l| {
            let (text, enc) = l.split_once("; encoding: [")?;
            let bytes: Vec<u8> = enc
                .trim_end_matches(']')
                .split(',')
                .filter(|b| !b.is_empty())
                .map(|b| u8::from_str_radix(b.trim().trim_start_matches("0x"), 16).unwrap())
                .collect();
            (!bytes.is_empty()).then(|| (ours(text.trim()), bytes))
        })
        .collect()
}

#[test]
fn encodings_are_llcs() {
    let has_r600 = Command::new("llc")
        .arg("--version")
        .output()
        .is_ok_and(|o| String::from_utf8_lossy(&o.stdout).contains("r600"));
    if !has_r600 {
        eprintln!("needs llc with the r600 target; skipping");
        return;
    }
    let t = r600();
    let asm = Assembler::new(&t);
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/r600");
    let mut names: Vec<PathBuf> = std::fs::read_dir(&dir)
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|x| x == "ll"))
        .collect();
    names.sort();
    let (mut lines, mut failures) = (0, Vec::new());
    for ll in &names {
        let o = Command::new("llc")
            .args(["-mtriple=r600", "-mcpu=cypress", "-show-mc-encoding", "-o", "-"])
            .arg(ll)
            .output()
            .unwrap();
        assert!(o.status.success(), "llc {}: {}", ll.display(), String::from_utf8_lossy(&o.stderr));
        for (line, want) in encoded(&String::from_utf8_lossy(&o.stdout)) {
            lines += 1;
            let got = asm
                .assemble(&format!("\t{line}\n"), Some(Syntax::Elf))
                .map(|a| a.object.sections.iter().flat_map(|s| s.data.clone()).collect::<Vec<u8>>());
            match got {
                Ok(b) if b == want => {}
                Ok(b) => failures.push(format!("{line}: ours {b:02x?}, llc's {want:02x?}")),
                Err(e) => failures.push(format!("{line}: {e}")),
            }
        }
    }
    assert!(failures.is_empty(), "{} of {lines} lines differ:\n{}", failures.len(), failures.join("\n"));
}
