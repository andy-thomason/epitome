//! `asm!` on x86-64: outputs, inputs, `inout` (with `=>` and `_`), named,
//! numbered and implicit operands, modifiers, explicit registers, `const`
//! and `sym` operands, `clobber_abi`, AT&T syntax, options; and the
//! functions of `core::arch::x86_64` written with it.
#![no_std]
#![no_main]

use core::arch::asm;
use core::arch::x86_64::{__cpuid_count, _bittest64, _bittestandcomplement64, _bittestandreset64, _bittestandset64, _mm_stream_si32};

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

fn lea3(x: u64) -> u64 {
    let r;
    unsafe {
        asm!("lea {0}, [{1} + {1}*2]", out(reg) r, in(reg) x, options(pure, nomem, nostack));
    }
    r
}

fn add_shl(mut x: u32, y: u32) -> u32 {
    unsafe {
        asm!("add {x:e}, {y:e}", "shl {x:e}, 1", x = inout(reg) x, y = in(reg) y);
    }
    x
}

fn swap(a: u64, b: u64) -> (u64, u64) {
    let (c, d);
    unsafe {
        asm!("xchg {}, {}", inout(reg) a => c, inout(reg) b => d, options(nomem, nostack));
    }
    (c, d)
}

fn mul_hi(a: u64, b: u64) -> u64 {
    let hi;
    unsafe {
        asm!("mul {b}", b = in(reg) b, inlateout("rax") a => _, lateout("rdx") hi, options(nomem, nostack));
    }
    hi
}

const K: u64 = 40;

fn add_k(x: u64) -> u64 {
    let r;
    unsafe {
        asm!("lea {r}, [{x} + {k}]", r = lateout(reg) r, x = in(reg) x, k = const K, options(pure, nomem, nostack));
    }
    r
}

fn att(x: u32) -> u32 {
    let r: u32;
    unsafe {
        asm!("movl {0:e}, {1:e}", "addl $7, {1:e}", in(reg) x, out(reg) r, options(att_syntax, nomem, nostack));
    }
    r
}

extern "C" fn helper() -> u64 {
    1234
}

fn call_sym() -> u64 {
    let r: u64;
    unsafe {
        asm!("call {f}", f = sym helper, out("rax") r, clobber_abi("C"));
    }
    r
}

fn bytes(x: u32) -> u32 {
    let r: u32;
    unsafe {
        asm!("mov {r:e}, {x:e}", "bswap {r:e}", x = in(reg) x, r = out(reg) r, options(pure, nomem, nostack));
    }
    r
}

#[unsafe(no_mangle)]
pub extern "C" fn main() -> i32 {
    put_i64(lea3(14) as i64);
    put_i64(add_shl(5, 6) as i64);
    let (c, d) = swap(3, 4);
    put_i64((c * 10 + d) as i64);
    put_i64(mul_hi(1 << 40, 1 << 40) as i64);
    put_i64(add_k(2) as i64);
    put_i64(att(35) as i64);
    put_i64(call_sym() as i64);
    put_i64(bytes(0x01020304) as i64);

    let mut v: i64 = 0b1010;
    unsafe {
        put_i64(_bittest64(&v, 1) as i64);
        put_i64(_bittest64(&v, 2) as i64);
        put_i64(_bittestandset64(&mut v, 0) as i64);
        put_i64(_bittestandreset64(&mut v, 3) as i64);
        put_i64(_bittestandcomplement64(&mut v, 63) as i64);
    }
    put_i64(v);
    let mut s = 0i32;
    unsafe {
        _mm_stream_si32(&mut s, 77);
    }
    put_i64(s as i64);
    // The highest leaf is the same for both builds on one machine.
    let leaf = unsafe { __cpuid_count(0, 0) }.eax;
    put_i64((leaf > 0) as i64);
    0
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}
