//! Scalar replacement of aggregates: splitting an alloca into one alloca
//! per independently used piece, so that `mem2reg` can promote them.
//!
//! A struct or array in memory is often used one field at a time:
//!
//! ```llvm
//! %p = alloca { i32, i32 }
//! %y = getelementptr { i32, i32 }, ptr %p, i32 0, i32 1
//! store i32 1, ptr %p
//! store i32 2, ptr %y
//! ```
//!
//! `mem2reg` cannot promote `%p`: it is accessed at an offset, and not as a
//! `{ i32, i32 }`. SROA gives each field its own alloca, `i32` each, which
//! `mem2reg` then promotes.
//!
//! The pass looks at every fixed-size alloca in the entry block:
//!
//! 1. It follows the alloca through `getelementptr`s with constant indices
//!    and `bitcast`s, and records every load and store as a byte range.
//!    Any other use (a call, a variable index, a stored address, a volatile
//!    access, `memcpy`) leaves the alloca alone.
//! 2. Overlapping ranges merge into *partitions*: pieces of the alloca used
//!    independently of each other. Bytes nothing touches are dropped.
//! 3. Each partition becomes a new alloca:
//!    - *Uniform*: every access covers the whole partition. The new alloca
//!      has the accessed type. Integer and float accesses of the same size
//!      share an integer type, with `bitcast`s at the floats.
//!    - *Packed integer*: some accesses cover only part of it, all integers
//!      or floats, and the partition is at most 128 bits. The new alloca is
//!      one integer of the partition's width. A narrower load shifts and
//!      truncates; a narrower store masks and merges, the shifts following
//!      the data layout's byte order. This is how a struct returned in a
//!      register (stored as one `i64`, read as two `i32`s) gets promoted.
//!
//!    If any partition fits neither, the alloca is left as it is.
//!
//! The pass runs before `mem2reg`, which promotes the new allocas.

use crate::fx::{FxHashMap as HashMap, FxHashSet as HashSet};

use epitome_ir::apint::ApInt;
use epitome_ir::edit::Names;
use epitome_ir::module::*;
use epitome_ir::types::{Type, TypeId};

use crate::ctx::Shared;

#[derive(Default, Debug, Clone, Copy)]
pub struct Stats {
    /// Allocas split or retyped.
    pub split: usize,
    /// New allocas made.
    pub pieces: usize,
}

/// A load or store of an alloca.
#[derive(Clone, Copy, Debug)]
struct Access {
    inst: InstId,
    offset: u64,
    size: u64,
    ty: TypeId,
    is_store: bool,
}

/// A `memcpy`, `memmove` or `memset` with a constant length that touches
/// the alloca.
#[derive(Clone, Copy, Debug)]
struct Copy {
    inst: InstId,
    offset: u64,
    len: u64,
    role: Role,
}

#[derive(Clone, Copy, Debug)]
enum Role {
    /// Copies into the alloca from `src`.
    Into(Value),
    /// Copies out of the alloca to `dst`.
    OutOf(Value),
    /// Fills with a byte.
    Set(u8),
}

/// What an alloca turned into: the instructions to delete, and the accesses
/// to rewrite.
struct Uses {
    accesses: Vec<Access>,
    copies: Vec<Copy>,
    /// GEPs, bitcasts and lifetime markers, deleted when the alloca is.
    others: Vec<InstId>,
}

/// The kind of memory intrinsic a call is, from its name.
fn mem_intrinsic(sh: &Shared, call: &Call) -> Option<&'static str> {
    let name = sh.direct_callee(call).and_then(|g| sh.module.global(g).name.as_deref())?;
    ["llvm.memcpy.", "llvm.memmove.", "llvm.memset."].into_iter().find(|p| name.starts_with(p))
}

fn is_lifetime(sh: &Shared, call: &Call) -> bool {
    sh.direct_callee(call)
        .and_then(|g| sh.module.global(g).name.as_deref())
        .is_some_and(|n| n.starts_with("llvm.lifetime."))
}

