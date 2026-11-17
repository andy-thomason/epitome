//! The C functions of `svdpi.h` (Annex H), which `export_dpi!` exports
//! from the executable under their names, for DPI libraries to call.
//!
//! Scopes (`svScope`) are pointers to `ScopeRec`s that the simulation
//! keeps, and open array handles (`svOpenArrayHandle`) pointers to
//! `OpenArray`s that live for the call they are passed to. Neither needs
//! the simulation, so most functions work on them alone; the ones that
//! name scopes or read the time find the simulation of the thread.
//!
//! The functions that take a variable number of indices are defined with
//! fixed parameters, which C cannot express but which receive the indices
//! where the calling convention puts them: after the fixed arguments in
//! the argument sequence, except on Apple's arm64, where variadic
//! arguments all go on the stack, after the eight register slots.

#![allow(non_snake_case, clippy::missing_safety_doc)]

use super::{CKind, CHAIN, Chain, c_kind, c_size, current_sim, get, put, words};
use crate::design::types::{Type, TypeId, Types, UDim};
use crate::design::value::Bits;
use crate::design::{Const, ScopeId};
use crate::hash::FxMap;
use std::cell::RefCell;
use std::ffi::{CStr, CString, c_char, c_void};
use std::ptr::null_mut;

/// An instantiated scope, as an `svScope` points to it.
pub struct ScopeRec {
    pub(crate) inst: u32,
    pub(crate) scope: ScopeId,
    name: CString,
    /// `svPutUserData`'s values, by key.
    user: RefCell<FxMap<usize, usize>>,
}

impl ScopeRec {
    pub(crate) fn new(inst: u32, scope: ScopeId, name: String) -> ScopeRec {
        ScopeRec { inst, scope, name: CString::new(name).unwrap_or_default(), user: RefCell::default() }
    }
}

/// `svLogicVecVal`: 32 bits, as value bits and unknown bits.
#[repr(C)]
#[derive(Clone, Copy, Default, Debug)]
pub struct LogicVecVal {
    pub aval: u32,
    pub bval: u32,
}

/// `svTimeVal`, the same as VPI's `s_vpi_time`.
#[repr(C)]
pub struct TimeVal {
    pub type_: i32,
    pub high: u32,
    pub low: u32,
    pub real: f64,
}

/// What an open array's elements are, for the access functions.
#[derive(Clone, Copy, Debug)]
enum Elem {
    /// `bit` or `logic`: one `svBit` or `svLogic` byte each.
    Scalar,
    /// A packed value, in canonical words.
    Vec(u32, bool),
    /// Anything else, in its C layout.
    Other,
}

/// An actual argument passed for an open array formal (H.12): its
/// elements in C order, the lowest index of each dimension first.
pub struct OpenArray {
    data: Vec<u64>,
    len: usize,
    elem_ty: TypeId,
    elem: Elem,
    elem_size: usize,
    /// Each unpacked dimension's range, `[left:right]`.
    dims: Vec<(i64, i64)>,
    /// Each unpacked dimension's number of elements.
    sizes: Vec<usize>,
    /// Whether the actual's own dimension runs downwards, so that its
    /// value is in the reverse of C order.
    desc: Vec<bool>,
    strides: Vec<usize>,
    signed: bool,
}

/// How elements of type `ty` are held; for an open packed formal (`bit []`
/// or `logic []`) the width is the actual's, and `formal` says which form.
fn elem_shape(tys: &Types, ty: TypeId, open_packed: Option<TypeId>) -> (Elem, usize) {
    if let Some(formal) = open_packed
        && tys.is_integral(ty)
    {
        let w = tys.width(ty).max(1);
        let four = tys.is_four_state(formal);
        return (Elem::Vec(w, four), words(w) * if four { 8 } else { 4 });
    }
    match c_kind(tys, ty) {
        CKind::Scalar => (Elem::Scalar, 1),
        CKind::Vec(w, four, _) => (Elem::Vec(w, four), words(w) * if four { 8 } else { 4 }),
        _ => (Elem::Other, c_size(tys, ty).0),
    }
}

impl OpenArray {
    /// The handle for an actual of type `actual` and value `v`, passed for
    /// a formal of type `formal`; with its value in it when `fill`.
    pub(super) fn new(tys: &Types, formal: TypeId, actual: TypeId, v: &Const, open_packed: bool, fill: bool, strings: &mut Vec<CString>) -> OpenArray {
        let (mut ft, mut at) = (formal, actual);
        let mut probe = v.clone();
        let (mut dims, mut sizes, mut desc) = (Vec::new(), Vec::new(), Vec::new());
        while let Type::Unpacked { elem: fe, dim: fd } = tys.get(ft) {
            if matches!(fd, UDim::Assoc(_) | UDim::Wildcard) {
                break;
            }
            let ad = match tys.get(at) {
                Type::Unpacked { elem, dim } => Some((*elem, *dim)),
                _ => None,
            };
            let len = match &probe {
                Const::Aggregate(items) => items.len() as i64,
                _ => 0,
            };
            let (range, down) = match (fd, ad) {
                (_, Some((_, UDim::Range(l, r)))) => (if let UDim::Range(fl, fr) = fd { (*fl, *fr) } else { (l, r) }, l > r),
                (UDim::Range(l, r), _) => ((*l, *r), false),
                _ => ((0, len - 1), false),
            };
            dims.push(range);
            sizes.push(if matches!(fd, UDim::Range(..)) || matches!(ad, Some((_, UDim::Range(..)))) {
                (range.0 - range.1).unsigned_abs() as usize + 1
            } else {
                len.max(0) as usize
            });
            desc.push(down);
            probe = match probe {
                Const::Aggregate(items) => items.into_iter().next().unwrap_or(Const::Null),
                x => x,
            };
            ft = *fe;
            at = ad.map_or(*fe, |a| a.0);
        }
        let elem_ty = if open_packed { at } else { ft };
        let (elem, elem_size) = elem_shape(tys, elem_ty, open_packed.then_some(ft));
        let mut strides = vec![0; dims.len()];
        let mut step = elem_size;
        for k in (0..dims.len()).rev() {
            strides[k] = step;
            step *= sizes[k];
        }
        let len = step;
        let mut a = OpenArray {
            data: super::buffer(len),
            len,
            elem_ty,
            elem,
            elem_size,
            dims,
            sizes,
            desc,
            strides,
            signed: tys.is_integral(elem_ty) && tys.is_signed(elem_ty),
        };
        if fill {
            a.fill(tys, 0, 0, v, strings);
        }
        a
    }

