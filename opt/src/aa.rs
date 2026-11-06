//! Basic alias analysis: whether two memory accesses can touch the same
//! bytes, and whether an instruction can read or write a location.
//!
//! It answers from simple facts, without looking at other functions:
//!
//! - **Different objects.** Every pointer is traced back through
//!   `getelementptr`s to the value it is based on. Allocas, globals, results
//!   of `noalias` calls (allocators) and `noalias` or `byval` arguments are
//!   *identified* objects: distinct ones never overlap, and none is null.
//! - **Objects that do not escape.** If an alloca's or allocation's address
//!   is only ever loaded through, stored through, or offset, nothing else
//!   can hold a pointer to it: it cannot alias a pointer that comes from
//!   anywhere else, and no call can read or write it.
//! - **Offsets.** Two pointers from the same base, with the same variable
//!   parts (`a[i]` and `a[i + 1]` both have `i` scaled by the element
//!   size), are compared by their constant offsets: byte ranges that do
//!   not overlap do not alias.
//! - **Calls.** `memory(...)`, `readnone`, `readonly` and `writeonly` on the
//!   call or the callee say what it may touch. A call that only touches its
//!   pointer arguments (`memory(argmem: ...)`) cannot touch a location that
//!   none of them may alias.
//!
//! The rules follow the Language Reference's pointer aliasing rules: memory
//! may only be accessed through a pointer based on the object it belongs to
//! (`docs/llvm-ir/01-module-structure.md`, section 14).

use crate::fx::{FxHashMap as HashMap, FxHashSet as HashSet};

use epitome_ir::module::*;
use epitome_ir::types::{Type, TypeId};

use crate::combine::eval;
use crate::ctx::Shared;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum AliasResult {
    NoAlias,
    /// Nothing is known.
    MayAlias,
    /// The ranges overlap but do not start at the same address.
    PartialAlias,
    /// Same start address.
    MustAlias,
}

/// A memory access: where it starts, and how many bytes, if known.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Location {
    pub ptr: Value,
    pub size: Option<u64>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct ModRef {
    pub mods: bool,
    pub refs: bool,
}

impl ModRef {
    pub const NONE: ModRef = ModRef { mods: false, refs: false };
    pub const ALL: ModRef = ModRef { mods: true, refs: true };
    pub const READ: ModRef = ModRef { mods: false, refs: true };
    pub const WRITE: ModRef = ModRef { mods: true, refs: false };
}

/// The objects a pointer can be based on.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Object {
    Alloca(LocalId),
    /// The result of a call marked `noalias`: fresh memory.
    Allocation(LocalId),
    NoaliasArg(LocalId),
    Global(GlobalId),
    Null,
    Unknown,
}

impl Object {
    fn identified(self) -> bool {
        !matches!(self, Object::Unknown)
    }
}

/// A pointer as base + constant offset + scaled variable terms.
#[derive(Clone, Debug, PartialEq)]
struct Decomposed {
    base: Value,
    offset: i64,
    terms: Vec<(Value, i64)>,
}

pub struct BasicAa<'s, 'm> {
    sh: &'s Shared<'m>,
    /// Allocas and allocations whose address never escapes.
    local_only: HashSet<LocalId>,
    /// Pointers already decomposed. A pointer's address never changes (SSA
    /// values are defined once, and a pass that replaces an operand puts an
    /// equal value in its place), so the answers stay right while the
    /// function is edited.
    decomposed: std::cell::RefCell<HashMap<Value, std::rc::Rc<Decomposed>>>,
}

/// The memory intrinsics, whose locations are their pointer and length
/// arguments.
enum MemIntrinsic {
    /// `memcpy` and `memmove`: (destination, source, length).
    Copy(Value, Value, Value),
    /// `memset`: (destination, length).
    Set(Value, Value),
}

impl<'s, 'm> BasicAa<'s, 'm> {
    pub fn new(sh: &'s Shared<'m>, f: &Function) -> Self {
        BasicAa { sh, local_only: not_captured(sh, f), decomposed: Default::default() }
    }

