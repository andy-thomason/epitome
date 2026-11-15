//! A static's initial bytes, worked out at compile time by running the
//! function lowering made to compute its value (`lower_static_init`) over
//! the IR: memory as bytes (stack slots, and globals as their
//! initialisers give them), with the addresses of globals kept as
//! relocations; integer arithmetic, comparisons, casts and control flow;
//! and calls of functions with bodies, followed through. When it reaches
//! its end, what it stored in the static is the static's initialiser, a
//! constant, and nothing need run at start-up. That is what a
//! `#[thread_local]` static needs: each thread's copy starts as that
//! image. Anything else (floating point, atomics, a call of a function
//! with no body, an instruction budget spent) gives up, and the function
//! runs at start-up as before.

use epitome_ir::layout::DataLayout;
use epitome_ir::module::{BinOp, CastOp, ConstExpr, ConstId, ConstKind, Constant, Function, GlobalId, GlobalKind, ICmpPred, Module, Op, Value};
use epitome_ir::types::{Type, TypeId};
use std::collections::BTreeMap;

/// What a pointer points into.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Base {
    /// A block of memory of the run (a stack slot, or a global's copy).
    Mem(usize),
    /// A global whose contents the run does not touch (a function).
    Global(GlobalId),
}

#[derive(Clone, Copy, Debug)]
enum V {
    Int(u128),
    /// An address: into `Base` at an offset, or (no base) a plain number
    /// (null is 0).
    Ptr(Option<Base>, i64),
}

/// A block of memory: its bytes, and where it holds addresses.
struct Mem {
    bytes: Vec<u8>,
    ptrs: BTreeMap<u64, (Base, i64)>,
    /// The global it is the copy of, if any.
    global: Option<GlobalId>,
    writable: bool,
}

/// A static's initial bytes, with the addresses of globals in them: at
/// each offset, the global and an addend.
pub(crate) struct Image {
    pub bytes: Vec<u8>,
    pub ptrs: Vec<(u64, GlobalId, i64)>,
}

struct Run<'m> {
    m: &'m Module,
    dl: DataLayout,
    mems: Vec<Mem>,
    globals: BTreeMap<GlobalId, usize>,
    target: GlobalId,
    steps: usize,
    depth: usize,
}

const BUDGET: usize = 1_000_000;
const DEPTH: usize = 64;

/// Runs `init` (a `void()` function storing static `target`'s value) and
/// gives what it leaves in `target` (of `size` bytes).
pub(crate) fn run(m: &Module, init: GlobalId, target: GlobalId, size: u64) -> Option<Image> {
    let dl = DataLayout::parse(&String::from_utf8_lossy(m.data_layout.as_deref().unwrap_or_default())).ok()?;
    let mut r = Run { m, dl, mems: Vec::new(), globals: BTreeMap::new(), target, steps: 0, depth: 0 };
    let t = r.mems.len();
    r.mems.push(Mem { bytes: vec![0; size as usize], ptrs: BTreeMap::new(), global: Some(target), writable: true });
    r.globals.insert(target, t);
    r.call(init, Vec::new())?;
    let mem = &r.mems[t];
    let mut ptrs = Vec::new();
    for (&off, &(b, add)) in &mem.ptrs {
        let g = match b {
            Base::Global(g) => g,
            Base::Mem(i) => r.mems[i].global?,
        };
        ptrs.push((off, g, add));
    }
    Some(Image { bytes: mem.bytes.clone(), ptrs })
}