/// The byte offset a constant-index GEP adds, if it can be worked out.
fn gep_offset(sh: &Shared, source: TypeId, indices: &[Value]) -> Option<i64> {
    let (types, dl) = (sh.types(), &sh.layout);
    let int = |v: Value| -> Option<i64> {
        let Value::Const(c) = v else { return None };
        let k = crate::combine::eval::int_const(sh.consts(), types, c)?;
        Some(crate::combine::eval::sext(k.value, k.bits))
    };
    let mut offset = int(*indices.first()?)?.checked_mul(dl.alloc_size(types, source)? as i64)?;
    let mut ty = source;
    for &index in &indices[1..] {
        let n = int(index)?;
        match types.get(ty).clone() {
            Type::Struct { fields, packed } => {
                let layout = dl.struct_layout(types, &fields, packed)?;
                offset += *layout.offsets.get(usize::try_from(n).ok()?)? as i64;
                ty = fields[n as usize];
            }
            Type::Named(i) => {
                let body = types.structs()[i as usize].body.clone()?;
                let layout = dl.struct_layout(types, &body.fields, body.packed)?;
                offset += *layout.offsets.get(usize::try_from(n).ok()?)? as i64;
                ty = body.fields[n as usize];
            }
            Type::Array { elem, .. } | Type::Vector { scalable: false, elem, .. } => {
                offset = offset.checked_add(n.checked_mul(dl.alloc_size(types, elem)? as i64)?)?;
                ty = elem;
            }
            _ => return None,
        }
    }
    Some(offset)
}

/// Every access of the alloca `a`, which holds `size` bytes, or `None` if
/// it is used in a way SROA cannot handle.
fn collect(sh: &Shared, f: &Function, users: &HashMap<LocalId, Vec<InstId>>, a: LocalId, size: u64) -> Option<Uses> {
    let mut out = Uses { accesses: Vec::new(), copies: Vec::new(), others: Vec::new() };
    let int = |v: Value| -> Option<u64> {
        let Value::Const(c) = v else { return None };
        crate::combine::eval::int_const(sh.consts(), sh.types(), c).map(|k| k.value)
    };
    // Pointers whose other side is checked once every derived pointer is
    // known: stored values and copy partners.
    let mut partners: Vec<Value> = Vec::new();
    let mut work: Vec<(LocalId, i64)> = vec![(a, 0)];
    let mut derived: Vec<LocalId> = vec![a];
    while let Some((p, offset)) = work.pop() {
        let me = Value::Local(p);
        for &u in users.get(&p).map(|v| v.as_slice()).unwrap_or(&[]) {
            let inst = &f.insts[u.index()];
            match &inst.op {
                Op::Gep { source_ty, ptr, indices, .. } if *ptr == me && !indices.contains(&me) => {
                    let delta = gep_offset(sh, *source_ty, indices)?;
                    let r = inst.result?;
                    out.others.push(u);
                    derived.push(r);
                    work.push((r, offset.checked_add(delta)?));
                }
                Op::Cast { op: CastOp::BitCast, value, .. } if *value == me => {
                    let r = inst.result?;
                    out.others.push(u);
                    derived.push(r);
                    work.push((r, offset));
                }
                Op::Load { volatile: false, atomic: None, ptr, .. } if *ptr == me => {
                    let ty = f.locals[inst.result?.index()].ty;
                    out.accesses.push(Access {
                        inst: u,
                        offset: u64::try_from(offset).ok()?,
                        size: 0,
                        ty,
                        is_store: false,
                    });
                }
                Op::Store { volatile: false, atomic: None, ptr, value, .. } if *ptr == me => {
                    // Storing a pointer to the alloca itself lets it escape.
                    partners.push(*value);
                    let ty = match value {
                        Value::Local(l) => f.locals[l.index()].ty,
                        Value::Const(c) => sh.consts().get(*c).ty,
                        Value::Metadata(_) => return None,
                    };
                    out.accesses.push(Access {
                        inst: u,
                        offset: u64::try_from(offset).ok()?,
                        size: 0,
                        ty,
                        is_store: true,
                    });
                }
                Op::Call(call) if is_lifetime(sh, call) => out.others.push(u),
                Op::Call(call) if mem_intrinsic(sh, call).is_some() => {
                    let kind = mem_intrinsic(sh, call).unwrap();
                    let args: Vec<Value> = call.args.iter().map(|a| a.value).collect();
                    let [dst, other, len, volatile] = args[..] else { return None };
                    if int(volatile)? != 0 {
                        return None;
                    }
                    let len = int(len)?;
                    let offset = u64::try_from(offset).ok()?;
                    let role = if kind == "llvm.memset." {
                        if dst != me || other == me {
                            return None;
                        }
                        Role::Set(int(other)? as u8)
                    } else if dst == me && other != me {
                        partners.push(other);
                        Role::Into(other)
                    } else if other == me && dst != me {
                        partners.push(dst);
                        Role::OutOf(dst)
                    } else {
                        return None;
                    };
                    out.copies.push(Copy { inst: u, offset, len, role });
                }
                _ => return None,
            }
        }
    }
    if partners.iter().any(|v| matches!(v, Value::Local(l) if derived.contains(l))) {
        return None;
    }
    for c in &out.copies {
        if c.offset + c.len > size {
            return None;
        }
    }
    for acc in &mut out.accesses {
        acc.size = sh.layout.store_size(sh.types(), acc.ty)?;
        // An access outside the object is undefined behaviour; leave it be.
        if acc.size == 0 || acc.offset + acc.size > size {
            return None;
        }
    }
    Some(out)
}

