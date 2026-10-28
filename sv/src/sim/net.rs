//! Nets with more than one driver, and strengths (6.6, 28.11 to 28.15).
//!
//! A net that needs resolving keeps each driver's contribution, one
//! signal per bit, and combines them whenever one changes. A signal is the
//! range of strength levels it covers on the scale of 28.11, written here
//! as -7 (supply 0) through 0 (high impedance) to 7 (supply 1): a strong 1
//! is `6..6`, a weak `x` is `-3..3` and an `H` of pull strength is `0..5`.


#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Sig {
    pub lo: i8,
    pub hi: i8,
}

pub const STRONG: i8 = 6;
pub const PULL: i8 = 5;
pub const SUPPLY: i8 = 7;

impl Sig {
    pub const Z: Sig = Sig { lo: 0, hi: 0 };

    fn at(level: i8) -> Sig {
        Sig { lo: level, hi: level }
    }

    /// A driven value with the driver's strength for 0 and for 1.
    pub fn drive(l: Logic, s0: i8, s1: i8) -> Sig {
        match l {
            Logic::Zero => Sig::at(-s0),
            Logic::One => Sig::at(s1),
            Logic::X => Sig { lo: -s0, hi: s1 },
            Logic::Z => Sig::Z,
        }
    }

    /// The four-state value: `L` and `H` read as `x`.
    pub fn logic(self) -> Logic {
        if self == Sig::Z {
            Logic::Z
        } else if self.lo > 0 {
            Logic::One
        } else if self.hi < 0 {
            Logic::Zero
        } else {
            Logic::X
        }
    }

    fn hull(self, o: Sig) -> Sig {
        Sig { lo: self.lo.min(o.lo), hi: self.hi.max(o.hi) }
    }

    /// Each level's strength mapped by `f`, keeping its side.
    fn map(self, f: impl Fn(i8) -> i8) -> Sig {
        let m = |l: i8| l.signum() * f(l.abs());
        Sig { lo: m(self.lo), hi: m(self.hi) }
    }

    /// Through a MOS switch or a `tran`: supply becomes strong (28.13).
    pub fn pass(self) -> Sig {
        self.map(|s| s.min(STRONG))
    }

    /// Through a resistive device (table 28-8).
    pub fn resist(self) -> Sig {
        self.map(|s| match s {
            7 | 6 => 5,
            5 => 3,
            4 | 3 => 2,
            2 | 1 => 1,
            _ => 0,
        })
    }

    /// The level as a charge of strength `c` holds it (28.15.2).
    pub fn charge(self, c: i8) -> Sig {
        match self.logic() {
            Logic::Zero => Sig::at(-c),
            Logic::One => Sig::at(c),
            _ => Sig { lo: -c, hi: c },
        }
    }
}

/// How a net settles conflicts between drivers of equal strength.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Wired {
    /// `wire`, `tri` and the rest: `x` (table 6-2).
    No,
    /// `wand`, `triand` (table 6-3).
    And,
    /// `wor`, `trior` (table 6-4).
    Or,
}

/// Two single levels: the stronger wins; equal and opposite give `x`, or
/// the wired-logic value (28.12.1, 28.12.4).
fn point(x: i8, y: i8, w: Wired) -> Sig {
    if x == 0 {
        return Sig::at(y);
    }
    if y == 0 || x == y || x.abs() > y.abs() {
        return Sig::at(x);
    }
    if y.abs() > x.abs() {
        return Sig::at(y);
    }
    let s = x.abs();
    match w {
        Wired::No => Sig { lo: -s, hi: s },
        Wired::And => Sig::at(-s),
        Wired::Or => Sig::at(s),
    }
}

/// Two signals combined (28.12). Each extreme of one meets each extreme
/// of the other, and the result covers every outcome; this keeps the
/// stronger levels of an ambiguous signal and fills the gap between
/// opposite values, as 28.12.3 asks.
pub fn combine(a: Sig, b: Sig, w: Wired) -> Sig {
    if a == Sig::Z {
        return b;
    }
    if b == Sig::Z {
        return a;
    }
    let mut r = point(a.lo, b.lo, w);
    for (x, y) in [(a.lo, b.hi), (a.hi, b.lo), (a.hi, b.hi)] {
        r = r.hull(point(x, y, w));
    }
    r
}

