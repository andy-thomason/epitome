//! ELF relocatable files: writing an [`Object`], and reading
//! any ELF file back (for `epitome-objdump` and the tests).
//!
//! From the System V ABI's ELF chapters. The machine number, flags and
//! relocation numbers are the description's (`machine.rs`). Mapping
//! symbols (from Arm's *ELF for the Arm 64-bit Architecture*, and used
//! alike for other machines) mark where code starts in a section (`$x`)
//! and where data does (`$d`), so that disassemblers can tell them apart.
//!
//! A written file has, after the null section: the object's sections, in
//! its order (`.text`, `.data`, `.bss`, `.rodata`, `.data.rel.ro`); a
//! `.rela` section for each with relocations; an empty `.note.GNU-stack`
//! (the stack need not be executable); then `.symtab`, `.strtab` and
//! `.shstrtab`. Every symbol is local or global: local ones first, as ELF
//! requires (a symbol for each section relocations refer to, mapping
//! symbols, then the object's own), then global ones, defined and
//! undefined.

use crate::machine::{Machine, RelocUse};
use crate::object::{Object, RelocTarget, SectionKind, SymbolKind};

pub const ET_REL: u16 = 1;

pub const SHT_NULL: u32 = 0;
pub const SHT_PROGBITS: u32 = 1;
pub const SHT_SYMTAB: u32 = 2;
pub const SHT_STRTAB: u32 = 3;
pub const SHT_RELA: u32 = 4;
pub const SHT_NOBITS: u32 = 8;
pub const SHT_INIT_ARRAY: u32 = 14;
pub const SHT_FINI_ARRAY: u32 = 15;
pub const SHT_PREINIT_ARRAY: u32 = 16;

pub const SHF_WRITE: u64 = 1;
pub const SHF_ALLOC: u64 = 2;
pub const SHF_EXECINSTR: u64 = 4;
pub const SHF_INFO_LINK: u64 = 0x40;

pub const STB_LOCAL: u8 = 0;
pub const STB_GLOBAL: u8 = 1;
pub const STB_WEAK: u8 = 2;
pub const STT_NOTYPE: u8 = 0;
pub const STT_OBJECT: u8 = 1;
pub const STT_FUNC: u8 = 2;
pub const STT_SECTION: u8 = 3;
pub const STT_FILE: u8 = 4;
pub const STT_TLS: u8 = 6;

pub fn section_name(kind: SectionKind) -> &'static str {
    match kind {
        SectionKind::Text => ".text",
        SectionKind::Data => ".data",
        SectionKind::Bss => ".bss",
        SectionKind::ReadOnly => ".rodata",
        SectionKind::RelRo => ".data.rel.ro",
        SectionKind::EhFrame => ".eh_frame",
        SectionKind::ExceptTable => ".gcc_except_table",
        // (ThreadVars, Mach-O's descriptors, are not made for ELF.)
        SectionKind::ThreadVars | SectionKind::ThreadData => ".tdata",
    }
}

fn section_flags(kind: SectionKind) -> u64 {
    match kind {
        SectionKind::Text => SHF_ALLOC | SHF_EXECINSTR,
        SectionKind::Data | SectionKind::Bss | SectionKind::RelRo => SHF_ALLOC | SHF_WRITE,
        SectionKind::ReadOnly | SectionKind::EhFrame | SectionKind::ExceptTable => SHF_ALLOC,
        // SHF_TLS
        SectionKind::ThreadVars | SectionKind::ThreadData => SHF_ALLOC | SHF_WRITE | 0x400,
    }
}

/// A string table: names, each ended by a zero byte, after a first zero.
#[derive(Default)]
struct Strings {
    bytes: Vec<u8>,
}

impl Strings {
    fn new() -> Strings {
        Strings { bytes: vec![0] }
    }

    fn add(&mut self, s: &str) -> u32 {
        if s.is_empty() {
            return 0;
        }
        let at = self.bytes.len() as u32;
        self.bytes.extend_from_slice(s.as_bytes());
        self.bytes.push(0);
        at
    }
}

struct Header {
    name: u32,
    ty: u32,
    flags: u64,
    offset: u64,
    size: u64,
    link: u32,
    info: u32,
    align: u64,
    entsize: u64,
}

