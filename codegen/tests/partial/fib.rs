#![no_std]

/// fib(n) by a loop: no memory, no stack, no calls.
#[no_mangle]
pub extern "C" fn fib(n: u32) -> u32 {
    let (mut a, mut b) = (0u32, 1u32);
    for _ in 0..n {
        let c = a.wrapping_add(b);
        a = b;
        b = c;
    }
    a
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}
