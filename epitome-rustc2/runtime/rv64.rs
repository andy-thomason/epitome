//! The launch runtime for RISC-V kernels: an RV64 interpreter (I, M, A
//! and the basic F and D operations) that runs a kernel of an image the
//! compiler linked (`epitome_codegen::flat`) in memory of its own.
//! epitome-rustc2 compiles this crate into a host program that calls
//! kernels; a program can give its own `__epitome_launch` (a driver for a
//! real machine) instead (`--launch-runtime none`).
//!
//! Each launch loads the image afresh, puts the arguments in registers as
//! the LP64D convention has them (integers in a0-a7, floats in fa0-fa7),
//! runs the kernel until it returns, and gives a0 (or fa0). A call to a
//! stub address does what the C library function would: `putchar` is the
//! host's, `memcpy`, `memmove`, `memset` and `memcmp` work on the machine's
//! memory. Anything wrong (an instruction it does not have, an address
//! outside its memory) is reported on stderr, and the host aborts.
#![no_std]
#![allow(clippy::all)]

unsafe extern "C" {
    fn putchar(c: i32) -> i32;
    fn write(fd: i32, buf: *const u8, n: usize) -> isize;
    fn abort() -> !;
    fn calloc(n: usize, size: usize) -> *mut u8;
}

const MEM: usize = 16 << 20;
const RETURN: u64 = 0x800;
const STUBS: u64 = 0x1000;

/// The stubs, in the order the compiler gives them addresses.
pub const STUB_NAMES: [&str; 6] = ["putchar", "memcpy", "memmove", "memset", "memcmp", "bcmp"];

// The machine's memory comes from the host's `calloc` at the first launch.
static mut MACHINE: Machine = Machine { x: [0; 32], f: [0; 32], pc: 0, reserved: 0, mem: core::ptr::null_mut() };

struct Machine {
    x: [u64; 32],
    f: [u64; 32],
    pc: u64,
    /// The address `lr` reserved, plus one (0 for none).
    reserved: u64,
    /// `MEM` bytes.
    mem: *mut u8,
}

fn fail(what: &str, at: u64) -> ! {
    let mut buf = [0u8; 160];
    let mut n = 0;
    for &b in b"epitome launch (riscv64): ".iter().chain(what.as_bytes()).chain(b" at 0x".iter()) {
        if n < buf.len() {
            buf[n] = b;
            n += 1;
        }
    }
    for i in (0..16).rev() {
        let d = ((at >> (4 * i)) & 15) as u8;
        if n < buf.len() {
            buf[n] = if d < 10 { b'0' + d } else { b'a' + d - 10 };
            n += 1;
        }
    }
    if n < buf.len() {
        buf[n] = b'\n';
        n += 1;
    }
    unsafe {
        write(2, buf.as_ptr(), n);
        abort()
    }
}

fn sext(v: u64, bits: u32) -> i64 {
    let s = 64 - bits;
    ((v << s) as i64) >> s
}

fn bits(w: u32, hi: u32, lo: u32) -> u32 {
    (w >> lo) & ((1u32 << (hi - lo + 1)) - 1)
}

fn le64(b: &[u8], at: usize) -> u64 {
    let mut v = 0u64;
    for i in 0..8 {
        v |= (b[at + i] as u64) << (8 * i);
    }
    v
}

impl Machine {
    fn mem(&self) -> &mut [u8] {
        unsafe { core::slice::from_raw_parts_mut(self.mem, MEM) }
    }

    fn check(&self, a: u64, n: usize) -> usize {
        let a = a as usize;
        if a < 0x1_0000 || a + n > MEM {
            fail("a memory access outside the machine", a as u64);
        }
        a
    }
    fn load(&self, a: u64, n: usize) -> u64 {
        let a = self.check(a, n);
        let mut v = 0u64;
        for i in 0..n {
            v |= (self.mem()[a + i] as u64) << (8 * i);
        }
        v
    }
    fn store(&mut self, a: u64, n: usize, v: u64) {
        let a = self.check(a, n);
        for i in 0..n {
            self.mem()[a + i] = (v >> (8 * i)) as u8;
        }
    }
    fn set(&mut self, r: u32, v: u64) {
        if r != 0 {
            self.x[r as usize] = v;
        }
    }
    fn fd(&self, r: u32) -> f64 {
        f64::from_bits(self.f[r as usize])
    }
    fn fs(&self, r: u32) -> f32 {
        f32::from_bits(self.f[r as usize] as u32)
    }
    fn set_d(&mut self, r: u32, v: f64) {
        self.f[r as usize] = v.to_bits();
    }
    fn set_s(&mut self, r: u32, v: f32) {
        self.f[r as usize] = 0xffff_ffff_0000_0000 | v.to_bits() as u64;
    }

