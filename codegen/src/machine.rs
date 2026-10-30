//! What a description says about its machine as a whole, in named metadata:
//! the names of its registers, its relocations, and a few parameters.
//!
//! ```llvm
//! !target.registers = !{!1, ...}
//! !1 = !{!"x", !"x{0-30}", !"xzr"}        ; class x: x0 ... x30, then xzr
//! !target.classes = !{!4, ...}
//! !4 = !{!"x", !"file", !"gpr", !"bits", !"64", !"holds", !"int"}
//! !target.params = !{!2, ...}
//! !2 = !{!"sp", !"sp"}                     ; the stack pointer, by name
//! !target.relocs = !{!3, ...}
//! !3 = !{!"name", !"call26", !"elf", !"283", ...}
//! ```
//!
//! Each register class lists the names of its registers in number order;
//! `x{0-30}` stands for x0 to x30. `!target.classes` says, for each class:
//!
//! - `file`: the register file its registers are in. Classes of one file
//!   name the same registers (`w3` is `x3`). The code generator numbers
//!   the registers of all files together, each file after the one before
//!   (in the order they are first named), so AArch64's integer registers
//!   are 0-31 and its floating-point and vector registers 32-63;
//! - `bits`: how wide a value it holds;
//! - `holds`: `int`, `float` or `vector`: what values. Of the classes that
//!   hold a kind of value at a width, the first is where such values are
//!   put; a class without `holds` only names registers for templates (a
//!   *view*: `{n:b}` is `b3`, and `lanes` gives `v3` in `{n:4s}`, `v3.4s`);
//! - `sp`: `yes` if a number it shares with the zero register means the
//!   stack pointer in this class (AArch64's `xsp`), so that the zero
//!   register cannot be named in it.
//!
//! The parameters:
//!
//! - `sp`, `zero`: the stack pointer, and the register that reads as zero
//!   (if there is one), by name;
//! - `vector-bits`: the width of a vector register, or 0 if there are none
//!   (vectors are then done lane by lane);
//! - `narrow-results`: how a 32-bit result sits in a 64-bit register:
//!   `zero` (the top half is zeroed; AArch64) or `sign` (sign-extended;
//!   RISC-V), which decides which extensions and truncations are free;
//! - `external-via-got`: `yes` if symbols another module defines are
//!   reached through the global offset table;
//! - `elf-machine`, `elf-flags`, `macho-cpu`: the object files' identity,
//!   and `elf-format`, what objdump calls the format (`elf64-littleriscv`);
//! - `cpu`: the CPU to assume when none is named;
//! - `boolean-branches`: `bit` if a branch can test a boolean's lowest bit
//!   (the rest being undefined), or `compare` if branches compare whole
//!   registers: then a branch on a boolean that is not a comparison's is
//!   made a comparison of it, zero-extended, with zero.
//!
//! `vector-bits` may name a third value, the feature the vector registers
//! come with (`neon`): without it, vectors are done lane by lane.
//!
//! `!target.cpus` lists CPUs, each a name and then the features it has
//! (`!{!"generic", !"fp-armv8", !"neon"}`). Entries of the description with
//! a `"feature"` attribute are used only when their features are on; see
//! `features.rs`.
//!
//! A relocation is a list of properties, as pairs:
//!
//! - `name`: as `"reloc"` attributes name it;
//! - `elf`, `elf-name`: its ELF number and name; `macho`, `macho-name`,
//!   `macho-pcrel`, `macho-length`: Mach-O's type, name, whether it is
//!   PC-relative, and log2 of its size;
//! - `elf-syntax`, `macho-syntax`: how assembly writes an operand it fills
//!   in, with `{s}` for the symbol (`:lo12:{s}`, `{s}@PAGEOFF`);
//! - `use`: `call` (a branch to a symbol, resolved in place when the
//!   symbol is in the same section), `got` (through the global offset
//!   table: kept against the symbol), `tls` (to a thread-local variable:
//!   kept against the symbol, which ELF marks as one), or `data8` (a pointer of 8 bytes in
//!   data; `dataN` for N bytes);
//! - `target`: the address an instruction reaches, from its address `pc`
//!   and the operand's value `v`, for disassembly (`(pc & -4096) + (v << 12)`).
//!
//! Relocations with only a number and a name are for reading other
//! programs' object files.
//!
//! `!target.lacks` lists what the machine does not have, from `LACKABLE`
//! (`docs/design/partial-machines.md`). So far only `stack` changes what a
//! description must say: there is then no `sp`.
//!
//! `!target.enums` names sets of values an operand written as a name takes
//! (`"enum"="chan"` on the operand): each is its name, then pairs of a
//! first value and a pattern of names, which take values from it in order.
//! A pattern's braces are a range, `{0-127}`, or alternatives, `{X,Y,Z,W}`,
//! the leftmost varying slowest:
//!
//! ```llvm
//! !5 = !{!"src", !"0", !"T{0-127}.{X,Y,Z,W}", !"1016", !"PV.{X,Y,Z,W}", !"1020", !"PS"}
//! !6 = !{!"neg", !"0", !"", !"1", !"-"}
//! ```
//!
//! So `T1.Y` is 5 and `PS` 1020. A name may be empty: an operand that is
//! sometimes written as nothing (no negation). Where two names have one
//! value, the first is how it is written.
//!
//! Inline assembly's target-specific parts are two more lists. The codes
//! of its constraints (`"=r,w,I"`; the generic ones, `=`, `&`, `*`,
//! digits, `{reg}`, `~{...}`, `i`, `n`, are the code generator's):
//!
//! ```llvm
//! !target.asm-constraints = !{!5, ...}
//! !5 = !{!"r", !"file", !"r", !"print", !"x"}      ; a register of file r, written as x names it
//! !6 = !{!"I", !"imm", !"0", !"4095"}               ; an immediate in this range
//! !7 = !{!"m", !"mem", !"[{r}]"}                    ; memory at a register, written so
//! !8 = !{!"Z", !"zero"}                             ; the constant zero, as the zero register
//! ```
//!
//! and the template's operand modifiers (`${0:w}`):
//!
//! ```llvm
//! !target.asm-modifiers = !{!9, ...}
//! !9 = !{!"w", !"class", !"w"}                      ; the register as class w names it
//! !10 = !{!"i", !"if-imm", !"i"}                    ; this text if the operand is an immediate
//! ```
//!
//! `asm-comment`, a parameter, is how assembly starts a comment (`//`, `#`).

