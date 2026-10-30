//! The lexer for SystemVerilog text (clause 5).
//!
//! A token is just a slice of the source, as in `epitome_ir`'s lexer. Its
//! kind is worked out from its first characters when needed, and its
//! position from where the slice sits in the source (see `source`).
//!
//! Tokens are made on demand, from a byte offset, because how text splits
//! depends on where the parser is: after a base such as `'h` the next token
//! is digits even if it looks like a name (`'h ab`), and a UDP table has its
//! own single-character entries.

use crate::keywords::Keywords;
use crate::source::Error;

/// A token: a slice of the source. The end-of-file token is the empty slice
/// at the end of the text.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Token<'a>(pub &'a str);

impl<'a> Token<'a> {
    pub fn text(self) -> &'a str {
        self.0
    }

    pub fn is_eof(self) -> bool {
        self.0.is_empty()
    }

    /// An identifier, simple or escaped, which may still be a keyword.
    pub fn is_ident(self) -> bool {
        self.0.bytes().next().is_some_and(|c| c.is_ascii_alphabetic() || c == b'_') || self.is_escaped_ident()
    }

    /// `\name`: an escaped identifier, never a keyword (5.6.1).
    pub fn is_escaped_ident(self) -> bool {
        self.0.starts_with('\\') && !self.is_line_continuation()
    }

    /// A backslash ending a line, which continues a macro's text onto the
    /// next line (22.5.1). Only the preprocessor accepts it.
    pub fn is_line_continuation(self) -> bool {
        self.0 == "\\\n" || self.0 == "\\\r\n"
    }

    /// The name an identifier stands for. An escaped identifier's
    /// backslash is not part of it, so `\cpu3` and `cpu3` are the same name.
    pub fn ident_name(self) -> &'a str {
        self.0.strip_prefix('\\').unwrap_or(self.0)
    }

    /// A reserved keyword of 1800-2023 (Annex B), not escaped.
    pub fn is_keyword(self) -> bool {
        !self.is_escaped_ident() && is_keyword(self.0)
    }

    /// A simple or escaped identifier that is not a keyword.
    pub fn is_name(self) -> bool {
        self.is_ident() && !self.is_keyword()
    }

    /// `$display`, `$unit`, `$root`: a system task, function or scope name.
    pub fn is_system(self) -> bool {
        self.0.len() > 1 && self.0.starts_with('$')
    }

    /// `` `define ``, `` `FOO ``, `` `\M@CRO ``: a compiler directive or
    /// macro use.
    pub fn is_directive(self) -> bool {
        let b = self.0.as_bytes();
        b.len() > 1
            && b[0] == b'`'
            && (b[1].is_ascii_alphabetic() || b[1] == b'_' || b[1] == b'\\' && b.len() > 2 && b[2] != b'`')
    }

    /// The name of a directive or macro use, without the backtick or an
    /// escaped name's backslash.
    pub fn directive_name(self) -> &'a str {
        let name = &self.0[1..];
        name.strip_prefix('\\').unwrap_or(name)
    }

    /// `"..."` or `"""..."""`.
    pub fn is_string(self) -> bool {
        self.0.starts_with('"')
    }

    /// `"""..."""` (5.9).
    pub fn is_triple_string(self) -> bool {
        self.0.len() >= 6 && self.0.starts_with("\"\"\"")
    }

    /// A decimal number, real or time literal: `12`, `1_000`, `1.5e-3`, `10ns`.
    pub fn is_number(self) -> bool {
        self.0.bytes().next().is_some_and(|c| c.is_ascii_digit())
    }

    /// The base part of a based literal: `'b`, `'sh`, `'D`. The token after
    /// it is the digits.
    pub fn is_base(self) -> bool {
        let b = self.0.as_bytes();
        let s = b.get(1).is_some_and(|c| *c == b's' || *c == b'S') as usize;
        b.first() == Some(&b'\'') && b.len() == 2 + s && b"bBoOdDhH".contains(&b[1 + s])
    }

    /// The base letter of a base token, in lower case.
    pub fn base(self) -> Option<u8> {
        self.is_base().then(|| self.0.as_bytes()[self.0.len() - 1].to_ascii_lowercase())
    }

    /// Whether a base token has the `s` marker.
    pub fn is_signed_base(self) -> bool {
        self.0.len() == 3 && self.is_base()
    }

    /// `'0`, `'1`, `'x` or `'z` (5.7.1).
    pub fn is_unbased_unsized(self) -> bool {
        self.0.len() == 2 && self.0.starts_with('\'') && b"01xXzZ".contains(&self.0.as_bytes()[1])
    }

    /// An operator or punctuation token equal to `s`.
    pub fn is(self, s: &str) -> bool {
        self.0 == s
    }
}

