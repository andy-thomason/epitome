//! Register allocation: linear scan over live ranges.
//!
//! Instruction `n` reads its operands at position `2n` and writes its
//! results at `2n + 1`, so a value last read by an instruction can share a
//! register with the value it writes. Liveness is worked out per block,
//! and each virtual register gets one segment per block it is live in,
//! from where it becomes live to where it dies or the block ends.
//!
//! Physical registers have fixed stretches where they are taken: an
//! argument from its copy to the call, a result from the call to its copy,
//! and every caller-saved register at each call. Values live across a call
//! therefore end up in callee-saved registers. Copies give hints, so that a
//! copy's two sides often share a register and the copy disappears.
//! Before that, the two sides of a copy whose live ranges never meet are
//! made one value (coalescing); where the allocation then spills, the one
//! without coalescing is made too, and the smaller kept.
//!
//! When no register is free, a value may take one from values that are
//! cheaper to spill: those whose uses are sparser over their ranges (their
//! weight, uses and definitions over length, is less), which then try
//! again (each is moved so at most once). Otherwise the value goes to a
//! stack slot and is loaded into a scratch register (`x16`, `x17`, `x15`)
//! before each use and stored after each definition.

use std::collections::{HashMap, HashSet};

use crate::mir::*;
use crate::target::Target;

/// The registers an instruction reads and writes, including implicit ones.
fn regs_of(t: &Target, inst: &MInst) -> (Vec<Reg>, Vec<Reg>) {
    let (d, u) = inst.defs_uses(t);
    let reg = |k: usize| match inst.ops[k] {
        MOp::Reg(r) => Some(r),
        _ => None,
    };
    let mut defs: Vec<Reg> = d.into_iter().filter_map(reg).collect();
    let mut uses: Vec<Reg> = u.into_iter().filter_map(reg).collect();
    defs.extend(inst.imp_defs.iter().map(|&r| Reg::P(r)));
    uses.extend(inst.imp_uses.iter().map(|&r| Reg::P(r)));
    // The zero register and the stack pointer are not allocated.
    let fixed = |r: &Reg| *r == Reg::P(t.machine.sp) || t.machine.zero.is_some_and(|z| *r == Reg::P(z));
    defs.retain(|r| !fixed(r));
    uses.retain(|r| !fixed(r));
    (defs, uses)
}

type Interval = (u32, u32);

/// Where a physical register is taken: closed intervals, kept as their
/// union, disjoint and by start, so that whether an interval meets any of
/// them is one lookup (the last one starting at or before its end).
#[derive(Default, Clone)]
struct Busy(std::collections::BTreeMap<u32, u32>);

impl Busy {
    fn meets(&self, (a, b): Interval) -> bool {
        self.0.range(..=b).next_back().is_some_and(|(_, &e)| e >= a)
    }

    fn add(&mut self, (mut a, mut b): Interval) {
        while let Some((&s, &e)) = self.0.range(..=b).next_back() {
            if e < a {
                break;
            }
            self.0.remove(&s);
            a = a.min(s);
            b = b.max(e);
        }
        self.0.insert(a, b);
    }

    /// Takes away a value's intervals, added when they met nothing already
    /// there. They may meet each other (an empty block's segment shares its
    /// position with the next block's), so it is their union that is one
    /// entry each.
    fn remove(&mut self, segs: &[Interval]) {
        let mut k = 0;
        while k < segs.len() {
            let (a, mut b) = segs[k];
            k += 1;
            while k < segs.len() && segs[k].0 <= b {
                b = b.max(segs[k].1);
                k += 1;
            }
            let e = self.0.remove(&a);
            debug_assert_eq!(e, Some(b), "an interval merged with another value's cannot be taken away");
        }
    }
}

/// Allocates registers. Spill code needs scratch registers (`scratch`,
/// `fp-scratch`), and so do frames too big for an instruction's offsets;
/// a function that needs neither may use them like any others. So the
/// first attempt, for a small frame, allocates the scratch registers too,
/// and is kept if nothing spills: the function then has no scratch
/// registers, and anything after that would need one says so. Otherwise
/// the allocation is made again with them kept free.
pub fn allocate(t: &Target, f: &mut MFunc) -> Result<(), String> {
    let slots: u64 = f.slots.iter().map(|s| s.size + s.align).sum();
    let small = !f.dynamic_stack && slots + f.out_size.max(0) as u64 + 512 < 4000;
    allocate_as(t, f, small)
}

/// Allocates registers keeping the scratch registers free: for a frame
/// that turned out to need one (`frame::NEEDS_SCRATCH`).
pub fn allocate_keeping_scratch(t: &Target, f: &mut MFunc) -> Result<(), String> {
    allocate_as(t, f, false)
}

fn allocate_as(t: &Target, f: &mut MFunc, try_scratch: bool) -> Result<(), String> {
    let before = f.clone();
    if try_scratch && !(f.abi.scratch.is_empty() && f.abi.fp_scratch.is_empty()) {
        let kept = f.abi.clone();
        f.abi.scratch.clear();
        f.abi.fp_scratch.clear();
        if allocate_with(t, f, true, true)? {
            return Ok(());
        }
        f.abi = kept;
    }
    let slots = f.slots.len();
    allocate_with(t, f, false, true)?;
    // Coalesced values live longer, so where registers ran out, the
    // allocation without coalescing may do better: the one of fewer
    // instructions is kept.
    if f.slots.len() > slots {
        let mut other = before;
        allocate_with(t, &mut other, false, false)?;
        let size = |f: &MFunc| f.blocks.iter().map(|b| b.insts.len()).sum::<usize>();
        if size(&other) < size(f) {
            *f = other;
        }
    }
    Ok(())
}

