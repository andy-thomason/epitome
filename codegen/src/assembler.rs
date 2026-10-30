//! An assembler: AArch64 assembly text to an object file (Mach-O or ELF),
//! with the machine description run backwards.
//!
//! Each instruction in the description says how it is written, in its
//! `"asm"` template (`add {d}, {n}, #{imm}`). To read a line, the
//! templates whose first word is the line's mnemonic are tried. A template
//! is literal text and holes; the text between holes must be in the line
//! as it is, and each hole takes what comes before the next piece of
//! literal text. A hole's text is then read loosely, as what its operand
//! is: a register (`w3`, `sp`, `v2.4s`, `q0`), a number (`#-12`, `0x1f`), a
//! condition (`ne`), a symbol (spelled as the instruction's relocation
//! spells it: `_x@PAGEOFF`, `:lo12:x`), or a label. Holes
//! that are expressions (`#{=0-imm}`) are solved for their operand.
//!
//! A reading is kept only if the template, filled in with the values read,
//! gives back the line exactly, and if the values encode (and decode back
//! to themselves: an immediate the encoding cannot hold does not fit). So
//! what is accepted is exactly what the printer writes, and the printer is
//! what the rest of the code generator is tested with. Where two
//! instructions both fit (`mov x0, x1` is both `orr` from `xzr` and `add`
//! to the stack pointer), the one using fewer registers of the
//! stack-pointer classes is chosen, which is the instruction an
//! assembler's alias rules choose; then the first in the description.
//!
//! Assembly is in two passes. The first lays out sections, labels and
//! data, an instruction being 4 bytes; the second encodes the
//! instructions, now that labels have addresses. References to symbols
//! are resolved as `object::resolve` does for compiled code, by the
//! format's rules, so the object file is the one `epitome-llc` writes
//! directly for the same program.
//!
//! The directives are those the code generator writes, and modules'
//! own assembly (Rust's `global_asm!`) uses: `.text`, `.data`, `.bss`,
//! `.section`, `.pushsection` and `.popsection`, `.globl`, `.p2align`,
//! `.type`, `.size`, `.byte`,
//! `.short`, `.long`, `.quad` (of numbers, or of `symbol+offset`),
//! `.ascii`, `.asciz`, `.space`, `.zero` and `.build_version`; the
//! `.cfi_*` directives of unwind tables, with `.cfi_personality` and
//! `.cfi_lsda` (the encodings we write); `.long symbol@GOT-label` (Mach-O)
//! or `.long symbol-label` (ELF), with the label at the field (exception
//! tables' references to types) or, for ELF, earlier in its section (a
//! relative lookup table's entries); and
//! `.subsections_via_symbols`, which is accepted and ignored. Also those
//! of hand-written assembly, as GNU assemblers take them: `.weak`,
//! `.word`, `.dword`, `.half`, `.float16`, `.float`, `.double`, `.fill`,
//! `.balign`, `.option` (ignored), and `.macro`, `.rept` and `.if`
//! (`asm_text`), with expressions of constants wherever a number goes.
//!
//! Before an instruction is read, its operands that are expressions become
//! numbers, names of numbers and other names of registers the description
//! gives become what the templates write, and its pseudo-instructions of
//! several instructions become those (`asm_text::instruction`). One that
//! does not fit (a branch too far) is read as the longer form the
//! description gives (`!target.asm-relax`).

use std::collections::{HashMap, HashSet};

use crate::abi::Syntax;
use crate::enc::{self, Env};
use crate::machine::RelocUse;
use crate::mir::{self, Operand, Reg};
use crate::object::{self, Object, Pending, RelocKind, Section, SectionKind, Symbol, SymbolKind};
use crate::target::{Kind, Target};

/// A function between `.cfi_startproc` and `.cfi_endproc`.
struct OpenFrame {
    section: usize,
    start: u64,
    /// Its label.
    name: Option<String>,
    rules: Vec<(u64, crate::unwind::Cfi)>,
    personality: Option<String>,
    /// The label of its LSDA.
    lsda: Option<String>,
}

/// An assembled file.
pub struct Assembled {
    pub object: Object,
    pub syntax: Syntax,
    /// From `.build_version`, for Mach-O; else 11.0.
    pub min_os: (u32, u32, u32),
}

impl Assembled {
    /// The object file, in the format the assembly was written for.
    pub fn file(&self, t: &Target) -> Result<Vec<u8>, String> {
        match self.syntax {
            Syntax::Elf => crate::elf::write(&self.object, &t.machine),
            Syntax::MachO => crate::macho::write(&self.object, self.min_os, &t.machine),
        }
    }
}

#[derive(Clone, Debug)]
enum Part {
    Lit(String),
    Hole(String),
}

#[derive(Clone, Debug)]
struct Template {
    instr: usize,
    parts: Vec<Part>,
}

fn split_template(tpl: &str) -> Vec<Part> {
    let mut parts = Vec::new();
    let mut rest = tpl;
    while let Some(open) = rest.find('{') {
        if open > 0 {
            parts.push(Part::Lit(rest[..open].to_string()));
        }
        let close = rest[open..].find('}').map_or(rest.len(), |c| open + c);
        let hole = &rest[open + 1..close];
        match crate::mir::literal_brace(hole) {
            Some(b) => parts.push(Part::Lit(b.to_string())),
            None => parts.push(Part::Hole(hole.to_string())),
        }
        rest = &rest[(close + 1).min(rest.len())..];
    }
    if !rest.is_empty() {
        parts.push(Part::Lit(rest.to_string()));
    }
    // Literal text side by side (a brace beside the text) is one piece.
    let mut merged: Vec<Part> = Vec::new();
    for p in parts {
        match (merged.last_mut(), p) {
            (Some(Part::Lit(a)), Part::Lit(b)) => a.push_str(&b),
            (_, p) => merged.push(p),
        }
    }
    merged
}

fn mnemonic(s: &str) -> &str {
    s.split_whitespace().next().unwrap_or("")
}

/// Reads the assembly of `t`'s instructions.
pub struct Assembler<'t> {
    t: &'t Target,
    by_mnemonic: HashMap<String, Vec<Template>>,
    /// Templates with a hole in the mnemonic (`b.{c}`), by the literal
    /// text before it.
    by_prefix: Vec<(String, Template)>,
    /// How many lines that cannot be assembled to report.
    pub max_errors: usize,
}

/// What a line's holes said.
/// A symbol an instruction refers to: where its field is in the
/// instruction, and the field's value with it at 0.
struct Sym {
    name: String,
    at: usize,
    addend: i64,
}

struct Reading {
    env: Env,
    shown: HashMap<usize, Operand>,
    /// The symbol operand: its index and the symbol's name, and the offset
    /// from it written.
    sym: Option<(usize, String)>,
    sym_offset: i64,
    /// The label operand: its index and the label.
    label: Option<(usize, String)>,
}

fn parse_int(s: &str) -> Option<i64> {
    let s = s.trim();
    let (neg, body) = match s.strip_prefix('-') {
        Some(b) => (true, b),
        None => (false, s.strip_prefix('+').unwrap_or(s)),
    };
    let v = match body.strip_prefix("0x").or_else(|| body.strip_prefix("0X")) {
        Some(h) => u64::from_str_radix(h, 16).ok(),
        None => body.parse::<u64>().ok(),
    };
    // (Else an expression of constants: `0x6000 & (0x6000 >> 1)`.)
    match v {
        Some(v) => Some(if neg { (v as i64).wrapping_neg() } else { v as i64 }),
        None => crate::asm_text::eval(s),
    }
}

fn valid_symbol(s: &str) -> bool {
    !s.is_empty()
        && !s.starts_with(|c: char| c.is_ascii_digit())
        && s.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '.' | '$'))
}

