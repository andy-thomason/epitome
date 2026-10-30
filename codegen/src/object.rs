//! Object files in the abstract: sections of bytes, symbols, and
//! relocations, built from compiled functions and globals. `elf.rs` writes
//! one as an ELF relocatable file, `macho.rs` as a Mach-O object.
//!
//! Globals are first described as [`GlobalData`], a list of [`Item`]s,
//! which both the assembly printer (`asm.rs`) and [`build`] read, so that
//! the two outputs say the same thing.
//!
//! Building does what an assembler for the convention's format does with
//! the same program:
//!
//! - a call to a function defined in the same section is resolved in place
//!   (the distance is known), with no relocation. In ELF only local
//!   functions: a global one could be replaced by another library's
//!   definition, so is left to the linker. In Mach-O, whose symbols are
//!   bound to the library that defines them, global ones too;
//! - in ELF, other references to a local symbol are made against its
//!   section, with the symbol's offset in the addend, so that the symbol
//!   need not be kept for the linker; references through the GOT stay
//!   against the symbol. Mach-O keeps every reference against its symbol;
//! - references to global and undefined symbols are made against them;
//! - the bytes a relocation will fill are zero, the addend being in the
//!   relocation. (The Mach-O writer moves a pointer's addend into the
//!   pointer's bytes, where that format keeps it.)

use std::collections::HashMap;

use crate::abi::{Abi, Syntax};
use crate::machine::RelocUse;
use crate::mir::{self, MFunc};
use crate::target::{Kind, Target};

/// How a relocation fills in its place: an index into the description's
/// relocations (`Machine::relocs`), which say what each is called in each
/// object format and how assembly writes it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct RelocKind(pub u16);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum SectionKind {
    /// Code.
    Text,
    /// Writable data.
    Data,
    /// Writable data that starts as zeros: only its size is stored.
    Bss,
    /// Constants.
    ReadOnly,
    /// Constants holding addresses, which the loader writes once.
    RelRo,
    /// Unwind tables: DWARF's call frame information (`unwind.rs`).
    EhFrame,
    /// Exception tables: personality routines' data (`eh.rs`).
    ExceptTable,
    /// Thread-local variables' descriptors (Mach-O's `__thread_vars`):
    /// for each, the function giving its address, and where its initial
    /// value is.
    ThreadVars,
    /// Thread-local variables' initial values, copied for each thread.
    ThreadData,
}

#[derive(Clone, Debug)]
pub struct Section {
    pub kind: SectionKind,
    /// A section of a name of its own (`#[link_section]`), rather than
    /// the kind's: for Mach-O `segment,section[,type[,attributes]]`, for
    /// ELF the name. The kind says what it holds.
    pub name: Option<String>,
    pub align: u64,
    /// The contents; empty for `Bss`, whose size is `bss_size`.
    pub data: Vec<u8>,
    pub bss_size: u64,
    pub relocs: Vec<Reloc>,
}

impl Section {
    pub fn size(&self) -> u64 {
        if self.kind == SectionKind::Bss { self.bss_size } else { self.data.len() as u64 }
    }
}

/// What a relocation refers to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RelocTarget {
    Symbol(usize),
    /// The start of a section (the addend says where in it).
    Section(usize),
}

