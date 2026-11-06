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
//!
//! Also: wide integers narrowed to its 32 bits (`epitome_ir::narrow`);
//! calls to `"epitome-opaque"` functions made launches; what `prepare`
//! adds checked by `fits`; and, on RV64 described without calls, memory
//! operations as loops (run on `riscv/emu.rs`'s machine).

#[path = "riscv/emu.rs"]
#[allow(dead_code)]
mod emu;

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
    run_with(code, args, &mut Vec::new())
}

/// `run`, with a list of the launches made: where, and r1-r4 when they
/// were.
fn run_with(code: &[u8], args: &[u32], launches: &mut Vec<(usize, [u32; 4])>) -> u32 {
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
            0x29 => launches.push((pc, [r[1], r[2], r[3], r[4]])),
            // The machine's own: a cycle count (1234 here), and two that
            // are recorded with the launches, tagged.
            0x2a => set(1234),
            0x2b => launches.push((pc, [0x516, x, imm, 0])),
            0x2c => launches.push((pc, [0x92, r[5], r[6], 0])),
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

/// RV64 as `targets/riscv64` describes it, but without calls.
fn riscv_without_calls() -> Target {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("targets/riscv64");
    let mut files: Vec<(String, String)> = std::fs::read_dir(&root)
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|x| x == "ll"))
        .map(|p| (p.file_name().unwrap().to_string_lossy().into_owned(), std::fs::read_to_string(&p).unwrap()))
        .collect();
    files.sort();
    files.push(("99-lacks.ll".into(), "!target.lacks = !{!0}\n!0 = !{!\"calls\"}\n".into()));
    let refs: Vec<(&str, &str)> = files.iter().map(|(n, t)| (n.as_str(), t.as_str())).collect();
    Target::load_files(&refs).unwrap()
}

/// Without calls, the memcpy, memmove and memset that would be calls to
/// the C library are loops (on RV64 without calls, run on the RV64
/// machine of `riscv/emu.rs`); a memmove copies the right way round when
/// its ranges overlap.
#[test]
fn memory_operations_are_loops() {
    // Each length is read from a volatile global, so is not known, and a
    // memcpy, memmove or memset of it is not written out as stores.
    let ir = r#"
target datalayout = "e-m:e-p:64:64-i64:64-i128:128-n32:64-S128"
target triple = "riscv64-unknown-linux-gnu"
@buf = global [64 x i8] zeroinitializer
@len = global i64 0
define i32 @main() {
entry:
  br label %init
init:
  %i = phi i64 [ 0, %entry ], [ %i1, %init ]
  %p = getelementptr i8, ptr @buf, i64 %i
  %b = trunc i64 %i to i8
  store i8 %b, ptr %p
  %i1 = add i64 %i, 1
  %more = icmp ult i64 %i1, 64
  br i1 %more, label %init, label %go
go:
  store volatile i64 10, ptr @len
  %n1 = load volatile i64, ptr @len
  %d1 = getelementptr i8, ptr @buf, i64 40
  call void @llvm.memcpy.p0.p0.i64(ptr %d1, ptr @buf, i64 %n1, i1 false)
  store volatile i64 20, ptr @len
  %n2 = load volatile i64, ptr @len
  %d2 = getelementptr i8, ptr @buf, i64 10
  %s2 = getelementptr i8, ptr @buf, i64 4
  call void @llvm.memmove.p0.p0.i64(ptr %d2, ptr %s2, i64 %n2, i1 false)
  %n3 = load volatile i64, ptr @len
  call void @llvm.memmove.p0.p0.i64(ptr %s2, ptr %d2, i64 %n3, i1 false)
  store volatile i64 12, ptr @len
  %n4 = load volatile i64, ptr @len
  %d4 = getelementptr i8, ptr @buf, i64 50
  call void @llvm.memset.p0.i64(ptr %d4, i8 -85, i64 %n4, i1 false)
  store volatile i64 0, ptr @len
  %n5 = load volatile i64, ptr @len
  call void @llvm.memset.p0.i64(ptr @buf, i8 -1, i64 %n5, i1 false)
  br label %sum
sum:
  %k = phi i64 [ 0, %go ], [ %k1, %sum ]
  %h = phi i32 [ 0, %go ], [ %h2, %sum ]
  %q = getelementptr i8, ptr @buf, i64 %k
  %v = load i8, ptr %q
  %w = zext i8 %v to i32
  %h1 = mul i32 %h, 31
  %h2 = add i32 %h1, %w
  %k1 = add i64 %k, 1
  %again = icmp ult i64 %k1, 64
  br i1 %again, label %sum, label %out
out:
  ret i32 %h2
}
declare void @llvm.memcpy.p0.p0.i64(ptr, ptr, i64, i1)
declare void @llvm.memmove.p0.p0.i64(ptr, ptr, i64, i1)
declare void @llvm.memset.p0.i64(ptr, i8, i64, i1)
"#;
    let mut want: Vec<u8> = (0..64).collect();
    want.copy_within(0..10, 40);
    want.copy_within(4..24, 10);
    want.copy_within(10..30, 4);
    want[50..62].fill(0xab);
    let hash = want.iter().fold(0u32, |h, &b| h.wrapping_mul(31).wrapping_add(u32::from(b)));

    let t = riscv_without_calls();
    let mut m = epitome_ir::parse_module(ir).unwrap();
    let c = compile(&t, &mut m, &Options::default()).unwrap();
    assert!(!c.asm.contains("jal\tra") && !c.asm.contains("jal ra"), "a call:\n{}", c.asm);
    let mut machine = emu::Machine::link(&[c.object_file(&t).unwrap()]).unwrap();
    assert_eq!(machine.run(1_000_000).unwrap(), hash as i32);
}

