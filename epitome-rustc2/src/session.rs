//! The compiler as a library: a `Session` holds what all crates of a run
//! share (the source map, the options), and loads crates in parallel.
//!
//! ```no_run
//! use epitome_rustc2::session::{CrateRoot, Options, Session};
//! let session = Session::new(Options::default());
//! let mut crates = session.parse_crates(vec![CrateRoot::new("hello", "src/main.rs")]);
//! let stats = session.expand(&mut crates);
//! for f in &crates[0].files {
//!     println!("{} ({} nodes)", f.source.name, f.ast.nodes.len());
//! }
//! ```
//!
//! Parsing a file finds its modules (`mod foo;`), and each module's file is
//! a new job on the pool, so the files of all crates are read and parsed
//! at once, as they are found. The result does not depend on the order the
//! jobs ran in: each file is placed by its position in the text of the
//! crate (`ParsedFile::pos`).

use crate::ast::{Ast, Kind, NodeId};
use crate::cfg::{CfgSet, strip};
use crate::lexer::{Edition, Token};
use crate::parse::parse_str;
use crate::pool;
use crate::source::{Error, Source, SourceMap};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

/// Options for a whole run.
#[derive(Clone, Debug, Default)]
pub struct Options {
    /// Worker threads; 0 means one per core.
    pub threads: usize,
    /// The configuration to strip each file with as it is parsed. Without
    /// one, nothing is stripped and every `mod` whose file exists is
    /// loaded (for parsing everything, whatever the target).
    pub cfg: Option<CfgSet>,
    /// The proc-macro server (`libexec/rust-analyzer-proc-macro-srv` of the
    /// toolchain that built the proc-macro libraries).
    pub proc_macro_server: Option<PathBuf>,
}

/// A crate to load: its name, root file and edition.
#[derive(Clone, Debug)]
pub struct CrateRoot {
    pub name: String,
    pub root: PathBuf,
    pub edition: Edition,
    /// Whether to load the files of `mod foo;` declarations. Off, only the
    /// root file is parsed (for parsing loose files).
    pub follow_modules: bool,
    /// Proc-macro libraries the crate uses, by the name it knows them by
    /// (as `--extern name=lib.so`).
    pub proc_macros: Vec<(String, PathBuf)>,
}

impl CrateRoot {
    pub fn new(name: impl Into<String>, root: impl Into<PathBuf>) -> Self {
        CrateRoot {
            name: name.into(),
            root: root.into(),
            edition: Edition::default(),
            follow_modules: true,
            proc_macros: Vec::new(),
        }
    }

    pub fn proc_macro(mut self, name: impl Into<String>, lib: impl Into<PathBuf>) -> Self {
        self.proc_macros.push((name.into(), lib.into()));
        self
    }

    pub fn edition(mut self, edition: Edition) -> Self {
        self.edition = edition;
        self
    }

    pub fn follow_modules(mut self, follow: bool) -> Self {
        self.follow_modules = follow;
        self
    }
}

/// One step of a module path: the module's name, and whether its `mod`
/// item has `#[macro_use]` (so its macros stay in scope after it).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScopeStep {
    pub name: String,
    pub macro_use: bool,
}

/// One parsed file of a crate.
pub struct ParsedFile<'s> {
    /// Items whose `#[derive]`s have been expanded, so that a later round
    /// does not expand them again.
    pub derived: HashSet<NodeId>,
    pub source: &'s Source,
    pub path: PathBuf,
    /// Whether the file owns its directory (`lib.rs`, `main.rs`, `mod.rs`,
    /// or a `#[path]` file), so that its modules are beside it.
    pub mod_rs: bool,
    pub ast: Ast<'s>,
    /// The module's path from the crate root: `["a", "b"]` for
    /// `crate::a::b`; empty for the root.
    pub module: Vec<String>,
    /// The same path, with each step's `#[macro_use]`.
    pub scope: Vec<ScopeStep>,
    /// Where the file sits in the text of the crate: for each `mod` item
    /// on the way down, the position of its first token in its file (the
    /// token a macro expansion was called from, then the token itself).
    /// Files and macro definitions are ordered by this.
    pub pos: Vec<u32>,
    /// The file holding this module's `mod` item, and that item.
    pub parent: Option<(usize, NodeId)>,
}