    fn size(&self, k: usize) -> usize {
        self.sizes[k]
    }

    fn bytes(&mut self) -> &mut [u8] {
        let n = self.len;
        &mut super::bytes_mut(&mut self.data)[..n]
    }

    fn fill(&mut self, tys: &Types, level: usize, at: usize, v: &Const, strings: &mut Vec<CString>) {
        if level == self.dims.len() {
            let (elem, ty, size) = (self.elem, self.elem_ty, self.elem_size);
            if at + size > self.len {
                return;
            }
            let out = &mut self.bytes()[at..at + size];
            match elem {
                Elem::Scalar => {
                    let b = super::bits_of(tys, v, ty);
                    out[0] = super::logic_code(b.bit(0));
                }
                Elem::Vec(w, four) => {
                    let b = match v {
                        Const::Bits(b) => b.resize(w, b.signed()),
                        _ => super::bits_of(tys, v, ty),
                    };
                    put_vec(&b, w, four, out);
                }
                Elem::Other => put(tys, ty, v, out, strings),
            }
            return;
        }
        let Const::Aggregate(items) = v else { return };
        let n = self.size(level);
        for c in 0..n.min(items.len()) {
            let k = if self.desc[level] { n - 1 - c } else { c };
            if let Some(item) = items.get(k) {
                self.fill(tys, level + 1, at + c * self.strides[level], item, strings);
            }
        }
    }

    /// The value the C code left, shaped as the actual.
    pub(super) fn read(&self, tys: &Types) -> Const {
        self.read_at(tys, 0, 0)
    }

    fn read_at(&self, tys: &Types, level: usize, at: usize) -> Const {
        let all = &super::bytes(&self.data)[..self.len];
        if level == self.dims.len() {
            if at + self.elem_size > self.len {
                return crate::design::ops::default_value(tys, self.elem_ty);
            }
            let p = &all[at..at + self.elem_size];
            return match self.elem {
                Elem::Scalar => crate::design::ops::convert(tys, &Const::Bits(Bits::of_logic(super::code_logic(p[0]))), self.elem_ty),
                Elem::Vec(w, four) => Const::Bits(get_vec(w, four, self.signed, p)),
                Elem::Other => get(tys, self.elem_ty, p),
            };
        }
        let n = self.size(level);
        let mut items: Vec<Const> = (0..n).map(|c| self.read_at(tys, level + 1, at + c * self.strides[level])).collect();
        if self.desc[level] {
            items.reverse();
        }
        Const::Aggregate(items)
    }

    /// The address of the element at `idx`, one index per unpacked
    /// dimension, in the actual's own ranges.
    fn elem_ptr(&self, idx: &[i32]) -> *mut u8 {
        if idx.len() < self.dims.len() || self.len == 0 {
            return null_mut();
        }
        let mut at = 0;
        for (k, &(l, r)) in self.dims.iter().enumerate() {
            let pos = idx[k] as i64 - l.min(r);
            if pos < 0 || pos as usize >= self.sizes[k] {
                return null_mut();
            }
            at += pos as usize * self.strides[k];
        }
        unsafe { (self.data.as_ptr() as *mut u8).add(at) }
    }

    /// The packed dimension's range: the elements' width, normalised.
    fn packed(&self) -> (i64, i64) {
        match self.elem {
            Elem::Vec(w, _) => (w as i64 - 1, 0),
            Elem::Scalar => (0, 0),
            Elem::Other => (-1, 0),
        }
    }

    fn range(&self, d: i32) -> Option<(i64, i64)> {
        match d {
            0 => Some(self.packed()),
            d if d >= 1 && (d as usize) <= self.dims.len() => Some(self.dims[d as usize - 1]),
            _ => None,
        }
    }
}

fn put_vec(b: &Bits, w: u32, four: bool, out: &mut [u8]) {
    for k in 0..words(w) {
        let (a, u) = b.chunk32(k as u32);
        if four {
            out[k * 8..k * 8 + 4].copy_from_slice(&a.to_le_bytes());
            out[k * 8 + 4..k * 8 + 8].copy_from_slice(&u.to_le_bytes());
        } else {
            out[k * 4..k * 4 + 4].copy_from_slice(&(a & !u).to_le_bytes());
        }
    }
}

fn get_vec(w: u32, four: bool, signed: bool, p: &[u8]) -> Bits {
    let word = |at: usize| u32::from_le_bytes(p[at..at + 4].try_into().unwrap());
    let chunks = (0..words(w)).map(|k| if four { (word(k * 8), word(k * 8 + 4)) } else { (word(k * 4), 0) });
    Bits::from_chunks32(w, signed, chunks)
}

