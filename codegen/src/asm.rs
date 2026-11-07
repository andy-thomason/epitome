//! Assembly output, in the dialect of the platform's assembler (Mach-O or
//! ELF): functions from their machine IR, and global variables as data
//! directives, from the same description of them (`object::GlobalData`)
//! that object files are written from.

use std::fmt::Write;

use epitome_ir::module::*;
use epitome_ir::types::{Type, TypeId};

use crate::abi::{Abi, Syntax};
use crate::mir::*;
use crate::object::{self, GlobalData, Item, SectionKind};
use crate::prepare::{self, Ctx};
use crate::target::{Target, sext};

/// Writes function `f`, the `index`th, with its exception table if it has
/// landing pads.
pub fn function(t: &Target, f: &MFunc, index: usize, out: &mut String) -> Result<(), String> {
    let abi = &f.abi;
    let sym = abi.symbol(&f.name);
    let table = crate::eh::lsda(t, f, &|n| abi.eh_ref(n))?;
    let table_label = format!("{}exception{index}", abi.local_prefix());
    let _ = writeln!(out, "\t.p2align 2");
    if f.global {
        let _ = writeln!(out, "\t.globl {sym}");
    }
    if abi.syntax == Syntax::Elf {
        let _ = writeln!(out, "\t.type {sym},%function");
    }
    let _ = writeln!(out, "{sym}:");
    // No unwind tables for a machine without calls: nothing unwinds.
    let cfi = !t.machine.lacks("calls");
    if cfi {
        let _ = writeln!(out, "\t.cfi_startproc");
    }
    if let (Some(eh), Some(_)) = (&f.eh, &table) {
        // Indirect, PC-relative, 4 bytes; and PC-relative (Mach-O's 8
        // bytes, ELF's 4).
        let _ = writeln!(out, "\t.cfi_personality 155, {}", abi.eh_ref(&eh.personality));
        let lsda_enc = if abi.syntax == Syntax::Elf { 27 } else { 16 };
        let _ = writeln!(out, "\t.cfi_lsda {lsda_enc}, {table_label}");
    }
    let mut referenced = vec![false; f.blocks.len()];
    for b in &f.blocks {
        for i in &b.insts {
            for op in &i.ops {
                if let MOp::Label(x) = op {
                    referenced[*x] = true;
                }
            }
        }
    }
    for (k, b) in f.blocks.iter().enumerate() {
        if k > 0 && referenced[k] {
            let _ = writeln!(out, "{}:", b.label);
        }
        for i in &b.insts {
            if !cfi && matches!(i.op, crate::mir::Opcode::Cfi(_)) {
                continue;
            }
            let _ = writeln!(out, "\t{}", format_inst(t, f, i));
        }
    }
    if cfi {
        let _ = writeln!(out, "\t.cfi_endproc");
    }
    if abi.syntax == Syntax::Elf {
        let _ = writeln!(out, "\t.size {sym}, .-{sym}");
    }
    if let Some(table) = table {
        // The table's bytes, its references to types (through the GOT, or
        // a word of ours) each relative to a label at it.
        let _ = writeln!(out, "\t{}", abi.section_directive(SectionKind::ExceptTable));
        let _ = writeln!(out, "\t.p2align 2");
        // A symbol at it too, for the unwind tables to name.
        let _ = writeln!(out, "GCC_except_table{index}:\n{table_label}:");
        let mut at = 0;
        let mut refs = table.type_refs.iter().peekable();
        let mut n = 0;
        while at < table.bytes.len() {
            if let Some((off, name)) = refs.next_if(|r| r.0 as usize == at) {
                let l = format!("{}ttype{index}_{n}", abi.local_prefix());
                n += 1;
                match abi.syntax {
                    Syntax::MachO => writeln!(out, "{l}:\n\t.long {name}@GOT-{l}"),
                    Syntax::Elf => writeln!(out, "{l}:\n\t.long {name}-{l}"),
                }
                .ok();
                at = *off as usize + 4;
                continue;
            }
            let end = refs.peek().map_or(table.bytes.len(), |r| r.0 as usize).min(at + 16);
            let list: Vec<String> = table.bytes[at..end].iter().map(|b| b.to_string()).collect();
            let _ = writeln!(out, "\t.byte {}", list.join(", "));
            at = end;
        }
        let _ = writeln!(out, "\t{}", abi.section_directive(SectionKind::Text));
    }
    out.push('\n');
    Ok(())
}