/// A crate's files, in text order (the root first, then each module's file
/// after its parent's, in the order of the `mod` items).
pub struct ParsedCrate<'s> {
    pub name: String,
    pub edition: Edition,
    pub follow_modules: bool,
    /// The root file's directory, the current directory for proc macros.
    pub dir: PathBuf,
    pub proc_macros: Vec<(String, PathBuf)>,
    pub files: Vec<ParsedFile<'s>>,
    /// Every error found, by file.
    pub errors: Vec<Error<'s>>,
    /// The `mod` items whose files have been loaded (or tried), as the
    /// declaring file's position and the item.
    loaded: HashSet<(Vec<u32>, NodeId)>,
    /// Parent links, by position, for files loaded later.
    parent_pos: Vec<Option<(Vec<u32>, NodeId)>>,
}

pub struct Session {
    pub sources: SourceMap,
    pub options: Options,
    pub(crate) proc_macro_server: Option<crate::proc_macro::Server>,
    /// Files read by `include!` (and later `include_str!`), which are
    /// sources of the compilation as much as module files are.
    pub included: Mutex<Vec<PathBuf>>,
}

/// Reading and parsing one file.
struct Job<'s> {
    krate: usize,
    path: PathBuf,
    pos: Vec<u32>,
    module: Vec<String>,
    scope: Vec<ScopeStep>,
    mod_rs: bool,
    edition: Edition,
    follow: bool,
    /// The parent's position and `mod` item, and the item's name for
    /// errors.
    decl: Option<(Vec<u32>, NodeId, &'s str)>,
}

/// What a job produced.
struct Done<'s> {
    krate: usize,
    file: Option<ParsedFile<'s>>,
    parent: Option<(Vec<u32>, NodeId)>,
    errors: Vec<Error<'s>>,
    /// `mod` items this job queued files for.
    loaded: Vec<(Vec<u32>, NodeId)>,
}

/// The position of a node in its file's text order: the token its
/// expansion was called from, then its own first token.
pub fn node_pos(ast: &Ast, id: NodeId) -> [u32; 2] {
    let first = ast.node(id).first as usize;
    [ast.origin_of(first), first as u32]
}

impl Session {
    pub fn new(options: Options) -> Self {
        let proc_macro_server = options.proc_macro_server.as_ref().map(crate::proc_macro::Server::new);
        Session { sources: SourceMap::new(), options, proc_macro_server, included: Mutex::new(Vec::new()) }
    }

    pub(crate) fn threads(&self) -> usize {
        if self.options.threads == 0 { pool::default_threads() } else { self.options.threads }
    }

    /// Adds text made during the compilation to the source map, and
    /// returns it for the life of the session.
    pub fn add_text(&self, name: &str, text: String) -> &str {
        &self.sources.add(Source::new(name, text)).text
    }