/// How a partition is rewritten.
enum Kind {
    /// One type for every access; `int` is set when accesses of other
    /// types were unified as this integer type.
    Uniform(TypeId),
    /// One integer of this many bits.
    Packed(u32),
}

struct Partition {
    start: u64,
    end: u64,
    accesses: Vec<Access>,
}

fn scalar_kind(sh: &Shared, ty: TypeId) -> Option<bool> {
    // Some(true) for an integer, Some(false) for a float.
    match sh.types().get(ty) {
        Type::Int(_) => Some(true),
        Type::Half | Type::BFloat | Type::Float | Type::Double | Type::Fp128 => Some(false),
        _ => None,
    }
}

fn plan(sh: &Shared, p: &Partition) -> Option<Kind> {
    let bytes = p.end - p.start;
    let whole = p.accesses.iter().all(|a| a.offset == p.start && a.size == bytes);
    if whole {
        let first = p.accesses[0].ty;
        if p.accesses.iter().all(|a| a.ty == first) {
            return Some(Kind::Uniform(first));
        }
        // Same size, different types: share an integer, if all are
        // integers or floats and the size is a whole number of bits.
        if p.accesses.iter().all(|a| scalar_kind(sh, a.ty).is_some())
            && p.accesses.iter().all(|a| sh.layout.size_in_bits(sh.types(), a.ty) == Some(bytes * 8))
        {
            return Some(Kind::Uniform(sh.intern_type(Type::Int((bytes * 8) as u32))));
        }
        return None;
    }
    let bits_ok = |a: &Access| sh.layout.size_in_bits(sh.types(), a.ty) == Some(a.size * 8);
    if bytes * 8 <= 128 && p.accesses.iter().all(|a| scalar_kind(sh, a.ty).is_some() && bits_ok(a)) {
        return Some(Kind::Packed((bytes * 8) as u32));
    }
    None
}

/// Runs SROA on one function.
pub fn run(sh: &Shared, f: &mut Function) -> Stats {
    let mut stats = Stats::default();
    if !f.has_body {
        return stats;
    }
    let entry = f.entry();
    split_aggregate_accesses(sh, f);
    // Splitting one alloca can turn a copy from another into plain loads,
    // which may let that one split too, so go round until nothing changes.
    for _round in 0..4 {
        let before = stats.split;
        split_round(sh, f, entry, &mut stats);
        if stats.split == before {
            break;
        }
    }
    stats
}

