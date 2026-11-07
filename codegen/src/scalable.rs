//! Scalable vectors of more than one register (`<vscale x 4 x i64>`, which
//! the loop vectoriser makes widening `<vscale x 4 x i32>`), as parts of
//! one register each, in lane order: each such value becomes a value for
//! each part, and each operation on it one on each part.
//!
//! Lanes cannot be counted, so the parts are not made by shuffles as fixed
//! vectors' are: an extension's are the source's halves unpacked (`sunpklo`
//! and `sunpkhi`), a truncation's the parts' even halves zipped (`uzp1`),
//! and memory's at the part's size, `vscale` times 16 bytes, apart.

use std::collections::HashMap;

use epitome_ir::module::*;
use epitome_ir::types::{Type, TypeId};

use crate::prepare::{Ctx, Seq, call_intrinsic};

/// The bits of a register of scalable vectors, at the least vector length.
const PART_BITS: u32 = 128;

/// The lane count, lane type and lane width of a scalable vector.
fn scalable(cx: &Ctx, ty: TypeId) -> Option<(u32, TypeId, u32)> {
    match *cx.m.types.get(ty) {
        Type::Vector { len, elem, scalable: true } => {
            let eb = match cx.m.types.get(elem) {
                Type::Int(b) => *b,
                Type::Float => 32,
                Type::Double => 64,
                Type::Half | Type::BFloat => 16,
                Type::Ptr(a) => epitome_ir::layout::pointer_bits(cx.m.data_layout.as_deref(), *a),
                _ => return None,
            };
            Some((len, elem, eb))
        }
        _ => None,
    }
}

/// How many parts a scalable vector of `ty` is, if more than one.
fn parts(cx: &Ctx, ty: TypeId) -> Option<usize> {
    let (len, _, eb) = scalable(cx, ty)?;
    (eb > 1 && len * eb > PART_BITS).then(|| (len * eb / PART_BITS) as usize)
}

/// The type of each part of a scalable vector of `ty`.
fn part_ty(cx: &mut Ctx, ty: TypeId) -> TypeId {
    let (len, elem, _) = scalable(cx, ty).unwrap();
    let n = parts(cx, ty).unwrap_or(1) as u32;
    cx.m.types.intern(Type::Vector { len: len / n, elem, scalable: true })
}

/// The LLVM name of a scalable type: `nxv2i64`.
fn mangle(cx: &Ctx, ty: TypeId) -> String {
    crate::legalize::mangle(cx, ty)
}

/// The parts of `v`, of type `ty`: those already made, or a constant's.
fn parts_of(cx: &mut Ctx, s: &mut Seq, split: &HashMap<LocalId, Vec<Value>>, v: Value) -> Result<Vec<Value>, String> {
    let ty = cx.value_ty(s.f, v);
    let n = parts(cx, ty).ok_or("parts of a vector of one register")?;
    match v {
        Value::Local(x) => split.get(&x).cloned().ok_or_else(|| "a split vector not yet made".to_string()),
        Value::Const(c) => {
            let pt = part_ty(cx, ty);
            let kind = match cx.m.constant(c).kind.clone() {
                k @ (ConstKind::Splat(_) | ConstKind::Zero | ConstKind::Undef | ConstKind::Poison) => k,
                other => return Err(format!("a scalable constant {other:?}")),
            };
            Ok(vec![Value::Const(cx.m.consts.intern(Constant { ty: pt, kind })); n])
        }
        _ => Err("metadata".into()),
    }
}

/// `v` (one register) as twice as many lanes' halves (`sunpklo`,
/// `sunpkhi`), of lanes twice as wide.
fn unpack(cx: &mut Ctx, s: &mut Seq, v: Value, signed: bool) -> Result<[Value; 2], String> {
    let (len, _, eb) = scalable(cx, cx.value_ty(s.f, v)).ok_or("unpacking a scalar")?;
    let e = cx.m.types.int(eb * 2);
    let wt = cx.m.types.intern(Type::Vector { len: len / 2, elem: e, scalable: true });
    let k = if signed { "s" } else { "u" };
    let m = mangle(cx, wt);
    let lo = call_intrinsic(cx, s, &format!("llvm.aarch64.sve.{k}unpklo.{m}"), vec![v], wt);
    let hi = call_intrinsic(cx, s, &format!("llvm.aarch64.sve.{k}unpkhi.{m}"), vec![v], wt);
    Ok([lo, hi])
}

/// `a` and `b` (one register each, of the same type) truncated to lanes of
/// half their width, one register: their lanes' low halves, zipped
/// (`uzp1`).
fn narrow(cx: &mut Ctx, s: &mut Seq, a: Value, b: Value) -> Result<Value, String> {
    let (len, _, eb) = scalable(cx, cx.value_ty(s.f, a)).ok_or("narrowing a scalar")?;
    let e = cx.m.types.int(eb / 2);
    let nt = cx.m.types.intern(Type::Vector { len: len * 2, elem: e, scalable: true });
    let (x, y) = (s.cast(CastOp::BitCast, a, nt), s.cast(CastOp::BitCast, b, nt));
    let m = mangle(cx, nt);
    Ok(call_intrinsic(cx, s, &format!("llvm.aarch64.sve.uzp1.{m}"), vec![x, y], nt))
}

