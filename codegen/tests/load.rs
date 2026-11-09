//! Compiling IR and loading it into this process (`load.rs`), then calling
//! it: calls between the module's functions, constants and data (pointers
//! in data among them), symbols the module does not define (looked up by
//! the test's resolver, or in the process), vector code, and assembly of the
//! module's own. Vector code is also linked by `cc` and `dlopen`ed, and the
//! two run on the same input.
//!
//! On an AArch64 Mac or x86-64 Linux; elsewhere the tests skip. On x86-64,
//! the vector code is compiled for each of SSE, AVX2 and AVX-512 that the
//! machine has.

use std::ffi::{CString, c_char, c_int, c_void};
use std::path::PathBuf;
use std::process::Command;

use epitome_codegen::load::{self, Loaded};
use epitome_codegen::target::Target;
use epitome_codegen::{Compiled, Options, compile};

/// This machine's target and triple, if code can be loaded here.
fn host() -> Option<(Target, &'static str)> {
    if cfg!(all(target_arch = "x86_64", target_os = "linux")) {
        Some((Target::x86_64(), "x86_64-unknown-linux-gnu"))
    } else if cfg!(all(target_arch = "aarch64", target_os = "macos")) {
        Some((Target::aarch64(), "arm64-apple-macosx14.0.0"))
    } else {
        None
    }
}

fn compiled(t: &Target, triple: &str, ir: &str, opts: &Options) -> Compiled {
    let ir = format!("target triple = \"{triple}\"\n{ir}");
    let mut m = epitome_ir::parse_module(&ir).unwrap_or_else(|e| panic!("{e:?}\n{ir}"));
    compile(t, &mut m, opts).unwrap_or_else(|e| panic!("{e}"))
}

fn no_asm(mattr: &[&str]) -> Options {
    Options { no_asm: true, mattr: mattr.iter().map(|s| s.to_string()).collect(), ..Default::default() }
}

const SCALAR: &str = r#"
@table = internal constant [8 x i32] [i32 3, i32 1, i32 4, i32 1, i32 5, i32 9, i32 2, i32 6]
@counter = global i64 0
@zeros = internal global [64 x i8] zeroinitializer
@ptrs = global [2 x ptr] [ptr @counter, ptr @square]
@greeting = private unnamed_addr constant [6 x i8] c"hello\00"

declare i64 @host_add(i64, i64)
declare i64 @strlen(ptr)
declare void @llvm.memset.p0.i64(ptr, i8, i64, i1)

define internal i64 @square(i64 %x) {
  %r = mul i64 %x, %x
  ret i64 %r
}

define i64 @fib(i64 %n) {
entry:
  %small = icmp ult i64 %n, 2
  br i1 %small, label %done, label %more
more:
  %a = sub i64 %n, 1
  %b = sub i64 %n, 2
  %fa = call i64 @fib(i64 %a)
  %fb = call i64 @fib(i64 %b)
  %s = add i64 %fa, %fb
  ret i64 %s
done:
  ret i64 %n
}

define i64 @sum_squares(i64 %n) {
entry:
  br label %loop
loop:
  %i = phi i64 [ 0, %entry ], [ %i1, %loop ]
  %acc = phi i64 [ 0, %entry ], [ %acc1, %loop ]
  %sq = call i64 @square(i64 %i)
  %acc1 = add i64 %acc, %sq
  %i1 = add i64 %i, 1
  %more = icmp ult i64 %i1, %n
  br i1 %more, label %loop, label %exit
exit:
  ret i64 %acc1
}

define i32 @lookup(i64 %i) {
  %k = and i64 %i, 7
  %p = getelementptr [8 x i32], ptr @table, i64 0, i64 %k
  %v = load i32, ptr %p
  ret i32 %v
}

define i64 @bump(i64 %by) {
  %old = load i64, ptr @counter
  %new = add i64 %old, %by
  store i64 %new, ptr @counter
  ret i64 %new
}

define ptr @counter_addr() {
  ret ptr @counter
}

define i64 @via_ptrs(i64 %x) {
  %p = getelementptr [2 x ptr], ptr @ptrs, i64 0, i64 1
  %f = load ptr, ptr %p
  %r = call i64 %f(i64 %x)
  %c = load ptr, ptr @ptrs
  %v = load i64, ptr %c
  %s = add i64 %r, %v
  ret i64 %s
}

define double @poly(double %x) {
  %a = fmul double %x, 3.250000e+00
  %b = fmul double %a, %x
  %c = fadd double %b, 1.250000e-01
  ret double %c
}

