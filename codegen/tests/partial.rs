//! A partial machine (`docs/design/partial-machines.md`): `tests/partial/
//! tiny.ll`, a made-up control core with registers, arithmetic and
//! branches, and no memory, stack, calls, floating point, 64-bit integers,
//! branch through a register or trap.
//!
//! Its description passes `--check-target` with what it lacks waived;
//! Rust's `fib` (`tests/partial/fib.rs`, as rustc writes its IR) and some
//! IR of our own compile for it and run, on a small interpreter of its
//! instructions here, to the right answers; and programs that use what it
//! lacks are turned away, each use named.

use std::path::PathBuf;
use std::process::Command;

use epitome_codegen::assembler::Assembler;
use epitome_codegen::target::Target;
use epitome_codegen::{Options, check, compile, mir};

fn dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/partial")
}

fn tiny() -> Target {
    Target::load_path(dir().join("tiny.ll").to_str().unwrap()).unwrap()
}

/// Each function of `ir` compiled for the machine, by name: its code.
/// The object file must be what the assembler makes of the assembly.
fn compiled(t: &Target, ir: &str) -> Result<Vec<(String, Vec<u8>)>, String> {
    let mut m = epitome_ir::parse_module(ir).map_err(|e| format!("{e:?}"))?;
    let c = compile(t, &mut m, &Options::default())?;
    let bytes = c.object_file(t)?;
    let assembled = Assembler::new(t).assemble(&c.asm, None)?.file(t)?;
    assert_eq!(bytes, assembled, "the object assembled from the assembly differs:\n{}", c.asm);
    Ok(c.funcs.iter().map(|f| (f.name.clone(), mir::encode_function(t, f).unwrap().bytes)).collect())
}

/// Runs code for the machine from its start, with `args` in r1..., to
/// `halt`: r1. (The machine as `tiny.ll` describes it.)
fn run(code: &[u8], args: &[u32]) -> u32 {
    let mut r = [0u32; 16];
    r[1..=args.len()].copy_from_slice(args);
    let mut pc = 0usize;
    for _ in 0..100_000 {
        let w = u32::from_le_bytes(code[pc..pc + 4].try_into().unwrap());
        let op = w >> 26;
        let (d, a, b) = ((w >> 22 & 15) as usize, (w >> 18 & 15) as usize, (w >> 14 & 15) as usize);
        let imm = ((w << 14) as i32 >> 14) as u32;
        let (x, y) = (r[a], r[b]);
        let mut next = pc + 4;
        let mut set = |v: u32| r[d] = v;
        match op {
            0x01 => set(x.wrapping_add(y)),
            0x02 => set(x.wrapping_sub(y)),
            0x03 => set(x & y),
            0x04 => set(x | y),
            0x05 => set(x ^ y),
            0x06 => set(x.wrapping_shl(y)),
            0x07 => set(x.wrapping_shr(y)),
            0x08 => set((x as i32).wrapping_shr(y) as u32),
            0x09 => set(x.wrapping_mul(y)),
            0x0a => set(((x as i32) < (y as i32)) as u32),
            0x0b => set((x < y) as u32),
            0x0c => set((x == y) as u32),
            0x0d => set((x as i32).wrapping_div(y as i32) as u32),
            0x0e => set(x / y),
            0x0f => set((x as i32).wrapping_rem(y as i32) as u32),
            0x17 => set(x % y),
            0x10 => set(x.wrapping_add(imm)),
            0x11 => set(x & imm),
            0x12 => set(x | imm),
            0x13 => set(x ^ imm),
            0x14 => set(x << (w & 31)),
            0x15 => set(x >> (w & 31)),
            0x16 => set(((x as i32) >> (w & 31)) as u32),
            0x18 => set(imm),
            0x19 => set(imm << 14),
            0x1f => set(x),
            0x20..=0x28 => {
                let (p, q) = (r[d], r[a]);
                let taken = match op {
                    0x20 => p == q,
                    0x21 => p != q,
                    0x22 => (p as i32) < (q as i32),
                    0x23 => (p as i32) >= (q as i32),
                    0x24 => p < q,
                    0x25 => p >= q,
                    0x26 => p == 0,
                    0x27 => p != 0,
                    _ => true,
                };
                if taken {
                    next = (pc as i64 + imm as i32 as i64 * 4) as usize;
                }
            }
            0x3f => return r[1],
            _ => panic!("no instruction {w:08x} at {pc}"),
        }
        pc = next;
    }
    panic!("did not halt")
}

