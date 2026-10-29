//! Frames: laying out stack slots, saving registers, and tidying branches.
//!
//! The frame, from `sp` up: locals and spill slots, then the callee-saved
//! registers the function uses, then the caller's frame pointer and the
//! return address, where `x29` points. Its size is a multiple of 16.
//!
//! A function that makes no calls keeps its return address in `x30` and
//! needs no frame record; with no slots and no saved registers either, it
//! has no frame at all.

use crate::mir::*;
use crate::target::Target;
use crate::unwind::Cfi;

/// Ends the error of a frame that needed a scratch register when the
/// allocator had given them all to values (it does when nothing spills):
/// the function is allocated again with them kept free.
pub const NEEDS_SCRATCH: &str = " (the scratch registers were allocated)";

pub fn lower(t: &Target, f: &mut MFunc) -> Result<(), String> {
    let (fp_reg, lr_reg) = (f.abi.fp, f.abi.lr);
    let mut callee_saved = f.abi.callee_saved();
    callee_saved.extend(f.abi.fp_callee_saved());
    // Callee-saved registers written anywhere.
    let mut saved: Vec<u8> = Vec::new();
    for b in &f.blocks {
        for inst in &b.insts {
            let (defs, _) = inst.defs_uses(t);
            let written =
                defs.iter().filter_map(|&k| if let MOp::Reg(Reg::P(r)) = inst.ops[k] { Some(r) } else { None });
            // And those inline assembly and instructions clobber (a call's
            // clobbers are the caller-saved registers, so add nothing).
            let clobbered = inst.imp_defs.clone();
            for r in written.chain(clobbered) {
                if callee_saved.contains(&r) && !saved.contains(&r) {
                    saved.push(r);
                }
            }
        }
    }
    // Where sp moves, the frame is addressed from the base register, which
    // is saved like the others.
    let base = if f.dynamic_stack {
        let b = f.abi.base.ok_or_else(|| format!("{}: sp moves, and the convention names no base register", f.name))?;
        if !saved.contains(&b) {
            saved.push(b);
        }
        Some(b)
    } else {
        None
    };
    saved.sort();
    // Arguments for calls go at the bottom, where the callee finds them;
    // then the slots, smallest first, so that the most of them (spill
    // slots, small locals) are near enough sp for pairs' offsets.
    let mut off: i64 = f.out_size;
    let mut order: Vec<usize> = (0..f.slots.len()).collect();
    order.sort_by_key(|&k| (f.slots[k].size, k));
    for k in order {
        let s = &mut f.slots[k];
        let a = s.align as i64;
        off = (off + a - 1) / a * a;
        s.offset = off;
        off += s.size as i64;
    }
    off = (off + 7) / 8 * 8;
    let save_at: Vec<(u8, i64)> = saved.iter().enumerate().map(|(k, &r)| (r, off + 8 * k as i64)).collect();
    off += 8 * saved.len() as i64;
    let calls = f.blocks.iter().any(|b| b.insts.iter().any(|i| i.is_call(t)));
    // With the return address on the stack (x86), the call pushed it: no
    // record, and the frame keeps sp aligned below its 8 bytes.
    let ra = f.abi.ra_on_stack;
    let record = if calls && !ra { 16 } else { 0 };
    let align = f.abi.stack_align;
    let size = if !ra {
        (off + record + align - 1) / align * align
    } else if off == 0 && !calls {
        0
    } else {
        (off + 8 + align - 1) / align * align - 8
    };
    let fp_at = size - 16;
    f.frame_size = size;
    // Where the CFA (sp on entry, before the call pushed the return
    // address) is from sp after the frame is made.
    let cfa = size + if ra { 8 } else { 0 };

    // Frame offsets become immediates. Where one is more than the
    // instruction's immediate holds (RISC-V's reach 2047 bytes), the
    // address is made in a scratch register the instruction does not use:
    // the offset, added to the base; the instruction then uses offset 0.
    let far = f.abi.far_scratch;
    for b in &mut f.blocks {
        let mut old = std::mem::take(&mut b.insts);
        for j in 0..old.len() {
            let mut inst = std::mem::replace(&mut old[j], MInst::new(0, vec![]));
            let mut at = None;
            for (k, op) in inst.ops.iter_mut().enumerate() {
                match op {
                    MOp::Frame(s, o) => {
                        *op = MOp::Imm(f.slots[*s].offset + *o);
                        at = Some(k);
                    }
                    MOp::Incoming(o) => {
                        *op = MOp::Imm(cfa + *o);
                        at = Some(k);
                    }
                    MOp::OutArea => *op = MOp::Imm(f.out_size),
                    _ => {}
                }
            }
            if let (Some(k), Some(b)) = (at, base)
                && k > 0
                && inst.ops[k - 1] == MOp::Reg(Reg::P(t.machine.sp))
            {
                inst.ops[k - 1] = MOp::Reg(Reg::P(b));
            }
            if let (Some(k), Opcode::Instr(i)) = (at, inst.op)
                && k > 0
                && let MOp::Imm(off) = inst.ops[k]
                && let MOp::Reg(Reg::P(base)) = inst.ops[k - 1]
            {
                let values: Vec<Option<i64>> =
                    inst.ops.iter().map(|o| if let MOp::Imm(v) = o { Some(*v) } else { None }).collect();
                if !t.fits_cached(i, values) {
                    // A spill scratch register free here, else the
                    // convention's far-scratch register.
                    let tmp = free_scratch(t, &f.abi.scratch, &inst, &old[j + 1..]).or(far).ok_or_else(|| {
                        format!(
                            "{}: a frame offset of {off}, no scratch register free, and the convention names no far-scratch register{NEEDS_SCRATCH}",
                            f.name
                        )
                    })?;
                    // (An add that may take sp, where plain "add" cannot.)
                    let add = t
                        .role("far-add")
                        .or_else(|| t.role("add"))
                        .ok_or("no \"add\" instruction for a far frame offset")?;
                    constant(t, &mut b.insts, tmp, off)?;
                    b.insts.push(MInst::new(
                        add,
                        vec![MOp::Reg(Reg::P(tmp)), MOp::Reg(Reg::P(base)), MOp::Reg(Reg::P(tmp))],
                    ));
                    inst.ops[k - 1] = MOp::Reg(Reg::P(tmp));
                    inst.ops[k] = MOp::Imm(0);
                }
            }
            b.insts.push(inst);
        }
    }

    // Where the return address is on the stack (x86) and the description
    // pushes and pops the saved registers (roles push.<class> and
    // pop.<class>), they are pushed in order, just below it, and popped in
    // the reverse order, and the rest of the frame made by one adjustment
    // of sp: a byte or two each, where a store or load at an offset takes
    // five or more. (The frame's size is the same: the saves are its top.)
    let push = |r: u8| t.role(&format!("push.{}", t.machine.save_name(r)));
    let pop = |r: u8| t.role(&format!("pop.{}", t.machine.save_name(r)));
    let pushing = ra && !saved.is_empty() && saved.iter().all(|&r| push(r).is_some() && pop(r).is_some());
    let cfi = |c: Cfi| MInst { op: Opcode::Cfi(c), ..MInst::new(0, vec![]) };
    let sp = t.machine.sp;
    let (prologue, epilogue) = if pushing {
        let n = saved.len() as i64;
        let rest = size - 8 * n;
        let mut prologue = Vec::new();
        for (k, &r) in saved.iter().enumerate() {
            let k = k as i64;
            prologue.push(MInst::new(push(r).unwrap(), vec![MOp::Reg(Reg::P(r))]));
            prologue.push(cfi(Cfi::DefCfaOffset(8 + 8 * (k + 1))));
            prologue.push(cfi(Cfi::Offset(r, -16 - 8 * k)));
        }
        if rest > 0 {
            adjust(t, &mut prologue, sp, sp, -rest, &f.abi.scratch)?;
            prologue.push(cfi(Cfi::DefCfaOffset(cfa)));
        }
        let mut epilogue = Vec::new();
        if let Some(b) = base {
            let addi = t.by_role("addi");
            prologue.push(MInst::new(addi, vec![MOp::Reg(Reg::P(b)), MOp::Reg(Reg::P(sp)), MOp::Imm(0)]));
            prologue.push(cfi(Cfi::DefCfa(b, cfa)));
            epilogue.push(MInst::new(addi, vec![MOp::Reg(Reg::P(sp)), MOp::Reg(Reg::P(b)), MOp::Imm(0)]));
        }
        if rest > 0 {
            adjust(t, &mut epilogue, sp, sp, rest, &f.abi.scratch)?;
        }
        for &r in saved.iter().rev() {
            epilogue.push(MInst::new(pop(r).unwrap(), vec![MOp::Reg(Reg::P(r))]));
        }
        (prologue, epilogue)
    } else {
        // Registers saved and restored two at a time where they can be: two of
        // the same file at neighbouring slots, at an offset a pair reaches.
        let mut slots: Vec<(u8, i64)> = Vec::new();
        if calls && !ra {
            slots.push((fp_reg, fp_at));
            slots.push((lr_reg, fp_at + 8));
        }
        slots.extend(save_at.iter().copied());
        // Where the saves are too far from sp for a store's offset (RISC-V's
        // reach 2047 bytes, AArch64's pairs 504), the frame is made in two
        // steps: the part above the lowest save first, with the saves near sp,
        // then the rest.
        let reaches = |slots: &[(u8, i64)]| {
            slots.iter().all(|&(reg, at)| {
                let role = format!("store.{}", t.machine.save_name(reg));
                t.role(&role).is_some_and(|i| t.instrs[i].fits(&[None, None, Some(at)]))
            })
        };
        // And where pairs would not reach them, with enough saves that pairs
        // save more than the second step costs.
        let pairs_reach = |slots: &[(u8, i64)]| {
            slots.iter().all(|&(reg, at)| {
                let role = format!("store.pair.{}", t.machine.save_name(reg));
                t.role(&role).is_none_or(|i| t.instrs[i].fits(&[None, None, None, Some(at)]))
            })
        };
        let low = if reaches(&slots) && (slots.len() < 4 || pairs_reach(&slots)) {
            0
        } else {
            slots.iter().map(|s| s.1).min().unwrap_or(0) / align * align
        };
        let near: Vec<(u8, i64)> = slots.iter().map(|&(reg, at)| (reg, at - low)).collect();
        let (saves, restores) = save_restore(t, &near);
        // The unwind rules (unwind.rs) go after what they describe: where the
        // CFA (sp on entry) is after each move of sp, and where each register
        // was saved, from the CFA.
        let mut prologue = Vec::new();
        let extra = cfa - size;
        if size - low > 0 {
            adjust(t, &mut prologue, sp, sp, -(size - low), &f.abi.scratch)?;
            prologue.push(cfi(Cfi::DefCfaOffset(size - low + extra)));
        }
        prologue.extend(saves);
        prologue.extend(slots.iter().map(|&(reg, at)| cfi(Cfi::Offset(reg, at - cfa))));
        if calls && !ra {
            adjust(t, &mut prologue, fp_reg, sp, fp_at - low, &f.abi.scratch)?;
        }
        if low > 0 {
            adjust(t, &mut prologue, sp, sp, -low, &f.abi.scratch)?;
            prologue.push(cfi(Cfi::DefCfaOffset(cfa)));
        }
        let mut epilogue = Vec::new();
        if let Some(b) = base {
            let addi = t.by_role("addi");
            prologue.push(MInst::new(addi, vec![MOp::Reg(Reg::P(b)), MOp::Reg(Reg::P(sp)), MOp::Imm(0)]));
            prologue.push(cfi(Cfi::DefCfa(b, cfa)));
            epilogue.push(MInst::new(addi, vec![MOp::Reg(Reg::P(sp)), MOp::Reg(Reg::P(b)), MOp::Imm(0)]));
        }
        if low > 0 {
            adjust(t, &mut epilogue, sp, sp, low, &f.abi.scratch)?;
        }
        epilogue.extend(restores);
        if size - low > 0 {
            adjust(t, &mut epilogue, sp, sp, size - low, &f.abi.scratch)?;
        }
        (prologue, epilogue)
    };

    // Before each return, and each tail call (whose callee returns to this
    // function's caller).
    let leaves = |inst: &MInst| matches!(inst.op, Opcode::Instr(k) if t.has_role(k, "ret") || t.has_role(k, "tailcall") || t.has_role(k, "tailcallr"));
    for b in &mut f.blocks {
        let old = std::mem::take(&mut b.insts);
        for inst in old {
            if leaves(&inst) {
                b.insts.extend(epilogue.iter().cloned());
            }
            b.insts.push(inst);
        }
    }
    let entry = &mut f.blocks[0].insts;
    entry.splice(0..0, prologue);
    tidy_branches(t, f);
    Ok(())
}