    /// `llvm.memcpy`, `llvm.memmove` or `llvm.memset` (and their `.inline`
    /// forms), if `call` is one and not volatile.
    fn mem_intrinsic(&self, call: &Call) -> Option<MemIntrinsic> {
        let g = self.sh.direct_callee(call)?;
        let name = self.sh.module.global(g).name.as_deref()?;
        let arg = |k: usize| call.args.get(k).map(|a| a.value);
        // The last argument is `isvolatile`.
        let volatile = arg(3).and_then(|v| self.int_value(v)) != Some(0);
        if volatile {
            return None;
        }
        if name.starts_with("llvm.memcpy.") || name.starts_with("llvm.memmove.") {
            Some(MemIntrinsic::Copy(arg(0)?, arg(1)?, arg(2)?))
        } else if name.starts_with("llvm.memset.") {
            Some(MemIntrinsic::Set(arg(0)?, arg(2)?))
        } else {
            None
        }
    }

    /// The bytes from `ptr` that a memory intrinsic of length `len` covers.
    fn span(&self, ptr: Value, len: Value) -> Location {
        Location { ptr, size: self.int_value(len).filter(|&n| n >= 0).map(|n| n as u64) }
    }

    /// Whether `inst` may write or read any memory at all, whatever the
    /// location: what memory SSA needs to know to place it.
    pub fn effects(&self, inst: &Inst) -> ModRef {
        match &inst.op {
            Op::Load { volatile: true, .. } | Op::Load { atomic: Some(_), .. } => ModRef::ALL,
            Op::Store { volatile: true, .. } | Op::Store { atomic: Some(_), .. } => ModRef::ALL,
            Op::Load { .. } => ModRef::READ,
            Op::Store { .. } => ModRef::WRITE,
            Op::Call(call) | Op::Invoke { call, .. } | Op::CallBr { call, .. } => {
                let sh = self.sh;
                let name =
                    sh.direct_callee(call).and_then(|g| sh.module.global(g).name.as_deref()).unwrap_or("").to_string();
                if ["llvm.dbg.", "llvm.assume", "llvm.experimental.noalias.scope.decl"]
                    .iter()
                    .any(|p| name.starts_with(p))
                {
                    return ModRef::NONE;
                }
                if name.starts_with("llvm.lifetime.") {
                    return ModRef::WRITE;
                }
                if let Some(mi) = self.mem_intrinsic(call) {
                    return match mi {
                        MemIntrinsic::Copy(..) => ModRef::ALL,
                        MemIntrinsic::Set(..) => ModRef::WRITE,
                    };
                }
                let mut e = MemoryEffects::parse(sh.module, &call.fn_attrs);
                if let Some(body) = sh.direct_callee(call).and_then(|g| sh.published(g)) {
                    e = e.intersect(MemoryEffects::parse(sh.module, &body.fn_attrs));
                }
                let k = e.argmem.union(e.other);
                ModRef { mods: k.0 & MemKind::WRITE.0 != 0, refs: k.0 & MemKind::READ.0 != 0 }
            }
            Op::AtomicRmw { .. } | Op::CmpXchg { .. } | Op::Fence { .. } | Op::VaArg { .. } => ModRef::ALL,
            _ => ModRef::NONE,
        }
    }

    /// Whether `ptr` points into an alloca or allocation of this function
    /// whose address never escapes: nothing outside the function, a caller
    /// or an exception handler, can read it.
    pub fn is_local_object(&self, f: &Function, ptr: Value) -> bool {
        self.is_local_only(self.object(f, self.underlying(f, ptr)))
    }

    /// The type of a constant.
    pub fn const_type(&self, c: ConstId) -> TypeId {
        self.sh.consts().get(c).ty
    }

    /// The location a load or store of type `ty` through `ptr` accesses.
    pub fn location(&self, ptr: Value, ty: TypeId) -> Location {
        Location { ptr, size: self.sh.layout.store_size(self.sh.types(), ty) }
    }

