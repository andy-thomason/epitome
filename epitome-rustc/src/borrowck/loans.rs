//! Loans: what a borrow keeps from being used (NLL, milestone 5).
//!
//! `Ty` has no regions, so a value's regions are kept as *slots*: one for
//! each place a lifetime can be in its type, in order (`&'a Vec<&'b T>` has
//! two: `'a`, `'b`; a struct has one for each lifetime parameter, then those
//! of its type arguments). Each local has, for each slot, the set of loans
//! its region may hold: a flow-sensitive stand-in for rustc's region
//! inference.
//!
//! - A borrow `r = &place` is a *loan*. `r`'s outer slot holds it; the rest
//!   hold what `place` holds. A reborrow `&*q` also holds what `q`'s outer
//!   slot holds.
//! - Copies and aggregates move sets between slots. A call's result gets what
//!   the signature (`regions::sig`) ties to it: an input slot and an output
//!   slot with the same lifetime or type parameter. `&mut` arguments get what
//!   other arguments tie to what they point to (`v.push(&x)`).
//! - A loan is *live* where a local that holds it is live (used later). An
//!   assignment to a variable ends the loans of what it pointed to.
//! - An access to a place (read, write, move, borrow, end of the variable's
//!   scope) conflicts with a live loan of an overlapping place: E0499, E0502,
//!   E0503, E0505, E0506, E0597, E0716; and what the return value holds that
//!   is local to the function is E0515.
//!
//! Where it is unsure it holds less (never more), so that what rustc
//! accepts is accepted.

use super::cfg::*;
use super::moves::Raw;
use crate::ast::NodeId;
use crate::hash::FastMap;
use crate::regions::sig::{self, FnRegions, RArg, RTy, Region};
use crate::regions::variance::{ParamKind, Variances};
use crate::resolve::{DefId, Resolver};
use crate::session::ParsedCrate;
use crate::ty::{ClosureId, Mutbl, ParamId, Ty, TyCtxt, TyKind};
use crate::typeck::Solver;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

/// What `check` needs besides the body: caches of what is read from the
/// source (signatures, fields) and of how many slots a type has.
pub struct Cx<'a, 's> {
    pub tcx: &'a TyCtxt<'s>,
    pub r: &'a Resolver<'s>,
    pub crates: &'a [ParsedCrate<'s>],
    pub var: &'a Variances,
    /// How each closure uses what it captured.
    pub closures: &'a FastMap<ClosureId, Vec<UpvarUse>>,
    /// For each closure parameter, whether its region is one the caller
    /// does not name (`F: FnOnce(&T)`): it stands for any lifetime.
    pub hr: &'a FastMap<ClosureId, Vec<bool>>,
    sigs: RefCell<FastMap<DefId, Option<Rc<FnRegions>>>>,
    fields: RefCell<FastMap<DefId, Option<Rc<Vec<Vec<RTy>>>>>>,
    counts: RefCell<FastMap<Ty, u32>>,
}

impl<'a, 's> Cx<'a, 's> {
    pub fn new(tcx: &'a TyCtxt<'s>, r: &'a Resolver<'s>, crates: &'a [ParsedCrate<'s>], var: &'a Variances, closures: &'a FastMap<ClosureId, Vec<UpvarUse>>, hr: &'a FastMap<ClosureId, Vec<bool>>) -> Self {
        Cx { tcx, r, crates, var, closures, hr, sigs: Default::default(), fields: Default::default(), counts: Default::default() }
    }

    fn sig(&self, d: DefId) -> Option<Rc<FnRegions>> {
        self.sigs.borrow_mut().entry(d).or_insert_with(|| sig::fn_regions(self.crates, self.r, self.tcx, d).map(Rc::new)).clone()
    }

    fn adt_fields(&self, d: DefId) -> Option<Rc<Vec<Vec<RTy>>>> {
        self.fields.borrow_mut().entry(d).or_insert_with(|| sig::adt_fields(self.crates, self.r, self.tcx, d).map(Rc::new)).clone()
    }

    fn lifetimes(&self, d: DefId) -> u32 {
        self.var.of(d).map_or(0, |ps| ps.iter().filter(|p| p.kind == ParamKind::Lifetime).count() as u32)
    }

    /// How many regions a value of type `t` has.
    fn slots(&self, t: Ty) -> u32 {
        if let Some(&n) = self.counts.borrow().get(&t) {
            return n;
        }
        let tcx = self.tcx;
        let n = match tcx.kind(t) {
            TyKind::Ref(_, i) => 1 + self.slots(i),
            TyKind::Adt(d, args) => self.lifetimes(d) + tcx.tys(args).into_iter().map(|a| self.slots(a)).sum::<u32>(),
            TyKind::Tuple(l) => tcx.tys(l).into_iter().map(|a| self.slots(a)).sum(),
            TyKind::Array(e, _) | TyKind::Slice(e) | TyKind::Pat(e, _, _) => self.slots(e),
            TyKind::Dyn(_)
            | TyKind::Param(_)
            | TyKind::Projection(..)
            | TyKind::Opaque(..)
            | TyKind::Closure(..)
            | TyKind::AsyncBlock(..)
            | TyKind::Coroutine(..)
            | TyKind::GenBlock(..) => 1,
            _ => 0,
        };
        self.counts.borrow_mut().insert(t, n);
        n
    }

    fn is_box(&self, t: Ty) -> bool {
        matches!(self.tcx.kind(t), TyKind::Adt(d, _) if self.tcx.lang.get("owned_box") == Some(&d))
    }

    /// The type a projection goes to, from the type before it.
    fn after(&self, t: Ty, p: &Proj) -> Ty {
        match p {
            Proj::Deref => match self.tcx.kind(t) {
                TyKind::Ref(_, i) | TyKind::Ptr(_, i) => i,
                TyKind::Adt(..) if self.is_box(t) => self.tcx.tys(match self.tcx.kind(t) {
                    TyKind::Adt(_, l) => l,
                    _ => unreachable!(),
                })
                .first()
                .copied()
                .unwrap_or(t),
                _ => t,
            },
            Proj::Downcast(_) => t,
            Proj::Field(_, f) => *f,
            Proj::Index(_, e) | Proj::ConstIndex(_, _, e) | Proj::Subslice(_, _, _, e) => *e,
        }
    }
}

/// A symbol of a signature: what ties one slot to another.
#[derive(Clone, PartialEq, Eq, Debug)]
enum Sym {
    Reg(Region),
    /// Slot `k` of the type given for a parameter.
    Par(ParamId, u32),
    /// Every slot of it.
    ParAny(ParamId),
}

impl Sym {
    fn ties(&self, o: &Sym) -> bool {
        match (self, o) {
            (Sym::Reg(a), Sym::Reg(b)) => a == b && !matches!(a, Region::Static | Region::Unknown | Region::Bound(_)),
            (Sym::Par(a, i), Sym::Par(b, j)) => a == b && i == j,
            (Sym::Par(a, _), Sym::ParAny(b)) | (Sym::ParAny(a), Sym::Par(b, _)) | (Sym::ParAny(a), Sym::ParAny(b)) => a == b,
            _ => false,
        }
    }
}

/// The symbols of each slot of a value of type `ty`, which the signature
/// type `t` describes: the two are walked together, so that what a
/// signature cannot say (an associated type, an opaque type) takes the
/// slots the type has.
fn desc(cx: &Cx, t: &RTy, ty: Ty, out: &mut Vec<Vec<Sym>>) {
    let start = out.len();
    let want = cx.slots(ty) as usize;
    let mentioned = |a: &[RArg]| -> Vec<Sym> {
        let mut v = Vec::new();
        let mut regs = Vec::new();
        for x in a {
            match x {
                RArg::Region(r) => v.push(Sym::Reg(*r)),
                RArg::Ty(t) => {
                    t.regions(&mut regs);
                    params(t, &mut v);
                }
                RArg::Const => {}
            }
        }
        v.extend(regs.into_iter().map(Sym::Reg));
        v
    };
    // Every slot of the type tied to what the signature type mentions.
    let broadcast = |syms: Vec<Sym>, out: &mut Vec<Vec<Sym>>| {
        for _ in 0..want {
            out.push(syms.clone());
        }
    };
    let tcx = cx.tcx;
    match (t, tcx.kind(ty)) {
        (RTy::Plain, _) | (RTy::Ptr(..), _) | (RTy::FnPtr(..), _) => {
            for _ in 0..want {
                out.push(Vec::new());
            }
        }
        (RTy::Ref(r, _, i), TyKind::Ref(_, ti)) => {
            out.push(vec![Sym::Reg(*r)]);
            desc(cx, i, ti, out);
        }
        (RTy::Slice(i), TyKind::Slice(ti)) | (RTy::Array(i), TyKind::Array(ti, _)) | (RTy::Slice(i), TyKind::Array(ti, _)) | (RTy::Array(i), TyKind::Slice(ti)) => desc(cx, i, ti, out),
        (RTy::Tuple(ts), TyKind::Tuple(l)) if ts.len() == tcx.tys(l).len() => {
            for (x, ti) in ts.iter().zip(tcx.tys(l)) {
                desc(cx, x, ti, out);
            }
        }
        (RTy::Adt(d, a), TyKind::Adt(d2, targs)) if *d == d2 => {
            // Lifetime arguments come first.
            let nl = cx.lifetimes(*d) as usize;
            let regions: Vec<Region> = a.iter().filter_map(|x| if let RArg::Region(r) = x { Some(*r) } else { None }).collect();
            if regions.len() != nl {
                return broadcast(mentioned(a), out);
            }
            out.extend(regions.into_iter().map(|r| vec![Sym::Reg(r)]));
            let rest: Vec<&RArg> = a.iter().filter(|x| !matches!(x, RArg::Region(_))).collect();
            for (x, ti) in rest.into_iter().zip(tcx.tys(targs)) {
                match x {
                    RArg::Ty(t) => desc(cx, t, ti, out),
                    _ => {
                        for _ in 0..cx.slots(ti) {
                            out.push(Vec::new());
                        }
                    }
                }
            }
        }
        (RTy::Param(p), _) => {
            for k in 0..want {
                out.push(vec![Sym::Par(*p, k as u32)]);
            }
        }
        (RTy::Dyn(r, _), TyKind::Dyn(_)) => out.push(vec![Sym::Reg(*r)]),
        (RTy::Proj(a), _) => broadcast(mentioned(a), out),
        (RTy::Opaque(a, bs), _) => {
            let mut v = mentioned(a);
            for b in bs {
                // `Fn(A) -> B`: what it is given is not what it holds.
                let sugar = b.args.len() == 1 && matches!(&b.args[0], RArg::Ty(RTy::Tuple(_))) && b.assoc.iter().any(|(n, _)| n == "Output");
                if sugar {
                    continue;
                }
                v.extend(mentioned(&b.args));
                for (_, t) in &b.assoc {
                    v.extend(mentioned(&[RArg::Ty(t.clone())]));
                }
            }
            broadcast(v, out)
        }
        // A shape the signature and the type do not agree on.
        (other, _) => {
            let mut v = Vec::new();
            let mut regs = Vec::new();
            other.regions(&mut regs);
            params(other, &mut v);
            v.extend(regs.into_iter().map(Sym::Reg));
            broadcast(v, out)
        }
    }
    out.resize(start + want, Vec::new());
}

fn params(t: &RTy, out: &mut Vec<Sym>) {
    let args = |a: &[RArg], out: &mut Vec<Sym>| {
        for x in a {
            if let RArg::Ty(t) = x {
                params(t, out);
            }
        }
    };
    match t {
        RTy::Param(p) => out.push(Sym::ParAny(*p)),
        RTy::Ref(_, _, i) | RTy::Ptr(_, i) | RTy::Slice(i) | RTy::Array(i) => params(i, out),
        RTy::Tuple(ts) => ts.iter().for_each(|x| params(x, out)),
        RTy::Adt(_, a) | RTy::Proj(a) => args(a, out),
        RTy::Dyn(_, bs) => bs.iter().for_each(|b| args(&b.args, out)),
        RTy::Opaque(a, bs) => {
            args(a, out);
            bs.iter().for_each(|b| args(&b.args, out));
        }
        RTy::FnPtr(_, i, o) => {
            i.iter().for_each(|x| params(x, out));
            params(o, out);
        }
        RTy::Plain => {}
    }
}

