#![no_std]
// A tt muncher, nested repetition, and expressions as operands.

macro_rules! reverse {
    (@acc [$($acc:tt)*]) => { [$($acc),*] };
    (@acc [$($acc:tt)*] $x:tt $($rest:tt)*) => { reverse!(@acc [$x $($acc)*] $($rest)*) };
    ($($x:tt)*) => { reverse!(@acc [] $($x)*) };
}

pub fn r() -> [u32; 3] {
    reverse!(1 2 3)
}

macro_rules! matrix {
    ($([$($x:expr),*]),*) => { [$([$($x * 10),*]),*] };
}

pub fn m() -> [[u32; 2]; 2] {
    matrix!([1 + 1, 2], [3, 4 - 1])
}

macro_rules! neg {
    ($e:expr) => { -$e };
}

macro_rules! call {
    ($f:expr, $($a:expr),*) => { $f($($a),*) };
}

pub fn ops(x: i32) -> i32 {
    let a = neg!(x - 1);
    let b = call!(|p: i32, q: i32| p * q, a, 2);
    neg!(a) * b + neg!(1 + 2) as i32
}