/// The stores that save registers at their offsets from `sp`, and the loads
/// that restore them: `stp` and `ldp` for two of the same register file at
/// neighbouring slots, if a pair's offset can hold it; else `str` and
/// `ldr`.
fn save_restore(t: &Target, slots: &[(u8, i64)]) -> (Vec<MInst>, Vec<MInst>) {
    let r = |n: u8| MOp::Reg(Reg::P(n));
    let file = |reg: u8| t.machine.save_name(reg);
    let (mut saves, mut restores) = (Vec::new(), Vec::new());
    let mut k = 0;
    while k < slots.len() {
        let (a, at) = slots[k];
        if let Some(&(b, bt)) = slots.get(k + 1)
            && file(a) == file(b)
            && bt == at + 8
        {
            let pair = t.role(&format!("store.pair.{}", file(a))).zip(t.role(&format!("load.pair.{}", file(a))));
            if let Some((stp, ldp)) = pair
                && t.instrs[stp].fits(&[None, None, None, Some(at)])
            {
                saves.push(MInst::new(stp, vec![r(a), r(b), r(t.machine.sp), MOp::Imm(at)]));
                restores.push(MInst::new(ldp, vec![r(a), r(b), r(t.machine.sp), MOp::Imm(at)]));
                k += 2;
                continue;
            }
        }
        let st = t.by_role(&format!("store.{}", file(a)));
        let ld = t.by_role(&format!("load.{}", file(a)));
        saves.push(MInst::new(st, vec![r(a), r(t.machine.sp), MOp::Imm(at)]));
        restores.push(MInst::new(ld, vec![r(a), r(t.machine.sp), MOp::Imm(at)]));
        k += 1;
    }
    (saves, restores)
}