fn has_opaque(t: &RTy) -> bool {
    match t {
        RTy::Opaque(..) => true,
        RTy::Ref(_, _, i) | RTy::Ptr(_, i) | RTy::Slice(i) | RTy::Array(i) => has_opaque(i),
        RTy::Tuple(ts) => ts.iter().any(has_opaque),
        RTy::Adt(_, a) | RTy::Proj(a) => a.iter().any(|x| matches!(x, RArg::Ty(t) if has_opaque(t))),
        _ => false,
    }
}

/// A set of loans, sorted.
type Set = Vec<u32>;

fn union_into(a: &mut Set, b: &[u32]) -> bool {
    if b.iter().all(|x| a.binary_search(x).is_ok()) {
        return false;
    }
    a.extend_from_slice(b);
    a.sort_unstable();
    a.dedup();
    true
}

/// What each local's regions hold.
#[derive(Clone, PartialEq, Eq, Default, Debug)]
struct State {
    held: BTreeMap<u32, Vec<Set>>,
}

impl State {
    fn join(&mut self, o: &State) -> bool {
        let mut changed = false;
        for (l, slots) in &o.held {
            match self.held.get_mut(l) {
                Some(mine) => {
                    for (a, b) in mine.iter_mut().zip(slots) {
                        changed |= union_into(a, b);
                    }
                }
                None => {
                    self.held.insert(*l, slots.clone());
                    changed = true;
                }
            }
        }
        changed
    }

    fn kill(&mut self, ids: &[u32]) {
        if ids.is_empty() {
            return;
        }
        self.held.retain(|_, slots| {
            for s in slots.iter_mut() {
                s.retain(|x| !ids.contains(x));
            }
            slots.iter().any(|s| !s.is_empty())
        });
    }
}

struct Loan {
    place: Place,
    mutbl: Mutbl,
    node: NodeId,
    block: u32,
    idx: u32,
    /// For a borrow that is two-phase: the call that activates it (block,
    /// terminator), and the points between where it only reserves.
    act: Option<(u32, u32)>,
    reserved: std::collections::HashSet<(u32, u32)>,
    /// Behind a shared reference or a raw pointer: nothing can conflict.
    ignored: bool,
    /// Made by a closure capturing a variable.
    capture: bool,
    /// Not a borrow: what a parameter's region stands for (the caller's).
    universal: bool,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Kind {
    Read,
    SharedBorrow,
    /// The reservation of a two-phase mutable borrow: as a shared one, but
    /// it is a mutable borrow that it conflicts with.
    Reserve,
    MutBorrow,
    Write,
    Move,
    Dead,
}

struct Access {
    place: Place,
    kind: Kind,
    node: NodeId,
    /// Of the place itself, not what it points through.
    shallow: bool,
}

/// Locals as a bit set.
#[derive(Clone, PartialEq, Eq)]
struct Bits(Vec<u64>);

impl Bits {
    fn new(n: usize) -> Bits {
        Bits(vec![0; n.div_ceil(64)])
    }
    fn get(&self, i: u32) -> bool {
        self.0[(i / 64) as usize] >> (i % 64) & 1 == 1
    }
    fn set(&mut self, i: u32, v: bool) {
        if v {
            self.0[(i / 64) as usize] |= 1 << (i % 64);
        } else {
            self.0[(i / 64) as usize] &= !(1 << (i % 64));
        }
    }
    fn or(&mut self, o: &Bits) -> bool {
        let mut ch = false;
        for (a, b) in self.0.iter_mut().zip(&o.0) {
            let n = *a | *b;
            ch |= n != *a;
            *a = n;
        }
        ch
    }
}

const MAX_LOANS: usize = 1500;

pub fn check<'b, 's>(cx: &Cx<'_, 's>, cfg: &Cfg, solver: &mut Solver<'b, 's>, original: &dyn Fn(NodeId) -> bool) -> Vec<Raw> {
    // What a constant or static borrows is promoted to `'static`: only a
    // variable of its own body, borrowed in its value, is an error.
    let const_mode = matches!(
        cx.r.defs[cfg.owner.0 as usize].kind,
        crate::resolve::DefKind::Const | crate::resolve::DefKind::Static | crate::resolve::DefKind::AssocConst
    );
    let mut an = An::new(cx, cfg);
    an.const_mode = const_mode;
    an.original = Some(original);
    if an.loans.is_empty() || an.loans.len() > MAX_LOANS || cfg.blocks.len() * cfg.locals.len() > 2_000_000 {
        return Vec::new();
    }
    an.drops(solver);
    an.liveness();
    an.flow();
    an.report(solver)
}

struct An<'a, 'c, 's> {
    /// What the parameters hold on entry.
    init: State,
    /// The body of a constant or static.
    const_mode: bool,
    /// Types the function says outlive `'static`: they hold no loans.
    static_proj: bool,
    static_params: Vec<ParamId>,
    /// Whether a node is written in the file (not made by a macro).
    original: Option<&'a dyn Fn(NodeId) -> bool>,
    trace: bool,
    cx: &'a Cx<'c, 's>,
    cfg: &'a Cfg,
    loans: Vec<Loan>,
    /// The loan a statement makes: (block, statement) to loan.
    loan_at: FastMap<(u32, u32), u32>,
    /// The loans a closure capture makes.
    capture_loans: FastMap<(u32, u32), Vec<u32>>,
    /// Loans of each local's places.
    rooted: Vec<Vec<u32>>,
    /// Locals live on entry to each block.
    live_in: Vec<Bits>,
    /// What each block starts with.
    entry: Vec<Option<State>>,
    copy: FastMap<Ty, bool>,
    out: Vec<Raw>,
    /// Types whose drop reads the borrows they hold.
    drop_use: FastMap<Ty, bool>,
    /// The drops (block, statement) of what may be set, that read them.
    drop_live: std::collections::HashSet<(u32, u32)>,
    /// The node of the first assignment to each local (for a temporary).
    def_node: FastMap<u32, NodeId>,
}

impl<'a, 'c, 's> An<'a, 'c, 's> {
    fn new(cx: &'a Cx<'c, 's>, cfg: &'a Cfg) -> Self {
        let owner_sig = cx.sig(cfg.owner);
        let static_proj = owner_sig.as_ref().is_some_and(|s| s.ty_outlives.iter().any(|(t, r)| *r == Region::Static && matches!(t, RTy::Proj(_))));
        let static_params = owner_sig
            .as_ref()
            .map(|s| s.ty_outlives.iter().filter_map(|(t, r)| if let (RTy::Param(p), Region::Static) = (t, r) { Some(*p) } else { None }).collect())
            .unwrap_or_default();
        let mut an = An {
            init: State::default(),
            const_mode: false,
            static_proj,
            static_params,
            original: None,
            trace: std::env::var_os("EPITOME_LOANS_TRACE").is_some_and(|f| cfg.name.contains(f.to_str().unwrap_or(""))),
            cx,
            cfg,
            loans: Vec::new(),
            loan_at: FastMap::default(),
            capture_loans: FastMap::default(),
            rooted: vec![Vec::new(); cfg.locals.len()],
            live_in: Vec::new(),
            entry: Vec::new(),
            copy: FastMap::default(),
            out: Vec::new(),
            def_node: FastMap::default(),
            drop_use: FastMap::default(),
            drop_live: Default::default(),
        };
        let promoted = promotable(cfg, cx);
        for b in cfg.reachable() {
            let data = &cfg.blocks[b.0 as usize];
            for (i, s) in data.stmts.iter().enumerate() {
                if let StmtKind::Assign(d, rv) = &s.kind {
                    an.def_node.entry(d.local.0).or_insert(s.node);
                    if let Rvalue::Ref(m, p) = rv {
                        let (act, reserved) = match s.two_phase && *m == Mutbl::Mut {
                            true => match an.reservation(b.0, i as u32, d.local) {
                                Some((pts, at)) => (Some(at), pts),
                                None => (None, Default::default()),
                            },
                            false => (None, Default::default()),
                        };
                        let ignored = an.behind_shared(p)
                            || cfg.locals[p.local.0 as usize].kind == LocalKind::Static
                            || promoted.get(&p.local.0).is_some_and(|&empty_array| *m == Mutbl::Not || empty_array);
                        let id = an.loans.len() as u32;
                        an.rooted[p.local.0 as usize].push(id);
                        an.loans.push(Loan { place: p.clone(), mutbl: *m, node: p.at.unwrap_or(s.node), block: b.0, idx: i as u32, act, reserved, ignored, capture: false, universal: false });
                        an.loan_at.insert((b.0, i as u32), id);
                    }
                }
            }
            if let Some((Term::Call { dest, .. }, node)) = &data.term {
                an.def_node.entry(dest.local.0).or_insert(*node);
            }
            an.captures(b.0, &promoted);
        }
        an.universals();
        an
    }

    /// Each region of a parameter (that the signature does not make
    /// `'static`) is held, on entry, by a loan of the caller's.
    fn universals(&mut self) {
        let (cx, cfg) = (self.cx, self.cfg);
        if self.const_mode {
            return;
        }
        if let Some(c) = cfg.closure {
            // A closure's parameters the caller does not name: any lifetime.
            let Some(hr) = cx.hr.get(&c) else { return };
            for (i, &is_hr) in hr.iter().enumerate() {
                let a = 2 + i;
                if !is_hr || a > cfg.arg_count || cx.slots(cfg.locals[a].ty) == 0 || !matches!(cx.tcx.kind(cfg.locals[a].ty), TyKind::Ref(..)) {
                    continue;
                }
                let ty = cfg.locals[a].ty;
                let id = self.loans.len() as u32;
                self.loans.push(Loan {
                    place: Place::local(Local(a as u32), ty),
                    mutbl: Mutbl::Not,
                    node: cfg.locals[a].node.unwrap_or(cfg.node),
                    block: 0,
                    idx: 0,
                    act: None,
                    reserved: Default::default(),
                    ignored: true,
                    capture: false,
                    universal: true,
                });
                let mut slots: Vec<Set> = vec![Vec::new(); cx.slots(ty) as usize];
                slots[0] = vec![id];
                self.init.held.insert(a as u32, slots);
            }
            return;
        }
        let Some(sig) = cx.sig(cfg.owner) else { return };
        for a in 1..=cfg.arg_count {
            let Some(rty) = sig.inputs.get(a - 1) else { continue };
            let ty = cfg.locals[a].ty;
            let mut ds = Vec::new();
            desc(cx, rty, ty, &mut ds);
            if ds.len() != cx.slots(ty) as usize {
                continue;
            }
            let mut slots: Vec<Set> = vec![Vec::new(); ds.len()];
            for (k, syms) in ds.iter().enumerate() {
                let [Sym::Reg(r @ (Region::Param(_) | Region::Elided(_) | Region::ImplAnon(_)))] = syms.as_slice() else { continue };
                if sig.outlives.iter().any(|(x, y)| x == r && *y == Region::Static) {
                    continue;
                }
                let id = self.loans.len() as u32;
                self.loans.push(Loan {
                    place: Place::local(Local(a as u32), ty),
                    mutbl: Mutbl::Not,
                    node: cfg.locals[a].node.unwrap_or(cfg.node),
                    block: 0,
                    idx: 0,
                    act: None,
                    reserved: Default::default(),
                    ignored: true,
                    capture: false,
                    universal: true,
                });
                slots[k] = vec![id];
            }
            if slots.iter().any(|x| !x.is_empty()) {
                self.init.held.insert(a as u32, slots);
            }
        }
    }