fn split_round(sh: &Shared, f: &mut Function, entry: BlockId, stats: &mut Stats) {
    let candidates: Vec<InstId> = f.blocks[entry.index()].insts.clone();
    let mut names = Names::new(f);
    let mut erased: Vec<InstId> = Vec::new();
    let mut new_allocas: Vec<InstId> = Vec::new();
    let big_endian = sh.layout.big_endian;
    let ptr = sh.intern_type(Type::Ptr(0));

    for i in candidates {
        let Op::Alloca { inalloca: false, ty, count: None, addrspace: None | Some(0), .. } = f.insts[i.index()].op
        else {
            continue;
        };
        let Some(size) = sh.layout.alloc_size(sh.types(), ty) else { continue };
        let a = f.insts[i.index()].result.unwrap();
        // Uses are found afresh for each alloca: splitting one can add
        // accesses to another, when one was copied into the other.
        let mut users: HashMap<LocalId, Vec<InstId>> = HashMap::default();
        for x in f.insts_in_order() {
            if erased.contains(&x) {
                continue;
            }
            f.insts[x.index()].op.for_each_value(|v| {
                if let Value::Local(l) = v {
                    users.entry(*l).or_default().push(x);
                }
            });
        }
        let Some(uses) = collect(sh, f, &users, a, size) else { continue };
        if uses.accesses.is_empty() {
            // Only ever written: the alloca and everything writing it go.
            if !uses.copies.is_empty()
                && uses.copies.iter().all(|c| !matches!(c.role, Role::OutOf(_)))
                && epitome_ir::chop::allow(|| {
                    format!("sroa: remove {}, only ever written", epitome_ir::printer::local_name(f, a))
                })
            {
                erased.push(i);
                erased.extend(uses.others);
                erased.extend(uses.copies.iter().map(|c| c.inst));
                stats.split += 1;
            }
            continue;
        }

        // Partitions: overlapping accesses merge.
        let mut sorted = uses.accesses.clone();
        sorted.sort_by_key(|x| (x.offset, x.size));
        let mut parts: Vec<Partition> = Vec::new();
        for acc in sorted {
            match parts.last_mut() {
                Some(p) if acc.offset < p.end => {
                    p.end = p.end.max(acc.offset + acc.size);
                    p.accesses.push(acc);
                }
                _ => parts.push(Partition { start: acc.offset, end: acc.offset + acc.size, accesses: vec![acc] }),
            }
        }
        let Some(kinds) = parts.iter().map(|p| plan(sh, p)).collect::<Option<Vec<Kind>>>() else { continue };
        // Each copy must take whole partitions, and a copy out must have
        // every byte it reads in some partition.
        let copies_fit = uses.copies.iter().all(|c| {
            let (lo, hi) = (c.offset, c.offset + c.len);
            let mut covered = 0;
            for (p, kind) in parts.iter().zip(&kinds) {
                let inside = p.start >= lo && p.end <= hi;
                if !inside && p.start < hi && lo < p.end {
                    return false;
                }
                if inside {
                    covered += p.end - p.start;
                    if let Role::Set(byte) = c.role
                        && byte != 0
                        && !matches!(kind, Kind::Packed(_))
                        && !matches!(kind, Kind::Uniform(t) if scalar_kind(sh, *t).is_some())
                    {
                        return false;
                    }
                }
            }
            !matches!(c.role, Role::OutOf(_)) || covered == c.len
        });
        if !copies_fit {
            continue;
        }
        // Nothing to gain: one piece, already of the alloca's own type.
        if parts.len() == 1 && matches!(kinds[0], Kind::Uniform(t) if t == ty) && parts[0].start == 0 {
            continue;
        }
        if !epitome_ir::chop::allow(|| {
            format!("sroa: split {} into {} pieces", epitome_ir::printer::local_name(f, a), parts.len())
        }) {
            continue;
        }

        let base_name = f.locals[a.index()].name.clone().unwrap_or_else(|| "sroa".into());
        let at = f.blocks[entry.index()].insts.iter().position(|&x| x == i).unwrap();
        let mut pieces: Vec<(Value, TypeId)> = Vec::new();
        for (k, (p, kind)) in parts.iter().zip(&kinds).enumerate() {
            let piece_ty = match kind {
                Kind::Uniform(t) => *t,
                Kind::Packed(bits) => sh.intern_type(Type::Int(*bits)),
            };
            let name = Some(names.fresh(&format!("{base_name}.{}", p.start)));
            let new = f.add_inst(
                Op::Alloca { inalloca: false, ty: piece_ty, count: None, align: None, addrspace: None },
                Some((name, ptr)),
            );
            f.blocks[entry.index()].insts.insert(at + k, new);
            new_allocas.push(new);
            let piece = f.result(new);
            pieces.push((piece, piece_ty));
            for acc in &p.accesses {
                rewrite(sh, f, &mut names, acc, piece, piece_ty, p.start, kind, big_endian);
            }
        }
        for c in &uses.copies {
            for (p, &(piece, piece_ty)) in parts.iter().zip(&pieces) {
                if p.start >= c.offset && p.end <= c.offset + c.len {
                    split_copy(sh, f, &mut names, c, p.start - c.offset, piece, piece_ty);
                }
            }
            erased.push(c.inst);
        }
        erased.push(i);
        erased.extend(uses.others);
        stats.split += 1;
        stats.pieces += parts.len();
    }
    if !erased.is_empty() {
        let mut gone = vec![false; f.insts.len()];
        erased.iter().for_each(|x| gone[x.index()] = true);
        for b in f.layout.clone() {
            f.blocks[b.index()].insts.retain(|x| !gone[x.index()]);
        }
        // Debug records that named the old alloca or its offsets now name
        // nothing: optimised out. (**Open:** describe each piece with a
        // `DW_OP_LLVM_fragment` expression instead.)
        let dead = |v: Value| matches!(v, Value::Local(l) if matches!(f.locals[l.index()].def, LocalDef::Inst(d) if gone.get(d.index()).copied().unwrap_or(false)));
        let mut changes = Vec::new();
        for x in f.insts_in_order() {
            for (k, r) in f.insts[x.index()].debug_records.iter().enumerate() {
                for (j, &arg) in r.args.iter().enumerate() {
                    if let Metadata::Value(v @ Value::Local(l)) = sh.metadata()[arg.index()].data
                        && dead(v)
                    {
                        changes.push((x, k, j, f.locals[l.index()].ty));
                    }
                }
            }
        }
        for (x, k, j, ty) in changes {
            f.insts[x.index()].debug_records[k].args[j] = sh.md_value(sh.poison(ty));
        }
    }
}

