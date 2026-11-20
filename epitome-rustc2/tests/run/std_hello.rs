//! A program with `std`: `println!` and `format!`, `Vec`, `String`,
//! iterators, `Box<dyn Trait>`, drops, `std::env::args` and the exit
//! status, built with `std` from rust-src.

use std::fmt;

struct Point {
    x: i32,
    y: i32,
}

impl fmt::Display for Point {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

trait Shape {
    fn area(&self) -> f64;
    fn name(&self) -> String;
}

struct Circle(f64);
struct Square(f64);

impl Shape for Circle {
    fn area(&self) -> f64 {
        3.0 * self.0 * self.0
    }
    fn name(&self) -> String {
        String::from("circle")
    }
}

impl Shape for Square {
    fn area(&self) -> f64 {
        self.0 * self.0
    }
    fn name(&self) -> String {
        "square".to_string()
    }
}

struct Noisy(&'static str);

impl Drop for Noisy {
    fn drop(&mut self) {
        println!("drop {}", self.0);
    }
}

fn main() {
    println!("hello, world");
    println!("{} + {} = {}", 2, 3, 2 + 3);
    let p = Point { x: 3, y: -4 };
    println!("point {p}, width {:>6}|", p.x);
    let q = Point { x: 3, y: -4 };
    let (a, b) = (p.x, q.y);
    println!("{} {}", a == b, a == q.x);
    let v: Vec<i32> = (1..=5).collect();
    println!("{:?} sum {}", v, v.iter().sum::<i32>());
    let doubled: Vec<i32> = v.iter().map(|x| x * 2).filter(|x| x % 3 != 0).collect();
    println!("{doubled:?}");
    let s = format!("{}-{}-{:.2}", "a", 42, 1.5);
    println!("{s} has {} bytes", s.len());
    let shapes: Vec<Box<dyn Shape>> = vec![Box::new(Circle(1.0)), Box::new(Square(2.0))];
    for sh in &shapes {
        println!("{} {}", sh.name(), sh.area());
    }
    let words = vec!["one", "two", "three"];
    let joined = words.join(", ");
    println!("{joined} ({} words)", words.len());
    let _a = Noisy("a");
    {
        let _b = Noisy("b");
    }
    println!("args: {}", std::env::args().count());
}
