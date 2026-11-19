//! x86-64's selections for what used to go lane by lane through the stack
//! on the baseline (SSE2) target (`tests/x86_isel/`): the high halves of
//! 16-bit products as `pmulhuw` and `pmulhw` (`vpmulhuw` with AVX2), then a
//! shift; narrowing in registers (with AVX2, of a ymm register's lanes too); a scalar into a vector of zeros as `movq`
//! or `movd`; and 16-aligned loads folded into SSE2's operands. Each
//! function's assembly is checked for its instructions and for no stack
//! access, and on x86-64 Linux with clang, our build is run against clang's.

use std::process::Command;

use epitome_codegen::target::Target;
use epitome_codegen::{Options, compile};

#[test]
fn multiply_high() {
    check(
        "mulhi",
        &[
            ("mulhu_8", &["pmulhuw %xmm"], &["pmuludq", "punpck"]),
            ("mulhs_8", &["pmulhw %xmm"], &["pmuludq", "punpck"]),
            ("mulhu_19", &["pmulhuw", "psrlw $3"], &["pmuludq"]),
            ("mulhs_18", &["pmulhw", "psraw $2"], &["pmuludq"]),
            ("mulhu_const", &["pmulhuw", "psrlw $2"], &["pmuludq"]),
            ("mulhu_16", &["pmulhuw %xmm2", "pmulhuw %xmm3"], &["pmuludq"]),
            ("mulhs_4", &["pmulhw %xmm"], &["pmuludq"]),
            ("mulhu_16_avx2", &["vpmulhuw %ymm", "vpsrlw $1"], &["vpmulld"]),
        ],
    );
}

#[test]
fn narrowing() {
    check(
        "trunc",
        &[
            ("trunc_8x32_16", &["packssdw"], &[]),
            ("trunc_4x32_16", &["pshuflw $8", "pshufhw $8", "pshufd $8"], &[]),
            ("trunc_16x16_8", &["packuswb"], &[]),
            ("trunc_8x16_8", &["packuswb"], &[]),
            ("trunc_4x64_32", &["shufps $136"], &[]),
            ("trunc_2x64_32", &["pshufd $8"], &[]),
            ("trunc_16x32_8", &["packssdw", "packuswb"], &[]),
            ("trunc_8x32_16_avx2", &["vpsrad $16", "vextracti128 $1", "packssdw"], &[]),
            ("trunc_16x16_8_avx2", &["vpsrlw $8", "vextracti128 $1", "packuswb"], &[]),
            ("trunc_4x64_32_avx2", &["vextracti128 $1", "shufps $136"], &[]),
        ],
    );
}

#[test]
fn scalar_into_zeros() {
    check("insert", &[("insert_q", &["movq %rdi, %xmm"], &[]), ("insert_d", &["movd %edi, %xmm"], &[])]);
}

#[test]
fn folded_loads() {
    check(
        "fold",
        &[
            ("fold_aligned", &["paddw 16(%rdi)", "pmullw 32(%rdi)", "pxor 48(%rdi)"], &[]),
            ("fold_unaligned", &["movups 16(%rdi)"], &["paddw 16("]),
        ],
    );
}

/// Compiles `tests/x86_isel/{name}.ll` for the baseline x86-64; checks
/// that each function's assembly has each of its instructions, none of
/// those it must not, and nothing through the stack; and, where it can,
/// runs it against clang's build with `{name}_main.c`.
fn check(name: &str, funcs: &[(&str, &[&str], &[&str])]) {
    let dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/x86_isel");
    let ll = std::fs::read_to_string(dir.join(format!("{name}.ll"))).unwrap();
    let t = Target::x86_64();
    let mut m = epitome_ir::parse_module(&ll).unwrap();
    let c = compile(&t, &mut m, &Options::default()).unwrap();
    for (f, want, not) in funcs {
        let body = function(&c.asm, f);
        for i in *want {
            assert!(body.contains(i), "@{f}: no {i}:\n{body}");
        }
        for i in *not {
            assert!(!body.contains(i), "@{f}: {i}:\n{body}");
        }
        assert!(!body.contains("(%rsp)"), "@{f}: through the stack:\n{body}");
    }
    let clang = Command::new("clang").arg("--version").output().is_ok_and(|o| o.status.success());
    if !(cfg!(target_arch = "x86_64") && cfg!(target_os = "linux") && clang) {
        eprintln!("needs clang on x86-64 Linux to run; checked the assembly only");
        return;
    }
    let tmp = std::path::PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!("x86-isel-{name}"));
    std::fs::create_dir_all(&tmp).unwrap();
    let ours = tmp.join("ours.o");
    std::fs::write(&ours, c.object_file(&t).unwrap()).unwrap();
    let theirs = tmp.join("theirs.o");
    let st = Command::new("clang").args(["-O1", "-c"]).arg(dir.join(format!("{name}.ll"))).arg("-o").arg(&theirs).status().unwrap();
    assert!(st.success());
    let mut out = Vec::new();
    for (who, obj) in [("ours", &ours), ("theirs", &theirs)] {
        let exe = tmp.join(who);
        let st = Command::new("clang").arg(obj).arg(dir.join(format!("{name}_main.c"))).arg("-o").arg(&exe).status().unwrap();
        assert!(st.success());
        let o = Command::new(&exe).output().unwrap();
        assert!(o.status.success());
        out.push(String::from_utf8_lossy(&o.stdout).into_owned());
    }
    assert_eq!(out[0], out[1], "our build of {name}.ll differs from clang's");
}

/// Function `f`'s lines of `asm`: from its label to the next function's.
fn function<'a>(asm: &'a str, f: &str) -> &'a str {
    let start = asm.find(&format!("\n{f}:")).unwrap_or_else(|| panic!("no @{f} in:\n{asm}"));
    let rest = &asm[start + 1..];
    let end = rest.find(".cfi_endproc").unwrap_or(rest.len());
    &rest[..end]
}
