//! A kernel for RISC-V: `main` is marked `#[epitome::target("riscv64")]`,
//! so it, `put`, `squares` and the parts of `core` they use (iterators,
//! `sort_unstable`) are compiled for RV64, and run on codegen's RV64
//! machine, which gives `putchar`.
//!
//! run on: riscv64
//! prints: 173
//! prints: 1
//! prints: 1
//! prints: 2
//! prints: 3
//! prints: 4
//! prints: 5
//! prints: 6
//! prints: 9

unsafe extern "C" {
    fn putchar(c: i32) -> i32;
}

fn put(mut n: u64) {
    let mut buf = [0u8; 20];
    let mut i = 0;
    loop {
        buf[i] = b'0' + (n % 10) as u8;
        i += 1;
        n /= 10;
        if n == 0 {
            break;
        }
    }
    while i > 0 {
        i -= 1;
        unsafe { putchar(buf[i] as i32) };
    }
    unsafe { putchar(10) };
}

fn squares(v: &[u64]) -> u64 {
    v.iter().map(|&x| x * x).sum()
}

#[epitome::target("riscv64")]
#[unsafe(no_mangle)]
pub extern "C" fn main() -> i32 {
    let v = [3u64, 1, 4, 1, 5, 9, 2, 6];
    put(squares(&v));
    let mut sorted = v;
    sorted.sort_unstable();
    for x in sorted {
        put(x);
    }
    0
}
