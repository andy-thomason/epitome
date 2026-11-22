//! Borrow checking.
//!
//! The plan, and what each milestone covers, is in `docs/borrowck.md`. So
//! far: the control-flow graph of a body (`cfg`, built by `build`), and
//! moves and initialisation (`moves`). The interface is `Session::borrowck` and
//! `Diagnostic`, the driver's `--borrowck`, and the expected-errors runner of
//! `tests/ui_check_pass.rs` (`EPITOME_UI_WANT=borrowck`) measures it.
//!
//! The rule that matters most: it must never reject what rustc accepts.
//! Every program of the check-pass and run-pass UI tests, and `core`,
//! `alloc` and `std`, are checked and must have no diagnostics.

pub mod build;
pub mod cfg;
pub mod moves;
pub mod mutability;

use crate::ast::{Ast, NodeId};
use crate::resolve::DefId;
use crate::resolve::Resolver;
use crate::session::{ParsedCrate, Session};
use crate::ty::TyCtxt;
use crate::typeck::TypeckResults;

/// A borrow-checking error, where rustc would report it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Diagnostic {
    /// The crate it is in.
    pub krate: u32,
    /// The file name, line and column (1-based) of its primary span.
    pub file: String,
    pub line: usize,
    pub col: usize,
    /// rustc's error code: `E0382`.
    pub code: &'static str,
    pub message: String,
}

impl std::fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}:{}: error[{}]: {}", self.file, self.line, self.col, self.code, self.message)
    }
}

/// The error codes this checker is to give: those of rustc's borrow
/// checker (and its mutability and move checks). A UI test is of it when
/// all the errors it expects have one of these.
pub const CODES: &[&str] = &[
    "E0381", "E0382", "E0384", "E0499", "E0500", "E0501", "E0502", "E0503", "E0505", "E0506", "E0507", "E0508",
    "E0509", "E0510", "E0515", "E0516", "E0521", "E0594", "E0596", "E0597", "E0712", "E0713", "E0716", "E0373",
];

impl Session {
    /// Borrow checks the bodies of the crates `only` names (the others are
    /// not looked at): rustc's errors, or fewer. So far: moves and
    /// initialisation (`moves`).
    pub fn borrowck<'s>(
        &'s self,
        crates: &[ParsedCrate<'s>],
        r: &Resolver<'s>,
        tcx: &TyCtxt<'s>,
        res: &TypeckResults,
        only: &[u32],
    ) -> Vec<Diagnostic> {
        let built = self.cfgs(crates, r, tcx, res, only);
        let fn_mut = mutability::mutating_closures(&built.cfgs);
        let index = crate::typeck::Index::new(tcx, r);
        // A compiler's stack: the solver recurses deeply on some types.
        let mut out: Vec<Diagnostic> = std::thread::scope(|sc| {
            std::thread::Builder::new()
                .stack_size(512 << 20)
                .spawn_scoped(sc, || {
                    let mut out = Vec::new();
                    for cfg in &built.cfgs {
                        let ast = &crates[cfg.krate as usize].files[cfg.file as usize].ast;
                        let found = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                            let mut solver = crate::typeck::Solver::for_owner(tcx, r, crates, &index, cfg.owner);
                            let order = |n: NodeId| ast.origin_of(ast.node(n).first as usize);
                            let mut v = moves::check(cfg, tcx, &mut solver, &order);
                            v.extend(mutability::check(cfg, tcx, &fn_mut));
                            v
                        }))
                        .unwrap_or_default();
                        for raw in found {
                            out.push(self.diagnostic_at(ast, raw.node, cfg.krate, raw.code, raw.message));
                        }
                    }
                    out
                })
                .unwrap()
                .join()
                .unwrap()
        });
        out.sort_by(|a, b| (a.krate, &a.file, a.line, a.col, a.code).cmp(&(b.krate, &b.file, b.line, b.col, b.code)));
        out.dedup();
        out
    }

    /// A diagnostic for `node` of `ast` (in crate `krate`).
    pub fn diagnostic_at(&self, ast: &Ast<'_>, node: NodeId, krate: u32, code: &'static str, message: impl Into<String>) -> Diagnostic {
        // (A node a macro made is at the macro's call.)
        let tok = ast.token_origin(ast.node(node).first as usize).0;
        let at = self.sources.find(tok).map(|s| s.locate(tok));
        let (file, line, col) = at.map_or((String::new(), 0, 0), |l| (l.name, l.line, l.col));
        Diagnostic { krate, file, line, col, code, message: message.into() }
    }
}