    /// The loans of the closure captures made in block `b`: a borrow of
    /// each part of a variable the closure uses, mutable if it changes it.
    fn captures(&mut self, b: u32, promoted: &FastMap<u32, bool>) {
        let cfg = self.cfg;
        let data = &cfg.blocks[b as usize];
        let mut defs: FastMap<u32, u32> = FastMap::default();
        for (i, s) in data.stmts.iter().enumerate() {
            let StmtKind::Assign(d, rv) = &s.kind else { continue };
            if matches!(rv, Rvalue::CaptureRef(_)) && d.proj.is_empty() && !s.moved {
                defs.insert(d.local.0, i as u32);
            }
            let Rvalue::Aggregate(AggKind::Closure(c), ops) = rv else { continue };
            for (k, o) in ops.iter().enumerate() {
                let Operand::Use(t) = o else { continue };
                let (Some(&at), true) = (defs.get(&t.local.0), t.proj.is_empty()) else { continue };
                let StmtKind::Assign(_, Rvalue::CaptureRef(src)) = &data.stmts[at as usize].kind else { continue };
                let Some(u) = self.cx.closures.get(c).and_then(|v| v.get(k)) else { continue };
                if !u.used {
                    continue;
                }
                // The parts used, and not those inside another.
                let mut paths: Vec<&Vec<Proj>> = u.paths.iter().collect();
                paths.sort_by_key(|p| p.len());
                let mut kept: Vec<&Vec<Proj>> = Vec::new();
                for p in paths {
                    if !kept.iter().any(|q| p.starts_with(q)) {
                        kept.push(p);
                    }
                }
                let mut ids = Vec::new();
                for path in kept {
                    let mut place = src.clone();
                    for pr in path {
                        let ty = self.cx.after(place.ty, pr);
                        place = place.project(*pr, ty);
                    }
                    let ignored = self.behind_shared(&place) || cfg.locals[place.local.0 as usize].kind == LocalKind::Static || promoted.contains_key(&place.local.0);
                    let id = self.loans.len() as u32;
                    self.rooted[place.local.0 as usize].push(id);
                    let mutbl = if u.mutated { Mutbl::Mut } else { Mutbl::Not };
                    self.loans.push(Loan {
                        place: place.clone(),
                        mutbl,
                        node: data.stmts[at as usize].node,
                        block: b,
                        idx: at,
                        act: None,
                        reserved: Default::default(),
                        ignored,
                        capture: true,
                        universal: false,
                    });
                    ids.push(id);
                }
                self.capture_loans.insert((b, at), ids);
            }
        }
    }

    /// The call that takes `holder` (a two-phase borrow's temporary) as an
    /// argument, following the code from the borrow while it is not
    /// branching, and the points on the way (where the borrow only
    /// reserves). `None` if the temporary is used any other way first.
    fn reservation(&self, mut b: u32, from: u32, holder: Local) -> Option<(std::collections::HashSet<(u32, u32)>, (u32, u32))> {
        let mut pts = std::collections::HashSet::new();
        let mut start = from as usize + 1;
        for _ in 0..64 {
            let data = &self.cfg.blocks[b as usize];
            for (i, s) in data.stmts.iter().enumerate().skip(start) {
                if let StmtKind::Assign(_, rv) = &s.kind
                    && rvalue_places(rv).iter().any(|p| p.local == holder)
                {
                    return None;
                }
                pts.insert((b, i as u32));
            }
            let n = data.stmts.len() as u32;
            pts.insert((b, n));
            match &data.term {
                Some((Term::Call { func, args, target, .. }, _)) => {
                    let direct = args.iter().any(|a| matches!(a, Operand::Use(p) if p.local == holder && p.proj.is_empty()));
                    if direct {
                        return Some((pts, (b, n)));
                    }
                    if matches!(func, Callee::Value(Operand::Use(p)) if p.local == holder) || args.iter().any(|a| matches!(a, Operand::Use(p) if p.local == holder)) {
                        return None;
                    }
                    b = (*target)?.0;
                }
                Some((Term::Goto(t), _)) => b = t.0,
                _ => return None,
            }
            start = 0;
        }
        None
    }

    /// Whether the two-phase borrow made by `s` has a call to activate it.
    fn has_activation(&self, s: &Stmt) -> bool {
        self.loans.iter().any(|l| std::ptr::eq(self.cfg.blocks[l.block as usize].stmts.get(l.idx as usize).map_or(std::ptr::null(), |x| x as *const Stmt), s) && l.act.is_some())
    }

    /// Whether `p` goes through a shared reference or a raw pointer.
    fn behind_shared(&self, p: &Place) -> bool {
        let mut t = self.cfg.locals[p.local.0 as usize].ty;
        for pr in &p.proj {
            if matches!(pr, Proj::Deref) && matches!(self.cx.tcx.kind(t), TyKind::Ref(Mutbl::Not, _) | TyKind::Ptr(..)) {
                return true;
            }
            t = self.cx.after(t, pr);
        }
        false
    }

    // ---- slots of places ----

    /// For each slot of `p`'s type, the slot of its local it is (`u32::MAX`
    /// if it is not one).
    fn place_slots(&self, p: &Place) -> Vec<u32> {
        let cx = self.cx;
        let mut ty = self.cfg.locals[p.local.0 as usize].ty;
        let mut idx: Vec<u32> = (0..cx.slots(ty)).collect();
        let mut variant = 0u32;
        let none = |n: u32| vec![u32::MAX; n as usize];
        for pr in &p.proj {
            let next = cx.after(ty, pr);
            let n = cx.slots(next);
            match pr {
                Proj::Deref => match cx.tcx.kind(ty) {
                    TyKind::Ref(..) => idx = idx.get(1..).map_or(none(n), |s| s.to_vec()),
                    TyKind::Ptr(..) => idx = none(n),
                    TyKind::Adt(d, _) if cx.is_box(ty) => {
                        let off = cx.lifetimes(d) as usize;
                        idx = idx.get(off..off + n as usize).map_or(none(n), |s| s.to_vec());
                    }
                    _ => idx = none(n),
                },
                Proj::Downcast(v) => variant = *v,
                Proj::Field(i, _) => {
                    idx = match cx.tcx.kind(ty) {
                        TyKind::Tuple(l) => {
                            let before: u32 = cx.tcx.tys(l).iter().take(*i as usize).map(|&t| cx.slots(t)).sum();
                            idx.get(before as usize..(before + n) as usize).map_or(none(n), |s| s.to_vec())
                        }
                        TyKind::Adt(d, args) => self.field_slots(d, args, variant, *i, next, &idx, n),
                        // (A closure's captures are one region.)
                        TyKind::Closure(..) => idx.first().map_or(none(n), |&f| vec![f; n as usize]),
                        _ => none(n),
                    };
                    variant = 0;
                }
                Proj::Index(..) | Proj::ConstIndex(..) | Proj::Subslice(..) => {
                    if idx.len() != n as usize {
                        idx = idx.get(..n as usize).map_or(none(n), |s| s.to_vec());
                    }
                }
            }
            ty = next;
        }
        idx
    }

    /// The slots of field `i` of variant `v` of an ADT value whose slots are
    /// `idx` (of its local).
    fn field_slots(&self, d: DefId, args: crate::ty::List, v: u32, i: u32, fty: Ty, idx: &[u32], n: u32) -> Vec<u32> {
        let cx = self.cx;
        let none = vec![u32::MAX; n as usize];
        let Some(fields) = cx.adt_fields(d) else { return none };
        let Some(rty) = fields.get(v as usize).and_then(|f| f.get(i as usize)) else { return none };
        let targs = cx.tcx.tys(args);
        let mut ds = Vec::new();
        desc(cx, rty, fty, &mut ds);
        if ds.len() != n as usize {
            return none;
        }
        let nl = cx.lifetimes(d);
        // Where each type argument's slots start.
        let mut starts = Vec::new();
        let mut at = nl;
        for &a in &targs {
            starts.push(at);
            at += cx.slots(a);
        }
        ds.into_iter()
            .map(|syms| match syms.first() {
                Some(Sym::Reg(Region::Param(k))) if *k < nl => idx.get(*k as usize).copied().unwrap_or(u32::MAX),
                Some(Sym::Par(p, k)) => {
                    let ai = cx.tcx.params[p.0 as usize].index as usize;
                    starts.get(ai).and_then(|s| idx.get((s + k) as usize)).copied().unwrap_or(u32::MAX)
                }
                _ => u32::MAX,
            })
            .collect()
    }

    fn read(&self, st: &State, p: &Place) -> Vec<Set> {
        let n = self.cx.slots(p.ty) as usize;
        let Some(slots) = st.held.get(&p.local.0) else { return vec![Vec::new(); n] };
        let idx = self.place_slots(p);
        (0..n)
            .map(|k| match idx.get(k) {
                Some(&s) if s != u32::MAX => slots.get(s as usize).cloned().unwrap_or_default(),
                _ => Vec::new(),
            })
            .collect()
    }

    fn operand(&self, st: &State, o: &Operand) -> Vec<Set> {
        match o {
            Operand::Use(p) => self.read(st, p),
            Operand::Const(_, t) => vec![Vec::new(); self.cx.slots(*t) as usize],
        }
    }

    /// Replaces, or adds to, what `p`'s slots hold.
    fn write(&self, st: &mut State, p: &Place, vals: Vec<Set>, strong: bool, depth: u32) {
        if vals.iter().all(|v| v.is_empty()) && !strong {
            return;
        }
        let nslots = self.cx.slots(self.cfg.locals[p.local.0 as usize].ty) as usize;
        if p.proj.is_empty() && strong {
            if vals.iter().all(|v| v.is_empty()) {
                st.held.remove(&p.local.0);
            } else {
                let mut v = vals;
                v.resize(nslots, Vec::new());
                st.held.insert(p.local.0, v);
            }
            return;
        }
        let idx = self.place_slots(p);
        let slots = st.held.entry(p.local.0).or_insert_with(|| vec![Vec::new(); nslots]);
        for (k, v) in vals.iter().enumerate() {
            if let Some(&s) = idx.get(k)
                && s != u32::MAX
                && let Some(t) = slots.get_mut(s as usize)
            {
                union_into(t, v);
            }
        }
        // Through a reference: the places it borrows hold it too.
        if depth < 4
            && let Some(d) = self.first_deref_of_ref(p)
        {
            let prefix = Place { local: p.local, proj: p.proj[..d].to_vec(), ty: self.prefix_ty(p, d), at: None };
            let outer = self.read(st, &prefix).first().cloned().unwrap_or_default();
            for l in outer {
                let loan = &self.loans[l as usize];
                if loan.universal {
                    continue;
                }
                let mut proj = loan.place.proj.clone();
                proj.extend_from_slice(&p.proj[d..]);
                let target = Place { local: loan.place.local, proj, ty: p.ty, at: None };
                self.write(st, &target, vals.clone(), false, depth + 1);
            }
        }
        if st.held.get(&p.local.0).is_some_and(|s| s.iter().all(|x| x.is_empty())) {
            st.held.remove(&p.local.0);
        }
    }

    /// The position of the first `*` in `p` of a reference.
    fn first_deref_of_ref(&self, p: &Place) -> Option<usize> {
        let mut t = self.cfg.locals[p.local.0 as usize].ty;
        for (i, pr) in p.proj.iter().enumerate() {
            if matches!(pr, Proj::Deref) && matches!(self.cx.tcx.kind(t), TyKind::Ref(..)) {
                return Some(i);
            }
            t = self.cx.after(t, pr);
        }
        None
    }

    /// The type of `p` cut to its first `n` projections.
    fn prefix_ty(&self, p: &Place, n: usize) -> Ty {
        let mut t = self.cfg.locals[p.local.0 as usize].ty;
        for pr in &p.proj[..n] {
            t = self.cx.after(t, pr);
        }
        t
    }

    // ---- drops ----

