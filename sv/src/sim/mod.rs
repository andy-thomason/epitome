//! The interpreter (`docs/design/sv.md` section 8.2).
//!
//! Building a simulation is three linear passes over the design: a frame
//! layout per specialisation, one bytecode body per process and
//! subroutine, and a tree of runtime instances, each with a frame base in
//! one value store. Nothing is compiled to machine code.
//!
//! The scheduler follows clause 4.5 with the regions that matter without
//! PLI: Active, Inactive, NBA and Postponed, with the reactive set run as
//! the active one for now.

mod code;
mod eval;
mod sys;

use crate::ast::Ast;
use crate::design::types::{Type, TypeId, Types, UDim};
use crate::design::value::{Bits, Logic};
use crate::design::*;
use code::{Compiler, Op};
use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap, VecDeque};
use std::io::Write;
use std::rc::Rc;

/// Limits and inputs for a run.
pub struct Options {
    /// Stop after this many operations, as a guard against endless loops.
    pub max_steps: u64,
    /// `+name` and `+name=value` arguments (21.6).
    pub plusargs: Vec<String>,
    /// Stop after this much wall-clock time.
    pub time_limit: Option<std::time::Duration>,
}

impl Default for Options {
    fn default() -> Self {
        Options { max_steps: 200_000_000, plusargs: vec![], time_limit: None }
    }
}

/// How a run ended.
#[derive(Debug, Default)]
pub struct Outcome {
    /// `$finish` or `$stop` was called (rather than running out of events).
    pub finished: bool,
    /// `$fatal` was called, or the run hit a limit or an unsupported
    /// construct it could not step over.
    pub failed: bool,
    pub errors: u32,
    /// The time the run ended, in the global precision.
    pub time: u64,
    /// Constructs that were not run because they are not modelled yet.
    pub unsupported: Vec<String>,
}

/// Runs a design until it finishes or runs out of events.
pub fn simulate(d: &Design, ast: &Ast, out: &mut dyn Write, opts: &Options) -> Outcome {
    let mut s = Sim::new(d, ast, out, opts);
    s.run();
    s.outcome
}

/// Where an object's value lives.
#[derive(Clone, Copy, Debug)]
enum Home {
    /// In its specialisation's frame, at this offset.
    Frame(SpecId, u32),
    /// In the global part of the store (packages, the compilation unit,
    /// static class properties).
    Global(u32),
    /// In the current call frame (automatic variables).
    Local,
    /// A class property, at this index of the object.
    Field(u32),
    /// A parameter or enum constant.
    Const,
}

/// A runtime instance: one per instance in the elaborated hierarchy,
/// expanded so that instances of shared specialisations are distinct.
struct RInst {
    spec: SpecId,
    parent: Option<u32>,
    elab: InstId,
    base: u32,
}

/// The automatic variables of a call or block, shared by the branches a
/// fork makes.
type Locals = Rc<RefCell<Vec<(ObjId, Const)>>>;

/// What an evaluation runs in: the instance, the call frame's locals and
/// the class object `this`.
#[derive(Clone)]
struct Cx {
    inst: u32,
    locals: Locals,
    this: Option<u32>,
}

struct Frame {
    cx: Cx,
    /// Where to go back to, and for a task call, the call (to copy
    /// outputs back).
    ret: Option<(u32, ExprId)>,
    sub: Option<SubId>,
    /// `repeat` counts and `foreach` indices, by the op that uses them.
    counters: HashMap<u32, i64>,
    iters: HashMap<u32, (Vec<Vec<i64>>, usize)>,
    ret_value: Option<Const>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum State {
    Ready,
    Waiting,
    Done,
}

/// Why a process is waiting.
#[derive(Clone)]
enum Wait {
    None,
    /// For time to pass.
    Time,
    /// For one of these event terms, with the values they had.
    Event(Vec<(EventTerm, Const)>, Cx),
    /// For a condition.
    Cond(ExprId, Cx),
    /// For any change of what it is sensitive to.
    Change,
    /// For forked children: how many must still finish.
    Join(u32),
    /// For every child to finish (`wait fork`).
    AllChildren,
}

struct Proc {
    pc: u32,
    frames: Vec<Frame>,
    state: State,
    /// Bumped on each wait, so stale entries in watch lists are ignored.
    epoch: u32,
    wait: Wait,
    parent: Option<u32>,
    children: Vec<u32>,
    /// The objects an `always_comb` or `always_latch` reads, or the
    /// driver's inputs, re-resolved when it waits.
    sensitive: Vec<(ObjId, Option<ExprId>)>,
    /// A continuous driver: which one.
    driver: Option<Driven>,
}

/// What a driver process computes.
#[derive(Clone, Copy)]
enum Driven {
    /// A continuous assignment or net declaration assignment.
    Assign(DriverId),
    /// A port connection: the instance and index of the connection.
    Port(u32, usize),
    /// A gate.
    Cell(CellId),
}

/// Where a write goes: a place and the path of selects within its value.
#[derive(Clone, Debug)]
enum Place {
    Slot(usize),
    Local(ObjId),
    Field(u32, u32),
    None,
}

#[derive(Clone, Debug)]
enum Sel {
    /// An element of an unpacked array, by position.
    Elem(usize),
    /// A key of an associative array.
    Key(Const),
    /// Bits of a packed value: the low bit and the width.
    Bits(u32, u32),
    /// A member of an unpacked struct.
    Member(u32),
}

#[derive(Clone, Debug)]
struct Target {
    place: Place,
    path: Vec<Sel>,
    /// The type at the end of the path, for conversions.
    ty: TypeId,
}

/// An entry in the time wheel.
enum Timed {
    Wake(u32, u32),
    Nba(Target, Const, Cx),
}

/// A class object.
struct Object {
    class: ClassId,
    fields: Vec<Const>,
}

/// `$monitor`'s arguments and what it last printed.
struct Monitor {
    cx: Cx,
    args: Vec<ExprId>,
    last: Option<String>,
    radix: u8,
}

pub(crate) struct Sim<'d, 'a, 'o> {
    d: &'d Design<'a>,
    ast: &'d Ast<'a>,
    out: &'o mut dyn Write,
    opts: &'d Options,
    code: Vec<Op>,
    proc_entry: HashMap<ProcId, u32>,
    sub_entry: HashMap<SubId, u32>,
    homes: Vec<Home>,
    spec_size: Vec<u32>,
    globals: u32,
    insts: Vec<RInst>,
    /// Children of each runtime instance, by the elaborated instance.
    kids: HashMap<(u32, InstId), u32>,
    vals: Vec<Const>,
    canon: Vec<u32>,
    watch: Vec<Vec<(u32, u32)>>,
    /// When each named event last triggered.
    triggered: HashMap<usize, u64>,
    procs: Vec<Proc>,
    active: VecDeque<u32>,
    inactive: Vec<u32>,
    nba: Vec<(Target, Const, Cx)>,
    wheel: BTreeMap<u64, Vec<Timed>>,
    time: u64,
    /// The global time precision, as a power of ten of a second (3.14.3).
    precision: i8,
    heap: Vec<Object>,
    class_fields: HashMap<ClassId, Vec<ObjId>>,
    monitors: Vec<Monitor>,
    monitor_on: bool,
    strobes: Vec<(Cx, Vec<ExprId>, u8)>,
    finals: Vec<(ProcId, u32)>,
    steps: u64,
    started: std::time::Instant,
    rng: u64,
    finishing: bool,
    outcome: Outcome,
    timeformat: (i8, usize, String, usize),
    /// Files opened with `$fopen`, by descriptor.
    files: HashMap<u32, String>,
    file_text: HashMap<u32, Vec<u8>>,
    next_fd: u32,
    /// Blocking writes delayed by intra-assignment timing, done when the
    /// process resumes.
    pending: HashMap<u32, (Target, Const, Cx)>,
    /// The index of each array-method iterator in use (7.12.4).
    iter_index: HashMap<ObjId, i64>,
}