/// A call to an "epitome-opaque" function, for another unit, is the
/// launch instruction, its arguments where a call's would be and a
/// relocation naming it; the function is not compiled here, but written
/// out as IR, every other function in it a declaration.
#[test]
fn opaque_functions_are_launched() {
    let ir = r#"
define void @kernel(i32 %a, i32 %b) "epitome-opaque" {
  %x = sitofp i32 %a to float
  ret void
}
define i32 @start(i32 %a) {
  %b = add i32 %a, 1
  call void @kernel(i32 %a, i32 %b)
  %c = add i32 %b, 1
  ret i32 %c
}
"#;
    let t = tiny();
    let mut m = epitome_ir::parse_module(ir).unwrap();
    let c = compile(&t, &mut m, &Options::default()).unwrap();
    assert_eq!(c.funcs.iter().map(|f| f.name.as_str()).collect::<Vec<_>>(), ["start"]);
    let opaque = c.opaque.as_deref().expect("the opaque function's IR");
    assert!(opaque.contains("define void @kernel(i32 %a, i32 %b)"), "{opaque}");
    assert!(opaque.contains("declare i32 @start(i32"), "{opaque}");
    let code = mir::encode_function(&t, &c.funcs[0]).unwrap();
    assert_eq!(code.fixups.iter().map(|x| x.symbol.as_str()).collect::<Vec<_>>(), ["kernel"]);
    let mut launches = Vec::new();
    assert_eq!(run_with(&code.bytes, &[7], &mut launches), 9);
    assert_eq!(launches.len(), 1);
    assert_eq!(launches[0].1[..2], [7, 8]);
    // The object is what the assembler makes of the assembly.
    let funcs = compiled(&t, ir).unwrap();
    assert_eq!(funcs.len(), 1);
}

