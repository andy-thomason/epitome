//! Conditional compilation: the target's configuration, `cfg` predicates,
//! and stripping a tree of what is configured away.
//!
//! A predicate is a name (`unix`), a name and a string (`target_os =
//! "linux"`), `all(...)`, `any(...)`, `not(...)`, or `true` / `false`.
//! `#[cfg(p)]` on a node removes it when `p` is false; `#[cfg_attr(p, a,
//! b)]` becomes `#[a] #[b]` when `p` is true and nothing otherwise. Both
//! are done before anything else looks at a file, so the module loader
//! only follows `mod` items that are configured in, with any `#[path]`
//! from a `cfg_attr`.
//!
//! Stripping works on one file and touches nothing else, so files are
//! stripped in parallel, each as soon as it is parsed.

use crate::ast::{Ast, Kind, NodeId};
use crate::lexer::{Edition, Token};
use crate::parse::{Fragment, parse_fragment};
use crate::source::Error;
use std::collections::BTreeSet;

/// The configuration names and name-value pairs that are set.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CfgSet {
    names: BTreeSet<String>,
    pairs: BTreeSet<(String, String)>,
}

/// Features that `cfg!(target_feature = ...)` can report for the targets
/// we run on. The host's set is those this program was built with, which
/// are the target's defaults, as for rustc.
macro_rules! host_features {
    ($set:ident: $($f:literal)*) => {
        $( if cfg!(target_feature = $f) { $set.set_value("target_feature", $f); } )*
    };
}

impl CfgSet {
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets a name: `unix`, `test`, `debug_assertions`.
    pub fn set(&mut self, name: &str) -> &mut Self {
        self.names.insert(name.to_string());
        self
    }

    /// Sets a name-value pair: `target_os = "linux"`, `feature = "std"`.
    pub fn set_value(&mut self, name: &str, value: &str) -> &mut Self {
        self.pairs.insert((name.to_string(), value.to_string()));
        self
    }

    pub fn is_set(&self, name: &str) -> bool {
        self.names.contains(name)
    }

    pub fn has_value(&self, name: &str, value: &str) -> bool {
        self.pairs.contains(&(name.to_string(), value.to_string()))
    }

    /// Reads `rustc --print cfg` output: one `name` or `name="value"` per
    /// line.
    pub fn from_print_cfg(text: &str) -> Self {
        let mut set = CfgSet::new();
        for line in text.lines().map(str::trim).filter(|l| !l.is_empty()) {
            match line.split_once('=') {
                Some((n, v)) => set.set_value(n.trim(), v.trim().trim_matches('"')),
                None => set.set(line),
            };
        }
        set
    }