impl<'d, 'a, 'o> Sim<'d, 'a, 'o> {
    fn new(d: &'d Design<'a>, ast: &'d Ast<'a>, out: &'o mut dyn Write, opts: &'d Options) -> Self {
        let mut s = Sim {
            d,
            ast,
            out,
            opts,
            code: Vec::new(),
            proc_entry: HashMap::new(),
            sub_entry: HashMap::new(),
            homes: Vec::new(),
            spec_size: vec![0; d.specs.len()],
            globals: 0,
            insts: Vec::new(),
            kids: HashMap::new(),
            vals: Vec::new(),
            canon: Vec::new(),
            watch: Vec::new(),
            triggered: HashMap::new(),
            procs: Vec::new(),
            active: VecDeque::new(),
            inactive: Vec::new(),
            nba: Vec::new(),
            wheel: BTreeMap::new(),
            time: 0,
            precision: d.unit_time.1,
            heap: Vec::new(),
            class_fields: HashMap::new(),
            monitors: Vec::new(),
            monitor_on: true,
            strobes: Vec::new(),
            finals: Vec::new(),
            steps: 0,
            started: std::time::Instant::now(),
            rng: 0x2545_f491_4f6c_dd1d,
            finishing: false,
            outcome: Outcome::default(),
            timeformat: (0, 0, String::new(), 20),
            files: HashMap::new(),
            file_text: HashMap::new(),
            next_fd: 3,
            pending: HashMap::new(),
            iter_index: HashMap::new(),
        };
        for sp in &d.specs {
            s.precision = s.precision.min(sp.time_precision);
        }
        s.timeformat.0 = s.precision;
        s.layout();
        s.compile();
        s.instantiate();
        s
    }

