use crate::ast::Ast;
use crate::preprocess::{Options, preprocess};
use crate::source::{Source, SourceMap};

/// Parses `text` and returns the tree as s-expressions, or the first
/// line of the error.
fn run(text: &str) -> Result<String, String> {
    let map = SourceMap::new();
    map.add(Source::new("t.sv", text));
    let out = preprocess(&map, &[0], &Options::default()).map_err(|e| map.render(&e))?;
    let ast: Ast = super::parse(&out).map_err(|e| map.render(&e).lines().next().unwrap().to_string())?;
    Ok(ast.sexp(ast.root))
}

fn ok(text: &str) -> String {
    run(text).unwrap_or_else(|e| panic!("{e}"))
}

fn err(text: &str) -> String {
    match run(text) {
        Ok(t) => panic!("expected an error, got {t}"),
        Err(e) => e.lines().next().unwrap().to_string(),
    }
}

#[test]
fn module_with_ports() {
    assert_eq!(
        ok("module m(input logic a, output b); endmodule"),
        "(SourceText (Module (Keyword module) (Name m) (Ports (AnsiPort (Keyword input) (BuiltinType (Keyword logic)) (Declarator (Name a))) (AnsiPort (Keyword output) (ImplicitType) (Declarator (Name b))))))"
    );
}

#[test]
fn expression_precedence() {
    let t = ok("module m; assign x = a + b * c ** d - e; endmodule");
    assert!(t.contains("(Binary (Binary (Name a) (Op +) (Binary (Name b) (Op *) (Binary (Name c) (Op **) (Name d)))) (Op -) (Name e))"), "{t}");
    let t = ok("module m; assign x = a ? b : c ? d : e; endmodule");
    assert!(t.contains("(Conditional (Name a) (Name b) (Conditional (Name c) (Name d) (Name e)))"), "{t}");
}

#[test]
fn keywords_are_not_names() {
    assert_eq!(
        err("module m; int while; endmodule"),
        "t.sv:1:15: error: `while` is a keyword, so it cannot be used as a name"
    );
}

/// Asserts that the tree of `text` contains `part`.
fn has(text: &str, part: &str) {
    let t = ok(text);
    assert!(t.contains(part), "{part}\nnot in\n{t}");
}

#[test]
fn declarations() {
    has(
        "module m; logic [7:0] a [4], b = 1; endmodule",
        "(VarDecl (BuiltinType (Keyword logic) (RangeDim (Number 7) (Number 0))) (Declarator (Name a) (SizeDim (Number 4))) (Declarator (Name b) (Number 1)))",
    );
    has(
        "module m; wire (strong0, weak1) w = x; endmodule",
        "(NetDecl (Keyword wire) (Strength (Keyword strong0) (Keyword weak1)) (ImplicitType) (Declarator (Name w) (Name x)))",
    );
    has(
        "module m; T #(8) x; p::q y; endmodule",
        "(VarDecl (NamedType (Specialize (Name T) (ParamValues (Number 8)))) (Declarator (Name x)))",
    );
    has("module m; p::q y; endmodule", "(NamedType (Scope (Name p) (Name q)))");
    has(
        "module m; int q[$], d[], h[string]; endmodule",
        "(Declarator (Name q) (QueueDim)) (Declarator (Name d) (UnsizedDim)) (Declarator (Name h) (AssocDim (BuiltinType (Keyword string))))",
    );
    has(
        "typedef enum logic [1:0] {A, B = 2} e_t;",
        "(Typedef (EnumType (BuiltinType (Keyword logic) (RangeDim (Number 1) (Number 0))) (EnumItem (Name A)) (EnumItem (Name B) (Number 2))) (Name e_t))",
    );
    has(
        "typedef struct packed { logic a; byte b; } s_t;",
        "(StructType (Keyword struct) (Keyword packed) (StructMember (BuiltinType (Keyword logic)) (Declarator (Name a)))",
    );
    has(
        "package p; localparam int N = 4; endpackage",
        "(Package (Name p) (ParamDecl (Keyword localparam) (BuiltinType (Keyword int)) (ParamAssign (Name N) (Number 4))))",
    );
}

#[test]
fn declaration_errors() {
    assert_eq!(
        err("module m; initial begin static x = 1; end endmodule"),
        "t.sv:1:32: error: this declaration needs a data type, or `var`"
    );
    assert_eq!(
        err("module m; logic [8] v; endmodule"),
        "t.sv:1:19: error: a packed dimension needs a range, such as [7:0]"
    );
    assert_eq!(
        err("module m; struct { int a; } [1:0] s; endmodule"),
        "t.sv:1:29: error: packed dimensions need a packed struct or union"
    );
    assert_eq!(
        err("module m; wire (pull0, pull1) w; endmodule"),
        "t.sv:1:11: error: a drive strength needs an assignment in the declaration"
    );
    assert_eq!(
        err("module m; type(x) y; endmodule"),
        "t.sv:1:11: error: a variable declared with a type reference needs `var` before it"
    );
}