/// `ptr` plus `k` registers' bytes (`vscale` times 16 each).
fn part_addr(cx: &mut Ctx, s: &mut Seq, ptr: Value, k: usize) -> Value {
    if k == 0 {
        return ptr;
    }
    let (i8t, i64t, pt) = (cx.m.types.int(8), cx.m.types.int(64), cx.m.types.ptr(0));
    let vs = call_intrinsic(cx, s, "llvm.vscale.i64", vec![], i64t);
    let size = cx.int(i64t, (PART_BITS / 8) as i64 * k as i64);
    let off = s.bin(BinOp::Mul, vs, size, i64t);
    s.op(Op::Gep { flags: GepFlags::default(), source_ty: i8t, ptr, indices: vec![off].into() }, pt)
}

/// The parts of the extension of `v` to `n` parts of `pt`: each part of
/// the source unpacked until its lanes are as wide.
fn extend(
    cx: &mut Ctx,
    s: &mut Seq,
    split: &HashMap<LocalId, Vec<Value>>,
    v: Value,
    pt: TypeId,
    signed: bool,
) -> Result<Vec<Value>, String> {
    let src_ty = cx.value_ty(s.f, v);
    let mut cur = if parts(cx, src_ty).is_some() { parts_of(cx, s, split, v)? } else { vec![v] };
    let want = scalable(cx, pt).unwrap().2;
    loop {
        let eb = scalable(cx, cx.value_ty(s.f, cur[0])).unwrap().2;
        if eb == want {
            return Ok(cur);
        }
        if eb * 2 > want {
            return Err(format!("an extension of {eb}-bit lanes to {want}"));
        }
        let mut next = Vec::new();
        for x in cur {
            next.extend(unpack(cx, s, x, signed)?);
        }
        cur = next;
    }
}