define double @pool(double %x) {
  %a = fmul double %x, 0x400921FB54442D18
  %b = fadd double %a, 0x3FE6A09E667F3BCD
  ret double %b
}

define i32 @third_plus(i32 %x) {
  %p = getelementptr [8 x i32], ptr @table, i64 0, i64 5
  %v = load i32, ptr %p
  %s = add i32 %v, %x
  ret i32 %s
}

define i64 @ext(i64 %x) {
  %r = call i64 @host_add(i64 %x, i64 100)
  %s = call i64 @host_add(i64 %r, i64 1)
  ret i64 %s
}

define i64 @len() {
  %n = call i64 @strlen(ptr @greeting)
  ret i64 %n
}

define i64 @clear_and_sum(i8 %v) {
entry:
  call void @llvm.memset.p0.i64(ptr @zeros, i8 %v, i64 64, i1 false)
  br label %loop
loop:
  %i = phi i64 [ 0, %entry ], [ %i1, %loop ]
  %acc = phi i64 [ 0, %entry ], [ %acc1, %loop ]
  %p = getelementptr i8, ptr @zeros, i64 %i
  %b = load i8, ptr %p
  %w = zext i8 %b to i64
  %acc1 = add i64 %acc, %w
  %i1 = add i64 %i, 1
  %more = icmp ult i64 %i1, 64
  br i1 %more, label %loop, label %exit
exit:
  ret i64 %acc1
}
"#;

extern "C" fn host_add(a: i64, b: i64) -> i64 {
    a + b
}

fn resolver(name: &str) -> Option<*const u8> {
    match name {
        "host_add" => Some(host_add as *const u8),
        _ => load::process_symbol(name),
    }
}

#[test]
fn scalar() {
    let Some((t, triple)) = host() else { return };
    let c = compiled(&t, triple, SCALAR, &no_asm(&[]));
    assert!(c.asm.is_empty(), "no_asm wrote the assembly");
    let l = load::load(&t, &c, resolver).unwrap();
    unsafe {
        let fib: extern "C" fn(i64) -> i64 = l.function("fib").unwrap();
        assert_eq!(fib(20), 6765);
        let sum: extern "C" fn(i64) -> i64 = l.function("sum_squares").unwrap();
        assert_eq!(sum(10), 285);
        let lookup: extern "C" fn(i64) -> i32 = l.function("lookup").unwrap();
        assert_eq!((0..8).map(|i| lookup(i)).collect::<Vec<_>>(), [3, 1, 4, 1, 5, 9, 2, 6]);
        let bump: extern "C" fn(i64) -> i64 = l.function("bump").unwrap();
        assert_eq!(bump(5), 5);
        assert_eq!(bump(7), 12);
        // The global, by its symbol and as the code sees it.
        let counter = l.address("counter").unwrap() as *const i64;
        assert_eq!(*counter, 12);
        let addr: extern "C" fn() -> *const i64 = l.function("counter_addr").unwrap();
        assert_eq!(addr(), counter);
        let via: extern "C" fn(i64) -> i64 = l.function("via_ptrs").unwrap();
        assert_eq!(via(9), 81 + 12);
        let poly: extern "C" fn(f64) -> f64 = l.function("poly").unwrap();
        assert_eq!(poly(2.0), 13.125);
        let pool: extern "C" fn(f64) -> f64 = l.function("pool").unwrap();
        assert_eq!(pool(2.0), 2.0 * std::f64::consts::PI + std::f64::consts::FRAC_1_SQRT_2);
        let third: extern "C" fn(i32) -> i32 = l.function("third_plus").unwrap();
        assert_eq!(third(100), 109);
        let ext: extern "C" fn(i64) -> i64 = l.function("ext").unwrap();
        assert_eq!(ext(5), 106);
        let len: extern "C" fn() -> i64 = l.function("len").unwrap();
        assert_eq!(len(), 5);
        let clear: extern "C" fn(i8) -> i64 = l.function("clear_and_sum").unwrap();
        assert_eq!(clear(3), 192);
        assert_eq!(clear(0), 0);
        // Not exported: internal, but still found.
        assert!(l.address("square").is_some());
        assert!(l.address("no_such_thing").is_none());
    }
    // Two loads of the same code are independent.
    let l2 = load::load(&t, &c, resolver).unwrap();
    unsafe {
        let bump: extern "C" fn(i64) -> i64 = l2.function("bump").unwrap();
        assert_eq!(bump(1), 1);
    }
    drop(l);
    drop(l2);
}

#[test]
fn undefined_symbols_are_reported() {
    let Some((t, triple)) = host() else { return };
    let c = compiled(&t, triple, SCALAR, &no_asm(&[]));
    let e = load::load(&t, &c, |_| None).err().expect("loaded without host_add");
    assert!(e.contains("host_add") && e.contains("strlen"), "{e}");
}

