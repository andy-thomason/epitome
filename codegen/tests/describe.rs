//! Checks of the descriptions themselves, and of the checks.

use epitome_codegen::check;
use epitome_codegen::target::Target;

/// The descriptions pass `epitome-llc --check-target`'s checks: every
/// instruction encodes, decodes, disassembles and assembles back to itself;
/// each recipe's instructions, by their bodies, compute what the IR it
/// replaces computes; and the smoke program compiles and reads back.
#[test]
fn descriptions_pass_their_checks() {
    for t in [Target::aarch64(), Target::load_path(&riscv_dir()).unwrap()] {
        let report = check::check(&t);
        assert_eq!(report.errors(), 0, "{}", report.render(true));
        let encodings = report.sections.iter().find(|s| s.name == "encodings").unwrap();
        assert!(encodings.checked > 1000, "{}", encodings.checked);
    }
}

fn riscv_dir() -> String {
    format!("{}/targets/riscv64", env!("CARGO_MANIFEST_DIR"))
}

/// The RISC-V description with one change, loaded.
fn changed(file: &str, from: &str, to: &str) -> Result<Target, String> {
    let mut files: Vec<(String, String)> = std::fs::read_dir(riscv_dir())
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|x| x == "ll"))
        .map(|p| (p.file_name().unwrap().to_string_lossy().into_owned(), std::fs::read_to_string(&p).unwrap()))
        .collect();
    files.sort();
    let f = files.iter_mut().find(|f| f.0 == file).unwrap();
    assert_eq!(f.1.matches(from).count(), 1, "{from}");
    f.1 = f.1.replace(from, to);
    let refs: Vec<(&str, &str)> = files.iter().map(|(n, t)| (n.as_str(), t.as_str())).collect();
    Target::load_files(&refs)
}

/// The RISC-V description with one change, checked: the errors of section
/// `section`.
fn broken(file: &str, from: &str, to: &str, section: &str) -> Vec<String> {
    let t = changed(file, from, to).unwrap();
    let report = check::check(&t);
    report.sections.into_iter().find(|s| s.name == section).unwrap().errors
}

/// The checks find mistakes of the kinds a description's author makes.
#[test]
fn checks_find_mistakes() {
    let has = |errors: &[String], want: &str| {
        assert!(errors.iter().any(|e| e.contains(want)), "no error with {want:?} in {errors:#?}");
    };
    // SUB given ADD's encoding: the disassembler cannot tell them apart.
    let e = broken(
        "10-integer.ll",
        "\"enc\"=\"0100000 m:5 n:5 000 d:5 0110011\"",
        "\"enc\"=\"0000000 m:5 n:5 000 d:5 0110011\"",
        "encodings",
    );
    has(&e, "SUB");
    // A template that names the wrong operand.
    let xor = "@XOR(i64 \"reg\"=\"x\" %n, i64 \"reg\"=\"x\" %m) \"enc\"=\"0000000 m:5 n:5 100 d:5 0110011\" \"asm\"=\"xor {d}, {n}, ";
    let e = broken("10-integer.ll", &format!("{xor}{{m}}\""), &format!("{xor}{{n}}\""), "assembly");
    has(&e, "XOR");
    // A recipe that adds where it should subtract.
    let e = broken(
        "60-recipes.ll",
        "  %r = call i64 @SUB(i64 %x, i64 %s)\n  ret i64 %r\n}\n\ndeclare i32 @llvm.abs.i32",
        "  %r = call i64 @ADD(i64 %x, i64 %s)\n  ret i64 %r\n}\n\ndeclare i32 @llvm.abs.i32",
        "recipes",
    );
    has(&e, "abs.x.expand");
    // No instruction for branches.
    let e = broken("10-integer.ll", "\"asm\"=\"j {dest}\" \"role\"=\"jump\"", "\"asm\"=\"j {dest}\"", "structure");
    has(&e, "role jump");
    // A relocation without its ELF number, under an ELF convention.
    let e =
        broken("00-base.ll", "!\"name\", !\"lo12_i\", !\"elf\", !\"27\", ", "!\"name\", !\"lo12_i\", ", "structure");
    has(&e, "no elf number");
    // A convention whose saved registers are also clobbered.
    let e = broken(
        "70-abi.ll",
        "!{!\"saved\", !\"s1\", !\"s2-s11\"}",
        "!{!\"saved\", !\"s1\", !\"s2-s11\", !\"t0\"}",
        "structure",
    );
    has(&e, "both saved and clobbered");
}

/// A convention with an empty result list is an error, not a panic.
#[test]
fn empty_results_are_errors() {
    let e = changed("70-abi.ll", "!{!\"fp-result\", !\"fa0\"}", "!{!\"fp-result\"}").err().unwrap();
    assert!(e.contains("empty fp-result"), "{e}");
    let e = changed("70-abi.ll", "!{!\"result\", !\"a0\"}", "!{!\"result\"}").err().unwrap();
    assert!(e.contains("empty result"), "{e}");
}

#[test]
fn table_is_built() {
    let t = Target::aarch64();
    assert!(t.patterns.len() > 500);
    // Comparisons and branches were combined.
    assert!(t.patterns.iter().any(|p| p.name == "CMPWrr+Bcc.lt"));
    // TST clears the carry, so it is not combined with conditions reading it.
    assert!(!t.patterns.iter().any(|p| p.name == "TSTWrr+Bcc.hs"));
    assert!(t.patterns.iter().any(|p| p.name == "TSTWrr+Bcc.eq"));
}

#[test]
fn built_in_files_are_the_directory() {
    // Every .ll file in targets/aarch64, in name order, is in the built-in
    // list, so a new file is not left out; and the directory loads as the
    // built-in description does.
    let dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("targets/aarch64");
    let mut names: Vec<String> = std::fs::read_dir(&dir)
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .filter(|n| n.ends_with(".ll"))
        .collect();
    names.sort();
    let built_in: Vec<&str> = epitome_codegen::target::AARCH64_FILES.iter().map(|f| f.0).collect();
    assert_eq!(names, built_in);
    let from_dir = Target::load_path(dir.to_str().unwrap()).unwrap();
    let t = Target::aarch64();
    assert_eq!(from_dir.instrs.len(), t.instrs.len());
    assert_eq!(from_dir.patterns.len(), t.patterns.len());
}

#[test]
fn errors_name_their_file() {
    let e =
        Target::load_files(&[("a.ll", "%nzcv = type { i64, i64 }\n"), ("b.ll", "\ndefine nonsense\n")]).err().unwrap();
    assert!(e.starts_with("b.ll:2:"), "{e}");
}
