//! Patterns.

use super::types::PathMode;
use super::{PResult, Parser};
use crate::ast::Kind;

impl<'a> Parser<'a> {
    /// A pattern, which may be `a | b` and may start with `|`.
    pub(crate) fn pattern(&mut self) -> PResult<'a> {
        let m = self.mark();
        if self.at("|") && !self.at_op("||") {
            self.bump();
        }
        self.pattern_no_alt()?;
        let mut alts = 1;
        while self.at("|") && !self.at_op("||") && !self.at_op("|=") {
            self.bump();
            self.pattern_no_alt()?;
            alts += 1;
        }
        if alts > 1 {
            self.finish(m, Kind::OrPat);
        }
        Ok(())
    }

    /// A pattern without top-level `|`: a closure or function parameter.
    pub(crate) fn pattern_no_alt(&mut self) -> PResult<'a> {
        let m = self.mark();
        let t = self.tok(0);
        match t.0 {
            "_" => {
                self.bump();
                self.finish(m, Kind::WildPat);
            }
            "." if self.at_op("..=") || self.at_op("...") => {
                self.leaf(Kind::Op, 3);
                self.range_end()?;
                self.finish(m, Kind::RangePat);
            }
            "." if self.at_op("..") => {
                if self.range_end_at(2) {
                    // `..X`, a half-open range to X.
                    self.leaf(Kind::Op, 2);
                    self.range_end()?;
                    self.finish(m, Kind::RangePat);
                } else {
                    self.bump_n(2);
                    self.finish(m, Kind::RestPat);
                }
            }
            "&" => {
                self.bump();
                if self.at("mut") {
                    self.leaf(Kind::Keyword, 1);
                }
                self.pattern_no_range()?;
                self.finish(m, Kind::RefPat);
            }
            "(" => {
                self.bump();
                let mut n = 0;
                let mut comma = false;
                while !self.at(")") {
                    self.pattern()?;
                    n += 1;
                    comma = self.eat(",");
                    if !comma {
                        break;
                    }
                }
                self.expect(")")?;
                let paren = n == 1 && !comma && self.top_kind() != Kind::RestPat;
                self.finish(m, if paren { Kind::ParenPat } else { Kind::TuplePat });
            }
            "[" => {
                self.bump();
                while !self.at("]") {
                    self.pattern()?;
                    if !self.eat(",") {
                        break;
                    }
                }
                self.expect("]")?;
                self.finish(m, Kind::SlicePat);
            }
            "box" => {
                self.bump();
                self.pattern_no_alt()?;
                self.finish(m, Kind::BoxPat);
            }
            "!" => {
                self.bump();
                self.finish(m, Kind::NeverPat);
            }
            "ref" | "mut" => self.ident_pattern()?,
            "const" if self.at_n(1, "{") => {
                self.bump();
                self.block_expr()?;
                self.finish(m, Kind::ConstBlock);
            }
            _ if self.literal_at(0) || self.at("-") && self.literal_at(1) => {
                self.lit_pattern()?;
                self.maybe_range(m)?;
            }
            _ if self.name_at(0) && !self.at_op_n(1, "::") && !matches!(self.tok(1).0, "(" | "{" | "!") => {
                self.ident_pattern()?;
            }
            _ if self.path_start_at(0) => {
                self.path(PathMode::Expr)?;
                if self.at("!") {
                    self.bump();
                    self.token_tree()?;
                    self.finish(m, Kind::PatMacro);
                } else if self.at("(") {
                    self.bump();
                    while !self.at(")") {
                        self.pattern()?;
                        if !self.eat(",") {
                            break;
                        }
                    }
                    self.expect(")")?;
                    self.finish(m, Kind::TupleStructPat);
                } else if self.at("{") {
                    self.struct_pattern_fields()?;
                    self.finish(m, Kind::StructPat);
                } else {
                    self.finish(m, Kind::PathPat);
                    self.maybe_range(m)?;
                }
            }
            _ => return self.error("expected a pattern"),
        }
        Ok(())
    }

    /// The pattern after `&`, where `&a..=b` is not allowed to mean
    /// `&(a..=b)`.
    fn pattern_no_range(&mut self) -> PResult<'a> {
        self.pattern_no_alt()
    }

    /// `ref mut name @ sub`. A lone name followed by a range operator is a
    /// path (a constant), not a binding.
    fn ident_pattern(&mut self) -> PResult<'a> {
        let m = self.mark();
        if self.at("ref") {
            self.leaf(Kind::Keyword, 1);
        }
        if self.at("mut") {
            self.leaf(Kind::Keyword, 1);
        }
        if self.stack.len() == m.stack && (self.at_op_n(1, "..=") || self.at_op_n(1, "...") || self.at_op_n(1, "..")) {
            // `A..=B` with a constant `A`.
            let pm = self.mark();
            self.path(PathMode::Expr)?;
            self.finish(pm, Kind::PathPat);
            return self.maybe_range(pm);
        }
        self.name()?;
        if self.eat("@") {
            self.pattern_no_alt()?;
        }
        self.finish(m, Kind::IdentPat);
        Ok(())
    }

    fn lit_pattern(&mut self) -> PResult<'a> {
        let m = self.mark();
        if self.at("-") {
            let um = self.mark();
            self.leaf(Kind::Op, 1);
            self.leaf(Kind::Literal, 1);
            self.finish(um, Kind::Unary);
        } else {
            self.leaf(Kind::Literal, 1);
        }
        self.finish(m, Kind::LitPat);
        Ok(())
    }

    /// After a literal or path pattern: `..=`, `...` or `..` and the end.
    fn maybe_range(&mut self, m: super::Mark) -> PResult<'a> {
        if self.at_op("..=") || self.at_op("...") {
            self.leaf(Kind::Op, 3);
            self.range_end()?;
        } else if self.at_op("..") {
            self.leaf(Kind::Op, 2);
            if self.range_end_at(0) {
                self.range_end()?;
            }
        } else {
            return Ok(());
        }
        self.finish(m, Kind::RangePat);
        Ok(())
    }

    /// Whether a range pattern's end starts at token `n`.
    fn range_end_at(&self, n: usize) -> bool {
        self.literal_at(n)
            || self.at_n(n, "-") && self.literal_at(n + 1)
            || self.path_start_at(n)
            || self.at_n(n, "const") && self.at_n(n + 1, "{")
    }

    fn range_end(&mut self) -> PResult<'a> {
        if self.literal_at(0) || self.at("-") {
            self.lit_pattern()
        } else if self.at("const") && self.at_n(1, "{") {
            let m = self.mark();
            self.bump();
            self.block_expr()?;
            self.finish(m, Kind::ConstBlock);
            Ok(())
        } else {
            let m = self.mark();
            self.path(PathMode::Expr)?;
            self.finish(m, Kind::PathPat);
            Ok(())
        }
    }

    /// `{ a, b: p, ref c, 0: p, .. }`.
    fn struct_pattern_fields(&mut self) -> PResult<'a> {
        self.expect("{")?;
        while !self.at("}") {
            let m = self.mark();
            self.outer_attributes()?;
            if self.at_op("..") {
                self.bump_n(2);
                self.finish(m, Kind::RestPat);
                break;
            }
            let named = (self.name_at(0) || self.tok(0).is_number()) && self.at_n(1, ":") && !self.at_op_n(1, "::");
            if named {
                let kind = if self.tok(0).is_number() { Kind::Literal } else { Kind::Name };
                self.leaf(kind, 1);
                self.bump();
                self.pattern()?;
            } else if self.at("box") {
                self.pattern_no_alt()?;
            } else {
                self.ident_pattern()?;
            }
            self.finish(m, Kind::FieldPat);
            if !self.eat(",") {
                break;
            }
        }
        self.expect("}")
    }
}
