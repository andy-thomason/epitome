//! Drops: the drop glue of each type, and dropping what a body owns when
//! it goes out of scope.
//!
//! Everything a body owns that needs dropping is a *root*: a local, a
//! parameter, or a temporary (a value expression used as a place). Each
//! root belongs to a *scope*, and is dropped when control leaves the
//! scope: at its end, or on `break`, `continue`, `return` or `?` past it.
//! Scopes are blocks (for locals), statements and the other temporary
//! scopes of the reference (for temporaries), and the function (for
//! parameters). A scope drops its roots in the reverse of the order they
//! were made.
//!
//! Each root has a *drop flag*, a byte in the frame: set when the root is
//! initialised, cleared when it is moved out of or dropped. A root is
//! dropped only if its flag is set, so a value moved away on one path and
//! not another is dropped exactly once. The optimiser removes flags whose
//! value is known.
//!
//! Moving out of part of a root (a field, or what a pattern binds) gives
//! that *move path* a flag of its own. Then the root is dropped part by
//! part: each part not moved is dropped, and each moved part only if its
//! flag says it came back. Which parts of a root are moved is only known
//! once the whole body is lowered, so each place that drops (or
//! initialises) a root that could have parts moved is a block of its own,
//! left empty and filled in at the end (`finish_drops`).

use super::body::{Fx, Operand, Place, Pv, R, Val};
use super::{Cx, Decl, Repr};
use crate::ast::{Kind, NodeId};
use crate::typeck::Adjust;
use crate::mono::Item;
use crate::resolve::Res;
use crate::ty::{AdtKind, Mutbl, Ty, TyKind};
use epitome_ir::build::FunctionBuilder;
use epitome_ir::module::{Align, BlockId, Function as IrFunction, ICmpPred, Linkage, Module, Op, Value};

/// A step into a value: variant (0 but for an enum) and field; or
/// `DEREF`, into what a `Box` holds.
pub(crate) type Elem = (u32, usize);

/// The step into what a `Box` holds (`*b`).
pub(crate) const DEREF: Elem = (u32::MAX, 0);

/// A part of a root: the root, and the steps to the part.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct MovePath {
    pub root: usize,
    pub path: Vec<Elem>,
}

/// Something the body owns and drops.
pub(crate) struct Root {
    pub place: Place,
    /// The drop flag's address (an `i8`).
    flag: Value,
    /// The parts moved out of it, each with its flag.
    paths: Vec<(Vec<Elem>, Value)>,
    /// Whether parts of it can be moved (a struct, tuple, enum or closure
    /// with no `Drop` impl), so that its drops wait for `finish_drops`.
    deferred: bool,
}

/// Where control leaves (or enters) roots.
pub(crate) struct Scope {
    roots: Vec<usize>,
    /// A block or the function: what locals belong to. Other scopes are
    /// for temporaries.
    pub block: bool,
}

/// A block left for `finish_drops` to fill in, then go on to `cont`.
struct Site {
    block: BlockId,
    cont: BlockId,
    root: usize,
    /// Drop the part at this path (the whole root if empty), or mark it
    /// (and every part under it) initialised.
    path: Vec<Elem>,
    drop: bool,
}

/// What a body knows of its roots and scopes.
#[derive(Default)]
pub(crate) struct Drops {
    pub roots: Vec<Root>,
    pub scopes: Vec<Scope>,
    /// The root each local (by its binding's key) is.
    pub locals: crate::hash::FastMap<(u32, u32, NodeId), usize>,
    /// The root each value expression used as a place became.
    pub temps: crate::hash::FastMap<NodeId, usize>,
    /// Temporaries whose scope a `let` extends: the scope they go in.
    pub extended: crate::hash::FastMap<NodeId, usize>,
    /// Bindings a pattern made, moving out of the path, not yet committed
    /// (until the pattern, and a match arm's guard, have matched).
    pub pending: Vec<(Option<MovePath>, (u32, u32, NodeId))>,
    /// While matching a pattern: the path of the place being matched.
    pub pat_path: Option<MovePath>,
    /// While matching a slice pattern: how deep.
    pub in_slice: usize,
    sites: Vec<Site>,
}

