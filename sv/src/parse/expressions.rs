//! Expressions (A.8), events and delays (A.6.5).

use super::{PResult, Parser};
use crate::ast::Kind;

/// Binding strengths, weakest first, following table 11-2.
const IMPLY: u8 = 1;
const COND: u8 = 2;
const RELATIONAL: u8 = 9;

/// Keywords that start a data type in places an expression could also go.
pub(crate) const TYPE_KEYWORDS: &[&str] = &[
    "bit",
    "logic",
    "reg",
    "byte",
    "shortint",
    "int",
    "longint",
    "integer",
    "time",
    "shortreal",
    "real",
    "realtime",
    "string",
    "chandle",
    "event",
    "struct",
    "union",
    "enum",
    "virtual",
    "void",
];

/// Assignment operators (11.4.1).
pub(crate) const ASSIGN_OPS: &[&str] =
    &["=", "+=", "-=", "*=", "/=", "%=", "&=", "|=", "^=", "<<=", ">>=", "<<<=", ">>>="];

const UNARY_OPS: &[&str] = &["+", "-", "!", "~", "&", "~&", "|", "~|", "^", "~^", "^~"];

impl<'a> Parser<'a> {
    pub(crate) fn type_keyword_at(&self, n: usize) -> bool {
        TYPE_KEYWORDS.iter().any(|k| self.at_n(n, k))
    }

    pub(crate) fn assign_op_at(&self) -> bool {
        self.at_any(ASSIGN_OPS)
    }

    /// The strength and associativity of the binary operator at the
    /// current token, if it is one.
    fn binary_op(&self) -> Option<(u8, bool)> {
        let t = self.tok(0).0;
        let p = match t {
            "->" | "<->" => return Some((IMPLY, true)),
            "||" => 3,
            "&&" => 4,
            "|" => 5,
            "^" | "~^" | "^~" => 6,
            "&" => 7,
            "==" | "!=" | "===" | "!==" | "==?" | "!=?" => 8,
            "<" | "<=" | ">" | ">=" => RELATIONAL,
            "inside" if self.at("inside") => RELATIONAL,
            "<<" | ">>" | "<<<" | ">>>" => 10,
            "+" | "-" => 11,
            "*" | "/" | "%" => 12,
            "**" => 13,
            _ => return None,
        };
        Some((p, false))
    }

