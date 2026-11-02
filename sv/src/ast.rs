//! The syntax tree.
//!
//! As tokens are slices of the source, nodes are slices of the token list:
//! a node is a kind, the range of tokens it covers, and a range of the
//! child list. Nothing is copied out of the source. All nodes live in one
//! arena, `Ast::nodes`, and refer to each other by `NodeId`.
//!
//! Children are the parts of a construct that carry meaning: names,
//! literals, operators, the keywords that select between forms (such as a
//! port's direction), and sub-constructs. Punctuation and keywords that
//! only introduce a construct are not children; they are still in the
//! node's token range. Each `Kind` says what its children are.

use crate::lexer::Token;
use std::fmt::Write;

/// An index into `Ast::nodes`.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct NodeId(pub u32);

#[derive(Clone, Copy, Debug)]
pub struct Node {
    pub kind: Kind,
    /// The tokens covered, as a half-open range of indices into
    /// `Ast::tokens`. An empty node (an omitted type) has `first == end`.
    pub first: u32,
    pub end: u32,
    child_start: u32,
    child_len: u32,
}

/// A parsed compilation unit.
pub struct Ast<'a> {
    pub tokens: Vec<Token<'a>>,
    pub nodes: Vec<Node>,
    children: Vec<NodeId>,
    pub root: NodeId,
}

impl<'a> Ast<'a> {
    pub(crate) fn new(tokens: Vec<Token<'a>>) -> Self {
        Ast { tokens, nodes: Vec::new(), children: Vec::new(), root: NodeId(0) }
    }

    pub(crate) fn push(&mut self, kind: Kind, first: usize, end: usize, children: &[NodeId]) -> NodeId {
        let id = NodeId(self.nodes.len() as u32);
        let child_start = self.children.len() as u32;
        self.children.extend_from_slice(children);
        self.nodes.push(Node {
            kind,
            first: first as u32,
            end: end as u32,
            child_start,
            child_len: children.len() as u32,
        });
        id
    }

    /// Drops every node from `nodes` on, for backtracking.
    pub(crate) fn truncate(&mut self, nodes: usize, children: usize) {
        self.nodes.truncate(nodes);
        self.children.truncate(children);
    }

    pub(crate) fn sizes(&self) -> (usize, usize) {
        (self.nodes.len(), self.children.len())
    }

    pub fn node(&self, id: NodeId) -> &Node {
        &self.nodes[id.0 as usize]
    }

    pub fn kind(&self, id: NodeId) -> Kind {
        self.node(id).kind
    }

    pub fn children(&self, id: NodeId) -> &[NodeId] {
        let n = self.node(id);
        &self.children[n.child_start as usize..(n.child_start + n.child_len) as usize]
    }

    /// The first child of kind `kind`.
    pub fn child(&self, id: NodeId, kind: Kind) -> Option<NodeId> {
        self.children(id).iter().copied().find(|&c| self.kind(c) == kind)
    }

    /// The tokens a node covers.
    pub fn tokens_of(&self, id: NodeId) -> &[Token<'a>] {
        let n = self.node(id);
        &self.tokens[n.first as usize..n.end as usize]
    }

