//! Loads from constant globals, folded: the part of LLVM's `globalopt` that
//! is a local rewrite. A load whose address is a constant global plus a
//! constant offset (through `getelementptr`s, as instructions or constant
//! expressions) reads the global's initialiser, which is known.
//!
//! The global must be `constant`, have an initialiser, not be
//! `externally_initialized`, and have a definition that cannot be replaced
//! at link time: not `weak`, `linkonce`, `common` or `extern_weak` (the
//! `_odr` forms promise every definition is the same). The value is read
//! out of the initialiser through arrays, vectors, structs, byte strings
//! and zero initialisers; an integer may also be read out of a byte string
//! at any offset. Anything else, such as a value straddling two fields,
//! is left as a load.

use epitome_ir::apint::ApInt;
use epitome_ir::module::*;
use epitome_ir::types::{Type, TypeId};

use crate::ctx::Shared;

/// The value a load of type `ty` from `ptr` gives, if `ptr` points into a
/// constant global's initialiser at a known offset.
pub fn load_constant(sh: &Shared, f: &Function, ptr: Value, ty: TypeId) -> Option<Value> {
    let (g, offset) = address(sh, f, ptr, 0)?;
    let GlobalKind::Variable(var) = &sh.module.global(g).kind else { return None };
    let replaceable = matches!(
        sh.module.global(g).props.linkage,
        Linkage::Weak | Linkage::LinkOnce | Linkage::Common | Linkage::ExternWeak
    );
    if !var.constant || var.externally_initialized || replaceable || offset < 0 {
        return None;
    }
    read(sh, var.init?, offset as u64, ty).map(Value::Const)
}

/// A constant integer operand.
fn int(sh: &Shared, v: Value) -> Option<i64> {
    let Value::Const(c) = v else { return None };
    let k = super::eval::int_const(sh.consts(), sh.types(), c)?;
    Some(super::eval::sext(k.value, k.bits))
}

/// The global and byte offset `v` points to, if both are constant.
fn address(sh: &Shared, f: &Function, v: Value, depth: u32) -> Option<(GlobalId, i64)> {
    if depth > 16 {
        return None;
    }
    let gep = |source: TypeId, ptr: Value, indices: Vec<Value>| -> Option<(GlobalId, i64)> {
        let (g, base) = address(sh, f, ptr, depth + 1)?;
        Some((g, base.checked_add(gep_offset(sh, source, &indices)?)?))
    };
    match v {
        Value::Const(c) => match &sh.consts().get(c).kind {
            ConstKind::Global(g) => Some((*g, 0)),
            ConstKind::Expr(ConstExpr::Gep { source_ty, ptr, indices, .. }) => {
                let (s, p, ix) = (*source_ty, Value::Const(*ptr), indices.iter().map(|&i| Value::Const(i)).collect());
                gep(s, p, ix)
            }
            ConstKind::Expr(ConstExpr::Cast { op: CastOp::BitCast, value }) => {
                address(sh, f, Value::Const(*value), depth + 1)
            }
            _ => None,
        },
        Value::Local(l) => match f.locals[l.index()].def {
            LocalDef::Inst(i) => match &f.insts[i.index()].op {
                Op::Gep { source_ty, ptr, indices, .. } => gep(*source_ty, *ptr, indices.to_vec()),
                Op::Cast { op: CastOp::BitCast, value, .. } => address(sh, f, *value, depth + 1),
                _ => None,
            },
            _ => None,
        },
        Value::Metadata(_) => None,
    }
}