    pub(crate) fn expr(&mut self) -> PResult<'a> {
        self.expr_bp(0)
    }

    /// An expression whose operators bind at least as strongly as `min`.
    /// `expr_bp(COND)` stops before `->`, so a constraint can read
    /// `a -> { ... }`.
    pub(crate) fn expr_bp(&mut self, min: u8) -> PResult<'a> {
        let m = self.mark();
        self.unary()?;
        loop {
            if self.at("?") && min <= COND {
                self.bump();
                self.attributes_opt()?;
                self.expr()?;
                self.expect(":")?;
                self.expr_bp(COND)?;
                self.finish(m, Kind::Conditional);
                continue;
            }
            if self.at("matches") && min <= COND {
                // `e matches pattern` inside a conditional's predicate.
                self.bump();
                self.pattern()?;
                self.finish(m, Kind::CondPattern);
                continue;
            }
            // `*)` ends an attribute.
            if self.at("*") && self.at_n(1, ")") && self.adjacent(0) {
                break;
            }
            let Some((prec, right)) = self.binary_op() else { break };
            if prec < min {
                break;
            }
            if self.at("inside") {
                self.bump();
                self.range_list()?;
                self.finish(m, Kind::Inside);
                continue;
            }
            self.leaf(Kind::Op);
            self.attributes_opt()?;
            self.expr_bp(if right { prec } else { prec + 1 })?;
            self.finish(m, Kind::Binary);
        }
        Ok(())
    }

    /// An expression stopping before `->`, for constraints.
    pub(crate) fn expr_no_imply(&mut self) -> PResult<'a> {
        self.expr_bp(COND)
    }

    pub(crate) fn unary(&mut self) -> PResult<'a> {
        let m = self.mark();
        if self.at_any(UNARY_OPS) {
            self.leaf(Kind::Op);
            self.attributes_opt()?;
            self.unary()?;
            self.finish(m, Kind::Unary);
            return Ok(());
        }
        if self.at_any(&["++", "--"]) {
            self.leaf(Kind::Op);
            self.attributes_opt()?;
            self.postfix()?;
            self.finish(m, Kind::IncDec);
            return Ok(());
        }
        self.postfix()?;
        if self.at_any(&["++", "--"]) {
            self.leaf(Kind::Op);
            self.finish(m, Kind::IncDec);
        }
        Ok(())
    }

    /// Whether `[` here starts a sequence repetition (`[*`, `[=`, `[->`,
    /// `[+]`) rather than a select.
    pub(crate) fn repetition_at(&self) -> bool {
        self.at("[")
            && (self.at_n(1, "*") || self.at_n(1, "=") || self.at_n(1, "->") || self.at_n(1, "+") && self.at_n(2, "]"))
    }

    /// A primary followed by selects, members, calls and casts.
    pub(crate) fn postfix(&mut self) -> PResult<'a> {
        let m = self.mark();
        self.primary()?;
        self.postfix_ops(m)
    }

    pub(crate) fn postfix_ops(&mut self, m: super::Mark) -> PResult<'a> {
        loop {
            let last = self.last_kind();
            if self.at("[") && !self.repetition_at() {
                self.bump();
                self.expr()?;
                if self.at_any(&[":", "+:", "-:"]) {
                    self.leaf(Kind::Op);
                    self.expr()?;
                    self.expect("]")?;
                    self.finish(m, Kind::RangeSelect);
                } else {
                    self.expect("]")?;
                    self.finish(m, Kind::Index);
                }
            } else if self.at(".")
                && self.tok(1).is_ident()
                && !matches!(last, Some(Kind::Number | Kind::String | Kind::TimeLiteral | Kind::UnbasedUnsized))
            {
                self.bump();
                self.leaf(Kind::Name);
                self.finish(m, Kind::Member);
            } else if self.at("::") && self.tok(1).is_ident() {
                self.bump();
                self.leaf(Kind::Name);
                self.finish(m, Kind::Scope);
            } else if self.at("#") && self.at_n(1, "(") && self.skip_balanced(1).is_some_and(|j| self.at_n(j, "::")) {
                self.param_values()?;
                self.finish(m, Kind::Specialize);
            } else if self.attribute_at() && self.adjacent(0) && !self.at_n(2, ")") {
                // Attributes between a function's name and its arguments.
                if !matches!(last, Some(Kind::Name | Kind::Member | Kind::Scope)) {
                    break;
                }
                self.attributes_opt()?;
                if !self.at("(") {
                    return Err(self.expected("`(`"));
                }
                self.args()?;
                self.finish(m, Kind::Call);
            } else if self.at("(")
                && matches!(last, Some(Kind::Name | Kind::Member | Kind::Scope | Kind::SystemName | Kind::Specialize))
            {
                self.args()?;
                self.finish(m, Kind::Call);
            } else if self.at("'") && self.at_n(1, "(") {
                self.bump();
                self.bump();
                self.expr()?;
                self.expect(")")?;
                self.finish(m, Kind::Cast);
            } else if self.at("'{") && matches!(last, Some(Kind::Name | Kind::Scope | Kind::Keyword | Kind::TypeOf)) {
                self.assignment_pattern()?;
                self.finish(m, Kind::AssignmentPattern);
            } else if self.at("with")
                && (self.at_n(1, "(") || self.at_n(1, "{"))
                && matches!(last, Some(Kind::Call | Kind::Member | Kind::Name))
            {
                self.with_clause()?;
                self.finish(m, Kind::With);
            } else {
                break;
            }
        }
        Ok(())
    }

    /// `with (expr)` after an array method, or `with [(names)] {...}`
    /// after `randomize`.
    fn with_clause(&mut self) -> PResult<'a> {
        self.bump();
        if self.at("{") {
            return self.constraint_block();
        }
        let after = self.skip_balanced(0).ok_or_else(|| self.error("unbalanced `(`"))?;
        if self.at_n(after, "{") {
            self.bump();
            while !self.at(")") {
                self.name()?;
                if !self.eat(",") {
                    break;
                }
            }
            self.expect(")")?;
            return self.constraint_block();
        }
        self.bump();
        self.expr()?;
        self.expect(")")
    }

    pub(crate) fn primary(&mut self) -> PResult<'a> {
        let t = self.tok(0);
        if t.is_number() || t.is_base() {
            return self.number();
        }
        if t.is_unbased_unsized() {
            self.leaf(Kind::UnbasedUnsized);
            return Ok(());
        }
        if t.is_string() {
            return self.string_literal();
        }
        if t.is_system() {
            let kind = if t.0 == "$root" || t.0 == "$unit" { Kind::ScopeKeyword } else { Kind::SystemName };
            self.leaf(kind);
            return Ok(());
        }
        match t.0 {
            "$" => {
                self.leaf(Kind::Dollar);
                return Ok(());
            }
            "(" => return self.paren(),
            "{" => return self.concat(),
            "'{" => {
                let m = self.mark();
                self.assignment_pattern()?;
                self.finish(m, Kind::AssignmentPattern);
                return Ok(());
            }
            _ => {}
        }
        if self.name_at(0) {
            self.leaf(Kind::Name);
            return Ok(());
        }
        if self.at("null") {
            self.leaf(Kind::Null);
        } else if self.at("this") {
            self.leaf(Kind::This);
        } else if self.at("super") {
            self.leaf(Kind::Super);
        } else if self.at("local") && self.at_n(1, "::") {
            self.leaf(Kind::ScopeKeyword);
        } else if self.at("new") {
            self.new_expr()?;
        } else if self.at("tagged") {
            self.tagged()?;
        } else if self.at("type") {
            self.type_reference()?;
        } else if self.at("binsof") {
            self.binsof()?;
        } else if (self.type_keyword_at(0) || self.at_any(&["signed", "unsigned", "const"])) && self.at_n(1, "'") {
            // A cast's type: `int'(x)`, `signed'(x)`.
            self.leaf(Kind::Keyword);
        } else if self.type_keyword_at(0) && self.skip_type(0).is_some_and(|i| self.at_n(i, "'")) {
            return Err(
                self.error("a cast's type must be a type keyword or a type name; use a typedef for anything more")
            );
        } else if self.tok(0).is_ident() {
            return Err(self.error(format!("`{}` is a keyword, so it cannot be used as a name", t.0)));
        } else {
            return Err(self.expected("an expression"));
        }
        Ok(())
    }

    /// A number literal: `12`, `8'hFF` (three tokens), `'b1`, `1.5`, `10ns`.
    pub(crate) fn number(&mut self) -> PResult<'a> {
        let m = self.mark();
        let t = self.tok(0);
        if t.is_number() {
            if t.0.ends_with(|c: char| c.is_ascii_alphabetic()) {
                self.leaf(Kind::TimeLiteral);
                return Ok(());
            }
            self.bump();
            if self.tok(0).is_base() {
                if t.0.contains(['.', 'e', 'E']) {
                    return Err(Error::new(t.0, "the size of a literal must be an integer"));
                }
                if t.0.bytes().all(|c| c == b'0' || c == b'_') {
                    return Err(Error::new(t.0, "the size of a literal cannot be zero"));
                }
                self.base_digits()?;
            }
        } else {
            self.base_digits()?;
        }
        self.finish(m, Kind::Number);
        Ok(())
    }

    /// `'h` and the digits after it.
    fn base_digits(&mut self) -> PResult<'a> {
        let base = self.tok(0).base().unwrap_or(b'd');
        self.bump();
        let d = self.tok(0);
        let starts = d.0.bytes().next().is_some_and(|c| c.is_ascii_alphanumeric() || c == b'?');
        if !starts || self.keyword_at(0) {
            return Err(self.expected("the digits of the literal"));
        }
        let xz = |c: u8| b"xXzZ?".contains(&c);
        let ok = |c: u8| match base {
            b'b' => matches!(c, b'0' | b'1' | b'_') || xz(c),
            b'o' => matches!(c, b'0'..=b'7' | b'_') || xz(c),
            b'h' => c.is_ascii_hexdigit() || c == b'_' || xz(c),
            _ => c.is_ascii_digit() || c == b'_',
        };
        let single_xz = base == b'd' && xz(d.0.as_bytes()[0]) && d.0[1..].bytes().all(|c| c == b'_');
        if !single_xz && !d.0.bytes().all(ok) {
            return Err(self.error(format!("`{}` is not made of digits of base '{}", d.0, base as char)));
        }
        self.bump();
        Ok(())
    }

    /// A string literal, with its escapes checked (5.9.1).
    fn string_literal(&mut self) -> PResult<'a> {
        let t = self.tok(0).0;
        let b = t.as_bytes();
        let mut i = 1;
        while i < b.len() {
            if b[i] == b'\\' && i + 1 < b.len() {
                let xz = |c: u8| b"xXzZ?".contains(&c);
                if b[i + 1] == b'x' {
                    let digits: Vec<u8> =
                        b[i + 2..].iter().copied().take_while(|&c| c.is_ascii_hexdigit() || xz(c)).take(2).collect();
                    if digits.is_empty() || digits.iter().any(|&c| xz(c)) {
                        return Err(self.error("a `\\x` escape needs one or two hexadecimal digits"));
                    }
                    i += 2 + digits.len();
                    continue;
                }
                if b[i + 1].is_ascii_digit() && b[i + 1] < b'8' {
                    let digits: Vec<u8> = b[i + 1..]
                        .iter()
                        .copied()
                        .take_while(|&c| (b'0'..=b'7').contains(&c) || xz(c))
                        .take(3)
                        .collect();
                    if digits.iter().any(|&c| xz(c)) {
                        return Err(self.error("an octal escape can only use the digits 0 to 7"));
                    }
                    i += 1 + digits.len();
                    continue;
                }
                i += 2;
                continue;
            }
            i += 1;
        }
        self.leaf(Kind::String);
        Ok(())
    }

    /// `( e )`, `( a : b : c )` or `( a = b )`.
    fn paren(&mut self) -> PResult<'a> {
        let m = self.mark();
        self.bump();
        let inner = self.mark();
        self.expr()?;
        if self.at(":") {
            self.bump();
            self.expr()?;
            self.expect(":")?;
            self.expr()?;
            self.finish(inner, Kind::MinTypMax);
        } else if self.assign_op_at() {
            self.leaf(Kind::Op);
            self.expr()?;
            self.finish(inner, Kind::Assign);
            self.expect(")")?;
            self.finish(m, Kind::AssignExpr);
            return Ok(());
        }
        self.expect(")")?;
        self.finish(m, Kind::Paren);
        Ok(())
    }

    /// `{a, b}`, `{n{a}}`, `{<< 8 {a}}` or `{}`.
    pub(crate) fn concat(&mut self) -> PResult<'a> {
        let m = self.mark();
        self.expect("{")?;
        if self.eat("}") {
            self.finish(m, Kind::Concat);
            return Ok(());
        }
        if self.at_any(&["<<", ">>"]) {
            self.leaf(Kind::Op);
            if !self.at("{") {
                if self.type_keyword_at(0) {
                    self.type_expr()?;
                } else {
                    self.expr()?;
                }
            }
            self.expect("{")?;
            loop {
                let s = self.mark();
                self.expr()?;
                if self.at("with") && self.at_n(1, "[") {
                    self.bump();
                    self.bump();
                    let r = self.mark();
                    self.expr()?;
                    if self.at_any(&[":", "+:", "-:"]) {
                        self.leaf(Kind::Op);
                        self.expr()?;
                        self.finish(r, Kind::RangeSelect);
                    }
                    self.expect("]")?;
                }
                self.finish(s, Kind::StreamExpr);
                if !self.eat(",") {
                    break;
                }
            }
            self.expect("}")?;
            self.expect("}")?;
            self.finish(m, Kind::Streaming);
            return Ok(());
        }
        self.expr()?;
        if self.at("{") {
            self.concat()?;
            self.expect("}")?;
            self.finish(m, Kind::Replicate);
            return Ok(());
        }
        while self.eat(",") {
            self.expr()?;
        }
        self.expect("}")?;
        self.finish(m, Kind::Concat);
        Ok(())
    }

    /// The body of `'{ ... }`, after any type.
    pub(crate) fn assignment_pattern(&mut self) -> PResult<'a> {
        self.expect("'{")?;
        if self.eat("}") {
            return Ok(());
        }
        let first = self.mark();
        self.pattern_key_or_value()?;
        if self.at("{") {
            // `'{n{a, b}}`
            self.concat()?;
            self.finish(first, Kind::Replicate);
            return self.expect("}");
        }
        self.pattern_value(first)?;
        while self.eat(",") {
            let m = self.mark();
            self.pattern_key_or_value()?;
            self.pattern_value(m)?;
        }
        self.expect("}")
    }

    fn pattern_key_or_value(&mut self) -> PResult<'a> {
        if self.at("default") {
            self.leaf(Kind::Keyword);
        } else if self.type_keyword_at(0) && self.at_n(1, ":") {
            self.type_expr()?;
        } else {
            self.expr()?;
        }
        Ok(())
    }

    fn pattern_value(&mut self, m: super::Mark) -> PResult<'a> {
        if self.eat(":") {
            self.expr()?;
            self.finish(m, Kind::PatternKey);
        } else if self.last_kind() == Some(Kind::Keyword) {
            return Err(self.expected("`:`"));
        }
        Ok(())
    }

    /// `new`, `new(args)`, `new[n]`, `new[n](init)`, `new other`.
    fn new_expr(&mut self) -> PResult<'a> {
        let m = self.mark();
        self.bump();
        if self.at("[") {
            self.bump();
            self.expr()?;
            self.expect("]")?;
            if self.at("(") {
                self.bump();
                self.expr()?;
                self.expect(")")?;
            }
        } else if self.at("(") {
            self.args()?;
        } else if self.name_at(0) || self.at("this") {
            self.postfix()?;
        }
        self.finish(m, Kind::New);
        Ok(())
    }

    /// `tagged Member [value]`.
    fn tagged(&mut self) -> PResult<'a> {
        let m = self.mark();
        self.bump();
        self.name()?;
        if self.primary_start_at(0) {
            self.postfix()?;
        }
        self.finish(m, Kind::Tagged);
        Ok(())
    }

    /// Whether token `n` can start a primary.
    pub(crate) fn primary_start_at(&self, n: usize) -> bool {
        let t = self.tok(n);
        t.is_number()
            || t.is_base()
            || t.is_string()
            || t.is_unbased_unsized()
            || t.is_system()
            || self.name_at(n)
            || matches!(t.0, "(" | "{" | "'{")
            || self.at_n(n, "null")
            || self.at_n(n, "this")
    }

    /// `type(expr)` or `type(data_type)`.
    pub(crate) fn type_reference(&mut self) -> PResult<'a> {
        let m = self.mark();
        self.bump();
        self.expect("(")?;
        let keyword_type = self.type_keyword_at(0) || self.at_any(&["signed", "unsigned"]);
        if keyword_type
            || !self.attempt(|p| {
                p.expr()?;
                if p.at(")") { Ok(()) } else { Err(p.expected("`)`")) }
            })
        {
            self.data_type(false)?;
        }
        self.expect(")")?;
        self.finish(m, Kind::TypeOf);
        Ok(())
    }

    /// `binsof(x.y) [intersect {...}]`.
    fn binsof(&mut self) -> PResult<'a> {
        let m = self.mark();
        self.bump();
        self.expect("(")?;
        self.postfix()?;
        self.expect(")")?;
        if self.eat("intersect") {
            self.range_list()?;
        }
        self.finish(m, Kind::Binsof);
        Ok(())
    }

    /// A data type where an expression could go.
    pub(crate) fn type_expr(&mut self) -> PResult<'a> {
        let m = self.mark();
        self.data_type(false)?;
        self.finish(m, Kind::TypeExpr);
        Ok(())
    }

    /// `{ value, [a:b], ... }`.
    pub(crate) fn range_list(&mut self) -> PResult<'a> {
        let m = self.mark();
        self.expect("{")?;
        loop {
            self.value_range()?;
            if !self.eat(",") {
                break;
            }
        }
        self.expect("}")?;
        self.finish(m, Kind::RangeList);
        Ok(())
    }

    /// A value, `[a:b]`, `[$:b]`, `[a +/- b]` or `[a +%- b]`.
    pub(crate) fn value_range(&mut self) -> PResult<'a> {
        if !self.at("[") {
            return self.expr();
        }
        let m = self.mark();
        self.bump();
        self.expr_or_dollar()?;
        if self.at(":") {
            self.bump();
            self.expr_or_dollar()?;
        } else if self.at("+") && (self.at_n(1, "/") || self.at_n(1, "%")) && self.at_n(2, "-") {
            let o = self.mark();
            self.bump();
            self.bump();
            self.bump();
            self.finish(o, Kind::Op);
            self.expr()?;
        } else {
            return Err(self.expected("`:`, `+/-` or `+%-`"));
        }
        self.expect("]")?;
        self.finish(m, Kind::ValueRange);
        Ok(())
    }

    pub(crate) fn expr_or_dollar(&mut self) -> PResult<'a> {
        if self.at("$") && (self.at_n(1, "]") || self.at_n(1, ":") || self.at_n(1, ")")) {
            self.leaf(Kind::Dollar);
            Ok(())
        } else {
            self.expr()
        }
    }

    /// `( args )` of a call: positional, then named.
    pub(crate) fn args(&mut self) -> PResult<'a> {
        let m = self.mark();
        self.expect("(")?;
        if self.eat(")") {
            self.finish(m, Kind::Args);
            return Ok(());
        }
        let mut named = false;
        loop {
            if self.at(".") && self.tok(1).is_ident() {
                let n = self.mark();
                self.bump();
                self.leaf(Kind::Name);
                self.expect("(")?;
                if !self.at(")") {
                    self.arg_value()?;
                }
                self.expect(")")?;
                self.finish(n, Kind::NamedArg);
                named = true;
            } else if named {
                return Err(self.error("a positional argument cannot follow a named one"));
            } else if self.at(",") || self.at(")") {
                self.empty();
            } else if self.at("default") && (self.at_n(1, ")") || self.at_n(1, ",")) {
                self.leaf(Kind::Keyword);
            } else {
                self.arg_value()?;
            }
            if !self.eat(",") {
                break;
            }
        }
        self.expect(")")?;
        self.finish(m, Kind::Args);
        Ok(())
    }

    /// One argument: an expression, a data type (`$bits(int)`), a clocking
    /// event (`$rose(a, @(posedge c))`) or a sequence.
    pub(crate) fn arg_value(&mut self) -> PResult<'a> {
        if self.at("@") {
            return self.event_control();
        }
        if self.at_any(&["posedge", "negedge", "edge"]) {
            return self.event_expr();
        }
        if (self.type_keyword_at(0) || self.at_any(&["signed", "unsigned"])) && !self.at_n(1, "'") {
            return self.type_expr();
        }
        let s = self.save();
        self.expr()?;
        if self.at(",") || self.at(")") {
            return Ok(());
        }
        // A sequence or property as an argument.
        self.restore(s);
        self.property_expr()
    }

    // ---- events and delays ----

    /// `@e`, `@(e)`, `@*`, `@(*)`.
    pub(crate) fn event_control(&mut self) -> PResult<'a> {
        let m = self.mark();
        self.expect("@")?;
        if self.eat("*") {
        } else if self.at("(") && self.at_n(1, "*") && self.at_n(2, ")") {
            self.bump();
            self.bump();
            self.bump();
        } else if self.at("(") {
            self.bump();
            self.event_expr()?;
            self.expect(")")?;
        } else {
            self.postfix()?;
        }
        self.finish(m, Kind::Event);
        Ok(())
    }

    /// Terms joined by `or` or `,`.
    pub(crate) fn event_expr(&mut self) -> PResult<'a> {
        let m = self.mark();
        self.event_term()?;
        let mut many = false;
        while self.at("or") || self.at(",") {
            self.bump();
            self.event_term()?;
            many = true;
        }
        if many {
            self.finish(m, Kind::EventOr);
        }
        Ok(())
    }

    fn event_term(&mut self) -> PResult<'a> {
        let m = self.mark();
        // `@((a or b))`: a parenthesised event expression, unless the
        // parentheses are part of an expression (`@((a + b) == c)`).
        if self.at("(")
            && self.attempt(|p| {
                p.bump();
                p.event_expr()?;
                p.expect(")")?;
                if p.at_any(&["or", ",", ")", "iff"]) { Ok(()) } else { Err(p.expected("`or`")) }
            })
        {
            self.finish(m, Kind::Paren);
        } else {
            self.opt_keyword(&["posedge", "negedge", "edge"]);
            self.expr()?;
        }
        if self.at("iff") {
            let i = self.mark();
            self.bump();
            self.expr()?;
            self.finish(i, Kind::Iff);
        }
        self.finish(m, Kind::EventTerm);
        Ok(())
    }

    /// `#5`, `#1ns`, `#(1:2:3, 4)`, `#d`, `#pkg::d`.
    pub(crate) fn delay(&mut self) -> PResult<'a> {
        let m = self.mark();
        self.expect("#")?;
        if self.at("(") {
            self.bump();
            loop {
                self.mintypmax()?;
                if !self.eat(",") {
                    break;
                }
            }
            self.expect(")")?;
        } else if self.tok(0).is_number() {
            self.number()?;
        } else if self.name_at(0) || self.tok(0).is_system() {
            let n = self.mark();
            self.primary()?;
            while self.at("::") || self.at(".") {
                self.bump();
                self.name()?;
                self.finish(n, Kind::Scope);
            }
        } else {
            return Err(self.expected("a delay value"));
        }
        self.finish(m, Kind::Delay);
        Ok(())
    }

    /// `e` or `a:b:c`.
    pub(crate) fn mintypmax(&mut self) -> PResult<'a> {
        let m = self.mark();
        self.expr()?;
        if self.at(":") {
            self.bump();
            self.expr()?;
            self.expect(":")?;
            self.expr()?;
            self.finish(m, Kind::MinTypMax);
        }
        Ok(())
    }

    /// `##1`, `##n`, `##(e)`, `##[1:3]`, `##[*]`, `##[+]`.
    pub(crate) fn cycle_delay(&mut self) -> PResult<'a> {
        let m = self.mark();
        self.expect("##")?;
        if self.at("[") {
            self.bump();
            if (self.at("*") || self.at("+")) && self.at_n(1, "]") {
                self.leaf(Kind::Op);
            } else {
                self.expr()?;
                if self.eat(":") {
                    self.expr_or_dollar()?;
                }
            }
            self.expect("]")?;
        } else if self.at("(") {
            self.bump();
            self.expr()?;
            self.expect(")")?;
        } else if self.tok(0).is_number() {
            self.number()?;
        } else if self.name_at(0) {
            self.leaf(Kind::Name);
        } else {
            return Err(self.expected("a cycle delay"));
        }
        self.finish(m, Kind::CycleDelay);
        Ok(())
    }

    /// `#d`, `@e`, `repeat (n) @e`: the control in `a = #1 b`.
    pub(crate) fn delay_or_event(&mut self) -> PResult<'a> {
        if self.at("#") {
            self.delay()
        } else if self.at("@") {
            self.event_control()
        } else if self.at("repeat") {
            let m = self.mark();
            self.bump();
            self.expect("(")?;
            self.expr()?;
            self.expect(")")?;
            self.event_control()?;
            self.finish(m, Kind::RepeatEvent);
            Ok(())
        } else {
            Ok(())
        }
    }

    /// Whether `(*` starts an attribute here (and not `@(*)`).
    pub(crate) fn attribute_at(&self) -> bool {
        self.at("(") && self.at_n(1, "*") && self.adjacent(0) && !self.at_n(2, ")")
    }

    /// Zero or more `(* ... *)`.
    pub(crate) fn attributes_opt(&mut self) -> PResult<'a> {
        while self.attribute_at() {
            if self.in_attribute {
                return Err(self.error("an attribute cannot contain another attribute"));
            }
            self.in_attribute = true;
            let r = self.attribute();
            self.in_attribute = false;
            r?;
        }
        Ok(())
    }

    fn attribute(&mut self) -> PResult<'a> {
        {
            let m = self.mark();
            self.bump();
            self.bump();
            loop {
                let s = self.mark();
                self.name()?;
                if self.eat("=") {
                    self.expr()?;
                }
                self.finish(s, Kind::AttrSpec);
                if !self.eat(",") {
                    break;
                }
            }
            if !(self.at("*") && self.at_n(1, ")") && self.adjacent(0)) {
                return Err(self.expected("`*)`"));
            }
            self.bump();
            self.bump();
            self.finish(m, Kind::Attributes);
        }
        Ok(())
    }
}

use crate::source::Error;