use std::collections::HashMap;

use epitome_ir::module::{MdId, Metadata, Module};

use crate::abi::Syntax;
use crate::enc::{self, Env, Expr};
use crate::target::Class;

/// How a 32-bit result sits in a 64-bit register.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Narrow {
    /// Its top half is zero: zero extension is free, and so is truncation.
    Zero,
    /// It is sign-extended: sign extension is free; truncation is not.
    Sign,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RelocUse {
    Plain,
    /// A branch to a symbol.
    Call,
    /// Through the global offset table.
    Got,
    /// To a thread-local variable (kept against its symbol, which is
    /// made a thread-local one).
    Tls,
    /// A pointer in data, of this many bytes.
    Data(u8),
    /// In unwind tables (`.eh_frame`): ELF's 32-bit PC-relative address of
    /// a function; Mach-O's pair, a subtractor of a symbol at the table's
    /// start and an unsigned 64-bit address of the function.
    EhPcRel,
    EhSub,
    EhUnsigned,
    /// A 32-bit PC-relative reference to a symbol's GOT entry: unwind and
    /// exception tables' personality routines and types caught.
    GotPcRel32,
    /// The low part of a PC-relative address, whose symbol is the
    /// instruction with the high part (RISC-V's `%pcrel_lo`): kept against
    /// that symbol, as linkers look for it there.
    PcRelLo,
}

#[derive(Clone, Debug)]
pub struct RelocDef {
    pub name: String,
    pub elf: Option<u32>,
    pub elf_name: String,
    pub macho: Option<u8>,
    pub macho_name: String,
    pub macho_pcrel: bool,
    pub macho_length: u8,
    pub elf_syntax: String,
    pub macho_syntax: String,
    pub usage: RelocUse,
    pub target: Option<Expr>,
}

impl RelocDef {
    /// Whether it measures from where it is: its target reads `pc`.
    pub fn pc_relative(&self) -> bool {
        self.target.as_ref().is_some_and(|e| {
            let mut names = Vec::new();
            enc::names_in(e, &mut names);
            names.iter().any(|n| n == "pc")
        })
    }

    /// How assembly in `syntax` writes an operand this fills in.
    pub fn spell(&self, syntax: Syntax, sym: &str) -> String {
        let tpl = match syntax {
            Syntax::Elf => &self.elf_syntax,
            Syntax::MachO => &self.macho_syntax,
        };
        tpl.replace("{s}", sym)
    }

    /// The symbol in an operand written `text`, if this is how it is
    /// written in `syntax`.
    pub fn unspell<'a>(&self, syntax: Syntax, text: &'a str) -> Option<&'a str> {
        // Not one the format has (Mach-O's thread-local variables' in
        // ELF): nothing reads as it.
        let tpl = match syntax {
            Syntax::Elf if self.elf.is_some() => &self.elf_syntax,
            Syntax::MachO if self.macho.is_some() => &self.macho_syntax,
            _ => return None,
        };
        let (pre, post) = tpl.split_once("{s}")?;
        text.strip_prefix(pre)?.strip_suffix(post)
    }

    /// The address an instruction at `pc` reaches, with operand value `v`.
    pub fn reaches(&self, pc: u64, v: i64) -> Option<u64> {
        let e = self.target.as_ref()?;
        let mut env = Env::new();
        env.insert("pc".into(), pc as i64);
        env.insert("v".into(), v);
        enc::eval(e, &env).map(|a| a as u64)
    }
}

/// A target constraint code of inline assembly.
#[derive(Clone, Debug, PartialEq)]
pub enum AsmCode {
    /// A register of this file, written as this class names it.
    Reg { file: usize, print: Option<Class> },
    /// An immediate in this range.
    Imm(i64, i64),
    /// Memory at the address in a register, written by this template
    /// (`{r}` is the register).
    Mem(String),
    /// The constant zero, as the zero register.
    Zero,
}

/// A template modifier of inline assembly.
#[derive(Clone, Debug, PartialEq)]
pub enum AsmModifier {
    /// The register as this class names it.
    Class(Class),
    /// This text if the operand is an immediate, else nothing.
    IfImm(String),
}

/// What values a register class holds.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Holds {
    Int,
    Float,
    Vector,
}

#[derive(Clone, Debug)]
pub struct ClassDef {
    pub name: String,
    pub file: usize,
    /// How wide a value it holds (0 for a view).
    pub bits: u32,
    pub holds: Option<Holds>,
    /// The number it shares with the zero register is the stack pointer.
    pub sp: bool,
    /// Its registers' names, in number order within the file.
    pub names: Vec<String>,
    /// The one register an operand of this class is in (`"fixed"`: x86's
    /// `rax` of a division, `cl` of a shift), by its number in the file.
    /// Selection copies values there and back; no value is kept in it.
    pub fixed: Option<u8>,
}

/// A register file: its registers are numbers `base` to `base + size - 1`.
#[derive(Clone, Debug)]
pub struct FileDef {
    pub name: String,
    pub base: u8,
    pub size: u8,
}

