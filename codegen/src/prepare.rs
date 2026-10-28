//! Selection form: rewriting a copy of a function so that its IR has the
//! shapes the instruction descriptions' bodies have.
//!
//! - Every `getelementptr` becomes `getelementptr i8, ptr %base, i64 %off`,
//!   with the offset worked out in `i64`: indices sign-extended, scaled by
//!   `shl` for powers of two and `mul` otherwise, constant parts added up.
//! - `switch` becomes a chain of `icmp eq` and conditional branches.
//! - Edges from a block with several successors to a block with phis are
//!   split, so that the phis' copies have somewhere to go.
//! - Calls to intrinsics that do nothing in code (`llvm.lifetime.*`,
//!   `llvm.dbg.*`, `llvm.assume`) are removed.
//! - Comparisons, divisions and right shifts of values narrower than 32
//!   bits are done on 32-bit extensions of them: a narrow value lives in a
//!   W register with its upper bits undefined, so these operations, which
//!   read every bit, need them made right first.
//! - Loads and stores of small structs (`[2 x i64]`) become one per part.
//! - Intrinsics the description has no instruction for are written out:
//!   minimum, maximum and absolute value of narrow integers as a comparison
//!   and a select, and population count as bit arithmetic.
//! - Conversions between floating point and integers narrower than 32
//!   bits go through `i32`; `fcmp one` and `fcmp ueq`, which no single
//!   condition tests after AArch64's `fcmp`, are split into two; a store of
//!   a floating-point constant stores its bits as an integer.
//! - Division and remainder by constants become multiplications and
//!   shifts (see `divconst`).
//! - Integers of other widths below 64 (`i33`, say) are held in the next
//!   register width, with the same rule: masked or sign-extended in place
//!   before an operation that reads their top bits.

use epitome_ir::apint::ApInt;
use epitome_ir::layout::DataLayout;
use epitome_ir::module::*;
use epitome_ir::types::{Type, TypeId};

use crate::target::{Head, Tree, sext};

pub struct Ctx<'a> {
    pub m: &'a mut Module,
    pub dl: &'a DataLayout,
    /// The machine description, for what it says about the machine as a
    /// whole (its vector registers' width, which shuffles it has).
    pub t: &'a crate::target::Target,
    /// The features of the function being compiled.
    pub features: crate::features::Features,
    /// Whether the convention passes homogeneous floating-point aggregates
    /// in floating-point registers (`Abi::hfa`).
    pub hfa: bool,
}

impl Ctx<'_> {
    /// The width of the vector registers, or 0 if there are none (or their
    /// feature is off): then vectors are done lane by lane.
    pub fn vector_bits(&self) -> u32 {
        // The widest whose feature is on (x86: 256 bits with avx2).
        self.t
            .machine
            .vector_widths
            .iter()
            .filter(|(_, f)| f.as_ref().is_none_or(|f| self.features.contains(f)))
            .map(|&(b, _)| b)
            .max()
            .unwrap_or(0)
    }

    /// The narrowest vector kept whole in a register: narrower ones are
    /// done lane by lane.
    pub fn min_vector_bits(&self) -> u32 {
        let vb = self.vector_bits();
        self.t.machine.min_vector_bits.map_or(vb / 2, |m| m.min(vb))
    }

    /// Are instruction `i`'s features on?
    pub fn enabled(&self, i: usize) -> bool {
        self.t.instrs[i].features.iter().all(|f| self.features.contains(f))
    }

    /// Is there a pattern, of instructions that are on, with an IR node
    /// `want` accepts?
    pub fn selects_head(&self, want: impl Fn(&Head) -> bool) -> bool {
        self.selects(want)
    }

    /// Is there an instruction for intrinsic `name` whose last operand is
    /// the constant `last` (`llvm.epitome.cmpxchg.i64`'s ordering)?
    pub fn selects_intrinsic_with(&self, name: &str, last: i64) -> bool {
        let Some(id) = self.t.intrinsics.iter().position(|n| n == name) else { return false };
        fn has(t: &Tree, id: usize, last: i64) -> bool {
            match t {
                Tree::Op(Head::Intrinsic(k, _), kids) if *k as usize == id => {
                    matches!(kids.last(), Some(Tree::Lit(v, _)) if *v == last)
                }
                Tree::Op(_, kids) => kids.iter().any(|k| has(k, id, last)),
                _ => false,
            }
        }
        self.t.patterns.iter().any(|p| has(&p.tree, id, last) && p.emit.iter().all(|e| self.enabled(e.instr)))
    }

    fn selects(&self, want: impl Fn(&Head) -> bool) -> bool {
        fn has(t: &Tree, want: &dyn Fn(&Head) -> bool) -> bool {
            match t {
                Tree::Op(h, kids) => want(h) || kids.iter().any(|k| has(k, want)),
                _ => false,
            }
        }
        self.t.patterns.iter().any(|p| has(&p.tree, &want) && p.emit.iter().all(|e| self.enabled(e.instr)))
    }

    /// Is there a pattern, of instructions that are on, that is `head`
    /// alone on values (`mul` of any two registers), not only on what some
    /// other operation made (AArch64's `smull`, a `mul` of extended halves)?
    pub fn selects_head_alone(&self, head: &Head) -> bool {
        self.t.patterns.iter().any(|p| {
            matches!(&p.tree, Tree::Op(h, kids) if h == head && kids.iter().all(|k| !matches!(k, Tree::Op(..))))
                && p.emit.iter().all(|e| self.enabled(e.instr))
        })
    }

    /// Is there an instruction for intrinsic `name` (`llvm.ctlz.i64`)?
    pub fn selects_intrinsic(&self, name: &str) -> bool {
        match self.t.intrinsics.iter().position(|n| n == name) {
            Some(id) => self.selects(|h| matches!(h, Head::Intrinsic(k, _) if *k as usize == id)),
            None => false,
        }
    }

    fn i64t(&mut self) -> TypeId {
        self.m.types.int(64)
    }

    fn i8t(&mut self) -> TypeId {
        self.m.types.int(8)
    }

    pub fn int(&mut self, ty: TypeId, v: i64) -> Value {
        let bits = match self.m.types.get(ty) {
            Type::Int(b) => *b,
            _ => 64,
        };
        Value::Const(self.m.consts.intern(Constant { ty, kind: ConstKind::Int(ApInt::from_i64(bits, v)) }))
    }

    pub fn const_int(&self, v: Value) -> Option<i64> {
        let Value::Const(c) = v else { return None };
        let k = self.m.constant(c);
        match (&k.kind, self.m.types.get(k.ty)) {
            (ConstKind::Int(a), Type::Int(b)) => Some(sext(a.low_u64(), *b)),
            _ => None,
        }
    }

    pub fn value_ty(&self, f: &Function, v: Value) -> TypeId {
        match v {
            Value::Local(x) => f.locals[x.index()].ty,
            Value::Const(c) => self.m.constant(c).ty,
            Value::Metadata(_) => unreachable!(),
        }
    }

    pub fn size_of(&self, ty: TypeId) -> Result<u64, String> {
        self.dl.alloc_size(&self.m.types, ty).ok_or_else(|| "a type without a size".to_string())
    }
}

/// Puts a copy of `f` into selection form.
pub fn prepare(cx: &mut Ctx, f: &Function, fname: &str) -> Result<Function, String> {
    let mut f = f.clone();
    neon_intrinsics(cx, &mut f);
    crate::neon::lower(cx, &mut f);
    number_min_max(cx, &mut f);
    crate::intrinsics::lower(cx, &mut f);
    // What the runtime does, then types no register holds as what holds
    // them (fp128 as <2 x i64>), as the description says.
    crate::runtime::lower(cx, &mut f).map_err(|e| format!("@{fname}: {e}"))?;
    crate::runtime::hold(cx, &mut f).map_err(|e| format!("@{fname}: {e}"))?;
    thread_locals(cx, &mut f);
    const_exprs(cx, &mut f);
    addresses_as_ints(&mut f);
    half_as_int(cx, &mut f).map_err(|e| format!("@{fname}: {e}"))?;
    crate::x87::lower(cx, &mut f).map_err(|e| format!("@{fname}: {e}"))?;
    identity_bitcasts(&mut f);
    one_lane_vectors(cx, &mut f);
    odd_lane_vectors(cx, &mut f);
    bits_of_vectors(cx, &mut f);
    fields_of_masks(cx, &mut f);
    vector_arguments(cx, &mut f);
    // (Before masks' bitcasts are made other operations, which want to
    // see the comparisons masks come from.)
    mask_logic(cx, &mut f);
    wide_mask_bits(cx, &mut f);
    vector_int_bitcasts(cx, &mut f);
    bit_masks(cx, &mut f);
    bitcast_round_trips(cx, &mut f);
    variable_lanes(cx, &mut f);
    memset_patterns(cx, &mut f);
    crate::atomics::expand_atomics(cx, &mut f).map_err(|e| format!("@{fname}: {e}"))?;
    expand_overflow(cx, &mut f).map_err(|e| format!("@{fname}: {e}"))?;
    wide_intrinsics(cx, &mut f);
    wide_products(cx, &mut f);
    // (Before wide integers are split, so that a switch of one is their
    // comparisons.)
    unreachable_edges(&mut f);
    lower_switches(cx, &mut f);
    // (Integers wider than 128 bits as limbs, before wide.rs sees the
    // i128s that leaves.)
    crate::huge::split_huge(cx, &mut f).map_err(|e| format!("@{fname}: {e}"))?;
    crate::wide_vectors::scalarize(cx, &mut f).map_err(|e| format!("@{fname}: {e}"))?;
    // (Aggregates of wide integers, as the overflow intrinsics' { i128,
    // i1 }, as their fields first, which wide.rs then splits.)
    crate::aggregates::scalarize(cx, &mut f).map_err(|e| format!("@{fname}: {e}"))?;
    crate::wide::split_wide(cx, &mut f).map_err(|e| format!("@{fname}: {e}"))?;
    identity_ops(cx, &mut f);
    extended_bool_compares(cx, &mut f);
    crate::bitmaps::simplify(cx, &mut f);
    crate::aggregates::scalarize(cx, &mut f).map_err(|e| format!("@{fname}: {e}"))?;
    split_struct_selects(cx, &mut f).map_err(|e| format!("@{fname}: {e}"))?;
    split_struct_phis(cx, &mut f).map_err(|e| format!("@{fname}: {e}"))?;
    check(cx, &f, fname)?;
    lower_dynamic_allocas(cx, &mut f)?;
    drop_noop_calls(cx, &mut f);
    expand_intrinsics(cx, &mut f);
    lane_indices(cx, &mut f);
    split_aggregate_memory(cx, &mut f);
    bool_memory(cx, &mut f);
    expand_small_mem_ops(cx, &mut f);
    crate::legalize::legalize_vectors(cx, &mut f).map_err(|e| format!("@{fname}: {e}"))?;
    // Lanes of masks legalisation leaves (it handles those of split masks,
    // at their comparisons' widths; before it, this would get in its way).
    mask_lanes(cx, &mut f);
    // What no instruction does, a lane at a time (after the masks' lanes).
    crate::lanewise::lanewise(cx, &mut f).map_err(|e| format!("@{fname}: {e}"))?;
    // Element-wise intrinsics split into scalars expand as scalars do.
    expand_intrinsics(cx, &mut f);
    constant_branches(cx, &mut f);
    sink_addresses(&mut f);
    constant_addresses(&mut f);
    lower_geps(cx, &mut f)?;
    constant_offsets_last(cx, &mut f);
    promote_odd(cx, &mut f)?;
    widen_narrow(cx, &mut f);
    legalize_fp(cx, &mut f)?;
    lower_const_divs(cx, &mut f);
    const_multiplies(cx, &mut f);
    if cx.t.machine.compare_booleans {
        compare_boolean_branches(cx, &mut f);
    } else {
        compares_per_use(cx, &mut f);
    }
    split_phi_edges(&mut f);
    Ok(f)
}

/// Field `k` of struct value `v` (of field type `fty`): a constant's, or
/// an `extractvalue` made in `s`.
fn struct_field(cx: &mut Ctx, s: &mut Seq, v: Value, k: usize, fty: TypeId) -> Result<Value, String> {
    Ok(match v {
        Value::Const(c) => {
            let kind = match &cx.m.constant(c).kind {
                ConstKind::Struct(e) => cx.m.constant(e[k]).kind.clone(),
                k @ (ConstKind::Poison | ConstKind::Undef | ConstKind::Zero) => k.clone(),
                _ => return Err("a struct constant not supported".into()),
            };
            Value::Const(cx.m.consts.intern(Constant { ty: fty, kind }))
        }
        _ => s.op(Op::ExtractValue { aggregate: v, indices: vec![k as u32] }, fty),
    })
}

/// A select of a struct of scalars (`{ ptr, i64 }`, a slice) is a select
/// of each field, and the struct made again of them.
fn split_struct_selects(cx: &mut Ctx, f: &mut Function) -> Result<(), String> {
    let blocks: Vec<BlockId> = f.layout.clone();
    for b in blocks {
        let mut k = 0;
        while k < f.blocks[b.index()].insts.len() {
            let i = f.blocks[b.index()].insts[k];
            // A freeze of an aggregate: in code, a value is what its
            // registers hold, so the freeze is its operand.
            if let Op::Freeze(v) = f.insts[i.index()].op
                && let Some(r) = f.insts[i.index()].result
                && matches!(cx.m.types.get(f.locals[r.index()].ty), Type::Struct { .. } | Type::Array { .. })
            {
                f.replace_uses(Value::Local(r), v);
                f.blocks[b.index()].insts.remove(k);
                continue;
            }
            let Op::Select { fmf, cond, then_value, else_value } = f.insts[i.index()].op.clone() else {
                k += 1;
                continue;
            };
            let r = f.insts[i.index()].result.unwrap();
            let ty = f.locals[r.index()].ty;
            let Type::Struct { fields, .. } = cx.m.types.get(ty).clone() else {
                k += 1;
                continue;
            };
            if fields.iter().any(|t| matches!(cx.m.types.get(*t), Type::Struct { .. } | Type::Array { .. })) {
                k += 1;
                continue;
            }
            let mut s = Seq { f, new: Vec::new() };
            let mut agg = Value::Const(cx.m.consts.intern(Constant { ty, kind: ConstKind::Poison }));
            for (n, &fty) in fields.iter().enumerate() {
                let a = struct_field(cx, &mut s, then_value, n, fty)?;
                let c = struct_field(cx, &mut s, else_value, n, fty)?;
                let v = s.op(Op::Select { fmf, cond, then_value: a, else_value: c }, fty);
                agg = s.op(Op::InsertValue { aggregate: agg, element: v, indices: vec![n as u32] }, ty);
            }
            s.f.replace_uses(Value::Local(r), agg);
            let new = std::mem::take(&mut s.new);
            let n = new.len();
            f.blocks[b.index()].insts.splice(k..=k, new);
            k += n;
        }
    }
    Ok(())
}

/// `llvm.experimental.memset.pattern(dst, pattern, count, volatile)`:
/// `count` stores of `pattern`, one after another from `dst`, as a loop.
fn memset_patterns(cx: &mut Ctx, f: &mut Function) {
    let (i1, i8t, i64t, ptr) = (cx.m.types.int(1), cx.m.types.int(8), cx.m.types.int(64), cx.m.types.ptr(0));
    let mut b = 0;
    'blocks: while b < f.layout.len() {
        let blk = f.layout[b];
        for (k, &i) in f.blocks[blk.index()].insts.iter().enumerate() {
            let Op::Call(call) = &f.insts[i.index()].op else { continue };
            if !callee_name(cx.m, call).is_some_and(|n| n.starts_with("llvm.experimental.memset.pattern.")) {
                continue;
            }
            let (dst, pat, count) = (call.args[0].value, call.args[1].value, call.args[2].value);
            let volatile = cx.const_int(call.args[3].value).is_some_and(|v| v != 0);
            let pty = cx.value_ty(f, pat);
            let size = cx.size_of(pty).unwrap_or(1) as i64;
            let count_ty = cx.value_ty(f, count);
            // blk: count == 0 ? done : lp
            // lp:  n = phi [0, blk], [n + 1, lp]; store pat at dst + n*size
            let lp = f.split_block(blk, k, None);
            let done = f.split_block(lp, 1, None);
            f.blocks[lp.index()].insts.clear();
            let mut s = Seq { f, new: Vec::new() };
            let count = if count_ty == i64t { count } else { s.cast(CastOp::ZExt, count, i64t) };
            let zero = cx.int(i64t, 0);
            let none = s.op(Op::ICmp { pred: ICmpPred::Eq, samesign: false, lhs: count, rhs: zero }, i1);
            let pre = std::mem::take(&mut s.new);
            let phi = s.f.add_inst(Op::Phi { fmf: FastMath::default(), incoming: vec![] }, Some((None, i64t)));
            let n = s.f.result(phi);
            let off = if size > 0 && size & (size - 1) == 0 {
                let sh = cx.int(i64t, i64::from(size.trailing_zeros()));
                s.bin(BinOp::Shl, n, sh, i64t)
            } else {
                let sz = cx.int(i64t, size);
                s.bin(BinOp::Mul, n, sz, i64t)
            };
            let at = s.op(
                Op::Gep { flags: GepFlags::default(), inrange: None, source_ty: i8t, ptr: dst, indices: vec![off] },
                ptr,
            );
            s.op_void(Op::Store { volatile, atomic: None, value: pat, ptr: at, align: None });
            let one = cx.int(i64t, 1);
            let next = s.bin(BinOp::Add, n, one, i64t);
            let more = s.op(Op::ICmp { pred: ICmpPred::Ult, samesign: false, lhs: next, rhs: count }, i1);
            let body = std::mem::take(&mut s.new);
            f.insts[phi.index()].op = Op::Phi { fmf: FastMath::default(), incoming: vec![(zero, blk), (next, lp)] };
            f.blocks[blk.index()].insts.extend(pre);
            f.push_inst(blk, Op::CondBr { cond: none, then_dest: done, else_dest: lp }, None);
            f.blocks[lp.index()].insts.push(phi);
            f.blocks[lp.index()].insts.extend(body);
            f.push_inst(lp, Op::CondBr { cond: more, then_dest: lp, else_dest: done }, None);
            b = f.layout.iter().position(|&x| x == done).unwrap();
            continue 'blocks;
        }
        b += 1;
    }
}

/// A phi of a struct of scalars (a landing pad's `{ ptr, i32 }`, carried
/// to its `resume`) becomes a phi for each field, each value's field
/// extracted at the end of its block, and the struct made again after the
/// phis, as instructions selection keeps aggregates: as their parts.
fn split_struct_phis(cx: &mut Ctx, f: &mut Function) -> Result<(), String> {
    let blocks: Vec<BlockId> = f.layout.clone();
    for b in blocks {
        let phis: Vec<InstId> =
            f.blocks[b.index()].insts.iter().copied().take_while(|&i| f.insts[i.index()].op.is_phi()).collect();
        // A struct an invoke returns, coming into a phi: its fields are
        // taken out on the edge, in a block of its own between the invoke
        // and this one (the invoke ends its block, and they can only be
        // taken once it has returned).
        let mut edges: Vec<BlockId> = Vec::new();
        for &i in &phis {
            let r = f.insts[i.index()].result.unwrap();
            if !matches!(cx.m.types.get(f.locals[r.index()].ty), Type::Struct { .. }) {
                continue;
            }
            let Op::Phi { incoming, .. } = &f.insts[i.index()].op else { unreachable!() };
            for &(v, pred) in incoming {
                let term = f.terminator(pred);
                if term.is_some_and(|t| f.insts[t.index()].result.map(Value::Local) == Some(v))
                    && !edges.contains(&pred)
                {
                    edges.push(pred);
                }
            }
        }
        for pred in edges {
            let e = f.add_block(None);
            let at = f.layout.iter().position(|&x| x == pred).unwrap();
            f.layout.insert(at + 1, e);
            f.push_inst(e, Op::Br(b), None);
            let t = f.terminator(pred).unwrap();
            if let Op::Invoke { normal, .. } = &mut f.insts[t.index()].op
                && *normal == b
            {
                *normal = e;
            }
            f.retarget_phis(b, pred, e);
        }
        let mut rebuilt = Vec::new();
        for &i in &phis {
            let r = f.insts[i.index()].result.unwrap();
            let ty = f.locals[r.index()].ty;
            let Type::Struct { fields, .. } = cx.m.types.get(ty).clone() else { continue };
            if fields.iter().any(|t| matches!(cx.m.types.get(*t), Type::Struct { .. } | Type::Array { .. })) {
                continue;
            }
            let Op::Phi { fmf, incoming } = f.insts[i.index()].op.clone() else { unreachable!() };
            let mut parts = Vec::new();
            for (k, &fty) in fields.iter().enumerate() {
                let mut inc = Vec::new();
                for &(v, pred) in &incoming {
                    let part = match v {
                        Value::Const(c) => {
                            let kind = match &cx.m.constant(c).kind {
                                ConstKind::Struct(e) => cx.m.constant(e[k]).kind.clone(),
                                k @ (ConstKind::Poison | ConstKind::Undef | ConstKind::Zero) => k.clone(),
                                _ => return Err("a phi of a struct constant not supported".into()),
                            };
                            Value::Const(cx.m.consts.intern(Constant { ty: fty, kind }))
                        }
                        _ => {
                            // At the end of the block it comes from, before
                            // its terminator (which must not define it).
                            let insts = &f.blocks[pred.index()].insts;
                            let term = *insts.last().unwrap();
                            if f.insts[term.index()].result.map(Value::Local) == Some(v) {
                                return Err("a phi of a struct an invoke returns".into());
                            }
                            let e = f.add_inst(
                                Op::ExtractValue { aggregate: v, indices: vec![k as u32] },
                                Some((None, fty)),
                            );
                            let n = f.blocks[pred.index()].insts.len();
                            f.blocks[pred.index()].insts.insert(n - 1, e);
                            f.result(e)
                        }
                    };
                    inc.push((part, pred));
                }
                let p = f.add_inst(Op::Phi { fmf, incoming: inc }, Some((None, fty)));
                parts.push(p);
            }
            rebuilt.push((i, r, ty, parts));
        }
        if rebuilt.is_empty() {
            continue;
        }
        let mut new_phis = Vec::new();
        let mut after = Vec::new();
        for (_, r, ty, parts) in rebuilt.iter() {
            let mut s = Seq { f, new: Vec::new() };
            let mut agg = Value::Const(cx.m.consts.intern(Constant { ty: *ty, kind: ConstKind::Poison }));
            for (k, &p) in parts.iter().enumerate() {
                new_phis.push(p);
                let pv = s.f.result(p);
                agg = s.op(Op::InsertValue { aggregate: agg, element: pv, indices: vec![k as u32] }, *ty);
            }
            after.extend(std::mem::take(&mut s.new));
            f.replace_uses(Value::Local(*r), agg);
        }
        let old: Vec<InstId> = rebuilt.iter().map(|x| x.0).collect();
        let insts = &mut f.blocks[b.index()].insts;
        let rest: Vec<InstId> = insts.drain(..).filter(|i| !old.contains(i)).collect();
        let n_phis = rest.iter().take_while(|&&i| f.insts[i.index()].op.is_phi()).count();
        let mut list = rest[..n_phis].to_vec();
        list.extend(new_phis);
        // A landing pad stays first after the phis.
        let mut tail = rest[n_phis..].to_vec();
        let lp = tail.first().is_some_and(|&i| matches!(f.insts[i.index()].op, Op::LandingPad { .. }));
        if lp {
            list.push(tail.remove(0));
        }
        list.extend(after);
        list.extend(tail);
        f.blocks[b.index()].insts = list;
    }
    Ok(())
}

/// `ptrtoaddr`: a pointer's address, which on the targets described is
/// all of it, as `ptrtoint` (after `const_exprs`, which makes constant
/// ones instructions).
fn addresses_as_ints(f: &mut Function) {
    for inst in &mut f.insts {
        if let Op::Cast { op: op @ CastOp::PtrToAddr, .. } = &mut inst.op {
            *op = CastOp::PtrToInt;
        }
    }
}

/// `llvm.minimumnum` and `llvm.maximumnum` (Rust's `f32::min`), which
/// take a number over any NaN, as `llvm.minnum` and `llvm.maxnum` of the
/// operands each made quiet by its minnum with itself (fminnm gives NaN
/// for a signalling NaN), where the machine has minnum.
fn number_min_max(cx: &mut Ctx, f: &mut Function) {
    for b in f.layout.clone() {
        let mut k = 0;
        while k < f.blocks[b.index()].insts.len() {
            let i = f.blocks[b.index()].insts[k];
            let to = match &f.insts[i.index()].op {
                Op::Call(call) => callee_name(cx.m, call).and_then(|name| {
                    let (op, suffix) = name.strip_prefix("llvm.")?.split_once('.')?;
                    let to = match op {
                        "minimumnum" => "minnum",
                        "maximumnum" => "maxnum",
                        _ => return None,
                    };
                    // (Where minnum is an instruction that orders -0 below
                    // +0, AArch64's fminnm; elsewhere the C library's
                    // fminimum_num is.)
                    if !cx.selects_intrinsic("llvm.minnum.f32") {
                        return None;
                    }
                    let args: Vec<Value> = call.args.iter().map(|a| a.value).collect();
                    Some((format!("llvm.{to}.{suffix}"), format!("llvm.minnum.{suffix}"), args))
                }),
                _ => None,
            };
            let (Some((name, quiet, args)), Some(r)) = (to, f.insts[i.index()].result) else {
                k += 1;
                continue;
            };
            let rt = f.locals[r.index()].ty;
            let mut s = Seq { f, new: Vec::new() };
            let q: Vec<Value> = args.iter().map(|&a| call_intrinsic(cx, &mut s, &quiet, vec![a, a], rt)).collect();
            let v = call_intrinsic(cx, &mut s, &name, q, rt);
            let new = std::mem::take(&mut s.new);
            f.replace_uses(Value::Local(r), v);
            let n = new.len();
            f.blocks[b.index()].insts.splice(k..=k, new);
            k += n;
        }
    }
}

/// NEON's intrinsics that LLVM has generic ones for (Rust's `std::arch`
/// and C's arm_neon.h use them): element-wise ones renamed
/// (`llvm.aarch64.neon.fmin.v4f32` is `llvm.minimum.v4f32`), and the
/// across-lanes ones as LLVM's reductions, extended to their wider results
/// (`uaddv.i32.v16i8`); `faddv` adds pairwise, in NEON's order.
fn neon_intrinsics(cx: &mut Ctx, f: &mut Function) {
    let renames: &[(&str, &str)] = &[
        ("fmin", "minimum"),
        ("fmax", "maximum"),
        ("fminnm", "minnum"),
        ("fmaxnm", "maxnum"),
        ("umin", "umin"),
        ("umax", "umax"),
        ("smin", "smin"),
        ("smax", "smax"),
        ("uqadd", "uadd.sat"),
        ("sqadd", "sadd.sat"),
        ("uqsub", "usub.sat"),
        ("sqsub", "ssub.sat"),
        ("frintn", "roundeven"),
    ];
    let reductions: &[(&str, &str, Option<CastOp>)] = &[
        ("uaddv", "add", Some(CastOp::ZExt)),
        // (Each zero-extended, as LLVM's back end gives them: the lane is
        // read with `fmov` from its narrow register.)
        ("saddv", "add", Some(CastOp::ZExt)),
        ("umaxv", "umax", Some(CastOp::ZExt)),
        ("uminv", "umin", Some(CastOp::ZExt)),
        ("smaxv", "smax", Some(CastOp::ZExt)),
        ("sminv", "smin", Some(CastOp::ZExt)),
        ("fmaxv", "fmaximum", None),
        ("fminv", "fminimum", None),
        ("fmaxnmv", "fmax", None),
        ("fminnmv", "fmin", None),
    ];
    let i64t = cx.m.types.int(64);
    for b in f.layout.clone() {
        let mut k = 0;
        while k < f.blocks[b.index()].insts.len() {
            let i = f.blocks[b.index()].insts[k];
            let Op::Call(call) = &f.insts[i.index()].op else {
                k += 1;
                continue;
            };
            let Some(name) = callee_name(cx.m, call) else {
                k += 1;
                continue;
            };
            let Some(rest) = name.strip_prefix("llvm.aarch64.neon.") else {
                k += 1;
                continue;
            };
            let (op, suffix) = rest.split_once('.').unwrap_or((rest, ""));
            let args: Vec<Value> = call.args.iter().map(|a| a.value).collect();
            let Some(r) = f.insts[i.index()].result else {
                k += 1;
                continue;
            };
            let rt = f.locals[r.index()].ty;
            let mut s = Seq { f, new: Vec::new() };
            let v = if let Some(&(_, to)) = renames.iter().find(|(from, _)| *from == op) {
                Some(call_intrinsic(cx, &mut s, &format!("llvm.{to}.{suffix}"), args, rt))
            } else if op == "faddv" && args.len() == 1 {
                let vt = cx.value_ty(s.f, args[0]);
                let Type::Vector { len, elem, .. } = *cx.m.types.get(vt) else { unreachable!() };
                let lane = |cx: &mut Ctx, s: &mut Seq, l: u32| {
                    let idx = cx.int(i64t, l as i64);
                    s.op(Op::ExtractElement { vector: args[0], index: idx }, elem)
                };
                let mut level: Vec<Value> = (0..len).map(|l| lane(cx, &mut s, l)).collect();
                while level.len() > 1 {
                    level = level
                        .chunks(2)
                        .map(|p| {
                            s.op(
                                Op::Binary {
                                    op: BinOp::FAdd,
                                    flags: IntFlags::NONE,
                                    fmf: FastMath::default(),
                                    lhs: p[0],
                                    rhs: p[1],
                                },
                                elem,
                            )
                        })
                        .collect();
                }
                Some(level[0])
            } else if let Some((round, signed)) = match op {
                // Conversions to integers by a rounding (saturating, as the
                // instructions do): to nearest, even (n) or away (a), down
                // (m), up (p), toward zero (z).
                "fcvtns" => Some(("roundeven", true)),
                "fcvtnu" => Some(("roundeven", false)),
                "fcvtas" => Some(("round", true)),
                "fcvtau" => Some(("round", false)),
                "fcvtms" => Some(("floor", true)),
                "fcvtmu" => Some(("floor", false)),
                "fcvtps" => Some(("ceil", true)),
                "fcvtpu" => Some(("ceil", false)),
                "fcvtzs" => Some(("trunc", true)),
                "fcvtzu" => Some(("trunc", false)),
                _ => None,
            } && args.len() == 1
            {
                let ft = cx.value_ty(s.f, args[0]);
                let fm = crate::legalize::mangle(cx, ft);
                let rm = crate::legalize::mangle(cx, rt);
                let x = call_intrinsic(cx, &mut s, &format!("llvm.{round}.{fm}"), args, ft);
                let sat = if signed { "fptosi" } else { "fptoui" };
                Some(call_intrinsic(cx, &mut s, &format!("llvm.{sat}.sat.{rm}.{fm}"), vec![x], rt))
            } else if matches!(
                op,
                "sqxtn"
                    | "uqxtn"
                    | "sqxtun"
                    | "sqshrn"
                    | "uqshrn"
                    | "sqshrun"
                    | "sqrshrn"
                    | "uqrshrn"
                    | "sqrshrun"
                    | "rshrn"
            ) && (args.len() == 1 || args.len() == 2)
            {
                // Narrowing with saturation: clamped to the narrow range,
                // then truncated.
                let wt = cx.value_ty(s.f, args[0]);
                let (Type::Vector { elem: we, .. }, Type::Vector { elem: ne, .. }) =
                    (cx.m.types.get(wt).clone(), cx.m.types.get(rt).clone())
                else {
                    unreachable!()
                };
                let (Type::Int(wb), Type::Int(nb)) = (cx.m.types.get(we).clone(), cx.m.types.get(ne).clone()) else {
                    unreachable!()
                };
                let (lo, hi) = match op {
                    "sqxtn" | "sqshrn" | "sqrshrn" => (-(1i64 << (nb - 1)), (1i64 << (nb - 1)) - 1),
                    _ => (0, ((1u64 << nb) - 1) as i64),
                };
                let splat = |cx: &mut Ctx, v: i64| {
                    let e = cx.m.consts.intern(Constant { ty: we, kind: ConstKind::Int(ApInt::from_i64(wb, v)) });
                    Value::Const(cx.m.consts.intern(Constant { ty: wt, kind: ConstKind::Splat(e) }))
                };
                let wm = crate::legalize::mangle(cx, wt);
                let (hc, lc) = (splat(cx, hi), splat(cx, lo));
                // The shifting ones: shifted right first (arithmetically,
                // but for an unsigned one's), rounding first if `r`.
                let unsigned = op.starts_with('u');
                let mut v = args[0];
                if let Some(&sh) = args.get(1) {
                    let n = cx.const_int(sh).unwrap_or(0);
                    let shr = if unsigned { BinOp::LShr } else { BinOp::AShr };
                    let k = splat(cx, n);
                    let down = s.bin(shr, v, k, wt);
                    v = if op.contains("rshr") && n > 0 {
                        // Rounding: plus the last bit shifted out (as adding
                        // half first would, without overflowing).
                        let k1 = splat(cx, n - 1);
                        let one = splat(cx, 1);
                        let b = s.bin(BinOp::LShr, args[0], k1, wt);
                        let b = s.bin(BinOp::And, b, one, wt);
                        s.bin(BinOp::Add, down, b, wt)
                    } else {
                        down
                    };
                }
                let x = if op == "rshrn" {
                    v
                } else if unsigned {
                    call_intrinsic(cx, &mut s, &format!("llvm.umin.{wm}"), vec![v, hc], wt)
                } else {
                    let y = call_intrinsic(cx, &mut s, &format!("llvm.smin.{wm}"), vec![v, hc], wt);
                    call_intrinsic(cx, &mut s, &format!("llvm.smax.{wm}"), vec![y, lc], wt)
                };
                Some(s.cast(CastOp::Trunc, x, rt))
            } else if matches!(op, "uaddlv" | "saddlv") && args.len() == 1 {
                // The lanes widened to twice their width, then added (the
                // instruction's sum, in a register of that width), then
                // zero-extended to the result, as LLVM's back end reads it.
                let vt = cx.value_ty(s.f, args[0]);
                let Type::Vector { len, elem, .. } = *cx.m.types.get(vt) else { unreachable!() };
                let eb = match cx.m.types.get(elem) {
                    Type::Int(b) => *b,
                    _ => 32,
                };
                let rb = match cx.m.types.get(rt) {
                    Type::Int(b) => *b,
                    _ => 64,
                };
                let st = cx.m.types.int((2 * eb).min(rb));
                let wt = cx.m.types.intern(Type::Vector { scalable: false, len, elem: st });
                let c = if op == "uaddlv" { CastOp::ZExt } else { CastOp::SExt };
                let w = s.cast(c, args[0], wt);
                let mangled = crate::legalize::mangle(cx, wt);
                let sum = call_intrinsic(cx, &mut s, &format!("llvm.vector.reduce.add.{mangled}"), vec![w], st);
                Some(if st == rt { sum } else { s.cast(CastOp::ZExt, sum, rt) })
            } else if let Some(&(_, red, ext)) = reductions.iter().find(|(from, _, _)| *from == op)
                && args.len() == 1
            {
                let vt = cx.value_ty(s.f, args[0]);
                let Type::Vector { elem, .. } = *cx.m.types.get(vt) else { unreachable!() };
                let mangled = crate::legalize::mangle(cx, vt);
                let x = call_intrinsic(cx, &mut s, &format!("llvm.vector.reduce.{red}.{mangled}"), args, elem);
                Some(match ext {
                    Some(c) if rt != elem => s.cast(c, x, rt),
                    _ => x,
                })
            } else {
                None
            };
            let Some(v) = v else {
                k += 1;
                continue;
            };
            let new = std::mem::take(&mut s.new);
            f.replace_uses(Value::Local(r), v);
            let n = new.len();
            f.blocks[b.index()].insts.splice(k..=k, new);
            k += n;
        }
    }
}

