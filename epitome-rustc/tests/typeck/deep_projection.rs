//! An associated type that is another projection is followed through
//! many levels; checked against `core` from rust-src.
#![no_std]

trait Trait {
    fn anyone_can_call(&self);
}

trait HasAssoc {
    type Assoc;
}

struct W<T>(T);

impl<T: HasAssoc> HasAssoc for W<T> {
    type Assoc = T::Assoc;
}

impl HasAssoc for () {
    type Assoc = ();
}

impl Trait for () {
    fn anyone_can_call(&self) {}
}

fn deep() {
    let b: <W<W<W<W<W<W<W<W<W<W<()>>>>>>>>>> as HasAssoc>::Assoc = loop {};
    b.anyone_can_call();
}