#[derive(Clone, Debug)]
pub struct Machine {
    pub classes: Vec<ClassDef>,
    pub files: Vec<FileDef>,
    pub sp: u8,
    pub zero: Option<u8>,
    pub vector_bits: u32,
    pub narrow: Narrow,
    pub external_via_got: bool,
    pub elf_machine: u16,
    pub elf_flags: u32,
    /// How objdump names the ELF format (`elf64-littleaarch64`).
    pub elf_format: String,
    pub macho_cpu: Option<u32>,
    /// The shortest instruction's length in bytes ("instruction-unit"; 4
    /// unless the description says).
    pub unit: u32,
    /// Encodings are parsed when first used, not as the description is read
    /// ("lazy-encodings"): for a description of tens of thousands.
    pub lazy_encodings: bool,
    pub relocs: Vec<RelocDef>,
    /// The feature the vector registers come with, if they are optional.
    pub vector_feature: Option<String>,
    /// Every width of vector registers, with the feature each needs (x86's
    /// 128 bits, 256 with `avx2`): a function's vectors are the widest its
    /// features allow.
    pub vector_widths: Vec<(u32, Option<String>)>,
    /// The class masks (vectors of `i1`) are held in, by the width of the
    /// vectors in use (`mask-class`), where it is not the smallest that
    /// holds vectors (x86's 256-bit masks are in ymm registers).
    pub mask_classes: Vec<(u32, Class)>,
    /// The narrowest vector kept in a register (`min-vector-bits`; half the
    /// vector registers' width unless the description says: x86's 64-bit
    /// vectors are in xmm registers whatever the width in use).
    pub min_vector_bits: Option<u32>,
    /// CPUs, with the features each has.
    pub cpus: Vec<(String, Vec<String>)>,
    /// The CPU to assume when none is named.
    pub default_cpu: Option<String>,
    /// Branches on a boolean in a register must compare the whole register
    /// (RISC-V), rather than test its lowest bit (AArch64's `tbnz`).
    pub compare_booleans: bool,
    /// The floating-point comparisons no one condition tests, which are
    /// made of two (`"fcmp-split"`: AArch64's one and ueq, the default;
    /// x86's oeq and une).
    pub fcmp_split: Vec<String>,
    /// A sequentially consistent store is a swap whose old value goes
    /// unused (`"seq-cst-stores"="swap"`: x86's `xchg`, as its loads of
    /// any ordering are plain ones).
    pub seq_cst_swap: bool,
    /// Loads and stores need no alignment and cost no more without it
    /// (`unaligned-access` = `fast`), so small copies may use wide ones.
    pub unaligned_fast: bool,
    /// The names of outline atomic helpers, for atomics the machine has no
    /// instruction for (`atomic-helpers`: `__aarch64_{op}{bytes}_{order}`,
    /// `{order}` one of `relax`, `acq`, `rel` and `acq_rel`).
    pub atomic_helpers: Option<String>,
    /// The flags a comparison sets and a condition reads where an entry
    /// does not say (`flags`: AArch64's `nzcv`).
    pub flags: Option<String>,
    pub asm_constraints: HashMap<char, AsmCode>,
    pub asm_modifiers: HashMap<char, AsmModifier>,
    /// How assembly starts a comment.
    pub asm_comment: String,
    /// How inline assembly writes an immediate operand (`asm-imm`, with
    /// `{i}` the number: x86's AT&T `${i}`); modifier `c` writes it bare.
    pub asm_imm: String,
    /// DWARF's number for each file's first register (`!target.dwarf`).
    pub dwarf: HashMap<usize, u32>,
    /// Or for each of its registers, where they are not in order (x86's).
    pub dwarf_each: HashMap<usize, Vec<u32>>,
    /// Unwind tables' CFA at a function's entry, from the stack pointer
    /// (`"cfi-cfa-offset"`: x86's 8, past the return address the call
    /// pushed, which is then at the CFA less 8), and DWARF's number for the
    /// return address (`"cfi-return-column"`: x86's 16, rip), where it is
    /// not the link register.
    pub cfi_cfa_offset: i64,
    pub cfi_return_column: Option<u32>,
    /// No-ops to pad code with, by length from 1 (`!target.nops`, hex):
    /// the longest are used as often as they fit, as the assemblers do.
    /// Without them, code is padded with zeros.
    pub nops: Vec<Vec<u8>>,
    /// The data alignment factor of unwind tables, in ELF and in Mach-O
    /// (`cfi-data-align`: one value, or ELF's then Mach-O's).
    pub cfi_data_align: (i64, i64),
    /// Other ways of writing operands, and how the templates write them
    /// (`!target.asm-aliases`: `[{r}]` is `[{r}, #0]`), for assembly that
    /// other compilers or people write.
    pub asm_aliases: Vec<(String, String)>,
    /// Names of numbers in the operands of instructions whose mnemonic
    /// starts so (`!target.asm-names`: `csr`, `mstatus`, `0x300`).
    pub asm_names: Vec<(String, String, i64)>,
    /// The pseudo-instructions of several instructions assembly may use
    /// (`asm-pseudos`: `riscv`, `asm_text`'s).
    pub asm_pseudos: String,
    /// Registers' other names, as the templates name them (a class's
    /// `names-of`: RISC-V's `x10` is `a0`).
    pub asm_register_names: Vec<(String, String)>,
    /// Longer forms of instructions that do not fit, as `asm_aliases`
    /// write them, their statements separated by `;` (`!target.asm-relax`:
    /// a branch too far, over a jump).
    pub asm_relax: Vec<(String, String)>,
    /// Sets of named values (`!target.enums`).
    pub enums: Vec<EnumDef>,
    /// What the machine does not have (`!target.lacks`; see
    /// `docs/design/partial-machines.md`).
    pub lacks: Vec<String>,
}

/// What `!target.lacks` may list.
pub const LACKABLE: &[&str] = &["memory", "stack", "calls", "globals", "float", "i64", "trap", "indirect"];

/// A set of named values an operand may take (`!target.enums`).
#[derive(Clone, Debug)]
pub struct EnumDef {
    pub name: String,
    /// Each name and its value, in the order the description gives them.
    pub names: Vec<(String, i64)>,
}

impl EnumDef {
    /// The value written `text`.
    pub fn value(&self, text: &str) -> Option<i64> {
        self.names.iter().find(|(n, _)| n == text).map(|&(_, v)| v)
    }

    /// How value `v` is written: the first of its names.
    pub fn name(&self, v: i64) -> Option<&str> {
        self.names.iter().find(|&&(_, x)| x == v).map(|(n, _)| n.as_str())
    }
}

/// The names of an enum's pattern, in order: `T{0-1}.{X,Y}` is T0.X, T0.Y,
/// T1.X, T1.Y.
fn expand_pattern(p: &str) -> Result<Vec<String>, String> {
    let Some((pre, rest)) = p.split_once('{') else { return Ok(vec![p.to_string()]) };
    let (group, post) = rest.split_once('}').ok_or_else(|| format!("bad enum pattern {p}"))?;
    let choices: Vec<String> = match group.split_once('-') {
        Some((a, b)) if !group.contains(',') => {
            let (a, b): (u32, u32) = (
                a.parse().map_err(|_| format!("bad enum pattern {p}"))?,
                b.parse().map_err(|_| format!("bad enum pattern {p}"))?,
            );
            (a..=b).map(|k| k.to_string()).collect()
        }
        _ => group.split(',').map(str::to_string).collect(),
    };
    let tails = expand_pattern(post)?;
    Ok(choices.iter().flat_map(|c| tails.iter().map(move |t| format!("{pre}{c}{t}"))).collect())
}