    /// The configuration of the machine this program runs on, as rustc
    /// sets it for a build without optimisation (`debug_assertions`,
    /// `overflow_checks` and `ub_checks` on), including what it prints only
    /// with `-Zunstable-options`.
    pub fn host() -> Self {
        let mut s = CfgSet::new();
        s.set("debug_assertions");
        s.set_value("panic", "unwind");
        s.set_value("target_abi", "");
        s.set_value("target_arch", std::env::consts::ARCH);
        s.set_value("target_os", std::env::consts::OS);
        let family = std::env::consts::FAMILY;
        if !family.is_empty() {
            s.set_value("target_family", family);
            s.set(family);
        }
        s.set_value("target_endian", if cfg!(target_endian = "little") { "little" } else { "big" });
        s.set_value("target_pointer_width", &(usize::BITS).to_string());
        let env = if cfg!(target_env = "gnu") {
            "gnu"
        } else if cfg!(target_env = "musl") {
            "musl"
        } else if cfg!(target_env = "msvc") {
            "msvc"
        } else {
            ""
        };
        s.set_value("target_env", env);
        let vendor = if cfg!(target_vendor = "apple") {
            "apple"
        } else if cfg!(target_vendor = "pc") {
            "pc"
        } else {
            "unknown"
        };
        s.set_value("target_vendor", vendor);
        let atomics = [
            ("8", cfg!(target_has_atomic = "8")),
            ("16", cfg!(target_has_atomic = "16")),
            ("32", cfg!(target_has_atomic = "32")),
            ("64", cfg!(target_has_atomic = "64")),
            ("128", cfg!(target_has_atomic = "128")),
            ("ptr", cfg!(target_has_atomic = "ptr")),
        ];
        for (width, has) in atomics {
            if has {
                s.set_value("target_has_atomic", width);
                s.set_value("target_has_atomic_primitive_alignment", width);
                s.set_value("target_has_atomic_load_store", width);
            }
        }
        // Configuration rustc sets but prints only with
        // `-Zunstable-options`; `core` and `std` depend on it.
        s.set("target_has_atomic");
        s.set("target_has_atomic_load_store");
        s.set("overflow_checks");
        s.set("ub_checks");
        s.set("target_thread_local");
        s.set("target_has_threads");
        s.set_value("fmt_debug", "full");
        s.set_value("relocation_model", "pic");
        let object = match std::env::consts::OS {
            "macos" | "ios" | "tvos" | "watchos" | "visionos" => "mach-o",
            "windows" => "coff",
            "aix" => "xcoff",
            _ => "elf",
        };
        s.set_value("target_object_format", object);
        if matches!(std::env::consts::ARCH, "x86_64" | "aarch64") {
            s.set("target_has_reliable_f16");
            s.set("target_has_reliable_f16_math");
            s.set("target_has_reliable_f128");
        }
        // Features rustc enables by default but `cfg!` cannot ask about
        // on stable.
        match std::env::consts::ARCH {
            "x86_64" | "x86" => {
                s.set_value("target_feature", "x87");
            }
            "aarch64" if cfg!(target_vendor = "apple") => {
                for f in ["flagm2", "lse2", "v8.1a", "v8.2a", "v8.3a", "v8.4a"] {
                    s.set_value("target_feature", f);
                }
            }
            _ => {}
        }
        host_features!(s:
            "fxsr" "sse" "sse2" "sse3" "ssse3" "sse4.1" "sse4.2" "popcnt" "avx" "avx2" "fma" "bmi1" "bmi2"
            "lzcnt" "movbe" "f16c" "avx512f" "cmpxchg16b" "xsave"
            "aes" "crc" "dit" "dotprod" "dpb" "dpb2" "fcma" "fhm" "flagm" "fp16" "frintts" "jsconv" "lor"
            "lse" "neon" "paca" "pacg" "pan" "pmuv3" "ras" "rcpc" "rcpc2" "rdm" "sb" "sha2" "sha3" "ssbs" "vh"
        );
        s
    }

    /// The configuration of `x86_64-unknown-linux-gnu` with its default
    /// features, as rustc sets it for a build without optimisation: for
    /// compiling `core` for that target on another machine.
    pub fn x86_64_linux() -> Self {
        let mut s = CfgSet::new();
        for name in [
            "debug_assertions",
            "overflow_checks",
            "ub_checks",
            "unix",
            "target_has_atomic",
            "target_has_atomic_load_store",
            "target_has_reliable_f16",
            "target_has_reliable_f16_math",
            "target_has_reliable_f128",
            "target_has_threads",
            "target_thread_local",
        ] {
            s.set(name);
        }
        for (name, value) in [
            ("fmt_debug", "full"),
            ("panic", "unwind"),
            ("relocation_model", "pic"),
            ("target_abi", ""),
            ("target_arch", "x86_64"),
            ("target_endian", "little"),
            ("target_env", "gnu"),
            ("target_family", "unix"),
            ("target_object_format", "elf"),
            ("target_os", "linux"),
            ("target_pointer_width", "64"),
            ("target_vendor", "unknown"),
        ] {
            s.set_value(name, value);
        }
        for f in ["fxsr", "sse", "sse2", "x87"] {
            s.set_value("target_feature", f);
        }
        for width in ["8", "16", "32", "64", "ptr"] {
            s.set_value("target_has_atomic", width);
            s.set_value("target_has_atomic_load_store", width);
            s.set_value("target_has_atomic_primitive_alignment", width);
        }
        s
    }