/// Rewrites one access to use the partition's new alloca `piece`.
#[allow(clippy::too_many_arguments)]
fn rewrite(
    sh: &Shared,
    f: &mut Function,
    names: &mut Names,
    acc: &Access,
    piece: Value,
    piece_ty: TypeId,
    start: u64,
    kind: &Kind,
    big_endian: bool,
) {
    let i = acc.inst;
    let block = f.inst_blocks()[i.index()].unwrap();
    let pos = |f: &Function| f.blocks[block.index()].insts.iter().position(|&x| x == i).unwrap();
    // Inserts an instruction before the access.
    let mut before = |f: &mut Function, op: Op, ty: Option<TypeId>| -> Value {
        let name = ty.map(|_| Some(names.fresh("sroa")));
        let new = f.add_inst(op, name.zip(ty));
        let p = pos(f);
        f.blocks[block.index()].insts.insert(p, new);
        if ty.is_some() { f.result(new) } else { Value::Const(ConstId(0)) }
    };
    let acc_int = match sh.types().get(acc.ty) {
        Type::Int(_) => acc.ty,
        _ => sh.intern_type(Type::Int((acc.size * 8) as u32)),
    };
    let is_float = acc_int != acc.ty;
    let cast = |op, value| Op::Cast { op, flags: IntFlags::NONE, fmf: FastMath::default(), value };
    let bin = |op, lhs, rhs| Op::Binary { op, flags: IntFlags::NONE, fmf: FastMath::default(), lhs, rhs };

    match kind {
        Kind::Uniform(t) if *t == acc.ty => {
            // Same type: just point the access at the new alloca.
            set_ptr(f, i, piece);
        }
        Kind::Uniform(_) => {
            // A float access to an integer piece.
            if acc.is_store {
                let Op::Store { value, .. } = f.insts[i.index()].op else { unreachable!() };
                let as_int = before(f, cast(CastOp::BitCast, value), Some(piece_ty));
                set_store(f, i, piece, as_int);
            } else {
                let load =
                    before(f, Op::Load { volatile: false, atomic: None, ptr: piece, align: None }, Some(piece_ty));
                let back = before(f, cast(CastOp::BitCast, load), Some(acc.ty));
                replace_load(f, i, back);
            }
        }
        Kind::Packed(bits) => {
            let bits = *bits;
            let acc_bits = (acc.size * 8) as u32;
            let byte = acc.offset - start;
            let shift = if big_endian { bits as u64 - 8 * byte - acc_bits as u64 } else { 8 * byte };
            let wide = |v: u128| wide_const(sh, piece_ty, bits, v);
            if acc.is_store {
                let Op::Store { value, .. } = f.insts[i.index()].op else { unreachable!() };
                let mut v = value;
                if is_float {
                    v = before(f, cast(CastOp::BitCast, v), Some(acc_int));
                }
                if acc_bits == bits {
                    set_store(f, i, piece, v);
                    return;
                }
                let old =
                    before(f, Op::Load { volatile: false, atomic: None, ptr: piece, align: None }, Some(piece_ty));
                let ext = before(f, cast(CastOp::ZExt, v), Some(piece_ty));
                let moved =
                    if shift > 0 { before(f, bin(BinOp::Shl, ext, wide(shift as u128)), Some(piece_ty)) } else { ext };
                let field_mask: u128 = if acc_bits >= 128 { u128::MAX } else { ((1u128 << acc_bits) - 1) << shift };
                let keep_mask = !field_mask & if bits >= 128 { u128::MAX } else { (1u128 << bits) - 1 };
                let kept = before(f, bin(BinOp::And, old, wide(keep_mask)), Some(piece_ty));
                let merged = before(f, bin(BinOp::Or, kept, moved), Some(piece_ty));
                set_store(f, i, piece, merged);
            } else {
                let whole =
                    before(f, Op::Load { volatile: false, atomic: None, ptr: piece, align: None }, Some(piece_ty));
                let mut v = whole;
                if acc_bits != bits {
                    if shift > 0 {
                        v = before(f, bin(BinOp::LShr, v, wide(shift as u128)), Some(piece_ty));
                    }
                    v = before(f, cast(CastOp::Trunc, v), Some(acc_int));
                }
                if is_float {
                    v = before(f, cast(CastOp::BitCast, v), Some(acc.ty));
                }
                replace_load(f, i, v);
            }
        }
    }
}

