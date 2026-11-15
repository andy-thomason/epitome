//! rustc's `check-pass` UI tests: programs rustc accepts, so they must
//! parse, expand, resolve and type check here without errors.
//!
//! Set `EPITOME_RUST` to a checkout of rust-lang/rust with `tests/ui` and
//! `library` (a sparse checkout of the two is enough), at the commit named
//! in `tests/ui_check_pass_todo.txt`; without it the test is skipped. The
//! tests chosen are loaded, a few hundred at a time, as crates of one
//! session beside `core`, `alloc` and `std` from the checkout's `library`.
//! Each test crate has its own configuration (`--cfg`) and may use only
//! those three, so that tests cannot see each other. Each session runs in
//! a child process, so that a test that crashes or hangs the compiler is
//! found (by halving) and fails on its own.
//!
//! Each revision of a test with `//@ revisions` is a test of its own,
//! `path#rev`, compiled with `--cfg rev` and the directives for it
//! (`//@[rev] ...`) as well as the file's.
//!
//! `tests/ui_check_pass_todo.txt` is the work left: every `check-pass`
//! test that does not pass yet, with why. Those that fail say at which
//! stage and the first error; those left out say what they need that the
//! runner cannot give yet (auxiliary crates, another target, `--test`).
//! Tests for other hosts (`ignore-*`, `only-*`) are not listed. The list
//! is for one rust-lang/rust commit and one host, named on its first line.
//!
//! The test fails if a test not on the list fails, or one on it passes:
//! `EPITOME_UI_BLESS=1` writes the list again, so a change that fixes
//! tests takes them off in the same commit. `EPITOME_UI_FILTER=substring`
//! runs only the tests whose path has it in (`EPITOME_UI_FILTER=/ui/fmt/`).

use epitome_rustc2::cfg::CfgSet;
use epitome_rustc2::lexer::Edition;
use epitome_rustc2::session::{CrateRoot, Options, Session};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::process::Command;

/// A test to run: its path under `tests/ui` (with `#revision` for one
/// revision of a test with several), the edition it asks for, and the
/// options it sets with `--cfg` (`name` or `name="value"`).
struct Test {
    rel: String,
    path: PathBuf,
    edition: Edition,
    cfg: Vec<String>,
}

/// How many tests a child process takes at a time.
const BATCH: usize = 250;

fn year(e: Edition) -> &'static str {
    match e {
        Edition::E2015 => "2015",
        Edition::E2018 => "2018",
        Edition::E2021 => "2021",
        Edition::E2024 => "2024",
    }
}