    /// Loads and parses crates, all at once.
    pub fn parse_crates(&self, roots: Vec<CrateRoot>) -> Vec<ParsedCrate<'_>> {
        let mut crates: Vec<ParsedCrate> = roots
            .iter()
            .map(|r| ParsedCrate {
                name: r.name.clone(),
                edition: r.edition,
                follow_modules: r.follow_modules,
                dir: r.root.parent().map_or_else(PathBuf::new, Path::to_path_buf),
                proc_macros: r.proc_macros.clone(),
                files: Vec::new(),
                errors: Vec::new(),
                loaded: HashSet::new(),
                parent_pos: Vec::new(),
            })
            .collect();
        let jobs = roots
            .iter()
            .enumerate()
            .map(|(k, r)| Job {
                krate: k,
                path: r.root.clone(),
                pos: Vec::new(),
                module: Vec::new(),
                scope: Vec::new(),
                mod_rs: true,
                edition: r.edition,
                follow: r.follow_modules,
                decl: None,
            })
            .collect();
        self.load(&mut crates, jobs);
        crates
    }

    /// Runs file jobs (and the jobs for the modules they find), and puts
    /// the files in their crates in text order.
    fn load<'s>(&'s self, crates: &mut [ParsedCrate<'s>], jobs: Vec<Job<'s>>) {
        if jobs.is_empty() {
            return;
        }
        let done: Mutex<Vec<Done>> = Mutex::new(Vec::new());
        pool::run(self.threads(), jobs, |job, queue| {
            let result = self.parse_job(job, queue);
            done.lock().unwrap().push(result);
        });
        let mut done = done.into_inner().unwrap();
        done.sort_by(|a, b| {
            let pa = a.file.as_ref().map(|f| &f.pos);
            let pb = b.file.as_ref().map(|f| &f.pos);
            (a.krate, pa).cmp(&(b.krate, pb))
        });
        let mut changed = vec![false; crates.len()];
        for d in done {
            let c = &mut crates[d.krate];
            c.errors.extend(d.errors);
            c.loaded.extend(d.loaded);
            if let Some(f) = d.file {
                c.files.push(f);
                c.parent_pos.push(d.parent);
                changed[d.krate] = true;
            }
        }
        for (c, changed) in crates.iter_mut().zip(changed) {
            if changed {
                c.sort_files();
            }
        }
    }

    fn parse_job<'s>(&'s self, job: Job<'s>, queue: &pool::Queue<Job<'s>>) -> Done<'s> {
        let parent = job.decl.as_ref().map(|(k, n, _)| (k.clone(), *n));
        let mut done = Done { krate: job.krate, file: None, parent, errors: Vec::new(), loaded: Vec::new() };
        let text = match std::fs::read_to_string(&job.path) {
            Ok(t) => t,
            Err(e) => {
                let at = job.decl.as_ref().map_or("", |d| d.2);
                done.errors.push(Error::new(at, format!("cannot read `{}`: {e}", job.path.display())));
                return done;
            }
        };
        let source = self.sources.add(Source::new(job.path.display().to_string(), text));
        let mut ast = match parse_str(&source.text, job.edition) {
            Ok(ast) => ast,
            Err(e) => {
                done.errors.push(e);
                return done;
            }
        };
        if let Some(cfg) = &self.options.cfg {
            let eof = Token(&source.text[source.text.len()..]);
            done.errors.extend(strip(&mut ast, cfg, job.edition, eof));
        }
        let file = ParsedFile {
            derived: HashSet::new(),
            source,
            path: job.path,
            mod_rs: job.mod_rs,
            ast,
            module: job.module,
            scope: job.scope,
            pos: job.pos,
            parent: None,
        };
        if job.follow {
            let (jobs, errors) = module_jobs(job.krate, &file, job.edition, &HashSet::new());
            done.errors.extend(errors);
            for j in jobs {
                done.loaded.push((file.pos.clone(), j.decl.as_ref().unwrap().1));
                queue.push(j);
            }
        }
        done.file = Some(file);
        done
    }

    /// Loads the files of `mod` items that are new since the last load
    /// (made by macro expansion). Returns how many files were loaded.
    pub fn load_new_modules<'s>(&'s self, crates: &mut [ParsedCrate<'s>]) -> usize {
        let mut jobs = Vec::new();
        for (k, c) in crates.iter_mut().enumerate() {
            if !c.follow_modules {
                continue;
            }
            for f in &c.files {
                let (js, errors) = module_jobs(k, f, c.edition, &c.loaded);
                c.errors.extend(errors);
                jobs.extend(js);
            }
            for j in &jobs {
                if j.krate == k
                    && let Some((p, n, _)) = &j.decl
                {
                    c.loaded.insert((p.clone(), *n));
                }
            }
        }
        let before: usize = crates.iter().map(|c| c.files.len()).sum();
        self.load(crates, jobs);
        crates.iter().map(|c| c.files.len()).sum::<usize>() - before
    }
}

impl<'s> ParsedCrate<'s> {
    /// Puts the files in text order and links each to its parent.
    fn sort_files(&mut self) {
        let mut order: Vec<usize> = (0..self.files.len()).collect();
        order.sort_by(|&a, &b| self.files[a].pos.cmp(&self.files[b].pos));
        let mut files: Vec<Option<ParsedFile<'s>>> = std::mem::take(&mut self.files).into_iter().map(Some).collect();
        let mut parents: Vec<Option<(Vec<u32>, NodeId)>> = std::mem::take(&mut self.parent_pos);
        let mut sorted = Vec::with_capacity(files.len());
        let mut sorted_parents = Vec::with_capacity(files.len());
        for i in order {
            sorted.push(files[i].take().unwrap());
            sorted_parents.push(parents[i].take());
        }
        let index: std::collections::HashMap<Vec<u32>, usize> =
            sorted.iter().enumerate().map(|(i, f)| (f.pos.clone(), i)).collect();
        for (f, p) in sorted.iter_mut().zip(&sorted_parents) {
            f.parent = p.as_ref().and_then(|(pp, n)| index.get(pp).map(|&i| (i, *n)));
        }
        self.files = sorted;
        self.parent_pos = sorted_parents;
    }
}

