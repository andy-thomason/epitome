//! Passes over machine instructions after selection.
//!
//! **Redundant compares.** Selection makes each user of a comparison with
//! its own compare, since a pattern covers the compare and what reads the
//! flags together: `(a < b) + (a == b) * 2` compares `a` with `b` twice. In
//! a block, a compare the same as the last instruction that set the flags
//! (the same instruction, on the same operands) is removed, when nothing
//! between them wrote the flags or the operands. What may write the flags is
//! an instruction that sets them (`"sets"` in the description), one that
//! clobbers them (x86's arithmetic: `"clobbers"="flags"`), a call, anything
//! else that clobbers registers (`imp_defs`), and inline assembly.
//! Each removal is an elective change (autochop).

use std::collections::{HashMap, HashSet};

use crate::machine::Holds;
use crate::mir::*;
use crate::target::{Kind, Target};

/// Removes redundant compares; returns how many.
pub fn redundant_compares(t: &Target, mf: &mut MFunc) -> usize {
    let mut removed = 0;
    let name = mf.name.clone();
    for b in &mut mf.blocks {
        let mut last: Option<MInst> = None;
        let mut keep = Vec::with_capacity(b.insts.len());
        for inst in std::mem::take(&mut b.insts) {
            let sets = matches!(inst.op, Opcode::Instr(i) if t.instrs[i].sets.is_some());
            if sets {
                // (Never a barrier: x86's fucompi also pops the x87 stack.)
                if !inst.barrier
                    && last.as_ref().is_some_and(|l| l.op == inst.op && l.ops == inst.ops)
                    && epitome_ir::chop::allow(|| format!("{name}: remove a compare the same as the last"))
                {
                    removed += 1;
                    continue;
                }
                last = Some(inst.clone());
                keep.push(inst);
                continue;
            }
            let clobbers = matches!(inst.op, Opcode::Instr(i) if t.instrs[i].clobbers_flags);
            if matches!(inst.op, Opcode::Asm { .. }) || !inst.imp_defs.is_empty() || clobbers {
                last = None;
            } else if let Some(l) = &last {
                let (defs, _) = inst.defs_uses(t);
                if defs.iter().any(|&k| l.ops.contains(&inst.ops[k])) {
                    last = None;
                }
            }
            keep.push(inst);
        }
        b.insts = keep;
    }
    removed
}

/// **Stack slots addressed from `sp`.** Selection makes each alloca's
/// address once, `add d, sp, #slot`, and uses `d` everywhere, so `d` lives
/// across the whole function (in a callee-saved register, saved and
/// restored). Instead, a load, store or add whose operand is `d` plus an
/// immediate (`Instr::base_offset`) takes `sp` and the slot directly
/// (`[sp, #slot+k]`), and any other use gets its own `add t, sp, #slot`
/// just before it. Frame lowering makes the slot operands offsets (from the
/// base register where `sp` moves), and copes with those too far for an
/// instruction. Returns how many uses changed.
pub fn frame_addresses(t: &Target, mf: &mut MFunc) -> usize {
    let sp = t.machine.sp;
    let Some(addi) = t.role("addi") else { return 0 };
    // Virtual registers defined once, by `add d, sp, #slot`.
    let mut defs: std::collections::HashMap<VReg, usize> = std::collections::HashMap::new();
    let mut frame_of: std::collections::HashMap<VReg, (usize, i64)> = std::collections::HashMap::new();
    for b in &mf.blocks {
        for inst in &b.insts {
            let (ds, _) = inst.defs_uses(t);
            for k in ds {
                if let MOp::Reg(Reg::V(v)) = inst.ops[k] {
                    *defs.entry(v).or_default() += 1;
                    if let (Opcode::Instr(i), [_, MOp::Reg(Reg::P(base)), MOp::Frame(s, o)]) = (inst.op, &inst.ops[..])
                        && (i == addi || t.instrs[i].address_of && t.instrs[i].base_offset == Some((1, 2)))
                        && *base == sp
                    {
                        frame_of.insert(v, (*s, *o));
                    }
                }
            }
        }
    }
    frame_of.retain(|v, _| defs.get(v) == Some(&1));
    if frame_of.is_empty() {
        return 0;
    }
    let mut changed = 0;
    let mut vclass = std::mem::take(&mut mf.vclass);
    for b in &mut mf.blocks {
        let old = std::mem::take(&mut b.insts);
        let mut new = Vec::with_capacity(old.len());
        for mut inst in old {
            // The definitions themselves go; every use makes its own.
            if let Opcode::Instr(i) = inst.op
                && (i == addi || t.instrs[i].address_of)
                && let MOp::Reg(Reg::V(v)) = inst.ops[0]
                && frame_of.contains_key(&v)
            {
                continue;
            }
            let (ds, us) = inst.defs_uses(t);
            // base + immediate: the slot, from sp.
            if let Opcode::Instr(i) = inst.op
                && let Some((base, off)) = t.instrs[i].base_offset
                && let (MOp::Reg(Reg::V(v)), MOp::Imm(k)) = (inst.ops[base].clone(), inst.ops[off].clone())
                && let Some(&(s, o)) = frame_of.get(&v)
                && !ds.contains(&base)
                // Below the slot: an offset from sp an instruction may not
                // take (unsigned offsets); left as base and offset.
                && o + k >= 0
            {
                inst.ops[base] = MOp::Reg(Reg::P(sp));
                inst.ops[off] = MOp::Frame(s, o + k);
                changed += 1;
            }
            // Any other use: the address made again just before.
            for k in us {
                if let MOp::Reg(Reg::V(v)) = inst.ops[k]
                    && let Some(&(s, o)) = frame_of.get(&v)
                {
                    let tmp = vclass.len() as VReg;
                    vclass.push(vclass[v as usize]);
                    new.push(MInst::new(addi, vec![MOp::Reg(Reg::V(tmp)), MOp::Reg(Reg::P(sp)), MOp::Frame(s, o)]));
                    inst.ops[k] = MOp::Reg(Reg::V(tmp));
                    changed += 1;
                }
            }
            new.push(inst);
        }
        b.insts = new;
    }
    mf.vclass = vclass;
    changed
}

