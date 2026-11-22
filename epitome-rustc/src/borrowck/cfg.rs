//! The control-flow graph of a body, for borrow checking.
//!
//! Built from the typed syntax tree (`build.rs`) and used by nothing else:
//! the compiler has no MIR, and this is not one. It is MIR-shaped because
//! that is what the analyses want: a body is basic blocks of statements
//! that assign places and take borrows, ending in a terminator; places are a
//! local and a path of projections.
//!
//! What it leaves out, for now:
//! - Whether a use of a place copies or moves it: an operand is `Use`, and
//!   the move analysis decides (it needs the body's trait environment).
//! - Unwinding: a call has one successor.
//! - Drop elaboration: `Drop` is emitted for every variable that goes out of
//!   scope; whether its type needs dropping is for the analyses.
//!
//! `Cfg::to_rust` shows a body as Rust: a state machine, one `match` arm for
//! each block. Downcasts (`(_3 as Some).0`) are the one thing in it that is
//! not Rust.

use crate::ast::NodeId;
use crate::resolve::{DefId, Resolver};
use crate::ty::{ClosureId, List, Mutbl, Ty, TyCtxt, TyKind};
use std::fmt::Write;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, PartialOrd, Ord)]
pub struct Local(pub u32);

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, PartialOrd, Ord)]
pub struct Block(pub u32);

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum LocalKind {
    /// `_0`, where the result goes.
    Return,
    /// A parameter (`_1`, ...).
    Arg,
    /// A variable the program names.
    Var,
    /// A temporary the builder made.
    Temp,
    /// A static item, as used here (`mutable` if it is `static mut`).
    Static,
}

#[derive(Clone, Debug)]
pub struct LocalDecl {
    pub ty: Ty,
    pub kind: LocalKind,
    /// The name in the program, for variables and parameters.
    pub name: Option<String>,
    /// Declared `mut`.
    pub mutable: bool,
    /// The pattern (or parameter) that declares it.
    pub node: Option<NodeId>,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Proj {
    Deref,
    /// Field `index` (of the variant a `Downcast` before it chose), of this
    /// type.
    Field(u32, Ty),
    /// An enum place taken as one of its variants.
    Downcast(u32),
    /// `p[i]`, `i` a local; of this element type.
    Index(Local, Ty),
    /// `p[k]` or `p[len - k]` (from the end), as slice patterns make.
    ConstIndex(u64, bool, Ty),
    /// `p[from..len - to]` (`from_end`) or `p[from..to]`.
    Subslice(u64, u64, bool, Ty),
}

#[derive(Clone, Debug)]
pub struct Place {
    pub local: Local,
    pub proj: Vec<Proj>,
    /// The type of the place.
    pub ty: Ty,
    /// The expression it was named by, where there is one: what an error
    /// about its use points at. (Not part of what the place is.)
    pub at: Option<NodeId>,
}

impl PartialEq for Place {
    fn eq(&self, o: &Place) -> bool {
        self.local == o.local && self.proj == o.proj && self.ty == o.ty
    }
}

impl Eq for Place {}

impl std::hash::Hash for Place {
    fn hash<H: std::hash::Hasher>(&self, h: &mut H) {
        self.local.hash(h);
        self.proj.hash(h);
        self.ty.hash(h);
    }
}

impl Place {
    pub fn local(local: Local, ty: Ty) -> Place {
        Place { local, proj: Vec::new(), ty, at: None }
    }

