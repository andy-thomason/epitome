//! Building the control-flow graph of a body from the typed tree.
//!
//! It follows what the lowering to IR does (`lower/body.rs`), with the
//! difference that matters here: a program is kept as places and borrows,
//! not as values. A place expression (a local, a field, `*p`, `a[i]`) is a
//! `Place`; anything else is evaluated into a temporary. Adjustments typeck
//! recorded (autoderef, autoref, unsizing) are applied to the place.
//!
//! Scopes: a block's variables and a statement's temporaries each leave
//! scope with a `StorageDead` (and a `Drop`, where the type is not plainly
//! trivial), in reverse order, on every way out (`break`, `continue`,
//! `return`, `?` too). A `let` that borrows a temporary (`let x = &f();`)
//! keeps it for the block.
//!
//! A body it cannot build says why (`Err`), and `Session::cfgs` counts them.

use super::cfg::*;
use crate::ast::{Ast, Kind, NodeId};
use crate::hash::FastMap;
use crate::resolve::{DefId, DefKind, Res, Resolver};
use crate::session::ParsedCrate;
use crate::ty::collect::subst;
use crate::ty::{AdtKind, Mutbl, Ty, TyCtxt, TyKind};
use crate::typeck::{Adjust, BindingMode, Index, Solver, Tables, TypeckResults};

type R<T> = Result<T, String>;

/// What building needs from outside the body.
pub struct Cx<'a, 's> {
    pub tcx: &'a TyCtxt<'s>,
    pub r: &'a Resolver<'s>,
    pub crates: &'a [ParsedCrate<'s>],
    pub res: &'a TypeckResults,
    pub index: &'a Index<'s>,
}

struct Scope {
    /// Locals declared in it, in order.
    locals: Vec<Local>,
    /// Whether variables (not only temporaries) are declared in it.
    vars: bool,
}

struct LoopCx<'s> {
    label: Option<&'s str>,
    exit: Block,
    cont: Option<Block>,
    dest: Option<Place>,
    /// How many scopes were open outside it.
    depth: usize,
    is_block: bool,
}

/// A binding a pattern makes once it has matched.
struct Bind {
    pat: NodeId,
    place: Place,
    mode: BindingMode,
}

pub struct Builder<'a, 's> {
    cx: &'a Cx<'a, 's>,
    solver: Solver<'a, 's>,
    ast: &'a Ast<'s>,
    tables: &'a Tables,
    types: &'a Vec<Option<Ty>>,
    k: u32,
    file: u32,
    cfg: Cfg,
    cur: Option<Block>,
    scopes: Vec<Scope>,
    loops: Vec<LoopCx<'s>>,
    /// The `try` blocks being built: where `?` goes, with the scopes outside.
    tries: Vec<(Block, Place, usize)>,
    /// The local each binding pattern declares.
    locals: FastMap<NodeId, Local>,
    /// Another alternative's binding is the first's.
    alias: FastMap<NodeId, NodeId>,
    /// Captured locals in a closure: field, whether by value, type.
    upvars: FastMap<NodeId, (usize, bool, Ty)>,
    env: Option<Local>,
    binds: Vec<Bind>,
    /// Scope to put temporaries in instead of the innermost (a `let`'s
    /// extended ones).
    extend: Option<usize>,
    /// Whether the expression being built is where a `let`'s temporaries
    /// are extended to the block (`let x = &f();`, not `let x = g(&f());`).
    extending: bool,
    /// Before edition 2024 the temporaries of a block's tail expression live to
    /// the end of the statement the block is in: that scope.
    tail_outer: Option<(usize, usize)>,
    /// The block about to be built is coerced as a whole (its tail's
    /// temporaries live until after the coercion).
    block_adjusted: bool,
    /// When each local first got a value (a count of what was emitted): what
    /// is dropped is dropped in reverse of this, as rustc's temporaries are.
    first_def: FastMap<u32, u32>,
    seq: u32,
    /// How deep in expressions: a guard against runaway recursion.
    depth: u32,
    /// The place being built is to be changed (assigned to, borrowed with
    /// `&mut`): `DerefMut` and `IndexMut`, not `Deref` and `Index`.
    mut_use: bool,
    /// The next statement is a two-phase borrow (an autoref receiver).
    next_two_phase: bool,
    /// The next statement borrows a variable a closure captures by value.
    next_moved: bool,
    /// The closure expressions met, to build bodies for.
    pub closures: Vec<NodeId>,
}

/// Whether a type has a projection in it.
fn has_proj(tcx: &TyCtxt, t: Ty) -> bool {
    match tcx.kind(t) {
        TyKind::Projection(..) => true,
        TyKind::Tuple(l) | TyKind::Adt(_, l) | TyKind::Opaque(_, l) => tcx.tys(l).into_iter().any(|x| has_proj(tcx, x)),
        TyKind::Ref(_, x) | TyKind::Ptr(_, x) | TyKind::Slice(x) | TyKind::Array(x, _) => has_proj(tcx, x),
        _ => false,
    }
}

fn is_pattern(k: Kind) -> bool {
    matches!(
        k,
        Kind::WildPat
            | Kind::RestPat
            | Kind::IdentPat
            | Kind::LitPat
            | Kind::RangePat
            | Kind::RefPat
            | Kind::TuplePat
            | Kind::ParenPat
            | Kind::GuardPat
            | Kind::SlicePat
            | Kind::PathPat
            | Kind::TupleStructPat
            | Kind::StructPat
            | Kind::OrPat
            | Kind::BoxPat
            | Kind::NeverPat
            | Kind::BuiltinPat
            | Kind::PatMacro
    )
}

/// An item written among a block's statements: nothing to run.
fn is_item(k: Kind) -> bool {
    matches!(
        k,
        Kind::Fn
            | Kind::Struct
            | Kind::Enum
            | Kind::Union
            | Kind::Trait
            | Kind::TraitAlias
            | Kind::Impl
            | Kind::Mod
            | Kind::Use
            | Kind::Const
            | Kind::Static
            | Kind::TypeAlias
            | Kind::ExternCrate
            | Kind::ExternBlock
            | Kind::MacroRules
            | Kind::MacroDef
            | Kind::Delegation
    )
}

fn trivial(tcx: &TyCtxt, t: Ty) -> bool {
    match tcx.kind(t) {
        TyKind::Bool
        | TyKind::Char
        | TyKind::Int(_)
        | TyKind::Uint(_)
        | TyKind::Float(_)
        | TyKind::Ref(..)
        | TyKind::Ptr(..)
        | TyKind::FnPtr(_)
        | TyKind::FnDef(..)
        | TyKind::Never => true,
        TyKind::Tuple(l) => tcx.tys(l).into_iter().all(|x| trivial(tcx, x)),
        _ => false,
    }
}

impl<'a, 's> Builder<'a, 's> {
    fn new(cx: &'a Cx<'a, 's>, owner: DefId, node: NodeId, name: String, closure: Option<crate::ty::ClosureId>) -> Self {
        let def = &cx.r.defs[owner.0 as usize];
        let (k, file) = (def.krate, def.file);
        let f = &cx.crates[k as usize].files[file as usize];
        Builder {
            cx,
            solver: Solver::for_owner(cx.tcx, cx.r, cx.crates, cx.index, owner),
            ast: &f.ast,
            tables: &cx.res.tables[k as usize][file as usize],
            types: &cx.res.types[k as usize][file as usize],
            k,
            file,
            cfg: Cfg { owner, krate: k, file, node, closure, name, locals: Vec::new(), arg_count: 0, blocks: Vec::new(), upvars: Vec::new(), static_vars: Vec::new() },
            cur: None,
            scopes: Vec::new(),
            loops: Vec::new(),
            tries: Vec::new(),
            locals: FastMap::default(),
            alias: FastMap::default(),
            upvars: FastMap::default(),
            env: None,
            binds: Vec::new(),
            extend: None,
            extending: false,
            tail_outer: None,
            block_adjusted: false,
            first_def: FastMap::default(),
            seq: 0,
            depth: 0,
            mut_use: false,
            next_two_phase: false,
            next_moved: false,
            closures: Vec::new(),
        }
    }

    // ---- small things ----

    fn tcx(&self) -> &'a TyCtxt<'s> {
        self.cx.tcx
    }

    fn ty(&self, n: NodeId) -> R<Ty> {
        let t = self.types.get(n.0 as usize).copied().flatten().ok_or_else(|| format!("no type for {:?}", self.ast.kind(n)))?;
        if matches!(self.tcx().kind(t), TyKind::Error) {
            return Err(format!("error type for {:?}", self.ast.kind(n)));
        }
        Ok(t)
    }

    /// The children of `n`, without attributes (which statements, arms,
    /// fields and patterns may start with).
    fn kids(&self, n: NodeId) -> Vec<NodeId> {
        self.ast.children(n).iter().copied().filter(|&c| !matches!(self.ast.kind(c), Kind::Attribute | Kind::InnerAttribute)).collect()
    }