/// Allocates registers, coalescing copies first if `coalescing`; with
/// `bail` set, gives up (`false`) instead of spilling.
fn allocate_with(t: &Target, f: &mut MFunc, bail: bool, coalescing: bool) -> Result<bool, String> {
    record_copy_classes(f);
    let m = &t.machine;
    let abi = f.abi.clone();
    let caller_saved = abi.caller_saved();
    let callee_saved = abi.callee_saved();
    let fp_caller_saved = abi.fp_caller_saved();
    let fp_callee_saved = abi.fp_callee_saved();
    let allocatable = |r: u8| {
        caller_saved.contains(&r)
            || callee_saved.contains(&r)
            || fp_caller_saved.contains(&r)
            || fp_callee_saved.contains(&r)
    };
    let nv = f.vclass.len();
    let classes = f.vclass.clone();
    // A value wider than the registers a function preserves (a vector, of
    // which only the low 64 bits of AArch64's v8-v15 survive a call) cannot
    // be kept in one across a call.
    let too_wide = |v: VReg| {
        let c = m.def(classes[v as usize]);
        m.save_class(c.file).is_none_or(|s| c.bits > m.def(s).bits)
    };
    // Positions.
    let mut block_range: Vec<Interval> = Vec::new();
    let mut n = 0u32;
    for b in &f.blocks {
        let start = 2 * n;
        n += b.insts.len() as u32;
        let end = if b.insts.is_empty() { start } else { 2 * n - 1 };
        block_range.push((start, end));
    }

    // Liveness of virtual registers.
    let nb = f.blocks.len();
    let mut gen_: Vec<HashSet<VReg>> = vec![HashSet::new(); nb];
    let mut kill: Vec<HashSet<VReg>> = vec![HashSet::new(); nb];
    for (bi, b) in f.blocks.iter().enumerate() {
        for inst in &b.insts {
            let (defs, uses) = regs_of(t, inst);
            for r in uses {
                if let Reg::V(v) = r
                    && !kill[bi].contains(&v)
                {
                    gen_[bi].insert(v);
                }
            }
            for r in defs {
                if let Reg::V(v) = r {
                    kill[bi].insert(v);
                }
            }
        }
    }
    let mut live_in: Vec<HashSet<VReg>> = vec![HashSet::new(); nb];
    let mut live_out: Vec<HashSet<VReg>> = vec![HashSet::new(); nb];
    let mut changed = true;
    while changed {
        changed = false;
        for bi in (0..nb).rev() {
            let mut out: HashSet<VReg> = HashSet::new();
            for &s in &f.blocks[bi].succs {
                out.extend(live_in[s].iter().copied());
            }
            let mut inn: HashSet<VReg> = gen_[bi].clone();
            inn.extend(out.iter().filter(|v| !kill[bi].contains(v)).copied());
            if inn.len() != live_in[bi].len() || out.len() != live_out[bi].len() {
                changed = true;
            }
            live_in[bi] = inn;
            live_out[bi] = out;
        }
    }

    // Segments, fixed stretches, hints and calls.
    let mut segs: Vec<Vec<Interval>> = vec![Vec::new(); nv];
    let mut fixed: HashMap<u8, Vec<Interval>> = HashMap::new();
    let mut hints: Vec<Vec<Reg>> = vec![Vec::new(); nv];
    // Each value's uses and definitions, for its weight.
    let mut occ: Vec<u32> = vec![0; nv];
    let mut calls: Vec<u32> = Vec::new();
    let mut idx = 0u32;
    for (bi, b) in f.blocks.iter().enumerate() {
        let (bs, be) = block_range[bi];
        let mut local: HashMap<VReg, Interval> = HashMap::new();
        let widen = |local: &mut HashMap<VReg, Interval>, v: VReg, p: u32| {
            local.entry(v).and_modify(|e| e.1 = e.1.max(p)).or_insert((p, p));
        };
        for &v in &live_in[bi] {
            widen(&mut local, v, bs);
        }
        let mut open: HashMap<u8, u32> = HashMap::new();
        for inst in &b.insts {
            let (defs, uses) = regs_of(t, inst);
            let (up, dp) = (2 * idx, 2 * idx + 1);
            // Inline assembly's early-clobber results are written while its
            // operands are still being read.
            let early: Vec<Reg> = match inst.op {
                Opcode::Asm { id, .. } => f.asms[id as usize]
                    .early
                    .iter()
                    .enumerate()
                    .filter(|e| *e.1)
                    .filter_map(|(k, _)| if let MOp::Reg(r) = inst.ops[k] { Some(r) } else { None })
                    .collect(),
                _ => vec![],
            };
            for r in &uses {
                match *r {
                    Reg::V(v) => {
                        occ[v as usize] += 1;
                        widen(&mut local, v, up)
                    }
                    Reg::P(p) if allocatable(p) => {
                        let s = open.remove(&p).unwrap_or(bs);
                        fixed.entry(p).or_default().push((s, up));
                    }
                    _ => {}
                }
            }
            for r in &defs {
                let dp = if early.contains(r) { up } else { dp };
                match *r {
                    Reg::V(v) => {
                        occ[v as usize] += 1;
                        widen(&mut local, v, dp)
                    }
                    Reg::P(p) if allocatable(p) => {
                        if let Some(s) = open.insert(p, dp) {
                            fixed.entry(p).or_default().push((s, s));
                        }
                    }
                    _ => {}
                }
            }
            if inst.is_call(t) {
                calls.push(dp);
            }
            if inst.op == Opcode::Copy
                && let (MOp::Reg(a), MOp::Reg(c)) = (&inst.ops[0], &inst.ops[1])
            {
                if let Reg::V(v) = a {
                    hints[*v as usize].push(*c);
                }
                if let Reg::V(v) = c {
                    hints[*v as usize].push(*a);
                }
            }
            idx += 1;
        }
        for (p, s) in open {
            fixed.entry(p).or_default().push((s, s));
        }
        for &v in &live_out[bi] {
            widen(&mut local, v, be);
        }
        for (v, s) in local {
            segs[v as usize].push(s);
        }
    }

    if coalescing {
        coalesce(f, &mut segs, &mut hints, &mut occ);
    }

    // Linear scan.
    for s in &mut segs {
        s.sort();
    }
    let mut order: Vec<VReg> = (0..nv as VReg).filter(|&v| !segs[v as usize].is_empty()).collect();
    order.sort_by_key(|&v| (segs[v as usize][0].0, v));
    let mut taken: HashMap<u8, Busy> = HashMap::new();
    // The fixed stretches alone (which no value can be moved out of), and
    // which value holds each stretch of a register.
    let mut fixed_busy: HashMap<u8, Busy> = HashMap::new();
    for (p, list) in fixed {
        let busy = taken.entry(p).or_default();
        let fb = fixed_busy.entry(p).or_default();
        for s in list {
            busy.add(s);
            fb.add(s);
        }
    }
    let mut owners: HashMap<u8, std::collections::BTreeMap<u32, (u32, VReg)>> = HashMap::new();
    let weight = |v: VReg| -> f64 {
        let len: u32 = segs[v as usize].iter().map(|s| s.1 - s.0 + 1).sum();
        f64::from(occ[v as usize]) / f64::from(len + 1)
    };
    let mut assigned: Vec<Option<u8>> = vec![None; nv];
    let mut spilled: Vec<VReg> = Vec::new();
    let mut moved: HashSet<VReg> = HashSet::new();
    let mut work: std::collections::VecDeque<VReg> = order.into_iter().collect();
    // Each value: is a call strictly inside one of its segments (calls are
    // in order)?
    let across: Vec<bool> = segs
        .iter()
        .map(|mine| {
            mine.iter().any(|s| {
                let k = calls.partition_point(|&c| c <= s.0);
                k < calls.len() && calls[k] < s.1
            })
        })
        .collect();
    while let Some(v) = work.pop_front() {
        let mine = &segs[v as usize];
        let across_call = across[v as usize];
        // A value copied to or from one that lives across a call (a loop's
        // phi and its next value, with a call in the loop) tries the
        // callee-saved registers first too, so that both may share one.
        let partner_across = hints[v as usize].iter().any(|h| matches!(h, Reg::V(x) if across[*x as usize]));
        // Registers of values joined to this one by copies, nearest first.
        let mut cands: Vec<u8> = Vec::new();
        let mut seen: HashSet<VReg> = HashSet::from([v]);
        let mut frontier: Vec<VReg> = vec![v];
        for _ in 0..4 {
            let mut next = Vec::new();
            for &w in &frontier {
                for h in &hints[w as usize] {
                    match h {
                        Reg::P(p) if allocatable(*p) => cands.push(*p),
                        Reg::V(x) => {
                            if let Some(p) = assigned[*x as usize] {
                                cands.push(p);
                            }
                            if seen.insert(*x) {
                                next.push(*x);
                            }
                        }
                        _ => {}
                    }
                }
            }
            frontier = next;
        }
        let no_saved: Vec<u8> = Vec::new();
        let wide = too_wide(v);
        if wide && across_call {
            cands.clear();
        }
        let (callee, fp_callee) = if wide { (&no_saved, &no_saved) } else { (&callee_saved, &fp_callee_saved) };
        let caller = caller_saved.iter().chain(&fp_caller_saved);
        let callee = callee.iter().chain(fp_callee);
        if across_call || partner_across {
            cands.extend(callee.chain(caller).copied());
        } else {
            cands.extend(caller.chain(callee).copied());
        }
        // Only registers of the value's own class.
        let class = classes[v as usize];
        cands.retain(|&p| m.in_class(class, p));
        let free = cands.iter().copied().find(|p| taken.get(p).is_none_or(|busy| mine.iter().all(|&m| !busy.meets(m))));
        // Else a register whose values in the way are all lighter (and
        // movable), the lightest in all.
        let free = free.map(|p| (p, Vec::new())).or_else(|| {
            let w = weight(v);
            let mut best: Option<(u8, Vec<VReg>, f64)> = None;
            'regs: for &p in &cands {
                if fixed_busy.get(&p).is_some_and(|fb| mine.iter().any(|&m| fb.meets(m))) {
                    continue;
                }
                // The values in the way, in the order met. A register is
                // given up at the first that cannot move (already moved
                // once, or no lighter): a long value would otherwise list
                // every value along it, on every register.
                let mut away: Vec<VReg> = Vec::new();
                let mut met: HashSet<VReg> = HashSet::new();
                let mut cost = 0.0;
                if let Some(os) = owners.get(&p) {
                    for &(a, b) in mine {
                        for (_, &(e, o)) in os.range(..=b).rev() {
                            if e < a {
                                break;
                            }
                            if met.insert(o) {
                                if moved.contains(&o) || weight(o) >= w {
                                    continue 'regs;
                                }
                                away.push(o);
                                cost += weight(o);
                            }
                        }
                    }
                }
                if best.as_ref().is_none_or(|b| cost < b.2) {
                    best = Some((p, away, cost));
                }
            }
            best.map(|(p, away, _)| (p, away))
        });
        match free {
            Some((p, away)) => {
                if !away.is_empty() {
                    // A value's segments met nothing on the register when
                    // they were added (neither fixed stretches nor other
                    // values), so they are still entries of their own, and
                    // taking them away is a lookup each, not a rebuild of
                    // the register's whole map.
                    let os = owners.entry(p).or_default();
                    let busy = taken.entry(p).or_default();
                    for &o in &away {
                        for &s in &segs[o as usize] {
                            os.remove(&s.0);
                        }
                        busy.remove(&segs[o as usize]);
                    }
                    for o in away {
                        assigned[o as usize] = None;
                        moved.insert(o);
                        work.push_back(o);
                    }
                }
                assigned[v as usize] = Some(p);
                let busy = taken.entry(p).or_default();
                let os = owners.entry(p).or_default();
                for &s in mine {
                    busy.add(s);
                    os.insert(s.0, (s.1, v));
                }
            }
            None => spilled.push(v),
        }
    }
    if bail && !spilled.is_empty() {
        return Ok(false);
    }

    // A spilled value is held in a register through stretches of a block
    // where one is free: its appearances in the block, in order, grouped
    // as far as a register stays free from a group's first to its last
    // (caller-saved ones between calls, callee-saved ones across them).
    // In a group of two or more, it is loaded once (if read before it is
    // written) and read from the register; it is stored after each
    // definition, as everywhere, so memory always has it.
    // A spilled value made by one instruction from nothing but immediates
    // and symbols (a constant, an address), which reads no register,
    // memory or flags and changes nothing else, is made again before each use
    // instead: no slot, no store, no loads. (Rematerialisation; each is an
    // elective change, autochop.)
    let remat: HashMap<VReg, MInst> = {
        let mut defs_of: HashMap<VReg, Vec<MInst>> = HashMap::new();
        let want: HashSet<VReg> = spilled.iter().copied().collect();
        for b in &f.blocks {
            for inst in &b.insts {
                let (defs, _) = regs_of(t, inst);
                for r in defs {
                    if let Reg::V(v) = r
                        && want.contains(&v)
                    {
                        defs_of.entry(v).or_default().push(inst.clone());
                    }
                }
            }
        }
        defs_of
            .into_iter()
            .filter_map(|(v, ds)| {
                let [d] = &ds[..] else { return None };
                let Opcode::Instr(k) = d.op else { return None };
                let ins = &t.instrs[k];
                // (Nor one that reads the flags: x86's setcc, whose only
                // operand is its condition.)
                let pure = !ins.memory
                    && !ins.operands.iter().any(|o| o.kind == crate::target::Kind::Cond)
                    && ins.sets.is_none()
                    && !ins.clobbers_flags
                    && ins.clobbers.is_empty()
                    && !ins.second_result
                    && d.imp_defs.is_empty()
                    && d.imp_uses.is_empty()
                    && !d.barrier
                    && d.ops.first() == Some(&MOp::Reg(Reg::V(v)))
                    && d.ops[1..].iter().all(|o| matches!(o, MOp::Imm(_) | MOp::Sym(_)));
                (pure && epitome_ir::chop::allow(|| format!("{}: make v{v} again where it is used", f.name)))
                    .then(|| (v, d.clone()))
            })
            .collect()
    };
    let spilled_set: HashSet<VReg> = spilled.iter().copied().filter(|v| !remat.contains_key(v)).collect();
    let mut local: HashMap<(VReg, usize), Vec<(u32, u32, u8)>> = HashMap::new();
    {
        let mut idx = 0u32;
        for (bi, b) in f.blocks.iter().enumerate() {
            // Each spilled value's positions in the block, in order.
            let mut seen: HashMap<VReg, Vec<u32>> = HashMap::new();
            for inst in &b.insts {
                let (defs, uses) = regs_of(t, inst);
                for (r, p) in uses.iter().map(|r| (r, 2 * idx)).chain(defs.iter().map(|r| (r, 2 * idx + 1))) {
                    if let Reg::V(v) = r
                        && spilled_set.contains(v)
                    {
                        let ps = seen.entry(*v).or_default();
                        if ps.last() != Some(&p) {
                            ps.push(p);
                        }
                    }
                }
                idx += 1;
            }
            let mut vs: Vec<(VReg, Vec<u32>)> = seen.into_iter().filter(|(_, ps)| ps.len() > 1).collect();
            vs.sort_by_key(|(v, ps)| (ps[0], *v));
            for (v, ps) in vs {
                let class = classes[v as usize];
                let wide = too_wide(v);
                let cands: Vec<u8> = caller_saved
                    .iter()
                    .chain(&fp_caller_saved)
                    .chain(callee_saved.iter().chain(&fp_callee_saved).filter(|_| !wide))
                    .copied()
                    .filter(|&p| m.in_class(class, p))
                    .collect();
                let free = |taken: &HashMap<u8, Busy>, p: u8, iv: Interval| taken.get(&p).is_none_or(|b| !b.meets(iv));
                let mut a = 0;
                while a + 1 < ps.len() {
                    // The furthest appearance a register stays free to.
                    let mut best: Option<(usize, u8)> = None;
                    for z in a + 1..ps.len() {
                        let iv = (ps[a], ps[z]);
                        match best
                            .map(|b| b.1)
                            .filter(|&p| free(&taken, p, iv))
                            .or_else(|| cands.iter().copied().find(|&p| free(&taken, p, iv)))
                        {
                            Some(p) => best = Some((z, p)),
                            None => break,
                        }
                    }
                    let Some((z, p)) = best else {
                        a += 1;
                        continue;
                    };
                    if epitome_ir::chop::allow(|| format!("{}: hold a spilled value in a register in a block", f.name))
                    {
                        taken.entry(p).or_default().add((ps[a], ps[z]));
                        local.entry((v, bi)).or_default().push((ps[a], ps[z], p));
                    }
                    a = z + 1;
                }
            }
        }
    }
    // The group of a spilled value's appearance at a position, if any.
    let group_at = |v: VReg, bi: usize, pos: u32| -> Option<(u32, u8)> {
        local.get(&(v, bi))?.iter().find(|g| g.0 <= pos && pos <= g.1).map(|g| (g.0, g.2))
    };

    // Spill slots, and loads and stores around each use and definition.
    let mut slot_of: HashMap<VReg, usize> = HashMap::new();
    for &v in spilled.iter().filter(|v| !remat.contains_key(v)) {
        let size = (m.def(f.vclass[v as usize]).bits as u64 / 8).max(8);
        f.slots.push(Slot { size, align: size, offset: 0 });
        slot_of.insert(v, f.slots.len() - 1);
    }
    // Spill code: a value is spilled as its file's registers are saved
    // (load.x, load.d), or, if wider, by its own class's (load.q), as the
    // description names them by role (those a machine lacks are needed only
    // if values of that kind are spilled).
    let spill_of = |c: crate::target::Class| -> Option<(usize, usize)> {
        let d = m.def(c);
        let as_class = match m.save_class(d.file) {
            Some(s) if m.def(s).bits >= d.bits => s,
            _ => c,
        };
        let n = &m.def(as_class).name;
        t.role(&format!("load.{n}")).zip(t.role(&format!("store.{n}")))
    };
    let vclass = f.vclass.clone();
    let fname = f.name.clone();
    // Each file's scratch registers, for spill code.
    let pool_of = |file: usize| -> Vec<u8> {
        abi.scratch.iter().chain(&abi.fp_scratch).copied().filter(|&r| m.file_of(r) == file).collect()
    };
    // Each instruction's number, as the positions above count them.
    let mut pos = 0u32;
    for (bi, b) in f.blocks.iter_mut().enumerate() {
        let old = std::mem::take(&mut b.insts);
        let mut new = Vec::with_capacity(old.len());
        // Values held in a register, and the group whose register holds
        // them now.
        let mut loaded: HashMap<VReg, u32> = HashMap::new();
        for mut inst in old {
            let here = pos;
            pos += 1;
            let held = |r: Reg| matches!(r, Reg::V(v) if group_at(v, bi, 2 * here).or(group_at(v, bi, 2 * here + 1)).is_some());
            // The definition of a value made again where it is used goes.
            if let Some(MOp::Reg(Reg::V(v))) = inst.ops.first()
                && remat.get(v).is_some_and(|d| d.op == inst.op && d.ops == inst.ops)
            {
                continue;
            }
            // A copy between a spilled value and one in a register is the
            // spill code alone: the register loaded or stored directly.
            if inst.op == Opcode::Copy
                && let [MOp::Reg(d), MOp::Reg(s)] = inst.ops[..]
                && !held(d)
                && !held(s)
            {
                let slot = |r: Reg| match r {
                    Reg::V(v) => slot_of.get(&v).copied(),
                    Reg::P(_) => None,
                };
                let phys = |r: Reg| match r {
                    Reg::P(p) => Some(p),
                    Reg::V(v) if !slot_of.contains_key(&v) => assigned[v as usize],
                    Reg::V(_) => None,
                };
                let direct = match (slot(d), slot(s), d, s) {
                    (Some(sl), None, Reg::V(v), _) => phys(s).map(|p| (sl, v, p, false)),
                    (None, Some(sl), _, Reg::V(v)) => phys(d).map(|p| (sl, v, p, true)),
                    _ => None,
                };
                if let Some((sl, v, p, load)) = direct
                    && m.in_class(vclass[v as usize], p)
                    && !(load && m.zero == Some(p))
                    && let Some((ld, st)) = spill_of(vclass[v as usize])
                {
                    let op = if load { ld } else { st };
                    new.push(MInst::new(
                        op,
                        vec![MOp::Reg(Reg::P(p)), MOp::Reg(Reg::P(t.machine.sp)), MOp::Frame(sl, 0)],
                    ));
                    continue;
                }
            }
            let (defs, uses) = regs_of(t, &inst);
            let mut scratch: HashMap<VReg, u8> = HashMap::new();
            let mut next: HashMap<usize, usize> = HashMap::new();
            let mut after = Vec::new();
            // Operands read first, then results: a result whose scratch
            // registers have run out can share one an operand used, since
            // operands are read before results are written (but not one
            // that is both, such as a written-back base).
            let both: Vec<Reg> = uses.iter().filter(|r| defs.contains(r)).copied().collect();
            for r in uses.iter().chain(defs.iter()) {
                if let Reg::V(v) = r
                    && (slot_of.contains_key(v) || remat.contains_key(v))
                    && !scratch.contains_key(v)
                {
                    let slot = slot_of.get(v).copied().unwrap_or(0);
                    let class = vclass[*v as usize];
                    let at = if uses.contains(r) { 2 * here } else { 2 * here + 1 };
                    if let Some((g, p)) = group_at(*v, bi, at) {
                        let (ld, st) = spill_of(class).ok_or_else(|| {
                            format!(
                                "{fname}: no instructions load and store a {} register to spill it",
                                m.def(class).name
                            )
                        })?;
                        scratch.insert(*v, p);
                        if uses.contains(r) && loaded.get(v) != Some(&g) {
                            new.push(MInst::new(
                                ld,
                                vec![MOp::Reg(Reg::P(p)), MOp::Reg(Reg::P(t.machine.sp)), MOp::Frame(slot, 0)],
                            ));
                        }
                        if defs.contains(r) {
                            after.push(MInst::new(
                                st,
                                vec![MOp::Reg(Reg::P(p)), MOp::Reg(Reg::P(t.machine.sp)), MOp::Frame(slot, 0)],
                            ));
                        }
                        loaded.insert(*v, g);
                        continue;
                    }
                    let pool = pool_of(m.def(class).file);
                    let k = next.entry(m.def(class).file).or_default();
                    let s = match pool.get(*k) {
                        Some(&s) => {
                            *k += 1;
                            s
                        }
                        None if !uses.contains(r) => *pool
                            .iter()
                            .find(|&&s| !both.iter().any(|b| matches!(b, Reg::V(w) if scratch.get(w) == Some(&s))))
                            .ok_or_else(|| format!("{fname}: more spilled operands than scratch registers"))?,
                        None => return Err(format!("{fname}: more spilled operands than scratch registers")),
                    };
                    scratch.insert(*v, s);
                    if let Some(d) = remat.get(v) {
                        if uses.contains(r) {
                            let mut again = d.clone();
                            again.ops[0] = MOp::Reg(Reg::P(s));
                            new.push(again);
                        }
                        continue;
                    }
                    let (ld, st) = spill_of(class).ok_or_else(|| {
                        format!("{fname}: no instructions load and store a {} register to spill it", m.def(class).name)
                    })?;
                    if uses.contains(r) {
                        new.push(MInst::new(
                            ld,
                            vec![MOp::Reg(Reg::P(s)), MOp::Reg(Reg::P(t.machine.sp)), MOp::Frame(slot, 0)],
                        ));
                    }
                    if defs.contains(r) {
                        after.push(MInst::new(
                            st,
                            vec![MOp::Reg(Reg::P(s)), MOp::Reg(Reg::P(t.machine.sp)), MOp::Frame(slot, 0)],
                        ));
                    }
                }
            }
            for op in &mut inst.ops {
                if let MOp::Reg(Reg::V(v)) = op {
                    let class = vclass[*v as usize];
                    *op = MOp::Reg(Reg::P(match scratch.get(v) {
                        Some(&s) => s,
                        None => match assigned[*v as usize] {
                            Some(p) => p,
                            // A result nothing reads: the zero register
                            // takes it, or a scratch register of its file
                            // that spill code is not using.
                            None if m.zero_in(class) => m.zero.unwrap(),
                            None => {
                                let pool = pool_of(m.def(class).file);
                                *pool.iter().rev().find(|r| !scratch.values().any(|s| s == *r)).ok_or_else(|| {
                                    format!("{fname}: no register for an unused {} result", m.def(class).name)
                                })?
                            }
                        },
                    }));
                }
            }
            new.push(inst);
            new.extend(after);
        }
        b.insts = new;
    }
    let spill_ops: Vec<(usize, usize)> =
        (0..m.classes.len()).filter_map(|c| spill_of(crate::target::Class(c as u8))).collect();
    let slots: HashSet<usize> = slot_of.values().copied().collect();
    for b in &mut f.blocks {
        reused_reloads(t, &mut b.insts, &slots, &spill_ops, &fname);
    }
    redundant_copies(t, f);
    lower_copies(t, f).map(|_| true)
}