    fn types(&self) -> &'d Types {
        &self.d.types
    }

    // ---- layout ----

    /// Gives every object its home (section 8.2, step 1).
    fn layout(&mut self) {
        let d = self.d;
        let spec_of_scope: HashMap<ScopeId, SpecId> = d.specs.iter().enumerate().map(|(i, s)| (s.scope, SpecId(i as u32))).collect();
        // Class fields: base first.
        for (i, c) in d.classes.iter().enumerate() {
            let _ = c;
            self.fields_of(ClassId(i as u32));
        }
        for (i, o) in d.objects.iter().enumerate() {
            let home = match o.kind {
                ObjKind::Param { .. } | ObjKind::EnumConst | ObjKind::Genvar => Home::Const,
                _ => {
                    // The nearest class or specialisation above it.
                    let mut cur = Some(o.scope);
                    let mut home = None;
                    while let Some(sc) = cur {
                        let scope = d.scope(sc);
                        if scope.kind == ScopeKind::Class {
                            let is_static = o.qualifiers.iter().any(|&q| d.name(q) == "static");
                            if o.lifetime == Lifetime::Automatic && !is_static && o.scope != sc {
                                home = Some(Home::Local);
                            } else if is_static {
                                home = None;
                            } else {
                                let c = ClassId(d.classes.iter().position(|c| c.scope == sc).unwrap() as u32);
                                let idx = self.class_fields[&c].iter().position(|&f| f == ObjId(i as u32));
                                home = Some(idx.map_or(Home::Local, |x| Home::Field(x as u32)));
                            }
                            break;
                        }
                        if o.lifetime == Lifetime::Automatic {
                            home = Some(Home::Local);
                            break;
                        }
                        if scope.kind == ScopeKind::Spec {
                            let spec = spec_of_scope[&sc];
                            let off = self.spec_size[spec.0 as usize];
                            self.spec_size[spec.0 as usize] += 1;
                            home = Some(Home::Frame(spec, off));
                            break;
                        }
                        cur = scope.parent;
                    }
                    match home {
                        Some(h) => h,
                        None => {
                            let g = self.globals;
                            self.globals += 1;
                            Home::Global(g)
                        }
                    }
                }
            };
            self.homes.push(home);
        }
    }

    /// A class's properties, its bases' first.
    fn fields_of(&mut self, c: ClassId) -> Vec<ObjId> {
        if let Some(f) = self.class_fields.get(&c) {
            return f.clone();
        }
        let class = &self.d.classes[c.0 as usize];
        let mut f = match class.base {
            Some(b) => self.fields_of(b),
            None => vec![],
        };
        for item in &self.d.scope(class.scope).items {
            if let Item::Object(o) = *item {
                let obj = self.d.object(o);
                if obj.kind == ObjKind::Variable && !obj.qualifiers.iter().any(|&q| self.d.name(q) == "static") {
                    f.push(o);
                }
            }
        }
        self.class_fields.insert(c, f.clone());
        f
    }

    // ---- code ----

    fn compile(&mut self) {
        let mut c = Compiler::new(self.d);
        for (i, p) in self.d.processes.iter().enumerate() {
            let at = c.process(p);
            self.proc_entry.insert(ProcId(i as u32), at);
        }
        for (i, s) in self.d.subroutines.iter().enumerate() {
            if !s.body.is_empty() || s.qualifiers.iter().all(|&q| self.d.name(q) != "import") {
                let at = c.sub(s);
                self.sub_entry.insert(SubId(i as u32), at);
            }
        }
        self.code = c.code;
    }

    // ---- instances ----

    fn instantiate(&mut self) {
        // The global part of the store comes first.
        let globals = self.globals as usize;
        self.vals = vec![Const::Null; globals];
        for (i, o) in self.d.objects.iter().enumerate() {
            if let Home::Global(g) = self.homes[i] {
                self.vals[g as usize] = crate::design::ops::default_value(self.types(), o.ty);
            }
        }
        for &t in &self.d.tops.clone() {
            self.add_inst(t, None);
        }
        self.canon = (0..self.vals.len() as u32).collect();
        self.watch = vec![Vec::new(); self.vals.len()];
        // Ports that can share their outside net's value do (4.9.6).
        for i in 0..self.insts.len() as u32 {
            self.alias_ports(i);
        }
        // Static variables start at their initial values before any
        // process (6.8, 10.5); packages first.
        let unit_cx = Cx { inst: u32::MAX, locals: Rc::default(), this: None };
        for (i, o) in self.d.objects.iter().enumerate() {
            if let (Home::Global(_), Some(init)) = (self.homes[i], o.init)
                && o.kind == ObjKind::Variable
            {
                let v = self.eval(&unit_cx, init);
                let t = self.target_obj(&unit_cx, ObjId(i as u32));
                self.write(&t, v, false);
            }
        }
        for i in 0..self.insts.len() as u32 {
            self.init_inst(i);
        }
        for i in 0..self.insts.len() as u32 {
            self.start_inst(i);
        }
    }

    fn add_inst(&mut self, elab: InstId, parent: Option<u32>) -> u32 {
        let spec = self.d.instance(elab).spec;
        let id = self.insts.len() as u32;
        let base = self.vals.len() as u32;
        let size = self.spec_size[spec.0 as usize];
        self.insts.push(RInst { spec, parent, elab, base });
        // Objects of the frame start at their types' defaults.
        self.vals.resize(base as usize + size as usize, Const::Null);
        for (i, o) in self.d.objects.iter().enumerate() {
            if let Home::Frame(sp, off) = self.homes[i]
                && sp == spec
            {
                self.vals[(base + off) as usize] = crate::design::ops::default_value(self.types(), o.ty);
            }
        }
        if let Some(p) = parent {
            self.kids.insert((p, elab), id);
        }
        let scope = self.d.spec(spec).scope;
        self.add_children(scope, id);
        id
    }

    fn add_children(&mut self, scope: ScopeId, inst: u32) {
        for item in self.d.scope(scope).items.clone() {
            match item {
                Item::Instance(c) => {
                    self.add_inst(c, Some(inst));
                }
                Item::Scope(s) => self.add_children(s, inst),
                _ => {}
            }
        }
    }

    /// Ports whose outside connection is a whole net or variable of the
    /// same width share its slot; the rest become drivers.
    fn alias_ports(&mut self, i: u32) {
        let Some(parent) = self.insts[i as usize].parent else { return };
        let elab = self.insts[i as usize].elab;
        let spec = self.insts[i as usize].spec;
        let conns = self.d.instance(elab).connections.clone();
        let ports = self.d.spec(spec).ports.clone();
        for (k, c) in conns.iter().enumerate() {
            let Some(port) = ports.iter().find(|p| p.name == c.port) else { continue };
            let Some(po) = port.object else { continue };
            let Some(x) = c.expr else { continue };
            let inner = self.slot_of_obj(i, po);
            let outer = match &self.d.expr(x).kind {
                ExprKind::Object(o) => self.slot_of_obj(parent, *o),
                _ => None,
            };
            let same = matches!(&self.d.expr(x).kind, ExprKind::Object(o)
                if self.d.types.width(self.d.object(*o).ty) == self.d.types.width(self.d.object(po).ty)
                    && self.d.types.is_four_state(self.d.object(*o).ty) == self.d.types.is_four_state(self.d.object(po).ty));
            match (inner, outer) {
                (Some(a), Some(b)) if same => {
                    let (a, b) = (self.find(a), self.find(b));
                    if a != b {
                        self.canon[a] = b as u32;
                    }
                }
                _ => {
                    // A driver in one direction or the other.
                    let p = self.new_proc(i, None, Some(Driven::Port(i, k)));
                    let _ = p;
                }
            }
        }
    }

    fn find(&self, mut s: usize) -> usize {
        while self.canon[s] as usize != s {
            s = self.canon[s] as usize;
        }
        s
    }

    /// Initial values of the instance's static variables, and of nets'
    /// declaration assignments (which are drivers).
    fn init_inst(&mut self, i: u32) {
        let spec = self.insts[i as usize].spec;
        let cx = Cx { inst: i, locals: Rc::default(), this: None };
        for (k, o) in self.d.objects.iter().enumerate() {
            if let (Home::Frame(sp, _), Some(init)) = (self.homes[k], o.init)
                && sp == spec
                && o.kind == ObjKind::Variable
                && o.dir.is_none_or(|d| d != Dir::Input)
            {
                let v = self.eval(&cx, init);
                let t = self.target_obj(&cx, ObjId(k as u32));
                self.write(&t, v, false);
            }
        }
    }

    /// The instance's processes, drivers and gates.
    fn start_inst(&mut self, i: u32) {
        let spec = self.insts[i as usize].spec;
        let scope = self.d.spec(spec).scope;
        self.start_scope(scope, i);
    }

    fn start_scope(&mut self, scope: ScopeId, i: u32) {
        for item in self.d.scope(scope).items.clone() {
            match item {
                Item::Process(p) => {
                    let pr = &self.d.processes[p.0 as usize];
                    if pr.kind == ProcKind::Final {
                        self.finals.push((p, i));
                        continue;
                    }
                    let id = self.new_proc(i, Some(p), None);
                    let sens: Vec<(ObjId, Option<ExprId>)> = pr.sensitivity.iter().map(|&o| (o, None)).collect();
                    self.procs[id as usize].sensitive = sens;
                }
                Item::Driver(dr) => {
                    self.new_proc(i, None, Some(Driven::Assign(dr)));
                }
                Item::Cell(c) => {
                    self.new_proc(i, None, Some(Driven::Cell(c)));
                }
                Item::Scope(s) => self.start_scope(s, i),
                _ => {}
            }
        }
    }

    fn new_proc(&mut self, inst: u32, p: Option<ProcId>, driver: Option<Driven>) -> u32 {
        let id = self.procs.len() as u32;
        let pc = p.map_or(u32::MAX, |p| self.proc_entry[&p]);
        let cx = Cx { inst, locals: Rc::default(), this: None };
        self.procs.push(Proc {
            pc,
            frames: vec![Frame { cx, ret: None, sub: None, counters: HashMap::new(), iters: HashMap::new(), ret_value: None }],
            state: State::Ready,
            epoch: 0,
            wait: Wait::None,
            parent: None,
            children: vec![],
            sensitive: vec![],
            driver,
        });
        self.active.push_back(id);
        id
    }

    // ---- slots ----

    /// The runtime instance holding objects of `spec`, from `inst`: itself
    /// or the nearest ancestor of that specialisation (a nested module
    /// sees its parent's names).
    fn inst_for(&self, mut inst: u32, spec: SpecId) -> Option<u32> {
        loop {
            if inst == u32::MAX {
                return None;
            }
            let r = &self.insts[inst as usize];
            if r.spec == spec {
                return Some(inst);
            }
            inst = r.parent?;
        }
    }

    fn slot_of_obj(&self, inst: u32, o: ObjId) -> Option<usize> {
        match self.homes[o.0 as usize] {
            Home::Frame(spec, off) => {
                let i = self.inst_for(inst, spec)?;
                Some((self.insts[i as usize].base + off) as usize)
            }
            Home::Global(g) => Some(g as usize),
            _ => None,
        }
    }

    /// The instance a hierarchical reference reaches (section 8.2).
    fn hops(&self, inst: u32, x: ExprId) -> Option<u32> {
        let hops = self.d.hops.get(&x)?;
        let mut at = inst;
        for h in hops {
            at = match *h {
                Hop::Down(c) => *self.kids.get(&(at, c))?,
                Hop::Top(t) => self.insts.iter().position(|r| r.elab == t && r.parent.is_none())? as u32,
                Hop::Up(name) => {
                    let mut cur = Some(at);
                    let mut found = None;
                    while let Some(c) = cur {
                        let r = &self.insts[c as usize];
                        if self.d.instance(r.elab).name == name || self.d.spec(r.spec).name == name {
                            found = Some(c);
                            break;
                        }
                        cur = r.parent;
                    }
                    found?
                }
            };
        }
        Some(at)
    }

    /// The hierarchical name of an instance, for `%m`.
    fn inst_path(&self, inst: u32) -> String {
        if inst == u32::MAX {
            return "$unit".into();
        }
        let mut names = Vec::new();
        let mut cur = Some(inst);
        while let Some(c) = cur {
            let r = &self.insts[c as usize];
            let i = self.d.instance(r.elab);
            let mut n = self.d.name(i.name).to_string();
            if let Some(ix) = i.index {
                n += &format!("[{ix}]");
            }
            names.push(n);
            cur = r.parent;
        }
        names.reverse();
        names.join(".")
    }

    // ---- reading and writing ----

    fn read_slot(&self, s: usize) -> Const {
        self.vals[self.find(s)].clone()
    }

    fn read_local(&self, cx: &Cx, o: ObjId) -> Const {
        let l = cx.locals.borrow();
        match l.iter().rev().find(|(k, _)| *k == o) {
            Some((_, v)) => v.clone(),
            None => crate::design::ops::default_value(self.types(), self.d.object(o).ty),
        }
    }

    /// The target of an object written by name.
    fn target_obj(&self, cx: &Cx, o: ObjId) -> Target {
        let ty = self.d.object(o).ty;
        let place = match self.homes[o.0 as usize] {
            Home::Local => Place::Local(o),
            Home::Field(f) => match cx.this {
                Some(h) => Place::Field(h, f),
                None => Place::None,
            },
            _ => match self.slot_of_obj(cx.inst, o) {
                Some(s) => Place::Slot(s),
                None => Place::None,
            },
        };
        Target { place, path: vec![], ty }
    }

    /// Reads the whole value at a place.
    fn read_place(&self, cx: &Cx, p: &Place) -> Const {
        match p {
            Place::Slot(s) => self.read_slot(*s),
            Place::Local(o) => self.read_local(cx, *o),
            Place::Field(h, f) => self.heap.get(*h as usize).and_then(|o| o.fields.get(*f as usize).cloned()).unwrap_or(Const::Null),
            Place::None => Const::Null,
        }
    }

    /// Writes `v` to a target: the value at the end of its path is
    /// replaced, converted to the target's type. Watchers of a slot that
    /// changes are woken, unless `quiet`.
    fn write_cx(&mut self, cx: &Cx, t: &Target, v: Const, quiet: bool) {
        let v = crate::design::ops::convert(self.types(), &v, t.ty);
        let old = self.read_place(cx, &t.place);
        let new = if t.path.is_empty() { v } else { update(self.types(), old.clone(), &t.path, v) };
        match &t.place {
            Place::Slot(s) => {
                let s = self.find(*s);
                let changed = !same(&self.vals[s], &new);
                self.vals[s] = new;
                if changed && !quiet {
                    self.changed(s);
                }
            }
            Place::Local(o) => {
                let mut l = cx.locals.borrow_mut();
                match l.iter_mut().rev().find(|(k, _)| k == o) {
                    Some(e) => e.1 = new,
                    None => l.push((*o, new)),
                }
            }
            Place::Field(h, f) => {
                if let Some(obj) = self.heap.get_mut(*h as usize)
                    && let Some(slot) = obj.fields.get_mut(*f as usize)
                {
                    *slot = new;
                }
            }
            Place::None => {}
        }
    }

    fn write(&mut self, t: &Target, v: Const, quiet: bool) {
        let cx = Cx { inst: u32::MAX, locals: Rc::default(), this: None };
        self.write_cx(&cx, t, v, quiet);
    }

    /// Wakes what watches slot `s`.
    fn changed(&mut self, s: usize) {
        let list = std::mem::take(&mut self.watch[s]);
        let mut keep = Vec::new();
        for (p, epoch) in list {
            let pr = &self.procs[p as usize];
            if pr.epoch != epoch || pr.state != State::Waiting {
                continue;
            }
            if self.wakes(p) {
                self.wake(p);
            } else {
                keep.push((p, epoch));
            }
        }
        // Anything added while waking goes after the survivors.
        keep.append(&mut self.watch[s]);
        self.watch[s] = keep;
    }

    /// Whether a change wakes process `p`: an edge it waits for, a
    /// condition that became true, or any change.
    fn wakes(&mut self, p: u32) -> bool {
        match self.procs[p as usize].wait.clone() {
            Wait::Change => true,
            Wait::Cond(c, cx) => {
                let v = self.eval(&cx, c);
                truth(&v) == Some(true)
            }
            Wait::Event(terms, cx) => {
                let mut hit = false;
                let mut updated = Vec::new();
                for (t, old) in terms {
                    let new = self.eval(&cx, t.expr);
                    let fired = edge(t.edge, &old, &new);
                    let iff = match t.iff {
                        Some(i) => {
                            let v = self.eval(&cx, i);
                            truth(&v) == Some(true)
                        }
                        None => true,
                    };
                    if fired && iff {
                        hit = true;
                    }
                    updated.push((t, new));
                }
                if !hit {
                    self.procs[p as usize].wait = Wait::Event(updated, cx);
                }
                hit
            }
            _ => false,
        }
    }

    fn wake(&mut self, p: u32) {
        let pr = &mut self.procs[p as usize];
        if pr.state == State::Waiting {
            pr.state = State::Ready;
            pr.epoch = pr.epoch.wrapping_add(1);
            pr.wait = Wait::None;
            self.active.push_back(p);
        }
    }

    /// Makes `p` wait on the slots an expression reads.
    fn watch_expr(&mut self, p: u32, cx: &Cx, x: ExprId) {
        let mut slots = Vec::new();
        self.reads(cx, x, &mut slots);
        let epoch = self.procs[p as usize].epoch;
        for s in slots {
            let s = self.find(s);
            if !self.watch[s].contains(&(p, epoch)) {
                self.watch[s].push((p, epoch));
            }
        }
    }

    fn watch_slot(&mut self, p: u32, s: usize) {
        let epoch = self.procs[p as usize].epoch;
        let s = self.find(s);
        if !self.watch[s].contains(&(p, epoch)) {
            self.watch[s].push((p, epoch));
        }
    }

    /// The slots an expression reads, for waiting on.
    fn reads(&self, cx: &Cx, x: ExprId, out: &mut Vec<usize>) {
        let e = self.d.expr(x);
        match &e.kind {
            ExprKind::Object(o) => {
                if let Some(s) = self.slot_of_obj(cx.inst, *o) {
                    out.push(s);
                }
            }
            ExprKind::Hier { object: Some(o), .. } => {
                if let Some(i) = self.hops(cx.inst, x)
                    && let Some(s) = self.slot_of_obj(i, *o)
                {
                    out.push(s);
                }
            }
            ExprKind::Call(sub) => {
                // What the function reads that lives outside it.
                for &b in &self.d.sub(*sub).body {
                    self.stmt_reads(cx, b, out, 0);
                }
            }
            _ => {}
        }
        for &a in &e.args {
            self.reads(cx, a, out);
        }
    }

    fn stmt_reads(&self, cx: &Cx, s: StmtId, out: &mut Vec<usize>, depth: u32) {
        if depth > 8 {
            return;
        }
        let mut xs = Vec::new();
        stmt_exprs(self.d, s, &mut xs);
        for x in xs {
            self.reads(cx, x, out);
        }
    }

    // ---- running ----

    fn run(&mut self) {
        loop {
            self.slot();
            if self.finishing {
                break;
            }
            // Postponed: `$strobe` and `$monitor` (4.4.2.9).
            self.postponed();
            if self.finishing {
                break;
            }
            let Some((&t, _)) = self.wheel.iter().next() else { break };
            let entries = self.wheel.remove(&t).unwrap();
            self.time = t;
            for e in entries {
                match e {
                    Timed::Wake(p, epoch) => {
                        if self.procs[p as usize].epoch == epoch && self.procs[p as usize].state == State::Waiting {
                            self.wake(p);
                        }
                    }
                    Timed::Nba(t, v, cx) => self.nba.push((t, v, cx)),
                }
            }
        }
        // `final` procedures run once, at the end (9.2.3).
        for (p, i) in self.finals.clone() {
            self.finishing = false;
            let id = self.new_proc(i, Some(p), None);
            let _ = id;
            self.slot();
        }
        self.outcome.time = self.time;
        let _ = self.out.flush();
    }

    /// One time slot's active, inactive and NBA regions, to quiescence.
    fn slot(&mut self) {
        loop {
            while let Some(p) = self.active.pop_front() {
                self.step(p);
                if self.finishing {
                    return;
                }
            }
            if !self.inactive.is_empty() {
                for p in std::mem::take(&mut self.inactive) {
                    if self.procs[p as usize].state == State::Waiting {
                        self.wake(p);
                    }
                }
                continue;
            }
            if !self.nba.is_empty() {
                for (t, v, cx) in std::mem::take(&mut self.nba) {
                    self.write_cx(&cx, &t, v, false);
                }
                continue;
            }
            break;
        }
    }

    fn postponed(&mut self) {
        for (cx, args, radix) in std::mem::take(&mut self.strobes) {
            let s = self.format_args(&cx, &args, radix);
            let _ = writeln!(self.out, "{s}");
        }
        if self.monitor_on {
            for i in 0..self.monitors.len() {
                let (cx, args, radix) = (self.monitors[i].cx.clone(), self.monitors[i].args.clone(), self.monitors[i].radix);
                let s = self.format_args(&cx, &args, radix);
                if self.monitors[i].last.as_ref() != Some(&s) {
                    let _ = writeln!(self.out, "{s}");
                    self.monitors[i].last = Some(s);
                }
            }
        }
    }

    /// The current time in an instance's time unit (for `$time`).
    fn time_in(&self, inst: u32) -> f64 {
        let unit = self.unit_of(inst);
        self.time as f64 / 10f64.powi((unit - self.precision) as i32)
    }

    fn unit_of(&self, inst: u32) -> i8 {
        if inst == u32::MAX {
            return self.d.unit_time.0;
        }
        self.d.spec(self.insts[inst as usize].spec).time_unit
    }

    /// A delay in an instance's time unit, as ticks of the global
    /// precision.
    fn ticks(&self, inst: u32, v: &Const) -> u64 {
        let unit = self.unit_of(inst);
        let scale = 10f64.powi((unit - self.precision) as i32);
        let x = match v {
            Const::Real(r) => *r,
            Const::Bits(b) => {
                if !b.is_known() {
                    return 0;
                }
                b.to_f64()
            }
            _ => 0.0,
        };
        (x * scale).round().max(0.0) as u64
    }

    fn after(&mut self, ticks: u64, e: Timed) {
        self.wheel.entry(self.time + ticks).or_default().push(e);
    }

    fn unsupported(&mut self, what: String) {
        if !self.outcome.unsupported.contains(&what) {
            self.outcome.unsupported.push(what);
        }
    }

    fn line_of(&self, n: crate::ast::NodeId) -> String {
        let t = self.ast.token(n).0;
        t.to_string()
    }

    /// Counts a step; past the step or time limit, stops the run.
    pub(crate) fn over_limit(&mut self) -> bool {
        self.steps += 1;
        if self.finishing {
            return true;
        }
        let over = self.steps > self.opts.max_steps
            || (self.steps.is_multiple_of(4096) && self.opts.time_limit.is_some_and(|l| self.started.elapsed() > l));
        if over {
            let _ = writeln!(self.out, "epitome-sv: the simulation ran too long; stopping");
            self.outcome.failed = true;
            self.finishing = true;
        }
        over
    }

    /// Runs process `p` until it waits or ends.
    fn step(&mut self, p: u32) {
        if let Some((t, v, cx)) = self.pending.remove(&p) {
            self.write_cx(&cx, &t, v, false);
        }
        if let Some(dr) = self.procs[p as usize].driver {
            if !self.over_limit() {
                self.drive(p, dr);
            }
            return;
        }
        loop {
            if self.over_limit() {
                return;
            }
            let pc = self.procs[p as usize].pc;
            let Some(op) = self.code.get(pc as usize).cloned() else {
                self.end_proc(p);
                return;
            };
            self.procs[p as usize].pc = pc + 1;
            match self.exec(p, pc, op) {
                Flow::Next => {}
                Flow::Suspend => return,
                Flow::Done => {
                    self.end_proc(p);
                    return;
                }
            }
            if self.finishing {
                return;
            }
        }
    }

    fn end_proc(&mut self, p: u32) {
        self.procs[p as usize].state = State::Done;
        self.procs[p as usize].epoch = self.procs[p as usize].epoch.wrapping_add(1);
        if let Some(parent) = self.procs[p as usize].parent {
            let pw = self.procs[parent as usize].wait.clone();
            match pw {
                Wait::Join(n) => {
                    if n <= 1 {
                        self.wake(parent);
                    } else {
                        self.procs[parent as usize].wait = Wait::Join(n - 1);
                    }
                }
                Wait::AllChildren => {
                    let live = self.procs[parent as usize].children.iter().any(|&c| self.procs[c as usize].state != State::Done);
                    if !live {
                        self.wake(parent);
                    }
                }
                _ => {}
            }
        }
    }

    /// Suspends `p` with this wait.
    fn suspend(&mut self, p: u32, w: Wait) -> Flow {
        let pr = &mut self.procs[p as usize];
        pr.state = State::Waiting;
        pr.epoch = pr.epoch.wrapping_add(1);
        pr.wait = w;
        Flow::Suspend
    }

    fn cx(&self, p: u32) -> Cx {
        self.procs[p as usize].frames.last().unwrap().cx.clone()
    }

    fn exec(&mut self, p: u32, pc: u32, op: Op) -> Flow {
        let cx = self.cx(p);
        match op {
            Op::Expr(x) => {
                self.eval(&cx, x);
            }
            Op::Assign(s) => return self.assign(p, &cx, s),
            Op::Jump(t) => self.procs[p as usize].pc = t,
            Op::JumpIfNot(c, t) => {
                let v = self.eval(&cx, c);
                if truth(&v) != Some(true) {
                    self.procs[p as usize].pc = t;
                }
            }
            Op::Case(s, targets, dflt) => {
                let to = self.case_target(&cx, s, &targets, dflt);
                self.procs[p as usize].pc = to;
            }
            Op::Wait(s) => {
                let StmtKind::Timed(t, _) = self.d.stmt(s).kind.clone() else { return Flow::Next };
                return self.timing(p, &cx, &t, Some(s));
            }
            Op::WaitCond(c) => {
                let v = self.eval(&cx, c);
                if truth(&v) == Some(true) {
                    return Flow::Next;
                }
                let f = self.suspend(p, Wait::Cond(c, cx.clone()));
                self.watch_expr(p, &cx, c);
                return f;
            }
            Op::RepeatInit(n) => {
                let v = self.eval(&cx, n);
                let k = v.to_i64().unwrap_or(0).max(0);
                self.frame(p).counters.insert(pc + 1, k);
            }
            Op::RepeatNext(exit) => {
                let c = self.frame(p).counters.get(&pc).copied().unwrap_or(0);
                if c <= 0 {
                    self.procs[p as usize].pc = exit;
                } else {
                    self.frame(p).counters.insert(pc, c - 1);
                }
            }
            Op::ForeachInit(s) => {
                let list = self.foreach_indices(&cx, s);
                self.frame(p).iters.insert(pc + 1, (list, 0));
            }
            Op::ForeachNext(s, exit) => {
                let next = self.frame(p).iters.get_mut(&pc).and_then(|(list, i)| {
                    let v = list.get(*i).cloned();
                    *i += 1;
                    v
                });
                match next {
                    Some(ix) => {
                        let StmtKind::Foreach { vars, .. } = self.d.stmt(s).kind.clone() else { return Flow::Next };
                        for (k, v) in vars.iter().enumerate() {
                            if let (Some(o), Some(&i)) = (v, ix.get(k)) {
                                let t = self.target_obj(&cx, *o);
                                self.write_cx(&cx, &t, Const::Bits(Bits::from_i64(32, true, i)), true);
                            }
                        }
                    }
                    None => self.procs[p as usize].pc = exit,
                }
            }
            Op::Enter(sc) => self.enter(&cx, sc),
            Op::CallTask(x) => return self.call_task(p, &cx, x),
            Op::Return(v) => return self.ret(p, &cx, v),
            Op::Fork(starts, join, after) => {
                let mut kids = Vec::new();
                for st in starts {
                    let id = self.procs.len() as u32;
                    let frame = Frame {
                        cx: cx.clone(),
                        ret: None,
                        sub: None,
                        counters: HashMap::new(),
                        iters: HashMap::new(),
                        ret_value: None,
                    };
                    self.procs.push(Proc {
                        pc: st,
                        frames: vec![frame],
                        state: State::Ready,
                        epoch: 0,
                        wait: Wait::None,
                        parent: Some(p),
                        children: vec![],
                        sensitive: vec![],
                        driver: None,
                    });
                    self.active.push_back(id);
                    kids.push(id);
                }
                let n = kids.len() as u32;
                self.procs[p as usize].children.extend(kids);
                self.procs[p as usize].pc = after;
                match join {
                    Join::None => {}
                    _ if n == 0 => {}
                    Join::All => return self.suspend(p, Wait::Join(n)),
                    Join::Any => return self.suspend(p, Wait::Join(1)),
                }
            }
            Op::BranchEnd => return Flow::Done,
            Op::WaitFork => {
                let live = self.procs[p as usize].children.iter().any(|&c| self.procs[c as usize].state != State::Done);
                if live {
                    return self.suspend(p, Wait::AllChildren);
                }
            }
            Op::DisableFork => {
                let kids = self.procs[p as usize].children.clone();
                for k in kids {
                    self.kill(k);
                }
            }
            Op::Disable(x, target) => match target {
                Some(t) => self.procs[p as usize].pc = t,
                None => {
                    let _ = x;
                    self.unsupported("`disable` of a block or task outside the current process".into());
                }
            },
            Op::Trigger(s) => {
                let StmtKind::Trigger { event, nonblocking } = self.d.stmt(s).kind.clone() else { return Flow::Next };
                let _ = nonblocking;
                self.trigger(&cx, event);
            }
            Op::ProcCont(s) => self.proc_cont(&cx, s),
            Op::Assert(s, fail, after) => {
                let StmtKind::Assert { cond, keyword, .. } = self.d.stmt(s).kind.clone() else { return Flow::Next };
                let v = self.eval(&cx, cond);
                if truth(&v) != Some(true) && self.d.name(keyword) != "cover" {
                    match fail {
                        Some(f) => self.procs[p as usize].pc = f,
                        None => {
                            let at = self.line_of(self.d.stmt(s).node);
                            let _ = writeln!(self.out, "Error: assertion failed at time {} ({at})", self.time);
                            self.outcome.errors += 1;
                            self.procs[p as usize].pc = after;
                        }
                    }
                }
            }
            Op::WaitSensitive => {
                let sens = self.procs[p as usize].sensitive.clone();
                let f = self.suspend(p, Wait::Change);
                for (o, _) in sens {
                    if let Some(s) = self.slot_of_obj(cx.inst, o) {
                        self.watch_slot(p, s);
                    }
                }
                return f;
            }
            Op::Halt => return Flow::Done,
            Op::Unsupported(s) => {
                let n = self.d.stmt(s).node;
                self.unsupported(format!("statement `{}`", self.line_of(n)));
            }
        }
        Flow::Next
    }

    fn frame(&mut self, p: u32) -> &mut Frame {
        self.procs[p as usize].frames.last_mut().unwrap()
    }

    fn kill(&mut self, p: u32) {
        let kids = self.procs[p as usize].children.clone();
        for k in kids {
            self.kill(k);
        }
        if self.procs[p as usize].state != State::Done {
            self.procs[p as usize].state = State::Done;
            self.procs[p as usize].epoch = self.procs[p as usize].epoch.wrapping_add(1);
        }
    }

    /// A block's automatic variables start again at their initial values.
    fn enter(&mut self, cx: &Cx, sc: ScopeId) {
        for item in self.d.scope(sc).items.clone() {
            if let Item::Object(o) = item
                && matches!(self.homes[o.0 as usize], Home::Local)
            {
                let obj = self.d.object(o);
                let v = match obj.init {
                    Some(i) => self.eval(cx, i),
                    None => crate::design::ops::default_value(self.types(), obj.ty),
                };
                let v = crate::design::ops::convert(self.types(), &v, obj.ty);
                let mut l = cx.locals.borrow_mut();
                match l.iter_mut().rev().find(|(k, _)| *k == o) {
                    Some(e) => e.1 = v,
                    None => l.push((o, v)),
                }
            }
        }
    }

    /// Waits as a timing control says: a delay or an event control.
    fn timing(&mut self, p: u32, cx: &Cx, t: &Timing, _s: Option<StmtId>) -> Flow {
        match t {
            Timing::Delay(xs) => {
                let v = xs.first().map_or(Const::Bits(Bits::zero(1, false)), |&x| self.eval(cx, x));
                let ticks = self.ticks(cx.inst, &v);
                let f = self.suspend(p, Wait::Time);
                let epoch = self.procs[p as usize].epoch;
                if ticks == 0 {
                    self.inactive.push(p);
                } else {
                    self.after(ticks, Timed::Wake(p, epoch));
                }
                f
            }
            Timing::Event(terms) | Timing::Implicit(terms) => {
                let mut saved = Vec::new();
                for t in terms {
                    let v = self.eval(cx, t.expr);
                    saved.push((t.clone(), v));
                }
                let implicit = matches!(t, Timing::Implicit(_)) && terms.is_empty();
                let f = self.suspend(p, if implicit { Wait::Change } else { Wait::Event(saved, cx.clone()) });
                if implicit {
                    // `@*`: whatever the statement after it reads.
                    if let Some(s) = _s
                        && let StmtKind::Timed(_, body) = self.d.stmt(s).kind
                    {
                        let mut slots = Vec::new();
                        self.stmt_reads(cx, body, &mut slots, 0);
                        for sl in slots {
                            self.watch_slot(p, sl);
                        }
                    }
                } else {
                    for t in terms {
                        self.watch_expr(p, cx, t.expr);
                    }
                }
                f
            }
            Timing::Repeat(n, terms) => {
                let _ = (n, terms);
                self.unsupported("`repeat (n) @(...)` timing".into());
                Flow::Next
            }
            Timing::Cycle(_) => {
                self.unsupported("cycle delays (`##`)".into());
                Flow::Next
            }
        }
    }

    /// An assignment statement, with any intra-assignment timing (9.4.5).
    fn assign(&mut self, p: u32, cx: &Cx, s: StmtId) -> Flow {
        let StmtKind::Assign { kind, lhs, rhs, timing } = self.d.stmt(s).kind.clone() else { return Flow::Next };
        let v = match kind {
            AssignKind::Compound(op) => {
                let old = self.eval(cx, lhs);
                let r = self.eval(cx, rhs);
                let lt = self.d.expr(lhs).ty;
                crate::design::ops::binary(self.types(), op, old, r, lt).unwrap_or(Const::Null)
            }
            _ => self.eval(cx, rhs),
        };
        let t = self.target(cx, lhs);
        match (kind, timing) {
            (AssignKind::NonBlocking, None) => {
                self.nba.push((t, v, cx.clone()));
                Flow::Next
            }
            (AssignKind::NonBlocking, Some(Timing::Delay(xs))) => {
                let d = xs.first().map_or(Const::Null, |&x| self.eval(cx, x));
                let ticks = self.ticks(cx.inst, &d);
                if ticks == 0 {
                    self.nba.push((t, v, cx.clone()));
                } else {
                    self.after(ticks, Timed::Nba(t, v, cx.clone()));
                }
                Flow::Next
            }
            (_, None) => {
                self.write_cx(cx, &t, v, false);
                Flow::Next
            }
            (_, Some(timing)) => {
                // `a = #d b`: the value now, the write after the wait.
                // The rest of the statement runs when the process resumes:
                // here, the write is queued to happen just before.
                let f = self.timing(p, cx, &timing, None);
                match timing {
                    Timing::Delay(xs) => {
                        let d = xs.first().map_or(Const::Null, |&x| self.eval(cx, x));
                        let ticks = self.ticks(cx.inst, &d);
                        if kind == AssignKind::NonBlocking {
                            self.after(ticks, Timed::Nba(t, v, cx.clone()));
                        } else {
                            self.pending_write(p, t, v, cx.clone());
                        }
                    }
                    _ => self.pending_write(p, t, v, cx.clone()),
                }
                f
            }
        }
    }

    /// A blocking write that happens when `p` resumes: done by a tiny
    /// trampoline, an inserted assignment step.
    fn pending_write(&mut self, p: u32, t: Target, v: Const, cx: Cx) {
        self.pending.insert(p, (t, v, cx));
    }
}

