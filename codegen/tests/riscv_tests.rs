//! The RISC-V ISA tests (riscv-tests, `isa/`, the physical-memory `p`
//! environment) assembled by epitome-as, linked by ld.lld and run on
//! QEMU's spike machine, each as clang assembles it too.
//!
//! Every test must assemble; and where clang's object of it links, ours
//! must end as clang's does: a test passing on QEMU with clang's must
//! pass with ours, and one failing (an extension this QEMU does not have:
//! Zabha, Zacas, Zalasr) must fail alike. The tests are preprocessed by
//! clang, as their makefile has gcc do. clang will not have a `.weak`
//! symbol then defined `.global`, as the trap handlers are, so its copy is
//! without the `.weak`.
//!
//! The test runs where `EPITOME_RISCV_TESTS` is a riscv-tests checkout,
//! and clang, ld.lld and qemu-system-riscv64 are installed; else it is
//! skipped.

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use epitome_codegen::abi::Syntax;
use epitome_codegen::assembler::Assembler;
use epitome_codegen::target::Target;

/// The test groups (riscv-tests' directories) and their `-march`: P is not
/// ratified, and the hypervisor's are in directories of their own.
fn march(dir: &str) -> Option<&'static str> {
    Some(match dir {
        "rv64up" => return None,
        "rv64ui" | "rv64uc" | "rv64um" | "rv64ua" | "rv64uf" | "rv64ud" | "rv64uziccid" | "rv64mi" | "rv64si"
        | "rv64ssvnapot" => "rv64g",
        "rv64mzicbo" => "rv64g_zicboz",
        "rv64uzabha" => "rv64g_zabha",
        "rv64uzabha_zacas" => "rv64g_zacas_zabha",
        "rv64uzacas" => "rv64g_zacas",
        "rv64uzalasr" => "rv64g_zalasr",
        "rv64uzfh" => "rv64g_zfh",
        "rv64uzba" => "rv64g_zba",
        "rv64uzbb" => "rv64g_zbb",
        "rv64uzbc" => "rv64g_zbc",
        "rv64uzbkb" => "rv64g_zbkb",
        "rv64uzbkx" => "rv64g_zbkx",
        "rv64uzbs" => "rv64g_zbs",
        "rv64uzicond" => "rv64g_zicond",
        d if d.starts_with("hypervisor") => "rv64gh",
        _ => return None,
    })
}

/// The tests of a group, from its Makefrag (`rv64ui_sc_tests = add addi ...`).
fn tests(makefrag: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut on = false;
    for line in makefrag.lines() {
        let words = match line.split_once('=') {
            Some((var, rest)) if var.trim().ends_with("_sc_tests") && !var.contains('$') => rest,
            _ if on => line,
            _ => continue,
        };
        out.extend(words.split_whitespace().filter(|w| *w != "\\" && !w.contains('$')).map(str::to_string));
        on = line.trim_end().ends_with('\\');
    }
    out
}

/// How a test ended: QEMU's exit status (0 passes), or why it did not run.
#[derive(Clone, Debug, PartialEq)]
enum End {
    Exit(i32),
    NotBuilt(String),
    TimedOut,
}

fn qemu(elf: &Path) -> End {
    let mut child = match Command::new("qemu-system-riscv64")
        .args(["-nographic", "-machine", "spike", "-cpu", "max", "-bios", "none", "-kernel"])
        .arg(elf)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
    {
        Ok(c) => c,
        Err(e) => return End::NotBuilt(format!("qemu: {e}")),
    };
    let start = Instant::now();
    loop {
        if let Ok(Some(status)) = child.try_wait() {
            return End::Exit(status.code().unwrap_or(-1));
        }
        if start.elapsed() > Duration::from_secs(30) {
            let _ = child.kill();
            let _ = child.wait();
            return End::TimedOut;
        }
        std::thread::sleep(Duration::from_millis(20));
    }
}

fn link(root: &Path, obj: &Path, elf: &Path) -> Result<(), String> {
    let out = Command::new("ld.lld")
        .arg("--no-relax")
        .arg("-T")
        .arg(root.join("env/p/link.ld"))
        .arg(obj)
        .arg("-o")
        .arg(elf)
        .output()
        .map_err(|e| e.to_string())?;
    out.status.success().then_some(()).ok_or_else(|| String::from_utf8_lossy(&out.stderr).into_owned())
}

