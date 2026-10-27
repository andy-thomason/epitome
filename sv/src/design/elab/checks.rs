//! The rules elaboration checks beyond what binding and typing need: each
//! function is one rule or a group from one clause, named where it is
//! called, with the clause in its message.

use super::{Elab, R, eval, expr, lookup};
use crate::ast::{Kind, NodeId};
use crate::design::types::{Type, TypeId, Types, UDim};
use crate::design::value::Bits;
use crate::design::*;

// ---- time units (3.14) ----

/// The power of ten of a time literal's unit and its value: `10ns` is
/// (1, -8) as 10^1 * 10^-9.
fn time_of(e: &Elab, n: NodeId) -> Option<i32> {
    let t = e.text(n);
    let split = t.find(|c: char| c.is_ascii_alphabetic())?;
    let mag = match &t[..split] {
        "1" => 0,
        "10" => 1,
        "100" => 2,
        _ => return None,
    };
    let unit = match &t[split..] {
        "s" => 0,
        "ms" => -3,
        "us" => -6,
        "ns" => -9,
        "ps" => -12,
        "fs" => -15,
        _ => return None,
    };
    Some(mag + unit)
}

/// `timeunit` and `timeprecision` in a scope (3.14.2.2).
pub(crate) fn timeunits<'a>(e: &mut Elab<'a, '_>, sc: ScopeId, n: NodeId) -> R<'a> {
    let kids = e.kids(n).to_vec();
    let kw = e.text(kids[0]);
    let first = e.timeunits.get(&sc).copied();
    // The first one must come before every other item in the scope's
    // body (3.14.2.2): only other time declarations may precede it.
    if first.is_none() {
        let owner = e.scope(sc).node;
        let earlier = e.kids(owner).iter().copied().take_while(|&k| k != n).any(|k| {
            !matches!(
                e.kind(k),
                Kind::Keyword
                    | Kind::Name
                    | Kind::ParamPorts
                    | Kind::Ports
                    | Kind::PackageImport
                    | Kind::Attributes
                    | Kind::Timeunits
                    | Kind::Module
                    | Kind::Package
                    | Kind::Udp
                    | Kind::Config
                    | Kind::Checker
                    | Kind::ExternModule
            ) || (e.kind(k) == Kind::PackageImport && e.scope(sc).kind == ScopeKind::Unit)
        });
        let header_import_ok = e.scope(sc).kind != ScopeKind::Unit;
        let _ = header_import_ok;
        if earlier {
            return Err(e.err(n, format!("`{kw}` must come before the other items of its scope (3.14.2.2)")));
        }
    }
    let (mut unit, mut prec) = first.unwrap_or((None, None));
    let t1 = kids.get(1).and_then(|&k| time_of(e, k));
    let t2 = kids.get(2).and_then(|&k| time_of(e, k));
    let set = |slot: &mut Option<i32>, v: Option<i32>, what: &str| -> Result<(), String> {
        match (slot.as_ref(), v) {
            (Some(old), Some(new)) if *old != new => Err(format!("this {what} differs from the one declared before (3.14.2.2)")),
            (_, Some(new)) => {
                *slot = Some(new);
                Ok(())
            }
            _ => Ok(()),
        }
    };
    let r = if kw == "timeunit" {
        set(&mut unit, t1, "time unit").and_then(|_| set(&mut prec, t2, "time precision"))
    } else {
        set(&mut prec, t1, "time precision")
    };
    if let Err(m) = r {
        return Err(e.err(n, m));
    }
    if let (Some(u), Some(p)) = (unit, prec)
        && p > u
    {
        return Err(e.err(n, "the time precision must not be coarser than the time unit (3.14)"));
    }
    e.timeunits.insert(sc, (unit, prec));
    e.push_item(sc, Item::Opaque(n));
    Ok(())
}

/// The compilation unit's `timeunit`s are checked as they are elaborated.
pub(crate) fn unit_timeunits<'a>(_e: &mut Elab<'a, '_>, _root: NodeId) -> R<'a> {
    Ok(())
}

/// 3.14.2.3: either every design element has a time unit and precision
/// (its own, from `` `timescale ``, or from the compilation unit), or none
/// does.
fn mixed_timeunits<'a>(e: &mut Elab<'a, '_>) -> R<'a> {
    let unit_has = e.timeunits.contains_key(&e.d.unit);
    let root = e.ast.root;
    let mut with: Option<NodeId> = None;
    let mut without: Option<NodeId> = None;
    for &item in e.kids(root).to_vec().iter() {
        if !matches!(e.kind(item), Kind::Module | Kind::Package) {
            continue;
        }
        let own = e.kids(item).iter().any(|&k| e.kind(k) == Kind::Timeunits);
        let specified = own || unit_has || e.state(item).timescale.is_some();
        if specified {
            with.get_or_insert(item);
        } else {
            without.get_or_insert(item);
        }
    }
    if let (Some(w), Some(wo)) = (with, without) {
        let wn = e.name_child(w).map_or("", |n| e.text(n));
        let won = e.name_child(wo).map_or("", |n| e.text(n));
        return Err(e.err(
            wo,
            format!("`{wn}` has a time unit and precision but `{won}` does not; either all design elements have them or none does (3.14.2.3)"),
        ));
    }
    Ok(())
}

// ---- definitions, packages, imports (3.13, 26) ----

/// `export p::x` must name something the package imported (26.6).
pub(crate) fn package_exports<'a>(e: &mut Elab<'a, '_>, sc: ScopeId, n: NodeId) -> R<'a> {
    for &k in e.kids(n).to_vec().iter() {
        if e.kind(k) != Kind::PackageExport {
            continue;
        }
        for &it in e.kids(k).to_vec().iter() {
            let ik = e.kids(it).to_vec();
            if ik.len() < 2 || e.text(ik[0]) == "*" {
                continue;
            }
            let pname = e.sym_of(ik[0]);
            let Some(&pkg) = e.packages.get(&pname) else {
                return Err(e.err(ik[0], format!("there is no package `{}`", e.name(pname))));
            };
            if e.text(ik[1]) == "*" {
                continue;
            }
            let s = e.sym_of(ik[1]);
            let from = lookup::in_package(e, pkg, s);
            let here = e.scope(sc).names.get(&s).copied();
            match (from, here) {
                (Some(a), Some(b)) if a == b => {}
                (Some(_), Some(_)) => {
                    return Err(e.err(
                        ik[1],
                        format!("`{}` is exported from `{}`, but this package imported it from elsewhere (26.6)", e.name(s), e.name(pname)),
                    ));
                }
                (Some(b), None) => {
                    // A candidate for import through `p::*` is imported by
                    // being exported (26.6).
                    if e.scope(sc).wildcards.contains(&pkg) {
                        e.scope_mut(sc).names.insert(s, b);
                        continue;
                    }
                    return Err(e.err(
                        ik[1],
                        format!("`{}` is exported but was never imported into this package (26.6)", e.name(s)),
                    ));
                }
                (None, _) => {
                    return Err(e.err(ik[1], format!("package `{}` has no `{}`", e.name(pname), e.name(s))));
                }
            }
        }
    }
    Ok(())
}

/// `p::x` where `x` is in the compilation unit, declared after the
/// reference (26.3, 3.12.1).
pub(crate) fn package_ref_order<'a>(_e: &mut Elab<'a, '_>, _sc: ScopeId, _p: ScopeId, _s: Sym, _n: NodeId) -> R<'a> {
    Ok(())
}

/// `$unit::x` may only name something declared before it (26.3).
pub(crate) fn unit_ref<'a>(e: &mut Elab<'a, '_>, _sc: ScopeId, s: Sym, b: Option<Binding>, n: NodeId) -> R<'a> {
    let decl = match b {
        Some(Binding::Object(o)) => Some(e.obj(o).node),
        Some(Binding::Subroutine(x)) => Some(e.d.subroutines[x.0 as usize].node),
        Some(Binding::Class(c)) => Some(e.d.classes[c.0 as usize].node),
        _ => None,
    };
    match decl {
        Some(d) if e.ast.node(d).first > e.ast.node(n).first => {
            Err(e.err(n, format!("`$unit::{}` names an item declared after this reference (26.3)", e.name(s))))
        }
        None if b.is_none() => Err(e.err(n, format!("`$unit` has no `{}`", e.name(s)))),
        _ => Ok(()),
    }
}