/// **Load and store pairs.** After frame lowering, when offsets are final:
/// two loads (or stores) of one register class, neighbours or with only
/// instructions between that touch no memory and leave the second's
/// registers alone (it moves up to the first), from one base
/// register at offsets one register's width apart, become one pair
/// instruction (`ldp`, `stp`), when the description has one for the class
/// (roles `load.pair.<class>`, `store.pair.<class>`) and it holds the lower
/// offset. For loads, the two destinations differ, and the first does not
/// overwrite the base the second reads. Each pair is an elective change
/// (autochop). Returns how many pairs were made.
pub fn pair_memory(t: &Target, mf: &mut MFunc) -> usize {
    // Each single load or store with a pair: (its class's pair, width,
    // whether a load).
    let mut single: std::collections::HashMap<usize, (usize, i64, bool)> = std::collections::HashMap::new();
    for cd in &t.machine.classes {
        let (c, bytes) = (&cd.name, (cd.bits / 8) as i64);
        if bytes == 0 {
            continue;
        }
        for (load, kind) in [(true, "load"), (false, "store")] {
            if let (Some(one), Some(two)) = (t.role(&format!("{kind}.{c}")), t.role(&format!("{kind}.pair.{c}"))) {
                single.insert(one, (two, bytes, load));
            }
        }
    }
    if single.is_empty() {
        return 0;
    }
    // (instruction's single, register, base, offset)
    let access = |inst: &MInst| -> Option<(usize, u8, u8, i64)> {
        // An atomic access (a monotonic one is a plain load or store) is a
        // barrier: never merged.
        if inst.barrier {
            return None;
        }
        let Opcode::Instr(i) = inst.op else { return None };
        single.get(&i)?;
        match inst.ops[..] {
            [MOp::Reg(Reg::P(r)), MOp::Reg(Reg::P(base)), MOp::Imm(k)] => Some((i, r, base, k)),
            _ => None,
        }
    };
    let name = mf.name.clone();
    let mut made = 0;
    // What an instruction between two accesses must not be, for the second
    // to move up to the first: one that touches memory, calls or orders.
    let fence = |inst: &MInst| {
        inst.barrier
            || !inst.imp_defs.is_empty()
            || match inst.op {
                Opcode::Instr(k) => t.instrs[k].memory,
                Opcode::Asm { .. } => true,
                _ => false,
            }
    };
    for b in &mut mf.blocks {
        let old = std::mem::take(&mut b.insts);
        let mut gone = vec![false; old.len()];
        let mut new = Vec::with_capacity(old.len());
        for k in 0..old.len() {
            if gone[k] {
                continue;
            }
            let Some((i, ra, base, ka)) = access(&old[k]) else {
                new.push(old[k].clone());
                continue;
            };
            let (pair, width, load) = single[&i];
            // The next access, if nothing between touches memory.
            let Some(j) = (k + 1..old.len().min(k + 7)).find(|&j| access(&old[j]).is_some() || fence(&old[j])) else {
                new.push(old[k].clone());
                continue;
            };
            let mut paired = false;
            if let Some((i2, rb, base_b, kb)) = access(&old[j])
                && !gone[j]
                && i2 == i
                && base == base_b
            {
                // The second moves up past those between: they must not
                // write the base, or read or write the second's register
                // (a load's result; a store's value, written).
                let clear = old[k + 1..j].iter().all(|between| {
                    let (defs, uses) = between.defs_uses(t);
                    let names = |ks: &[usize], r: u8| ks.iter().any(|&q| between.ops[q] == MOp::Reg(Reg::P(r)));
                    !names(&defs, base) && !names(&defs, rb) && !(load && names(&uses, rb))
                });
                let ((lo_r, lo_k), (hi_r, _)) = if ka < kb { ((ra, ka), (rb, kb)) } else { ((rb, kb), (ra, ka)) };
                let apart = (ka - kb).abs() == width;
                // A load pair writes both at once: two different registers,
                // and the first of the two loads must not change the base
                // (nor the second, moved up past what reads it).
                let ok_load = !load || (ra != rb && ra != base && (j == k + 1 || rb != base));
                if clear
                    && apart
                    && ok_load
                    && t.instrs[pair].fits(&[None, None, None, Some(lo_k)])
                    && epitome_ir::chop::allow(|| format!("{name}: pair two accesses at [{base}, #{lo_k}]"))
                {
                    let r = |n: u8| MOp::Reg(Reg::P(n));
                    new.push(MInst::new(pair, vec![r(lo_r), r(hi_r), r(base), MOp::Imm(lo_k)]));
                    made += 1;
                    gone[j] = true;
                    paired = true;
                }
            }
            if !paired {
                new.push(old[k].clone());
            }
        }
        b.insts = new;
    }
    made
}