/// `%v` (21.2.1.5, table 21-5).
pub fn format(s: Sig) -> String {
    let name = |l: i8| ["Hi", "Sm", "Me", "We", "La", "Pu", "St", "Su"][l.unsigned_abs().min(7) as usize];
    if s == Sig::Z {
        return "HiZ".into();
    }
    match s.logic() {
        Logic::Zero if s.lo == s.hi => format!("{}0", name(s.lo)),
        Logic::One if s.lo == s.hi => format!("{}1", name(s.hi)),
        Logic::Zero => format!("{}{}0", -s.lo, -s.hi),
        Logic::One => format!("{}{}1", s.hi, s.lo),
        _ if s.hi == 0 => format!("{}L", name(s.lo)),
        _ if s.lo == 0 => format!("{}H", name(s.hi)),
        _ if -s.lo == s.hi => format!("{}X", name(s.hi)),
        _ => format!("{}{}X", -s.lo, s.hi),
    }
}

/// The strength level a strength keyword names (table 28-7).
pub fn level(kw: &str) -> Option<i8> {
    Some(match kw.trim_end_matches(['0', '1']) {
        "supply" => SUPPLY,
        "strong" => STRONG,
        "pull" => PULL,
        "weak" => 3,
        "highz" => 0,
        "large" => 4,
        "medium" => 2,
        "small" => 1,
        _ => return None,
    })
}

/// A strength specification's levels for 0 and for 1, from its keywords
/// in either order; what is not given is `default`.
pub fn strengths(kws: &[&str], default: (i8, i8)) -> (i8, i8) {
    let (mut s0, mut s1) = default;
    for k in kws {
        if let Some(l) = level(k) {
            if k.ends_with('0') {
                s0 = l;
            } else if k.ends_with('1') {
                s1 = l;
            }
        }
    }
    (s0, s1)
}

pub struct Net {
    pub wired: Wired,
    /// The value a `tri0`, `tri1` or supply net drives itself.
    pub own: Option<Sig>,
    /// A trireg's charge strength.
    pub charge: Option<i8>,
    /// A trireg's charge decay time, in ticks.
    pub decay: Option<u64>,
    /// Bumped when a trireg is driven again, cancelling a pending decay.
    pub decay_epoch: u32,
    pub width: u32,
    pub ty: TypeId,
    /// Each driver's contribution, one signal per bit.
    pub contrib: Vec<Vec<Sig>>,
    /// The resolved signal.
    pub sig: Vec<Sig>,
}

impl Net {
    /// Recomputes the net from its drivers. Returns whether a trireg has
    /// just started holding its charge.
    pub fn resolve(&mut self) -> bool {
        let mut stored = false;
        for i in 0..self.width as usize {
            let mut acc = self.own.unwrap_or(Sig::Z);
            for c in &self.contrib {
                acc = combine(acc, c.get(i).copied().unwrap_or(Sig::Z), self.wired);
            }
            if let Some(c) = self.charge
                && acc == Sig::Z
            {
                // Every driver is off: the net keeps its last value as charge.
                let held = self.sig[i].charge(c);
                stored |= held != self.sig[i];
                acc = held;
            }
            self.sig[i] = acc;
        }
        stored
    }

    pub fn bits(&self) -> Bits {
        let mut b = Bits::zero(self.width, false);
        for (i, s) in self.sig.iter().enumerate() {
            b.set_bit(i as u32, s.logic());
        }
        b
    }
}

/// A value's bits as signals of the given strengths.
pub fn sigs_of(v: &crate::design::Const, s0: i8, s1: i8) -> Vec<Sig> {
    match v {
        crate::design::Const::Bits(b) => (0..b.width()).map(|i| Sig::drive(b.bit(i), s0, s1)).collect(),
        _ => vec![Sig::drive(Logic::X, s0, s1)],
    }
}

