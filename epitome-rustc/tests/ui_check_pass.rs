//! rustc's `check-pass` UI tests: programs rustc accepts, so they must
//! parse, expand, resolve and type check here without errors.
//!
//! The same machinery measures the borrow checker (`borrowck_expected_errors
//! _are_found`, `tests/ui_borrowck_todo.txt`): the tests that expect errors,
//! all of them borrow-checking ones (`epitome_rustc::borrowck::CODES`), whose
//! `.stderr` says which (code and line). Each must type check, and
//! `Session::borrowck` must give those errors and no others.
//!
//! Set `EPITOME_RUST` to a checkout of rust-lang/rust with `tests/ui`,
//! `tests/auxiliary` and `library` (a sparse checkout of those is
//! enough), at the commit named
//! in `tests/ui_check_pass_todo.txt`; without it the test is skipped. The
//! tests chosen are loaded, a few hundred at a time, as crates of one
//! session beside `core`, `alloc` and `std` from the checkout's `library`.
//! Each test crate has its own configuration (`--cfg`) and may use only
//! those three, so that tests cannot see each other. Each session runs in
//! a child process, so that a test that crashes or hangs the compiler is
//! found (by halving) and fails on its own.
//!
//! A test's auxiliary crates (`//@ aux-build`, `//@ aux-crate`,
//! `//@ add-minicore`) are crates of the session too, by their names; a
//! batch never has two different crates of one name. Lang items are the
//! session's, so a test that defines them, or whose auxiliary crates do
//! (`minicore`), runs in a session of its own.
//!
//! Proc-macro crates (`//@ proc-macro`, or an auxiliary crate of that
//! type) are built by rustc, run as a program, and expanded through the
//! toolchain's proc-macro server; `proc_macro` itself is loaded for the
//! tests that name it.
//!
//! Flags that change the configuration (`--target`, `-C panic=abort`,
//! `-Zsanitizer=...`) are given to `rustc --print cfg`, and the test's
//! crates get what it prints: so tests for other targets run too.
//!
//! Each revision of a test with `//@ revisions` is a test of its own,
//! `path#rev`, compiled with `--cfg rev` and the directives for it
//! (`//@[rev] ...`) as well as the file's.
//!
//! `tests/ui_check_pass_todo.txt` is the work left: every `check-pass`
//! test that does not pass yet, with why. Those that fail say at which
//! stage and the first error; those left out say what they need that the
//! runner cannot give yet (`--extern`, `--test`).
//! Tests for other hosts (`ignore-*`, `only-*`) are not listed. The list
//! is for one rust-lang/rust commit and one host, named on its first line.
//!
//! The test fails if a test not on the list fails, one on it now crashes
//! the compiler (where its reason was not a crash), or one on it passes:
//! `EPITOME_UI_BLESS=1` writes the list again, so a change that fixes
//! tests takes them off in the same commit. `EPITOME_UI_FILTER=substring`
//! runs only the tests whose path has it in (`EPITOME_UI_FILTER=/ui/fmt/`).

use epitome_rustc::cfg::CfgSet;
use epitome_rustc::lexer::Edition;
use epitome_rustc::session::{CrateRoot, Options, Session};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;

/// A test to run: its path under `tests/ui` (with `#revision` for one
/// revision of a test with several), the edition it asks for, the
/// options it sets with `--cfg` (`name` or `name="value"`; and
/// `rustc:flag` for a flag that changes the configuration, such as
/// `--target` or `-C panic=abort`), and its
/// auxiliary crates: those it uses itself (`deps`) and all those they
/// need (`aux`).
struct Test {
    rel: String,
    path: PathBuf,
    edition: Edition,
    cfg: Vec<String>,
    deps: Vec<String>,
    aux: Vec<Aux>,
    /// The proc-macro crates it uses: names and libraries built by rustc.
    pm: Vec<(String, PathBuf)>,
    /// It or one of its auxiliary crates defines lang items, which are
    /// the whole session's: it runs in a session of its own.
    alone: bool,
}

/// An auxiliary crate (`//@ aux-build`, `//@ aux-crate`): its name, file,
/// edition, `--cfg` options and the auxiliary crates it uses.
#[derive(Clone, PartialEq)]
struct Aux {
    name: String,
    path: PathBuf,
    edition: Edition,
    cfg: Vec<String>,
    deps: Vec<String>,
    /// The proc-macro crates it uses.
    pm: Vec<(String, PathBuf)>,
    /// It defines lang items (`#![no_core]` crates such as `minicore`).
    lang: bool,
}

/// The crates every test and auxiliary crate may use (and `proc_macro`,
/// loaded only for tests that name it).
/// What the tests are for.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Want {
    /// `check-pass`: accepted.
    Pass,
    /// Tests that expect borrow-checking errors: found exactly.
    Borrowck,
    /// Tests with `#[rustc_dump_variances]`: the variances printed are
    /// rustc's.
    Variance,
}

impl Want {
    fn from_env() -> Want {
        match std::env::var("EPITOME_UI_WANT").as_deref() {
            Ok("borrowck") => Want::Borrowck,
            Ok("variance") => Want::Variance,
            _ => Want::Pass,
        }
    }

