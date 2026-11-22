//! Integer and float arithmetic, shifts, casts, `char`, constants and
//! statics, and integer methods from `core`.
#![no_std]
#![no_main]

unsafe extern "C" {
    fn putchar(c: i32) -> i32;
}

fn put_str(s: &str) {
    for &b in s.as_bytes() {
        unsafe {
            putchar(b as i32);
        }
    }
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
    put_str("\n");
}

fn put_i64(n: i64) {
    if n < 0 {
        put_str("-");
        put_u128((n as i128).unsigned_abs());
    } else {
        put_u128(n as u128);
    }
}

const LIMIT: u32 = 1 << 10;
const MASKS: [u8; 3] = [0x0f, 0xf0, 0xff];
static mut COUNTER: u32 = 7;
static GREETING: u32 = 42;

struct Wrapper;
impl Wrapper {
    const SCALE: i64 = -3;
}

trait HasZero {
    const ZERO: Self;
}
impl HasZero for i64 {
    const ZERO: i64 = 0;
}

fn zero_of<T: HasZero>() -> T {
    T::ZERO
}

fn bump() -> u32 {
    unsafe {
        COUNTER += 1;
        COUNTER
    }
}

fn upper(c: char) -> char {
    if c.is_ascii_lowercase() { ((c as u8) - 32) as char } else { c }
}

#[unsafe(no_mangle)]
pub extern "C" fn main() -> i32 {
    let a: i32 = -17;
    let b: i32 = 5;
    put_i64((a / b) as i64);
    put_i64((a % b) as i64);
    put_i64((a >> 2) as i64);
    put_i64(((a as u32) >> 28) as i64);
    put_i64((b << 3) as i64);
    put_i64((a & 0xff) as i64);
    put_i64((a | 3) as i64);
    put_i64((a ^ b) as i64);
    put_i64(!b as i64);
    put_i64(-a as i64);

    let x: u8 = 250;
    put_i64(x.wrapping_add(10) as i64);
    put_i64(x.checked_add(10).map_or(-1, |v| v as i64));
    put_i64(x.saturating_add(10) as i64);
    put_i64(200u8.checked_add(10).map_or(-1, |v| v as i64));

    let big: u128 = 1 << 100;
    put_u128(big + 12345);
    put_u128(u64::MAX as u128 * 3);
    let neg: i128 = -(1i128 << 70);
    put_i64((neg >> 64) as i64);

    put_i64(LIMIT as i64);
    put_i64(MASKS[1] as i64);
    put_i64(bump() as i64);
    put_i64(bump() as i64);
    put_i64(GREETING as i64);
    put_i64(Wrapper::SCALE * 2);
    put_i64(zero_of::<i64>() + 1);
    put_i64(i16::MAX as i64);
    put_i64(u32::BITS as i64);

    put_i64(37u32.leading_zeros() as i64);
    put_i64(40u64.trailing_zeros() as i64);
    put_i64(0xf0f0u16.count_ones() as i64);
    put_i64(3i64.pow(13));
    put_i64((-9i64).abs());
    put_i64(7u32.max(3) as i64);
    put_i64(7i32.min(-3) as i64);
    put_i64(0x1234u16.swap_bytes() as i64);
    put_i64(0x80000001u32.rotate_left(4) as i64);

    let f = 2.5f64;
    let g = f * 4.0 - 1.5;
    put_i64(g as i64);
    put_i64((g * 100.0) as i64);
    put_i64((1.0f32 / 3.0 * 3000.0) as i64);
    put_i64((-7.9f64) as i64);
    put_i64(((3 as f64) / 2.0 * 10.0) as i64);
    put_i64((g > f) as i64);

    let c = 'q';
    put_i64(c as i64);
    put_i64(upper(c) as i64);
    put_i64(upper('Z') as i64);
    put_i64(('a' as u8 + 2) as char as i64);
    put_i64(char::from_u32(0x41).map_or(0, |c| c as i64));
    let e = 'é';
    put_i64(e.len_utf8() as i64);
    0
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}
