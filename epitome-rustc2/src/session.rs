//! The compiler as a library: a `Session` holds what all crates of a run
//! share (the source map, the options), and loads crates in parallel.
//!
//! ```no_run
//! use epitome_rustc2::session::{CrateRoot, Options, Session};
//! let session = Session::new(Options::default());
//! let crates = session.parse_crates(vec![CrateRoot::new("hello", "src/main.rs")]);
//! for f in &crates[0].files {
//!     println!("{} ({} nodes)", f.source.name, f.ast.nodes.len());
//! }
//! ```
//!
//! Parsing a file finds its modules (`mod foo;`), and each module's file is
//! a new job on the pool, so the files of all crates are read and parsed
//! at once, as they are found. The result does not depend on the order the
//! jobs ran in: each file is placed by its position in the module tree.

use crate::ast::{Ast, Kind, NodeId};
use crate::lexer::Edition;
use crate::parse::parse_str;
use crate::pool;
use crate::source::{Error, Source, SourceMap};
use std::path::{Path, PathBuf};
use std::sync::Mutex;

/// Options for a whole run.
#[derive(Clone, Debug, Default, Default)]
pub struct Options {
    /// Worker threads; 0 means one per core.
    pub threads: usize,
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
}

impl CrateRoot {
    pub fn new(name: impl Into<String>, root: impl Into<PathBuf>) -> Self {
        CrateRoot { name: name.into(), root: root.into(), edition: Edition::default(), follow_modules: true }
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

/// One parsed file of a crate.
pub struct ParsedFile<'s> {
    pub source: &'s Source,
    pub ast: Ast<'s>,
    /// The module's path from the crate root: `["a", "b"]` for
    /// `crate::a::b`; empty for the root.
    pub module: Vec<String>,
    /// The file holding this module's `mod` item, and that item.
    pub parent: Option<(usize, NodeId)>,
}

/// A crate's files, in module-tree order (the root first, then each
/// module's file after its parent's, in the order of the `mod` items).
pub struct ParsedCrate<'s> {
    pub name: String,
    pub edition: Edition,
    pub files: Vec<ParsedFile<'s>>,
    /// Every error found, in the same order as the files.
    pub errors: Vec<Error<'s>>,
}

pub struct Session {
    pub sources: SourceMap,
    pub options: Options,
}

/// Reading and parsing one file.
struct Job<'s> {
    krate: usize,
    path: PathBuf,
    /// The file's place in the module tree: the index of each `mod` item
    /// on the way down, counting in source order within each file.
    key: Vec<u32>,
    module: Vec<String>,
    /// Whether the file owns its directory (`lib.rs`, `main.rs`, `mod.rs`,
    /// or a `#[path]` file), so that its modules are beside it.
    mod_rs: bool,
    /// The parent's key and `mod` item, and the item's name for errors.
    decl: Option<(Vec<u32>, NodeId, &'s str)>,
}

/// What a job produced.
struct Done<'s> {
    krate: usize,
    key: Vec<u32>,
    file: Option<ParsedFile<'s>>,
    parent: Option<(Vec<u32>, NodeId)>,
    error: Option<Error<'s>>,
}

impl Session {
    pub fn new(options: Options) -> Self {
        Session { sources: SourceMap::new(), options }
    }

    fn threads(&self) -> usize {
        if self.options.threads == 0 { pool::default_threads() } else { self.options.threads }
    }

    /// Loads and parses crates, all at once.
    pub fn parse_crates(&self, roots: Vec<CrateRoot>) -> Vec<ParsedCrate<'_>> {
        let jobs = roots
            .iter()
            .enumerate()
            .map(|(k, r)| Job {
                krate: k,
                path: r.root.clone(),
                key: Vec::new(),
                module: Vec::new(),
                mod_rs: true,
                decl: None,
            })
            .collect();
        let done: Mutex<Vec<Done>> = Mutex::new(Vec::new());
        pool::run(self.threads(), jobs, |job, queue| {
            let result = self.parse_job(&roots[job.krate], job, queue);
            done.lock().unwrap().push(result);
        });
        let mut done = done.into_inner().unwrap();
        done.sort_by(|a, b| (a.krate, &a.key).cmp(&(b.krate, &b.key)));

