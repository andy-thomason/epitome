//! Operations on legal vectors that no instruction of the description does,
//! done a lane at a time, after legalisation (`legalize::legalize_vectors`)
//! and the lanes of masks it leaves (`prepare::mask_lanes`).
//! It matters where the vector instructions are few (x86's SSE2: no
//! multiplication of 32-bit lanes, no shifts by vectors, no byte lanes'
//! inserts); on AArch64 nearly everything has an instruction.
//!
//! - An operation (arithmetic, a comparison, a cast, a select by a mask, an
//!   element-wise intrinsic) whose head no pattern has: its operands' lanes
//!   read, the scalar operation on each, and the result's lanes written.
//! - Lanes are read and written through a stack slot: the vector stored,
//!   each lane loaded; each lane stored, the vector loaded. A constant's
//!   lanes are its elements.
//! - A mask (`<N x i1>`) is read and written as the integers of its lanes,
//!   all ones or zeros (`llvm.epitome.mask.bits.vNiW` and
//!   `llvm.epitome.bits.mask.vNiW`, which the description has as copies),
//!   `W` the width of the lanes compared.
//! - Then `insertelement` and `extractelement` with no instruction go
//!   through a stack slot themselves.

use epitome_ir::module::*;
use epitome_ir::types::{Type, TypeId};

use crate::prepare::{Ctx, Seq, callee_name};
use crate::target::{Head, head_bits, wide_bits};

fn vec_of(cx: &Ctx, ty: TypeId) -> Option<(u32, TypeId)> {
    match *cx.m.types.get(ty) {
        Type::Vector { len, elem, scalable: false } => Some((len, elem)),
        _ => None,
    }
}

fn is_mask(cx: &Ctx, ty: TypeId) -> bool {
    vec_of(cx, ty).is_some_and(|(_, e)| matches!(cx.m.types.get(e), Type::Int(1)))
}

fn lane_bits(cx: &Ctx, ty: TypeId) -> u32 {
    match cx.m.types.get(ty) {
        Type::Int(b) => *b,
        Type::Float => 32,
        Type::Double => 64,
        Type::Half => 16,
        Type::Ptr(_) => 64,
        _ => 0,
    }
}

/// Element-wise intrinsics, whose lanes are independent: `llvm.NAME.vN...`.
const ELEMENTWISE: &[&str] = &[
    "fabs",
    "sqrt",
    "floor",
    "ceil",
    "trunc",
    "rint",
    "nearbyint",
    "round",
    "roundeven",
    "minnum",
    "maxnum",
    "minimum",
    "maximum",
    "minimumnum",
    "maximumnum",
    "copysign",
    "fma",
    "fmuladd",
    "smin",
    "smax",
    "umin",
    "umax",
    "abs",
    "ctpop",
    "ctlz",
    "cttz",
    "bswap",
    "bitreverse",
    "fshl",
    "fshr",
    "sadd.sat",
    "uadd.sat",
    "ssub.sat",
    "usub.sat",
];

/// The base of an element-wise intrinsic's name (`llvm.smin`), if `name`
/// is one on vectors.
fn elementwise(name: &str) -> Option<&str> {
    let rest = name.strip_prefix("llvm.")?;
    let (base, ty) = rest.rsplit_once('.')?;
    (ty.starts_with('v') && ELEMENTWISE.contains(&base)).then(|| &name[..5 + base.len()])
}

