//! Tools for the iCESugar's demonstration computer
//! (`sv/arch/ice40-up5k/flow/rv32i.sv`): an assembler for RV32I, the
//! RISC-V base integer instructions, and an emulator to check the board
//! against. Both are written from the RISC-V unprivileged specification's
//! instruction formats and are only as complete as the demonstration needs.
//!
//! The computer's memory map, which the emulator shares:
//!
//! - `0x0000_0000` to `MEM_BYTES - 1`: memory, the program's image loaded
//!   at 0, where it starts.
//! - `0x8000_0000` (load): bit 0 is set while the UART transmitter is busy,
//!   bit 1 while a received byte is waiting. (store): sends the low byte.
//! - `0x8000_0004` (load): the received byte, which then stops waiting.
//! - `0x8000_0008` (store): the LED, bit 0.

use crate::hash::FxMap;
use std::collections::VecDeque;

pub const MEM_BYTES: usize = 8192;
pub const IO_BASE: u32 = 0x8000_0000;

/// An assembled program: its image from address 0, and its labels.
pub struct Program {
    pub image: Vec<u8>,
    pub labels: FxMap<String, u32>,
}

impl Program {
    /// The image as little-endian words.
    pub fn words(&self) -> Vec<u32> {
        self.image.chunks(4).map(|c| c.iter().rev().fold(0, |a, &b| a << 8 | b as u32)).collect()
    }

    /// The image as an assignment pattern for a memory of words, the
    /// words that are not zero by index: the memory's initialiser, to be
    /// `` `include ``d by the design.
    pub fn to_sv(&self, source: &str) -> String {
        let mut s = format!(
            "// The program's image, assembled from {source} by\n// `cargo run --release -p epitome-sv --example ice40_rv32i`: do not edit.\n'{{\n"
        );
        for (i, w) in self.words().iter().enumerate().filter(|(_, w)| **w != 0) {
            s.push_str(&format!("  {i}: 32'h{w:08x},\n"));
        }
        s.push_str("  default: 32'h0\n}\n");
        s
    }
}

fn reg(s: &str) -> Option<u32> {
    const ABI: [&str; 32] = [
        "zero", "ra", "sp", "gp", "tp", "t0", "t1", "t2", "s0", "s1", "a0", "a1", "a2", "a3", "a4", "a5", "a6", "a7",
        "s2", "s3", "s4", "s5", "s6", "s7", "s8", "s9", "s10", "s11", "t3", "t4", "t5", "t6",
    ];
    if s == "fp" {
        return Some(8);
    }
    if let Some(n) = s.strip_prefix('x').and_then(|n| n.parse::<u32>().ok()) {
        return (n < 32).then_some(n);
    }
    ABI.iter().position(|&r| r == s).map(|n| n as u32)
}

/// The operands of a line: split at commas outside quotes.
fn operands(s: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut quote = None;
    let mut esc = false;
    for c in s.chars() {
        match quote {
            Some(q) => {
                cur.push(c);
                if esc {
                    esc = false;
                } else if c == '\\' {
                    esc = true;
                } else if c == q {
                    quote = None;
                }
            }
            None if c == '"' || c == '\'' => {
                quote = Some(c);
                cur.push(c);
            }
            None if c == ',' => out.push(std::mem::take(&mut cur).trim().to_string()),
            None => cur.push(c),
        }
    }
    if !cur.trim().is_empty() {
        out.push(cur.trim().to_string());
    }
    out
}

/// The text of a quoted string or character, escapes undone.
fn unquote(s: &str) -> Option<Vec<u8>> {
    let inner = s.get(1..s.len().checked_sub(1)?)?;
    let mut out = Vec::new();
    let mut it = inner.bytes();
    while let Some(c) = it.next() {
        if c == b'\\' {
            out.push(match it.next()? {
                b'n' => b'\n',
                b'r' => b'\r',
                b't' => b'\t',
                b'0' => 0,
                c => c,
            });
        } else {
            out.push(c);
        }
    }
    Some(out)
}

/// A line's code without its comment (`#` outside quotes) or labels.
fn strip(line: &str) -> String {
    let mut quote = None;
    let mut esc = false;
    for (i, c) in line.char_indices() {
        match quote {
            Some(q) => {
                if esc {
                    esc = false;
                } else if c == '\\' {
                    esc = true;
                } else if c == q {
                    quote = None;
                }
            }
            None if c == '"' || c == '\'' => quote = Some(c),
            None if c == '#' => return line[..i].to_string(),
            None => {}
        }
    }
    line.to_string()
}

