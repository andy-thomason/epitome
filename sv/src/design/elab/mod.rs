//! Elaboration: the syntax tree to a `Design` (clauses 3.12, 23, 26, 27).
//!
//! It runs in three passes over each scope:
//!
//! 1. **Declare** subroutines, classes and the names of generate blocks
//!    and instances, so that code can call a function written further
//!    down and name a block before it.
//! 2. **Elaborate declarations in order**: parameters, types, objects,
//!    imports, generate constructs (evaluated and unrolled) and instances
//!    (whose specialisations are elaborated, recursively, at that point).
//! 3. **Elaborate code**: processes, continuous assignments, port
//!    connections, gates and subroutine bodies. These are queued during
//!    pass 2 and run once every declaration in the design exists, since
//!    they may name anything in the hierarchy.
//!
//! The first error stops elaboration, as in the parser.

mod checks;
mod eval;
mod expr;
mod lookup;
mod stmt;
mod ty;

use super::types::{Type, TypeId, Types};
use super::value::Bits;
use super::*;
use crate::ast::{Ast, Kind, NodeId};
use crate::lexer::Token;
use crate::preprocess::{NetType, Output, State};
use crate::source::Error;
use std::collections::HashMap;

pub(crate) type R<'a, T = ()> = Result<T, Error<'a>>;

