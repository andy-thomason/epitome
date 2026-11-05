//! The lexer for Rust source text.
//!
//! A token is just a slice of the source, as in Epitome's other lexers. Its
//! kind is worked out from its first characters when needed, and its
//! position from where the slice sits in the source (see `source`).
//!
//! Punctuation is one character per token, as `proc_macro` sees it: `::`
//! is two `:` tokens that touch. The parser joins touching tokens into the
//! operator it wants (`at_op`), so `Vec<Vec<u8>>` and `&&x` need no
//! splitting, and whether a token is joined to the next is known from the
//! two slices' addresses.
//!
//! Comments and white space are skipped, apart from doc comments, which
//! are tokens because they are attributes.

use crate::source::Error;

/// A token: a slice of the source. The end-of-file token is the empty slice
/// at the end of the text.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Token<'a>(pub &'a str);

/// The edition, which decides some keywords.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Default)]
pub enum Edition {
    E2015,
    E2018,
    E2021,
    #[default]
    E2024,
}

impl Edition {
    pub fn parse(s: &str) -> Option<Edition> {
        Some(match s {
            "2015" => Edition::E2015,
            "2018" => Edition::E2018,
            "2021" => Edition::E2021,
            "2024" => Edition::E2024,
            _ => return None,
        })
    }
}

/// Keywords reserved in every edition: strict keywords, and reserved words
/// with no meaning yet.
const KEYWORDS: &[&str] = &[
    "as", "break", "const", "continue", "crate", "else", "enum", "extern", "false", "fn", "for", "if", "impl", "in",
    "let", "loop", "match", "mod", "move", "mut", "pub", "ref", "return", "self", "Self", "static", "struct", "super",
    "trait", "true", "type", "unsafe", "use", "where", "while", "abstract", "become", "box", "do", "final", "macro",
    "override", "priv", "typeof", "unsized", "virtual", "yield",
];

/// Keywords from 2018 on.
const KEYWORDS_2018: &[&str] = &["async", "await", "dyn", "try"];

/// Keywords from 2024 on.
const KEYWORDS_2024: &[&str] = &["gen"];

/// Whether `s` is a reserved keyword in `edition`. Weak keywords (`union`,
/// `macro_rules`, `default`, `auto`, `safe`, `raw`) are not: they are
/// names except in the places that give them meaning.
pub fn is_keyword(s: &str, edition: Edition) -> bool {
    KEYWORDS.contains(&s)
        || edition >= Edition::E2018 && KEYWORDS_2018.contains(&s)
        || edition >= Edition::E2024 && KEYWORDS_2024.contains(&s)
}

fn is_ident_start(c: char) -> bool {
    c == '_' || c.is_alphabetic()
}

fn is_ident_continue(c: char) -> bool {
    c == '_' || c.is_alphanumeric()
}

