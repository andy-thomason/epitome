//! An argument is coerced to the raw pointer the expected result wants,
//! its pointee still unknown; checked against `core` from rust-src.
#![no_std]

fn takes<T>(_: Option<*mut T>) {}

fn uses() {
    let mut y = 1u8;
    let mut x = None;
    takes(x);
    // `Some(&mut y)` as `Option<*mut ?T>`: `?T` is `u8`.
    x = Some(&mut y);
}
