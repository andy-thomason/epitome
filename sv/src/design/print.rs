//! The design as text, for `epitome-sv --design` and for tests.
//!
//! Each specialisation is printed once, with its parameter values and how
//! many instances share it, followed by the hierarchy. Code is printed in
//! SystemVerilog's syntax with every conversion clause 11 implies written
//! out: `zext<8>(x)`, `sext<8>(x)` and `trunc<4>(x)` change an integral
//! value's width, `cvt<T>(x)` any other conversion.

use super::types::{Type, Types};
use super::*;
use std::fmt::Write;

pub fn print(d: &Design) -> String {
    let mut p = Printer { d, out: String::new(), depth: 0 };
    p.design();
    p.out
}

struct Printer<'d, 'a> {
    d: &'d Design<'a>,
    out: String,
    depth: usize,
}

impl Printer<'_, '_> {
    fn line(&mut self, s: impl AsRef<str>) {
        for _ in 0..self.depth {
            self.out.push_str("  ");
        }
        self.out.push_str(s.as_ref());
        self.out.push('\n');
    }

    fn name(&self, s: Sym) -> &str {
        self.d.name(s)
    }

    fn ty(&self, t: super::types::TypeId) -> String {
        match self.d.types.get(t) {
            Type::Class(c) => self.name(self.d.class(*c).name).to_string(),
            Type::Unpacked { elem, .. } => {
                // Unpacked dimensions after the element type.
                let mut dims = String::new();
                let mut cur = t;
                while let Type::Unpacked { elem: e, dim } = self.d.types.get(cur) {
                    dims += &match dim {
                        super::types::UDim::Range(l, r) => format!("[{l}:{r}]"),
                        super::types::UDim::Dynamic => "[]".into(),
                        super::types::UDim::Queue(None) => "[$]".into(),
                        super::types::UDim::Queue(Some(n)) => format!("[$:{n}]"),
                        super::types::UDim::Wildcard => "[*]".into(),
                        super::types::UDim::Assoc(k) => format!("[{}]", self.ty(*k)),
                    };
                    cur = *e;
                }
                let _ = elem;
                format!("{} {dims}", self.ty(cur))
            }
            _ => self.d.types.describe(t, &self.d.syms),
        }
    }

    fn design(&mut self) {
        let unit = self.d.unit;
        if !self.d.scope(unit).items.is_empty() {
            self.line("unit {");
            self.depth += 1;
            self.items(unit);
            self.depth -= 1;
            self.line("}");
        }
        for &p in &self.d.packages {
            let name = self.d.scope(p).name.map_or("", |s| self.name(s)).to_string();
            self.line(format!("package {name} {{"));
            self.depth += 1;
            self.items(p);
            self.depth -= 1;
            self.line("}");
        }
        for (i, spec) in self.d.specs.iter().enumerate() {
            let kind = match spec.kind {
                DefKind::Module => "module",
                DefKind::Interface => "interface",
                DefKind::Program => "program",
            };
            let params: Vec<String> =
                spec.params.iter().map(|(s, c)| format!("{} = {}", self.name(*s), self.konst(*c))).collect();
            let params = if params.is_empty() { String::new() } else { format!(" #({})", params.join(", ")) };
            let n = spec.instances;
            self.line(format!(
                "{kind} {}#{i}{params} {{  // {n} instance{}",
                self.name(spec.name),
                if n == 1 { "" } else { "s" }
            ));
            self.depth += 1;
            let ports: Vec<String> = spec
                .ports
                .iter()
                .map(|p| match p.interface {
                    Some((i, m)) => format!(
                        "{}: {}{}",
                        self.name(p.name),
                        self.name(i),
                        m.map_or(String::new(), |m| format!(".{}", self.name(m)))
                    ),
                    None => format!("{} {}", dir(p.dir), self.name(p.name)),
                })
                .collect();
            if !ports.is_empty() {
                self.line(format!("ports ({})", ports.join(", ")));
            }
            self.items(spec.scope);
            self.depth -= 1;
            self.line("}");
        }
        if !self.d.tops.is_empty() {
            self.line("hierarchy {");
            self.depth += 1;
            for &t in &self.d.tops.clone() {
                self.hierarchy(t);
            }
            self.depth -= 1;
            self.line("}");
        }
    }