impl Fx<'_, '_, '_, '_> {
    /// Whether dropping a value of `t` runs code.
    pub fn needs_drop(&mut self, t: Ty) -> bool {
        let cx = self.cx;
        !crate::mono::has_param_or_infer(cx.tcx, t) && cx.drops.needs_drop(cx.tcx, cx.typeck, &mut self.solver, t)
    }

    // ---- scopes ----

    pub fn push_scope(&mut self, block: bool) {
        self.drops.scopes.push(Scope { roots: Vec::new(), block });
    }

    /// Leaves the innermost scope: drops its roots.
    pub fn pop_scope(&mut self) -> R<()> {
        let depth = self.drops.scopes.len() - 1;
        self.drop_scope(depth)?;
        self.drops.scopes.pop();
        Ok(())
    }

    /// Leaves the innermost scope without dropping (its drops were
    /// emitted on each way out already).
    pub fn forget_scope(&mut self) {
        self.drops.scopes.pop();
    }

    /// Drops the roots of scope `i`, the last made first.
    pub fn drop_scope(&mut self, i: usize) -> R<()> {
        let roots = self.drops.scopes[i].roots.clone();
        for &r in roots.iter().rev() {
            self.drop_root(r)?;
        }
        Ok(())
    }

    /// Drops every scope from `depth` in, the innermost first: for a jump
    /// out of them.
    pub fn drop_scopes_to(&mut self, depth: usize) -> R<()> {
        for i in (depth..self.drops.scopes.len()).rev() {
            self.drop_scope(i)?;
        }
        Ok(())
    }

    /// Whether any scope from `depth` in owns anything.
    pub fn owns_any(&self, depth: usize) -> bool {
        self.drops.scopes[depth..].iter().any(|s| !s.roots.is_empty())
    }

    /// The innermost block scope (where a `let` puts its locals).
    pub fn block_scope(&self) -> usize {
        self.drops.scopes.iter().rposition(|s| s.block).unwrap_or(0)
    }

    // ---- roots ----

    /// Makes `p` a root of scope `scope`, not yet initialised.
    pub fn new_root(&mut self, p: Place, scope: usize) -> R<usize> {
        let flag = self.flag_slot();
        let deferred = self.can_split(p.ty);
        let r = self.drops.roots.len();
        self.drops.roots.push(Root { place: p, flag, paths: Vec::new(), deferred });
        self.drops.scopes[scope].roots.push(r);
        self.set_flag(flag, false);
        Ok(r)
    }

    /// A drop flag: a byte in the frame, clear on entry.
    fn flag_slot(&mut self) -> Value {
        let here = self.b.current_block();
        let entry = self.entry();
        self.b.at(entry);
        let i8t = self.i8t();
        let ptr = self.ptr_t();
        let addr = self
            .b
            .push(
                Op::Alloca { inalloca: false, ty: i8t, count: None, align: Align::new(1), addrspace: None },
                Some(ptr),
            )
            .unwrap();
        let z = self.b.const_int(i8t, 0);
        self.store_imm(z, addr, 1);
        if let Some(h) = here {
            self.b.at(h);
        }
        addr
    }

    fn set_flag(&mut self, flag: Value, on: bool) {
        if self.terminated() {
            return;
        }
        let i8t = self.i8t();
        let v = self.b.const_int(i8t, on as i64);
        self.store_imm(v, flag, 1);
    }

    /// Whether parts of a value of `t` can be moved out of it.
    fn can_split(&mut self, t: Ty) -> bool {
        let tcx = self.cx.tcx;
        match tcx.kind(t) {
            TyKind::Tuple(_) | TyKind::Closure(..) => true,
            // What a box holds can be moved out of it.
            TyKind::Adt(d, _) if tcx.lang.get("owned_box") == Some(&d) => true,
            TyKind::Adt(d, _) => !self.cx.drops.has_impl(d) && tcx.adts.get(&d).is_some_and(|a| a.kind != AdtKind::Union),
            _ => false,
        }
    }

    /// Makes `p` a root of scope `scope`, initialised.
    pub fn owned(&mut self, p: Place, scope: usize) -> R<usize> {
        let r = self.new_root(p, scope)?;
        self.init(&MovePath { root: r, path: Vec::new() })?;
        Ok(r)
    }

