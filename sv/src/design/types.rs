//! Data types (clause 6, 7), interned: equal types have equal ids.
//!
//! Packed types are kept with their structure (atoms, packed dimensions,
//! packed structs and enums) so that selects and members can be turned
//! into bit offsets, and so that type equivalence (6.22) can be checked.
//! A struct, union or enum declaration is a new type each time it is
//! elaborated, which its `uid` records.

use super::{ClassId, SpecId, Sym};
use std::collections::HashMap;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, PartialOrd, Ord)]
pub struct TypeId(pub u32);

/// The built-in integral types (6.11) and the single-bit ones (6.3).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Atom {
    Bit,
    Logic,
    Reg,
    Byte,
    ShortInt,
    Int,
    LongInt,
    Integer,
    Time,
}

impl Atom {
    pub fn width(self) -> u32 {
        match self {
            Atom::Bit | Atom::Logic | Atom::Reg => 1,
            Atom::Byte => 8,
            Atom::ShortInt => 16,
            Atom::Int | Atom::Integer => 32,
            Atom::LongInt | Atom::Time => 64,
        }
    }

    pub fn four_state(self) -> bool {
        matches!(self, Atom::Logic | Atom::Reg | Atom::Integer | Atom::Time)
    }

    pub fn default_signed(self) -> bool {
        matches!(self, Atom::Byte | Atom::ShortInt | Atom::Int | Atom::LongInt | Atom::Integer)
    }

    /// Whether the atom is one bit and so may take packed dimensions.
    pub fn is_scalar(self) -> bool {
        matches!(self, Atom::Bit | Atom::Logic | Atom::Reg)
    }