impl<'t> Assembler<'t> {
    pub fn new(t: &'t Target) -> Assembler<'t> {
        let mut by_mnemonic: HashMap<String, Vec<Template>> = HashMap::new();
        let mut by_prefix = Vec::new();
        for (i, ins) in t.instrs.iter().enumerate() {
            if ins.asm.is_empty() {
                continue;
            }
            let tpl = Template { instr: i, parts: split_template(&ins.asm) };
            // (By the mnemonic in lower case, as lines are looked up.)
            let m = mnemonic(&ins.asm).to_ascii_lowercase();
            match m.find('{') {
                Some(k) => by_prefix.push((m[..k].to_string(), tpl)),
                None => by_mnemonic.entry(m.to_string()).or_default().push(tpl),
            }
        }
        Assembler { t, by_mnemonic, by_prefix, max_errors: 20 }
    }

    /// The holes' texts, if `line` has the template's literal text: a hole
    /// takes the text to the nearest occurrence of the literal text after
    /// it, outside brackets; or, before a bracket, the nearest from which
    /// the rest of the template matches (`%pcrel_lo(.L1)` before `(t5)`).
    fn holes<'l>(&self, tpl: &Template, line: &'l str) -> Option<Vec<(String, &'l str)>> {
        let mut out = Vec::new();
        self.holes_from(tpl, 0, line, 0, &mut out).then_some(out)
    }

    /// Whether `line` from `pos` matches the template from part `k`, the
    /// holes' texts pushed to `out`. A hole of an enum operand is one of its
    /// names (perhaps empty), each tried in turn until the rest matches, so
    /// enum holes may be side by side (`{neg}{src}`); any other hole is not
    /// empty, is balanced in brackets, and runs to an occurrence of the next
    /// literal text, the first unless that text has a bracket.
    fn holes_from<'l>(
        &self,
        tpl: &Template,
        k: usize,
        line: &'l str,
        pos: usize,
        out: &mut Vec<(String, &'l str)>,
    ) -> bool {
        let Some(part) = tpl.parts.get(k) else { return pos == line.len() };
        match part {
            Part::Lit(l) => {
                line[pos..].starts_with(l.as_str()) && self.holes_from(tpl, k + 1, line, pos + l.len(), out)
            }
            Part::Hole(inner) => {
                if let Some(e) = self.enum_of(tpl.instr, inner) {
                    for (name, _) in &e.names {
                        if line[pos..].starts_with(name.as_str()) {
                            out.push((inner.clone(), &line[pos..pos + name.len()]));
                            if self.holes_from(tpl, k + 1, line, pos + name.len(), out) {
                                return true;
                            }
                            out.pop();
                        }
                    }
                    return false;
                }
                let balanced = |s: &str| {
                    if !s.contains(['(', ')']) {
                        return true;
                    }
                    let mut depth = 0i32;
                    s.chars().all(|c| {
                        depth += (c == '(') as i32 - (c == ')') as i32;
                        depth >= 0
                    }) && depth == 0
                };
                let take = |end: usize, out: &mut Vec<(String, &'l str)>| {
                    if end == pos || !balanced(&line[pos..end]) {
                        return None;
                    }
                    out.push((inner.clone(), &line[pos..end]));
                    let ok = self.holes_from(tpl, k + 1, line, end, out);
                    if !ok {
                        out.pop();
                    }
                    Some(ok)
                };
                match tpl.parts.get(k + 1) {
                    None => take(line.len(), out) == Some(true),
                    Some(Part::Hole(_)) => false,
                    Some(Part::Lit(l)) => {
                        let again = l.contains('(');
                        for (at, _) in line[pos..].match_indices(l.as_str()) {
                            match take(pos + at, out) {
                                Some(true) => return true,
                                Some(false) if !again => return false,
                                _ => {}
                            }
                        }
                        false
                    }
                }
            }
        }
    }

    /// The enum a template's hole is an operand of, if it is one.
    fn enum_of(&self, i: usize, hole: &str) -> Option<&crate::machine::EnumDef> {
        let o = self.t.instrs[i].operands.iter().find(|o| o.name == hole)?;
        match o.kind {
            Kind::Enum(e) => Some(&self.t.machine.enums[e as usize]),
            _ => None,
        }
    }

    /// Reads a line as instruction `i`, by the holes' texts.
    fn read(&self, i: usize, holes: &[(String, &str)], syntax: Syntax) -> Option<Reading> {
        let ins = &self.t.instrs[i];
        let mut r = Reading { env: Env::new(), shown: HashMap::new(), sym: None, sym_offset: 0, label: None };
        let set = |env: &mut Env, name: &str, v: i64| -> Option<()> {
            match env.insert(name.to_string(), v) {
                Some(old) if old != v => None,
                _ => Some(()),
            }
        };
        for (inner, text) in holes {
            if let Some(expr) = inner.strip_prefix('=') {
                let e = enc::parse_expr(expr).ok()?;
                let mut x = parse_int(text)?;
                // A value written masked to n bits (`{=(imm>>12)&1048575}`,
                // as RISC-V's lui is written) is read as n bits, signed.
                if let enc::Expr::Bin('&', _, m) = &e
                    && let enc::Expr::Int(m) = **m
                    && (m + 1).count_ones() == 1
                    && x > m / 2
                    && x <= m
                {
                    x -= m + 1;
                }
                let (name, v) = enc::invert(&e, x)?;
                set(&mut r.env, &name, v)?;
                continue;
            }
            let (name, modifier) = match inner.split_once(':') {
                Some((n, m)) => (n, Some(m)),
                None => (inner.as_str(), None),
            };
            let k = ins.operands.iter().position(|o| o.name == name)?;
            match ins.operands[k].kind {
                Kind::Reg(class) => {
                    // By any of its names (a vector's without its
                    // arrangement); the template, printing it back,
                    // decides whether the name was the right one.
                    let reg = text.split('.').next()?;
                    let p = self.t.machine.lookup(reg).or_else(|| self.t.machine.lookup(&reg.to_ascii_lowercase()))?;
                    let n = self.t.machine.field(class, p)?;
                    set(&mut r.env, name, n)?;
                    r.shown.insert(k, Operand::Reg(Reg::P(p)));
                }
                // A floating-point immediate, written as the number.
                Kind::Imm if modifier == Some("f64") => {
                    set(&mut r.env, name, text.parse::<f64>().ok()?.to_bits() as i64)?
                }
                Kind::Imm if modifier == Some("f32") => {
                    set(&mut r.env, name, text.parse::<f32>().ok()?.to_bits() as i64)?
                }
                Kind::Imm => set(&mut r.env, name, parse_int(text)?)?,
                Kind::Enum(e) => set(&mut r.env, name, self.t.machine.enums[e as usize].value(text)?)?,
                Kind::Cond => {
                    // (In either case: `NE` is `ne`.)
                    let c = self.t.conds.iter().find(|c| c.asm.eq_ignore_ascii_case(text))?;
                    set(&mut r.env, name, c.value)?;
                }
                Kind::Sym => {
                    // As the instruction's relocation spells it.
                    let sym = match ins.reloc {
                        Some(rk) => self.t.machine.relocs[rk.0 as usize].unspell(syntax, text)?,
                        None => text,
                    };
                    // (A symbol and an offset, `tohost + 4`, are the symbol
                    // with an addend.)
                    let (s, off) = match valid_symbol(sym) {
                        true => (sym.to_string(), 0),
                        false => parse_sym_offset(&sym.replace(' ', ""))?,
                    };
                    set(&mut r.env, name, 0)?;
                    r.sym = Some((k, s));
                    r.sym_offset = off;
                    r.shown.insert(k, Operand::Text(text.to_string()));
                }
                Kind::Label => {
                    if !valid_symbol(text) && here_relative(text).is_none() {
                        return None;
                    }
                    r.label = Some((k, text.to_string()));
                    r.shown.insert(k, Operand::Text(text.to_string()));
                }
            }
        }
        Some(r)
    }

