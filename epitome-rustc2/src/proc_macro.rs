//! Running procedural macros: libraries built by rustc, loaded and run by
//! the toolchain's own server, `libexec/rust-analyzer-proc-macro-srv`.
//!
//! The server reads one JSON request per line on stdin and answers one per
//! line on stdout. What is here was learnt by running it (its error
//! messages name the fields it wants), for API version 6:
//!
//! - `{"ApiVersionCheck":{}}` → `{"ApiVersionCheck":6}`.
//! - `{"ListMacros":{"dylib_path":P}}` → `{"ListMacros":{"Ok":[[name,
//!   kind], ...]}}`, kind `CustomDerive`, `FuncLike` or `Attr`.
//! - `{"ExpandMacro":{"lib":P, "env":[], "current_dir":D, "macro_body":T,
//!   "macro_name":N, "attributes":T or null}}` → `{"ExpandMacro":{"Ok":T}}`.
//!
//! A token tree `T` is flat: six arrays. `text` holds strings; `ident` is
//! `[span, text, raw]` per identifier; `punct` is `[span, char, joint]`;
//! `literal` is `[span, text, kind, suffix]` (suffix `u32::MAX` for none;
//! kind 0 `true`/`false`, 1 byte, 2 char, 3 integer, 4 float, 5 string,
//! 6 raw string, 7 byte string, 8 raw byte string, 9 C string, 10 raw C
//! string, with a raw string's `#` count in bits 8 and up); `subtree` is
//! `[open span, close span, delimiter, first, end]` per group (delimiter
//! 0 none, 1 `()`, 2 `{}`, 3 `[]`), whose members are `token_tree[first..
//! end]`; a member is `index << 2 | tag`, tag 0 subtree, 1 literal, 2
//! punct, 3 ident. Subtree 0 is the whole input, undelimited.
//!
//! The server's API is unstable and tied to the toolchain that built the
//! libraries, so the server must come from that toolchain.

use crate::expand::mbe_delims;
use crate::json::{Json, nums, obj};
use crate::lexer::Token;
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
use std::sync::Mutex;

/// What kind of procedural macro.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ProcMacroKind {
    Derive,
    FuncLike,
    Attr,
}

/// The server, started when first needed. Requests from all threads go
/// through one process, one at a time.
pub struct Server {
    path: PathBuf,
    process: Mutex<Option<Process>>,
}

struct Process {
    child: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
}

impl Drop for Process {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

impl Server {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Server { path: path.into(), process: Mutex::new(None) }
    }

    /// The server of the `rustc` on `PATH`, if it has one.
    pub fn from_rustc() -> Option<PathBuf> {
        let out = Command::new("rustc").args(["--print", "sysroot"]).output().ok()?;
        let sysroot = String::from_utf8(out.stdout).ok()?;
        let path = Path::new(sysroot.trim()).join("libexec/rust-analyzer-proc-macro-srv");
        path.exists().then_some(path)
    }

