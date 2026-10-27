//! Data types from syntax (clauses 6, 7), typedefs, nettypes and classes.

use super::{Elab, R, eval, lookup};
use crate::ast::{Kind, NodeId};
use crate::design::types::{Atom, Member, RealKind, Type, TypeId, Types, UDim};
use crate::design::value::Bits;
use crate::design::*;

pub(crate) const NET_KEYWORDS: &[&str] =
    &["wire", "tri", "tri0", "tri1", "wand", "triand", "wor", "trior", "trireg", "uwire", "supply0", "supply1"];

pub(crate) fn is_dim(k: Kind) -> bool {
    matches!(k, Kind::RangeDim | Kind::SizeDim | Kind::UnsizedDim | Kind::AssocDim | Kind::QueueDim)
}

pub(crate) fn is_type(k: Kind) -> bool {
    matches!(
        k,
        Kind::BuiltinType
            | Kind::ImplicitType
            | Kind::NamedType
            | Kind::StructType
            | Kind::EnumType
            | Kind::VirtualInterfaceType
            | Kind::TypeOf
            | Kind::VoidType
    )
}

/// Whether an implicit type has no packed range (`parameter signed p`).
pub(crate) fn implicit_has_no_range(e: &Elab, t: NodeId) -> bool {
    !e.kids(t).iter().any(|&k| is_dim(e.kind(k)))
}

fn atom_of(s: &str) -> Option<Atom> {
    Some(match s {
        "bit" => Atom::Bit,
        "logic" => Atom::Logic,
        "reg" => Atom::Reg,
        "byte" => Atom::Byte,
        "shortint" => Atom::ShortInt,
        "int" => Atom::Int,
        "longint" => Atom::LongInt,
        "integer" => Atom::Integer,
        "time" => Atom::Time,
        _ => return None,
    })
}

/// A constant `[l:r]` (or `[n]`, which is `[0:n-1]`).
pub(crate) fn const_range<'a>(e: &mut Elab<'a, '_>, sc: ScopeId, d: NodeId) -> R<'a, (i64, i64)> {
    let k = e.kids(d).to_vec();
    match e.kind(d) {
        Kind::RangeDim => {
            let l = const_int(e, sc, k[0])?;
            let r = const_int(e, sc, k[1])?;
            Ok((l, r))
        }
        Kind::SizeDim => {
            let n = const_int(e, sc, k[0])?;
            if n <= 0 {
                return Err(e.err(k[0], format!("an array's size must be positive, not {n}")));
            }
            Ok((0, n - 1))
        }
        _ => Err(e.err(d, "this dimension needs a constant range")),
    }
}

pub(crate) fn const_int<'a>(e: &mut Elab<'a, '_>, sc: ScopeId, n: NodeId) -> R<'a, i64> {
    let v = eval::const_expr(e, sc, n)?;
    match v.to_i64() {
        Some(i) if v.as_bits().is_none_or(|b| b.is_known()) => Ok(i),
        _ => Err(e.err(n, "a dimension's bounds must be known constant integers")),
    }
}