#[test]
fn instances_and_generate() {
    has(
        "module m; sub #(.W(4)) u1 (.a(x), .b), u2 (.*); endmodule",
        "(Instantiation (Name sub) (ParamValues (NamedArg (Name W) (Number 4))) (Instance (Name u1) (NamedConnection (Name a) (Name x)) (NamedConnection (Name b))) (Instance (Name u2) (NamedConnection (Op .*))))",
    );
    has(
        "module m; and #2 g (o, a, b); endmodule",
        "(GateInstantiation (Keyword and) (Delay (Number 2)) (Instance (Name g) (Connection (Name o)) (Connection (Name a)) (Connection (Name b))))",
    );
    has(
        "module m; for (genvar i = 0; i < 4; i++) begin : g end endmodule",
        "(GenerateFor (Assign (Keyword genvar) (Name i) (Number 0)) (Binary (Name i) (Op <) (Number 4)) (IncDec (Name i) (Op ++)) (GenerateBlock (Name g)))",
    );
    assert_eq!(
        err("module m; s u(.a(1), 2); endmodule"),
        "t.sv:1:22: error: ordered and named port connections cannot be mixed"
    );
    assert_eq!(
        err("module m; if (1) begin generate endgenerate end endmodule"),
        "t.sv:1:24: error: a generate region cannot be nested inside a generate block"
    );
    assert_eq!(
        err("module m(input a); output b; endmodule"),
        "t.sv:1:20: error: a module whose header declares its ports cannot declare ports in its body"
    );
}

#[test]
fn statements() {
    has(
        "module m; initial begin : b int i; i = 1; #5 i <= @(c) 2; end : b endmodule",
        "(Block (Keyword begin) (Name b) (VarDecl (BuiltinType (Keyword int)) (Declarator (Name i))) (Assign (Name i) (Op =) (Number 1)) (Timed (Delay (Number 5)) (Assign (Name i) (Op <=) (Event (EventTerm (Name c))) (Number 2))) (EndLabel (Name b)))",
    );
    has(
        "module m; initial unique case (x) 1, 2: y = 0; default y = 1; endcase endmodule",
        "(Case (Keyword unique) (Keyword case) (Name x) (CaseItem (Number 1) (Number 2) (Assign (Name y) (Op =) (Number 0))) (CaseItem (Assign (Name y) (Op =) (Number 1))))",
    );
    has(
        "module m; initial for (int i = 0, j = 1; i < j; i++, j--) ; endmodule",
        "(For (ForInit (VarDecl (BuiltinType (Keyword int)) (Declarator (Name i) (Number 0)) (Declarator (Name j) (Number 1)))) (Binary (Name i) (Op <) (Name j)) (ForStep (IncDec (Name i) (Op ++)) (IncDec (Name j) (Op --))) (NullStatement))",
    );
    has(
        "module m; initial foreach (a[i, , k]) f(i); endmodule",
        "(Foreach (Name a) (Name i) (Empty) (Name k) (ExprStatement (Call (Name f) (Args (Name i)))))",
    );
    has(
        "module m; initial fork a(); join_none endmodule",
        "(Block (Keyword fork) (ExprStatement (Call (Name a) (Args))) (Keyword join_none))",
    );
    assert_eq!(
        err("module m; initial begin x = 1; int y; end endmodule"),
        "t.sv:1:32: error: declarations must come before the statements of a block"
    );
    assert_eq!(
        err("module m; initial begin : a end : b endmodule"),
        "t.sv:1:35: error: the name after the end must be `a`"
    );
    assert_eq!(
        err("module m; initial l: begin : n end endmodule"),
        "t.sv:1:28: error: a block cannot have both a statement label and a block name"
    );
    assert_eq!(err("module m; initial if (a = b) ; endmodule"), "t.sv:1:25: error: expected `)`, found `=`");
}

