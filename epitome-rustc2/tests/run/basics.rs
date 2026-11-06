//! Integers, structs, enums, references, generics and a trait method.
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
        put_char(buf[i]);
    }
    put_char(b'\n');
}

struct Point {
    x: i64,
    y: i64,
}

enum Shape {
    Circle(u64),
    Rect { w: u64, h: u64 },
    Empty,
}

trait Area {
    fn area(&self) -> u64;
}

impl Area for Shape {
    fn area(&self) -> u64 {
        match self {
            Shape::Circle(r) => 3 * r * r,
            Shape::Rect { w, h } => w * h,
            Shape::Empty => 0,
        }
    }
}

fn total<T: Area>(xs: &[T]) -> u64 {
    let mut sum = 0;
    let mut i = 0;
    while i < xs.len() {
        sum += xs[i].area();
        i += 1;
    }
    sum
}

fn swap(p: &mut Point) {
    let t = p.x;
    p.x = p.y;
    p.y = t;
}

fn first_even(xs: &[u32]) -> Option<u32> {
    for &x in xs {
        if x % 2 == 0 {
            return Some(x);
        }
    }
    None
}

#[unsafe(no_mangle)]
pub extern "C" fn main() -> i32 {
    let mut p = Point { x: 3, y: 40 };
    swap(&mut p);
    put_u64((p.x * 10 + p.y) as u64);
    let shapes = [Shape::Circle(2), Shape::Rect { w: 3, h: 4 }, Shape::Empty];
    put_u64(total(&shapes));
    match first_even(&[3, 5, 8, 9]) {
        Some(x) => put_u64(x as u64),
        None => put_u64(0),
    }
    let n = if let Some(x) = first_even(&[1, 3]) { x } else { 7 };
    put_u64(n as u64);
    0
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}