#[derive(Clone, Debug)]
pub struct Reloc {
    pub offset: u64,
    pub kind: RelocKind,
    pub target: RelocTarget,
    pub addend: i64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SymbolKind {
    Func,
    Object,
    /// Undefined: whatever another file defines.
    NoType,
}

#[derive(Clone, Debug)]
pub struct Symbol {
    pub name: String,
    /// The section defining it; `None` if it is undefined.
    pub section: Option<usize>,
    pub value: u64,
    pub size: u64,
    pub global: bool,
    pub kind: SymbolKind,
}

#[derive(Clone, Debug, Default)]
pub struct Object {
    pub sections: Vec<Section>,
    pub symbols: Vec<Symbol>,
    /// The global symbols that are weak (ELF's `.weak`): another
    /// definition may take their place, and they need none.
    pub weak: Vec<String>,
}

/// A piece of a global's initial value.
#[derive(Clone, Debug, PartialEq)]
pub enum Item {
    /// An integer of `size` bytes (a float's bits, if `hex`, which is how
    /// assembly prints it).
    Int {
        size: u8,
        value: u64,
        hex: bool,
    },
    /// Bytes, as a string.
    Bytes(Vec<u8>),
    Zero(u64),
    /// The 64-bit address of `symbol` (as the platform names it), plus
    /// `offset`.
    Addr {
        symbol: String,
        offset: i64,
    },
    /// The 32-bit distance from the start of the global holding it
    /// (`base`, as the platform names it) to `symbol` (a relative lookup
    /// table's entry: `.long symbol-base`).
    Rel32 {
        symbol: String,
        base: String,
    },
}

impl Item {
    pub fn size(&self) -> u64 {
        match self {
            Item::Int { size, .. } => *size as u64,
            Item::Bytes(b) => b.len() as u64,
            Item::Zero(n) => *n,
            Item::Addr { .. } => 8,
            Item::Rel32 { .. } => 4,
        }
    }
}

/// A global variable: where it goes and what it starts as.
#[derive(Clone, Debug)]
pub struct GlobalData {
    /// As the platform names it.
    pub symbol: String,
    pub global: bool,
    pub section: SectionKind,
    /// The section's own name, if it is not the kind's (`Section::name`).
    pub section_name: Option<String>,
    pub align: u64,
    pub items: Vec<Item>,
}

impl GlobalData {
    pub fn size(&self) -> u64 {
        self.items.iter().map(Item::size).sum()
    }
}

/// A global alias: another name for an address in this file, `target`
/// (as the platform names it) plus `offset`.
#[derive(Clone, Debug)]
pub struct Alias {
    pub symbol: String,
    pub target: String,
    pub offset: i64,
    pub global: bool,
}

/// Defines each alias as a symbol at its target's place. (The target must
/// be defined here: an alias is of a definition.)
pub fn add_aliases(obj: &mut Object, aliases: &[Alias]) -> Result<(), String> {
    for a in aliases {
        let t = obj
            .symbols
            .iter()
            .find(|s| s.name == a.target && s.section.is_some())
            .ok_or_else(|| format!("{}: an alias of {}, which is not defined here", a.symbol, a.target))?
            .clone();
        obj.symbols.push(Symbol {
            name: a.symbol.clone(),
            section: t.section,
            value: (t.value as i64 + a.offset) as u64,
            size: if a.offset == 0 { t.size } else { 0 },
            global: a.global,
            kind: t.kind,
        });
    }
    Ok(())
}

/// A reference to a symbol, by name, before the symbols are all known.
pub struct Pending {
    pub section: usize,
    pub offset: u64,
    pub kind: RelocKind,
    /// As the platform names it.
    pub symbol: String,
    pub addend: i64,
    /// The instruction holding it (an index into the description), for
    /// those resolved in place.
    pub instr: Option<usize>,
}

/// The object for compiled functions (laid out and allocated) and globals.
pub fn build(
    t: &Target,
    abi: &Abi,
    funcs: &[MFunc],
    globals: &[GlobalData],
    aliases: &[Alias],
) -> Result<Object, String> {
    let mut obj = Object::default();
    let mut by_kind: HashMap<(SectionKind, Option<String>), usize> = HashMap::new();
    let mut section = |obj: &mut Object, kind: SectionKind, name: Option<&String>| -> usize {
        *by_kind.entry((kind, name.cloned())).or_insert_with(|| {
            obj.sections.push(Section {
                kind,
                name: name.cloned(),
                align: 1,
                data: Vec::new(),
                bss_size: 0,
                relocs: Vec::new(),
            });
            obj.sections.len() - 1
        })
    };
    // Pads section `s` to `align`, and returns where the next thing goes.
    let place = |obj: &mut Object, s: usize, align: u64| -> u64 {
        let sec = &mut obj.sections[s];
        sec.align = sec.align.max(align);
        let at = sec.size().next_multiple_of(align);
        if sec.kind == SectionKind::Bss {
            sec.bss_size = at;
        } else if sec.kind == SectionKind::Text {
            let fill = t.machine.code_fill(at as usize - sec.data.len());
            sec.data.extend(fill);
        } else {
            sec.data.resize(at as usize, 0);
        }
        at
    };
    let mut pending = Vec::new();
    let mut frames = Vec::new();

    // Each function's machine code, in parallel.
    let threads = std::thread::available_parallelism().map_or(1, |n| n.get());
    let all: Vec<usize> = (0..funcs.len()).collect();
    let encoded = crate::par_run(funcs.len(), &all, threads, || (), |_, k| mir::encode_function(t, &funcs[k]), || ());
    for (f, code) in funcs.iter().zip(encoded) {
        let s = section(&mut obj, SectionKind::Text, None);
        let at = place(&mut obj, s, 4);
        let code = code?;
        obj.sections[s].data.extend_from_slice(&code.bytes);
        obj.symbols.push(Symbol {
            name: abi.symbol(&f.name),
            section: Some(s),
            value: at,
            size: code.bytes.len() as u64,
            global: f.global,
            kind: SymbolKind::Func,
        });
        let symbol = obj.symbols.len() - 1;
        // Its exception table, if it has landing pads, its references to
        // the types caught through their GOT entries.
        let mut lsda = None;
        if let Some(table) = crate::eh::lsda(t, f, &|n| abi.eh_ref(n))? {
            let es = section(&mut obj, SectionKind::ExceptTable, None);
            let eat = place(&mut obj, es, 4);
            obj.sections[es].data.extend(&table.bytes);
            // A symbol at it, for the unwind tables to name (unwind.rs).
            obj.symbols.push(Symbol {
                name: format!("GCC_except_table{}", frames.len()),
                section: Some(es),
                value: eat,
                size: 0,
                global: false,
                kind: SymbolKind::NoType,
            });
            let macho = abi.syntax == Syntax::MachO;
            let usage = if macho { RelocUse::GotPcRel32 } else { RelocUse::EhPcRel };
            let kind = t.machine.reloc_used(usage).ok_or_else(|| format!("no relocation has use {usage:?}"))?;
            for (off, name) in table.type_refs {
                let field = eat + off;
                if macho {
                    obj.sections[es].data[field as usize..field as usize + 4]
                        .copy_from_slice(&crate::unwind::macho_got_field(field));
                }
                pending.push(Pending {
                    section: es,
                    offset: field,
                    kind: RelocKind(kind as u16),
                    symbol: name,
                    addend: 0,
                    instr: None,
                });
            }
            lsda = Some((es, eat));
        }
        frames.push(crate::unwind::Frame {
            section: s,
            start: at,
            len: code.bytes.len() as u64,
            symbol,
            rules: mir::cfi_rules(f),
            personality: f.eh.as_ref().map(|e| abi.eh_ref(&e.personality)),
            lsda,
        });
        for fx in code.fixups {
            let i = t.by_name[&fx.instr];
            let kind = t.instrs[i].reloc.ok_or_else(|| format!("{}: {} has no \"reloc\"", f.name, fx.instr))?;
            // The field measures from the instruction's start; a
            // PC-relative relocation, from the field's.
            let pc_rel = t.machine.relocs[kind.0 as usize].pc_relative();
            pending.push(Pending {
                section: s,
                offset: at + fx.offset as u64,
                kind,
                symbol: abi.symbol(&fx.symbol),
                addend: fx.addend + if pc_rel { fx.at as i64 } else { 0 },
                instr: Some(i),
            });
        }
    }

    for g in globals {
        let s = section(&mut obj, g.section, g.section_name.as_ref());
        let at = place(&mut obj, s, g.align.max(1));
        let bss = g.section == SectionKind::Bss;
        for item in &g.items {
            let sec = &mut obj.sections[s];
            match item {
                Item::Zero(n) if bss => sec.bss_size += n,
                _ if bss => return Err(format!("{}: a value other than zero in .bss", g.symbol)),
                Item::Int { size, value, .. } => sec.data.extend_from_slice(&value.to_le_bytes()[..*size as usize]),
                Item::Bytes(b) => sec.data.extend_from_slice(b),
                Item::Zero(n) => sec.data.resize(sec.data.len() + *n as usize, 0),
                Item::Addr { symbol, offset } => {
                    let kind =
                        t.machine.data_reloc(8).ok_or("the description has no relocation for pointers in data")?;
                    pending.push(Pending {
                        section: s,
                        offset: sec.data.len() as u64,
                        kind: RelocKind(kind as u16),
                        symbol: symbol.clone(),
                        addend: *offset,
                        instr: None,
                    });
                    sec.data.extend_from_slice(&[0; 8]);
                }
                // PC-relative, from the field: plus its distance from the
                // global's start.
                Item::Rel32 { symbol, base } => {
                    if *base != g.symbol || abi.syntax == Syntax::MachO {
                        return Err(format!("{}: a distance from another global in data", g.symbol));
                    }
                    let kind = t.machine.reloc_used(RelocUse::EhPcRel).ok_or("no relocation with use eh-pcrel")?;
                    pending.push(Pending {
                        section: s,
                        offset: sec.data.len() as u64,
                        kind: RelocKind(kind as u16),
                        symbol: symbol.clone(),
                        addend: (sec.data.len() as u64 - at) as i64,
                        instr: None,
                    });
                    sec.data.extend_from_slice(&[0; 4]);
                }
            }
        }
        obj.symbols.push(Symbol {
            name: g.symbol.clone(),
            section: Some(s),
            value: at,
            size: g.size(),
            global: g.global,
            kind: SymbolKind::Object,
        });
    }

    add_aliases(&mut obj, aliases)?;
    resolve(t, &mut obj, pending, abi.syntax == Syntax::MachO)?;
    let p = crate::unwind::Params { syntax: abi.syntax, data_align: t.machine.data_align(abi.syntax), ra: abi.lr };
    crate::unwind::add_eh_frame(&mut obj, &t.machine, &p, &frames)?;
    Ok(obj)
}

/// Turns references by name into relocations (or resolves them in place),
/// by the rules of the format (see the top of this file): Mach-O's if
/// `macho`, else ELF's. Symbols not defined are added as undefined.
pub fn resolve(t: &Target, obj: &mut Object, pending: Vec<Pending>, macho: bool) -> Result<(), String> {
    let mut by_name: HashMap<String, usize> =
        obj.symbols.iter().enumerate().map(|(k, s)| (s.name.clone(), k)).collect();
    for p in pending {
        let sym = match by_name.get(&p.symbol) {
            Some(&k) => k,
            None => {
                obj.symbols.push(Symbol {
                    name: p.symbol.clone(),
                    section: None,
                    value: 0,
                    size: 0,
                    global: true,
                    kind: SymbolKind::NoType,
                });
                by_name.insert(p.symbol.clone(), obj.symbols.len() - 1);
                obj.symbols.len() - 1
            }
        };
        let def = obj.symbols[sym].clone();
        // A call within the section: the distance is known.
        let rd = &t.machine.relocs[p.kind.0 as usize];
        let usage = rd.usage;
        // So is any other plain PC-relative reference to a local symbol
        // there (x86's `leaq sym(%rip)`), for ELF: in a 32-bit field of its
        // own, it is what a linker would write, S + A - P.
        let plain = !macho
            && usage != RelocUse::Call
            && !def.global
            && def.section == Some(p.section)
            && rd.target.as_ref().is_some_and(|e| {
                use crate::enc::Expr;
                matches!(e, Expr::Bin('+', a, b) if **a == Expr::Var("pc".into()) && **b == Expr::Var("v".into()))
            });
        if plain && let Some(i) = p.instr {
            let ins = &t.instrs[i];
            let op = ins.operands.iter().find(|o| o.kind == Kind::Sym).ok_or("a reference without a symbol operand")?;
            let whole = ins.enc.alts.iter().all(|a| {
                a.fields
                    .iter()
                    .filter(|x| x.names.contains(&op.name))
                    .all(|x| x.width == 32 && x.lo == 0 && a.pieces[x.piece].bytes == 4)
            });
            if whole {
                let v = def.value as i64 + p.addend - p.offset as i64;
                let v = i32::try_from(v).map_err(|_| format!("{}: {} is out of range", ins.name, def.name))?;
                let at = p.offset as usize;
                obj.sections[p.section].data[at..at + 4].copy_from_slice(&v.to_le_bytes());
                continue;
            }
        }
        if usage == RelocUse::Call
            && (macho || !def.global)
            && def.section == Some(p.section)
            && let Some(i) = p.instr
        {
            let ins = &t.instrs[i];
            let op = ins.operands.iter().find(|o| o.kind == Kind::Sym).ok_or("a call without a symbol operand")?;
            // The instruction's start: the field is `at` into it.
            let at_in = ins
                .enc
                .alts
                .iter()
                .find_map(|a| a.fields.iter().find(|x| x.names.contains(&op.name)).map(|x| a.pieces[x.piece].offset))
                .unwrap_or(0) as usize;
            let start = p.offset as usize - at_in;
            let data = &mut obj.sections[p.section].data;
            let (mut env, len) =
                ins.decode(&data[start..]).ok_or_else(|| format!("cannot decode the {} to {}", ins.name, def.name))?;
            // (A call's symbol has no addend of its own: the field's
            // expression gives the rest.)
            env.insert(op.name.clone(), def.value as i64 - start as i64);
            let bytes =
                ins.encode(&env).ok_or_else(|| format!("{}: the call to {} is out of range", ins.name, def.name))?;
            if bytes.len() != len {
                return Err(format!("{}: the call to {} changed size", ins.name, def.name));
            }
            data[start..start + len].copy_from_slice(&bytes);
            continue;
        }
        let (target, addend) = match def.section {
            Some(s) if !macho && !def.global && !matches!(usage, RelocUse::Got | RelocUse::Tls | RelocUse::PcRelLo) => {
                (RelocTarget::Section(s), p.addend + def.value as i64)
            }
            _ => (RelocTarget::Symbol(sym), p.addend),
        };
        obj.sections[p.section].relocs.push(Reloc { offset: p.offset, kind: p.kind, target, addend });
    }
    Ok(())
}

/// Where two canonical forms (`ElfFile::canonical`, `MachFile::canonical`)
/// differ: up to `max` lines of each that are not in the other.
pub fn differences(ours: &str, theirs: &str, max: usize) -> Vec<String> {
    let a: std::collections::HashSet<&str> = ours.lines().collect();
    let b: std::collections::HashSet<&str> = theirs.lines().collect();
    let mut out: Vec<String> =
        ours.lines().filter(|l| !b.contains(l)).take(max).map(|l| format!("ours:   {l}")).collect();
    out.extend(theirs.lines().filter(|l| !a.contains(l)).take(max).map(|l| format!("theirs: {l}")));
    out
}