/// Is there an instruction for `op`, as selection would look for it?
fn selectable(cx: &Ctx, f: &Function, op: &Op, res: Option<TypeId>) -> bool {
    let hb = |v: Value| head_bits(cx.m, cx.value_ty(f, v));
    let res_hb = res.map_or(0, |t| head_bits(cx.m, t));
    let head = match op {
        Op::Binary { op, rhs, .. } => {
            // (Shifts are by an immediate, the same in each lane.)
            if matches!(op, BinOp::Shl | BinOp::LShr | BinOp::AShr) && !splat_const(cx, *rhs) {
                return false;
            }
            Head::Bin(*op, res_hb)
        }
        Op::ICmp { pred, lhs, .. } => Head::ICmp(*pred, hb(*lhs)),
        Op::FCmp { pred, lhs, .. } => Head::FCmp(*pred, hb(*lhs)),
        Op::FNeg { .. } => Head::FNeg(res_hb),
        Op::Cast { op, value, .. } => Head::Cast(*op, hb(*value), wide_bits(res_hb)),
        Op::Select { .. } => Head::Select(wide_bits(res_hb)),
        Op::Call(c) => return callee_name(cx.m, c).is_none_or(|n| cx.selects_intrinsic(&n)),
        // (A pattern for the lane, or for any lane, of any vector: not
        // only of zero, as `movd` is.)
        Op::InsertElement { index, .. } => return lane_pattern(cx, Head::InsertElement(res_hb), 2, *index),
        Op::ExtractElement { vector, index } => return lane_pattern(cx, Head::ExtractElement(hb(*vector)), 1, *index),
        _ => return true,
    };
    cx.selects_head(|h| *h == head)
}

/// How many times `v` is used.
fn uses(f: &Function, v: Value) -> usize {
    f.insts_in_order()
        .map(|j| {
            let mut n = 0;
            f.insts[j.index()].op.for_each_value(|w| n += usize::from(*w == v));
            n
        })
        .sum()
}

/// Is there a pattern for this shuffle: of any vector, or of an insert's
/// (a splat's), if `a` is one?
fn shuffle_selectable(cx: &Ctx, f: &Function, a: Value, mask: ConstId, rty: TypeId) -> bool {
    use crate::target::Tree;
    let Some(id) = crate::target::mask_lanes(cx.m, mask).and_then(|l| cx.t.find_mask(&l)) else { return false };
    let head = Head::Shuffle(id, head_bits(cx.m, rty));
    // (A splat's: an insert into poison at lane 0, used once, as selection
    // folds only what has one use.)
    let from_insert = uses(f, a) == 1
        && matches!(a, Value::Local(x)
        if matches!(f.locals[x.index()].def, LocalDef::Inst(d)
            if matches!(f.insts[d.index()].op, Op::InsertElement { vector: Value::Const(c), index, .. }
                if matches!(cx.m.constant(c).kind, ConstKind::Poison | ConstKind::Undef) && cx.const_int(index) == Some(0))));
    cx.t.patterns.iter().any(|p| match &p.tree {
        Tree::Op(h, kids) if *h == head => match kids.first() {
            Some(Tree::Leaf(_)) => true,
            Some(Tree::Op(Head::InsertElement(b), _)) => from_insert && *b == head_bits(cx.m, cx.value_ty(f, a)),
            _ => false,
        },
        _ => false,
    })
}

/// Is there a pattern rooted at `head` whose vector is any value and whose
/// lane (operand `at`) is any, or `index`?
fn lane_pattern(cx: &Ctx, head: Head, at: usize, index: Value) -> bool {
    use crate::target::Tree;
    let k = cx.const_int(index);
    cx.t.patterns.iter().any(|p| match &p.tree {
        Tree::Op(h, kids) if *h == head => {
            matches!(kids.first(), Some(Tree::Leaf(_)))
                && match kids.get(at) {
                    Some(Tree::Leaf(_)) => true,
                    Some(Tree::Lit(v, _)) => Some(*v) == k,
                    _ => false,
                }
        }
        _ => false,
    })
}

fn splat_const(cx: &Ctx, v: Value) -> bool {
    let Value::Const(c) = v else { return false };
    match &cx.m.constant(c).kind {
        ConstKind::Splat(_) => true,
        ConstKind::Vector(es) => es.iter().all(|e| *e == es[0]),
        _ => false,
    }
}

/// Where lanes are written and read: stack slots, made in the entry block.
struct Lanes {
    slots: Vec<InstId>,
}