pub fn split(cx: &mut Ctx, f: &mut Function) -> Result<(), String> {
    if !f.locals.iter().any(|l| parts(cx, l.ty).is_some()) {
        return Ok(());
    }
    let mut split: HashMap<LocalId, Vec<Value>> = HashMap::new();
    let mut repl: HashMap<LocalId, Value> = HashMap::new();
    let mut blocks = crate::wide::reverse_postorder(f);
    for &b in &f.layout {
        if !blocks.contains(&b) {
            blocks.push(b);
        }
    }
    // Phis first, one for each part, so that values may come round loops;
    // their incoming parts are filled in at the end.
    let mut phis: Vec<(InstId, Vec<InstId>)> = Vec::new();
    for &b in &blocks {
        for &i in &f.blocks[b.index()].insts.clone() {
            if !f.insts[i.index()].op.is_phi() {
                break;
            }
            let r = f.insts[i.index()].result.unwrap();
            let ty = f.locals[r.index()].ty;
            let Some(n) = parts(cx, ty) else { continue };
            let pt = part_ty(cx, ty);
            let fmf = match &f.insts[i.index()].op {
                Op::Phi { fmf, .. } => *fmf,
                _ => FastMath::default(),
            };
            let part_phis: Vec<InstId> =
                (0..n).map(|_| f.add_inst(Op::Phi { fmf, incoming: vec![] }, Some((None, pt)))).collect();
            split.insert(r, part_phis.iter().map(|&p| f.result(p)).collect());
            phis.push((i, part_phis));
        }
    }
    for b in blocks {
        let old = std::mem::take(&mut f.blocks[b.index()].insts);
        let mut out = Vec::new();
        for i in old {
            if let Some((_, part_phis)) = phis.iter().find(|(p, _)| *p == i) {
                out.extend(part_phis.iter().copied());
                continue;
            }
            f.insts[i.index()].op.for_each_value_mut(|v| {
                if let Value::Local(x) = v
                    && let Some(&n) = repl.get(x)
                {
                    *v = n;
                }
            });
            let op = f.insts[i.index()].op.clone();
            let res = f.insts[i.index()].result;
            let rty = res.map(|r| f.locals[r.index()].ty);
            let res_split = rty.and_then(|t| parts(cx, t));
            let mut split_operand = false;
            op.for_each_value(|v| {
                if let Value::Local(x) = v {
                    split_operand |= split.contains_key(x);
                }
                if let Value::Const(c) = v {
                    split_operand |= parts(cx, cx.m.constant(*c).ty).is_some();
                }
            });
            if res_split.is_none() && !split_operand {
                out.push(i);
                continue;
            }
            let pt = rty.filter(|_| res_split.is_some()).map(|t| part_ty(cx, t));
            let mut s = Seq { f, new: Vec::new() };
            let made: Option<Vec<Value>> = match op {
                Op::Load { volatile, atomic: None, ptr, align } if res_split.is_some() => {
                    let pt = pt.unwrap();
                    Some(
                        (0..res_split.unwrap())
                            .map(|k| {
                                let at = part_addr(cx, &mut s, ptr, k);
                                s.op(Op::Load { volatile, atomic: None, ptr: at, align }, pt)
                            })
                            .collect(),
                    )
                }
                Op::Store { volatile, atomic: None, value, ptr, align } => {
                    let ps = parts_of(cx, &mut s, &split, value)?;
                    for (k, x) in ps.into_iter().enumerate() {
                        let at = part_addr(cx, &mut s, ptr, k);
                        s.op_void(Op::Store { volatile, atomic: None, value: x, ptr: at, align });
                    }
                    out.extend(std::mem::take(&mut s.new));
                    continue;
                }
                Op::Binary { op, flags, fmf, lhs, rhs } if res_split.is_some() => {
                    let l = parts_of(cx, &mut s, &split, lhs)?;
                    let r = parts_of(cx, &mut s, &split, rhs)?;
                    let pt = pt.unwrap();
                    Some(
                        l.into_iter()
                            .zip(r)
                            .map(|(x, y)| s.op(Op::Binary { op, flags, fmf, lhs: x, rhs: y }, pt))
                            .collect(),
                    )
                }
                Op::Freeze(v) if res_split.is_some() => {
                    let l = parts_of(cx, &mut s, &split, v)?;
                    let pt = pt.unwrap();
                    Some(l.into_iter().map(|x| s.op(Op::Freeze(x), pt)).collect())
                }
                Op::Cast { op: op @ (CastOp::SExt | CastOp::ZExt), value, .. } if res_split.is_some() => {
                    Some(extend(cx, &mut s, &split, value, pt.unwrap(), op == CastOp::SExt)?)
                }
                // Truncations: the parts' lanes' low halves, zipped, until
                // the lanes are as narrow.
                Op::Cast { op: CastOp::Trunc, value, .. } => {
                    // (Into one part, the rest a truncation of it: to an
                    // unpacked type, which `unpacked` then holds.)
                    let mut cur = parts_of(cx, &mut s, &split, value)?;
                    let want = scalable(cx, rty.unwrap()).unwrap().2;
                    while scalable(cx, cx.value_ty(s.f, cur[0])).unwrap().2 > want && cur.len() > 1 {
                        if cur.len() % 2 == 1 {
                            return Err("a truncation of an odd number of parts".into());
                        }
                        let mut next = Vec::new();
                        for pair in cur.chunks(2) {
                            next.push(narrow(cx, &mut s, pair[0], pair[1])?);
                        }
                        cur = next;
                    }
                    match res_split {
                        Some(_) => Some(cur),
                        None => {
                            let ty = rty.unwrap();
                            let v = if cx.value_ty(s.f, cur[0]) == ty { cur[0] } else { s.cast(CastOp::Trunc, cur[0], ty) };
                            repl.insert(res.unwrap(), v);
                            None
                        }
                    }
                }
                // Other conversions of as many parts: of each part.
                Op::Cast { op, flags, fmf, value } if res_split.is_some() => {
                    let l = parts_of(cx, &mut s, &split, value)?;
                    let pt = pt.unwrap();
                    if l.len() != res_split.unwrap() {
                        return Err(format!("a {op:?} between scalable vectors of different numbers of parts"));
                    }
                    Some(l.into_iter().map(|x| s.op(Op::Cast { op, flags, fmf, value: x }, pt)).collect())
                }
                // A splat (an insertelement of lane 0, shuffled by zeros):
                // the same of each part.
                Op::InsertElement { vector, element, index }
                    if res_split.is_some() && cx.const_int(index) == Some(0) =>
                {
                    let ps = parts_of(cx, &mut s, &split, vector)?;
                    let pt = pt.unwrap();
                    let first = s.op(Op::InsertElement { vector: ps[0], element, index }, pt);
                    Some(std::iter::once(first).chain(ps[1..].iter().copied()).collect())
                }
                Op::ShuffleVector { a, mask, .. }
                    if res_split.is_some()
                        && matches!(cx.m.constant(mask).kind, ConstKind::Zero)
                        && parts(cx, cx.value_ty(s.f, a)).is_some() =>
                {
                    let ps = parts_of(cx, &mut s, &split, a)?;
                    let pt = pt.unwrap();
                    let (plen, _, _) = scalable(cx, pt).unwrap();
                    let i32t = cx.m.types.int(32);
                    let mt = cx.m.types.intern(Type::Vector { len: plen, elem: i32t, scalable: true });
                    let zero = cx.m.consts.intern(Constant { ty: mt, kind: ConstKind::Zero });
                    let poison = Value::Const(cx.m.consts.intern(Constant { ty: pt, kind: ConstKind::Poison }));
                    let one = s.op(Op::ShuffleVector { a: ps[0], b: poison, mask: zero }, pt);
                    Some(vec![one; res_split.unwrap()])
                }
                Op::Call(ref call) => {
                    let name = crate::prepare::callee_name(cx.m, call).unwrap_or_default();
                    let args: Vec<Value> = call.args.iter().map(|a| a.value).collect();
                    // Two vectors' even and odd lanes, from one of two
                    // parts: its parts unzipped; and back, zipped.
                    if name.starts_with("llvm.vector.deinterleave2.") && res_split.is_none() {
                        let ps = parts_of(cx, &mut s, &split, args[0])?;
                        let [x, y] = ps[..] else { return Err(format!("{name} of more than two parts")) };
                        let ty = rty.unwrap();
                        let pt = cx.value_ty(s.f, x);
                        let m = mangle(cx, pt);
                        let e = call_intrinsic(cx, &mut s, &format!("llvm.aarch64.sve.uzp1.{m}"), vec![x, y], pt);
                        let o = call_intrinsic(cx, &mut s, &format!("llvm.aarch64.sve.uzp2.{m}"), vec![x, y], pt);
                        let poison = Value::Const(cx.m.consts.intern(Constant { ty, kind: ConstKind::Poison }));
                        let a = s.op(Op::InsertValue { aggregate: poison, element: e, indices: vec![0].into() }, ty);
                        let r = s.op(Op::InsertValue { aggregate: a, element: o, indices: vec![1].into() }, ty);
                        repl.insert(res.unwrap(), r);
                        out.extend(std::mem::take(&mut s.new));
                        continue;
                    }
                    if name.starts_with("llvm.vector.interleave2.") && res_split == Some(2) {
                        let pt = pt.unwrap();
                        let m = mangle(cx, pt);
                        let lo = call_intrinsic(cx, &mut s, &format!("llvm.aarch64.sve.zip1.{m}"), args.clone(), pt);
                        let hi = call_intrinsic(cx, &mut s, &format!("llvm.aarch64.sve.zip2.{m}"), args, pt);
                        split.insert(res.unwrap(), vec![lo, hi]);
                        out.extend(std::mem::take(&mut s.new));
                        continue;
                    }
                    // A partial reduction into one register: the parts
                    // added in.
                    if name.starts_with("llvm.vector.partial.reduce.add.") && res_split.is_none() {
                        let ps = parts_of(cx, &mut s, &split, args[1])?;
                        let ty = rty.unwrap();
                        if cx.value_ty(s.f, ps[0]) != ty {
                            return Err(format!("{name}: parts of another type than its result"));
                        }
                        let mut acc = args[0];
                        for x in ps {
                            acc = s.bin(BinOp::Add, acc, x, ty);
                        }
                        repl.insert(res.unwrap(), acc);
                        None
                    } else if let Some(rest) = name.strip_prefix("llvm.vector.reduce.")
                        && res_split.is_none()
                        && args.len() == 1
                    {
                        // A reduction: the parts combined, then one reduced.
                        let red = rest.split('.').next().unwrap_or("").to_string();
                        let ps = parts_of(cx, &mut s, &split, args[0])?;
                        let p = cx.value_ty(s.f, ps[0]);
                        let mut acc = ps[0];
                        for &x in &ps[1..] {
                            acc = match red.as_str() {
                                "add" => s.bin(BinOp::Add, acc, x, p),
                                "and" => s.bin(BinOp::And, acc, x, p),
                                "or" => s.bin(BinOp::Or, acc, x, p),
                                "xor" => s.bin(BinOp::Xor, acc, x, p),
                                "mul" => s.bin(BinOp::Mul, acc, x, p),
                                "smax" | "smin" | "umax" | "umin" => {
                                    let m = mangle(cx, p);
                                    call_intrinsic(cx, &mut s, &format!("llvm.{red}.{m}"), vec![acc, x], p)
                                }
                                _ => return Err(format!("{name} of a scalable vector of parts is not supported yet")),
                            };
                        }
                        let ty = rty.unwrap();
                        let m = mangle(cx, p);
                        let r = call_intrinsic(cx, &mut s, &format!("llvm.vector.reduce.{red}.{m}"), vec![acc], ty);
                        repl.insert(res.unwrap(), r);
                        None
                    } else if let Some(base) = ["smax", "smin", "umax", "umin", "abs"]
                        .iter()
                        .find(|b| name.starts_with(&format!("llvm.{b}.")))
                        && res_split.is_some()
                    {
                        // Lane-wise intrinsics: of each part.
                        let pt = pt.unwrap();
                        let m = mangle(cx, pt);
                        let mut per: Vec<Vec<Value>> = Vec::new();
                        for &a in &args {
                            let t = cx.value_ty(s.f, a);
                            per.push(if parts(cx, t).is_some() {
                                parts_of(cx, &mut s, &split, a)?
                            } else {
                                vec![a; res_split.unwrap()]
                            });
                        }
                        Some(
                            (0..res_split.unwrap())
                                .map(|k| {
                                    let xs = per.iter().map(|p| p[k]).collect();
                                    call_intrinsic(cx, &mut s, &format!("llvm.{base}.{m}"), xs, pt)
                                })
                                .collect(),
                        )
                    } else {
                        return Err(format!("{name} of scalable vectors of more than one register is not supported yet"));
                    }
                }
                _ => {
                    return Err(format!(
                        "a scalable vector of more than one register used by {}",
                        format!("{op:?}").split([' ', '(', '{']).next().unwrap_or("?")
                    ));
                }
            };
            if let Some(ps) = made {
                split.insert(res.unwrap(), ps);
            }
            out.extend(std::mem::take(&mut s.new));
        }
        f.blocks[b.index()].insts = out;
    }
    for (i, part_phis) in phis {
        let Op::Phi { incoming, .. } = f.insts[i.index()].op.clone() else { unreachable!() };
        let mut per: Vec<Vec<(Value, BlockId)>> = vec![Vec::new(); part_phis.len()];
        for (v, pred) in incoming {
            let v = match v {
                Value::Local(x) => repl.get(&x).copied().unwrap_or(v),
                v => v,
            };
            let mut s = Seq { f, new: Vec::new() };
            let ps = parts_of(cx, &mut s, &split, v)?;
            for (k, x) in ps.into_iter().enumerate() {
                per[k].push((x, pred));
            }
        }
        for (p, inc) in part_phis.into_iter().zip(per) {
            if let Op::Phi { incoming, .. } = &mut f.insts[p.index()].op {
                *incoming = inc;
            }
        }
    }
    for (x, v) in repl {
        f.replace_uses(Value::Local(x), v);
    }
    Ok(())
}