impl<'a> Token<'a> {
    pub fn text(self) -> &'a str {
        self.0
    }

    pub fn is_eof(self) -> bool {
        self.0.is_empty()
    }

    /// An identifier, raw or not, which may be a keyword.
    pub fn is_ident(self) -> bool {
        self.0.chars().next().is_some_and(is_ident_start) && !self.is_literal()
    }

    /// `r#name`: a raw identifier, never a keyword.
    pub fn is_raw_ident(self) -> bool {
        self.0.starts_with("r#") && self.0.len() > 2
    }

    /// The name an identifier stands for, without `r#`.
    pub fn ident_name(self) -> &'a str {
        self.0.strip_prefix("r#").unwrap_or(self.0)
    }

    /// A keyword reserved in `edition`, not written raw.
    pub fn is_keyword(self, edition: Edition) -> bool {
        is_keyword(self.0, edition)
    }

    /// `'a`, `'static`, `'_`: a lifetime or a label.
    pub fn is_lifetime(self) -> bool {
        self.0.starts_with('\'') && !self.0[1..].contains('\'')
    }

    /// Any literal: a number, character, byte, string, byte string or C
    /// string, with its suffix if any.
    pub fn is_literal(self) -> bool {
        let b = self.0.as_bytes();
        match b.first() {
            Some(b'0'..=b'9' | b'"') => true,
            Some(b'\'') => !self.is_lifetime(),
            Some(b'b' | b'c') => {
                matches!(b.get(1), Some(b'\'' | b'"')) || b.get(1) == Some(&b'r') && self.is_raw_str(1)
            }
            Some(b'r') => self.is_raw_str(0),
            _ => false,
        }
    }

    /// `r"..."` or `r#"..."#` starting at byte `at`.
    fn is_raw_str(self, at: usize) -> bool {
        let rest = &self.0[at + 1..];
        rest.trim_start_matches('#').starts_with('"')
    }

    /// An integer or float literal.
    pub fn is_number(self) -> bool {
        self.0.as_bytes().first().is_some_and(u8::is_ascii_digit)
    }

    /// A string literal of any kind (plain, raw, byte or C).
    pub fn is_string(self) -> bool {
        self.is_literal() && !self.is_number() && self.0.contains('"')
    }

    /// One character of punctuation.
    pub fn is_punct(self) -> bool {
        let mut cs = self.0.chars();
        matches!((cs.next(), cs.next()), (Some(c), None) if c.is_ascii_punctuation() && c != '_' && c != '\'' && c != '"')
    }

    /// `///`, `//!`, `/** */` or `/*! */`.
    pub fn is_doc_comment(self) -> bool {
        self.0.starts_with("//") || self.0.starts_with("/*")
    }

    /// An inner doc comment, `//!` or `/*!`.
    pub fn is_inner_doc_comment(self) -> bool {
        self.0.starts_with("//!") || self.0.starts_with("/*!")
    }

    /// Whether `next` starts right where this token ends.
    pub fn touches(self, next: Token) -> bool {
        self.0.as_ptr() as usize + self.0.len() == next.0.as_ptr() as usize
    }
}

/// Splits `src` into tokens. The end-of-file token is not included.
pub fn lex(src: &str) -> Result<Vec<Token<'_>>, Error<'_>> {
    let mut lexer = Lexer::new(src);
    let mut tokens = Vec::new();
    loop {
        let t = lexer.next_token()?;
        if t.is_eof() {
            return Ok(tokens);
        }
        tokens.push(t);
    }
}

/// Reads tokens from `src`, one at a time.
pub struct Lexer<'a> {
    src: &'a str,
    pos: usize,
    /// The last two tokens, to tell a tuple index after `.` (`x.0.1`).
    prev: [Token<'a>; 2],
}

impl<'a> Lexer<'a> {
    pub fn new(src: &'a str) -> Self {
        let mut lexer = Lexer { src, pos: 0, prev: [Token(""), Token("")] };
        lexer.skip_shebang();
        lexer
    }

