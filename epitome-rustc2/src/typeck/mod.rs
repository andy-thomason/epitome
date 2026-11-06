//! Type checking function bodies (and the values of constants and
//! statics), in parallel: each body is a job of its own.
//!
//! A body is checked with fresh inference variables (`infer.rs`):
//! expressions are checked with the type expected of them where it is
//! known, patterns against the type they match, and the types of every
//! expression and pattern go in a side table indexed by node. At the end,
//! integer and float variables left free become `i32` and `f64`.
//!
//! Besides types, the tables keep what lowering needs to know of what
//! checking decided: the function each method call and overloaded
//! operator calls, with its generic arguments; the adjustments made to an
//! expression (autoderef, autoref, coercions); and, for patterns, how many
//! references each matched through and how each binding binds.

mod check;
pub mod infer;
mod traits;

use crate::ast::NodeId;
use crate::hash::FastMap;
use crate::pool;
use crate::resolve::{DefId, DefKind, Resolver};
use crate::session::{ParsedCrate, Session};
use crate::ty::{List, Mutbl, Ty, TyCtxt, TyKind};
use std::sync::Mutex;

/// What type checking found.
pub struct TypeckResults {
    /// The type of every expression and pattern: per crate, per file, by
    /// node.
    pub types: Vec<Vec<Vec<Option<Ty>>>>,
    /// Everything else, per crate, per file.
    pub tables: Vec<Vec<Tables>>,
    /// The bodies checked, by owner.
    pub bodies: Vec<DefId>,
    pub stats: TypeckStats,
}

#[derive(Default, Debug)]
pub struct TypeckStats {
    pub bodies: usize,
    /// Bodies with no error type and no type left unknown.
    pub clean_bodies: usize,
    /// Expressions and patterns typed.
    pub nodes: usize,
    /// ... whose type has an error in it.
    pub error_nodes: usize,
    /// ... whose type was left unknown.
    pub unknown_nodes: usize,
    /// Why, most common first.
    pub reasons: Vec<(String, usize)>,
}

/// A step that turns an expression's value into what its use wants, in
/// order. A method receiver `v` of `v.len()` (`v: &Vec<u8>`) is adjusted
/// by `[Deref, OverloadedDeref, Borrow(Not)]`: `&*Deref::deref(&*v)`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Adjust {
    /// `*` of a reference, raw pointer or `Box`.
    Deref,
    /// `*` through `Deref::deref` (or `DerefMut::deref_mut` for a mutable
    /// use).
    OverloadedDeref,
    /// `&` or `&mut` (autoref, or the end of a deref coercion).
    Borrow(Mutbl),
    /// A pointer to something sized made a pointer to something unsized:
    /// `&[T; N]` to `&[T]`, `&T` or `Box<T>` to `dyn Trait`.
    Unsize,
    /// `!` to any type (the value never exists).
    NeverToAny,
    /// A function item made a function pointer.
    ReifyFnPointer,
    /// A closure that captures nothing made a function pointer.
    ClosureFnPointer,
}

impl std::fmt::Display for Adjust {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Adjust::Deref => "*",
            Adjust::OverloadedDeref => "*Deref",
            Adjust::Borrow(Mutbl::Not) => "&",
            Adjust::Borrow(Mutbl::Mut) => "&mut",
            Adjust::Unsize => "unsize",
            Adjust::NeverToAny => "never",
            Adjust::ReifyFnPointer => "fn-pointer",
            Adjust::ClosureFnPointer => "closure-fn-pointer",
        })
    }
}

/// How a binding binds the value it matches.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BindingMode {
    Value,
    Ref(Mutbl),
}

/// One file's tables, by node.
#[derive(Default)]
pub struct Tables {
    /// The function a method call, overloaded operator (`+`, `==`, unary
    /// `*`, `[]`) or call through the `Fn` traits calls, and its generic
    /// arguments (a trait method's start with `Self`).
    pub methods: FastMap<NodeId, (DefId, List)>,
    /// The adjustments made to an expression's value.
    pub adjustments: FastMap<NodeId, Vec<Adjust>>,
    /// How many references a pattern matched through before matching
    /// (default binding modes).
    pub pat_derefs: FastMap<NodeId, u32>,
    /// How each binding pattern binds.
    pub bindings: FastMap<NodeId, BindingMode>,
}

