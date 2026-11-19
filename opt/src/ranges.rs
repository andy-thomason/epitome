//! Value ranges: what each integer can be, as an unsigned and a signed
//! interval.
//!
//! Each integer value of at most 64 bits gets `[umin, umax]` and
//! `[smin, smax]`; it is in both. They are computed once per function,
//! in reverse postorder, from constants, casts, masks, division, interval
//! arithmetic where nothing wraps (or a flag says it cannot), selects,
//! `min`/`max`, `range` attributes, and the phis of counters stepping by
//! one under their loop's test. Other phis on a back edge are unknown:
//! there is no iteration to a fixed point.
//!
//! `implied` refines a value's range by the facts in force where it is
//! used, and decides comparisons with it. See `docs/design/ranges.md`.

use epitome_ir::module::*;
use epitome_ir::types::Type;

use crate::cfg::DomTree;
use crate::combine::eval;
use crate::ctx::Shared;

/// The values an integer of `bits` bits can take: unsigned `[umin, umax]`
/// and signed `[smin, smax]`, both at once.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Range {
    pub bits: u32,
    pub umin: u64,
    pub umax: u64,
    pub smin: i64,
    pub smax: i64,
}

pub fn smin_of(bits: u32) -> i64 {
    eval::sext(1u64 << (bits - 1), bits)
}

pub fn smax_of(bits: u32) -> i64 {
    (eval::mask(bits) >> 1) as i64
}

impl Range {
    pub fn full(bits: u32) -> Range {
        Range { bits, umin: 0, umax: eval::mask(bits), smin: smin_of(bits), smax: smax_of(bits) }
    }

    pub fn constant(bits: u32, v: u64) -> Range {
        let v = v & eval::mask(bits);
        let s = eval::sext(v, bits);
        Range { bits, umin: v, umax: v, smin: s, smax: s }
    }

    /// Unsigned `[lo, hi]`, and what that says of the signed view.
    pub fn unsigned(bits: u32, lo: u64, hi: u64) -> Range {
        Range { umin: lo, umax: hi, ..Range::full(bits) }.normal()
    }

    /// Signed `[lo, hi]`, and what that says of the unsigned view.
    pub fn signed(bits: u32, lo: i64, hi: i64) -> Range {
        Range { smin: lo, smax: hi, ..Range::full(bits) }.normal()
    }

    pub fn is_empty(&self) -> bool {
        self.umin > self.umax || self.smin > self.smax
    }

    pub fn as_constant(&self) -> Option<u64> {
        (self.umin == self.umax).then_some(self.umin)
    }

    /// Each view narrowed by the other: an unsigned interval below the
    /// sign bit is the same signed, and so on.
    fn normal(mut self) -> Range {
        let top = smax_of(self.bits) as u64;
        // Unsigned to signed.
        if self.umax <= top {
            self.smin = self.smin.max(self.umin as i64);
            self.smax = self.smax.min(self.umax as i64);
        } else if self.umin > top {
            self.smin = self.smin.max(eval::sext(self.umin, self.bits));
            self.smax = self.smax.min(eval::sext(self.umax, self.bits));
        }
        // Signed to unsigned.
        let m = eval::mask(self.bits);
        if self.smin >= 0 {
            self.umin = self.umin.max(self.smin as u64);
            self.umax = self.umax.min(self.smax as u64);
        } else if self.smax < 0 {
            self.umin = self.umin.max(self.smin as u64 & m);
            self.umax = self.umax.min(self.smax as u64 & m);
        }
        self
    }

    /// Values in both.
    pub fn meet(self, o: Range) -> Range {
        if self.bits != o.bits {
            return self;
        }
        Range {
            bits: self.bits,
            umin: self.umin.max(o.umin),
            umax: self.umax.min(o.umax),
            smin: self.smin.max(o.smin),
            smax: self.smax.min(o.smax),
        }
        .normal()
    }

    /// Values in either.
    pub fn join(self, o: Range) -> Range {
        if self.bits != o.bits {
            return Range::full(self.bits);
        }
        if self.is_empty() {
            return o;
        }
        if o.is_empty() {
            return self;
        }
        Range {
            bits: self.bits,
            umin: self.umin.min(o.umin),
            umax: self.umax.max(o.umax),
            smin: self.smin.min(o.smin),
            smax: self.smax.max(o.smax),
        }
    }
}

/// The smallest `2^k - 1` at or above `x`: every bit `x` could set.
fn fill(x: u64) -> u64 {
    if x == 0 { 0 } else { u64::MAX >> x.leading_zeros() }
}

