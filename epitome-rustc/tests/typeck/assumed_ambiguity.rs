//! Of several bounds the body assumes, an unknown in a projection's trait
//! is not taken from the first; checked against `core` from rust-src.
#![no_std]

struct Alloc;
struct S;
struct D1;
struct D2;

trait Al<R> {
    type Buf;
}
impl Al<D1> for Alloc {
    type Buf = u8;
}
impl Al<D2> for Alloc {
    type Buf = u16;
}

// The method's own result names `Al<G>` for a `G` only the impl knows.
trait Fe<G> {
    fn go(&self) -> <Alloc as Al<G>>::Buf;
}
impl Fe<D2> for S {
    fn go(&self) -> u16 {
        0
    }
}

// `Alloc: Al<D1>` is assumed first, but `go` is `Fe<D2>`'s.
fn uses()
where
    Alloc: Al<D1> + Al<D2>,
{
    let _ = S.go();
}

// An assumed bound that does not bind `Buf` leaves it to the impl.
fn through_impl()
where
    Alloc: Al<D1>,
{
    let b: <Alloc as Al<D1>>::Buf = 7u8;
    let _ = b;
}