/// The jobs for a file's out-of-line modules not yet in `loaded`, and
/// errors for modules with no file.
fn module_jobs<'s>(
    krate: usize,
    file: &ParsedFile<'s>,
    edition: Edition,
    loaded: &HashSet<(Vec<u32>, NodeId)>,
) -> (Vec<Job<'s>>, Vec<Error<'s>>) {
    let ast = &file.ast;
    let file_dir = file.path.parent().unwrap_or(Path::new("")).to_path_buf();
    let children_dir =
        if file.mod_rs { file_dir.clone() } else { file_dir.join(file.path.file_stem().unwrap_or_default()) };
    let mut found = Vec::new();
    scan_modules(ast, ast.root, &file_dir, &children_dir, true, &mut found);
    let mut jobs = Vec::new();
    let mut errors = Vec::new();
    for m in found {
        if loaded.contains(&(file.pos.clone(), m.item)) {
            continue;
        }
        let mut pos = file.pos.clone();
        pos.extend(node_pos(ast, m.item));
        let mut module = file.module.clone();
        let mut scope = file.scope.clone();
        for step in &m.inline {
            module.push(step.name.clone());
            scope.push(step.clone());
        }
        module.push(m.name.to_string());
        scope.push(ScopeStep { name: m.name.to_string(), macro_use: has_attribute(ast, m.item, "macro_use") });
        let decl = Some((file.pos.clone(), m.item, m.name));
        match m.file {
            Ok((path, mod_rs)) => {
                jobs.push(Job { krate, path, pos, module, scope, mod_rs, edition, follow: true, decl });
            }
            // Reported in the module's place, as if it were a file that
            // failed.
            Err(Some(message)) => errors.push(Error::new(m.name, message)),
            // Missing, but under `cfg`: left to cfg evaluation.
            Err(None) => {}
        }
    }
    (jobs, errors)
}

/// An out-of-line module found in a file.
struct FoundModule<'s> {
    item: NodeId,
    name: &'s str,
    /// Inline modules between the file and this one.
    inline: Vec<ScopeStep>,
    /// The module's file and whether it owns its directory; or why there
    /// is none (`None` when it may be configured away).
    file: Result<(PathBuf, bool), Option<String>>,
}

/// Finds the `mod name;` items under `node` (the file's root or an inline
/// module) and works out their files, as the Reference's module rules
/// say: `dir/name.rs` or `dir/name/mod.rs`, or a `#[path]`, which is
/// relative to the file's own directory outside inline modules and to the
/// inline modules' directory inside them.
fn scan_modules<'s>(
    ast: &Ast<'s>,
    node: NodeId,
    file_dir: &Path,
    dir: &Path,
    top: bool,
    out: &mut Vec<FoundModule<'s>>,
) {
    scan_into(ast, node, file_dir, dir, top, &mut Vec::new(), out);
}

fn scan_into<'s>(
    ast: &Ast<'s>,
    node: NodeId,
    file_dir: &Path,
    dir: &Path,
    top: bool,
    inline: &mut Vec<ScopeStep>,
    out: &mut Vec<FoundModule<'s>>,
) {
    for &item in ast.children(node) {
        if ast.kind(item) != Kind::Mod {
            continue;
        }
        let Some(name_node) = ast.child(item, Kind::Name) else { continue };
        let name = ast.token(name_node).ident_name();
        let path_attr = attribute_value(ast, item, "path");
        let configured = has_attribute(ast, item, "cfg") || has_attribute(ast, item, "cfg_attr");
        let is_inline = ast.tokens_of(item).last().is_some_and(|t| t.0 == "}");
        if is_inline {
            let sub = match &path_attr {
                Some(p) => dir.join(p),
                None => dir.join(name),
            };
            inline.push(ScopeStep { name: name.to_string(), macro_use: has_attribute(ast, item, "macro_use") });
            scan_into(ast, item, file_dir, &sub, false, inline, out);
            inline.pop();
            continue;
        }
        let file = if let Some(p) = path_attr {
            let path = if top { file_dir.join(p) } else { dir.join(p) };
            if path.exists() || !configured { Ok((path, true)) } else { Err(None) }
        } else {
            let flat = dir.join(format!("{name}.rs"));
            let nested = dir.join(name).join("mod.rs");
            match (flat.exists(), nested.exists()) {
                (true, false) => Ok((flat, false)),
                (false, true) => Ok((nested, true)),
                (true, true) => {
                    Err(Some(format!("module `{name}` is in both `{}` and `{}`", flat.display(), nested.display())))
                }
                (false, false) if configured => Err(None),
                (false, false) => Err(Some(format!(
                    "no file for module `{name}`: neither `{}` nor `{}`",
                    flat.display(),
                    nested.display()
                ))),
            }
        };
        out.push(FoundModule { item, name, inline: inline.clone(), file });
    }
}