/// What a `check-pass` test (or one revision of one) needs.
enum Selected {
    /// Not run on this host (`ignore-*`, `only-*`): neither run nor listed.
    Ignored,
    LeftOut(&'static str),
    Run(Test),
}

/// The modes compiletest knows; a test is in one.
const MODES: [&str; 6] = ["check-pass", "build-pass", "run-pass", "check-fail", "build-fail", "run-fail"];

/// The `check-pass` tests of a file, by name (the path, or `path#rev` for
/// each revision), with what each needs. A directive `//@[a,b] name`
/// applies to revisions `a` and `b` only; each revision is compiled with
/// `--cfg rev` as well as its own flags.
fn select(ui: &Path, path: &Path) -> Vec<(String, Selected)> {
    let Ok(text) = std::fs::read_to_string(path) else { return Vec::new() };
    // Each directive: the revisions it is for (all if `None`), its name
    // and its value.
    let mut directives: Vec<(Option<Vec<&str>>, &str, &str)> = Vec::new();
    for line in text.lines() {
        let Some(d) = line.trim_start().strip_prefix("//@") else { continue };
        let mut d = d.trim();
        let mut revs = None;
        if let Some(rest) = d.strip_prefix('[') {
            let Some((r, rest)) = rest.split_once(']') else { continue };
            revs = Some(r.split(',').map(str::trim).collect());
            d = rest.trim();
        }
        let (name, value) = match d.split_once(':') {
            Some((n, v)) => (n.trim(), v.trim()),
            None => (d, ""),
        };
        directives.push((revs, name, value));
    }
    let rel = path.strip_prefix(ui).unwrap().to_string_lossy().into_owned();
    let revisions: Vec<&str> = directives
        .iter()
        .filter(|d| d.0.is_none() && d.1 == "revisions")
        .flat_map(|d| d.2.split([' ', ',']).filter(|r| !r.is_empty()))
        .collect();
    if revisions.is_empty() {
        let all: Vec<(&str, &str)> = directives.iter().map(|d| (d.1, d.2)).collect();
        return select_one(path, rel, &all, None).into_iter().collect();
    }
    revisions
        .iter()
        .filter_map(|&rev| {
            // The file's directives, then the revision's (which come
            // after, so that its mode wins).
            let mut mine: Vec<(&str, &str)> = directives.iter().filter(|d| d.0.is_none()).map(|d| (d.1, d.2)).collect();
            mine.extend(
                directives.iter().filter(|d| d.0.as_ref().is_some_and(|r| r.contains(&rev))).map(|d| (d.1, d.2)),
            );
            select_one(path, format!("{rel}#{rev}"), &mine, Some(rev))
        })
        .collect()
}

/// What one test (or revision) whose directives are `directives` needs;
/// `None` if it is not a `check-pass` test.
fn select_one(path: &Path, name: String, directives: &[(&str, &str)], rev: Option<&str>) -> Option<(String, Selected)> {
    let mode = directives.iter().rev().find(|d| MODES.contains(&d.0)).map(|d| d.0);
    if mode != Some("check-pass") {
        return None;
    }
    let mut ignored = false;
    // The first reason the test cannot run here.
    let mut left_out: Option<&'static str> = None;
    let mut edition = Edition::E2015;
    let mut cfg: Vec<String> = rev.map(str::to_string).into_iter().collect();
    for &(name, value) in directives {
        match name {
            "aux-build" | "aux-crate" | "aux-bin" | "aux-codegen-backend" | "proc-macro" => {
                left_out.get_or_insert("auxiliary crates");
            }
            "edition" => {
                // `2021`, or a range `2018..` / `2015..2021`: its start.
                let start: String = value.chars().take_while(char::is_ascii_digit).collect();
                match Edition::parse(&start) {
                    Some(e) => edition = e,
                    None => _ = left_out.get_or_insert("edition"),
                }
            }
            "compile-flags" => {
                let mut flags = value.split_whitespace();
                while let Some(f) = flags.next() {
                    // `--flag value` or `--flag=value`.
                    let mut arg = |prefix: &str| {
                        let v = f.strip_prefix(prefix)?;
                        Some(
                            if v.is_empty() { flags.next().unwrap_or("") } else { v.trim_start_matches('=') }
                                .to_string(),
                        )
                    };
                    if let Some(e) = arg("--edition") {
                        match Edition::parse(&e) {
                            Some(e) => edition = e,
                            None => _ = left_out.get_or_insert("edition"),
                        }
                    } else if let Some(c) = arg("--cfg") {
                        cfg.push(c.trim_matches('\'').to_string());
                    } else if f == "--test" {
                        left_out.get_or_insert("--test");
                    } else if f.starts_with("--extern") || f.starts_with("--target") || f.starts_with("-Zcrate-attr") {
                        left_out.get_or_insert("flags that change the crate");
                    } else if f.starts_with("--crate-type")
                        && (f.contains("proc-macro") || flags.clone().next() == Some("proc-macro"))
                    {
                        left_out.get_or_insert("proc-macro crate");
                    }
                }
            }
            _ => {
                if let Some(x) = name.strip_prefix("ignore-") {
                    ignored |= x == "test" || host_is(x);
                } else if let Some(x) = name.strip_prefix("only-") {
                    ignored |= !host_is(x);
                }
            }
        }
    }
    let what = if ignored {
        Selected::Ignored
    } else if let Some(why) = left_out {
        Selected::LeftOut(why)
    } else {
        Selected::Run(Test { rel: name.clone(), path: path.to_path_buf(), edition, cfg })
    };
    Some((name, what))
}

/// Whether `ignore-x` / `only-x` names this host.
fn host_is(x: &str) -> bool {
    let os = std::env::consts::OS;
    let arch = std::env::consts::ARCH;
    x == os
        || x == arch
        || x == std::env::consts::FAMILY
        || (x == "apple" && matches!(os, "macos" | "ios"))
        || (x == "64bit" && cfg!(target_pointer_width = "64"))
        || (x == "32bit" && cfg!(target_pointer_width = "32"))
        || (x == "gnu" && cfg!(target_env = "gnu"))
        || (x == "msvc" && cfg!(target_env = "msvc"))
        || (x == "musl" && cfg!(target_env = "musl"))
        || (x == "osx" && os == "macos")
        || (x == "arm64" && arch == "aarch64")
        || x.strip_suffix("-unknown-linux-gnu").is_some_and(|a| a == arch && os == "linux")
        || x.strip_suffix("-apple-darwin")
            .is_some_and(|a| (a == arch || (a == "arm64" && arch == "aarch64")) && os == "macos")
}

fn collect(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else { return };
    for e in entries.flatten() {
        let p = e.path();
        if p.is_dir() {
            // Auxiliary crates are not tests, and a test's modules are
            // loaded through it.
            if p.file_name().is_some_and(|n| n != "auxiliary") {
                collect(&p, out);
            }
        } else if p.extension().is_some_and(|x| x == "rs") {
            out.push(p);
        }
    }
}

/// Runs `tests` in one session: for each, `None` if it passed, or why not.
fn run(library: &Path, tests: &[&Test]) -> Vec<Option<String>> {
    let session = Session::new(Options { cfg: Some(CfgSet::host()), ..Default::default() });
    let mut roots: Vec<CrateRoot> = ["core", "alloc", "std"]
        .iter()
        .map(|n| CrateRoot::new(*n, library.join(n).join("src/lib.rs")).edition(Edition::E2024))
        .collect();
    let first = roots.len();
    // Names no program could mean, so that one test's paths cannot find
    // another test.
    roots.extend(tests.iter().enumerate().map(|(i, t)| {
        let root = CrateRoot::new(format!("__ui_test_{i}"), &t.path)
            .edition(t.edition)
            .deps(["core", "alloc", "std"].map(String::from).to_vec());
        if t.cfg.is_empty() {
            return root;
        }
        let mut cfg = CfgSet::host();
        for c in &t.cfg {
            match c.split_once('=') {
                Some((n, v)) => cfg.set_value(n, v.trim_matches('"')),
                None => cfg.set(c),
            };
        }
        root.cfg(cfg)
    }));
    let mut crates = session.parse_crates(roots);
    let expanded = session.expand(&mut crates);
    let r = session.resolve(&crates);
    let tcx = session.collect_types(&crates, &r);
    let res = session.typeck(&crates, &r, &tcx);
    (first..crates.len())
        .map(|k| {
            let c = &crates[k];
            if let Some(e) = c.errors.first() {
                // `file:line:col: error: message`: the message, then where.
                let line = e.render_in(&session.sources).lines().next().unwrap_or("").to_string();
                return Some(match line.split_once(": error: ") {
                    Some((at, message)) => format!("error: {message} ({})", at.rsplit('/').next().unwrap_or(at)),
                    None => line,
                });
            }
            if let Some(m) = expanded.unresolved_by_crate.get(&k) {
                return Some(format!("macro not found: {}!", m.keys().next().unwrap()));
            }
            if let Some(i) = r.failed_import_lines_of(k as u32).first() {
                return Some(format!("import not resolved: {i}"));
            }
            let s = r.crate_stats(&crates, k);
            if s.unresolved > 0 {
                return Some(format!("path not resolved: {}", s.unresolved_names[0].0));
            }
            let t = &res.crate_stats[k];
            if t.clean_bodies < t.bodies {
                let why = t.reasons.first().map_or("a type left unknown", |w| w.0.as_str());
                return Some(format!("type check: {why}"));
            }
            None
        })
        .collect()
}

/// Runs `tests` in a child process (this test again, with the tests in
/// `EPITOME_UI_BATCH`), so that a test that crashes the compiler (a panic,
/// a stack overflow) or makes it hang cannot take the others with it.
/// When the child fails, the tests are split in two and run again, until
/// the test that fails is found.
fn run_isolated(library: &Path, tests: &[&Test], out: &mut Vec<Option<String>>) {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"));
    let (batch, results) = (dir.join("ui_check_pass.batch"), dir.join("ui_check_pass.results"));
    let list: String = tests
        .iter()
        .map(|t| format!("{}\t{}\t{}\t{}\n", year(t.edition), t.rel, t.path.display(), t.cfg.join("\x1f")))
        .collect();
    std::fs::write(&batch, list).unwrap();
    let _ = std::fs::remove_file(&results);
    let mut child = Command::new(std::env::current_exe().unwrap())
        .args(["check_pass_tests_are_accepted", "--exact", "--nocapture"])
        .env("EPITOME_UI_BATCH", &batch)
        .env("EPITOME_UI_RESULTS", &results)
        .env("EPITOME_RUST_LIBRARY", library)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    // Read stderr as it comes, so that the child never blocks on it.
    let mut stderr = child.stderr.take().unwrap();
    let reader = std::thread::spawn(move || {
        let mut s = String::new();
        let _ = std::io::Read::read_to_string(&mut stderr, &mut s);
        s
    });
    let limit = std::time::Duration::from_secs(30) + std::time::Duration::from_millis(200) * tests.len() as u32;
    let started = std::time::Instant::now();
    let status = loop {
        if let Some(s) = child.try_wait().unwrap() {
            break Some(s);
        }
        if started.elapsed() > limit {
            let _ = child.kill();
            let _ = child.wait();
            break None;
        }
        std::thread::sleep(std::time::Duration::from_millis(50));
    };
    let stderr = reader.join().unwrap();
    let lines = std::fs::read_to_string(&results).unwrap_or_default();
    if status.is_some_and(|s| s.success()) && lines.lines().count() == tests.len() {
        out.extend(lines.lines().map(|l| l.strip_prefix("fail\t").map(str::to_string)));
        return;
    }
    if tests.len() == 1 {
        let why = match status {
            None => "timed out".to_string(),
            Some(_) if stderr.contains("overflowed its stack") => "crash: stack overflow".to_string(),
            Some(s) => {
                // `thread '...' panicked at file:line:col:` and the message.
                let mut it = stderr.lines().skip_while(|l| !l.contains("panicked at"));
                let at = it.next().and_then(|l| l.split("panicked at ").nth(1)).unwrap_or("").trim_end_matches(':');
                let at = at.rsplit('/').next().unwrap_or(at);
                match it.next() {
                    Some(message) => format!("crash: panic: {message} ({at})"),
                    None => format!("crash: {s}"),
                }
            }
        };
        out.push(Some(why));
        return;
    }
    let (a, b) = tests.split_at(tests.len() / 2);
    run_isolated(library, a, out);
    run_isolated(library, b, out);
}

/// In a child process: runs the tests `EPITOME_UI_BATCH` lists and writes
/// a line for each to `EPITOME_UI_RESULTS`, `ok` or `fail\twhy`.
fn child(batch: &Path) {
    let library = PathBuf::from(std::env::var_os("EPITOME_RUST_LIBRARY").unwrap());
    let results = PathBuf::from(std::env::var_os("EPITOME_UI_RESULTS").unwrap());
    let tests: Vec<Test> = std::fs::read_to_string(batch)
        .unwrap()
        .lines()
        .map(|l| {
            let mut f = l.splitn(4, '\t');
            let (e, rel, path, cfg) = (f.next().unwrap(), f.next().unwrap(), f.next().unwrap(), f.next().unwrap());
            let cfg = cfg.split('\x1f').filter(|c| !c.is_empty()).map(str::to_string).collect();
            Test { rel: rel.to_string(), path: PathBuf::from(path), edition: Edition::parse(e).unwrap(), cfg }
        })
        .collect();
    // A compiler's stack: deep expressions recurse deeply.
    let verdicts = std::thread::Builder::new()
        .stack_size(256 << 20)
        .spawn(move || run(&library, &tests.iter().collect::<Vec<_>>()))
        .unwrap()
        .join()
        .unwrap();
    let text: String = verdicts
        .iter()
        .map(|v| match v {
            None => "ok\n".to_string(),
            Some(why) => format!("fail\t{}\n", why.replace('\n', " ")),
        })
        .collect();
    std::fs::write(results, text).unwrap();
}

#[test]
fn check_pass_tests_are_accepted() {
    if let Some(batch) = std::env::var_os("EPITOME_UI_BATCH") {
        return child(Path::new(&batch));
    }
    let Some(rust) = std::env::var_os("EPITOME_RUST").map(PathBuf::from) else {
        eprintln!("EPITOME_RUST not set: skipped");
        return;
    };
    let (ui, library) = (rust.join("tests/ui"), rust.join("library"));
    assert!(ui.is_dir() && library.is_dir(), "{} needs tests/ui and library", rust.display());
    let list_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/ui_check_pass_todo.txt");
    let list = std::fs::read_to_string(&list_path).unwrap_or_default();
    let pinned = list.lines().next().and_then(|l| l.strip_prefix("# rust-lang/rust ")).unwrap_or("").to_string();
    let head = Command::new("git").arg("-C").arg(&rust).args(["rev-parse", "HEAD"]).output().ok();
    let head = head.map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string()).unwrap_or_default();
    // What is ignored, and the configuration, depend on the host.
    let head = format!("{head} on {}-{}", std::env::consts::ARCH, std::env::consts::OS);
    let bless = std::env::var_os("EPITOME_UI_BLESS").is_some();
    let filter = std::env::var("EPITOME_UI_FILTER").unwrap_or_default();