    /// Evaluates a predicate written as `tokens` (without the surrounding
    /// parentheses of `cfg(...)`).
    pub fn eval<'a>(&self, tokens: &[Token<'a>], eof: Token<'a>) -> Result<bool, Error<'a>> {
        let mut p = Pred { toks: tokens, pos: 0, eof };
        let v = p.pred(self)?;
        if p.pos != tokens.len() {
            return Err(Error::new(p.tok().0, "expected the end of the `cfg` predicate"));
        }
        Ok(v)
    }
}

/// A predicate being read.
struct Pred<'t, 'a> {
    toks: &'t [Token<'a>],
    pos: usize,
    eof: Token<'a>,
}

impl<'t, 'a> Pred<'t, 'a> {
    fn tok(&self) -> Token<'a> {
        self.toks.get(self.pos).copied().unwrap_or(self.eof)
    }

    fn eat(&mut self, s: &str) -> bool {
        if self.tok().0 == s {
            self.pos += 1;
            true
        } else {
            false
        }
    }

    fn pred(&mut self, set: &CfgSet) -> Result<bool, Error<'a>> {
        let t = self.tok();
        if !t.is_ident() {
            return Err(Error::new(t.0, "expected a `cfg` predicate"));
        }
        self.pos += 1;
        let name = t.ident_name();
        if self.eat("(") {
            let mut values = Vec::new();
            while !self.eat(")") {
                values.push(self.pred(set)?);
                if !self.eat(",") {
                    if self.eat(")") {
                        break;
                    }
                    return Err(Error::new(self.tok().0, "expected `,` or `)`"));
                }
            }
            return match name {
                "all" => Ok(values.iter().all(|&v| v)),
                "any" => Ok(values.iter().any(|&v| v)),
                "not" if values.len() == 1 => Ok(!values[0]),
                "not" => Err(Error::new(t.0, "`not` takes one predicate")),
                _ => Err(Error::new(t.0, format!("unknown `cfg` operator `{name}`"))),
            };
        }
        if self.eat("=") {
            let v = self.tok();
            if !v.is_string() {
                return Err(Error::new(v.0, "expected a string"));
            }
            self.pos += 1;
            return Ok(set.has_value(name, string_value(v.0)));
        }
        Ok(match t.0 {
            "true" => true,
            "false" => false,
            _ => set.is_set(name),
        })
    }
}

/// The contents of a plain or raw string literal, without quotes. Escapes
/// are left as they are: `cfg` values do not use them.
pub fn string_value(s: &str) -> &str {
    let s = s.strip_prefix('r').map_or(s, |r| r.trim_matches('#'));
    s.strip_prefix('"').and_then(|s| s.strip_suffix('"')).unwrap_or(s)
}

/// The tokens between the delimiters of a token tree node.
fn inside<'p, 'a>(ast: &'p Ast<'a>, tt: NodeId) -> &'p [Token<'a>] {
    let t = ast.tokens_of(tt);
    if t.len() >= 2 { &t[1..t.len() - 1] } else { &[] }
}

/// The token-index range of a node.
fn range(ast: &Ast, id: NodeId) -> (usize, usize) {
    let n = ast.node(id);
    (n.first as usize, n.end as usize)
}

/// Removes from `ast` everything `cfg` configures away, and replaces each
/// `cfg_attr` with the attributes it gives. Returns the errors found (a
/// malformed predicate is an error, and the node it is on is kept).
pub fn strip<'a>(ast: &mut Ast<'a>, cfg: &CfgSet, edition: Edition, eof: Token<'a>) -> Vec<Error<'a>> {
    let mut s = Stripper { cfg, edition, eof, errors: Vec::new() };
    let root = ast.root;
    s.expand_cfg_attrs(ast, root);
    if !s.configured_in(ast, root) {
        // `#![cfg(...)]` false on the file: the module is empty.
        ast.set_children(root, &[]);
    } else {
        s.visit(ast, root);
    }
    s.errors
}