/// The data type a type node describes.
pub(crate) fn data_type<'a>(e: &mut Elab<'a, '_>, sc: ScopeId, n: NodeId) -> R<'a, TypeId> {
    let kids = e.kids(n).to_vec();
    match e.kind(n) {
        Kind::BuiltinType => {
            let kw = e.text(kids[0]);
            let signing = kids.get(1).filter(|&&k| e.kind(k) == Kind::Keyword).map(|&k| e.text(k));
            let dims: Vec<NodeId> = kids.iter().copied().filter(|&k| is_dim(e.kind(k))).collect();
            if let Some(a) = atom_of(kw) {
                if !a.is_scalar() && !dims.is_empty() {
                    return Err(e.err(dims[0], format!("`{kw}` cannot be given a packed dimension (6.11)")));
                }
                let signed = match signing {
                    Some("signed") => true,
                    Some("unsigned") => false,
                    _ => a.default_signed(),
                };
                let packed = packed_dims(e, sc, &dims)?;
                return Ok(e.d.types.intern(Type::Vector { atom: a, signed, dims: packed }));
            }
            if !dims.is_empty() {
                return Err(e.err(dims[0], format!("`{kw}` cannot be given a packed dimension")));
            }
            Ok(match kw {
                "real" => Types::REAL,
                "shortreal" => Types::SHORTREAL,
                "realtime" => Types::REALTIME,
                "string" => Types::STRING,
                "chandle" => Types::CHANDLE,
                "event" => Types::EVENT,
                "void" => Types::VOID,
                _ => Types::UNKNOWN,
            })
        }
        Kind::VoidType => Ok(Types::VOID),
        Kind::ImplicitType => {
            let signed = kids.iter().any(|&k| e.kind(k) == Kind::Keyword && e.text(k) == "signed");
            let dims: Vec<NodeId> = kids.iter().copied().filter(|&k| is_dim(e.kind(k))).collect();
            let packed = packed_dims(e, sc, &dims)?;
            Ok(e.d.types.intern(Type::Vector { atom: Atom::Logic, signed, dims: packed }))
        }
        Kind::NamedType => {
            let base = named(e, sc, kids[0])?;
            let dims: Vec<NodeId> = kids.iter().copied().filter(|&k| is_dim(e.kind(k))).collect();
            if dims.is_empty() {
                return Ok(base);
            }
            packed_of(e, sc, base, &dims)
        }
        Kind::StructType => struct_type(e, sc, n),
        Kind::EnumType => enum_type(e, sc, n),
        Kind::VirtualInterfaceType => {
            let iname = e.sym_of(kids[0]);
            let mp = kids.iter().skip(1).copied().find(|&k| e.kind(k) == Kind::Name).map(|k| e.sym_of(k));
            if !e.defs.get(&iname).is_some_and(|d| d.kind == super::DefKind2::Design(DefKind::Interface)) {
                return Err(e.err(kids[0], format!("`{}` is not an interface", e.name(iname))));
            }
            Ok(e.d.types.intern(Type::Interface { spec: None, def: iname, modport: mp, virtual_: true }))
        }
        Kind::TypeOf => {
            let inner = kids[0];
            if is_type(e.kind(inner)) {
                data_type(e, sc, inner)
            } else if e.kind(inner) == Kind::This {
                Ok(lookup::enclosing(e, sc, ScopeKind::Class)
                    .and_then(|c| lookup::class_of_scope(e, c))
                    .map_or(Types::UNKNOWN, |c| e.d.types.intern(Type::Class(c))))
            } else {
                if let Some(t) = maybe_type(e, sc, inner)? {
                    return Ok(t);
                }
                let x = super::expr::expr(e, sc, inner)?;
                Ok(e.ex(x).ty)
            }
        }
        _ => Err(e.err(n, "expected a data type")),
    }
}

/// `int [3:0]` is not allowed; `my_struct [3:0]` is a packed array of an
/// integral type.
fn packed_of<'a>(e: &mut Elab<'a, '_>, sc: ScopeId, base: TypeId, dims: &[NodeId]) -> R<'a, TypeId> {
    let packed = packed_dims(e, sc, dims)?;
    match e.d.types.get(base).clone() {
        Type::Vector { atom, signed, dims: inner } => {
            if !atom.is_scalar() {
                return Err(e.err(dims[0], format!("`{}` cannot be given a packed dimension (6.11)", atom.name())));
            }
            let mut all = packed;
            all.extend(inner);
            Ok(e.d.types.intern(Type::Vector { atom, signed, dims: all }))
        }
        Type::Struct { packed: true, .. } | Type::Enum { .. } | Type::PackedArray { .. } => {
            let mut t = base;
            for &d in packed.iter().rev() {
                t = e.d.types.intern(Type::PackedArray { elem: t, dim: d, signed: false });
            }
            Ok(t)
        }
        Type::Unknown => Ok(Types::UNKNOWN),
        _ => Err(e.err(dims[0], "packed dimensions need an integral type")),
    }
}