/// **Short forms.** After allocation, where the description has them (x86):
/// a move of 0 into a register becomes its zero idiom (role
/// `zero.<class>`, x86's `xor` of the register with itself, two bytes for
/// five) where the flags, which that clobbers, are dead after it: written
/// again before anything reads them, or never read in the block (nothing
/// keeps them from one block to the next). And a compare of a register
/// with 0 (role `cmpi.<class>`) becomes a test of it with itself (role
/// `test.<class>`, a byte shorter), which sets the flags as the compare
/// does. Each change is an elective one (autochop). Returns how many.
pub fn short_forms(t: &Target, mf: &mut MFunc) -> usize {
    let m = &t.machine;
    let class_name = |k: usize| match t.instrs[k].operands.first().map(|o| o.kind) {
        Some(Kind::Reg(c)) => Some(m.def(c).name.clone()),
        _ => None,
    };
    let name = mf.name.clone();
    let mut made = 0;
    for b in &mut mf.blocks {
        for k in 0..b.insts.len() {
            let Opcode::Instr(op) = b.insts[k].op else { continue };
            // A compare with 0: a test.
            if let [MOp::Reg(r), MOp::Imm(0)] = b.insts[k].ops[..]
                && let Some(c) = class_name(op)
                && t.has_role(op, &format!("cmpi.{c}"))
                && let Some(test) = t.role(&format!("test.{c}"))
                && epitome_ir::chop::allow(|| format!("{name}: test a register against itself for a compare with 0"))
            {
                b.insts[k] = MInst::new(test, vec![MOp::Reg(r), MOp::Reg(r)]);
                made += 1;
                continue;
            }
            // A move of 0: the zero idiom, where the flags are dead.
            if t.constants.contains(&op)
                && let [MOp::Reg(d), MOp::Imm(0)] = b.insts[k].ops[..]
                && let Some(c) = class_name(op)
                && let Some(zero) = t.role(&format!("zero.{c}"))
            {
                let dead = flags_dead(t, &b.insts[k + 1..]);
                if dead && epitome_ir::chop::allow(|| format!("{name}: zero a register by its zero idiom")) {
                    b.insts[k] = MInst::new(zero, vec![MOp::Reg(d)]);
                    made += 1;
                }
            }
        }
    }
    made
}

/// Whether the flags are dead after `rest` begins: written again before
/// anything reads them, or not read in the rest of the block.
fn flags_dead(t: &Target, rest: &[MInst]) -> bool {
    let reads = |i: &MInst| match i.op {
        Opcode::Instr(k) => t.instrs[k].operands.iter().any(|o| o.kind == Kind::Cond),
        Opcode::Asm { .. } => true,
        _ => false,
    };
    let writes = |i: &MInst| match i.op {
        Opcode::Instr(k) => t.instrs[k].sets.is_some() || t.instrs[k].clobbers_flags || i.is_call(t),
        Opcode::Asm { .. } => true,
        _ => false,
    };
    rest.iter().find(|i| reads(i) || writes(i)).is_none_or(|i| !reads(i))
}