/// Writes `obj` as an ELF64 little-endian relocatable file for machine `m`
/// (its `elf-machine` and `elf-flags`, and its relocations' numbers).
pub fn write(obj: &Object, m: &Machine) -> Result<Vec<u8>, String> {
    let mut shstr = Strings::new();
    let mut strtab = Strings::new();
    let n = obj.sections.len();
    // Section indices: 0 null, then the object's.
    let index_of = |s: usize| (s + 1) as u16;

    // Symbols: locals first.
    let mut syms: Vec<[u8; 24]> = vec![[0; 24]];
    let sym = |name: u32, bind: u8, ty: u8, shndx: u16, value: u64, size: u64| -> [u8; 24] {
        let mut e = [0u8; 24];
        e[0..4].copy_from_slice(&name.to_le_bytes());
        e[4] = (bind << 4) | ty;
        e[6..8].copy_from_slice(&shndx.to_le_bytes());
        e[8..16].copy_from_slice(&value.to_le_bytes());
        e[16..24].copy_from_slice(&size.to_le_bytes());
        e
    };
    // A symbol for each section that relocations refer to.
    let mut section_sym = vec![0u32; n];
    for (k, s) in obj.sections.iter().enumerate() {
        let referred = obj.sections.iter().flat_map(|x| &x.relocs).any(|r| r.target == RelocTarget::Section(k));
        if referred {
            section_sym[k] = syms.len() as u32;
            syms.push(sym(0, STB_LOCAL, STT_SECTION, index_of(k), 0, 0));
        }
        if s.size() > 0 {
            let name = strtab.add(if s.kind == SectionKind::Text { "$x" } else { "$d" });
            // (In a thread-local section, a thread-local symbol too.)
            let ty =
                if matches!(s.kind, SectionKind::ThreadData | SectionKind::ThreadVars) { STT_TLS } else { STT_NOTYPE };
            syms.push(sym(name, STB_LOCAL, ty, index_of(k), 0, 0));
        }
    }
    let mut sym_index = vec![0u32; obj.symbols.len()];
    // Thread-local variables: those in a thread-local section, and those
    // others define that thread-local relocations refer to.
    let mut tls = vec![false; obj.symbols.len()];
    for (k, s) in obj.symbols.iter().enumerate() {
        tls[k] = s
            .section
            .is_some_and(|x| matches!(obj.sections[x].kind, SectionKind::ThreadData | SectionKind::ThreadVars));
    }
    for r in obj.sections.iter().flat_map(|x| &x.relocs) {
        if let RelocTarget::Symbol(k) = r.target
            && m.relocs[r.kind.0 as usize].usage == RelocUse::Tls
        {
            tls[k] = true;
        }
    }
    let elf_sym = |k: usize, s: &crate::object::Symbol, strtab: &mut Strings| {
        let ty = match s.kind {
            _ if tls[k] => STT_TLS,
            SymbolKind::Func => STT_FUNC,
            SymbolKind::Object => STT_OBJECT,
            SymbolKind::NoType => STT_NOTYPE,
        };
        let bind = match (s.global, obj.weak.contains(&s.name)) {
            (true, true) => STB_WEAK,
            (true, false) => STB_GLOBAL,
            (false, _) => STB_LOCAL,
        };
        let shndx = s.section.map_or(0, index_of);
        sym(strtab.add(&s.name), bind, ty, shndx, s.value, s.size)
    };
    for global in [false, true] {
        for (k, s) in obj.symbols.iter().enumerate() {
            if s.global == global {
                sym_index[k] = syms.len() as u32;
                syms.push(elf_sym(k, s, &mut strtab));
            }
        }
    }
    let first_global = 1 + syms[1..].iter().take_while(|e| e[4] >> 4 == STB_LOCAL).count();

    // Section contents and headers.
    let mut out = vec![0u8; 64];
    let mut headers: Vec<Header> =
        vec![Header { name: 0, ty: SHT_NULL, flags: 0, offset: 0, size: 0, link: 0, info: 0, align: 0, entsize: 0 }];
    let put = |out: &mut Vec<u8>, bytes: &[u8], align: u64| -> u64 {
        let at = (out.len() as u64).next_multiple_of(align.max(1));
        out.resize(at as usize, 0);
        out.extend_from_slice(bytes);
        at
    };
    for s in &obj.sections {
        let bss = s.kind == SectionKind::Bss;
        let offset = put(&mut out, if bss { &[] } else { &s.data }, s.align);
        headers.push(Header {
            name: shstr.add(s.name.as_deref().unwrap_or(section_name(s.kind))),
            ty: if bss {
                SHT_NOBITS
            } else {
                match s.name.as_deref() {
                    Some(n) if n.starts_with(".init_array") => SHT_INIT_ARRAY,
                    Some(n) if n.starts_with(".fini_array") => SHT_FINI_ARRAY,
                    Some(n) if n.starts_with(".preinit_array") => SHT_PREINIT_ARRAY,
                    _ => SHT_PROGBITS,
                }
            },
            flags: section_flags(s.kind),
            offset,
            size: s.size(),
            link: 0,
            info: 0,
            align: s.align,
            entsize: 0,
        });
    }
    let symtab_index = (1 + n + obj.sections.iter().filter(|s| !s.relocs.is_empty()).count() + 1) as u32;
    for (k, s) in obj.sections.iter().enumerate() {
        if s.relocs.is_empty() {
            continue;
        }
        let mut bytes = Vec::with_capacity(24 * s.relocs.len());
        for r in &s.relocs {
            let target = match r.target {
                RelocTarget::Symbol(i) => sym_index[i],
                RelocTarget::Section(i) => section_sym[i],
            };
            bytes.extend_from_slice(&r.offset.to_le_bytes());
            let def = &m.relocs[r.kind.0 as usize];
            let ty = def.elf.ok_or_else(|| format!("relocation {} has no ELF number", def.name))?;
            bytes.extend_from_slice(&(((target as u64) << 32) | ty as u64).to_le_bytes());
            bytes.extend_from_slice(&r.addend.to_le_bytes());
        }
        let offset = put(&mut out, &bytes, 8);
        headers.push(Header {
            name: shstr.add(&format!(".rela{}", s.name.as_deref().unwrap_or(section_name(s.kind)))),
            ty: SHT_RELA,
            flags: SHF_INFO_LINK,
            offset,
            size: bytes.len() as u64,
            link: symtab_index,
            info: index_of(k) as u32,
            align: 8,
            entsize: 24,
        });
    }
    headers.push(Header {
        name: shstr.add(".note.GNU-stack"),
        ty: SHT_PROGBITS,
        flags: 0,
        offset: out.len() as u64,
        size: 0,
        link: 0,
        info: 0,
        align: 1,
        entsize: 0,
    });
    let symtab: Vec<u8> = syms.concat();
    let offset = put(&mut out, &symtab, 8);
    headers.push(Header {
        name: shstr.add(".symtab"),
        ty: SHT_SYMTAB,
        flags: 0,
        offset,
        size: symtab.len() as u64,
        link: symtab_index + 1,
        info: first_global as u32,
        align: 8,
        entsize: 24,
    });
    let offset = put(&mut out, &strtab.bytes, 1);
    headers.push(Header {
        name: shstr.add(".strtab"),
        ty: SHT_STRTAB,
        flags: 0,
        offset,
        size: strtab.bytes.len() as u64,
        link: 0,
        info: 0,
        align: 1,
        entsize: 0,
    });
    let shstrtab_name = shstr.add(".shstrtab");
    let offset = put(&mut out, &shstr.bytes, 1);
    headers.push(Header {
        name: shstrtab_name,
        ty: SHT_STRTAB,
        flags: 0,
        offset,
        size: shstr.bytes.len() as u64,
        link: 0,
        info: 0,
        align: 1,
        entsize: 0,
    });

    let shoff = put(&mut out, &[], 8);
    for h in &headers {
        out.extend_from_slice(&h.name.to_le_bytes());
        out.extend_from_slice(&h.ty.to_le_bytes());
        out.extend_from_slice(&h.flags.to_le_bytes());
        out.extend_from_slice(&0u64.to_le_bytes());
        out.extend_from_slice(&h.offset.to_le_bytes());
        out.extend_from_slice(&h.size.to_le_bytes());
        out.extend_from_slice(&h.link.to_le_bytes());
        out.extend_from_slice(&h.info.to_le_bytes());
        out.extend_from_slice(&h.align.to_le_bytes());
        out.extend_from_slice(&h.entsize.to_le_bytes());
    }

    // The file header.
    let mut h = Vec::with_capacity(64);
    h.extend_from_slice(&[0x7f, b'E', b'L', b'F', 2, 1, 1, 0]);
    h.extend_from_slice(&[0; 8]);
    h.extend_from_slice(&ET_REL.to_le_bytes());
    h.extend_from_slice(&m.elf_machine.to_le_bytes());
    h.extend_from_slice(&1u32.to_le_bytes());
    h.extend_from_slice(&0u64.to_le_bytes()); // entry
    h.extend_from_slice(&0u64.to_le_bytes()); // program headers
    h.extend_from_slice(&shoff.to_le_bytes());
    h.extend_from_slice(&m.elf_flags.to_le_bytes());
    h.extend_from_slice(&64u16.to_le_bytes());
    h.extend_from_slice(&0u16.to_le_bytes());
    h.extend_from_slice(&0u16.to_le_bytes());
    h.extend_from_slice(&64u16.to_le_bytes());
    h.extend_from_slice(&(headers.len() as u16).to_le_bytes());
    h.extend_from_slice(&((headers.len() - 1) as u16).to_le_bytes());
    out[..64].copy_from_slice(&h);
    Ok(out)
}

