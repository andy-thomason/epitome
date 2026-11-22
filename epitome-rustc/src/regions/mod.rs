//! Lifetimes: what `ty` leaves out, kept for borrow checking.
//!
//! `Ty` has no regions. This module reads them from the source where they
//! are written, in item signatures (milestone 4 of `docs/borrowck.md`):
//!
//! - `variance`: how each generic parameter of a struct, enum, union and
//!   alias varies, as `#[rustc_dump_variances]` prints it.
//! - `sig`: a function's lifetime parameters (early or late bound), its
//!   outlives bounds, and its parameter and return types with a region in
//!   each reference and lifetime argument, after elision.

pub mod sig;
pub mod variance;

use crate::resolve::{DefId, Resolver};
use crate::session::{ParsedCrate, Session};
use crate::ty::TyCtxt;

/// A line `#[rustc_dump_variances]` prints.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VarianceDump {
    pub krate: u32,
    pub file: String,
    pub line: usize,
    pub def: DefId,
    /// `['a: +, T: o]`.
    pub text: String,
}

impl Session {
    /// The variance of every generic parameter of the structs, enums,
    /// unions and aliases of `crates`.
    pub fn variances<'s>(&'s self, crates: &[ParsedCrate<'s>], r: &Resolver<'s>, tcx: &TyCtxt<'s>) -> variance::Variances {
        variance::compute(crates, r, tcx)
    }