/// A reserved keyword of 1800-2023, the default set.
pub fn is_keyword(s: &str) -> bool {
    Keywords::LATEST.contains(s)
}

/// Operators and punctuation, longest first so the first match is the
/// longest one (5.5, 11.3).
const OPERATORS: &[&str] = &[
    "<<<=", ">>>=", //
    "===", "!==", "==?", "!=?", "<<<", ">>>", "<<=", ">>=", "<->", "|->", "|=>", "->>", "#-#", "#=#", "&&&", "==",
    "!=", "<=", ">=", "&&", "||", "**", "<<", ">>", "++", "--", "+=", "-=", "*=", "/=", "%=", "&=", "|=", "^=", "->",
    "~&", "~|", "~^", "^~", "::", ":=", ":/", "##", ".*", "@@", "+:", "-:", "=>", "*>", //
    "(", ")", "[", "]", "{", "}", ";", ",", ".", ":", "?", "#", "@", "=", "+", "-", "*", "/", "%", "&", "|", "^", "~",
    "!", "<", ">",
];

/// Time units that may follow a number with no space: `10ns`, `1step` (5.8).
const TIME_UNITS: &[&str] = &["step", "ms", "us", "ns", "ps", "fs", "s"];

fn radix(base: u8) -> u32 {
    match base {
        b'b' => 2,
        b'o' => 8,
        b'h' => 16,
        _ => 10,
    }
}

fn is_ident_char(c: u8) -> bool {
    c.is_ascii_alphanumeric() || c == b'_' || c == b'$'
}

/// Makes tokens from a source text on demand.
pub struct Lexer<'a> {
    src: &'a str,
    pos: usize,
    /// Set after a base token: the next token is that base's digits.
    digits_of: Option<u8>,
}

impl<'a> Lexer<'a> {
    pub fn new(src: &'a str) -> Self {
        Lexer { src, pos: 0, digits_of: None }
    }

    /// The byte offset of the next token, before white space.
    pub fn pos(&self) -> usize {
        self.pos
    }

    /// Moves to byte offset `pos`, for the parser to back up.
    pub fn set_pos(&mut self, pos: usize) {
        self.pos = pos;
        self.digits_of = None;
    }

