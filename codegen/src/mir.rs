//! Machine IR: blocks of machine instructions, each an instruction from the
//! description (or a copy) with its operands. Printing, encoding and
//! disassembly all go through the description: the `"asm"` template and
//! the `"enc"` fields.

use std::fmt::Write;

use crate::enc::{self, Env};
use crate::target::{Class, Kind, Target};

pub type VReg = u32;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum Reg {
    V(VReg),
    /// A physical register, numbered as `machine.rs` numbers them: the
    /// files one after another.
    P(u8),
}

#[derive(Clone, PartialEq, Debug)]
pub enum MOp {
    Reg(Reg),
    Imm(i64),
    /// A block of the same function.
    Label(usize),
    Sym(String),
    /// An offset in the frame, known once the frame is laid out: the slot
    /// and a byte offset in it.
    Frame(usize, i64),
    /// An offset from the stack pointer on entry: an argument the caller
    /// put on the stack. Known once the frame's size is.
    Incoming(i64),
    /// The size of the area at the bottom of the frame for calls'
    /// arguments: a dynamic alloca's memory is above it.
    OutArea,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Opcode {
    Instr(usize),
    /// `ops[0] = ops[1]`, both registers of the width of `ops[0]`.
    Copy,
    /// Inline assembly: block `id` of the function's `asms`. Its first
    /// `outs` operands are written, the rest read.
    Asm {
        id: u32,
        outs: u16,
    },
    /// A rule of the unwind tables, for the instructions after it: no code.
    Cfi(crate::unwind::Cfi),
}

/// A function's exception handling. A call that can unwind to a landing
/// pad (an `invoke`) has two more operands than its instruction: the
/// landing pad's block, and the index of its action in `actions`.
#[derive(Clone, Debug, Default)]
pub struct EhInfo {
    /// The personality routine's symbol, as the IR names it.
    pub personality: String,
    /// The types caught, by their type information's symbols; `None`
    /// catches all. A landing pad's selector is a type's index here, from 1.
    pub types: Vec<Option<String>>,
    /// Each landing pad's action: whether it cleans up, and its clauses in
    /// order: a type caught (its index into `types`, from 1), or a filter
    /// (the types that may pass, -1 less its list's offset in `specs`).
    pub actions: Vec<(bool, Vec<i64>)>,
    /// The filters' lists of types: each type's index as a ULEB128, then 0.
    pub specs: Vec<u8>,
}

/// How an inline assembly operand is written into its template.
#[derive(Clone, Debug, PartialEq)]
pub enum AsmPrint {
    /// A register, as this class names it (by default).
    Reg(Class),
    /// An immediate.
    Imm,
    /// Memory at the address in a register: the template, with `{r}` for
    /// the register as the class names it.
    Mem(String, Class),
}

/// One inline assembly statement block: its template (LLVM's, with `$0`
/// and `${1:w}`), and for each `$N` the operand of the instruction it is
/// and how it is written.
#[derive(Clone, Debug)]
pub struct AsmBlock {
    pub template: String,
    pub operands: Vec<(usize, AsmPrint)>,
    /// For `${:uid}`: a number unique to this block.
    pub uid: u64,
    /// How many instructions it is, as far as can be told: for branch
    /// distances.
    pub statements: u32,
    /// The outputs written before every input is read (`=&r`), which must
    /// not share a register with one.
    pub early: Vec<bool>,
}

#[derive(Clone, Debug)]
pub struct MInst {
    pub op: Opcode,
    pub ops: Vec<MOp>,
    pub imp_uses: Vec<u8>,
    pub imp_defs: Vec<u8>,
    /// For a copy: the class of what is copied (recorded before register
    /// allocation, while the virtual registers say).
    pub class: Option<Class>,
    /// A call that does not return: the block ends with it, with no trap
    /// after (as a return or a jump would end it).
    pub noreturn: bool,
    /// Made from an atomic operation (any ordering) or a fence: a memory
    /// barrier. No pass moves, merges or removes it, or moves another
    /// access across it.
    pub barrier: bool,
    /// Its encoding's length in bytes, once known (see `set_sizes`); 0
    /// until then.
    pub size: u8,
}

impl MInst {
    /// Whether it is a call, for frames and register preferences: it
    /// clobbers registers (a call's are the convention's), and not only
    /// those its description says it writes (x86's division's rdx).
    pub fn is_call(&self, t: &crate::target::Target) -> bool {
        !self.imp_defs.is_empty()
            && match self.op {
                Opcode::Instr(i) => self.imp_defs != t.instrs[i].clobbers,
                Opcode::Asm { .. } => false,
                _ => true,
            }
    }

