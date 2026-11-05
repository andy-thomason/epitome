//! Hand-written IR for what crates' code showed us we did wrong or not at
//! all: each `tests/ir/NAME.ll`, built by us and by clang, linked with
//! `NAME.main.c` (built by clang) and run on this machine; the two must
//! print the same. On an AArch64 Mac with clang, or on x86-64 Linux (with
//! its triple and data layout, leaving out what calls AArch64's
//! intrinsics).

use std::path::PathBuf;
use std::process::Command;

use epitome_codegen::target::Target;
use epitome_codegen::{Options, compile};

/// What a test whose driver says `// links: compiler-builtins` links too:
/// the runtime's functions (quad precision's `__addtf3`...) from Rust's
/// compiler_builtins, and the symbol it expects of a Rust program.
fn runtime(main: &std::path::Path, tmp: &std::path::Path) -> Vec<std::path::PathBuf> {
    let text = std::fs::read_to_string(main).unwrap_or_default();
    if !text.contains("// links: compiler-builtins") {
        return Vec::new();
    }
    let Ok(o) = Command::new("rustc").arg("--print").arg("sysroot").output() else { return Vec::new() };
    let lib = PathBuf::from(String::from_utf8_lossy(&o.stdout).trim()).join("lib/rustlib/aarch64-apple-darwin/lib");
    let Some(rlib) = std::fs::read_dir(&lib).ok().and_then(|d| {
        d.filter_map(|e| e.ok().map(|e| e.path())).find(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.starts_with("libcompiler_builtins-") && n.ends_with(".rlib"))
        })
    }) else {
        return Vec::new();
    };
    let stub = tmp.join("rust_eh_personality.c");
    std::fs::write(&stub, "void rust_eh_personality(void) {}\n").unwrap();
    vec![stub, rlib]
}

fn output(obj: &std::path::Path, main: &std::path::Path, exe: &std::path::Path) -> Result<String, String> {
    let extra = runtime(main, exe.parent().unwrap());
    let st = Command::new("clang")
        .arg(obj)
        .arg(main)
        .args(&extra)
        .arg("-lm")
        .arg("-o")
        .arg(exe)
        .status()
        .map_err(|e| e.to_string())?;
    if !st.success() {
        return Err(format!("linking {} failed", exe.display()));
    }
    let o = Command::new(exe).output().map_err(|e| e.to_string())?;
    if !o.status.success() {
        return Err(format!("{} failed", exe.display()));
    }
    Ok(String::from_utf8_lossy(&o.stdout).into_owned())
}

#[test]
fn ir_behaves_as_clangs_build() {
    let x86 = cfg!(target_arch = "x86_64") && cfg!(target_os = "linux");
    let ok = (cfg!(target_arch = "aarch64") && cfg!(target_os = "macos") || x86)
        && Command::new("clang").arg("--version").output().is_ok_and(|o| o.status.success());
    if !ok {
        eprintln!("needs clang on an AArch64 Mac or x86-64 Linux; skipping");
        return;
    }
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/ir");
    let tmp = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("ir");
    std::fs::create_dir_all(&tmp).unwrap();
    let t = if x86 { Target::x86_64() } else { Target::aarch64() };
    let mut names: Vec<String> = std::fs::read_dir(&dir)
        .unwrap()
        .filter_map(|e| e.ok()?.file_name().to_str()?.strip_suffix(".ll").map(str::to_string))
        .collect();
    names.sort();
    let mut failures = Vec::new();
    for name in &names {
        let ll = dir.join(format!("{name}.ll"));
        let main = dir.join(format!("{name}.main.c"));
        let mut ll = ll;
        if x86 {
            // This machine's triple and layout; not AArch64's intrinsics.
            let text = std::fs::read_to_string(&ll).unwrap();
            // (Nor number_minmax: of two NaNs, clang's x86 build returns a
            // signalling one as it is, where LLVM's reference says a quiet
            // one, as ours does.)
            // (Nor module_asm, whose assembly is AArch64's, nor fp128_ops:
            // fp128 on x86-64 is not done yet.)
            if text.contains("@llvm.aarch64.") || ["number_minmax", "module_asm", "fp128_ops"].contains(&name.as_str())
            {
                continue;
            }
            let text: String = text
                .lines()
                .map(|l| match l {
                    l if l.starts_with("target triple") => "target triple = \"x86_64-unknown-linux-gnu\"".to_string(),
                    l if l.starts_with("target datalayout") => String::new(),
                    l => l.to_string(),
                })
                .collect::<Vec<_>>()
                .join("\n");
            ll = tmp.join(format!("{name}.x86.ll"));
            std::fs::write(&ll, text).unwrap();
        }
        let run = || -> Result<(), String> {
            let src = std::fs::read_to_string(&ll).map_err(|e| e.to_string())?;
            let mut m = epitome_ir::parse_module(&src).map_err(|e| format!("{e:?}"))?;
            let ours = tmp.join(format!("{name}.ours.o"));
            std::fs::write(&ours, compile(&t, &mut m, &Options::default())?.object_file(&t)?).unwrap();
            let theirs = tmp.join(format!("{name}.clang.o"));
            let st = Command::new("clang").arg("-c").arg(&ll).arg("-o").arg(&theirs).status().unwrap();
            if !st.success() {
                return Err("clang could not build it".into());
            }
            let a = output(&ours, &main, &tmp.join(format!("{name}.ours")))?;
            let b = output(&theirs, &main, &tmp.join(format!("{name}.clang")))?;
            if a != b {
                return Err("differs from clang's build".into());
            }
            println!("ok {name}");
            Ok(())
        };
        if let Err(e) = run() {
            failures.push(format!("{name}: {e}"));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
