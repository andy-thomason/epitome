//! SVE and SME: each `tests/sve/NAME.ll` (IR of scalable vectors, for
//! `aarch64-unknown-linux-gnu`), or `NAME.c` (loops, which clang's
//! vectoriser makes IR of scalable vectors of), built by us and by clang
//! from the same IR, linked with
//! `NAME.main.c` (built by clang, with `arm_sve.h`) and run under qemu at
//! vector lengths of 128, 256, 512 and 2048 bits; the two builds must
//! print the same at each. Code of scalable vectors must be right at
//! every length, so each is tried.
//!
//! On x86-64 Linux with clang, `qemu-aarch64` and rustup's
//! `aarch64-unknown-linux-musl` target (whose musl the programs are linked
//! with, statically); elsewhere skipped.
use std::path::{Path, PathBuf};
use std::process::Command;

use epitome_codegen::target::Target;
use epitome_codegen::{Options, compile};

/// What clang is told the machine has.
const MARCH: &str = "-march=armv9-a+sve2+sme";

/// The vector lengths tried, in bytes.
const LENGTHS: [u32; 4] = [16, 32, 64, 256];

/// qemu, rust-lld, musl's files and Rust's compiler builtins, if here.
struct Tools {
    qemu: PathBuf,
    lld: PathBuf,
    musl: PathBuf,
    builtins: PathBuf,
}

fn tools() -> Option<Tools> {
    if !(cfg!(target_arch = "x86_64") && cfg!(target_os = "linux")) {
        return None;
    }
    Command::new("clang").arg("--version").output().ok().filter(|o| o.status.success())?;
    let path = std::env::var_os("PATH")?;
    let qemu = ["qemu-aarch64-static", "qemu-aarch64"]
        .iter()
        .flat_map(|q| std::env::split_paths(&path).map(move |d| d.join(q)))
        .find(|p| p.exists())?;
    let o = Command::new("rustc").arg("--print").arg("sysroot").output().ok()?;
    let root = PathBuf::from(String::from_utf8_lossy(&o.stdout).trim());
    let lib = root.join("lib/rustlib/aarch64-unknown-linux-musl/lib");
    let builtins = std::fs::read_dir(&lib).ok()?.flatten().map(|e| e.path()).find(|p| {
        p.extension().is_some_and(|x| x == "rlib")
            && p.file_name().is_some_and(|n| n.to_string_lossy().starts_with("libcompiler_builtins-"))
    })?;
    let lld = root.join("lib/rustlib/x86_64-unknown-linux-gnu/bin/gcc-ld/ld.lld");
    let musl = lib.join("self-contained");
    (lld.exists() && musl.join("libc.a").exists()).then_some(Tools { qemu, lld, musl, builtins })
}

/// `obj` linked with the driver `main`, run at each vector length: what it
/// printed at each.
fn outputs(t: &Tools, obj: &Path, main: &Path, exe: &Path) -> Result<Vec<String>, String> {
    let stub = exe.with_extension("personality.c");
    std::fs::write(&stub, "void rust_eh_personality(void) {}\n").unwrap();
    let st = Command::new("clang")
        .args(["--target=aarch64-unknown-linux-musl", MARCH, "-O1", "-fuse-ld=lld", "-static", "-nostdlib"])
        .arg(format!("--ld-path={}", t.lld.display()))
        .arg(t.musl.join("crt1.o"))
        .arg(t.musl.join("crti.o"))
        .arg(&stub)
        .arg(main)
        .arg(obj)
        .arg(t.musl.join("libc.a"))
        .arg(&t.builtins)
        .arg(t.musl.join("crtn.o"))
        .arg("-o")
        .arg(exe)
        .status()
        .map_err(|e| e.to_string())?;
    if !st.success() {
        return Err(format!("linking {} failed", exe.display()));
    }
    let mut out = Vec::new();
    for vl in LENGTHS {
        let o = Command::new(&t.qemu)
            .arg("-cpu")
            .arg(format!("max,sve-default-vector-length={vl}"))
            .arg(exe)
            .output()
            .map_err(|e| e.to_string())?;
        if !o.status.success() {
            return Err(format!("{} failed at {} bits: {}", exe.display(), vl * 8, String::from_utf8_lossy(&o.stderr)));
        }
        out.push(String::from_utf8_lossy(&o.stdout).into_owned());
    }
    Ok(out)
}

#[test]
fn sve_behaves_as_clangs_build() {
    let Some(tools) = tools() else {
        eprintln!("needs x86-64 Linux with clang, qemu-aarch64 and rustup's aarch64-unknown-linux-musl; skipping");
        return;
    };
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/sve");
    let tmp = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("sve");
    std::fs::create_dir_all(&tmp).unwrap();
    let only = std::env::var("EPITOME_SVE").ok();
    let mut names: Vec<String> = std::fs::read_dir(&dir)
        .unwrap()
        .flatten()
        .filter_map(|e| {
            let n = e.file_name().to_str()?.to_string();
            if n.ends_with(".main.c") {
                return None;
            }
            n.strip_suffix(".ll").or_else(|| n.strip_suffix(".c")).map(str::to_string)
        })
        .filter(|n| only.as_ref().is_none_or(|o| n.contains(o.as_str())))
        .collect();
    names.sort();
    let t = Target::aarch64();
    let mut failures = Vec::new();
    for name in &names {
        let main = dir.join(format!("{name}.main.c"));
        let run = || -> Result<(), String> {
            let ll = if dir.join(format!("{name}.c")).exists() {
                let ll = tmp.join(format!("{name}.ll"));
                let st = Command::new("clang")
                    .args(["--target=aarch64-unknown-linux-gnu", MARCH, "-O2", "-S", "-emit-llvm"])
                    .arg(dir.join(format!("{name}.c")))
                    .arg("-o")
                    .arg(&ll)
                    .status()
                    .map_err(|e| e.to_string())?;
                if !st.success() {
                    return Err("clang could not make IR of it".into());
                }
                ll
            } else {
                dir.join(format!("{name}.ll"))
            };
            let src = std::fs::read_to_string(&ll).map_err(|e| e.to_string())?;
            let mut m = epitome_ir::parse_module(&src).map_err(|e| format!("{e:?}"))?;
            let ours = tmp.join(format!("{name}.ours.o"));
            std::fs::write(&ours, compile(&t, &mut m, &Options::default())?.object_file(&t)?).unwrap();
            let theirs = tmp.join(format!("{name}.clang.o"));
            let st = Command::new("clang")
                .args(["--target=aarch64-unknown-linux-gnu", MARCH, "-O2", "-c"])
                .arg(&ll)
                .arg("-o")
                .arg(&theirs)
                .status()
                .unwrap();
            if !st.success() {
                return Err("clang could not build it".into());
            }
            let a = outputs(&tools, &ours, &main, &tmp.join(format!("{name}.ours")))?;
            let b = outputs(&tools, &theirs, &main, &tmp.join(format!("{name}.clang")))?;
            for ((x, y), vl) in a.iter().zip(&b).zip(LENGTHS) {
                if x != y {
                    return Err(format!("differs from clang's build at {} bits:\nours:\n{x}\nclang's:\n{y}", vl * 8));
                }
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