/// The part of a copy or fill that lands in one piece, `delta` bytes into
/// the copied range, as a load and a store placed before the call. The
/// other side's alignment is not known, so its accesses say `align 1`.
#[allow(clippy::too_many_arguments)]
fn split_copy(sh: &Shared, f: &mut Function, names: &mut Names, c: &Copy, delta: u64, piece: Value, piece_ty: TypeId) {
    let block = f.inst_blocks()[c.inst.index()].unwrap();
    let mut before = |f: &mut Function, op: Op, ty: Option<TypeId>| -> Value {
        let new = f.add_inst(op, ty.map(|t| (Some(names.fresh("sroa")), t)));
        let p = f.blocks[block.index()].insts.iter().position(|&x| x == c.inst).unwrap();
        f.blocks[block.index()].insts.insert(p, new);
        if ty.is_some() { f.result(new) } else { Value::Const(ConstId(0)) }
    };
    let i8 = sh.intern_type(Type::Int(8));
    let ptr = sh.intern_type(Type::Ptr(0));
    let at = |f: &mut Function, base: Value, before: &mut dyn FnMut(&mut Function, Op, Option<TypeId>) -> Value| {
        if delta == 0 {
            base
        } else {
            let op = Op::Gep {
                flags: GepFlags(0),
                source_ty: i8,
                ptr: base,
                indices: vec![sh.int(64, delta as i64)].into(),
            };
            before(f, op, Some(ptr))
        }
    };
    match c.role {
        Role::Into(src) => {
            let from = at(f, src, &mut before);
            let v = before(
                f,
                Op::Load { volatile: false, atomic: None, ptr: from, align: Some(Align::of(1)) },
                Some(piece_ty),
            );
            before(f, Op::Store { volatile: false, atomic: None, value: v, ptr: piece, align: None }, None);
        }
        Role::OutOf(dst) => {
            let v = before(f, Op::Load { volatile: false, atomic: None, ptr: piece, align: None }, Some(piece_ty));
            let to = at(f, dst, &mut before);
            before(f, Op::Store { volatile: false, atomic: None, value: v, ptr: to, align: Some(Align::of(1)) }, None);
        }
        Role::Set(byte) => {
            let is_int = matches!(sh.types().get(piece_ty), Type::Int(_));
            let v = if byte == 0 && !is_int {
                Value::Const(sh.intern_const(Constant { ty: piece_ty, kind: ConstKind::Zero }))
            } else {
                let bits = sh.layout.size_in_bits(sh.types(), piece_ty).unwrap() as u32;
                let mut pattern: u128 = 0;
                for _ in 0..bits / 8 {
                    pattern = (pattern << 8) | byte as u128;
                }
                let int_ty = sh.intern_type(Type::Int(bits));
                let k = wide_const(sh, int_ty, bits, pattern);
                if int_ty == piece_ty {
                    k
                } else {
                    let op =
                        Op::Cast { op: CastOp::BitCast, flags: IntFlags::NONE, fmf: FastMath::default(), value: k };
                    before(f, op, Some(piece_ty))
                }
            };
            before(f, Op::Store { volatile: false, atomic: None, value: v, ptr: piece, align: None }, None);
        }
    }
}