    fn name(self) -> &'static str {
        match self {
            Want::Pass => "check-pass",
            Want::Borrowck => "borrowck",
            Want::Variance => "variance",
        }
    }
}

/// The variances a test expects, from its `.stderr`: each as (line, text),
/// for `error: ['a: +, T: o]` lines. `None` if it prints none.
fn expected_variances(path: &Path, rev: Option<&str>) -> Option<Vec<(usize, String)>> {
    let stem = path.file_stem()?.to_string_lossy().into_owned();
    let stderr = match rev {
        Some(r) => path.with_file_name(format!("{stem}.{r}.stderr")),
        None => path.with_extension("stderr"),
    };
    let text = std::fs::read_to_string(stderr).ok()?;
    let mut out = Vec::new();
    let mut pending: Option<String> = None;
    for line in text.lines() {
        if let Some(rest) = line.strip_prefix("error: ") {
            pending = (rest.starts_with('[') && rest.ends_with(']')).then(|| rest.to_string());
        } else if let Some(at) = line.trim_start().strip_prefix("--> ")
            && let Some(t) = pending.take()
        {
            let mut f = at.rsplitn(3, ':');
            let (_col, line_no) = (f.next()?, f.next()?);
            out.push((line_no.parse().ok()?, t));
        }
    }
    (!out.is_empty()).then_some(out)
}

/// The errors a test expects, from its `.stderr` (the revision's, if it has
/// one): each as (error code, line). `None` unless it expects errors and
/// all of them are borrow-checking ones in the test's own file.
fn expected_errors(path: &Path, rev: Option<&str>) -> Option<Vec<(String, usize)>> {
    let stem = path.file_stem()?.to_string_lossy().into_owned();
    let name = path.file_name()?.to_string_lossy().into_owned();
    let stderr = match rev {
        Some(r) => path.with_file_name(format!("{stem}.{r}.stderr")),
        None => path.with_extension("stderr"),
    };
    let text = std::fs::read_to_string(stderr).ok()?;
    let mut out = Vec::new();
    // The code of the error whose `-->` line is next.
    let mut pending: Option<String> = None;
    for line in text.lines() {
        if let Some(rest) = line.strip_prefix("error[") {
            pending = Some(rest.split(']').next()?.to_string());
        } else if line.starts_with("error") {
            // An error with no code: not a borrow-checking one, except the
            // closing count.
            if !line.starts_with("error: aborting due to") && !line.starts_with("error: could not compile") {
                return None;
            }
            pending = None;
        } else if line.starts_with("warning") {
            pending = None;
        } else if let Some(at) = line.trim_start().strip_prefix("--> ")
            && let Some(code) = pending.take()
        {
            // `$DIR/name.rs:LINE:COL`: in this test's own file.
            let mut f = at.rsplitn(3, ':');
            let (_col, line_no, file) = (f.next()?, f.next()?, f.next()?);
            if file.rsplit('/').next()? != name {
                return None;
            }
            out.push((code, line_no.parse().ok()?));
        }
    }
    let codes = epitome_rustc::borrowck::CODES;
    (!out.is_empty() && out.iter().all(|(c, _)| codes.contains(&c.as_str()))).then_some(out)
}

const STD: [&str; 4] = ["core", "alloc", "std", "proc_macro"];

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
fn select(ui: &Path, path: &Path, want: Want) -> Vec<(String, Selected)> {
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
        return select_one(path, rel, &all, None, want).into_iter().collect();
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
            select_one(path, format!("{rel}#{rev}"), &mine, Some(rev), want)
        })
        .collect()
}