impl Lanes {
    fn slot(&mut self, cx: &mut Ctx, s: &mut Seq, vty: TypeId) -> Value {
        let ptr = cx.m.types.ptr(0);
        let align = Some(Align::of(cx.size_of(vty).unwrap_or(16).next_power_of_two().clamp(1, 16)));
        let a = s
            .f
            .add_inst(Op::Alloca { inalloca: false, ty: vty, count: None, align, addrspace: None }, Some((None, ptr)));
        self.slots.push(a);
        s.f.result(a)
    }

    /// The lanes of vector `v` (of a mask, as `W`-bit integers, each all
    /// ones or zeros).
    fn read(&mut self, cx: &mut Ctx, s: &mut Seq, v: Value, w: u32) -> Vec<Value> {
        let vty = cx.value_ty(s.f, v);
        let (len, elem) = vec_of(cx, vty).unwrap();
        let mask = is_mask(cx, vty);
        let ety = if mask { cx.m.types.int(w) } else { elem };
        if let Value::Const(c) = v {
            let k = cx.m.constant(c).clone();
            let lane = |cx: &mut Ctx, e: Option<ConstId>| -> Value {
                match e.map(|e| cx.m.constant(e).kind.clone()) {
                    Some(ConstKind::Int(a)) if mask => cx.int(ety, -((a.low_u64() & 1) as i64)),
                    Some(_) if !mask => Value::Const(e.unwrap()),
                    _ => Value::Const(cx.m.consts.intern(Constant { ty: ety, kind: ConstKind::Poison })),
                }
            };
            return match k.kind {
                ConstKind::Vector(es) => es.iter().map(|&e| lane(cx, Some(e))).collect(),
                ConstKind::Splat(e) => (0..len).map(|_| lane(cx, Some(e))).collect(),
                // (Zero lanes as numbers: selection reads an integer's
                // zeroinitializer as no constant.)
                ConstKind::Zero => {
                    let z = match cx.m.types.get(ety) {
                        Type::Int(_) => cx.int(ety, 0),
                        Type::Float | Type::Double | Type::Half => {
                            Value::Const(cx.m.consts.intern(Constant { ty: ety, kind: ConstKind::Float(0) }))
                        }
                        _ => Value::Const(cx.m.consts.intern(Constant { ty: ety, kind: ConstKind::Zero })),
                    };
                    vec![z; len as usize]
                }
                _ => (0..len).map(|_| lane(cx, None)).collect(),
            };
        }
        let (v, vty) = if mask {
            let ity = cx.m.types.intern(Type::Vector { len, elem: ety, scalable: false });
            (crate::prepare::call_intrinsic(cx, s, &format!("llvm.epitome.mask.bits.v{len}i{w}"), vec![v], ity), ity)
        } else {
            (v, vty)
        };
        let at = self.slot(cx, s, vty);
        s.op_void(Op::Store { volatile: false, atomic: None, value: v, ptr: at, align: None });
        let esize = cx.size_of(ety).unwrap_or(1) as i64;
        (0..len as i64)
            .map(|k| {
                let p = gep(cx, s, at, k * esize);
                s.op(Op::Load { volatile: false, atomic: None, ptr: p, align: None }, ety)
            })
            .collect()
    }

    /// A vector of type `vty` made of `lanes` (of a mask: `i1`s, written as
    /// `W`-bit integers).
    fn write(&mut self, cx: &mut Ctx, s: &mut Seq, vty: TypeId, lanes: &[Value], w: u32) -> Value {
        let (len, elem) = vec_of(cx, vty).unwrap();
        let mask = is_mask(cx, vty);
        let ety = if mask { cx.m.types.int(w) } else { elem };
        let ity = cx.m.types.intern(Type::Vector { len, elem: ety, scalable: false });
        let at = self.slot(cx, s, ity);
        let esize = cx.size_of(ety).unwrap_or(1) as i64;
        for (k, &l) in lanes.iter().enumerate() {
            let l = if mask { s.cast(CastOp::SExt, l, ety) } else { l };
            let p = gep(cx, s, at, k as i64 * esize);
            s.op_void(Op::Store { volatile: false, atomic: None, value: l, ptr: p, align: None });
        }
        let v = s.op(Op::Load { volatile: false, atomic: None, ptr: at, align: None }, ity);
        if mask {
            crate::prepare::call_intrinsic(cx, s, &format!("llvm.epitome.bits.mask.v{len}i{w}"), vec![v], vty)
        } else {
            v
        }
    }
}