fn strings(m: &Module, id: MdId) -> Vec<String> {
    match &m.metadata[id.index()].data {
        Metadata::Tuple { ops, .. } => ops
            .iter()
            .filter_map(|o| match o.map(|x| &m.metadata[x.index()].data) {
                Some(Metadata::String(s)) => Some(String::from_utf8_lossy(s).into_owned()),
                _ => None,
            })
            .collect(),
        _ => vec![],
    }
}

/// The tuples of strings in named metadata `name`.
pub fn tuples(m: &Module, name: &str) -> Vec<Vec<String>> {
    m.named_metadata
        .iter()
        .filter(|nm| nm.name == name)
        .flat_map(|nm| nm.ops.iter().map(|&op| strings(m, op)))
        .collect()
}

/// `x{0-30}` is x0 ... x30; other names are themselves.
fn expand(names: &[String]) -> Result<Vec<String>, String> {
    let mut out = Vec::new();
    for n in names {
        match n.split_once('{') {
            Some((pre, rest)) => {
                let (range, post) = rest.split_once('}').ok_or_else(|| format!("bad register names {n}"))?;
                let (a, b) = range.split_once('-').ok_or_else(|| format!("bad register names {n}"))?;
                let (a, b): (u32, u32) = (a.parse().map_err(|_| n.clone())?, b.parse().map_err(|_| n.clone())?);
                out.extend((a..=b).map(|k| format!("{pre}{k}{post}")));
            }
            None => out.push(n.clone()),
        }
    }
    Ok(out)
}

impl Machine {
    /// Whether the machine lacks `what` (`!target.lacks`): `memory` implies
    /// `stack` and `globals`.
    pub fn lacks(&self, what: &str) -> bool {
        self.lacks.iter().any(|l| l == what || (l == "memory" && (what == "stack" || what == "globals")))
    }

    /// The enum named `name`, by its index.
    pub fn enum_index(&self, name: &str) -> Option<usize> {
        self.enums.iter().position(|e| e.name == name)
    }

