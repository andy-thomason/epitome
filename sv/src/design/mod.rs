//! The design: what elaboration makes of the syntax tree (`docs/design/sv.md`
//! section 7).
//!
//! Like the AST, it is a set of arenas indexed by small ids, and each entry
//! keeps the `NodeId` of the syntax it came from, so an error found at any
//! later stage can point at the source. Names are interned `Sym`s whose
//! text is a slice of the source.
//!
//! The layers, from the outside in:
//!
//! - a **specialisation** is a module, interface or program with one set
//!   of parameter values, elaborated once and shared by its instances;
//! - a **scope** is a specialisation's body, a package, the compilation
//!   unit, a generate block, a named block, a subroutine or a class; it
//!   holds its members in source order and a table of its names;
//! - members are **objects** (variables, nets, parameters), **processes**,
//!   **drivers** (continuous assignments), **cells** (gates and UDPs),
//!   **instances**, **subroutines**, **classes** and nested scopes;
//! - code is **expressions** and **statements**, typed, with every
//!   conversion clause 11 implies written out as a `Convert`.

pub mod elab;
pub mod print;
pub mod types;
pub mod value;

use crate::ast::NodeId;
use std::collections::HashMap;
use types::{TypeId, Types};
use value::Bits;

macro_rules! ids {
    ($($name:ident),*) => {$(
        #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, PartialOrd, Ord)]
        pub struct $name(pub u32);
    )*};
}
ids!(Sym, ScopeId, ObjId, ExprId, StmtId, ProcId, DriverId, CellId, InstId, SpecId, SubId, ClassId, ConstId);

/// Interned names. A name is the text of its identifier, with an escaped
/// identifier's `\` and trailing space removed (5.6.1).
#[derive(Default)]
pub struct Syms<'a> {
    list: Vec<&'a str>,
    map: HashMap<&'a str, Sym>,
}

impl<'a> Syms<'a> {
    pub fn intern(&mut self, s: &'a str) -> Sym {
        if let Some(&id) = self.map.get(s) {
            return id;
        }
        let id = Sym(self.list.len() as u32);
        self.list.push(s);
        self.map.insert(s, id);
        id
    }

    pub fn get(&self, s: Sym) -> &'a str {
        self.list[s.0 as usize]
    }

    pub fn lookup(&self, s: &str) -> Option<Sym> {
        self.map.get(s).copied()
    }
}

/// A constant value.
#[derive(Clone, PartialEq, Debug)]
pub enum Const {
    Bits(Bits),
    Real(f64),
    Str(Vec<u8>),
    /// An unpacked array, struct or other aggregate, element by element.
    Aggregate(Vec<Const>),
    Type(TypeId),
    Null,
    /// `$` given as a parameter's value (6.20.2).
    Unbounded,
}

impl Const {
    pub fn as_bits(&self) -> Option<&Bits> {
        match self {
            Const::Bits(b) => Some(b),
            _ => None,
        }
    }