fn gep(cx: &mut Ctx, s: &mut Seq, p: Value, off: i64) -> Value {
    let (i8t, i64t, ptr) = (cx.m.types.int(8), cx.m.types.int(64), cx.m.types.ptr(0));
    if off == 0 {
        return p;
    }
    let k = cx.int(i64t, off);
    s.op(Op::Gep { flags: GepFlags::default(), source_ty: i8t, ptr: p, indices: vec![k].into() }, ptr)
}

/// The width of a mask's lanes, from what made it (or 8).
fn mask_w(cx: &Ctx, f: &Function, v: Value) -> u32 {
    crate::legalize::mask_width(cx, f, v, &mut Vec::new()).unwrap_or(8)
}

pub fn lanewise(cx: &mut Ctx, f: &mut Function) -> Result<(), String> {
    // (For descriptions with masks' bits as copies, which this needs; on
    // AArch64, legalisation's own lane-by-lane cases are enough.)
    if cx.vector_bits() == 0 || !cx.selects_intrinsic("llvm.epitome.mask.bits.v4i32") {
        return Ok(());
    }
    // (Lane numbers legalisation made as i32s, as i64s.)
    crate::prepare::lane_indices(cx, f);
    let mut lanes = Lanes { slots: Vec::new() };
    let blocks: Vec<BlockId> = f.layout.clone();
    // Operations first, then the inserts and extracts left.
    for pass in 0..2 {
        for &b in &blocks {
            let mut k = 0;
            while k < f.blocks[b.index()].insts.len() {
                let i = f.blocks[b.index()].insts[k];
                let n = if pass == 0 { one(cx, f, &mut lanes, i)? } else { one_lane(cx, f, &mut lanes, i) };
                match n {
                    Some(new) => {
                        let n = new.len();
                        f.blocks[b.index()].insts.splice(k..=k, new);
                        k += n;
                    }
                    None => k += 1,
                }
            }
        }
    }
    if let Some(&entry) = f.layout.first() {
        f.blocks[entry.index()].insts.splice(0..0, lanes.slots);
    }
    Ok(())
}

