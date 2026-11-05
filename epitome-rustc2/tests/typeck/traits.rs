#![no_std]
#![feature(no_core)]
enum E { A, B(u8) }
impl E {
    fn mk(x: u8) -> Self { if x == 0 { Self::A } else { Self::B(x) } }
}
struct W(u32);
impl W { fn get(&self) -> u32 { self.0 } }
fn q() -> u32 { <W>::get(&W(3)) }

fn digits(s: &[u8], mut f: impl FnMut(u8)) { for &c in s { f(c - b'0'); } }
fn use_digits() -> u64 { let mut x = 0u64; digits(b"12", |d| { x = x * 10 + d as u64; }); x }

fn pred<T>(p: &mut impl FnMut(&T) -> bool, t: T) -> bool { p(&t) }

fn cmp_them<T: Ord>(a: T, b: T) -> core::cmp::Ordering { a.cmp(&b) }
fn hash_it<T: core::hash::Hash, H: core::hash::Hasher>(t: &T, h: &mut H) { t.hash(h) }

struct Holder<F> { f: F }
impl<F: FnMut(u8) -> u16> Holder<F> { fn call(&mut self, x: u8) -> u16 { (self.f)(x) } }

fn sum_iter<I: Iterator<Item = u32>>(i: I) -> u32 { let mut t = 0; for x in i { t += x; } t }
fn maps(v: &[u32]) -> u32 { v.iter().map(|x| x + 1).sum() }
fn opt(o: Option<u8>) -> Option<u16> { let x = o?; Some(x as u16) }