/// `dst = src + amount`, for a frame's sizes: one add (or subtract) of an
/// immediate if the description has one that holds it; else, where it has
/// a shifted one (`addi12`, AArch64's `add #imm, lsl #12`), that for the
/// high part and the plain one for the rest, the split being where the
/// shifted one's smallest step is; else the amount made in a scratch
/// register and added.
fn adjust(t: &Target, out: &mut Vec<MInst>, dst: u8, src: u8, amount: i64, scratch: &[u8]) -> Result<(), String> {
    let r = |n: u8| MOp::Reg(Reg::P(n));
    let fits = |role: &str, v: i64| t.role(role).filter(|&i| t.instrs[i].fits(&[None, None, Some(v)]));
    // sp moved by itself, in a frame (where the flags are dead): by the
    // description's own instructions for that, where it has them (x86's
    // subq and addq, shorter than leaq).
    if dst == src {
        let own = if amount < 0 { fits("frame.subi", -amount) } else { fits("frame.addi", amount) };
        if let Some(i) = own {
            out.push(MInst::new(i, vec![r(dst), r(src), MOp::Imm(amount.abs())]));
            return Ok(());
        }
    }
    if let Some(i) = fits("addi", amount) {
        out.push(MInst::new(i, vec![r(dst), r(src), MOp::Imm(amount)]));
        return Ok(());
    }
    if amount < 0
        && let Some(i) = fits("subi", -amount)
    {
        out.push(MInst::new(i, vec![r(dst), r(src), MOp::Imm(-amount)]));
        return Ok(());
    }
    let (op, op12) = if amount < 0 { (t.role("subi"), t.role("subi12")) } else { (t.role("addi"), t.role("addi12")) };
    let a = amount.abs();
    let fit = |i: usize, v: i64| t.instrs[i].fits(&[None, None, Some(v)]);
    // The shifted add's step: the least power of two it holds.
    let split = op.zip(op12).and_then(|(op, op12)| {
        let step = (0..40).map(|k| 1i64 << k).find(|&s| fit(op12, s))?;
        let (high, low) = (a & !(step - 1), a & (step - 1));
        (fit(op12, high) || high == 0).then_some(()).filter(|_| fit(op, low) || low == 0).map(|_| (op, op12, high, low))
    });
    let Some((op, op12, pages, rest)) = split else {
        // Else the amount, made in a scratch register, added.
        let scratch =
            *scratch.first().ok_or_else(|| format!("no scratch register for a large frame{NEEDS_SCRATCH}"))?;
        constant(t, out, scratch, amount)?;
        let add = t.role("add").ok_or("no \"add\" instruction for a large frame")?;
        out.push(MInst::new(add, vec![r(dst), r(src), r(scratch)]));
        return Ok(());
    };
    let mut from = src;
    if pages != 0 {
        out.push(MInst::new(op12, vec![r(dst), r(from), MOp::Imm(pages)]));
        from = dst;
    }
    if rest != 0 || pages == 0 {
        out.push(MInst::new(op, vec![r(dst), r(from), MOp::Imm(rest)]));
    }
    Ok(())
}