/// One test: how it ends with clang's object, and with ours.
fn one(t: &Target, root: &Path, tmp: &Path, dir: &str, name: &str, march: &str) -> (End, End) {
    let w = tmp.join(format!("{dir}-{name}"));
    std::fs::create_dir_all(&w).unwrap();
    let abi = if matches!(dir, "rv64uzbkb" | "rv64uzbkx" | "rv64uzicond") { "lp64" } else { "lp64d" };
    let clang = |args: &[&str]| {
        let mut c = Command::new("clang");
        c.args(["--target=riscv64-unknown-elf", &format!("-march={march}"), &format!("-mabi={abi}")]).args(args);
        c
    };
    let src = root.join(format!("isa/{dir}/{name}.S"));
    let (env_p, macros) = (root.join("env/p"), root.join("isa/macros/scalar"));
    let pp = clang(&["-E", "-x", "assembler-with-cpp"])
        .arg(format!("-I{}", env_p.display()))
        .arg(format!("-I{}", macros.display()))
        .arg(&src)
        .output()
        .unwrap();
    if !pp.status.success() {
        let e = End::NotBuilt(String::from_utf8_lossy(&pp.stderr).into_owned());
        return (e.clone(), e);
    }
    let text = String::from_utf8_lossy(&pp.stdout).into_owned();
    // Clang's.
    let mut theirs = text.clone();
    for line in text.split([';', '\n']) {
        if let Some(sym) = line.trim().strip_prefix(".global ") {
            theirs = theirs.replace(&format!(".weak {};", sym.trim()), "");
        }
    }
    std::fs::write(w.join("theirs.s"), &theirs).unwrap();
    let built =
        clang(&["-mno-relax", "-c"]).arg(w.join("theirs.s")).arg("-o").arg(w.join("theirs.o")).output().unwrap();
    let reference = if !built.status.success() {
        End::NotBuilt(String::from_utf8_lossy(&built.stderr).lines().next().unwrap_or("").to_string())
    } else {
        match link(root, &w.join("theirs.o"), &w.join("theirs.elf")) {
            Ok(()) => qemu(&w.join("theirs.elf")),
            Err(e) => End::NotBuilt(e),
        }
    };
    // Ours.
    let ours = match Assembler::new(t).assemble(&text, Some(Syntax::Elf)).and_then(|a| a.file(t)) {
        Err(e) => End::NotBuilt(format!("epitome-as: {}", e.lines().take(3).collect::<Vec<_>>().join("; "))),
        Ok(bytes) => {
            std::fs::write(w.join("ours.o"), bytes).unwrap();
            match link(root, &w.join("ours.o"), &w.join("ours.elf")) {
                Ok(()) => qemu(&w.join("ours.elf")),
                Err(e) => End::NotBuilt(format!("ld.lld: {e}")),
            }
        }
    };
    (reference, ours)
}

#[test]
fn riscv_tests_run_on_qemu() {
    let Some(root) = std::env::var_os("EPITOME_RISCV_TESTS").map(PathBuf::from) else {
        eprintln!("EPITOME_RISCV_TESTS is not set; skipping");
        return;
    };
    for tool in ["clang", "ld.lld", "qemu-system-riscv64"] {
        if Command::new(tool).arg("--version").output().is_err() {
            eprintln!("{tool} is not installed; skipping");
            return;
        }
    }
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let t = Target::load_path(manifest.join("targets/riscv64").to_str().unwrap()).unwrap();
    let tmp = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("riscv-tests");
    let mut work = Vec::new();
    let mut dirs: Vec<PathBuf> = std::fs::read_dir(root.join("isa")).unwrap().map(|e| e.unwrap().path()).collect();
    dirs.sort();
    for d in dirs {
        let dir = d.file_name().unwrap().to_string_lossy().into_owned();
        let (Some(march), Ok(frag)) = (march(&dir), std::fs::read_to_string(d.join("Makefrag"))) else { continue };
        work.extend(tests(&frag).into_iter().map(|name| (dir.clone(), name, march)));
    }
    assert!(!work.is_empty(), "no tests under {}", root.display());
    // Run on every core.
    let threads = std::thread::available_parallelism().map_or(4, |n| n.get());
    let next = std::sync::atomic::AtomicUsize::new(0);
    let results = std::sync::Mutex::new(Vec::new());
    std::thread::scope(|s| {
        for _ in 0..threads {
            s.spawn(|| {
                loop {
                    let k = next.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    let Some((dir, name, march)) = work.get(k) else { break };
                    let ends = one(&t, &root, &tmp, dir, name, march);
                    results.lock().unwrap().push((format!("{dir}-{name}"), ends));
                }
            });
        }
    });
    let mut results = results.into_inner().unwrap();
    results.sort_by(|a, b| a.0.cmp(&b.0));
    let mut wrong = Vec::new();
    let (mut passed, mut alike, mut theirs_unbuilt) = (0, 0, 0);
    for (name, (theirs, ours)) in &results {
        match (theirs, ours) {
            (_, End::NotBuilt(e)) => wrong.push(format!("{name}: {e}")),
            (End::NotBuilt(_), End::Exit(0)) => {
                theirs_unbuilt += 1;
                passed += 1;
            }
            (End::NotBuilt(_), _) => theirs_unbuilt += 1,
            (a, b) if a == b => {
                alike += 1;
                passed += (*b == End::Exit(0)) as usize;
            }
            (a, b) => wrong.push(format!("{name}: clang's ends {a:?}, ours {b:?}")),
        }
    }
    eprintln!(
        "{} tests: {passed} pass with ours; {alike} end as clang's do; {theirs_unbuilt} clang does not build",
        results.len()
    );
    assert!(wrong.is_empty(), "{} differ:\n{}", wrong.len(), wrong.join("\n"));
}