fn handle<'x>(h: *const c_void) -> Option<&'x OpenArray> {
    // SAFETY: a handle is an `OpenArray` passed to the running call.
    if h.is_null() { None } else { Some(unsafe { &*(h as *const OpenArray) }) }
}

fn scope<'x>(s: *const c_void) -> Option<&'x ScopeRec> {
    // SAFETY: a scope is a record the simulation made.
    if s.is_null() { None } else { Some(unsafe { &*(s as *const ScopeRec) }) }
}

fn mask(w: u32) -> u64 {
    if w >= 32 { 0xffff_ffff } else { (1u64 << w) - 1 }
}

/// `w` (at most 32) bits from bit `i` of the words `word(0)`, `word(1)`...
fn extract(word: impl Fn(usize) -> u32, i: i32, w: i32) -> u32 {
    let (i, w) = (i.max(0) as usize, w.clamp(0, 32) as u32);
    let (k, off) = (i / 32, (i % 32) as u32);
    let mut x = word(k) as u64 >> off;
    if off + w > 32 {
        x |= (word(k + 1) as u64) << (32 - off);
    }
    (x & mask(w)) as u32
}

/// Puts the low `w` (at most 32) bits of `v` at bit `i` of the words.
fn insert(word: impl Fn(usize) -> u32, mut set: impl FnMut(usize, u32), i: i32, w: i32, v: u32) {
    let (i, w) = (i.max(0) as usize, w.clamp(0, 32) as u32);
    if w == 0 {
        return;
    }
    let (k, off) = (i / 32, (i % 32) as u32);
    let two = off + w > 32;
    let cur = word(k) as u64 | if two { (word(k + 1) as u64) << 32 } else { 0 };
    let m = mask(w) << off;
    let new = cur & !m | ((v as u64) << off) & m;
    set(k, new as u32);
    if two {
        set(k + 1, (new >> 32) as u32);
    }
}

// ---- version ----

pub unsafe extern "C" fn svDpiVersion() -> *const c_char {
    c"1800-2005".as_ptr()
}

// ---- bit and part selects (H.11.5) ----

pub unsafe extern "C" fn svGetBitselBit(s: *const u32, i: i32) -> u8 {
    if s.is_null() || i < 0 {
        return 0;
    }
    (unsafe { *s.add(i as usize / 32) } >> (i % 32) & 1) as u8
}

pub unsafe extern "C" fn svGetBitselLogic(s: *const LogicVecVal, i: i32) -> u8 {
    if s.is_null() || i < 0 {
        return 3;
    }
    let w = unsafe { *s.add(i as usize / 32) };
    ((w.aval >> (i % 32) & 1) | (w.bval >> (i % 32) & 1) << 1) as u8
}

pub unsafe extern "C" fn svPutBitselBit(d: *mut u32, i: i32, s: u8) {
    if d.is_null() || i < 0 {
        return;
    }
    let w = unsafe { &mut *d.add(i as usize / 32) };
    *w = *w & !(1 << (i % 32)) | ((s & 1) as u32) << (i % 32);
}

pub unsafe extern "C" fn svPutBitselLogic(d: *mut LogicVecVal, i: i32, s: u8) {
    if d.is_null() || i < 0 {
        return;
    }
    let w = unsafe { &mut *d.add(i as usize / 32) };
    let b = i % 32;
    w.aval = w.aval & !(1 << b) | ((s & 1) as u32) << b;
    w.bval = w.bval & !(1 << b) | ((s >> 1 & 1) as u32) << b;
}

pub unsafe extern "C" fn svGetPartselBit(d: *mut u32, s: *const u32, i: i32, w: i32) {
    if d.is_null() || s.is_null() {
        return;
    }
    let v = extract(|k| unsafe { *s.add(k) }, i, w);
    let keep = !(mask(w.clamp(0, 32) as u32) as u32);
    unsafe { *d = *d & keep | v };
}

pub unsafe extern "C" fn svGetPartselLogic(d: *mut LogicVecVal, s: *const LogicVecVal, i: i32, w: i32) {
    if d.is_null() || s.is_null() {
        return;
    }
    let a = extract(|k| unsafe { (*s.add(k)).aval }, i, w);
    let b = extract(|k| unsafe { (*s.add(k)).bval }, i, w);
    let keep = !(mask(w.clamp(0, 32) as u32) as u32);
    unsafe {
        (*d).aval = (*d).aval & keep | a;
        (*d).bval = (*d).bval & keep | b;
    }
}

pub unsafe extern "C" fn svPutPartselBit(d: *mut u32, s: u32, i: i32, w: i32) {
    if d.is_null() {
        return;
    }
    insert(|k| unsafe { *d.add(k) }, |k, v| unsafe { *d.add(k) = v }, i, w, s);
}

pub unsafe extern "C" fn svPutPartselLogic(d: *mut LogicVecVal, s: LogicVecVal, i: i32, w: i32) {
    if d.is_null() {
        return;
    }
    insert(|k| unsafe { (*d.add(k)).aval }, |k, v| unsafe { (*d.add(k)).aval = v }, i, w, s.aval);
    insert(|k| unsafe { (*d.add(k)).bval }, |k, v| unsafe { (*d.add(k)).bval = v }, i, w, s.bval);
}

// ---- open arrays (H.12) ----

pub unsafe extern "C" fn svLeft(h: *const c_void, d: i32) -> i32 {
    handle(h).and_then(|a| a.range(d)).map_or(0, |r| r.0 as i32)
}