    /// The value a pointer is based on, through every `getelementptr` and
    /// `bitcast`, whether or not its offset can be worked out. Objects are
    /// identified from this, never from a partial decomposition, which may
    /// stop at a pointer still derived from the object.
    fn underlying(&self, f: &Function, mut v: Value) -> Value {
        for _ in 0..64 {
            let next = match v {
                Value::Local(l) => match f.locals[l.index()].def {
                    LocalDef::Inst(i) => match &f.insts[i.index()].op {
                        Op::Gep { ptr, .. } => Some(*ptr),
                        Op::Cast { op: CastOp::BitCast, value, .. } => Some(*value),
                        _ => None,
                    },
                    _ => None,
                },
                Value::Const(c) => match &self.sh.consts().get(c).kind {
                    ConstKind::Expr(ConstExpr::Gep { ptr, .. })
                    | ConstKind::Expr(ConstExpr::Cast { op: CastOp::BitCast, value: ptr }) => Some(Value::Const(*ptr)),
                    _ => None,
                },
                Value::Metadata(_) => None,
            };
            match next {
                Some(n) => v = n,
                None => return v,
            }
        }
        v
    }

    fn type_of(&self, f: &Function, v: Value) -> Option<TypeId> {
        match v {
            Value::Local(l) => Some(f.locals[l.index()].ty),
            Value::Const(c) => Some(self.sh.consts().get(c).ty),
            Value::Metadata(_) => None,
        }
    }

    fn int_value(&self, v: Value) -> Option<i64> {
        let Value::Const(c) = v else { return None };
        let k = eval::int_const(self.sh.consts(), self.sh.types(), c)?;
        Some(eval::sext(k.value, k.bits))
    }

    /// Walks `getelementptr`s back to the base pointer, once per pointer.
    fn decompose(&self, f: &Function, v: Value) -> std::rc::Rc<Decomposed> {
        if let Some(d) = self.decomposed.borrow().get(&v) {
            return d.clone();
        }
        let d = std::rc::Rc::new(self.decompose_uncached(f, v));
        self.decomposed.borrow_mut().insert(v, d.clone());
        d
    }

    fn decompose_uncached(&self, f: &Function, mut v: Value) -> Decomposed {
        let sh = self.sh;
        let (types, dl) = (sh.types(), &sh.layout);
        let mut offset: i64 = 0;
        let mut terms: Vec<(Value, i64)> = Vec::new();
        for _ in 0..32 {
            // The GEP (instruction or constant expression) that made `v`.
            let gep: Option<(TypeId, Value, Vec<Value>)> = match v {
                Value::Local(l) => match f.locals[l.index()].def {
                    LocalDef::Inst(i) => match &f.insts[i.index()].op {
                        Op::Gep { source_ty, ptr, indices, .. } => Some((*source_ty, *ptr, indices.to_vec())),
                        Op::Cast { op: CastOp::BitCast, value, .. } => {
                            v = *value;
                            continue;
                        }
                        _ => None,
                    },
                    _ => None,
                },
                Value::Const(c) => match &sh.consts().get(c).kind {
                    ConstKind::Expr(ConstExpr::Gep { source_ty, ptr, indices, .. }) => {
                        Some((*source_ty, Value::Const(*ptr), indices.iter().map(|&i| Value::Const(i)).collect()))
                    }
                    ConstKind::Expr(ConstExpr::Cast { op: CastOp::BitCast, value }) => {
                        v = Value::Const(*value);
                        continue;
                    }
                    _ => None,
                },
                Value::Metadata(_) => None,
            };
            let Some((source, ptr, indices)) = gep else { break };
            // Vectors of pointers are not followed.
            if !matches!(self.type_of(f, ptr).map(|t| types.get(t)), Some(Type::Ptr(_))) {
                break;
            }
            let mut add = 0i64;
            let mut add_terms: Vec<(Value, i64)> = Vec::new();
            let mut ty = source;
            let mut ok = true;
            for (k, &index) in indices.iter().enumerate() {
                let step = if k == 0 {
                    dl.alloc_size(types, ty)
                } else {
                    match types.get(ty).clone() {
                        Type::Struct { .. } | Type::Named(_) => {
                            let (fields, packed) = match types.get(ty).clone() {
                                Type::Struct { fields, packed } => (fields, packed),
                                Type::Named(n) => match &types.structs()[n as usize].body {
                                    Some(b) => (b.fields.clone(), b.packed),
                                    None => {
                                        ok = false;
                                        break;
                                    }
                                },
                                _ => unreachable!(),
                            };
                            let Some(field) = self.int_value(index) else {
                                ok = false;
                                break;
                            };
                            let Some(layout) = dl.struct_layout(types, &fields, packed) else {
                                ok = false;
                                break;
                            };
                            add = add.wrapping_add(layout.offsets[field as usize] as i64);
                            ty = fields[field as usize];
                            continue;
                        }
                        Type::Array { elem, .. } | Type::Vector { elem, .. } => {
                            ty = elem;
                            dl.alloc_size(types, elem)
                        }
                        _ => None,
                    }
                };
                let Some(step) = step else {
                    ok = false;
                    break;
                };
                match self.int_value(index) {
                    Some(n) => add = add.wrapping_add(n.wrapping_mul(step as i64)),
                    None => {
                        let (var, extra) = self.peel_add(f, index);
                        add = add.wrapping_add(extra.wrapping_mul(step as i64));
                        add_terms.push((var, step as i64));
                    }
                }
            }
            if !ok {
                break;
            }
            offset = offset.wrapping_add(add);
            terms.extend(add_terms);
            v = ptr;
        }
        // Merge terms of the same value.
        let mut merged: Vec<(Value, i64)> = Vec::new();
        for (var, s) in terms {
            match merged.iter_mut().find(|(x, _)| *x == var) {
                Some(e) => e.1 = e.1.wrapping_add(s),
                None => merged.push((var, s)),
            }
        }
        merged.retain(|(_, s)| *s != 0);
        merged.sort_by_key(|(x, _)| match x {
            Value::Local(l) => (0, l.0),
            Value::Const(c) => (1, c.0),
            Value::Metadata(m) => (2, m.0),
        });
        Decomposed { base: v, offset, terms: merged }
    }