    /// A value expression `n` used as a place: its temporary, a root of
    /// the innermost scope (or the one a `let` extends it to).
    pub fn temporary(&mut self, n: NodeId, pv: Pv) -> R<Place> {
        let p = self.place_of(pv)?;
        if let Pv::Val(o) = pv
            && self.needs_drop(o.ty)
        {
            let scope = self.drops.extended.get(&n).copied().unwrap_or(self.drops.scopes.len() - 1);
            let r = self.owned(p, scope)?;
            self.drops.temps.insert(n, r);
        }
        Ok(p)
    }

    /// A value an expression statement makes and nothing uses: dropped at
    /// the end of the statement.
    pub fn discard(&mut self, pv: Pv) -> R<()> {
        if let Pv::Val(o) = pv
            && !matches!(o.v, Val::Zst)
            && self.needs_drop(o.ty)
        {
            let p = self.place_of(pv)?;
            let scope = self.drops.scopes.len() - 1;
            self.owned(p, scope)?;
        }
        Ok(())
    }

    /// The flag of the part of root `r` at `path`, made the first time.
    fn path_flag(&mut self, r: usize, path: &[Elem]) -> Value {
        if let Some((_, f)) = self.drops.roots[r].paths.iter().find(|(p, _)| p == path) {
            return *f;
        }
        let f = self.flag_slot();
        self.drops.roots[r].paths.push((path.to_vec(), f));
        f
    }

    /// Records that the part at `mp` was moved out.
    pub fn moved(&mut self, mp: &MovePath) -> R<()> {
        if self.terminated() {
            return Ok(());
        }
        let flag = if mp.path.is_empty() {
            self.drops.roots[mp.root].flag
        } else {
            if !self.drops.roots[mp.root].deferred {
                return Err("moving out of part of a value whose parts cannot be moved".into());
            }
            self.path_flag(mp.root, &mp.path)
        };
        self.set_flag(flag, false);
        Ok(())
    }

    /// Records that the part at `mp` (and everything in it) holds a value.
    pub fn init(&mut self, mp: &MovePath) -> R<()> {
        if self.terminated() {
            return Ok(());
        }
        if !self.drops.roots[mp.root].deferred {
            if mp.path.is_empty() {
                let f = self.drops.roots[mp.root].flag;
                self.set_flag(f, true);
            }
            return Ok(());
        }
        self.site(mp.root, mp.path.clone(), false);
        Ok(())
    }

    /// Drops root `r` if its flag is set.
    pub fn drop_root(&mut self, r: usize) -> R<()> {
        self.drop_part(&MovePath { root: r, path: Vec::new() })
    }

    /// Drops the part at `mp` if it holds a value (before it is assigned).
    pub fn drop_part(&mut self, mp: &MovePath) -> R<()> {
        if self.terminated() {
            return Ok(());
        }
        if self.drops.roots[mp.root].deferred {
            self.site(mp.root, mp.path.clone(), true);
            return Ok(());
        }
        let root = &self.drops.roots[mp.root];
        let (place, flag) = (root.place, root.flag);
        self.if_flag(flag, |fx| {
            fx.drop_place(place)?;
            fx.set_flag(flag, false);
            Ok(())
        })
    }

    /// Runs `body` only if the byte at `flag` is set.
    fn if_flag(&mut self, flag: Value, body: impl FnOnce(&mut Self) -> R<()>) -> R<()> {
        let i8t = self.i8t();
        let v = self.load_imm(i8t, flag, 1);
        let on = self.truthy(v);
        let then = self.new_block("drop");
        let next = self.new_block("drop.next");
        self.b.cond_br(on, then, next);
        self.b.at(then);
        body(self)?;
        self.goto(next);
        self.b.at(next);
        Ok(())
    }

    /// A block for `finish_drops` to fill in.
    fn site(&mut self, root: usize, path: Vec<Elem>, drop: bool) {
        let block = self.new_block(if drop { "drop.site" } else { "init.site" });
        let cont = self.new_block("site.next");
        self.b.br(block);
        self.b.at(cont);
        self.drops.sites.push(Site { block, cont, root, path, drop });
    }