/// Integer `v` in register `reg`, for frame sizes: by one of the
/// description's constant instructions if it holds it, else a chunk at a
/// time where it has move-wide instructions (AArch64's `movz`, `movk`),
/// else by one for the high part and an add-immediate (`"addi"`) for the
/// low 12 bits.
fn constant(t: &Target, out: &mut Vec<MInst>, reg: u8, v: i64) -> Result<(), String> {
    let r = |n: u8| MOp::Reg(Reg::P(n));
    let wide = |ci: &usize| t.instrs[*ci].operands[0].bits == 64;
    // (Whether an instruction holds a value is cached: frames with many far
    // slots ask it for each access.)
    if let Some(&ci) = t.constants.iter().filter(|ci| wide(ci)).find(|&&ci| t.fits_cached(ci, vec![None, Some(v)])) {
        out.push(MInst::new(ci, vec![r(reg), MOp::Imm(v)]));
        return Ok(());
    }
    if let Some((movz, movk, chunk)) = crate::isel::move_wide_of(t, true) {
        let mask = (1u64 << chunk) - 1;
        let chunks: Vec<(i64, i64)> = (0..64u32.div_ceil(chunk))
            .map(|k| ((((v as u64) >> (chunk * k)) & mask) as i64, (chunk * k) as i64))
            .filter(|&(c, _)| c != 0)
            .collect();
        let Some((first, rest)) = chunks.split_first() else {
            return Err(format!("no way to make the frame size {v}"));
        };
        out.push(MInst::new(movz, vec![r(reg), MOp::Imm(first.0), MOp::Imm(first.1)]));
        for &(c, sh) in rest {
            out.push(MInst::new(movk, vec![r(reg), r(reg), MOp::Imm(c), MOp::Imm(sh)]));
        }
        return Ok(());
    }
    let add = t.role("addi").ok_or("no \"addi\" instruction")?;
    let lo = (v << 52) >> 52;
    let hi = v - lo;
    let ci = t
        .constants
        .iter()
        .copied()
        .filter(|ci| wide(ci))
        .find(|&ci| t.fits_cached(ci, vec![None, Some(hi)]))
        .ok_or_else(|| format!("no way to make the frame size {v}"))?;
    if !t.instrs[add].fits(&[None, None, Some(lo)]) {
        return Err(format!("no way to make the frame size {v}"));
    }
    out.push(MInst::new(ci, vec![r(reg), MOp::Imm(hi)]));
    out.push(MInst::new(add, vec![r(reg), r(reg), MOp::Imm(lo)]));
    Ok(())
}