/// Strips nodes made by macro expansion before they go into the tree:
/// returns those of `nodes` that are configured in (each stripped inside),
/// and any errors.
pub fn strip_new<'a>(
    ast: &mut Ast<'a>,
    nodes: &[NodeId],
    cfg: &CfgSet,
    edition: Edition,
    eof: Token<'a>,
) -> (Vec<NodeId>, Vec<Error<'a>>) {
    let mut s = Stripper { cfg, edition, eof, errors: Vec::new() };
    let mut kept = Vec::with_capacity(nodes.len());
    for &n in nodes {
        if ast.kind(n).is_leaf() {
            kept.push(n);
            continue;
        }
        s.expand_cfg_attrs(ast, n);
        if s.configured_in(ast, n) {
            s.visit(ast, n);
            kept.push(n);
        }
    }
    (kept, s.errors)
}

struct Stripper<'c, 'a> {
    cfg: &'c CfgSet,
    edition: Edition,
    eof: Token<'a>,
    errors: Vec<Error<'a>>,
}

fn is_attr(k: Kind) -> bool {
    matches!(k, Kind::Attribute | Kind::InnerAttribute)
}

impl<'c, 'a> Stripper<'c, 'a> {
    /// The attribute's path, if it is a single name.
    fn attr_name<'p>(&self, ast: &'p Ast<'a>, attr: NodeId) -> Option<&'a str> {
        let path = ast.children(attr).iter().copied().find(|&c| ast.kind(c) == Kind::Path)?;
        let segs = ast.children(path);
        if segs.len() != 1 {
            return None;
        }
        Some(ast.token(segs[0]).ident_name())
    }

    /// The token tree of `#[name(...)]`.
    fn attr_args(&self, ast: &Ast<'a>, attr: NodeId) -> Option<NodeId> {
        ast.children(attr).iter().copied().find(|&c| ast.kind(c) == Kind::TokenTree)
    }

    /// Evaluates the predicate in a token tree, or records an error.
    fn eval(&mut self, ast: &Ast<'a>, tt: Option<NodeId>, at: NodeId) -> Option<bool> {
        let Some(tt) = tt else {
            self.errors.push(Error::new(ast.token(at).0, "expected `cfg(predicate)`"));
            return None;
        };
        match self.cfg.eval(inside(ast, tt), self.eof) {
            Ok(v) => Some(v),
            Err(e) => {
                self.errors.push(e);
                None
            }
        }
    }

    /// Replaces each `cfg_attr` among `id`'s children with what it gives.
    fn expand_cfg_attrs(&mut self, ast: &mut Ast<'a>, id: NodeId) {
        let kids = ast.children(id);
        if !kids.iter().any(|&k| is_attr(ast.kind(k)) && self.attr_name(ast, k) == Some("cfg_attr")) {
            return;
        }
        let kids = kids.to_vec();
        let mut out = Vec::with_capacity(kids.len());
        for k in kids {
            self.expand_one(ast, k, &mut out);
        }
        ast.set_children(id, &out);
    }