    pub fn read(m: &Module) -> Result<Machine, String> {
        let props: HashMap<String, HashMap<String, String>> = tuples(m, "target.classes")
            .into_iter()
            .filter_map(|t| {
                let (name, rest) = t.split_first()?;
                Some((name.clone(), rest.chunks(2).filter_map(|c| Some((c[0].clone(), c.get(1)?.clone()))).collect()))
            })
            .collect();
        let mut classes: Vec<ClassDef> = Vec::new();
        let mut files: Vec<FileDef> = Vec::new();
        for t in tuples(m, "target.registers") {
            let Some((class, list)) = t.split_first() else { continue };
            let names = expand(list)?;
            let p = props.get(class).ok_or_else(|| format!("register class {class} is not in !target.classes"))?;
            let fname = p.get("file").ok_or_else(|| format!("register class {class}: no file"))?;
            let file = match files.iter().position(|f| f.name == *fname) {
                Some(k) => k,
                None => {
                    files.push(FileDef { name: fname.clone(), base: 0, size: 0 });
                    files.len() - 1
                }
            };
            files[file].size = files[file].size.max(names.len().min(255) as u8);
            let bits = match p.get("bits") {
                Some(b) => b.parse().map_err(|_| format!("register class {class}: bits {b}"))?,
                None => 0,
            };
            let holds = match p.get("holds").map(String::as_str) {
                None => None,
                Some("int") => Some(Holds::Int),
                Some("float") => Some(Holds::Float),
                Some("vector") => Some(Holds::Vector),
                Some(o) => return Err(format!("register class {class}: holds {o}")),
            };
            if holds.is_some() && bits == 0 {
                return Err(format!("register class {class} holds values, but has no bits"));
            }
            let sp = p.get("sp").is_some_and(|v| v == "yes");
            let fixed = match p.get("fixed") {
                Some(r) => Some(
                    names.iter().position(|n| n == r).ok_or_else(|| format!("register class {class}: fixed {r}"))?
                        as u8,
                ),
                None => None,
            };
            if fixed.is_some() && holds.is_some() {
                return Err(format!("register class {class} is fixed, so holds no values"));
            }
            classes.push(ClassDef { name: class.clone(), file, bits, holds, sp, names, fixed });
        }
        if classes.is_empty() {
            return Err("the description names no registers (!target.registers)".into());
        }
        if let Some(c) = props.keys().find(|c| !classes.iter().any(|d| d.name == **c)) {
            return Err(format!("register class {c} is in !target.classes, but not in !target.registers"));
        }
        let mut base = 0u32;
        for f in &mut files {
            f.base = base as u8;
            base += f.size as u32;
        }
        if base > 255 {
            return Err(format!("the register files have {base} registers in all; at most 255 are allowed"));
        }
        let all_params = tuples(m, "target.params");
        let params: HashMap<String, String> = all_params
            .iter()
            .filter_map(|t| Some((t.first()?.clone(), t.get(1).cloned().unwrap_or_default())))
            .collect();
        let mut mc = Machine {
            classes,
            files,
            sp: 0,
            zero: None,
            vector_bits: 0,
            narrow: Narrow::Zero,
            external_via_got: false,
            elf_machine: 0,
            elf_flags: 0,
            elf_format: String::new(),
            macho_cpu: None,
            unit: 4,
            lazy_encodings: false,
            relocs: Vec::new(),
            vector_feature: None,
            vector_widths: Vec::new(),
            mask_classes: Vec::new(),
            min_vector_bits: None,
            cpus: Vec::new(),
            default_cpu: None,
            compare_booleans: false,
            unaligned_fast: false,
            atomic_helpers: None,
            flags: None,
            asm_constraints: HashMap::new(),
            asm_modifiers: HashMap::new(),
            asm_comment: String::new(),
            asm_imm: "{i}".into(),
            dwarf: HashMap::new(),
            dwarf_each: HashMap::new(),
            cfi_cfa_offset: 0,
            cfi_return_column: None,
            nops: Vec::new(),
            fcmp_split: vec!["one".into(), "ueq".into()],
            seq_cst_swap: false,
            cfi_data_align: (-8, -8),
            asm_aliases: Vec::new(),
            asm_names: Vec::new(),
            asm_pseudos: String::new(),
            asm_register_names: Vec::new(),
            asm_relax: Vec::new(),
            enums: Vec::new(),
            lacks: Vec::new(),
        };
        let num = |k: &str| -> Result<Option<u64>, String> {
            params
                .get(k)
                .map(|v| {
                    let v = v.trim();
                    match v.strip_prefix("0x") {
                        Some(h) => u64::from_str_radix(h, 16),
                        None => v.parse(),
                    }
                    .map_err(|_| format!("target parameter {k}: {v}"))
                })
                .transpose()
        };
        for t in tuples(m, "target.lacks") {
            for what in t {
                if !LACKABLE.contains(&what.as_str()) {
                    return Err(format!("!target.lacks: {what} is not one of {}", LACKABLE.join(", ")));
                }
                mc.lacks.push(what);
            }
        }
        // (A machine without a stack has no stack pointer; nothing that
        // reads `sp` runs for it.)
        mc.sp = match params.get("sp") {
            Some(sp) => mc.reg(sp)?,
            None if mc.lacks("stack") => u8::MAX,
            None => return Err("no stack pointer (target parameter sp)".into()),
        };
        mc.zero = params.get("zero").map(|z| mc.reg(z)).transpose()?;
        mc.vector_bits = num("vector-bits")?.unwrap_or(0) as u32;
        mc.vector_feature = all_params.iter().find(|t| t[0] == "vector-bits").and_then(|t| t.get(2).cloned());
        for t in all_params.iter().filter(|t| t[0] == "vector-bits") {
            let bits = t.get(1).and_then(|b| b.parse().ok()).ok_or("target parameter vector-bits: a width")?;
            mc.vector_widths.push((bits, t.get(2).cloned()));
        }
        mc.min_vector_bits = num("min-vector-bits")?.map(|b| b as u32);
        for t in all_params.iter().filter(|t| t[0] == "mask-class") {
            let bits = t.get(1).and_then(|b| b.parse().ok()).ok_or("target parameter mask-class: a width")?;
            let class = t.get(2).and_then(|c| mc.class(c)).ok_or("target parameter mask-class: a class")?;
            mc.mask_classes.push((bits, class));
        }
        mc.default_cpu = params.get("cpu").cloned();
        mc.compare_booleans = match params.get("boolean-branches").map(String::as_str) {
            None | Some("bit") => false,
            Some("compare") => true,
            Some(o) => return Err(format!("target parameter boolean-branches: {o}")),
        };
        mc.flags = params.get("flags").cloned();
        mc.atomic_helpers = params.get("atomic-helpers").cloned();
        mc.unaligned_fast = match params.get("unaligned-access").map(String::as_str) {
            None | Some("no") => false,
            Some("fast") => true,
            Some(o) => return Err(format!("target parameter unaligned-access: {o}")),
        };
        for t in tuples(m, "target.cpus") {
            if let Some((name, feats)) = t.split_first() {
                mc.cpus.push((name.clone(), feats.to_vec()));
            }
        }
        mc.narrow = match params.get("narrow-results").map(String::as_str) {
            None | Some("zero") => Narrow::Zero,
            Some("sign") => Narrow::Sign,
            Some(o) => return Err(format!("target parameter narrow-results: {o}")),
        };
        mc.seq_cst_swap = params.get("seq-cst-stores").is_some_and(|v| v == "swap");
        mc.external_via_got = params.get("external-via-got").is_some_and(|v| v == "yes");
        if let Some(t) = all_params.iter().find(|t| t[0] == "fcmp-split") {
            mc.fcmp_split = t[1..].to_vec();
        }
        mc.elf_machine = num("elf-machine")?.unwrap_or(0) as u16;
        mc.elf_flags = num("elf-flags")?.unwrap_or(0) as u32;
        mc.elf_format = params.get("elf-format").cloned().unwrap_or_else(|| "elf64-little".into());
        mc.asm_comment = params.get("asm-comment").cloned().unwrap_or_else(|| "//".into());
        mc.asm_imm = params.get("asm-imm").cloned().unwrap_or_else(|| "{i}".into());
        if let Some(t) = all_params.iter().find(|t| t[0] == "cfi-data-align") {
            let v = |k: usize| t.get(k).and_then(|x| x.parse::<i64>().ok());
            let elf = v(1).ok_or("target parameter cfi-data-align: a number")?;
            mc.cfi_data_align = (elf, v(2).unwrap_or(elf));
        }
        let one_char = |k: &str| -> Result<char, String> {
            let mut c = k.chars();
            match (c.next(), c.next()) {
                (Some(x), None) => Ok(x),
                _ => Err(format!("inline assembly: {k:?} is not one letter")),
            }
        };
        for t in tuples(m, "target.asm-constraints") {
            let Some(code) = t.first() else { continue };
            let class = |n: &str| mc.class(n).ok_or_else(|| format!("asm constraint {code}: no register class {n}"));
            let int = |k: usize| -> Result<i64, String> {
                t.get(k).and_then(|v| v.parse().ok()).ok_or_else(|| format!("asm constraint {code}: bad number"))
            };
            let def = match t.get(1).map(String::as_str) {
                Some("file") => {
                    let fname = t.get(2).ok_or_else(|| format!("asm constraint {code}: no file"))?;
                    let file = mc
                        .files
                        .iter()
                        .position(|f| f.name == *fname)
                        .ok_or_else(|| format!("asm constraint {code}: no register file {fname}"))?;
                    let print = match (t.get(3).map(String::as_str), t.get(4)) {
                        (Some("print"), Some(c)) => Some(class(c)?),
                        _ => None,
                    };
                    AsmCode::Reg { file, print }
                }
                Some("imm") => AsmCode::Imm(int(2)?, int(3)?),
                Some("mem") => AsmCode::Mem(t.get(2).cloned().unwrap_or_else(|| "{r}".into())),
                Some("zero") => AsmCode::Zero,
                o => return Err(format!("asm constraint {code}: {o:?}")),
            };
            mc.asm_constraints.insert(one_char(code)?, def);
        }
        for t in tuples(m, "target.dwarf") {
            let Some((file, nums)) = t.split_first() else { continue };
            let k = mc
                .files
                .iter()
                .position(|f| f.name == *file)
                .ok_or_else(|| format!("!target.dwarf: no file {file}"))?;
            let nums: Vec<u32> =
                nums.iter().map(|n| n.parse().map_err(|_| format!("!target.dwarf: {n}"))).collect::<Result<_, _>>()?;
            match nums[..] {
                [base] => {
                    mc.dwarf.insert(k, base);
                }
                _ => {
                    mc.dwarf_each.insert(k, nums);
                }
            }
        }
        for t in tuples(m, "target.nops") {
            for hex in t {
                let bytes = (0..hex.len() / 2)
                    .map(|k| u8::from_str_radix(&hex[2 * k..2 * k + 2], 16))
                    .collect::<Result<Vec<u8>, _>>()
                    .map_err(|_| format!("!target.nops: {hex}"))?;
                mc.nops.push(bytes);
            }
        }
        mc.cfi_cfa_offset = num("cfi-cfa-offset")?.unwrap_or(0) as i64;
        mc.cfi_return_column = num("cfi-return-column")?.map(|v| v as u32);
        for t in tuples(m, "target.asm-aliases") {
            if let [from, to] = &t[..] {
                mc.asm_aliases.push((from.clone(), to.clone()));
            }
        }
        for t in tuples(m, "target.asm-relax") {
            if let [from, to] = &t[..] {
                mc.asm_relax.push((from.clone(), to.clone()));
            }
        }
        for t in tuples(m, "target.asm-names") {
            if let [prefix, name, value] = &t[..] {
                let v = crate::asm_text::eval(value).ok_or_else(|| format!("!target.asm-names: {name} = {value}"))?;
                mc.asm_names.push((prefix.clone(), name.clone(), v));
            }
        }
        mc.asm_pseudos = params.get("asm-pseudos").cloned().unwrap_or_default();
        for t in tuples(m, "target.classes") {
            let Some(k) = t.iter().position(|x| x == "names-of") else { continue };
            let (Some(from), Some(to)) = (mc.class(&t[0]), t.get(k + 1).and_then(|c| mc.class(c))) else {
                return Err(format!("register class {}: names-of", t[0]));
            };
            let pairs = mc.def(from).names.iter().zip(&mc.def(to).names);
            let pairs: Vec<(String, String)> = pairs.map(|(a, b)| (a.clone(), b.clone())).collect();
            mc.asm_register_names.extend(pairs);
        }
        for t in tuples(m, "target.enums") {
            let Some((name, pairs)) = t.split_first() else { continue };
            let mut names = Vec::new();
            for pair in pairs.chunks(2) {
                let [first, pattern] = pair else { return Err(format!("!target.enums {name}: a value without names")) };
                let first: i64 = first.parse().map_err(|_| format!("!target.enums {name}: {first}"))?;
                names.extend(expand_pattern(pattern)?.into_iter().zip(first..));
            }
            mc.enums.push(EnumDef { name: name.clone(), names });
        }
        for t in tuples(m, "target.asm-modifiers") {
            let Some(code) = t.first() else { continue };
            let def = match (t.get(1).map(String::as_str), t.get(2)) {
                (Some("class"), Some(c)) => AsmModifier::Class(
                    mc.class(c).ok_or_else(|| format!("asm modifier {code}: no register class {c}"))?,
                ),
                (Some("if-imm"), Some(text)) => AsmModifier::IfImm(text.clone()),
                o => return Err(format!("asm modifier {code}: {o:?}")),
            };
            mc.asm_modifiers.insert(one_char(code)?, def);
        }
        mc.macho_cpu = num("macho-cpu")?.map(|v| v as u32);
        mc.unit = num("instruction-unit")?.unwrap_or(4) as u32;
        mc.lazy_encodings = num("lazy-encodings")?.unwrap_or(0) != 0;
        for t in tuples(m, "target.relocs") {
            let p: HashMap<&str, &str> =
                t.chunks(2).filter_map(|c| Some((c[0].as_str(), c.get(1)?.as_str()))).collect();
            let name = p.get("name").ok_or("a relocation without a name")?.to_string();
            let parse = |k: &str| -> Result<Option<u32>, String> {
                p.get(k).map(|v| v.parse::<u32>().map_err(|_| format!("relocation {name}: {k} {v}"))).transpose()
            };
            mc.relocs.push(RelocDef {
                elf: parse("elf")?,
                elf_name: p.get("elf-name").unwrap_or(&"").to_string(),
                macho: parse("macho")?.map(|v| v as u8),
                macho_name: p.get("macho-name").unwrap_or(&"").to_string(),
                macho_pcrel: p.get("macho-pcrel") == Some(&"yes"),
                macho_length: parse("macho-length")?.unwrap_or(2) as u8,
                elf_syntax: p.get("elf-syntax").unwrap_or(&"{s}").to_string(),
                macho_syntax: p.get("macho-syntax").unwrap_or(&"{s}").to_string(),
                usage: match p.get("use").copied() {
                    None | Some("") => RelocUse::Plain,
                    Some("call") => RelocUse::Call,
                    Some("eh-pcrel") => RelocUse::EhPcRel,
                    Some("eh-sub") => RelocUse::EhSub,
                    Some("eh-unsigned") => RelocUse::EhUnsigned,
                    Some("got-pcrel32") => RelocUse::GotPcRel32,
                    Some("got") => RelocUse::Got,
                    Some("tls") => RelocUse::Tls,
                    Some("pcrel-lo") => RelocUse::PcRelLo,
                    Some(d) if d.starts_with("data") => RelocUse::Data(
                        d.trim_start_matches("data").parse().map_err(|_| format!("relocation {name}: use {d}"))?,
                    ),
                    Some(o) => return Err(format!("relocation {name}: use {o}")),
                },
                target: p
                    .get("target")
                    .map(|e| enc::parse_expr(e).map_err(|err| format!("relocation {name}: {err}")))
                    .transpose()?,
                name,
            });
        }
        Ok(mc)
    }

