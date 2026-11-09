//! Loading compiled code into this process, to be run here: no object
//! file, no linker, no `dlopen`.
//!
//! [`load`] takes what [`crate::compile`] made (or [`load_object`] an
//! [`Object`]), lays its sections out in fresh memory, fills in the
//! relocations, and gives back the functions' and globals' addresses by
//! name. The memory goes when the [`Loaded`] is dropped.
//!
//! - Code goes in one place, read-only data (and a table of addresses, for
//!   references through the GOT) in a second and writable data in a
//!   third, each on pages of its own: code is made executable and not
//!   writable, read-only data not writable.
//! - A relocation fills in what a linker would: a pointer in data, a
//!   distance in a 32-bit field of an instruction (x86's `rip`-relative
//!   operands and calls, which are the field's own bytes), or an operand of
//!   an instruction the description encodes (AArch64's `adrp`, `add`,
//!   `ldr`, `bl`): it is decoded, given the operand the address needs and
//!   encoded again, as the assembler would have.
//! - Symbols the module does not define are looked up by the caller's
//!   resolver, given the name as the IR has it (Mach-O's `_` taken off).
//!   Calls to them go through a stub of this file's own, next to the code,
//!   which jumps to wherever the symbol is: a call's field reaches only so
//!   far.
//! - Unwind and exception tables are not loaded: nothing unwinds through
//!   the code (no exceptions thrown through it, no backtraces). Thread-local
//!   variables cannot be loaded.
//!
//! On an AArch64 Mac the code is on `MAP_JIT` pages, written with this
//! thread's write protection off (`pthread_jit_write_protect_np`) and the
//! instruction cache invalidated after; elsewhere it is written and then
//! `mprotect`ed to read and execute.

use std::collections::HashMap;
use std::ffi::{CString, c_char, c_int, c_void};

use crate::Compiled;
use crate::abi::Syntax;
use crate::machine::RelocUse;
use crate::object::{Object, RelocTarget, SectionKind};
use crate::target::{Kind, Target};

/// Code and data loaded into this process (see the top of this file).
pub struct Loaded {
    base: *mut u8,
    size: usize,
    /// Every symbol defined, by the name the object gives it, with its
    /// address. (A global one wins over a local one of the same name.)
    symbols: HashMap<String, usize>,
    syntax: Syntax,
}

// The code and read-only data are never written again; writable data is
// the compiled code's to look after, as any global's is.
unsafe impl Send for Loaded {}
unsafe impl Sync for Loaded {}

impl Loaded {
    /// The address of `name`, as the IR names it (or as the object does:
    /// Mach-O's `_name`).
    pub fn address(&self, name: &str) -> Option<*const u8> {
        let own = match self.syntax {
            Syntax::MachO => format!("_{name}"),
            Syntax::Elf => name.to_string(),
        };
        self.symbols.get(&own).or_else(|| self.symbols.get(name)).map(|&a| a as *const u8)
    }

    /// Function `name` as a function pointer of type `F` (as
    /// `unsafe extern "C" fn(*mut u8)`).
    ///
    /// # Safety
    ///
    /// `F` must be a function pointer type whose signature and calling
    /// convention are the compiled function's. It must not be called after
    /// `self` is dropped.
    pub unsafe fn function<F: Copy>(&self, name: &str) -> Option<F> {
        assert_eq!(size_of::<F>(), size_of::<*const u8>(), "a function pointer");
        self.address(name).map(|p| unsafe { std::mem::transmute_copy::<*const u8, F>(&p) })
    }

    /// The symbols defined, by the object's names, with their addresses.
    pub fn symbols(&self) -> impl Iterator<Item = (&str, *const u8)> {
        self.symbols.iter().map(|(n, &a)| (n.as_str(), a as *const u8))
    }

    /// The bytes of memory it takes, pages and all.
    pub fn size(&self) -> usize {
        self.size
    }
}

impl Drop for Loaded {
    fn drop(&mut self) {
        unsafe { sys::munmap(self.base as *mut c_void, self.size) };
    }
}

/// Loads `c` into this process, looking up the symbols it does not define
/// with `resolve` (given the IR's name). With inline assembly, by
/// assembling its assembly; otherwise straight from its machine code.
pub fn load(t: &Target, c: &Compiled, resolve: impl FnMut(&str) -> Option<*const u8>) -> Result<Loaded, String> {
    let obj = if c.has_inline_asm() { c.assembled(t)?.object } else { c.object(t)? };
    load_object(t, &obj, c.abi.syntax, resolve)
}