/// What one test (or revision) whose directives are `directives` needs;
/// `None` if it is not a `check-pass` test.
fn select_one(path: &Path, name: String, directives: &[(&str, &str)], rev: Option<&str>, want: Want) -> Option<(String, Selected)> {
    let mode = directives.iter().rev().find(|d| MODES.contains(&d.0)).map(|d| d.0);
    match want {
        Want::Pass => {
            if mode != Some("check-pass") {
                return None;
            }
        }
        // No mode is a test of errors; so are `check-fail` and `build-fail`.
        Want::Borrowck => {
            if !matches!(mode, None | Some("check-fail") | Some("build-fail")) || expected_errors(path, rev).is_none() {
                return None;
            }
        }
        Want::Variance => {
            if expected_variances(path, rev).is_none() || !std::fs::read_to_string(path).is_ok_and(|t| t.contains("rustc_dump_variances")) {
                return None;
            }
        }
    }
    let mut ignored = false;
    // The first reason the test cannot run here.
    let mut left_out: Option<&'static str> = None;
    let mut edition = Edition::E2015;
    let mut cfg: Vec<String> = rev.map(str::to_string).into_iter().collect();
    // Auxiliary crates, as (name, file).
    // Auxiliary crates: name, file, and whether `//@ proc-macro` made it one.
    let mut aux_files: Vec<(String, String, bool)> = Vec::new();
    for &(name, value) in directives {
        match name {
            "aux-build" | "aux-crate" | "proc-macro" => match aux_spec(name, value) {
                Some((n, f)) => {
                    // `#![crate_name = "Vec"]` names an `aux-build` crate.
                    let n = match name {
                        "aux-build" => crate_name_attr(&path.parent().unwrap().join("auxiliary").join(&f)).unwrap_or(n),
                        _ => n,
                    };
                    aux_files.push((n, f, name == "proc-macro"))
                }
                None => _ = left_out.get_or_insert("auxiliary crates"),
            },
            // `tests/auxiliary/minicore.rs`, a `core` of lang items.
            "add-minicore" => {
                let tests = path.ancestors().find(|a| a.file_name().is_some_and(|n| n == "ui")).and_then(Path::parent);
                let minicore = tests.map(|t| t.join("auxiliary/minicore.rs"));
                aux_files.push(("minicore".into(), minicore.map_or(String::new(), |m| m.display().to_string()), false));
            }
            "aux-bin" | "aux-codegen-backend" => _ = left_out.get_or_insert("auxiliary binaries"),
            "edition" => {
                // `2021`, or a range `2018..` / `2015..2021`: its start;
                // `future`, the latest.
                let start: String = value.chars().take_while(char::is_ascii_digit).collect();
                match Edition::parse(&start) {
                    Some(e) => edition = e,
                    None if value == "future" => edition = Edition::E2024,
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
                        // The test harness's `main` is not looked at here.
                        cfg.push("test".into());
                    } else if let Some(e) = arg("--extern") {
                        // A test's auxiliary crates are its deps already;
                        // a name without a crate is not looked for.
                        let _ = e;
                    } else if let Some(a) = arg("-Zcrate-attr") {
                        // `#![feature(..)]`, `#![crate_type = ".."]` change
                        // nothing here; `#![cfg(..)]` could remove the crate.
                        if a.contains("cfg(") {
                            left_out.get_or_insert("flags that change the crate");
                        }
                    } else if let Some(t) = arg("--target") {
                        if t.ends_with(".json") {
                            left_out.get_or_insert("a target rustc does not know");
                        }
                        cfg.push(format!("rustc:--target={t}"));
                    } else if f == "-O" {
                        cfg.push("rustc:-O".into());
                    } else if let Some(c) = arg("-C") {
                        cfg.push(format!("rustc:-C{c}"));
                    } else if let Some(z) = arg("-Z") {
                        cfg.push(format!("rustc:-Z{z}"));
                    } else if f.starts_with("--crate-type")
                        && (f.contains("proc-macro") || flags.clone().next() == Some("proc-macro"))
                    {
                        left_out.get_or_insert("proc-macro crate");
                    }
                }
            }
            // Enzyme (`-Zautodiff`) is not in a rustc built without it.
            "needs-enzyme" => ignored = true,
            _ => {
                if let Some(x) = name.strip_prefix("ignore-") {
                    ignored |= x == "test" || host_is(x);
                } else if let Some(x) = name.strip_prefix("only-") {
                    ignored |= !host_is(x);
                }
            }
        }
    }
    // Flags that change the configuration (`--target`, `-C`, `-Z`) are
    // given to `rustc --print cfg`. If rustc does not take them, the test
    // is left out if one names a target, and the host's configuration is
    // used if not.
    if cfg.iter().any(|c| c.starts_with("rustc:")) && rustc_cfg(&cfg).is_none() {
        if cfg.iter().any(|c| c.starts_with("rustc:--target=")) {
            left_out.get_or_insert("a target rustc does not know");
        }
        // A sanitizer this host does not have (`needs-sanitizer-leak`).
        if cfg.iter().any(|c| c.starts_with("rustc:-Zsanitizer")) {
            ignored = true;
        }
        cfg.retain(|c| !c.starts_with("rustc:"));
    }
    let aux_dir = path.parent().unwrap().join("auxiliary");
    let mut aux = Vec::new();
    let mut pm = Vec::new();
    if left_out.is_none() {
        match aux_crates(&aux_dir, &aux_files, &base_cfg(rev, &cfg), &mut aux, 0) {
            Ok(p) => pm = p,
            Err(why) => left_out = Some(why),
        }
    }
    let what = if ignored {
        Selected::Ignored
    } else if let Some(why) = left_out {
        Selected::LeftOut(why)
    } else {
        let deps =
            aux_files.into_iter().filter(|a| !pm.iter().any(|p: &(String, PathBuf)| p.0 == a.0)).map(|a| a.0).collect();
        let lang = std::fs::read_to_string(path).is_ok_and(|t| defines_lang_items(&t));
        let alone = lang || aux.iter().any(|a| a.lang);
        Selected::Run(Test { rel: name.clone(), path: path.to_path_buf(), edition, cfg, deps, aux, pm, alone })
    };
    Some((name, what))
}

/// Whether a crate's source defines lang items.
fn defines_lang_items(text: &str) -> bool {
    text.contains("#[lang =") || text.contains("#[lang=")
}

/// What a test's auxiliary crates are built with: its revision's `--cfg`
/// and its target.
fn base_cfg(rev: Option<&str>, cfg: &[String]) -> Vec<String> {
    let mut base: Vec<String> = rev.map(str::to_string).into_iter().collect();
    base.extend(cfg.iter().filter(|c| c.starts_with("rustc:--target=")).cloned());
    base
}