/// Where a launch cannot be made, `fits` says why: a machine without a
/// launch instruction (and without calls), a launch that would return a
/// value, and one with more arguments than registers.
#[test]
fn launches_that_do_not_fit() {
    let text = std::fs::read_to_string(dir().join("tiny.ll")).unwrap();
    let start = text.find("define void @launch(").unwrap();
    let end = start + text[start..].find("\n}\n").unwrap() + 3;
    let path = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("tiny-without-launch.ll");
    std::fs::write(&path, format!("{}{}", &text[..start], &text[end..])).unwrap();
    let without = Target::load_path(path.to_str().unwrap()).unwrap();
    let ir = r#"
define void @k(i32 %a) "epitome-opaque" {
  ret void
}
define i32 @v(i32 %a) "epitome-opaque" {
  ret i32 %a
}
define void @w(i32 %a, i32 %b, i32 %c, i32 %d, i32 %e) "epitome-opaque" {
  ret void
}
define i32 @one(i32 %a) {
  call void @k(i32 %a)
  ret i32 %a
}
define i32 @value(i32 %a) {
  %r = call i32 @v(i32 %a)
  ret i32 %r
}
define i32 @five(i32 %a) {
  call void @w(i32 %a, i32 %a, i32 %a, i32 %a, i32 %a)
  ret i32 %a
}
"#;
    let errors = |t: &Target| {
        let mut m = epitome_ir::parse_module(ir).unwrap();
        compile(t, &mut m, &Options { keep_going: true, ..Options::default() }).unwrap().errors.join("\n")
    };
    let e = errors(&without);
    assert!(e.contains("@one: does not fit: block %0, call: a call to @k, which is epitome-opaque, and the machine has no calls and no launch instruction"), "{e}");
    let e = errors(&tiny());
    assert!(!e.contains("@one"), "{e}");
    assert!(e.contains("@value: does not fit: block %0, %r = call: a launch of @v that returns a value"), "{e}");
    assert!(
        e.contains(
            "@five: does not fit: block %0, call: a launch of @w with 5 arguments: only 4 are passed in registers"
        ),
        "{e}"
    );
}

/// What preparing a function for selection adds is checked too: here
/// the overflow check of a multiplication, which is made with a 64-bit
/// product, on a machine that lacks i64.
#[test]
fn what_preparing_adds_is_checked() {
    let ir = r#"
define i32 @product(i32 %a, i32 %b) {
  %r = call { i32, i1 } @llvm.umul.with.overflow.i32(i32 %a, i32 %b)
  %v = extractvalue { i32, i1 } %r, 0
  ret i32 %v
}
define i32 @count(i32 %a) {
  %r = call i32 @llvm.ctpop.i32(i32 %a)
  ret i32 %r
}
declare { i32, i1 } @llvm.umul.with.overflow.i32(i32, i32)
declare i32 @llvm.ctpop.i32(i32)
"#;
    let t = tiny();
    let mut m = epitome_ir::parse_module(ir).unwrap();
    let c = compile(&t, &mut m, &Options { keep_going: true, ..Options::default() }).unwrap();
    let errors = c.errors.join("\n");
    assert!(errors.contains("@product: does not fit:") && errors.contains("an i64"), "{errors}");
    assert_eq!(c.funcs.iter().map(|f| f.name.as_str()).collect::<Vec<_>>(), ["count"]);
    let code = mir::encode_function(&t, &c.funcs[0]).unwrap().bytes;
    for x in [0u32, 1, 0xff, 0x8000_0001, u32::MAX] {
        assert_eq!(run(&code, &[x]), x.count_ones(), "ctpop({x:#x})");
    }
}

