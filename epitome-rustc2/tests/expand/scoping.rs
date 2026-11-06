#![no_std]
// Text-order scoping, `#[macro_use]`, `#[macro_export]` and paths.

#[macro_use]
mod defs {
    macro_rules! twice {
        ($e:expr) => { $e + $e };
    }
    #[macro_export]
    macro_rules! exported {
        () => { 42 };
    }
}

pub fn a() -> u32 {
    twice!(3)
}

pub fn b() -> u32 {
    crate::exported!() + exported!()
}

// A macro that defines a macro.
macro_rules! make_getter {
    ($name:ident, $value:expr) => {
        macro_rules! $name {
            () => { $value };
        }
    };
}

make_getter!(answer, 6 * 7);

pub fn c() -> u32 {
    answer!()
}

// Shadowing: the later definition wins after it.
macro_rules! which { () => { 1 }; }
pub fn d() -> u32 { which!() }
macro_rules! which { () => { 2 }; }
pub fn e() -> u32 { which!() }

pub trait T {
    fn f(&self) -> u32;
}

macro_rules! impl_t {
    ($($ty:ty => $v:expr);* $(;)?) => {
        $(impl T for $ty { fn f(&self) -> u32 { $v } })*
    };
}

impl_t! {
    u8 => 1;
    u16 => twice!(2);
}
