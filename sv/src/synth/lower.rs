//! An elaborated design to a [`Netlist`]: the hierarchy flattened, every
//! value a vector of literals, least significant bit first.
//!
//! Each object of each instance has a wire for each bit, driven by what
//! assigns it: a continuous assignment or port connection, the end of an
//! `always_comb`, or a flip-flop. Procedures are run symbolically: an
//! environment holds, for each variable a procedure has assigned, each
//! bit's value and the condition under which it was assigned (its
//! enable). An `if` runs both arms and merges them under the condition;
//! a `case` is a chain of them; a `for` with constant bounds is unrolled.
//! At the end of an `always_comb` every bit it assigns must have been
//! assigned on every path (else it would be a latch); in an
//! `always_ff` the enable is the flip-flop's.

use super::{F, Ff, Lit, Netlist, Node, Port, T, node};
use crate::ast::{Ast, NodeId};
use crate::design::types::Type;
use crate::design::value::{Bits, Logic};
use crate::design::*;
use crate::hash::FxMap;
use crate::source::Error;

type Key = (u32, ObjId);
type R<'a, T> = Result<T, Error<'a>>;

#[derive(Clone)]
struct Slot {
    val: Vec<Lit>,
    en: Vec<Lit>,
}

/// What a procedure has assigned so far: by blocking assignments (which
/// later reads see), and by nonblocking ones (which they do not).
#[derive(Clone, Default)]
struct Env {
    blk: FxMap<Key, Slot>,
    nba: FxMap<Key, Slot>,
}

struct Inst {
    path: String,
}

pub struct Lower<'d, 'a> {
    d: &'d Design<'a>,
    ast: &'d Ast<'a>,
    pub n: Netlist,
    insts: Vec<Inst>,
    wires: FxMap<Key, Vec<Lit>>,
    inits: FxMap<Key, Vec<Lit>>,
}

/// Lowers the design under top-level instance `top`.
pub fn lower<'a>(d: &Design<'a>, ast: &Ast<'a>, top: InstId) -> Result<Netlist, Error<'a>> {
    let mut l = Lower { d, ast, n: Netlist::new(), insts: Vec::new(), wires: FxMap::default(), inits: FxMap::default() };
    let i = d.instance(top);
    let spec = d.spec(i.spec);
    l.insts.push(Inst { path: d.name(i.name).to_string() });
    let mut outs = Vec::new();
    for p in &spec.ports {
        let Some(o) = p.object else { return Err(l.err(p.node, "an interface port on the top module is not synthesised")) };
        let w = d.types.width(d.object(o).ty);
        match p.dir {
            Dir::Input => {
                let port = l.n.inputs.len() as u32;
                let bits = (0..w).map(|b| l.n.input(port, b)).collect();
                l.n.inputs.push(Port { name: d.name(p.name).to_string(), width: w, bits: Vec::new() });
                l.wires.insert((0, o), bits);
            }
            Dir::Output => outs.push((p.name, o, w)),
            _ => return Err(l.err(p.node, "only input and output ports are synthesised")),
        }
    }
    l.scope(0, spec.scope)?;
    for (name, o, w) in outs {
        let bits = l.obj_bits(0, o);
        l.n.outputs.push(Port { name: d.name(name).to_string(), width: w, bits });
    }
    Ok(std::mem::take(&mut l.n))
}

fn bits_of(c: &Const, w: u32) -> Option<Vec<Lit>> {
    let b = match c {
        Const::Bits(b) => b.clone(),
        Const::Str(s) => Bits::from_str_bytes(s),
        _ => return None,
    };
    let b = b.resize(w, b.signed());
    Some((0..w).map(|i| if b.bit(i) == Logic::One { T } else { F }).collect())
}

fn konst(v: &[Lit]) -> Option<i64> {
    if v.len() > 63 && v[63..].iter().any(|&b| b != F) {
        return None;
    }
    v.iter().enumerate().try_fold(0i64, |acc, (i, &b)| match b {
        F => Some(acc),
        T if i < 63 => Some(acc | 1 << i),
        _ => None,
    })
}

