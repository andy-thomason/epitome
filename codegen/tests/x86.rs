//! The x86-64 description's instructions (generated from Intel XED's
//! tables: integer, x87, MMX, SSE, AVX, AVX-512 and the rest), written and
//! read back. For every
//! entry, lines at the edges of its operands: every register in each place,
//! immediates and displacements at their limits, each memory form,
//! RIP-relative symbols, branches near and far. The test:
//!
//! 1. checks the description (`--check-target`'s encodings: each sample
//!    encodes, decodes as itself, and disassembles);
//! 2. assembles the lines with epitome-as, and checks epitome-objdump reads
//!    the object back as them;
//! 3. where `llvm-mc` is installed (`llvm-mc` or `llvm-mc-18`), assembles
//!    them with it too, and checks the code and relocations are the same.

use std::process::Command;

use epitome_codegen::abi::Syntax;
use epitome_codegen::assembler::Assembler;
use epitome_codegen::enc::Env;
use epitome_codegen::mir::{Operand, Reg, fill_template};
use epitome_codegen::objdump::{self, Image};
use epitome_codegen::target::{Instr, Kind, Target};

const DISPS: &[i64] = &[0, 1, -1, 127, -128, 128, -129, i32::MAX as i64, i32::MIN as i64];

/// Values at the edges of the widest field holding immediate `name`.
fn imms(ins: &Instr, name: &str) -> Vec<i64> {
    // The widest fields' ranges together (one may be read signed, another
    // unsigned).
    let fields: Vec<_> = ins.enc.fields().filter(|f| f.names.as_slice() == [name.to_string()] && f.width > 0).collect();
    let Some(w) = fields.iter().map(|f| f.width).max() else { return vec![0, 1] };
    let range = |signed: bool| match (signed, w) {
        (_, 64..) => (i64::MIN, i64::MAX),
        (true, w) => (-(1i64 << (w - 1)), (1i64 << (w - 1)) - 1),
        (false, w) => (0, (1i64 << w) - 1),
    };
    let widest = fields.iter().filter(|f| f.width == w).map(|f| range(f.signed));
    let (lo, hi) = widest.fold((i64::MAX, i64::MIN), |(lo, hi), (l, h)| (lo.min(l), hi.max(h)));
    let mut out = Vec::new();
    for v in [0, 1, hi, lo, 127, -128, 128, -129, 255, 32767, -32768, i32::MAX as i64, i32::MIN as i64] {
        if (lo..=hi).contains(&v) && !out.contains(&v) {
            out.push(v);
        }
    }
    out
}

/// An operand's sample value.
enum V {
    Reg(u8),
    Imm(i64),
    Text(String),
}

/// Lines for every entry (or, not `all`, those the disassembler reads as
/// themselves: not "nodecode" ones, written another way), each operand
/// cycling through its values.
fn samples(t: &Target, all: bool) -> String {
    let mut out = String::from("\t.text\n");
    for (i, ins) in t.instrs.iter().enumerate() {
        if ins.enc.is_empty() || ins.asm.is_empty() || !(all || ins.decode) {
            continue;
        }
        let back = format!(".Lback{}", ins.name);
        let lists: Vec<Vec<V>> = ins
            .operands
            .iter()
            .map(|o| match o.kind {
                // (A mask, written `{%k1}`, is k1-k7: k0 is none.)
                Kind::Reg(c) => (0..t.machine.def(c).names.len() as i64)
                    .filter(|&k| !(o.name == "m.index" && k == 4))
                    .filter(|&k| !(k == 0 && ins.asm.contains(&format!("{{lb}}{{{}}}{{rb}}", o.name))))
                    .map(|k| V::Reg(t.machine.phys(c, k)))
                    .collect(),
                Kind::Imm => {
                    let vals = match o.name.as_str() {
                        "m.disp" => DISPS.to_vec(),
                        "m.scale" => vec![1, 2, 4, 8],
                        name => imms(ins, name),
                    };
                    vals.into_iter().map(V::Imm).collect()
                }
                Kind::Sym => vec![V::Text("sym".into()), V::Text("other".into())],
                // (Branches of a byte only reach near.)
                Kind::Label if ins.enc.fields().any(|f| f.width == 32 && f.names.contains(&o.name)) => {
                    vec![V::Text(back.clone()), V::Text(".Lfar".into())]
                }
                Kind::Label => vec![V::Text(back.clone())],
                Kind::Cond => t.conds.iter().map(|c| V::Imm(c.value)).collect(),
                Kind::Enum(e) => t.machine.enums[e as usize].names.iter().map(|&(_, v)| V::Imm(v)).collect(),
            })
            .collect();
        let n = lists.iter().map(Vec::len).max().unwrap_or(1).max(1);
        if ins.operands.iter().any(|o| o.kind == Kind::Label) {
            out.push_str(&format!("{back}:\n"));
        }
        let mut seen = std::collections::HashSet::new();
        for k in 0..n {
            let pick = |j: usize| &lists[j][(k * (j + 1)) % lists[j].len()];
            // The values must encode (an index register is never rsp); and
            // to be read back as written, an immediate must be the value its
            // bytes decode as ($255 to a byte operation is $-1).
            {
                let env: Env = ins
                    .operands
                    .iter()
                    .enumerate()
                    .map(|(j, o)| {
                        let v = match pick(j) {
                            V::Reg(r) => t.machine.index(*r),
                            V::Imm(v) => *v,
                            V::Text(_) => 0,
                        };
                        (o.name.clone(), v)
                    })
                    .collect();
                let Some(bytes) = ins.encode(&env) else { continue };
                let back = ins.decode(&bytes).map(|(e, _)| e);
                let same = back.is_some_and(|b| {
                    ins.operands.iter().all(|o| o.kind != Kind::Imm || b.get(&o.name) == env.get(&o.name))
                });
                if !all && !same {
                    continue;
                }
            }
            let line = fill_template(t, None, i, |j| match pick(j) {
                V::Reg(r) => Operand::Reg(Reg::P(*r)),
                V::Imm(v) => Operand::Imm(*v),
                V::Text(s) => Operand::Text(s.clone()),
            });
            if seen.insert(line.clone()) {
                out.push_str(&format!("\t{line}\n"));
            }
        }
    }
    out.push_str(".Lfar:\n");
    out
}

