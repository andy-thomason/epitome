//! A host program that launches RISC-V kernels: `squares` and `scale` are
//! marked `#[epitome::target("riscv64")]`, so the host's calls of them run
//! them on the RV64 interpreter compiled into the program
//! (`runtime/rv64.rs`), with integer and float arguments and results, and
//! slices, `&mut` ones changed in place, and a `&str`.
//!
//! run on: host
//! prints: on the host: 385
//! prints: on RISC-V: 385
//! prints: scaled: -7.5
//! prints: mixed: 7 1 true
//! prints: dot: 70
//! prints: scaled in place: [2, 4, 6, 8, 10]
//! prints: vowels: 4

#[epitome::target("riscv64")]
pub fn squares(n: u64) -> u64 {
    (1..=n).map(|i| i * i).sum()
}

#[epitome::target("riscv64")]
pub fn scale(x: f64, k: i32) -> f64 {
    x * k as f64
}

#[epitome::target("riscv64")]
pub fn gcd(mut a: u32, mut b: u32) -> u32 {
    while b != 0 {
        let t = a % b;
        a = b;
        b = t;
    }
    a
}

#[epitome::target("riscv64")]
pub fn small(x: i8, y: f32) -> bool {
    (x as f32) < y
}

#[epitome::target("riscv64")]
pub fn dot(a: &[i64], b: &[i64]) -> i64 {
    let mut s = 0;
    for i in 0..a.len().min(b.len()) {
        s += a[i] * b[i];
    }
    s
}

#[epitome::target("riscv64")]
pub fn double(v: &mut [u32]) {
    for x in v.iter_mut() {
        *x *= 2;
    }
}

#[epitome::target("riscv64")]
pub fn vowels(s: &str) -> u64 {
    s.bytes().filter(|b| matches!(b, b'a' | b'e' | b'i' | b'o' | b'u')).count() as u64
}

fn main() {
    println!("on the host: {}", (1..=10u64).map(|i| i * i).sum::<u64>());
    println!("on RISC-V: {}", squares(10));
    println!("scaled: {}", scale(2.5, -3));
    println!("mixed: {} {} {}", gcd(91, 49), gcd(17, 5), small(-3, -2.5));
    println!("dot: {}", dot(&[1, 2, 3, 4], &[5, 6, 7, 8]));
    let mut v = vec![1u32, 2, 3, 4, 5];
    double(&mut v);
    println!("scaled in place: {v:?}");
    println!("vowels: {}", vowels("kernel launch"));
}
