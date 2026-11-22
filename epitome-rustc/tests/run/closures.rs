//! Closures (capturing by reference and by value, `FnMut` state, passed
//! to generic functions and returned as `impl Fn`), and `core`'s
//! iterator adapters and `Option` and `Result` combinators.
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
    for &c in buf[..i].iter().rev() {
        unsafe {
            putchar(c as i32);
        }
    }
    unsafe {
        putchar(10);
    }
}

fn apply<F: Fn(i64) -> i64>(f: F, x: i64) -> i64 {
    f(f(x))
}

fn call_n(mut f: impl FnMut(), n: u32) {
    for _ in 0..n {
        f();
    }
}

fn adder(k: i64) -> impl Fn(i64) -> i64 {
    move |x| x + k
}

fn compose<A, B, C>(f: impl Fn(A) -> B, g: impl Fn(B) -> C) -> impl Fn(A) -> C {
    move |x| g(f(x))
}

fn by_pointer(f: fn(i64) -> i64, x: i64) -> i64 {
    f(x) + 1
}

fn triple(x: i64) -> i64 {
    3 * x
}

fn parse_digit(c: u8) -> Result<i64, u8> {
    if c.is_ascii_digit() { Ok((c - b'0') as i64) } else { Err(c) }
}

#[unsafe(no_mangle)]
pub extern "C" fn main() -> i32 {
    let base = 10;
    put_i64(apply(|x| x * 2 + base, 1));
    let mut count = 0;
    call_n(|| count += 3, 4);
    put_i64(count);
    let add5 = adder(5);
    put_i64(add5(37));
    let f = compose(adder(1), |x: i64| x * x);
    put_i64(f(6));
    put_i64(by_pointer(triple, 7));
    let mut log = [0i64; 4];
    let mut n = 0;
    let mut push = |v: i64| {
        log[n] = v;
        n += 1;
    };
    push(4);
    push(9);
    put_i64(log[0] + log[1] + n as i64);

    let xs = [3i64, -1, 4, -1, 5, -9, 2, 6];
    put_i64(xs.iter().map(|&x| x * x).sum());
    put_i64(xs.iter().filter(|&&x| x > 0).count() as i64);
    put_i64(xs.iter().fold(0, |acc, &x| acc * 2 + x));
    put_i64(xs.iter().copied().max().unwrap_or(0));
    put_i64(xs.iter().position(|&x| x == 5).map_or(-1, |i| i as i64));
    put_i64(xs.iter().any(|&x| x == -9) as i64);
    put_i64(xs.iter().all(|&x| x < 7) as i64);
    put_i64(xs.iter().rev().skip(1).map(|x| x * 10).sum());
    put_i64(xs.iter().take_while(|x| **x != 5).count() as i64);
    put_i64((1..=5i64).map(|i| i * i).filter(|i| i % 2 == 1).sum());

    let some: Option<i64> = Some(4);
    let none: Option<i64> = None;
    put_i64(some.map(|x| x + 1).unwrap_or(0));
    put_i64(none.map(|x| x + 1).unwrap_or_else(|| -1));
    put_i64(some.and_then(|x| if x > 3 { Some(x * 10) } else { None }).unwrap_or(0));
    put_i64(some.filter(|x| *x > 10).is_some() as i64);
    put_i64(none.or(Some(8)).unwrap());
    let digits: Result<i64, u8> = parse_digit(b'7');
    put_i64(digits.map(|d| d * 3).unwrap_or(0));
    put_i64(parse_digit(b'x').map_err(|c| c as i64).unwrap_err());
    put_i64(parse_digit(b'2').ok().unwrap_or(0));
    0
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}