/// `rustc --print cfg` with the flags among `options` (`rustc:flag`; run
/// once for each set), or `None` if rustc does not take them.
fn rustc_cfg(options: &[String]) -> Option<String> {
    static CACHE: std::sync::Mutex<BTreeMap<Vec<String>, Option<String>>> = std::sync::Mutex::new(BTreeMap::new());
    let flags: Vec<String> = options.iter().filter_map(|c| c.strip_prefix("rustc:")).map(str::to_string).collect();
    let mut cache = CACHE.lock().unwrap();
    cache
        .entry(flags.clone())
        .or_insert_with(|| {
            let out = Command::new("rustc")
                .args(["--print", "cfg", "-Zunstable-options"])
                .args(&flags)
                .env("RUSTC_BOOTSTRAP", "1")
                .output()
                .ok()?;
            out.status.success().then(|| String::from_utf8_lossy(&out.stdout).into_owned())
        })
        .clone()
}

/// The name and file of `//@ aux-build: file.rs` (the file's name, with
/// `-` as `_`) or `//@ aux-crate: [options:]name=file.rs`.
fn aux_spec(directive: &str, value: &str) -> Option<(String, String)> {
    if directive == "aux-crate" {
        let (name, file) = value.split_once('=')?;
        // Options before a single `:` (`noprelude:name`); a namespaced
        // crate's name has `::` in it (`my_api::utils`).
        let mut name = name.trim();
        while let Some(i) = name.find(':') {
            if name[i + 1..].starts_with(':') {
                break;
            }
            name = &name[i + 1..];
        }
        return Some((name.to_string(), file.trim().to_string()));
    }
    let file = value.trim();
    let stem = Path::new(file).file_stem()?.to_string_lossy().replace('-', "_");
    Some((stem, file.to_string()))
}

/// The name a crate gives itself, `#![crate_name = "name"]`.
fn crate_name_attr(path: &Path) -> Option<String> {
    let text = std::fs::read_to_string(path).ok()?;
    let rest = text.split("#![crate_name").nth(1)?;
    let value = rest.trim_start().strip_prefix('=')?.trim_start().strip_prefix('"')?;
    Some(value.split('"').next()?.to_string())
}

/// Adds the auxiliary crates `files` (from `dir`) to `out`, with those
/// they use in turn, each once, and gives the proc-macro crates among
/// `files`, built by rustc. A crate name used for two different crates
/// leaves the test out.
fn aux_crates(
    dir: &Path,
    files: &[(String, String, bool)],
    base: &[String],
    out: &mut Vec<Aux>,
    depth: usize,
) -> Result<Vec<(String, PathBuf)>, &'static str> {
    if depth > 8 {
        return Err("auxiliary crates");
    }
    let mut pm = Vec::new();
    for (name, file, proc_macro) in files {
        let path = dir.join(file);
        let text = std::fs::read_to_string(&path).map_err(|_| "auxiliary crates")?;
        let mut edition = Edition::E2015;
        let mut cfg: Vec<String> = base.to_vec();
        let mut nested = Vec::new();
        let mut is_pm = *proc_macro || text.contains("crate_type = \"proc-macro\"");
        for line in text.lines() {
            let Some(d) = line.trim_start().strip_prefix("//@") else { continue };
            let (n, v) = d.split_once(':').map_or((d.trim(), ""), |(n, v)| (n.trim(), v.trim()));
            match n {
                "edition" => {
                    let start: String = v.chars().take_while(char::is_ascii_digit).collect();
                    edition = Edition::parse(&start).ok_or("edition")?;
                }
                "aux-build" | "aux-crate" | "proc-macro" => {
                    let (nn, nf) = aux_spec(n, v).ok_or("auxiliary crates")?;
                    let nn = if n == "aux-build" { crate_name_attr(&dir.join(&nf)).unwrap_or(nn) } else { nn };
                    nested.push((nn, nf, n == "proc-macro"));
                }
                "compile-flags" => {
                    let mut flags = v.split_whitespace();
                    while let Some(f) = flags.next() {
                        if f == "--cfg" {
                            cfg.extend(flags.next().map(|c| c.trim_matches('\'').to_string()));
                        } else if let Some(c) = f.strip_prefix("--cfg=") {
                            cfg.push(c.trim_matches('\'').to_string());
                        } else if f == "--edition" {
                            edition = flags.next().and_then(Edition::parse).ok_or("edition")?;
                        } else if let Some(e) = f.strip_prefix("--edition=") {
                            edition = Edition::parse(e).ok_or("edition")?;
                        } else if f.contains("proc-macro") {
                            is_pm = true;
                        }
                    }
                }
                _ => {}
            }
        }
        if is_pm {
            // Built for the host by rustc, and run by its proc-macro server.
            if !nested.is_empty() {
                return Err("proc-macro crates with auxiliary crates");
            }
            pm.push((name.clone(), build_proc_macro(&path, name, edition)?));
            continue;
        }
        let nested_pm = aux_crates(dir, &nested, base, out, depth + 1)?;
        let deps = nested.into_iter().filter(|n| !nested_pm.iter().any(|p| p.0 == n.0)).map(|n| n.0).collect();
        let a = Aux { name: name.clone(), path, edition, cfg, deps, pm: nested_pm, lang: defines_lang_items(&text) };
        match out.iter().find(|x| x.name == a.name) {
            Some(x) if *x != a => return Err("auxiliary crates with the same name"),
            Some(_) => {}
            None => out.push(a),
        }
    }
    Ok(pm)
}