    /// Whether dropping a value of type `t` reads what it borrows: it has a
    /// `Drop` of its own that is not `#[may_dangle]` (one of a library's types
    /// is, for its type parameters, but not for its lifetimes), or holds one.
    fn drop_uses(&mut self, solver: &mut Solver<'_, '_>, t: Ty, depth: u32) -> bool {
        if let Some(&c) = self.drop_use.get(&t) {
            return c;
        }
        let cx = self.cx;
        let c = if cx.slots(t) == 0 || depth > 3 {
            false
        } else {
            match cx.tcx.kind(t) {
                TyKind::Adt(d, _) if cx.tcx.lang.get("manually_drop") == Some(&d) => false,
                TyKind::Adt(d, args) => {
                    let user = !matches!(cx.crates[cx.r.defs[d.0 as usize].krate as usize].name.as_str(), "core" | "alloc" | "std");
                    let own = (user || cx.lifetimes(d) > 0) && !self.may_dangle(d) && solver.has_drop_impl(t);
                    // (A type of the program with a `Drop` of its own drops what its
                    // fields say, not what it is generic in.)
                    let skip_args = user && solver.has_drop_impl(t);
                    own || (!skip_args && cx.tcx.tys(args).into_iter().any(|a| self.drop_uses(solver, a, depth + 1))) || {
                        // What its fields hold.
                        let params = cx.tcx.all_params(d);
                        let targs = cx.tcx.tys(args);
                        let tys: Vec<Ty> = cx.tcx.adts.get(&d).map(|a| a.variants.iter().flat_map(|v| v.fields.iter().map(|f| f.ty)).collect()).unwrap_or_default();
                        tys.into_iter().any(|f| {
                            let f = crate::ty::collect::subst(cx.tcx, f, &params, &targs);
                            self.drop_uses(solver, f, depth + 1)
                        })
                    }
                }
                TyKind::Tuple(l) => cx.tcx.tys(l).into_iter().any(|a| self.drop_uses(solver, a, depth + 1)),
                TyKind::Array(e, _) | TyKind::Slice(e) => self.drop_uses(solver, e, depth + 1),
                TyKind::Dyn(_) => true,
                _ => false,
            }
        };
        self.drop_use.insert(t, c);
        c
    }

    /// A move of all of a local leaves it without a value.
    fn move_out(&mut self, solver: &mut Solver<'_, '_>, st: &mut Bits, o: &Operand) {
        // (A part moved out is as the whole: what is left is not for us to say.)
        if let Operand::Use(p) = o
            && !self.is_copy(solver, p.ty)
        {
            st.set(p.local.0, false);
        }
    }

    /// Whether the `Drop` of `d` says `#[may_dangle]` of what it is generic in.
    fn may_dangle(&self, d: DefId) -> bool {
        let cx = self.cx;
        cx.tcx.impls.iter().any(|i| {
            let is_drop = i.trait_ref.as_ref().is_some_and(|tr| cx.tcx.lang.get("drop") == Some(&tr.def));
            if !is_drop || !matches!(cx.tcx.kind(i.self_ty), TyKind::Adt(a, _) if a == d) {
                return false;
            }
            let def = &cx.r.defs[i.def.0 as usize];
            let ast = &cx.crates[def.krate as usize].files[def.file as usize].ast;
            ast.child(def.node, crate::ast::Kind::Generics).is_some_and(|g| ast.text(g).contains("may_dangle"))
        })
    }

    /// The drops whose value may be set and that read what it borrows.
    fn drops(&mut self, solver: &mut Solver<'_, '_>) {
        let cfg = self.cfg;
        let nl = cfg.locals.len();
        let order = cfg.reachable();
        // Which locals may have a value: set by an assignment, unset by a move
        // of all of it, and when its storage comes or goes.
        let mut entry: Vec<Option<Bits>> = vec![None; cfg.blocks.len()];
        let mut first = Bits::new(nl);
        for l in 1..=cfg.arg_count as u32 {
            first.set(l, true);
        }
        entry[0] = Some(first);
        for round in 0..2 {
            for _ in 0..30 {
                let mut changed = false;
                for &b in &order {
                    let Some(mut st) = entry[b.0 as usize].clone() else { continue };
                    let data = &cfg.blocks[b.0 as usize];
                    for (i, s) in data.stmts.iter().enumerate() {
                        match &s.kind {
                            StmtKind::Assign(d, rv) => {
                                match rv {
                                    Rvalue::Use(o) | Rvalue::Unary(_, o) | Rvalue::Cast(o, _) | Rvalue::Repeat(o, _) => self.move_out(solver, &mut st, o),
                                    Rvalue::Binary(_, a, b2) => {
                                        self.move_out(solver, &mut st, a);
                                        self.move_out(solver, &mut st, b2);
                                    }
                                    Rvalue::Aggregate(_, os) => os.iter().for_each(|o| self.move_out(solver, &mut st, o)),
                                    _ => {}
                                }
                                if d.proj.is_empty() {
                                    st.set(d.local.0, true);
                                }
                            }
                            StmtKind::StorageLive(l) | StmtKind::StorageDead(l) => st.set(l.0, false),
                            StmtKind::MaybeInit(p) => st.set(p.local.0, true),
                            StmtKind::Drop(p) => {
                                if round == 1 && p.proj.is_empty() && st.get(p.local.0) && self.drop_uses(solver, p.ty, 0) {
                                    self.drop_live.insert((b.0, i as u32));
                                }
                            }
                        }
                    }
                    let Some((t, _)) = &data.term else { continue };
                    if let Term::Call { func, args, dest, .. } = t {
                        for a in args {
                            self.move_out(solver, &mut st, a);
                        }
                        if let Callee::Value(o) = func {
                            self.move_out(solver, &mut st, o);
                        }
                        if dest.proj.is_empty() {
                            st.set(dest.local.0, true);
                        }
                    }
                    for s in t.successors() {
                        match &mut entry[s.0 as usize] {
                            Some(e) => changed |= e.or(&st),
                            slot @ None => {
                                *slot = Some(st.clone());
                                changed = true;
                            }
                        }
                    }
                }
                if !changed {
                    break;
                }
            }
        }
    }

    // ---- liveness ----

    fn uses(&self, s: &StmtKind, pos: (u32, u32), out: &mut Vec<u32>) {
        let place = |p: &Place, out: &mut Vec<u32>| {
            // Assigning to part of a variable is not using it.
            out.push(p.local.0);
            for pr in &p.proj {
                if let Proj::Index(l, _) = pr {
                    out.push(l.0);
                }
            }
        };
        match s {
            StmtKind::Assign(d, rv) => {
                if !d.proj.is_empty() && d.proj.iter().any(|p| matches!(p, Proj::Deref | Proj::Index(..))) {
                    place(d, out);
                } else {
                    for pr in &d.proj {
                        if let Proj::Index(l, _) = pr {
                            out.push(l.0);
                        }
                    }
                }
                for p in rvalue_places(rv) {
                    place(p, out);
                }
            }
            StmtKind::Drop(p) => {
                // Dropping what may be set and has a `Drop` that reads its borrows.
                if self.drop_live.contains(&pos) {
                    out.push(p.local.0);
                }
            }
            StmtKind::MaybeInit(_) | StmtKind::StorageLive(_) | StmtKind::StorageDead(_) => {}
        }
    }

    fn term_uses(&self, t: &Term, out: &mut Vec<u32>) {
        let op = |o: &Operand, out: &mut Vec<u32>| {
            if let Operand::Use(p) = o {
                out.push(p.local.0);
                for pr in &p.proj {
                    if let Proj::Index(l, _) = pr {
                        out.push(l.0);
                    }
                }
            }
        };
        match t {
            Term::Switch { discr, .. } => op(discr, out),
            Term::Call { func, args, dest, .. } => {
                if let Callee::Value(o) = func {
                    op(o, out);
                }
                args.iter().for_each(|a| op(a, out));
                if dest.proj.iter().any(|p| matches!(p, Proj::Deref | Proj::Index(..))) {
                    out.push(dest.local.0);
                }
            }
            Term::Return => {
                // What the caller sees: the result, and everything an
                // argument points to.
                out.push(0);
                out.extend(1..=self.cfg.arg_count as u32);
            }
            Term::Goto(_) | Term::Unreachable => {}
        }
    }

    /// Applies a statement backwards: what is live before it.
    fn live_before(&self, s: &StmtKind, pos: (u32, u32), live: &mut Bits) {
        match s {
            StmtKind::Assign(d, _) if d.proj.is_empty() => live.set(d.local.0, false),
            StmtKind::StorageDead(l) | StmtKind::StorageLive(l) => live.set(l.0, false),
            _ => {}
        }
        let mut u = Vec::new();
        self.uses(s, pos, &mut u);
        for l in u {
            live.set(l, true);
        }
    }

    fn live_before_term(&self, t: &Term, live: &mut Bits) {
        if let Term::Call { dest, .. } = t
            && dest.proj.is_empty()
        {
            live.set(dest.local.0, false);
        }
        let mut u = Vec::new();
        self.term_uses(t, &mut u);
        for l in u {
            live.set(l, true);
        }
    }

    fn liveness(&mut self) {
        let n = self.cfg.blocks.len();
        let nl = self.cfg.locals.len();
        let mut live_in = vec![Bits::new(nl); n];
        let order = self.cfg.reachable();
        loop {
            let mut changed = false;
            for &b in order.iter().rev() {
                let data = &self.cfg.blocks[b.0 as usize];
                let Some((t, _)) = &data.term else { continue };
                let mut live = Bits::new(nl);
                for s in t.successors() {
                    live.or(&live_in[s.0 as usize]);
                }
                self.live_before_term(t, &mut live);
                for (i, s) in data.stmts.iter().enumerate().rev() {
                    self.live_before(&s.kind, (b.0, i as u32), &mut live);
                }
                if live != live_in[b.0 as usize] {
                    live_in[b.0 as usize] = live;
                    changed = true;
                }
            }
            if !changed {
                break;
            }
        }
        self.live_in = live_in;
    }

    // ---- what is held ----

    /// A type that outlives `'static` by the function's bounds.
    fn holds_nothing(&self, t: Ty) -> bool {
        match self.cx.tcx.kind(t) {
            TyKind::Projection(..) => self.static_proj,
            TyKind::Param(p) => self.static_params.contains(&p),
            _ => false,
        }
    }

    fn rooted_ids(&self, l: Local) -> &[u32] {
        &self.rooted[l.0 as usize]
    }

    fn transfer_stmt(&self, b: u32, i: u32, s: &Stmt, st: &mut State) {
        match &s.kind {
            StmtKind::StorageLive(l) => {
                st.held.remove(&l.0);
            }
            StmtKind::StorageDead(l) => {
                st.held.remove(&l.0);
                st.kill(self.rooted_ids(*l));
            }
            StmtKind::Drop(_) | StmtKind::MaybeInit(_) => {}
            StmtKind::Assign(d, rv) => {
                if d.proj.is_empty() {
                    st.kill(self.rooted_ids(d.local));
                }
                let n = self.cx.slots(d.ty) as usize;
                let mut vals = match rv {
                    Rvalue::Use(o) | Rvalue::Repeat(o, _) => self.operand(st, o),
                    Rvalue::Cast(o, _) => self.operand(st, o),
                    Rvalue::Ref(_, p) => {
                        let mut v = vec![Vec::new()];
                        if let Some(&l) = self.loan_at.get(&(b, i)) {
                            v[0].push(l);
                        }
                        // A reborrow also holds what the references it goes
                        // through hold: from the last `*` back, as far as the
                        // first of a shared reference.
                        let mut t = self.cfg.locals[p.local.0 as usize].ty;
                        let mut derefs = Vec::new();
                        for (k, pr) in p.proj.iter().enumerate() {
                            if matches!(pr, Proj::Deref) && matches!(self.cx.tcx.kind(t), TyKind::Ref(..)) {
                                derefs.push((k, t));
                            }
                            t = self.cx.after(t, pr);
                        }
                        for &(k, rt) in derefs.iter().rev() {
                            let pre = Place { local: p.local, proj: p.proj[..k].to_vec(), ty: rt, at: None };
                            if let Some(o) = self.read(st, &pre).first() {
                                union_into(&mut v[0], o);
                            }
                            if matches!(self.cx.tcx.kind(rt), TyKind::Ref(Mutbl::Not, _)) {
                                break;
                            }
                        }
                        v.extend(self.read(st, p));
                        v
                    }
                    Rvalue::CaptureRef(p) => {
                        let mut v = vec![self.capture_loans.get(&(b, i)).cloned().unwrap_or_default()];
                        v.extend(self.read(st, p));
                        v
                    }
                    Rvalue::AddrOf(..) | Rvalue::Binary(..) | Rvalue::Unary(..) | Rvalue::Discriminant(_) | Rvalue::Len(_) => Vec::new(),
                    Rvalue::Aggregate(kind, ops) => self.aggregate(st, kind, ops, d.ty),
                };
                fit(&mut vals, n);
                if self.holds_nothing(d.ty) {
                    vals = vec![Vec::new(); n];
                }
                self.write(st, d, vals, d.proj.is_empty(), 0);
            }
        }
    }