/// What a binary operation can give, from what its operands can be.
pub fn binary(op: BinOp, flags: IntFlags, a: Range, b: Range) -> Range {
    let bits = a.bits;
    let m = eval::mask(bits);
    let (smin, smax) = (smin_of(bits) as i128, smax_of(bits) as i128);
    let full = Range::full(bits);
    // Each view on its own, from 128-bit sums, kept only when no bound
    // leaves the type (or a flag says the result would be poison if one
    // did).
    let views = |u: Option<(u128, u128)>, s: Option<(i128, i128)>| {
        let mut r = full;
        if let Some((lo, hi)) = u {
            if hi <= m as u128 {
                r = r.meet(Range::unsigned(bits, lo as u64, hi as u64));
            } else if flags.contains(IntFlags::NUW) && lo <= m as u128 {
                r = r.meet(Range::unsigned(bits, lo as u64, m));
            }
        }
        if let Some((lo, hi)) = s {
            if lo >= smin && hi <= smax {
                r = r.meet(Range::signed(bits, lo as i64, hi as i64));
            } else if flags.contains(IntFlags::NSW) {
                r = r.meet(Range::signed(bits, lo.clamp(smin, smax) as i64, hi.clamp(smin, smax) as i64));
            }
        }
        r
    };
    let (au, bu) = ((a.umin as u128, a.umax as u128), (b.umin as u128, b.umax as u128));
    let (as_, bs) = ((a.smin as i128, a.smax as i128), (b.smin as i128, b.smax as i128));
    match op {
        BinOp::Add => views(Some((au.0 + bu.0, au.1 + bu.1)), Some((as_.0 + bs.0, as_.1 + bs.1))),
        BinOp::Sub => {
            let u = (au.0 >= bu.1).then(|| (au.0 - bu.1, au.1 - bu.0));
            views(u, Some((as_.0 - bs.1, as_.1 - bs.0)))
        }
        BinOp::Mul => {
            let s = [as_.0 * bs.0, as_.0 * bs.1, as_.1 * bs.0, as_.1 * bs.1];
            let s = (*s.iter().min().unwrap(), *s.iter().max().unwrap());
            let u = au.1.checked_mul(bu.1).map(|hi| (au.0 * bu.0, hi));
            views(u, Some(s))
        }
        BinOp::UDiv if b.umin > 0 => Range::unsigned(bits, a.umin / b.umax, a.umax / b.umin),
        BinOp::URem if b.umax > 0 => Range::unsigned(bits, 0, a.umax.min(b.umax - 1)),
        BinOp::SRem if b.smin > 0 => {
            // The result has the dividend's sign and is smaller than the divisor.
            let k = (b.smax - 1).max(0);
            let lo = if a.smin >= 0 { 0 } else { (-k).max(a.smin) };
            let hi = if a.smax < 0 { 0 } else { k.min(a.smax) };
            Range::signed(bits, lo, hi)
        }
        BinOp::And => Range::unsigned(bits, 0, a.umax.min(b.umax)),
        BinOp::Or => Range::unsigned(bits, a.umin.max(b.umin), fill(a.umax | b.umax)),
        BinOp::Xor => Range::unsigned(bits, 0, fill(a.umax | b.umax)),
        BinOp::LShr if b.umax < bits as u64 => Range::unsigned(bits, a.umin >> b.umax, a.umax >> b.umin),
        BinOp::LShr => Range::unsigned(bits, 0, a.umax),
        BinOp::AShr if b.umax < bits as u64 => {
            let lo = (a.smin >> b.umin).min(a.smin >> b.umax);
            let hi = (a.smax >> b.umin).max(a.smax >> b.umax);
            Range::signed(bits, lo, hi)
        }
        BinOp::Shl if b.umax < bits as u64 => {
            let u = (au.0 << b.umin, au.1 << b.umax);
            let s = [as_.0 << b.umin, as_.0 << b.umax, as_.1 << b.umin, as_.1 << b.umax];
            views(Some(u), Some((*s.iter().min().unwrap(), *s.iter().max().unwrap())))
        }
        _ => full,
    }
}