/// The address of a thread-local variable, where the machine reaches them
/// by Mach-O's descriptors (`tlv.hi` and `tlv.lo`): its descriptor's
/// address (`llvm.epitome.tlv`), then a call of the descriptor's first
/// word with it, which returns the variable's address in this thread. (The
/// call keeps more registers than a call need, but is taken as one.)
///
/// For ELF, by its descriptors where the machine has them (`tlsdesc.*`):
/// the thread pointer plus the variable's offset from it, which a call
/// through the descriptor gives (`llvm.epitome.tlsdesc`). That is right
/// whatever the variable's model says (a linker makes it cheaper in a
/// program).
///
/// Where the machine has neither, but has offsets from the thread pointer
/// (`tprel.*`, as RISC-V's code is a program's): the variable's address
/// as that sum (`llvm.epitome.tprel`). Or, with the offset in the GOT
/// (`tls.ie`, initial exec: x86's for programs and the libraries they
/// load), the thread pointer plus that (`llvm.epitome.tls.ie`).
fn thread_locals(cx: &mut Ctx, f: &mut Function) {
    let darwin = cx.m.triple.as_deref().is_some_and(|t| {
        let t = String::from_utf8_lossy(t);
        t.contains("apple") || t.contains("darwin") || t.contains("macos")
    });
    let elf = !darwin && cx.t.role("tlsdesc.call").is_some();
    let tprel = !darwin && !elf && cx.t.role("tprel.hi").is_some();
    let ie = !darwin && !elf && !tprel && cx.t.role("tls.ie").is_some();
    if !elf && !tprel && !ie && (!darwin || cx.t.role("tlv.hi").is_none()) {
        return;
    }
    let ptr = cx.m.types.ptr(0);
    let blocks: Vec<BlockId> = f.layout.clone();
    for b in blocks {
        let mut k = 0;
        while k < f.blocks[b.index()].insts.len() {
            let i = f.blocks[b.index()].insts[k];
            let Op::Call(call) = &f.insts[i.index()].op else {
                k += 1;
                continue;
            };
            if !callee_name(cx.m, call).is_some_and(|n| n.starts_with("llvm.threadlocal.address.")) {
                k += 1;
                continue;
            }
            let var = call.args[0].value;
            let r = f.insts[i.index()].result.unwrap();
            let mut s = Seq { f, new: Vec::new() };
            if tprel {
                let addr = call_intrinsic(cx, &mut s, "llvm.epitome.tprel", vec![var], ptr);
                s.f.replace_uses(Value::Local(r), addr);
                let new = std::mem::take(&mut s.new);
                let n = new.len();
                f.blocks[b.index()].insts.splice(k..=k, new);
                k += n;
                continue;
            }
            if elf || ie {
                let i64t = cx.m.types.int(64);
                let how = if ie { "llvm.epitome.tls.ie" } else { "llvm.epitome.tlsdesc" };
                let off = call_intrinsic(cx, &mut s, how, vec![var], i64t);
                let tp = call_intrinsic(cx, &mut s, "llvm.thread.pointer.p0", vec![], ptr);
                let i8t = cx.m.types.int(8);
                let addr = s.op(
                    Op::Gep { flags: Default::default(), inrange: None, source_ty: i8t, ptr: tp, indices: vec![off] },
                    ptr,
                );
                s.f.replace_uses(Value::Local(r), addr);
                let new = std::mem::take(&mut s.new);
                let n = new.len();
                f.blocks[b.index()].insts.splice(k..=k, new);
                k += n;
                continue;
            }
            let desc = call_intrinsic(cx, &mut s, "llvm.epitome.tlv", vec![var], ptr);
            let get = s.op(Op::Load { volatile: false, atomic: None, ptr: desc, align: Some(8) }, ptr);
            let fn_ty = cx.m.types.intern(Type::Function { ret: ptr, params: vec![ptr], varargs: false });
            let addr = s.op(
                Op::Call(Box::new(Call {
                    tail: TailKind::None,
                    fmf: FastMath::default(),
                    cconv: CallConv::default(),
                    ret_attrs: vec![],
                    addrspace: None,
                    fn_ty,
                    callee: get,
                    args: vec![CallArg { value: desc, attrs: vec![] }],
                    fn_attrs: vec![],
                    bundles: vec![],
                })),
                ptr,
            );
            s.f.replace_uses(Value::Local(r), addr);
            let new = std::mem::take(&mut s.new);
            let n = new.len();
            f.blocks[b.index()].insts.splice(k..=k, new);
            k += n;
        }
    }
}

/// Vectors of integers of odd widths (`<8 x i24>`, as SROA makes of
/// `[u8; 3]`s), where they only live in registers: each lane in the next
/// width (`i32`), its value in the low bits and the rest any. Operations
/// that read the bits above (right shifts, divisions, comparisons, wider
/// extensions) first make them the value's extension; lanes taken out are
/// truncated back to the odd width, for `promote_odd`. A function that
/// loads, stores, bitcasts or passes one is left as it is (and rejected).
fn odd_lane_vectors(cx: &mut Ctx, f: &mut Function) {
    let odd = |cx: &Ctx, ty: TypeId| -> Option<(u32, u32, u32)> {
        let Type::Vector { len, elem, scalable: false } = *cx.m.types.get(ty) else { return None };
        let Type::Int(b) = *cx.m.types.get(elem) else { return None };
        container(b).map(|c| (len, b, c))
    };
    let mut any = false;
    for i in f.insts_in_order().collect::<Vec<_>>() {
        let inst = &f.insts[i.index()];
        let mut touches = inst.result.is_some_and(|r| odd(cx, f.locals[r.index()].ty).is_some());
        inst.op.for_each_value(|v| {
            if !matches!(v, Value::Metadata(_)) {
                touches |= odd(cx, cx.value_ty(f, *v)).is_some();
            }
        });
        if !touches {
            continue;
        }
        any = true;
        let fine = matches!(
            inst.op,
            Op::Binary { .. }
                | Op::ICmp { .. }
                | Op::Select { .. }
                | Op::Phi { .. }
                | Op::Freeze(_)
                | Op::ShuffleVector { .. }
                | Op::ExtractElement { .. }
                | Op::InsertElement { .. }
                | Op::Cast { op: CastOp::ZExt | CastOp::SExt | CastOp::Trunc, .. }
        );
        if !fine {
            return;
        }
    }
    if !any || f.params.iter().any(|p| odd(cx, f.locals[p.local.index()].ty).is_some()) {
        return;
    }
    // The widened type of each odd vector type.
    let widen = |cx: &mut Ctx, ty: TypeId| -> Option<TypeId> {
        let (len, _, c) = odd(cx, ty)?;
        let e = cx.m.types.int(c);
        Some(cx.m.types.intern(Type::Vector { scalable: false, len, elem: e }))
    };
    // Constants: their lanes, zero-extended.
    let wide_const = |cx: &mut Ctx, k: ConstId| -> Option<ConstId> {
        let c = cx.m.constant(k).clone();
        let wt = widen(cx, c.ty)?;
        let Type::Vector { elem: we, .. } = *cx.m.types.get(wt) else { unreachable!() };
        let Type::Int(wb) = *cx.m.types.get(we) else { unreachable!() };
        let lane = |cx: &mut Ctx, l: ConstId| -> ConstId {
            let kind = match &cx.m.constant(l).kind {
                ConstKind::Int(a) => ConstKind::Int(ApInt::from_u64(wb, a.low_u64())),
                other => other.clone(),
            };
            cx.m.consts.intern(Constant { ty: we, kind })
        };
        let kind = match c.kind {
            ConstKind::Vector(ls) => ConstKind::Vector(ls.iter().map(|&l| lane(cx, l)).collect()),
            ConstKind::Splat(l) => ConstKind::Splat(lane(cx, l)),
            other => other,
        };
        Some(cx.m.consts.intern(Constant { ty: wt, kind }))
    };
    // The odd lanes' widths, before the types change.
    let mut bits: std::collections::HashMap<LocalId, (u32, u32)> = std::collections::HashMap::new();
    for k in 0..f.locals.len() {
        let t = f.locals[k].ty;
        if let Some((_, b, c)) = odd(cx, t) {
            bits.insert(LocalId(k as u32), (b, c));
            f.locals[k].ty = widen(cx, t).unwrap();
        }
    }
    let blocks: Vec<BlockId> = f.layout.clone();
    for b in blocks {
        let mut k = 0;
        while k < f.blocks[b.index()].insts.len() {
            let i = f.blocks[b.index()].insts[k];
            let mut op = f.insts[i.index()].op.clone();
            op.for_each_value_mut(|v| {
                if let Value::Const(c) = *v
                    && let Some(w) = wide_const(cx, c)
                {
                    *v = Value::Const(w);
                }
            });
            let width = |v: Value| match v {
                Value::Local(x) => bits.get(&x).copied(),
                _ => None,
            };
            let mut s = Seq { f, new: Vec::new() };
            // `v` (of odd lanes `b` wide in `c`) extended properly.
            let extend = |cx: &mut Ctx, s: &mut Seq, v: Value, b: u32, c: u32, signed: bool| -> Value {
                let ty = cx.value_ty(s.f, v);
                let Type::Vector { len, elem, .. } = *cx.m.types.get(ty) else { return v };
                let splat = |cx: &mut Ctx, n: i64| {
                    let l = cx.m.consts.intern(Constant { ty: elem, kind: ConstKind::Int(ApInt::from_i64(c, n)) });
                    let t = cx.m.types.intern(Type::Vector { scalable: false, len, elem });
                    Value::Const(cx.m.consts.intern(Constant { ty: t, kind: ConstKind::Splat(l) }))
                };
                if signed {
                    let sh = splat(cx, (c - b) as i64);
                    let up = s.bin(BinOp::Shl, v, sh, ty);
                    s.bin(BinOp::AShr, up, sh, ty)
                } else {
                    let m = splat(cx, ((1u64 << b) - 1) as i64);
                    s.bin(BinOp::And, v, m, ty)
                }
            };
            let res_odd = s.f.insts[i.index()].result.and_then(|r| bits.get(&r).copied());
            match op.clone() {
                Op::Binary { op: o, flags, fmf, lhs, rhs }
                    if matches!(
                        o,
                        BinOp::LShr | BinOp::AShr | BinOp::UDiv | BinOp::SDiv | BinOp::URem | BinOp::SRem
                    ) =>
                {
                    if let Some((bw, cw)) = res_odd {
                        let signed = matches!(o, BinOp::AShr | BinOp::SDiv | BinOp::SRem);
                        let l = extend(cx, &mut s, lhs, bw, cw, signed);
                        // (A shift's amount, a divisor: their values too.)
                        let r = extend(cx, &mut s, rhs, bw, cw, signed && o != BinOp::AShr);
                        op = Op::Binary { op: o, flags, fmf, lhs: l, rhs: r };
                    }
                }
                Op::ICmp { pred, samesign, lhs, rhs } => {
                    if let Some((bw, cw)) = width(lhs).or(width(rhs)) {
                        let signed = matches!(pred, ICmpPred::Slt | ICmpPred::Sle | ICmpPred::Sgt | ICmpPred::Sge);
                        let l = extend(cx, &mut s, lhs, bw, cw, signed);
                        let r = extend(cx, &mut s, rhs, bw, cw, signed);
                        op = Op::ICmp { pred, samesign, lhs: l, rhs: r };
                    }
                }
                Op::Cast { op: c @ (CastOp::ZExt | CastOp::SExt), flags, fmf, value } => {
                    if let Some((bw, cw)) = width(value) {
                        // From odd lanes: extended within them first; to
                        // as wide as they now are, that is all.
                        let e = extend(cx, &mut s, value, bw, cw, c == CastOp::SExt);
                        let rt = s.f.locals[s.f.insts[i.index()].result.unwrap().index()].ty;
                        let et = cx.value_ty(s.f, e);
                        op = if rt == et { Op::Freeze(e) } else { Op::Cast { op: c, flags, fmf, value: e } };
                    }
                }
                Op::ExtractElement { vector, index } if width(vector).is_some() => {
                    // The lane, back at its odd width.
                    let (_, cw) = width(vector).unwrap();
                    let wt = cx.m.types.int(cw);
                    let x = s.op(Op::ExtractElement { vector, index }, wt);
                    op = Op::Cast { op: CastOp::Trunc, flags: IntFlags::NONE, fmf: FastMath::default(), value: x };
                }
                Op::InsertElement { vector, element, index } if res_odd.is_some() => {
                    let (_, cw) = res_odd.unwrap();
                    let wt = cx.m.types.int(cw);
                    let e = s.cast(CastOp::ZExt, element, wt);
                    op = Op::InsertElement { vector, element: e, index };
                }
                _ => {}
            }
            s.f.insts[i.index()].op = op;
            let new = std::mem::take(&mut s.new);
            let n = new.len();
            f.blocks[b.index()].insts.splice(k..k, new);
            k += n + 1;
        }
    }
}

/// How a vector no register holds is passed and returned (as LLVM's
/// AArch64 backend passes them, which clang's IR leaves to it): its lanes
/// made a power of two (`<3 x float>` as `<4 x float>`), narrow lanes
/// widened to fill 64 bits (`<2 x i8>` as `<2 x i32>`), and then, if wider
/// than a register, 128-bit parts, each an argument of its own (returned
/// together, as a struct of them). `None` for those a register holds.
struct VectorAbi {
    /// The vector, made a power of two and widened.
    whole: TypeId,
    lanes: u32,
    promoted: bool,
    part: TypeId,
    parts: usize,
}

fn vector_abi(cx: &mut Ctx, ty: TypeId) -> Option<VectorAbi> {
    let Type::Vector { scalable: false, len, elem } = *cx.m.types.get(ty) else { return None };
    let eb = match cx.m.types.get(elem) {
        Type::Int(b @ (8 | 16 | 32 | 64)) => *b,
        Type::Float => 32,
        Type::Double | Type::Ptr(0) => 64,
        _ => return None,
    };
    if len.is_power_of_two() && matches!(len * eb, 64 | 128) {
        return None;
    }
    let lanes = len.next_power_of_two();
    let (elem, eb, promoted) = if lanes * eb < 64 {
        let wider = 64 / lanes;
        if !matches!(cx.m.types.get(elem), Type::Int(_)) || wider > 64 {
            return None;
        }
        (cx.m.types.int(wider), wider, true)
    } else {
        (elem, eb, false)
    };
    let whole = cx.m.types.intern(Type::Vector { scalable: false, len: lanes, elem });
    let bits = lanes * eb;
    let (part, parts) = if bits <= 128 {
        (whole, 1)
    } else {
        (cx.m.types.intern(Type::Vector { scalable: false, len: 128 / eb, elem }), (bits / 128) as usize)
    };
    Some(VectorAbi { whole, lanes, promoted, part, parts })
}

/// A shuffle of `a` and `b` taking `lanes`, of type `ty`.
fn shuffle_lanes(cx: &mut Ctx, s: &mut Seq, a: Value, b: Value, lanes: &[u32], ty: TypeId) -> Value {
    let i32t = cx.m.types.int(32);
    let elems: Vec<ConstId> = lanes
        .iter()
        .map(|&l| cx.m.consts.intern(Constant { ty: i32t, kind: ConstKind::Int(ApInt::from_i64(32, l as i64)) }))
        .collect();
    let mty = cx.m.types.intern(Type::Vector { scalable: false, len: lanes.len() as u32, elem: i32t });
    let mask = cx.m.consts.intern(Constant { ty: mty, kind: ConstKind::Vector(elems) });
    s.op(Op::ShuffleVector { a, b, mask }, ty)
}

/// Vector `v` (of type `ty`) as its ABI parts.
fn to_abi_parts(cx: &mut Ctx, s: &mut Seq, v: Value, ty: TypeId, a: &VectorAbi) -> Vec<Value> {
    let Type::Vector { len, elem, .. } = *cx.m.types.get(ty) else { unreachable!() };
    let mut w = v;
    if a.lanes != len {
        // The lanes added are any: the first's again.
        let lanes: Vec<u32> = (0..a.lanes).map(|l| if l < len { l } else { 0 }).collect();
        let wt = cx.m.types.intern(Type::Vector { scalable: false, len: a.lanes, elem });
        let p = Value::Const(cx.m.consts.intern(Constant { ty, kind: ConstKind::Poison }));
        w = shuffle_lanes(cx, s, v, p, &lanes, wt);
    }
    if a.promoted {
        w = s.cast(CastOp::ZExt, w, a.whole);
    }
    if a.parts == 1 {
        return vec![w];
    }
    let per = a.lanes / a.parts as u32;
    let p = Value::Const(cx.m.consts.intern(Constant { ty: a.whole, kind: ConstKind::Poison }));
    (0..a.parts as u32)
        .map(|k| {
            let lanes: Vec<u32> = (k * per..(k + 1) * per).collect();
            shuffle_lanes(cx, s, w, p, &lanes, a.part)
        })
        .collect()
}

/// The vector of type `ty` its ABI parts make.
fn from_abi_parts(cx: &mut Ctx, s: &mut Seq, parts: Vec<Value>, ty: TypeId, a: &VectorAbi) -> Value {
    let Type::Vector { len, elem, .. } = *cx.m.types.get(ty) else { unreachable!() };
    // Halves joined, then those, to the whole.
    let mut level = parts;
    while level.len() > 1 {
        let n = cx.m.types.get(cx.value_ty(s.f, level[0])).clone();
        let Type::Vector { len: l, elem: e, .. } = n else { unreachable!() };
        let jt = cx.m.types.intern(Type::Vector { scalable: false, len: 2 * l, elem: e });
        let lanes: Vec<u32> = (0..2 * l).collect();
        level = level.chunks(2).map(|pair| shuffle_lanes(cx, s, pair[0], pair[1], &lanes, jt)).collect();
    }
    let mut v = level[0];
    if a.promoted {
        let nt = cx.m.types.intern(Type::Vector { scalable: false, len: a.lanes, elem });
        v = s.cast(CastOp::Trunc, v, nt);
    }
    if a.lanes != len {
        let vt = cx.value_ty(s.f, v);
        let p = Value::Const(cx.m.consts.intern(Constant { ty: vt, kind: ConstKind::Poison }));
        let lanes: Vec<u32> = (0..len).collect();
        v = shuffle_lanes(cx, s, v, p, &lanes, ty);
    }
    v
}

/// The type a result of type `ty` is returned as: its part, or a struct of
/// its parts.
fn abi_result_ty(cx: &mut Ctx, a: &VectorAbi) -> TypeId {
    if a.parts == 1 { a.part } else { cx.m.types.intern(Type::Struct { packed: false, fields: vec![a.part; a.parts] }) }
}

/// Vector parameters, arguments and results no register holds, in the
/// form they are passed in (see `vector_abi`), where the convention has
/// AArch64's vector registers: the function's own (taken apart at its
/// entry and put together at its returns), and its calls' (other than
/// intrinsics' and variable arguments).
fn vector_arguments(cx: &mut Ctx, f: &mut Function) {
    if !cx.hfa || cx.vector_bits() != 128 {
        return;
    }
    let Type::Function { ret, varargs, .. } = cx.m.types.get(f.ty).clone() else { return };
    let entry = f.layout[0];
    // Parameters.
    let mut new_params = Vec::new();
    let mut new_tys = Vec::new();
    let mut entry_new = Vec::new();
    for p in f.params.clone() {
        let ty = f.locals[p.local.index()].ty;
        let Some(a) = vector_abi(cx, ty) else {
            new_tys.push(ty);
            new_params.push(p);
            continue;
        };
        let mut parts = Vec::new();
        for _ in 0..a.parts {
            let l = f.add_local(None, a.part, LocalDef::Param(0));
            parts.push(Value::Local(l));
            new_tys.push(a.part);
            new_params.push(Param { local: l, attrs: vec![] });
        }
        let mut s = Seq { f, new: Vec::new() };
        let v = from_abi_parts(cx, &mut s, parts, ty, &a);
        entry_new.extend(std::mem::take(&mut s.new));
        f.replace_uses(Value::Local(p.local), v);
    }
    let changed =
        new_params.len() != f.params.len() || new_params.iter().zip(&f.params).any(|(a, b)| a.local != b.local);
    let ret_abi = vector_abi(cx, ret);
    if changed || ret_abi.is_some() {
        for (k, p) in new_params.iter().enumerate() {
            f.locals[p.local.index()].def = LocalDef::Param(k as u32);
        }
        f.params = new_params;
        let new_ret = match &ret_abi {
            Some(a) => abi_result_ty(cx, a),
            None => ret,
        };
        f.ty = cx.m.types.intern(Type::Function { ret: new_ret, params: new_tys, varargs });
        let at = f.blocks[entry.index()].insts.iter().take_while(|&&i| f.insts[i.index()].op.is_phi()).count();
        f.blocks[entry.index()].insts.splice(at..at, entry_new);
    }
    // Returns, and calls.
    for b in f.layout.clone() {
        let mut k = 0;
        while k < f.blocks[b.index()].insts.len() {
            let i = f.blocks[b.index()].insts[k];
            match f.insts[i.index()].op.clone() {
                Op::Ret(Some(v)) if ret_abi.is_some() => {
                    let a = ret_abi.as_ref().unwrap();
                    let mut s = Seq { f, new: Vec::new() };
                    let parts = to_abi_parts(cx, &mut s, v, ret, a);
                    let r = if a.parts == 1 {
                        parts[0]
                    } else {
                        let st = abi_result_ty(cx, a);
                        let mut agg = Value::Const(cx.m.consts.intern(Constant { ty: st, kind: ConstKind::Poison }));
                        for (n, p) in parts.into_iter().enumerate() {
                            agg = s.op(Op::InsertValue { aggregate: agg, element: p, indices: vec![n as u32] }, st);
                        }
                        agg
                    };
                    let new = std::mem::take(&mut s.new);
                    f.insts[i.index()].op = Op::Ret(Some(r));
                    let n = new.len();
                    f.blocks[b.index()].insts.splice(k..k, new);
                    k += n + 1;
                }
                Op::Call(_) | Op::Invoke { .. } => {
                    k += 1 + call_vectors(cx, f, b, k);
                }
                _ => k += 1,
            }
        }
    }
}

/// The call at `k` in block `b`, its vector arguments and result passed as
/// `vector_abi` says; how many instructions were added after it in `b`.
fn call_vectors(cx: &mut Ctx, f: &mut Function, b: BlockId, k: usize) -> usize {
    let i = f.blocks[b.index()].insts[k];
    let call = f.insts[i.index()].op.call().unwrap().clone();
    let asm = matches!(call.callee, Value::Const(c) if matches!(cx.m.constant(c).kind, ConstKind::InlineAsm(_)));
    if asm || callee_name(cx.m, &call).is_some_and(|n| n.starts_with("llvm.")) {
        return 0;
    }
    let Type::Function { ret, params, varargs } = cx.m.types.get(call.fn_ty).clone() else { return 0 };
    let ret_abi = vector_abi(cx, ret);
    let any_arg = params.iter().any(|&t| vector_abi(cx, t).is_some());
    if !any_arg && ret_abi.is_none() {
        return 0;
    }
    let mut s = Seq { f, new: Vec::new() };
    let mut args = Vec::new();
    let mut tys = Vec::new();
    for (n, a) in call.args.iter().enumerate() {
        let ty = cx.value_ty(s.f, a.value);
        match (n < params.len()).then(|| vector_abi(cx, ty)).flatten() {
            Some(abi) => {
                for p in to_abi_parts(cx, &mut s, a.value, ty, &abi) {
                    args.push(CallArg { value: p, attrs: vec![] });
                    tys.push(abi.part);
                }
            }
            None => {
                if n < params.len() {
                    tys.push(params[n]);
                }
                args.push(a.clone());
            }
        }
    }
    let before = std::mem::take(&mut s.new);
    let new_ret = match &ret_abi {
        Some(a) => abi_result_ty(cx, a),
        None => ret,
    };
    let fn_ty = cx.m.types.intern(Type::Function { ret: new_ret, params: tys, varargs });
    let mut new_call = call.clone();
    new_call.args = args;
    new_call.fn_ty = fn_ty;
    if let Some(c) = f.insts[i.index()].op.call_mut() {
        *c = new_call;
    }
    f.blocks[b.index()].insts.splice(k..k, before.iter().copied());
    let k = k + before.len();
    // The result, put together again after the call (an invoke's, on its
    // normal edge, in a block of its own).
    let (Some(a), Some(r)) = (ret_abi, f.insts[i.index()].result) else { return before.len() };
    let orig_ty = f.locals[r.index()].ty;
    let new_r = f.add_local(None, new_ret, LocalDef::Inst(i));
    f.insts[i.index()].result = Some(new_r);
    let mut s = Seq { f, new: Vec::new() };
    let parts: Vec<Value> = if a.parts == 1 {
        vec![Value::Local(new_r)]
    } else {
        (0..a.parts as u32)
            .map(|n| s.op(Op::ExtractValue { aggregate: Value::Local(new_r), indices: vec![n] }, a.part))
            .collect()
    };
    let v = from_abi_parts(cx, &mut s, parts, orig_ty, &a);
    let after = std::mem::take(&mut s.new);
    f.replace_uses(Value::Local(r), v);
    match f.insts[i.index()].op.clone() {
        Op::Invoke { normal, .. } => {
            let e = f.add_block(None);
            let at = f.layout.iter().position(|&x| x == b).unwrap();
            f.layout.insert(at + 1, e);
            f.blocks[e.index()].insts.extend(after);
            f.push_inst(e, Op::Br(normal), None);
            if let Op::Invoke { normal: nb, .. } = &mut f.insts[i.index()].op {
                *nb = e;
            }
            f.retarget_phis(normal, b, e);
            before.len()
        }
        _ => {
            let n = after.len();
            f.blocks[b.index()].insts.splice(k + 1..k + 1, after);
            before.len() + n
        }
    }
}

/// Masks split across registers and the integers of their bits: a load
/// or store of one of more than 16 lanes is of its integer (memory holds a
/// mask as its bits, packed); and a bitcast between one of more than 64
/// lanes and its integer is made 64 lanes at a time, through bytes (which
/// split as masks' shuffles do not): the mask's lanes as bytes, each 64 of
/// them compared with zero again and taken as the `i64` of their bits, as
/// smaller masks are; and the other way, each `i64` as a mask, as bytes,
/// joined and compared. (Lane counts not a multiple of 64 are left.)
fn wide_mask_bits(cx: &mut Ctx, f: &mut Function) {
    let i1 = cx.m.types.int(1);
    let i8t = cx.m.types.int(8);
    let mask_len = |cx: &Ctx, ty: TypeId| match cx.m.types.get(ty) {
        Type::Vector { len, elem, scalable: false } if *elem == i1 => Some(*len),
        _ => None,
    };
    if !f.locals.iter().any(|l| mask_len(cx, l.ty).is_some_and(|n| n > 16)) {
        return;
    }
    let vec = |cx: &mut Ctx, len: u32, elem: TypeId| cx.m.types.intern(Type::Vector { scalable: false, len, elem });
    let blocks: Vec<BlockId> = f.layout.clone();
    // Loads and stores, of the integers.
    for &b in &blocks {
        let mut k = 0;
        while k < f.blocks[b.index()].insts.len() {
            let i = f.blocks[b.index()].insts[k];
            let mut s = Seq { f, new: Vec::new() };
            match s.f.insts[i.index()].op.clone() {
                Op::Store { volatile, atomic: None, value, ptr, align } => {
                    let vt = cx.value_ty(s.f, value);
                    if let Some(n) = mask_len(cx, vt).filter(|&n| n > 16) {
                        let it = cx.m.types.int(n);
                        let bits = s.cast(CastOp::BitCast, value, it);
                        s.f.insts[i.index()].op = Op::Store { volatile, atomic: None, value: bits, ptr, align };
                    }
                }
                Op::Load { volatile, atomic: None, ptr, align } => {
                    let r = s.f.insts[i.index()].result.unwrap();
                    let rt = s.f.locals[r.index()].ty;
                    if let Some(n) = mask_len(cx, rt).filter(|&n| n > 16) {
                        let it = cx.m.types.int(n);
                        let ld = s.f.add_inst(Op::Load { volatile, atomic: None, ptr, align }, Some((None, it)));
                        let lv = s.f.result(ld);
                        s.f.insts[i.index()].op = Op::Cast {
                            op: CastOp::BitCast,
                            flags: IntFlags::NONE,
                            fmf: FastMath::default(),
                            value: lv,
                        };
                        s.new.push(ld);
                    }
                }
                _ => {}
            }
            let new = std::mem::take(&mut s.new);
            let n = new.len();
            f.blocks[b.index()].insts.splice(k..k, new);
            k += n + 1;
        }
    }
    // Bitcasts, 64 lanes at a time.
    for &b in &blocks {
        let mut k = 0;
        while k < f.blocks[b.index()].insts.len() {
            let i = f.blocks[b.index()].insts[k];
            let Op::Cast { op: CastOp::BitCast, value, .. } = f.insts[i.index()].op else {
                k += 1;
                continue;
            };
            let r = f.insts[i.index()].result.unwrap();
            let (from, to) = (cx.value_ty(f, value), f.locals[r.index()].ty);
            let wide = |n: Option<u32>| n.filter(|&n| n > 64 && n % 64 == 0);
            let int = |cx: &Ctx, t: TypeId| matches!(cx.m.types.get(t), Type::Int(_));
            let (fm, tm) = (wide(mask_len(cx, from)), wide(mask_len(cx, to)));
            let mut s = Seq { f, new: Vec::new() };
            let v = match (fm, tm) {
                (Some(n), None) if int(cx, to) => {
                    let bt = vec(cx, n, i8t);
                    let bytes = s.cast(CastOp::SExt, value, bt);
                    let (c64, m64, i64t) = (vec(cx, 64, i8t), vec(cx, 64, i1), cx.m.types.int(64));
                    let zero = Value::Const(cx.m.consts.intern(Constant { ty: c64, kind: ConstKind::Zero }));
                    let p = Value::Const(cx.m.consts.intern(Constant { ty: bt, kind: ConstKind::Poison }));
                    let mut acc = cx.int(to, 0);
                    for c in 0..n / 64 {
                        let lanes: Vec<u32> = (c * 64..c * 64 + 64).collect();
                        let chunk = shuffle_lanes(cx, &mut s, bytes, p, &lanes, c64);
                        let m = s.op(Op::ICmp { pred: ICmpPred::Ne, samesign: false, lhs: chunk, rhs: zero }, m64);
                        let bits = s.cast(CastOp::BitCast, m, i64t);
                        let w = s.cast(CastOp::ZExt, bits, to);
                        let sh = cx.int(to, (c * 64) as i64);
                        let placed = s.bin(BinOp::Shl, w, sh, to);
                        acc = s.bin(BinOp::Or, acc, placed, to);
                    }
                    Some(acc)
                }
                (None, Some(n)) if int(cx, from) => {
                    let (c64, m64, i64t) = (vec(cx, 64, i8t), vec(cx, 64, i1), cx.m.types.int(64));
                    let mut level: Vec<Value> = Vec::new();
                    for c in 0..n / 64 {
                        let sh = cx.int(from, (c * 64) as i64);
                        let moved = s.bin(BinOp::LShr, value, sh, from);
                        let bits = s.cast(CastOp::Trunc, moved, i64t);
                        let m = s.cast(CastOp::BitCast, bits, m64);
                        level.push(s.cast(CastOp::SExt, m, c64));
                    }
                    let mut len = 64;
                    while level.len() > 1 {
                        let jt = vec(cx, 2 * len, i8t);
                        let lanes: Vec<u32> = (0..2 * len).collect();
                        level = level.chunks(2).map(|p| shuffle_lanes(cx, &mut s, p[0], p[1], &lanes, jt)).collect();
                        len *= 2;
                    }
                    let bt = vec(cx, n, i8t);
                    let zero = Value::Const(cx.m.consts.intern(Constant { ty: bt, kind: ConstKind::Zero }));
                    Some(s.op(Op::ICmp { pred: ICmpPred::Ne, samesign: false, lhs: level[0], rhs: zero }, to))
                }
                _ => None,
            };
            let Some(v) = v else {
                k += 1;
                continue;
            };
            let new = std::mem::take(&mut s.new);
            f.replace_uses(Value::Local(r), v);
            let n = new.len();
            f.blocks[b.index()].insts.splice(k..=k, new);
            k += n;
        }
    }
}

