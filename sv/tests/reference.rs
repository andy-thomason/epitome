//! Compares the preprocessor's output with another tool's, token by token,
//! over the IEEE 1800-2023 conformance tests. Set `IEEE1800_TESTS` to a
//! checkout of the tests and `SV_REFERENCE_PP` to a Verilator binary; the
//! tool is only run as a program, and its `-E -P` output lexed with our
//! lexer.
//!
//! Where the reference tool and the standard disagree, the standard wins:
//! those tests are listed in `KNOWN` with the reason.

use epitome_sv::lexer::tokenize;
use epitome_sv::preprocess::{Options, preprocess_files};
use epitome_sv::source::SourceMap;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Directives the reference tool copies to its output.
const PASSED_THROUGH: &[&str] = &[
    "`timescale",
    "`default_nettype",
    "`resetall",
    "`unconnected_drive",
    "`nounconnected_drive",
    "`celldefine",
    "`endcelldefine",
    "`pragma",
    "`begin_keywords",
    "`end_keywords",
    "`line",
];

/// Tests where the reference tool's output differs from what the standard
/// asks for, and why.
const KNOWN: &[(&str, &str)] = &[];

fn files(dir: &Path, out: &mut Vec<PathBuf>) {
    let mut entries: Vec<_> = std::fs::read_dir(dir).unwrap().map(|e| e.unwrap().path()).collect();
    entries.sort();
    for path in entries {
        if path.is_dir() {
            files(&path, out);
        } else if path.extension().is_some_and(|e| e == "sv") {
            out.push(path);
        }
    }
}

#[test]
fn matches_reference_preprocessor() {
    let (Ok(root), Ok(tool)) = (std::env::var("IEEE1800_TESTS"), std::env::var("SV_REFERENCE_PP")) else {
        eprintln!("IEEE1800_TESTS or SV_REFERENCE_PP is not set; skipped");
        return;
    };
    let root = PathBuf::from(root).join("tests");
    let mut paths = Vec::new();
    files(&root, &mut paths);
    let (mut same, mut skipped, mut diffs) = (0, 0, Vec::new());
    for path in &paths {
        let name = path.strip_prefix(&root).unwrap().display().to_string();
        let text = std::fs::read_to_string(path).unwrap();
        if text.contains(":should_fail_because:") || text.contains(":files:") {
            skipped += 1;
            continue;
        }
        let map = SourceMap::new();
        let Ok(ours) = preprocess_files(&map, std::slice::from_ref(path), &Options::default()) else {
            diffs.push(format!("{name}: we reject it"));
            continue;
        };
        let ours: Vec<&str> = ours.tokens.iter().map(|t| t.token.0).collect();
        let out = Command::new(&tool)
            .args(["-E", "-P"])
            .arg(format!("-I{}", path.parent().unwrap().display()))
            .arg(path)
            .output()
            .unwrap();
        if !out.status.success() {
            skipped += 1;
            continue;
        }
        // The reference copies directives other than macros to its output;
        // we act on them and drop them.
        let theirs_text: String = String::from_utf8_lossy(&out.stdout)
            .lines()
            .filter(|l| {
                let l = l.trim_start();
                !PASSED_THROUGH.iter().any(|d| {
                    l.strip_prefix(d).is_some_and(|r| !r.starts_with(|c: char| c.is_alphanumeric() || c == '_'))
                })
            })
            .map(|l| format!("{l}\n"))
            .collect();
        let Ok(theirs) = tokenize(&theirs_text) else {
            diffs.push(format!("{name}: the reference output does not lex"));
            continue;
        };
        let theirs: Vec<&str> = theirs.iter().map(|t| t.0).filter(|t| !t.is_empty()).collect();
        if ours == theirs {
            same += 1;
            continue;
        }
        if KNOWN.iter().any(|(k, _)| *k == name) {
            continue;
        }
        let i = ours.iter().zip(&theirs).position(|(a, b)| a != b).unwrap_or(ours.len().min(theirs.len()));
        let lo = i.saturating_sub(4);
        diffs.push(format!(
            "{name}: token {i}\n  ours:   {:?}\n  theirs: {:?}",
            &ours[lo..(i + 6).min(ours.len())],
            &theirs[lo..(i + 6).min(theirs.len())]
        ));
    }
    eprintln!("{same} the same, {skipped} skipped, {} different", diffs.len());
    assert!(diffs.is_empty(), "{}", diffs.join("\n"));
}