#[test]
fn expressions() {
    has(
        "module m; assign x = {a, {2{b}}, {<<8{c}}}; endmodule",
        "(Concat (Name a) (Replicate (Number 2) (Concat (Name b))) (Streaming (Op <<) (Number 8) (StreamExpr (Name c))))",
    );
    has(
        "module m; assign x = a[3:0] + b[i+:2] + c.d[1]; endmodule",
        "(RangeSelect (Name a) (Number 3) (Op :) (Number 0))",
    );
    has("module m; assign x = a[3:0] + b[i+:2] + c.d[1]; endmodule", "(Index (Member (Name c) (Name d)) (Number 1))");
    has("module m; assign x = int'(y) + T'(z) + 8'(w); endmodule", "(Cast (Keyword int) (Name y))");
    has("module m; assign x = int'(y) + T'(z) + 8'(w); endmodule", "(Cast (Number 8) (Name w))");
    has(
        "module m; assign x = a inside {1, [2:3]}; endmodule",
        "(Inside (Name a) (RangeList (Number 1) (ValueRange (Number 2) (Number 3))))",
    );
    has(
        "module m; initial x = '{a: 1, default: 0}; endmodule",
        "(AssignmentPattern (PatternKey (Name a) (Number 1)) (PatternKey (Keyword default) (Number 0)))",
    );
    has(
        "module m; initial x = q.find with (item > 2); endmodule",
        "(With (Member (Name q) (Name find)) (Binary (Name item) (Op >) (Number 2)))",
    );
    has("module m; assign x = 12'hFFF + 'b1 + 2.5 + 3ns; endmodule", "(Number 12'hFFF)");
    has(
        "module m; assign x = a -> b <-> c; endmodule",
        "(Binary (Name a) (Op ->) (Binary (Name b) (Op <->) (Name c)))",
    );
    assert_eq!(err("module m; assign x = 0'h1; endmodule"), "t.sv:1:22: error: the size of a literal cannot be zero");
    assert_eq!(err("module m; assign x = 4'b12; endmodule"), "t.sv:1:26: error: `2` is not a digit of base 'b");
    assert_eq!(
        err("module m; string s = \"\\x4z\"; endmodule"),
        "t.sv:1:22: error: a `\\x` escape needs one or two hexadecimal digits"
    );
    assert_eq!(
        err("module m; initial f(.a(1), 2); endmodule"),
        "t.sv:1:28: error: a positional argument cannot follow a named one"
    );
}

#[test]
fn attributes() {
    has(
        "module m; (* keep, w = 2 *) logic a; endmodule",
        "(Attributes (AttrSpec (Name keep)) (AttrSpec (Name w) (Number 2))) (VarDecl (BuiltinType (Keyword logic)) (Declarator (Name a)))",
    );
    has(
        "module m; assign s = a + (* fast *) b; endmodule",
        "(Binary (Name a) (Op +) (Attributes (AttrSpec (Name fast))) (Name b))",
    );
    has("module m; always @(*) x = y; endmodule", "(Timed (Event) (Assign (Name x) (Op =) (Name y)))");
    assert_eq!(
        err("module m; (* a = 1 + (* b *) 2 *) logic x; endmodule"),
        "t.sv:1:22: error: an attribute cannot contain another attribute"
    );
}

#[test]
fn assertions_and_sequences() {
    has(
        "module m; a1: assert property (@(posedge c) a ##[1:2] b |=> c) else $error; endmodule",
        "(Labeled (Name a1) (ConcurrentAssertion (Keyword assert) (Keyword property) (PropertySpec (Event (EventTerm (Keyword posedge) (Name c))) (PropBinary (SeqDelay (Name a) (CycleDelay (Number 1) (Number 2)) (Name b)) (Op |=>) (Name c))) (ActionBlock (Empty) (ExprStatement (SystemName $error)))))",
    );
    has(
        "module m; property p(x); x[*3] and not (x ##1 x); endproperty endmodule",
        "(PropertyDecl (Name p) (PropPort (Empty) (Name x)) (PropertySpec (PropBinary (SeqRepeat (Name x) (Op *) (Number 3)) (Keyword and) (PropUnary (Keyword not) (SeqParen (SeqDelay (Name x) (CycleDelay (Number 1)) (Name x)))))))",
    );
    has(
        "module m; initial assert #0 (x) else $error(\"no\"); endmodule",
        "(ImmediateAssertion (Keyword assert) (Op #0) (Name x) (ActionBlock (Empty) (ExprStatement (Call (SystemName $error) (Args (String \"no\"))))))",
    );
    has("module m; assert property (disable iff (r) a); endmodule", "(PropertySpec (DisableIff (Name r)) (Name a))");
    assert_eq!(
        err("module m; cover property (a) $info; else $error; endmodule"),
        "t.sv:1:37: error: a cover statement cannot have an else"
    );
    assert_eq!(
        err("module m; restrict property (a) x = 1; endmodule"),
        "t.sv:1:33: error: a restrict statement ends with `;` and has no action"
    );
}