/// One body to check.
#[derive(Clone, Copy)]
struct Body {
    owner: DefId,
    k: u32,
    file: u32,
    /// The function, constant or static.
    item: NodeId,
}

/// What one body's check gives.
struct BodyResult {
    k: u32,
    file: u32,
    types: Vec<(NodeId, Ty)>,
    methods: Vec<(NodeId, DefId, List)>,
    adjustments: Vec<(NodeId, Vec<Adjust>)>,
    pat_derefs: Vec<(NodeId, u32)>,
    bindings: Vec<(NodeId, BindingMode)>,
    errors: FastMap<String, usize>,
    error_nodes: usize,
    unknown_nodes: usize,
}

pub use check::Index;

/// Trait selection and normalisation outside any body, for types whose
/// generic parameters have all been replaced (monomorphisation, layout).
/// Cheap to make: one per job.
pub struct Solver<'a, 's> {
    cx: check::FnCtxt<'a, 's>,
}

/// What makes a concrete type implement a trait.
#[derive(Clone, Debug)]
pub enum ImplSource {
    /// An impl (its index in `TyCtxt::impls`) with its arguments.
    Impl(usize, Vec<Ty>),
    /// Built in: closures and functions for the `Fn` traits, `Sized`,
    /// auto traits, ...
    Builtin,
}

impl<'a, 's> Solver<'a, 's> {
    pub fn new(tcx: &'a TyCtxt<'s>, r: &'a Resolver<'s>, crates: &'a [ParsedCrate<'s>], index: &'a Index<'s>) -> Self {
        // The first crate's root: no predicates to assume.
        let root = r.defs.iter().position(|d| d.parent.is_none()).map_or(DefId(0), |i| DefId(i as u32));
        let def = &r.defs[root.0 as usize];
        Solver { cx: check::FnCtxt::new(tcx, r, crates, index, root, def.krate, def.file) }
    }

    /// `t` with its projections worked out.
    pub fn normalize(&mut self, t: Ty) -> Ty {
        let mut t = t;
        for _ in 0..4 {
            let n = self.cx.normalize(t, 1);
            let n = self.cx.infer.resolve(n);
            if n == t {
                break;
            }
            t = n;
        }
        t
    }

    /// How `t: tr<targs>` holds, if it does unambiguously.
    pub fn select(&mut self, t: Ty, tr: DefId, targs: &[Ty]) -> Option<ImplSource> {
        let s = self.cx.probe();
        let got = match self.cx.select(t, tr, targs, 0) {
            traits::Select::Yes(traits::Source::Impl(i, args)) => {
                let args = args.into_iter().map(|a| self.cx.infer.resolve(a)).collect();
                Some(ImplSource::Impl(i, args))
            }
            traits::Select::Yes(_) => Some(ImplSource::Builtin),
            _ => None,
        };
        self.cx.rollback(s);
        got
    }

    /// The function a call of trait method `item` at `args` (`Self`, the
    /// trait's other arguments, then the method's own) runs: the impl's
    /// method with its arguments, or the trait's default method. `None`
    /// for a built-in impl (a closure's `call`, say) or no impl.
    pub fn resolve_method(&mut self, item: DefId, args: &[Ty]) -> Option<(DefId, Vec<Ty>)> {
        let tcx = self.cx.tcx;
        let Some(tr) = self.cx.trait_of(item) else { return Some((item, args.to_vec())) };
        let ntrait = tcx.all_params(tr).len();
        let (self_ty, targs) = (args[0], &args[1..ntrait.min(args.len())]);
        match self.select(self_ty, tr, targs)? {
            ImplSource::Impl(i, iargs) => {
                let name = self.cx.r.defs[item.0 as usize].name;
                let m = self.cx.r.defs[tcx.impls[i].def.0 as usize].module?;
                match self.cx.r.modules[m.0 as usize].names.get(&(name, crate::resolve::Ns::Value)).map(|b| &b.res) {
                    Some(crate::resolve::Res::Def(f)) => {
                        let mut a = iargs;
                        a.extend_from_slice(&args[ntrait.min(args.len())..]);
                        Some((*f, a))
                    }
                    // Not overridden: the trait's own body.
                    _ => Some((item, args.to_vec())),
                }
            }
            ImplSource::Builtin => None,
        }
    }
}