/// **Three-address adds.** After allocation, where the description has
/// them (x86): a copy of `s` into `d` followed by a two-address add to `d`
/// (roles `add2i.<class>`, of an immediate, and `add2.<class>`, of a
/// register), whose flags are dead, becomes one three-address add that
/// leaves the flags alone (roles `addi` and `add`, x86's `lea`): `movq
/// %r15, %rax; addq $1, %rax` is `leaq 1(%r15), %rax`. Each change is an
/// elective one (autochop). Returns how many.
pub fn three_address(t: &Target, mf: &mut MFunc) -> usize {
    let m = &t.machine;
    let (Some(addi), Some(add)) = (t.role("addi"), t.role("add")) else { return 0 };
    let copies: Vec<usize> = t.copies.values().copied().collect();
    let sp = m.sp;
    let name = mf.name.clone();
    let mut made = 0;
    for b in &mut mf.blocks {
        let mut k = 0;
        while k + 1 < b.insts.len() {
            let (c, j) = (&b.insts[k], &b.insts[k + 1]);
            let (Opcode::Instr(ci), Opcode::Instr(ji)) = (c.op, j.op) else {
                k += 1;
                continue;
            };
            let [MOp::Reg(Reg::P(d)), MOp::Reg(Reg::P(s))] = c.ops[..] else {
                k += 1;
                continue;
            };
            let class = match t.instrs[ji].operands.first().map(|o| o.kind) {
                Some(Kind::Reg(cl)) => m.def(cl).name.clone(),
                _ => String::new(),
            };
            let new = match j.ops[..] {
                [MOp::Reg(Reg::P(x)), MOp::Reg(Reg::P(y)), MOp::Imm(v)]
                    if x == d && y == d && t.has_role(ji, &format!("add2i.{class}")) =>
                {
                    Some(MInst::new(addi, vec![MOp::Reg(Reg::P(d)), MOp::Reg(Reg::P(s)), MOp::Imm(v)]))
                }
                [MOp::Reg(Reg::P(x)), MOp::Reg(Reg::P(y)), MOp::Reg(Reg::P(r))]
                    if x == d && y == d && t.has_role(ji, &format!("add2.{class}")) =>
                {
                    // (d + d is s + s; an index cannot be sp.)
                    let r = if r == d { s } else { r };
                    let (base, index) = if r == sp { (r, s) } else { (s, r) };
                    (index != sp).then(|| {
                        MInst::new(add, vec![MOp::Reg(Reg::P(d)), MOp::Reg(Reg::P(base)), MOp::Reg(Reg::P(index))])
                    })
                }
                _ => None,
            };
            if let Some(new) = new
                && copies.contains(&ci)
                && d != s
                && flags_dead(t, &b.insts[k + 2..])
                && epitome_ir::chop::allow(|| format!("{name}: a copy and a two-address add as one three-address add"))
            {
                b.insts[k] = new;
                b.insts.remove(k + 1);
                made += 1;
            }
            k += 1;
        }
    }
    made
}

/// Whether `inst` makes its one result from immediates and symbols alone:
/// it reads no register, memory or flags and changes nothing else (SVE's
/// `ptrue p0.s`, `rdvl x0, #1`, `mov z0.s, #1`), as the allocator's
/// rematerialisation asks.
fn from_nothing(t: &Target, inst: &MInst) -> Option<VReg> {
    let Opcode::Instr(k) = inst.op else { return None };
    let ins = &t.instrs[k];
    let pure = ins.has_result
        && !ins.memory
        && !ins.operands.iter().any(|o| o.kind == Kind::Cond)
        && ins.sets.is_none()
        && !ins.clobbers_flags
        && ins.clobbers.is_empty()
        && !ins.second_result
        && inst.imp_defs.is_empty()
        && inst.imp_uses.is_empty()
        && !inst.barrier
        && inst.ops[1..].iter().all(|o| matches!(o, MOp::Imm(_) | MOp::Sym(_)));
    match inst.ops.first() {
        Some(&MOp::Reg(Reg::V(v))) if pure => Some(v),
        _ => None,
    }
}

/// How many times each virtual register is written.
fn def_counts(t: &Target, mf: &MFunc) -> Vec<u32> {
    let mut n = vec![0u32; mf.vclass.len()];
    for b in &mf.blocks {
        for inst in &b.insts {
            for k in inst.defs_uses(t).0 {
                if let MOp::Reg(Reg::V(v)) = inst.ops[k] {
                    n[v as usize] += 1;
                }
            }
        }
    }
    n
}