/// The image as a constant: a packed struct of its bytes and addresses
/// (or just its bytes).
pub(crate) fn constant(m: &mut Module, img: &Image) -> ConstId {
    let bytes = |m: &mut Module, b: &[u8]| match m.const_bytes(b) {
        Value::Const(c) => c,
        _ => unreachable!("bytes are a constant"),
    };
    if img.ptrs.is_empty() {
        return bytes(m, &img.bytes);
    }
    let ptr = m.types.ptr(0);
    let i8t = m.types.int(8);
    let ps = 8u64;
    let mut fields = Vec::new();
    let mut tys = Vec::new();
    let mut at = 0u64;
    for &(off, g, add) in &img.ptrs {
        if off > at {
            let c = bytes(m, &img.bytes[at as usize..off as usize]);
            tys.push(m.consts.get(c).ty);
            fields.push(c);
        }
        let base = m.consts.intern(Constant { ty: ptr, kind: ConstKind::Global(g) });
        let c = if add == 0 {
            base
        } else {
            let i64t = m.types.int(64);
            let k = m.consts.intern(Constant::int(&m.types, i64t, add));
            m.consts.intern(Constant {
                ty: ptr,
                kind: ConstKind::Expr(ConstExpr::Gep { flags: Default::default(), inrange: None, source_ty: i8t, ptr: base, indices: vec![k].into() }),
            })
        };
        tys.push(ptr);
        fields.push(c);
        at = off + ps;
    }
    if (at as usize) < img.bytes.len() {
        let c = bytes(m, &img.bytes[at as usize..]);
        tys.push(m.consts.get(c).ty);
        fields.push(c);
    }
    let ty = m.types.intern(Type::Struct { packed: true, fields: tys.into() });
    m.consts.intern(Constant { ty, kind: ConstKind::Struct(fields) })
}

fn mask(v: u128, bits: u32) -> u128 {
    if bits >= 128 { v } else { v & ((1u128 << bits) - 1) }
}

fn sext(v: u128, bits: u32) -> i128 {
    if bits >= 128 {
        return v as i128;
    }
    let s = 128 - bits;
    ((v << s) as i128) >> s
}

impl<'m> Run<'m> {
    fn ty(&self, t: TypeId) -> &Type {
        self.m.types.get(t)
    }

    fn bits(&self, t: TypeId) -> Option<u32> {
        match self.ty(t) {
            Type::Int(n) => Some(*n),
            Type::Ptr(_) => Some(64),
            _ => None,
        }
    }

    fn size(&self, t: TypeId) -> Option<u64> {
        self.dl.alloc_size(&self.m.types, t)
    }

    fn store_size(&self, t: TypeId) -> Option<u64> {
        self.dl.store_size(&self.m.types, t)
    }

    /// The block of memory a global's contents are in, made the first time
    /// from its initialiser (only `target` may be written).
    fn global_mem(&mut self, g: GlobalId) -> Option<Base> {
        if let Some(&i) = self.globals.get(&g) {
            return Some(Base::Mem(i));
        }
        let GlobalKind::Variable(v) = &self.m.global(g).kind else { return Some(Base::Global(g)) };
        let size = self.size(v.ty)? as usize;
        let mut mem = Mem { bytes: vec![0; size], ptrs: BTreeMap::new(), global: Some(g), writable: false };
        if let Some(c) = v.init {
            self.serialize(c, &mut mem, 0)?;
        } else {
            return Some(Base::Global(g));
        }
        let i = self.mems.len();
        self.mems.push(mem);
        self.globals.insert(g, i);
        Some(Base::Mem(i))
    }

    /// Writes constant `c` into `mem` at `off`.
    fn serialize(&mut self, c: ConstId, mem: &mut Mem, off: u64) -> Option<()> {
        let k = self.m.consts.get(c).clone();
        match &k.kind {
            ConstKind::Zero | ConstKind::Undef | ConstKind::Poison | ConstKind::Null => Some(()),
            ConstKind::Int(a) => {
                let n = self.store_size(k.ty)? as usize;
                let mut v: u128 = 0;
                for (i, w) in a.words().iter().take(2).enumerate() {
                    v |= (*w as u128) << (64 * i);
                }
                for i in 0..n.min(16) {
                    mem.bytes[off as usize + i] = (v >> (8 * i)) as u8;
                }
                Some(())
            }
            ConstKind::Float(bits) => {
                let n = self.store_size(k.ty)? as usize;
                for i in 0..n.min(16) {
                    mem.bytes[off as usize + i] = (bits >> (8 * i)) as u8;
                }
                Some(())
            }
            ConstKind::Bytes(b) => {
                mem.bytes[off as usize..off as usize + b.len()].copy_from_slice(b);
                Some(())
            }
            ConstKind::Array(es) => {
                let Type::Array { elem, .. } = *self.ty(k.ty) else { return None };
                let stride = self.size(elem)?;
                for (i, &e) in es.iter().enumerate() {
                    self.serialize(e, mem, off + i as u64 * stride)?;
                }
                Some(())
            }
            ConstKind::Struct(es) => {
                let Type::Struct { packed, fields } = self.ty(k.ty).clone() else { return None };
                let sl = self.dl.struct_layout(&self.m.types, &fields, packed)?;
                for (i, &e) in es.iter().enumerate() {
                    self.serialize(e, mem, off + sl.offsets[i])?;
                }
                Some(())
            }
            ConstKind::Global(_) | ConstKind::Expr(_) => match self.constant(c)? {
                V::Ptr(Some(b), a) => {
                    mem.ptrs.insert(off, (b, a));
                    Some(())
                }
                V::Ptr(None, a) => {
                    for i in 0..8 {
                        mem.bytes[off as usize + i] = (a >> (8 * i)) as u8;
                    }
                    Some(())
                }
                V::Int(_) => None,
            },
            _ => None,
        }
    }