impl Session {
    /// Type checks every body of `crates`.
    pub fn typeck<'s>(&'s self, crates: &[ParsedCrate<'s>], r: &Resolver<'s>, tcx: &TyCtxt<'s>) -> TypeckResults {
        let index = check::Index::new(tcx, r);
        self.typeck_with(crates, r, tcx, &index)
    }

    /// `typeck`, with an index of impls already made (to share with
    /// later stages).
    pub fn typeck_with<'s>(&'s self, crates: &[ParsedCrate<'s>], r: &Resolver<'s>, tcx: &TyCtxt<'s>, index: &Index<'s>) -> TypeckResults {
        let bodies: Vec<Body> = (0..r.defs.len() as u32)
            .map(DefId)
            .filter(|&d| {
                let def = &r.defs[d.0 as usize];
                def.parent.is_some()
                    && matches!(
                        def.kind,
                        DefKind::Fn | DefKind::AssocFn | DefKind::Const | DefKind::Static | DefKind::AssocConst
                    )
            })
            .map(|d| {
                let def = &r.defs[d.0 as usize];
                Body { owner: d, k: def.krate, file: def.file, item: def.node }
            })
            .collect();
        let out: Mutex<Vec<BodyResult>> = Mutex::new(Vec::new());
        let checked: Mutex<Vec<DefId>> = Mutex::new(Vec::new());
        pool::run(self.threads(), bodies, |b, _| {
            if let Some(res) = check::check_body(tcx, r, crates, index, b.owner, b.k, b.file, b.item) {
                checked.lock().unwrap().push(b.owner);
                out.lock().unwrap().push(res);
            }
        });
        let mut types: Vec<Vec<Vec<Option<Ty>>>> =
            crates.iter().map(|c| c.files.iter().map(|f| vec![None; f.ast.nodes.len()]).collect()).collect();
        let mut tables: Vec<Vec<Tables>> =
            crates.iter().map(|c| c.files.iter().map(|_| Tables::default()).collect()).collect();
        let mut stats = TypeckStats::default();
        let mut reasons: FastMap<String, usize> = FastMap::default();
        for res in out.into_inner().unwrap() {
            stats.bodies += 1;
            if res.error_nodes == 0 && res.unknown_nodes == 0 {
                stats.clean_bodies += 1;
            }
            stats.nodes += res.types.len();
            stats.error_nodes += res.error_nodes;
            stats.unknown_nodes += res.unknown_nodes;
            let file = &mut types[res.k as usize][res.file as usize];
            for (n, t) in res.types {
                if let Some(slot) = file.get_mut(n.0 as usize) {
                    *slot = Some(t);
                }
            }
            let t = &mut tables[res.k as usize][res.file as usize];
            t.methods.extend(res.methods.into_iter().map(|(n, d, l)| (n, (d, l))));
            t.adjustments.extend(res.adjustments);
            t.pat_derefs.extend(res.pat_derefs);
            t.bindings.extend(res.bindings);
            for (k, v) in res.errors {
                *reasons.entry(k).or_default() += v;
            }
        }
        let mut reasons: Vec<(String, usize)> = reasons.into_iter().collect();
        reasons.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
        stats.reasons = reasons;
        let mut bodies = checked.into_inner().unwrap();
        bodies.sort();
        TypeckResults { types, tables, bodies, stats }
    }
}

impl TypeckResults {
    /// Each checked body's expressions and patterns with their types, in
    /// source order: `line:col text : type`.
    pub fn dump(&self, crates: &[ParsedCrate], r: &Resolver, tcx: &TyCtxt, sources: &crate::source::SourceMap) -> String {
        self.dump_crate(crates, r, tcx, sources, None)
    }