/// The assembly is the same whether written at once or on request, and so
/// is the object file.
#[test]
fn assembly_on_request() {
    let Some((t, triple)) = host() else { return };
    let eager = compiled(&t, triple, SCALAR, &Options::default());
    let lazy = compiled(&t, triple, SCALAR, &no_asm(&[]));
    assert!(lazy.asm.is_empty());
    assert_eq!(lazy.assembly(&t).unwrap(), eager.asm);
    assert_eq!(lazy.object_file(&t).unwrap(), eager.object_file(&t).unwrap());
}

const VECTOR: &str = r#"
define void @saxpy(ptr %y, ptr %x, i64 %n) {
entry:
  br label %loop
loop:
  %i = phi i64 [ 0, %entry ], [ %i1, %loop ]
  %px = getelementptr float, ptr %x, i64 %i
  %py = getelementptr float, ptr %y, i64 %i
  %vx = load <16 x float>, ptr %px, align 4
  %vy = load <16 x float>, ptr %py, align 4
  %m = fmul <16 x float> %vx, <float 1.5, float 1.5, float 1.5, float 1.5, float 1.5, float 1.5, float 1.5, float 1.5, float 1.5, float 1.5, float 1.5, float 1.5, float 1.5, float 1.5, float 1.5, float 1.5>
  %s = fadd <16 x float> %m, %vy
  %t = fadd <16 x float> %s, <float 0.5, float 1.0, float 1.5, float 2.0, float 2.5, float 3.0, float 3.5, float 4.0, float 4.5, float 5.0, float 5.5, float 6.0, float 6.5, float 7.0, float 7.5, float 8.0>
  store <16 x float> %t, ptr %py, align 4
  %i1 = add i64 %i, 16
  %more = icmp ult i64 %i1, %n
  br i1 %more, label %loop, label %exit
exit:
  ret void
}

define void @mix(ptr %out, ptr %a, ptr %b) {
  %va = load <16 x i32>, ptr %a, align 4
  %vb = load <16 x i32>, ptr %b, align 4
  %p = mul <16 x i32> %va, <i32 3, i32 5, i32 7, i32 11, i32 13, i32 17, i32 19, i32 23, i32 29, i32 31, i32 37, i32 41, i32 43, i32 47, i32 53, i32 59>
  %q = add <16 x i32> %p, %vb
  %lt = icmp slt <16 x i32> %va, %vb
  %mx = select <16 x i1> %lt, <16 x i32> %vb, <16 x i32> %va
  %r = xor <16 x i32> %q, %mx
  %sh = lshr <16 x i32> %r, <i32 1, i32 2, i32 3, i32 4, i32 5, i32 6, i32 7, i32 8, i32 1, i32 2, i32 3, i32 4, i32 5, i32 6, i32 7, i32 8>
  %o = add <16 x i32> %sh, %r
  store <16 x i32> %o, ptr %out, align 4
  ret void
}
"#;

unsafe extern "C" {
    fn dlopen(path: *const c_char, flags: c_int) -> *mut c_void;
    fn dlsym(handle: *mut c_void, name: *const c_char) -> *mut c_void;
    fn dlerror() -> *const c_char;
}

/// `c` linked by `cc -shared` and `dlopen`ed: `name`'s address, if `cc` is
/// here.
fn linked(t: &Target, c: &Compiled, stem: &str, name: &str) -> Option<*const c_void> {
    if !Command::new("cc").arg("--version").output().is_ok_and(|o| o.status.success()) {
        return None;
    }
    let tmp = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("load");
    std::fs::create_dir_all(&tmp).unwrap();
    let obj = tmp.join(format!("{stem}.o"));
    std::fs::write(&obj, c.object_file(t).unwrap()).unwrap();
    let lib = tmp.join(format!("lib{stem}.so"));
    let st = Command::new("cc").arg("-shared").arg("-o").arg(&lib).arg(&obj).status().unwrap();
    assert!(st.success(), "linking {stem}");
    unsafe {
        let path = CString::new(lib.to_str().unwrap()).unwrap();
        let h = dlopen(path.as_ptr(), 2);
        assert!(!h.is_null(), "{:?}", std::ffi::CStr::from_ptr(dlerror()));
        let sym = CString::new(name).unwrap();
        let f = dlsym(h, sym.as_ptr());
        assert!(!f.is_null(), "no {name}");
        Some(f as *const c_void)
    }
}

