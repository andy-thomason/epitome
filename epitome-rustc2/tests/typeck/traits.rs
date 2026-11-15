//! Type checking with traits; checked against `core` from rust-src.
#![no_std]

use core::fmt::Write;

enum Shape {
    Circle(u32),
    Square(u32),
}

impl Shape {
    fn new(round: bool, size: u32) -> Self {
        if round { Self::Circle(size) } else { Self::Square(size) }
    }
}

/// Trait methods through autoref, operator traits, closures from `Fn`
/// bounds, iterator adapters.
fn total_area(shapes: &[Shape]) -> u64 {
    shapes
        .iter()
        .map(|s| match s {
            Shape::Circle(r) => 3 * r * r,
            Shape::Square(w) => w * w,
        })
        .map(u64::from)
        .sum()
}

/// `impl Trait` arguments, `&mut impl FnMut`, `?` through `Try`.
fn first_even(it: impl Iterator<Item = u32>, seen: &mut impl FnMut(u32)) -> Option<u32> {
    for x in it {
        seen(x);
        if x % 2 == 0 {
            return Some(x);
        }
    }
    None
}

fn parse_pair(a: &str, b: &str) -> Option<(u8, u16)> {
    let x: u8 = a.parse().ok()?;
    let y = b.parse::<u16>().ok()?;
    Some((x, y))
}

/// Generic bounds with associated types.
fn sum_all<I>(i: I) -> I::Item
where
    I: Iterator,
    I::Item: core::ops::Add<Output = I::Item> + Default,
{
    i.fold(Default::default(), |a, b| a + b)
}

/// A `dyn` trait object, `Deref` coercion through `&mut`.
fn shout(w: &mut dyn Write, s: &str) -> core::fmt::Result {
    w.write_str(s)?;
    w.write_char('!')
}

fn zipped(a: [u8; 3], b: &[u16]) -> usize {
    a.iter().zip(b).filter(|(x, y)| u16::from(**x) < **y).count()
}

/// `async` blocks: futures whose output is the block's value (or what a
/// `return` in it gives), awaited on their own and as an operand of
/// `format_args!`.
async fn awaits(w: &mut dyn Write) -> u16 {
    let a = async { 1u8 }.await;
    let b = async {
        if a > 0 {
            return 2;
        }
        3u16
    };
    let _ = w.write_fmt(format_args!("{} {:?}", "", async {}.await));
    b.await
}

/// A closure's parameters from its parameter's `FnOnce` bound, where the
/// call's expected result has an `impl Trait` that the body defines.
struct Staged<F>(F);

impl<F> Staged<F> {
    fn new(f: F) -> Self
    where
        F: FnOnce(Counter) -> Counter,
    {
        Staged(f)
    }
}

trait Stage {}

impl<F: FnOnce(Counter) -> Counter> Stage for F {}

struct Counter(u32);

impl Counter {
    fn bump(&mut self) {
        self.0 += 1;
    }
}

fn staged() -> Staged<impl Stage> {
    Staged::new(|mut c| {
        c.bump();
        c
    })
}
