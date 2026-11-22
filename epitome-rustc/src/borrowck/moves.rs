//! Moves and initialisation: a value used before it has one (E0381) or after
//! it moved (E0382), a variable assigned twice that is not `mut` (E0384), and
//! what cannot be moved out of: a borrow (E0507), an array or slice element
//! (E0508), a type with a destructor (E0509).
//!
//! Each place the body names has a *move path*: a local, or a field of one (or
//! the contents of a `Box`). Three sets of them are followed through the
//! graph, each "may" (a path is in it if it is on any way here): not
//! initialised, moved, and initialised. A use of a path asks about it, what it
//! is part of and what is part of it: a moved field is a partly moved value.
//!
//! Whether a use of a place copies or moves it is the type's: `may_be_copy`
//! says "copy" unless it is known not to be, so an error that is not there is
//! never made for want of knowing.

use super::cfg::*;
use crate::ast::NodeId;
use crate::hash::FastMap;
use crate::ty::{Ty, TyCtxt, TyKind};
use crate::typeck::Solver;

/// An error found, before it has a position.
#[derive(Clone, Debug)]
pub struct Raw {
    pub code: &'static str,
    pub node: NodeId,
    pub message: String,
    /// The variable it is about: of those that a variable has not got a value
    /// or has moved, only the first use (in the source) is reported.
    pub local: Option<Local>,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
enum Step {
    /// A field (of this variant, for an enum's).
    Field(Option<u32>, u32),
    /// The contents of a `Box`.
    BoxDeref,
}

#[derive(Clone)]
struct Bits(Vec<u64>);

impl Bits {
    fn new(n: usize) -> Bits {
        Bits(vec![0; n.div_ceil(64)])
    }
    fn get(&self, i: usize) -> bool {
        self.0[i / 64] >> (i % 64) & 1 == 1
    }
    fn set(&mut self, i: usize, v: bool) {
        if v {
            self.0[i / 64] |= 1 << (i % 64);
        } else {
            self.0[i / 64] &= !(1 << (i % 64));
        }
    }
    /// `self` with `o` added; whether that changed it.
    fn union(&mut self, o: &Bits) -> bool {
        let mut changed = false;
        for (a, b) in self.0.iter_mut().zip(&o.0) {
            let n = *a | *b;
            changed |= n != *a;
            *a = n;
        }
        changed
    }
}

#[derive(Clone)]
struct State {
    uninit: Bits,
    moved: Bits,
    init: Bits,
}

impl State {
    fn union(&mut self, o: &State) -> bool {
        let a = self.uninit.union(&o.uninit);
        let b = self.moved.union(&o.moved);
        let c = self.init.union(&o.init);
        a | b | c
    }
}

struct PathInfo {
    local: Local,
    parent: Option<usize>,
    /// Itself and everything below it.
    below: Vec<usize>,
}

/// Where a place stops being something to follow.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Stop {
    /// All of it is a path.
    No,
    /// Through a reference or pointer.
    Deref,
    /// Into an array or slice, by an index.
    Index,
}

struct An<'a, 'b, 's> {
    cfg: &'a Cfg,
    tcx: &'a TyCtxt<'s>,
    solver: &'a mut Solver<'b, 's>,
    paths: Vec<PathInfo>,
    index: FastMap<(Local, Vec<Step>), usize>,
    copy: FastMap<Ty, bool>,
    out: Vec<Raw>,
    reporting: bool,
}

