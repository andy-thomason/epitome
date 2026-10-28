//! Checks of a machine description, for `epitome-llc --check-target`:
//! what someone writing a description needs to know before trusting it.
//!
//! - **structure**: the roles, copies, relocations, conventions and CPUs
//!   the code generator needs are there and agree with each other;
//! - **encodings**: every instruction encodes and decodes back to itself,
//!   over a spread of operands, and the disassembler reads each word as an
//!   instruction that encodes it again;
//! - **assembly**: each instruction's assembly, as the code generator
//!   writes it, is read back by the assembler as the same word;
//! - **recipes**: each recipe's instructions, by their bodies, compute
//!   what the IR they replace computes, on edge values;
//! - **smoke**: a small C program (`smoke.c`, as IR in `smoke.ll`)
//!   compiles under every convention and CPU, its object file is what the
//!   assembler makes of its assembly, and every word of its code
//!   disassembles; and which of its functions need each optional feature.
//!
//! None of this shows that an instruction's body says what the hardware
//! does: only running code can (see `tests/riscv.rs`).

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;

use crate::abi::Syntax;
use crate::assembler::Assembler;
use crate::enc::Env;
use crate::machine::Holds;
use crate::mir::{self, Operand, Reg};
use crate::object::SectionKind;
use crate::target::{Class, Kind, Target};
use crate::{Options, compile};

/// The smoke program: clang's IR for `smoke.c`, without a triple.
pub const SMOKE: &str = include_str!("smoke.ll");

/// The roles the code generator cannot do without.
const NEEDED_ROLES: &[(&str, &str)] = &[
    ("load.x", "reloading spilled values, and arguments on the stack"),
    ("store.x", "spilling values, saving registers"),
    ("addi", "adjusting the stack pointer, addresses of stack slots"),
    ("add", "addresses of stack slots out of an addi's reach"),
    ("call", "calls"),
    ("jump", "branches"),
    ("ret", "returning"),
    ("trap", "unreachable"),
];

/// One check's results.
#[derive(Default, Debug)]
pub struct Section {
    pub name: &'static str,
    /// How many things were checked, and what they are.
    pub checked: usize,
    pub what: &'static str,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    /// Facts worth knowing that are not problems.
    pub notes: Vec<String>,
}

#[derive(Default, Debug)]
pub struct Report {
    pub summary: String,
    pub sections: Vec<Section>,
}

impl Report {
    pub fn errors(&self) -> usize {
        self.sections.iter().map(|s| s.errors.len()).sum()
    }

    /// The report as text; with `all`, every message, else the first few
    /// of each section's.
    pub fn render(&self, all: bool) -> String {
        let mut out = format!("{}\n", self.summary);
        for s in &self.sections {
            let status = if !s.errors.is_empty() {
                "FAIL"
            } else if !s.warnings.is_empty() {
                "warn"
            } else {
                "ok"
            };
            let _ = writeln!(out, "{status:<5}{:<11}{} {}", s.name, s.checked, s.what);
            let limit = if all { usize::MAX } else { 10 };
            for (kind, list) in [("error", &s.errors), ("warning", &s.warnings), ("note", &s.notes)] {
                for m in list.iter().take(limit) {
                    let _ = writeln!(out, "       {kind}: {m}");
                }
                if list.len() > limit {
                    let _ = writeln!(out, "       ... and {} more {kind}s (--verbose shows them)", list.len() - limit);
                }
            }
        }
        let (e, w): (usize, usize) = (self.errors(), self.sections.iter().map(|s| s.warnings.len()).sum());
        let _ = writeln!(out, "{e} errors, {w} warnings");
        out
    }
}

