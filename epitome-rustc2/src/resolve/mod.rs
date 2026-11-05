//! Name resolution: what every path in every crate refers to.
//!
//! 1. **Collect** (`collect.rs`): every item becomes a `Def`, and modules
//!    are made: the crate root, each `mod`, each `enum` (its variants),
//!    each `trait` (its items), and each block that holds items. A module
//!    has names in three namespaces (types, values, macros) and a list of
//!    glob imports. `use` declarations become `Import`s.
//! 2. **Imports** (`imports.rs`): imports are resolved in rounds until a
//!    round resolves nothing new. A glob is an edge to another module,
//!    whose names are looked up when asked for, so names a module defines
//!    or imports by name always come before glob-imported ones.
//! 3. **Paths** (`paths.rs`): every path in every file is resolved, in
//!    parallel over files, with scopes for local variables and generic
//!    parameters. The result is a side table from the path's `NodeId` to a
//!    `Res`; the tree is not changed.
//!
//! A path that reaches a type and goes on (`Vec::new`, `T::Item`,
//! `<T as Tr>::f`) resolves as far as the type; the rest is left to type
//! checking (`Res::Partial`).

mod collect;
mod imports;
mod paths;

use crate::ast::NodeId;
use crate::hash::FastMap;
use crate::lexer::Edition;
use crate::session::{ParsedCrate, Session};

/// The built-in types.
pub(crate) const PRIMITIVES: &[&str] = &[
    "bool", "char", "str", "u8", "u16", "u32", "u64", "u128", "usize", "i8", "i16", "i32", "i64", "i128", "isize",
    "f16", "f32", "f64", "f128",
];

/// A definition: a crate's index into `Resolver::defs`.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, PartialOrd, Ord)]
pub struct DefId(pub u32);

/// A module (or anything with names inside: an enum, a trait, a block):
/// an index into `Resolver::modules`.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct ModId(pub u32);

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum DefKind {
    /// A module, including a crate's root.
    Mod,
    Struct,
    Union,
    Enum,
    Variant,
    Trait,
    TraitAlias,
    TypeAlias,
    Fn,
    Const,
    Static,
    /// `macro_rules!`, or a macro 2.0.
    Macro,
    /// A function, static or type in an `extern` block.
    ForeignFn,
    ForeignStatic,
    ForeignType,
    /// Items of a trait or an impl.
    AssocFn,
    AssocConst,
    AssocType,
    /// An `impl` block: no name, but a scope for `Self`.
    Impl,
}

/// Which namespace a name is in.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Ns {
    Type,
    Value,
    Macro,
}

/// Who may see a name.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Vis {
    Public,
    /// Visible in this module and the modules inside it.
    Restricted(ModId),
}

pub struct Def<'s> {
    pub kind: DefKind,
    pub name: &'s str,
    pub krate: u32,
    /// The module it is in (for the crate root, none).
    pub parent: Option<ModId>,
    /// Where it is: the file (an index into the crate's files) and node.
    pub file: u32,
    pub node: NodeId,
    pub vis: Vis,
    /// The module it makes, for a `mod`, `enum`, `trait` or block.
    pub module: Option<ModId>,
}

/// What a name or path refers to.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Res {
    Def(DefId),
    /// A local variable: the pattern node that binds it.
    Local(NodeId),
    /// A generic parameter: its node.
    Generic(NodeId),
    /// A built-in type: `u8`, `str`, `bool`.
    PrimTy(&'static str),
    /// `Self` in an impl or trait: that item's node, in that file.
    SelfTy(u32, NodeId),
    /// A path resolved as far as `base`; its last `rest` segments are
    /// associated items, for type checking to find.
    Partial(Box<Res>, u32),
}

/// One name in a module.
#[derive(Clone, Debug)]
pub struct Binding {
    pub res: Res,
    pub vis: Vis,
}