/// Whether `op` on operands in these ranges can wrap, unsigned and signed:
/// `(nuw holds, nsw holds)`.
pub fn no_wrap(op: BinOp, a: Range, b: Range) -> (bool, bool) {
    let bits = a.bits;
    let m = eval::mask(bits) as u128;
    let (smin, smax) = (smin_of(bits) as i128, smax_of(bits) as i128);
    let (au, bu) = ((a.umin as u128, a.umax as u128), (b.umin as u128, b.umax as u128));
    let (as_, bs) = ((a.smin as i128, a.smax as i128), (b.smin as i128, b.smax as i128));
    let fits = |lo: i128, hi: i128| lo >= smin && hi <= smax;
    match op {
        BinOp::Add => (au.1 + bu.1 <= m, fits(as_.0 + bs.0, as_.1 + bs.1)),
        BinOp::Sub => (au.0 >= bu.1, fits(as_.0 - bs.1, as_.1 - bs.0)),
        BinOp::Mul => {
            let s = [as_.0 * bs.0, as_.0 * bs.1, as_.1 * bs.0, as_.1 * bs.1];
            (au.1.checked_mul(bu.1).is_some_and(|x| x <= m), fits(*s.iter().min().unwrap(), *s.iter().max().unwrap()))
        }
        _ => (false, false),
    }
}

/// The range a `range(iN lo, hi)` attribute gives: `[lo, hi)`, wrapping
/// round when `lo > hi`.
fn range_attr(attrs: &[Attr], bits: u32) -> Option<Range> {
    for a in attrs {
        let Attr::Parens { name, text } = a else { continue };
        if name != "range" {
            continue;
        }
        let (_, nums) = text.trim().split_once(' ')?;
        let (lo, hi) = nums.split_once(',')?;
        let parse = |t: &str| -> Option<u64> {
            let t = t.trim();
            let v: i128 = t.parse().ok()?;
            Some(v as u64 & eval::mask(bits))
        };
        let (lo, hi) = (parse(lo)?, parse(hi)?);
        let m = eval::mask(bits);
        let last = hi.wrapping_sub(1) & m;
        if lo == hi {
            return None;
        }
        // Without wrapping round in one view or the other.
        if lo <= last {
            return Some(Range::unsigned(bits, lo, last));
        }
        let (sl, sh) = (eval::sext(lo, bits), eval::sext(last, bits));
        if sl <= sh {
            return Some(Range::signed(bits, sl, sh));
        }
    }
    None
}

pub struct Ranges {
    /// By local; `None` for values that are not integers of at most 64
    /// bits, or not yet computed.
    of: Vec<Option<Range>>,
    /// The block of each instruction.
    blocks: Vec<Option<BlockId>>,
}

impl Ranges {
    pub fn compute(sh: &Shared, f: &Function, dom: &DomTree, preds: &[Vec<BlockId>]) -> Ranges {
        let mut r = Ranges { of: vec![None; f.locals.len()], blocks: f.inst_blocks() };
        for p in &f.params {
            let Some(bits) = int_bits(sh, f.locals[p.local.index()].ty) else { continue };
            r.of[p.local.index()] = Some(range_attr(&p.attrs, bits).unwrap_or(Range::full(bits)));
        }
        for b in crate::cfg::reverse_postorder(f) {
            for &i in &f.blocks[b.index()].insts {
                let Some(res) = f.insts[i.index()].result else { continue };
                let Some(bits) = int_bits(sh, f.locals[res.index()].ty) else { continue };
                let range = r.transfer(sh, f, dom, preds, b, i, bits).meet(Range::full(bits));
                r.of[res.index()] = Some(if range.is_empty() { Range::full(bits) } else { range });
            }
        }
        r
    }