    /// `line` with its mnemonic, and the words that are registers or
    /// conditions, in lower case (symbols are left as they are).
    fn lower_names(&self, line: &str) -> String {
        let mut out = String::new();
        let mut word = String::new();
        let mut first = true;
        let flush = |word: &mut String, out: &mut String, first: &mut bool| {
            if !word.is_empty() {
                let l = word.to_ascii_lowercase();
                let name = l.split('.').next().unwrap_or("");
                if *first || self.t.machine.lookup(name).is_some() || self.t.conds.iter().any(|c| c.asm == l) {
                    out.push_str(&l);
                } else {
                    out.push_str(word);
                }
                *first = false;
                word.clear();
            }
        };
        for c in line.chars() {
            if c.is_ascii_alphanumeric() || c == '_' || c == '.' {
                word.push(c);
            } else {
                flush(&mut word, &mut out, &mut first);
                out.push(c);
            }
        }
        flush(&mut word, &mut out, &mut first);
        out
    }

    /// The instruction `line` is, encoded at `pc`, with `label` giving the
    /// address of a label in the same section; and the symbol it refers
    /// to, if it does.
    fn encode(
        &self,
        line: &str,
        pc: u64,
        syntax: Syntax,
        label: &dyn Fn(&str) -> Option<u64>,
    ) -> Result<(usize, Vec<u8>, Option<Sym>), String> {
        let lower = mnemonic(line).to_ascii_lowercase();
        let m = lower.as_str();
        let cands: Vec<&Template> = self
            .by_mnemonic
            .get(m)
            .into_iter()
            .flatten()
            .chain(self.by_prefix.iter().filter(|(p, _)| m.starts_with(p.as_str())).map(|(_, t)| t))
            .collect();
        if cands.is_empty() {
            return Err("no instruction has this mnemonic".into());
        }
        let mut best: Option<(usize, usize, Vec<u8>, Option<Sym>)> = None;
        let mut no_label = None;
        for tpl in cands {
            let Some(holes) = self.holes(tpl, line) else { continue };
            let Some(mut r) = self.read(tpl.instr, &holes, syntax) else { continue };
            let ins = &self.t.instrs[tpl.instr];
            if let Some((k, name)) = &r.label {
                // Not a label here: perhaps a symbol, as another entry reads it.
                let Some(at) = here_relative(name).map(|d| pc.wrapping_add_signed(d)).or_else(|| label(name)) else {
                    no_label = Some(format!("no label {name} in this section"));
                    continue;
                };
                r.env.insert(ins.operands[*k].name.clone(), at as i64 - pc as i64);
            }
            // The template, filled in, must give the line back.
            let printed = mir::fill_template(self.t, None, tpl.instr, |k| {
                if let Some(op) = r.shown.get(&k) {
                    return match op {
                        Operand::Reg(x) => Operand::Reg(*x),
                        Operand::Text(s) => Operand::Text(s.clone()),
                        Operand::Imm(v) => Operand::Imm(*v),
                        Operand::Sym(s) => Operand::Sym(s.clone()),
                    };
                }
                match r.env.get(&ins.operands[k].name) {
                    Some(v) => Operand::Imm(*v),
                    None => Operand::Text("?".into()),
                }
            });
            // (In any case: inline assembly may write `NE`, or `X0`.)
            if !printed.eq_ignore_ascii_case(line) {
                continue;
            }
            // The values must encode, and decode back as themselves.
            let Some((bytes, alt)) = ins.encode_alt(&r.env) else { continue };
            let Some((back, _)) = ins.decode_alt(&bytes, alt) else { continue };

            if r.env.iter().any(|(n, v)| back.get(n).is_some_and(|b| b != v)) {
                continue;
            }
            let sp_classes =
                ins.operands.iter().filter(|o| matches!(o.kind, Kind::Reg(c) if self.t.machine.def(c).sp)).count();
            if best.as_ref().is_none_or(|b| sp_classes < b.0) {
                let mut bytes = bytes;
                let sym = r.sym.map(|(k, name)| {
                    let (at, addend) = ins.field_of(alt, &ins.operands[k].name, &r.env);
                    // (The relocation's addend fills the field in.)
                    if addend != 0 {
                        ins.enc.alts[alt].clear(&mut bytes, &ins.operands[k].name);
                    }
                    Sym { name, at, addend: addend + r.sym_offset }
                });
                best = Some((sp_classes, tpl.instr, bytes, sym));
            }
        }
        best.map(|(_, i, b, s)| (i, b, s)).ok_or_else(|| {
            no_label.unwrap_or_else(|| "no instruction is written this way, or its values do not fit".into())
        })
    }

    /// An instruction that does not fit (a branch too far), as the
    /// longer form the description gives: its statements, each where the
    /// one before ends, one after the other (with no symbols).
    fn relaxed(
        &self,
        line: &str,
        pc: u64,
        syntax: Syntax,
        label: &dyn Fn(&str) -> Option<u64>,
    ) -> Option<(usize, Vec<u8>, Option<Sym>)> {
        let canonical = self.t.machine.asm_aliases.iter().fold(line.to_string(), |l, (a, b)| rewrite(&l, a, b));
        for text in [line, canonical.as_str()] {
            for (from, to) in &self.t.machine.asm_relax {
                let long = rewrite(text, from, to);
                if long == text {
                    continue;
                }
                let mut bytes = Vec::new();
                let mut last = None;
                for part in long.split(';') {
                    let (i, b, sym) =
                        self.encode(&normalise(part.trim()), pc + bytes.len() as u64, syntax, label).ok()?;
                    if sym.is_some() {
                        return None;
                    }
                    bytes.extend(b);
                    last = Some(i);
                }
                return Some((last?, bytes, None));
            }
        }
        None
    }

    /// Assembles `text`: Mach-O assembly if it says so (`.build_version`,
    /// or `__TEXT` and `__DATA` sections), or `syntax` says so; else ELF.
    pub fn assemble(&self, text: &str, syntax: Option<Syntax>) -> Result<Assembled, String> {
        // Each instruction's length, by its line: the target's usual one
        // (the shortest, or on RISC-V the uncompressed), until assembling it
        // says otherwise. After that first word, lengths only grow, so
        // distances do, so this ends.
        let mut lens: HashMap<usize, usize> = HashMap::new();
        loop {
            match self.assemble_sized(text, syntax, &lens)? {
                Ok(a) => return Ok(a),
                Err(longer) => lens.extend(longer),
            }
        }
    }