    /// Fills in each drop and initialisation left for when every move
    /// path of the body is known.
    pub fn finish_drops(&mut self) -> R<()> {
        let sites = std::mem::take(&mut self.drops.sites);
        for s in sites {
            self.b.at(s.block);
            let (place, flag, paths) = {
                let root = &self.drops.roots[s.root];
                (root.place, root.flag, root.paths.clone())
            };
            if s.drop {
                // The part's place (along fields; parts moved by patterns
                // are only dropped with their root).
                let mut p = place;
                for &e in &s.path {
                    p = self.step(p, e)?;
                }
                let path = s.path.clone();
                self.if_flag(flag, |fx| {
                    fx.drop_path(p, s.root, &path)?;
                    if path.is_empty() {
                        fx.set_flag(flag, false);
                    }
                    Ok(())
                })?;
            } else {
                if s.path.is_empty() {
                    self.set_flag(flag, true);
                }
                for (path, f) in paths {
                    if path.starts_with(&s.path) {
                        self.set_flag(f, true);
                    }
                }
            }
            self.goto(s.cont);
        }
        Ok(())
    }

    /// The part of the value at `p` that step `e` leads to.
    fn step(&mut self, p: Place, e: Elem) -> R<Place> {
        if e == DEREF {
            let o = self.load(p)?;
            return self.deref(o);
        }
        self.field(p, e.0, e.1)
    }

    /// Drops what a box holds no longer: frees it (its `Drop`), and drops
    /// its allocator.
    fn drop_box_shell(&mut self, p: Place) -> R<()> {
        let tcx = self.cx.tcx;
        let TyKind::Adt(d, _) = tcx.kind(p.ty) else { return Err("not a `Box`".into()) };
        let tr = *tcx.lang.get("drop").ok_or("no `Drop`")?;
        let m = self.trait_method(tr, "drop").ok_or("no `Drop::drop`")?;
        let rt = tcx.mk(TyKind::Ref(Mutbl::Mut, p.ty));
        let recv = self.addr_of(p, rt)?;
        self.call_def(m, vec![p.ty], vec![recv], None)?;
        let n = tcx.adts.get(&d).map_or(0, |a| a.variants[0].fields.len());
        self.drop_fields(p, 0, n)
    }

    /// Drops the part of root `r` at `prefix` (at `p`): whole if no part
    /// of it was moved; else each part not moved, and each moved part
    /// whose flag says it holds a value again.
    fn drop_path(&mut self, p: Place, r: usize, prefix: &[Elem]) -> R<()> {
        let paths = self.drops.roots[r].paths.clone();
        let own = paths.iter().find(|(x, _)| x == prefix).map(|(_, f)| *f);
        let split = paths.iter().any(|(x, _)| x.len() > prefix.len() && x.starts_with(prefix));
        let go = |fx: &mut Self| -> R<()> {
            if !split {
                return fx.drop_place(p);
            }
            let tcx = fx.cx.tcx;
            let step = |fx: &mut Self, v: u32, n: usize| -> R<()> {
                for i in 0..n {
                    let ft = fx.field_ty(p.ty, v, i)?;
                    if fx.needs_drop(ft) {
                        let fp = fx.field(p, v, i)?;
                        let mut sub = prefix.to_vec();
                        sub.push((v, i));
                        fx.drop_path(fp, r, &sub)?;
                    }
                }
                Ok(())
            };
            match tcx.kind(p.ty) {
                // A box: what it holds (if still there), then the box.
                TyKind::Adt(d, _) if tcx.lang.get("owned_box") == Some(&d) => {
                    let o = fx.load(p)?;
                    let inner = fx.deref(o)?;
                    let mut sub = prefix.to_vec();
                    sub.push(DEREF);
                    fx.drop_path(inner, r, &sub)?;
                    fx.drop_box_shell(p)
                }
                TyKind::Tuple(l) => step(fx, 0, tcx.tys(l).len()),
                TyKind::Closure(c, _, l) => {
                    let n = fx.cx.lcx.upvar_tys(c, l).map_or(0, |v| v.len());
                    step(fx, 0, n)
                }
                TyKind::Adt(d, _) => {
                    let adt = tcx.adts.get(&d).ok_or("not an ADT")?;
                    let counts: Vec<usize> = adt.variants.iter().map(|v| v.fields.len()).collect();
                    if adt.kind != AdtKind::Enum {
                        return step(fx, 0, counts[0]);
                    }
                    for (v, &n) in counts.iter().enumerate() {
                        let is = fx.test_variant(p, v as u32)?;
                        let then = fx.new_block("drop.variant");
                        let next = fx.new_block("drop.variant.next");
                        fx.b.cond_br(is, then, next);
                        fx.b.at(then);
                        step(fx, v as u32, n)?;
                        fx.goto(next);
                        fx.b.at(next);
                    }
                    Ok(())
                }
                _ => fx.drop_place(p),
            }
        };
        match own {
            Some(f) => self.if_flag(f, go),
            None => go(self),
        }
    }

