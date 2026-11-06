//! `for` over ranges and slices through `core`'s iterators, and a user
//! iterator.
#![no_std]
#![no_main]

unsafe extern "C" {
    fn putchar(c: i32) -> i32;
}

fn put_u64(mut n: u64) {
    let mut buf = [0u8; 20];
    let mut i = 0;
    loop {
        buf[i] = b'0' + (n % 10) as u8;
        i += 1;
        n /= 10;
        if n == 0 {
            break;
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

struct Fib {
    a: u64,
    b: u64,
}

impl Iterator for Fib {
    type Item = u64;
    fn next(&mut self) -> Option<u64> {
        let r = self.a;
        self.a = self.b;
        self.b += r;
        Some(r)
    }
}

struct Countdown(u32);

impl Iterator for Countdown {
    type Item = u32;
    fn next(&mut self) -> Option<u32> {
        if self.0 == 0 {
            return None;
        }
        self.0 -= 1;
        Some(self.0)
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn main() -> i32 {
    let mut total = 0;
    for i in 0..10 {
        total += i;
    }
    put_u64(total);
    let mut prod = 1u64;
    for i in 1..=10u64 {
        prod *= i;
    }
    put_u64(prod);
    for i in (0..5u64).rev() {
        put_u64(i);
    }
    let xs = [5u64, 10, 15, 20];
    let mut weighted = 0;
    for (i, &x) in xs.iter().enumerate() {
        weighted += i as u64 * x;
    }
    put_u64(weighted);
    let ys = [1u64, 2, 3];
    let mut dot = 0;
    for (x, y) in xs.iter().zip(ys.iter()) {
        dot += x * y;
    }
    put_u64(dot);
    let mut fib = Fib { a: 0, b: 1 };
    for _ in 0..10 {
        fib.next();
    }
    put_u64(fib.next().unwrap_or(0));
    let mut n = 0;
    for v in (Fib { a: 0, b: 1 }) {
        if v > 1000 {
            break;
        }
        n += 1;
    }
    put_u64(n);
    for c in Countdown(3) {
        put_u64(c as u64);
    }
    let mut m = [1u64, 2, 3, 4];
    for v in m.iter_mut() {
        *v *= *v;
    }
    put_u64(m[3]);
    put_u64(Countdown(5).count() as u64);
    put_u64(xs.iter().sum::<u64>());
    put_u64(Countdown(4).last().unwrap_or(99) as u64);
    0
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}