fn code_of<'a>(funcs: &'a [(String, Vec<u8>)], name: &str) -> &'a [u8] {
    &funcs.iter().find(|(n, _)| n == name).unwrap_or_else(|| panic!("no {name}")).1
}

fn fib(n: u32) -> u32 {
    (0..n).fold((0u32, 1u32), |(a, b), _| (b, a.wrapping_add(b))).0
}

/// The description's own checks pass, with what it lacks waived, and the
/// smoke functions that use what it lacks are noted, not failed.
#[test]
fn check_target_passes() {
    let report = check::check(&tiny());
    assert_eq!(report.errors(), 0, "{}", report.render(true));
    let text = report.render(true);
    assert!(text.contains("requirements waived"), "{text}");
    assert!(text.contains("do not fit the machine"), "{text}");
}

/// Rust's fib, as rustc wrote its IR, compiles and runs.
#[test]
fn rust_fib_runs() {
    let t = tiny();
    let funcs = compiled(&t, &std::fs::read_to_string(dir().join("fib.ll")).unwrap()).unwrap();
    let code = code_of(&funcs, "fib");
    for n in 0..30 {
        assert_eq!(run(code, &[n]), fib(n), "fib({n})");
    }
}

/// The same, from rustc now, where it has the bare-metal RISC-V target.
#[test]
fn rust_fib_from_rustc() {
    let tmp = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("partial-fib");
    std::fs::create_dir_all(&tmp).unwrap();
    let ll = tmp.join("fib.ll");
    let ok = Command::new("rustc")
        .args(["--edition", "2021", "--crate-type=lib", "--target", "riscv64imac-unknown-none-elf"])
        .args(["-C", "opt-level=2", "-C", "panic=abort", "--emit=llvm-ir", "-o"])
        .arg(&ll)
        .arg(dir().join("fib.rs"))
        .output()
        .is_ok_and(|o| o.status.success());
    if !ok {
        eprintln!("skipped: rustc has no riscv64imac-unknown-none-elf");
        return;
    }
    let t = tiny();
    let funcs = compiled(&t, &std::fs::read_to_string(&ll).unwrap()).unwrap();
    assert_eq!(run(code_of(&funcs, "fib"), &[24]), fib(24));
}

/// A machine whose branches only compare a register with zero (as
/// Alpha's do): the toy without its two-register `beq` and `bne`, and
/// recipes for them, a subtraction and `bz` or `bnz`. Rust's fib compiles
/// and runs on it.
#[test]
fn branch_recipes() {
    let mut text = std::fs::read_to_string(dir().join("tiny.ll")).unwrap();
    for name in ["beq", "bne"] {
        let start = text.find(&format!("define void @{name}(")).unwrap();
        let end = start + text[start..].find("\n}\n").unwrap() + 3;
        text.replace_range(start..end, "");
    }
    text.push_str(
        r#"
define void @beq.rr(i32 "reg"="r" %a, i32 "reg"="r" %b, ptr "label" %l) "recipe"="beq.rr.expand" {
  %c = icmp eq i32 %a, %b
  br i1 %c, label %t, label %n
t:
  indirectbr ptr %l, []
n:
  ret void
}
define void @beq.rr.expand(i32 %a, i32 %b, ptr %l) {
  %d = call i32 @sub(i32 %a, i32 %b)
  call void @bz(i32 %d, ptr %l)
  ret void
}
define void @bne.rr(i32 "reg"="r" %a, i32 "reg"="r" %b, ptr "label" %l) "recipe"="bne.rr.expand" {
  %c = icmp ne i32 %a, %b
  br i1 %c, label %t, label %n
t:
  indirectbr ptr %l, []
n:
  ret void
}
define void @bne.rr.expand(i32 %a, i32 %b, ptr %l) {
  %d = call i32 @sub(i32 %a, i32 %b)
  call void @bnz(i32 %d, ptr %l)
  ret void
}
"#,
    );
    let t = Target::load(&text).unwrap();
    let funcs = compiled(&t, &std::fs::read_to_string(dir().join("fib.ll")).unwrap()).unwrap();
    let code = code_of(&funcs, "fib");
    for n in 0..30 {
        assert_eq!(run(code, &[n]), fib(n), "fib({n})");
    }
}

