//! `f16` (half precision) arithmetic, comparisons, conversions and
//! methods, and float-to-integer casts, which saturate (NaN gives 0).
#![feature(f16)]
#![no_std]
#![no_main]

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

/// `x` to three decimal places, as an integer.
fn milli(x: f16) -> i64 {
    (x as f32 * 1000.0) as i64
}

fn halve(x: f16) -> f16 {
    x / 2.0
}

fn poly(x: f16) -> f16 {
    x * x - 3.0 * x + 0.5
}

#[unsafe(no_mangle)]
pub extern "C" fn main() -> i32 {
    let a: f16 = 1.5;
    let b: f16 = 2.25;
    put_i64(milli(a + b));
    put_i64(milli(a - b));
    put_i64(milli(a * b));
    put_i64(milli(a / b));
    put_i64(milli(-a));
    put_i64(milli(halve(b)));
    put_i64(milli(poly(b)));
    put_i64((a < b) as i64);
    put_i64((a == 1.5) as i64);
    put_i64((b >= 3.0) as i64);
    // Rounding to half precision: 0.1 is not exact.
    put_i64(((0.1f16 as f32) * 1_000_000.0) as i64);
    // The bits, both ways.
    put_i64(b.to_bits() as i64);
    put_i64(milli(f16::from_bits(0x3c00)));
    // Large and small: 65504 is the largest finite f16.
    let big: f16 = 60000.0;
    put_i64(big as i64);
    put_i64((big * 2.0) as i64);
    put_i64((big * 2.0 > big) as i64);
    put_i64(milli((1.0f32 / 3.0) as f16));
    put_i64(milli(7.0f64 as f16));
    put_i64((300i32 as f16) as i64);
    put_i64((a as f64 * 1000.0) as i64);
    // Methods of `core`, through its maths intrinsics.
    let c: f16 = -2.75;
    put_i64(milli(c.abs()));
    put_i64(milli(c.floor()));
    put_i64(milli(c.ceil()));
    put_i64(milli(c.trunc()));
    put_i64(milli(c.round()));
    put_i64(milli(c.copysign(1.0)));
    put_i64(milli(c.max(a)));
    put_i64(milli(c.min(a)));
    put_i64(milli(a.mul_add(b, c)));
    put_i64(milli(10.0f16.sqrt()));
    put_i64(milli(2.5f16.round_ties_even()));
    put_i64(milli(3.5f16.round_ties_even()));
    // `f32` and `f64` have these in `core` (the rest are in `std`).
    put_i64(((-7.25f32).abs() * 100.0) as i64);
    put_i64((f64::NAN.max(3.0)) as i64);
    put_i64((2.0f64.min(f64::NAN)) as i64);
    put_i64(((-0.5f64).copysign(1.0) * 10.0) as i64);
    put_i64((1.5f32.clamp(2.0, 3.0) * 10.0) as i64);
    let mut acc: f16 = 0.0;
    for i in 0..10 {
        acc += i as f16 * 0.5;
    }
    put_i64(milli(acc));

    // Casts saturate, and NaN is 0.
    put_i64(300.7f32 as u8 as i64);
    put_i64(-5.5f32 as u8 as i64);
    put_i64(-5.5f32 as i8 as i64);
    put_i64(1e20f64 as i32 as i64);
    put_i64(-1e20f64 as i32 as i64);
    put_i64(f32::NAN as i32 as i64);
    put_i64(f64::INFINITY as u16 as i64);
    put_i64(big as u8 as i64);
    put_i64((big * 2.0) as i16 as i64);
    0
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}