/// Runs every check on `t`.
pub fn check(t: &Target) -> Report {
    let m = &t.machine;
    // Encodings read lazily are parsed here first: one that does not parse
    // stops the rest.
    let unparsed: Vec<String> = t.instrs.iter().filter_map(|i| i.enc.parsed().err()).collect();
    if !unparsed.is_empty() {
        let s = Section { name: "encodings", what: "encodings parsed", errors: unparsed, ..Default::default() };
        return Report { summary: format!("{} instructions", t.instrs.len()), sections: vec![s] };
    }
    let recipes = t.instrs.iter().filter(|i| i.enc.is_empty()).count();
    let summary = format!(
        "{} instructions, {} condition forms, {} patterns, {} convention{} ({}), {} CPU{}, {} relocations",
        t.instrs.len() - recipes.min(t.instrs.len()),
        t.conds.len(),
        t.patterns.len(),
        t.abis.len(),
        if t.abis.len() == 1 { "" } else { "s" },
        t.abis.iter().map(|a| a.name.as_str()).collect::<Vec<_>>().join(", "),
        m.cpus.len(),
        if m.cpus.len() == 1 { "" } else { "s" },
        m.relocs.len(),
    );
    let structure = structure(t);
    let smoke = if structure.errors.is_empty() {
        smoke(t)
    } else {
        Section {
            name: "smoke",
            what: "compilations of the smoke program",
            notes: vec!["not run: the structure is not complete".into()],
            ..Default::default()
        }
    };
    Report { summary, sections: vec![structure, encodings(t), assembly(t), recipes_check(t), smoke] }
}

// ---------------------------------------------------------------------------
// Structure