/// Masks moved about as the integers of their lanes, and chosen between by
/// logic: a shuffle of masks is a shuffle of them sign-extended to the
/// width of the comparison that made them (so that what uses it sees the
/// same width), compared with zero again; a select of one mask or another
/// by a mask is `(c & t) | (!c & e)`. And `llvm.vector.partial.reduce.add`,
/// which may add the input's lanes into the accumulator's in any way: the
/// accumulator plus each of the input's parts of its width.
fn mask_logic(cx: &mut Ctx, f: &mut Function) {
    let i1 = cx.m.types.int(1);
    let mask_len = |cx: &Ctx, t: TypeId| match cx.m.types.get(t) {
        Type::Vector { len, elem, scalable: false } if *elem == i1 => Some(*len),
        _ => None,
    };
    let blocks: Vec<BlockId> = f.layout.clone();
    for b in blocks {
        let mut k = 0;
        while k < f.blocks[b.index()].insts.len() {
            let i = f.blocks[b.index()].insts[k];
            let Some(r) = f.insts[i.index()].result else {
                k += 1;
                continue;
            };
            let rt = f.locals[r.index()].ty;
            let op = f.insts[i.index()].op.clone();
            let mut s = Seq { f, new: Vec::new() };
            let v = match op {
                Op::ShuffleVector { a, b: bv, mask } if mask_len(cx, cx.value_ty(s.f, a)).is_some() => {
                    let n = mask_len(cx, cx.value_ty(s.f, a)).unwrap();
                    let m = mask_len(cx, rt).unwrap();
                    let w = crate::legalize::mask_width(cx, s.f, a, &mut Vec::new())
                        .or_else(|| crate::legalize::mask_width(cx, s.f, bv, &mut Vec::new()))
                        .unwrap_or(8)
                        .max(8);
                    // Not of masks whose lanes as integers would be narrower
                    // than a register (which split lane by lane): their
                    // shuffles are selection's, lanes chosen by poison made
                    // the first.
                    if n * w < 64 || m * w < 64 {
                        let mk = cx.m.constant(mask).clone();
                        if let ConstKind::Vector(es) = &mk.kind
                            && es.iter().any(|&e| matches!(cx.m.constant(e).kind, ConstKind::Poison | ConstKind::Undef))
                        {
                            let i32t = cx.m.types.int(32);
                            let zero =
                                cx.m.consts.intern(Constant { ty: i32t, kind: ConstKind::Int(ApInt::from_i64(32, 0)) });
                            let es: Vec<ConstId> = es
                                .iter()
                                .map(|&e| {
                                    if matches!(cx.m.constant(e).kind, ConstKind::Poison | ConstKind::Undef) {
                                        zero
                                    } else {
                                        e
                                    }
                                })
                                .collect();
                            let nm = cx.m.consts.intern(Constant { ty: mk.ty, kind: ConstKind::Vector(es) });
                            s.f.insts[i.index()].op = Op::ShuffleVector { a, b: bv, mask: nm };
                        }
                        k += 1;
                        continue;
                    }
                    let e = cx.m.types.int(w);
                    let (st, dt) = (
                        cx.m.types.intern(Type::Vector { scalable: false, len: n, elem: e }),
                        cx.m.types.intern(Type::Vector { scalable: false, len: m, elem: e }),
                    );
                    // (A constant's lanes extended as constants: poison, zero,
                    // all ones.)
                    let widen = |cx: &mut Ctx, s: &mut Seq, v: Value| match v {
                        Value::Const(c) => {
                            let kind = match &cx.m.constant(c).kind {
                                ConstKind::Poison | ConstKind::Undef => Some(ConstKind::Poison),
                                ConstKind::Zero => Some(ConstKind::Zero),
                                _ => None,
                            };
                            match kind {
                                Some(kind) => Value::Const(cx.m.consts.intern(Constant { ty: st, kind })),
                                None => s.cast(CastOp::SExt, v, st),
                            }
                        }
                        v => s.cast(CastOp::SExt, v, st),
                    };
                    let (x, y) = (widen(cx, &mut s, a), widen(cx, &mut s, bv));
                    let sh = s.op(Op::ShuffleVector { a: x, b: y, mask }, dt);
                    let zero = Value::Const(cx.m.consts.intern(Constant { ty: dt, kind: ConstKind::Zero }));
                    Some(s.op(Op::ICmp { pred: ICmpPred::Ne, samesign: false, lhs: sh, rhs: zero }, rt))
                }
                // A lane of a constant, by a constant: the lane.
                Op::ExtractElement { vector: Value::Const(c), index } if cx.const_int(index).is_some() => {
                    let l = cx.const_int(index).unwrap() as usize;
                    match cx.m.constant(c).kind.clone() {
                        ConstKind::Vector(es) if l < es.len() => Some(Value::Const(es[l])),
                        ConstKind::Splat(e) => Some(Value::Const(e)),
                        ConstKind::Zero | ConstKind::Poison | ConstKind::Undef => {
                            let kind = cx.m.constant(c).kind.clone();
                            Some(Value::Const(cx.m.consts.intern(Constant { ty: rt, kind })))
                        }
                        _ => None,
                    }
                }
                Op::Select { cond, then_value, else_value, .. }
                    if mask_len(cx, rt).is_some() && mask_len(cx, cx.value_ty(s.f, cond)).is_some() =>
                {
                    let ones = cx.m.consts.intern(Constant { ty: i1, kind: ConstKind::Int(ApInt::from_i64(1, -1)) });
                    let all = Value::Const(cx.m.consts.intern(Constant { ty: rt, kind: ConstKind::Splat(ones) }));
                    let t = s.bin(BinOp::And, cond, then_value, rt);
                    let nc = s.bin(BinOp::Xor, cond, all, rt);
                    let e = s.bin(BinOp::And, nc, else_value, rt);
                    Some(s.bin(BinOp::Or, t, e, rt))
                }
                Op::Call(call)
                    if callee_name(cx.m, &call).is_some_and(|n| n.starts_with("llvm.vector.partial.reduce.add."))
                        && call.args.len() == 2 =>
                {
                    let (acc, x) = (call.args[0].value, call.args[1].value);
                    let (at, xt) = (cx.value_ty(s.f, acc), cx.value_ty(s.f, x));
                    match (cx.m.types.get(at).clone(), cx.m.types.get(xt).clone()) {
                        (Type::Vector { len: an, .. }, Type::Vector { len: xn, .. }) if xn % an == 0 => {
                            let p = Value::Const(cx.m.consts.intern(Constant { ty: xt, kind: ConstKind::Poison }));
                            let mut sum = acc;
                            for c in 0..xn / an {
                                let lanes: Vec<u32> = (c * an..(c + 1) * an).collect();
                                let part = shuffle_lanes(cx, &mut s, x, p, &lanes, at);
                                sum = s.bin(BinOp::Add, sum, part, at);
                            }
                            Some(sum)
                        }
                        _ => None,
                    }
                }
                _ => None,
            };
            let Some(v) = v else {
                k += 1;
                continue;
            };
            let new = std::mem::take(&mut s.new);
            f.replace_uses(Value::Local(r), v);
            let n = new.len();
            f.blocks[b.index()].insts.splice(k..=k, new);
            k += n;
        }
    }
}

/// The halves of the 256-bit product of two 128-bit integers, which is how
/// Rust's `u128` multiplications widen (`mul (zext a), (zext b)` in 256
/// bits): from the four 64-bit products of their halves, in 128 bits
/// (which `wide.rs` does), made once after the product; what uses it in
/// terms of its halves (`trunc`s of it and of it shifted down 128, a
/// comparison of its sign or of its high half with zero) uses them. What
/// else is left of 256 bits is unused and removed.
fn wide_products(cx: &mut Ctx, f: &mut Function) {
    let (i64t, i128t, i256t, i1) = (cx.m.types.int(64), cx.m.types.int(128), cx.m.types.int(256), cx.m.types.int(1));
    if !f.locals.iter().any(|l| l.ty == i256t) {
        return;
    }
    let def = |f: &Function, v: Value| -> Option<Op> {
        let Value::Local(x) = v else { return None };
        let LocalDef::Inst(d) = f.locals[x.index()].def else { return None };
        Some(f.insts[d.index()].op.clone())
    };
    // The 128-bit integer `v` was extended from (or is, a constant that
    // fits).
    let narrow = |cx: &mut Ctx, f: &Function, v: Value| -> Option<Value> {
        if let Value::Const(c) = v
            && let ConstKind::Int(a) = &cx.m.constant(c).kind
        {
            let w = a.words();
            if w.iter().skip(2).any(|&x| x != 0) {
                return None;
            }
            let hex = format!("{:016x}{:016x}", w.get(1).copied().unwrap_or(0), w[0]);
            let n = ApInt::parse_hex(128, &hex, false)?;
            return Some(Value::Const(cx.m.consts.intern(Constant { ty: i128t, kind: ConstKind::Int(n) })));
        }
        match def(f, v)? {
            Op::Cast { op: CastOp::ZExt, value, .. } if cx.value_ty(f, value) == i128t => Some(value),
            _ => None,
        }
    };
    // Each product's halves, made after it.
    let mut halves: std::collections::HashMap<LocalId, (Value, Value)> = std::collections::HashMap::new();
    for b in f.layout.clone() {
        let mut k = 0;
        while k < f.blocks[b.index()].insts.len() {
            let i = f.blocks[b.index()].insts[k];
            k += 1;
            let Op::Binary { op: BinOp::Mul, lhs, rhs, .. } = f.insts[i.index()].op else { continue };
            let r = f.insts[i.index()].result.unwrap();
            if f.locals[r.index()].ty != i256t {
                continue;
            }
            let (Some(a), Some(bv)) = (narrow(cx, f, lhs), narrow(cx, f, rhs)) else { continue };
            let mut s = Seq { f, new: Vec::new() };
            let k64 = cx.int(i128t, 64);
            let half = |s: &mut Seq, x: Value, up: bool| {
                let x = if up { s.bin(BinOp::LShr, x, k64, i128t) } else { x };
                let t = s.cast(CastOp::Trunc, x, i64t);
                s.cast(CastOp::ZExt, t, i128t)
            };
            let lo = s.bin(BinOp::Mul, a, bv, i128t);
            let (a0, a1) = (half(&mut s, a, false), half(&mut s, a, true));
            let (b0, b1) = (half(&mut s, bv, false), half(&mut s, bv, true));
            let p00 = s.bin(BinOp::Mul, a0, b0, i128t);
            let p01 = s.bin(BinOp::Mul, a0, b1, i128t);
            let p10 = s.bin(BinOp::Mul, a1, b0, i128t);
            let p11 = s.bin(BinOp::Mul, a1, b1, i128t);
            let carry = s.bin(BinOp::LShr, p00, k64, i128t);
            let l01 = half(&mut s, p01, false);
            let l10 = half(&mut s, p10, false);
            let mid = s.bin(BinOp::Add, carry, l01, i128t);
            let mid = s.bin(BinOp::Add, mid, l10, i128t);
            let h01 = s.bin(BinOp::LShr, p01, k64, i128t);
            let h10 = s.bin(BinOp::LShr, p10, k64, i128t);
            let hm = s.bin(BinOp::LShr, mid, k64, i128t);
            let h = s.bin(BinOp::Add, p11, h01, i128t);
            let h = s.bin(BinOp::Add, h, h10, i128t);
            let hi = s.bin(BinOp::Add, h, hm, i128t);
            let new = std::mem::take(&mut s.new);
            let n = new.len();
            f.blocks[b.index()].insts.splice(k..k, new);
            k += n;
            halves.insert(r, (hi, lo));
        }
    }
    if halves.is_empty() {
        return;
    }
    // The product of `v`, and whether `v` is it shifted down 128.
    let product = |cx: &Ctx, f: &Function, v: Value| -> Option<(LocalId, bool)> {
        let Value::Local(x) = v else { return None };
        if halves.contains_key(&x) {
            return Some((x, false));
        }
        match def(f, v)? {
            Op::Binary { op: BinOp::LShr, lhs: Value::Local(p), rhs, .. }
                if halves.contains_key(&p) && cx.const_int(rhs) == Some(128) =>
            {
                Some((p, true))
            }
            _ => None,
        }
    };
    let blocks: Vec<BlockId> = f.layout.clone();
    for b in blocks {
        let mut k = 0;
        while k < f.blocks[b.index()].insts.len() {
            let i = f.blocks[b.index()].insts[k];
            let Some(r) = f.insts[i.index()].result else {
                k += 1;
                continue;
            };
            let mut s = Seq { f, new: Vec::new() };
            let v = match s.f.insts[i.index()].op.clone() {
                Op::Cast { op: CastOp::Trunc, value, .. } if s.f.locals[r.index()].ty == i128t => {
                    product(cx, s.f, value).map(|(p, high)| if high { halves[&p].0 } else { halves[&p].1 })
                }
                Op::ICmp { pred, lhs, rhs, .. } => match (product(cx, s.f, lhs), cx.const_int(rhs)) {
                    // Its sign: its high half's.
                    (Some((p, false)), Some(-1)) if pred == ICmpPred::Sgt => {
                        let m1 = cx.int(i128t, -1);
                        Some(s.op(Op::ICmp { pred, samesign: false, lhs: halves[&p].0, rhs: m1 }, i1))
                    }
                    (Some((p, false)), Some(0)) if pred == ICmpPred::Slt => {
                        let z = cx.int(i128t, 0);
                        Some(s.op(Op::ICmp { pred, samesign: false, lhs: halves[&p].0, rhs: z }, i1))
                    }
                    // Its high half with zero.
                    (Some((p, true)), Some(0)) if matches!(pred, ICmpPred::Eq | ICmpPred::Ne) => {
                        let z = cx.int(i128t, 0);
                        Some(s.op(Op::ICmp { pred, samesign: false, lhs: halves[&p].0, rhs: z }, i1))
                    }
                    // All of it with zero.
                    (Some((p, false)), Some(0)) if matches!(pred, ICmpPred::Eq | ICmpPred::Ne) => {
                        let (h, l) = halves[&p];
                        let o = s.bin(BinOp::Or, h, l, i128t);
                        let z = cx.int(i128t, 0);
                        Some(s.op(Op::ICmp { pred, samesign: false, lhs: o, rhs: z }, i1))
                    }
                    _ => None,
                },
                _ => None,
            };
            let Some(v) = v else {
                k += 1;
                continue;
            };
            let new = std::mem::take(&mut s.new);
            f.replace_uses(Value::Local(r), v);
            let n = new.len();
            f.blocks[b.index()].insts.splice(k..=k, new);
            k += n;
        }
    }
    // What is left of 256 bits, now unused.
    loop {
        let mut used = vec![0usize; f.locals.len()];
        for i in f.insts_in_order() {
            f.insts[i.index()].op.for_each_value(|v| {
                if let Value::Local(x) = v {
                    used[x.index()] += 1;
                }
            });
        }
        let mut gone = false;
        for b in f.layout.clone() {
            let before = f.blocks[b.index()].insts.len();
            let keep: Vec<InstId> = f.blocks[b.index()]
                .insts
                .iter()
                .copied()
                .filter(|&i| {
                    let inst = &f.insts[i.index()];
                    !(inst.result.is_some_and(|r| f.locals[r.index()].ty == i256t && used[r.index()] == 0)
                        && matches!(inst.op, Op::Cast { .. } | Op::Binary { .. }))
                })
                .collect();
            gone |= keep.len() != before;
            f.blocks[b.index()].insts = keep;
        }
        if !gone {
            break;
        }
    }
}

/// Lanes of a mask taken as a vector of narrow integers (`extractelement
/// (bitcast <4 x i1> %m to <2 x i2>), 0`, as glam tests two lanes at
/// once), at constant indices: the mask's bits as an integer, shifted down
/// and truncated. The bitcast is removed.
fn fields_of_masks(cx: &mut Ctx, f: &mut Function) {
    let i1 = cx.m.types.int(1);
    for i in f.insts_in_order().collect::<Vec<_>>() {
        let Op::Cast { op: CastOp::BitCast, value, .. } = f.insts[i.index()].op else { continue };
        let Some(r) = f.insts[i.index()].result else { continue };
        let (Type::Vector { len: n, elem: me, scalable: false }, Type::Vector { len: m, elem: fe, scalable: false }) =
            (cx.m.types.get(cx.value_ty(f, value)).clone(), cx.m.types.get(f.locals[r.index()].ty).clone())
        else {
            continue;
        };
        let Type::Int(w) = *cx.m.types.get(fe) else { continue };
        if me != i1 || w == 1 || n > 64 || m * w != n {
            continue;
        }
        let mut users = Vec::new();
        let mut other = false;
        for j in f.insts_in_order() {
            let mut uses = false;
            f.insts[j.index()].op.for_each_value(|v| uses |= *v == Value::Local(r));
            if uses {
                match f.insts[j.index()].op {
                    Op::ExtractElement { vector: Value::Local(x), index }
                        if x == r && cx.const_int(index).is_some() =>
                    {
                        users.push(j)
                    }
                    _ => other = true,
                }
            }
        }
        if other || users.is_empty() {
            continue;
        }
        let it = cx.m.types.int(n);
        for j in users {
            let Op::ExtractElement { index, .. } = f.insts[j.index()].op else { unreachable!() };
            let k = cx.const_int(index).unwrap();
            let jr = f.insts[j.index()].result.unwrap();
            let blk = f.blocks.iter().position(|b| b.insts.contains(&j)).unwrap();
            let at = f.blocks[blk].insts.iter().position(|&x| x == j).unwrap();
            let mut s = Seq { f, new: Vec::new() };
            let bits = s.cast(CastOp::BitCast, value, it);
            let sh = cx.int(it, k * i64::from(w));
            let x = s.bin(BinOp::LShr, bits, sh, it);
            let v = s.cast(CastOp::Trunc, x, fe);
            let new = std::mem::take(&mut s.new);
            f.replace_uses(Value::Local(jr), v);
            f.blocks[blk].insts.splice(at..=at, new);
        }
        // The bitcast, now unused (and of a type no register holds).
        for blk in &mut f.blocks {
            blk.insts.retain(|&x| x != i);
        }
    }
}

/// A bit of a vector of integers taken as a mask's lane (`extractelement
/// (bitcast <4 x i32> %v to <128 x i1>), 32`, as chrono tests a bit): the
/// lane holding it, shifted down, its lowest bit. A bitcast no longer used
/// is removed.
fn bits_of_vectors(cx: &mut Ctx, f: &mut Function) {
    let i1 = cx.m.types.int(1);
    let i64t = cx.m.types.int(64);
    let mut changed = false;
    for b in f.layout.clone() {
        let mut k = 0;
        while k < f.blocks[b.index()].insts.len() {
            let i = f.blocks[b.index()].insts[k];
            let Op::ExtractElement { vector: Value::Local(m), index } = f.insts[i.index()].op else {
                k += 1;
                continue;
            };
            let Some(bit) = cx.const_int(index) else {
                k += 1;
                continue;
            };
            let LocalDef::Inst(d) = f.locals[m.index()].def else {
                k += 1;
                continue;
            };
            let Op::Cast { op: CastOp::BitCast, value, .. } = f.insts[d.index()].op else {
                k += 1;
                continue;
            };
            let (mt, vt) = (f.locals[m.index()].ty, cx.value_ty(f, value));
            let mask = matches!(cx.m.types.get(mt), Type::Vector { elem, .. } if *elem == i1);
            let Type::Vector { elem, .. } = *cx.m.types.get(vt) else {
                k += 1;
                continue;
            };
            let Type::Int(eb) = *cx.m.types.get(elem) else {
                k += 1;
                continue;
            };
            if !mask || eb == 1 {
                k += 1;
                continue;
            }
            let r = f.insts[i.index()].result.unwrap();
            let mut s = Seq { f, new: Vec::new() };
            let lane = cx.int(i64t, bit / i64::from(eb));
            let x = s.op(Op::ExtractElement { vector: value, index: lane }, elem);
            let sh = cx.int(elem, bit % i64::from(eb));
            let y = s.bin(BinOp::LShr, x, sh, elem);
            let z = s.cast(CastOp::Trunc, y, i1);
            let new = std::mem::take(&mut s.new);
            f.replace_uses(Value::Local(r), z);
            let n = new.len();
            f.blocks[b.index()].insts.splice(k..=k, new);
            k += n;
            changed = true;
        }
    }
    if !changed {
        return;
    }
    // Bitcasts to masks left without uses.
    let mut used = vec![false; f.locals.len()];
    for i in f.insts_in_order() {
        f.insts[i.index()].op.for_each_value(|v| {
            if let Value::Local(x) = v {
                used[x.index()] = true;
            }
        });
    }
    for b in f.layout.clone() {
        f.blocks[b.index()].insts.retain(|&i| {
            let inst = &f.insts[i.index()];
            !(matches!(inst.op, Op::Cast { op: CastOp::BitCast, .. })
                && inst.result.is_some_and(|r| {
                    !used[r.index()]
                        && matches!(cx.m.types.get(f.locals[r.index()].ty), Type::Vector { elem, .. } if *elem == i1)
                }))
        });
    }
}