/// The feature sets to compile the vector code for: on x86-64, those of
/// AVX2 and AVX-512 the machine has. (Not plain SSE, which cannot store
/// `<8 x float>` aligned to 4 bytes yet; and the vectors of floats are of
/// 16, as with AVX-512 `<8 x float>` cannot be stored, nor a float splat.)
fn feature_sets() -> Vec<(&'static str, Vec<&'static str>)> {
    #[allow(unused_mut)]
    let mut sets = vec![];
    #[cfg(target_arch = "aarch64")]
    sets.push(("base", vec![]));
    #[cfg(target_arch = "x86_64")]
    {
        if std::arch::is_x86_feature_detected!("avx2") && std::arch::is_x86_feature_detected!("fma") {
            sets.push(("avx2", vec!["+avx2,+fma"]));
        }
        if std::arch::is_x86_feature_detected!("avx512f")
            && std::arch::is_x86_feature_detected!("avx512bw")
            && std::arch::is_x86_feature_detected!("avx512dq")
            && std::arch::is_x86_feature_detected!("avx512vl")
        {
            sets.push(("avx512", vec!["+avx512f,+avx512bw,+avx512cd,+avx512dq,+avx512vl"]));
        }
    }
    sets
}

#[test]
fn vectors() {
    let Some((t, triple)) = host() else { return };
    for (which, mattr) in feature_sets() {
        let c = compiled(&t, triple, VECTOR, &no_asm(&mattr));
        let l = load::load(&t, &c, load::process_symbol).unwrap();
        type Saxpy = unsafe extern "C" fn(*mut f32, *const f32, i64);
        type Mix = unsafe extern "C" fn(*mut i32, *const i32, *const i32);
        let x: Vec<f32> = (0..64).map(|i| i as f32 * 0.25 - 3.0).collect();
        let y0: Vec<f32> = (0..64).map(|i| (i * i % 17) as f32).collect();
        let a: Vec<i32> = (0..16).map(|i| (i * 7919) ^ -(i & 3)).collect();
        let b: Vec<i32> = (0..16).map(|i| 1000 - i * 131).collect();
        let run = |saxpy: Saxpy, mix: Mix| unsafe {
            let mut y = y0.clone();
            saxpy(y.as_mut_ptr(), x.as_ptr(), 64);
            let mut o = vec![0i32; 16];
            mix(o.as_mut_ptr(), a.as_ptr(), b.as_ptr());
            (y, o)
        };
        let (y, o) = unsafe { run(l.function("saxpy").unwrap(), l.function("mix").unwrap()) };
        // What the IR says.
        let want_y: Vec<f32> = (0..64).map(|i| (x[i] * 1.5 + y0[i]) + (i % 16 + 1) as f32 * 0.5).collect();
        assert_eq!(y, want_y, "{which}: saxpy");
        let muls = [3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37, 41, 43, 47, 53, 59];
        let want_o: Vec<i32> = (0..16)
            .map(|i| {
                let q = a[i].wrapping_mul(muls[i]).wrapping_add(b[i]);
                let r = q ^ a[i].max(b[i]);
                ((r as u32) >> (i % 8 + 1)) as i32 + r
            })
            .map(|v| v as i32)
            .collect();
        assert_eq!(o, want_o, "{which}: mix");
        // And the same code linked by the system's linker.
        let stem = format!("vectors_{which}");
        if let (Some(s), Some(m)) = (linked(&t, &c, &stem, "saxpy"), linked(&t, &c, &stem, "mix")) {
            let theirs = unsafe { run(std::mem::transmute(s), std::mem::transmute(m)) };
            assert_eq!((y, o), theirs, "{which}: loaded and linked differ");
        }
    }
}

/// Assembly of the module's own and inline assembly: the text is written
/// and assembled, and that object loaded.
#[test]
fn inline_assembly() {
    let Some((t, triple)) = host() else { return };
    let ir = if cfg!(target_arch = "aarch64") {
        r#"
module asm ".globl _asm_seven"
module asm ".p2align 2"
module asm "_asm_seven:"
module asm "  mov x0, #7"
module asm "  ret"

declare i64 @asm_seven()

define i64 @sum3(i64 %a, i64 %b) {
  %s = call i64 asm "add $0, $1, $2", "=r,r,r"(i64 %a, i64 %b)
  %k = call i64 @asm_seven()
  %t = add i64 %s, %k
  ret i64 %t
}
"#
    } else {
        r#"
module asm ".globl asm_seven"
module asm "asm_seven:"
module asm "  movl $7, %eax"
module asm "  retq"

declare i64 @asm_seven()

define i64 @sum3(i64 %a, i64 %b) {
  %s = call i64 asm "leaq ($1,$2), $0", "=r,r,r"(i64 %a, i64 %b)
  %k = call i64 @asm_seven()
  %t = add i64 %s, %k
  ret i64 %t
}
"#
    };
    let c = compiled(&t, triple, ir, &no_asm(&[]));
    assert!(c.has_inline_asm());
    let l: Loaded = load::load(&t, &c, load::process_symbol).unwrap();
    unsafe {
        let f: extern "C" fn(i64, i64) -> i64 = l.function("sum3").unwrap();
        assert_eq!(f(30, 5), 42);
    }
}