    /// What rustc's `#[rustc_dump_variances]` prints for the items of the
    /// crates `only` names that carry it.
    pub fn dump_variances<'s>(&'s self, crates: &[ParsedCrate<'s>], r: &Resolver<'s>, tcx: &TyCtxt<'s>, only: &[u32]) -> Vec<VarianceDump> {
        let v = self.variances(crates, r, tcx);
        let mut out = Vec::new();
        for (i, def) in r.defs.iter().enumerate() {
            if !only.contains(&def.krate) {
                continue;
            }
            let ast = &crates[def.krate as usize].files[def.file as usize].ast;
            if !crate::session::has_attribute(ast, def.node, "rustc_dump_variances") {
                continue;
            }
            let d = DefId(i as u32);
            let Some(text) = v.show(d) else { continue };
            let at = ast.child(def.node, crate::ast::Kind::Name).unwrap_or(def.node);
            let tok = ast.token_origin(ast.node(at).first as usize).0;
            let loc = self.sources.find(tok).map(|s| s.locate(tok));
            let (file, line) = loc.map_or((String::new(), 0), |l| (l.name, l.line));
            out.push(VarianceDump { krate: def.krate, file, line, def: d, text });
        }
        out.sort_by(|a, b| (a.krate, &a.file, a.line).cmp(&(b.krate, &b.file, b.line)));
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::session::{CrateRoot, Options};

    fn core_src() -> Option<std::path::PathBuf> {
        let out = std::process::Command::new("rustc").args(["--print", "sysroot"]).output().ok()?;
        let root = std::path::PathBuf::from(String::from_utf8(out.stdout).ok()?.trim());
        let core = root.join("lib/rustlib/src/rust/library/core/src/lib.rs");
        core.exists().then_some(core)
    }

    /// Runs `f` on a file of `tests/borrowck`, with `core` if there is one.
    fn with_crate<T>(core: bool, file: Option<&str>, f: impl FnOnce(&Session, &[ParsedCrate], &Resolver, &TyCtxt) -> T) -> Option<T> {
        let session = Session::new(Options { threads: 4, cfg: Some(crate::cfg::CfgSet::host()), ..Default::default() });
        let mut roots = Vec::new();
        if core {
            roots.push(CrateRoot::new("core", core_src()?));
        }
        if let Some(file) = file {
            let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(file);
            let name = path.file_stem().unwrap().to_string_lossy().into_owned();
            roots.push(CrateRoot::new(&name, &path));
        }
        let mut crates = session.parse_crates(roots);
        session.expand(&mut crates);
        let r = session.resolve(&crates);
        let tcx = session.collect_types(&crates, &r);
        Some(f(&session, &crates, &r, &tcx))
    }

    fn find(r: &Resolver, name: &str) -> DefId {
        let hits: Vec<usize> = (0..r.defs.len()).filter(|&i| r.def_path(DefId(i as u32)).ends_with(name)).collect();
        assert_eq!(hits.len(), 1, "{name}: {hits:?}");
        DefId(hits[0] as u32)
    }

    #[test]
    fn signatures_keep_their_regions() {
        with_crate(false, Some("tests/borrowck/regions.rs"), |_, crates, r, tcx| {
            let show = |name: &str| sig::fn_regions(crates, r, tcx, find(r, name)).unwrap().show(r, tcx);
            for (name, want) in [
                // Named, and in an argument only: late bound.
                ("::explicit", "fn<'a(late), 'b(late)>(&'a _, &'b _) -> &'a _"),
                // Elided in an argument: its own; in the result: the only one.
                ("::elided_one", "fn<>(&'_0 _) -> &'_0 _"),
                ("::elided_adt", "fn<>(Wrap<'_0, _>) -> &'_0 _"),
                ("::hidden_lifetime", "fn<>(Wrap<'_0, _>) -> &'_0 _"),
                // Named in a bound: a parameter of the function.
                ("::early_bound", "fn<'a, 'b>(&'a _, &'b _) -> &'b _ where 'a: 'b"),
                ("::outlives_where", "fn<'a, 'b>(&'a _, &'b _) -> &'a _ where 'b: 'a"),
                ("::ty_bound", "fn<'a>(&'a T) -> &'a T where T: 'a"),
                ("::static_out", "fn<>(&'_0 _) -> &'static _"),
                // `&'a dyn Trait` is `&'a (dyn Trait + 'a)`, `Boxed<dyn Trait>` is `'static`.
                ("::dyn_default", "fn<'a(late)>(&'a dyn Show<> + 'a) -> &'a _"),
                ("::boxed", "fn<>(Boxed<dyn Show<> + 'static>) -> _"),
                ("::alias", "fn<'a(late)>(Wrap<'a, _>) -> &'a _"),
                // A function pointer binds its own.
                ("::fn_ptr", "fn<>(for<'^0> fn(&'^0 _) -> &'^0 _) -> ()"),
                ("::two_inputs", "fn<>(&'_0 _, &'_1 _) -> _"),
                // Methods: `&self`'s lifetime is the result's; an impl's are parameters.
                ("::by_ref", "fn<>(&'_0 S<>, &'_1 _) -> &'_0 _"),
                ("::by_mut", "fn<>(&'_0 mut S<>) -> &'_0 mut S<>"),
                ("::chosen", "fn<'a(late)>(&'_0 S<>, &'a _) -> &'a _"),
                ("::get", "fn<'h(parent)>(&'_0 Holder<'h>) -> &'h _"),
                ("::me", "fn<'h(parent)>(&'_0 Holder<'h>) -> Holder<'_0>"),
                ("::this", "fn<'h(parent)>(Holder<'h>) -> Holder<'h>"),
            ] {
                assert_eq!(show(name), want, "{name}");
            }
            // `F: for<'x> Fn(&'x u8) -> &'x u8`.
            let f = sig::fn_regions(crates, r, tcx, find(r, "::higher_ranked")).unwrap();
            let [(_, b)] = f.bounds.as_slice() else { panic!("{:?}", f.bounds) };
            assert_eq!(b.binder.len(), 1);
            assert_eq!(b.assoc.len(), 1);
            assert_eq!(b.assoc[0].1, sig::RTy::Ref(sig::Region::Bound(b.binder[0]), crate::ty::Mutbl::Not, Box::new(sig::RTy::Plain)));
            // Fields keep theirs.
            let wrap = sig::adt_fields(crates, r, tcx, find(r, "::Wrap")).unwrap();
            assert_eq!(wrap.len(), 1);
            assert!(matches!(&wrap[0][0], sig::RTy::Ref(sig::Region::Param(0), crate::ty::Mutbl::Not, _)), "{:?}", wrap);
        })
        .unwrap_or_else(|| eprintln!("no rust-src: skipped"));
    }

    /// Every function and type of `core` has its signature read, and no
    /// elided lifetime of a result is left without one to be.
    #[test]
    fn core_signatures_are_read() {
        let Some(()) = with_crate(true, None, |_, crates, r, tcx| {
            let (mut fns, mut unknown, mut shown) = (0, 0, Vec::new());
            for i in 0..r.defs.len() {
                let d = DefId(i as u32);
                if r.defs[i].krate != 0 {
                    continue;
                }
                if let Some(f) = sig::fn_regions(crates, r, tcx, d) {
                    fns += 1;
                    let mut regs = Vec::new();
                    f.output.regions(&mut regs);
                    for t in &f.inputs {
                        t.regions(&mut regs);
                    }
                    if regs.contains(&sig::Region::Unknown) {
                        unknown += 1;
                        if shown.len() < 10 {
                            shown.push(format!("{}: {}", r.def_path(d), f.show(r, tcx)));
                        }
                    }
                }
                sig::adt_fields(crates, r, tcx, d);
            }
            assert!(fns > 5000, "{fns} functions");
            assert_eq!(unknown, 0, "{unknown} of {fns} have a region left out, such as:\n{}", shown.join("\n"));
        }) else {
            eprintln!("no rust-src: skipped");
            return;
        };
    }

    /// The variances of types of `core` that everyone knows.
    #[test]
    fn core_variances_are_known() {
        let Some(()) = with_crate(true, None, |session, crates, r, tcx| {
            let v = session.variances(crates, r, tcx);
            let show = |name: &str| v.show(find(r, name)).unwrap();
            for (name, want) in [
                ("core::option::Option", "[T: +]"),
                ("core::cell::Cell", "[T: o]"),
                ("core::cell::UnsafeCell", "[T: o]"),
                ("core::cell::RefCell", "[T: o]"),
                ("core::marker::PhantomData", "[T: +]"),
                ("core::slice::iter::Iter", "['a: +, T: +]"),
                ("core::slice::iter::IterMut", "['a: +, T: o]"),
                ("core::result::Result", "[T: +, E: +]"),
                ("core::cmp::Reverse", "[T: +]"),
                ("core::ptr::non_null::NonNull", "[T: +]"),
                ("core::marker::PhantomPinned", "[]"),
                ("core::pin::Pin", "[Ptr: +]"),
                ("core::cell::Ref", "['b: +, T: +]"),
                ("core::cell::RefMut", "['b: +, T: o]"),
            ] {
                assert_eq!(show(name), want, "{name}");
            }
        }) else {
            eprintln!("no rust-src: skipped");
            return;
        };
    }
}