    pub fn new(op: usize, ops: Vec<MOp>) -> MInst {
        MInst {
            op: Opcode::Instr(op),
            ops,
            imp_uses: Vec::new(),
            imp_defs: Vec::new(),
            class: None,
            noreturn: false,
            barrier: false,
            size: 0,
        }
    }

    pub fn copy(dst: Reg, src: Reg) -> MInst {
        MInst {
            op: Opcode::Copy,
            ops: vec![MOp::Reg(dst), MOp::Reg(src)],
            imp_uses: Vec::new(),
            imp_defs: Vec::new(),
            class: None,
            noreturn: false,
            barrier: false,
            size: 0,
        }
    }

    /// The registers this instruction defines and uses, by operand index.
    pub fn defs_uses(&self, t: &Target) -> (Vec<usize>, Vec<usize>) {
        let mut defs = Vec::new();
        let mut uses = Vec::new();
        match self.op {
            Opcode::Copy => {
                defs.push(0);
                uses.push(1);
            }
            Opcode::Cfi(_) => {}
            Opcode::Asm { outs, .. } => {
                for (k, op) in self.ops.iter().enumerate() {
                    if matches!(op, MOp::Reg(_)) {
                        if k < outs as usize { defs.push(k) } else { uses.push(k) }
                    }
                }
            }
            Opcode::Instr(i) => {
                let ins = &t.instrs[i];
                for (k, o) in ins.operands.iter().enumerate() {
                    if !matches!(self.ops.get(k), Some(MOp::Reg(_))) {
                        continue;
                    }
                    if (k == 0 && ins.has_result) || (k == 1 && ins.second_result) {
                        defs.push(k);
                    } else if matches!(o.kind, Kind::Reg(_)) {
                        uses.push(k);
                        if o.writeback {
                            defs.push(k);
                        }
                    }
                }
            }
        }
        (defs, uses)
    }
}

#[derive(Clone, Debug)]
pub struct MBlock {
    pub label: String,
    pub insts: Vec<MInst>,
    pub succs: Vec<usize>,
    /// Where the instructions for the block's terminator start: copies for
    /// the successors' phis go here.
    pub term_start: usize,
}

#[derive(Clone, Debug)]
pub struct Slot {
    pub size: u64,
    pub align: u64,
    pub offset: i64,
}

#[derive(Clone, Debug)]
pub struct MFunc {
    pub name: String,
    pub global: bool,
    pub blocks: Vec<MBlock>,
    /// Each virtual register's class.
    pub vclass: Vec<Class>,
    /// The inline assembly blocks its `Opcode::Asm` instructions are.
    pub asms: Vec<AsmBlock>,
    pub slots: Vec<Slot>,
    pub frame_size: i64,
    /// Bytes at the bottom of the frame for arguments to calls.
    pub out_size: i64,
    /// `sp` moves (dynamic allocas): the frame is addressed from the
    /// convention's `base` register.
    pub dynamic_stack: bool,
    /// Exception handling: the personality routine, the types caught, and
    /// the landing pads' actions (see `eh.rs`), if it has landing pads.
    pub eh: Option<EhInfo>,
    pub abi: crate::abi::Abi,
}

impl MFunc {
    pub fn new_vreg(&mut self, class: Class) -> VReg {
        self.vclass.push(class);
        (self.vclass.len() - 1) as VReg
    }