/// Wide counters and arithmetic whose high bits are never needed are
/// computed in 32 bits, on the toy that lacks i64, and run to the answers
/// 64-bit arithmetic gives.
#[test]
fn wide_integers_are_narrowed() {
    let ir = r#"
; rustc's, for `for _ in 0..n as usize` on a 64-bit target.
define i32 @fib_usize(i32 %n) {
start:
  %iter1 = zext i32 %n to i64
  %_65.not = icmp eq i32 %n, 0
  br i1 %_65.not, label %bb3, label %bb2
bb3:
  %a.sroa.0.0.lcssa = phi i32 [ 0, %start ], [ %b.sroa.0.07, %bb2 ]
  ret i32 %a.sroa.0.0.lcssa
bb2:
  %a.sroa.0.08 = phi i32 [ %b.sroa.0.07, %bb2 ], [ 0, %start ]
  %b.sroa.0.07 = phi i32 [ %t, %bb2 ], [ 1, %start ]
  %iter.sroa.0.06 = phi i64 [ %_7, %bb2 ], [ 0, %start ]
  %_7 = add nuw nsw i64 %iter.sroa.0.06, 1
  %t = add i32 %a.sroa.0.08, %b.sroa.0.07
  %exitcond.not = icmp eq i64 %_7, %iter1
  br i1 %exitcond.not, label %bb3, label %bb2
}

; Down to zero from n, which the way in says is not zero.
define i32 @down(i32 %n) {
start:
  %w = zext i32 %n to i64
  %z = icmp eq i64 %w, 0
  br i1 %z, label %out, label %loop
loop:
  %i = phi i64 [ %w, %start ], [ %i1, %loop ]
  %s = phi i32 [ 0, %start ], [ %s1, %loop ]
  %t = trunc i64 %i to i32
  %s1 = add i32 %s, %t
  %i1 = add nsw i64 %i, -1
  %c = icmp ne i64 %i1, 0
  br i1 %c, label %loop, label %out
out:
  %r = phi i32 [ 0, %start ], [ %s1, %loop ]
  ret i32 %r
}

; Up while below n, with a wide sum of which only the low half is kept.
define i32 @mix(i32 %n, i32 %k) {
start:
  %w = zext i32 %n to i64
  %kk = sext i32 %k to i64
  br label %loop
loop:
  %i = phi i64 [ 0, %start ], [ %i1, %loop ]
  %acc = phi i64 [ 0, %start ], [ %acc1, %loop ]
  %m = mul i64 %i, %kk
  %x = xor i64 %m, %i
  %acc1 = add i64 %acc, %x
  %i1 = add i64 %i, 1
  %c = icmp ult i64 %i1, %w
  br i1 %c, label %loop, label %out
out:
  %r = trunc i64 %acc1 to i32
  ret i32 %r
}

; Compares, shifts and divisions of values that fit.
define i32 @less(i32 %a, i32 %b) {
  %x = sext i32 %a to i64
  %y = sext i32 %b to i64
  %c = icmp slt i64 %x, %y
  %r = zext i1 %c to i32
  ret i32 %r
}
define i32 @parts(i32 %a, i32 %b) {
  %x = zext i32 %a to i64
  %y = zext i32 %b to i64
  %s = lshr i64 %x, 3
  %q = udiv i64 %x, %y
  %t = add i64 %s, %q
  %r = trunc i64 %t to i32
  ret i32 %r
}
"#;
    let t = tiny();
    let funcs = compiled(&t, ir).unwrap();
    let run2 = |name: &str, a: u32, b: u32| run(code_of(&funcs, name), &[a, b]);
    for n in [0u32, 1, 2, 10, 30] {
        assert_eq!(run2("fib_usize", n, 0), fib(n), "fib_usize({n})");
        assert_eq!(run2("down", n, 0), (n * (n + 1) / 2), "down({n})");
    }
    for (n, k) in [(0u32, 5u32), (1, 7), (10, 0xffff_fff3), (200, 12345)] {
        let mut acc = 0u64;
        let mut i = 0u64;
        loop {
            acc = acc.wrapping_add(i.wrapping_mul(k as i32 as i64 as u64) ^ i);
            i += 1;
            if i >= u64::from(n) {
                break;
            }
        }
        assert_eq!(run2("mix", n, k), acc as u32, "mix({n}, {k})");
    }
    for (a, b) in [(1i32, 2i32), (-5, 3), (3, -5), (i32::MIN, i32::MAX), (7, 7)] {
        assert_eq!(run2("less", a as u32, b as u32), u32::from(a < b), "less({a}, {b})");
    }
    for (a, b) in [(100u32, 7u32), (u32::MAX, 3), (5, u32::MAX)] {
        let want = (u64::from(a) >> 3) + u64::from(a) / u64::from(b);
        assert_eq!(run2("parts", a, b), want as u32, "parts({a}, {b})");
    }
}