/// **Values made from nothing, made once.** Selection makes SVE's
/// all-true predicate for each predicated operation (`ptrue p0.s` under
/// every `mul`), and each multiple of the vector length (`rdvl`) and
/// small splat (`mov z0.s, #1`) where it is used, in a loop too. Alike
/// ones (the same instruction and immediates, values of the same register
/// file) of SVE's and SME's (their features), written once each, become
/// one, made in the block that dominates them, or out of the loops that
/// block is in (in the nearest block above it that is in none), before
/// anything there reads it; so does one alone in a loop. Where the value
/// cannot stay in a register (across a call, or a switch of streaming
/// mode, which clears the vector and predicate registers), the allocator
/// makes it again before its uses (rematerialisation), never spilling
/// it. Streaming mode changes the
/// vector length, so where a function switches it (role `sm.switch`) the
/// vector length's multiples are left where they are. Before allocation;
/// each merge is an elective change (autochop). Returns how many went.
pub fn shared_constants(t: &Target, mf: &mut MFunc) -> usize {
    let m = &t.machine;
    let switches = mf
        .blocks
        .iter()
        .flat_map(|b| &b.insts)
        .any(|i| matches!(i.op, Opcode::Instr(k) if t.has_role(k, "sm.switch")));
    let defs = def_counts(t, mf);
    // Blocks on a cycle: reached again from their successors.
    let n = mf.blocks.len();
    let mut cyclic = vec![false; n];
    for b in 0..n {
        let mut seen = vec![false; n];
        let mut work: Vec<usize> = mf.blocks[b].succs.clone();
        while let Some(x) = work.pop() {
            if x >= n || seen[x] {
                continue;
            }
            if x == b {
                cyclic[b] = true;
                break;
            }
            seen[x] = true;
            work.extend(mf.blocks[x].succs.iter().copied());
        }
    }
    // Groups of alike instructions: (the instruction, the vreg each
    // writes, and the blocks they are in).
    let mut groups: Vec<(MInst, Vec<VReg>, Vec<usize>)> = Vec::new();
    for (bi, b) in mf.blocks.iter().enumerate() {
        for inst in &b.insts {
            let Some(v) = from_nothing(t, inst) else { continue };
            let Opcode::Instr(k) = inst.op else { continue };
            let ins = &t.instrs[k];
            if defs[v as usize] != 1 || !ins.features.iter().any(|f| f.starts_with("sve") || f.starts_with("sme")) {
                continue;
            }
            let holds = m.def(mf.vclass[v as usize]).holds;
            if switches && !matches!(holds, Some(Holds::Scalable | Holds::Predicate)) {
                continue;
            }
            let file = m.def(mf.vclass[v as usize]).file;
            match groups.iter_mut().find(|(g, vs, _)| {
                g.op == inst.op && g.ops[1..] == inst.ops[1..] && m.def(mf.vclass[vs[0] as usize]).file == file
            }) {
                Some((_, vs, bs)) => {
                    vs.push(v);
                    bs.push(bi);
                }
                None => groups.push((inst.clone(), vec![v], vec![bi])),
            }
        }
    }
    let idom = dominators(mf);
    let name = mf.name.clone();
    let mut removed = 0;
    let mut rename: Vec<Option<VReg>> = vec![None; mf.vclass.len()];
    let mut made: Vec<(usize, MInst)> = Vec::new();
    for (inst, vs, bs) in groups {
        // Where they are made: the block that dominates them all, or the
        // nearest above it that is in no loop.
        let mut at = bs[0];
        for &b in &bs[1..] {
            at = common_dominator(&idom, at, b);
        }
        while cyclic[at] && at != 0 {
            at = idom[at];
        }
        if vs.len() < 2 && at == bs[0] {
            continue;
        }
        // The class each register of the others' classes is in: the
        // narrowest (pg's p0-p7 of p's p0-p15).
        let names = |v: VReg| -> HashSet<&str> {
            m.def(mf.vclass[v as usize]).names.iter().map(String::as_str).filter(|s| !s.is_empty()).collect()
        };
        let Some(&keep) = vs.iter().min_by_key(|&&v| names(v).len()) else { continue };
        if !vs.iter().all(|&v| names(keep).is_subset(&names(v))) {
            continue;
        }
        if !epitome_ir::chop::allow(|| format!("{name}: make v{keep} once, for {} uses", vs.len())) {
            continue;
        }
        for &v in &vs {
            rename[v as usize] = Some(keep);
        }
        let mut one = inst.clone();
        one.ops[0] = MOp::Reg(Reg::V(keep));
        made.push((at, one));
        removed += vs.len() - 1;
    }
    if made.is_empty() {
        return 0;
    }
    let to = |r: &mut MOp| {
        if let MOp::Reg(Reg::V(v)) = r
            && let Some(k) = rename[*v as usize]
        {
            *v = k;
        }
    };
    for b in &mut mf.blocks {
        let mut k = 0;
        while k < b.insts.len() {
            if from_nothing(t, &b.insts[k]).is_some_and(|v| rename[v as usize].is_some()) {
                b.insts.remove(k);
                if k < b.term_start {
                    b.term_start -= 1;
                }
                continue;
            }
            b.insts[k].ops.iter_mut().for_each(to);
            k += 1;
        }
    }
    // Before the first instruction of its block that reads it, and the
    // copies just before that (into an operand it overwrites, which
    // `overwrite_dying` and `prefixes` look for next to it).
    for (bi, inst) in made {
        let r = inst.ops[0].clone();
        let b = &mut mf.blocks[bi];
        let mut at = b
            .insts
            .iter()
            .position(|i| i.defs_uses(t).1.iter().any(|&u| i.ops[u] == r))
            .unwrap_or(b.term_start)
            .min(b.term_start);
        while at > 0 && b.insts[at - 1].op == Opcode::Copy && matches!(b.insts[at - 1].ops[1], MOp::Reg(Reg::V(_))) {
            at -= 1;
        }
        b.insts.insert(at, inst);
        b.term_start += 1;
    }
    removed
}