    fn hierarchy(&mut self, i: InstId) {
        let inst = self.d.instance(i);
        let spec = self.d.spec(inst.spec);
        let idx = inst.index.map_or(String::new(), |k| format!("[{k}]"));
        self.line(format!("{}{idx}: {}#{}", self.name(inst.name), self.name(spec.name), inst.spec.0));
        self.depth += 1;
        let scope = spec.scope;
        self.child_instances(scope);
        self.depth -= 1;
    }

    fn child_instances(&mut self, scope: ScopeId) {
        for item in self.d.scope(scope).items.clone() {
            match item {
                Item::Instance(c) => self.hierarchy(c),
                Item::Scope(s) => {
                    let sc = self.d.scope(s);
                    if has_instances(self.d, s) {
                        let idx = sc.index.map_or(String::new(), |k| format!("[{k}]"));
                        self.line(format!("{}{idx}:", sc.name.map_or("", |n| self.name(n))));
                        self.depth += 1;
                        self.child_instances(s);
                        self.depth -= 1;
                    }
                }
                _ => {}
            }
        }
    }

    fn konst(&self, c: ConstId) -> String {
        konst(self.d, self.d.konst(c))
    }

    fn items(&mut self, scope: ScopeId) {
        for item in self.d.scope(scope).items.clone() {
            self.item(item);
        }
    }

