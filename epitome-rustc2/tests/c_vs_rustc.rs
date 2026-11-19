//! Each program in `tests/c/` calls C functions of the C file of its name
//! (built into `libcNAME.a`, and `NAME.o`, in a directory both compilers
//! are given with `-L`): it is compiled by `epitome-rustc2` (with `std`
//! from rust-src, optimised and not) and by rustc, and run, and the
//! outputs must be the same. A line `//@ link: ARGS` gives both compilers
//! more arguments (`$DIR` is that directory). On x86-64 Linux and AArch64
//! macOS, with rust-src and `cc`.

use std::path::{Path, PathBuf};
use std::process::Command;

fn run(exe: &Path) -> (String, Option<i32>) {
    let out = Command::new(exe).output().expect("the program runs");
    (String::from_utf8_lossy(&out.stdout).into_owned(), out.status.code())
}

#[test]
fn c_programs_match_rustc() {
    if !cfg!(any(all(target_os = "linux", target_arch = "x86_64"), all(target_os = "macos", target_arch = "aarch64"))) {
        eprintln!("not x86-64 Linux or AArch64 macOS: skipped");
        return;
    }
    if epitome_rustc2::sysroot::rust_src().is_none() {
        eprintln!("no rust-src: skipped");
        return;
    }
    let dir = std::env::temp_dir().join(format!("epitome-rustc2-c-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/c");
    let mut c_files: Vec<PathBuf> = std::fs::read_dir(&src).unwrap().flatten().map(|e| e.path()).filter(|p| p.extension().is_some_and(|x| x == "c")).collect();
    c_files.sort();
    for c in &c_files {
        let name = c.file_stem().unwrap().to_string_lossy().into_owned();
        let obj = dir.join(format!("{name}.o"));
        assert!(Command::new("cc").arg("-c").arg("-O1").arg(c).arg("-o").arg(&obj).status().unwrap().success(), "cc failed on {name}");
        let lib = dir.join(format!("libc{name}.a"));
        assert!(Command::new("ar").arg("rcs").arg(&lib).arg(&obj).status().unwrap().success(), "ar failed on {name}");
    }
    let mut programs: Vec<PathBuf> = std::fs::read_dir(&src).unwrap().flatten().map(|e| e.path()).filter(|p| p.extension().is_some_and(|x| x == "rs")).collect();
    programs.sort();
    assert!(!programs.is_empty());
    let mut failed = Vec::new();
    for p in &programs {
        let name = p.file_stem().unwrap().to_string_lossy().into_owned();
        let text = std::fs::read_to_string(p).unwrap();
        let mut args = vec!["-L".to_string(), dir.display().to_string()];
        for l in text.lines().filter_map(|l| l.strip_prefix("//@ link:")) {
            args.extend(l.split_whitespace().map(|a| a.replace("$DIR", &dir.display().to_string())));
        }
        let rc = dir.join(format!("{name}.rustc"));
        let built = Command::new("rustc").args(["--edition", "2024", "-C", "panic=abort"]).args(&args).arg(p).arg("-o").arg(&rc).output().unwrap();
        assert!(built.status.success(), "rustc failed on {name}:\n{}", String::from_utf8_lossy(&built.stderr));
        let want = run(&rc);
        for opt in [true, false] {
            let ep = dir.join(format!("{name}.{}", if opt { "ep" } else { "ep0" }));
            let mut cmd = Command::new(env!("CARGO_BIN_EXE_epitome-rustc2"));
            if !opt {
                cmd.arg("-O0");
            }
            let built = cmd.arg("--std").args(&args).arg("-o").arg(&ep).arg(p).output().unwrap();
            if !built.status.success() {
                failed.push(format!("{name} (optimised: {opt}): not built:\n{}", String::from_utf8_lossy(&built.stderr)));
                continue;
            }
            let got = run(&ep);
            if got != want {
                failed.push(format!("{name} (optimised: {opt}): got {got:?}\nrustc's gives {want:?}"));
            }
        }
    }
    let _ = std::fs::remove_dir_all(&dir);
    assert!(failed.is_empty(), "{}", failed.join("\n\n"));
}