    /// A constant's value.
    fn constant(&mut self, c: ConstId) -> Option<V> {
        let k = self.m.consts.get(c).clone();
        Some(match &k.kind {
            ConstKind::Int(a) => {
                let mut v: u128 = 0;
                for (i, w) in a.words().iter().take(2).enumerate() {
                    v |= (*w as u128) << (64 * i);
                }
                V::Int(v)
            }
            ConstKind::Float(bits) => V::Int(*bits),
            ConstKind::Null => V::Ptr(None, 0),
            ConstKind::Zero | ConstKind::Undef | ConstKind::Poison => match self.ty(k.ty) {
                Type::Ptr(_) => V::Ptr(None, 0),
                Type::Int(_) | Type::Half | Type::Float | Type::Double | Type::Fp128 => V::Int(0),
                _ => return None,
            },
            ConstKind::Global(g) => {
                // A global the run reads or writes has its block; any
                // other (a function) is a relocation as it is.
                let b = if *g == self.target { Base::Mem(self.globals[g]) } else { self.global_mem_or_self(*g) };
                V::Ptr(Some(b), 0)
            }
            ConstKind::Expr(ConstExpr::Gep { source_ty, ptr, indices, .. }) => {
                let base = self.constant(*ptr)?;
                let idx: Vec<V> = indices.iter().map(|&i| self.constant(i)).collect::<Option<_>>()?;
                self.gep(base, *source_ty, &idx)?
            }
            ConstKind::Expr(ConstExpr::Cast { op: CastOp::BitCast | CastOp::AddrSpaceCast, value }) => self.constant(*value)?,
            _ => return None,
        })
    }

    /// A global's address as the run sees it: its block if it is a
    /// variable with an initialiser (whose contents may be read), else the
    /// global itself.
    fn global_mem_or_self(&mut self, g: GlobalId) -> Base {
        match &self.m.global(g).kind {
            GlobalKind::Variable(v) if v.init.is_some() => self.global_mem(g).unwrap_or(Base::Global(g)),
            _ => Base::Global(g),
        }
    }

    fn gep(&self, base: V, source_ty: TypeId, idx: &[V]) -> Option<V> {
        let V::Ptr(b, mut off) = base else { return None };
        let int = |v: &V| match v {
            V::Int(x) => Some(*x as i64),
            _ => None,
        };
        let first = int(idx.first()?)?;
        off += first.checked_mul(self.size(source_ty)? as i64)?;
        let mut t = source_ty;
        for i in &idx[1..] {
            let i = int(i)?;
            match self.ty(t).clone() {
                Type::Array { elem, .. } | Type::Vector { elem, .. } => {
                    off += i * self.size(elem)? as i64;
                    t = elem;
                }
                Type::Struct { packed, fields } => {
                    let sl = self.dl.struct_layout(&self.m.types, &fields, packed)?;
                    off += *sl.offsets.get(i as usize)? as i64;
                    t = fields[i as usize];
                }
                _ => return None,
            }
        }
        Some(V::Ptr(b, off))
    }

    fn mem_at(&self, p: V) -> Option<(usize, u64)> {
        match p {
            V::Ptr(Some(Base::Mem(i)), off) if off >= 0 => Some((i, off as u64)),
            _ => None,
        }
    }

    fn load(&self, p: V, t: TypeId) -> Option<V> {
        let (i, off) = self.mem_at(p)?;
        let n = self.store_size(t)?;
        let mem = &self.mems[i];
        if off + n > mem.bytes.len() as u64 {
            return None;
        }
        if matches!(self.ty(t), Type::Ptr(_)) {
            if let Some(&(b, a)) = mem.ptrs.get(&off) {
                return Some(V::Ptr(Some(b), a));
            }
        }
        if mem.ptrs.range(off.saturating_sub(7)..off + n).next().is_some() {
            return None;
        }
        let mut v: u128 = 0;
        for k in 0..n.min(16) {
            v |= (mem.bytes[(off + k) as usize] as u128) << (8 * k);
        }
        Some(match self.ty(t) {
            Type::Ptr(_) => V::Ptr(None, v as i64),
            _ => V::Int(v),
        })
    }