pub unsafe extern "C" fn svRight(h: *const c_void, d: i32) -> i32 {
    handle(h).and_then(|a| a.range(d)).map_or(0, |r| r.1 as i32)
}

pub unsafe extern "C" fn svLow(h: *const c_void, d: i32) -> i32 {
    handle(h).and_then(|a| a.range(d)).map_or(0, |r| r.0.min(r.1) as i32)
}

pub unsafe extern "C" fn svHigh(h: *const c_void, d: i32) -> i32 {
    handle(h).and_then(|a| a.range(d)).map_or(0, |r| r.0.max(r.1) as i32)
}

pub unsafe extern "C" fn svIncrement(h: *const c_void, d: i32) -> i32 {
    handle(h).and_then(|a| a.range(d)).map_or(0, |r| if r.0 >= r.1 { 1 } else { -1 })
}

pub unsafe extern "C" fn svSize(h: *const c_void, d: i32) -> i32 {
    let Some(a) = handle(h) else { return 0 };
    match d {
        0 => match a.elem {
            Elem::Vec(w, _) => w as i32,
            Elem::Scalar => 1,
            Elem::Other => 0,
        },
        _ => a.range(d).map_or(0, |_| a.size(d as usize - 1) as i32),
    }
}

pub unsafe extern "C" fn svDimensions(h: *const c_void) -> i32 {
    handle(h).map_or(0, |a| a.dims.len().max(1) as i32)
}

pub unsafe extern "C" fn svGetArrayPtr(h: *const c_void) -> *mut c_void {
    match handle(h) {
        Some(a) if a.len > 0 => a.data.as_ptr() as *mut c_void,
        _ => null_mut(),
    }
}

pub unsafe extern "C" fn svSizeOfArray(h: *const c_void) -> i32 {
    handle(h).map_or(0, |a| a.len as i32)
}

pub unsafe extern "C" fn svGetArrElemPtr1(h: *const c_void, i1: i32) -> *mut c_void {
    handle(h).map_or(null_mut(), |a| a.elem_ptr(&[i1]) as *mut c_void)
}

pub unsafe extern "C" fn svGetArrElemPtr2(h: *const c_void, i1: i32, i2: i32) -> *mut c_void {
    handle(h).map_or(null_mut(), |a| a.elem_ptr(&[i1, i2]) as *mut c_void)
}

pub unsafe extern "C" fn svGetArrElemPtr3(h: *const c_void, i1: i32, i2: i32, i3: i32) -> *mut c_void {
    handle(h).map_or(null_mut(), |a| a.elem_ptr(&[i1, i2, i3]) as *mut c_void)
}

/// Copies an element out as canonical `svBitVecVal` words.
fn get_bit_vec(d: *mut u32, h: *const c_void, idx: &[i32]) {
    let Some(a) = handle(h) else { return };
    let p = a.elem_ptr(idx);
    if p.is_null() || d.is_null() {
        return;
    }
    unsafe {
        match a.elem {
            Elem::Vec(w, four) => {
                for k in 0..words(w) {
                    *d.add(k) = if four {
                        let (av, bv) = (*(p.add(k * 8) as *const u32), *(p.add(k * 8 + 4) as *const u32));
                        av & !bv
                    } else {
                        *(p.add(k * 4) as *const u32)
                    };
                }
            }
            Elem::Scalar => *d = (*p == 1) as u32,
            Elem::Other => std::ptr::copy_nonoverlapping(p, d as *mut u8, a.elem_size),
        }
    }
}

fn get_logic_vec(d: *mut LogicVecVal, h: *const c_void, idx: &[i32]) {
    let Some(a) = handle(h) else { return };
    let p = a.elem_ptr(idx);
    if p.is_null() || d.is_null() {
        return;
    }
    unsafe {
        match a.elem {
            Elem::Vec(w, four) => {
                for k in 0..words(w) {
                    *d.add(k) = if four {
                        LogicVecVal { aval: *(p.add(k * 8) as *const u32), bval: *(p.add(k * 8 + 4) as *const u32) }
                    } else {
                        LogicVecVal { aval: *(p.add(k * 4) as *const u32), bval: 0 }
                    };
                }
            }
            Elem::Scalar => *d = LogicVecVal { aval: (*p & 1) as u32, bval: (*p >> 1 & 1) as u32 },
            Elem::Other => {}
        }
    }
}

fn put_bit_vec(h: *const c_void, s: *const u32, idx: &[i32]) {
    let Some(a) = handle(h) else { return };
    let p = a.elem_ptr(idx);
    if p.is_null() || s.is_null() {
        return;
    }
    unsafe {
        match a.elem {
            Elem::Vec(w, four) => {
                for k in 0..words(w) {
                    let v = *s.add(k);
                    if four {
                        *(p.add(k * 8) as *mut u32) = v;
                        *(p.add(k * 8 + 4) as *mut u32) = 0;
                    } else {
                        *(p.add(k * 4) as *mut u32) = v;
                    }
                }
            }
            Elem::Scalar => *p = (*s & 1) as u8,
            Elem::Other => std::ptr::copy_nonoverlapping(s as *const u8, p, a.elem_size),
        }
    }
}