/// For ELF, the words unwind and exception tables reach personality
/// routines and the types caught through (`Abi::eh_ref`): each holds the
/// address of the one it is named after.
pub fn eh_stubs(funcs: &[MFunc], abi: &Abi) -> Vec<GlobalData> {
    let mut out: Vec<GlobalData> = Vec::new();
    if abi.syntax != Syntax::Elf {
        return out;
    }
    for eh in funcs.iter().filter_map(|f| f.eh.as_ref()) {
        for name in std::iter::once(&eh.personality).chain(eh.types.iter().flatten()) {
            let symbol = abi.eh_ref(name);
            if out.iter().any(|g| g.symbol == symbol) {
                continue;
            }
            out.push(GlobalData {
                symbol,
                global: false,
                section: SectionKind::Data,
                section_name: None,
                align: 8,
                items: vec![Item::Addr { symbol: abi.symbol(name), offset: 0, size: 8 }],
            });
        }
    }
    out
}

/// The constants the functions' pool operands name, each once, in
/// read-only data.
pub fn pool_data(funcs: &[MFunc], abi: &Abi) -> Vec<GlobalData> {
    let mut out: Vec<GlobalData> = Vec::new();
    for (name, value, bytes) in funcs.iter().flat_map(|f| &f.pool) {
        let symbol = abi.symbol(name);
        if out.iter().any(|g| g.symbol == symbol) {
            continue;
        }
        out.push(GlobalData {
            symbol,
            global: false,
            section: SectionKind::ReadOnly,
            section_name: None,
            align: u64::from(*bytes),
            items: vec![Item::Int { size: *bytes, value: *value, hex: true }],
        });
    }
    out
}

/// The globals with initial values, in module order, as data.
pub fn collect_globals(cx: &Ctx, abi: &Abi) -> Result<Vec<GlobalData>, String> {
    let m = &*cx.m;
    let mut out = Vec::new();
    for &g in &m.global_order {
        let gl = m.global(g);
        let GlobalKind::Variable(v) = &gl.kind else { continue };
        let Some(init) = v.init else { continue };
        let name = gl.name.clone().unwrap_or_else(|| format!("g{}", g.0));
        // Lists for the compiler (`@llvm.used`), not data.
        if v.section.as_deref() == Some(b"llvm.metadata") {
            continue;
        }
        if name == "llvm.global_ctors" || name == "llvm.global_dtors" {
            out.push(init_array(cx, abi, init, name == "llvm.global_ctors")?);
            continue;
        }
        let align = v.align.or_else(|| cx.dl.abi_align(&m.types, v.ty)).unwrap_or(8).max(1);
        let mut items = Vec::new();
        data(cx, abi, v.ty, init, &mut items)?;
        let zero = items.iter().all(|i| matches!(i, Item::Zero(_)));
        let global = !matches!(gl.props.linkage, Linkage::Private | Linkage::Internal);
        // ELF's: the initial value, in the thread-local section, which
        // the loader copies for each thread. (Zeros too: there is no
        // .tbss yet.)
        if gl.props.thread_local.is_some() && abi.syntax == Syntax::Elf {
            out.push(GlobalData {
                symbol: abi.symbol(&name),
                global,
                section: SectionKind::ThreadData,
                section_name: None,
                align,
                items,
            });
            continue;
        }
        if gl.props.thread_local.is_some() {
            // Mach-O's: the initial value, under a name of its own, and
            // the descriptor the variable's symbol names (the function
            // the loader makes give its address in each thread, a word it
            // keeps, and where the initial value is).
            let symbol = abi.symbol(&name);
            let init = format!("{symbol}$tlv$init");
            out.push(GlobalData {
                symbol: init.clone(),
                global: false,
                section: SectionKind::ThreadData,
                section_name: None,
                align,
                items,
            });
            out.push(GlobalData {
                symbol,
                global,
                section: SectionKind::ThreadVars,
                section_name: None,
                align: 8,
                items: vec![
                    Item::Addr { symbol: abi.symbol("_tlv_bootstrap"), offset: 0, size: 8 },
                    Item::Int { size: 8, value: 0, hex: false },
                    Item::Addr { symbol: init, offset: 0, size: 8 },
                ],
            });
            continue;
        }
        let section_name = v.section.as_ref().map(|s| named_section(&String::from_utf8_lossy(s), abi.syntax));
        let section = match (v.constant, has_addresses(m, init)) {
            // A section of its own holds its zeros.
            (false, _) if section_name.is_some() => SectionKind::Data,
            // Zeros are only counted, where the format has a section for
            // that which assembly can name simply.
            (false, _) if zero && abi.syntax == Syntax::Elf => SectionKind::Bss,
            (false, _) => SectionKind::Data,
            (true, false) => SectionKind::ReadOnly,
            (true, true) => SectionKind::RelRo,
        };
        out.push(GlobalData { symbol: abi.symbol(&name), global, section, section_name, align, items });
    }
    Ok(out)
}