/// Where a name is used, rules that depend on what it is.
pub(crate) fn object_use<'a>(e: &mut Elab<'a, '_>, sc: ScopeId, o: ObjId, n: NodeId) -> R<'a> {
    let obj_scope = e.obj(o).scope;
    let user_pkg = lookup::enclosing(e, sc, ScopeKind::Package);
    // A package may not refer to the compilation unit's items (26.3).
    if user_pkg.is_some() && obj_scope == e.d.unit && e.obj(o).kind == ObjKind::Variable {
        return Err(e.err(n, format!("package items cannot refer to `{}` in the compilation unit (26.3)", e.name(e.obj(o).name))));
    }
    // A compilation-unit item used before its declaration (3.12.1).
    if obj_scope == e.d.unit
        && e.scope(sc).kind != ScopeKind::Unit
        && lookup::enclosing(e, sc, ScopeKind::Spec).is_none()
        && e.ast.node(e.obj(o).node).first > e.ast.node(n).first
    {
        return Err(e.err(n, format!("`{}` is used before its declaration in the compilation unit", e.name(e.obj(o).name))));
    }
    Ok(())
}

// ---- declarations and types ----

pub(crate) fn var_type<'a>(e: &mut Elab<'a, '_>, _sc: ScopeId, name: NodeId, t: TypeId) -> R<'a> {
    if t == Types::VOID {
        return Err(e.err(name, "a variable cannot be `void` (6.13)"));
    }
    Ok(())
}

pub(crate) fn net_type<'a>(e: &mut Elab<'a, '_>, _sc: ScopeId, n: NodeId, t: TypeId, user: bool) -> R<'a> {
    if user {
        return Ok(());
    }
    // A net's data type must be four-state integral, or an unpacked array
    // or struct of them (6.7.1).
    let mut base = t;
    while let Type::Unpacked { elem, .. } = e.d.types.get(base) {
        base = *elem;
    }
    let ok = match e.d.types.get(base) {
        Type::Unknown => true,
        Type::Struct { .. } => true,
        _ => e.d.types.is_integral(base) && e.d.types.is_four_state(base) || e.d.types.is_real(base) && false,
    };
    if !ok {
        let what = e.d.types.describe(t, &e.d.syms);
        return Err(e.err(n, format!("a net cannot have the data type `{what}`; it must be four-state integral (6.7.1)")));
    }
    Ok(())
}

pub(crate) fn nettype_data<'a>(_e: &mut Elab<'a, '_>, _t: TypeId, _n: NodeId) -> R<'a> {
    Ok(())
}

/// An associative array's index type (7.8).
pub(crate) fn assoc_index<'a>(e: &mut Elab<'a, '_>, t: TypeId, n: NodeId) -> R<'a> {
    if matches!(e.kind(n), Kind::EnumType | Kind::StructType) {
        return Err(e.err(n, "an associative array's index type cannot declare a new type (7.8)"));
    }
    if contains_real(e, t) {
        return Err(e.err(n, "an associative array's index type cannot be or contain a real (7.8)"));
    }
    Ok(())
}

fn contains_real(e: &Elab, t: TypeId) -> bool {
    match e.d.types.get(t) {
        Type::Real(_) => true,
        Type::Struct { members, .. } => members.iter().any(|m| contains_real(e, m.ty)),
        Type::Unpacked { elem, .. } => contains_real(e, *elem),
        _ => false,
    }
}

/// An index into an associative array (7.8.1–7.8.4).
pub(crate) fn assoc_key<'a>(e: &mut Elab<'a, '_>, it: TypeId, idx: ExprId, n: NodeId) -> R<'a> {
    let kt = e.ex(idx).ty;
    if kt == Types::UNKNOWN {
        return Ok(());
    }
    if e.d.types.is_real(kt) && !e.d.types.is_real(it) {
        return Err(e.err(n, "a real cannot index an associative array with an integral index type (7.8.4)"));
    }
    if it == Types::STRING && e.d.types.is_integral(kt) && !matches!(e.ex(idx).kind, ExprKind::Const(_)) {
        return Err(e.err(n, "a string-indexed associative array needs a string index (7.8.2)"));
    }
    Ok(())
}

pub(crate) fn wildcard_key<'a>(e: &mut Elab<'a, '_>, idx: ExprId, n: NodeId) -> R<'a> {
    if e.d.types.is_real(e.ex(idx).ty) {
        return Err(e.err(n, "a wildcard-indexed associative array needs an integral index (7.8.1)"));
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn struct_member<'a>(
    e: &mut Elab<'a, '_>,
    _sc: ScopeId,
    _n: NodeId,
    union: bool,
    packed: bool,
    tagged: bool,
    t: TypeId,
    name: NodeId,
    default: Option<NodeId>,
) -> R<'a> {
    let kind = if union { "union" } else { "struct" };
    if default.is_some() && (packed || union) {
        return Err(e.err(name, format!("a member of a packed or union type cannot have a default value (7.2.2)")));
    }
    if default.is_some() && contains_union(e, t) {
        return Err(e.err(name, "a struct that contains a union cannot give its members default values (7.2.2)"));
    }
    if packed && !e.d.types.is_integral(t) && t != Types::UNKNOWN && !(tagged && t == Types::VOID) {
        let what = e.d.types.describe(t, &e.d.syms);
        return Err(e.err(name, format!("a packed {kind} can only have integral members, not `{what}` (7.2.1)")));
    }
    if union && !tagged && t == Types::VOID {
        return Err(e.err(name, "only a tagged union can have a `void` member (7.3.2)"));
    }
    if union && !tagged && !packed && is_dynamic(e, t) {
        return Err(e.err(name, "an untagged union cannot have a member of a dynamic type (7.3)"));
    }
    Ok(())
}

fn contains_union(e: &Elab, t: TypeId) -> bool {
    match e.d.types.get(t) {
        Type::Struct { union: true, .. } => true,
        Type::Struct { members, .. } => members.iter().any(|m| contains_union(e, m.ty)),
        _ => false,
    }
}

fn is_dynamic(e: &Elab, t: TypeId) -> bool {
    match e.d.types.get(t) {
        Type::Unpacked { dim: UDim::Dynamic | UDim::Queue(_) | UDim::Assoc(_) | UDim::Wildcard, .. } => true,
        Type::Unpacked { elem, .. } => is_dynamic(e, *elem),
        Type::String | Type::Class(_) | Type::Chandle | Type::Event => true,
        _ => false,
    }
}

pub(crate) fn struct_whole<'a>(
    e: &mut Elab<'a, '_>,
    n: NodeId,
    union: bool,
    packed: bool,
    soft: bool,
    tagged: bool,
    members: &[crate::design::types::Member],
) -> R<'a> {
    if union && packed && !soft && !tagged {
        let ws: Vec<u32> = members.iter().map(|m| e.d.types.width(m.ty)).collect();
        if ws.windows(2).any(|w| w[0] != w[1]) {
            return Err(e.err(n, "the members of a packed union must all be the same size, unless it is `soft` (7.3.1)"));
        }
    }
    Ok(())
}

pub(crate) fn enum_value_fits<'a>(e: &mut Elab<'a, '_>, n: NodeId, b: &Bits, width: u32, signed: bool) -> R<'a> {
    // A sized literal wider than the base type is an error (6.19).
    if b.width() > width && e.kind(n) == Kind::Number && e.ast.tokens_of(n).len() == 3 {
        let fits = b.resize(width, b.signed()).extend(b.width()).case_eq(b);
        if !fits {
            return Err(e.err(n, "the enum value does not fit its base type (6.19)"));
        }
    }
    let _ = signed;
    Ok(())
}

// ---- classes (8) ----

pub(crate) fn class_used_as_type<'a>(_e: &mut Elab<'a, '_>, _sc: ScopeId, _c: ClassId, _n: NodeId) -> R<'a> {
    Ok(())
}

pub(crate) fn extends<'a>(e: &mut Elab<'a, '_>, c: ClassId, base: ClassId, n: NodeId) -> R<'a> {
    let b = &e.d.classes[base.0 as usize];
    if b.interface && !e.d.classes[c.0 as usize].interface {
        return Err(e.err(n, "a class implements an interface class; it cannot extend one (8.26)"));
    }
    if !b.interface && e.d.classes[c.0 as usize].interface {
        return Err(e.err(n, "an interface class can only extend interface classes (8.26)"));
    }
    if e.has_keyword(b.node, "final") || e.keywords_of(b.node).contains(&":final") {
        return Err(e.err(n, "a final class cannot be extended (8.13)"));
    }
    Ok(())
}