/// Scalable predicates extended to integers (`zext <vscale x 4 x i1>`):
/// one or minus one where on, zero elsewhere (`mov z.s, p/z, #1`).
pub fn predicate_casts(cx: &mut Ctx, f: &mut Function) {
    let i1 = cx.m.types.int(1);
    // A lane of a predicate (a tail-folded loop's test of its first): the
    // lane of it extended to integers (made a select below), truncated.
    for b in f.layout.clone() {
        let mut k = 0;
        while k < f.blocks[b.index()].insts.len() {
            let i = f.blocks[b.index()].insts[k];
            let Op::ExtractElement { vector, index } = f.insts[i.index()].op else {
                k += 1;
                continue;
            };
            let Type::Vector { len, elem, scalable: true } = *cx.m.types.get(cx.value_ty(f, vector)) else {
                k += 1;
                continue;
            };
            if elem != i1 {
                k += 1;
                continue;
            }
            let r = f.insts[i.index()].result.unwrap();
            let e = cx.m.types.int(PART_BITS / len);
            let it = cx.m.types.intern(Type::Vector { len, elem: e, scalable: true });
            let mut s = Seq { f, new: Vec::new() };
            let wide = s.cast(CastOp::ZExt, vector, it);
            let x = s.op(Op::ExtractElement { vector: wide, index }, e);
            let t = s.cast(CastOp::Trunc, x, i1);
            let new = std::mem::take(&mut s.new);
            f.replace_uses(Value::Local(r), t);
            let n = new.len();
            f.blocks[b.index()].insts.splice(k..=k, new);
            k += n;
        }
    }
    for b in f.layout.clone() {
        for &i in &f.blocks[b.index()].insts.clone() {
            let Op::Cast { op: op @ (CastOp::ZExt | CastOp::SExt), value, .. } = f.insts[i.index()].op else {
                continue;
            };
            let r = f.insts[i.index()].result.unwrap();
            let ty = f.locals[r.index()].ty;
            let from = cx.value_ty(f, value);
            if !matches!(*cx.m.types.get(from), Type::Vector { scalable: true, elem, .. } if elem == i1) {
                continue;
            }
            let on = splat_int(cx, ty, if op == CastOp::SExt { -1 } else { 1 });
            let zero = Value::Const(cx.m.consts.intern(Constant { ty, kind: ConstKind::Zero }));
            f.insts[i.index()].op =
                Op::Select { fmf: FastMath::default(), cond: value, then_value: on, else_value: zero };
        }
    }
}