/// A section named by `#[link_section]`, as the object writers and the
/// assembler take it: Mach-O's constructor lists get the type that makes
/// them so (which LLVM infers from the name).
pub fn named_section(name: &str, syntax: Syntax) -> String {
    let parts: Vec<&str> = name.split(',').map(str::trim).collect();
    match (syntax, parts.as_slice()) {
        (Syntax::MachO, [seg, "__mod_init_func"]) => format!("{seg},__mod_init_func,mod_init_funcs"),
        (Syntax::MachO, [seg, "__mod_term_func"]) => format!("{seg},__mod_term_func,mod_term_funcs"),
        // The default type is left out.
        (Syntax::MachO, [seg, sect, "regular"]) => format!("{seg},{sect}"),
        (Syntax::MachO, _) => parts.join(","),
        (Syntax::Elf, _) => name.to_string(),
    }
}

/// `@llvm.global_ctors` (or `_dtors`), `{ priority, function, data }`
/// for each: the functions, by priority, in the list the loader calls.
fn init_array(cx: &Ctx, abi: &Abi, init: ConstId, ctors: bool) -> Result<GlobalData, String> {
    let m = &*cx.m;
    let mut list = Vec::new();
    if let ConstKind::Array(elems) = &m.constant(init).kind {
        for &e in elems {
            let ConstKind::Struct(fields) = &m.constant(e).kind else {
                return Err("@llvm.global_ctors: an entry not a struct".into());
            };
            let priority = fields.first().and_then(|&p| cx.const_int(Value::Const(p))).unwrap_or(65535);
            let Some(&ConstKind::Global(f)) = fields.get(1).map(|&f| &m.constant(f).kind) else {
                return Err("@llvm.global_ctors: an entry not of a function".into());
            };
            list.push((priority, m.global(f).name.clone().unwrap_or_else(|| format!("f{}", f.0))));
        }
    }
    list.sort_by_key(|e| e.0);
    let section_name = match (abi.syntax, ctors) {
        (Syntax::MachO, true) => "__DATA,__mod_init_func,mod_init_funcs",
        (Syntax::MachO, false) => "__DATA,__mod_term_func,mod_term_funcs",
        (Syntax::Elf, true) => ".init_array",
        (Syntax::Elf, false) => ".fini_array",
    };
    Ok(GlobalData {
        symbol: format!("{}{}", abi.local_prefix(), if ctors { "global_ctors" } else { "global_dtors" }),
        global: false,
        section: SectionKind::RelRo,
        section_name: Some(section_name.to_string()),
        align: 8,
        items: list
            .into_iter()
            .map(|(_, f)| Item::Addr { symbol: abi.symbol(&f), offset: 0, size: pointer_bytes(m) })
            .collect(),
    })
}

/// The module's aliases, of a global plus a constant offset.
pub fn collect_aliases(cx: &Ctx, abi: &Abi) -> Result<Vec<object::Alias>, String> {
    let m = &*cx.m;
    let mut out = Vec::new();
    for &g in &m.global_order {
        let gl = m.global(g);
        let GlobalKind::Alias { aliasee, .. } = &gl.kind else { continue };
        let name = gl.name.clone().unwrap_or_else(|| format!("g{}", g.0));
        let (t, offset) = crate::prepare::const_address(cx, *aliasee)
            .ok_or_else(|| format!("@{name}: an alias of something not a global plus an offset"))?;
        let target = m.global(t).name.clone().unwrap_or_else(|| format!("g{}", t.0));
        out.push(object::Alias {
            symbol: abi.symbol(&name),
            target: abi.symbol(&target),
            offset,
            global: !matches!(gl.props.linkage, Linkage::Private | Linkage::Internal),
        });
    }
    Ok(out)
}

