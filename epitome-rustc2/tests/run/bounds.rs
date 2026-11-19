//! Indexing in loops whose own counter keeps the index in range: the
//! optimiser removes those bounds checks (`opt/src/implied.rs`). Slices of
//! every length from empty up, so each loop runs zero, one and many times,
//! and checks that must stay.
#![no_std]
#![no_main]

unsafe extern "C" {
    fn putchar(c: i32) -> i32;
}

fn put_u64(mut n: u64) {
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
        unsafe {
            putchar(buf[i] as i32);
        }
    }
    unsafe {
        putchar(10);
    }
}

fn put_bytes(v: &[u8]) {
    for &b in v {
        unsafe {
            putchar(b as i32);
        }
    }
    unsafe {
        putchar(10);
    }
}

#[inline(never)]
fn sum(v: &[u64]) -> u64 {
    let mut s = 0;
    for i in 0..v.len() {
        s += v[i];
    }
    s
}

#[inline(never)]
fn dot(a: &[i64], b: &[i64]) -> i64 {
    let mut s = 0;
    for i in 0..a.len().min(b.len()) {
        s += a[i] * b[i];
    }
    s
}

#[inline(never)]
fn rev(v: &mut [u8]) {
    let n = v.len();
    for i in 0..n / 2 {
        v.swap(i, n - 1 - i);
    }
}

/// Counts to `n`, not the length: the check stays (and passes).
#[inline(never)]
fn prefix(v: &[u64], n: usize) -> u64 {
    let mut s = 0;
    for i in 0..n {
        s += v[i] * (i as u64 + 1);
    }
    s
}

/// Neighbours: `v[i + 1]` under `i < len - 1`.
#[inline(never)]
fn rises(v: &[u64]) -> u64 {
    let mut n = 0;
    if v.len() > 1 {
        for i in 0..v.len() - 1 {
            if v[i + 1] > v[i] {
                n += 1;
            }
        }
    }
    n
}

#[unsafe(no_mangle)]
pub extern "C" fn main() -> i32 {
    let xs = [3u64, 1, 4, 1, 5, 9, 2, 6, 5, 3, 5];
    let a = [2i64, -7, 1, 8, -2, 8];
    let b = [3i64, 1, -4, 1, 5];
    for k in 0..=xs.len() {
        put_u64(sum(&xs[..k]));
        put_u64(rises(&xs[..k]));
        put_u64(prefix(&xs, k));
    }
    for k in 0..=a.len() {
        for j in 0..=b.len() {
            put_u64(dot(&a[..k], &b[..j]) as u64);
        }
    }
    let mut word = *b"abcdefgh";
    for k in 0..=word.len() {
        rev(&mut word[..k]);
        put_bytes(&word);
    }
    0
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}
