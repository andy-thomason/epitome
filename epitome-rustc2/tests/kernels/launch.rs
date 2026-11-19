//! A host program that launches RISC-V kernels: `squares` and `scale` are
//! marked `#[epitome::target("riscv64")]`, so the host's calls of them run
//! them on the RV64 interpreter compiled into the program
//! (`runtime/rv64.rs`), with integer and float arguments and results.
//!
//! run on: host
//! prints: on the host: 385
//! prints: on RISC-V: 385
//! prints: scaled: -7.5
//! prints: mixed: 7 1 true

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

fn main() {
    println!("on the host: {}", (1..=10u64).map(|i| i * i).sum::<u64>());
    println!("on RISC-V: {}", squares(10));
    println!("scaled: {}", scale(2.5, -3));
    println!("mixed: {} {} {}", gcd(91, 49), gcd(17, 5), small(-3, -2.5));
}