/// The instructions in place of operation `i`, done a lane at a time, if no
/// instruction does it.
fn one(cx: &mut Ctx, f: &mut Function, lanes: &mut Lanes, i: InstId) -> Result<Option<Vec<InstId>>, String> {
    let op = f.insts[i.index()].op.clone();
    let Some(r) = f.insts[i.index()].result else { return Ok(None) };
    let rty = f.locals[r.index()].ty;
    let Some((len, relem)) = vec_of(cx, rty) else { return Ok(None) };
    let lanewise_op = match &op {
        Op::Binary { .. } | Op::ICmp { .. } | Op::FCmp { .. } | Op::FNeg { .. } => true,
        // (A bitcast is not lane for lane.)
        Op::Cast { op, value, .. } => {
            *op != CastOp::BitCast && vec_of(cx, cx.value_ty(f, *value)).is_some_and(|(n, _)| n == len)
        }
        Op::Select { cond, .. } => vec_of(cx, cx.value_ty(f, *cond)).is_some(),
        Op::Call(c) => callee_name(cx.m, c).is_some_and(|n| elementwise(&n).is_some()),
        _ => false,
    };
    if !lanewise_op || selectable(cx, f, &op, Some(rty)) {
        return Ok(None);
    }
    // A mask sign-extended to lanes of its width: its bits.
    if let Op::Cast { op: CastOp::SExt, value, .. } = op
        && is_mask(cx, cx.value_ty(f, value))
        && mask_w(cx, f, value) == lane_bits(cx, relem)
    {
        let w = lane_bits(cx, relem);
        let mut s = Seq { f, new: Vec::new() };
        let v =
            crate::prepare::call_intrinsic(cx, &mut s, &format!("llvm.epitome.mask.bits.v{len}i{w}"), vec![value], rty);
        s.f.replace_uses(Value::Local(r), v);
        return Ok(Some(s.new));
    }
    // Masks' lane widths: of what is compared, or of the result's lanes.
    let rw = lane_bits(cx, relem);
    let mut s = Seq { f, new: Vec::new() };
    let i1 = cx.m.types.int(1);
    let (out, w) = match op {
        Op::Binary { op, flags, fmf, lhs, rhs } => {
            let w = if relem == i1 { mask_w(cx, s.f, lhs).max(mask_w(cx, s.f, rhs)) } else { rw };
            let (a, b) = (lanes.read(cx, &mut s, lhs, w), lanes.read(cx, &mut s, rhs, w));
            let ety = if relem == i1 { cx.m.types.int(w) } else { relem };
            let out: Vec<Value> = (0..len as usize)
                .map(|k| {
                    let x = s.op(Op::Binary { op, flags, fmf, lhs: a[k], rhs: b[k] }, ety);
                    if relem == i1 { s.cast(CastOp::Trunc, x, i1) } else { x }
                })
                .collect();
            (out, w)
        }
        Op::ICmp { pred, samesign, lhs, rhs } => {
            let et = vec_of(cx, cx.value_ty(s.f, lhs)).unwrap().1;
            let w = lane_bits(cx, et);
            let (a, b) = (lanes.read(cx, &mut s, lhs, w), lanes.read(cx, &mut s, rhs, w));
            ((0..len as usize).map(|k| s.op(Op::ICmp { pred, samesign, lhs: a[k], rhs: b[k] }, i1)).collect(), w)
        }
        Op::FCmp { pred, fmf, lhs, rhs } => {
            let et = vec_of(cx, cx.value_ty(s.f, lhs)).unwrap().1;
            let w = lane_bits(cx, et);
            let (a, b) = (lanes.read(cx, &mut s, lhs, w), lanes.read(cx, &mut s, rhs, w));
            ((0..len as usize).map(|k| s.op(Op::FCmp { pred, fmf, lhs: a[k], rhs: b[k] }, i1)).collect(), w)
        }
        Op::FNeg { fmf, value } => {
            let a = lanes.read(cx, &mut s, value, rw);
            ((0..len as usize).map(|k| s.op(Op::FNeg { fmf, value: a[k] }, relem)).collect(), rw)
        }
        Op::Cast { op, flags, fmf, value } => {
            // A mask's lanes as wide as the result's (or what made it).
            let vt = cx.value_ty(s.f, value);
            let w = if is_mask(cx, vt) { if relem == i1 { mask_w(cx, s.f, value) } else { rw } } else { rw };
            let src_w = if is_mask(cx, vt) { w } else { 8 };
            let a = lanes.read(cx, &mut s, value, src_w);
            let out = (0..len as usize)
                .map(|k| {
                    if is_mask(cx, vt) {
                        // The lane, as the i1 it is.
                        let b = s.cast(CastOp::Trunc, a[k], i1);
                        if relem == i1 { b } else { s.op(Op::Cast { op, flags, fmf, value: b }, relem) }
                    } else {
                        s.op(Op::Cast { op, flags, fmf, value: a[k] }, relem)
                    }
                })
                .collect();
            (out, w)
        }
        Op::Select { fmf, cond, then_value, else_value } => {
            let w = if relem == i1 { mask_w(cx, s.f, then_value) } else { rw };
            let c = lanes.read(cx, &mut s, cond, w.max(8));
            let (a, b) = (lanes.read(cx, &mut s, then_value, w), lanes.read(cx, &mut s, else_value, w));
            let ety = if relem == i1 { cx.m.types.int(w) } else { relem };
            let out = (0..len as usize)
                .map(|k| {
                    let ck = s.cast(CastOp::Trunc, c[k], i1);
                    let x = s.op(Op::Select { fmf, cond: ck, then_value: a[k], else_value: b[k] }, ety);
                    if relem == i1 { s.cast(CastOp::Trunc, x, i1) } else { x }
                })
                .collect();
            (out, w)
        }
        Op::Call(c) => {
            let name = callee_name(cx.m, &c).unwrap();
            let base = elementwise(&name).unwrap().to_string();
            let scalar = format!("{base}.{}", crate::legalize::mangle(cx, relem));
            let args: Vec<Vec<Value>> = c
                .args
                .iter()
                .map(|a| {
                    if vec_of(cx, cx.value_ty(s.f, a.value)).is_some() {
                        lanes.read(cx, &mut s, a.value, rw)
                    } else {
                        vec![a.value; len as usize]
                    }
                })
                .collect();
            let out = (0..len as usize)
                .map(|k| {
                    let lane_args: Vec<Value> = args.iter().map(|a| a[k]).collect();
                    crate::prepare::call_intrinsic(cx, &mut s, &scalar, lane_args, relem)
                })
                .collect();
            (out, rw)
        }
        _ => unreachable!(),
    };
    let v = lanes.write(cx, &mut s, rty, &out, w.max(8));
    s.f.replace_uses(Value::Local(r), v);
    Ok(Some(s.new))
}

