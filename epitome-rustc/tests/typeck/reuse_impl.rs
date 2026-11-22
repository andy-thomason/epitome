//! `reuse impl Tr for T { .. }` is an impl of the trait; checked against
//! `core` from rust-src.
#![no_std]
#![feature(fn_delegation)]

trait Tr {
    fn foo(&self) -> u8 {
        0
    }
    fn bar(&self, x: u8);
}

struct S;
impl Tr for S {
    fn bar(&self, _: u8) {}
}

struct F(S);
reuse impl Tr for F { &self.0 }

fn uses() {
    let f = F(S);
    f.foo();
    f.bar(1);
}