    fn item(&mut self, item: Item) {
        match item {
            Item::Object(o) => {
                let obj = self.d.object(o);
                let kind = match obj.kind {
                    ObjKind::Variable => "var".to_string(),
                    ObjKind::Net(k) => self.name(k).to_string(),
                    ObjKind::Param { local: true } => "localparam".into(),
                    ObjKind::Param { local: false } => "parameter".into(),
                    ObjKind::Genvar => "genvar".into(),
                    ObjKind::EnumConst => "enumconst".into(),
                };
                let dir = obj.dir.map_or(String::new(), |d| format!("{} ", dir(d)));
                let auto = if obj.lifetime == Lifetime::Automatic { "automatic " } else { "" };
                let quals: Vec<&str> = obj
                    .qualifiers
                    .iter()
                    .map(|&q| self.name(q))
                    .filter(|q| *q != "automatic" && *q != "static")
                    .collect();
                let quals = if quals.is_empty() { String::new() } else { format!("{} ", quals.join(" ")) };
                let mut s = format!("{dir}{quals}{auto}{kind} {}: {}", self.name(obj.name), self.ty(obj.ty));
                if let Some(c) = obj.value {
                    s += &format!(" = {}", self.konst(c));
                } else if let Some(i) = obj.init {
                    s += &format!(" = {}", self.expr(i));
                }
                self.line(s);
            }
            Item::Typedef(s, t) => {
                let line = format!("typedef {} = {}", self.name(s), self.d.types.describe(t, &self.d.syms));
                let full = match self.d.types.get(t) {
                    Type::Struct { .. } | Type::Enum { .. } => {
                        let inner = match self.d.types.get(t).clone() {
                            Type::Struct { packed, union, members, signed, .. } => {
                                let mut x = if union { "union" } else { "struct" }.to_string();
                                if packed {
                                    x += " packed";
                                }
                                if signed {
                                    x += " signed";
                                }
                                let ms: Vec<String> = members
                                    .iter()
                                    .map(|m| format!("{}: {}", self.name(m.name), self.ty(m.ty)))
                                    .collect();
                                format!("{x} {{ {} }}", ms.join("; "))
                            }
                            Type::Enum { base, items, .. } => {
                                let is: Vec<String> =
                                    items.iter().map(|(n, v)| format!("{} = {}", self.name(*n), v)).collect();
                                format!("enum {} {{ {} }}", self.ty(base), is.join(", "))
                            }
                            _ => unreachable!(),
                        };
                        format!("typedef {} = {inner}", self.name(s))
                    }
                    _ => line,
                };
                self.line(full);
            }
            Item::Process(p) => {
                let pr = &self.d.processes[p.0 as usize];
                let kind = match pr.kind {
                    ProcKind::Initial => "initial",
                    ProcKind::Always => "always",
                    ProcKind::AlwaysComb => "always_comb",
                    ProcKind::AlwaysLatch => "always_latch",
                    ProcKind::AlwaysFf => "always_ff",
                    ProcKind::Final => "final",
                };
                let region = if pr.region == RegionSet::Reactive { " reactive" } else { "" };
                let names =
                    |v: &[ObjId]| v.iter().map(|&o| self.name(self.d.object(o).name)).collect::<Vec<_>>().join(", ");
                let mut head = format!("{kind}{region}");
                if !pr.reads.is_empty() {
                    head += &format!(" reads({})", names(&pr.reads));
                }
                if !pr.writes.is_empty() {
                    head += &format!(" writes({})", names(&pr.writes));
                }
                if !pr.sensitivity.is_empty() {
                    head += &format!(" sensitive({})", names(&pr.sensitivity));
                }
                self.line(head);
                self.depth += 1;
                self.stmt(pr.body);
                self.depth -= 1;
            }
            Item::Driver(d) => {
                let dr = &self.d.drivers[d.0 as usize];
                let delay = if dr.delay.is_empty() {
                    String::new()
                } else {
                    format!("#({}) ", dr.delay.iter().map(|&x| self.expr(x)).collect::<Vec<_>>().join(", "))
                };
                let strength =
                    dr.strength.map_or(String::new(), |(a, b)| format!("({}, {}) ", self.name(a), self.name(b)));
                self.line(format!("assign {strength}{delay}{} = {}", self.expr(dr.lhs), self.expr(dr.rhs)));
            }
            Item::Cell(c) => {
                let cell = &self.d.cells[c.0 as usize];
                let terms: Vec<String> = cell.terminals.iter().map(|&x| self.expr(x)).collect();
                let name = cell.name.map_or(String::new(), |n| format!(" {}", self.name(n)));
                let delay = if cell.delay.is_empty() {
                    String::new()
                } else {
                    format!(" #({})", cell.delay.iter().map(|&x| self.expr(x)).collect::<Vec<_>>().join(", "))
                };
                self.line(format!("cell {}{delay}{name}({})", self.name(cell.kind), terms.join(", ")));
            }
            Item::Instance(i) => {
                let inst = self.d.instance(i);
                let spec = self.d.spec(inst.spec);
                let conns: Vec<String> = inst
                    .connections
                    .iter()
                    .map(|c| {
                        let v = match (c.expr, c.interface) {
                            (Some(x), _) => self.expr(x),
                            (None, Some(i)) => self.name(self.d.instance(i).name).to_string(),
                            _ => String::new(),
                        };
                        format!(".{}({v})", self.name(c.port))
                    })
                    .collect();
                let idx = inst.index.map_or(String::new(), |k| format!("[{k}]"));
                self.line(format!(
                    "instance {}{idx}: {}#{} ({})",
                    self.name(inst.name),
                    self.name(spec.name),
                    inst.spec.0,
                    conns.join(", ")
                ));
            }
            Item::Subroutine(s) => {
                let sub = self.d.sub(s);
                let kind = if sub.kind == SubKind::Task { "task" } else { "function" };
                let args: Vec<String> = sub
                    .args
                    .iter()
                    .map(|&a| {
                        let o = self.d.object(a);
                        format!("{} {}: {}", dir(o.dir.unwrap_or(Dir::Input)), self.name(o.name), self.ty(o.ty))
                    })
                    .collect();
                let ret = if sub.kind == SubKind::Function && sub.ret != Types::VOID {
                    format!(" -> {}", self.ty(sub.ret))
                } else {
                    String::new()
                };
                let quals: Vec<&str> =
                    sub.qualifiers.iter().map(|&q| self.name(q)).filter(|q| *q != "automatic" && *q != "static").collect();
                let quals = if quals.is_empty() { String::new() } else { format!("{} ", quals.join(" ")) };
                let auto = if sub.lifetime == Lifetime::Automatic { "automatic " } else { "" };
                self.line(format!("{quals}{auto}{kind} {}({}){ret}", self.name(sub.name), args.join(", ")));
                self.depth += 1;
                let locals: Vec<Item> = self.d.scope(sub.scope).items.clone();
                for it in locals {
                    self.item(it);
                }
                for &st in &sub.body.clone() {
                    self.stmt(st);
                }
                self.depth -= 1;
            }
            Item::Class(c) => {
                let class = self.d.class(c);
                let base =
                    class.base.map_or(String::new(), |b| format!(" extends {}", self.name(self.d.class(b).name)));
                let kw = if class.interface {
                    "interface class"
                } else if class.virtual_ {
                    "virtual class"
                } else {
                    "class"
                };
                self.line(format!("{kw} {}{base} {{", self.name(class.name)));
                self.depth += 1;
                self.items(class.scope);
                self.depth -= 1;
                self.line("}");
            }
            Item::Scope(s) => {
                let sc = self.d.scope(s);
                let idx = sc.index.map_or(String::new(), |k| format!("[{k}]"));
                let name = sc.name.map_or("", |n| self.name(n)).to_string();
                self.line(format!("generate {name}{idx} {{"));
                self.depth += 1;
                self.items(s);
                self.depth -= 1;
                self.line("}");
            }
            Item::Import(p, s) => {
                let pn = self.d.scope(p).name.map_or("", |n| self.name(n)).to_string();
                self.line(format!("import {pn}::{}", s.map_or("*", |s| self.name(s))));
            }
            Item::Opaque(n) => {
                let _ = n;
                self.line("opaque");
            }
        }
    }