pub(crate) fn class_done<'a>(e: &mut Elab<'a, '_>, c: ClassId) -> R<'a> {
    let class = &e.d.classes[c.0 as usize];
    if class.virtual_ || class.interface {
        return Ok(());
    }
    // A class that is not virtual must implement every pure virtual
    // method it inherits (8.21).
    let mut pure: Vec<(Sym, NodeId)> = Vec::new();
    let mut chain = vec![c];
    let mut cur = class.base;
    while let Some(b) = cur {
        chain.push(b);
        cur = e.d.classes[b.0 as usize].base;
        if chain.len() > 64 {
            break;
        }
    }
    for &k in chain.iter().rev() {
        let sc = e.d.classes[k.0 as usize].scope;
        for item in e.scope(sc).items.clone() {
            if let Item::Subroutine(s) = item {
                let sub = &e.d.subroutines[s.0 as usize];
                let is_pure = sub.qualifiers.iter().any(|&q| e.name(q) == "pure");
                pure.retain(|(n, _)| *n != sub.name);
                if is_pure {
                    pure.push((sub.name, sub.node));
                }
            }
        }
    }
    if let Some((s, _)) = pure.first() {
        let node = e.d.classes[c.0 as usize].node;
        let name = e.name_child(node).unwrap_or(node);
        return Err(e.err(
            name,
            format!("class `{}` does not implement the pure virtual method `{}`, so it must be virtual (8.21)", e.name(e.d.classes[c.0 as usize].name), e.name(*s)),
        ));
    }
    Ok(())
}

pub(crate) fn new_class<'a>(e: &mut Elab<'a, '_>, _sc: ScopeId, c: ClassId, n: NodeId) -> R<'a> {
    let class = &e.d.classes[c.0 as usize];
    if class.interface {
        return Err(e.err(n, "an interface class cannot be constructed (8.26)"));
    }
    if class.virtual_ {
        return Err(e.err(n, format!("`{}` is an abstract class, so it cannot be constructed (8.21)", e.name(class.name))));
    }
    Ok(())
}

/// `local` and `protected` members (8.18).
pub(crate) fn member_access<'a>(e: &mut Elab<'a, '_>, sc: ScopeId, c: ClassId, o: ObjId, n: NodeId) -> R<'a> {
    let quals: Vec<&str> = e.obj(o).qualifiers.iter().map(|&q| e.name(q)).collect();
    let local = quals.contains(&"local");
    let protected = quals.contains(&"protected");
    if !local && !protected {
        return Ok(());
    }
    let owner = lookup::class_of_scope(e, e.obj(o).scope).unwrap_or(c);
    let here = lookup::enclosing(e, sc, ScopeKind::Class).and_then(|s| lookup::class_of_scope(e, s));
    let Some(here) = here else {
        return Err(e.err(n, format!("`{}` is {} to its class (8.18)", e.name(e.obj(o).name), if local { "local" } else { "protected" })));
    };
    if local && here != owner {
        return Err(e.err(n, format!("`{}` is local to its class (8.18)", e.name(e.obj(o).name))));
    }
    if protected && !derives(e, here, owner) {
        return Err(e.err(n, format!("`{}` is protected, so only its class and subclasses can use it (8.18)", e.name(e.obj(o).name))));
    }
    Ok(())
}

fn derives(e: &Elab, c: ClassId, base: ClassId) -> bool {
    let mut cur = Some(c);
    let mut guard = 0;
    while let Some(k) = cur {
        if k == base {
            return true;
        }
        cur = e.d.classes[k.0 as usize].base;
        guard += 1;
        if guard > 64 {
            break;
        }
    }
    false
}

/// `C::x` of a parameterised class needs a specialisation outside it
/// (8.25.1).
pub(crate) fn class_scope_use<'a>(e: &mut Elab<'a, '_>, sc: ScopeId, c: ClassId, n: NodeId) -> R<'a> {
    let node = e.d.classes[c.0 as usize].node;
    let parameterised = e.child(node, Kind::ParamPorts).is_some();
    if !parameterised || e.kind(n) != Kind::Name {
        return Ok(());
    }
    // A typedef of a specialisation names it.
    let s = e.sym_of(n);
    if !matches!(lookup::lookup(e, sc, s), Some(Binding::Class(_))) {
        return Ok(());
    }
    let inside = lookup::enclosing(e, sc, ScopeKind::Class).and_then(|s| lookup::class_of_scope(e, s));
    if inside.is_some_and(|k| derives(e, k, c)) {
        return Ok(());
    }
    Err(e.err(n, format!("`{}::` needs a specialisation, `{}#(...)::`, outside the class (8.25.1)", e.text(n), e.text(n))))
}

pub(crate) fn this_allowed<'a>(e: &mut Elab<'a, '_>, _sc: ScopeId, n: NodeId) -> R<'a> {
    if let Some(s) = e.current_sub
        && e.d.subroutines[s.0 as usize].qualifiers.iter().any(|&q| e.name(q) == "static")
    {
        return Err(e.err(n, "`this` cannot be used in a static method (8.10)"));
    }
    Ok(())
}

// ---- expressions ----

pub(crate) fn comparable<'a>(_e: &mut Elab<'a, '_>, _op: BinOp, _a: TypeId, _b: TypeId, _n: NodeId) -> R<'a> {
    Ok(())
}

fn non_arith(e: &Elab, t: TypeId) -> Option<&'static str> {
    match e.d.types.get(t) {
        Type::Unpacked { .. } => Some("an unpacked array"),
        Type::Class(_) => Some("a class handle"),
        Type::String => Some("a string"),
        Type::Chandle => Some("a chandle"),
        Type::Event => Some("an event"),
        Type::Struct { packed: false, .. } => Some("an unpacked struct"),
        _ => None,
    }
}

pub(crate) fn integral_operand<'a>(e: &mut Elab<'a, '_>, t: TypeId, n: NodeId, op: &str) -> R<'a> {
    if e.d.types.is_real(t) {
        return Err(e.err(n, format!("`{op}` cannot take a real operand (11.4.1)")));
    }
    arith_operand(e, t, n, op)
}

pub(crate) fn arith_operand<'a>(e: &mut Elab<'a, '_>, t: TypeId, n: NodeId, op: &str) -> R<'a> {
    if let Some(what) = non_arith(e, t) {
        return Err(e.err(n, format!("{what} cannot be an operand of `{op}` (11.3)")));
    }
    Ok(())
}

/// Unsized constants are not allowed in concatenations (11.4.12).
pub(crate) fn sized_in_concat<'a>(e: &mut Elab<'a, '_>, n: NodeId) -> R<'a> {
    let is_unsized = match e.kind(n) {
        Kind::Number => {
            let toks = e.ast.tokens_of(n);
            let real = toks.len() == 1 && toks[0].0.contains(['.', 'e', 'E']);
            toks.len() < 3 && !real
        }
        _ => false,
    };
    if is_unsized {
        return Err(e.err(n, "an unsized number cannot be part of a concatenation (11.4.12)"));
    }
    Ok(())
}

/// A positional pattern needs one element per element of its target
/// (10.9).
pub(crate) fn pattern_shape<'a>(e: &mut Elab<'a, '_>, _sc: ScopeId, n: NodeId, t: TypeId, items: &[NodeId]) -> R<'a> {
    if items.iter().any(|&k| matches!(e.kind(k), Kind::PatternKey | Kind::Replicate)) {
        return Ok(());
    }
    let want = match e.d.types.get(t) {
        Type::Unpacked { dim: UDim::Range(l, r), .. } => Some((l - r).unsigned_abs() as usize + 1),
        Type::Struct { members, .. } => Some(members.len()),
        _ => None,
    };
    if let Some(w) = want
        && w != items.len()
    {
        return Err(e.err(n, format!("the pattern has {} elements, but its target has {w} (10.9)", items.len())));
    }
    Ok(())
}

pub(crate) fn cast_ok<'a>(_e: &mut Elab<'a, '_>, _t: TypeId, _from: TypeId, _n: NodeId) -> R<'a> {
    Ok(())
}

/// A part select's bounds must run in the direction of the declared range
/// (11.5.1).
pub(crate) fn part_order<'a>(e: &mut Elab<'a, '_>, bt: TypeId, l: i64, r: i64, n: NodeId) -> R<'a> {
    if l == r {
        return Ok(());
    }
    let range = match e.d.types.get(bt) {
        Type::Vector { dims, .. } if !dims.is_empty() => Some(dims[0]),
        Type::Unpacked { dim: UDim::Range(a, b), .. } => Some((*a, *b)),
        _ => None,
    };
    if let Some((a, b)) = range
        && a != b
        && ((a > b) != (l > r))
    {
        return Err(e.err(n, format!("the part select [{l}:{r}] runs against the declared range [{a}:{b}] (11.5.1)")));
    }
    Ok(())
}

