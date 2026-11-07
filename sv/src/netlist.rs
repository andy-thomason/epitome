//! The compiled netlist (docs/design/sv.md sections 8.1 and 8.3), first
//! cut: combinational specialisations, two-state, every value in 32-bit
//! lanes.
//!
//! Each specialisation becomes one function, `sv_eval_<n>(ptr %state)`,
//! whose argument holds the state of a group of [`GROUP`] instances: for
//! each variable or net, its 32-bit pieces (low piece first), each piece
//! [`CHUNKS`] registers of [`LANES`] lanes. The function is a loop over the
//! group's registers, so its code grows with the specialisation's source,
//! not with the number of instances. A child instance's state is part of
//! its parent's, and the parent calls the child's function once for the
//! whole group, between two loops: one that works out the child's inputs,
//! and one that reads its outputs.
//!
//! The output is Epitome IR as text, `<16 x i32>` vectors and `<16 x i1>`
//! masks, for `epitome-opt` and `epitome-codegen`.
//!
//! Not yet: clocked processes, four-state values, unpacked arrays, function
//! calls, hierarchical references, gates, and loops whose trip count is not
//! a constant. Generate loops arrive unrolled from the elaborator and stay
//! unrolled; procedural `for` loops are unrolled here. Immediate assertions
//! and `initial` blocks that only assert are dropped.

use std::cell::{Cell, RefCell};
use std::collections::{BinaryHeap, HashMap, HashSet};
use std::fmt::Write;

use crate::design::types::{Type, TypeId};
use crate::design::value::{Bits, Logic};
use crate::design::{
    AssignKind, BinOp, CaseKind, Const, Design, Dir, DriverId, ExprId, ExprKind, InstId, Item, Lifetime, ObjId, ObjKind,
    PartKind, ProcId, ProcKind, ScopeId, ScopeKind, SpecId, StmtId, StmtKind, UnOp, ops,
};

/// Instances in one register: 32-bit lanes of a 512-bit vector.
pub const LANES: usize = 16;
/// Registers in a group.
pub const CHUNKS: usize = 32;
/// Instances in a group.
pub const GROUP: usize = LANES * CHUNKS;
/// Bytes in one register of lanes.
const REG: u64 = 64;
/// Bytes in one piece of a value for the whole group.
const PIECE: u64 = REG * CHUNKS as u64;

/// The compiled netlist of one top-level instance.
pub struct Netlist {
    /// The functions, as Epitome IR text.
    pub ir: String,
    /// The top specialisation's function.
    pub entry: String,
    /// Bytes of state for one group.
    pub size: usize,
    /// The top's ports, where they are in the state.
    pub ports: Vec<Field>,
    /// Assertions and assertion-only `initial` blocks left out.
    pub dropped: usize,
}

/// Where a value is in a group's state.
#[derive(Clone, Debug)]
pub struct Field {
    pub name: String,
    pub dir: Dir,
    pub offset: usize,
    pub width: u32,
}

impl Field {
    /// The byte offset of `piece` (bits `32 * piece` up) of instance `n`.
    pub fn at(&self, piece: usize, n: usize) -> usize {
        self.offset + piece * PIECE as usize + (n / LANES) * REG as usize + (n % LANES) * 4
    }

    pub fn pieces(&self) -> usize {
        pieces(self.width)
    }
}

impl Netlist {
    pub fn port(&self, name: &str) -> Option<&Field> {
        self.ports.iter().find(|p| p.name == name)
    }
}

fn pieces(w: u32) -> usize {
    (w as usize).div_ceil(32).max(1)
}

/// The bits of the top piece of a `w`-bit value.
fn top_mask(w: u32) -> u32 {
    if w % 32 == 0 { !0 } else { (1u32 << (w % 32)) - 1 }
}

/// Choices in how the netlist is laid out.
#[derive(Clone, Copy, Debug)]
pub struct Options {
    /// One-bit elements as bit planes, a 16-bit mask for each register of
    /// lanes: for targets with mask registers (AVX-512, SVE). Otherwise
    /// they take a 32-bit lane each.
    pub bit_planes: bool,
}

impl Default for Options {
    fn default() -> Self {
        Options { bit_planes: true }
    }
}

/// Lowers the instance `top` and everything below it.
pub fn compile(d: &Design, top: InstId) -> Result<Netlist, String> {
    compile_with(d, top, &Options::default())
}

pub fn compile_with(d: &Design, top: InstId, opts: &Options) -> Result<Netlist, String> {
    let mut n = Lowering {
        opts: *opts,
        d,
        layouts: HashMap::new(),
        done: HashSet::new(),
        ir: String::new(),
        folds: RefCell::default(),
        dropped: Cell::new(0),
        asserts: RefCell::default(),
        loops: Vec::new(),
        loop_vars: RefCell::default(),
    };
    let spec = d.instance(top).spec;
    n.layout(spec)?;
    n.ir.push_str("declare i32 @llvm.umin.i32(i32, i32)\n\n");
    n.function(spec)?;
    let lay = &n.layouts[&spec];
    let mut ports = Vec::new();
    for p in &d.spec(spec).ports {
        let o = p.object.ok_or_else(|| format!("interface port {} is not supported yet", d.name(p.name)))?;
        ports.push(Field {
            name: d.name(p.name).to_string(),
            dir: p.dir,
            offset: lay.objs[&o] as usize,
            width: d.types.width(d.object(o).ty),
        });
    }
    let dropped = n.dropped.get() + n.asserts.borrow().len();
    Ok(Netlist { ir: n.ir, entry: fn_name(spec), size: lay.size as usize, ports, dropped })
}

fn fn_name(s: SpecId) -> String {
    format!("sv_eval_{}", s.0)
}

/// Where a specialisation's values and children are in its state.
struct Layout {
    size: u64,
    objs: HashMap<ObjId, u64>,
    /// The objects kept an element to a slot (docs/design/sv.md section
    /// 8.3); the rest are 32-bit pieces.
    elems: HashMap<ObjId, Elems>,
    kids: HashMap<InstId, u64>,
    /// Its processes, drivers and children, in source order.
    code: Vec<Node>,
}

/// An object used only an element at a time, kept an element to a slot:
/// one-bit elements as bit planes, a 16-bit mask for each register of
/// lanes (a k register with AVX-512); wider ones (up to 32 bits) in a
/// register of 32-bit lanes each. Selecting an element is then choosing
/// an address, with no shifts.
#[derive(Clone, Copy, Debug)]
struct Elems {
    /// The element's width.
    ew: u32,
    /// How many elements.
    n: u32,
    /// One-bit elements as bit planes.
    bits: bool,
}

impl Elems {
    fn planes(self) -> bool {
        self.bits
    }

