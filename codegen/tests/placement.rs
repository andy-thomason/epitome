//! Block placement (`layout.rs`, `cold.rs`): cold blocks (panics, cold
//! calls, unlikely branches) are placed after the hot ones, with the hot
//! side of each branch into them falling through. Each test compares a
//! function's instructions and labels with what they should be.

use epitome_codegen::target::Target;
use epitome_codegen::{Options, compile};

/// Function `name`'s instructions and labels, as compiled for the
/// module's triple.
fn function(src: &str, name: &str) -> Vec<String> {
    let t = if src.contains("x86_64") { Target::x86_64() } else { Target::aarch64() };
    let mut m = epitome_ir::parse_module(src).unwrap();
    let asm = compile(&t, &mut m, &Options::default()).unwrap().asm;
    let mut out = Vec::new();
    let mut inside = false;
    for l in asm.lines().map(str::trim) {
        let local = l.starts_with('L') || l.starts_with(".L");
        if l.ends_with(':') && !local {
            inside = l.trim_end_matches(':').trim_start_matches('_') == name;
        } else if inside && !l.is_empty() && (local || !l.starts_with('.')) {
            out.push(l.to_string());
        }
    }
    out
}

fn check(src: &str, name: &str, want: &str) {
    let got = function(src, name).join("\n");
    let want: Vec<&str> = want.lines().map(str::trim).filter(|l| !l.is_empty()).collect();
    assert_eq!(got, want.join("\n"), "@{name}:\n{got}");
}

const BOUNDS: &str = r#"
target triple = "arm64-apple-macosx14.0.0"
declare void @panic_bounds(i64, i64) noreturn
define i64 @sum(ptr %p, i64 %len, i64 %n) {
entry:
  br label %loop
loop:
  %i = phi i64 [ 0, %entry ], [ %i1, %ok ]
  %acc = phi i64 [ 0, %entry ], [ %acc1, %ok ]
  %done = icmp eq i64 %i, %n
  br i1 %done, label %exit, label %check
check:
  %in = icmp ult i64 %i, %len
  br i1 %in, label %ok, label %fail
fail:
  call void @panic_bounds(i64 %i, i64 %len)
  unreachable
ok:
  %a = getelementptr inbounds i64, ptr %p, i64 %i
  %v = load i64, ptr %a
  %acc1 = add i64 %acc, %v
  %i1 = add i64 %i, 1
  br label %loop
exit:
  ret i64 %acc
}
"#;

const BOUNDS_X86: &str = r#"
target triple = "x86_64-unknown-linux-gnu"
declare void @panic_bounds(i64, i64) noreturn
define i64 @sum(ptr %p, i64 %len, i64 %n) {
entry:
  br label %loop
loop:
  %i = phi i64 [ 0, %entry ], [ %i1, %ok ]
  %acc = phi i64 [ 0, %entry ], [ %acc1, %ok ]
  %done = icmp eq i64 %i, %n
  br i1 %done, label %exit, label %check
check:
  %in = icmp ult i64 %i, %len
  br i1 %in, label %ok, label %fail
fail:
  call void @panic_bounds(i64 %i, i64 %len)
  unreachable
ok:
  %a = getelementptr inbounds i64, ptr %p, i64 %i
  %v = load i64, ptr %a
  %acc1 = add i64 %acc, %v
  %i1 = add i64 %i, 1
  br label %loop
exit:
  ret i64 %acc
}
"#;

const COLD: &str = r#"
target triple = "arm64-apple-macosx14.0.0"
declare void @grow(ptr) cold
declare void @log(i32)
declare i1 @llvm.expect.i1(i1, i1)
define void @push(ptr %v, i64 %len, i64 %cap) {
entry:
  %full = icmp eq i64 %len, %cap
  br i1 %full, label %slow, label %fast
slow:
  call void @grow(ptr %v)
  br label %fast
fast:
  store i64 %len, ptr %v
  ret void
}
define void @site(ptr %v, i64 %len, i64 %cap) {
entry:
  %full = icmp eq i64 %len, %cap
  br i1 %full, label %slow, label %fast
slow:
  call void @log(i32 1) cold
  br label %fast
fast:
  store i64 %len, ptr %v
  ret void
}
define i32 @expect(i32 %x) {
entry:
  %c = icmp sgt i32 %x, 100
  %e = call i1 @llvm.expect.i1(i1 %c, i1 false)
  br i1 %e, label %rare, label %common
rare:
  call void @log(i32 %x)
  ret i32 1
common:
  ret i32 0
}
define i32 @weights(i32 %x) {
entry:
  %c = icmp sgt i32 %x, 100
  br i1 %c, label %common, label %rare, !prof !0
rare:
  call void @log(i32 %x)
  ret i32 1
common:
  ret i32 0
}
define i32 @early(i32 %x) {
entry:
  %c = icmp eq i32 %x, 0
  br i1 %c, label %zero, label %more
zero:
  ret i32 7
more:
  call void @log(i32 %x)
  br label %more2
more2:
  call void @log(i32 2)
  ret i32 3
}
define void @trap(i1 %c) {
entry:
  br i1 %c, label %bad, label %good
bad:
  call void @llvm.trap()
  unreachable
good:
  ret void
}
declare void @llvm.trap()
!0 = !{!"branch_weights", i32 2000, i32 1}
"#;