    /// Sends a request and reads the answer. A server that has failed is
    /// started again for the next request.
    fn request(&self, req: &Json) -> Result<Json, String> {
        let mut guard = self.process.lock().unwrap();
        if guard.is_none() {
            let mut child = Command::new(&self.path)
                .env("RUST_ANALYZER_INTERNALS_DO_NOT_USE", "1")
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::null())
                .spawn()
                .map_err(|e| format!("cannot start `{}`: {e}", self.path.display()))?;
            let stdin = child.stdin.take().unwrap();
            let stdout = BufReader::new(child.stdout.take().unwrap());
            *guard = Some(Process { child, stdin, stdout });
        }
        let p = guard.as_mut().unwrap();
        let mut line = req.to_text();
        line.push('\n');
        let mut answer = String::new();
        let ok = p.stdin.write_all(line.as_bytes()).and_then(|_| p.stdin.flush()).is_ok()
            && p.stdout.read_line(&mut answer).is_ok_and(|n| n > 0);
        if !ok {
            *guard = None;
            return Err("the proc-macro server stopped (a macro may have crashed it)".to_string());
        }
        Json::parse(&answer)
    }

    /// The procedural macros in a library.
    pub fn list_macros(&self, dylib: &Path) -> Result<Vec<(String, ProcMacroKind)>, String> {
        let req = obj(vec![("ListMacros", obj(vec![("dylib_path", Json::String(dylib.display().to_string()))]))]);
        let answer = self.request(&req)?;
        let result = answer.get("ListMacros").ok_or("bad answer to ListMacros")?;
        if let Some(e) = result.get("Err") {
            return Err(format!("cannot load `{}`: {}", dylib.display(), e.to_text()));
        }
        let list = result.get("Ok").and_then(Json::as_array).ok_or("bad answer to ListMacros")?;
        let mut out = Vec::new();
        for m in list {
            let pair = m.as_array().ok_or("bad macro in ListMacros")?;
            let (Some(name), Some(kind)) = (pair[0].as_str(), pair.get(1).and_then(Json::as_str)) else {
                return Err("bad macro in ListMacros".into());
            };
            let kind = match kind {
                "CustomDerive" => ProcMacroKind::Derive,
                "FuncLike" => ProcMacroKind::FuncLike,
                _ => ProcMacroKind::Attr,
            };
            out.push((name.to_string(), kind));
        }
        Ok(out)
    }

    /// Runs macro `name` of `dylib` on `body` (and, for an attribute macro,
    /// the attribute's arguments), and returns the output as source text.
    pub fn expand(
        &self,
        dylib: &Path,
        name: &str,
        body: &[Token],
        attr: Option<&[Token]>,
        current_dir: &Path,
    ) -> Result<String, String> {
        let mut fields = vec![
            ("lib", Json::String(dylib.display().to_string())),
            ("env", Json::Array(Vec::new())),
            ("current_dir", Json::String(current_dir.display().to_string())),
            ("macro_body", to_flat(body)),
            ("macro_name", Json::String(name.to_string())),
        ];
        if let Some(a) = attr {
            fields.push(("attributes", to_flat(a)));
        }
        let answer = self.request(&obj(vec![("ExpandMacro", obj(fields))]))?;
        let result = answer.get("ExpandMacro").ok_or("bad answer to ExpandMacro")?;
        if let Some(e) = result.get("Err") {
            let msg =
                e.as_str().map(str::to_string).or_else(|| e.get("message").and_then(Json::as_str).map(str::to_string));
            return Err(msg.unwrap_or_else(|| e.to_text()));
        }
        from_flat(result.get("Ok").ok_or("bad answer to ExpandMacro")?)
    }
}

/// A flat token tree being built.
#[derive(Default)]
struct Flat {
    subtree: Vec<u32>,
    literal: Vec<u32>,
    punct: Vec<u32>,
    ident: Vec<u32>,
    token_tree: Vec<u32>,
    text: Vec<String>,
    texts: HashMap<String, u32>,
}

impl Flat {
    fn text(&mut self, s: &str) -> u32 {
        if let Some(&i) = self.texts.get(s) {
            return i;
        }
        let i = self.text.len() as u32;
        self.text.push(s.to_string());
        self.texts.insert(s.to_string(), i);
        i
    }

    fn json(self) -> Json {
        obj(vec![
            ("subtree", nums(&self.subtree)),
            ("literal", nums(&self.literal)),
            ("punct", nums(&self.punct)),
            ("ident", nums(&self.ident)),
            ("token_tree", nums(&self.token_tree)),
            ("text", Json::Array(self.text.into_iter().map(Json::String).collect())),
        ])
    }
}