    fn stmt(&mut self, s: StmtId) {
        let st = self.d.stmt(s).kind.clone();
        match st {
            StmtKind::Null => self.line(";"),
            StmtKind::Block { join, scope, stmts } => {
                let name =
                    scope.and_then(|s| self.d.scope(s).name).map_or(String::new(), |n| format!(" : {}", self.name(n)));
                let (open, close) = match join {
                    None => ("begin", "end".to_string()),
                    Some(j) => (
                        "fork",
                        match j {
                            Join::All => "join".into(),
                            Join::Any => "join_any".into(),
                            Join::None => "join_none".into(),
                        },
                    ),
                };
                self.line(format!("{open}{name}"));
                self.depth += 1;
                if let Some(sc) = scope {
                    for it in self.d.scope(sc).items.clone() {
                        self.item(it);
                    }
                }
                for x in stmts {
                    self.stmt(x);
                }
                self.depth -= 1;
                self.line(close);
            }
            StmtKind::Assign { kind, lhs, rhs, timing } => {
                let op = match kind {
                    AssignKind::Blocking => "=".to_string(),
                    AssignKind::NonBlocking => "<=".to_string(),
                    AssignKind::Compound(b) => format!("{}=", b.text()),
                };
                let t = timing.map_or(String::new(), |t| format!("{} ", self.timing(&t)));
                self.line(format!("{} {op} {t}{};", self.expr(lhs), self.expr(rhs)));
            }
            StmtKind::Expr(x) => self.line(format!("{};", self.expr(x))),
            StmtKind::If { check, cond, then, else_ } => {
                self.line(format!("{}if ({})", check_text(check), self.expr(cond)));
                self.depth += 1;
                self.stmt(then);
                self.depth -= 1;
                if let Some(e) = else_ {
                    self.line("else");
                    self.depth += 1;
                    self.stmt(e);
                    self.depth -= 1;
                }
            }
            StmtKind::Case { check, kind, expr, items, default } => {
                let kw = match kind {
                    CaseKind::Case => "case",
                    CaseKind::CaseZ => "casez",
                    CaseKind::CaseX => "casex",
                    CaseKind::Inside => "case inside",
                };
                self.line(format!("{}{kw} ({})", check_text(check), self.expr(expr)));
                self.depth += 1;
                for (xs, s) in items {
                    let labels: Vec<String> = xs.iter().map(|&x| self.expr(x)).collect();
                    self.line(format!("{}:", labels.join(", ")));
                    self.depth += 1;
                    self.stmt(s);
                    self.depth -= 1;
                }
                if let Some(d) = default {
                    self.line("default:");
                    self.depth += 1;
                    self.stmt(d);
                    self.depth -= 1;
                }
                self.depth -= 1;
                self.line("endcase");
            }
            StmtKind::Forever(b) => {
                self.line("forever");
                self.indented(b);
            }
            StmtKind::Repeat(x, b) => {
                self.line(format!("repeat ({})", self.expr(x)));
                self.indented(b);
            }
            StmtKind::While(x, b) => {
                self.line(format!("while ({})", self.expr(x)));
                self.indented(b);
            }
            StmtKind::DoWhile(b, x) => {
                self.line("do");
                self.indented(b);
                self.line(format!("while ({});", self.expr(x)));
            }
            StmtKind::For { scope, init, cond, step, body } => {
                if let Some(sc) = scope {
                    for it in self.d.scope(sc).items.clone() {
                        self.item(it);
                    }
                }
                let init: Vec<String> = init.iter().map(|&s| self.inline(s)).collect();
                let step: Vec<String> = step.iter().map(|&s| self.inline(s)).collect();
                let cond = cond.map_or(String::new(), |c| self.expr(c));
                self.line(format!("for ({}; {cond}; {})", init.join(", "), step.join(", ")));
                self.indented(body);
            }
            StmtKind::Foreach { array, vars, body, .. } => {
                let vs: Vec<String> = vars
                    .iter()
                    .map(|v| v.map_or(String::new(), |o| self.name(self.d.object(o).name).to_string()))
                    .collect();
                self.line(format!("foreach ({}[{}])", self.expr(array), vs.join(", ")));
                self.indented(body);
            }
            StmtKind::Timed(t, b) => {
                self.line(self.timing(&t));
                self.indented(b);
            }
            StmtKind::Wait(x, b) => {
                self.line(format!("wait ({})", self.expr(x)));
                self.indented(b);
            }
            StmtKind::WaitFork => self.line("wait fork;"),
            StmtKind::Return(x) => {
                self.line(format!("return{};", x.map_or(String::new(), |x| format!(" {}", self.expr(x)))))
            }
            StmtKind::Break => self.line("break;"),
            StmtKind::Continue => self.line("continue;"),
            StmtKind::Disable(x) => self.line(format!("disable {};", self.expr(x))),
            StmtKind::DisableFork => self.line("disable fork;"),
            StmtKind::Trigger { nonblocking, event } => {
                self.line(format!("{} {};", if nonblocking { "->>" } else { "->" }, self.expr(event)))
            }
            StmtKind::ProcContinuous { keyword, lhs, rhs } => {
                let r = rhs.map_or(String::new(), |r| format!(" = {}", self.expr(r)));
                self.line(format!("{} {}{r};", self.name(keyword), self.expr(lhs)));
            }
            StmtKind::Assert { keyword, cond, pass, fail } => {
                self.line(format!("{} ({})", self.name(keyword), self.expr(cond)));
                if let Some(p) = pass {
                    self.indented(p);
                }
                if let Some(f) = fail {
                    self.line("else");
                    self.indented(f);
                }
            }
            StmtKind::Opaque => self.line("opaque;"),
        }
    }