pub fn print_aliases(aliases: &[object::Alias], out: &mut String) {
    for a in aliases {
        if a.global {
            let _ = writeln!(out, "\t.globl {}", a.symbol);
        }
        match a.offset {
            0 => {
                let _ = writeln!(out, "\t.set {}, {}", a.symbol, a.target);
            }
            o => {
                let _ = writeln!(out, "\t.set {}, {}{o:+}", a.symbol, a.target);
            }
        }
    }
}

pub fn print_globals(abi: &Abi, globals: &[GlobalData], out: &mut String) {
    for g in globals {
        match &g.section_name {
            Some(name) => {
                let _ = writeln!(out, "\t{}", abi.named_section_directive(name, g.section));
            }
            None => {
                let _ = writeln!(out, "\t{}", abi.section_directive(g.section));
            }
        }
        if g.global {
            let _ = writeln!(out, "\t.globl {}", g.symbol);
        }
        let _ = writeln!(out, "\t.p2align {}", g.align.trailing_zeros());
        if abi.syntax == Syntax::Elf {
            let _ = writeln!(out, "\t.type {},%object", g.symbol);
        }
        let _ = writeln!(out, "{}:", g.symbol);
        for item in &g.items {
            match item {
                Item::Int { size, value, hex } => {
                    let dir = match size {
                        1 => ".byte",
                        2 => ".short",
                        4 => ".long",
                        _ => ".quad",
                    };
                    if *hex {
                        let _ = writeln!(out, "\t{dir} 0x{value:x}");
                    } else {
                        let _ = writeln!(out, "\t{dir} {value}");
                    }
                }
                Item::Bytes(bytes) => {
                    let mut s = String::new();
                    for &b in bytes {
                        match b {
                            b'"' => s.push_str("\\\""),
                            b'\\' => s.push_str("\\\\"),
                            0x20..=0x7e => s.push(b as char),
                            _ => {
                                let _ = write!(s, "\\{:03o}", b);
                            }
                        }
                    }
                    let _ = writeln!(out, "\t.ascii \"{s}\"");
                }
                Item::Zero(n) => {
                    let _ = writeln!(out, "\t.space {n}");
                }
                Item::Addr { symbol, offset, size } => {
                    let dir = if *size == 4 { ".long" } else { ".quad" };
                    if *offset == 0 {
                        let _ = writeln!(out, "\t{dir} {symbol}");
                    } else {
                        let _ = writeln!(out, "\t{dir} {symbol}{offset:+}");
                    }
                }
                Item::Rel32 { symbol, base } => {
                    let _ = writeln!(out, "\t.long {symbol}-{base}");
                }
            }
        }
        if abi.syntax == Syntax::Elf {
            let _ = writeln!(out, "\t.size {}, {}", g.symbol, g.size());
        }
        out.push('\n');
    }
}

/// Does constant `c` contain the address of anything?
fn has_addresses(m: &Module, c: ConstId) -> bool {
    match &m.constant(c).kind {
        ConstKind::Global(_)
        | ConstKind::BlockAddress { .. }
        | ConstKind::DsoLocalEquivalent(_)
        | ConstKind::NoCfi(_) => true,
        ConstKind::Array(e) | ConstKind::Struct(e) | ConstKind::Vector(e) => e.iter().any(|&x| has_addresses(m, x)),
        ConstKind::Splat(x) => has_addresses(m, *x),
        ConstKind::Expr(ConstExpr::Cast { value, .. }) => has_addresses(m, *value),
        ConstKind::Expr(ConstExpr::Gep { ptr, .. }) => has_addresses(m, *ptr),
        ConstKind::Expr(_) => true,
        _ => false,
    }
}

/// The size in bytes of a pointer (in address space 0) in module `m`.
fn pointer_bytes(m: &Module) -> u8 {
    (epitome_ir::layout::pointer_bits(m.data_layout.as_deref(), 0) / 8) as u8
}

