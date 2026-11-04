//! What `epitome-objdump` prints: an object file's sections, symbols,
//! relocations and contents, and its code disassembled with the machine
//! description.
//!
//! A file (ELF, or Mach-O: objects, linked images, and the arm64 slice of
//! universal files) is first read into an [`Image`], which says the same
//! for every format: sections with their addresses and bytes, symbols
//! with their addresses, and relocations against names.
//!
//! Disassembly decodes each word as the description's instruction whose
//! literal bits it has (see `mir::decode`). Where a relocation will fill a
//! word in, the word is decoded as the instruction that carries that kind
//! of relocation (its `"reloc"`), and the symbol is printed in its place, as
//! assembly would write it: `add x0, x0, #0` with an `R_AARCH64_ADD_ABS_LO12_NC`
//! against `names` is `add x0, x0, :lo12:names`. Branch targets are printed
//! with the symbol they are in, `0x98 <use+0x90>`. Mapping symbols (`$x`,
//! `$d`) say where code and data are in a section: data is printed as
//! `.word`s.

use std::fmt::Write;

use crate::abi::Syntax;
use crate::enc::{self, Env};
use crate::machine::Machine;
use crate::mir::{self, Operand, Reg};
use crate::object::RelocKind;
use crate::target::{Kind, Target};
use crate::{elf, macho};

#[derive(Clone, Debug)]
pub struct Image {
    /// The format, as objdump names it: `elf64-littleaarch64`.
    pub format: String,
    /// Mach-O: symbols are written `sym@PAGE`, not `:lo12:sym`.
    pub macho: bool,
    pub sections: Vec<ImageSection>,
    pub symbols: Vec<ImageSymbol>,
}

#[derive(Clone, Debug)]
pub struct ImageSection {
    pub name: String,
    pub addr: u64,
    pub size: u64,
    /// The contents (empty if the section has none in the file).
    pub data: Vec<u8>,
    pub code: bool,
    /// `TEXT`, `DATA`, `BSS`, or empty for sections not loaded.
    pub kind: &'static str,
    pub relocs: Vec<ImageReloc>,
}

#[derive(Clone, Debug)]
pub struct ImageReloc {
    /// From the start of the section.
    pub offset: u64,
    pub type_name: String,
    /// The kind, if the description names it.
    pub kind: Option<RelocKind>,
    pub target: String,
    pub addend: i64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SymbolClass {
    Func,
    Object,
    Section,
    /// `$x` or `$d` (or `$x.<n>`): where code or data starts.
    Mapping,
    Other,
}

#[derive(Clone, Debug)]
pub struct ImageSymbol {
    pub name: String,
    pub value: u64,
    pub size: u64,
    /// An index into the image's sections; `None` if undefined (or
    /// absolute).
    pub section: Option<usize>,
    pub global: bool,
    pub weak: bool,
    pub class: SymbolClass,
}

impl Image {
    /// Reads an image from a file's bytes.
    /// Reads an image from a file's bytes; machine `m` names its
    /// relocations.
    pub fn read(bytes: &[u8], m: &Machine) -> Result<Image, String> {
        if elf::is_elf(bytes) {
            return Ok(Image::from_elf(&elf::read(bytes)?, m));
        }
        if macho::is_macho(bytes) {
            return Ok(Image::from_macho(&macho::read(bytes)?, m));
        }
        Err("not a file format epitome-objdump reads (ELF, Mach-O)".into())
    }