    /// Pushes `attr` to `out`, or what it expands to if it is `cfg_attr`.
    fn expand_one(&mut self, ast: &mut Ast<'a>, attr: NodeId, out: &mut Vec<NodeId>) {
        if !is_attr(ast.kind(attr)) || self.attr_name(ast, attr) != Some("cfg_attr") {
            out.push(attr);
            return;
        }
        let Some(tt) = self.attr_args(ast, attr) else {
            self.errors.push(Error::new(ast.token(attr).0, "expected `cfg_attr(predicate, attributes)`"));
            return;
        };
        // Split the contents at top-level commas: the predicate, then one
        // attribute per piece.
        let (first, end) = range(ast, tt);
        let (first, end) = (first + 1, end - 1);
        let mut pieces = Vec::new();
        let mut depth = 0i32;
        let mut start = first;
        for i in first..end {
            match ast.tokens[i].0 {
                "(" | "[" | "{" => depth += 1,
                ")" | "]" | "}" => depth -= 1,
                "," if depth == 0 => {
                    pieces.push((start, i));
                    start = i + 1;
                }
                _ => {}
            }
        }
        if start < end {
            pieces.push((start, end));
        }
        let Some(&(ps, pe)) = pieces.first() else {
            self.errors.push(Error::new(ast.token(tt).0, "expected a predicate"));
            return;
        };
        let value = match self.cfg.eval(&ast.tokens[ps..pe], self.eof) {
            Ok(v) => v,
            Err(e) => {
                self.errors.push(e);
                return;
            }
        };
        if !value {
            return;
        }
        let inner = ast.kind(attr) == Kind::InnerAttribute;
        for &(s, e) in &pieces[1..] {
            let tokens = ast.tokens[s..e].to_vec();
            let eof = Token(tokens.last().map_or("", |t| &t.0[t.0.len()..]));
            match parse_fragment(tokens, eof, self.edition, Fragment::Meta) {
                Ok(frag) => {
                    let id = ast.graft(&frag, s);
                    if inner {
                        ast.set_kind(id, Kind::InnerAttribute);
                    }
                    // `cfg_attr(a, cfg_attr(b, x))` nests.
                    self.expand_one(ast, id, out);
                }
                Err(e) => self.errors.push(e),
            }
        }
    }

    /// Whether `id`'s own `cfg` attributes (outer, or inner among its
    /// children) are all true. A malformed one counts as true.
    fn configured_in(&mut self, ast: &Ast<'a>, id: NodeId) -> bool {
        let kids = ast.children(id);
        for &k in kids {
            if is_attr(ast.kind(k)) && self.attr_name(ast, k) == Some("cfg") {
                let tt = self.attr_args(ast, k);
                if self.eval(ast, tt, k) == Some(false) {
                    return false;
                }
            }
        }
        true
    }