    /// Calls the drop glue of the value at `p`.
    pub fn drop_place(&mut self, p: Place) -> R<()> {
        if self.terminated() || !self.needs_drop(p.ty) {
            return Ok(());
        }
        let tcx = self.cx.tcx;
        let decl = self
            .cx
            .fns
            .get(&Item::DropGlue(p.ty))
            .cloned()
            .ok_or_else(|| format!("drop glue of {} was not declared", tcx.display(self.cx.r, p.ty)))?;
        let pt = tcx.mk(TyKind::Ptr(Mutbl::Mut, p.ty));
        let arg = self.addr_of(p, pt)?;
        let g = decl.g;
        self.emit_call(&decl.abi, |b, a| b.call(g, a), vec![arg])?;
        Ok(())
    }

    // ---- moves ----

    /// The part of a root expression `n` (a place expression) names, if it
    /// names one: a local, a field of one, or a temporary.
    pub fn move_path(&mut self, n: NodeId) -> R<Option<MovePath>> {
        let tcx = self.cx.tcx;
        // Through boxes autoderef went into (`self.rust_start` of a
        // `self: Box<Self>`): steps into what each holds. Any other
        // adjustment (through a reference) names nothing to move.
        if let Some(adj) = self.body.tables.adjustments.get(&n).cloned()
            && !adj.is_empty()
        {
            let mut t = self.ty(n)?;
            for a in &adj {
                match (a, tcx.kind(t)) {
                    (Adjust::Deref, TyKind::Adt(d, l)) if tcx.lang.get("owned_box") == Some(&d) => t = tcx.tys(l)[0],
                    _ => return Ok(None),
                }
            }
            let Some(mut mp) = self.move_path_unadjusted(n)? else { return Ok(None) };
            mp.path.extend(std::iter::repeat_n(DEREF, adj.len()));
            return Ok(Some(mp));
        }
        self.move_path_unadjusted(n)
    }

    /// `move_path` of `n` itself, before its adjustments.
    fn move_path_unadjusted(&mut self, n: NodeId) -> R<Option<MovePath>> {
        let ast = self.body.ast;
        let tcx = self.cx.tcx;
        if let Some(&r) = self.drops.temps.get(&n) {
            return Ok(Some(MovePath { root: r, path: Vec::new() }));
        }
        let kids = self.kids(n);
        match ast.kind(n) {
            Kind::Paren => self.move_path(*kids.last().unwrap()),
            Kind::PathExpr => match self.res(kids[0]) {
                Some(Res::Local(lp)) => {
                    let key = (self.body.k, self.body.f, *lp);
                    if let Some(&r) = self.drops.locals.get(&key) {
                        return Ok(Some(MovePath { root: r, path: Vec::new() }));
                    }
                    Ok(self.capture_path(key))
                }
                _ => Ok(None),
            },
            Kind::Unary if ast.token(kids[0]).0 == "*" => {
                let bt = self.ty(kids[1])?;
                let is_box = matches!(tcx.kind(bt), TyKind::Adt(d, _) if tcx.lang.get("owned_box") == Some(&d));
                if !is_box {
                    return Ok(None);
                }
                let Some(mut mp) = self.move_path(kids[1])? else { return Ok(None) };
                mp.path.push(DEREF);
                Ok(Some(mp))
            }
            Kind::Field => {
                let Some(mut mp) = self.move_path(kids[0])? else { return Ok(None) };
                let bt = self.ty(kids[0])?;
                let name = ast.token(kids[1]).0;
                let i = match tcx.kind(bt) {
                    TyKind::Adt(d, _) => {
                        tcx.adts.get(&d).and_then(|a| a.variants[0].fields.iter().position(|f| f.name == name))
                    }
                    TyKind::Tuple(_) => name.parse::<usize>().ok(),
                    _ => None,
                };
                let Some(i) = i else { return Ok(None) };
                mp.path.push((0, i));
                Ok(Some(mp))
            }
            _ => Ok(None),
        }
    }