    /// Assembles `text` with instructions of the lengths `lens` gives, or
    /// gives the lengths of those that turned out longer (or, not yet
    /// given one, shorter).
    fn assemble_sized(
        &self,
        text: &str,
        syntax: Option<Syntax>,
        lens: &HashMap<usize, usize>,
    ) -> Result<Result<Assembled, HashMap<usize, usize>>, String> {
        let unit = self.t.unit();
        let mut longer: HashMap<usize, usize> = HashMap::new();
        let syntax = syntax.unwrap_or_else(|| {
            let macho = text.lines().any(|l| {
                let l = l.trim();
                l.starts_with(".build_version")
                    || (l.starts_with(".section") && (l.contains("__TEXT") || l.contains("__DATA")))
            });
            if macho { Syntax::MachO } else { Syntax::Elf }
        });
        let temp = |name: &str| match syntax {
            Syntax::MachO => name.starts_with('L'),
            Syntax::Elf => name.starts_with(".L"),
        };
        let mut obj = Object::default();
        let mut by_kind: HashMap<(SectionKind, Option<String>), usize> = HashMap::new();
        let mut cur: Option<usize> = None;
        // Sections `.pushsection` left, for `.popsection`.
        let mut pushed: Vec<(Option<usize>, bool)> = Vec::new();
        let mut discard = false;
        let mut held: Option<String> = None;
        let mut labels: HashMap<String, (usize, u64)> = HashMap::new();
        let mut defined: Vec<String> = Vec::new();
        let mut sets: Vec<(String, String, i64)> = Vec::new();
        let mut globals: HashSet<String> = HashSet::new();
        let mut types: HashMap<String, SymbolKind> = HashMap::new();
        let mut sizes: HashMap<String, u64> = HashMap::new();
        // Unwind rules: the function open (section, start, its label, its
        // rules), and those closed.
        let mut open: Option<OpenFrame> = None;
        let mut frames: Vec<(OpenFrame, u64)> = Vec::new();
        let mut min_os = (11, 0, 0);
        // What the second pass does, in order: instructions, and data
        // that refers to symbols.
        enum Later {
            Instr {
                section: usize,
                offset: u64,
                line: usize,
                /// Its statement's number, by which its length is kept.
                stmt: usize,
                text: String,
            },
            Data {
                section: usize,
                offset: u64,
                symbol: String,
                addend: i64,
            },
            /// A 4-byte reference to `symbol`'s GOT entry, less `label`.
            Got {
                section: usize,
                offset: u64,
                symbol: String,
                label: String,
                line: usize,
            },
        }
        let mut later: Vec<Later> = Vec::new();
        let lines: Vec<&str> = text.lines().collect();

        let err = |n: usize, msg: String| format!("line {}: {msg}: `{}`", n + 1, lines[n].trim());
        let stmts = statements(&lines, &self.t.machine.asm_comment, syntax).map_err(|(n, m)| err(n, m))?;
        // Instructions with their operands' expressions as numbers, and the
        // pseudo-instructions of several instructions as those
        // (`asm_text::instruction`).
        let mut pcrel = 0;
        let stmts: Vec<(usize, String)> = stmts
            .into_iter()
            .flat_map(|(n, s)| {
                let body = s.trim_start().rsplit_once(':').map_or(s.trim_start(), |(_, b)| b).trim_start();
                let directive = body.starts_with('.') && body[1..].starts_with(|c: char| c.is_ascii_alphabetic());
                let label_only = s.trim_end().ends_with(':');
                if directive || label_only || body.is_empty() || body.contains('"') {
                    return vec![(n, s)];
                }
                let mut fresh = || {
                    pcrel += 1;
                    format!(".Lpcrel_hi{pcrel}")
                };
                crate::asm_text::instruction(&self.t.machine, &s, &mut fresh).into_iter().map(|t| (n, t)).collect()
            })
            .collect();
        for (k_stmt, (n, stmt)) in stmts.iter().enumerate() {
            let n = *n;
            let mut line = stmt.trim();
            // Labels, perhaps followed by more on the line.
            while let Some((name, rest)) = line.split_once(':')
                && valid_symbol(name)
            {
                let name = name.to_string();
                let s = section_of(&mut obj, &mut by_kind, &mut cur, SectionKind::Text);
                let at = obj.sections[s].size();
                if labels.insert(name.clone(), (s, at)).is_some() {
                    return Err(err(n, format!("{name} is defined twice")));
                }
                if !temp(&name) {
                    defined.push(name);
                }
                line = rest.trim();
            }
            if line.is_empty() {
                continue;
            }
            if let Some(d) = line.strip_prefix('.').filter(|d| d.starts_with(|c: char| c.is_ascii_alphabetic())) {
                let (name, args) = d.split_once(char::is_whitespace).unwrap_or((d, ""));
                let args = args.trim();
                let list = || split_args(args);
                // (The section to go back to, kept before the switch.)
                match name {
                    "pushsection" => pushed.push((cur, discard)),
                    "popsection" => {
                        let (c, d) =
                            pushed.pop().ok_or_else(|| err(n, "a .popsection without a .pushsection".into()))?;
                        cur = c;
                        discard = d;
                        continue;
                    }
                    _ => {}
                }
                let mut switch = |kind: Option<(SectionKind, Option<String>)>| {
                    discard = kind.is_none();
                    if let Some((k, name)) = kind {
                        cur = Some(section_for(&mut obj, &mut by_kind, k, name));
                    }
                };
                match name {
                    "text" => switch(Some((SectionKind::Text, None))),
                    "data" => switch(Some((SectionKind::Data, None))),
                    "bss" => switch(Some((SectionKind::Bss, None))),
                    "section" | "pushsection" => {
                        let kind =
                            section_kind(args, syntax).ok_or_else(|| err(n, "a section not supported".into()))?;
                        switch(kind);
                    }
                    "globl" | "global" => {
                        globals.insert(args.to_string());
                    }
                    "weak" => {
                        globals.insert(args.to_string());
                        obj.weak.push(args.to_string());
                    }
                    // Which instructions may be compressed, and relaxed: only
                    // those written so are, and none are.
                    "option" => {}
                    // Another name for a label (plus an offset): an alias.
                    "set" | "equ" => {
                        let (name, value) =
                            args.split_once(',').ok_or_else(|| err(n, "a .set without a value".into()))?;
                        let (target, offset) =
                            parse_sym_offset(value).ok_or_else(|| err(n, "a .set of other than a label".into()))?;
                        sets.push((name.trim().to_string(), target, offset));
                    }
                    "p2align" | "align" | "balign" => {
                        let v = list()
                            .first()
                            .and_then(|a| parse_int(a))
                            .ok_or_else(|| err(n, "a bad alignment".into()))?;
                        let align = if name == "balign" { v as u64 } else { 1u64 << v };
                        let s = section_of(&mut obj, &mut by_kind, &mut cur, SectionKind::Text);
                        let sec = &mut obj.sections[s];
                        sec.align = sec.align.max(align);
                        let at = sec.size().next_multiple_of(align);
                        if sec.kind == SectionKind::Bss {
                            sec.bss_size = at;
                        } else if sec.kind == SectionKind::Text {
                            let fill = self.t.machine.code_fill(at as usize - sec.data.len());
                            sec.data.extend(fill);
                        } else {
                            sec.data.resize(at as usize, 0);
                        }
                    }
                    // Unwind rules (unwind.rs).
                    "cfi_startproc" => {
                        let s = section_of(&mut obj, &mut by_kind, &mut cur, SectionKind::Text);
                        let at = obj.sections[s].size();
                        let name =
                            labels.iter().filter(|(k, v)| **v == (s, at) && !temp(k)).map(|(k, _)| k.clone()).min();
                        open = Some(OpenFrame {
                            section: s,
                            start: at,
                            name,
                            rules: Vec::new(),
                            personality: None,
                            lsda: None,
                        });
                    }
                    "cfi_endproc" => {
                        let f = open.take().ok_or_else(|| err(n, ".cfi_endproc without .cfi_startproc".into()))?;
                        let len = obj.sections[f.section].size() - f.start;
                        frames.push((f, len));
                    }
                    "cfi_personality" | "cfi_lsda" => {
                        let Some(f) = open.as_mut() else {
                            return Err(err(n, "a .cfi directive outside .cfi_startproc".into()));
                        };
                        // Indirect, PC-relative and 4 bytes; PC-relative
                        // (Mach-O's 8 bytes, ELF's 4).
                        let want = match (name, syntax) {
                            ("cfi_personality", _) => 155,
                            (_, Syntax::MachO) => 16,
                            (_, Syntax::Elf) => 27,
                        };
                        match list().as_slice() {
                            [e, sym] if parse_int(e) == Some(want) => {
                                if name == "cfi_personality" {
                                    f.personality = Some(sym.clone());
                                } else {
                                    f.lsda = Some(sym.clone());
                                }
                            }
                            _ => return Err(err(n, format!("a .{name} not supported"))),
                        }
                    }
                    "cfi_def_cfa_offset" | "cfi_def_cfa" | "cfi_offset" => {
                        let Some(OpenFrame { section: s, start: at, rules, .. }) = open.as_mut() else {
                            return Err(err(n, "a .cfi directive outside .cfi_startproc".into()));
                        };
                        let a = list();
                        let reg = |x: &str| self.t.machine.lookup(x).ok_or_else(|| err(n, format!("no register {x}")));
                        let num = |x: &str| parse_int(x).ok_or_else(|| err(n, format!("a bad number {x}")));
                        let rule = match (name, a.as_slice()) {
                            ("cfi_def_cfa_offset", [o]) => crate::unwind::Cfi::DefCfaOffset(num(o)?),
                            ("cfi_def_cfa", [r, o]) => crate::unwind::Cfi::DefCfa(reg(r)?, num(o)?),
                            ("cfi_offset", [r, o]) => crate::unwind::Cfi::Offset(reg(r)?, num(o)?),
                            _ => return Err(err(n, "a bad .cfi directive".into())),
                        };
                        let here = obj.sections[*s].size() - *at;
                        rules.push((here, rule));
                    }
                    "type" => {
                        let a = list();
                        let kind = match a.get(1).map(|s| s.trim_start_matches(['%', '@'])) {
                            Some("function") => SymbolKind::Func,
                            Some("object") => SymbolKind::Object,
                            _ => return Err(err(n, "a symbol type not supported".into())),
                        };
                        types.insert(a[0].clone(), kind);
                    }
                    "size" => {
                        let a = list();
                        let (Some(sym), Some(expr)) = (a.first(), a.get(1)) else {
                            return Err(err(n, "a bad .size".into()));
                        };
                        let size = match expr.strip_prefix(".-") {
                            Some(start) if start == sym => {
                                let (s, at) = *labels.get(start).ok_or_else(|| err(n, format!("no label {start}")))?;
                                obj.sections[s].size() - at
                            }
                            _ => parse_int(expr).ok_or_else(|| err(n, "a size not supported".into()))? as u64,
                        };
                        sizes.insert(sym.clone(), size);
                    }
                    "float16" | "half_float" | "float" | "single" | "double" => {
                        let s = section_of(&mut obj, &mut by_kind, &mut cur, SectionKind::Data);
                        for v in list() {
                            // (`0h:7c00` is the bits themselves.)
                            if let Some(bits) = v.split_once(':').filter(|(p, _)| p.len() == 2 && p.starts_with('0'))
                                && let Ok(bits) = u64::from_str_radix(bits.1, 16)
                            {
                                let size = match name {
                                    "float16" | "half_float" => 2,
                                    "double" => 8,
                                    _ => 4,
                                };
                                obj.sections[s].data.extend_from_slice(&bits.to_le_bytes()[..size]);
                                continue;
                            }
                            let x: f64 = float_literal(&v).ok_or_else(|| err(n, format!("a bad number {v}")))?;
                            let bytes = match name {
                                "float16" | "half_float" => f16_bits(x).to_le_bytes().to_vec(),
                                "double" => x.to_bits().to_le_bytes().to_vec(),
                                _ => (x as f32).to_bits().to_le_bytes().to_vec(),
                            };
                            obj.sections[s].data.extend_from_slice(&bytes);
                        }
                    }
                    "fill" => {
                        let a = list();
                        let num = |k: usize, d: i64| a.get(k).map_or(Some(d), |x| parse_int(x));
                        let (Some(count), Some(size), Some(value)) = (num(0, 0), num(1, 1), num(2, 0)) else {
                            return Err(err(n, "a bad .fill".into()));
                        };
                        let size = size.clamp(0, 8) as usize;
                        let s = section_of(&mut obj, &mut by_kind, &mut cur, SectionKind::Data);
                        for _ in 0..count {
                            // (A value of 4 bytes or more is 4 bytes, then zeros.)
                            let v = if size > 4 { value as u32 as u64 } else { value as u64 };
                            obj.sections[s].data.extend_from_slice(&v.to_le_bytes()[..size]);
                        }
                    }
                    "byte" | "short" | "hword" | "half" | "2byte" | "long" | "word" | "4byte" | "quad" | "dword"
                    | "xword" | "8byte" => {
                        let size = match name {
                            "byte" => 1,
                            "short" | "hword" | "half" | "2byte" => 2,
                            "long" | "word" | "4byte" => 4,
                            _ => 8,
                        };
                        let s = section_of(&mut obj, &mut by_kind, &mut cur, SectionKind::Data);
                        for v in list() {
                            let sec = &mut obj.sections[s];
                            // (Zeros in .bss are room, as GNU assemblers take them.)
                            if sec.kind == SectionKind::Bss && parse_int(&v) == Some(0) {
                                sec.bss_size += size as u64;
                                continue;
                            }
                            if sec.kind == SectionKind::Bss {
                                return Err(err(n, "data in .bss".into()));
                            }
                            // A reference relative to a label at it: Mach-O's
                            // to a GOT entry, ELF's to a symbol.
                            let got = match syntax {
                                Syntax::MachO => v.split_once("@GOT-"),
                                Syntax::Elf => v.split_once('-').filter(|(a, b)| valid_symbol(a) && valid_symbol(b)),
                            }
                            .filter(|_| size == 4);
                            match parse_int(&v) {
                                Some(x) => sec.data.extend_from_slice(&(x as u64).to_le_bytes()[..size]),
                                None if let Some((symbol, label)) = got => {
                                    later.push(Later::Got {
                                        section: s,
                                        offset: sec.data.len() as u64,
                                        symbol: symbol.to_string(),
                                        label: label.to_string(),
                                        line: n,
                                    });
                                    sec.data.extend_from_slice(&[0; 4]);
                                }
                                None if size == 8 => {
                                    let (symbol, addend) =
                                        parse_sym_offset(&v).ok_or_else(|| err(n, format!("a bad value {v}")))?;
                                    later.push(Later::Data {
                                        section: s,
                                        offset: sec.data.len() as u64,
                                        symbol,
                                        addend,
                                    });
                                    sec.data.extend_from_slice(&[0; 8]);
                                }
                                None => return Err(err(n, format!("a value not supported here: {v}"))),
                            }
                        }
                    }
                    "ascii" | "asciz" | "string" => {
                        let mut bytes = unescape(args).ok_or_else(|| err(n, "a bad string".into()))?;
                        if name != "ascii" {
                            bytes.push(0);
                        }
                        let s = section_of(&mut obj, &mut by_kind, &mut cur, SectionKind::Data);
                        obj.sections[s].data.extend_from_slice(&bytes);
                    }
                    "space" | "zero" | "skip" => {
                        let a = list();
                        let count = a.first().and_then(|x| parse_int(x)).ok_or_else(|| err(n, "a bad size".into()))?;
                        let fill = a.get(1).and_then(|x| parse_int(x)).unwrap_or(0) as u8;
                        let s = section_of(&mut obj, &mut by_kind, &mut cur, SectionKind::Data);
                        let sec = &mut obj.sections[s];
                        if sec.kind == SectionKind::Bss {
                            sec.bss_size += count as u64;
                        } else {
                            sec.data.resize(sec.data.len() + count as usize, fill);
                        }
                    }
                    "build_version" => {
                        let a = list();
                        let num = |k: usize| a.get(k).and_then(|x| parse_int(x)).map(|v| v as u32);
                        if a.first().map(String::as_str) != Some("macos") {
                            return Err(err(n, "a platform other than macOS".into()));
                        }
                        min_os = (num(1).unwrap_or(11), num(2).unwrap_or(0), num(3).unwrap_or(0));
                    }
                    "subsections_via_symbols" => {}
                    // Read with the instruction after it, as one template
                    // (`.tlsdesccall {s}` then `blr {n}`).
                    "tlsdesccall" => held = Some(normalise(line)),
                    _ => return Err(err(n, format!("the directive .{name} is not supported"))),
                }
                continue;
            }
            if discard {
                return Err(err(n, "an instruction in a section that is not kept".into()));
            }
            let s = section_of(&mut obj, &mut by_kind, &mut cur, SectionKind::Text);
            let sec = &mut obj.sections[s];
            if sec.kind == SectionKind::Bss {
                return Err(err(n, "an instruction in .bss".into()));
            }
            let text = match held.take() {
                Some(h) => format!("{h}\n\t{}", normalise(line)),
                None => normalise(line),
            };
            later.push(Later::Instr { section: s, offset: sec.data.len() as u64, line: n, stmt: k_stmt, text });
            sec.data.resize(sec.data.len() + lens.get(&k_stmt).copied().unwrap_or(unit), 0);
        }

        // The second pass: instructions, and references to symbols in order.
        // Every instruction that cannot be read is reported.
        let mut pending = Vec::new();
        let mut errors = Vec::new();
        for l in later {
            match l {
                Later::Instr { section, offset, line, stmt, text } => {
                    // A branch to a global symbol is relocated, as another
                    // definition may take its place; to a local label, it
                    // is resolved here.
                    let find = |name: &str| {
                        labels.get(name).filter(|l| l.0 == section && !globals.contains(name)).map(|l| l.1)
                    };
                    // As written, or as the description's aliases write it
                    // the way its templates do.
                    let read = self.encode(&text, offset, syntax, &find).or_else(|e| {
                        let canonical =
                            self.t.machine.asm_aliases.iter().fold(text.clone(), |l, (a, b)| rewrite(&l, a, b));
                        if canonical == text { Err(e) } else { self.encode(&canonical, offset, syntax, &find) }
                    });
                    // Written in capitals (`CSEL X0, XZR, X9, NE`): its
                    // mnemonic, registers and conditions in lower case.
                    let read = read.or_else(|e| {
                        let lower = self.lower_names(&text);
                        if lower == text { Err(e) } else { self.encode(&lower, offset, syntax, &find) }
                    });
                    // A branch too far: as the description's longer form
                    // (`!target.asm-relax`).
                    let read = read.or_else(|e| self.relaxed(&text, offset, syntax, &find).ok_or(e));
                    let (i, bytes, sym) = match read {
                        Ok(x) => x,
                        // (The statement, not its line: a line may hold
                        // several, or a macro's.)
                        Err(e) => {
                            errors.push(format!("line {}: {e}: `{text}`", line + 1));
                            continue;
                        }
                    };
                    let room = lens.get(&stmt).copied().unwrap_or(unit);
                    if bytes.len() > room {
                        longer.insert(stmt, bytes.len());
                        continue;
                    }
                    if bytes.len() < room && !lens.contains_key(&stmt) {
                        longer.insert(stmt, bytes.len());
                        continue;
                    }
                    if bytes.len() < room {
                        errors.push(err(line, format!("{} bytes where {room} were kept", bytes.len())));
                        continue;
                    }
                    obj.sections[section].data[offset as usize..offset as usize + room].copy_from_slice(&bytes);
                    if let Some(sym) = sym {
                        let kind = self.t.instrs[i]
                            .reloc
                            .ok_or_else(|| err(line, "a symbol in an instruction without a \"reloc\"".into()))?;
                        let pc_rel = self.t.machine.relocs[kind.0 as usize].pc_relative();
                        pending.push(Pending {
                            section,
                            offset: offset + sym.at as u64,
                            kind,
                            symbol: sym.name,
                            addend: sym.addend + if pc_rel { sym.at as i64 } else { 0 },
                            instr: Some(i),
                        });
                    }
                }
                Later::Data { section, offset, symbol, addend } => {
                    let kind =
                        self.t.machine.data_reloc(8).ok_or("the description has no relocation for pointers in data")?;
                    pending.push(Pending {
                        section,
                        offset,
                        kind: RelocKind(kind as u16),
                        symbol,
                        addend,
                        instr: None,
                    });
                }
                Later::Got { section, offset, symbol, label, line } => {
                    // (ELF's label may be earlier in the section: a
                    // relative lookup table's start.)
                    let at = match labels.get(&label) {
                        Some(&(s, at)) if s == section && (at == offset || syntax == Syntax::Elf && at <= offset) => at,
                        _ => {
                            errors.push(err(line, format!("{label} is not where the reference is")));
                            continue;
                        }
                    };
                    let usage = if syntax == Syntax::MachO { RelocUse::GotPcRel32 } else { RelocUse::EhPcRel };
                    let kind = self
                        .t
                        .machine
                        .reloc_used(usage)
                        .ok_or_else(|| format!("the description has no relocation with use {usage:?}"))?;
                    if syntax == Syntax::MachO {
                        obj.sections[section].data[offset as usize..offset as usize + 4]
                            .copy_from_slice(&crate::unwind::macho_got_field(offset));
                    }
                    pending.push(Pending {
                        section,
                        offset,
                        kind: RelocKind(kind as u16),
                        symbol,
                        addend: (offset - at) as i64,
                        instr: None,
                    });
                }
            }
        }
        if !longer.is_empty() {
            return Ok(Err(longer));
        }
        if !errors.is_empty() {
            let n = errors.len();
            errors.truncate(self.max_errors);
            let more =
                if n > self.max_errors { format!("\n... and {} more", n - self.max_errors) } else { String::new() };
            return Err(format!("{n} lines could not be assembled:\n{}{more}", errors.join("\n")));
        }
        for name in defined {
            let (s, value) = labels[&name];
            let code = obj.sections[s].kind == SectionKind::Text;
            obj.symbols.push(Symbol {
                size: sizes.get(&name).copied().unwrap_or(0),
                // Without a .type, ELF's are of no type (as its assemblers
                // make them).
                kind: types.get(&name).copied().unwrap_or(match (syntax, code) {
                    (Syntax::Elf, _) => SymbolKind::NoType,
                    (Syntax::MachO, true) => SymbolKind::Func,
                    (Syntax::MachO, false) => SymbolKind::Object,
                }),
                global: globals.contains(&name),
                section: Some(s),
                value,
                name,
            });
        }
        let aliases: Vec<object::Alias> = sets
            .into_iter()
            .map(|(symbol, target, offset)| object::Alias { global: globals.contains(&symbol), symbol, target, offset })
            .collect();
        object::add_aliases(&mut obj, &aliases)?;
        // Temporary labels a relocation refers to are local symbols (the
        // `auipc` that `%pcrel_lo` names).
        for p in &pending {
            if let Some(&(s, value)) = labels.get(&p.symbol)
                && !obj.symbols.iter().any(|x| x.name == p.symbol)
            {
                obj.symbols.push(Symbol {
                    name: p.symbol.clone(),
                    section: Some(s),
                    value,
                    size: 0,
                    global: false,
                    kind: SymbolKind::NoType,
                });
            }
        }
        object::resolve(self.t, &mut obj, pending, syntax == Syntax::MachO)?;
        // The unwind tables, as the convention of this dialect has them.
        if !frames.is_empty() {
            let abi = self.t.abis.iter().find(|a| a.syntax == syntax).ok_or("no convention for this dialect")?;
            let mut list = Vec::new();
            for (OpenFrame { section, start, name, rules, personality, lsda }, len) in frames {
                let lsda = match lsda {
                    Some(l) => Some(*labels.get(&l).ok_or_else(|| format!("no label {l}, the LSDA of {name:?}"))?),
                    None => None,
                };
                let symbol = match name.and_then(|n| obj.symbols.iter().position(|s| s.name == n)) {
                    Some(k) => k,
                    None => {
                        obj.symbols.push(Symbol {
                            name: format!("ltmp_fn{start}"),
                            section: Some(section),
                            value: start,
                            size: 0,
                            global: false,
                            kind: SymbolKind::NoType,
                        });
                        obj.symbols.len() - 1
                    }
                };
                list.push(crate::unwind::Frame { section, start, len, symbol, rules, personality, lsda });
            }
            let p = crate::unwind::Params { syntax, data_align: self.t.machine.data_align(syntax), ra: abi.lr };
            crate::unwind::add_eh_frame(&mut obj, &self.t.machine, &p, &list)?;
        }
        Ok(Ok(Assembled { object: obj, syntax, min_os }))
    }
}