/// What running an op does to the process.
enum Flow {
    Next,
    Suspend,
    Done,
}

/// Whether two values are the same, x and z included.
fn same(a: &Const, b: &Const) -> bool {
    match (a, b) {
        (Const::Bits(x), Const::Bits(y)) => x.width() == y.width() && x.case_eq(y),
        _ => a == b,
    }
}

pub(crate) fn truth(c: &Const) -> Option<bool> {
    crate::design::ops::truth(c)
}

/// Whether a change from `old` to `new` is the edge waited for (9.4.2).
fn edge(e: Edge, old: &Const, new: &Const) -> bool {
    let lsb = |c: &Const| match c {
        Const::Bits(b) => b.bit(0),
        Const::Real(r) => {
            if *r != 0.0 {
                Logic::One
            } else {
                Logic::Zero
            }
        }
        _ => Logic::X,
    };
    match e {
        Edge::Any => !same(old, new),
        Edge::Pos | Edge::Neg | Edge::Both => {
            let (a, b) = (lsb(old), lsb(new));
            let pos = matches!((a, b), (Logic::Zero, Logic::One | Logic::X | Logic::Z) | (Logic::X | Logic::Z, Logic::One));
            let neg = matches!((a, b), (Logic::One, Logic::Zero | Logic::X | Logic::Z) | (Logic::X | Logic::Z, Logic::Zero));
            match e {
                Edge::Pos => pos,
                Edge::Neg => neg,
                _ => pos || neg,
            }
        }
    }
}