/// Masked loads and stores of scalable vectors (`llvm.masked.load`) as
/// SVE's, which are predicated: `ld1`, whose inactive lanes are zero (the
/// passthru selected in for them, unless it is zero or poison), and `st1`.
/// (Of either form: with the alignment an argument, or not.)
pub fn masked_memory(cx: &mut Ctx, f: &mut Function) {
    for b in f.layout.clone() {
        let mut k = 0;
        while k < f.blocks[b.index()].insts.len() {
            let i = f.blocks[b.index()].insts[k];
            let Op::Call(call) = &f.insts[i.index()].op else {
                k += 1;
                continue;
            };
            let name = crate::prepare::callee_name(cx.m, call).unwrap_or_default();
            let args: Vec<Value> = call.args.iter().map(|a| a.value).collect();
            let res = f.insts[i.index()].result;
            let mut s = Seq { f, new: Vec::new() };
            let _ = &name;
            if name.starts_with("llvm.masked.store.") && args.len() >= 3 {
                let (v, p, m) = (args[0], args[1], *args.last().unwrap());
                let vt = cx.value_ty(s.f, v);
                if scalable(cx, vt).is_none() {
                    k += 1;
                    continue;
                }
                let void = cx.m.types.intern(Type::Void);
                let mn = mangle(cx, vt);
                call_intrinsic(cx, &mut s, &format!("llvm.aarch64.sve.st1.{mn}"), vec![v, m, p], void);
            } else if name.starts_with("llvm.masked.load.") && args.len() >= 3 && res.is_some() {
                let r = res.unwrap();
                let vt = s.f.locals[r.index()].ty;
                if scalable(cx, vt).is_none() {
                    k += 1;
                    continue;
                }
                let (p, m, pass) = if args.len() == 4 { (args[0], args[2], args[3]) } else { (args[0], args[1], args[2]) };
                let mn = mangle(cx, vt);
                let l = call_intrinsic(cx, &mut s, &format!("llvm.aarch64.sve.ld1.{mn}"), vec![m, p], vt);
                let l = if matches!(pass, Value::Const(c)
                    if matches!(cx.m.constant(c).kind, ConstKind::Zero | ConstKind::Poison | ConstKind::Undef))
                {
                    l
                } else {
                    s.op(Op::Select { fmf: FastMath::default(), cond: m, then_value: l, else_value: pass }, vt)
                };
                let new = std::mem::take(&mut s.new);
                f.replace_uses(Value::Local(r), l);
                let n = new.len();
                f.blocks[b.index()].insts.splice(k..=k, new);
                k += n;
                continue;
            } else {
                k += 1;
                continue;
            }
            let new = std::mem::take(&mut s.new);
            let n = new.len();
            f.blocks[b.index()].insts.splice(k..=k, new);
            k += n;
        }
    }
}