fn packed_dims<'a>(e: &mut Elab<'a, '_>, sc: ScopeId, dims: &[NodeId]) -> R<'a, Vec<(i64, i64)>> {
    let mut out = Vec::new();
    for &d in dims {
        match e.kind(d) {
            Kind::RangeDim => out.push(const_range(e, sc, d)?),
            Kind::UnsizedDim => out.push((0, 0)),
            _ => return Err(e.err(d, "a packed dimension must be a range, `[msb:lsb]` (7.4.1)")),
        }
    }
    Ok(out)
}

/// A type named by `T`, `p::T`, `$unit::T` or a class specialisation.
fn named<'a>(e: &mut Elab<'a, '_>, sc: ScopeId, n: NodeId) -> R<'a, TypeId> {
    match e.kind(n) {
        Kind::Name => {
            let s = e.sym_of(n);
            match lookup::resolve(e, sc, s, Some(n))? {
                Some(Binding::Type(t)) => Ok(t),
                Some(Binding::Class(c)) => {
                    super::checks::class_used_as_type(e, sc, c, n)?;
                    Ok(e.d.types.intern(Type::Class(c)))
                }
                Some(Binding::Other(m)) if e.kind(m) == Kind::ForwardTypedef => Ok(Types::UNKNOWN),
                Some(Binding::Other(m)) if e.kind(m) == Kind::Covergroup => Ok(Types::UNKNOWN),
                Some(Binding::Other(_)) if e.std_pkg.is_some_and(|p| e.scope(p).names.contains_key(&s)) => {
                    Ok(Types::UNKNOWN)
                }
                Some(b) => Err(e.err(n, format!("`{}` is {}, not a type", e.name(s), e.describe_binding(b)))),
                None => {
                    if let Some(d) = e.defs.get(&s) {
                        if d.kind == super::DefKind2::Design(DefKind::Interface) {
                            return Ok(e.d.types.intern(Type::Interface {
                                spec: None,
                                def: s,
                                modport: None,
                                virtual_: false,
                            }));
                        }
                        return Err(e.err(n, format!("`{}` is a design element, not a type", e.name(s))));
                    }
                    Err(e.err(n, format!("`{}` is not declared", e.name(s))))
                }
            }
        }
        Kind::Scope => {
            let k = e.kids(n).to_vec();
            let s = e.sym_of(k[1]);
            let inner = scope_target(e, sc, k[0])?;
            match inner {
                ScopeTarget::Package(p) => match lookup::in_package(e, p, s) {
                    Some(Binding::Type(t)) => Ok(t),
                    Some(Binding::Class(c)) => Ok(e.d.types.intern(Type::Class(c))),
                    Some(Binding::Other(_)) => Ok(Types::UNKNOWN),
                    _ => Err(e.err(k[1], format!("there is no type `{}` there", e.name(s)))),
                },
                ScopeTarget::Class(c) => match lookup::class_member(e, c, s) {
                    Some(Binding::Type(t)) => Ok(t),
                    Some(Binding::Class(c2)) => Ok(e.d.types.intern(Type::Class(c2))),
                    _ => Ok(Types::UNKNOWN),
                },
                ScopeTarget::Unit => match e.scope(e.d.unit).names.get(&s).copied() {
                    Some(Binding::Type(t)) => Ok(t),
                    Some(Binding::Class(c)) => Ok(e.d.types.intern(Type::Class(c))),
                    _ => Err(e.err(k[1], format!("`$unit` has no type `{}`", e.name(s)))),
                },
                ScopeTarget::Unknown => Ok(Types::UNKNOWN),
            }
        }
        Kind::Specialize => {
            let k = e.kids(n).to_vec();
            named(e, sc, k[0])
        }
        // A type from an interface port, `port.T` (25.10): not modelled.
        Kind::Member => Ok(Types::UNKNOWN),
        _ => Err(e.err(n, "expected a type name")),
    }
}

pub(crate) enum ScopeTarget {
    Package(ScopeId),
    Class(ClassId),
    Unit,
    Unknown,
}