fn put_logic_vec(h: *const c_void, s: *const LogicVecVal, idx: &[i32]) {
    let Some(a) = handle(h) else { return };
    let p = a.elem_ptr(idx);
    if p.is_null() || s.is_null() {
        return;
    }
    unsafe {
        match a.elem {
            Elem::Vec(w, four) => {
                for k in 0..words(w) {
                    let v = *s.add(k);
                    if four {
                        *(p.add(k * 8) as *mut u32) = v.aval;
                        *(p.add(k * 8 + 4) as *mut u32) = v.bval;
                    } else {
                        *(p.add(k * 4) as *mut u32) = v.aval & !v.bval;
                    }
                }
            }
            Elem::Scalar => *p = ((*s).aval & 1 | ((*s).bval & 1) << 1) as u8,
            Elem::Other => {}
        }
    }
}

fn get_scalar(h: *const c_void, idx: &[i32]) -> u8 {
    let Some(a) = handle(h) else { return 3 };
    let p = a.elem_ptr(idx);
    if p.is_null() {
        return 3;
    }
    unsafe {
        match a.elem {
            Elem::Scalar => *p,
            Elem::Vec(_, true) => ((*(p as *const u32) & 1) | (*(p.add(4) as *const u32) & 1) << 1) as u8,
            Elem::Vec(_, false) => (*(p as *const u32) & 1) as u8,
            Elem::Other => 3,
        }
    }
}

fn put_scalar(h: *const c_void, v: u8, idx: &[i32]) {
    let Some(a) = handle(h) else { return };
    let p = a.elem_ptr(idx);
    if p.is_null() {
        return;
    }
    let v = v & 3;
    unsafe {
        match a.elem {
            Elem::Scalar => *p = v,
            Elem::Vec(_, true) => {
                let (av, bv) = (p as *mut u32, p.add(4) as *mut u32);
                *av = *av & !1 | (v & 1) as u32;
                *bv = *bv & !1 | (v >> 1) as u32;
            }
            Elem::Vec(_, false) => {
                let av = p as *mut u32;
                *av = *av & !1 | (v == 1) as u32;
            }
            Elem::Other => {}
        }
    }
}

pub unsafe extern "C" fn svGetBitArrElem1VecVal(d: *mut u32, s: *const c_void, i1: i32) {
    get_bit_vec(d, s, &[i1])
}
pub unsafe extern "C" fn svGetBitArrElem2VecVal(d: *mut u32, s: *const c_void, i1: i32, i2: i32) {
    get_bit_vec(d, s, &[i1, i2])
}
pub unsafe extern "C" fn svGetBitArrElem3VecVal(d: *mut u32, s: *const c_void, i1: i32, i2: i32, i3: i32) {
    get_bit_vec(d, s, &[i1, i2, i3])
}
pub unsafe extern "C" fn svGetLogicArrElem1VecVal(d: *mut LogicVecVal, s: *const c_void, i1: i32) {
    get_logic_vec(d, s, &[i1])
}
pub unsafe extern "C" fn svGetLogicArrElem2VecVal(d: *mut LogicVecVal, s: *const c_void, i1: i32, i2: i32) {
    get_logic_vec(d, s, &[i1, i2])
}
pub unsafe extern "C" fn svGetLogicArrElem3VecVal(d: *mut LogicVecVal, s: *const c_void, i1: i32, i2: i32, i3: i32) {
    get_logic_vec(d, s, &[i1, i2, i3])
}
pub unsafe extern "C" fn svPutBitArrElem1VecVal(d: *const c_void, s: *const u32, i1: i32) {
    put_bit_vec(d, s, &[i1])
}
pub unsafe extern "C" fn svPutBitArrElem2VecVal(d: *const c_void, s: *const u32, i1: i32, i2: i32) {
    put_bit_vec(d, s, &[i1, i2])
}
pub unsafe extern "C" fn svPutBitArrElem3VecVal(d: *const c_void, s: *const u32, i1: i32, i2: i32, i3: i32) {
    put_bit_vec(d, s, &[i1, i2, i3])
}
pub unsafe extern "C" fn svPutLogicArrElem1VecVal(d: *const c_void, s: *const LogicVecVal, i1: i32) {
    put_logic_vec(d, s, &[i1])
}
pub unsafe extern "C" fn svPutLogicArrElem2VecVal(d: *const c_void, s: *const LogicVecVal, i1: i32, i2: i32) {
    put_logic_vec(d, s, &[i1, i2])
}
pub unsafe extern "C" fn svPutLogicArrElem3VecVal(d: *const c_void, s: *const LogicVecVal, i1: i32, i2: i32, i3: i32) {
    put_logic_vec(d, s, &[i1, i2, i3])
}
pub unsafe extern "C" fn svGetBitArrElem1(s: *const c_void, i1: i32) -> u8 {
    (get_scalar(s, &[i1]) == 1) as u8
}
pub unsafe extern "C" fn svGetBitArrElem2(s: *const c_void, i1: i32, i2: i32) -> u8 {
    (get_scalar(s, &[i1, i2]) == 1) as u8
}
pub unsafe extern "C" fn svGetBitArrElem3(s: *const c_void, i1: i32, i2: i32, i3: i32) -> u8 {
    (get_scalar(s, &[i1, i2, i3]) == 1) as u8
}
pub unsafe extern "C" fn svGetLogicArrElem1(s: *const c_void, i1: i32) -> u8 {
    get_scalar(s, &[i1])
}
pub unsafe extern "C" fn svGetLogicArrElem2(s: *const c_void, i1: i32, i2: i32) -> u8 {
    get_scalar(s, &[i1, i2])
}
pub unsafe extern "C" fn svGetLogicArrElem3(s: *const c_void, i1: i32, i2: i32, i3: i32) -> u8 {
    get_scalar(s, &[i1, i2, i3])
}
pub unsafe extern "C" fn svPutBitArrElem1(d: *const c_void, v: u8, i1: i32) {
    put_scalar(d, v & 1, &[i1])
}
pub unsafe extern "C" fn svPutBitArrElem2(d: *const c_void, v: u8, i1: i32, i2: i32) {
    put_scalar(d, v & 1, &[i1, i2])
}
pub unsafe extern "C" fn svPutBitArrElem3(d: *const c_void, v: u8, i1: i32, i2: i32, i3: i32) {
    put_scalar(d, v & 1, &[i1, i2, i3])
}
pub unsafe extern "C" fn svPutLogicArrElem1(d: *const c_void, v: u8, i1: i32) {
    put_scalar(d, v, &[i1])
}
pub unsafe extern "C" fn svPutLogicArrElem2(d: *const c_void, v: u8, i1: i32, i2: i32) {
    put_scalar(d, v, &[i1, i2])
}
pub unsafe extern "C" fn svPutLogicArrElem3(d: *const c_void, v: u8, i1: i32, i2: i32, i3: i32) {
    put_scalar(d, v, &[i1, i2, i3])
}

