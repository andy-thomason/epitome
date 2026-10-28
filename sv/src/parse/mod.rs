//! The parser: preprocessed tokens to an `Ast` (Annex A).
//!
//! A hand-written recursive-descent parser. Every parse function pushes the
//! node it built onto `stack`; `finish(mark, kind)` gathers what was pushed
//! since `mark` as the children of a new node. Wrapping a node that is
//! already built (a binary operator's left side, a select after a name) is
//! another `finish` with the earlier mark.
//!
//! Where SystemVerilog cannot be read without knowing which names are types
//! (`a b;` declares `b` of type `a`; `a * b;` is not a declaration), the
//! parser looks ahead at the shape of what follows. Where that is not
//! enough it tries one reading and backs up to try another.
//!
//! The first error stops the parse. The message is that of the error that
//! got furthest through the tokens, across any readings that were tried.

mod assertions;
mod declarations;
mod expressions;
mod items;
mod other;
mod statements;

use crate::ast::{Ast, Kind, NodeId};
use crate::lexer::Token;
use crate::preprocess::Output;
use crate::source::Error;

pub(crate) type PResult<'a, T = ()> = Result<T, Error<'a>>;

/// Parses a preprocessed compilation unit.
pub fn parse<'a>(out: &Output<'a>) -> Result<Ast<'a>, Error<'a>> {
    let tokens: Vec<Token<'a>> = out.tokens.iter().map(|t| t.token).collect();
    let kw: Vec<bool> = out.tokens.iter().map(|&t| out.is_keyword(t)).collect();
    let eof = match tokens.last() {
        Some(t) => Token(&t.0[t.0.len()..]),
        None => Token(""),
    };
    let mut p = Parser {
        ast: Ast::new(tokens),
        kw,
        pos: 0,
        stack: Vec::new(),
        eof,
        furthest: None,
        scope: items::Scope::Unit,
        generate_depth: 0,
        ansi: Vec::new(),
        in_attribute: false,
    };
    let m = p.mark();
    let result = p.items_until(items::Scope::Unit, None);
    match result {
        Ok(()) => {
            p.ast.root = p.finish(m, Kind::SourceText);
            Ok(p.ast)
        }
        Err(e) => Err(p.best_error(e)),
    }
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

pub(crate) struct Parser<'a> {
    ast: Ast<'a>,
    /// Whether each token is a keyword in its `` `begin_keywords `` set.
    kw: Vec<bool>,
    pos: usize,
    stack: Vec<NodeId>,
    eof: Token<'a>,
    /// The error that reached the furthest token, and that token's index.
    furthest: Option<(usize, Error<'a>)>,
    /// What kind of design element the items being read are in.
    scope: items::Scope,
    /// How many generate blocks the items being read are in.
    generate_depth: usize,
    /// For each module being read, whether its header had ANSI ports.
    ansi: Vec<bool>,
    /// Inside an attribute, where another attribute cannot start (5.12).
    in_attribute: bool,
}

impl<'a> Parser<'a> {
    // ---- tokens ----

    pub(crate) fn tok(&self, n: usize) -> Token<'a> {
        self.ast.tokens.get(self.pos + n).copied().unwrap_or(self.eof)
    }

    pub(crate) fn at_eof(&self) -> bool {
        self.pos >= self.ast.tokens.len()
    }

    fn is_kw_at(&self, n: usize) -> bool {
        self.kw.get(self.pos + n).copied().unwrap_or(false)
    }

    /// Whether token `n` ahead is `s`: an operator or punctuation by its
    /// text, a keyword only if it is reserved where it was read.
    pub(crate) fn at_n(&self, n: usize, s: &str) -> bool {
        let t = self.tok(n);
        t.0 == s && (!s.as_bytes()[0].is_ascii_alphabetic() || self.is_kw_at(n))
    }

    pub(crate) fn at(&self, s: &str) -> bool {
        self.at_n(0, s)
    }

    pub(crate) fn at_any(&self, ss: &[&str]) -> bool {
        ss.iter().any(|s| self.at(s))
    }

    /// Whether token `n` ahead is an identifier that is not a keyword.
    pub(crate) fn name_at(&self, n: usize) -> bool {
        self.tok(n).is_ident() && !self.is_kw_at(n)
    }

    /// Whether token `n` ahead is a keyword (any keyword).
    pub(crate) fn keyword_at(&self, n: usize) -> bool {
        self.is_kw_at(n)
    }

    /// Whether the tokens at `n` and `n + 1` touch, with nothing between:
    /// `(*` starts an attribute only when written together.
    pub(crate) fn adjacent(&self, n: usize) -> bool {
        let (a, b) = (self.tok(n), self.tok(n + 1));
        !b.is_eof() && a.0.as_ptr() as usize + a.0.len() == b.0.as_ptr() as usize
    }

    pub(crate) fn bump(&mut self) {
        if self.pos < self.ast.tokens.len() {
            self.pos += 1;
        }
    }

    pub(crate) fn eat(&mut self, s: &str) -> bool {
        if self.at(s) {
            self.bump();
            true
        } else {
            false
        }
    }

    pub(crate) fn expect(&mut self, s: &str) -> PResult<'a> {
        if self.eat(s) { Ok(()) } else { Err(self.expected(&format!("`{s}`"))) }
    }

    // ---- errors ----

    pub(crate) fn error_at(&self, n: usize, msg: impl Into<String>) -> Error<'a> {
        Error::new(self.tok(n).0, msg)
    }

    pub(crate) fn error(&self, msg: impl Into<String>) -> Error<'a> {
        self.error_at(0, msg)
    }

    pub(crate) fn expected(&self, what: &str) -> Error<'a> {
        let found = if self.at_eof() { "the end of the input".to_string() } else { format!("`{}`", self.tok(0).0) };
        self.error(format!("expected {what}, found {found}"))
    }

    /// Remembers an error from a reading that was abandoned, if it got
    /// further than any before.
    fn note(&mut self, pos: usize, e: Error<'a>) {
        if self.furthest.as_ref().is_none_or(|(p, _)| pos > *p) {
            self.furthest = Some((pos, e));
        }
    }

    fn best_error(&mut self, e: Error<'a>) -> Error<'a> {
        let pos = self.pos;
        match self.furthest.take() {
            Some((p, f)) if p > pos => f,
            _ => e,
        }
    }

    // ---- building nodes ----

    pub(crate) fn mark(&self) -> Mark {
        Mark { pos: self.pos, stack: self.stack.len() }
    }

    /// Makes a node of everything pushed since `m`.
    pub(crate) fn finish(&mut self, m: Mark, kind: Kind) -> NodeId {
        let kids = self.stack.split_off(m.stack);
        let id = self.ast.push(kind, m.pos, self.pos, &kids);
        self.stack.push(id);
        id
    }

    /// A leaf of the current token.
    pub(crate) fn leaf(&mut self, kind: Kind) -> NodeId {
        let m = self.mark();
        self.bump();
        self.finish(m, kind)
    }

    /// An empty node at the current position.
    pub(crate) fn empty(&mut self) -> NodeId {
        let m = self.mark();
        self.finish(m, Kind::Empty)
    }

    /// The kind of the node most recently pushed.
    pub(crate) fn last_kind(&self) -> Option<Kind> {
        self.stack.last().map(|&id| self.ast.kind(id))
    }

    pub(crate) fn last(&self) -> Option<NodeId> {
        self.stack.last().copied()
    }

    /// The nodes pushed since `m`.
    pub(crate) fn since(&self, m: Mark) -> &[NodeId] {
        &self.stack[m.stack..]
    }

    /// Pushes the current token as a keyword leaf if it is one of `ss`.
    pub(crate) fn opt_keyword(&mut self, ss: &[&str]) -> bool {
        if self.at_any(ss) {
            self.leaf(Kind::Keyword);
            true
        } else {
            false
        }
    }

    /// Expects an identifier that is not a keyword.
    pub(crate) fn name(&mut self) -> PResult<'a> {
        if self.name_at(0) {
            self.leaf(Kind::Name);
            Ok(())
        } else if self.tok(0).is_ident() {
            Err(self.error(format!("`{}` is a keyword, so it cannot be used as a name", self.tok(0).0)))
        } else {
            Err(self.expected("a name"))
        }
    }

    // ---- backtracking ----

    pub(crate) fn save(&self) -> Save {
        Save { pos: self.pos, stack: self.stack.len(), nodes: self.ast.sizes() }
    }

    pub(crate) fn restore(&mut self, s: Save) {
        self.pos = s.pos;
        self.stack.truncate(s.stack);
        self.ast.truncate(s.nodes.0, s.nodes.1);
    }

    /// Runs `f`; if it fails, backs up to where it started and returns
    /// false. Its error is kept in case it is the best one to report.
    pub(crate) fn attempt(&mut self, f: impl FnOnce(&mut Self) -> PResult<'a>) -> bool {
        let s = self.save();
        match f(self) {
            Ok(()) => true,
            Err(e) => {
                let pos = self.pos;
                self.note(pos, e);
                self.restore(s);
                false
            }
        }
    }

    /// Runs `f` to look ahead, then backs up whatever it did.
    pub(crate) fn lookahead(&mut self, f: impl FnOnce(&mut Self) -> bool) -> bool {
        let s = self.save();
        let saved = self.furthest.take();
        let r = f(self);
        self.restore(s);
        self.furthest = saved;
        r
    }

    /// The index of the token after the bracket that matches the one at
    /// `n` (`(`, `[` or `{`), counting from the current position.
    pub(crate) fn skip_balanced(&self, n: usize) -> Option<usize> {
        let mut depth = 0usize;
        let mut i = n;
        loop {
            let t = self.tok(i);
            if t.is_eof() {
                return None;
            }
            match t.0 {
                "(" | "[" | "{" | "'{" => depth += 1,
                ")" | "]" | "}" => {
                    depth = depth.checked_sub(1)?;
                    if depth == 0 {
                        return Some(i + 1);
                    }
                }
                _ => {}
            }
            i += 1;
        }
    }

    /// Fails with a message at the node `id`.
    pub(crate) fn error_node(&self, id: NodeId, msg: impl Into<String>) -> Error<'a> {
        let t = self.ast.token(id);
        Error::new(if t.is_eof() { self.eof.0 } else { t.0 }, msg)
    }
}

#[cfg(test)]
mod tests;
