//! The layout of types in memory: size, alignment, where each field is,
//! and how an enum says which variant it holds.
//!
//! Layouts are for concrete types (no generic parameters; projections are
//! normalised first), as monomorphisation makes them. They are worked out
//! once per type and shared, from any thread.
//!
//! - **Scalars**: `bool` (a byte, 0 or 1), `char` (four bytes, at most
//!   0x10FFFF), integers, floats, and pointers. A reference or function
//!   pointer is never null, and a `#[rustc_layout_scalar_valid_range_*]`
//!   struct (`NonNull`, the `NonZero` integers) narrows its field's range.
//!   The values a scalar can never hold are a *niche*.
//! - **Pointers** to unsized types are a pair: the address, and the
//!   length (slices, `str`) or the vtable (`dyn Trait`).
//! - **Structs and tuples** (`repr(Rust)`) have their fields put in order
//!   of decreasing alignment, so there is less padding; `repr(C)` keeps
//!   them in order. `packed` and `align` are followed, and an unsized last
//!   field stays last.
//! - **Enums** with one variant and no `repr` are that variant. Otherwise
//!   each variant's fields follow a tag, the smallest integer that holds
//!   the discriminants (or the `repr`'s). Without a `repr`, when all
//!   variants but one have no data, the others' tags are put in a niche of
//!   that one's fields if it has room, and the enum is no bigger than that
//!   variant: `Option<&T>` is a pointer, `None` is null.
//! - **Unions** are as big as their biggest field.
//!
//! This is our own layout, not rustc's, though it gives the same size and
//! alignment for the types the tests compare: Epitome's code only has to
//! agree with itself (and with C for `repr(C)`).

use crate::ast::{Kind, NodeId};
use crate::consteval::{ConstEnv, Value, int_bits, wrap};
use crate::hash::FastMap;
use crate::resolve::{DefId, DefKind, Resolver};
use crate::session::ParsedCrate;
use crate::ty::{AdtKind, FloatTy, Ty, TyCtxt, TyKind};
use crate::typeck::{Index, Solver};
use std::sync::{Arc, Mutex};

/// What layout needs of the target.
#[derive(Clone, Copy, Debug)]
pub struct TargetData {
    /// The size (and alignment) of a pointer, in bytes.
    pub ptr_size: u64,
}

impl Default for TargetData {
    fn default() -> Self {
        TargetData { ptr_size: 8 }
    }
}

/// A scalar's kind.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Prim {
    /// An integer of `size` bytes.
    Int { size: u8, signed: bool },
    Float { size: u8 },
    Pointer,
}

/// A scalar, with the values it may hold: `start..=end`, which wraps
/// around if `start > end`.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Scalar {
    pub prim: Prim,
    pub start: u128,
    pub end: u128,
}

impl Scalar {
    pub fn size(&self, ptr: u64) -> u64 {
        match self.prim {
            Prim::Int { size, .. } | Prim::Float { size } => size as u64,
            Prim::Pointer => ptr,
        }
    }

    fn mask(&self, ptr: u64) -> u128 {
        let bits = self.size(ptr) * 8;
        if bits >= 128 { u128::MAX } else { (1u128 << bits) - 1 }
    }

    /// Any value of its size.
    pub fn full(prim: Prim, ptr: u64) -> Scalar {
        let s = Scalar { prim, start: 0, end: 0 };
        Scalar { end: s.mask(ptr), ..s }
    }

    /// How many values it cannot hold (that a niche can use).
    pub fn available(&self, ptr: u64) -> u128 {
        if matches!(self.prim, Prim::Float { .. }) {
            return 0;
        }
        self.start.wrapping_sub(self.end).wrapping_sub(1) & self.mask(ptr)
    }
}

/// How a value is passed and held, as far as lowering cares.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Abi {
    /// No value of the type exists (`!`, an enum without variants).
    Uninhabited,
    /// One scalar (perhaps inside structs of one field).
    Scalar(Scalar),
    /// Two scalars: a pointer to something unsized, a pair of integers.
    ScalarPair(Scalar, Scalar),
    /// Anything else: kept in memory. `sized` is false for `[T]`, `str`,
    /// `dyn Trait` and structs ending in one.
    Aggregate { sized: bool },
}

/// Where a type's fields are.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Fields {
    /// No fields (a scalar).
    Primitive,
    /// Each field at its offset; `order` lists the fields in memory order.
    Arbitrary { offsets: Vec<u64>, order: Vec<u32> },
    /// An array or slice: `count` elements `stride` apart (`count` is 0 for
    /// a slice).
    Array { stride: u64, count: u64 },
    /// A union's fields, all at 0.
    Union(u32),
}