    fn indented(&mut self, s: StmtId) {
        self.depth += 1;
        self.stmt(s);
        self.depth -= 1;
    }

    /// A simple statement on one line, for `for` headers.
    fn inline(&self, s: StmtId) -> String {
        match self.d.stmt(s).kind.clone() {
            StmtKind::Assign { kind, lhs, rhs, .. } => {
                let op = match kind {
                    AssignKind::Blocking => "=".to_string(),
                    AssignKind::NonBlocking => "<=".to_string(),
                    AssignKind::Compound(b) => format!("{}=", b.text()),
                };
                format!("{} {op} {}", self.expr(lhs), self.expr(rhs))
            }
            StmtKind::Expr(x) => self.expr(x),
            _ => "...".into(),
        }
    }

    fn timing(&self, t: &Timing) -> String {
        let terms = |ts: &[EventTerm]| -> String {
            ts.iter()
                .map(|t| {
                    let edge = match t.edge {
                        Edge::Any => "",
                        Edge::Pos => "posedge ",
                        Edge::Neg => "negedge ",
                        Edge::Both => "edge ",
                    };
                    let iff = t.iff.map_or(String::new(), |i| format!(" iff {}", self.expr(i)));
                    format!("{edge}{}{iff}", self.expr(t.expr))
                })
                .collect::<Vec<_>>()
                .join(" or ")
        };
        match t {
            Timing::Delay(xs) => format!("#({})", xs.iter().map(|&x| self.expr(x)).collect::<Vec<_>>().join(", ")),
            Timing::Event(ts) => format!("@({})", terms(ts)),
            Timing::Implicit(ts) => {
                if ts.is_empty() {
                    "@*".into()
                } else {
                    format!("@*({})", terms(ts))
                }
            }
            Timing::Cycle(x) => format!("##{}", self.expr(*x)),
            Timing::Repeat(x, ts) => format!("repeat ({}) @({})", self.expr(*x), terms(ts)),
        }
    }

    fn expr(&self, x: ExprId) -> String {
        expr(self.d, x)
    }
}

fn has_instances(d: &Design, s: ScopeId) -> bool {
    d.scope(s).items.iter().any(|i| match i {
        Item::Instance(_) => true,
        Item::Scope(c) => has_instances(d, *c),
        _ => false,
    })
}