    /// The whole text this lexer reads.
    pub fn text(&self) -> &'a str {
        self.src
    }

    /// Lexes the next token as the digits of a literal with base `base`
    /// (`b'b'`, `b'o'`, `b'd'` or `b'h'`), if it starts like digits. The
    /// preprocessor uses this when a base and its digits come from
    /// different macros.
    pub fn expect_digits(&mut self, base: u8) {
        self.digits_of = Some(base);
    }

    fn rest(&self) -> &'a str {
        &self.src[self.pos..]
    }

    fn peek(&self, n: usize) -> Option<u8> {
        self.src.as_bytes().get(self.pos + n).copied()
    }

    /// The token from `start` to the current position.
    fn token(&self, start: usize) -> Token<'a> {
        Token(&self.src[start..self.pos])
    }

    fn error(&self, start: usize, len: usize, message: impl Into<String>) -> Error<'a> {
        let end = (start + len).min(self.src.len());
        Error::new(&self.src[start..end], message)
    }

    /// Skips white space and comments.
    fn skip_space(&mut self) -> Result<(), Error<'a>> {
        loop {
            let rest = self.rest();
            if let Some(c) = rest.bytes().next()
                && c.is_ascii_whitespace()
            {
                self.pos += 1;
            } else if rest.starts_with("//") {
                self.pos += rest.find('\n').unwrap_or(rest.len());
            } else if let Some(body) = rest.strip_prefix("/*") {
                // Block comments do not nest (5.4).
                let Some(end) = body.find("*/") else {
                    return Err(self.error(self.pos, 2, "unterminated block comment"));
                };
                self.pos += end + 4;
            } else {
                return Ok(());
            }
        }
    }

    /// The next token. At the end of the text this is the empty token,
    /// as many times as it is asked for.
    pub fn next_token(&mut self) -> Result<Token<'a>, Error<'a>> {
        self.skip_space()?;
        let start = self.pos;
        // Without digits after it, a base is left for the parser to report:
        // in a macro's text the digits may come from another macro.
        if let Some(base) = self.digits_of.take()
            && self
                .peek(0)
                .is_some_and(|c| c == b'_' || c == b'?' || (c as char).is_digit(radix(base)) || b"xXzZ".contains(&c))
        {
            return self.based_digits(base);
        }
        let Some(c) = self.peek(0) else { return Ok(self.token(start)) };
        match c {
            b'a'..=b'z' | b'A'..=b'Z' | b'_' => {
                self.take_while(is_ident_char);
            }
            b'\\' => {
                // Up to the next white space, which is not part of the name.
                self.pos += 1;
                for end in ["\n", "\r\n"] {
                    if self.rest().starts_with(end) {
                        self.pos += end.len();
                        return Ok(self.token(start));
                    }
                }
                self.take_while(|c| c.is_ascii_graphic());
                if self.pos == start + 1 {
                    return Err(self.error(start, 1, "empty escaped identifier"));
                }
            }
            b'$' => {
                self.pos += 1;
                self.take_while(is_ident_char);
            }
            b'`' => return self.backtick(),
            b'"' => return self.string(),
            b'0'..=b'9' => self.number(),
            b'\'' => return self.apostrophe(),
            _ => {
                let rest = self.rest();
                let Some(op) = OPERATORS.iter().find(|op| rest.starts_with(**op)) else {
                    let len = rest.chars().next().map_or(1, char::len_utf8);
                    return Err(self.error(start, len, format!("unexpected character `{}`", &rest[..len])));
                };
                self.pos += op.len();
            }
        }
        Ok(self.token(start))
    }

    fn take_while(&mut self, f: impl Fn(u8) -> bool) {
        while self.peek(0).is_some_and(&f) {
            self.pos += 1;
        }
    }

    /// A directive or macro use, or one of the macro-body forms
    /// `` `" ``, `` `""" ``, ` `` ` and `` `\`" `` (22.5.1). A macro's name
    /// may be an escaped identifier: `` `\M@CRO ``.
    fn backtick(&mut self) -> Result<Token<'a>, Error<'a>> {
        let start = self.pos;
        let rest = &self.rest()[1..];
        if rest.bytes().next().is_some_and(|c| c.is_ascii_alphabetic() || c == b'_') {
            self.pos += 1;
            self.take_while(is_ident_char);
        } else if let Some(form) = ["\\`\"", "`", "\"\"\"", "\""].iter().find(|f| rest.starts_with(**f)) {
            self.pos += 1 + form.len();
        } else if rest.starts_with('\\') && rest.as_bytes().get(1).is_some_and(|c| c.is_ascii_graphic()) {
            self.pos += 2;
            self.take_while(|c| c.is_ascii_graphic());
        } else {
            return Err(self.error(start, 1, "expected a directive name after `"));
        }
        Ok(self.token(start))
    }

    /// `"..."` with backslash escapes and line continuations, or
    /// `"""..."""`, which may also hold newlines (5.9).
    fn string(&mut self) -> Result<Token<'a>, Error<'a>> {
        let start = self.pos;
        let triple = self.rest().starts_with("\"\"\"");
        self.pos += if triple { 3 } else { 1 };
        loop {
            match self.peek(0) {
                None => return Err(self.error(start, 1, "unterminated string")),
                Some(b'\\') => self.pos += 2,
                Some(b'\n') if !triple => return Err(self.error(start, 1, "unterminated string")),
                Some(b'"') if !triple => {
                    self.pos += 1;
                    break;
                }
                Some(b'"') if self.rest().starts_with("\"\"\"") => {
                    self.pos += 3;
                    break;
                }
                Some(_) => self.pos += 1,
            }
        }
        self.pos = self.pos.min(self.src.len());
        Ok(self.token(start))
    }

    /// A size or an unsigned number, a real, or a time literal (5.7, 5.8).
    fn number(&mut self) {
        let digits = |c: u8| c.is_ascii_digit() || c == b'_';
        self.take_while(digits);
        if self.peek(0) == Some(b'.') && self.peek(1).is_some_and(|c| c.is_ascii_digit()) {
            self.pos += 1;
            self.take_while(digits);
        }
        if matches!(self.peek(0), Some(b'e' | b'E')) {
            let sign = matches!(self.peek(1), Some(b'+' | b'-')) as usize;
            if self.peek(1 + sign).is_some_and(|c| c.is_ascii_digit()) {
                self.pos += 1 + sign;
                self.take_while(digits);
            }
        }
        let rest = self.rest();
        if let Some(unit) = TIME_UNITS.iter().find(|u| rest.starts_with(**u))
            && !rest.as_bytes().get(unit.len()).is_some_and(|&c| is_ident_char(c))
        {
            self.pos += unit.len();
        }
    }

    /// After `'`: a base (`'h`, `'sb`), an unbased unsized literal (`'1`),
    /// `'{`, or a lone `'` as in a cast `int'(x)`.
    fn apostrophe(&mut self) -> Result<Token<'a>, Error<'a>> {
        let start = self.pos;
        let is_base = |c: Option<u8>| c.is_some_and(|c| b"bBoOdDhH".contains(&c));
        let signed = matches!(self.peek(1), Some(b's' | b'S')) as usize;
        if is_base(self.peek(1 + signed)) {
            self.pos += 2 + signed;
            self.digits_of = Some(self.src.as_bytes()[self.pos - 1].to_ascii_lowercase());
        } else if self.peek(1) == Some(b'{')
            || self.peek(1).is_some_and(|c| b"01xXzZ".contains(&c)) && !self.peek(2).is_some_and(is_ident_char)
        {
            self.pos += 2;
        } else {
            // 5.7.1: the apostrophe and the base letter may not be split.
            let after = self.rest()[1..].trim_start_matches([' ', '\t']);
            let gap = self.rest().len() - 1 - after.len();
            let b = after.as_bytes();
            let s = matches!(b.first(), Some(b's' | b'S')) as usize;
            if gap > 0 && is_base(b.get(s).copied()) && b.get(s + 1).is_some_and(|c| c.is_ascii_hexdigit()) {
                return Err(self.error(start, 1 + gap, "white space between ' and the base of a literal"));
            }
            self.pos += 1;
        }
        Ok(self.token(start))
    }

    /// The digits after a base token, checked against the base (5.7.1).
    fn based_digits(&mut self, base: u8) -> Result<Token<'a>, Error<'a>> {
        let start = self.pos;
        self.take_while(|c| c.is_ascii_alphanumeric() || c == b'_' || c == b'?');
        let t = self.token(start).0;
        if t.starts_with('_') {
            return Err(self.error(start, 1, "the digits of a literal cannot start with `_`"));
        }
        let xz = |c: char| matches!(c, 'x' | 'X' | 'z' | 'Z' | '?');
        let ok = |c: char| match base {
            b'b' => matches!(c, '0' | '1' | '_') || xz(c),
            b'o' => matches!(c, '0'..='7' | '_') || xz(c),
            b'h' => c.is_ascii_hexdigit() || c == '_' || xz(c),
            _ => c.is_ascii_digit() || c == '_',
        };
        if let Some((i, c)) = t.char_indices().find(|&(_, c)| !ok(c)) {
            // A decimal literal may be one x or z digit and underscores.
            let single_xz = base == b'd' && xz(t.as_bytes()[0] as char) && t[1..].bytes().all(|c| c == b'_');
            if !single_xz {
                return Err(self.error(start + i, 1, format!("`{c}` is not a digit of base '{}", base as char)));
            }
        }
        Ok(Token(t))
    }
}