/// A literal's kind code, contents and suffix, from its text.
fn literal_parts(t: &str) -> (u32, &str, Option<&str>) {
    let b = t.as_bytes();
    // Quoted forms: the prefix, `r` and `#`s, then the quote.
    let (prefix_len, raw) = match b {
        [b'b' | b'c', b'r', ..] => (2, true),
        [b'r', ..] => (1, true),
        [b'b' | b'c', b'"' | b'\'', ..] => (1, false),
        _ => (0, false),
    };
    if matches!(b.get(prefix_len), Some(b'"' | b'\'' | b'#')) || raw {
        let hashes = t[prefix_len..].bytes().take_while(|&c| c == b'#').count();
        let open = prefix_len + hashes;
        let quote = b[open];
        let close = if quote == b'"' {
            let pat: String = std::iter::once('"').chain(std::iter::repeat_n('#', hashes)).collect();
            if raw {
                t[open + 1..].find(&pat).map_or(t.len(), |k| open + 1 + k)
            } else {
                // The closing quote: the last `"` (the suffix has none).
                t.rfind('"').unwrap_or(t.len())
            }
        } else {
            t.rfind('\'').unwrap_or(t.len())
        };
        let content = &t[open + 1..close];
        let after = (close + 1 + if raw { hashes } else { 0 }).min(t.len());
        let suffix = (after < t.len()).then(|| &t[after..]);
        let kind = match (&t[..prefix_len], quote, raw) {
            ("", b'\'', _) => 2,
            ("b", b'\'', _) => 1,
            ("", _, false) => 5,
            ("r", _, true) => 6 | (hashes as u32) << 8,
            ("b", _, false) => 7,
            ("br", _, true) => 8 | (hashes as u32) << 8,
            ("c", _, false) => 9,
            (_, _, _) => 10 | (hashes as u32) << 8,
        };
        return (kind, content, suffix);
    }
    // A number: the digits, then any suffix.
    let radix_prefix = t.len() > 1 && b[0] == b'0' && matches!(b[1], b'x' | b'o' | b'b');
    let end = if radix_prefix {
        const SUFFIXES: [&str; 12] =
            ["i8", "i16", "i32", "i64", "i128", "isize", "u8", "u16", "u32", "u64", "u128", "usize"];
        SUFFIXES.iter().find(|s| t.ends_with(*s)).map_or(t.len(), |s| t.len() - s.len())
    } else {
        let mut i = 0;
        while i < b.len() && (b[i].is_ascii_digit() || b[i] == b'_') {
            i += 1;
        }
        if i < b.len() && b[i] == b'.' {
            i += 1;
            while i < b.len() && (b[i].is_ascii_digit() || b[i] == b'_') {
                i += 1;
            }
        }
        if i < b.len() && matches!(b[i], b'e' | b'E') {
            let mut j = i + 1;
            if j < b.len() && matches!(b[j], b'+' | b'-') {
                j += 1;
            }
            if j < b.len() && b[j].is_ascii_digit() {
                i = j;
                while i < b.len() && (b[i].is_ascii_digit() || b[i] == b'_') {
                    i += 1;
                }
            }
        }
        i
    };
    let digits = &t[..end];
    let suffix = (end < t.len()).then(|| &t[end..]);
    let float = !radix_prefix && (digits.contains('.') || digits.contains(['e', 'E']))
        || matches!(suffix, Some("f32" | "f64" | "f16" | "f128"));
    (if float { 4 } else { 3 }, digits, suffix)
}

/// A literal's text from its kind code, contents and suffix.
fn literal_text(kind: u32, content: &str, suffix: &str) -> String {
    let hashes = "#".repeat((kind >> 8) as usize);
    let body = match kind & 0xff {
        0 | 3 | 4 => content.to_string(),
        1 => format!("b'{content}'"),
        2 => format!("'{content}'"),
        5 => format!("\"{content}\""),
        6 => format!("r{hashes}\"{content}\"{hashes}"),
        7 => format!("b\"{content}\""),
        8 => format!("br{hashes}\"{content}\"{hashes}"),
        9 => format!("c\"{content}\""),
        _ => format!("cr{hashes}\"{content}\"{hashes}"),
    };
    body + suffix
}