/// What has no instruction of its own (recipes, extensions, division,
/// constants) computes what the IR does.
#[test]
fn recipes_run() {
    let ir = r#"
define i32 @smin(i32 %a, i32 %b) {
  %r = call i32 @llvm.smin.i32(i32 %a, i32 %b)
  ret i32 %r
}
define i32 @umax(i32 %a, i32 %b) {
  %r = call i32 @llvm.umax.i32(i32 %a, i32 %b)
  ret i32 %r
}
define i32 @sel(i32 %a, i32 %b) {
  %c = icmp sgt i32 %a, %b
  %r = select i1 %c, i32 %a, i32 %b
  ret i32 %r
}
define i32 @ne(i32 %a, i32 %b) {
  %c = icmp ne i32 %a, %b
  %r = zext i1 %c to i32
  ret i32 %r
}
define i32 @ule(i32 %a, i32 %b) {
  %c = icmp ule i32 %a, %b
  %r = zext i1 %c to i32
  ret i32 %r
}
define i32 @sext8(i32 %a, i32 %b) {
  %t = trunc i32 %a to i8
  %r = sext i8 %t to i32
  ret i32 %r
}
define i32 @div7(i32 %a, i32 %b) {
  %r = sdiv i32 %a, 7
  ret i32 %r
}
define i32 @udiv8(i32 %a, i32 %b) {
  %r = udiv i32 %a, 8
  ret i32 %r
}
define i32 @big(i32 %a, i32 %b) {
  %r = add i32 %a, 305419896
  ret i32 %r
}
define i32 @abs(i32 %a, i32 %b) {
  %r = call i32 @llvm.abs.i32(i32 %a, i1 false)
  ret i32 %r
}
declare i32 @llvm.smin.i32(i32, i32)
declare i32 @llvm.umax.i32(i32, i32)
declare i32 @llvm.abs.i32(i32, i1)
"#;
    let t = tiny();
    let funcs = compiled(&t, ir).unwrap();
    let want: &[(&str, fn(i32, i32) -> i32)] = &[
        ("smin", |a, b| a.min(b)),
        ("umax", |a, b| (a as u32).max(b as u32) as i32),
        ("sel", |a, b| if a > b { a } else { b }),
        ("ne", |a, b| (a != b) as i32),
        ("ule", |a, b| (a as u32 <= b as u32) as i32),
        ("sext8", |a, _| a as i8 as i32),
        ("div7", |a, _| a / 7),
        ("udiv8", |a, _| ((a as u32) / 8) as i32),
        ("big", |a, _| a.wrapping_add(305419896)),
        ("abs", |a, _| a.wrapping_abs()),
    ];
    let vals = [0, 1, -1, 2, -7, 7, 100, -100, 127, 128, -129, 65535, i32::MAX, i32::MIN];
    for (name, f) in want {
        let code = code_of(&funcs, name);
        for &a in &vals {
            for &b in &vals {
                assert_eq!(run(code, &[a as u32, b as u32]) as i32, f(a, b), "{name}({a}, {b})");
            }
        }
    }
}

/// A program that uses what the machine lacks is turned away, with each
/// use named, and the functions that fit are compiled.
#[test]
fn what_does_not_fit_is_named() {
    let ir = r#"
define i32 @stack(i32 %a) {
  %p = alloca i32
  store i32 %a, ptr %p
  %r = load i32, ptr %p
  ret i32 %r
}
define i32 @calls(i32 %a) {
  %r = call i32 @elsewhere(i32 %a)
  ret i32 %r
}
define float @float(float %a) {
  %r = fadd float %a, 1.0
  ret float %r
}
define i64 @wide(i64 %a) {
  %r = add i64 %a, 1
  ret i64 %r
}
define i32 @many(i32 %a, i32 %b, i32 %c, i32 %d, i32 %e) {
  %r = add i32 %a, %e
  ret i32 %r
}
define i32 @fine(i32 %a) {
  %r = add i32 %a, 1
  ret i32 %r
}
declare i32 @elsewhere(i32)
"#;
    let t = tiny();
    let mut m = epitome_ir::parse_module(ir).unwrap();
    let c = compile(&t, &mut m, &Options { keep_going: true, ..Options::default() }).unwrap();
    let errors = c.errors.join("\n");
    for want in [
        "@stack: does not fit: block %0, %p = alloca: an alloca, and the machine has no stack",
        "@stack: does not fit: block %0, store: the machine has no memory",
        "@calls: does not fit: block %0, %r = call: a call to @elsewhere, and the machine has no calls",
        "@float: does not fit: the result: a floating-point value, and the machine has no floating point",
        "@wide: does not fit: block %0, %r = add: an i64, and the machine's integers are at most 32 bits",
        "@many: does not fit: 5 arguments: only 4 are passed in registers, and the machine has no stack",
    ] {
        assert!(errors.contains(want), "no `{want}` in:\n{errors}");
    }
    assert_eq!(c.funcs.iter().map(|f| f.name.as_str()).collect::<Vec<_>>(), ["fine"]);
}
