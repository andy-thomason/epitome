//! Vectors of integers wider than a register (`<8 x i128>`, which a
//! vectorised bit test of a 128-bit mask makes), taken apart into their
//! lanes before `wide.rs` splits each lane into halves: each such vector
//! value becomes a value for each lane, and each operation on it one on
//! each lane.
//!
//! Vectors of narrow lanes that meet them (a `zext` from `<8 x i8>`, an
//! `icmp`'s mask) are taken apart, or made again, at the edge.

use std::collections::HashMap;

use epitome_ir::module::*;
use epitome_ir::types::{Type, TypeId};

use crate::prepare::{Ctx, Seq};

/// The lane count and lane type of a vector of wide integers.
fn wide_vector(cx: &Ctx, ty: TypeId) -> Option<(u32, TypeId)> {
    match *cx.m.types.get(ty) {
        Type::Vector { len, elem, scalable: false } if matches!(cx.m.types.get(elem), Type::Int(b) if *b > 64) => {
            Some((len, elem))
        }
        _ => None,
    }
}

/// The lanes of `v`: those already taken apart, a constant's, or
/// extractelements of a vector of narrow lanes.
fn lanes_of(cx: &mut Ctx, s: &mut Seq, split: &HashMap<LocalId, Vec<Value>>, v: Value) -> Result<Vec<Value>, String> {
    let ty = cx.value_ty(s.f, v);
    let Type::Vector { len, elem, .. } = *cx.m.types.get(ty) else { return Err("lanes of a scalar".into()) };
    match v {
        Value::Local(x) if split.contains_key(&x) => Ok(split[&x].clone()),
        Value::Const(c) => {
            let k = cx.m.constant(c).kind.clone();
            Ok((0..len as usize)
                .map(|l| {
                    let kind = match &k {
                        ConstKind::Vector(es) => cx.m.constant(es[l]).kind.clone(),
                        ConstKind::Splat(e) => cx.m.constant(*e).kind.clone(),
                        other => other.clone(),
                    };
                    Value::Const(cx.m.consts.intern(Constant { ty: elem, kind }))
                })
                .collect())
        }
        _ => {
            let i64t = cx.m.types.int(64);
            Ok((0..len)
                .map(|l| {
                    let idx = cx.int(i64t, i64::from(l));
                    s.op(Op::ExtractElement { vector: v, index: idx }, elem)
                })
                .collect())
        }
    }
}

/// A vector of narrow lanes made of `lanes`.
fn build(cx: &mut Ctx, s: &mut Seq, ty: TypeId, lanes: &[Value]) -> Value {
    let i64t = cx.m.types.int(64);
    let mut v = Value::Const(cx.m.consts.intern(Constant { ty, kind: ConstKind::Poison }));
    for (l, &x) in lanes.iter().enumerate() {
        let idx = cx.int(i64t, l as i64);
        v = s.op(Op::InsertElement { vector: v, element: x, index: idx }, ty);
    }
    v
}