/// Each block's immediate dominator (the entry's, and an unreachable
/// block's, the entry).
fn dominators(mf: &MFunc) -> Vec<usize> {
    let n = mf.blocks.len();
    let mut preds = vec![Vec::new(); n];
    for (b, blk) in mf.blocks.iter().enumerate() {
        for &s in &blk.succs {
            if s < n {
                preds[s].push(b);
            }
        }
    }
    // Reverse postorder from the entry.
    let mut order = Vec::new();
    let mut seen = vec![false; n];
    let mut stack = vec![(0usize, 0usize)];
    seen[0] = true;
    while let Some((b, i)) = stack.pop() {
        if let Some(&s) = mf.blocks[b].succs.get(i) {
            stack.push((b, i + 1));
            if s < n && !seen[s] {
                seen[s] = true;
                stack.push((s, 0));
            }
        } else {
            order.push(b);
        }
    }
    order.reverse();
    let mut rank = vec![usize::MAX; n];
    for (k, &b) in order.iter().enumerate() {
        rank[b] = k;
    }
    let mut idom: Vec<Option<usize>> = vec![None; n];
    idom[0] = Some(0);
    let mut changed = true;
    while changed {
        changed = false;
        for &b in &order[1..] {
            let mut new: Option<usize> = None;
            for &p in &preds[b] {
                if idom[p].is_none() {
                    continue;
                }
                new = Some(match new {
                    None => p,
                    Some(mut x) => {
                        let mut y = p;
                        while x != y {
                            while rank[x] > rank[y] {
                                x = idom[x].unwrap();
                            }
                            while rank[y] > rank[x] {
                                y = idom[y].unwrap();
                            }
                        }
                        x
                    }
                });
            }
            if new.is_some() && idom[b] != new {
                idom[b] = new;
                changed = true;
            }
        }
    }
    idom.into_iter().map(|d| d.unwrap_or(0)).collect()
}

/// The nearest block that dominates both `a` and `b`.
fn common_dominator(idom: &[usize], a: usize, b: usize) -> usize {
    let mut above = HashSet::new();
    let mut x = a;
    loop {
        above.insert(x);
        if x == 0 || idom[x] == x {
            break;
        }
        x = idom[x];
    }
    let mut y = b;
    while !above.contains(&y) {
        if y == 0 || idom[y] == y {
            return 0;
        }
        y = idom[y];
    }
    y
}

/// Each block's virtual registers live out of it.
fn live_out(t: &Target, mf: &MFunc) -> Vec<HashSet<VReg>> {
    let n = mf.blocks.len();
    let mut gen_: Vec<HashSet<VReg>> = vec![HashSet::new(); n];
    let mut kill: Vec<HashSet<VReg>> = vec![HashSet::new(); n];
    for (bi, b) in mf.blocks.iter().enumerate() {
        for inst in &b.insts {
            let (defs, uses) = inst.defs_uses(t);
            for u in uses {
                if let MOp::Reg(Reg::V(v)) = inst.ops[u]
                    && !kill[bi].contains(&v)
                {
                    gen_[bi].insert(v);
                }
            }
            for d in defs {
                if let MOp::Reg(Reg::V(v)) = inst.ops[d] {
                    kill[bi].insert(v);
                }
            }
        }
    }
    let mut live_in: Vec<HashSet<VReg>> = gen_.clone();
    let mut out: Vec<HashSet<VReg>> = vec![HashSet::new(); n];
    let mut changed = true;
    while changed {
        changed = false;
        for bi in (0..n).rev() {
            let o: HashSet<VReg> =
                mf.blocks[bi].succs.iter().filter(|&&s| s < n).flat_map(|&s| live_in[s].iter().copied()).collect();
            let i: HashSet<VReg> = gen_[bi].iter().copied().chain(o.difference(&kill[bi]).copied()).collect();
            if i.len() != live_in[bi].len() {
                live_in[bi] = i;
                changed = true;
            }
            out[bi] = o;
        }
    }
    out
}