    /// `n` bytes of padding for code: no-ops, if the description gives them.
    pub fn code_fill(&self, mut n: usize) -> Vec<u8> {
        if self.nops.is_empty() {
            return vec![0; n];
        }
        let mut out = Vec::with_capacity(n);
        while n > 0 {
            let k = n.min(self.nops.len());
            out.extend_from_slice(&self.nops[k - 1]);
            n -= k;
        }
        out
    }

    /// What a class holds: for a fixed class (one register), what its
    /// file's classes do.
    pub fn holds_of(&self, c: Class) -> Option<Holds> {
        let d = self.def(c);
        match (d.holds, d.fixed) {
            (None, Some(_)) => {
                self.classes.iter().find(|x| x.file == d.file && x.holds.is_some()).and_then(|x| x.holds)
            }
            (h, _) => h,
        }
    }

    /// The class called `name`.
    pub fn class(&self, name: &str) -> Option<Class> {
        self.classes.iter().position(|c| c.name == name).map(|k| Class(k as u8))
    }

    pub fn def(&self, c: Class) -> &ClassDef {
        &self.classes[c.0 as usize]
    }

    /// The name of register `r` in class `class`.
    pub fn name(&self, class: Class, r: u8) -> String {
        self.field(class, r)
            .and_then(|k| self.def(class).names.get(k as usize))
            .cloned()
            .unwrap_or_else(|| format!("r{r}?"))
    }