    /// Strips `id`'s children, then theirs.
    fn visit(&mut self, ast: &mut Ast<'a>, id: NodeId) {
        let kids = ast.children(id).to_vec();
        let mut kept = Vec::with_capacity(kids.len());
        for k in kids {
            if ast.kind(k) == Kind::TokenTree {
                kept.push(k);
                continue;
            }
            self.expand_cfg_attrs(ast, k);
            if self.configured_in(ast, k) {
                kept.push(k);
            }
        }
        if kept.len() != ast.children(id).len() {
            ast.set_children(id, &kept);
        }
        for k in kept {
            if !ast.kind(k).is_leaf() {
                self.visit(ast, k);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::lex;
    use crate::parse::parse_str;
    use crate::pretty::pretty;

    fn eval(set: &CfgSet, src: &str) -> bool {
        let toks = lex(src).unwrap();
        set.eval(&toks, Token(&src[src.len()..])).unwrap()
    }

    #[test]
    fn predicates() {
        let mut s = CfgSet::new();
        s.set("unix").set_value("target_os", "linux").set_value("feature", "std");
        assert!(eval(&s, "unix"));
        assert!(!eval(&s, "windows"));
        assert!(eval(&s, r#"target_os = "linux""#));
        assert!(!eval(&s, r#"target_os = "macos""#));
        assert!(eval(&s, r#"all(unix, feature = "std")"#));
        assert!(!eval(&s, r#"all(unix, feature = "alloc")"#));
        assert!(eval(&s, "any(windows, unix,)"));
        assert!(eval(&s, "not(windows)"));
        assert!(eval(&s, "all()") && !eval(&s, "any()"));
        assert!(eval(&s, "true") && !eval(&s, "false"));
        let toks = lex("not(a, b)").unwrap();
        assert!(s.eval(&toks, Token("")).is_err());
    }

    fn stripped(src: &str, set: &CfgSet) -> String {
        let mut ast = parse_str(src, Edition::E2024).unwrap();
        let errors = strip(&mut ast, set, Edition::E2024, Token(&src[src.len()..]));
        assert!(errors.is_empty(), "{:?}", errors.iter().map(|e| &e.message).collect::<Vec<_>>());
        pretty(&ast)
    }

    #[test]
    fn strips_items_fields_statements_and_arms() {
        let mut s = CfgSet::new();
        s.set("unix");
        let out = stripped(
            "#[cfg(windows)] fn w() {}\n#[cfg(unix)] fn u() {}\n\
             struct S { #[cfg(windows)] a: u8, b: u8 }\n\
             enum E { #[cfg(not(unix))] A, B }\n\
             fn f() { #[cfg(windows)] let x = 1; match y { #[cfg(windows)] 1 => {} _ => {} } g(1, #[cfg(windows)] 2); }",
            &s,
        );
        assert_eq!(
            out,
            "#[cfg(unix)]\nfn u() {}\n\nstruct S {\n    b: u8,\n}\n\nenum E {\n    B,\n}\n\n\
             fn f() {\n    match y {\n        _ => {},\n    }\n    g(1);\n}\n"
        );
    }

    #[test]
    fn cfg_attr_gives_attributes() {
        let mut s = CfgSet::new();
        s.set("unix");
        let out = stripped(
            "#[cfg_attr(unix, path = \"u.rs\", allow(x))] mod a;\n\
             #[cfg_attr(windows, path = \"w.rs\")] mod b;\n\
             #[cfg_attr(unix, cfg_attr(unix, inline))] fn f() {}\n\
             #[cfg_attr(unix, cfg(windows))] fn gone() {}",
            &s,
        );
        assert_eq!(out, "#[path = \"u.rs\"]\n#[allow(x)]\nmod a;\n\nmod b;\n\n#[inline]\nfn f() {}\n");
    }

    #[test]
    fn inner_cfg_empties_a_file_or_module() {
        let s = CfgSet::new();
        assert_eq!(stripped("#![cfg(unix)]\nfn f() {}", &s), "\n");
        assert_eq!(stripped("mod m { #![cfg(unix)] fn f() {} }\nfn g() {}", &s), "fn g() {}\n");
    }

    #[test]
    fn host_matches_rustc() {
        // Only where rustc is installed: its `--print cfg`, with the
        // unstable options it hides, is the reference for the host's
        // configuration.
        let Ok(out) = std::process::Command::new("rustc")
            .args(["--print", "cfg", "-Zunstable-options"])
            .env("RUSTC_BOOTSTRAP", "1")
            .output()
        else {
            return;
        };
        let theirs = CfgSet::from_print_cfg(&String::from_utf8_lossy(&out.stdout));
        let ours = CfgSet::host();
        let missing: Vec<_> = theirs.pairs.difference(&ours.pairs).collect();
        let extra: Vec<_> = ours.pairs.difference(&theirs.pairs).collect();
        assert!(missing.is_empty() && extra.is_empty(), "missing {missing:?}, extra {extra:?}");
        assert_eq!(theirs.names, ours.names);
    }

    #[test]
    fn x86_64_linux_matches_rustc() {
        // rustc prints any target's configuration, whatever it runs on.
        let Ok(out) = std::process::Command::new("rustc")
            .args(["--print", "cfg", "--target", "x86_64-unknown-linux-gnu", "-Zunstable-options"])
            .env("RUSTC_BOOTSTRAP", "1")
            .output()
        else {
            return;
        };
        assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
        let theirs = CfgSet::from_print_cfg(&String::from_utf8_lossy(&out.stdout));
        let ours = CfgSet::x86_64_linux();
        let missing: Vec<_> = theirs.pairs.difference(&ours.pairs).collect();
        let extra: Vec<_> = ours.pairs.difference(&theirs.pairs).collect();
        assert!(missing.is_empty() && extra.is_empty(), "missing {missing:?}, extra {extra:?}");
        assert_eq!(theirs.names, ours.names);
    }
}