/// Replaces the part of `v` at the end of `path` with `new`.
fn update(tys: &Types, v: Const, path: &[Sel], new: Const) -> Const {
    let Some((first, rest)) = path.split_first() else { return new };
    match (first, v) {
        (Sel::Elem(i), Const::Aggregate(mut items)) => {
            if *i < items.len() {
                let old = std::mem::replace(&mut items[*i], Const::Null);
                items[*i] = update(tys, old, rest, new);
            }
            Const::Aggregate(items)
        }
        (Sel::Member(i), Const::Aggregate(mut items)) => {
            let i = *i as usize;
            if i < items.len() {
                let old = std::mem::replace(&mut items[i], Const::Null);
                items[i] = update(tys, old, rest, new);
            }
            Const::Aggregate(items)
        }
        (Sel::Key(k), Const::Aggregate(mut items)) => {
            // An associative array: pairs of key and value.
            let pos = items.chunks(2).position(|c| crate::design::ops::const_eq(&c[0], k));
            match pos {
                Some(p) => {
                    let old = std::mem::replace(&mut items[p * 2 + 1], Const::Null);
                    items[p * 2 + 1] = update(tys, old, rest, new);
                }
                None => {
                    items.push(k.clone());
                    items.push(update(tys, Const::Null, rest, new));
                }
            }
            Const::Aggregate(items)
        }
        (Sel::Bits(lo, w), Const::Bits(mut b)) => {
            let part = b.slice(*lo, *w);
            let nb = match update(tys, Const::Bits(part), rest, new) {
                Const::Bits(x) => x,
                Const::Real(r) => Bits::from_f64(*w, false, r),
                _ => Bits::x(*w),
            };
            b.set_slice(*lo, &nb.resize(*w, nb.signed()));
            Const::Bits(b)
        }
        (_, other) => other,
    }
}