/// What the left of `::` names.
pub(crate) fn scope_target<'a>(e: &mut Elab<'a, '_>, sc: ScopeId, n: NodeId) -> R<'a, ScopeTarget> {
    match e.kind(n) {
        Kind::ScopeKeyword => {
            if e.text(n) == "$unit" {
                Ok(ScopeTarget::Unit)
            } else {
                Ok(ScopeTarget::Unknown)
            }
        }
        Kind::Name => {
            let s = e.sym_of(n);
            // A class (or typedef of one) in scope takes precedence over a
            // package of the same name (26.3).
            match lookup::lookup(e, sc, s) {
                Some(Binding::Class(c)) => return Ok(ScopeTarget::Class(c)),
                Some(Binding::Type(t)) => {
                    if let Type::Class(c) = e.d.types.get(t) {
                        return Ok(ScopeTarget::Class(*c));
                    }
                    return Ok(ScopeTarget::Unknown);
                }
                _ => {}
            }
            if let Some(&p) = e.packages.get(&s) {
                return Ok(ScopeTarget::Package(p));
            }
            // Built-in classes of `std`: `process::self()`.
            if e.std_pkg.is_some_and(|p| e.scope(p).names.contains_key(&s)) {
                return Ok(ScopeTarget::Unknown);
            }
            Err(e.err(n, format!("`{}` is not a package or class", e.name(s))))
        }
        Kind::Specialize => {
            let k = e.kids(n).to_vec();
            scope_target(e, sc, k[0])
        }
        Kind::Scope => {
            let k = e.kids(n).to_vec();
            let outer = scope_target(e, sc, k[0])?;
            let s = e.sym_of(k[1]);
            match outer {
                ScopeTarget::Package(p) => match lookup::in_package(e, p, s) {
                    Some(Binding::Class(c)) => Ok(ScopeTarget::Class(c)),
                    _ => Ok(ScopeTarget::Unknown),
                },
                ScopeTarget::Class(c) => match lookup::class_member(e, c, s) {
                    Some(Binding::Class(c2)) => Ok(ScopeTarget::Class(c2)),
                    _ => Ok(ScopeTarget::Unknown),
                },
                _ => Ok(ScopeTarget::Unknown),
            }
        }
        _ => Ok(ScopeTarget::Unknown),
    }
}

/// If `n`, written where an expression goes, names a type, that type.
pub(crate) fn maybe_type<'a>(e: &mut Elab<'a, '_>, sc: ScopeId, n: NodeId) -> R<'a, Option<TypeId>> {
    match e.kind(n) {
        k if is_type(k) => Ok(Some(data_type(e, sc, n)?)),
        Kind::TypeExpr => Ok(Some(data_type(e, sc, e.kids(n)[0])?)),
        Kind::Name => {
            let s = e.sym_of(n);
            Ok(match lookup::lookup(e, sc, s) {
                Some(Binding::Type(t)) => Some(t),
                Some(Binding::Class(c)) => Some(e.d.types.intern(Type::Class(c))),
                _ => None,
            })
        }
        Kind::Scope => {
            let k = e.kids(n).to_vec();
            let s = e.sym_of(k[1]);
            match scope_target(e, sc, k[0]) {
                Ok(ScopeTarget::Package(p)) => Ok(match lookup::in_package(e, p, s) {
                    Some(Binding::Type(t)) => Some(t),
                    Some(Binding::Class(c)) => Some(e.d.types.intern(Type::Class(c))),
                    _ => None,
                }),
                Ok(ScopeTarget::Class(c)) => Ok(match lookup::class_member(e, c, s) {
                    Some(Binding::Type(t)) => Some(t),
                    _ => None,
                }),
                _ => Ok(None),
            }
        }
        _ => Ok(None),
    }
}