/// Which of a gate's terminals are outputs (28.3 to 28.10).
pub fn cell_outputs(kind: &str, n: usize) -> std::ops::Range<usize> {
    match kind {
        "buf" | "not" => 0..n.saturating_sub(1),
        "pullup" | "pulldown" => 0..n,
        "tran" | "rtran" | "tranif0" | "tranif1" | "rtranif0" | "rtranif1" => 0..0,
        _ => 0..1.min(n),
    }
}

fn udp_level(c: u8) -> Logic {
    match c {
        b'0' => Logic::Zero,
        b'1' => Logic::One,
        _ => Logic::X,
    }
}

/// Whether a level symbol (table 29-1) covers a value.
fn udp_matches(sym: u8, v: Logic) -> bool {
    match sym {
        b'0' => v == Logic::Zero,
        b'1' => v == Logic::One,
        b'x' => v == Logic::X,
        b'?' => true,
        b'b' => v.is_known(),
        _ => false,
    }
}

/// A UDP's next output from its inputs before and now and its state
/// (29.4 to 29.6, 29.9). Edge rows are tried for each input that changed,
/// then level rows, which win; a change no row covers gives x, and `-`
/// keeps the state.
pub fn udp_eval(u: &crate::design::Udp, prev: &[Logic], cur: &[Logic], q: Logic) -> Logic {
    use crate::design::UdpIn;
    let state_ok = |r: &crate::design::UdpRow, q: Logic| r.state.is_none_or(|s| udp_matches(s, q));
    let out = |r: &crate::design::UdpRow, q: Logic| if r.out == b'-' { q } else { udp_level(r.out) };
    let levels_ok = |r: &crate::design::UdpRow, skip: Option<usize>| {
        r.inputs.len() == cur.len()
            && r.inputs.iter().enumerate().all(|(k, i)| match i {
                UdpIn::Level(c) => udp_matches(*c, cur[k]),
                _ => Some(k) == skip,
            })
    };
    if !u.sequential {
        return u.rows.iter().find(|r| levels_ok(r, None)).map_or(Logic::X, |r| out(r, q));
    }
    let mut result = None;
    let mut q2 = q;
    for k in (0..cur.len()).filter(|&k| prev.get(k) != Some(&cur[k])) {
        let (a, b) = (prev.get(k).copied().unwrap_or(Logic::X), cur[k]);
        let edge = |i: &UdpIn| match *i {
            UdpIn::Edge(x, y) => udp_matches(x, a) && udp_matches(y, b),
            UdpIn::Pos => matches!((a, b), (Logic::Zero, Logic::One | Logic::X) | (Logic::X, Logic::One)),
            UdpIn::Neg => matches!((a, b), (Logic::One, Logic::Zero | Logic::X) | (Logic::X, Logic::Zero)),
            UdpIn::Level(_) => false,
        };
        let hit = u.rows.iter().find(|r| r.inputs.get(k).is_some_and(edge) && levels_ok(r, Some(k)) && state_ok(r, q2));
        q2 = match hit {
            Some(r) => out(r, q2),
            None => Logic::X,
        };
        result = Some(q2);
    }
    if let Some(r) = u.rows.iter().find(|r| levels_ok(r, None) && state_ok(r, q)) {
        return out(r, q);
    }
    // A change no row covers gave x above; no change keeps the state.
    result.unwrap_or(q)
}

/// The net kinds that resolve even with one driver, or none.
fn special(kind: &str) -> bool {
    !matches!(kind, "wire" | "tri" | "uwire" | "interconnect")
}

use super::*;
use crate::design::value::{Bits, Logic};