fn structure(t: &Target) -> Section {
    let mut s = Section { name: "structure", what: "requirements", ..Default::default() };
    let m = &t.machine;
    let need = |ok: bool, msg: String, s: &mut Section| {
        s.checked += 1;
        if !ok {
            s.errors.push(msg);
        }
    };
    for (role, why) in NEEDED_ROLES {
        need(t.role(role).is_some(), format!("no instruction has role {role} (for {why})"), &mut s);
    }
    // A global's address: in one instruction (addr), or two.
    if t.role("addr").is_none() {
        for role in ["addr.hi", "addr.lo"] {
            need(
                t.role(role).is_some(),
                format!("no instruction has role {role} (or addr, for the addresses of globals)"),
                &mut s,
            );
        }
    }
    if m.external_via_got && t.role("got").is_none() {
        for role in ["got.hi", "got.lo"] {
            need(
                t.role(role).is_some(),
                format!("external-via-got is yes, but no instruction has role {role} (or got)"),
                &mut s,
            );
        }
    }
    if t.role("callr").is_none() {
        s.warnings.push("no instruction has role callr: calls through pointers cannot be compiled".into());
    }

    // The copies register allocation makes: within a file, for each class
    // values are put in, by the copy of it or of a wider class of its file;
    // and from the integer registers to each floating-point class, for
    // constants and bit-casts (role move.<to>.<from>).
    let used: BTreeSet<Class> = t
        .instrs
        .iter()
        .flat_map(|i| i.operands.iter())
        .filter_map(|o| if let Kind::Reg(c) = o.kind { Some(c) } else { None })
        .collect();
    let int_file = m.value_class(Holds::Int, 64).map(|c| m.def(c).file);
    need(int_file.is_some(), "no register class holds 64-bit integers (\"holds\"=\"int\")".into(), &mut s);
    for (k, c) in m.classes.iter().enumerate() {
        let class = Class(k as u8);
        if c.holds.is_none() || c.sp || !used.contains(&class) {
            continue;
        }
        let copied = t.copies.keys().any(|&x| m.def(x).file == c.file && m.def(x).bits >= c.bits);
        need(copied, format!("no \"copy\" instruction copies class {} (or a wider class of its file)", c.name), &mut s);
        if c.holds == Some(Holds::Float)
            && let Some(f) = int_file
            && let Some(i) = m.scalar_in(f, c.bits)
        {
            let role = format!("move.{}.{}", c.name, m.def(i).name);
            need(t.role(&role).is_some(), format!("no instruction has role {role} (for {} constants)", c.name), &mut s);
        }
    }

    // Symbols and relocations.
    let syntaxes: BTreeSet<&'static str> = t
        .abis
        .iter()
        .map(|a| match a.syntax {
            Syntax::Elf => "elf",
            Syntax::MachO => "macho",
        })
        .collect();
    for ins in &t.instrs {
        if ins.operands.iter().any(|o| o.kind == Kind::Sym) {
            need(ins.reloc.is_some(), format!("{}: a \"sym\" operand, but no \"reloc\"", ins.name), &mut s);
        }
        if let Some(k) = ins.reloc {
            let r = &m.relocs[k.0 as usize];
            for f in &syntaxes {
                let only = ins.format.map(|x| if x == Syntax::Elf { "elf" } else { "macho" });
                if only.is_some_and(|o| o != *f) {
                    continue;
                }
                let has = if *f == "elf" { r.elf.is_some() } else { r.macho.is_some() };
                need(
                    has,
                    format!("{}: relocation {} has no {f} number, and a convention writes {f}", ins.name, r.name),
                    &mut s,
                );
            }
        }
    }
    if syntaxes.contains("elf") {
        need(m.elf_machine != 0, "a convention writes ELF, but elf-machine is not given".into(), &mut s);
    }
    if syntaxes.contains("macho") {
        need(m.macho_cpu.is_some(), "a convention writes Mach-O, but macho-cpu is not given".into(), &mut s);
    }
    if !t.machine.relocs.iter().any(|r| matches!(r.usage, crate::machine::RelocUse::Data(8))) {
        s.warnings.push("no relocation has use data8: pointers in data cannot be written".into());
    }

    // Inline assembly: register constraints are written as classes of
    // their own files.
    for (code, c) in &m.asm_constraints {
        if let crate::machine::AsmCode::Reg { file, print: Some(p) } = c {
            need(
                m.def(*p).file == *file,
                format!(
                    "asm constraint {code} is a register of {}, but is written as {}",
                    m.files[*file].name,
                    m.def(*p).name
                ),
                &mut s,
            );
        }
    }
    if m.asm_constraints.is_empty() {
        s.warnings.push("no !target.asm-constraints: inline assembly with operands cannot be compiled".into());
    }

    // Conventions.
    need(!t.abis.is_empty(), "no calling convention (!abi.<name>)".into(), &mut s);
    for a in &t.abis {
        let name = |r: &u8| m.lookup_name(*r);
        let names = |v: &[u8]| v.iter().map(name).collect::<Vec<_>>().join(", ");
        let both = |x: &[u8], y: &[u8]| x.iter().copied().filter(|r| y.contains(r)).collect::<Vec<u8>>();
        let checks: [(&str, Vec<u8>); 6] = [
            ("are both saved and clobbered", both(&a.saved, &a.clobbers)),
            (
                "are arguments but not clobbered",
                a.args.iter().copied().filter(|r| !a.clobbers.contains(r) && !a.saved.contains(r)).collect(),
            ),
            ("are scratch and arguments", both(&a.scratch, &a.args)),
            ("are reserved and saved", both(&a.reserved, &a.saved)),
            ("are scratch and saved", both(&a.scratch, &a.saved)),
            (
                "are the stack pointer and something else",
                [&a.args[..], &a.saved, &a.scratch, &a.clobbers].concat().into_iter().filter(|r| *r == m.sp).collect(),
            ),
        ];
        for (what, regs) in checks {
            need(regs.is_empty(), format!("convention {}: {} {what}", a.name, names(&regs)), &mut s);
        }
        if a.scratch.len() < 2 {
            s.warnings.push(format!("convention {}: fewer than two scratch registers for spill code", a.name));
        }
    }

    // CPUs and features.
    if !m.cpus.is_empty() {
        let d = m.default_cpu.clone().unwrap_or_default();
        need(m.cpus.iter().any(|(c, _)| *c == d), format!("the default CPU ({d:?}) is not among !target.cpus"), &mut s);
    }
    let on_some_cpu: BTreeSet<&String> = m.cpus.iter().flat_map(|(_, f)| f.iter()).collect();
    let used: BTreeSet<&String> = t.instrs.iter().flat_map(|i| i.features.iter()).collect();
    for f in used.iter().filter(|f| !m.cpus.is_empty() && !on_some_cpu.contains(*f)) {
        s.warnings.push(format!("feature {f} is needed by entries, but no CPU has it (only --mattr=+{f} turns it on)"));
    }
    s
}

// ---------------------------------------------------------------------------
// Encodings

const IMMS: &[i64] = &[
    0,
    1,
    2,
    3,
    4,
    7,
    8,
    12,
    16,
    31,
    32,
    48,
    63,
    255,
    256,
    2047,
    4095,
    4096,
    65535,
    -1,
    -8,
    -256,
    -2048,
    0xff00,
    0x5555_5555,
];