    /// What a call to stub `k` does.
    fn stub(&mut self, k: u64) {
        let a = |m: &Machine, i: usize| m.x[10 + i];
        match k {
            0 => unsafe {
                putchar(a(self, 0) as i32);
            },
            1 | 2 => {
                let (d, s, n) = (a(self, 0), a(self, 1), a(self, 2) as usize);
                let (d, s) = (self.check(d, n), self.check(s, n));
                self.mem().copy_within(s..s + n, d);
            }
            3 => {
                let (d, v, n) = (a(self, 0), a(self, 1) as u8, a(self, 2) as usize);
                let d = self.check(d, n);
                for i in 0..n {
                    self.mem()[d + i] = v;
                }
            }
            4 | 5 => {
                let (p, q, n) = (a(self, 0), a(self, 1), a(self, 2) as usize);
                let (p, q) = (self.check(p, n), self.check(q, n));
                let mut r = 0i64;
                for i in 0..n {
                    if self.mem()[p + i] != self.mem()[q + i] {
                        r = self.mem()[p + i] as i64 - self.mem()[q + i] as i64;
                        break;
                    }
                }
                self.x[10] = r as u64;
            }
            _ => fail("a call of a function the machine does not have", self.pc),
        }
    }

    fn step(&mut self) {
        let pc = self.pc;
        let w = self.load(pc, 4) as u32;
        let (rd, rs1, rs2, f3, f7) = (bits(w, 11, 7), bits(w, 19, 15), bits(w, 24, 20), bits(w, 14, 12), bits(w, 31, 25));
        let x1 = self.x[rs1 as usize];
        let x2 = self.x[rs2 as usize];
        let imm_i = sext(bits(w, 31, 20) as u64, 12);
        let imm_s = sext(((bits(w, 31, 25) << 5) | bits(w, 11, 7)) as u64, 12);
        let imm_b = sext(((bits(w, 31, 31) << 12) | (bits(w, 7, 7) << 11) | (bits(w, 30, 25) << 5) | (bits(w, 11, 8) << 1)) as u64, 13);
        let imm_u = sext((w & 0xffff_f000) as u64, 32);
        let imm_j = sext(((bits(w, 31, 31) << 20) | (bits(w, 19, 12) << 12) | (bits(w, 20, 20) << 11) | (bits(w, 30, 21) << 1)) as u64, 21);
        let mut next = pc.wrapping_add(4);
        match w & 0x7f {
            0x37 => self.set(rd, imm_u as u64),
            0x17 => self.set(rd, pc.wrapping_add(imm_u as u64)),
            0x6f => {
                self.set(rd, next);
                next = pc.wrapping_add(imm_j as u64);
            }
            0x67 => {
                let t = x1.wrapping_add(imm_i as u64) & !1;
                self.set(rd, next);
                next = t;
            }
            0x63 => {
                let take = match f3 {
                    0 => x1 == x2,
                    1 => x1 != x2,
                    4 => (x1 as i64) < (x2 as i64),
                    5 => (x1 as i64) >= (x2 as i64),
                    6 => x1 < x2,
                    7 => x1 >= x2,
                    _ => fail("an instruction the machine does not have", pc),
                };
                if take {
                    next = pc.wrapping_add(imm_b as u64);
                }
            }
            0x03 => {
                let a = x1.wrapping_add(imm_i as u64);
                let v = match f3 {
                    0 => sext(self.load(a, 1), 8) as u64,
                    1 => sext(self.load(a, 2), 16) as u64,
                    2 => sext(self.load(a, 4), 32) as u64,
                    3 => self.load(a, 8),
                    4 => self.load(a, 1),
                    5 => self.load(a, 2),
                    6 => self.load(a, 4),
                    _ => fail("an instruction the machine does not have", pc),
                };
                self.set(rd, v);
            }
            0x23 => {
                let a = x1.wrapping_add(imm_s as u64);
                let n = match f3 {
                    0 => 1,
                    1 => 2,
                    2 => 4,
                    3 => 8,
                    _ => fail("an instruction the machine does not have", pc),
                };
                self.store(a, n, x2);
            }
            0x13 => {
                let sh = bits(w, 25, 20);
                let v = match f3 {
                    0 => x1.wrapping_add(imm_i as u64),
                    2 => ((x1 as i64) < imm_i) as u64,
                    3 => (x1 < imm_i as u64) as u64,
                    4 => x1 ^ imm_i as u64,
                    6 => x1 | imm_i as u64,
                    7 => x1 & imm_i as u64,
                    1 if bits(w, 31, 26) == 0 => x1 << sh,
                    5 if bits(w, 31, 26) == 0 => x1 >> sh,
                    5 if bits(w, 31, 26) == 0x10 => ((x1 as i64) >> sh) as u64,
                    _ => fail("an instruction the machine does not have", pc),
                };
                self.set(rd, v);
            }
            0x1b => {
                let v = match (f3, f7) {
                    (0, _) => sext(x1.wrapping_add(imm_i as u64), 32),
                    (1, 0) => sext((x1 as u32).wrapping_shl(rs2) as u64, 32),
                    (5, 0) => sext(((x1 as u32) >> rs2) as u64, 32),
                    (5, 0x20) => ((x1 as i32) >> rs2) as i64,
                    _ => fail("an instruction the machine does not have", pc),
                };
                self.set(rd, v as u64);
            }
            0x33 => {
                let (s1, s2) = (x1 as i64, x2 as i64);
                let v = match (f7, f3) {
                    (0, 0) => x1.wrapping_add(x2),
                    (0x20, 0) => x1.wrapping_sub(x2),
                    (0, 1) => x1 << (x2 & 63),
                    (0, 2) => (s1 < s2) as u64,
                    (0, 3) => (x1 < x2) as u64,
                    (0, 4) => x1 ^ x2,
                    (0, 5) => x1 >> (x2 & 63),
                    (0x20, 5) => (s1 >> (x2 & 63)) as u64,
                    (0, 6) => x1 | x2,
                    (0, 7) => x1 & x2,
                    (1, 0) => x1.wrapping_mul(x2),
                    (1, 1) => ((s1 as i128 * s2 as i128) >> 64) as u64,
                    (1, 2) => ((s1 as i128 * x2 as i128) >> 64) as u64,
                    (1, 3) => ((x1 as u128 * x2 as u128) >> 64) as u64,
                    (1, 4) => {
                        if s2 == 0 {
                            u64::MAX
                        } else if s1 == i64::MIN && s2 == -1 {
                            s1 as u64
                        } else {
                            (s1 / s2) as u64
                        }
                    }
                    (1, 5) => {
                        if x2 == 0 {
                            u64::MAX
                        } else {
                            x1 / x2
                        }
                    }
                    (1, 6) => {
                        if s2 == 0 {
                            x1
                        } else if s1 == i64::MIN && s2 == -1 {
                            0
                        } else {
                            (s1 % s2) as u64
                        }
                    }
                    (1, 7) => {
                        if x2 == 0 {
                            x1
                        } else {
                            x1 % x2
                        }
                    }
                    _ => fail("an instruction the machine does not have", pc),
                };
                self.set(rd, v);
            }
            0x3b => {
                let (a, b) = (x1 as u32, x2 as u32);
                let (sa, sb) = (a as i32, b as i32);
                let v: i32 = match (f7, f3) {
                    (0, 0) => a.wrapping_add(b) as i32,
                    (0x20, 0) => a.wrapping_sub(b) as i32,
                    (0, 1) => a.wrapping_shl(b & 31) as i32,
                    (0, 5) => (a >> (b & 31)) as i32,
                    (0x20, 5) => sa >> (b & 31),
                    (1, 0) => a.wrapping_mul(b) as i32,
                    (1, 4) => {
                        if sb == 0 {
                            -1
                        } else if sa == i32::MIN && sb == -1 {
                            sa
                        } else {
                            sa / sb
                        }
                    }
                    (1, 5) => {
                        if b == 0 {
                            -1
                        } else {
                            (a / b) as i32
                        }
                    }
                    (1, 6) => {
                        if sb == 0 {
                            sa
                        } else if sa == i32::MIN && sb == -1 {
                            0
                        } else {
                            sa % sb
                        }
                    }
                    (1, 7) => {
                        if b == 0 {
                            sa
                        } else {
                            (a % b) as i32
                        }
                    }
                    _ => fail("an instruction the machine does not have", pc),
                };
                self.set(rd, v as i64 as u64);
            }
            0x0f => {}
            // A, with one hart: the orderings change nothing.
            0x2f => {
                let n = match f3 {
                    2 => 4,
                    3 => 8,
                    _ => fail("an instruction the machine does not have", pc),
                };
                let a = x1;
                let narrow = |v: u64| if n == 4 { sext(v, 32) as u64 } else { v };
                let f5 = bits(w, 31, 27);
                if f5 == 0b00010 {
                    self.reserved = a + 1;
                    let v = narrow(self.load(a, n));
                    self.set(rd, v);
                } else if f5 == 0b00011 {
                    let ok = self.reserved == a + 1;
                    self.reserved = 0;
                    if ok {
                        self.store(a, n, x2);
                    }
                    self.set(rd, !ok as u64);
                } else {
                    let old = narrow(self.load(a, n));
                    let src = narrow(x2);
                    let (so, ss) = (old as i64, src as i64);
                    let new = match f5 {
                        0b00001 => src,
                        0b00000 => old.wrapping_add(src),
                        0b00100 => old ^ src,
                        0b01100 => old & src,
                        0b01000 => old | src,
                        0b10000 => (if so < ss { so } else { ss }) as u64,
                        0b10100 => (if so > ss { so } else { ss }) as u64,
                        0b11000 => if old < src { old } else { src },
                        0b11100 => if old > src { old } else { src },
                        _ => fail("an instruction the machine does not have", pc),
                    };
                    self.store(a, n, new);
                    self.set(rd, old);
                }
            }
            0x07 => {
                let a = x1.wrapping_add(imm_i as u64);
                match f3 {
                    2 => self.f[rd as usize] = 0xffff_ffff_0000_0000 | self.load(a, 4),
                    3 => self.f[rd as usize] = self.load(a, 8),
                    _ => fail("an instruction the machine does not have", pc),
                }
            }
            0x27 => {
                let a = x1.wrapping_add(imm_s as u64);
                match f3 {
                    2 => self.store(a, 4, self.f[rs2 as usize]),
                    3 => self.store(a, 8, self.f[rs2 as usize]),
                    _ => fail("an instruction the machine does not have", pc),
                }
            }
            0x53 => self.fp_op(rd, rs1, rs2, f3, f7),
            _ => fail("an instruction the machine does not have", pc),
        }
        self.pc = next;
    }

