//! Golden tests: each `tests/examples/<name>.ll` is run through the
//! pipeline and the result, with its remarks, must match
//! `tests/examples/<name>.expected.ll`.
//!
//! The first line of an example may give options, as
//! `; epitome-opt: --no-inline -O3`. Run with `UPDATE_EXPECTED=1` to
//! rewrite the expected files after a deliberate change.
//!
//! Each example is also checked to be the same with 1 and 4 threads and to
//! pass `cfg::verify`. When `clang` is installed, the input and output are
//! both built and run, and `main` must return the same value.

use std::path::{Path, PathBuf};
use std::process::Command;

use epitome_opt::inline::OptLevel;
use epitome_opt::pipeline::{self, Options};
use epitome_opt::recursion::RecursionMode;

fn options(src: &str) -> (Options, bool) {
    let mut opts = Options::default();
    let mut cycles = false;
    let Some(args) = src.lines().next().and_then(|l| l.strip_prefix("; epitome-opt:")) else {
        return (opts, cycles);
    };
    for a in args.split_whitespace() {
        match a {
            "--no-inline" => opts.inline = false,
            "--no-combine" => opts.combine = false,
            "--no-cse" => opts.cse = false,
            "--no-mem2reg" => opts.mem2reg = false,
            "--no-sroa" => opts.sroa = false,
            "--no-simplifycfg" => opts.simplifycfg = false,
            "--no-loops" => opts.loops = false,
            "--no-implied" => opts.implied = false,
            s if s.starts_with("--passes=") => {
                let list: Vec<&str> = s["--passes=".len()..].split(',').collect();
                opts.inline = list.contains(&"inline");
                opts.combine = list.contains(&"combine");
                opts.cse = list.contains(&"cse");
                opts.memory = list.contains(&"memory");
                opts.tailmark = list.contains(&"tailmark");
                opts.ipo = list.contains(&"ipo");
                opts.mem2reg = list.contains(&"mem2reg");
                opts.sroa = list.contains(&"sroa");
                opts.simplifycfg = list.contains(&"simplifycfg");
                opts.loops = list.contains(&"loops");
                opts.implied = list.contains(&"implied");
                if !list.contains(&"recursion") {
                    opts.recursion = RecursionMode::Off;
                }
            }
            "--print-cycles" => cycles = true,
            "--recursion=off" => opts.recursion = RecursionMode::Off,
            "--recursion=tail" => opts.recursion = RecursionMode::Tail,
            "--recursion=stack" => opts.recursion = RecursionMode::Stack,
            s if s.starts_with("-O") => opts.level = OptLevel::parse(&s[2..]).expect("bad level"),
            s => panic!("unknown option {s}"),
        }
    }
    (opts, cycles)
}

fn optimise(src: &str, threads: usize) -> String {
    let (mut opts, cycles) = options(src);
    opts.threads = threads;
    let mut m = epitome_ir::parse_module(src).unwrap();
    let report = pipeline::run(&mut m, &opts);
    for (g, f) in m.functions() {
        let errors = epitome_opt::cfg::verify(f, m.global(g).name.as_deref().unwrap_or(""));
        assert!(errors.is_empty(), "{}", errors.join("\n"));
    }
    let mut out = String::new();
    if cycles {
        for c in &report.cycles {
            out.push_str(&format!("; cycle: {c}\n"));
        }
    }
    for (f, list) in &report.remarks {
        for r in list {
            out.push_str(&format!("; @{f}: {r}\n"));
        }
    }
    for f in &report.deleted {
        out.push_str(&format!("; @{f}: deleted, no longer used\n"));
    }
    out.push('\n');
    out.push_str(&m.to_string());
    out
}

/// Builds `ll` and returns `main`'s exit status.
fn exit_code(ll: &Path, exe: &Path) -> i32 {
    let built = Command::new("clang").args(["-w", "-x", "ir"]).arg(ll).arg("-o").arg(exe).output().unwrap();
    assert!(built.status.success(), "{}: {}", ll.display(), String::from_utf8_lossy(&built.stderr));
    Command::new(exe).status().unwrap().code().unwrap_or(-1)
}

#[test]
fn examples_match_expected_output() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/examples");
    let tmp = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("examples");
    std::fs::create_dir_all(&tmp).unwrap();
    let update = std::env::var_os("UPDATE_EXPECTED").is_some();
    let clang = Command::new("clang").arg("--version").output().is_ok_and(|o| o.status.success());
    let mut inputs: Vec<PathBuf> = std::fs::read_dir(&dir)
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|e| e == "ll") && !p.to_string_lossy().ends_with(".expected.ll"))
        .collect();
    inputs.sort();
    assert!(!inputs.is_empty());
    let mut failures = Vec::new();
    for input in &inputs {
        let src = std::fs::read_to_string(input).unwrap();
        let got = optimise(&src, 1);
        assert_eq!(got, optimise(&src, 4), "{}: differs between 1 and 4 threads", input.display());
        let expected_path = input.with_extension("expected.ll");
        if update {
            std::fs::write(&expected_path, &got).unwrap();
        } else {
            match std::fs::read_to_string(&expected_path) {
                Ok(want) if want == got => {}
                Ok(_) => failures.push(format!("{}: output differs from {}", input.display(), expected_path.display())),
                Err(_) => failures.push(format!("{}: no expected output; run with UPDATE_EXPECTED=1", input.display())),
            }
        }
        if clang && src.contains("@main(") {
            let stem = input.file_stem().unwrap().to_string_lossy().to_string();
            let out_ll = tmp.join(format!("{stem}.out.ll"));
            std::fs::write(&out_ll, &got).unwrap();
            let before = exit_code(input, &tmp.join(format!("{stem}.before")));
            let after = exit_code(&out_ll, &tmp.join(format!("{stem}.after")));
            if before != after {
                failures.push(format!("{}: main returned {before} before and {after} after", input.display()));
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