/// Most elements an aggregate access is split into.
const MAX_ELEMENTS: u64 = 16;

/// Breaks loads and stores of whole structs and arrays in entry-block
/// allocas into one access per element, so that the elements can be
/// partitioned like any other access. A struct passed by value often
/// travels as `[2 x i64]` while its fields are read as `i32`s; split, each
/// `i64` packs with the fields that share its bytes. Nested aggregates are
/// split a level at a time until none are left.
fn split_aggregate_accesses(sh: &Shared, f: &mut Function) {
    let entry = f.entry();
    let allocas: HashSet<LocalId> = f.blocks[entry.index()]
        .insts
        .iter()
        .filter(|&&i| matches!(f.insts[i.index()].op, Op::Alloca { .. }))
        .filter_map(|&i| f.insts[i.index()].result)
        .collect();
    let based_on_alloca = |f: &Function, mut v: Value| -> bool {
        for _ in 0..32 {
            let Value::Local(l) = v else { return false };
            if allocas.contains(&l) {
                return true;
            }
            match f.locals[l.index()].def {
                LocalDef::Inst(d) => match &f.insts[d.index()].op {
                    Op::Gep { ptr, .. } => v = *ptr,
                    Op::Cast { op: CastOp::BitCast, value, .. } => v = *value,
                    _ => return false,
                },
                _ => return false,
            }
        }
        false
    };
    let mut names = Names::new(f);
    let ptr_ty = sh.intern_type(Type::Ptr(0));
    for _level in 0..4 {
        let mut changed = false;
        // (An instruction's block does not change as others are added.)
        let blocks = f.inst_blocks();
        for i in f.insts_in_order().collect::<Vec<_>>() {
            let (ptr, ty, store_value) = match &f.insts[i.index()].op {
                Op::Load { volatile: false, atomic: None, ptr, .. } => {
                    (*ptr, f.locals[f.insts[i.index()].result.unwrap().index()].ty, None)
                }
                Op::Store { volatile: false, atomic: None, ptr, value, .. } => {
                    let ty = match value {
                        Value::Local(l) => f.locals[l.index()].ty,
                        Value::Const(c) => sh.consts().get(*c).ty,
                        Value::Metadata(_) => continue,
                    };
                    (*ptr, ty, Some(*value))
                }
                _ => continue,
            };
            // The element types, and how to index each one.
            let (elems, is_struct): (Vec<TypeId>, bool) = match sh.types().get(ty).clone() {
                Type::Struct { fields, .. } => (fields, true),
                Type::Named(n) => match sh.types().structs()[n as usize].body.clone() {
                    Some(b) => (b.fields, true),
                    None => continue,
                },
                Type::Array { len, elem } if len <= MAX_ELEMENTS => (vec![elem; len as usize], false),
                _ => continue,
            };
            if elems.is_empty() || elems.len() as u64 > MAX_ELEMENTS || !based_on_alloca(f, ptr) {
                continue;
            }
            if !epitome_ir::chop::allow(|| {
                format!("sroa: split the aggregate access {}", epitome_ir::printer::inst_name(f, i))
            }) {
                continue;
            }
            let block = blocks[i.index()].unwrap();
            let mut new_insts = Vec::new();
            let mut emit = |f: &mut Function, op: Op, result: Option<TypeId>| -> Value {
                let new = f.add_inst(op, result.map(|t| (Some(names.fresh("sroa")), t)));
                new_insts.push(new);
                if result.is_some() { f.result(new) } else { Value::Const(ConstId(0)) }
            };
            let mut aggregate = sh.poison(ty);
            for (k, &e) in elems.iter().enumerate() {
                let indices = if is_struct {
                    vec![sh.int(32, 0), sh.int(32, k as i64)]
                } else {
                    vec![sh.int(64, 0), sh.int(64, k as i64)]
                };
                let at = emit(
                    f,
                    Op::Gep { flags: GepFlags(GepFlags::INBOUNDS), source_ty: ty, ptr, indices: indices.into() },
                    Some(ptr_ty),
                );
                match store_value {
                    Some(v) => {
                        let part = emit(f, Op::ExtractValue { aggregate: v, indices: vec![k as u32].into() }, Some(e));
                        emit(f, Op::Store { volatile: false, atomic: None, value: part, ptr: at, align: None }, None);
                    }
                    None => {
                        let part = emit(f, Op::Load { volatile: false, atomic: None, ptr: at, align: None }, Some(e));
                        aggregate = emit(
                            f,
                            Op::InsertValue { aggregate, element: part, indices: vec![k as u32].into() },
                            Some(ty),
                        );
                    }
                }
            }
            let records = std::mem::take(&mut f.insts[i.index()].debug_records);
            f.insts[new_insts[0].index()].debug_records = records;
            let insts = &mut f.blocks[block.index()].insts;
            let at = insts.iter().position(|&x| x == i).unwrap();
            insts.splice(at..at + 1, new_insts);
            if store_value.is_none() {
                let r = f.insts[i.index()].result.unwrap();
                f.replace_uses(Value::Local(r), aggregate);
            }
            changed = true;
        }
        if !changed {
            break;
        }
    }
}