fn check_text(c: Check) -> &'static str {
    match c {
        Check::None => "",
        Check::Unique => "unique ",
        Check::Unique0 => "unique0 ",
        Check::Priority => "priority ",
    }
}

fn dir(d: Dir) -> &'static str {
    match d {
        Dir::Input => "input",
        Dir::Output => "output",
        Dir::Inout => "inout",
        Dir::Ref => "ref",
        Dir::ConstRef => "const ref",
    }
}

pub fn konst(d: &Design, c: &Const) -> String {
    match c {
        Const::Bits(b) => b.to_string(),
        Const::Real(r) => format!("{r:?}"),
        Const::Str(s) => format!("{:?}", String::from_utf8_lossy(s)),
        Const::Aggregate(v) => format!("'{{{}}}", v.iter().map(|c| konst(d, c)).collect::<Vec<_>>().join(", ")),
        Const::Type(t) => d.types.describe(*t, &d.syms),
        Const::Null => "null".into(),
        Const::Unbounded => "$".into(),
    }
}

/// An expression in SystemVerilog's syntax, with conversions written out.
pub fn expr(d: &Design, x: ExprId) -> String {
    let e = d.expr(x);
    let a = |i: usize| expr(d, e.args[i]);
    let all = |from: usize| e.args[from.min(e.args.len())..].iter().map(|&y| expr(d, y)).collect::<Vec<_>>().join(", ");
    match &e.kind {
        ExprKind::Const(c) => konst(d, d.konst(*c)),
        ExprKind::Object(o) => d.name(d.object(*o).name).to_string(),
        ExprKind::Hier { path, .. } => path.iter().map(|&s| d.name(s)).collect::<Vec<_>>().join("."),
        ExprKind::Unary(op) => {
            let s = match op {
                UnOp::Plus => "+",
                UnOp::Neg => "-",
                UnOp::Not => "~",
                UnOp::LogNot => "!",
                UnOp::RedAnd => "&",
                UnOp::RedNand => "~&",
                UnOp::RedOr => "|",
                UnOp::RedNor => "~|",
                UnOp::RedXor => "^",
                UnOp::RedXnor => "~^",
                UnOp::PreInc => return format!("++{}", a(0)),
                UnOp::PreDec => return format!("--{}", a(0)),
                UnOp::PostInc => return format!("{}++", a(0)),
                UnOp::PostDec => return format!("{}--", a(0)),
            };
            format!("{s}{}", a(0))
        }
        ExprKind::Binary(op) => format!("({} {} {})", a(0), op.text(), a(1)),
        ExprKind::Cond => format!("({} ? {} : {})", a(0), a(1), a(2)),
        ExprKind::Concat => format!("{{{}}}", all(0)),
        ExprKind::Replicate(n) => format!("{{{n}{}}}", a(0)),
        ExprKind::Index => format!("{}[{}]", a(0), a(1)),
        ExprKind::Part { kind, width } => match kind {
            PartKind::Range => format!("{}[{}:{}]", a(0), a(1), a(2)),
            PartKind::Up => format!("{}[{}+:{width}]", a(0), a(1)),
            PartKind::Down => format!("{}[{}-:{width}]", a(0), a(1)),
        },
        ExprKind::Member(i) => {
            let bt = d.expr(e.args[0]).ty;
            let name = match d.types.get(bt) {
                Type::Struct { members, .. } => members.get(*i as usize).map_or("?", |m| d.name(m.name)),
                _ => "?",
            };
            format!("{}.{name}", a(0))
        }
        ExprKind::Field(o) => {
            let n = d.name(d.object(*o).name);
            if e.args.is_empty() { n.to_string() } else { format!("{}.{n}", a(0)) }
        }
        ExprKind::Convert => {
            let from = d.expr(e.args[0]).ty;
            let to = e.ty;
            let t = &d.types;
            if t.is_integral(from) && t.is_integral(to) {
                let (wf, wt) = (t.width(from), t.width(to));
                if wt > wf {
                    let k = if t.is_signed(to) { "sext" } else { "zext" };
                    let sign = if t.is_signed(to) != t.is_signed(from) {
                        if t.is_signed(to) { " signed" } else { " unsigned" }
                    } else {
                        ""
                    };
                    return format!("{k}<{wt}{sign}>({})", a(0));
                }
                if wt < wf {
                    return format!("trunc<{wt}>({})", a(0));
                }
            }
            let ts = match t.get(to) {
                Type::Class(c) => d.name(d.class(*c).name).to_string(),
                _ => t.describe(to, &d.syms),
            };
            format!("cvt<{ts}>({})", a(0))
        }
        ExprKind::Call(s) => {
            // A method's object comes first among the arguments.
            let sub = d.sub(*s);
            if e.args.len() == sub.args.len() + 1 {
                format!("{}.{}({})", a(0), d.name(sub.name), all(1))
            } else {
                format!("{}({})", d.name(sub.name), all(0))
            }
        }
        ExprKind::SysCall(s) => {
            if e.args.is_empty() {
                d.name(*s).to_string()
            } else {
                format!("{}({})", d.name(*s), all(0))
            }
        }
        ExprKind::Method(m) => format!("{}.{}({})", a(0), d.name(*m), all(1)),
        ExprKind::Pattern => format!("'{{{}}}", all(0)),
        ExprKind::Inside => format!("({} inside {{{}}})", a(0), all(1)),
        ExprKind::Range => format!("[{}:{}]", a(0), a(1)),
        ExprKind::Streaming { left } => format!("{{{} {{{}}}}}", if *left { "<<" } else { ">>" }, all(0)),
        ExprKind::New => {
            // A class's constructor call carries the arguments.
            match e.args.first().map(|&c| &d.expr(c).kind) {
                Some(ExprKind::Call(s)) if d.name(d.sub(*s).name) == "new" => {
                    let c = d.expr(e.args[0]);
                    let args: Vec<String> = c.args.iter().map(|&y| expr(d, y)).collect();
                    format!("new({})", args.join(", "))
                }
                None => "new".into(),
                _ => format!("new({})", all(0)),
            }
        }
        ExprKind::Assign => format!("({} = {})", a(0), a(1)),
        ExprKind::Dollar => "$".into(),
        ExprKind::Null => "null".into(),
        ExprKind::This => "this".into(),
        ExprKind::TypeRef(t) => format!("type({})", d.types.describe(*t, &d.syms)),
        ExprKind::Tagged(_) => format!("tagged({})", all(0)),
        ExprKind::Opaque => {
            if e.args.is_empty() {
                "<opaque>".into()
            } else {
                format!("<opaque>({})", all(0))
            }
        }
    }
}