/// Builds the proc-macro crate at `path` with rustc (once each), for its
/// server to load.
fn build_proc_macro(path: &Path, name: &str, edition: Edition) -> Result<PathBuf, &'static str> {
    static BUILT: std::sync::Mutex<BTreeMap<(PathBuf, &'static str), Option<PathBuf>>> =
        std::sync::Mutex::new(BTreeMap::new());
    let mut built = BUILT.lock().unwrap();
    let n = built.len();
    built
        .entry((path.to_path_buf(), year(edition)))
        .or_insert_with(|| {
            let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("ui_proc_macros/{n}"));
            std::fs::create_dir_all(&dir).ok()?;
            let lib = dir.join(format!("lib{name}.{}", std::env::consts::DLL_EXTENSION));
            let ok = Command::new("rustc")
                .args(["--crate-type", "proc-macro", "--edition", year(edition), "--crate-name", name, "-o"])
                .arg(&lib)
                .arg(path)
                .env("RUSTC_BOOTSTRAP", "1")
                .stderr(std::process::Stdio::null())
                .status()
                .is_ok_and(|s| s.success());
            ok.then_some(lib)
        })
        .clone()
        .ok_or("proc-macro crates rustc cannot build")
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

/// A crate's configuration: the host's, or with rustc flags among the
/// options (`rustc:--target=x`, `rustc:-Cpanic=abort`) what `rustc --print
/// cfg` gives with them, with `--cfg` options added.
fn cfg_with(options: &[String]) -> Option<CfgSet> {
    if options.is_empty() {
        return None;
    }
    let printed = options.iter().any(|c| c.starts_with("rustc:")).then(|| rustc_cfg(options)).flatten();
    let mut cfg = match printed {
        Some(text) => CfgSet::from_print_cfg(&text),
        None => CfgSet::host(),
    };
    for c in options.iter().filter(|c| !c.starts_with("rustc:")) {
        // `--cfg r#false` sets `false`.
        let c = c.strip_prefix("r#").unwrap_or(c);
        match c.split_once('=') {
            Some((n, v)) => cfg.set_value(n, v.trim_matches('"')),
            None => cfg.set(c),
        };
    }
    Some(cfg)
}

/// A crate that may use the standard crates and `deps`.
fn crate_root(
    name: String,
    path: &Path,
    edition: Edition,
    cfg: &[String],
    deps: &[String],
    pm: &[(String, PathBuf)],
) -> CrateRoot {
    let mut all: Vec<String> = STD.map(String::from).to_vec();
    all.extend(deps.iter().cloned());
    let mut root = CrateRoot::new(name, path).edition(edition).deps(all);
    for (n, lib) in pm {
        root = root.proc_macro(n.clone(), lib.clone());
    }
    match cfg_with(cfg) {
        Some(c) => root.cfg(c),
        None => root,
    }
}

/// Whether source text names the crate `core` (`core::`, not as the end
/// of another name such as `minicore::`).
fn names_core(text: &str) -> bool {
    text.match_indices("core::")
        .any(|(i, _)| !text[..i].chars().next_back().is_some_and(|c| c.is_alphanumeric() || c == '_'))
}

/// How far rustc gets with a test's flags: the stages after it are not
/// checked.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Stage {
    /// `-Zparse-crate-root-only`, `-Zunpretty=normal` and the like.
    Parse,
    /// `-Zunpretty=expanded`.
    Expand,
    /// `-Zunpretty=hir`.
    Resolve,
    Check,
}

impl Stage {
    fn of(cfg: &[String]) -> Stage {
        let mut stage = Stage::Check;
        for c in cfg {
            if c == "rustc:-Zparse-crate-root-only" {
                stage = stage.min(Stage::Parse);
            } else if let Some(u) = c.strip_prefix("rustc:-Zunpretty=") {
                let s = if u.contains("expanded") {
                    Stage::Expand
                } else if matches!(u, "normal" | "identified" | "ast-tree") {
                    Stage::Parse
                } else if matches!(u, "hir" | "hir-tree" | "hir,identified") {
                    Stage::Resolve
                } else {
                    Stage::Check
                };
                stage = stage.min(s);
            }
        }
        stage
    }
}