    /// `x + C` as an index: `x` and `C`, when the addition cannot wrap at a
    /// width other than the address width. `(x + C) * s` is then `x * s +
    /// C * s` exactly.
    fn peel_add(&self, f: &Function, index: Value) -> (Value, i64) {
        let Value::Local(l) = index else { return (index, 0) };
        let LocalDef::Inst(i) = f.locals[l.index()].def else { return (index, 0) };
        let Op::Binary { op: BinOp::Add, flags, lhs, rhs, .. } = f.insts[i.index()].op else { return (index, 0) };
        let Some(c) = self.int_value(rhs) else { return (index, 0) };
        let width = self.type_of(f, index).and_then(|t| eval::int_bits(self.sh.types(), t));
        let full_width = width == Some(self.sh.layout.pointer(0).index_bits);
        if full_width || flags.contains(IntFlags::NSW) { (lhs, c) } else { (index, 0) }
    }

    fn object(&self, f: &Function, base: Value) -> Object {
        let sh = self.sh;
        match base {
            Value::Local(l) => match f.locals[l.index()].def {
                LocalDef::Inst(i) => match &f.insts[i.index()].op {
                    Op::Alloca { .. } => Object::Alloca(l),
                    Op::Call(call) if crate::attrs::has_named(sh.module, &call.ret_attrs, "noalias") => {
                        Object::Allocation(l)
                    }
                    _ => Object::Unknown,
                },
                LocalDef::Param(k) => {
                    let attrs = &f.params[k as usize].attrs;
                    if crate::attrs::has_named(sh.module, attrs, "noalias")
                        || crate::attrs::has_named(sh.module, attrs, "byval")
                    {
                        Object::NoaliasArg(l)
                    } else {
                        Object::Unknown
                    }
                }
                _ => Object::Unknown,
            },
            Value::Const(c) => {
                let k = sh.consts().get(c);
                match k.kind {
                    ConstKind::Global(g) => match sh.module.global(g).kind {
                        GlobalKind::Variable(_) | GlobalKind::Function(_) => Object::Global(g),
                        _ => Object::Unknown,
                    },
                    ConstKind::Null if matches!(sh.types().get(k.ty), Type::Ptr(0)) => Object::Null,
                    _ => Object::Unknown,
                }
            }
            Value::Metadata(_) => Object::Unknown,
        }
    }