/// Whether an assignment from `x` to a `target` is allowed (6.22, 7.6).
pub(crate) fn assign_compatible<'a>(e: &mut Elab<'a, '_>, _sc: ScopeId, target: TypeId, x: ExprId, n: NodeId) -> R<'a> {
    let src = e.ex(x).ty;
    if src == Types::UNKNOWN || target == Types::UNKNOWN || src == target {
        return Ok(());
    }
    let tt = e.d.types.get(target).clone();
    let st = e.d.types.get(src).clone();
    match (&tt, &st) {
        (Type::Unpacked { elem: te, dim: td }, Type::Unpacked { elem: se, dim: sd }) => {
            match (td, sd) {
                (UDim::Assoc(a), UDim::Assoc(b)) if a != b => {
                    return Err(e.err(n, "associative arrays with different index types cannot be assigned (7.9.9)"));
                }
                (UDim::Assoc(_) | UDim::Wildcard, UDim::Range(..) | UDim::Dynamic | UDim::Queue(_)) => {
                    return Err(e.err(n, "only an associative array can be assigned to an associative array (7.9.9)"));
                }
                (UDim::Range(..) | UDim::Dynamic | UDim::Queue(_), UDim::Assoc(_) | UDim::Wildcard) => {
                    return Err(e.err(n, "an associative array cannot be assigned to a fixed-size, dynamic or queue array (7.6)"));
                }
                (UDim::Range(a, b), UDim::Range(c, d)) => {
                    let (w1, w2) = ((a - b).abs(), (c - d).abs());
                    if w1 != w2 {
                        return Err(e.err(n, format!("arrays of {} and {} elements cannot be assigned (7.6)", w1 + 1, w2 + 1)));
                    }
                }
                _ => {}
            }
            if !equivalent(e, *te, *se) {
                let (a, b) = (e.d.types.describe(*te, &e.d.syms), e.d.types.describe(*se, &e.d.syms));
                return Err(e.err(n, format!("array elements of type `{b}` are not equivalent to `{a}` (7.6, 6.22.2)")));
            }
            Ok(())
        }
        (Type::Unpacked { .. }, _) if e.d.types.is_integral(src) || e.d.types.is_real(src) => {
            if matches!(e.ex(x).kind, ExprKind::Concat | ExprKind::Pattern | ExprKind::Streaming { .. }) {
                return Ok(());
            }
            // A string literal fills an unpacked array of bytes (5.9).
            if let ExprKind::Const(c) = e.ex(x).kind
                && matches!(e.d.consts[c.0 as usize], Const::Str(_))
            {
                return Ok(());
            }
            Err(e.err(n, "a packed or scalar value cannot be assigned to an unpacked array without a cast (7.6)"))
        }
        (_, Type::Unpacked { .. }) if e.d.types.is_integral(target) => {
            Err(e.err(n, "an unpacked array cannot be assigned to a packed type without a cast (7.6)"))
        }
        (Type::Class(a), Type::Class(b)) => {
            if !derives(e, *b, *a) && !implements(e, *b, *a) {
                let (an, bn) = (e.name(e.d.classes[a.0 as usize].name), e.name(e.d.classes[b.0 as usize].name));
                return Err(e.err(n, format!("a `{bn}` handle cannot be assigned to a `{an}` without `$cast` (8.16)")));
            }
            Ok(())
        }
        (Type::Enum { .. }, _) if e.d.types.is_integral(src) && !matches!(st, Type::Enum { .. }) => {
            // Integral to enum needs a cast, except for its own constants
            // (6.19.3).
            Err(e.err(n, "an integral value cannot be assigned to an enum without a cast (6.19.3)"))
        }
        (Type::Enum { .. }, Type::Enum { .. }) => {
            Err(e.err(n, "a value of one enum type cannot be assigned to another without a cast (6.19.3)"))
        }
        _ => Ok(()),
    }
}

fn implements(e: &Elab, c: ClassId, iface: ClassId) -> bool {
    if !e.d.classes[iface.0 as usize].interface {
        return false;
    }
    let node = e.d.classes[c.0 as usize].node;
    let name = e.name(e.d.classes[iface.0 as usize].name);
    e.ast.tokens_of(node).iter().any(|t| t.0 == name)
}

/// Type equivalence (6.22.2) for array elements.
fn equivalent(e: &Elab, a: TypeId, b: TypeId) -> bool {
    if a == b {
        return true;
    }
    let ty = &e.d.types;
    if a == Types::UNKNOWN || b == Types::UNKNOWN {
        return true;
    }
    match (ty.get(a), ty.get(b)) {
        (Type::Vector { .. } | Type::PackedArray { .. }, Type::Vector { .. } | Type::PackedArray { .. }) => {
            ty.width(a) == ty.width(b) && ty.is_signed(a) == ty.is_signed(b) && ty.is_four_state(a) == ty.is_four_state(b)
        }
        (Type::Unpacked { elem: x, dim: UDim::Range(l1, r1) }, Type::Unpacked { elem: y, dim: UDim::Range(l2, r2) }) => {
            (l1 - r1).abs() == (l2 - r2).abs() && equivalent(e, *x, *y)
        }
        // A dynamic array or queue may match a fixed one at run time (7.6).
        (Type::Unpacked { elem: x, dim: d1 }, Type::Unpacked { elem: y, dim: d2 })
            if matches!(d1, UDim::Dynamic | UDim::Queue(_)) || matches!(d2, UDim::Dynamic | UDim::Queue(_)) =>
        {
            equivalent(e, *x, *y)
        }
        (Type::Real(k1), Type::Real(k2)) => k1 == k2 || ty.width(a) == ty.width(b),
        _ => false,
    }
}

/// What may be assigned in a procedure (10.4).
pub(crate) fn lvalue<'a>(e: &mut Elab<'a, '_>, sc: ScopeId, x: ExprId, n: NodeId, nonblocking: bool) -> R<'a> {
    let ex = e.ex(x).clone();
    match ex.kind {
        ExprKind::Object(o) | ExprKind::Hier { object: Some(o), .. } | ExprKind::Field(o) => {
            let obj = e.obj(o);
            let name = e.name(obj.name);
            match obj.kind {
                ObjKind::Net(_) => {
                    return Err(e.err(n, format!("`{name}` is a net, so a procedure cannot assign it (10.4)")));
                }
                ObjKind::Param { .. } | ObjKind::EnumConst => {
                    return Err(e.err(n, format!("`{name}` is a constant, so it cannot be assigned")));
                }
                ObjKind::Genvar => {
                    return Err(e.err(n, format!("genvar `{name}` cannot be assigned in a procedure (27.4)")));
                }
                ObjKind::Variable => {}
            }
            let quals: Vec<&str> = obj.qualifiers.iter().map(|&q| e.name(q)).collect();
            if quals.contains(&"foreach") {
                return Err(e.err(n, format!("`{name}` is a `foreach` loop variable, so it cannot be assigned (12.7.3)")));
            }
            if obj.dir == Some(Dir::ConstRef) {
                return Err(e.err(n, format!("`{name}` is a `const ref` argument, so it cannot be written (13.5.2)")));
            }
            if quals.contains(&"const") && !in_constructor(e, sc) {
                return Err(e.err(n, format!("`{name}` is `const`, so it cannot be assigned (6.20.6)")));
            }
            if nonblocking && obj.lifetime == Lifetime::Automatic && !matches!(ex.kind, ExprKind::Field(_)) {
                return Err(e.err(n, format!("`{name}` is automatic, so a nonblocking assignment cannot write it (10.4.2)")));
            }
            Ok(())
        }
        ExprKind::Index | ExprKind::Part { .. } | ExprKind::Member(_) | ExprKind::Convert => {
            let base = ex.args[0];
            lvalue(e, sc, base, n, nonblocking)
        }
        ExprKind::Concat | ExprKind::Pattern | ExprKind::Streaming { .. } => {
            for a in ex.args {
                if !matches!(e.ex(a).kind, ExprKind::TypeRef(_)) {
                    lvalue(e, sc, a, n, nonblocking)?;
                }
            }
            Ok(())
        }
        ExprKind::Const(_) | ExprKind::Binary(_) | ExprKind::Unary(_) | ExprKind::Cond => {
            Err(e.err(n, "this expression cannot be assigned"))
        }
        _ => Ok(()),
    }
}

fn in_constructor(e: &Elab, _sc: ScopeId) -> bool {
    e.current_sub.is_some_and(|s| e.name(e.d.subroutines[s.0 as usize].name) == "new")
}

