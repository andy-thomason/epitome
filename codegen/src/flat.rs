//! Flat images for machines with no loader: RISC-V RV64 objects linked at
//! one base address into a single block of memory (code, data and zeroed
//! space), with every global symbol's address, ready to copy into the
//! machine's memory and run.
//!
//! A symbol no object defines is given a stub address, one of `stubs` in
//! order, from `stub_base` on, 16 bytes apart: the machine's runtime does
//! what a call there means (`putchar`, `memcpy`) and returns.
//!
//! The image is written as bytes (`FlatImage::encode`) for a host program
//! to carry: `EPIM`, the machine's name, the base, the size in memory,
//! the symbols, then the contents.

use crate::elf::{self, ElfFile};
use std::collections::HashMap;

/// A linked image.
#[derive(Clone, Debug)]
pub struct FlatImage {
    /// Where it is loaded.
    pub base: u64,
    /// Its contents, from `base`: code and data, then zeroes for what has
    /// no contents (`.bss`), to its full size.
    pub bytes: Vec<u8>,
    /// Each global symbol (stubs included) and its address.
    pub symbols: Vec<(String, u64)>,
}

/// Links RV64 relocatable `objects` at `base`.
pub fn link_riscv64(objects: &[Vec<u8>], base: u64, stub_base: u64, stubs: &[&str]) -> Result<FlatImage, String> {
    let files: Vec<ElfFile> = objects.iter().map(|b| elf::read(b)).collect::<Result<_, _>>()?;
    // Every allocated section, in order; unwind tables are left out
    // (nothing on the machine unwinds).
    let mut at = base;
    let mut place: HashMap<(usize, usize), u64> = HashMap::new();
    for (fi, f) in files.iter().enumerate() {
        for (si, s) in f.sections.iter().enumerate() {
            if s.flags & elf::SHF_ALLOC == 0 || s.name == ".eh_frame" {
                continue;
            }
            if s.flags & 0x400 != 0 {
                return Err(format!("thread-local section {} on a machine without threads", s.name));
            }
            at = at.next_multiple_of(s.align.max(1));
            place.insert((fi, si), at);
            at += s.size;
        }
    }
    let mut bytes = vec![0u8; (at - base) as usize];
    for (fi, f) in files.iter().enumerate() {
        for (si, s) in f.sections.iter().enumerate() {
            if let Some(&a) = place.get(&(fi, si))
                && s.sh_type != elf::SHT_NOBITS
            {
                let o = (a - base) as usize;
                bytes[o..o + s.data.len()].copy_from_slice(&s.data);
            }
        }
    }
    let mut globals: HashMap<String, u64> = HashMap::new();
    let mut order: Vec<String> = Vec::new();
    for (fi, f) in files.iter().enumerate() {
        for s in &f.symbols {
            if s.bind != elf::STB_LOCAL && s.shndx != 0 && !s.name.is_empty() {
                let b = place.get(&(fi, s.shndx as usize)).ok_or_else(|| format!("{} is in a section not loaded", s.name))?;
                if globals.insert(s.name.clone(), b + s.value).is_none() {
                    order.push(s.name.clone());
                }
            }
        }
    }
    for (k, name) in stubs.iter().enumerate() {
        if !globals.contains_key(*name) {
            globals.insert(name.to_string(), stub_base + 16 * k as u64);
            order.push(name.to_string());
        }
    }
    let read32 = |b: &[u8], o: usize| u32::from_le_bytes(b[o..o + 4].try_into().unwrap());
    let bits = |w: u32, hi: u32, lo: u32| (w >> lo) & ((1u32 << (hi - lo + 1)) - 1);
    for (fi, f) in files.iter().enumerate() {
        for (si, s) in f.sections.iter().enumerate() {
            let Some(&sbase) = place.get(&(fi, si)) else { continue };
            for r in &s.relocs {
                let sym = &f.symbols[r.sym as usize];
                let sv = if sym.shndx == 0 {
                    *globals.get(&sym.name).ok_or_else(|| format!("undefined symbol {}", sym.name))?
                } else {
                    place.get(&(fi, sym.shndx as usize)).ok_or("a symbol in a section not loaded")? + sym.value
                };
                let target = sv.wrapping_add(r.addend as u64);
                let p = sbase + r.offset;
                let o = (p - base) as usize;
                let insn = if o + 4 <= bytes.len() { read32(&bytes, o) } else { 0 };
                match r.rel_type {
                    // R_RISCV_64
                    2 => bytes[o..o + 8].copy_from_slice(&target.to_le_bytes()),
                    // R_RISCV_BRANCH
                    16 => {
                        let off = target.wrapping_sub(p) as i64;
                        if !(-(1 << 12)..(1 << 12)).contains(&off) {
                            return Err(format!("a branch to {} out of range", sym.name));
                        }
                        let v = off as u32;
                        let imm = (bits(v, 12, 12) << 31) | (bits(v, 10, 5) << 25) | (bits(v, 4, 1) << 8) | (bits(v, 11, 11) << 7);
                        write32(&mut bytes, o, (insn & 0x01ff_f07f) | imm);
                    }
                    // R_RISCV_JAL
                    17 => {
                        let off = target.wrapping_sub(p) as i64;
                        if !(-(1 << 20)..(1 << 20)).contains(&off) {
                            return Err(format!("a jump to {} out of range", sym.name));
                        }
                        let v = off as u32;
                        let imm = (bits(v, 20, 20) << 31) | (bits(v, 10, 1) << 21) | (bits(v, 11, 11) << 20) | (bits(v, 19, 12) << 12);
                        write32(&mut bytes, o, (insn & 0xfff) | imm);
                    }
                    // R_RISCV_CALL / CALL_PLT: auipc then jalr.
                    18 | 19 => {
                        let off = target.wrapping_sub(p);
                        let hi = (off.wrapping_add(0x800) >> 12) as u32 & 0xfffff;
                        let lo = off as u32 & 0xfff;
                        let next = read32(&bytes, o + 4);
                        write32(&mut bytes, o, (insn & 0xfff) | (hi << 12));
                        bytes[o + 4..o + 8].copy_from_slice(&((next & 0x000f_ffff) | (lo << 20)).to_le_bytes());
                    }
                    // R_RISCV_HI20, LO12_I, LO12_S
                    26 => write32(&mut bytes, o, (insn & 0xfff) | ((target.wrapping_add(0x800) >> 12) as u32 & 0xfffff) << 12),
                    27 => write32(&mut bytes, o, (insn & 0x000f_ffff) | ((target as u32 & 0xfff) << 20)),
                    28 => {
                        let lo = target as u32 & 0xfff;
                        write32(&mut bytes, o, (insn & 0x01ff_f07f) | ((lo >> 5) << 25) | ((lo & 31) << 7));
                    }
                    // R_RISCV_RELAX: nothing is relaxed.
                    51 => {}
                    t => return Err(format!("RISC-V relocation type {t} is not supported in a flat image")),
                }
            }
        }
    }
    let symbols = order.into_iter().map(|n| {
        let a = globals[&n];
        (n, a)
    });
    Ok(FlatImage { base, bytes, symbols: symbols.collect() })
}

fn write32(bytes: &mut [u8], o: usize, v: u32) {
    bytes[o..o + 4].copy_from_slice(&v.to_le_bytes());
}

impl FlatImage {
    /// The image as bytes, for machine `machine`: `EPIM`, then, each
    /// little-endian, the machine's name (`u16` length, bytes), the base
    /// (`u64`), the contents' length (`u64`), the number of symbols
    /// (`u32`) and each symbol (`u16` length, name, `u64` address), then
    /// the contents.
    pub fn encode(&self, machine: &str) -> Vec<u8> {
        let mut out = b"EPIM".to_vec();
        out.extend_from_slice(&(machine.len() as u16).to_le_bytes());
        out.extend_from_slice(machine.as_bytes());
        out.extend_from_slice(&self.base.to_le_bytes());
        out.extend_from_slice(&(self.bytes.len() as u64).to_le_bytes());
        out.extend_from_slice(&(self.symbols.len() as u32).to_le_bytes());
        for (n, a) in &self.symbols {
            out.extend_from_slice(&(n.len() as u16).to_le_bytes());
            out.extend_from_slice(n.as_bytes());
            out.extend_from_slice(&a.to_le_bytes());
        }
        out.extend_from_slice(&self.bytes);
        out
    }
}
