//! Statements (A.6.2 to A.6.12).

use super::{PResult, Parser};
use crate::ast::Kind;
use crate::parse::declarations::DIRECTIONS;

impl<'a> Parser<'a> {
    /// A statement or `;`.
    pub(crate) fn statement(&mut self) -> PResult<'a> {
        let m = self.mark();
        self.attributes_opt()?;
        if self.eat(";") {
            self.finish(m, Kind::NullStatement);
            return Ok(());
        }
        if self.name_at(0) && self.at_n(1, ":") {
            let label = self.tok(0).ident_name();
            self.leaf(Kind::Name);
            self.bump();
            self.statement_item(Some(label))?;
            self.finish(m, Kind::Labeled);
            return Ok(());
        }
        self.statement_item(None)
    }

    fn statement_item(&mut self, label: Option<&'a str>) -> PResult<'a> {
        let m = self.mark();
        let t = self.tok(0).0;
        let kw = self.keyword_at(0);
        match t {
            "begin" | "fork" if kw => return self.block(label),
            "if" if kw => return self.if_statement(m),
            "unique" | "unique0" | "priority" if kw => {
                self.leaf(Kind::Keyword);
                return if self.at("if") { self.if_statement(m) } else { self.case_statement(m) };
            }
            "case" | "casex" | "casez" if kw => return self.case_statement(m),
            "randcase" if kw => return self.randcase(),
            "randsequence" if kw => return self.randsequence(),
            "forever" | "repeat" | "while" if kw => {
                self.leaf(Kind::Keyword);
                if t != "forever" {
                    self.expect("(")?;
                    self.expr()?;
                    self.expect(")")?;
                }
                self.statement()?;
                self.finish(m, Kind::Loop);
            }
            "for" if kw => return self.for_statement(),
            "do" if kw => {
                self.bump();
                self.statement()?;
                self.expect("while")?;
                self.expect("(")?;
                self.expr()?;
                self.expect(")")?;
                self.expect(";")?;
                self.finish(m, Kind::DoWhile);
            }
            "foreach" if kw => return self.foreach(),
            "return" | "break" | "continue" if kw => {
                self.leaf(Kind::Keyword);
                if t == "return" && !self.at(";") {
                    self.expr()?;
                }
                self.expect(";")?;
                self.finish(m, Kind::Jump);
            }
            "#" => {
                self.delay()?;
                self.statement()?;
                self.finish(m, Kind::Timed);
            }
            "@" => {
                self.event_control()?;
                self.statement()?;
                self.finish(m, Kind::Timed);
            }
            "##" => {
                self.cycle_delay()?;
                self.statement()?;
                self.finish(m, Kind::Timed);
            }
            "wait" if kw => {
                self.leaf(Kind::Keyword);
                if self.eat("fork") {
                    self.expect(";")?;
                } else {
                    self.expect("(")?;
                    self.expr()?;
                    self.expect(")")?;
                    self.statement()?;
                }
                self.finish(m, Kind::Wait);
            }
            "wait_order" if kw => {
                self.leaf(Kind::Keyword);
                self.expect("(")?;
                loop {
                    self.postfix()?;
                    if !self.eat(",") {
                        break;
                    }
                }
                self.expect(")")?;
                self.action_block()?;
                self.finish(m, Kind::Wait);
            }
            "->" | "->>" => {
                self.leaf(Kind::Op);
                if t == "->>" {
                    self.delay_or_event()?;
                }
                self.postfix()?;
                self.expect(";")?;
                self.finish(m, Kind::EventTrigger);
            }
            "disable" if kw => {
                self.bump();
                if self.at("fork") {
                    self.leaf(Kind::Keyword);
                } else {
                    self.postfix()?;
                }
                self.expect(";")?;
                self.finish(m, Kind::Disable);
            }
            "assign" | "force" if kw => {
                self.leaf(Kind::Keyword);
                let a = self.mark();
                self.unary()?;
                self.expect("=")?;
                self.expr()?;
                self.finish(a, Kind::Assign);
                self.expect(";")?;
                self.finish(m, Kind::ProceduralContinuous);
            }
            "deassign" | "release" if kw => {
                self.leaf(Kind::Keyword);
                self.unary()?;
                self.expect(";")?;
                self.finish(m, Kind::ProceduralContinuous);
            }
            "assert" | "assume" | "cover" | "restrict" | "expect" if kw => return self.assertion_statement(m),
            _ => {
                if self.var_decl_at() && !self.at("void") {
                    return Err(self.error("declarations must come before the statements of a block"));
                }
                if self.name_at(0) && self.name_at(1) && self.at_n(2, "(") {
                    // A checker instantiated in a procedure.
                    return self.instantiation(m);
                }
                return self.expression_statement();
            }
        }
        Ok(())
    }

    /// An assignment, an increment, or a call used as a statement.
    fn expression_statement(&mut self) -> PResult<'a> {
        let m = self.mark();
        self.unary()?;
        if self.assign_op_at() {
            let blocking_eq = self.at("=");
            self.leaf(Kind::Op);
            if blocking_eq {
                self.delay_or_event()?;
            }
            self.expr()?;
            self.expect(";")?;
            self.finish(m, Kind::Assign);
            return Ok(());
        }
        if self.at("<=") {
            self.leaf(Kind::Op);
            if self.at("##") {
                self.cycle_delay()?;
            } else {
                self.delay_or_event()?;
            }
            self.expr()?;
            self.expect(";")?;
            self.finish(m, Kind::Assign);
            return Ok(());
        }
        let ok = matches!(
            self.last_kind(),
            Some(
                Kind::Call
                    | Kind::IncDec
                    | Kind::Name
                    | Kind::Member
                    | Kind::Scope
                    | Kind::SystemName
                    | Kind::With
                    | Kind::Cast
                    | Kind::New
                    | Kind::AssignExpr
            )
        );
        if !ok {
            return Err(self.expected("`=`, `<=` or `;`"));
        }
        let target = self.ast.children(self.last().unwrap()).first().map(|&c| self.ast.text(c));
        if self.last_kind() == Some(Kind::Call)
            && target.is_some_and(|t| crate::parse::other::TIMING_CHECKS.contains(&t.as_str()))
        {
            return Err(self.error_at_mark(m, "a timing check can only appear in a specify block"));
        }
        self.expect(";")?;
        self.finish(m, Kind::ExprStatement);
        Ok(())
    }

    /// `begin ... end` or `fork ... join`, with an optional name. `label`
    /// is the statement label in front, if any.
    fn block(&mut self, label: Option<&'a str>) -> PResult<'a> {
        let m = self.mark();
        let fork = self.at("fork");
        self.leaf(Kind::Keyword);
        let mut name = label;
        if self.at(":") {
            if label.is_some() {
                return Err(self.error("a block cannot have both a statement label and a block name"));
            }
            self.bump();
            name = Some(self.tok(0).ident_name());
            self.name()?;
        }
        let ends: &[&str] = if fork { &["join", "join_any", "join_none"] } else { &["end"] };
        self.block_body(ends[0], false)?;
        if fork {
            if !self.at_any(ends) {
                return Err(self.expected("`join`, `join_any` or `join_none`"));
            }
            self.leaf(Kind::Keyword);
        } else {
            self.expect("end")?;
        }
        self.check_end_label(name)?;
        self.finish(m, Kind::Block);
        Ok(())
    }

    /// An end label, which must repeat the block's name.
    pub(crate) fn check_end_label(&mut self, name: Option<&str>) -> PResult<'a> {
        if self.at(":") && !self.at_n(1, ":") {
            let end = self.tok(1);
            match name {
                Some(n) if n == end.ident_name() => {}
                Some(n) => return Err(Error::new(end.0, format!("the name after the end must be `{n}`"))),
                None => return Err(Error::new(end.0, "a block without a name cannot have one at its end")),
            }
            self.end_label()?;
        }
        Ok(())
    }

    /// The declarations then statements of a block or subroutine, up to
    /// `end` (or one of the join keywords, or `}`). With `ports`, port
    /// declarations are allowed among the declarations.
    pub(crate) fn block_body(&mut self, end: &str, ports: bool) -> PResult<'a> {
        let ends: &[&str] = match end {
            "join" => &["join", "join_any", "join_none"],
            _ => std::slice::from_ref(&end),
        };
        loop {
            let s = self.save();
            let m = self.mark();
            self.attributes_opt()?;
            if ports && (self.at_any(DIRECTIONS) || self.at("const") && self.at_n(1, "ref")) {
                self.tf_port_decl()?;
            } else if self.at_any(&["parameter", "localparam"]) {
                self.param_decl(false)?;
                self.expect(";")?;
            } else if self.at("let") {
                self.let_decl()?;
            } else if self.at("import") && !self.tok(1).is_string() {
                self.package_import()?;
            } else if self.at("typedef") || self.var_decl_at() && !(self.at("void") && self.at_n(1, "'")) {
                self.data_declaration()?;
            } else {
                self.restore(s);
                break;
            }
            let _ = m;
        }
        while !self.at_any(ends) {
            if self.at_eof() {
                return Err(self.expected(&format!("`{}`", ends[0])));
            }
            self.statement()?;
        }
        Ok(())
    }

    fn if_statement(&mut self, m: super::Mark) -> PResult<'a> {
        self.expect("if")?;
        self.expect("(")?;
        self.cond_predicate()?;
        self.expect(")")?;
        self.statement()?;
        if self.eat("else") {
            self.statement()?;
        }
        self.finish(m, Kind::If);
        Ok(())
    }

    /// `e`, `e matches p`, and `&&&` between them.
    pub(crate) fn cond_predicate(&mut self) -> PResult<'a> {
        let m = self.mark();
        self.expr()?;
        if self.at("&&&") {
            while self.eat("&&&") {
                self.expr()?;
            }
            self.finish(m, Kind::CondPredicate);
        }
        Ok(())
    }

    fn case_statement(&mut self, m: super::Mark) -> PResult<'a> {
        if !self.at_any(&["case", "casex", "casez"]) {
            return Err(self.expected("`if` or `case`"));
        }
        self.leaf(Kind::Keyword);
        self.expect("(")?;
        self.expr()?;
        self.expect(")")?;
        let inside = self.at("inside");
        let matches = self.at("matches");
        self.opt_keyword(&["inside", "matches"]);
        while !self.at("endcase") {
            let i = self.mark();
            if self.eat("default") {
                self.eat(":");
            } else {
                loop {
                    if inside {
                        self.value_range()?;
                    } else if matches {
                        self.pattern()?;
                        if self.eat("&&&") {
                            self.expr()?;
                        }
                    } else {
                        self.expr()?;
                    }
                    if !self.eat(",") {
                        break;
                    }
                }
                self.expect(":")?;
            }
            self.statement()?;
            self.finish(i, Kind::CaseItem);
            if self.at_eof() {
                return Err(self.expected("`endcase`"));
            }
        }
        self.bump();
        self.finish(m, Kind::Case);
        Ok(())
    }

    /// A pattern after `matches` (12.6).
    pub(crate) fn pattern(&mut self) -> PResult<'a> {
        let m = self.mark();
        if self.at(".*") {
            self.leaf(Kind::Op);
        } else if self.at(".") {
            self.bump();
            self.name()?;
        } else if self.at("tagged") {
            self.bump();
            self.name()?;
            if !self.at_any(&[")", ":", ",", "&&&", "}", "?"]) && !self.at("iff") {
                self.pattern()?;
            }
        } else if self.at("'{") {
            self.bump();
            loop {
                let p = self.mark();
                if self.name_at(0) && self.at_n(1, ":") {
                    self.name()?;
                    self.bump();
                }
                self.pattern()?;
                if self.since(p).len() > 1 {
                    self.finish(p, Kind::PatternKey);
                }
                if !self.eat(",") {
                    break;
                }
            }
            self.expect("}")?;
        } else if self.at("(")
            && self.lookahead(|p| {
                p.bump();
                p.at(".") || p.at(".*") || p.at("tagged") || p.at("'{")
            })
        {
            self.bump();
            self.pattern()?;
            self.expect(")")?;
        } else {
            self.expr_bp(3)?;
        }
        self.finish(m, Kind::Pattern);
        Ok(())
    }

    fn randcase(&mut self) -> PResult<'a> {
        let m = self.mark();
        self.bump();
        while !self.at("endcase") {
            let i = self.mark();
            self.expr()?;
            self.expect(":")?;
            self.statement()?;
            self.finish(i, Kind::CaseItem);
            if self.at_eof() {
                return Err(self.expected("`endcase`"));
            }
        }
        self.bump();
        self.finish(m, Kind::Randcase);
        Ok(())
    }

    fn for_statement(&mut self) -> PResult<'a> {
        let m = self.mark();
        self.bump();
        self.expect("(")?;
        let init = self.mark();
        if !self.at(";") {
            let decl = |p: &Self| p.at("var") || p.type_keyword_at(0) || p.type_then_name_at(0);
            if decl(self) {
                loop {
                    let d = self.mark();
                    self.opt_keyword(&["var"]);
                    self.data_type(false)?;
                    self.declarator()?;
                    while self.at(",") && self.name_at(1) && self.at_n(2, "=") {
                        self.bump();
                        self.declarator()?;
                    }
                    self.finish(d, Kind::VarDecl);
                    if !self.eat(",") {
                        break;
                    }
                }
            } else {
                loop {
                    let a = self.mark();
                    self.unary()?;
                    if !self.assign_op_at() {
                        return Err(self.expected("`=`"));
                    }
                    self.leaf(Kind::Op);
                    self.expr()?;
                    self.finish(a, Kind::Assign);
                    if !self.eat(",") {
                        break;
                    }
                    if decl(self) {
                        return Err(self.error("a for loop must declare all of its variables or none of them"));
                    }
                }
            }
        }
        self.finish(init, Kind::ForInit);
        self.expect(";")?;
        if self.at(";") {
            self.empty();
        } else {
            self.expr()?;
        }
        self.expect(";")?;
        let step = self.mark();
        if !self.at(")") {
            loop {
                let a = self.mark();
                self.unary()?;
                if self.assign_op_at() {
                    self.leaf(Kind::Op);
                    self.expr()?;
                    self.finish(a, Kind::Assign);
                }
                if !self.eat(",") {
                    break;
                }
            }
        }
        self.finish(step, Kind::ForStep);
        self.expect(")")?;
        self.statement()?;
        self.finish(m, Kind::For);
        Ok(())
    }

    fn foreach(&mut self) -> PResult<'a> {
        let m = self.mark();
        self.foreach_header()?;
        self.statement()?;
        self.finish(m, Kind::Foreach);
        Ok(())
    }

    /// `foreach (array[i, j])`: the array, then the loop variables.
    pub(crate) fn foreach_header(&mut self) -> PResult<'a> {
        self.bump();
        self.expect("(")?;
        // The array, then its loop variables in the last brackets.
        let a = self.mark();
        self.primary()?;
        loop {
            if (self.at(".") || self.at("::")) && self.tok(1).is_ident() {
                let member = self.at(".");
                self.bump();
                self.leaf(Kind::Name);
                self.finish(a, if member { Kind::Member } else { Kind::Scope });
            } else if self.at("[") && self.skip_balanced(0).is_some_and(|j| self.at_n(j, "[")) {
                self.bump();
                self.expr()?;
                self.expect("]")?;
                self.finish(a, Kind::Index);
            } else {
                break;
            }
        }
        self.expect("[")?;
        loop {
            if self.name_at(0) {
                self.name()?;
            } else {
                self.empty();
            }
            if !self.eat(",") {
                break;
            }
        }
        self.expect("]")?;
        self.expect(")")
    }

    /// `pass_statement [else fail_statement]`.
    pub(crate) fn action_block(&mut self) -> PResult<'a> {
        let m = self.mark();
        if self.at("else") {
            self.empty();
        } else {
            self.statement()?;
        }
        if self.eat("else") {
            self.statement()?;
        }
        self.finish(m, Kind::ActionBlock);
        Ok(())
    }

    /// `assert`, `assume`, `cover`, `restrict` or `expect`, immediate or
    /// concurrent.
    pub(crate) fn assertion_statement(&mut self, m: super::Mark) -> PResult<'a> {
        let kw = self.tok(0).0;
        self.leaf(Kind::Keyword);
        if kw == "expect" {
            self.expect("(")?;
            self.property_spec()?;
            self.expect(")")?;
            self.action_block()?;
            self.finish(m, Kind::Expect);
            return Ok(());
        }
        if self.at("property") || self.at("sequence") {
            self.leaf(Kind::Keyword);
            self.expect("(")?;
            self.property_spec()?;
            self.expect(")")?;
            match kw {
                "assert" | "assume" => self.action_block()?,
                "cover" => {
                    self.statement()?;
                    if self.at("else") {
                        return Err(self.error("a cover statement cannot have an else"));
                    }
                }
                _ => {
                    if !self.eat(";") {
                        return Err(self.error("a restrict statement ends with `;` and has no action"));
                    }
                }
            }
            self.finish(m, Kind::ConcurrentAssertion);
            return Ok(());
        }
        if kw == "restrict" {
            return Err(self.expected("`property`"));
        }
        if self.at("#") {
            if !(self.tok(1).0 == "0") {
                return Err(self.error("a deferred assertion is written `#0`"));
            }
            let d = self.mark();
            self.bump();
            self.bump();
            self.finish(d, Kind::Op);
        } else {
            self.opt_keyword(&["final"]);
        }
        self.expect("(")?;
        self.expr()?;
        self.expect(")")?;
        if kw == "cover" {
            self.statement()?;
            if self.at("else") {
                return Err(self.error("a cover statement cannot have an else"));
            }
        } else {
            self.action_block()?;
        }
        self.finish(m, Kind::ImmediateAssertion);
        Ok(())
    }

    fn randsequence(&mut self) -> PResult<'a> {
        let m = self.mark();
        self.bump();
        self.expect("(")?;
        if self.name_at(0) {
            self.name()?;
        }
        self.expect(")")?;
        while !self.at("endsequence") {
            if self.at_eof() {
                return Err(self.expected("`endsequence`"));
            }
            let p = self.mark();
            if self.at("void") {
                self.leaf(Kind::VoidType);
            } else if self.type_keyword_at(0) || self.type_then_name_at(0) {
                self.data_type(false)?;
            }
            self.name()?;
            if self.at("(") {
                self.tf_ports()?;
            }
            self.expect(":")?;
            loop {
                let r = self.mark();
                if self.at("rand") && self.at_n(1, "join") {
                    self.bump();
                    self.leaf(Kind::Keyword);
                    if self.at("(") {
                        self.bump();
                        self.expr()?;
                        self.expect(")")?;
                    }
                }
                while !self.at_any(&["|", ";", ":="]) {
                    if self.at_eof() {
                        return Err(self.expected("`;`"));
                    }
                    self.production_item()?;
                }
                if self.eat(":=") {
                    if self.at("(") {
                        self.bump();
                        self.expr()?;
                        self.expect(")")?;
                    } else {
                        self.primary()?;
                    }
                    if self.at("{") {
                        self.code_block()?;
                    }
                }
                self.finish(r, Kind::Rule);
                if !self.eat("|") {
                    break;
                }
            }
            self.expect(";")?;
            self.finish(p, Kind::Production);
        }
        self.bump();
        self.finish(m, Kind::Randsequence);
        Ok(())
    }

    /// One item of a randsequence rule.
    fn production_item(&mut self) -> PResult<'a> {
        let m = self.mark();
        if self.at("{") {
            return self.code_block();
        }
        if self.at("if") {
            self.bump();
            self.expect("(")?;
            self.expr()?;
            self.expect(")")?;
            self.production_item()?;
            if self.eat("else") {
                self.production_item()?;
            }
        } else if self.at("repeat") {
            self.bump();
            self.expect("(")?;
            self.expr()?;
            self.expect(")")?;
            self.production_item()?;
        } else if self.at("case") {
            self.bump();
            self.expect("(")?;
            self.expr()?;
            self.expect(")")?;
            while !self.at("endcase") {
                let i = self.mark();
                if self.eat("default") {
                    self.eat(":");
                } else {
                    loop {
                        self.expr()?;
                        if !self.eat(",") {
                            break;
                        }
                    }
                    self.expect(":")?;
                }
                self.production_item()?;
                self.expect(";")?;
                self.finish(i, Kind::CaseItem);
            }
            self.bump();
        } else {
            self.name()?;
            if self.at("(") {
                self.args()?;
            }
        }
        self.finish(m, Kind::ProdItem);
        Ok(())
    }

    /// `{ declarations statements }` in a randsequence.
    fn code_block(&mut self) -> PResult<'a> {
        let m = self.mark();
        self.expect("{")?;
        self.block_body("}", false)?;
        self.expect("}")?;
        self.finish(m, Kind::Block);
        Ok(())
    }
}

use crate::source::Error;
