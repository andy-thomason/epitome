//! Resolving `use` declarations to a fixed point, and finding each crate's
//! prelude.

use super::{ImportKind, ModId, Ns, Res, Resolver};
use crate::lexer::Edition;

/// What trying an import gave.
enum Outcome {
    Done,
    /// Not yet: something it needs may come from another import.
    Wait,
}

pub(super) fn resolve_imports(r: &mut Resolver) {
    let mut pending: Vec<usize> = (0..r.imports.len()).collect();
    loop {
        let before = pending.len();
        let mut still = Vec::new();
        for i in pending {
            match try_import(r, i) {
                Outcome::Done => {}
                Outcome::Wait => still.push(i),
            }
        }
        pending = still;
        if pending.len() == before {
            break;
        }
    }
    // Imports from a proc-macro library (`use test_macros::Derive;`) are of
    // macros, which expansion finds by name: nothing to bind here.
    pending.retain(|&i| {
        let imp = &r.imports[i];
        !imp.path.first().is_some_and(|first| r.crate_proc_macros[imp.krate as usize].iter().any(|p| p == first))
    });
    r.failed_imports = pending;
    find_preludes(r);
}

impl<'s> Resolver<'s> {
    /// The crate root module of a crate named `name` in the session.
    pub(super) fn crate_root(&self, name: &str) -> Option<ModId> {
        self.crate_names.iter().position(|c| c == name).map(|k| self.roots[k])
    }

    /// A crate in crate `krate`'s extern prelude: a crate of the session by
    /// its name (one of its deps, if it has a list of them), or an `extern
    /// crate` (perhaps renamed) at the crate root.
    pub(super) fn extern_prelude(&self, krate: u32, name: &str) -> Option<ModId> {
        let root = self.roots[krate as usize];
        if let Some(b) = self.modules[root.0 as usize].names.get(&(name, Ns::Type))
            && let Res::Def(d) = b.res
            && self.defs[d.0 as usize].parent.is_none()
        {
            return self.defs[d.0 as usize].module;
        }
        if let Some(deps) = &self.crate_deps[krate as usize]
            && !deps.iter().any(|d| d == name)
        {
            return None;
        }
        self.crate_root(name)
    }

    /// The module a path's first segment starts from, for a path written
    /// in module `m` of crate `krate`: `crate`, `self`, `super`, an extern
    /// crate, or (2018 on) a name in `m`. Returns the module, and how many
    /// segments it used.
    pub(super) fn path_start(
        &self,
        path: &[&str],
        rooted: bool,
        m: ModId,
        krate: u32,
        edition: Edition,
    ) -> Option<(ModId, usize)> {
        let normal = self.modules[m.0 as usize].normal;
        let first = *path.first()?;
        if rooted {
            // `::a`: a crate in 2018 on; the crate root in 2015.
            return if edition >= Edition::E2018 {
                Some((self.extern_prelude(krate, first)?, 1))
            } else {
                Some((self.roots[krate as usize], 0))
            };
        }
        match first {
            "crate" => Some((self.roots[krate as usize], 1)),
            "self" => Some((normal, 1)),
            "super" => {
                let mut cur = normal;
                let mut used = 0;
                while path.get(used) == Some(&"super") {
                    cur = self.modules[cur.0 as usize].parent.map(|p| self.modules[p.0 as usize].normal)?;
                    used += 1;
                }
                Some((cur, used))
            }
            _ if edition < Edition::E2018 => Some((self.roots[krate as usize], 0)),
            _ => {
                // A name in scope here, else a crate.
                if let Some(res) =
                    self.lookup(m, first, Ns::Type, m).or_else(|| self.lookup(normal, first, Ns::Type, m))
                    && let Some(md) = self.module_of_res(&res)
                {
                    return Some((md, 1));
                }
                if let Some(c) = self.extern_prelude(krate, first) {
                    return Some((c, 1));
                }
                // A name of the standard prelude (`use Default::default;`).
                let p = prelude_module(self, krate as usize)?;
                let res = self.lookup(p, first, Ns::Type, m)?;
                Some((self.module_of_res(&res)?, 1))
            }
        }
    }

    /// Follows module names from `start` along `segs`.
    pub(super) fn follow(&self, start: ModId, segs: &[&str], from: ModId) -> Option<ModId> {
        let mut cur = start;
        for &s in segs {
            let res = self.lookup(cur, s, Ns::Type, from)?;
            cur = self.module_of_res(&res)?;
        }
        Some(cur)
    }
}

