//! Moves and initialisation. A line with `//~ E0382` (and so on) is where that
//! error is; no other line has one.
#![no_std]
#![allow(unused)]

struct S(u32);
struct Pair(S, S);
struct D(u32);
impl Drop for D {
    fn drop(&mut self) {}
}
fn take(_: S) {}
fn read(_: &S) {}

// ---- moves ----

fn use_after_move() {
    let a = S(1);
    take(a);
    take(a); //~ E0382
}

fn borrow_after_move() {
    let a = S(1);
    take(a);
    read(&a); //~ E0382
}

fn partly_moved() {
    let t = Pair(S(1), S(2));
    take(t.0);
    take(t.1);
}

fn partly_moved_then_whole() {
    let t = Pair(S(1), S(2));
    take(t.0);
    let u = t; //~ E0382
}

fn moved_in_a_loop() {
    let a = S(1);
    loop {
        take(a); //~ E0382
    }
}

fn moved_in_one_branch(c: bool) {
    let a = S(1);
    if c {
        take(a);
    }
    take(a); //~ E0382
}

fn moved_in_both_branches(c: bool) {
    let a = S(1);
    if c {
        take(a);
    } else {
        take(S(2));
    }
}

fn reinitialised() {
    let mut a = S(1);
    take(a);
    a = S(2);
    take(a);
}

fn moved_into_a_closure() {
    let a = S(1);
    let c = move || take(a);
    take(a); //~ E0382
}

fn copies_are_free(x: u32, y: (u32, u32)) -> u32 {
    x + x + y.0 + y.0
}

fn reference_is_copy(r: &S) {
    read(r);
    read(r);
}

fn mut_reference_is_reborrowed(r: &mut S) {
    r.0 += 1;
    r.0 += 1;
}

fn generic_moves<T>(t: T) {
    let a = t;
    let b = t; //~ E0382
}

fn generic_copy<T: Copy>(t: T) {
    let a = t;
    let b = t;
}

// ---- conditions ----

fn short_circuit() {
    let z;
    true && { z = 3; true } && z == 3;
}

fn guard_does_not_move(o: Option<S>) {
    match o {
        Some(s) if s.0 > 3 => take(s),
        Some(s) => take(s),
        None => {}
    }
}

// ---- initialisation ----

fn uninitialised() -> u32 {
    let x: u32;
    x //~ E0381
}

fn initialised_in_branches(c: bool) -> u32 {
    let x: u32;
    if c {
        x = 1;
    } else {
        x = 2;
    }
    x
}

fn initialised_in_one_branch(c: bool) -> u32 {
    let x: u32;
    if c {
        x = 1;
    }
    x //~ E0381
}

fn assigned_twice() {
    let x = 1;
    x = 2; //~ E0384
}

fn assigned_twice_in_a_loop() {
    let x;
    loop {
        x = 1; //~ E0384
    }
}

fn assigned_once_in_a_branch(c: bool) {
    let x;
    if c {
        x = 1;
    }
}

// ---- what cannot be moved out of ----

fn out_of_a_reference(r: &S) {
    take(*r); //~ E0507
}

fn out_of_a_box(b: &mut [S; 1], i: usize) {
    take(b[i]); //~ E0508
}

fn out_of_a_drop_type(d: D) -> u32 {
    let D(n) = d;
    n
}

fn unreachable_after_never() {
    let x: u32;
    loop {}
    x;
}
