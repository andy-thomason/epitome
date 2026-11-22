//! `f128` (quad precision): literals rounded exactly, arithmetic beyond
//! `f64`'s precision, comparisons, conversions to and from integers and
//! the other floats (casts to integers saturating), and its methods in
//! `core`.
#![feature(f128, f16)]
#![no_std]
#![no_main]

unsafe extern "C" {
    fn putchar(c: i32) -> i32;
}

fn put_char(c: u8) {
    unsafe {
        putchar(c as i32);
    }
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
        put_char(b'-');
    }
    while i > 0 {
        i -= 1;
        put_char(buf[i]);
    }
    put_char(b'\n');
}

fn put_hex(n: u128) {
    for i in (0..32).rev() {
        let d = (n >> (i * 4)) as u8 & 15;
        put_char(if d < 10 { b'0' + d } else { b'a' + d - 10 });
    }
    put_char(b'\n');
}

fn third(x: f128) -> f128 {
    x / 3.0
}

#[unsafe(no_mangle)]
pub extern "C" fn main() -> i32 {
    // Literals, by their bits.
    put_hex(1.0f128.to_bits());
    put_hex(0.1f128.to_bits());
    put_hex(core::f128::consts::PI.to_bits());
    put_hex(f128::MAX.to_bits());
    put_hex(f128::EPSILON.to_bits());
    put_hex(f128::MIN_POSITIVE.to_bits());
    put_hex(1e-4960f128.to_bits());
    put_hex(12345678901234567890123456789.0f128.to_bits());
    // Arithmetic, with more precision than f64 has.
    let a: f128 = 1.0;
    let t = third(a);
    put_hex(t.to_bits());
    put_hex((t * 3.0).to_bits());
    let big: f128 = 9007199254740993.0; // 2^53 + 1: not an f64
    put_i64((big + 1.0 - 9007199254740992.0) as i64);
    put_i64(((big as f64) as f128 == big) as i64);
    put_hex((-t).to_bits());
    put_hex((t - 1.0).to_bits());
    put_hex((t * t + 0.5).to_bits());
    put_i64((7.5f128 % 2.0 * 10.0) as i64);
    // Comparisons, NaN included.
    put_i64((t < 0.5) as i64);
    put_i64((t >= 0.34) as i64);
    put_i64((t == t) as i64);
    let nan = f128::NAN;
    put_i64((nan == nan) as i64);
    put_i64((nan < 1.0 || nan >= 1.0) as i64);
    put_i64(nan.is_nan() as i64);
    // Conversions.
    put_i64((t * 1e18) as i64);
    put_i64((2.5f128 as f64 * 4.0) as i64);
    put_i64(((1.0f64 / 3.0) as f128 == t) as i64);
    put_i64((1.5f32 as f128 * 2.0) as i64);
    put_i64(((0.75f16 as f128) * 100.0) as i64);
    put_i64(((t as f16) as f32 * 10000.0) as i64);
    put_i64((-123456789i64 as f128 / 1000.0) as i64);
    put_i64((200u8 as f128 + 0.5) as i64);
    put_hex(u128::MAX as f128 as u128);
    put_hex((-3i128 as f128).to_bits());
    // Casts saturate, and NaN is 0.
    put_i64(1e30f128 as i64);
    put_i64(-1e30f128 as i32 as i64);
    put_i64(300.5f128 as u8 as i64);
    put_i64(-2.5f128 as u16 as i64);
    put_i64(nan as i64);
    put_hex(f128::INFINITY as u128);
    put_hex(1e40f128 as u128);
    // Methods in core.
    put_i64(((-2.25f128).abs() * 100.0) as i64);
    put_i64((2.0f128.copysign(-1.0)) as i64);
    put_i64((t.max(0.5) * 10.0) as i64);
    put_i64((t.min(0.5) * 1e6) as i64);
    put_i64((nan.max(4.0)) as i64);
    put_i64((-0.0f128).is_sign_negative() as i64);
    put_i64(f128::INFINITY.is_infinite() as i64);
    put_hex(1.5f128.powi(7).to_bits());
    put_hex(t.powi(-3).to_bits());
    0
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}