    fn aggregate(&self, st: &State, kind: &AggKind, ops: &[Operand], ty: Ty) -> Vec<Set> {
        let cx = self.cx;
        let n = cx.slots(ty) as usize;
        match kind {
            AggKind::Tuple => ops.iter().flat_map(|o| self.operand(st, o)).collect(),
            AggKind::Array => {
                let mut v = vec![Vec::new(); n];
                for o in ops {
                    for (a, b) in v.iter_mut().zip(self.operand(st, o)) {
                        union_into(a, &b);
                    }
                }
                v
            }
            AggKind::Closure(_) => {
                let mut all = Vec::new();
                for o in ops {
                    for s in self.operand(st, o) {
                        union_into(&mut all, &s);
                    }
                }
                vec![all]
            }
            AggKind::Adt(d, v, args) | AggKind::Union(d, v, args) => {
                let mut out = vec![Vec::new(); n];
                let identity: Vec<u32> = (0..n as u32).collect();
                for (i, o) in ops.iter().enumerate() {
                    let vals = self.operand(st, o);
                    let oty = match o { Operand::Use(p) => p.ty, Operand::Const(_, t) => *t };
                    let map = self.field_slots(*d, *args, *v, i as u32, oty, &identity, vals.len() as u32);
                    for (k, val) in vals.iter().enumerate() {
                        if let Some(&s) = map.get(k)
                            && s != u32::MAX
                            && let Some(t) = out.get_mut(s as usize)
                        {
                            union_into(t, val);
                        }
                    }
                }
                out
            }
        }
    }

    fn transfer_term(&self, t: &Term, st: &mut State) {
        let Term::Call { func, args, dest, .. } = t else { return };
        let n = self.cx.slots(dest.ty) as usize;
        let argv: Vec<Vec<Set>> = args.iter().map(|a| self.operand(st, a)).collect();
        let mut result = vec![Vec::new(); n];
        let mut writes: Vec<(usize, usize, Vec<u32>)> = Vec::new();
        match func {
            Callee::Fn(d, targs) => self.call_flow(*d, *targs, dest.ty, args, &argv, n, &mut result, &mut writes),
            Callee::Value(o) => {
                // A closure or function pointer: what it holds (not what it
                // is given: its signature may not let that out).
                let mut all: Set = Vec::new();
                for s in self.operand(st, o) {
                    union_into(&mut all, &s);
                }
                // (Not the borrows of the variables it captured: it reads those.)
                all.retain(|&l| !self.loans[l as usize].capture);
                result = vec![all; n];
            }
        }
        for (j, k, loans) in writes {
            // Through the `&mut` that is argument `j`: into what it
            // points to (slot `k` of it).
            let Operand::Use(p) = &args[j] else { continue };
            let holder = self.read(st, p);
            let Some(outer) = holder.first().cloned() else { continue };
            for l in outer {
                let loan = &self.loans[l as usize];
                let pointee_ty = match self.cx.tcx.kind(p.ty) {
                    TyKind::Ref(_, i) => i,
                    _ => continue,
                };
                let target = Place { local: loan.place.local, proj: loan.place.proj.clone(), ty: pointee_ty, at: None };
                let idx = self.place_slots(&target);
                let nslots = self.cx.slots(self.cfg.locals[target.local.0 as usize].ty) as usize;
                let mut vals = vec![Vec::new(); idx.len()];
                if let Some(v) = vals.get_mut(k) {
                    *v = loans.clone();
                }
                let _ = nslots;
                self.write(st, &target, vals, false, 0);
            }
        }
        if dest.proj.is_empty() {
            st.kill(self.rooted_ids(dest.local));
        }
        fit(&mut result, n);
        if self.holds_nothing(dest.ty) {
            result = vec![Vec::new(); n];
        }
        self.write(st, dest, result, dest.proj.is_empty(), 0);
    }

    /// What a call to `d` gives its result, and what it adds to what its
    /// `&mut` arguments point to, from the signature.
    #[allow(clippy::too_many_arguments)]
    fn call_flow(&self, d: DefId, targs: crate::ty::List, dest_ty: Ty, args: &[Operand], argv: &[Vec<Set>], n: usize, result: &mut [Set], writes: &mut Vec<(usize, usize, Vec<u32>)>) {
        let cx = self.cx;
        let Some(sig) = cx.sig(d) else { return };
        if sig.inputs.len() != args.len() {
            return;
        }
        let _ = targs;
        let mut ins: Vec<Option<Vec<Vec<Sym>>>> = Vec::new();
        let mut ins_w: Vec<Option<Vec<Vec<Sym>>>> = Vec::new();
        for (j, t) in sig.inputs.iter().enumerate() {
            let mut ds = Vec::new();
            let aty = match &args[j] {
                Operand::Use(p) => p.ty,
                Operand::Const(_, t) => *t,
            };
            desc(cx, t, aty, &mut ds);
            ins.push((ds.len() == argv[j].len()).then_some(ds));
            // (What an `impl Trait<Item = T>` argument is bound to is not what it holds.)
            let mut ws = Vec::new();
            if let RTy::Opaque(a, bs) = t {
                let bs2: Vec<_> = bs
                    .iter()
                    .map(|b| {
                        let mut b2 = b.clone();
                        let sugar = b.args.len() == 1 && matches!(&b.args[0], RArg::Ty(RTy::Tuple(_))) && b.assoc.iter().any(|(n, _)| n == "Output");
                        if !sugar {
                            b2.assoc.clear();
                        }
                        b2
                    })
                    .collect();
                desc(cx, &RTy::Opaque(a.clone(), bs2), aty, &mut ws);
            } else {
                ws = ins.last().and_then(|x| x.clone()).unwrap_or_default();
            }
            ins_w.push((ws.len() == argv[j].len()).then_some(ws));
        }
        let mut out_ds = Vec::new();
        desc(cx, &sig.output, dest_ty, &mut out_ds);
        if self.trace {
            eprintln!("  call {:?}: in={:?} out={:?} ins={:?} out_ds={:?} n={n}", d, sig.inputs, sig.output, ins, out_ds);
        }
        // `for<'a> T::Assoc<'a>: 'static` lets nothing out.
        let proj_static = sig.ty_outlives.iter().any(|(t, r)| *r == Region::Static && matches!(t, RTy::Proj(_)));
        // (What a trait's method returns as `impl Trait` is what the impl chooses.)
        let opaque_static = opaque_static(&sig.output) || (has_opaque(&sig.output) && sig::parent_of(cx.r, d).is_some_and(|p| cx.r.defs[p.0 as usize].kind == crate::resolve::DefKind::Trait));
        if out_ds.len() == n && !proj_static && !opaque_static {
            for (o, syms) in out_ds.iter().enumerate() {
                for (j, ds) in ins.iter().enumerate() {
                    let Some(ds) = ds else { continue };
                    for (k, d2) in ds.iter().enumerate() {
                        if syms.iter().any(|a| d2.iter().any(|b| a.ties(b))) {
                            union_into(&mut result[o], &argv[j][k]);
                        }
                    }
                }
            }
        }
        // A `&mut` argument gets what its pointee shares a symbol with.
        for (j, t) in sig.inputs.iter().enumerate() {
            let RTy::Ref(_, Mutbl::Mut, _) = t else { continue };
            let Some(dj) = &ins[j] else { continue };
            for (k, syms) in dj.iter().enumerate().skip(1) {
                let mut loans: Set = Vec::new();
                for (j2, d2) in ins_w.iter().enumerate() {
                    if j2 == j {
                        continue;
                    }
                    let Some(d2) = d2 else { continue };
                    for (k2, s2) in d2.iter().enumerate() {
                        if syms.iter().any(|a| s2.iter().any(|b| a.ties(b))) {
                            union_into(&mut loans, &argv[j2][k2]);
                        }
                    }
                }
                if !loans.is_empty() {
                    writes.push((j, k - 1, loans));
                }
            }
        }
        let _ = args;
    }

    fn flow(&mut self) {
        let nb = self.cfg.blocks.len();
        self.entry = vec![None; nb];
        self.entry[0] = Some(self.init.clone());
        let order = self.cfg.reachable();
        for _ in 0..40 {
            let mut changed = false;
            for &b in &order {
                let Some(mut st) = self.entry[b.0 as usize].clone() else { continue };
                let data = &self.cfg.blocks[b.0 as usize];
                for (i, s) in data.stmts.iter().enumerate() {
                    self.transfer_stmt(b.0, i as u32, s, &mut st);
                }
                let Some((t, _)) = &data.term else { continue };
                self.transfer_term(t, &mut st);
                for s in t.successors() {
                    match &mut self.entry[s.0 as usize] {
                        Some(e) => changed |= e.join(&st),
                        slot @ None => {
                            *slot = Some(st.clone());
                            changed = true;
                        }
                    }
                }
            }
            if !changed {
                break;
            }
        }
    }

    // ---- conflicts ----

    fn is_copy(&mut self, solver: &mut Solver<'_, '_>, t: Ty) -> bool {
        if let Some(&c) = self.copy.get(&t) {
            return c;
        }
        let c = matches!(self.cx.tcx.kind(t), TyKind::Ref(Mutbl::Mut, _)) || solver.may_be_copy(t) || self.cx.slots(t) == 0 && false;
        self.copy.insert(t, c);
        c
    }

    /// Whether an access to `a` and a loan of `b` are of the same memory.
    fn overlap(&self, b: &Place, a: &Place, shallow: bool) -> bool {
        if a.local != b.local {
            return false;
        }
        let n = a.proj.len().min(b.proj.len());
        let mut t = self.cfg.locals[a.local.0 as usize].ty;
        for k in 0..n {
            let len = match self.cx.tcx.kind(t) {
                TyKind::Array(_, c) => match self.cx.tcx.const_of(c) {
                    crate::ty::Const::Value(n) => Some(n as u64),
                    _ => None,
                },
                _ => None,
            };
            let norm = |p: &Proj| -> Proj {
                match (p, len) {
                    (Proj::ConstIndex(i, true, e), Some(l)) if *i <= l => Proj::ConstIndex(l - *i, false, *e),
                    (Proj::Subslice(f, to, true, e), Some(l)) if *to <= l => Proj::Subslice(*f, l - *to, false, *e),
                    _ => p.clone(),
                }
            };
            t = self.cx.after(t, &a.proj[k]);
            let (pa, pb) = (norm(&a.proj[k]), norm(&b.proj[k]));
            match (&pa, &pb) {
                (Proj::Field(i, _), Proj::Field(j, _)) if i != j => return false,
                (Proj::Downcast(i), Proj::Downcast(j)) if i != j => return false,
                (Proj::ConstIndex(i, e1, _), Proj::ConstIndex(j, e2, _)) if i != j && e1 == e2 => return false,
                // (From the start against from the end, of a length not known: a
                // pattern that has both asks for a length that tells them apart.)
                (Proj::ConstIndex(_, e1, _), Proj::ConstIndex(_, e2, _)) if e1 != e2 => return false,
                (Proj::Subslice(f1, t1, false, _), Proj::Subslice(f2, t2, false, _)) if t1 <= f2 || t2 <= f1 => return false,
                (Proj::ConstIndex(i, ie, _), Proj::Subslice(from, to, se, _)) | (Proj::Subslice(from, to, se, _), Proj::ConstIndex(i, ie, _))
                    if parts_disjoint(*i, *ie, *from, *to, *se) =>
                {
                    return false;
                }
                _ => {}
            }
        }
        // The access is of a prefix of the loan's place: shallow does not
        // reach through a reference.
        if shallow && a.proj.len() < b.proj.len() {
            let mut t = self.prefix_ty(b, a.proj.len());
            for pr in &b.proj[a.proj.len()..] {
                if matches!(pr, Proj::Deref) && matches!(self.cx.tcx.kind(t), TyKind::Ref(..) | TyKind::Ptr(..)) {
                    return false;
                }
                t = self.cx.after(t, pr);
            }
        }
        true
    }