/// The functions with a variable number of indices, given their argument
/// words (see the module's comment).
pub mod varargs {
    use super::*;

    /// The words after `fixed` named arguments: the indices.
    fn rest(a: &[u64; 14], fixed: usize) -> [i32; 6] {
        let from = if cfg!(all(target_arch = "aarch64", target_vendor = "apple")) { 8 } else { fixed };
        std::array::from_fn(|k| a[from + k] as u32 as i32)
    }

    /// The indices a handle takes, after `fixed` named arguments, the
    /// last of which is the first index.
    fn idx(h: *const c_void, a: &[u64; 14], fixed: usize) -> Vec<i32> {
        let n = handle(h).map_or(1, |x| x.dims.len().max(1));
        let mut v = vec![a[fixed - 1] as u32 as i32];
        v.extend(rest(a, fixed).iter().take(n - 1));
        v
    }

    pub unsafe fn svGetArrElemPtr(a: &[u64; 14]) -> *mut c_void {
        let h = a[0] as *const c_void;
        handle(h).map_or(null_mut(), |x| x.elem_ptr(&idx(h, a, 2)) as *mut c_void)
    }

    pub unsafe fn svGetBitArrElemVecVal(a: &[u64; 14]) {
        let h = a[1] as *const c_void;
        get_bit_vec(a[0] as *mut u32, h, &idx(h, a, 3))
    }

    pub unsafe fn svGetLogicArrElemVecVal(a: &[u64; 14]) {
        let h = a[1] as *const c_void;
        get_logic_vec(a[0] as *mut LogicVecVal, h, &idx(h, a, 3))
    }

    pub unsafe fn svPutBitArrElemVecVal(a: &[u64; 14]) {
        let h = a[0] as *const c_void;
        put_bit_vec(h, a[1] as *const u32, &idx(h, a, 3))
    }

    pub unsafe fn svPutLogicArrElemVecVal(a: &[u64; 14]) {
        let h = a[0] as *const c_void;
        put_logic_vec(h, a[1] as *const LogicVecVal, &idx(h, a, 3))
    }

    pub unsafe fn svGetBitArrElem(a: &[u64; 14]) -> u8 {
        let h = a[0] as *const c_void;
        (get_scalar(h, &idx(h, a, 2)) == 1) as u8
    }

    pub unsafe fn svGetLogicArrElem(a: &[u64; 14]) -> u8 {
        let h = a[0] as *const c_void;
        get_scalar(h, &idx(h, a, 2))
    }

    pub unsafe fn svPutBitArrElem(a: &[u64; 14]) {
        let h = a[0] as *const c_void;
        put_scalar(h, a[1] as u8 & 1, &idx(h, a, 3))
    }

    pub unsafe fn svPutLogicArrElem(a: &[u64; 14]) {
        let h = a[0] as *const c_void;
        put_scalar(h, a[1] as u8, &idx(h, a, 3))
    }
}

// ---- context (H.9) ----

pub unsafe extern "C" fn svGetScope() -> *mut c_void {
    CHAIN.with(|c| c.get().scope) as *mut c_void
}

pub unsafe extern "C" fn svSetScope(s: *const c_void) -> *mut c_void {
    CHAIN.with(|c| {
        let old = c.get();
        c.set(Chain { scope: s as usize, ..old });
        old.scope as *mut c_void
    })
}

pub unsafe extern "C" fn svGetNameFromScope(s: *const c_void) -> *const c_char {
    scope(s).map_or(std::ptr::null(), |r| r.name.as_ptr())
}

pub unsafe extern "C" fn svGetScopeFromName(name: *const c_char) -> *mut c_void {
    if name.is_null() {
        return null_mut();
    }
    let name = unsafe { CStr::from_ptr(name) }.to_string_lossy().into_owned();
    match current_sim() {
        Some(s) if s.dpi.is_some() => s.dpi_scope_named(&name) as *mut c_void,
        _ => null_mut(),
    }
}

pub unsafe extern "C" fn svPutUserData(s: *const c_void, key: *mut c_void, data: *mut c_void) -> i32 {
    let Some(r) = scope(s) else { return -1 };
    if key.is_null() || data.is_null() {
        return -1;
    }
    r.user.borrow_mut().insert(key as usize, data as usize);
    0
}

pub unsafe extern "C" fn svGetUserData(s: *const c_void, key: *mut c_void) -> *mut c_void {
    let Some(r) = scope(s) else { return null_mut() };
    r.user.borrow().get(&(key as usize)).map_or(null_mut(), |&d| d as *mut c_void)
}