    /// Records the move of what place expression `n` (at `p`) names, if it
    /// needs dropping.
    pub fn move_out(&mut self, n: NodeId, p: Place) -> R<()> {
        if !self.needs_drop(p.ty) {
            return Ok(());
        }

        if let Some(mp) = self.move_path(n)? {
            self.moved(&mp)?;
        }
        Ok(())
    }

    /// Records the move of local `key` (captured, or named in a struct
    /// literal's shorthand field).
    pub fn move_local(&mut self, key: (u32, u32, NodeId), t: Ty) -> R<()> {
        if !self.needs_drop(t) {
            return Ok(());
        }
        if let Some(&r) = self.drops.locals.get(&key) {
            return self.moved(&MovePath { root: r, path: Vec::new() });
        }
        if let Some(mp) = self.capture_path(key) {
            return self.moved(&mp);
        }
        Ok(())
    }

    /// The part of the closure's captures that local `key` is, if it is
    /// captured by value and the closure owns its captures. Otherwise
    /// (not owning them) a move out of one is noted, for the closure to be
    /// lowered again owning them.
    fn capture_path(&mut self, key: (u32, u32, NodeId)) -> Option<MovePath> {
        let (i, by_value) = self.upvar(key)?;
        if !by_value {
            return None;
        }
        match self.env_root {
            Some(r) => Some(MovePath { root: r, path: vec![(0, i)] }),
            None => {
                self.moved_capture = true;
                None
            }
        }
    }

    /// Commits the bindings a pattern made: each moves out of the part it
    /// binds, and becomes a root of scope `scope`.
    pub fn commit_bindings(&mut self, scope: usize) -> R<()> {
        let pending = std::mem::take(&mut self.drops.pending);
        for (mp, key) in pending {
            if let Some(mp) = mp {
                self.moved(&mp)?;
            }
            let slot = *self.locals.get(&key).ok_or("binding without a slot")?;
            let r = self.owned(slot, scope)?;
            self.drops.locals.insert(key, r);
        }
        Ok(())
    }

    /// Before `lhs` (at `p`) is assigned: drops the value it holds.
    pub fn drop_before_assign(&mut self, lhs: NodeId, p: Place) -> R<Option<MovePath>> {
        if !self.needs_drop(p.ty) {
            return Ok(None);
        }
        match self.move_path(lhs) {
            Ok(Some(mp)) => {
                self.drop_part(&mp)?;
                Ok(Some(mp))
            }
            // Through a reference, or what a closure captured: it holds a
            // value.
            _ => {
                self.drop_place(p)?;
                Ok(None)
            }
        }
    }

    /// The temporaries a `let` initialiser `e` makes that live as long as
    /// the `let`'s block (`let x = &f();`): each goes in scope `scope`.
    pub fn extend_temporaries(&mut self, e: NodeId, scope: usize) {
        let ast = self.body.ast;
        let kids = self.kids(e);
        match ast.kind(e) {
            Kind::Borrow => self.extend_place(*kids.last().unwrap(), scope),
            Kind::Paren | Kind::Cast => self.extend_temporaries(kids[0], scope),
            Kind::Tuple | Kind::Array => {
                for k in kids {
                    self.extend_temporaries(k, scope);
                }
            }
            Kind::StructExpr => {
                for &f in &kids[1..] {
                    if ast.kind(f) == Kind::FieldInit
                        && let Some(&x) = self.kids(f).get(1)
                    {
                        self.extend_temporaries(x, scope);
                    }
                }
            }
            Kind::Block => {
                if let Some(&t) = kids.last()
                    && super::pat::is_expr(ast.kind(t))
                {
                    self.extend_temporaries(t, scope);
                }
            }
            _ => {}
        }
    }