fn data(cx: &Ctx, abi: &Abi, ty: TypeId, c: ConstId, out: &mut Vec<Item>) -> Result<(), String> {
    let m = &*cx.m;
    let size = cx.size_of(ty)?;
    let k = m.constant(c);
    match (&k.kind, m.types.get(ty).clone()) {
        (ConstKind::Zero | ConstKind::Null | ConstKind::Undef | ConstKind::Poison, _) => out.push(Item::Zero(size)),
        // Wider than a word: its words, low first (the machines here are
        // little-endian).
        (ConstKind::Int(a), Type::Int(bits)) if size > 8 && size % 8 == 0 => {
            let words = a.words();
            let top = if a.is_negative() { u64::MAX } else { 0 };
            for k in 0..(size / 8) as usize {
                let mut w = words.get(k).copied().unwrap_or(top);
                // The last word's bits past the integer's are its sign's.
                if (k as u32 + 1) * 64 > bits && bits % 64 != 0 && k as u32 * 64 < bits {
                    w = sext(w, bits % 64) as u64;
                }
                out.push(Item::Int { size: 8, value: w, hex: false });
            }
        }
        (ConstKind::Int(a), Type::Int(bits)) => {
            if !matches!(size, 1 | 2 | 4 | 8) {
                return Err(format!("an integer of {size} bytes in data"));
            }
            let v = sext(a.low_u64(), bits) as u64;
            let mask = if size >= 8 { u64::MAX } else { (1u64 << (size * 8)) - 1 };
            out.push(Item::Int { size: size as u8, value: v & mask, hex: false });
        }
        (ConstKind::Float(bits), _) => {
            if !matches!(size, 2 | 4 | 8) {
                return Err(format!("a floating-point value of {size} bytes in data"));
            }
            out.push(Item::Int { size: size as u8, value: *bits as u64, hex: true });
        }
        (ConstKind::Bytes(bytes), _) => out.push(Item::Bytes(bytes.clone())),
        (ConstKind::Array(elems) | ConstKind::Vector(elems), Type::Array { elem, .. } | Type::Vector { elem, .. }) => {
            for &e in elems {
                data(cx, abi, elem, e, out)?;
            }
        }
        (ConstKind::Struct(elems), _) => {
            let (fields, packed) = prepare::struct_fields(cx, ty)?;
            let layout = cx.dl.struct_layout(&m.types, &fields, packed).ok_or("struct layout")?;
            let mut at = 0u64;
            for (n, &e) in elems.iter().enumerate() {
                let o = layout.offsets[n];
                if o > at {
                    out.push(Item::Zero(o - at));
                }
                data(cx, abi, fields[n], e, out)?;
                at = o + cx.size_of(fields[n])?;
            }
            if layout.size > at {
                out.push(Item::Zero(layout.size - at));
            }
        }
        // An integer made a pointer: the integer.
        (ConstKind::Expr(ConstExpr::Cast { op: CastOp::IntToPtr, value }), _)
            if matches!(m.constant(*value).kind, ConstKind::Int(_)) =>
        {
            let ConstKind::Int(a) = &m.constant(*value).kind else { unreachable!() };
            out.push(Item::Int { size: 8, value: a.low_u64(), hex: false });
        }
        // The distance from a global to another place, in 32 bits: a
        // relative lookup table's entry (`base` must be the global being
        // written).
        (ConstKind::Expr(ConstExpr::Cast { op: CastOp::Trunc, value }), _)
            if size == 4
                && let ConstKind::Expr(ConstExpr::Binary { op: BinOp::Sub, lhs, rhs, .. }) =
                    &m.constant(*value).kind
                && let (Some((a, 0)), Some((b, 0))) =
                    (prepare::const_address(cx, *lhs), prepare::const_address(cx, *rhs))
                && a != b =>
        {
            let name = |g: GlobalId| abi.symbol(&m.global(g).name.clone().unwrap_or_default());
            out.push(Item::Rel32 { symbol: name(a), base: name(b) });
        }
        // The distance between two places in one global: a number.
        (ConstKind::Expr(ConstExpr::Binary { op: BinOp::Sub, lhs, rhs, .. }), _)
            if matches!(
                (prepare::const_address(cx, *lhs), prepare::const_address(cx, *rhs)),
                (Some((a, _)), Some((b, _))) if a == b
            ) =>
        {
            let (Some((_, x)), Some((_, y))) = (prepare::const_address(cx, *lhs), prepare::const_address(cx, *rhs))
            else {
                unreachable!()
            };
            let mask = if size >= 8 { u64::MAX } else { (1u64 << (size * 8)) - 1 };
            out.push(Item::Int { size: size as u8, value: (x - y) as u64 & mask, hex: false });
        }
        _ => match prepare::const_address(cx, c) {
            Some((g, offset)) => {
                let name = m.global(g).name.clone().unwrap_or_default();
                out.push(Item::Addr { symbol: abi.symbol(&name), offset, size: pointer_bytes(m) });
            }
            None => return Err(format!("unsupported constant in data: {:?}", k.kind)),
        },
    }
    Ok(())
}