    /// The loans live where `live` locals are, in `st`.
    fn live_loans(&self, st: &State, live: &Bits) -> Set {
        let mut out: Set = Vec::new();
        for (l, slots) in &st.held {
            if live.get(*l) {
                for s in slots {
                    union_into(&mut out, s);
                }
            }
        }
        out
    }

    /// What using `p` as an operand does: a `&mut` is reborrowed (it is
    /// `&mut *p` that is borrowed), what is `Copy` is read, the rest moved.
    fn operand_access(&mut self, solver: &mut Solver<'_, '_>, p: &Place, node: NodeId) -> Access {
        let at = p.at.unwrap_or(node);
        if let TyKind::Ref(Mutbl::Mut, inner) = self.cx.tcx.kind(p.ty) {
            return Access { place: p.project(Proj::Deref, inner), kind: Kind::MutBorrow, node: at, shallow: false };
        }
        let kind = if self.is_copy(solver, p.ty) { Kind::Read } else { Kind::Move };
        Access { place: p.clone(), kind, node: at, shallow: false }
    }

    fn accesses(&mut self, solver: &mut Solver<'_, '_>, s: &Stmt, at: (u32, u32)) -> (Vec<Access>, Vec<Access>) {
        // (What is read first, then what is written.)
        let mut reads = Vec::new();
        let mut writes = Vec::new();
        let StmtKind::Assign(d, rv) = &s.kind else {
            if let StmtKind::StorageDead(l) = &s.kind {
                let ty = self.cfg.locals[l.0 as usize].ty;
                writes.push(Access { place: Place::local(*l, ty), kind: Kind::Dead, node: s.node, shallow: true });
            }
            return (reads, writes);
        };
        let mut op = |this: &mut Self, o: &Operand, out: &mut Vec<Access>| {
            if let Operand::Use(p) = o {
                out.push(this.operand_access(solver, p, s.node));
            }
        };
        match rv {
            Rvalue::Use(o) | Rvalue::Unary(_, o) | Rvalue::Cast(o, _) | Rvalue::Repeat(o, _) => op(self, o, &mut reads),
            Rvalue::Binary(_, a, b) => {
                op(self, a, &mut reads);
                op(self, b, &mut reads);
            }
            Rvalue::Aggregate(_, os) => os.iter().for_each(|o| op(self, o, &mut reads)),
            Rvalue::Ref(m, p) => {
                // (A two-phase borrow only reserves, as a shared one, until its call.)
                let reserving = s.two_phase && *m == Mutbl::Mut && self.has_activation(s);
                let kind = if *m == Mutbl::Mut && !reserving { Kind::MutBorrow } else if reserving { Kind::Reserve } else { Kind::SharedBorrow };
                reads.push(Access { place: p.clone(), kind, node: p.at.unwrap_or(s.node), shallow: false });
            }
            Rvalue::AddrOf(m, p) => {
                let kind = if *m == Mutbl::Mut { Kind::MutBorrow } else { Kind::SharedBorrow };
                reads.push(Access { place: p.clone(), kind, node: p.at.unwrap_or(s.node), shallow: false });
            }
            Rvalue::CaptureRef(p) => {
                let ids = self.capture_loans.get(&at).cloned().unwrap_or_default();
                for id in ids {
                    let loan = &self.loans[id as usize];
                    let kind = if loan.mutbl == Mutbl::Mut { Kind::MutBorrow } else { Kind::SharedBorrow };
                    reads.push(Access { place: loan.place.clone(), kind, node: s.node, shallow: false });
                }
            }
            Rvalue::Discriminant(p) if matches!(self.cx.tcx.kind(p.ty), TyKind::Adt(d, _) if self.cx.tcx.adts.get(&d).is_some_and(|a| a.variants.len() == 1)) => {}
            Rvalue::Discriminant(p) | Rvalue::Len(p) => reads.push(Access { place: p.clone(), kind: Kind::Read, node: p.at.unwrap_or(s.node), shallow: true }),
        }
        writes.push(Access { place: d.clone(), kind: Kind::Write, node: s.node, shallow: true });
        (reads, writes)
    }

    fn conflict(&mut self, st: &State, live: &Bits, acc: &Access, except: Option<u32>, pt: (u32, u32), returns: &Set) {
        if self.const_mode && acc.kind != Kind::Dead {
            return;
        }
        let mut ids = self.live_loans(st, live);
        // What an assignment overwrites is not in the way of its own loan.
        ids.retain(|&l| Some(l) != except);
        ids.sort_unstable();
        for l in ids {
            let loan = &self.loans[l as usize];
            if loan.ignored || !self.overlap(&loan.place, &acc.place, acc.shallow) {
                continue;
            }
            // A two-phase borrow only reserves until its call: it is as a
            // shared borrow to what only reads.
            let reserved = loan.reserved.contains(&pt);
            let (code, what): (&'static str, String) = match (acc.kind, loan.mutbl) {
                (Kind::Read, Mutbl::Mut) if !reserved => ("E0503", "cannot use a value while it is mutably borrowed".into()),
                (Kind::Reserve, Mutbl::Mut) if !reserved => ("E0499", "cannot borrow as mutable more than once at a time".into()),
                (Kind::SharedBorrow, Mutbl::Mut) if !reserved => ("E0502", "cannot borrow as immutable because it is also borrowed as mutable".into()),
                (Kind::MutBorrow, Mutbl::Mut) => ("E0499", "cannot borrow as mutable more than once at a time".into()),
                (Kind::MutBorrow, Mutbl::Not) => ("E0502", "cannot borrow as mutable because it is also borrowed as immutable".into()),
                (Kind::Write, _) => ("E0506", "cannot assign to a borrowed value".into()),
                (Kind::Move, _) => ("E0505", "cannot move out of a value while it is borrowed".into()),
                (Kind::Dead, _) => {
                    let local = loan.place.local;
                    // (What a macro declares is not for us to judge.)
                    let decl = if self.cfg.locals[local.0 as usize].kind == LocalKind::Temp { self.def_node.get(&local.0).copied() } else { self.cfg.locals[local.0 as usize].node };
                    if let (Some(n), Some(orig)) = (decl, self.original)
                        && !orig(n)
                    {
                        continue;
                    }
                    if self.const_mode {
                        // (Its temporaries are promoted; its variables are not.)
                        if self.cfg.locals[local.0 as usize].kind != LocalKind::Var || returns.binary_search(&l).is_err() || matches!(self.cx.tcx.kind(self.cfg.locals[0].ty), TyKind::Ptr(..)) {
                            continue;
                        }
                        ("E0597", "a borrowed value does not live long enough".into())
                    } else if loan.capture && returns.binary_search(&l).is_ok() && !self.result_def(pt.0).1 {
                        ("E0373", "closure may outlive the current function, but it borrows a variable".into())
                    } else if loan.capture && returns.binary_search(&l).is_ok() {
                        ("E0597", "a borrowed value does not live long enough".into())
                    } else if returns.binary_search(&l).is_ok() {
                        ("E0515", "cannot return a value referencing data owned by the current function".into())
                    } else if self.cfg.locals[local.0 as usize].kind == LocalKind::Temp {
                        ("E0716", "a temporary value is dropped while borrowed".into())
                    } else if loan.capture
                        && st.held.iter().any(|(&h, slots)| h != 0 && h < local.0 && live.get(h) && slots.iter().any(|s| s.contains(&l)) && !self.moved_from_temp(h))
                    {
                        // A closure that outlives what it borrows, held by a
                        // variable declared before it.
                        ("E0373", "closure may outlive the current block, but it borrows a variable".into())
                    } else {
                        ("E0597", "a borrowed value does not live long enough".into())
                    }
                }
                _ => continue,
            };
            let node = match code {
                "E0515" | "E0373" if returns.binary_search(&l).is_ok() => self.result_def(pt.0).0.unwrap_or(loan.node),
                "E0597" | "E0515" | "E0373" => loan.node,
                "E0716" => self.def_node.get(&loan.place.local.0).copied().unwrap_or(loan.node),
                _ => acc.node,
            };
            self.out.push(Raw { code, node, message: what, local: None });
            // (One error for what dies, at the first borrow of it.)
            if acc.kind == Kind::Dead {
                break;
            }
        }
    }

    /// Whether `h` is given a value by a call, or a block tail (a temporary
    /// moved into it): `let f = { let x = ..; || x };`.
    fn moved_from_temp(&self, h: u32) -> bool {
        self.cfg.blocks.iter().any(|b| {
            matches!(&b.term, Some((Term::Call { dest, .. }, _)) if dest.local.0 == h) ||
            b.stmts.iter().any(|s| {
                matches!(&s.kind, StmtKind::Assign(d, Rvalue::Use(Operand::Use(p)))
                    if d.local.0 == h && d.proj.is_empty() && self.cfg.locals[p.local.0 as usize].kind == LocalKind::Temp)
            })
        })
    }