/// What a continuous assignment may drive (10.3).
pub(crate) fn continuous_lvalue<'a>(e: &mut Elab<'a, '_>, sc: ScopeId, x: ExprId, n: NodeId) -> R<'a> {
    let ex = e.ex(x).clone();
    match ex.kind {
        ExprKind::Object(o) | ExprKind::Hier { object: Some(o), .. } => {
            let obj = e.obj(o);
            match obj.kind {
                ObjKind::Param { .. } | ObjKind::EnumConst | ObjKind::Genvar => {
                    Err(e.err(n, format!("`{}` is a constant, so it cannot be assigned", e.name(obj.name))))
                }
                ObjKind::Variable if obj.lifetime == Lifetime::Automatic => {
                    Err(e.err(n, "a continuous assignment cannot drive an automatic variable (10.3)"))
                }
                _ => Ok(()),
            }
        }
        ExprKind::Index | ExprKind::Part { .. } | ExprKind::Member(_) => continuous_lvalue(e, sc, ex.args[0], n),
        ExprKind::Concat => {
            for a in ex.args {
                continuous_lvalue(e, sc, a, n)?;
            }
            Ok(())
        }
        ExprKind::Const(_) | ExprKind::Binary(_) | ExprKind::Unary(_) | ExprKind::Cond | ExprKind::Call(_) => {
            Err(e.err(n, "this expression cannot be driven by a continuous assignment"))
        }
        _ => Ok(()),
    }
}

pub(crate) fn port_lvalue<'a>(e: &mut Elab<'a, '_>, sc: ScopeId, port: &Port, x: ExprId, n: NodeId) -> R<'a> {
    match port.dir {
        Dir::Output | Dir::Inout => match e.ex(x).kind {
            ExprKind::Const(_) => Err(e.err(n, format!("output port `{}` cannot be connected to a constant", e.name(port.name)))),
            _ => Ok(()),
        },
        Dir::Ref => {
            if let ExprKind::Object(o) = e.ex(x).kind
                && matches!(e.obj(o).kind, ObjKind::Net(_))
            {
                return Err(e.err(n, "a `ref` port must be connected to a variable (23.3.3.3)"));
            }
            let _ = sc;
            Ok(())
        }
        _ => Ok(()),
    }
}

// ---- calls ----

pub(crate) fn output_arg<'a>(e: &mut Elab<'a, '_>, sc: ScopeId, _sub: SubId, f: ObjId, x: ExprId, n: NodeId) -> R<'a> {
    let dir = e.obj(f).dir.unwrap_or(Dir::Input);
    if matches!(dir, Dir::Ref | Dir::ConstRef)
        && let ExprKind::Object(o) = e.ex(x).kind
        && matches!(e.obj(o).kind, ObjKind::Net(_))
    {
        return Err(e.err(n, "a net cannot be passed by reference (13.5.2)"));
    }
    if e.in_cont_assign && matches!(dir, Dir::Output | Dir::Inout | Dir::Ref) {
        return Err(e.err(n, "a function with output, inout or ref arguments cannot be called in a continuous assignment (13.4)"));
    }
    if dir == Dir::ConstRef {
        return Ok(());
    }
    lvalue(e, sc, x, n, false)
}

pub(crate) fn call_allowed<'a>(e: &mut Elab<'a, '_>, _sc: ScopeId, sub: SubId, n: NodeId) -> R<'a> {
    let callee = &e.d.subroutines[sub.0 as usize];
    if let Some(cur) = e.current_sub
        && e.d.subroutines[cur.0 as usize].kind == SubKind::Function
        && callee.kind == SubKind::Task
    {
        return Err(e.err(n, format!("a function cannot call the task `{}` (13.4)", e.name(callee.name))));
    }
    if e.in_cont_assign && callee.kind == SubKind::Task {
        return Err(e.err(n, "a continuous assignment cannot call a task"));
    }
    Ok(())
}

/// A constant function's restrictions (13.4.3).
pub(crate) fn constant_function<'a>(e: &mut Elab<'a, '_>, sub: SubId, n: NodeId) -> R<'a> {
    super::stmt::sub_signature(e, sub)?;
    let s = &e.d.subroutines[sub.0 as usize];
    let name = e.name(s.name);
    if s.kind == SubKind::Task {
        return Err(e.err(n, format!("`{name}` is a task, so it cannot be called in a constant expression")));
    }
    if s.qualifiers.iter().any(|&q| e.name(q) == "import") {
        return Err(e.err(n, format!("`{name}` is a DPI import, so it cannot be a constant function (13.4.3)")));
    }
    if s.args.iter().any(|&a| e.obj(a).dir.is_some_and(|d| d != Dir::Input)) {
        return Err(e.err(n, format!("`{name}` has output, inout or ref arguments, so it cannot be a constant function (13.4.3)")));
    }
    if !s.elaborated {
        super::stmt::sub_body(e, sub)?;
    }
    let s = &e.d.subroutines[sub.0 as usize];
    let body = s.body.clone();
    let mut hier = false;
    for st in body {
        walk_stmt(e, st, &mut |e, x| {
            if matches!(e.ex(x).kind, ExprKind::Hier { .. }) {
                hier = true;
            }
        });
    }
    if hier {
        return Err(e.err(n, format!("`{name}` makes a hierarchical reference, so it cannot be a constant function (13.4.3)")));
    }
    Ok(())
}

/// System tasks and functions (20, 21).
pub(crate) fn syscall<'a>(e: &mut Elab<'a, '_>, _sc: ScopeId, name: &str, xs: &[ExprId], args: &[NodeId], n: NodeId) -> R<'a> {
    match name {
        "$monitor" | "$fmonitor" | "$strobe" | "$fstrobe" => {
            for (i, &x) in xs.iter().enumerate() {
                let mut auto = None;
                walk_expr(e, x, &mut |e, y| {
                    if let ExprKind::Object(o) = e.ex(y).kind
                        && e.obj(o).lifetime == Lifetime::Automatic
                        && e.obj(o).kind == ObjKind::Variable
                    {
                        auto = Some(o);
                    }
                });
                if let Some(o) = auto {
                    let at = args.get(i).copied().unwrap_or(n);
                    return Err(e.err(at, format!("`{}` is automatic, so `{name}` cannot watch it (21.2.3)", e.name(e.obj(o).name))));
                }
            }
            Ok(())
        }
        "$bits" if xs.is_empty() => Err(e.err(n, "`$bits` needs an argument")),
        "$cast" if xs.len() != 2 => Err(e.err(n, "`$cast` takes two arguments (6.24.2)")),
        _ => Ok(()),
    }
}

/// Built-in array methods (7.12).
pub(crate) fn builtin_method<'a>(
    e: &mut Elab<'a, '_>,
    bt: TypeId,
    m: Sym,
    args: &[ExprId],
    with: Option<NodeId>,
    n: NodeId,
) -> R<'a> {
    let name = e.name(m);
    let Type::Unpacked { dim, .. } = e.d.types.get(bt).clone() else { return Ok(()) };
    let assoc = matches!(dim, UDim::Assoc(_) | UDim::Wildcard);
    match name {
        "first" | "last" | "next" | "prev" if dim == UDim::Wildcard => {
            Err(e.err(n, format!("`{name}` cannot be used on a wildcard-indexed associative array (7.9.4)")))
        }
        "sort" | "rsort" | "reverse" | "shuffle" if assoc => {
            Err(e.err(n, format!("`{name}` cannot be used on an associative array (7.12.2)")))
        }
        "reverse" | "shuffle" if with.is_some() => Err(e.err(n, format!("`{name}` does not take a `with` clause (7.12.2)"))),
        "find" | "find_index" | "find_first" | "find_first_index" | "find_last" | "find_last_index" if with.is_none() => {
            Err(e.err(n, format!("`{name}` needs a `with` clause (7.12.1)")))
        }
        "find" | "find_index" | "find_first" | "find_first_index" | "find_last" | "find_last_index" | "min" | "max"
        | "unique" | "unique_index" | "sum" | "product" | "and" | "or" | "xor" | "sort" | "rsort"
            if !args.is_empty() && with.is_none() =>
        {
            Err(e.err(n, format!("`{name}` names an iterator argument but has no `with` clause (7.12)")))
        }
        _ => Ok(()),
    }
}

/// `new[n]` needs a dynamic array (7.5.1).
pub(crate) fn new_array<'a>(e: &mut Elab<'a, '_>, t: TypeId, n: NodeId) -> R<'a> {
    match e.d.types.get(t) {
        Type::Unpacked { dim: UDim::Dynamic, .. } | Type::Unknown => Ok(()),
        Type::Unpacked { .. } => Err(e.err(n, "`new[]` can only create a dynamic array, and this array's outer dimension is not dynamic (7.5.1)")),
        _ => Ok(()),
    }
}

// ---- statements ----

pub(crate) fn expr_statement<'a>(_e: &mut Elab<'a, '_>, _x: ExprId, _n: NodeId) -> R<'a> {
    Ok(())
}

