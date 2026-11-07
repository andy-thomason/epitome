//! What the machine does by calling its runtime, as the description says
//! (`docs/design/targets.md`, "Runtime calls"):
//!
//! - A description function with `"libcall"="f"` is IR the machine has no
//!   instruction for, done by calling runtime function `f` (declared in
//!   the description): its body is the IR (one instruction of its
//!   parameters and constants, as an instruction's is), and the call takes
//!   the parameters in order (`fadd fp128` is `__addtf3`).
//! - One with `"lower"="g"` is IR that becomes the body of description
//!   function `g` (of the same parameters), which may call the runtime and
//!   do other IR: a comparison as a call and a test of its result, a
//!   negation as an exclusive or of the sign bit.
//!
//! Either is used only where its `"feature"`s are on and its `"format"`
//! (`macho`, `elf`), if it has one, is the convention's.
//!
//! - `!target.held` names types no register holds, each with the type it
//!   is held as once nothing but moves of it is left (loads, stores,
//!   selects, phis, arguments and results of calls, constants): AArch64
//!   holds `fp128` as `<2 x i64>`, in a q register, where AAPCS64 passes
//!   it. Its constants are the same bits (in lanes from the low end, as on
//!   a little-endian machine).

use epitome_ir::apint::ApInt;
use epitome_ir::module::*;
use epitome_ir::types::{Type, TypeId};

use crate::abi::Syntax;
use crate::prepare::{Ctx, Seq, call_intrinsic};

/// A description function with `"libcall"` or `"lower"`, and the one
/// instruction its body is.
struct Entry<'t> {
    func: &'t Function,
    root: &'t Inst,
    /// Its result's type (none for a call of no result) and its
    /// parameters', in the module being compiled.
    result_ty: Option<TypeId>,
    param_tys: Vec<TypeId>,
    /// The runtime function it calls, or the function whose body it
    /// becomes.
    target: Replacement<'t>,
}

enum Replacement<'t> {
    Call(&'t str),
    Body(&'t Function),
}

fn str_attr<'a>(attrs: &'a [Attr], key: &str) -> Option<&'a str> {
    attrs.iter().find_map(|a| match a {
        Attr::Str { key: k, value: Some(v) } if k == key.as_bytes() => std::str::from_utf8(v).ok(),
        _ => None,
    })
}

/// The description's entries usable in this function.
fn entries<'t>(cx: &mut Ctx<'t>, syntax: Syntax) -> Vec<Entry<'t>> {
    let d: &'t Module = &cx.t.module;
    let mut out = Vec::new();
    for (_, f) in d.functions() {
        let (call, body) = (str_attr(&f.fn_attrs, "libcall"), str_attr(&f.fn_attrs, "lower"));
        if call.is_none() && body.is_none() {
            continue;
        }
        if let Some(fs) = str_attr(&f.fn_attrs, "feature")
            && !fs.split(',').all(|x| cx.features.contains(x.trim()))
        {
            continue;
        }
        match str_attr(&f.fn_attrs, "format") {
            Some("macho") if syntax != Syntax::MachO => continue,
            Some("elf") if syntax != Syntax::Elf => continue,
            _ => {}
        }
        let insts: Vec<&Inst> = f.insts_in_order().map(|i| &f.insts[i.index()]).collect();
        let Some(root) = insts.iter().find(|i| !matches!(i.op, Op::Ret(_))) else { continue };
        let target = match (call, body) {
            (Some(c), _) => Replacement::Call(c),
            (None, Some(g)) => {
                let Some((_, gf)) = d.functions().find(|(id, _)| d.global(*id).name.as_deref() == Some(g)) else {
                    continue;
                };
                Replacement::Body(gf)
            }
            _ => unreachable!(),
        };
        // (None for a call of no result.)
        let result_ty = root.result.map(|r| import_ty(d, cx.m, f.locals[r.index()].ty));
        let param_tys = f.params.iter().map(|p| import_ty(d, cx.m, f.locals[p.local.index()].ty)).collect();
        out.push(Entry { func: f, root, result_ty, param_tys, target });
    }
    out
}

/// Type `ty` of the description, in the module being compiled.
fn import_ty(d: &Module, m: &mut Module, ty: TypeId) -> TypeId {
    let t = d.types.get(ty).clone();
    let t = match t {
        Type::Vector { scalable, len, elem } => Type::Vector { scalable, len, elem: import_ty(d, m, elem) },
        Type::Array { len, elem } => Type::Array { len, elem: import_ty(d, m, elem) },
        Type::Struct { fields, packed } => {
            Type::Struct { fields: fields.into_iter().map(|f| import_ty(d, m, f)).collect(), packed }
        }
        Type::Function { ret, params, varargs } => Type::Function {
            ret: import_ty(d, m, ret),
            params: params.into_iter().map(|p| import_ty(d, m, p)).collect(),
            varargs,
        },
        other => other,
    };
    m.types.intern(t)
}

