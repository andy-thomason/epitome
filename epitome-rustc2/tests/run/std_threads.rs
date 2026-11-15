//! `std` threads with `thread_local!`: each thread has its own copy of a
//! thread local (one with a `const` initialiser, one initialised on first
//! use, one with a destructor), threads are spawned and joined, and
//! values come back from them and through a channel of `Arc<Mutex<_>>`.

use std::cell::{Cell, RefCell};
use std::sync::{Arc, Mutex};
use std::thread;

thread_local! {
    static COUNTER: Cell<u32> = const { Cell::new(1) };
    static NAME: RefCell<String> = RefCell::new(String::from("main"));
}

fn bump(n: u32) -> u32 {
    COUNTER.with(|c| {
        c.set(c.get() + n);
        c.get()
    })
}

fn main() {
    bump(10);
    NAME.with(|n| n.borrow_mut().push_str("-thread"));
    let total = Arc::new(Mutex::new(0u32));
    let mut handles = Vec::new();
    for i in 0..4u32 {
        let total = Arc::clone(&total);
        handles.push(thread::spawn(move || {
            let mine = bump(i + 1);
            NAME.with(|n| *n.borrow_mut() = format!("worker {i}"));
            *total.lock().unwrap() += mine;
            NAME.with(|n| n.borrow().len() as u32) + mine * 100
        }));
    }
    let results: Vec<u32> = handles.into_iter().map(|h| h.join().unwrap()).collect();
    println!("results {results:?}");
    println!("total {}", *total.lock().unwrap());
    println!("main counter {}", COUNTER.with(|c| c.get()));
    NAME.with(|n| println!("main name {}", n.borrow()));
}