/// Splits `src` into tokens. The last token is always the empty
/// end-of-file token.
pub fn tokenize(src: &str) -> Result<Vec<Token<'_>>, Error<'_>> {
    let mut lexer = Lexer::new(src);
    let mut toks = Vec::new();
    loop {
        let t = lexer.next_token()?;
        toks.push(t);
        if t.is_eof() {
            return Ok(toks);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::source::Source;

    fn texts(src: &str) -> Vec<&str> {
        tokenize(src).unwrap().iter().map(|t| t.0).collect()
    }

    #[test]
    fn keywords() {
        assert!(Token("module").is_keyword());
        assert!(!Token("\\module").is_keyword());
        assert!(Token("\\module").is_name());
    }

    #[test]
    fn module_header() {
        assert_eq!(
            texts("module top #(parameter W = 8) (input logic [W-1:0] a); // c\nendmodule"),
            [
                "module",
                "top",
                "#",
                "(",
                "parameter",
                "W",
                "=",
                "8",
                ")",
                "(",
                "input",
                "logic",
                "[",
                "W",
                "-",
                "1",
                ":",
                "0",
                "]",
                "a",
                ")",
                ";",
                "endmodule",
                ""
            ]
        );
    }

    #[test]
    fn operators_longest_match() {
        assert_eq!(texts("a<<<=b|->c ##1 d !=?e"), ["a", "<<<=", "b", "|->", "c", "##", "1", "d", "!=?", "e", ""]);
        assert_eq!(texts("@(*)"), ["@", "(", "*", ")", ""]);
    }

    #[test]
    fn identifiers() {
        assert_eq!(texts("\\bus+index  a$b $display $"), ["\\bus+index", "a$b", "$display", "$", ""]);
        assert_eq!(Token("\\cpu3").ident_name(), "cpu3");
    }

    #[test]
    fn numbers() {
        assert_eq!(
            texts("8'hFF 8 'h ab 4'sb1x?z 'd5 '1 'z"),
            ["8", "'h", "FF", "8", "'h", "ab", "4", "'sb", "1x?z", "'d", "5", "'1", "'z", ""]
        );
        assert_eq!(
            texts("1.5e-3 1e3 10ns 1step 1_000 2.5ps"),
            ["1.5e-3", "1e3", "10ns", "1step", "1_000", "2.5ps", ""]
        );
        assert_eq!(
            texts("6'dx int'(x) '{1, 2}"),
            ["6", "'d", "x", "int", "'", "(", "x", ")", "'{", "1", ",", "2", "}", ""]
        );
        assert!(Token("'sb").is_signed_base());
        assert_eq!(Token("'H").base(), Some(b'h'));
    }

    #[test]
    fn strings() {
        assert_eq!(
            texts(
                r#""a \"b\"" "c\
d""#
            ),
            [r#""a \"b\"""#, "\"c\\\nd\"", ""]
        );
        assert_eq!(texts("\"\"\"x\n\"y\"\n\"\"\" z"), ["\"\"\"x\n\"y\"\n\"\"\"", "z", ""]);
    }

    #[test]
    fn macro_text() {
        assert_eq!(
            texts("`define H 'h\n`define M (1 + \\\n 2)"),
            ["`define", "H", "'h", "`define", "M", "(", "1", "+", "\\\n", "2", ")", ""]
        );
        assert!(Token("\\\n").is_line_continuation() && !Token("\\\n").is_ident());
    }

    #[test]
    fn directives() {
        assert_eq!(
            texts("`define M(a) `\"a`\" ``x `\\`\""),
            ["`define", "M", "(", "a", ")", "`\"", "a", "`\"", "``", "x", "`\\`\"", ""]
        );
        assert_eq!(texts("`\"\"\"y`\"\"\""), ["`\"\"\"", "y", "`\"\"\"", ""]);
        let t = texts("`\\M@CRO3 (1)");
        assert_eq!(t, ["`\\M@CRO3", "(", "1", ")", ""]);
        assert!(Token(t[0]).is_directive() && Token(t[0]).directive_name() == "M@CRO3");
        assert!(!Token("`\\`\"").is_directive() && !Token("`\"").is_directive());
    }

    fn error(text: &str) -> String {
        let src = Source::new("t.sv", text);
        let e = tokenize(&src.text).unwrap_err();
        let loc = src.locate(e.at);
        format!("{}:{}: {}", loc.line, loc.col, e.message)
    }

    #[test]
    fn errors_point_at_the_slice() {
        assert_eq!(error("x = 8' hC4;"), "1:6: white space between ' and the base of a literal");
        assert_eq!(error("x =\n 8'd4x;"), "2:6: `x` is not a digit of base 'd");
        assert_eq!(error("x = 'b_01;"), "1:7: the digits of a literal cannot start with `_`");
        assert_eq!(error("a /* b"), "1:3: unterminated block comment");
        assert_eq!(error("s = \"abc\n"), "1:5: unterminated string");
        assert_eq!(error("a £ b"), "1:3: unexpected character `£`");
    }
}