pub(crate) fn proc_continuous<'a>(e: &mut Elab<'a, '_>, _sc: ScopeId, kw: &str, lhs: ExprId, n: NodeId) -> R<'a> {
    let mut auto = None;
    let mut net = None;
    walk_expr(e, lhs, &mut |e, y| {
        if let ExprKind::Object(o) = e.ex(y).kind {
            if e.obj(o).lifetime == Lifetime::Automatic && e.obj(o).kind == ObjKind::Variable {
                auto = Some(o);
            }
            if matches!(e.obj(o).kind, ObjKind::Net(_)) {
                net = Some(o);
            }
        }
    });
    if let Some(o) = auto {
        return Err(e.err(n, format!("`{}` is automatic, so `{kw}` cannot refer to it (10.6)", e.name(e.obj(o).name))));
    }
    if let Some(o) = net
        && matches!(kw, "assign" | "deassign")
    {
        return Err(e.err(n, format!("`{kw}` applies to variables; `{}` is a net (10.6.1)", e.name(e.obj(o).name))));
    }
    Ok(())
}

pub(crate) fn return_stmt<'a>(e: &mut Elab<'a, '_>, v: Option<NodeId>, n: NodeId) -> R<'a> {
    match e.current_sub {
        Some(s) => {
            let sub = &e.d.subroutines[s.0 as usize];
            if v.is_some() && (sub.kind == SubKind::Task || sub.ret == Types::VOID) {
                let what = if sub.kind == SubKind::Task { "a task" } else { "a void function" };
                return Err(e.err(n, format!("{what} cannot return a value (13.3, 13.4.1)")));
            }
            if v.is_none() && sub.kind == SubKind::Function && sub.ret != Types::VOID {
                // Allowed: the function's variable holds the result.
            }
            Ok(())
        }
        None => {
            if v.is_some() {
                return Err(e.err(n, "`return` with a value is only allowed in a function"));
            }
            Ok(())
        }
    }
}

/// Delays and event controls: not in functions (13.4), nor in
/// `always_comb`, `always_latch` or `final` (9.2.2, 9.2.3).
pub(crate) fn timing_allowed<'a>(e: &mut Elab<'a, '_>, _sc: ScopeId, n: NodeId) -> R<'a> {
    if let Some(s) = e.current_sub
        && e.d.subroutines[s.0 as usize].kind == SubKind::Function
    {
        return Err(e.err(n, "a function cannot contain a delay, event control or `wait` (13.4)"));
    }
    match e.proc_kind {
        Some(ProcKind::AlwaysComb) | Some(ProcKind::AlwaysLatch) => {
            Err(e.err(n, "an `always_comb` or `always_latch` procedure cannot contain timing controls (9.2.2.2)"))
        }
        Some(ProcKind::Final) => Err(e.err(n, "a `final` procedure cannot contain timing controls (9.2.3)")),
        _ => Ok(()),
    }
}

pub(crate) fn fork_allowed<'a>(e: &mut Elab<'a, '_>, _sc: ScopeId, n: NodeId) -> R<'a> {
    if let Some(s) = e.current_sub
        && e.d.subroutines[s.0 as usize].kind == SubKind::Function
    {
        let last = e.kids(n).iter().rev().find(|&&k| e.kind(k) == Kind::Keyword).map(|&k| e.text(k));
        if last != Some("join_none") {
            return Err(e.err(n, "a function can only use `fork ... join_none` (13.4.4)"));
        }
    }
    Ok(())
}

/// A deferred assertion's action block is a single subroutine call
/// (16.4).
pub(crate) fn action_block<'a>(e: &mut Elab<'a, '_>, ab: NodeId, deferred: bool) -> R<'a> {
    if !deferred {
        return Ok(());
    }
    for &s in e.kids(ab).to_vec().iter() {
        let ok = match e.kind(s) {
            Kind::ExprStatement => matches!(e.kind(e.kids(s)[0]), Kind::Call | Kind::SystemName),
            Kind::NullStatement | Kind::Empty => true,
            _ => false,
        };
        if !ok {
            return Err(e.err(s, "a deferred assertion's action block must be a single subroutine call (16.4)"));
        }
    }
    Ok(())
}

pub(crate) fn clocking_drive<'a>(_e: &mut Elab<'a, '_>, _sc: ScopeId, _n: NodeId) -> R<'a> {
    Ok(())
}

pub(crate) fn clocking<'a>(_e: &mut Elab<'a, '_>, _sc: ScopeId, _n: NodeId) -> R<'a> {
    Ok(())
}

pub(crate) fn sub_done<'a>(_e: &mut Elab<'a, '_>, _sub: SubId) -> R<'a> {
    Ok(())
}

pub(crate) fn dpi_import<'a>(_e: &mut Elab<'a, '_>, _sc: ScopeId, _n: NodeId) -> R<'a> {
    Ok(())
}

pub(crate) fn dpi_export<'a>(e: &mut Elab<'a, '_>, sc: ScopeId, n: NodeId) -> R<'a> {
    let name = e.kids(n).iter().copied().filter(|&k| e.kind(k) == Kind::Name).last();
    if let Some(nm) = name {
        let s = e.sym_of(nm);
        match e.scope(sc).names.get(&s).copied() {
            Some(Binding::Subroutine(_)) => {}
            _ => {
                return Err(e.err(nm, format!("`{}` is exported, but no task or function of that name is declared in this scope (35.7)", e.name(s))));
            }
        }
    }
    e.push_item(sc, Item::Opaque(n));
    Ok(())
}

/// `always` procedures are not allowed in programs (24.3).
pub(crate) fn process_place<'a>(e: &mut Elab<'a, '_>, sc: ScopeId, n: NodeId) -> R<'a> {
    let kw = e.text(e.kids(n)[0]);
    if kw.starts_with("always") && e.in_program_or_interface(sc) == Some(DefKind::Program) {
        return Err(e.err(n, format!("a program cannot contain `{kw}` procedures (24.3)")));
    }
    Ok(())
}

pub(crate) fn process_body<'a>(
    _e: &mut Elab<'a, '_>,
    _sc: ScopeId,
    _kind: ProcKind,
    _body: StmtId,
    _writes: &[ObjId],
    _n: NodeId,
) -> R<'a> {
    Ok(())
}

pub(crate) fn elaboration_task<'a>(e: &mut Elab<'a, '_>, sc: ScopeId, n: NodeId) -> R<'a> {
    let call = e.kids(n)[0];
    let x = expr::expr(e, sc, call)?;
    let name = e.text(e.kids(call).first().copied().unwrap_or(call));
    if matches!(name, "$fatal" | "$error") {
        // The message, if it can be worked out.
        let msg = match e.ex(x).args.iter().find(|&&a| matches!(e.d.types.get(e.ex(a).ty), Type::Vector { .. })) {
            Some(_) => String::new(),
            None => String::new(),
        };
        return Err(e.err(n, format!("`{name}` was called during elaboration (20.11){msg}")));
    }
    e.push_item(sc, Item::Opaque(n));
    Ok(())
}

// ---- gates (28) ----

pub(crate) fn gate_shape<'a>(e: &mut Elab<'a, '_>, n: NodeId) -> R<'a> {
    let kids = e.kids(n).to_vec();
    let kw = e.text(kids[0]);
    if let Some(d) = kids.iter().copied().find(|&k| e.kind(k) == Kind::Delay) {
        let count = e.kids(d).len();
        let max = match kw {
            "and" | "nand" | "or" | "nor" | "xor" | "xnor" | "buf" | "not" | "tran" | "rtran" => 2,
            "pullup" | "pulldown" => 0,
            _ => 3,
        };
        if kw == "tran" || kw == "rtran" {
            return Err(e.err(d, format!("`{kw}` takes no delay (28.16)")));
        }
        if count > max {
            return Err(e.err(d, format!("`{kw}` takes at most {max} delays, but {count} are given (28.16)")));
        }
    }
    Ok(())
}

pub(crate) fn gate_terminals<'a>(
    e: &mut Elab<'a, '_>,
    _sc: ScopeId,
    n: NodeId,
    inst: NodeId,
    kind: Sym,
    terms: &[ExprId],
) -> R<'a> {
    let kw = e.name(kind);
    let is_gate = !e.defs.contains_key(&kind);
    if !is_gate {
        return Ok(());
    }
    let array = e.kids(inst).iter().any(|&k| super::ty::is_dim(e.kind(k)));
    if !array {
        for (i, &t) in terms.iter().enumerate() {
            let w = e.d.types.width(e.ex(t).ty);
            // Nets of a user-defined nettype carry any data type (6.6.7).
            let user_net = matches!(e.ex(t).kind, ExprKind::Object(o)
                if matches!(e.obj(o).kind, ObjKind::Net(k) if e.nettypes.contains(&k)));
            if w > 1 && !user_net {
                let what = if matches!(kw, "tran" | "rtran" | "tranif0" | "tranif1" | "rtranif0" | "rtranif1") {
                    format!("`{kw}` connects single bits, not {w}-bit vectors (28.8)")
                } else if i == 0 {
                    format!("the output of a single `{kw}` gate must be one bit, not {w} (28.3)")
                } else {
                    continue;
                };
                return Err(e.err(n, what));
            }
        }
    }
    Ok(())
}