/// The section a `.section` directive names: `None` for one that is not
/// kept (`.note.GNU-stack`, which the ELF writer adds itself).
fn section_kind(args: &str, syntax: Syntax) -> Option<Option<(SectionKind, Option<String>)>> {
    let parts: Vec<&str> = args.split(',').map(str::trim).collect();
    let name = match syntax {
        Syntax::MachO => format!("{},{}", parts.first()?, parts.get(1)?),
        Syntax::Elf => parts.first()?.to_string(),
    };
    Some(Some((
        match name.as_str() {
            "__TEXT,__text" | ".text" => SectionKind::Text,
            "__DATA,__data" | ".data" => SectionKind::Data,
            "__DATA,__bss" | ".bss" => SectionKind::Bss,
            "__TEXT,__const" | ".rodata" => SectionKind::ReadOnly,
            "__DATA,__const" | ".data.rel.ro" => SectionKind::RelRo,
            "__TEXT,__gcc_except_tab" | ".gcc_except_table" => SectionKind::ExceptTable,
            "__DATA,__thread_vars" => SectionKind::ThreadVars,
            "__DATA,__thread_data" | ".tdata" => SectionKind::ThreadData,
            ".note.GNU-stack" => return Some(None),
            _ => return Some(Some(named_section_kind(&parts, syntax))),
        },
        None,
    )))
}

