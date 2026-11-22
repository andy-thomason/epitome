//! A method of the receiver itself is found even when the deref chain
//! would settle an unknown another way; checked against `core` from
//! rust-src.
#![no_std]

use core::ops::Deref;

struct Foo<T, U>(T, U);

// `Foo<?T, _>` derefs to `U` if `?T` is `u32`...
impl<U> Deref for Foo<u32, U> {
    type Target = U;
    fn deref(&self) -> &U {
        &self.1
    }
}

// ...but `method` wants `i32`, and the receiver is not derefed.
impl Foo<i32, i32> {
    fn method(&self) {}
}

fn uses() {
    let x = Foo(Default::default(), 1i32);
    x.method();
}

fn through_opaque() -> impl Sized {
    if false {
        let x = Foo(Default::default(), through_opaque());
        x.method();
    }
    1i32
}