/// The control-flow graphs of a set of bodies, and those that could not be
/// built.
#[derive(Default)]
pub struct CfgResults {
    pub cfgs: Vec<cfg::Cfg>,
    /// The bodies that failed: what they are, and why.
    pub failed: Vec<(DefId, String, String)>,
}

impl CfgResults {
    /// The reasons bodies failed, most common first, each with one such body.
    pub fn reasons(&self) -> Vec<(String, usize, String)> {
        let mut m: std::collections::BTreeMap<String, (usize, String)> = std::collections::BTreeMap::new();
        for (_, name, why) in &self.failed {
            let e = m.entry(why.clone()).or_insert((0, name.clone()));
            e.0 += 1;
        }
        let mut v: Vec<_> = m.into_iter().map(|(why, (n, ex))| (why, n, ex)).collect();
        v.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
        v
    }
}

impl Session {
    /// The control-flow graph of each body of the crates `only` names (and
    /// of the closures in them).
    pub fn cfgs<'s>(
        &'s self,
        crates: &[ParsedCrate<'s>],
        r: &Resolver<'s>,
        tcx: &TyCtxt<'s>,
        res: &TypeckResults,
        only: &[u32],
    ) -> CfgResults {
        // A compiler's stack: the solver recurses deeply on some types.
        std::thread::scope(|sc| {
            std::thread::Builder::new()
                .stack_size(512 << 20)
                .spawn_scoped(sc, || {
                let trace = std::env::var_os("EPITOME_CFG_TRACE").is_some();
        let index = crate::typeck::Index::new(tcx, r);
                let cx = build::Cx { tcx, r, crates, res, index: &index };
                let mut owners: Vec<DefId> = res.bodies.iter().copied().filter(|d| only.contains(&r.defs[d.0 as usize].krate)).collect();
                owners.sort();
                let mut out = CfgResults::default();
                for owner in owners {
                    let guarded = |f: &dyn Fn() -> (Result<cfg::Cfg, String>, Vec<NodeId>)| {
                        std::panic::catch_unwind(std::panic::AssertUnwindSafe(f)).unwrap_or_else(|_| (Err("a panic".to_string()), Vec::new()))
                    };
                    if trace {
                        eprintln!("cfg: {}", r.def_path(owner));
                    }
                    let (built, mut closures) = guarded(&|| build::build_item(&cx, owner));
                    match built {
                        Ok(c) => out.cfgs.push(c),
                        Err(why) => out.failed.push((owner, r.def_path(owner), why)),
                    }
                    while let Some(n) = closures.pop() {
                        let (built, more) = guarded(&|| build::build_closure(&cx, owner, n));
                        closures.extend(more);
                        match built {
                            Ok(c) => out.cfgs.push(c),
                            Err(why) => out.failed.push((owner, format!("{}::{{closure@{}}}", r.def_path(owner), n.0), why)),
                        }
                    }
                }
                    out
                })
                .unwrap()
                .join()
                .unwrap()
        })
    }

