//! The C functions that `design.sv` imports, written in Rust against the C
//! layer of IEEE 1800-2023 Annex H. The `dpi` test builds this as a shared
//! library and loads it into a simulation of the design. It uses the
//! `svdpi.h` functions, and the design's exports, which it finds in the
//! process at run time.

#![allow(non_snake_case, clippy::missing_safety_doc)]

use std::ffi::{CStr, c_char, c_void};
use std::ptr::null_mut;

type Scope = *mut c_void;
type Handle = *const c_void;

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct LogicVecVal {
    aval: u32,
    bval: u32,
}

#[repr(C)]
pub struct TimeVal {
    ty: i32,
    high: u32,
    low: u32,
    real: f64,
}

/// `rec_t` as C lays it out (H.7.8).
#[repr(C)]
pub struct Rec {
    count: i32,
    scale: f64,
    tag: i8,
    wide: [u32; 2],
    label: *const c_char,
}

#[cfg_attr(target_os = "linux", link(name = "dl"))]
unsafe extern "C" {
    fn svDpiVersion() -> *const c_char;
    fn svGetBitselBit(s: *const u32, i: i32) -> u8;
    fn svGetBitselLogic(s: *const LogicVecVal, i: i32) -> u8;
    fn svPutBitselLogic(d: *mut LogicVecVal, i: i32, s: u8);
    fn svGetPartselBit(d: *mut u32, s: *const u32, i: i32, w: i32);
    fn svGetPartselLogic(d: *mut LogicVecVal, s: *const LogicVecVal, i: i32, w: i32);
    fn svPutPartselLogic(d: *mut LogicVecVal, s: LogicVecVal, i: i32, w: i32);
    fn svLeft(h: Handle, d: i32) -> i32;
    fn svRight(h: Handle, d: i32) -> i32;
    fn svLow(h: Handle, d: i32) -> i32;
    fn svHigh(h: Handle, d: i32) -> i32;
    fn svIncrement(h: Handle, d: i32) -> i32;
    fn svSize(h: Handle, d: i32) -> i32;
    fn svDimensions(h: Handle) -> i32;
    fn svGetArrayPtr(h: Handle) -> *mut c_void;
    fn svSizeOfArray(h: Handle) -> i32;
    fn svGetArrElemPtr(h: Handle, i1: i32, ...) -> *mut c_void;
    fn svGetArrElemPtr1(h: Handle, i1: i32) -> *mut c_void;
    fn svGetArrElemPtr2(h: Handle, i1: i32, i2: i32) -> *mut c_void;
    fn svGetBitArrElemVecVal(d: *mut u32, s: Handle, i1: i32, ...);
    fn svGetBitArrElem2VecVal(d: *mut u32, s: Handle, i1: i32, i2: i32);
    fn svPutLogicArrElem1VecVal(d: Handle, s: *const LogicVecVal, i1: i32);
    fn svGetLogicArrElem1(s: Handle, i1: i32) -> u8;
    fn svPutLogicArrElem(d: Handle, v: u8, i1: i32, ...);
    fn svGetScope() -> Scope;
    fn svSetScope(s: Scope) -> Scope;
    fn svGetNameFromScope(s: Scope) -> *const c_char;
    fn svGetScopeFromName(name: *const c_char) -> Scope;
    fn svPutUserData(s: Scope, key: *mut c_void, data: *mut c_void) -> i32;
    fn svGetUserData(s: Scope, key: *mut c_void) -> *mut c_void;
    fn svIsDisabledState() -> i32;
    fn svAckDisabledState();

    fn dlsym(handle: *mut c_void, name: *const c_char) -> *mut c_void;

    // The design's exports.
    fn cell_id(k: i32, id: *mut i32) -> i32;
    fn cell_wait(n: i32, at: *mut i64) -> i32;
    fn descend(depth: i32) -> i32;
    fn long_wait() -> i32;
    fn trace(what: *const c_char, v: i32);
}

fn text(p: *const c_char) -> String {
    if p.is_null() { "(null)".into() } else { unsafe { CStr::from_ptr(p) }.to_string_lossy().into_owned() }
}