/// The expressions a statement uses directly and in nested statements.
fn stmt_exprs(d: &Design, s: StmtId, out: &mut Vec<ExprId>) {
    match &d.stmt(s).kind {
        StmtKind::Block { stmts, .. } => stmts.iter().for_each(|&x| stmt_exprs(d, x, out)),
        StmtKind::Assign { lhs, rhs, .. } => {
            out.push(*rhs);
            // Indices of the target are read too.
            for &a in d.expr(*lhs).args.iter().skip(1) {
                out.push(a);
            }
        }
        StmtKind::Expr(x) => out.push(*x),
        StmtKind::If { cond, then, else_, .. } => {
            out.push(*cond);
            stmt_exprs(d, *then, out);
            if let Some(e) = else_ {
                stmt_exprs(d, *e, out);
            }
        }
        StmtKind::Case { expr, items, default, .. } => {
            out.push(*expr);
            for (xs, b) in items {
                out.extend(xs.iter().copied());
                stmt_exprs(d, *b, out);
            }
            if let Some(b) = default {
                stmt_exprs(d, *b, out);
            }
        }
        StmtKind::For { cond, body, .. } => {
            if let Some(c) = cond {
                out.push(*c);
            }
            stmt_exprs(d, *body, out);
        }
        StmtKind::While(c, b) | StmtKind::DoWhile(b, c) | StmtKind::Repeat(c, b) => {
            out.push(*c);
            stmt_exprs(d, *b, out);
        }
        StmtKind::Forever(b) | StmtKind::Timed(_, b) => stmt_exprs(d, *b, out),
        StmtKind::Foreach { array, body, .. } => {
            out.push(*array);
            stmt_exprs(d, *body, out);
        }
        StmtKind::Return(Some(x)) => out.push(*x),
        _ => {}
    }
}
