//! Generic structs and functions, traits with default methods and
//! associated types, derives, and methods taking `self` every way.
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

fn put_i64(n: i64) {
    let mut buf = [0u8; 20];
    let mut i = 0;
    let mut m = if n < 0 { (n as i128).unsigned_abs() as u64 } else { n as u64 };
    loop {
        buf[i] = b'0' + (m % 10) as u8;
        i += 1;
        m /= 10;
        if m == 0 {
            break;
        }
    }
    if n < 0 {
        put_str("-");
    }
    while i > 0 {
        i -= 1;
        unsafe {
            putchar(buf[i] as i32);
        }
    }
    put_str("\n");
}

#[derive(Clone, Copy, PartialEq, Debug)]
struct Pair<T> {
    a: T,
    b: T,
}

impl<T: Copy + PartialOrd> Pair<T> {
    fn new(a: T, b: T) -> Self {
        Pair { a, b }
    }
    fn larger(&self) -> T {
        if self.a > self.b { self.a } else { self.b }
    }
    fn swap(self) -> Self {
        Pair { a: self.b, b: self.a }
    }
    fn set_a(&mut self, a: T) {
        self.a = a;
    }
}

trait Shape {
    fn area(&self) -> i64;
    fn name(&self) -> &'static str;
    fn describe(&self) {
        put_str(self.name());
        put_str(" ");
        put_i64(self.area());
    }
}

struct Square(i64);
struct Rect {
    w: i64,
    h: i64,
}

impl Shape for Square {
    fn area(&self) -> i64 {
        self.0 * self.0
    }
    fn name(&self) -> &'static str {
        "square"
    }
}

impl Shape for Rect {
    fn area(&self) -> i64 {
        self.w * self.h
    }
    fn name(&self) -> &'static str {
        "rect"
    }
    fn describe(&self) {
        put_str("a rectangle of area ");
        put_i64(self.area());
    }
}

fn biggest<S: Shape>(shapes: &[S]) -> i64 {
    let mut best = i64::MIN;
    for s in shapes {
        if s.area() > best {
            best = s.area();
        }
    }
    best
}

trait Container {
    type Item;
    fn get(&self, i: usize) -> Option<Self::Item>;
    fn first(&self) -> Option<Self::Item> {
        self.get(0)
    }
}

struct Stack<const N: usize> {
    items: [i64; N],
    len: usize,
}

impl<const N: usize> Stack<N> {
    fn new() -> Self {
        Stack { items: [0; N], len: 0 }
    }
    fn push(&mut self, x: i64) -> bool {
        if self.len == N {
            return false;
        }
        self.items[self.len] = x;
        self.len += 1;
        true
    }
    fn pop(&mut self) -> Option<i64> {
        if self.len == 0 {
            None
        } else {
            self.len -= 1;
            Some(self.items[self.len])
        }
    }
}

impl<const N: usize> Container for Stack<N> {
    type Item = i64;
    fn get(&self, i: usize) -> Option<i64> {
        if i < self.len { Some(self.items[i]) } else { None }
    }
}

fn sum_all<C: Container<Item = i64>>(c: &C) -> i64 {
    let mut i = 0;
    let mut total = 0;
    while let Some(x) = c.get(i) {
        total += x;
        i += 1;
    }
    total
}

fn max_of<T: PartialOrd + Copy>(xs: &[T]) -> Option<T> {
    let mut it = xs.iter();
    let mut best = *it.next()?;
    for &x in it {
        if x > best {
            best = x;
        }
    }
    Some(best)
}

#[unsafe(no_mangle)]
pub extern "C" fn main() -> i32 {
    let mut p = Pair::new(3i64, 9);
    put_i64(p.larger());
    let q = p.swap();
    put_i64(q.a);
    p.set_a(20);
    put_i64(p.larger());
    put_i64((p == q) as i64);
    let r = p;
    put_i64((p == r) as i64);
    let pu = Pair::new(2u8, 1);
    put_i64(pu.larger() as i64);

    Square(4).describe();
    Rect { w: 2, h: 5 }.describe();
    put_i64(biggest(&[Square(2), Square(7), Square(3)]));
    put_i64(biggest(&[Rect { w: 1, h: 1 }, Rect { w: 3, h: 3 }]));

    let mut s: Stack<4> = Stack::new();
    for i in 1..=6 {
        if !s.push(i * 10) {
            put_str("full at ");
            put_i64(i);
            break;
        }
    }
    put_i64(sum_all(&s));
    put_i64(s.first().unwrap_or(-1));
    put_i64(s.pop().unwrap_or(-1));
    put_i64(s.pop().unwrap_or(-1));
    put_i64(sum_all(&s));

    put_i64(max_of(&[3, -8, 12, 5]).unwrap_or(0));
    put_i64(max_of::<i64>(&[]).unwrap_or(-99));
    0
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}