    /// The F and D operations, but for square roots and fused ones;
    /// conversions to integers round towards zero.
    fn fp_op(&mut self, rd: u32, rs1: u32, rs2: u32, f3: u32, f7: u32) {
        let double = f7 & 3 == 1;
        let x1 = self.x[rs1 as usize];
        let pc = self.pc;
        match f7 >> 2 {
            0b00000 | 0b00001 | 0b00010 | 0b00011 => {
                if double {
                    let (a, b) = (self.fd(rs1), self.fd(rs2));
                    let r = match f7 >> 2 {
                        0b00000 => a + b,
                        0b00001 => a - b,
                        0b00010 => a * b,
                        _ => a / b,
                    };
                    self.set_d(rd, r);
                } else {
                    let (a, b) = (self.fs(rs1), self.fs(rs2));
                    let r = match f7 >> 2 {
                        0b00000 => a + b,
                        0b00001 => a - b,
                        0b00010 => a * b,
                        _ => a / b,
                    };
                    self.set_s(rd, r);
                }
            }
            0b00100 => {
                let (a, b) = (self.f[rs1 as usize], self.f[rs2 as usize]);
                let sign = if double { 1u64 << 63 } else { 1u64 << 31 };
                let s = match f3 {
                    0 => b & sign,
                    1 => !b & sign,
                    2 => (a ^ b) & sign,
                    _ => fail("an instruction the machine does not have", pc),
                };
                let v = (a & !sign) | s;
                self.f[rd as usize] = if double { v } else { 0xffff_ffff_0000_0000 | (v & 0xffff_ffff) };
            }
            0b01000 => {
                if double {
                    let v = self.fs(rs1) as f64;
                    self.set_d(rd, v);
                } else {
                    let v = self.fd(rs1) as f32;
                    self.set_s(rd, v);
                }
            }
            0b10100 => {
                let (a, b) = if double { (self.fd(rs1), self.fd(rs2)) } else { (self.fs(rs1) as f64, self.fs(rs2) as f64) };
                let r = match f3 {
                    2 => a == b,
                    1 => a < b,
                    0 => a <= b,
                    _ => fail("an instruction the machine does not have", pc),
                };
                self.set(rd, r as u64);
            }
            0b11000 => {
                let v = if double { self.fd(rs1) } else { self.fs(rs1) as f64 };
                let r = match rs2 {
                    0 => v as i32 as i64 as u64,
                    1 => v as u32 as i32 as i64 as u64,
                    2 => v as i64 as u64,
                    _ => v as u64,
                };
                self.set(rd, r);
            }
            0b11010 => {
                if double {
                    let v = match rs2 {
                        0 => x1 as i32 as f64,
                        1 => x1 as u32 as f64,
                        2 => x1 as i64 as f64,
                        _ => x1 as f64,
                    };
                    self.set_d(rd, v);
                } else {
                    let v = match rs2 {
                        0 => x1 as i32 as f32,
                        1 => x1 as u32 as f32,
                        2 => x1 as i64 as f32,
                        _ => x1 as f32,
                    };
                    self.set_s(rd, v);
                }
            }
            0b11100 if f3 == 0 => {
                let v = if double { self.f[rs1 as usize] } else { sext(self.f[rs1 as usize] & 0xffff_ffff, 32) as u64 };
                self.set(rd, v);
            }
            0b11110 if f3 == 0 => {
                self.f[rd as usize] = if double { x1 } else { 0xffff_ffff_0000_0000 | (x1 & 0xffff_ffff) };
            }
            _ => fail("a floating-point instruction the runtime does not have", pc),
        }
    }
}