struct Asm<'a> {
    labels: &'a FxMap<String, u32>,
    /// The first pass: labels may not be known yet.
    sizing: bool,
}

impl Asm<'_> {
    /// A number, a character, a label, or `a+b` / `a-b` of those.
    fn value(&self, s: &str) -> Result<i64, String> {
        let s = s.trim();
        if let Some(i) = s.rfind(['+', '-']).filter(|&i| i > 0 && !s.starts_with('\'')) {
            let (a, b) = (self.value(&s[..i])?, self.value(&s[i + 1..])?);
            return Ok(if &s[i..i + 1] == "+" { a + b } else { a - b });
        }
        if let Some(r) = s.strip_prefix('-') {
            return Ok(-self.value(r)?);
        }
        if s.starts_with('\'') {
            return match unquote(s).as_deref() {
                Some([c]) => Ok(*c as i64),
                _ => Err(format!("bad character {s}")),
            };
        }
        if let Some(h) = s.strip_prefix("0x") {
            return i64::from_str_radix(&h.replace('_', ""), 16).map_err(|_| format!("bad number {s}"));
        }
        if s.starts_with(|c: char| c.is_ascii_digit()) {
            return s.replace('_', "").parse().map_err(|_| format!("bad number {s}"));
        }
        match self.labels.get(s) {
            Some(&v) => Ok(v as i64),
            None if self.sizing => Ok(0),
            None => Err(format!("no label {s}")),
        }
    }

    fn reg(&self, s: &str) -> Result<u32, String> {
        reg(s.trim()).ok_or_else(|| format!("not a register: {s}"))
    }

    /// `imm(reg)`.
    fn mem(&self, s: &str) -> Result<(i64, u32), String> {
        let (imm, r) = s.trim().strip_suffix(')').and_then(|s| s.split_once('(')).ok_or_else(|| format!("expected imm(reg): {s}"))?;
        Ok((if imm.trim().is_empty() { 0 } else { self.value(imm)? }, self.reg(r)?))
    }
}

fn fits(v: i64, bits: u32) -> bool {
    v >= -(1 << (bits - 1)) && v < 1 << (bits - 1)
}

fn r_type(f7: u32, rs2: u32, rs1: u32, f3: u32, rd: u32, op: u32) -> u32 {
    f7 << 25 | rs2 << 20 | rs1 << 15 | f3 << 12 | rd << 7 | op
}

fn i_type(imm: i64, rs1: u32, f3: u32, rd: u32, op: u32) -> u32 {
    ((imm as u32) & 0xfff) << 20 | rs1 << 15 | f3 << 12 | rd << 7 | op
}

fn s_type(imm: i64, rs2: u32, rs1: u32, f3: u32, op: u32) -> u32 {
    let i = imm as u32;
    (i >> 5 & 0x7f) << 25 | rs2 << 20 | rs1 << 15 | f3 << 12 | (i & 0x1f) << 7 | op
}

fn b_type(off: i64, rs2: u32, rs1: u32, f3: u32) -> u32 {
    let i = off as u32;
    (i >> 12 & 1) << 31 | (i >> 5 & 0x3f) << 25 | rs2 << 20 | rs1 << 15 | f3 << 12 | (i >> 1 & 0xf) << 8 | (i >> 11 & 1) << 7 | 0x63
}

fn j_type(off: i64, rd: u32) -> u32 {
    let i = off as u32;
    (i >> 20 & 1) << 31 | (i >> 1 & 0x3ff) << 21 | (i >> 11 & 1) << 20 | (i >> 12 & 0xff) << 12 | rd << 7 | 0x6f
}

/// `li`'s instructions: `lui` for the upper bits (rounded so that the
/// `addi` of the lower twelve, sign-extended, makes the value), then `addi`.
fn li(rd: u32, v: i64) -> Vec<u32> {
    let v = v as i32 as i64;
    if fits(v, 12) {
        return vec![i_type(v, 0, 0, rd, 0x13)];
    }
    let lo = (v << 52) >> 52;
    let hi = ((v - lo) as u32) & 0xffff_f000;
    let mut out = vec![hi | rd << 7 | 0x37];
    if lo != 0 {
        out.push(i_type(lo, rd, 0, rd, 0x13));
    }
    out
}