/// Constant `c` of the description, in the module being compiled (numbers
/// and their vectors only).
fn import_const(d: &Module, m: &mut Module, c: ConstId) -> Option<ConstId> {
    let k = d.constant(c).clone();
    let ty = import_ty(d, m, k.ty);
    let kind = match k.kind {
        ConstKind::Int(_)
        | ConstKind::Float(_)
        | ConstKind::Zero
        | ConstKind::Null
        | ConstKind::Undef
        | ConstKind::Poison => k.kind,
        ConstKind::Splat(e) => ConstKind::Splat(import_const(d, m, e)?),
        ConstKind::Vector(es) => {
            ConstKind::Vector(es.into_iter().map(|e| import_const(d, m, e)).collect::<Option<_>>()?)
        }
        _ => return None,
    };
    Some(m.consts.intern(Constant { ty, kind }))
}

/// The operands of an instruction a pattern can be, in order.
fn operands(op: &Op) -> Option<Vec<Value>> {
    Some(match op {
        Op::Binary { lhs, rhs, .. } | Op::FCmp { lhs, rhs, .. } | Op::ICmp { lhs, rhs, .. } => vec![*lhs, *rhs],
        Op::Cast { value, .. } | Op::FNeg { value, .. } => vec![*value],
        Op::Call(c) => c.args.iter().map(|a| a.value).collect(),
        _ => return None,
    })
}

/// Whether two instructions are the same operation (their operands aside).
fn same_op(cx: &Ctx, d: &Module, a: &Op, b: &Op) -> bool {
    match (a, b) {
        (Op::Binary { op: x, .. }, Op::Binary { op: y, .. }) => x == y,
        (Op::FCmp { pred: x, .. }, Op::FCmp { pred: y, .. }) => x == y,
        (Op::ICmp { pred: x, .. }, Op::ICmp { pred: y, .. }) => x == y,
        (Op::Cast { op: x, .. }, Op::Cast { op: y, .. }) => x == y,
        (Op::FNeg { .. }, Op::FNeg { .. }) => true,
        (Op::Call(x), Op::Call(y)) => {
            let name = |m: &Module, c: &Call| match c.callee {
                Value::Const(k) => match m.constant(k).kind {
                    ConstKind::Global(g) => m.global(g).name.clone(),
                    _ => None,
                },
                _ => None,
            };
            name(d, x).is_some_and(|n| Some(n) == name(cx.m, y))
        }
        _ => false,
    }
}

/// What entry `e`'s parameters are where it matches instruction `i` (its
/// operation, its operands' and result's types, its constants), if it does.
fn matches(cx: &mut Ctx, f: &Function, e: &Entry, i: &Inst) -> Option<Vec<Value>> {
    let d = &cx.t.module;
    if !same_op(cx, d, &e.root.op, &i.op) {
        return None;
    }
    let (want, have) = (operands(&e.root.op)?, operands(&i.op)?);
    if want.len() != have.len() {
        return None;
    }
    if e.result_ty != i.result.map(|r| f.locals[r.index()].ty) {
        return None;
    }
    let mut args = vec![None; e.func.params.len()];
    for (w, &h) in want.iter().zip(&have) {
        match w {
            Value::Local(l) => {
                let k = e.func.params.iter().position(|p| p.local == *l)?;
                if e.param_tys[k] != cx.value_ty(f, h) {
                    return None;
                }
                if args[k].is_some_and(|a| a != h) {
                    return None;
                }
                args[k] = Some(h);
            }
            Value::Const(c) => {
                if Value::Const(import_const(d, cx.m, *c)?) != h {
                    return None;
                }
            }
            Value::Metadata(_) => return None,
        }
    }
    args.into_iter().collect()
}