/// Runs kernel `kernel` (its symbol, `kernel_len` bytes) of `image` on
/// `nargs` arguments: each a `u64` of `args`, of the kind the same element
/// of `kinds` says (0 an integer or pointer, 1 an `f32`, 2 an `f64`).
/// Gives a0, or fa0 if `ret` is 1 or 2.
///
/// # Safety
/// The pointers are to what the compiler made for the call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn __epitome_launch(image: *const u8, kernel: *const u8, kernel_len: usize, args: *const u64, kinds: *const u8, nargs: usize, ret: u8) -> u64 {
    let m = unsafe { &mut *(&raw mut MACHINE) };
    if m.mem.is_null() {
        m.mem = unsafe { calloc(MEM, 1) };
        if m.mem.is_null() {
            fail("no memory for the machine", MEM as u64);
        }
    }
    // The image: `EPIM`, machine, base, length, symbols, contents.
    let head = unsafe { core::slice::from_raw_parts(image, 6) };
    if &head[..4] != b"EPIM" {
        fail("not an image", 0);
    }
    let name_len = head[4] as usize | (head[5] as usize) << 8;
    let mut at = 6 + name_len;
    let fixed = unsafe { core::slice::from_raw_parts(image, at + 20) };
    let base = le64(fixed, at);
    let len = le64(fixed, at + 8) as usize;
    let nsyms = u32::from_le_bytes([fixed[at + 16], fixed[at + 17], fixed[at + 18], fixed[at + 19]]) as usize;
    at += 20;
    let want = unsafe { core::slice::from_raw_parts(kernel, kernel_len) };
    let mut entry = 0u64;
    for _ in 0..nsyms {
        let h = unsafe { core::slice::from_raw_parts(image.add(at), 2) };
        let n = h[0] as usize | (h[1] as usize) << 8;
        let sym = unsafe { core::slice::from_raw_parts(image.add(at + 2), n + 8) };
        if &sym[..n] == want {
            entry = le64(sym, n);
        }
        at += 2 + n + 8;
    }
    if entry == 0 {
        fail("no such kernel in the image", 0);
    }
    if base as usize + len > MEM - (1 << 20) {
        fail("an image too large for the machine", len as u64);
    }
    let contents = unsafe { core::slice::from_raw_parts(image.add(at), len) };
    m.mem()[base as usize..base as usize + len].copy_from_slice(contents);
    m.x = [0; 32];
    m.f = [0; 32];
    m.reserved = 0;
    let (mut ni, mut nf) = (0, 0);
    for i in 0..nargs {
        let v = unsafe { *args.add(i) };
        match unsafe { *kinds.add(i) } {
            0 => {
                m.x[10 + ni] = v;
                ni += 1;
            }
            1 => {
                m.f[10 + nf] = 0xffff_ffff_0000_0000 | (v & 0xffff_ffff);
                nf += 1;
            }
            _ => {
                m.f[10 + nf] = v;
                nf += 1;
            }
        }
        if ni > 8 || nf > 8 {
            fail("more arguments than registers", i as u64);
        }
    }
    m.x[2] = (MEM as u64 - 4096) & !15;
    m.x[1] = RETURN;
    m.pc = entry;
    loop {
        if m.pc == RETURN {
            break;
        }
        if m.pc >= STUBS && m.pc < STUBS + 16 * STUB_NAMES.len() as u64 {
            m.stub((m.pc - STUBS) / 16);
            m.pc = m.x[1];
            continue;
        }
        m.step();
    }
    match ret {
        1 => m.f[10] & 0xffff_ffff,
        2 => m.f[10],
        _ => m.x[10],
    }
}