/// One instruction or pseudo-instruction at `pc`.
fn encode(a: &Asm, pc: u32, op: &str, o: &[String]) -> Result<Vec<u32>, String> {
    let n = |k: usize| -> Result<&str, String> { o.get(k).map(|s| s.as_str()).ok_or_else(|| format!("{op}: too few operands")) };
    let want = |k: usize| -> Result<(), String> {
        if o.len() != k { Err(format!("{op}: {k} operands expected")) } else { Ok(()) }
    };
    let rel = |s: &str| -> Result<i64, String> { Ok(a.value(s)? - pc as i64) };
    let imm12 = |v: i64| -> Result<i64, String> {
        if fits(v, 12) || a.sizing { Ok(v) } else { Err(format!("{op}: {v} does not fit 12 bits")) }
    };
    let branch = |f3: u32, rs1: &str, rs2: &str, to: &str| -> Result<Vec<u32>, String> {
        let off = rel(to)?;
        if !a.sizing && (!fits(off, 13) || off & 1 != 0) {
            return Err(format!("{op}: branch to {to} too far"));
        }
        Ok(vec![b_type(off, a.reg(rs2)?, a.reg(rs1)?, f3)])
    };
    let alu = ["add", "sll", "slt", "sltu", "xor", "srl", "or", "and"];
    let alui = ["addi", "slli", "slti", "sltiu", "xori", "srli", "ori", "andi"];
    let loads = ["lb", "lh", "lw", "", "lbu", "lhu"];
    let stores = ["sb", "sh", "sw"];
    let branches = [("beq", 0), ("bne", 1), ("blt", 4), ("bge", 5), ("bltu", 6), ("bgeu", 7)];
    Ok(match op {
        _ if alu.contains(&op) || op == "sub" || op == "sra" => {
            want(3)?;
            let f3 = match op {
                "sub" => 0,
                "sra" => 5,
                _ => alu.iter().position(|&x| x == op).unwrap() as u32,
            };
            let f7 = if op == "sub" || op == "sra" { 0x20 } else { 0 };
            vec![r_type(f7, a.reg(n(2)?)?, a.reg(n(1)?)?, f3, a.reg(n(0)?)?, 0x33)]
        }
        "slli" | "srli" | "srai" => {
            want(3)?;
            let sh = a.value(n(2)?)?;
            if !(0..32).contains(&sh) {
                return Err(format!("{op}: shift {sh}"));
            }
            let f3 = if op == "slli" { 1 } else { 5 };
            let f7 = if op == "srai" { 0x20 } else { 0 };
            vec![r_type(f7, sh as u32, a.reg(n(1)?)?, f3, a.reg(n(0)?)?, 0x13)]
        }
        _ if alui.contains(&op) => {
            want(3)?;
            let f3 = alui.iter().position(|&x| x == op).unwrap() as u32;
            vec![i_type(imm12(a.value(n(2)?)?)?, a.reg(n(1)?)?, f3, a.reg(n(0)?)?, 0x13)]
        }
        _ if loads.contains(&op) && !op.is_empty() => {
            want(2)?;
            let (imm, rs1) = a.mem(n(1)?)?;
            let f3 = loads.iter().position(|&x| x == op).unwrap() as u32;
            vec![i_type(imm12(imm)?, rs1, f3, a.reg(n(0)?)?, 0x03)]
        }
        _ if stores.contains(&op) => {
            want(2)?;
            let (imm, rs1) = a.mem(n(1)?)?;
            let f3 = stores.iter().position(|&x| x == op).unwrap() as u32;
            vec![s_type(imm12(imm)?, a.reg(n(0)?)?, rs1, f3, 0x23)]
        }
        _ if branches.iter().any(|b| b.0 == op) => {
            want(3)?;
            branch(branches.iter().find(|b| b.0 == op).unwrap().1, n(0)?, n(1)?, n(2)?)?
        }
        "bgt" | "ble" | "bgtu" | "bleu" => {
            want(3)?;
            let f3 = match op {
                "bgt" => 4,
                "ble" => 5,
                "bgtu" => 6,
                _ => 7,
            };
            branch(f3, n(1)?, n(0)?, n(2)?)?
        }
        "beqz" | "bnez" | "bltz" | "bgez" => {
            want(2)?;
            let f3 = match op {
                "beqz" => 0,
                "bnez" => 1,
                "bltz" => 4,
                _ => 5,
            };
            branch(f3, n(0)?, "zero", n(1)?)?
        }
        "blez" | "bgtz" => {
            want(2)?;
            branch(if op == "blez" { 5 } else { 4 }, "zero", n(0)?, n(1)?)?
        }
        "lui" | "auipc" => {
            want(2)?;
            let v = a.value(n(1)?)?;
            vec![((v as u32) << 12) | a.reg(n(0)?)? << 7 | if op == "lui" { 0x37 } else { 0x17 }]
        }
        "jal" if o.len() == 1 => vec![j_type(rel(n(0)?)?, 1)],
        "jal" => {
            want(2)?;
            vec![j_type(rel(n(1)?)?, a.reg(n(0)?)?)]
        }
        "jalr" if o.len() == 1 => vec![i_type(0, a.reg(n(0)?)?, 0, 1, 0x67)],
        "jalr" => {
            want(2)?;
            let (imm, rs1) = a.mem(n(1)?)?;
            vec![i_type(imm12(imm)?, rs1, 0, a.reg(n(0)?)?, 0x67)]
        }
        "j" => vec![j_type(rel(n(0)?)?, 0)],
        "call" => vec![j_type(rel(n(0)?)?, 1)],
        "jr" => vec![i_type(0, a.reg(n(0)?)?, 0, 0, 0x67)],
        "ret" => vec![i_type(0, 1, 0, 0, 0x67)],
        "nop" => vec![i_type(0, 0, 0, 0, 0x13)],
        "mv" => vec![i_type(0, a.reg(n(1)?)?, 0, a.reg(n(0)?)?, 0x13)],
        "not" => vec![i_type(-1, a.reg(n(1)?)?, 4, a.reg(n(0)?)?, 0x13)],
        "neg" => vec![r_type(0x20, a.reg(n(1)?)?, 0, 0, a.reg(n(0)?)?, 0x33)],
        "seqz" => vec![i_type(1, a.reg(n(1)?)?, 3, a.reg(n(0)?)?, 0x13)],
        "snez" => vec![r_type(0, a.reg(n(1)?)?, 0, 3, a.reg(n(0)?)?, 0x33)],
        // Its size from the value, known in the first pass: a constant.
        "li" => {
            want(2)?;
            li(a.reg(n(0)?)?, a.value(n(1)?)?)
        }
        // Always two instructions: a label may not be known yet.
        "la" => {
            want(2)?;
            let rd = a.reg(n(0)?)?;
            let mut v = li(rd, a.value(n(1)?)?);
            if v.len() == 1 {
                v.insert(0, 0x37 | rd << 7);
                v[1] = i_type(a.value(n(1)?)?, rd, 0, rd, 0x13);
            }
            v
        }
        "fence" => vec![0x0ff0_000f],
        "ecall" => vec![0x73],
        "ebreak" => vec![0x0010_0073],
        _ => return Err(format!("unknown instruction {op}")),
    })
}