/// Runs `tests` in one session: for each, `None` if it passed, or why not.
/// The tests' auxiliary crates are loaded once each, by name (the batch
/// has no two different crates of one name).
fn run(library: &Path, tests: &[&Test], want: Want) -> Vec<Option<String>> {
    // The toolchain's proc-macro server, if any crate uses proc macros.
    let uses_pm = tests.iter().any(|t| !t.pm.is_empty() || t.aux.iter().any(|a| !a.pm.is_empty()));
    let server = if uses_pm { epitome_rustc::proc_macro::Server::from_rustc() } else { None };
    let session = Session::new(Options { cfg: Some(CfgSet::host()), proc_macro_server: server, ..Default::default() });
    // Tests whose crates are all `#![no_core]` (over `minicore`) need no
    // standard library, which is most of a session's work: unless they
    // name `core` or derive (which writes `::core` paths), as rustc finds
    // `core` in the sysroot even then.
    let no_core = |p: &Path| {
        std::fs::read_to_string(p).is_ok_and(|t| t.contains("#![no_core]") && !names_core(&t) && !t.contains("derive"))
    };
    let standalone = tests.iter().all(|t| no_core(&t.path) && t.aux.iter().all(|a| no_core(&a.path)));
    let mut std_crates: Vec<&str> = if standalone { Vec::new() } else { STD[..3].to_vec() };
    // `proc_macro` too, from the sysroot as rustc finds it, if a test or
    // auxiliary crate names it.
    let names_pm = |p: &Path| std::fs::read_to_string(p).is_ok_and(|t| t.contains("proc_macro"));
    if !standalone && tests.iter().any(|t| names_pm(&t.path) || t.aux.iter().any(|a| names_pm(&a.path))) {
        std_crates.push("proc_macro");
    }
    let mut roots: Vec<CrateRoot> = std_crates
        .iter()
        .map(|n| CrateRoot::new(*n, library.join(n).join("src/lib.rs")).edition(Edition::E2024))
        .collect();
    let mut aux_index: BTreeMap<&str, usize> = BTreeMap::new();
    for a in tests.iter().flat_map(|t| &t.aux) {
        if !aux_index.contains_key(a.name.as_str()) {
            aux_index.insert(&a.name, roots.len());
            roots.push(crate_root(a.name.clone(), &a.path, a.edition, &a.cfg, &a.deps, &a.pm));
        }
    }
    let first = roots.len();
    // Names no program could mean, so that one test's paths cannot find
    // another test.
    roots.extend(
        tests
            .iter()
            .enumerate()
            .map(|(i, t)| crate_root(format!("__ui_test_{i}"), &t.path, t.edition, &t.cfg, &t.deps, &t.pm)),
    );
    let mut crates = session.parse_crates(roots);
    // `-Zparse-crate-root-only`: the test is of the parser only.
    let parse_errors: Vec<Option<String>> = crates
        .iter()
        .map(|c| c.errors.first().map(|e| e.render_in(&session.sources).lines().next().unwrap_or("").to_string()))
        .collect();
    let expanded = session.expand(&mut crates);
    let r = session.resolve(&crates);
    let tcx = session.collect_types(&crates, &r);
    let res = session.typeck(&crates, &r, &tcx);
    // The borrow checker, over the tests' crates.
    // (Also for a `check-pass` test, where it must find nothing.)
    let only: Vec<u32> = (first as u32..crates.len() as u32).collect();
    let diagnostics = if want == Want::Variance { Vec::new() } else { session.borrowck(&crates, &r, &tcx, &res, &only) };
    let dumped = if want == Want::Variance { session.dump_variances(&crates, &r, &tcx, &only) } else { Vec::new() };
    // How far rustc goes: `-Zunpretty=expanded` stops after expansion,
    // `-Zunpretty=hir` after name resolution (see `Stage`).
    let verdict = |k: usize, stage: Stage| {
        if stage == Stage::Parse {
            return parse_errors[k].clone();
        }
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
        if stage == Stage::Expand {
            return None;
        }
        if let Some(i) = r.failed_import_lines_of(k as u32).first() {
            return Some(format!("import not resolved: {i}"));
        }
        let s = r.crate_stats(&crates, k);
        if s.unresolved > 0 {
            return Some(format!("path not resolved: {}", s.unresolved_names[0].0));
        }
        if stage == Stage::Resolve {
            return None;
        }
        let t = &res.crate_stats[k];
        if t.clean_bodies < t.bodies {
            let why = t.reasons.first().map_or("a type left unknown", |w| w.0.as_str());
            return Some(format!("type check: {why}"));
        }
        None
    };
    tests
        .iter()
        .enumerate()
        .map(|(i, t)| {
            // An auxiliary crate that fails fails the test.
            let stage = match want {
                Want::Borrowck => Stage::Check,
                Want::Variance => Stage::Resolve,
                Want::Pass => Stage::of(&t.cfg),
            };
            for a in &t.aux {
                if let Some(why) = verdict(aux_index[a.name.as_str()], Stage::Check) {
                    return Some(format!("auxiliary crate `{}`: {why}", a.name));
                }
            }
            let v = verdict(first + i, stage);
            if want == Want::Pass {
                // The borrow checker never rejects what rustc accepts (if it
                // goes as far, which a test that only prints does not).
                if v.is_none() && stage == Stage::Check {
                    // Every region of a signature is read.
                    let me = (first + i) as u32;
                    for (j, def) in r.defs.iter().enumerate() {
                        if def.krate != me {
                            continue;
                        }
                        if let Some(f) = epitome_rustc::regions::sig::fn_regions(&crates, &r, &tcx, epitome_rustc::resolve::DefId(j as u32)) {
                            let mut regs = Vec::new();
                            f.output.regions(&mut regs);
                            f.inputs.iter().for_each(|t| t.regions(&mut regs));
                            if regs.contains(&epitome_rustc::regions::sig::Region::Unknown) {
                                return Some(format!("regions: `{}` has a lifetime left out: {}", r.def_path(epitome_rustc::resolve::DefId(j as u32)), f.show(&r, &tcx)));
                            }
                        }
                    }
                    let found: Vec<String> =
                        diagnostics.iter().filter(|d| d.krate as usize == first + i).map(|d| format!("{}@{}", d.code, d.line)).collect();
                    if !found.is_empty() {
                        return Some(format!("borrowck rejects it ({})", found.join(" ")));
                    }
                }
                return v;
            }
            // A test of the borrow checker must get to it.
            if let Some(why) = v {
                return Some(format!("before borrowck: {why}"));
            }
            let rev = t.rel.split_once('#').map(|x| x.1);
            if want == Want::Variance {
                let mut expected = expected_variances(&t.path, rev).unwrap_or_default();
                let mut got: Vec<(usize, String)> =
                    dumped.iter().filter(|d| d.krate as usize == first + i).map(|d| (d.line, d.text.clone())).collect();
                expected.sort();
                got.sort();
                if expected == got {
                    return None;
                }
                let show = |v: &[(usize, String)]| v.iter().map(|(l, t)| format!("{l}:{t}")).collect::<Vec<_>>().join(" ");
                return Some(format!("wrong variances (expected {}; got {})", show(&expected), show(&got)));
            }
            let mut expected = expected_errors(&t.path, rev).unwrap_or_default();
            let mut got: Vec<(String, usize)> =
                diagnostics.iter().filter(|d| d.krate as usize == first + i).map(|d| (d.code.to_string(), d.line)).collect();
            expected.sort();
            got.sort();
            if expected == got {
                return None;
            }
            // Each side's errors the other lacks.
            let (mut missing, mut extra) = (expected.clone(), got.clone());
            missing.retain(|e| { let k = got.iter().position(|g| g == e); if let Some(k) = k { got.remove(k); false } else { true } });
            let mut left = expected.clone();
            extra.retain(|g| { let k = left.iter().position(|e| e == g); if let Some(k) = k { left.remove(k); false } else { true } });
            let show = |v: &[(String, usize)]| v.iter().map(|(c, l)| format!("{c}@{l}")).collect::<Vec<_>>().join(" ");
            Some(if extra.is_empty() {
                format!("no diagnostics (missing {})", show(&missing))
            } else {
                format!("wrong diagnostics (missing {}; extra {})", show(&missing), show(&extra))
            })
        })
        .collect()
}

