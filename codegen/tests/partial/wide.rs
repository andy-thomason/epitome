//! Loops over `usize` for `tests/partial.rs`, compiled by rustc for a
//! 64-bit target, so the counters are i64: Epitome narrows them for a
//! machine that lacks i64.
#![no_std]

#[no_mangle]
pub fn fib_usize(n: u32) -> u32 {
    let (mut a, mut b) = (0u32, 1u32);
    for _ in 0..n as usize {
        let t = a.wrapping_add(b);
        a = b;
        b = t;
    }
    a
}

/// (A plain sum would be worked out in closed form, with a 33-bit
/// product that cannot be narrowed.)
#[no_mangle]
pub fn mix_down(n: u32) -> u32 {
    let mut s = 0u32;
    let mut i = n as usize;
    while i != 0 {
        s = s.rotate_left(3) ^ i as u32;
        i -= 1;
    }
    s
}
