//! Gates (A.3), UDPs (A.5), specify blocks (A.7) and configurations
//! (A.1.5).

use super::{Mark, PResult, Parser};
use crate::ast::{Kind, NodeId};
use crate::source::Error;

pub(crate) const TIMING_CHECKS: &[&str] = &[
    "$setup",
    "$hold",
    "$setuphold",
    "$recovery",
    "$removal",
    "$recrem",
    "$skew",
    "$timeskew",
    "$fullskew",
    "$period",
    "$width",
    "$nochange",
];

impl<'a> Parser<'a> {
    /// `and #1 g1 (o, a, b), g2 (...);` and the other gates and switches.
    pub(crate) fn gate_instantiation(&mut self, m: Mark) -> PResult<'a> {
        let kind = self.tok(0).0;
        self.leaf(Kind::Keyword);
        let no_delay = matches!(kind, "tran" | "rtran" | "pullup" | "pulldown");
        if self.strength_at(0) {
            self.strength()?;
        }
        if self.at("#") {
            if no_delay {
                return Err(self.error(format!("a {kind} cannot have a delay")));
            }
            self.delay()?;
        }
        loop {
            let i = self.mark();
            if self.name_at(0) {
                self.name()?;
                while self.at("[") {
                    self.variable_dim()?;
                }
            }
            self.expect("(")?;
            loop {
                let c = self.mark();
                self.expr()?;
                self.finish(c, Kind::Connection);
                if !self.eat(",") {
                    break;
                }
            }
            self.expect(")")?;
            self.finish(i, Kind::Instance);
            if !self.eat(",") {
                break;
            }
        }
        self.expect(";")?;
        self.finish(m, Kind::GateInstantiation);
        Ok(())
    }

    // ---- UDPs ----

    /// `primitive name (ports); declarations [initial] table ... endtable
    /// endprimitive`.
    pub(crate) fn udp(&mut self, m: Mark) -> PResult<'a> {
        self.bump();
        self.name()?;
        self.expect("(")?;
        let mut outputs = 0;
        let mut has_reg = false;
        if self.at(".") && self.at_n(1, "*") {
            self.bump();
            self.bump();
        } else if self.at_any(&["output", "input"]) || self.at("(") {
            loop {
                let (o, r) = self.udp_port_decl()?;
                outputs += o;
                has_reg |= r;
                if !self.eat(",") {
                    break;
                }
            }
        } else {
            loop {
                self.name()?;
                if !self.eat(",") {
                    break;
                }
            }
        }
        self.expect(")")?;
        self.expect(";")?;
        while self.at_any(&["output", "input", "reg", "inout"]) || self.at("(") && self.at_n(1, "*") {
            let (o, r) = self.udp_port_decl()?;
            outputs += o;
            has_reg |= r;
            self.expect(";")?;
        }
        if outputs > 1 {
            return Err(self.error_at_mark(m, "a UDP has exactly one output"));
        }
        if self.at("initial") {
            let i = self.mark();
            self.bump();
            if self.at("#") {
                return Err(self.error("the initial statement of a UDP cannot have a delay"));
            }
            self.name()?;
            self.expect("=")?;
            if self.at("#") {
                return Err(self.error("the initial statement of a UDP cannot have a delay"));
            }
            self.udp_init_value()?;
            self.expect(";")?;
            self.finish(i, Kind::UdpInitial);
        }
        let t = self.mark();
        self.expect("table")?;
        let mut sequential = None;
        while !self.at("endtable") {
            if self.at_eof() {
                return Err(self.expected("`endtable`"));
            }
            let fields = self.udp_entry()?;
            match sequential {
                None => sequential = Some(fields == 3),
                Some(s) if s != (fields == 3) => {
                    return Err(self.error_node(self.last().unwrap(), "every row of a UDP table has the same fields"));
                }
                _ => {}
            }
        }
        self.bump();
        let sequential = sequential.unwrap_or(false);
        let table = self.finish(t, Kind::UdpTable);
        if has_reg && !sequential {
            return Err(self.error_node(table, "a combinational UDP cannot declare its output reg"));
        }
        self.check_udp_rows(table, sequential)?;
        self.expect("endprimitive")?;
        self.end_label()?;
        self.finish(m, Kind::Udp);
        Ok(())
    }

    /// One port declaration of a UDP; returns how many outputs it declared
    /// and whether it made the output a reg.
    fn udp_port_decl(&mut self) -> PResult<'a, (usize, bool)> {
        let m = self.mark();
        self.attributes_opt()?;
        if self.at("inout") {
            return Err(self.error("a UDP port cannot be inout"));
        }
        let output = self.at("output");
        let reg = self.at("reg");
        self.leaf(Kind::Keyword);
        let mut has_reg = reg;
        if output && self.at("reg") {
            self.leaf(Kind::Keyword);
            has_reg = true;
        }
        if self.at("[") {
            return Err(self.error("a UDP port is a single bit and cannot have a range"));
        }
        let mut count = 0;
        loop {
            self.name()?;
            count += 1;
            if self.at("[") {
                return Err(self.error("a UDP port is a single bit and cannot have a range"));
            }
            if output && self.eat("=") {
                self.udp_init_value()?;
            }
            // In the body, `output y, z;` lists names; in an ANSI header the
            // next declaration starts with its direction.
            if !(self.at(",") && self.name_at(1) && (!output || !self.at_n(2, "=") && !self.at_n(2, ")"))) {
                break;
            }
            self.bump();
        }
        self.finish(m, Kind::UdpPortDecl);
        Ok((if output { count } else { 0 }, has_reg))
    }

    /// `0`, `1`, `1'b0`, `1'b1`, `1'bx`.
    fn udp_init_value(&mut self) -> PResult<'a> {
        let start = self.pos;
        self.number()?;
        let text: String = self.ast.tokens[start..self.pos].iter().map(|t| t.0).collect();
        let ok = matches!(text.to_ascii_lowercase().as_str(), "0" | "1" | "1'b0" | "1'b1" | "1'bx");
        if !ok {
            return Err(Error::new(self.ast.tokens[start].0, "a UDP's initial value is 0, 1 or x, one bit wide"));
        }
        Ok(())
    }

    /// One row of a table, up to its `;`. Returns how many fields it has.
    fn udp_entry(&mut self) -> PResult<'a, usize> {
        let m = self.mark();
        let mut fields = 1;
        while !self.at(";") {
            if self.at_eof() || self.at("endtable") {
                return Err(self.expected("`;`"));
            }
            if self.at(":") {
                fields += 1;
            }
            self.bump();
        }
        self.bump();
        self.finish(m, Kind::UdpEntry);
        Ok(fields)
    }

    /// Checks the symbols of each row (29.3.4, 29.3.6).
    fn check_udp_rows(&self, table: NodeId, sequential: bool) -> PResult<'a> {
        for &row in self.ast.children(table) {
            let toks = self.ast.tokens_of(row);
            let mut field = 0;
            let mut edges = 0;
            let fields = if sequential { 3 } else { 2 };
            let bad = |msg: &str| Err(self.error_node(row, msg));
            let mut in_paren = false;
            for t in toks {
                for c in t.0.chars() {
                    match c {
                        ':' => field += 1,
                        ';' => {}
                        'z' | 'Z' => return bad("z cannot appear in a UDP table"),
                        '(' => {
                            in_paren = true;
                            edges += 1;
                        }
                        ')' => in_paren = false,
                        _ if field == 0 => {
                            let level = matches!(c, '0' | '1' | 'x' | 'X' | '?' | 'b' | 'B');
                            let edge = matches!(c, 'r' | 'R' | 'f' | 'F' | 'p' | 'P' | 'n' | 'N' | '*');
                            if edge {
                                edges += 1;
                            }
                            if !level && !edge || edge && in_paren {
                                return bad(&format!("`{c}` is not a UDP input symbol"));
                            }
                        }
                        _ if field == fields - 1 => {
                            let ok = matches!(c, '0' | '1' | 'x' | 'X') || sequential && c == '-';
                            if !ok {
                                return bad(&format!("`{c}` cannot be an output in a UDP table"));
                            }
                        }
                        _ => {
                            if !matches!(c, '0' | '1' | 'x' | 'X' | '?' | 'b' | 'B') {
                                return bad(&format!("`{c}` is not a UDP state symbol"));
                            }
                        }
                    }
                }
            }
            if field != fields - 1 {
                return bad("a UDP table row has the wrong number of fields");
            }
            if edges > 0 && !sequential {
                return bad("a combinational UDP cannot have edges in its table");
            }
            if edges > 1 {
                return bad("a UDP table row can have at most one edge");
            }
        }
        Ok(())
    }

    // ---- specify blocks ----

    pub(crate) fn specify(&mut self, m: Mark) -> PResult<'a> {
        self.bump();
        while !self.at("endspecify") {
            if self.at_eof() {
                return Err(self.expected("`endspecify`"));
            }
            let i = self.mark();
            if self.at("specparam") {
                self.specparam_decl()?;
            } else if self.at_any(&["pulsestyle_onevent", "pulsestyle_ondetect", "showcancelled", "noshowcancelled"]) {
                self.leaf(Kind::Keyword);
                loop {
                    self.postfix()?;
                    if !self.eat(",") {
                        break;
                    }
                }
                self.expect(";")?;
                self.finish(i, Kind::PulseStyle);
            } else if self.at("if") || self.at("ifnone") || self.at("(") {
                self.path_decl()?;
            } else if self.tok(0).is_system() {
                if !TIMING_CHECKS.contains(&self.tok(0).0) {
                    return Err(self.error("only timing checks can be called in a specify block"));
                }
                self.timing_check()?;
            } else {
                return Err(self.expected("a specify item"));
            }
        }
        self.bump();
        self.finish(m, Kind::Specify);
        Ok(())
    }

    /// `[if (c) | ifnone] (inputs [+|-] => | *> outputs) = delays;`.
    fn path_decl(&mut self) -> PResult<'a> {
        let m = self.mark();
        let ifnone = self.at("ifnone");
        if self.at("if") {
            self.leaf(Kind::Keyword);
            self.expect("(")?;
            self.expr()?;
            self.expect(")")?;
        } else if ifnone {
            self.leaf(Kind::Keyword);
        }
        self.expect("(")?;
        if self.at_any(&["posedge", "negedge", "edge"]) {
            if ifnone {
                return Err(self.error("ifnone cannot be used on an edge-sensitive path"));
            }
            self.leaf(Kind::Keyword);
        }
        loop {
            self.postfix()?;
            if !self.eat(",") {
                break;
            }
        }
        // The polarity and the connection: `=>`, `*>`, `+=>`, `-*>`, ...
        if (self.at("+=") || self.at("-=")) && self.at_n(1, ">") && self.adjacent(0) {
            let o = self.mark();
            self.bump();
            self.bump();
            self.finish(o, Kind::Op);
        } else {
            if self.at("+") || self.at("-") {
                self.leaf(Kind::Op);
            }
            if !self.at("=>") && !self.at("*>") {
                return Err(self.expected("`=>` or `*>`"));
            }
            self.leaf(Kind::Op);
        }
        if self.at("(") {
            // `(q +: d)`: outputs, polarity and the data source.
            self.bump();
            loop {
                self.postfix()?;
                if !self.eat(",") {
                    break;
                }
            }
            if self.at_any(&["+:", "-:", ":"]) {
                self.leaf(Kind::Op);
            } else {
                if self.at("+") || self.at("-") {
                    self.leaf(Kind::Op);
                }
                self.expect(":")?;
            }
            self.expr()?;
            self.expect(")")?;
        } else {
            loop {
                self.postfix()?;
                if !self.eat(",") {
                    break;
                }
            }
        }
        self.expect(")")?;
        self.expect("=")?;
        let parens = self.at("(");
        if parens {
            self.bump();
        }
        loop {
            self.mintypmax()?;
            if !self.eat(",") {
                break;
            }
        }
        if parens {
            self.expect(")")?;
        }
        self.expect(";")?;
        self.finish(m, Kind::PathDecl);
        Ok(())
    }

    /// `$setup(data, posedge clk &&& en, 10, notifier);` and the rest.
    fn timing_check(&mut self) -> PResult<'a> {
        let m = self.mark();
        let name = self.tok(0).0;
        self.leaf(Kind::SystemName);
        self.expect("(")?;
        let mut args: Vec<bool> = Vec::new();
        loop {
            if self.at(",") || self.at(")") {
                self.empty();
                args.push(false);
            } else {
                self.timing_arg()?;
                args.push(true);
            }
            if !self.eat(",") {
                break;
            }
        }
        self.expect(")")?;
        if name == "$width" && args.len() >= 4 && !args[2] && args[3] {
            return Err(self.error_at_mark(m, "$width needs a threshold before its notifier"));
        }
        self.expect(";")?;
        self.finish(m, Kind::TimingCheck);
        Ok(())
    }

    fn timing_arg(&mut self) -> PResult<'a> {
        let m = self.mark();
        let mut event = false;
        if self.at_any(&["posedge", "negedge"]) {
            self.leaf(Kind::Keyword);
            event = true;
        } else if self.at("edge") {
            self.leaf(Kind::Keyword);
            if self.at("[") {
                let e = self.mark();
                while !self.at("]") {
                    if self.at_eof() {
                        return Err(self.expected("`]`"));
                    }
                    self.bump();
                }
                self.bump();
                self.finish(e, Kind::EdgeDescriptors);
            }
            event = true;
        }
        self.mintypmax()?;
        if self.eat("&&&") {
            self.expr()?;
            event = true;
        }
        if event {
            self.finish(m, Kind::TimingEvent);
        }
        Ok(())
    }

    // ---- configurations ----

    /// `config name; design lib.cell; rules endconfig`.
    pub(crate) fn config(&mut self, m: Mark) -> PResult<'a> {
        self.bump();
        self.name()?;
        self.expect(";")?;
        while self.at("localparam") {
            self.param_decl(false)?;
            self.expect(";")?;
        }
        let d = self.mark();
        self.expect("design")?;
        while !self.at(";") {
            self.cell_name()?;
        }
        self.bump();
        self.finish(d, Kind::ConfigDesign);
        while !self.at("endconfig") {
            if self.at_eof() {
                return Err(self.expected("`endconfig`"));
            }
            let r = self.mark();
            if self.at("default") {
                self.leaf(Kind::Keyword);
            } else if self.at("instance") || self.at("cell") {
                self.leaf(Kind::Keyword);
                self.cell_name()?;
            } else {
                return Err(self.expected("`default`, `instance` or `cell`"));
            }
            if self.at("liblist") {
                self.leaf(Kind::Keyword);
                while !self.at(";") {
                    self.name()?;
                }
            } else if self.at("use") {
                self.leaf(Kind::Keyword);
                if self.at("#") {
                    self.param_values()?;
                } else if self.at(".") {
                    loop {
                        let n = self.mark();
                        self.bump();
                        self.name()?;
                        self.expect("(")?;
                        if !self.at(")") {
                            self.expr()?;
                        }
                        self.expect(")")?;
                        self.finish(n, Kind::NamedArg);
                        if !self.eat(",") {
                            break;
                        }
                    }
                } else {
                    self.cell_name()?;
                    while self.eat(",") {
                        let n = self.mark();
                        self.expect(".")?;
                        self.name()?;
                        self.expect("(")?;
                        if !self.at(")") {
                            self.expr()?;
                        }
                        self.expect(")")?;
                        self.finish(n, Kind::NamedArg);
                    }
                }
                if self.eat(":") {
                    self.expect("config")?;
                }
            } else {
                return Err(self.expected("`liblist` or `use`"));
            }
            self.expect(";")?;
            self.finish(r, Kind::ConfigRule);
        }
        self.bump();
        self.end_label()?;
        self.finish(m, Kind::Config);
        Ok(())
    }

    /// `cell`, `lib.cell` or `top.u1.u2`.
    fn cell_name(&mut self) -> PResult<'a> {
        let m = self.mark();
        self.name()?;
        while self.at(".") && self.tok(1).is_ident() {
            self.bump();
            self.leaf(Kind::Name);
            self.finish(m, Kind::Member);
        }
        Ok(())
    }
}