    fn store(&mut self, p: V, t: TypeId, v: V) -> Option<()> {
        let (i, off) = self.mem_at(p)?;
        let n = self.store_size(t)?;
        if !self.mems[i].writable || off + n > self.mems[i].bytes.len() as u64 {
            return None;
        }
        let mem = &mut self.mems[i];
        let stale: Vec<u64> = mem.ptrs.range(off.saturating_sub(7)..off + n).map(|x| *x.0).collect();
        for s in stale {
            mem.ptrs.remove(&s);
        }
        match v {
            V::Ptr(Some(b), a) => {
                mem.ptrs.insert(off, (b, a));
                for k in 0..n {
                    mem.bytes[(off + k) as usize] = 0;
                }
            }
            V::Ptr(None, a) => {
                for k in 0..n.min(8) {
                    mem.bytes[(off + k) as usize] = (a >> (8 * k)) as u8;
                }
            }
            V::Int(x) => {
                for k in 0..n.min(16) {
                    mem.bytes[(off + k) as usize] = (x >> (8 * k)) as u8;
                }
            }
        }
        Some(())
    }

    fn fill(&mut self, dst: V, b: u8, n: u64) -> Option<()> {
        let (i, off) = self.mem_at(dst)?;
        let mem = &mut self.mems[i];
        if !mem.writable || off + n > mem.bytes.len() as u64 {
            return None;
        }
        let stale: Vec<u64> = mem.ptrs.range(off.saturating_sub(7)..off + n).map(|x| *x.0).collect();
        for s in stale {
            mem.ptrs.remove(&s);
        }
        mem.bytes[off as usize..(off + n) as usize].fill(b);
        Some(())
    }

    fn copy(&mut self, dst: V, src: V, n: u64) -> Option<()> {
        let (di, doff) = self.mem_at(dst)?;
        let (si, soff) = self.mem_at(src)?;
        if !self.mems[di].writable || doff + n > self.mems[di].bytes.len() as u64 || soff + n > self.mems[si].bytes.len() as u64 {
            return None;
        }
        let bytes = self.mems[si].bytes[soff as usize..(soff + n) as usize].to_vec();
        let ptrs: Vec<(u64, (Base, i64))> = self.mems[si].ptrs.range(soff..soff + n).map(|(&o, &p)| (o - soff, p)).collect();
        let mem = &mut self.mems[di];
        let stale: Vec<u64> = mem.ptrs.range(doff.saturating_sub(7)..doff + n).map(|x| *x.0).collect();
        for s in stale {
            mem.ptrs.remove(&s);
        }
        mem.bytes[doff as usize..(doff + n) as usize].copy_from_slice(&bytes);
        for (o, p) in ptrs {
            mem.ptrs.insert(doff + o, p);
        }
        Some(())
    }

    /// Calls function `g` with `args`: its result, if it gives one.
    fn call(&mut self, g: GlobalId, args: Vec<V>) -> Option<Option<V>> {
        let GlobalKind::Function(f) = &self.m.global(g).kind else { return None };
        if !f.has_body || self.depth >= DEPTH {
            return None;
        }
        self.depth += 1;
        let r = self.body(f, args);
        self.depth -= 1;
        r
    }