    let mut files = Vec::new();
    collect(&ui, &mut files);
    files.sort();
    let mut tests = Vec::new();
    // Every test that does not pass yet, with why: left out, or failed.
    let mut todo: BTreeMap<String, String> = BTreeMap::new();
    let mut left_out: BTreeMap<&str, usize> = BTreeMap::new();
    let mut ignored = 0;
    for f in &files {
        if !f.to_string_lossy().contains(&filter) {
            continue;
        }
        for (name, what) in select(&ui, f) {
            match what {
                Selected::Run(t) => tests.push(t),
                Selected::Ignored => ignored += 1,
                Selected::LeftOut(why) => {
                    *left_out.entry(why).or_default() += 1;
                    todo.insert(name, format!("left out: {why}"));
                }
            }
        }
    }
    let t = std::time::Instant::now();
    let refs: Vec<&Test> = tests.iter().collect();
    let mut verdicts = Vec::new();
    for chunk in refs.chunks(BATCH) {
        run_isolated(&library, chunk, &mut verdicts);
    }
    let elapsed = t.elapsed();

    let mut by_reason: BTreeMap<String, usize> = BTreeMap::new();
    for (t, v) in tests.iter().zip(&verdicts) {
        if let Some(why) = v {
            todo.insert(t.rel.clone(), why.clone());
            // The stage and its first words, to group alike failures.
            let key: String = why.split(" (").next().unwrap().split_whitespace().take(6).collect::<Vec<_>>().join(" ");
            *by_reason.entry(key).or_default() += 1;
        }
    }
    let failed = verdicts.iter().filter(|v| v.is_some()).count();
    eprintln!(
        "{} check-pass tests: {} pass, {failed} fail, {} left out ({ignored} not for this host), in {:.1} s",
        tests.len() + todo.len() - failed,
        tests.len() - failed,
        todo.len() - failed,
        elapsed.as_secs_f64(),
    );
    eprintln!("left out: {left_out:?}");
    let mut reasons: Vec<_> = by_reason.into_iter().collect();
    reasons.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    for (why, n) in reasons.iter().take(30) {
        eprintln!("  {n:6} {why}");
    }