/// Operand values for instruction `i`: a sample for each of the sample
/// immediates that fits. Registers are spread over their files; where that
/// gives no sample at all, they are chosen otherwise (RISC-V's compressed
/// forms take only x8-x15, and Zcmp's s0-s1 and s2-s7).
fn samples(t: &Target, i: usize) -> impl Iterator<Item = Env> + '_ {
    let spread = (0..64).find(|&s| (0..IMMS.len()).any(|k| sample(t, i, k, s).is_some())).unwrap_or(0);
    (0..IMMS.len()).filter_map(move |k| sample(t, i, k, spread))
}

/// Operand values for instruction `i`, sample `k`, if its immediates fit;
/// `spread` chooses the registers (0 is the usual choice).
fn sample(t: &Target, i: usize, k: usize, spread: usize) -> Option<Env> {
    let ins = &t.instrs[i];
    let imm = IMMS[k % IMMS.len()];
    let (step, shift) = if spread < 32 { (7, spread) } else { (1, spread - 32) };
    let mut env = Env::new();
    for (j, o) in ins.operands.iter().enumerate() {
        let v = match o.kind {
            Kind::Reg(c) => {
                let n = t.machine.def(c).names.len().max(1);
                ((j * step + k * 3 + shift) % n.min(31)) as i64
            }
            Kind::Imm => imm,
            Kind::Label => 0,
            Kind::Sym => 0,
            Kind::Cond => t.conds.get(k % t.conds.len().max(1)).map_or(0, |c| c.value),
            // A spread of its names, the operands of one entry differing.
            Kind::Enum(e) => {
                let names = &t.machine.enums[e as usize].names;
                names.get((j * 7 + k * 13) % names.len().max(1)).map_or(0, |&(_, v)| v)
            }
        };
        env.insert(o.name.clone(), v);
    }
    for o in ins.operands.iter().filter(|o| o.tied) {
        env.insert(o.name.clone(), env["d"]);
    }
    // The values must fit, registers as immediates do (some take only
    // some registers: an even one, for a pair; v0-v15; a mask other than
    // k0): they encode, and decode as themselves.
    let (back, _) = ins.decode(&ins.encode(&env)?)?;
    let same = ins.operands.iter().all(|o| match o.kind {
        Kind::Imm => back[&o.name] as u64 & mask(o.bits) == env[&o.name] as u64 & mask(o.bits),
        Kind::Reg(_) => back.get(&o.name) == env.get(&o.name),
        _ => true,
    });
    same.then_some(env)
}

fn mask(bits: u32) -> u64 {
    if bits >= 64 { u64::MAX } else { (1u64 << bits) - 1 }
}

