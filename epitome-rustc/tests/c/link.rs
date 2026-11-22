//! Libraries named on the command line (`-l`, `-L KIND=DIR`), an object
//! passed to the linker (`-C link-arg`), and on macOS a framework
//! (`#[link(kind = "framework")]`).
//@ link: -L native=$DIR -l static=clink -C link-arg=$DIR/extra.o

unsafe extern "C" {
    fn c_link_value() -> i32;
    fn c_extra_value() -> i32;
}

#[cfg(target_os = "macos")]
#[link(name = "CoreFoundation", kind = "framework")]
unsafe extern "C" {
    fn CFAbsoluteTimeGetCurrent() -> f64;
}

#[cfg(target_os = "macos")]
fn framework() -> bool {
    // Seconds since 2001: more than 20 years' worth.
    unsafe { CFAbsoluteTimeGetCurrent() > 20.0 * 365.0 * 86400.0 }
}

#[cfg(not(target_os = "macos"))]
fn framework() -> bool {
    true
}

fn main() {
    unsafe {
        println!("{} {}", c_link_value(), c_extra_value());
    }
    println!("{}", framework());
}