/// Assembles `text`: one instruction, directive or label per line
/// (`label:` may lead a line), `#` comments. Directives: `.word`, `.half`,
/// `.byte`, `.ascii`, `.asciz`, `.space N`, `.align N` (to 2^N bytes),
/// `.equ NAME, VALUE`.
pub fn assemble(text: &str) -> Result<Program, String> {
    let mut labels: FxMap<String, u32> = FxMap::default();
    for pass in 0..2 {
        let a = Asm { labels: &labels.clone(), sizing: pass == 0 };
        let mut image: Vec<u8> = Vec::new();
        let mut found: FxMap<String, u32> = FxMap::default();
        for (ln, line) in text.lines().enumerate() {
            let err = |e: String| format!("line {}: {e}: {}", ln + 1, line.trim());
            let mut code = strip(line).trim().to_string();
            while let Some(i) = code.find(':').filter(|&i| code[..i].chars().all(|c| c.is_alphanumeric() || c == '_' || c == '.')) {
                found.insert(code[..i].to_string(), image.len() as u32);
                code = code[i + 1..].trim().to_string();
            }
            if code.is_empty() {
                continue;
            }
            let (op, rest) = code.split_once(char::is_whitespace).unwrap_or((&code, ""));
            let o = operands(rest);
            let a = Asm { labels: &a.labels.iter().chain(&found).map(|(k, v)| (k.clone(), *v)).collect(), sizing: a.sizing };
            match op {
                ".word" | ".half" | ".byte" => {
                    let size = match op {
                        ".word" => 4,
                        ".half" => 2,
                        _ => 1,
                    };
                    for v in &o {
                        let v = a.value(v).map_err(err)?;
                        image.extend(&(v as u32).to_le_bytes()[..size]);
                    }
                }
                ".ascii" | ".asciz" => {
                    for s in &o {
                        image.extend(unquote(s).ok_or_else(|| err(format!("bad string {s}")))?);
                    }
                    if op == ".asciz" {
                        image.push(0);
                    }
                }
                ".space" => image.extend(std::iter::repeat_n(0, a.value(&o[0]).map_err(err)? as usize)),
                ".align" => {
                    let to = 1usize << a.value(&o[0]).map_err(err)?;
                    while image.len() % to != 0 {
                        image.push(0);
                    }
                }
                ".equ" => {
                    let v = a.value(&o[1]).map_err(err)?;
                    found.insert(o[0].clone(), v as u32);
                }
                _ => {
                    if image.len() % 4 != 0 {
                        return Err(err("an instruction not on a word boundary".into()));
                    }
                    let pc = image.len() as u32;
                    for w in encode(&a, pc, op, &o).map_err(err)? {
                        image.extend(w.to_le_bytes());
                    }
                }
            }
        }
        while image.len() % 4 != 0 {
            image.push(0);
        }
        if pass == 1 {
            if image.len() > MEM_BYTES {
                return Err(format!("the program is {} bytes; memory is {MEM_BYTES}", image.len()));
            }
            return Ok(Program { image, labels: found });
        }
        labels = found;
    }
    unreachable!()
}