/// Elaborates a parsed compilation unit.
pub fn elaborate<'a>(ast: &Ast<'a>, out: &Output<'a>) -> Result<Design<'a>, Error<'a>> {
    let mut e = Elab::new(ast, out);
    e.run()?;
    Ok(e.d)
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum DefKind2 {
    Design(DefKind),
    Udp,
    Checker,
    Config,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct Def {
    pub kind: DefKind2,
    pub node: NodeId,
}

/// Work queued for pass 3, with the instance path it was found under.
#[derive(Clone, Debug)]
pub(crate) enum Work {
    Process(ScopeId, NodeId),
    Assign(ScopeId, NodeId),
    Gate(ScopeId, NodeId, Sym),
    Connect(ScopeId, InstId, NodeId),
    Body(SubId),
    NetInit(ObjId, NodeId),
    Alias(ScopeId, NodeId),
    ElabTask(ScopeId, NodeId),
}

pub(crate) struct Elab<'a, 'x> {
    pub ast: &'x Ast<'a>,
    pub out: &'x Output<'a>,
    pub d: Design<'a>,
    pub defs: HashMap<Sym, Def>,
    pub packages: HashMap<Sym, ScopeId>,
    pub spec_cache: HashMap<(NodeId, String), SpecId>,
    pub work: Vec<(Work, Vec<InstId>)>,
    /// The instances being elaborated, outermost first.
    pub inst_path: Vec<InstId>,
    /// Syntax nodes of subroutines whose bodies are not elaborated yet.
    pub sub_nodes: HashMap<SubId, (NodeId, ScopeId)>,
    /// Class declarations not yet elaborated, by scope and name.
    pub class_nodes: HashMap<ClassId, (NodeId, ScopeId)>,
    /// The scope that each class's methods were declared in.
    pub depth: usize,
    /// Whether the spec being elaborated refers upwards through the
    /// hierarchy, which stops it being shared.
    pub upward: bool,
    /// Names looked up through a wildcard import, by the scope they were
    /// looked up from, for 26.3's later-declaration rule.
    pub imported_uses: HashMap<(ScopeId, Sym), NodeId>,
    /// Generate-for loops being unrolled: their genvars (27.4).
    pub genvars_active: Vec<ObjId>,
    /// The subroutine whose body is being elaborated.
    pub current_sub: Option<SubId>,
    /// Whether the code being elaborated is inside a loop, for `break`.
    pub loop_depth: usize,
    /// Calls made by each subroutine body, for read and write sets.
    pub calls: HashMap<SubId, Vec<SubId>>,
    pub std_pkg: Option<ScopeId>,
    /// The time unit given by `timeunit` in each design element (3.14).
    pub timeunits: HashMap<ScopeId, (Option<i32>, Option<i32>)>,
    /// Parameter overrides for the specialisation being elaborated.
    pub pending_overrides: Vec<(ScopeId, Vec<(Sym, Over)>)>,
    /// Names of conditional generate blocks, taken or not, by scope.
    pub reserved_blocks: HashMap<ScopeId, Vec<(Sym, NodeId)>>,
    /// How many unnamed generate constructs each scope has had (27.6).
    pub genblk_count: HashMap<ScopeId, u32>,
    /// Subroutines whose signatures are elaborated.
    pub sig_done: std::collections::HashSet<SubId>,
    /// Method bodies written outside their class (8.24): the class, the
    /// method's name, the body, and the class's scope.
    pub out_of_block: Vec<(ClassId, Sym, NodeId, ScopeId)>,
    /// The kind of the process whose body is being elaborated.
    pub proc_kind: Option<ProcKind>,
    /// Whether a continuous assignment is being elaborated.
    pub in_cont_assign: bool,
    /// `defparam`s, checked once the hierarchy exists.
    pub defparams: Vec<(ScopeId, NodeId, Vec<InstId>)>,
    /// `extern module` declarations, by name (23.4).
    pub externs: HashMap<Sym, NodeId>,
    /// Definitions nested in another, which are not top-level.
    pub nested: std::collections::HashSet<NodeId>,
    /// Names declared by `nettype` (6.6.7).
    pub nettypes: std::collections::HashSet<Sym>,
    /// Whether the connections of an element of an array of instances are
    /// being made; they are split across the array (23.3.3.5).
    pub connecting_array: bool,
    /// Whether the design uses `bind` (23.11).
    pub has_bind: bool,
    /// The scope a nested definition is being instantiated from.
    pub nested_parent: Option<ScopeId>,
}

impl<'a, 'x> Elab<'a, 'x> {
    fn new(ast: &'x Ast<'a>, out: &'x Output<'a>) -> Self {
        let d = Design {
            syms: Syms::default(),
            types: Types::new(),
            consts: Vec::new(),
            scopes: Vec::new(),
            objects: Vec::new(),
            exprs: Vec::new(),
            stmts: Vec::new(),
            processes: Vec::new(),
            drivers: Vec::new(),
            cells: Vec::new(),
            instances: Vec::new(),
            specs: Vec::new(),
            subroutines: Vec::new(),
            classes: Vec::new(),
            unit: ScopeId(0),
            packages: Vec::new(),
            tops: Vec::new(),
        };
        Elab {
            ast,
            out,
            d,
            defs: HashMap::new(),
            packages: HashMap::new(),
            spec_cache: HashMap::new(),
            work: Vec::new(),
            inst_path: Vec::new(),
            sub_nodes: HashMap::new(),
            class_nodes: HashMap::new(),
            depth: 0,
            upward: false,
            imported_uses: HashMap::new(),
            genvars_active: Vec::new(),
            current_sub: None,
            loop_depth: 0,
            calls: HashMap::new(),
            std_pkg: None,
            timeunits: HashMap::new(),
            pending_overrides: Vec::new(),
            reserved_blocks: HashMap::new(),
            genblk_count: HashMap::new(),
            sig_done: Default::default(),
            out_of_block: Vec::new(),
            proc_kind: None,
            in_cont_assign: false,
            defparams: Vec::new(),
            externs: HashMap::new(),
            nested: Default::default(),
            nettypes: Default::default(),
            connecting_array: false,
            has_bind: ast.nodes.iter().any(|n| n.kind == Kind::Bind),
            nested_parent: None,
        }
    }

    // ---- small helpers ----

    pub fn err(&self, n: NodeId, msg: impl Into<String>) -> Error<'a> {
        let t = self.ast.token(n);
        let at = if t.0.is_empty() {
            // An empty node: point at the token where it would have been.
            let i = self.ast.node(n).first as usize;
            self.ast.tokens.get(i).or(self.ast.tokens.last()).map_or("", |t| t.0)
        } else {
            t.0
        };
        Error::new(at, msg)
    }

    pub fn kind(&self, n: NodeId) -> Kind {
        self.ast.kind(n)
    }

    /// A node's children, without the attributes in front of them.
    pub fn kids(&self, n: NodeId) -> &'x [NodeId] {
        let k = self.ast.children(n);
        let skip = k.iter().take_while(|&&c| self.ast.kind(c) == Kind::Attributes).count();
        &k[skip..]
    }

    pub fn child(&self, n: NodeId, k: Kind) -> Option<NodeId> {
        self.ast.child(n, k)
    }

    pub fn tok(&self, n: NodeId) -> Token<'a> {
        self.ast.token(n)
    }

    /// The text of a leaf: a keyword, operator or name.
    pub fn text(&self, n: NodeId) -> &'a str {
        self.ast.token(n).0
    }

    pub fn sym_of(&mut self, n: NodeId) -> Sym {
        let t = self.ast.token(n);
        let s = if t.is_escaped_ident() { t.ident_name() } else { t.0 };
        self.d.syms.intern(s)
    }

    pub fn sym(&mut self, s: &'static str) -> Sym {
        self.d.syms.intern(s)
    }

    /// A made-up name (`genblk1`), kept for the life of the program.
    pub fn sym_owned(&mut self, s: String) -> Sym {
        if let Some(id) = self.d.syms.lookup(&s) {
            return id;
        }
        let leaked: &'static str = Box::leak(s.into_boxed_str());
        self.d.syms.intern(leaked)
    }

    pub fn name(&self, s: Sym) -> &'a str {
        self.d.syms.get(s)
    }

    pub fn state(&self, n: NodeId) -> &'x State {
        let i = self.ast.node(n).first as usize;
        match self.out.tokens.get(i).or(self.out.tokens.last()) {
            Some(&t) => self.out.state(t),
            None => &self.out.states[0],
        }
    }

    pub fn keywords_of(&self, n: NodeId) -> Vec<&'a str> {
        self.kids(n).iter().filter(|&&k| self.kind(k) == Kind::Keyword).map(|&k| self.text(k)).collect()
    }

    pub fn has_keyword(&self, n: NodeId, kw: &str) -> bool {
        self.kids(n).iter().any(|&k| self.kind(k) == Kind::Keyword && self.text(k) == kw)
    }

    pub fn new_scope(&mut self, kind: ScopeKind, name: Option<Sym>, parent: Option<ScopeId>, node: NodeId) -> ScopeId {
        let id = ScopeId(self.d.scopes.len() as u32);
        self.d.scopes.push(Scope {
            kind,
            name,
            parent,
            node,
            items: Vec::new(),
            names: HashMap::new(),
            wildcards: Vec::new(),
            wildcard_at: Vec::new(),
            index: None,
        });
        id
    }

    pub fn scope(&self, s: ScopeId) -> &Scope {
        &self.d.scopes[s.0 as usize]
    }

    pub fn scope_mut(&mut self, s: ScopeId) -> &mut Scope {
        &mut self.d.scopes[s.0 as usize]
    }

    pub fn push_item(&mut self, s: ScopeId, item: Item) {
        self.scope_mut(s).items.push(item);
    }

    /// Declares `name` in `s`, failing if the scope already has it (3.13,
    /// 6.21).
    pub fn declare(&mut self, s: ScopeId, name: Sym, b: Binding, at: NodeId) -> R<'a> {
        if self.imported_uses.contains_key(&(s, name)) && self.scope(s).names.get(&name) != Some(&b) {
            return Err(self.err(
                at,
                format!(
                    "`{}` is declared after a reference in this scope imported it from a package (26.3)",
                    self.name(name)
                ),
            ));
        }
        if let Some(old) = self.scope(s).names.get(&name).copied()
            && old != b
        {
            let what = self.describe_binding(old);
            return Err(self.err(at, format!("`{}` is already declared in this scope, as {what}", self.name(name))));
        }
        self.scope_mut(s).names.insert(name, b);
        Ok(())
    }

    pub fn describe_binding(&self, b: Binding) -> &'static str {
        match b {
            Binding::Object(o) => match self.d.objects[o.0 as usize].kind {
                ObjKind::Variable => "a variable",
                ObjKind::Net(_) => "a net",
                ObjKind::Param { .. } => "a parameter",
                ObjKind::Genvar => "a genvar",
                ObjKind::EnumConst => "an enum value",
            },
            Binding::Type(_) => "a type",
            Binding::Subroutine(_) => "a task or function",
            Binding::Scope(_) => "a block",
            Binding::Instance(_) | Binding::Instances(..) => "an instance",
            Binding::Class(_) => "a class",
            Binding::Other(_) => "another named item",
        }
    }

    pub fn new_object(&mut self, obj: Object) -> ObjId {
        let id = ObjId(self.d.objects.len() as u32);
        self.d.objects.push(obj);
        id
    }

    pub fn obj(&self, o: ObjId) -> &Object {
        &self.d.objects[o.0 as usize]
    }

    pub fn obj_mut(&mut self, o: ObjId) -> &mut Object {
        &mut self.d.objects[o.0 as usize]
    }

    pub fn new_const(&mut self, c: Const) -> ConstId {
        let id = ConstId(self.d.consts.len() as u32);
        self.d.consts.push(c);
        id
    }

    pub fn new_expr(&mut self, kind: ExprKind, ty: TypeId, node: NodeId, args: Vec<ExprId>) -> ExprId {
        let id = ExprId(self.d.exprs.len() as u32);
        self.d.exprs.push(Expr { kind, ty, node, args });
        id
    }

    pub fn ex(&self, e: ExprId) -> &Expr {
        &self.d.exprs[e.0 as usize]
    }

    pub fn new_stmt(&mut self, kind: StmtKind, node: NodeId) -> StmtId {
        let id = StmtId(self.d.stmts.len() as u32);
        self.d.stmts.push(Stmt { kind, node });
        id
    }

    pub fn defer(&mut self, w: Work) {
        let path = self.inst_path.clone();
        self.work.push((w, path));
    }

    /// The first `Name` child.
    pub fn name_child(&self, n: NodeId) -> Option<NodeId> {
        self.child(n, Kind::Name)
    }

    // ---- the driver ----

    fn run(&mut self) -> R<'a> {
        let root = self.ast.root;
        self.d.unit = self.new_scope(ScopeKind::Unit, None, None, root);
        self.std_package();
        let unit = self.d.unit;
        // Definitions first: they form a global name space (3.13).
        for &item in self.kids(root) {
            self.collect_def(item)?;
        }
        // Then the compilation unit's own items and packages, in order.
        self.declare_pass(unit, self.kids(root))?;
        for &item in self.kids(root) {
            match self.kind(item) {
                Kind::Module | Kind::Udp | Kind::Checker | Kind::Config | Kind::ExternModule | Kind::Library => {}
                Kind::Package => self.package(item)?,
                _ => self.item(unit, item)?,
            }
        }
        checks::unit_timeunits(self, root)?;
        self.run_work()?;
        // Top-level instances: design elements that nothing instantiates.
        let used = self.instantiated_names();
        let mut tops: Vec<(Sym, Def)> = Vec::new();
        for &item in self.kids(root) {
            if matches!(self.kind(item), Kind::Module | Kind::AnonymousProgram) {
                if self.kind(item) == Kind::AnonymousProgram {
                    continue;
                }
                let Some(name) = self.name_child(item) else { continue };
                let s = self.sym_of(name);
                if let Some(&def) = self.defs.get(&s)
                    && def.node == item
                    && !used.contains(&s)
                    && !self.nested.contains(&item)
                {
                    tops.push((s, def));
                }
            }
        }
        for (s, def) in tops {
            let DefKind2::Design(_) = def.kind else { continue };
            let inst = self.instantiate(None, unit, s, def, &[], def.node, None)?;
            self.d.tops.push(inst);
            self.run_work()?;
        }
        checks::after(self)?;
        Ok(())
    }

    fn run_work(&mut self) -> R<'a> {
        let mut i = 0;
        while i < self.work.len() {
            let (w, path) = self.work[i].clone();
            let saved = std::mem::replace(&mut self.inst_path, path);
            let r = self.do_work(w);
            self.inst_path = saved;
            r?;
            i += 1;
        }
        self.work.clear();
        Ok(())
    }

    fn do_work(&mut self, w: Work) -> R<'a> {
        match w {
            Work::Process(sc, n) => stmt::process(self, sc, n),
            Work::Assign(sc, n) => self.continuous_assign(sc, n),
            Work::Gate(sc, n, kind) => self.gate(sc, n, kind),
            Work::Connect(sc, inst, n) => self.connect(sc, inst, n),
            Work::Body(sub) => stmt::sub_body(self, sub),
            Work::NetInit(obj, n) => {
                let sc = self.obj(obj).scope;
                let ty = self.obj(obj).ty;
                let rhs = expr::rvalue(self, sc, n, ty)?;
                let lhs = self.new_expr(ExprKind::Object(obj), ty, n, vec![]);
                self.obj_mut(obj).init = Some(rhs);
                let id = DriverId(self.d.drivers.len() as u32);
                self.d.drivers.push(Driver { lhs, rhs, strength: None, delay: vec![], node: n, scope: sc });
                self.push_item(sc, Item::Driver(id));
                Ok(())
            }
            Work::Alias(sc, n) => {
                let mut parts = Vec::new();
                for &k in self.kids(n) {
                    parts.push(expr::expr(self, sc, k)?);
                }
                checks::net_alias(self, n, &parts)?;
                self.push_item(sc, Item::Opaque(n));
                Ok(())
            }
            Work::ElabTask(sc, n) => checks::elaboration_task(self, sc, n),
        }
    }

    /// Whether the syntax under `n` instantiates `s`.
    fn instantiated_in(&mut self, n: NodeId, s: Sym) -> bool {
        let text = self.name(s);
        let mut stack = vec![n];
        while let Some(x) = stack.pop() {
            if self.kind(x) == Kind::Instantiation && self.name_child(x).is_some_and(|k| self.text(k) == text) {
                return true;
            }
            stack.extend(self.ast.children(x).iter().copied());
        }
        false
    }

    /// The names used as the definition of an instance anywhere.
    fn instantiated_names(&mut self) -> std::collections::HashSet<Sym> {
        let mut used = std::collections::HashSet::new();
        for i in 0..self.ast.nodes.len() {
            let n = NodeId(i as u32);
            match self.kind(n) {
                Kind::Instantiation => {
                    if let Some(name) = self.name_child(n) {
                        let s = self.sym_of(name);
                        used.insert(s);
                    }
                }
                Kind::Bind => {
                    // The bound module is instantiated; so is the target,
                    // if named as a module.
                    if let Some(inst) = self.child(n, Kind::Instantiation)
                        && let Some(name) = self.name_child(inst)
                    {
                        let s = self.sym_of(name);
                        used.insert(s);
                    }
                }
                _ => {}
            }
        }
        used
    }

    /// A built-in `std` package (Annex G), with the names it holds as
    /// classes kept opaque.
    fn std_package(&mut self) {
        let root = self.ast.root;
        let s = self.sym("std");
        let unit = self.d.unit;
        let sc = self.new_scope(ScopeKind::Package, Some(s), Some(unit), root);
        for name in ["process", "semaphore", "mailbox", "randomize", "weak_reference"] {
            let n = self.sym(name);
            self.scope_mut(sc).names.insert(n, Binding::Other(root));
        }
        self.std_pkg = Some(sc);
        self.packages.insert(s, sc);
    }

    fn collect_def(&mut self, item: NodeId) -> R<'a> {
        if self.kind(item) == Kind::ExternModule {
            if let Some(n) = self.name_child(item) {
                let s = self.sym_of(n);
                self.externs.insert(s, item);
            }
            return Ok(());
        }
        // Nested definitions (23.4) are found too; they are visible only
        // inside their parent, which is not modelled yet.
        if self.kind(item) == Kind::Module {
            for &k in self.kids(item).to_vec().iter() {
                if matches!(self.kind(k), Kind::Module | Kind::ExternModule) {
                    let nested = k;
                    if let Some(n) = self.name_child(nested) {
                        let s = self.sym_of(n);
                        if !self.defs.contains_key(&s) {
                            self.collect_def(nested)?;
                            self.nested.insert(nested);
                        }
                    }
                }
            }
        }
        let kind = match self.kind(item) {
            Kind::Module => {
                let kw = self.text(self.kids(item)[0]);
                DefKind2::Design(match kw {
                    "interface" => DefKind::Interface,
                    "program" => DefKind::Program,
                    _ => DefKind::Module,
                })
            }
            Kind::Udp => DefKind2::Udp,
            Kind::Checker => DefKind2::Checker,
            Kind::Config => DefKind2::Config,
            Kind::Package => {
                let name = self.name_child(item).unwrap();
                let s = self.sym_of(name);
                if self.packages.contains_key(&s) && s != self.sym("std") {
                    return Err(self.err(name, format!("package `{}` is already defined (3.13)", self.name(s))));
                }
                return Ok(());
            }
            _ => return Ok(()),
        };
        let Some(name) = self.name_child(item) else { return Ok(()) };
        let s = self.sym_of(name);
        if let Some(old) = self.defs.get(&s) {
            let what = match old.kind {
                DefKind2::Design(DefKind::Module) => "a module",
                DefKind2::Design(DefKind::Interface) => "an interface",
                DefKind2::Design(DefKind::Program) => "a program",
                DefKind2::Udp => "a primitive",
                DefKind2::Checker => "a checker",
                DefKind2::Config => "a configuration",
            };
            if !(kind == DefKind2::Config) && old.kind != DefKind2::Config {
                return Err(self.err(
                    name,
                    format!("`{}` is already defined as {what}; definitions share one name space (3.13)", self.name(s)),
                ));
            }
        }
        if kind != DefKind2::Config || !self.defs.contains_key(&s) {
            self.defs.insert(s, Def { kind, node: item });
        }
        Ok(())
    }

    fn package(&mut self, n: NodeId) -> R<'a> {
        let name = self.name_child(n).unwrap();
        let s = self.sym_of(name);
        let unit = self.d.unit;
        let sc = self.new_scope(ScopeKind::Package, Some(s), Some(unit), n);
        self.packages.insert(s, sc);
        self.d.packages.push(sc);
        let items: Vec<NodeId> =
            self.kids(n).iter().copied().filter(|&k| !matches!(self.kind(k), Kind::Name | Kind::Keyword | Kind::EndLabel)).collect();
        self.declare_pass(sc, &items)?;
        for k in items {
            self.item(sc, k)?;
        }
        checks::package_exports(self, sc, n)?;
        Ok(())
    }

    // ---- pass 1 ----

    /// Declares the subroutines and classes among `items`, so that code
    /// before them can use them.
    pub fn declare_pass(&mut self, sc: ScopeId, items: &[NodeId]) -> R<'a> {
        for &k in items {
            match self.kind(k) {
                Kind::Function | Kind::Task => {
                    stmt::declare_sub(self, sc, k)?;
                }
                Kind::Class | Kind::InterfaceClass => {
                    ty::declare_class(self, sc, k)?;
                }
                Kind::GenerateRegion => {
                    let inner: Vec<NodeId> = self.kids(k).to_vec();
                    self.declare_pass(sc, &inner)?;
                }
                _ => {}
            }
        }
        Ok(())
    }

    // ---- pass 2: one item ----

    pub fn item(&mut self, sc: ScopeId, n: NodeId) -> R<'a> {
        match self.kind(n) {
            Kind::Attributes | Kind::EndLabel | Kind::Empty | Kind::NullStatement => {}
            Kind::ParamDecl => self.param_decl(sc, n, false)?,
            Kind::TypeParamDecl => self.type_param_decl(sc, n, None)?,
            Kind::SpecparamDecl => {
                for &a in self.kids(n) {
                    if self.kind(a) == Kind::SpecparamAssign {
                        let kids = self.kids(a);
                        let name = self.sym_of(kids[0]);
                        let v = eval::const_expr(self, sc, kids[1]).unwrap_or(Const::Bits(Bits::x(32)));
                        let ty = eval::type_of_const(self, &v);
                        let c = self.new_const(v);
                        let o = self.new_object(Object {
                            name,
                            kind: ObjKind::Param { local: true },
                            ty,
                            scope: sc,
                            node: a,
                            lifetime: Lifetime::Static,
                            dir: None,
                            init: None,
                            value: Some(c),
                            qualifiers: vec![],
                        });
                        self.declare(sc, name, Binding::Object(o), kids[0])?;
                        self.push_item(sc, Item::Object(o));
                    }
                }
            }
            Kind::Typedef => ty::typedef(self, sc, n)?,
            Kind::ForwardTypedef => {
                let name = self.name_child(n).unwrap();
                let s = self.sym_of(name);
                if !self.scope(sc).names.contains_key(&s) {
                    self.scope_mut(sc).names.insert(s, Binding::Other(n));
                }
            }
            Kind::NettypeDecl => ty::nettype(self, sc, n)?,
            Kind::VarDecl => self.var_decl(sc, n)?,
            Kind::NetDecl => self.net_decl(sc, n)?,
            Kind::InterconnectDecl => self.net_decl(sc, n)?,
            Kind::PortDecl => self.port_decl(sc, n)?,
            Kind::InterfacePortDecl => self.push_item(sc, Item::Opaque(n)),
            Kind::PackageImport => self.import(sc, n)?,
            Kind::PackageExport => self.push_item(sc, Item::Opaque(n)),
            Kind::GenvarDecl => {
                for &k in self.kids(n) {
                    let s = self.sym_of(k);
                    let o = self.new_object(Object {
                        name: s,
                        kind: ObjKind::Genvar,
                        ty: Types::INTEGER,
                        scope: sc,
                        node: k,
                        lifetime: Lifetime::Static,
                        dir: None,
                        init: None,
                        value: None,
                        qualifiers: vec![],
                    });
                    self.declare(sc, s, Binding::Object(o), k)?;
                }
            }
            Kind::Function | Kind::Task => {
                let s = self.sub_of(sc, n);
                if let Some(s) = s {
                    stmt::sub_signature(self, s)?;
                    self.push_item(sc, Item::Subroutine(s));
                    self.defer(Work::Body(s));
                }
            }
            Kind::Prototype | Kind::DpiImport => stmt::prototype(self, sc, n)?,
            Kind::DpiExport => checks::dpi_export(self, sc, n)?,
            Kind::Class | Kind::InterfaceClass => ty::class_body(self, sc, n)?,
            Kind::ContinuousAssign => self.defer(Work::Assign(sc, n)),
            Kind::NetAlias => self.defer(Work::Alias(sc, n)),
            Kind::Process => {
                checks::process_place(self, sc, n)?;
                self.defer(Work::Process(sc, n));
            }
            Kind::Instantiation => self.instantiation(sc, n)?,
            Kind::GateInstantiation => {
                let kw = self.kids(n)[0];
                let k = self.sym_of(kw);
                checks::gate_shape(self, n)?;
                self.defer(Work::Gate(sc, n, k));
            }
            Kind::GenerateRegion => {
                for &k in self.kids(n) {
                    self.item(sc, k)?;
                }
            }
            Kind::GenerateFor => {
                self.count_construct(sc);
                self.generate_for(sc, n)?
            }
            Kind::GenerateIf => {
                self.count_construct(sc);
                self.generate_if(sc, n)?
            }
            Kind::GenerateCase => {
                self.count_construct(sc);
                self.generate_case(sc, n)?
            }
            Kind::GenerateBlock => {
                self.count_construct(sc);
                self.generate_block(sc, n, None, None)?
            }
            Kind::ModportDecl => {
                for &mi in self.kids(n) {
                    if self.kind(mi) == Kind::ModportItem {
                        let name = self.name_child(mi).unwrap();
                        let s = self.sym_of(name);
                        self.declare(sc, s, Binding::Other(mi), name)?;
                    }
                }
                self.push_item(sc, Item::Opaque(n));
            }
            Kind::Clocking => {
                checks::clocking(self, sc, n)?;
                if let Some(name) = self.name_child(n) {
                    let s = self.sym_of(name);
                    self.declare(sc, s, Binding::Other(n), name)?;
                }
                self.push_item(sc, Item::Opaque(n));
            }
            Kind::Covergroup | Kind::PropertyDecl | Kind::SequenceDecl | Kind::LetDecl | Kind::Checker => {
                if let Some(name) = self.name_child(n) {
                    let s = self.sym_of(name);
                    self.declare(sc, s, Binding::Other(n), name)?;
                }
                self.push_item(sc, Item::Opaque(n));
            }
            Kind::Constraint => {
                if let Some(name) = self.name_child(n) {
                    let s = self.sym_of(name);
                    if !self.scope(sc).names.contains_key(&s) {
                        self.scope_mut(sc).names.insert(s, Binding::Other(n));
                    }
                }
                self.push_item(sc, Item::Opaque(n));
            }
            Kind::Timeunits => checks::timeunits(self, sc, n)?,
            Kind::ElaborationTask => self.defer(Work::ElabTask(sc, n)),
            Kind::Specify => {
                checks::specify(self, sc, n)?;
                self.push_item(sc, Item::Opaque(n));
            }
            Kind::Defparam => {
                checks::defparam(self, sc, n)?;
                self.push_item(sc, Item::Opaque(n));
            }
            Kind::Labeled => {
                // A labelled concurrent assertion or other item.
                for &k in self.kids(n) {
                    if self.kind(k) != Kind::Name {
                        self.item(sc, k)?;
                    }
                }
            }
            Kind::Module => {
                // A nested module or program that nothing instantiates is
                // instantiated once, by its own name (23.4, 24.3).
                let name = self.name_child(n).unwrap();
                let s = self.sym_of(name);
                let used = self.instantiated_in(self.scope(sc).node, s);
                if !used && let Some(&def) = self.defs.get(&s) && def.node == n {
                    let id = self.instantiate(Some(sc), sc, s, def, &[], n, None)?;
                    self.declare(sc, s, Binding::Instance(id), name)?;
                    self.push_item(sc, Item::Instance(id));
                }
            }
            Kind::AnonymousProgram => {
                // Its items belong to the enclosing scope (24.6).
                let items: Vec<NodeId> = self.kids(n).to_vec();
                self.declare_pass(sc, &items)?;
                for k in items {
                    self.item(sc, k)?;
                }
            }
            Kind::Udp | Kind::Config | Kind::Package | Kind::ExternModule => {
                self.push_item(sc, Item::Opaque(n));
            }
            _ => self.push_item(sc, Item::Opaque(n)),
        }
        Ok(())
    }

    /// The subroutine declared for a `Function` or `Task` node in pass 1.
    pub fn sub_of(&self, _sc: ScopeId, n: NodeId) -> Option<SubId> {
        self.d.subroutines.iter().position(|s| s.node == n).map(|i| SubId(i as u32))
    }

    // ---- parameters ----

    /// `parameter` and `localparam` declarations. `overrides` come from
    /// the instance, by name.
    pub fn param_decl(&mut self, sc: ScopeId, n: NodeId, _in_ports: bool) -> R<'a> {
        let kids = self.kids(n);
        let kw = self.text(kids[0]);
        let local = kw == "localparam" || self.local_by_context(sc, n);
        let ty_node = kids.get(1).copied().filter(|&k| self.kind(k) != Kind::ParamAssign);
        for &a in kids {
            if self.kind(a) != Kind::ParamAssign {
                continue;
            }
            self.param_assign(sc, a, ty_node, local)?;
        }
        Ok(())
    }

    /// A `parameter` in a body whose design element has a parameter port
    /// list, or in a generate block, or in a class with one, is local
    /// (6.20.1, 6.20.4).
    fn local_by_context(&mut self, sc: ScopeId, n: NodeId) -> bool {
        let kind = self.scope(sc).kind;
        let node = self.scope(sc).node;
        let def = if kind == ScopeKind::Spec { self.header_of(node) } else { node };
        let s = self.scope(sc);
        match s.kind {
            ScopeKind::Generate => true,
            ScopeKind::Spec | ScopeKind::Class => {
                let has_ports = self.child(def, Kind::ParamPorts).is_some();
                // In the port list itself it is not local.
                let in_ports = self.child(def, Kind::ParamPorts).is_some_and(|pp| {
                    let p = self.ast.node(pp);
                    let me = self.ast.node(n);
                    me.first >= p.first && me.end <= p.end
                });
                has_ports && !in_ports
            }
            _ => false,
        }
    }

    pub fn param_assign(&mut self, sc: ScopeId, a: NodeId, ty_node: Option<NodeId>, local: bool) -> R<'a> {
        let kids = self.kids(a);
        let name = kids[0];
        let s = self.sym_of(name);
        let value_node = kids.iter().copied().skip(1).find(|&k| !matches!(self.kind(k), Kind::RangeDim | Kind::SizeDim | Kind::UnsizedDim | Kind::QueueDim | Kind::AssocDim));
        let dims: Vec<NodeId> = kids[1..].iter().copied().filter(|&k| Some(k) != value_node).collect();
        // An override from the instance takes the place of the default.
        let over = if local { None } else { self.take_override(sc, s) };
        let declared = match ty_node {
            Some(t) if !(self.kind(t) == Kind::ImplicitType && self.kids(t).is_empty()) => {
                let base = ty::data_type(self, sc, t)?;
                Some(ty::unpacked(self, sc, base, &dims)?)
            }
            _ => None,
        };
        let v = match over {
            Some(Over::Value(v)) => v,
            Some(Over::Node(n2, psc)) => eval::const_expr(self, psc, n2)?,
            Some(Over::Type(_)) => return Err(self.err(name, format!("parameter `{}` is given a type", self.name(s)))),
            None => match value_node {
                Some(v) => eval::const_expr(self, sc, v)?,
                None => {
                    if self.scope(sc).kind != ScopeKind::Spec && self.scope(sc).kind != ScopeKind::Class {
                        return Err(self.err(name, format!("parameter `{}` has no value", self.name(s))));
                    }
                    Const::Bits(Bits::x(32))
                }
            },
        };
        let (ty, v) = match declared {
            Some(t) => {
                let ty_signing_only = ty_node.is_some_and(|t| self.kind(t) == Kind::ImplicitType);
                if ty_signing_only && ty::implicit_has_no_range(self, ty_node.unwrap()) {
                    // `parameter signed p = ...`: the value's width, signed.
                    let vt = eval::type_of_const(self, &v);
                    let w = self.d.types.width(vt).max(1);
                    let fs = self.d.types.is_four_state(vt);
                    let t2 = self.d.types.vector(w, true, fs);
                    (t2, eval::convert_const(self, &v, t2))
                } else {
                    (t, eval::convert_const(self, &v, t))
                }
            }
            None => {
                let t = eval::type_of_const(self, &v);
                (t, v)
            }
        };
        let c = self.new_const(v);
        let o = self.new_object(Object {
            name: s,
            kind: ObjKind::Param { local },
            ty,
            scope: sc,
            node: a,
            lifetime: Lifetime::Static,
            dir: None,
            init: None,
            value: Some(c),
            qualifiers: vec![],
        });
        self.declare(sc, s, Binding::Object(o), name)?;
        self.push_item(sc, Item::Object(o));
        if self.scope(sc).kind == ScopeKind::Spec {
            let spec = self.spec_of_scope(sc);
            if let Some(sp) = spec {
                self.d.specs[sp.0 as usize].params.push((s, c));
            }
        }
        Ok(())
    }

    pub fn type_param_decl(&mut self, sc: ScopeId, n: NodeId, _x: Option<()>) -> R<'a> {
        let local = self.text(self.kids(n)[0]) == "localparam" || self.local_by_context(sc, n);
        for &a in self.kids(n) {
            if self.kind(a) != Kind::TypeAssign {
                continue;
            }
            let kids = self.kids(a);
            let name = kids[0];
            let s = self.sym_of(name);
            let over = if local { None } else { self.take_override(sc, s) };
            let t = match over {
                Some(Over::Type(t)) => t,
                Some(Over::Node(n2, psc)) => ty::data_type(self, psc, n2)?,
                Some(Over::Value(_)) => {
                    return Err(self.err(name, format!("type parameter `{}` is given a value", self.name(s))));
                }
                None => match kids.get(1) {
                    Some(&t) => ty::data_type(self, sc, t)?,
                    None => Types::UNKNOWN,
                },
            };
            self.declare(sc, s, Binding::Type(t), name)?;
            self.push_item(sc, Item::Typedef(s, t));
            if let Some(sp) = self.spec_of_scope(sc) {
                let c = self.new_const(Const::Type(t));
                self.d.specs[sp.0 as usize].params.push((s, c));
            }
        }
        Ok(())
    }

    pub fn spec_of_scope(&self, sc: ScopeId) -> Option<SpecId> {
        self.d.specs.iter().position(|s| s.scope == sc).map(|i| SpecId(i as u32))
    }

    // ---- objects ----

    fn var_decl(&mut self, sc: ScopeId, n: NodeId) -> R<'a> {
        let kids = self.kids(n);
        let quals: Vec<Sym> = match self.child(n, Kind::Qualifiers) {
            Some(q) => {
                let ks: Vec<NodeId> = self.kids(q).to_vec();
                ks.into_iter().map(|k| self.sym_of(k)).collect()
            }
            None => vec![],
        };
        let ty_node = *kids.iter().find(|&&k| self.kind(k) != Kind::Qualifiers && self.kind(k) != Kind::Attributes).unwrap();
        // `M u;` with `M` a module: an instance without parentheses.
        if self.kind(ty_node) == Kind::NamedType
            && let Some(nm) = self.kids(ty_node).first().copied()
            && self.kind(nm) == Kind::Name
        {
            let s = self.sym_of(nm);
            if lookup::lookup(self, sc, s).is_none()
                && let Some(def) = self.defs.get(&s)
                && matches!(def.kind, DefKind2::Design(_) | DefKind2::Udp)
            {
                return Err(self.err(
                    nm,
                    format!("`{}` is a module, so instantiating it needs a port list in parentheses (23.3.2)", self.name(s)),
                ));
            }
        }
        // `T n;` with `T` a nettype declares a net (6.6.7).
        if self.kind(ty_node) == Kind::NamedType
            && let Some(&nm) = self.kids(ty_node).first()
            && self.kind(nm) == Kind::Name
            && { let ns = self.sym_of(nm); self.nettypes.contains(&ns) }
        {
            return self.net_decl(sc, n);
        }
        let base = ty::data_type(self, sc, ty_node)?;
        let is_const = quals.iter().any(|&q| self.name(q) == "const");
        let lifetime = self.lifetime_of(sc, &quals);
        for dcl in kids.iter().copied().filter(|&k| self.kind(k) == Kind::Declarator).collect::<Vec<_>>() {
            let dk = self.kids(dcl);
            let name = dk[0];
            let s = self.sym_of(name);
            let dims: Vec<NodeId> = dk[1..].iter().copied().filter(|&k| ty::is_dim(self.kind(k))).collect();
            let init = dk[1..].iter().copied().find(|&k| !ty::is_dim(self.kind(k)));
            let t = ty::unpacked(self, sc, base, &dims)?;
            checks::var_type(self, sc, name, t)?;
            let o = self.new_object(Object {
                name: s,
                kind: ObjKind::Variable,
                ty: t,
                scope: sc,
                node: dcl,
                lifetime,
                dir: None,
                init: None,
                value: None,
                qualifiers: quals.clone(),
            });
            // A non-ANSI port declared again as a variable (23.2.2.1).
            if let Some(Binding::Object(p)) = self.scope(sc).names.get(&s).copied()
                && self.obj(p).dir.is_some()
                && self.obj(p).node != dcl
            {
                self.complete_port(p, t, ObjKind::Variable, name)?;
                continue;
            }
            self.declare(sc, s, Binding::Object(o), name)?;
            self.push_item(sc, Item::Object(o));
            if let Some(i) = init {
                let e = expr::rvalue(self, sc, i, t)?;
                self.obj_mut(o).init = Some(e);
            } else if is_const && self.scope(sc).kind != ScopeKind::Class {
                // A `const` variable needs a value, except a class's
                // (8.19).
            }
        }
        Ok(())
    }

    pub fn lifetime_of(&self, sc: ScopeId, quals: &[Sym]) -> Lifetime {
        if quals.iter().any(|&q| self.name(q) == "automatic") {
            return Lifetime::Automatic;
        }
        if quals.iter().any(|&q| self.name(q) == "static") {
            return Lifetime::Static;
        }
        // Inside an automatic subroutine, or a class method, variables
        // are automatic (6.21).
        let mut s = Some(sc);
        while let Some(id) = s {
            let scope = self.scope(id);
            match scope.kind {
                ScopeKind::Subroutine => {
                    let sub = self.d.subroutines.iter().find(|x| x.scope == id);
                    return sub.map_or(Lifetime::Static, |x| x.lifetime);
                }
                ScopeKind::Block => s = scope.parent,
                _ => return Lifetime::Static,
            }
        }
        Lifetime::Static
    }

    fn net_decl(&mut self, sc: ScopeId, n: NodeId) -> R<'a> {
        let kids = self.kids(n);
        let interconnect = self.kind(n) == Kind::InterconnectDecl;
        // `T n;` with `T` a nettype comes here as a variable declaration.
        let as_var = self.kind(n) == Kind::VarDecl;
        let first = if as_var { self.kids(kids[0])[0] } else { kids[0] };
        let kind_sym = if interconnect { self.sym("interconnect") } else { self.sym_of(first) };
        let user_nettype = !interconnect && self.kind(first) == Kind::Name;
        let ty_node = if as_var { None } else { kids.iter().copied().find(|&k| ty::is_type(self.kind(k))) };
        let base = match ty_node {
            Some(t) => ty::data_type(self, sc, t)?,
            None => Types::LOGIC,
        };
        let base = if user_nettype {
            match lookup::lookup(self, sc, kind_sym) {
                Some(Binding::Type(t)) => t,
                _ => return Err(self.err(kids[0], format!("`{}` is not a net type", self.name(kind_sym)))),
            }
        } else if matches!(self.d.types.get(base), Type::Unknown) {
            base
        } else {
            // A net's data type is four-state logic unless given (6.7.1).
            base
        };
        checks::net_type(self, sc, n, base, user_nettype)?;
        for dcl in kids.iter().copied().filter(|&k| self.kind(k) == Kind::Declarator).collect::<Vec<_>>() {
            let dk = self.kids(dcl);
            let name = dk[0];
            let s = self.sym_of(name);
            let dims: Vec<NodeId> = dk[1..].iter().copied().filter(|&k| ty::is_dim(self.kind(k))).collect();
            let init = dk[1..].iter().copied().find(|&k| !ty::is_dim(self.kind(k)));
            let t = ty::unpacked(self, sc, base, &dims)?;
            if let Some(Binding::Object(p)) = self.scope(sc).names.get(&s).copied()
                && self.obj(p).dir.is_some()
                && self.obj(p).node != dcl
            {
                self.complete_port(p, t, ObjKind::Net(kind_sym), name)?;
                continue;
            }
            let o = self.new_object(Object {
                name: s,
                kind: ObjKind::Net(kind_sym),
                ty: t,
                scope: sc,
                node: dcl,
                lifetime: Lifetime::Static,
                dir: None,
                init: None,
                value: None,
                qualifiers: vec![],
            });
            self.declare(sc, s, Binding::Object(o), name)?;
            self.push_item(sc, Item::Object(o));
            if let Some(i) = init {
                self.defer(Work::NetInit(o, i));
            }
        }
        Ok(())
    }

    // ---- ports ----

    /// A non-ANSI port's declaration (23.2.2.1): `input [7:0] a;`.
    fn port_decl(&mut self, sc: ScopeId, n: NodeId) -> R<'a> {
        let kids = self.kids(n);
        let dir = match self.text(kids.iter().copied().find(|&k| self.kind(k) == Kind::Keyword).unwrap()) {
            "input" => Dir::Input,
            "output" => Dir::Output,
            "inout" => Dir::Inout,
            _ => Dir::Ref,
        };
        let kws = self.keywords_of(n);
        let is_var = kws.iter().any(|&k| k == "var" || k == "reg");
        let net_kw = kws.iter().copied().find(|k| ty::NET_KEYWORDS.contains(k));
        let ty_node = kids.iter().copied().find(|&k| ty::is_type(self.kind(k)));
        let base = match ty_node {
            Some(t) => ty::data_type(self, sc, t)?,
            None => Types::LOGIC,
        };
        let explicit_type = ty_node.is_some_and(|t| self.kind(t) != Kind::ImplicitType);
        for dcl in kids.iter().copied().filter(|&k| self.kind(k) == Kind::Declarator).collect::<Vec<_>>() {
            let dk = self.kids(dcl);
            let name = dk[0];
            let s = self.sym_of(name);
            let dims: Vec<NodeId> = dk[1..].iter().copied().filter(|&k| ty::is_dim(self.kind(k))).collect();
            let t = ty::unpacked(self, sc, base, &dims)?;
            let kind = if is_var || (explicit_type && net_kw.is_none() && dir != Dir::Inout && dir != Dir::Input) {
                ObjKind::Variable
            } else {
                ObjKind::Net(self.sym_owned(net_kw.unwrap_or("wire").to_string()))
            };
            let spec = self.spec_of_scope(sc);
            let in_list = spec.is_some_and(|sp| self.d.specs[sp.0 as usize].ports.iter().any(|p| p.name == s))
                || self.named_in_port_list(sc, s);
            if !in_list && spec.is_some() {
                return Err(self.err(name, format!("`{}` is declared as a port but is not in the port list", self.name(s))));
            }
            match self.scope(sc).names.get(&s).copied() {
                Some(Binding::Object(o)) if self.obj(o).dir.is_none() => {
                    // Declared first as a net or variable.
                    self.obj_mut(o).dir = Some(dir);
                    if self.obj(o).ty == Types::LOGIC && t != Types::LOGIC {
                        self.obj_mut(o).ty = t;
                    }
                    self.attach_port(sc, s, o, dir);
                }
                Some(_) => {
                    return Err(self.err(name, format!("port `{}` is declared twice", self.name(s))));
                }
                None => {
                    let o = self.new_object(Object {
                        name: s,
                        kind,
                        ty: t,
                        scope: sc,
                        node: dcl,
                        lifetime: Lifetime::Static,
                        dir: Some(dir),
                        init: None,
                        value: None,
                        qualifiers: vec![],
                    });
                    self.declare(sc, s, Binding::Object(o), name)?;
                    self.push_item(sc, Item::Object(o));
                    self.attach_port(sc, s, o, dir);
                    if let Some(i) = dk[1..].iter().copied().find(|&k| !ty::is_dim(self.kind(k))) {
                        let e = expr::rvalue(self, sc, i, t)?;
                        self.obj_mut(o).init = Some(e);
                    }
                }
            }
        }
        Ok(())
    }

    /// Whether `s` appears inside a port expression of the header, as in
    /// `module m(.a(bus[3:0]))` (23.2.2.1).
    fn named_in_port_list(&mut self, sc: ScopeId, s: Sym) -> bool {
        let node = self.scope(sc).node;
        let h = self.header_of(node);
        let Some(ports) = self.child(h, Kind::Ports) else { return false };
        let text = self.name(s);
        self.ast.tokens_of(ports).iter().any(|t| t.0 == text)
    }

    /// A port declared with a direction, then again as a net or variable.
    fn complete_port(&mut self, p: ObjId, t: TypeId, kind: ObjKind, at: NodeId) -> R<'a> {
        let old = self.obj(p).ty;
        let explicit_old = !(old == Types::LOGIC);
        if explicit_old && old != t && self.d.types.width(old) != self.d.types.width(t) {
            return Err(self.err(at, "the port's declarations give it different ranges (23.2.2.1)"));
        }
        let o = self.obj_mut(p);
        o.kind = kind;
        o.ty = t;
        Ok(())
    }

    fn attach_port(&mut self, sc: ScopeId, s: Sym, o: ObjId, dir: Dir) {
        if let Some(sp) = self.spec_of_scope(sc) {
            for p in self.d.specs[sp.0 as usize].ports.iter_mut() {
                if p.name == s {
                    p.object = Some(o);
                    p.dir = dir;
                }
            }
        }
    }

    fn import(&mut self, sc: ScopeId, n: NodeId) -> R<'a> {
        for &it in self.kids(n) {
            if self.kind(it) != Kind::ImportItem {
                continue;
            }
            let ik = self.kids(it);
            let pname = self.sym_of(ik[0]);
            let Some(&pkg) = self.packages.get(&pname) else {
                return Err(self.err(ik[0], format!("there is no package `{}`", self.name(pname))));
            };
            if self.kind(ik[1]) == Kind::Op {
                if !self.scope(sc).wildcards.contains(&pkg) {
                    let at = self.ast.node(it).first;
                    self.scope_mut(sc).wildcards.push(pkg);
                    self.scope_mut(sc).wildcard_at.push(at);
                }
                self.push_item(sc, Item::Import(pkg, None));
            } else {
                let s = self.sym_of(ik[1]);
                let Some(b) = lookup::in_package(self, pkg, s) else {
                    return Err(self.err(ik[1], format!("package `{}` has no `{}`", self.name(pname), self.name(s))));
                };
                if let Some(old) = self.scope(sc).names.get(&s).copied()
                    && old != b
                {
                    let from_import = self.scope(sc).items.iter().any(|i| matches!(i, Item::Import(_, Some(x)) if *x == s));
                    return Err(self.err(
                        ik[1],
                        if from_import {
                            format!("`{}` is already imported from another package (26.3)", self.name(s))
                        } else {
                            format!("`{}` is already declared in this scope, so it cannot be imported (26.3)", self.name(s))
                        },
                    ));
                }
                self.declare(sc, s, b, ik[1])?;
                self.push_item(sc, Item::Import(pkg, Some(s)));
            }
        }
        Ok(())
    }

    // ---- instances ----

    fn instantiation(&mut self, sc: ScopeId, n: NodeId) -> R<'a> {
        let kids = self.kids(n);
        let name = kids[0];
        let s = self.sym_of(name);
        let Some(def) = self.defs.get(&s).copied() else {
            // A class or a type used as `T x(...)` is not an instance.
            return Err(self.err(name, format!("there is no module, interface, program or primitive `{}`", self.name(s))));
        };
        match def.kind {
            DefKind2::Udp => {
                self.defer(Work::Gate(sc, n, s));
                return Ok(());
            }
            DefKind2::Checker | DefKind2::Config => {
                for inst in kids.iter().copied().filter(|&k| self.kind(k) == Kind::Instance).collect::<Vec<_>>() {
                    if let Some(iname) = self.name_child(inst) {
                        let is = self.sym_of(iname);
                        self.declare(sc, is, Binding::Other(inst), iname)?;
                    }
                }
                self.push_item(sc, Item::Opaque(n));
                return Ok(());
            }
            DefKind2::Design(DefKind::Program) if self.in_program_or_interface(sc) == Some(DefKind::Program) => {
                return Err(self.err(name, "a program cannot instantiate a program (24.3)"));
            }
            DefKind2::Design(DefKind::Module) => {
                if let Some(k) = self.in_program_or_interface(sc) {
                    let what = if k == DefKind::Program { "a program" } else { "an interface" };
                    return Err(self.err(name, format!("{what} cannot instantiate a module (24.3, 25.3)")));
                }
            }
            DefKind2::Design(_) => {}
        }
        let params = kids.iter().copied().find(|&k| self.kind(k) == Kind::ParamValues);
        if let Some(d) = kids.iter().copied().find(|&k| self.kind(k) == Kind::Delay) {
            return Err(self.err(d, format!("`{}` is a module; only gates and primitives take a delay", self.name(s))));
        }
        let overrides = match params {
            Some(p) => self.param_overrides(sc, p)?,
            None => vec![],
        };
        for inst in kids.iter().copied().filter(|&k| self.kind(k) == Kind::Instance).collect::<Vec<_>>() {
            let ik = self.kids(inst);
            let Some(iname) = ik.iter().copied().find(|&k| self.kind(k) == Kind::Name) else {
                return Err(self.err(inst, "a module instance needs a name (23.3.2)"));
            };
            let is = self.sym_of(iname);
            let dims: Vec<NodeId> = ik.iter().copied().filter(|&k| ty::is_dim(self.kind(k))).collect();
            if dims.len() > 1 {
                // Multi-dimensional arrays of instances are allowed (23.3.3.5).
            }
            let range = match dims.first() {
                Some(&dim) => Some(ty::const_range(self, sc, dim)?),
                None => None,
            };
            match range {
                None => {
                    let id = self.instantiate(Some(sc), sc, is, def, &overrides, inst, None)?;
                    self.declare(sc, is, Binding::Instance(id), iname)?;
                    self.push_item(sc, Item::Instance(id));
                    self.defer(Work::Connect(sc, id, inst));
                }
                Some((l, r)) => {
                    let count = (l - r).unsigned_abs() as u32 + 1;
                    let step = if l >= r { -1 } else { 1 };
                    let mut first = None;
                    let mut i = l;
                    for _ in 0..count {
                        let id = self.instantiate(Some(sc), sc, is, def, &overrides, inst, Some(i))?;
                        first.get_or_insert(id);
                        self.push_item(sc, Item::Instance(id));
                        self.defer(Work::Connect(sc, id, inst));
                        i += step;
                    }
                    self.declare(sc, is, Binding::Instances(first.unwrap(), count), iname)?;
                }
            }
        }
        Ok(())
    }

    /// Whether `sc` is inside a program or interface body.
    pub fn in_program_or_interface(&self, sc: ScopeId) -> Option<DefKind> {
        let mut s = Some(sc);
        while let Some(id) = s {
            if self.scope(id).kind == ScopeKind::Spec {
                return self.spec_of_scope(id).map(|sp| self.d.specs[sp.0 as usize].kind).filter(|&k| k != DefKind::Module);
            }
            s = self.scope(id).parent;
        }
        None
    }

    /// `#(...)` on an instance: the overrides, as names or positions, each
    /// evaluated in the parent.
    fn param_overrides(&mut self, sc: ScopeId, p: NodeId) -> R<'a, Vec<(Option<Sym>, Over)>> {
        let mut out = Vec::new();
        for &k in self.kids(p) {
            match self.kind(k) {
                Kind::NamedArg => {
                    let nk = self.kids(k);
                    let s = self.sym_of(nk[0]);
                    let Some(&v) = nk.get(1) else { continue };
                    out.push((Some(s), self.override_value(sc, v)?));
                }
                _ => out.push((None, self.override_value(sc, k)?)),
            }
        }
        Ok(out)
    }

    fn override_value(&mut self, sc: ScopeId, v: NodeId) -> R<'a, Over> {
        if ty::is_type(self.kind(v)) || self.kind(v) == Kind::TypeExpr {
            let t = if self.kind(v) == Kind::TypeExpr { self.kids(v)[0] } else { v };
            return Ok(Over::Type(ty::data_type(self, sc, t)?));
        }
        // A name that is a type is a type override.
        if self.kind(v) == Kind::Name {
            let s = self.sym_of(v);
            if let Some(Binding::Type(t)) = lookup::lookup(self, sc, s) {
                return Ok(Over::Type(t));
            }
        }
        if let Some(t) = ty::maybe_type(self, sc, v)? {
            return Ok(Over::Type(t));
        }
        Ok(Over::Value(eval::const_expr(self, sc, v)?))
    }

    /// Makes an instance of `def` with these overrides, elaborating its
    /// specialisation if this is the first instance with these values.
    #[allow(clippy::too_many_arguments)]
    pub fn instantiate(
        &mut self,
        parent: Option<ScopeId>,
        sc: ScopeId,
        name: Sym,
        def: Def,
        overrides: &[(Option<Sym>, Over)],
        node: NodeId,
        index: Option<i64>,
    ) -> R<'a, InstId> {
        let _ = parent;
        let id = InstId(self.d.instances.len() as u32);
        self.d.instances.push(Instance {
            name,
            spec: SpecId(u32::MAX),
            scope: sc,
            node,
            connections: vec![],
            index,
        });
        if self.inst_path.len() > 64 {
            return Err(self.err(node, "the hierarchy is more than 64 instances deep; is it recursive?"));
        }
        let key = (def.node, format!("{overrides:?}"));
        if self.nested.contains(&def.node) {
            self.nested_parent = Some(sc);
        }
        let spec = match self.spec_cache.get(&key).filter(|_| !self.nested.contains(&def.node)) {
            Some(&s) => s,
            None => {
                self.inst_path.push(id);
                let saved_upward = std::mem::replace(&mut self.upward, false);
                let r = self.elaborate_spec(def, overrides);
                let upward = std::mem::replace(&mut self.upward, saved_upward);
                self.inst_path.pop();
                let s = r?;
                if !upward {
                    self.spec_cache.insert(key, s);
                }
                s
            }
        };
        self.d.instances[id.0 as usize].spec = spec;
        self.d.specs[spec.0 as usize].instances += 1;
        Ok(id)
    }

    fn elaborate_spec(&mut self, def: Def, overrides: &[(Option<Sym>, Over)]) -> R<'a, SpecId> {
        let n = def.node;
        let DefKind2::Design(kind) = def.kind else { unreachable!() };
        let name = self.name_child(n).unwrap();
        let s = self.sym_of(name);
        // A nested definition sees the names of the one around it (23.4).
        let outer = if self.nested.contains(&n) { self.nested_parent.take() } else { None };
        let unit = self.d.unit;
        let sc = self.new_scope(ScopeKind::Spec, Some(s), Some(outer.unwrap_or(unit)), n);
        let spec = SpecId(self.d.specs.len() as u32);
        self.d.specs.push(Spec { kind, name: s, def: n, params: vec![], scope: sc, ports: vec![], instances: 0 });
        // Overrides by position apply to the parameters of the port list,
        // or, without one, to the body's parameters, in order (23.10.2).
        let order = self.overridable_params(n);
        let mut by_name: Vec<(Sym, Over)> = Vec::new();
        let mut pos = 0;
        for (nm, v) in overrides {
            match nm {
                Some(s2) => {
                    if !order.contains(s2) {
                        let all = self.all_params(n);
                        let at = def.node;
                        if all.contains(s2) {
                            return Err(self.err(
                                at,
                                format!("`{}` is a local parameter of `{}`, so it cannot be overridden", self.name(*s2), self.name(s)),
                            ));
                        }
                        return Err(self.err(at, format!("`{}` has no parameter `{}`", self.name(s), self.name(*s2))));
                    }
                    by_name.push((*s2, v.clone()));
                }
                None => {
                    let Some(&p) = order.get(pos) else {
                        return Err(self.err(def.node, format!("too many parameter values for `{}`", self.name(s))));
                    };
                    by_name.push((p, v.clone()));
                    pos += 1;
                }
            }
        }
        self.pending_overrides.push((sc, by_name));
        let r = self.spec_body(sc, n, kind);
        self.pending_overrides.pop();
        r?;
        Ok(spec)
    }

    /// The node whose header (parameters and ports) a definition uses: an
    /// `extern module` declaration of the same name gives it, for a
    /// definition written `module m(.*);` or with no header (23.4).
    fn header_of(&mut self, n: NodeId) -> NodeId {
        let Some(name) = self.name_child(n) else { return n };
        let s = self.sym_of(name);
        let Some(&ext) = self.externs.get(&s) else { return n };
        let wildcard = self.child(n, Kind::Ports).is_none_or(|p| self.ast.tokens_of(p).iter().any(|t| t.0 == ".*"));
        if wildcard && self.child(n, Kind::ParamPorts).is_none() { ext } else { n }
    }

    fn spec_body(&mut self, sc: ScopeId, n: NodeId, kind: DefKind) -> R<'a> {
        let kids: Vec<NodeId> = self.kids(n).to_vec();
        let h = self.header_of(n);
        // Header imports, then parameters, then ports.
        for &k in &kids {
            if self.kind(k) == Kind::PackageImport {
                self.import(sc, k)?;
            }
        }
        if let Some(pp) = self.child(h, Kind::ParamPorts) {
            let mut ty_node = None;
            let mut local = false;
            for &p in self.kids(pp) {
                match self.kind(p) {
                    Kind::ParamDecl => {
                        let pk = self.kids(p);
                        local = self.text(pk[0]) == "localparam";
                        ty_node = pk.get(1).copied().filter(|&k| self.kind(k) != Kind::ParamAssign);
                        for &a in pk {
                            if self.kind(a) == Kind::ParamAssign {
                                self.param_assign(sc, a, ty_node, local)?;
                            }
                        }
                    }
                    Kind::TypeParamDecl => self.type_param_decl(sc, p, None)?,
                    Kind::ParamAssign => self.param_assign(sc, p, ty_node, local)?,
                    _ => {}
                }
            }
        }
        if let Some(ports) = self.child(h, Kind::Ports) {
            self.ports(sc, ports)?;
        }
        let items: Vec<NodeId> = kids
            .iter()
            .copied()
            .filter(|&k| {
                !matches!(
                    self.kind(k),
                    Kind::Keyword | Kind::Name | Kind::PackageImport | Kind::ParamPorts | Kind::Ports | Kind::EndLabel
                )
            })
            .collect();
        self.declare_pass(sc, &items)?;
        for k in items {
            self.item(sc, k)?;
        }
        checks::spec_done(self, sc, kind)?;
        Ok(())
    }

    /// The parameters an instance may set, in order (23.10.2).
    fn overridable_params(&mut self, n: NodeId) -> Vec<Sym> {
        let n = self.header_of(n);
        let mut out = Vec::new();
        if let Some(pp) = self.child(n, Kind::ParamPorts) {
            let mut local = false;
            for &p in self.kids(pp) {
                match self.kind(p) {
                    Kind::ParamDecl | Kind::TypeParamDecl => {
                        local = self.text(self.kids(p)[0]) == "localparam";
                        for &a in self.kids(p) {
                            if matches!(self.kind(a), Kind::ParamAssign | Kind::TypeAssign) && !local {
                                let s = self.sym_of(self.kids(a)[0]);
                                out.push(s);
                            }
                        }
                    }
                    Kind::ParamAssign if !local => {
                        let s = self.sym_of(self.kids(p)[0]);
                        out.push(s);
                    }
                    _ => {}
                }
            }
            return out;
        }
        self.body_params(n, &mut out, false);
        out
    }

    fn all_params(&mut self, n: NodeId) -> Vec<Sym> {
        let mut out = Vec::new();
        self.body_params(n, &mut out, true);
        out
    }

    fn body_params(&mut self, n: NodeId, out: &mut Vec<Sym>, with_local: bool) {
        for &k in self.kids(n) {
            match self.kind(k) {
                Kind::ParamDecl | Kind::TypeParamDecl => {
                    let local = self.text(self.kids(k)[0]) == "localparam";
                    if local && !with_local {
                        continue;
                    }
                    for &a in self.kids(k) {
                        if matches!(self.kind(a), Kind::ParamAssign | Kind::TypeAssign) {
                            let s = self.sym_of(self.kids(a)[0]);
                            out.push(s);
                        }
                    }
                }
                Kind::ParamPorts | Kind::GenerateRegion if with_local => self.body_params(k, out, with_local),
                _ => {}
            }
        }
    }

    fn take_override(&mut self, sc: ScopeId, s: Sym) -> Option<Over> {
        let (osc, list) = self.pending_overrides.last_mut()?;
        if *osc != sc {
            return None;
        }
        let i = list.iter().position(|(n, _)| *n == s)?;
        Some(list.remove(i).1)
    }

    /// The ports of a design element's header: ANSI declarations, or the
    /// names of a non-ANSI list whose declarations come in the body.
    fn ports(&mut self, sc: ScopeId, n: NodeId) -> R<'a> {
        let spec = self.spec_of_scope(sc).unwrap();
        let mut dir = Dir::Inout;
        let mut prev_type: Option<NodeId> = None;
        let mut prev_kind: Option<&str> = None;
        let mut first = true;
        for &p in self.kids(n) {
            match self.kind(p) {
                Kind::Port => {
                    // Non-ANSI: a name, or `.name(expr)`.
                    let pk = self.kids(p);
                    let (pname, at) = match pk.first() {
                        Some(&k) if self.kind(k) == Kind::Name => (self.sym_of(k), k),
                        Some(&k) if self.kind(k) == Kind::NamedArg => (self.sym_of(self.kids(k)[0]), k),
                        Some(&k) => (self.sym_owned(format!("__port{}", self.d.specs[spec.0 as usize].ports.len())), k),
                        None => continue,
                    };
                    if self.d.specs[spec.0 as usize].ports.iter().any(|q| q.name == pname) {
                        return Err(self.err(at, format!("port `{}` is listed twice", self.name(pname))));
                    }
                    self.d.specs[spec.0 as usize].ports.push(Port {
                        name: pname,
                        dir: Dir::Inout,
                        object: None,
                        interface: None,
                        node: p,
                    });
                }
                Kind::AnsiPort => {
                    let pk = self.kids(p);
                    let kws: Vec<&str> = pk.iter().filter(|&&k| self.kind(k) == Kind::Keyword).map(|&k| self.text(k)).collect();
                    let explicit_dir = kws.iter().find_map(|&k| match k {
                        "input" => Some(Dir::Input),
                        "output" => Some(Dir::Output),
                        "inout" => Some(Dir::Inout),
                        "ref" => Some(Dir::Ref),
                        _ => None,
                    });
                    let net_kw = kws.iter().copied().find(|k| ty::NET_KEYWORDS.contains(k) || *k == "var" || *k == "interconnect");
                    if let Some(d) = explicit_dir {
                        dir = d;
                    } else if first {
                        dir = Dir::Inout;
                    }
                    let iface = pk.iter().copied().find(|&k| self.kind(k) == Kind::InterfacePortType);
                    let decl = pk.iter().copied().find(|&k| self.kind(k) == Kind::Declarator);
                    let Some(decl) = decl else {
                        // An explicit port, `.name(expr)`.
                        if let Some(na) = pk.iter().copied().find(|&k| self.kind(k) == Kind::NamedArg) {
                            let s = self.sym_of(self.kids(na)[0]);
                            self.d.specs[spec.0 as usize].ports.push(Port {
                                name: s,
                                dir,
                                object: None,
                                interface: None,
                                node: p,
                            });
                        }
                        continue;
                    };
                    let dk = self.kids(decl);
                    let pname = self.sym_of(dk[0]);
                    if let Some(ip) = iface {
                        let ik = self.kids(ip);
                        let iname = self.sym_of(ik[0]);
                        let mp = ik.get(1).map(|&m| self.sym_of(m));
                        self.declare(sc, pname, Binding::Other(p), dk[0])?;
                        self.d.specs[spec.0 as usize].ports.push(Port {
                            name: pname,
                            dir: Dir::Inout,
                            object: None,
                            interface: Some((iname, mp)),
                            node: p,
                        });
                        first = false;
                        continue;
                    }
                    let ty_node = pk.iter().copied().find(|&k| ty::is_type(self.kind(k)));
                    // A port with only a name takes the previous port's
                    // direction, kind and type (23.2.2.3).
                    let inherit = explicit_dir.is_none()
                        && net_kw.is_none()
                        && ty_node.is_none_or(|t| self.kind(t) == Kind::ImplicitType && self.kids(t).is_empty())
                        && !first;
                    let ty_used = if inherit { prev_type } else { ty_node };
                    // A named type that is an interface: an interface port.
                    if let Some(t) = ty_used
                        && self.kind(t) == Kind::NamedType
                        && let Some(&nm) = self.kids(t).first()
                        && self.kind(nm) == Kind::Name
                    {
                        let ts = self.sym_of(nm);
                        if lookup::lookup(self, sc, ts).is_none()
                            && let Some(def) = self.defs.get(&ts)
                            && def.kind == DefKind2::Design(DefKind::Interface)
                        {
                            self.declare(sc, pname, Binding::Other(p), dk[0])?;
                            self.d.specs[spec.0 as usize].ports.push(Port {
                                name: pname,
                                dir: Dir::Inout,
                                object: None,
                                interface: Some((ts, None)),
                                node: p,
                            });
                            first = false;
                            prev_type = ty_used;
                            continue;
                        }
                    }
                    let base = match ty_used {
                        Some(t) => ty::data_type(self, sc, t)?,
                        None => Types::LOGIC,
                    };
                    let dims: Vec<NodeId> = dk[1..].iter().copied().filter(|&k| ty::is_dim(self.kind(k))).collect();
                    let t = ty::unpacked(self, sc, base, &dims)?;
                    let kw = if inherit { prev_kind } else { net_kw };
                    let has_type = ty_used.is_some_and(|t| self.kind(t) != Kind::ImplicitType);
                    let kind = match kw {
                        Some("var") => ObjKind::Variable,
                        Some(k) => ObjKind::Net(self.sym_owned(k.to_string())),
                        None if dir == Dir::Output && has_type => ObjKind::Variable,
                        None if dir == Dir::Ref => ObjKind::Variable,
                        None => {
                            if has_type && !self.d.types.is_integral(t) {
                                ObjKind::Variable
                            } else {
                                ObjKind::Net(self.sym("wire"))
                            }
                        }
                    };
                    if dir == Dir::Ref && matches!(kind, ObjKind::Net(_)) {
                        return Err(self.err(dk[0], "a `ref` port must be a variable (23.2.2.3)"));
                    }
                    let o = self.new_object(Object {
                        name: pname,
                        kind,
                        ty: t,
                        scope: sc,
                        node: decl,
                        lifetime: Lifetime::Static,
                        dir: Some(dir),
                        init: None,
                        value: None,
                        qualifiers: vec![],
                    });
                    self.declare(sc, pname, Binding::Object(o), dk[0])?;
                    self.push_item(sc, Item::Object(o));
                    if let Some(i) = dk[1..].iter().copied().find(|&k| !ty::is_dim(self.kind(k))) {
                        let e = expr::rvalue(self, sc, i, t)?;
                        self.obj_mut(o).init = Some(e);
                    }
                    self.d.specs[spec.0 as usize].ports.push(Port {
                        name: pname,
                        dir,
                        object: Some(o),
                        interface: None,
                        node: p,
                    });
                    prev_type = ty_used;
                    prev_kind = kw;
                    first = false;
                }
                _ => {}
            }
        }
        Ok(())
    }

    /// Connects an instance's ports (23.3.2) once both sides exist.
    fn connect(&mut self, sc: ScopeId, inst: InstId, n: NodeId) -> R<'a> {
        let in_array = self.d.instances[inst.0 as usize].index.is_some();
        let interface_ports =
            self.d.specs[self.d.instances[inst.0 as usize].spec.0 as usize].ports.iter().any(|p| p.interface.is_some());
        self.connecting_array = in_array && !interface_ports;
        let r = self.connect_ports(sc, inst, n);
        self.connecting_array = false;
        r
    }

    fn connect_ports(&mut self, sc: ScopeId, inst: InstId, n: NodeId) -> R<'a> {
        let spec = self.d.instances[inst.0 as usize].spec;
        let ports = self.d.specs[spec.0 as usize].ports.clone();
        let conns: Vec<NodeId> =
            self.kids(n).iter().copied().filter(|&k| matches!(self.kind(k), Kind::Connection | Kind::NamedConnection)).collect();
        let mut out: Vec<Connection> = Vec::new();
        let named = conns.iter().any(|&c| self.kind(c) == Kind::NamedConnection);
        let mut wildcard = false;
        if named {
            for &c in &conns {
                if self.kind(c) == Kind::Connection {
                    return Err(self.err(c, "ordered and named port connections cannot be mixed (23.3.2)"));
                }
                let ck: Vec<NodeId> = self.kids(c).iter().copied().filter(|&k| self.kind(k) != Kind::Attributes).collect();
                if self.kind(ck[0]) == Kind::Op {
                    wildcard = true;
                    continue;
                }
                let pname = self.sym_of(ck[0]);
                let Some(port) = ports.iter().find(|p| p.name == pname) else {
                    return Err(self.err(
                        ck[0],
                        format!("`{}` has no port `{}`", self.name(self.d.specs[spec.0 as usize].name), self.name(pname)),
                    ));
                };
                if out.iter().any(|x| x.port == pname) {
                    return Err(self.err(ck[0], format!("port `{}` is connected twice", self.name(pname))));
                }
                // `.name` alone connects the same name in the parent.
                let parens = self.ast.tokens_of(c).iter().any(|t| t.0 == "(");
                let conn = match ck.get(1) {
                    Some(&e) if self.kind(e) == Kind::Empty => Connection { port: pname, expr: None, interface: None },
                    Some(&e) => self.port_expr(sc, port, e)?,
                    // `.name()`: left open.
                    None if parens => Connection { port: pname, expr: None, interface: None },
                    None => {
                        if lookup::lookup(self, sc, pname).is_none() {
                            return Err(self.err(
                                ck[0],
                                format!("`.{}` needs a `{}` in the instantiating scope (23.3.2.3)", self.name(pname), self.name(pname)),
                            ));
                        }
                        self.port_expr(sc, port, ck[0])?
                    }
                };
                out.push(conn);
            }
            if wildcard {
                for p in &ports {
                    if out.iter().any(|x| x.port == p.name) {
                        continue;
                    }
                    if lookup::lookup(self, sc, p.name).is_some() {
                        let c = self.port_expr_named(sc, p, n)?;
                        out.push(c);
                    } else if p.object.is_some_and(|o| self.obj(o).init.is_some()) {
                        // A port with a default value.
                    } else {
                        return Err(self.err(
                            n,
                            format!("`.*` cannot connect port `{}`: nothing of that name is visible (23.3.2.4)", self.name(p.name)),
                        ));
                    }
                }
            }
        } else {
            if conns.len() > ports.len() && !(ports.is_empty() && conns.len() == 1 && self.conn_is_empty(conns[0])) {
                return Err(self.err(
                    n,
                    format!(
                        "`{}` has {} ports but {} connections are given",
                        self.name(self.d.specs[spec.0 as usize].name),
                        ports.len(),
                        conns.len()
                    ),
                ));
            }
            for (i, &c) in conns.iter().enumerate() {
                let Some(port) = ports.get(i) else { continue };
                let e = self.kids(c).iter().copied().find(|&k| self.kind(k) != Kind::Attributes);
                match e {
                    Some(e) if self.kind(e) != Kind::Empty => {
                        let conn = self.port_expr(sc, port, e)?;
                        out.push(conn);
                    }
                    _ => out.push(Connection { port: port.name, expr: None, interface: None }),
                }
            }
        }
        for p in &ports {
            if p.interface.is_some() && !out.iter().any(|c| c.port == p.name && (c.interface.is_some() || c.expr.is_some())) {
                return Err(self.err(n, format!("interface port `{}` is not connected (25.3)", self.name(p.name))));
            }
        }
        self.d.instances[inst.0 as usize].connections = out;
        Ok(())
    }

    fn conn_is_empty(&self, c: NodeId) -> bool {
        self.kids(c).iter().all(|&k| matches!(self.kind(k), Kind::Empty | Kind::Attributes))
    }

    fn port_expr_named(&mut self, sc: ScopeId, port: &Port, at: NodeId) -> R<'a, Connection> {
        let b = lookup::lookup(self, sc, port.name);
        if let Some(Binding::Instance(i)) = b {
            return Ok(Connection { port: port.name, expr: None, interface: Some(i) });
        }
        if let Some(Binding::Other(_)) = b {
            return Ok(Connection { port: port.name, expr: None, interface: None });
        }
        let Some(Binding::Object(o)) = b else {
            return Err(self.err(at, format!("`{}` cannot be connected to a port", self.name(port.name))));
        };
        let ty = self.obj(o).ty;
        let e = self.new_expr(ExprKind::Object(o), ty, at, vec![]);
        let e = self.port_conv(port, e, at)?;
        Ok(Connection { port: port.name, expr: Some(e), interface: None })
    }

    fn port_expr(&mut self, sc: ScopeId, port: &Port, e: NodeId) -> R<'a, Connection> {
        if self.connecting_array {
            let x = expr::expr(self, sc, e)?;
            return Ok(Connection { port: port.name, expr: Some(x), interface: None });
        }
        if let Some((iname, _)) = port.interface {
            // An interface instance, or an interface port, passed on.
            let target = self.interface_target(sc, e);
            match target {
                Some(Binding::Instance(i)) => {
                    let spec = self.d.instances[i.0 as usize].spec;
                    let got = self.d.specs[spec.0 as usize].name;
                    if got != iname && self.name(iname) != "interface" {
                        return Err(self.err(
                            e,
                            format!("port `{}` needs an instance of `{}`, not `{}`", self.name(port.name), self.name(iname), self.name(got)),
                        ));
                    }
                    return Ok(Connection { port: port.name, expr: None, interface: Some(i) });
                }
                Some(_) => return Ok(Connection { port: port.name, expr: None, interface: None }),
                None => {
                    return Err(self.err(e, format!("port `{}` needs an interface instance", self.name(port.name))));
                }
            }
        }
        // An implicit net for an undeclared simple name (6.10).
        if self.kind(e) == Kind::Name {
            let s = self.sym_of(e);
            if lookup::lookup(self, sc, s).is_none() && !self.defs.contains_key(&s) {
                self.implicit_net(sc, e, s)?;
            }
        }
        let x = match port.dir {
            Dir::Input => match port.object {
                Some(o) => {
                    let t = self.obj(o).ty;
                    expr::rvalue(self, sc, e, t)?
                }
                None => expr::expr(self, sc, e)?,
            },
            _ => {
                let x = expr::expr(self, sc, e)?;
                if matches!(port.dir, Dir::Output | Dir::Inout | Dir::Ref) {
                    checks::port_lvalue(self, sc, port, x, e)?;
                }
                x
            }
        };
        Ok(Connection { port: port.name, expr: Some(x), interface: None })
    }

    fn port_conv(&mut self, port: &Port, e: ExprId, _at: NodeId) -> R<'a, ExprId> {
        if port.dir == Dir::Input
            && let Some(o) = port.object
        {
            let t = self.obj(o).ty;
            return Ok(expr::assign_conv(self, e, t));
        }
        Ok(e)
    }

    /// What an interface port's connection names: an interface instance,
    /// an interface port, or a modport of either.
    fn interface_target(&mut self, sc: ScopeId, e: NodeId) -> Option<Binding> {
        match self.kind(e) {
            Kind::Name => {
                let s = self.sym_of(e);
                lookup::lookup(self, sc, s)
            }
            Kind::Index => {
                let k = self.kids(e);
                let base = self.interface_target(sc, k[0])?;
                if let Binding::Instances(first, count) = base {
                    let idx = eval::const_expr(self, sc, k[1]).ok()?.to_i64()?;
                    let lo = self.d.instances[first.0 as usize].index.unwrap_or(0);
                    let last = self.d.instances[first.0 as usize + count as usize - 1].index.unwrap_or(0);
                    let off = if last >= lo { idx - lo } else { lo - idx };
                    if off < 0 || off >= count as i64 {
                        return None;
                    }
                    return Some(Binding::Instance(InstId(first.0 + off as u32)));
                }
                Some(base)
            }
            Kind::Member => {
                let k = self.kids(e);
                self.interface_target(sc, k[0])
            }
            _ => None,
        }
    }

    pub fn implicit_net(&mut self, sc: ScopeId, at: NodeId, s: Sym) -> R<'a, ObjId> {
        let nt = self.state(at).default_nettype;
        if nt == NetType::None {
            return Err(self.err(
                at,
                format!("`{}` is not declared, and `default_nettype none forbids implicit nets (22.8)", self.name(s)),
            ));
        }
        let kw = format!("{nt:?}").to_lowercase();
        let k = self.sym_owned(kw);
        let sym_implicit = self.sym("implicit");
        let o = self.new_object(Object {
            name: s,
            kind: ObjKind::Net(k),
            ty: Types::LOGIC,
            scope: sc,
            node: at,
            lifetime: Lifetime::Static,
            dir: None,
            init: None,
            value: None,
            qualifiers: vec![sym_implicit],
        });
        self.declare(sc, s, Binding::Object(o), at)?;
        self.push_item(sc, Item::Object(o));
        Ok(o)
    }

    // ---- continuous assignments, gates ----

    fn continuous_assign(&mut self, sc: ScopeId, n: NodeId) -> R<'a> {
        let kids = self.kids(n);
        let strength = kids.iter().copied().find(|&k| self.kind(k) == Kind::Strength).map(|s| {
            let ks = self.kids(s);
            (self.sym_of(ks[0]), ks.get(1).map_or(self.sym_of(ks[0]), |&k| self.sym_of(k)))
        });
        let mut delay = Vec::new();
        if let Some(dl) = kids.iter().copied().find(|&k| self.kind(k) == Kind::Delay) {
            for &k in self.kids(dl) {
                delay.push(expr::expr(self, sc, k)?);
            }
        }
        self.in_cont_assign = true;
        let r = self.continuous_assigns(sc, &kids, strength, &delay);
        self.in_cont_assign = false;
        r
    }

    fn continuous_assigns(&mut self, sc: ScopeId, kids: &[NodeId], strength: Option<(Sym, Sym)>, delay: &[ExprId]) -> R<'a> {
        for a in kids.iter().copied().filter(|&k| self.kind(k) == Kind::Assign).collect::<Vec<_>>() {
            let ak = self.kids(a);
            let (l, r) = (ak[0], ak[ak.len() - 1]);
            // Undeclared names on the left are implicit nets (6.10).
            let names: Vec<NodeId> = if self.kind(l) == Kind::Concat { self.kids(l).to_vec() } else { vec![l] };
            for nm in names {
                if self.kind(nm) == Kind::Name {
                    let s = self.sym_of(nm);
                    if lookup::lookup(self, sc, s).is_none() {
                        self.implicit_net(sc, nm, s)?;
                    }
                }
            }
            let lhs = expr::expr(self, sc, l)?;
            checks::continuous_lvalue(self, sc, lhs, l)?;
            let lt = self.ex(lhs).ty;
            let rhs = expr::rvalue(self, sc, r, lt)?;
            let id = DriverId(self.d.drivers.len() as u32);
            self.d.drivers.push(Driver { lhs, rhs, strength, delay: delay.to_vec(), node: a, scope: sc });
            self.push_item(sc, Item::Driver(id));
        }
        Ok(())
    }

    fn gate(&mut self, sc: ScopeId, n: NodeId, kind: Sym) -> R<'a> {
        let kids = self.kids(n);
        let strength: Vec<Sym> = match kids.iter().copied().find(|&k| self.kind(k) == Kind::Strength) {
            Some(s) => {
                let ks: Vec<NodeId> = self.kids(s).to_vec();
                ks.into_iter().map(|k| self.sym_of(k)).collect()
            }
            None => vec![],
        };
        let mut delay = Vec::new();
        if let Some(dl) = kids.iter().copied().find(|&k| self.kind(k) == Kind::Delay) {
            for &k in self.kids(dl) {
                delay.push(expr::expr(self, sc, k)?);
            }
        }
        for inst in kids.iter().copied().filter(|&k| self.kind(k) == Kind::Instance).collect::<Vec<_>>() {
            let ik = self.kids(inst);
            let name = ik.iter().copied().find(|&k| self.kind(k) == Kind::Name).map(|k| self.sym_of(k));
            let mut terminals = Vec::new();
            for c in ik.iter().copied().filter(|&k| matches!(self.kind(k), Kind::Connection | Kind::NamedConnection)).collect::<Vec<_>>() {
                if self.kind(c) == Kind::NamedConnection {
                    let is_udp = self.defs.get(&kind).is_some_and(|d| d.kind == DefKind2::Udp);
                    if !is_udp {
                        return Err(self.err(c, "gate terminals are connected by position only (28.3)"));
                    }
                }
                let e = self.kids(c).iter().copied().find(|&k| !matches!(self.kind(k), Kind::Attributes));
                match e {
                    Some(e) if self.kind(e) != Kind::Empty => {
                        if self.kind(e) == Kind::Name {
                            let s = self.sym_of(e);
                            if lookup::lookup(self, sc, s).is_none() {
                                self.implicit_net(sc, e, s)?;
                            }
                        }
                        terminals.push(expr::expr(self, sc, e)?);
                    }
                    _ => {}
                }
            }
            checks::gate_terminals(self, sc, n, inst, kind, &terminals)?;
            if let Some(nm) = name
                && !self.scope(sc).names.contains_key(&nm)
            {
                let at = ik.iter().copied().find(|&k| self.kind(k) == Kind::Name).unwrap();
                self.declare(sc, nm, Binding::Other(inst), at)?;
            }
            let id = CellId(self.d.cells.len() as u32);
            self.d.cells.push(Cell { kind, name, terminals, strength: strength.clone(), delay: delay.clone(), node: inst });
            self.push_item(sc, Item::Cell(id));
        }
        Ok(())
    }

    // ---- generate (27) ----

    fn generate_for(&mut self, sc: ScopeId, n: NodeId) -> R<'a> {
        let kids = self.kids(n);
        let (init, cond, step, body) = (kids[0], kids[1], kids[2], kids[3]);
        // A genvar declared in the loop header belongs to the loop (27.4).
        let outer = sc;
        let sc = if self.ast.tokens_of(init).iter().any(|t| t.0 == "genvar") {
            self.new_scope(ScopeKind::Generate, None, Some(outer), n)
        } else {
            sc
        };
        // The loop variable: `genvar i = 0` or `i = 0` of a declared genvar.
        let (gv, init_val) = self.genvar_init(sc, init)?;
        if self.obj(gv).scope == sc && sc != outer {
            let gname = self.obj(gv).name;
            self.scope_mut(sc).names.insert(gname, Binding::Object(gv));
        }
        let gname = self.obj(gv).name;
        if self.genvars_active.contains(&gv) {
            return Err(self.err(init, format!("genvar `{}` is already the index of an enclosing loop (27.4)", self.name(gname))));
        }
        checks::genvar_step(self, sc, step, gname)?;
        // The body is usually a named block; its name names the array.
        let (block_name, block_label) = self.block_name(body);
        let name = match block_name {
            Some(s) => s,
            None => self.genblk_name(outer),
        };
        let container = self.new_scope(ScopeKind::Generate, Some(name), Some(sc), n);
        self.declare(outer, name, Binding::Scope(container), block_label.unwrap_or(n))?;
        self.push_item(outer, Item::Scope(container));
        self.genvars_active.push(gv);
        let mut v = init_val;
        let mut seen = std::collections::HashSet::new();
        let mut count = 0;
        let r = (|| -> R<'a> {
            loop {
                self.obj_mut(gv).value = Some(ConstId(u32::MAX));
                let vc = self.new_const(Const::Bits(Bits::from_i64(32, true, v)));
                self.obj_mut(gv).value = Some(vc);
                let c = eval::const_expr(self, sc, cond)?;
                let Some(b) = c.as_bits().and_then(|b| b.truth()) else {
                    return Err(self.err(cond, "the generate loop's condition is not a known value"));
                };
                if !b {
                    break;
                }
                if !seen.insert(v) {
                    return Err(self.err(step, format!("genvar `{}` takes the value {v} twice (27.4)", self.name(gname))));
                }
                count += 1;
                if count > 1_000_000 {
                    return Err(self.err(cond, "the generate loop's condition never becomes false"));
                }
                // One copy of the body, with a localparam of the genvar.
                let blk = self.new_scope(ScopeKind::Generate, Some(name), Some(container), body);
                self.scope_mut(blk).index = Some(v);
                self.push_item(container, Item::Scope(blk));
                let p = self.new_object(Object {
                    name: gname,
                    kind: ObjKind::Param { local: true },
                    ty: Types::INTEGER,
                    scope: blk,
                    node: init,
                    lifetime: Lifetime::Static,
                    dir: None,
                    init: None,
                    value: Some(vc),
                    qualifiers: vec![],
                });
                self.scope_mut(blk).names.insert(gname, Binding::Object(p));
                self.generate_body(blk, body)?;
                v = self.genvar_step_value(sc, step, gv, v)?;
            }
            Ok(())
        })();
        self.genvars_active.pop();
        self.obj_mut(gv).value = None;
        r
    }

    fn genvar_init(&mut self, sc: ScopeId, init: NodeId) -> R<'a, (ObjId, i64)> {
        // `genvar i = 0` parses as a declaration; `i = 0` as an assignment.
        let (name_node, val_node) = match self.kind(init) {
            Kind::Assign => {
                let k = self.kids(init);
                let name = k.iter().copied().find(|&x| self.kind(x) == Kind::Name).unwrap_or(k[0]);
                (name, k[k.len() - 1])
            }
            Kind::VarDecl | Kind::GenvarDecl => {
                let d = self.kids(init).iter().copied().find(|&k| self.kind(k) == Kind::Declarator).unwrap_or(init);
                let k = self.kids(d);
                (k[0], k[k.len() - 1])
            }
            _ => {
                let k = self.kids(init);
                let a = k.iter().copied().find(|&x| self.kind(x) == Kind::Assign || self.kind(x) == Kind::Declarator);
                match a {
                    Some(a) => {
                        let k2 = self.kids(a);
                        (k2[0], k2[k2.len() - 1])
                    }
                    None => return Err(self.err(init, "a generate loop starts by setting its genvar")),
                }
            }
        };
        let declares = self.ast.tokens_of(init).iter().any(|t| t.0 == "genvar");
        let s = self.sym_of(name_node);
        let decl_scope = sc;
        if checks::mentions(self, val_node, s) {
            return Err(self.err(val_node, format!("the loop's initial value reads its own genvar `{}` (27.4)", self.name(s))));
        }
        let gv = if declares {
            let o = self.new_object(Object {
                name: s,
                kind: ObjKind::Genvar,
                ty: Types::INTEGER,
                scope: decl_scope,
                node: init,
                lifetime: Lifetime::Static,
                dir: None,
                init: None,
                value: None,
                qualifiers: vec![],
            });
            o
        } else {
            match lookup::lookup(self, sc, s) {
                Some(Binding::Object(o)) if self.obj(o).kind == ObjKind::Genvar => o,
                Some(_) => {
                    return Err(self.err(name_node, format!("`{}` is not a genvar, so it cannot index a generate loop", self.name(s))));
                }
                None => return Err(self.err(name_node, format!("genvar `{}` is not declared", self.name(s)))),
            }
        };
        let v = eval::const_expr(self, sc, val_node)?;
        let Some(v) = v.to_i64() else {
            return Err(self.err(val_node, "a genvar's value must be known (27.4)"));
        };
        Ok((gv, v))
    }

    fn genvar_step_value(&mut self, sc: ScopeId, step: NodeId, gv: ObjId, v: i64) -> R<'a, i64> {
        // Evaluate `i = i + 1`, `i++`, `i += 2` with the genvar at `v`.
        let a = if self.kind(step) == Kind::ForStep || self.kind(step) == Kind::ExprStatement {
            self.kids(step).first().copied().unwrap_or(step)
        } else {
            step
        };
        let _ = gv;
        match self.kind(a) {
            Kind::Assign => {
                let k = self.kids(a);
                let op = self.text(k[1]);
                let rhs = eval::const_expr(self, sc, k[k.len() - 1])?.to_i64().unwrap_or(0);
                Ok(match op {
                    "=" => rhs,
                    "+=" => v + rhs,
                    "-=" => v - rhs,
                    "*=" => v * rhs,
                    "/=" if rhs != 0 => v / rhs,
                    "<<=" => v << rhs,
                    ">>=" => v >> rhs,
                    _ => return Err(self.err(a, "this genvar step is not supported")),
                })
            }
            Kind::IncDec => {
                let txt = self.ast.text(a);
                Ok(if txt.contains("++") { v + 1 } else { v - 1 })
            }
            _ => {
                let e = eval::const_expr(self, sc, a)?;
                Ok(e.to_i64().unwrap_or(v))
            }
        }
    }

    fn generate_if(&mut self, sc: ScopeId, n: NodeId) -> R<'a> {
        let kids = self.kids(n);
        let c = eval::const_expr(self, sc, kids[0]).map_err(|e| {
            if e.message.contains("not a constant") || e.message.contains("constant") {
                self.err(kids[0], format!("a generate condition must be a constant expression: {}", e.message))
            } else {
                e
            }
        })?;
        let taken = c.as_bits().and_then(|b| b.truth()).unwrap_or(false);
        // The names of blocks not taken are still names in the scope (27.5).
        self.reserve_block_names(sc, n)?;
        let chosen = if taken { Some(kids[1]) } else { kids.get(2).copied() };
        if let Some(b) = chosen {
            self.generate_branch(sc, b)?;
        }
        Ok(())
    }

    fn generate_case(&mut self, sc: ScopeId, n: NodeId) -> R<'a> {
        let kids = self.kids(n);
        let v = eval::const_expr(self, sc, kids[0])?;
        self.reserve_block_names(sc, n)?;
        let mut default = None;
        let mut chosen = None;
        let mut defaults = 0;
        for &item in &kids[1..] {
            let ik = self.kids(item);
            let body = *ik.last().unwrap();
            if ik.len() == 1 {
                default = Some(body);
                defaults += 1;
                if defaults > 1 {
                    return Err(self.err(item, "a case generate has two default items (27.5)"));
                }
                continue;
            }
            for &e in &ik[..ik.len() - 1] {
                let c = eval::const_expr(self, sc, e)?;
                if chosen.is_none() && eval::const_eq(&v, &c) {
                    chosen = Some(body);
                }
            }
        }
        if let Some(b) = chosen.or(default) {
            self.generate_branch(sc, b)?;
        }
        Ok(())
    }

    /// A branch of a conditional generate: a block, a nested conditional
    /// (which needs no block of its own), or a single item.
    fn generate_branch(&mut self, sc: ScopeId, b: NodeId) -> R<'a> {
        match self.kind(b) {
            Kind::GenerateIf => self.generate_if(sc, b),
            Kind::GenerateCase => self.generate_case(sc, b),
            Kind::GenerateBlock => self.generate_block(sc, b, None, None),
            Kind::Empty | Kind::NullStatement => Ok(()),
            _ => {
                // A single item gets an unnamed block (27.6).
                let name = self.genblk_name(sc);
                let blk = self.new_scope(ScopeKind::Generate, Some(name), Some(sc), b);
                self.scope_mut(sc).names.entry(name).or_insert(Binding::Scope(blk));
                self.push_item(sc, Item::Scope(blk));
                let items = [b];
                self.declare_pass(blk, &items)?;
                self.item(blk, b)
            }
        }
    }

    /// Reserves the names of every block of a conditional generate, taken
    /// or not, and checks they do not clash with other names (27.5).
    fn reserve_block_names(&mut self, sc: ScopeId, n: NodeId) -> R<'a> {
        let mut names = Vec::new();
        self.collect_block_names(n, &mut names);
        for (s, at) in names {
            if let Some(b) = self.scope(sc).names.get(&s).copied()
                && !matches!(b, Binding::Scope(_))
            {
                return Err(self.err(
                    at,
                    format!("the generate block name `{}` is already used in this scope for {}", self.name(s), self.describe_binding(b)),
                ));
            }
            self.reserved_blocks.entry(sc).or_default().push((s, n));
        }
        Ok(())
    }

    fn collect_block_names(&mut self, n: NodeId, out: &mut Vec<(Sym, NodeId)>) {
        for &k in self.kids(n) {
            match self.kind(k) {
                Kind::GenerateBlock => {
                    if let Some(nm) = self.name_child(k) {
                        let s = self.sym_of(nm);
                        out.push((s, nm));
                    }
                }
                Kind::GenerateIf | Kind::GenerateCase | Kind::GenerateCaseItem => self.collect_block_names(k, out),
                _ => {}
            }
        }
    }

    fn block_name(&mut self, b: NodeId) -> (Option<Sym>, Option<NodeId>) {
        if self.kind(b) == Kind::GenerateBlock
            && let Some(nm) = self.name_child(b)
        {
            return (Some(self.sym_of(nm)), Some(nm));
        }
        (None, None)
    }

    /// `genblk<n>`: the name of the n-th unnamed generate construct in the
    /// scope (27.6).
    /// `genblk<n>` for an unnamed block of the scope's n-th generate
    /// construct, with zeros added before n while it clashes with a
    /// declared name (27.6).
    fn genblk_name(&mut self, sc: ScopeId) -> Sym {
        let k = self.genblk_count.get(&sc).copied().unwrap_or(0).max(1);
        let declared = self.declared_names(sc);
        let mut zeros = String::new();
        loop {
            let text = format!("genblk{zeros}{k}");
            let s = self.sym_owned(text.clone());
            if !self.scope(sc).names.contains_key(&s) && !declared.contains(&text) {
                return s;
            }
            zeros.push('0');
        }
    }

    /// Names declared anywhere directly in a scope's syntax, so that a
    /// generated name can avoid them even if they come later.
    fn declared_names(&self, sc: ScopeId) -> std::collections::HashSet<String> {
        let mut out = std::collections::HashSet::new();
        let node = self.scope(sc).node;
        for &k in self.ast.children(node) {
            for &c in self.ast.children(k) {
                match self.ast.kind(c) {
                    Kind::Declarator | Kind::Instance => {
                        if let Some(nm) = self.ast.child(c, Kind::Name) {
                            out.insert(self.ast.token(nm).0.to_string());
                        }
                    }
                    Kind::Name if matches!(self.ast.kind(k), Kind::GenerateBlock | Kind::Typedef | Kind::Function | Kind::Task) => {
                        out.insert(self.ast.token(c).0.to_string());
                    }
                    _ => {}
                }
            }
        }
        out
    }

    /// Counts a generate construct in its scope (27.6).
    fn count_construct(&mut self, sc: ScopeId) {
        *self.genblk_count.entry(sc).or_insert(0) += 1;
    }

    pub fn generate_block(&mut self, sc: ScopeId, n: NodeId, name: Option<Sym>, _index: Option<i64>) -> R<'a> {
        let label = self.name_child(n);
        let name = match (name, label) {
            (Some(s), _) => s,
            (None, Some(l)) => self.sym_of(l),
            (None, None) => self.genblk_name(sc),
        };
        let blk = self.new_scope(ScopeKind::Generate, Some(name), Some(sc), n);
        // Two blocks of one conditional may share a name; only one exists.
        if let Some(old) = self.scope(sc).names.get(&name).copied() {
            let reserved_here = self.reserved_blocks.get(&sc).is_some_and(|v| v.iter().any(|(s, _)| *s == name));
            if !(reserved_here && matches!(old, Binding::Scope(_))) {
                return Err(self.err(label.unwrap_or(n), format!("`{}` is already declared in this scope", self.name(name))));
            }
        }
        self.scope_mut(sc).names.insert(name, Binding::Scope(blk));
        self.push_item(sc, Item::Scope(blk));
        self.generate_body(blk, n)
    }

    fn generate_body(&mut self, blk: ScopeId, body: NodeId) -> R<'a> {
        let items: Vec<NodeId> = if self.kind(body) == Kind::GenerateBlock {
            self.kids(body).iter().copied().filter(|&k| !matches!(self.kind(k), Kind::Name | Kind::EndLabel)).collect()
        } else {
            vec![body]
        };
        self.declare_pass(blk, &items)?;
        for k in items {
            self.item(blk, k)?;
        }
        Ok(())
    }
}

/// A parameter override: a value, a type, or an expression to evaluate.
#[derive(Clone, Debug)]
pub(crate) enum Over {
    Value(Const),
    Type(TypeId),
    #[allow(dead_code)]
    Node(NodeId, ScopeId),
}