    fn body(&mut self, f: &'m Function, args: Vec<V>) -> Option<Option<V>> {
        let mut locals: Vec<Option<V>> = vec![None; f.locals.len()];
        for (i, p) in f.params.iter().enumerate() {
            locals[p.local.index()] = Some(*args.get(i)?);
        }
        let mut block = *f.layout.first()?;
        let mut prev = block;
        loop {
            let mut next = None;
            for &ii in &f.blocks[block.index()].insts {
                self.steps += 1;
                if self.steps > BUDGET {
                    return None;
                }
                let inst = &f.insts[ii.index()];
                let val = |run: &mut Self, locals: &[Option<V>], v: Value| -> Option<V> {
                    match v {
                        Value::Local(l) => locals[l.index()],
                        Value::Const(c) => run.constant(c),
                        Value::Metadata(_) => None,
                    }
                };
                let rty = inst.result.map(|r| f.locals[r.index()].ty);
                let out: Option<V> = match &inst.op {
                    Op::Ret(v) => {
                        return Some(match v {
                            Some(v) => Some(val(self, &locals, *v)?),
                            None => None,
                        });
                    }
                    Op::Br(b) => {
                        next = Some(*b);
                        None
                    }
                    Op::CondBr { cond, then_dest, else_dest } => {
                        let V::Int(c) = val(self, &locals, *cond)? else { return None };
                        next = Some(if c & 1 != 0 { *then_dest } else { *else_dest });
                        None
                    }
                    Op::Switch { value, default, cases } => {
                        let V::Int(x) = val(self, &locals, *value)? else { return None };
                        let mut to = *default;
                        for &(c, b) in cases.iter() {
                            if let V::Int(k) = self.constant(c)?
                                && k == x
                            {
                                to = b;
                            }
                        }
                        next = Some(to);
                        None
                    }
                    Op::Alloca { ty, count, .. } => {
                        let n = match count {
                            Some(c) => match val(self, &locals, *c)? {
                                V::Int(n) => n as u64,
                                _ => return None,
                            },
                            None => 1,
                        };
                        let size = self.size(*ty)? * n;
                        let i = self.mems.len();
                        self.mems.push(Mem { bytes: vec![0; size as usize], ptrs: BTreeMap::new(), global: None, writable: true });
                        Some(V::Ptr(Some(Base::Mem(i)), 0))
                    }
                    Op::Load { ptr, atomic: None, .. } => {
                        let p = val(self, &locals, *ptr)?;
                        Some(self.load(p, rty?)?)
                    }
                    Op::Store { value, ptr, atomic: None, .. } => {
                        let p = val(self, &locals, *ptr)?;
                        let v = val(self, &locals, *value)?;
                        let t = match *value {
                            Value::Local(l) => f.locals[l.index()].ty,
                            Value::Const(c) => self.m.consts.get(c).ty,
                            Value::Metadata(_) => return None,
                        };
                        self.store(p, t, v)?;
                        None
                    }
                    Op::Gep { source_ty, ptr, indices, .. } => {
                        let p = val(self, &locals, *ptr)?;
                        let idx: Vec<V> = indices.iter().map(|&i| val(self, &locals, i)).collect::<Option<_>>()?;
                        Some(self.gep(p, *source_ty, &idx)?)
                    }
                    Op::Binary { op, lhs, rhs, .. } => {
                        let bits = self.bits(rty?)?;
                        let (V::Int(a), V::Int(b)) = (val(self, &locals, *lhs)?, val(self, &locals, *rhs)?) else { return None };
                        let (a, b) = (mask(a, bits), mask(b, bits));
                        let r = match op {
                            BinOp::Add => a.wrapping_add(b),
                            BinOp::Sub => a.wrapping_sub(b),
                            BinOp::Mul => a.wrapping_mul(b),
                            BinOp::UDiv if b != 0 => a / b,
                            BinOp::URem if b != 0 => a % b,
                            BinOp::SDiv if b != 0 => sext(a, bits).wrapping_div(sext(b, bits)) as u128,
                            BinOp::SRem if b != 0 => sext(a, bits).wrapping_rem(sext(b, bits)) as u128,
                            BinOp::Shl if b < bits as u128 => a << b,
                            BinOp::LShr if b < bits as u128 => a >> b,
                            BinOp::AShr if b < bits as u128 => (sext(a, bits) >> b) as u128,
                            BinOp::And => a & b,
                            BinOp::Or => a | b,
                            BinOp::Xor => a ^ b,
                            _ => return None,
                        };
                        Some(V::Int(mask(r, bits)))
                    }
                    Op::ICmp { pred, lhs, rhs, .. } => {
                        let (a, b) = (val(self, &locals, *lhs)?, val(self, &locals, *rhs)?);
                        let t = match *lhs {
                            Value::Local(l) => f.locals[l.index()].ty,
                            Value::Const(c) => self.m.consts.get(c).ty,
                            Value::Metadata(_) => return None,
                        };
                        let r = match (a, b) {
                            (V::Int(a), V::Int(b)) => {
                                let bits = self.bits(t)?;
                                let (ua, ub) = (mask(a, bits), mask(b, bits));
                                let (sa, sb) = (sext(a, bits), sext(b, bits));
                                match pred {
                                    ICmpPred::Eq => ua == ub,
                                    ICmpPred::Ne => ua != ub,
                                    ICmpPred::Ult => ua < ub,
                                    ICmpPred::Ule => ua <= ub,
                                    ICmpPred::Ugt => ua > ub,
                                    ICmpPred::Uge => ua >= ub,
                                    ICmpPred::Slt => sa < sb,
                                    ICmpPred::Sle => sa <= sb,
                                    ICmpPred::Sgt => sa > sb,
                                    ICmpPred::Sge => sa >= sb,
                                }
                            }
                            (V::Ptr(ba, oa), V::Ptr(bb, ob)) if ba == bb => match pred {
                                ICmpPred::Eq => oa == ob,
                                ICmpPred::Ne => oa != ob,
                                _ => return None,
                            },
                            // An address of something against null.
                            (V::Ptr(Some(_), _), V::Ptr(None, 0)) | (V::Ptr(None, 0), V::Ptr(Some(_), _)) => match pred {
                                ICmpPred::Eq => false,
                                ICmpPred::Ne => true,
                                _ => return None,
                            },
                            _ => return None,
                        };
                        Some(V::Int(r as u128))
                    }
                    Op::Select { cond, then_value, else_value, .. } => {
                        let V::Int(c) = val(self, &locals, *cond)? else { return None };
                        Some(if c & 1 != 0 { val(self, &locals, *then_value)? } else { val(self, &locals, *else_value)? })
                    }
                    Op::Phi { incoming, .. } => {
                        let &(v, _) = incoming.iter().find(|x| x.1 == prev)?;
                        Some(val(self, &locals, v)?)
                    }
                    Op::Freeze(v) => Some(val(self, &locals, *v)?),
                    Op::Cast { op, value, .. } => {
                        let v = val(self, &locals, *value)?;
                        let from = match *value {
                            Value::Local(l) => f.locals[l.index()].ty,
                            Value::Const(c) => self.m.consts.get(c).ty,
                            Value::Metadata(_) => return None,
                        };
                        let to = rty?;
                        Some(match (op, v) {
                            (CastOp::ZExt | CastOp::Trunc, V::Int(x)) => V::Int(mask(mask(x, self.bits(from)?), self.bits(to)?)),
                            (CastOp::SExt, V::Int(x)) => V::Int(mask(sext(x, self.bits(from)?) as u128, self.bits(to)?)),
                            (CastOp::BitCast | CastOp::AddrSpaceCast, v) => v,
                            (CastOp::PtrToInt | CastOp::PtrToAddr, V::Ptr(None, a)) => V::Int(mask(a as u128, self.bits(to)?)),
                            (CastOp::IntToPtr, V::Int(x)) => V::Ptr(None, x as i64),
                            _ => return None,
                        })
                    }
                    Op::Call(call) => {
                        let Value::Const(c) = call.callee else { return None };
                        let ConstKind::Global(g) = self.m.consts.get(c).kind else { return None };
                        let name = self.m.global(g).name.clone().unwrap_or_default();
                        let args: Vec<V> = call.args.iter().map(|a| val(self, &locals, a.value)).collect::<Option<_>>()?;
                        if name.starts_with("llvm.memcpy") || name.starts_with("llvm.memmove") {
                            let V::Int(n) = *args.get(2)? else { return None };
                            self.copy(args[0], args[1], n as u64)?;
                            None
                        } else if name.starts_with("llvm.memset") {
                            let (V::Int(b), V::Int(n)) = (*args.get(1)?, *args.get(2)?) else { return None };
                            self.fill(args[0], b as u8, n as u64)?;
                            None
                        } else if name.starts_with("llvm.threadlocal.address") {
                            Some(args[0])
                        } else if name.starts_with("llvm.lifetime") || name.starts_with("llvm.assume") || name.starts_with("llvm.dbg") {
                            None
                        } else {
                            self.call(g, args)?
                        }
                    }
                    _ => return None,
                };
                if let (Some(r), Some(v)) = (inst.result, out) {
                    locals[r.index()] = Some(v);
                }
                if next.is_some() {
                    break;
                }
            }
            let n = next?;
            prev = block;
            block = n;
        }
    }
}