/// The container of an unpacked scalable vector of integers (`<vscale x 8
/// x i16>` for `<vscale x 8 x i8>`, whose bytes SVE keeps one in each
/// 16-bit lane): its lanes as wide as fill a register.
fn container(cx: &mut Ctx, ty: TypeId) -> Option<TypeId> {
    let (len, elem, eb) = scalable(cx, ty)?;
    if eb == 1 || len * eb >= PART_BITS || !matches!(cx.m.types.get(elem), Type::Int(_)) {
        return None;
    }
    let e = cx.m.types.int(PART_BITS / len);
    Some(cx.m.types.intern(Type::Vector { len, elem: e, scalable: true }))
}

/// A splat of the integer `v` of type `ty` (scalable).
fn splat_int(cx: &mut Ctx, ty: TypeId, v: i64) -> Value {
    let (_, elem, eb) = scalable(cx, ty).unwrap();
    let k = cx.m.consts.intern(Constant {
        ty: elem,
        kind: ConstKind::Int(epitome_ir::apint::ApInt::from_i64(eb, v)),
    });
    Value::Const(cx.m.consts.intern(Constant { ty, kind: ConstKind::Splat(k) }))
}

/// Unpacked scalable vectors of integers held in their containers: each
/// such value becomes one of its container's type, its lanes' upper bits
/// unknown, and each operation on it one on the container, extended
/// first where those bits matter (shifts right, divisions, comparisons,
/// extensions). Loads and stores of one are SVE's of its lanes
/// (`ld1b {z.h}`), `llvm.epitome.sve.ld1.nxv8i8`.
pub fn unpacked(cx: &mut Ctx, f: &mut Function) -> Result<(), String> {
    let mut any = false;
    for l in f.locals.clone() {
        any |= container(cx, l.ty).is_some();
    }
    if !any {
        return Ok(());
    }
    // Of each unpacked local, the container value that stands for it.
    let mut held: HashMap<LocalId, Value> = HashMap::new();
    let mut repl: HashMap<LocalId, Value> = HashMap::new();
    let mut blocks = crate::wide::reverse_postorder(f);
    for &b in &f.layout {
        if !blocks.contains(&b) {
            blocks.push(b);
        }
    }
    let mut phis: Vec<(InstId, InstId)> = Vec::new();
    for &b in &blocks {
        for &i in &f.blocks[b.index()].insts.clone() {
            if !f.insts[i.index()].op.is_phi() {
                break;
            }
            let r = f.insts[i.index()].result.unwrap();
            let Some(ct) = container(cx, f.locals[r.index()].ty) else { continue };
            let p = f.add_inst(Op::Phi { fmf: FastMath::default(), incoming: vec![] }, Some((None, ct)));
            held.insert(r, f.result(p));
            phis.push((i, p));
        }
    }
    // The container value of `v` (unpacked): made, or a constant's.
    fn hold(cx: &mut Ctx, s: &mut Seq, held: &HashMap<LocalId, Value>, v: Value) -> Result<Value, String> {
        let ty = cx.value_ty(s.f, v);
        let ct = container(cx, ty).ok_or("holding a packed vector")?;
        match v {
            Value::Local(x) => held.get(&x).copied().ok_or_else(|| "an unpacked vector not yet held".to_string()),
            Value::Const(c) => Ok(match cx.m.constant(c).kind.clone() {
                ConstKind::Splat(e) => {
                    let ConstKind::Int(a) = &cx.m.constant(e).kind else { return Err("an unpacked splat".into()) };
                    let bits = scalable(cx, ty).unwrap().2;
                    splat_int(cx, ct, crate::target::sext(a.low_u64(), bits))
                }
                k @ (ConstKind::Zero | ConstKind::Undef | ConstKind::Poison) => {
                    Value::Const(cx.m.consts.intern(Constant { ty: ct, kind: k }))
                }
                other => return Err(format!("an unpacked constant {other:?}")),
            }),
            _ => Err("metadata".into()),
        }
    }
    // `x`, a container of lanes of `bits`, with its lanes' upper bits
    // their extension.
    fn extended(cx: &mut Ctx, s: &mut Seq, x: Value, bits: u32, signed: bool) -> Value {
        let ct = cx.value_ty(s.f, x);
        let w = scalable(cx, ct).unwrap().2;
        if signed {
            let k = splat_int(cx, ct, i64::from(w - bits));
            let l = s.bin(BinOp::Shl, x, k, ct);
            s.bin(BinOp::AShr, l, k, ct)
        } else {
            let m = splat_int(cx, ct, ((1u64 << bits) - 1) as i64);
            s.bin(BinOp::And, x, m, ct)
        }
    }
    for b in blocks {
        let old = std::mem::take(&mut f.blocks[b.index()].insts);
        let mut out = Vec::new();
        for i in old {
            if let Some(&(_, p)) = phis.iter().find(|(q, _)| *q == i) {
                out.push(p);
                continue;
            }
            f.insts[i.index()].op.for_each_value_mut(|v| {
                if let Value::Local(x) = v
                    && let Some(&n) = repl.get(x)
                {
                    *v = n;
                }
            });
            let op = f.insts[i.index()].op.clone();
            let res = f.insts[i.index()].result;
            let rty = res.map(|r| f.locals[r.index()].ty);
            let rct = rty.and_then(|t| container(cx, t));
            let mut touches = false;
            op.for_each_value(|v| match v {
                Value::Local(x) => touches |= held.contains_key(x),
                Value::Const(c) => touches |= container(cx, cx.m.constant(*c).ty).is_some(),
                _ => {}
            });
            if rct.is_none() && !touches {
                out.push(i);
                continue;
            }
            let mut s = Seq { f, new: Vec::new() };
            let bits_of = |cx: &Ctx, s: &Seq, v: Value| scalable(cx, cx.value_ty(s.f, v)).map_or(0, |x| x.2);
            // The result: a container value (of an unpacked result), or a
            // value replacing a packed one.
            let made: Value = match op {
                Op::Load { atomic: None, ptr, .. } if rct.is_some() => {
                    let t = rty.unwrap();
                    let (len, _, _) = scalable(cx, t).unwrap();
                    let i1 = cx.m.types.int(1);
                    let mt = cx.m.types.intern(Type::Vector { len, elem: i1, scalable: true });
                    let on = splat_int(cx, mt, 1);
                    let m = mangle(cx, t);
                    call_intrinsic(cx, &mut s, &format!("llvm.epitome.sve.ld1.{m}"), vec![on, ptr], rct.unwrap())
                }
                Op::Store { atomic: None, value, ptr, .. } => {
                    let t = cx.value_ty(s.f, value);
                    let (len, _, _) = scalable(cx, t).ok_or("a store of a scalar")?;
                    let x = hold(cx, &mut s, &held, value)?;
                    let i1 = cx.m.types.int(1);
                    let mt = cx.m.types.intern(Type::Vector { len, elem: i1, scalable: true });
                    let on = splat_int(cx, mt, 1);
                    let m = mangle(cx, t);
                    let void = cx.m.types.intern(Type::Void);
                    call_intrinsic(cx, &mut s, &format!("llvm.epitome.sve.st1.{m}"), vec![x, on, ptr], void);
                    out.extend(std::mem::take(&mut s.new));
                    continue;
                }
                Op::Binary { op, flags, fmf, lhs, rhs } if rct.is_some() => {
                    let ct = rct.unwrap();
                    let bits = bits_of(cx, &s, lhs);
                    let (mut x, mut y) = (hold(cx, &mut s, &held, lhs)?, hold(cx, &mut s, &held, rhs)?);
                    let ext = match op {
                        BinOp::LShr | BinOp::UDiv | BinOp::URem => Some(false),
                        BinOp::AShr | BinOp::SDiv | BinOp::SRem => Some(true),
                        _ => None,
                    };
                    if let Some(signed) = ext {
                        x = extended(cx, &mut s, x, bits, signed);
                        if !matches!(op, BinOp::LShr | BinOp::AShr) {
                            y = extended(cx, &mut s, y, bits, signed);
                        }
                    }
                    let flags = if ext.is_none() { IntFlags::NONE } else { flags };
                    s.op(Op::Binary { op, flags, fmf, lhs: x, rhs: y }, ct)
                }
                Op::ICmp { pred, samesign, lhs, rhs } => {
                    let bits = bits_of(cx, &s, lhs);
                    let signed = matches!(pred, ICmpPred::Sgt | ICmpPred::Sge | ICmpPred::Slt | ICmpPred::Sle);
                    let x = hold(cx, &mut s, &held, lhs)?;
                    let y = hold(cx, &mut s, &held, rhs)?;
                    let (x, y) = (extended(cx, &mut s, x, bits, signed), extended(cx, &mut s, y, bits, signed));
                    s.op(Op::ICmp { pred, samesign, lhs: x, rhs: y }, rty.unwrap())
                }
                Op::Select { fmf, cond, then_value, else_value } if rct.is_some() => {
                    let x = hold(cx, &mut s, &held, then_value)?;
                    let y = hold(cx, &mut s, &held, else_value)?;
                    s.op(Op::Select { fmf, cond, then_value: x, else_value: y }, rct.unwrap())
                }
                Op::Freeze(v) if rct.is_some() => {
                    let x = hold(cx, &mut s, &held, v)?;
                    s.op(Op::Freeze(x), rct.unwrap())
                }
                // To an unpacked type: of the container, its lanes' low bits
                // (a truncation to the container, if wider).
                Op::Cast { op: CastOp::Trunc, value, .. } if rct.is_some() => {
                    let ct = rct.unwrap();
                    let from = cx.value_ty(s.f, value);
                    let x = if container(cx, from).is_some() { hold(cx, &mut s, &held, value)? } else { value };
                    let xt = cx.value_ty(s.f, x);
                    if xt == ct { x } else { s.cast(CastOp::Trunc, x, ct) }
                }
                // From one: extended in the container, then to the result's
                // type (or its container).
                Op::Cast { op: op @ (CastOp::SExt | CastOp::ZExt | CastOp::SIToFP | CastOp::UIToFP), value, .. }
                    if container(cx, cx.value_ty(s.f, value)).is_some() =>
                {
                    let bits = bits_of(cx, &s, value);
                    let signed = matches!(op, CastOp::SExt | CastOp::SIToFP);
                    let x = hold(cx, &mut s, &held, value)?;
                    let x = extended(cx, &mut s, x, bits, signed);
                    let to = rct.unwrap_or(rty.unwrap());
                    let xt = cx.value_ty(s.f, x);
                    let xb = bits_of(cx, &s, x);
                    match op {
                        CastOp::SIToFP | CastOp::UIToFP => s.cast(op, x, to),
                        _ if xt == to => x,
                        _ => {
                            let tb = scalable(cx, to).unwrap().2;
                            let c = if tb > xb { op } else { CastOp::Trunc };
                            s.cast(c, x, to)
                        }
                    }
                }
                Op::InsertElement { vector, element, index } if rct.is_some() => {
                    let ct = rct.unwrap();
                    let v = hold(cx, &mut s, &held, vector)?;
                    let e = scalable(cx, ct).unwrap().1;
                    let x = s.cast(CastOp::ZExt, element, e);
                    s.op(Op::InsertElement { vector: v, element: x, index }, ct)
                }
                Op::ShuffleVector { a, b: bv, mask } if rct.is_some() => {
                    let (x, y) = (hold(cx, &mut s, &held, a)?, hold(cx, &mut s, &held, bv)?);
                    s.op(Op::ShuffleVector { a: x, b: y, mask }, rct.unwrap())
                }
                // SVE's loads and stores of unpacked lanes (ld1b {z.d}, its
                // first-faulting, non-faulting and gathering forms, and the
                // scatters): llvm.epitome.sve's of the containers, the
                // offsets too (an sxtw's reads each lane's low 32 bits).
                Op::Call(ref call) => {
                    let name = crate::prepare::callee_name(cx.m, call).unwrap_or_default();
                    let m = name
                        .strip_prefix("llvm.aarch64.sve.")
                        .filter(|m| {
                            ["ld1.", "ldff1.", "ldnf1.", "ldnt1.", "st1.", "stnt1."].iter().any(|p| m.starts_with(p))
                        })
                        .ok_or_else(|| format!("{name} of unpacked scalable vectors is not supported yet"))?;
                    let mut args = Vec::new();
                    for a in &call.args {
                        let held_ty = container(cx, cx.value_ty(s.f, a.value)).is_some();
                        args.push(if held_ty { hold(cx, &mut s, &held, a.value)? } else { a.value });
                    }
                    let void = cx.m.types.intern(Type::Void);
                    let r =
                        call_intrinsic(cx, &mut s, &format!("llvm.epitome.sve.{m}"), args, rct.or(rty).unwrap_or(void));
                    if res.is_none() {
                        out.extend(std::mem::take(&mut s.new));
                        continue;
                    }
                    r
                }
                _ => {
                    return Err(format!(
                        "an unpacked scalable vector used by {}",
                        format!("{op:?}").split([' ', '(', '{']).next().unwrap_or("?")
                    ));
                }
            };
            match rct {
                Some(_) => {
                    held.insert(res.unwrap(), made);
                }
                None => {
                    repl.insert(res.unwrap(), made);
                }
            }
            out.extend(std::mem::take(&mut s.new));
        }
        f.blocks[b.index()].insts = out;
    }
    for (i, p) in phis {
        let Op::Phi { incoming, .. } = f.insts[i.index()].op.clone() else { unreachable!() };
        let mut inc = Vec::new();
        for (v, pred) in incoming {
            let v = match v {
                Value::Local(x) => repl.get(&x).copied().unwrap_or(v),
                v => v,
            };
            let mut s = Seq { f, new: Vec::new() };
            inc.push((hold(cx, &mut s, &held, v)?, pred));
        }
        if let Op::Phi { incoming, .. } = &mut f.insts[p.index()].op {
            *incoming = inc;
        }
    }
    for (x, v) in repl {
        f.replace_uses(Value::Local(x), v);
    }
    Ok(())
}
