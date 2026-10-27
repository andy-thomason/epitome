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

/// Legal tests that do not follow the standard, and the issues raised
/// about them.
const SUITE_ISSUES: &[(&str, &str)] = &[
    (
        "05-lexical-conventions/05.09.1-special-characters-in-strings/escape_sequences_octal.sv",
        "IEEE1800-2023-tests#4: `\\7z` is not a legal octal escape (5.9.1)",
    ),
    (
        "06-data-types/06.06-net-types/nettype_valid_data_types.sv",
        "IEEE1800-2023-tests#3: no dimensions may follow a nettype's name (6.6.7)",
    ),
    (
        "40-coverage-api/40.03-coverage-system-functions/coverage_save_then_merge.sv",
        "IEEE1800-2023-tests#2: `before` is a reserved keyword (Annex B)",
    ),
];

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
            (Err(_), None) if SUITE_ISSUES.iter().any(|(n, _)| *n == name) => {}
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
/// Legal tests that break the standard, fixed in IEEE1800-2023-tests#5:
/// a `timeunit` after other items (3.14.2.2), or an initialised variable
/// in a static block without `static` (6.21).
const FIXED_IN_PR5: &[&str] = &[
    "03-design-and-verification-building-blocks/03.12.1-compilation-units/identifier_search_order.sv",
    "03-design-and-verification-building-blocks/03.13-name-spaces/block_scope_shadows_module.sv",
    "09-processes/09.03-block-statements/block_names.sv",
    "23-modules-and-hierarchy/23.06-hierarchical-names/hierarchical_references.sv",
    "H-dpi-c-layer/H.09-context/scope_functions.sv",
    "14-clocking-blocks/14.03-clocking-blocks/clocking_block_event.sv",
    "14-clocking-blocks/14.03-clocking-blocks/cycle_delay.sv",
    "14-clocking-blocks/14.03-clocking-blocks/cycle_delay_zero.sv",
    "14-clocking-blocks/14.03-clocking-blocks/default_clocking_by_name.sv",
    "14-clocking-blocks/14.03-clocking-blocks/global_clocking.sv",
    "14-clocking-blocks/14.03-clocking-blocks/hierarchical_clocking_signals.sv",
    "14-clocking-blocks/14.03-clocking-blocks/input_sampling_default_skew.sv",
    "14-clocking-blocks/14.03-clocking-blocks/input_skew_values.sv",
    "14-clocking-blocks/14.03-clocking-blocks/multiple_future_drives.sv",
    "14-clocking-blocks/14.03-clocking-blocks/synchronous_drives.sv",
    "16-assertions/16.05-concurrent-sampling/sampled_values.sv",
    "16-assertions/16.07-sequences/first_match_throughout_within.sv",
    "16-assertions/16.07-sequences/repetition.sv",
    "16-assertions/16.07-sequences/sampled_value_functions.sv",
    "16-assertions/16.07-sequences/sequence_and_or_intersect.sv",
    "16-assertions/16.07-sequences/sequence_delays.sv",
    "16-assertions/16.10-local-variables/sequence_local_variables.sv",
    "16-assertions/16.11-subroutines-on-match/subroutine_calls_on_match.sv",
    "16-assertions/16.12-properties/abort_and_disable.sv",
    "16-assertions/16.12-properties/boolean_property_operators.sv",
    "16-assertions/16.12-properties/case_and_recursive_properties.sv",
    "16-assertions/16.12-properties/followed_by.sv",
    "16-assertions/16.12-properties/implication.sv",
    "16-assertions/16.12-properties/temporal_operators.sv",
    "16-assertions/16.13-sequence-methods/triggered_method.sv",
    "16-assertions/16.14-concurrent-assertions/assert_assume_cover.sv",
    "16-assertions/16.14-concurrent-assertions/default_clocking_and_disable.sv",
    "16-assertions/16.14-concurrent-assertions/procedural_concurrent_assertion.sv",
    "16-assertions/16.17-expect/expect_statement.sv",
    "17-checkers/17.02-checker-declaration/checker_with_assertions.sv",
    "17-checkers/17.03-checker-instantiation/procedural_checker_instance.sv",
    "17-checkers/17.07-checker-variables/checker_variable_assignments.sv",
    "17-checkers/17.07-checker-variables/free_variable_with_assumption.sv",
    "20-utility-system-tasks/20.03-time-functions/time_functions.sv",
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
        if SUITE_ISSUES.iter().any(|(n, _)| *n == name) || FIXED_IN_PR5.contains(&name.as_str()) {
            continue;
        }
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
            (Ok(_), Some(why)) if kind == "elaboration" => accepted.push(format!("{name}: {why}")),
            (Ok(_), Some(_)) => {}
            (Err(_), Some(_)) if kind == "elaboration" || kind == "parse" => rejected += 1,
            (Err(e), Some(why)) => early.push(format!("{name}: {why}\n{e}")),
            (Err(e), None) => wrong.push(format!("{name}\n{e}")),
        }
    }
    eprintln!(
        "{} tests: {ok} legal tests elaborate, {} do not; {rejected} should-fail tests rejected, {} elaboration tests wrongly accepted, {} simulation should-fail tests rejected early; {unknown} expressions of unknown type, {opaque} items kept as syntax; {separate_units} tests need one compilation unit per file and were skipped",
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
        wrong.is_empty() && accepted.is_empty(),
        "{} legal tests do not elaborate; {} should-fail elaboration tests accepted",
        wrong.len(),
        accepted.len()
    );
}
