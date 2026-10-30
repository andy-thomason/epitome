//! Data types and declarations (A.2), ports and parameters (A.1.3).

use super::{Mark, PResult, Parser};
use crate::ast::Kind;

pub(crate) const NET_TYPES: &[&str] =
    &["supply0", "supply1", "tri", "triand", "trior", "trireg", "tri0", "tri1", "uwire", "wire", "wand", "wor"];

pub(crate) const DIRECTIONS: &[&str] = &["input", "output", "inout", "ref"];

const STRENGTHS: &[&str] = &[
    "supply0", "strong0", "pull0", "weak0", "highz0", "supply1", "strong1", "pull1", "weak1", "highz1", "small",
    "medium", "large",
];

const INTEGER_VECTOR: &[&str] = &["bit", "logic", "reg"];
const INTEGER_ATOM: &[&str] = &["byte", "shortint", "int", "longint", "integer", "time"];
const OTHER_TYPES: &[&str] = &["shortreal", "real", "realtime", "string", "chandle", "event"];

impl<'a> Parser<'a> {
    // ---- looking ahead ----

    /// If a data type starts at token `i`, the index just after it.
    pub(crate) fn skip_type(&self, mut i: usize) -> Option<usize> {
        let t = |n: usize| self.tok(n).0;
        let kw = |n: usize, ss: &[&str]| ss.iter().any(|s| self.at_n(n, s));
        if kw(i, &["struct", "union", "enum"]) {
            while !self.at_n(i, "{") {
                if self.tok(i).is_eof() || self.at_n(i, ";") {
                    return None;
                }
                i += 1;
            }
            i = self.skip_balanced(i)?;
        } else if kw(i, INTEGER_VECTOR) || kw(i, INTEGER_ATOM) || kw(i, OTHER_TYPES) {
            i += 1;
        } else if self.at_n(i, "virtual") {
            i += 1;
            if self.at_n(i, "interface") {
                i += 1;
            }
            if !self.name_at(i) {
                return None;
            }
            i += 1;
            if t(i) == "#" && t(i + 1) == "(" {
                i = self.skip_balanced(i + 1)?;
            }
            if t(i) == "." {
                i += 2;
            }
            return Some(i);
        } else if self.at_n(i, "type") && t(i + 1) == "(" {
            i = self.skip_balanced(i + 1)?;
        } else if self.name_at(i) || t(i) == "$unit" || self.at_n(i, "local") && t(i + 1) == "::" {
            i += 1;
            loop {
                if t(i) == "#" && t(i + 1) == "(" {
                    i = self.skip_balanced(i + 1)?;
                } else if t(i) == "::" && self.tok(i + 1).is_ident() {
                    i += 2;
                } else {
                    break;
                }
            }
        } else {
            return None;
        }
        if kw(i, &["signed", "unsigned"]) {
            i += 1;
        }
        while t(i) == "[" {
            i = self.skip_balanced(i)?;
        }
        Some(i)
    }

    /// Whether a data type followed by a declared name starts at token `n`:
    /// `T x;`, `p::T #(8) x = 1;`, `int x[3];`.
    pub(crate) fn type_then_name_at(&self, n: usize) -> bool {
        match self.skip_type(n) {
            Some(i) if i > n => self.name_at(i),
            _ => false,
        }
    }

    /// Whether a variable declaration starts here.
    pub(crate) fn var_decl_at(&self) -> bool {
        if self.at_any(&["const", "var", "static", "automatic", "typedef", "rand", "randc"]) {
            return true;
        }
        if (self.type_keyword_at(0) || self.at("type")) && !self.at_n(1, "'") {
            return !self.at("virtual") || !self.at_n(1, "function") && !self.at_n(1, "task") && !self.at_n(1, "class");
        }
        self.type_then_name_at(0) && {
            let i = self.skip_type(0).unwrap();
            matches!(self.tok(i + 1).0, ";" | "=" | "," | "[")
        }
    }

    // ---- data types ----

    pub(crate) fn signing_opt(&mut self) {
        self.opt_keyword(&["signed", "unsigned"]);
    }