/// A block holding only `b X` is skipped: branches to it go to `X`. Blocks
/// no longer reached are removed, and branches to the next block dropped.
fn tidy_branches(t: &Target, f: &mut MFunc) {
    let b_op = Opcode::Instr(t.by_role("jump"));
    let ends_flow = |t: &Target, b: &MBlock| {
        b.insts.last().is_some_and(|i| match i.op {
            Opcode::Instr(k) => {
                i.noreturn
                    || t.has_role(k, "jump")
                    || t.has_role(k, "ret")
                    || t.has_role(k, "trap")
                    || t.has_role(k, "tailcall")
                    || t.has_role(k, "tailcallr")
            }
            _ => false,
        })
    };
    // Skip blocks that only branch.
    let n = f.blocks.len();
    let mut forward: Vec<usize> = (0..n).collect();
    for (i, b) in f.blocks.iter().enumerate() {
        if let [only] = b.insts.as_slice()
            && only.op == b_op
            && let MOp::Label(x) = only.ops[0]
            && x != i
        {
            forward[i] = x;
        }
    }
    let resolve = |mut i: usize| {
        for _ in 0..n {
            if forward[i] == i {
                break;
            }
            i = forward[i];
        }
        i
    };
    for b in &mut f.blocks {
        for inst in &mut b.insts {
            for op in &mut inst.ops {
                if let MOp::Label(x) = op {
                    *x = resolve(*x);
                }
            }
        }
    }
    // Remove blocks nothing reaches.
    let mut reached = vec![false; n];
    reached[0] = true;
    for i in 0..n {
        for inst in &f.blocks[i].insts {
            for op in &inst.ops {
                if let MOp::Label(x) = op {
                    reached[*x] = true;
                }
            }
        }
        if i + 1 < n && !ends_flow(t, &f.blocks[i]) {
            reached[i + 1] = true;
        }
    }
    let mut new_index = vec![0usize; n];
    let mut k = 0;
    for i in 0..n {
        new_index[i] = k;
        if reached[i] {
            k += 1;
        }
    }
    let old = std::mem::take(&mut f.blocks);
    for (i, mut b) in old.into_iter().enumerate() {
        if !reached[i] {
            continue;
        }
        for inst in &mut b.insts {
            for op in &mut inst.ops {
                if let MOp::Label(x) = op {
                    *x = new_index[*x];
                }
            }
        }
        b.succs = b.succs.iter().map(|&s| new_index[s]).collect();
        f.blocks.push(b);
    }
    // A branch to the next block is a fall-through.
    let nb = f.blocks.len();
    for i in 0..nb {
        let drop = f.blocks[i].insts.last().is_some_and(|last| last.op == b_op && last.ops[0] == MOp::Label(i + 1));
        if drop {
            f.blocks[i].insts.pop();
        }
    }
}

/// A scratch register that `inst` does not name and that nothing after it
/// in the block reads before writing. Scratch registers hold values only
/// within a spill sequence, never from one block to the next.
fn free_scratch(t: &Target, scratch: &[u8], inst: &MInst, rest: &[MInst]) -> Option<u8> {
    let names = |i: &MInst, r: u8| i.ops.iter().any(|o| *o == MOp::Reg(Reg::P(r)));
    scratch.iter().copied().find(|&r| {
        if names(inst, r) || inst.imp_uses.contains(&r) {
            return false;
        }
        for next in rest {
            let (defs, uses) = next.defs_uses(t);
            let is = |k: &usize| next.ops[*k] == MOp::Reg(Reg::P(r));
            if uses.iter().any(is) || next.imp_uses.contains(&r) {
                return false;
            }
            if defs.iter().any(is) || next.imp_defs.contains(&r) {
                return true;
            }
        }
        true
    })
}