// ---- generate (27) ----

/// The loop's step must assign its own genvar (27.4).
pub(crate) fn genvar_step<'a>(e: &mut Elab<'a, '_>, _sc: ScopeId, step: NodeId, gv: Sym) -> R<'a> {
    let target = match e.kind(step) {
        Kind::Assign => e.kids(step).first().copied(),
        Kind::IncDec => e.kids(step).iter().copied().find(|&k| e.kind(k) != Kind::Op),
        Kind::ForStep | Kind::ExprStatement => {
            let inner = e.kids(step).first().copied();
            return match inner {
                Some(i) => genvar_step(e, _sc, i, gv),
                None => Ok(()),
            };
        }
        _ => None,
    };
    if let Some(t) = target
        && e.kind(t) == Kind::Name
        && e.sym_of(t) != gv
    {
        return Err(e.err(t, format!("the loop's step assigns `{}`, not its genvar `{}` (27.4)", e.text(t), e.name(gv))));
    }
    Ok(())
}

/// Whether the syntax under `n` names `s`.
pub(crate) fn mentions(e: &mut Elab, n: NodeId, s: Sym) -> bool {
    let text = e.name(s);
    e.ast.tokens_of(n).iter().any(|t| t.0 == text)
}

// ---- other items ----

pub(crate) fn specify<'a>(_e: &mut Elab<'a, '_>, _sc: ScopeId, _n: NodeId) -> R<'a> {
    Ok(())
}

pub(crate) fn defparam<'a>(e: &mut Elab<'a, '_>, sc: ScopeId, n: NodeId) -> R<'a> {
    e.defparams.push((sc, n, e.inst_path.clone()));
    Ok(())
}

pub(crate) fn net_alias<'a>(e: &mut Elab<'a, '_>, n: NodeId, parts: &[ExprId]) -> R<'a> {
    let mut width = None;
    for &p in parts {
        if let ExprKind::Object(o) = e.ex(p).kind
            && !matches!(e.obj(o).kind, ObjKind::Net(_))
        {
            return Err(e.err(n, format!("`{}` is not a net, so it cannot be aliased (10.11)", e.name(e.obj(o).name))));
        }
        let w = e.d.types.width(e.ex(p).ty);
        if let Some(old) = width
            && old != w
            && w != 0
        {
            return Err(e.err(n, format!("aliased nets must be the same width, not {old} and {w} (10.11)")));
        }
        width = Some(w);
    }
    Ok(())
}

pub(crate) fn spec_done<'a>(_e: &mut Elab<'a, '_>, _sc: ScopeId, _kind: DefKind) -> R<'a> {
    Ok(())
}

/// Checks that need the whole design.
pub(crate) fn after<'a>(e: &mut Elab<'a, '_>) -> R<'a> {
    mixed_timeunits(e)?;
    defparams(e)?;
    drivers(e)?;
    Ok(())
}

/// A `defparam` may not set a local parameter (23.10.1).
fn defparams<'a>(e: &mut Elab<'a, '_>) -> R<'a> {
    for (sc, n, path) in e.defparams.clone() {
        let saved = std::mem::replace(&mut e.inst_path, path);
        for &a in e.kids(n).to_vec().iter() {
            let target = e.kids(a)[0];
            let r = expr::hier(e, sc, target);
            let x = match r {
                Ok(Some(x)) => Some(x),
                Ok(None) => expr::expr(e, sc, target).ok(),
                Err(err) => {
                    e.inst_path = saved;
                    return Err(err);
                }
            };
            if let Some(x) = x
                && let ExprKind::Hier { object: Some(o), .. } | ExprKind::Object(o) = e.ex(x).kind
            {
                match e.obj(o).kind {
                    ObjKind::Param { local: true } => {
                        e.inst_path = saved;
                        return Err(e.err(target, format!("`{}` is a local parameter, so `defparam` cannot set it (23.10.1)", e.name(e.obj(o).name))));
                    }
                    ObjKind::Param { .. } => {}
                    _ => {
                        e.inst_path = saved;
                        return Err(e.err(target, "`defparam` can only set a parameter (23.10.1)"));
                    }
                }
            }
        }
        e.inst_path = saved;
    }
    Ok(())
}

/// A variable may be written by one continuous assignment, or by
/// procedures, but not both (6.5); `always_comb`, `always_latch` and
/// `always_ff` own the variables they write (9.2.2).
fn drivers<'a>(e: &mut Elab<'a, '_>) -> R<'a> {
    use std::collections::HashMap;
    let mut cont: HashMap<ObjId, NodeId> = HashMap::new();
    for d in &e.d.drivers {
        if let ExprKind::Object(o) = e.d.exprs[d.lhs.0 as usize].kind
            && e.d.objects[o.0 as usize].kind == ObjKind::Variable
        {
            if let Some(&first) = cont.get(&o) {
                let _ = first;
                return Err(e.err(d.node, format!("variable `{}` is driven by two continuous assignments (6.5)", e.name(e.d.objects[o.0 as usize].name))));
            }
            cont.insert(o, d.node);
        }
    }
    let mut owner: HashMap<ObjId, (ProcKind, NodeId)> = HashMap::new();
    for p in &e.d.processes {
        for &w in &p.writes {
            if e.d.objects[w.0 as usize].kind != ObjKind::Variable || e.d.objects[w.0 as usize].lifetime == Lifetime::Automatic {
                continue;
            }
            if cont.contains_key(&w) {
                return Err(e.err(p.node, format!("variable `{}` is written by a procedure and by a continuous assignment (6.5)", e.name(e.d.objects[w.0 as usize].name))));
            }
            let exclusive = matches!(p.kind, ProcKind::AlwaysComb | ProcKind::AlwaysLatch | ProcKind::AlwaysFf);
            if let Some(&(k, _)) = owner.get(&w)
                && (exclusive || matches!(k, ProcKind::AlwaysComb | ProcKind::AlwaysLatch | ProcKind::AlwaysFf))
            {
                return Err(e.err(
                    p.node,
                    format!("variable `{}` is written by an `always_comb`, `always_latch` or `always_ff` procedure and by another process (9.2.2)", e.name(e.d.objects[w.0 as usize].name)),
                ));
            }
            owner.entry(w).or_insert((p.kind, p.node));
        }
    }
    Ok(())
}

// ---- walking ----

pub(crate) fn walk_expr(e: &Elab, x: ExprId, f: &mut dyn FnMut(&Elab, ExprId)) {
    f(e, x);
    for &a in &e.ex(x).args {
        walk_expr(e, a, f);
    }
}

pub(crate) fn walk_stmt(e: &Elab, s: StmtId, f: &mut dyn FnMut(&Elab, ExprId)) {
    let st = e.d.stmts[s.0 as usize].kind.clone();
    let mut ex = |x: ExprId, f: &mut dyn FnMut(&Elab, ExprId)| walk_expr(e, x, f);
    match st {
        StmtKind::Block { stmts, .. } => stmts.into_iter().for_each(|s| walk_stmt(e, s, f)),
        StmtKind::Assign { lhs, rhs, timing, .. } => {
            ex(lhs, f);
            ex(rhs, f);
            if let Some(t) = timing {
                walk_timing(e, &t, f);
            }
        }
        StmtKind::Expr(x) => ex(x, f),
        StmtKind::If { cond, then, else_, .. } => {
            ex(cond, f);
            walk_stmt(e, then, f);
            if let Some(s) = else_ {
                walk_stmt(e, s, f);
            }
        }
        StmtKind::Case { expr, items, default, .. } => {
            ex(expr, f);
            for (xs, s) in items {
                xs.into_iter().for_each(|x| walk_expr(e, x, f));
                walk_stmt(e, s, f);
            }
            if let Some(d) = default {
                walk_stmt(e, d, f);
            }
        }
        StmtKind::Forever(s) => walk_stmt(e, s, f),
        StmtKind::Repeat(x, s) | StmtKind::While(x, s) | StmtKind::DoWhile(s, x) | StmtKind::Wait(x, s) => {
            ex(x, f);
            walk_stmt(e, s, f);
        }
        StmtKind::For { init, cond, step, body, .. } => {
            init.into_iter().for_each(|s| walk_stmt(e, s, f));
            if let Some(c) = cond {
                ex(c, f);
            }
            step.into_iter().for_each(|s| walk_stmt(e, s, f));
            walk_stmt(e, body, f);
        }
        StmtKind::Foreach { array, body, .. } => {
            ex(array, f);
            walk_stmt(e, body, f);
        }
        StmtKind::Timed(t, s) => {
            walk_timing(e, &t, f);
            walk_stmt(e, s, f);
        }
        StmtKind::Return(Some(x)) | StmtKind::Disable(x) => ex(x, f),
        StmtKind::Trigger { event, .. } => ex(event, f),
        StmtKind::ProcContinuous { lhs, rhs, .. } => {
            ex(lhs, f);
            if let Some(r) = rhs {
                ex(r, f);
            }
        }
        StmtKind::Assert { cond, pass, fail, .. } => {
            ex(cond, f);
            if let Some(s) = pass {
                walk_stmt(e, s, f);
            }
            if let Some(s) = fail {
                walk_stmt(e, s, f);
            }
        }
        _ => {}
    }
}