    /// A data type; with `implicit`, also `[signing] {packed_dimension}`
    /// alone, which may be empty.
    pub(crate) fn data_type(&mut self, implicit: bool) -> PResult<'a> {
        let m = self.mark();
        if self.at_any(INTEGER_VECTOR) {
            self.leaf(Kind::Keyword);
            self.signing_opt();
            self.packed_dims()?;
            self.finish(m, Kind::BuiltinType);
        } else if self.at_any(INTEGER_ATOM) || self.at_any(OTHER_TYPES) {
            self.leaf(Kind::Keyword);
            self.signing_opt();
            self.finish(m, Kind::BuiltinType);
        } else if self.at_any(&["struct", "union"]) {
            self.struct_type()?;
        } else if self.at("enum") {
            self.enum_type()?;
        } else if self.at("virtual") {
            self.bump();
            self.eat("interface");
            self.name()?;
            if self.at("#") {
                self.param_values()?;
            }
            if self.eat(".") {
                self.name()?;
            }
            self.finish(m, Kind::VirtualInterfaceType);
        } else if self.at("type") {
            self.type_reference()?;
        } else if self.at("void") {
            self.leaf(Kind::VoidType);
        } else if (self.name_at(0) || self.tok(0).0 == "$unit" || self.at("local") && self.at_n(1, "::"))
            && (!implicit || self.type_then_name_at(0))
        {
            let n = self.mark();
            if self.name_at(0) {
                self.leaf(Kind::Name);
            } else {
                self.leaf(Kind::ScopeKeyword);
            }
            loop {
                if self.at("#") && self.at_n(1, "(") {
                    self.param_values()?;
                    self.finish(n, Kind::Specialize);
                } else if self.at("::") && self.tok(1).is_ident() {
                    self.bump();
                    self.name()?;
                    self.finish(n, Kind::Scope);
                } else {
                    break;
                }
            }
            self.packed_dims()?;
            self.finish(m, Kind::NamedType);
        } else if implicit {
            self.signing_opt();
            self.packed_dims()?;
            self.finish(m, Kind::ImplicitType);
        } else {
            return Err(self.expected("a data type"));
        }
        Ok(())
    }

    /// `{ [a:b] | [] }`: a packed dimension must be a range.
    pub(crate) fn packed_dims(&mut self) -> PResult<'a> {
        while self.at("[") {
            let m = self.mark();
            self.bump();
            if self.eat("]") {
                self.finish(m, Kind::UnsizedDim);
                continue;
            }
            self.expr()?;
            if !self.eat(":") {
                return Err(self.error("a packed dimension needs a range, such as [7:0]"));
            }
            self.expr()?;
            self.expect("]")?;
            self.finish(m, Kind::RangeDim);
        }
        Ok(())
    }

    /// One unpacked (variable) dimension: `[a:b]`, `[n]`, `[]`, `[*]`,
    /// `[type]`, `[$]`, `[$:n]`.
    pub(crate) fn variable_dim(&mut self) -> PResult<'a> {
        let m = self.mark();
        self.expect("[")?;
        if self.eat("]") {
            self.finish(m, Kind::UnsizedDim);
        } else if self.at("*") && self.at_n(1, "]") {
            self.bump();
            self.bump();
            self.finish(m, Kind::AssocDim);
        } else if self.at("$") {
            self.bump();
            if self.eat(":") {
                self.expr()?;
            }
            self.expect("]")?;
            self.finish(m, Kind::QueueDim);
        } else if self.type_keyword_at(0) {
            self.data_type(false)?;
            self.expect("]")?;
            self.finish(m, Kind::AssocDim);
        } else {
            self.expr()?;
            if self.eat(":") {
                self.expr()?;
                self.expect("]")?;
                self.finish(m, Kind::RangeDim);
            } else {
                self.expect("]")?;
                self.finish(m, Kind::SizeDim);
            }
        }
        Ok(())
    }

    fn struct_type(&mut self) -> PResult<'a> {
        let m = self.mark();
        let is_union = self.at("union");
        self.leaf(Kind::Keyword);
        let mut packed = is_union && self.at("soft");
        if is_union {
            self.opt_keyword(&["tagged", "soft"]);
        }
        if self.at("packed") {
            self.leaf(Kind::Keyword);
            packed = true;
            self.signing_opt();
        } else if self.at_any(&["signed", "unsigned"]) {
            return Err(self.error("only a packed struct or union can be signed or unsigned"));
        }
        self.expect("{")?;
        while !self.at("}") {
            let mm = self.mark();
            self.attributes_opt()?;
            if self.at_any(&["rand", "randc"]) {
                if packed {
                    return Err(self.error("a member of a packed struct or union cannot be rand or randc"));
                }
                self.leaf(Kind::Keyword);
            }
            self.data_type(false)?;
            self.declarators()?;
            self.expect(";")?;
            self.finish(mm, Kind::StructMember);
        }
        self.expect("}")?;
        if self.at("[") && !packed {
            return Err(self.error("packed dimensions need a packed struct or union"));
        }
        self.packed_dims()?;
        self.finish(m, Kind::StructType);
        Ok(())
    }

    fn enum_type(&mut self) -> PResult<'a> {
        let m = self.mark();
        self.bump();
        if !self.at("{") {
            self.data_type(false)?;
        }
        self.expect("{")?;
        loop {
            let e = self.mark();
            self.name()?;
            if self.eat("[") {
                self.expr()?;
                if self.eat(":") {
                    self.expr()?;
                }
                self.expect("]")?;
            }
            if self.eat("=") {
                self.expr()?;
            }
            self.finish(e, Kind::EnumItem);
            if !self.eat(",") {
                break;
            }
        }
        self.expect("}")?;
        self.packed_dims()?;
        self.finish(m, Kind::EnumType);
        Ok(())
    }

    // ---- declarations ----

    /// `name {dim} [= value]`.
    pub(crate) fn declarator(&mut self) -> PResult<'a> {
        let m = self.mark();
        self.name()?;
        while self.at("[") {
            self.variable_dim()?;
        }
        if self.eat("=") {
            self.expr()?;
        }
        self.finish(m, Kind::Declarator);
        Ok(())
    }

    /// Whether a declarator has `= value`.
    pub(crate) fn declarator_has_value(&self, d: crate::ast::NodeId) -> bool {
        self.ast.tokens_of(d).iter().any(|t| t.0 == "=")
    }

    pub(crate) fn declarators(&mut self) -> PResult<'a> {
        loop {
            self.declarator()?;
            if !self.eat(",") {
                return Ok(());
            }
        }
    }

    /// Qualifiers before a declaration. In a class, `local` and
    /// `protected` cannot go together.
    pub(crate) fn qualifiers(&mut self, allowed: &[&str]) -> PResult<'a> {
        let m = self.mark();
        let mut seen: Vec<&str> = Vec::new();
        while self.at_any(allowed) && !(self.at("virtual") && !self.at_n(1, "function") && !self.at_n(1, "task")) {
            let t = self.tok(0).0;
            if seen.contains(&t) {
                return Err(self.error(format!("`{t}` is given twice")));
            }
            if (t == "local" && seen.contains(&"protected")) || (t == "protected" && seen.contains(&"local")) {
                return Err(self.error("a member cannot be both local and protected"));
            }
            seen.push(t);
            self.leaf(Kind::Keyword);
        }
        if !seen.is_empty() {
            self.finish(m, Kind::Qualifiers);
        }
        Ok(())
    }

    /// A variable declaration, after any qualifiers (pushed since `m`).
    pub(crate) fn var_decl(&mut self, m: Mark) -> PResult<'a> {
        let has_var = self.since(m).iter().any(|&q| self.ast.text(q).split(' ').any(|w| w == "var"));
        // 6.23 writes `static type(this) x;` without `var`.
        if self.at("type") && !has_var && !(self.at_n(2, "this") && self.at_n(3, ")")) {
            return Err(self.error("a variable declared with a type reference needs `var` before it"));
        }
        let t = self.mark();
        self.data_type(true)?;
        if self.ast.kind(self.last().unwrap()) == Kind::ImplicitType && !has_var {
            return Err(self.error_at_mark(t, "this declaration needs a data type, or `var`"));
        }
        self.declarators()?;
        self.expect(";")?;
        self.finish(m, Kind::VarDecl);
        Ok(())
    }

    pub(crate) fn error_at_mark(&self, m: Mark, msg: &str) -> crate::source::Error<'a> {
        let t = self.ast.tokens.get(m.pos).copied().unwrap_or(self.eof);
        crate::source::Error::new(t.0, msg)
    }

    /// A data declaration with its qualifiers: `const int x = 1;`.
    pub(crate) fn data_declaration(&mut self) -> PResult<'a> {
        let m = self.mark();
        if self.at("typedef") {
            return self.typedef();
        }
        if self.at("import") {
            return self.package_import();
        }
        if self.at("nettype") {
            return self.nettype_decl();
        }
        self.qualifiers(&["const", "var", "static", "automatic"])?;
        self.var_decl(m)
    }

    /// `typedef`.
    pub(crate) fn typedef(&mut self) -> PResult<'a> {
        let m = self.mark();
        self.expect("typedef")?;
        // `typedef enum T;`, `typedef class C;`, `typedef T;`
        let forward = ["enum", "struct", "union", "class"];
        if self.at_any(&forward) && self.name_at(1) && self.at_n(2, ";") {
            self.leaf(Kind::Keyword);
            self.name()?;
            self.expect(";")?;
            self.finish(m, Kind::ForwardTypedef);
            return Ok(());
        }
        if self.at("interface") && self.at_n(1, "class") {
            self.bump();
            self.leaf(Kind::Keyword);
            self.name()?;
            self.expect(";")?;
            self.finish(m, Kind::ForwardTypedef);
            return Ok(());
        }
        if self.name_at(0) && self.at_n(1, ";") {
            self.name()?;
            self.bump();
            self.finish(m, Kind::ForwardTypedef);
            return Ok(());
        }
        // `typedef intf.T name;` (an interface's type): read as a named
        // type with a member.
        if self.name_at(0) && (self.at_n(1, ".") || self.at_n(1, "[")) && !self.type_then_name_at(0) {
            let n = self.mark();
            self.postfix()?;
            self.finish(n, Kind::NamedType);
        } else {
            self.data_type(false)?;
        }
        self.name()?;
        while self.at("[") {
            self.variable_dim()?;
        }
        self.expect(";")?;
        self.finish(m, Kind::Typedef);
        Ok(())
    }

    /// `nettype T name [with f];` or `nettype n other;`.
    fn nettype_decl(&mut self) -> PResult<'a> {
        let m = self.mark();
        self.bump();
        self.data_type(false)?;
        self.name()?;
        if self.eat("with") {
            self.postfix()?;
        }
        self.expect(";")?;
        self.finish(m, Kind::NettypeDecl);
        Ok(())
    }

    /// `import p::x, q::*;`.
    pub(crate) fn package_import(&mut self) -> PResult<'a> {
        let m = self.mark();
        self.expect("import")?;
        loop {
            self.import_item()?;
            if !self.eat(",") {
                break;
            }
        }
        self.expect(";")?;
        self.finish(m, Kind::PackageImport);
        Ok(())
    }

    fn import_item(&mut self) -> PResult<'a> {
        let m = self.mark();
        if self.tok(0).0 == "$unit" {
            return Err(self.error("$unit cannot be imported"));
        }
        self.name()?;
        self.expect("::")?;
        if self.at("*") {
            self.leaf(Kind::Op);
        } else {
            self.name()?;
        }
        self.finish(m, Kind::ImportItem);
        Ok(())
    }

    /// `export p::x;` or `export *::*;`.
    pub(crate) fn package_export(&mut self) -> PResult<'a> {
        let m = self.mark();
        self.expect("export")?;
        if self.at("*") && self.at_n(1, "::") && self.at_n(2, "*") {
            self.bump();
            self.bump();
            self.bump();
        } else {
            loop {
                self.import_item()?;
                if !self.eat(",") {
                    break;
                }
            }
        }
        self.expect(";")?;
        self.finish(m, Kind::PackageExport);
        Ok(())
    }

    // ---- nets ----

    pub(crate) fn strength_at(&self, n: usize) -> bool {
        self.at_n(n, "(") && STRENGTHS.iter().any(|s| self.at_n(n + 1, s))
    }

    /// `(strong0, weak1)`, `(pull1)`, `(small)`.
    pub(crate) fn strength(&mut self) -> PResult<'a> {
        let m = self.mark();
        self.expect("(")?;
        let mut highz = 0;
        loop {
            if !self.at_any(STRENGTHS) {
                return Err(self.expected("a strength"));
            }
            if self.at_any(&["highz0", "highz1"]) {
                highz += 1;
            }
            self.leaf(Kind::Keyword);
            if !self.eat(",") {
                break;
            }
        }
        if highz > 1 {
            return Err(self.error_at_mark(m, "a strength cannot be high impedance for both 0 and 1"));
        }
        self.expect(")")?;
        self.finish(m, Kind::Strength);
        Ok(())
    }

    /// A net declaration: `wire [7:0] #1 a = b, c;`.
    pub(crate) fn net_decl(&mut self) -> PResult<'a> {
        let m = self.mark();
        let trireg = self.at("trireg");
        self.leaf(Kind::Keyword);
        let mut drive = false;
        if self.strength_at(0) {
            let charge = self.at_n(1, "small") || self.at_n(1, "medium") || self.at_n(1, "large");
            if charge && !trireg {
                return Err(self.error("a charge strength is only for trireg nets"));
            }
            drive = !charge;
            self.strength()?;
        }
        let vectored = self.opt_keyword(&["vectored", "scalared"]);
        if self.at("reg") {
            return Err(self.error("a net cannot have the type reg"));
        }
        let t = self.mark();
        if self.type_then_name_at(0) || self.type_keyword_at(0) {
            self.data_type(false)?;
        } else {
            self.data_type(true)?;
        }
        if vectored && !self.since(t).iter().any(|&n| self.ast.child(n, Kind::RangeDim).is_some()) {
            return Err(self.error_at_mark(t, "vectored and scalared need a packed dimension"));
        }
        if self.at("#") {
            self.delay()?;
        }
        let d = self.mark();
        self.declarators()?;
        if drive && self.since(d).iter().any(|&n| !self.declarator_has_value(n)) {
            return Err(self.error_at_mark(m, "a drive strength needs an assignment in the declaration"));
        }
        self.expect(";")?;
        self.finish(m, Kind::NetDecl);
        Ok(())
    }

    /// `interconnect [signing] [dims] [#d] a [dims], b;`.
    pub(crate) fn interconnect_decl(&mut self) -> PResult<'a> {
        let m = self.mark();
        self.bump();
        if self.type_keyword_at(0) || self.type_then_name_at(0) {
            return Err(self.error("an interconnect net has no data type"));
        }
        if self.strength_at(0) {
            return Err(self.error("an interconnect net has no strength"));
        }
        self.data_type(true)?;
        if self.at("#") {
            if self.at_n(1, "(") && self.skip_balanced(1).is_some_and(|j| (2..j).any(|i| self.at_n(i, ","))) {
                return Err(self.error("an interconnect net has only one delay"));
            }
            self.delay()?;
        }
        loop {
            let d = self.mark();
            self.name()?;
            while self.at("[") {
                self.variable_dim()?;
            }
            if self.at("=") {
                return Err(self.error("an interconnect net cannot be assigned in its declaration"));
            }
            self.finish(d, Kind::Declarator);
            if !self.eat(",") {
                break;
            }
        }
        self.expect(";")?;
        self.finish(m, Kind::InterconnectDecl);
        Ok(())
    }

    // ---- parameters ----

    /// `#( ... )` as values: ordered or named, not both.
    pub(crate) fn param_values(&mut self) -> PResult<'a> {
        let m = self.mark();
        self.expect("#")?;
        self.expect("(")?;
        if self.eat(")") {
            self.finish(m, Kind::ParamValues);
            return Ok(());
        }
        let named = self.at(".");
        loop {
            if self.at(".") {
                if !named {
                    return Err(self.error("ordered and named parameter values cannot be mixed"));
                }
                let n = self.mark();
                self.bump();
                self.name()?;
                self.expect("(")?;
                if !self.at(")") {
                    self.param_value()?;
                }
                self.expect(")")?;
                self.finish(n, Kind::NamedArg);
            } else {
                if named {
                    return Err(self.error("ordered and named parameter values cannot be mixed"));
                }
                self.param_value()?;
            }
            if !self.eat(",") {
                break;
            }
        }
        self.expect(")")?;
        self.finish(m, Kind::ParamValues);
        Ok(())
    }

    /// A parameter's value: an expression, a data type or `$`.
    pub(crate) fn param_value(&mut self) -> PResult<'a> {
        if self.at("$") && (self.at_n(1, ")") || self.at_n(1, ",")) {
            self.leaf(Kind::Dollar);
            return Ok(());
        }
        if (self.type_keyword_at(0) || self.at_any(&["signed", "unsigned"])) && !self.at_n(1, "'") {
            return self.type_expr();
        }
        if self.type_then_name_at(0) {
            return Err(self.expected("a value"));
        }
        let s = self.save();
        if self.mintypmax().is_ok() && (self.at(",") || self.at(")") || self.at(";")) {
            return Ok(());
        }
        self.restore(s);
        self.type_expr()
    }

    /// `#( ... )` on a declaration.
    pub(crate) fn param_ports(&mut self) -> PResult<'a> {
        let m = self.mark();
        self.expect("#")?;
        self.expect("(")?;
        if self.eat(")") {
            self.finish(m, Kind::ParamPorts);
            return Ok(());
        }
        loop {
            self.param_decl(true)?;
            if !self.eat(",") {
                break;
            }
        }
        self.expect(")")?;
        self.finish(m, Kind::ParamPorts);
        Ok(())
    }

    /// `parameter`, `localparam` or (in a port list) a type or name alone,
    /// with its assignments. Stops before a `,` that starts another
    /// declaration.
    pub(crate) fn param_decl(&mut self, in_ports: bool) -> PResult<'a> {
        let m = self.mark();
        let local = self.at("localparam");
        self.opt_keyword(&["parameter", "localparam"]);
        if self.at("type") && !self.at_n(1, "(") {
            self.bump();
            if self.at("interface") && self.at_n(1, "class") {
                self.bump();
                self.leaf(Kind::Keyword);
            } else {
                self.opt_keyword(&["enum", "struct", "union", "class"]);
            }
            loop {
                let a = self.mark();
                self.name()?;
                if self.eat("=") {
                    self.data_type(false)?;
                } else if local && in_ports {
                    return Err(self.error("a localparam in a parameter port list needs a value"));
                }
                self.finish(a, Kind::TypeAssign);
                if !(self.at(",") && self.name_at(1) && !self.type_then_name_at(1)) {
                    break;
                }
                self.bump();
            }
            self.finish(m, Kind::TypeParamDecl);
            return Ok(());
        }
        if self.type_then_name_at(0) {
            self.data_type(false)?;
        } else {
            self.data_type(true)?;
        }
        loop {
            let a = self.mark();
            self.name()?;
            while self.at("[") {
                self.variable_dim()?;
            }
            if self.eat("=") {
                self.param_value()?;
            } else if local && in_ports {
                return Err(self.error("a localparam in a parameter port list needs a value"));
            }
            self.finish(a, Kind::ParamAssign);
            // Another name for the same declaration, or a new declaration?
            if !(self.at(",") && self.name_at(1) && matches!(self.tok(2).0, "=" | "," | ")" | ";" | "[")) {
                break;
            }
            self.bump();
        }
        self.finish(m, Kind::ParamDecl);
        Ok(())
    }

    /// `specparam [range] a = 1, b = 2;`.
    pub(crate) fn specparam_decl(&mut self) -> PResult<'a> {
        let m = self.mark();
        self.bump();
        self.packed_dims()?;
        loop {
            let a = self.mark();
            if self.tok(0).0.starts_with("PATHPULSE$") {
                self.leaf(Kind::Name);
            } else {
                self.name()?;
            }
            self.expect("=")?;
            if self.at("(") {
                // A pulse limit pair.
                self.bump();
                self.mintypmax()?;
                if self.eat(",") {
                    self.mintypmax()?;
                }
                self.expect(")")?;
            } else {
                self.mintypmax()?;
            }
            self.finish(a, Kind::SpecparamAssign);
            if !self.eat(",") {
                break;
            }
        }
        self.expect(";")?;
        self.finish(m, Kind::SpecparamDecl);
        Ok(())
    }

    // ---- ports ----

    /// `( ... )` after a module, interface or program name.
    pub(crate) fn ports(&mut self) -> PResult<'a> {
        let m = self.mark();
        self.expect("(")?;
        if self.eat(")") {
            self.finish(m, Kind::Ports);
            return Ok(());
        }
        if self.at(".*") {
            self.bump();
            self.expect(")")?;
            self.finish(m, Kind::Ports);
            return Ok(());
        }
        let ansi = self.ansi_port_at();
        loop {
            if ansi {
                self.ansi_port()?
            } else {
                self.port()?
            }
            if !self.eat(",") {
                break;
            }
        }
        self.expect(")")?;
        self.finish(m, Kind::Ports);
        Ok(())
    }

    /// Whether the first port is written ANSI style (with a direction or
    /// type) rather than as a bare name.
    fn ansi_port_at(&mut self) -> bool {
        self.lookahead(|p| {
            if p.attributes_opt().is_err() {
                return false;
            }
            p.at_any(DIRECTIONS)
                || p.at_any(NET_TYPES)
                || p.at_any(&["var", "interface", "interconnect", "signed", "unsigned"])
                || p.type_keyword_at(0)
                || p.at("[")
                || p.type_then_name_at(0)
                || p.name_at(0) && p.at_n(1, ".") && p.name_at(2) && p.name_at(3)
        })
    }

    /// A non-ANSI port: `a`, `a[3:0]`, `{a, b}`, `.x(a)`, or nothing.
    fn port(&mut self) -> PResult<'a> {
        let m = self.mark();
        if self.at(".") {
            self.named_paren_expr()?;
        } else if !self.at(",") && !self.at(")") {
            self.expr()?;
        }
        self.finish(m, Kind::Port);
        Ok(())
    }

    /// `.name(expr)`, as a NamedArg.
    pub(crate) fn named_paren_expr(&mut self) -> PResult<'a> {
        let m = self.mark();
        self.expect(".")?;
        self.name()?;
        self.expect("(")?;
        if !self.at(")") {
            self.expr()?;
        }
        self.expect(")")?;
        self.finish(m, Kind::NamedArg);
        Ok(())
    }

    /// An ANSI port.
    fn ansi_port(&mut self) -> PResult<'a> {
        let m = self.mark();
        self.attributes_opt()?;
        self.opt_keyword(DIRECTIONS);
        if self.at(".") {
            self.named_paren_expr()?;
            self.finish(m, Kind::AnsiPort);
            return Ok(());
        }
        if self.at("interface") || self.name_at(0) && self.at_n(1, ".") && self.name_at(2) && self.name_at(3) {
            let t = self.mark();
            if self.at("interface") {
                self.leaf(Kind::Keyword);
            } else {
                self.name()?;
            }
            if self.eat(".") {
                self.name()?;
            }
            self.finish(t, Kind::InterfacePortType);
        } else {
            if self.at_any(NET_TYPES) || self.at_any(&["var", "interconnect"]) {
                let net = !self.at("var");
                self.leaf(Kind::Keyword);
                if net && self.at("reg") {
                    return Err(self.error("a net cannot have the type reg"));
                }
            }
            if self.type_then_name_at(0) {
                self.data_type(false)?;
            } else {
                self.data_type(true)?;
            }
        }
        self.declarator()?;
        self.finish(m, Kind::AnsiPort);
        Ok(())
    }

    /// `input [7:0] a, b;` in a module body.
    pub(crate) fn port_decl(&mut self) -> PResult<'a> {
        let m = self.mark();
        self.attributes_opt()?;
        self.opt_keyword(DIRECTIONS);
        if self.at_any(NET_TYPES) || self.at_any(&["var", "interconnect"]) {
            self.leaf(Kind::Keyword);
        }
        if self.type_then_name_at(0) {
            self.data_type(false)?;
        } else {
            self.data_type(true)?;
        }
        self.declarators()?;
        self.expect(";")?;
        self.finish(m, Kind::PortDecl);
        Ok(())
    }

    // ---- functions and tasks ----

    /// `function ...` or `task ...` with a body. `m` marks any qualifiers
    /// already pushed; `in_class` forbids a static lifetime.
    pub(crate) fn subroutine(&mut self, m: Mark, in_class: bool) -> PResult<'a> {
        let is_function = self.at("function");
        self.bump();
        self.dynamic_override()?;
        if self.at("static") && in_class {
            return Err(self.error("a class method cannot have a static lifetime"));
        }
        self.opt_keyword(&["static", "automatic"]);
        if is_function {
            self.return_type_and_name()?;
        } else {
            self.subroutine_name()?;
        }
        if self.at("(") {
            self.tf_ports()?;
        }
        self.expect(";")?;
        let end = if is_function { "endfunction" } else { "endtask" };
        self.block_body(end, true)?;
        self.expect(end)?;
        self.end_label()?;
        self.finish(m, if is_function { Kind::Function } else { Kind::Task });
        Ok(())
    }

    /// `:initial`, `:extends`, `:final` after `function`, `task` or
    /// `constraint`.
    pub(crate) fn dynamic_override(&mut self) -> PResult<'a> {
        while self.at(":") && self.at_any_n(1, &["initial", "extends", "final"]) {
            self.bump();
            self.leaf(Kind::Keyword);
        }
        Ok(())
    }

    pub(crate) fn at_any_n(&self, n: usize, ss: &[&str]) -> bool {
        ss.iter().any(|s| self.at_n(n, s))
    }

    /// A function's return type (perhaps implicit) and name.
    fn return_type_and_name(&mut self) -> PResult<'a> {
        let scoped_name_then = |p: &Self, mut i: usize| {
            // name {:: name | . name} followed by ( or ;
            if !(p.name_at(i) || p.at_n(i, "new")) {
                return false;
            }
            i += 1;
            while (p.at_n(i, "::") || p.at_n(i, ".")) && (p.tok(i + 1).is_ident()) {
                i += 2;
            }
            p.at_n(i, "(") || p.at_n(i, ";")
        };
        if scoped_name_then(self, 0) {
            self.empty();
        } else if self.at("void") {
            self.leaf(Kind::VoidType);
        } else {
            self.data_type(true)?;
        }
        self.subroutine_name()
    }

    /// `name`, `new`, `C::name`, `intf.name`.
    fn subroutine_name(&mut self) -> PResult<'a> {
        let m = self.mark();
        if self.at("new") {
            self.leaf(Kind::Name);
        } else {
            self.name()?;
        }
        let mut scoped = false;
        while (self.at("::") || self.at(".")) && self.tok(1).is_ident() {
            self.bump();
            self.leaf(Kind::Name);
            scoped = true;
        }
        if scoped {
            self.finish(m, Kind::Scope);
        }
        Ok(())
    }

    /// `( ports )` of a function or task.
    pub(crate) fn tf_ports(&mut self) -> PResult<'a> {
        let m = self.mark();
        self.expect("(")?;
        if self.eat(")") {
            self.finish(m, Kind::TfPorts);
            return Ok(());
        }
        loop {
            let p = self.mark();
            self.attributes_opt()?;
            if self.at("default") {
                self.leaf(Kind::Keyword);
            } else {
                if self.at("const") && self.at_n(1, "ref") {
                    self.leaf(Kind::Keyword);
                }
                if self.opt_keyword(&["ref"]) {
                    if self.at_any(&["input", "output", "inout"]) {
                        return Err(self.error("ref cannot be combined with a direction"));
                    }
                    self.opt_keyword(&["static"]);
                } else {
                    self.opt_keyword(&["input", "output", "inout"]);
                }
                self.opt_keyword(&["var"]);
                if self.name_at(0) && matches!(self.tok(1).0, "," | ")" | "=" | "[") {
                    self.empty();
                } else {
                    self.data_type(true)?;
                }
                if self.name_at(0) {
                    self.declarator()?;
                }
            }
            self.finish(p, Kind::TfPort);
            if !self.eat(",") {
                break;
            }
        }
        self.expect(")")?;
        self.finish(m, Kind::TfPorts);
        Ok(())
    }

    /// A prototype: `function int f(int a)` or `task t(...)`, without a
    /// body. `m` marks any qualifiers.
    pub(crate) fn prototype(&mut self, m: Mark) -> PResult<'a> {
        let is_function = self.at("function");
        if !is_function && !self.at("task") {
            return Err(self.expected("`function` or `task`"));
        }
        self.leaf(Kind::Keyword);
        self.dynamic_override()?;
        if is_function {
            if self.name_at(0) && (self.at_n(1, "(") || self.at_n(1, ";") || self.at_n(1, ",")) || self.at("new") {
                if !self.at("new") {
                    return Err(self.error("a function prototype needs a return type"));
                }
                self.empty();
            } else if self.at("void") {
                self.leaf(Kind::VoidType);
            } else {
                self.data_type(false)?;
            }
        }
        self.subroutine_name()?;
        if self.at("(") {
            self.tf_ports()?;
        }
        self.finish(m, Kind::Prototype);
        Ok(())
    }

    /// `input int a, b;` in a function or task body.
    pub(crate) fn tf_port_decl(&mut self) -> PResult<'a> {
        let m = self.mark();
        if self.at("const") && self.at_n(1, "ref") {
            self.leaf(Kind::Keyword);
        }
        self.opt_keyword(DIRECTIONS);
        self.opt_keyword(&["var"]);
        if self.type_then_name_at(0) {
            self.data_type(false)?;
        } else {
            self.data_type(true)?;
        }
        self.declarators()?;
        self.expect(";")?;
        self.finish(m, Kind::TfPortDecl);
        Ok(())
    }

    /// `: name` after an `end` keyword.
    pub(crate) fn end_label(&mut self) -> PResult<'a> {
        if self.at(":") && !self.at_n(1, ":") {
            let m = self.mark();
            self.bump();
            if self.at("new") {
                self.leaf(Kind::Name);
            } else {
                self.name()?;
            }
            self.finish(m, Kind::EndLabel);
        }
        Ok(())
    }

    /// `let name(ports) = expr;`.
    pub(crate) fn let_decl(&mut self) -> PResult<'a> {
        let m = self.mark();
        self.bump();
        self.name()?;
        if self.eat("(") {
            while !self.at(")") {
                let p = self.mark();
                self.attributes_opt()?;
                if self.at("untyped") {
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
                    self.expr()?;
                }
                self.finish(p, Kind::LetPort);
                if !self.eat(",") {
                    break;
                }
            }
            self.expect(")")?;
        }
        self.expect("=")?;
        self.expr()?;
        self.expect(";")?;
        self.finish(m, Kind::LetDecl);
        Ok(())
    }
}