    /// Bytes from one element's slot to the next.
    fn stride(self) -> u64 {
        if self.planes() { 2 * CHUNKS as u64 } else { PIECE }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
enum Node {
    Proc(ProcId),
    Driver(DriverId),
    /// A child's inputs, worked out in the parent.
    In(InstId),
    /// The call of a child's function, then its outputs.
    Call(InstId),
    /// A run of a generate loop's iterations, rolled back into a loop.
    Loop(usize),
}

/// Iterations of a generate loop with the same code, which differ only in
/// constants that step by the same amount each time (the genvar, and
/// what is worked out from it): run as a loop over `count` iterations.
struct Rolled {
    count: u32,
    /// The first iteration's continuous assignments, the loop's body.
    body: Vec<DriverId>,
    /// The first iteration's parameters that step: the parameter, its
    /// first value and its step.
    steps: Vec<(ObjId, i64, i64)>,
    /// Every iteration's continuous assignments, for what the loop reads
    /// and writes.
    all: Vec<DriverId>,
}

/// Fewest iterations worth a loop.
const ROLL_MIN: usize = 4;

struct Lowering<'d, 'a> {
    opts: Options,
    d: &'d Design<'a>,
    layouts: HashMap<SpecId, Layout>,
    done: HashSet<SpecId>,
    ir: String,
    folds: RefCell<HashMap<ExprId, Option<Const>>>,
    /// `initial` blocks and kept syntax left out.
    dropped: Cell<usize>,
    /// Immediate assertions left out.
    asserts: RefCell<HashSet<StmtId>>,
    loops: Vec<Rolled>,
    /// The parameters that step in the loop being lowered: not constants
    /// there.
    loop_vars: RefCell<HashSet<ObjId>>,
}

impl<'d, 'a> Lowering<'d, 'a> {
    fn layout(&mut self, spec: SpecId) -> Result<(), String> {
        if self.layouts.contains_key(&spec) {
            return Ok(());
        }
        let d = self.d;
        let mut objs = Vec::new();
        let mut code = Vec::new();
        self.collect(d.spec(spec).scope, &mut objs, &mut code)?;
        let mut lay = Layout { size: 0, objs: HashMap::new(), elems: HashMap::new(), kids: HashMap::new(), code: Vec::new() };
        let elems = self.element_only(&objs, &code);
        for o in objs {
            let ob = d.object(o);
            if !d.types.is_integral(ob.ty) {
                return Err(format!("{}: only integral values are supported yet", d.name(ob.name)));
            }
            lay.objs.insert(o, lay.size);
            match elems.get(&o) {
                Some(&e) => {
                    lay.elems.insert(o, e);
                    lay.size += e.n as u64 * e.stride();
                }
                None => lay.size += pieces(d.types.width(ob.ty)) as u64 * PIECE,
            }
            lay.size = lay.size.next_multiple_of(REG);
        }
        for node in code {
            if let Node::In(i) = node {
                let child = d.instance(i).spec;
                self.layout(child)?;
                lay.kids.insert(i, lay.size);
                lay.size += self.layouts[&child].size;
                lay.code.push(Node::In(i));
                lay.code.push(Node::Call(i));
            } else {
                lay.code.push(node);
            }
        }
        self.layouts.insert(spec, lay);
        Ok(())
    }

    /// The static values and the code of a scope and the generate blocks
    /// in it.
    fn collect(&mut self, scope: ScopeId, objs: &mut Vec<ObjId>, code: &mut Vec<Node>) -> Result<(), String> {
        let d = self.d;
        for item in &d.scope(scope).items {
            match *item {
                Item::Object(o) => {
                    let ob = d.object(o);
                    if matches!(ob.kind, ObjKind::Variable | ObjKind::Net(_)) && ob.lifetime == Lifetime::Static {
                        objs.push(o);
                    }
                }
                Item::Process(p) => {
                    let pr = &d.processes[p.0 as usize];
                    match pr.kind {
                        ProcKind::AlwaysComb => code.push(Node::Proc(p)),
                        ProcKind::Initial if self.only_asserts(pr.body) => self.dropped.set(self.dropped.get() + 1),
                        k => return Err(format!("{k:?} processes are not supported yet")),
                    }
                }
                Item::Driver(dr) => {
                    let drv = &d.drivers[dr.0 as usize];
                    if !drv.delay.is_empty() || drv.strength.is_some() {
                        return Err("continuous assignments with delays or strengths are not supported yet".into());
                    }
                    code.push(Node::Driver(dr));
                }
                Item::Instance(i) => code.push(Node::In(i)),
                Item::Cell(_) => return Err("gates and UDPs are not supported yet".into()),
                Item::Scope(s) => match d.scope(s).kind {
                    ScopeKind::Generate if self.is_generate_loop(s) => self.generate_loop(s, objs, code)?,
                    ScopeKind::Generate | ScopeKind::Block => self.collect(s, objs, code)?,
                    _ => {}
                },
                Item::Opaque(_) => self.dropped.set(self.dropped.get() + 1),
                Item::Typedef(..) | Item::Subroutine(_) | Item::Class(_) | Item::Import(..) => {}
            }
        }
        Ok(())
    }

    /// The objects that every use selects one element of, with the same
    /// element width: these are kept an element to a slot. Ports are not,
    /// so that a parent and a child meet in one layout.
    fn element_only(&self, objs: &[ObjId], code: &[Node]) -> HashMap<ObjId, Elems> {
        let d = self.d;
        let mut uses: HashMap<ObjId, Option<u32>> = HashMap::new();
        for node in code {
            match *node {
                Node::Driver(dr) => {
                    let drv = &d.drivers[dr.0 as usize];
                    self.expr_uses(drv.lhs, &mut uses);
                    self.expr_uses(drv.rhs, &mut uses);
                }
                Node::Loop(l) => {
                    for &dr in &self.loops[l].all {
                        let drv = &d.drivers[dr.0 as usize];
                        self.expr_uses(drv.lhs, &mut uses);
                        self.expr_uses(drv.rhs, &mut uses);
                    }
                }
                Node::Proc(p) => self.stmt_uses(d.processes[p.0 as usize].body, &mut uses),
                Node::In(i) | Node::Call(i) => {
                    for c in &d.instance(i).connections {
                        if let Some(e) = c.expr {
                            self.expr_uses(e, &mut uses);
                        }
                    }
                }
            }
        }
        let mut out = HashMap::new();
        for &o in objs {
            let ob = d.object(o);
            let w = d.types.width(ob.ty);
            if let (None, Some(Some(ew))) = (ob.dir, uses.get(&o))
                && *ew <= 32
                && w % ew == 0
                && w / ew >= 2
                && d.types.packed_range(ob.ty).is_some()
            {
                out.insert(o, Elems { ew: *ew, n: w / ew, bits: *ew == 1 && self.opts.bit_planes });
            }
        }
        out
    }

    /// Notes each object an expression uses: `Some(width)` while every use
    /// selects one element of that width, `None` once one does not.
    fn expr_uses(&self, e: ExprId, uses: &mut HashMap<ObjId, Option<u32>>) {
        let d = self.d;
        let x = d.expr(e);
        let mut note = |o: ObjId, u: Option<u32>| {
            let slot = uses.entry(o).or_insert(u);
            if *slot != u {
                *slot = None;
            }
        };
        match &x.kind {
            ExprKind::Object(o) => note(*o, None),
            ExprKind::Index if matches!(d.expr(x.args[0]).kind, ExprKind::Object(_)) => {
                let ExprKind::Object(o) = d.expr(x.args[0]).kind else { unreachable!() };
                note(o, Some(d.types.width(x.ty)));
                self.expr_uses(x.args[1], uses);
            }
            _ => {
                for &a in &x.args {
                    self.expr_uses(a, uses);
                }
            }
        }
    }

    fn stmt_uses(&self, s: StmtId, uses: &mut HashMap<ObjId, Option<u32>>) {
        let d = self.d;
        let kind = &d.stmt(s).kind;
        if !matches!(kind, StmtKind::Null | StmtKind::Block { .. }) && self.only_asserts(s) {
            return;
        }
        match kind {
            StmtKind::Block { stmts, .. } => stmts.iter().for_each(|&s| self.stmt_uses(s, uses)),
            StmtKind::Assign { lhs, rhs, .. } => {
                self.expr_uses(*lhs, uses);
                self.expr_uses(*rhs, uses);
            }
            StmtKind::Expr(e) => self.expr_uses(*e, uses),
            StmtKind::If { cond, then, else_, .. } => {
                self.expr_uses(*cond, uses);
                self.stmt_uses(*then, uses);
                if let Some(e) = else_ {
                    self.stmt_uses(*e, uses);
                }
            }
            StmtKind::Case { expr, items, default, .. } => {
                self.expr_uses(*expr, uses);
                for (labels, body) in items {
                    labels.iter().for_each(|&l| self.expr_uses(l, uses));
                    self.stmt_uses(*body, uses);
                }
                if let Some(s) = default {
                    self.stmt_uses(*s, uses);
                }
            }
            StmtKind::For { init, cond, step, body, .. } => {
                init.iter().chain(step).for_each(|&s| self.stmt_uses(s, uses));
                if let Some(c) = cond {
                    self.expr_uses(*c, uses);
                }
                self.stmt_uses(*body, uses);
            }
            _ => {}
        }
    }

    /// Whether a generate scope holds a loop's iterations.
    fn is_generate_loop(&self, s: ScopeId) -> bool {
        let d = self.d;
        let mut kids = d.scope(s).items.iter().filter_map(|i| if let Item::Scope(k) = i { Some(*k) } else { None }).peekable();
        kids.peek().is_some() && kids.all(|k| d.scope(k).kind == ScopeKind::Generate && d.scope(k).index.is_some())
    }

    /// A generate loop's iterations: runs of them with the same code become
    /// loops; the rest are as written.
    fn generate_loop(&mut self, s: ScopeId, objs: &mut Vec<ObjId>, code: &mut Vec<Node>) -> Result<(), String> {
        let d = self.d;
        let iters: Vec<ScopeId> = d.scope(s).items.iter().filter_map(|i| if let Item::Scope(k) = i { Some(*k) } else { None }).collect();
        // Runs of iterations of the same shape.
        let shapes: Vec<Option<String>> = iters.iter().map(|&i| self.shape(i)).collect();
        let mut at = 0;
        while at < iters.len() {
            let mut end = at + 1;
            while end < iters.len() && shapes[at].is_some() && shapes[end] == shapes[at] {
                end += 1;
            }
            let rolled = if end - at >= ROLL_MIN { self.roll(&iters[at..end]) } else { None };
            match rolled {
                Some(r) => {
                    code.push(Node::Loop(self.loops.len()));
                    self.loops.push(r);
                }
                None => {
                    for &i in &iters[at..end] {
                        self.collect(i, objs, code)?;
                    }
                }
            }
            at = end;
        }
        Ok(())
    }

    /// An iteration's code, as the source nodes it was made from, if it is
    /// only continuous assignments and parameters (in nested generate
    /// blocks that are not loops).
    fn shape(&self, s: ScopeId) -> Option<String> {
        let d = self.d;
        let mut out = String::new();
        for item in &d.scope(s).items {
            match *item {
                Item::Object(o) if matches!(d.object(o).kind, ObjKind::Param { .. }) => {
                    let _ = write!(out, "p{} ", d.name(d.object(o).name));
                }
                Item::Driver(dr) => {
                    let drv = &d.drivers[dr.0 as usize];
                    if !drv.delay.is_empty() || drv.strength.is_some() {
                        return None;
                    }
                    let _ = write!(out, "d{:?} ", drv.node);
                }
                Item::Scope(k) if d.scope(k).kind == ScopeKind::Generate && !self.is_generate_loop(k) => {
                    let _ = write!(out, "s{:?}({}) ", d.scope(k).node, self.shape(k)?);
                }
                Item::Typedef(..) | Item::Import(..) => {}
                _ => return None,
            }
        }
        Some(out)
    }

    /// An iteration's continuous assignments and parameters, in order.
    fn gather(&self, s: ScopeId, drivers: &mut Vec<DriverId>, params: &mut Vec<ObjId>) {
        let d = self.d;
        for item in &d.scope(s).items {
            match *item {
                Item::Object(o) => params.push(o),
                Item::Driver(dr) => drivers.push(dr),
                Item::Scope(k) => self.gather(k, drivers, params),
                _ => {}
            }
        }
    }

    /// The iterations as one loop, if every one's code is the first's with
    /// parameters that step evenly.
    fn roll(&self, iters: &[ScopeId]) -> Option<Rolled> {
        let d = self.d;
        let mut drivers = Vec::new();
        let mut params = Vec::new();
        for &i in iters {
            let (mut dv, mut pv) = (Vec::new(), Vec::new());
            self.gather(i, &mut dv, &mut pv);
            drivers.push(dv);
            params.push(pv);
        }
        let n = iters.len();
        let mut steps = Vec::new();
        for (k, &p) in params[0].iter().enumerate() {
            let val = |t: usize| d.object(params[t][k]).value.and_then(|c| d.konst(c).to_i64());
            let (a, b) = (val(0)?, val(1)?);
            if (0..n).any(|t| val(t) != Some(a + (b - a) * t as i64)) {
                return None;
            }
            if b != a {
                steps.push((p, a, b - a));
            }
        }
        for t in 1..n {
            let map: HashMap<ObjId, ObjId> = params[t].iter().copied().zip(params[0].iter().copied()).collect();
            for (&x, &y) in drivers[0].iter().zip(&drivers[t]) {
                let (x, y) = (&d.drivers[x.0 as usize], &d.drivers[y.0 as usize]);
                if !self.same_expr(x.lhs, y.lhs, &map) || !self.same_expr(x.rhs, y.rhs, &map) {
                    return None;
                }
            }
        }
        // An iteration must not read what another writes, so that running
        // them in order is running them at once.
        let mut writes: HashMap<ObjId, Vec<(u32, u32, usize)>> = HashMap::new();
        let mut reads = Vec::new();
        for (t, dv) in drivers.iter().enumerate() {
            let mut r = Vec::new();
            for &dr in dv {
                let drv = &d.drivers[dr.0 as usize];
                let mut w = Vec::new();
                self.reads(drv.rhs, &mut r);
                self.writes(drv.lhs, &mut w, &mut r);
                for (o, lo, hi) in w {
                    writes.entry(o).or_default().push((lo, hi, t));
                }
            }
            reads.push(r);
        }
        for (t, r) in reads.iter().enumerate() {
            for &(o, lo, hi) in r {
                if writes.get(&o).is_some_and(|v| v.iter().any(|&(wl, wh, u)| u != t && wl < hi && lo < wh)) {
                    return None;
                }
            }
        }
        Some(Rolled { count: n as u32, body: drivers[0].clone(), steps, all: drivers.concat() })
    }

    /// Whether two expressions are the same, with `map` taking the second's
    /// parameters to the first's.
    fn same_expr(&self, a: ExprId, b: ExprId, map: &HashMap<ObjId, ObjId>) -> bool {
        let d = self.d;
        let (x, y) = (d.expr(a), d.expr(b));
        if x.ty != y.ty || x.args.len() != y.args.len() {
            return false;
        }
        let same = match (&x.kind, &y.kind) {
            (ExprKind::Object(p), ExprKind::Object(q)) => p == q || map.get(q) == Some(p),
            (ExprKind::Const(p), ExprKind::Const(q)) => ops::const_eq(d.konst(*p), d.konst(*q)),
            (k, l) => k == l,
        };
        same && x.args.iter().zip(&y.args).all(|(&p, &q)| self.same_expr(p, q, map))
    }

    /// Whether a statement does nothing but check assertions, so leaving
    /// it out, conditions and all, changes no value. Its assertions are
    /// added to `asserts`.
    fn only_asserts(&self, s: StmtId) -> bool {
        let d = self.d;
        let all = |v: &mut dyn Iterator<Item = StmtId>| v.fold(true, |all, s| self.only_asserts(s) && all);
        match &d.stmt(s).kind {
            StmtKind::Null => true,
            StmtKind::Assert { .. } => {
                self.asserts.borrow_mut().insert(s);
                true
            }
            StmtKind::Block { join: None, stmts, .. } => all(&mut stmts.iter().copied()),
            StmtKind::If { then, else_, .. } => all(&mut std::iter::once(*then).chain(*else_)),
            StmtKind::Case { items, default, .. } => all(&mut items.iter().map(|i| i.1).chain(*default)),
            // The loop variable is the loop's own.
            StmtKind::For { scope: Some(_), body, .. } => self.only_asserts(*body),
            _ => false,
        }
    }

    /// The function for `spec`, after its children's.
    fn function(&mut self, spec: SpecId) -> Result<(), String> {
        if !self.done.insert(spec) {
            return Ok(());
        }
        let kids: Vec<SpecId> = self.layouts[&spec].kids.keys().map(|&i| self.d.instance(i).spec).collect();
        for k in kids {
            self.function(k)?;
        }
        let order = self.order(spec)?;
        let mut f = Func::new(self, spec);
        f.body(&order)?;
        let text = f.finish();
        let _ = writeln!(self.ir, "; {}\n{text}", self.d.name(self.d.spec(spec).name));
        Ok(())
    }

    /// The specialisation's code in an order where everything is worked
    /// out before it is read: a topological sort on the bits each node
    /// reads and writes.
    fn order(&self, spec: SpecId) -> Result<Vec<Node>, String> {
        let lay = &self.layouts[&spec];
        let nodes = &lay.code;
        let mut reads = Vec::with_capacity(nodes.len());
        let mut writers: HashMap<ObjId, Vec<(u32, u32, usize)>> = HashMap::new();
        for (n, node) in nodes.iter().enumerate() {
            let (r, w) = self.access(spec, *node)?;
            for (o, lo, hi) in w {
                writers.entry(o).or_default().push((lo, hi, n));
            }
            reads.push(r);
        }
        let mut succ: Vec<Vec<usize>> = vec![Vec::new(); nodes.len()];
        let mut preds = vec![0usize; nodes.len()];
        for (n, r) in reads.iter().enumerate() {
            let mut from = Vec::new();
            for &(o, lo, hi) in r {
                for &(wlo, whi, w) in writers.get(&o).map(|v| v.as_slice()).unwrap_or(&[]) {
                    if w != n && wlo < hi && lo < whi {
                        from.push(w);
                    }
                }
            }
            if let Node::Call(i) = nodes[n] {
                from.push(nodes.iter().position(|&x| x == Node::In(i)).unwrap());
            }
            from.sort_unstable();
            from.dedup();
            preds[n] = from.len();
            for w in from {
                succ[w].push(n);
            }
        }
        let mut ready: BinaryHeap<std::cmp::Reverse<usize>> =
            (0..nodes.len()).filter(|&n| preds[n] == 0).map(std::cmp::Reverse).collect();
        let mut order = Vec::with_capacity(nodes.len());
        while let Some(std::cmp::Reverse(n)) = ready.pop() {
            order.push(nodes[n]);
            for &s in &succ[n] {
                preds[s] -= 1;
                if preds[s] == 0 {
                    ready.push(std::cmp::Reverse(s));
                }
            }
        }
        if order.len() < nodes.len() {
            let stuck: Vec<String> =
                (0..nodes.len()).filter(|&n| preds[n] > 0).take(4).map(|n| format!("{:?}", nodes[n])).collect();
            return Err(format!(
                "a combinational loop in {}: {} nodes wait on each other, such as {}",
                self.d.name(self.d.spec(spec).name),
                nodes.len() - order.len(),
                stuck.join(", ")
            ));
        }
        Ok(order)
    }

    /// The bits a node reads and writes: (object, from, to) with `to`
    /// exclusive.
    #[allow(clippy::type_complexity)]
    fn access(&self, spec: SpecId, node: Node) -> Result<(Vec<(ObjId, u32, u32)>, Vec<(ObjId, u32, u32)>), String> {
        let d = self.d;
        let lay = &self.layouts[&spec];
        let mut r = Vec::new();
        let mut w = Vec::new();
        let whole = |o: ObjId| (o, 0, d.types.width(d.object(o).ty).max(1));
        match node {
            Node::Proc(p) => {
                let pr = &d.processes[p.0 as usize];
                r.extend(pr.reads.iter().filter(|o| lay.objs.contains_key(o)).map(|&o| whole(o)));
                w.extend(pr.writes.iter().filter(|o| lay.objs.contains_key(o)).map(|&o| whole(o)));
            }
            Node::Driver(dr) => {
                let drv = &d.drivers[dr.0 as usize];
                self.reads(drv.rhs, &mut r);
                self.writes(drv.lhs, &mut w, &mut r);
            }
            Node::Loop(l) => {
                for &dr in &self.loops[l].all {
                    let drv = &d.drivers[dr.0 as usize];
                    self.reads(drv.rhs, &mut r);
                    self.writes(drv.lhs, &mut w, &mut r);
                }
            }
            Node::In(i) => {
                for c in &d.instance(i).connections {
                    if let (Some(e), Some(Dir::Input)) = (c.expr, self.port_dir(i, c.port)) {
                        self.reads(e, &mut r);
                    }
                }
            }
            Node::Call(i) => {
                for c in &d.instance(i).connections {
                    if let (Some(e), Some(Dir::Output)) = (c.expr, self.port_dir(i, c.port)) {
                        self.writes(e, &mut w, &mut r);
                    }
                }
            }
        }
        Ok((r, w))
    }

    fn port_dir(&self, i: InstId, name: crate::design::Sym) -> Option<Dir> {
        self.d.spec(self.d.instance(i).spec).ports.iter().find(|p| p.name == name).map(|p| p.dir)
    }

    fn reads(&self, e: ExprId, out: &mut Vec<(ObjId, u32, u32)>) {
        let x = self.d.expr(e);
        match &x.kind {
            ExprKind::Object(o) => {
                if matches!(self.d.object(*o).kind, ObjKind::Variable | ObjKind::Net(_)) {
                    out.push((*o, 0, self.d.types.width(self.d.object(*o).ty).max(1)));
                }
            }
            ExprKind::Index | ExprKind::Part { .. } | ExprKind::Member(_) => {
                if let Some(r) = self.span(e) {
                    out.push(r);
                    for &a in &x.args[1..] {
                        self.reads(a, out);
                    }
                } else {
                    for &a in &x.args {
                        self.reads(a, out);
                    }
                }
            }
            _ => {
                for &a in &x.args {
                    self.reads(a, out);
                }
            }
        }
    }

    /// What an lvalue writes, and what its indices read.
    fn writes(&self, e: ExprId, w: &mut Vec<(ObjId, u32, u32)>, r: &mut Vec<(ObjId, u32, u32)>) {
        let x = self.d.expr(e);
        match &x.kind {
            ExprKind::Concat => {
                for &a in &x.args {
                    self.writes(a, w, r);
                }
            }
            _ => {
                if let Some(s) = self.span(e) {
                    w.push(s);
                }
                if matches!(x.kind, ExprKind::Index | ExprKind::Part { .. } | ExprKind::Member(_)) {
                    for &a in &x.args[1..] {
                        self.reads(a, r);
                    }
                }
            }
        }
    }

    /// The bits a select of an object covers, if it is rooted at an object:
    /// exact for constant selects, the whole object otherwise.
    fn span(&self, e: ExprId) -> Option<(ObjId, u32, u32)> {
        let d = self.d;
        let x = d.expr(e);
        match &x.kind {
            ExprKind::Object(o) => {
                matches!(d.object(*o).kind, ObjKind::Variable | ObjKind::Net(_)).then(|| (*o, 0, d.types.width(d.object(*o).ty).max(1)))
            }
            ExprKind::Index | ExprKind::Part { .. } | ExprKind::Member(_) => {
                let (o, lo, hi) = self.span(x.args[0])?;
                let whole = (o, 0, d.types.width(d.object(o).ty).max(1));
                match self.const_select(e) {
                    Some(Some((off, w))) if off >= 0 && (lo as i64 + off + w as i64) <= hi as i64 => {
                        let lo = lo + off as u32;
                        Some((o, lo, lo + w))
                    }
                    _ => Some(whole),
                }
            }
            _ => None,
        }
    }

    /// A select with constant indices: its bit offset in its base and its
    /// width (`None` inside if out of range); `None` if not constant.
    fn const_select(&self, e: ExprId) -> Option<Option<(i64, u32)>> {
        let d = self.d;
        let tys = &d.types;
        let x = d.expr(e);
        let bt = d.expr(x.args[0]).ty;
        match &x.kind {
            ExprKind::Index => {
                if !matches!(tys.get(bt), Type::Vector { .. } | Type::PackedArray { .. } | Type::Struct { packed: true, .. } | Type::Enum { .. }) {
                    return None;
                }
                let k = self.fold(x.args[1])?.to_i64()?;
                let ew = tys.width(x.ty).max(1);
                Some(ops::lsb_offset(tys, bt, k, k).map(|off| (off * ew as i64, ew)))
            }
            ExprKind::Part { kind, width } => {
                let a = self.fold(x.args[1])?.to_i64()?;
                let (hi, lo) = match kind {
                    PartKind::Range => (a, self.fold(x.args[2])?.to_i64()?),
                    PartKind::Up => (a + *width as i64 - 1, a),
                    PartKind::Down => (a, a - *width as i64 + 1),
                };
                let ew = tys.elem_width(bt).max(1);
                let (start, _) = ops::select_span(tys, bt, lo, hi);
                Some(Some((start * ew as i64, *width * ew)))
            }
            ExprKind::Member(i) => match tys.get(bt) {
                Type::Struct { packed: true, members, union, .. } => {
                    let w = tys.width(members[*i as usize].ty);
                    let below: u32 = if *union { 0 } else { members[*i as usize + 1..].iter().map(|m| tys.width(m.ty)).sum() };
                    Some(Some((below as i64, w)))
                }
                _ => None,
            },
            _ => None,
        }
    }

    /// The value of a constant expression: literals, parameters and what
    /// is built from them.
    fn fold(&self, e: ExprId) -> Option<Const> {
        // In a rolled loop, what is worked out from its stepping
        // parameters is not constant, and not remembered.
        if !self.loop_vars.borrow().is_empty() {
            return self.fold_new(e);
        }
        if let Some(c) = self.folds.borrow().get(&e) {
            return c.clone();
        }
        let c = self.fold_new(e);
        self.folds.borrow_mut().insert(e, c.clone());
        c
    }

    fn fold_new(&self, e: ExprId) -> Option<Const> {
        let d = self.d;
        let tys = &d.types;
        let x = d.expr(e);
        match &x.kind {
            ExprKind::Const(c) => Some(d.konst(*c).clone()),
            ExprKind::Object(o) => {
                let ob = d.object(*o);
                match ob.kind {
                    _ if self.loop_vars.borrow().contains(o) => None,
                    ObjKind::Param { .. } | ObjKind::EnumConst | ObjKind::Genvar => ob.value.map(|v| d.konst(v).clone()),
                    _ => None,
                }
            }
            ExprKind::Unary(op) => {
                if matches!(op, UnOp::PreInc | UnOp::PreDec | UnOp::PostInc | UnOp::PostDec) {
                    return None;
                }
                ops::unary(tys, *op, self.fold(x.args[0])?, x.ty)
            }
            ExprKind::Binary(op) => ops::binary(tys, *op, self.fold(x.args[0])?, self.fold(x.args[1])?, x.ty),
            ExprKind::Convert => Some(ops::convert(tys, &self.fold(x.args[0])?, x.ty)),
            ExprKind::Cond => {
                if ops::truth(&self.fold(x.args[0])?)? {
                    self.fold(x.args[1])
                } else {
                    self.fold(x.args[2])
                }
            }
            ExprKind::Concat => {
                let parts: Option<Vec<Bits>> = x.args.iter().map(|&a| ops::bits_of(&self.fold(a)?)).collect();
                Some(Const::Bits(Bits::concat(&parts?)))
            }
            ExprKind::Member(i) => {
                let b = self.fold(x.args[0])?;
                let bt = d.expr(x.args[0]).ty;
                match (tys.get(bt), &b) {
                    (Type::Struct { packed: true, members, union, .. }, Const::Bits(bits)) => {
                        let w = tys.width(members[*i as usize].ty);
                        let below: u32 = if *union { 0 } else { members[*i as usize + 1..].iter().map(|m| tys.width(m.ty)).sum() };
                        Some(Const::Bits(bits.slice(below, w).with_signed(tys.is_signed(x.ty))))
                    }
                    (_, Const::Aggregate(v)) => v.get(*i as usize).cloned(),
                    _ => None,
                }
            }
            ExprKind::Index => {
                let b = self.fold(x.args[0])?;
                let i = self.fold(x.args[1])?;
                ops::index(tys, &b, d.expr(x.args[0]).ty, &i, x.ty)
            }
            _ => None,
        }
    }
}

/// One 32-bit piece of a value in every lane: a constant, or `%tN`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum P {
    K(u32),
    V(u32),
}

/// A mask, one bit per lane: a constant, or `%tN`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum M {
    K(bool),
    V(u32),
}