/// A resolver of the symbols this process has: those of the program and
/// the libraries it has loaded (`dlsym` of `RTLD_DEFAULT`).
pub fn process_symbol(name: &str) -> Option<*const u8> {
    let c = CString::new(name).ok()?;
    let p = unsafe { sys::dlsym(sys::RTLD_DEFAULT, c.as_ptr()) };
    (!p.is_null()).then_some(p as *const u8)
}

/// Where each loaded section goes: which of the three regions, and its
/// offset in the whole.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Region {
    Code,
    ReadOnly,
    Data,
}

fn region(kind: SectionKind) -> Option<Region> {
    match kind {
        SectionKind::Text => Some(Region::Code),
        SectionKind::ReadOnly | SectionKind::RelRo => Some(Region::ReadOnly),
        SectionKind::Data | SectionKind::Bss => Some(Region::Data),
        SectionKind::EhFrame | SectionKind::ExceptTable | SectionKind::ThreadVars | SectionKind::ThreadData => None,
    }
}

/// A stub that jumps to the 64-bit address in its last 8 bytes, for calls
/// to what lies out of a call's reach, by the machine's ELF number.
fn stub(elf_machine: u16) -> Option<[u8; 8]> {
    match elf_machine {
        // ldr x16, #8; br x16 (x16 is the register the procedure call
        // standard leaves to veneers).
        183 => Some([0x50, 0x00, 0x00, 0x58, 0x00, 0x02, 0x1f, 0xd6]),
        // jmp *2(%rip); ud2
        62 => Some([0xff, 0x25, 0x02, 0x00, 0x00, 0x00, 0x0f, 0x0b]),
        _ => None,
    }
}

/// The machine this process runs on, by ELF number.
const HOST_MACHINE: u16 = if cfg!(target_arch = "aarch64") {
    183
} else if cfg!(target_arch = "x86_64") {
    62
} else {
    0
};