    fn res(&self, n: NodeId) -> Option<&'a Res> {
        let r: &'a Resolver<'s> = self.cx.r;
        r.res_of(self.k as usize, self.file as usize, n)
    }

    /// `t` with its projections worked out, where they can be.
    fn norm(&mut self, t: Ty) -> Ty {
        // (Not one with an error or an unknown in it: the solver does not end on those.)
        if has_proj(self.tcx(), t) && !self.tcx().has_error(t) && !self.tcx().has_infer(t) { self.solver.normalize(t) } else { t }
    }

    fn unit(&self) -> Ty {
        self.tcx().unit()
    }

    fn never(&self) -> Ty {
        self.tcx().mk(TyKind::Never)
    }

    fn bool_ty(&self) -> Ty {
        self.tcx().mk(TyKind::Bool)
    }

    fn usize_ty(&self) -> Ty {
        self.tcx().mk(TyKind::Uint(crate::ty::UintTy::Usize))
    }

    fn text(&self, n: NodeId) -> String {
        self.ast.text(n)
    }

    fn is_prim(&self, t: Ty) -> bool {
        matches!(self.tcx().kind(t), TyKind::Bool | TyKind::Char | TyKind::Int(_) | TyKind::Uint(_) | TyKind::Float(_) | TyKind::Ptr(..))
    }

    fn pointee(&self, t: Ty) -> Option<Ty> {
        let tcx = self.tcx();
        match tcx.kind(t) {
            TyKind::Ref(_, x) | TyKind::Ptr(_, x) => Some(x),
            TyKind::Adt(d, l) if tcx.lang.get("owned_box") == Some(&d) => tcx.tys(l).first().copied(),
            _ => None,
        }
    }

    /// Whether `t` has no values: `!`, an enum with no variants (or whose
    /// variants each have such a field), a struct or tuple with such a field.
    /// (Whether it can be seen to be is not asked: a call past which the code
    /// is taken as not reached only loses errors.)
    fn uninhabited(&self, t: Ty, depth: u32) -> bool {
        let tcx = self.tcx();
        if depth > 4 {
            return false;
        }
        match tcx.kind(t) {
            TyKind::Never => true,
            TyKind::Tuple(l) => tcx.tys(l).into_iter().any(|x| self.uninhabited(x, depth + 1)),
            TyKind::Adt(d, l) => {
                let Some(adt) = tcx.adts.get(&d) else { return false };
                let args = tcx.tys(l);
                let field_void = |v: &crate::ty::VariantDef| {
                    v.fields.iter().any(|f| self.uninhabited(subst(tcx, f.ty, &tcx.all_params(d), &args), depth + 1))
                };
                match adt.kind {
                    AdtKind::Enum => adt.variants.iter().all(field_void),
                    AdtKind::Struct => adt.variants.first().is_some_and(field_void),
                    AdtKind::Union => false,
                }
            }
            _ => false,
        }
    }

    fn is_never(&self, t: Ty) -> bool {
        matches!(self.tcx().kind(t), TyKind::Never)
    }

    // ---- blocks, statements, scopes ----

    fn new_block(&mut self) -> Block {
        self.cfg.blocks.push(BlockData::default());
        Block(self.cfg.blocks.len() as u32 - 1)
    }

    /// The block being built; if control cannot get here (after a `return`),
    /// a fresh one nothing jumps to.
    fn block(&mut self) -> Block {
        match self.cur {
            Some(b) => b,
            None => {
                let b = self.new_block();
                self.cur = Some(b);
                b
            }
        }
    }

    fn at(&mut self, b: Block) {
        self.cur = Some(b);
    }

    fn emit(&mut self, kind: StmtKind, node: NodeId) {
        let b = self.block();
        self.seq += 1;
        if let StmtKind::Assign(d, _) = &kind {
            self.first_def.entry(d.local.0).or_insert(self.seq);
        }
        let two_phase = std::mem::take(&mut self.next_two_phase);
        self.cfg.blocks[b.0 as usize].stmts.push(Stmt { kind, node, two_phase, moved: std::mem::take(&mut self.next_moved) });
    }

    fn assign(&mut self, p: &Place, rv: Rvalue, node: NodeId) {
        self.emit(StmtKind::Assign(p.clone(), rv), node);
    }

    fn end(&mut self, t: Term, node: NodeId) {
        self.seq += 1;
        if let Term::Call { dest, .. } = &t {
            self.first_def.entry(dest.local.0).or_insert(self.seq);
        }
        let b = self.block();
        self.cfg.blocks[b.0 as usize].term = Some((t, node));
        self.cur = None;
    }

    fn goto(&mut self, to: Block, node: NodeId) {
        if self.cur.is_some() {
            self.end(Term::Goto(to), node);
        }
    }

    /// Branches on `cond`: to `then`, or to `els`.
    fn branch(&mut self, cond: Operand, then: Block, els: Block, node: NodeId) {
        self.end(Term::Switch { discr: cond, targets: vec![(0, els)], otherwise: then }, node);
    }

    fn new_local(&mut self, ty: Ty, kind: LocalKind, name: Option<String>, mutable: bool, node: Option<NodeId>) -> Local {
        self.cfg.locals.push(LocalDecl { ty, kind, name, mutable, node });
        Local(self.cfg.locals.len() as u32 - 1)
    }

    fn push_scope(&mut self, vars: bool) {
        self.scopes.push(Scope { locals: Vec::new(), vars });
    }

    /// Where a new local goes: variables in the innermost scope for them,
    /// temporaries in the innermost (or the `let`'s extended one).
    fn register(&mut self, l: Local, var: bool) {
        let i = if var {
            self.scopes.iter().rposition(|s| s.vars).unwrap_or(0)
        } else {
            self.extend.filter(|_| self.extending).or(self.tail_outer.filter(|&(_, depth)| depth == self.scopes.len()).map(|x| x.0)).unwrap_or(self.scopes.len().saturating_sub(1))
        };
        if let Some(s) = self.scopes.get_mut(i) {
            s.locals.push(l);
        }
    }

    fn temp(&mut self, ty: Ty, node: NodeId) -> Place {
        let l = self.new_local(ty, LocalKind::Temp, None, true, None);
        self.register(l, false);
        self.emit(StmtKind::StorageLive(l), node);
        Place::local(l, ty)
    }

    /// What leaving scope `i` does, innermost first.
    fn scope_exit(&mut self, i: usize, node: NodeId) {
        let mut locals = self.scopes[i].locals.clone();
        // (Reverse of when they got their values, as made; the rest as declared.)
        let key = |l: &Local, at: usize| self.first_def.get(&l.0).map_or((0, at as u32), |&s| (1, s));
        let mut keyed: Vec<(Local, (u32, u32))> = locals.iter().enumerate().map(|(at, l)| (*l, key(l, at))).collect();
        keyed.sort_by_key(|k| std::cmp::Reverse(k.1));
        locals = keyed.into_iter().map(|k| k.0).collect();
        for &l in locals.iter() {
            let ty = self.cfg.locals[l.0 as usize].ty;
            if !trivial(self.tcx(), ty) {
                self.emit(StmtKind::Drop(Place::local(l, ty)), node);
            }
            self.emit(StmtKind::StorageDead(l), node);
        }
    }

    /// Leaving the scopes from `depth` in, all of them, without closing them
    /// (a `break` goes on, in code that is not reached).
    fn exit_to(&mut self, depth: usize, node: NodeId) {
        for i in (depth..self.scopes.len()).rev() {
            self.scope_exit(i, node);
        }
    }

    fn pop_scope(&mut self, node: NodeId) {
        let i = self.scopes.len() - 1;
        if self.cur.is_some() {
            self.scope_exit(i, node);
        }
        self.scopes.pop();
    }

    fn finish(mut self, node: NodeId) -> Cfg {
        for b in &mut self.cfg.blocks {
            if b.term.is_none() {
                b.term = Some((Term::Unreachable, node));
            }
        }
        self.cfg
    }

    // ---- places ----

    /// The binding an or-pattern's other alternatives' is the same as: the
    /// first's, through the nesting.
    fn canon(&self, mut n: NodeId) -> NodeId {
        for _ in 0..16 {
            match self.alias.get(&n) {
                Some(&m) if m != n => n = m,
                _ => break,
            }
        }
        n
    }

    fn place_of_local(&mut self, key: NodeId) -> R<Place> {
        let key = self.canon(key);
        if let Some(&l) = self.locals.get(&key) {
            return Ok(Place::local(l, self.cfg.locals[l.0 as usize].ty));
        }
        if let Some(&(i, by_value, ty)) = self.upvars.get(&key) {
            let env = self.env.ok_or("a captured local outside a closure")?;
            let et = self.cfg.locals[env.0 as usize].ty;
            let fp = Place::local(env, et);
            return Ok(if by_value {
                fp.project(Proj::Field(i as u32, ty), ty)
            } else {
                let rt = self.tcx().mk(TyKind::Ref(Mutbl::Not, ty));
                fp.project(Proj::Field(i as u32, rt), rt).deref_to(ty)
            });
        }
        Err("a local with no place".into())
    }

    /// The type of field `i` of variant `v` of ADT (or tuple) type `t`.
    fn field_ty(&mut self, t: Ty, v: u32, i: usize) -> R<Ty> {
        let tcx = self.tcx();
        let t = self.norm(t);
        match tcx.kind(t) {
            TyKind::Adt(d, l) => {
                let adt = tcx.adts.get(&d).ok_or("not an ADT")?;
                let f = adt.variants.get(v as usize).and_then(|x| x.fields.get(i)).ok_or("no such field")?;
                let ft = subst(tcx, f.ty, &tcx.all_params(d), &tcx.tys(l));
                Ok(self.norm(ft))
            }
            TyKind::Tuple(l) => tcx.tys(l).get(i).copied().ok_or_else(|| "no such tuple field".into()),
            TyKind::Array(e, _) | TyKind::Slice(e) => Ok(e),
            _ => Err(format!("field of {}", tcx.display(self.cx.r, t))),
        }
    }

    /// Field `i` of variant `v` of the place `p` (a downcast first, for an
    /// enum).
    fn field(&mut self, p: &Place, v: u32, i: usize) -> R<Place> {
        let tcx = self.tcx();
        let ft = self.field_ty(p.ty, v, i)?;
        let pty = self.norm(p.ty);
        let enum_ = matches!(tcx.kind(pty), TyKind::Adt(d, _) if tcx.adts.get(&d).is_some_and(|a| a.kind == AdtKind::Enum));
        let base = if enum_ { p.project(Proj::Downcast(v), p.ty) } else { p.clone() };
        Ok(base.project(Proj::Field(i as u32, ft), ft))
    }

    fn deref(&mut self, p: &Place) -> R<Place> {
        let pty = self.norm(p.ty);
        let t = self.pointee(pty).ok_or_else(|| format!("deref of {}", self.tcx().display(self.cx.r, p.ty)))?;
        let t = self.norm(t);
        Ok(p.project(Proj::Deref, t))
    }

    fn variant_named(&self, d: DefId, name: &str) -> Option<u32> {
        let adt = self.tcx().adts.get(&d)?;
        adt.variants.iter().position(|v| v.def.is_some_and(|x| self.cx.r.defs[x.0 as usize].name == name)).map(|i| i as u32)
    }

    fn variant_of_path(&self, path: NodeId, d: DefId) -> Option<u32> {
        let adt = self.tcx().adts.get(&d)?;
        if adt.kind != AdtKind::Enum {
            return Some(0);
        }
        match self.res(path) {
            Some(Res::Def(v)) if self.cx.r.defs[v.0 as usize].kind == DefKind::Variant => {
                adt.variants.iter().position(|x| x.def == Some(*v)).map(|i| i as u32)
            }
            _ => {
                let name = if self.ast.kind(path) == Kind::Path {
                    self.ast.children(path).iter().rev().find(|&&s| self.ast.kind(s) == Kind::Segment).map(|&s| self.ast.token(s).0)
                } else {
                    Some(self.ast.token(path).0)
                }?;
                self.variant_named(d, name)
            }
        }
    }

    /// The trait method `name` of the lang-item trait `lang`.
    fn lang_method(&self, lang: &str, name: &str) -> Option<DefId> {
        let tr = *self.tcx().lang.get(lang)?;
        let r = self.cx.r;
        let m = r.defs[tr.0 as usize].module?;
        match r.modules[m.0 as usize].names.get(&(name, crate::resolve::Ns::Value)).map(|b| &b.res) {
            Some(Res::Def(d)) => Some(*d),
            _ => None,
        }
    }

    /// The type of parameter `i` of function `d` at generic arguments `args`.
    fn arg_ty(&mut self, d: DefId, args: crate::ty::List, i: usize) -> R<Ty> {
        let tcx = self.tcx();
        let sig = tcx.sig_of((*tcx.fn_sig.get(d.0 as usize).ok_or("no signature")?).ok_or("no signature")?);
        let t = *tcx.tys(sig.inputs).get(i).ok_or("no such parameter")?;
        let t = subst(tcx, t, &tcx.all_params(d), &tcx.tys(args));
        Ok(self.norm(t))
    }

    /// The result type of function `d` at generic arguments `args`.
    fn ret_ty(&mut self, d: DefId, args: crate::ty::List) -> R<Ty> {
        let tcx = self.tcx();
        let sig = tcx.sig_of((*tcx.fn_sig.get(d.0 as usize).ok_or("no signature")?).ok_or("no signature")?);
        let t = subst(tcx, sig.output, &tcx.all_params(d), &tcx.tys(args));
        Ok(self.norm(t))
    }

    // ---- calls ----

    fn call_into(&mut self, func: Callee, args: Vec<Operand>, dest: &Place, node: NodeId) {
        // It does not return if it is to a `!` (not only if where it goes
        // is: a diverging call's value is made `()` or anything).
        let returns_never = match &func {
            Callee::Fn(d, l) => self.ret_ty(*d, *l).is_ok_and(|t| self.is_never(t)),
            Callee::Value(o) => match self.tcx().kind(self.operand_ty(o)) {
                TyKind::FnPtr(sig) => self.is_never(self.tcx().sig_of(sig).output),
                _ => false,
            },
        };
        let diverges = self.is_never(dest.ty) || returns_never || self.uninhabited(dest.ty, 0);
        let target = if diverges { None } else { Some(self.new_block()) };
        self.end(Term::Call { func, args, dest: dest.clone(), target }, node);
        if let Some(t) = target {
            self.at(t);
        }
    }

    /// A call whose result type is `ty`, into a temporary.
    fn call_temp(&mut self, func: Callee, args: Vec<Operand>, ty: Ty, node: NodeId) -> Place {
        let t = self.temp(ty, node);
        self.call_into(func, args, &t, node);
        t
    }

    // ---- expressions ----

    /// The value of `n` into `dest`.
    fn expr_into(&mut self, n: NodeId, dest: &Place) -> R<()> {
        if self.tables.adjustments.get(&n).is_some_and(|a| !a.is_empty()) {
            self.block_adjusted = self.ast.kind(n) == Kind::Block;
            let p = self.expr_place(n);
            self.block_adjusted = false;
            let p = p?;
            // (A coerced block's value is its tail's: errors point there.)
            let at = if self.ast.kind(n) == Kind::Block { self.kids(n).last().copied().unwrap_or(n) } else { n };
            self.assign(dest, Rvalue::Use(Operand::Use(p)), at);
            return Ok(());
        }
        self.expr_into_raw(n, dest)
    }

    /// `n` as a place: itself if it is one, else a temporary holding it;
    /// adjusted as typeck says.
    fn expr_place(&mut self, n: NodeId) -> R<Place> {
        // Borrowed mutably, by an adjustment, or in a context that changes it.
        let adj = self.tables.adjustments.get(&n).cloned().unwrap_or_default();
        let needs_mut = self.mut_use || adj.iter().any(|a| matches!(a, Adjust::Borrow(Mutbl::Mut)));
        let saved = std::mem::replace(&mut self.mut_use, needs_mut);
        let r = self.unadjusted_place(n).and_then(|p| if adj.is_empty() { Ok(p) } else { self.adjust(n, p, &adj) });
        self.mut_use = saved;
        let mut p = r?;
        // Named by `n`, for an error to point at.
        p.at = Some(n);
        Ok(p)
    }

    /// `n` as a place that is to be changed (or not).
    fn expr_place_mut(&mut self, n: NodeId, m: bool) -> R<Place> {
        let saved = std::mem::replace(&mut self.mut_use, m);
        let r = self.expr_place(n);
        self.mut_use = saved;
        r
    }

    /// `n` as an operand: its value, not a place that is changed.
    fn expr_value(&mut self, n: NodeId) -> R<Operand> {
        let saved = std::mem::replace(&mut self.mut_use, false);
        let r = self.expr_operand(n);
        self.mut_use = saved;
        r
    }

    fn expr_operand(&mut self, n: NodeId) -> R<Operand> {
        if !self.tables.adjustments.get(&n).is_some_and(|a| !a.is_empty())
            && let Some(c) = self.const_operand(n)?
        {
            return Ok(c);
        }
        Ok(Operand::Use(self.expr_place(n)?))
    }

    /// The operands of `nodes`, in order: each one that is not the last is
    /// copied (or moved) into a temporary where it is evaluated, as rustc
    /// does, so that what is evaluated after it cannot change what it is.
    fn operands(&mut self, nodes: &[NodeId]) -> R<Vec<Operand>> {
        let mut out = Vec::new();
        for (i, &e) in nodes.iter().enumerate() {
            let o = self.expr_operand(e)?;
            // (Unless what is evaluated after it cannot change it.)
            out.push(if nodes[i + 1..].iter().any(|&n| !self.is_pure(n)) { self.freeze(o, e) } else { o });
        }
        Ok(out)
    }

    /// Whether evaluating `n` cannot change anything: a literal or a path.
    fn is_pure(&self, n: NodeId) -> bool {
        match self.ast.kind(n) {
            Kind::Literal | Kind::PathExpr => true,
            Kind::Paren => self.ast.children(n).first().is_some_and(|&c| self.is_pure(c)),
            _ => false,
        }
    }

    /// A place operand copied into a temporary.
    fn freeze(&mut self, o: Operand, node: NodeId) -> Operand {
        let Operand::Use(p) = &o else { return o };
        if p.proj.is_empty() && self.cfg.locals[p.local.0 as usize].kind == LocalKind::Temp {
            return o;
        }
        // (What is moved, or only used by reference, stays where it is.)
        if self.tcx().has_error(p.ty) || self.tcx().has_infer(p.ty) || !self.solver.may_be_copy(p.ty) {
            return o;
        }
        let t = self.temp(p.ty, node);
        self.assign(&t, Rvalue::Use(o), node);
        Operand::Use(t)
    }

    /// `n` if it is a constant that needs no place.
    fn const_operand(&mut self, n: NodeId) -> R<Option<Operand>> {
        let kids = self.kids(n);
        Ok(match self.ast.kind(n) {
            Kind::Literal => Some(Operand::Const(self.ast.token(n).0.to_string(), self.ty(n)?)),
            Kind::Paren => self.const_operand(kids[0])?,
            Kind::PathExpr => {
                let t = self.ty(n)?;
                match self.res(kids[0]) {
                    Some(Res::Local(_)) => None,
                    _ if self.tables.consts.contains_key(&n) => Some(Operand::Const(self.text(n), t)),
                    Some(Res::Def(d)) if matches!(self.cx.r.defs[d.0 as usize].kind, DefKind::Fn | DefKind::AssocFn | DefKind::Const | DefKind::Static | DefKind::AssocConst) => {
                        Some(Operand::Const(self.text(n), t))
                    }
                    Some(Res::Generic(_)) => Some(Operand::Const(self.text(n), t)),
                    _ if matches!(self.tcx().kind(t), TyKind::FnDef(..)) => Some(Operand::Const(self.text(n), t)),
                    _ => None,
                }
            }
            _ => None,
        })
    }

    fn adjust(&mut self, n: NodeId, mut p: Place, adj: &[Adjust]) -> R<Place> {
        let tcx = self.tcx();
        let adjusted = self.tables.adjusted.get(&n).copied();
        for (i, a) in adj.iter().enumerate() {
            let last = i + 1 == adj.len();
            match a {
                Adjust::Deref => p = self.deref(&p)?,
                Adjust::OverloadedDeref => {
                    // `Deref::deref(&p)` (or `deref_mut`, if what follows
                    // borrows mutably), then through what it gives.
                    let mutable = self.mut_use && self.solver.may_implement(p.ty, "deref_mut", &[]);
                    let (lang, name) = if mutable { ("deref_mut", "deref_mut") } else { ("deref", "deref") };
                    let m = self.lang_method(lang, name).ok_or("no `Deref` method")?;
                    let m_args = tcx.list(vec![p.ty]);
                    let m_ret = self.ret_ty(m, m_args)?;
                    let rm = if mutable { Mutbl::Mut } else { Mutbl::Not };
                    let recv_t = tcx.mk(TyKind::Ref(rm, p.ty));
                    let recv = self.temp(recv_t, n);
                    self.next_two_phase = true;
                    self.assign(&recv, Rvalue::Ref(rm, p.clone()), n);
                    let out = self.call_temp(Callee::Fn(m, m_args), vec![Operand::Use(recv)], m_ret, n);
                    p = self.deref(&out)?;
                }
                Adjust::Borrow(m) => {
                    let rt = tcx.mk(TyKind::Ref(*m, p.ty));
                    let t = self.temp(rt, n);
                    self.next_two_phase = true;
                    self.assign(&t, Rvalue::Ref(*m, p.clone()), n);
                    p = t;
                }
                Adjust::Unsize | Adjust::NeverToAny | Adjust::ReifyFnPointer | Adjust::ClosureFnPointer => {
                    let target = if last { adjusted.unwrap_or(p.ty) } else { p.ty };
                    let t = self.temp(target, n);
                    self.assign(&t, Rvalue::Cast(Operand::Use(p), target), n);
                    p = t;
                }
            }
        }
        Ok(p)
    }

    fn unadjusted_place(&mut self, n: NodeId) -> R<Place> {
        let kids = self.kids(n);
        match self.ast.kind(n) {
            Kind::PathExpr => {
                if !self.tables.consts.contains_key(&n)
                    && let Some(Res::Local(p)) = self.res(kids[0])
                {
                    return self.place_of_local(*p);
                }
            }
            Kind::Paren | Kind::AttrExpr => return self.expr_place(*kids.last().unwrap()),
            Kind::Field => {
                let base = self.expr_place(kids[0])?;
                let name = self.ast.token(kids[1]).0;
                let i = match self.tcx().kind(base.ty) {
                    TyKind::Adt(d, _) => {
                        let adt = self.tcx().adts.get(&d).ok_or("not an ADT")?;
                        adt.variants[0].fields.iter().position(|f| f.name == name).ok_or("no such field")?
                    }
                    TyKind::Tuple(_) => name.parse::<usize>().map_err(|_| "bad tuple field")?,
                    _ => return Err(format!("field of {}", self.tcx().display(self.cx.r, base.ty))),
                };
                return self.field(&base, 0, i);
            }
            Kind::Index => return self.index_place(n, &kids),
            Kind::Unary if self.ast.kind(kids[0]) != Kind::Keyword && self.ast.token(kids[0]).0 == "*" => {
                if let Some(&(m, margs)) = self.tables.methods.get(&n) {
                    // `*x` through `Deref::deref(&x)` (or `deref_mut`).
                    let base = self.expr_place(kids[1])?;
                    // (Typeck says `deref`; whether it is `deref_mut` is where
                    // the result is used.)
                    let mutable = self.mut_use && self.solver.may_implement(base.ty, "deref_mut", &[]);
                    let m = if mutable { self.lang_method("deref_mut", "deref_mut").unwrap_or(m) } else { m };
                    let rm = if mutable { Mutbl::Mut } else { Mutbl::Not };
                    let recv_t = self.tcx().mk(TyKind::Ref(rm, base.ty));
                    let recv = self.temp(recv_t, n);
                    self.next_two_phase = true;
                    self.next_two_phase = true;
            self.assign(&recv, Rvalue::Ref(rm, base), n);
                    let out_t = self.tcx().mk(TyKind::Ref(rm, self.ty(n)?));
                    let out = self.call_temp(Callee::Fn(m, margs), vec![Operand::Use(recv)], out_t, n);
                    return self.deref(&out);
                }
                let base = self.expr_place(kids[1])?;
                return self.deref(&base);
            }
            _ => {}
        }
        // A static item is a place: of its own, that is `mut` or not.
        if self.ast.kind(n) == Kind::PathExpr
            && !self.tables.consts.contains_key(&n)
            && let Some(Res::Def(d)) = self.res(kids[0])
            && self.cx.r.defs[d.0 as usize].kind == DefKind::Static
        {
            let t = self.ty(n)?;
            let def = &self.cx.r.defs[d.0 as usize];
            let sast = &self.cx.crates[def.krate as usize].files[def.file as usize].ast;
            let mutable = sast.children(def.node).iter().any(|&c| sast.kind(c) == Kind::Keyword && sast.token(c).0 == "mut");
            let l = self.new_local(t, LocalKind::Static, Some(def.name.to_string()), mutable, None);
            self.register(l, false);
            self.emit(StmtKind::StorageLive(l), n);
            let p = Place::local(l, t);
            // (It has its value: this is not an assignment of the program's.)
            self.emit(StmtKind::MaybeInit(p.clone()), n);
            return Ok(p);
        }
        let t = self.ty(n)?;
        let tmp = self.temp(t, n);
        self.expr_into_raw(n, &tmp)?;
        Ok(tmp)
    }

    fn index_place(&mut self, n: NodeId, kids: &[NodeId]) -> R<Place> {
        let tcx = self.tcx();
        let base = self.expr_place(kids[0])?;
        if let Some(&(m, margs)) = self.tables.methods.get(&n) {
            // (Typeck says `index`; whether it is `index_mut` is where the
            // result is used.)
            let idx = self.expr_value(kids[1])?;
            let idx_ty = self.operand_ty(&idx);
            let mutable = self.mut_use && self.solver.may_implement(base.ty, "index_mut", &[idx_ty]);
            let m = if mutable { self.lang_method("index_mut", "index_mut").unwrap_or(m) } else { m };
            let rm = if mutable { Mutbl::Mut } else { Mutbl::Not };
            let recv_t = tcx.mk(TyKind::Ref(rm, base.ty));
            let recv = self.temp(recv_t, n);
            self.next_two_phase = true;
            self.assign(&recv, Rvalue::Ref(rm, base), n);
            let out_t = tcx.mk(TyKind::Ref(rm, self.ty(n)?));
            let out = self.call_temp(Callee::Fn(m, margs), vec![Operand::Use(recv), idx], out_t, n);
            return self.deref(&out);
        }
        let elem = match tcx.kind(base.ty) {
            TyKind::Array(e, _) | TyKind::Slice(e) => e,
            _ => return Err(format!("index of {}", tcx.display(self.cx.r, base.ty))),
        };
        let idx = self.expr_value(kids[1])?;
        let il = match idx {
            Operand::Use(p) if p.proj.is_empty() => p.local,
            other => {
                let t = self.temp(self.usize_ty(), n);
                self.assign(&t, Rvalue::Use(other), n);
                t.local
            }
        };
        Ok(base.project(Proj::Index(il, elem), elem))
    }

    /// `n`'s own value (before adjustments) into `dest`.
    fn expr_into_raw(&mut self, n: NodeId, dest: &Place) -> R<()> {
        // (What is made here is a value, not a place that is changed.)
        let saved = std::mem::replace(&mut self.mut_use, false);
        // What is extended: through what only holds or wraps its operands.
        let entered = self.extending;
        let keep = matches!(self.ast.kind(n), Kind::Paren | Kind::Tuple | Kind::Array | Kind::StructExpr | Kind::Cast | Kind::Block | Kind::FieldInit | Kind::AttrExpr);
        self.extending = entered && keep;
        self.depth += 1;
        let r = if self.depth > 400 { Err("expressions nested too deep".to_string()) } else { self.expr_into_raw_in(n, dest, entered) };
        self.depth -= 1;
        self.extending = entered;
        self.mut_use = saved;
        r
    }

    fn expr_into_raw_in(&mut self, n: NodeId, dest: &Place, extending: bool) -> R<()> {
        let tcx = self.tcx();
        let kids = self.kids(n);
        match self.ast.kind(n) {
            Kind::Literal => {
                let t = self.ty(n)?;
                self.assign(dest, Rvalue::Use(Operand::Const(self.ast.token(n).0.to_string(), t)), n);
            }
            Kind::PathExpr => self.path_into(n, kids[0], dest)?,
            Kind::Paren | Kind::AttrExpr | Kind::ConstBlock => {
                let e = *kids.last().unwrap();
                self.expr_into(e, dest)?;
            }
            Kind::Tuple => {
                let ops = self.operands(&kids)?;
                self.assign(dest, Rvalue::Aggregate(AggKind::Tuple, ops), n);
            }
            Kind::Array => {
                let ops = self.operands(&kids)?;
                self.assign(dest, Rvalue::Aggregate(AggKind::Array, ops), n);
            }
            Kind::ArrayRepeat => {
                let o = self.expr_operand(kids[0])?;
                let count = self.text(kids[1]);
                self.assign(dest, Rvalue::Repeat(o, count), n);
            }
            Kind::Block => self.block_into(n, dest)?,
            Kind::Unary => self.unary_into(n, &kids, dest)?,
            Kind::Borrow => {
                let t = self.ty(n)?;
                let e = *kids.last().unwrap();
                let changes = matches!(tcx.kind(t), TyKind::Ref(Mutbl::Mut, _) | TyKind::Ptr(Mutbl::Mut, _));
                // (The operand of a borrow that is extended is extended.)
                let saved = std::mem::replace(&mut self.extending, extending);
                let p = self.expr_place_mut(e, changes);
                self.extending = saved;
                let p = p?;
                let rv = match tcx.kind(t) {
                    TyKind::Ref(m, _) => Rvalue::Ref(m, p),
                    TyKind::Ptr(m, _) => Rvalue::AddrOf(m, p),
                    _ => return Err("borrow of no reference type".into()),
                };
                self.assign(dest, rv, n);
            }
            Kind::Cast => {
                let o = self.expr_operand(kids[0])?;
                let t = self.ty(n)?;
                self.assign(dest, Rvalue::Cast(o, t), n);
            }
            Kind::Binary => self.binary_into(n, &kids, dest)?,
            Kind::Range => {
                let t = self.ty(n)?;
                let TyKind::Adt(d, l) = tcx.kind(t) else { return Err("range of no range type".into()) };
                let op_at = kids.iter().position(|&c| self.ast.kind(c) == Kind::Op).unwrap_or(0);
                let start = kids[..op_at].first().copied();
                let end = kids.get(op_at + 1).copied();
                let adt = tcx.adts.get(&d).ok_or("range type not found")?;
                let mut ops = Vec::new();
                for f in &adt.variants[0].fields {
                    ops.push(match (f.name, start, end) {
                        ("start", Some(s), _) => self.expr_operand(s)?,
                        ("end", _, Some(e)) => self.expr_operand(e)?,
                        ("exhausted", ..) => Operand::Const("false".into(), self.bool_ty()),
                        _ => return Err("range field not given".into()),
                    });
                }
                self.assign(dest, Rvalue::Aggregate(AggKind::Adt(d, 0, l), ops), n);
            }
            Kind::Call => self.call_expr(n, &kids, dest)?,
            Kind::MethodCall => {
                let &(m, margs) = self.tables.methods.get(&n).ok_or("method call without a method")?;
                let mut nodes = vec![kids[0]];
                nodes.extend(self.kids(kids[2]));
                let args = self.operands(&nodes)?;
                self.call_into(Callee::Fn(m, margs), args, dest, n);
            }
            Kind::Field | Kind::Index => {
                let p = self.unadjusted_place(n)?;
                self.assign(dest, Rvalue::Use(Operand::Use(p)), n);
            }
            Kind::StructExpr => self.struct_into(n, &kids, dest)?,
            Kind::If => self.if_into(n, &kids, dest)?,
            Kind::Match => self.match_into(n, &kids, dest)?,
            Kind::Loop => {
                let label = kids.first().filter(|&&c| self.ast.kind(c) == Kind::Lifetime).map(|&c| self.ast.token(c).0);
                let head = self.new_block();
                let exit = self.new_block();
                self.goto(head, n);
                self.at(head);
                self.loops.push(LoopCx { label, exit, cont: Some(head), dest: Some(dest.clone()), depth: self.scopes.len(), is_block: false });
                self.discard(*kids.last().unwrap(), n)?;
                self.loops.pop();
                self.goto(head, n);
                self.at(exit);
            }
            Kind::While => {
                let label = kids.first().filter(|&&c| self.ast.kind(c) == Kind::Lifetime).map(|&c| self.ast.token(c).0);
                let rest: Vec<NodeId> = kids.iter().copied().filter(|&c| self.ast.kind(c) != Kind::Lifetime).collect();
                let head = self.new_block();
                let exit = self.new_block();
                let fail = self.new_block();
                self.goto(head, n);
                self.at(head);
                let depth = self.scopes.len();
                self.push_scope(true);
                self.cond(rest[0], fail)?;
                self.loops.push(LoopCx { label, exit, cont: Some(head), dest: None, depth, is_block: false });
                self.discard(rest[1], n)?;
                self.loops.pop();
                self.exit_to(depth, n);
                self.goto(head, n);
                self.at(fail);
                self.exit_to(depth, n);
                self.scopes.pop();
                self.goto(exit, n);
                self.at(exit);
                self.assign(dest, Rvalue::Aggregate(AggKind::Tuple, Vec::new()), n);
            }
            Kind::For => self.for_into(n, &kids, dest)?,
            Kind::Break => {
                let label = kids.first().filter(|&&c| self.ast.kind(c) == Kind::Lifetime).map(|&c| self.ast.token(c).0);
                let value = kids.iter().copied().find(|&c| self.ast.kind(c) != Kind::Lifetime);
                let target = match label {
                    Some(l) => self.loops.iter().rposition(|x| x.label == Some(l)),
                    None => self.loops.iter().rposition(|x| !x.is_block),
                }
                .ok_or("`break` outside a loop")?;
                if let Some(v) = value {
                    match self.loops[target].dest.clone() {
                        Some(d) => self.expr_into(v, &d)?,
                        None => {
                            let o = self.expr_operand(v)?;
                            let _ = o;
                        }
                    }
                } else if let Some(d) = self.loops[target].dest.clone() {
                    self.assign(&d, Rvalue::Aggregate(AggKind::Tuple, Vec::new()), n);
                }
                let (exit, depth) = (self.loops[target].exit, self.loops[target].depth);
                self.exit_to(depth, n);
                self.goto(exit, n);
            }
            Kind::Continue => {
                let label = kids.first().filter(|&&c| self.ast.kind(c) == Kind::Lifetime).map(|&c| self.ast.token(c).0);
                let target = match label {
                    Some(l) => self.loops.iter().rposition(|x| x.label == Some(l)),
                    None => self.loops.iter().rposition(|x| !x.is_block),
                }
                .ok_or("`continue` outside a loop")?;
                let cont = self.loops[target].cont.ok_or("`continue` to a block")?;
                let depth = self.loops[target].depth;
                self.exit_to(depth, n);
                self.goto(cont, n);
            }
            Kind::Return => {
                let ret = Place::local(Local(0), self.cfg.locals[0].ty);
                match kids.first() {
                    Some(&e) => self.expr_into(e, &ret)?,
                    None => self.assign(&ret, Rvalue::Aggregate(AggKind::Tuple, Vec::new()), n),
                }
                self.exit_to(0, n);
                self.end(Term::Return, n);
            }
            Kind::Try => self.try_into(n, &kids, dest)?,
            Kind::Closure => self.closure_into(n, dest)?,
            Kind::MacroCall => {
                let name = self.ast.children(kids[0]).iter().rev().find(|&&s| self.ast.kind(s) == Kind::Segment).map_or("", |&s| self.ast.token(s).0);
                match name {
                    "panic" | "core_panic" | "std_panic" | "unreachable_2015" | "unreachable_2021" | "const_panic" | "unreachable" | "todo" | "unimplemented" => {
                        self.end(Term::Unreachable, n);
                    }
                    // Its operands are not followed.
                    "asm" | "global_asm" => {
                        let t = self.ty(n)?;
                        // Any named local in it may be one it gives a value.
                        let text = self.text(n);
                        let words: std::collections::HashSet<&str> = text.split(|c: char| !(c.is_alphanumeric() || c == '_')).collect();
                        let named: Vec<(Local, Ty)> = self
                            .cfg
                            .locals
                            .iter()
                            .enumerate()
                            .filter(|(_, d)| matches!(d.kind, LocalKind::Var | LocalKind::Arg) && d.name.as_deref().is_some_and(|nm| words.contains(nm)))
                            .map(|(i, d)| (Local(i as u32), d.ty))
                            .collect();
                        for (l, lt) in named {
                            self.emit(StmtKind::MaybeInit(Place::local(l, lt)), n);
                        }
                        self.assign(dest, Rvalue::Use(Operand::Const("asm!(..)".into(), t)), n);
                    }
                    _ => return Err(format!("macro `{name}!`")),
                }
            }
            Kind::Builtin => match self.ast.token(kids[0]).0 {
                "type_ascribe" | "wrap_binder" | "unwrap_binder" => self.expr_into(kids[1], dest)?,
                // A number the layout gives.
                "offset_of" => {
                    let t = self.ty(n)?;
                    self.assign(dest, Rvalue::Use(Operand::Const(self.text(n), t)), n);
                }
                b => return Err(format!("`builtin # {b}`")),
            },
            Kind::Await => return Err("`.await`".into()),
            k => return Err(format!("{k:?} `{}`", self.text(n).chars().take(50).collect::<String>())),
        }
        Ok(())
    }

    /// `n`'s value, which is not wanted, in a scope of its own.
    fn discard(&mut self, n: NodeId, node: NodeId) -> R<()> {
        self.push_scope(false);
        let t = self.ty(n)?;
        let sink = self.temp(t, node);
        self.expr_into(n, &sink)?;
        self.pop_scope(node);
        Ok(())
    }

    fn path_into(&mut self, n: NodeId, path: NodeId, dest: &Place) -> R<()> {
        let tcx = self.tcx();
        let t = self.ty(n)?;
        if let Some(Res::Local(p)) = self.res(path)
            && !self.tables.consts.contains_key(&n)
        {
            let src = self.place_of_local(*p)?;
            self.assign(dest, Rvalue::Use(Operand::Use(src)), n);
            return Ok(());
        }
        if let TyKind::Adt(d, l) = tcx.kind(t)
            && !self.tables.consts.contains_key(&n)
            && !matches!(self.res(path), Some(Res::Def(x)) if matches!(self.cx.r.defs[x.0 as usize].kind, DefKind::Const | DefKind::Static | DefKind::AssocConst))
        {
            // A unit struct or variant.
            let v = self.variant_of_path(path, d).unwrap_or(0);
            self.assign(dest, Rvalue::Aggregate(AggKind::Adt(d, v, l), Vec::new()), n);
            return Ok(());
        }
        self.assign(dest, Rvalue::Use(Operand::Const(self.text(n), t)), n);
        Ok(())
    }

    fn unary_into(&mut self, n: NodeId, kids: &[NodeId], dest: &Place) -> R<()> {
        if self.ast.kind(kids[0]) == Kind::Keyword {
            return Err("`box`".into());
        }
        let op = self.ast.token(kids[0]).0;
        if op == "*" {
            let p = self.unadjusted_place(n)?;
            self.assign(dest, Rvalue::Use(Operand::Use(p)), n);
            return Ok(());
        }
        let o = self.expr_operand(kids[1])?;
        let ot = match &o {
            Operand::Use(p) => p.ty,
            Operand::Const(_, t) => *t,
        };
        if !self.is_prim(ot)
            && let Some(&(m, margs)) = self.tables.methods.get(&n)
        {
            self.call_into(Callee::Fn(m, margs), vec![o], dest, n);
            return Ok(());
        }
        self.assign(dest, Rvalue::Unary(op.to_string(), o), n);
        Ok(())
    }

    fn operand_ty(&self, o: &Operand) -> Ty {
        match o {
            Operand::Use(p) => p.ty,
            Operand::Const(_, t) => *t,
        }
    }

    fn binary_into(&mut self, n: NodeId, kids: &[NodeId], dest: &Place) -> R<()> {
        let tcx = self.tcx();
        let op = self.text(kids[1]);
        match op.as_str() {
            "&&" | "||" => {
                // As control flow, as it is in a condition: the right side
                // runs only if the left said so. `dest` is assigned once.
                let (yes, no, join) = (self.new_block(), self.new_block(), self.new_block());
                self.cond_branch(n, yes, no)?;
                let b = self.bool_ty();
                self.at(yes);
                self.assign(dest, Rvalue::Use(Operand::Const("true".into(), b)), n);
                self.goto(join, n);
                self.at(no);
                self.assign(dest, Rvalue::Use(Operand::Const("false".into(), b)), n);
                self.goto(join, n);
                self.at(join);
            }
            "=" => {
                let rhs = self.expr_operand(kids[2])?;
                self.assign_to(kids[0], rhs, n)?;
                self.assign(dest, Rvalue::Aggregate(AggKind::Tuple, Vec::new()), n);
            }
            _ => {
                let compound = op.ends_with('=') && !matches!(op.as_str(), "==" | "!=" | "<=" | ">=");
                if compound {
                    let base = op.trim_end_matches('=').to_string();
                    let rhs = self.expr_operand(kids[2])?;
                    let p = self.expr_place_mut(kids[0], true)?;
                    if let Some(&(m, margs)) = self.tables.methods.get(&n)
                        && !(self.is_prim(p.ty) && self.is_prim(self.operand_ty(&rhs)))
                    {
                        let rt = tcx.mk(TyKind::Ref(Mutbl::Mut, p.ty));
                        let recv = self.temp(rt, n);
                        self.next_two_phase = true;
                        self.assign(&recv, Rvalue::Ref(Mutbl::Mut, p), n);
                        let u = self.unit();
                        let sink = self.temp(u, n);
                        self.call_into(Callee::Fn(m, margs), vec![Operand::Use(recv), rhs], &sink, n);
                    } else {
                        self.assign(&p, Rvalue::Binary(base, Operand::Use(p.clone()), rhs), n);
                    }
                    self.assign(dest, Rvalue::Aggregate(AggKind::Tuple, Vec::new()), n);
                    return Ok(());
                }
                let a = self.expr_operand(kids[0])?;
                let a = if self.is_pure(kids[2]) { a } else { self.freeze(a, kids[0]) };
                let b = self.expr_operand(kids[2])?;
                let cmp = matches!(op.as_str(), "==" | "!=" | "<" | "<=" | ">" | ">=");
                if let Some(&(m, margs)) = self.tables.methods.get(&n)
                    && !(self.is_prim(self.operand_ty(&a)) && self.is_prim(self.operand_ty(&b)))
                {
                    // Comparisons take both by reference.
                    let args = if cmp {
                        let (ra, rb) = (self.borrow_operand(a, n), self.borrow_operand(b, n));
                        vec![ra, rb]
                    } else {
                        vec![a, b]
                    };
                    self.call_into(Callee::Fn(m, margs), args, dest, n);
                } else {
                    self.assign(dest, Rvalue::Binary(op, a, b), n);
                }
            }
        }
        Ok(())
    }

    /// `&o`, as an operand.
    fn borrow_operand(&mut self, o: Operand, node: NodeId) -> Operand {
        let p = match o {
            Operand::Use(p) => p,
            Operand::Const(c, t) => {
                let tmp = self.temp(t, node);
                self.assign(&tmp, Rvalue::Use(Operand::Const(c, t)), node);
                tmp
            }
        };
        let rt = self.tcx().mk(TyKind::Ref(Mutbl::Not, p.ty));
        let r = self.temp(rt, node);
        self.assign(&r, Rvalue::Ref(Mutbl::Not, p), node);
        Operand::Use(r)
    }

    /// `lhs = value`: to a place, or destructuring a tuple or array.
    fn assign_to(&mut self, lhs: NodeId, value: Operand, node: NodeId) -> R<()> {
        match self.ast.kind(lhs) {
            Kind::Underscore => Ok(()),
            Kind::Paren => {
                let k = self.kids(lhs);
                self.assign_to(k[0], value, node)
            }
            Kind::Tuple => {
                let src = match value {
                    Operand::Use(p) => p,
                    Operand::Const(c, t) => {
                        let tmp = self.temp(t, node);
                        self.assign(&tmp, Rvalue::Use(Operand::Const(c, t)), node);
                        tmp
                    }
                };
                let elems = self.kids(lhs);
                if elems.iter().any(|&e| self.ast.kind(e) == Kind::Range) {
                    return Err("`..` in a destructuring assignment".into());
                }
                for (i, &e) in elems.iter().enumerate() {
                    let fp = self.field(&src, 0, i)?;
                    self.assign_to(e, Operand::Use(fp), node)?;
                }
                Ok(())
            }
            Kind::StructExpr | Kind::Call | Kind::Array => Err("destructuring assignment of a struct or array".into()),
            _ => {
                let p = self.expr_place_mut(lhs, true)?;
                self.assign(&p, Rvalue::Use(value), node);
                Ok(())
            }
        }
    }

    fn struct_into(&mut self, n: NodeId, kids: &[NodeId], dest: &Place) -> R<()> {
        let tcx = self.tcx();
        let t = self.ty(n)?;
        let TyKind::Adt(d, l) = tcx.kind(t) else { return Err("struct literal of something not a struct".into()) };
        let adt = tcx.adts.get(&d).ok_or("not an ADT")?;
        let v = self.variant_of_path(kids[0], d).unwrap_or(0);
        if adt.kind == AdtKind::Union {
            // The one field given.
            let fields: Vec<&str> = adt.variants[0].fields.iter().map(|f| f.name).collect();
            let fi = kids[1..].iter().copied().find(|&f| self.ast.kind(f) == Kind::FieldInit).ok_or("a union literal without a field")?;
            let fk = self.kids(fi);
            let name = self.ast.token(fk[0]).0;
            let i = fields.iter().position(|&x| x == name).ok_or_else(|| format!("no field `{name}`"))?;
            let o = match fk.get(1) {
                Some(&e) => self.expr_operand(e)?,
                None => {
                    let Some(Res::Local(lp)) = self.res(fk[0]) else { return Err("shorthand field without a local".into()) };
                    Operand::Use(self.place_of_local(*lp)?)
                }
            };
            self.assign(dest, Rvalue::Aggregate(AggKind::Union(d, i as u32, l), vec![o]), n);
            return Ok(());
        }
        let names: Vec<&str> = adt.variants[v as usize].fields.iter().map(|f| f.name).collect();
        let mut given: Vec<Option<Operand>> = vec![None; names.len()];
        let mut base: Option<NodeId> = None;
        for &f in &kids[1..] {
            let fk = self.kids(f);
            match self.ast.kind(f) {
                Kind::FieldInit => {
                    let name = self.ast.token(fk[0]).0;
                    let i = names.iter().position(|&x| x == name).ok_or_else(|| format!("no field `{name}`"))?;
                    let o = match fk.get(1) {
                        Some(&e) => self.expr_operand(e)?,
                        None => {
                            let Some(Res::Local(lp)) = self.res(fk[0]) else { return Err("shorthand field without a local".into()) };
                            Operand::Use(self.place_of_local(*lp)?)
                        }
                    };
                    given[i] = Some(if kids.len() > 2 { self.freeze(o, f) } else { o });
                }
                Kind::StructBase => base = fk.first().copied(),
                _ => {}
            }
        }
        let mut ops = Vec::new();
        let mut base_place: Option<Place> = None;
        for (i, g) in given.into_iter().enumerate() {
            ops.push(match g {
                Some(o) => o,
                None => {
                    let b = match &base_place {
                        Some(p) => p.clone(),
                        None => {
                            let b = base.ok_or("a field not given")?;
                            let p = self.expr_place(b)?;
                            base_place = Some(p.clone());
                            p
                        }
                    };
                    Operand::Use(self.field(&b, v, i)?)
                }
            });
        }
        self.assign(dest, Rvalue::Aggregate(AggKind::Adt(d, v, l), ops), n);
        Ok(())
    }

    // ---- calls ----

    fn call_expr(&mut self, n: NodeId, kids: &[NodeId], dest: &Place) -> R<()> {
        let tcx = self.tcx();
        let arg_nodes = self.kids(kids[1]);
        if let Some(&(m, margs)) = self.tables.methods.get(&n) {
            // Through the `Fn` traits: `f(a, b)` is `Fn::call(&f, (a, b))`.
            let name = self.cx.r.defs[m.0 as usize].name;
            let mut callee = self.expr_place_mut(kids[0], name == "call_mut")?;
            // The callee is what `Self` is: through the references (and boxes)
            // it is behind (`f: &mut F` calls `F`).
            let self_ty = tcx.tys(margs).first().copied();
            let mut guard = 0;
            while Some(callee.ty) != self_ty && self.pointee(callee.ty).is_some() && guard < 4 {
                callee = self.deref(&callee)?;
                guard += 1;
            }
            // Through a reference too, where `Self` is the reference (`impl
            // FnMut for &mut F`): its variable is not borrowed, what it
            // points to is.
            while name != "call_once" && matches!(tcx.kind(callee.ty), TyKind::Ref(..)) && guard < 4 {
                callee = self.deref(&callee)?;
                guard += 1;
            }
            let recv = match name {
                "call" => self.borrow_operand(Operand::Use(callee), n),
                "call_mut" => {
                    let rt = tcx.mk(TyKind::Ref(Mutbl::Mut, callee.ty));
                    let r = self.temp(rt, n);
                    self.next_two_phase = true;
                    self.assign(&r, Rvalue::Ref(Mutbl::Mut, callee), n);
                    Operand::Use(r)
                }
                _ => Operand::Use(callee),
            };
            let ops = self.operands(&arg_nodes)?;
            let tuple_t = tcx.mk(TyKind::Tuple(tcx.list(ops.iter().map(|o| self.operand_ty(o)).collect())));
            let tuple = self.temp(tuple_t, n);
            self.assign(&tuple, Rvalue::Aggregate(AggKind::Tuple, ops), n);
            self.call_into(Callee::Fn(m, margs), vec![recv, Operand::Use(tuple)], dest, n);
            return Ok(());
        }
        let ct = self.ty(kids[0])?;
        let args = self.operands(&arg_nodes)?;
        match tcx.kind(ct) {
            TyKind::FnDef(d, l) => match self.cx.r.defs[d.0 as usize].kind {
                DefKind::Struct => {
                    let TyKind::Adt(e, al) = tcx.kind(self.ty(n)?) else { return Err("constructor of no ADT".into()) };
                    self.assign(dest, Rvalue::Aggregate(AggKind::Adt(e, 0, al), args), n);
                }
                DefKind::Variant => {
                    let TyKind::Adt(e, al) = tcx.kind(self.ty(n)?) else { return Err("variant of no enum".into()) };
                    let v = tcx.adts.get(&e).and_then(|a| a.variants.iter().position(|x| x.def == Some(d))).ok_or("no such variant")? as u32;
                    self.assign(dest, Rvalue::Aggregate(AggKind::Adt(e, v, al), args), n);
                }
                _ => self.call_into(Callee::Fn(d, l), args, dest, n),
            },
            TyKind::FnPtr(_) => {
                let f = self.expr_operand(kids[0])?;
                self.call_into(Callee::Value(f), args, dest, n);
            }
            // A closure (or a reference to one) called as itself.
            TyKind::Closure(..) | TyKind::Ref(..) => {
                let f = self.expr_place(kids[0])?;
                self.call_into(Callee::Value(Operand::Use(f)), args, dest, n);
            }
            _ => return Err(format!("call of {}", tcx.display(self.cx.r, ct))),
        }
        Ok(())
    }

    // ---- control flow ----

    fn block_into(&mut self, n: NodeId, dest: &Place) -> R<()> {
        let kids = self.kids(n);
        if kids.iter().any(|&c| self.ast.kind(c) == Kind::Keyword && matches!(self.ast.token(c).0, "async" | "gen")) {
            return Err("an `async` or `gen` block".into());
        }
        if kids.iter().any(|&c| self.ast.kind(c) == Kind::Keyword && self.ast.token(c).0 == "try") {
            return self.try_block(n, &kids, dest);
        }
        let label = kids.first().filter(|&&c| self.ast.kind(c) == Kind::Lifetime).map(|&c| self.ast.token(c).0);
        let stmts: Vec<NodeId> = kids.iter().copied().filter(|&c| !matches!(self.ast.kind(c), Kind::Lifetime | Kind::Keyword | Kind::InnerAttribute)).collect();
        let exit = label.map(|l| {
            let exit = self.new_block();
            self.loops.push(LoopCx { label: Some(l), exit, cont: None, dest: Some(dest.clone()), depth: self.scopes.len(), is_block: true });
            exit
        });
        self.stmts(&stmts, dest, n)?;
        if let Some(exit) = exit {
            self.loops.pop();
            self.goto(exit, n);
            self.at(exit);
        }
        Ok(())
    }

    /// A block's statements, in a scope of their own for the variables, each
    /// in one for its temporaries; the last, if an expression, is the value.
    fn stmts(&mut self, stmts: &[NodeId], dest: &Place, node: NodeId) -> R<()> {
        let adjusted = std::mem::take(&mut self.block_adjusted);
        self.push_scope(true);
        let mut valued = false;
        let last = stmts.len().checked_sub(1);
        for (i, &s) in stmts.iter().enumerate() {
            match self.ast.kind(s) {
                Kind::LetStmt => self.let_stmt(s)?,
                Kind::ExprStmt => {
                    let Some(&e) = self.ast.children(s).first() else { continue };
                    self.push_scope(false);
                    let t = self.ty(e)?;
                    let sink = self.temp(t, s);
                    self.expr_into(e, &sink)?;
                    self.pop_scope(s);
                }
                Kind::EmptyStmt => {}
                k if is_item(k) => {}
                _ if Some(i) == last && self.scopes.len() >= 2 && (adjusted || matches!(self.cx.crates[self.cfg.krate as usize].edition, crate::lexer::Edition::E2015 | crate::lexer::Edition::E2018 | crate::lexer::Edition::E2021)) => {
                    let saved = self.tail_outer.replace((self.scopes.len() - 2, self.scopes.len()));
                    let r = self.expr_into(s, dest);
                    self.tail_outer = saved;
                    r?;
                    valued = true;
                }
                _ if Some(i) == last => {
                    self.push_scope(false);
                    self.expr_into(s, dest)?;
                    self.pop_scope(s);
                    valued = true;
                }
                _ => {
                    self.push_scope(false);
                    let t = self.ty(s)?;
                    let sink = self.temp(t, s);
                    self.expr_into(s, &sink)?;
                    self.pop_scope(s);
                }
            }
        }
        if !valued {
            // No tail expression: `()`, unless the block cannot be left.
            if self.cur.is_some() && !self.is_never(dest.ty) {
                self.assign(dest, Rvalue::Aggregate(AggKind::Tuple, Vec::new()), node);
            }
        }
        self.pop_scope(node);
        Ok(())
    }

    /// `try { .. }`: `?` in it leaves it, with the residual; its value is
    /// made by `Try::from_output`.
    fn try_block(&mut self, n: NodeId, kids: &[NodeId], dest: &Place) -> R<()> {
        let tcx = self.tcx();
        let t = self.ty(n)?;
        let from_output = self.lang_method("Try", "from_output").ok_or("no `Try::from_output`")?;
        let margs = tcx.list(vec![t]);
        let out_ty = self.arg_ty(from_output, margs, 0)?;
        let value = self.temp(out_ty, n);
        let exit = self.new_block();
        self.tries.push((exit, dest.clone(), self.scopes.len()));
        let stmts: Vec<NodeId> = kids.iter().copied().filter(|&c| !matches!(self.ast.kind(c), Kind::Lifetime | Kind::Keyword | Kind::InnerAttribute)).collect();
        let r = self.stmts(&stmts, &value, n);
        self.tries.pop();
        r?;
        if self.cur.is_some() {
            self.call_into(Callee::Fn(from_output, margs), vec![Operand::Use(value)], dest, n);
        }
        self.goto(exit, n);
        self.at(exit);
        Ok(())
    }

    fn let_stmt(&mut self, s: NodeId) -> R<()> {
        let sk: Vec<NodeId> = self.kids(s).into_iter().filter(|&c| self.ast.kind(c) != Kind::Keyword).collect();
        let pat = sk[0];
        let mut rest = &sk[1..];
        let mut static_ann = false;
        if rest.first().is_some_and(|&t| crate::ty::lower::is_type(self.ast.kind(t))) {
            let t = rest[0];
            static_ann = self.ast.kind(t) == Kind::RefType
                && self.ast.children(t).iter().any(|&c| self.ast.kind(c) == Kind::Lifetime && self.ast.token(c).0 == "'static");
            rest = &rest[1..];
        }
        let Some(&init) = rest.first() else {
            // Declared, assigned later.
            return self.declare_bindings(pat);
        };
        let block_scope = self.scopes.iter().rposition(|x| x.vars).unwrap_or(0);
        // The initialiser's temporaries: its own scope, except what a
        // reference in it keeps alive (`let x = &f();`).
        self.push_scope(false);
        let stmt_scope = self.scopes.len() - 1;
        let result = (|| -> R<()> {
            if let Some(&els) = rest.get(1) {
                self.extend = Some(block_scope);
                self.extending = true;
                let p = self.expr_place_mut(init, self.binds_mut(pat));
                self.extend = None;
                self.extending = false;
                let p = p?;
                let fail = self.new_block();
                let ok = self.new_block();
                let mark = self.binds.len();
                self.match_pat(pat, p, Some(fail))?;
                self.flush_binds(mark, s);
                self.goto(ok, s);
                self.at(fail);
                let u = self.never();
                let sink = self.temp(u, s);
                self.expr_into(els, &sink)?;
                if self.cur.is_some() {
                    self.end(Term::Unreachable, s);
                }
                self.at(ok);
                return Ok(());
            }
            if self.is_simple_binding(pat) {
                let l = self.declare_var(pat)?;
                if static_ann {
                    self.cfg.static_vars.push(l);
                }
                let place = Place::local(l, self.cfg.locals[l.0 as usize].ty);
                self.extend = Some(block_scope);
                self.extending = true;
                let r = self.expr_into_extending(init, &place);
                self.extend = None;
                self.extending = false;
                return r;
            }
            self.extend = Some(block_scope);
            self.extending = true;
            let p = self.expr_place_mut(init, self.binds_mut(pat));
            self.extend = None;
            self.extending = false;
            let p = p?;
            let mark = self.binds.len();
            self.match_pat(pat, p, None)?;
            self.flush_binds(mark, s);
            Ok(())
        })();
        // The statement scope's temporaries die here; those the `let`
        // extended were put in the block's.
        let _ = stmt_scope;
        self.pop_scope(s);
        result
    }

    /// `n` into `dest`, a reference's operand's temporary kept for the block
    /// (see `let_stmt`).
    fn expr_into_extending(&mut self, n: NodeId, dest: &Place) -> R<()> {
        self.expr_into(n, dest)
    }

    fn is_simple_binding(&self, pat: NodeId) -> bool {
        self.ast.kind(pat) == Kind::IdentPat
            && self.res(pat).is_none()
            && self.tables.bindings.get(&pat) == Some(&BindingMode::Value)
            && !self.tables.pat_derefs.contains_key(&pat)
            && self.ast.children(pat).iter().all(|&c| matches!(self.ast.kind(c), Kind::Keyword | Kind::Name))
    }

    /// The variable a binding pattern declares: made (and live) at first.
    fn declare_var(&mut self, pat: NodeId) -> R<Local> {
        let key = self.canon(pat);
        if let Some(&l) = self.locals.get(&key) {
            return Ok(l);
        }
        let ty = self.ty(pat)?;
        let name = self.ast.child(pat, Kind::Name).map(|nm| self.ast.token(nm).0.to_string());
        let mutable = self.ast.children(pat).iter().any(|&c| self.ast.kind(c) == Kind::Keyword && self.ast.token(c).0 == "mut");
        let l = self.new_local(ty, LocalKind::Var, name, mutable, Some(pat));
        self.register(l, true);
        self.emit(StmtKind::StorageLive(l), pat);
        self.locals.insert(key, l);
        Ok(l)
    }

    fn declare_bindings(&mut self, pat: NodeId) -> R<()> {
        if self.ast.kind(pat) == Kind::IdentPat && self.res(pat).is_none() {
            self.declare_var(pat)?;
        }
        for c in self.kids(pat) {
            if is_pattern(self.ast.kind(c)) || self.ast.kind(c) == Kind::FieldPat {
                self.declare_bindings(c)?;
            }
        }
        Ok(())
    }

    /// Assigns what the patterns matched (from `from`) to their variables.
    fn flush_binds(&mut self, from: usize, node: NodeId) {
        self.flush_binds_in(from, node, None);
    }

    /// The same for an arm with a guard: a variable bound by value gets a
    /// stand-in for the guard to read, and what it is bound to is `rebind`s
    /// once the guard holds (the guard sees the scrutinee, not a move out of
    /// it, and an arm after a failed guard still has all of it).
    fn flush_binds_guarded(&mut self, from: usize, node: NodeId) -> Vec<(Local, Place)> {
        let mut later = Vec::new();
        self.flush_binds_in(from, node, Some(&mut later));
        later
    }

    fn rebind(&mut self, later: Vec<(Local, Place)>, node: NodeId) {
        for (l, from) in later {
            let ty = self.cfg.locals[l.0 as usize].ty;
            self.emit(StmtKind::StorageLive(l), node);
            self.assign(&Place::local(l, ty), Rvalue::Use(Operand::Use(from)), node);
        }
    }

    fn flush_binds_in(&mut self, from: usize, node: NodeId, mut later: Option<&mut Vec<(Local, Place)>>) {
        let binds: Vec<Bind> = self.binds.drain(from..).collect();
        for b in binds {
            let Ok(l) = self.declare_var(b.pat) else { continue };
            let ty = self.cfg.locals[l.0 as usize].ty;
            let place = Place::local(l, ty);
            let rv = match (b.mode, later.as_deref_mut()) {
                (BindingMode::Value, Some(later)) => {
                    later.push((l, b.place));
                    Rvalue::Use(Operand::Const("<what the guard reads>".into(), ty))
                }
                (BindingMode::Value, None) => Rvalue::Use(Operand::Use(b.place)),
                (BindingMode::Ref(m), _) => Rvalue::Ref(m, b.place),
            };
            self.assign(&place, rv, node);
        }
    }

    /// `n` as a condition: control goes to `yes` if it holds and to `no` if
    /// not, `&&` and `||` by their short-circuiting (no value is made of the
    /// parts).
    fn cond_branch(&mut self, n: NodeId, yes: Block, no: Block) -> R<()> {
        let kids = self.kids(n);
        match self.ast.kind(n) {
            Kind::Paren => self.cond_branch(kids[0], yes, no),
            Kind::Binary if self.text(kids[1]) == "&&" => {
                let mid = self.new_block();
                self.cond_branch(kids[0], mid, no)?;
                self.at(mid);
                self.cond_branch(kids[2], yes, no)
            }
            Kind::Binary if self.text(kids[1]) == "||" => {
                let mid = self.new_block();
                self.cond_branch(kids[0], yes, mid)?;
                self.at(mid);
                self.cond_branch(kids[2], yes, no)
            }
            _ => {
                self.push_scope(false);
                let o = self.expr_operand(n)?;
                let c = self.temp_outside(self.bool_ty(), n);
                self.assign(&c, Rvalue::Use(o), n);
                self.pop_scope(n);
                self.branch(Operand::Use(c), yes, no, n);
                Ok(())
            }
        }
    }

    /// The condition of an `if` or `while` (with `let`s, and `&&`): control
    /// goes on if it holds and to `els` if not.
    fn cond(&mut self, n: NodeId, els: Block) -> R<()> {
        let kids = self.kids(n);
        match self.ast.kind(n) {
            Kind::Binary if self.text(kids[1]) == "||" => {
                let then = self.new_block();
                self.cond_branch(n, then, els)?;
                self.at(then);
                Ok(())
            }
            Kind::Binary if self.text(kids[1]) == "&&" => {
                self.cond(kids[0], els)?;
                self.cond(kids[2], els)
            }
            Kind::Paren => self.cond(kids[0], els),
            Kind::Let => {
                let p = self.expr_place_mut(kids[1], self.binds_mut(kids[0]))?;
                let mark = self.binds.len();
                self.match_pat(kids[0], p, Some(els))?;
                self.flush_binds(mark, n);
                Ok(())
            }
            _ => {
                self.push_scope(false);
                let o = self.expr_operand(n)?;
                // A bool operand outlives the scope: copy it.
                let c = self.temp_outside(self.bool_ty(), n);
                self.assign(&c, Rvalue::Use(o), n);
                self.pop_scope(n);
                let then = self.new_block();
                self.branch(Operand::Use(c), then, els, n);
                self.at(then);
                Ok(())
            }
        }
    }

    /// A temporary in the scope outside the innermost.
    fn temp_outside(&mut self, ty: Ty, node: NodeId) -> Place {
        let l = self.new_local(ty, LocalKind::Temp, None, true, None);
        let i = self.scopes.len().saturating_sub(2);
        if let Some(s) = self.scopes.get_mut(i) {
            s.locals.push(l);
        }
        self.emit(StmtKind::StorageLive(l), node);
        Place::local(l, ty)
    }

    fn if_into(&mut self, n: NodeId, kids: &[NodeId], dest: &Place) -> R<()> {
        let els = self.new_block();
        let join = self.new_block();
        let depth = self.scopes.len();
        // What an `if let` binds, and its scrutinee's temporaries: for the
        // `then` branch, dropped before the `else`.
        self.push_scope(true);
        self.cond(kids[0], els)?;
        self.expr_into(kids[1], dest)?;
        self.exit_to(depth, n);
        self.goto(join, n);
        self.at(els);
        self.exit_to(depth, n);
        self.scopes.pop();
        match kids.get(2) {
            Some(&e) => self.expr_into(e, dest)?,
            None => self.assign(dest, Rvalue::Aggregate(AggKind::Tuple, Vec::new()), n),
        }
        self.goto(join, n);
        self.at(join);
        Ok(())
    }

    fn match_into(&mut self, n: NodeId, kids: &[NodeId], dest: &Place) -> R<()> {
        let arm_pats: Vec<NodeId> = kids[1..].iter().filter(|&&a| self.ast.kind(a) == Kind::MatchArm).filter_map(|&a| self.kids(a).first().copied()).collect();
        let changes = arm_pats.iter().any(|&p| self.binds_mut(p));
        let scrut = self.expr_place_mut(kids[0], changes)?;
        let join = self.new_block();
        for &arm in &kids[1..] {
            if self.ast.kind(arm) != Kind::MatchArm {
                continue;
            }
            let ak = self.kids(arm);
            let next = self.new_block();
            let depth = self.scopes.len();
            self.push_scope(true);
            let mark = self.binds.len();
            self.match_pat(ak[0], scrut.clone(), Some(next))?;
            let mut i = 1;
            if let Some(&g) = ak.get(1)
                && self.ast.kind(g) == Kind::Guard
            {
                let later = self.flush_binds_guarded(mark, arm);
                let ge = self.ast.children(g)[0];
                self.cond(ge, next)?;
                self.rebind(later, arm);
                i = 2;
            } else {
                self.flush_binds(mark, arm);
            }
            if let Some(&body) = ak.get(i) {
                self.expr_into(body, dest)?;
            }
            self.exit_to(depth, arm);
            self.goto(join, arm);
            // Failing the pattern or guard: out of the arm's scope too.
            self.at(next);
            self.exit_to(depth, arm);
            self.scopes.pop();
        }
        // No arm matched: cannot happen.
        if self.cur.is_some() {
            self.end(Term::Unreachable, n);
        }
        self.at(join);
        Ok(())
    }

    fn for_into(&mut self, n: NodeId, kids: &[NodeId], dest: &Place) -> R<()> {
        let tcx = self.tcx();
        let rest: Vec<NodeId> = kids.iter().copied().filter(|&c| !matches!(self.ast.kind(c), Kind::Lifetime | Kind::Keyword)).collect();
        let label = kids.first().filter(|&&c| self.ast.kind(c) == Kind::Lifetime).map(|&c| self.ast.token(c).0);
        let calls = self.tables.implicit.get(&n).cloned().ok_or("`for` without its calls")?;
        let [(into_iter, l1), (next, l2)] = calls[..] else { return Err("`for` without its calls".into()) };
        let outer = self.scopes.len();
        self.push_scope(false);
        let src = self.expr_operand(rest[1])?;
        let it_ty = self.ret_ty(into_iter, l1)?;
        let it = self.call_temp(Callee::Fn(into_iter, l1), vec![src], it_ty, n);
        let head = self.new_block();
        let exit = self.new_block();
        let done = self.new_block();
        self.goto(head, n);
        self.at(head);
        let depth = self.scopes.len();
        self.push_scope(true);
        let rt = tcx.mk(TyKind::Ref(Mutbl::Mut, it.ty));
        let recv = self.temp(rt, n);
        self.assign(&recv, Rvalue::Ref(Mutbl::Mut, it.clone()), n);
        let item_ty = self.ret_ty(next, l2)?;
        let item = self.call_temp(Callee::Fn(next, l2), vec![Operand::Use(recv)], item_ty, n);
        let TyKind::Adt(od, _) = tcx.kind(item.ty) else { return Err("`next` of no `Option`".into()) };
        let some = self.variant_named(od, "Some").ok_or("no `Some`")?;
        let d = self.temp(self.usize_ty(), n);
        self.assign(&d, Rvalue::Discriminant(item.clone()), n);
        let body = self.new_block();
        self.end(Term::Switch { discr: Operand::Use(d), targets: vec![(some as u128, body)], otherwise: done }, n);
        self.at(body);
        let x = self.field(&item, some, 0)?;
        let mark = self.binds.len();
        self.match_pat(rest[0], x, None)?;
        self.flush_binds(mark, n);
        self.loops.push(LoopCx { label, exit, cont: Some(head), dest: None, depth, is_block: false });
        self.discard(rest[2], n)?;
        self.loops.pop();
        self.exit_to(depth, n);
        self.goto(head, n);
        self.at(done);
        self.exit_to(depth, n);
        self.scopes.pop();
        self.goto(exit, n);
        self.at(exit);
        self.exit_to(outer, n);
        self.scopes.pop();
        self.assign(dest, Rvalue::Aggregate(AggKind::Tuple, Vec::new()), n);
        Ok(())
    }

    fn try_into(&mut self, n: NodeId, kids: &[NodeId], dest: &Place) -> R<()> {
        let tcx = self.tcx();
        let calls = self.tables.implicit.get(&n).cloned().ok_or("`?` without its calls")?;
        let [(branch, l1), (from_residual, l2)] = calls[..] else { return Err("`?` without its calls".into()) };
        let v = self.expr_operand(kids[0])?;
        let cf_ty = self.ret_ty(branch, l1)?;
        let cf = self.call_temp(Callee::Fn(branch, l1), vec![v], cf_ty, n);
        let TyKind::Adt(cd, _) = tcx.kind(cf.ty) else { return Err("`branch` of no `ControlFlow`".into()) };
        let cont = self.variant_named(cd, "Continue").ok_or("no `Continue`")?;
        let brk = self.variant_named(cd, "Break").ok_or("no `Break`")?;
        let d = self.temp(self.usize_ty(), n);
        self.assign(&d, Rvalue::Discriminant(cf.clone()), n);
        let go_on = self.new_block();
        let bail = self.new_block();
        self.end(Term::Switch { discr: Operand::Use(d), targets: vec![(cont as u128, go_on)], otherwise: bail }, n);
        self.at(bail);
        let residual = self.field(&cf, brk, 0)?;
        match self.tries.last().cloned() {
            // In a `try` block: its value, and out of it.
            Some((exit, tdest, depth)) => {
                self.call_into(Callee::Fn(from_residual, l2), vec![Operand::Use(residual)], &tdest, n);
                self.exit_to(depth, n);
                self.goto(exit, n);
            }
            None => {
                let ret = Place::local(Local(0), self.cfg.locals[0].ty);
                self.call_into(Callee::Fn(from_residual, l2), vec![Operand::Use(residual)], &ret, n);
                self.exit_to(0, n);
                self.end(Term::Return, n);
            }
        }
        self.at(go_on);
        let f = self.field(&cf, cont, 0)?;
        self.assign(dest, Rvalue::Use(Operand::Use(f)), n);
        Ok(())
    }

    fn closure_into(&mut self, n: NodeId, dest: &Place) -> R<()> {
        let tcx = self.tcx();
        let t = self.ty(n)?;
        let TyKind::Closure(c, _, _) = tcx.kind(t) else { return Err("closure of no closure type".into()) };
        let caps = self.cx.res.captures.get(&c).cloned().unwrap_or_default();
        let mut ops = Vec::new();
        for cap in &caps {
            let src = self.place_of_local(cap.local)?;
            // (A `use ||` closure clones what it uses, where it can.)
            let uses = self.ast.children(n).iter().any(|&c| self.ast.kind(c) == Kind::Keyword && self.ast.token(c).0 == "use");
            if cap.by_value && !uses && !self.only_fields_used(n, cap.local) {
                ops.push(Operand::Use(src));
            } else {
                // (Shared: whether it is changed through is not recorded.)
                let rt = self.tcx().mk(TyKind::Ref(Mutbl::Not, src.ty));
                let t = self.temp(rt, n);
                self.next_moved = cap.by_value || uses;
                self.assign(&t, Rvalue::CaptureRef(src), n);
                ops.push(Operand::Use(t));
            }
        }
        self.closures.push(n);
        self.assign(dest, Rvalue::Aggregate(AggKind::Closure(c), ops), n);
        Ok(())
    }

    /// Whether the closure `n` uses the captured variable `local` only through
    /// fields (`p.x`): a capture of the whole variable stands for one of those
    /// fields, whose move leaves the rest of the variable as it was.
    fn only_fields_used(&self, n: NodeId, local: NodeId) -> bool {
        let ast = self.ast;
        let mut stack = vec![(n, n)];
        let mut found = false;
        while let Some((x, parent)) = stack.pop() {
            if ast.kind(x) == Kind::PathExpr
                && let Some(Res::Local(p)) = ast.children(x).first().and_then(|&path| self.res(path))
                && *p == local
            {
                found = true;
                if !(ast.kind(parent) == Kind::Field && ast.children(parent).first() == Some(&x)) {
                    return false;
                }
            }
            for &c in ast.children(x) {
                stack.push((c, x));
            }
        }
        found
    }

    // ---- patterns ----

    fn require(&mut self, cond: Operand, fail: Option<Block>, node: NodeId) {
        if let Some(f) = fail {
            let ok = self.new_block();
            self.branch(cond, ok, f, node);
            self.at(ok);
        }
    }

    /// `p == c`, held in a temporary.
    fn test_eq(&mut self, op: &str, p: &Place, c: Operand, node: NodeId) -> Operand {
        let t = self.temp(self.bool_ty(), node);
        self.assign(&t, Rvalue::Binary(op.to_string(), Operand::Use(p.clone()), c), node);
        Operand::Use(t)
    }

    fn test_variant(&mut self, p: &Place, v: u32, fail: Option<Block>, node: NodeId) {
        if let Some(f) = fail {
            let d = self.temp(self.usize_ty(), node);
            self.assign(&d, Rvalue::Discriminant(p.clone()), node);
            let ok = self.new_block();
            self.end(Term::Switch { discr: Operand::Use(d), targets: vec![(v as u128, ok)], otherwise: f }, node);
            self.at(ok);
        }
    }

    fn lit_const(&self, n: NodeId, t: Ty) -> Operand {
        let n = if self.ast.kind(n) == Kind::LitPat { self.ast.children(n)[0] } else { n };
        Operand::Const(self.text(n), t)
    }

    /// Matches `pat` against `place`: control goes to `fail` if it does not
    /// match; what it binds is noted (in `binds`) for `flush_binds` to do once
    /// the whole pattern has.
    fn match_pat(&mut self, pat: NodeId, place: Place, fail: Option<Block>) -> R<()> {
        let tcx = self.tcx();
        let kids = self.kids(pat);
        let mut p = place;
        for _ in 0..self.tables.pat_derefs.get(&pat).copied().unwrap_or(0) {
            p = self.deref(&p)?;
        }
        match self.ast.kind(pat) {
            Kind::WildPat | Kind::RestPat => Ok(()),
            // Nothing is there to match: this way is not taken.
            Kind::NeverPat => {
                self.end(Term::Unreachable, pat);
                Ok(())
            }
            Kind::ParenPat => self.match_pat(kids[0], p, fail),
            Kind::IdentPat => {
                if let Some(res) = self.res(pat) {
                    return self.path_test(pat, pat, res.clone(), p, fail);
                }
                if let Some(&sub) = kids.iter().find(|&&c| !matches!(self.ast.kind(c), Kind::Keyword | Kind::Name)) {
                    self.match_pat(sub, p.clone(), fail)?;
                }
                let mode = self.tables.bindings.get(&pat).copied().unwrap_or(BindingMode::Value);
                self.binds.push(Bind { pat, place: p, mode });
                Ok(())
            }
            Kind::RefPat | Kind::BoxPat => {
                let inner = self.deref(&p)?;
                self.match_pat(*kids.last().unwrap(), inner, fail)
            }
            Kind::BuiltinPat => match tcx.kind(p.ty) {
                TyKind::Ref(..) | TyKind::Ptr(..) => {
                    let inner = self.deref(&p)?;
                    self.match_pat(kids[1], inner, fail)
                }
                _ => Err("`deref!` through `Deref`".into()),
            },
            Kind::TuplePat => {
                let n = match tcx.kind(p.ty) {
                    TyKind::Tuple(l) => tcx.tys(l).len(),
                    _ => return Err("tuple pattern on something not a tuple".into()),
                };
                self.positional(&kids, &p, 0, n, fail)
            }
            Kind::TupleStructPat => {
                let v = self.pat_variant(kids[0], &p, fail, pat)?;
                let n = self.variant_fields(&p, v)?;
                self.positional(&kids[1..], &p, v, n, fail)
            }
            Kind::StructPat => {
                let v = self.pat_variant(kids[0], &p, fail, pat)?;
                let TyKind::Adt(d, _) = tcx.kind(p.ty) else { return Err("struct pattern on something not a struct".into()) };
                let names: Vec<&str> = tcx.adts[&d].variants[v as usize].fields.iter().map(|f| f.name).collect();
                for &f in &kids[1..] {
                    if self.ast.kind(f) != Kind::FieldPat {
                        continue;
                    }
                    let fk = self.kids(f);
                    let (name, sub) = if fk.len() == 2 {
                        (self.ast.token(fk[0]).0, fk[1])
                    } else {
                        let sub = fk[0];
                        (self.ast.child(sub, Kind::Name).map_or("", |nm| self.ast.token(nm).0), sub)
                    };
                    let i = names.iter().position(|&x| x == name).ok_or_else(|| format!("no field `{name}`"))?;
                    let fp = self.field(&p, v, i)?;
                    self.match_pat(sub, fp, fail)?;
                }
                Ok(())
            }
            Kind::PathPat => {
                let res = self.res(kids[0]).ok_or("unresolved path pattern")?.clone();
                self.path_test(pat, kids[0], res, p, fail)
            }
            Kind::LitPat => {
                let c = self.lit_const(pat, p.ty);
                let ok = self.test_eq("==", &p, c, pat);
                self.require(ok, fail, pat);
                Ok(())
            }
            Kind::RangePat => {
                let op_at = kids.iter().position(|&c| self.ast.kind(c) == Kind::Op).ok_or("range pattern without `..`")?;
                let inclusive = self.text(kids[op_at]) != "..";
                if let Some(&lo) = kids[..op_at].first() {
                    let c = self.lit_const(lo, p.ty);
                    let ok = self.test_eq(">=", &p, c, pat);
                    self.require(ok, fail, pat);
                }
                if let Some(&hi) = kids.get(op_at + 1) {
                    let c = self.lit_const(hi, p.ty);
                    let ok = self.test_eq(if inclusive { "<=" } else { "<" }, &p, c, pat);
                    self.require(ok, fail, pat);
                }
                Ok(())
            }
            Kind::OrPat => {
                // Each alternative binds the same names: the first's.
                let mut first: Vec<(String, NodeId)> = Vec::new();
                self.binding_names(kids[0], &mut first);
                for &alt in &kids[1..] {
                    let mut names = Vec::new();
                    self.binding_names(alt, &mut names);
                    for (name, node) in names {
                        if let Some((_, f)) = first.iter().find(|x| x.0 == name)
                            && *f != node
                        {
                            self.alias.insert(node, *f);
                        }
                    }
                }
                let matched = self.new_block();
                let mark = self.binds.len();
                for (i, &alt) in kids.iter().enumerate() {
                    let last = i + 1 == kids.len();
                    let next = if last { fail } else { Some(self.new_block()) };
                    self.match_pat(alt, p.clone(), next)?;
                    // This alternative's bindings, now it has matched.
                    self.flush_binds(mark, pat);
                    self.goto(matched, pat);
                    match next {
                        Some(nb) if !last => self.at(nb),
                        _ => break,
                    }
                }
                self.at(matched);
                Ok(())
            }
            Kind::SlicePat => self.slice_pat(&kids, &p, fail, pat),
            k => Err(format!("pattern {k:?}")),
        }
    }

    /// Whether a pattern has a `ref mut` binding (what it matches is then
    /// to be changed).
    fn binds_mut(&self, pat: NodeId) -> bool {
        if self.ast.kind(pat) == Kind::IdentPat && self.tables.bindings.get(&pat) == Some(&BindingMode::Ref(Mutbl::Mut)) {
            return true;
        }
        self.ast.children(pat).iter().any(|&c| (is_pattern(self.ast.kind(c)) || self.ast.kind(c) == Kind::FieldPat) && self.binds_mut(c))
    }

    fn binding_names(&self, pat: NodeId, out: &mut Vec<(String, NodeId)>) {
        if self.ast.kind(pat) == Kind::IdentPat
            && self.res(pat).is_none()
            && let Some(nm) = self.ast.child(pat, Kind::Name)
        {
            out.push((self.ast.token(nm).0.to_string(), pat));
        }
        for &c in self.ast.children(pat) {
            if is_pattern(self.ast.kind(c)) || self.ast.kind(c) == Kind::FieldPat {
                self.binding_names(c, out);
            }
        }
    }

    fn variant_fields(&self, p: &Place, v: u32) -> R<usize> {
        let tcx = self.tcx();
        match tcx.kind(p.ty) {
            TyKind::Adt(d, _) => Ok(tcx.adts.get(&d).and_then(|a| a.variants.get(v as usize)).map_or(0, |x| x.fields.len())),
            _ => Err("constructor pattern on something not a struct or enum".into()),
        }
    }

    fn pat_variant(&mut self, path: NodeId, p: &Place, fail: Option<Block>, node: NodeId) -> R<u32> {
        let tcx = self.tcx();
        let TyKind::Adt(d, _) = tcx.kind(p.ty) else { return Err("constructor pattern on something not an ADT".into()) };
        let v = self.variant_of_path(path, d).ok_or("variant not found")?;
        if tcx.adts[&d].kind == AdtKind::Enum {
            self.test_variant(p, v, fail, node);
        }
        Ok(v)
    }

    fn positional(&mut self, pats: &[NodeId], p: &Place, v: u32, n: usize, fail: Option<Block>) -> R<()> {
        let rest = pats.iter().position(|&c| self.ast.kind(c) == Kind::RestPat);
        for (i, &sp) in pats.iter().enumerate() {
            if Some(i) == rest {
                continue;
            }
            let fi = match rest {
                Some(r) if i > r => n - (pats.len() - i),
                _ => i,
            };
            let fp = self.field(p, v, fi)?;
            self.match_pat(sp, fp, fail)?;
        }
        Ok(())
    }

    fn path_test(&mut self, pat: NodeId, path: NodeId, res: Res, p: Place, fail: Option<Block>) -> R<()> {
        let tcx = self.tcx();
        let variant_or_struct = match &res {
            Res::Def(d) => matches!(self.cx.r.defs[d.0 as usize].kind, DefKind::Variant | DefKind::Struct),
            Res::Partial(..) | Res::SelfTy(..) => !self.tables.consts.contains_key(&pat),
            _ => false,
        };
        if variant_or_struct {
            if let TyKind::Adt(e, _) = tcx.kind(p.ty)
                && tcx.adts[&e].kind == AdtKind::Enum
            {
                let v = self.variant_of_path(path, e).ok_or("variant not found")?;
                self.test_variant(&p, v, fail, pat);
            }
            return Ok(());
        }
        // A constant: compared.
        let c = Operand::Const(self.text(if self.ast.kind(pat) == Kind::PathPat { path } else { pat }), p.ty);
        let ok = self.test_eq("==", &p, c, pat);
        self.require(ok, fail, pat);
        Ok(())
    }

    fn slice_pat(&mut self, kids: &[NodeId], p: &Place, fail: Option<Block>, node: NodeId) -> R<()> {
        let tcx = self.tcx();
        let is_rest = |c: NodeId| {
            self.ast.kind(c) == Kind::RestPat || self.ast.kind(c) == Kind::IdentPat && self.ast.children(c).iter().any(|&x| self.ast.kind(x) == Kind::RestPat)
        };
        let rest = kids.iter().position(|&c| is_rest(c));
        let elem = match tcx.kind(p.ty) {
            TyKind::Array(e, _) | TyKind::Slice(e) => e,
            _ => return Err("slice pattern on something not an array or slice".into()),
        };
        let fixed = kids.len() - rest.is_some() as usize;
        if matches!(tcx.kind(p.ty), TyKind::Slice(_)) && fail.is_some() {
            let len = self.temp(self.usize_ty(), node);
            self.assign(&len, Rvalue::Len(p.clone()), node);
            let op = if rest.is_some() { ">=" } else { "==" };
            let ok = self.temp(self.bool_ty(), node);
            self.assign(&ok, Rvalue::Binary(op.into(), Operand::Use(len), Operand::Const(fixed.to_string(), self.usize_ty())), node);
            self.require(Operand::Use(ok), fail, node);
        }
        for (i, &sp) in kids.iter().enumerate() {
            let ep = match rest {
                Some(r) if i == r => {
                    if self.ast.kind(sp) == Kind::IdentPat {
                        // `rest @ ..`: the middle.
                        let st = tcx.mk(TyKind::Slice(elem));
                        let mid = p.project(Proj::Subslice(i as u64, (kids.len() - i - 1) as u64, true, elem), st);
                        let mode = self.tables.bindings.get(&sp).copied().unwrap_or(BindingMode::Value);
                        self.binds.push(Bind { pat: sp, place: mid, mode });
                    }
                    continue;
                }
                Some(r) if i > r => p.project(Proj::ConstIndex((kids.len() - i) as u64, true, elem), elem),
                _ => p.project(Proj::ConstIndex(i as u64, false, elem), elem),
            };
            self.match_pat(sp, ep, fail)?;
        }
        Ok(())
    }
}