    /// The source of `cfg`'s body, as it is, with what the graph does under
    /// each line it comes from: `bbN:` and its statements, and where it goes.
    /// What came from text that is not this file's (a macro's expansion) is
    /// listed after.
    pub fn annotate_cfg<'s>(&'s self, crates: &[ParsedCrate<'s>], r: &Resolver<'s>, tcx: &TyCtxt<'s>, cfg: &cfg::Cfg) -> String {
        use std::fmt::Write;
        let ast = &crates[cfg.krate as usize].files[cfg.file as usize].ast;
        let no_pos = |_: NodeId| None;
        let show = cfg::Show { tcx, r, at: &no_pos, compilable: false };
        let toks = ast.tokens_of(cfg.node);
        let (Some(first), Some(last)) = (toks.first(), toks.last()) else { return String::new() };
        let Some(src) = self.sources.find(first.0) else { return String::new() };
        let (l0, l1) = (src.locate(first.0).line, src.locate(last.0).line);
        let lines: Vec<&str> = src.text.lines().collect();
        // Where a node is in this source: its first line, or its last (for
        // what a macro made, where it was called).
        let line_of = |n: NodeId, end: bool| -> Option<usize> {
            let node = ast.node(n);
            if node.end <= node.first {
                return None;
            }
            let i = if end { node.end - 1 } else { node.first } as usize;
            let tok = ast.token_origin(i).0;
            src.contains(tok).then(|| src.locate(tok).line)
        };
        let mut notes: std::collections::BTreeMap<usize, Vec<(u32, String)>> = std::collections::BTreeMap::new();
        let mut elsewhere: Vec<(u32, String)> = Vec::new();
        let mut put = |line: Option<usize>, b: u32, text: String| match line.filter(|l| (l0..=l1).contains(l)) {
            Some(l) => notes.entry(l).or_default().push((b, text)),
            None => elsewhere.push((b, text)),
        };
        for b in cfg.reachable_sorted() {
            let data = &cfg.blocks[b.0 as usize];
            for s in &data.stmts {
                // Leaving a scope is at the end of what it is the scope of.
                let end = matches!(s.kind, cfg::StmtKind::StorageDead(_) | cfg::StmtKind::Drop(_));
                put(line_of(s.node, end), b.0, show.stmt_text(cfg, s));
            }
            if let Some((t, node)) = &data.term {
                let end = matches!(t, cfg::Term::Return);
                put(line_of(*node, end), b.0, show.term_text(cfg, t));
            }
        }
        let mut out = String::new();
        let _ = writeln!(out, "// {}: {}", cfg.name, show.legend(cfg));
        for l in l0..=l1 {
            let text = lines.get(l - 1).copied().unwrap_or("");
            let _ = writeln!(out, "{text}");
            let indent: String = text.chars().take_while(|c| c.is_whitespace()).collect();
            let mut prev = None;
            for (b, t) in notes.get(&l).into_iter().flatten() {
                let tag = if prev == Some(*b) { "     ".to_string() } else { format!("bb{b}: ") };
                prev = Some(*b);
                let _ = writeln!(out, "{indent}    //  {tag}{t}");
            }
        }
        if !elsewhere.is_empty() {
            let _ = writeln!(out, "// from elsewhere (macro expansions, and the like):");
            let mut prev = None;
            for (b, t) in &elsewhere {
                let tag = if prev == Some(*b) { "     ".to_string() } else { format!("bb{b}: ") };
                prev = Some(*b);
                let _ = writeln!(out, "//     {tag}{t}");
            }
        }
        out
    }