/// A bitcast back to the type a bitcast came from is the value it started
/// as (the steps above make such pairs: `<1 x i64>` through `<8 x i8>` to
/// `i64` and back).
fn bitcast_round_trips(cx: &mut Ctx, f: &mut Function) {
    // Until none is left: folding one can make another.
    let mut folded: Vec<InstId> = Vec::new();
    loop {
        let mut changed = false;
        let order: Vec<InstId> = f.insts_in_order().collect();
        for i in order {
            let Op::Cast { op: CastOp::BitCast, value: Value::Local(x), .. } = f.insts[i.index()].op else { continue };
            let LocalDef::Inst(d) = f.locals[x.index()].def else { continue };
            let Op::Cast { op: CastOp::BitCast, value: orig, .. } = f.insts[d.index()].op else { continue };
            let r = f.insts[i.index()].result.unwrap();
            if cx.value_ty(f, orig) == f.locals[r.index()].ty && !folded.contains(&i) {
                f.replace_uses(Value::Local(r), orig);
                folded.push(i);
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }
    // Bitcasts no longer used, until none is (a chain goes link by link).
    loop {
        let mut used = vec![false; f.locals.len()];
        for i in f.insts_in_order() {
            f.insts[i.index()].op.for_each_value(|v| {
                if let Value::Local(x) = v {
                    used[x.index()] = true;
                }
            });
        }
        let dead = |f: &Function, i: InstId| {
            matches!(f.insts[i.index()].op, Op::Cast { op: CastOp::BitCast, .. })
                && f.insts[i.index()].result.is_some_and(|r| !used[r.index()])
        };
        let mut removed = false;
        for b in f.layout.clone() {
            let before = f.blocks[b.index()].insts.len();
            let keep: Vec<InstId> = f.blocks[b.index()].insts.iter().copied().filter(|&i| !dead(f, i)).collect();
            removed |= keep.len() != before;
            f.blocks[b.index()].insts = keep;
        }
        if !removed {
            break;
        }
    }
}

/// Lanes of masks (`<N x i1>`), taken out or put in, through integer
/// lanes: the mask sign-extended (all ones or zeros a lane), the lane
/// taken and truncated; or put in sign-extended, and the vector compared
/// with zero to be a mask again. The lanes are as wide as those of the
/// comparison that made the mask, where that is known (its register's:
/// after legalisation, an extension to another width would be taken as
/// free).
fn mask_lanes(cx: &mut Ctx, f: &mut Function) {
    let i1 = cx.m.types.int(1);
    let wide = |cx: &mut Ctx, f: &Function, ty: TypeId, mask: Option<Value>| -> Option<(TypeId, TypeId)> {
        let Type::Vector { len, elem, scalable: false } = *cx.m.types.get(ty) else { return None };
        if elem != i1 {
            return None;
        }
        // (Not narrower than a d register: a comparison of narrower lanes
        // was done lane by lane, and made no register of them.)
        let w = mask
            .and_then(|m| crate::legalize::mask_width(cx, f, m, &mut Vec::new()))
            .filter(|w| w * len >= 64)
            .unwrap_or((128 / len).clamp(8, 64));
        let e = cx.m.types.int(w);
        Some((cx.m.types.intern(Type::Vector { len, elem: e, scalable: false }), e))
    };
    let blocks: Vec<BlockId> = f.layout.clone();
    for b in blocks {
        let mut k = 0;
        while k < f.blocks[b.index()].insts.len() {
            let i = f.blocks[b.index()].insts[k];
            let Some(r) = f.insts[i.index()].result else {
                k += 1;
                continue;
            };
            let op = f.insts[i.index()].op.clone();
            let (mty, mask) = match &op {
                Op::ExtractElement { vector, .. } => (cx.value_ty(f, *vector), *vector),
                Op::InsertElement { vector, .. } => (f.locals[r.index()].ty, *vector),
                _ => {
                    k += 1;
                    continue;
                }
            };
            let Some((wt, e)) = wide(cx, f, mty, Some(mask)) else {
                k += 1;
                continue;
            };
            let mut s = Seq { f, new: Vec::new() };
            let v = match op {
                Op::ExtractElement { vector, index } => {
                    let w = s.cast(CastOp::SExt, vector, wt);
                    let x = s.op(Op::ExtractElement { vector: w, index }, e);
                    s.cast(CastOp::Trunc, x, i1)
                }
                Op::InsertElement { vector, element, index } => {
                    let w = s.cast(CastOp::SExt, vector, wt);
                    let x = s.cast(CastOp::SExt, element, e);
                    let w = s.op(Op::InsertElement { vector: w, element: x, index }, wt);
                    let zero = Value::Const(cx.m.consts.intern(Constant { ty: wt, kind: ConstKind::Zero }));
                    s.op(Op::ICmp { pred: ICmpPred::Ne, samesign: false, lhs: w, rhs: zero }, mty)
                }
                _ => unreachable!(),
            };
            s.f.replace_uses(Value::Local(r), v);
            let new = std::mem::take(&mut s.new);
            let n = new.len();
            f.blocks[b.index()].insts.splice(k..=k, new);
            k += n;
        }
    }
}

/// A bitcast of bits to a vector of booleans (`bitcast <8 x i8> to <64 x
/// i1>`, a set of flags), whose lanes are only taken out: each lane is a
/// bit of the bits as an integer, shifted down and truncated.
fn bit_masks(cx: &mut Ctx, f: &mut Function) {
    let i1 = cx.m.types.int(1);
    let order: Vec<InstId> = f.insts_in_order().collect();
    for i in order {
        let Op::Cast { op: CastOp::BitCast, value, .. } = f.insts[i.index()].op else { continue };
        let Some(r) = f.insts[i.index()].result else { continue };
        let Type::Vector { len, elem, scalable: false } = *cx.m.types.get(f.locals[r.index()].ty) else { continue };
        if elem != i1
            || len > 64
            || matches!(cx.m.types.get(cx.value_ty(f, value)), Type::Vector { elem, .. } if *elem == i1)
        {
            continue;
        }
        // Only extractelements of it.
        let mut users = Vec::new();
        let mut other = false;
        for j in f.insts_in_order() {
            let mut uses = false;
            f.insts[j.index()].op.for_each_value(|v| uses |= *v == Value::Local(r));
            if uses {
                match f.insts[j.index()].op {
                    Op::ExtractElement { vector: Value::Local(x), .. } if x == r => users.push(j),
                    _ => other = true,
                }
            }
        }
        if other || users.is_empty() {
            continue;
        }
        let it = cx.m.types.int(len);
        for j in users {
            let Op::ExtractElement { index, .. } = f.insts[j.index()].op else { unreachable!() };
            let jr = f.insts[j.index()].result.unwrap();
            let blk = f.blocks.iter().position(|b| b.insts.contains(&j)).unwrap();
            let k = f.blocks[blk].insts.iter().position(|&x| x == j).unwrap();
            let mut s = Seq { f, new: Vec::new() };
            let bits = s.cast(CastOp::BitCast, value, it);
            let ity = cx.value_ty(s.f, index);
            let idx = match cx.m.types.get(ity) {
                Type::Int(b) if *b > len => s.cast(CastOp::Trunc, index, it),
                Type::Int(b) if *b < len => s.cast(CastOp::ZExt, index, it),
                _ => index,
            };
            let sh = s.bin(BinOp::LShr, bits, idx, it);
            let v = s.cast(CastOp::Trunc, sh, i1);
            let new = std::mem::take(&mut s.new);
            f.replace_uses(Value::Local(jr), v);
            f.blocks[blk].insts.splice(k..=k, new);
        }
    }
}

/// A lane chosen by a value, not a constant (`extractelement <4 x i32> %v,
/// i64 %k`), through memory: the vector stored to a stack slot, and the
/// lane loaded from it (or stored into it, and the vector loaded back).
/// The index is taken modulo the lanes, a power of two, so that one past
/// the end (poison in LLVM) stays in the slot. A mask's lanes go through
/// it as bytes.
fn variable_lanes(cx: &mut Ctx, f: &mut Function) {
    let (i8t, i64t, ptr) = (cx.m.types.int(8), cx.m.types.int(64), cx.m.types.ptr(0));
    let Some(&entry) = f.layout.first() else { return };
    let mut slots = Vec::new();
    let blocks: Vec<BlockId> = f.layout.clone();
    for b in blocks {
        let mut k = 0;
        while k < f.blocks[b.index()].insts.len() {
            let i = f.blocks[b.index()].insts[k];
            let (vector, index, element) = match f.insts[i.index()].op {
                Op::ExtractElement { vector, index } if cx.const_int(index).is_none() => (vector, index, None),
                Op::InsertElement { vector, element, index } if cx.const_int(index).is_none() => {
                    (vector, index, Some(element))
                }
                _ => {
                    k += 1;
                    continue;
                }
            };
            let vty = cx.value_ty(f, vector);
            let Type::Vector { len, elem, scalable: false } = *cx.m.types.get(vty) else {
                k += 1;
                continue;
            };
            if !len.is_power_of_two() {
                k += 1;
                continue;
            }
            let r = f.insts[i.index()].result.unwrap();
            let mut s = Seq { f, new: Vec::new() };
            // A mask's lanes, whose bits memory packs, as bytes (each all
            // ones or zeros): the byte taken, and truncated; or put in, and
            // the bytes compared with zero to be a mask again.
            let mask_ty = matches!(cx.m.types.get(elem), Type::Int(1)).then_some(vty);
            let (vector, element, vty, elem) = match mask_ty {
                Some(_) => {
                    let bty = cx.m.types.intern(Type::Vector { len, elem: i8t, scalable: false });
                    let v = s.cast(CastOp::SExt, vector, bty);
                    let e = element.map(|e| s.cast(CastOp::SExt, e, i8t));
                    (v, e, bty, i8t)
                }
                None => (vector, element, vty, elem),
            };
            let esize = cx.size_of(elem).unwrap_or(1) as i64;
            let align = Some(cx.size_of(vty).unwrap_or(16).next_power_of_two().clamp(1, 16));
            let slot = s.f.add_inst(
                Op::Alloca { inalloca: false, ty: vty, count: None, align, addrspace: None },
                Some((None, ptr)),
            );
            slots.push(slot);
            let at = s.f.result(slot);
            s.op_void(Op::Store { volatile: false, atomic: None, value: vector, ptr: at, align });
            let idx = if cx.value_ty(s.f, index) == i64t { index } else { s.cast(CastOp::ZExt, index, i64t) };
            let mask = cx.int(i64t, i64::from(len) - 1);
            let idx = s.bin(BinOp::And, idx, mask, i64t);
            let off = if esize & (esize - 1) == 0 {
                let sh = cx.int(i64t, i64::from(esize.trailing_zeros()));
                s.bin(BinOp::Shl, idx, sh, i64t)
            } else {
                let sz = cx.int(i64t, esize);
                s.bin(BinOp::Mul, idx, sz, i64t)
            };
            let lane = s.op(
                Op::Gep { flags: GepFlags::default(), inrange: None, source_ty: i8t, ptr: at, indices: vec![off] },
                ptr,
            );
            let ealign = Some(esize as u64);
            let v = match element {
                None => s.op(Op::Load { volatile: false, atomic: None, ptr: lane, align: ealign }, elem),
                Some(e) => {
                    s.op_void(Op::Store { volatile: false, atomic: None, value: e, ptr: lane, align: ealign });
                    s.op(Op::Load { volatile: false, atomic: None, ptr: at, align }, vty)
                }
            };
            let v = match (mask_ty, element) {
                (Some(_), None) => {
                    let i1 = cx.m.types.int(1);
                    s.cast(CastOp::Trunc, v, i1)
                }
                (Some(mty), Some(_)) => {
                    let zero = Value::Const(cx.m.consts.intern(Constant { ty: vty, kind: ConstKind::Zero }));
                    s.op(Op::ICmp { pred: ICmpPred::Ne, samesign: false, lhs: v, rhs: zero }, mty)
                }
                _ => v,
            };
            s.f.replace_uses(Value::Local(r), v);
            let new = std::mem::take(&mut s.new);
            let n = new.len();
            f.blocks[b.index()].insts.splice(k..=k, new);
            k += n;
        }
    }
    f.blocks[entry.index()].insts.splice(0..0, slots);
}

/// The element of a one-lane vector `v`, as type `e`: a constant's, or a
/// bitcast.
fn one_lane_elem(cx: &mut Ctx, s: &mut Seq, v: Value, e: TypeId) -> Value {
    match v {
        Value::Const(c) => {
            let kind = match cx.m.constant(c).kind.clone() {
                ConstKind::Vector(es) => cx.m.constant(es[0]).kind.clone(),
                ConstKind::Splat(x) => cx.m.constant(x).kind.clone(),
                k => k,
            };
            Value::Const(cx.m.consts.intern(Constant { ty: e, kind }))
        }
        v => s.cast(CastOp::BitCast, v, e),
    }
}

/// Vectors of one lane (`<1 x i64>`, as NEON's `vget_lane_u64` has them)
/// stay vectors where the convention passes them (in a d register), but
/// what is done with them is done to their element: taking out or putting
/// in the lane is a bitcast, and arithmetic is the element's, between
/// bitcasts.
fn one_lane_vectors(cx: &mut Ctx, f: &mut Function) {
    let one = |cx: &Ctx, ty: TypeId| match cx.m.types.get(ty) {
        Type::Vector { len: 1, elem, scalable: false } => Some(*elem),
        _ => None,
    };
    let blocks: Vec<BlockId> = f.layout.clone();
    for b in blocks {
        let mut k = 0;
        while k < f.blocks[b.index()].insts.len() {
            let i = f.blocks[b.index()].insts[k];
            // A store of one: of its element.
            if let Op::Store { volatile, atomic: None, value, ptr, align } = f.insts[i.index()].op.clone()
                && let Some(e) = one(cx, cx.value_ty(f, value))
            {
                let mut s = Seq { f, new: Vec::new() };
                let x = one_lane_elem(cx, &mut s, value, e);
                s.op_void(Op::Store { volatile, atomic: None, value: x, ptr, align });
                let new = std::mem::take(&mut s.new);
                let n = new.len();
                f.blocks[b.index()].insts.splice(k..=k, new);
                k += n;
                continue;
            }
            let Some(r) = f.insts[i.index()].result else {
                k += 1;
                continue;
            };
            let ty = f.locals[r.index()].ty;
            let mut s = Seq { f, new: Vec::new() };
            let v = match s.f.insts[i.index()].op.clone() {
                Op::Load { volatile, atomic: None, ptr, align } if let Some(e) = one(cx, ty) => {
                    let x = s.op(Op::Load { volatile, atomic: None, ptr, align }, e);
                    s.cast(CastOp::BitCast, x, ty)
                }
                Op::ExtractElement { vector, .. } if one(cx, cx.value_ty(s.f, vector)).is_some() => {
                    one_lane_elem(cx, &mut s, vector, ty)
                }
                Op::InsertElement { element, .. } if one(cx, ty).is_some() => s.cast(CastOp::BitCast, element, ty),
                // A shuffle of one-lane vectors into a wider one (a splat):
                // the lanes (a's as 0, b's as 1) put into one of the
                // result's width, shuffled by the same mask.
                Op::ShuffleVector { a, b: bv, mask }
                    if let Some(e) = one(cx, cx.value_ty(s.f, a))
                        && matches!(cx.m.types.get(ty), Type::Vector { len, .. } if *len >= 2) =>
                {
                    let i64t = cx.m.types.int(64);
                    // A one-lane value's lane: a constant's, or a bitcast.
                    let lane = |cx: &mut Ctx, s: &mut Seq, v: Value| match v {
                        Value::Const(c) => {
                            let kind = match cx.m.constant(c).kind.clone() {
                                ConstKind::Vector(es) => cx.m.constant(es[0]).kind.clone(),
                                ConstKind::Splat(e1) => cx.m.constant(e1).kind.clone(),
                                k => k,
                            };
                            Value::Const(cx.m.consts.intern(Constant { ty: e, kind }))
                        }
                        v => s.cast(CastOp::BitCast, v, e),
                    };
                    let x = lane(cx, &mut s, a);
                    let poison = Value::Const(cx.m.consts.intern(Constant { ty, kind: ConstKind::Poison }));
                    let (z, o) = (cx.int(i64t, 0), cx.int(i64t, 1));
                    let mut w = s.op(Op::InsertElement { vector: poison, element: x, index: z }, ty);
                    if crate::target::mask_lanes(cx.m, mask).is_some_and(|m| m.contains(&1)) {
                        let y = lane(cx, &mut s, bv);
                        w = s.op(Op::InsertElement { vector: w, element: y, index: o }, ty);
                    }
                    s.op(Op::ShuffleVector { a: w, b: poison, mask }, ty)
                }
                Op::FNeg { fmf, value } if let Some(e) = one(cx, ty) => {
                    let a = one_lane_elem(cx, &mut s, value, e);
                    let x = s.op(Op::FNeg { fmf, value: a }, e);
                    s.cast(CastOp::BitCast, x, ty)
                }
                Op::Binary { op, flags, fmf, lhs, rhs } if let Some(e) = one(cx, ty) => {
                    let (a, c) = (one_lane_elem(cx, &mut s, lhs, e), one_lane_elem(cx, &mut s, rhs, e));
                    let x = s.op(Op::Binary { op, flags, fmf, lhs: a, rhs: c }, e);
                    s.cast(CastOp::BitCast, x, ty)
                }
                _ => {
                    k += 1;
                    continue;
                }
            };
            s.f.replace_uses(Value::Local(r), v);
            let new = std::mem::take(&mut s.new);
            let n = new.len();
            f.blocks[b.index()].insts.splice(k..=k, new);
            k += n;
        }
    }
}

/// Whether constant `c` has arithmetic in it (`add (i64 ptrtoint (ptr @g
/// to i64), i64 1)`), which selection does not take as a constant; an
/// address (a global, plus offsets, cast to a pointer-wide integer) it
/// does. (An address made narrower, or a float, `bitcast (i32 ptrtoint
/// (ptr @g to i32) to float)`, is arithmetic.)
fn has_arithmetic(cx: &Ctx, c: ConstId) -> bool {
    let k = cx.m.constant(c);
    match &k.kind {
        ConstKind::Expr(ConstExpr::Binary { .. }) => true,
        ConstKind::Expr(ConstExpr::Cast { value, .. }) => {
            has_arithmetic(cx, *value)
                || (!matches!(cx.m.types.get(k.ty), Type::Ptr(_) | Type::Int(64))
                    && const_address(cx, *value).is_some())
        }
        ConstKind::Expr(ConstExpr::Gep { ptr, indices, .. }) => {
            has_arithmetic(cx, *ptr) || indices.iter().any(|&i| has_arithmetic(cx, i))
        }
        _ => false,
    }
}

/// Constant `c` computed by instructions in `s` (its arithmetic; what is
/// left, addresses and numbers, stays constant).
fn materialize(cx: &mut Ctx, s: &mut Seq, c: ConstId) -> Value {
    if !has_arithmetic(cx, c) {
        return Value::Const(c);
    }
    let k = cx.m.constant(c).clone();
    match k.kind {
        ConstKind::Expr(ConstExpr::Binary { op, flags, lhs, rhs }) => {
            let (l, r) = (materialize(cx, s, lhs), materialize(cx, s, rhs));
            s.op(Op::Binary { op, flags, fmf: FastMath::default(), lhs: l, rhs: r }, k.ty)
        }
        ConstKind::Expr(ConstExpr::Cast { op, value }) => {
            let v = materialize(cx, s, value);
            s.cast(op, v, k.ty)
        }
        ConstKind::Expr(ConstExpr::Gep { flags, inrange, source_ty, ptr, indices }) => {
            let p = materialize(cx, s, ptr);
            let idx: Vec<Value> = indices.iter().map(|&i| materialize(cx, s, i)).collect();
            s.op(Op::Gep { flags, inrange, source_ty, ptr: p, indices: idx }, k.ty)
        }
        _ => Value::Const(c),
    }
}

/// Constant expressions with arithmetic, as instructions just before the
/// instruction using them (for a phi, at the end of the block the value
/// comes from).
fn const_exprs(cx: &mut Ctx, f: &mut Function) {
    let blocks: Vec<BlockId> = f.layout.clone();
    for b in blocks {
        let mut k = 0;
        while k < f.blocks[b.index()].insts.len() {
            let i = f.blocks[b.index()].insts[k];
            let mut needs = false;
            f.insts[i.index()].op.for_each_value(|v| {
                needs |= matches!(v, Value::Const(c) if has_arithmetic(cx, *c));
            });
            if !needs {
                k += 1;
                continue;
            }
            if let Op::Phi { fmf, incoming } = f.insts[i.index()].op.clone() {
                let mut inc = Vec::new();
                for (v, pred) in incoming {
                    let v = match v {
                        Value::Const(c) if has_arithmetic(cx, c) => {
                            let mut s = Seq { f, new: Vec::new() };
                            let x = materialize(cx, &mut s, c);
                            let new = std::mem::take(&mut s.new);
                            let n = f.blocks[pred.index()].insts.len();
                            f.blocks[pred.index()].insts.splice(n - 1..n - 1, new);
                            x
                        }
                        v => v,
                    };
                    inc.push((v, pred));
                }
                f.insts[i.index()].op = Op::Phi { fmf, incoming: inc };
                k += 1;
                continue;
            }
            let mut op = f.insts[i.index()].op.clone();
            let mut s = Seq { f, new: Vec::new() };
            let mut done: Vec<(ConstId, Value)> = Vec::new();
            op.for_each_value_mut(|v| {
                if let Value::Const(c) = *v
                    && has_arithmetic(cx, c)
                {
                    let x = match done.iter().find(|(d, _)| *d == c) {
                        Some(&(_, x)) => x,
                        None => {
                            let x = materialize(cx, &mut s, c);
                            done.push((c, x));
                            x
                        }
                    };
                    *v = x;
                }
            });
            s.f.insts[i.index()].op = op;
            let new = std::mem::take(&mut s.new);
            let n = new.len();
            f.blocks[b.index()].insts.splice(k..k, new);
            k += n + 1;
        }
    }
}

/// Multiplications by constants that a shift does (a power of two), or an
/// addition of a shifted operand and a shift (`(2^a + 1) << b`: 24 is
/// `(x + (x << 1)) << 3`), which machines with shifted operands make one
/// or two instructions without the constant's register (and the shift
/// often goes into the address it is added to).
fn const_multiplies(cx: &mut Ctx, f: &mut Function) {
    for b in f.layout.clone() {
        let mut k = 0;
        while k < f.blocks[b.index()].insts.len() {
            let i = f.blocks[b.index()].insts[k];
            k += 1;
            let Op::Binary { op: BinOp::Mul, lhs, rhs, .. } = f.insts[i.index()].op else { continue };
            let Some(r) = f.insts[i.index()].result else { continue };
            let ty = f.locals[r.index()].ty;
            let Type::Int(w) = *cx.m.types.get(ty) else { continue };
            let (x, c) = match (cx.const_int(lhs), cx.const_int(rhs)) {
                (None, Some(c)) => (lhs, c),
                (Some(c), None) => (rhs, c),
                _ => continue,
            };
            if w > 64 || c <= 1 || (w < 64 && c >= 1 << (w - 1)) {
                continue;
            }
            let c = c as u64;
            let sh = c.trailing_zeros();
            let odd = c >> sh;
            if odd != 1 && !(odd - 1).is_power_of_two() {
                continue;
            }
            let mut s = Seq { f, new: Vec::new() };
            let mut v = x;
            if odd != 1 {
                let a = cx.int(ty, i64::from((odd - 1).trailing_zeros()));
                let up = s.bin(BinOp::Shl, x, a, ty);
                v = s.bin(BinOp::Add, x, up, ty);
            }
            if sh > 0 {
                let n = cx.int(ty, i64::from(sh));
                v = s.bin(BinOp::Shl, v, n, ty);
            }
            let new = std::mem::take(&mut s.new);
            let n = new.len();
            f.blocks[b.index()].insts.splice(k - 1..k - 1, new);
            k += n;
            f.replace_uses(Value::Local(r), v);
        }
    }
}

/// Integer operations with an operand that changes nothing (`or %x, 0`,
/// `xor 0, %x`, `shl %x, 0`, `and %x, -1`, `mul %x, 1`), as the halves of
/// wide integers with a constant half make many of: the other operand. And
/// `and %x, 0`, `mul %x, 0`: zero.
fn identity_ops(cx: &mut Ctx, f: &mut Function) {
    // What each value replaced so far is, so that chains of them go (an
    // `add` of a `mul` by zero).
    let mut changed: std::collections::HashMap<LocalId, Value> = std::collections::HashMap::new();
    let resolve = |changed: &std::collections::HashMap<LocalId, Value>, mut v: Value| {
        while let Value::Local(l) = v
            && let Some(&w) = changed.get(&l)
        {
            v = w;
        }
        v
    };
    for i in f.insts_in_order() {
        let Op::Binary { op, lhs, rhs, .. } = f.insts[i.index()].op else { continue };
        let Some(r) = f.insts[i.index()].result else { continue };
        if !matches!(cx.m.types.get(f.locals[r.index()].ty), Type::Int(_)) {
            continue;
        }
        let (lhs, rhs) = (resolve(&changed, lhs), resolve(&changed, rhs));
        let (l, rr) = (cx.const_int(lhs), cx.const_int(rhs));
        let v = match (op, l, rr) {
            (BinOp::Or | BinOp::Xor | BinOp::Add, Some(0), _) => Some(rhs),
            (BinOp::Or | BinOp::Xor | BinOp::Add | BinOp::Sub | BinOp::Shl | BinOp::LShr | BinOp::AShr, _, Some(0)) => {
                Some(lhs)
            }
            (BinOp::And, Some(-1), _) | (BinOp::Mul, Some(1), _) => Some(rhs),
            (BinOp::And, _, Some(-1)) | (BinOp::Mul, _, Some(1)) => Some(lhs),
            (BinOp::And | BinOp::Mul, Some(0), _) => Some(lhs),
            (BinOp::And | BinOp::Mul, _, Some(0)) => Some(rhs),
            _ => None,
        };
        if let Some(v) = v {
            changed.insert(r, v);
        }
    }
    f.replace_many(&changed);
}

/// A boolean extended and compared with what it was extended from
/// (`icmp eq (zext i1 %c to i32), 0`, as Rust makes for `!c`) is the
/// boolean, or the boolean inverted: the inverse comparison where it is a
/// comparison (so that a branch on it is one compare), else an `xor` with
/// true.
fn extended_bool_compares(cx: &mut Ctx, f: &mut Function) {
    let i1 = cx.m.types.int(1);
    for b in f.layout.clone() {
        let mut k = 0;
        while k < f.blocks[b.index()].insts.len() {
            let i = f.blocks[b.index()].insts[k];
            k += 1;
            let Op::ICmp { pred: pred @ (ICmpPred::Eq | ICmpPred::Ne), lhs, rhs, .. } = f.insts[i.index()].op else {
                continue;
            };
            let Some(r) = f.insts[i.index()].result else { continue };
            let ext = |v: Value| match v {
                Value::Local(l) => match f.locals[l.index()].def {
                    LocalDef::Inst(d) => match f.insts[d.index()].op {
                        Op::Cast { op: op @ (CastOp::ZExt | CastOp::SExt), value, .. }
                            if cx.value_ty(f, value) == i1 =>
                        {
                            Some((value, if op == CastOp::ZExt { 1 } else { -1 }))
                        }
                        _ => None,
                    },
                    _ => None,
                },
                _ => None,
            };
            let ((bv, one), c) = match (ext(lhs), ext(rhs)) {
                (Some(e), None) => (e, cx.const_int(rhs)),
                (None, Some(e)) => (e, cx.const_int(lhs)),
                _ => continue,
            };
            let Some(c) = c else { continue };
            // Whether the result is the boolean itself (else its inverse).
            let same = match (pred, c) {
                (ICmpPred::Ne, 0) => true,
                (ICmpPred::Eq, 0) => false,
                (ICmpPred::Eq, x) if x == one => true,
                (ICmpPred::Ne, x) if x == one => false,
                _ => continue,
            };
            let v = if same {
                bv
            } else {
                let def = match bv {
                    Value::Local(l) => match f.locals[l.index()].def {
                        LocalDef::Inst(d) => Some(f.insts[d.index()].op.clone()),
                        _ => None,
                    },
                    _ => None,
                };
                let op = match def {
                    Some(Op::ICmp { pred, samesign, lhs, rhs }) => {
                        Op::ICmp { pred: crate::target::inverse(pred), samesign, lhs, rhs }
                    }
                    Some(Op::FCmp { pred, fmf, lhs, rhs }) if crate::target::inverse_f(pred).is_some() => {
                        Op::FCmp { pred: crate::target::inverse_f(pred).unwrap(), fmf, lhs, rhs }
                    }
                    _ => {
                        let t = cx.int(i1, 1);
                        Op::Binary { op: BinOp::Xor, flags: IntFlags::NONE, fmf: FastMath::default(), lhs: bv, rhs: t }
                    }
                };
                let n = f.add_inst(op, Some((None, i1)));
                f.blocks[b.index()].insts.insert(k - 1, n);
                k += 1;
                f.result(n)
            };
            f.replace_uses(Value::Local(r), v);
        }
    }
}

/// A conditional branch on a constant (`llvm.is.constant`'s answer, at
/// `-O0`) is a branch to the side it picks; the other side's phis lose
/// this block.
fn constant_branches(cx: &Ctx, f: &mut Function) {
    for b in f.layout.clone() {
        let Some(&t) = f.blocks[b.index()].insts.last() else { continue };
        let Op::CondBr { cond, then_dest, else_dest } = f.insts[t.index()].op else { continue };
        let Some(c) = cx.const_int(cond) else { continue };
        let (to, other) = if c & 1 != 0 { (then_dest, else_dest) } else { (else_dest, then_dest) };
        f.insts[t.index()].op = Op::Br(to);
        if other == to {
            continue;
        }
        for &i in &f.blocks[other.index()].insts {
            if let Op::Phi { incoming, .. } = &mut f.insts[i.index()].op {
                incoming.retain(|&(_, from)| from != b);
            }
        }
    }
}

/// A bitcast to its operand's own type is the operand.
fn identity_bitcasts(f: &mut Function) {
    for i in f.insts_in_order().collect::<Vec<_>>() {
        if let Op::Cast { op: CastOp::BitCast, value, .. } = f.insts[i.index()].op
            && let Some(r) = f.insts[i.index()].result
        {
            let from = match value {
                Value::Local(x) => Some(f.locals[x.index()].ty),
                _ => None,
            };
            if from == Some(f.locals[r.index()].ty) {
                f.replace_uses(Value::Local(r), value);
            }
        }
    }
}

/// The mark of an argument, parameter or result of `half` turned into the
/// `i16` of its bits (`half_as_int`): passed as a half is, in a
/// floating-point register.
pub const HALF: &str = "epitome.half";

/// Values of `half` (and vectors of them) as `i16`s of their bits: moves
/// of bits (loads, stores, bitcasts, lanes, phis and selects) as they are,
/// `fneg` an `xor` of the sign bit, and arithmetic, comparisons,
/// conversions and intrinsics in `float` (`half_arith`); parameters,
/// results and calls' arguments marked `HALF`. (There is no `half`
/// register class; vectors of `half` as parameters are an error.)
fn half_as_int(cx: &mut Ctx, f: &mut Function) -> Result<(), String> {
    let half = cx.m.types.intern(Type::Half);
    let i16t = cx.m.types.int(16);
    let as_int = |cx: &mut Ctx, ty: TypeId| -> Option<TypeId> {
        match *cx.m.types.get(ty) {
            Type::Half => Some(i16t),
            Type::Vector { len, elem, scalable } if elem == half => {
                Some(cx.m.types.intern(Type::Vector { len, elem: i16t, scalable }))
            }
            _ => None,
        }
    };
    let has_half = |cx: &mut Ctx, ty: TypeId| as_int(cx, ty).is_some();
    if !f.locals.iter().map(|l| l.ty).collect::<Vec<_>>().into_iter().any(|t| has_half(cx, t)) {
        return Ok(());
    }
    // C's sqrtf16 (which clang calls where errno may be set): the C
    // libraries have none; it is llvm.sqrt.f16, but for errno.
    for i in f.insts_in_order().collect::<Vec<_>>() {
        if let Op::Call(c) = &f.insts[i.index()].op
            && callee_name(cx.m, c).as_deref() == Some("sqrtf16")
        {
            let callee = crate::legalize::declare(cx, "llvm.sqrt.f16", half, &[half]);
            if let Op::Call(c) = &mut f.insts[i.index()].op {
                c.callee = callee;
            }
        }
    }
    // Parameters and results of half, and calls' arguments and results:
    // the i16s of their bits, marked as passed where halves are (`HALF`).
    let scalar_half = |cx: &Ctx, t: TypeId| matches!(cx.m.types.get(t), Type::Half);
    // Arithmetic and passing halves need the machine's conversions and
    // moves of their bits (`move.w.h`, `llvm.convert.*.fp16`).
    let converts = cx.t.role("move.w.h").is_some();
    let ret_half = matches!(cx.m.types.get(f.ty), Type::Function { ret, .. } if scalar_half(cx, *ret));
    if !converts && (ret_half || f.params.iter().any(|p| has_half(cx, f.locals[p.local.index()].ty))) {
        return Err("parameters and results of half are not supported on this machine yet".into());
    }
    for k in 0..f.params.len() {
        let t = f.locals[f.params[k].local.index()].ty;
        if scalar_half(cx, t) {
            f.params[k].attrs.push(Attr::Word(HALF.into()));
        } else if has_half(cx, t) {
            return Err("parameters of vectors of half are not supported yet".into());
        }
    }
    if let Type::Function { ret, params, varargs } = cx.m.types.get(f.ty).clone() {
        if scalar_half(cx, ret) {
            f.ret_attrs.push(Attr::Word(HALF.into()));
        }
        let params: Vec<TypeId> = params.iter().map(|&t| as_int(cx, t).unwrap_or(t)).collect();
        let ret = as_int(cx, ret).unwrap_or(ret);
        f.ty = cx.m.types.intern(Type::Function { ret, params, varargs });
    }
    // Only operations on bits.
    for i in f.insts_in_order() {
        let op = &f.insts[i.index()].op;
        let res_half = f.insts[i.index()].result.is_some_and(|r| has_half(cx, f.locals[r.index()].ty));
        let mut uses_half = false;
        op.for_each_value(|v| {
            if !matches!(v, Value::Metadata(_)) {
                uses_half |= has_half(cx, cx.value_ty(f, *v));
            }
        });
        if !(res_half || uses_half) {
            continue;
        }
        let bits_only = matches!(
            op,
            Op::Load { .. }
                | Op::Store { .. }
                | Op::Cast { op: CastOp::BitCast, .. }
                | Op::FNeg { .. }
                | Op::Phi { .. }
                | Op::Select { .. }
                | Op::Freeze(_)
                | Op::ExtractElement { .. }
                | Op::InsertElement { .. }
                | Op::ShuffleVector { .. }
        ) || matches!(op, Op::Ret(_) if converts);
        let call = match op {
            Op::Call(c) | Op::Invoke { call: c, .. } => {
                !callee_name(cx.m, c).is_some_and(|n| n.starts_with("llvm."))
                    && c.args.iter().all(|a| {
                        let t = cx.value_ty(f, a.value);
                        !has_half(cx, t) || scalar_half(cx, t)
                    })
                    && f.insts[i.index()].result.is_none_or(|r| {
                        let t = f.locals[r.index()].ty;
                        !has_half(cx, t) || scalar_half(cx, t)
                    })
            }
            _ => false,
        };
        let arith = call
            || match op {
                Op::Binary { op: BinOp::FAdd | BinOp::FSub | BinOp::FMul | BinOp::FDiv | BinOp::FRem, .. }
                | Op::FCmp { .. }
                | Op::Cast {
                    op:
                        CastOp::FPExt | CastOp::FPTrunc | CastOp::SIToFP | CastOp::UIToFP | CastOp::FPToSI | CastOp::FPToUI,
                    ..
                } => true,
                Op::Call(c) => callee_name(cx.m, c).is_some_and(|n| half_intrinsic(&n).is_some()),
                _ => false,
            };
        if !bits_only && !(arith && converts) {
            return Err(match op {
                Op::Call(_) | Op::Invoke { .. } => "calls with arguments or results of half are not supported yet",
                _ => "this operation on half is not supported yet",
            }
            .into());
        }
    }
    // Constants of half: integers of their bits.
    let int_const = |cx: &mut Ctx, c: ConstId| -> Option<ConstId> {
        let k = cx.m.constant(c).clone();
        let t = as_int(cx, k.ty)?;
        let lane = |k: &ConstKind| -> ConstKind {
            match k {
                ConstKind::Float(b) => ConstKind::Int(ApInt::from_u64(16, *b as u64)),
                other => other.clone(),
            }
        };
        let kind = match &k.kind {
            ConstKind::Vector(es) => {
                let es: Vec<ConstId> = es
                    .iter()
                    .map(|&e| {
                        let ek = cx.m.constant(e).kind.clone();
                        let kind = lane(&ek);
                        cx.m.consts.intern(Constant { ty: i16t, kind })
                    })
                    .collect();
                ConstKind::Vector(es)
            }
            ConstKind::Splat(e) => {
                let ek = cx.m.constant(*e).kind.clone();
                let kind = lane(&ek);
                ConstKind::Splat(cx.m.consts.intern(Constant { ty: i16t, kind }))
            }
            other => lane(other),
        };
        Some(cx.m.consts.intern(Constant { ty: t, kind }))
    };
    // Calls' halves marked, before their types change.
    for i in f.insts_in_order().collect::<Vec<_>>() {
        let res_half = f.insts[i.index()].result.is_some_and(|r| scalar_half(cx, f.locals[r.index()].ty));
        let marks: Vec<bool> = match &f.insts[i.index()].op {
            Op::Call(c) | Op::Invoke { call: c, .. }
                if !callee_name(cx.m, c).is_some_and(|n| n.starts_with("llvm.")) =>
            {
                c.args.iter().map(|a| scalar_half(cx, cx.value_ty(f, a.value))).collect()
            }
            _ => continue,
        };
        let Some(c) = f.insts[i.index()].op.call_mut() else { continue };
        for (a, m) in c.args.iter_mut().zip(marks) {
            if m {
                a.attrs.push(Attr::Word(HALF.into()));
            }
        }
        if res_half {
            c.ret_attrs.push(Attr::Word(HALF.into()));
        }
        if let Type::Function { ret, params, varargs } = cx.m.types.get(c.fn_ty).clone() {
            let params: Vec<TypeId> = params.iter().map(|&t| as_int(cx, t).unwrap_or(t)).collect();
            let ret = as_int(cx, ret).unwrap_or(ret);
            c.fn_ty = cx.m.types.intern(Type::Function { ret, params, varargs });
        }
    }
    for k in 0..f.locals.len() {
        if let Some(t) = as_int(cx, f.locals[k].ty) {
            f.locals[k].ty = t;
        }
    }
    for i in f.insts_in_order().collect::<Vec<_>>() {
        let mut op = f.insts[i.index()].op.clone();
        op.for_each_value_mut(|v| {
            if let Value::Const(c) = *v
                && let Some(n) = int_const(cx, c)
            {
                *v = Value::Const(n);
            }
        });
        // fneg: the sign bit flipped.
        if let Op::FNeg { value, .. } = op
            && let Some(r) = f.insts[i.index()].result
            && matches!(cx.m.types.get(f.locals[r.index()].ty), Type::Int(16))
                | matches!(cx.m.types.get(f.locals[r.index()].ty), Type::Vector { elem, .. } if *elem == i16t)
        {
            let ty = f.locals[r.index()].ty;
            let sign = cx.m.consts.intern(Constant { ty: i16t, kind: ConstKind::Int(ApInt::from_u64(16, 0x8000)) });
            let c = if ty == i16t { sign } else { cx.m.consts.intern(Constant { ty, kind: ConstKind::Splat(sign) }) };
            op = Op::Binary {
                op: BinOp::Xor,
                flags: IntFlags::NONE,
                fmf: FastMath::default(),
                lhs: value,
                rhs: Value::Const(c),
            };
        }
        // (A bitcast to or from half may now be between one type, which
        // `identity_bitcasts` takes away.)
        f.insts[i.index()].op = op;
    }
    half_arith(cx, f);
    half_helpers(cx, f);
    Ok(())
}

/// Conversions of half the machine has no instruction for (x86-64 without
/// F16C): calls of the C library's (libgcc's and compiler-rt's
/// `__extendhfsf2`, `__truncsfhf2`, `__truncdfhf2`), their halves passed
/// as halves are.
fn half_helpers(cx: &mut Ctx, f: &mut Function) {
    for i in f.insts_in_order().collect::<Vec<_>>() {
        let Op::Call(c) = &f.insts[i.index()].op else { continue };
        let Some(name) = callee_name(cx.m, c) else { continue };
        let (helper, arg_half) = match name.as_str() {
            "llvm.convert.from.fp16.f32" => ("__extendhfsf2", true),
            "llvm.convert.to.fp16.f32" => ("__truncsfhf2", false),
            "llvm.convert.to.fp16.f64" => ("__truncdfhf2", false),
            _ => continue,
        };
        if cx.selects_intrinsic(&name) {
            continue;
        }
        let Type::Function { ret, params, .. } = cx.m.types.get(c.fn_ty).clone() else { continue };
        let callee = crate::legalize::declare(cx, helper, ret, &params);
        let Op::Call(c) = &mut f.insts[i.index()].op else { unreachable!() };
        c.callee = callee;
        if arg_half {
            c.args[0].attrs.push(Attr::Word(HALF.into()));
        } else {
            c.ret_attrs.push(Attr::Word(HALF.into()));
        }
    }
}

/// What intrinsic `name` of `half` (`llvm.sqrt.f16`) is done as: its
/// `float` form, or for a fused one, its `double` form (whose product of
/// halves is exact, and whose one rounding before the half's is almost
/// always right; see later.md). `None` for others.
fn half_intrinsic(name: &str) -> Option<(String, bool)> {
    let rest = name.strip_prefix("llvm.")?;
    let (base, ty) = rest.rsplit_once('.')?;
    if !(ty == "f16" || ty.starts_with('v') && ty.ends_with("f16")) {
        return None;
    }
    let ok = matches!(
        base,
        "fabs"
            | "sqrt"
            | "minnum"
            | "maxnum"
            | "minimum"
            | "maximum"
            | "minimumnum"
            | "maximumnum"
            | "copysign"
            | "floor"
            | "ceil"
            | "trunc"
            | "rint"
            | "nearbyint"
            | "round"
            | "roundeven"
            | "canonicalize"
            | "sin"
            | "cos"
            | "tan"
            | "exp"
            | "exp2"
            | "exp10"
            | "log"
            | "log2"
            | "log10"
            | "pow"
            | "fma"
            | "fmuladd"
    );
    ok.then(|| {
        let wide = matches!(base, "fma" | "fmuladd");
        (format!("llvm.{base}.{}", if wide { "f64" } else { "f32" }), wide)
    })
}

/// Arithmetic left on the `i16`s halves became (`half_as_int`), which only
/// halves can be the operands of: done in `float` (exactly rounded to half
/// for `+ - * /` and square roots, as float's 24 bits are more than twice
/// half's 11 and two), and the result narrowed back; vectors lane by lane.
/// Integers become halves through `double`, which holds every integer that
/// is not too big for a half exactly.
fn half_arith(cx: &mut Ctx, f: &mut Function) {
    let i16t = cx.m.types.int(16);
    let f32t = cx.m.types.intern(Type::Float);
    let f64t = cx.m.types.intern(Type::Double);
    let i64t = cx.m.types.int(64);
    let is_h =
        |cx: &Ctx, t: TypeId| t == i16t || matches!(cx.m.types.get(t), Type::Vector { elem, .. } if *elem == i16t);
    let h2f = |cx: &mut Ctx, s: &mut Seq, v: Value| call_intrinsic(cx, s, "llvm.convert.from.fp16.f32", vec![v], f32t);
    let blocks: Vec<BlockId> = f.layout.clone();
    for b in blocks {
        let mut k = 0;
        while k < f.blocks[b.index()].insts.len() {
            let i = f.blocks[b.index()].insts[k];
            let op = f.insts[i.index()].op.clone();
            let Some(r) = f.insts[i.index()].result else {
                k += 1;
                continue;
            };
            let rty = f.locals[r.index()].ty;
            let mut operand_tys = Vec::new();
            op.for_each_value(|v| {
                if !matches!(v, Value::Metadata(_)) {
                    operand_tys.push(cx.value_ty(f, *v));
                }
            });
            // Which of the operations on halves this is.
            let of_half = match &op {
                Op::Binary { op: BinOp::FAdd | BinOp::FSub | BinOp::FMul | BinOp::FDiv | BinOp::FRem, .. } => {
                    is_h(cx, rty)
                }
                Op::FCmp { lhs, .. } => is_h(cx, cx.value_ty(f, *lhs)),
                Op::Cast { op: CastOp::FPExt | CastOp::FPToSI | CastOp::FPToUI, value, .. } => {
                    is_h(cx, cx.value_ty(f, *value))
                }
                Op::Cast { op: CastOp::FPTrunc | CastOp::SIToFP | CastOp::UIToFP, .. } => is_h(cx, rty),
                Op::Call(c) => callee_name(cx.m, c).is_some_and(|n| half_intrinsic(&n).is_some()),
                _ => false,
            };
            if !of_half {
                k += 1;
                continue;
            }
            // Lanes, if a vector: each done as a scalar.
            let lanes = match cx.m.types.get(rty) {
                Type::Vector { len, .. } => Some(*len),
                _ => operand_tys.iter().find_map(|&t| match cx.m.types.get(t) {
                    Type::Vector { len, .. } => Some(*len),
                    _ => None,
                }),
            };
            let scalar = |cx: &Ctx, t: TypeId| match cx.m.types.get(t) {
                Type::Vector { elem, .. } => *elem,
                _ => t,
            };
            let mut s = Seq { f, new: Vec::new() };
            let one = |cx: &mut Ctx, s: &mut Seq, op: &Op, lane: Option<u32>| -> Value {
                let take = |cx: &mut Ctx, s: &mut Seq, v: Value| match lane {
                    Some(l) => {
                        let vt = cx.value_ty(s.f, v);
                        let e = scalar(cx, vt);
                        let idx = cx.int(i64t, l as i64);
                        s.op(Op::ExtractElement { vector: v, index: idx }, e)
                    }
                    None => v,
                };
                let rs = scalar(cx, rty);
                match op.clone() {
                    Op::Binary { op: o, fmf, lhs, rhs, .. } => {
                        let (a, b) = (take(cx, s, lhs), take(cx, s, rhs));
                        let (a, b) = (h2f(cx, s, a), h2f(cx, s, b));
                        let z = s.op(Op::Binary { op: o, flags: IntFlags::NONE, fmf, lhs: a, rhs: b }, f32t);
                        call_intrinsic(cx, s, "llvm.convert.to.fp16.f32", vec![z], i16t)
                    }
                    Op::FCmp { pred, fmf, lhs, rhs } => {
                        let (a, b) = (take(cx, s, lhs), take(cx, s, rhs));
                        let (a, b) = (h2f(cx, s, a), h2f(cx, s, b));
                        s.op(Op::FCmp { pred, fmf, lhs: a, rhs: b }, rs)
                    }
                    Op::Cast { op: c, value, .. } => {
                        let x = take(cx, s, value);
                        match c {
                            CastOp::FPExt => {
                                let w = h2f(cx, s, x);
                                if rs == f32t { w } else { s.cast(CastOp::FPExt, w, rs) }
                            }
                            CastOp::FPToSI | CastOp::FPToUI => {
                                let w = h2f(cx, s, x);
                                s.cast(c, w, rs)
                            }
                            CastOp::FPTrunc => {
                                let xt = cx.value_ty(s.f, x);
                                if xt == f64t {
                                    call_intrinsic(cx, s, "llvm.convert.to.fp16.f64", vec![x], i16t)
                                } else {
                                    call_intrinsic(cx, s, "llvm.convert.to.fp16.f32", vec![x], i16t)
                                }
                            }
                            _ => {
                                let d = s.cast(c, x, f64t);
                                call_intrinsic(cx, s, "llvm.convert.to.fp16.f64", vec![d], i16t)
                            }
                        }
                    }
                    Op::Call(call) => {
                        let name = callee_name(cx.m, &call).unwrap();
                        let (wide_name, wide) = half_intrinsic(&name).unwrap();
                        let ft = if wide { f64t } else { f32t };
                        let mut args = Vec::new();
                        for a in &call.args {
                            let x = take(cx, s, a.value);
                            let xt = cx.value_ty(s.f, x);
                            args.push(if xt == i16t {
                                let w = h2f(cx, s, x);
                                if wide { s.cast(CastOp::FPExt, w, f64t) } else { w }
                            } else {
                                x
                            });
                        }
                        let z = call_intrinsic(cx, s, &wide_name, args, ft);
                        let back = if wide { "llvm.convert.to.fp16.f64" } else { "llvm.convert.to.fp16.f32" };
                        call_intrinsic(cx, s, back, vec![z], i16t)
                    }
                    _ => unreachable!(),
                }
            };
            let v = match lanes {
                None => one(cx, &mut s, &op, None),
                Some(n) => {
                    let mut acc = Value::Const(cx.m.consts.intern(Constant { ty: rty, kind: ConstKind::Poison }));
                    for l in 0..n {
                        let x = one(cx, &mut s, &op, Some(l));
                        let idx = cx.int(i64t, l as i64);
                        acc = s.op(Op::InsertElement { vector: acc, element: x, index: idx }, rty);
                    }
                    acc
                }
            };
            let new = std::mem::take(&mut s.new);
            f.replace_uses(Value::Local(r), v);
            let n = new.len();
            f.blocks[b.index()].insts.splice(k..=k, new);
            k += n;
        }
    }
}

/// Bitcasts, in forms the code generator has: between registers where
/// both types are ones a register holds whole (integers and floating point
/// of up to 64 bits; vectors of 64 or 128 bits of lanes of a byte or more;
/// a mask and the integer of its bits), and a 64-bit vector and `i64`
/// through `<8 x i8>` (`fmov`). Any other (a vector narrower than a
/// register or wider than one, `i128` and a vector, vectors of one lane) is
/// through memory: stored as one type to a stack slot and loaded back as
/// the other, the passes after splitting each access as they do. A mask on
/// either side goes through the integer of its bits, which memory does not
/// hold as it does a mask's lanes.
fn vector_int_bitcasts(cx: &mut Ctx, f: &mut Function) {
    let i8t = cx.m.types.int(8);
    let v8i8 = cx.m.types.intern(Type::Vector { len: 8, elem: i8t, scalable: false });
    let ptr = cx.m.types.ptr(0);
    let i1 = cx.m.types.int(1);
    let mask_lanes = |cx: &Ctx, ty: TypeId| match cx.m.types.get(ty) {
        Type::Vector { len, elem, scalable: false } if *elem == i1 => Some(*len),
        _ => None,
    };
    // Held whole by a register (vectors only where there are vector
    // registers).
    let has_vectors = cx.vector_bits() > 0;
    let register = |cx: &Ctx, ty: TypeId| match cx.m.types.get(ty) {
        Type::Int(b) => *b <= 64,
        Type::Float | Type::Double | Type::Ptr(_) => true,
        Type::Vector { len, elem, scalable: false } => {
            let eb = cx.size_of(*elem).unwrap_or(0) * 8;
            has_vectors && *len > 1 && len.is_power_of_two() && eb >= 8 && matches!(len * eb as u32, 64 | 128)
        }
        _ => false,
    };
    let is_int = |cx: &Ctx, ty: TypeId| matches!(cx.m.types.get(ty), Type::Int(_));
    let Some(&entry) = f.layout.first() else { return };
    let mut slots = Vec::new();
    let blocks: Vec<BlockId> = f.layout.clone();
    for b in blocks {
        let mut k = 0;
        while k < f.blocks[b.index()].insts.len() {
            let i = f.blocks[b.index()].insts[k];
            let Op::Cast { op: CastOp::BitCast, value, .. } = f.insts[i.index()].op else {
                k += 1;
                continue;
            };
            let r = f.insts[i.index()].result.unwrap();
            let (from, to) = (cx.value_ty(f, value), f.locals[r.index()].ty);
            if from == to {
                k += 1;
                continue;
            }
            // A vector of one lane and its element: as they are (what
            // `one_lane_vectors` left at the edges); to anything else, by
            // its element.
            let elem_of = |cx: &Ctx, ty: TypeId| match cx.m.types.get(ty) {
                Type::Vector { len: 1, elem, scalable: false } => Some(*elem),
                _ => None,
            };
            if elem_of(cx, from) == Some(to) || elem_of(cx, to) == Some(from) {
                k += 1;
                continue;
            }
            if let Some(e) = elem_of(cx, from).or(elem_of(cx, to)) {
                let mut s = Seq { f, new: Vec::new() };
                let x = s.cast(CastOp::BitCast, value, e);
                let v = s.cast(CastOp::BitCast, x, to);
                s.f.replace_uses(Value::Local(r), v);
                let new = std::mem::take(&mut s.new);
                f.blocks[b.index()].insts.splice(k..=k, new);
                continue;
            }
            let fm = mask_lanes(cx, from);
            let tm = mask_lanes(cx, to);
            // A mask and the integer of its bits, and registers both: as
            // they are; a 64-bit vector and i64, through <8 x i8>.
            let both_regs = register(cx, from) && register(cx, to);
            let size = cx.size_of(to).unwrap_or(0);
            let is_vec = |cx: &Ctx, ty: TypeId| matches!(cx.m.types.get(ty), Type::Vector { .. });
            let vec_and_int = (is_int(cx, from) && is_vec(cx, to)) || (is_vec(cx, from) && is_int(cx, to));
            let direct = (fm.is_some() && is_int(cx, to)) || (tm.is_some() && is_int(cx, from)) || both_regs;
            let mut s = Seq { f, new: Vec::new() };
            let v = if direct {
                if both_regs && vec_and_int && size == 8 && from != v8i8 && to != v8i8 && fm.is_none() && tm.is_none() {
                    let x = s.cast(CastOp::BitCast, value, v8i8);
                    s.cast(CastOp::BitCast, x, to)
                } else {
                    k += 1;
                    continue;
                }
            } else if let Some(n) = fm {
                // From a mask: its bits as an integer first.
                let it = cx.m.types.int(n);
                let x = s.cast(CastOp::BitCast, value, it);
                s.cast(CastOp::BitCast, x, to)
            } else if let Some(n) = tm {
                let it = cx.m.types.int(n);
                let x = s.cast(CastOp::BitCast, value, it);
                s.cast(CastOp::BitCast, x, to)
            } else {
                let align = Some(size.next_power_of_two().clamp(1, 16));
                let slot = s.f.add_inst(
                    Op::Alloca { inalloca: false, ty: from, count: None, align, addrspace: None },
                    Some((None, ptr)),
                );
                slots.push(slot);
                let at = s.f.result(slot);
                s.op_void(Op::Store { volatile: false, atomic: None, value, ptr: at, align });
                s.op(Op::Load { volatile: false, atomic: None, ptr: at, align }, to)
            };
            s.f.replace_uses(Value::Local(r), v);
            let new = std::mem::take(&mut s.new);
            // The new bitcasts are looked at in turn (a mask's integer to
            // a vector may itself go through memory).
            f.blocks[b.index()].insts.splice(k..=k, new);
        }
    }
    // The slots, first in the entry block, as fixed allocas are.
    f.blocks[entry.index()].insts.splice(0..0, slots);
}

/// An `i1` in memory is a byte, 0 or 1: its stores store it zero-extended,
/// and its loads truncate the byte.
fn bool_memory(cx: &mut Ctx, f: &mut Function) {
    let (i1, i8t) = (cx.m.types.int(1), cx.m.types.int(8));
    let blocks: Vec<BlockId> = f.layout.clone();
    for b in blocks {
        let mut k = 0;
        while k < f.blocks[b.index()].insts.len() {
            let i = f.blocks[b.index()].insts[k];
            let mut s = Seq { f, new: Vec::new() };
            match s.f.insts[i.index()].op.clone() {
                Op::Store { volatile, atomic, value, ptr, align } if cx.value_ty(s.f, value) == i1 => {
                    let v = s.cast(CastOp::ZExt, value, i8t);
                    s.op_void(Op::Store { volatile, atomic, value: v, ptr, align });
                }
                Op::Load { volatile, atomic, ptr, align }
                    if s.f.insts[i.index()].result.is_some_and(|r| s.f.locals[r.index()].ty == i1) =>
                {
                    let r = s.f.insts[i.index()].result.unwrap();
                    let v = s.op(Op::Load { volatile, atomic, ptr, align }, i8t);
                    let t = s.cast(CastOp::Trunc, v, i1);
                    s.f.replace_uses(Value::Local(r), t);
                }
                _ => {
                    k += 1;
                    continue;
                }
            }
            let new = std::mem::take(&mut s.new);
            let n = new.len();
            f.blocks[b.index()].insts.splice(k..=k, new);
            k += n;
        }
    }
}

/// The most loads and stores a `memset` or `memcpy` is written out as.
const MAX_MEM_PIECES: u64 = 16;

/// Small `llvm.memset` and `llvm.memcpy` calls of a constant length are
/// written out as stores (and, for a copy, loads), when that is at most
/// `MAX_MEM_PIECES` of them; longer ones stay calls to the C library. Where
/// unaligned accesses are fast, every piece is as wide as the length
/// allows, up to a vector register (16 bytes, 32 with AVX2) past 16 bytes
/// and an integer register up to them, and the last overlaps the one
/// before rather than a ladder of narrower ones: 15 bytes are two 8-byte
/// stores, at 0 and 7. Elsewhere each piece is as wide as
/// the alignment and what is left allow (8, 4, 2 or 1 bytes).
/// `memcpyopt` turns runs of stores into memsets that come back here as
/// wider stores. Volatile ones, and memsets of a value not known, stay
/// calls.
fn expand_small_mem_ops(cx: &mut Ctx, f: &mut Function) {
    let blocks: Vec<BlockId> = f.layout.clone();
    for b in blocks {
        let mut k = 0;
        while k < f.blocks[b.index()].insts.len() {
            let i = f.blocks[b.index()].insts[k];
            let Op::Call(call) = &f.insts[i.index()].op else {
                k += 1;
                continue;
            };
            let name = callee_name(cx.m, call).unwrap_or_default();
            let copy = name.starts_with("llvm.memcpy.");
            if !(copy || name.starts_with("llvm.memset.")) || call.args.len() != 4 {
                k += 1;
                continue;
            }
            let arg = |n: usize| call.args[n].value;
            let align = |n: usize| {
                call.args[n]
                    .attrs
                    .iter()
                    .find_map(|a| match a {
                        Attr::Int { name, value, .. } if name == "align" => Some(*value),
                        _ => None,
                    })
                    .unwrap_or(1)
            };
            let (Some(len), Some(0)) = (cx.const_int(arg(2)), cx.const_int(arg(3))) else {
                k += 1;
                continue;
            };
            let byte = if copy { None } else { cx.const_int(arg(1)).map(|v| v as u8) };
            if len <= 0 || (!copy && byte.is_none()) {
                k += 1;
                continue;
            }
            let len = len as u64;
            // The pieces: (offset, bytes), each as wide as it can be.
            let mut pieces = Vec::new();
            if cx.t.machine.unaligned_fast {
                // (Up to 16 bytes, integer registers: their stores and loads
                // have less latency, and the value needs no vector made.)
                let widest = if len <= 16 { 8 } else { (cx.vector_bits() as u64 / 8).max(8) };
                let w = [32u64, 16, 8, 4, 2, 1].into_iter().find(|&w| w <= widest && w <= len).unwrap();
                let n = len.div_ceil(w);
                pieces.extend((0..n).map(|k| (if k + 1 == n { len - w } else { k * w }, w)));
            } else {
                let widest = if copy { align(0).min(align(1)) } else { align(0) }.clamp(1, 8);
                let mut at = 0u64;
                while at < len {
                    let w = [8u64, 4, 2, 1].into_iter().find(|&w| w <= widest && at + w <= len).unwrap();
                    pieces.push((at, w));
                    at += w;
                }
            }
            if pieces.len() as u64 > MAX_MEM_PIECES
                || !epitome_ir::chop::allow(|| {
                    format!("write the {name} at {} as {} accesses", epitome_ir::printer::inst_name(f, i), pieces.len())
                })
            {
                k += 1;
                continue;
            }
            let (dst, src) = (arg(0), arg(1));
            let (dst_align, src_align) = (align(0), if copy { align(1) } else { 1 });
            let (i8t, i64t, ptr) = (cx.m.types.int(8), cx.m.types.int(64), cx.m.types.ptr(0));
            let mut s = Seq { f, new: Vec::new() };
            // Two pieces at a time: both loads, then both stores, so that
            // the accesses are neighbours a load or store pair can take
            // (`peephole::pair_memory`). A memcpy's two ranges do not
            // overlap, so the order is free.
            for two in pieces.chunks(2) {
                let mut values = Vec::new();
                for &(off, w) in two {
                    // What alignment each piece really has: the call's, at
                    // most, and no more than its offset's.
                    let real = |a: u64| if off == 0 { a } else { a.min(1 << off.trailing_zeros()) };
                    // A vector of i64 for 16 and 32 bytes.
                    let ty = if w > 8 {
                        let i64t = cx.m.types.int(64);
                        cx.m.types.intern(Type::Vector { scalable: false, len: (w / 8) as u32, elem: i64t })
                    } else {
                        cx.m.types.int(8 * w as u32)
                    };
                    let value = match byte {
                        Some(b) if w > 8 => {
                            let i64t = cx.m.types.int(64);
                            let lane = cx.int(i64t, (0..8).fold(0u64, |acc, _| (acc << 8) | b as u64) as i64);
                            let Value::Const(lane) = lane else { unreachable!() };
                            Value::Const(
                                cx.m.consts.intern(Constant { ty, kind: ConstKind::Vector(vec![lane; (w / 8) as usize]) }),
                            )
                        }
                        Some(b) => {
                            let v = (0..w).fold(0u64, |acc, _| (acc << 8) | b as u64);
                            cx.int(ty, v as i64)
                        }
                        None => {
                            let from = offset_ptr(cx, &mut s, src, off, i8t, i64t, ptr);
                            s.op(
                                Op::Load {
                                    volatile: false,
                                    atomic: None,
                                    ptr: from,
                                    align: Some(real(src_align).min(w)),
                                },
                                ty,
                            )
                        }
                    };
                    values.push(value);
                }
                for (&(off, w), value) in two.iter().zip(values) {
                    let real = |a: u64| if off == 0 { a } else { a.min(1 << off.trailing_zeros()) };
                    let to = offset_ptr(cx, &mut s, dst, off, i8t, i64t, ptr);
                    let st = s.f.add_inst(
                        Op::Store {
                            volatile: false,
                            atomic: None,
                            value,
                            ptr: to,
                            align: Some(real(dst_align).min(w)),
                        },
                        None,
                    );
                    s.new.push(st);
                }
            }
            let new = std::mem::take(&mut s.new);
            let n = new.len();
            f.blocks[b.index()].insts.splice(k..=k, new);
            k += n;
        }
    }
}

/// Address sinking, the local half of LLVM's `codegenprepare`: a
/// `getelementptr` used as the address of loads and stores in other blocks
/// is copied into each of them, so that instruction selection, which looks
/// within a block, can fold it into the access (`ldr x0, [x1, #24]`, `ldr
/// x0, [x1, x2, lsl #3]`) instead of computing it into a register first.
///
/// Only address operands move, and only into blocks that are not on a
/// cycle the original's block is not on, unless the address is a constant
/// offset from its base (which always folds): an address that does not fold
/// would otherwise be computed every time round a loop. Each block gets one
/// copy, before its first use there. Each sink is an elective choice
/// (autochop).
fn sink_addresses(f: &mut Function) {
    let blocks = f.inst_blocks();
    let on_cycle = cycles(f);
    // Each GEP's address uses, by block.
    let mut uses: std::collections::HashMap<InstId, Vec<(BlockId, InstId)>> = Default::default();
    for &b in &f.layout {
        for &i in &f.blocks[b.index()].insts {
            let ptr = match &f.insts[i.index()].op {
                Op::Load { ptr, .. } | Op::Store { ptr, .. } => *ptr,
                _ => continue,
            };
            if let Value::Local(l) = ptr
                && let LocalDef::Inst(d) = f.locals[l.index()].def
                && matches!(f.insts[d.index()].op, Op::Gep { .. })
                && blocks[d.index()].is_some_and(|db| db != b)
            {
                uses.entry(d).or_default().push((b, i));
            }
        }
    }
    let mut gepped: Vec<InstId> = uses.keys().copied().collect();
    gepped.sort();
    for d in gepped {
        let Some(home) = blocks[d.index()] else { continue };
        let r = f.insts[d.index()].result.unwrap();
        // A constant offset from its base, which folds into any access.
        let constant = matches!(&f.insts[d.index()].op, Op::Gep { indices, .. }
            if indices.iter().all(|&x| matches!(x, Value::Const(_))));
        let mut list = uses[&d].clone();
        list.sort_by_key(|&(b, _)| b);
        let mut k = 0;
        while k < list.len() {
            let b = list[k].0;
            let users: Vec<InstId> = list[k..].iter().take_while(|x| x.0 == b).map(|x| x.1).collect();
            k += users.len();
            if on_cycle[b.index()].is_some() && on_cycle[b.index()] != on_cycle[home.index()] && !constant {
                continue;
            }
            if !epitome_ir::chop::allow(|| {
                format!(
                    "sink the address {} into {}",
                    epitome_ir::printer::local_name(f, r),
                    epitome_ir::printer::local_name(f, f.blocks[b.index()].local)
                )
            }) {
                continue;
            }
            let ty = f.locals[r.index()].ty;
            let copy = f.add_inst(f.insts[d.index()].op.clone(), Some((None, ty)));
            let first =
                users.iter().filter_map(|u| f.blocks[b.index()].insts.iter().position(|x| x == u)).min().unwrap();
            f.blocks[b.index()].insts.insert(first, copy);
            let new = f.result(copy);
            for u in users {
                match &mut f.insts[u.index()].op {
                    Op::Load { ptr, .. } | Op::Store { ptr, .. } => *ptr = new,
                    _ => {}
                }
            }
        }
    }
}

/// A constant offset from a base used by more than one instruction: each
/// load or store using it as its address gets its own copy, just before
/// it, so that selection (which folds only what one instruction uses) can
/// make it the access's offset (`ldr x0, [x1, #24]`) rather than compute it
/// into a register for them all. Each copy is an elective change
/// (autochop).
fn constant_addresses(f: &mut Function) {
    let mut count = vec![0u32; f.locals.len()];
    for i in f.insts_in_order() {
        f.insts[i.index()].op.for_each_value(|v| {
            if let Value::Local(l) = v {
                count[l.index()] += 1;
            }
        });
    }
    for b in f.layout.clone() {
        let mut k = 0;
        while k < f.blocks[b.index()].insts.len() {
            let i = f.blocks[b.index()].insts[k];
            k += 1;
            let (Op::Load { ptr: Value::Local(l), .. } | Op::Store { ptr: Value::Local(l), .. }) =
                f.insts[i.index()].op
            else {
                continue;
            };
            let LocalDef::Inst(d) = f.locals[l.index()].def else { continue };
            let constant = matches!(&f.insts[d.index()].op, Op::Gep { indices, .. }
                if indices.iter().all(|&x| matches!(x, Value::Const(_))));
            if !constant
                || count[l.index()] < 2
                || !epitome_ir::chop::allow(|| {
                    format!("copy the address {} for one access", epitome_ir::printer::local_name(f, l))
                })
            {
                continue;
            }
            let ty = f.locals[l.index()].ty;
            let copy = f.add_inst(f.insts[d.index()].op.clone(), Some((None, ty)));
            f.blocks[b.index()].insts.insert(k - 1, copy);
            k += 1;
            let new = f.result(copy);
            if let Op::Load { ptr, .. } | Op::Store { ptr, .. } = &mut f.insts[i.index()].op {
                *ptr = new;
            }
            count[l.index()] -= 1;
        }
    }
}

/// A scalar comparison used by more than one instruction: each select and
/// conditional branch in its block that it decides gets its own copy, just
/// before it, so that selection (which folds only what one instruction
/// uses) makes the compare and the `csel` or `b.cond` together, rather
/// than the flags made into a boolean (`cset`) and tested again (`tst`).
/// For a machine with flags; each copy is an elective change (autochop).
fn compares_per_use(cx: &Ctx, f: &mut Function) {
    let mut count = vec![0u32; f.locals.len()];
    for i in f.insts_in_order() {
        f.insts[i.index()].op.for_each_value(|v| {
            if let Value::Local(l) = v {
                count[l.index()] += 1;
            }
        });
    }
    let blocks = f.inst_blocks();
    for b in f.layout.clone() {
        let mut k = 0;
        while k < f.blocks[b.index()].insts.len() {
            let i = f.blocks[b.index()].insts[k];
            k += 1;
            let cond = match f.insts[i.index()].op {
                Op::Select { cond: Value::Local(l), .. } | Op::CondBr { cond: Value::Local(l), .. } => l,
                _ => continue,
            };
            let LocalDef::Inst(d) = f.locals[cond.index()].def else { continue };
            let compare = match &f.insts[d.index()].op {
                Op::ICmp { lhs, .. } | Op::FCmp { lhs, .. } => {
                    !matches!(cx.m.types.get(cx.value_ty(f, *lhs)), Type::Vector { .. })
                }
                _ => false,
            };
            if !compare
                || count[cond.index()] < 2
                || blocks[d.index()] != Some(b)
                || !epitome_ir::chop::allow(|| {
                    format!("copy the compare {} for one use", epitome_ir::printer::local_name(f, cond))
                })
            {
                continue;
            }
            let ty = f.locals[cond.index()].ty;
            let copy = f.add_inst(f.insts[d.index()].op.clone(), Some((None, ty)));
            f.blocks[b.index()].insts.insert(k - 1, copy);
            k += 1;
            let new = f.result(copy);
            match &mut f.insts[i.index()].op {
                Op::Select { cond, .. } | Op::CondBr { cond, .. } => *cond = new,
                _ => {}
            }
            count[cond.index()] -= 1;
        }
    }
}

/// For each block on a cycle, its strongly connected component (Tarjan's
/// algorithm); `None` for the rest.
fn cycles(f: &Function) -> Vec<Option<usize>> {
    let n = f.blocks.len();
    let succs: Vec<Vec<BlockId>> = (0..n).map(|b| f.successors(BlockId(b as u32))).collect();
    let (mut index, mut low, mut on_stack) = (vec![usize::MAX; n], vec![0; n], vec![false; n]);
    let mut stack = Vec::new();
    let mut comp = vec![None; n];
    let mut next = 0;
    let mut count = 0;
    for &root in &f.layout {
        if index[root.index()] != usize::MAX {
            continue;
        }
        // An explicit stack of (block, next successor to look at).
        let mut work = vec![(root.index(), 0usize)];
        index[root.index()] = next;
        low[root.index()] = next;
        next += 1;
        stack.push(root.index());
        on_stack[root.index()] = true;
        while let Some(&mut (v, ref mut k)) = work.last_mut() {
            if let Some(&w) = succs[v].get(*k) {
                *k += 1;
                let w = w.index();
                if index[w] == usize::MAX {
                    index[w] = next;
                    low[w] = next;
                    next += 1;
                    stack.push(w);
                    on_stack[w] = true;
                    work.push((w, 0));
                } else if on_stack[w] {
                    low[v] = low[v].min(index[w]);
                }
            } else {
                work.pop();
                if let Some(&(u, _)) = work.last() {
                    low[u] = low[u].min(low[v]);
                }
                if low[v] == index[v] {
                    let mut members = Vec::new();
                    loop {
                        let w = stack.pop().unwrap();
                        on_stack[w] = false;
                        members.push(w);
                        if w == v {
                            break;
                        }
                    }
                    let self_loop = succs[v].iter().any(|s| s.index() == v);
                    if members.len() > 1 || self_loop {
                        for w in members {
                            comp[w] = Some(count);
                        }
                        count += 1;
                    }
                }
            }
        }
    }
    comp
}

/// A dynamic alloca (`alloca T, i32 %n`) becomes one of bytes,
/// `alloca i8, i64 %size`, its size rounded up to keep `sp` 16-aligned.
fn lower_dynamic_allocas(cx: &mut Ctx, f: &mut Function) -> Result<(), String> {
    let i8t = cx.m.types.int(8);
    let i64t = cx.m.types.int(64);
    let blocks: Vec<BlockId> = f.layout.clone();
    for b in blocks {
        let mut k = 0;
        while k < f.blocks[b.index()].insts.len() {
            let i = f.blocks[b.index()].insts[k];
            let Op::Alloca { ty, count: Some(c), align, inalloca, addrspace } = f.insts[i.index()].op.clone() else {
                k += 1;
                continue;
            };
            if cx.const_int(c).is_some() {
                k += 1;
                continue;
            }
            let elem = cx.size_of(ty)? as i64;
            let mut s = Seq { f, new: Vec::new() };
            let n = match cx.m.types.get(cx.value_ty(s.f, c)) {
                Type::Int(64) => c,
                Type::Int(b) if *b > 64 => s.cast(CastOp::Trunc, c, i64t),
                _ => s.cast(CastOp::ZExt, c, i64t),
            };
            let size = cx.int(i64t, elem);
            let bytes = s.bin(BinOp::Mul, n, size, i64t);
            let fifteen = cx.int(i64t, 15);
            let up = s.bin(BinOp::Add, bytes, fifteen, i64t);
            let mask = cx.int(i64t, -16);
            let rounded = s.bin(BinOp::And, up, mask, i64t);
            let new = std::mem::take(&mut s.new);
            f.insts[i.index()].op = Op::Alloca {
                inalloca,
                ty: i8t,
                count: Some(rounded),
                align: Some(align.unwrap_or(16).max(16)),
                addrspace,
            };
            let n = new.len();
            f.blocks[b.index()].insts.splice(k..k, new);
            k += n + 1;
        }
    }
    Ok(())
}

/// A homogeneous floating-point aggregate, as AAPCS64 has them: up to four
/// members of one type, a float, a double or a vector of 64 or 128 bits
/// (`[3 x float]`, `{ <4 x float>, <4 x float> }`, as clang passes and
/// returns C's structs of them): how many, and the type.
pub(crate) fn hfa_of(m: &Module, ty: TypeId) -> Option<(usize, TypeId)> {
    let member = |t: TypeId| match m.types.get(t) {
        Type::Float | Type::Double => true,
        Type::Vector { scalable: false, len, elem } => {
            let eb = match m.types.get(*elem) {
                Type::Int(b @ (8 | 16 | 32 | 64)) => *b,
                Type::Float => 32,
                Type::Double => 64,
                _ => return false,
            };
            matches!(len * eb, 64 | 128)
        }
        _ => false,
    };
    let (n, t) = match m.types.get(ty) {
        Type::Array { len, elem } => (*len as usize, *elem),
        Type::Struct { fields, .. } if !fields.is_empty() && fields.iter().all(|f| *f == fields[0]) => {
            (fields.len(), fields[0])
        }
        Type::Named(i) => {
            let fields = &m.types.structs[*i as usize].body.as_ref()?.fields;
            if fields.is_empty() || fields.iter().any(|f| *f != fields[0]) {
                return None;
            }
            (fields.len(), fields[0])
        }
        _ => return None,
    };
    ((1..=4).contains(&n) && member(t)).then_some((n, t))
}

/// Rejects what the code generator cannot do yet.
fn check(cx: &Ctx, f: &Function, fname: &str) -> Result<(), String> {
    // Small structs, as clang passes them ([2 x i64]), live as their parts;
    // so do homogeneous floating-point ones, where the convention has them.
    let hfa = |ty: TypeId| cx.hfa && hfa_of(cx.m, ty).is_some();
    let small_struct = |ty: &Type| match ty {
        Type::Array { len, elem } => *len <= 2 && matches!(cx.m.types.get(*elem), Type::Int(64)),
        Type::Struct { fields, .. } => {
            fields.len() <= 2 && fields.iter().all(|f| matches!(cx.m.types.get(*f), Type::Int(64)))
        }
        _ => false,
    };
    let bad_type = |ty: TypeId| match cx.m.types.get(ty) {
        // i128 remains only in the high half of a product (see wide.rs).
        Type::Int(b) => *b > 64 && *b != 128,
        Type::Ptr(a) => *a != 0,
        Type::Void | Type::Label | Type::Double | Type::Float => false,
        // Vectors that do not fit a register are split (see
        // `legalize_vectors`); scalable ones are not supported.
        Type::Vector { scalable: false, elem, .. } => !matches!(
            cx.m.types.get(*elem),
            Type::Int(1 | 8 | 16 | 32 | 64) | Type::Float | Type::Double | Type::Ptr(0)
        ),
        t => !small_struct(t),
    };
    for p in &f.params {
        let ty = f.locals[p.local.index()].ty;
        if bad_type(ty) && !hfa(ty) {
            return Err(format!("@{fname}: parameters of this type are not supported yet"));
        }
    }
    for i in f.insts_in_order() {
        let inst = &f.insts[i.index()];
        if let Some(r) = inst.result
            && bad_type(f.locals[r.index()].ty)
            && !matches!(
                inst.op,
                Op::Alloca { .. } | Op::Call(_) | Op::Invoke { .. } | Op::LandingPad { .. } | Op::InsertValue { .. }
            )
        {
            return Err(format!(
                "@{fname}: values of type {} are not supported yet",
                type_name(cx, f.locals[r.index()].ty)
            ));
        }
        match &inst.op {
            Op::CallBr { .. } | Op::IndirectBr { .. } => {
                return Err(format!("@{fname}: indirect branches are not supported yet"));
            }
            Op::CatchSwitch { .. }
            | Op::CatchPad { .. }
            | Op::CleanupPad { .. }
            | Op::CatchRet { .. }
            | Op::CleanupRet { .. } => {
                return Err(format!("@{fname}: Windows exception handling is not supported"));
            }
            _ => {}
        }
    }
    Ok(())
}

fn type_name(cx: &Ctx, ty: TypeId) -> String {
    format!("{:?}", cx.m.types.get(ty))
}

pub fn callee_name(m: &Module, call: &Call) -> Option<String> {
    let Value::Const(c) = call.callee else { return None };
    match m.constant(c).kind {
        ConstKind::Global(g) => m.global(g).name.clone(),
        _ => None,
    }
}

fn drop_noop_calls(cx: &Ctx, f: &mut Function) {
    let order: Vec<BlockId> = f.layout.clone();
    for b in order {
        let insts = f.blocks[b.index()].insts.clone();
        let keep: Vec<InstId> = insts
            .into_iter()
            .filter(|&i| match &f.insts[i.index()].op {
                Op::Call(call) => !callee_name(cx.m, call).is_some_and(|n| {
                    n.starts_with("llvm.lifetime.")
                        || n.starts_with("llvm.dbg.")
                        || n == "llvm.assume"
                        || n.starts_with("llvm.experimental.noalias")
                }),
                _ => true,
            })
            .collect();
        f.blocks[b.index()].insts = keep;
    }
}

/// Edges to a block that is only `unreachable` are never taken (taking one
/// is undefined): a conditional branch with one goes the other way, and a
/// switch drops cases with one (or, for its default, takes its last case
/// as the default, so that it is not tested).
fn unreachable_edges(f: &mut Function) {
    let dead = |f: &Function, b: BlockId| matches!(&f.blocks[b.index()].insts[..], [i] if matches!(f.insts[i.index()].op, Op::Unreachable));
    for b in f.layout.clone() {
        let Some(t) = f.terminator(b) else { continue };
        let op = match f.insts[t.index()].op.clone() {
            Op::CondBr { then_dest, else_dest, .. } => match (dead(f, then_dest), dead(f, else_dest)) {
                (true, true) => Op::Unreachable,
                (true, false) => Op::Br(else_dest),
                (false, true) => Op::Br(then_dest),
                (false, false) => continue,
            },
            Op::Switch { value, mut default, mut cases } => {
                let n = cases.len();
                cases.retain(|&(_, to)| !dead(f, to));
                if dead(f, default)
                    && let Some((_, last)) = cases.pop()
                {
                    default = last;
                }
                if cases.len() == n && !dead(f, default) {
                    continue;
                }
                if cases.is_empty() { Op::Br(default) } else { Op::Switch { value, default, cases } }
            }
            _ => continue,
        };
        f.insts[t.index()].op = op;
    }
}

fn lower_switches(cx: &mut Ctx, f: &mut Function) {
    let blocks: Vec<BlockId> = f.layout.clone();
    for b in blocks {
        let Some(t) = f.terminator(b) else { continue };
        let Op::Switch { value, default, cases } = f.insts[t.index()].op.clone() else { continue };
        let ty = cx.value_ty(f, value);
        let i1 = cx.m.types.int(1);
        // The block each test is in: `b` for the first, then new ones.
        let mut at = b;
        let mut pos = f.layout.iter().position(|&x| x == b).unwrap() + 1;
        // The phi values `b` sent to each target.
        let phi_value = |f: &Function, target: BlockId, phi: InstId| -> Option<Value> {
            match &f.insts[phi.index()].op {
                Op::Phi { incoming, .. } => incoming.iter().find(|(_, p)| *p == b).map(|(v, _)| *v),
                _ => None,
            }
            .filter(|_| f.blocks[target.index()].insts.contains(&phi))
        };
        let mut edges: Vec<(BlockId, BlockId)> = Vec::new();
        f.blocks[b.index()].insts.pop();
        for (k, (c, target)) in cases.iter().enumerate() {
            let cmp = f.push_inst(
                at,
                Op::ICmp { pred: ICmpPred::Eq, samesign: false, lhs: value, rhs: Value::Const(*c) },
                Some((None, i1)),
            );
            let next = if k + 1 == cases.len() {
                default
            } else {
                let nb = f.add_block(None);
                f.layout.insert(pos, nb);
                pos += 1;
                nb
            };
            f.push_inst(at, Op::CondBr { cond: f.result(cmp), then_dest: *target, else_dest: next }, None);
            edges.push((at, *target));
            if k + 1 == cases.len() {
                edges.push((at, default));
            }
            at = next;
        }
        if cases.is_empty() {
            f.push_inst(b, Op::Br(default), None);
            continue;
        }
        let _ = ty;
        // Phis: one entry for each new edge, in place of `b`'s.
        let mut targets: Vec<BlockId> = edges.iter().map(|e| e.1).collect();
        targets.sort();
        targets.dedup();
        for target in targets {
            let phis: Vec<InstId> = f.blocks[target.index()]
                .insts
                .iter()
                .copied()
                .take_while(|&i| f.insts[i.index()].op.is_phi())
                .collect();
            for phi in phis {
                let Some(v) = phi_value(f, target, phi) else { continue };
                if let Op::Phi { incoming, .. } = &mut f.insts[phi.index()].op {
                    incoming.retain(|(_, p)| *p != b);
                    for &(from, to) in &edges {
                        if to == target {
                            incoming.push((v, from));
                        }
                    }
                }
            }
        }
    }
}

fn lower_geps(cx: &mut Ctx, f: &mut Function) -> Result<(), String> {
    let i64t = cx.i64t();
    let i8t = cx.i8t();
    let blocks: Vec<BlockId> = f.layout.clone();
    for b in blocks {
        let mut k = 0;
        while k < f.blocks[b.index()].insts.len() {
            let i = f.blocks[b.index()].insts[k];
            let Op::Gep { ptr, indices, source_ty, .. } = f.insts[i.index()].op.clone() else {
                k += 1;
                continue;
            };
            if indices.len() == 1
                && matches!(cx.m.types.get(source_ty), Type::Int(8))
                && cx.value_ty(f, indices[0]) == i64t
            {
                k += 1;
                continue;
            }
            // Constant part, and (index, scale) for the rest.
            let mut constant: i64 = 0;
            let mut terms: Vec<(Value, i64)> = Vec::new();
            let mut ty = source_ty;
            for (n, &idx) in indices.iter().enumerate() {
                let scale;
                if n == 0 {
                    scale = cx.size_of(ty)? as i64;
                } else {
                    match cx.m.types.get(ty).clone() {
                        Type::Struct { .. } | Type::Named(_) => {
                            let field = cx.const_int(idx).ok_or("a struct index that is not constant")? as u64;
                            let (fields, packed) = struct_fields(cx, ty)?;
                            let layout = cx.dl.struct_layout(&cx.m.types, &fields, packed).ok_or("struct layout")?;
                            constant += layout.offsets[field as usize] as i64;
                            ty = fields[field as usize];
                            continue;
                        }
                        Type::Array { elem, .. } | Type::Vector { elem, .. } => {
                            ty = elem;
                            scale = cx.size_of(elem)? as i64;
                        }
                        other => return Err(format!("getelementptr into {other:?}")),
                    }
                }
                match cx.const_int(idx) {
                    Some(c) => constant = constant.wrapping_add(c.wrapping_mul(scale)),
                    None => terms.push((idx, scale)),
                }
            }
            // Build the offset before the gep.
            let mut new: Vec<InstId> = Vec::new();
            let mut total: Option<Value> = None;
            for (idx, scale) in terms {
                let ity = cx.value_ty(f, idx);
                let mut v = idx;
                if ity != i64t {
                    let c = f.add_inst(
                        Op::Cast { op: CastOp::SExt, flags: IntFlags::NONE, fmf: FastMath::default(), value: v },
                        Some((None, i64t)),
                    );
                    new.push(c);
                    v = f.result(c);
                }
                if scale != 1 {
                    let op = if scale > 0 && (scale & (scale - 1)) == 0 {
                        let sh = cx.int(i64t, scale.trailing_zeros() as i64);
                        Op::Binary { op: BinOp::Shl, flags: IntFlags::NONE, fmf: FastMath::default(), lhs: v, rhs: sh }
                    } else {
                        let s = cx.int(i64t, scale);
                        Op::Binary { op: BinOp::Mul, flags: IntFlags::NONE, fmf: FastMath::default(), lhs: v, rhs: s }
                    };
                    let c = f.add_inst(op, Some((None, i64t)));
                    new.push(c);
                    v = f.result(c);
                }
                total = Some(match total {
                    None => v,
                    Some(t) => {
                        let c = f.add_inst(
                            Op::Binary {
                                op: BinOp::Add,
                                flags: IntFlags::NONE,
                                fmf: FastMath::default(),
                                lhs: t,
                                rhs: v,
                            },
                            Some((None, i64t)),
                        );
                        new.push(c);
                        f.result(c)
                    }
                });
            }
            // A constant offset goes on its own gep, so it can become an
            // addressing-mode offset.
            let mut base = ptr;
            if let Some(t) = total {
                if constant == 0 {
                    f.insts[i.index()].op = gep8(i8t, ptr, t);
                } else {
                    let ptr_ty = f.locals[f.insts[i.index()].result.unwrap().index()].ty;
                    let g = f.add_inst(gep8(i8t, ptr, t), Some((None, ptr_ty)));
                    new.push(g);
                    base = f.result(g);
                    let c = cx.int(i64t, constant);
                    f.insts[i.index()].op = gep8(i8t, base, c);
                }
            } else {
                let c = cx.int(i64t, constant);
                f.insts[i.index()].op = gep8(i8t, base, c);
            }
            let n = new.len();
            f.blocks[b.index()].insts.splice(k..k, new);
            k += n + 1;
        }
    }
    Ok(())
}

/// A variable offset from an address that is itself a constant offset
/// from another (`gep i8 (gep i8 %p, 360), %i`, as Rust makes for a field's
/// array) is the constant offset from the variable one, `gep i8 (gep i8 %p,
/// %i), 360`, so that the constant can be an addressing-mode offset; and
/// constant offsets of constant offsets are one.
fn constant_offsets_last(cx: &mut Ctx, f: &mut Function) {
    let i8t = cx.i8t();
    let offset = |f: &Function, v: Value| -> Option<(Value, Value)> {
        let Value::Local(l) = v else { return None };
        let LocalDef::Inst(d) = f.locals[l.index()].def else { return None };
        match &f.insts[d.index()].op {
            Op::Gep { ptr, indices, source_ty, .. } if *source_ty == i8t && indices.len() == 1 => {
                Some((*ptr, indices[0]))
            }
            _ => None,
        }
    };
    for b in f.layout.clone() {
        let mut k = 0;
        while k < f.blocks[b.index()].insts.len() {
            let i = f.blocks[b.index()].insts[k];
            k += 1;
            let Some(r) = f.insts[i.index()].result else { continue };
            let Some((mut base, t)) = offset(f, Value::Local(r)) else { continue };
            let mut c: i64 = 0;
            while let Some((p, o)) = offset(f, base)
                && let Some(n) = cx.const_int(o)
            {
                c = c.wrapping_add(n);
                base = p;
            }
            if c == 0 {
                continue;
            }
            let i64t = cx.i64t();
            if let Some(n) = cx.const_int(t) {
                // Constant offsets of constant offsets: one.
                let cv = cx.int(i64t, c.wrapping_add(n));
                f.insts[i.index()].op = gep8(i8t, base, cv);
                continue;
            }
            let ptr_ty = f.locals[r.index()].ty;
            let g = f.add_inst(gep8(i8t, base, t), Some((None, ptr_ty)));
            let cv = cx.int(i64t, c);
            f.insts[i.index()].op = gep8(i8t, f.result(g), cv);
            f.blocks[b.index()].insts.insert(k - 1, g);
            k += 1;
        }
    }
}

/// The register width an odd-sized integer lives in.
fn container(bits: u32) -> Option<u32> {
    match bits {
        1 | 8 | 16 | 32 | 64 => None,
        b if b < 32 => Some(32),
        b if b < 64 => Some(64),
        _ => None,
    }
}

fn promote_odd(cx: &mut Ctx, f: &mut Function) -> Result<(), String> {
    // The original width of each value that moves to a wider type.
    let mut orig: std::collections::HashMap<LocalId, u32> = std::collections::HashMap::new();
    for (k, l) in f.locals.iter_mut().enumerate() {
        if let Type::Int(b) = *cx.m.types.get(l.ty)
            && let Some(c) = container(b)
        {
            orig.insert(LocalId(k as u32), b);
            l.ty = cx.m.types.int(c);
        }
    }
    // (Or constants of them: a store of an `i24` constant.)
    let odd_const = || {
        f.insts_in_order().any(|i| {
            let mut odd = false;
            f.insts[i.index()].op.for_each_value(|v| {
                odd |= matches!(*v, Value::Const(c)
                    if matches!(*cx.m.types.get(cx.m.constant(c).ty), Type::Int(b) if container(b).is_some()));
            });
            odd
        })
    };
    if orig.is_empty() && !odd_const() {
        return Ok(());
    }
    let bits_of = |cx: &Ctx, f: &Function, orig: &std::collections::HashMap<LocalId, u32>, v: Value| -> u32 {
        match v {
            Value::Local(x) => orig.get(&x).copied().unwrap_or_else(|| match cx.m.types.get(f.locals[x.index()].ty) {
                Type::Int(b) => *b,
                _ => 64,
            }),
            Value::Const(c) => match cx.m.types.get(cx.m.constant(c).ty) {
                Type::Int(b) => *b,
                _ => 64,
            },
            _ => 64,
        }
    };
    let blocks: Vec<BlockId> = f.layout.clone();
    for b in blocks {
        let mut k = 0;
        while k < f.blocks[b.index()].insts.len() {
            let i = f.blocks[b.index()].insts[k];
            let mut new: Vec<InstId> = Vec::new();
            // What a store stores, before its constant is widened below:
            // a store of an `i24` constant writes three bytes, not four.
            let stored_bits = match &f.insts[i.index()].op {
                Op::Store { value, .. } => Some(bits_of(cx, f, &orig, *value)),
                _ => None,
            };
            // Constants of odd widths become constants of the wider type.
            let mut op = f.insts[i.index()].op.clone();
            op.for_each_value_mut(|v| {
                if let Value::Const(c) = *v
                    && let Type::Int(bits) = *cx.m.types.get(cx.m.constant(c).ty)
                    && let Some(w) = container(bits)
                    && let Some(n) = cx.const_int(*v)
                {
                    let t = cx.m.types.int(w);
                    *v = cx.int(t, n);
                }
            });
            // Operations that read the top bits get their operands made
            // right: masked for unsigned readings, sign-extended for signed.
            let mut fix = |cx: &mut Ctx, f: &mut Function, v: Value, bits: u32, signed: bool| -> Value {
                let Some(w) = container(bits) else { return v };
                let t = cx.m.types.int(w);
                let nf = IntFlags::NONE;
                let fm = FastMath::default();
                if signed {
                    let sh = cx.int(t, (w - bits) as i64);
                    let a =
                        f.add_inst(Op::Binary { op: BinOp::Shl, flags: nf, fmf: fm, lhs: v, rhs: sh }, Some((None, t)));
                    let r = f.add_inst(
                        Op::Binary { op: BinOp::AShr, flags: nf, fmf: fm, lhs: f.result(a), rhs: sh },
                        Some((None, t)),
                    );
                    new.push(a);
                    new.push(r);
                    f.result(r)
                } else {
                    let mask = cx.int(t, ((1u64 << bits) - 1) as i64);
                    let a = f.add_inst(
                        Op::Binary { op: BinOp::And, flags: nf, fmf: fm, lhs: v, rhs: mask },
                        Some((None, t)),
                    );
                    new.push(a);
                    f.result(a)
                }
            };
            let res = f.insts[i.index()].result;
            let res_odd = res.and_then(|r| orig.get(&r).copied());
            if matches!(op, Op::Load { .. } | Op::Store { .. }) {
                let mut odd = res_odd.is_some() || stored_bits.and_then(container).is_some();
                op.for_each_value(|v| odd |= container(bits_of(cx, f, &orig, *v)).is_some());
                // A whole number of bytes: in pieces of 4, 2 and 1 bytes,
                // the lowest first (little-endian).
                let bits = match stored_bits {
                    Some(b) => b,
                    None => res_odd.unwrap_or(0),
                };
                let plain = matches!(
                    &op,
                    Op::Store { volatile: false, atomic: None, .. } | Op::Load { volatile: false, atomic: None, .. }
                );
                if odd && !(plain && bits % 8 == 0) {
                    return Err("loads and stores of odd-sized integers are not supported yet".into());
                }
                if odd {
                    let w = cx.m.types.int(container(bits).unwrap());
                    let (i8t, i64t, ptr_ty) = (cx.m.types.int(8), cx.m.types.int(64), cx.m.types.ptr(0));
                    let mut pieces = Vec::new();
                    let mut off = 0;
                    while off < bits / 8 {
                        let size = [4, 2, 1].into_iter().find(|&n| off + n <= bits / 8).unwrap();
                        pieces.push((off, size));
                        off += size;
                    }
                    let nf = IntFlags::NONE;
                    let fm = FastMath::default();
                    let mut add = |f: &mut Function, op: Op, ty: Option<TypeId>| -> Value {
                        let x = f.add_inst(op, ty.map(|t| (None, t)));
                        new.push(x);
                        if ty.is_some() { f.result(x) } else { Value::Const(ConstId(0)) }
                    };
                    let mut last = None;
                    match op.clone() {
                        Op::Store { value, ptr, align, .. } => {
                            for (n, &(off, size)) in pieces.iter().enumerate() {
                                let pt = cx.m.types.int(8 * size as u32);
                                let sh = cx.int(w, 8 * off as i64);
                                let v = if off == 0 {
                                    value
                                } else {
                                    add(
                                        f,
                                        Op::Binary { op: BinOp::LShr, flags: nf, fmf: fm, lhs: value, rhs: sh },
                                        Some(w),
                                    )
                                };
                                let v = add(f, Op::Cast { op: CastOp::Trunc, flags: nf, fmf: fm, value: v }, Some(pt));
                                let o = cx.int(i64t, off as i64);
                                let p = add(f, gep8(i8t, ptr, o), Some(ptr_ty));
                                let st = Op::Store {
                                    volatile: false,
                                    atomic: None,
                                    value: v,
                                    ptr: p,
                                    align: align.map(|a| a.min(size as u64)),
                                };
                                if n + 1 == pieces.len() {
                                    last = Some(st);
                                } else {
                                    add(f, st, None);
                                }
                            }
                        }
                        Op::Load { ptr, align, .. } => {
                            let mut acc = None;
                            for (n, &(off, size)) in pieces.iter().enumerate() {
                                let pt = cx.m.types.int(8 * size as u32);
                                let o = cx.int(i64t, off as i64);
                                let p = add(f, gep8(i8t, ptr, o), Some(ptr_ty));
                                let v = add(
                                    f,
                                    Op::Load {
                                        volatile: false,
                                        atomic: None,
                                        ptr: p,
                                        align: align.map(|a| a.min(size as u64)),
                                    },
                                    Some(pt),
                                );
                                let v = add(f, Op::Cast { op: CastOp::ZExt, flags: nf, fmf: fm, value: v }, Some(w));
                                let sh = cx.int(w, 8 * off as i64);
                                let v = if off == 0 {
                                    v
                                } else {
                                    add(f, Op::Binary { op: BinOp::Shl, flags: nf, fmf: fm, lhs: v, rhs: sh }, Some(w))
                                };
                                acc = Some(match acc {
                                    None => v,
                                    Some(a) if n + 1 == pieces.len() => {
                                        last = Some(Op::Binary { op: BinOp::Or, flags: nf, fmf: fm, lhs: a, rhs: v });
                                        break;
                                    }
                                    Some(a) => add(
                                        f,
                                        Op::Binary { op: BinOp::Or, flags: nf, fmf: fm, lhs: a, rhs: v },
                                        Some(w),
                                    ),
                                });
                            }
                        }
                        _ => unreachable!(),
                    }
                    f.insts[i.index()].op = last.unwrap();
                    let n = new.len();
                    f.blocks[b.index()].insts.splice(k..k, new);
                    k += n + 1;
                    continue;
                }
            }
            match &mut op {
                Op::Binary { op: bop, flags, lhs, rhs, .. } => {
                    let bits = bits_of(cx, f, &orig, *lhs);
                    if container(bits).is_some() {
                        *flags = IntFlags::NONE;
                        match bop {
                            BinOp::LShr | BinOp::UDiv | BinOp::URem => {
                                *lhs = fix(cx, f, *lhs, bits, false);
                                *rhs = fix(cx, f, *rhs, bits, false);
                            }
                            BinOp::AShr => {
                                *lhs = fix(cx, f, *lhs, bits, true);
                                *rhs = fix(cx, f, *rhs, bits, false);
                            }
                            BinOp::SDiv | BinOp::SRem => {
                                *lhs = fix(cx, f, *lhs, bits, true);
                                *rhs = fix(cx, f, *rhs, bits, true);
                            }
                            _ => {}
                        }
                    }
                }
                Op::ICmp { pred, lhs, rhs, .. } => {
                    let bits = bits_of(cx, f, &orig, *lhs);
                    let signed = matches!(pred, ICmpPred::Sgt | ICmpPred::Sge | ICmpPred::Slt | ICmpPred::Sle);
                    *lhs = fix(cx, f, *lhs, bits, signed);
                    *rhs = fix(cx, f, *rhs, bits, signed);
                }
                Op::Cast { op: cop, flags, value, .. } => {
                    let from = bits_of(cx, f, &orig, *value);
                    let to_ty = f.locals[res.unwrap().index()].ty;
                    let to = match cx.m.types.get(to_ty) {
                        Type::Int(b) => *b,
                        _ => 64,
                    };
                    let from_odd = container(from).is_some();
                    if from_odd || res_odd.is_some() {
                        *flags = IntFlags::NONE;
                        if from_odd && matches!(cop, CastOp::ZExt | CastOp::SExt) {
                            *value = fix(cx, f, *value, from, *cop == CastOp::SExt);
                        }
                        // Between equal register widths, the cast is free; from
                        // a wider container (an i3 held in 32 bits, made right
                        // above, extended to i8) it takes the low bits.
                        let from_w = container(from).unwrap_or(from);
                        if from_w == to {
                            *cop = CastOp::BitCast;
                        } else if from_w > to && matches!(cop, CastOp::ZExt | CastOp::SExt) {
                            *cop = CastOp::Trunc;
                        }
                    }
                }
                _ => {}
            }
            // A bitcast between integers is the identity; use freeze, which
            // is a plain copy, as the selector treats bitcasts of pointers.
            // (Not one to or from a float or a vector: those move bits
            // between register files.)
            if let Op::Cast { op: CastOp::BitCast, value, .. } = op
                && let Some(r) = res
                && matches!(cx.m.types.get(f.locals[r.index()].ty), Type::Int(_))
                && matches!(cx.m.types.get(cx.value_ty(f, value)), Type::Int(_))
            {
                op = Op::Freeze(value);
            }
            f.insts[i.index()].op = op;
            let n = new.len();
            f.blocks[b.index()].insts.splice(k..k, new);
            k += n + 1;
        }
    }
    Ok(())
}

fn split_aggregate_memory(cx: &mut Ctx, f: &mut Function) {
    let i64t = cx.m.types.int(64);
    let i8t = cx.m.types.int(8);
    let parts_of = |cx: &Ctx, ty: TypeId| -> Option<u32> {
        match cx.m.types.get(ty) {
            Type::Array { len, elem } if *len <= 2 && matches!(cx.m.types.get(*elem), Type::Int(64)) => {
                Some(*len as u32)
            }
            Type::Struct { fields, .. }
                if fields.len() <= 2 && fields.iter().all(|t| matches!(cx.m.types.get(*t), Type::Int(64))) =>
            {
                Some(fields.len() as u32)
            }
            _ => None,
        }
    };
    let blocks: Vec<BlockId> = f.layout.clone();
    for b in blocks {
        let mut k = 0;
        while k < f.blocks[b.index()].insts.len() {
            let i = f.blocks[b.index()].insts[k];
            let mut s = Seq { f, new: Vec::new() };
            let done = match s.f.insts[i.index()].op.clone() {
                Op::Store { value, ptr, volatile: false, atomic: None, .. } => {
                    let ty = cx.value_ty(s.f, value);
                    match parts_of(cx, ty) {
                        Some(n) => {
                            let ptr_ty = cx.m.types.ptr(0);
                            for part in 0..n {
                                let e = s.op(Op::ExtractValue { aggregate: value, indices: vec![part] }, i64t);
                                let off = cx.int(i64t, 8 * part as i64);
                                let p = s.op(gep8(i8t, ptr, off), ptr_ty);
                                let st = s.f.add_inst(
                                    Op::Store { volatile: false, atomic: None, value: e, ptr: p, align: Some(8) },
                                    None,
                                );
                                s.new.push(st);
                            }
                            true
                        }
                        None => false,
                    }
                }
                Op::Load { ptr, volatile: false, atomic: None, .. } => {
                    let r = s.f.insts[i.index()].result.unwrap();
                    let ty = s.f.locals[r.index()].ty;
                    match parts_of(cx, ty) {
                        Some(n) => {
                            let ptr_ty = cx.m.types.ptr(0);
                            let mut agg = Value::Const(cx.m.consts.intern(Constant { ty, kind: ConstKind::Poison }));
                            for part in 0..n {
                                let off = cx.int(i64t, 8 * part as i64);
                                let p = s.op(gep8(i8t, ptr, off), ptr_ty);
                                let v = s.op(Op::Load { volatile: false, atomic: None, ptr: p, align: Some(8) }, i64t);
                                agg = s.op(Op::InsertValue { aggregate: agg, element: v, indices: vec![part] }, ty);
                            }
                            s.f.replace_uses(Value::Local(r), agg);
                            true
                        }
                        None => false,
                    }
                }
                _ => false,
            };
            let new = std::mem::take(&mut s.new);
            if done {
                let n = new.len();
                f.blocks[b.index()].insts.splice(k..=k, new);
                k += n;
            } else {
                k += 1;
            }
        }
    }
}

/// Arithmetic with overflow (`llvm.sadd.with.overflow.i32`, as Rust's
/// overflow checks call it): the result and whether it overflowed, as a
/// struct the selector keeps as its parts. It runs before wide integers
/// are split, so `i128` sums are, and a 64-bit product's high half is the
/// shape `wide.rs` keeps for instructions such as `umulh`.
///
/// - unsigned sums overflow if the result is below an operand, differences
///   if the first operand is below the second;
/// - signed ones if the result's sign differs from both operands' (sums),
///   or from the first's while the operands' differ (differences);
/// - products narrower than 64 bits are made at twice the width and
///   overflow if they do not fit back; 64-bit ones overflow if the high
///   half is not zero (unsigned) or not the low half's sign (signed: the
///   unsigned high half less each operand where the other is negative).
fn expand_overflow(cx: &mut Ctx, f: &mut Function) -> Result<(), String> {
    let blocks: Vec<BlockId> = f.layout.clone();
    for b in blocks {
        let mut k = 0;
        while k < f.blocks[b.index()].insts.len() {
            let i = f.blocks[b.index()].insts[k];
            let Op::Call(call) = &f.insts[i.index()].op else {
                k += 1;
                continue;
            };
            let name = callee_name(cx.m, call).unwrap_or_default();
            let Some(kind) = name.strip_prefix("llvm.").and_then(|n| n.split_once(".with.overflow")).map(|x| x.0)
            else {
                k += 1;
                continue;
            };
            let kind = kind.to_string();
            let (a, b2) = (call.args[0].value, call.args[1].value);
            let r = f.insts[i.index()].result.ok_or("an overflow intrinsic without a result")?;
            let sty = f.locals[r.index()].ty;
            let ty = cx.value_ty(f, a);
            let Type::Int(bits) = *cx.m.types.get(ty) else {
                return Err(format!("{name} of vectors is not supported yet"));
            };
            let i1 = cx.m.types.int(1);
            let mut s = Seq { f, new: Vec::new() };
            let zero = cx.int(ty, 0);
            let cmp = |s: &mut Seq, pred, l, r| s.op(Op::ICmp { pred, samesign: false, lhs: l, rhs: r }, i1);
            let (value, ov) = match kind.as_str() {
                "uadd" | "sadd" | "usub" | "ssub" => {
                    let op = if kind.ends_with("add") { BinOp::Add } else { BinOp::Sub };
                    let v = s.bin(op, a, b2, ty);
                    let ov = match kind.as_str() {
                        "uadd" => cmp(&mut s, ICmpPred::Ult, v, a),
                        "usub" => cmp(&mut s, ICmpPred::Ult, a, b2),
                        _ => {
                            let (x, y) = if kind == "sadd" { (a, b2) } else { (a, a) };
                            let p = s.bin(BinOp::Xor, x, v, ty);
                            let q = if kind == "sadd" {
                                s.bin(BinOp::Xor, b2, v, ty)
                            } else {
                                s.bin(BinOp::Xor, y, b2, ty)
                            };
                            let both = s.bin(BinOp::And, p, q, ty);
                            cmp(&mut s, ICmpPred::Slt, both, zero)
                        }
                    };
                    (v, ov)
                }
                "umul" | "smul" if bits <= 32 => {
                    let wide = cx.m.types.int(2 * bits);
                    let ext = if kind == "umul" { CastOp::ZExt } else { CastOp::SExt };
                    let (wa, wb) = (s.cast(ext, a, wide), s.cast(ext, b2, wide));
                    let p = s.bin(BinOp::Mul, wa, wb, wide);
                    let v = s.cast(CastOp::Trunc, p, ty);
                    let back = s.cast(ext, v, wide);
                    (v, cmp(&mut s, ICmpPred::Ne, back, p))
                }
                "umul" | "smul" if bits == 64 => {
                    let i128t = cx.m.types.int(128);
                    let v = s.bin(BinOp::Mul, a, b2, ty);
                    let (wa, wb) = (s.cast(CastOp::ZExt, a, i128t), s.cast(CastOp::ZExt, b2, i128t));
                    let p = s.bin(BinOp::Mul, wa, wb, i128t);
                    let sh = cx.int(i128t, 64);
                    let h = s.bin(BinOp::LShr, p, sh, i128t);
                    let mut hi = s.cast(CastOp::Trunc, h, ty);
                    if kind == "umul" {
                        (v, cmp(&mut s, ICmpPred::Ne, hi, zero))
                    } else {
                        let c63 = cx.int(ty, 63);
                        for (x, y) in [(a, b2), (b2, a)] {
                            let sign = s.bin(BinOp::AShr, x, c63, ty);
                            let take = s.bin(BinOp::And, sign, y, ty);
                            hi = s.bin(BinOp::Sub, hi, take, ty);
                        }
                        let sign = s.bin(BinOp::AShr, v, c63, ty);
                        (v, cmp(&mut s, ICmpPred::Ne, hi, sign))
                    }
                }
                // 128 bits, from the 64-bit halves (a = ah:al, b = bh:bl):
                // it overflows if both high halves are nonzero, if either
                // cross product (ah*bl, al*bh) needs more than 64 bits, or
                // if the middle column (their low halves, plus the carry
                // from al*bl) carries out.
                "umul" if bits == 128 => umul128_overflow(cx, &mut s, a, b2, ty),
                // Signed, by the magnitudes: the unsigned product must not
                // overflow, and must fit the sign: up to 2^127 - 1, or
                // 2^127 when negative.
                "smul" if bits == 128 => {
                    let zero = cx.int(ty, 0);
                    let mag = |s: &mut Seq, x: Value| {
                        let neg = s.op(Op::ICmp { pred: ICmpPred::Slt, samesign: false, lhs: x, rhs: zero }, i1);
                        let minus = s.bin(BinOp::Sub, zero, x, ty);
                        let m = s.op(
                            Op::Select { fmf: FastMath::default(), cond: neg, then_value: minus, else_value: x },
                            ty,
                        );
                        (neg, m)
                    };
                    let (na, ma) = mag(&mut s, a);
                    let (nb, mb) = mag(&mut s, b2);
                    let (p, uov) = umul128_overflow(cx, &mut s, ma, mb, ty);
                    let neg = s.bin(BinOp::Xor, na, nb, i1);
                    let max = Value::Const(cx.m.consts.intern(Constant { ty, kind: ConstKind::Int(signed_max(128)) }));
                    let extra = s.cast(CastOp::ZExt, neg, ty);
                    let limit = s.bin(BinOp::Add, max, extra, ty);
                    let past = cmp(&mut s, ICmpPred::Ugt, p, limit);
                    let ov = s.bin(BinOp::Or, uov, past, i1);
                    (s.bin(BinOp::Mul, a, b2, ty), ov)
                }
                _ => return Err(format!("{name} is not supported yet")),
            };
            replace_pair(cx, &mut s, r, sty, value, ov);
            let new = std::mem::take(&mut s.new);
            // (Extractvalues before it in its block were removed.)
            let k0 = f.blocks[b.index()].insts.iter().position(|&j| j == i).unwrap();
            k = k0;
            let n = new.len();
            f.blocks[b.index()].insts.splice(k..=k, new);
            k += n;
        }
    }
    Ok(())
}

/// The uses of `r`, a pair (`{ i64, i1 }`) now made as `a` and `b`: its
/// extractvalues take them; anything else, a struct made of them (at the
/// end of `s`). The extractvalues are removed from their blocks.
pub(crate) fn replace_pair(cx: &mut Ctx, s: &mut Seq, r: LocalId, sty: TypeId, a: Value, b: Value) {
    let mut gone = Vec::new();
    let mut others = false;
    let order: Vec<InstId> = s.f.insts_in_order().collect();
    for j in order {
        match &s.f.insts[j.index()].op {
            Op::ExtractValue { aggregate: Value::Local(x), indices } if *x == r && indices.len() == 1 => {
                let part = if indices[0] == 0 { a } else { b };
                if let Some(e) = s.f.insts[j.index()].result {
                    s.f.replace_uses(Value::Local(e), part);
                }
                gone.push(j);
            }
            op => op.for_each_value(|v| others |= *v == Value::Local(r)),
        }
    }
    for blk in s.f.blocks.iter_mut() {
        blk.insts.retain(|j| !gone.contains(j));
    }
    if others {
        let poison = Value::Const(cx.m.consts.intern(Constant { ty: sty, kind: ConstKind::Poison }));
        let x = s.op(Op::InsertValue { aggregate: poison, element: a, indices: vec![0] }, sty);
        let y = s.op(Op::InsertValue { aggregate: x, element: b, indices: vec![1] }, sty);
        s.f.replace_uses(Value::Local(r), y);
    }
}

fn scalar_float(cx: &Ctx, ty: TypeId) -> bool {
    matches!(cx.m.types.get(ty), Type::Float | Type::Double)
}

/// The C library function an intrinsic (`llvm.log2`) is, if it is one.
fn libm_base(base: &str) -> Option<&'static str> {
    Some(match base.strip_prefix("llvm.")? {
        "sin" => "sin",
        "cos" => "cos",
        "tan" => "tan",
        "asin" => "asin",
        "acos" => "acos",
        "atan" => "atan",
        "atan2" => "atan2",
        "sinh" => "sinh",
        "cosh" => "cosh",
        "tanh" => "tanh",
        "exp" => "exp",
        "exp2" => "exp2",
        "exp10" => "exp10",
        "log" => "log",
        "log2" => "log2",
        "log10" => "log10",
        "pow" => "pow",
        "fma" => "fma",
        "powi" => "powi",
        "ldexp" => "ldexp",
        // Rounding, where the machine has no instruction for it (x86-64
        // before SSE4.1's roundsd).
        "floor" => "floor",
        "ceil" => "ceil",
        "trunc" => "trunc",
        "rint" => "rint",
        "nearbyint" => "nearbyint",
        "round" => "round",
        "roundeven" => "roundeven",
        // And the least and greatest: C's fmin and fmax ignore a NaN as
        // minnum does; fminimum and fmaximum are IEEE 754-2019's.
        "minnum" => "fmin",
        "maxnum" => "fmax",
        "minimum" => "fminimum",
        "maximum" => "fmaximum",
        "minimumnum" => "fminimum_num",
        "maximumnum" => "fmaximum_num",
        _ => return None,
    })
}

/// The C library's name for function `c` (`log2`) at floating-point type
/// `ty` (or its lanes'): `log2` for double, `log2f` for float. `powi` is
/// the runtime's (`__powidf2`, `__powisf2`); Apple's `exp10` is `__exp10`.
/// Whether `name`, an intrinsic on vectors (`llvm.floor.v8f32`), has an
/// instruction for vectors of the same elements at some width
/// (`llvm.floor.v4f32`).
fn selects_at_some_width(cx: &Ctx, name: &str) -> bool {
    let Some((base, ty)) = name.rsplit_once('.') else { return false };
    let Some(elem) = ty.strip_prefix('v').map(|t| t.trim_start_matches(|c: char| c.is_ascii_digit())) else {
        return false;
    };
    cx.t.intrinsics.iter().any(|n| {
        n.rsplit_once('.').is_some_and(|(b, t)| {
            b == base && t.strip_prefix('v').is_some_and(|t| t.trim_start_matches(|c: char| c.is_ascii_digit()) == elem)
        }) && cx.selects_intrinsic(n)
    })
}

fn libm_name(cx: &Ctx, c: &str, ty: TypeId) -> Option<String> {
    let elem = match cx.m.types.get(ty) {
        Type::Vector { elem, .. } => *elem,
        _ => ty,
    };
    let float = match cx.m.types.get(elem) {
        Type::Double => false,
        Type::Float => true,
        _ => return None,
    };
    // (Apple's C library has exp10 as __exp10.)
    let darwin = cx.m.triple.as_deref().is_some_and(|t| {
        let t = String::from_utf8_lossy(t);
        t.contains("apple") || t.contains("darwin") || t.contains("macos")
    });
    Some(match (c, float) {
        ("powi", false) => "__powidf2".into(),
        ("powi", true) => "__powisf2".into(),
        ("exp10", false) if darwin => "__exp10".into(),
        ("exp10", true) if darwin => "__exp10f".into(),
        (c, false) => c.into(),
        (c, true) => format!("{c}f"),
    })
}

/// A call of `name` with `args`, of result type `ty`; of a vector type,
/// one for each lane (vector arguments taken lane by lane, the others as
/// they are).
fn lanewise_call(cx: &mut Ctx, s: &mut Seq, name: &str, args: Vec<Value>, ty: TypeId) -> Value {
    let Type::Vector { len, elem, .. } = *cx.m.types.get(ty) else {
        return call_intrinsic(cx, s, name, args, ty);
    };
    let i64t = cx.m.types.int(64);
    let mut v = Value::Const(cx.m.consts.intern(Constant { ty, kind: ConstKind::Poison }));
    for lane in 0..len {
        let idx = cx.int(i64t, lane as i64);
        let mut a = Vec::new();
        for &x in &args {
            match *cx.m.types.get(cx.value_ty(s.f, x)) {
                Type::Vector { elem: e, .. } => a.push(s.op(Op::ExtractElement { vector: x, index: idx }, e)),
                _ => a.push(x),
            }
        }
        let y = call_intrinsic(cx, s, name, a, elem);
        v = s.op(Op::InsertElement { vector: v, element: y, index: idx }, ty);
    }
    v
}

/// A conversion of float `x` to an integer of type `ty` (`bits` wide, up to
/// 128) that saturates (`llvm.fptosi.sat`): the limits outside the range,
/// NaN as 0, else the plain conversion (defined there).
fn saturating_convert(cx: &mut Ctx, s: &mut Seq, x: Value, ty: TypeId, bits: u32, signed: bool) -> Value {
    let i1 = cx.m.types.int(1);
    let fty = cx.value_ty(s.f, x);
    let fc = |cx: &mut Ctx, v: f64| {
        let b = match cx.m.types.get(fty) {
            Type::Float => u128::from((v as f32).to_bits()),
            _ => u128::from(v.to_bits()),
        };
        Value::Const(cx.m.consts.intern(Constant { ty: fty, kind: ConstKind::Float(b) }))
    };
    let fcmp = |s: &mut Seq, pred, l, r| s.op(Op::FCmp { pred, fmf: FastMath::default(), lhs: l, rhs: r }, i1);
    let sel =
        |s: &mut Seq, c, t, e| s.op(Op::Select { fmf: FastMath::default(), cond: c, then_value: t, else_value: e }, ty);
    let big = |cx: &mut Ctx, a: ApInt| Value::Const(cx.m.consts.intern(Constant { ty, kind: ConstKind::Int(a) }));
    let (min, max, lo_bound, hi_bound) = if signed {
        let h = 2f64.powi(bits as i32 - 1);
        let max = signed_max(bits);
        // 2^(bits - 1), the sign bit alone.
        let min_hex = format!("{:x}", 1u32 << ((bits - 1) % 4)) + &"0".repeat(((bits - 1) / 4) as usize);
        let min = ApInt::parse_hex(bits, &min_hex, false).unwrap();
        (big(cx, min), big(cx, max), -h, h)
    } else {
        (cx.int(ty, 0), cx.int(ty, -1), -1.0, 2f64.powi(bits as i32))
    };
    let (lo_c, hi_c) = (fc(cx, lo_bound), fc(cx, hi_bound));
    let plain = s.cast(if signed { CastOp::FPToSI } else { CastOp::FPToUI }, x, ty);
    // Signed: below -2^(w-1) the minimum; unsigned: at -1 or below, 0
    // (between -1 and 0 the conversion gives 0).
    let below = fcmp(s, if signed { FCmpPred::Olt } else { FCmpPred::Ole }, x, lo_c);
    let v = sel(s, below, min, plain);
    let above = fcmp(s, FCmpPred::Oge, x, hi_c);
    let v = sel(s, above, max, v);
    let nan = fcmp(s, FCmpPred::Uno, x, x);
    let zero = cx.int(ty, 0);
    sel(s, nan, zero, v)
}

/// The largest signed value of `bits` bits, 2^(bits - 1) - 1.
fn signed_max(bits: u32) -> ApInt {
    let ones = bits - 1;
    let lead = ones % 4;
    let mut hex = String::new();
    if lead > 0 {
        hex.push(char::from_digit((1 << lead) - 1, 16).unwrap());
    }
    hex.push_str(&"f".repeat((ones / 4) as usize));
    // (Of one bit: 0.)
    if hex.is_empty() {
        hex.push('0');
    }
    ApInt::parse_hex(bits, &hex, false).unwrap()
}

/// The product of two `i128`s, and whether it overflows unsigned: from the
/// 64-bit halves (a = ah:al, b = bh:bl), it does if both high halves are
/// nonzero, if either cross product (ah*bl, al*bh) needs more than 64
/// bits, or if the middle column (their low halves, plus the carry from
/// al*bl) carries out.
fn umul128_overflow(cx: &mut Ctx, s: &mut Seq, a: Value, b2: Value, ty: TypeId) -> (Value, Value) {
    let i1 = cx.m.types.int(1);
    let cmp = |s: &mut Seq, pred, l, r| s.op(Op::ICmp { pred, samesign: false, lhs: l, rhs: r }, i1);
    let i64t = cx.m.types.int(64);
    let c64 = cx.int(ty, 64);
    let lo64 = |s: &mut Seq, x: Value| s.cast(CastOp::Trunc, x, i64t);
    let hi64 = |s: &mut Seq, x: Value| {
        let h = s.bin(BinOp::LShr, x, c64, ty);
        s.cast(CastOp::Trunc, h, i64t)
    };
    let (al, ah, bl, bh) = (lo64(s, a), hi64(s, a), lo64(s, b2), hi64(s, b2));
    let wide = |s: &mut Seq, x: Value, y: Value| {
        let (wx, wy) = (s.cast(CastOp::ZExt, x, ty), s.cast(CastOp::ZExt, y, ty));
        s.bin(BinOp::Mul, wx, wy, ty)
    };
    let (p0, p1, p2) = (wide(s, al, bl), wide(s, ah, bl), wide(s, al, bh));
    let z64 = cx.int(i64t, 0);
    let ah_nz = cmp(s, ICmpPred::Ne, ah, z64);
    let bh_nz = cmp(s, ICmpPred::Ne, bh, z64);
    let mut ov = s.bin(BinOp::And, ah_nz, bh_nz, i1);
    let (h1, h2) = (hi64(s, p1), hi64(s, p2));
    for h in [h1, h2] {
        let nz = cmp(s, ICmpPred::Ne, h, z64);
        ov = s.bin(BinOp::Or, ov, nz, i1);
    }
    let (l1, l2) = (lo64(s, p1), lo64(s, p2));
    let (w1, w2) = (s.cast(CastOp::ZExt, l1, ty), s.cast(CastOp::ZExt, l2, ty));
    let carry = s.bin(BinOp::LShr, p0, c64, ty);
    let mid = s.bin(BinOp::Add, w1, w2, ty);
    let mid = s.bin(BinOp::Add, mid, carry, ty);
    let out = hi64(s, mid);
    let nz = cmp(s, ICmpPred::Ne, out, z64);
    ov = s.bin(BinOp::Or, ov, nz, i1);
    (s.bin(BinOp::Mul, a, b2, ty), ov)
}

/// Intrinsics of 128-bit integers that `wide.rs` has no halves for,
/// written as `i128` operations it does: three-way comparisons, and
/// saturating additions and subtractions.
fn wide_intrinsics(cx: &mut Ctx, f: &mut Function) {
    let i1 = cx.m.types.int(1);
    let blocks: Vec<BlockId> = f.layout.clone();
    for b in blocks {
        let mut k = 0;
        while k < f.blocks[b.index()].insts.len() {
            let i = f.blocks[b.index()].insts[k];
            let Op::Call(call) = &f.insts[i.index()].op else {
                k += 1;
                continue;
            };
            let name = callee_name(cx.m, call).unwrap_or_default();
            let args: Vec<Value> = call.args.iter().map(|a| a.value).collect();
            let (Some(r), Some(&a0)) = (f.insts[i.index()].result, args.first()) else {
                k += 1;
                continue;
            };
            let aty = cx.value_ty(f, a0);
            let rty = f.locals[r.index()].ty;
            // A saturating conversion to a wide integer, as conversions and
            // selects of it, which wide.rs splits.
            if (name.starts_with("llvm.fptosi.sat.") || name.starts_with("llvm.fptoui.sat."))
                && let Type::Int(bits) = *cx.m.types.get(rty)
                && bits > 64
            {
                let mut s = Seq { f, new: Vec::new() };
                let v = saturating_convert(cx, &mut s, a0, rty, bits, name.starts_with("llvm.fptosi"));
                s.f.replace_uses(Value::Local(r), v);
                let new = std::mem::take(&mut s.new);
                let n = new.len();
                f.blocks[b.index()].insts.splice(k..=k, new);
                k += n;
                continue;
            }
            if !matches!(cx.m.types.get(aty), Type::Int(b) if *b > 64) {
                k += 1;
                continue;
            }
            // `llvm.scmp`, `llvm.usub.sat`: the name without its types.
            let is_type =
                |p: &&str| p.strip_prefix('i').is_some_and(|d| !d.is_empty() && d.bytes().all(|c| c.is_ascii_digit()));
            let base = name.split('.').take_while(|p| !is_type(p)).collect::<Vec<_>>().join(".");
            let mut s = Seq { f, new: Vec::new() };
            let cmp = |s: &mut Seq, pred, l, r| s.op(Op::ICmp { pred, samesign: false, lhs: l, rhs: r }, i1);
            let sel = |s: &mut Seq, c, t, e, ty| {
                s.op(Op::Select { fmf: FastMath::default(), cond: c, then_value: t, else_value: e }, ty)
            };
            let (a, bv) = (a0, args.get(1).copied().unwrap_or(a0));
            let v = match base.as_str() {
                // (a > b) - (a < b)
                "llvm.scmp" | "llvm.ucmp" => {
                    let (gt, lt) = if base == "llvm.scmp" {
                        (ICmpPred::Sgt, ICmpPred::Slt)
                    } else {
                        (ICmpPred::Ugt, ICmpPred::Ult)
                    };
                    let g = cmp(&mut s, gt, a, bv);
                    let l = cmp(&mut s, lt, a, bv);
                    let g = s.cast(CastOp::ZExt, g, rty);
                    let l = s.cast(CastOp::ZExt, l, rty);
                    s.bin(BinOp::Sub, g, l, rty)
                }
                "llvm.usub.sat" => {
                    let d = s.bin(BinOp::Sub, a, bv, aty);
                    let ok = cmp(&mut s, ICmpPred::Uge, a, bv);
                    let zero = cx.int(aty, 0);
                    sel(&mut s, ok, d, zero, aty)
                }
                "llvm.uadd.sat" => {
                    let t = s.bin(BinOp::Add, a, bv, aty);
                    let wrapped = cmp(&mut s, ICmpPred::Ult, t, a);
                    let ones = cx.int(aty, -1);
                    sel(&mut s, wrapped, ones, t, aty)
                }
                // Overflowed if the result's sign is not what the operands
                // say; then the limit on the side of a's sign.
                "llvm.sadd.sat" | "llvm.ssub.sat" => {
                    let add = base == "llvm.sadd.sat";
                    let t = s.bin(if add { BinOp::Add } else { BinOp::Sub }, a, bv, aty);
                    let x = s.bin(BinOp::Xor, a, t, aty);
                    let y = if add { s.bin(BinOp::Xor, bv, t, aty) } else { s.bin(BinOp::Xor, a, bv, aty) };
                    let both = s.bin(BinOp::And, x, y, aty);
                    let zero = cx.int(aty, 0);
                    let ov = cmp(&mut s, ICmpPred::Slt, both, zero);
                    let bits = match cx.m.types.get(aty) {
                        Type::Int(b) => *b,
                        _ => 128,
                    };
                    let max =
                        Value::Const(cx.m.consts.intern(Constant { ty: aty, kind: ConstKind::Int(signed_max(bits)) }));
                    let neg = cmp(&mut s, ICmpPred::Slt, a, zero);
                    let min = s.bin(BinOp::Xor, max, cx.int(aty, -1), aty);
                    let limit = sel(&mut s, neg, min, max, aty);
                    sel(&mut s, ov, limit, t, aty)
                }
                _ => {
                    k += 1;
                    continue;
                }
            };
            s.f.replace_uses(Value::Local(r), v);
            let new = std::mem::take(&mut s.new);
            let n = new.len();
            f.blocks[b.index()].insts.splice(k..=k, new);
            k += n;
        }
    }
}

fn expand_intrinsics(cx: &mut Ctx, f: &mut Function) {
    let blocks: Vec<BlockId> = f.layout.clone();
    for b in blocks {
        let mut k = 0;
        while k < f.blocks[b.index()].insts.len() {
            let i = f.blocks[b.index()].insts[k];
            let Op::Call(call) = &f.insts[i.index()].op else {
                k += 1;
                continue;
            };
            let Some(name) = callee_name(cx.m, call) else {
                k += 1;
                continue;
            };
            let args: Vec<Value> = call.args.iter().map(|a| a.value).collect();
            let Some(r) = f.insts[i.index()].result else {
                k += 1;
                continue;
            };
            let ty = f.locals[r.index()].ty;
            // Scalars only: vector intrinsics have instructions.
            let bits = match cx.m.types.get(ty) {
                Type::Int(b) => *b,
                _ => 64,
            };
            let scalar = matches!(cx.m.types.get(ty), Type::Int(_));
            let i1 = cx.m.types.int(1);
            let mut s = Seq { f, new: Vec::new() };
            let base = name.split('.').take(2).collect::<Vec<_>>().join(".");
            let minmax = match base.as_str() {
                "llvm.smax" => Some(ICmpPred::Sgt),
                "llvm.smin" => Some(ICmpPred::Slt),
                "llvm.umax" => Some(ICmpPred::Ugt),
                "llvm.umin" => Some(ICmpPred::Ult),
                _ => None,
            };
            // An ordered floating-point reduction: the start value, then
            // each lane added (or multiplied) in order, as written.
            if (name.starts_with("llvm.vector.reduce.fadd.") || name.starts_with("llvm.vector.reduce.fmul."))
                && let Type::Vector { len, elem, .. } = cx.m.types.get(cx.value_ty(s.f, args[1])).clone()
            {
                let i64t = cx.m.types.int(64);
                let op = if name.contains(".fadd.") { BinOp::FAdd } else { BinOp::FMul };
                let mut acc = args[0];
                for lane in 0..len {
                    let idx = cx.int(i64t, lane as i64);
                    let e = s.op(Op::ExtractElement { vector: args[1], index: idx }, elem);
                    acc =
                        s.op(Op::Binary { op, flags: IntFlags::NONE, fmf: FastMath::default(), lhs: acc, rhs: e }, ty);
                }
                let new = std::mem::take(&mut s.new);
                f.replace_uses(Value::Local(r), acc);
                let n = new.len();
                f.blocks[b.index()].insts.splice(k..=k, new);
                k += n;
                continue;
            }
            // A relative lookup table's entry (as clang makes them for x86):
            // the table's address plus the 32-bit offset at `offset` in it.
            if name.starts_with("llvm.load.relative.") {
                let (i8t, i32t, i64t) = (cx.m.types.int(8), cx.m.types.int(32), cx.m.types.int(64));
                let gep = |s: &mut Seq, p: Value, k: Value| {
                    s.op(
                        Op::Gep { flags: Default::default(), inrange: None, source_ty: i8t, ptr: p, indices: vec![k] },
                        ty,
                    )
                };
                let at = gep(&mut s, args[0], args[1]);
                let rel = s.op(Op::Load { volatile: false, atomic: None, ptr: at, align: Some(4) }, i32t);
                let rel = s.cast(CastOp::SExt, rel, i64t);
                let v = gep(&mut s, args[0], rel);
                let new = std::mem::take(&mut s.new);
                f.replace_uses(Value::Local(r), v);
                let n = new.len();
                f.blocks[b.index()].insts.splice(k..=k, new);
                k += n;
                continue;
            }
            // A pointer with some bits cleared: of its integer.
            if name.starts_with("llvm.ptrmask.") && matches!(cx.m.types.get(ty), Type::Ptr(_)) {
                let mty = cx.value_ty(s.f, args[1]);
                let p = s.cast(CastOp::PtrToInt, args[0], mty);
                let masked = s.bin(BinOp::And, p, args[1], mty);
                let v = s.cast(CastOp::IntToPtr, masked, ty);
                let new = std::mem::take(&mut s.new);
                f.replace_uses(Value::Local(r), v);
                let n = new.len();
                f.blocks[b.index()].insts.splice(k..=k, new);
                k += n;
                continue;
            }
            // A funnel shift by a constant, of vectors or (where the
            // description has no instruction for it) of scalars: two shifts
            // and an or. By a variable amount, which no instruction takes,
            // the same with the amount masked, and the other shift done in
            // two steps so that it is never by the full width.
            let vector = matches!(cx.m.types.get(ty), Type::Vector { .. });
            if base == "llvm.fshl" || base == "llvm.fshr" {
                let elem = match cx.m.types.get(ty).clone() {
                    Type::Vector { elem, .. } => elem,
                    _ => ty,
                };
                let eb = match cx.m.types.get(elem) {
                    Type::Int(b) => *b as i64,
                    _ => 64,
                };
                let amount = |cx: &mut Ctx, v: i64| -> Value {
                    let e = cx.int(elem, v);
                    if !vector {
                        return e;
                    }
                    let Value::Const(e) = e else { unreachable!() };
                    Value::Const(cx.m.consts.intern(Constant { ty, kind: ConstKind::Splat(e) }))
                };
                let constant = match args[2] {
                    Value::Const(c) => crate::target::lit_of(cx.m, c),
                    _ => None,
                };
                // A rotation by a variable amount, where the description
                // has one (the role "rotr.iN"): right as it is, left as
                // right by the negated amount.
                let rotate =
                    !vector && constant.is_none() && args[0] == args[1] && cx.t.role(&format!("rotr.i{eb}")).is_some();
                if rotate && base == "llvm.fshl" {
                    let zero = cx.int(ty, 0);
                    let neg = s.bin(BinOp::Sub, zero, args[2], ty);
                    let v = call_intrinsic(cx, &mut s, &format!("llvm.fshr.i{eb}"), vec![args[0], args[0], neg], ty);
                    let new = std::mem::take(&mut s.new);
                    f.replace_uses(Value::Local(r), v);
                    let n = new.len();
                    f.blocks[b.index()].insts.splice(k..=k, new);
                    k += n;
                    continue;
                }
                let value = match constant {
                    _ if rotate => None,
                    Some(_) if !vector && cx.selects_intrinsic(&name) => None,
                    Some(c) => {
                        // fshl(a, b, c) is (a << c) | (b >> (w - c)); fshr(a, b, c)
                        // is fshl(a, b, w - c).
                        let amt = c.0.rem_euclid(eb);
                        let amt = if base == "llvm.fshr" { (eb - amt) % eb } else { amt };
                        Some(if amt == 0 {
                            if base == "llvm.fshr" { args[1] } else { args[0] }
                        } else {
                            let (a, b) = (amount(cx, amt), amount(cx, eb - amt));
                            let hi = s.bin(BinOp::Shl, args[0], a, ty);
                            let lo = s.bin(BinOp::LShr, args[1], b, ty);
                            s.bin(BinOp::Or, hi, lo, ty)
                        })
                    }
                    None => {
                        // fshl: (a << c) | ((b >> 1) >> (w - 1 - c));
                        // fshr: ((a << 1) << (w - 1 - c)) | (b >> c); c mod w.
                        let mask = amount(cx, eb - 1);
                        let one = amount(cx, 1);
                        let c = s.bin(BinOp::And, args[2], mask, ty);
                        let rest = s.bin(BinOp::Xor, c, mask, ty);
                        Some(if base == "llvm.fshl" {
                            let hi = s.bin(BinOp::Shl, args[0], c, ty);
                            let lo = s.bin(BinOp::LShr, args[1], one, ty);
                            let lo = s.bin(BinOp::LShr, lo, rest, ty);
                            s.bin(BinOp::Or, hi, lo, ty)
                        } else {
                            let hi = s.bin(BinOp::Shl, args[0], one, ty);
                            let hi = s.bin(BinOp::Shl, hi, rest, ty);
                            let lo = s.bin(BinOp::LShr, args[1], c, ty);
                            s.bin(BinOp::Or, hi, lo, ty)
                        })
                    }
                };
                if let Some(value) = value {
                    let new = std::mem::take(&mut s.new);
                    f.replace_uses(Value::Local(r), value);
                    let n = new.len();
                    f.blocks[b.index()].insts.splice(k..=k, new);
                    k += n;
                    continue;
                }
            }
            // ctlz and cttz may say a zero input gives poison; giving the
            // full width instead is always allowed, and is what clz does.
            if (base == "llvm.ctlz" || base == "llvm.cttz") && args.len() == 2 {
                let no = cx.int(i1, 0);
                if let Op::Call(c) = &mut s.f.insts[i.index()].op {
                    c.args[1].value = no;
                }
            }
            // The classes of a float (`llvm.is.fpclass`), from its bits: the
            // magnitude against the exponent's limits, the quiet bit, the
            // sign. Mask bits 0-9: signalling NaN, quiet NaN, -inf,
            // -normal, -subnormal, -0, +0, +subnormal, +normal, +inf.
            if name.starts_with("llvm.is.fpclass.") && scalar_float(cx, cx.value_ty(s.f, args[0])) {
                let fty = cx.value_ty(s.f, args[0]);
                let double = matches!(cx.m.types.get(fty), Type::Double);
                let (bits, exp, quiet, min_normal): (u32, i64, i64, i64) = if double {
                    (64, 0x7ff0_0000_0000_0000, 0x0008_0000_0000_0000, 0x0010_0000_0000_0000)
                } else {
                    (32, 0x7f80_0000, 0x0040_0000, 0x0080_0000)
                };
                let it = cx.m.types.int(bits);
                let mask = cx.const_int(args[1]).unwrap_or(0);
                let x = s.cast(CastOp::BitCast, args[0], it);
                let c = |cx: &mut Ctx, v: i64| cx.int(it, v);
                let all = c(cx, if double { i64::MAX } else { 0x7fff_ffff });
                let abs = s.bin(BinOp::And, x, all, it);
                let cmp = |s: &mut Seq, p, l, r| s.op(Op::ICmp { pred: p, samesign: false, lhs: l, rhs: r }, i1);
                let and1 = |s: &mut Seq, a, b| s.bin(BinOp::And, a, b, i1);
                let zero = c(cx, 0);
                let neg = cmp(&mut s, ICmpPred::Slt, x, zero);
                let t = cx.int(i1, 1);
                let pos = s.bin(BinOp::Xor, neg, t, i1);
                let e = c(cx, exp);
                let q = c(cx, quiet);
                let mn = c(cx, min_normal);
                let nan = cmp(&mut s, ICmpPred::Ugt, abs, e);
                let qbit = s.bin(BinOp::And, abs, q, it);
                let is_q = cmp(&mut s, ICmpPred::Ne, qbit, zero);
                let inf = cmp(&mut s, ICmpPred::Eq, abs, e);
                let z = cmp(&mut s, ICmpPred::Eq, abs, zero);
                let below = cmp(&mut s, ICmpPred::Ult, abs, mn);
                let nz = s.bin(BinOp::Xor, z, t, i1);
                let sub = and1(&mut s, below, nz);
                let from = cmp(&mut s, ICmpPred::Uge, abs, mn);
                let under = cmp(&mut s, ICmpPred::Ult, abs, e);
                let normal = and1(&mut s, from, under);
                let not_q = s.bin(BinOp::Xor, is_q, t, i1);
                let classes: [Value; 10] = [
                    and1(&mut s, nan, not_q),
                    and1(&mut s, nan, is_q),
                    and1(&mut s, inf, neg),
                    and1(&mut s, normal, neg),
                    and1(&mut s, sub, neg),
                    and1(&mut s, z, neg),
                    and1(&mut s, z, pos),
                    and1(&mut s, sub, pos),
                    and1(&mut s, normal, pos),
                    and1(&mut s, inf, pos),
                ];
                let mut v = cx.int(i1, 0);
                for (k, &cl) in classes.iter().enumerate() {
                    if mask & (1 << k) != 0 {
                        v = s.bin(BinOp::Or, v, cl, i1);
                    }
                }
                let new = std::mem::take(&mut s.new);
                f.replace_uses(Value::Local(r), v);
                let n = new.len();
                f.blocks[b.index()].insts.splice(k..=k, new);
                k += n;
                continue;
            }
            // Floating-point maximum and minimum across lanes, without an
            // instruction: the scalar ones, lane by lane.
            if let Some(op) = ["fmax", "fmin", "fmaximum", "fminimum"]
                .into_iter()
                .find(|o| name.starts_with(&format!("llvm.vector.reduce.{o}.")))
                && !cx.selects_intrinsic(&name)
                && let Type::Vector { len, elem, scalable: false } = *cx.m.types.get(cx.value_ty(s.f, args[0]))
            {
                let scalar = match op {
                    "fmax" => "maxnum",
                    "fmin" => "minnum",
                    "fmaximum" => "maximum",
                    _ => "minimum",
                };
                let suf = if matches!(cx.m.types.get(elem), Type::Float) { "f32" } else { "f64" };
                let i64t = cx.m.types.int(64);
                let lane = |cx: &mut Ctx, s: &mut Seq, l: u32| {
                    let idx = cx.int(i64t, i64::from(l));
                    s.op(Op::ExtractElement { vector: args[0], index: idx }, elem)
                };
                let mut acc = lane(cx, &mut s, 0);
                for l in 1..len {
                    let x = lane(cx, &mut s, l);
                    acc = call_intrinsic(cx, &mut s, &format!("llvm.{scalar}.{suf}"), vec![acc, x], elem);
                }
                let new = std::mem::take(&mut s.new);
                f.replace_uses(Value::Local(r), acc);
                let n = new.len();
                f.blocks[b.index()].insts.splice(k..=k, new);
                k += n;
                continue;
            }
            // Whether any (or every) lane of a mask is set: the unsigned
            // maximum (minimum) of its lanes as integers, all ones or zeros.
            if (name.starts_with("llvm.vector.reduce.or.") || name.starts_with("llvm.vector.reduce.and."))
                && let Type::Vector { len, elem, scalable: false } = *cx.m.types.get(cx.value_ty(s.f, args[0]))
                && elem == i1
                && len.is_power_of_two()
                && len > 1
            {
                let w = (128 / len).clamp(8, 64);
                let e = cx.m.types.int(w);
                let it = cx.m.types.intern(Type::Vector { len, elem: e, scalable: false });
                let wide = s.cast(CastOp::SExt, args[0], it);
                let red = if name.contains(".or.") { "umax" } else { "umin" };
                let x = call_intrinsic(cx, &mut s, &format!("llvm.vector.reduce.{red}.v{len}i{w}"), vec![wide], e);
                let zero = cx.int(e, 0);
                let v = s.op(Op::ICmp { pred: ICmpPred::Ne, samesign: false, lhs: x, rhs: zero }, i1);
                let new = std::mem::take(&mut s.new);
                f.replace_uses(Value::Local(r), v);
                let n = new.len();
                f.blocks[b.index()].insts.splice(k..=k, new);
                k += n;
                continue;
            }
            // The first set lane of a mask: of the mask as an integer, its
            // trailing zeros (all the lanes, if none is set).
            if name.starts_with("llvm.experimental.cttz.elts.")
                && let Type::Vector { len, .. } = *cx.m.types.get(cx.value_ty(s.f, args[0]))
            {
                let it = cx.m.types.int(len);
                let bits = s.cast(CastOp::BitCast, args[0], it);
                let no = cx.int(i1, 0);
                let n = call_intrinsic(cx, &mut s, &format!("llvm.cttz.i{len}"), vec![bits, no], it);
                let rb = match cx.m.types.get(ty) {
                    Type::Int(b) => *b,
                    _ => 64,
                };
                let v = match rb.cmp(&len) {
                    std::cmp::Ordering::Greater => s.cast(CastOp::ZExt, n, ty),
                    std::cmp::Ordering::Less => s.cast(CastOp::Trunc, n, ty),
                    std::cmp::Ordering::Equal => n,
                };
                let new = std::mem::take(&mut s.new);
                f.replace_uses(Value::Local(r), v);
                let n = new.len();
                f.blocks[b.index()].insts.splice(k..=k, new);
                k += n;
                continue;
            }
            // Mathematical functions without an instruction: the C
            // library's (lane by lane, for vectors), as LLVM calls them.
            // (A vector the description has at another width, as AArch64's
            // `<4 x float>` for an `<8 x float>`, is left: legalisation makes
            // it that width.)
            if let Some(c) = libm_base(&base)
                && !cx.selects_intrinsic(&name)
                && !selects_at_some_width(cx, &name)
                && let Some(lib) = libm_name(cx, c, ty)
            {
                let v = lanewise_call(cx, &mut s, &lib, args.clone(), ty);
                let new = std::mem::take(&mut s.new);
                f.replace_uses(Value::Local(r), v);
                let n = new.len();
                f.blocks[b.index()].insts.splice(k..=k, new);
                k += n;
                continue;
            }
            // The sign of one, the rest of the other: as integers.
            if base == "llvm.copysign" && !cx.selects_intrinsic(&name) {
                let (ity, bits) = match cx.m.types.get(ty).clone() {
                    Type::Vector { len, elem, scalable: false } => {
                        let b = cx.size_of(elem).unwrap_or(8) as u32 * 8;
                        let e = cx.m.types.int(b);
                        (cx.m.types.intern(Type::Vector { len, elem: e, scalable: false }), b)
                    }
                    _ => {
                        let b = cx.size_of(ty).unwrap_or(8) as u32 * 8;
                        (cx.m.types.int(b), b)
                    }
                };
                let elem_int = match cx.m.types.get(ity) {
                    Type::Vector { elem, .. } => *elem,
                    _ => ity,
                };
                let sign_c = cx
                    .m
                    .consts
                    .intern(Constant { ty: elem_int, kind: ConstKind::Int(ApInt::from_u64(bits, 1u64 << (bits - 1))) });
                let rest_c = cx.m.consts.intern(Constant {
                    ty: elem_int,
                    kind: ConstKind::Int(ApInt::from_u64(bits, (1u64 << (bits - 1)) - 1)),
                });
                let splat = |cx: &mut Ctx, c: ConstId| {
                    if ity == elem_int {
                        Value::Const(c)
                    } else {
                        Value::Const(cx.m.consts.intern(Constant { ty: ity, kind: ConstKind::Splat(c) }))
                    }
                };
                let (sign, rest) = (splat(cx, sign_c), splat(cx, rest_c));
                let a = s.cast(CastOp::BitCast, args[0], ity);
                let b2 = s.cast(CastOp::BitCast, args[1], ity);
                let a = s.bin(BinOp::And, a, rest, ity);
                let b2 = s.bin(BinOp::And, b2, sign, ity);
                let v = s.bin(BinOp::Or, a, b2, ity);
                let v = s.cast(CastOp::BitCast, v, ty);
                let new = std::mem::take(&mut s.new);
                f.replace_uses(Value::Local(r), v);
                let n = new.len();
                f.blocks[b.index()].insts.splice(k..=k, new);
                k += n;
                continue;
            }
            // Saturating conversions of vectors: lane by lane, by the
            // scalar one (`llvm.fptosi.sat.v2i32.v2f64` by `.i32.f64`).
            if (base == "llvm.fptosi" || base == "llvm.fptoui")
                && name.contains(".sat.")
                && !scalar
                && matches!(cx.m.types.get(ty), Type::Vector { .. })
                && !cx.selects_intrinsic(&name)
            {
                let lane_name: String = name
                    .split('.')
                    .map(|p| match p.strip_prefix('v') {
                        Some(rest) if rest.starts_with(|c: char| c.is_ascii_digit()) => {
                            rest.trim_start_matches(|c: char| c.is_ascii_digit()).to_string()
                        }
                        _ => p.to_string(),
                    })
                    .collect::<Vec<_>>()
                    .join(".");
                let v = lanewise_call(cx, &mut s, &lane_name, args.clone(), ty);
                let new = std::mem::take(&mut s.new);
                f.replace_uses(Value::Local(r), v);
                let n = new.len();
                f.blocks[b.index()].insts.splice(k..=k, new);
                k += n;
                continue;
            }
            // Saturating conversions without an instruction: the limits,
            // NaN as 0, else the plain conversion (defined there).
            if (base == "llvm.fptosi" || base == "llvm.fptoui")
                && name.contains(".sat.")
                && scalar
                && !cx.selects_intrinsic(&name)
            {
                let v = saturating_convert(cx, &mut s, args[0], ty, bits, base == "llvm.fptosi");
                let new = std::mem::take(&mut s.new);
                f.replace_uses(Value::Local(r), v);
                let n = new.len();
                f.blocks[b.index()].insts.splice(k..=k, new);
                k += n;
                continue;
            }
            // Bit counts and byte swaps of vectors without an instruction
            // (NEON counts only bytes' bits): lane by lane.
            if !scalar
                && matches!(base.as_str(), "llvm.ctpop" | "llvm.ctlz" | "llvm.cttz" | "llvm.bswap" | "llvm.bitreverse")
                && !cx.selects_intrinsic(&name)
                && let Type::Vector { len, elem, scalable: false } = *cx.m.types.get(ty)
                && let Type::Int(eb) = *cx.m.types.get(elem)
            {
                let i64t = cx.m.types.int(64);
                let mut v = Value::Const(cx.m.consts.intern(Constant { ty, kind: ConstKind::Poison }));
                for lane in 0..len {
                    let idx = cx.int(i64t, lane as i64);
                    let x = s.op(Op::ExtractElement { vector: args[0], index: idx }, elem);
                    let mut a = vec![x];
                    a.extend(args.iter().skip(1).copied());
                    let y = call_intrinsic(cx, &mut s, &format!("{base}.i{eb}"), a, elem);
                    v = s.op(Op::InsertElement { vector: v, element: y, index: idx }, ty);
                }
                let new = std::mem::take(&mut s.new);
                f.replace_uses(Value::Local(r), v);
                let n = new.len();
                f.blocks[b.index()].insts.splice(k..=k, new);
                k += n;
                continue;
            }
            // Any other intrinsic of floating-point vectors (of up to a
            // register's width; wider ones are split first) without an
            // instruction: lane by lane, by its scalar form
            // (`llvm.exp.v2f64` by `.f64`). (Integer ones are expanded
            // later, as vectors: compare and select for `umin`.)
            if !name.starts_with("llvm.vector.")
                && !name.starts_with("llvm.experimental.")
                && !name.starts_with("llvm.masked.")
                && !name.starts_with("llvm.aarch64.")
                && let Type::Vector { len, elem, scalable: false } = *cx.m.types.get(ty)
                && len > 1
                && matches!(cx.m.types.get(elem), Type::Float | Type::Double)
                && cx.size_of(ty).is_ok_and(|b| b <= 16)
                && !cx.selects_intrinsic(&name)
            {
                let lane_name: String = name
                    .split('.')
                    .map(|p| match p.strip_prefix('v') {
                        Some(rest) if rest.starts_with(|c: char| c.is_ascii_digit()) => {
                            rest.trim_start_matches(|c: char| c.is_ascii_digit()).to_string()
                        }
                        _ => p.to_string(),
                    })
                    .collect::<Vec<_>>()
                    .join(".");
                let v = lanewise_call(cx, &mut s, &lane_name, args.clone(), ty);
                let new = std::mem::take(&mut s.new);
                f.replace_uses(Value::Local(r), v);
                let n = new.len();
                f.blocks[b.index()].insts.splice(k..=k, new);
                k += n;
                continue;
            }
            // Bit counts, byte swaps and bit reversals without an
            // instruction: widened to a width that has one, or done with
            // shifts and masks.
            if scalar
                && matches!(base.as_str(), "llvm.ctpop" | "llvm.ctlz" | "llvm.cttz" | "llvm.bswap" | "llvm.bitreverse")
                && !cx.selects_intrinsic(&name)
            {
                let value = bit_intrinsic(cx, &mut s, &base, args[0], bits, ty);
                let new = std::mem::take(&mut s.new);
                f.replace_uses(Value::Local(r), value);
                let n = new.len();
                f.blocks[b.index()].insts.splice(k..=k, new);
                k += n;
                continue;
            }
            let value = match (minmax, base.as_str()) {
                // Narrow minimum and maximum: compare and select.
                (Some(pred), _) if scalar && bits < 32 => {
                    let c = s.op(Op::ICmp { pred, samesign: false, lhs: args[0], rhs: args[1] }, i1);
                    Some(s.op(
                        Op::Select { fmf: FastMath::default(), cond: c, then_value: args[0], else_value: args[1] },
                        ty,
                    ))
                }
                // Vector minimum and maximum without an instruction (NEON's
                // of 64-bit lanes): compare and select, lane by lane.
                (Some(pred), _) if !scalar && !cx.selects_intrinsic(&name) => {
                    let Type::Vector { len, scalable: false, .. } = *cx.m.types.get(ty) else { unreachable!() };
                    let mask = cx.m.types.intern(Type::Vector { len, elem: i1, scalable: false });
                    let c = s.op(Op::ICmp { pred, samesign: false, lhs: args[0], rhs: args[1] }, mask);
                    Some(s.op(
                        Op::Select { fmf: FastMath::default(), cond: c, then_value: args[0], else_value: args[1] },
                        ty,
                    ))
                }
                // The size of the object a pointer points into, when it is
                // not known here: "unknown" is -1, or 0 if a lower bound
                // is asked for. (An optimiser folds the known cases.)
                (_, "llvm.objectsize") => {
                    let min = cx.const_int(args[1]).unwrap_or(0) != 0;
                    Some(cx.int(ty, if min { 0 } else { -1 }))
                }
                // Three-way comparison (Rust's `Ord::cmp`): 1, 0 or -1, as
                // (a > b) - (a < b).
                (_, "llvm.ucmp" | "llvm.scmp") if scalar => {
                    let signed = base == "llvm.scmp";
                    let (gt, lt) = if signed { (ICmpPred::Sgt, ICmpPred::Slt) } else { (ICmpPred::Ugt, ICmpPred::Ult) };
                    let g = s.op(Op::ICmp { pred: gt, samesign: false, lhs: args[0], rhs: args[1] }, i1);
                    let l = s.op(Op::ICmp { pred: lt, samesign: false, lhs: args[0], rhs: args[1] }, i1);
                    let (g, l) = (s.cast(CastOp::ZExt, g, ty), s.cast(CastOp::ZExt, l, ty));
                    Some(s.bin(BinOp::Sub, g, l, ty))
                }
                // Whether a value is known to be constant here: "no" is
                // always an allowed answer. (An optimiser folds the cases
                // it can see.)
                (_, "llvm.is") if name.starts_with("llvm.is.constant") => Some(cx.int(ty, 0)),
                // Saturating subtraction: a > b ? a - b : 0.
                (_, "llvm.usub") if scalar && name.starts_with("llvm.usub.sat") => {
                    let zero = cx.int(ty, 0);
                    let diff = s.bin(BinOp::Sub, args[0], args[1], ty);
                    let c = s.op(Op::ICmp { pred: ICmpPred::Ugt, samesign: false, lhs: args[0], rhs: args[1] }, i1);
                    Some(s.op(Op::Select { fmf: FastMath::default(), cond: c, then_value: diff, else_value: zero }, ty))
                }
                // Saturating addition: the sum, or all ones if it wrapped
                // (came out below an operand).
                (_, "llvm.uadd") if scalar && name.starts_with("llvm.uadd.sat") => {
                    let ones = cx.int(ty, -1);
                    let sum = s.bin(BinOp::Add, args[0], args[1], ty);
                    let c = s.op(Op::ICmp { pred: ICmpPred::Ult, samesign: false, lhs: sum, rhs: args[0] }, i1);
                    Some(s.op(Op::Select { fmf: FastMath::default(), cond: c, then_value: ones, else_value: sum }, ty))
                }
                // Signed saturation: on overflow, the most negative value if
                // a is negative, else the most positive (a's sign bit
                // spread, xor the most positive). Addition overflows when
                // the result's sign differs from both operands'; subtraction
                // when the operands' signs differ and the result's differs
                // from a's.
                (_, "llvm.sadd" | "llvm.ssub")
                    if scalar
                        && bits <= 64
                        && (name.starts_with("llvm.sadd.sat") || name.starts_with("llvm.ssub.sat")) =>
                {
                    let add = base == "llvm.sadd";
                    let (a, b) = (args[0], args[1]);
                    let r = s.bin(if add { BinOp::Add } else { BinOp::Sub }, a, b, ty);
                    let ar = s.bin(BinOp::Xor, a, r, ty);
                    let other = if add { s.bin(BinOp::Xor, b, r, ty) } else { s.bin(BinOp::Xor, a, b, ty) };
                    let both = s.bin(BinOp::And, ar, other, ty);
                    let zero = cx.int(ty, 0);
                    let over = s.op(Op::ICmp { pred: ICmpPred::Slt, samesign: false, lhs: both, rhs: zero }, i1);
                    let top = cx.int(ty, bits as i64 - 1);
                    let sign = s.bin(BinOp::AShr, a, top, ty);
                    let max = cx.int(ty, if bits >= 64 { i64::MAX } else { (1i64 << (bits - 1)) - 1 });
                    let sat = s.bin(BinOp::Xor, sign, max, ty);
                    Some(s.op(Op::Select { fmf: FastMath::default(), cond: over, then_value: sat, else_value: r }, ty))
                }
                // Vector abs keeps its call, for the abs instruction; its
                // flag (poison for the most negative value) may be dropped.
                (_, "llvm.abs")
                    if cx.value_ty(s.f, args[0]) != ty || matches!(cx.m.types.get(ty), Type::Vector { .. }) =>
                {
                    let no = cx.int(i1, 0);
                    if let Op::Call(c) = &mut s.f.insts[i.index()].op {
                        c.args[1].value = no;
                    }
                    None
                }
                (_, "llvm.abs") => {
                    let zero = cx.int(ty, 0);
                    let neg = s.bin(BinOp::Sub, zero, args[0], ty);
                    let c = s.op(Op::ICmp { pred: ICmpPred::Slt, samesign: false, lhs: args[0], rhs: zero }, i1);
                    Some(
                        s.op(
                            Op::Select { fmf: FastMath::default(), cond: c, then_value: neg, else_value: args[0] },
                            ty,
                        ),
                    )
                }
                _ => None,
            };
            let new = std::mem::take(&mut s.new);
            match value {
                Some(v) => {
                    f.replace_uses(Value::Local(r), v);
                    let n = new.len();
                    f.blocks[b.index()].insts.splice(k..=k, new);
                    k += n;
                }
                None => k += 1,
            }
        }
    }
}

/// `base` (`llvm.ctpop`, `llvm.ctlz`, `llvm.cttz` or `llvm.bswap`) of `x`,
/// an integer of `bits` bits, of type `ty`, where the description has no
/// instruction for it: by the same at a wider width it has one for, or
/// with shifts and masks. Zero gives the full width.
/// A call of intrinsic `name`, declared if new, returning `ret`.
pub(crate) fn call_intrinsic(cx: &mut Ctx, s: &mut Seq, name: &str, args: Vec<Value>, ret: TypeId) -> Value {
    let params: Vec<TypeId> = args.iter().map(|&a| cx.value_ty(s.f, a)).collect();
    let callee = crate::legalize::declare(cx, name, ret, &params);
    let fn_ty = cx.m.types.intern(Type::Function { ret, params, varargs: false });
    s.op(
        Op::Call(Box::new(Call {
            tail: TailKind::None,
            fmf: FastMath::default(),
            cconv: CallConv::default(),
            ret_attrs: vec![],
            addrspace: None,
            fn_ty,
            callee,
            args: args.into_iter().map(|value| CallArg { value, attrs: vec![] }).collect(),
            fn_attrs: vec![],
            bundles: vec![],
        })),
        ret,
    )
}

fn bit_intrinsic(cx: &mut Ctx, s: &mut Seq, base: &str, x: Value, bits: u32, ty: TypeId) -> Value {
    let i1 = cx.m.types.int(1);
    let wide = [16, 32, 64].into_iter().find(|&w| w > bits && cx.selects_intrinsic(&format!("{base}.i{w}")));
    let w = wide.unwrap_or(if bits <= 32 { 32 } else { 64 });
    let wt = cx.m.types.int(w);
    let c = |cx: &mut Ctx, v: i64| cx.int(wt, v);
    let mut x = if bits < w { s.cast(CastOp::ZExt, x, wt) } else { x };
    let narrow = |s: &mut Seq, v: Value| if bits < w { s.cast(CastOp::Trunc, v, ty) } else { v };
    // A zero input counts as having its lowest bit just above the value.
    if base == "llvm.cttz" && bits < w {
        let top = c(cx, 1i64 << bits);
        x = s.bin(BinOp::Or, x, top, wt);
    }
    // Trailing zeros as leading zeros of the bits reversed (rbit, clz),
    // when the description has both at this width.
    if base == "llvm.cttz"
        && cx.selects_intrinsic(&format!("llvm.bitreverse.i{w}"))
        && cx.selects_intrinsic(&format!("llvm.ctlz.i{w}"))
    {
        let rev = call_intrinsic(cx, s, &format!("llvm.bitreverse.i{w}"), vec![x], wt);
        let no = cx.int(i1, 0);
        let v = call_intrinsic(cx, s, &format!("llvm.ctlz.i{w}"), vec![rev, no], wt);
        return narrow(s, v);
    }
    if wide.is_some() {
        let name = format!("{base}.i{w}");
        let no = cx.int(i1, 0);
        let args = if base == "llvm.ctlz" || base == "llvm.cttz" { vec![x, no] } else { vec![x] };
        let params: Vec<TypeId> = args.iter().map(|a| if *a == no { i1 } else { wt }).collect();
        let callee = crate::legalize::declare(cx, &name, wt, &params);
        let fn_ty = cx.m.types.intern(Type::Function { ret: wt, params, varargs: false });
        let mut v = s.op(
            Op::Call(Box::new(Call {
                tail: TailKind::None,
                fmf: FastMath::default(),
                cconv: CallConv::default(),
                ret_attrs: vec![],
                addrspace: None,
                fn_ty,
                callee,
                args: args.into_iter().map(|value| CallArg { value, attrs: vec![] }).collect(),
                fn_attrs: vec![],
                bundles: vec![],
            })),
            wt,
        );
        if base == "llvm.ctlz" && bits < w {
            let d = c(cx, (w - bits) as i64);
            v = s.bin(BinOp::Sub, v, d, wt);
        }
        if (base == "llvm.bswap" || base == "llvm.bitreverse") && bits < w {
            let d = c(cx, (w - bits) as i64);
            v = s.bin(BinOp::LShr, v, d, wt);
        }
        return narrow(s, v);
    }
    let rep =
        |pattern: u64, every: u32| -> i64 { (0..w / every).fold(0u64, |acc, k| acc | (pattern << (every * k))) as i64 };
    let v = match base {
        "llvm.bswap" | "llvm.bitreverse" => {
            // Swap bytes (bits, for a reversal), then pairs of them, then
            // their pairs, to halves.
            let mut step = if base == "llvm.bswap" { 8 } else { 1 };
            while step < w {
                let m = c(cx, rep((1u64 << step) - 1, 2 * step));
                let sh = c(cx, step as i64);
                let lo = s.bin(BinOp::And, x, m, wt);
                let lo = s.bin(BinOp::Shl, lo, sh, wt);
                let hi = s.bin(BinOp::LShr, x, sh, wt);
                let hi = s.bin(BinOp::And, hi, m, wt);
                x = s.bin(BinOp::Or, hi, lo, wt);
                step *= 2;
            }
            if bits < w {
                let d = c(cx, (w - bits) as i64);
                x = s.bin(BinOp::LShr, x, d, wt);
            }
            x
        }
        "llvm.ctlz" => {
            // Every bit below the highest set, counted, from the width.
            let mut step = 1;
            while step < w {
                let sh = c(cx, step as i64);
                let y = s.bin(BinOp::LShr, x, sh, wt);
                x = s.bin(BinOp::Or, x, y, wt);
                step *= 2;
            }
            let n = popcount(cx, s, x, w);
            let full = c(cx, bits as i64);
            s.bin(BinOp::Sub, full, n, wt)
        }
        "llvm.cttz" => {
            // The bits below the lowest set: !x & (x - 1).
            let one = c(cx, 1);
            let all = c(cx, -1);
            let below = s.bin(BinOp::Sub, x, one, wt);
            let not = s.bin(BinOp::Xor, x, all, wt);
            let y = s.bin(BinOp::And, not, below, wt);
            popcount(cx, s, y, w)
        }
        _ => popcount(cx, s, x, w),
    };
    narrow(s, v)
}

/// The number of bits set in `x`, an integer of `w` (32 or 64) bits: sums
/// of bits in pairs, nibbles and bytes, then the bytes added up, with a
/// multiply if the machine has one.
fn popcount(cx: &mut Ctx, s: &mut Seq, x: Value, w: u32) -> Value {
    let wt = cx.m.types.int(w);
    let rep = |byte: u64| -> i64 { (0..w / 8).fold(0u64, |acc, k| acc | (byte << (8 * k))) as i64 };
    let c = |cx: &mut Ctx, v: i64| cx.int(wt, v);
    let one = c(cx, 1);
    let half = s.bin(BinOp::LShr, x, one, wt);
    let m1 = c(cx, rep(0x55));
    let half = s.bin(BinOp::And, half, m1, wt);
    let mut x = s.bin(BinOp::Sub, x, half, wt);
    let m2 = c(cx, rep(0x33));
    let lo = s.bin(BinOp::And, x, m2, wt);
    let two = c(cx, 2);
    let hi = s.bin(BinOp::LShr, x, two, wt);
    let hi = s.bin(BinOp::And, hi, m2, wt);
    x = s.bin(BinOp::Add, lo, hi, wt);
    let four = c(cx, 4);
    let sh = s.bin(BinOp::LShr, x, four, wt);
    x = s.bin(BinOp::Add, x, sh, wt);
    let m4 = c(cx, rep(0x0f));
    x = s.bin(BinOp::And, x, m4, wt);
    if cx.selects(|h| *h == Head::Bin(BinOp::Mul, w)) {
        let ones = c(cx, rep(0x01));
        x = s.bin(BinOp::Mul, x, ones, wt);
        let top = c(cx, w as i64 - 8);
        return s.bin(BinOp::LShr, x, top, wt);
    }
    let mut step = 8;
    while step < w {
        let sh = c(cx, step as i64);
        let y = s.bin(BinOp::LShr, x, sh, wt);
        x = s.bin(BinOp::Add, x, y, wt);
        step *= 2;
    }
    let m = c(cx, 0x7f);
    s.bin(BinOp::And, x, m, wt)
}

/// `p + off` as a pointer: `p` itself for 0.
fn offset_ptr(cx: &mut Ctx, s: &mut Seq, p: Value, off: u64, i8t: TypeId, i64t: TypeId, ptr: TypeId) -> Value {
    if off == 0 {
        p
    } else {
        let o = cx.int(i64t, off as i64);
        s.op(gep8(i8t, p, o), ptr)
    }
}

/// Bit casts between a vector of booleans and an integer of as many bits
/// (`bitcast <16 x i1> %m to i16`, a vector comparison's lanes as a mask),
/// lane by lane: the lanes extended to bytes, each extracted, extended and
/// shifted to its bit, and
/// the bits or'ed; or the other way, each bit shifted down, truncated and
/// inserted. Lane k is bit k. Run after `legalize::fix_masks`, which
/// makes the common `icmp eq (bitcast mask), 0` a reduction; casts left
/// unused by that are skipped.
pub(crate) fn bool_vector_casts(cx: &mut Ctx, f: &mut Function) {
    let i1 = cx.m.types.int(1);
    let i64t = cx.m.types.int(64);
    let mut used = std::collections::HashSet::new();
    for &b in &f.layout {
        for &i in &f.blocks[b.index()].insts {
            f.insts[i.index()].op.for_each_value(|v| {
                if let Value::Local(l) = v {
                    used.insert(*l);
                }
            });
        }
    }
    for b in f.layout.clone() {
        let mut k = 0;
        while k < f.blocks[b.index()].insts.len() {
            let i = f.blocks[b.index()].insts[k];
            let (Op::Cast { op: CastOp::BitCast, value, .. }, Some(r)) =
                (&f.insts[i.index()].op, f.insts[i.index()].result)
            else {
                k += 1;
                continue;
            };
            let value = *value;
            if !used.contains(&r) {
                k += 1;
                continue;
            }
            let (from, to) = (cx.value_ty(f, value), f.locals[r.index()].ty);
            let lanes_of = |cx: &Ctx, t: TypeId| match cx.m.types.get(t) {
                Type::Vector { len, elem, scalable: false } if *elem == i1 => Some(*len as u32),
                _ => None,
            };
            let bits_of = |cx: &Ctx, t: TypeId| match cx.m.types.get(t) {
                Type::Int(w) if *w <= 64 => Some(*w),
                _ => None,
            };
            let mut s = Seq { f, new: Vec::new() };
            let out = match (lanes_of(cx, from), bits_of(cx, to), bits_of(cx, from), lanes_of(cx, to)) {
                (Some(n), Some(w), _, _)
                    if n == w
                        && (n == 8 || n == 16)
                        && cx.vector_bits() >= 128
                        && cx.selects_intrinsic("llvm.vector.reduce.add.v8i8") =>
                {
                    // Each lane, as all ones or zeros, kept to its bit
                    // within its byte, and the bytes of each eight summed.
                    let i8t = cx.m.types.int(8);
                    let bytes_ty = cx.m.types.intern(Type::Vector { scalable: false, len: n, elem: i8t });
                    let ones = s.cast(CastOp::SExt, value, bytes_ty);
                    let weights: Vec<ConstId> = (0..n)
                        .map(|k| {
                            cx.m.consts.intern(Constant {
                                ty: i8t,
                                kind: ConstKind::Int(ApInt::from_i64(8, i64::from((1u8 << (k % 8)) as i8))),
                            })
                        })
                        .collect();
                    let weights =
                        Value::Const(cx.m.consts.intern(Constant { ty: bytes_ty, kind: ConstKind::Vector(weights) }));
                    let bits = s.bin(BinOp::And, ones, weights, bytes_ty);
                    let eight = cx.m.types.intern(Type::Vector { scalable: false, len: 8, elem: i8t });
                    let mut acc = cx.int(to, 0);
                    for h in 0..n / 8 {
                        let part = if n == 8 {
                            bits
                        } else {
                            let p =
                                Value::Const(cx.m.consts.intern(Constant { ty: bytes_ty, kind: ConstKind::Poison }));
                            let lanes: Vec<u32> = (h * 8..h * 8 + 8).collect();
                            shuffle_lanes(cx, &mut s, bits, p, &lanes, eight)
                        };
                        let sum = call_intrinsic(cx, &mut s, "llvm.vector.reduce.add.v8i8", vec![part], i8t);
                        let z = if to == i8t { sum } else { s.cast(CastOp::ZExt, sum, to) };
                        acc = if h == 0 {
                            z
                        } else {
                            let sh = cx.int(to, 8 * h as i64);
                            let up = s.bin(BinOp::Shl, z, sh, to);
                            s.bin(BinOp::Or, acc, up, to)
                        };
                    }
                    acc
                }
                (Some(n), Some(w), _, _) if n == w => {
                    // The lanes as bytes of 0 or 1 first: masks have
                    // instructions for that, and bytes can be extracted.
                    let i8t = cx.m.types.int(8);
                    let bytes_ty = cx.m.types.intern(Type::Vector { scalable: false, len: n, elem: i8t });
                    let bytes = s.cast(CastOp::ZExt, value, bytes_ty);
                    let mut acc = cx.int(to, 0);
                    for lane in 0..n {
                        let idx = cx.int(i64t, lane as i64);
                        let e = s.op(Op::ExtractElement { vector: bytes, index: idx }, i8t);
                        let z = s.cast(CastOp::ZExt, e, to);
                        let sh = cx.int(to, lane as i64);
                        let bit = if lane == 0 { z } else { s.bin(BinOp::Shl, z, sh, to) };
                        acc = if lane == 0 { bit } else { s.bin(BinOp::Or, acc, bit, to) };
                    }
                    acc
                }
                (_, _, Some(w), Some(n)) if n == w => {
                    // Bytes of 0 or 1, then compared with zero.
                    let i8t = cx.m.types.int(8);
                    let bytes_ty = cx.m.types.intern(Type::Vector { scalable: false, len: n, elem: i8t });
                    let one = cx.int(from, 1);
                    let mut acc = Value::Const(cx.m.consts.intern(Constant { ty: bytes_ty, kind: ConstKind::Poison }));
                    for lane in 0..n {
                        let sh = cx.int(from, lane as i64);
                        let down = if lane == 0 { value } else { s.bin(BinOp::LShr, value, sh, from) };
                        let bit = s.bin(BinOp::And, down, one, from);
                        let byte = s.cast(CastOp::Trunc, bit, i8t);
                        let idx = cx.int(i64t, lane as i64);
                        acc = s.op(Op::InsertElement { vector: acc, element: byte, index: idx }, bytes_ty);
                    }
                    let zero = Value::Const(cx.m.consts.intern(Constant { ty: bytes_ty, kind: ConstKind::Zero }));
                    s.op(Op::ICmp { pred: ICmpPred::Ne, samesign: false, lhs: acc, rhs: zero }, to)
                }
                _ => {
                    k += 1;
                    continue;
                }
            };
            let new = std::mem::take(&mut s.new);
            f.replace_uses(Value::Local(r), out);
            let n = new.len();
            f.blocks[b.index()].insts.splice(k..=k, new);
            k += n;
        }
    }
}

/// Constant lane indices as `i64`, whatever type they were written with
/// (rustc writes `i32 0`): the description's patterns name them so, and a
/// lane's number does not depend on its type.
pub(crate) fn lane_indices(cx: &mut Ctx, f: &mut Function) {
    let i64t = cx.m.types.int(64);
    for b in f.layout.clone() {
        for k in 0..f.blocks[b.index()].insts.len() {
            let i = f.blocks[b.index()].insts[k];
            let index = match &f.insts[i.index()].op {
                Op::ExtractElement { index, .. } | Op::InsertElement { index, .. } => *index,
                _ => continue,
            };
            if cx.value_ty(f, index) == i64t {
                continue;
            }
            let Some(v) = cx.const_int(index) else { continue };
            let new = cx.int(i64t, v);
            match &mut f.insts[i.index()].op {
                Op::ExtractElement { index, .. } | Op::InsertElement { index, .. } => *index = new,
                _ => {}
            }
        }
    }
}

/// Builds instructions to go before another, of one integer type.
pub(crate) struct Seq<'f> {
    pub f: &'f mut Function,
    pub new: Vec<InstId>,
}