fn encodings(t: &Target) -> Section {
    let mut s = Section { name: "encodings", what: "words encoded, decoded and disassembled", ..Default::default() };
    // Pairs of entries the disassembler confuses, reported once.
    let mut confused: BTreeMap<(usize, usize), Vec<u8>> = BTreeMap::new();
    let mut ambiguous: BTreeMap<(usize, usize), Vec<u8>> = BTreeMap::new();
    for (i, ins) in t.instrs.iter().enumerate() {
        if ins.enc.is_empty() {
            continue; // A recipe, or a pseudo-instruction.
        }
        let (mut fitted, mut encoded) = (0, 0);
        for env in samples(t, i) {
            fitted += 1;
            // A register beyond a narrow field (v0-v15 only) does not
            // encode: another sample may.
            let Some(word) = ins.encode(&env) else { continue };
            encoded += 1;
            s.checked += 1;
            // Decoding as itself gives back the operands.
            let Some((back, _)) = ins.decode(&word) else {
                s.errors.push(format!("{}: {} does not decode as itself", ins.name, hex(&word)));
                break;
            };
            let mut failed = false;
            for o in ins.operands.iter().filter(|o| o.kind != Kind::Sym && o.kind != Kind::Label) {
                let mk = mask(o.bits);
                if back[&o.name] as u64 & mk != env[&o.name] as u64 & mk {
                    s.errors.push(format!(
                        "{}: operand {} = {} encodes in {} but decodes as {}",
                        ins.name,
                        o.name,
                        env[&o.name],
                        hex(&word),
                        back[&o.name]
                    ));
                    failed = true;
                }
            }
            if failed {
                break;
            }
            // The disassembler reads it as something that encodes it again.
            if ins.decode {
                match mir::decode(t, &word) {
                    None => {
                        s.errors.push(format!("{}: the disassembler does not read {}", ins.name, hex(&word)));
                        failed = true;
                    }
                    Some((j, env2, _)) => {
                        let again = t.instrs[j].encode(&env2);
                        if again.as_ref() != Some(&word) {
                            confused.entry((i, j)).or_insert(word.clone());
                        }
                        // Read as another entry: fine for an alias, which
                        // fixes more bits (cmp, of subs); otherwise the two
                        // cannot be told apart.
                        if j != i && t.instrs[j].enc.fixed_bits() <= ins.enc.fixed_bits() {
                            ambiguous.entry((i.min(j), i.max(j))).or_insert(word.clone());
                        }
                    }
                }
            }
            if failed {
                break;
            }
        }
        if fitted == 0 {
            s.warnings.push(format!("{}: none of the sample immediates fit, so it was not checked", ins.name));
        } else if encoded == 0 {
            s.errors.push(format!("{}: none of its samples encode", ins.name));
        }
    }
    for ((i, j), word) in ambiguous {
        // Fine if one accepts fewer words (an alias: `mov` is `orr` with
        // both sources the same), or both write each word alike (one
        // instruction, described for several IR types).
        let words = |k: usize| -> Vec<Vec<u8>> { samples(t, k).filter_map(|e| t.instrs[k].encode(&e)).collect() };
        let takes =
            |k: usize, w: &[u8]| t.instrs[k].decode(w).and_then(|(e, _)| t.instrs[k].encode(&e)).as_deref() == Some(w);
        let (wi, wj) = (words(i), words(j));
        let narrower = wi.iter().any(|w| !takes(j, w)) || wj.iter().any(|w| !takes(i, w));
        let alike = wi.iter().chain(&wj).all(|w| text_of(t, i, w) == text_of(t, j, w));
        if narrower || alike {
            continue;
        }
        s.errors.push(format!(
            "{} and {} both encode {}, and neither fixes more bits: the disassembler cannot tell them apart \
             (if one is an alias, mark it \"nodecode\")",
            t.instrs[i].name,
            t.instrs[j].name,
            hex(&word)
        ));
    }
    for ((i, j), word) in confused {
        s.errors.push(format!(
            "{}: the disassembler reads {} as {}, which encodes it otherwise",
            t.instrs[i].name,
            hex(&word),
            t.instrs[j].name
        ));
    }
    s
}

/// An encoding, for messages: a word as a word, other lengths as bytes.
fn hex(b: &[u8]) -> String {
    match b.len() {
        4 => format!("{:08x}", crate::enc::word(b)),
        _ => b.iter().map(|x| format!("{x:02x}")).collect::<Vec<_>>().join(" "),
    }
}

/// Bytes `w` as instruction `i` writes them.
fn text_of(t: &Target, i: usize, w: &[u8]) -> Option<String> {
    let ins = &t.instrs[i];
    let (env, _) = ins.decode(w)?;
    Some(mir::fill_template(t, None, i, |k| {
        let o = &ins.operands[k];
        let v = env[&o.name];
        match o.kind {
            Kind::Reg(c) => Operand::Reg(Reg::P(t.machine.phys(c, v))),
            Kind::Label | Kind::Sym => Operand::Text(v.to_string()),
            Kind::Imm | Kind::Cond | Kind::Enum(_) => Operand::Imm(v),
        }
    }))
}

// ---------------------------------------------------------------------------
// Assembly