/// A relocation: where, its type, its target and addend.
type Reloc = (u64, String, String, i64);

fn text_of(img: &Image) -> (&[u8], Vec<Reloc>) {
    let s = img.sections.iter().find(|s| s.name == ".text").expect("a .text section");
    let relocs = s.relocs.iter().map(|r| (r.offset, r.type_name.clone(), r.target.clone(), r.addend)).collect();
    (&s.data, relocs)
}

#[test]
fn x86_instructions_round_trip() {
    let t = Target::x86_64();

    // 1. The description's own check.
    let report = epitome_codegen::check::check(&t);
    let enc = report.sections.iter().find(|s| s.name == "encodings").unwrap();
    assert!(enc.errors.is_empty(), "{}", enc.errors.join("\n"));
    assert!(enc.checked > 400000, "{} checked", enc.checked);

    // 2. Ours, written and read back.
    let read = samples(&t, false);
    let lines = read.lines().filter(|l| l.starts_with('\t')).count() - 1;
    let object = Assembler::new(&t).assemble(&read, Some(Syntax::Elf)).and_then(|a| a.file(&t)).unwrap();
    let img = Image::read(&object, &t.machine).unwrap();
    let n = objdump::reads_back(&t, &img, &read, &[]).unwrap();
    assert_eq!(n, lines);

    // Every entry, for llvm-mc.
    let asm = samples(&t, true);
    let lines = asm.lines().filter(|l| l.starts_with('\t')).count() - 1;
    let ours = Assembler::new(&t).assemble(&asm, Some(Syntax::Elf)).and_then(|a| a.file(&t)).unwrap();
    let img = Image::read(&ours, &t.machine).unwrap();

    // 3. llvm-mc's, if it is here.
    let Some(mc) = ["llvm-mc", "llvm-mc-18"].into_iter().find(|c| Command::new(c).arg("--version").output().is_ok())
    else {
        eprintln!("llvm-mc is not installed: not compared with it");
        return;
    };
    let tmp = std::env::temp_dir().join(format!("epitome-x86-{}", std::process::id()));
    std::fs::create_dir_all(&tmp).unwrap();
    let (s, o) = (tmp.join("sample.s"), tmp.join("theirs.o"));
    std::fs::write(&s, &asm).unwrap();
    let out =
        Command::new(mc).args(["-triple=x86_64-linux-gnu", "-filetype=obj", "-o"]).arg(&o).arg(&s).output().unwrap();
    assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
    let theirs = Image::read(&std::fs::read(&o).unwrap(), &t.machine).unwrap();
    let ((ours_code, ours_rel), (their_code, their_rel)) = (text_of(&img), text_of(&theirs));
    if ours_code != their_code {
        let at = ours_code.iter().zip(their_code).position(|(a, b)| a != b).unwrap_or(0);
        // The first instruction of a different length, or else the first
        // that differs: both walked from the start.
        let (mut a, mut b, mut k) = (0, 0, 0);
        let len = |code: &[u8]| epitome_codegen::mir::decode(&t, code).map_or(1, |(_, _, n)| n);
        let mut first = None;
        while a < ours_code.len() && b < their_code.len() {
            let (na, nb) = (len(&ours_code[a..]), len(&their_code[b..]));
            if na != nb || (first.is_none() && ours_code[a..a + na] != their_code[b..b + nb]) {
                first = Some((k, a, b));
                if na != nb {
                    break;
                }
            }
            a += na;
            b += nb;
            k += 1;
        }
        let (k, a, b) = first.unwrap_or((0, at, at));
        let line = asm.lines().filter(|l| l.starts_with('\t')).nth(k + 1).unwrap_or("?");
        panic!(
            "the code differs from llvm-mc's at byte {at:#x}; first at `{}`: ours {:02x?}, llvm-mc's {:02x?}",
            line.trim(),
            &ours_code[a..(a + 16).min(ours_code.len())],
            &their_code[b..(b + 16).min(their_code.len())]
        );
    }
    assert_eq!(ours_rel, their_rel, "the relocations differ from llvm-mc's");
    let _ = std::fs::remove_dir_all(&tmp);
    eprintln!("{lines} lines: the same {} bytes and {} relocations as {mc}", ours_code.len(), ours_rel.len());
}