/// The body of description function `g` (one block) on `args`, in `s`: its
/// value (none for `ret void`).
fn instantiate(cx: &mut Ctx, s: &mut Seq, g: &Function, args: &[Value]) -> Result<Option<Value>, String> {
    let d = &cx.t.module;
    let mut map: std::collections::HashMap<LocalId, Value> =
        g.params.iter().zip(args).map(|(p, &a)| (p.local, a)).collect();
    let value = |cx: &mut Ctx, map: &std::collections::HashMap<LocalId, Value>, v: Value| -> Result<Value, String> {
        match v {
            Value::Local(l) => map.get(&l).copied().ok_or_else(|| "a lowering's value used before it is made".into()),
            Value::Const(c) => {
                import_const(d, cx.m, c).map(Value::Const).ok_or_else(|| "a lowering's constant is not a number".into())
            }
            Value::Metadata(_) => Err("metadata in a lowering".into()),
        }
    };
    for i in g.insts_in_order() {
        let inst = &g.insts[i.index()];
        let ty = inst.result.map(|r| import_ty(d, cx.m, g.locals[r.index()].ty));
        let v = match &inst.op {
            Op::Ret(Some(r)) => return value(cx, &map, *r).map(Some),
            Op::Ret(None) => return Ok(None),
            Op::Load { volatile, atomic, ptr, align } => {
                let ptr = value(cx, &map, *ptr)?;
                s.op(Op::Load { volatile: *volatile, atomic: atomic.clone(), ptr, align: *align }, ty.unwrap())
            }
            Op::Binary { op, flags, fmf, lhs, rhs } => {
                let (lhs, rhs) = (value(cx, &map, *lhs)?, value(cx, &map, *rhs)?);
                s.op(Op::Binary { op: *op, flags: *flags, fmf: *fmf, lhs, rhs }, ty.unwrap())
            }
            Op::ICmp { pred, samesign, lhs, rhs } => {
                let (lhs, rhs) = (value(cx, &map, *lhs)?, value(cx, &map, *rhs)?);
                s.op(Op::ICmp { pred: *pred, samesign: *samesign, lhs, rhs }, ty.unwrap())
            }
            Op::Cast { op, flags, fmf, value: x } => {
                let x = value(cx, &map, *x)?;
                s.op(Op::Cast { op: *op, flags: *flags, fmf: *fmf, value: x }, ty.unwrap())
            }
            Op::Select { fmf, cond, then_value, else_value } => {
                let (c, a, b) = (value(cx, &map, *cond)?, value(cx, &map, *then_value)?, value(cx, &map, *else_value)?);
                s.op(Op::Select { fmf: *fmf, cond: c, then_value: a, else_value: b }, ty.unwrap())
            }
            Op::Call(c) => {
                let Value::Const(k) = c.callee else { return Err("a lowering's call of a value".into()) };
                let ConstKind::Global(gid) = d.constant(k).kind else {
                    return Err("a lowering's call of a value".into());
                };
                let name = d.global(gid).name.clone().ok_or("a lowering's call of a function without a name")?;
                let a: Vec<Value> = c.args.iter().map(|x| value(cx, &map, x.value)).collect::<Result<_, _>>()?;
                let rt = ty.unwrap_or_else(|| cx.m.types.intern(Type::Void));
                call_intrinsic(cx, s, &name, a, rt)
            }
            other => return Err(format!("a lowering's {other:?}, which lowerings cannot do")),
        };
        if let Some(r) = inst.result {
            map.insert(r, v);
        }
    }
    Err("a lowering without a result".into())
}

/// Each instruction an entry matches, replaced as it says.
pub fn lower(cx: &mut Ctx, f: &mut Function) -> Result<(), String> {
    // (The object format, by the triple, as the conventions choose it.)
    let macho = cx.m.triple.as_deref().is_some_and(|t| {
        let t = String::from_utf8_lossy(t);
        ["apple", "darwin", "macos", "ios"].iter().any(|p| t.contains(p))
    });
    let entries = entries(cx, if macho { Syntax::MachO } else { Syntax::Elf });
    if entries.is_empty() {
        return Ok(());
    }
    let blocks: Vec<BlockId> = f.layout.clone();
    for b in blocks {
        let mut k = 0;
        while k < f.blocks[b.index()].insts.len() {
            let i = f.blocks[b.index()].insts[k];
            let inst = f.insts[i.index()].clone();
            let Some((e, args)) = entries.iter().find_map(|e| matches(cx, f, e, &inst).map(|a| (e, a))) else {
                k += 1;
                continue;
            };
            let mut s = Seq { f, new: Vec::new() };
            let v = match e.target {
                Replacement::Call(name) => {
                    let rt =
                        inst.result.map(|r| s.f.locals[r.index()].ty).unwrap_or_else(|| cx.m.types.intern(Type::Void));
                    Some(call_intrinsic(cx, &mut s, name, args, rt))
                }
                Replacement::Body(g) => instantiate(cx, &mut s, g, &args)?,
            };
            let new = std::mem::take(&mut s.new);
            let n = new.len();
            f.blocks[b.index()].insts.splice(k..=k, new);
            if let Some(r) = inst.result {
                f.replace_uses(Value::Local(r), v.ok_or("a lowering of no value for an instruction of one")?);
            }
            k += n;
        }
    }
    Ok(())
}