    pub fn from_macho(f: &macho::MachFile, m: &Machine) -> Image {
        let sections: Vec<ImageSection> = f
            .sections
            .iter()
            .map(|s| ImageSection {
                name: format!("{},{}", s.segment, s.name),
                addr: s.addr,
                size: s.size,
                data: s.data.clone(),
                code: s.is_code(),
                kind: match (s.is_code(), s.is_zerofill()) {
                    (true, _) => "TEXT",
                    (false, true) => "BSS",
                    _ => "DATA",
                },
                relocs: Vec::new(),
            })
            .collect();
        let mut img = Image { format: "mach-o arm64".into(), macho: true, sections, symbols: Vec::new() };
        for (k, s) in f.sections.iter().enumerate() {
            img.sections[k].relocs = s
                .relocs
                .iter()
                .map(|r| ImageReloc {
                    offset: r.offset,
                    type_name: m.macho_name(r.rel_type),
                    kind: m.by_macho(r.rel_type, r.length).map(|k| RelocKind(k as u16)),
                    target: if r.external {
                        f.symbols
                            .get(r.symbolnum as usize)
                            .map_or_else(|| format!("<symbol {}>", r.symbolnum), |x| x.name.clone())
                    } else {
                        img.sections
                            .get(r.symbolnum as usize - 1)
                            .map_or_else(|| format!("<section {}>", r.symbolnum), |x| x.name.clone())
                    },
                    addend: r.addend,
                })
                .collect();
        }
        img.symbols = f
            .symbols
            .iter()
            .filter(|s| !s.is_stab())
            .map(|s| {
                let section = if s.is_defined() && s.sect > 0 { Some(s.sect as usize - 1) } else { None };
                ImageSymbol {
                    name: s.name.clone(),
                    value: s.value,
                    size: 0,
                    section,
                    global: s.is_external(),
                    weak: s.desc & macho::N_WEAK_DEF != 0,
                    class: match section {
                        Some(k) if img.sections.get(k).is_some_and(|x| x.code) => SymbolClass::Func,
                        Some(_) => SymbolClass::Object,
                        None => SymbolClass::Other,
                    },
                }
            })
            .collect();
        img
    }

    pub fn from_elf(f: &elf::ElfFile, m: &Machine) -> Image {
        // The image keeps every section but the null one.
        let index = |shndx: u16| -> Option<usize> {
            (shndx != 0 && (shndx as usize) < f.sections.len() && shndx < 0xff00).then(|| shndx as usize - 1)
        };
        let sections = f
            .sections
            .iter()
            .skip(1)
            .map(|s| {
                let alloc = s.flags & elf::SHF_ALLOC != 0;
                let code = s.flags & elf::SHF_EXECINSTR != 0;
                ImageSection {
                    name: s.name.clone(),
                    addr: s.addr,
                    size: s.size,
                    data: s.data.clone(),
                    code,
                    kind: match (alloc, code, s.sh_type) {
                        (true, true, _) => "TEXT",
                        (true, false, elf::SHT_NOBITS) => "BSS",
                        (true, false, _) => "DATA",
                        _ => "",
                    },
                    relocs: s
                        .relocs
                        .iter()
                        .map(|r| ImageReloc {
                            offset: r.offset - if f.file_type == elf::ET_REL { 0 } else { s.addr },
                            type_name: m.elf_name(r.rel_type),
                            kind: m.by_elf(r.rel_type).map(|k| RelocKind(k as u16)),
                            target: f.reloc_target(r),
                            addend: r.addend,
                        })
                        .collect(),
                }
            })
            .collect();
        let symbols = f
            .symbols
            .iter()
            .skip(1)
            .map(|s| ImageSymbol {
                name: s.name.clone(),
                value: s.value,
                size: s.size,
                section: index(s.shndx),
                global: s.bind == elf::STB_GLOBAL,
                weak: s.bind == elf::STB_WEAK,
                class: match s.sym_type {
                    elf::STT_FUNC => SymbolClass::Func,
                    elf::STT_OBJECT => SymbolClass::Object,
                    elf::STT_SECTION => SymbolClass::Section,
                    _ if is_mapping(&s.name) => SymbolClass::Mapping,
                    _ => SymbolClass::Other,
                },
            })
            .collect();
        Image { format: m.elf_format.clone(), macho: false, sections, symbols }
    }

