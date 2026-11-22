//! A panic with `std`: an index out of bounds, after some output. The
//! panic hook prints the message and where it happened to stderr, and the
//! process aborts. Before it, a `RefCell` borrowed twice and a division by
//! zero are checked, and `catch_unwind` runs its closure.

use std::cell::RefCell;

fn total(v: &[u64], upto: usize) -> u64 {
    v[..upto].iter().sum()
}

fn main() {
    let v: Vec<u64> = (1..=5).collect();
    println!("sum of 3: {}", total(&v, 3));
    let c = RefCell::new(1);
    {
        let _a = c.borrow();
        println!("second borrow ok: {}", c.try_borrow_mut().is_err());
    }
    let d = std::hint::black_box(4u64);
    println!("20 / d = {}", 20 / d);
    let r = std::panic::catch_unwind(|| 7);
    println!("caught: {:?}", r.ok());
    let n = std::hint::black_box(9);
    println!("sum of {n}: {}", total(&v, n));
    println!("not reached");
}
