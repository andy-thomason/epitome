//! The gfx1151 (RDNA 3.5) description against LLVM's AMDGPU assembler and
//! disassembler (`llvm-mc -mcpu=gfx1151`). Sample lines of every entry
//! (`epitome-llc --sample-asm`, over a spread of operands) are assembled by
//! both, and must give the same bytes; and each instruction llvm-mc makes
//! must disassemble, by us, as llvm-mc writes it.
//!
//! Skipped when there is no llvm-mc 22 or later with the AMDGPU target
//! (`LLVM_MC` names one; otherwise `llvm-mc-22` and `llvm-mc` are tried).

use std::io::Write;
use std::process::{Command, Stdio};

use epitome_codegen::abi::Syntax;
use epitome_codegen::assembler::Assembler;
use epitome_codegen::check;
use epitome_codegen::objdump;
use epitome_codegen::target::Target;

fn gfx1151() -> Target {
    Target::load_path(&format!("{}/targets/gfx1151", env!("CARGO_MANIFEST_DIR"))).unwrap()
}

/// The description's own checks: every entry encodes, decodes and
/// disassembles as itself.
#[test]
fn description_passes_its_checks() {
    let report = check::check(&gfx1151());
    for name in ["encodings", "assembly"] {
        let s = report.sections.iter().find(|s| s.name == name).unwrap();
        assert!(s.errors.is_empty(), "{}", report.render(true));
    }
}

fn llvm_mc() -> Option<String> {
    let names =
        std::env::var("LLVM_MC").map(|n| vec![n]).unwrap_or_else(|_| vec!["llvm-mc-22".into(), "llvm-mc".into()]);
    names.into_iter().find(|n| {
        Command::new(n).arg("--version").output().is_ok_and(|o| {
            let v = String::from_utf8_lossy(&o.stdout);
            // (The description follows LLVM 22's syntax; older ones differ.)
            let major = v.split("version ").nth(1).and_then(|r| r.split('.').next()?.parse::<u32>().ok());
            v.contains("amdgcn") && major.is_some_and(|m| m >= 22)
        })
    })
}

/// Runs llvm-mc on `input` with `args`: its output, and the input lines
/// (from 0) it refused.
fn run_mc(mc: &str, args: &[&str], input: String) -> (String, std::collections::HashSet<usize>, bool) {
    let mut child = Command::new(mc)
        .args(["-triple=amdgcn-amd-amdhsa", "-mcpu=gfx1151"])
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let mut stdin = child.stdin.take().unwrap();
    std::thread::spawn(move || stdin.write_all(input.as_bytes()).unwrap());
    let out = child.wait_with_output().unwrap();
    let refused = String::from_utf8_lossy(&out.stderr)
        .lines()
        .filter(|l| l.contains("error") || l.contains("warning"))
        .filter_map(|l| l.strip_prefix("<stdin>:")?.split(':').next()?.parse::<usize>().ok())
        .map(|n| n - 1)
        .collect();
    (String::from_utf8_lossy(&out.stdout).into_owned(), refused, out.status.success())
}

/// llvm-mc's bytes for each line; `None` for a line it refuses, and empty
/// for a branch to a label, which has a fixup for bytes
/// (`branches_to_labels` has those).
fn their_bytes(mc: &str, lines: &[String]) -> Vec<Option<Vec<u8>>> {
    let (out, refused, _) = run_mc(mc, &["-show-encoding"], lines.iter().map(|l| format!("{l}\n")).collect());
    let mut read = out
        .lines()
        .filter_map(|l| {
            let enc = l.split_once("; encoding: [")?.1.trim_end_matches(']');
            let bytes: Option<Vec<u8>> = enc
                .split(',')
                .filter(|b| !b.is_empty())
                .map(|b| b.trim().strip_prefix("0x").and_then(|h| u8::from_str_radix(h, 16).ok()))
                .collect();
            Some(bytes.unwrap_or_default())
        })
        .collect::<Vec<_>>()
        .into_iter();
    (0..lines.len()).map(|k| if refused.contains(&k) { None } else { read.next() }).collect()
}

/// llvm-mc's disassembly of each instruction: `None` where it has none,
/// and the instructions it crashes on (llvm-mc 22 traps on a few words).
/// Each is followed by `s_nop k` (k counting from 1, as no instruction can
/// take it as its literal), and what is written before the marker is the
/// instruction's: one that llvm-mc reads as two instructions, or as one
/// too short, is not then taken for the next's.
fn their_text(mc: &str, insts: &[&Vec<u8>]) -> (Vec<Option<String>>, Vec<usize>) {
    let mut texts = vec![None; insts.len()];
    let mut crashes = Vec::new();
    let mut start = 0;
    while start < insts.len() {
        let input: String = insts[start..]
            .iter()
            .enumerate()
            .map(|(k, b)| {
                let marker = (0xbf80_0000u32 | (start + k) as u32 + 1 & 0xffff).to_le_bytes();
                b.iter().chain(&marker).map(|x| format!("0x{x:02x}")).collect::<Vec<_>>().join(",") + "\n"
            })
            .collect();
        let (out, _, finished) = run_mc(mc, &["--disassemble"], input);
        let (mut k, mut lines) = (start, Vec::new());
        for l in out.lines().filter(|l| l.starts_with('\t') && !l.trim_start().starts_with('.')) {
            let l = l.split_whitespace().collect::<Vec<_>>().join(" ");
            let marker = l.strip_prefix("s_nop ").and_then(|n| match n.strip_prefix("0x") {
                Some(h) => usize::from_str_radix(h, 16).ok(),
                None => n.parse().ok(),
            });
            // (A marker taken as the literal of an instruction too short is
            // missed: the next found starts again from its own number.)
            let ahead = marker.and_then(|m| (1..=16).find(|d| (k + d) & 0xffff == m));
            match ahead {
                Some(d) if k < insts.len() => {
                    k += d - 1;
                    if k < insts.len() && d == 1 {
                        texts[k] = (!lines.is_empty()).then(|| lines.join(" ; "));
                    }
                    lines.clear();
                    k += 1;
                }
                _ => lines.push(l),
            }
        }
        if finished {
            break;
        }
        // It crashed on instruction k: on from the next.
        crashes.push(k);
        start = k + 1;
    }
    (texts, crashes)
}

