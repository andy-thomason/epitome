//! Expands each program in `tests/expand/` and compares the result, token
//! by token, with `rustc -Zunpretty=expanded`. Skipped where `rustc` is not
//! installed.

use epitome_rustc2::cfg::CfgSet;
use epitome_rustc2::lexer::{Edition, lex};
use epitome_rustc2::pretty::pretty;
use epitome_rustc2::session::{CrateRoot, Options, Session};
use std::path::Path;
use std::process::Command;

/// What rustc adds to every crate, which we leave out.
fn injected(line: &str) -> bool {
    let l = line.trim();
    l == "#![feature(prelude_import)]"
        || l == "#[prelude_import]"
        || l.starts_with("extern crate core;")
        || l.starts_with("extern crate std;")
        || (l.starts_with("use core::prelude::rust_") || l.starts_with("use std::prelude::rust_"))
            && l.ends_with("::*;")
}

fn rustc_expanded(path: &Path) -> Option<String> {
    let out = Command::new("rustc")
        .args(["-Zunpretty=expanded", "--edition", "2021", "--crate-type", "lib"])
        .arg(path)
        .env("RUSTC_BOOTSTRAP", "1")
        .output()
        .ok()?;
    assert!(out.status.success(), "rustc failed on {}:\n{}", path.display(), String::from_utf8_lossy(&out.stderr));
    let text = String::from_utf8(out.stdout).unwrap();
    Some(text.lines().filter(|l| !injected(l)).collect::<Vec<_>>().join("\n"))
}

fn ours(path: &Path) -> String {
    let session = Session::new(Options { threads: 2, cfg: Some(CfgSet::host()) });
    let mut crates = session.parse_crates(vec![CrateRoot::new("t", path).edition(Edition::E2021)]);
    session.expand(&mut crates);
    let c = &crates[0];
    let errors: Vec<String> = c.errors.iter().map(|e| e.render_in(&session.sources)).collect();
    assert!(errors.is_empty(), "{}", errors.join(""));
    pretty(&c.files[0].ast)
}

#[test]
fn expansion_matches_rustc() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/expand");
    let mut files: Vec<_> = std::fs::read_dir(&dir).unwrap().map(|e| e.unwrap().path()).collect();
    files.sort();
    let mut failures = Vec::new();
    for f in files.iter().filter(|f| f.extension().is_some_and(|x| x == "rs")) {
        let Some(theirs) = rustc_expanded(f) else { return };
        let ours = ours(f);
        let a: Vec<&str> = lex(&ours).unwrap().into_iter().map(|t| t.0).collect();
        let b: Vec<&str> = lex(&theirs).unwrap().into_iter().map(|t| t.0).collect();
        if a != b {
            let i = a.iter().zip(&b).position(|(x, y)| x != y).unwrap_or(a.len().min(b.len()));
            let lo = i.saturating_sub(12);
            failures.push(format!(
                "{}: differs at token {i}\n  ours:  {}\n  rustc: {}",
                f.file_name().unwrap().to_string_lossy(),
                a[lo..(i + 12).min(a.len())].join(" "),
                b[lo..(i + 12).min(b.len())].join(" ")
            ));
        }
    }
    assert!(failures.is_empty(), "\n{}", failures.join("\n"));
}
