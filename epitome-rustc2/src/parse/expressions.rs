//! Expressions, blocks and statements.
//!
//! Binary operators are parsed by precedence climbing. From loosest to
//! tightest (the Reference's table): assignment and compound assignment
//! (right to left), ranges, `||`, `&&`, comparisons, `|`, `^`, `&`, shifts,
//! `+ -`, `* / %`, `as`, prefix operators, then postfix forms (calls,
//! fields, methods, indexing, `?`, `.await`).
//!
//! Two restrictions change what an expression may be:
//!
//! - **No struct literal** where `{` starts a block: the condition of `if`
//!   and `while`, the scrutinee of `match`, the iterator of `for`. `if x ==
//!   S {}` compares with `S`. Inside brackets the restriction is lifted.
//! - **Statement position:** an expression that starts with a block-like
//!   form (`if`, `match`, a block, a loop, a brace macro call) ends there
//!   unless a `.` or `?` follows, so `if c {} -1` is two statements.

use super::types::PathMode;
use super::{Mark, PResult, Parser};
use crate::ast::Kind;

const PREC_ASSIGN: u8 = 1;
const PREC_RANGE: u8 = 2;
const PREC_OR: u8 = 3;
const PREC_AND: u8 = 4;
const PREC_CAST: u8 = 12;
const PREC_PREFIX: u8 = 13;

/// The precedence of a binary operator, and whether it groups right to
/// left.
fn binary_prec(op: &str) -> Option<(u8, bool)> {
    Some(match op {
        "=" | "+=" | "-=" | "*=" | "/=" | "%=" | "^=" | "&=" | "|=" | "<<=" | ">>=" => (PREC_ASSIGN, true),
        "||" => (PREC_OR, false),
        "&&" => (PREC_AND, false),
        "==" | "!=" | "<" | ">" | "<=" | ">=" => (5, false),
        "|" => (6, false),
        "^" => (7, false),
        "&" => (8, false),
        "<<" | ">>" => (9, false),
        "+" | "-" => (10, false),
        "*" | "/" | "%" => (11, false),
        _ => return None,
    })
}

/// Restrictions on the expression being read.
#[derive(Clone, Copy, Default)]
pub(crate) struct Restrict {
    /// `{` after a path starts a block, not a struct literal.
    pub no_struct: bool,
}

