//! `core::fmt`: `write!` and `writeln!` to a `fmt::Write` that prints
//! with `putchar`, through `format_args!` and `core`'s own formatting:
//! integers in every base, with widths, fill, alignment, signs, `#` and
//! zero padding; widths and precisions from arguments; strings, chars and
//! bools; `Debug` of tuples, arrays, `Option` and derived structs and
//! enums (`{:?}` and `{:#?}`); a user `Display`; positional, named and
//! captured arguments; `{{` and `}}`; `format_args!` nested; and floats
//! (`f32` and `f64`: shortest, with a precision, in exponent form).
#![no_std]
#![allow(dead_code)]
#![no_main]

use core::fmt::{self, Write};

unsafe extern "C" {
    fn putchar(c: i32) -> i32;
}

struct Out;

impl Write for Out {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for b in s.bytes() {
            unsafe {
                putchar(b as i32);
            }
        }
        Ok(())
    }
}

#[derive(Debug)]
struct Point {
    x: i32,
    y: i32,
}

#[derive(Debug)]
enum Shape {
    Dot,
    Circle(u8),
    Box { w: u16, h: u16 },
}

struct Money(i64);

impl fmt::Display for Money {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "${}.{:02}", self.0 / 100, self.0 % 100)
    }
}

/// Counts what it is given, and fails after `limit` bytes.
struct Counter {
    n: usize,
    limit: usize,
}

impl Write for Counter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.n += s.len();
        if self.n > self.limit { Err(fmt::Error) } else { Ok(()) }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn main() -> i32 {
    let mut o = Out;
    let _ = writeln!(o, "hello");
    let _ = writeln!(o, "{} {} {}", 1, -2, 300u64);
    let _ = writeln!(o, "{:5}|{:<5}|{:^5}|{:>5}|", 42, 42, 42, 42);
    let _ = writeln!(o, "{:*^9}|{:-<6}|{:#>4}", "mid", 7, 'c');
    let _ = writeln!(o, "{:+} {:+} {:05} {:+06} {:<05}", 5, -5, 42, 42, 42);
    let _ = writeln!(o, "{:x} {:X} {:o} {:b} {:#x} {:#b} {:#o}", 255, 255, 8, 5, 255, 5, 8);
    let _ = writeln!(o, "{:08x} {:#010x} {:x}", 0xbeefu32, 0xbeef, -1i8);
    let _ = writeln!(o, "{} {} {}", u128::MAX, i128::MIN, i64::MIN);
    let w = 7;
    let p = 3;
    let _ = writeln!(o, "[{:w$}] [{:1$}] [{:.*}] [{:>w$.p$}]", 5, 6, 2, "abcdef", "xyzxyz");
    let name = "world";
    let _ = writeln!(o, "{name}! {0} {0} {greeting}", 9, greeting = "hi");
    let _ = writeln!(o, "{{literal}} {}", true);
    let _ = writeln!(o, "{:?} {:?} {:?}", "q\"uote\n", 'x', '\'');
    let _ = writeln!(o, "{:?} {:?}", (1, "two", 3.5f64 > 1.0), [1u8, 2, 3]);
    let _ = writeln!(o, "{:?} {:?}", Some(4), None::<i32>);
    let _ = writeln!(o, "{:?}", Point { x: 1, y: -2 });
    let _ = writeln!(o, "{:#?}", Point { x: 3, y: 4 });
    let _ = writeln!(o, "{:?} {:?} {:?}", Shape::Dot, Shape::Circle(9), Shape::Box { w: 2, h: 3 });
    let _ = writeln!(o, "{:x?} {:X?}", [10, 11], 255);
    let _ = writeln!(o, "{} {:>10}|", Money(12345), Money(7));
    let _ = writeln!(o, "{}", format_args!("nested {}", format_args!("{}-{}", 1, 2)));
    let _ = writeln!(o, "{:.3} {}", "truncated", "");
    let mut c = Counter { n: 0, limit: 10 };
    let ok = write!(c, "{}", 12345).is_ok();
    let bad = write!(c, "{:>20}", 1).is_err();
    let _ = writeln!(o, "{} {} {}", ok, bad, c.n);
    let xs = [0.0f64, -0.0, 1.0, 0.1, 1.5e300, -2.5e-300, 3.14159265358979, 1e21, f64::NAN, f64::INFINITY, f64::MIN_POSITIVE, 123456.789];
    for x in xs {
        let _ = writeln!(o, "{} {:?} {:.3} {:e} {:E} {:+.1} {:10.2}|", x, x, x, x, x, x, x);
    }
    for y in [0.1f32, 16777217.0, -1.25e-7, f32::MAX] {
        let _ = writeln!(o, "{} {:?} {:.2} {:e}", y, y, y, y);
    }
    let _ = write!(o, "{}", "end\n");
    0
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}
