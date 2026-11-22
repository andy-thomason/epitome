//! An associated const's binding in a bound settles a const argument;
//! checked against `core` from rust-src.
#![no_std]
#![feature(min_generic_const_args, generic_const_items)]
#![allow(incomplete_features)]

trait Foo {
    type const ASSOC<const N: u32>: u32;
}

impl Foo for () {
    type const ASSOC<const N: u32>: u32 = N;
}

fn bar<const N: u32, T: Foo<ASSOC<N> = 10>>() {}

fn uses() {
    // `<() as Foo>::ASSOC<N>` is `N`, and the bound says it is 10.
    bar::<_, ()>();
}