    /// Register `r`'s number within its file, if it is in `class`'s.
    pub fn field(&self, class: Class, r: u8) -> Option<i64> {
        let f = &self.files[self.def(class).file];
        (r >= f.base && r < f.base.saturating_add(f.size)).then(|| (r - f.base) as i64)
    }

    /// The register numbered `k` within `class`'s file.
    pub fn phys(&self, class: Class, k: i64) -> u8 {
        let f = &self.files[self.def(class).file];
        (f.base as i64 + k.clamp(0, f.size as i64 - 1)) as u8
    }

    /// Register `r`'s number within its file, as encodings hold it.
    pub fn index(&self, r: u8) -> i64 {
        (r - self.files[self.file_of(r)].base) as i64
    }

    /// The data alignment factor of unwind tables in this dialect's object
    /// files.
    pub fn data_align(&self, syntax: Syntax) -> i64 {
        match syntax {
            Syntax::Elf => self.cfi_data_align.0,
            Syntax::MachO => self.cfi_data_align.1,
        }
    }

    /// DWARF's number for register `r`, for unwind tables.
    pub fn dwarf_number(&self, r: u8) -> Option<u32> {
        let f = self.file_of(r);
        if let Some(each) = self.dwarf_each.get(&f) {
            return each.get(self.index(r) as usize).copied();
        }
        self.dwarf.get(&f).map(|b| b + self.index(r) as u32)
    }

    /// The relocation used so.
    pub fn reloc_used(&self, u: RelocUse) -> Option<usize> {
        self.relocs.iter().position(|r| r.usage == u)
    }

    /// Whether register `r` is one of class `c`'s: in its file, and among
    /// the registers it names (x86's 128- and 256-bit classes have 16 of
    /// the vector file's 32).
    pub fn in_class(&self, c: Class, r: u8) -> bool {
        let d = self.def(c);
        self.file_of(r) == d.file && (self.index(r) as usize) < d.names.len()
    }

    /// The file register `r` is in.
    pub fn file_of(&self, r: u8) -> usize {
        self.files.iter().position(|f| r >= f.base && r < f.base.saturating_add(f.size)).unwrap_or(0)
    }

    /// The class values of this kind and width are put in: the narrowest
    /// that holds them, the first declared of equals.
    pub fn value_class(&self, holds: Holds, bits: u32) -> Option<Class> {
        let mut best: Option<usize> = None;
        for (k, c) in self.classes.iter().enumerate() {
            if c.holds == Some(holds) && !c.sp && c.bits >= bits && best.is_none_or(|b| c.bits < self.classes[b].bits) {
                best = Some(k);
            }
        }
        best.map(|k| Class(k as u8))
    }

    /// The class of file `file` a value of this kind and width is put in,
    /// for inline assembly's register constraints: one that holds the kind,
    /// else any that is wide enough (an integer in a floating-point
    /// register is held as a float of its width).
    pub fn value_class_in(&self, file: usize, holds: Holds, bits: u32) -> Option<Class> {
        let fits = |c: &ClassDef| c.file == file && !c.sp && c.holds.is_some() && c.bits >= bits;
        let pick = |want: &dyn Fn(&ClassDef) -> bool| {
            self.classes
                .iter()
                .enumerate()
                .filter(|(_, c)| fits(c) && want(c))
                .min_by_key(|(k, c)| (c.bits, *k))
                .map(|(k, _)| Class(k as u8))
        };
        pick(&|c| c.holds == Some(holds)).or_else(|| pick(&|_| true))
    }

    /// The class of file `file` holding scalars of `bits` bits, for the
    /// other side of a copy between files.
    pub fn scalar_in(&self, file: usize, bits: u32) -> Option<Class> {
        self.classes
            .iter()
            .position(|c| {
                c.file == file && c.bits == bits && matches!(c.holds, Some(Holds::Int | Holds::Float)) && !c.sp
            })
            .map(|k| Class(k as u8))
    }

    /// Can the zero register be named in this class?
    pub fn zero_in(&self, class: Class) -> bool {
        let c = self.def(class);
        self.zero.is_some_and(|z| self.file_of(z) == c.file && !(c.sp && z == self.sp))
    }

    /// The class a register of file `file` is saved and restored as, by a
    /// function that must preserve it: its widest scalar class (AArch64's
    /// d, of q, d and s: only the low 64 bits of v8-v15 are preserved).
    pub fn save_class(&self, file: usize) -> Option<Class> {
        let mut best: Option<usize> = None;
        for (k, c) in self.classes.iter().enumerate() {
            if c.file == file
                && !c.sp
                && matches!(c.holds, Some(Holds::Int | Holds::Float))
                && best.is_none_or(|b| c.bits > self.classes[b].bits)
            {
                best = Some(k);
            }
        }
        best.map(|k| Class(k as u8))
    }

    /// Register `r`'s name, as its file's save class names it (`x3`, `d8`).
    pub fn lookup_name(&self, r: u8) -> String {
        match self.save_class(self.file_of(r)) {
            Some(c) => self.name(c, r),
            None => format!("r{r}"),
        }
    }

    /// The name of the class register `r` is saved as (`x`, `d`).
    pub fn save_name(&self, r: u8) -> &str {
        self.save_class(self.file_of(r)).map_or("?", |c| self.def(c).name.as_str())
    }

    /// The register a name names, in any class or view.
    pub fn reg(&self, name: &str) -> Result<u8, String> {
        self.lookup(name).ok_or_else(|| format!("no register {name}"))
    }