pub struct Module<'s> {
    pub krate: u32,
    pub def: Option<DefId>,
    pub parent: Option<ModId>,
    /// The module paths and names resolve from: itself for a `mod`, the
    /// enclosing `mod` for an enum, trait or block.
    pub normal: ModId,
    /// Whether this is a block (its names are seen only lexically).
    pub block: bool,
    pub names: FastMap<(&'s str, Ns), Binding>,
    /// Glob imports: the module, and the visibility the import gives.
    pub globs: Vec<(ModId, Vis)>,
}

/// A `use` to resolve.
pub struct Import<'s> {
    /// The module the `use` is in.
    pub module: ModId,
    pub krate: u32,
    /// The path's segments, after any leading `::`.
    pub path: Vec<&'s str>,
    /// `use ::a::b`.
    pub rooted: bool,
    pub kind: ImportKind<'s>,
    pub vis: Vis,
    /// Where it is, to record what it resolved to.
    pub file: u32,
    pub node: NodeId,
    pub edition: Edition,
}

pub enum ImportKind<'s> {
    /// `use a::b;` or `use a::b as c;`: the name it binds (`_` binds none).
    Single(&'s str),
    /// `use a::*;`
    Glob,
}

/// Everything name resolution found.
pub struct Resolver<'s> {
    pub defs: Vec<Def<'s>>,
    pub modules: Vec<Module<'s>>,
    /// Each crate's root module.
    pub roots: Vec<ModId>,
    /// Crate names, for the extern prelude.
    pub crate_names: Vec<String>,
    pub editions: Vec<Edition>,
    /// Each crate's std prelude module, if it has one.
    pub preludes: Vec<Option<ModId>>,
    /// The module each file's top level is: per crate, per file.
    pub file_modules: Vec<Vec<ModId>>,
    /// Modules made by `mod`, `enum`, `trait` and blocks, by where they are.
    pub module_of: FastMap<(u32, u32, NodeId), ModId>,
    pub imports: Vec<Import<'s>>,
    /// Imports that never resolved.
    pub failed_imports: Vec<usize>,
    /// Each crate's `#[prelude_import]` import, if it has one.
    pub prelude_imports: Vec<Option<usize>>,
    /// Whether each crate has `#![no_core]` or `#![no_std]`.
    pub no_core: Vec<bool>,
    pub no_std: Vec<bool>,
    /// What each path resolved to: per crate, per file, indexed by node.
    pub res: Vec<Vec<Vec<Option<Res>>>>,
    /// The text of each macro 2.0's body (start and end addresses) and the
    /// module it is defined in, sorted: a path whose first token is in such
    /// text resolves in that module (definition-site hygiene).
    pub def_site: Vec<(usize, usize, ModId)>,
    /// The text of every macro body (`macro_rules!` and macro 2.0), sorted:
    /// a token's syntax context for local variables is the body it is in.
    pub macro_bodies: Vec<(usize, usize)>,
}

/// How many paths resolved, for progress reports.
#[derive(Default, Debug)]
pub struct ResolveStats {
    pub defs: usize,
    pub modules: usize,
    pub imports: usize,
    pub imports_failed: usize,
    pub paths: usize,
    pub resolved: usize,
    /// Resolved as far as a type, the rest left to type checking.
    pub partial: usize,
    pub unresolved: usize,
    /// The first segments of unresolved paths, most common first.
    pub unresolved_names: Vec<(String, usize)>,
}

/// The module tree and imports of `crates`, without resolving paths in
/// bodies: what macro expansion needs to find macros by path and `use`.
pub(crate) fn module_tree<'s>(crates: &[ParsedCrate<'s>]) -> Resolver<'s> {
    let mut r = collect::collect(crates);
    imports::resolve_imports(&mut r);
    r
}

