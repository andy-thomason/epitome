//! A coinductive cycle in an impl's `where` clauses settles what is
//! unknown in it; checked against `core` from rust-src.
#![no_std]
#![feature(rustc_attrs)]

#[rustc_coinductive]
trait Trait {}

// `(T, U): Trait` needs `(U, T): Trait`, which needs it back, and
// `(): ConstrainToU32<T>` gives `T`: then the cycle gives `U` the same.
impl<T, U> Trait for (T, U)
where
    (U, T): Trait,
    (): ConstrainToU32<T>,
{
}

trait ConstrainToU32<T> {}
impl ConstrainToU32<u32> for () {}

fn impls_trait<T, U>()
where
    (T, U): Trait,
{
}

fn uses() {
    impls_trait::<_, _>();
}