// ---------------------------------------------------------------------------
// Reading

#[derive(Clone, Debug)]
pub struct ElfFile {
    pub file_type: u16,
    pub machine: u16,
    pub entry: u64,
    pub sections: Vec<ElfSection>,
    pub symbols: Vec<ElfSymbol>,
}

#[derive(Clone, Debug)]
pub struct ElfSection {
    pub name: String,
    pub sh_type: u32,
    pub flags: u64,
    pub addr: u64,
    pub offset: u64,
    pub size: u64,
    pub link: u32,
    pub info: u32,
    pub align: u64,
    pub entsize: u64,
    /// The contents (none for `SHT_NOBITS`).
    pub data: Vec<u8>,
    /// The relocations that apply to this section, from the `SHT_RELA`
    /// section whose `info` names it.
    pub relocs: Vec<ElfRela>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ElfSymbol {
    pub name: String,
    pub value: u64,
    pub size: u64,
    pub bind: u8,
    pub sym_type: u8,
    /// The section index; 0 if undefined.
    pub shndx: u16,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ElfRela {
    pub offset: u64,
    /// An index into the symbol table.
    pub sym: u32,
    pub rel_type: u32,
    pub addend: i64,
}

struct Bytes<'a>(&'a [u8]);

impl Bytes<'_> {
    fn get(&self, at: u64, n: u64) -> Result<&[u8], String> {
        let end = at.checked_add(n).ok_or("a size past the end")?;
        self.0.get(at as usize..end as usize).ok_or_else(|| format!("{n} bytes at {at:#x} are past the end"))
    }
    fn u16(&self, at: u64) -> Result<u16, String> {
        Ok(u16::from_le_bytes(self.get(at, 2)?.try_into().unwrap()))
    }
    fn u32(&self, at: u64) -> Result<u32, String> {
        Ok(u32::from_le_bytes(self.get(at, 4)?.try_into().unwrap()))
    }
    fn u64(&self, at: u64) -> Result<u64, String> {
        Ok(u64::from_le_bytes(self.get(at, 8)?.try_into().unwrap()))
    }
}