/// A section of a name of its own: what it holds, by its segment, type
/// and attributes (Mach-O) or flags and type (ELF), and its name as the
/// object writers take it (`asm::named_section`).
fn named_section_kind(parts: &[&str], syntax: Syntax) -> (SectionKind, Option<String>) {
    let kind = match syntax {
        Syntax::MachO => {
            let attrs = parts.get(3).copied().unwrap_or("");
            if parts.get(2) == Some(&"zerofill") {
                SectionKind::Bss
            } else if attrs.contains("pure_instructions") {
                SectionKind::Text
            } else if parts[0] == "__TEXT" {
                SectionKind::ReadOnly
            } else {
                SectionKind::Data
            }
        }
        Syntax::Elf => {
            let flags = parts.get(1).copied().unwrap_or("").trim_matches('"');
            // Without flags, those of its name, as GNU assemblers take them
            // (`.text.init` is code).
            let named = |p: &str| parts[0] == p || parts[0].starts_with(&format!("{p}."));
            if parts.get(1).is_none() && named(".text") {
                SectionKind::Text
            } else if parts.get(1).is_none() && (named(".data") || named(".sdata")) {
                SectionKind::Data
            } else if parts.get(1).is_none() && (named(".bss") || named(".sbss")) {
                SectionKind::Bss
            } else if parts.get(2).is_some_and(|t| t.ends_with("nobits")) {
                SectionKind::Bss
            } else if flags.contains('x') {
                SectionKind::Text
            } else if flags.contains('w') {
                SectionKind::Data
            } else {
                SectionKind::ReadOnly
            }
        }
    };
    let name = match syntax {
        Syntax::MachO => parts.join(","),
        Syntax::Elf => parts[0].to_string(),
    };
    (kind, Some(crate::asm::named_section(&name, syntax)))
}