/// A niche: a scalar at `offset` whose unused values can stand for
/// something else.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Niche {
    pub offset: u64,
    pub scalar: Scalar,
}

/// How an enum's variant is told.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum TagEncoding {
    /// The tag holds the discriminant.
    Direct,
    /// Variant `untagged` is held by the tag's normal values; variants
    /// `first..=last` (but `untagged`) by `start + (v - first)`.
    Niche { untagged: u32, first: u32, last: u32, start: u128 },
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Variants {
    /// A struct, union, tuple, or an enum laid out as one variant.
    Single { index: u32 },
    /// An enum: the tag (at `tag_offset`), and each variant's layout (its
    /// fields' offsets are from the start of the enum).
    Multiple { tag: Scalar, tag_offset: u64, encoding: TagEncoding, discriminants: Vec<i128>, variants: Vec<Layout> },
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Layout {
    pub size: u64,
    pub align: u64,
    pub abi: Abi,
    pub fields: Fields,
    pub variants: Variants,
    /// The niche with the most room, if any.
    pub niche: Option<Niche>,
}

impl Layout {
    pub fn is_zst(&self) -> bool {
        self.size == 0 && self.is_sized()
    }

    pub fn is_sized(&self) -> bool {
        !matches!(self.abi, Abi::Aggregate { sized: false })
    }

    pub fn is_uninhabited(&self) -> bool {
        self.abi == Abi::Uninhabited
    }

    fn scalar(s: Scalar, ptr: u64) -> Layout {
        let size = s.size(ptr);
        let niche = (s.available(ptr) > 0).then_some(Niche { offset: 0, scalar: s });
        Layout { size, align: size.max(1), abi: Abi::Scalar(s), fields: Fields::Primitive, variants: Variants::Single { index: 0 }, niche }
    }

    /// The offset of field `i` (for an array, element `i`).
    pub fn offset(&self, i: usize) -> u64 {
        match &self.fields {
            Fields::Arbitrary { offsets, .. } => offsets[i],
            Fields::Array { stride, .. } => stride * i as u64,
            _ => 0,
        }
    }
}

/// Why a type has no layout.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum LayoutError {
    /// It still has generic parameters, projections or variables: the
    /// type, printed.
    NotConcrete(String),
    /// An array length or discriminant that could not be evaluated.
    BadConst,
    /// Something not yet supported (closures, before captures are known).
    Unsupported(&'static str),
    /// A type containing itself.
    Recursive,
}

/// `repr` attributes.
#[derive(Clone, Copy, Default, Debug)]
struct Repr {
    c: bool,
    transparent: bool,
    /// `packed(n)`.
    pack: Option<u64>,
    /// `align(n)`.
    align: Option<u64>,
    /// `u8`, `i32`, ... for an enum's tag.
    int: Option<TyKind>,
    /// `#[rustc_layout_scalar_valid_range_start/end]`.
    valid_start: Option<u128>,
    valid_end: Option<u128>,
}

/// A variant's fields' layouts.
type FieldLayouts<'f> = dyn Fn(&crate::ty::VariantDef) -> Result<Vec<Arc<Layout>>, LayoutError> + 'f;

/// Works out and keeps layouts (and constants' values).
pub struct LayoutCx<'a, 's> {
    pub tcx: &'a TyCtxt<'s>,
    pub r: &'a Resolver<'s>,
    pub crates: &'a [ParsedCrate<'s>],
    pub index: &'a Index<'s>,
    pub target: TargetData,
    /// Type checking's results, for closures' captures.
    pub typeck: Option<&'a crate::typeck::TypeckResults>,
    cache: Mutex<FastMap<Ty, Result<Arc<Layout>, LayoutError>>>,
    pub(crate) const_cache: Mutex<FastMap<DefId, Option<Value>>>,
    reprs: Mutex<FastMap<DefId, Repr>>,
}

fn round_up(x: u64, a: u64) -> u64 {
    x.div_ceil(a.max(1)) * a.max(1)
}

