//! `Box` and allocation through `alloc`, with a global allocator over the
//! C library's `malloc`: boxes made, moved, dropped and moved out of;
//! `Box<dyn Trait>`; a recursive list; `Vec` growing and shrinking;
//! `String`; and the allocator's calls, counted.
#![no_std]
#![no_main]
#![allow(dead_code)]

extern crate alloc;

use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;
use core::alloc::{GlobalAlloc, Layout};
use core::sync::atomic::{AtomicUsize, Ordering};

unsafe extern "C" {
    fn putchar(c: i32) -> i32;
    fn malloc(size: usize) -> *mut u8;
    fn calloc(n: usize, size: usize) -> *mut u8;
    fn realloc(p: *mut u8, size: usize) -> *mut u8;
    fn free(p: *mut u8);
}

/// `malloc`, counting what is live.
struct Libc;

static LIVE: AtomicUsize = AtomicUsize::new(0);
static CALLS: AtomicUsize = AtomicUsize::new(0);

unsafe impl GlobalAlloc for Libc {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        LIVE.fetch_add(1, Ordering::Relaxed);
        CALLS.fetch_add(1, Ordering::Relaxed);
        unsafe { malloc(layout.size()) }
    }
    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        LIVE.fetch_add(1, Ordering::Relaxed);
        CALLS.fetch_add(1, Ordering::Relaxed);
        unsafe { calloc(1, layout.size()) }
    }
    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        LIVE.fetch_sub(1, Ordering::Relaxed);
        unsafe { free(ptr) }
    }
    unsafe fn realloc(&self, ptr: *mut u8, _layout: Layout, new_size: usize) -> *mut u8 {
        CALLS.fetch_add(1, Ordering::Relaxed);
        unsafe { realloc(ptr, new_size) }
    }
}

#[global_allocator]
static A: Libc = Libc;

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

fn section(n: i64) {
    put_i64(-n);
    put_i64(LIVE.load(Ordering::Relaxed) as i64);
}

/// Says when it is dropped.
struct D(i64);

impl Drop for D {
    fn drop(&mut self) {
        put_i64(self.0);
    }
}

trait Shape {
    fn area(&self) -> i64;
}

struct Sq(i64, D);
struct Rect(i64, i64);

impl Shape for Sq {
    fn area(&self) -> i64 {
        self.0 * self.0
    }
}

impl Shape for Rect {
    fn area(&self) -> i64 {
        self.0 * self.1
    }
}

enum List {
    Cons(i64, Box<List>),
    Nil,
}

fn sum(l: &List) -> i64 {
    match l {
        List::Cons(v, rest) => v + sum(rest),
        List::Nil => 0,
    }
}

fn boxes() {
    section(1);
    let b = Box::new(41i64);
    put_i64(*b + 1);
    let d = Box::new(D(10));
    put_i64(d.0);
    let e = d;
    put_i64(e.0 + 1);
    let inner = *Box::new(D(11));
    put_i64(inner.0);
    let big = Box::new([7u64; 64]);
    put_i64(big[63] as i64);
    section(1);
}

fn dyns() {
    section(2);
    let shapes: [Box<dyn Shape>; 2] = [Box::new(Sq(3, D(20))), Box::new(Rect(2, 5))];
    for s in &shapes {
        put_i64(s.area());
    }
    let one: Box<dyn Shape> = Box::new(Sq(4, D(21)));
    put_i64(one.area());
    drop(one);
    section(2);
}

fn lists() {
    section(3);
    let mut l = List::Nil;
    for i in 1..=5 {
        l = List::Cons(i, Box::new(l));
    }
    put_i64(sum(&l));
    section(3);
}

fn vecs() {
    section(4);
    let mut v = Vec::new();
    for i in 0..100 {
        v.push(i * 2);
    }
    put_i64(v.len() as i64);
    put_i64(v[99]);
    put_i64(v.iter().sum::<i64>());
    v.truncate(10);
    v.shrink_to_fit();
    put_i64(v.capacity() as i64);
    let mut ds = Vec::new();
    ds.push(D(40));
    ds.push(D(41));
    ds.push(D(42));
    let last = ds.pop();
    put_i64(ds.len() as i64);
    drop(last);
    let zeros: Vec<u8> = alloc::vec![0; 1000];
    put_i64(zeros.len() as i64);
    section(4);
}

fn strings() {
    section(5);
    let mut s = String::new();
    s.push_str("hello");
    s.push(',');
    s.push_str(" world");
    put_str(&s);
    put_i64(s.len() as i64);
    let t = String::from("boxed");
    let u = t.clone() + " again";
    put_str(&u);
    put_str(&t);
    section(5);
}

#[unsafe(no_mangle)]
pub extern "C" fn main() -> i32 {
    boxes();
    dyns();
    lists();
    vecs();
    strings();
    section(6);
    put_i64((CALLS.load(Ordering::Relaxed) > 0) as i64);
    0
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}
