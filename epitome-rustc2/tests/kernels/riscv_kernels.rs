//! Rust on the host (an Arm Mac, say) calling RISC-V code: the functions
//! marked `#[epitome::target("riscv64")]` are compiled for RV64 and run
//! there; `main` and everything else runs on the host.
//!
//!     epitome-rustc2 --std -o riscv_kernels riscv_kernels.rs && ./riscv_kernels
//!
//! `arch()` is compiled once for each machine, with its own `cfg`, so the
//! host's says `aarch64` (or `x86_64`) and the kernel's `riscv64`. The
//! slices are copied into the RISC-V machine's memory and, being `&mut`,
//! back. (`tests/kernels.rs` runs this, on the host it is built for.)
//!
//! run on: host
//! prints: main runs on {host_arch}
//! prints: the kernels run on riscv64
//! prints:                                           ....
//! prints:                                          ....-..
//! prints:                                        ..+-+@*-+.
//! prints:                                   ........@@@@@:.......
//! prints:                                ...:#@@@@@@@@@@@@@@#::=:*.
//! prints:                             .....+:@@@@@@@@@@@@@@@@@@=:..
//! prints:                   ..:....%......@@@@@@@@@@@@@@@@@@@@@@@@:.
//! prints:                   ...-@@@@@@=:::@@@@@@@@@@@@@@@@@@@@@@@@:.
//! prints:             .....-.:@@@@@@@@@@@+@@@@@@@@@@@@@@@@@@@@@@@:.
//! prints:      @@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@:...
//! prints:             .....-.:@@@@@@@@@@@+@@@@@@@@@@@@@@@@@@@@@@@:.
//! prints:                   ...-@@@@@@=:::@@@@@@@@@@@@@@@@@@@@@@@@:.
//! prints:                   ..:....%......@@@@@@@@@@@@@@@@@@@@@@@@:.
//! prints:                             .....+:@@@@@@@@@@@@@@@@@@=:..
//! prints:                                ...:#@@@@@@@@@@@@@@#::=:*.
//! prints:                                   ........@@@@@:.......
//! prints:                                        ..+-+@*-+.
//! prints:                                          ....-..
//! prints:                                           ....

/// The machine this code was compiled for. Both sides use it, and each
/// gets its own copy, built with its own `cfg`.
fn arch() -> &'static str {
    if cfg!(target_arch = "riscv64") {
        "riscv64"
    } else if cfg!(target_arch = "aarch64") {
        "aarch64"
    } else {
        "x86_64"
    }
}

/// Writes the name of the machine it runs on into `out`.
#[epitome::target("riscv64")]
pub fn whoami(out: &mut [u8]) -> u64 {
    let a = arch().as_bytes();
    out[..a.len()].copy_from_slice(a);
    a.len() as u64
}

/// One row of the Mandelbrot set, at height `y`, as characters.
#[epitome::target("riscv64")]
pub fn mandel_row(y: f64, row: &mut [u8]) {
    let w = row.len();
    for (i, c) in row.iter_mut().enumerate() {
        let x = -2.2 + 3.0 * i as f64 / w as f64;
        let (mut zr, mut zi) = (0.0f64, 0.0f64);
        let mut n = 0;
        while n < 50 && zr * zr + zi * zi < 4.0 {
            let t = zr * zr - zi * zi + x;
            zi = 2.0 * zr * zi + y;
            zr = t;
            n += 1;
        }
        *c = b" .:-=+*#%@"[n * 9 / 50];
    }
}

fn main() {
    println!("main runs on {}", arch());
    let mut name = [0u8; 16];
    let n = whoami(&mut name) as usize;
    println!("the kernels run on {}", std::str::from_utf8(&name[..n]).unwrap());
    for k in 1..20 {
        let mut row = [b' '; 64];
        mandel_row(-1.2 + 0.12 * k as f64, &mut row);
        println!("{}", std::str::from_utf8(&row).unwrap().trim_end());
    }
}
