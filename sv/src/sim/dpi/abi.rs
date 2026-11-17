//! Calling a C function whose signature is known only at run time.
//!
//! The arguments a DPI import takes are few in kind (Annex H, table H.1):
//! integers and pointers, which go in the general registers, and `double`
//! and `float`, which go in the floating-point ones; results are the same
//! kinds. So a call is a list of words placed by the platform's calling
//! convention, and a few instructions that load them and branch. The
//! conventions here are the System V AMD64 one (Linux x86-64) and AAPCS64
//! with Apple's variant (macOS arm64), which packs stack arguments by
//! their size.

use std::ffi::c_void;

/// One argument, as C sees it.
#[derive(Clone, Copy, Debug)]
pub enum Arg {
    /// An integer or pointer of `size` bytes, already sign- or
    /// zero-extended to 64 bits.
    Int(u64, u8),
    F64(f64),
    F32(f32),
}

/// What a call left in the result registers: the first general one and
/// the first floating-point one (as bits).
#[derive(Clone, Copy, Debug, Default)]
pub struct Ret {
    pub int: u64,
    pub fp: u64,
}

impl Ret {
    pub fn f64(self) -> f64 {
        f64::from_bits(self.fp)
    }

    pub fn f32(self) -> f32 {
        f32::from_bits(self.fp as u32)
    }
}

/// The block the assembly reads its registers from; the offsets are
/// written into the assembly.
#[repr(C)]
struct Regs {
    gp: [u64; 8],
    fp: [u64; 8],
    stack: *const u8,
    stack_len: usize,
    func: *const c_void,
    ret_gp: u64,
    ret_fp: u64,
}

/// Whether this target can make calls.
pub const SUPPORTED: bool = cfg!(any(all(target_arch = "x86_64", unix, not(target_os = "windows")), target_arch = "aarch64"));

/// Calls `f` with `args`.
///
/// # Safety
///
/// `f` must be a C function whose parameters are, in order, of the kinds
/// and sizes in `args`.
pub unsafe fn call(f: *const c_void, args: &[Arg]) -> Ret {
    let mut regs = Regs { gp: [0; 8], fp: [0; 8], stack: std::ptr::null(), stack_len: 0, func: f, ret_gp: 0, ret_fp: 0 };
    let stack = place(args, &mut regs);
    regs.stack = stack.as_ptr();
    regs.stack_len = stack.len();
    unsafe { raw_call(&mut regs) };
    Ret { int: regs.ret_gp, fp: regs.ret_fp }
}

/// Puts the arguments in registers and on the stack; returns the stack
/// area, a multiple of 16 bytes.
fn place(args: &[Arg], regs: &mut Regs) -> Vec<u8> {
    let (ngp_max, apple) = if cfg!(target_arch = "x86_64") { (6, false) } else { (8, cfg!(target_vendor = "apple")) };
    let (mut ngp, mut nfp) = (0, 0);
    let mut stack: Vec<u8> = Vec::new();
    // A stack argument: packed by its size on Apple's arm64, an 8-byte
    // slot elsewhere.
    let mut push = |bytes: &[u8]| {
        let size = if apple { bytes.len() } else { 8 };
        let at = stack.len().next_multiple_of(size);
        stack.resize(at + size, 0);
        stack[at..at + bytes.len()].copy_from_slice(bytes);
    };
    for a in args {
        match *a {
            Arg::Int(v, size) => {
                if ngp < ngp_max {
                    regs.gp[ngp] = v;
                    ngp += 1;
                } else {
                    let n = if apple { size.max(1) as usize } else { 8 };
                    push(&v.to_le_bytes()[..n]);
                }
            }
            Arg::F64(x) => {
                if nfp < 8 {
                    regs.fp[nfp] = x.to_bits();
                    nfp += 1;
                } else {
                    push(&x.to_bits().to_le_bytes());
                }
            }
            Arg::F32(x) => {
                if nfp < 8 {
                    regs.fp[nfp] = x.to_bits() as u64;
                    nfp += 1;
                } else {
                    push(&x.to_bits().to_le_bytes());
                }
            }
        }
    }
    let n = stack.len().next_multiple_of(16);
    stack.resize(n, 0);
    stack
}

#[cfg(all(target_arch = "x86_64", unix))]
unsafe fn raw_call(regs: &mut Regs) {
    // r12 holds the block and r13 the stack pointer to go back to; both
    // survive the call. The stack area is copied below an aligned stack
    // pointer, where the callee finds its stack arguments.
    unsafe {
        std::arch::asm!(
            "mov r13, rsp",
            "mov rcx, [r12 + 136]",
            "sub rsp, rcx",
            "and rsp, -16",
            "mov rsi, [r12 + 128]",
            "mov rdi, rsp",
            "cld",
            "rep movsb",
            "movsd xmm0, [r12 + 64]",
            "movsd xmm1, [r12 + 72]",
            "movsd xmm2, [r12 + 80]",
            "movsd xmm3, [r12 + 88]",
            "movsd xmm4, [r12 + 96]",
            "movsd xmm5, [r12 + 104]",
            "movsd xmm6, [r12 + 112]",
            "movsd xmm7, [r12 + 120]",
            "mov rdi, [r12]",
            "mov rsi, [r12 + 8]",
            "mov rdx, [r12 + 16]",
            "mov rcx, [r12 + 24]",
            "mov r8, [r12 + 32]",
            "mov r9, [r12 + 40]",
            // An upper bound on the vector registers used, for a variadic
            // callee.
            "mov eax, 8",
            "call qword ptr [r12 + 144]",
            "mov [r12 + 152], rax",
            "movq [r12 + 160], xmm0",
            "mov rsp, r13",
            in("r12") regs as *mut Regs,
            out("r13") _,
            clobber_abi("C"),
        );
    }
}

