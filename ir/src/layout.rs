//! The data layout: how big types are and where they sit in memory.
//!
//! Parsed from `target datalayout = "..."`, starting from the defaults the
//! Language Reference lists (see `docs/llvm-ir/01-module-structure.md`,
//! section 12.2). Sizes and alignments are in bytes unless a name says
//! bits.
//!
//! Epitome's choices where the reference is silent:
//!
//! - a vector width with no `v` entry is aligned to its size rounded up to
//!   a power of two;
//! - a float width with no `f` entry likewise (so `x86_fp80` is 16-byte
//!   aligned unless the layout says `f80:...`).

use crate::types::{NamedStruct, Type, TypeId, Types};

/// Read access to a type table, so layouts work on the module's table and
/// on the tables passes use while running in parallel.
pub trait TypeAccess {
    fn ty(&self, id: TypeId) -> &Type;
    fn named(&self) -> &[NamedStruct];
}

impl TypeAccess for Types {
    fn ty(&self, id: TypeId) -> &Type {
        self.get(id)
    }
    fn named(&self) -> &[NamedStruct] {
        &self.structs
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Align {
    abi: u64,
    pref: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PointerSpec {
    pub addrspace: u32,
    pub size_bits: u32,
    pub abi: u64,
    pub pref: u64,
    /// Width of GEP indices, and of addresses.
    pub index_bits: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DataLayout {
    pub big_endian: bool,
    /// Natural stack alignment in bytes; 0 if unspecified.
    pub stack_align: u64,
    pub alloca_addrspace: u32,
    pub program_addrspace: u32,
    pub globals_addrspace: u32,
    pointers: Vec<PointerSpec>,
    /// (width in bits, alignment), sorted by width.
    ints: Vec<(u32, Align)>,
    floats: Vec<(u32, Align)>,
    vectors: Vec<(u32, Align)>,
    aggregate: Align,
    /// Native integer widths.
    pub native_ints: Vec<u32>,
}

impl Default for DataLayout {
    fn default() -> Self {
        let a = |abi: u64, pref: u64| Align { abi: abi / 8, pref: pref / 8 };
        DataLayout {
            big_endian: false,
            stack_align: 0,
            alloca_addrspace: 0,
            program_addrspace: 0,
            globals_addrspace: 0,
            pointers: vec![PointerSpec { addrspace: 0, size_bits: 64, abi: 8, pref: 8, index_bits: 64 }],
            ints: vec![(1, a(8, 8)), (8, a(8, 8)), (16, a(16, 16)), (32, a(32, 32)), (64, a(32, 64))],
            floats: vec![(16, a(16, 16)), (32, a(32, 32)), (64, a(64, 64)), (128, a(128, 128))],
            vectors: vec![(64, a(64, 64)), (128, a(128, 128))],
            aggregate: Align { abi: 1, pref: 8 },
            native_ints: Vec::new(),
        }
    }
}

fn set(list: &mut Vec<(u32, Align)>, bits: u32, align: Align) {
    match list.iter_mut().find(|(b, _)| *b == bits) {
        Some(e) => e.1 = align,
        None => {
            list.push((bits, align));
            list.sort_by_key(|e| e.0);
        }
    }
}

fn bytes(bits: &str, what: &str) -> Result<u64, String> {
    let b: u64 = bits.parse().map_err(|_| format!("bad {what} '{bits}'"))?;
    if !b.is_multiple_of(8) {
        return Err(format!("{what} {b} is not a whole number of bytes"));
    }
    Ok(b / 8)
}

/// The size in bits of a pointer in address space `addrspace`, as data
/// layout `spec` says (its `p` entries), without parsing the rest of it:
/// 64 where it says nothing.
pub fn pointer_bits(spec: Option<&[u8]>, addrspace: u32) -> u32 {
    let Some(spec) = spec else { return 64 };
    let mut default = 64;
    for item in spec.split(|&b| b == b'-') {
        let Some(rest) = item.strip_prefix(b"p") else { continue };
        let text = String::from_utf8_lossy(rest);
        let mut fields = text.split(':');
        let head = fields.next().unwrap_or("");
        let digits: String = head.chars().filter(|c| c.is_ascii_digit()).collect();
        let space: u32 = digits.parse().unwrap_or(0);
        let Some(size) = fields.next().and_then(|f| f.parse().ok()) else { continue };
        if space == addrspace {
            return size;
        }
        if space == 0 {
            default = size;
        }
    }
    default
}

impl DataLayout {
    pub fn parse(spec: &str) -> Result<DataLayout, String> {
        let mut dl = DataLayout::default();
        for item in spec.split('-').filter(|s| !s.is_empty()) {
            let (head, rest) = item.split_at(1);
            let fields: Vec<&str> = rest.split(':').collect();
            let align_from = |fields: &[&str], at: usize| -> Result<Align, String> {
                let abi = bytes(fields.get(at).copied().unwrap_or("0"), "alignment")?;
                let pref = match fields.get(at + 1) {
                    Some(p) => bytes(p, "alignment")?,
                    None => abi,
                };
                Ok(Align { abi: abi.max(1), pref: pref.max(1) })
            };
            match head {
                "e" => dl.big_endian = false,
                "E" => dl.big_endian = true,
                "S" => dl.stack_align = bytes(rest, "stack alignment")?,
                "A" => dl.alloca_addrspace = rest.parse().map_err(|_| format!("bad '{item}'"))?,
                "P" => dl.program_addrspace = rest.parse().map_err(|_| format!("bad '{item}'"))?,
                "G" => dl.globals_addrspace = rest.parse().map_err(|_| format!("bad '{item}'"))?,
                "p" => {
                    // p[flags][as][(name)]:size:abi[:pref[:idx]]
                    let first = fields[0];
                    let digits: String = first.chars().filter(|c| c.is_ascii_digit()).take_while(|_| true).collect();
                    let addrspace: u32 =
                        if digits.is_empty() { 0 } else { digits.parse().map_err(|_| format!("bad '{item}'"))? };
                    let size_bits: u32 =
                        fields.get(1).ok_or(format!("bad '{item}'"))?.parse().map_err(|_| format!("bad '{item}'"))?;
                    let align = align_from(&fields, 2)?;
                    let index_bits = match fields.get(4) {
                        Some(i) => i.parse().map_err(|_| format!("bad '{item}'"))?,
                        None => size_bits,
                    };
                    let spec = PointerSpec { addrspace, size_bits, abi: align.abi, pref: align.pref, index_bits };
                    match dl.pointers.iter_mut().find(|p| p.addrspace == addrspace) {
                        Some(p) => *p = spec,
                        None => dl.pointers.push(spec),
                    }
                }
                "i" | "f" | "v" if fields[0].chars().all(|c| c.is_ascii_digit()) && !fields[0].is_empty() => {
                    let bits: u32 = fields[0].parse().map_err(|_| format!("bad '{item}'"))?;
                    let align = align_from(&fields, 1)?;
                    let list = match head {
                        "i" => &mut dl.ints,
                        "f" => &mut dl.floats,
                        _ => &mut dl.vectors,
                    };
                    set(list, bits, align);
                }
                "a" => {
                    let abi = bytes(fields.get(1).copied().unwrap_or("0"), "alignment")?;
                    let pref = match fields.get(2) {
                        Some(p) => bytes(p, "alignment")?,
                        None => abi,
                    };
                    dl.aggregate = Align { abi: abi.max(1), pref: pref.max(1) };
                }
                "n" => {
                    dl.native_ints = rest.split(':').filter_map(|s| s.parse().ok()).collect();
                }
                // Mangling, function pointer alignment, `ve`, `ni`: nothing
                // here depends on them.
                _ => {}
            }
        }
        Ok(dl)
    }

    pub fn pointer(&self, addrspace: u32) -> PointerSpec {
        self.pointers
            .iter()
            .find(|p| p.addrspace == addrspace)
            .copied()
            .unwrap_or(PointerSpec { addrspace, ..self.pointers[0] })
    }

    fn int_align(&self, bits: u32) -> Align {
        // An exact match, else the smallest wider entry, else the widest.
        self.ints
            .iter()
            .find(|(b, _)| *b >= bits)
            .or(self.ints.last())
            .map(|e| e.1)
            .unwrap_or(Align { abi: 1, pref: 1 })
    }

    fn natural(size_bytes: u64) -> Align {
        let a = size_bytes.max(1).next_power_of_two();
        Align { abi: a, pref: a }
    }

    fn align(&self, t: &dyn TypeAccess, ty: TypeId) -> Option<Align> {
        Some(match t.ty(ty) {
            Type::Int(b) | Type::Byte(b) => self.int_align(*b),
            Type::Half | Type::BFloat | Type::Float | Type::Double | Type::X86Fp80 | Type::Fp128 | Type::PpcFp128 => {
                let bits = self.size_in_bits(t, ty)?;
                let bits = if matches!(t.ty(ty), Type::X86Fp80) { 80 } else { bits as u32 };
                match self.floats.iter().find(|(b, _)| *b == bits) {
                    Some(e) => e.1,
                    None => Self::natural(bits.div_ceil(8) as u64),
                }
            }
            Type::Ptr(a) => {
                let p = self.pointer(*a);
                Align { abi: p.abi, pref: p.pref }
            }
            Type::Vector { scalable: false, .. } => {
                let bits = self.size_in_bits(t, ty)?;
                match self.vectors.iter().find(|(b, _)| *b as u64 == bits) {
                    Some(e) => e.1,
                    None => Self::natural(bits.div_ceil(8)),
                }
            }
            Type::Array { elem, .. } => self.align(t, *elem)?,
            Type::Struct { packed, fields } => self.struct_layout(t, fields, *packed)?.align,
            Type::Named(i) => {
                let body = t.named()[*i as usize].body.as_ref()?;
                self.struct_layout(t, &body.fields, body.packed)?.align
            }
            _ => return None,
        })
    }

    /// The ABI alignment in bytes.
    pub fn abi_align(&self, t: &dyn TypeAccess, ty: TypeId) -> Option<u64> {
        self.align(t, ty).map(|a| a.abi)
    }

    /// The preferred alignment in bytes, used for allocas and globals.
    pub fn pref_align(&self, t: &dyn TypeAccess, ty: TypeId) -> Option<u64> {
        self.align(t, ty).map(|a| a.pref)
    }

    /// The number of bits a value of the type holds. `None` for types with
    /// no size, and for scalable vectors, whose size is only known at run
    /// time.
    pub fn size_in_bits(&self, t: &dyn TypeAccess, ty: TypeId) -> Option<u64> {
        Some(match t.ty(ty) {
            Type::Int(b) | Type::Byte(b) => *b as u64,
            Type::Half | Type::BFloat => 16,
            Type::Float => 32,
            Type::Double => 64,
            Type::X86Fp80 => 80,
            Type::Fp128 | Type::PpcFp128 => 128,
            Type::Ptr(a) => self.pointer(*a).size_bits as u64,
            Type::Vector { scalable: false, len, elem } => *len as u64 * self.size_in_bits(t, *elem)?,
            Type::Array { .. } | Type::Struct { .. } | Type::Named(_) => self.alloc_size(t, ty)? * 8,
            _ => return None,
        })
    }

    /// The bytes a store of the type writes.
    pub fn store_size(&self, t: &dyn TypeAccess, ty: TypeId) -> Option<u64> {
        match t.ty(ty) {
            Type::Array { .. } | Type::Struct { .. } | Type::Named(_) => self.alloc_size(t, ty),
            _ => Some(self.size_in_bits(t, ty)?.div_ceil(8)),
        }
    }

    /// The distance between consecutive values of the type in memory: the
    /// store size rounded up to the ABI alignment. GEP steps by this.
    pub fn alloc_size(&self, t: &dyn TypeAccess, ty: TypeId) -> Option<u64> {
        match t.ty(ty) {
            Type::Array { len, elem } => Some(len * self.alloc_size(t, *elem)?),
            Type::Struct { packed, fields } => Some(self.struct_layout(t, fields, *packed)?.size),
            Type::Named(i) => {
                let body = t.named()[*i as usize].body.as_ref()?;
                Some(self.struct_layout(t, &body.fields, body.packed)?.size)
            }
            _ => {
                let size = self.store_size(t, ty)?;
                let align = self.abi_align(t, ty)?;
                Some(size.div_ceil(align) * align)
            }
        }
    }

    /// Field offsets, size and alignment of a struct.
    pub fn struct_layout(&self, t: &dyn TypeAccess, fields: &[TypeId], packed: bool) -> Option<StructLayout> {
        let mut offsets = Vec::with_capacity(fields.len());
        let mut at = 0u64;
        let mut align = if packed { 1 } else { self.aggregate.abi };
        for &f in fields {
            let a = if packed { 1 } else { self.abi_align(t, f)? };
            at = at.div_ceil(a) * a;
            offsets.push(at);
            at += self.alloc_size(t, f)?;
            align = align.max(a);
        }
        let size = at.div_ceil(align) * align;
        Some(StructLayout {
            offsets,
            size,
            align: Align { abi: align, pref: align.max(if packed { 1 } else { self.aggregate.pref.min(align) }) },
        })
    }
}

/// Where a struct's fields go.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StructLayout {
    pub offsets: Vec<u64>,
    pub size: u64,
    align: Align,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn types() -> Types {
        Types::default()
    }

    #[test]
    fn struct_layouts() {
        let mut t = types();
        let i8 = t.int(8);
        let i32 = t.int(32);
        let i64 = t.int(64);
        let d = t.intern(Type::Double);
        let dl = DataLayout::default();
        let s = t.intern(Type::Struct { packed: false, fields: vec![i8, i32] });
        assert_eq!(dl.alloc_size(&t, s), Some(8));
        let p = t.intern(Type::Struct { packed: true, fields: vec![i8, i32] });
        assert_eq!(dl.alloc_size(&t, p), Some(5));
        let sd = t.intern(Type::Struct { packed: false, fields: vec![i8, d] });
        assert_eq!(dl.struct_layout(&t, &[i8, d], false).unwrap().offsets, vec![0, 8]);
        assert_eq!(dl.alloc_size(&t, sd), Some(16));
        // The default gives i64 a 4-byte ABI alignment.
        assert_eq!(dl.struct_layout(&t, &[i8, i64], false).unwrap().offsets, vec![0, 4]);
        let dl64 = DataLayout::parse("e-m:o-i64:64-i128:128-n32:64-S128").unwrap();
        assert_eq!(dl64.struct_layout(&t, &[i8, i64], false).unwrap().offsets, vec![0, 8]);
        assert_eq!(dl64.stack_align, 16);
        let arr = t.intern(Type::Array { len: 10, elem: s });
        assert_eq!(dl.alloc_size(&t, arr), Some(80));
        let i7 = t.int(7);
        assert_eq!((dl.store_size(&t, i7), dl.alloc_size(&t, i7)), (Some(1), Some(1)));
        let i65 = t.int(65);
        assert_eq!((dl.store_size(&t, i65), dl.abi_align(&t, i65)), (Some(9), Some(4)));
        let v = t.intern(Type::Vector { scalable: false, len: 4, elem: i32 });
        assert_eq!((dl.alloc_size(&t, v), dl.abi_align(&t, v)), (Some(16), Some(16)));
    }

    #[test]
    fn pointer_specs() {
        let dl = DataLayout::parse("e-p:32:32-p1:64:64:64:32-A5").unwrap();
        assert_eq!(dl.pointer(0).size_bits, 32);
        assert_eq!(dl.pointer(1).index_bits, 32);
        assert_eq!(dl.pointer(7).size_bits, 32);
        assert_eq!(dl.alloca_addrspace, 5);
    }
}