    /// What `v` can be, if it is an integer of at most 64 bits. A value not
    /// computed yet (along a back edge) can be anything.
    pub fn get(&self, sh: &Shared, f: &Function, v: Value) -> Option<Range> {
        match v {
            Value::Local(x) => {
                let bits = int_bits(sh, f.locals[x.index()].ty)?;
                Some(self.of[x.index()].unwrap_or(Range::full(bits)))
            }
            Value::Const(c) => {
                let bits = int_bits(sh, sh.consts().get(c).ty)?;
                Some(match eval::int_const(sh.consts(), sh.types(), c) {
                    Some(k) => Range::constant(bits, k.value),
                    None => Range::full(bits),
                })
            }
            Value::Metadata(_) => None,
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn transfer(
        &self,
        sh: &Shared,
        f: &Function,
        dom: &DomTree,
        preds: &[Vec<BlockId>],
        b: BlockId,
        i: InstId,
        bits: u32,
    ) -> Range {
        let full = Range::full(bits);
        let get = |v: Value| self.get(sh, f, v).unwrap_or(full);
        match &f.insts[i.index()].op {
            &Op::Binary { op, flags, lhs, rhs, .. } => {
                let (a, c) = (get(lhs), get(rhs));
                if a.bits != bits || c.bits != bits {
                    return full;
                }
                binary(op, flags, a, c)
            }
            &Op::Cast { op, value, .. } => {
                let Some(a) = self.get(sh, f, value) else { return full };
                match op {
                    CastOp::ZExt => Range::unsigned(bits, a.umin, a.umax),
                    CastOp::SExt => {
                        let m = eval::mask(bits);
                        let r = Range::signed(bits, a.smin, a.smax);
                        if a.smin >= 0 { r.meet(Range::unsigned(bits, a.umin, a.umax & m)) } else { r }
                    }
                    CastOp::Trunc if a.umax <= eval::mask(bits) => Range::unsigned(bits, a.umin, a.umax),
                    CastOp::Trunc if a.smin >= smin_of(bits) && a.smax <= smax_of(bits) => {
                        Range::signed(bits, a.smin, a.smax)
                    }
                    _ => full,
                }
            }
            Op::ICmp { .. } | Op::FCmp { .. } => Range::unsigned(bits, 0, 1),
            &Op::Select { then_value, else_value, .. } => get(then_value).join(get(else_value)),
            Op::Phi { incoming, .. } => {
                if let Some(r) = self.counter(sh, f, dom, preds, b, i, bits) {
                    return r;
                }
                let mut r: Option<Range> = None;
                for &(v, from) in incoming {
                    // A value along a back edge is not known yet.
                    if dom.dominates(b, from) {
                        return full;
                    }
                    let x = get(v);
                    r = Some(r.map_or(x, |r| r.join(x)));
                }
                r.unwrap_or(full)
            }
            Op::Call(call) => {
                let mut r = range_attr(&call.ret_attrs, bits).unwrap_or(full);
                let Some(g) = sh.direct_callee(call) else { return r };
                let name = sh.name(g);
                let args: Vec<Range> = call.args.iter().map(|a| get(a.value)).collect();
                let base = name.split('.').take(2).collect::<Vec<_>>().join(".");
                r = r.meet(match (base.as_str(), args.as_slice()) {
                    ("llvm.umin", [x, y]) => Range::unsigned(bits, x.umin.min(y.umin), x.umax.min(y.umax)),
                    ("llvm.umax", [x, y]) => Range::unsigned(bits, x.umin.max(y.umin), x.umax.max(y.umax)),
                    ("llvm.smin", [x, y]) => Range::signed(bits, x.smin.min(y.smin), x.smax.min(y.smax)),
                    ("llvm.smax", [x, y]) => Range::signed(bits, x.smin.max(y.smin), x.smax.max(y.smax)),
                    ("llvm.ctpop" | "llvm.ctlz" | "llvm.cttz", _) => Range::unsigned(bits, 0, bits as u64),
                    _ => full,
                });
                if let Some(a) = sh.published(g).and_then(|c| range_attr(&c.ret_attrs, bits)) {
                    r = r.meet(a);
                }
                r
            }
            _ => full,
        }
    }

    /// A counter stepping by one under its loop's own test:
    /// `i = phi [start, outside], [i + 1, latch]`, where the header ends in
    /// `br (i < n), body, exit` and every latch is in what `body`, entered
    /// only from the header, dominates. `i` goes up only while below `n`,
    /// so it never passes `max(start, n)` and never wraps. Counting down
    /// by one while above `x` (or not zero) is the mirror.
    #[allow(clippy::too_many_arguments)]
    fn counter(
        &self,
        sh: &Shared,
        f: &Function,
        dom: &DomTree,
        preds: &[Vec<BlockId>],
        h: BlockId,
        phi: InstId,
        bits: u32,
    ) -> Option<Range> {
        let Op::Phi { incoming, .. } = &f.insts[phi.index()].op else { return None };
        let me = f.result(phi);
        let full = Range::full(bits);
        let get = |v: Value| self.get(sh, f, v).unwrap_or(full);
        // The header's test, and the block the loop goes on to.
        let t = f.terminator(h)?;
        let Op::CondBr { cond, then_dest, else_dest } = f.insts[t.index()].op else { return None };
        if then_dest == else_dest {
            return None;
        }
        let Op::ICmp { pred, lhs, rhs, .. } = *op_of(f, cond)? else { return None };
        let (pred, n) = if lhs == me {
            (pred, rhs)
        } else if rhs == me {
            (eval::swapped(pred), lhs)
        } else {
            return None;
        };
        // `n` must be the same each time round.
        if let Value::Local(x) = n
            && let LocalDef::Inst(d) = f.locals[x.index()].def
        {
            let db = self.blocks[d.index()]?;
            if db == h || !dom.dominates(db, h) {
                return None;
            }
        }
        let mut start: Option<Range> = None;
        let mut step: Option<i64> = None;
        let mut latches = Vec::new();
        for &(v, from) in incoming {
            if dom.dominates(h, from) {
                // Along a back edge: `i + k`, or a select of that and `i`.
                let k = step_of(sh, f, me, v)?;
                if step.is_some_and(|s| s != k) {
                    return None;
                }
                step = Some(k);
                latches.push(from);
            } else {
                let s = get(v);
                start = Some(start.map_or(s, |r| r.join(s)));
            }
        }
        let (Some(s), Some(k)) = (start, step) else { return None };
        // Each step only after going on past the test: every latch is in
        // what the block the test goes on to dominates, and that block is
        // entered only from the header.
        let on = |x: BlockId| x != h && preds[x.index()].as_slice() == [h] && latches.iter().all(|&l| dom.dominates(x, l));
        let p = if on(then_dest) {
            pred
        } else if on(else_dest) {
            inverse(pred)
        } else {
            return None;
        };
        let nr = get(n);
        let m = eval::mask(bits);
        Some(match (k, p) {
            (1, ICmpPred::Ult) => Range::unsigned(bits, s.umin, s.umax.max(nr.umax)),
            (1, ICmpPred::Ule) if nr.umax < m => Range::unsigned(bits, s.umin, s.umax.max(nr.umax + 1)),
            (1, ICmpPred::Slt) => Range::signed(bits, s.smin, s.smax.max(nr.smax)),
            (1, ICmpPred::Sle) if nr.smax < smax_of(bits) => Range::signed(bits, s.smin, s.smax.max(nr.smax + 1)),
            // Up by one until equal to `n`, from at or below it.
            (1, ICmpPred::Ne) if s.umax <= nr.umin => Range::unsigned(bits, s.umin, nr.umax),
            (-1, ICmpPred::Ugt) => Range::unsigned(bits, s.umin.min(nr.umin), s.umax),
            (-1, ICmpPred::Uge) if nr.umin > 0 => Range::unsigned(bits, s.umin.min(nr.umin - 1), s.umax),
            (-1, ICmpPred::Sgt) => Range::signed(bits, s.smin.min(nr.smin), s.smax),
            (-1, ICmpPred::Sge) if nr.smin > smin_of(bits) => Range::signed(bits, s.smin.min(nr.smin - 1), s.smax),
            (-1, ICmpPred::Ne) if nr.umax <= s.umin => Range::unsigned(bits, nr.umin, s.umax),
            _ => return None,
        })
    }
}

/// The predicate that holds when `p` does not.
fn inverse(p: ICmpPred) -> ICmpPred {
    use ICmpPred::*;
    match p {
        Eq => Ne,
        Ne => Eq,
        Ugt => Ule,
        Uge => Ult,
        Ult => Uge,
        Ule => Ugt,
        Sgt => Sle,
        Sge => Slt,
        Slt => Sge,
        Sle => Sgt,
    }
}

/// The step `k` if `v` is `i + k` (or `select c, i + k, i`), `k` being 1
/// or -1.
fn step_of(sh: &Shared, f: &Function, i: Value, v: Value) -> Option<i64> {
    match *op_of(f, v)? {
        Op::Binary { op: BinOp::Add, lhs, rhs, .. } => {
            let (x, k) = if lhs == i { (lhs, rhs) } else { (rhs, lhs) };
            if x != i {
                return None;
            }
            let Value::Const(c) = k else { return None };
            let k = eval::int_const(sh.consts(), sh.types(), c)?;
            let k = eval::sext(k.value, k.bits);
            (k == 1 || k == -1).then_some(k)
        }
        Op::Select { then_value, else_value, .. } => {
            if else_value == i {
                step_of(sh, f, i, then_value)
            } else if then_value == i {
                step_of(sh, f, i, else_value)
            } else {
                None
            }
        }
        _ => None,
    }
}

fn op_of(f: &Function, v: Value) -> Option<&Op> {
    let Value::Local(x) = v else { return None };
    match f.locals[x.index()].def {
        LocalDef::Inst(i) => Some(&f.insts[i.index()].op),
        _ => None,
    }
}

fn int_bits(sh: &Shared, ty: epitome_ir::types::TypeId) -> Option<u32> {
    match *sh.types().get(ty) {
        Type::Int(b) if (1..=64).contains(&b) => Some(b),
        _ => None,
    }
}