    /// True if no pointer to `o` can exist outside the function's own
    /// loads, stores and offsets.
    fn is_local_only(&self, o: Object) -> bool {
        matches!(o, Object::Alloca(l) | Object::Allocation(l) if self.local_only.contains(&l))
    }

    /// Like `is_local_only`, and also a `noalias` argument the function
    /// does not let escape: memory modified through it is not accessed
    /// through any pointer not based on it, and nothing here makes another
    /// pointer based on it. This holds for the function's own accesses,
    /// not for what a callee may do, so calls do not use it.
    fn is_unreachable_from_others(&self, o: Object) -> bool {
        self.is_local_only(o) || matches!(o, Object::NoaliasArg(l) if self.local_only.contains(&l))
    }

    pub fn alias(&self, f: &Function, a: Location, b: Location) -> AliasResult {
        if a.ptr == b.ptr {
            return AliasResult::MustAlias;
        }
        let (da, db) = (self.decompose(f, a.ptr), self.decompose(f, b.ptr));
        if da.base == db.base {
            if da.terms != db.terms {
                return AliasResult::MayAlias;
            }
            let (oa, ob) = (da.offset, db.offset);
            return match (a.size, b.size) {
                (Some(sa), Some(sb)) => {
                    if oa.wrapping_add(sa as i64) <= ob || ob.wrapping_add(sb as i64) <= oa {
                        AliasResult::NoAlias
                    } else if oa == ob {
                        if sa == sb { AliasResult::MustAlias } else { AliasResult::PartialAlias }
                    } else {
                        AliasResult::PartialAlias
                    }
                }
                _ if oa == ob => AliasResult::MustAlias,
                _ => AliasResult::MayAlias,
            };
        }
        let (ua, ub) = (self.underlying(f, a.ptr), self.underlying(f, b.ptr));
        if ua == ub {
            // The same object, by paths whose offsets cannot be compared.
            return AliasResult::MayAlias;
        }
        let (oa, ob) = (self.object(f, ua), self.object(f, ub));
        if oa == Object::Null || ob == Object::Null {
            // No access through null is valid.
            return AliasResult::NoAlias;
        }
        if oa.identified() && ob.identified() {
            return AliasResult::NoAlias;
        }
        // Every pointer based on a local-only object leads back to it
        // through offsets, so a different underlying value points elsewhere.
        if self.is_unreachable_from_others(oa) || self.is_unreachable_from_others(ob) {
            return AliasResult::NoAlias;
        }
        AliasResult::MayAlias
    }

    /// Whether `inst` may write (`mods`) or read (`refs`) the location.
    pub fn mod_ref(&self, f: &Function, inst: &Inst, loc: Location) -> ModRef {
        let sh = self.sh;
        let touches = |ptr: Value, ty: Option<TypeId>| {
            let size = ty.and_then(|t| sh.layout.store_size(sh.types(), t));
            self.alias(f, loc, Location { ptr, size }) != AliasResult::NoAlias
        };
        match &inst.op {
            Op::Load { volatile: true, .. } | Op::Load { atomic: Some(_), .. } => ModRef::ALL,
            Op::Store { volatile: true, .. } | Op::Store { atomic: Some(_), .. } => ModRef::ALL,
            Op::Load { ptr, .. } => {
                let ty = inst.result.map(|r| f.locals[r.index()].ty);
                ModRef { mods: false, refs: touches(*ptr, ty) }
            }
            Op::Store { ptr, value, .. } => ModRef { mods: touches(*ptr, self.type_of(f, *value)), refs: false },
            Op::Call(call) | Op::Invoke { call, .. } | Op::CallBr { call, .. } => match self.mem_intrinsic(call) {
                // Its own bytes: the destination written, the source read.
                Some(MemIntrinsic::Copy(dst, src, len)) => ModRef {
                    mods: self.alias(f, loc, self.span(dst, len)) != AliasResult::NoAlias,
                    refs: self.alias(f, loc, self.span(src, len)) != AliasResult::NoAlias,
                },
                Some(MemIntrinsic::Set(dst, len)) => {
                    ModRef { mods: self.alias(f, loc, self.span(dst, len)) != AliasResult::NoAlias, refs: false }
                }
                None => self.call_mod_ref(f, call, loc),
            },
            Op::AtomicRmw { .. } | Op::CmpXchg { .. } | Op::Fence { .. } | Op::VaArg { .. } => ModRef::ALL,
            _ => ModRef::NONE,
        }
    }

