//! Expressions at run time, assignment targets, calls, classes and
//! continuous drivers.

use super::*;
use crate::design::ops;

impl Sim<'_, '_, '_> {
    /// The value of an expression.
    pub(super) fn eval(&mut self, cx: &Cx, x: ExprId) -> Const {
        let e = self.d.expr(x).clone();
        let ty = e.ty;
        let tys = &self.d.types;
        use ExprKind as K;
        match &e.kind {
            K::Const(c) => self.d.konst(*c).clone(),
            K::Object(o) => self.read_obj(cx, *o),
            K::Hier { object: Some(o), .. } => match self.hops(cx.inst, x) {
                Some(i) => {
                    let c2 = Cx { inst: i, locals: Rc::default(), this: None };
                    self.read_obj(&c2, *o)
                }
                None => self.read_obj(cx, *o),
            },
            K::Hier { .. } => Const::Null,
            K::TypeRef(t) => Const::Type(*t),
            K::Convert => {
                let v = self.eval(cx, e.args[0]);
                ops::convert(tys, &v, ty)
            }
            K::Unary(op @ (UnOp::PreInc | UnOp::PreDec | UnOp::PostInc | UnOp::PostDec)) => {
                let old = self.eval(cx, e.args[0]);
                let one = Const::Bits(Bits::from_i64(32, true, 1));
                let bop = if matches!(op, UnOp::PreInc | UnOp::PostInc) { BinOp::Add } else { BinOp::Sub };
                let new = if matches!(old, Const::Real(_)) {
                    ops::binary(tys, bop, old.clone(), Const::Real(1.0), ty)
                } else {
                    ops::binary(tys, bop, old.clone(), one, ty)
                }
                .unwrap_or(Const::Null);
                let t = self.target(cx, e.args[0]);
                self.write_cx(cx, &t, new.clone(), false);
                if matches!(op, UnOp::PreInc | UnOp::PreDec) { new } else { old }
            }
            K::Unary(op) => {
                let v = self.eval(cx, e.args[0]);
                ops::unary(tys, *op, v, ty).unwrap_or_else(|| ops::default_value(tys, ty))
            }
            K::Binary(op) => {
                if matches!(op, BinOp::LogAnd | BinOp::LogOr) {
                    let a = self.eval(cx, e.args[0]);
                    let at = truth(&a);
                    if *op == BinOp::LogAnd && at == Some(false) {
                        return Const::Bits(Bits::from_bool(false));
                    }
                    if *op == BinOp::LogOr && at == Some(true) {
                        return Const::Bits(Bits::from_bool(true));
                    }
                    let b = self.eval(cx, e.args[1]);
                    let bt = truth(&b);
                    let r = match (op, at, bt) {
                        (BinOp::LogAnd, _, Some(false)) => Logic::Zero,
                        (BinOp::LogAnd, Some(true), Some(true)) => Logic::One,
                        (BinOp::LogOr, _, Some(true)) => Logic::One,
                        (BinOp::LogOr, Some(false), Some(false)) => Logic::Zero,
                        _ => Logic::X,
                    };
                    return Const::Bits(Bits::of_logic(r));
                }
                let a = self.eval(cx, e.args[0]);
                let b = self.eval(cx, e.args[1]);
                // Handles and null compare by identity.
                if matches!(op, BinOp::Eq | BinOp::Ne | BinOp::CaseEq | BinOp::CaseNe)
                    && (matches!(a, Const::Handle(_) | Const::Null) || matches!(b, Const::Handle(_) | Const::Null))
                {
                    let eq = a == b;
                    let r = if matches!(op, BinOp::Eq | BinOp::CaseEq) { eq } else { !eq };
                    return Const::Bits(Bits::from_bool(r));
                }
                if matches!(op, BinOp::Eq | BinOp::Ne | BinOp::CaseEq | BinOp::CaseNe)
                    && matches!((&a, &b), (Const::Aggregate(_), Const::Aggregate(_)))
                {
                    let eq = ops::const_eq(&a, &b) || a == b;
                    let r = if matches!(op, BinOp::Eq | BinOp::CaseEq) { eq } else { !eq };
                    return Const::Bits(Bits::from_bool(r));
                }
                ops::binary(tys, *op, a, b, ty).unwrap_or_else(|| ops::default_value(tys, ty))
            }
            K::Cond => {
                let c = self.eval(cx, e.args[0]);
                match truth(&c) {
                    Some(true) => self.eval(cx, e.args[1]),
                    Some(false) => self.eval(cx, e.args[2]),
                    None => {
                        let a = self.eval(cx, e.args[1]);
                        let b = self.eval(cx, e.args[2]);
                        match (&a, &b) {
                            (Const::Bits(x), Const::Bits(y)) => {
                                let mut r = x.clone();
                                for i in 0..r.width() {
                                    if x.bit(i) != y.bit(i) || !x.bit(i).is_known() {
                                        r.set_bit(i, Logic::X);
                                    }
                                }
                                Const::Bits(r)
                            }
                            _ => a,
                        }
                    }
                }
            }
            K::Concat => self.concat(cx, &e),
            K::Replicate(k) => {
                let v = self.eval(cx, e.args[0]);
                let k = if *k == u32::MAX { self.eval(cx, e.args[1]).to_i64().unwrap_or(0).max(0) as usize } else { *k as usize };
                match v {
                    Const::Str(s) => Const::Str(s.repeat(k)),
                    Const::Bits(b) => Const::Bits(Bits::concat(&vec![b; k])),
                    other => other,
                }
            }
            K::Index => {
                let base = self.eval(cx, e.args[0]);
                let i = self.eval(cx, e.args[1]);
                let bt = self.d.expr(e.args[0]).ty;
                if let Type::Unpacked { dim: UDim::Assoc(_) | UDim::Wildcard, .. } = tys.get(bt) {
                    if let Const::Aggregate(items) = &base {
                        for c in items.chunks(2) {
                            if ops::const_eq(&c[0], &i) {
                                return c[1].clone();
                            }
                        }
                    }
                    return ops::default_value(tys, ty);
                }
                if let Type::Unpacked { dim: UDim::Dynamic | UDim::Queue(_), .. } = tys.get(bt) {
                    let k = i.to_i64().unwrap_or(-1);
                    if let Const::Aggregate(items) = &base
                        && k >= 0
                        && (k as usize) < items.len()
                    {
                        return items[k as usize].clone();
                    }
                    return ops::default_value(tys, ty);
                }
                ops::index(tys, &base, bt, &i, ty).unwrap_or_else(|| ops::default_value(tys, ty))
            }
            K::Part { kind, width } => self.part(cx, &e, *kind, *width),
            K::Member(i) => {
                let base = self.eval(cx, e.args[0]);
                let bt = self.d.expr(e.args[0]).ty;
                member(tys, base, bt, *i)
            }
            K::Field(o) => {
                let h = match e.args.first() {
                    Some(&b) => self.eval(cx, b),
                    None => cx.this.map_or(Const::Null, Const::Handle),
                };
                match (h, self.homes[o.0 as usize]) {
                    (Const::Handle(h), Home::Field(f)) => {
                        self.heap.get(h as usize).and_then(|ob| ob.fields.get(f as usize).cloned()).unwrap_or(Const::Null)
                    }
                    (_, Home::Global(g)) => self.read_slot(g as usize),
                    _ => ops::default_value(tys, ty),
                }
            }
            K::Pattern => {
                let mut out = Vec::new();
                for &a in &e.args {
                    out.push(self.eval(cx, a));
                }
                if tys.is_integral(ty) {
                    let elems: Vec<TypeId> = match tys.get(ty).clone() {
                        Type::Struct { members, .. } => members.iter().map(|m| m.ty).collect(),
                        _ => vec![],
                    };
                    let mut parts = Vec::new();
                    for (k, v) in out.iter().enumerate() {
                        let b = match v {
                            Const::Bits(b) => b.clone(),
                            Const::Real(r) => Bits::from_f64(64, true, *r),
                            _ => Bits::x(1),
                        };
                        let w = elems.get(k).map(|&t| tys.width(t)).unwrap_or(b.width());
                        parts.push(b.resize(w, b.signed()));
                    }
                    return Const::Bits(Bits::concat(&parts).resize(tys.width(ty), false));
                }
                Const::Aggregate(out)
            }
            K::Call(sub) => self.call_function(cx, *sub, &e.args, x),
            K::SysCall(s) => self.syscall(cx, *s, &e.args, ty, x),
            K::Method(m) => self.method(cx, *m, &e.args, ty, x),
            K::Inside => {
                let v = self.eval(cx, e.args[0]);
                let mut any_x = false;
                for &r in &e.args[1..] {
                    let re = self.d.expr(r).clone();
                    if re.kind == K::Range {
                        let lo = self.eval(cx, re.args[0]);
                        let hi = self.eval(cx, re.args[1]);
                        let ge = ops::binary(tys, BinOp::Ge, v.clone(), lo, Types::BIT);
                        let le = ops::binary(tys, BinOp::Le, v.clone(), hi, Types::BIT);
                        match (ge.as_ref().and_then(truth), le.as_ref().and_then(truth)) {
                            (Some(true), Some(true)) => return Const::Bits(Bits::from_bool(true)),
                            (None, _) | (_, None) => any_x = true,
                            _ => {}
                        }
                    } else {
                        let c = self.eval(cx, r);
                        if let Const::Aggregate(items) = &c {
                            if items.iter().any(|i| ops::const_eq(i, &v)) {
                                return Const::Bits(Bits::from_bool(true));
                            }
                            continue;
                        }
                        match (&v, &c) {
                            (Const::Bits(a), Const::Bits(b)) => match a.wildcard_eq(b) {
                                Logic::One => return Const::Bits(Bits::from_bool(true)),
                                Logic::X => any_x = true,
                                _ => {}
                            },
                            _ => {
                                if ops::const_eq(&v, &c) {
                                    return Const::Bits(Bits::from_bool(true));
                                }
                            }
                        }
                    }
                }
                Const::Bits(if any_x { Bits::x(1) } else { Bits::from_bool(false) })
            }
            K::New => self.new_value(cx, &e, x),
            K::Null => Const::Null,
            K::This => cx.this.map_or(Const::Null, Const::Handle),
            K::Assign => {
                let v = self.eval(cx, e.args[1]);
                let t = self.target(cx, e.args[0]);
                self.write_cx(cx, &t, v.clone(), false);
                v
            }
            K::Streaming { left } => self.stream(cx, &e, *left),
            K::Dollar => Const::Unbounded,
            K::Range | K::Tagged(_) | K::Opaque => {
                self.unsupported("an expression the elaborator keeps as syntax".into());
                ops::default_value(tys, ty)
            }
        }
    }

    pub(super) fn read_obj(&mut self, cx: &Cx, o: ObjId) -> Const {
        match self.homes[o.0 as usize] {
            Home::Const => match self.d.object(o).value {
                Some(c) => self.d.konst(c).clone(),
                None => Const::Null,
            },
            Home::Local => self.read_local(cx, o),
            Home::Field(f) => match cx.this {
                Some(h) => self.heap.get(h as usize).and_then(|ob| ob.fields.get(f as usize).cloned()).unwrap_or(Const::Null),
                None => Const::Null,
            },
            _ => match self.slot_of_obj(cx.inst, o) {
                Some(s) => self.read_slot(s),
                None => ops::default_value(self.types(), self.d.object(o).ty),
            },
        }
    }

    fn concat(&mut self, cx: &Cx, e: &Expr) -> Const {
        let tys = &self.d.types;
        if e.ty == Types::STRING {
            let mut s = Vec::new();
            for &a in &e.args {
                match self.eval(cx, a) {
                    Const::Str(x) => s.extend(x),
                    Const::Bits(b) => s.extend(b.to_str_bytes()),
                    _ => {}
                }
            }
            return Const::Str(s);
        }
        if tys.is_unpacked_array(e.ty) {
            let mut out = Vec::new();
            for &a in &e.args {
                let at = self.d.expr(a).ty;
                match self.eval(cx, a) {
                    Const::Aggregate(v) if self.d.types.is_unpacked_array(at) => out.extend(v),
                    v => out.push(v),
                }
            }
            return Const::Aggregate(out);
        }
        let mut parts = Vec::new();
        for &a in &e.args {
            let at = self.d.expr(a).ty;
            let w = self.d.types.width(at);
            let b = match self.eval(cx, a) {
                Const::Bits(b) => b,
                Const::Str(s) => Bits::from_str_bytes(&s),
                Const::Real(r) => Bits::from_f64(64, true, r),
                _ => Bits::x(w.max(1)),
            };
            parts.push(if w > 0 { b.resize(w, b.signed()) } else { b });
        }
        Const::Bits(Bits::concat(&parts))
    }

    fn part(&mut self, cx: &Cx, e: &Expr, kind: PartKind, width: u32) -> Const {
        let base = self.eval(cx, e.args[0]);
        let a = self.eval(cx, e.args[1]).to_i64();
        let bt = self.d.expr(e.args[0]).ty;
        let tys = &self.d.types;
        let Some(a) = a else { return ops::default_value(tys, e.ty) };
        let (hi, lo) = match kind {
            PartKind::Range => (a, self.eval(cx, e.args[2]).to_i64().unwrap_or(0)),
            PartKind::Up => (a + width as i64 - 1, a),
            PartKind::Down => (a, a - width as i64 + 1),
        };
        let tys = &self.d.types;
        match &base {
            Const::Aggregate(items) => {
                let (l, r) = match tys.get(bt) {
                    Type::Unpacked { dim: UDim::Range(l, r), .. } => (*l, *r),
                    _ => (0, items.len() as i64 - 1),
                };
                let off = |i: i64| if l <= r { i - l } else { l - i };
                let (x, y) = (off(hi).min(off(lo)), off(hi).max(off(lo)));
                Const::Aggregate((x..=y).filter_map(|i| items.get(i.max(0) as usize).cloned()).collect())
            }
            Const::Str(s) => {
                let (x, y) = (lo.min(hi).max(0) as usize, hi.max(lo).max(0) as usize);
                Const::Str(s.get(x..=y.min(s.len().saturating_sub(1))).map_or(vec![], |v| v.to_vec()))
            }
            Const::Bits(b) => {
                let ew = tys.elem_width(bt).max(1) as i64;
                match ops::lsb_offset(tys, bt, lo.min(hi), hi.max(lo)) {
                    Some(off) => Const::Bits(b.slice((off * ew) as u32, width * ew as u32)),
                    None => {
                        // Partly out of range: the bits that are in range,
                        // x for the rest (11.5.1).
                        Const::Bits(Bits::x(width * ew as u32))
                    }
                }
            }
            _ => ops::default_value(tys, e.ty),
        }
    }

    fn stream(&mut self, cx: &Cx, e: &Expr, left: bool) -> Const {
        // `{>> {a, b}}` packs left to right; `{<< n {...}}` reverses the
        // order of n-bit slices (11.4.14).
        let mut args = e.args.clone();
        let mut slice = 1u32;
        if let Some(&first) = args.first()
            && !matches!(self.d.expr(first).kind, ExprKind::Object(_) | ExprKind::Hier { .. } | ExprKind::Index | ExprKind::Part { .. } | ExprKind::Member(_) | ExprKind::Field(_) | ExprKind::Concat)
            && args.len() > 1
        {
            let v = self.eval(cx, first);
            slice = match v {
                Const::Type(t) => self.d.types.width(t).max(1),
                v => v.to_i64().unwrap_or(1).max(1) as u32,
            };
            args.remove(0);
        }
        let mut parts = Vec::new();
        for a in args {
            match self.eval(cx, a) {
                Const::Bits(b) => parts.push(b),
                Const::Aggregate(items) => {
                    for it in items {
                        if let Const::Bits(b) = it {
                            parts.push(b);
                        }
                    }
                }
                Const::Str(s) => parts.push(Bits::from_str_bytes(&s)),
                _ => {}
            }
        }
        let all = Bits::concat(&parts);
        if !left {
            return Const::Bits(all);
        }
        let w = all.width();
        let mut out = Bits::zero(w, false);
        let n = w.div_ceil(slice);
        for k in 0..n {
            // Slice k from the top goes to slice k from the bottom.
            let hi = w as i64 - (k * slice) as i64;
            let lo = (hi - slice as i64).max(0);
            let piece = all.slice(lo as u32, (hi - lo) as u32);
            out.set_slice(k * slice, &piece);
        }
        // Slices of a width that does not divide are kept at the top.
        Const::Bits(out)
    }

    // ---- targets ----

    /// Where an assignment to `x` writes, its indices evaluated now.
    pub(super) fn target(&mut self, cx: &Cx, x: ExprId) -> Target {
        let e = self.d.expr(x).clone();
        let tys = &self.d.types;
        match &e.kind {
            ExprKind::Object(o) => self.target_obj(cx, *o),
            ExprKind::Hier { object: Some(o), .. } => match self.hops(cx.inst, x) {
                Some(i) => {
                    let c2 = Cx { inst: i, locals: cx.locals.clone(), this: cx.this };
                    self.target_obj(&c2, *o)
                }
                None => self.target_obj(cx, *o),
            },
            ExprKind::Field(o) => {
                let h = match e.args.first() {
                    Some(&b) => self.eval(cx, b),
                    None => cx.this.map_or(Const::Null, Const::Handle),
                };
                let place = match (h, self.homes[o.0 as usize]) {
                    (Const::Handle(h), Home::Field(f)) => Place::Field(h, f),
                    (_, Home::Global(g)) => Place::Slot(g as usize),
                    _ => Place::None,
                };
                Target { place, path: vec![], ty: e.ty }
            }
            ExprKind::Convert => {
                let mut t = self.target(cx, e.args[0]);
                t.ty = self.d.expr(e.args[0]).ty;
                t
            }
            ExprKind::Index => {
                let mut t = self.target(cx, e.args[0]);
                let i = self.eval(cx, e.args[1]);
                let bt = self.d.expr(e.args[0]).ty;
                let tys = &self.d.types;
                let sel = match tys.get(bt) {
                    Type::Unpacked { dim: UDim::Assoc(_) | UDim::Wildcard, .. } => Some(Sel::Key(i)),
                    Type::Unpacked { dim: UDim::Range(l, r), .. } => i.to_i64().and_then(|k| {
                        let off = if l <= r { k - l } else { l - k };
                        (off >= 0 && off <= (l - r).abs()).then_some(Sel::Elem(off as usize))
                    }),
                    Type::Unpacked { .. } => i.to_i64().filter(|&k| k >= 0).map(|k| Sel::Elem(k as usize)),
                    _ => {
                        let ew = tys.width(e.ty).max(1);
                        i.to_i64().and_then(|k| ops::lsb_offset(tys, bt, k, k)).map(|off| Sel::Bits(off as u32 * ew, ew))
                    }
                };
                match sel {
                    Some(s) => {
                        // Growing a dynamic array or queue by writing past
                        // its end is not allowed; the write is dropped.
                        t.path.push(s);
                        t.ty = e.ty;
                    }
                    None => t.place = Place::None,
                }
                t
            }
            ExprKind::Part { kind, width } => {
                let mut t = self.target(cx, e.args[0]);
                let a = self.eval(cx, e.args[1]).to_i64();
                let bt = self.d.expr(e.args[0]).ty;
                let Some(a) = a else {
                    t.place = Place::None;
                    return t;
                };
                let (hi, lo) = match kind {
                    PartKind::Range => (a, self.eval(cx, e.args[2]).to_i64().unwrap_or(0)),
                    PartKind::Up => (a + *width as i64 - 1, a),
                    PartKind::Down => (a, a - *width as i64 + 1),
                };
                let tys = &self.d.types;
                let ew = tys.elem_width(bt).max(1);
                match ops::lsb_offset(tys, bt, lo.min(hi), hi.max(lo)) {
                    Some(off) => {
                        t.path.push(Sel::Bits(off as u32 * ew, *width * ew));
                        t.ty = e.ty;
                    }
                    None => t.place = Place::None,
                }
                t
            }
            ExprKind::Member(i) => {
                let mut t = self.target(cx, e.args[0]);
                let bt = self.d.expr(e.args[0]).ty;
                match tys.get(bt).clone() {
                    Type::Struct { packed: true, members, union, .. } => {
                        let w = tys.width(members[*i as usize].ty);
                        let below: u32 = if union { 0 } else { members[*i as usize + 1..].iter().map(|m| tys.width(m.ty)).sum() };
                        t.path.push(Sel::Bits(below, w));
                    }
                    _ => t.path.push(Sel::Member(*i)),
                }
                t.ty = e.ty;
                t
            }
            _ => Target { place: Place::None, path: vec![], ty: e.ty },
        }
    }

    // ---- case and foreach ----

    pub(super) fn case_target(&mut self, cx: &Cx, s: StmtId, targets: &[u32], dflt: u32) -> u32 {
        let StmtKind::Case { kind, expr, items, .. } = self.d.stmt(s).kind.clone() else { return dflt };
        let v = self.eval(cx, expr);
        for ((xs, _), &t) in items.iter().zip(targets) {
            for &x in xs {
                let c = self.eval(cx, x);
                let hit = match (&v, &c) {
                    (Const::Bits(a), Const::Bits(b)) => match kind {
                        CaseKind::Case => a.case_eq(b),
                        CaseKind::CaseZ | CaseKind::CaseX => {
                            let w = a.width().max(b.width());
                            let (a, b) = (a.extend(w), b.extend(w));
                            (0..w).all(|i| {
                                let (p, q) = (a.bit(i), b.bit(i));
                                let dont = |l: Logic| l == Logic::Z || (kind == CaseKind::CaseX && l == Logic::X);
                                dont(p) || dont(q) || p == q
                            })
                        }
                        CaseKind::Inside => {
                            if let ExprKind::Range = self.d.expr(x).kind {
                                false
                            } else {
                                a.wildcard_eq(b) == Logic::One
                            }
                        }
                    },
                    _ => ops::const_eq(&v, &c),
                };
                let hit = hit
                    || (kind == CaseKind::Inside && matches!(self.d.expr(x).kind, ExprKind::Range) && {
                        let r = self.d.expr(x).clone();
                        let lo = self.eval(cx, r.args[0]);
                        let hi = self.eval(cx, r.args[1]);
                        let tys = &self.d.types;
                        let ge = ops::binary(tys, BinOp::Ge, v.clone(), lo, Types::BIT).as_ref().and_then(truth);
                        let le = ops::binary(tys, BinOp::Le, v.clone(), hi, Types::BIT).as_ref().and_then(truth);
                        ge == Some(true) && le == Some(true)
                    });
                if hit {
                    return t;
                }
            }
        }
        dflt
    }

    /// Every combination of the loop variables of a `foreach` (12.7.3).
    pub(super) fn foreach_indices(&mut self, cx: &Cx, s: StmtId) -> Vec<Vec<i64>> {
        let StmtKind::Foreach { array, vars, .. } = self.d.stmt(s).kind.clone() else { return vec![] };
        let v = self.eval(cx, array);
        let at = self.d.expr(array).ty;
        let tys = &self.d.types;
        // The ranges, outermost first, as far as there are variables.
        let mut ranges: Vec<Vec<i64>> = Vec::new();
        let mut t = at;
        let mut val = Some(v);
        for _ in 0..vars.len() {
            match tys.get(t).clone() {
                Type::Unpacked { elem, dim } => {
                    let r: Vec<i64> = match dim {
                        UDim::Range(l, r) => {
                            if l <= r {
                                (l..=r).collect()
                            } else {
                                (r..=l).rev().collect()
                            }
                        }
                        UDim::Assoc(_) | UDim::Wildcard => match &val {
                            Some(Const::Aggregate(items)) => items.chunks(2).filter_map(|c| c[0].to_i64()).collect(),
                            _ => vec![],
                        },
                        _ => match &val {
                            Some(Const::Aggregate(items)) => (0..items.len() as i64).collect(),
                            _ => vec![],
                        },
                    };
                    val = match &val {
                        Some(Const::Aggregate(items)) if !matches!(dim, UDim::Assoc(_) | UDim::Wildcard) => items.first().cloned(),
                        _ => None,
                    };
                    ranges.push(r);
                    t = elem;
                }
                Type::String => {
                    let n = match &val {
                        Some(Const::Str(s)) => s.len() as i64,
                        _ => 0,
                    };
                    ranges.push((0..n).collect());
                }
                _ => match tys.packed_range(t) {
                    Some((l, r)) => {
                        ranges.push(if l >= r { (r..=l).rev().collect() } else { (l..=r).collect() });
                        t = tys.element_of(t).unwrap_or(t);
                    }
                    None => ranges.push(vec![]),
                },
            }
        }
        let mut out: Vec<Vec<i64>> = vec![vec![]];
        for r in ranges {
            let mut next = Vec::new();
            for prefix in &out {
                for &i in &r {
                    let mut p = prefix.clone();
                    p.push(i);
                    next.push(p);
                }
            }
            out = next;
        }
        if vars.is_empty() { vec![] } else { out }
    }

    // ---- calls ----

    /// Binds a call's arguments into a new frame's locals.
    fn bind_args(&mut self, cx: &Cx, sub: SubId, args: &[ExprId]) -> (Locals, Option<u32>, Vec<ExprId>) {
        let s = self.d.sub(sub);
        let formals = s.args.clone();
        let (this, actuals) = if args.len() == formals.len() + 1 {
            let h = match self.eval(cx, args[0]) {
                Const::Handle(h) => Some(h),
                _ => cx.this,
            };
            (h, args[1..].to_vec())
        } else {
            (cx.this, args.to_vec())
        };
        let locals: Locals = Rc::default();
        for (i, &f) in formals.iter().enumerate() {
            let dir = self.d.object(f).dir.unwrap_or(Dir::Input);
            let v = match (dir, actuals.get(i)) {
                (Dir::Output, _) | (_, None) => ops::default_value(self.types(), self.d.object(f).ty),
                (_, Some(&a)) => self.eval(cx, a),
            };
            let v = ops::convert(self.types(), &v, self.d.object(f).ty);
            locals.borrow_mut().push((f, v));
        }
        (locals, this, actuals)
    }

    /// Copies output, inout and ref arguments back to the caller.
    fn copy_out(&mut self, cx: &Cx, sub: SubId, locals: &Locals, actuals: &[ExprId]) {
        let formals = self.d.sub(sub).args.clone();
        for (i, &f) in formals.iter().enumerate() {
            let dir = self.d.object(f).dir.unwrap_or(Dir::Input);
            if matches!(dir, Dir::Output | Dir::Inout | Dir::Ref)
                && let Some(&a) = actuals.get(i)
            {
                let v = locals.borrow().iter().find(|(k, _)| *k == f).map(|(_, v)| v.clone());
                if let Some(v) = v {
                    let t = self.target(cx, a);
                    self.write_cx(cx, &t, v, false);
                }
            }
        }
    }

    /// The method a call reaches on an object of class `c`: the most
    /// derived override of a virtual method (8.20).
    fn dispatch(&self, sub: SubId, this: Option<u32>, x: ExprId) -> SubId {
        let s = self.d.sub(sub);
        let Some(h) = this else { return sub };
        // `super.m()` calls the base method itself.
        let node = self.d.expr(x).node;
        if self.ast.tokens_of(node).first().is_some_and(|t| t.0 == "super") {
            return sub;
        }
        let Some(obj) = self.heap.get(h as usize) else { return sub };
        let mut c = Some(obj.class);
        while let Some(k) = c {
            let sc = self.d.class(k).scope;
            if let Some(Binding::Subroutine(m)) = self.d.scope(sc).names.get(&s.name)
                && self.d.sub(*m).node != crate::ast::NodeId(u32::MAX)
            {
                if *m == sub || self.is_virtual(sub) {
                    return *m;
                }
                return sub;
            }
            c = self.d.class(k).base;
        }
        sub
    }

    fn is_virtual(&self, sub: SubId) -> bool {
        let s = self.d.sub(sub);
        if s.qualifiers.iter().any(|&q| matches!(self.d.name(q), "virtual" | "pure")) {
            return true;
        }
        // An override of a virtual method is virtual.
        let class = self.d.classes.iter().position(|c| self.d.scope(s.scope).parent == Some(c.scope));
        let mut c = class.and_then(|c| self.d.classes[c].base);
        while let Some(k) = c {
            let sc = self.d.class(k).scope;
            if let Some(Binding::Subroutine(m)) = self.d.scope(sc).names.get(&s.name) {
                return self.is_virtual(*m);
            }
            c = self.d.class(k).base;
        }
        false
    }

    /// A function call in an expression: run to its return.
    pub(super) fn call_function(&mut self, cx: &Cx, sub: SubId, args: &[ExprId], x: ExprId) -> Const {
        let (locals, this, actuals) = self.bind_args(cx, sub, args);
        let sub = self.dispatch(sub, this, x);
        let s = self.d.sub(sub);
        if s.qualifiers.iter().any(|&q| self.d.name(q) == "import") {
            self.unsupported(format!("DPI import `{}`", self.d.name(s.name)));
            return ops::default_value(self.types(), s.ret);
        }
        let Some(&entry) = self.sub_entry.get(&sub) else {
            return ops::default_value(self.types(), s.ret);
        };
        // The function's own name holds its result (13.4.1).
        let ret_ty = s.ret;
        let result_obj = match self.d.scope(s.scope).names.get(&s.name) {
            Some(Binding::Object(o)) => Some(*o),
            _ => None,
        };
        if let Some(r) = result_obj {
            locals.borrow_mut().push((r, ops::default_value(self.types(), ret_ty)));
        }
        let inst = self.def_inst(cx.inst, s.scope);
        let fcx = Cx { inst, locals: locals.clone(), this };
        let v = self.run_call(fcx, entry, sub);
        self.copy_out(cx, sub, &locals, &actuals);
        let v = match v {
            Some(v) => v,
            None => match result_obj {
                Some(r) => locals.borrow().iter().rev().find(|(k, _)| *k == r).map(|(_, v)| v.clone()).unwrap_or(Const::Null),
                None => Const::Null,
            },
        };
        if ret_ty == Types::VOID { Const::Null } else { ops::convert(self.types(), &v, ret_ty) }
    }

    /// The instance a subroutine runs in: the caller's, or for one
    /// declared in a design element above it, that ancestor.
    fn def_inst(&self, inst: u32, scope: ScopeId) -> u32 {
        let mut cur = Some(scope);
        while let Some(sc) = cur {
            if self.d.scope(sc).kind == ScopeKind::Spec {
                let spec = self.d.specs.iter().position(|s| s.scope == sc).map(|i| SpecId(i as u32));
                return spec.and_then(|sp| self.inst_for(inst, sp)).unwrap_or(inst);
            }
            if matches!(self.d.scope(sc).kind, ScopeKind::Package | ScopeKind::Unit) {
                return inst;
            }
            cur = self.d.scope(sc).parent;
        }
        inst
    }

    /// Runs a function body in a process of its own until it returns. A
    /// function cannot wait, so this never suspends.
    fn run_call(&mut self, cx: Cx, entry: u32, sub: SubId) -> Option<Const> {
        let id = self.procs.len() as u32;
        self.procs.push(Proc {
            pc: entry,
            frames: vec![Frame { cx, ret: None, sub: Some(sub), counters: HashMap::new(), iters: HashMap::new(), ret_value: None }],
            state: State::Ready,
            epoch: 0,
            wait: Wait::None,
            parent: None,
            children: vec![],
            sensitive: vec![],
            driver: None,
        });
        loop {
            if self.over_limit() {
                break;
            }
            let pc = self.procs[id as usize].pc;
            let Some(op) = self.code.get(pc as usize).cloned() else { break };
            self.procs[id as usize].pc = pc + 1;
            match self.exec(id, pc, op) {
                Flow::Next => {}
                Flow::Suspend => {
                    // A function that waits: not allowed (13.4); stop it.
                    self.procs[id as usize].state = State::Done;
                    break;
                }
                Flow::Done => break,
            }
            if self.procs[id as usize].state == State::Done {
                break;
            }
        }
        let v = self.procs[id as usize].frames.first_mut().and_then(|f| f.ret_value.take());
        // The process slot is recycled when it is the last.
        if self.procs.len() as u32 == id + 1 {
            self.procs.pop();
        } else {
            self.procs[id as usize].state = State::Done;
        }
        v
    }

    /// A task call statement: a new frame on the process, so the task can
    /// wait.
    pub(super) fn call_task(&mut self, p: u32, cx: &Cx, x: ExprId) -> Flow {
        let e = self.d.expr(x).clone();
        let ExprKind::Call(sub) = e.kind else { return Flow::Next };
        let (locals, this, _) = self.bind_args(cx, sub, &e.args);
        let sub = self.dispatch(sub, this, x);
        let Some(&entry) = self.sub_entry.get(&sub) else {
            let n = self.d.sub(sub).name;
            self.unsupported(format!("task `{}` has no body here", self.d.name(n)));
            return Flow::Next;
        };
        let inst = self.def_inst(cx.inst, self.d.sub(sub).scope);
        let ret = self.procs[p as usize].pc;
        let frame = Frame {
            cx: Cx { inst, locals, this },
            ret: Some((ret, x)),
            sub: Some(sub),
            counters: HashMap::new(),
            iters: HashMap::new(),
            ret_value: None,
        };
        self.procs[p as usize].frames.push(frame);
        self.procs[p as usize].pc = entry;
        Flow::Next
    }

    /// Return from a task (to the caller's next op) or a function (ends
    /// its evaluation).
    pub(super) fn ret(&mut self, p: u32, cx: &Cx, v: Option<ExprId>) -> Flow {
        let value = v.map(|x| self.eval(cx, x));
        let frames = self.procs[p as usize].frames.len();
        let top = self.procs[p as usize].frames.last().unwrap();
        match top.ret {
            Some((ret_pc, call)) if frames > 1 => {
                let f = self.procs[p as usize].frames.pop().unwrap();
                let caller = self.cx(p);
                let ExprKind::Call(sub) = self.d.expr(call).kind else { return Flow::Next };
                let args = self.d.expr(call).args.clone();
                let actuals = if args.len() == self.d.sub(sub).args.len() + 1 { args[1..].to_vec() } else { args };
                let sub = f.sub.unwrap_or(sub);
                self.copy_out(&caller, sub, &f.cx.locals, &actuals);
                self.procs[p as usize].pc = ret_pc;
                Flow::Next
            }
            _ => {
                if let Some(f) = self.procs[p as usize].frames.last_mut() {
                    f.ret_value = value;
                }
                if self.procs[p as usize].frames.last().is_some_and(|f| f.sub.is_some()) {
                    // A function's body finished: stop its evaluation.
                    self.procs[p as usize].state = State::Done;
                    Flow::Suspend
                } else {
                    Flow::Done
                }
            }
        }
    }

    // ---- classes ----

    /// `new`, `new[n]` and `new(args)` (7.5.1, 8.7).
    fn new_value(&mut self, cx: &Cx, e: &Expr, _x: ExprId) -> Const {
        let tys = &self.d.types;
        match tys.get(e.ty).clone() {
            Type::Unpacked { elem, dim: UDim::Dynamic } => {
                let n = e.args.first().map_or(0, |&a| self.eval(cx, a).to_i64().unwrap_or(0).max(0) as usize);
                let mut items = vec![ops::default_value(self.types(), elem); n];
                if let Some(&init) = e.args.get(1)
                    && let Const::Aggregate(old) = self.eval(cx, init)
                {
                    for (i, v) in old.into_iter().enumerate().take(n) {
                        items[i] = v;
                    }
                }
                Const::Aggregate(items)
            }
            Type::Class(c) => {
                if self.d.class(c).node == self.ast.root {
                    // A built-in class: an empty object stands for it.
                    let h = self.heap.len() as u32;
                    self.heap.push(Object { class: c, fields: vec![] });
                    return Const::Handle(h);
                }
                let h = self.alloc(c);
                // The constructor call, if the class has one.
                match e.args.first().map(|&a| (a, self.d.expr(a).kind.clone())) {
                    Some((call, ExprKind::Call(ctor))) => {
                        let args = self.d.expr(call).args.clone();
                        self.construct(cx, c, h, Some((ctor, args)));
                    }
                    _ => self.construct(cx, c, h, None),
                }
                Const::Handle(h)
            }
            _ => Const::Null,
        }
    }

    /// A new object with every property at its default.
    fn alloc(&mut self, c: ClassId) -> u32 {
        let fields: Vec<Const> =
            self.class_fields[&c].clone().iter().map(|&f| ops::default_value(self.types(), self.d.object(f).ty)).collect();
        let h = self.heap.len() as u32;
        self.heap.push(Object { class: c, fields });
        h
    }

    /// Runs initialisers and constructors from the base down (8.7, 8.17).
    fn construct(&mut self, cx: &Cx, c: ClassId, h: u32, call: Option<(SubId, Vec<ExprId>)>) {
        let class = self.d.class(c);
        let base = class.base;
        let new = self.d.syms.lookup("new");
        let own_ctor = new.and_then(|n| match self.d.scope(class.scope).names.get(&n) {
            Some(Binding::Subroutine(s)) => Some(*s),
            _ => None,
        });
        // The base part: explicitly by `super.new` in the constructor's
        // first statement, else here, with the `extends` arguments or none.
        let calls_super = own_ctor.is_some_and(|s| {
            self.d.sub(s).body.first().is_some_and(|&st| {
                let n = self.d.stmt(st).node;
                let toks = self.ast.tokens_of(n);
                toks.len() > 2 && toks[0].0 == "super" && toks[2].0 == "new"
            })
        });
        if let Some(b) = base
            && !calls_super
            && self.d.class(b).node != self.ast.root
        {
            self.construct(cx, b, h, None);
        }
        // This class's property initialisers.
        let ocx = Cx { inst: cx.inst, locals: Rc::default(), this: Some(h) };
        for item in self.d.scope(class.scope).items.clone() {
            if let Item::Object(o) = item
                && let Some(init) = self.d.object(o).init
                && let Home::Field(f) = self.homes[o.0 as usize]
            {
                let v = self.eval(&ocx, init);
                let v = ops::convert(self.types(), &v, self.d.object(o).ty);
                if let Some(slot) = self.heap[h as usize].fields.get_mut(f as usize) {
                    *slot = v;
                }
            }
        }
        if let Some(ctor) = own_ctor {
            let args = match &call {
                Some((s, a)) if *s == ctor => a.clone(),
                _ => vec![],
            };
            let (locals, _, actuals) = self.bind_args(cx, ctor, &args);
            let Some(&entry) = self.sub_entry.get(&ctor) else { return };
            let fcx = Cx { inst: cx.inst, locals: locals.clone(), this: Some(h) };
            self.run_call(fcx, entry, ctor);
            self.copy_out(cx, ctor, &locals, &actuals);
        }
    }

    // ---- drivers ----

    /// A continuous driver: compute and write, then wait for its inputs.
    pub(super) fn drive(&mut self, p: u32, dr: Driven) {
        let inst = self.procs[p as usize].frames[0].cx.inst;
        let cx = Cx { inst, locals: Rc::default(), this: None };
        let mut reads: Vec<usize> = Vec::new();
        match dr {
            Driven::Assign(id) => {
                let d = &self.d.drivers[id.0 as usize];
                let (lhs, rhs, delay) = (d.lhs, d.rhs, d.delay.clone());
                let v = self.eval(&cx, rhs);
                let t = self.target(&cx, lhs);
                if let Some(&dl) = delay.first() {
                    let dv = self.eval(&cx, dl);
                    let ticks = self.ticks(inst, &dv);
                    if ticks > 0 {
                        self.after(ticks, Timed::Nba(t, v, cx.clone()));
                    } else {
                        self.write_cx(&cx, &t, v, false);
                    }
                } else {
                    self.write_cx(&cx, &t, v, false);
                }
                self.reads(&cx, rhs, &mut reads);
                for &a in self.d.expr(lhs).args.clone().iter().skip(1) {
                    self.reads(&cx, a, &mut reads);
                }
            }
            Driven::Port(child, k) => {
                let Some(parent) = self.insts[child as usize].parent else { return };
                let elab = self.insts[child as usize].elab;
                let spec = self.insts[child as usize].spec;
                let conn = self.d.instance(elab).connections[k].clone();
                let Some(port) = self.d.spec(spec).ports.iter().find(|q| q.name == conn.port).cloned() else { return };
                let (Some(x), Some(po)) = (conn.expr, port.object) else { return };
                let pcx = Cx { inst: parent, locals: Rc::default(), this: None };
                let ccx = Cx { inst: child, locals: Rc::default(), this: None };
                if port.dir == Dir::Input {
                    let v = self.eval(&pcx, x);
                    let t = self.target_obj(&ccx, po);
                    self.write_cx(&ccx, &t, v, false);
                    self.reads(&pcx, x, &mut reads);
                } else {
                    let v = self.read_obj(&ccx, po);
                    let t = self.target(&pcx, x);
                    self.write_cx(&pcx, &t, v, false);
                    if let Some(s) = self.slot_of_obj(child, po) {
                        reads.push(s);
                    }
                }
            }
            Driven::Cell(c) => {
                let cell = &self.d.cells[c.0 as usize];
                let kind = self.d.name(cell.kind).to_string();
                let terms = cell.terminals.clone();
                let ins: Vec<Const> = terms.iter().skip(1).map(|&t| self.eval(&cx, t)).collect();
                if let Some(v) = gate(&kind, &ins) {
                    if let Some(&out) = terms.first() {
                        let t = self.target(&cx, out);
                        self.write_cx(&cx, &t, v, false);
                    }
                } else {
                    self.unsupported(format!("gate or primitive `{kind}`"));
                }
                for &t in terms.iter().skip(1) {
                    self.reads(&cx, t, &mut reads);
                }
            }
        }
        let pr = &mut self.procs[p as usize];
        pr.state = State::Waiting;
        pr.epoch = pr.epoch.wrapping_add(1);
        pr.wait = Wait::Change;
        for s in reads {
            self.watch_slot(p, s);
        }
    }

    // ---- events, procedural continuous assignments ----

    pub(super) fn trigger(&mut self, cx: &Cx, event: ExprId) {
        let mut slots = Vec::new();
        self.reads(cx, event, &mut slots);
        for s in slots {
            let s = self.find(s);
            let n = match &self.vals[s] {
                Const::Bits(b) => b.to_u64().unwrap_or(0),
                _ => 0,
            };
            self.vals[s] = Const::Bits(Bits::from_u64(32, false, n.wrapping_add(1)));
            self.triggered.insert(s, self.time);
            self.changed(s);
        }
    }

    pub(super) fn proc_cont(&mut self, cx: &Cx, s: StmtId) {
        let StmtKind::ProcContinuous { keyword, lhs, rhs } = self.d.stmt(s).kind.clone() else { return };
        if let ("assign" | "force", Some(r)) = (self.d.name(keyword), rhs) {
            // Held only until the next write, not continuously: the
            // full semantics of 10.6 are not modelled yet.
            let v = self.eval(cx, r);
            let t = self.target(cx, lhs);
            self.write_cx(cx, &t, v, false);
        }
    }
}