// ---- Place helpers ----

impl Place {
    /// `*self`, of pointee type `ty`.
    fn deref_to(self, ty: Ty) -> Place {
        self.project(Proj::Deref, ty)
    }
}

// ---- entry points ----

/// The body of item `owner` (a function, constant or static).
pub fn build_item<'a, 's>(cx: &'a Cx<'a, 's>, owner: DefId) -> (R<Cfg>, Vec<NodeId>) {
    let def = &cx.r.defs[owner.0 as usize];
    let mut b = Builder::new(cx, owner, def.node, cx.r.def_path(owner), None);
    let r = build_fn(&mut b, owner);
    let closures = std::mem::take(&mut b.closures);
    let node = b.cfg.node;
    (r.map(|()| b.finish(node)), closures)
}

fn build_fn(b: &mut Builder, owner: DefId) -> R<()> {
    let tcx = b.tcx();
    let item = b.cfg.node;
    let ast = b.ast;
    let kind = b.cx.r.defs[owner.0 as usize].kind;
    let fn_like = matches!(kind, DefKind::Fn | DefKind::AssocFn);
    // The result, then the parameters.
    let (ret_ty, inputs) = if fn_like {
        let sig = tcx.sig_of((*tcx.fn_sig.get(owner.0 as usize).ok_or("no signature")?).ok_or("no signature")?);
        (sig.output, tcx.tys(sig.inputs))
    } else {
        let e = ast.children(item).iter().rev().copied().find(|&c| !crate::ty::lower::is_type(ast.kind(c))).ok_or("a constant without a value")?;
        (b.ty(e)?, Vec::new())
    };
    b.new_local(ret_ty, LocalKind::Return, None, true, None);
    let entry = b.new_block();
    b.at(entry);
    b.push_scope(true);
    let mut pats: Vec<(NodeId, Local)> = Vec::new();
    if fn_like {
        let params = ast.child(item, Kind::Params).ok_or("a function without parameters")?;
        let mut ps = Vec::new();
        for &p in ast.children(params) {
            match ast.kind(p) {
                Kind::SelfParam | Kind::Param => ps.push(p),
                _ => {}
            }
        }
        if ps.len() != inputs.len() {
            return Err("parameters and signature differ".into());
        }
        for (&p, &t) in ps.iter().zip(&inputs) {
            let (pat, name, mutable) = if ast.kind(p) == Kind::SelfParam {
                (None, Some("self".to_string()), ast.children(p).iter().any(|&c| ast.kind(c) == Kind::Keyword && ast.token(c).0 == "mut"))
            } else {
                let pat = ast.children(p).iter().copied().find(|&c| is_pattern(ast.kind(c))).ok_or("a parameter without a pattern")?;
                let simple = ast.kind(pat) == Kind::IdentPat && b.res(pat).is_none();
                let name = if simple { ast.child(pat, Kind::Name).map(|nm| ast.token(nm).0.to_string()) } else { None };
                // (A parameter that is not just a name is a `mut` local, as
                // in rustc: its pattern may borrow it mutably.)
                let mutable = !b.is_simple_binding(pat) || ast.children(pat).iter().any(|&c| ast.kind(c) == Kind::Keyword && ast.token(c).0 == "mut");
                (Some(pat), name, mutable)
            };
            let l = b.new_local(t, LocalKind::Arg, name, mutable, Some(p));
            b.cfg.arg_count += 1;
            b.locals.insert(p, l);
            if let Some(pat) = pat {
                if b.is_simple_binding(pat) {
                    b.locals.insert(pat, l);
                } else {
                    pats.push((pat, l));
                }
            }
            b.register(l, true);
        }
    }
    for (pat, l) in pats {
        let place = Place::local(l, b.cfg.locals[l.0 as usize].ty);
        let mark = b.binds.len();
        b.match_pat(pat, place, None)?;
        b.flush_binds(mark, pat);
    }
    let body = if fn_like {
        ast.child(item, Kind::Block).ok_or("a function without a body")?
    } else {
        ast.children(item).iter().rev().copied().find(|&c| !crate::ty::lower::is_type(ast.kind(c))).ok_or("a constant without a value")?
    };
    let ret = Place::local(Local(0), ret_ty);
    b.expr_into(body, &ret)?;
    b.exit_to(0, body);
    b.end(Term::Return, body);
    Ok(())
}

