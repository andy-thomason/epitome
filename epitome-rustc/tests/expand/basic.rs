#![no_std]
// Rules, repetition, recursion, and expansion in every position.

macro_rules! square {
    ($x:expr) => {
        $x * $x
    };
}

macro_rules! max {
    ($x:expr) => { $x };
    ($x:expr, $($rest:expr),+) => {
        if $x > max!($($rest),+) { $x } else { max!($($rest),+) }
    };
}

macro_rules! make_fns {
    ($($name:ident => $value:expr),* $(,)?) => {
        $(pub fn $name() -> u32 { $value })*
    };
}

make_fns! {
    one => 1,
    two => square!(1 + 1),
}

macro_rules! point {
    ($t:ty) => { (u8, $t) };
}

macro_rules! pair {
    ($a:pat, $b:pat) => { ($a, $b) };
}

pub fn uses(p: point!(u16)) -> u32 {
    let pair!(a, _) = p;
    let m = max!(1, 2, 3);
    square!(a as u32 + 1) + m
}

mod inner {
    macro_rules! local {
        () => { 7 };
    }
    pub fn seven() -> u32 {
        local!()
    }
}

pub fn statements() -> u32 {
    macro_rules! decl {
        ($n:ident) => { let $n = 5; };
    }
    decl!(x);
    x + inner::seven()
}
