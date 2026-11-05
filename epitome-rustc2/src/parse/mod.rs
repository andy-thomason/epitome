//! The parser: tokens to an `Ast`.
//!
//! A hand-written recursive-descent parser, with precedence climbing for
//! binary operators. Every parse function pushes the node it built onto
//! `stack`; `finish(mark, kind)` gathers what was pushed since `mark` as
//! the children of a new node. Wrapping a node that is already built (a
//! binary operator's left side, a method call's receiver) is another
//! `finish` with the earlier mark.
//!
//! Punctuation tokens are single characters (see `lexer`), so an operator
//! such as `::` or `>>=` is several tokens that touch. `at_op` checks for
//! one, and a closing `>` of generic arguments is always one token, even in
//! `Vec<Vec<u8>>`.
//!
//! Where Rust cannot be read with a fixed lookahead (a parameter that is a
//! pattern and a type, or only a type; an associated constraint or a type
//! argument) the parser tries one reading and backs up to try the other.
//!
//! The first error stops the parse.

mod expressions;
mod items;
mod patterns;
mod types;

#[cfg(test)]
mod tests;

use crate::ast::{Ast, Kind, NodeId};
use crate::lexer::{Edition, Token};
use crate::source::Error;

pub(crate) type PResult<'a, T = ()> = Result<T, Error<'a>>;

/// Parses a whole file (a crate root or a module) from its tokens.
/// `eof` is the empty slice at the end of the text, for errors at the end.
pub fn parse<'a>(tokens: Vec<Token<'a>>, eof: Token<'a>, edition: Edition) -> Result<Ast<'a>, Error<'a>> {
    let mut p = Parser::new(tokens, eof, edition);
    let m = p.mark();
    p.inner_attributes()?;
    p.items_until(None)?;
    p.ast.root = p.finish(m, Kind::Crate);
    Ok(p.ast)
}

/// Lexes and parses `src`.
pub fn parse_str(src: &str, edition: Edition) -> Result<Ast<'_>, Error<'_>> {
    let tokens = crate::lexer::lex(src)?;
    parse(tokens, Token(&src[src.len()..]), edition)
}

/// What to parse a fragment as, for macros' fragment specifiers and tests.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Fragment {
    Expr,
    Type,
    Pattern,
    Item,
    Stmts,
    /// Items until the end; the root is a `Crate` node holding them.
    Items,
    /// An attribute's contents without `#[` and `]`, as in `cfg_attr`:
    /// the root is an `Attribute`.
    Meta,
}

/// Parses all of `tokens` as one fragment; the node is the AST's root.
pub fn parse_fragment<'a>(
    tokens: Vec<Token<'a>>,
    eof: Token<'a>,
    edition: Edition,
    what: Fragment,
) -> Result<Ast<'a>, Error<'a>> {
    let mut p = Parser::new(tokens, eof, edition);
    let root = match what {
        Fragment::Expr => {
            p.expr()?;
            p.top()
        }
        Fragment::Type => {
            p.ty()?;
            p.top()
        }
        Fragment::Pattern => {
            p.pattern()?;
            p.top()
        }
        Fragment::Item => {
            p.item()?;
            p.top()
        }
        Fragment::Stmts => {
            let m = p.mark();
            while !p.at_eof() {
                p.stmt(None)?;
            }
            p.finish(m, Kind::Block)
        }
        Fragment::Items => {
            let m = p.mark();
            p.items_until(None)?;
            p.finish(m, Kind::Crate)
        }
        Fragment::Meta => {
            let m = p.mark();
            p.meta()?;
            p.finish(m, Kind::Attribute)
        }
    };
    if !p.at_eof() {
        return p.error("expected the end of the input");
    }
    p.ast.root = root;
    Ok(p.ast)
}

/// Where a node starts: the token position and the depth of the stack.
#[derive(Clone, Copy)]
pub(crate) struct Mark {
    pos: usize,
    stack: usize,
}

