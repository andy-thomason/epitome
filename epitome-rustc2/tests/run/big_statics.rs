//! Large statics, bigger than the stack: a 16 MB array of zeros, a struct
//! holding one, a struct of zeros and other values (worked out at compile
//! time), and one whose initialiser runs at start-up (a float the static
//! evaluator does not work out), each written to and read from. Also
//! null pointers and `MaybeUninit::uninit()`, which are zeros too, and
//! array repeats of constants (a static's, and locals') and of a value
//! known only at run time.
#![no_std]
#![no_main]
#![allow(static_mut_refs)]

unsafe extern "C" {
    fn putchar(c: i32) -> i32;
}

fn put_u64(n: u64) {
    let mut buf = [0u8; 20];
    let mut i = 0;
    let mut m = n;
    loop {
        buf[i] = b'0' + (m % 10) as u8;
        i += 1;
        m /= 10;
        if m == 0 {
            break;
        }
    }
    for &c in buf[..i].iter().rev() {
        unsafe {
            putchar(c as i32);
        }
    }
    unsafe {
        putchar(10);
    }
}

const MB16: usize = 16 << 20;

static mut MEM: [u8; MB16] = [0; MB16];

struct Machine {
    pc: u64,
    regs: [u64; 32],
    mem: [u8; MB16],
    halted: bool,
}

static mut M: Machine = Machine { pc: 0, regs: [0; 32], mem: [0; MB16], halted: false };

// Not all zeros: worked out at compile time.
static mut N: Machine = Machine { pc: 0x1000, regs: [7; 32], mem: [0; MB16], halted: true };

struct Scaled {
    scale: f64,
    words: [u32; 4 << 20],
}

// A float the static evaluator gives up on: built at start-up, in place.
static mut S: Scaled = Scaled { scale: 1.5 * 3.0, words: [9; 4 << 20] };

// A repeat of a constant that is not zeros: worked out at compile time.
static mut W: [u32; 1 << 16] = [0x0102_0304; 1 << 16];

// Null pointers and uninitialised memory: zeros too.
struct Rv {
    x: [u64; 32],
    mem: *mut u8,
    scratch: core::mem::MaybeUninit<[u8; 1 << 20]>,
}

static mut RV: Rv = Rv { x: [0; 32], mem: core::ptr::null_mut(), scratch: core::mem::MaybeUninit::uninit() };

static ZEROS: ([u16; 1000], (u8, bool, char), [f32; 3]) = ([0; 1000], (0, false, '\0'), [0.0; 3]);

fn sum(b: &[u8]) -> u64 {
    b.iter().map(|&x| x as u64).sum()
}

fn fill(x: u32) -> [u32; 7] {
    [x; 7]
}

#[unsafe(no_mangle)]
pub extern "C" fn main() -> i32 {
    unsafe {
        put_u64(sum(&MEM));
        MEM[0] = 1;
        MEM[MB16 / 2] = 2;
        MEM[MB16 - 1] = 3;
        put_u64(sum(&MEM));

        put_u64(M.pc + sum(&M.mem) + M.regs.iter().sum::<u64>() + M.halted as u64);
        M.pc = 4;
        M.regs[31] = 5;
        M.mem[MB16 - 1] = 6;
        M.halted = true;
        put_u64(M.pc + sum(&M.mem) + M.regs.iter().sum::<u64>() + M.halted as u64);

        put_u64(N.pc + sum(&N.mem) + N.regs.iter().sum::<u64>() + N.halted as u64);
        N.mem[12345] = 8;
        put_u64(sum(&N.mem));

        put_u64((S.scale * 2.0) as u64);
        put_u64(S.words.iter().map(|&w| w as u64).sum::<u64>());
        S.words[100] = 1;
        put_u64(S.words.iter().map(|&w| w as u64).sum::<u64>());

        put_u64(W.iter().map(|&w| w as u64).sum::<u64>());
        W[65535] = 0;
        put_u64(W.iter().map(|&w| w as u64).sum::<u64>());

        put_u64(RV.x.iter().sum::<u64>() + RV.mem.is_null() as u64);
        RV.mem = MEM.as_mut_ptr();
        RV.scratch.as_mut_ptr().cast::<u8>().add(1000).write(*RV.mem.add(MB16 / 2));
        put_u64(*RV.scratch.as_ptr().cast::<u8>().add(1000) as u64);

        put_u64(ZEROS.0.iter().map(|&x| x as u64).sum::<u64>() + ZEROS.1.0 as u64 + ZEROS.1.1 as u64 + ZEROS.1.2 as u64 + ZEROS.2[2] as u64);
    }
    let a = [0x0101u16; 9];
    put_u64(a.iter().map(|&x| x as u64).sum::<u64>());
    let b = [(3u8, 4u32); 5];
    put_u64(b.iter().map(|&(x, y)| x as u64 * y as u64).sum::<u64>());
    let mut total = 0;
    for x in 0..4 {
        total += fill(x * 3).iter().sum::<u32>();
    }
    put_u64(total as u64);
    0
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}