/// A `w`-bit value: its pieces, low first, with the bits above `w` zero.
#[derive(Clone, PartialEq, Debug)]
struct Val {
    w: u32,
    p: Vec<P>,
}

impl Val {
    fn zero(w: u32) -> Val {
        Val { w, p: vec![P::K(0); pieces(w)] }
    }

    fn is_const(&self) -> Option<u64> {
        if self.p.len() > 2 {
            return None;
        }
        let mut v = 0u64;
        for (k, p) in self.p.iter().enumerate() {
            match p {
                P::K(x) => v |= (*x as u64) << (32 * k),
                P::V(_) => return None,
            }
        }
        Some(v)
    }
}

#[derive(Clone, Default)]
struct Env {
    vals: HashMap<ObjId, Val>,
    /// The static values written since the loop began.
    dirty: HashSet<ObjId>,
    /// Elements of objects kept an element to a slot, and those written.
    elems: HashMap<(ObjId, u32), Val>,
    edirty: HashSet<(ObjId, u32)>,
}

/// One function being written.
struct Func<'l, 'd, 'a> {
    l: &'l Lowering<'d, 'a>,
    spec: SpecId,
    out: String,
    next: u32,
    env: Env,
    /// The label of the block being written.
    block: String,
    loops: u32,
    /// Rolled loops written so far.
    rolled: u32,
    /// In a rolled loop: static values are read from and written to the
    /// state as they are used, not kept for the end of the outer loop.
    mem: bool,
    /// The temporaries with the same value in every lane.
    uniform: HashSet<u32>,
    /// Temporaries that are 0 or 1 a lane, made from these masks.
    mask_of: HashMap<u32, M>,
    /// Masks that are a bit plane's 16 bits (`%tN`, an i16): logic on them
    /// is done on those.
    bits_of: HashMap<u32, u32>,
}

const V16: &str = "<16 x i32>";
const M16: &str = "<16 x i1>";

fn ps(p: P) -> String {
    match p {
        P::K(k) => format!("splat (i32 {})", k as i32),
        P::V(n) => format!("%t{n}"),
    }
}

fn ms(m: M) -> String {
    match m {
        M::K(b) => format!("splat (i1 {b})"),
        M::V(n) => format!("%t{n}"),
    }
}

impl<'l, 'd, 'a> Func<'l, 'd, 'a> {
    fn new(l: &'l Lowering<'d, 'a>, spec: SpecId) -> Self {
        let out = format!("define void @{}(ptr %s) {{\nentry:\n", fn_name(spec));
        Func { l, spec, out, next: 0, env: Env::default(), block: "entry".into(), loops: 0, rolled: 0, mem: false, uniform: HashSet::new(), mask_of: HashMap::new(), bits_of: HashMap::new() }
    }

    fn finish(mut self) -> String {
        self.out.push_str("  ret void\n}\n");
        self.out
    }

    fn d(&self) -> &'d Design<'a> {
        self.l.d
    }