    /// The end-of-file token: the empty slice at the end.
    pub fn eof(&self) -> Token<'a> {
        Token(&self.src[self.src.len()..])
    }

    fn rest(&self) -> &'a str {
        &self.src[self.pos..]
    }

    fn error<T>(&self, at: usize, len: usize, message: impl Into<String>) -> Result<T, Error<'a>> {
        let end = (at + len).min(self.src.len());
        Err(Error::new(&self.src[at..end], message))
    }

    /// `#!` on the first line is a shebang unless it starts an inner
    /// attribute, `#![`, perhaps with white space or comments between.
    fn skip_shebang(&mut self) {
        let Some(rest) = self.src.strip_prefix("#!") else { return };
        let mut after = Lexer { src: rest, pos: 0, prev: self.prev };
        if after.skip_trivia().is_ok() && after.rest().starts_with('[') {
            return;
        }
        self.pos = self.src.find('\n').unwrap_or(self.src.len());
    }

    /// Skips white space and comments other than doc comments.
    fn skip_trivia(&mut self) -> Result<(), Error<'a>> {
        loop {
            let rest = self.rest();
            let trimmed = rest.trim_start();
            self.pos += rest.len() - trimmed.len();
            if trimmed.starts_with("//") {
                if is_doc_line(trimmed) {
                    return Ok(());
                }
                self.pos += trimmed.find('\n').unwrap_or(trimmed.len());
            } else if trimmed.starts_with("/*") {
                if is_doc_block(trimmed) {
                    return Ok(());
                }
                self.pos += self.block_comment_len()?;
            } else {
                return Ok(());
            }
        }
    }

    /// The length of the block comment at `pos`, which may nest.
    fn block_comment_len(&self) -> Result<usize, Error<'a>> {
        let b = self.rest().as_bytes();
        let mut depth = 0;
        let mut i = 0;
        while i + 1 < b.len() {
            if b[i] == b'/' && b[i + 1] == b'*' {
                depth += 1;
                i += 2;
            } else if b[i] == b'*' && b[i + 1] == b'/' {
                depth -= 1;
                i += 2;
                if depth == 0 {
                    return Ok(i);
                }
            } else {
                i += 1;
            }
        }
        self.error(self.pos, 2, "unterminated block comment")
    }

    /// The next token, or the end-of-file token.
    pub fn next_token(&mut self) -> Result<Token<'a>, Error<'a>> {
        self.skip_trivia()?;
        let start = self.pos;
        let len = self.token_len()?;
        self.pos += len;
        let t = Token(&self.src[start..start + len]);
        self.prev = [self.prev[1], t];
        Ok(t)
    }

    fn token_len(&self) -> Result<usize, Error<'a>> {
        let rest = self.rest();
        let Some(c) = rest.chars().next() else { return Ok(0) };
        if rest.starts_with("//") {
            return Ok(rest.find('\n').unwrap_or(rest.len()));
        }
        if rest.starts_with("/*") {
            return self.block_comment_len();
        }
        if c.is_ascii_digit() {
            return Ok(self.number_len());
        }
        if c == '\'' {
            return self.quote_len(0);
        }
        if c == '"' {
            return self.string_len(0);
        }
        // Prefixed literals: b'x', b"..", br"..", c"..", cr"..", r"..",
        // r#".."#; and raw identifiers, r#name.
        let b = rest.as_bytes();
        match (b[0], b.get(1), b.get(2)) {
            (b'b', Some(b'\''), _) => return self.quote_len(1),
            (b'b' | b'c', Some(b'"'), _) => return self.string_len(1),
            (b'b' | b'c', Some(b'r'), Some(b'"' | b'#')) => return self.raw_string_len(2),
            (b'r', Some(b'"'), _) => return self.raw_string_len(1),
            (b'r', Some(b'#'), Some(&n)) if n == b'"' || n == b'#' => return self.raw_string_len(1),
            (b'r', Some(b'#'), Some(_)) => return Ok(2 + self.ident_len(2)),
            _ => {}
        }
        if is_ident_start(c) {
            return Ok(self.ident_len(0));
        }
        if c.is_ascii_punctuation() || c == '$' {
            return Ok(1);
        }
        self.error(self.pos, c.len_utf8(), format!("unexpected character {c:?}"))
    }

    /// Identifier characters from byte `at` of the rest.
    fn ident_len(&self, at: usize) -> usize {
        let rest = &self.rest()[at..];
        rest.find(|c: char| !is_ident_continue(c)).unwrap_or(rest.len())
    }

    /// An integer or float, with its suffix. After a lone `.`, as in
    /// `x.0.1`, only the digits are read, so a tuple index is never a
    /// float.
    fn number_len(&self) -> usize {
        let rest = self.rest();
        let b = rest.as_bytes();
        let digits = |from: usize, radix: u32| {
            from + rest[from..].find(|c: char| !(c == '_' || c.is_digit(radix))).unwrap_or(rest.len() - from)
        };
        let ident_end = |from: usize| from + self.ident_len(from);
        if b[0] == b'0' && matches!(b.get(1), Some(b'x' | b'o' | b'b')) {
            let radix = match b[1] {
                b'x' => 16,
                b'o' => 8,
                _ => 2,
            };
            // Digits of any radix are taken, so `0b12` is one bad token
            // rather than two good ones; the suffix follows.
            let end = digits(2, if radix == 16 { 16 } else { 10 });
            return ident_end(end);
        }
        let mut end = digits(0, 10);
        let range = self.prev[0].0 == "." && self.prev[0].touches(self.prev[1]);
        let tuple_index = self.prev[1].0 == "." && !range;
        if tuple_index {
            return end;
        }
        // `1.5`, and `1.` when what follows cannot continue an expression
        // as a field, method or range.
        if b.get(end) == Some(&b'.') {
            let next = rest[end + 1..].chars().next();
            if next.is_some_and(|c| c.is_ascii_digit()) {
                end = digits(end + 1, 10);
            } else if !next.is_some_and(|c| c == '.' || is_ident_start(c)) {
                return end + 1;
            }
        }
        // An exponent: `e`, an optional sign, then at least one digit.
        if matches!(b.get(end), Some(b'e' | b'E')) {
            let mut k = end + 1;
            if matches!(b.get(k), Some(b'+' | b'-')) {
                k += 1;
            }
            let d = digits(k, 10);
            if rest[k..d].bytes().any(|c| c.is_ascii_digit()) {
                end = d;
            }
        }
        ident_end(end)
    }

    /// `'x'`, `b'x'`, or a lifetime `'a`, starting at byte `at` (after any
    /// `b`).
    fn quote_len(&self, at: usize) -> Result<usize, Error<'a>> {
        let rest = self.rest();
        let body = &rest[at + 1..];
        let mut cs = body.chars();
        match cs.next() {
            Some('\\') => {
                let end = self.escape_end(body, 0)?;
                if body[end..].starts_with('\'') {
                    return Ok(at + 1 + end + 1 + self.ident_len(at + 1 + end + 1));
                }
                self.error(self.pos, at + 1 + end, "unterminated character literal")
            }
            Some(c) if c != '\'' && c != '\n' => {
                let after = c.len_utf8();
                if body[after..].starts_with('\'') {
                    return Ok(at + 1 + after + 1 + self.ident_len(at + 1 + after + 1));
                }
                if at == 0 && (is_ident_start(c) || c.is_ascii_digit()) {
                    // A lifetime or label; `'r#name` is a raw lifetime.
                    let raw = if body.starts_with("r#") { 2 } else { 0 };
                    return Ok(1 + raw + self.ident_len(1 + raw));
                }
                self.error(self.pos, at + 1 + after, "unterminated character literal")
            }
            _ => self.error(self.pos, at + 1, "empty or unterminated character literal"),
        }
    }

    /// The byte offset just past the escape that starts at `at` in `body`.
    fn escape_end(&self, body: &str, at: usize) -> Result<usize, Error<'a>> {
        let after = &body[at + 1..];
        if after.starts_with("u{") {
            if let Some(close) = after.find('}') {
                return Ok(at + 1 + close + 1);
            }
        } else if after.starts_with('x') {
            return Ok((at + 4).min(body.len()));
        } else if let Some(c) = after.chars().next() {
            return Ok(at + 1 + c.len_utf8());
        }
        let off = body.as_ptr() as usize - self.src.as_ptr() as usize;
        self.error(off + at, 1, "bad escape")
    }

    /// `"..."` starting at byte `at` (after any `b` or `c`), with its
    /// suffix.
    fn string_len(&self, at: usize) -> Result<usize, Error<'a>> {
        let rest = self.rest();
        let b = rest.as_bytes();
        let mut i = at + 1;
        while i < b.len() {
            match b[i] {
                b'\\' => i += 2,
                b'"' => return Ok(i + 1 + self.ident_len(i + 1)),
                _ => i += 1,
            }
        }
        self.error(self.pos, at + 1, "unterminated string")
    }

    /// `r"..."` or `r##"..."##`, where `at` is the offset of the `r`'s
    /// following character.
    fn raw_string_len(&self, at: usize) -> Result<usize, Error<'a>> {
        let rest = self.rest();
        let hashes = rest[at..].bytes().take_while(|&c| c == b'#').count();
        if rest.as_bytes().get(at + hashes) != Some(&b'"') {
            return self.error(self.pos, at + hashes, "expected `\"` in raw string");
        }
        let body = at + hashes + 1;
        let close: String = std::iter::once('"').chain(std::iter::repeat_n('#', hashes)).collect();
        match rest[body..].find(&close) {
            Some(k) => {
                let end = body + k + close.len();
                Ok(end + self.ident_len(end))
            }
            None => self.error(self.pos, body, "unterminated raw string"),
        }
    }
}