/// Runs `tests` in a child process (this test again, with the tests in
/// `EPITOME_UI_BATCH`), so that a test that crashes the compiler (a panic,
/// a stack overflow) or makes it hang cannot take the others with it.
/// When the child fails, the tests are split in two and run again, until
/// the test that fails is found.
fn run_isolated(library: &Path, tests: &[&Test], out: &mut Vec<Option<String>>, want: Want) {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"));
    let (batch, results) = (dir.join(format!("ui_{}.batch", want.name())), dir.join(format!("ui_{}.results", want.name())));
    let mut list = String::new();
    for t in tests {
        let line = |kind: &str, name: &str, e: Edition, path: &Path, cfg: &[String], deps: &[String]| {
            format!("{kind}\t{name}\t{}\t{}\t{}\t{}\n", year(e), path.display(), cfg.join("\x1f"), deps.join("\x1f"))
        };
        // A proc-macro crate (`M name lib`) belongs to the line before.
        let macros =
            |pm: &[(String, PathBuf)]| pm.iter().map(|(n, l)| format!("M\t{n}\t{}\n", l.display())).collect::<String>();
        list += &line("T", &t.rel, t.edition, &t.path, &t.cfg, &t.deps);
        list += &macros(&t.pm);
        for a in &t.aux {
            list += &line("A", &a.name, a.edition, &a.path, &a.cfg, &a.deps);
            list += &macros(&a.pm);
        }
    }
    std::fs::write(&batch, list).unwrap();
    let _ = std::fs::remove_file(&results);
    let mut child = Command::new(std::env::current_exe().unwrap())
        .args(["check_pass_tests_are_accepted", "--exact", "--nocapture"])
        .env("EPITOME_UI_BATCH", &batch)
        .env("EPITOME_UI_WANT", want.name())
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
    run_isolated(library, a, out, want);
    run_isolated(library, b, out, want);
}

/// In a child process: runs the tests `EPITOME_UI_BATCH` lists and writes
/// a line for each to `EPITOME_UI_RESULTS`, `ok` or `fail\twhy`.
fn child(batch: &Path) {
    let library = PathBuf::from(std::env::var_os("EPITOME_RUST_LIBRARY").unwrap());
    let results = PathBuf::from(std::env::var_os("EPITOME_UI_RESULTS").unwrap());
    let mut tests: Vec<Test> = Vec::new();
    for l in std::fs::read_to_string(batch).unwrap().lines() {
        let f: Vec<&str> = l.split('\t').collect();
        if f[0] == "M" {
            let t = tests.last_mut().unwrap();
            let pm = (f[1].to_string(), PathBuf::from(f[2]));
            match t.aux.last_mut() {
                Some(a) => a.pm.push(pm),
                None => t.pm.push(pm),
            }
            continue;
        }
        let list = |x: &str| x.split('\x1f').filter(|c| !c.is_empty()).map(str::to_string).collect::<Vec<_>>();
        let (name, edition, path) = (f[1].to_string(), Edition::parse(f[2]).unwrap(), PathBuf::from(f[3]));
        if f[0] == "T" {
            let (cfg, deps) = (list(f[4]), list(f[5]));
            tests.push(Test { rel: name, path, edition, cfg, deps, aux: Vec::new(), pm: Vec::new(), alone: false });
        } else {
            let a = Aux { name, path, edition, cfg: list(f[4]), deps: list(f[5]), pm: Vec::new(), lang: false };
            tests.last_mut().unwrap().aux.push(a);
        }
    }
    // A compiler's stack: deep expressions recurse deeply.
    let verdicts = std::thread::Builder::new()
        .stack_size(256 << 20)
        .spawn(move || run(&library, &tests.iter().collect::<Vec<_>>(), Want::from_env()))
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
    drive(Want::Pass);
}