/// Loads `obj` (in the conventions of `syntax`'s format) into this
/// process, looking up the symbols it does not define with `resolve`.
pub fn load_object(
    t: &Target,
    obj: &Object,
    syntax: Syntax,
    mut resolve: impl FnMut(&str) -> Option<*const u8>,
) -> Result<Loaded, String> {
    let m = &t.machine;
    if cfg!(all(target_arch = "aarch64", not(target_os = "macos"))) {
        // (Its instruction cache would need cleaning by hand.)
        return Err("loading code is not done on AArch64 other than macOS yet".into());
    }
    if m.elf_machine != HOST_MACHINE {
        return Err(format!("code for machine {} cannot run on this one ({HOST_MACHINE})", m.elf_machine));
    }
    for s in &obj.sections {
        if matches!(s.kind, SectionKind::ThreadVars | SectionKind::ThreadData) && s.size() > 0 {
            return Err("thread-local variables cannot be loaded".into());
        }
    }
    let page = sys::page_size();
    let loaded = |s: usize| region(obj.sections[s].kind).is_some();
    // What the loaded sections refer to that the object does not define,
    // and which of those calls reach (through stubs) or the GOT holds.
    let mut externs: HashMap<usize, usize> = HashMap::new();
    let mut stubs: HashMap<usize, usize> = HashMap::new();
    let mut got: HashMap<usize, usize> = HashMap::new();
    for (k, s) in obj.sections.iter().enumerate() {
        if !loaded(k) {
            continue;
        }
        for r in &s.relocs {
            let RelocTarget::Symbol(sym) = r.target else { continue };
            let usage = m.relocs[r.kind.0 as usize].usage;
            if obj.symbols[sym].section.is_none() {
                let n = externs.len();
                externs.entry(sym).or_insert(n);
                if usage == RelocUse::Call {
                    let n = stubs.len();
                    stubs.entry(sym).or_insert(n);
                }
            }
            if matches!(usage, RelocUse::Got | RelocUse::GotPcRel32) {
                let n = got.len();
                got.entry(sym).or_insert(n);
            }
        }
    }
    let stub_code =
        if stubs.is_empty() { [0; 8] } else { stub(m.elf_machine).ok_or("no stub for calls on this machine")? };
    // Where everything goes: each region's sections in order, the stubs
    // after the code and the GOT after the read-only data.
    let mut at = vec![0usize; obj.sections.len()];
    let mut ends = [0usize; 3];
    let mut stubs_at = 0;
    let mut got_at = 0;
    for (r, region_kind) in [Region::Code, Region::ReadOnly, Region::Data].into_iter().enumerate() {
        let mut end = if r == 0 { 0 } else { ends[r - 1].next_multiple_of(page) };
        for (k, s) in obj.sections.iter().enumerate() {
            if region(s.kind) == Some(region_kind) {
                end = end.next_multiple_of(s.align.max(1) as usize);
                at[k] = end;
                end += s.size() as usize;
            }
        }
        match region_kind {
            Region::Code => {
                stubs_at = end.next_multiple_of(16);
                end = stubs_at + 16 * stubs.len();
            }
            Region::ReadOnly => {
                got_at = end.next_multiple_of(8);
                end = got_at + 8 * got.len();
            }
            Region::Data => {}
        }
        ends[r] = end;
    }
    let code_size = ends[0].next_multiple_of(page);
    let ro_size = ends[1].next_multiple_of(page) - code_size;
    let size = ends[2].next_multiple_of(page).max(page);
    let base = sys::map(size, code_size)?;
    let mut image = Loaded { base, size, symbols: HashMap::new(), syntax };
    // Writes to the code are allowed until this goes.
    let writing = sys::Writing::start();

    // Each symbol's address.
    let mut addrs = vec![0usize; obj.symbols.len()];
    let mut missing = Vec::new();
    for (k, s) in obj.symbols.iter().enumerate() {
        match s.section {
            Some(sec) if loaded(sec) => addrs[k] = base as usize + at[sec] + s.value as usize,
            // In a table not loaded: nothing loaded should refer to it.
            Some(_) => {}
            None if externs.contains_key(&k) => {
                let ir_name = match syntax {
                    Syntax::MachO => s.name.strip_prefix('_').unwrap_or(&s.name),
                    Syntax::Elf => &s.name,
                };
                match resolve(ir_name) {
                    Some(p) => addrs[k] = p as usize,
                    None if obj.weak.contains(&s.name) => {}
                    None => missing.push(ir_name.to_string()),
                }
            }
            None => {}
        }
    }
    if !missing.is_empty() {
        missing.sort();
        missing.dedup();
        return Err(format!("undefined symbols: {}", missing.join(", ")));
    }
    for (k, s) in obj.symbols.iter().enumerate() {
        if let Some(sec) = s.section
            && loaded(sec)
            && (s.global || !image.symbols.contains_key(&s.name))
        {
            image.symbols.insert(s.name.clone(), addrs[k]);
        }
    }

    // The contents, the stubs and the GOT.
    let mem = |off: usize, len: usize| unsafe { std::slice::from_raw_parts_mut(base.add(off), len) };
    for (k, s) in obj.sections.iter().enumerate() {
        if loaded(k) && s.kind != SectionKind::Bss {
            mem(at[k], s.data.len()).copy_from_slice(&s.data);
        }
    }
    for (&sym, &n) in &stubs {
        let place = mem(stubs_at + 16 * n, 16);
        place[..8].copy_from_slice(&stub_code);
        place[8..].copy_from_slice(&(addrs[sym] as u64).to_le_bytes());
    }
    for (&sym, &n) in &got {
        mem(got_at + 8 * n, 8).copy_from_slice(&(addrs[sym] as u64).to_le_bytes());
    }

    // The relocations.
    let page_mask = low_page(t);
    for (k, s) in obj.sections.iter().enumerate() {
        if !loaded(k) {
            continue;
        }
        let sec_base = base as usize + at[k];
        for r in &s.relocs {
            let def = &m.relocs[r.kind.0 as usize];
            let (sym, target) = match r.target {
                RelocTarget::Symbol(sym) => (Some(sym), addrs[sym]),
                RelocTarget::Section(sec) if loaded(sec) => (None, base as usize + at[sec]),
                RelocTarget::Section(_) => return Err(format!("{}: a reference to a table not loaded", def.name)),
            };
            let name = || sym.map_or_else(|| "a section".to_string(), |s| obj.symbols[s].name.clone());
            if let Some(sym) = sym
                && obj.symbols[sym].section.is_some_and(|sec| !loaded(sec))
            {
                return Err(format!("{}: a reference to {}, in a table not loaded", def.name, name()));
            }
            // The address the field is to reach: the symbol's, its stub's
            // (a call to what the object does not define) or its GOT
            // entry's.
            let x = match def.usage {
                RelocUse::Call if sym.is_some_and(|s| stubs.contains_key(&s)) => {
                    base as usize + stubs_at + 16 * stubs[&sym.unwrap()]
                }
                RelocUse::Got | RelocUse::GotPcRel32 => {
                    let sym = sym.ok_or_else(|| format!("{}: a GOT entry of a section", def.name))?;
                    base as usize + got_at + 8 * got[&sym]
                }
                _ => target,
            }
            .wrapping_add(r.addend as usize);
            let p = sec_base + r.offset as usize;
            let field = mem(at[k] + r.offset as usize, s.data.len().saturating_sub(r.offset as usize));
            match def.usage {
                RelocUse::Data(n) => {
                    let bytes = (x as u64).to_le_bytes();
                    if n < 8 && (x as u64) >> (8 * n) != 0 {
                        return Err(format!("{}: the address of {} does not fit {n} bytes", def.name, name()));
                    }
                    field[..n as usize].copy_from_slice(&bytes[..n as usize]);
                }
                RelocUse::EhPcRel | RelocUse::GotPcRel32 if r.instr.is_none() => {
                    let d = i32::try_from(x as i64 - p as i64)
                        .map_err(|_| format!("{}: {} is out of reach", def.name, name()))?;
                    field[..4].copy_from_slice(&d.to_le_bytes());
                }
                RelocUse::Plain | RelocUse::Call | RelocUse::Got => {
                    let i = r.instr.ok_or_else(|| format!("{}: a relocation not in an instruction", def.name))?;
                    fill(t, i, def, field, p, x, page_mask).map_err(|e| format!("{e} ({})", name()))?;
                }
                u => return Err(format!("{}: relocations of use {u:?} cannot be loaded ({})", def.name, name())),
            }
        }
    }
    drop(writing);
    sys::invalidate(base, code_size);
    sys::protect(base, code_size, sys::PROT_READ | sys::PROT_EXEC)?;
    if ro_size > 0 {
        sys::protect(unsafe { base.add(code_size) }, ro_size, sys::PROT_READ)?;
    }
    Ok(image)
}