#[cfg(target_arch = "aarch64")]
unsafe fn raw_call(regs: &mut Regs) {
    // x20 holds the block and x21 the stack pointer to go back to; both
    // survive the call.
    unsafe {
        std::arch::asm!(
            "mov x21, sp",
            "ldr x9, [x20, #136]",
            "sub x10, x21, x9",
            "and x10, x10, #-16",
            "mov sp, x10",
            "ldr x11, [x20, #128]",
            "mov x12, #0",
            "3:",
            "cmp x12, x9",
            "b.hs 4f",
            "ldr x13, [x11, x12]",
            "str x13, [x10, x12]",
            "add x12, x12, #8",
            "b 3b",
            "4:",
            "ldp d0, d1, [x20, #64]",
            "ldp d2, d3, [x20, #80]",
            "ldp d4, d5, [x20, #96]",
            "ldp d6, d7, [x20, #112]",
            "ldp x0, x1, [x20, #0]",
            "ldp x2, x3, [x20, #16]",
            "ldp x4, x5, [x20, #32]",
            "ldp x6, x7, [x20, #48]",
            "ldr x16, [x20, #144]",
            "blr x16",
            "str x0, [x20, #152]",
            "str d0, [x20, #160]",
            "mov sp, x21",
            in("x20") regs as *mut Regs,
            out("x21") _,
            clobber_abi("C"),
        );
    }
}

#[cfg(not(any(all(target_arch = "x86_64", unix), target_arch = "aarch64")))]
unsafe fn raw_call(_regs: &mut Regs) {
    unreachable!("DPI calls are not supported on this target");
}

#[cfg(test)]
mod tests {
    use super::*;

    extern "C" fn mix(a: i8, b: f64, c: i16, d: f32, e: i32, f: i64, g: *const u8, h: u8, i: i32, j: f64, k: i8, l: f32) -> f64 {
        let s = unsafe { *g } as f64;
        a as f64 + b + c as f64 + d as f64 + e as f64 + f as f64 + s + h as f64 + i as f64 + j + k as f64 + l as f64
    }

    extern "C" fn many(
        a0: i32, a1: i32, a2: i32, a3: i32, a4: i32, a5: i32, a6: i32, a7: i32, a8: i8, a9: i16, a10: i64, a11: i32,
        f0: f64, f1: f64, f2: f64, f3: f64, f4: f64, f5: f64, f6: f64, f7: f64, f8: f32, f9: f64,
    ) -> i64 {
        let ints = [a0, a1, a2, a3, a4, a5, a6, a7, a8 as i32, a9 as i32, a10 as i32, a11];
        let mut r = 0i64;
        for (k, v) in ints.iter().enumerate() {
            r += (*v as i64) << (k * 4);
        }
        r + (f0 + f1 + f2 + f3 + f4 + f5 + f6 + f7 + f8 as f64 + f9) as i64 * 1_000_000_000_000
    }

    extern "C" fn ret_f32(x: f32) -> f32 {
        x * 2.0
    }

    extern "C" fn ret_i8(x: i8) -> i8 {
        x - 1
    }

    #[test]
    fn calls() {
        if !SUPPORTED {
            return;
        }
        let byte = 7u8;
        let args = [
            Arg::Int(-1i64 as u64, 1),
            Arg::F64(0.5),
            Arg::Int(-300i64 as u64, 2),
            Arg::F32(0.25),
            Arg::Int(100_000, 4),
            Arg::Int(1 << 40, 8),
            Arg::Int(&byte as *const u8 as u64, 8),
            Arg::Int(200, 1),
            Arg::Int(-5i64 as u64, 4),
            Arg::F64(-2.0),
            Arg::Int(3, 1),
            Arg::F32(1.5),
        ];
        let r = unsafe { call(mix as *const c_void, &args) };
        let want = mix(-1, 0.5, -300, 0.25, 100_000, 1 << 40, &byte, 200, -5, -2.0, 3, 1.5);
        assert_eq!(r.f64(), want);

        let mut args: Vec<Arg> = (1..=8).map(|k| Arg::Int(k, 4)).collect();
        args.extend([Arg::Int(9, 1), Arg::Int(10, 2), Arg::Int(11, 8), Arg::Int(12, 4)]);
        args.extend((0..8).map(|k| Arg::F64(k as f64)));
        args.extend([Arg::F32(100.0), Arg::F64(1000.0)]);
        let r = unsafe { call(many as *const c_void, &args) };
        let want = many(1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 100.0, 1000.0);
        assert_eq!(r.int as i64, want);

        let r = unsafe { call(ret_f32 as *const c_void, &[Arg::F32(1.25)]) };
        assert_eq!(r.f32(), 2.5);
        let r = unsafe { call(ret_i8 as *const c_void, &[Arg::Int(-128i64 as u64, 1)]) };
        assert_eq!(r.int as u8 as i8, 127);
    }
}