/// **The source that dies, overwritten.** SVE's predicated operations
/// overwrite their first source (`sdiv z0.s, p0/m, z0.s, z1.s`), so
/// selection copies it into the result first; where that source is still
/// needed after and the other dies there, the copy stays. Under an
/// all-true predicate (a `ptrue` of the operation's lanes, or of bytes,
/// where no lane keeps its first source's value), the entry the
/// description gives as its swap (`"swap"`: itself where it commutes,
/// `sdivr` of `sdiv`) takes the two sources the other way, so the one
/// overwritten is the one that dies, and allocation leaves no copy. Where
/// both die, the one from the physical register the result is copied to
/// (an argument's, returned) is overwritten. And where the copied source
/// is read again by the operation (a passthru and the source, the same
/// value) and dies there, the operation reads the copy instead. Before
/// allocation; each is an elective change (autochop). Returns how many.
pub fn overwrite_dying(t: &Target, mf: &mut MFunc) -> usize {
    if !t.instrs.iter().any(|i| i.swap.is_some()) {
        return 0;
    }
    let m = &t.machine;
    let defs = def_counts(t, mf);
    // The width of the lanes each all-true predicate is of.
    let mut all_true: HashMap<VReg, u32> = HashMap::new();
    for b in &mf.blocks {
        for inst in &b.insts {
            if let Opcode::Instr(k) = inst.op
                && t.instrs[k].role.as_deref().is_some_and(|r| r.split(',').any(|x| x.starts_with("ptrue.")))
                && let Some(&MOp::Reg(Reg::V(v))) = inst.ops.first()
                && defs[v as usize] == 1
            {
                all_true.insert(v, t.instrs[k].operands[0].bits);
            }
        }
    }
    // The physical register each value comes from or goes to by a copy
    // (an argument's, a result's): where allocation draws it.
    let mut drawn: HashMap<VReg, u8> = HashMap::new();
    for b in &mf.blocks {
        for inst in &b.insts {
            match inst.ops[..] {
                [MOp::Reg(Reg::V(v)), MOp::Reg(Reg::P(r))] | [MOp::Reg(Reg::P(r)), MOp::Reg(Reg::V(v))]
                    if inst.op == Opcode::Copy =>
                {
                    drawn.entry(v).or_insert(r);
                }
                _ => {}
            }
        }
    }
    let outs = live_out(t, mf);
    let name = mf.name.clone();
    let mut made = 0;
    for (bi, b) in mf.blocks.iter_mut().enumerate() {
        let mut live = outs[bi].clone();
        for k in (0..b.insts.len()).rev() {
            // The copied source read again by the operation, where it dies
            // there (the passthru and the source of `fneg z0.s, p0/m,
            // z1.s`, both a): the copy read instead, so the source dies at
            // the copy, which then goes.
            if k > 0
                && let Opcode::Instr(i) = b.insts[k].op
                && let Some(j) = t.instrs[i].operands.iter().position(|o| o.tied && !o.writeback)
                && let [MOp::Reg(Reg::V(res)), MOp::Reg(Reg::V(a))] = b.insts[k - 1].ops[..]
                && b.insts[k - 1].op == Opcode::Copy
                && a != res
                && b.insts[k].ops[0] == MOp::Reg(Reg::V(res))
                && b.insts[k].ops[j] == MOp::Reg(Reg::V(res))
                && !live.contains(&a)
                && b.insts[k].ops.contains(&MOp::Reg(Reg::V(a)))
                && b.insts[k].defs_uses(t).0.iter().all(|&d| b.insts[k].ops[d] != MOp::Reg(Reg::V(a)))
                && epitome_ir::chop::allow(|| format!("{name}: read v{res}, the copy of v{a}, in {}", t.instrs[i].name))
            {
                for op in &mut b.insts[k].ops[1..] {
                    if *op == MOp::Reg(Reg::V(a)) {
                        *op = MOp::Reg(Reg::V(res));
                    }
                }
                made += 1;
            }
            if k > 0
                && let Opcode::Instr(i) = b.insts[k].op
                && let Some(other) = t.instrs[i].swap.as_deref().and_then(|s| t.by_name.get(s).copied())
                && let Some(j) = t.instrs[i].operands.iter().position(|o| o.tied && !o.writeback)
                && t.instrs[i].operands.get(j + 1).is_some_and(|o| o.kind == t.instrs[i].operands[j].kind)
                && t.instrs[other].operands.get(j).is_some_and(|o| o.tied)
                && let [MOp::Reg(Reg::V(res)), MOp::Reg(Reg::V(a))] = b.insts[k - 1].ops[..]
                && b.insts[k - 1].op == Opcode::Copy
                && b.insts[k].ops[0] == MOp::Reg(Reg::V(res))
                && b.insts[k].ops[j] == MOp::Reg(Reg::V(res))
                && let MOp::Reg(Reg::V(x)) = b.insts[k].ops[j + 1]
                && x != a
                && x != res
                && a != res
                && !live.contains(&x)
                // (Where both die: the one from the register the result
                // goes to, as `smin` of an argument returned.)
                && (live.contains(&a)
                    || drawn.get(&res).is_some_and(|r| drawn.get(&x) == Some(r) && drawn.get(&a) != Some(r)))
                && b.insts[k].ops.iter().filter(|o| **o == MOp::Reg(Reg::V(x))).count() == 1
                && t.instrs[i].operands.iter().zip(&b.insts[k].ops).all(|(o, op)| match o.kind {
                    Kind::Reg(c) if m.def(c).holds == Some(Holds::Predicate) => match op {
                        MOp::Reg(Reg::V(p)) => all_true.get(p).is_some_and(|&w| w == 16 || w == o.bits),
                        _ => false,
                    },
                    _ => true,
                })
                && epitome_ir::chop::allow(|| format!("{name}: overwrite the source that dies, by {}", t.instrs[other].name))
            {
                b.insts[k - 1].ops[1] = MOp::Reg(Reg::V(x));
                b.insts[k].op = Opcode::Instr(other);
                b.insts[k].ops[j + 1] = MOp::Reg(Reg::V(a));
                made += 1;
            }
            let (defs, uses) = b.insts[k].defs_uses(t);
            for d in defs {
                if let MOp::Reg(Reg::V(v)) = b.insts[k].ops[d] {
                    live.remove(&v);
                }
            }
            for u in uses {
                if let MOp::Reg(Reg::V(v)) = b.insts[k].ops[u] {
                    live.insert(v);
                }
            }
        }
    }
    made
}

