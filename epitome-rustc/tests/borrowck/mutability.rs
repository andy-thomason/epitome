//! Mutability. A line with `//~ E0596` (and so on) is where that error is; no
//! other line has one.
#![no_std]
#![allow(unused)]

struct S {
    x: u32,
}
impl S {
    fn bump(&mut self) {}
    fn peek(&self) -> u32 {
        self.x
    }
}
struct Wrap {
    inner: S,
    r: &'static mut S,
}

// ---- borrowing ----

fn borrow_immutable() {
    let x = 1;
    let r = &mut x; //~ E0596
}

fn borrow_mutable() {
    let mut x = 1;
    let r = &mut x;
    *r = 2;
}

fn method_on_immutable() {
    let s = S { x: 1 };
    s.bump(); //~ E0596
}

fn method_on_mutable() {
    let mut s = S { x: 1 };
    s.bump();
    s.peek();
}

fn method_through_shared(s: &S) {
    s.bump(); //~ E0596
}

fn method_through_mut(s: &mut S) {
    s.bump();
    s.bump();
}

fn param_immutable(s: S) {
    s.bump(); //~ E0596
}

fn param_mutable(mut s: S) {
    s.bump();
}

fn ref_mut_pattern_of_immutable() {
    let t = (1, 2);
    let (ref mut a, _) = t; //~ E0596
}

fn ref_mut_pattern_of_mutable() {
    let mut t = (1, 2);
    let (ref mut a, _) = t;
}

// ---- assigning ----

fn assign_field_of_immutable() {
    let s = S { x: 1 };
    s.x = 2; //~ E0594
}

fn assign_field_of_mutable() {
    let mut s = S { x: 1 };
    s.x = 2;
    s.x += 1;
}

fn assign_through_shared(s: &S) {
    s.x = 1; //~ E0594
}

fn assign_through_mut(s: &mut S) {
    s.x = 1;
    s.x += 1;
}

fn assign_behind_shared_of_mut(s: &&mut S) {
    s.x = 1; //~ E0594
}

fn assign_through_mut_field(w: &mut Wrap) {
    w.r.x = 1;
    w.inner.x = 1;
}

fn assign_through_shared_wrap(w: &Wrap) {
    w.inner.x = 1; //~ E0594
    w.r.x = 1; //~ E0594
}

fn assign_element_of_immutable() {
    let a = [1, 2];
    a[0] = 3; //~ E0594
}

fn assign_element_of_mutable() {
    let mut a = [1, 2];
    a[0] = 3;
}

fn assign_through_raw(p: *const u32, q: *mut u32) {
    unsafe {
        *p = 1; //~ E0594
        *q = 1;
    }
}

fn reference_need_not_be_mut(r: &mut u32) {
    *r = 1;
    *r += 1;
}

// ---- closures ----

fn closure_mutates_immutable() {
    let x = 1;
    let mut c = || x += 1; //~ E0594
}

fn closure_mutates_mutable() {
    let mut x = 1;
    let mut c = || x += 1;
    c();
}

fn closure_called_through_immutable() {
    let mut y = 0;
    let c = || y += 1;
    c(); //~ E0596
}

fn closure_that_only_reads() {
    let y = 0;
    let c = || y + 1;
    c();
    c();
}

// ---- statics and parameters ----

static N: u32 = 0;
static mut M: u32 = 0;

fn assign_static() {
    N = 1; //~ E0594
    unsafe {
        M = 1;
    }
}

fn pattern_parameter((ref mut a, ref b): (u32, u32)) {}

fn plain_parameter(x: u32) {
    let y = &mut x; //~ E0596
}