/// The byte offset a `getelementptr` of `source` with constant `indices`
/// adds.
pub(crate) fn gep_offset(sh: &Shared, source: TypeId, indices: &[Value]) -> Option<i64> {
    let (types, dl) = (sh.types(), &sh.layout);
    let mut ty = source;
    let mut offset = 0i64;
    for (k, &index) in indices.iter().enumerate() {
        let n = int(sh, index)?;
        if k == 0 {
            offset = offset.checked_add(n.checked_mul(dl.alloc_size(types, ty)? as i64)?)?;
            continue;
        }
        match types.get(ty).clone() {
            Type::Array { elem, .. } | Type::Vector { elem, .. } => {
                offset = offset.checked_add(n.checked_mul(dl.alloc_size(types, elem)? as i64)?)?;
                ty = elem;
            }
            Type::Struct { fields, packed } => {
                let layout = dl.struct_layout(types, &fields, packed)?;
                offset = offset.checked_add(*layout.offsets.get(n as usize)? as i64)?;
                ty = fields[n as usize];
            }
            Type::Named(s) => {
                let body = types.structs()[s as usize].body.clone()?;
                let layout = dl.struct_layout(types, &body.fields, body.packed)?;
                offset = offset.checked_add(*layout.offsets.get(n as usize)? as i64)?;
                ty = body.fields[n as usize];
            }
            _ => return None,
        }
    }
    Some(offset)
}

/// The zero of a scalar type.
fn zero(sh: &Shared, ty: TypeId) -> Option<ConstId> {
    let kind = match sh.types().get(ty) {
        Type::Int(bits) => ConstKind::Int(ApInt::from_u64(*bits, 0)),
        Type::Ptr(_) => ConstKind::Null,
        Type::Float | Type::Double | Type::Half | Type::BFloat => ConstKind::Float(0),
        _ => return None,
    };
    Some(sh.intern_const(Constant { ty, kind }))
}

/// The value of type `ty` at byte `offset` of constant `c`.
fn read(sh: &Shared, c: ConstId, offset: u64, ty: TypeId) -> Option<ConstId> {
    let (types, dl) = (sh.types(), &sh.layout);
    let k = sh.consts().get(c).clone();
    let size = dl.store_size(types, ty)?;
    let whole = dl.store_size(types, k.ty)?;
    if offset.checked_add(size)? > whole {
        return None;
    }
    if offset == 0 && k.ty == ty {
        return Some(c);
    }
    // The element of an array, vector or struct that holds the bytes, and
    // where it starts.
    let element = |elems: &[ConstId]| -> Option<ConstId> {
        let (start, field) = match types.get(k.ty).clone() {
            Type::Array { elem, .. } | Type::Vector { elem, .. } => {
                let step = dl.alloc_size(types, elem)?;
                let n = offset / step.max(1);
                (n * step, *elems.get(n as usize)?)
            }
            Type::Struct { fields, packed } => {
                let layout = dl.struct_layout(types, &fields, packed)?;
                let n = layout.offsets.iter().rposition(|&o| o <= offset)?;
                (layout.offsets[n], *elems.get(n)?)
            }
            Type::Named(s) => {
                let body = types.structs()[s as usize].body.clone()?;
                let layout = dl.struct_layout(types, &body.fields, body.packed)?;
                let n = layout.offsets.iter().rposition(|&o| o <= offset)?;
                (layout.offsets[n], *elems.get(n)?)
            }
            _ => return None,
        };
        read(sh, field, offset - start, ty)
    };
    match &k.kind {
        ConstKind::Zero | ConstKind::Null => zero(sh, ty),
        ConstKind::Array(elems) | ConstKind::Struct(elems) | ConstKind::Vector(elems) => element(elems),
        ConstKind::Splat(e) => {
            let n = match types.get(k.ty) {
                Type::Vector { len, .. } => *len as usize,
                _ => return None,
            };
            element(&vec![*e; n])
        }
        // A byte string: an integer of whole bytes from anywhere in it.
        ConstKind::Bytes(bytes) => {
            let Type::Int(bits) = *types.get(ty) else { return None };
            if bits % 8 != 0 || bits > 64 {
                return None;
            }
            let at = &bytes[offset as usize..(offset + size) as usize];
            let mut v = 0u64;
            for (i, &b) in at.iter().enumerate() {
                let shift = if dl.big_endian { 8 * (at.len() - 1 - i) } else { 8 * i };
                v |= (b as u64) << shift;
            }
            Some(sh.intern_const(Constant { ty, kind: ConstKind::Int(ApInt::from_u64(bits, v)) }))
        }
        _ => None,
    }
}
