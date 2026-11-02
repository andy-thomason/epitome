//! Finding what a simple name means (23.9, 26.3).
//!
//! A name is looked up in the scope it is used in, then each enclosing
//! scope out to the compilation unit. In each scope, its own declarations
//! and explicit imports come first; then the packages it imports with
//! `p::*`, of which exactly one may supply the name. A name found through
//! a wildcard import becomes declared in the importing scope, so a later
//! declaration of it there is an error (26.3). Inside a class, the base
//! classes are searched before the enclosing scopes. Last comes the `std`
//! package (26.7).

use super::{Elab, R};
use crate::ast::NodeId;
use crate::design::{Binding, ClassId, Item, ScopeId, ScopeKind, Sym};

/// What `s` means from scope `sc`, ignoring ambiguity.
pub(crate) fn lookup(e: &mut Elab, sc: ScopeId, s: Sym) -> Option<Binding> {
    resolve(e, sc, s, None).ok().flatten()
}

/// What `s` means from scope `sc`. With `at`, an ambiguous wildcard import
/// is an error reported there.
pub(crate) fn resolve<'a>(e: &mut Elab<'a, '_>, sc: ScopeId, s: Sym, at: Option<NodeId>) -> R<'a, Option<Binding>> {
    let mut cur = Some(sc);
    while let Some(id) = cur {
        if let Some(b) = e.scope(id).names.get(&s).copied() {
            return Ok(Some(b));
        }
        if e.scope(id).kind == ScopeKind::Class
            && let Some(b) = in_bases(e, id, s)
        {
            return Ok(Some(b));
        }
        let wild = e.scope(id).wildcards.clone();
        let wild_at = e.scope(id).wildcard_at.clone();
        let mut found: Option<(Binding, ScopeId)> = None;
        // A reference sees only the imports written before it (26.3); a
        // reference from code elaborated later may come after all of them.
        let here = at.map(|n| e.ast.node(n).first);
        for (p, pos) in wild.into_iter().zip(wild_at) {
            if here.is_some_and(|h| pos > h) && e.ast.node(e.scope(id).node).first < pos {
                continue;
            }
            if let Some(b) = in_package(e, p, s) {
                match found {
                    Some((old, op)) if old != b => {
                        if let Some(at) = at {
                            return Err(e.err(
                                at,
                                format!(
                                    "`{}` is visible through the wildcard imports of both `{}` and `{}` (26.3)",
                                    e.name(s),
                                    e.name(e.scope(op).name.unwrap()),
                                    e.name(e.scope(p).name.unwrap())
                                ),
                            ));
                        }
                    }
                    Some(_) => {}
                    None => found = Some((b, p)),
                }
            }
        }
        if let Some((b, _)) = found {
            // Now declared here; record where, for 26.3.
            e.scope_mut(id).names.insert(s, b);
            let at = at.unwrap_or(e.scope(id).node);
            e.imported_uses.entry((id, s)).or_insert(at);
            return Ok(Some(b));
        }
        cur = e.scope(id).parent;
    }
    if let Some(std) = e.std_pkg
        && let Some(b) = e.scope(std).names.get(&s).copied()
    {
        return Ok(Some(b));
    }
    Ok(None)
}

/// A name a package makes visible: its own declarations, and what it
/// exports (26.6).
pub(crate) fn in_package(e: &mut Elab, p: ScopeId, s: Sym) -> Option<Binding> {
    // A name the package imported, explicitly or by using a wildcard
    // candidate, is visible only if exported (26.6).
    let imported_here = e.scope(p).items.iter().any(|i| matches!(i, Item::Import(_, Some(x)) if *x == s))
        || e.imported_uses.contains_key(&(p, s));
    if let Some(b) = e.scope(p).names.get(&s).copied() {
        if !imported_here || exports(e, p, s) {
            return Some(b);
        }
        return None;
    }
    None
}

/// Whether package `p` exports `s`: `export *::*`, `export q::*` or
/// `export q::s` (26.6).
fn exports(e: &Elab, p: ScopeId, s: Sym) -> bool {
    let ast = e.ast;
    for item in &e.scope(p).items {
        let Item::Opaque(n) = *item else { continue };
        if ast.kind(n) != crate::ast::Kind::PackageExport {
            continue;
        }
        // `export *::*;` has no items.
        if ast.children(n).is_empty() && ast.tokens_of(n).iter().any(|t| t.0 == "*") {
            return true;
        }
        for &it in ast.children(n) {
            let k = ast.children(it);
            if k.len() < 2 {
                continue;
            }
            let name = ast.token(k[1]).0;
            if name == "*" || e.d.syms.lookup(name) == Some(s) {
                return true;
            }
        }
    }
    false
}

/// A member of a class's base classes (8.13).
pub(crate) fn in_bases(e: &Elab, class_scope: ScopeId, s: Sym) -> Option<Binding> {
    let mut c = class_of_scope(e, class_scope)?;
    let mut guard = 0;
    while let Some(base) = e.d.classes[c.0 as usize].base {
        let bs = e.d.classes[base.0 as usize].scope;
        if let Some(b) = e.scope(bs).names.get(&s).copied() {
            return Some(b);
        }
        c = base;
        guard += 1;
        if guard > 64 {
            break;
        }
    }
    None
}

pub(crate) fn class_of_scope(e: &Elab, sc: ScopeId) -> Option<ClassId> {
    e.d.classes.iter().position(|c| c.scope == sc).map(|i| ClassId(i as u32))
}

/// A member of a class or its bases, by name.
pub(crate) fn class_member(e: &Elab, c: ClassId, s: Sym) -> Option<Binding> {
    let sc = e.d.classes[c.0 as usize].scope;
    if let Some(b) = e.scope(sc).names.get(&s).copied() {
        return Some(b);
    }
    in_bases(e, sc, s)
}

/// The innermost enclosing scope of kind `k`.
pub(crate) fn enclosing(e: &Elab, sc: ScopeId, k: ScopeKind) -> Option<ScopeId> {
    let mut cur = Some(sc);
    while let Some(id) = cur {
        if e.scope(id).kind == k {
            return Some(id);
        }
        cur = e.scope(id).parent;
    }
    None
}