fn assembly(t: &Target) -> Section {
    let mut s = Section { name: "assembly", what: "instructions written and read back", ..Default::default() };
    let asm = Assembler::new(t);
    let mut alike: BTreeMap<(usize, usize), String> = BTreeMap::new();
    // One convention of each dialect.
    let mut abis: Vec<&crate::abi::Abi> = Vec::new();
    for a in &t.abis {
        if !abis.iter().any(|b| b.syntax == a.syntax) {
            abis.push(a);
        }
    }
    for abi in abis {
        let label = match abi.syntax {
            Syntax::Elf => ".Lhere",
            Syntax::MachO => "Lhere",
        };
        for (i, ins) in t.instrs.iter().enumerate() {
            if ins.format.is_some_and(|f| f != abi.syntax) {
                continue;
            }
            if ins.enc.is_empty() || ins.asm.is_empty() {
                continue;
            }
            // Each operand with bits of its own is written, or assembly
            // loses it.
            let holes: Vec<&str> = ins.asm.split('{').skip(1).filter_map(|h| h.split('}').next()).collect();
            let written = |n: &str| {
                holes.iter().any(|h| {
                    h.split(':').next() == Some(n)
                        || h.strip_prefix('=')
                            .is_some_and(|e| e.split(|c: char| !c.is_alphanumeric() && c != '_').any(|w| w == n))
                })
            };
            let lost: Vec<String> = ins
                .operands
                .iter()
                .filter(|o| !o.tied && !written(&o.name))
                .filter(|o| ins.enc.fields().any(|f| f.names.contains(&o.name)))
                .map(|o| o.name.clone())
                .collect();
            if !lost.is_empty() {
                s.errors.push(format!(
                    "{}: `{}` does not write operand{} {}, which the encoding holds",
                    ins.name,
                    ins.asm,
                    if lost.len() == 1 { "" } else { "s" },
                    lost.join(", ")
                ));
                continue;
            }
            // Three samples that fit.
            let envs: Vec<Env> = samples(t, i).take(3).collect();
            for env in envs {
                let Some((mut word, alt)) = ins.encode_alt(&env) else { continue };
                // A symbol's field holds zero in an object (the relocation's
                // addend has the rest), as the assembler writes it.
                for o in ins.operands.iter().filter(|o| o.kind == Kind::Sym) {
                    if ins.field_of(alt, &o.name, &env).1 != 0 {
                        ins.enc.alts[alt].clear(&mut word, &o.name);
                    }
                }
                let line = mir::fill_template(t, Some(abi), i, |k| {
                    let o = &ins.operands[k];
                    let v = env[&o.name];
                    match o.kind {
                        Kind::Reg(c) => Operand::Reg(Reg::P(t.machine.phys(c, v))),
                        Kind::Label => Operand::Text(label.into()),
                        Kind::Sym => Operand::Sym("sym".into()),
                        Kind::Imm | Kind::Cond | Kind::Enum(_) => Operand::Imm(v),
                    }
                });
                s.checked += 1;
                let text = format!("\t.text\n{label}:\n\t{line}\n");
                let got = asm.assemble(&text, Some(abi.syntax)).and_then(|a| {
                    let sec = a.object.sections.iter().find(|x| x.kind == SectionKind::Text).ok_or("no code")?;
                    if sec.data.is_empty() {
                        return Err("no instruction".into());
                    }
                    Ok(sec.data.clone())
                });
                match got {
                    Err(e) => {
                        s.errors.push(format!("{} ({}): `{line}` does not assemble: {e}", ins.name, abi.name));
                        break;
                    }
                    Ok(w) if w == word => {}
                    Ok(w) => {
                        // Another entry written the same way: fine if it
                        // reads back as the same text (an alias), but say so.
                        let other = mir::decode(t, &w).map(|(j, _, _)| j);
                        let back = mir::disassemble(t, &w, 0);
                        let mine = mir::disassemble(t, &word, 0);
                        if back == mine
                            && let Some(j) = other
                        {
                            alike.entry((i, j)).or_insert(line);
                        } else {
                            s.errors.push(format!(
                                "{} ({}): `{line}` is {}, but assembles as {} ({})",
                                ins.name,
                                abi.name,
                                hex(&word),
                                hex(&w),
                                other.map_or("nothing", |j| t.instrs[j].name.as_str())
                            ));
                            break;
                        }
                    }
                }
            }
        }
    }
    for ((i, j), line) in alike {
        if i != j {
            s.warnings.push(format!(
                "{} and {} are both written like `{line}`; the assembler chooses {}",
                t.instrs[i].name, t.instrs[j].name, t.instrs[j].name
            ));
        }
    }
    s
}

// ---------------------------------------------------------------------------
// Recipes

const SAMPLES: &[u64] = &[
    0,
    1,
    2,
    3,
    5,
    7,
    100,
    0x7fff_ffff,
    0x8000_0000,
    0xffff_ffff,
    0xffff_fffe,
    0x8000_0000_0000_0000,
    u64::MAX,
    0x1234_5678_9abc_def0,
];