/// The type of a declared name with unpacked dimensions after it.
pub(crate) fn unpacked<'a>(e: &mut Elab<'a, '_>, sc: ScopeId, base: TypeId, dims: &[NodeId]) -> R<'a, TypeId> {
    let mut t = base;
    for &d in dims.iter().rev() {
        let dim = match e.kind(d) {
            Kind::RangeDim => {
                let (l, r) = const_range(e, sc, d)?;
                UDim::Range(l, r)
            }
            Kind::SizeDim => {
                let k = e.kids(d)[0];
                // `[T]` with `T` a type: an associative array.
                if let Some(it) = maybe_type(e, sc, k)? {
                    super::checks::assoc_index(e, it, k)?;
                    UDim::Assoc(it)
                } else {
                    let (l, r) = const_range(e, sc, d)?;
                    UDim::Range(l, r)
                }
            }
            Kind::UnsizedDim => UDim::Dynamic,
            Kind::QueueDim => match e.kids(d).first() {
                Some(&b) => {
                    let n = const_int(e, sc, b)?;
                    if n <= 0 {
                        return Err(e.err(b, format!("a queue's bound must be positive, not {n} (7.10)")));
                    }
                    UDim::Queue(Some(n))
                }
                None => UDim::Queue(None),
            },
            Kind::AssocDim => match e.kids(d).first() {
                Some(&it) => {
                    let t = data_type(e, sc, it)?;
                    super::checks::assoc_index(e, t, it)?;
                    UDim::Assoc(t)
                }
                None => UDim::Wildcard,
            },
            _ => continue,
        };
        t = e.d.types.intern(Type::Unpacked { elem: t, dim });
    }
    Ok(t)
}

fn struct_type<'a>(e: &mut Elab<'a, '_>, sc: ScopeId, n: NodeId) -> R<'a, TypeId> {
    let kids = e.kids(n).to_vec();
    let kws = e.keywords_of(n);
    let union = kws.first() == Some(&"union");
    let soft_kw = kws.contains(&"soft");
    let packed = kws.contains(&"packed") || soft_kw;
    let tagged = kws.contains(&"tagged");
    let soft = kws.contains(&"soft");
    let signed = kws.contains(&"signed");
    let mut members = Vec::new();
    for m in kids.iter().copied().filter(|&k| e.kind(k) == Kind::StructMember).collect::<Vec<_>>() {
        let mk = e.kids(m).to_vec();
        let rand = mk.iter().any(|&k| e.kind(k) == Kind::Keyword && e.text(k).starts_with("rand"));
        let tn = *mk.iter().find(|&&k| is_type(e.kind(k))).unwrap();
        let base = data_type(e, sc, tn)?;
        for d in mk.iter().copied().filter(|&k| e.kind(k) == Kind::Declarator).collect::<Vec<_>>() {
            let dk = e.kids(d).to_vec();
            let name = e.sym_of(dk[0]);
            let dims: Vec<NodeId> = dk[1..].iter().copied().filter(|&k| is_dim(e.kind(k))).collect();
            let default = dk[1..].iter().copied().find(|&k| !is_dim(e.kind(k)));
            let t = unpacked(e, sc, base, &dims)?;
            if members.iter().any(|x: &Member| x.name == name) {
                return Err(e.err(dk[0], format!("member `{}` is declared twice", e.name(name))));
            }
            super::checks::struct_member(e, sc, n, union, packed, tagged, t, dk[0], default)?;
            members.push(Member { name, ty: t, rand });
        }
    }
    super::checks::struct_whole(e, n, union, packed, soft, tagged, &members)?;
    let uid = e.d.types.uid();
    let base = e.d.types.intern(Type::Struct { packed, union, tagged, soft, signed, members, name: None, uid });
    let dims: Vec<NodeId> = kids.iter().copied().filter(|&k| is_dim(e.kind(k))).collect();
    if dims.is_empty() { Ok(base) } else { packed_of(e, sc, base, &dims) }
}