    fn call_mod_ref(&self, f: &Function, call: &Call, loc: Location) -> ModRef {
        let sh = self.sh;
        let m = sh.module;
        let callee = sh.direct_callee(call);
        let name = callee.and_then(|g| m.global(g).name.as_deref()).unwrap_or("");
        if ["llvm.dbg.", "llvm.assume", "llvm.experimental.noalias.scope.decl"].iter().any(|p| name.starts_with(p)) {
            return ModRef::NONE;
        }
        // No call can reach memory nothing outside this function points to.
        let object = self.object(f, self.underlying(f, loc.ptr));
        if self.is_local_only(object) {
            return ModRef::NONE;
        }
        if name.starts_with("llvm.lifetime.") {
            // Treated as a write of the object, so nothing is carried across.
            let hit = call.args.iter().any(|a| {
                matches!(self.type_of(f, a.value).map(|t| sh.types().get(t)), Some(Type::Ptr(_)))
                    && self.alias(f, loc, Location { ptr: a.value, size: None }) != AliasResult::NoAlias
            });
            return if hit { ModRef { mods: true, refs: false } } else { ModRef::NONE };
        }
        let mut effects = MemoryEffects::parse(m, &call.fn_attrs);
        if let Some(body) = callee.and_then(|g| sh.published(g)) {
            effects = effects.intersect(MemoryEffects::parse(m, &body.fn_attrs));
        }
        let mut result = effects.other;
        if effects.argmem != MemKind::NONE {
            let hit = call.args.iter().any(|a| {
                matches!(self.type_of(f, a.value).map(|t| sh.types().get(t)), Some(Type::Ptr(_)))
                    && self.alias(f, loc, Location { ptr: a.value, size: None }) != AliasResult::NoAlias
            });
            if hit {
                result = result.union(effects.argmem);
            }
        }
        ModRef { mods: result.0 & MemKind::WRITE.0 != 0, refs: result.0 & MemKind::READ.0 != 0 }
    }
}

/// None, read, write or both, as bits.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct MemKind(u8);

impl MemKind {
    const NONE: MemKind = MemKind(0);
    const READ: MemKind = MemKind(1);
    const WRITE: MemKind = MemKind(2);
    const ALL: MemKind = MemKind(3);

    fn parse(s: &str) -> Option<MemKind> {
        Some(match s.trim() {
            "none" => MemKind::NONE,
            "read" => MemKind::READ,
            "write" => MemKind::WRITE,
            "readwrite" => MemKind::ALL,
            _ => return None,
        })
    }

    fn union(self, o: MemKind) -> MemKind {
        MemKind(self.0 | o.0)
    }
}

/// What a call may do to memory: through its pointer arguments, and to
/// everything else it can reach.
#[derive(Clone, Copy, Debug)]
struct MemoryEffects {
    argmem: MemKind,
    other: MemKind,
}

impl MemoryEffects {
    fn intersect(self, o: MemoryEffects) -> MemoryEffects {
        MemoryEffects { argmem: MemKind(self.argmem.0 & o.argmem.0), other: MemKind(self.other.0 & o.other.0) }
    }