/// The errors of moves and initialisation in a body.
/// `order` is where a node is in the file: of several errors about one variable,
/// the first is the one.
pub fn check<'b, 's>(cfg: &Cfg, tcx: &TyCtxt<'s>, solver: &mut Solver<'b, 's>, order: &dyn Fn(NodeId) -> u32) -> Vec<Raw> {
    let mut an = An { cfg, tcx, solver, paths: Vec::new(), index: FastMap::default(), copy: FastMap::default(), out: Vec::new(), reporting: false };
    an.run();
    let mut seen = std::collections::HashSet::new();
    an.out.retain(|r| seen.insert((r.code, r.node)));
    // One error for a variable that is not there (rustc does the same): the
    // first use in the source.
    let mut first: FastMap<(&'static str, Local), NodeId> = FastMap::default();
    for r in &an.out {
        if let (Some(l), "E0381" | "E0382") = (r.local, r.code) {
            let e = first.entry((r.code, l)).or_insert(r.node);
            if order(r.node) < order(*e) {
                *e = r.node;
            }
        }
    }
    an.out.retain(|r| match (r.local, r.code) {
        (Some(l), "E0381" | "E0382") => first[&(r.code, l)] == r.node,
        _ => true,
    });
    an.out
}

impl<'a, 'b, 's> An<'a, 'b, 's> {
    // ---- paths ----

    fn pointee(&self, t: Ty) -> Option<Ty> {
        match self.tcx.kind(t) {
            TyKind::Ref(_, x) | TyKind::Ptr(_, x) => Some(x),
            TyKind::Adt(d, l) if self.tcx.lang.get("owned_box") == Some(&d) => self.tcx.tys(l).first().copied(),
            _ => None,
        }
    }

    fn copies_by_structure(&mut self, t: Ty) -> bool {
        match self.tcx.kind(t) {
            TyKind::Pat(x, ..) => self.is_copy(x),
            TyKind::Adt(d, l) if self.tcx.lang.get("pin") == Some(&d) => {
                matches!(self.tcx.tys(l).first().map(|&x| self.tcx.kind(x)), Some(TyKind::Ref(crate::ty::Mutbl::Mut, _)))
            }
            // A type that says how it is reborrowed is.
            TyKind::Adt(..) => self.solver.implements_lang(t, "reborrow"),
            _ => false,
        }
    }

    /// Whether `t` has no size known: a slice, `str` or `dyn`, or a struct
    /// that ends in one.
    fn is_unsized(&self, t: Ty) -> bool {
        match self.tcx.kind(t) {
            TyKind::Slice(_) | TyKind::Str | TyKind::Dyn(_) => true,
            TyKind::Adt(d, l) => {
                let Some(adt) = self.tcx.adts.get(&d) else { return false };
                if adt.kind != crate::ty::AdtKind::Struct {
                    return false;
                }
                let Some(f) = adt.variants.first().and_then(|v| v.fields.last()) else { return false };
                let ft = crate::ty::collect::subst(self.tcx, f.ty, &self.tcx.all_params(d), &self.tcx.tys(l));
                self.is_unsized(ft)
            }
            _ => false,
        }
    }

    fn is_box(&self, t: Ty) -> bool {
        matches!(self.tcx.kind(t), TyKind::Adt(d, _) if self.tcx.lang.get("owned_box") == Some(&d))
    }

    /// The steps of the part of `p` that is a path, and where the rest starts.
    fn steps(&self, p: &Place) -> (Vec<Step>, Stop) {
        let mut ty = self.cfg.locals[p.local.0 as usize].ty;
        let mut steps = Vec::new();
        let mut variant = None;
        for pr in &p.proj {
            match pr {
                Proj::Downcast(v) => variant = Some(*v),
                Proj::Field(i, fty) => {
                    // The fields of a union are one: giving one a value gives
                    // them all.
                    let union = matches!(self.tcx.kind(ty), TyKind::Adt(d, _) if self.tcx.adts.get(&d).is_some_and(|a| a.kind == crate::ty::AdtKind::Union));
                    steps.push(Step::Field(variant.take(), if union { 0 } else { *i }));
                    ty = *fty;
                }
                Proj::Deref => {
                    if self.is_box(ty) {
                        steps.push(Step::BoxDeref);
                        ty = self.pointee(ty).unwrap_or(ty);
                    } else {
                        return (steps, Stop::Deref);
                    }
                }
                Proj::Index(..) | Proj::ConstIndex(..) | Proj::Subslice(..) => return (steps, Stop::Index),
            }
        }
        (steps, Stop::No)
    }

    fn ensure(&mut self, local: Local, steps: &[Step]) -> usize {
        let mut parent = None;
        let mut idx = 0;
        for n in 0..=steps.len() {
            let key = (local, steps[..n].to_vec());
            idx = match self.index.get(&key) {
                Some(&i) => i,
                None => {
                    let i = self.paths.len();
                    self.paths.push(PathInfo { local, parent, below: Vec::new() });
                    self.index.insert(key, i);
                    i
                }
            };
            parent = Some(idx);
        }
        idx
    }

    /// The path of the part of `p` that is one.
    fn locate(&self, p: &Place) -> Option<(usize, Stop)> {
        let (steps, stop) = self.steps(p);
        self.index.get(&(p.local, steps)).map(|&i| (i, stop))
    }

    fn collect_places(&mut self) {
        let mut places: Vec<Place> = Vec::new();
        let op = |o: &Operand, v: &mut Vec<Place>| {
            if let Operand::Use(p) = o {
                v.push(p.clone());
            }
        };
        for b in self.cfg.reachable() {
            let data = &self.cfg.blocks[b.0 as usize];
            for s in &data.stmts {
                match &s.kind {
                    StmtKind::Assign(d, rv) => {
                        places.push(d.clone());
                        match rv {
                            Rvalue::Use(o) | Rvalue::Unary(_, o) | Rvalue::Cast(o, _) | Rvalue::Repeat(o, _) => op(o, &mut places),
                            Rvalue::Binary(_, a, b) => {
                                op(a, &mut places);
                                op(b, &mut places);
                            }
                            Rvalue::Aggregate(_, os) => os.iter().for_each(|o| op(o, &mut places)),
                            Rvalue::Ref(_, p) | Rvalue::AddrOf(_, p) | Rvalue::CaptureRef(p) | Rvalue::Discriminant(p) | Rvalue::Len(p) => places.push(p.clone()),
                        }
                    }
                    StmtKind::StorageLive(l) | StmtKind::StorageDead(l) => places.push(Place::local(*l, self.cfg.locals[l.0 as usize].ty)),
                    StmtKind::Drop(_) => {}
                    StmtKind::MaybeInit(p) => places.push(p.clone()),
                }
            }
            match data.term.as_ref().map(|t| &t.0) {
                Some(Term::Switch { discr, .. }) => op(discr, &mut places),
                Some(Term::Call { func, args, dest, .. }) => {
                    if let Callee::Value(o) = func {
                        op(o, &mut places);
                    }
                    args.iter().for_each(|o| op(o, &mut places));
                    places.push(dest.clone());
                }
                _ => {}
            }
        }
        for i in 0..self.cfg.locals.len() {
            self.ensure(Local(i as u32), &[]);
        }
        for p in &places {
            let (steps, _) = self.steps(p);
            self.ensure(p.local, &steps);
        }
        // What is below each path.
        let n = self.paths.len();
        let mut below: Vec<Vec<usize>> = (0..n).map(|i| vec![i]).collect();
        for i in (0..n).rev() {
            if let Some(par) = self.paths[i].parent {
                let b = std::mem::take(&mut below[i]);
                below[par].extend(b.iter().copied());
                below[i] = b;
            }
        }
        for (i, b) in below.into_iter().enumerate() {
            self.paths[i].below = b;
        }
    }

    // ---- the analysis ----

    fn run(&mut self) {
        self.collect_places();
        let n = self.paths.len();
        let reach = self.cfg.reachable();
        // At the start: the parameters have values, nothing else does.
        let mut entry = State { uninit: Bits::new(n), moved: Bits::new(n), init: Bits::new(n) };
        for (i, d) in self.cfg.locals.iter().enumerate() {
            let root = self.index[&(Local(i as u32), Vec::new())];
            let arg = d.kind == LocalKind::Arg;
            for &p in &self.paths[root].below.clone() {
                entry.uninit.set(p, !arg);
                entry.init.set(p, arg);
            }
        }
        let empty = State { uninit: Bits::new(n), moved: Bits::new(n), init: Bits::new(n) };
        let mut states: Vec<Option<State>> = vec![None; self.cfg.blocks.len()];
        states[0] = Some(entry);
        let mut work: Vec<Block> = vec![Cfg::ENTRY];
        while let Some(b) = work.pop() {
            let mut st = states[b.0 as usize].clone().unwrap_or_else(|| empty.clone());
            self.block(b, &mut st);
            let succs = self.cfg.blocks[b.0 as usize].term.as_ref().map(|t| t.0.successors()).unwrap_or_default();
            for s in succs {
                match &mut states[s.0 as usize] {
                    Some(old) => {
                        if old.union(&st) {
                            work.push(s);
                        }
                    }
                    slot @ None => {
                        *slot = Some(st.clone());
                        work.push(s);
                    }
                }
            }
        }
        // Again, now to say what is wrong.
        self.reporting = true;
        for b in reach {
            if let Some(st) = &states[b.0 as usize] {
                let mut st = st.clone();
                self.block(b, &mut st);
            }
        }
    }

    fn block(&mut self, b: Block, st: &mut State) {
        let data = &self.cfg.blocks[b.0 as usize];
        for s in &data.stmts {
            self.stmt(s, st);
        }
        if let Some((t, node)) = &data.term {
            self.term(t, *node, st);
        }
    }

    fn err(&mut self, code: &'static str, node: NodeId, message: &str, local: Option<Local>) {
        if self.reporting {
            self.out.push(Raw { code, node, message: message.to_string(), local });
        }
    }

    fn is_copy(&mut self, t: Ty) -> bool {
        if let Some(&c) = self.copy.get(&t) {
            return c;
        }
        // A `&mut` is reborrowed where it is passed on, which typeck does not
        // always say: taken as a copy. So is `Pin<&mut T>`, and a pattern
        // type is as copy as the type it narrows. Nor is what has no size
        // moved.
        let c = matches!(self.tcx.kind(t), TyKind::Ref(crate::ty::Mutbl::Mut, _))
            || self.solver.may_be_copy(t)
            || self.is_unsized(t)
            || self.copies_by_structure(t);
        self.copy.insert(t, c);
        c
    }

    fn stmt(&mut self, s: &Stmt, st: &mut State) {
        match &s.kind {
            StmtKind::StorageLive(l) | StmtKind::StorageDead(l) => {
                // A variable that has just come, or gone, has no value.
                let root = self.index[&(*l, Vec::new())];
                for &p in &self.paths[root].below.clone() {
                    st.uninit.set(p, true);
                    st.moved.set(p, false);
                    st.init.set(p, false);
                }
            }
            StmtKind::Drop(_) => {}
            StmtKind::MaybeInit(p) => {
                if let Some((path, Stop::No)) = self.locate(p) {
                    for &q in &self.paths[path].below.clone() {
                        st.uninit.set(q, false);
                        st.moved.set(q, false);
                        st.init.set(q, true);
                    }
                }
            }
            StmtKind::Assign(dest, rv) => {
                match rv {
                    Rvalue::Use(o) | Rvalue::Unary(_, o) | Rvalue::Cast(o, _) | Rvalue::Repeat(o, _) => self.operand(o, s.node, st),
                    Rvalue::Binary(op, a, b) => {
                        // A comparison takes what it compares by reference.
                        let cmp = matches!(op.as_str(), "==" | "!=" | "<" | "<=" | ">" | ">=");
                        for o in [a, b] {
                            match o {
                                Operand::Use(p) if cmp => self.read(p, s.node, st),
                                _ => self.operand(o, s.node, st),
                            }
                        }
                    }
                    Rvalue::Aggregate(_, os) => os.iter().for_each(|o| self.operand(o, s.node, st)),
                    Rvalue::Ref(_, p) | Rvalue::AddrOf(_, p) | Rvalue::Discriminant(p) | Rvalue::Len(p) => self.read(p, s.node, st),
                    Rvalue::CaptureRef(p) => self.read_shallow(p, s.node, st),
                }
                self.assign(dest, s.node, st);
            }
        }
    }

    fn term(&mut self, t: &Term, node: NodeId, st: &mut State) {
        match t {
            Term::Switch { discr, .. } => self.operand(discr, node, st),
            Term::Call { func, args, dest, .. } => {
                if let Callee::Value(o) = func {
                    self.operand(o, node, st);
                }
                for a in args {
                    self.operand(a, node, st);
                }
                self.assign(dest, node, st);
            }
            _ => {}
        }
    }

    fn operand(&mut self, o: &Operand, node: NodeId, st: &mut State) {
        if let Operand::Use(p) = o {
            // No value of type `!` is made: what uses it is not reached.
            if matches!(self.tcx.kind(p.ty), TyKind::Never) {
                return;
            }
            if self.is_copy(p.ty) {
                self.read(p, node, st);
            } else {
                self.mov(p, node, st);
            }
        }
    }

    /// Asks whether the part of `p` that is a path can be used.
    fn read(&mut self, p: &Place, node: NodeId, st: &State) {
        let Some((path, _)) = self.locate(p) else { return };
        self.check_use(path, p.at.unwrap_or(node), st);
    }

    fn root_local(&self, path: usize) -> Local {
        self.paths[path].local
    }

    /// `read`, but not of the parts: what is moved out of a variable does not
    /// stop a closure capturing the rest.
    fn read_shallow(&mut self, p: &Place, node: NodeId, st: &State) {
        let Some((path, _)) = self.locate(p) else { return };
        self.check_use_in(path, p.at.unwrap_or(node), st, false);
    }

    fn check_use(&mut self, path: usize, node: NodeId, st: &State) {
        self.check_use_in(path, node, st, true);
    }

    fn check_use_in(&mut self, path: usize, node: NodeId, st: &State, parts: bool) {
        let local = self.root_local(path);
        // What it is part of, itself, and what is part of it.
        let mut related: Vec<usize> = if parts { self.paths[path].below.clone() } else { vec![path] };
        let mut up = self.paths[path].parent;
        while let Some(a) = up {
            related.push(a);
            up = self.paths[a].parent;
        }
        if related.iter().any(|&r| st.moved.get(r)) {
            self.err("E0382", node, "use of moved value", Some(local));
        } else if related.iter().any(|&r| st.uninit.get(r)) {
            self.err("E0381", node, "used binding isn't initialized", Some(local));
        }
    }

    /// The type of the place after the first `n` projections.
    /// Whether `p` is what a closure captured: a field of its environment.
    fn in_closure_env(&self, p: &Place) -> bool {
        self.cfg.closure.is_some() && p.local == Local(1) && matches!(p.proj.first(), Some(Proj::Field(..)))
    }

    fn type_before(&self, p: &Place, n: usize) -> Ty {
        let mut ty = self.cfg.locals[p.local.0 as usize].ty;
        for pr in &p.proj[..n] {
            ty = match pr {
                Proj::Field(_, t) => *t,
                Proj::Deref => self.pointee(ty).unwrap_or(ty),
                Proj::Index(_, e) | Proj::ConstIndex(_, _, e) => *e,
                Proj::Subslice(_, _, _, e) => self.tcx.mk(TyKind::Slice(*e)),
                Proj::Downcast(_) => ty,
            };
        }
        ty
    }

    fn mov(&mut self, p: &Place, node: NodeId, st: &mut State) {
        let node = p.at.unwrap_or(node);
        let Some((path, stop)) = self.locate(p) else { return };
        self.check_use(path, node, st);
        if self.cfg.locals[p.local.0 as usize].kind == LocalKind::Static {
            self.err("E0507", node, "cannot move out of a static item", None);
            return;
        }
        // What cannot be moved out of. (An element of an array or slice is
        // the more particular error, behind a reference or not.)
        let mut behind = false;
        for (i, pr) in p.proj.iter().enumerate() {
            let base = self.type_before(p, i);
            match pr {
                Proj::Deref if !self.is_box(base) => behind = true,
                Proj::Index(..) | Proj::Subslice(..) => {
                    self.err("E0508", node, "cannot move out of an array or slice by an index", None);
                    return;
                }
                Proj::ConstIndex(..) if behind || matches!(self.tcx.kind(base), TyKind::Slice(_)) => {
                    self.err("E0508", node, "cannot move out of a slice", None);
                    return;
                }
                Proj::Field(..) if self.solver.has_drop_impl(base) => {
                    self.err("E0509", node, "cannot move out of a type that implements `Drop`", None);
                    return;
                }
                _ => {}
            }
        }
        if behind {
            // What a closure moves out of a variable it has by reference is
            // not an error of the closure's: how it captured it is typeck's
            // guess.
            if !self.in_closure_env(p) {
                self.err("E0507", node, "cannot move out of a borrow", None);
            }
            return;
        }
        if stop == Stop::No {
            for &q in &self.paths[path].below.clone() {
                st.moved.set(q, true);
            }
        }
    }

    fn assign(&mut self, dest: &Place, node: NodeId, st: &mut State) {
        let Some((path, stop)) = self.locate(dest) else { return };
        if stop != Stop::No {
            // Through a reference: it must be there, and nothing changes.
            self.check_use(path, node, st);
            return;
        }
        // A part of what is moved or not there cannot be assigned.
        let mut up = self.paths[path].parent;
        while let Some(a) = up {
            if st.moved.get(a) {
                self.err("E0382", node, "assign to part of moved value", Some(dest.local));
                break;
            }
            if st.uninit.get(a) {
                self.err("E0381", node, "partially assigned binding isn't fully initialized", Some(dest.local));
                break;
            }
            up = self.paths[a].parent;
        }
        // A variable that is not `mut` is assigned once.
        if dest.proj.is_empty() {
            let d = &self.cfg.locals[dest.local.0 as usize];
            if matches!(d.kind, LocalKind::Var | LocalKind::Arg) && !d.mutable && st.init.get(path) {
                self.err("E0384", node, "cannot assign twice to immutable variable", None);
            }
        }
        for &q in &self.paths[path].below.clone() {
            st.uninit.set(q, false);
            st.moved.set(q, false);
            st.init.set(q, true);
        }
    }
}