impl Seq<'_> {
    pub fn op(&mut self, op: Op, ty: TypeId) -> Value {
        let i = self.f.add_inst(op, Some((None, ty)));
        self.new.push(i);
        self.f.result(i)
    }

    pub fn op_void(&mut self, op: Op) {
        let i = self.f.add_inst(op, None);
        self.new.push(i);
    }

    pub fn bin(&mut self, op: BinOp, lhs: Value, rhs: Value, ty: TypeId) -> Value {
        self.op(Op::Binary { op, flags: IntFlags::NONE, fmf: FastMath::default(), lhs, rhs }, ty)
    }

    pub fn cast(&mut self, op: CastOp, value: Value, ty: TypeId) -> Value {
        self.op(Op::Cast { op, flags: IntFlags::NONE, fmf: FastMath::default(), value }, ty)
    }
}

/// The high half of `x * m` at `w` bits, then shifted right by `extra`:
/// the product of the operands extended to `2w` bits.
fn mul_high(cx: &mut Ctx, s: &mut Seq, x: Value, m: i64, w: u32, signed: bool, extra: u32) -> Value {
    let ty = cx.m.types.int(w);
    let wide = cx.m.types.int(2 * w);
    let ext = if signed { CastOp::SExt } else { CastOp::ZExt };
    let mc = cx.int(ty, m);
    let a = s.cast(ext, x, wide);
    let b = s.cast(ext, mc, wide);
    let p = s.bin(BinOp::Mul, a, b, wide);
    // At 32 bits, any further shift joins this one; at 64, smulh and
    // umulh take the high half as it is.
    let (shift, rest) = if w == 32 { (32 + extra, 0) } else { (64, extra) };
    let sh = cx.int(wide, shift as i64);
    let op = if signed && w == 32 { BinOp::AShr } else { BinOp::LShr };
    let h = s.bin(op, p, sh, wide);
    let hi = s.cast(CastOp::Trunc, h, ty);
    if rest > 0 {
        let k = cx.int(ty, rest as i64);
        return s.bin(if signed { BinOp::AShr } else { BinOp::LShr }, hi, k, ty);
    }
    hi
}

