//! Bodies of numbers, references and control flow: their control-flow graphs
//! show as Rust that `rustc` takes (checked in `borrowck::tests`).
#![no_std]
#![allow(dead_code)]

pub fn add(a: u32, b: u32) -> u32 {
    a + b
}

pub fn abs(x: i32) -> i32 {
    if x < 0 { -x } else { x }
}

pub fn sum(n: u32) -> u32 {
    let mut total = 0;
    let mut i = 0;
    while i < n {
        total += i;
        i += 1;
    }
    total
}

pub fn first_big(limit: u32) -> u32 {
    let mut i = 1;
    loop {
        if i * i > limit {
            break i;
        }
        i += 1;
    }
}

pub fn labelled(n: u32) -> u32 {
    let mut count = 0;
    let mut i = 0;
    'outer: while i < n {
        let mut j = 0;
        while j < n {
            if j > i {
                continue 'outer;
            }
            if i + j > 100 {
                break 'outer;
            }
            count += 1;
            j += 1;
        }
        i += 1;
    }
    count
}

pub fn class(n: u32) -> u32 {
    match n {
        0 => 1,
        1..=9 => 2,
        10 | 20 | 30 => 3,
        _ => 4,
    }
}

pub fn clamp(x: i32, lo: i32, hi: i32) -> i32 {
    if x < lo {
        lo
    } else if x > hi {
        hi
    } else {
        x
    }
}

pub fn swap(a: &mut u32, b: &mut u32) {
    let t = *a;
    *a = *b;
    *b = t;
}

pub fn array_sum(a: [u32; 4]) -> u32 {
    let mut s = 0;
    let mut i = 0;
    while i < 4 {
        s += a[i];
        i += 1;
    }
    s
}

pub fn pair(a: u32, b: u32) -> u32 {
    let p = (a, b);
    let (x, y) = p;
    x + y + p.0
}

pub fn both(a: bool, b: bool) -> bool {
    (a && b) || !a
}

pub fn shadow(x: u32) -> u32 {
    let x = x + 1;
    let x = x * 2;
    x
}