    /// How many bytes an instruction is: its size once known, else 4; for
    /// inline assembly, 4 for each of its statements.
    pub fn bytes(&self, inst: &MInst) -> i64 {
        match inst.op {
            Opcode::Asm { id, .. } => 4 * self.asms[id as usize].statements as i64,
            Opcode::Cfi(_) => 0,
            _ if inst.size > 0 => inst.size as i64,
            _ => 4,
        }
    }
}

/// An inline assembly block, with its operands written in.
pub fn render_asm(t: &Target, f: &MFunc, inst: &MInst, block: &AsmBlock) -> String {
    let m = &t.machine;
    let reg = |r: Reg, c: Class| reg_name(t, r, c);
    let operand = |n: usize, modifier: Option<char>| -> String {
        let Some((k, print)) = block.operands.get(n) else { return format!("${n}?") };
        let modifier = modifier.map(|c| (c, m.asm_modifiers.get(&c)));
        match (&inst.ops[*k], print) {
            (MOp::Imm(v), _) => match modifier {
                Some(('n', None)) => format!("{}", -v),
                Some(('z', None)) if *v == 0 => m.zero.map_or("0".into(), |z| m.lookup_name(z)),
                Some((_, Some(crate::machine::AsmModifier::IfImm(text)))) => text.clone(),
                Some(('c', None)) => format!("{v}"),
                _ => m.asm_imm.replace("{i}", &v.to_string()),
            },
            (MOp::Reg(r), AsmPrint::Mem(tpl, c)) => tpl.replace("{r}", &reg(*r, *c)),
            (MOp::Reg(r), AsmPrint::Reg(c)) => match modifier {
                Some((_, Some(crate::machine::AsmModifier::Class(mc)))) => reg(*r, *mc),
                Some((_, Some(crate::machine::AsmModifier::IfImm(_)))) => String::new(),
                _ => reg(*r, *c),
            },
            (other, _) => format!("{other:?}"),
        }
    };
    let private = match f.abi.syntax {
        crate::abi::Syntax::Elf => ".L",
        crate::abi::Syntax::MachO => "L",
    };
    let src: Vec<char> = block.template.chars().collect();
    let mut out = String::new();
    let mut k = 0;
    while k < src.len() {
        let c = src[k];
        k += 1;
        if c != '$' || k >= src.len() {
            out.push(c);
            continue;
        }
        match src[k] {
            '$' => {
                out.push('$');
                k += 1;
            }
            d if d.is_ascii_digit() => {
                let start = k;
                while k < src.len() && src[k].is_ascii_digit() {
                    k += 1;
                }
                let n: usize = src[start..k].iter().collect::<String>().parse().unwrap();
                out.push_str(&operand(n, None));
            }
            '{' => {
                let close = src[k..].iter().position(|&c| c == '}').map_or(src.len(), |p| k + p);
                let inner: String = src[k + 1..close].iter().collect();
                k = (close + 1).min(src.len());
                match inner.split_once(':') {
                    Some(("", "uid")) => out.push_str(&block.uid.to_string()),
                    Some(("", "comment")) => out.push_str(&m.asm_comment),
                    Some(("", "private")) => out.push_str(private),
                    Some((n, md)) => out.push_str(&operand(n.parse().unwrap_or(usize::MAX), md.chars().next())),
                    None => out.push_str(&operand(inner.parse().unwrap_or(usize::MAX), None)),
                }
            }
            _ => out.push('$'),
        }
    }
    let c = &m.asm_comment;
    format!("{c}APP\n\t{out}\n\t{c}NO_APP")
}

// ---------------------------------------------------------------------------
// Printing

pub fn reg_name(t: &Target, r: Reg, class: Class) -> String {
    match r {
        Reg::P(n) => t.machine.name(class, n),
        Reg::V(v) => format!("%{}{v}", t.machine.def(class).name),
    }
}

/// The assembly for one instruction.
pub fn format_inst(t: &Target, f: &MFunc, inst: &MInst) -> String {
    match inst.op {
        Opcode::Copy => {
            let class = |o: &MOp| match o {
                MOp::Reg(Reg::V(v)) => Some(f.vclass[*v as usize]),
                _ => inst.class,
            };
            let r = |o: &MOp| match (o, class(o).or(class(&inst.ops[0])).or(class(&inst.ops[1]))) {
                (MOp::Reg(Reg::P(p)), Some(c)) if t.machine.field(c, *p).is_none() => {
                    let file = t.machine.file_of(*p);
                    let c = t.machine.scalar_in(file, t.machine.def(c).bits).unwrap_or(c);
                    reg_name(t, Reg::P(*p), c)
                }
                (MOp::Reg(r), Some(c)) => reg_name(t, *r, c),
                (MOp::Reg(Reg::P(p)), None) => format!("r{p}"),
                _ => "?".into(),
            };
            format!("COPY {}, {}", r(&inst.ops[0]), r(&inst.ops[1]))
        }
        Opcode::Asm { id, .. } => render_asm(t, f, inst, &f.asms[id as usize]),
        Opcode::Cfi(c) => {
            let m = &t.machine;
            match c {
                crate::unwind::Cfi::DefCfaOffset(o) => format!(".cfi_def_cfa_offset {o}"),
                crate::unwind::Cfi::DefCfa(r, o) => format!(".cfi_def_cfa {}, {o}", m.lookup_name(r)),
                crate::unwind::Cfi::Offset(r, o) => format!(".cfi_offset {}, {o}", m.lookup_name(r)),
            }
        }
        Opcode::Instr(i) => {
            let value = |k: usize| -> Operand {
                match &inst.ops[k] {
                    MOp::Reg(r) => Operand::Reg(*r),
                    MOp::Imm(v) => Operand::Imm(*v),
                    MOp::Label(b) => Operand::Text(f.blocks[*b].label.clone()),
                    MOp::Sym(s) => Operand::Sym(s.clone()),
                    MOp::Frame(s, o) => Operand::Text(format!("frame{s}+{o}")),
                    MOp::Incoming(o) => Operand::Text(format!("incoming+{o}")),
                    MOp::OutArea => Operand::Text("outarea".into()),
                }
            };
            fill_template(t, Some(&f.abi), i, value)
        }
    }
}

pub enum Operand {
    Reg(Reg),
    Imm(i64),
    Text(String),
    /// A symbol, written as the platform writes it.
    Sym(String),
}

/// The brace a template's `{lb}` or `{rb}` writes.
pub fn literal_brace(hole: &str) -> Option<char> {
    match hole {
        "lb" => Some('{'),
        "rb" => Some('}'),
        _ => None,
    }
}

/// Fills in instruction `i`'s template, with `value(k)` for operand `k`.
/// `abi` says how to write symbols (`{s:page}` and `{s:pageoff}` are the
/// page holding a symbol and its offset in it).
pub fn fill_template(t: &Target, abi: Option<&crate::abi::Abi>, i: usize, value: impl Fn(usize) -> Operand) -> String {
    let ins = &t.instrs[i];
    let tpl = &ins.asm;
    let mut out = String::new();
    let mut rest = tpl.as_str();
    while let Some(open) = rest.find('{') {
        out.push_str(&rest[..open]);
        let close = rest[open..].find('}').map(|c| open + c).unwrap_or(rest.len());
        let inner = &rest[open + 1..close];
        rest = &rest[(close + 1).min(rest.len())..];
        // Braces themselves (`tbl`'s register list): `{lb}` and `{rb}`.
        if let Some(b) = literal_brace(inner) {
            out.push(b);
            continue;
        }
        if let Some(expr) = inner.strip_prefix('=') {
            let mut env = Env::new();
            for (k, o) in ins.operands.iter().enumerate() {
                if let Operand::Imm(v) = value(k) {
                    env.insert(o.name.clone(), v);
                }
            }
            let v = enc::parse_expr(expr).ok().and_then(|e| enc::eval(&e, &env));
            let _ = write!(out, "{}", v.map_or("?".to_string(), |v| v.to_string()));
            continue;
        }
        let (name, modifier) = match inner.split_once(':') {
            Some((n, m)) => (n, Some(m)),
            None => (inner, None),
        };
        let Some(k) = ins.operands.iter().position(|o| o.name == name) else {
            out.push('?');
            continue;
        };
        let o = &ins.operands[k];
        match (value(k), o.kind) {
            (Operand::Reg(r), Kind::Reg(class)) => {
                let m = &t.machine;
                // A vector arrangement, `{n:4s}`: v3.4s; or a lane's size,
                // `{n:.s}`: v3.s (followed in the template by `[k]`).
                if let Some(arr) =
                    modifier.filter(|m| m.len() >= 2 && (m.as_bytes()[0].is_ascii_digit() || m.starts_with('.')))
                {
                    let arr = arr.trim_start_matches('.');
                    match (r, m.class("lanes")) {
                        (Reg::P(n), Some(lanes)) => out.push_str(&format!("{}.{arr}", m.name(lanes, n))),
                        (Reg::P(_), None) => out.push('?'),
                        (Reg::V(v), _) => out.push_str(&format!("%v{v}.{arr}")),
                    }
                    continue;
                }
                // The register as another class names it: `{n:w}` is w3,
                // `{n:s}` s3, `{n:b}` b3.
                let class = match modifier {
                    Some(v) => match m.class(v) {
                        Some(c) => c,
                        None => {
                            out.push('?');
                            continue;
                        }
                    },
                    None => class,
                };
                out.push_str(&reg_name(t, r, class));
            }
            (Operand::Imm(v), Kind::Cond) => {
                out.push_str(t.conds.iter().find(|c| c.value == v).map_or("?", |c| c.asm.as_str()));
            }
            (Operand::Imm(v), Kind::Enum(e)) => out.push_str(t.machine.enums[e as usize].name(v).unwrap_or("?")),
            (Operand::Imm(v), _) => match modifier {
                Some("h32") => {
                    let _ = write!(out, "0x{:x}", v as u64 & 0xffff_ffff);
                }
                Some("h64") => {
                    let _ = write!(out, "0x{:x}", v as u64);
                }
                // A floating-point immediate's bits, as the number, the
                // way assemblers print fmov's (eight decimals hold every
                // value it can).
                Some("f64") => {
                    let _ = write!(out, "{:.8}", f64::from_bits(v as u64));
                }
                Some("f32") => {
                    let _ = write!(out, "{:.8}", f32::from_bits(v as u32));
                }
                _ => {
                    let _ = write!(out, "{v}");
                }
            },
            (Operand::Text(s), _) => out.push_str(&s),
            // A symbol, as the relocation that fills it in writes it.
            (Operand::Sym(s), _) => out.push_str(&match (abi, ins.reloc) {
                (Some(a), Some(k)) => t.machine.relocs[k.0 as usize].spell(a.syntax, &a.symbol(&s)),
                (Some(a), None) => a.symbol(&s),
                (None, _) => s,
            }),
            (Operand::Reg(r), _) => out.push_str(&match r {
                Reg::P(p) => format!("r{p}"),
                Reg::V(v) => format!("%{v}"),
            }),
        }
    }
    out.push_str(rest);
    out
}

// ---------------------------------------------------------------------------
// Encoding

#[derive(Clone, Debug, PartialEq)]
pub struct Fixup {
    /// Where the field holding the symbol starts (its piece), from the
    /// function's start.
    pub offset: usize,
    /// How far into its instruction that is.
    pub at: usize,
    pub symbol: String,
    pub instr: String,
    /// The field's value with the symbol at 0 (a displacement added, or
    /// the instruction's end subtracted).
    pub addend: i64,
}

/// A function's machine code: its bytes, where each instruction starts,
/// and a fixup for each symbol reference.
#[derive(Clone, Debug, Default)]
pub struct Code {
    pub bytes: Vec<u8>,
    pub starts: Vec<usize>,
    pub fixups: Vec<Fixup>,
}

/// The operands of `inst` at `pc`, with blocks at `starts`: labels are
/// measured from the instruction's start and symbols are 0.
fn inst_env(t: &Target, f: &MFunc, inst: &MInst, pc: usize, starts: &[usize]) -> Result<Env, String> {
    let Opcode::Instr(i) = inst.op else { unreachable!() };
    let mut env = Env::new();
    for (o, op) in t.instrs[i].operands.iter().zip(&inst.ops) {
        let v = match op {
            MOp::Reg(Reg::P(r)) => t.machine.index(*r),
            MOp::Reg(Reg::V(_)) => return Err(format!("{}: a virtual register left after allocation", f.name)),
            MOp::Imm(v) => *v,
            MOp::Label(target) => starts[*target] as i64 - pc as i64,
            MOp::Sym(_) => 0,
            MOp::Frame(..) | MOp::Incoming(_) | MOp::OutArea => {
                return Err(format!("{}: a frame offset left after layout", f.name));
            }
        };
        env.insert(o.name.clone(), v);
    }
    Ok(env)
}

/// How many bytes each instruction of each block takes once encoded.
/// Where an instruction has encodings of several lengths (a short branch
/// and a long one), the lengths are found together: each starts as short
/// as it can be and grows until all fit, which ends, as growing only
/// moves targets further.
pub fn sizes(t: &Target, f: &MFunc) -> Result<Vec<Vec<usize>>, String> {
    let mut size: Vec<Vec<usize>> = f
        .blocks
        .iter()
        .map(|b| {
            b.insts
                .iter()
                .map(|i| match i.op {
                    Opcode::Instr(k) => t.instrs[k].enc.alts.iter().map(|a| a.len as usize).min().unwrap_or(0),
                    _ => f.bytes(i) as usize,
                })
                .collect()
        })
        .collect();
    let fixed = f.blocks.iter().flat_map(|b| &b.insts).all(|i| match i.op {
        Opcode::Instr(k) => t.instrs[k].enc.alts.len() <= 1,
        _ => true,
    });
    if fixed {
        return Ok(size);
    }
    loop {
        let starts = block_starts(&size);
        let mut changed = false;
        for (bi, b) in f.blocks.iter().enumerate() {
            let mut pc = starts[bi];
            for (k, inst) in b.insts.iter().enumerate() {
                if let Opcode::Instr(i) = inst.op
                    && t.instrs[i].enc.alts.len() > 1
                {
                    let env = inst_env(t, f, inst, pc, &starts)?;
                    let n = t.instrs[i]
                        .encode(&env)
                        .ok_or_else(|| format!("{}: cannot encode `{}`", f.name, format_inst(t, f, inst)))?
                        .len();
                    if n > size[bi][k] {
                        size[bi][k] = n;
                        changed = true;
                    }
                }
                pc += size[bi][k];
            }
        }
        if !changed {
            return Ok(size);
        }
    }
}

fn block_starts(size: &[Vec<usize>]) -> Vec<usize> {
    let mut at = 0;
    size.iter()
        .map(|b| {
            let s = at;
            at += b.iter().sum::<usize>();
            s
        })
        .collect()
}

/// Records each instruction's size (see `sizes`) in it, for what measures
/// the code without encoding it (unwind and exception tables).
pub fn set_sizes(t: &Target, f: &mut MFunc) {
    if let Ok(size) = sizes(t, f) {
        for (b, s) in f.blocks.iter_mut().zip(size) {
            for (inst, n) in b.insts.iter_mut().zip(s) {
                if matches!(inst.op, Opcode::Instr(_)) {
                    inst.size = n as u8;
                }
            }
        }
    }
}

/// The machine code for a function laid out and allocated, with a fixup
/// for each symbol reference. Branches within the function are resolved.
pub fn encode_function(t: &Target, f: &MFunc) -> Result<Code, String> {
    let size = sizes(t, f)?;
    let starts = block_starts(&size);
    let mut code = Code::default();
    for (bi, b) in f.blocks.iter().enumerate() {
        let mut pc = starts[bi];
        for (k, inst) in b.insts.iter().enumerate() {
            let i = match inst.op {
                Opcode::Cfi(_) => continue,
                Opcode::Instr(i) => i,
                Opcode::Copy => return Err(format!("{}: a copy left after allocation", f.name)),
                Opcode::Asm { .. } => {
                    return Err(format!("{}: inline assembly is encoded by assembling the assembly", f.name));
                }
            };
            let ins = &t.instrs[i];
            let env = inst_env(t, f, inst, pc, &starts)?;
            let cannot = || format!("{}: cannot encode `{}`", f.name, format_inst(t, f, inst));
            // As long as `sizes` settled on: padding before it may have
            // shrunk since, so that a branch made long would now fit short,
            // but its length is part of where everything after it is.
            let (mut bytes, alt) = if ins.enc.alts.len() > 1 {
                ins.encode_alt_of_len(&env, size[bi][k] as u32)
            } else {
                ins.encode_alt(&env)
            }
            .ok_or_else(cannot)?;
            if bytes.len() != size[bi][k] {
                return Err(format!("{}: `{}` changed size", f.name, format_inst(t, f, inst)));
            }
            for (o, op) in ins.operands.iter().zip(&inst.ops) {
                let MOp::Sym(s) = op else { continue };
                let (at, addend) = ins.field_of(alt, &o.name, &env);
                // (The relocation's addend fills the field in.)
                if addend != 0 {
                    ins.enc.alts[alt].clear(&mut bytes, &o.name);
                }
                code.fixups.push(Fixup { offset: pc + at, at, symbol: s.clone(), instr: ins.name.clone(), addend });
            }
            code.starts.push(pc);
            code.bytes.extend_from_slice(&bytes);
            pc += bytes.len();
        }
    }
    Ok(code)
}

/// The function's unwind rules, each at its offset from the function's
/// start (after the instruction before it).
pub fn cfi_rules(f: &MFunc) -> Vec<(u64, crate::unwind::Cfi)> {
    let mut out = Vec::new();
    let mut at = 0u64;
    for b in &f.blocks {
        for inst in &b.insts {
            match inst.op {
                Opcode::Cfi(c) => out.push((at, c)),
                _ => at += f.bytes(inst) as u64,
            }
        }
    }
    out
}

// ---------------------------------------------------------------------------
// Disassembly

/// The instruction at the start of `bytes`, its operands and its length,
/// if the description has it: the candidates whose literal bits match,
/// most literal bits first.
pub fn decode(t: &Target, bytes: &[u8]) -> Option<(usize, Env, usize)> {
    let cands = t.decode_candidates(bytes);
    for (i, alt) in cands {
        let ins = &t.instrs[i];
        if !ins.decode {
            continue;
        }
        let Some((env, len)) = ins.decode_alt(bytes, alt) else { continue };
        // A condition must be one the description names.
        let conds_ok =
            ins.operands.iter().all(|o| o.kind != Kind::Cond || t.conds.iter().any(|c| c.value == env[&o.name]));
        if conds_ok {
            return Some((i, env, len));
        }
    }
    None
}

/// Disassembles the instruction at the start of `bytes`, at address `pc`,
/// and gives its length. What does not decode is a `.word` (or on a target
/// with instructions shorter than 4 bytes, a `.byte`).
pub fn disassemble(t: &Target, bytes: &[u8], pc: u64) -> (String, usize) {
    match decode(t, bytes) {
        None if t.unit() == 4 && bytes.len() >= 4 => (format!(".word 0x{:08x}", crate::enc::word(&bytes[..4])), 4),
        None => (format!(".byte 0x{:02x}", bytes.first().copied().unwrap_or(0)), 1),
        Some((i, env, len)) => {
            let ins = &t.instrs[i];
            let text = fill_template(t, None, i, |k| {
                let o = &ins.operands[k];
                let v = env[&o.name];
                match o.kind {
                    Kind::Reg(c) => Operand::Reg(Reg::P(t.machine.phys(c, v))),
                    Kind::Label => Operand::Text(format!("0x{:x}", (pc as i64 + v) as u64)),
                    Kind::Sym => Operand::Text("<symbol>".into()),
                    Kind::Imm | Kind::Cond | Kind::Enum(_) => Operand::Imm(v),
                }
            });
            (text, len)
        }
    }
}