fn lower_const_divs(cx: &mut Ctx, f: &mut Function) {
    let blocks: Vec<BlockId> = f.layout.clone();
    for b in blocks {
        let mut k = 0;
        while k < f.blocks[b.index()].insts.len() {
            let i = f.blocks[b.index()].insts[k];
            let Op::Binary { op, lhs: x, rhs, .. } = f.insts[i.index()].op else {
                k += 1;
                continue;
            };
            let signed = matches!(op, BinOp::SDiv | BinOp::SRem);
            let rem = matches!(op, BinOp::SRem | BinOp::URem);
            let (Some(d), Type::Int(w)) = (cx.const_int(rhs), cx.m.types.get(cx.value_ty(f, x)).clone()) else {
                k += 1;
                continue;
            };
            if !matches!(op, BinOp::SDiv | BinOp::SRem | BinOp::UDiv | BinOp::URem) || !(w == 32 || w == 64) || d == 0 {
                k += 1;
                continue;
            }
            // Elective (autochop): the division instruction does it too.
            if !epitome_ir::chop::allow(|| {
                format!("divide by {d} with a multiply at {}", epitome_ir::printer::inst_name(f, i))
            }) {
                k += 1;
                continue;
            }
            let ty = cx.m.types.int(w);
            let mask: u64 = if w == 64 { u64::MAX } else { (1u64 << w) - 1 };
            let mut s = Seq { f, new: Vec::new() };
            let zero = cx.int(ty, 0);
            let c = |cx: &mut Ctx, v: i64| cx.int(ty, v);
            let result = if signed {
                let ad = d.unsigned_abs() & mask;
                if d == 1 || d == -1 {
                    let q = if d == 1 { x } else { s.bin(BinOp::Sub, zero, x, ty) };
                    if rem { zero } else { q }
                } else if ad.is_power_of_two() {
                    // Round towards zero: add 2^k - 1 to negative values.
                    let kk = ad.trailing_zeros() as i64;
                    let sign = s.bin(BinOp::AShr, x, c(cx, w as i64 - 1), ty);
                    let bias = s.bin(BinOp::LShr, sign, c(cx, w as i64 - kk), ty);
                    let t = s.bin(BinOp::Add, x, bias, ty);
                    if rem {
                        // (The mask is -ad, which for the most negative divisor is itself.)
                        let low = s.bin(BinOp::And, t, c(cx, (ad as i64).wrapping_neg()), ty);
                        s.bin(BinOp::Sub, x, low, ty)
                    } else {
                        let q = s.bin(BinOp::AShr, t, c(cx, kk), ty);
                        if d < 0 { s.bin(BinOp::Sub, zero, q, ty) } else { q }
                    }
                } else {
                    let (m, sh) = crate::divconst::magic_signed(d, w);
                    let needs_fix = (d > 0 && m < 0) || (d < 0 && m > 0);
                    let mut q = mul_high(cx, &mut s, x, m, w, true, if needs_fix { 0 } else { sh });
                    if needs_fix {
                        q = s.bin(if d > 0 { BinOp::Add } else { BinOp::Sub }, q, x, ty);
                        if sh > 0 {
                            q = s.bin(BinOp::AShr, q, c(cx, sh as i64), ty);
                        }
                    }
                    let sign = s.bin(BinOp::LShr, q, c(cx, w as i64 - 1), ty);
                    let q = s.bin(BinOp::Add, q, sign, ty);
                    if rem {
                        let p = s.bin(BinOp::Mul, q, c(cx, d), ty);
                        s.bin(BinOp::Sub, x, p, ty)
                    } else {
                        q
                    }
                }
            } else {
                let du = (d as u64) & mask;
                if du == 1 {
                    if rem { zero } else { x }
                } else if du.is_power_of_two() {
                    if rem {
                        s.bin(BinOp::And, x, c(cx, (du - 1) as i64), ty)
                    } else {
                        s.bin(BinOp::LShr, x, c(cx, du.trailing_zeros() as i64), ty)
                    }
                } else {
                    let q = if du > mask / 2 {
                        // Only 0 or 1 fits.
                        let i1 = cx.m.types.int(1);
                        let ge = s.op(Op::ICmp { pred: ICmpPred::Uge, samesign: false, lhs: x, rhs: c(cx, d) }, i1);
                        s.cast(CastOp::ZExt, ge, ty)
                    } else {
                        let (m, add, sh) = crate::divconst::magic_unsigned(du, w);
                        if add {
                            let t = mul_high(cx, &mut s, x, m as i64, w, false, 0);
                            let diff = s.bin(BinOp::Sub, x, t, ty);
                            let half = s.bin(BinOp::LShr, diff, c(cx, 1), ty);
                            let sum = s.bin(BinOp::Add, half, t, ty);
                            s.bin(BinOp::LShr, sum, c(cx, sh as i64 - 1), ty)
                        } else {
                            mul_high(cx, &mut s, x, m as i64, w, false, sh)
                        }
                    };
                    if rem {
                        let p = s.bin(BinOp::Mul, q, c(cx, d), ty);
                        s.bin(BinOp::Sub, x, p, ty)
                    } else {
                        q
                    }
                }
            };
            let new = std::mem::take(&mut s.new);
            let res = f.insts[i.index()].result.unwrap();
            f.replace_uses(Value::Local(res), result);
            let n = new.len();
            f.blocks[b.index()].insts.splice(k..=k, new);
            k += n;
        }
    }
}