    fn lay(&self) -> &'l Layout {
        &self.l.layouts[&self.spec]
    }

    fn tmp(&mut self) -> u32 {
        self.next += 1;
        self.next
    }

    /// Splits the nodes into loops at the children's calls, and writes them.
    fn body(&mut self, order: &[Node]) -> Result<(), String> {
        let mut seg: Vec<Node> = Vec::new();
        for &n in order {
            if let Node::Call(i) = n {
                self.run_loop(&seg)?;
                let off = self.lay().kids[&i];
                let child = self.d().instance(i).spec;
                let t = self.tmp();
                let _ = writeln!(self.out, "  %t{t} = getelementptr i8, ptr %s, i64 {off}");
                let _ = writeln!(self.out, "  call void @{}(ptr %t{t})", fn_name(child));
                seg = vec![n];
            } else {
                seg.push(n);
            }
        }
        self.run_loop(&seg)
    }

    /// A loop over the group's registers running `nodes` in order.
    fn run_loop(&mut self, nodes: &[Node]) -> Result<(), String> {
        if nodes.is_empty() {
            return Ok(());
        }
        let k = self.loops;
        self.loops += 1;
        let prev = std::mem::replace(&mut self.block, format!("L{k}"));
        let _ = writeln!(
            self.out,
            "  br label %L{k}\nL{k}:\n  %c{k} = phi i64 [ 0, %{prev} ], [ %c{k}.next, %L{k}.latch ]\n  %c{k}.off = shl i64 %c{k}, 6\n  \
             %b{k} = getelementptr i8, ptr %s, i64 %c{k}.off\n  %c{k}.half = shl i64 %c{k}, 1\n  \
             %h{k} = getelementptr i8, ptr %s, i64 %c{k}.half"
        );
        self.env = Env::default();
        for &n in nodes {
            self.node(n)?;
        }
        let mut dirty: Vec<ObjId> = self.env.dirty.iter().copied().collect();
        dirty.sort_unstable_by_key(|o| o.0);
        for o in dirty {
            let v = self.env.vals[&o].clone();
            let off = self.lay().objs[&o];
            self.store(off, &v);
        }
        self.flush_elems(None);
        let _ = writeln!(
            self.out,
            "  %c{k}.next = add i64 %c{k}, 1\n  %c{k}.end = icmp eq i64 %c{k}.next, {CHUNKS}\n  \
             br i1 %c{k}.end, label %X{k}, label %L{k}\nX{k}:"
        );
        // The back edge leaves from the last block of the body.
        self.out = self.out.replace(&format!("%L{k}.latch"), &format!("%{}", self.block));
        self.block = format!("X{k}");
        Ok(())
    }

    fn base(&self) -> String {
        format!("%b{}", self.loops - 1)
    }

    fn load(&mut self, off: u64, w: u32) -> Val {
        let base = self.base();
        let mut p = Vec::new();
        for k in 0..pieces(w) as u64 {
            let (a, v) = (self.tmp(), self.tmp());
            let _ = writeln!(
                self.out,
                "  %t{a} = getelementptr i8, ptr {base}, i64 {}\n  %t{v} = load {V16}, ptr %t{a}, align 64",
                off + k * PIECE
            );
            p.push(P::V(v));
        }
        Val { w, p }
    }

    fn store(&mut self, off: u64, v: &Val) {
        let base = self.base();
        for (k, &p) in v.p.iter().enumerate() {
            let a = self.tmp();
            let _ = writeln!(
                self.out,
                "  %t{a} = getelementptr i8, ptr {base}, i64 {}\n  store {V16} {}, ptr %t{a}, align 64",
                off + k as u64 * PIECE,
                ps(p)
            );
        }
    }

    fn node(&mut self, n: Node) -> Result<(), String> {
        let d = self.d();
        match n {
            Node::Loop(l) => self.rolled_loop(l),
            Node::Driver(dr) => {
                let drv = &d.drivers[dr.0 as usize];
                let v = self.expr(drv.rhs)?;
                self.assign(drv.lhs, v)
            }
            Node::Proc(p) => {
                let pr = &d.processes[p.0 as usize];
                self.stmt(pr.body)?;
                // Automatic variables end with the process.
                self.env.vals.retain(|o, _| d.object(*o).lifetime == Lifetime::Static);
                Ok(())
            }
            Node::In(i) => {
                let inst = d.instance(i);
                let child = &self.l.layouts[&inst.spec];
                let off = self.lay().kids[&i];
                for port in &d.spec(inst.spec).ports {
                    if port.dir != Dir::Input {
                        continue;
                    }
                    let o = port.object.ok_or("interface ports are not supported yet")?;
                    let w = d.types.width(d.object(o).ty);
                    let v = match inst.connections.iter().find(|c| c.port == port.name).and_then(|c| c.expr) {
                        Some(e) => {
                            let v = self.expr(e)?;
                            self.resize(&v, w, false)
                        }
                        None => Val::zero(w),
                    };
                    self.store(off + child.objs[&o], &v);
                }
                Ok(())
            }
            Node::Call(i) => {
                let inst = d.instance(i);
                let child = &self.l.layouts[&inst.spec];
                let off = self.lay().kids[&i];
                for c in &inst.connections {
                    let Some(port) = d.spec(inst.spec).ports.iter().find(|p| p.name == c.port) else { continue };
                    match (port.dir, c.expr) {
                        (Dir::Output, Some(e)) => {
                            let o = port.object.ok_or("interface ports are not supported yet")?;
                            let ty = d.object(o).ty;
                            let v = self.load(off + child.objs[&o], d.types.width(ty));
                            let w = d.types.width(d.expr(e).ty);
                            let v = self.resize(&v, w, d.types.is_signed(ty));
                            self.assign(e, v)?;
                        }
                        (Dir::Input, _) | (Dir::Output, None) => {}
                        (dir, _) => return Err(format!("{dir:?} ports are not supported yet")),
                    }
                }
                Ok(())
            }
        }
    }

    // ---- statements ----

    fn stmt(&mut self, s: StmtId) -> Result<(), String> {
        let d = self.d();
        if !matches!(d.stmt(s).kind, StmtKind::Null | StmtKind::Block { .. }) && self.l.only_asserts(s) {
            return Ok(());
        }
        match &d.stmt(s).kind {
            StmtKind::Null => Ok(()),
            StmtKind::Block { join: None, scope, stmts } => {
                if let Some(sc) = scope {
                    for item in &d.scope(*sc).items {
                        if let Item::Object(o) = *item {
                            let ob = d.object(o);
                            if ob.lifetime == Lifetime::Automatic && ob.kind == ObjKind::Variable {
                                let w = d.types.width(ob.ty);
                                let v = match ob.init {
                                    Some(e) => {
                                        let v = self.expr(e)?;
                                        self.resize(&v, w, false)
                                    }
                                    None => Val::zero(w),
                                };
                                self.env.vals.insert(o, v);
                            }
                        }
                    }
                }
                stmts.iter().try_for_each(|&s| self.stmt(s))
            }
            StmtKind::Assign { kind, lhs, rhs, timing: None } => {
                let v = match kind {
                    AssignKind::Blocking => self.expr(*rhs)?,
                    AssignKind::Compound(op) => {
                        let ty = d.expr(*lhs).ty;
                        let a = self.expr(*lhs)?;
                        let b = self.expr(*rhs)?;
                        self.binary(*op, a, b, ty, ty)?
                    }
                    AssignKind::NonBlocking => return Err("nonblocking assignments are not supported yet".into()),
                };
                self.assign(*lhs, v)
            }
            StmtKind::Expr(e) => match d.expr(*e).kind {
                ExprKind::Unary(op @ (UnOp::PreInc | UnOp::PostInc | UnOp::PreDec | UnOp::PostDec)) => {
                    let x = d.expr(*e).args[0];
                    let v = self.expr(x)?;
                    let one = self.konst(&Bits::from_u64(v.w, false, 1));
                    let v = if matches!(op, UnOp::PreInc | UnOp::PostInc) { self.add(&v, &one, false) } else { self.add(&v, &one, true) };
                    self.assign(x, v)
                }
                _ => Err("expression statements other than ++ and -- are not supported yet".into()),
            },
            StmtKind::If { cond, then, else_, .. } => {
                let c = self.expr(*cond)?;
                let m = self.truth(&c);
                let (then, else_) = (*then, *else_);
                self.branch(
                    m,
                    |f| f.stmt(then),
                    |f| match else_ {
                        Some(e) => f.stmt(e),
                        None => Ok(()),
                    },
                )
            }
            StmtKind::Case { kind, expr, items, default, .. } => {
                let sel = self.expr(*expr)?;
                let st = d.expr(*expr).ty;
                let mut arms = Vec::new();
                for (labels, body) in items {
                    let mut m = M::K(false);
                    for &l in labels {
                        let hit = self.case_match(*kind, &sel, st, l)?;
                        m = self.mor(m, hit);
                    }
                    arms.push((m, *body));
                }
                self.case_chain(&arms, *default)
            }
            StmtKind::For { init, cond, step, body, .. } => {
                for &s in init {
                    self.stmt(s)?;
                }
                for _ in 0..65536 {
                    if let Some(c) = cond {
                        let v = self.expr(*c)?;
                        match self.truth(&v) {
                            M::K(true) => {}
                            M::K(false) => return Ok(()),
                            M::V(_) => return Err("loops whose trip count is not a constant are not supported yet".into()),
                        }
                    }
                    self.stmt(*body)?;
                    for &s in step {
                        self.stmt(s)?;
                    }
                }
                Err("a loop of more than 65536 iterations".into())
            }
            StmtKind::Assert { .. } => {
                self.l.asserts.borrow_mut().insert(s);
                Ok(())
            }
            k => Err(format!("statement {} is not supported yet", stmt_name(k))),
        }
    }

    fn case_chain(&mut self, arms: &[(M, StmtId)], default: Option<StmtId>) -> Result<(), String> {
        match arms.split_first() {
            None => match default {
                Some(s) => self.stmt(s),
                None => Ok(()),
            },
            Some((&(m, body), rest)) => self.branch(m, |f| f.stmt(body), |f| f.case_chain(rest, default)),
        }
    }

    /// Whether a case item matches, with `casez` and `casex` don't-cares in
    /// constant items (12.5.1).
    fn case_match(&mut self, kind: CaseKind, sel: &Val, st: TypeId, item: ExprId) -> Result<M, String> {
        let d = self.d();
        let w = sel.w;
        if let Some(Const::Bits(b)) = self.l.fold(item)
            && !b.is_known()
        {
            let b = b.resize(w, d.types.is_signed(d.expr(item).ty));
            let (mut care, mut val) = (Bits::zero(w, false), Bits::zero(w, false));
            for i in 0..w {
                let dc = match (kind, b.bit(i)) {
                    (_, Logic::Zero | Logic::One) => false,
                    (CaseKind::CaseZ, Logic::Z) | (CaseKind::CaseX, Logic::Z | Logic::X) => true,
                    _ => return Ok(M::K(false)),
                };
                if !dc {
                    care.set_bit(i, Logic::One);
                    val.set_bit(i, b.bit(i));
                }
            }
            let (care, val) = (self.konst(&care), self.konst(&val));
            let masked = self.bitwise("and", sel, &care);
            return Ok(self.eq(&masked, &val));
        }
        if kind == CaseKind::Inside {
            return Err("case inside is not supported yet".into());
        }
        let v = self.expr(item)?;
        let v = self.resize(&v, w, d.types.is_signed(st));
        Ok(self.eq(sel, &v))
    }

    /// Both arms, each on its own copy of the values, then a select of
    /// each value that differs. A constant condition runs one arm.
    fn branch(
        &mut self,
        m: M,
        then: impl FnOnce(&mut Self) -> Result<(), String>,
        else_: impl FnOnce(&mut Self) -> Result<(), String>,
    ) -> Result<(), String> {
        match m {
            M::K(true) => return then(self),
            M::K(false) => return else_(self),
            M::V(_) => {}
        }
        let before = self.env.clone();
        then(self)?;
        let a = std::mem::replace(&mut self.env, before.clone());
        else_(self)?;
        let b = std::mem::replace(&mut self.env, before);
        let mut keys: Vec<ObjId> = a.vals.keys().chain(b.vals.keys()).copied().collect();
        keys.sort_unstable_by_key(|o| o.0);
        keys.dedup();
        for o in keys {
            let va = match a.vals.get(&o) {
                Some(v) => v.clone(),
                None => self.read(o),
            };
            let vb = match b.vals.get(&o) {
                Some(v) => v.clone(),
                None => self.read(o),
            };
            let v = if va == vb { va } else { self.select(m, &va, &vb) };
            self.env.vals.insert(o, v);
        }
        self.env.dirty.extend(a.dirty.into_iter().chain(b.dirty));
        let mut keys: Vec<(ObjId, u32)> = a.elems.keys().chain(b.elems.keys()).copied().collect();
        keys.sort_unstable_by_key(|k| (k.0.0, k.1));
        keys.dedup();
        for (o, i) in keys {
            let va = match a.elems.get(&(o, i)) {
                Some(v) => v.clone(),
                None => self.read_elem(o, i),
            };
            let vb = match b.elems.get(&(o, i)) {
                Some(v) => v.clone(),
                None => self.read_elem(o, i),
            };
            let v = if va == vb { va } else { self.select(m, &va, &vb) };
            self.env.elems.insert((o, i), v);
        }
        self.env.edirty.extend(a.edirty.into_iter().chain(b.edirty));
        Ok(())
    }

    /// The address of element `i` (a constant) of `o`, in this register's
    /// part of the state.
    fn elem_addr(&mut self, o: ObjId, i: u32) -> String {
        let e = self.lay().elems[&o];
        let base = if e.planes() { format!("%h{}", self.loops - 1) } else { self.base() };
        let a = self.tmp();
        let _ = writeln!(self.out, "  %t{a} = getelementptr i8, ptr {base}, i64 {}", self.lay().objs[&o] + i as u64 * e.stride());
        format!("%t{a}")
    }

    /// The address of element `i`, the same in every lane (held at the
    /// last element).
    fn elem_addr_uniform(&mut self, o: ObjId, i: P) -> String {
        if let P::K(k) = i {
            let n = self.lay().elems[&o].n;
            return self.elem_addr(o, k.min(n - 1));
        }
        let e = self.lay().elems[&o];
        let base = if e.planes() { format!("%h{}", self.loops - 1) } else { self.base() };
        let off = self.lay().objs[&o];
        let t: Vec<u32> = (0..6).map(|_| self.tmp()).collect();
        let _ = writeln!(
            self.out,
            "  %t{} = extractelement {V16} {}, i64 0\n  %t{} = call i32 @llvm.umin.i32(i32 %t{}, i32 {})\n  \
             %t{} = zext i32 %t{} to i64\n  %t{} = mul i64 %t{}, {}\n  %t{} = getelementptr i8, ptr {base}, i64 {off}\n  \
             %t{} = getelementptr i8, ptr %t{}, i64 %t{}",
            t[0], ps(i), t[1], t[0], e.n - 1, t[2], t[1], t[3], t[2], e.stride(), t[4], t[5], t[4], t[3]
        );
        format!("%t{}", t[5])
    }

    fn load_slot(&mut self, o: ObjId, addr: &str) -> Val {
        let e = self.lay().elems[&o];
        let t = self.tmp();
        if e.planes() {
            let m = self.tmp();
            let _ = writeln!(self.out, "  %t{t} = load i16, ptr {addr}, align 2\n  %t{m} = bitcast i16 %t{t} to {M16}");
            self.bits_of.insert(m, t);
            self.bit(M::V(m), 1)
        } else {
            let _ = writeln!(self.out, "  %t{t} = load {V16}, ptr {addr}, align 64");
            Val { w: e.ew, p: vec![P::V(t)] }
        }
    }

    fn store_slot(&mut self, o: ObjId, addr: &str, v: &Val) {
        let e = self.lay().elems[&o];
        if e.planes() {
            let m = self.truth(v);
            let bits = match m {
                M::K(b) => format!("{}", if b { -1 } else { 0 }),
                M::V(n) => match self.bits_of.get(&n) {
                    Some(&i) => format!("%t{i}"),
                    None => {
                        let t = self.tmp();
                        let _ = writeln!(self.out, "  %t{t} = bitcast {M16} %t{n} to i16");
                        format!("%t{t}")
                    }
                },
            };
            let _ = writeln!(self.out, "  store i16 {bits}, ptr {addr}, align 2");
        } else {
            let v = self.resize(v, e.ew, false);
            let _ = writeln!(self.out, "  store {V16} {}, ptr {addr}, align 64", ps(v.p[0]));
        }
    }

    /// Element `i` (a constant) of an object kept an element to a slot.
    fn read_elem(&mut self, o: ObjId, i: u32) -> Val {
        if let Some(v) = self.env.elems.get(&(o, i)) {
            return v.clone();
        }
        let a = self.elem_addr(o, i);
        let v = self.load_slot(o, &a);
        self.env.elems.insert((o, i), v.clone());
        v
    }

    fn write_elem(&mut self, o: ObjId, i: u32, v: Val) {
        let ew = self.lay().elems[&o].ew;
        let v = self.resize(&v, ew, false);
        if self.mem {
            let a = self.elem_addr(o, i);
            self.store_slot(o, &a, &v);
        } else {
            self.env.edirty.insert((o, i));
        }
        self.env.elems.insert((o, i), v);
    }

    /// Stores the elements written since the loop began: of `o`, or of all.
    fn flush_elems(&mut self, o: Option<ObjId>) {
        let mut dirty: Vec<(ObjId, u32)> = self.env.edirty.iter().copied().filter(|k| o.is_none_or(|o| k.0 == o)).collect();
        dirty.sort_unstable_by_key(|k| (k.0.0, k.1));
        for k in dirty {
            self.env.edirty.remove(&k);
            let v = self.env.elems[&k].clone();
            let a = self.elem_addr(k.0, k.1);
            self.store_slot(k.0, &a, &v);
        }
    }

    /// Which element of an object kept an element to a slot `e` (an
    /// `Index` of it) selects: a constant, or one a lane, with whether it
    /// is in range.
    fn elem_index(&mut self, e: ExprId) -> Result<(P, M), String> {
        let d = self.d();
        let x = d.expr(e);
        let bt = d.expr(x.args[0]).ty;
        let (l, r) = d.types.packed_range(bt).unwrap_or((d.types.width(bt) as i64 - 1, 0));
        let idx = match self.l.fold(x.args[1]) {
            Some(c) => self.konst_of(&c, d.types.width(d.expr(x.args[1]).ty))?,
            None => self.expr(x.args[1])?,
        };
        let signed = d.types.is_signed(d.expr(x.args[1]).ty);
        let k = idx.p[0];
        let mut ok = M::K(true);
        for &hp in &idx.p[1..] {
            let z = self.icmp("eq", hp, P::K(0));
            ok = self.mand(ok, z);
        }
        let (min, max) = (l.min(r), l.max(r));
        let (ge, le) = if signed { ("sge", "sle") } else { ("uge", "ule") };
        let a = self.icmp(ge, k, P::K(min as u32));
        let b = self.icmp(le, k, P::K(max as u32));
        ok = self.mand(ok, a);
        ok = self.mand(ok, b);
        let i = if l >= r { self.bin("sub", k, P::K(r as u32)) } else { self.bin("sub", P::K(r as u32), k) };
        Ok((i, ok))
    }

    /// `o[...]` read, for an object kept an element to a slot.
    fn elem_read(&mut self, o: ObjId, e: ExprId) -> Result<Val, String> {
        let el = self.lay().elems[&o];
        let (i, ok) = self.elem_index(e)?;
        let v = match i {
            P::K(k) if k < el.n => self.read_elem(o, k),
            P::K(_) => Val::zero(el.ew),
            _ if self.is_uniform(i) => {
                let a = self.elem_addr_uniform(o, i);
                self.load_slot(o, &a)
            }
            // A different element a lane: each, chosen by a select.
            _ => {
                let mut acc = Val::zero(el.ew);
                for k in 0..el.n {
                    let hit = self.icmp("eq", i, P::K(k));
                    let v = self.read_elem(o, k);
                    acc = self.select(hit, &v, &acc);
                }
                acc
            }
        };
        let z = Val::zero(el.ew);
        Ok(self.select(ok, &v, &z))
    }

    /// `o[...] = v`, for an object kept an element to a slot.
    fn elem_write(&mut self, o: ObjId, e: ExprId, v: Val) -> Result<(), String> {
        let el = self.lay().elems[&o];
        let (i, ok) = self.elem_index(e)?;
        match i {
            P::K(k) if k < el.n => {
                let v = match ok {
                    M::K(true) => v,
                    _ => {
                        let old = self.read_elem(o, k);
                        self.select(ok, &v, &old)
                    }
                };
                self.write_elem(o, k, v);
            }
            P::K(_) => {}
            _ if self.is_uniform(i) && self.mem => {
                // Any element may change: none is remembered.
                self.flush_elems(Some(o));
                self.env.elems.retain(|k, _| k.0 != o);
                let a = self.elem_addr_uniform(o, i);
                let v = match ok {
                    M::K(true) => v,
                    _ => {
                        let old = self.load_slot(o, &a);
                        self.select(ok, &v, &old)
                    }
                };
                self.store_slot(o, &a, &v);
            }
            _ => {
                for k in 0..el.n {
                    let hit = self.icmp("eq", i, P::K(k));
                    let hit = self.mand(hit, ok);
                    let old = self.read_elem(o, k);
                    let new = self.select(hit, &v, &old);
                    self.write_elem(o, k, new);
                }
            }
        }
        Ok(())
    }

    /// The object kept an element to a slot that an `Index` selects from
    /// directly, if its element is what the `Index` gives.
    fn elem_target(&self, e: ExprId) -> Option<ObjId> {
        let d = self.d();
        let x = d.expr(e);
        if !matches!(x.kind, ExprKind::Index) {
            return None;
        }
        let ExprKind::Object(o) = d.expr(x.args[0]).kind else { return None };
        let el = self.lay().elems.get(&o)?;
        (d.types.width(x.ty) == el.ew).then_some(o)
    }

    /// An object's value: from this loop's values, from the state, or zero
    /// for an automatic variable not yet set.
    fn read(&mut self, o: ObjId) -> Val {
        if let Some(&e) = self.lay().elems.get(&o) {
            // The whole value, from its elements.
            let mut acc = Val::zero(e.n * e.ew);
            for i in 0..e.n {
                let v = self.read_elem(o, i);
                acc = self.insert(&acc, i * e.ew, e.ew, &v);
            }
            return acc;
        }
        if let Some(v) = self.env.vals.get(&o) {
            return v.clone();
        }
        let d = self.d();
        let w = d.types.width(d.object(o).ty);
        let v = match self.lay().objs.get(&o) {
            Some(&off) => self.load(off, w),
            None => Val::zero(w),
        };
        self.env.vals.insert(o, v.clone());
        v
    }

    fn set(&mut self, o: ObjId, v: Val) {
        if let Some(&e) = self.lay().elems.get(&o) {
            for i in 0..e.n {
                let x = self.extract(&v, i * e.ew, e.ew);
                self.write_elem(o, i, x);
            }
            return;
        }
        if let Some(&off) = self.lay().objs.get(&o) {
            if self.mem {
                self.store(off, &v);
            } else {
                self.env.dirty.insert(o);
            }
        }
        self.env.vals.insert(o, v);
    }

    fn assign(&mut self, lhs: ExprId, v: Val) -> Result<(), String> {
        let d = self.d();
        let x = d.expr(lhs);
        match &x.kind {
            ExprKind::Object(o) => {
                let w = d.types.width(d.object(*o).ty);
                let v = self.resize(&v, w, false);
                self.set(*o, v);
                Ok(())
            }
            ExprKind::Concat => {
                let mut top: u32 = x.args.iter().map(|&a| d.types.width(d.expr(a).ty)).sum();
                for &a in &x.args {
                    let w = d.types.width(d.expr(a).ty);
                    top -= w;
                    let part = self.extract(&v, top, w);
                    self.assign(a, part)?;
                }
                Ok(())
            }
            ExprKind::Index | ExprKind::Part { .. } | ExprKind::Member(_) => {
                if let Some(o) = self.elem_target(lhs) {
                    return self.elem_write(o, lhs, v);
                }
                let (o, off, w) = self.place(lhs)?;
                if self.mem
                    && !self.lay().elems.contains_key(&o)
                    && w <= 32
                    && self.lay().objs.contains_key(&o)
                    && let Some((p, ok)) = self.uniform_off(off)
                {
                    self.mem_insert(o, p, ok, w, &v);
                    return Ok(());
                }
                let old = self.read(o);
                let new = match off {
                    Off::K(k) => self.insert(&old, k, w, &v),
                    Off::V(p, ok) => self.insert_var(&old, p, ok, w, &v),
                    Off::Neg(_) => return Err("a part select partly out of range as a target".into()),
                };
                self.set(o, new);
                Ok(())
            }
            k => Err(format!("assignments to {k:?} are not supported yet")),
        }
    }

    /// Where a select on the left of an assignment is: its object, its bit
    /// offset there, its width.
    fn place(&mut self, e: ExprId) -> Result<(ObjId, Off, u32), String> {
        let d = self.d();
        let x = d.expr(e);
        if let ExprKind::Object(o) = x.kind {
            return Ok((o, Off::K(0), d.types.width(x.ty)));
        }
        let (o, base, _) = self.place(x.args[0])?;
        let (off, w) = self.offset(e)?;
        let off = match (base, off) {
            (Off::K(a), Off::K(b)) => Off::K(a + b),
            (Off::Neg(_), _) | (_, Off::Neg(_)) => return Err("a part select partly out of range as a target".into()),
            (a, b) => {
                let (pa, ma) = a.var();
                let (pb, mb) = b.var();
                let p = self.bin("add", pa, pb);
                let m = self.mand(ma, mb);
                Off::V(p, m)
            }
        };
        Ok((o, off, w))
    }

    /// The bit offset of a select in its base, and its width.
    fn offset(&mut self, e: ExprId) -> Result<(Off, u32), String> {
        let d = self.d();
        let tys = &d.types;
        let x = d.expr(e);
        if let Some(c) = self.l.const_select(e) {
            return Ok(match c {
                Some((off, w)) => (Off::K(off as u32).with_sign(off), w),
                None => (Off::V(P::K(0), M::K(false)), tys.width(x.ty)),
            });
        }
        let bt = d.expr(x.args[0]).ty;
        let (l, r) = tys.packed_range(bt).unwrap_or((tys.width(bt) as i64 - 1, 0));
        let idx = self.expr(x.args[1])?;
        let signed = tys.is_signed(d.expr(x.args[1]).ty);
        let (lo_index, ew, w) = match &x.kind {
            ExprKind::Index => {
                let ew = tys.width(x.ty).max(1);
                (idx.clone(), ew, ew)
            }
            ExprKind::Part { kind, width } => {
                let ew = tys.elem_width(bt).max(1);
                let lo = match kind {
                    PartKind::Up => idx.clone(),
                    PartKind::Down => {
                        let k = self.konst(&Bits::from_u64(idx.w, false, *width as u64 - 1));
                        self.add(&idx, &k, true)
                    }
                    PartKind::Range => return Err("a part select with variable bounds".into()),
                };
                (lo, ew, *width * ew)
            }
            _ => return Err(format!("a select {:?} with variable indices", x.kind)),
        };
        // Elements from the least significant end: k - r if descending,
        // r - (k + n - 1) if ascending; in range if min <= k <= max - n + 1.
        let n = (w / ew) as i64;
        let k = lo_index.p[0];
        let mut ok = M::K(true);
        for &hp in &lo_index.p[1..] {
            let z = self.icmp("eq", hp, P::K(0));
            ok = self.mand(ok, z);
        }
        let (min, max) = (l.min(r), l.max(r));
        let (lt, le) = if signed { ("sge", "sle") } else { ("uge", "ule") };
        let a = self.icmp(lt, k, P::K(min as u32));
        // A part select that runs off the top reads and writes the bits
        // still in range.
        let top = if matches!(x.kind, ExprKind::Part { .. }) && l >= r { max } else { max - n + 1 };
        let b = self.icmp(le, k, P::K(top as u32));
        ok = self.mand(ok, a);
        ok = self.mand(ok, b);
        let elems = if l >= r { self.bin("sub", k, P::K(r as u32)) } else { self.bin("sub", P::K((r - n + 1) as u32), k) };
        let bits = if ew.is_power_of_two() { self.bin("shl", elems, P::K(ew.trailing_zeros())) } else { self.bin("mul", elems, P::K(ew)) };
        Ok((Off::V(bits, ok), w))
    }

    /// A run of iterations as a loop inside the loop over registers.
    fn rolled_loop(&mut self, l: usize) -> Result<(), String> {
        let d = self.d();
        let lp = &self.l.loops[l];
        // The loop reads and writes the state: what this loop has worked
        // out goes there first, and is read back after.
        let mut touched = Vec::new();
        for &dr in &lp.all {
            let drv = &d.drivers[dr.0 as usize];
            let (mut r, mut w) = (Vec::new(), Vec::new());
            self.l.reads(drv.rhs, &mut r);
            self.l.writes(drv.lhs, &mut w, &mut r);
            touched.extend(r.into_iter().chain(w).map(|t| t.0));
        }
        touched.sort_unstable_by_key(|o| o.0);
        touched.dedup();
        for &o in &touched {
            if self.env.dirty.remove(&o) {
                let v = self.env.vals[&o].clone();
                let off = self.lay().objs[&o];
                self.store(off, &v);
            }
            self.env.vals.remove(&o);
            if self.lay().elems.contains_key(&o) {
                self.flush_elems(Some(o));
                self.env.elems.retain(|k, _| k.0 != o);
            }
        }
        let k = self.rolled;
        self.rolled += 1;
        let prev = std::mem::replace(&mut self.block, format!("R{k}"));
        let _ = writeln!(self.out, "  br label %R{k}\nR{k}:\n  %r{k} = phi i32 [ 0, %{prev} ], [ %r{k}.next, %R{k} ]");
        let outer = std::mem::take(&mut self.env);
        self.mem = true;
        *self.l.loop_vars.borrow_mut() = lp.steps.iter().map(|s| s.0).collect();
        for &(o, first, step) in &lp.steps {
            let (a, b, c, t) = (self.tmp(), self.tmp(), self.tmp(), self.tmp());
            let _ = writeln!(
                self.out,
                "  %t{a} = mul i32 %r{k}, {}\n  %t{b} = add i32 %t{a}, {}\n  \
                 %t{c} = insertelement {V16} poison, i32 %t{b}, i64 0\n  \
                 %t{t} = shufflevector {V16} %t{c}, {V16} poison, {V16} zeroinitializer",
                step as i32,
                first as i32
            );
            self.uniform.insert(t);
            let w = d.types.width(d.object(o).ty);
            let mut p = vec![P::K(0); pieces(w)];
            p[0] = P::V(t);
            if pieces(w) > 1 {
                let f = self.bin("ashr", P::V(t), P::K(31));
                for q in &mut p[1..] {
                    *q = f;
                }
            }
            let v = self.norm(Val { w, p });
            self.env.vals.insert(o, v);
        }
        let result = lp.body.iter().try_for_each(|&dr| {
            let drv = &d.drivers[dr.0 as usize];
            let v = self.expr(drv.rhs)?;
            self.assign(drv.lhs, v)
        });
        self.l.loop_vars.borrow_mut().clear();
        self.mem = false;
        self.env = outer;
        result?;
        let _ = writeln!(
            self.out,
            "  %r{k}.next = add i32 %r{k}, 1\n  %r{k}.end = icmp eq i32 %r{k}.next, {}\n  \
             br i1 %r{k}.end, label %Y{k}, label %R{k}\nY{k}:",
            lp.count
        );
        self.block = format!("Y{k}");
        Ok(())
    }

    /// The static object an expression names, in a rolled loop, where
    /// selects of it go to the state.
    fn in_memory(&self, e: ExprId) -> Option<ObjId> {
        match self.d().expr(e).kind {
            ExprKind::Object(o) if self.mem && self.lay().objs.contains_key(&o) && !self.lay().elems.contains_key(&o) => Some(o),
            _ => None,
        }
    }

    /// A select's offset if it is the same in every lane.
    fn uniform_off(&self, off: Off) -> Option<(P, M)> {
        match off {
            Off::K(k) => Some((P::K(k), M::K(true))),
            Off::V(p, ok) if self.is_uniform(p) => Some((p, ok)),
            _ => None,
        }
    }

    fn is_uniform(&self, p: P) -> bool {
        match p {
            P::K(_) => true,
            P::V(n) => self.uniform.contains(&n),
        }
    }

    /// The addresses of the two pieces holding bits `off..off + 32` of
    /// `o` (the second held at the last piece), and the shift within the
    /// first.
    fn piece_addrs(&mut self, o: ObjId, off: P) -> (String, String, P) {
        let d = self.d();
        let base = self.base();
        let start = self.lay().objs[&o];
        let last = pieces(d.types.width(d.object(o).ty)) as u64 - 1;
        let r = self.bin("and", off, P::K(31));
        if let P::K(k) = off {
            let q = (k as u64 / 32).min(last);
            let q1 = (q + 1).min(last);
            let (a, b) = (self.tmp(), self.tmp());
            let _ = writeln!(
                self.out,
                "  %t{a} = getelementptr i8, ptr {base}, i64 {}\n  %t{b} = getelementptr i8, ptr {base}, i64 {}",
                start + q * PIECE,
                start + q1 * PIECE
            );
            return (format!("%t{a}"), format!("%t{b}"), r);
        }
        let t = self.tmp();
        let _ = writeln!(self.out, "  %t{t} = extractelement {V16} {}, i64 0", ps(off));
        let n: Vec<u32> = (0..9).map(|_| self.tmp()).collect();
        let _ = writeln!(
            self.out,
            "  %t{0} = lshr i32 %t{t}, 5\n  %t{1} = call i32 @llvm.umin.i32(i32 %t{0}, i32 {last})\n  \
             %t{2} = add i32 %t{1}, 1\n  %t{3} = call i32 @llvm.umin.i32(i32 %t{2}, i32 {last})\n  \
             %t{4} = zext i32 %t{1} to i64\n  %t{5} = zext i32 %t{3} to i64\n  \
             %t{6} = mul i64 %t{4}, {PIECE}\n  %t{7} = mul i64 %t{5}, {PIECE}\n  \
             %t{8} = getelementptr i8, ptr {base}, i64 {start}",
            n[0], n[1], n[2], n[3], n[4], n[5], n[6], n[7], n[8]
        );
        let (a, b) = (self.tmp(), self.tmp());
        let _ = writeln!(
            self.out,
            "  %t{a} = getelementptr i8, ptr %t{}, i64 %t{}\n  %t{b} = getelementptr i8, ptr %t{}, i64 %t{}",
            n[8], n[6], n[8], n[7]
        );
        (format!("%t{a}"), format!("%t{b}"), r)
    }

    fn load_at(&mut self, addr: &str) -> P {
        let t = self.tmp();
        let _ = writeln!(self.out, "  %t{t} = load {V16}, ptr {addr}, align 64");
        P::V(t)
    }

    /// Bits `off..off + w` (`w` at most 32) of `o`, read from the state.
    fn mem_extract(&mut self, o: ObjId, off: P, ok: M, w: u32) -> Val {
        let (a0, a1, r) = self.piece_addrs(o, off);
        let (v0, v1) = (self.load_at(&a0), self.load_at(&a1));
        let lo = self.bin("lshr", v0, r);
        let ri = self.bin("sub", P::K(32), r);
        let ri = self.bin("and", ri, P::K(31));
        let rz = self.icmp("eq", r, P::K(0));
        let hi = self.bin("shl", v1, ri);
        let hi = self.sel(rz, P::K(0), hi);
        let v = self.bin("or", lo, hi);
        let v = self.norm(Val { w, p: vec![v] });
        let z = Val::zero(w);
        self.select(ok, &v, &z)
    }

    /// Writes `x` to bits `off..off + w` (`w` at most 32) of `o` in the
    /// state.
    fn mem_insert(&mut self, o: ObjId, off: P, ok: M, w: u32, x: &Val) {
        let (a0, a1, r) = self.piece_addrs(o, off);
        let (v0, v1) = (self.load_at(&a0), self.load_at(&a1));
        let x = self.resize(x, w, false).p[0];
        let m = P::K(top_mask(w));
        let ri = self.bin("sub", P::K(32), r);
        let ri = self.bin("and", ri, P::K(31));
        let rz = self.icmp("eq", r, P::K(0));
        let m0 = self.bin("shl", m, r);
        let x0 = self.bin("shl", x, r);
        let m1 = self.bin("lshr", m, ri);
        let m1 = self.sel(rz, P::K(0), m1);
        let x1 = self.bin("lshr", x, ri);
        let x1 = self.sel(rz, P::K(0), x1);
        let put = |f: &mut Self, v: P, m: P, x: P| {
            let keep = f.bin("xor", m, P::K(!0));
            let kept = f.bin("and", v, keep);
            let n = f.bin("or", kept, x);
            f.sel(ok, n, v)
        };
        let n1 = put(self, v1, m1, x1);
        let n0 = put(self, v0, m0, x0);
        // The second piece first: where both are the last piece, the
        // first's write is the one that stays.
        for (a, n) in [(a1, n1), (a0, n0)] {
            let _ = writeln!(self.out, "  store {V16} {}, ptr {a}, align 64", ps(n));
        }
        self.env.vals.remove(&o);
    }

    // ---- expressions ----

    fn expr(&mut self, e: ExprId) -> Result<Val, String> {
        let d = self.d();
        let tys = &d.types;
        let x = d.expr(e);
        if !matches!(x.kind, ExprKind::Object(_) | ExprKind::Const(_))
            && let Some(c) = self.l.fold(e)
        {
            return self.konst_of(&c, tys.width(x.ty));
        }
        let w = tys.width(x.ty);
        match &x.kind {
            ExprKind::Const(c) => self.konst_of(d.konst(*c), w),
            ExprKind::Object(o) => {
                let ob = d.object(*o);
                match ob.kind {
                    ObjKind::Variable | ObjKind::Net(_) => {
                        if ob.lifetime == Lifetime::Static && !self.lay().objs.contains_key(o) {
                            return Err(format!("{} is not in this module", d.name(ob.name)));
                        }
                        Ok(self.read(*o))
                    }
                    _ => {
                        if let Some(v) = self.env.vals.get(o) {
                            let v = v.clone();
                            return Ok(self.resize(&v, w, true));
                        }
                        let c = self.l.fold(e).ok_or_else(|| format!("{} has no value", d.name(ob.name)))?;
                        self.konst_of(&c, w)
                    }
                }
            }
            ExprKind::Unary(op) => {
                let a = self.expr(x.args[0])?;
                self.unary(*op, a, x.ty)
            }
            ExprKind::Binary(op) => {
                let a = self.expr(x.args[0])?;
                let b = self.expr(x.args[1])?;
                self.binary(*op, a, b, x.ty, d.expr(x.args[0]).ty)
            }
            ExprKind::Cond => {
                let c = self.expr(x.args[0])?;
                match self.truth(&c) {
                    M::K(true) => self.expr(x.args[1]),
                    M::K(false) => self.expr(x.args[2]),
                    m => {
                        let a = self.expr(x.args[1])?;
                        let b = self.expr(x.args[2])?;
                        let (a, b) = (self.resize(&a, w, false), self.resize(&b, w, false));
                        Ok(self.select(m, &a, &b))
                    }
                }
            }
            ExprKind::Concat => {
                let mut acc = Val::zero(w);
                let mut top = w;
                for &a in &x.args {
                    let v = self.expr(a)?;
                    top -= v.w;
                    acc = self.insert(&acc, top, v.w, &v);
                }
                Ok(acc)
            }
            ExprKind::Replicate(n) => {
                let v = self.expr(x.args[0])?;
                let mut acc = Val::zero(w);
                for k in 0..*n {
                    acc = self.insert(&acc, k * v.w, v.w, &v);
                }
                Ok(acc)
            }
            ExprKind::Index | ExprKind::Part { .. } | ExprKind::Member(_) => {
                if let Some(o) = self.elem_target(e) {
                    return self.elem_read(o, e);
                }
                if let Some(o) = self.in_memory(x.args[0]) {
                    let (off, w2) = self.offset(e)?;
                    if let Some((p, ok)) = self.uniform_off(off)
                        && w2 <= 32
                    {
                        return Ok(self.mem_extract(o, p, ok, w2));
                    }
                }
                let base = self.expr(x.args[0])?;
                let (off, w) = self.offset(e)?;
                Ok(match off {
                    Off::K(k) => self.extract(&base, k, w),
                    Off::Neg(k) => {
                        let v = self.resize(&base, base.w + k, false);
                        let v = self.shl_const(&v, k as u64);
                        self.extract(&v, 0, w)
                    }
                    Off::V(p, ok) => {
                        let v = self.lshr_var(&base, p, None);
                        let v = self.extract(&v, 0, w);
                        let z = Val::zero(w);
                        self.select(ok, &v, &z)
                    }
                })
            }
            ExprKind::Convert => {
                let st = d.expr(x.args[0]).ty;
                if !tys.is_integral(st) || !tys.is_integral(x.ty) {
                    return Err(format!("conversions from {} to {} are not supported yet", tys.describe(st, &d.syms), tys.describe(x.ty, &d.syms)));
                }
                let v = self.expr(x.args[0])?;
                Ok(self.resize(&v, w, tys.is_signed(st)))
            }
            k => Err(format!("{} is not supported yet", expr_name(k))),
        }
    }

    fn konst_of(&mut self, c: &Const, w: u32) -> Result<Val, String> {
        let b = ops::bits_of(c).ok_or("a constant that is not a packed value")?;
        let b = b.known_or_zero();
        Ok(self.resize(&self.konst(&b), w, b.signed()))
    }

    fn konst(&self, b: &Bits) -> Val {
        let w = b.width().max(1);
        let p = (0..pieces(w) as u32)
            .map(|k| {
                let lo = 32 * k;
                if lo >= b.width() { P::K(0) } else { P::K(b.slice(lo, 32.min(b.width() - lo)).to_u64().unwrap_or(0) as u32) }
            })
            .collect();
        Val { w, p }
    }

    fn unary(&mut self, op: UnOp, a: Val, ty: TypeId) -> Result<Val, String> {
        let w = self.d().types.width(ty);
        Ok(match op {
            UnOp::Plus => self.resize(&a, w, false),
            UnOp::Neg => {
                let a = self.resize(&a, w, false);
                self.add(&Val::zero(w), &a, true)
            }
            UnOp::Not => {
                let a = self.resize(&a, w, false);
                self.not(&a)
            }
            UnOp::LogNot => {
                let m = self.truth(&a);
                let m = self.mnot(m);
                self.bit(m, w)
            }
            UnOp::RedOr | UnOp::RedNor => {
                let mut m = self.truth(&a);
                if op == UnOp::RedNor {
                    m = self.mnot(m);
                }
                self.bit(m, w)
            }
            UnOp::RedAnd | UnOp::RedNand => {
                let ones = self.ones(a.w);
                let mut m = self.eq(&a, &ones);
                if op == UnOp::RedNand {
                    m = self.mnot(m);
                }
                self.bit(m, w)
            }
            UnOp::RedXor | UnOp::RedXnor => {
                let mut x = a.p[0];
                for &p in &a.p[1..] {
                    x = self.bin("xor", x, p);
                }
                for s in [16, 8, 4, 2, 1] {
                    let y = self.bin("lshr", x, P::K(s));
                    x = self.bin("xor", x, y);
                }
                let mut x = self.bin("and", x, P::K(1));
                if op == UnOp::RedXnor {
                    x = self.bin("xor", x, P::K(1));
                }
                let v = Val { w: 1, p: vec![x] };
                self.resize(&v, w, false)
            }
            _ => return Err(format!("{op:?} in an expression is not supported yet")),
        })
    }

    /// `op` on `a` and `b`; `ty` is the result's type, `at` the operands'.
    fn binary(&mut self, op: BinOp, a: Val, b: Val, ty: TypeId, at: TypeId) -> Result<Val, String> {
        let tys = &self.d().types;
        let w = tys.width(ty);
        let signed = tys.is_signed(at);
        Ok(match op {
            BinOp::And | BinOp::Or | BinOp::Xor | BinOp::Xnor => {
                let (a, b) = (self.resize(&a, w, false), self.resize(&b, w, false));
                let v = match op {
                    BinOp::And => self.bitwise("and", &a, &b),
                    BinOp::Or => self.bitwise("or", &a, &b),
                    _ => self.bitwise("xor", &a, &b),
                };
                if op == BinOp::Xnor { self.not(&v) } else { v }
            }
            BinOp::Add | BinOp::Sub => {
                let (a, b) = (self.resize(&a, w, false), self.resize(&b, w, false));
                self.add(&a, &b, op == BinOp::Sub)
            }
            BinOp::Mul if w <= 32 => {
                let (a, b) = (self.resize(&a, w, false), self.resize(&b, w, false));
                let p = self.bin("mul", a.p[0], b.p[0]);
                self.norm(Val { w, p: vec![p] })
            }
            BinOp::Shl | BinOp::AShl | BinOp::Shr | BinOp::AShr => {
                let a = self.resize(&a, w, false);
                let fill = (op == BinOp::AShr && tys.is_signed(ty)).then(|| self.sign_fill(&a));
                match b.is_const() {
                    Some(n) if b.p.len() <= 2 => {
                        if matches!(op, BinOp::Shl | BinOp::AShl) {
                            self.shl_const(&a, n)
                        } else {
                            self.lshr_const(&a, n, fill)
                        }
                    }
                    _ => {
                        let mut ok = M::K(true);
                        for &hp in &b.p[1..] {
                            let z = self.icmp("eq", hp, P::K(0));
                            ok = self.mand(ok, z);
                        }
                        let v = if matches!(op, BinOp::Shl | BinOp::AShl) { self.shl_var(&a, b.p[0]) } else { self.lshr_var(&a, b.p[0], fill) };
                        let big = match fill {
                            Some(f) => Val { w, p: vec![f; a.p.len()] },
                            None => Val::zero(w),
                        };
                        let big = self.norm(big);
                        self.select(ok, &v, &big)
                    }
                }
            }
            BinOp::Eq | BinOp::Ne | BinOp::CaseEq | BinOp::CaseNe => {
                let n = a.w.max(b.w);
                let (a, b) = (self.resize(&a, n, signed), self.resize(&b, n, signed));
                let mut m = self.eq(&a, &b);
                if matches!(op, BinOp::Ne | BinOp::CaseNe) {
                    m = self.mnot(m);
                }
                self.bit(m, w)
            }
            BinOp::Lt | BinOp::Le | BinOp::Gt | BinOp::Ge => {
                let n = a.w.max(b.w);
                let (a, b) = (self.resize(&a, n, signed), self.resize(&b, n, signed));
                let m = match op {
                    BinOp::Lt => self.lt(&a, &b, signed),
                    BinOp::Gt => self.lt(&b, &a, signed),
                    BinOp::Le => {
                        let m = self.lt(&b, &a, signed);
                        self.mnot(m)
                    }
                    _ => {
                        let m = self.lt(&a, &b, signed);
                        self.mnot(m)
                    }
                };
                self.bit(m, w)
            }
            BinOp::LogAnd | BinOp::LogOr | BinOp::Implies | BinOp::Equiv => {
                let (x, y) = (self.truth(&a), self.truth(&b));
                let m = match op {
                    BinOp::LogAnd => self.mand(x, y),
                    BinOp::LogOr => self.mor(x, y),
                    BinOp::Implies => {
                        let nx = self.mnot(x);
                        self.mor(nx, y)
                    }
                    _ => {
                        let ne = self.mxor(x, y);
                        self.mnot(ne)
                    }
                };
                self.bit(m, w)
            }
            _ => return Err(format!("{op:?} on values that are not constant is not supported yet")),
        })
    }

    // ---- values ----

    /// A mask as a `w`-bit 0 or 1.
    fn bit(&mut self, m: M, w: u32) -> Val {
        let p0 = match m {
            M::K(b) => P::K(b as u32),
            M::V(n) => {
                let t = self.tmp();
                let _ = writeln!(self.out, "  %t{t} = zext {M16} %t{n} to {V16}");
                self.mask_of.insert(t, m);
                if self.uniform.contains(&n) {
                    self.uniform.insert(t);
                }
                P::V(t)
            }
        };
        let mut p = vec![P::K(0); pieces(w)];
        p[0] = p0;
        Val { w, p }
    }

    fn ones(&self, w: u32) -> Val {
        let mut p = vec![P::K(!0); pieces(w)];
        *p.last_mut().unwrap() = P::K(top_mask(w));
        Val { w, p }
    }

    fn norm(&mut self, mut v: Val) -> Val {
        let t = top_mask(v.w);
        if t != !0 {
            let last = *v.p.last().unwrap();
            *v.p.last_mut().unwrap() = self.bin("and", last, P::K(t));
        }
        v
    }

    fn not(&mut self, a: &Val) -> Val {
        if a.w == 1
            && let P::V(t) = a.p[0]
            && let Some(&m) = self.mask_of.get(&t)
        {
            let n = self.mnot(m);
            return self.bit(n, 1);
        }
        let p = a.p.iter().map(|&x| self.bin("xor", x, P::K(!0))).collect();
        self.norm(Val { w: a.w, p })
    }

    fn bitwise(&mut self, op: &str, a: &Val, b: &Val) -> Val {
        let p = a.p.iter().zip(&b.p).map(|(&x, &y)| self.bin(op, x, y)).collect();
        Val { w: a.w, p }
    }

    /// `a + b`, or `a - b` as `a + ~b + 1`, in `a`'s width.
    fn add(&mut self, a: &Val, b: &Val, sub: bool) -> Val {
        let mut carry = P::K(sub as u32);
        let mut p = Vec::with_capacity(a.p.len());
        for (&x, &y) in a.p.iter().zip(&b.p) {
            let y = if sub { self.bin("xor", y, P::K(!0)) } else { y };
            let s1 = self.bin("add", x, y);
            let s = self.bin("add", s1, carry);
            if p.len() + 1 < a.p.len() {
                let c1 = self.icmp("ult", s1, x);
                let c2 = self.icmp("ult", s, s1);
                let c = self.mor(c1, c2);
                carry = self.bit(c, 32).p[0];
            }
            p.push(s);
        }
        self.norm(Val { w: a.w, p })
    }

    fn eq(&mut self, a: &Val, b: &Val) -> M {
        let mut m = M::K(true);
        for (&x, &y) in a.p.iter().zip(&b.p) {
            let e = self.icmp("eq", x, y);
            m = self.mand(m, e);
        }
        m
    }

    fn lt(&mut self, a: &Val, b: &Val, signed: bool) -> M {
        let n = a.p.len();
        let mut m = M::K(false);
        for k in 0..n {
            let (x, y, pred) = if k == n - 1 && signed {
                let sh = 32 - (a.w - 32 * (n as u32 - 1));
                let x = self.bin("shl", a.p[k], P::K(sh));
                let x = self.bin("ashr", x, P::K(sh));
                let y = self.bin("shl", b.p[k], P::K(sh));
                let y = self.bin("ashr", y, P::K(sh));
                (x, y, "slt")
            } else {
                (a.p[k], b.p[k], "ult")
            };
            let l = self.icmp(pred, x, y);
            let e = self.icmp("eq", x, y);
            let below = self.mand(e, m);
            m = self.mor(l, below);
        }
        m
    }

    fn truth(&mut self, a: &Val) -> M {
        let mut x = a.p[0];
        for &p in &a.p[1..] {
            x = self.bin("or", x, p);
        }
        if let P::V(t) = x
            && let Some(&m) = self.mask_of.get(&t)
        {
            return m;
        }
        self.icmp("ne", x, P::K(0))
    }

    /// All ones where the sign bit is set.
    fn sign_fill(&mut self, a: &Val) -> P {
        let top = *a.p.last().unwrap();
        let sh = 32 - (a.w - 32 * (a.p.len() as u32 - 1));
        let x = self.bin("shl", top, P::K(sh));
        self.bin("ashr", x, P::K(31))
    }

    /// `v` cut or extended to `w` bits.
    fn resize(&mut self, v: &Val, w: u32, signed: bool) -> Val {
        if w == v.w {
            return v.clone();
        }
        if w < v.w {
            let mut p = v.p[..pieces(w)].to_vec();
            p.truncate(pieces(w));
            return self.norm(Val { w, p });
        }
        let mut p = v.p.clone();
        if signed {
            let fill = self.sign_fill(v);
            let above = self.bin("and", fill, P::K(!top_mask(v.w)));
            let last = p.len() - 1;
            p[last] = self.bin("or", p[last], above);
            p.resize(pieces(w), fill);
        } else {
            p.resize(pieces(w), P::K(0));
        }
        self.norm(Val { w, p })
    }

    /// Bits `lo..lo + w` of `v`.
    fn extract(&mut self, v: &Val, lo: u32, w: u32) -> Val {
        // One bit: a test of it, which is a mask (vptestmd).
        if w == 1 && lo < v.w && matches!(v.p[(lo / 32) as usize], P::V(_)) {
            let x = self.bin("and", v.p[(lo / 32) as usize], P::K(1 << (lo % 32)));
            let m = self.icmp("ne", x, P::K(0));
            return self.bit(m, 1);
        }
        let s = self.lshr_const(v, lo as u64, None);
        self.resize(&s, w, false)
    }

    /// `v` with bits `lo..lo + w` replaced by `x`.
    fn insert(&mut self, v: &Val, lo: u32, w: u32, x: &Val) -> Val {
        if lo >= v.w {
            return v.clone();
        }
        let x = self.resize(x, w, false);
        let x = self.resize(&x, v.w, false);
        let x = self.shl_const(&x, lo as u64);
        let m = self.ones(w);
        let m = self.resize(&m, v.w, false);
        let m = self.shl_const(&m, lo as u64);
        let keep = self.not(&m);
        let kept = self.bitwise("and", v, &keep);
        self.bitwise("or", &kept, &x)
    }

    fn insert_var(&mut self, v: &Val, off: P, ok: M, w: u32, x: &Val) -> Val {
        let x = self.resize(x, w, false);
        let x = self.resize(&x, v.w, false);
        let x = self.shl_var(&x, off);
        let m = self.ones(w);
        let m = self.resize(&m, v.w, false);
        let m = self.shl_var(&m, off);
        let keep = self.not(&m);
        let kept = self.bitwise("and", v, &keep);
        let new = self.bitwise("or", &kept, &x);
        self.select(ok, &new, v)
    }

    fn shl_const(&mut self, v: &Val, n: u64) -> Val {
        if n >= v.w as u64 {
            return Val::zero(v.w);
        }
        let (q, r) = ((n / 32) as usize, (n % 32) as u32);
        let get = |k: isize| if k < 0 { P::K(0) } else { v.p[k as usize] };
        let mut p = Vec::with_capacity(v.p.len());
        for k in 0..v.p.len() as isize {
            let hi = self.bin("shl", get(k - q as isize), P::K(r));
            let lo = if r == 0 { P::K(0) } else { self.bin("lshr", get(k - q as isize - 1), P::K(32 - r)) };
            p.push(self.bin("or", hi, lo));
        }
        self.norm(Val { w: v.w, p })
    }

    /// `v >> n`, shifting in `fill` (zeros if `None`).
    fn lshr_const(&mut self, v: &Val, n: u64, fill: Option<P>) -> Val {
        let f = fill.unwrap_or(P::K(0));
        let len = v.p.len();
        let mut src = v.p.clone();
        if fill.is_some() {
            let above = self.bin("and", f, P::K(!top_mask(v.w)));
            src[len - 1] = self.bin("or", src[len - 1], above);
        }
        if n >= v.w as u64 {
            return self.norm(Val { w: v.w, p: vec![f; len] });
        }
        let (q, r) = ((n / 32) as usize, (n % 32) as u32);
        let get = |k: usize| if k < len { src[k] } else { f };
        let mut p = Vec::with_capacity(len);
        for k in 0..len {
            let lo = self.bin("lshr", get(k + q), P::K(r));
            let hi = if r == 0 { P::K(0) } else { self.bin("shl", get(k + q + 1), P::K(32 - r)) };
            p.push(self.bin("or", lo, hi));
        }
        self.norm(Val { w: v.w, p })
    }

    /// `v << s` for a shift in each lane; zero if `s >= v.w`.
    fn shl_var(&mut self, v: &Val, s: P) -> Val {
        if let P::K(n) = s {
            return self.shl_const(v, n as u64);
        }
        let len = v.p.len();
        let (q, r, ri, rz) = self.split_shift(s);
        let mut p = Vec::with_capacity(len);
        for k in 0..len {
            let mut acc = P::K(0);
            for t in 0..=k {
                let hi = self.bin("shl", v.p[k - t], r);
                let lo = if k > t {
                    let x = self.bin("lshr", v.p[k - t - 1], ri);
                    self.sel(rz, P::K(0), x)
                } else {
                    P::K(0)
                };
                let term = self.bin("or", hi, lo);
                let hit = self.icmp("eq", q, P::K(t as u32));
                acc = self.sel(hit, term, acc);
            }
            p.push(acc);
        }
        let big = self.icmp("uge", s, P::K(v.w));
        let v2 = self.norm(Val { w: v.w, p });
        let z = Val::zero(v.w);
        self.select(big, &z, &v2)
    }

    /// `v >> s` for a shift in each lane, shifting in `fill`.
    fn lshr_var(&mut self, v: &Val, s: P, fill: Option<P>) -> Val {
        if let P::K(n) = s {
            return self.lshr_const(v, n as u64, fill);
        }
        let f = fill.unwrap_or(P::K(0));
        let len = v.p.len();
        let mut src = v.p.clone();
        if fill.is_some() {
            let above = self.bin("and", f, P::K(!top_mask(v.w)));
            src[len - 1] = self.bin("or", src[len - 1], above);
        }
        let get = |k: usize| if k < len { src[k] } else { f };
        let (q, r, ri, rz) = self.split_shift(s);
        let mut p = Vec::with_capacity(len);
        for k in 0..len {
            // Whole pieces past the top bring in only the fill.
            let mut acc = f;
            for t in 0..len - k {
                let lo = self.bin("lshr", get(k + t), r);
                let x = self.bin("shl", get(k + t + 1), ri);
                let hi = self.sel(rz, P::K(0), x);
                let term = self.bin("or", lo, hi);
                let hit = self.icmp("eq", q, P::K(t as u32));
                acc = self.sel(hit, term, acc);
            }
            p.push(acc);
        }
        let big = self.icmp("uge", s, P::K(v.w));
        let v2 = self.norm(Val { w: v.w, p });
        let b = self.norm(Val { w: v.w, p: vec![f; len] });
        self.select(big, &b, &v2)
    }

    /// A shift's whole pieces, the bits within a piece, `32 - bits` (mod
    /// 32) and whether the bits are zero, so no shift is by 32 or more.
    fn split_shift(&mut self, s: P) -> (P, P, P, M) {
        let q = self.bin("lshr", s, P::K(5));
        let r = self.bin("and", s, P::K(31));
        let ri = self.bin("sub", P::K(32), r);
        let ri = self.bin("and", ri, P::K(31));
        let rz = self.icmp("eq", r, P::K(0));
        (q, r, ri, rz)
    }

    fn select(&mut self, m: M, a: &Val, b: &Val) -> Val {
        let p = a.p.iter().zip(&b.p).map(|(&x, &y)| self.sel(m, x, y)).collect();
        Val { w: a.w, p }
    }

    // ---- instructions, folding constants ----

    fn bin(&mut self, op: &str, a: P, b: P) -> P {
        // Logic on values made from masks is logic on the masks.
        if let ("and" | "or" | "xor", P::V(x), P::V(y)) = (op, a, b)
            && let (Some(&mx), Some(&my)) = (self.mask_of.get(&x), self.mask_of.get(&y))
        {
            let m = self.mlogic(op, mx, my);
            return self.bit(m, 1).p[0];
        }
        if let (P::K(x), P::K(y)) = (a, b) {
            return P::K(match op {
                "and" => x & y,
                "or" => x | y,
                "xor" => x ^ y,
                "add" => x.wrapping_add(y),
                "sub" => x.wrapping_sub(y),
                "mul" => x.wrapping_mul(y),
                "shl" => x.checked_shl(y).unwrap_or(0),
                "lshr" => x.checked_shr(y).unwrap_or(0),
                "ashr" => (x as i32).checked_shr(y).unwrap_or((x as i32) >> 31) as u32,
                _ => unreachable!("{op}"),
            });
        }
        match (op, a, b) {
            ("and", P::K(0), _) | ("and", _, P::K(0)) => return P::K(0),
            ("and", P::K(u32::MAX), x) | ("and", x, P::K(u32::MAX)) => return x,
            ("or" | "xor" | "add", P::K(0), x) | ("or" | "xor" | "add" | "sub" | "shl" | "lshr" | "ashr", x, P::K(0)) => {
                return x;
            }
            ("or", P::K(u32::MAX), _) | ("or", _, P::K(u32::MAX)) => return P::K(!0),
            ("shl" | "lshr", P::K(0), _) => return P::K(0),
            ("shl" | "lshr", _, P::K(n)) if n >= 32 => return P::K(0),
            ("and" | "or", x, y) if x == y => return x,
            ("xor", x, y) if x == y => return P::K(0),
            _ => {}
        }
        let t = self.tmp();
        let _ = writeln!(self.out, "  %t{t} = {op} {V16} {}, {}", ps(a), ps(b));
        if self.is_uniform(a) && self.is_uniform(b) {
            self.uniform.insert(t);
        }
        P::V(t)
    }

    fn icmp(&mut self, pred: &str, a: P, b: P) -> M {
        if let (P::V(x), P::K(k @ (0 | 1)), "eq" | "ne") = (a, b, pred)
            && let Some(&m) = self.mask_of.get(&x)
        {
            return if (k == 1) == (pred == "eq") { m } else { self.mnot(m) };
        }
        if let (P::K(x), P::K(y)) = (a, b) {
            return M::K(match pred {
                "eq" => x == y,
                "ne" => x != y,
                "ult" => x < y,
                "ule" => x <= y,
                "ugt" => x > y,
                "uge" => x >= y,
                "slt" => (x as i32) < (y as i32),
                "sle" => (x as i32) <= (y as i32),
                "sgt" => (x as i32) > (y as i32),
                "sge" => (x as i32) >= (y as i32),
                _ => unreachable!("{pred}"),
            });
        }
        match (pred, b) {
            ("uge", P::K(0)) => return M::K(true),
            ("ult", P::K(0)) => return M::K(false),
            _ => {}
        }
        if a == b {
            return M::K(matches!(pred, "eq" | "ule" | "uge" | "sle" | "sge"));
        }
        let t = self.tmp();
        let _ = writeln!(self.out, "  %t{t} = icmp {pred} {V16} {}, {}", ps(a), ps(b));
        if self.is_uniform(a) && self.is_uniform(b) {
            self.uniform.insert(t);
        }
        M::V(t)
    }

    fn sel(&mut self, m: M, a: P, b: P) -> P {
        match m {
            M::K(true) => a,
            M::K(false) => b,
            M::V(_) if a == b => a,
            M::V(n) => {
                let t = self.tmp();
                let _ = writeln!(self.out, "  %t{t} = select {M16} %t{n}, {V16} {}, {V16} {}", ps(a), ps(b));
                if self.uniform.contains(&n) && self.is_uniform(a) && self.is_uniform(b) {
                    self.uniform.insert(t);
                }
                P::V(t)
            }
        }
    }

    fn mlogic(&mut self, op: &str, a: M, b: M) -> M {
        if let (M::K(x), M::K(y)) = (a, b) {
            return M::K(match op {
                "and" => x & y,
                "or" => x | y,
                _ => x ^ y,
            });
        }
        match (op, a, b) {
            ("and", M::K(false), _) | ("and", _, M::K(false)) => return M::K(false),
            ("and", M::K(true), x) | ("and", x, M::K(true)) => return x,
            ("or", M::K(true), _) | ("or", _, M::K(true)) => return M::K(true),
            ("or" | "xor", M::K(false), x) | ("or" | "xor", x, M::K(false)) => return x,
            _ => {}
        }
        // Masks from bit planes: the logic on their 16 bits, a scalar
        // instruction, and a mask of that only for what needs one.
        let bits = |f: &Self, m: M| match m {
            M::K(b) => Some(format!("{}", if b { -1 } else { 0 })),
            M::V(n) => f.bits_of.get(&n).map(|i| format!("%t{i}")),
        };
        if let (Some(x), Some(y)) = (bits(self, a), bits(self, b)) {
            let (i, t) = (self.tmp(), self.tmp());
            let _ = writeln!(self.out, "  %t{i} = {op} i16 {x}, {y}\n  %t{t} = bitcast i16 %t{i} to {M16}");
            self.bits_of.insert(t, i);
            return M::V(t);
        }
        let t = self.tmp();
        let _ = writeln!(self.out, "  %t{t} = {op} {M16} {}, {}", ms(a), ms(b));
        M::V(t)
    }

    fn mand(&mut self, a: M, b: M) -> M {
        self.mlogic("and", a, b)
    }

    fn mor(&mut self, a: M, b: M) -> M {
        self.mlogic("or", a, b)
    }

    fn mxor(&mut self, a: M, b: M) -> M {
        self.mlogic("xor", a, b)
    }

    fn mnot(&mut self, a: M) -> M {
        self.mlogic("xor", a, M::K(true))
    }
}