    if bless {
        assert!(filter.is_empty(), "EPITOME_UI_BLESS writes the whole list: unset EPITOME_UI_FILTER");
        let mut out = format!(
            "# rust-lang/rust {head}\n\
             # The check-pass UI tests that do not pass yet, and why (tests/ui_check_pass.rs).\n"
        );
        for (p, why) in &todo {
            out.push_str(&format!("{p}: {why}\n"));
        }
        std::fs::write(&list_path, out).unwrap();
        eprintln!("wrote {}", list_path.display());
        return;
    }
    if pinned != head {
        eprintln!("the list is for rust-lang/rust {pinned:?}, the checkout is {head:?}: not compared");
        return;
    }
    // Only the paths are compared: a test may fail for a new reason.
    let listed: BTreeSet<&str> = list
        .lines()
        .filter(|l| !l.starts_with('#') && !l.is_empty())
        .map(|l| l.split_once(": ").map_or(l, |x| x.0))
        .filter(|p| ui.join(p).to_string_lossy().contains(&filter))
        .collect();
    let now_fail: Vec<String> =
        todo.iter().filter(|(p, _)| !listed.contains(p.as_str())).map(|(p, why)| format!("  {p}: {why}")).collect();
    let now_pass: Vec<&str> = listed.iter().copied().filter(|p| !todo.contains_key(*p)).collect();
    let mut problems = String::new();
    if !now_fail.is_empty() {
        problems += &format!("{} tests that passed now fail:\n{}\n", now_fail.len(), now_fail.join("\n"));
    }
    if !now_pass.is_empty() {
        problems += &format!(
            "{} tests pass now (EPITOME_UI_BLESS=1 takes them off the list):\n  {}\n",
            now_pass.len(),
            now_pass.join("\n  ")
        );
    }
    assert!(problems.is_empty(), "{problems}");
}