impl<'a, 's> LayoutCx<'a, 's> {
    pub fn new(tcx: &'a TyCtxt<'s>, r: &'a Resolver<'s>, crates: &'a [ParsedCrate<'s>], index: &'a Index<'s>, target: TargetData) -> Self {
        LayoutCx {
            tcx,
            r,
            crates,
            index,
            target,
            typeck: None,
            cache: Mutex::new(FastMap::default()),
            const_cache: Mutex::new(FastMap::default()),
            reprs: Mutex::new(FastMap::default()),
        }
    }

    /// The types of what closure `c` (with its owner's arguments `l`)
    /// captures: a reference to each local captured by reference, the
    /// local's type for one captured by value.
    pub fn upvar_tys(&self, c: crate::ty::ClosureId, l: crate::ty::List) -> Option<Vec<Ty>> {
        let typeck = self.typeck?;
        let tcx = self.tcx;
        let caps = typeck.captures.get(&c)?;
        let owner = *typeck.closure_owners.get(&c)?;
        let (k, f, _) = tcx.closure_at(c);
        let params = tcx.all_params(owner);
        let args = tcx.tys(l);
        let mut s = Solver::new(tcx, self.r, self.crates, self.index);
        caps.iter()
            .map(|cap| {
                let t = (*typeck.types.get(k as usize)?.get(f as usize)?.get(cap.local.0 as usize)?)?;
                let t = crate::mono::sub(tcx, &typeck.hidden, &mut s, t, &params, &args);
                Some(if cap.by_value { t } else { tcx.mk(TyKind::Ref(crate::ty::Mutbl::Mut, t)) })
            })
            .collect()
    }

    pub fn ptr_bits(&self) -> u32 {
        (self.target.ptr_size * 8) as u32
    }

    fn ptr(&self) -> u64 {
        self.target.ptr_size
    }

    /// The type written at node `n` (in crate `k`, file `f`, inside item
    /// `owner`), if it names no generic parameter.
    pub fn lower_written(&self, k: u32, f: u32, owner: DefId, n: NodeId) -> Option<Ty> {
        let env = crate::ty::lower::Env { r: self.r, crates: self.crates };
        let mut l = crate::ty::lower::Lower {
            tcx: self.tcx,
            env: &env,
            expanding: Default::default(),
            impl_self: None,
            holes: false,
            k,
            file: f,
            owner,
            failed: false,
        };
        let t = l.ty(n);
        (!l.failed && !self.tcx.has_error(t)).then_some(t)
    }

    /// The layout of `t`.
    pub fn layout_of(&self, t: Ty) -> Result<Arc<Layout>, LayoutError> {
        if let Some(l) = self.cache.lock().unwrap().get(&t) {
            return l.clone();
        }
        // A type being worked out that needs itself (only through a
        // pointer is fine, and pointers do not look inside). Kept for
        // this thread only: another thread may be working out the same
        // type, and must not read that it is recursive.
        if !COMPUTING.with(|c| c.borrow_mut().insert(t)) {
            return Err(LayoutError::Recursive);
        }
        let l = {
            let _done = Computing(t);
            self.compute(t).map(Arc::new)
        };
        self.cache.lock().unwrap().insert(t, l.clone());
        l
    }

    fn not_concrete(&self, t: Ty) -> LayoutError {
        LayoutError::NotConcrete(self.tcx.display(self.r, t))
    }

    fn compute(&self, t: Ty) -> Result<Layout, LayoutError> {
        let tcx = self.tcx;
        let ptr = self.ptr();
        let int = |size: u8, signed: bool| Layout::scalar(Scalar::full(Prim::Int { size, signed }, ptr), ptr);
        Ok(match tcx.kind(t) {
            TyKind::Bool => Layout::scalar(Scalar { prim: Prim::Int { size: 1, signed: false }, start: 0, end: 1 }, ptr),
            TyKind::Char => Layout::scalar(Scalar { prim: Prim::Int { size: 4, signed: false }, start: 0, end: 0x10FFFF }, ptr),
            k @ (TyKind::Int(_) | TyKind::Uint(_)) => {
                let (bits, signed) = int_bits(k, self.ptr_bits()).unwrap();
                let mut l = int((bits / 8) as u8, signed);
                // `u128` and `i128` are 16-aligned, as on x86-64 and AArch64.
                l.align = l.size.min(16);
                l
            }
            TyKind::Float(f) => {
                let size = match f {
                    FloatTy::F16 => 2,
                    FloatTy::F32 => 4,
                    FloatTy::F64 => 8,
                    FloatTy::F128 => 16,
                };
                Layout::scalar(Scalar::full(Prim::Float { size }, ptr), ptr)
            }
            TyKind::Never => Layout {
                size: 0,
                align: 1,
                abi: Abi::Uninhabited,
                fields: Fields::Primitive,
                variants: Variants::Single { index: 0 },
                niche: None,
            },
            TyKind::Ref(_, x) | TyKind::Ptr(_, x) => {
                let nonnull = matches!(tcx.kind(t), TyKind::Ref(..));
                let addr = Scalar { prim: Prim::Pointer, start: nonnull as u128, end: Scalar::full(Prim::Pointer, ptr).end };
                match self.metadata(x)? {
                    None => Layout::scalar(addr, ptr),
                    Some(meta) => self.pair(addr, meta),
                }
            }
            TyKind::FnPtr(_) => Layout::scalar(Scalar { prim: Prim::Pointer, start: 1, end: Scalar::full(Prim::Pointer, ptr).end }, ptr),
            TyKind::FnDef(..) => self.univariant(&[], Repr::default(), 0, false)?,
            TyKind::Array(e, c) => {
                let el = self.layout_of(e)?;
                let count = self.eval_const(c, &ConstEnv::default()).ok_or(LayoutError::BadConst)? as u64;
                let stride = el.size;
                let niche = if count > 0 { el.niche } else { None };
                let abi = if el.is_uninhabited() && count > 0 { Abi::Uninhabited } else { Abi::Aggregate { sized: true } };
                Layout { size: stride * count, align: el.align, abi, fields: Fields::Array { stride, count }, variants: Variants::Single { index: 0 }, niche }
            }
            TyKind::Slice(e) => {
                let el = self.layout_of(e)?;
                Layout {
                    size: 0,
                    align: el.align,
                    abi: Abi::Aggregate { sized: false },
                    fields: Fields::Array { stride: el.size, count: 0 },
                    variants: Variants::Single { index: 0 },
                    niche: None,
                }
            }
            TyKind::Str => Layout {
                size: 0,
                align: 1,
                abi: Abi::Aggregate { sized: false },
                fields: Fields::Array { stride: 1, count: 0 },
                variants: Variants::Single { index: 0 },
                niche: None,
            },
            TyKind::Dyn(_) | TyKind::Foreign(_) => Layout {
                size: 0,
                align: 1,
                abi: Abi::Aggregate { sized: false },
                fields: Fields::Union(0),
                variants: Variants::Single { index: 0 },
                niche: None,
            },
            TyKind::Tuple(l) => {
                let fields = tcx.tys(l).into_iter().map(|f| self.layout_of(f)).collect::<Result<Vec<_>, _>>()?;
                self.univariant(&fields, Repr::default(), 0, false)?
            }
            TyKind::Adt(d, l) => self.adt(d, &tcx.tys(l))?,
            // A pattern type: its type's scalar, holding fewer values.
            TyKind::Pat(x, a, b) => {
                let mut l = (*self.layout_of(x)?).clone();
                if let (Abi::Scalar(s), crate::ty::Const::Value(a), crate::ty::Const::Value(b)) = (l.abi, tcx.const_of(a), tcx.const_of(b)) {
                    let mask = Scalar::full(s.prim, ptr).end;
                    let s = Scalar { start: a & mask, end: b & mask, ..s };
                    l.abi = Abi::Scalar(s);
                    l.niche = (s.available(ptr) > 0).then_some(Niche { offset: 0, scalar: s });
                }
                l
            }
            TyKind::Projection(..) => {
                let mut s = Solver::new(tcx, self.r, self.crates, self.index);
                let n = s.normalize(t);
                if n == t {
                    return Err(self.not_concrete(t));
                }
                return self.layout_of(n).map(|l| (*l).clone());
            }
            // A closure is a tuple of what it captures.
            TyKind::Closure(c, _, l) => match self.upvar_tys(c, l) {
                Some(tys) => (*self.layout_of(tcx.mk(TyKind::Tuple(tcx.list(tys))))?).clone(),
                None => return Err(LayoutError::Unsupported("closure (captures not known)")),
            },
            _ => return Err(self.not_concrete(t)),
        })
    }

    /// What a pointer to `t` carries besides the address: nothing for a
    /// sized type, the length for a slice or `str` (or a struct ending in
    /// one), the vtable for `dyn Trait`.
    fn metadata(&self, t: Ty) -> Result<Option<Scalar>, LayoutError> {
        let tcx = self.tcx;
        let ptr = self.ptr();
        let usize_ = Scalar::full(Prim::Int { size: ptr as u8, signed: false }, ptr);
        Ok(match tcx.kind(t) {
            TyKind::Slice(_) | TyKind::Str => Some(usize_),
            TyKind::Dyn(_) => Some(Scalar { prim: Prim::Pointer, start: 1, end: Scalar::full(Prim::Pointer, ptr).end }),
            TyKind::Adt(d, l) => {
                let Some(adt) = tcx.adts.get(&d) else { return Ok(None) };
                if adt.kind != AdtKind::Struct {
                    return Ok(None);
                }
                match adt.variants[0].fields.last() {
                    Some(f) => {
                        let ft = crate::ty::collect::subst(tcx, f.ty, &tcx.all_params(d), &tcx.tys(l));
                        self.metadata(ft)?
                    }
                    None => None,
                }
            }
            TyKind::Tuple(l) => match tcx.tys(l).last() {
                Some(&x) => self.metadata(x)?,
                None => None,
            },
            TyKind::Projection(..) => {
                let mut s = Solver::new(tcx, self.r, self.crates, self.index);
                let n = s.normalize(t);
                if n == t {
                    return Err(self.not_concrete(t));
                }
                self.metadata(n)?
            }
            TyKind::Param(_) | TyKind::Infer(_) | TyKind::Error => return Err(self.not_concrete(t)),
            _ => None,
        })
    }

    /// A pair of scalars, the second after the first, aligned.
    fn pair(&self, a: Scalar, b: Scalar) -> Layout {
        let ptr = self.ptr();
        let (sa, sb) = (a.size(ptr), b.size(ptr));
        let off_b = round_up(sa, sb.max(1));
        let align = sa.max(sb).max(1);
        let size = round_up(off_b + sb, align);
        let niche = [(0, a), (off_b, b)]
            .into_iter()
            .filter(|(_, s)| s.available(ptr) > 0)
            .max_by_key(|(_, s)| s.available(ptr))
            .map(|(offset, scalar)| Niche { offset, scalar });
        Layout {
            size,
            align,
            abi: Abi::ScalarPair(a, b),
            fields: Fields::Arbitrary { offsets: vec![0, off_b], order: vec![0, 1] },
            variants: Variants::Single { index: 0 },
            niche,
        }
    }

    /// Fields laid out one after another from `prefix` bytes in (an enum
    /// variant's after its tag), reordered unless `repr(C)` or `in_order`.
    fn univariant(&self, fields: &[Arc<Layout>], repr: Repr, prefix: u64, in_order: bool) -> Result<Layout, LayoutError> {
        let ptr = self.ptr();
        let n = fields.len();
        let mut order: Vec<u32> = (0..n as u32).collect();
        let last_unsized = fields.last().is_some_and(|f| !f.is_sized());
        if !repr.c && !in_order && repr.pack.is_none() {
            // Most aligned first; an unsized last field stays last.
            let m = if last_unsized { n - 1 } else { n };
            order[..m].sort_by_key(|&i| std::cmp::Reverse(fields[i as usize].align));
        }
        let field_align = |f: &Layout| match repr.pack {
            Some(p) => f.align.min(p),
            None => f.align,
        };
        let mut offsets = vec![0u64; n];
        let mut at = prefix;
        let mut align = 1u64;
        for &i in &order {
            let f = &fields[i as usize];
            let a = field_align(f);
            at = round_up(at, a);
            offsets[i as usize] = at;
            at += f.size;
            align = align.max(a);
        }
        if let Some(a) = repr.align {
            align = align.max(a);
        }
        let size = round_up(at, align);
        let sized = !last_unsized;
        let uninhabited = fields.iter().any(|f| f.is_uninhabited());
        // The niche with the most room.
        let mut niche = fields
            .iter()
            .enumerate()
            .filter_map(|(i, f)| f.niche.map(|nc| Niche { offset: offsets[i] + nc.offset, ..nc }))
            .max_by_key(|nc| nc.scalar.available(ptr));
        // A newtype (one field that is not zero-sized, at 0, filling it) is
        // passed as that field.
        let big: Vec<usize> = (0..n).filter(|&i| !fields[i].is_zst()).collect();
        let mut abi = if uninhabited {
            Abi::Uninhabited
        } else if !sized {
            Abi::Aggregate { sized: false }
        } else if big.len() == 1 && offsets[big[0]] == 0 && fields[big[0]].size == size && fields[big[0]].align == align {
            match fields[big[0]].abi {
                a @ (Abi::Scalar(_) | Abi::ScalarPair(..)) => a,
                _ => Abi::Aggregate { sized: true },
            }
        } else {
            Abi::Aggregate { sized: true }
        };
        // `#[rustc_layout_scalar_valid_range_start(1)]`: a narrower scalar.
        if let (Abi::Scalar(mut s), true) = (abi, repr.valid_start.is_some() || repr.valid_end.is_some()) {
            if let Some(v) = repr.valid_start {
                s.start = v;
            }
            if let Some(v) = repr.valid_end {
                s.end = v;
            }
            abi = Abi::Scalar(s);
            niche = (s.available(ptr) > 0).then_some(Niche { offset: 0, scalar: s });
        }
        Ok(Layout { size, align, abi, fields: Fields::Arbitrary { offsets, order }, variants: Variants::Single { index: 0 }, niche })
    }

    /// `repr` and layout attributes of an ADT.
    fn repr(&self, d: DefId) -> Repr {
        if let Some(r) = self.reprs.lock().unwrap().get(&d) {
            return *r;
        }
        let def = &self.r.defs[d.0 as usize];
        let ast = &self.crates[def.krate as usize].files[def.file as usize].ast;
        let mut repr = Repr::default();
        for (name, a) in crate::session::attributes(ast, def.node) {
            let inner = ast.child(a, Kind::TokenTree).map(|tt| ast.text(tt)).unwrap_or_default();
            let inner = inner.trim().trim_start_matches('(').trim_end_matches(')').to_string();
            match name.as_str() {
                "repr" => {
                    for part in split_top(&inner) {
                        let part = part.trim();
                        let (word, arg) = match part.find('(') {
                            Some(i) => (&part[..i], part[i + 1..].trim_end_matches(')').trim().parse::<u64>().ok()),
                            None => (part, None),
                        };
                        match word.trim() {
                            "C" => repr.c = true,
                            "transparent" => repr.transparent = true,
                            "packed" => repr.pack = Some(arg.unwrap_or(1)),
                            "align" => repr.align = arg,
                            w => repr.int = repr.int.or(crate::consteval::prim_named(w)),
                        }
                    }
                }
                "rustc_layout_scalar_valid_range_start" => repr.valid_start = parse_int(&inner),
                "rustc_layout_scalar_valid_range_end" => repr.valid_end = parse_int(&inner),
                _ => {}
            }
        }
        self.reprs.lock().unwrap().insert(d, repr);
        repr
    }

    fn adt(&self, d: DefId, args: &[Ty]) -> Result<Layout, LayoutError> {
        let tcx = self.tcx;
        let adt = tcx.adts.get(&d).ok_or(LayoutError::Unsupported("not an ADT"))?;
        let params = tcx.all_params(d);
        let repr = self.repr(d);
        let field_layouts = |v: &crate::ty::VariantDef| -> Result<Vec<Arc<Layout>>, LayoutError> {
            v.fields.iter().map(|f| self.layout_of(crate::ty::collect::subst(tcx, f.ty, &params, args))).collect()
        };
        match adt.kind {
            AdtKind::Struct => {
                let fields = field_layouts(&adt.variants[0])?;
                self.univariant(&fields, repr, 0, false)
            }
            AdtKind::Union => {
                let fields = field_layouts(&adt.variants[0])?;
                let align = fields.iter().map(|f| f.align).max().unwrap_or(1).max(repr.align.unwrap_or(1));
                let align = match repr.pack {
                    Some(p) => align.min(p),
                    None => align,
                };
                let size = round_up(fields.iter().map(|f| f.size).max().unwrap_or(0), align);
                Ok(Layout {
                    size,
                    align,
                    abi: Abi::Aggregate { sized: true },
                    fields: Fields::Union(fields.len() as u32),
                    variants: Variants::Single { index: 0 },
                    niche: None,
                })
            }
            AdtKind::Enum => self.enum_layout(d, adt, repr, &field_layouts),
        }
    }

    /// Each variant's discriminant: written, or one more than the one
    /// before.
    pub fn discriminants(&self, d: DefId) -> Result<Vec<i128>, LayoutError> {
        let adt = self.tcx.adts.get(&d).ok_or(LayoutError::Unsupported("not an ADT"))?;
        let mut out = Vec::with_capacity(adt.variants.len());
        let mut next = 0i128;
        for v in &adt.variants {
            let mut val = next;
            if let Some(vd) = v.def {
                let def = &self.r.defs[vd.0 as usize];
                let ast = &self.crates[def.krate as usize].files[def.file as usize].ast;
                let written = ast
                    .children(def.node)
                    .iter()
                    .copied()
                    .find(|&c| !matches!(ast.kind(c), Kind::Name | Kind::Attribute | Kind::DocComment | Kind::Visibility | Kind::TupleFields | Kind::NamedFields));
                if let Some(e) = written {
                    val = self.eval(def.krate, def.file, e, &ConstEnv::default()).and_then(|v| v.int()).ok_or(LayoutError::BadConst)?;
                }
            }
            out.push(val);
            next = val.wrapping_add(1);
        }
        Ok(out)
    }

    fn enum_layout(
        &self,
        d: DefId,
        adt: &crate::ty::AdtDef,
        repr: Repr,
        field_layouts: &FieldLayouts,
    ) -> Result<Layout, LayoutError> {
        let ptr = self.ptr();
        if adt.variants.is_empty() {
            return Ok(Layout {
                size: 0,
                align: 1,
                abi: Abi::Uninhabited,
                fields: Fields::Arbitrary { offsets: vec![], order: vec![] },
                variants: Variants::Single { index: 0 },
                niche: None,
            });
        }
        let discrs = self.discriminants(d)?;
        let variant_fields: Vec<Vec<Arc<Layout>>> = adt.variants.iter().map(field_layouts).collect::<Result<_, _>>()?;
        let explicit = repr.int.is_some() || repr.c;
        // One variant, no `repr`: just that variant.
        if adt.variants.len() == 1 && !explicit {
            return self.univariant(&variant_fields[0], repr, 0, false);
        }
        // The tag: the `repr`'s, `C`'s `int`, or the smallest that fits.
        let (min, max) = (discrs.iter().copied().min().unwrap(), discrs.iter().copied().max().unwrap());
        let (tag_size, signed) = match repr.int {
            Some(t) => {
                let (bits, s) = int_bits(t, self.ptr_bits()).unwrap();
                ((bits / 8) as u8, s)
            }
            None if repr.c => (4, true),
            None => {
                let signed = min < 0;
                let fits = |bits: u32| {
                    if signed {
                        min >= -(1i128 << (bits - 1)) && max < (1i128 << (bits - 1))
                    } else {
                        max < (1i128 << bits)
                    }
                };
                let bits = [8u32, 16, 32, 64].into_iter().find(|&b| fits(b)).unwrap_or(128);
                ((bits / 8) as u8, signed)
            }
        };
        let tag_bits = tag_size as u32 * 8;
        let as_tag = |v: i128| wrap(v, tag_bits, false) as u128;
        let tag = Scalar { prim: Prim::Int { size: tag_size, signed }, start: as_tag(min), end: as_tag(max) };
        let tag_align = tag_size as u64;
        // Each variant after the tag.
        let mut variants = Vec::new();
        for f in &variant_fields {
            let mut v = self.univariant(f, repr, tag_align, false)?;
            v.variants = Variants::Single { index: variants.len() as u32 };
            variants.push(v);
        }
        let align = variants.iter().map(|v| v.align).max().unwrap_or(1).max(tag_align).max(repr.align.unwrap_or(1));
        let size = round_up(variants.iter().map(|v| v.size).max().unwrap_or(0).max(tag_align), align);
        let all_unit = variant_fields.iter().all(|f| f.iter().all(|l| l.is_zst()));
        let uninhabited = variants.iter().all(|v| v.is_uninhabited());
        let tagged = Layout {
            size,
            align,
            abi: if uninhabited {
                Abi::Uninhabited
            } else if all_unit {
                Abi::Scalar(tag)
            } else {
                Abi::Aggregate { sized: true }
            },
            fields: Fields::Arbitrary { offsets: vec![0], order: vec![0] },
            variants: Variants::Multiple { tag, tag_offset: 0, encoding: TagEncoding::Direct, discriminants: discrs.clone(), variants },
            niche: (tag.available(ptr) > 0).then_some(Niche { offset: 0, scalar: tag }),
        };
        if explicit {
            return Ok(tagged);
        }
        // A niche: all variants but one have no data.
        let dataful: Vec<usize> = (0..variant_fields.len()).filter(|&i| variant_fields[i].iter().any(|l| !l.is_zst())).collect();
        if dataful.len() != 1 {
            return Ok(tagged);
        }
        let untagged = dataful[0];
        let others: Vec<usize> = (0..variant_fields.len()).filter(|&i| i != untagged).collect();
        let (first, last) = (*others.first().unwrap() as u32, *others.last().unwrap() as u32);
        let count = (last - first + 1) as u128;
        let inner = self.univariant(&variant_fields[untagged], repr, 0, false)?;
        let Some(niche) = inner.niche else { return Ok(tagged) };
        if niche.scalar.available(ptr) < count || inner.size > tagged.size {
            return Ok(tagged);
        }
        let mask = Scalar::full(niche.scalar.prim, ptr).end;
        let start = niche.scalar.end.wrapping_add(1) & mask;
        // The niche's scalar now holds the others' values too.
        let widened = Scalar { end: start.wrapping_add(count - 1) & mask, ..niche.scalar };
        let mut variants = Vec::new();
        for (i, f) in variant_fields.iter().enumerate() {
            let mut v = if i == untagged { inner.clone() } else { self.univariant(f, repr, 0, false)? };
            v.variants = Variants::Single { index: i as u32 };
            variants.push(v);
        }
        let abi = match inner.abi {
            Abi::Scalar(_) if inner.size == widened.size(ptr) => Abi::Scalar(widened),
            Abi::ScalarPair(a, b) => {
                if niche.offset == 0 { Abi::ScalarPair(widened, b) } else { Abi::ScalarPair(a, widened) }
            }
            _ => Abi::Aggregate { sized: true },
        };
        Ok(Layout {
            size: inner.size,
            align: inner.align,
            abi,
            fields: Fields::Arbitrary { offsets: vec![niche.offset], order: vec![0] },
            variants: Variants::Multiple {
                tag: widened,
                tag_offset: niche.offset,
                encoding: TagEncoding::Niche { untagged: untagged as u32, first, last, start },
                discriminants: discrs,
                variants,
            },
            niche: (widened.available(ptr) > 0).then_some(Niche { offset: niche.offset, scalar: widened }),
        })
    }

    /// A layout in a line: size, alignment, fields' offsets, and an enum's
    /// tag.
    pub fn describe(&self, d_names: Option<&[&str]>, l: &Layout) -> String {
        use std::fmt::Write;
        let mut s = format!("size {}, align {}", l.size, l.align);
        if !l.is_sized() {
            s.push_str(", unsized");
        }
        if l.is_uninhabited() {
            s.push_str(", uninhabited");
        }
        match &l.variants {
            Variants::Single { .. } => {
                if let Fields::Arbitrary { offsets, .. } = &l.fields
                    && !offsets.is_empty()
                {
                    let names: Vec<String> = offsets
                        .iter()
                        .enumerate()
                        .map(|(i, o)| match d_names.and_then(|n| n.get(i)) {
                            Some(n) => format!("{n}@{o}"),
                            None => format!("{i}@{o}"),
                        })
                        .collect();
                    let _ = write!(s, ", fields {}", names.join(" "));
                }
            }
            Variants::Multiple { tag, tag_offset, encoding, discriminants, .. } => {
                let ty = match tag.prim {
                    Prim::Int { size, signed } => format!("{}{}", if signed { "i" } else { "u" }, size as u32 * 8),
                    Prim::Pointer => "ptr".into(),
                    Prim::Float { .. } => "float".into(),
                };
                match encoding {
                    TagEncoding::Direct => {
                        let _ = write!(s, ", tag {ty}@{tag_offset} = {discriminants:?}");
                    }
                    TagEncoding::Niche { untagged, first, last, start } => {
                        let _ = write!(s, ", niche {ty}@{tag_offset}: variants {first}..={last} (not {untagged}) from {start}");
                    }
                }
            }
        }
        if let Some(n) = l.niche {
            let _ = write!(s, ", niche room {}", n.scalar.available(self.ptr()).min(u64::MAX as u128));
        }
        s
    }

    /// Layouts of every non-generic struct, enum, union and type alias of
    /// crate `only` (or of all crates), one per line, for `--dump-layout`.
    pub fn dump(&self, only: Option<u32>) -> String {
        use std::fmt::Write;
        let tcx = self.tcx;
        let mut out = String::new();
        for (i, def) in self.r.defs.iter().enumerate() {
            let d = DefId(i as u32);
            if only.is_some_and(|k| k != def.krate) || !tcx.all_params(d).is_empty() {
                continue;
            }
            let (word, t) = match def.kind {
                DefKind::Struct | DefKind::Enum | DefKind::Union if tcx.adts.contains_key(&d) => {
                    (if def.kind == DefKind::Enum { "enum" } else { "struct" }, tcx.mk(TyKind::Adt(d, tcx.list(vec![]))))
                }
                DefKind::TypeAlias => match tcx.type_of(d) {
                    Some(t) => ("type", t),
                    None => continue,
                },
                _ => continue,
            };
            let names: Option<Vec<&str>> = tcx.adts.get(&d).filter(|a| a.kind != AdtKind::Enum).map(|a| a.variants[0].fields.iter().map(|f| f.name).collect());
            let desc = match self.layout_of(t) {
                Ok(l) => self.describe(names.as_deref(), &l),
                Err(e) => format!("no layout: {e:?}"),
            };
            if word == "type" {
                let _ = writeln!(out, "type {} = {}: {desc}", self.r.def_path(d), tcx.display(self.r, t));
            } else {
                let _ = writeln!(out, "{word} {}: {desc}", self.r.def_path(d));
            }
        }
        out
    }
}

/// `a, b(c, d), e` split at the top-level commas.
fn split_top(s: &str) -> Vec<&str> {
    let (mut out, mut depth, mut start) = (Vec::new(), 0, 0);
    for (i, c) in s.char_indices() {
        match c {
            '(' => depth += 1,
            ')' => depth -= 1,
            ',' if depth == 0 => {
                out.push(&s[start..i]);
                start = i + 1;
            }
            _ => {}
        }
    }
    if !s[start..].trim().is_empty() {
        out.push(&s[start..]);
    }
    out
}

fn parse_int(s: &str) -> Option<u128> {
    let s = s.trim().replace('_', "");
    match s.strip_prefix("0x") {
        Some(h) => u128::from_str_radix(h, 16).ok(),
        None => s.parse().ok(),
    }
}

thread_local! {
    /// The types whose layouts this thread is working out.
    static COMPUTING: std::cell::RefCell<crate::hash::FastSet<Ty>> = std::cell::RefCell::new(Default::default());
}

/// Takes a type out of `COMPUTING` when its layout is worked out, by
/// return or by panic.
struct Computing(Ty);

impl Drop for Computing {
    fn drop(&mut self) {
        COMPUTING.with(|c| c.borrow_mut().remove(&self.0));
    }
}