/// **`movprfx` for a copy into what is overwritten.** After allocation, a
/// copy of a z register (`mov z2.d, z0.d`) just before an operation that
/// overwrites the copy (`"movprfx"`: SVE's destructive forms) and reads
/// it nowhere else becomes `movprfx z2, z0`, which cores may fuse with
/// the operation into one. Each is an elective change (autochop). Returns
/// how many.
pub fn prefixes(t: &Target, mf: &mut MFunc) -> usize {
    let Some(prfx) = t.role("movprfx") else { return 0 };
    let Kind::Reg(z) = t.instrs[prfx].operands[0].kind else { return 0 };
    let Some(&mov) = t.copies.get(&z) else { return 0 };
    let name = mf.name.clone();
    let mut made = 0;
    for b in &mut mf.blocks {
        for k in 0..b.insts.len().saturating_sub(1) {
            if b.insts[k].op != Opcode::Instr(mov) {
                continue;
            }
            let [MOp::Reg(Reg::P(d)), MOp::Reg(Reg::P(s))] = b.insts[k].ops[..] else { continue };
            let Opcode::Instr(i) = b.insts[k + 1].op else { continue };
            let ins = &t.instrs[i];
            let Some(j) = ins.operands.iter().position(|o| o.tied && !o.writeback) else { continue };
            let next = &b.insts[k + 1].ops;
            if d != s
                && ins.prefixable
                && next[0] == MOp::Reg(Reg::P(d))
                && next[j] == MOp::Reg(Reg::P(d))
                && next.iter().enumerate().all(|(x, o)| x == 0 || x == j || *o != MOp::Reg(Reg::P(d)))
                && epitome_ir::chop::allow(|| format!("{name}: movprfx for the copy before {}", ins.name))
            {
                b.insts[k] = MInst::new(prfx, vec![MOp::Reg(Reg::P(d)), MOp::Reg(Reg::P(s))]);
                made += 1;
            }
        }
    }
    made
}