/// Coalescing: the two sides of a copy between virtual registers of one
/// class whose live ranges never meet (the source's last read the copy,
/// the destination first written there) become one register, so the copy
/// copies a register to itself and goes, and allocation places one value
/// where it would have placed two, perhaps apart. Each merge is an
/// elective change (autochop).
fn coalesce(f: &mut MFunc, segs: &mut [Vec<Interval>], hints: &mut [Vec<Reg>], occ: &mut [u32]) {
    let n = segs.len();
    let mut parent: Vec<VReg> = (0..n as VReg).collect();
    fn root(parent: &mut [VReg], mut v: VReg) -> VReg {
        while parent[v as usize] != v {
            parent[v as usize] = parent[parent[v as usize] as usize];
            v = parent[v as usize];
        }
        v
    }
    // Whether two sorted lists of closed intervals meet.
    let meet = |a: &[Interval], b: &[Interval]| {
        let (mut i, mut j) = (0, 0);
        while i < a.len() && j < b.len() {
            if a[i].1 < b[j].0 {
                i += 1;
            } else if b[j].1 < a[i].0 {
                j += 1;
            } else {
                return true;
            }
        }
        false
    };
    let name = f.name.clone();
    let mut merged = false;
    for b in &f.blocks {
        for inst in &b.insts {
            let (Opcode::Copy, [MOp::Reg(Reg::V(d)), MOp::Reg(Reg::V(s))]) = (inst.op, &inst.ops[..]) else { continue };
            let (d, s) = (root(&mut parent, *d), root(&mut parent, *s));
            if d == s
                || f.vclass[d as usize] != f.vclass[s as usize]
                || segs[d as usize].is_empty()
                || segs[s as usize].is_empty()
                || meet(&segs[d as usize], &segs[s as usize])
                || !epitome_ir::chop::allow(|| format!("{name}: coalesce the copy v{d} = v{s}"))
            {
                continue;
            }
            let mut all = std::mem::take(&mut segs[s as usize]);
            all.extend(segs[d as usize].iter().copied());
            all.sort();
            segs[d as usize] = all;
            let hs = std::mem::take(&mut hints[s as usize]);
            hints[d as usize].extend(hs);
            occ[d as usize] += occ[s as usize];
            parent[s as usize] = d;
            merged = true;
        }
    }
    if !merged {
        return;
    }
    let roots: Vec<VReg> = (0..n as VReg).map(|v| root(&mut parent, v)).collect();
    for b in &mut f.blocks {
        for inst in &mut b.insts {
            for op in &mut inst.ops {
                if let MOp::Reg(Reg::V(v)) = op {
                    *v = roots[*v as usize];
                }
            }
        }
    }
    for h in hints.iter_mut() {
        for r in h.iter_mut() {
            if let Reg::V(v) = r {
                *v = roots[*v as usize];
            }
        }
    }
}