    /// `dump`, of crate `only` alone if given.
    pub fn dump_crate(
        &self,
        crates: &[ParsedCrate],
        r: &Resolver,
        tcx: &TyCtxt,
        sources: &crate::source::SourceMap,
        only: Option<u32>,
    ) -> String {
        use std::fmt::Write;
        let mut out = String::new();
        for &d in &self.bodies {
            let def = &r.defs[d.0 as usize];
            if only.is_some_and(|k| k != def.krate) {
                continue;
            }
            let (k, f) = (def.krate as usize, def.file as usize);
            let ast = &crates[k].files[f].ast;
            let _ = writeln!(out, "{} {}", if matches!(def.kind, DefKind::Fn | DefKind::AssocFn) { "fn" } else { "const" }, r.def_path(d));
            let mut nodes = Vec::new();
            collect_nodes(ast, def.node, &mut nodes);
            let types = &self.types[k][f];
            for n in nodes {
                let Some(Some(t)) = types.get(n.0 as usize) else { continue };
                let tok = ast.token(n).0;
                let at = sources.find(tok).map(|s| {
                    let l = s.locate(tok);
                    format!("{}:{}", l.line, l.col)
                });
                let mut text = ast.text(n).replace('\n', " ");
                if text.len() > 48 {
                    let cut = (0..=45).rev().find(|&i| text.is_char_boundary(i)).unwrap_or(0);
                    text.truncate(cut);
                    text.push_str("...");
                }
                let tables = &self.tables[k][f];
                let method = tables.methods.get(&n).map(|m| format!("  [{}]", r.def_path(m.0))).unwrap_or_default();
                let mut extra = String::new();
                if let Some(a) = tables.adjustments.get(&n) {
                    let a: Vec<String> = a.iter().map(|x| x.to_string()).collect();
                    let _ = write!(extra, "  {{{}}}", a.join(", "));
                }
                if let Some(&d) = tables.pat_derefs.get(&n) {
                    let _ = write!(extra, "  {{through {d} &}}");
                }
                if let Some(BindingMode::Ref(m)) = tables.bindings.get(&n) {
                    let _ = write!(extra, "  {{by {}}}", if *m == Mutbl::Mut { "ref mut" } else { "ref" });
                }
                let _ = writeln!(out, "  {:8} {text} : {}{method}{extra}", at.unwrap_or_default(), tcx.display(r, *t));
            }
        }
        out
    }
}

/// The nodes under `n`, in source order, not entering nested items.
fn collect_nodes(ast: &crate::ast::Ast, n: NodeId, out: &mut Vec<NodeId>) {
    for &c in ast.children(n) {
        let k = ast.kind(c);
        if matches!(k, crate::ast::Kind::Fn | crate::ast::Kind::Impl | crate::ast::Kind::Mod | crate::ast::Kind::Struct | crate::ast::Kind::Enum | crate::ast::Kind::Trait) {
            continue;
        }
        out.push(c);
        collect_nodes(ast, c, out);
    }
}