/// The description's held types (`!target.held`), as types of the module
/// being compiled: (the type, what it is held as).
fn held(cx: &mut Ctx) -> Result<Vec<(TypeId, TypeId)>, String> {
    let d = &cx.t.module;
    let mut out = Vec::new();
    let Some(nm) = d.named_metadata.iter().find(|n| n.name == "target.held") else { return Ok(out) };
    for &id in &nm.ops {
        let Metadata::Tuple { ops, .. } = &d.metadata[id.index()].data else { continue };
        let text = |k: usize| -> Option<String> {
            let id = (*ops.get(k)?)?;
            match &d.metadata[id.index()].data {
                Metadata::String(s) => Some(String::from_utf8_lossy(s).into_owned()),
                _ => None,
            }
        };
        let (Some(a), Some(b)) = (text(0), text(1)) else { return Err("!target.held: a pair of type names".into()) };
        out.push((parse_ty(cx.m, &a)?, parse_ty(cx.m, &b)?));
    }
    Ok(out)
}

/// A type written as in IR: `fp128`, `i128`, `<2 x i64>`, `double`.
fn parse_ty(m: &mut Module, s: &str) -> Result<TypeId, String> {
    let s = s.trim();
    let t = match s {
        "half" => Type::Half,
        "float" => Type::Float,
        "double" => Type::Double,
        "fp128" => Type::Fp128,
        _ if s.starts_with('i') => Type::Int(s[1..].parse().map_err(|_| format!("a type `{s}`"))?),
        _ if s.starts_with('<') && s.ends_with('>') => {
            let (n, e) = s[1..s.len() - 1].split_once(" x ").ok_or_else(|| format!("a type `{s}`"))?;
            let elem = parse_ty(m, e)?;
            Type::Vector { scalable: false, len: n.trim().parse().map_err(|_| format!("a type `{s}`"))?, elem }
        }
        _ => return Err(format!("!target.held: a type `{s}`")),
    };
    Ok(m.types.intern(t))
}

/// Type `ty` with each held type in it as what holds it.
fn map_ty(m: &mut Module, held: &[(TypeId, TypeId)], ty: TypeId) -> TypeId {
    if let Some(&(_, h)) = held.iter().find(|(t, _)| *t == ty) {
        return h;
    }
    let t = match m.types.get(ty).clone() {
        Type::Vector { scalable, len, elem } => Type::Vector { scalable, len, elem: map_ty(m, held, elem) },
        Type::Array { len, elem } => Type::Array { len, elem: map_ty(m, held, elem) },
        Type::Struct { fields, packed } => {
            Type::Struct { fields: fields.into_iter().map(|f| map_ty(m, held, f)).collect(), packed }
        }
        Type::Function { ret, params, varargs } => Type::Function {
            ret: map_ty(m, held, ret),
            params: params.into_iter().map(|p| map_ty(m, held, p)).collect(),
            varargs,
        },
        _ => return ty,
    };
    m.types.intern(t)
}

/// Constant `c` with each held type in it as what holds it: a
/// floating-point value's bits as an integer, or as a vector of integers
/// of them from the low end.
fn map_const(m: &mut Module, held: &[(TypeId, TypeId)], c: ConstId) -> ConstId {
    let k = m.constant(c).clone();
    let ty = map_ty(m, held, k.ty);
    if ty == k.ty {
        return c;
    }
    let kind = match k.kind {
        ConstKind::Float(bits) => match m.types.get(ty).clone() {
            Type::Int(w) => ConstKind::Int(ApInt::parse_hex(w, &format!("{bits:x}"), false).unwrap()),
            Type::Vector { len, elem, .. } => {
                let w = match m.types.get(elem) {
                    Type::Int(w) => *w,
                    _ => 64,
                };
                let lanes = (0..len)
                    .map(|l| {
                        let v = (bits >> (l * w)) & ((1u128 << w) - 1);
                        let kind = ConstKind::Int(ApInt::parse_hex(w, &format!("{v:x}"), false).unwrap());
                        m.consts.intern(Constant { ty: elem, kind })
                    })
                    .collect();
                ConstKind::Vector(lanes)
            }
            _ => ConstKind::Float(bits),
        },
        ConstKind::Splat(e) => ConstKind::Splat(map_const(m, held, e)),
        ConstKind::Vector(es) => ConstKind::Vector(es.into_iter().map(|e| map_const(m, held, e)).collect()),
        ConstKind::Array(es) => ConstKind::Array(es.into_iter().map(|e| map_const(m, held, e)).collect()),
        ConstKind::Struct(es) => ConstKind::Struct(es.into_iter().map(|e| map_const(m, held, e)).collect()),
        other => other,
    };
    m.consts.intern(Constant { ty, kind })
}