impl<'a> Parser<'a> {
    /// An expression.
    pub(crate) fn expr(&mut self) -> PResult<'a> {
        self.expr_bp(PREC_ASSIGN, Restrict::default())
    }

    /// An expression where a struct literal is not allowed.
    pub(crate) fn expr_no_struct(&mut self) -> PResult<'a> {
        self.expr_bp(PREC_ASSIGN, Restrict { no_struct: true })
    }

    /// The binary operator at the current token: its length in tokens,
    /// precedence, and whether it groups right to left.
    fn binary_op(&self) -> Option<(usize, u8, bool)> {
        let n = self.op_len_n(0);
        let mut buf = [0u8; 3];
        for (k, b) in buf.iter_mut().enumerate().take(n) {
            *b = self.tok(k).0.as_bytes()[0];
        }
        let (prec, right) = binary_prec(std::str::from_utf8(&buf[..n]).ok()?)?;
        Some((n, prec, right))
    }

    /// Whether an expression can start at the current token.
    pub(crate) fn expr_start(&self) -> bool {
        let t = self.tok(0);
        if t.is_eof() {
            return false;
        }
        if self.literal_at(0) || t.is_lifetime() || self.path_start_at(0) {
            return true;
        }
        matches!(
            t.0,
            "(" | "["
                | "{"
                | "|"
                | "!"
                | "-"
                | "*"
                | "&"
                | "#"
                | "_"
                | "if"
                | "match"
                | "loop"
                | "while"
                | "for"
                | "unsafe"
                | "async"
                | "move"
                | "static"
                | "const"
                | "return"
                | "break"
                | "continue"
                | "let"
                | "yield"
                | "become"
                | "do"
                | "try"
                | "gen"
                | "box"
        ) || self.at_op("..")
            || self.at_op("..=")
    }

    /// An expression whose binary operators all bind at least as tightly
    /// as `min`.
    fn expr_bp(&mut self, min: u8, r: Restrict) -> PResult<'a> {
        let m = self.mark();
        if self.at_op("..") || self.at_op("..=") {
            // `..`, `..b`, `..=b`.
            let n = if self.at_op("..") { 2 } else { 3 };
            self.leaf(Kind::Op, n);
            if self.range_end_follows(r) {
                self.expr_bp(PREC_RANGE + 1, r)?;
            }
            self.finish(m, Kind::Range);
        } else {
            self.prefix_expr(r)?;
        }
        self.binary_rest(m, min, r)
    }

    /// Binary operators, casts and ranges after an operand at `m`.
    fn binary_rest(&mut self, m: Mark, min: u8, r: Restrict) -> PResult<'a> {
        loop {
            if self.at("as") && PREC_CAST >= min {
                self.bump();
                self.ty_no_plus()?;
                self.finish(m, Kind::Cast);
                continue;
            }
            if (self.at_op("..") || self.at_op("..=")) && PREC_RANGE >= min {
                let n = if self.at_op("..") { 2 } else { 3 };
                self.leaf(Kind::Op, n);
                if self.range_end_follows(r) {
                    self.expr_bp(PREC_RANGE + 1, r)?;
                }
                self.finish(m, Kind::Range);
                continue;
            }
            let Some((n, prec, right)) = self.binary_op() else { break };
            if prec < min {
                break;
            }
            self.leaf(Kind::Op, n);
            self.expr_bp(if right { prec } else { prec + 1 }, r)?;
            self.finish(m, Kind::Binary);
        }
        Ok(())
    }

    /// Whether the end of a range follows `..`.
    fn range_end_follows(&self, r: Restrict) -> bool {
        self.expr_start() && !(r.no_struct && self.at("{")) && !self.at_op("=>")
    }

    /// Prefix operators, then a primary expression and its postfix forms.
    fn prefix_expr(&mut self, r: Restrict) -> PResult<'a> {
        let m = self.mark();
        match self.tok(0).0 {
            "-" | "!" | "*" => {
                self.leaf(Kind::Op, 1);
                self.expr_bp(PREC_PREFIX, r)?;
                self.finish(m, Kind::Unary);
            }
            "&" => {
                self.bump();
                if self.at("raw") && (self.at_n(1, "const") || self.at_n(1, "mut")) {
                    self.leaf(Kind::Keyword, 1);
                    self.leaf(Kind::Keyword, 1);
                } else if self.at("mut") {
                    self.leaf(Kind::Keyword, 1);
                }
                self.expr_bp(PREC_PREFIX, r)?;
                self.finish(m, Kind::Borrow);
            }
            "#" => {
                self.outer_attributes()?;
                self.expr_bp(PREC_PREFIX, r)?;
                self.finish(m, Kind::AttrExpr);
            }
            _ => {
                self.primary(r)?;
                self.postfix(m)?;
            }
        }
        Ok(())
    }

    /// Calls, method calls, fields, indexing, `?` and `.await` after the
    /// expression at `m`.
    fn postfix(&mut self, m: Mark) -> PResult<'a> {
        loop {
            if self.at("?") {
                self.bump();
                self.finish(m, Kind::Try);
            } else if self.at("(") {
                self.args()?;
                self.finish(m, Kind::Call);
            } else if self.at("[") {
                self.bump();
                self.expr()?;
                self.expect("]")?;
                self.finish(m, Kind::Index);
            } else if self.at(".") && !self.at_prefix_n(0, "..") {
                self.bump();
                if self.at("await") {
                    self.bump();
                    self.finish(m, Kind::Await);
                } else if self.at("use") {
                    self.bump();
                    self.finish(m, Kind::UseExpr);
                } else if self.tok(0).is_number() {
                    self.leaf(Kind::Literal, 1);
                    self.finish(m, Kind::Field);
                } else if self.name_at(0) {
                    let sm = self.mark();
                    self.leaf(Kind::Name, 1);
                    let turbofish = self.at_op("::") && self.at_n(2, "<");
                    if turbofish {
                        self.bump_n(2);
                        self.generic_args()?;
                    }
                    if turbofish || self.at("(") {
                        self.finish(sm, Kind::Segment);
                        self.args()?;
                        self.finish(m, Kind::MethodCall);
                    } else {
                        self.finish(m, Kind::Field);
                    }
                } else {
                    return self.error("expected a field or method name");
                }
            } else {
                return Ok(());
            }
        }
    }

    /// `(a, b)`.
    fn args(&mut self) -> PResult<'a> {
        let m = self.mark();
        self.expect("(")?;
        while !self.at(")") {
            self.expr()?;
            if !self.eat(",") {
                break;
            }
        }
        self.expect(")")?;
        self.finish(m, Kind::Args);
        Ok(())
    }

    fn primary(&mut self, r: Restrict) -> PResult<'a> {
        let m = self.mark();
        let t = self.tok(0);
        if self.literal_at(0) {
            self.leaf(Kind::Literal, 1);
            return Ok(());
        }
        if t.is_lifetime() {
            // A label: `'a: loop {}`.
            return self.labelled(r);
        }
        if self.at("builtin") && self.at_n(1, "#") {
            self.bump_n(2);
            let name = self.tok(0).0;
            self.name()?;
            match name {
                "offset_of" => {
                    self.expect("(")?;
                    self.ty()?;
                    self.expect(",")?;
                    self.field_path()?;
                    self.expect(")")?;
                }
                "type_ascribe" | "wrap_binder" | "unwrap_binder" => {
                    self.expect("(")?;
                    self.expr()?;
                    if self.eat(",") && !self.at(")") {
                        self.ty()?;
                    }
                    self.expect(")")?;
                }
                _ => {
                    self.token_tree()?;
                }
            }
            self.finish(m, Kind::Builtin);
            return Ok(());
        }
        if self.name_at(0) || !self.keyword_at(0) && self.path_start_at(0) {
            // Weak keywords (`union`, `default`, `raw`) and, in older
            // editions, `async`, `try` and `gen` are names here.
            return self.path_expr(m, r);
        }
        match t.0 {
            "(" => {
                self.bump();
                let mut n = 0;
                let mut comma = false;
                while !self.at(")") {
                    self.expr()?;
                    n += 1;
                    comma = self.eat(",");
                    if !comma {
                        break;
                    }
                }
                self.expect(")")?;
                self.finish(m, if n == 1 && !comma { Kind::Paren } else { Kind::Tuple });
            }
            "[" => {
                self.bump();
                let mut kind = Kind::Array;
                if !self.at("]") {
                    self.expr()?;
                    if self.eat(";") {
                        self.expr()?;
                        kind = Kind::ArrayRepeat;
                    } else {
                        while self.eat(",") && !self.at("]") {
                            self.expr()?;
                        }
                    }
                }
                self.expect("]")?;
                self.finish(m, kind);
            }
            "{" => self.block_expr()?,
            "_" => {
                self.bump();
                self.finish(m, Kind::Underscore);
            }
            "if" => self.if_expr()?,
            "match" => self.match_expr()?,
            "loop" | "while" | "for" if !(t.0 == "for" && self.at_n(1, "<")) => self.loop_expr(m)?,
            "unsafe" => self.block_expr()?,
            "const" if self.at_n(1, "{") => {
                self.bump();
                self.block_expr()?;
                self.finish(m, Kind::ConstBlock);
            }
            "try" | "gen" if self.at_n(1, "{") || self.at_n(1, "move") && self.at_n(2, "{") => self.block_expr()?,
            "async" if self.async_block_at() => self.block_expr()?,
            "|" | "move" | "async" | "static" | "for" | "gen" | "use" | "const" => self.closure(r)?,
            "return" | "yield" => {
                self.bump();
                if self.value_follows(r) {
                    self.expr_bp(PREC_ASSIGN, r)?;
                }
                self.finish(m, if t.0 == "return" { Kind::Return } else { Kind::Yield });
            }
            "become" => {
                self.bump();
                self.expr_bp(PREC_ASSIGN, r)?;
                self.finish(m, Kind::Become);
            }
            "do" if self.at_n(1, "yeet") => {
                self.bump_n(2);
                if self.value_follows(r) {
                    self.expr_bp(PREC_ASSIGN, r)?;
                }
                self.finish(m, Kind::Yeet);
            }
            "break" => {
                self.bump();
                if self.lifetime_at(0) {
                    self.leaf(Kind::Lifetime, 1);
                }
                if self.value_follows(r) {
                    self.expr_bp(PREC_ASSIGN, r)?;
                }
                self.finish(m, Kind::Break);
            }
            "continue" => {
                self.bump();
                if self.lifetime_at(0) {
                    self.leaf(Kind::Lifetime, 1);
                }
                self.finish(m, Kind::Continue);
            }
            "let" => {
                self.bump();
                self.pattern()?;
                self.expect("=")?;
                // The value binds tighter than `&&` and `||`, which join
                // `let`s in a condition.
                self.expr_bp(PREC_AND + 1, r)?;
                self.finish(m, Kind::Let);
            }
            "box" if self.expr_start_n(1) => {
                // `box e` is gone from the language; read it as a call-like
                // prefix for old code.
                self.leaf(Kind::Keyword, 1);
                self.expr_bp(PREC_PREFIX, r)?;
                self.finish(m, Kind::Unary);
            }
            _ if self.path_start_at(0) => return self.path_expr(m, r),
            _ => return self.error("expected an expression"),
        }
        Ok(())
    }

    /// A path, a macro call or a struct literal.
    fn path_expr(&mut self, m: Mark, r: Restrict) -> PResult<'a> {
        self.path(PathMode::Expr)?;
        if self.at("!") && !self.at_op("!=") {
            self.bump();
            self.token_tree()?;
            self.finish(m, Kind::MacroCall);
        } else if self.at("{") && !r.no_struct {
            self.struct_fields()?;
            self.finish(m, Kind::StructExpr);
        } else {
            self.finish(m, Kind::PathExpr);
        }
        Ok(())
    }

    fn expr_start_n(&self, n: usize) -> bool {
        let t = self.tok(n);
        !t.is_eof() && (t.is_ident() || t.is_literal() || matches!(t.0, "(" | "[" | "{" | "&" | "*" | "-" | "!"))
    }

    /// Whether `return`, `break` or `yield` has a value.
    fn value_follows(&self, r: Restrict) -> bool {
        self.expr_start() && !(r.no_struct && self.at("{"))
    }

    /// `async {`, `async move {`, `async gen {`.
    fn async_block_at(&self) -> bool {
        let mut n = 1;
        if self.at_n(n, "gen") {
            n += 1;
        }
        if self.at_n(n, "move") {
            n += 1;
        }
        self.at_n(n, "{")
    }

    /// `'a: loop`, `'a: while`, `'a: for`, `'a: {`.
    fn labelled(&mut self, r: Restrict) -> PResult<'a> {
        let m = self.mark();
        if !self.at_n(1, ":") {
            return self.error("expected an expression");
        }
        self.leaf(Kind::Lifetime, 1);
        self.bump();
        match self.tok(0).0 {
            "loop" | "while" | "for" => self.loop_expr(m),
            "{" | "unsafe" => self.block_expr_from(m),
            _ => {
                let _ = r;
                self.error("expected a loop or block after a label")
            }
        }
    }

    /// `{ a, b: e, 0: e, ..base }`.
    fn struct_fields(&mut self) -> PResult<'a> {
        self.expect("{")?;
        while !self.at("}") {
            let m = self.mark();
            if self.at_op("..") {
                self.bump_n(2);
                if !self.at("}") {
                    self.expr()?;
                }
                self.finish(m, Kind::StructBase);
                break;
            }
            self.outer_attributes()?;
            if self.tok(0).is_number() {
                self.leaf(Kind::Literal, 1);
            } else {
                self.name()?;
            }
            if self.eat(":") {
                self.expr()?;
            }
            self.finish(m, Kind::FieldInit);
            if !self.eat(",") {
                break;
            }
        }
        self.expect("}")
    }

    /// `if c { } else if d { } else { }`.
    fn if_expr(&mut self) -> PResult<'a> {
        let m = self.mark();
        self.expect("if")?;
        self.expr_no_struct()?;
        self.block_expr()?;
        if self.eat("else") {
            if self.at("if") {
                self.if_expr()?;
            } else {
                self.block_expr()?;
            }
        }
        self.finish(m, Kind::If);
        Ok(())
    }

    /// `match e { pat if guard => e, ... }`.
    fn match_expr(&mut self) -> PResult<'a> {
        let m = self.mark();
        self.expect("match")?;
        self.expr_no_struct()?;
        self.expect("{")?;
        self.inner_attributes()?;
        while !self.at("}") {
            let am = self.mark();
            self.outer_attributes()?;
            self.pattern()?;
            if self.at("if") {
                let gm = self.mark();
                self.bump();
                self.expr()?;
                self.finish(gm, Kind::Guard);
            }
            if self.eat_op("=>") {
                let block_like = self.stmt_like_expr()?;
                self.finish(am, Kind::MatchArm);
                if !self.eat(",") && !block_like && !self.at("}") {
                    return self.error("expected `,` or `}` after a match arm");
                }
            } else {
                // A never pattern needs no body.
                self.finish(am, Kind::MatchArm);
                if !self.eat(",") && !self.at("}") {
                    return self.error("expected `=>`");
                }
            }
        }
        self.expect("}")?;
        self.finish(m, Kind::Match);
        Ok(())
    }

    /// `loop {}`, `while c {}`, `for p in e {}`, after any label (which is
    /// already under `m`).
    fn loop_expr(&mut self, m: Mark) -> PResult<'a> {
        match self.tok(0).0 {
            "loop" => {
                self.bump();
                self.block_expr()?;
                self.finish(m, Kind::Loop);
            }
            "while" => {
                self.bump();
                self.expr_no_struct()?;
                self.block_expr()?;
                self.finish(m, Kind::While);
            }
            _ => {
                self.expect("for")?;
                if self.at("await") {
                    self.leaf(Kind::Keyword, 1);
                }
                self.pattern()?;
                self.expect("in")?;
                self.expr_no_struct()?;
                self.block_expr()?;
                self.finish(m, Kind::For);
            }
        }
        Ok(())
    }

    /// `move |a, b: T| -> R { }` and its variants.
    fn closure(&mut self, r: Restrict) -> PResult<'a> {
        let m = self.mark();
        self.for_lifetimes()?;
        while matches!(self.tok(0).0, "const" | "static" | "async" | "gen" | "move" | "use") {
            self.leaf(Kind::Keyword, 1);
        }
        let pm = self.mark();
        if self.at_op("||") {
            self.bump_n(2);
        } else {
            self.expect("|")?;
            while !self.at("|") {
                let cm = self.mark();
                self.outer_attributes()?;
                self.pattern_no_alt()?;
                if self.eat(":") {
                    self.ty_no_plus()?;
                }
                self.finish(cm, Kind::ClosureParam);
                if !self.eat(",") {
                    break;
                }
            }
            self.expect("|")?;
        }
        self.finish(pm, Kind::ClosureParams);
        if self.at_op("->") {
            self.ret_type(false)?;
            self.block_expr()?;
        } else {
            self.expr_bp(PREC_ASSIGN, r)?;
        }
        self.finish(m, Kind::Closure);
        Ok(())
    }

    /// A block with any qualifiers and label: `{}`, `unsafe {}`, `async
    /// move {}`, `try {}`, `'a: {}`.
    pub(crate) fn block_expr(&mut self) -> PResult<'a> {
        let m = self.mark();
        self.block_expr_from(m)
    }

    fn block_expr_from(&mut self, m: Mark) -> PResult<'a> {
        while matches!(self.tok(0).0, "unsafe" | "async" | "move" | "try" | "gen") {
            self.leaf(Kind::Keyword, 1);
        }
        self.expect("{")?;
        self.inner_attributes()?;
        while !self.at("}") {
            if self.at_eof() {
                return self.error("expected `}`");
            }
            self.stmt(Some("}"))?;
        }
        self.bump();
        self.finish(m, Kind::Block);
        Ok(())
    }

    /// Whether a macro call with braces starts here: `a::b! {`.
    fn brace_macro_at(&self) -> bool {
        let mut n = 0;
        if self.at_op("::") {
            n += 2;
        }
        loop {
            if !self.segment_at(n) {
                return false;
            }
            n += if self.at_n(n, "$") { 2 } else { 1 };
            if self.at_op_n(n, "::") {
                n += 2;
            } else {
                break;
            }
        }
        self.at_n(n, "!") && self.at_n(n + 1, "{")
    }

    /// Whether the current token starts a block-like expression, which
    /// ends a statement without `;`.
    fn block_like_at(&self) -> bool {
        match self.tok(0).0 {
            "{" | "if" | "match" | "loop" | "while" | "unsafe" => true,
            "for" => !self.at_n(1, "<"),
            "const" => self.at_n(1, "{"),
            "try" | "gen" => self.at_n(1, "{"),
            "async" => self.async_block_at(),
            _ => self.lifetime_at(0) && self.at_n(1, ":"),
        }
    }

    /// An expression in statement position (or a match arm's body). A
    /// block-like expression ends there unless `.` or `?` follows; the
    /// result says whether it did.
    fn stmt_like_expr(&mut self) -> PResult<'a, bool> {
        if self.brace_macro_at() {
            // `m! { ... }` as a statement ends there, like a block.
            let m = self.mark();
            self.path(PathMode::Expr)?;
            self.expect("!")?;
            self.token_tree()?;
            self.finish(m, Kind::MacroCall);
            return Ok(true);
        }
        if !self.block_like_at() {
            self.expr()?;
            return Ok(self.top_is_brace_macro());
        }
        let m = self.mark();
        self.primary(Restrict::default())?;
        if self.at("?") || self.at(".") && !self.at_prefix_n(0, "..") {
            self.postfix(m)?;
            self.binary_rest(m, PREC_ASSIGN, Restrict::default())?;
            return Ok(false);
        }
        Ok(true)
    }

    /// Whether the last node is a macro call written with braces, which
    /// ends a statement like a block.
    fn top_is_brace_macro(&self) -> bool {
        let id = self.top();
        self.ast.kind(id) == Kind::MacroCall && self.ast.tokens_of(id).last().is_some_and(|t| t.0 == "}")
    }

    /// One statement in a block, whose closing token is `close` (or the
    /// end of the input for a fragment).
    pub(crate) fn stmt(&mut self, close: Option<&str>) -> PResult<'a> {
        let m = self.mark();
        if self.eat(";") {
            self.finish(m, Kind::EmptyStmt);
            return Ok(());
        }
        let attrs = self.outer_attributes()?;
        if self.at("let") || self.at("super") && self.at_n(1, "let") {
            if self.at("super") {
                self.leaf(Kind::Keyword, 1);
            }
            self.bump();
            self.pattern()?;
            if self.eat(":") {
                self.ty()?;
            }
            if self.eat("=") {
                self.expr()?;
                if self.eat("else") {
                    self.block_expr()?;
                }
            }
            self.expect(";")?;
            self.finish(m, Kind::LetStmt);
            return Ok(());
        }
        if self.item_start() {
            return self.item_from(m);
        }
        let ends = |p: &Self| close.map_or(p.at_eof(), |c| p.at(c));
        let block_like = self.stmt_like_expr()?;
        if self.eat(";") {
            self.finish(m, Kind::ExprStmt);
        } else if block_like || ends(self) {
            if attrs > 0 {
                self.finish(m, Kind::AttrExpr);
            }
        } else {
            return self.error("expected `;`");
        }
        Ok(())
    }
}