/// Reloads of a spill slot into a register that already holds it, from an
/// earlier load or store of the slot in the block with nothing written to
/// either since: removed (an elective change, autochop). Scratch registers
/// held this way are read later in the block, which frame lowering sees
/// when it looks for a free one.
fn reused_reloads(t: &Target, insts: &mut Vec<MInst>, slots: &HashSet<usize>, ops: &[(usize, usize)], name: &str) {
    let sp = Reg::P(t.machine.sp);
    // (register, slot, whether a load) of a spill load or store.
    let access = |i: &MInst| match (i.op, &i.ops[..]) {
        (Opcode::Instr(k), [MOp::Reg(r @ Reg::P(_)), MOp::Reg(base), MOp::Frame(slot, 0)])
            if *base == sp && slots.contains(slot) =>
        {
            ops.iter().find_map(|&(ld, st)| {
                if k == ld {
                    Some((*r, *slot, true))
                } else if k == st {
                    Some((*r, *slot, false))
                } else {
                    None
                }
            })
        }
        _ => None,
    };
    let mut holds: HashMap<Reg, usize> = HashMap::new();
    let mut keep = Vec::with_capacity(insts.len());
    for inst in std::mem::take(insts) {
        match access(&inst) {
            Some((r, slot, true)) => {
                if holds.get(&r) == Some(&slot)
                    && epitome_ir::chop::allow(|| format!("{name}: reuse a reload of a spill slot"))
                {
                    continue;
                }
                holds.insert(r, slot);
            }
            Some((r, slot, false)) => {
                holds.retain(|_, s| *s != slot);
                holds.insert(r, slot);
            }
            None if matches!(inst.op, Opcode::Asm { .. }) => holds.clear(),
            None => {
                let (defs, _) = regs_of(t, &inst);
                holds.retain(|r, _| !defs.contains(r));
            }
        }
        keep.push(inst);
    }
    *insts = keep;
}