    fn report(mut self, solver: &mut Solver<'_, '_>) -> Vec<Raw> {
        let cfg = self.cfg;
        let nl = cfg.locals.len();
        if self.trace {
            eprintln!("=== {}", cfg.name);
            for (i, l) in self.loans.iter().enumerate() {
                eprintln!("loan {i}: {:?} {:?} ignored={} at bb{}#{}", l.mutbl, (l.place.local, &l.place.proj), l.ignored, l.block, l.idx);
            }
        }
        for b in cfg.reachable() {
            let Some(mut st) = self.entry[b.0 as usize].clone() else { continue };
            let data = &cfg.blocks[b.0 as usize];
            let Some((term, term_node)) = &data.term else { continue };
            // What is live before each statement.
            let n = data.stmts.len();
            let mut live = vec![Bits::new(nl); n + 2];
            let mut out = Bits::new(nl);
            for s in term.successors() {
                out.or(&self.live_in[s.0 as usize]);
            }
            live[n + 1] = out.clone();
            let mut cur = out;
            self.live_before_term(term, &mut cur);
            live[n] = cur.clone();
            for i in (0..n).rev() {
                self.live_before(&data.stmts[i].kind, (b.0, i as u32), &mut cur);
                live[i] = cur.clone();
            }
            for (i, s) in data.stmts.iter().enumerate() {
                if self.trace {
                    eprintln!("bb{} #{i}: {:?}  live={:?}", b.0, s.kind, (0..nl as u32).filter(|&l| live[i].get(l)).collect::<Vec<_>>());
                    eprintln!("      held={:?}", st.held);
                }
                let (reads, writes) = self.accesses(solver, s, (b.0, i as u32));
                let own = self.loan_at.get(&(b.0, i as u32)).copied();
                let returns = st.held.get(&0).map(|v| v.iter().flatten().copied().collect::<Set>()).map(|mut v| {
                    v.sort_unstable();
                    v.dedup();
                    v
                });
                let returns = returns.unwrap_or_default();
                for a in &reads {
                    self.conflict(&st, &live[i], a, own, (b.0, i as u32), &returns);
                }
                for a in &writes {
                    // (A whole-variable assignment ends the loans of what it
                    // pointed to: they are not in its way unless it is.)
                    self.conflict(&st, &live[i + 1], a, own, (b.0, i as u32), &returns);
                }
                self.static_fields(s, &st);
                let env_before = self.env_holds(&st);
                self.transfer_stmt(b.0, i as u32, s, &mut st);
                self.closure_env(env_before, &st, s.node);
                self.static_params(s, &st);
                if let StmtKind::Assign(d, _) = &s.kind
                    && d.proj.is_empty()
                    && self.cfg.static_vars.contains(&d.local)
                {
                    self.static_annotation(d.local, &st, s.node);
                }
            }
            // The terminator.
            let ti = n as u32;
            match term {
                Term::Call { func, args, dest, .. } => {
                    let mut reads: Vec<Access> = Vec::new();
                    let mut ops: Vec<&Operand> = args.iter().collect();
                    if let Callee::Value(o) = func {
                        ops.push(o);
                    }
                    for o in ops {
                        if let Operand::Use(p) = o {
                            reads.push(self.operand_access(solver, p, *term_node));
                        }
                    }
                    // Two-phase borrows of this call become mutable now.
                    let acts: Vec<u32> = (0..self.loans.len() as u32).filter(|&l| self.loans[l as usize].act == Some((b.0, ti))).collect();
                    let returns = Set::new();
                    for l in acts {
                        let place = self.loans[l as usize].place.clone();
                        let node = self.loans[l as usize].node;
                        let a = Access { place, kind: Kind::MutBorrow, node, shallow: false };
                        self.conflict(&st, &live[n], &a, Some(l), (b.0, ti), &returns);
                    }
                    for a in &reads {
                        self.conflict(&st, &live[n], a, None, (b.0, ti), &returns);
                    }
                    let w = Access { place: dest.clone(), kind: Kind::Write, node: *term_node, shallow: true };
                    self.conflict(&st, &live[n + 1], &w, None, (b.0, ti), &returns);
                    if let Callee::Fn(d, _) = func {
                        self.static_requirements(*d, args, &st, *term_node);
                    }
                }
                Term::Switch { discr: Operand::Use(p), .. } => {
                    let a = Access { place: p.clone(), kind: Kind::Read, node: p.at.unwrap_or(*term_node), shallow: false };
                    self.conflict(&st, &live[n], &a, None, (b.0, ti), &Set::new());
                }
                Term::Return => {
                    // What is returned must not hold the function's own data.
                    if let Some(slots) = st.held.get(&0) {
                        let mut ids: Set = Vec::new();
                        for s in slots {
                            union_into(&mut ids, s);
                        }
                        for l in ids {
                            let loan = &self.loans[l as usize];
                            if loan.ignored || self.through_ref(&loan.place) {
                                continue;
                            }
                            if self.const_mode {
                                // (Its temporaries are promoted; its variables are not.)
                                let local = loan.place.local;
                                if self.cfg.locals[local.0 as usize].kind == LocalKind::Var
                                    && let (Some(n), Some(orig)) = (self.cfg.locals[local.0 as usize].node, self.original)
                                    && orig(n)
                                {
                                    let node = loan.node;
                                    self.out.push(Raw { code: "E0597", node, message: "a borrowed value does not live long enough".into(), local: None });
                                }
                                continue;
                            }
                            let (rnode, from_var) = self.result_def(b.0);
                            let node = rnode.unwrap_or(loan.node);
                            let code = if loan.capture && from_var { "E0597" } else if loan.capture { "E0373" } else { "E0515" };
                            let node = if code == "E0597" { loan.node } else { node };
                            self.out.push(Raw { code, node, message: "cannot return a value referencing data owned by the current function".into(), local: None });
                        }
                    }
                }
                _ => {}
            }
            let env_before = self.env_holds(&st);
            self.transfer_term(term, &mut st);
            self.closure_env(env_before, &st, *term_node);
        }
        let mut seen = std::collections::HashSet::new();
        self.out.retain(|r| seen.insert((r.code, r.node)));
        self.out
    }

    /// Where the function's result is given its value: the node (a `return`'s
    /// operand, the tail), and whether it is copied from a variable (a closure
    /// returned through a variable is E0597, not E0373). In block `b` if it is
    /// there, else the last.
    fn result_def(&self, b: u32) -> (Option<NodeId>, bool) {
        let find = |data: &BlockData| {
            if let Some((Term::Call { dest, .. }, n)) = &data.term
                && dest.local == Local(0)
                && dest.proj.is_empty()
            {
                return Some((Some(*n), false));
            }
            data.stmts.iter().rev().find_map(|s| match &s.kind {
                StmtKind::Assign(d, rv) if d.local == Local(0) && d.proj.is_empty() => {
                    let from_var = matches!(rv, Rvalue::Use(Operand::Use(p)) if self.cfg.locals[p.local.0 as usize].kind == LocalKind::Var);
                    Some((Some(s.node), from_var))
                }
                _ => None,
            })
        };
        if let Some(r) = find(&self.cfg.blocks[b as usize]) {
            return r;
        }
        self.cfg.reachable().iter().rev().find_map(|x| find(&self.cfg.blocks[x.0 as usize])).unwrap_or((None, false))
    }

    /// `let x: &'static T = ..;`: what it holds must not be borrowed from
    /// the function's own data.
    fn static_annotation(&mut self, v: Local, st: &State, at: NodeId) {
        let Some(slots) = st.held.get(&v.0) else { return };
        let Some(first) = slots.first() else { return };
        let ids = first.clone();
        self.flag_static(&ids, at);
    }

    /// Loans of the function's own data that must outlive `'static`.
    fn flag_static(&mut self, ids: &[u32], _at: NodeId) {
        if self.const_mode {
            return;
        }
        for &l in ids {
            let loan = &self.loans[l as usize];
            // (A parameter stored where `'static` is wanted is a lifetime error without a code.)
            if loan.universal {
                continue;
            }
            if loan.ignored || self.through_ref(&loan.place) {
                continue;
            }
            let local = loan.place.local;
            let temp = self.cfg.locals[local.0 as usize].kind == LocalKind::Temp;
            let decl = if temp { self.def_node.get(&local.0).copied() } else { self.cfg.locals[local.0 as usize].node };
            if let (Some(n), Some(orig)) = (decl, self.original)
                && !orig(n)
            {
                continue;
            }
            let (code, node) = if temp { ("E0716", self.def_node.get(&local.0).copied().unwrap_or(loan.node)) } else { ("E0597", loan.node) };
            self.out.push(Raw { code, node, message: "borrowed data must outlive 'static".into(), local: None });
        }
    }

    /// A value stored where the type says `'static`: into a field declared
    /// `&'static T`, before it is stored.
    fn static_fields(&mut self, s: &Stmt, st: &State) {
        let StmtKind::Assign(_, Rvalue::Aggregate(AggKind::Adt(d, v, _), ops)) = &s.kind else { return };
        let cx = self.cx;
        let Some(fields) = cx.adt_fields(*d) else { return };
        let Some(fs) = fields.get(*v as usize) else { return };
        for (i, o) in ops.iter().enumerate() {
            let Some(rty) = fs.get(i) else { continue };
            let oty = match o { Operand::Use(p) => p.ty, Operand::Const(_, t) => *t };
            let mut ds = Vec::new();
            desc(cx, rty, oty, &mut ds);
            let vals = self.operand(st, o);
            if ds.len() != vals.len() {
                continue;
            }
            for (k, syms) in ds.iter().enumerate() {
                if syms.iter().any(|x| matches!(x, Sym::Reg(Region::Static))) {
                    let ids = vals[k].clone();
                    self.flag_static(&ids, s.node);
                }
            }
        }
    }

    /// A store into a parameter (through it) whose declared type says
    /// `'static` there.
    fn static_params(&mut self, s: &Stmt, st: &State) {
        let StmtKind::Assign(d, _) = &s.kind else { return };
        let a = d.local.0 as usize;
        if self.cfg.closure.is_some() || a == 0 || a > self.cfg.arg_count || self.cfg.locals[a].kind != LocalKind::Arg {
            return;
        }
        let cx = self.cx;
        let Some(sig) = cx.sig(self.cfg.owner) else { return };
        let Some(rty) = sig.inputs.get(a - 1) else { return };
        let mut ds = Vec::new();
        desc(cx, rty, self.cfg.locals[a].ty, &mut ds);
        let idx = self.place_slots(d);
        let Some(held) = st.held.get(&d.local.0) else { return };
        for &si in &idx {
            if si == u32::MAX {
                continue;
            }
            if ds.get(si as usize).is_some_and(|x| x.iter().any(|y| matches!(y, Sym::Reg(Region::Static)))) {
                let ids = held.get(si as usize).cloned().unwrap_or_default();
                self.flag_static(&ids, s.node);
            }
        }
    }

    /// What a closure's own parameters stand for must not reach what it
    /// captured (it lives longer than any call).
    fn env_holds(&self, st: &State) -> bool {
        self.cfg.closure.is_some() && st.held.get(&1).is_some_and(|slots| slots.iter().flatten().any(|&l| self.loans[l as usize].universal))
    }

    fn closure_env(&mut self, before: bool, st: &State, at: NodeId) {
        if !before && self.env_holds(st) {
            self.out.push(Raw { code: "E0521", node: at, message: "borrowed data escapes outside of closure".into(), local: None });
        }
    }

    /// A call whose signature wants `'static` (`&'static T`, `F: 'static`) of
    /// an argument that holds a borrow of the function's own data.
    fn static_requirements(&mut self, d: DefId, args: &[Operand], st: &State, at: NodeId) {
        let cx = self.cx;
        let Some(sig) = cx.sig(d) else { return };
        if sig.inputs.len() != args.len() {
            return;
        }
        let statics: Vec<ParamId> = sig.ty_outlives.iter().filter_map(|(t, r)| if let (RTy::Param(p), Region::Static) = (t, r) { Some(*p) } else { None }).collect();
        for (j, t) in sig.inputs.iter().enumerate() {
            let aty = match &args[j] {
                Operand::Use(p) => p.ty,
                Operand::Const(_, t) => *t,
            };
            let mut ds = Vec::new();
            desc(cx, t, aty, &mut ds);
            let slots = self.operand(st, &args[j]);
            if ds.len() != slots.len() {
                continue;
            }
            for (k, syms) in ds.iter().enumerate() {
                let needs = syms.iter().any(|s| match s {
                    Sym::Reg(Region::Static) => true,
                    Sym::Par(p, _) | Sym::ParAny(p) => statics.contains(p),
                    _ => false,
                });
                if !needs {
                    continue;
                }
                for &l in &slots[k] {
                    let loan = &self.loans[l as usize];
                    if loan.universal {
                        let node = match &args[j] {
                            Operand::Use(p) => p.at.unwrap_or(at),
                            Operand::Const(..) => at,
                        };
                        self.out.push(Raw { code: "E0521", node, message: "borrowed data escapes outside of the function".into(), local: None });
                        continue;
                    }
                    if loan.ignored || self.through_ref(&loan.place) {
                        continue;
                    }
                    let local = loan.place.local;
                    let decl = if self.cfg.locals[local.0 as usize].kind == LocalKind::Temp { self.def_node.get(&local.0).copied() } else { self.cfg.locals[local.0 as usize].node };
                    if let (Some(n), Some(orig)) = (decl, self.original)
                        && !orig(n)
                    {
                        continue;
                    }
                    let from_var = matches!(&args[j], Operand::Use(p) if self.cfg.locals[p.local.0 as usize].kind == LocalKind::Var);
                    let (code, node) = if loan.capture && !from_var {
                        ("E0373", loan.node)
                    } else if self.cfg.locals[local.0 as usize].kind == LocalKind::Temp {
                        ("E0716", self.def_node.get(&local.0).copied().unwrap_or(loan.node))
                    } else {
                        ("E0597", loan.node)
                    };
                    self.out.push(Raw { code, node, message: "borrowed data must outlive `'static`".into(), local: None });
                }
            }
        }
    }