#[test]
fn epitome_and_llvm_mc_agree() {
    let Some(mc) = llvm_mc() else {
        eprintln!("needs llvm-mc 22 or later with the AMDGPU target; skipping");
        return;
    };
    let t = gfx1151();
    let asm = Assembler::new(&t);
    let sample = check::sample_assembly(&t, Syntax::Elf, 3);
    // Each sample line, without its label and comment.
    let lines: Vec<String> = sample
        .lines()
        .filter_map(|l| l.split_once(":\t"))
        .map(|(_, l)| l.split_once('\t').map_or(l, |(l, _)| l).trim().to_string())
        .collect();
    let bytes = their_bytes(&mc, &lines);
    let insts: Vec<&Vec<u8>> = bytes.iter().flatten().filter(|b| !b.is_empty()).collect();
    let (texts, crashes) = their_text(&mc, &insts);
    let crashed: Vec<String> = crashes.iter().map(|&k| format!("{:02x?}", insts[k])).collect();
    let crashes: std::collections::HashSet<usize> = crashes.into_iter().collect();
    let mut texts = texts.into_iter().enumerate();
    let (mut wrong, mut refused) = (Vec::new(), Vec::new());
    for (line, their) in lines.iter().zip(&bytes) {
        let Some(bytes) = their else {
            refused.push(line.clone());
            continue;
        };
        if bytes.is_empty() {
            // A branch to a label (resolved later, by a fixup).
            continue;
        }
        let (k, text) = texts.next().unwrap();
        if crashes.contains(&k) {
            continue;
        }
        let ours = asm
            .assemble(&format!("\t{line}\n"), Some(Syntax::Elf))
            .map(|a| a.object.sections.iter().flat_map(|s| s.data.clone()).collect::<Vec<u8>>());
        match ours {
            Ok(b) if &b == bytes => {}
            Ok(b) => wrong.push(format!("{line}: ours {b:02x?}, llvm-mc's {bytes:02x?}")),
            Err(e) => wrong.push(format!("{line}: {e}")),
        }
        // As llvm-mc's disassembler writes it (which need not be the line:
        // `0x0` for a zero offset is written `null`).
        let (back, len) = objdump::instruction(&t, bytes, 0, None, false, &|a| format!("{a}"), &|_| {});
        let back = back.split_whitespace().collect::<Vec<_>>().join(" ");
        if text.as_deref() != Some(back.as_str()) || len != bytes.len() {
            wrong.push(format!("{bytes:02x?}: we read `{back}` ({len} bytes), llvm-mc `{}`", text.unwrap_or_default()));
        }
    }
    let report = std::path::PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("gfx1151-differences.txt");
    std::fs::write(
        &report,
        format!(
            "refused:\n{}\n\ndiffer:\n{}\n\nllvm-mc's disassembler crashes on:\n{}\n",
            refused.join("\n"),
            wrong.join("\n"),
            crashed.join("\n")
        ),
    )
    .unwrap();
    eprintln!(
        "{} lines; llvm-mc refused {}; {} differ; llvm-mc's disassembler crashes on {}",
        lines.len(),
        refused.len(),
        wrong.len(),
        crashed.len()
    );
    assert!(
        refused.is_empty() && wrong.is_empty(),
        "llvm-mc refused {}:\n{}\n{} differ:\n{}\n(all of them in {})",
        refused.len(),
        refused[..refused.len().min(20)].join("\n"),
        wrong.len(),
        wrong[..wrong.len().min(40)].join("\n"),
        report.display()
    );
}

/// Branches to labels, before and after: the offset is in words from the
/// next instruction. (The bytes are llvm-mc's.)
#[test]
fn branches_to_labels() {
    let t = gfx1151();
    let src = "back:\n\ts_nop 0\n\ts_cbranch_execz back\n\ts_branch on\n\ts_nop 0\non:\n\ts_endpgm\n";
    let a = Assembler::new(&t).assemble(src, Some(Syntax::Elf)).unwrap();
    let text: Vec<u8> = a.object.sections.iter().flat_map(|s| s.data.clone()).collect();
    let want: Vec<u8> =
        [0xbf800000u32, 0xbfa5fffe, 0xbfa00001, 0xbf800000, 0xbfb00000].iter().flat_map(|w| w.to_le_bytes()).collect();
    assert_eq!(text, want);
}