/// Everything needed to back up.
#[derive(Clone, Copy)]
pub(crate) struct Save {
    pos: usize,
    stack: usize,
    nodes: (usize, usize),
}

/// Operators, longest first, so the first that matches touching tokens is
/// the one meant.
const OPERATORS: &[&str] = &[
    "<<=", ">>=", "...", "..=", "::", "->", "=>", "==", "!=", "<=", ">=", "&&", "||", "+=", "-=", "*=", "/=", "%=",
    "^=", "&=", "|=", "<<", ">>", "..",
];

/// Whether the tokens from `i` spell `op`, each touching the next.
fn at_prefix(tokens: &[Token], i: usize, op: &str) -> bool {
    op.bytes().enumerate().all(|(k, c)| {
        tokens
            .get(i + k)
            .is_some_and(|t| t.0.len() == 1 && t.0.as_bytes()[0] == c && (k == 0 || tokens[i + k - 1].touches(*t)))
    })
}

/// The length in tokens of the operator starting at token `i`.
fn operator_len(tokens: &[Token], i: usize) -> usize {
    if !tokens[i].is_punct() {
        return 0;
    }
    OPERATORS.iter().find(|op| at_prefix(tokens, i, op)).map_or(1, |op| op.len())
}

pub(crate) struct Parser<'a> {
    ast: Ast<'a>,
    /// For each token, the length in tokens of the operator that starts
    /// there (see `op_len_n`), worked out once: the parser asks about
    /// operators many times per token.
    op_len: Vec<u8>,
    pos: usize,
    stack: Vec<NodeId>,
    eof: Token<'a>,
    pub(crate) edition: Edition,
}

impl<'a> Parser<'a> {
    fn new(tokens: Vec<Token<'a>>, eof: Token<'a>, edition: Edition) -> Self {
        let op_len = (0..tokens.len()).map(|i| operator_len(&tokens, i) as u8).collect();
        Parser { ast: Ast::new(tokens), op_len, pos: 0, stack: Vec::new(), eof, edition }
    }

    // ---- tokens ----

