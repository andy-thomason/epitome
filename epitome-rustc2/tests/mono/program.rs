//! A program for monomorphisation: generic functions, trait methods,
//! operators, iterators, closures, `impl Trait` (as a parameter and as a
//! result), `?`, and a trait object.
#![no_std]

use core::fmt::Write;

pub trait Area {
    fn area(&self) -> u64;
    fn twice(&self) -> u64 { 2 * self.area() }
}

pub struct Square(pub u32);
pub struct Rect { pub w: u32, pub h: u32 }

impl Area for Square {
    fn area(&self) -> u64 { u64::from(self.0) * u64::from(self.0) }
}
impl Area for Rect {
    fn area(&self) -> u64 { u64::from(self.w) * u64::from(self.h) }
}

fn total<T: Area>(xs: &[T]) -> u64 {
    xs.iter().map(|x| x.twice()).sum()
}

fn largest(shapes: &[&dyn Area]) -> u64 {
    let mut best = 0;
    for s in shapes {
        best = best.max(s.area());
    }
    best
}

fn parse(s: &str) -> Option<u32> {
    let n: u32 = s.parse().ok()?;
    Some(n + 1)
}

fn apply(f: impl Fn(u64) -> u64, x: u64) -> u64 {
    f(x)
}

fn adder(n: u64) -> impl Fn(u64) -> u64 {
    move |x| x + n
}

struct Sink;
impl Write for Sink {
    fn write_str(&mut self, _: &str) -> core::fmt::Result { Ok(()) }
}

pub fn main() -> u64 {
    let squares = [Square(2), Square(3)];
    let r = Rect { w: 4, h: 5 };
    let dyns: [&dyn Area; 2] = [&squares[0], &r];
    let mut sink = Sink;
    let _ = sink.write_str("hi");
    total(&squares) + largest(&dyns) + u64::from(parse("41").unwrap_or(0)) + apply(adder(1), 2)
}