impl Sim<'_, '_, '_> {
    /// Makes a resolved net of each net with several drivers, a kind that
    /// resolves (`wand`, `tri1`, `trireg`, ...), or a driver with a
    /// strength, and points each of its drivers at a place in it.
    pub(super) fn setup_nets(&mut self) {
        // The nets of each specialisation, then the kind each slot holds.
        let mut spec_nets: HashMap<SpecId, Vec<(u32, ObjId)>> = HashMap::new();
        for (i, o) in self.d.objects.iter().enumerate() {
            if let (Home::Frame(sp, off), ObjKind::Net(_)) = (self.homes[i], o.kind)
                && self.d.types.is_integral(o.ty)
            {
                spec_nets.entry(sp).or_default().push((off, ObjId(i as u32)));
            }
        }
        let mut kinds: HashMap<usize, (ObjId, u32)> = HashMap::new();
        for inst in 0..self.insts.len() {
            let r = &self.insts[inst];
            let Some(list) = spec_nets.get(&r.spec) else { continue };
            for &(off, o) in list {
                let s = self.find((r.base + off) as usize);
                let plain = |o: ObjId| match self.d.object(o).kind {
                    ObjKind::Net(k) => !special(self.d.name(k)),
                    _ => true,
                };
                match kinds.get(&s) {
                    Some(&(old, _)) if !plain(old) || plain(o) => {}
                    _ => {
                        kinds.insert(s, (o, inst as u32));
                    }
                }
            }
        }
        if kinds.is_empty() {
            return;
        }
        // Each driver's outputs onto those nets.
        type Use = (u32, usize, Option<(u32, u32)>);
        let mut users: HashMap<usize, Vec<Use>> = HashMap::new();
        let mut strong: std::collections::HashSet<usize> = Default::default();
        for p in 0..self.procs.len() as u32 {
            let Some(dr) = self.procs[p as usize].driver else { continue };
            let by_strength = match dr {
                Driven::Cell(_) => true,
                Driven::Assign(id) => self.d.drivers[id.0 as usize].strength.is_some(),
                Driven::Port(..) => false,
            };
            for (k, t) in self.driver_targets(p, dr).into_iter().enumerate() {
                let Some(t) = t else { continue };
                let Place::Slot(s) = t.place else { continue };
                let s = self.find(s);
                if !kinds.contains_key(&s) {
                    continue;
                }
                let bits = match t.path.as_slice() {
                    [] => None,
                    [Sel::Bits(lo, w)] => Some((*lo, *w)),
                    _ => continue,
                };
                users.entry(s).or_default().push((p, k, bits));
                if by_strength {
                    strong.insert(s);
                }
            }
        }
        let mut made = Vec::new();
        for (&s, &(o, inst)) in &kinds {
            let ObjKind::Net(k) = self.d.object(o).kind else { continue };
            let kind = self.d.name(k);
            let us = users.get(&s).cloned().unwrap_or_default();
            if !(special(kind) || us.len() >= 2 || strong.contains(&s)) {
                continue;
            }
            let ty = self.d.object(o).ty;
            let width = self.d.types.width(ty);
            let info = self.d.nets.get(&o).cloned().unwrap_or_default();
            let charge = (kind == "trireg").then(|| info.charge.and_then(|c| level(self.d.name(c))).unwrap_or(2));
            let decay = match (charge, info.delay.get(2)) {
                (Some(_), Some(&x)) => {
                    let cx = Cx { inst, locals: Rc::default(), this: None };
                    let v = self.eval(&cx, x);
                    Some(self.ticks(inst, &v))
                }
                _ => None,
            };
            let own = match kind {
                "tri0" => Some(Sig::at(-PULL)),
                "tri1" => Some(Sig::at(PULL)),
                "supply0" => Some(Sig::at(-SUPPLY)),
                "supply1" => Some(Sig::at(SUPPLY)),
                _ => None,
            };
            let wired = match kind {
                "wand" | "triand" => Wired::And,
                "wor" | "trior" => Wired::Or,
                _ => Wired::No,
            };
            // A trireg starts as x, held at its charge (6.6.4).
            let start = match charge {
                Some(c) => Sig { lo: -c, hi: c },
                None => Sig::Z,
            };
            // A gate's output is x until it is first evaluated, and a
            // delayed gate's for its delay.
            let mut contrib = vec![vec![Sig::Z; width as usize]; us.len()];
            for (index, &(p, _, _)) in us.iter().enumerate() {
                if let Some(Driven::Cell(c)) = self.procs[p as usize].driver
                    && !self.d.cells[c.0 as usize].delay.is_empty()
                {
                    contrib[index] = vec![Sig { lo: -STRONG, hi: STRONG }; width as usize];
                }
            }
            let mut net = Net {
                wired,
                own,
                charge,
                decay,
                decay_epoch: 0,
                width,
                ty,
                contrib,
                sig: vec![start; width as usize],
            };
            net.resolve();
            for (index, &(p, k, bits)) in us.iter().enumerate() {
                let outs = &mut self.procs[p as usize].outs;
                if outs.len() <= k {
                    outs.resize(k + 1, None);
                }
                outs[k] = Some(NetOut { slot: s, index: index as u32, bits });
            }
            self.nets.insert(s, net);
            made.push(s);
        }
        for s in made {
            self.write_net(s);
        }
    }

    /// Where each of a driver's outputs goes.
    fn driver_targets(&mut self, p: u32, dr: Driven) -> Vec<Option<Target>> {
        let inst = self.procs[p as usize].frames[0].cx.inst;
        let cx = Cx { inst, locals: Rc::default(), this: None };
        let lvalue = |me: &Self, x: ExprId| {
            matches!(me.d.expr(x).kind, ExprKind::Object(_) | ExprKind::Hier { .. } | ExprKind::Index | ExprKind::Part { .. })
        };
        match dr {
            Driven::Assign(id) => {
                let lhs = self.d.drivers[id.0 as usize].lhs;
                if lvalue(self, lhs) { vec![Some(self.target(&cx, lhs))] } else { vec![None] }
            }
            Driven::Port(child, k) => {
                let Some(parent) = self.insts[child as usize].parent else { return vec![] };
                let elab = self.insts[child as usize].elab;
                let spec = self.insts[child as usize].spec;
                let conn = self.d.instance(elab).connections[k].clone();
                let Some(port) = self.d.spec(spec).ports.iter().find(|q| q.name == conn.port).cloned() else { return vec![] };
                let (Some(x), Some(po)) = (conn.expr, port.object) else { return vec![] };
                if port.dir == Dir::Input {
                    let ccx = Cx { inst: child, locals: Rc::default(), this: None };
                    vec![Some(self.target_obj(&ccx, po))]
                } else if lvalue(self, x) {
                    let pcx = Cx { inst: parent, locals: Rc::default(), this: None };
                    vec![Some(self.target(&pcx, x))]
                } else {
                    vec![None]
                }
            }
            Driven::Cell(c) => {
                let cell = &self.d.cells[c.0 as usize];
                let terms = cell.terminals.clone();
                let outs = cell_outputs(self.d.name(cell.kind), terms.len());
                terms[outs].iter().map(|&t| lvalue(self, t).then(|| self.target(&cx, t))).collect()
            }
        }
    }

    /// Bumps the driver's generation and puts its outputs out now or after
    /// its delay: rise for a 1, fall for a 0, turn-off for a z, and the
    /// least for an x (28.16).
    pub(super) fn emit(&mut self, p: u32, inst: u32, delay: &[ExprId], outs: Vec<(Const, Vec<Sig>)>) {
        let pr = &mut self.procs[p as usize];
        pr.round = pr.round.wrapping_add(1);
        let round = pr.round;
        if delay.is_empty() {
            self.put_outputs(p, outs);
            return;
        }
        let cx = Cx { inst, locals: Rc::default(), this: None };
        let mut ds = Vec::new();
        for &x in delay {
            let v = self.eval(&cx, x);
            ds.push(self.ticks(inst, &v));
        }
        let rise = ds[0];
        let fall = ds.get(1).copied().unwrap_or(rise);
        let off = ds.get(2).copied().unwrap_or(rise.min(fall));
        let to = outs.first().and_then(|(_, s)| s.first()).map_or(Logic::X, |s| s.logic());
        let ticks = match to {
            Logic::One => rise,
            Logic::Zero => fall,
            Logic::Z => off,
            Logic::X => rise.min(fall).min(off),
        };
        if ticks == 0 {
            self.put_outputs(p, outs);
        } else {
            self.after(ticks, Timed::Drive(p, round, outs));
        }
    }

    /// Writes a driver's outputs: into its places in resolved nets, or
    /// straight to its targets.
    pub(super) fn put_outputs(&mut self, p: u32, outs: Vec<(Const, Vec<Sig>)>) {
        let mut targets = None;
        for (k, (v, sig)) in outs.into_iter().enumerate() {
            match self.procs[p as usize].outs.get(k).copied().flatten() {
                Some(o) => self.net_drive(o, sig),
                None => {
                    let dr = self.procs[p as usize].driver.unwrap();
                    let ts = targets.get_or_insert_with(|| self.driver_targets(p, dr));
                    if let Some(Some(t)) = ts.get(k).cloned() {
                        let inst = self.procs[p as usize].frames[0].cx.inst;
                        let cx = Cx { inst, locals: Rc::default(), this: None };
                        self.write_cx(&cx, &t, v, false);
                    }
                }
            }
        }
    }

    fn net_drive(&mut self, o: NetOut, sig: Vec<Sig>) {
        let Some(net) = self.nets.get_mut(&o.slot) else { return };
        let c = &mut net.contrib[o.index as usize];
        match o.bits {
            None => {
                for (i, x) in c.iter_mut().enumerate() {
                    *x = sig.get(i).copied().unwrap_or(Sig::Z);
                }
            }
            Some((lo, w)) => {
                for i in 0..w as usize {
                    if let Some(x) = c.get_mut(lo as usize + i) {
                        *x = sig.get(i).copied().unwrap_or(Sig::Z);
                    }
                }
            }
        }
        let stored = net.resolve();
        if net.charge.is_some() && net.sig.iter().any(|s| *s != Sig::Z) {
            let driven = net.contrib.iter().any(|c| c.iter().any(|s| *s != Sig::Z));
            if driven || stored {
                // Driven again, or newly holding charge: restart the decay.
                net.decay_epoch = net.decay_epoch.wrapping_add(1);
            }
            if stored && !driven && let Some(t) = net.decay {
                let e = net.decay_epoch;
                self.after(t, Timed::Decay(o.slot, e));
            }
        }
        self.write_net(o.slot);
    }

    fn write_net(&mut self, s: usize) {
        let Some(net) = self.nets.get(&s) else { return };
        let t = Target { place: Place::Slot(s), path: vec![], ty: net.ty };
        let v = Const::Bits(net.bits());
        let cx = Cx { inst: 0, locals: Rc::default(), this: None };
        self.write_cx(&cx, &t, v, false);
    }

    /// A trireg's held charge decays to x (28.16.2).
    pub(super) fn decay(&mut self, s: usize, epoch: u32) {
        let Some(net) = self.nets.get_mut(&s) else { return };
        if net.decay_epoch != epoch {
            return;
        }
        let c = net.charge.unwrap_or(2);
        for x in net.sig.iter_mut() {
            *x = Sig { lo: -c, hi: c };
        }
        self.write_net(s);
    }

    /// A gate's, switch's or pull's outputs, or `None` if it is not
    /// modelled.
    pub(super) fn cell(&mut self, p: u32, cx: &Cx, c: CellId) -> Option<Vec<(Const, Vec<Sig>)>> {
        let cell = &self.d.cells[c.0 as usize];
        let kind = self.d.name(cell.kind);
        let terms = cell.terminals.clone();
        let names: Vec<&str> = cell.strength.iter().map(|&k| self.d.name(k)).collect();
        let pull = matches!(kind, "pullup" | "pulldown");
        let (s0, s1) = strengths(&names, if pull { (PULL, PULL) } else { (STRONG, STRONG) });
        let width = |me: &Self, x: ExprId| me.d.types.width(me.d.expr(x).ty).max(1);
        let outs = cell_outputs(kind, terms.len());
        let bit = |v: &Const, i: u32| match v {
            Const::Bits(b) => b.bit(if b.width() > 1 { i } else { 0 }),
            _ => Logic::X,
        };
        let sigs: Vec<Sig> = match kind {
            "pullup" | "pulldown" => {
                let l = if kind == "pullup" { Logic::One } else { Logic::Zero };
                vec![Sig::drive(l, s0, s1); width(self, terms[0]) as usize]
            }
            "nmos" | "pmos" | "rnmos" | "rpmos" | "cmos" | "rcmos" => {
                let dv = self.eval(cx, terms[1]);
                let data = self.sigs_of_expr(cx, terms[1], &dv);
                let w = width(self, terms[0]);
                let switch = |d: Sig, c: Logic, on: Logic| {
                    if c == on {
                        d
                    } else if c.is_known() {
                        Sig::Z
                    } else {
                        // Unknown control: the value or nothing (L or H).
                        Sig { lo: d.lo.min(0), hi: d.hi.max(0) }
                    }
                };
                let mut r = Vec::new();
                if kind.ends_with("cmos") {
                    let (n, pc) = (self.eval(cx, terms[2]), self.eval(cx, terms[3]));
                    for i in 0..w {
                        let d = data.get(i as usize).copied().unwrap_or(data[0]);
                        r.push(combine(switch(d, bit(&n, i), Logic::One), switch(d, bit(&pc, i), Logic::Zero), Wired::No));
                    }
                } else {
                    let ctl = self.eval(cx, terms[2]);
                    let on = if kind.ends_with("nmos") { Logic::One } else { Logic::Zero };
                    for i in 0..w {
                        let d = data.get(i as usize).copied().unwrap_or(data[0]);
                        r.push(switch(d, bit(&ctl, i), on));
                    }
                }
                let resistive = kind.starts_with('r');
                r.into_iter().map(|x| if resistive { x.resist() } else { x.pass() }).collect()
            }
            "and" | "nand" | "or" | "nor" | "xor" | "xnor" | "buf" | "not" | "bufif0" | "bufif1" | "notif0" | "notif1" => {
                let ins: Vec<Const> = (0..terms.len()).filter(|k| !outs.contains(k)).map(|k| self.eval(cx, terms[k])).collect();
                // An array of instances: a bit of each terminal per instance
                // (28.3.6).
                let w = terms.iter().map(|&t| width(self, t)).max().unwrap_or(1);
                let mut r = Vec::new();
                for i in 0..w {
                    let bits: Vec<Const> = ins.iter().map(|v| Const::Bits(Bits::of_logic(bit(v, i)))).collect();
                    let l = match super::eval::gate(kind, &bits) {
                        Some(Const::Bits(b)) => b.bit(0),
                        _ => Logic::X,
                    };
                    let tri = kind.contains("if");
                    let ctl = if tri { bit(&ins[1], i) } else { Logic::One };
                    if tri && !ctl.is_known() {
                        // Unknown enable: the data value or z (28.6).
                        let d = bit(&ins[0], i);
                        let d = if kind.starts_with("not") { super::eval::gate("not", &[Const::Bits(Bits::of_logic(d))]).map_or(Logic::X, |c| match c { Const::Bits(b) => b.bit(0), _ => Logic::X }) } else { d };
                        let x = Sig::drive(if d == Logic::Z { Logic::X } else { d }, s0, s1);
                        r.push(Sig { lo: x.lo.min(0), hi: x.hi.max(0) });
                    } else {
                        r.push(Sig::drive(l, s0, s1));
                    }
                }
                r
            }
            _ => {
                let d = self.d;
                let u = d.udps.get(&cell.kind)?;
                let ins: Vec<Const> = terms[1..].iter().map(|&t| self.eval(cx, t)).collect();
                let w = terms.iter().map(|&t| width(self, t)).max().unwrap_or(1);
                let mut r = Vec::new();
                for i in 0..w {
                    // z reads as x (29.3.5).
                    let cur: Vec<Logic> = ins.iter().map(|v| if bit(v, i) == Logic::Z { Logic::X } else { bit(v, i) }).collect();
                    let next = match self.udp_state.get(&(p, i)) {
                        Some((prev, q)) => udp_eval(u, prev, &cur, *q),
                        // The first evaluation: a sequential UDP starts at
                        // its initial value (29.7).
                        None if u.sequential => udp_eval(u, &cur, &cur, udp_level(u.init)),
                        None => udp_eval(u, &cur, &cur, Logic::X),
                    };
                    self.udp_state.insert((p, i), (cur, next));
                    r.push(Sig::drive(next, s0, s1));
                }
                r
            }
        };
        let mut res = Vec::new();
        for k in outs {
            let w = width(self, terms[k]) as usize;
            let sig: Vec<Sig> = (0..w).map(|i| sigs.get(i).copied().unwrap_or(sigs[0])).collect();
            let mut b = Bits::zero(w as u32, false);
            for (i, s) in sig.iter().enumerate() {
                b.set_bit(i as u32, s.logic());
            }
            res.push((Const::Bits(b), sig));
        }
        Some(res)
    }

    /// The signals an expression carries: a resolved net's own, or its
    /// value at strong strength.
    pub(super) fn sigs_of_expr(&self, cx: &Cx, x: ExprId, v: &Const) -> Vec<Sig> {
        self.net_sigs(cx, x).unwrap_or_else(|| sigs_of(v, STRONG, STRONG))
    }

    /// A resolved net's signals, for a name or a bit of one.
    pub(super) fn net_sigs(&self, cx: &Cx, x: ExprId) -> Option<Vec<Sig>> {
        let e = self.d.expr(x);
        let slot = |me: &Self, x: ExprId| -> Option<usize> {
            match me.d.expr(x).kind {
                ExprKind::Object(o) => me.slot_of_obj(cx.inst, o),
                ExprKind::Hier { object: Some(o), .. } => me.slot_of_obj(me.hops(cx.inst, x)?, o),
                _ => None,
            }
            .map(|s| me.find(s))
        };
        match e.kind {
            ExprKind::Object(_) | ExprKind::Hier { .. } => Some(self.nets.get(&slot(self, x)?)?.sig.clone()),
            ExprKind::Index => {
                let net = self.nets.get(&slot(self, e.args[0])?)?;
                let ExprKind::Const(c) = self.d.expr(e.args[1]).kind else { return None };
                let i = self.d.konst(c).to_i64()?;
                let bt = self.d.expr(e.args[0]).ty;
                let off = crate::design::ops::lsb_offset(&self.d.types, bt, i, i)?;
                Some(vec![*net.sig.get(off as usize)?])
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(l: Logic, st: i8) -> Sig {
        Sig::drive(l, st, st)
    }

    #[test]
    fn wire_table() {
        // Table 6-2, equal strengths.
        let v = [Logic::Zero, Logic::One, Logic::X, Logic::Z];
        let want = "0xx0x1x1xxxx01xz";
        for (i, &a) in v.iter().enumerate() {
            for (j, &b) in v.iter().enumerate() {
                let r = combine(s(a, STRONG), s(b, STRONG), Wired::No).logic();
                let c = match r {
                    Logic::Zero => '0',
                    Logic::One => '1',
                    Logic::X => 'x',
                    Logic::Z => 'z',
                };
                assert_eq!(c, want.as_bytes()[i * 4 + j] as char, "{a:?} {b:?}");
            }
        }
    }

    #[test]
    fn wired_tables() {
        let v = [Logic::Zero, Logic::One, Logic::X, Logic::Z];
        let ch = |l: Logic| match l {
            Logic::Zero => '0',
            Logic::One => '1',
            Logic::X => 'x',
            Logic::Z => 'z',
        };
        for (w, want) in [(Wired::And, "000001x10xxx01xz"), (Wired::Or, "01x01111x1xx01xz")] {
            for (i, &a) in v.iter().enumerate() {
                for (j, &b) in v.iter().enumerate() {
                    let r = combine(s(a, STRONG), s(b, STRONG), w).logic();
                    assert_eq!(ch(r), want.as_bytes()[i * 4 + j] as char, "{w:?} {a:?} {b:?}");
                }
            }
        }
    }

    #[test]
    fn strengths_combine() {
        // The stronger signal wins (28.12.1).
        assert_eq!(combine(s(Logic::One, PULL), s(Logic::Zero, STRONG), Wired::No), Sig::at(-6));
        assert_eq!(format(combine(s(Logic::One, 3), s(Logic::Zero, 3), Wired::No)), "WeX");
        // Rule c of 28.12.3: a range across both values.
        let amb = Sig { lo: -6, hi: 3 };
        assert_eq!(combine(Sig::at(5), amb, Wired::No), Sig { lo: -6, hi: 5 });
        assert_eq!(format(Sig { lo: -6, hi: 5 }), "65X");
        assert_eq!(format(Sig { lo: -5, hi: -2 }), "520");
        assert_eq!(format(Sig { lo: 0, hi: 5 }), "PuH");
        assert_eq!(Sig::at(7).resist(), Sig::at(5));
        assert_eq!(Sig::at(-7).pass(), Sig::at(-6));
    }
}
