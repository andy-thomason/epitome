//! An impl's `where` clauses settle each other whichever comes first;
//! checked against `core` from rust-src.
#![no_std]

use core::future::{ready, Future};

// `Fut` is named only by the `where` clauses: `F: FnMut() -> Fut` gives it,
// and then `Fut: Future<Output = R>` gives `R`. Written either way round.
trait Before<R> {}
impl<F, R, Fut> Before<R> for F
where
    Fut: Future<Output = R>,
    F: FnMut() -> Fut,
{
}

trait After<R> {}
impl<F, R, Fut> After<R> for F
where
    F: FnMut() -> Fut,
    Fut: Future<Output = R>,
{
}

fn before<R, F: Before<R>>(_f: F) -> Option<R> {
    None
}

fn after<R, F: After<R>>(_f: F) -> Option<R> {
    None
}

fn uses() {
    before(|| ready(1usize));
    after(|| ready(1usize));
}