/// Where the call came from is not kept, so this says so (H.9.3).
pub unsafe extern "C" fn svGetCallerInfo(_file: *mut *const c_char, _line: *mut i32) -> i32 {
    0
}

// ---- disabling (35.9) ----

pub unsafe extern "C" fn svIsDisabledState() -> i32 {
    CHAIN.with(|c| c.get().disabled) as i32
}

pub unsafe extern "C" fn svAckDisabledState() {}

// ---- time (H.13) ----

fn pow10(e: i32) -> f64 {
    10f64.powi(e)
}

pub unsafe extern "C" fn svGetTime(s: *const c_void, t: *mut TimeVal) -> i32 {
    let Some(sim) = current_sim() else { return -1 };
    if t.is_null() {
        return -1;
    }
    let (now, precision) = sim.dpi_now();
    let t = unsafe { &mut *t };
    if t.type_ == 1 {
        // `sv_scaled_real_time`: in the scope's time unit.
        let (unit, _) = sim.dpi_time_scale(s as usize);
        t.real = now as f64 * pow10(precision as i32 - unit as i32);
    } else {
        t.high = (now >> 32) as u32;
        t.low = now as u32;
    }
    0
}

pub unsafe extern "C" fn svGetTimeUnit(s: *const c_void, unit: *mut i32) -> i32 {
    let Some(sim) = current_sim() else { return -1 };
    if unit.is_null() {
        return -1;
    }
    unsafe { *unit = sim.dpi_time_scale(s as usize).0 as i32 };
    0
}

pub unsafe extern "C" fn svGetTimePrecision(s: *const c_void, precision: *mut i32) -> i32 {
    let Some(sim) = current_sim() else { return -1 };
    if precision.is_null() {
        return -1;
    }
    unsafe { *precision = sim.dpi_time_scale(s as usize).1 as i32 };
    0
}

