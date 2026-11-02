//! Aggregate values (`{ ptr, i64 }`, `{ { ptr, ptr } }`, a named struct,
//! `[2 x { i32, i32 }]`), as their scalar leaves: selection takes
//! aggregates only as `insertvalue`s make them and calls return them, so
//! each such value becomes a value for each leaf, and each operation on it
//! operations on the leaves.
//!
//! - A load of one is a load of each leaf, at its offset (a volatile load
//!   is a volatile load of each); a store, a store of each.
//! - A phi, select or freeze of one is one of each leaf.
//! - `extractvalue` takes leaves; `insertvalue` puts them in.
//! - A constant's leaves are constants.
//! - Anything else given one (a call's argument, a return) gets it made
//!   again, by `insertvalue`s of one index, just before.
//!
//! Values made by what this leaves alone (a call's result, a landing pad)
//! stay aggregates; their leaves are taken by `extractvalue`s of one index
//! where a phi or another leaf-wise operation needs them.

use std::collections::HashMap;

use epitome_ir::module::*;
use epitome_ir::types::{Type, TypeId};

use crate::prepare::{Ctx, Seq};

/// A scalar inside an aggregate: the indices to it, its type, and its
/// offset in bytes.
#[derive(Clone, Debug)]
struct Leaf {
    path: Vec<u32>,
    ty: TypeId,
    offset: u64,
}

/// The most leaves an aggregate is split into.
const MAX_LEAVES: usize = 64;

/// The fields of a struct (literal or named) or array type, and their
/// offsets.
fn members(cx: &Ctx, ty: TypeId) -> Option<Vec<(TypeId, u64)>> {
    match cx.m.types.get(ty).clone() {
        Type::Struct { fields, packed } => {
            let l = cx.dl.struct_layout(&cx.m.types, &fields, packed)?;
            Some(fields.into_iter().zip(l.offsets).collect())
        }
        Type::Named(i) => {
            let body = cx.m.types.structs[i as usize].body.clone()?;
            let l = cx.dl.struct_layout(&cx.m.types, &body.fields, body.packed)?;
            Some(body.fields.into_iter().zip(l.offsets).collect())
        }
        Type::Array { len, elem } if len as usize <= MAX_LEAVES => {
            let size = cx.dl.alloc_size(&cx.m.types, elem)?;
            Some((0..len).map(|k| (elem, k * size)).collect())
        }
        _ => None,
    }
}

/// A leaf: a scalar, or a vector of up to a register's width (a member of
/// a homogeneous aggregate, `[2 x <4 x float>]`).
fn is_scalar(cx: &Ctx, ty: TypeId) -> bool {
    match cx.m.types.get(ty) {
        // (A 128-bit integer too, which `wide.rs` splits after.)
        Type::Int(1..=64 | 128) | Type::Ptr(0) | Type::Float | Type::Double => true,
        Type::Vector { scalable: false, .. } => cx.size_of(ty).is_ok_and(|b| b <= 16),
        _ => false,
    }
}

/// The leaves of `ty`, if it is an aggregate of scalars, not too many.
fn leaves(cx: &Ctx, ty: TypeId) -> Option<Vec<Leaf>> {
    fn walk(cx: &Ctx, ty: TypeId, path: &mut Vec<u32>, at: u64, out: &mut Vec<Leaf>) -> Option<()> {
        if is_scalar(cx, ty) {
            out.push(Leaf { path: path.clone(), ty, offset: at });
            return (out.len() <= MAX_LEAVES).then_some(());
        }
        for (k, (t, off)) in members(cx, ty)?.into_iter().enumerate() {
            path.push(k as u32);
            walk(cx, t, path, at + off, out)?;
            path.pop();
        }
        Some(())
    }
    let mut out = Vec::new();
    walk(cx, ty, &mut Vec::new(), 0, &mut out)?;
    Some(out)
}

/// An aggregate this pass splits: any of scalar leaves, not too many,
/// but the pairs of 64-bit halves wide integers become (see `wide.rs`),
/// which selection keeps in two registers throughout. (Selection takes
/// flat ones where they are made by `insertvalue`s, a call or a landing
/// pad, and taken apart by `extractvalue`s; a load or a phi of one it does
/// not.)
fn complex(cx: &Ctx, ty: TypeId) -> bool {
    let halves = matches!(cx.m.types.get(ty), Type::Array { len: 1..=2, elem } if matches!(cx.m.types.get(*elem), Type::Int(64)));
    !halves && members(cx, ty).is_some() && leaves(cx, ty).is_some()
}