/// The computer, emulated an instruction at a time.
pub struct Emulator {
    pub x: [u32; 32],
    pub pc: u32,
    pub mem: Vec<u8>,
    /// Bytes the UART has received, not yet read.
    pub rx: VecDeque<u8>,
    /// Bytes the program has sent.
    pub tx: Vec<u8>,
    pub led: bool,
    pub steps: u64,
    /// Status reads since the last byte sent or received.
    polls: u32,
}

impl Emulator {
    pub fn new(p: &Program) -> Emulator {
        let mut mem = p.image.clone();
        mem.resize(MEM_BYTES, 0);
        Emulator { x: [0; 32], pc: 0, mem, rx: VecDeque::new(), tx: Vec::new(), led: false, steps: 0, polls: 0 }
    }

    fn load(&mut self, a: u32, size: u32) -> Result<u32, String> {
        if a >= IO_BASE {
            return Ok(match a - IO_BASE {
                0 => {
                    self.polls += 1;
                    (!self.rx.is_empty() as u32) << 1
                }
                4 => {
                    self.polls = 0;
                    self.rx.pop_front().unwrap_or(0) as u32
                }
                _ => 0,
            });
        }
        let a = a as usize;
        if a % size as usize != 0 || a + size as usize > MEM_BYTES {
            return Err(format!("load of {size} bytes at {a:#x}, pc {:#x}", self.pc));
        }
        Ok((0..size as usize).rev().fold(0, |v, k| v << 8 | self.mem[a + k] as u32))
    }

    fn store(&mut self, a: u32, size: u32, v: u32) -> Result<(), String> {
        if a >= IO_BASE {
            match a - IO_BASE {
                0 => {
                    self.polls = 0;
                    self.tx.push(v as u8);
                }
                8 => self.led = v & 1 != 0,
                _ => {}
            }
            return Ok(());
        }
        let a = a as usize;
        if a % size as usize != 0 || a + size as usize > MEM_BYTES {
            return Err(format!("store of {size} bytes at {a:#x}, pc {:#x}", self.pc));
        }
        for k in 0..size as usize {
            self.mem[a + k] = (v >> (8 * k)) as u8;
        }
        Ok(())
    }

    /// Whether the program is waiting for input: it has read the UART's
    /// status several times since it last sent or received a byte.
    pub fn waiting(&self) -> bool {
        self.rx.is_empty() && self.polls > 2
    }