/// The section of this kind (and name), made the first time it is used.
fn section_for(
    obj: &mut Object,
    by_kind: &mut HashMap<(SectionKind, Option<String>), usize>,
    kind: SectionKind,
    name: Option<String>,
) -> usize {
    *by_kind.entry((kind, name.clone())).or_insert_with(|| {
        obj.sections.push(Section { kind, name, align: 1, data: Vec::new(), bss_size: 0, relocs: Vec::new() });
        obj.sections.len() - 1
    })
}

/// The current section (`default`'s, if none was named yet).
fn section_of(
    obj: &mut Object,
    by_kind: &mut HashMap<(SectionKind, Option<String>), usize>,
    cur: &mut Option<usize>,
    default: SectionKind,
) -> usize {
    let s = match *cur {
        Some(s) => s,
        None => section_for(obj, by_kind, default, None),
    };
    *cur = Some(s);
    s
}

/// `line` with each piece written as `from` written as `to` instead.
/// In the patterns, `{i}` stands for an integer (`12`, `-3`, `0x1f`) and
/// any other `{x}` for a token (a register: `x0`); `to` may use them.
pub fn rewrite(line: &str, from: &str, to: &str) -> String {
    // The pattern as literals and holes.
    let mut parts: Vec<(bool, &str)> = Vec::new();
    let mut rest = from;
    while let Some(open) = rest.find('{') {
        let close = rest[open..].find('}').map_or(rest.len(), |c| open + c);
        parts.push((false, &rest[..open]));
        parts.push((true, &rest[open + 1..close]));
        rest = &rest[(close + 1).min(rest.len())..];
    }
    parts.push((false, rest));
    let token = |s: &str, int: bool| -> usize {
        let b = s.as_bytes();
        let mut k = 0;
        if int {
            if b.first() == Some(&b'-') {
                k = 1;
            }
            let start = k;
            if s[k..].starts_with("0x") {
                k += 2;
                while k < b.len() && b[k].is_ascii_hexdigit() {
                    k += 1;
                }
            } else {
                while k < b.len() && b[k].is_ascii_digit() {
                    k += 1;
                }
            }
            if k == start { 0 } else { k }
        } else {
            while k < b.len() && (b[k].is_ascii_alphanumeric() || b[k] == b'_' || b[k] == b'.') {
                k += 1;
            }
            k
        }
    };
    let matches_at = |at: usize| -> Option<(usize, HashMap<&str, &str>)> {
        let mut pos = at;
        let mut env = HashMap::new();
        for &(hole, p) in &parts {
            if hole {
                let n = token(&line[pos..], p == "i");
                if n == 0 {
                    return None;
                }
                env.insert(p, &line[pos..pos + n]);
                pos += n;
            } else {
                if !line[pos..].starts_with(p) {
                    return None;
                }
                pos += p.len();
            }
        }
        Some((pos, env))
    };
    let mut out = String::new();
    let mut k = 0;
    while k < line.len() {
        if let Some((end, env)) = matches_at(k) {
            let mut filled = to.to_string();
            for (name, v) in env {
                filled = filled.replace(&format!("{{{name}}}"), v);
            }
            out.push_str(&filled);
            k = end.max(k + 1);
            continue;
        }
        let c = line[k..].chars().next().unwrap();
        out.push(c);
        k += c.len_utf8();
    }
    out
}

/// The line without a comment: from `comment` (`//`, `#`), or for
/// Mach-O `;`, outside strings.
fn strip_comment<'l>(line: &'l str, comment: &str, syntax: Syntax) -> &'l str {
    let mut in_str = false;
    let mut escaped = false;
    let b = line.as_bytes();
    for (k, &c) in b.iter().enumerate() {
        if in_str {
            match c {
                _ if escaped => escaped = false,
                b'\\' => escaped = true,
                b'"' => in_str = false,
                _ => {}
            }
            continue;
        }
        match c {
            b'"' => in_str = true,
            b';' if syntax == Syntax::MachO => return &line[..k],
            _ if line[k..].starts_with(comment) => return &line[..k],
            _ => {}
        }
    }
    line
}

/// `line` without its `/* ... */` comments (which inline assembly writes:
/// `/*${0}*/`, an empty template naming its operand), each a space;
/// `in_block` carries one that goes on past the line's end.
fn strip_block_comments(line: &str, in_block: &mut bool) -> String {
    if !*in_block && !line.contains("/*") {
        return line.to_string();
    }
    let mut out = String::new();
    let mut rest = line;
    let mut in_str = false;
    loop {
        if *in_block {
            match rest.find("*/") {
                Some(k) => {
                    rest = &rest[k + 2..];
                    *in_block = false;
                    out.push(' ');
                }
                None => return out,
            }
        }
        // The next opening outside a string.
        let mut open = None;
        let mut escaped = false;
        for (k, c) in rest.char_indices() {
            if in_str {
                match c {
                    _ if escaped => escaped = false,
                    '\\' => escaped = true,
                    '"' => in_str = false,
                    _ => {}
                }
                continue;
            }
            if c == '"' {
                in_str = true;
            } else if rest[k..].starts_with("/*") {
                open = Some(k);
                break;
            }
        }
        match open {
            Some(k) => {
                out.push_str(&rest[..k]);
                rest = &rest[k + 2..];
                *in_block = true;
            }
            None => {
                out.push_str(rest);
                return out;
            }
        }
    }
}