pub fn scalarize(cx: &mut Ctx, f: &mut Function) -> Result<(), String> {
    if !f.locals.iter().any(|l| wide_vector(cx, l.ty).is_some()) {
        return Ok(());
    }
    let mut split: HashMap<LocalId, Vec<Value>> = HashMap::new();
    // Values replaced, applied as each instruction is reached (the block
    // being rewritten is out of the function, where `replace_uses` would
    // not see it).
    let mut repl: HashMap<LocalId, Value> = HashMap::new();
    let mut blocks = crate::wide::reverse_postorder(f);
    for &b in &f.layout {
        if !blocks.contains(&b) {
            blocks.push(b);
        }
    }
    // Phis first, a phi of each lane, so that values may come round loops;
    // their incoming lanes are filled in at the end.
    let mut phis: Vec<(InstId, Vec<InstId>)> = Vec::new();
    for &b in &blocks {
        for &i in &f.blocks[b.index()].insts.clone() {
            if !f.insts[i.index()].op.is_phi() {
                break;
            }
            let r = f.insts[i.index()].result.unwrap();
            let Some((len, e)) = wide_vector(cx, f.locals[r.index()].ty) else { continue };
            let lane_phis: Vec<InstId> = (0..len)
                .map(|_| f.add_inst(Op::Phi { fmf: FastMath::default(), incoming: vec![] }, Some((None, e))))
                .collect();
            split.insert(r, lane_phis.iter().map(|&p| f.result(p)).collect());
            phis.push((i, lane_phis));
        }
    }
    for b in blocks {
        let old = std::mem::take(&mut f.blocks[b.index()].insts);
        let mut out = Vec::new();
        for i in old {
            if let Some((_, lane_phis)) = phis.iter().find(|(p, _)| *p == i) {
                out.extend(lane_phis.iter().copied());
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
            let res_wide = rty.and_then(|t| wide_vector(cx, t));
            let mut wide_operand = false;
            op.for_each_value(|v| {
                if let Value::Local(x) = v {
                    wide_operand |= split.contains_key(x);
                }
            });
            if res_wide.is_none() && !wide_operand {
                out.push(i);
                continue;
            }
            let mut s = Seq { f, new: Vec::new() };
            // The lanes of the result, of its lane type.
            let lane_ty = res_wide.map(|(_, e)| e).or_else(|| match rty.map(|t| cx.m.types.get(t).clone()) {
                Some(Type::Vector { elem, .. }) => Some(elem),
                _ => None,
            });
            let lanes: Option<Vec<Value>> = match op {
                // Loads and stores of one: of each lane, in turn.
                Op::Load { volatile, atomic: None, ptr, align } if res_wide.is_some() => {
                    let (len, e) = res_wide.unwrap();
                    let size = cx.size_of(e)? as i64;
                    let (i8t, i64t, pt) = (cx.m.types.int(8), cx.m.types.int(64), cx.m.types.ptr(0));
                    Some(
                        (0..len)
                            .map(|l| {
                                let at = if l == 0 {
                                    ptr
                                } else {
                                    let off = cx.int(i64t, i64::from(l) * size);
                                    s.op(
                                        Op::Gep {
                                            flags: GepFlags::default(),
                                            source_ty: i8t,
                                            ptr,
                                            indices: vec![off].into(),
                                        },
                                        pt,
                                    )
                                };
                                let a = align.map(|a| a.at_most(size as u64));
                                s.op(Op::Load { volatile, atomic: None, ptr: at, align: a }, e)
                            })
                            .collect(),
                    )
                }
                Op::Store { volatile, atomic: None, value, ptr, align } => {
                    let ls = lanes_of(cx, &mut s, &split, value)?;
                    let e = cx.value_ty(s.f, ls[0]);
                    let size = cx.size_of(e)? as i64;
                    let (i8t, i64t, pt) = (cx.m.types.int(8), cx.m.types.int(64), cx.m.types.ptr(0));
                    for (l, x) in ls.into_iter().enumerate() {
                        let at = if l == 0 {
                            ptr
                        } else {
                            let off = cx.int(i64t, l as i64 * size);
                            s.op(
                                Op::Gep { flags: GepFlags::default(), source_ty: i8t, ptr, indices: vec![off].into() },
                                pt,
                            )
                        };
                        let a = align.map(|a| a.at_most(size as u64));
                        s.op_void(Op::Store { volatile, atomic: None, value: x, ptr: at, align: a });
                    }
                    out.extend(std::mem::take(&mut s.new));
                    continue;
                }
                Op::InsertElement { vector, element, index } => {
                    let mut ls = lanes_of(cx, &mut s, &split, vector)?;
                    let k = cx.const_int(index).ok_or("a lane of a wide vector chosen by a variable")?;
                    ls[k as usize] = element;
                    Some(ls)
                }
                Op::ExtractElement { vector, index } => {
                    let ls = lanes_of(cx, &mut s, &split, vector)?;
                    let k = cx.const_int(index).ok_or("a lane of a wide vector chosen by a variable")?;
                    repl.insert(res.unwrap(), ls[k as usize]);
                    None
                }
                Op::ShuffleVector { a, b: bv, mask } => {
                    let la = lanes_of(cx, &mut s, &split, a)?;
                    let lb = lanes_of(cx, &mut s, &split, bv)?;
                    let m = crate::target::mask_lanes(cx.m, mask).ok_or("a shuffle mask")?;
                    let n = la.len() as i32;
                    let e = lane_ty.unwrap();
                    let poison = Value::Const(cx.m.consts.intern(Constant { ty: e, kind: ConstKind::Poison }));
                    Some(
                        m.iter()
                            .map(|&k| match k {
                                k if k < 0 => poison,
                                k if k < n => la[k as usize],
                                k => lb[(k - n) as usize],
                            })
                            .collect(),
                    )
                }
                Op::Binary { op, flags, fmf, lhs, rhs } => {
                    let l = lanes_of(cx, &mut s, &split, lhs)?;
                    let r = lanes_of(cx, &mut s, &split, rhs)?;
                    let e = lane_ty.unwrap();
                    Some(
                        l.into_iter()
                            .zip(r)
                            .map(|(x, y)| s.op(Op::Binary { op, flags, fmf, lhs: x, rhs: y }, e))
                            .collect(),
                    )
                }
                Op::Cast { op, flags, fmf, value } => {
                    let l = lanes_of(cx, &mut s, &split, value)?;
                    let e = lane_ty.unwrap();
                    Some(l.into_iter().map(|x| s.op(Op::Cast { op, flags, fmf, value: x }, e)).collect())
                }
                Op::ICmp { pred, samesign, lhs, rhs } => {
                    let l = lanes_of(cx, &mut s, &split, lhs)?;
                    let r = lanes_of(cx, &mut s, &split, rhs)?;
                    let e = lane_ty.unwrap();
                    Some(
                        l.into_iter()
                            .zip(r)
                            .map(|(x, y)| s.op(Op::ICmp { pred, samesign, lhs: x, rhs: y }, e))
                            .collect(),
                    )
                }
                Op::Select { fmf, cond, then_value, else_value } => {
                    let t = lanes_of(cx, &mut s, &split, then_value)?;
                    let el = lanes_of(cx, &mut s, &split, else_value)?;
                    let n = t.len();
                    let c = if matches!(cx.m.types.get(cx.value_ty(s.f, cond)), Type::Vector { .. }) {
                        lanes_of(cx, &mut s, &split, cond)?
                    } else {
                        vec![cond; n]
                    };
                    let e = lane_ty.unwrap();
                    Some(
                        (0..n)
                            .map(|k| s.op(Op::Select { fmf, cond: c[k], then_value: t[k], else_value: el[k] }, e))
                            .collect(),
                    )
                }
                Op::Freeze(v) => {
                    let l = lanes_of(cx, &mut s, &split, v)?;
                    let e = lane_ty.unwrap();
                    Some(l.into_iter().map(|x| s.op(Op::Freeze(x), e)).collect())
                }
                Op::Call(ref call)
                    if crate::prepare::callee_name(cx.m, call)
                        .is_some_and(|n| n.starts_with("llvm.vector.reduce.")) =>
                {
                    // A reduction: the lanes combined in order.
                    let name = crate::prepare::callee_name(cx.m, call).unwrap();
                    let kind =
                        name.trim_start_matches("llvm.vector.reduce.").split('.').next().unwrap_or("").to_string();
                    let bop = match kind.as_str() {
                        "or" => BinOp::Or,
                        "and" => BinOp::And,
                        "xor" => BinOp::Xor,
                        "add" => BinOp::Add,
                        "mul" => BinOp::Mul,
                        _ => return Err(format!("{name} of a wide vector is not supported yet")),
                    };
                    let l = lanes_of(cx, &mut s, &split, call.args[0].value)?;
                    let r = res.unwrap();
                    let ty = s.f.locals[r.index()].ty;
                    let mut acc = l[0];
                    for &x in &l[1..] {
                        acc = s.bin(bop, acc, x, ty);
                    }
                    repl.insert(r, acc);
                    None
                }
                _ => {
                    // Anything else given one: the vector made again, which
                    // works only for narrow lanes.
                    return Err(format!(
                        "a vector of wide integers used by {}",
                        format!("{op:?}").split([' ', '(', '{']).next().unwrap_or("?")
                    ));
                }
            };
            if let Some(ls) = lanes {
                let r = res.unwrap();
                if res_wide.is_some() {
                    split.insert(r, ls);
                } else {
                    // Narrow lanes: a vector again; a mask, as one a
                    // comparison makes (of integer lanes with zero), so
                    // that its lanes have a width.
                    let ty = rty.unwrap();
                    let v = match *cx.m.types.get(ty) {
                        Type::Vector { len, elem, scalable: false } if matches!(cx.m.types.get(elem), Type::Int(1)) => {
                            let w = (128 / len).clamp(8, 64);
                            let e = cx.m.types.int(w);
                            let it = cx.m.types.intern(Type::Vector { len, elem: e, scalable: false });
                            let wide: Vec<Value> = ls.iter().map(|&x| s.cast(CastOp::SExt, x, e)).collect();
                            let iv = build(cx, &mut s, it, &wide);
                            let zero = Value::Const(cx.m.consts.intern(Constant { ty: it, kind: ConstKind::Zero }));
                            s.op(Op::ICmp { pred: ICmpPred::Ne, samesign: false, lhs: iv, rhs: zero }, ty)
                        }
                        _ => build(cx, &mut s, ty, &ls),
                    };
                    repl.insert(r, v);
                }
            }
            out.extend(std::mem::take(&mut s.new));
        }
        f.blocks[b.index()].insts = out;
    }
    // The lane phis' incoming lanes: of each incoming vector, taken at the
    // end of its block if it was left whole.
    for (i, lane_phis) in phis {
        let Op::Phi { incoming, .. } = f.insts[i.index()].op.clone() else { unreachable!() };
        let mut per: Vec<Vec<(Value, BlockId)>> = vec![Vec::new(); lane_phis.len()];
        for (v, pred) in incoming {
            let v = match v {
                Value::Local(x) => repl.get(&x).copied().unwrap_or(v),
                v => v,
            };
            let mut s = Seq { f, new: Vec::new() };
            let ls = lanes_of(cx, &mut s, &split, v)?;
            let new = std::mem::take(&mut s.new);
            if !new.is_empty() {
                let n = f.blocks[pred.index()].insts.len();
                f.blocks[pred.index()].insts.splice(n - 1..n - 1, new);
            }
            for (l, x) in ls.into_iter().enumerate() {
                per[l].push((x, pred));
            }
        }
        for (p, inc) in lane_phis.into_iter().zip(per) {
            f.insts[p.index()].op = Op::Phi { fmf: FastMath::default(), incoming: inc };
        }
    }
    for (x, v) in repl {
        f.replace_uses(Value::Local(x), v);
    }
    Ok(())
}
