//! Runs the lexer and preprocessor over the IEEE 1800-2023 conformance tests
//! (github.com/andy-thomason/IEEE1800-2023-tests), when `IEEE1800_TESTS`
//! names a checkout of them. Without it the tests pass and say they were
//! skipped.

use epitome_sv::lexer::tokenize;
use epitome_sv::parse::parse;
use epitome_sv::preprocess::{Options, preprocess_files};
use epitome_sv::source::{Source, SourceMap};
use std::path::{Path, PathBuf};

fn suite() -> Option<PathBuf> {
    match std::env::var("IEEE1800_TESTS") {
        Ok(root) => Some(PathBuf::from(root).join("tests")),
        Err(_) => {
            eprintln!("IEEE1800_TESTS is not set; skipped");
            None
        }
    }
}

fn files(dir: &Path, out: &mut Vec<PathBuf>) {
    let mut entries: Vec<_> = std::fs::read_dir(dir).unwrap().map(|e| e.unwrap().path()).collect();
    entries.sort();
    for path in entries {
        if path.is_dir() {
            files(&path, out);
        } else if path.extension().is_some_and(|e| e == "sv" || e == "svh") {
            out.push(path);
        }
    }
}

/// The value of `:key:` in a test's header comment, with continuation
/// lines joined.
fn header(text: &str, key: &str) -> Option<String> {
    let head = text.strip_prefix("/*")?.split("*/").next()?;
    let mut value: Option<String> = None;
    for line in head.lines() {
        if let Some(rest) = line.strip_prefix(&format!(":{key}:")) {
            value = Some(rest.trim().to_string());
        } else if line.starts_with(':') {
            if value.is_some() {
                break;
            }
        } else if let Some(v) = &mut value {
            v.push(' ');
            v.push_str(line.trim());
        }
    }
    value
}

#[test]
fn lex_test1800() {
    let Some(root) = suite() else { return };
    let mut paths = Vec::new();
    files(&root, &mut paths);
    assert!(!paths.is_empty(), "no tests under {}", root.display());
    let (mut lexed, mut rejected, mut wrong) = (0, 0, Vec::new());
    for path in &paths {
        let src = Source::new(path.display().to_string(), std::fs::read_to_string(path).unwrap());
        let should_fail = src.text.contains(":should_fail_because:");
        match tokenize(&src.text) {
            Ok(_) => lexed += 1,
            // A test that should fail may fail in the lexer; which ones
            // should is checked once the parser decides the rest.
            Err(_) if should_fail => rejected += 1,
            Err(e) => wrong.push(e.render(&src)),
        }
    }
    eprintln!("{} files: {lexed} lexed, {rejected} should-fail tests rejected, {} wrong", paths.len(), wrong.len());
    assert!(wrong.is_empty(), "legal files the lexer rejects:\n{}", wrong.join("\n"));
}

/// Should-fail tests whose error is the preprocessor's to find.
const PREPROCESSOR_ERRORS: &[&str] = &[
    "03-design-and-verification-building-blocks/03.14-simulation-time-units/timescale_precision_coarser_than_unit_error.sv",
    "22-compiler-directives/22.03-resetall/resetall_inside_design_element_error.sv",
    "22-compiler-directives/22.05-define/macro_named_like_directive_error.sv",
    "22-compiler-directives/22.05-define/macro_text_split_string_error.sv",
    "22-compiler-directives/22.06-conditional-compilation/ignored_text_lexical_error.sv",
    "22-compiler-directives/22.07-timescale/timescale_inside_module_error.sv",
];