// ---- small values ----

#[unsafe(no_mangle)]
pub extern "C" fn neg_byte(x: i8) -> i8 {
    x.wrapping_neg()
}

#[unsafe(no_mangle)]
pub extern "C" fn neg_short(x: i16) -> i16 {
    -x
}

#[unsafe(no_mangle)]
pub extern "C" fn add_int(a: i32, b: i32) -> i32 {
    a + b
}

#[unsafe(no_mangle)]
pub extern "C" fn mul_long(a: i64, b: i64) -> i64 {
    a * b
}

#[unsafe(no_mangle)]
pub extern "C" fn top_bit(x: u32) -> u32 {
    x >> 31
}

#[unsafe(no_mangle)]
pub extern "C" fn inc_ubyte(x: u8) -> u8 {
    x.wrapping_add(1)
}

#[unsafe(no_mangle)]
pub extern "C" fn half(x: f64) -> f64 {
    x / 2.0
}

#[unsafe(no_mangle)]
pub extern "C" fn scale_short(x: f32, k: f64) -> f32 {
    x * k as f32
}

#[unsafe(no_mangle)]
pub extern "C" fn not_bit(b: u8) -> u8 {
    (b ^ 1) & 1
}

/// `svLogic` codes go through unchanged: 0, 1, z (2) and x (3).
#[unsafe(no_mangle)]
pub extern "C" fn pass_logic(l: u8) -> u8 {
    l
}

