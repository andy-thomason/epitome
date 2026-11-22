//! A closure's parameter type comes from the bound that does not name the
//! closure's own type, whichever order they are in; checked against
//! `core` from rust-src.
#![no_std]

trait Trait<'a> {
    type Assoc;
}
impl<'a, F> Trait<'a> for F {
    type Assoc = u32;
}

fn first<F>(_: F)
where
    F: Fn(u32) + for<'a> Fn(<F as Trait<'a>>::Assoc),
{
}

fn second<F>(_: F)
where
    F: for<'a> Fn(<F as Trait<'a>>::Assoc) + Fn(u32),
{
}

fn uses() {
    first(move |x| {
        let _: u32 = x.into();
    });
    second(move |x| {
        let _: u32 = x.into();
    });
}