/// An `insertelement` or `extractelement` no instruction does, through a
/// stack slot.
fn one_lane(cx: &mut Ctx, f: &mut Function, lanes: &mut Lanes, i: InstId) -> Option<Vec<InstId>> {
    let op = f.insts[i.index()].op.clone();
    let r = f.insts[i.index()].result?;
    let rty = f.locals[r.index()].ty;
    // A shuffle no pattern takes: each lane from the lanes of the two.
    if let Op::ShuffleVector { a, b, mask } = op {
        if is_mask(cx, rty) || shuffle_selectable(cx, f, a, mask, rty) {
            return None;
        }
        let picks = crate::target::mask_lanes(cx.m, mask)?;
        let mut s = Seq { f, new: Vec::new() };
        let (la, lb) = (lanes.read(cx, &mut s, a, 8), lanes.read(cx, &mut s, b, 8));
        let elem = vec_of(cx, rty)?.1;
        let out: Vec<Value> = picks
            .iter()
            .map(|&p| match p {
                p if p < 0 => Value::Const(cx.m.consts.intern(Constant { ty: elem, kind: ConstKind::Poison })),
                p if (p as usize) < la.len() => la[p as usize],
                p => lb[p as usize - la.len()],
            })
            .collect();
        let v = lanes.write(cx, &mut s, rty, &out, 8);
        s.f.replace_uses(Value::Local(r), v);
        return Some(s.new);
    }
    // An insert making a splat's source, which its shuffle's pattern takes
    // (used once, by the shuffle).
    if let Op::InsertElement { vector: Value::Const(c), .. } = op
        && matches!(cx.m.constant(c).kind, ConstKind::Poison | ConstKind::Undef)
        && uses(f, Value::Local(r)) == 1
        && f.insts_in_order().any(|j| {
            matches!(&f.insts[j.index()].op, Op::ShuffleVector { a, mask, .. }
                if *a == Value::Local(r) && shuffle_selectable(cx, f, *a, *mask, f.locals[f.insts[j.index()].result.unwrap().index()].ty))
        })
    {
        return None;
    }
    let (vector, index, element) = match op {
        Op::ExtractElement { vector, index } => (vector, index, None),
        Op::InsertElement { vector, element, index } => (vector, index, Some(element)),
        _ => return None,
    };
    let vty = cx.value_ty(f, vector);
    let (len, _) = vec_of(cx, vty)?;
    if is_mask(cx, vty) || selectable(cx, f, &op, Some(rty)) {
        return None;
    }
    let k = cx.const_int(index)?;
    let mut s = Seq { f, new: Vec::new() };
    let mut ls = lanes.read(cx, &mut s, vector, 8);
    let k = (k as u64 % u64::from(len)) as usize;
    let v = match element {
        None => ls[k],
        Some(e) => {
            ls[k] = e;
            lanes.write(cx, &mut s, rty, &ls, 8)
        }
    };
    s.f.replace_uses(Value::Local(r), v);
    Some(s.new)
}