impl Session {
    /// Resolves every name in `crates`.
    pub fn resolve<'s>(&'s self, crates: &[ParsedCrate<'s>]) -> Resolver<'s> {
        let mut r = collect::collect(crates);
        imports::resolve_imports(&mut r);
        paths::resolve_paths(self, &mut r, crates);
        r
    }
}

impl<'s> Resolver<'s> {
    /// What path (or pattern) `node` of file `file` of crate `krate`
    /// resolved to.
    pub fn res_of(&self, krate: usize, file: usize, node: NodeId) -> Option<&Res> {
        self.res.get(krate)?.get(file)?.get(node.0 as usize)?.as_ref()
    }

    /// A definition's path: `core::option::Option`.
    pub fn def_path(&self, d: DefId) -> String {
        let def = &self.defs[d.0 as usize];
        let mut parts = vec![def.name.to_string()];
        let mut m = def.parent;
        while let Some(id) = m {
            let module = &self.modules[id.0 as usize];
            if module.block {
                parts.push("{block}".to_string());
            } else if let Some(md) = module.def {
                let md = &self.defs[md.0 as usize];
                if md.parent.is_none() {
                    parts.push(self.crate_names[md.krate as usize].clone());
                    break;
                }
                parts.push(md.name.to_string());
            }
            m = module.parent;
        }
        parts.reverse();
        parts.join("::")
    }

    /// A resolution as text, for dumps.
    pub fn describe(&self, res: &Res) -> String {
        match res {
            Res::Def(d) => format!("{:?} {}", self.defs[d.0 as usize].kind, self.def_path(*d)),
            Res::Local(n) => format!("local #{}", n.0),
            Res::Generic(n) => format!("generic #{}", n.0),
            Res::PrimTy(t) => format!("builtin {t}"),
            Res::SelfTy(_, n) => format!("Self of #{}", n.0),
            Res::Partial(base, rest) => format!("{} + {rest} associated", self.describe(base)),
        }
    }

    /// The imports that did not resolve, as `module: use path`.
    pub fn failed_import_lines(&self) -> Vec<String> {
        self.failed_imports
            .iter()
            .map(|&i| {
                let imp = &self.imports[i];
                let module = self.modules[imp.module.0 as usize].normal;
                let at = self.modules[module.0 as usize].def.map_or_else(|| "?".to_string(), |d| self.def_path(d));
                let what = match imp.kind {
                    ImportKind::Glob => "*".to_string(),
                    ImportKind::Single(n) => format!("(as {n})"),
                };
                format!("{at}: use {}{} {what}", if imp.rooted { "::" } else { "" }, imp.path.join("::"))
            })
            .collect()
    }

    /// Whether a name with `vis` can be seen from module `from`.
    pub fn visible(&self, vis: Vis, from: ModId) -> bool {
        match vis {
            Vis::Public => true,
            Vis::Restricted(m) => {
                if self.modules[m.0 as usize].krate != self.modules[from.0 as usize].krate {
                    return false;
                }
                let mut cur = Some(from);
                while let Some(c) = cur {
                    if c == m {
                        return true;
                    }
                    cur = self.modules[c.0 as usize].parent;
                }
                false
            }
        }
    }

    /// The name `name` in `ns` in module `m`, as seen from `from`: its own
    /// names, then its globs' (not following a glob twice).
    pub fn lookup(&self, m: ModId, name: &str, ns: Ns, from: ModId) -> Option<Res> {
        let mut seen = Vec::new();
        self.lookup_in(m, name, ns, from, &mut seen)
    }

    fn lookup_in(&self, m: ModId, name: &str, ns: Ns, from: ModId, seen: &mut Vec<ModId>) -> Option<Res> {
        if seen.contains(&m) {
            return None;
        }
        seen.push(m);
        let module = &self.modules[m.0 as usize];
        if let Some(b) = module.names.get(&(name, ns))
            && self.visible(b.vis, from)
        {
            return Some(b.res.clone());
        }
        for &(g, vis) in &module.globs {
            if !self.visible(vis, from) {
                continue;
            }
            if let Some(r) = self.lookup_in(g, name, ns, from, seen) {
                return Some(r);
            }
        }
        None
    }

