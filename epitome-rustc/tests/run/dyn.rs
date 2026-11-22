//! Trait objects: `&dyn` and `&mut dyn` of several types, in slices and
//! struct fields; default methods and supertraits' methods called through
//! `dyn`; a generic trait; results larger than a register; `size_of_val`
//! and `align_of_val`; `dyn Tr + Send` made `dyn Tr`; `dyn Any`; and
//! closures as `dyn Fn` and `dyn FnMut`.
#![no_std]
#![no_main]

use core::any::Any;

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

trait Named {
    fn id(&self) -> i64;
}

trait Shape: Named {
    fn area(&self) -> i64;
    fn scale(&mut self, k: i64);
    fn describe(&self) -> i64 {
        self.id() * 1000 + self.area()
    }
    fn corners(&self) -> [i64; 4] {
        [0, 0, self.area(), self.id()]
    }
}

struct Square(i64);
struct Rect {
    w: i64,
    h: i64,
}
struct Unit;

impl Named for Square {
    fn id(&self) -> i64 {
        1
    }
}
impl Named for Rect {
    fn id(&self) -> i64 {
        2
    }
}
impl Named for Unit {
    fn id(&self) -> i64 {
        3
    }
}

impl Shape for Square {
    fn area(&self) -> i64 {
        self.0 * self.0
    }
    fn scale(&mut self, k: i64) {
        self.0 *= k;
    }
}
impl Shape for Rect {
    fn area(&self) -> i64 {
        self.w * self.h
    }
    fn scale(&mut self, k: i64) {
        self.w *= k;
        self.h *= k;
    }
    fn describe(&self) -> i64 {
        -self.area()
    }
    fn corners(&self) -> [i64; 4] {
        [self.w, self.h, self.w + self.h, 7]
    }
}
impl Shape for Unit {
    fn area(&self) -> i64 {
        0
    }
    fn scale(&mut self, _: i64) {}
}

trait Apply<T> {
    fn apply(&self, x: T) -> T;
}

struct AddN(i64);
impl Apply<i64> for AddN {
    fn apply(&self, x: i64) -> i64 {
        x + self.0
    }
}
struct Twice;
impl Apply<i64> for Twice {
    fn apply(&self, x: i64) -> i64 {
        x * 2
    }
}

struct Holder<'a> {
    tag: i64,
    shape: &'a dyn Shape,
}

fn total(shapes: &[&dyn Shape]) -> i64 {
    shapes.iter().map(|s| s.area()).sum()
}

fn grow(s: &mut dyn Shape) {
    s.scale(3);
}

fn chain(fs: &[&dyn Apply<i64>], mut x: i64) -> i64 {
    for f in fs {
        x = f.apply(x);
    }
    x
}

fn name_of(n: &dyn Named) -> i64 {
    n.id()
}

fn send_away(s: &(dyn Shape + Send)) -> &dyn Shape {
    s
}

fn which(a: &dyn Any) -> i64 {
    if let Some(x) = a.downcast_ref::<i64>() {
        *x
    } else if let Some(s) = a.downcast_ref::<Square>() {
        100 + s.0
    } else {
        -1
    }
}

fn run(f: &dyn Fn(i64) -> i64, x: i64) -> i64 {
    f(x)
}

fn run_mut(f: &mut dyn FnMut(i64), x: i64) {
    f(x)
}

#[unsafe(no_mangle)]
pub extern "C" fn main() -> i32 {
    let sq = Square(4);
    let rc = Rect { w: 2, h: 5 };
    let u = Unit;
    let shapes: [&dyn Shape; 3] = [&sq, &rc, &u];
    put_i64(total(&shapes));
    for s in &shapes {
        put_i64(s.describe());
        put_i64(s.id());
        let c = s.corners();
        put_i64(c[0] + c[1] * 10 + c[2] * 100 + c[3] * 1000);
        put_i64(core::mem::size_of_val(*s) as i64);
        put_i64(core::mem::align_of_val(*s) as i64);
    }

    let mut big = Square(2);
    grow(&mut big);
    put_i64(big.0);
    let mut r2 = Rect { w: 1, h: 2 };
    {
        let d: &mut dyn Shape = &mut r2;
        d.scale(5);
        put_i64(d.area());
    }
    put_i64(r2.w + r2.h);

    let h = Holder { tag: 9, shape: &rc };
    put_i64(h.tag + h.shape.area());

    let a = AddN(3);
    let t = Twice;
    put_i64(chain(&[&a, &t, &a], 1));
    put_i64(chain(&[&t, &t, &t], 5));

    put_i64(name_of(&u) + name_of(&sq));
    put_i64(send_away(&sq).area());

    put_i64(which(&42i64));
    put_i64(which(&Square(5)));
    put_i64(which(&1u8));

    let k = 10;
    put_i64(run(&|x| x + k, 5));
    put_i64(run(&|x| x * x, 9));
    let mut acc = 0;
    run_mut(&mut |x| acc += x, 7);
    run_mut(&mut |x| acc += x * 2, 1);
    put_i64(acc);
    let ops: [&dyn Fn(i64, i64) -> i64; 2] = [&|a, b| a + b, &|a, b| a * b];
    for f in ops {
        put_i64(f(6, 7));
    }
    0
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}
