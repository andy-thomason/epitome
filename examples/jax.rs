//! A block of a model written in JAX, compiled by Epitome into an
//! executable that checks itself against JAX: for AArch64 macOS (Mach-O) or
//! x86-64 Linux (ELF), whichever this runs on.
//!
//! `examples/jax/export.py` exports the block (attention over 64
//! positions: a projection, GELU, a leaky ReLU, four heads, a causal
//! softmax, an output projection) as StableHLO, with an input and JAX's
//! output. This program takes it the rest of the way, every stage in
//! Epitome:
//!
//! 1. `stablehlo-to-linalg`: StableHLO to `linalg` on tensors;
//! 2. the passes that follow for a model from PyTorch (`examples/llm.rs`):
//!    `linalg-generalize-named-ops`, `linalg-fold-constants`, `bufferize`,
//!    `outline-linalg`, `linalg-vectorize=4`, `linalg-to-loops`,
//!    `math-polynomial-approximation` and `to-llvm`;
//! 3. translation to LLVM IR, `epitome-opt`'s pipeline, and code generation
//!    for AArch64 or x86-64, to an object, `block.o`;
//!
//! then links it with the Rust program in `examples/jax/driver.rs`, which
//! runs it and prints how far its output is from JAX's.
//!
//! With `--sme`, matrix products go to the SME kernels of
//! `mlir/targets/aarch64-sme.mlir` and the code generator targets
//! `apple-m4`; `--weights=bf16` stores the constant weights in BF16 for the
//! kernels that take them so. `--dump` writes each stage next to the block.
//!
//! On x86-64, as in `examples/llm.rs`, the code generator targets AVX2 and
//! FMA, vectors of eight (`linalg-vectorize=8`), and `--avx512` is
//! `--sme`'s counterpart: matrix products by the kernels of
//! `mlir/targets/x86-avx512.mlir`, which need AVX-512 and AVX512-BF16 and
//! so `--weights=bf16`.
//!
//! ```sh
//! python examples/jax/export.py target/jax      # with JAX installed
//! cargo run --release --example jax -- target/jax --sme
//! target/jax/jax
//! ```
//!
//! On an AArch64 Mac or an x86-64 Linux machine, with `rustc` to build the
//! driver.

use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

use epitome_codegen::target::Target;
use epitome_codegen::{Options, compile};

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let dump = args.iter().any(|a| a == "--dump");
    let sme = args.iter().any(|a| a == "--sme" || a == "--avx512");
    let weights = args.iter().find_map(|a| a.strip_prefix("--weights=")).map(str::to_string);
    let dir = args
        .iter()
        .find(|a| !a.starts_with("--"))
        .map(PathBuf::from)
        .unwrap_or_else(|| Path::new(env!("CARGO_MANIFEST_DIR")).join("target/jax"));
    match build(&dir, dump, sme, weights.as_deref()) {
        Ok(exe) => {
            println!("built {}", exe.display());
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("jax: {e}");
            ExitCode::FAILURE
        }
    }
}

/// The passes from StableHLO to the `llvm` dialect, with the SME kernels
/// (AVX-512's on x86-64; of the format `weights`, if given) when `sme` is
/// set.
pub fn passes(sme: bool, weights: Option<&str>) -> Vec<String> {
    let x86 = cfg!(target_arch = "x86_64");
    let set = if x86 { "x86-avx512" } else { "aarch64-sme" };
    let mut passes = vec!["stablehlo-to-linalg".to_string()];
    passes.extend(epitome_mlir::LINALG_TO_LLVM.iter().map(|p| p.to_string()));
    let mut at = passes.iter().position(|p| p == "bufferize").unwrap() + 1;
    if sme {
        let kernels = match weights {
            Some(w) => format!("linalg-to-kernels={set}:{w}"),
            None => format!("linalg-to-kernels={set}"),
        };
        passes.insert(at, kernels);
        at += 1;
    }
    passes.insert(at, "outline-linalg".into());
    passes.insert(at + 1, if x86 { "linalg-vectorize=8" } else { "linalg-vectorize=4" }.into());
    let at = passes.iter().position(|p| p == "to-llvm").unwrap();
    passes.insert(at, "math-polynomial-approximation".into());
    passes
}

fn build(dir: &Path, dump: bool, sme: bool, weights: Option<&str>) -> Result<PathBuf, String> {
    let path = dir.join("block.mlir");
    let src = std::fs::read_to_string(&path)
        .map_err(|e| format!("{}: {e} (run examples/jax/export.py first)", path.display()))?;
    let mut m = epitome_mlir::parse_module(&src).map_err(|e| format!("{}:{e}", path.display()))?;
    for (k, pass) in passes(sme, weights).iter().enumerate() {
        m = epitome_mlir::run_pass(pass, &m).map_err(|e| format!("{pass}: {e}"))?;
        println!("{pass}");
        if dump {
            let name = pass.split('=').next().unwrap_or(pass);
            let out = dir.join(format!("stage{:02}-{name}.mlir", k + 1));
            std::fs::write(&out, m.to_string()).map_err(|e| format!("{}: {e}", out.display()))?;
        }
    }
    let mut ir = epitome_mlir::translate::translate(&m).map_err(|e| format!("translate: {e}"))?;
    let x86 = cfg!(target_arch = "x86_64");
    ir.triple = Some(if x86 { b"x86_64-unknown-linux-gnu".to_vec() } else { b"arm64-apple-macosx14.0.0".to_vec() });
    epitome_opt::pipeline::run(&mut ir, &epitome_opt::pipeline::Options::default());
    if dump {
        let out = dir.join("block.ll");
        std::fs::write(&out, ir.to_string()).map_err(|e| format!("{}: {e}", out.display()))?;
    }
    let (t, opts) = if x86 {
        (Target::x86_64(), Options { mattr: vec!["+avx2,+fma".to_string()], ..Options::default() })
    } else {
        (Target::aarch64(), Options { mcpu: sme.then(|| "apple-m4".to_string()), ..Options::default() })
    };
    let compiled = compile(&t, &mut ir, &opts)?;
    let suffix = weights.map(|w| format!("-{w}")).unwrap_or_default();
    let object = dir.join(format!("block{suffix}.o"));
    std::fs::write(&object, compiled.object_file(&t)?).map_err(|e| format!("{}: {e}", object.display()))?;

    let driver = Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/jax/driver.rs");
    let exe = dir.join(format!("jax{suffix}"));
    let rustc = std::env::var("RUSTC").unwrap_or_else(|_| "rustc".to_string());
    let built = Command::new(&rustc)
        .args(["--edition", "2024", "-O", "-C"])
        .arg(format!("link-arg={}", object.display()))
        .arg("-o")
        .arg(&exe)
        .arg(&driver)
        .output()
        .map_err(|e| format!("{rustc}: {e}"))?;
    if !built.status.success() {
        return Err(format!("building the driver failed:\n{}", String::from_utf8_lossy(&built.stderr)));
    }
    Ok(exe)
}