/// Copies that change nothing, once registers are physical. In a block,
/// each register a copy wrote holds its source's value (in the copy's
/// class) until either is written: a copy from such a register copies
/// from its source instead, and a copy between registers holding the same
/// value goes. Then a copy whose register is written again in the block
/// before anything reads it goes, as does a constant's. And a copy of a
/// constant made in one instruction makes it again in place, so the first
/// often goes. (Phi copies, sequenced through spare registers, and copies
/// around calls and landing pads make all of these.)
/// Each change is an elective one (autochop). Returns how many copies went.
fn redundant_copies(t: &Target, f: &mut MFunc) -> usize {
    let name = f.name.clone();
    let regs = |i: &MInst| match (i.op, &i.ops[..]) {
        (Opcode::Copy, [MOp::Reg(d @ Reg::P(_)), MOp::Reg(s @ Reg::P(_))]) => Some((*d, *s)),
        _ => None,
    };
    // A constant made in one instruction from immediates alone (`mov x0,
    // #1`): its register.
    let konst = |i: &MInst| match (i.op, i.ops.first()) {
        (Opcode::Instr(k), Some(MOp::Reg(d @ Reg::P(_))))
            if t.constants.contains(&k)
                && t.instrs[k].sets.is_none()
                && i.imp_defs.is_empty()
                && i.ops[1..].iter().all(|o| matches!(o, MOp::Imm(_))) =>
        {
            Some(*d)
        }
        _ => None,
    };
    let m = &t.machine;
    let mut removed = 0;
    for b in &mut f.blocks {
        // Register -> the register whose value it holds, and the class.
        let mut of: HashMap<Reg, (Reg, Option<crate::target::Class>)> = HashMap::new();
        // Register -> the instruction that made the constant it holds.
        let mut made: HashMap<Reg, MInst> = HashMap::new();
        let mut keep = Vec::with_capacity(b.insts.len());
        for mut inst in std::mem::take(&mut b.insts) {
            // A copy of a constant is the constant made again in place.
            if let Some((Reg::P(d), Reg::P(s))) = regs(&inst)
                && m.file_of(d) == m.file_of(s)
                && let Some(k) = made.get(&Reg::P(s))
                && epitome_ir::chop::allow(|| format!("{name}: make a copied constant in place"))
            {
                let mut k = k.clone();
                k.ops[0] = MOp::Reg(Reg::P(d));
                inst = k;
            }
            if let Some((d, s)) = regs(&inst) {
                let src = match of.get(&s) {
                    Some(&(r, c)) if c == inst.class => r,
                    _ => s,
                };
                let holds = |r: Reg| r == src || of.get(&r).is_some_and(|&(x, c)| x == src && c == inst.class);
                if holds(d)
                    && epitome_ir::chop::allow(|| format!("{name}: remove a copy of registers already the same"))
                {
                    removed += 1;
                    continue;
                }
                if src != s && epitome_ir::chop::allow(|| format!("{name}: copy from the copy's source")) {
                    inst.ops[1] = MOp::Reg(src);
                }
            }
            if matches!(inst.op, Opcode::Asm { .. }) {
                of.clear();
            }
            let (defs, _) = regs_of(t, &inst);
            of.retain(|r, (x, _)| !defs.contains(r) && !defs.contains(x));
            made.retain(|r, _| !defs.contains(r));
            if let Some((d, s)) = regs(&inst)
                && d != s
            {
                of.insert(d, (s, inst.class));
            }
            if let Some(d) = konst(&inst) {
                made.insert(d, inst.clone());
            }
            keep.push(inst);
        }
        let mut dead = vec![false; keep.len()];
        for k in 0..keep.len() {
            let d = match (regs(&keep[k]), konst(&keep[k])) {
                (Some((d, s)), _) if d != s => d,
                (None, Some(d)) => d,
                _ => continue,
            };
            for later in &keep[k + 1..] {
                let (defs, uses) = regs_of(t, later);
                if uses.contains(&d) || matches!(later.op, Opcode::Asm { .. }) {
                    break;
                }
                if defs.contains(&d) {
                    dead[k] = epitome_ir::chop::allow(|| {
                        format!("{name}: remove a copy or constant written over before it is read")
                    });
                    break;
                }
            }
        }
        removed += dead.iter().filter(|&&x| x).count();
        let mut k = 0;
        keep.retain(|_| {
            k += 1;
            !dead[k - 1]
        });
        b.insts = keep;
    }
    removed
}