    /// The module a path whose first token is `t` resolves in, if `t` is
    /// part of a macro 2.0's body.
    pub fn def_site_module(&self, t: &str) -> Option<ModId> {
        let p = t.as_ptr() as usize;
        let i = self.def_site.partition_point(|r| r.0 <= p);
        let (s, e, m) = *self.def_site.get(i.checked_sub(1)?)?;
        (p >= s && p < e).then_some(m)
    }

    /// The syntax context of a token for local variables: 0 for code
    /// written where it is, else 1 + the index of the macro body it was
    /// written in. A local bound in one context is not seen from another
    /// (rustc's hygiene for locals, in `macro_rules!` and macros 2.0).
    pub fn ctx_of(&self, t: &str) -> u32 {
        let p = t.as_ptr() as usize;
        let i = self.macro_bodies.partition_point(|r| r.0 <= p);
        match i.checked_sub(1).map(|j| (j, self.macro_bodies[j])) {
            Some((j, (s, e))) if p >= s && p < e => j as u32 + 1,
            _ => 0,
        }
    }

    /// The macro a call `segs!` in module `m` of crate `krate` names, by
    /// path and imports (text-order `macro_rules!` scope is expansion's).
    pub(crate) fn resolve_macro(&self, krate: u32, m: ModId, segs: &[&str], rooted: bool) -> Option<DefId> {
        let res = if segs.len() == 1 && !rooted {
            let name = segs[0];
            let mut cur = m;
            let mut found = None;
            loop {
                if let Some(r) = self.lookup(cur, name, Ns::Macro, m) {
                    found = Some(r);
                    break;
                }
                let module = &self.modules[cur.0 as usize];
                match module.parent {
                    Some(p) if module.block || module.normal != cur => cur = p,
                    _ => break,
                }
            }
            found
                .or_else(|| self.preludes[krate as usize].and_then(|p| self.lookup(p, name, Ns::Macro, m)))
                .or_else(|| self.roots.iter().find_map(|&root| self.lookup(root, name, Ns::Macro, root)))?
        } else {
            let edition = self.editions[krate as usize];
            let (start, used) = self.path_start(segs, rooted, m, krate, edition)?;
            if used >= segs.len() {
                return None;
            }
            let target = self.follow(start, &segs[used..segs.len() - 1], m)?;
            self.lookup(target, segs[segs.len() - 1], Ns::Macro, m)?
        };
        match res {
            Res::Def(d) if self.defs[d.0 as usize].kind == DefKind::Macro => Some(d),
            _ => None,
        }
    }