/// The low bits of an address that an instruction holds where another
/// holds the rest (AArch64's `add x0, x0, :lo12:sym` after `adrp`): one
/// less than the largest page a relocation of the description measures in
/// (one whose target is `(pc & -4096) + (v << 12)`).
fn low_page(t: &Target) -> u64 {
    t.machine
        .relocs
        .iter()
        .filter_map(|r| r.reaches(u64::MAX, 0))
        .map(|a| 0u64.wrapping_sub(a))
        .filter(|p| p.is_power_of_two())
        .max()
        .unwrap_or(1)
        - 1
}

/// Fills in the symbol operand of instruction `i` (of the description),
/// whose field `field` starts (at address `p`), to reach address `x`.
fn fill(
    t: &Target,
    i: usize,
    def: &crate::machine::RelocDef,
    field: &mut [u8],
    p: usize,
    x: usize,
    page_mask: u64,
) -> Result<(), String> {
    let ins = &t.instrs[i];
    let op = ins.operands.iter().find(|o| o.kind == Kind::Sym).ok_or("a reference without a symbol operand")?;
    // The field is a 32-bit one of its own (x86's): it holds the distance
    // from its own place, as a linker writes it, S + A - P.
    let whole = ins.enc.alts.iter().all(|a| {
        a.fields
            .iter()
            .filter(|x| x.names.contains(&op.name))
            .all(|x| x.width == 32 && x.lo == 0 && a.pieces[x.piece].bytes == 4)
    });
    if whole && def.pc_relative() {
        let d = i32::try_from(x as i64 - p as i64).map_err(|_| format!("{}: out of reach", ins.name))?;
        field[..4].copy_from_slice(&d.to_le_bytes());
        return Ok(());
    }
    // Otherwise the instruction is decoded, given the operand that reaches
    // `x` and encoded again: it starts where its field's piece does not.
    for (k, a) in ins.enc.alts.iter().enumerate() {
        let Some(f) = a.fields.iter().find(|f| f.names.contains(&op.name)) else { continue };
        let back = a.pieces[f.piece].offset as usize;
        if back != 0 {
            // (No machine yet has an instruction like this whose field is
            // not in its first piece: the field's place is its start.)
            continue;
        }
        let Some((mut env, len)) = ins.decode_alt(field, k) else { continue };
        let v = if def.pc_relative() {
            // The target is the operand scaled, from a place the pc gives:
            // `pc + v`, `(pc & -4096) + (v << 12)`.
            let from = def.reaches(p as u64, 0).ok_or("an unreadable target")? as i64;
            let step = def.reaches(p as u64, 1).ok_or("an unreadable target")? as i64 - from;
            (x as i64 - from).div_euclid(step)
        } else {
            (x as u64 & page_mask) as i64
        };
        env.insert(op.name.clone(), v);
        let bytes = ins.encode(&env).ok_or_else(|| format!("{}: out of reach", ins.name))?;
        if bytes.len() != len {
            return Err(format!("{}: changed size", ins.name));
        }
        field[..len].copy_from_slice(&bytes);
        return Ok(());
    }
    Err(format!("cannot decode the {} to fill it in", ins.name))
}