fn enum_type<'a>(e: &mut Elab<'a, '_>, sc: ScopeId, n: NodeId) -> R<'a, TypeId> {
    let kids = e.kids(n).to_vec();
    let base = match kids.first() {
        Some(&b) if is_type(e.kind(b)) => data_type(e, sc, b)?,
        _ => Types::INT,
    };
    if !e.d.types.is_integral(base) {
        return Err(e.err(kids[0], "an enum's base type must be integral (6.19)"));
    }
    let width = e.d.types.width(base);
    let signed = e.d.types.is_signed(base);
    let four = e.d.types.is_four_state(base);
    let mut items: Vec<(Sym, Bits)> = Vec::new();
    let mut item_nodes = Vec::new();
    let mut next = Bits::zero(width, signed);
    let mut next_known = true;
    for it in kids.iter().copied().filter(|&k| e.kind(k) == Kind::EnumItem).collect::<Vec<_>>() {
        let ik = e.kids(it).to_vec();
        let name = e.sym_of(ik[0]);
        let toks = e.ast.tokens_of(it);
        let eq = toks.iter().position(|t| t.0 == "=");
        let first = e.ast.node(it).first as usize;
        let mut range: Vec<NodeId> = Vec::new();
        let mut value = None;
        for &x in &ik[1..] {
            let at = e.ast.node(x).first as usize - first;
            if eq.is_some_and(|q| at > q) { value = Some(x) } else { range.push(x) }
        }
        let start = match value {
            Some(v) => {
                let c = eval::const_expr(e, sc, v)?;
                let b = match c {
                    Const::Bits(b) => b,
                    Const::Real(r) => Bits::from_f64(width, signed, r),
                    _ => return Err(e.err(v, "an enum value must be integral")),
                };
                if !four && !b.is_known() {
                    return Err(e.err(v, "a two-state enum cannot have x or z in a value (6.19)"));
                }
                super::checks::enum_value_fits(e, v, &b, width, signed)?;
                b.resize(width, b.signed()).with_signed(signed)
            }
            None => {
                if !next_known {
                    return Err(e.err(ik[0], "an enum value after one with x or z must be given explicitly (6.19)"));
                }
                next.clone()
            }
        };
        let names: Vec<(Sym, Bits)> = match range.len() {
            0 => vec![(name, start.clone())],
            _ => {
                let a = super::ty::const_int(e, sc, range[0])?;
                let (lo, hi) = if range.len() == 2 { (a, super::ty::const_int(e, sc, range[1])?) } else { (0, a - 1) };
                let base_name = e.name(name).to_string();
                let mut v = start.clone();
                let mut out = Vec::new();
                let step: i64 = if hi >= lo { 1 } else { -1 };
                let mut i = lo;
                loop {
                    let s = e.sym_owned(format!("{base_name}{i}"));
                    out.push((s, v.clone()));
                    v = v.add(&Bits::from_u64(width, signed, 1)).resize(width, false).with_signed(signed);
                    if i == hi {
                        break;
                    }
                    i += step;
                }
                out
            }
        };
        for (s, v) in names {
            if let Some((other, _)) = items.iter().find(|(_, x)| x.case_eq(&v)) {
                return Err(e.err(
                    it,
                    format!("enum values `{}` and `{}` are both {} (6.19)", e.name(*other), e.name(s), v.to_decimal()),
                ));
            }
            next_known = v.is_known();
            next = v.add(&Bits::from_u64(width, signed, 1)).resize(width, false).with_signed(signed);
            if next_known && !v.is_zero() && next.is_zero() && !signed {
                // Wrapped: the next implicit value would not fit.
                next_known = true;
            }
            items.push((s, v));
            item_nodes.push(it);
        }
    }
    let uid = e.d.types.uid();
    let t = e.d.types.intern(Type::Enum { base, items: items.clone(), name: None, uid });
    // The names are constants in the enclosing scope (6.19).
    for ((s, v), node) in items.into_iter().zip(item_nodes) {
        let c = e.new_const(Const::Bits(v));
        let o = e.new_object(Object {
            name: s,
            kind: ObjKind::EnumConst,
            ty: t,
            scope: sc,
            node,
            lifetime: Lifetime::Static,
            dir: None,
            init: None,
            value: Some(c),
            qualifiers: vec![],
        });
        e.declare(sc, s, Binding::Object(o), node)?;
    }
    let dims: Vec<NodeId> = kids.iter().copied().filter(|&k| is_dim(e.kind(k))).collect();
    if dims.is_empty() { Ok(t) } else { packed_of(e, sc, t, &dims) }
}

