//! Macro expansion: `macro_rules!`, macros 2.0 (`macro m { ... }`), and the
//! built-in macros, across all the crates of a session.
//!
//! Expansion runs in rounds, each in parallel over every file:
//!
//! 1. **Collect** the macro definitions of every file, with where each is
//!    in the crate's text (`ParsedFile::pos` and the token) and in which
//!    module it is visible.
//! 2. **Expand** every call in every file whose macro can be found, and
//!    the calls in what they expand to, up to the recursion limit. Only
//!    the file being expanded is changed, so files are independent jobs.
//! 3. **Load** the files of any `mod` items expansion made.
//!
//! Rounds repeat until one changes nothing: macros made by macros, and
//! modules declared inside `cfg_select!`, take a round each.
//!
//! A call `m!` finds, in order: the last `macro_rules! m` before it in its
//! module or an enclosing one (a module's macros reach its parent after
//! the `mod` item if it has `#[macro_use]`); a macro exported by a crate
//! in its macro prelude (`#[macro_use] extern crate`, and `std` or `core`
//! unless `no_std` or `no_core`); a built-in. A path (`$crate::m!`,
//! `core::m!`, `self::a::m!`) looks in the named crate (its exported
//! macros, then any macro of that name).
//!
//! Macros brought in by `use` are found by name in the crate (or a prelude
//! crate) for now; name resolution will follow the `use` itself.

pub mod builtin;
pub mod mbe;

pub(crate) use mbe::match_delims as mbe_delims;

use crate::ast::{Ast, Kind, NodeId};
use crate::lexer::{Edition, Token, lex};
use crate::parse::{Fragment, parse_fragment};
use crate::pool;
use crate::session::{ParsedCrate, ParsedFile, ScopeStep, Session, has_attribute, node_pos};
use crate::source::Error;
use mbe::{Rules, Transcribe};
use std::collections::{BTreeMap, HashMap};
use std::sync::Mutex;

/// How deep expansions may nest, as rustc's default `recursion_limit`.
const RECURSION_LIMIT: usize = 128;

/// Rounds of expansion before giving up.
const MAX_ROUNDS: usize = 64;

/// Text that synthetic tokens are slices of.
static SYNTH: &str = "():::crate";

fn synth(range: std::ops::Range<usize>) -> Token<'static> {
    Token(&SYNTH[range])
}

/// Whether a token is one expansion made up (a parenthesis around a
/// transcribed expression), not a slice of any source.
pub fn is_synthetic(t: Token) -> bool {
    let p = t.0.as_ptr() as usize;
    let s = SYNTH.as_ptr() as usize;
    p >= s && p < s + SYNTH.len()
}

/// What a macro is.
pub enum MacroKind<'s> {
    Rules(Rules<'s>),
    /// A procedural macro: its library, and which kind it is.
    Proc(std::path::PathBuf, crate::proc_macro::ProcMacroKind),
    /// A built-in, by its name: `#[rustc_builtin_macro]` on a definition,
    /// or a known name when no definition is found.
    Builtin(&'s str),
}

/// A macro definition.
pub struct MacroDef<'s> {
    pub name: &'s str,
    pub krate: usize,
    /// Its file (an index into the crate's files) and item.
    pub file: u32,
    pub node: NodeId,
    /// Where it is in the crate's text.
    pub pos: Vec<u32>,
    /// The module path it is visible in (and below), after `#[macro_use]`.
    pub scope: Vec<String>,
    /// `#[macro_export]`: at the crate root, for other crates.
    pub exported: bool,
    /// A macro 2.0 (`macro m`), found by path, not by text order.
    pub item: bool,
    pub kind: MacroKind<'s>,
}

/// What a session's expansion did, for progress reports.
#[derive(Default, Debug)]
pub struct ExpandStats {
    pub rounds: usize,
    pub expanded: usize,
    /// Calls to built-ins that are left for later stages (`format_args!`,
    /// `asm!`).
    pub builtins_left: usize,
    /// Calls whose macro was not found, by name.
    pub unresolved: BTreeMap<String, usize>,
    /// Calls whose macro was found only by searching for its name (not by
    /// scope, path or import): proc macros of crates outside the session.
    pub found_by_name: BTreeMap<String, usize>,
    pub files_loaded: usize,
    /// Calls expanded in each round.
    pub per_round: Vec<usize>,
    /// Calls expanded in the last round, by macro: what kept the rounds
    /// going if they reached the limit.
    pub last_round: BTreeMap<String, usize>,
}

/// Every macro of the session, indexed for lookup.
struct Macros<'s> {
    defs: Vec<MacroDef<'s>>,
    /// Per crate: name to definitions, in text order.
    by_name: Vec<HashMap<&'s str, Vec<usize>>>,
    /// Per crate: exported name to definition (the last).
    exported: Vec<HashMap<&'s str, usize>>,
    /// Per crate: crates whose exported macros are in scope by name.
    prelude: Vec<Vec<usize>>,
    /// Per crate: the tokens `$crate` becomes in another crate (`::name`).
    crate_path: Vec<Vec<Token<'s>>>,
    /// Definitions by crate, file and item, to match name resolution's.
    by_node: HashMap<(usize, u32, NodeId), usize>,
}