/// The few calls into the C library loading needs.
mod sys {
    use super::*;

    pub const PROT_READ: c_int = 1;
    pub const PROT_WRITE: c_int = 2;
    pub const PROT_EXEC: c_int = 4;
    const MAP_PRIVATE: c_int = 2;
    const MAP_FIXED: c_int = 0x10;
    #[cfg(target_os = "macos")]
    const MAP_ANON: c_int = 0x1000;
    #[cfg(not(target_os = "macos"))]
    const MAP_ANON: c_int = 0x20;
    #[cfg(target_os = "macos")]
    const MAP_JIT: c_int = 0x800;
    #[cfg(target_os = "macos")]
    pub const RTLD_DEFAULT: *mut c_void = -2isize as *mut c_void;
    #[cfg(not(target_os = "macos"))]
    pub const RTLD_DEFAULT: *mut c_void = std::ptr::null_mut();

    unsafe extern "C" {
        fn mmap(addr: *mut c_void, len: usize, prot: c_int, flags: c_int, fd: c_int, offset: i64) -> *mut c_void;
        pub fn munmap(addr: *mut c_void, len: usize) -> c_int;
        fn mprotect(addr: *mut c_void, len: usize, prot: c_int) -> c_int;
        fn getpagesize() -> c_int;
        pub fn dlsym(handle: *mut c_void, name: *const c_char) -> *mut c_void;
    }

    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    unsafe extern "C" {
        fn pthread_jit_write_protect_np(enabled: c_int);
        fn sys_icache_invalidate(start: *mut c_void, len: usize);
    }

    pub fn page_size() -> usize {
        unsafe { getpagesize() as usize }
    }

    fn failed(p: *mut c_void) -> bool {
        p as isize == -1
    }

    /// `size` bytes of fresh memory, the first `code` of them to hold code,
    /// all writable (the code by `Writing`).
    pub fn map(size: usize, code: usize) -> Result<*mut u8, String> {
        let rw = PROT_READ | PROT_WRITE;
        #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
        unsafe {
            // All of it as JIT memory (which cannot be put at a place of
            // one's choosing), then what is not code replaced by ordinary
            // memory: JIT memory is writable only by a thread that is not
            // running code in it.
            let p = mmap(std::ptr::null_mut(), size, rw | PROT_EXEC, MAP_PRIVATE | MAP_ANON | MAP_JIT, -1, 0);
            if failed(p) {
                return Err(format!("mmap of {size} bytes for code: {}", std::io::Error::last_os_error()));
            }
            if size > code {
                let d = p.cast::<u8>().add(code).cast();
                let q = mmap(d, size - code, rw, MAP_PRIVATE | MAP_ANON | MAP_FIXED, -1, 0);
                if failed(q) {
                    let e = std::io::Error::last_os_error();
                    munmap(p, size);
                    return Err(format!("mmap of {} bytes for data: {e}", size - code));
                }
            }
            Ok(p.cast())
        }
        #[cfg(not(all(target_os = "macos", target_arch = "aarch64")))]
        unsafe {
            let _ = (code, MAP_FIXED);
            let p = mmap(std::ptr::null_mut(), size, rw, MAP_PRIVATE | MAP_ANON, -1, 0);
            if failed(p) {
                return Err(format!("mmap of {size} bytes: {}", std::io::Error::last_os_error()));
            }
            Ok(p.cast())
        }
    }

    /// While it lives, this thread may write the code.
    pub struct Writing;

    impl Writing {
        pub fn start() -> Writing {
            #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
            unsafe {
                pthread_jit_write_protect_np(0)
            };
            Writing
        }
    }

    impl Drop for Writing {
        fn drop(&mut self) {
            #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
            unsafe {
                pthread_jit_write_protect_np(1)
            };
        }
    }

    /// Makes the instruction cache see the code written.
    pub fn invalidate(start: *mut u8, len: usize) {
        #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
        unsafe {
            sys_icache_invalidate(start.cast(), len)
        };
        // (x86 keeps its caches coherent with what is written.)
        let _ = (start, len);
    }

    pub fn protect(start: *mut u8, len: usize, prot: c_int) -> Result<(), String> {
        // MAP_JIT code is executable already, and stays writable only to a
        // thread that asks.
        if cfg!(all(target_os = "macos", target_arch = "aarch64")) && prot & PROT_EXEC != 0 {
            return Ok(());
        }
        if unsafe { mprotect(start.cast(), len, prot) } != 0 {
            return Err(format!("mprotect: {}", std::io::Error::last_os_error()));
        }
        Ok(())
    }
}
