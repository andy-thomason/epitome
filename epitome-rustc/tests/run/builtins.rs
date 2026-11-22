//! The `builtin #` forms, through the macros that expand to them:
//! `offset_of!`, `type_ascribe!`, `deref!` patterns, unsafe binders
//! (`wrap_binder!`, `unwrap_binder!`) and `field_of!`; and
//! `ptr::read_unaligned` and `write_unaligned`, which use `offset_of!`.
#![feature(type_ascription, deref_patterns, deref_pure_trait, unsafe_binders, field_projections, offset_of_enum)]
#![allow(incomplete_features)]
#![no_std]
#![no_main]

use core::any::TypeId;
use core::mem::{offset_of, size_of};
use core::ops::{Deref, DerefPure};
use core::ptr;
use core::unsafe_binder::{unwrap_binder, wrap_binder};

unsafe extern "C" {
    fn putchar(c: i32) -> i32;
}

fn put_i64(n: i64) {
    let mut buf = [0u8; 20];
    let mut i = 0;
    let neg = n < 0;
    let mut m = n.unsigned_abs();
    loop {
        buf[i] = b'0' + (m % 10) as u8;
        i += 1;
        m /= 10;
        if m == 0 {
            break;
        }
    }
    if neg {
        unsafe {
            putchar(b'-' as i32);
        }
    }
    while i > 0 {
        i -= 1;
        unsafe {
            putchar(buf[i] as i32);
        }
    }
    unsafe {
        putchar(10);
    }
}

#[repr(C)]
struct Inner {
    a: u8,
    b: u32,
    c: (u16, u64),
}

#[repr(C)]
struct Outer {
    x: u16,
    inner: Inner,
    y: [u8; 3],
}

struct Pair(u8, u64);

#[repr(u8)]
enum E {
    A(u8, u32),
    B { p: u16, q: u64 },
}

/// A wrapper that dereferences to what it holds.
struct Wrap<T>(T);

impl<T> Deref for Wrap<T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.0
    }
}

unsafe impl<T> DerefPure for Wrap<T> {}

fn classify(w: &Wrap<Option<u32>>) -> i64 {
    match w {
        deref!(Some(n)) if *n > 10 => *n as i64,
        deref!(Some(_)) => -1,
        deref!(None) => -2,
    }
}

fn inner(w: Wrap<Wrap<(u8, u16)>>) -> i64 {
    let deref!(deref!((a, b))) = w;
    a as i64 * 1000 + b as i64
}

#[unsafe(no_mangle)]
pub extern "C" fn main() -> i32 {
    // Offsets in `repr(C)` types, nested, and through enum variants.
    put_i64(offset_of!(Inner, b) as i64);
    put_i64(offset_of!(Inner, c) as i64);
    put_i64(offset_of!(Outer, inner) as i64);
    put_i64(offset_of!(Outer, inner.b) as i64);
    put_i64(offset_of!(Outer, y) as i64);
    put_i64(offset_of!(E, A.1) as i64);
    put_i64(offset_of!(E, B.q) as i64);
    // Where the layout is the compiler's to choose (tuples, `Pair`), the
    // offsets are checked against the fields' addresses.
    let o = Outer { x: 1, inner: Inner { a: 2, b: 3, c: (4, 5) }, y: [6; 3] };
    let base = &o as *const Outer as usize;
    put_i64((offset_of!(Outer, inner.c.0) == &o.inner.c.0 as *const u16 as usize - base) as i64);
    put_i64((offset_of!(Outer, inner.c.1) == &o.inner.c.1 as *const u64 as usize - base) as i64);
    let t = (1u8, 2u32, 3u16);
    let tb = &t as *const (u8, u32, u16) as usize;
    put_i64((offset_of!((u8, u32, u16), 2) == &t.2 as *const u16 as usize - tb) as i64);
    put_i64((offset_of!((u8, u32, u16), 0) == &t.0 as *const u8 as usize - tb) as i64);
    let pr = Pair(1, 2);
    let pb = &pr as *const Pair as usize;
    put_i64((offset_of!(Pair, 1) == &pr.1 as *const u64 as usize - pb) as i64);

    // Unaligned reads and writes.
    let bytes = [1u8, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12];
    let v = unsafe { ptr::read_unaligned(bytes.as_ptr().add(1) as *const u32) };
    put_i64(v as i64);
    let w = unsafe { ptr::read_unaligned(bytes.as_ptr().add(3) as *const u64) };
    put_i64(w as i64);
    let mut out = [0u8; 10];
    unsafe {
        ptr::write_unaligned(out.as_mut_ptr().add(1) as *mut u64, 0x0102_0304_0506_0708);
        ptr::write_unaligned(out.as_mut_ptr().add(9) as *mut u8, 0xff);
    }
    put_i64(out.iter().enumerate().fold(0i64, |s, (i, &b)| s + (i as i64 + 1) * b as i64));

    // Type ascription.
    let t = core::prelude::v1::type_ascribe!(7 + 8, u64);
    put_i64((t << 40) as i64);

    // Deref patterns.
    put_i64(classify(&Wrap(Some(42))));
    put_i64(classify(&Wrap(Some(3))));
    put_i64(classify(&Wrap(None)));
    put_i64(inner(Wrap(Wrap((7, 345)))));

    // Unsafe binders.
    let x = 1234u32;
    let b: unsafe<'a> &'a u32 = unsafe { wrap_binder!(&x) };
    let r: &u32 = unsafe { unwrap_binder!(b) };
    put_i64(*r as i64);

    // Field-representing types: each field its own type, of no size.
    type Fa = core::field::field_of!(Inner, a);
    type Fb = core::field::field_of!(Inner, b);
    type Fq = core::field::field_of!(E, B.q);
    put_i64(size_of::<Fa>() as i64);
    put_i64((TypeId::of::<Fa>() == TypeId::of::<Fb>()) as i64);
    put_i64((TypeId::of::<Fa>() == TypeId::of::<core::field::field_of!(Inner, a)>()) as i64);
    put_i64((TypeId::of::<Fq>() == TypeId::of::<Fb>()) as i64);
    0
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}