    /// `cfg` as Rust (see `cfg::Show`); `compilable` initialises its locals.
    pub fn show_cfg<'s>(&'s self, crates: &[ParsedCrate<'s>], r: &Resolver<'s>, tcx: &TyCtxt<'s>, cfg: &cfg::Cfg, compilable: bool) -> String {
        let ast = &crates[cfg.krate as usize].files[cfg.file as usize].ast;
        let at = |n: NodeId| {
            let t = ast.token(n).0;
            self.sources.find(t).map(|s| {
                let l = s.locate(t);
                (l.line, l.col)
            })
        };
        cfg::Show { tcx, r, at: &at, compilable }.cfg(cfg)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::session::{CrateRoot, Options};

    /// `core` from the toolchain's rust-src, if it is installed.
    fn core_src() -> Option<std::path::PathBuf> {
        let out = std::process::Command::new("rustc").args(["--print", "sysroot"]).output().ok()?;
        let root = std::path::PathBuf::from(String::from_utf8(out.stdout).ok()?.trim());
        let core = root.join("lib/rustlib/src/rust/library/core/src/lib.rs");
        core.exists().then_some(core)
    }

    /// Runs `f` on the control-flow graphs of the last of `files` (with
    /// `core`, if given).
    fn with_cfgs<T>(core: Option<std::path::PathBuf>, file: &str, f: impl FnOnce(&Session, &[ParsedCrate], &Resolver, &TyCtxt, &CfgResults) -> T) -> T {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(file);
        let session = Session::new(Options { threads: 4, cfg: Some(crate::cfg::CfgSet::host()), ..Default::default() });
        let mut roots = Vec::new();
        if let Some(c) = core {
            roots.push(CrateRoot::new("core", c));
        }
        let name = path.file_stem().unwrap().to_string_lossy().into_owned();
        roots.push(CrateRoot::new(&name, &path));
        let mut crates = session.parse_crates(roots);
        session.expand(&mut crates);
        let r = session.resolve(&crates);
        let tcx = session.collect_types(&crates, &r);
        let res = session.typeck(&crates, &r, &tcx);
        let last = crates.len() as u32 - 1;
        let out = session.cfgs(&crates, &r, &tcx, &res, &[last]);
        f(&session, &crates, &r, &tcx, &out)
    }

    #[test]
    fn small_bodies_build_and_are_valid() {
        let Some(core) = core_src() else {
            eprintln!("no rust-src: skipped");
            return;
        };
        with_cfgs(Some(core), "tests/borrowck/cfg_small.rs", |session, crates, r, tcx, out| {
            assert!(out.failed.is_empty(), "not built: {:?}", out.failed);
            assert!(out.cfgs.len() >= 12, "{} bodies", out.cfgs.len());
            for c in &out.cfgs {
                c.validate().unwrap_or_else(|e| panic!("{}: {e}", c.name));
            }
            let find = |name: &str| out.cfgs.iter().find(|c| c.name.ends_with(name)).unwrap_or_else(|| panic!("no {name}"));
            let abs = session.show_cfg(crates, r, tcx, find("::abs"), false);
            // `if x < 0`: a comparison, then a branch on it.
            assert!(abs.contains("_1 < 0"), "{abs}");
            assert!(abs.contains("bb = if"), "{abs}");
            // The loop of `sum` goes back to its head.
            let sum = find("::sum");
            let heads: Vec<_> = sum.reachable().into_iter().filter(|b| sum.blocks[b.0 as usize].term.as_ref().is_some_and(|t| t.0.successors().iter().any(|s| s.0 < b.0))).collect();
            assert!(!heads.is_empty(), "no back edge in sum");
            // Every `StorageLive` is matched by a `StorageDead` on the way out.
            let first_big = session.show_cfg(crates, r, tcx, find("::first_big"), false);
            assert!(first_big.matches("StorageLive").count() > 0 && first_big.contains("StorageDead"), "{first_big}");
        });
    }

    /// The annotated source is the source, with notes under its lines.
    #[test]
    fn annotated_source_keeps_the_source() {
        let Some(core) = core_src() else {
            eprintln!("no rust-src: skipped");
            return;
        };
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/borrowck/cfg_small.rs");
        let original = std::fs::read_to_string(&path).unwrap();
        with_cfgs(Some(core), "tests/borrowck/cfg_small.rs", |session, crates, r, tcx, out| {
            for c in out.cfgs.iter().filter(|c| c.closure.is_none()) {
                let text = session.annotate_cfg(crates, r, tcx, c);
                // Taking the notes out leaves lines of the file, in order.
                let kept: Vec<&str> = text.lines().filter(|l| !l.trim_start().starts_with("//  ") && !l.starts_with("// ")).collect();
                assert!(!kept.is_empty(), "{}: nothing", c.name);
                let mut at = 0;
                for l in &kept {
                    let found = original.lines().skip(at).position(|o| o == *l);
                    assert!(found.is_some(), "{}: `{l}` is not the next line of the source", c.name);
                    at += found.unwrap() + 1;
                }
            }
            let find = |name: &str| out.cfgs.iter().find(|c| c.name.ends_with(name)).unwrap();
            let abs = session.annotate_cfg(crates, r, tcx, find("::abs"));
            assert!(abs.contains("pub fn abs(x: i32) -> i32 {"), "{abs}");
            assert!(abs.contains("//  bb0: "), "{abs}");
            assert!(abs.contains("_2 = _1 < 0"), "{abs}");
            // The loop of `sum`: its condition's notes are on its line.
            let sum = session.annotate_cfg(crates, r, tcx, find("::sum"));
            let line = sum.lines().position(|l| l.trim() == "while i < n {").expect("the while line");
            assert!(sum.lines().nth(line + 1).unwrap().contains("//  bb"), "{sum}");
        });
    }

    /// What a file says its errors are: the lines with `//~ E0xxx`.
    fn annotated(path: &std::path::Path) -> Vec<(usize, String)> {
        let text = std::fs::read_to_string(path).unwrap();
        let mut v = Vec::new();
        for (i, l) in text.lines().enumerate() {
            if let Some(rest) = l.split("//~").nth(1) {
                for code in rest.split_whitespace() {
                    let is_code = code.len() == 5 && code.starts_with('E') && code[1..].chars().all(|c| c.is_ascii_digit());
                    if is_code {
                        v.push((i + 1, code.to_string()));
                    }
                }
            }
        }
        v
    }

    /// The errors `borrowck` finds in a file of the tests, and no others than
    /// the file says.
    fn assert_errors_as_annotated(file: &str) {
        let Some(core) = core_src() else {
            eprintln!("no rust-src: skipped");
            return;
        };
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(file);
        let mut want = annotated(&path);
        want.sort();
        let session = Session::new(Options { threads: 4, cfg: Some(crate::cfg::CfgSet::host()), ..Default::default() });
        let mut crates = session.parse_crates(vec![CrateRoot::new("core", core), CrateRoot::new("tested", &path)]);
        session.expand(&mut crates);
        let r = session.resolve(&crates);
        let tcx = session.collect_types(&crates, &r);
        let res = session.typeck(&crates, &r, &tcx);
        let mut got: Vec<(usize, String)> = session.borrowck(&crates, &r, &tcx, &res, &[1]).iter().map(|d| (d.line, d.code.to_string())).collect();
        got.sort();
        assert_eq!(got, want, "{file}: found (left), annotated (right)");
    }

    /// Moves and initialisation: the errors it says, and no others.
    #[test]
    fn moves_and_initialisation() {
        assert_errors_as_annotated("tests/borrowck/moves.rs");
    }

    /// Mutability: the errors it says, and no others.
    #[test]
    fn mutability() {
        assert_errors_as_annotated("tests/borrowck/mutability.rs");
    }

    /// The rendering is Rust: `rustc` takes it (for numbers and references).
    #[test]
    fn rendered_bodies_compile() {
        let Some(core) = core_src() else {
            eprintln!("no rust-src: skipped");
            return;
        };
        let text = with_cfgs(Some(core), "tests/borrowck/cfg_small.rs", |session, crates, r, tcx, out| {
            let mut text = String::from("#![allow(warnings)]\n");
            for c in &out.cfgs {
                text += &session.show_cfg(crates, r, tcx, c, true);
                text.push('\n');
            }
            text
        });
        let dir = std::env::temp_dir().join(format!("epitome-cfg-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let src = dir.join("rendered.rs");
        std::fs::write(&src, &text).unwrap();
        let out = std::process::Command::new("rustc")
            .args(["--edition", "2021", "--crate-type", "lib", "--emit=metadata", "-o"])
            .arg(dir.join("rendered.rmeta"))
            .arg(&src)
            .output();
        let Ok(out) = out else {
            eprintln!("no rustc: skipped");
            return;
        };
        let _ = std::fs::remove_dir_all(&dir).ok();
        assert!(out.status.success(), "rustc says:\n{}\nfor:\n{text}", String::from_utf8_lossy(&out.stderr));
    }

    /// Nearly every body of `core` has one (the rest are those type checking
    /// could not type).
    #[test]
    fn core_bodies_build() {
        let Some(core) = core_src() else {
            eprintln!("no rust-src: skipped");
            return;
        };
        let session = Session::new(Options { threads: 4, cfg: Some(crate::cfg::CfgSet::host()), ..Default::default() });
        let mut crates = session.parse_crates(vec![CrateRoot::new("core", core)]);
        session.expand(&mut crates);
        let r = session.resolve(&crates);
        let tcx = session.collect_types(&crates, &r);
        let res = session.typeck(&crates, &r, &tcx);
        let out = session.cfgs(&crates, &r, &tcx, &res, &[0]);
        let invalid: Vec<_> = out.cfgs.iter().filter_map(|c| c.validate().err().map(|e| format!("{}: {e}", c.name))).take(5).collect();
        assert!(invalid.is_empty(), "invalid: {invalid:?}");
        let total = out.cfgs.len() + out.failed.len();
        assert!(total > 20_000, "{total} bodies");
        assert!(out.cfgs.len() * 1000 >= total * 995, "{} of {total} built; not built: {:?}", out.cfgs.len(), &out.reasons()[..out.reasons().len().min(8)]);
    }
}