#[unsafe(no_mangle)]
pub extern "C" fn greet(who: *const c_char) -> *const c_char {
    // A result string stays the C side's (H.8.10).
    static mut BUF: [u8; 64] = [0; 64];
    let s = format!("hello, {}\0", text(who));
    unsafe {
        let buf = &mut *std::ptr::addr_of_mut!(BUF);
        buf[..s.len()].copy_from_slice(s.as_bytes());
        buf.as_ptr() as *const c_char
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn make_counter(start: i32) -> *mut c_void {
    Box::into_raw(Box::new(start)) as *mut c_void
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bump_counter(c: *mut c_void) -> i32 {
    let c = unsafe { &mut *(c as *mut i32) };
    *c += 1;
    *c
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn free_counter(c: *mut c_void) {
    drop(unsafe { Box::from_raw(c as *mut i32) });
}

#[unsafe(no_mangle)]
pub extern "C" fn color_next(c: i32) -> i32 {
    c + 5
}

/// Twenty-two arguments: some of each kind go on the stack.
#[unsafe(no_mangle)]
#[allow(clippy::too_many_arguments)]
pub extern "C" fn weigh(
    a0: i32, a1: i32, a2: i32, a3: i32, a4: i32, a5: i32, a6: i32, a7: i32, a8: i32, a9: i32,
    r0: f64, r1: f64, r2: f64, r3: f64, r4: f64, r5: f64, r6: f64, r7: f64, r8: f64, r9: f64,
    b: i8, s: f32,
) -> f64 {
    let ints = [a0, a1, a2, a3, a4, a5, a6, a7, a8, a9];
    let reals = [r0, r1, r2, r3, r4, r5, r6, r7, r8, r9];
    ints.iter().zip(reals).map(|(&a, r)| a as f64 * r).sum::<f64>() + b as f64 * s as f64
}

#[unsafe(no_mangle)]
pub extern "C" fn pkg_twice(x: i32) -> i32 {
    2 * x
}

// ---- outputs and inouts ----

#[unsafe(no_mangle)]
pub unsafe extern "C" fn split_long(x: i64, hi: *mut i32, lo: *mut i32) {
    unsafe {
        *hi = (x >> 32) as i32;
        *lo = x as i32;
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn swap_reals(a: *mut f64, b: *mut f64) {
    unsafe { std::ptr::swap(a, b) };
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn out_small(b: *mut i8, s: *mut i16, t: *mut u8, l: *mut u8, f: *mut f32, u: *mut u8) {
    unsafe {
        *b = -7;
        *s = 30000;
        *t = 1;
        *l = 3;
        *f = 0.75;
        *u = 200;
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn out_string(s: *mut *const c_char) {
    unsafe { *s = c"from C".as_ptr() };
}

/// An inout string arrives as a pointer to the old one, and leaves as a
/// new one (H.8.10).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shout(s: *mut *const c_char) {
    static mut BUF: [u8; 64] = [0; 64];
    let up = format!("{}!\0", text(unsafe { *s }).to_uppercase());
    unsafe {
        let buf = &mut *std::ptr::addr_of_mut!(BUF);
        buf[..up.len()].copy_from_slice(up.as_bytes());
        *s = buf.as_ptr() as *const c_char;
    }
}

// ---- packed values ----

#[unsafe(no_mangle)]
pub unsafe extern "C" fn vec_ops(a: *const u32, b: *const LogicVecVal, wide: *mut u32, l: *mut LogicVecVal) {
    unsafe {
        let a = *a & 0xff;
        let b = *b;
        // The 96-bit result: a in each byte of the top word, b's value and
        // unknown bits below.
        *wide.add(2) = a * 0x0101_0101;
        *wide.add(1) = b.aval & 0xff;
        *wide = (b.bval & 0xff) << 8;
        // The 70-bit inout: words swapped, and its top six bits inverted
        // where known.
        let (w0, w1, w2) = (*l, *l.add(1), *l.add(2));
        *l = w1;
        *l.add(1) = w0;
        let top = w2.aval & 0x3f;
        let unk = w2.bval & 0x3f;
        *l.add(2) = LogicVecVal { aval: (!top & !unk | top & unk) & 0x3f, bval: unk };
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn vec_popcount(v: *const u32) -> i32 {
    (0..4).map(|k| unsafe { *v.add(k) }.count_ones() as i32).sum()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn integer_time(i: *const LogicVecVal, t: *const LogicVecVal, oi: *mut LogicVecVal, ot: *mut LogicVecVal) {
    unsafe {
        let x = (*i).aval as i32;
        *oi = LogicVecVal { aval: (x * 3) as u32, bval: 0 };
        let tv = (*t).aval as u64 | ((*t.add(1)).aval as u64) << 32;
        let r = tv + 1;
        *ot = LogicVecVal { aval: r as u32, bval: 0 };
        *ot.add(1) = LogicVecVal { aval: (r >> 32) as u32, bval: 0 };
    }
}

/// `pk_t` is 8 bits: `hi` (two-state) above `lo`, so the struct is
/// four-state as a whole.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn packed_struct(p: *const LogicVecVal, q: *mut LogicVecVal) -> i32 {
    unsafe {
        let p = *p;
        *q = LogicVecVal { aval: (p.aval >> 4 & 0xf) + 1 << 4, bval: 0 };
        ((p.aval & 0xff) | (p.bval & 0xff) << 8) as i32
    }
}

// ---- unpacked structs and arrays ----

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rec_sum(r: *const Rec) -> f64 {
    let r = unsafe { &*r };
    r.count as f64 * r.scale + r.tag as f64 + (r.wide[1] & 0xff) as f64 + text(r.label).len() as f64
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rec_make(r: *mut Rec) {
    unsafe {
        *r = Rec { count: 9, scale: 0.125, tag: -1, wide: [0xdead_beef, 0xab], label: c"made".as_ptr() };
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn arr_sum(a: *const i32) -> i32 {
    (0..4).map(|k| unsafe { *a.add(k) } * (k as i32 + 1)).sum()
}

/// `m [2:0]` in C order is m[0], m[1], m[2]; it comes back reversed.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn arr_rev(m: *mut LogicVecVal) {
    unsafe {
        std::ptr::swap(m, m.add(2));
        (*m.add(1)).aval += 1;
    }
}

// ---- open arrays ----

#[unsafe(no_mangle)]
pub unsafe extern "C" fn open_sum(h: Handle) -> i32 {
    unsafe {
        let mut sum = 0;
        for i in svLow(h, 1)..=svHigh(h, 1) {
            sum += *(svGetArrElemPtr1(h, i) as *const i32) * (i + 1);
        }
        // The whole array in C layout, too.
        let n = svSizeOfArray(h) as usize / 4;
        let all = std::slice::from_raw_parts(svGetArrayPtr(h) as *const i32, n);
        sum * 1000 + all.iter().sum::<i32>()
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn open_fill(h: Handle) {
    unsafe {
        for i in svLow(h, 1)..=svHigh(h, 1) {
            let v = LogicVecVal { aval: 0xa000 + i as u32, bval: if i % 2 == 1 { 0xf } else { 0 } };
            svPutLogicArrElem1VecVal(h, &v, i);
        }
    }
}

/// An open packed formal: its width, and its ones counted.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn open_packed(h: Handle) -> i32 {
    unsafe {
        let w = svSize(h, 0);
        let p = svGetArrayPtr(h) as *const u32;
        let ones = (0..w).filter(|&i| svGetBitselBit(p, i) == 1).count() as i32;
        w * 100 + ones
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn open_2d(h: Handle, r: *mut i32) {
    unsafe {
        let r = std::slice::from_raw_parts_mut(r, 12);
        r[0] = svDimensions(h);
        r[1] = svLeft(h, 1);
        r[2] = svRight(h, 1);
        r[3] = svLow(h, 2);
        r[4] = svHigh(h, 2);
        r[5] = svIncrement(h, 1);
        r[6] = svSize(h, 1);
        r[7] = svSize(h, 0);
        let mut w = 0u32;
        svGetBitArrElemVecVal(&mut w, h, 3, 1);
        r[8] = w as i32;
        svGetBitArrElem2VecVal(&mut w, h, 5, 0);
        r[9] = w as i32;
        // An element's own representation is the simulator's (H.12.4);
        // its low byte holds these four bits in either form.
        r[10] = (*(svGetArrElemPtr(h, 4, 1) as *const u8) & 0xf) as i32;
        r[11] = svGetArrElemPtr2(h, 6, 0).is_null() as i32;
    }
}

/// Each scalar inverted where known; x and z swapped.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn open_scalars(h: Handle) {
    unsafe {
        for i in svLow(h, 1)..=svHigh(h, 1) {
            let v = svGetLogicArrElem1(h, i);
            let n = match v {
                0 => 1,
                1 => 0,
                2 => 3,
                _ => 2,
            };
            svPutLogicArrElem(h, n, i);
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn open_recs(h: Handle) -> f64 {
    unsafe { (svLow(h, 1)..=svHigh(h, 1)).map(|i| rec_sum(svGetArrElemPtr1(h, i) as *const Rec) * i as f64).sum() }
}

// ---- the canonical-form utilities ----

#[unsafe(no_mangle)]
pub unsafe extern "C" fn selects(v: *mut LogicVecVal, b: *const u32, r: *mut i32) {
    unsafe {
        let r = std::slice::from_raw_parts_mut(r, 6);
        r[0] = svGetBitselLogic(v, 0) as i32 * 10 + svGetBitselLogic(v, 3) as i32;
        r[1] = svGetBitselBit(b, 63) as i32;
        let mut part = 0xffff_ffffu32;
        svGetPartselBit(&mut part, b, 28, 8);
        r[2] = part as i32;
        let mut lp = LogicVecVal { aval: 0, bval: 0 };
        svGetPartselLogic(&mut lp, v, 30, 4);
        r[3] = (lp.aval | lp.bval << 4) as i32;
        // Writes that straddle the two words.
        svPutPartselLogic(v, LogicVecVal { aval: 0x5, bval: 0x2 }, 30, 4);
        svPutBitselLogic(v, 63, 2);
        r[4] = (*v.add(1)).aval as i32;
        r[5] = (*v.add(1)).bval as i32;
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn dpi_version() -> *const c_char {
    unsafe { svDpiVersion() }
}

// ---- context ----

#[unsafe(no_mangle)]
pub unsafe extern "C" fn scope_name() -> *const c_char {
    unsafe { svGetNameFromScope(svGetScope()) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn call_cell(path: *const c_char, k: i32) -> i32 {
    unsafe {
        let prev = svSetScope(svGetScopeFromName(path));
        let mut id = 0;
        let r = cell_id(k, &mut id);
        let back = svSetScope(prev);
        r * 10 + id + (back.is_null() as i32) * 1_000_000
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn no_such_scope() -> i32 {
    unsafe { svGetScopeFromName(c"top.nowhere".as_ptr()).is_null() as i32 }
}

/// Declared in `node`: the export called is the one in the import's own
/// instance.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ask_cell(k: i32) -> i32 {
    let mut id = 0;
    unsafe { cell_id(k, &mut id) + id * 1000 }
}

static KEY: u8 = 0;

#[unsafe(no_mangle)]
pub unsafe extern "C" fn remember(v: i32) {
    unsafe {
        let key = &KEY as *const u8 as *mut c_void;
        svPutUserData(svGetScope(), key, Box::into_raw(Box::new(v)) as *mut c_void);
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn recall() -> i32 {
    unsafe {
        let key = &KEY as *const u8 as *mut c_void;
        let p = svGetUserData(svGetScope(), key) as *const i32;
        if p.is_null() { -1 } else { *p }
    }
}

/// A function of the process, if it has it: the time functions are newer
/// than some simulators.
unsafe fn find<F: Copy>(name: &CStr) -> Option<F> {
    let any = if cfg!(target_os = "macos") { -2isize as *mut c_void } else { null_mut() };
    let p = unsafe { dlsym(any, name.as_ptr()) };
    (!p.is_null()).then(|| unsafe { std::mem::transmute_copy(&p) })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn time_info(unit: *mut i32, prec: *mut i32, ticks: *mut i64, scaled: *mut f64) -> i32 {
    type Get = unsafe extern "C" fn(Scope, *mut TimeVal) -> i32;
    type Scale = unsafe extern "C" fn(Scope, *mut i32) -> i32;
    unsafe {
        let (Some(svGetTime), Some(svGetTimeUnit), Some(svGetTimePrecision)) =
            (find::<Get>(c"svGetTime"), find::<Scale>(c"svGetTimeUnit"), find::<Scale>(c"svGetTimePrecision"))
        else {
            return -1;
        };
        let s = svGetScope();
        let mut ok = svGetTimeUnit(s, unit) + svGetTimePrecision(s, prec);
        let mut t = TimeVal { ty: 2, high: 0, low: 0, real: 0.0 };
        ok += svGetTime(null_mut(), &mut t);
        *ticks = (t.high as i64) << 32 | t.low as i64;
        let mut t = TimeVal { ty: 1, high: 0, low: 0, real: 0.0 };
        ok += svGetTime(s, &mut t);
        *scaled = t.real;
        ok
    }
}

// ---- tasks that wait ----

#[unsafe(no_mangle)]
pub unsafe extern "C" fn run_cell(path: *const c_char, n: i32, at: *mut i64) -> i32 {
    unsafe {
        let prev = svSetScope(svGetScopeFromName(path));
        let mut first = 0;
        cell_wait(n, &mut first);
        // An exported function, between two waits, in the same scope.
        let mut id = 0;
        let r = cell_id(0, &mut id);
        cell_wait(n, at);
        svSetScope(prev);
        trace(c"run_cell".as_ptr(), r + first as i32);
        0
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn nested(depth: i32) -> i32 {
    unsafe {
        trace(c"nested".as_ptr(), depth);
        descend(depth)
    }
}

static mut SAW: i32 = 0;

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wait_until_disabled() -> i32 {
    unsafe {
        let r = long_wait();
        let disabled = svIsDisabledState();
        SAW = r * 10 + disabled;
        if disabled != 0 {
            svAckDisabledState();
            return 1;
        }
        0
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn disabled_seen() -> i32 {
    unsafe { SAW }
}

/// A task that is not `context`: it runs like a function.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn plain_task(x: i32, y: *mut i32) -> i32 {
    unsafe { *y = x * 7 };
    0
}