#[test]
fn classes_and_constraints() {
    has(
        "class C extends B #(int); local rand int x; extern function void f(); constraint c { x < 4 -> { x > 0; } soft x == 2; } endclass",
        "(Class (Name C) (Extends (NamedType (Specialize (Name B) (ParamValues (TypeExpr (BuiltinType (Keyword int))))))) (VarDecl (Qualifiers (Keyword local) (Keyword rand)) (BuiltinType (Keyword int)) (Declarator (Name x))) (Prototype (Qualifiers (Keyword extern)) (Keyword function) (VoidType void) (Name f) (TfPorts)) (Constraint (Name c) (ConstraintBlock (ConstraintImplies (Binary (Name x) (Op <) (Number 4)) (ConstraintBlock (ConstraintExpr (Binary (Name x) (Op >) (Number 0))))) (ConstraintExpr (Keyword soft) (Binary (Name x) (Op ==) (Number 2))))))",
    );
    assert_eq!(
        err("class C; local protected int x; endclass"),
        "t.sv:1:16: error: a member cannot be both local and protected"
    );
    assert_eq!(err("class C; import p::*; endclass"), "t.sv:1:10: error: an import cannot appear directly in a class");
    assert_eq!(
        err("class C; function static void f(); endfunction endclass"),
        "t.sv:1:19: error: a class method cannot have a static lifetime"
    );
}

#[test]
fn covergroups() {
    has(
        "module m; covergroup g @(posedge c); cp: coverpoint x iff (en) { bins lo = {[0:3]}; bins t = (1 => 2[*2]); } cross cp, y; endgroup endmodule",
        "(Coverpoint (Label (Name cp)) (Name x) (Iff (Name en)) (Bins (Keyword bins) (Name lo) (RangeList (ValueRange (Number 0) (Number 3)))) (Bins (Keyword bins) (Name t) (TransList (TransSet (TransItem (RangeList (Number 1))) (TransItem (RangeList (Number 2)) (Op *) (Number 2))))))",
    );
}

#[test]
fn udps() {
    has(
        "primitive mux (output y, input s, a, b); table 0 0 ? : 0; 1 ? 1 : 1; endtable endprimitive",
        "(UdpTable (UdpEntry 0 0 ? : 0;) (UdpEntry 1 ? 1 : 1;))",
    );
    has(
        "primitive ff (q, c, d); output q; reg q; input c, d; initial q = 1'b0; table (01) 1 : ? : 1; (01) 0 : ? : 0; ? * : ? : -; endtable endprimitive",
        "(UdpInitial (Name q) (Number 1'b0))",
    );
    assert_eq!(
        err("primitive p (output y, input a); table 0 : -; endtable endprimitive"),
        "t.sv:1:40: error: `-` cannot be an output in a UDP table"
    );
    assert_eq!(
        err("primitive p (output reg y, input a, b); table (01) (10) : ? : 1; endtable endprimitive"),
        "t.sv:1:47: error: a UDP table row can have at most one edge"
    );
    assert_eq!(
        err("module m; primitive p (output y, input a); table 0 : 0; endtable endprimitive endmodule"),
        "t.sv:1:11: error: a UDP can only be declared outside other design elements"
    );
}

#[test]
fn specify_blocks() {
    has(
        "module m(input a, output y); specify specparam t = 2; (a => y) = (t, 3); if (a) (posedge a *> y) = 1; $setup(a, posedge y &&& a, 1, n); endspecify endmodule",
        "(PathDecl (Name a) (Op =>) (Name y) (Name t) (Number 3))",
    );
    has(
        "module m(input a, output y); specify $setup(a, posedge y &&& a, 1, n); endspecify endmodule",
        "(TimingCheck (SystemName $setup) (Name a) (TimingEvent (Keyword posedge) (Name y) (Name a)) (Number 1) (Name n))",
    );
    assert_eq!(
        err("module m(input a, output y); specify ifnone (posedge a => y) = 1; endspecify endmodule"),
        "t.sv:1:46: error: ifnone cannot be used on an edge-sensitive path"
    );
    assert_eq!(
        err("module m; specify $display(1); endspecify endmodule"),
        "t.sv:1:19: error: only timing checks can be called in a specify block"
    );
    assert_eq!(
        err("module m; initial $setup(a, b, 1); endmodule"),
        "t.sv:1:19: error: a timing check can only appear in a specify block"
    );
}

#[test]
fn configurations() {
    has(
        "config cfg; design work.top; default liblist work lib2; instance top.u1 use lib2.fast; endconfig",
        "(Config (Name cfg) (ConfigDesign (Member (Name work) (Name top))) (ConfigRule (Keyword default) (Keyword liblist) (Name work) (Name lib2)) (ConfigRule (Keyword instance) (Member (Name top) (Name u1)) (Keyword use) (Member (Name lib2) (Name fast))))",
    );
}

#[test]
fn keyword_sets_follow_begin_keywords() {
    has("`begin_keywords \"1364-2005\"\nmodule m; reg logic; endmodule\n`end_keywords", "(Declarator (Name logic))");
    assert_eq!(
        err("module m; reg logic; endmodule"),
        "t.sv:1:15: error: `logic` is a keyword, so it cannot be used as a name"
    );
}