        let mut crates: Vec<ParsedCrate> = roots
            .iter()
            .map(|r| ParsedCrate { name: r.name.clone(), edition: r.edition, files: Vec::new(), errors: Vec::new() })
            .collect();
        // Index of each file by key, to link files to their parents.
        let mut index: Vec<std::collections::HashMap<Vec<u32>, usize>> = vec![Default::default(); crates.len()];
        for d in done {
            let c = &mut crates[d.krate];
            if let Some(e) = d.error {
                c.errors.push(e);
            }
            if let Some(mut f) = d.file {
                if let Some((pkey, node)) = d.parent {
                    f.parent = index[d.krate].get(&pkey).map(|&i| (i, node));
                }
                index[d.krate].insert(d.key, c.files.len());
                c.files.push(f);
            }
        }
        crates
    }

    fn parse_job<'s>(&'s self, root: &CrateRoot, job: Job<'s>, queue: &pool::Queue<Job<'s>>) -> Done<'s> {
        let parent = job.decl.as_ref().map(|(k, n, _)| (k.clone(), *n));
        let mut done = Done { krate: job.krate, key: job.key.clone(), file: None, parent, error: None };
        let text = match std::fs::read_to_string(&job.path) {
            Ok(t) => t,
            Err(e) => {
                let at = job.decl.as_ref().map_or("", |d| d.2);
                done.error = Some(Error::new(at, format!("cannot read `{}`: {e}", job.path.display())));
                return done;
            }
        };
        let source = self.sources.add(Source::new(job.path.display().to_string(), text));
        let ast = match parse_str(&source.text, root.edition) {
            Ok(ast) => ast,
            Err(e) => {
                done.error = Some(e);
                return done;
            }
        };
        if root.follow_modules {
            let file_dir = job.path.parent().unwrap_or(Path::new("")).to_path_buf();
            let children_dir =
                if job.mod_rs { file_dir.clone() } else { file_dir.join(job.path.file_stem().unwrap_or_default()) };
            let mut found = Vec::new();
            scan_modules(&ast, ast.root, &file_dir, &children_dir, true, &mut found);
            for (ordinal, m) in found.into_iter().enumerate() {
                let mut key = job.key.clone();
                key.push(ordinal as u32);
                let mut module = job.module.clone();
                module.extend(m.inline.iter().cloned());
                module.push(m.name.to_string());
                let decl = Some((job.key.clone(), m.item, m.name));
                match m.file {
                    Ok((path, mod_rs)) => {
                        queue.push(Job { krate: job.krate, path, key, module, mod_rs, decl });
                    }
                    Err(Some(message)) => {
                        // Reported in the module's place, as if it were a
                        // file that failed.
                        done.error.get_or_insert(Error::new(m.name, message));
                    }
                    // Missing, but under `cfg`: left to cfg evaluation.
                    Err(None) => {}
                }
            }
        }
        done.file = Some(ParsedFile { source, ast, module: job.module, parent: None });
        done
    }
}

/// An out-of-line module found in a file.
struct FoundModule<'s> {
    item: NodeId,
    name: &'s str,
    /// Inline modules between the file and this one.
    inline: Vec<String>,
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
    inline: &mut Vec<String>,
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
            inline.push(name.to_string());
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
fn attributes<'p>(ast: &'p Ast, item: NodeId) -> impl Iterator<Item = (String, NodeId)> + 'p {
    ast.children(item).iter().copied().filter(move |&a| ast.kind(a) == Kind::Attribute).filter_map(move |a| {
        let path = ast.child(a, Kind::Path)?;
        Some((ast.text(path), a))
    })
}

fn has_attribute(ast: &Ast, item: NodeId, name: &str) -> bool {
    attributes(ast, item).any(|(n, _)| n == name)
}

/// The string in `#[name = "value"]`, without quotes.
fn attribute_value(ast: &Ast, item: NodeId, name: &str) -> Option<String> {
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
            let s = Session::new(Options { threads });
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
        let s = Session::new(Options { threads: 3 });
        let crates =
            s.parse_crates(vec![CrateRoot::new("a", dir.join("a/lib.rs")), CrateRoot::new("b", dir.join("b/lib.rs"))]);
        assert_eq!(crates[0].files.len(), 2);
        assert!(crates[0].errors.is_empty());
        assert_eq!(crates[1].files.len(), 0);
        assert_eq!(crates[1].errors.len(), 1);
        let _ = std::fs::remove_dir_all(dir);
    }
}