fn legalize_fp(cx: &mut Ctx, f: &mut Function) -> Result<(), String> {
    let i32t = cx.m.types.int(32);
    let i1 = cx.m.types.int(1);
    let blocks: Vec<BlockId> = f.layout.clone();
    for b in blocks {
        let mut k = 0;
        while k < f.blocks[b.index()].insts.len() {
            let i = f.blocks[b.index()].insts[k];
            let res = f.insts[i.index()].result;
            let mut s = Seq { f, new: Vec::new() };
            // Scalars only: vectors have their own conversions.
            let bits_of = |cx: &Ctx, ty: TypeId| match cx.m.types.get(ty) {
                Type::Int(b) => *b,
                _ => 64,
            };
            let mut replace: Option<Value> = None;
            match s.f.insts[i.index()].op.clone() {
                Op::Cast { op: op @ (CastOp::SIToFP | CastOp::UIToFP), value, .. } => {
                    let from = bits_of(cx, cx.value_ty(s.f, value));
                    if from < 32 {
                        let ext = if op == CastOp::SIToFP { CastOp::SExt } else { CastOp::ZExt };
                        let w = s.cast(ext, value, i32t);
                        if let Op::Cast { value, .. } = &mut s.f.insts[i.index()].op {
                            *value = w;
                        }
                    }
                }
                Op::Cast { op: op @ (CastOp::FPToSI | CastOp::FPToUI), value, .. } => {
                    let r = res.unwrap();
                    let to_ty = s.f.locals[r.index()].ty;
                    if bits_of(cx, to_ty) < 32 {
                        // Convert to 32 bits, then take the low bits.
                        let w = s.op(Op::Cast { op, flags: IntFlags::NONE, fmf: FastMath::default(), value }, i32t);
                        replace = Some(s.cast(CastOp::Trunc, w, to_ty));
                    }
                }
                // (Of scalars: vectors' are legalisation's, and x86's cmppd
                // has oeq and une.)
                Op::FCmp { pred, lhs, rhs, fmf }
                    if !matches!(cx.m.types.get(cx.value_ty(s.f, lhs)), Type::Vector { .. })
                        && (matches!(pred, FCmpPred::True | FCmpPred::False)
                            || cx.t.machine.fcmp_split.iter().any(|p| {
                                matches!(
                                    (p.as_str(), pred),
                                    ("one", FCmpPred::One)
                                        | ("ueq", FCmpPred::Ueq)
                                        | ("oeq", FCmpPred::Oeq)
                                        | ("une", FCmpPred::Une)
                                )
                            })) =>
                {
                    replace = Some(match pred {
                        FCmpPred::True => cx.int(i1, 1),
                        FCmpPred::False => cx.int(i1, 0),
                        _ => {
                            // A comparison the machine's conditions do not
                            // test, of two that they do: one, ordered and not
                            // equal; ueq, unordered or equal; oeq, ordered and
                            // (unordered or) equal; une, unordered or ordered
                            // and not equal.
                            let (a, b, op) = match pred {
                                FCmpPred::One => (FCmpPred::Ord, FCmpPred::Une, BinOp::And),
                                FCmpPred::Ueq => (FCmpPred::Uno, FCmpPred::Oeq, BinOp::Or),
                                FCmpPred::Oeq => (FCmpPred::Ord, FCmpPred::Ueq, BinOp::And),
                                _ => (FCmpPred::Uno, FCmpPred::One, BinOp::Or),
                            };
                            let x = s.op(Op::FCmp { pred: a, fmf, lhs, rhs }, i1);
                            let y = s.op(Op::FCmp { pred: b, fmf, lhs, rhs }, i1);
                            s.bin(op, x, y, i1)
                        }
                    });
                }
                Op::Store { value: Value::Const(c), ptr, volatile, atomic, align } => {
                    let k = cx.m.constant(c);
                    if let ConstKind::Float(raw) = k.kind {
                        let bits = match cx.m.types.get(k.ty) {
                            Type::Double => 64,
                            Type::Float => 32,
                            _ => 0,
                        };
                        if bits > 0 {
                            let ity = cx.m.types.int(bits);
                            let v = cx.int(ity, crate::target::sext(raw as u64, bits));
                            s.f.insts[i.index()].op = Op::Store { value: v, ptr, volatile, atomic, align };
                        }
                    }
                }
                // The C library's remainder, which is frem's.
                Op::Binary { op: BinOp::FRem, lhs, rhs, .. } => {
                    let ty = s.f.locals[res.unwrap().index()].ty;
                    let name = libm_name(cx, "fmod", ty).ok_or("frem of this type is not supported")?;
                    replace = Some(lanewise_call(cx, &mut s, &name, vec![lhs, rhs], ty));
                }
                _ => {}
            }
            let new = std::mem::take(&mut s.new);
            let n = new.len();
            match replace {
                Some(v) => {
                    f.replace_uses(Value::Local(res.unwrap()), v);
                    f.blocks[b.index()].insts.splice(k..=k, new);
                    k += n;
                }
                None => {
                    f.blocks[b.index()].insts.splice(k..k, new);
                    k += n + 1;
                }
            }
        }
    }
    Ok(())
}