/// What needs its high bits stays wide, and is reported: the high half of
/// a product, and a counter whose end is not known to fit.
#[test]
fn wide_integers_that_cannot_be_narrowed() {
    let ir = r#"
define i32 @high(i32 %a, i32 %b) {
  %x = zext i32 %a to i64
  %y = zext i32 %b to i64
  %p = mul i64 %x, %y
  %h = lshr i64 %p, 32
  %r = trunc i64 %h to i32
  ret i32 %r
}
define i32 @far(i32 %n) {
start:
  %w = zext i32 %n to i64
  %e = shl i64 %w, 4
  br label %loop
loop:
  %i = phi i64 [ 0, %start ], [ %i1, %loop ]
  %i1 = add i64 %i, 1
  %c = icmp ult i64 %i1, %e
  br i1 %c, label %loop, label %out
out:
  %r = trunc i64 %i1 to i32
  ret i32 %r
}
"#;
    let t = tiny();
    let mut m = epitome_ir::parse_module(ir).unwrap();
    let c = compile(&t, &mut m, &Options { keep_going: true, ..Options::default() }).unwrap();
    let errors = c.errors.join("\n");
    for f in ["@high", "@far"] {
        assert!(errors.contains(&format!("{f}: does not fit:")) && errors.contains("an i64"), "{f}:\n{errors}");
    }
    assert!(c.funcs.is_empty());
}

/// The same from rustc now, where it has the bare-metal RISC-V target.
#[test]
fn rust_usize_loops_from_rustc() {
    let tmp = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("partial-wide");
    std::fs::create_dir_all(&tmp).unwrap();
    let ll = tmp.join("wide.ll");
    let ok = Command::new("rustc")
        .args(["--edition", "2021", "--crate-type=lib", "--target", "riscv64imac-unknown-none-elf"])
        .args(["-C", "opt-level=2", "-C", "panic=abort", "--emit=llvm-ir", "-o"])
        .arg(&ll)
        .arg(dir().join("wide.rs"))
        .output()
        .is_ok_and(|o| o.status.success());
    if !ok {
        eprintln!("skipped: rustc has no riscv64imac-unknown-none-elf");
        return;
    }
    let t = tiny();
    let funcs = compiled(&t, &std::fs::read_to_string(&ll).unwrap()).unwrap();
    for n in [0u32, 1, 9, 24] {
        assert_eq!(run(code_of(&funcs, "fib_usize"), &[n]), fib(n), "fib_usize({n})");
        let want = (1..=n).rev().fold(0u32, |s, i| s.rotate_left(3) ^ i);
        assert_eq!(run(code_of(&funcs, "mix_down"), &[n]), want, "mix_down({n})");
    }
}

/// The machine's own intrinsics: calls to declarations marked "intrinsic"
/// are its instructions (`fits` does not count them as calls), with an
/// immediate taken from a constant argument, and values put in the one
/// register a class of one register holds (a pair, r5 and r6).
#[test]
fn machine_intrinsics_are_instructions() {
    let ir = r#"
define i32 @work(i32 %a, i32 %b) {
  %c = call i32 @tiny.cycles()
  %x = add i32 %a, %c
  call void @tiny.signal(i32 %x, i32 7)
  %y = mul i32 %a, %b
  call void @tiny.put2(i32 %y, i32 %x)
  call void @tiny.put2(i32 %b, i32 %a)
  %r = sub i32 %x, %y
  ret i32 %r
}
declare i32 @tiny.cycles()
declare void @tiny.signal(i32, i32)
declare void @tiny.put2(i32, i32)
"#;
    let t = tiny();
    let funcs = compiled(&t, ir).unwrap();
    let mut events = Vec::new();
    let r = run_with(code_of(&funcs, "work"), &[5, 6], &mut events);
    assert_eq!(r, (5 + 1234) - 30);
    let seen: Vec<[u32; 4]> = events.iter().map(|e| e.1).collect();
    assert_eq!(seen, [[0x516, 1239, 7, 0], [0x92, 30, 1239, 0], [0x92, 6, 5, 0]]);
}