fn c_string(table: &[u8], at: u32) -> String {
    let rest = table.get(at as usize..).unwrap_or(&[]);
    let end = rest.iter().position(|&b| b == 0).unwrap_or(rest.len());
    String::from_utf8_lossy(&rest[..end]).into_owned()
}

/// Is this an ELF file (by its first bytes)?
pub fn is_elf(bytes: &[u8]) -> bool {
    bytes.starts_with(&[0x7f, b'E', b'L', b'F'])
}

/// Reads a 64-bit little-endian ELF file: its sections, symbols, and
/// relocations.
pub fn read(bytes: &[u8]) -> Result<ElfFile, String> {
    let b = Bytes(bytes);
    if !is_elf(bytes) {
        return Err("not an ELF file".into());
    }
    let ident = b.get(0, 16)?;
    if ident[4] != 2 || ident[5] != 1 {
        return Err("not a 64-bit little-endian ELF file".into());
    }
    let file_type = b.u16(16)?;
    let machine = b.u16(18)?;
    let entry = b.u64(24)?;
    let shoff = b.u64(40)?;
    let shentsize = b.u16(58)? as u64;
    let shnum = b.u16(60)? as u64;
    let shstrndx = b.u16(62)? as u64;
    if shnum > 0 && shentsize < 64 {
        return Err("section headers too small".into());
    }
    let mut sections = Vec::new();
    for k in 0..shnum {
        let at = shoff + k * shentsize;
        let sh_type = b.u32(at + 4)?;
        let offset = b.u64(at + 24)?;
        let size = b.u64(at + 32)?;
        let data =
            if sh_type == SHT_NOBITS || sh_type == SHT_NULL { Vec::new() } else { b.get(offset, size)?.to_vec() };
        sections.push(ElfSection {
            name: String::new(),
            sh_type,
            flags: b.u64(at + 8)?,
            addr: b.u64(at + 16)?,
            offset,
            size,
            link: b.u32(at + 40)?,
            info: b.u32(at + 44)?,
            align: b.u64(at + 48)?,
            entsize: b.u64(at + 56)?,
            data,
            relocs: Vec::new(),
        });
        // The name comes once the section-name table is read.
        let _ = b.u32(at)?;
    }
    if let Some(names) = sections.get(shstrndx as usize).map(|s| s.data.clone()) {
        for k in 0..shnum {
            let name_at = b.u32(shoff + k * shentsize)?;
            sections[k as usize].name = c_string(&names, name_at);
        }
    }
    let mut symbols = Vec::new();
    if let Some(st) = sections.iter().find(|s| s.sh_type == SHT_SYMTAB) {
        let names = sections.get(st.link as usize).map(|s| s.data.clone()).unwrap_or_default();
        let d = Bytes(&st.data);
        for k in 0..st.data.len() as u64 / 24 {
            let at = k * 24;
            let info = d.get(at + 4, 1)?[0];
            symbols.push(ElfSymbol {
                name: c_string(&names, d.u32(at)?),
                bind: info >> 4,
                sym_type: info & 15,
                shndx: d.u16(at + 6)?,
                value: d.u64(at + 8)?,
                size: d.u64(at + 16)?,
            });
        }
    }
    for k in 0..sections.len() {
        if sections[k].sh_type != SHT_RELA {
            continue;
        }
        let target = sections[k].info as usize;
        let d = Bytes(&sections[k].data);
        let mut relocs = Vec::new();
        for r in 0..sections[k].data.len() as u64 / 24 {
            let info = d.u64(r * 24 + 8)?;
            relocs.push(ElfRela {
                offset: d.u64(r * 24)?,
                sym: (info >> 32) as u32,
                rel_type: info as u32,
                addend: d.u64(r * 24 + 16)? as i64,
            });
        }
        if let Some(t) = sections.get_mut(target) {
            t.relocs.extend(relocs);
        }
    }
    Ok(ElfFile { file_type, machine, entry, sections, symbols })
}