    /// This place with `p` added, of type `ty`.
    pub fn project(&self, p: Proj, ty: Ty) -> Place {
        let mut proj = self.proj.clone();
        proj.push(p);
        Place { local: self.local, proj, ty, at: self.at }
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Operand {
    /// The value of a place: a copy, or a move, which the move analysis says.
    Use(Place),
    /// A constant, as written (`1u8`, `"s"`, `CONST`, `foo::<T>`).
    Const(String, Ty),
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum AggKind {
    Tuple,
    Array,
    /// A struct, or one variant of an enum, with the type's arguments.
    Adt(DefId, u32, List),
    Closure(ClosureId),
    /// A union with one field (this one) given.
    Union(DefId, u32, List),
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Rvalue {
    Use(Operand),
    /// `&p` or `&mut p`.
    Ref(Mutbl, Place),
    /// `&raw const p` or `&raw mut p`.
    AddrOf(Mutbl, Place),
    /// A closure's shared capture of a variable: of the parts of it the
    /// closure uses, which are not known here, so not of the parts.
    CaptureRef(Place),
    Binary(String, Operand, Operand),
    Unary(String, Operand),
    /// `as`, and the coercions that change a value (unsizing, to `fn`).
    Cast(Operand, Ty),
    Aggregate(AggKind, Vec<Operand>),
    /// `[x; N]`, `N` as written.
    Repeat(Operand, String),
    Discriminant(Place),
    Len(Place),
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum StmtKind {
    Assign(Place, Rvalue),
    StorageLive(Local),
    StorageDead(Local),
    /// The value in the place is dropped (if its type needs it).
    Drop(Place),
    /// The place may have been given a value by code that is not followed
    /// (an `asm!`'s output).
    MaybeInit(Place),
}

#[derive(Clone, Debug)]
pub struct Stmt {
    /// A mutable borrow that only reserves until the call it is an argument
    /// of (an autoref receiver).
    pub two_phase: bool,
    /// A `CaptureRef` of what the closure captures by value (a field of it,
    /// moved): no borrow.
    pub moved: bool,
    pub kind: StmtKind,
    /// Where in the source, for errors.
    pub node: NodeId,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Callee {
    /// A function item (or a method, or a trait's method with `Self` first
    /// in the arguments) and its generic arguments.
    Fn(DefId, List),
    /// A function pointer, or any other value called.
    Value(Operand),
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Term {
    Goto(Block),
    /// Branches on the value: to the block of the first target that has it,
    /// else `otherwise`. (For a `bool`, `[(0, else)]` and `then` otherwise.)
    Switch { discr: Operand, targets: Vec<(u128, Block)>, otherwise: Block },
    /// `dest = func(args)`, then `target`; none if it never returns.
    Call { func: Callee, args: Vec<Operand>, dest: Place, target: Option<Block> },
    Return,
    Unreachable,
}

impl Term {
    /// The blocks control can go to next.
    pub fn successors(&self) -> Vec<Block> {
        match self {
            Term::Goto(b) => vec![*b],
            Term::Switch { targets, otherwise, .. } => {
                let mut v: Vec<Block> = targets.iter().map(|t| t.1).collect();
                v.push(*otherwise);
                v
            }
            Term::Call { target, .. } => target.iter().copied().collect(),
            Term::Return | Term::Unreachable => Vec::new(),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct BlockData {
    pub stmts: Vec<Stmt>,
    pub term: Option<(Term, NodeId)>,
}

/// One body: a function, a constant, or a closure.
#[derive(Debug)]
pub struct Cfg {
    pub owner: DefId,
    pub krate: u32,
    pub file: u32,
    /// The item (or closure expression) it is the body of.
    pub node: NodeId,
    pub closure: Option<ClosureId>,
    /// What to call it: the item's path (and `{closure#n}`).
    pub name: String,
    pub locals: Vec<LocalDecl>,
    pub arg_count: usize,
    pub blocks: Vec<BlockData>,
    /// In a closure: for each variable captured, in order of the fields of its
    /// environment: whether it was declared `mut`, whether it is held by value,
    /// and its type.
    pub upvars: Vec<(bool, bool, Ty)>,
}

impl Cfg {
    pub const ENTRY: Block = Block(0);

    /// Every block is ended, every target and local exists.
    pub fn validate(&self) -> Result<(), String> {
        let (nb, nl) = (self.blocks.len() as u32, self.locals.len() as u32);
        let place_ok = |p: &Place| -> Result<(), String> {
            if p.local.0 >= nl {
                return Err(format!("no local _{}", p.local.0));
            }
            for pr in &p.proj {
                if let Proj::Index(l, _) = pr
                    && l.0 >= nl
                {
                    return Err(format!("no local _{}", l.0));
                }
            }
            Ok(())
        };
        let operand_ok = |o: &Operand| match o {
            Operand::Use(p) => place_ok(p),
            Operand::Const(..) => Ok(()),
        };
        for (i, b) in self.blocks.iter().enumerate() {
            for s in &b.stmts {
                match &s.kind {
                    StmtKind::Assign(p, rv) => {
                        place_ok(p)?;
                        match rv {
                            Rvalue::Use(o) | Rvalue::Unary(_, o) | Rvalue::Cast(o, _) | Rvalue::Repeat(o, _) => operand_ok(o)?,
                            Rvalue::Ref(_, p) | Rvalue::AddrOf(_, p) | Rvalue::CaptureRef(p) | Rvalue::Discriminant(p) | Rvalue::Len(p) => place_ok(p)?,
                            Rvalue::Binary(_, a, b) => {
                                operand_ok(a)?;
                                operand_ok(b)?
                            }
                            Rvalue::Aggregate(_, os) => {
                                for o in os {
                                    operand_ok(o)?
                                }
                            }
                        }
                    }
                    StmtKind::StorageLive(l) | StmtKind::StorageDead(l) => {
                        if l.0 >= nl {
                            return Err(format!("no local _{}", l.0));
                        }
                    }
                    StmtKind::Drop(p) | StmtKind::MaybeInit(p) => place_ok(p)?,
                }
            }
            let Some((t, _)) = &b.term else { return Err(format!("bb{i} has no terminator")) };
            for s in t.successors() {
                if s.0 >= nb {
                    return Err(format!("bb{i} goes to bb{}, which does not exist", s.0));
                }
            }
            match t {
                Term::Switch { discr, .. } => operand_ok(discr)?,
                Term::Call { func, args, dest, .. } => {
                    if let Callee::Value(o) = func {
                        operand_ok(o)?;
                    }
                    for a in args {
                        operand_ok(a)?;
                    }
                    place_ok(dest)?;
                }
                _ => {}
            }
        }
        Ok(())
    }

    /// The blocks reachable from the entry, in order of first reaching.
    pub fn reachable(&self) -> Vec<Block> {
        let mut seen = vec![false; self.blocks.len()];
        let mut order = Vec::new();
        let mut stack = vec![Cfg::ENTRY];
        while let Some(b) = stack.pop() {
            if std::mem::replace(&mut seen[b.0 as usize], true) {
                continue;
            }
            order.push(b);
            if let Some((t, _)) = &self.blocks[b.0 as usize].term {
                stack.extend(t.successors().into_iter().rev());
            }
        }
        order
    }
}

// ---- showing it as Rust ----

/// What showing a body needs.
pub struct Show<'a, 's> {
    pub tcx: &'a TyCtxt<'s>,
    pub r: &'a Resolver<'s>,
    /// The line and column of a node, for a comment on each statement.
    pub at: &'a dyn Fn(NodeId) -> Option<(usize, usize)>,
    /// Initialise every local, so that the result is code `rustc` takes
    /// (for values it need not move out of twice: the numbers, say).
    pub compilable: bool,
}

impl<'a, 's> Show<'a, 's> {
    fn ty(&self, t: Ty) -> String {
        self.tcx.display(self.r, t)
    }

    fn local(&self, cfg: &Cfg, l: Local) -> String {
        let _ = cfg;
        format!("_{}", l.0)
    }

    fn place(&self, cfg: &Cfg, p: &Place) -> String {
        let mut s = self.local(cfg, p.local);
        for pr in &p.proj {
            s = match pr {
                Proj::Deref => format!("(*{s})"),
                Proj::Field(i, _) => format!("{s}.{i}"),
                Proj::Downcast(v) => {
                    // The variant's name is what a reader wants; the index is
                    // what is there.
                    format!("({s} as variant#{v})")
                }
                Proj::Index(l, _) => format!("{s}[{}]", self.local(cfg, *l)),
                Proj::ConstIndex(k, false, _) => format!("{s}[{k}]"),
                Proj::ConstIndex(k, true, _) => format!("{s}[len - {k}]"),
                Proj::Subslice(a, b, false, _) => format!("{s}[{a}..{b}]"),
                Proj::Subslice(a, b, true, _) => format!("{s}[{a}..len - {b}]"),
            };
        }
        s
    }

    fn operand(&self, cfg: &Cfg, o: &Operand) -> String {
        match o {
            Operand::Use(p) => self.place(cfg, p),
            Operand::Const(c, _) => c.clone(),
        }
    }

    fn operands(&self, cfg: &Cfg, os: &[Operand]) -> String {
        os.iter().map(|o| self.operand(cfg, o)).collect::<Vec<_>>().join(", ")
    }

    fn adt_name(&self, d: DefId) -> String {
        self.r.def_path(d)
    }

    fn rvalue(&self, cfg: &Cfg, rv: &Rvalue) -> String {
        match rv {
            Rvalue::Use(o) => self.operand(cfg, o),
            Rvalue::Ref(Mutbl::Not, p) => format!("&{}", self.place(cfg, p)),
            Rvalue::Ref(Mutbl::Mut, p) => format!("&mut {}", self.place(cfg, p)),
            Rvalue::AddrOf(Mutbl::Not, p) => format!("&raw const {}", self.place(cfg, p)),
            Rvalue::AddrOf(Mutbl::Mut, p) => format!("&raw mut {}", self.place(cfg, p)),
            Rvalue::CaptureRef(p) => format!("&capture {}", self.place(cfg, p)),
            Rvalue::Binary(op, a, b) => format!("{} {op} {}", self.operand(cfg, a), self.operand(cfg, b)),
            Rvalue::Unary(op, a) => format!("{op}{}", self.operand(cfg, a)),
            Rvalue::Cast(o, t) => format!("{} as {}", self.operand(cfg, o), self.ty(*t)),
            Rvalue::Aggregate(AggKind::Tuple, os) => match os.len() {
                1 => format!("({},)", self.operands(cfg, os)),
                _ => format!("({})", self.operands(cfg, os)),
            },
            Rvalue::Aggregate(AggKind::Array, os) => format!("[{}]", self.operands(cfg, os)),
            Rvalue::Aggregate(AggKind::Adt(d, v, _), os) => {
                let fields: Vec<String> = os.iter().enumerate().map(|(i, o)| format!("{i}: {}", self.operand(cfg, o))).collect();
                let name = self.adt_name(*d);
                let _ = v;
                if fields.is_empty() { name } else { format!("{name} {{ {} }}", fields.join(", ")) }
            }
            Rvalue::Aggregate(AggKind::Union(d, i, _), os) => format!("{} {{ {i}: {} }}", self.adt_name(*d), self.operands(cfg, os)),
            Rvalue::Aggregate(AggKind::Closure(c), os) => format!("closure#{}({})", c.0, self.operands(cfg, os)),
            Rvalue::Repeat(o, n) => format!("[{}; {n}]", self.operand(cfg, o)),
            Rvalue::Discriminant(p) => format!("discriminant({})", self.place(cfg, p)),
            Rvalue::Len(p) => format!("len({})", self.place(cfg, p)),
        }
    }

    fn comment(&self, node: NodeId) -> String {
        match (self.at)(node) {
            Some((l, c)) => format!(" // {l}:{c}"),
            None => String::new(),
        }
    }

    /// A statement, as one line with no position.
    pub fn stmt_text(&self, cfg: &Cfg, s: &Stmt) -> String {
        match &s.kind {
            StmtKind::Assign(p, rv) => format!("{} = {}", self.place(cfg, p), self.rvalue(cfg, rv)),
            StmtKind::StorageLive(l) => format!("StorageLive({})", self.local(cfg, *l)),
            StmtKind::StorageDead(l) => format!("StorageDead({})", self.local(cfg, *l)),
            StmtKind::Drop(p) => format!("drop({})", self.place(cfg, p)),
            StmtKind::MaybeInit(p) => format!("maybe_init({})", self.place(cfg, p)),
        }
    }

    /// A terminator, as one line.
    pub fn term_text(&self, cfg: &Cfg, t: &Term) -> String {
        match t {
            Term::Goto(b) => format!("goto bb{}", b.0),
            Term::Switch { discr, targets, otherwise } => {
                let d = self.operand(cfg, discr);
                if targets.len() == 1 && targets[0].0 == 0 && matches!(discr_ty(cfg, discr, self.tcx), Some(TyKind::Bool)) {
                    format!("if {d}: bb{} else bb{}", otherwise.0, targets[0].1.0)
                } else {
                    let arms: Vec<String> = targets.iter().map(|(v, b)| format!("{v} -> bb{}", b.0)).collect();
                    format!("switch {d}: {}, else bb{}", arms.join(", "), otherwise.0)
                }
            }
            Term::Call { func, args, dest, target } => {
                let f = match func {
                    Callee::Fn(d, l) => {
                        let targs = self.tcx.tys(*l);
                        if targs.is_empty() {
                            self.r.def_path(*d)
                        } else {
                            let t: Vec<String> = targs.iter().map(|&t| self.ty(t)).collect();
                            format!("{}::<{}>", self.r.def_path(*d), t.join(", "))
                        }
                    }
                    Callee::Value(o) => format!("({})", self.operand(cfg, o)),
                };
                let call = format!("{} = {f}({})", self.place(cfg, dest), self.operands(cfg, args));
                match target {
                    Some(b) => format!("{call}, then bb{}", b.0),
                    None => format!("{call}, never returns"),
                }
            }
            Term::Return => "return".to_string(),
            Term::Unreachable => "unreachable".to_string(),
        }
    }

    /// The locals the program names, and the result, as a comment.
    pub fn legend(&self, cfg: &Cfg) -> String {
        let mut parts = Vec::new();
        for (i, d) in cfg.locals.iter().enumerate() {
            match (&d.name, d.kind) {
                (_, LocalKind::Return) => parts.push(format!("_0: {} (result)", self.ty(d.ty))),
                (Some(n), _) => parts.push(format!("_{i}: {} ({n})", self.ty(d.ty))),
                _ => {}
            }
        }
        parts.join(", ")
    }

    fn stmt(&self, cfg: &Cfg, s: &Stmt) -> String {
        let c = self.comment(s.node);
        match &s.kind {
            StmtKind::Assign(p, rv) => format!("{} = {};{c}", self.place(cfg, p), self.rvalue(cfg, rv)),
            StmtKind::StorageLive(l) => format!("/* StorageLive({}) */{c}", self.local(cfg, *l)),
            StmtKind::StorageDead(l) => format!("/* StorageDead({}) */{c}", self.local(cfg, *l)),
            StmtKind::Drop(p) => format!("/* drop({}) */{c}", self.place(cfg, p)),
            StmtKind::MaybeInit(p) => format!("/* maybe initialised by asm!: {} */{c}", self.place(cfg, p)),
        }
    }

    fn term(&self, cfg: &Cfg, t: &Term, node: NodeId, out: &mut String, ind: &str) {
        let c = self.comment(node);
        match t {
            Term::Goto(b) => {
                let _ = writeln!(out, "{ind}bb = {};{c}", b.0);
            }
            Term::Switch { discr, targets, otherwise } => {
                let d = self.operand(cfg, discr);
                if targets.len() == 1 && targets[0].0 == 0 && matches!(discr_ty(cfg, discr, self.tcx), Some(TyKind::Bool)) {
                    let _ = writeln!(out, "{ind}bb = if {d} {{ {} }} else {{ {} }};{c}", otherwise.0, targets[0].1.0);
                } else {
                    let arms: Vec<String> = targets.iter().map(|(v, b)| format!("{v} => {}", b.0)).collect();
                    let _ = writeln!(out, "{ind}bb = match {d} {{ {}, _ => {} }};{c}", arms.join(", "), otherwise.0);
                }
            }
            Term::Call { func, args, dest, target } => {
                let f = match func {
                    Callee::Fn(d, l) => {
                        let targs = self.tcx.tys(*l);
                        if targs.is_empty() {
                            self.r.def_path(*d)
                        } else {
                            let t: Vec<String> = targs.iter().map(|&t| self.ty(t)).collect();
                            format!("{}::<{}>", self.r.def_path(*d), t.join(", "))
                        }
                    }
                    Callee::Value(o) => format!("({})", self.operand(cfg, o)),
                };
                let call = format!("{f}({})", self.operands(cfg, args));
                match target {
                    Some(b) => {
                        let _ = writeln!(out, "{ind}{} = {call};{c}", self.place(cfg, dest));
                        let _ = writeln!(out, "{ind}bb = {};", b.0);
                    }
                    None => {
                        let _ = writeln!(out, "{ind}{} = {call};{c} // does not return", self.place(cfg, dest));
                        let _ = writeln!(out, "{ind}unreachable!();");
                    }
                }
            }
            Term::Return => {
                let _ = writeln!(out, "{ind}return _0;{c}");
            }
            Term::Unreachable => {
                let _ = writeln!(out, "{ind}unreachable!();{c}");
            }
        }
    }

    /// The body as a Rust function.
    pub fn cfg(&self, cfg: &Cfg) -> String {
        let mut out = String::new();
        let args: Vec<String> = (1..=cfg.arg_count)
            .map(|i| {
                let d = &cfg.locals[i];
                format!("mut _{i}: {}", self.ty(d.ty))
            })
            .collect();
        let ret = &cfg.locals[0];
        let _ = writeln!(out, "// {}", cfg.name);
        let name: String = cfg.name.chars().map(|c| if c.is_alphanumeric() { c } else { '_' }).collect();
        let _ = writeln!(out, "fn {name}({}) -> {} {{", args.join(", "), self.ty(ret.ty));
        for (i, d) in cfg.locals.iter().enumerate() {
            if i >= 1 && i <= cfg.arg_count {
                if let Some(n) = &d.name {
                    let _ = writeln!(out, "    // _{i} is `{n}`");
                }
                continue;
            }
            let note = match (&d.name, d.kind) {
                (Some(n), _) => format!(" // {n}"),
                (None, LocalKind::Return) => " // the result".to_string(),
                _ => String::new(),
            };
            if self.compilable {
                // (`!` is not a type a local may have.)
                let ty = if matches!(self.tcx.kind(d.ty), TyKind::Never) { "()".to_string() } else { self.ty(d.ty) };
                let _ = writeln!(out, "    let mut _{i}: {ty} = unsafe {{ core::mem::zeroed() }};{note}");
            } else {
                let _ = writeln!(out, "    let mut _{i}: {};{note}", self.ty(d.ty));
            }
        }
        let _ = writeln!(out, "    let mut bb: u32 = 0;");
        let _ = writeln!(out, "    loop {{");
        let _ = writeln!(out, "        match bb {{");
        for b in cfg.reachable_sorted() {
            let data = &cfg.blocks[b.0 as usize];
            let _ = writeln!(out, "            {} => {{", b.0);
            for s in &data.stmts {
                let _ = writeln!(out, "                {}", self.stmt(cfg, s));
            }
            if let Some((t, node)) = &data.term {
                self.term(cfg, t, *node, &mut out, "                ");
            }
            let _ = writeln!(out, "            }}");
        }
        let _ = writeln!(out, "            _ => unreachable!(),");
        let _ = writeln!(out, "        }}");
        let _ = writeln!(out, "    }}");
        let _ = writeln!(out, "}}");
        out
    }
}

fn discr_ty<'a>(cfg: &Cfg, o: &Operand, tcx: &'a TyCtxt) -> Option<TyKind> {
    let t = match o {
        Operand::Use(p) => p.ty,
        Operand::Const(_, t) => *t,
    };
    let _ = cfg;
    Some(tcx.kind(t))
}

impl Cfg {
    /// The reachable blocks, by number.
    pub fn reachable_sorted(&self) -> Vec<Block> {
        let mut v = self.reachable();
        v.sort();
        v
    }
}