    /// Runs one instruction.
    pub fn step(&mut self) -> Result<(), String> {
        let pc = self.pc;
        if pc as usize + 4 > MEM_BYTES || pc % 4 != 0 {
            return Err(format!("pc {pc:#x}"));
        }
        let i = self.load(pc, 4)?;
        let op = i & 0x7f;
        let rd = (i >> 7 & 31) as usize;
        let f3 = i >> 12 & 7;
        let a = self.x[(i >> 15 & 31) as usize];
        let b = self.x[(i >> 20 & 31) as usize];
        let imm_i = ((i as i32) >> 20) as u32;
        let imm_s = (((i as i32) >> 25) << 5) as u32 | (i >> 7 & 31);
        let imm_b = (((i as i32) >> 31) << 12) as u32 | (i >> 7 & 1) << 11 | (i >> 25 & 0x3f) << 5 | (i >> 8 & 0xf) << 1;
        let imm_u = i & 0xffff_f000;
        let imm_j = (((i as i32) >> 31) << 20) as u32 | (i & 0xff000) | (i >> 20 & 1) << 11 | (i >> 21 & 0x3ff) << 1;
        let alu = |f3: u32, a: u32, b: u32, alt: bool| -> u32 {
            match f3 {
                0 if alt => a.wrapping_sub(b),
                0 => a.wrapping_add(b),
                1 => a << (b & 31),
                2 => ((a as i32) < (b as i32)) as u32,
                3 => (a < b) as u32,
                4 => a ^ b,
                5 if alt => ((a as i32) >> (b & 31)) as u32,
                5 => a >> (b & 31),
                6 => a | b,
                _ => a & b,
            }
        };
        let mut next = pc.wrapping_add(4);
        let mut wb: Option<u32> = None;
        match op {
            0x37 => wb = Some(imm_u),
            0x17 => wb = Some(pc.wrapping_add(imm_u)),
            0x6f => {
                wb = Some(next);
                next = pc.wrapping_add(imm_j);
            }
            0x67 => {
                wb = Some(next);
                next = a.wrapping_add(imm_i) & !1;
            }
            0x63 => {
                let take = match f3 {
                    0 => a == b,
                    1 => a != b,
                    4 => (a as i32) < (b as i32),
                    5 => (a as i32) >= (b as i32),
                    6 => a < b,
                    7 => a >= b,
                    _ => return Err(format!("bad branch at {pc:#x}")),
                };
                if take {
                    next = pc.wrapping_add(imm_b);
                }
            }
            0x03 => {
                let ea = a.wrapping_add(imm_i);
                wb = Some(match f3 {
                    0 => self.load(ea, 1)? as i8 as i32 as u32,
                    1 => self.load(ea, 2)? as i16 as i32 as u32,
                    2 => self.load(ea, 4)?,
                    4 => self.load(ea, 1)?,
                    5 => self.load(ea, 2)?,
                    _ => return Err(format!("bad load at {pc:#x}")),
                });
            }
            0x23 => {
                let ea = a.wrapping_add(imm_s);
                match f3 {
                    0 => self.store(ea, 1, b)?,
                    1 => self.store(ea, 2, b)?,
                    2 => self.store(ea, 4, b)?,
                    _ => return Err(format!("bad store at {pc:#x}")),
                }
            }
            0x13 => wb = Some(alu(f3, a, imm_i, f3 == 5 && i >> 30 & 1 != 0)),
            0x33 => wb = Some(alu(f3, a, b, i >> 30 & 1 != 0)),
            0x0f | 0x73 => {}
            _ => return Err(format!("bad instruction {i:#010x} at {pc:#x}")),
        }
        if let Some(v) = wb {
            if rd != 0 {
                self.x[rd] = v;
            }
        }
        self.pc = next;
        self.steps += 1;
        Ok(())
    }

    /// Runs until the program waits for input, at most `limit`
    /// instructions.
    pub fn run(&mut self, limit: u64) -> Result<(), String> {
        for _ in 0..limit {
            if self.waiting() {
                return Ok(());
            }
            self.step()?;
        }
        Err(format!("still running after {limit} instructions, pc {:#x}", self.pc))
    }

    /// What the program sends, at the start and after each line of
    /// `lines` (each sent with a CR) once it waits for more.
    pub fn session(p: &Program, lines: &[&[u8]]) -> Result<Vec<Vec<u8>>, String> {
        let mut e = Emulator::new(p);
        let mut out = Vec::new();
        e.run(50_000_000)?;
        out.push(std::mem::take(&mut e.tx));
        for l in lines {
            e.rx.extend(l.iter().copied());
            e.rx.push_back(b'\r');
            e.run(50_000_000)?;
            out.push(std::mem::take(&mut e.tx));
        }
        Ok(out)
    }
}