/// `typedef T name [dims];`
pub(crate) fn typedef<'a>(e: &mut Elab<'a, '_>, sc: ScopeId, n: NodeId) -> R<'a> {
    let kids = e.kids(n).to_vec();
    let name_node = *kids.iter().find(|&&k| e.kind(k) == Kind::Name).unwrap();
    let s = e.sym_of(name_node);
    let tn = kids[0];
    let base = data_type(e, sc, tn)?;
    let dims: Vec<NodeId> = kids.iter().copied().filter(|&k| is_dim(e.kind(k))).collect();
    let mut t = unpacked(e, sc, base, &dims)?;
    // Name a struct, union or enum after its first typedef.
    if dims.is_empty() {
        let named = match e.d.types.get(t).clone() {
            Type::Struct { packed, union, tagged, soft, signed, members, name: None, uid } => {
                Some(Type::Struct { packed, union, tagged, soft, signed, members, name: Some(s), uid })
            }
            Type::Enum { base, items, name: None, uid } => Some(Type::Enum { base, items, name: Some(s), uid }),
            _ => None,
        };
        if let Some(nt) = named {
            let old = t;
            t = e.d.types.intern(nt);
            // The enum's constants have the named type.
            for o in e.d.objects.iter_mut() {
                if o.ty == old && o.kind == ObjKind::EnumConst {
                    o.ty = t;
                }
            }
        }
    }
    match e.scope(sc).names.get(&s).copied() {
        Some(Binding::Other(m)) if e.kind(m) == Kind::ForwardTypedef => {
            e.scope_mut(sc).names.insert(s, Binding::Type(t));
        }
        _ => e.declare(sc, s, Binding::Type(t), name_node)?,
    }
    e.push_item(sc, Item::Typedef(s, t));
    Ok(())
}

/// `nettype T name [with f];` (6.6.7): a named data type for nets.
pub(crate) fn nettype<'a>(e: &mut Elab<'a, '_>, sc: ScopeId, n: NodeId) -> R<'a> {
    let kids = e.kids(n).to_vec();
    let t = data_type(e, sc, kids[0])?;
    let name_node = kids[1];
    let s = e.sym_of(name_node);
    super::checks::nettype_data(e, t, kids[0])?;
    e.nettypes.insert(s);
    e.declare(sc, s, Binding::Type(t), name_node)?;
    e.push_item(sc, Item::Typedef(s, t));
    Ok(())
}

// ---- classes (8) ----

pub(crate) fn declare_class<'a>(e: &mut Elab<'a, '_>, sc: ScopeId, n: NodeId) -> R<'a> {
    let Some(name) = e.name_child(n) else { return Ok(()) };
    let s = e.sym_of(name);
    if let Some(Binding::Class(_)) = e.scope(sc).names.get(&s) {
        return Ok(());
    }
    let cs = e.new_scope(ScopeKind::Class, Some(s), Some(sc), n);
    let id = ClassId(e.d.classes.len() as u32);
    let interface = e.kind(n) == Kind::InterfaceClass;
    let virtual_ = e.has_keyword(n, "virtual");
    e.d.classes.push(Class { name: s, scope: cs, base: None, node: n, virtual_, interface, params: vec![] });
    e.class_nodes.insert(id, (n, sc));
    match e.scope(sc).names.get(&s).copied() {
        Some(Binding::Other(m)) if e.kind(m) == Kind::ForwardTypedef => {
            e.scope_mut(sc).names.insert(s, Binding::Class(id));
        }
        _ => e.declare(sc, s, Binding::Class(id), name)?,
    }
    Ok(())
}