fn is_doc_line(s: &str) -> bool {
    (s.starts_with("///") && !s.starts_with("////")) || s.starts_with("//!")
}

fn is_doc_block(s: &str) -> bool {
    (s.starts_with("/**") && !s.starts_with("/***") && !s.starts_with("/**/")) || s.starts_with("/*!")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn toks(s: &str) -> Vec<&str> {
        lex(s).unwrap().into_iter().map(|t| t.0).collect()
    }

    #[test]
    fn punctuation_is_one_character() {
        assert_eq!(toks("a::b>>=c"), ["a", ":", ":", "b", ">", ">", "=", "c"]);
        let t = lex("a :: b").unwrap();
        assert!(t[1].touches(t[2]) && !t[0].touches(t[1]));
    }

    #[test]
    fn numbers() {
        assert_eq!(
            toks("1 1u8 1.5f32 1e10 1.5e-3 0xffu64 0b1010_1010 1_000"),
            ["1", "1u8", "1.5f32", "1e10", "1.5e-3", "0xffu64", "0b1010_1010", "1_000"]
        );
        assert_eq!(toks("1..2"), ["1", ".", ".", "2"]);
        assert_eq!(toks("1.foo()"), ["1", ".", "foo", "(", ")"]);
        assert_eq!(toks("1. + x"), ["1.", "+", "x"]);
        assert_eq!(toks("x.0.1"), ["x", ".", "0", ".", "1"]);
        assert_eq!(toks("a..0.5"), ["a", ".", ".", "0.5"]);
        assert_eq!(toks("0x1e5"), ["0x1e5"]);
    }

    #[test]
    fn quotes_and_lifetimes() {
        assert_eq!(
            toks("'a' 'a '\\n' '\\u{1F600}' b'x' 'static '_"),
            ["'a'", "'a", "'\\n'", "'\\u{1F600}'", "b'x'", "'static", "'_"]
        );
        assert!(Token("'a").is_lifetime() && !Token("'a'").is_lifetime());
        assert!(Token("'a'").is_literal() && !Token("'a").is_literal());
        assert_eq!(toks("'é'"), ["'é'"]);
    }

    #[test]
    fn strings() {
        assert_eq!(
            toks(r####""a\"b" b"x" c"y" r"z" r#"q"q"# br##"w"#"## "s"suf"####),
            [r#""a\"b""#, r#"b"x""#, r#"c"y""#, r#"r"z""#, r###"r#"q"q"#"###, r###"br##"w"#"##"###, r#""s"suf"#]
        );
        assert_eq!(toks("r#match r#type"), ["r#match", "r#type"]);
        assert!(Token("r#match").is_raw_ident() && Token("r#match").is_ident());
        assert!(!Token(r#"r"z""#).is_ident() && Token(r#"r"z""#).is_literal());
    }

    #[test]
    fn comments() {
        assert_eq!(toks("a // x\n /* b /* c */ d */ e"), ["a", "e"]);
        assert_eq!(
            toks("/// doc\n//! inner\n//// not\n/** blk */ /*! in */ /**/ x"),
            ["/// doc", "//! inner", "/** blk */", "/*! in */", "x"]
        );
        assert!(lex("/* open").is_err());
    }

    #[test]
    fn shebang() {
        assert_eq!(toks("#!/usr/bin/env run\nfn"), ["fn"]);
        assert_eq!(toks("#![allow(x)]"), ["#", "!", "[", "allow", "(", "x", ")", "]"]);
    }

    #[test]
    fn keywords_by_edition() {
        assert!(is_keyword("async", Edition::E2018) && !is_keyword("async", Edition::E2015));
        assert!(is_keyword("gen", Edition::E2024) && !is_keyword("gen", Edition::E2021));
        assert!(!is_keyword("union", Edition::E2024));
    }
}