    pub fn name(self) -> &'static str {
        match self {
            Atom::Bit => "bit",
            Atom::Logic => "logic",
            Atom::Reg => "reg",
            Atom::Byte => "byte",
            Atom::ShortInt => "shortint",
            Atom::Int => "int",
            Atom::LongInt => "longint",
            Atom::Integer => "integer",
            Atom::Time => "time",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum RealKind {
    Real,
    ShortReal,
    RealTime,
}

/// An unpacked dimension (7.4).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum UDim {
    /// `[l:r]`, or `[n]` as `[0:n-1]`.
    Range(i64, i64),
    /// `[]`
    Dynamic,
    /// `[$]` or `[$:n]`
    Queue(Option<i64>),
    /// `[*]`
    Wildcard,
    /// `[T]`
    Assoc(TypeId),
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Member {
    pub name: Sym,
    pub ty: TypeId,
    pub rand: bool,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Type {
    /// A type that could not be worked out: something not yet elaborated.
    /// Counted, so that it never hides silently.
    Unknown,
    Void,
    /// An atom with packed dimensions, outermost first: `logic [7:0]`.
    Vector {
        atom: Atom,
        signed: bool,
        dims: Vec<(i64, i64)>,
    },
    Real(RealKind),
    String,
    Chandle,
    Event,
    /// The type of `null`.
    Null,
    /// A packed array of a packed struct, union or enum: `s_t [3:0]`.
    PackedArray {
        elem: TypeId,
        dim: (i64, i64),
        signed: bool,
    },
    Struct {
        packed: bool,
        union: bool,
        tagged: bool,
        soft: bool,
        signed: bool,
        members: Vec<Member>,
        name: Option<Sym>,
        uid: u32,
    },
    Enum {
        base: TypeId,
        items: Vec<(Sym, super::value::Bits)>,
        name: Option<Sym>,
        uid: u32,
    },
    Unpacked {
        elem: TypeId,
        dim: UDim,
    },
    Class(ClassId),
    /// An interface instance's type, as a port or `virtual` type: the
    /// interface specialisation (or `None` for a generic `interface`
    /// port) and the modport.
    Interface {
        spec: Option<SpecId>,
        def: Sym,
        modport: Option<Sym>,
        virtual_: bool,
    },
    /// The type of a type: `type(int)` used as a value.
    TypeOfType,
}

pub struct Types {
    pub list: Vec<Type>,
    map: HashMap<Type, TypeId>,
    next_uid: u32,
}

impl Default for Types {
    fn default() -> Self {
        Self::new()
    }
}

impl Types {
    pub const UNKNOWN: TypeId = TypeId(0);
    pub const VOID: TypeId = TypeId(1);
    pub const BIT: TypeId = TypeId(2);
    pub const LOGIC: TypeId = TypeId(3);
    pub const INT: TypeId = TypeId(4);
    pub const INTEGER: TypeId = TypeId(5);
    pub const REAL: TypeId = TypeId(6);
    pub const STRING: TypeId = TypeId(7);
    pub const NULL: TypeId = TypeId(8);
    pub const EVENT: TypeId = TypeId(9);
    pub const TIME: TypeId = TypeId(10);
    pub const LONGINT: TypeId = TypeId(11);
    pub const TYPE: TypeId = TypeId(12);
    pub const CHANDLE: TypeId = TypeId(13);
    pub const REALTIME: TypeId = TypeId(14);
    pub const SHORTREAL: TypeId = TypeId(15);

    pub fn new() -> Types {
        let mut t = Types { list: Vec::new(), map: HashMap::new(), next_uid: 0 };
        let atom = |a: Atom| Type::Vector { atom: a, signed: a.default_signed(), dims: vec![] };
        for ty in [
            Type::Unknown,
            Type::Void,
            atom(Atom::Bit),
            atom(Atom::Logic),
            atom(Atom::Int),
            atom(Atom::Integer),
            Type::Real(RealKind::Real),
            Type::String,
            Type::Null,
            Type::Event,
            atom(Atom::Time),
            atom(Atom::LongInt),
            Type::TypeOfType,
            Type::Chandle,
            Type::Real(RealKind::RealTime),
            Type::Real(RealKind::ShortReal),
        ] {
            t.intern(ty);
        }
        t
    }

    pub fn intern(&mut self, ty: Type) -> TypeId {
        if let Some(&id) = self.map.get(&ty) {
            return id;
        }
        let id = TypeId(self.list.len() as u32);
        self.list.push(ty.clone());
        self.map.insert(ty, id);
        id
    }

    pub fn uid(&mut self) -> u32 {
        self.next_uid += 1;
        self.next_uid
    }

    pub fn get(&self, id: TypeId) -> &Type {
        &self.list[id.0 as usize]
    }

    /// A plain vector: `logic [w-1:0]` or `bit [w-1:0]`, signed or not.
    pub fn vector(&mut self, width: u32, signed: bool, four_state: bool) -> TypeId {
        let atom = if four_state { Atom::Logic } else { Atom::Bit };
        let dims = if width == 1 && !signed { vec![] } else { vec![(width as i64 - 1, 0)] };
        self.intern(Type::Vector { atom, signed, dims })
    }

    pub fn is_integral(&self, id: TypeId) -> bool {
        matches!(self.get(id), Type::Vector { .. } | Type::PackedArray { .. } | Type::Enum { .. })
            || matches!(self.get(id), Type::Struct { packed: true, .. })
    }

    pub fn is_real(&self, id: TypeId) -> bool {
        matches!(self.get(id), Type::Real(_))
    }

    pub fn is_unknown(&self, id: TypeId) -> bool {
        matches!(self.get(id), Type::Unknown)
    }

    /// The width in bits of an integral type, or 0.
    pub fn width(&self, id: TypeId) -> u32 {
        match self.get(id) {
            Type::Vector { atom, dims, .. } => {
                dims.iter().fold(atom.width() as u64, |w, &(l, r)| w * ((l - r).unsigned_abs() + 1)) as u32
            }
            Type::PackedArray { elem, dim: (l, r), .. } => self.width(*elem) * ((l - r).unsigned_abs() as u32 + 1),
            Type::Struct { packed: true, union, members, .. } => {
                let ws = members.iter().map(|m| self.width(m.ty));
                if *union { ws.max().unwrap_or(0) } else { ws.sum() }
            }
            Type::Enum { base, .. } => self.width(*base),
            Type::Real(RealKind::ShortReal) => 32,
            Type::Real(_) => 64,
            _ => 0,
        }
    }

    pub fn is_signed(&self, id: TypeId) -> bool {
        match self.get(id) {
            Type::Vector { signed, .. } | Type::PackedArray { signed, .. } | Type::Struct { signed, .. } => *signed,
            Type::Enum { base, .. } => self.is_signed(*base),
            Type::Real(_) => true,
            _ => false,
        }
    }

    pub fn is_four_state(&self, id: TypeId) -> bool {
        match self.get(id) {
            Type::Vector { atom, .. } => atom.four_state(),
            Type::PackedArray { elem, .. } => self.is_four_state(*elem),
            Type::Struct { packed: true, members, .. } => members.iter().any(|m| self.is_four_state(m.ty)),
            Type::Enum { base, .. } => self.is_four_state(*base),
            _ => false,
        }
    }

    /// The type of a self-determined integral value of this width,
    /// signedness and state count, as clause 11 makes intermediate results.
    pub fn integral_like(&mut self, width: u32, signed: bool, four_state: bool) -> TypeId {
        self.vector(width, signed, four_state)
    }

    pub fn is_unpacked_array(&self, id: TypeId) -> bool {
        matches!(self.get(id), Type::Unpacked { .. })
    }

    /// The element type after one index: an unpacked dimension, a packed
    /// dimension, or a bit of a vector.
    pub fn element(&mut self, id: TypeId) -> Option<TypeId> {
        match self.get(id).clone() {
            Type::Unpacked { elem, .. } => Some(elem),
            Type::PackedArray { elem, .. } => Some(elem),
            Type::Vector { atom, dims, .. } => {
                if dims.is_empty() {
                    if atom.is_scalar() {
                        None
                    } else {
                        Some(if atom.four_state() { Types::LOGIC } else { Types::BIT })
                    }
                } else if dims.len() == 1 {
                    let a = if atom.is_scalar() {
                        atom
                    } else if atom.four_state() {
                        Atom::Logic
                    } else {
                        Atom::Bit
                    };
                    Some(self.intern(Type::Vector { atom: a, signed: false, dims: vec![] }))
                } else {
                    Some(self.intern(Type::Vector { atom, signed: false, dims: dims[1..].to_vec() }))
                }
            }
            Type::Struct { packed: true, .. } | Type::Enum { .. } => {
                let w = self.width(id);
                if w > 0 {
                    let fs = self.is_four_state(id);
                    Some(self.vector(1, false, fs))
                } else {
                    None
                }
            }
            Type::String => Some(self.intern(Type::Vector { atom: Atom::Byte, signed: true, dims: vec![] })),
            _ => None,
        }
    }

    /// The width of one element after an index, without making types: a
    /// bit of a vector, a packed element, or 8 for a string's character.
    pub fn elem_width(&self, id: TypeId) -> u32 {
        match self.get(id) {
            Type::Unpacked { elem, .. } | Type::PackedArray { elem, .. } => self.width(*elem),
            // `logic [3:0][7:0]`: an element is 8 bits; `logic [7:0]` or
            // `int`: one bit.
            Type::Vector { atom, dims, .. } => match dims.len() {
                0 => 1,
                1 => atom.width(),
                _ => dims[1..].iter().fold(atom.width() as u64, |w, &(l, r)| w * ((l - r).unsigned_abs() + 1)) as u32,
            },
            Type::String => 8,
            _ => 1,
        }
    }

    /// The element type after an index, if that type already exists.
    pub fn element_of(&self, id: TypeId) -> Option<TypeId> {
        let t = match self.get(id).clone() {
            Type::Unpacked { elem, .. } | Type::PackedArray { elem, .. } => return Some(elem),
            Type::Vector { atom, dims, .. } if dims.len() > 1 => Type::Vector { atom, signed: false, dims: dims[1..].to_vec() },
            Type::Vector { atom, .. } => {
                let a = if atom.is_scalar() { atom } else if atom.four_state() { Atom::Logic } else { Atom::Bit };
                Type::Vector { atom: a, signed: false, dims: vec![] }
            }
            _ => return None,
        };
        self.map.get(&t).copied()
    }

    /// The outermost packed range of an integral type, `[l:r]`.
    pub fn packed_range(&self, id: TypeId) -> Option<(i64, i64)> {
        match self.get(id) {
            Type::Vector { atom, dims, .. } => {
                if let Some(&d) = dims.first() {
                    Some(d)
                } else if !atom.is_scalar() {
                    Some((atom.width() as i64 - 1, 0))
                } else {
                    None
                }
            }
            Type::PackedArray { dim, .. } => Some(*dim),
            Type::Struct { packed: true, .. } | Type::Enum { .. } => {
                let w = self.width(id);
                Some((w as i64 - 1, 0))
            }
            _ => None,
        }
    }

    /// How the type is written: `logic [7:0]`, `int`, `my_t`.
    pub fn describe(&self, id: TypeId, syms: &super::Syms) -> String {
        match self.get(id) {
            Type::Unknown => "?".into(),
            Type::Void => "void".into(),
            Type::Vector { atom, signed, dims } => {
                let mut s = atom.name().to_string();
                if *signed != atom.default_signed() {
                    s += if *signed { " signed" } else { " unsigned" };
                }
                for (l, r) in dims {
                    s += &format!("[{l}:{r}]");
                }
                s
            }
            Type::Real(RealKind::Real) => "real".into(),
            Type::Real(RealKind::ShortReal) => "shortreal".into(),
            Type::Real(RealKind::RealTime) => "realtime".into(),
            Type::String => "string".into(),
            Type::Chandle => "chandle".into(),
            Type::Event => "event".into(),
            Type::Null => "null".into(),
            Type::TypeOfType => "type".into(),
            Type::PackedArray { elem, dim: (l, r), signed } => {
                format!("{}{}[{l}:{r}]", self.describe(*elem, syms), if *signed { " signed" } else { "" })
            }
            Type::Struct { name: Some(n), .. } | Type::Enum { name: Some(n), .. } => syms.get(*n).to_string(),
            Type::Struct { packed, union, members, .. } => {
                let mut s = if *union { "union" } else { "struct" }.to_string();
                if *packed {
                    s += " packed";
                }
                s += " {";
                for m in members {
                    s += &format!(" {} {};", self.describe(m.ty, syms), syms.get(m.name));
                }
                s + " }"
            }
            Type::Enum { base, items, .. } => {
                let names: Vec<&str> = items.iter().map(|(n, _)| syms.get(*n)).collect();
                format!("enum {} {{{}}}", self.describe(*base, syms), names.join(", "))
            }
            Type::Unpacked { elem, dim } => {
                let d = match dim {
                    UDim::Range(l, r) => format!("[{l}:{r}]"),
                    UDim::Dynamic => "[]".into(),
                    UDim::Queue(None) => "[$]".into(),
                    UDim::Queue(Some(n)) => format!("[$:{n}]"),
                    UDim::Wildcard => "[*]".into(),
                    UDim::Assoc(t) => format!("[{}]", self.describe(*t, syms)),
                };
                // Unpacked dimensions are written after the name; here, after the type.
                format!("{} $ {d}", self.describe(*elem, syms)).replace(" $ ", " ")
            }
            Type::Class(c) => format!("class#{}", c.0),
            Type::Interface { def, modport, virtual_, .. } => {
                let mut s = if *virtual_ { "virtual " } else { "" }.to_string() + syms.get(*def);
                if let Some(m) = modport {
                    s += ".";
                    s += syms.get(*m);
                }
                s
            }
        }
    }
}