/// `core`, `alloc` and `std` are accepted by the borrow checker.
#[test]
fn borrowck_accepts_the_standard_library() {
    let Some(rust) = std::env::var_os("EPITOME_RUST").map(PathBuf::from) else {
        eprintln!("EPITOME_RUST not set: skipped");
        return;
    };
    let library = rust.join("library");
    let session = Session::new(Options { cfg: Some(CfgSet::host()), ..Default::default() });
    let roots: Vec<CrateRoot> = STD[..3]
        .iter()
        .map(|n| CrateRoot::new(*n, library.join(n).join("src/lib.rs")).edition(Edition::E2024))
        .collect();
    let mut crates = session.parse_crates(roots);
    session.expand(&mut crates);
    let r = session.resolve(&crates);
    let tcx = session.collect_types(&crates, &r);
    let res = session.typeck(&crates, &r, &tcx);
    let found = session.borrowck(&crates, &r, &tcx, &res, &[0, 1, 2]);
    let shown: Vec<String> = found.iter().take(20).map(|d| d.to_string()).collect();
    assert!(found.is_empty(), "{} errors in the standard library, such as:\n{}", found.len(), shown.join("\n"));
}

/// The tests that print variances: each prints rustc's.
#[test]
fn variances_match_rustc() {
    drive(Want::Variance);
}

/// The tests that expect borrow-checking errors: each gets exactly those.
#[test]
fn borrowck_expected_errors_are_found() {
    drive(Want::Borrowck);
}

fn drive(want: Want) {
    let Some(rust) = std::env::var_os("EPITOME_RUST").map(PathBuf::from) else {
        eprintln!("EPITOME_RUST not set: skipped");
        return;
    };
    let (ui, library) = (rust.join("tests/ui"), rust.join("library"));
    assert!(ui.is_dir() && library.is_dir(), "{} needs tests/ui and library", rust.display());
    let list_name = match want {
        Want::Pass => "tests/ui_check_pass_todo.txt",
        Want::Borrowck => "tests/ui_borrowck_todo.txt",
        Want::Variance => "tests/ui_variance_todo.txt",
    };
    let list_path = Path::new(env!("CARGO_MANIFEST_DIR")).join(list_name);
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
        for (name, what) in select(&ui, f, want) {
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
    // Batches of tests whose auxiliary crates do not clash: one name, one
    // crate.
    let mut batches: Vec<Vec<&Test>> = vec![Vec::new()];
    let mut names: BTreeMap<&str, &Aux> = BTreeMap::new();
    for t in &tests {
        if t.alone {
            batches.push(vec![t]);
            batches.push(Vec::new());
            names.clear();
            continue;
        }
        let clash = t.aux.iter().any(|a| names.get(a.name.as_str()).is_some_and(|b| *b != a));
        if clash || batches.last().unwrap().len() == BATCH {
            batches.push(Vec::new());
            names.clear();
        }
        names.extend(t.aux.iter().map(|a| (a.name.as_str(), a)));
        batches.last_mut().unwrap().push(t);
    }
    let mut verdicts = Vec::new();
    for b in batches.iter().filter(|b| !b.is_empty()) {
        run_isolated(&library, b, &mut verdicts, want);
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
        "{} {} tests: {} pass, {failed} fail, {} left out ({ignored} not for this host), in {:.1} s",
        tests.len() + todo.len() - failed,
        want.name(),
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
             # The {} UI tests that do not pass yet, and why (tests/ui_check_pass.rs).\n",
            want.name()
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
    let listed: BTreeMap<&str, &str> = list
        .lines()
        .filter(|l| !l.starts_with('#') && !l.is_empty())
        .map(|l| l.split_once(": ").unwrap_or((l, "")))
        .filter(|(p, _)| ui.join(p).to_string_lossy().contains(&filter))
        .collect();
    // Failing for a new reason is no change, but a crash (a panic, a stack
    // overflow, a hang) where there was none is.
    let crash = |why: &str| why.starts_with("crash") || why == "timed out";
    let now_fail: Vec<String> = todo
        .iter()
        .filter(|(p, why)| listed.get(p.as_str()).is_none_or(|was| crash(why) && !crash(was)))
        .map(|(p, why)| format!("  {p}: {why}"))
        .collect();
    let now_pass: Vec<&str> = listed.keys().copied().filter(|p| !todo.contains_key(*p)).collect();
    let mut problems = String::new();
    if !now_fail.is_empty() {
        problems += &format!("{} tests that passed now fail, or now crash:\n{}\n", now_fail.len(), now_fail.join("\n"));
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