/// The constant at `path` inside constant `c` (of type `ty`).
fn const_at(cx: &mut Ctx, c: ConstId, path: &[u32], leaf_ty: TypeId) -> ConstId {
    let mut cur = c;
    for (n, &k) in path.iter().enumerate() {
        match cx.m.constant(cur).kind.clone() {
            ConstKind::Struct(es) | ConstKind::Array(es) => cur = es[k as usize],
            kind @ (ConstKind::Zero | ConstKind::Undef | ConstKind::Poison) => {
                let _ = n;
                return cx.m.consts.intern(Constant { ty: leaf_ty, kind });
            }
            _ => return cx.m.consts.intern(Constant { ty: leaf_ty, kind: ConstKind::Undef }),
        }
    }
    cur
}

/// The value made again of its leaves, by insertvalues of one index.
fn rebuild(cx: &mut Ctx, s: &mut Seq, ty: TypeId, vals: &[Value], ls: &[Leaf], depth: usize) -> Value {
    let mut agg = Value::Const(cx.m.consts.intern(Constant { ty, kind: ConstKind::Poison }));
    let ms = members(cx, ty).unwrap_or_default();
    for (k, (t, _)) in ms.into_iter().enumerate() {
        let idx: Vec<usize> =
            (0..ls.len()).filter(|&j| ls[j].path.len() > depth && ls[j].path[depth] == k as u32).collect();
        let part = if is_scalar(cx, t) {
            vals[idx[0]]
        } else {
            let sub_vals: Vec<Value> = idx.iter().map(|&j| vals[j]).collect();
            let sub_ls: Vec<Leaf> = idx.iter().map(|&j| ls[j].clone()).collect();
            rebuild(cx, s, t, &sub_vals, &sub_ls, depth + 1)
        };
        agg = s.op(Op::InsertValue { aggregate: agg, element: part, indices: vec![k as u32].into() }, ty);
    }
    agg
}

/// The leaves of `v` (of type `ty`): from those already split, a
/// constant's, or extractvalues of one index of an aggregate left whole.
fn leaves_of(
    cx: &mut Ctx,
    s: &mut Seq,
    split: &HashMap<LocalId, Vec<Value>>,
    v: Value,
    ty: TypeId,
) -> Result<Vec<Value>, String> {
    let ls = leaves(cx, ty).ok_or("an aggregate of other than scalars")?;
    match v {
        Value::Local(x) if split.contains_key(&x) => Ok(split[&x].clone()),
        Value::Const(c) => Ok(ls.iter().map(|l| Value::Const(const_at(cx, c, &l.path, l.ty))).collect()),
        _ => {
            let mut out = Vec::new();
            for l in &ls {
                let mut cur = v;
                let mut cty = ty;
                for &k in &l.path {
                    let t = cx.m.types.member(cty, u64::from(k)).ok_or("a bad path")?;
                    cur = s.op(Op::ExtractValue { aggregate: cur, indices: vec![k].into() }, t);
                    cty = t;
                }
                out.push(cur);
            }
            Ok(out)
        }
    }
}