    pub fn to_i64(&self) -> Option<i64> {
        match self {
            Const::Bits(b) => b.to_i64(),
            Const::Real(r) => Some(r.round() as i64),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ScopeKind {
    /// `$unit`: the compilation unit's own declarations.
    Unit,
    Package,
    /// A module, interface or program specialisation's body.
    Spec,
    /// A generate block, named or not (27).
    Generate,
    /// A `begin`/`fork` block with declarations or a name.
    Block,
    Subroutine,
    Class,
    /// A checker, covergroup, clocking block or other scope kept whole.
    Other,
}

/// What a name in a scope stands for.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Binding {
    Object(ObjId),
    Type(TypeId),
    Subroutine(SubId),
    Scope(ScopeId),
    Instance(InstId),
    /// An array of instances: the first, and how many.
    Instances(InstId, u32),
    Class(ClassId),
    /// A name the elaborator keeps but does not yet model: a covergroup,
    /// property, sequence, `let`, clocking block, modport, checker...
    Other(NodeId),
}

/// A member of a scope, in source order.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Item {
    Object(ObjId),
    Typedef(Sym, TypeId),
    Process(ProcId),
    Driver(DriverId),
    Cell(CellId),
    Instance(InstId),
    Subroutine(SubId),
    Class(ClassId),
    Scope(ScopeId),
    /// An import of a package's name or of `*`.
    Import(ScopeId, Option<Sym>),
    /// Syntax kept as it is: what the elaborator does not model yet
    /// (assertions, coverage, specify blocks, ...).
    Opaque(NodeId),
}

pub struct Scope {
    pub kind: ScopeKind,
    pub name: Option<Sym>,
    pub parent: Option<ScopeId>,
    pub node: NodeId,
    pub items: Vec<Item>,
    pub names: HashMap<Sym, Binding>,
    /// Packages imported with `p::*` (26.3), and the token where each
    /// import is, since a wildcard import only serves names used after it.
    pub wildcards: Vec<ScopeId>,
    pub wildcard_at: Vec<u32>,
    /// An element of a generate loop: its genvar's value.
    pub index: Option<i64>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Dir {
    Input,
    Output,
    Inout,
    Ref,
    ConstRef,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Lifetime {
    Static,
    Automatic,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ObjKind {
    Variable,
    /// A net, with its kind's keyword (`wire`, `tri`, `wand`, `supply0`,
    /// ...) or a user nettype's name.
    Net(Sym),
    /// A parameter, local or not, or a specparam; its value is in `value`.
    Param { local: bool },
    Genvar,
    /// An enum's named value.
    EnumConst,
}

pub struct Object {
    pub name: Sym,
    pub kind: ObjKind,
    pub ty: TypeId,
    pub scope: ScopeId,
    pub node: NodeId,
    pub lifetime: Lifetime,
    /// A port's direction, for the objects that are ports.
    pub dir: Option<Dir>,
    /// The initial value: a variable's initialiser, a net's declaration
    /// assignment.
    pub init: Option<ExprId>,
    /// A parameter's or enum constant's value.
    pub value: Option<ConstId>,
    /// `const`, `rand`, `randc`, `static`, `local`, `protected`, ...
    pub qualifiers: Vec<Sym>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum UnOp {
    Plus,
    Neg,
    Not,
    LogNot,
    RedAnd,
    RedNand,
    RedOr,
    RedNor,
    RedXor,
    RedXnor,
    PreInc,
    PreDec,
    PostInc,
    PostDec,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Pow,
    And,
    Or,
    Xor,
    Xnor,
    Shl,
    Shr,
    AShl,
    AShr,
    LogAnd,
    LogOr,
    Implies,
    Equiv,
    Eq,
    Ne,
    CaseEq,
    CaseNe,
    WildEq,
    WildNe,
    Lt,
    Le,
    Gt,
    Ge,
}

impl BinOp {
    pub fn text(self) -> &'static str {
        use BinOp::*;
        match self {
            Add => "+",
            Sub => "-",
            Mul => "*",
            Div => "/",
            Mod => "%",
            Pow => "**",
            And => "&",
            Or => "|",
            Xor => "^",
            Xnor => "~^",
            Shl => "<<",
            Shr => ">>",
            AShl => "<<<",
            AShr => ">>>",
            LogAnd => "&&",
            LogOr => "||",
            Implies => "->",
            Equiv => "<->",
            Eq => "==",
            Ne => "!=",
            CaseEq => "===",
            CaseNe => "!==",
            WildEq => "==?",
            WildNe => "!=?",
            Lt => "<",
            Le => "<=",
            Gt => ">",
            Ge => ">=",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum ExprKind {
    Const(ConstId),
    Object(ObjId),
    /// A reference through the hierarchy, kept as its path of names and
    /// resolved for each instance (23.6). `object` is what it found for the
    /// first instance, which gives its type.
    Hier { path: Vec<Sym>, object: Option<ObjId> },
    Unary(UnOp),
    Binary(BinOp),
    /// `c ? a : b`: [c, a, b]
    Cond,
    Concat,
    /// `{n{...}}`: [the concatenation], with `n` evaluated.
    Replicate(u32),
    /// An element or bit select, `a[i]`: [a, i]. The index is the one
    /// written; `lsb_offset` turns it into a bit offset for packed types.
    Index,
    /// A part select, `a[i:j]`, `a[i+:w]`, `a[i-:w]`: [a, i] (and [a, i, j]
    /// for a constant range), the width, and the kind.
    Part { kind: PartKind, width: u32 },
    /// A struct or union member: [the value], the member's index.
    Member(u32),
    /// A class property through a handle: [the handle] (or none for a
    /// static property named through the class).
    Field(ObjId),
    /// A conversion to the expression's type: [the value]. Integral
    /// values are extended by the target's signedness (11.8.2) or
    /// truncated; reals round; four-state to two-state makes x and z 0.
    Convert,
    /// A user-defined function or task: the arguments, in the order of the
    /// formals, with defaults filled in.
    Call(SubId),
    /// A system function: `$bits`, `$clog2`, `$display`, ...
    SysCall(Sym),
    /// A method of a class, or of a built-in type (an array's `size`, a
    /// string's `len`, an enum's `next`): [the object, arguments...].
    Method(Sym),
    /// An assignment pattern, `'{...}`, in element order.
    Pattern,
    /// `inside`: [the value, the ranges...]; a range is a `Range`.
    Inside,
    /// `[a:b]` in `inside`, `dist` and the like: [a, b]
    Range,
    /// `{<< n {...}}` / `{>> {...}}`: [the slice size?, the parts...]
    Streaming { left: bool },
    /// `new` and `new[n]`
    New,
    /// An assignment used as an expression, `(a = b)`: [a, b]
    Assign,
    /// `$` as an index or bound.
    Dollar,
    Null,
    This,
    /// A type used as a value: `$bits(T)`, a type parameter's value.
    TypeRef(TypeId),
    /// A tagged union value: [the value?], the member.
    Tagged(u32),
    /// Syntax the elaborator does not model yet, with its type unknown.
    Opaque,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PartKind {
    Range,
    Up,
    Down,
}

#[derive(Clone, Debug)]
pub struct Expr {
    pub kind: ExprKind,
    pub ty: TypeId,
    pub node: NodeId,
    pub args: Vec<ExprId>,
}

/// A delay or event control (9.4).
#[derive(Clone, Debug, PartialEq)]
pub enum Timing {
    /// `#d`: the delay, or rise, fall and turn-off delays.
    Delay(Vec<ExprId>),
    /// `@(...)`: the terms, or-ed.
    Event(Vec<EventTerm>),
    /// `@*` and `@(*)`, with the terms worked out from what the statement
    /// reads (9.4.2.2).
    Implicit(Vec<EventTerm>),
    /// `##n`
    Cycle(ExprId),
    /// `repeat (n) @(...)` on an assignment.
    Repeat(ExprId, Vec<EventTerm>),
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Edge {
    Any,
    Pos,
    Neg,
    Both,
}

#[derive(Clone, Debug, PartialEq)]
pub struct EventTerm {
    pub edge: Edge,
    pub expr: ExprId,
    pub iff: Option<ExprId>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum AssignKind {
    Blocking,
    NonBlocking,
    /// `+=` and the other compound forms (11.4.1): the operator.
    Compound(BinOp),
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Join {
    All,
    Any,
    None,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CaseKind {
    Case,
    CaseZ,
    CaseX,
    Inside,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Check {
    None,
    Unique,
    Unique0,
    Priority,
}

#[derive(Clone, Debug)]
pub enum StmtKind {
    Null,
    /// `begin ... end` or `fork ... join*`, with its scope if it declares
    /// anything or has a name.
    Block { join: Option<Join>, scope: Option<ScopeId>, stmts: Vec<StmtId> },
    Assign { kind: AssignKind, lhs: ExprId, rhs: ExprId, timing: Option<Timing> },
    /// A call, increment, or other expression run for its effect.
    Expr(ExprId),
    If { check: Check, cond: ExprId, then: StmtId, else_: Option<StmtId> },
    Case { check: Check, kind: CaseKind, expr: ExprId, items: Vec<(Vec<ExprId>, StmtId)>, default: Option<StmtId> },
    Forever(StmtId),
    Repeat(ExprId, StmtId),
    While(ExprId, StmtId),
    DoWhile(StmtId, ExprId),
    /// `for`: its scope (for loop variables), the initialisation, the
    /// condition, the steps and the body.
    For { scope: Option<ScopeId>, init: Vec<StmtId>, cond: Option<ExprId>, step: Vec<StmtId>, body: StmtId },
    /// `foreach (a[i, j])`: the array, the loop variables (none for a
    /// skipped one), the body.
    Foreach { scope: ScopeId, array: ExprId, vars: Vec<Option<ObjId>>, body: StmtId },
    Timed(Timing, StmtId),
    Wait(ExprId, StmtId),
    WaitFork,
    Return(Option<ExprId>),
    Break,
    Continue,
    Disable(ExprId),
    DisableFork,
    /// `-> e` or `->> e`
    Trigger { nonblocking: bool, event: ExprId },
    /// `assign`, `deassign`, `force`, `release` in a procedure (10.6).
    ProcContinuous { keyword: Sym, lhs: ExprId, rhs: Option<ExprId> },
    /// An immediate or deferred assertion (16.3, 16.4).
    Assert { keyword: Sym, cond: ExprId, pass: Option<StmtId>, fail: Option<StmtId> },
    /// What the elaborator does not model yet: kept as syntax.
    Opaque,
}

#[derive(Clone, Debug)]
pub struct Stmt {
    pub kind: StmtKind,
    pub node: NodeId,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ProcKind {
    Initial,
    Always,
    AlwaysComb,
    AlwaysLatch,
    AlwaysFf,
    Final,
}

/// Which half of a time slot a process runs in (4.4.1).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum RegionSet {
    Active,
    Reactive,
}

pub struct Process {
    pub kind: ProcKind,
    pub body: StmtId,
    pub scope: ScopeId,
    pub region: RegionSet,
    pub node: NodeId,
    /// What the process may read and write, through the subroutines it
    /// calls too (section 8.5).
    pub reads: Vec<ObjId>,
    pub writes: Vec<ObjId>,
    /// For `always_comb`, `always_latch` and `always @*`, what it is
    /// sensitive to (9.2.2.2).
    pub sensitivity: Vec<ObjId>,
}

/// A continuous assignment, or a port connection that acts as one (4.9.6).
pub struct Driver {
    pub lhs: ExprId,
    pub rhs: ExprId,
    pub strength: Option<(Sym, Sym)>,
    pub delay: Vec<ExprId>,
    pub node: NodeId,
    pub scope: ScopeId,
}

/// A gate, switch, pull or UDP instance (28, 29).
pub struct Cell {
    /// The gate's keyword or the UDP's name.
    pub kind: Sym,
    pub name: Option<Sym>,
    pub terminals: Vec<ExprId>,
    pub strength: Vec<Sym>,
    pub delay: Vec<ExprId>,
    pub node: NodeId,
}

/// How a port of an instance is connected.
#[derive(Clone, Debug)]
pub struct Connection {
    pub port: Sym,
    /// The expression in the parent, or `None` if left open.
    pub expr: Option<ExprId>,
    /// An interface instance bound to an interface port.
    pub interface: Option<InstId>,
}

pub struct Instance {
    pub name: Sym,
    pub spec: SpecId,
    /// The scope the instance is declared in.
    pub scope: ScopeId,
    pub node: NodeId,
    pub connections: Vec<Connection>,
    /// The index for an element of an array of instances.
    pub index: Option<i64>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum DefKind {
    Module,
    Interface,
    Program,
}

/// A design element with one set of parameter values.
pub struct Spec {
    pub kind: DefKind,
    pub name: Sym,
    pub def: NodeId,
    pub params: Vec<(Sym, ConstId)>,
    pub scope: ScopeId,
    /// The ports, in order: their names and the objects (or interface
    /// ports) inside.
    pub ports: Vec<Port>,
    pub instances: u32,
}

#[derive(Clone, Debug)]
pub struct Port {
    pub name: Sym,
    pub dir: Dir,
    pub object: Option<ObjId>,
    /// For an interface port: the interface's name and modport.
    pub interface: Option<(Sym, Option<Sym>)>,
    pub node: NodeId,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SubKind {
    Function,
    Task,
}

pub struct Subroutine {
    pub kind: SubKind,
    pub name: Sym,
    pub ret: TypeId,
    pub args: Vec<ObjId>,
    /// Defaults for the arguments that have them.
    pub defaults: Vec<Option<NodeId>>,
    pub scope: ScopeId,
    pub body: Vec<StmtId>,
    pub lifetime: Lifetime,
    pub node: NodeId,
    /// `virtual`, `pure`, `static`, `extern`, `local`, `protected`, and for
    /// a DPI import, `import`.
    pub qualifiers: Vec<Sym>,
    /// Bodies are elaborated after every declaration, so that calls can
    /// come before the subroutine.
    pub elaborated: bool,
}

pub struct Class {
    pub name: Sym,
    pub scope: ScopeId,
    pub base: Option<ClassId>,
    pub node: NodeId,
    pub virtual_: bool,
    pub interface: bool,
    pub params: Vec<(Sym, ConstId)>,
    /// The interface classes it implements (8.26).
    pub implements: Vec<ClassId>,
}

/// The whole design.
pub struct Design<'a> {
    pub syms: Syms<'a>,
    pub types: Types,
    pub consts: Vec<Const>,
    pub scopes: Vec<Scope>,
    pub objects: Vec<Object>,
    pub exprs: Vec<Expr>,
    pub stmts: Vec<Stmt>,
    pub processes: Vec<Process>,
    pub drivers: Vec<Driver>,
    pub cells: Vec<Cell>,
    pub instances: Vec<Instance>,
    pub specs: Vec<Spec>,
    pub subroutines: Vec<Subroutine>,
    pub classes: Vec<Class>,
    /// The compilation unit's scope.
    pub unit: ScopeId,
    pub packages: Vec<ScopeId>,
    /// The top-level instances (23.3.1).
    pub tops: Vec<InstId>,
}

impl<'a> Design<'a> {
    pub fn scope(&self, id: ScopeId) -> &Scope {
        &self.scopes[id.0 as usize]
    }

    pub fn object(&self, id: ObjId) -> &Object {
        &self.objects[id.0 as usize]
    }

    pub fn expr(&self, id: ExprId) -> &Expr {
        &self.exprs[id.0 as usize]
    }

    pub fn stmt(&self, id: StmtId) -> &Stmt {
        &self.stmts[id.0 as usize]
    }

    pub fn konst(&self, id: ConstId) -> &Const {
        &self.consts[id.0 as usize]
    }

    pub fn spec(&self, id: SpecId) -> &Spec {
        &self.specs[id.0 as usize]
    }

    pub fn instance(&self, id: InstId) -> &Instance {
        &self.instances[id.0 as usize]
    }

    pub fn sub(&self, id: SubId) -> &Subroutine {
        &self.subroutines[id.0 as usize]
    }

    pub fn class(&self, id: ClassId) -> &Class {
        &self.classes[id.0 as usize]
    }

    pub fn name(&self, s: Sym) -> &'a str {
        self.syms.get(s)
    }

    /// How many expressions have a type the elaborator could not work out,
    /// and how many items and statements it kept as syntax.
    pub fn gaps(&self) -> (usize, usize) {
        let unknown = self.exprs.iter().filter(|e| e.ty == Types::UNKNOWN).count();
        let opaque = self.scopes.iter().flat_map(|s| &s.items).filter(|i| matches!(i, Item::Opaque(_))).count()
            + self.stmts.iter().filter(|s| matches!(s.kind, StmtKind::Opaque)).count();
        (unknown, opaque)
    }
}