pub(crate) fn class_body<'a>(e: &mut Elab<'a, '_>, sc: ScopeId, n: NodeId) -> R<'a> {
    let Some(name) = e.name_child(n) else { return Ok(()) };
    let s = e.sym_of(name);
    let Some(Binding::Class(id)) = e.scope(sc).names.get(&s).copied() else {
        declare_class(e, sc, n)?;
        return class_body(e, sc, n);
    };
    if e.d.classes[id.0 as usize].node != n {
        return Ok(());
    }
    let cs = e.d.classes[id.0 as usize].scope;
    if !e.scope(cs).items.is_empty() {
        return Ok(());
    }
    e.push_item(sc, Item::Class(id));
    let kids = e.kids(n).to_vec();
    // Parameters, with their defaults: a parameterised class is elaborated
    // once, generically (8.25).
    if let Some(pp) = kids.iter().copied().find(|&k| e.kind(k) == Kind::ParamPorts) {
        let mut ty_node = None;
        for &p in e.kids(pp).to_vec().iter() {
            match e.kind(p) {
                Kind::ParamDecl => {
                    let pk = e.kids(p).to_vec();
                    ty_node = pk.get(1).copied().filter(|&k| e.kind(k) != Kind::ParamAssign);
                    for &a in &pk {
                        if e.kind(a) == Kind::ParamAssign {
                            class_param(e, cs, a, ty_node)?;
                        }
                    }
                }
                Kind::TypeParamDecl => e.type_param_decl(cs, p, None)?,
                Kind::ParamAssign => class_param(e, cs, p, ty_node)?,
                _ => {}
            }
        }
    }
    for &k in &kids {
        if e.kind(k) == Kind::Extends {
            let ek = e.kids(k).to_vec();
            let t = data_type(e, sc, ek[0])?;
            match e.d.types.get(t).clone() {
                Type::Class(b) => {
                    if b == id {
                        return Err(e.err(ek[0], "a class cannot extend itself"));
                    }
                    super::checks::extends(e, id, b, ek[0])?;
                    e.d.classes[id.0 as usize].base = Some(b);
                }
                Type::Unknown => {}
                _ => return Err(e.err(ek[0], "a class can only extend a class (8.13)")),
            }
        }
        if e.kind(k) == Kind::Implements {
            for &it in e.kids(k).to_vec().iter() {
                let t = data_type(e, sc, it)?;
                if let Type::Class(c) = e.d.types.get(t).clone()
                    && !e.d.classes[c.0 as usize].interface
                {
                    return Err(e.err(it, "a class can only implement interface classes (8.26)"));
                }
            }
        }
    }
    let items: Vec<NodeId> = kids
        .iter()
        .copied()
        .filter(|&k| {
            !matches!(
                e.kind(k),
                Kind::Keyword | Kind::Name | Kind::ParamPorts | Kind::Extends | Kind::Implements | Kind::EndLabel
            )
        })
        .collect();
    e.declare_pass(cs, &items)?;
    for k in items {
        e.item(cs, k)?;
    }
    super::checks::class_done(e, id)?;
    Ok(())
}

fn class_param<'a>(e: &mut Elab<'a, '_>, cs: ScopeId, a: NodeId, ty_node: Option<NodeId>) -> R<'a> {
    let k = e.kids(a).to_vec();
    let has_value = k.iter().skip(1).any(|&x| !is_dim(e.kind(x)));
    if has_value {
        e.param_assign(cs, a, ty_node, false)
    } else {
        // No default: the value comes only from a specialisation.
        let s = e.sym_of(k[0]);
        let c = e.new_const(Const::Bits(Bits::x(32)));
        let o = e.new_object(Object {
            name: s,
            kind: ObjKind::Param { local: false },
            ty: Types::INTEGER,
            scope: cs,
            node: a,
            lifetime: Lifetime::Static,
            dir: None,
            init: None,
            value: Some(c),
            qualifiers: vec![],
        });
        e.declare(cs, s, Binding::Object(o), k[0])?;
        e.push_item(cs, Item::Object(o));
        Ok(())
    }
}

#[allow(dead_code)]
pub(crate) fn real_kind(t: &Type) -> Option<RealKind> {
    match t {
        Type::Real(k) => Some(*k),
        _ => None,
    }
}