impl TyCtxt<'_> {
    /// Whether a type has an unbound inference variable in it.
    pub fn has_infer(&self, t: Ty) -> bool {
        match self.kind(t) {
            TyKind::Infer(_) => true,
            TyKind::Tuple(l) | TyKind::Adt(_, l) | TyKind::FnDef(_, l) | TyKind::Projection(_, l) => {
                self.tys(l).into_iter().any(|x| self.has_infer(x))
            }
            TyKind::Ref(_, x) | TyKind::Ptr(_, x) | TyKind::Slice(x) | TyKind::Array(x, _) | TyKind::Pat(x, ..) => self.has_infer(x),
            TyKind::FnPtr(s) | TyKind::Closure(_, s) => {
                let s = self.sig_of(s);
                self.has_infer(s.output) || self.tys(s.inputs).into_iter().any(|x| self.has_infer(x))
            }
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::session::{CrateRoot, Options, Session};

    /// `--dump-types` of a test file (with `core`, if `with` names it),
    /// without positions.
    fn dump_with(path: &str, name: &str, with: Option<std::path::PathBuf>) -> String {
        let file = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(path);
        let session = Session::new(Options { threads: 4, cfg: Some(crate::cfg::CfgSet::host()), ..Default::default() });
        let mut roots = Vec::new();
        if let Some(core) = with {
            roots.push(CrateRoot::new("core", core));
        }
        roots.push(CrateRoot::new(name, &file));
        let mut crates = session.parse_crates(roots);
        session.expand(&mut crates);
        let r = session.resolve(&crates);
        let tcx = session.collect_types(&crates, &r);
        let res = session.typeck(&crates, &r, &tcx);
        let k = crates.len() as u32 - 1;
        res.dump_crate(&crates, &r, &tcx, &session.sources, Some(k))
            .lines()
            .map(|l| match l.strip_prefix("  ") {
                Some(rest) => rest.split_once(' ').map_or(rest, |x| x.1).trim_start().to_string(),
                None => l.to_string(),
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn dump(path: &str) -> String {
        plain(&dump_with(path, "basic", None))
    }

    /// The dump without its `  {..}` notes (adjustments, binding modes).
    fn plain(s: &str) -> String {
        s.lines()
            .map(|l| match l.find("  {") {
                Some(i) => &l[..i],
                None => l,
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// `core` from the toolchain's rust-src, if it is installed.
    fn core_src() -> Option<std::path::PathBuf> {
        let out = std::process::Command::new("rustc").args(["--print", "sysroot"]).output().ok()?;
        let root = std::path::PathBuf::from(String::from_utf8(out.stdout).ok()?.trim());
        let core = root.join("lib/rustlib/src/rust/library/core/src/lib.rs");
        core.exists().then_some(core)
    }

    #[test]
    fn traits_methods_and_closures() {
        let Some(core) = core_src() else {
            eprintln!("no rust-src: skipped");
            return;
        };
        let full = dump_with("tests/typeck/traits.rs", "traits", Some(core));
        // What lowering is told: adjustments, binding modes and the
        // methods operators and calls through `Fn` call.
        for want in [
            "a : &str  {*, &}",
            "Shape::Circle(r) : Shape  {through 1 &}",
            "r : &u32  {by ref}",
            "3 * r : u32  [core::ops::arith::Mul::mul]",
            "seen(x) : ()  [core::ops::function::FnMut::call_mut]",
        ] {
            assert!(full.lines().any(|l| l == want), "no `{want}` in:\n{full}");
        }
        let s = plain(&full);
        // Only this crate's bodies, which should have no error types.
        let ours: Vec<&str> =
            s.lines().skip_while(|l| !l.starts_with("fn traits::")).collect();
        assert!(!ours.is_empty(), "no bodies of `traits`");
        for l in &ours {
            assert!(!l.contains("{error}"), "error type in `traits`:\n{}", ours.join("\n"));
        }
        for want in [
            // A trait method through autoref, with its closure's parameter
            // from the `FnMut(Self::Item)` bound.
            "|s| match s { Shape::Circle(r) => 3 * r * r, ... : {closure}(&Shape) -> u32",
            "r : &u32",
            "3 * r * r : u32  [core::ops::arith::Mul::mul]",
            // `impl Trait` parameters and `&mut impl FnMut`.
            "seen(x) : ()  [core::ops::function::FnMut::call_mut]",
            "x : u32",
            // `?` through `Try`, and `parse` from the type wanted.
            "a.parse() : Result<u8, ParseIntError>  [core::str::{impl}::parse]",
            "a.parse().ok()? : u8",
            // `Default::default()` and `+` from bounds on an associated type.
            "Default::default() : <I as Iterator>::Item",
            "a + b : <I as Iterator>::Item",
            // A `dyn Trait` method.
            "w.write_str(s) : Result<(), Error>  [core::fmt::Write::write_str]",
            // Patterns through references and iterator adapters.
            "(x, y) : (&u8, &u16)",
            "u16::from : fn item core::convert::From::from<u16, u8>",
        ] {
            assert!(ours.contains(&want), "no `{want}` in:\n{}", ours.join("\n"));
        }
    }

    #[test]
    fn bodies_and_their_types() {
        let s = dump("tests/typeck/basic.rs");
        for want in [
            "Point { x, y } : Point",
            "self.x - o.x : i32",
            "dx * dx + dy * dy : i32",
            // Patterns bind by reference through `&Shape`.
            "r : &u32",
            "a : &Point",
            "((b.x - a.x) * (b.y - a.y)) as u32 : u32",
            // `0` becomes `u64` from what is added to it later.
            "0 : u64",
            "x : &u64",
            "p.dist2(&q) : i32  [basic::{impl}::dist2]",
            // The array's integers are `u64` from `sum(&arr)`, after.
            "arr : [u64; 3]",
            "1.5 : f64",
            "pair : (i32, f64)",
            "loop { break 7u8; } : u8",
            "|a: i32| a + 1 : {closure}(i32) -> i32",
            "c(d) : i32",
        ] {
            assert!(s.lines().any(|l| l == want), "no `{want}` in:\n{s}");
        }
    }
}