    /// From `memory(...)` and the older `readnone`, `readonly`, `writeonly`
    /// and `argmemonly`. Memory that is not accessible from this module
    /// (`inaccessiblemem`) and target state cannot alias a location here,
    /// so they are ignored; `errnomem` counts as other memory.
    fn parse(m: &Module, attrs: &[Attr]) -> MemoryEffects {
        let mut e = MemoryEffects { argmem: MemKind::ALL, other: MemKind::ALL };
        if let Some(text) = crate::attrs::paren_text(m, attrs, "memory") {
            let mut default = MemKind::ALL;
            let mut argmem = None;
            let mut errno = None;
            let mut listed_default = false;
            for part in text.split(',') {
                match part.split_once(':') {
                    None => {
                        if let Some(k) = MemKind::parse(part) {
                            default = k;
                            listed_default = true;
                        }
                    }
                    Some((loc, kind)) => {
                        let Some(kind) = MemKind::parse(kind) else { continue };
                        match loc.trim() {
                            "argmem" => argmem = Some(kind),
                            "errnomem" => errno = Some(kind),
                            _ => {}
                        }
                    }
                }
            }
            if !listed_default {
                default = MemKind::NONE;
            }
            e = MemoryEffects { argmem: argmem.unwrap_or(default), other: default.union(errno.unwrap_or(default)) };
        }
        let has = |w: &str| crate::attrs::has(m, attrs, w);
        if has("readnone") {
            e = e.intersect(MemoryEffects { argmem: MemKind::NONE, other: MemKind::NONE });
        }
        if has("readonly") {
            e = e.intersect(MemoryEffects { argmem: MemKind::READ, other: MemKind::READ });
        }
        if has("writeonly") {
            e = e.intersect(MemoryEffects { argmem: MemKind::WRITE, other: MemKind::WRITE });
        }
        if has("argmemonly") {
            e = e.intersect(MemoryEffects { argmem: MemKind::ALL, other: MemKind::NONE });
        }
        e
    }
}

/// Allocas and `noalias` call results whose address never leaves the
/// function's own loads, stores and offsets.
fn not_captured(sh: &Shared, f: &Function) -> HashSet<LocalId> {
    let mut users: HashMap<LocalId, Vec<InstId>> = HashMap::default();
    for i in f.insts_in_order() {
        f.insts[i.index()].op.for_each_value(|v| {
            if let Value::Local(l) = v {
                users.entry(*l).or_default().push(i);
            }
        });
    }
    let mut out = HashSet::default();
    let mut candidates: Vec<LocalId> =
        f.params.iter().filter(|p| crate::attrs::has_named(sh.module, &p.attrs, "noalias")).map(|p| p.local).collect();
    for i in f.insts_in_order() {
        let inst = &f.insts[i.index()];
        let Some(r) = inst.result else { continue };
        let candidate = match &inst.op {
            Op::Alloca { .. } => true,
            Op::Call(call) => crate::attrs::has_named(sh.module, &call.ret_attrs, "noalias"),
            _ => false,
        };
        if candidate {
            candidates.push(r);
        }
    }
    for r in candidates {
        let mut work = vec![r];
        let mut seen = HashSet::default();
        let mut escapes = false;
        while let Some(p) = work.pop() {
            if !seen.insert(p) {
                continue;
            }
            let me = Value::Local(p);
            for &u in users.get(&p).map(|v| v.as_slice()).unwrap_or(&[]) {
                match &f.insts[u.index()].op {
                    Op::Load { ptr, .. } if *ptr == me => {}
                    Op::Store { ptr, value, .. } if *ptr == me && *value != me => {}
                    Op::Gep { ptr, indices, .. } if *ptr == me && !indices.contains(&me) => {
                        work.push(f.insts[u.index()].result.unwrap());
                    }
                    Op::Cast { op: CastOp::BitCast, .. } => work.push(f.insts[u.index()].result.unwrap()),
                    Op::Call(call)
                        if sh
                            .direct_callee(call)
                            .and_then(|g| sh.module.global(g).name.as_deref())
                            .is_some_and(|n| n.starts_with("llvm.lifetime.")) => {}
                    _ => {
                        escapes = true;
                        break;
                    }
                }
            }
            if escapes {
                break;
            }
        }
        if !escapes {
            out.insert(r);
        }
    }
    out
}