#[test]
fn preprocess_test1800() {
    let Some(root) = suite() else { return };
    let mut paths = Vec::new();
    files(&root, &mut paths);
    paths.retain(|p| p.extension().is_some_and(|e| e == "sv"));
    let (mut ok, mut wrong, mut rejected) = (0, Vec::new(), Vec::new());
    for path in &paths {
        let text = std::fs::read_to_string(path).unwrap();
        let dir = path.parent().unwrap();
        let mut sources = vec![path.clone()];
        if let Some(more) = header(&text, "files") {
            sources.extend(more.split_whitespace().map(|f| dir.join(f)));
        }
        let should_fail = header(&text, "should_fail_because").is_some();
        let name = path.strip_prefix(&root).unwrap().display().to_string();
        let map = SourceMap::new();
        match preprocess_files(&map, &sources, &Options::default()) {
            Ok(_) if PREPROCESSOR_ERRORS.contains(&name.as_str()) => {
                wrong.push(format!("{name}: should have been rejected"));
            }
            Ok(_) => ok += 1,
            Err(e) if should_fail => rejected.push(format!("{name}\n{e}")),
            Err(e) => wrong.push(e),
        }
    }
    eprintln!(
        "{} tests: {ok} preprocessed, {} should-fail tests rejected, {} wrong",
        paths.len(),
        rejected.len(),
        wrong.len()
    );
    for r in &rejected {
        eprintln!("rejected: {r}");
    }
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

/// Should-fail parse tests that only elaboration can reject: the syntax is
/// valid, and the error is in what a name turns out to be.
const NEEDS_ELABORATION: &[(&str, &str)] = &[(
    "23-modules-and-hierarchy/23.03-module-instances/missing_parentheses_error.sv",
    "`M u;` declares a variable of type M until M is known to be a module",
)];

#[test]
fn parse_test1800() {
    let Some(root) = suite() else { return };
    let mut paths = Vec::new();
    files(&root, &mut paths);
    paths.retain(|p| p.extension().is_some_and(|e| e == "sv"));
    let (mut parsed, mut rejected) = (0, 0);
    let (mut wrong, mut accepted) = (Vec::new(), Vec::new());
    for path in &paths {
        let text = std::fs::read_to_string(path).unwrap();
        let dir = path.parent().unwrap();
        let mut sources = vec![path.clone()];
        if let Some(more) = header(&text, "files") {
            sources.extend(more.split_whitespace().map(|f| dir.join(f)));
        }
        let should_fail = header(&text, "should_fail_because");
        let parse_test = header(&text, "type").is_some_and(|t| t == "parse");
        let name = path.strip_prefix(&root).unwrap().display().to_string();
        let map = SourceMap::new();
        let result = preprocess_files(&map, &sources, &Options::default())
            .and_then(|out| parse(&out).map(|_| ()).map_err(|e| map.render(&e)));
        match (result, &should_fail) {
            (Ok(()), None) => parsed += 1,
            (Ok(()), Some(why)) if parse_test && !NEEDS_ELABORATION.iter().any(|(n, _)| *n == name) => {
                accepted.push(format!("{name}: {why}"));
            }
            (Ok(()), Some(_)) => {}
            (Err(_), Some(_)) => rejected += 1,
            (Err(e), None) => wrong.push(format!("{name}\n{e}")),
        }
    }
    eprintln!(
        "{} tests: {parsed} legal tests parse, {} do not; {rejected} should-fail tests rejected, {} parse tests wrongly accepted",
        paths.len(),
        wrong.len(),
        accepted.len()
    );
    if let Ok(out) = std::env::var("PARSE_REPORT") {
        let report = format!(
            "# legal tests that do not parse\n\n{}\n\n# should-fail parse tests accepted\n\n{}\n",
            wrong.join("\n"),
            accepted.join("\n")
        );
        std::fs::write(out, report).unwrap();
    }
    assert!(
        wrong.is_empty() && accepted.is_empty(),
        "{} legal tests do not parse; {} should-fail parse tests accepted",
        wrong.len(),
        accepted.len()
    );
}

/// Runs elaboration over every test. Legal tests must elaborate; tests
/// that should fail at elaboration must be rejected by the parser or by
/// elaboration. `ELAB_REPORT=<file>` writes the details.
/// Should-fail elaboration tests whose rules need what the elaborator does
/// not model yet: clocking blocks (14.3), checker bodies (17), mailbox
/// element types (15.4), clock inference (16.9), configurations (33) and
/// specialised virtual interface types (25.9). Each is accepted for now;
/// the test fails if one starts being rejected, so the list only shrinks.
const NOT_YET_CHECKED: &[&str] = &[
    "14-clocking-blocks/14.03-clocking-blocks/blocking_assign_to_clockvar_error.sv",
    "14-clocking-blocks/14.03-clocking-blocks/continuous_assign_to_output_clockvar_error.sv",
    "14-clocking-blocks/14.03-clocking-blocks/cycle_delay_without_default_clocking_error.sv",
    "14-clocking-blocks/14.03-clocking-blocks/other_intra_delay_in_drive_error.sv",
    "14-clocking-blocks/14.03-clocking-blocks/read_output_clockvar_error.sv",
    "14-clocking-blocks/14.03-clocking-blocks/two_default_clockings_error.sv",
    "14-clocking-blocks/14.03-clocking-blocks/write_input_clockvar_error.sv",
    "15-interprocess-synchronization/15.04-mailboxes/parameterized_mailbox_type_error.sv",
    "16-assertions/16.09-sampled-value-functions/sampled_function_without_clock_error.sv",
    "17-checkers/17.02-checker-declaration/output_without_type_error.sv",
    "17-checkers/17.03-checker-instantiation/checker_in_fork_error.sv",
    "17-checkers/17.07-checker-variables/free_variable_continuous_assign_error.sv",
    "17-checkers/17.07-checker-variables/hierarchical_checker_variable_assignment_error.sv",
    "17-checkers/17.08-functions-in-checkers/function_with_output_in_checker_error.sv",
    "25-interfaces/25.09-virtual-interfaces/parameter_mismatch_assignment_error.sv",
    "33-configurations/33.04-configurations/cell_library_with_liblist_error.sv",
    "33-configurations/33.04-configurations/config_localparam_nonliteral_error.sv",
    "33-configurations/33.04-configurations/default_use_clause_error.sv",
    "33-configurations/33.04-configurations/instance_inside_other_config_error.sv",
];

#[test]
fn elaborate_test1800() {
    // Elaboration recurses as deeply as the design nests.
    std::thread::Builder::new().stack_size(512 << 20).spawn(elaborate_all).unwrap().join().unwrap();
}

fn elaborate_all() {
    let Some(root) = suite() else { return };
    let mut paths = Vec::new();
    files(&root, &mut paths);
    paths.retain(|p| p.extension().is_some_and(|e| e == "sv"));
    let (mut ok, mut rejected) = (0, 0);
    let (mut wrong, mut accepted, mut early) = (Vec::new(), Vec::new(), Vec::new());
    let (mut unknown, mut opaque) = (0, 0);
    let mut separate_units = 0;
    let (mut not_yet, mut now_checked) = (0, Vec::new());
    for path in &paths {
        let text = std::fs::read_to_string(path).unwrap();
        let dir = path.parent().unwrap();
        let mut sources = vec![path.clone()];
        if let Some(more) = header(&text, "files") {
            sources.extend(more.split_whitespace().map(|f| dir.join(f)));
        }
        let should_fail = header(&text, "should_fail_because");
        let kind = header(&text, "type").unwrap_or_else(|| "simulation".into());
        let name = path.strip_prefix(&root).unwrap().display().to_string();
        // One compilation unit per file (3.12.1 b) is not supported yet.
        if header(&text, "flags-icarus").is_some_and(|f| f.split_whitespace().any(|w| w == "-u")) {
            separate_units += 1;
            continue;
        }
        let map = SourceMap::new();
        let Ok(out) = preprocess_files(&map, &sources, &Options::default()) else { continue };
        let Ok(ast) = parse(&out) else {
            if should_fail.is_some() {
                rejected += 1;
            }
            continue;
        };
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            epitome_sv::design::elab::elaborate(&ast, &out).map(|d| d.gaps()).map_err(|e| map.render(&e))
        }));
        let result = match result {
            Ok(r) => r,
            Err(_) => Err(format!("{name}: the elaborator panicked")),
        };
        match (result, &should_fail) {
            (Ok((u, o)), None) => {
                ok += 1;
                unknown += u;
                opaque += o;
            }
            (Ok(_), Some(_)) if kind == "elaboration" && NOT_YET_CHECKED.contains(&name.as_str()) => not_yet += 1,
            (Ok(_), Some(why)) if kind == "elaboration" => accepted.push(format!("{name}: {why}")),
            (Err(_), Some(_)) if NOT_YET_CHECKED.contains(&name.as_str()) => {
                now_checked.push(name.clone());
                rejected += 1;
            }
            (Ok(_), Some(_)) => {}
            (Err(_), Some(_)) if kind == "elaboration" || kind == "parse" => rejected += 1,
            (Err(e), Some(why)) => early.push(format!("{name}: {why}\n{e}")),
            (Err(e), None) => wrong.push(format!("{name}\n{e}")),
        }
    }
    eprintln!(
        "{} tests: {ok} legal tests elaborate, {} do not; {rejected} should-fail tests rejected, {} elaboration tests wrongly accepted ({not_yet} more known not to be checked yet), {} simulation should-fail tests rejected early; {unknown} expressions of unknown type, {opaque} items kept as syntax; {separate_units} tests need one compilation unit per file and were skipped",
        paths.len(),
        wrong.len(),
        accepted.len(),
        early.len()
    );
    if let Ok(out) = std::env::var("ELAB_REPORT") {
        let report = format!(
            "# legal tests that do not elaborate\n\n{}\n\n# should-fail elaboration tests accepted\n\n{}\n\n# simulation should-fail tests rejected at elaboration\n\n{}\n",
            wrong.join("\n"),
            accepted.join("\n"),
            early.join("\n")
        );
        std::fs::write(out, report).unwrap();
    }
    assert!(
        wrong.is_empty() && accepted.is_empty() && now_checked.is_empty(),
        "{} legal tests do not elaborate; {} should-fail elaboration tests accepted; remove from NOT_YET_CHECKED: {:?}",
        wrong.len(),
        accepted.len(),
        now_checked
    );
}