/// A bounds-checked loop: the panic goes after the return, and the loop
/// falls through its checks.
#[test]
fn failed_bounds_checks_go_last() {
    check(
        BOUNDS,
        "sum",
        "
        sub sp, sp, #16
        stp x29, x30, [sp, #0]
        add x29, sp, #0
        mov x3, xzr
        mov x4, xzr
        LBB0_1:
        cmp x3, x2
        b.eq LBB0_4
        cmp x3, x1
        b.hs LBB0_5
        ldr x5, [x0, x3, lsl #3]
        add x5, x4, x5
        add x6, x3, #1
        mov x3, x6
        mov x4, x5
        b LBB0_1
        LBB0_4:
        mov x0, x4
        ldp x29, x30, [sp, #0]
        add sp, sp, #16
        ret
        LBB0_5:
        mov x0, x3
        bl _panic_bounds
        ",
    );
}

/// The same on x86-64.
#[test]
fn failed_bounds_checks_go_last_on_x86() {
    check(
        BOUNDS_X86,
        "sum",
        "
        pushq %rax
        xorl %eax, %eax
        xorl %ecx, %ecx
        .LBB0_1:
        cmpq %rdx, %rax
        je .LBB0_4
        cmpq %rsi, %rax
        jae .LBB0_5
        leaq (%rdi,%rax,8), %r8
        movq %rcx, %r9
        addq 0(%r8), %r9
        leaq 1(%rax), %r8
        movq %r8, %rax
        movq %r9, %rcx
        jmp .LBB0_1
        .LBB0_4:
        movq %rcx, %rax
        popq %rcx
        retq
        .LBB0_5:
        movq %rax, %rdi
        callq panic_bounds
        ",
    );
}

/// A call of a `cold` function: its block goes after the return.
#[test]
fn push() {
    check(
        COLD,
        "push",
        "
        sub sp, sp, #32
        stp x29, x30, [sp, #16]
        stp x19, x20, [sp, #0]
        add x29, sp, #16
        mov x19, x0
        mov x20, x1
        cmp x20, x2
        b.eq LBB0_2
        LBB0_1:
        str x20, [x19, #0]
        ldp x29, x30, [sp, #16]
        ldp x19, x20, [sp, #0]
        add sp, sp, #32
        ret
        LBB0_2:
        mov x0, x19
        bl _grow
        b LBB0_1
        ",
    );
}

/// A call marked `cold` at the call site.
#[test]
fn site() {
    check(
        COLD,
        "site",
        "
        sub sp, sp, #32
        stp x29, x30, [sp, #16]
        stp x19, x20, [sp, #0]
        add x29, sp, #16
        mov x19, x0
        mov x20, x1
        cmp x20, x2
        b.eq LBB1_2
        LBB1_1:
        str x20, [x19, #0]
        ldp x29, x30, [sp, #16]
        ldp x19, x20, [sp, #0]
        add sp, sp, #32
        ret
        LBB1_2:
        mov w0, #1
        bl _log
        b LBB1_1
        ",
    );
}

/// `llvm.expect` of false: the branch it tests is unlikely.
#[test]
fn expect() {
    check(
        COLD,
        "expect",
        "
        sub sp, sp, #16
        stp x29, x30, [sp, #0]
        add x29, sp, #0
        cmp w0, #100
        b.gt LBB2_3
        mov x0, xzr
        LBB2_2:
        ldp x29, x30, [sp, #0]
        add sp, sp, #16
        ret
        LBB2_3:
        bl _log
        mov w0, #1
        b LBB2_2
        ",
    );
}

/// Branch weights of 2000 to 1: the side with 1 goes last.
#[test]
fn weights() {
    check(
        COLD,
        "weights",
        "
        sub sp, sp, #16
        stp x29, x30, [sp, #0]
        add x29, sp, #0
        cmp w0, #100
        b.le LBB3_3
        mov x0, xzr
        LBB3_2:
        ldp x29, x30, [sp, #0]
        add sp, sp, #16
        ret
        LBB3_3:
        bl _log
        mov w0, #1
        b LBB3_2
        ",
    );
}

/// An early return is hot like the rest: it falls through from the
/// test, not put off to the end as returns once were.
#[test]
fn early() {
    check(
        COLD,
        "early",
        "
        sub sp, sp, #16
        stp x29, x30, [sp, #0]
        add x29, sp, #0
        cbnz w0, LBB4_3
        mov w0, #7
        LBB4_2:
        ldp x29, x30, [sp, #0]
        add sp, sp, #16
        ret
        LBB4_3:
        bl _log
        mov w0, #2
        bl _log
        mov w0, #3
        b LBB4_2
        ",
    );
}

/// `llvm.trap` and the `unreachable` after it: one `brk`.
#[test]
fn trap() {
    check(
        COLD,
        "trap",
        "
        tbnz w0, #0, LBB5_2
        ret
        LBB5_2:
        brk #0x1
        ",
    );
}