/// An integer constant of up to 128 bits.
fn wide_const(sh: &Shared, ty: TypeId, bits: u32, v: u128) -> Value {
    let masked = if bits >= 128 { v } else { v & ((1u128 << bits) - 1) };
    let ap = ApInt::parse_decimal(bits, &masked.to_string()).expect("constant fits its width");
    Value::Const(sh.intern_const(Constant { ty, kind: ConstKind::Int(ap) }))
}

/// Points an access at the new alloca. Its old alignment may be more than
/// the new alloca has, so it falls back to the type's own.
fn set_ptr(f: &mut Function, i: InstId, to: Value) {
    match &mut f.insts[i.index()].op {
        Op::Load { ptr, align, .. } | Op::Store { ptr, align, .. } => {
            *ptr = to;
            *align = None;
        }
        _ => unreachable!(),
    }
}

fn set_store(f: &mut Function, i: InstId, ptr_to: Value, value_to: Value) {
    if let Op::Store { ptr, value, align, .. } = &mut f.insts[i.index()].op {
        *ptr = ptr_to;
        *value = value_to;
        *align = None;
    }
}

/// Replaces a load by `v` and removes it from its block.
fn replace_load(f: &mut Function, i: InstId, v: Value) {
    let r = f.insts[i.index()].result.unwrap();
    f.replace_uses(Value::Local(r), v);
    // Debug records on the load move to the next instruction.
    let records = std::mem::take(&mut f.insts[i.index()].debug_records);
    let block = f.inst_blocks()[i.index()].unwrap();
    let insts = &mut f.blocks[block.index()].insts;
    let k = insts.iter().position(|&x| x == i).unwrap();
    insts.remove(k);
    let next = insts[k];
    f.insts[next.index()].debug_records.splice(0..0, records);
}