impl ElfFile {
    /// What relocation `r` refers to, as text: the symbol's name, or for a
    /// section symbol, the section's.
    pub fn reloc_target(&self, r: &ElfRela) -> String {
        match self.symbols.get(r.sym as usize) {
            Some(s) if s.sym_type == STT_SECTION => {
                self.sections.get(s.shndx as usize).map_or_else(|| "?".into(), |x| x.name.clone())
            }
            Some(s) => s.name.clone(),
            None => format!("<symbol {}>", r.sym),
        }
    }
}

impl ElfFile {
    /// The file as sorted text, for comparing two files that should say
    /// the same: each section's kind, flags, size and contents, its
    /// relocations (against names, section symbols by their section's),
    /// and the symbols other than section symbols. Where things are in the
    /// file, and in what order, is left out.
    pub fn canonical(&self, m: &Machine) -> String {
        use std::fmt::Write;
        let mut out = String::new();
        let mut secs: Vec<&ElfSection> = self
            .sections
            .iter()
            .filter(|s| !matches!(s.sh_type, SHT_NULL | SHT_SYMTAB | SHT_STRTAB | SHT_RELA))
            .collect();
        secs.sort_by(|a, b| a.name.cmp(&b.name));
        for s in secs {
            let _ = writeln!(out, "section {} type {} flags {:#x} size {:#x}", s.name, s.sh_type, s.flags, s.size);
            for (k, chunk) in s.data.chunks(16).enumerate() {
                let hex: Vec<String> = chunk.iter().map(|b| format!("{b:02x}")).collect();
                let _ = writeln!(out, "  {:06x} {}", k * 16, hex.join(""));
            }
            let mut relocs: Vec<String> = s
                .relocs
                .iter()
                .map(|r| {
                    format!(
                        "  reloc {:06x} {} {}{:+#x}",
                        r.offset,
                        m.elf_name(r.rel_type),
                        self.reloc_target(r),
                        r.addend
                    )
                })
                .collect();
            relocs.sort();
            for r in relocs {
                let _ = writeln!(out, "{r}");
            }
        }
        let mut syms: Vec<String> = self
            .symbols
            .iter()
            .skip(1)
            .filter(|s| s.sym_type != STT_SECTION)
            .map(|s| {
                let sec = match s.shndx {
                    0 => "*UND*".to_string(),
                    k => self.sections.get(k as usize).map_or_else(|| format!("#{k}"), |x| x.name.clone()),
                };
                format!(
                    "symbol {} bind {} type {} in {} at {:#x} size {:#x}",
                    s.name, s.bind, s.sym_type, sec, s.value, s.size
                )
            })
            .collect();
        syms.sort();
        for s in syms {
            let _ = writeln!(out, "{s}");
        }
        out
    }
}