    pub(crate) fn tok(&self, n: usize) -> Token<'a> {
        self.ast.tokens.get(self.pos + n).copied().unwrap_or(self.eof)
    }

    pub(crate) fn at_eof(&self) -> bool {
        self.pos >= self.ast.tokens.len()
    }

    /// Whether token `n` ahead is the single token `s`.
    pub(crate) fn at_n(&self, n: usize, s: &str) -> bool {
        self.tok(n).0 == s
    }

    pub(crate) fn at(&self, s: &str) -> bool {
        self.at_n(0, s)
    }

    /// Whether the tokens from `n` ahead spell the operator `op`, each
    /// touching the next. A one-character operator must not be the start
    /// of a longer one (`=` is not the start of `=>`).
    pub(crate) fn at_op_n(&self, n: usize, op: &str) -> bool {
        if self.op_len_n(n) != op.len() {
            return false;
        }
        op.chars().enumerate().all(|(k, c)| {
            let t = self.tok(n + k);
            t.0.len() == c.len_utf8() && t.0.starts_with(c)
        })
    }

    pub(crate) fn at_op(&self, op: &str) -> bool {
        self.at_op_n(0, op)
    }

    /// Whether the tokens from `n` start with `op`'s characters, touching,
    /// even if they go on to make a longer operator (`.` `.` at the start
    /// of `..=`).
    pub(crate) fn at_prefix_n(&self, n: usize, op: &str) -> bool {
        at_prefix(&self.ast.tokens, self.pos + n, op)
    }

    /// The length in tokens of the operator at `n`: the longest in
    /// `OPERATORS` spelt by touching tokens, else 1 for any punctuation
    /// and 0 for anything else.
    pub(crate) fn op_len_n(&self, n: usize) -> usize {
        self.op_len.get(self.pos + n).copied().unwrap_or(0) as usize
    }

    pub(crate) fn bump(&mut self) {
        if self.pos < self.ast.tokens.len() {
            self.pos += 1;
        }
    }

    pub(crate) fn bump_n(&mut self, n: usize) {
        for _ in 0..n {
            self.bump();
        }
    }

    /// Eats the single token `s` if it is next.
    pub(crate) fn eat(&mut self, s: &str) -> bool {
        if self.at(s) {
            self.bump();
            true
        } else {
            false
        }
    }

    /// Eats the operator `op` if it is next.
    pub(crate) fn eat_op(&mut self, op: &str) -> bool {
        if self.at_op(op) {
            self.bump_n(op.len());
            true
        } else {
            false
        }
    }

    pub(crate) fn expect(&mut self, s: &str) -> PResult<'a> {
        if self.eat(s) { Ok(()) } else { self.error(&format!("expected `{s}`")) }
    }

    pub(crate) fn expect_op(&mut self, op: &str) -> PResult<'a> {
        if self.eat_op(op) { Ok(()) } else { self.error(&format!("expected `{op}`")) }
    }

    /// Eats one `>` closing generics, even if it touches another token
    /// (`>>`, `>=`).
    pub(crate) fn expect_close_angle(&mut self) -> PResult<'a> {
        self.expect(">")
    }

    pub(crate) fn error<T>(&self, message: &str) -> PResult<'a, T> {
        let t = self.tok(0);
        let found = if t.is_eof() { "the end of the input".to_string() } else { format!("`{}`", t.0) };
        Err(Error::new(t.0, format!("{message}, found {found}")))
    }

    /// Whether token `n` is a keyword in this edition (not raw).
    pub(crate) fn keyword_at(&self, n: usize) -> bool {
        self.tok(n).is_keyword(self.edition)
    }

    /// Whether token `n` is an identifier that is not a keyword or `_`.
    pub(crate) fn name_at(&self, n: usize) -> bool {
        let t = self.tok(n);
        t.is_ident() && t.0 != "_" && (t.is_raw_ident() || !t.is_keyword(self.edition))
    }

    /// Whether token `n` is a lifetime or label.
    pub(crate) fn lifetime_at(&self, n: usize) -> bool {
        self.tok(n).is_lifetime()
    }

    /// Whether token `n` is a literal, including `true` and `false`.
    pub(crate) fn literal_at(&self, n: usize) -> bool {
        let t = self.tok(n);
        t.is_literal() || t.0 == "true" || t.0 == "false"
    }

    // ---- building nodes ----

    pub(crate) fn mark(&self) -> Mark {
        Mark { pos: self.pos, stack: self.stack.len() }
    }

    /// Makes a node of everything pushed since `m`, covering the tokens
    /// read since `m`, and pushes it.
    pub(crate) fn finish(&mut self, m: Mark, kind: Kind) -> NodeId {
        let id = self.ast.push(kind, m.pos, self.pos, &self.stack[m.stack..]);
        self.stack.truncate(m.stack);
        self.stack.push(id);
        id
    }

    /// A leaf of `n` tokens.
    pub(crate) fn leaf(&mut self, kind: Kind, n: usize) -> NodeId {
        let m = self.mark();
        self.bump_n(n);
        self.finish(m, kind)
    }

    /// The last node pushed.
    pub(crate) fn top(&self) -> NodeId {
        *self.stack.last().expect("a node")
    }

    /// The kind of the last node pushed.
    pub(crate) fn top_kind(&self) -> Kind {
        self.ast.kind(self.top())
    }

    pub(crate) fn save(&self) -> Save {
        Save { pos: self.pos, stack: self.stack.len(), nodes: self.ast.sizes() }
    }

    pub(crate) fn restore(&mut self, s: Save) {
        self.pos = s.pos;
        self.stack.truncate(s.stack);
        self.ast.truncate(s.nodes.0, s.nodes.1);
    }

    /// Replaces the node on top of the stack with its children, so that a
    /// node already built can be rebuilt as something else (a path type
    /// that turns out to start a bound list).
    pub(crate) fn unwrap_top(&mut self) {
        let id = self.stack.pop().expect("a node");
        let kids = self.ast.children(id).to_vec();
        self.stack.extend(kids);
    }

    // ---- shared pieces ----

    /// A delimited group of tokens, kept unparsed: `(...)`, `[...]` or
    /// `{...}`.
    pub(crate) fn token_tree(&mut self) -> PResult<'a, NodeId> {
        let m = self.mark();
        self.skip_group()?;
        Ok(self.finish(m, Kind::TokenTree))
    }

    /// Steps over a delimited group, checking that its delimiters match.
    pub(crate) fn skip_group(&mut self) -> PResult<'a> {
        let mut open: Vec<Token<'a>> = Vec::new();
        loop {
            let t = self.tok(0);
            match t.0 {
                "(" | "[" | "{" => open.push(t),
                ")" | "]" | "}" => {
                    let want = match t.0 {
                        ")" => "(",
                        "]" => "[",
                        _ => "{",
                    };
                    match open.pop() {
                        Some(o) if o.0 == want => {}
                        Some(o) => return Err(Error::new(t.0, format!("`{}` does not match `{}`", t.0, o.0))),
                        None => return self.error("expected `(`, `[` or `{`"),
                    }
                }
                "" if t.is_eof() => {
                    return match open.last() {
                        Some(o) => Err(Error::new(o.0, format!("unclosed `{}`", o.0))),
                        None => self.error("expected `(`, `[` or `{`"),
                    };
                }
                _ if open.is_empty() => return self.error("expected `(`, `[` or `{`"),
                _ => {}
            }
            self.bump();
            if open.is_empty() {
                return Ok(());
            }
        }
    }

    /// Outer attributes and doc comments, pushed as children of whatever
    /// node is being built.
    pub(crate) fn outer_attributes(&mut self) -> PResult<'a, usize> {
        let mut n = 0;
        loop {
            let t = self.tok(0);
            if t.is_doc_comment() && !t.is_inner_doc_comment() {
                self.leaf(Kind::DocComment, 1);
            } else if self.at("#") && self.at_n(1, "[") {
                let m = self.mark();
                self.bump();
                self.attribute_body()?;
                self.finish(m, Kind::Attribute);
            } else {
                return Ok(n);
            }
            n += 1;
        }
    }

    /// `#![...]` and inner doc comments.
    pub(crate) fn inner_attributes(&mut self) -> PResult<'a> {
        loop {
            let t = self.tok(0);
            if t.is_inner_doc_comment() {
                self.leaf(Kind::DocComment, 1);
            } else if self.at("#") && self.at_n(1, "!") && self.at_n(2, "[") {
                let m = self.mark();
                self.bump_n(2);
                self.attribute_body()?;
                self.finish(m, Kind::InnerAttribute);
            } else {
                return Ok(());
            }
        }
    }

    /// `[path args]`, `[path = value]`, `[unsafe(...)]`.
    fn attribute_body(&mut self) -> PResult<'a> {
        self.expect("[")?;
        self.meta()?;
        self.expect("]")
    }

    /// `path args`, `path = value`, `unsafe(...)`: an attribute's contents.
    fn meta(&mut self) -> PResult<'a> {
        let is_unsafe = self.at("unsafe") && self.at_n(1, "(");
        if is_unsafe {
            self.leaf(Kind::Keyword, 1);
            self.bump();
        }
        self.path(types::PathMode::Simple)?;
        if self.at("(") || self.at("[") || self.at("{") {
            self.token_tree()?;
        } else if self.at("=") {
            self.bump();
            self.expr()?;
        }
        if is_unsafe {
            self.expect(")")?;
        }
        Ok(())
    }
}