    /// A symbol's name as printed: a section symbol is its section's name.
    fn symbol_name(&self, s: &ImageSymbol) -> String {
        match (s.class, s.section) {
            (SymbolClass::Section, Some(k)) => self.sections[k].name.clone(),
            _ => s.name.clone(),
        }
    }
}

fn is_mapping(name: &str) -> bool {
    matches!(name, "$x" | "$d") || name.starts_with("$x.") || name.starts_with("$d.")
}

/// `-h`: the section headers.
pub fn headers(img: &Image) -> String {
    let w = img.sections.iter().map(|s| s.name.len()).max().unwrap_or(4).max(4);
    let mut out = format!("Sections:\nIdx {:w$} Size     VMA              Type\n", "Name");
    let _ = writeln!(out, "  0 {:w$} {:08x} {:016x} ", "", 0, 0);
    for (k, s) in img.sections.iter().enumerate() {
        let _ = writeln!(out, "{:>3} {:w$} {:08x} {:016x} {}", k + 1, s.name, s.size, s.addr, s.kind);
    }
    out
}

/// `-t`: the symbol table.
pub fn symbols(img: &Image) -> String {
    let mut out = String::from("SYMBOL TABLE:\n");
    for s in &img.symbols {
        let bind = if s.section.is_none() {
            ' '
        } else if s.weak {
            'w'
        } else if s.global {
            'g'
        } else {
            'l'
        };
        // objdump's flag columns: the sixth says debugging (a section
        // symbol), the seventh function or object.
        let (debug, class) = match s.class {
            SymbolClass::Func => (' ', 'F'),
            SymbolClass::Object => (' ', 'O'),
            SymbolClass::Section => ('d', ' '),
            _ => (' ', ' '),
        };
        let section = s.section.map_or("*UND*".to_string(), |k| img.sections[k].name.clone());
        let _ = writeln!(
            out,
            "{:016x} {bind}    {debug}{class} {section}\t{:016x} {}",
            s.value,
            s.size,
            img.symbol_name(s)
        );
    }
    out
}

/// `-r`: each section's relocations.
pub fn relocations(img: &Image) -> String {
    let mut out = String::new();
    for s in &img.sections {
        if s.relocs.is_empty() {
            continue;
        }
        let _ = writeln!(out, "RELOCATION RECORDS FOR [{}]:", s.name);
        let _ = writeln!(out, "OFFSET           TYPE                     VALUE");
        for r in &s.relocs {
            let _ = writeln!(out, "{:016x} {:<24} {}", r.offset, r.type_name, target_text(r));
        }
        out.push('\n');
    }
    out
}

fn target_text(r: &ImageReloc) -> String {
    match r.addend {
        0 => r.target.clone(),
        a if a > 0 => format!("{}+{a:#x}", r.target),
        a => format!("{}-{:#x}", r.target, a.unsigned_abs()),
    }
}

/// `-s`: each section's contents in hex, sixteen bytes to a line.
pub fn contents(img: &Image) -> String {
    let mut out = String::new();
    for s in &img.sections {
        if s.data.is_empty() {
            continue;
        }
        let _ = writeln!(out, "Contents of section {}:", s.name);
        for (k, chunk) in s.data.chunks(16).enumerate() {
            let mut hex = String::new();
            for (j, b) in chunk.iter().enumerate() {
                if j > 0 && j % 4 == 0 {
                    hex.push(' ');
                }
                let _ = write!(hex, "{b:02x}");
            }
            let text: String = chunk.iter().map(|&b| if (0x20..0x7f).contains(&b) { b as char } else { '.' }).collect();
            let _ = writeln!(out, " {:04x} {hex:<35}  {text}", s.addr + 16 * k as u64);
        }
    }
    out
}

/// How assembly writes a relocation's symbol in an instruction, as the
/// description spells it.
fn reloc_operand(m: &Machine, r: &ImageReloc, macho: bool) -> String {
    let t = target_text(r);
    let syntax = if macho { Syntax::MachO } else { Syntax::Elf };
    match r.kind {
        Some(k) => m.relocs[k.0 as usize].spell(syntax, &t),
        None => t,
    }
}

/// The instruction at the start of `bytes`, when a relocation of `kind`
/// fills it in.
fn decode_reloc(t: &Target, bytes: &[u8], kind: RelocKind) -> Option<(usize, Env, usize)> {
    // Relocations an object file numbers alike are one to it (Mach-O's
    // BRANCH26 is both a call's and a tail call's).
    let defs = &t.machine.relocs;
    let k = &defs[kind.0 as usize];
    let alike = |r: RelocKind| {
        let d = &defs[r.0 as usize];
        r == kind || (d.macho.is_some() && d.macho == k.macho) || (d.elf.is_some() && d.elf == k.elf)
    };
    t.decode_candidates(bytes)
        .into_iter()
        .filter(|&(i, _)| t.instrs[i].reloc.is_some_and(alike))
        .find_map(|(i, alt)| t.instrs[i].decode_alt(bytes, alt).map(|(env, len)| (i, env, len)))
}

/// `-d`: the code sections disassembled.
pub fn disassemble(t: &Target, img: &Image, show_relocs: bool) -> String {
    let mut out = String::new();
    let unit = t.unit();
    // Symbols to name addresses by: functions, objects and others, not
    // section or mapping symbols.
    let mut named: Vec<(u64, usize, String)> = img
        .symbols
        .iter()
        .filter(|s| matches!(s.class, SymbolClass::Func | SymbolClass::Object | SymbolClass::Other))
        .filter_map(|s| s.section.map(|k| (img.sections[k].addr + s.value_in(img, k), k, s.name.clone())))
        // Not Mach-O's assembler temporaries (`ltmp0`, at each section's
        // start).
        .filter(|(_, _, n)| !(n.is_empty() || img.macho && n.starts_with("ltmp")))
        .collect();
    named.sort();
    named.dedup();
    // The first named at each place, by section and address.
    let mut at_addr: std::collections::HashMap<(usize, u64), &String> = std::collections::HashMap::new();
    for (a, k, n) in &named {
        at_addr.entry((*k, *a)).or_insert(n);
    }
    for (k, s) in img.sections.iter().enumerate() {
        if !s.code || s.data.is_empty() {
            continue;
        }
        let _ = writeln!(out, "\nDisassembly of section {}:", s.name);
        let here: Vec<&(u64, usize, String)> = named.iter().filter(|x| x.1 == k).collect();
        if let Some(f) = t.machine.bundles.iter().find(|f| f.holds(&s.name)) {
            bundles(t, f, s, &here, &mut out);
            continue;
        }
        // Where code and data start, by the mapping symbols.
        let mut mapping: Vec<(u64, bool)> = img
            .symbols
            .iter()
            .filter(|m| m.class == SymbolClass::Mapping && m.section == Some(k))
            .map(|m| (m.value_in(img, k), m.name.starts_with("$d")))
            .collect();
        mapping.sort();
        let is_data = |off: u64| {
            let k = mapping.partition_point(|m| m.0 <= off);
            k > 0 && mapping[k - 1].1
        };
        // A target address, with the symbol it is in (`here` is in address
        // order: found by halving).
        let label = |addr: u64| -> String {
            let k = here.partition_point(|x| x.0 <= addr);
            let sym = if k > 0 { Some(here[k - 1]) } else { here.first().copied().filter(|x| x.0 == addr) };
            match sym {
                Some((a, _, n)) if *a == addr => format!("{addr:#x} <{n}>"),
                Some((a, _, n)) => format!("{addr:#x} <{n}+{:#x}>", addr - a),
                None => format!("{addr:#x}"),
            }
        };
        if here.first().is_none_or(|x| x.0 > s.addr) {
            let _ = writeln!(out, "\n{:016x} <{}>:", s.addr, s.name);
        }
        // The relocations by where they are.
        let mut by_offset: Vec<&ImageReloc> = s.relocs.iter().collect();
        by_offset.sort_by_key(|r| r.offset);
        let mut off = 0u64;
        // The next symbol to write, as the code reaches it.
        let mut next = 0;
        while off < s.data.len() as u64 {
            let addr = s.addr + off;
            while next < here.len() && here[next].0 < addr {
                next += 1;
            }
            while next < here.len() && here[next].0 == addr {
                let _ = writeln!(out, "\n{addr:016x} <{}>:", here[next].2);
                next += 1;
            }
            let rest = &s.data[off as usize..];
            // (Less than an instruction's usual length left may still be a
            // shorter one: RISC-V's compressed.)
            if rest.len() < unit && (is_data(off) || mir::decode(t, rest).is_none()) {
                let bytes: Vec<String> = rest.iter().map(|b| format!("{b:02x}")).collect();
                let _ = writeln!(out, "{addr:>8x}: {:<8}     \t.byte {}", bytes.join(""), bytes.join(", "));
                break;
            }
            // Its length, then the relocations in it.
            let len = match mir::decode(t, rest) {
                _ if is_data(off) => 4,
                Some((_, _, n)) => n,
                None => unit,
            };
            let from = by_offset.partition_point(|r| r.offset < off);
            let relocs: Vec<&ImageReloc> =
                by_offset[from..].iter().copied().take_while(|r| r.offset < off + len as u64).collect();
            let (text, len) = if is_data(off) {
                (format!(".word {:#010x}", enc::word(&rest[..4])), 4)
            } else {
                // Against a section: the symbol there, if one is.
                // (Its addend, less what the instruction adds itself.)
                let named = |r: &mut ImageReloc| {
                    if let Some(sk) = img.sections.iter().position(|x| x.name == r.target)
                        && let Some(n) = at_addr.get(&(sk, img.sections[sk].addr + r.addend as u64))
                    {
                        r.target = (*n).clone();
                        r.addend = 0;
                    }
                };
                instruction(t, rest, addr, relocs.first().copied(), img.macho, &label, &named)
            };
            // A word as a word, as other disassemblers show one, and RISC-V's
            // compressed instructions as halfwords; other lengths as their
            // bytes.
            let hex = if unit == 4 && len == 4 {
                format!("{:08x}", enc::word(&rest[..4]))
            } else if unit == 4 && len == 2 {
                format!("{:04x}", u16::from_le_bytes([rest[0], rest[1]]))
            } else {
                rest[..len].iter().map(|b| format!("{b:02x}")).collect::<Vec<_>>().join(" ")
            };
            let _ = writeln!(out, "{addr:>8x}: {hex}     \t{text}");
            if show_relocs {
                for r in relocs {
                    let _ = writeln!(out, "\t\t{:016x}:  {}\t{}", s.addr + r.offset, r.type_name, target_text(r));
                }
            }
            off += len as u64;
        }
    }
    out
}

/// A section of bundles of format `f`, each written as the assembler
/// reads it: its prefix and `{`, an instruction for each slot that is not
/// zero (`:: ` before one of lane 1), then `}`.
fn bundles(t: &Target, f: &crate::bundle::Format, s: &ImageSection, here: &[&(u64, usize, String)], out: &mut String) {
    let size = f.bytes as usize;
    let count = s.data.len() / size;
    for b in 0..count {
        let addr = s.addr + (b * size) as u64;
        for (_, _, name) in here.iter().filter(|x| x.0 == addr) {
            let _ = writeln!(out, "\n{addr:016x} <{name}>:");
        }
        let bytes = &s.data[b * size..(b + 1) * size];
        let mut explained: std::collections::HashMap<String, i64> = std::collections::HashMap::new();
        if b + 1 == count {
            explained.extend(f.end.iter().cloned());
        }
        let mut lines = Vec::new();
        for slot in &f.slots {
            let word = crate::bundle::get(bytes, slot.lsb, slot.width);
            if word == 0 {
                continue;
            }
            let wb = word.to_le_bytes();
            let wb = &wb[..(slot.width / 8) as usize];
            match decode_slot(t, f, slot, wb, bytes) {
                Some((i, env, sets)) => {
                    explained.extend(sets);
                    let ins = &t.instrs[i];
                    let text = mir::fill_template(t, None, i, |k| Operand::Imm(env[&ins.operands[k].name]));
                    let lane = ins.slot.as_ref().and_then(|u| u.lane).unwrap_or(0);
                    let sep = f.lanes.as_deref().filter(|_| lane == 1).map_or(String::new(), |l| format!("{l} "));
                    lines.push(format!("{sep}{text}"));
                }
                None => {
                    lines.push(format!("{} .word {word:#x} (slot {})", t.machine.asm_comment, slot.units.join(",")))
                }
            }
        }
        let prefix = match f.write_prefix(bytes, &explained) {
            Some(p) if p.is_empty() => String::new(),
            Some(p) => format!("{p} "),
            None => format!("{} header not written by any prefix: ", t.machine.asm_comment),
        };
        let _ = writeln!(out, "{addr:>8x}: {prefix}{{");
        for l in lines {
            let _ = writeln!(out, "\t\t{l}");
        }
        let _ = writeln!(out, "\t}}");
    }
}

/// The entry in `slot` of a bundle whose bytes are `bundle`, its operands,
/// and the header fields they explain. Where the slot alone does not give
/// an operand (a register whose number is in the header), each value of its
/// enum that encodes the same is tried, and the one whose header fields
/// agree with the header, and explain most of it, taken.
pub fn decode_slot(
    t: &Target,
    f: &crate::bundle::Format,
    slot: &crate::bundle::SlotDef,
    word: &[u8],
    bundle: &[u8],
) -> Option<(usize, Env, Vec<(String, i64)>)> {
    let header = |name: &str| f.field(name).map(|x| crate::bundle::get(bundle, x.lsb, x.width) as i64);
    for (i, ins) in t.instrs.iter().enumerate() {
        let Some(u) = &ins.slot else { continue };
        if !ins.decode || !u.units.iter().any(|x| slot.units.contains(x)) {
            continue;
        }
        let Some((mut env, len)) = ins.decode(word) else { continue };
        if len != word.len() {
            continue;
        }
        // Each enum operand the header fields depend on.
        let mut names = Vec::new();
        for a in &u.header {
            for e in a.index.iter().chain([&a.value]).chain(a.guard.iter()) {
                enc::names_in(e, &mut names);
            }
        }
        // The header fields an assignment sets, with those of `only` (when
        // given) the operand it is worked out from alone.
        let mentions = |a: &crate::bundle::Assign| {
            let mut m = Vec::new();
            for e in a.index.iter().chain([&a.value]).chain(a.guard.iter()) {
                enc::names_in(e, &mut m);
            }
            m
        };
        let score_of = |env: &Env, only: Option<&String>| -> Option<(usize, Vec<(String, i64)>)> {
            let mut sets = Vec::new();
            for a in &u.header {
                if let Some(n) = only
                    && mentions(a).iter().any(|m| m != n && names.contains(m))
                {
                    continue;
                }
                if let Some((n, v)) = a.eval(env)? {
                    if header(&n)? != v {
                        return None;
                    }
                    sets.push((n, v));
                }
            }
            Some((sets.len(), sets))
        };
        let score = |env: &Env| score_of(env, None);
        let mut ok = true;
        for n in &names {
            let Some(o) = ins.operands.iter().find(|o| &o.name == n) else { continue };
            let Kind::Enum(e) = o.kind else { continue };
            let mut best: Option<(usize, i64)> = None;
            let mut seen = std::collections::HashSet::new();
            for &(_, v) in &t.machine.enums[e as usize].names {
                if !seen.insert(v) {
                    continue;
                }
                let mut try_env = env.clone();
                try_env.insert(n.clone(), v);
                if ins.encode(&try_env).as_deref() != Some(word) {
                    continue;
                }
                if let Some((k, _)) = score_of(&try_env, Some(n))
                    && best.is_none_or(|b| k > b.0)
                {
                    best = Some((k, v));
                }
            }
            match best {
                Some((_, v)) => {
                    env.insert(n.clone(), v);
                }
                None => ok = false,
            }
        }
        if !ok {
            continue;
        }
        if let Some((_, sets)) = score(&env) {
            return Some((i, env, sets));
        }
    }
    None
}

impl ImageSymbol {
    /// The symbol's offset in section `k`: its value, which in files that
    /// are linked is an address.
    fn value_in(&self, img: &Image, k: usize) -> u64 {
        let s = &img.sections[k];
        if self.value >= s.addr && s.addr != 0 { self.value - s.addr } else { self.value }
    }
}

/// The instruction at the start of `bytes`, as assembly, and its length;
/// `reloc` is the relocation that fills it in, if there is one, and `named`
/// names a section-relative one's place by a symbol there.
pub fn instruction(
    t: &Target,
    bytes: &[u8],
    pc: u64,
    reloc: Option<&ImageReloc>,
    macho: bool,
    label: &dyn Fn(u64) -> String,
    named: &dyn Fn(&mut ImageReloc),
) -> (String, usize) {
    let decoded = match reloc.and_then(|r| r.kind) {
        Some(k) => decode_reloc(t, bytes, k).or_else(|| mir::decode(t, bytes)),
        None => mir::decode(t, bytes),
    };
    let Some((i, env, len)) = decoded else {
        return match t.unit() {
            4 if bytes.len() >= 4 => (format!(".word {:#010x}", enc::word(&bytes[..4])), 4),
            _ => (format!(".byte {:#04x}", bytes[0]), 1),
        };
    };
    let ins = &t.instrs[i];
    let text = mir::fill_template(t, None, i, |k| {
        let o = &ins.operands[k];
        let v = env[&o.name];
        match o.kind {
            Kind::Reg(c) => Operand::Reg(Reg::P(t.machine.phys(c, v))),
            Kind::Label => Operand::Text(label(pc.wrapping_add(v as u64))),
            Kind::Sym => match reloc {
                // Less what the instruction's field adds itself (x86's
                // PC32 addend of -4 is the field's end, not the symbol's).
                Some(r) => {
                    let bias = ins.encode_alt(&env).map_or(0, |(_, alt)| {
                        let (at, a) = ins.field_of(alt, &o.name, &env);
                        let pc_rel = r.kind.is_some_and(|k| t.machine.relocs[k.0 as usize].pc_relative());
                        a + if pc_rel { at as i64 } else { 0 }
                    });
                    let mut r = r.clone();
                    r.addend -= bias;
                    named(&mut r);
                    Operand::Text(reloc_operand(&t.machine, &r, macho))
                }
                None => match ins.reloc.and_then(|kind| t.machine.relocs[kind.0 as usize].reaches(pc, v)) {
                    Some(a) => Operand::Text(label(a)),
                    None => Operand::Imm(v),
                },
            },
            Kind::Imm | Kind::Cond | Kind::Enum(_) => Operand::Imm(v),
        }
    });
    (text, len)
}

/// Checks that `img`'s code disassembles as `asm` (the assembly it was
/// made from) says: the same number of instructions, each with the same
/// mnemonic (or one `aliases` pairs with it), and where a relocation fills
/// an instruction in, the same instruction, symbol and all. Returns how
/// many instructions there are.
pub fn reads_back(t: &Target, img: &Image, asm: &str, aliases: &[(&str, &str)]) -> Result<usize, String> {
    let dump = disassemble(t, img, true);
    // Where functions are, by their symbols' sizes: what lies between them
    // is padding (x86's no-ops), not written as instructions.
    let funcs: Vec<(u64, u64)> = img
        .symbols
        .iter()
        .filter(|x| x.class == SymbolClass::Func && x.size > 0)
        .filter_map(|x| x.section.map(|k| (img.sections[k].addr + x.value_in(img, k), x.size)))
        .collect();
    let padding = |addr: u64| !funcs.is_empty() && !funcs.iter().any(|&(a, n)| addr >= a && addr < a + n);
    // Each instruction: its text, and whether a relocation fills it in.
    let mut lines: Vec<(String, bool)> = Vec::new();
    let mut skipping = false;
    for l in dump.lines() {
        if let Some((addr, _)) = l.split_once(": ")
            && let Ok(a) = u64::from_str_radix(addr.trim(), 16)
            && !l.starts_with('\t')
        {
            skipping = padding(a);
        }
        if skipping {
            continue;
        }
        if l.starts_with("\t\t") {
            if let Some(last) = lines.last_mut() {
                last.1 = true;
            }
        } else if let Some(more) = l.strip_prefix('\t') {
            // An instruction written on two lines (`.tlsdesccall x`, then
            // `blr x1`): its last is the instruction, as written.
            if let Some(last) = lines.last_mut() {
                last.0 = more.to_string();
            }
        } else if let Some((_, rest)) = l.split_once(": ")
            && let Some((_, text)) = rest.split_once('\t')
        {
            lines.push((text.to_string(), false));
        }
    }
    let written: Vec<&str> = asm.lines().filter_map(|l| l.strip_prefix('\t')).filter(|l| !l.starts_with('.')).collect();
    if written.len() != lines.len() {
        return Err(format!("{} instructions read back, {} written", lines.len(), written.len()));
    }
    let mnemonic = |s: &str| s.split_whitespace().next().unwrap_or("").to_string();
    for (k, ((read, relocated), wrote)) in lines.iter().zip(&written).enumerate() {
        let (a, b) = (mnemonic(read), mnemonic(wrote));
        let alias = aliases.contains(&(a.as_str(), b.as_str())) || aliases.contains(&(b.as_str(), a.as_str()));
        if (a != b && !alias) || (*relocated && read != wrote) {
            return Err(format!("instruction {k} reads back as `{read}`, was written `{wrote}`"));
        }
    }
    Ok(lines.len())
}
