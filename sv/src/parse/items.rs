//! Design elements and the items inside them (A.1, A.4).

use super::{Mark, PResult, Parser};
use crate::ast::Kind;
use crate::parse::declarations::NET_TYPES;

/// What kind of construct the items being read are in.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Scope {
    Unit,
    Module,
    Interface,
    Program,
    Package,
    Class,
    InterfaceClass,
    Checker,
}

pub(crate) const GATES: &[&str] = &[
    "cmos", "rcmos", "bufif0", "bufif1", "notif0", "notif1", "nmos", "pmos", "rnmos", "rpmos", "and", "nand", "or",
    "nor", "xor", "xnor", "buf", "not", "tranif0", "tranif1", "rtranif0", "rtranif1", "tran", "rtran", "pullup",
    "pulldown",
];

const CLASS_QUALIFIERS: &[&str] =
    &["rand", "randc", "static", "protected", "local", "const", "virtual", "pure", "extern"];

const SEVERITY_TASKS: &[&str] = &["$fatal", "$error", "$warning", "$info"];

impl<'a> Parser<'a> {
    /// Items until `end` (or the end of the input), in `scope`.
    pub(crate) fn items_until(&mut self, scope: Scope, end: Option<&str>) -> PResult<'a> {
        let outer = self.scope;
        self.scope = scope;
        loop {
            match end {
                Some(e) if self.at(e) => break,
                None if self.at_eof() => break,
                Some(e) if self.at_eof() => return Err(self.expected(&format!("`{e}`"))),
                _ => {}
            }
            self.item()?;
        }
        self.scope = outer;
        Ok(())
    }

    fn module_like(&self) -> bool {
        matches!(self.scope, Scope::Module | Scope::Interface | Scope::Program)
    }

    /// One item of the current scope.
    pub(crate) fn item(&mut self) -> PResult<'a> {
        let m = self.mark();
        self.attributes_opt()?;
        let t = self.tok(0).0;
        if !self.keyword_at(0) {
            if t == ";" {
                self.bump();
                return Ok(());
            }
            if SEVERITY_TASKS.contains(&t) {
                self.postfix()?;
                self.expect(";")?;
                self.finish(m, Kind::ElaborationTask);
                return Ok(());
            }
            if self.name_at(0) {
                return self.name_item(m);
            }
            if t == "$unit" || t == "local" {
                return self.data_declaration();
            }
            return Err(self.expected("a declaration or item"));
        }
        let class = matches!(self.scope, Scope::Class | Scope::InterfaceClass);
        if self.scope == Scope::Checker && self.at("rand") {
            // A free checker variable (17.7).
            let q = self.mark();
            self.leaf(Kind::Keyword);
            self.finish(q, Kind::Qualifiers);
            return self.var_decl(m);
        }
        if class && self.at_any(CLASS_QUALIFIERS) && !(self.at("virtual") && self.at_n(1, "class")) {
            return self.class_item(m);
        }
        match t {
            "module" | "macromodule" | "program" => {
                if t == "program" && self.at_n(1, ";") {
                    return self.anonymous_program(m);
                }
                self.design_element(m)
            }
            "interface" if self.at_n(1, "class") => self.interface_class(m),
            "interface" => self.design_element(m),
            "package" => {
                if self.scope != Scope::Unit {
                    return Err(self.error("a package can only be declared outside other design elements"));
                }
                self.package(m)
            }
            "primitive" => {
                if self.scope != Scope::Unit {
                    return Err(self.error("a UDP can only be declared outside other design elements"));
                }
                self.udp(m)
            }
            "config" => self.config(m),
            "checker" => self.checker(m),
            "class" => self.class(m),
            "virtual" if self.at_n(1, "class") => self.class(m),
            "extern" => self.extern_item(m),
            "bind" => self.bind(m),
            "input" | "output" | "inout" | "ref" => {
                if !self.module_like() {
                    return Err(self.error("port declarations belong in a module, interface or program"));
                }
                if self.generate_depth > 0 {
                    return Err(self.error("a port cannot be declared inside a generate block"));
                }
                if self.ansi.last() == Some(&true) {
                    return Err(self.error("a module whose header declares its ports cannot declare ports in its body"));
                }
                self.port_decl()
            }
            "parameter" | "localparam" => {
                self.param_decl(false)?;
                self.expect(";")
            }
            "specparam" => self.specparam_decl(),
            "typedef" | "nettype" => self.data_declaration(),
            "import" => {
                if class {
                    return Err(self.error("an import cannot appear directly in a class"));
                }
                if self.tok(1).is_string() { self.dpi_import(m) } else { self.package_import() }
            }
            "export" => {
                if self.tok(1).is_string() {
                    self.dpi_export(m)
                } else {
                    self.package_export()
                }
            }
            "genvar" => {
                self.bump();
                loop {
                    self.name()?;
                    if !self.eat(",") {
                        break;
                    }
                }
                self.expect(";")?;
                self.finish(m, Kind::GenvarDecl);
                Ok(())
            }
            "function" | "task" => self.subroutine(m, class),
            "constraint" => self.constraint_decl(m),
            "covergroup" => self.covergroup(),
            "property" | "sequence" => self.property_decl(),
            "let" => self.let_decl(),
            "assign" => self.continuous_assign(m),
            "alias" => {
                self.bump();
                self.postfix()?;
                while self.eat("=") {
                    self.postfix()?;
                }
                self.expect(";")?;
                self.finish(m, Kind::NetAlias);
                Ok(())
            }
            "initial" | "final" | "always" | "always_comb" | "always_ff" | "always_latch" => {
                self.leaf(Kind::Keyword);
                self.statement()?;
                self.finish(m, Kind::Process);
                Ok(())
            }
            "generate" => {
                if self.generate_depth > 0 {
                    return Err(self.error("a generate region cannot be nested inside a generate block"));
                }
                self.bump();
                self.generate_depth += 1;
                let r = self.items_until(self.scope, Some("endgenerate"));
                self.generate_depth -= 1;
                r?;
                self.bump();
                self.finish(m, Kind::GenerateRegion);
                Ok(())
            }
            "for" => self.generate_for(m),
            "if" => self.generate_if(m),
            "case" => self.generate_case(m),
            "begin" => self.generate_block(),
            "default" => self.default_item(m),
            "clocking" | "global" => self.clocking(m),
            "assert" | "assume" | "cover" | "restrict" => self.assertion_statement(m),
            "defparam" => {
                self.bump();
                loop {
                    let a = self.mark();
                    self.postfix()?;
                    self.expect("=")?;
                    self.mintypmax()?;
                    self.finish(a, Kind::DefparamAssign);
                    if !self.eat(",") {
                        break;
                    }
                }
                self.expect(";")?;
                self.finish(m, Kind::Defparam);
                Ok(())
            }
            "specify" => {
                if self.scope != Scope::Module {
                    return Err(self.error("a specify block can only appear in a module"));
                }
                self.specify(m)
            }
            "timeunit" | "timeprecision" => self.timeunits(m),
            "modport" => self.modport(m),
            "interconnect" => self.interconnect_decl(),
            _ if GATES.contains(&t) => self.gate_instantiation(m),
            _ if NET_TYPES.contains(&t) => self.net_decl(),
            _ if self.var_decl_at() => self.data_declaration(),
            _ => Err(self.expected("a declaration or item")),
        }
    }

    /// An item that starts with a name: an instantiation, a declaration of
    /// a variable of a named type, an interface port declaration, or a
    /// labelled assertion or generate block.
    fn name_item(&mut self, m: Mark) -> PResult<'a> {
        if self.at_n(1, ":") {
            if self.at_n(2, "begin") {
                return self.generate_block();
            }
            self.leaf(Kind::Name);
            self.bump();
            if !self.at_any(&["assert", "assume", "cover", "restrict"]) {
                return Err(self.expected("an assertion after the label"));
            }
            let a = self.mark();
            self.assertion_statement(a)?;
            self.finish(m, Kind::Labeled);
            return Ok(());
        }
        // `intf.mp a, b;` in a module body.
        if self.at_n(1, ".") && self.name_at(2) && self.name_at(3) {
            if self.ansi.last() == Some(&true) {
                return Err(self.error("a module whose header declares its ports cannot declare ports in its body"));
            }
            let t = self.mark();
            self.name()?;
            self.bump();
            self.name()?;
            self.finish(t, Kind::InterfacePortType);
            self.declarators()?;
            self.expect(";")?;
            self.finish(m, Kind::InterfacePortDecl);
            return Ok(());
        }
        // A UDP instance without a name, or with a strength or delay.
        if self.at_n(1, "(") || self.at_n(1, "#") && !self.at_n(2, "(") {
            return self.instantiation(m);
        }
        if let Some(i) = self.skip_type(0)
            && self.name_at(i)
        {
            let mut j = i + 1;
            while self.at_n(j, "[") {
                match self.skip_balanced(j) {
                    Some(k) => j = k,
                    None => break,
                }
            }
            if self.at_n(j, "(") {
                return self.instantiation(m);
            }
            return self.data_declaration();
        }
        Err(self.expected("a declaration or item"))
    }

    /// `module` (or `interface`, `program`) through its end keyword.
    fn design_element(&mut self, m: Mark) -> PResult<'a> {
        let kw = self.tok(0).0;
        let (scope, end) = match kw {
            "interface" => (Scope::Interface, "endinterface"),
            "program" => (Scope::Program, "endprogram"),
            _ => (Scope::Module, "endmodule"),
        };
        self.leaf(Kind::Keyword);
        self.header_rest()?;
        let ansi = self.last_kind() == Some(Kind::Ports)
            && self.ast.children(self.last().unwrap()).first().is_some_and(|&c| self.ast.kind(c) == Kind::AnsiPort);
        self.expect(";")?;
        self.ansi.push(ansi);
        let outer_depth = std::mem::replace(&mut self.generate_depth, 0);
        let r = self.items_until(scope, Some(end));
        self.generate_depth = outer_depth;
        self.ansi.pop();
        r?;
        self.bump();
        self.end_label()?;
        self.finish(m, Kind::Module);
        Ok(())
    }

    /// After `module`: lifetime, name, imports, parameters and ports.
    fn header_rest(&mut self) -> PResult<'a> {
        self.opt_keyword(&["static", "automatic"]);
        self.name()?;
        while self.at("import") {
            self.package_import()?;
        }
        if self.at("#") {
            self.param_ports()?;
        }
        if self.at("(") {
            self.ports()?;
        }
        Ok(())
    }

    /// `extern module ...;` and the like, or an extern prototype in a class
    /// or interface.
    fn extern_item(&mut self, m: Mark) -> PResult<'a> {
        if self.at_any_n(1, &["module", "macromodule", "interface", "program"]) {
            self.bump();
            self.leaf(Kind::Keyword);
            self.header_rest()?;
            self.expect(";")?;
            self.finish(m, Kind::ExternModule);
            return Ok(());
        }
        if self.at_n(1, "primitive") {
            self.bump();
            self.bump();
            self.name()?;
            self.ports()?;
            self.expect(";")?;
            self.finish(m, Kind::ExternModule);
            return Ok(());
        }
        // `extern forkjoin task t;` or `extern function ...;` in an
        // interface.
        self.leaf(Kind::Keyword);
        self.opt_keyword(&["forkjoin"]);
        let q = self.mark();
        self.prototype(q)?;
        self.expect(";")?;
        self.finish(m, Kind::Prototype);
        Ok(())
    }

    fn package(&mut self, m: Mark) -> PResult<'a> {
        self.bump();
        self.opt_keyword(&["static", "automatic"]);
        self.name()?;
        self.expect(";")?;
        self.items_until(Scope::Package, Some("endpackage"))?;
        self.bump();
        self.end_label()?;
        self.finish(m, Kind::Package);
        Ok(())
    }

    fn anonymous_program(&mut self, m: Mark) -> PResult<'a> {
        self.bump();
        self.bump();
        self.items_until(Scope::Program, Some("endprogram"))?;
        self.bump();
        self.finish(m, Kind::AnonymousProgram);
        Ok(())
    }

    fn checker(&mut self, m: Mark) -> PResult<'a> {
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
        let outer_depth = std::mem::replace(&mut self.generate_depth, 0);
        let r = self.items_until(Scope::Checker, Some("endchecker"));
        self.generate_depth = outer_depth;
        r?;
        self.bump();
        self.end_label()?;
        self.finish(m, Kind::Checker);
        Ok(())
    }

    /// `[virtual] class name ...; ... endclass`.
    fn class(&mut self, m: Mark) -> PResult<'a> {
        self.opt_keyword(&["virtual"]);
        self.expect("class")?;
        if self.at(":") && self.at_n(1, "final") {
            self.bump();
            self.leaf(Kind::Keyword);
        }
        self.name()?;
        if self.at("#") {
            self.param_ports()?;
        }
        if self.at("extends") {
            let e = self.mark();
            self.bump();
            self.data_type(false)?;
            if self.at("(") {
                if self.at_n(1, "default") && self.at_n(2, ")") {
                    self.bump();
                    self.leaf(Kind::Keyword);
                    self.bump();
                } else {
                    self.args()?;
                }
            }
            self.finish(e, Kind::Extends);
        }
        if self.at("implements") {
            let i = self.mark();
            self.bump();
            loop {
                self.data_type(false)?;
                if !self.eat(",") {
                    break;
                }
            }
            self.finish(i, Kind::Implements);
        }
        self.expect(";")?;
        self.items_until(Scope::Class, Some("endclass"))?;
        self.bump();
        self.end_label()?;
        self.finish(m, Kind::Class);
        Ok(())
    }

    fn interface_class(&mut self, m: Mark) -> PResult<'a> {
        self.bump();
        self.bump();
        self.name()?;
        if self.at("#") {
            self.param_ports()?;
        }
        if self.at("extends") {
            let e = self.mark();
            self.bump();
            loop {
                self.data_type(false)?;
                if !self.eat(",") {
                    break;
                }
            }
            self.finish(e, Kind::Extends);
        }
        self.expect(";")?;
        self.items_until(Scope::InterfaceClass, Some("endclass"))?;
        self.bump();
        self.end_label()?;
        self.finish(m, Kind::InterfaceClass);
        Ok(())
    }

    /// A class item that starts with qualifiers.
    fn class_item(&mut self, m: Mark) -> PResult<'a> {
        let proto = self.at("extern") || self.at("pure");
        self.qualifiers(CLASS_QUALIFIERS)?;
        if self.at("constraint") {
            return self.constraint_decl(m);
        }
        if proto {
            self.prototype(m)?;
            return self.expect(";");
        }
        if self.at("function") || self.at("task") {
            return self.subroutine(m, true);
        }
        if self.at("covergroup") {
            return self.covergroup();
        }
        if self.at("class") || self.at("virtual") && self.at_n(1, "class") {
            return self.class(m);
        }
        self.var_decl(m)
    }

    /// `import "DPI-C" [context|pure] [c_name =] function ...;`.
    fn dpi_import(&mut self, m: Mark) -> PResult<'a> {
        self.bump();
        self.dpi_string()?;
        self.opt_keyword(&["context", "pure"]);
        if self.name_at(0) && self.at_n(1, "=") {
            self.name()?;
            self.bump();
        }
        let p = self.mark();
        self.prototype(p)?;
        self.expect(";")?;
        self.finish(m, Kind::DpiImport);
        Ok(())
    }

    fn dpi_string(&mut self) -> PResult<'a> {
        let t = self.tok(0).0;
        if t != "\"DPI-C\"" && t != "\"DPI\"" {
            return Err(self.error("expected \"DPI-C\" or \"DPI\""));
        }
        self.leaf(Kind::String);
        Ok(())
    }

    /// `export "DPI-C" [c_name =] function name;`.
    fn dpi_export(&mut self, m: Mark) -> PResult<'a> {
        self.bump();
        self.dpi_string()?;
        if self.name_at(0) && self.at_n(1, "=") {
            self.name()?;
            self.bump();
        }
        if !self.at_any(&["function", "task"]) {
            return Err(self.expected("`function` or `task`"));
        }
        self.leaf(Kind::Keyword);
        self.name()?;
        self.expect(";")?;
        self.finish(m, Kind::DpiExport);
        Ok(())
    }

    /// `name [strength] [#params] instance, ...;`.
    pub(crate) fn instantiation(&mut self, m: Mark) -> PResult<'a> {
        let n = self.mark();
        self.name()?;
        while self.at("::") {
            self.bump();
            self.name()?;
            self.finish(n, Kind::Scope);
        }
        if self.strength_at(0) {
            self.strength()?;
        }
        if self.at("#") {
            if self.at_n(1, "(") {
                self.param_values()?;
            } else {
                self.delay()?;
            }
        }
        loop {
            let i = self.mark();
            if self.name_at(0) {
                self.name()?;
                while self.at("[") {
                    self.variable_dim()?;
                }
            }
            if !self.at("(") {
                return Err(self.error("an instance needs its connections in parentheses"));
            }
            self.connections()?;
            self.finish(i, Kind::Instance);
            if !self.eat(",") {
                break;
            }
        }
        self.expect(";")?;
        self.finish(m, Kind::Instantiation);
        Ok(())
    }

    /// `( ... )` of an instance: all ordered or all named.
    fn connections(&mut self) -> PResult<'a> {
        self.expect("(")?;
        if self.eat(")") {
            return Ok(());
        }
        let mut named = None;
        loop {
            let c = self.mark();
            self.attributes_opt()?;
            let is_named = self.at(".") || self.at(".*");
            if *named.get_or_insert(is_named) != is_named {
                return Err(self.error("ordered and named port connections cannot be mixed"));
            }
            let kind = if is_named { Kind::NamedConnection } else { Kind::Connection };
            if self.at(".*") {
                self.leaf(Kind::Op);
            } else if is_named {
                self.bump();
                {
                    self.name()?;
                    if self.eat("(") {
                        if !self.at(")") {
                            self.arg_or_expr()?;
                        }
                        self.expect(")")?;
                    }
                }
            } else if !self.at(",") && !self.at(")") {
                self.arg_or_expr()?;
            }
            self.finish(c, kind);
            if !self.eat(",") {
                break;
            }
        }
        self.expect(")")
    }

    /// An expression, or for a checker an event, sequence or property.
    fn arg_or_expr(&mut self) -> PResult<'a> {
        if self.at_any(&["posedge", "negedge", "edge"]) {
            return self.event_expr();
        }
        let s = self.save();
        if self.expr().is_ok() && (self.at(",") || self.at(")")) {
            return Ok(());
        }
        self.restore(s);
        self.property_expr()
    }

    /// `assign [strength] [#delay] a = b, c = d;`.
    fn continuous_assign(&mut self, m: Mark) -> PResult<'a> {
        self.bump();
        if self.strength_at(0) {
            self.strength()?;
        }
        if self.at("#") {
            self.delay()?;
        }
        loop {
            let a = self.mark();
            self.unary()?;
            self.expect("=")?;
            self.expr()?;
            self.finish(a, Kind::Assign);
            if !self.eat(",") {
                break;
            }
        }
        self.expect(";")?;
        self.finish(m, Kind::ContinuousAssign);
        Ok(())
    }

    // ---- generate ----

    fn generate_for(&mut self, m: Mark) -> PResult<'a> {
        self.bump();
        self.expect("(")?;
        let i = self.mark();
        self.opt_keyword(&["genvar"]);
        self.name()?;
        self.expect("=")?;
        self.expr()?;
        self.finish(i, Kind::Assign);
        self.expect(";")?;
        self.expr()?;
        self.expect(";")?;
        let s = self.mark();
        self.unary()?;
        if self.assign_op_at() {
            self.leaf(Kind::Op);
            self.expr()?;
            self.finish(s, Kind::Assign);
        }
        self.expect(")")?;
        self.generate_block()?;
        self.finish(m, Kind::GenerateFor);
        Ok(())
    }

    fn generate_if(&mut self, m: Mark) -> PResult<'a> {
        self.bump();
        self.expect("(")?;
        self.expr()?;
        self.expect(")")?;
        self.generate_block()?;
        if self.eat("else") {
            self.generate_block()?;
        }
        self.finish(m, Kind::GenerateIf);
        Ok(())
    }

    fn generate_case(&mut self, m: Mark) -> PResult<'a> {
        self.bump();
        self.expect("(")?;
        self.expr()?;
        self.expect(")")?;
        while !self.at("endcase") {
            if self.at_eof() {
                return Err(self.expected("`endcase`"));
            }
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
            self.generate_block()?;
            self.finish(i, Kind::GenerateCaseItem);
        }
        self.bump();
        self.finish(m, Kind::GenerateCase);
        Ok(())
    }

    /// `[label :] begin [: name] items end [: name]`, or a single item.
    fn generate_block(&mut self) -> PResult<'a> {
        self.generate_depth += 1;
        let r = self.generate_block_inner();
        self.generate_depth -= 1;
        r
    }

    fn generate_block_inner(&mut self) -> PResult<'a> {
        let m = self.mark();
        let is_block = self.at("begin") || self.name_at(0) && self.at_n(1, ":") && self.at_n(2, "begin");
        if !is_block {
            return self.item();
        }
        let mut name = None;
        if self.name_at(0) {
            name = Some(self.tok(0).ident_name());
            self.name()?;
            self.bump();
        }
        self.expect("begin")?;
        if self.at(":") {
            if name.is_some() {
                return Err(self.error("a block cannot have both a label and a name"));
            }
            self.bump();
            name = Some(self.tok(0).ident_name());
            self.name()?;
        }
        self.items_until(self.scope, Some("end"))?;
        self.bump();
        self.check_end_label(name)?;
        self.finish(m, Kind::GenerateBlock);
        Ok(())
    }

    /// `default clocking ...`, or `default disable iff ...;`.
    fn default_item(&mut self, m: Mark) -> PResult<'a> {
        if self.at_n(1, "disable") {
            self.bump();
            self.bump();
            self.expect("iff")?;
            self.expr_or_dist()?;
            self.expect(";")?;
            self.finish(m, Kind::DefaultDisable);
            return Ok(());
        }
        if self.at_n(1, "clocking") && self.name_at(2) && self.at_n(3, ";") {
            self.bump();
            self.bump();
            self.name()?;
            self.bump();
            self.finish(m, Kind::DefaultClocking);
            return Ok(());
        }
        self.clocking(m)
    }

    /// `[default|global] clocking [name] @event; ... endclocking`.
    fn clocking(&mut self, m: Mark) -> PResult<'a> {
        let global = self.at("global");
        self.opt_keyword(&["default", "global"]);
        self.expect("clocking")?;
        if self.name_at(0) {
            self.name()?;
        }
        self.event_control()?;
        self.expect(";")?;
        while !self.at("endclocking") {
            if self.at_eof() {
                return Err(self.expected("`endclocking`"));
            }
            if global {
                return Err(self.expected("`endclocking`"));
            }
            let i = self.mark();
            if self.at("default") {
                self.bump();
                self.clocking_direction()?;
                self.expect(";")?;
                self.finish(i, Kind::ClockingDefault);
            } else if self.at_any(&["input", "output", "inout"]) {
                self.clocking_direction()?;
                loop {
                    let d = self.mark();
                    self.name()?;
                    if self.eat("=") {
                        self.expr()?;
                    }
                    self.finish(d, Kind::Declarator);
                    if !self.eat(",") {
                        break;
                    }
                }
                self.expect(";")?;
                self.finish(i, Kind::ClockingSignals);
            } else if self.at_any(&["property", "sequence"]) {
                self.property_decl()?;
            } else if self.at("let") {
                self.let_decl()?;
            } else {
                return Err(self.expected("a clocking item"));
            }
        }
        self.bump();
        self.end_label()?;
        self.finish(m, Kind::Clocking);
        Ok(())
    }

    /// `input [skew] [output [skew]]`, `output [skew]` or `inout`.
    fn clocking_direction(&mut self) -> PResult<'a> {
        let m = self.mark();
        let inout = self.at("inout");
        if !self.at_any(&["input", "output", "inout"]) {
            return Err(self.expected("`input`, `output` or `inout`"));
        }
        let input = self.at("input");
        self.leaf(Kind::Keyword);
        if !inout {
            self.clocking_skew()?;
            if input && self.at("output") {
                self.leaf(Kind::Keyword);
                self.clocking_skew()?;
            }
        }
        self.finish(m, Kind::ClockingDirection);
        Ok(())
    }

    fn clocking_skew(&mut self) -> PResult<'a> {
        if !self.at_any(&["posedge", "negedge", "edge", "#"]) {
            return Ok(());
        }
        let m = self.mark();
        self.opt_keyword(&["posedge", "negedge", "edge"]);
        if self.at("#") {
            self.delay()?;
        }
        self.finish(m, Kind::ClockingSkew);
        Ok(())
    }

    /// `modport name (input a, output b, import f), ...;`.
    fn modport(&mut self, m: Mark) -> PResult<'a> {
        self.bump();
        let group_kw = ["input", "output", "inout", "ref", "import", "export", "clocking"];
        loop {
            let i = self.mark();
            self.name()?;
            self.expect("(")?;
            loop {
                let g = self.mark();
                self.attributes_opt()?;
                if !self.at_any(&group_kw) {
                    return Err(self.expected("a direction, `import`, `export` or `clocking`"));
                }
                let tf = self.at("import") || self.at("export");
                self.leaf(Kind::Keyword);
                loop {
                    let p = self.mark();
                    if tf && (self.at("function") || self.at("task")) {
                        self.prototype(p)?;
                    } else if self.at(".") {
                        self.named_paren_expr()?;
                        self.finish(p, Kind::ModportPort);
                    } else {
                        self.name()?;
                        self.finish(p, Kind::ModportPort);
                    }
                    if self.at(",") && !self.at_any_n(1, &group_kw) && !self.at_n(1, "(") {
                        self.bump();
                    } else {
                        break;
                    }
                }
                self.finish(g, Kind::ModportPorts);
                if !self.eat(",") {
                    break;
                }
            }
            self.expect(")")?;
            self.finish(i, Kind::ModportItem);
            if !self.eat(",") {
                break;
            }
        }
        self.expect(";")?;
        self.finish(m, Kind::ModportDecl);
        Ok(())
    }

    /// `bind target [: instances] instantiation`.
    fn bind(&mut self, m: Mark) -> PResult<'a> {
        self.bump();
        self.postfix()?;
        if self.eat(":") {
            loop {
                self.postfix()?;
                if !self.eat(",") {
                    break;
                }
            }
        }
        let i = self.mark();
        self.instantiation(i)?;
        self.finish(m, Kind::Bind);
        Ok(())
    }

    /// `timeunit 1ns [/ 1ps];` or `timeprecision 1ps;`.
    fn timeunits(&mut self, m: Mark) -> PResult<'a> {
        let unit = self.at("timeunit");
        self.leaf(Kind::Keyword);
        self.time_literal()?;
        if unit && self.eat("/") {
            self.time_literal()?;
        }
        self.expect(";")?;
        self.finish(m, Kind::Timeunits);
        Ok(())
    }

    /// A time literal whose number is 1, 10 or 100, for a time unit.
    fn time_literal(&mut self) -> PResult<'a> {
        let t = self.tok(0).0;
        let digits = t.bytes().take_while(|c| c.is_ascii_digit()).count();
        let unit = &t[digits..];
        if !t.is_empty() && unit == "step" {
            return Err(self.error("1step is not a time unit"));
        }
        if !matches!(unit, "s" | "ms" | "us" | "ns" | "ps" | "fs") {
            return Err(self.expected("a time literal such as 1ns"));
        }
        if !matches!(&t[..digits], "1" | "10" | "100") {
            return Err(self.error("a time unit must be 1, 10 or 100 of s, ms, us, ns, ps or fs"));
        }
        self.leaf(Kind::TimeLiteral);
        Ok(())
    }
}