fn recipes_check(t: &Target) -> Section {
    let mut s = Section { name: "recipes", what: "recipes evaluated on edge values", ..Default::default() };
    let recipes = match t.recipe_trees() {
        Ok(r) => r,
        Err(e) => {
            s.errors.push(e);
            return s;
        }
    };
    for (name, bits, want, got) in recipes {
        s.checked += 1;
        let (mut tried, mut wrong) = (0, None);
        let n = bits.len();
        let mut idx = vec![0usize; n];
        // Every combination, up to three operands; else a diagonal spread.
        let limit = if n <= 3 { usize::MAX } else { 2000 };
        let mut steps = 0;
        loop {
            let args: Vec<u64> = idx
                .iter()
                .zip(&bits)
                .map(|(&i, &b)| {
                    // Floating-point operands take the samples' bits, and
                    // some numbers of their own.
                    let fp = b & crate::target::FP != 0;
                    let w = b & !crate::target::FP;
                    let v = match (fp, i, w) {
                        (true, 1, 32) => f32::to_bits(1.0) as u64,
                        (true, 2, 32) => f32::to_bits(-2.5) as u64,
                        (true, 1, _) => f64::to_bits(1.0),
                        (true, 2, _) => f64::to_bits(-2.5),
                        _ => SAMPLES[i],
                    };
                    v & mask(w)
                })
                .collect();
            if let Some(w) = t.eval_tree(&want, &args) {
                tried += 1;
                let g = t.eval_tree(&got, &args);
                if g != Some(w) {
                    wrong = Some((args, w, g));
                    break;
                }
            }
            steps += 1;
            let mut k = 0;
            while k < n {
                idx[k] += 1;
                if idx[k] < SAMPLES.len() {
                    break;
                }
                idx[k] = 0;
                k += 1;
            }
            if k == n || steps >= limit {
                break;
            }
        }
        if let Some((args, w, g)) = wrong {
            s.errors.push(format!("{name}: on {args:x?}, the IR gives {w:#x}, the instructions {g:x?}"));
        } else if tried == 0 {
            s.warnings.push(format!("{name}: could not be evaluated on any sample"));
        }
    }
    s
}

// ---------------------------------------------------------------------------
// Smoke

/// `f`'s result, with a panic (a description the code generator did not
/// expect) as an error.
fn guarded<T>(f: impl FnOnce() -> Result<T, String>) -> Result<T, String> {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(f)).unwrap_or_else(|p| {
        let msg = p
            .downcast_ref::<String>()
            .cloned()
            .or_else(|| p.downcast_ref::<&str>().map(|s| s.to_string()))
            .unwrap_or_else(|| "?".into());
        Err(format!("the code generator panicked: {msg}"))
    })
}