/// A struct member of a value.
fn member(tys: &Types, base: Const, bt: TypeId, i: u32) -> Const {
    match base {
        Const::Aggregate(v) => v.get(i as usize).cloned().unwrap_or(Const::Null),
        Const::Bits(b) => {
            let Type::Struct { members, union, .. } = tys.get(bt).clone() else { return Const::Bits(b) };
            let m = &members[i as usize];
            let w = tys.width(m.ty);
            let below: u32 = if union { 0 } else { members[i as usize + 1..].iter().map(|m| tys.width(m.ty)).sum() };
            Const::Bits(b.slice(below, w).with_signed(tys.is_signed(m.ty)))
        }
        other => other,
    }
}

/// A gate's output from its inputs (28.4–28.7).
fn gate(kind: &str, ins: &[Const]) -> Option<Const> {
    let bit = |c: &Const| match c {
        Const::Bits(b) => b.bit(0),
        _ => Logic::X,
    };
    let b = |l: Logic| Some(Const::Bits(Bits::of_logic(l)));
    let fold = |f: fn(&Bits, &Bits) -> Bits| {
        let mut acc = Bits::of_logic(bit(&ins[0]));
        for i in &ins[1..] {
            acc = f(&acc, &Bits::of_logic(bit(i)));
        }
        acc.bit(0)
    };
    let not = |l: Logic| match l {
        Logic::Zero => Logic::One,
        Logic::One => Logic::Zero,
        _ => Logic::X,
    };
    match kind {
        "and" => b(fold(Bits::and)),
        "nand" => b(not(fold(Bits::and))),
        "or" => b(fold(Bits::or)),
        "nor" => b(not(fold(Bits::or))),
        "xor" => b(fold(Bits::xor)),
        "xnor" => b(not(fold(Bits::xor))),
        "buf" => b(match bit(&ins[0]) {
            Logic::Z => Logic::X,
            l => l,
        }),
        "not" => b(not(bit(&ins[0]))),
        "bufif1" | "bufif0" | "notif1" | "notif0" => {
            let (d, c) = (bit(&ins[0]), bit(ins.get(1)?));
            let on = if kind.ends_with('1') { c == Logic::One } else { c == Logic::Zero };
            let off = if kind.ends_with('1') { c == Logic::Zero } else { c == Logic::One };
            let v = if kind.starts_with("not") { not(d) } else if d == Logic::Z { Logic::X } else { d };
            b(if on {
                v
            } else if off {
                Logic::Z
            } else {
                Logic::X
            })
        }
        "pullup" => b(Logic::One),
        "pulldown" => b(Logic::Zero),
        _ => None,
    }
}