    /// The node's first token: for a leaf, the token itself.
    pub fn token(&self, id: NodeId) -> Token<'a> {
        self.tokens_of(id).first().copied().unwrap_or(Token(""))
    }

    /// The text of a node: its tokens, with a space between two that were
    /// not next to each other in the source.
    pub fn text(&self, id: NodeId) -> String {
        let mut out = String::new();
        let mut prev: Option<&str> = None;
        for t in self.tokens_of(id) {
            if let Some(p) = prev
                && p.as_ptr() as usize + p.len() != t.0.as_ptr() as usize
            {
                out.push(' ');
            }
            out += t.0;
            prev = Some(t.0);
        }
        out
    }

    /// The tree as nested lists, one node per line: `Kind` for a node with
    /// children, `Kind text` for a leaf.
    pub fn dump(&self, id: NodeId) -> String {
        let mut out = String::new();
        self.dump_into(id, 0, &mut out);
        out
    }

    fn dump_into(&self, id: NodeId, depth: usize, out: &mut String) {
        let kids = self.children(id);
        let _ = write!(out, "{:width$}{:?}", "", self.kind(id), width = depth * 2);
        if kids.is_empty() && self.kind(id).is_leaf() {
            let _ = write!(out, " {}", self.text(id));
        }
        out.push('\n');
        for &k in kids {
            self.dump_into(k, depth + 1, out);
        }
    }

    /// The tree on one line, as s-expressions: `(Binary (Name a) (Op +)
    /// (Number 1))`. Handy in tests.
    pub fn sexp(&self, id: NodeId) -> String {
        let kids = self.children(id);
        if kids.is_empty() {
            let n = self.node(id);
            return if n.first == n.end || !self.kind(id).is_leaf() {
                format!("({:?})", self.kind(id))
            } else {
                format!("({:?} {})", self.kind(id), self.text(id))
            };
        }
        let inner: Vec<String> = kids.iter().map(|&k| self.sexp(k)).collect();
        format!("({:?} {})", self.kind(id), inner.join(" "))
    }
}

impl Kind {
    /// Whether nodes of this kind are single tokens (or a literal's few
    /// tokens) rather than constructs with children.
    pub fn is_leaf(self) -> bool {
        use Kind::*;
        matches!(
            self,
            Name | Keyword
                | Op
                | Number
                | TimeLiteral
                | UnbasedUnsized
                | String
                | SystemName
                | Dollar
                | Null
                | This
                | Super
                | ScopeKeyword
                | VoidType
                | UdpEntry
                | EdgeDescriptors
        )
    }
}