/// `$sformatf`'s formatting, for constant evaluation (21.2.1).
pub fn format(vals: &[Const]) -> String {
    let Some(Const::Str(fmt)) = vals.first() else {
        return String::new();
    };
    let fmt = String::from_utf8_lossy(fmt).to_string();
    let mut out = String::new();
    let mut args = vals[1..].iter();
    let mut chars = fmt.chars().peekable();
    while let Some(c) = chars.next() {
        if c != '%' {
            out.push(c);
            continue;
        }
        let mut spec = String::new();
        while let Some(&d) = chars.peek() {
            if d.is_ascii_digit() || d == '.' || d == '-' {
                spec.push(d);
                chars.next();
            } else {
                break;
            }
        }
        let Some(conv) = chars.next() else { break };
        if conv == '%' {
            out.push('%');
            continue;
        }
        let v = args.next();
        let s = match (conv.to_ascii_lowercase(), v) {
            ('d', Some(Const::Bits(b))) => b.to_decimal(),
            ('d', Some(Const::Real(r))) => format!("{}", r.round() as i64),
            ('h' | 'x', Some(Const::Bits(b))) => b.to_radix(4),
            ('b', Some(Const::Bits(b))) => b.to_radix(1),
            ('o', Some(Const::Bits(b))) => b.to_radix(3),
            ('s', Some(Const::Str(s))) => String::from_utf8_lossy(s).to_string(),
            ('s', Some(Const::Bits(b))) => String::from_utf8_lossy(&b.to_str_bytes()).to_string(),
            ('c', Some(Const::Bits(b))) => (b.to_u64().unwrap_or(0) as u8 as char).to_string(),
            ('f', Some(Const::Real(r))) => format!("{r:.6}"),
            ('e', Some(Const::Real(r))) => format!("{r:e}"),
            ('g', Some(Const::Real(r))) => format!("{r}"),
            (_, Some(c)) => format!("{c:?}"),
            (_, None) => String::new(),
        };
        let _ = write!(out, "{s}");
    }
    out
}