    fn extend_place(&mut self, x: NodeId, scope: usize) {
        let ast = self.body.ast;
        self.drops.extended.insert(x, scope);
        let kids = self.kids(x);
        match ast.kind(x) {
            Kind::Field | Kind::Index | Kind::Paren => self.extend_place(kids[0], scope),
            Kind::Borrow => self.extend_temporaries(x, scope),
            _ => {}
        }
    }

    // ---- drop glue ----

    /// The body of the drop glue of the value at `p`: its `Drop::drop`,
    /// then each part it holds, in order.
    fn glue_body(&mut self, p: Place) -> R<()> {
        let tcx = self.cx.tcx;
        let t = p.ty;
        let is_box = matches!(tcx.kind(t), TyKind::Adt(d, _) if tcx.lang.get("owned_box") == Some(&d));
        if is_box {
            // What the box holds, before the box's own `Drop` frees it.
            let o = self.load(p)?;
            let inner = self.deref(o)?;
            self.drop_place(inner)?;
        }
        if let TyKind::Adt(d, _) = tcx.kind(t)
            && self.cx.drops.has_impl(d)
        {
            let tr = *tcx.lang.get("drop").ok_or("no `Drop`")?;
            let m = self.trait_method(tr, "drop").ok_or("no `Drop::drop`")?;
            let rt = tcx.mk(TyKind::Ref(Mutbl::Mut, t));
            let recv = self.addr_of(p, rt)?;
            self.call_def(m, vec![t], vec![recv], None)?;
        }
        match tcx.kind(t) {
            TyKind::Tuple(l) => self.drop_fields(p, 0, tcx.tys(l).len()),
            TyKind::Closure(c, _, l) => {
                let n = self.cx.lcx.upvar_tys(c, l).map_or(0, |v| v.len());
                self.drop_fields(p, 0, n)
            }
            TyKind::Adt(d, _) if is_box => {
                let n = tcx.adts.get(&d).map_or(0, |a| a.variants[0].fields.len());
                self.drop_fields(p, 0, n)
            }
            TyKind::Adt(d, _) => {
                let adt = tcx.adts.get(&d).ok_or("not an ADT")?;
                let counts: Vec<usize> = adt.variants.iter().map(|v| v.fields.len()).collect();
                match adt.kind {
                    AdtKind::Union => Ok(()),
                    AdtKind::Struct => self.drop_fields(p, 0, counts[0]),
                    AdtKind::Enum => {
                        for (v, &n) in counts.iter().enumerate() {
                            let any = (0..n).map(|i| self.field_ty(t, v as u32, i)).collect::<R<Vec<Ty>>>()?;
                            if !any.into_iter().any(|ft| self.needs_drop(ft)) {
                                continue;
                            }
                            let is = self.test_variant(p, v as u32)?;
                            let then = self.new_block("variant");
                            let next = self.new_block("variant.next");
                            self.b.cond_br(is, then, next);
                            self.b.at(then);
                            self.drop_fields(p, v as u32, n)?;
                            self.goto(next);
                            self.b.at(next);
                        }
                        Ok(())
                    }
                }
            }
            TyKind::Array(e, c) => {
                let n = self.cx.lcx.eval_const(c, &Default::default()).ok_or("array length not known")? as u64;
                let len = self.usize_const(n);
                self.drop_elements(p.addr, e, len)
            }
            TyKind::Slice(e) => {
                let len = p.meta.ok_or("slice without a length")?;
                self.drop_elements(p.addr, e, len)
            }
            TyKind::Dyn(_) => {
                // The vtable's first entry, unless null.
                let vt = p.meta.ok_or("`dyn` without a vtable")?;
                let ptr = self.ptr_t();
                let f = self.load_imm(ptr, vt, self.cx.lcx.target.ptr_size);
                let null = self.b.module().const_zero(ptr);
                let some = self.b.icmp(ICmpPred::Ne, f, null);
                let then = self.new_block("dyn.drop");
                let next = self.new_block("dyn.drop.next");
                self.b.cond_br(some, then, next);
                self.b.at(then);
                let void = self.b.module().types.void();
                let fty = self.b.module().types.function(void, &[ptr], false);
                self.b.call_indirect(fty, f, &[p.addr]);
                self.b.br(next);
                self.b.at(next);
                Ok(())
            }
            _ => Ok(()),
        }
    }