/// The attributes of an item, as (name, node).
pub(crate) fn attributes<'p>(ast: &'p Ast, item: NodeId) -> impl Iterator<Item = (String, NodeId)> + 'p {
    ast.children(item).iter().copied().filter(move |&a| ast.kind(a) == Kind::Attribute).filter_map(move |a| {
        let path = ast.child(a, Kind::Path)?;
        Some((ast.text(path), a))
    })
}

pub(crate) fn has_attribute(ast: &Ast, item: NodeId, name: &str) -> bool {
    attributes(ast, item).any(|(n, _)| n == name)
}

/// The string in `#[name = "value"]`, without quotes.
pub(crate) fn attribute_value(ast: &Ast, item: NodeId, name: &str) -> Option<String> {
    let (_, a) = attributes(ast, item).find(|(n, _)| n == name)?;
    let value = *ast.children(a).last()?;
    if ast.kind(value) != Kind::Literal {
        return None;
    }
    let t = ast.token(value).0;
    let body = t.trim_start_matches('r').trim_matches('#');
    Some(body.strip_prefix('"')?.strip_suffix('"')?.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Writes files under a fresh directory in the system's temporary
    /// directory, and returns it.
    fn tree(name: &str, files: &[(&str, &str)]) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("epitome-rustc2-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        for (path, text) in files {
            let p = dir.join(path);
            std::fs::create_dir_all(p.parent().unwrap()).unwrap();
            std::fs::write(p, text).unwrap();
        }
        dir
    }

    fn modules(c: &ParsedCrate) -> Vec<String> {
        c.files.iter().map(|f| format!("{} {}", f.module.join("::"), f.parent.map_or(-1, |p| p.0 as i64))).collect()
    }

    #[test]
    fn follows_modules_in_tree_order() {
        let dir = tree(
            "mods",
            &[
                ("src/lib.rs", "mod a; mod b; mod c { mod d; } #[path = \"other.rs\"] mod e;"),
                ("src/a.rs", "mod x;"),
                ("src/a/x.rs", "fn f() {}"),
                ("src/b/mod.rs", "mod y;"),
                ("src/b/y.rs", ""),
                ("src/c/d.rs", ""),
                ("src/other.rs", "mod z;"),
                ("src/z.rs", ""),
            ],
        );
        for threads in [1, 4] {
            let s = Session::new(Options { threads, ..Default::default() });
            let crates = s.parse_crates(vec![CrateRoot::new("t", dir.join("src/lib.rs"))]);
            let c = &crates[0];
            assert!(c.errors.is_empty(), "{}", c.errors[0].render_in(&s.sources));
            assert_eq!(modules(c), [" -1", "a 0", "a::x 1", "b 0", "b::y 3", "c::d 0", "e 0", "e::z 6"]);
        }
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn missing_modules() {
        let dir = tree("missing", &[("src/main.rs", "#[cfg(windows)] mod w; mod gone;")]);
        let s = Session::new(Options::default());
        let crates = s.parse_crates(vec![CrateRoot::new("t", dir.join("src/main.rs"))]);
        let c = &crates[0];
        assert_eq!(c.files.len(), 1);
        assert_eq!(c.errors.len(), 1);
        let msg = c.errors[0].render_in(&s.sources);
        assert!(msg.contains("main.rs:1:28: error: no file for module `gone`"), "{msg}");
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn several_crates_at_once() {
        let dir = tree("many", &[("a/lib.rs", "mod m;"), ("a/m.rs", ""), ("b/lib.rs", "fn (")]);
        let s = Session::new(Options { threads: 3, ..Default::default() });
        let crates =
            s.parse_crates(vec![CrateRoot::new("a", dir.join("a/lib.rs")), CrateRoot::new("b", dir.join("b/lib.rs"))]);
        assert_eq!(crates[0].files.len(), 2);
        assert!(crates[0].errors.is_empty());
        assert_eq!(crates[1].files.len(), 0);
        assert_eq!(crates[1].errors.len(), 1);
        let _ = std::fs::remove_dir_all(dir);
    }
}
