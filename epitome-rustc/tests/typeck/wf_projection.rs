//! What a written projection requires of its arguments can guide
//! inference; checked against `core` from rust-src.
#![no_std]

pub trait EqualTo {
    type Ty;
}
impl<X> EqualTo for X {
    type Ty = X;
}

trait MyTrait<U: EqualTo<Ty = Self>> {
    type Out;
}
impl<T, U: EqualTo<Ty = T>> MyTrait<U> for T {
    type Out = ();
}

fn uses() {
    // `u8: EqualTo<Ty = _>` makes `_` `u8`, then `Out` is `()`.
    let _: <_ as MyTrait<u8>>::Out;
}
