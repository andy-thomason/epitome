//! `#[thread_local]` statics: each thread (made with `pthread_create`)
//! has its own copy, starting as the initialiser: zero, a value a `const
//! fn` builds (`Cell::new(7)`), a tuple, and an address (`Cell::new("main")`).
//! Threads run at once and change their copies; the main thread's are
//! left as they were. Also a plain static whose value holds addresses (a
//! table of strings), which is constant data.
#![no_std]
#![no_main]
#![feature(thread_local)]
#![allow(dead_code)]

use core::cell::Cell;

unsafe extern "C" {
    fn putchar(c: i32) -> i32;
    fn pthread_create(t: *mut usize, attr: *const u8, f: extern "C" fn(*mut u8) -> *mut u8, arg: *mut u8) -> i32;
    fn pthread_join(t: usize, ret: *mut *mut u8) -> i32;
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

fn put_str(s: &str) {
    for b in s.bytes() {
        unsafe {
            putchar(b as i32);
        }
    }
    unsafe {
        putchar(10);
    }
}

#[thread_local]
static mut COUNT: u64 = 0;

#[thread_local]
static CELL: Cell<u32> = Cell::new(7);

#[thread_local]
static PAIR: (Cell<i64>, Cell<i64>) = (Cell::new(-1), Cell::new(100));

#[thread_local]
static NAME: Cell<&'static str> = Cell::new("main");

static NAMES: [&str; 3] = ["one", "two", "three"];

/// What each thread saw, by its number.
static mut SEEN: [[i64; 6]; 3] = [[0; 6]; 3];

fn work(i: usize) {
    unsafe {
        let seen = &mut *(&raw mut SEEN);
        // As each thread starts: the initial values.
        seen[i][0] = COUNT as i64;
        seen[i][1] = CELL.get() as i64;
        seen[i][2] = PAIR.0.get() + PAIR.1.get();
        seen[i][3] = NAME.get().len() as i64;
        for _ in 0..=i {
            COUNT += 10;
        }
        CELL.set(CELL.get() * (i as u32 + 2));
        PAIR.0.set(i as i64);
        NAME.set(NAMES[i]);
        seen[i][4] = COUNT as i64 + CELL.get() as i64;
        seen[i][5] = NAME.get().len() as i64 + PAIR.0.get() * 100;
    }
}

extern "C" fn thread(arg: *mut u8) -> *mut u8 {
    work(arg as usize);
    core::ptr::null_mut()
}

#[unsafe(no_mangle)]
pub extern "C" fn main() -> i32 {
    unsafe {
        COUNT = 5;
    }
    CELL.set(1000);
    NAME.set("changed");
    let mut ts = [0usize; 3];
    for (i, t) in ts.iter_mut().enumerate() {
        unsafe {
            pthread_create(t, core::ptr::null(), thread, i as *mut u8);
        }
    }
    for &t in &ts {
        unsafe {
            pthread_join(t, core::ptr::null_mut());
        }
    }
    for row in unsafe { &*(&raw const SEEN) } {
        for &v in row {
            put_i64(v);
        }
    }
    // The main thread's own copies, as it left them.
    put_i64(unsafe { COUNT } as i64);
    put_i64(CELL.get() as i64);
    put_i64(PAIR.0.get() + PAIR.1.get());
    put_str(NAME.get());
    for n in &NAMES {
        put_str(n);
    }
    0
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}