/// The statements of the text, each with its line's number: lines without
/// comments, split at `;` (for ELF; outside strings), with numeric local
/// labels (`1:`, referred to as `1b` and `1f`, as inline assembly writes
/// them) renamed to temporary labels of their own, after macros, `.rept`
/// and `.if` are done (`asm_text::expand`), so that each use of a macro
/// has labels of its own.
fn statements(lines: &[&str], comment: &str, syntax: Syntax) -> Result<Vec<(usize, String)>, (usize, String)> {
    let mut out: Vec<(usize, String)> = Vec::new();
    let mut in_block = false;
    for (n, raw) in lines.iter().enumerate() {
        let unblocked = strip_block_comments(raw, &mut in_block);
        let line = strip_comment(&unblocked, comment, syntax);
        // A line starting `#` (but a directive's `#` in an operand) is a
        // comment, as LLVM's assemblers take it (inline assembly's `# x0`).
        let line = if line.trim_start().starts_with('#') { "" } else { line };
        if syntax == Syntax::MachO || !line.contains(';') {
            out.push((n, line.to_string()));
            continue;
        }
        let (mut in_str, mut escaped) = (false, false);
        let mut start = 0;
        for (k, c) in line.char_indices() {
            match c {
                _ if escaped => escaped = false,
                '\\' if in_str => escaped = true,
                '"' => in_str = !in_str,
                ';' if !in_str => {
                    out.push((n, line[start..k].to_string()));
                    start = k + 1;
                }
                _ => {}
            }
        }
        out.push((n, line[start..].to_string()));
    }
    let mut out = crate::asm_text::expand(out)?;
    // Numeric labels: the k-th definition of `N` is `.Ln.N.k`.
    let temp = if syntax == Syntax::MachO { "Ln." } else { ".Ln." };
    let numeric = |s: &str| !s.is_empty() && s.bytes().all(|c| c.is_ascii_digit());
    if !out.iter().any(|(_, l)| l.trim_start().split_once(':').is_some_and(|(n, _)| numeric(n))) {
        return Ok(out);
    }
    let mut defined: HashMap<String, usize> = HashMap::new();
    for (_, l) in &mut out {
        let mut text = l.trim_start().to_string();
        let mut head = String::new();
        while let Some((name, rest)) = text.split_once(':')
            && numeric(name)
        {
            let k = defined.entry(name.to_string()).or_default();
            *k += 1;
            head.push_str(&format!("{temp}{name}.{k}: "));
            text = rest.trim_start().to_string();
        }
        // References: `1b` is the last definition of 1, `1f` the next.
        let mut body = String::new();
        let chars: Vec<char> = text.chars().collect();
        let mut i = 0;
        while i < chars.len() {
            let boundary = i == 0 || !(chars[i - 1].is_alphanumeric() || chars[i - 1] == '_' || chars[i - 1] == '.');
            if boundary && chars[i].is_ascii_digit() {
                let mut j = i;
                while j < chars.len() && chars[j].is_ascii_digit() {
                    j += 1;
                }
                let after = chars.get(j + 1).is_none_or(|c| !(c.is_alphanumeric() || *c == '_'));
                if let Some(&dir) = chars.get(j).filter(|c| matches!(c, 'b' | 'f'))
                    && after
                {
                    let name: String = chars[i..j].iter().collect();
                    let k = defined.get(&name).copied().unwrap_or(0);
                    let k = if dir == 'b' { k } else { k + 1 };
                    body.push_str(&format!("{temp}{name}.{k}"));
                    i = j + 1;
                    continue;
                }
            }
            body.push(chars[i]);
            i += 1;
        }
        *l = head + &body;
    }
    Ok(out)
}

/// An instruction line as the printer writes it: the mnemonic, one space,
/// then the operands.
fn normalise(line: &str) -> String {
    match line.split_once(char::is_whitespace) {
        Some((m, rest)) => format!("{m} {}", rest.trim()),
        None => line.to_string(),
    }
}

/// A directive's arguments, split at commas outside strings.
fn split_args(args: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut in_str = false;
    for c in args.chars() {
        match c {
            '"' => {
                in_str = !in_str;
                cur.push(c);
            }
            ',' if !in_str => out.push(std::mem::take(&mut cur).trim().to_string()),
            _ => cur.push(c),
        }
    }
    if !cur.trim().is_empty() || !out.is_empty() {
        out.push(cur.trim().to_string());
    }
    out
}

/// A floating-point number as data directives write them: `1.5`, `-2e3`,
/// `0x3f800000` (its bits are not meant: the number), `inf`, `nan`.
fn float_literal(s: &str) -> Option<f64> {
    let s = s.trim();
    s.parse::<f64>().ok().or_else(|| parse_int(s).map(|v| v as f64))
}

/// A number's nearest half-precision bits (ties to even), rounded once,
/// from the double.
fn f16_bits(x: f64) -> u16 {
    let sign = if x.is_sign_negative() { 0x8000 } else { 0 };
    if x.is_nan() {
        return sign | 0x7e00;
    }
    let bits = x.abs().to_bits();
    let exp = ((bits >> 52) & 0x7ff) as i32 - 1023;
    if (bits >> 52) & 0x7ff == 0 {
        return sign; // (Below any half: zero, or a double's subnormal.)
    }
    // Its 53 bits, from which a half keeps 11 (10 below a subnormal's
    // exponent of -14, fewer the smaller it is).
    let mant = (bits & ((1 << 52) - 1)) | (1 << 52);
    let shift = 42 + (-14 - exp).max(0) as u32;
    let round = |m: u64, s: u32| -> u64 {
        if s >= 64 {
            return 0;
        }
        let (kept, rest, half) = (m >> s, m & ((1 << s) - 1), 1u64 << (s - 1));
        kept + (rest > half || (rest == half && kept & 1 == 1)) as u64
    };
    let kept = round(mant, shift);
    let v = if exp < -14 {
        kept // A subnormal (or, rounded up, the least normal).
    } else {
        // (Rounding up to 2^11 is the next exponent's 1.)
        let e = (exp + 15) as u64 + (kept >> 11);
        (e << 10) + (kept >> (kept >> 11) & 0x3ff)
    };
    if v >= 0x7c00 { sign | 0x7c00 } else { sign | v as u16 }
}

/// A branch target written relative to the instruction: `.`, `.+8`, `.-4`.
fn here_relative(s: &str) -> Option<i64> {
    let rest = s.trim().strip_prefix('.')?;
    if rest.is_empty() {
        return Some(0);
    }
    if !rest.starts_with(['+', '-']) {
        return None;
    }
    parse_int(rest)
}

/// `sym`, `sym+8` or `sym-8`.
fn parse_sym_offset(s: &str) -> Option<(String, i64)> {
    let s = s.trim();
    let split = s.char_indices().skip(1).find(|&(_, c)| c == '+' || c == '-').map(|(k, _)| k);
    let (name, off) = match split {
        Some(k) => (&s[..k], parse_int(&s[k..])?),
        None => (s, 0),
    };
    valid_symbol(name).then(|| (name.to_string(), off))
}

/// The bytes of a quoted string, with `\\`, `\"`, `\n`, `\t`, `\r`, `\0`
/// and octal escapes `\ooo`.
fn unescape(s: &str) -> Option<Vec<u8>> {
    let inner = s.trim().strip_prefix('"')?.strip_suffix('"')?;
    let b = inner.as_bytes();
    let mut out = Vec::new();
    let mut k = 0;
    while k < b.len() {
        if b[k] != b'\\' {
            out.push(b[k]);
            k += 1;
            continue;
        }
        k += 1;
        let c = *b.get(k)?;
        match c {
            b'0'..=b'7' => {
                let mut v = 0u32;
                let mut n = 0;
                while n < 3 && k < b.len() && (b'0'..=b'7').contains(&b[k]) {
                    v = v * 8 + (b[k] - b'0') as u32;
                    k += 1;
                    n += 1;
                }
                out.push(v as u8);
                continue;
            }
            b'n' => out.push(b'\n'),
            b't' => out.push(b'\t'),
            b'r' => out.push(b'\r'),
            b'b' => out.push(8),
            b'f' => out.push(12),
            _ => out.push(c),
        }
        k += 1;
    }
    Some(out)
}
