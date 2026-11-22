//! Atomics from `core::sync::atomic`: loads and stores, read-modify-write
//! operations, compare-exchange, `AtomicBool`, `AtomicPtr`, fences, and an
//! atomic in a `static`.
#![no_std]
#![no_main]

use core::sync::atomic::{AtomicBool, AtomicI64, AtomicPtr, AtomicU8, AtomicU32, AtomicUsize, Ordering, fence};

unsafe extern "C" {
    fn putchar(c: i32) -> i32;
}

fn put_i64(n: i64) {
    let mut buf = [0u8; 20];
    let mut i = 0;
    let neg = n < 0;
    let mut m = n.unsigned_abs();
    loop {
        buf[i] = b'0' + (m % 10) as u8;
        i += 1;
        m /= 10;
        if m == 0 {
            break;
        }
    }
    if neg {
        unsafe {
            putchar(b'-' as i32);
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

static COUNTER: AtomicU32 = AtomicU32::new(5);

fn bump() -> u32 {
    COUNTER.fetch_add(1, Ordering::Relaxed) + 1
}

#[unsafe(no_mangle)]
pub extern "C" fn main() -> i32 {
    let a = AtomicU32::new(10);
    put_i64(a.load(Ordering::SeqCst) as i64);
    a.store(20, Ordering::Release);
    put_i64(a.fetch_add(5, Ordering::AcqRel) as i64);
    put_i64(a.fetch_sub(3, Ordering::Relaxed) as i64);
    put_i64(a.load(Ordering::Acquire) as i64);
    put_i64(a.swap(100, Ordering::SeqCst) as i64);
    put_i64(a.fetch_and(0b110, Ordering::SeqCst) as i64);
    put_i64(a.fetch_or(0b1001, Ordering::SeqCst) as i64);
    put_i64(a.fetch_xor(0xff, Ordering::SeqCst) as i64);
    put_i64(a.load(Ordering::SeqCst) as i64);
    match a.compare_exchange(242, 7, Ordering::SeqCst, Ordering::Relaxed) {
        Ok(old) => put_i64(old as i64),
        Err(now) => put_i64(-(now as i64)),
    }
    match a.compare_exchange(1, 9, Ordering::SeqCst, Ordering::Relaxed) {
        Ok(old) => put_i64(old as i64),
        Err(now) => put_i64(-(now as i64)),
    }
    let mut tries = 0;
    while a.compare_exchange_weak(7, 8, Ordering::AcqRel, Ordering::Acquire).is_err() {
        tries += 1;
    }
    put_i64((a.load(Ordering::Relaxed) + tries * 0) as i64);

    let m = AtomicI64::new(-3);
    put_i64(m.fetch_max(4, Ordering::SeqCst));
    put_i64(m.fetch_min(-10, Ordering::SeqCst));
    put_i64(m.load(Ordering::SeqCst));
    let u = AtomicUsize::new(3);
    put_i64(u.fetch_max(1, Ordering::SeqCst) as i64);
    put_i64(u.fetch_update(Ordering::SeqCst, Ordering::SeqCst, |x| Some(x * 7)).unwrap_or(0) as i64);
    put_i64(u.load(Ordering::SeqCst) as i64);
    let b = AtomicU8::new(250);
    put_i64(b.fetch_add(10, Ordering::SeqCst) as i64);
    put_i64(b.load(Ordering::SeqCst) as i64);

    let flag = AtomicBool::new(false);
    put_i64(flag.swap(true, Ordering::SeqCst) as i64);
    put_i64(flag.fetch_and(false, Ordering::SeqCst) as i64);
    put_i64(flag.fetch_or(true, Ordering::SeqCst) as i64);
    put_i64(flag.load(Ordering::SeqCst) as i64);

    let mut x = 41i64;
    let mut y = 0i64;
    let p = AtomicPtr::new(&mut x as *mut i64);
    unsafe {
        *p.load(Ordering::Acquire) += 1;
    }
    put_i64(x);
    let old = p.swap(&mut y, Ordering::AcqRel);
    unsafe {
        *old += 1;
        *p.load(Ordering::Relaxed) = 5;
    }
    put_i64(x);
    put_i64(y);
    fence(Ordering::SeqCst);

    put_i64(bump() as i64);
    put_i64(bump() as i64);
    put_i64(COUNTER.load(Ordering::SeqCst) as i64);
    0
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}