    /// Whether `p` goes through a reference or a pointer (so it is not the
    /// function's own data).
    fn through_ref(&self, p: &Place) -> bool {
        let mut t = self.cfg.locals[p.local.0 as usize].ty;
        for pr in &p.proj {
            if matches!(pr, Proj::Deref) && matches!(self.cx.tcx.kind(t), TyKind::Ref(..) | TyKind::Ptr(..)) {
                return true;
            }
            t = self.cx.after(t, pr);
        }
        false
    }
}

/// Makes `v` have `n` sets: cut, or (if it is not what the type has)
/// filled with what it holds in all.
fn fit(v: &mut Vec<Set>, n: usize) {
    if v.len() == n {
        return;
    }
    if v.len() > n || v.is_empty() {
        let mut all: Set = Vec::new();
        for s in v.iter() {
            union_into(&mut all, s);
        }
        *v = if n == 0 || all.is_empty() { vec![Vec::new(); n] } else { vec![all; n] };
        return;
    }
    let mut all: Set = Vec::new();
    for s in v.iter() {
        union_into(&mut all, s);
    }
    v.resize(n, all);
}

/// The places an rvalue reads.
fn rvalue_places(rv: &Rvalue) -> Vec<&Place> {
    fn op(o: &Operand) -> Option<&Place> {
        match o {
            Operand::Use(p) => Some(p),
            Operand::Const(..) => None,
        }
    }
    match rv {
        Rvalue::Use(o) | Rvalue::Unary(_, o) | Rvalue::Cast(o, _) | Rvalue::Repeat(o, _) => op(o).into_iter().collect(),
        Rvalue::Binary(_, a, b) => op(a).into_iter().chain(op(b)).collect(),
        Rvalue::Aggregate(_, os) => os.iter().filter_map(|o| op(o)).collect(),
        Rvalue::Ref(_, p) | Rvalue::AddrOf(_, p) | Rvalue::CaptureRef(p) | Rvalue::Discriminant(p) | Rvalue::Len(p) => vec![p],
    }
}

/// The temporaries that are constants, which a borrow of is promoted to a
/// `'static` (`&[]`, `&0`, `&Foo { x: 1 }`): with whether it is an empty
/// array (which `&mut` is promoted too).
fn promotable(cfg: &Cfg, cx: &Cx) -> FastMap<u32, bool> {
    let mut calls: FastMap<u32, (DefId, &Vec<Operand>)> = FastMap::default();
    let mut defs: FastMap<u32, Vec<Option<&Rvalue>>> = FastMap::default();
    for b in cfg.reachable() {
        let data = &cfg.blocks[b.0 as usize];
        for s in &data.stmts {
            match &s.kind {
                StmtKind::Assign(d, rv) if d.proj.is_empty() => defs.entry(d.local.0).or_default().push(Some(rv)),
                StmtKind::Assign(d, _) => defs.entry(d.local.0).or_default().push(None),
                _ => {}
            }
        }
        if let Some((Term::Call { dest, func, args, .. }, _)) = &data.term {
            defs.entry(dest.local.0).or_default().push(None);
            if let (Callee::Fn(d, _), true) = (func, dest.proj.is_empty()) {
                let def = &cx.r.defs[d.0 as usize];
                let ast = &cx.crates[def.krate as usize].files[def.file as usize].ast;
                if crate::session::has_attribute(ast, def.node, "rustc_promotable") {
                    calls.insert(dest.local.0, (*d, args));
                }
            }
        }
    }
    let mut out: FastMap<u32, bool> = FastMap::default();
    loop {
        let mut changed = false;
        for (l, ds) in &defs {
            if out.contains_key(l) || cfg.locals[*l as usize].kind != LocalKind::Temp || ds.len() != 1 {
                continue;
            }
            let ok = |o: &Operand| match o {
                Operand::Const(..) => true,
                Operand::Use(p) => {
                    out.contains_key(&p.local.0)
                        && p.proj.iter().all(|x| match x {
                            Proj::Field(..) | Proj::ConstIndex(..) | Proj::Downcast(_) => true,
                            Proj::Index(i, _) => out.contains_key(&i.0),
                            _ => false,
                        })
                }
            };
            if let Some((_, cargs)) = calls.get(l) {
                if cargs.iter().all(ok) {
                    out.insert(*l, false);
                    changed = true;
                }
                continue;
            }
            let Some(rv) = ds[0] else { continue };
            let good = match rv {
                Rvalue::Use(o) | Rvalue::Unary(_, o) | Rvalue::Cast(o, _) | Rvalue::Repeat(o, _) => ok(o),
                Rvalue::Binary(_, a, b) => ok(a) && ok(b),
                Rvalue::Aggregate(k, os) => !matches!(k, AggKind::Union(..)) && os.iter().all(ok),
                Rvalue::Ref(Mutbl::Not, p) => p.proj.is_empty() && out.contains_key(&p.local.0),
                _ => false,
            };
            if good {
                out.insert(*l, matches!(rv, Rvalue::Aggregate(AggKind::Array | AggKind::Tuple, os) if os.is_empty()));
                changed = true;
            }
        }
        if !changed {
            return out;
        }
    }
}

/// Whether element `i` (from the end if `ie`) is outside the subslice
/// `from..to` (`to` from the end if `se`).
fn parts_disjoint(i: u64, ie: bool, from: u64, to: u64, se: bool) -> bool {
    match (ie, se) {
        (false, true) => i < from,
        (true, true) => i <= to,
        (false, false) => i < from || i >= to,
        (true, false) => false,
    }
}

/// An `impl Trait + 'static`: it holds no loan.
fn opaque_static(t: &RTy) -> bool {
    match t {
        RTy::Opaque(a, _) => a.iter().any(|x| matches!(x, RArg::Region(Region::Static))),
        RTy::Ref(_, _, i) | RTy::Ptr(_, i) | RTy::Slice(i) | RTy::Array(i) => opaque_static(i),
        RTy::Tuple(ts) => ts.iter().any(opaque_static),
        RTy::Adt(_, a) => a.iter().any(|x| matches!(x, RArg::Ty(t) if opaque_static(t))),
        _ => false,
    }
}

/// How a closure uses one of the variables it captured.
#[derive(Clone, Default, Debug)]
pub struct UpvarUse {
    /// It changes it (or reborrows it as `&mut`).
    pub mutated: bool,
    /// The parts of it used (field paths); empty if all of it is.
    pub paths: Vec<Vec<Proj>>,
    pub used: bool,
}

/// For each closure, how its body uses each captured variable.
/// Which parameters of each closure the function it is passed to leaves
/// to any lifetime: `with_int(|y| ..)` for `F: FnOnce(&isize)`.
/// The closure a reference local was made from, through borrows and casts.
fn closure_behind(cfg: &Cfg, mut l: Local, tcx: &TyCtxt) -> Option<ClosureId> {
    for _ in 0..4 {
        let mut next = None;
        for b in &cfg.blocks {
            for s in &b.stmts {
                let StmtKind::Assign(d, rv) = &s.kind else { continue };
                if d.local != l || !d.proj.is_empty() {
                    continue;
                }
                match rv {
                    Rvalue::Ref(_, q) => {
                        if let TyKind::Closure(c, _, _) = tcx.kind(q.ty) {
                            return Some(c);
                        }
                    }
                    Rvalue::Use(Operand::Use(q)) | Rvalue::Cast(Operand::Use(q), _) if q.proj.is_empty() => next = Some(q.local),
                    _ => {}
                }
            }
        }
        l = next?;
    }
    None
}

pub fn closure_hr<'s>(cfgs: &[Cfg], crates: &[ParsedCrate<'s>], r: &Resolver<'s>, tcx: &TyCtxt<'s>) -> FastMap<ClosureId, Vec<bool>> {
    let mut out: FastMap<ClosureId, Vec<bool>> = FastMap::default();
    let mut sigs: FastMap<DefId, Option<FnRegions>> = FastMap::default();
    for cfg in cfgs {
        for b in &cfg.blocks {
            let Some((Term::Call { func: Callee::Fn(d, _), args, .. }, _)) = &b.term else { continue };
            for (j, a) in args.iter().enumerate() {
                let Operand::Use(p) = a else { continue };
                let sig = sigs.entry(*d).or_insert_with(|| sig::fn_regions(crates, r, tcx, *d));
                let Some(sig) = sig else { continue };
                if let Some(RTy::Ref(_, _, inner)) = sig.inputs.get(j) {
                    // `&mut |y| ..` unsized to `&mut dyn FnMut(&T)`.
                    if let RTy::Dyn(_, bs) = &**inner {
                        if let Some(c) = closure_behind(cfg, p.local, tcx) {
                            for bound in bs {
                                if let Some(RArg::Ty(RTy::Tuple(ts))) = bound.args.first() {
                                    out.insert(c, ts.iter().map(|t| matches!(t, RTy::Ref(Region::Bound(_), _, _))).collect());
                                }
                            }
                        }
                    }
                    continue;
                }
                let TyKind::Closure(c, _, _) = tcx.kind(p.ty) else { continue };
                let Some(RTy::Param(pp)) = sig.inputs.get(j) else { continue };
                for (t, bound) in &sig.bounds {
                    if !matches!(t, RTy::Param(q) if q == pp) {
                        continue;
                    }
                    let Some(RArg::Ty(RTy::Tuple(ts))) = bound.args.first() else { continue };
                    out.insert(c, ts.iter().map(|t| matches!(t, RTy::Ref(Region::Bound(_), _, _))).collect());
                }
            }
        }
    }
    out
}

pub fn closure_uses(cfgs: &[Cfg], tcx: &TyCtxt) -> FastMap<ClosureId, Vec<UpvarUse>> {
    let is_mut_ref = |t: Ty| matches!(tcx.kind(t), TyKind::Ref(Mutbl::Mut, _));
    let mut out: FastMap<ClosureId, Vec<UpvarUse>> = FastMap::default();
    for cfg in cfgs {
        let Some(c) = cfg.closure else { continue };
        let mut uses = vec![UpvarUse::default(); cfg.upvars.len()];
        let mut visit = |p: &Place, mutating: bool| {
            if p.local != Local(1) {
                return;
            }
            let Some(Proj::Field(i, _)) = p.proj.first() else { return };
            let Some(&(_, by_value, _)) = cfg.upvars.get(*i as usize) else { return };
            let start = if by_value { 1 } else { 2 };
            let rest = p.proj.get(start..).unwrap_or(&[]);
            let path: Vec<Proj> = rest.iter().take_while(|x| matches!(x, Proj::Field(..) | Proj::Deref | Proj::Downcast(_))).cloned().collect();
            let u = &mut uses[*i as usize];
            u.used = true;
            u.mutated |= mutating;
            if !u.paths.contains(&path) {
                u.paths.push(path);
            }
        };
        for b in cfg.reachable() {
            let data = &cfg.blocks[b.0 as usize];
            for s in &data.stmts {
                if let StmtKind::Assign(d, rv) = &s.kind {
                    visit(d, true);
                    match rv {
                        Rvalue::Ref(m, p) | Rvalue::AddrOf(m, p) => visit(p, *m == Mutbl::Mut),
                        _ => {
                            for p in rvalue_places(rv) {
                                // A `&mut` passed on is reborrowed: used uniquely.
                                visit(p, is_mut_ref(p.ty) && matches!(rv, Rvalue::Use(_) | Rvalue::Aggregate(..)));
                            }
                        }
                    }
                }
            }
            if let Some((Term::Call { func, args, dest, .. }, _)) = &data.term {
                visit(dest, true);
                for a in args {
                    if let Operand::Use(p) = a {
                        visit(p, is_mut_ref(p.ty));
                    }
                }
                if let Callee::Value(Operand::Use(p)) = func {
                    visit(p, false);
                }
            }
        }
        out.insert(c, uses);
    }
    out
}