/// Defines the `svdpi.h` functions in the crate that calls it, under the
/// names Annex H gives them, so that an executable that loads DPI
/// libraries has them for the libraries to find.
#[macro_export]
macro_rules! export_dpi {
    () => {
        $crate::export_dpi!(@fns
            svDpiVersion() -> *const ::std::ffi::c_char;
            svGetBitselBit(s: *const u32, i: i32) -> u8;
            svGetBitselLogic(s: *const $crate::sim::dpi::api::LogicVecVal, i: i32) -> u8;
            svPutBitselBit(d: *mut u32, i: i32, s: u8) -> ();
            svPutBitselLogic(d: *mut $crate::sim::dpi::api::LogicVecVal, i: i32, s: u8) -> ();
            svGetPartselBit(d: *mut u32, s: *const u32, i: i32, w: i32) -> ();
            svGetPartselLogic(d: *mut $crate::sim::dpi::api::LogicVecVal, s: *const $crate::sim::dpi::api::LogicVecVal, i: i32, w: i32) -> ();
            svPutPartselBit(d: *mut u32, s: u32, i: i32, w: i32) -> ();
            svPutPartselLogic(d: *mut $crate::sim::dpi::api::LogicVecVal, s: $crate::sim::dpi::api::LogicVecVal, i: i32, w: i32) -> ();
            svLeft(h: *const ::std::ffi::c_void, d: i32) -> i32;
            svRight(h: *const ::std::ffi::c_void, d: i32) -> i32;
            svLow(h: *const ::std::ffi::c_void, d: i32) -> i32;
            svHigh(h: *const ::std::ffi::c_void, d: i32) -> i32;
            svIncrement(h: *const ::std::ffi::c_void, d: i32) -> i32;
            svSize(h: *const ::std::ffi::c_void, d: i32) -> i32;
            svDimensions(h: *const ::std::ffi::c_void) -> i32;
            svGetArrayPtr(h: *const ::std::ffi::c_void) -> *mut ::std::ffi::c_void;
            svSizeOfArray(h: *const ::std::ffi::c_void) -> i32;
            svGetArrElemPtr1(h: *const ::std::ffi::c_void, i1: i32) -> *mut ::std::ffi::c_void;
            svGetArrElemPtr2(h: *const ::std::ffi::c_void, i1: i32, i2: i32) -> *mut ::std::ffi::c_void;
            svGetArrElemPtr3(h: *const ::std::ffi::c_void, i1: i32, i2: i32, i3: i32) -> *mut ::std::ffi::c_void;
            svGetBitArrElem1VecVal(d: *mut u32, s: *const ::std::ffi::c_void, i1: i32) -> ();
            svGetBitArrElem2VecVal(d: *mut u32, s: *const ::std::ffi::c_void, i1: i32, i2: i32) -> ();
            svGetBitArrElem3VecVal(d: *mut u32, s: *const ::std::ffi::c_void, i1: i32, i2: i32, i3: i32) -> ();
            svGetLogicArrElem1VecVal(d: *mut $crate::sim::dpi::api::LogicVecVal, s: *const ::std::ffi::c_void, i1: i32) -> ();
            svGetLogicArrElem2VecVal(d: *mut $crate::sim::dpi::api::LogicVecVal, s: *const ::std::ffi::c_void, i1: i32, i2: i32) -> ();
            svGetLogicArrElem3VecVal(d: *mut $crate::sim::dpi::api::LogicVecVal, s: *const ::std::ffi::c_void, i1: i32, i2: i32, i3: i32) -> ();
            svPutBitArrElem1VecVal(d: *const ::std::ffi::c_void, s: *const u32, i1: i32) -> ();
            svPutBitArrElem2VecVal(d: *const ::std::ffi::c_void, s: *const u32, i1: i32, i2: i32) -> ();
            svPutBitArrElem3VecVal(d: *const ::std::ffi::c_void, s: *const u32, i1: i32, i2: i32, i3: i32) -> ();
            svPutLogicArrElem1VecVal(d: *const ::std::ffi::c_void, s: *const $crate::sim::dpi::api::LogicVecVal, i1: i32) -> ();
            svPutLogicArrElem2VecVal(d: *const ::std::ffi::c_void, s: *const $crate::sim::dpi::api::LogicVecVal, i1: i32, i2: i32) -> ();
            svPutLogicArrElem3VecVal(d: *const ::std::ffi::c_void, s: *const $crate::sim::dpi::api::LogicVecVal, i1: i32, i2: i32, i3: i32) -> ();
            svGetBitArrElem1(s: *const ::std::ffi::c_void, i1: i32) -> u8;
            svGetBitArrElem2(s: *const ::std::ffi::c_void, i1: i32, i2: i32) -> u8;
            svGetBitArrElem3(s: *const ::std::ffi::c_void, i1: i32, i2: i32, i3: i32) -> u8;
            svGetLogicArrElem1(s: *const ::std::ffi::c_void, i1: i32) -> u8;
            svGetLogicArrElem2(s: *const ::std::ffi::c_void, i1: i32, i2: i32) -> u8;
            svGetLogicArrElem3(s: *const ::std::ffi::c_void, i1: i32, i2: i32, i3: i32) -> u8;
            svPutBitArrElem1(d: *const ::std::ffi::c_void, v: u8, i1: i32) -> ();
            svPutBitArrElem2(d: *const ::std::ffi::c_void, v: u8, i1: i32, i2: i32) -> ();
            svPutBitArrElem3(d: *const ::std::ffi::c_void, v: u8, i1: i32, i2: i32, i3: i32) -> ();
            svPutLogicArrElem1(d: *const ::std::ffi::c_void, v: u8, i1: i32) -> ();
            svPutLogicArrElem2(d: *const ::std::ffi::c_void, v: u8, i1: i32, i2: i32) -> ();
            svPutLogicArrElem3(d: *const ::std::ffi::c_void, v: u8, i1: i32, i2: i32, i3: i32) -> ();
            svGetScope() -> *mut ::std::ffi::c_void;
            svSetScope(s: *const ::std::ffi::c_void) -> *mut ::std::ffi::c_void;
            svGetNameFromScope(s: *const ::std::ffi::c_void) -> *const ::std::ffi::c_char;
            svGetScopeFromName(name: *const ::std::ffi::c_char) -> *mut ::std::ffi::c_void;
            svPutUserData(s: *const ::std::ffi::c_void, key: *mut ::std::ffi::c_void, data: *mut ::std::ffi::c_void) -> i32;
            svGetUserData(s: *const ::std::ffi::c_void, key: *mut ::std::ffi::c_void) -> *mut ::std::ffi::c_void;
            svGetCallerInfo(file: *mut *const ::std::ffi::c_char, line: *mut i32) -> i32;
            svIsDisabledState() -> i32;
            svAckDisabledState() -> ();
            svGetTime(s: *const ::std::ffi::c_void, t: *mut $crate::sim::dpi::api::TimeVal) -> i32;
            svGetTimeUnit(s: *const ::std::ffi::c_void, unit: *mut i32) -> i32;
            svGetTimePrecision(s: *const ::std::ffi::c_void, precision: *mut i32) -> i32;
        );
        $crate::export_dpi!(@var
            svGetArrElemPtr -> *mut ::std::ffi::c_void;
            svGetBitArrElemVecVal -> ();
            svGetLogicArrElemVecVal -> ();
            svPutBitArrElemVecVal -> ();
            svPutLogicArrElemVecVal -> ();
            svGetBitArrElem -> u8;
            svGetLogicArrElem -> u8;
            svPutBitArrElem -> ();
            svPutLogicArrElem -> ();
        );
    };
    (@fns $($name:ident($($arg:ident: $ty:ty),*) -> $ret:ty;)*) => {
        $(
            /// # Safety
            /// Called by C code as Annex H says.
            #[unsafe(no_mangle)]
            pub unsafe extern "C" fn $name($($arg: $ty),*) -> $ret {
                unsafe { $crate::sim::dpi::api::$name($($arg),*) }
            }
        )*
        /// Keeps the functions in the executable, which nothing in it calls.
        struct DpiExports(&'static [*const ()]);
        unsafe impl Sync for DpiExports {}
        #[used]
        static DPI_EXPORTS: DpiExports = DpiExports(&[$($name as *const ()),*]);
    };
    (@var $($name:ident -> $ret:ty;)*) => {
        $(
            /// # Safety
            /// Called by C code as Annex H says, with its indices after the
            /// named arguments.
            #[unsafe(no_mangle)]
            #[allow(clippy::too_many_arguments)]
            pub unsafe extern "C" fn $name(
                a0: u64, a1: u64, a2: u64, a3: u64, a4: u64, a5: u64, a6: u64,
                a7: u64, a8: u64, a9: u64, a10: u64, a11: u64, a12: u64, a13: u64,
            ) -> $ret {
                unsafe { $crate::sim::dpi::api::varargs::$name(&[a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11, a12, a13]) }
            }
        )*
        struct DpiVarExports(&'static [*const ()]);
        unsafe impl Sync for DpiVarExports {}
        #[used]
        static DPI_VAR_EXPORTS: DpiVarExports = DpiVarExports(&[$($name as *const ()),*]);
    };
}
