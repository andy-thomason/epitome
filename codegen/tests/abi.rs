//! Runs AAPCS64 code built by us against AAPCS64 code built by clang.
//!
//! There is no Linux here to run Linux programs, but the AAPCS64 calling
//! convention can still be exercised on this machine, as long as only code
//! built for it calls code built for it. `tests/abi/callee.c` has functions
//! taking arguments on the stack, `caller.c` calls them, and `main.c`
//! (built by clang for this machine) calls `caller.c`'s entry points, whose
//! few arguments are passed the same way in every convention.
//!
//! Each of `callee.c` and `caller.c` is built for Linux both by us
//! (`--abi=aapcs64`) and by clang (`--target=aarch64-linux-gnu`, with its
//! assembly rewritten into this machine's dialect), and every pairing is
//! linked and run: if our layout of stack arguments differed from clang's
//! in either direction, the results would. Both keep x18 free, since this
//! machine's kernel may change it at any time.

use std::path::{Path, PathBuf};
use std::process::Command;

use epitome_codegen::abi::Syntax;
use epitome_codegen::target::Target;
use epitome_codegen::{Options, compile};

fn run(cmd: &mut Command) -> Result<String, String> {
    let out = cmd.output().map_err(|e| format!("{cmd:?}: {e}"))?;
    if !out.status.success() {
        return Err(format!(
            "{cmd:?} failed:\n{}{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        ));
    }
    Ok(String::from_utf8_lossy(&out.stdout).into_owned())
}

/// Rewrites clang's ELF assembly for this machine's assembler: Mach-O
/// sections, symbols with a leading underscore, local labels starting `L`,
/// and no ELF-only directives.
fn elf_to_macho(asm: &str) -> String {
    let mut names: Vec<String> = Vec::new();
    for l in asm.lines() {
        let t = l.trim();
        for prefix in [".globl", "bl", "b"] {
            if let Some(rest) = t.strip_prefix(prefix)
                && rest.starts_with([' ', '\t'])
            {
                let n = rest.split_whitespace().next().unwrap_or("").to_string();
                if !n.starts_with('.') && !names.contains(&n) {
                    names.push(n);
                }
            }
        }
    }
    let mut out = String::new();
    for l in asm.lines() {
        let t = l.trim_start();
        let drop = [".file", ".type", ".size", ".cfi_", ".ident", ".addrsig", ".section\t\".note", ".section .note"];
        if drop.iter().any(|d| t.starts_with(d)) {
            continue;
        }
        if t.starts_with(".text") {
            out.push_str("\t.section __TEXT,__text,regular,pure_instructions\n");
            continue;
        }
        // Constants (clang's pools of them, `.rodata.cst16`).
        if t.starts_with(".section") && t.contains(".rodata") {
            out.push_str("\t.section __TEXT,__const\n");
            continue;
        }
        // A symbol's page and the offset in it, as Mach-O writes them.
        let paged;
        let l = if t.starts_with("adrp") {
            paged = format!("{l}@PAGE");
            paged.as_str()
        } else if let Some((head, rest)) = l.split_once(":lo12:") {
            let end = rest.find([']', ',']).unwrap_or(rest.len());
            paged = format!("{head}{}@PAGEOFF{}", &rest[..end], &rest[end..]);
            paged.as_str()
        } else {
            l
        };
        // Rename symbols and local labels, word by word.
        let mut word = String::new();
        let flush = |word: &mut String, out: &mut String| {
            if let Some(rest) = word.strip_prefix(".L") {
                out.push('L');
                out.push_str(rest);
            } else if names.contains(word) {
                out.push('_');
                out.push_str(word);
            } else {
                out.push_str(word);
            }
            word.clear();
        };
        for c in l.chars() {
            if c.is_ascii_alphanumeric() || c == '_' || c == '.' || c == '$' {
                word.push(c);
            } else {
                flush(&mut word, &mut out);
                out.push(c);
            }
        }
        flush(&mut word, &mut out);
        out.push('\n');
    }
    out
}

#[derive(Clone, Copy, Debug)]
enum By {
    Us,
    Clang,
}

/// `name.c` built for AAPCS64 by `by`, as assembly for this machine.
fn build(t: &Target, dir: &Path, tmp: &Path, name: &str, by: By) -> Result<PathBuf, String> {
    let src = dir.join(format!("{name}.c"));
    let s = tmp.join(format!("{name}.{by:?}.s"));
    match by {
        By::Clang => {
            let elf = run(Command::new("clang")
                .args(["--target=aarch64-linux-gnu", "-O2", "-ffixed-x18", "-fno-inline", "-S", "-o", "-"])
                .arg(&src))?;
            std::fs::write(&s, elf_to_macho(&elf)).unwrap();
        }
        By::Us => {
            let ll = tmp.join(format!("{name}.ll"));
            run(Command::new("clang")
                .args(["--target=aarch64-linux-gnu", "-O2", "-fno-inline", "-S", "-emit-llvm", "-o"])
                .arg(&ll)
                .arg(&src))?;
            let mut m =
                epitome_ir::parse_module(&std::fs::read_to_string(&ll).unwrap()).map_err(|e| format!("{e:?}"))?;
            let opts = Options {
                abi: Some("aapcs64".into()),
                reserve: vec![18],
                syntax: Some(Syntax::MachO),
                ..Default::default()
            };
            let c = compile(t, &mut m, &opts)?;
            std::fs::write(&s, c.asm).unwrap();
        }
    }
    Ok(s)
}

#[test]
fn aapcs64_against_clang() {
    if !cfg!(all(target_arch = "aarch64", target_os = "macos")) {
        eprintln!("runs AArch64 code on an AArch64 Mac; skipping");
        return;
    }
    if !Command::new("clang").arg("--version").output().is_ok_and(|o| o.status.success()) {
        eprintln!("clang not found; skipping");
        return;
    }
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/abi");
    let tmp = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("codegen-abi");
    std::fs::create_dir_all(&tmp).unwrap();
    let t = Target::aarch64();
    // The answer, from clang's own build for this machine.
    let reference = tmp.join("reference");
    run(Command::new("clang")
        .arg(dir.join("callee.c"))
        .arg(dir.join("caller.c"))
        .arg(dir.join("main.c"))
        .arg("-o")
        .arg(&reference))
    .unwrap();
    let want = run(&mut Command::new(&reference)).unwrap();
    let mut failures = Vec::new();
    for (callee, caller) in [(By::Clang, By::Clang), (By::Us, By::Clang), (By::Clang, By::Us), (By::Us, By::Us)] {
        let label = format!("callee by {callee:?}, caller by {caller:?}");
        let result = (|| -> Result<(), String> {
            let a = build(&t, &dir, &tmp, "callee", callee)?;
            let b = build(&t, &dir, &tmp, "caller", caller)?;
            let exe = tmp.join(format!("abi.{callee:?}.{caller:?}"));
            run(Command::new("clang").arg(&a).arg(&b).arg(dir.join("main.c")).arg("-o").arg(&exe))?;
            let got = run(&mut Command::new(&exe))?;
            if got != want {
                return Err(format!("output differs:\n{got}\nwanted:\n{want}"));
            }
            Ok(())
        })();
        match result {
            Ok(()) => println!("ok {label}"),
            Err(e) => failures.push(format!("{label}: {e}")),
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