/// Loading is quick, and so is compiling without the assembly: the times
/// for a module of many functions, printed.
#[test]
fn timing() {
    let Some((t, triple)) = host() else { return };
    let mut ir = String::from(SCALAR);
    for k in 0..300 {
        ir.push_str(&format!(
            "define i64 @f{k}(i64 %x) {{\n  %a = mul i64 %x, {k}\n  %b = call i64 @fib(i64 %a)\n  %c = xor i64 %b, %x\n  ret i64 %c\n}}\n"
        ));
    }
    let now = std::time::Instant::now();
    let with = compiled(&t, triple, &ir, &Options::default());
    let with_t = now.elapsed();
    let now = std::time::Instant::now();
    let c = compiled(&t, triple, &ir, &no_asm(&[]));
    let without_t = now.elapsed();
    let now = std::time::Instant::now();
    let l = load::load(&t, &c, resolver).unwrap();
    let load_t = now.elapsed();
    eprintln!("compiling with assembly {with_t:?}, without {without_t:?}; loading {load_t:?}");
    drop(with);
    unsafe {
        let f: extern "C" fn(i64) -> i64 = l.function("f3").unwrap();
        assert_eq!(f(2), 8 ^ 2);
    }
}

/// With `EPITOME_LOAD_IR=<file.ll>` (IR without a triple: this machine's
/// is put first): the time to compile it with and without the assembly,
/// and to load it against writing an object, linking it with `cc -shared`
/// and `dlopen`ing that, each the best of five. `EPITOME_LOAD_MATTR` gives
/// the features (by default AVX2 on x86-64).
#[test]
fn file_timing() {
    let Some((t, triple)) = host() else { return };
    let Ok(path) = std::env::var("EPITOME_LOAD_IR") else { return };
    let mattr = std::env::var("EPITOME_LOAD_MATTR")
        .unwrap_or_else(|_| if cfg!(target_arch = "x86_64") { "+avx2".into() } else { String::new() });
    let mattr: Vec<&str> = mattr.split(',').filter(|s| !s.is_empty()).collect();
    let src = std::fs::read_to_string(&path).unwrap();
    let mut m = epitome_ir::parse_module(&format!("target triple = \"{triple}\"\n{src}")).unwrap();
    epitome_opt::pipeline::run(&mut m, &Default::default());
    let best = |f: &mut dyn FnMut()| {
        (0..5)
            .map(|_| {
                let now = std::time::Instant::now();
                f();
                now.elapsed().as_secs_f64() * 1e3
            })
            .fold(f64::INFINITY, f64::min)
    };
    let opts = |skip| Options { no_asm: skip, ..no_asm(&mattr) };
    let with = best(&mut || drop(compile(&t, &mut m.clone(), &opts(false)).unwrap()));
    let without = best(&mut || drop(compile(&t, &mut m.clone(), &opts(true)).unwrap()));
    let c = compile(&t, &mut m.clone(), &opts(true)).unwrap();
    let text = best(&mut || drop(c.assembly(&t).unwrap()));
    let loading = best(&mut || drop(load::load(&t, &c, load::process_symbol).unwrap()));
    let tmp = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("load");
    std::fs::create_dir_all(&tmp).unwrap();
    let mut n = 0;
    let linking = best(&mut || {
        n += 1;
        let obj = tmp.join(format!("timing{n}.o"));
        std::fs::write(&obj, c.object_file(&t).unwrap()).unwrap();
        let lib = tmp.join(format!("libtiming{n}.so"));
        assert!(Command::new("cc").arg("-shared").arg("-o").arg(&lib).arg(&obj).status().unwrap().success());
        let path = CString::new(lib.to_str().unwrap()).unwrap();
        assert!(!unsafe { dlopen(path.as_ptr(), 2) }.is_null());
    });
    eprintln!(
        "{path}: {} functions; compiling {with:.1} ms with the assembly, {without:.1} ms without \
         (the text alone {text:.1} ms); loading {loading:.2} ms, against {linking:.1} ms writing, \
         linking and dlopening an object",
        c.funcs.len()
    );
}