fn try_import(r: &mut Resolver, i: usize) -> Outcome {
    let imp = &r.imports[i];
    let (m, krate, edition, rooted, vis) = (imp.module, imp.krate, imp.edition, imp.rooted, imp.vis);
    let path = imp.path.clone();
    match imp.kind {
        ImportKind::Glob => {
            let Some((start, used)) = r.path_start(&path, rooted, m, krate, edition) else { return Outcome::Wait };
            let Some(target) = r.follow(start, &path[used..], m) else { return Outcome::Wait };
            let globs = &mut r.modules[m.0 as usize].globs;
            if target != m && !globs.iter().any(|g| g.0 == target) {
                globs.push((target, vis));
            }
            Outcome::Done
        }
        ImportKind::Single(name) => {
            // `use a;` with one segment: a crate, or a name here.
            let (found, last) = if path.len() == 1 && !matches!(path[0], "crate" | "self" | "super") {
                let n = path[0];
                let mut found: Vec<(Ns, Res)> = Vec::new();
                for ns in [Ns::Type, Ns::Value, Ns::Macro] {
                    let normal = r.modules[m.0 as usize].normal;
                    if let Some(res) = r.lookup(normal, n, ns, m) {
                        found.push((ns, res));
                    }
                }
                // 2015: a name at the crate root (`use Foo;` in a module).
                if found.is_empty() && edition < Edition::E2018 {
                    let root = r.roots[krate as usize];
                    for ns in [Ns::Type, Ns::Value, Ns::Macro] {
                        if let Some(res) = r.lookup(root, n, ns, m) {
                            found.push((ns, res));
                        }
                    }
                }
                if found.is_empty()
                    && let Some(root) = r.extern_prelude(krate, n)
                {
                    found.push((Ns::Type, Res::Def(r.modules[root.0 as usize].def.unwrap())));
                }
                // The standard prelude (`use Ok;`).
                if found.is_empty()
                    && let Some(p) = prelude_module(r, krate as usize)
                {
                    for ns in [Ns::Type, Ns::Value, Ns::Macro] {
                        if let Some(res) = r.lookup(p, n, ns, m) {
                            found.push((ns, res));
                        }
                    }
                }
                // `use bool;` (as `core::primitive` does).
                if found.is_empty()
                    && let Some(&p) = super::PRIMITIVES.iter().find(|&&p| p == n)
                {
                    found.push((Ns::Type, Res::PrimTy(p)));
                }
                (found, n)
            } else {
                let Some((start, used)) = r.path_start(&path, rooted, m, krate, edition) else { return Outcome::Wait };
                if used == path.len() {
                    // `use crate as c;`, `use super;`: the module itself.
                    let Some(def) = r.modules[start.0 as usize].def else { return Outcome::Wait };
                    (vec![(Ns::Type, Res::Def(def))], *path.last().unwrap())
                } else {
                    let Some(target) = r.follow(start, &path[used..path.len() - 1], m) else { return Outcome::Wait };
                    let last = *path.last().unwrap();
                    let mut found = Vec::new();
                    for ns in [Ns::Type, Ns::Value, Ns::Macro] {
                        if let Some(res) = r.lookup(target, last, ns, m) {
                            found.push((ns, res));
                        }
                    }
                    (found, last)
                }
            };
            let _ = last;
            if found.is_empty() {
                return Outcome::Wait;
            }
            for (ns, res) in found {
                r.bind(m, name, ns, res, vis);
            }
            Outcome::Done
        }
    }
}

/// Each crate's prelude: what `#[prelude_import]` imports, or else
/// `std::prelude::rust_20xx` (`core`'s with `no_std`).
fn find_preludes(r: &mut Resolver) {
    for k in 0..r.roots.len() {
        r.preludes[k] = prelude_module(r, k);
    }
}

/// Crate `k`'s prelude module: its `#[prelude_import]`'s, or
/// `std::prelude::rust_20xx` (`core`'s with `no_std`).
fn prelude_module(r: &Resolver, k: usize) -> Option<ModId> {
    if let Some(i) = r.prelude_imports[k] {
        let imp = &r.imports[i];
        let (start, used) = r.path_start(&imp.path, imp.rooted, imp.module, imp.krate, imp.edition)?;
        return r.follow(start, &imp.path[used..], imp.module);
    }
    if r.no_core[k] {
        return None;
    }
    let from = if r.no_std[k] { "core" } else { "std" };
    let root = r.crate_root(from).or_else(|| r.crate_root("core"))?;
    let edition = match r.editions[k] {
        Edition::E2015 => "rust_2015",
        Edition::E2018 => "rust_2018",
        Edition::E2021 => "rust_2021",
        Edition::E2024 => "rust_2024",
    };
    r.follow(root, &["prelude", edition], root)
}