impl Session {
    /// Expands every macro call it can in `crates`, loading the files of
    /// any modules that expansion declares.
    pub fn expand<'s>(&'s self, crates: &mut [ParsedCrate<'s>]) -> ExpandStats {
        let mut stats = ExpandStats::default();
        let crate_path: Vec<Vec<Token<'s>>> = crates
            .iter()
            .map(|c| {
                let name = self.add_text("<crate name>", c.name.clone());
                vec![synth(2..3), synth(3..4), Token(name)]
            })
            .collect();
        let crate_index: HashMap<String, usize> = crates.iter().enumerate().map(|(i, c)| (c.name.clone(), i)).collect();
        // Calls that fail are tried again each round (a later round may
        // define what they need), so only the last round's errors count.
        let mut round_errors: Vec<(usize, Error<'s>)> = Vec::new();
        let procs = self.list_proc_macros(crates);
        for round in 0..MAX_ROUNDS {
            stats.rounds = round + 1;
            let (mut macros, def_errors) = self.collect_macros(crates, &crate_index, &crate_path);
            macros.add_proc_macros(&procs);
            if round == 0 {
                // Definitions that failed to compile, reported once.
                for (k, errors) in def_errors {
                    crates[k].errors.extend(errors);
                }
            }
            let results: Mutex<Vec<(usize, FileResult<'s>)>> = Mutex::new(Vec::new());
            let resolver = crate::resolve::module_tree(crates);
            let editions: Vec<Edition> = crates.iter().map(|c| c.edition).collect();
            let names: Vec<String> = crates.iter().map(|c| c.name.clone()).collect();
            let dirs: Vec<std::path::PathBuf> = crates.iter().map(|c| c.dir.clone()).collect();
            let jobs: Vec<(usize, usize, &mut ParsedFile<'s>)> = crates
                .iter_mut()
                .enumerate()
                .flat_map(|(k, c)| c.files.iter_mut().enumerate().map(move |(i, f)| (k, i, f)))
                .collect();
            pool::run(self.threads(), jobs, |(k, i, file), _| {
                let mut x = Expander {
                    session: self,
                    macros: &macros,
                    resolver: &resolver,
                    file_index: i as u32,
                    mods: vec![resolver.file_modules[k][i]],
                    krate: k,
                    found_by_name: std::cell::RefCell::new(Vec::new()),
                    dir: &dirs[k],
                    crate_name: &names[k],
                    edition: editions[k],
                    result: FileResult::default(),
                };
                x.expand_file(file);
                x.result.found_by_name = x.found_by_name.take();
                results.lock().unwrap().push((k, x.result));
            });
            let mut results = results.into_inner().unwrap();
            results.sort_by_key(|r| r.0);
            let mut expanded = 0;
            stats.last_round.clear();
            round_errors.clear();
            for (k, r) in results {
                expanded += r.expanded;
                stats.builtins_left += r.builtins_left;
                for n in r.unresolved {
                    *stats.unresolved.entry(n.to_string()).or_default() += 1;
                }
                for n in r.names {
                    *stats.last_round.entry(n.to_string()).or_default() += 1;
                }
                for n in r.found_by_name {
                    *stats.found_by_name.entry(n.to_string()).or_default() += 1;
                }
                round_errors.extend(r.errors.into_iter().map(|e| (k, e)));
            }
            stats.expanded += expanded;
            stats.per_round.push(expanded);
            let loaded = self.load_new_modules(crates);
            stats.files_loaded += loaded;
            if expanded == 0 && loaded == 0 {
                break;
            }
            // Calls left over are counted again next round; keep only the
            // last round's.
            stats.builtins_left = 0;
            stats.unresolved.clear();
        }
        for (k, e) in round_errors {
            crates[k].errors.push(e);
        }
        stats
    }

    /// Finds every macro definition, in parallel over files.
    #[allow(clippy::type_complexity)]
    fn collect_macros<'s>(
        &'s self,
        crates: &[ParsedCrate<'s>],
        crate_index: &HashMap<String, usize>,
        crate_path: &[Vec<Token<'s>>],
    ) -> (Macros<'s>, Vec<(usize, Vec<Error<'s>>)>) {
        let found: Mutex<Vec<(usize, Vec<MacroDef<'s>>, Vec<Error<'s>>)>> = Mutex::new(Vec::new());
        let jobs: Vec<(usize, u32, &ParsedFile<'s>)> = crates
            .iter()
            .enumerate()
            .flat_map(|(k, c)| c.files.iter().enumerate().map(move |(i, f)| (k, i as u32, f)))
            .collect();
        pool::run(self.threads(), jobs, |(k, i, f), _| {
            let mut defs = Vec::new();
            let mut errors = Vec::new();
            let mut scope = f.scope.clone();
            let at = (f, k, i, crates[k].edition);
            collect_defs(&f.ast, f.ast.root, at, &mut scope, &mut defs, &mut errors);
            found.lock().unwrap().push((k, defs, errors));
        });
        let mut found = found.into_inner().unwrap();
        // A fixed order, whatever order the jobs ran in.
        found.sort_by(|a, b| (a.0, a.1.first().map(|d| &d.pos)).cmp(&(b.0, b.1.first().map(|d| &d.pos))));
        let mut m = Macros {
            defs: Vec::new(),
            by_name: vec![HashMap::new(); crates.len()],
            exported: vec![HashMap::new(); crates.len()],
            prelude: crates.iter().map(|c| macro_prelude(c, crate_index)).collect(),
            crate_path: crate_path.to_vec(),
            by_node: HashMap::new(),
        };
        let mut errs = Vec::new();
        let mut all: Vec<MacroDef<'s>> = Vec::new();
        for (k, defs, errors) in found {
            all.extend(defs);
            if !errors.is_empty() {
                errs.push((k, errors));
            }
        }
        all.sort_by(|a, b| (a.krate, &a.pos).cmp(&(b.krate, &b.pos)));
        for (i, d) in all.iter().enumerate() {
            m.by_node.insert((d.krate, d.file, d.node), i);
            m.by_name[d.krate].entry(d.name).or_default().push(i);
            if d.exported {
                m.exported[d.krate].insert(d.name, i);
            }
        }
        m.defs = all;
        (m, errs)
    }
}

impl<'s> Macros<'s> {
    /// Adds each crate's procedural macros, found by name like a macro
    /// brought in by `use`.
    fn add_proc_macros(&mut self, procs: &[Vec<(&'s str, std::path::PathBuf, crate::proc_macro::ProcMacroKind)>]) {
        for (k, list) in procs.iter().enumerate() {
            for (name, lib, kind) in list {
                let i = self.defs.len();
                self.defs.push(MacroDef {
                    name,
                    krate: k,
                    file: u32::MAX,
                    node: NodeId(0),
                    pos: Vec::new(),
                    scope: Vec::new(),
                    exported: false,
                    item: true,
                    kind: MacroKind::Proc(lib.clone(), *kind),
                });
                self.by_name[k].entry(name).or_default().push(i);
            }
        }
    }
}

impl Session {
    /// The procedural macros of each crate's libraries.
    #[allow(clippy::type_complexity)]
    fn list_proc_macros<'s>(
        &'s self,
        crates: &mut [ParsedCrate<'s>],
    ) -> Vec<Vec<(&'s str, std::path::PathBuf, crate::proc_macro::ProcMacroKind)>> {
        let mut cache: HashMap<std::path::PathBuf, Vec<(&'s str, crate::proc_macro::ProcMacroKind)>> = HashMap::new();
        let mut out = Vec::new();
        for c in crates.iter_mut() {
            let mut list = Vec::new();
            for (_, lib) in &c.proc_macros {
                let Some(server) = &self.proc_macro_server else {
                    c.errors.push(Error::new("", "proc-macro libraries need a proc-macro server"));
                    break;
                };
                if !cache.contains_key(lib) {
                    match server.list_macros(lib) {
                        Ok(ms) => {
                            let ms = ms.into_iter().map(|(n, k)| (self.add_text("<proc macro>", n), k)).collect();
                            cache.insert(lib.clone(), ms);
                        }
                        Err(e) => {
                            c.errors.push(Error::new("", e));
                            cache.insert(lib.clone(), Vec::new());
                        }
                    }
                }
                list.extend(cache[lib].iter().map(|(n, k)| (*n, lib.clone(), *k)));
            }
            out.push(list);
        }
        out
    }
}

/// The crates whose exported macros a crate can use by name: those of
/// `#[macro_use] extern crate`, and `std` (or `core` with `no_std`).
fn macro_prelude(c: &ParsedCrate, crate_index: &HashMap<String, usize>) -> Vec<usize> {
    let mut out = Vec::new();
    let Some(root) = c.files.first() else { return out };
    let ast = &root.ast;
    let inner = |name: &str| {
        ast.children(ast.root).iter().any(|&a| {
            ast.kind(a) == Kind::InnerAttribute && ast.child(a, Kind::Path).is_some_and(|p| ast.text(p) == name)
        })
    };
    for &item in ast.children(ast.root) {
        if ast.kind(item) == Kind::ExternCrate && has_attribute(ast, item, "macro_use") {
            let name = ast.child(item, Kind::Name).map(|n| ast.token(n).ident_name());
            if let Some(&i) = name.and_then(|n| crate_index.get(n)) {
                out.push(i);
            }
        }
    }
    if !inner("no_core") {
        let implicit = if inner("no_std") { "core" } else { "std" };
        if let Some(&i) = crate_index.get(implicit).or_else(|| crate_index.get("core")) {
            out.push(i);
        }
    }
    out
}

/// The module path a definition is visible in: its scope, less any
/// trailing `#[macro_use]` modules, whose macros reach their parent.
fn effective_scope(scope: &[ScopeStep]) -> Vec<String> {
    let mut n = scope.len();
    while n > 0 && scope[n - 1].macro_use {
        n -= 1;
    }
    scope[..n].iter().map(|s| s.name.clone()).collect()
}

fn collect_defs<'s>(
    ast: &Ast<'s>,
    id: NodeId,
    at: (&ParsedFile<'s>, usize, u32, Edition),
    scope: &mut Vec<ScopeStep>,
    out: &mut Vec<MacroDef<'s>>,
    errors: &mut Vec<Error<'s>>,
) {
    let (file, krate, file_index, edition) = at;
    for &k in ast.children(id) {
        match ast.kind(k) {
            Kind::MacroRules | Kind::MacroDef => {
                let Some(name) = ast.child(k, Kind::Name).map(|n| ast.token(n).ident_name()) else { continue };
                let mut pos = file.pos.clone();
                pos.extend(node_pos(ast, k));
                let item = ast.kind(k) == Kind::MacroDef;
                let builtin = ast.children(k).iter().copied().find(|&a| {
                    ast.kind(a) == Kind::Attribute
                        && ast.child(a, Kind::Path).is_some_and(|p| ast.text(p) == "rustc_builtin_macro")
                });
                let kind = if let Some(a) = builtin {
                    // `#[rustc_builtin_macro(core_panic)]` names the
                    // built-in; plain `#[rustc_builtin_macro]` is the
                    // macro's own name.
                    let named = ast
                        .child(a, Kind::TokenTree)
                        .and_then(|tt| ast.tokens_of(tt).get(1).copied())
                        .filter(|t| t.is_ident())
                        .map(|t| t.0);
                    MacroKind::Builtin(named.unwrap_or(name))
                } else {
                    let tts: Vec<NodeId> =
                        ast.children(k).iter().copied().filter(|&c| ast.kind(c) == Kind::TokenTree).collect();
                    let compiled = match tts.as_slice() {
                        // `macro_rules! m { rules }` and `macro m { rules }`.
                        [body] => Rules::compile(inner(ast, *body), edition),
                        // `macro m(args) { body }`: one rule.
                        [args, body] => Rules::compile_one(ast.tokens_of(*args), ast.tokens_of(*body), edition),
                        _ => continue,
                    };
                    match compiled {
                        Ok(r) => MacroKind::Rules(r),
                        Err(e) => {
                            errors.push(e);
                            continue;
                        }
                    }
                };
                out.push(MacroDef {
                    name,
                    krate,
                    file: file_index,
                    node: k,
                    pos,
                    scope: effective_scope(scope),
                    exported: has_attribute(ast, k, "macro_export"),
                    item,
                    kind,
                });
            }
            Kind::Mod => {
                let name = ast.child(k, Kind::Name).map_or("", |n| ast.token(n).ident_name());
                scope.push(ScopeStep { name: name.to_string(), macro_use: has_attribute(ast, k, "macro_use") });
                collect_defs(ast, k, at, scope, out, errors);
                scope.pop();
            }
            Kind::Block => {
                scope.push(ScopeStep { name: block_scope(file, k), macro_use: false });
                collect_defs(ast, k, at, scope, out, errors);
                scope.pop();
            }
            Kind::TokenTree => {}
            kind if !kind.is_leaf() => collect_defs(ast, k, at, scope, out, errors),
            _ => {}
        }
    }
}

/// A name for a block's scope, unique in the crate.
fn block_scope(file: &ParsedFile, block: NodeId) -> String {
    format!("{{{:?}:{}}}", file.pos, block.0)
}

/// The tokens between a token tree's delimiters.
fn inner<'p, 'a>(ast: &'p Ast<'a>, tt: NodeId) -> &'p [Token<'a>] {
    let t = ast.tokens_of(tt);
    if t.len() >= 2 { &t[1..t.len() - 1] } else { &[] }
}

#[derive(Default)]
struct FileResult<'s> {
    expanded: usize,
    found_by_name: Vec<&'s str>,
    names: Vec<&'s str>,
    builtins_left: usize,
    unresolved: Vec<&'s str>,
    errors: Vec<Error<'s>>,
}

/// Where a call is, which decides what its expansion is parsed as.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Place {
    Items,
    /// Statements; `true` if the call was followed by `;`.
    Stmts(bool),
    Expr,
    Type,
    Pattern,
}

struct Expander<'x, 's> {
    session: &'s Session,
    macros: &'x Macros<'s>,
    /// The module tree and imports as the round began.
    resolver: &'x crate::resolve::Resolver<'s>,
    file_index: u32,
    /// The modules the walk is in (file, inline modules, blocks with items).
    mods: Vec<crate::resolve::ModId>,
    krate: usize,
    /// Calls found only by searching for their macro's name.
    found_by_name: std::cell::RefCell<Vec<&'s str>>,
    /// The crate root's directory.
    dir: &'x std::path::Path,
    crate_name: &'x str,
    edition: Edition,
    result: FileResult<'s>,
}

impl<'x, 's> Expander<'x, 's> {
    fn expand_file(&mut self, file: &mut ParsedFile<'s>) {
        let mut scope = file.scope.clone();
        let root = file.ast.root;
        self.walk(file, root, &mut scope, 0);
    }

    /// Expands the calls under `id`.
    fn walk(&mut self, file: &mut ParsedFile<'s>, id: NodeId, scope: &mut Vec<ScopeStep>, depth: usize) {
        let kind = file.ast.kind(id);
        let module = self.resolver.module_of.get(&(self.krate as u32, self.file_index, id)).copied();
        if let Some(m) = module {
            self.mods.push(m);
        }
        let pushed = match kind {
            Kind::Mod if id != file.ast.root => {
                let name = file.ast.child(id, Kind::Name).map_or("", |n| file.ast.token(n).ident_name());
                scope.push(ScopeStep { name: name.to_string(), macro_use: has_attribute(&file.ast, id, "macro_use") });
                true
            }
            Kind::Block => {
                scope.push(ScopeStep { name: block_scope(file, id), macro_use: false });
                true
            }
            _ => false,
        };
        let kids = file.ast.children(id).to_vec();
        let mut out = Vec::with_capacity(kids.len());
        let mut changed = false;
        for k in kids {
            // Attribute macros and derives on an item.
            if let Some(nodes) = self.expand_item_attrs(file, k, kind) {
                changed = true;
                let place = if kind == Kind::Block { Place::Stmts(false) } else { Place::Items };
                for n in nodes {
                    if n == k {
                        self.walk_child(file, k, scope, depth);
                        out.push(k);
                    } else {
                        self.walk_new(file, n, place, scope, depth + 1, &mut out);
                    }
                }
                continue;
            }
            let place = self.place_of(&file.ast, kind, k);
            match place {
                Some((call, place)) => match self.expand_call(file, call, place, scope) {
                    Some(nodes) => {
                        changed = true;
                        for n in nodes {
                            self.walk_new(file, n, place, scope, depth + 1, &mut out);
                        }
                    }
                    None => out.push(k),
                },
                None => {
                    self.walk_child(file, k, scope, depth);
                    out.push(k);
                }
            }
        }
        if changed {
            file.ast.set_children(id, &out);
        }
        if pushed {
            scope.pop();
        }
        if module.is_some() {
            self.mods.pop();
        }
    }

    fn walk_child(&mut self, file: &mut ParsedFile<'s>, k: NodeId, scope: &mut Vec<ScopeStep>, depth: usize) {
        let kind = file.ast.kind(k);
        if !kind.is_leaf()
            && !matches!(kind, Kind::MacroRules | Kind::MacroDef | Kind::Attribute | Kind::InnerAttribute)
        {
            self.walk(file, k, scope, depth);
        }
    }

    /// A node made by an expansion: if it is itself a call, expand it
    /// (one level deeper); otherwise expand the calls inside it.
    fn walk_new(
        &mut self,
        file: &mut ParsedFile<'s>,
        n: NodeId,
        place: Place,
        scope: &mut Vec<ScopeStep>,
        depth: usize,
        out: &mut Vec<NodeId>,
    ) {
        if depth > RECURSION_LIMIT {
            let at = file.ast.token(n).0;
            self.result.errors.push(Error::new(at, "recursion limit reached while expanding macros"));
            out.push(n);
            return;
        }
        let call = match file.ast.kind(n) {
            Kind::MacroCall | Kind::TypeMacro | Kind::PatMacro => Some(n),
            Kind::ExprStmt if matches!(place, Place::Stmts(_)) => {
                file.ast.children(n).iter().copied().find(|&c| file.ast.kind(c) == Kind::MacroCall)
            }
            _ => None,
        };
        let inner_place = match (place, file.ast.kind(n)) {
            (Place::Stmts(_), Kind::ExprStmt) => Place::Stmts(true),
            (Place::Stmts(_), Kind::MacroCall) => Place::Stmts(false),
            (p, _) => p,
        };
        if let Some(call) = call
            && let Some(nodes) = self.expand_call(file, call, inner_place, scope)
        {
            for m in nodes {
                self.walk_new(file, m, inner_place, scope, depth + 1, out);
            }
            return;
        }
        self.walk_child(file, n, scope, depth);
        out.push(n);
    }

    /// If child `k` of a node of kind `parent` is a macro call, the call
    /// and what its expansion is parsed as.
    fn place_of(&self, ast: &Ast<'s>, parent: Kind, k: NodeId) -> Option<(NodeId, Place)> {
        match ast.kind(k) {
            Kind::TypeMacro => Some((k, Place::Type)),
            Kind::PatMacro => Some((k, Place::Pattern)),
            Kind::MacroCall => Some((
                k,
                match parent {
                    Kind::Crate | Kind::Mod | Kind::Trait | Kind::Impl | Kind::ExternBlock => Place::Items,
                    // A tail call, or `m! { }` as a statement: its value is
                    // kept.
                    Kind::Block => Place::Stmts(false),
                    _ => Place::Expr,
                },
            )),
            Kind::ExprStmt if parent == Kind::Block => {
                let call = ast.children(k).iter().copied().find(|&c| ast.kind(c) == Kind::MacroCall)?;
                // `m!(...);`: the statements it gives, with the last one's
                // value dropped.
                (ast.children(k).len() == 1).then_some((call, Place::Stmts(true)))
            }
            _ => None,
        }
    }

    /// Expands one call. Returns the nodes that replace it, or `None` to
    /// leave it.
    fn expand_call(
        &mut self,
        file: &mut ParsedFile<'s>,
        call: NodeId,
        place: Place,
        scope: &[ScopeStep],
    ) -> Option<Vec<NodeId>> {
        let ast = &file.ast;
        let path = ast.child(call, Kind::Path)?;
        let tt = ast.child(call, Kind::TokenTree)?;
        let segs: Vec<&'s str> = ast
            .children(path)
            .iter()
            .filter(|&&s| ast.kind(s) == Kind::Segment)
            .map(|&s| ast.token(s).ident_name())
            .collect();
        let rooted = ast.child(path, Kind::PathRoot).is_some();
        let name = *segs.last()?;
        let mut pos = file.pos.clone();
        pos.extend(node_pos(ast, call));
        let module: Vec<String> = scope.iter().map(|s| s.name.clone()).collect();
        let def = self.resolve(&segs, rooted, &pos, &module);
        let input: Vec<Token<'s>> = self.desugar_docs(inner(ast, tt));
        let call_tok = ast.node(call).first as usize;
        let eof = ast.tokens_of(tt).last().map_or(Token(""), |t| Token(&t.0[..0]));
        let tokens = match def {
            Some(d) => match &self.macros.defs[d].kind {
                MacroKind::Rules(rules) => {
                    let make = |s: String| Token(self.session.add_text("<macro>", s));
                    let dollar_crate = if self.macros.defs[d].krate == self.krate {
                        vec![synth(5..10)]
                    } else {
                        self.macros.crate_path[self.macros.defs[d].krate].clone()
                    };
                    let mut cx = Transcribe { dollar_crate, make: &make, parens: (synth(0..1), synth(1..2)) };
                    match rules.expand(&input, eof, &mut cx) {
                        Ok(t) => t,
                        Err(msg) => {
                            let at = ast.token(path).0;
                            self.result.errors.push(Error::new(at, format!("in `{name}!`: {msg}")));
                            return None;
                        }
                    }
                }
                MacroKind::Builtin(b) => self.builtin(file, call, b, &input, place)?,
                MacroKind::Proc(lib, kind) => {
                    if *kind != crate::proc_macro::ProcMacroKind::FuncLike {
                        let at = ast.token(path).0;
                        self.result.errors.push(Error::new(at, format!("`{name}` is not a function-like macro")));
                        return None;
                    }
                    let lib = lib.clone();
                    self.run_proc_macro(file, call, &lib, name, &input, None)?
                }
            },
            None if builtin::is_builtin(name) => self.builtin(file, call, name, &input, place)?,
            None => {
                self.result.unresolved.push(name);
                return None;
            }
        };
        let nodes = self.splice(file, call, call_tok, &tokens, place, name)?;
        self.result.expanded += 1;
        self.result.names.push(name);
        Some(nodes)
    }

    /// Runs a procedural macro and lexes what it gives. Errors are
    /// reported at `at`.
    fn run_proc_macro(
        &mut self,
        file: &ParsedFile<'s>,
        at: NodeId,
        lib: &std::path::Path,
        name: &str,
        body: &[Token<'s>],
        attr: Option<&[Token<'s>]>,
    ) -> Option<Vec<Token<'s>>> {
        let server = self.session.proc_macro_server.as_ref()?;
        let site = file.ast.token(at).0;
        match server.expand(lib, name, body, attr, self.dir) {
            Ok(text) => match lex_text(self.session, &format!("<{name}!>"), text) {
                Ok(t) => Some(t),
                Err(e) => {
                    self.result.errors.push(Error::new(site, format!("output of `{name}`: {}", e.message)));
                    None
                }
            },
            Err(e) => {
                self.result.errors.push(Error::new(site, format!("proc macro `{name}`: {e}")));
                None
            }
        }
    }

    /// The tokens of an item as a macro sees it: printed (so `cfg`
    /// stripping shows), without the attributes in `skip`.
    fn item_tokens(&self, file: &mut ParsedFile<'s>, item: NodeId, skip: &[NodeId]) -> Option<Vec<Token<'s>>> {
        let kids = file.ast.children(item).to_vec();
        let kept: Vec<NodeId> = kids.iter().copied().filter(|k| !skip.contains(k)).collect();
        file.ast.set_children(item, &kept);
        let text = crate::pretty::pretty_node(&file.ast, item);
        file.ast.set_children(item, &kids);
        lex_text(self.session, "<item>", text).ok()
    }

    /// Expands attribute macros and derives on `item`, a child of a node
    /// of kind `parent`. Returns the nodes that take its place: an
    /// attribute macro's output, or the item followed by what its derives
    /// give. `None` if it has neither.
    fn expand_item_attrs(&mut self, file: &mut ParsedFile<'s>, item: NodeId, parent: Kind) -> Option<Vec<NodeId>> {
        if !is_item(file.ast.kind(item))
            || !matches!(parent, Kind::Crate | Kind::Mod | Kind::Block | Kind::Impl | Kind::Trait)
        {
            return None;
        }
        let place = if parent == Kind::Block { Place::Stmts(false) } else { Place::Items };
        let attrs: Vec<NodeId> =
            file.ast.children(item).iter().copied().filter(|&a| file.ast.kind(a) == Kind::Attribute).collect();
        let ast = &file.ast;
        let names = |a: NodeId| -> Vec<&'s str> {
            let Some(p) = ast.child(a, Kind::Path) else { return Vec::new() };
            ast.children(p)
                .iter()
                .filter(|&&s| ast.kind(s) == Kind::Segment)
                .map(|&s| ast.token(s).ident_name())
                .collect()
        };
        // An attribute macro: the item becomes its output.
        for &a in &attrs {
            let segs = names(a);
            let Some(&name) = segs.last() else { continue };
            if crate::expand::builtin::is_builtin(name) || matches!(name, "cfg" | "cfg_attr" | "derive") {
                continue;
            }
            let Some(d) = self.resolve(&segs, false, &[u32::MAX], &[]) else { continue };
            let MacroKind::Proc(lib, crate::proc_macro::ProcMacroKind::Attr) = &self.macros.defs[d].kind else {
                continue;
            };
            let lib = lib.clone();
            let args: Vec<Token<'s>> =
                file.ast.child(a, Kind::TokenTree).map_or(Vec::new(), |tt| inner(&file.ast, tt).to_vec());
            let body = self.item_tokens(file, item, &[a])?;
            let tokens = self.run_proc_macro(file, a, &lib, name, &body, Some(&args))?;
            let call_tok = file.ast.node(item).first as usize;
            let nodes = self.splice(file, item, call_tok, &tokens, place, name)?;
            self.result.expanded += 1;
            self.result.names.push(name);
            return Some(nodes);
        }
        // Derives: the item, then what each gives.
        if file.derived.contains(&item) {
            return None;
        }
        let mut derives = Vec::new();
        for &a in &attrs {
            if names(a).as_slice() != ["derive"] {
                continue;
            }
            let Some(tt) = file.ast.child(a, Kind::TokenTree) else { continue };
            for path in inner(&file.ast, tt).split(|t| t.0 == ",") {
                let segs: Vec<&'s str> = path.iter().filter(|t| t.is_ident()).map(|t| t.ident_name()).collect();
                if let Some(d) = self.resolve(&segs, false, &[u32::MAX], &[])
                    && let MacroKind::Proc(lib, crate::proc_macro::ProcMacroKind::Derive) = &self.macros.defs[d].kind
                {
                    derives.push((lib.clone(), *segs.last().unwrap(), a));
                }
            }
        }
        if derives.is_empty() {
            return None;
        }
        file.derived.insert(item);
        let derive_attrs: Vec<NodeId> = derives.iter().map(|d| d.2).collect();
        let body = self.item_tokens(file, item, &derive_attrs)?;
        let mut out = vec![item];
        for (lib, name, a) in derives {
            let Some(tokens) = self.run_proc_macro(file, a, &lib, name, &body, None) else { continue };
            let call_tok = file.ast.node(item).first as usize;
            if let Some(nodes) = self.splice(file, item, call_tok, &tokens, place, name) {
                self.result.expanded += 1;
                self.result.names.push(name);
                out.extend(nodes);
            }
        }
        Some(out)
    }

    /// Macro input with each doc comment as the attribute it stands for,
    /// `#[doc = r"text"]` (or `#![doc = ...]`), as macros see it.
    fn desugar_docs(&self, input: &[Token<'s>]) -> Vec<Token<'s>> {
        if !input.iter().any(|t| t.is_doc_comment()) {
            return input.to_vec();
        }
        let mut out = Vec::with_capacity(input.len());
        for &t in input {
            if !t.is_doc_comment() {
                out.push(t);
                continue;
            }
            let (inner, body) = doc_text(t.0);
            // A raw string with enough `#`s that the text cannot end it.
            let mut hashes = String::from("#");
            while body.contains(&format!("\"{hashes}")) {
                hashes.push('#');
            }
            let bang = if inner { "!" } else { "" };
            let text = format!("#{bang}[doc = r{hashes}\"{body}\"{hashes}]");
            match lex_text(self.session, "<doc comment>", text) {
                Ok(ts) => out.extend(ts),
                Err(_) => out.push(t),
            }
        }
        out
    }

    /// Runs a built-in; `None` leaves the call as it is.
    fn builtin(
        &mut self,
        file: &ParsedFile<'s>,
        call: NodeId,
        name: &str,
        input: &[Token<'s>],
        place: Place,
    ) -> Option<Vec<Token<'s>>> {
        let _ = place;
        let cx = builtin::Context { session: self.session, file, call, crate_name: self.crate_name };
        match builtin::expand(&cx, name, input) {
            Ok(Some(t)) => Some(t),
            Ok(None) => {
                self.result.builtins_left += 1;
                None
            }
            Err(e) => {
                self.result.errors.push(e);
                None
            }
        }
    }

    /// Parses `tokens` as what `place` needs, adds them and their nodes to
    /// the file, and returns the nodes that replace the call.
    fn splice(
        &mut self,
        file: &mut ParsedFile<'s>,
        call: NodeId,
        call_tok: usize,
        tokens: &[Token<'s>],
        place: Place,
        name: &str,
    ) -> Option<Vec<NodeId>> {
        let fragment = match place {
            Place::Items => Fragment::Items,
            Place::Stmts(_) => Fragment::Stmts,
            Place::Expr => Fragment::Expr,
            Place::Type => Fragment::Type,
            Place::Pattern => Fragment::Pattern,
        };
        let eof = tokens.last().map_or(Token(""), |t| Token(&t.0[t.0.len()..]));
        let frag = match parse_fragment(tokens.to_vec(), eof, self.edition, fragment) {
            Ok(f) => f,
            Err(e) => {
                let at = file.ast.token(call).0;
                self.result.errors.push(Error::new(at, format!("expanding `{name}!`: {}", e.message)));
                return None;
            }
        };
        let offset = file.ast.append_tokens(tokens, call_tok);
        let root = file.ast.graft(&frag, offset);
        let nodes = self.place_nodes(file, root, place);
        // What a macro gives is configured like any other code.
        Some(match &self.session.options.cfg {
            Some(cfg) => {
                let eof = Token(&file.source.text[file.source.text.len()..]);
                let (kept, errors) = crate::cfg::strip_new(&mut file.ast, &nodes, cfg, self.edition, eof);
                self.result.errors.extend(errors);
                kept
            }
            None => nodes,
        })
    }

    /// The nodes that replace a call, from the root of its parsed
    /// expansion.
    fn place_nodes(&mut self, file: &mut ParsedFile<'s>, root: NodeId, place: Place) -> Vec<NodeId> {
        let ast = &mut file.ast;
        match place {
            Place::Items => ast.children(root).to_vec(),
            Place::Stmts(semi) => {
                let mut stmts = ast.children(root).to_vec();
                if semi && let Some(&last) = stmts.last() {
                    let k = ast.kind(last);
                    let is_stmt = matches!(
                        k,
                        Kind::LetStmt | Kind::ExprStmt | Kind::EmptyStmt | Kind::MacroCall | Kind::MacroRules
                    ) || is_item(k);
                    if !is_stmt {
                        let n = *ast.node(last);
                        let wrapped = ast.push(Kind::ExprStmt, n.first as usize, n.end as usize, &[last]);
                        *stmts.last_mut().unwrap() = wrapped;
                    }
                }
                stmts
            }
            Place::Expr => {
                let k = ast.kind(root);
                if mbe::is_operand(k) {
                    vec![root]
                } else {
                    // Keep the expansion one expression wherever it is.
                    let n = *ast.node(root);
                    vec![ast.push(Kind::Paren, n.first as usize, n.end as usize, &[root])]
                }
            }
            Place::Type | Place::Pattern => vec![root],
        }
    }

    /// Finds the macro a call names.
    fn resolve(&self, segs: &[&'s str], rooted: bool, pos: &[u32], module: &[String]) -> Option<usize> {
        let m = self.macros;
        let name = *segs.last()?;
        if segs.len() == 1 && !rooted {
            // Text order: the last definition before the call in this
            // module or an enclosing one.
            if let Some(v) = m.by_name[self.krate].get(name) {
                let found = v.iter().rev().copied().find(|&d| {
                    let def = &m.defs[d];
                    !def.item && def.pos.as_slice() < pos && module.starts_with(&def.scope)
                });
                if found.is_some() {
                    return found;
                }
            }
            for &k in &m.prelude[self.krate] {
                if let Some(&d) = m.exported[k].get(name) {
                    return Some(d);
                }
            }
        }
        // By path and imports, as name resolution sees the module tree; a
        // call written in a macro 2.0's body, where that macro is.
        let call_site = *self.mods.last()?;
        let def_site = self.resolver.def_site_module(segs[0]);
        for here in def_site.into_iter().chain([call_site]) {
            if let Some(d) = self.resolver.resolve_macro(self.krate as u32, here, segs, rooted) {
                let def = &self.resolver.defs[d.0 as usize];
                if let Some(&i) = m.by_node.get(&(def.krate as usize, def.file, def.node)) {
                    return Some(i);
                }
            }
        }
        // Proc macros of crates outside the session (`hello::echo!`, or
        // `echo!` after `use hello::echo`) are found by name.
        let proc_in = |k: usize| -> Option<usize> {
            m.by_name[k].get(name)?.iter().rev().copied().find(|&d| matches!(m.defs[d].kind, MacroKind::Proc(..)))
        };
        // Anything else is left for the next round, whose module tree has
        // the imports this round's expansions made.
        let found = proc_in(self.krate);
        if found.is_some() {
            self.found_by_name.borrow_mut().push(name);
        }
        found
    }
}

/// Whether a doc comment is inner (`//!`, `/*!`), and its text: what
/// follows `///` or `//!`, or what is between `/**` and `*/`.
fn doc_text(c: &str) -> (bool, &str) {
    let inner = c.starts_with("//!") || c.starts_with("/*!");
    let body = if c.starts_with("//") { &c[3..] } else { &c[3..c.len() - 2] };
    (inner, body)
}

fn is_item(k: Kind) -> bool {
    use Kind::*;
    matches!(
        k,
        Fn | Struct
            | Union
            | Enum
            | Trait
            | TraitAlias
            | Impl
            | Mod
            | Use
            | Const
            | Static
            | TypeAlias
            | ExternCrate
            | ExternBlock
            | MacroDef
    )
}

/// Lexes text made during expansion, keeping it in the session.
pub(crate) fn lex_text<'s>(session: &'s Session, name: &str, text: String) -> Result<Vec<Token<'s>>, Error<'s>> {
    let text = session.add_text(name, text);
    lex(text)
}