/// Our tokens as a flat token tree. Spans are token indices (the whole
/// input's span is the token count).
fn to_flat(tokens: &[Token]) -> Json {
    let close = mbe_delims(tokens);
    let mut f = Flat::default();
    // Groups are numbered and laid out breadth first: each group's members
    // follow those of the groups numbered before it.
    let mut queue: Vec<(usize, usize, u32, u32)> = vec![(0, tokens.len(), 0, tokens.len() as u32)];
    f.subtree.extend([tokens.len() as u32, tokens.len() as u32, 0, 0, 0]);
    let mut q = 0;
    while q < queue.len() {
        let (start, end, delim, open_span) = queue[q];
        let _ = (delim, open_span);
        let first = f.token_tree.len() as u32;
        let mut i = start;
        while i < end {
            let t = tokens[i];
            let span = i as u32;
            let c = t.0.as_bytes()[0];
            if matches!(c, b'(' | b'[' | b'{') && close[i] != usize::MAX {
                let index = (f.subtree.len() / 5) as u32;
                let kind = match c {
                    b'(' => 1,
                    b'{' => 2,
                    _ => 3,
                };
                f.subtree.extend([span, close[i] as u32, kind, 0, 0]);
                f.token_tree.push(index << 2);
                queue.push((i + 1, close[i], kind, span));
                i = close[i] + 1;
                continue;
            }
            if t.is_lifetime() {
                let p = (f.punct.len() / 3) as u32;
                f.punct.extend([span, '\'' as u32, 1]);
                f.token_tree.push(p << 2 | 2);
                let name = &t.0[1..];
                let (raw, name) = match name.strip_prefix("r#") {
                    Some(n) => (1, n),
                    None => (0, name),
                };
                let x = (f.ident.len() / 3) as u32;
                let text = f.text(name);
                f.ident.extend([span, text, raw]);
                f.token_tree.push(x << 2 | 3);
            } else if t.is_literal() {
                let (kind, content, suffix) = literal_parts(t.0);
                let l = (f.literal.len() / 4) as u32;
                let text = f.text(content);
                let suffix = suffix.map_or(u32::MAX, |s| f.text(s));
                f.literal.extend([span, text, kind, suffix]);
                f.token_tree.push(l << 2 | 1);
            } else if t.is_ident() {
                let (raw, name) = match t.0.strip_prefix("r#") {
                    Some(n) => (1, n),
                    None => (0, t.0),
                };
                let x = (f.ident.len() / 3) as u32;
                let text = f.text(name);
                f.ident.extend([span, text, raw]);
                f.token_tree.push(x << 2 | 3);
            } else {
                let joint = tokens.get(i + 1).is_some_and(|n| i + 1 < end && n.is_punct() && t.touches(*n));
                let p = (f.punct.len() / 3) as u32;
                f.punct.extend([span, t.0.chars().next().unwrap() as u32, joint as u32]);
                f.token_tree.push(p << 2 | 2);
            }
            i += 1;
        }
        let last = f.token_tree.len() as u32;
        let at = q * 5;
        f.subtree[at + 3] = first;
        f.subtree[at + 4] = last;
        q += 1;
    }
    f.json()
}