    /// Drops fields `0..n` of variant `v` of the value at `p`, in order.
    fn drop_fields(&mut self, p: Place, v: u32, n: usize) -> R<()> {
        for i in 0..n {
            let ft = self.field_ty(p.ty, v, i)?;
            if self.needs_drop(ft) {
                let fp = self.field(p, v, i)?;
                self.drop_place(fp)?;
            }
        }
        Ok(())
    }

    /// Drops `len` elements of type `e` from `addr`, in order.
    fn drop_elements(&mut self, addr: Value, e: Ty, len: Value) -> R<()> {
        if !self.needs_drop(e) {
            return Ok(());
        }
        let stride = self.layout(e)?.size;
        let usize_t = self.cx.tcx.mk(TyKind::Uint(crate::ty::UintTy::Usize));
        let i = self.temp(usize_t)?;
        let isz = self.isize_t();
        let ps = self.cx.lcx.target.ptr_size;
        let zero = self.usize_const(0);
        self.store_imm(zero, i.addr, ps);
        let head = self.new_block("drop.elems");
        let body = self.new_block("drop.elem");
        let done = self.new_block("drop.elems.end");
        self.b.br(head);
        self.b.at(head);
        let iv = self.load_imm(isz, i.addr, ps);
        let more = self.b.icmp(ICmpPred::Ult, iv, len);
        self.b.cond_br(more, body, done);
        self.b.at(body);
        let s = self.usize_const(stride);
        let off = self.b.mul(iv, s);
        let i8t = self.i8t();
        let ea = self.b.gep(i8t, addr, &[off]);
        self.drop_place(Place { addr: ea, meta: None, ty: e })?;
        let one = self.usize_const(1);
        let next = self.b.add(iv, one);
        self.store_imm(next, i.addr, ps);
        self.b.br(head);
        self.b.at(done);
        Ok(())
    }
}

/// The symbol of the drop glue of `t`.
pub(crate) fn glue_symbol(cx: &Cx, t: Ty) -> String {
    let mut key = String::new();
    cx.type_key(t, &mut key);
    let text = cx.tcx.display(cx.r, t);
    let safe: String = text.chars().map(|c| if c.is_ascii_alphanumeric() || c == '_' { c } else { '.' }).collect();
    let mut safe = safe.replace("..", ".");
    safe.truncate(100);
    format!("_R.drop_in_place.{safe}.{:016x}", crate::hash::fnv(key.as_bytes()))
}

/// Defines the drop glue of `t` in the function `decl` declares: it takes
/// a `*mut t`.
pub(crate) fn lower_drop_glue(cx: &Cx, m: &mut Module, t: Ty, decl: &Decl) -> R<()> {
    let (ret, params) = Cx::ir_fn_type(m, &decl.abi);
    let mut spec = IrFunction::builder(&decl.name).returns(ret);
    for p in &params {
        spec = spec.param("", *p);
    }
    spec = spec.linkage(Linkage::Internal);
    let b: FunctionBuilder = spec.begin(m);
    let mut fx = Fx::for_glue(cx, b, &decl.name);
    let first = fx.b.current_block().expect("the body block");
    let pt = decl.abi.inputs[0];
    let slot = fx.temp(pt)?;
    let v = fx.b.param(0);
    let o = match decl.abi.args[0] {
        Repr::Imm(_) => Operand { ty: pt, v: Val::Imm(v) },
        Repr::Mem => Operand { ty: pt, v: Val::Mem(v) },
        Repr::Zst => return Err("drop glue of a pointer of no size".into()),
    };
    fx.store(o, slot)?;
    let o = fx.load(slot)?;
    let p = fx.deref(o)?;
    let _ = t;
    fx.glue_body(p)?;
    if !fx.terminated() {
        fx.b.ret_void();
    }
    fx.finish(first)
}
