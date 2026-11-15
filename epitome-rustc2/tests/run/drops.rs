//! Drops: locals at the end of their block, in reverse order, and
//! parameters after them; values moved away (wholly, partly, or on only
//! some paths) dropped once, where they end up; assignments dropping what
//! was there; temporaries at the end of their statement, or of the block
//! a `let` extends them to; `break`, `continue`, `return` and `?` dropping
//! what they leave; what patterns bind in `match`, `if let`, `while let`
//! and `for`; closures that capture by value; drop glue of tuples, arrays,
//! enums and `dyn`; `ManuallyDrop`, `mem::forget` and `needs_drop`.
#![no_std]
#![no_main]
#![allow(dead_code)]

use core::mem::{self, ManuallyDrop};

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

/// A section of the output.
fn section(n: i64) {
    put_i64(-n);
}

/// Says when it is dropped.
struct D(i64);

impl Drop for D {
    fn drop(&mut self) {
        put_i64(self.0);
    }
}

impl D {
    fn id(&self) -> i64 {
        self.0
    }
}

fn make(n: i64) -> D {
    D(n)
}

struct Pair {
    a: D,
    b: D,
}

/// No `Drop` of its own: its fields are dropped, in order.
struct Three(D, i64, D);

enum E {
    One(D),
    Two(D, D),
    Nothing,
}

trait Speak {
    fn say(&self) -> i64;
}

impl Speak for D {
    fn say(&self) -> i64 {
        self.0 * 10
    }
}

fn scopes() {
    section(1);
    let _a = D(1);
    {
        let _b = D(2);
        let _c = D(3);
    }
    let _d = D(4);
}

fn moves(flag: bool) {
    section(2);
    let a = D(10);
    let b = D(11);
    let c = a;
    if flag {
        mem::drop(b);
        put_i64(12);
    }
    put_i64(c.id());
}

fn take(d: D) -> i64 {
    d.0 + 1000
}

fn params(x: D, y: D) {
    section(3);
    let _z = D(22);
    put_i64(x.id() + y.id());
}

fn partial() {
    section(4);
    let p = Pair { a: D(30), b: D(31) };
    let a = p.a;
    put_i64(a.id());
    let t = Three(D(32), 7, D(33));
    let Three(_, n, last) = t;
    put_i64(n);
    put_i64(last.id());
}

fn assign() {
    section(5);
    let mut x = D(40);
    x = D(41);
    let y;
    y = D(42);
    let mut p = Pair { a: D(43), b: D(44) };
    p.b = D(45);
    let moved = p.a;
    p.a = D(46);
    put_i64(moved.id() + y.id() + x.id());
}

fn temporaries() {
    section(6);
    put_i64(make(50).id());
    make(51);
    let _ = make(52);
    let r = &make(53);
    put_i64(r.id());
    let n = { make(54).id() };
    put_i64(n);
}

fn early(n: i64) -> i64 {
    let _a = D(60);
    for i in 0..5 {
        let _b = D(61 + i);
        if i == 1 {
            continue;
        }
        if i == n {
            return i;
        }
        if i == 3 {
            break;
        }
    }
    let _c = D(70);
    -1
}

fn check(n: i64) -> Option<D> {
    if n > 0 { Some(D(n)) } else { None }
}

fn question(n: i64) -> Option<i64> {
    let _guard = D(80);
    let d = check(n)?;
    Some(d.id())
}

fn patterns() {
    section(9);
    let e = E::Two(D(90), D(91));
    match e {
        E::One(d) => put_i64(d.id()),
        E::Two(x, _) => put_i64(x.id() + 100),
        E::Nothing => {}
    }
    let o = Some(D(92));
    if let Some(d) = o {
        put_i64(d.id());
    } else {
        put_i64(0);
    }
    let mut v = [Some(D(93)), None, Some(D(94))];
    let mut i = 0;
    while let Some(x) = v[i % 3].take() {
        put_i64(x.id());
        i += 2;
    }
    let one = E::One(D(95));
    if let E::One(ref d) = one {
        put_i64(d.id());
    }
    let _nothing = E::Nothing;
}

fn loops() {
    section(10);
    for d in [D(100), D(101), D(102)] {
        if d.id() == 101 {
            continue;
        }
        put_i64(d.id() + 1000);
    }
    let mut n = 0;
    let r = loop {
        let d = D(103 + n);
        n += 1;
        if n == 2 {
            break d;
        }
    };
    put_i64(r.id());
}

fn call<F: FnOnce() -> i64>(f: F) -> i64 {
    f()
}

fn closures() {
    section(11);
    let d = D(110);
    let f = move || d.id();
    put_i64(f());
    put_i64(f());
    drop(f);
    let e = D(111);
    put_i64(call(move || e.id() + 1));
    let g = D(112);
    let h = || g.id();
    put_i64(h());
}

fn glue() {
    section(12);
    let _t = (D(120), 5, D(121));
    let _a = [D(122), D(123)];
    let _e = E::Two(D(124), D(125));
    let mut m = ManuallyDrop::new(D(126));
    put_i64(m.id());
    mem::forget(D(127));
    unsafe {
        ManuallyDrop::drop(&mut m);
    }
    let mut boxed: ManuallyDrop<D> = ManuallyDrop::new(D(128));
    let dynp: &mut dyn Speak = &mut *boxed;
    put_i64(dynp.say());
    unsafe {
        core::ptr::drop_in_place(dynp as *mut dyn Speak);
    }
    let mut x = D(129);
    let old = mem::replace(&mut x, D(130));
    put_i64(old.id());
    put_i64(mem::needs_drop::<D>() as i64);
    put_i64(mem::needs_drop::<(i64, u8)>() as i64);
    put_i64(mem::needs_drop::<ManuallyDrop<D>>() as i64);
    put_i64(mem::needs_drop::<Three>() as i64);
}

#[unsafe(no_mangle)]
pub extern "C" fn main() -> i32 {
    scopes();
    moves(true);
    moves(false);
    put_i64(take(D(20)));
    params(D(21), make(23));
    partial();
    assign();
    temporaries();
    section(7);
    put_i64(early(2));
    put_i64(early(9));
    section(8);
    put_i64(question(81).unwrap_or(-1));
    put_i64(question(0).unwrap_or(-1));
    patterns();
    loops();
    closures();
    glue();
    0
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}
