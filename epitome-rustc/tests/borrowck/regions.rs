// Signatures whose regions `regions::sig` reads.

pub trait Show {}
pub struct Wrap<'a, T>(&'a T);
pub struct Boxed<T: ?Sized>(*const T);
pub type Pair<'a, T> = Wrap<'a, T>;
pub struct Holder<'a>(&'a u8);

pub fn explicit<'a, 'b>(x: &'a u8, y: &'b u8) -> &'a u8 { x }
pub fn elided_one(x: &u8) -> &u8 { x }
pub fn elided_adt(x: Wrap<'_, u8>) -> &u8 { x.0 }
pub fn hidden_lifetime(x: Wrap<u8>) -> &u8 { x.0 }
pub fn early_bound<'a: 'b, 'b>(x: &'a u8, y: &'b u8) -> &'b u8 { x }
pub fn static_out(x: &u8) -> &'static u8 { &0 }
pub fn dyn_default<'a>(x: &'a dyn Show) -> &'a u8 { loop {} }
pub fn boxed(x: Boxed<dyn Show>) -> u8 { 0 }
pub fn alias<'a>(x: Pair<'a, u8>) -> &'a u8 { x.0 }
pub fn ty_bound<'a, T: 'a>(x: &'a T) -> &'a T { x }
pub fn fn_ptr(f: fn(&u8) -> &u8) {}
pub fn higher_ranked<F>(f: F) where F: for<'x> Fn(&'x u8) -> &'x u8 {}
pub fn two_inputs(x: &u8, y: &u8) -> u8 { 0 }
pub fn outlives_where<'a, 'b>(x: &'a u8, y: &'b u8) -> &'a u8 where 'b: 'a { x }

pub struct S;
impl S {
    pub fn by_ref(&self, x: &u8) -> &u8 { x }
    pub fn by_mut(&mut self) -> &mut S { self }
    pub fn chosen<'a>(&self, x: &'a u8) -> &'a u8 { x }
}
impl<'h> Holder<'h> {
    pub fn get(&self) -> &'h u8 { self.0 }
    pub fn me(&self) -> Holder<'_> { Holder(self.0) }
    pub fn this(self) -> Self { self }
}
pub trait Fn<Args> { type Output; }
