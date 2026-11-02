//! Properties and sequences (A.2.10), constraints (A.1.10) and covergroups
//! (A.2.11).

use super::{Mark, PResult, Parser};
use crate::ast::Kind;

impl<'a> Parser<'a> {
    // ---- properties and sequences ----

    /// `[@(event)] [disable iff (e)] property_expr`.
    pub(crate) fn property_spec(&mut self) -> PResult<'a> {
        let m = self.mark();
        if self.at("@") {
            self.event_control()?;
        }
        if self.at("disable") && self.at_n(1, "iff") {
            let d = self.mark();
            self.bump();
            self.bump();
            self.expect("(")?;
            self.expr_or_dist()?;
            self.expect(")")?;
            self.finish(d, Kind::DisableIff);
        }
        self.property_expr()?;
        self.finish(m, Kind::PropertySpec);
        Ok(())
    }

    pub(crate) fn property_expr(&mut self) -> PResult<'a> {
        self.prop_bp(0)
    }

    /// The strength and associativity of a sequence or property operator
    /// (table 16-3), if one is here.
    fn prop_op(&self) -> Option<(u8, bool)> {
        let t = self.tok(0).0;
        Some(match t {
            "|->" | "|=>" | "#-#" | "#=#" => (2, true),
            "until" | "s_until" | "until_with" | "s_until_with" | "implies" if self.keyword_at(0) => (3, true),
            "iff" if self.keyword_at(0) => (4, true),
            "or" if self.keyword_at(0) => (5, false),
            "and" if self.keyword_at(0) => (6, false),
            "intersect" if self.keyword_at(0) => (8, false),
            "within" if self.keyword_at(0) => (9, false),
            "throughout" if self.keyword_at(0) => (10, true),
            "##" => (11, false),
            _ => return None,
        })
    }

    fn prop_bp(&mut self, min: u8) -> PResult<'a> {
        let m = self.mark();
        self.prop_prefix()?;
        while let Some((prec, right)) = self.prop_op() {
            if prec < min {
                break;
            }
            if self.at("##") {
                self.cycle_delay()?;
                self.prop_bp(12)?;
                self.finish(m, Kind::SeqDelay);
                continue;
            }
            let kind = if self.keyword_at(0) { Kind::Keyword } else { Kind::Op };
            self.leaf(kind);
            self.prop_bp(if right { prec } else { prec + 1 })?;
            self.finish(m, Kind::PropBinary);
        }
        Ok(())
    }

    /// An operand with any prefix operators and repetitions.
    fn prop_prefix(&mut self) -> PResult<'a> {
        let m = self.mark();
        let t = self.tok(0).0;
        let kw = self.keyword_at(0);
        match t {
            "##" => {
                self.cycle_delay()?;
                self.prop_bp(12)?;
                self.finish(m, Kind::SeqDelay);
                return Ok(());
            }
            "not" if kw => {
                self.leaf(Kind::Keyword);
                self.prop_bp(7)?;
                self.finish(m, Kind::PropUnary);
                return Ok(());
            }
            "nexttime" | "s_nexttime" if kw => {
                self.leaf(Kind::Keyword);
                if self.at("[") {
                    self.bump();
                    self.expr()?;
                    self.expect("]")?;
                }
                self.prop_bp(7)?;
                self.finish(m, Kind::PropUnary);
                return Ok(());
            }
            "always" | "s_always" | "eventually" | "s_eventually" if kw => {
                self.leaf(Kind::Keyword);
                if self.at("[") {
                    let r = self.mark();
                    self.bump();
                    self.expr()?;
                    if self.eat(":") {
                        self.expr_or_dollar()?;
                    }
                    self.expect("]")?;
                    self.finish(r, Kind::RangeDim);
                }
                self.prop_bp(0)?;
                self.finish(m, Kind::PropUnary);
                return Ok(());
            }
            "accept_on" | "reject_on" | "sync_accept_on" | "sync_reject_on" if kw => {
                self.leaf(Kind::Keyword);
                self.expect("(")?;
                self.expr_or_dist()?;
                self.expect(")")?;
                self.prop_bp(0)?;
                self.finish(m, Kind::PropUnary);
                return Ok(());
            }
            "if" if kw => {
                self.bump();
                self.expect("(")?;
                self.expr_or_dist()?;
                self.expect(")")?;
                self.prop_bp(0)?;
                if self.eat("else") {
                    self.prop_bp(0)?;
                }
                self.finish(m, Kind::PropIf);
                return Ok(());
            }
            "case" if kw => {
                self.bump();
                self.expect("(")?;
                self.expr_or_dist()?;
                self.expect(")")?;
                while !self.at("endcase") {
                    let i = self.mark();
                    if self.eat("default") {
                        self.eat(":");
                    } else {
                        loop {
                            self.expr_or_dist()?;
                            if !self.eat(",") {
                                break;
                            }
                        }
                        self.expect(":")?;
                    }
                    self.property_expr()?;
                    self.expect(";")?;
                    self.finish(i, Kind::PropCaseItem);
                    if self.at_eof() {
                        return Err(self.expected("`endcase`"));
                    }
                }
                self.bump();
                self.finish(m, Kind::PropCase);
                return Ok(());
            }
            "strong" | "weak" | "first_match" if kw => {
                self.leaf(Kind::Keyword);
                self.expect("(")?;
                self.property_expr()?;
                while self.eat(",") {
                    self.match_item()?;
                }
                self.expect(")")?;
                self.finish(m, Kind::PropCall);
            }
            "@" => {
                self.event_control()?;
                self.prop_bp(0)?;
                self.finish(m, Kind::PropClocked);
                return Ok(());
            }
            "(" => {
                let expression = self.attempt(|p| {
                    p.expr_or_dist()?;
                    if p.at_any(&[")", ";", ",", "##", "[", "|->", "|=>", "#-#", "#=#"])
                        || p.prop_op().is_some()
                        || p.at("else")
                        || p.at_eof()
                    {
                        Ok(())
                    } else {
                        Err(p.expected("a sequence operator"))
                    }
                });
                if !expression {
                    self.bump();
                    self.property_expr()?;
                    while self.eat(",") {
                        self.match_item()?;
                    }
                    self.expect(")")?;
                    self.finish(m, Kind::SeqParen);
                }
            }
            _ => self.expr_or_dist()?,
        }
        while self.repetition_at() {
            self.bump();
            if self.at("+") || self.at("*") && self.at_n(1, "]") {
                self.leaf(Kind::Op);
            } else {
                self.leaf(Kind::Op);
                self.expr()?;
                if self.eat(":") {
                    self.expr_or_dollar()?;
                }
            }
            self.expect("]")?;
            self.finish(m, Kind::SeqRepeat);
        }
        Ok(())
    }

    /// An assignment, increment or call inside `(seq, ...)`.
    fn match_item(&mut self) -> PResult<'a> {
        let m = self.mark();
        self.unary()?;
        if self.assign_op_at() {
            self.leaf(Kind::Op);
            self.expr()?;
            self.finish(m, Kind::Assign);
        }
        Ok(())
    }

    /// `e` or `e dist { ... }`.
    pub(crate) fn expr_or_dist(&mut self) -> PResult<'a> {
        let m = self.mark();
        self.expr()?;
        if self.at("dist") {
            self.dist(m)?;
        }
        Ok(())
    }

    fn dist(&mut self, m: Mark) -> PResult<'a> {
        self.bump();
        self.expect("{")?;
        loop {
            let i = self.mark();
            if self.at("default") {
                self.leaf(Kind::Keyword);
                self.expect(":/")?;
                self.expr()?;
            } else {
                self.value_range()?;
                if self.at(":=") || self.at(":/") {
                    self.leaf(Kind::Op);
                    self.expr()?;
                }
            }
            self.finish(i, Kind::DistItem);
            if !self.eat(",") {
                break;
            }
        }
        self.expect("}")?;
        self.finish(m, Kind::Dist);
        Ok(())
    }

    /// `property name(ports); ... endproperty` or the same for `sequence`.
    pub(crate) fn property_decl(&mut self) -> PResult<'a> {
        let m = self.mark();
        let is_property = self.at("property");
        self.bump();
        self.name()?;
        if self.eat("(") {
            while !self.at(")") {
                self.prop_port()?;
                if !self.eat(",") {
                    break;
                }
            }
            self.expect(")")?;
        }
        self.expect(";")?;
        while self.var_decl_at() {
            self.data_declaration()?;
        }
        self.property_spec()?;
        self.eat(";");
        let end = if is_property { "endproperty" } else { "endsequence" };
        self.expect(end)?;
        self.end_label()?;
        self.finish(m, if is_property { Kind::PropertyDecl } else { Kind::SequenceDecl });
        Ok(())
    }

    /// A formal argument of a property, sequence or checker.
    pub(crate) fn prop_port(&mut self) -> PResult<'a> {
        let m = self.mark();
        self.attributes_opt()?;
        if self.opt_keyword(&["local"]) {
            self.opt_keyword(&["input", "inout", "output"]);
        } else {
            self.opt_keyword(&["input", "output"]);
        }
        if self.at_any(&["sequence", "property", "untyped"]) {
            self.leaf(Kind::Keyword);
        } else if self.name_at(0) && matches!(self.tok(1).0, "," | ")" | "=" | "[") {
            self.empty();
        } else {
            self.data_type(true)?;
        }
        self.name()?;
        while self.at("[") {
            self.variable_dim()?;
        }
        if self.eat("=") {
            if self.at("$") && (self.at_n(1, ",") || self.at_n(1, ")")) {
                self.leaf(Kind::Dollar);
            } else if self.at("@") {
                self.event_control()?;
            } else {
                self.property_expr()?;
            }
        }
        self.finish(m, Kind::PropPort);
        Ok(())
    }

    // ---- constraints ----

    /// `constraint name { ... }` or a prototype `constraint name;`. `m`
    /// marks any qualifiers.
    pub(crate) fn constraint_decl(&mut self, m: Mark) -> PResult<'a> {
        self.expect("constraint")?;
        self.dynamic_override()?;
        let n = self.mark();
        self.name()?;
        if self.at("::") {
            self.bump();
            self.name()?;
            self.finish(n, Kind::Scope);
        }
        if self.at("{") {
            self.constraint_block()?;
        } else {
            self.expect(";")?;
        }
        self.finish(m, Kind::Constraint);
        Ok(())
    }

    pub(crate) fn constraint_block(&mut self) -> PResult<'a> {
        let m = self.mark();
        self.expect("{")?;
        while !self.at("}") {
            if self.at_eof() {
                return Err(self.expected("`}`"));
            }
            self.constraint_item()?;
        }
        self.bump();
        self.finish(m, Kind::ConstraintBlock);
        Ok(())
    }

    fn constraint_set(&mut self) -> PResult<'a> {
        if self.at("{") { self.constraint_block() } else { self.constraint_item() }
    }

    fn constraint_item(&mut self) -> PResult<'a> {
        let m = self.mark();
        if self.eat("solve") {
            loop {
                self.postfix()?;
                if !self.eat(",") {
                    break;
                }
            }
            self.expect("before")?;
            loop {
                self.postfix()?;
                if !self.eat(",") {
                    break;
                }
            }
            self.expect(";")?;
            self.finish(m, Kind::Solve);
        } else if self.eat("if") {
            self.expect("(")?;
            self.expr()?;
            self.expect(")")?;
            self.constraint_set()?;
            if self.eat("else") {
                self.constraint_set()?;
            }
            self.finish(m, Kind::ConstraintIf);
        } else if self.at("foreach") {
            self.foreach_header()?;
            self.constraint_set()?;
            self.finish(m, Kind::ConstraintForeach);
        } else if self.at("unique") && self.at_n(1, "{") {
            self.bump();
            self.range_list()?;
            self.expect(";")?;
            self.finish(m, Kind::Uniqueness);
        } else if self.at("disable") && self.at_n(1, "soft") {
            self.bump();
            self.bump();
            self.postfix()?;
            self.expect(";")?;
            self.finish(m, Kind::DisableSoft);
        } else {
            let soft = self.opt_keyword(&["soft"]);
            let e = self.mark();
            self.expr_no_imply()?;
            if self.at("->") && !soft {
                self.bump();
                self.constraint_set()?;
                self.finish(m, Kind::ConstraintImplies);
                return Ok(());
            }
            if self.at("->") {
                // `soft a -> b;` is an implication expression.
                self.leaf(Kind::Op);
                self.expr()?;
                self.finish(e, Kind::Binary);
            }
            if self.at("dist") {
                self.dist(e)?;
            }
            self.expect(";")?;
            self.finish(m, Kind::ConstraintExpr);
        }
        Ok(())
    }

    // ---- covergroups ----

    /// `covergroup name (ports) @(event); ... endgroup`.
    pub(crate) fn covergroup(&mut self) -> PResult<'a> {
        let m = self.mark();
        self.bump();
        if self.eat("extends") {
            self.name()?;
        } else {
            self.name()?;
            if self.at("(") {
                self.tf_ports()?;
            }
            if self.at("@") {
                self.event_control()?;
            } else if self.at("@@") {
                let e = self.mark();
                self.bump();
                self.expect("(")?;
                loop {
                    let t = self.mark();
                    self.opt_keyword(&["begin", "end"]);
                    self.postfix()?;
                    self.finish(t, Kind::EventTerm);
                    if !self.eat("or") {
                        break;
                    }
                }
                self.expect(")")?;
                self.finish(e, Kind::Event);
            } else if self.at("with") {
                self.bump();
                self.expect("function")?;
                if self.tok(0).0 != "sample" {
                    return Err(self.expected("`sample`"));
                }
                self.leaf(Kind::Name);
                self.tf_ports()?;
            }
        }
        self.expect(";")?;
        while !self.at("endgroup") {
            if self.at_eof() {
                return Err(self.expected("`endgroup`"));
            }
            self.coverage_item()?;
        }
        self.bump();
        self.end_label()?;
        self.finish(m, Kind::Covergroup);
        Ok(())
    }

    fn coverage_option_at(&self) -> bool {
        (self.tok(0).0 == "option" || self.tok(0).0 == "type_option") && self.at_n(1, ".")
    }

    fn coverage_option(&mut self) -> PResult<'a> {
        let m = self.mark();
        self.leaf(Kind::Name);
        self.bump();
        self.name()?;
        self.expect("=")?;
        self.expr()?;
        self.finish(m, Kind::CoverageOption);
        Ok(())
    }

    fn coverage_item(&mut self) -> PResult<'a> {
        let m = self.mark();
        self.attributes_opt()?;
        if self.coverage_option_at() {
            self.coverage_option()?;
            return self.expect(";");
        }
        // `[type] [label :] coverpoint` or `[label :] cross`
        if !self.at("coverpoint") && !self.at("cross") {
            if !(self.name_at(0) && self.at_n(1, ":")) {
                self.data_type(false)?;
            }
            let l = self.mark();
            self.name()?;
            self.expect(":")?;
            self.finish(l, Kind::Label);
        }
        if self.eat("coverpoint") {
            self.expr()?;
            self.iff_opt()?;
            self.bins_or_empty(false)?;
            self.finish(m, Kind::Coverpoint);
        } else if self.eat("cross") {
            loop {
                self.postfix()?;
                if !self.eat(",") {
                    break;
                }
            }
            self.iff_opt()?;
            self.bins_or_empty(true)?;
            self.finish(m, Kind::Cross);
        } else {
            return Err(self.expected("`coverpoint` or `cross`"));
        }
        Ok(())
    }

    /// `{ bins and options }` or `;`.
    fn bins_or_empty(&mut self, cross: bool) -> PResult<'a> {
        if self.eat(";") {
            return Ok(());
        }
        self.expect("{")?;
        while !self.at("}") {
            if self.at_eof() {
                return Err(self.expected("`}`"));
            }
            let s = self.mark();
            self.attributes_opt()?;
            if self.coverage_option_at() {
                self.coverage_option()?;
            } else if cross && self.at("function") {
                self.subroutine(s, false)?;
                continue;
            } else {
                self.bins(cross)?;
            }
            self.expect(";")?;
        }
        self.bump();
        Ok(())
    }

    fn bins(&mut self, cross: bool) -> PResult<'a> {
        let m = self.mark();
        self.opt_keyword(&["wildcard"]);
        if !self.at_any(&["bins", "illegal_bins", "ignore_bins"]) {
            return Err(self.expected("`bins`, `illegal_bins` or `ignore_bins`"));
        }
        self.leaf(Kind::Keyword);
        self.name()?;
        if self.at("[") {
            let d = self.mark();
            self.bump();
            if !self.at("]") {
                self.expr()?;
            }
            self.expect("]")?;
            self.finish(d, Kind::SizeDim);
        }
        self.expect("=")?;
        if cross {
            self.expr()?;
            if self.at("matches") {
                self.bump();
                self.expr_or_dollar()?;
            }
        } else if self.at("default") {
            self.leaf(Kind::Keyword);
            self.opt_keyword(&["sequence"]);
        } else if self.at("{") {
            self.range_list()?;
            if self.at("with") {
                self.bump();
                self.expect("(")?;
                self.expr()?;
                self.expect(")")?;
            }
        } else if !(self.at("(") && self.attempt(|p| p.trans_list())) {
            self.expr()?;
        }
        self.iff_opt()?;
        self.finish(m, Kind::Bins);
        Ok(())
    }

    /// `iff (e)`, if there.
    fn iff_opt(&mut self) -> PResult<'a> {
        if self.at("iff") {
            let m = self.mark();
            self.bump();
            self.expect("(")?;
            self.expr()?;
            self.expect(")")?;
            self.finish(m, Kind::Iff);
        }
        Ok(())
    }

    fn trans_list(&mut self) -> PResult<'a> {
        let m = self.mark();
        loop {
            let s = self.mark();
            self.expect("(")?;
            loop {
                let i = self.mark();
                let r = self.mark();
                loop {
                    self.value_range()?;
                    if !self.eat(",") {
                        break;
                    }
                }
                self.finish(r, Kind::RangeList);
                if self.at("[") && (self.at_n(1, "*") || self.at_n(1, "->") || self.at_n(1, "=")) {
                    self.bump();
                    self.leaf(Kind::Op);
                    self.expr()?;
                    if self.eat(":") {
                        self.expr()?;
                    }
                    self.expect("]")?;
                }
                self.finish(i, Kind::TransItem);
                if !self.eat("=>") {
                    break;
                }
            }
            self.expect(")")?;
            self.finish(s, Kind::TransSet);
            if !(self.at(",") && self.at_n(1, "(")) {
                break;
            }
            self.bump();
        }
        self.finish(m, Kind::TransList);
        Ok(())
    }
}