impl<'d, 'a> Lower<'d, 'a> {
    fn err(&self, n: NodeId, msg: impl Into<String>) -> Error<'a> {
        let t = self.ast.token(n);
        let at = if t.0.is_empty() {
            let i = self.ast.node(n).first as usize;
            self.ast.tokens.get(i).or(self.ast.tokens.last()).map_or("", |t| t.0)
        } else {
            t.0
        };
        Error::new(at, msg)
    }

    fn tys(&self) -> &types::Types {
        &self.d.types
    }

    fn width(&self, e: ExprId) -> u32 {
        self.tys().width(self.d.expr(e).ty)
    }

    fn signed(&self, e: ExprId) -> bool {
        self.tys().is_signed(self.d.expr(e).ty)
    }

    fn obj_name(&self, inst: u32, o: ObjId) -> String {
        format!("{}.{}", self.insts[inst as usize].path, self.d.name(self.d.object(o).name))
    }

    /// The wires of object `o` in instance `inst`.
    fn obj_bits(&mut self, inst: u32, o: ObjId) -> Vec<Lit> {
        if let Some(v) = self.wires.get(&(inst, o)) {
            return v.clone();
        }
        let w = self.tys().width(self.d.object(o).ty);
        let name = self.obj_name(inst, o);
        let v: Vec<Lit> = (0..w).map(|b| self.n.wire(if w == 1 { name.clone() } else { format!("{name}[{b}]") })).collect();
        self.wires.insert((inst, o), v.clone());
        v
    }

    fn scope(&mut self, inst: u32, sc: ScopeId) -> R<'a, ()> {
        let items = self.d.scope(sc).items.clone();
        for it in items {
            match it {
                Item::Object(o) => {
                    let obj = self.d.object(o);
                    if let Some(init) = obj.init {
                        let v = self.eval(inst, init, &Env::default())?;
                        let v = self.fit(v, self.tys().width(obj.ty), self.signed(init));
                        if matches!(obj.kind, ObjKind::Net(_)) {
                            let w = self.obj_bits(inst, o);
                            for (a, b) in w.into_iter().zip(v) {
                                self.n.drive(a, b);
                            }
                        } else if obj.kind == ObjKind::Variable {
                            if v.iter().any(|&b| b != F && b != T) {
                                return Err(self.err(obj.node, "a variable's initial value must be a constant"));
                            }
                            self.inits.insert((inst, o), v);
                        }
                    }
                }
                Item::Driver(id) => {
                    let (lhs, rhs) = (self.d.drivers[id.0 as usize].lhs, self.d.drivers[id.0 as usize].rhs);
                    let v = self.eval(inst, rhs, &Env::default())?;
                    self.drive(inst, lhs, v)?;
                }
                Item::Process(p) => self.process(inst, p)?,
                Item::Instance(i) => self.instance(inst, i)?,
                Item::Scope(s) if self.d.scope(s).kind == ScopeKind::Generate => self.scope(inst, s)?,
                Item::Scope(_) => {}
                Item::Cell(c) => return Err(self.err(self.d.cells[c.0 as usize].node, "gates and UDPs are not synthesised yet")),
                Item::Typedef(..) | Item::Subroutine(_) | Item::Import(..) | Item::Class(_) | Item::Opaque(_) => {}
            }
        }
        Ok(())
    }

    fn instance(&mut self, parent: u32, i: InstId) -> R<'a, ()> {
        let ins = self.d.instance(i);
        let spec = self.d.spec(ins.spec);
        let child = self.insts.len() as u32;
        let mut path = format!("{}.{}", self.insts[parent as usize].path, self.d.name(ins.name));
        if let Some(k) = ins.index {
            path += &format!("[{k}]");
        }
        self.insts.push(Inst { path });
        for c in &ins.connections {
            let Some(port) = spec.ports.iter().find(|p| p.name == c.port) else { continue };
            let Some(o) = port.object else { return Err(self.err(ins.node, "interface ports are not synthesised")) };
            let Some(e) = c.expr else { continue };
            let w = self.tys().width(self.d.object(o).ty);
            match port.dir {
                Dir::Input => {
                    let v = self.eval(parent, e, &Env::default())?;
                    let v = self.fit(v, w, self.signed(e));
                    let bits = self.obj_bits(child, o);
                    for (a, b) in bits.into_iter().zip(v) {
                        if !self.n.drive(a, b) {
                            return Err(self.err(ins.node, format!("port {} has another driver", self.d.name(c.port))));
                        }
                    }
                }
                Dir::Output => {
                    let v = self.obj_bits(child, o);
                    let signed = self.tys().is_signed(self.d.object(o).ty);
                    let v = self.fit(v, self.width(e), signed);
                    self.drive(parent, e, v)?;
                }
                _ => return Err(self.err(port.node, "only input and output ports are synthesised")),
            }
        }
        self.scope(child, spec.scope)
    }

    /// Drives `lhs` (a continuous assignment's target) with `v`.
    fn drive(&mut self, inst: u32, lhs: ExprId, v: Vec<Lit>) -> R<'a, ()> {
        let targets = self.lvalue(inst, lhs, &Env::default())?;
        let v = self.fit(v, targets.len() as u32, false);
        for (t, b) in targets.into_iter().zip(v) {
            let [(key, bit, cond)] = t.as_slice() else {
                return Err(self.err(self.d.expr(lhs).node, "a continuous assignment to a select whose index is not constant"));
            };
            if *cond != T {
                return Err(self.err(self.d.expr(lhs).node, "a continuous assignment to a select whose index is not constant"));
            }
            let w = self.obj_bits(key.0, key.1)[*bit as usize];
            if !self.n.drive(w, b) {
                return Err(self.err(self.d.expr(lhs).node, format!("{} has more than one driver", self.obj_name(key.0, key.1))));
            }
        }
        Ok(())
    }

    fn process(&mut self, inst: u32, p: ProcId) -> R<'a, ()> {
        let pr = &self.d.processes[p.0 as usize];
        let (kind, body, pnode) = (pr.kind, pr.body, pr.node);
        let (timing, stmt) = match &self.d.stmt(body).kind {
            StmtKind::Timed(t, s) => (Some(t.clone()), *s),
            _ => (None, body),
        };
        match (kind, &timing) {
            (ProcKind::Final, _) => Ok(()),
            (ProcKind::Initial, _) => self.initial(inst, stmt, pnode),
            (ProcKind::AlwaysLatch, _) => Err(self.err(pnode, "latches are not synthesised")),
            (ProcKind::AlwaysComb, None) | (ProcKind::Always, Some(Timing::Implicit(_))) => self.comb(inst, stmt, pnode),
            (ProcKind::Always, Some(Timing::Event(terms))) if terms.iter().all(|t| t.edge == Edge::Any) => self.comb(inst, stmt, pnode),
            (ProcKind::AlwaysFf | ProcKind::Always, Some(Timing::Event(terms))) => self.seq(inst, terms.clone(), stmt, pnode),
            _ => Err(self.err(pnode, "this kind of process is not synthesised")),
        }
    }

    fn initial(&mut self, inst: u32, s: StmtId, at: NodeId) -> R<'a, ()> {
        let mut env = Env::default();
        self.stmt(inst, s, &mut env)?;
        for (key, slot) in env.blk.into_iter().chain(env.nba) {
            if slot.val.iter().chain(&slot.en).any(|&b| b != F && b != T) {
                return Err(self.err(at, "an initial block may only set variables to constants"));
            }
            let mut v = self.inits.get(&key).cloned().unwrap_or_else(|| vec![F; slot.val.len()]);
            for (i, (&x, &e)) in slot.val.iter().zip(&slot.en).enumerate() {
                if e == T {
                    v[i] = x;
                }
            }
            self.inits.insert(key, v);
        }
        Ok(())
    }

    fn is_static(&self, o: ObjId) -> bool {
        self.d.object(o).lifetime == Lifetime::Static
    }

    fn comb(&mut self, inst: u32, s: StmtId, at: NodeId) -> R<'a, ()> {
        let mut env = Env::default();
        self.stmt(inst, s, &mut env)?;
        if !env.nba.is_empty() {
            return Err(self.err(at, "a nonblocking assignment in combinational logic is not synthesised"));
        }
        let mut keys: Vec<_> = env.blk.keys().copied().collect();
        keys.sort();
        for key in keys {
            if !self.is_static(key.1) {
                continue;
            }
            let slot = &env.blk[&key];
            let wires = self.obj_bits(key.0, key.1);
            for (b, (&v, &e)) in slot.val.iter().zip(&slot.en).enumerate() {
                if e == F {
                    continue;
                }
                if e != T {
                    return Err(self.err(at, format!("{} is not assigned on every path: it would be a latch", self.obj_name(key.0, key.1))));
                }
                if !self.n.drive(wires[b], v) {
                    return Err(self.err(at, format!("{} has more than one driver", self.obj_name(key.0, key.1))));
                }
            }
        }
        Ok(())
    }

    /// A clocked process: one edge of a clock, and optionally an
    /// asynchronous reset whose arm comes first.
    fn seq(&mut self, inst: u32, terms: Vec<EventTerm>, s: StmtId, at: NodeId) -> R<'a, ()> {
        let none = Env::default();
        let mut sig = Vec::new();
        for t in &terms {
            if t.edge != Edge::Pos && t.edge != Edge::Neg || t.iff.is_some() {
                return Err(self.err(at, "a clocked process must wait on posedge or negedge alone"));
            }
            let v = self.eval(inst, t.expr, &none)?;
            sig.push(v[0]);
        }
        let (clock, negedge, reset) = match terms.len() {
            1 => (sig[0], terms[0].edge == Edge::Neg, None),
            2 => {
                // The reset is the event the body's first `if` tests.
                let mut st = s;
                while let StmtKind::Block { stmts, join: None, .. } = &self.d.stmt(st).kind {
                    match stmts.as_slice() {
                        [one] => st = *one,
                        _ => break,
                    }
                }
                let StmtKind::If { cond, then, else_, .. } = self.d.stmt(st).kind.clone() else {
                    return Err(self.err(at, "with two events, the body must be `if (reset) ... else ...`"));
                };
                let c = self.eval(inst, cond, &none)?;
                let c = self.truth(c);
                let r = (0..2).find(|&k| c == if terms[k].edge == Edge::Pos { sig[k] } else { sig[k] ^ 1 });
                let Some(r) = r else { return Err(self.err(self.d.expr(cond).node, "this condition is not the reset event's signal at its active level")) };
                let k = 1 - r;
                (sig[k], terms[k].edge == Edge::Neg, Some((c, then, else_)))
            }
            _ => return Err(self.err(at, "a clocked process may wait on a clock and one asynchronous reset")),
        };
        let mut env = Env::default();
        let mut rst = FxMap::default();
        let arst = match reset {
            None => {
                self.stmt(inst, s, &mut env)?;
                F
            }
            Some((c, then, else_)) => {
                let mut renv = Env::default();
                self.stmt(inst, then, &mut renv)?;
                for (key, slot) in renv.nba {
                    if slot.val.iter().chain(&slot.en).any(|&b| b != F && b != T) {
                        return Err(self.err(self.d.stmt(then).node, "an asynchronous reset must set constants"));
                    }
                    rst.insert(key, slot);
                }
                if let Some(e) = else_ {
                    self.stmt(inst, e, &mut env)?;
                }
                c
            }
        };
        let mut keys: Vec<Key> = env.nba.keys().chain(rst.keys()).copied().collect();
        keys.sort();
        keys.dedup();
        for key in keys {
            if !self.is_static(key.1) {
                continue;
            }
            let wires = self.obj_bits(key.0, key.1);
            let w = wires.len();
            let none = Slot { val: vec![F; w], en: vec![F; w] };
            let slot = env.nba.get(&key).unwrap_or(&none).clone();
            let r = rst.get(&key).unwrap_or(&none).clone();
            let init = self.inits.get(&key).cloned();
            for b in 0..w {
                if slot.en[b] == F && r.en[b] == F {
                    continue;
                }
                if arst != F && r.en[b] != T {
                    return Err(self.err(at, format!("{} is assigned by the clock but not by the reset", self.obj_name(key.0, key.1))));
                }
                let name = if w == 1 { self.obj_name(key.0, key.1) } else { format!("{}[{b}]", self.obj_name(key.0, key.1)) };
                let q = self.n.ff(Ff {
                    d: slot.val[b],
                    en: slot.en[b],
                    arst,
                    arst_value: r.val[b] == T,
                    init: init.as_ref().is_some_and(|v| v[b] == T),
                    clock,
                    negedge,
                    name,
                });
                if !self.n.drive(wires[b], q) {
                    return Err(self.err(at, format!("{} has more than one driver", self.obj_name(key.0, key.1))));
                }
            }
        }
        // Blocking assignments in a clocked process are its own
        // temporaries.
        Ok(())
    }

    // ---- statements ----

    fn stmt(&mut self, inst: u32, s: StmtId, env: &mut Env) -> R<'a, ()> {
        let st = self.d.stmt(s);
        let at = st.node;
        match st.kind.clone() {
            StmtKind::Null => Ok(()),
            StmtKind::Block { join, scope, stmts } => {
                if join.is_some() {
                    return Err(self.err(at, "fork is not synthesised"));
                }
                if let Some(sc) = scope {
                    self.locals(inst, sc, env)?;
                }
                for x in stmts {
                    self.stmt(inst, x, env)?;
                }
                Ok(())
            }
            StmtKind::Assign { kind, lhs, rhs, timing } => {
                if timing.is_some() {
                    return Err(self.err(at, "an intra-assignment delay is not synthesised"));
                }
                let mut v = self.eval(inst, rhs, env)?;
                if let AssignKind::Compound(op) = kind {
                    let old = self.eval(inst, lhs, env)?;
                    let w = old.len() as u32;
                    let v2 = self.fit(v, w, self.signed(rhs));
                    v = self.binop(op, old, v2, self.signed(lhs), at)?;
                }
                self.assign(inst, lhs, v, kind == AssignKind::NonBlocking, env)
            }
            StmtKind::Expr(e) => self.effect(inst, e, env),
            StmtKind::If { cond, then, else_, .. } => {
                let c = self.eval(inst, cond, env)?;
                let c = self.truth(c);
                if c == T {
                    return self.stmt(inst, then, env);
                }
                if c == F {
                    return else_.map_or(Ok(()), |e| self.stmt(inst, e, env));
                }
                let mut te = env.clone();
                self.stmt(inst, then, &mut te)?;
                if let Some(e) = else_ {
                    self.stmt(inst, e, env)?;
                }
                *env = self.merge(c, te, std::mem::take(env));
                Ok(())
            }
            StmtKind::Case { kind, expr, items, default, .. } => {
                let v = self.eval(inst, expr, env)?;
                if let Some(e) = self.case_tree(inst, kind, &v, &items, default, env)? {
                    *env = e;
                    return Ok(());
                }
                let mut arms = Vec::new();
                for (labels, body) in &items {
                    let mut c = F;
                    for &l in labels {
                        let m = self.case_match(inst, kind, &v, expr, l, env)?;
                        c = self.n.or(c, m);
                    }
                    arms.push((c, *body));
                }
                // Priority: the first arm that matches.
                let base = env.clone();
                let mut out = base.clone();
                if let Some(d) = default {
                    self.stmt(inst, d, &mut out)?;
                }
                for (c, body) in arms.into_iter().rev() {
                    if c == F {
                        continue;
                    }
                    let mut e = base.clone();
                    self.stmt(inst, body, &mut e)?;
                    out = self.merge(c, e, out);
                }
                *env = out;
                Ok(())
            }
            StmtKind::For { scope, init, cond, step, body } => {
                if let Some(sc) = scope {
                    self.locals(inst, sc, env)?;
                }
                for x in init {
                    self.stmt(inst, x, env)?;
                }
                for _ in 0..=4096 {
                    if let Some(c) = cond {
                        let v = self.eval(inst, c, env)?;
                        match self.truth(v) {
                            F => return Ok(()),
                            T => {}
                            _ => return Err(self.err(self.d.expr(c).node, "a loop's condition must be constant when unrolled")),
                        }
                    }
                    self.stmt(inst, body, env)?;
                    for &x in &step {
                        self.stmt(inst, x, env)?;
                    }
                }
                Err(self.err(at, "a loop of more than 4096 iterations is not unrolled"))
            }
            StmtKind::Repeat(n, body) => {
                let v = self.eval(inst, n, env)?;
                let Some(k) = konst(&v) else { return Err(self.err(at, "a repeat count must be constant")) };
                for _ in 0..k.clamp(0, 4096) {
                    self.stmt(inst, body, env)?;
                }
                Ok(())
            }
            StmtKind::Assert { .. } => Ok(()),
            _ => Err(self.err(at, "this statement is not synthesised")),
        }
    }

    /// A `case` on a narrow value whose labels are all constants, as a
    /// decision tree on the value's bits: for each value, the arm that
    /// runs, the arms' results chosen by muxes from the most significant
    /// bit down (equal subtrees are one by hashing). `None` where the case
    /// is not of that kind.
    fn case_tree(&mut self, inst: u32, kind: CaseKind, v: &[Lit], items: &[(Vec<ExprId>, StmtId)], default: Option<StmtId>, env: &Env) -> R<'a, Option<Env>> {
        let w = v.len();
        if kind != CaseKind::Case || w > 8 {
            return Ok(None);
        }
        let mut arm_of: Vec<Option<usize>> = vec![None; 1 << w];
        for (a, (labels, _)) in items.iter().enumerate() {
            for &l in labels {
                if matches!(self.d.expr(l).kind, ExprKind::Range) {
                    return Ok(None);
                }
                let lv = self.eval(inst, l, env)?;
                let Some(k) = konst(&lv) else { return Ok(None) };
                if k < 0 || k >= 1 << w || self.signed(l) && lv.last() == Some(&T) {
                    // Never matches (or a sign this does not model).
                    if self.signed(l) {
                        return Ok(None);
                    }
                    continue;
                }
                let slot = &mut arm_of[k as usize];
                if slot.is_none() {
                    *slot = Some(a);
                }
            }
        }
        let mut envs: Vec<Env> = Vec::new();
        for (_, body) in items {
            let mut e = env.clone();
            self.stmt(inst, *body, &mut e)?;
            envs.push(e);
        }
        let mut other = env.clone();
        if let Some(d) = default {
            self.stmt(inst, d, &mut other)?;
        }
        envs.push(other);
        let leaf: Vec<usize> = arm_of.iter().map(|a| a.unwrap_or(items.len())).collect();
        let mut out = Env::default();
        for nba in [false, true] {
            let mut keys: Vec<Key> = envs.iter().flat_map(|e| if nba { e.nba.keys() } else { e.blk.keys() }).copied().collect();
            keys.sort();
            keys.dedup();
            for k in keys {
                let width = envs.iter().find_map(|e| if nba { e.nba.get(&k) } else { e.blk.get(&k) }).unwrap().val.len();
                let mut slot = Slot { val: Vec::with_capacity(width), en: Vec::with_capacity(width) };
                for b in 0..width {
                    let leaves: Vec<(Lit, Lit)> = envs
                        .iter()
                        .map(|e| (if nba { e.nba.get(&k) } else { e.blk.get(&k) }).map_or((F, F), |s| (s.val[b], s.en[b])))
                        .collect();
                    let (val, en) = self.tree(v, w, 0, &leaf, &leaves);
                    slot.val.push(val);
                    slot.en.push(en);
                }
                if nba { out.nba.insert(k, slot) } else { out.blk.insert(k, slot) };
            }
        }
        Ok(Some(out))
    }

    /// The value and enable for selector values `base..base + 2^bits`.
    fn tree(&mut self, v: &[Lit], bits: usize, base: usize, leaf: &[usize], leaves: &[(Lit, Lit)]) -> (Lit, Lit) {
        if bits == 0 {
            return leaves[leaf[base]];
        }
        let s = v[bits - 1];
        let (lv, le) = self.tree(v, bits - 1, base, leaf, leaves);
        let (hv, he) = self.tree(v, bits - 1, base + (1 << (bits - 1)), leaf, leaves);
        let en = self.n.mux(s, he, le);
        let val = if he == F {
            lv
        } else if le == F {
            hv
        } else {
            self.n.mux(s, hv, lv)
        };
        (val, en)
    }

    /// A block's or loop's own variables, set to their initial values.
    fn locals(&mut self, inst: u32, sc: ScopeId, env: &mut Env) -> R<'a, ()> {
        for it in self.d.scope(sc).items.clone() {
            if let Item::Object(o) = it {
                let obj = self.d.object(o);
                let w = self.tys().width(obj.ty) as usize;
                let val = match obj.init {
                    Some(init) => {
                        let v = self.eval(inst, init, env)?;
                        self.fit(v, w as u32, self.signed(init))
                    }
                    None => vec![F; w],
                };
                env.blk.insert((inst, o), Slot { val, en: vec![T; w] });
            }
        }
        Ok(())
    }

    /// An expression statement: an increment, a call for its effect, an
    /// assignment used as a statement.
    fn effect(&mut self, inst: u32, e: ExprId, env: &mut Env) -> R<'a, ()> {
        let ex = self.d.expr(e).clone();
        match ex.kind {
            ExprKind::Unary(op @ (UnOp::PreInc | UnOp::PostInc | UnOp::PreDec | UnOp::PostDec)) => {
                let old = self.eval(inst, ex.args[0], env)?;
                let one: Vec<Lit> = (0..old.len()).map(|i| if i == 0 { T } else { F }).collect();
                let bop = if matches!(op, UnOp::PreInc | UnOp::PostInc) { BinOp::Add } else { BinOp::Sub };
                let v = self.binop(bop, old, one, false, ex.node)?;
                self.assign(inst, ex.args[0], v, false, env)
            }
            ExprKind::Assign => {
                let v = self.eval(inst, ex.args[1], env)?;
                self.assign(inst, ex.args[0], v, false, env)
            }
            // $display and the like do nothing in hardware.
            ExprKind::SysCall(_) => Ok(()),
            _ => Err(self.err(ex.node, "this expression statement is not synthesised")),
        }
    }

    fn case_match(&mut self, inst: u32, kind: CaseKind, v: &[Lit], expr: ExprId, label: ExprId, env: &Env) -> R<'a, Lit> {
        let le = self.d.expr(label);
        if kind == CaseKind::Inside || matches!(le.kind, ExprKind::Range) {
            return self.inside(inst, v, self.signed(expr), label, env);
        }
        // casez / casex: a constant label's z (and for casex, x) bits
        // match anything.
        let mut care: Option<Vec<bool>> = None;
        if matches!(kind, CaseKind::CaseZ | CaseKind::CaseX) {
            if let ExprKind::Const(c) = le.kind {
                if let Some(b) = self.d.konst(c).as_bits() {
                    let w = v.len() as u32;
                    let b = b.resize(w, b.signed());
                    care = Some((0..w).map(|i| !(b.bit(i) == Logic::Z || kind == CaseKind::CaseX && b.bit(i) == Logic::X)).collect());
                }
            }
        }
        let l = self.eval(inst, label, env)?;
        let w = v.len().max(l.len()) as u32;
        let a = self.fit(v.to_vec(), w, self.signed(expr));
        let b = self.fit(l, w, self.signed(label));
        let mut eq = T;
        for i in 0..w as usize {
            if care.as_ref().is_some_and(|c| !c.get(i).copied().unwrap_or(true)) {
                continue;
            }
            let x = self.n.xor(a[i], b[i]);
            eq = self.n.and(eq, x ^ 1);
        }
        Ok(eq)
    }

    fn inside(&mut self, inst: u32, v: &[Lit], signed: bool, item: ExprId, env: &Env) -> R<'a, Lit> {
        let ie = self.d.expr(item).clone();
        if matches!(ie.kind, ExprKind::Range) {
            let lo = self.eval(inst, ie.args[0], env)?;
            let hi = self.eval(inst, ie.args[1], env)?;
            let w = v.len().max(lo.len()).max(hi.len()) as u32;
            let (a, lo, hi) = (self.fit(v.to_vec(), w, signed), self.fit(lo, w, signed), self.fit(hi, w, signed));
            let below = self.less(&a, &lo, signed);
            let above = self.less(&hi, &a, signed);
            return Ok(self.n.and(below ^ 1, above ^ 1));
        }
        let l = self.eval(inst, item, env)?;
        let w = v.len().max(l.len()) as u32;
        let (a, b) = (self.fit(v.to_vec(), w, signed), self.fit(l, w, self.signed(item)));
        Ok(self.equal(&a, &b))
    }

    /// Merges the two arms of `if (c)`.
    fn merge(&mut self, c: Lit, mut t: Env, mut e: Env) -> Env {
        let mut out = Env::default();
        for nba in [false, true] {
            let (tm, em) = if nba { (std::mem::take(&mut t.nba), std::mem::take(&mut e.nba)) } else { (std::mem::take(&mut t.blk), std::mem::take(&mut e.blk)) };
            let mut keys: Vec<Key> = tm.keys().chain(em.keys()).copied().collect();
            keys.sort();
            keys.dedup();
            for k in keys {
                let w = tm.get(&k).or(em.get(&k)).unwrap().val.len();
                let none = Slot { val: vec![F; w], en: vec![F; w] };
                let (a, b) = (tm.get(&k).unwrap_or(&none), em.get(&k).unwrap_or(&none));
                let mut s = Slot { val: Vec::with_capacity(w), en: Vec::with_capacity(w) };
                for i in 0..w {
                    s.en.push(self.n.mux(c, a.en[i], b.en[i]));
                    s.val.push(if a.en[i] == F {
                        b.val[i]
                    } else if b.en[i] == F {
                        a.val[i]
                    } else {
                        self.n.mux(c, a.val[i], b.val[i])
                    });
                }
                if nba { out.nba.insert(k, s) } else { out.blk.insert(k, s) };
            }
        }
        out
    }

    /// Assigns `v` to the target `lhs` in a procedure.
    fn assign(&mut self, inst: u32, lhs: ExprId, v: Vec<Lit>, nba: bool, env: &mut Env) -> R<'a, ()> {
        let targets = self.lvalue(inst, lhs, env)?;
        let v = self.fit(v, targets.len() as u32, false);
        for (t, b) in targets.into_iter().zip(v) {
            for (key, bit, cond) in t {
                if !self.is_static(key.1) && nba {
                    return Err(self.err(self.d.expr(lhs).node, "a nonblocking assignment to an automatic variable"));
                }
                let w = self.tys().width(self.d.object(key.1).ty) as usize;
                let map = if nba { &mut env.nba } else { &mut env.blk };
                let slot = map.entry(key).or_insert_with(|| Slot { val: vec![F; w], en: vec![F; w] });
                let (old_v, old_e) = (slot.val[bit as usize], slot.en[bit as usize]);
                let nv = if old_e == F { b } else { self.n.mux(cond, b, old_v) };
                let ne = self.n.or(cond, old_e);
                let map = if nba { &mut env.nba } else { &mut env.blk };
                let slot = map.get_mut(&key).unwrap();
                slot.val[bit as usize] = nv;
                slot.en[bit as usize] = ne;
            }
        }
        Ok(())
    }

    /// For each bit of target `e`, least significant first, the bits of
    /// objects it may be, each under a condition (more than one where an
    /// index is not constant).
    fn lvalue(&mut self, inst: u32, e: ExprId, env: &Env) -> R<'a, Vec<Vec<(Key, u32, Lit)>>> {
        let ex = self.d.expr(e).clone();
        match ex.kind {
            ExprKind::Object(o) => {
                let w = self.tys().width(self.d.object(o).ty);
                Ok((0..w).map(|b| vec![((inst, o), b, T)]).collect())
            }
            ExprKind::Concat => {
                let mut out = Vec::new();
                for &a in ex.args.iter().rev() {
                    out.extend(self.lvalue(inst, a, env)?);
                }
                Ok(out)
            }
            ExprKind::Convert => self.lvalue(inst, ex.args[0], env),
            ExprKind::Index | ExprKind::Part { .. } | ExprKind::Member(_) => {
                let base = self.lvalue(inst, ex.args[0], env)?;
                let sel = self.select(inst, &ex, env)?;
                let w = self.width(e) as usize;
                let mut out = vec![Vec::new(); w];
                for (off, cond) in sel {
                    for (i, slot) in out.iter_mut().enumerate() {
                        let k = off + i as i64;
                        if k >= 0 && (k as usize) < base.len() {
                            for &(key, bit, c) in &base[k as usize] {
                                let c = self.n.and(c, cond);
                                if c != F {
                                    slot.push((key, bit, c));
                                }
                            }
                        }
                    }
                }
                Ok(out)
            }
            _ => Err(self.err(ex.node, "this kind of assignment target is not synthesised")),
        }
    }

    /// Where a select starts in its base, in bits from the least
    /// significant end: one offset if its index is constant, else each
    /// possible one with the condition that picks it.
    fn select(&mut self, inst: u32, ex: &Expr, env: &Env) -> R<'a, Vec<(i64, Lit)>> {
        let bt = self.d.expr(ex.args[0]).ty;
        if self.tys().is_unpacked_array(bt) {
            return Err(self.err(ex.node, "memories (unpacked arrays) are not synthesised yet"));
        }
        let ew = self.tys().elem_width(bt) as i64;
        let (l, r) = self.tys().packed_range(bt).unwrap_or((self.tys().width(bt) as i64 - 1, 0));
        match ex.kind {
            ExprKind::Member(m) => {
                let Type::Struct { packed: true, union, members, .. } = self.tys().get(bt).clone() else {
                    return Err(self.err(ex.node, "a member of an unpacked struct is not synthesised"));
                };
                if union {
                    return Ok(vec![(0, T)]);
                }
                // Members are listed most significant first.
                let off: u32 = members[m as usize + 1..].iter().map(|x| self.tys().width(x.ty)).sum();
                Ok(vec![(off as i64, T)])
            }
            ExprKind::Part { kind: PartKind::Range, .. } => {
                let i = self.eval(inst, ex.args[1], env)?;
                let j = self.eval(inst, ex.args[2], env)?;
                let (Some(i), Some(j)) = (konst(&i), konst(&j)) else { return Err(self.err(ex.node, "a part select's bounds must be constant")) };
                let (off, _) = ops::select_span(self.tys(), bt, i, j);
                Ok(vec![(off * ew, T)])
            }
            _ => {
                let width = match ex.kind {
                    ExprKind::Part { width, .. } => width as i64,
                    _ => 1,
                };
                let down = matches!(ex.kind, ExprKind::Part { kind: PartKind::Down, .. });
                let idx = self.eval(inst, ex.args[1], env)?;
                let d = self.d;
                let span = |k: i64| {
                    let (lo, hi) = if down { (k - width + 1, k) } else { (k, k + width - 1) };
                    ops::select_span(&d.types, bt, lo, hi).0 * ew
                };
                if let Some(k) = konst(&idx).filter(|_| !self.signed(ex.args[1]) || idx.last() == Some(&F)) {
                    return Ok(vec![(span(k), T)]);
                }
                let signed = self.signed(ex.args[1]);
                let mut out = Vec::new();
                for k in l.min(r)..=l.max(r) {
                    let kv: Vec<Lit> = (0..idx.len()).map(|b| if (k >> b.min(63)) & 1 != 0 { T } else { F }).collect();
                    if !signed && k < 0 {
                        continue;
                    }
                    let c = self.equal(&idx, &kv);
                    if c != F {
                        out.push((span(k), c));
                    }
                }
                Ok(out)
            }
        }
    }

    // ---- expressions ----

    /// Resizes `v` to `w` bits, extending by sign if `signed`.
    fn fit(&self, mut v: Vec<Lit>, w: u32, signed: bool) -> Vec<Lit> {
        let fill = if signed { v.last().copied().unwrap_or(F) } else { F };
        v.resize(w as usize, fill);
        v
    }

    fn truth(&mut self, v: Vec<Lit>) -> Lit {
        v.into_iter().fold(F, |a, b| self.n.or(a, b))
    }

    fn equal(&mut self, a: &[Lit], b: &[Lit]) -> Lit {
        let mut eq = T;
        for (&x, &y) in a.iter().zip(b) {
            let d = self.n.xor(x, y);
            eq = self.n.and(eq, d ^ 1);
        }
        eq
    }

    /// `a + b + c`, and the carry out.
    fn add(&mut self, a: &[Lit], b: &[Lit], mut c: Lit) -> (Vec<Lit>, Lit) {
        let mut s = Vec::with_capacity(a.len());
        for (&x, &y) in a.iter().zip(b) {
            let h = self.n.xor(x, y);
            s.push(self.n.xor(h, c));
            c = self.n.maj(x, y, c);
        }
        (s, c)
    }

    /// `a < b`: no carry out of `a - b`.
    fn less(&mut self, a: &[Lit], b: &[Lit], signed: bool) -> Lit {
        let (mut a, mut b) = (a.to_vec(), b.iter().map(|&x| x ^ 1).collect::<Vec<_>>());
        if signed {
            if let (Some(x), Some(y)) = (a.last_mut(), b.last_mut()) {
                *x ^= 1;
                *y ^= 1;
            }
        }
        let (_, c) = self.add(&a, &b, T);
        c ^ 1
    }

    fn shift(&mut self, v: &[Lit], n: &[Lit], left: bool, fill: Lit) -> Vec<Lit> {
        let w = v.len();
        let by = |v: &[Lit], k: usize| -> Vec<Lit> {
            (0..w)
                .map(|i| if left { if i >= k { v[i - k] } else { F } } else if i + k < w { v[i + k] } else { fill })
                .collect()
        };
        if let Some(k) = konst(n) {
            return by(v, (k as usize).min(w));
        }
        let mut cur = v.to_vec();
        for (b, &s) in n.iter().enumerate() {
            let k = if b >= 32 { w } else { (1usize << b).min(w) };
            let moved = by(&cur, k);
            cur = (0..w).map(|i| self.n.mux(s, moved[i], cur[i])).collect();
        }
        cur
    }

    fn binop(&mut self, op: BinOp, a: Vec<Lit>, b: Vec<Lit>, signed: bool, at: NodeId) -> R<'a, Vec<Lit>> {
        use BinOp::*;
        let w = a.len();
        Ok(match op {
            Add => self.add(&a, &b, F).0,
            Sub => {
                let nb: Vec<Lit> = b.iter().map(|&x| x ^ 1).collect();
                self.add(&a, &nb, T).0
            }
            Mul => {
                let mut acc = vec![F; w];
                for (i, &bi) in b.iter().enumerate().take(w) {
                    if bi == F {
                        continue;
                    }
                    let part: Vec<Lit> = (0..w).map(|k| if k >= i { self.n.and(a[k - i], bi) } else { F }).collect();
                    acc = self.add(&acc, &part, F).0;
                }
                acc
            }
            And => a.iter().zip(&b).map(|(&x, &y)| self.n.and(x, y)).collect(),
            Or => a.iter().zip(&b).map(|(&x, &y)| self.n.or(x, y)).collect(),
            Xor => a.iter().zip(&b).map(|(&x, &y)| self.n.xor(x, y)).collect(),
            Xnor => a.iter().zip(&b).map(|(&x, &y)| self.n.xor(x, y) ^ 1).collect(),
            Shl | AShl => self.shift(&a, &b, true, F),
            Shr => self.shift(&a, &b, false, F),
            AShr => {
                let fill = if signed { a.last().copied().unwrap_or(F) } else { F };
                self.shift(&a, &b, false, fill)
            }
            Div | Mod | Pow if w <= 64 && konst(&a).is_some() && konst(&b).is_some() => {
                // Both constant (a loop's index, say): worked out here.
                let word = |v: &[Lit]| v.iter().enumerate().fold(0u64, |x, (i, &l)| x | ((l == T) as u64) << i);
                let x = Bits::from_u64(w as u32, signed, word(&a));
                let y = Bits::from_u64(b.len() as u32, signed, word(&b));
                let r = match op {
                    Div => x.div(&y),
                    Mod => x.rem(&y),
                    _ => x.pow(&y),
                };
                bits_of(&Const::Bits(r), w as u32).unwrap()
            }
            _ => return Err(self.err(at, format!("operator {} is not synthesised", op.text()))),
        })
    }

    fn eval(&mut self, inst: u32, e: ExprId, env: &Env) -> R<'a, Vec<Lit>> {
        let ex = self.d.expr(e).clone();
        let w = self.width(e);
        let at = ex.node;
        Ok(match ex.kind {
            ExprKind::Const(c) => bits_of(self.d.konst(c), w).ok_or_else(|| self.err(at, "a constant that is not a number"))?,
            ExprKind::Object(o) => {
                let obj = self.d.object(o);
                match obj.kind {
                    ObjKind::Param { .. } | ObjKind::EnumConst | ObjKind::Genvar => {
                        let c = obj.value.ok_or_else(|| self.err(at, "a parameter without a value"))?;
                        bits_of(self.d.konst(c), w).ok_or_else(|| self.err(at, "a parameter that is not a number"))?
                    }
                    _ => self.read(inst, o, env),
                }
            }
            ExprKind::Unary(op) => {
                let a = self.eval(inst, ex.args[0], env)?;
                match op {
                    UnOp::Plus => a,
                    UnOp::Neg => {
                        let z = vec![F; a.len()];
                        self.binop(BinOp::Sub, z, a, false, at)?
                    }
                    UnOp::Not => a.into_iter().map(|x| x ^ 1).collect(),
                    UnOp::LogNot => vec![self.truth(a) ^ 1],
                    UnOp::RedAnd | UnOp::RedNand => {
                        let r = a.into_iter().fold(T, |x, y| self.n.and(x, y));
                        vec![r ^ (op == UnOp::RedNand) as Lit]
                    }
                    UnOp::RedOr | UnOp::RedNor => vec![self.truth(a) ^ (op == UnOp::RedNor) as Lit],
                    UnOp::RedXor | UnOp::RedXnor => {
                        let r = a.into_iter().fold(F, |x, y| self.n.xor(x, y));
                        vec![r ^ (op == UnOp::RedXnor) as Lit]
                    }
                    _ => return Err(self.err(at, "an increment inside an expression is not synthesised")),
                }
            }
            ExprKind::Binary(op) => {
                let a = self.eval(inst, ex.args[0], env)?;
                let b = self.eval(inst, ex.args[1], env)?;
                use BinOp::*;
                let both_signed = self.signed(ex.args[0]) && self.signed(ex.args[1]);
                match op {
                    LogAnd | LogOr | Implies | Equiv => {
                        let (x, y) = (self.truth(a), self.truth(b));
                        vec![match op {
                            LogAnd => self.n.and(x, y),
                            LogOr => self.n.or(x, y),
                            Implies => self.n.or(x ^ 1, y),
                            _ => self.n.xor(x, y) ^ 1,
                        }]
                    }
                    Eq | Ne | CaseEq | CaseNe | WildEq | WildNe => {
                        let n = a.len().max(b.len()) as u32;
                        let (a, b) = (self.fit(a, n, both_signed), self.fit(b, n, both_signed));
                        let eq = self.equal(&a, &b);
                        vec![eq ^ matches!(op, Ne | CaseNe | WildNe) as Lit]
                    }
                    Lt | Le | Gt | Ge => {
                        let n = a.len().max(b.len()) as u32;
                        let (a, b) = (self.fit(a, n, both_signed), self.fit(b, n, both_signed));
                        vec![match op {
                            Lt => self.less(&a, &b, both_signed),
                            Gt => self.less(&b, &a, both_signed),
                            Le => self.less(&b, &a, both_signed) ^ 1,
                            _ => self.less(&a, &b, both_signed) ^ 1,
                        }]
                    }
                    Shl | Shr | AShl | AShr => {
                        let a = self.fit(a, w, self.signed(ex.args[0]));
                        self.binop(op, a, b, self.signed(e), at)?
                    }
                    _ => {
                        let (a, b) = (self.fit(a, w, self.signed(ex.args[0])), self.fit(b, w, self.signed(ex.args[1])));
                        self.binop(op, a, b, self.signed(e), at)?
                    }
                }
            }
            ExprKind::Cond => {
                let c = self.eval(inst, ex.args[0], env)?;
                let c = self.truth(c);
                let a = self.eval(inst, ex.args[1], env)?;
                let b = self.eval(inst, ex.args[2], env)?;
                let (a, b) = (self.fit(a, w, self.signed(ex.args[1])), self.fit(b, w, self.signed(ex.args[2])));
                a.into_iter().zip(b).map(|(x, y)| self.n.mux(c, x, y)).collect()
            }
            ExprKind::Concat => {
                let mut out = Vec::new();
                for &a in ex.args.iter().rev() {
                    out.extend(self.eval(inst, a, env)?);
                }
                out
            }
            ExprKind::Replicate(n) => {
                let one = self.eval(inst, ex.args[0], env)?;
                (0..n).flat_map(|_| one.iter().copied()).collect()
            }
            ExprKind::Index | ExprKind::Part { .. } | ExprKind::Member(_) => {
                let base = self.eval(inst, ex.args[0], env)?;
                let sel = self.select(inst, &ex, env)?;
                let mut out = vec![F; w as usize];
                for (n, (off, cond)) in sel.into_iter().enumerate() {
                    for (i, o) in out.iter_mut().enumerate() {
                        let k = off + i as i64;
                        let b = if k >= 0 && (k as usize) < base.len() { base[k as usize] } else { F };
                        *o = if n == 0 { self.n.and(cond, b) } else { self.n.mux(cond, b, *o) };
                    }
                }
                out
            }
            ExprKind::Convert => {
                let a = self.eval(inst, ex.args[0], env)?;
                self.fit(a, w, self.signed(ex.args[0]))
            }
            ExprKind::Inside => {
                let v = self.eval(inst, ex.args[0], env)?;
                let signed = self.signed(ex.args[0]);
                let mut r = F;
                for &it in &ex.args[1..] {
                    let m = self.inside(inst, &v, signed, it, env)?;
                    r = self.n.or(r, m);
                }
                vec![r]
            }
            ExprKind::SysCall(f) if matches!(self.d.name(f), "$signed" | "$unsigned") && ex.args.len() == 1 => {
                let a = self.eval(inst, ex.args[0], env)?;
                self.fit(a, w, self.signed(ex.args[0]))
            }
            ExprKind::Call(_) => return Err(self.err(at, "function calls are not synthesised yet")),
            _ => return Err(self.err(at, "this expression is not synthesised")),
        })
    }

    /// Object `o`'s value as a procedure sees it: what it has assigned
    /// by blocking assignment where it has, the object's wires elsewhere.
    fn read(&mut self, inst: u32, o: ObjId, env: &Env) -> Vec<Lit> {
        let key = (inst, o);
        let slot = env.blk.get(&key).cloned();
        if let Some(s) = &slot {
            if s.en.iter().all(|&e| e == T) {
                return s.val.clone();
            }
        }
        let wires = self.obj_bits(inst, o);
        match slot {
            None => wires,
            Some(s) => (0..wires.len()).map(|i| self.n.mux(s.en[i], s.val[i], wires[i])).collect(),
        }
    }
}

/// Preprocesses, parses and elaborates `src` with top-level module `top`,
/// and lowers it; errors rendered with their positions.
pub fn synthesise(src: &crate::model::Sources, top: &str) -> Result<Netlist, String> {
    let n = crate::model::elaborate_with_ast(src, &[top], |d, ast| lower(d, ast, d.tops[0]))?.rebuild()?;
    n.clock()?;
    Ok(n)
}

impl Netlist {
    /// The number of nodes of each kind that are gates.
    pub fn gates(&self) -> usize {
        self.nodes.iter().filter(|n| matches!(n, Node::And(..) | Node::Xor(..) | Node::Maj(..))).count()
    }

    /// The output port `name`'s index.
    pub fn output(&self, name: &str) -> Option<usize> {
        self.outputs.iter().position(|p| p.name == name)
    }

    pub fn input_port(&self, name: &str) -> Option<usize> {
        self.inputs.iter().position(|p| p.name == name)
    }

    /// Whether literal `l` is a constant.
    pub fn is_const(l: Lit) -> bool {
        node(l) == 0
    }
}
