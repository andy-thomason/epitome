//! A panic in a `no_std` program: `Option::unwrap` of `None` reaches the
//! program's `#[panic_handler]` through `core`'s `panic_impl`, with the
//! message and the place of the call (`#[track_caller]`), which the
//! handler prints before it exits.
#![no_std]
#![no_main]

use core::fmt::Write;

unsafe extern "C" {
    fn putchar(c: i32) -> i32;
    fn exit(code: i32) -> !;
}

struct Out;

impl Write for Out {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for b in s.bytes() {
            unsafe {
                putchar(b as i32);
            }
        }
        Ok(())
    }
}

fn find(v: &[i32], x: i32) -> Option<usize> {
    v.iter().position(|&y| y == x)
}

#[unsafe(no_mangle)]
pub extern "C" fn main() -> i32 {
    let v = [3, 1, 4, 1, 5];
    let _ = writeln!(Out, "4 is at {}", find(&v, 4).unwrap());
    let missing = core::hint::black_box(9);
    let _ = writeln!(Out, "{} is at {}", missing, find(&v, missing).unwrap());
    0
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    let _ = writeln!(Out, "panicked: {}", info.message());
    if let Some(l) = info.location() {
        let _ = writeln!(Out, "at line {}, column {}", l.line(), l.column());
    }
    unsafe { exit(3) }
}
