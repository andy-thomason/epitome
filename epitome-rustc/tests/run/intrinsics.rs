//! Intrinsics called directly: funnel shifts, disjoint `or`, the
//! widening multiply-add, carry-less multiplication, byte equality,
//! typed swaps, `TypeId`s, and the C library's maths functions.
#![feature(core_intrinsics, f16, funnel_shifts, uint_carryless_mul)]
#![allow(internal_features)]
#![no_std]
#![no_main]

use core::any::TypeId;
use core::hint::black_box;
use core::intrinsics;

unsafe extern "C" {
    fn putchar(c: i32) -> i32;
}

fn put_u128(mut m: u128) {
    let mut buf = [0u8; 40];
    let mut i = 0;
    loop {
        buf[i] = b'0' + (m % 10) as u8;
        i += 1;
        m /= 10;
        if m == 0 {
            break;
        }
    }
    while i > 0 {
        i -= 1;
        unsafe {
            putchar(buf[i] as i32);
        }
    }
}

fn put_i64(n: i64) {
    if n < 0 {
        unsafe {
            putchar(b'-' as i32);
        }
    }
    put_u128(n.unsigned_abs() as u128);
    unsafe {
        putchar(10);
    }
}

fn put_pair(a: u128, b: i128) {
    put_u128(a);
    unsafe {
        putchar(b' ' as i32);
    }
    if b < 0 {
        unsafe {
            putchar(b'-' as i32);
        }
    }
    put_u128(b.unsigned_abs());
    unsafe {
        putchar(10);
    }
}

/// `x` to three decimal places, as an integer.
fn milli(x: f32) -> i64 {
    (x * 1000.0) as i64
}

fn milli64(x: f64) -> i64 {
    (x * 1000.0) as i64
}

#[derive(Clone, Copy)]
struct Pair {
    a: u32,
    b: u16,
}

struct Wrap<T>(T);

#[unsafe(no_mangle)]
pub extern "C" fn main() -> i32 {
    // Funnel shifts.
    let x: u32 = black_box(0x1234_5678);
    let y: u32 = black_box(0x9abc_def0);
    for s in [0u32, 4, 31] {
        unsafe {
            put_i64(intrinsics::unchecked_funnel_shl(x, y, black_box(s)) as i64);
            put_i64(intrinsics::unchecked_funnel_shr(x, y, black_box(s)) as i64);
        }
    }
    let w: u8 = black_box(0xa5);
    unsafe {
        put_i64(intrinsics::unchecked_funnel_shl(w, 0x3c, black_box(3)) as i64);
        put_i64(intrinsics::unchecked_funnel_shr(w, 0x3c, black_box(3)) as i64);
    }
    unsafe {
        put_i64(intrinsics::disjoint_bitor(black_box(0xf0u16), 0x0f) as i64);
    }

    // a × b + c + d, its low and high halves.
    let (lo, hi) = intrinsics::carrying_mul_add(black_box(u64::MAX), u64::MAX, u64::MAX, u64::MAX);
    put_pair(lo as u128, hi as i128);
    let (lo, hi) = intrinsics::carrying_mul_add(black_box(0xdead_beefu32), 0x1234_5678, 7, 9);
    put_pair(lo as u128, hi as i128);
    let (lo, hi) = intrinsics::carrying_mul_add(black_box(-5i32), -2, 0, 0);
    put_pair(lo as u128, hi as i128);
    let (lo, hi) = intrinsics::carrying_mul_add(black_box(1_000_000_000i32), -10, 10, 10);
    put_pair(lo as u128, hi as i128);
    let (lo, hi) = intrinsics::carrying_mul_add(black_box(i64::MIN), i64::MAX, -1, 3);
    put_pair(lo as u128, hi as i128);
    let (lo, hi) = intrinsics::carrying_mul_add(black_box(u128::MAX - 12345), u128::MAX / 3, 99, u128::MAX);
    put_pair(lo, hi as i128);
    let (lo, hi) = intrinsics::carrying_mul_add(black_box(i128::MIN + 7), -3, i128::MAX, -1);
    put_pair(lo, hi);
    let (lo, hi) = intrinsics::carrying_mul_add(black_box(200u8), 201, 255, 255);
    put_pair(lo as u128, hi as i128);

    // Carry-less multiplication.
    put_i64(intrinsics::carryless_mul(black_box(0b1011u8), 0b0110) as i64);
    put_i64(intrinsics::carryless_mul(black_box(0x8765_4321u32), 0x1357_9bdf) as i64);
    put_u128(intrinsics::carryless_mul(black_box(u128::MAX / 7), 0x1_0000_0000_0000_0003));
    put_i64(0);

    // Byte equality.
    let p = Pair { a: 7, b: 9 };
    let q = Pair { a: 7, b: 9 };
    let r = Pair { a: 7, b: 8 };
    let s = black_box([1u64, 2, 3]);
    let t = black_box([1u64, 2, 4]);
    unsafe {
        put_i64(intrinsics::raw_eq(&p.a, &q.a) as i64);
        put_i64(intrinsics::raw_eq(&p.b, &r.b) as i64);
        put_i64(intrinsics::raw_eq(&s, &s) as i64);
        put_i64(intrinsics::raw_eq(&s, &t) as i64);
        put_i64(intrinsics::raw_eq(&(), &()) as i64);
    }

    // A typed swap.
    let mut a = black_box([1u32, 2, 3, 4, 5]);
    let mut b = black_box([9u32, 8, 7, 6, 5]);
    unsafe {
        intrinsics::typed_swap_nonoverlapping(&mut a, &mut b);
    }
    put_i64((a[0] * 10 + b[0]) as i64);
    put_i64((a[4] * 10 + b[2]) as i64);

    // TypeIds: the same type alike, different types not.
    let ids = [
        TypeId::of::<u32>(),
        TypeId::of::<i32>(),
        TypeId::of::<Pair>(),
        TypeId::of::<Wrap<u8>>(),
        TypeId::of::<Wrap<i8>>(),
        TypeId::of::<&'static str>(),
        TypeId::of::<[u8]>(),
        TypeId::of::<fn(u8) -> u16>(),
    ];
    let mut same = 0;
    for i in 0..ids.len() {
        for j in 0..ids.len() {
            if ids[i] == ids[j] {
                same += 1;
            }
        }
    }
    put_i64(same);
    put_i64((TypeId::of::<Wrap<u8>>() == black_box(ids[3])) as i64);
    put_i64((TypeId::of::<Wrap<u16>>() == black_box(ids[3])) as i64);

    // The C library's maths.
    let v: f32 = black_box(0.75);
    put_i64(milli(intrinsics::sinf32(v)));
    put_i64(milli(intrinsics::cosf32(v)));
    put_i64(milli(intrinsics::expf32(v)));
    put_i64(milli(intrinsics::exp2f32(v)));
    put_i64(milli(intrinsics::logf32(v)));
    put_i64(milli(intrinsics::log2f32(v)));
    put_i64(milli(intrinsics::log10f32(v)));
    put_i64(milli(intrinsics::powf32(v, 2.5)));
    let d: f64 = black_box(2.5);
    put_i64(milli64(intrinsics::sinf64(d)));
    put_i64(milli64(intrinsics::cosf64(d)));
    put_i64(milli64(intrinsics::expf64(d)));
    put_i64(milli64(intrinsics::logf64(d)));
    put_i64(milli64(intrinsics::powf64(d, 0.5)));
    let h: f16 = black_box(1.5);
    put_i64(milli(intrinsics::sinf16(h) as f32));
    put_i64(milli(intrinsics::expf16(h) as f32));
    0
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}