/// A flat token tree as source text: tokens separated by spaces, except
/// after a joint punctuation character.
fn from_flat(tree: &Json) -> Result<String, String> {
    let get = |k: &str| tree.get(k).and_then(Json::as_u32s).ok_or_else(|| format!("bad token tree: no `{k}`"));
    let subtree = get("subtree")?;
    let literal = get("literal")?;
    let punct = get("punct")?;
    let ident = get("ident")?;
    let token_tree = get("token_tree")?;
    let text: Vec<&str> = tree
        .get("text")
        .and_then(Json::as_array)
        .ok_or("bad token tree: no `text`")?
        .iter()
        .map(|t| t.as_str().unwrap_or(""))
        .collect();
    let txt = |i: u32| text.get(i as usize).copied().unwrap_or("");
    let mut out = String::new();
    #[allow(clippy::type_complexity)]
    fn group(
        s: usize,
        out: &mut String,
        subtree: &[u32],
        token_tree: &[u32],
        f: &dyn Fn(u32, &mut String, &dyn Fn(usize, &mut String)),
    ) {
        let (kind, first, end) = (subtree[s * 5 + 2], subtree[s * 5 + 3], subtree[s * 5 + 4]);
        let (open, close) = match kind {
            1 => ("(", ")"),
            2 => ("{", "}"),
            3 => ("[", "]"),
            _ => ("", ""),
        };
        out.push_str(open);
        for &m in &token_tree[first as usize..end as usize] {
            f(m, out, &|sub, out| group(sub, out, subtree, token_tree, f));
        }
        out.push_str(close);
        out.push(' ');
    }
    let member = |m: u32, out: &mut String, sub: &dyn Fn(usize, &mut String)| {
        let i = (m >> 2) as usize;
        match m & 3 {
            0 => sub(i, out),
            1 => {
                let (content, kind, suffix) = (literal[i * 4 + 1], literal[i * 4 + 2], literal[i * 4 + 3]);
                let suffix = if suffix == u32::MAX { "" } else { txt(suffix) };
                out.push_str(&literal_text(kind, txt(content), suffix));
                out.push(' ');
            }
            2 => {
                out.push(char::from_u32(punct[i * 3 + 1]).unwrap_or('?'));
                if punct[i * 3 + 2] == 0 {
                    out.push(' ');
                }
            }
            _ => {
                if ident[i * 3 + 2] != 0 {
                    out.push_str("r#");
                }
                out.push_str(txt(ident[i * 3 + 1]));
                out.push(' ');
            }
        }
    };
    if subtree.len() < 5 {
        return Err("bad token tree: no root".into());
    }
    group(0, &mut out, &subtree, &token_tree, &member);
    Ok(out.trim_end().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::lex;

    #[test]
    fn literals() {
        assert_eq!(literal_parts("1u8"), (3, "1", Some("u8")));
        assert_eq!(literal_parts("1.5e3f32"), (4, "1.5e3", Some("f32")));
        assert_eq!(literal_parts("0xffu64"), (3, "0xff", Some("u64")));
        assert_eq!(literal_parts("0x1e5"), (3, "0x1e5", None));
        assert_eq!(literal_parts("'a'"), (2, "a", None));
        assert_eq!(literal_parts("b'\\n'"), (1, "\\n", None));
        assert_eq!(literal_parts("\"a\\\"b\"suf"), (5, "a\\\"b", Some("suf")));
        assert_eq!(literal_parts("r##\"a\"#b\"##"), (6 | 2 << 8, "a\"#b", None));
        assert_eq!(literal_parts("br\"x\""), (8, "x", None));
        assert_eq!(literal_parts("c\"x\""), (9, "x", None));
        for t in ["1u8", "1.5e3f32", "0xffu64", "'a'", "b'\\n'", "\"a\\\"b\"suf", "r##\"a\"#b\"##", "cr#\"x\"#"] {
            let (k, c, s) = literal_parts(t);
            assert_eq!(literal_text(k, c, s.unwrap_or("")), t);
        }
    }

    #[test]
    fn flat_round_trip() {
        let src = "impl X for Y { fn f(&'a self) -> [u8; 2] { a => b'x' + 1.0 } } r#match";
        let toks = lex(src).unwrap();
        let flat = to_flat(&toks);
        let back = from_flat(&flat).unwrap();
        let again: Vec<&str> = lex(&back).unwrap().into_iter().map(|t| t.0).collect();
        let orig: Vec<&str> = toks.iter().map(|t| t.0).collect();
        assert_eq!(again, orig);
        assert!(back.contains("=>"), "{back}");
    }
}