fn widen_narrow(cx: &mut Ctx, f: &mut Function) {
    let i32t = cx.m.types.int(32);
    let blocks: Vec<BlockId> = f.layout.clone();
    for b in blocks {
        let mut k = 0;
        while k < f.blocks[b.index()].insts.len() {
            let i = f.blocks[b.index()].insts[k];
            let (signed, lhs, rhs) = match f.insts[i.index()].op {
                Op::ICmp { pred, lhs, rhs, .. } => {
                    (matches!(pred, ICmpPred::Sgt | ICmpPred::Sge | ICmpPred::Slt | ICmpPred::Sle), lhs, rhs)
                }
                Op::Binary { op: BinOp::SDiv | BinOp::SRem | BinOp::AShr, lhs, rhs, .. } => (true, lhs, rhs),
                Op::Binary { op: BinOp::UDiv | BinOp::URem | BinOp::LShr, lhs, rhs, .. } => (false, lhs, rhs),
                _ => {
                    k += 1;
                    continue;
                }
            };
            let bits = match cx.m.types.get(cx.value_ty(f, lhs)) {
                Type::Int(b) if *b < 32 => *b,
                _ => {
                    k += 1;
                    continue;
                }
            };
            let mut new = Vec::new();
            let mut widen = |cx: &mut Ctx, f: &mut Function, v: Value| -> Value {
                if let Some(c) = cx.const_int(v) {
                    let raw = c as u64 & ((1u64 << bits) - 1);
                    return cx.int(i32t, if signed { sext(raw, bits) } else { raw as i64 });
                }
                let op = if signed { CastOp::SExt } else { CastOp::ZExt };
                let c = f.add_inst(
                    Op::Cast { op, flags: IntFlags::NONE, fmf: FastMath::default(), value: v },
                    Some((None, i32t)),
                );
                new.push(c);
                f.result(c)
            };
            let (l, r) = (widen(cx, f, lhs), widen(cx, f, rhs));
            let narrow_result = !matches!(f.insts[i.index()].op, Op::ICmp { .. });
            match &mut f.insts[i.index()].op {
                Op::ICmp { lhs, rhs, .. } | Op::Binary { lhs, rhs, .. } => {
                    *lhs = l;
                    *rhs = r;
                }
                _ => unreachable!(),
            }
            let n = new.len();
            f.blocks[b.index()].insts.splice(k..k, new);
            k += n;
            if narrow_result {
                // The operation is now 32 bits wide; its users still see
                // the narrow type through a truncation.
                let res = f.insts[i.index()].result.unwrap();
                let narrow_ty = f.locals[res.index()].ty;
                f.locals[res.index()].ty = i32t;
                let t = f.add_inst(
                    Op::Cast {
                        op: CastOp::Trunc,
                        flags: IntFlags::NONE,
                        fmf: FastMath::default(),
                        value: Value::Local(res),
                    },
                    Some((None, narrow_ty)),
                );
                let tv = f.result(t);
                f.replace_uses(Value::Local(res), tv);
                // Undo the replacement in the truncation itself.
                f.insts[t.index()].op = Op::Cast {
                    op: CastOp::Trunc,
                    flags: IntFlags::NONE,
                    fmf: FastMath::default(),
                    value: Value::Local(res),
                };
                f.blocks[b.index()].insts.insert(k + 1, t);
                k += 1;
            }
            k += 1;
        }
    }
}

fn gep8(i8t: TypeId, ptr: Value, off: Value) -> Op {
    Op::Gep { flags: GepFlags::default(), inrange: None, source_ty: i8t, ptr, indices: vec![off] }
}

pub fn struct_fields(cx: &Ctx, ty: TypeId) -> Result<(Vec<TypeId>, bool), String> {
    match cx.m.types.get(ty) {
        Type::Struct { packed, fields } => Ok((fields.clone(), *packed)),
        Type::Named(n) => {
            let body = cx.m.types.structs[*n as usize].body.as_ref().ok_or("an opaque struct")?;
            Ok((body.fields.clone(), body.packed))
        }
        _ => Err("not a struct".into()),
    }
}

/// For machines whose branches compare whole registers: a branch on a
/// boolean that is not an integer comparison in the same block (whose
/// branch can compare its operands) becomes a branch on the boolean,
/// zero-extended, compared with zero. A boolean's bits above the lowest are
/// undefined, so the extension is needed.
fn compare_boolean_branches(cx: &mut Ctx, f: &mut Function) {
    let i64t = cx.m.types.int(64);
    let i1 = cx.m.types.int(1);
    let zero = cx.int(i64t, 0);
    for b in f.layout.clone() {
        let Some(t) = f.terminator(b) else { continue };
        let Op::CondBr { cond: Value::Local(c), then_dest, else_dest } = f.insts[t.index()].op.clone() else {
            continue;
        };
        let compared_here = match f.locals[c.index()].def {
            LocalDef::Inst(d) if matches!(f.insts[d.index()].op, Op::ICmp { .. }) => {
                f.blocks[b.index()].insts.contains(&d).then_some(d)
            }
            _ => None,
        };
        if let Some(d) = compared_here {
            // Used elsewhere too, the comparison is a value selection
            // does not take into the branch: the branch gets its own.
            let mut uses = 0;
            for i in f.insts_in_order() {
                f.insts[i.index()].op.for_each_value(|v| uses += usize::from(*v == Value::Local(c)));
            }
            if uses > 1 {
                let again = f.add_inst(f.insts[d.index()].op.clone(), Some((None, i1)));
                let at = f.blocks[b.index()].insts.len() - 1;
                f.blocks[b.index()].insts.insert(at, again);
                f.insts[t.index()].op = Op::CondBr { cond: f.result(again), then_dest, else_dest };
            }
            continue;
        }
        let z = f.add_inst(
            Op::Cast { op: CastOp::ZExt, flags: IntFlags::NONE, fmf: FastMath::default(), value: Value::Local(c) },
            Some((None, i64t)),
        );
        let cmp =
            f.add_inst(Op::ICmp { pred: ICmpPred::Ne, samesign: false, lhs: f.result(z), rhs: zero }, Some((None, i1)));
        let at = f.blocks[b.index()].insts.len() - 1;
        f.blocks[b.index()].insts.splice(at..at, [z, cmp]);
        f.insts[t.index()].op = Op::CondBr { cond: f.result(cmp), then_dest, else_dest };
    }
}

/// Splits each edge from a block with several successors into a block
/// with phis.
fn split_phi_edges(f: &mut Function) {
    let blocks: Vec<BlockId> = f.layout.clone();
    for p in blocks {
        let Some(t) = f.terminator(p) else { continue };
        let succs = f.insts[t.index()].op.successors();
        if succs.len() < 2 {
            continue;
        }
        let has_phis = |f: &Function, s: BlockId| {
            f.blocks[s.index()].insts.first().is_some_and(|&i| f.insts[i.index()].op.is_phi())
        };
        let mut edge = 0;
        let mut new_targets: Vec<BlockId> = Vec::new();
        // An invoke's landing pad is reached only by unwinding: that edge
        // is never split (its phis' copies go before the call).
        let unwind = match &f.insts[t.index()].op {
            Op::Invoke { unwind, .. } => Some(*unwind),
            _ => None,
        };
        for s in succs {
            if !has_phis(f, s) || Some(s) == unwind {
                new_targets.push(s);
                continue;
            }
            let e = f.add_block(None);
            let at = f.layout.iter().position(|&x| x == p).unwrap() + 1 + edge;
            edge += 1;
            f.layout.insert(at, e);
            f.push_inst(e, Op::Br(s), None);
            // One of `p`'s entries in the phis moves to `e`.
            let phis: Vec<InstId> =
                f.blocks[s.index()].insts.iter().copied().take_while(|&i| f.insts[i.index()].op.is_phi()).collect();
            for phi in phis {
                if let Op::Phi { incoming, .. } = &mut f.insts[phi.index()].op
                    && let Some(entry) = incoming.iter_mut().find(|(_, from)| *from == p)
                {
                    entry.1 = e;
                }
            }
            new_targets.push(e);
        }
        let mut it = new_targets.into_iter();
        f.insts[t.index()].op.for_each_successor_mut(|s| *s = it.next().unwrap());
    }
}

/// A global and a byte offset, for a constant address.
pub fn const_address(cx: &Ctx, c: ConstId) -> Option<(GlobalId, i64)> {
    let k = cx.m.constant(c);
    match &k.kind {
        ConstKind::Global(g) => Some((*g, 0)),
        ConstKind::Expr(e) => match e {
            ConstExpr::Cast { value, .. } => const_address(cx, *value),
            // `getelementptr (i8, ptr null, i64 ptrtoint (ptr @g))`: @g's
            // address, as an offset from null.
            ConstExpr::Gep { ptr: base, indices, source_ty, .. }
                if matches!(cx.m.constant(*base).kind, ConstKind::Null | ConstKind::Zero)
                    && indices.len() == 1
                    && cx.size_of(*source_ty).ok() == Some(1) =>
            {
                match &cx.m.constant(indices[0]).kind {
                    ConstKind::Expr(ConstExpr::Cast { op: CastOp::PtrToInt | CastOp::PtrToAddr, value, .. }) => {
                        const_address(cx, *value)
                    }
                    _ => None,
                }
            }
            ConstExpr::Gep { ptr: base, indices, source_ty, .. } => {
                let (g, mut off) = const_address(cx, *base)?;
                let mut ty = *source_ty;
                for (n, &idx) in indices.iter().enumerate() {
                    let i = {
                        let ik = cx.m.constant(idx);
                        match (&ik.kind, cx.m.types.get(ik.ty)) {
                            (ConstKind::Int(a), Type::Int(b)) => sext(a.low_u64(), *b),
                            (ConstKind::Zero | ConstKind::Null, _) => 0,
                            _ => return None,
                        }
                    };
                    if n == 0 {
                        off += i * cx.size_of(ty).ok()? as i64;
                        continue;
                    }
                    match cx.m.types.get(ty).clone() {
                        Type::Struct { .. } | Type::Named(_) => {
                            let (fields, packed) = struct_fields(cx, ty).ok()?;
                            let layout = cx.dl.struct_layout(&cx.m.types, &fields, packed)?;
                            off += layout.offsets[i as usize] as i64;
                            ty = fields[i as usize];
                        }
                        Type::Array { elem, .. } | Type::Vector { elem, .. } => {
                            off += i * cx.size_of(elem).ok()? as i64;
                            ty = elem;
                        }
                        _ => return None,
                    }
                }
                Some((g, off))
            }
            _ => None,
        },
        _ => None,
    }
}