/// The body of closure expression `node`, written in `owner`'s body.
pub fn build_closure<'a, 's>(cx: &'a Cx<'a, 's>, owner: DefId, node: NodeId) -> (R<Cfg>, Vec<NodeId>) {
    let name = format!("{}::{{closure@{}}}", cx.r.def_path(owner), node.0);
    let mut b = Builder::new(cx, owner, node, name, None);
    let r = build_closure_body(&mut b, owner, node);
    let closures = std::mem::take(&mut b.closures);
    (r.map(|()| b.finish(node)), closures)
}

fn build_closure_body(b: &mut Builder, owner: DefId, node: NodeId) -> R<()> {
    let _ = owner;
    let tcx = b.tcx();
    let ast = b.ast;
    let t = b.ty(node)?;
    let TyKind::Closure(c, sig, _) = tcx.kind(t) else { return Err("closure of no closure type".into()) };
    b.cfg.closure = Some(c);
    let sig = tcx.sig_of(sig);
    b.new_local(sig.output, LocalKind::Return, None, true, None);
    let entry = b.new_block();
    b.at(entry);
    b.push_scope(true);
    // The environment: what the closure captured, as fields.
    let env = b.new_local(t, LocalKind::Arg, Some("env".to_string()), true, None);
    b.cfg.arg_count += 1;
    b.env = Some(env);
    for (i, cap) in b.cx.res.captures.get(&c).cloned().unwrap_or_default().iter().enumerate() {
        let ty = b.ty(cap.local)?;
        b.upvars.insert(cap.local, (i, cap.by_value, ty));
        // Declared `mut`: a `mut` among the binding's own tokens.
        let mutable = ast.children(cap.local).iter().any(|&c| ast.kind(c) == Kind::Keyword && ast.token(c).0 == "mut");
        b.cfg.upvars.push((mutable, cap.by_value, ty));
    }
    let kids = b.kids(node);
    let params = kids.iter().copied().find(|&c| ast.kind(c) == Kind::ClosureParams);
    let inputs = tcx.tys(sig.inputs);
    let mut pats = Vec::new();
    let nodes: Vec<NodeId> = params.map(|p| ast.children(p).to_vec()).unwrap_or_default();
    if nodes.len() != inputs.len() {
        return Err("closure parameters and signature differ".into());
    }
    for (&cp, &it) in nodes.iter().zip(&inputs) {
        let pat = ast.children(cp).iter().copied().find(|&x| is_pattern(ast.kind(x))).ok_or("a closure parameter without a pattern")?;
        let simple = ast.kind(pat) == Kind::IdentPat && b.res(pat).is_none();
        let name = if simple { ast.child(pat, Kind::Name).map(|nm| ast.token(nm).0.to_string()) } else { None };
        let mutable = !b.is_simple_binding(pat) || ast.children(pat).iter().any(|&c| ast.kind(c) == Kind::Keyword && ast.token(c).0 == "mut");
        let l = b.new_local(it, LocalKind::Arg, name, mutable, Some(cp));
        b.cfg.arg_count += 1;
        b.register(l, true);
        if b.is_simple_binding(pat) {
            b.locals.insert(pat, l);
        } else {
            pats.push((pat, l));
        }
    }
    for (pat, l) in pats {
        let place = Place::local(l, b.cfg.locals[l.0 as usize].ty);
        let mark = b.binds.len();
        b.match_pat(pat, place, None)?;
        b.flush_binds(mark, pat);
    }
    let body = *kids.last().ok_or("a closure without a body")?;
    let ret = Place::local(Local(0), sig.output);
    b.expr_into(body, &ret)?;
    b.exit_to(0, body);
    b.end(Term::Return, body);
    Ok(())
}