fn smoke(t: &Target) -> Section {
    let mut s = Section { name: "smoke", what: "compilations of the smoke program", ..Default::default() };
    let funcs = SMOKE.lines().filter(|l| l.starts_with("define ")).count();
    let mut runs: Vec<(String, Options)> = Vec::new();
    for a in &t.abis {
        runs.push((a.name.clone(), Options { abi: Some(a.name.clone()), ..Options::default() }));
    }
    let first = t.abis.first().map(|a| a.name.clone());
    for (cpu, _) in &t.machine.cpus {
        if Some(cpu) != t.machine.default_cpu.as_ref() {
            runs.push((
                format!("--mcpu={cpu}"),
                Options { abi: first.clone(), mcpu: Some(cpu.clone()), ..Options::default() },
            ));
        }
    }
    for (label, opts) in runs {
        s.checked += 1;
        let opts = Options { keep_going: true, ..opts };
        let mut m = match epitome_ir::parse_module(SMOKE) {
            Ok(m) => m,
            Err(e) => {
                s.errors.push(format!("the smoke program does not parse: {e:?}"));
                return s;
            }
        };
        let compiled = match guarded(|| compile(t, &mut m, &opts)) {
            Ok(c) => c,
            Err(e) => {
                s.errors.push(format!("{label}: {e}"));
                continue;
            }
        };
        for e in &compiled.errors {
            s.errors.push(format!("{label}: {e}"));
        }
        if !compiled.errors.is_empty() {
            continue;
        }
        let bytes = match compiled.object_file(t) {
            Ok(b) => b,
            Err(e) => {
                s.errors.push(format!("{label}: writing the object file: {e}"));
                continue;
            }
        };
        match Assembler::new(t).assemble(&compiled.asm, Some(compiled.abi.syntax)).and_then(|a| a.file(t)) {
            Ok(b) if b == bytes => {}
            Ok(_) => s.errors.push(format!("{label}: the assembler makes another object of the assembly")),
            Err(e) => s.errors.push(format!("{label}: the assembly does not assemble: {e}")),
        }
        // Every word of code disassembles, as something that encodes it.
        let mut bad = BTreeSet::new();
        for code in compiled.funcs.iter().filter_map(|f| mir::encode_function(t, f).ok()) {
            for (k, &start) in code.starts.iter().enumerate() {
                let end = code.starts.get(k + 1).copied().unwrap_or(code.bytes.len());
                // (Not one a relocation fills in: its field is not yet what
                // it will be.)
                if code.fixups.iter().any(|x| (start..end).contains(&x.offset)) {
                    continue;
                }
                let w = &code.bytes[start..end];
                match mir::decode(t, w) {
                    Some((j, env, n)) if n == w.len() && t.instrs[j].encode(&env).as_deref() == Some(w) => {}
                    _ => {
                        bad.insert(w.to_vec());
                    }
                }
            }
        }
        for w in bad.iter().take(5) {
            s.errors.push(format!("{label}: the code has {}, which does not disassemble", hex(w)));
        }
    }
    // Which functions each optional feature is needed for (of those that
    // compile with every feature the default CPU has).
    let known = crate::features::known(t);
    let failing: BTreeSet<String> = s
        .errors
        .iter()
        .filter_map(|e| e.split_once(": @").map(|(_, f)| f.split(':').next().unwrap_or(f).to_string()))
        .collect();
    for f in &known {
        let opts = Options { abi: first.clone(), mattr: vec![format!("-{f}")], keep_going: true, ..Options::default() };
        let Ok(mut m) = epitome_ir::parse_module(SMOKE) else { break };
        s.checked += 1;
        match guarded(|| compile(t, &mut m, &opts)) {
            Ok(c) if c.errors.is_empty() => {}
            Ok(c) => {
                let names: Vec<&str> = c
                    .errors
                    .iter()
                    .map(|e| e.split(':').next().unwrap_or(e).trim_start_matches('@'))
                    .filter(|n| !failing.contains(*n))
                    .collect();
                if names.is_empty() {
                    continue;
                }
                s.notes.push(format!(
                    "without {f}, {} of the {funcs} smoke functions cannot be compiled: {}",
                    names.len(),
                    names.join(", ")
                ));
            }
            Err(e) => s.notes.push(format!("without {f}: {e}")),
        }
    }
    s
}

/// Assembly with sample lines of every instruction the description has
/// (each operand's samples as the encodings check uses, as many lines as
/// `per` of those that encode), for comparing assemblers. A branch goes to
/// itself: the reach of some is short.
pub fn sample_assembly(t: &Target, syntax: Syntax, per: usize) -> String {
    let abi = t.abis.iter().find(|a| a.syntax == syntax).or(t.abis.first());
    let label = match syntax {
        Syntax::Elf => ".Lhere",
        Syntax::MachO => "Lhere",
    };
    let mut out = "\t.text\n\t.p2align 2\n".to_string();
    let mut n = 0;
    for (i, ins) in t.instrs.iter().enumerate() {
        if ins.enc.is_empty() || ins.asm.is_empty() || ins.operands.iter().any(|o| o.kind == Kind::Sym) {
            continue;
        }
        let mut written = 0;
        for env in samples(t, i) {
            if written == per {
                break;
            }
            if ins.enc.encode(&env).is_none() {
                continue;
            }
            n += 1;
            let here = format!("{label}{n}");
            let line = mir::fill_template(t, abi, i, |k| {
                let o = &ins.operands[k];
                let v = env[&o.name];
                match o.kind {
                    Kind::Reg(c) => Operand::Reg(Reg::P(t.machine.phys(c, v))),
                    Kind::Label => Operand::Text(here.clone()),
                    Kind::Sym => Operand::Sym("sym".into()),
                    Kind::Imm | Kind::Cond | Kind::Enum(_) => Operand::Imm(v),
                }
            });
            out.push_str(&format!("{here}:\t{line}\t{} {}\n", t.machine.asm_comment, ins.name));
            written += 1;
        }
    }
    out
}