    /// The module a resolution names, if it names one (a module, enum,
    /// trait, or a crate root).
    pub fn module_of_res(&self, res: &Res) -> Option<ModId> {
        match res {
            Res::Def(d) => self.defs[d.0 as usize].module,
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::session::{CrateRoot, Options, Session};

    /// Resolves `src` as a crate `t` (one file) and returns the dump.
    fn dump(src: &str) -> String {
        let dir = std::env::temp_dir().join(format!("epitome-rustc2-res-{}-{}", std::process::id(), src.len()));
        std::fs::create_dir_all(&dir).unwrap();
        let file = dir.join("lib.rs");
        std::fs::write(&file, src).unwrap();
        let session = Session::new(Options { threads: 2, ..Default::default() });
        let mut crates = session.parse_crates(vec![CrateRoot::new("t", &file)]);
        session.expand(&mut crates);
        assert!(crates[0].errors.is_empty());
        let r = session.resolve(&crates);
        let _ = std::fs::remove_dir_all(dir);
        // Without positions: `path -> resolution`.
        r.dump(&crates, &session.sources)
            .lines()
            .map(|l| l.split_once(' ').map_or(l, |x| x.1).to_string())
            .collect::<Vec<_>>()
            .join("\n")
    }

    #[test]
    fn modules_imports_and_scopes() {
        let d = dump(
            "mod a {\n\
                 pub struct S;\n\
                 pub enum E { X, Y(u8) }\n\
                 pub fn f() -> u32 { 1 }\n\
                 pub mod b { pub use super::S as T; pub fn g() {} }\n\
                 pub use self::b::*;\n\
             }\n\
             use a::{E, f};\n\
             const C: u32 = 3;\n\
             trait Tr {}\n\
             fn h<T: Tr>(x: T, n: u32) -> u32 {\n\
                 let s: a::T = a::S;\n\
                 a::g();\n\
                 let y = n + C + f();\n\
                 let v = match E::Y(1) { E::Y(z) => z as u32, E::X => y };\n\
                 let x2: T = x;\n\
                 fn inner() -> u32 { u32::MAX }\n\
                 v + inner()\n\
             }",
        );
        for want in [
            "a::T -> Struct t::a::S",
            "a::S -> Struct t::a::S",
            "a::g -> Fn t::a::b::g",
            "C -> Const t::C",
            "f -> Fn t::a::f",
            "E::Y -> Variant t::a::E::Y",
            "E::X -> Variant t::a::E::X",
            "u32::MAX -> builtin u32 + 1 associated",
            "inner -> Fn t::{block}::inner",
            "Tr -> Trait t::Tr",
        ] {
            assert!(d.lines().any(|l| l == want), "no `{want}` in:\n{d}");
        }
        // Locals and generics.
        assert!(d.lines().any(|l| l.starts_with("z -> local")), "{d}");
        assert!(d.lines().any(|l| l.starts_with("x -> local")), "{d}");
        assert!(d.lines().any(|l| l.starts_with("T -> generic")), "{d}");
        assert!(!d.contains("??"), "{d}");
    }

    #[test]
    fn pub_in_path() {
        let session = Session::new(Options { threads: 1, ..Default::default() });
        let dir = std::env::temp_dir().join(format!("epitome-rustc2-vis-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let file = dir.join("lib.rs");
        std::fs::write(&file, "mod a { pub mod b { pub(in crate::a) fn f() {} pub(super) fn g() {} } }").unwrap();
        let crates = session.parse_crates(vec![CrateRoot::new("t", &file)]);
        let r = session.resolve(&crates);
        let _ = std::fs::remove_dir_all(dir);
        let def = |name: &str| r.defs.iter().find(|d| d.name == name).unwrap();
        let module_named = |name: &str| def(name).module.unwrap();
        let a = module_named("a");
        assert_eq!(def("f").vis, super::Vis::Restricted(a));
        assert_eq!(def("g").vis, super::Vis::Restricted(a));
    }

    #[test]
    fn macro_locals_are_hygienic() {
        let d = dump("macro_rules! m { ($e:expr) => {{ let x = 2; $e + x }} }\nfn f() -> i32 { let x = 1; m!(x) }");
        // Uses of `x`: the argument (the caller's `x`) and the macro's own.
        let xs: Vec<&str> = d.lines().filter(|l| l.starts_with("x -> local")).collect();
        assert_eq!(xs.len(), 2, "{d}");
        assert_ne!(xs[0], xs[1], "{d}");
        // The caller's `x` is bound first, so it has the smaller node.
        let node = |l: &str| l.rsplit('#').next().unwrap().parse::<u32>().unwrap();
        assert!(node(xs[0]) < node(xs[1]), "the argument should be the caller's `x`:\n{d}");
    }

    #[test]
    fn nested_items_do_not_see_locals() {
        let d = dump("fn f() { let x = 1; fn g() -> u32 { x } }");
        assert!(d.lines().any(|l| l == "x -> ??"), "{d}");
    }

    #[test]
    fn identifier_patterns_name_constants() {
        let d = dump("const K: u32 = 1; fn f(v: u32) -> bool { match v { K => true, other => other == 2 } }");
        assert!(d.lines().any(|l| l.starts_with("other -> local")), "{d}");
        // `K` in the pattern is the constant (recorded on the pattern).
        let r = d.lines().filter(|l| l.starts_with("K ->")).count();
        assert_eq!(r, 0, "{d}");
    }
}