/// A select's bit offset: a constant, a constant below zero (part selects
/// that start out of range), or one per lane with whether it is in range.
#[derive(Clone, Copy)]
enum Off {
    K(u32),
    Neg(u32),
    V(P, M),
}

impl Off {
    fn with_sign(self, off: i64) -> Off {
        if off < 0 { Off::Neg((-off) as u32) } else { self }
    }

    fn var(self) -> (P, M) {
        match self {
            Off::K(k) => (P::K(k), M::K(true)),
            Off::Neg(_) => (P::K(0), M::K(false)),
            Off::V(p, m) => (p, m),
        }
    }
}

fn stmt_name(k: &StmtKind) -> &'static str {
    match k {
        StmtKind::Block { .. } => "fork",
        StmtKind::Assign { .. } => "assignment with timing",
        StmtKind::Forever(_) => "forever",
        StmtKind::Repeat(..) => "repeat",
        StmtKind::While(..) => "while",
        StmtKind::DoWhile(..) => "do-while",
        StmtKind::Foreach { .. } => "foreach",
        StmtKind::Timed(..) => "timing control",
        StmtKind::Wait(..) => "wait",
        StmtKind::Return(_) => "return",
        StmtKind::Break => "break",
        StmtKind::Continue => "continue",
        _ => "this statement",
    }
}

fn expr_name(k: &ExprKind) -> &'static str {
    match k {
        ExprKind::Hier { .. } => "a hierarchical reference",
        ExprKind::Call(_) => "a function call",
        ExprKind::SysCall(_) => "a system function call",
        ExprKind::Method(_) => "a method call",
        ExprKind::Streaming { .. } => "a streaming concatenation",
        ExprKind::Pattern => "an assignment pattern",
        ExprKind::Inside => "inside",
        _ => "this expression",
    }
}