/// Whether type `ty` has a held type in it.
fn has_held(m: &Module, held: &[(TypeId, TypeId)], ty: TypeId) -> bool {
    held.iter().any(|(t, _)| *t == ty)
        || match m.types.get(ty) {
            Type::Vector { elem, .. } | Type::Array { elem, .. } => has_held(m, held, *elem),
            Type::Struct { fields, .. } => fields.iter().any(|&f| has_held(m, held, f)),
            _ => false,
        }
}

/// That nothing but moves of held types is left in `f`: what no entry
/// matched (`sqrt` of fp128 where the C library has no `sqrtl` of it) is
/// an error naming it, not a value of the type that holds it.
fn moves_only(cx: &Ctx, f: &Function, held: &[(TypeId, TypeId)]) -> Result<(), String> {
    let m = &*cx.m;
    for i in f.insts_in_order() {
        let inst = &f.insts[i.index()];
        let mut touches = inst.result.is_some_and(|r| has_held(m, held, f.locals[r.index()].ty));
        inst.op.for_each_value(|v| {
            if !matches!(v, Value::Metadata(_)) {
                touches |= has_held(m, held, cx.value_ty(f, *v));
            }
        });
        if !touches {
            continue;
        }
        let ok = match &inst.op {
            Op::Load { .. }
            | Op::Store { .. }
            | Op::Select { .. }
            | Op::Phi { .. }
            | Op::Freeze(_)
            | Op::Ret(_)
            | Op::ExtractValue { .. }
            | Op::InsertValue { .. }
            | Op::ExtractElement { .. }
            | Op::InsertElement { .. }
            | Op::ShuffleVector { .. }
            | Op::Alloca { .. } => true,
            Op::Cast { op: CastOp::BitCast, .. } => true,
            // Calls of functions, not of intrinsics (which would be taken
            // for those of the holding type).
            Op::Call(c) => !crate::prepare::callee_name(m, c).is_some_and(|n| n.starts_with("llvm.")),
            Op::Invoke { call, .. } => !crate::prepare::callee_name(m, call).is_some_and(|n| n.starts_with("llvm.")),
            _ => false,
        };
        if !ok {
            let what = match &inst.op {
                Op::Call(c) => crate::prepare::callee_name(m, c).unwrap_or_default(),
                op => format!("{op:?}").split([' ', '(', '{']).next().unwrap_or("").to_string(),
            };
            let ty = held.iter().find(|(t, _)| inst.result.is_some_and(|r| f.locals[r.index()].ty == *t));
            let name = |t: TypeId| match m.types.get(t) {
                Type::Fp128 => "fp128".to_string(),
                other => format!("{other:?}"),
            };
            return Err(format!(
                "{what} of {}: no instruction does it, and the description names no runtime function for it here",
                ty.map_or_else(|| name(held[0].0), |(t, _)| name(*t))
            ));
        }
    }
    Ok(())
}

/// Each held type in function `f` (its values, constants, calls' types,
/// allocations) as what holds it. Run after `lower`, when only moves of
/// them are left.
pub fn hold(cx: &mut Ctx, f: &mut Function) -> Result<(), String> {
    let held = held(cx)?;
    if held.is_empty() {
        return Ok(());
    }
    moves_only(cx, f, &held)?;
    let m = &mut *cx.m;
    for l in &mut f.locals {
        l.ty = map_ty(m, &held, l.ty);
    }
    f.ty = map_ty(m, &held, f.ty);
    for inst in &mut f.insts {
        inst.op.for_each_value_mut(|v| {
            if let Value::Const(c) = v {
                *c = map_const(m, &held, *c);
            }
        });
        match &mut inst.op {
            Op::Call(c) => c.fn_ty = map_ty(m, &held, c.fn_ty),
            Op::Invoke { call, .. } => call.fn_ty = map_ty(m, &held, call.fn_ty),
            Op::Alloca { ty, .. } => *ty = map_ty(m, &held, *ty),
            Op::Gep { source_ty, .. } => *source_ty = map_ty(m, &held, *source_ty),
            _ => {}
        }
    }
    Ok(())
}