    pub fn lookup(&self, name: &str) -> Option<u8> {
        self.classes.iter().find_map(|c| {
            let k = c.names.iter().position(|n| n == name)?;
            Some(self.files[c.file].base + k as u8)
        })
    }

    /// Registers in a list such as `["x0-x7", "x30"]`.
    pub fn regs(&self, list: &[String]) -> Result<Vec<u8>, String> {
        let mut out = Vec::new();
        for n in list {
            match n.split_once('-').filter(|(a, b)| self.lookup(a).is_some() && self.lookup(b).is_some()) {
                Some((a, b)) => out.extend(self.reg(a)?..=self.reg(b)?),
                None => out.push(self.reg(n)?),
            }
        }
        Ok(out)
    }

    pub fn reloc(&self, name: &str) -> Option<usize> {
        self.relocs.iter().position(|r| r.name == name)
    }

    /// The relocation for a pointer of `bytes` bytes in data.
    pub fn data_reloc(&self, bytes: u8) -> Option<usize> {
        self.relocs.iter().position(|r| r.usage == RelocUse::Data(bytes))
    }

    pub fn by_elf(&self, ty: u32) -> Option<usize> {
        self.relocs.iter().position(|r| r.elf == Some(ty))
    }

    pub fn by_macho(&self, ty: u8, length: u8) -> Option<usize> {
        self.relocs
            .iter()
            .position(|r| r.macho == Some(ty) && (r.usage != RelocUse::Data(8) || length == 3))
            .or_else(|| self.relocs.iter().position(|r| r.macho == Some(ty)))
    }

    pub fn elf_name(&self, ty: u32) -> String {
        match self.by_elf(ty) {
            Some(k) if !self.relocs[k].elf_name.is_empty() => self.relocs[k].elf_name.clone(),
            _ => format!("R_{ty}"),
        }
    }

    pub fn macho_name(&self, ty: u8) -> String {
        match self.relocs.iter().find(|r| r.macho == Some(ty) && !r.macho_name.is_empty()) {
            Some(r) => r.macho_name.clone(),
            None => format!("RELOC_{ty}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Files are numbered one after another, however many registers each
    /// has, and classes are found by what they hold.
    #[test]
    fn files_and_classes() {
        let src = r#"
!target.registers = !{!0, !1, !2, !3, !4}
!0 = !{!"r", !"r{0-15}"}
!1 = !{!"rsp", !"r{0-14}", !"sp"}
!2 = !{!"f", !"f{0-7}"}
!3 = !{!"v", !"v{0-31}"}
!4 = !{!"p", !"p{0-7}"}
!target.classes = !{!10, !11, !12, !13, !14}
!10 = !{!"r", !"file", !"gpr", !"bits", !"64", !"holds", !"int"}
!11 = !{!"rsp", !"file", !"gpr", !"bits", !"64", !"holds", !"int", !"sp", !"yes"}
!12 = !{!"f", !"file", !"fpr", !"bits", !"64", !"holds", !"float"}
!13 = !{!"v", !"file", !"vec", !"bits", !"256", !"holds", !"vector"}
!14 = !{!"p", !"file", !"pred"}
!target.params = !{!20, !21}
!20 = !{!"sp", !"sp"}
!21 = !{!"zero", !"r15"}
"#;
        let m = Machine::read(&epitome_ir::parse_module(src).unwrap()).unwrap();
        let bases: Vec<(&str, u8, u8)> = m.files.iter().map(|f| (f.name.as_str(), f.base, f.size)).collect();
        assert_eq!(bases, [("gpr", 0, 16), ("fpr", 16, 8), ("vec", 24, 32), ("pred", 56, 8)]);
        assert_eq!(m.lookup("f3"), Some(19));
        assert_eq!(m.lookup("v0"), Some(24));
        assert_eq!(m.lookup("p7"), Some(63));
        let (r, rsp, v) = (m.class("r").unwrap(), m.class("rsp").unwrap(), m.class("v").unwrap());
        assert_eq!(m.name(r, 15), "r15");
        assert_eq!(m.name(rsp, 15), "sp");
        assert_eq!(m.index(m.lookup("v5").unwrap()), 5);
        assert_eq!(m.value_class(Holds::Int, 32), Some(r));
        assert_eq!(m.value_class(Holds::Vector, 128), Some(v));
        assert_eq!(m.value_class(Holds::Float, 128), None);
        // The zero register shares its number with sp: not in rsp.
        assert!(m.zero_in(r) && !m.zero_in(rsp));
        assert_eq!(m.save_class(1), m.class("f"));
        assert_eq!(m.save_class(3), None);
    }

    /// Enums' names take values in order, the leftmost brace varying
    /// slowest; a machine that lacks a stack needs no stack pointer, and
    /// one that lacks memory lacks a stack too.
    #[test]
    fn enums_and_lacks() {
        let src = r#"
!target.registers = !{!0}
!0 = !{!"t", !"T{0-3}"}
!target.classes = !{!1}
!1 = !{!"t", !"file", !"t"}
!target.lacks = !{!2}
!2 = !{!"memory", !"calls"}
!target.enums = !{!3, !4}
!3 = !{!"src", !"0", !"T{0-1}.{X,Y}", !"10", !"PS", !"0", !"T0"}
!4 = !{!"neg", !"0", !"", !"1", !"-"}
"#;
        let m = Machine::read(&epitome_ir::parse_module(src).unwrap()).unwrap();
        assert!(m.lacks("stack") && m.lacks("globals") && m.lacks("calls") && !m.lacks("float"));
        let src_enum = &m.enums[m.enum_index("src").unwrap()];
        let names: Vec<(&str, i64)> = src_enum.names.iter().map(|(n, v)| (n.as_str(), *v)).collect();
        assert_eq!(names, [("T0.X", 0), ("T0.Y", 1), ("T1.X", 2), ("T1.Y", 3), ("PS", 10), ("T0", 0)]);
        // The first name of a value is how it is written; any is read.
        assert_eq!(src_enum.name(0), Some("T0.X"));
        assert_eq!(src_enum.value("T0"), Some(0));
        assert_eq!(m.enums[m.enum_index("neg").unwrap()].value(""), Some(0));
        let bad = src.replace(r#"!"calls""#, r#"!"cats""#);
        assert!(Machine::read(&epitome_ir::parse_module(&bad).unwrap()).is_err());
    }
}