pub fn scalarize(cx: &mut Ctx, f: &mut Function) -> Result<(), String> {
    let any = f.locals.iter().any(|l| complex(cx, l.ty));
    if !any {
        return Ok(());
    }
    // An invoke of an aggregate this splits gets a block of its own on its
    // normal edge, where the result's leaves can be taken (for a phi
    // there, which takes them at the end of the block it comes from).
    for b in f.layout.clone() {
        let Some(t) = f.terminator(b) else { continue };
        let Op::Invoke { normal, .. } = f.insts[t.index()].op else { continue };
        if !f.insts[t.index()].result.is_some_and(|r| complex(cx, f.locals[r.index()].ty)) {
            continue;
        }
        let e = f.add_block(None);
        let at = f.layout.iter().position(|&x| x == b).unwrap();
        f.layout.insert(at + 1, e);
        f.push_inst(e, Op::Br(normal), None);
        if let Op::Invoke { normal: n, .. } = &mut f.insts[t.index()].op {
            *n = e;
        }
        f.retarget_phis(normal, b, e);
    }
    let i8t = cx.m.types.int(8);
    let i64t = cx.m.types.int(64);
    let ptr = cx.m.types.ptr(0);
    let mut split: HashMap<LocalId, Vec<Value>> = HashMap::new();
    // Scalars replaced (an extractvalue by the leaf it takes), applied to
    // each instruction as it is reached: the block being rewritten is out
    // of the function while it is, where `replace_uses` would not see it.
    let mut repl: HashMap<LocalId, Value> = HashMap::new();
    // Phis first, a phi of each leaf, so that values may come round loops.
    let mut phis: Vec<(InstId, Vec<InstId>)> = Vec::new();
    for b in f.layout.clone() {
        for &i in f.blocks[b.index()].insts.clone().iter() {
            if !f.insts[i.index()].op.is_phi() {
                break;
            }
            let r = f.insts[i.index()].result.unwrap();
            let ty = f.locals[r.index()].ty;
            if !complex(cx, ty) {
                continue;
            }
            let ls = leaves(cx, ty).unwrap();
            let new: Vec<InstId> = ls
                .iter()
                .map(|l| f.add_inst(Op::Phi { fmf: FastMath::default(), incoming: vec![] }, Some((None, l.ty))))
                .collect();
            split.insert(r, new.iter().map(|&p| f.result(p)).collect());
            phis.push((i, new));
        }
    }
    // Definitions before uses; unreachable blocks after.
    let mut blocks = crate::wide::reverse_postorder(f);
    for &b in &f.layout {
        if !blocks.contains(&b) {
            blocks.push(b);
        }
    }
    for b in blocks {
        let old = std::mem::take(&mut f.blocks[b.index()].insts);
        let mut out = Vec::new();
        for i in old {
            if let Some((_, leaf_phis)) = phis.iter().find(|p| p.0 == i) {
                out.extend(leaf_phis.iter().copied());
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
            let res_ty = res.map(|r| f.locals[r.index()].ty);
            let res_complex = res_ty.is_some_and(|t| complex(cx, t));
            let mut s = Seq { f, new: Vec::new() };
            let mut keep = true;
            match op {
                Op::Load { volatile, atomic: None, ptr: p, align } if res_complex => {
                    let ty = res_ty.unwrap();
                    let mut vals = Vec::new();
                    for l in leaves(cx, ty).unwrap() {
                        let at = if l.offset == 0 {
                            p
                        } else {
                            let off = cx.int(i64t, l.offset as i64);
                            s.op(
                                Op::Gep {
                                    flags: GepFlags::default(),
                                    source_ty: i8t,
                                    ptr: p,
                                    indices: vec![off].into(),
                                },
                                ptr,
                            )
                        };
                        let a =
                            align.map(|a| if l.offset == 0 { a } else { a.at_most(1 << l.offset.trailing_zeros()) });
                        vals.push(s.op(Op::Load { volatile, atomic: None, ptr: at, align: a }, l.ty));
                    }
                    split.insert(res.unwrap(), vals);
                    keep = false;
                }
                Op::Store { volatile, atomic: None, value, ptr: p, align } if complex(cx, cx.value_ty(s.f, value)) => {
                    let ty = cx.value_ty(s.f, value);
                    let vals = leaves_of(cx, &mut s, &split, value, ty)?;
                    for (l, v) in leaves(cx, ty).unwrap().into_iter().zip(vals) {
                        let at = if l.offset == 0 {
                            p
                        } else {
                            let off = cx.int(i64t, l.offset as i64);
                            s.op(
                                Op::Gep {
                                    flags: GepFlags::default(),
                                    source_ty: i8t,
                                    ptr: p,
                                    indices: vec![off].into(),
                                },
                                ptr,
                            )
                        };
                        let a =
                            align.map(|a| if l.offset == 0 { a } else { a.at_most(1 << l.offset.trailing_zeros()) });
                        s.op_void(Op::Store { volatile, atomic: None, value: v, ptr: at, align: a });
                    }
                    keep = false;
                }
                Op::Select { fmf, cond, then_value, else_value } if res_complex => {
                    let ty = res_ty.unwrap();
                    let a = leaves_of(cx, &mut s, &split, then_value, ty)?;
                    let c = leaves_of(cx, &mut s, &split, else_value, ty)?;
                    let vals = leaves(cx, ty)
                        .unwrap()
                        .iter()
                        .zip(a.iter().zip(c))
                        .map(|(l, (&x, y))| s.op(Op::Select { fmf, cond, then_value: x, else_value: y }, l.ty))
                        .collect();
                    split.insert(res.unwrap(), vals);
                    keep = false;
                }
                Op::Freeze(v) if res_complex => {
                    let ty = res_ty.unwrap();
                    let a = leaves_of(cx, &mut s, &split, v, ty)?;
                    let vals = leaves(cx, ty).unwrap().iter().zip(a).map(|(l, x)| s.op(Op::Freeze(x), l.ty)).collect();
                    split.insert(res.unwrap(), vals);
                    keep = false;
                }
                Op::ExtractValue { aggregate, indices } if complex(cx, cx.value_ty(s.f, aggregate)) => {
                    let aty = cx.value_ty(s.f, aggregate);
                    let all = leaves(cx, aty).unwrap();
                    let vals = leaves_of(cx, &mut s, &split, aggregate, aty)?;
                    let picked: Vec<usize> = (0..all.len()).filter(|&j| all[j].path.starts_with(&indices)).collect();
                    let r = res.unwrap();
                    let rty = res_ty.unwrap();
                    if is_scalar(cx, rty) && picked.len() == 1 {
                        repl.insert(r, vals[picked[0]]);
                    } else if complex(cx, rty) {
                        split.insert(r, picked.iter().map(|&j| vals[j]).collect());
                    } else {
                        // A flat struct, made again of its fields.
                        let sub: Vec<Leaf> = picked
                            .iter()
                            .map(|&j| Leaf {
                                path: all[j].path[indices.len()..].to_vec(),
                                ty: all[j].ty,
                                offset: all[j].offset,
                            })
                            .collect();
                        let sv: Vec<Value> = picked.iter().map(|&j| vals[j]).collect();
                        let v = rebuild(cx, &mut s, rty, &sv, &sub, 0);
                        repl.insert(r, v);
                    }
                    keep = false;
                }
                Op::InsertValue { aggregate, element, indices } if res_complex => {
                    let ty = res_ty.unwrap();
                    let all = leaves(cx, ty).unwrap();
                    let mut vals = leaves_of(cx, &mut s, &split, aggregate, ty)?;
                    let ety = cx.value_ty(s.f, element);
                    let picked: Vec<usize> = (0..all.len()).filter(|&j| all[j].path.starts_with(&indices)).collect();
                    let evals =
                        if is_scalar(cx, ety) { vec![element] } else { leaves_of(cx, &mut s, &split, element, ety)? };
                    for (&j, v) in picked.iter().zip(evals) {
                        vals[j] = v;
                    }
                    split.insert(res.unwrap(), vals);
                    keep = false;
                }
                mut op => {
                    // Anything else given a split value: the aggregate made
                    // again, just before.
                    let mut rebuilt: Vec<(Value, Value)> = Vec::new();
                    let mut uses = Vec::new();
                    op.for_each_value(|v| {
                        if let Value::Local(x) = v
                            && split.contains_key(x)
                        {
                            uses.push(*x);
                        }
                    });
                    for x in uses {
                        if rebuilt.iter().any(|(o, _)| *o == Value::Local(x)) {
                            continue;
                        }
                        let ty = s.f.locals[x.index()].ty;
                        let ls = leaves(cx, ty).unwrap();
                        let v = rebuild(cx, &mut s, ty, &split[&x].clone(), &ls, 0);
                        rebuilt.push((Value::Local(x), v));
                    }
                    if !rebuilt.is_empty() {
                        op.for_each_value_mut(|v| {
                            if let Some((_, n)) = rebuilt.iter().find(|(o, _)| o == v) {
                                *v = *n;
                            }
                        });
                        s.f.insts[i.index()].op = op;
                    }
                }
            }
            out.extend(std::mem::take(&mut s.new));
            if keep {
                out.push(i);
            }
        }
        f.blocks[b.index()].insts = out;
    }
    // The leaf phis' incoming values: leaves of each incoming one, taken
    // at the end of its block if it was left whole.
    for (i, leaf_phis) in phis {
        let Op::Phi { incoming, .. } = f.insts[i.index()].op.clone() else { unreachable!() };
        let ty = f.locals[f.insts[i.index()].result.unwrap().index()].ty;
        let mut per: Vec<Vec<(Value, BlockId)>> = vec![Vec::new(); leaf_phis.len()];
        for (v, pred) in incoming {
            let v = match v {
                Value::Local(x) => repl.get(&x).copied().unwrap_or(v),
                v => v,
            };
            let mut s = Seq { f, new: Vec::new() };
            let vals = leaves_of(cx, &mut s, &split, v, ty)?;
            let new = std::mem::take(&mut s.new);
            if !new.is_empty() {
                let insts = &f.blocks[pred.index()].insts;
                let term = *insts.last().unwrap();
                if f.insts[term.index()].result.map(Value::Local) == Some(v) {
                    return Err("a phi of an aggregate an invoke returns".into());
                }
                let n = insts.len();
                f.blocks[pred.index()].insts.splice(n - 1..n - 1, new);
            }
            for (k, x) in vals.into_iter().enumerate() {
                per[k].push((x, pred));
            }
        }
        for (p, inc) in leaf_phis.into_iter().zip(per) {
            f.insts[p.index()].op = Op::Phi { fmf: FastMath::default(), incoming: inc };
        }
    }
    // Uses the walk did not reach (other phis' operands, which it does
    // not rewrite).
    for (x, v) in repl {
        f.replace_uses(Value::Local(x), v);
    }
    Ok(())
}