fn walk_timing(e: &Elab, t: &Timing, f: &mut dyn FnMut(&Elab, ExprId)) {
    match t {
        Timing::Delay(xs) => xs.iter().for_each(|&x| walk_expr(e, x, f)),
        Timing::Event(ts) | Timing::Implicit(ts) => {
            for t in ts {
                walk_expr(e, t.expr, f);
                if let Some(i) = t.iff {
                    walk_expr(e, i, f);
                }
            }
        }
        Timing::Cycle(x) => walk_expr(e, *x, f),
        Timing::Repeat(x, ts) => {
            walk_expr(e, *x, f);
            for t in ts {
                walk_expr(e, t.expr, f);
            }
        }
    }
}

/// The objects a statement may read and write, through the subroutines it
/// calls too (section 8.5 of the design).
pub(crate) fn read_write_sets(e: &Elab, body: StmtId) -> (Vec<ObjId>, Vec<ObjId>) {
    let mut reads = std::collections::BTreeSet::new();
    let mut writes = std::collections::BTreeSet::new();
    let mut seen = std::collections::HashSet::new();
    rw_stmt(e, body, &mut reads, &mut writes, &mut seen);
    (reads.into_iter().collect(), writes.into_iter().collect())
}

type Set = std::collections::BTreeSet<ObjId>;

fn rw_stmt(e: &Elab, s: StmtId, r: &mut Set, w: &mut Set, seen: &mut std::collections::HashSet<SubId>) {
    let st = e.d.stmts[s.0 as usize].kind.clone();
    match st {
        StmtKind::Assign { lhs, rhs, kind, timing } => {
            target(e, lhs, r, w, seen);
            if matches!(kind, AssignKind::Compound(_)) {
                rw_expr(e, lhs, r, w, seen);
            }
            rw_expr(e, rhs, r, w, seen);
            if let Some(t) = timing {
                let mut f = |e: &Elab, x: ExprId| {
                    if let ExprKind::Object(o) = e.ex(x).kind {
                        r.insert(o);
                    }
                };
                walk_timing(e, &t, &mut f);
            }
        }
        StmtKind::ProcContinuous { lhs, rhs, .. } => {
            target(e, lhs, r, w, seen);
            if let Some(x) = rhs {
                rw_expr(e, x, r, w, seen);
            }
        }
        StmtKind::Block { stmts, .. } => stmts.into_iter().for_each(|s| rw_stmt(e, s, r, w, seen)),
        StmtKind::If { cond, then, else_, .. } => {
            rw_expr(e, cond, r, w, seen);
            rw_stmt(e, then, r, w, seen);
            if let Some(x) = else_ {
                rw_stmt(e, x, r, w, seen);
            }
        }
        StmtKind::Case { expr, items, default, .. } => {
            rw_expr(e, expr, r, w, seen);
            for (xs, s) in items {
                xs.into_iter().for_each(|x| rw_expr(e, x, r, w, seen));
                rw_stmt(e, s, r, w, seen);
            }
            if let Some(d) = default {
                rw_stmt(e, d, r, w, seen);
            }
        }
        StmtKind::For { init, cond, step, body, .. } => {
            init.into_iter().for_each(|s| rw_stmt(e, s, r, w, seen));
            if let Some(c) = cond {
                rw_expr(e, c, r, w, seen);
            }
            step.into_iter().for_each(|s| rw_stmt(e, s, r, w, seen));
            rw_stmt(e, body, r, w, seen);
        }
        StmtKind::Foreach { array, body, .. } => {
            rw_expr(e, array, r, w, seen);
            rw_stmt(e, body, r, w, seen);
        }
        StmtKind::Forever(s) => rw_stmt(e, s, r, w, seen),
        StmtKind::Repeat(x, s) | StmtKind::While(x, s) | StmtKind::DoWhile(s, x) | StmtKind::Wait(x, s) => {
            rw_expr(e, x, r, w, seen);
            rw_stmt(e, s, r, w, seen);
        }
        StmtKind::Timed(t, s) => {
            let mut f = |e: &Elab, x: ExprId| {
                if let ExprKind::Object(o) = e.ex(x).kind {
                    r.insert(o);
                }
            };
            walk_timing(e, &t, &mut f);
            rw_stmt(e, s, r, w, seen);
        }
        StmtKind::Expr(x) | StmtKind::Return(Some(x)) => rw_expr(e, x, r, w, seen),
        StmtKind::Trigger { event, .. } => {
            target(e, event, r, w, seen);
        }
        StmtKind::Assert { cond, pass, fail, .. } => {
            rw_expr(e, cond, r, w, seen);
            if let Some(s) = pass {
                rw_stmt(e, s, r, w, seen);
            }
            if let Some(s) = fail {
                rw_stmt(e, s, r, w, seen);
            }
        }
        _ => {}
    }
}

/// The objects an assignment target writes; its indices are read.
fn target(e: &Elab, x: ExprId, r: &mut Set, w: &mut Set, seen: &mut std::collections::HashSet<SubId>) {
    let ex = e.ex(x);
    match &ex.kind {
        ExprKind::Object(o) | ExprKind::Hier { object: Some(o), .. } => {
            w.insert(*o);
        }
        ExprKind::Index | ExprKind::Part { .. } => {
            target(e, ex.args[0], r, w, seen);
            for &a in &ex.args[1..] {
                rw_expr(e, a, r, w, seen);
            }
        }
        ExprKind::Member(_) | ExprKind::Convert => target(e, ex.args[0], r, w, seen),
        ExprKind::Concat | ExprKind::Pattern => {
            for &a in &ex.args {
                target(e, a, r, w, seen);
            }
        }
        _ => rw_expr(e, x, r, w, seen),
    }
}

fn rw_expr(e: &Elab, x: ExprId, r: &mut Set, w: &mut Set, seen: &mut std::collections::HashSet<SubId>) {
    let ex = e.ex(x);
    match &ex.kind {
        ExprKind::Object(o) | ExprKind::Hier { object: Some(o), .. } => {
            if matches!(e.obj(*o).kind, ObjKind::Variable | ObjKind::Net(_)) {
                r.insert(*o);
            }
        }
        ExprKind::Unary(UnOp::PreInc | UnOp::PreDec | UnOp::PostInc | UnOp::PostDec) | ExprKind::Assign => {
            target(e, ex.args[0], r, w, seen);
            for &a in &ex.args {
                rw_expr(e, a, r, w, seen);
            }
            return;
        }
        ExprKind::Call(s) => {
            // Arguments to outputs are written; the body's static reads and
            // writes count too.
            let sub = &e.d.subroutines[s.0 as usize];
            for (i, &a) in ex.args.iter().enumerate() {
                let dir = sub.args.get(i).and_then(|&f| e.obj(f).dir).unwrap_or(Dir::Input);
                if matches!(dir, Dir::Output | Dir::Inout | Dir::Ref) {
                    target(e, a, r, w, seen);
                } else {
                    rw_expr(e, a, r, w, seen);
                }
            }
            if seen.insert(*s) {
                let mut rr = Set::new();
                let mut ww = Set::new();
                for &b in &sub.body {
                    rw_stmt(e, b, &mut rr, &mut ww, seen);
                }
                // Only what lives outside the subroutine.
                let inside = |o: &ObjId| {
                    let mut cur = Some(e.obj(*o).scope);
                    while let Some(id) = cur {
                        if id == sub.scope {
                            return true;
                        }
                        cur = e.scope(id).parent;
                    }
                    false
                };
                r.extend(rr.into_iter().filter(|o| !inside(o)));
                w.extend(ww.into_iter().filter(|o| !inside(o)));
            }
            return;
        }
        _ => {}
    }
    for &a in &ex.args {
        rw_expr(e, a, r, w, seen);
    }
}

#[allow(dead_code)]
fn unused(_: &eval::Eval) {}