/// What a node is. The comment on each kind lists its children, which are
/// always in source order; `?` marks an optional child and `*` a repeated
/// one. Attributes, `(* ... *)`, come before the item, statement or
/// operand they apply to, as an `Attributes` node beside it.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Kind {
    // ---- leaves ----
    /// An identifier, simple or escaped.
    Name,
    /// A keyword that selects a form: a direction, a qualifier, an edge, a
    /// net type, `signed`, ...
    Keyword,
    /// An operator.
    Op,
    /// A number: `12`, `8'hff` (size, base and digits), `'sb101`, `1.5e3`.
    Number,
    /// `10ns`, `1step`.
    TimeLiteral,
    /// `'0`, `'1`, `'x`, `'z`.
    UnbasedUnsized,
    /// `"..."` or `"""..."""`.
    String,
    /// A system task or function name: `$display`.
    SystemName,
    /// `$` on its own: the last element of a queue, or an unbounded range.
    Dollar,
    Null,
    This,
    Super,
    /// `$root`, `$unit` and `local` as scope prefixes.
    ScopeKeyword,
    /// Nothing: an omitted type, port, argument or expression.
    Empty,

    // ---- source text ----
    /// The compilation unit: Item*.
    SourceText,
    /// Attributes, `(* name = value, ... *)`: AttrSpec*.
    Attributes,
    /// Name, Expr?
    AttrSpec,
    /// `module`, `macromodule`, `interface`, `program` and their bodies:
    /// Keyword (module, interface, ...), Keyword(lifetime)?, Name,
    /// PackageImport*, ParamPorts?, Ports?, Item*, EndLabel?
    Module,
    /// `extern module ...;`: as Module, with no items.
    ExternModule,
    /// Keyword(lifetime)?, Name, Item*, EndLabel?
    Package,
    /// Name, PropPort*, Item*, EndLabel?
    Checker,
    /// Keyword(virtual)?, Keyword(final)?, Name, ParamPorts?, Extends?,
    /// Implements?, Item*, EndLabel?
    Class,
    /// `interface class`: Name, ParamPorts?, Extends*, Item*, EndLabel?
    InterfaceClass,
    /// Type, Args? (or Keyword(default) for `(default)`)
    Extends,
    /// Type*
    Implements,
    /// `program; ... endprogram` with no name: Item*
    AnonymousProgram,
    /// The name after `: ` that closes a block.
    EndLabel,
    /// `timeunit 1ns / 1ps;` or `timeprecision 1ps;`: Keyword, TimeLiteral,
    /// TimeLiteral?
    Timeunits,
    /// `#( ... )` on a declaration: ParamDecl* or ParamAssign*
    ParamPorts,
    /// `( ... )` on a declaration: Port* or AnsiPort*
    Ports,
    /// A non-ANSI port: an expression, or `.name(expr)` as a NamedArg
    Port,
    /// An ANSI port: Attributes?, Keyword(direction)?, Keyword(net type or
    /// var)?, Type (or InterfacePortType), Declarator; or an explicit port:
    /// Keyword(direction)?, NamedArg
    AnsiPort,
    /// `intf.mp` or `interface.mp` as a port's type: Name or Keyword,
    /// Name?
    InterfacePortType,
    /// `input wire logic a, b;`: Attributes?, Keyword(direction),
    /// Keyword(net type or var)?, Type, Declarator*
    PortDecl,
    /// `intf.mp a, b;` in a non-ANSI module: InterfacePortType, Declarator*
    InterfacePortDecl,

    // ---- declarations ----
    /// `parameter` or `localparam` with values: Keyword, Type, ParamAssign*
    ParamDecl,
    /// `parameter type T = int`: Keyword, Keyword(forward type)?,
    /// TypeAssign*
    TypeParamDecl,
    /// Name, Dim*, Expr? (or a Type for a type parameter)
    ParamAssign,
    /// Name, Type?
    TypeAssign,
    /// `specparam`: Dim?, SpecparamAssign*
    SpecparamDecl,
    /// Name, Expr
    SpecparamAssign,
    /// A variable declaration: Qualifiers?, Type, Declarator*
    VarDecl,
    /// `const`, `var`, `static`, `automatic`, `rand`, `local`, ... before a
    /// declaration: Keyword*
    Qualifiers,
    /// One declared name: Name, Dim*, Expr? (the initial value)
    Declarator,
    /// A net declaration: Keyword(net type) or Name (user net type),
    /// Strength?, Keyword(vectored/scalared)?, Type, Delay?, Declarator*
    NetDecl,
    /// `interconnect`: Type, Delay?, Declarator*
    InterconnectDecl,
    /// `typedef`: Type, Name, Dim*
    Typedef,
    /// `typedef enum T;` and the like: Keyword?, Name
    ForwardTypedef,
    /// `nettype`: Type, Name, Name? (resolution function)
    NettypeDecl,
    /// `import p::x, q::*;`: ImportItem*
    PackageImport,
    /// `export p::x;` or `export *::*;`: ImportItem*
    PackageExport,
    /// Name (package), Name or Op(*)
    ImportItem,
    /// `genvar i, j;`: Name*
    GenvarDecl,
    /// `let f(a) = expr;`: Name, LetPort*, Expr
    LetDecl,
    /// Type, Name, Dim*, Expr?
    LetPort,
    /// `function`: Qualifiers?, Keyword(:initial/:extends/:final)*,
    /// Keyword(lifetime)?, Type (Empty if implicit, or VoidType), Name or Scope
    /// (`C::f`, `intf.f`), TfPorts?, declarations, Statement*, EndLabel?
    Function,
    /// `task`: as Function, without the type.
    Task,
    /// A prototype without a body (`extern`, `pure virtual`, imports):
    /// Qualifiers?, Keyword(function/task), Type?, Name, TfPorts?
    Prototype,
    /// `( ... )` of a function or task: TfPort*
    TfPorts,
    /// Keyword(direction)?, Type, Declarator?
    TfPort,
    /// `input int a, b;` in a function or task body: Keyword, Type,
    /// Declarator*
    TfPortDecl,
    /// `import "DPI-C" ...`: String, Keyword(context/pure)?, Name (C name)?,
    /// Prototype
    DpiImport,
    /// `export "DPI-C" ...`: String, Name (C name)?, Keyword, Name
    DpiExport,
    /// `modport`: ModportItem*
    ModportDecl,
    /// Name, ModportPorts*
    ModportItem,
    /// Keyword(direction, import, export or clocking), ModportPort*
    ModportPorts,
    /// Name, or `.name(expr)` as a NamedArg, or a Prototype
    ModportPort,
    /// `constraint`: Qualifiers?, Name (maybe scoped), ConstraintBlock?
    Constraint,
    /// `{ ... }`: ConstraintItem*
    ConstraintBlock,
    /// `solve a before b;`: Expr*, Expr*
    Solve,
    /// `[soft] expr;` or `expr dist {...};`: Keyword(soft)?, Expr
    ConstraintExpr,
    /// `unique { ... }`: Expr*
    Uniqueness,
    /// `a -> constraint_set`: Expr, ConstraintItem or ConstraintBlock
    ConstraintImplies,
    /// Expr, ConstraintSet, ConstraintSet?
    ConstraintIf,
    /// Expr (array), Name*, ConstraintSet
    ConstraintForeach,
    /// `disable soft x;`: Expr
    DisableSoft,
    /// `expr dist { ... }`: Expr, DistItem*
    Dist,
    /// Expr (value or range), Op(:= or :/)?, Expr?
    DistItem,
    /// `covergroup`: Name, TfPorts?, Event?, Item*, EndLabel?
    Covergroup,
    /// `option.x = e;` / `type_option.x = e;`: Name, Name, Expr
    CoverageOption,
    /// Type?, Label?, Expr, Iff?, Bins* and CoverageOption*
    Coverpoint,
    /// Label?, Expr*, Iff?, Bins*, CoverageOption* and Function*
    Cross,
    /// `bins b[] = {...} ...;`: Keyword(wildcard)?, Keyword(bins kind),
    /// Name, SizeDim?, RangeList, TransList, Keyword(default) or Expr,
    /// Expr(with)?, Iff?
    Bins,
    /// `binsof(x) intersect {..}`: Expr, RangeList?
    Binsof,
    /// `(a => b), (c => d)` in bins: TransSet*
    TransList,
    /// TransItem*
    TransSet,
    /// RangeList, Op(repeat kind)?, Expr?
    TransItem,

    // ---- types ----
    /// A built-in type: Keyword (int, logic, ...), Keyword(signing)?, Dim*
    BuiltinType,
    /// No type given, perhaps with signing and dimensions: Keyword?, Dim*
    ImplicitType,
    /// A named type: ScopedName, ParamValues?, Dim*
    NamedType,
    /// struct or union: Keyword(struct/union), Keyword(tagged/soft)?,
    /// Keyword(packed)?, Keyword(signing)?, Member*, Dim*
    StructType,
    /// Attributes?, Keyword(rand/randc)?, Type, Declarator*
    StructMember,
    /// Type?(base), EnumItem*, Dim*
    EnumType,
    /// Name, Expr?, Expr? (range), Expr? (value)
    EnumItem,
    /// `virtual [interface] I #(...) .mp`: Name, ParamValues?, Name?
    VirtualInterfaceType,
    /// `type(expr)` or `type(T)`: Expr or Type
    TypeOf,
    /// `void`
    VoidType,

    // ---- dimensions ----
    /// `[a:b]`: Expr, Expr
    RangeDim,
    /// `[n]`: Expr
    SizeDim,
    /// `[]`
    UnsizedDim,
    /// `[*]` or `[type]`: Type?
    AssocDim,
    /// `[$]` or `[$:n]`: Expr?
    QueueDim,

    // ---- instances ----
    /// Name (module, interface, program, checker or UDP), Strength?,
    /// ParamValues? or Delay?, Instance*
    Instantiation,
    /// `#( ... )`: Expr* or NamedArg*
    ParamValues,
    /// Name?, Dim*, Connection*
    Instance,
    /// An ordered connection: Attributes?, Expr?
    Connection,
    /// `.name(expr)`, `.name` or `.*`: Attributes?, Name or Op(.*), Expr?
    NamedConnection,
    /// A gate: Keyword (and, nmos, pullup, ...), Strength?, Delay?,
    /// Instance*
    GateInstantiation,
    /// `(strong0, weak1)` or `(small)`: Keyword*
    Strength,
    /// `defparam a.b = 1;`: DefparamAssign*
    Defparam,
    /// Expr, Expr
    DefparamAssign,
    /// `bind target [: instances] ...`: Expr*, Instantiation
    Bind,

    // ---- generate ----
    /// `generate ... endgenerate`: Item*
    GenerateRegion,
    /// `for (...)` generate: Statement(init), Expr, Statement(step), Item
    GenerateFor,
    /// Expr, Item, Item?
    GenerateIf,
    /// Expr, GenerateCaseItem*
    GenerateCase,
    /// Expr* (or none for default), Item
    GenerateCaseItem,
    /// `[name:] begin [: name] ... end`: Name?, Item*, EndLabel?
    GenerateBlock,

    // ---- module items ----
    /// `assign`: Strength?, Delay?, Assign*
    ContinuousAssign,
    /// `alias a = b = c;`: Expr*
    NetAlias,
    /// Keyword (initial, always, always_ff, final, ...), Statement
    Process,
    /// `default clocking cb;` or `default disable iff e;`: Keyword, Name or
    /// Expr
    DefaultClocking,
    DefaultDisable,
    /// `clocking`: Keyword(default/global)?, Name?, Event, ClockingItem*,
    /// EndLabel?
    Clocking,
    /// `default input #1 output #2;`: ClockingSkew*
    ClockingDefault,
    /// ClockingDirection, Declarator*
    ClockingSignals,
    /// Keyword (input, output, inout), ClockingSkew?, Keyword?,
    /// ClockingSkew?
    ClockingDirection,
    /// Keyword(edge)?, Delay?
    ClockingSkew,
    /// `$error(...)` etc. outside procedures: Call
    ElaborationTask,
    /// `specify ... endspecify`: Item*
    Specify,
    /// `(a => b) = 1;` and its edge and state forms: Expr?(condition),
    /// Keyword(if/ifnone)?, Keyword(edge)?, Expr* (inputs), Op(polarity)?,
    /// Op(=> or *>), Expr* (outputs), Op?, Expr?(data source), Expr*
    /// (delays)
    PathDecl,
    /// `$setup(...)` and friends: SystemName, TimingArg*
    TimingCheck,
    /// Keyword(edge)? or EdgeDescriptors?, Expr, Expr?(condition)
    TimingEvent,
    /// `edge [01, 10]`: the descriptors' tokens
    EdgeDescriptors,
    /// `pulsestyle_onevent a;` etc: Keyword, Expr*
    PulseStyle,
    /// `primitive`: Name, Ports or AnsiPort*, UdpPortDecl*, UdpInitial?,
    /// UdpTable, EndLabel?
    Udp,
    /// Keyword (output, input, reg), Keyword(reg)?, Name*, Expr?
    UdpPortDecl,
    /// `initial q = 1'b0;`: Name, Expr
    UdpInitial,
    /// `table ... endtable`: UdpEntry*
    UdpTable,
    /// One row; its symbols are the characters of its tokens.
    UdpEntry,
    /// `config`: Name, ParamDecl*, ConfigDesign, ConfigRule*, EndLabel?
    Config,
    /// `design lib.cell ...;`: Expr*
    ConfigDesign,
    /// Keyword (default, instance, cell), Expr?, Keyword (liblist or use),
    /// Expr*
    ConfigRule,
    /// `library name path, ...;` in a library map.
    Library,

    // ---- assertions ----
    /// `assert property (...)` etc.: Keyword (assert, assume, cover or
    /// restrict), Keyword(property/sequence), PropertySpec, then ActionBlock
    /// (assert, assume) or Statement (cover). A label is a Labeled around it.
    ConcurrentAssertion,
    /// `assert (e)`, `assert #0 (e)`, `assert final (e)`: Keyword, Keyword
    /// (final) or Op(#0)?, Expr, ActionBlock
    ImmediateAssertion,
    /// Statement?(pass), Statement?(else)
    ActionBlock,
    /// Event?, DisableIff?, Prop
    PropertySpec,
    /// `disable iff (e)`: Expr
    DisableIff,
    /// `iff (e)` on a coverpoint, cross, bins or event: Expr
    Iff,
    /// `name :` in front of a coverpoint or cross: Name
    Label,
    /// `property`: Name, PropPort*, Item* (variables), PropertySpec,
    /// EndLabel?
    PropertyDecl,
    /// `sequence`: Name, PropPort*, Item*, Prop, EndLabel?
    SequenceDecl,
    /// Keyword(local)?, Keyword(direction)?, Type, Name, Dim*, Expr?
    PropPort,
    /// Binary sequence or property operator: Prop, Op or Keyword, Prop
    PropBinary,
    /// Prefix property operator (not, always, nexttime, accept_on, ...):
    /// Keyword, Expr?(range or condition), Prop
    PropUnary,
    /// `if (e) p [else p]`: Expr, Prop, Prop?
    PropIf,
    /// `case (e) ... endcase`: Expr, PropCaseItem*
    PropCase,
    /// Expr*, Prop
    PropCaseItem,
    /// `@(...) p`: Event, Prop
    PropClocked,
    /// `a ##n b` or a leading `##n b`: Prop?(left), CycleDelay, Prop
    SeqDelay,
    /// `##1`, `##[1:3]`, `##[*]`, `##[+]`: Expr*, Op?
    CycleDelay,
    /// `s[*n]`, `s[=n]`, `s[->n]`, `s[+]`: Prop, Op, Expr?, Expr?
    SeqRepeat,
    /// `(s, a = b)`: Prop, Expr* (match items)
    SeqParen,
    /// `first_match(...)`, `strong(...)`, `weak(...)`: Keyword, Prop, Expr*
    PropCall,
    /// `expect (p) action`: PropertySpec, ActionBlock
    Expect,

    // ---- statements ----
    /// `label: stmt`: Name, Statement
    Labeled,
    /// `;`
    NullStatement,
    /// `begin [: n] ... end` or `fork ... join`: Keyword(begin/fork),
    /// Name?, Item* (declarations), Statement*, Keyword(join kind)?,
    /// EndLabel?
    Block,
    /// `a = b`, `a <= #1 b`, `a += b`: Expr, Op, Timing?, Expr
    Assign,
    /// `x++;` / a call / any expression used as a statement: Expr
    ExprStatement,
    /// `assign`, `deassign`, `force`, `release` in a procedure: Keyword,
    /// Assign or Expr
    ProceduralContinuous,
    /// Keyword(unique/priority)?, CondPredicate, Statement, Statement?
    If,
    /// Expr (with `&&& pattern`s as CondPattern)
    CondPredicate,
    /// `e matches pattern`: Expr, Pattern
    CondPattern,
    /// Keyword(unique...)?, Keyword(case/casex/casez), Expr,
    /// Keyword(inside/matches)?, CaseItem*
    Case,
    /// Expr* (none for default), Statement
    CaseItem,
    /// `randcase`: CaseItem*
    Randcase,
    /// `forever`, `repeat`, `while`: Keyword, Expr?, Statement
    Loop,
    /// `do s while (e);`: Statement, Expr
    DoWhile,
    /// For: ForInit, Expr?, ForStep, Statement
    For,
    /// VarDecl* or Assign*
    ForInit,
    /// Expr* / Assign*
    ForStep,
    /// `foreach (a[i, j]) s`: Expr (array), Name or Empty *, Statement
    Foreach,
    /// `return e;`, `break;`, `continue;`: Keyword, Expr?
    Jump,
    /// `#d s`, `@e s`, `##n s`: Timing, Statement
    Timed,
    /// `#5`, `#(1:2:3)`, `#1ns`: Expr*
    Delay,
    /// `@(...)`, `@x`, `@*`: EventExpr or Expr, or nothing for `@*`
    Event,
    /// `repeat (n) @(e)` before an assignment's value: Expr, Event
    RepeatEvent,
    /// Keyword(edge)?, Expr, Iff?
    EventTerm,
    /// `a or b`, `a, b`: EventTerm*
    EventOr,
    /// `wait (e) s`, `wait fork;`, `wait_order (...)`: Keyword, Expr*,
    /// Statement or ActionBlock?
    Wait,
    /// `-> e;` / `->> #1 e;`: Op, Timing?, Expr
    EventTrigger,
    /// `disable name;` / `disable fork;`: Expr or Keyword
    Disable,
    /// `cb.x <= ##2 v;`: Expr, CycleDelay?, Expr
    ClockingDrive,
    /// `randsequence (main) ...endsequence`: Name?, Production*
    Randsequence,
    /// Type?, Name, TfPorts?, Rule*
    Production,
    /// ProdItem*, Expr?(weight), Block?
    Rule,
    /// Name, Args? / an if, repeat, case or code block in a rule
    ProdItem,

    // ---- expressions ----
    /// `a + b`: Expr, Op, Attributes?, Expr. Also the `->` and `<->`
    /// logical operators.
    Binary,
    /// `-a`, `!a`, `&a`: Op, Attributes?, Expr
    Unary,
    /// `++a` / `a++`: Op, Expr / Expr, Op
    IncDec,
    /// `c ? a : b`: Expr, Expr, Expr
    Conditional,
    /// `a inside {...}`: Expr, RangeList
    Inside,
    /// `{ ... }` of values or ranges: Expr*
    RangeList,
    /// `[a:b]` in a range list, `[a +/- b]`: Expr, Op?, Expr
    ValueRange,
    /// `(e)`: Expr
    Paren,
    /// `a:b:c` (minimum, typical, maximum): Expr, Expr, Expr
    MinTypMax,
    /// `(a = b)` inside an expression: Assign
    AssignExpr,
    /// `{a, b}`: Expr*
    Concat,
    /// `{n{a, b}}`: Expr, Concat
    Replicate,
    /// `{<< 8 {a, b}}`: Op, Expr or Type?, StreamExpr*
    Streaming,
    /// Expr, Expr? (with [range])
    StreamExpr,
    /// `'{a, b}`, `'{x: 1, default: 0}`, `'{3{a}}`: Type?, PatternItem* or
    /// Replicate
    AssignmentPattern,
    /// Key (Expr, Type or Keyword(default)), Expr
    PatternKey,
    /// A pattern in `matches`: `.x`, `.*`, `tagged m p`, `'{...}`, or an
    /// expression: Name?, Pattern*, Expr?
    Pattern,
    /// `tagged m [e]`: Name, Expr?
    Tagged,
    /// `T'(e)`, `8'(e)`, `signed'(e)`: Type or Expr or Keyword, Expr
    Cast,
    /// `a.b`: Expr, Name
    Member,
    /// `p::x`, `$unit::x`, `C#(T)::x`: Expr, Name
    Scope,
    /// `C#(T)` before `::`: Expr, ParamValues
    Specialize,
    /// `a[i]`: Expr, Expr
    Index,
    /// `a[i:j]`, `a[i+:w]`, `a[i-:w]`: Expr, Expr, Op(: +: -:), Expr
    RangeSelect,
    /// `f(a, .b(c))`, `$f(x)`, `obj.m(x)`: Expr, Attributes?, Args?
    Call,
    /// `( ... )` of a call: Expr or NamedArg or Empty *
    Args,
    /// `.name(expr)`: Name, Expr?
    NamedArg,
    /// `with (e)` after an array method, or `with {...}` after randomize:
    /// Expr, Name*, Expr or ConstraintBlock
    With,
    /// `new`, `new(args)`: Args?; `new[n](init)`: Expr, Expr?; `new obj`:
    /// Expr
    New,
    /// A data type used where an expression goes (`$bits(T)`, `#(type)`):
    /// Type
    TypeExpr,
}