/// Turns copies into moves, dropping those between the same register:
/// within a file, by the copy of the class (or of a wider one of its file:
/// AArch64 copies 64-bit vectors as d registers); between files, by the
/// instruction with role `move.<to>.<from>` (`move.d.x`, `fmov d0, x1`).
fn lower_copies(t: &Target, f: &mut MFunc) -> Result<(), String> {
    let m = &t.machine;
    for b in &mut f.blocks {
        let old = std::mem::take(&mut b.insts);
        for inst in old {
            if inst.op != Opcode::Copy {
                b.insts.push(inst);
                continue;
            }
            let (MOp::Reg(Reg::P(d)), MOp::Reg(Reg::P(s))) = (&inst.ops[0], &inst.ops[1]) else {
                b.insts.push(inst);
                continue;
            };
            if d == s {
                continue;
            }
            let (fd, fs) = (m.file_of(*d), m.file_of(*s));
            let class = inst.class.or_else(|| m.save_class(fd)).ok_or("a copy of no class")?;
            let bits = m.def(class).bits;
            let op = if fd == fs {
                let mut within: Vec<(u32, usize)> = t
                    .copies
                    .iter()
                    .filter(|(c, _)| m.def(**c).file == fd && !m.def(**c).sp && m.def(**c).bits >= bits)
                    .map(|(c, &i)| (m.def(*c).bits, i))
                    .collect();
                within.sort();
                within
                    .first()
                    .map(|w| w.1)
                    .ok_or_else(|| format!("{}: no \"copy\" instruction for {} registers", f.name, m.files[fd].name))?
            } else {
                let name = |file: usize| m.scalar_in(file, bits).map(|c| m.def(c).name.clone());
                let (Some(to), Some(from)) = (name(fd), name(fs)) else {
                    return Err(format!("{}: no {bits}-bit class to copy between register files", f.name));
                };
                t.role(&format!("move.{to}.{from}"))
                    .ok_or_else(|| format!("{}: no instruction has role move.{to}.{from}", f.name))?
            };
            b.insts.push(MInst::new(op, vec![MOp::Reg(Reg::P(*d)), MOp::Reg(Reg::P(*s))]));
        }
    }
    Ok(())
}

/// Records each copy's class before allocation, while its virtual
/// registers still say.
fn record_copy_classes(f: &mut MFunc) {
    let vclass = f.vclass.clone();
    for b in &mut f.blocks {
        for inst in &mut b.insts {
            if inst.op != Opcode::Copy {
                continue;
            }
            let c = |o: &MOp| match o {
                MOp::Reg(Reg::V(v)) => Some(vclass[*v as usize]),
                _ => None,
            };
            inst.class = c(&inst.ops[0]).or(c(&inst.ops[1]));
        }
    }
}
