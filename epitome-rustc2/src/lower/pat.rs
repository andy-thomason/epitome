//! Patterns, and enums' variants.
//!
//! A pattern is matched against a place: each test branches away when it
//! fails, and bindings are made as the match goes (a binding made before
//! a later test fails is never used). A `match` tries its arms in order,
//! which is correct but tests the same things again where arms overlap;
//! a decision tree that tests each column once is the plan.

use super::Repr;
use super::body::{Fx, Operand, Place, R, Val};
use crate::ast::{Kind, NodeId};
use crate::layout::{TagEncoding, Variants};
use crate::resolve::{DefKind, Res};
use crate::ty::{AdtKind, TyKind};
use crate::typeck::BindingMode;
use epitome_ir::module::{BlockId, ICmpPred, Value};

pub(crate) fn is_pattern(k: Kind) -> bool {
    use Kind::*;
    matches!(
        k,
        WildPat
            | RestPat
            | IdentPat
            | LitPat
            | RangePat
            | RefPat
            | TuplePat
            | ParenPat
            | SlicePat
            | PathPat
            | TupleStructPat
            | StructPat
            | OrPat
            | BoxPat
            | NeverPat
            | PatMacro
    )
}

pub(crate) fn is_type(k: Kind) -> bool {
    use Kind::*;
    matches!(
        k,
        PathType
            | RefType
            | PtrType
            | ArrayType
            | SliceType
            | TupleType
            | ParenType
            | PatternType
            | NeverType
            | InferType
            | FnPtrType
            | ImplTraitType
            | DynTraitType
            | TypeMacro
    )
}

pub(crate) fn is_expr(k: Kind) -> bool {
    use Kind::*;
    matches!(
        k,
        Literal
            | PathExpr
            | Binary
            | Unary
            | Borrow
            | Cast
            | Range
            | Call
            | MethodCall
            | Field
            | Index
            | Try
            | Await
            | UseExpr
            | Tuple
            | Paren
            | Array
            | ArrayRepeat
            | StructExpr
            | Block
            | If
            | Let
            | While
            | Loop
            | For
            | Match
            | Closure
            | Break
            | Continue
            | Return
            | Yield
            | Become
            | Yeet
            | Underscore
            | ConstBlock
            | Builtin
            | AttrExpr
            | MacroCall
    )
}

impl Fx<'_, '_, '_, '_> {
    // ---- variants ----

    /// Makes the value at `p` say it holds variant `v` (its fields are
    /// written separately).
    pub fn set_variant(&mut self, p: Place, v: u32) -> R<()> {
        let l = self.layout(p.ty)?;
        let Variants::Multiple { tag, tag_offset, encoding, discriminants, .. } = &l.variants else { return Ok(()) };
        let size = tag.size(self.cx.lcx.target.ptr_size);
        let value: u128 = match encoding {
            TagEncoding::Direct => discriminants[v as usize] as u128,
            TagEncoding::Niche { untagged, first, start, .. } => {
                if v == *untagged {
                    return Ok(());
                }
                start.wrapping_add((v - first) as u128)
            }
        };
        let it = self.b.module().types.int(size as u32 * 8);
        let c = self.int_const(it, value);
        let a = self.offset(p.addr, *tag_offset);
        self.store_imm(c, a, size);
        Ok(())
    }

    /// The tag of the enum at `p`, as an integer, with its scalar's size.
    fn tag(&mut self, p: Place) -> R<(Value, u64)> {
        let l = self.layout(p.ty)?;
        let Variants::Multiple { tag, tag_offset, .. } = &l.variants else { return Err("tag of a single variant".into()) };
        let size = tag.size(self.cx.lcx.target.ptr_size);
        let it = self.b.module().types.int(size as u32 * 8);
        let a = self.offset(p.addr, *tag_offset);
        Ok((self.load_imm(it, a, size), size))
    }

    /// Whether the enum at `p` holds variant `v`, as an `i1`.
    pub fn test_variant(&mut self, p: Place, v: u32) -> R<Value> {
        let l = self.layout(p.ty)?;
        let i1 = self.b.module().types.int(1);
        let (encoding, discriminants) = match &l.variants {
            Variants::Single { index } => return Ok(self.b.const_int(i1, (*index == v) as i64)),
            Variants::Multiple { encoding, discriminants, .. } => (encoding.clone(), discriminants.clone()),
        };
        let (t, _) = self.tag(p)?;
        let it = self.b.type_of(t);
        Ok(match encoding {
            TagEncoding::Direct => {
                let d = self.int_const(it, discriminants[v as usize] as u128);
                self.b.icmp(ICmpPred::Eq, t, d)
            }
            TagEncoding::Niche { untagged, first, last, start } => {
                let s = self.int_const(it, start);
                let rel = self.b.sub(t, s);
                if v == untagged {
                    // Not one of the niche's values (or the one standing
                    // for the untagged variant itself, which none does).
                    let span = self.int_const(it, (last - first) as u128);
                    let outside = self.b.icmp(ICmpPred::Ugt, rel, span);
                    let own = self.int_const(it, (untagged - first) as u128);
                    let is_own = self.b.icmp(ICmpPred::Eq, rel, own);
                    self.b.or(outside, is_own)
                } else {
                    let want = self.int_const(it, (v - first) as u128);
                    self.b.icmp(ICmpPred::Eq, rel, want)
                }
            }
        })
    }

    /// The variant index of the enum at `p`, as an `i32`.
    pub fn variant_index(&mut self, p: Place) -> R<Value> {
        let l = self.layout(p.ty)?;
        let i32t = self.b.module().types.int(32);
        let n = match &l.variants {
            Variants::Single { index } => return Ok(self.b.const_int(i32t, *index as i64)),
            Variants::Multiple { variants, .. } => variants.len() as u32,
        };
        // A chain of selects: the last variant unless another matches.
        let mut r = self.b.const_int(i32t, (n - 1) as i64);
        for v in (0..n - 1).rev() {
            let c = self.test_variant(p, v)?;
            let k = self.b.const_int(i32t, v as i64);
            r = self.b.select(c, k, r);
        }
        Ok(r)
    }

    /// The discriminant of the enum at `p` (an `i64`, or the tag's width
    /// if wider).
    pub fn discriminant(&mut self, p: Place) -> R<Value> {
        let l = self.layout(p.ty)?;
        let i64t = self.b.module().types.int(64);
        match &l.variants {
            Variants::Single { index } => {
                let ds = match self.cx.tcx.kind(p.ty) {
                    TyKind::Adt(d, _) => self.cx.lcx.discriminants(d).unwrap_or_default(),
                    _ => Vec::new(),
                };
                let d = ds.get(*index as usize).copied().unwrap_or(0);
                Ok(self.b.const_int(i64t, d as i64))
            }
            Variants::Multiple { encoding: TagEncoding::Direct, tag, .. } => {
                let signed = matches!(tag.prim, crate::layout::Prim::Int { signed: true, .. });
                let (t, size) = self.tag(p)?;
                Ok(if size < 8 { self.int_to(t, i64t, signed) } else { t })
            }
            Variants::Multiple { discriminants, .. } => {
                let discriminants = discriminants.clone();
                let vi = self.variant_index(p)?;
                let mut r = self.b.const_int(i64t, *discriminants.last().unwrap_or(&0) as i64);
                let i32t = self.b.module().types.int(32);
                for (v, &d) in discriminants.iter().enumerate().rev().skip(1) {
                    let k = self.b.const_int(i32t, v as i64);
                    let c = self.b.icmp(ICmpPred::Eq, vi, k);
                    let dv = self.b.const_int(i64t, d as i64);
                    r = self.b.select(c, dv, r);
                }
                Ok(r)
            }
        }
    }

    /// Branches to `fail` unless `ok`.
    fn test(&mut self, ok: Value, fail: Option<BlockId>) -> R<()> {
        let Some(fail) = fail else { return Ok(()) };
        let next = self.new_block("pat.ok");
        self.b.cond_br(ok, next, fail);
        self.b.at(next);
        Ok(())
    }

    // ---- patterns ----

    /// Matches pattern `pat` against the value at `p`: jumps to `fail` if
    /// it does not match (none if it cannot fail), and binds as it goes.
    pub fn pat_match(&mut self, pat: NodeId, p: Place, fail: Option<BlockId>) -> R<()> {
        let ast = self.body.ast;
        let tcx = self.cx.tcx;
        // Through the references default binding modes passed.
        let mut p = p;
        let derefs = self.body.tables.pat_derefs.get(&pat).copied().unwrap_or(0);
        for _ in 0..derefs {
            let o = self.load(p)?;
            p = self.deref(o)?;
        }
        let kids = self.kids(pat);
        match ast.kind(pat) {
            Kind::WildPat | Kind::RestPat => Ok(()),
            Kind::ParenPat => self.pat_match(kids[0], p, fail),
            Kind::IdentPat => {
                if let Some(res) = self.res(pat) {
                    // A constant, or a unit struct or variant.
                    return self.path_test(pat, pat, res.clone(), p, fail);
                }
                if let Some(&sub) = kids.iter().find(|&&c| !matches!(ast.kind(c), Kind::Keyword | Kind::Name)) {
                    self.pat_match(sub, p, fail)?;
                }
                let mode = self.body.tables.bindings.get(&pat).copied().unwrap_or(BindingMode::Value);
                let key = (self.body.k, self.body.f, pat);
                let bt = self.ty(pat)?;
                let slot = match self.locals.get(&key) {
                    Some(&s) => s,
                    None => {
                        let s = self.temp(bt)?;
                        self.locals.insert(key, s);
                        s
                    }
                };
                let o = match mode {
                    BindingMode::Value => self.load(p)?,
                    BindingMode::Ref(_) => self.addr_of(p, bt)?,
                };
                self.store(Operand { ty: bt, v: o.v }, slot)
            }
            Kind::RefPat => {
                let o = self.load(p)?;
                let inner = self.deref(o)?;
                self.pat_match(*kids.last().unwrap(), inner, fail)
            }
            Kind::TuplePat => {
                let n = match tcx.kind(p.ty) {
                    TyKind::Tuple(l) => tcx.tys(l).len(),
                    _ => return Err("tuple pattern on something not a tuple".into()),
                };
                self.positional(&kids, p, 0, n, fail)
            }
            Kind::TupleStructPat => {
                let v = self.pat_variant(kids[0], p, fail)?;
                let n = self.variant_fields(p, v)?;
                self.positional(&kids[1..], p, v, n, fail)
            }
            Kind::StructPat => {
                let v = self.pat_variant(kids[0], p, fail)?;
                let TyKind::Adt(d, _) = tcx.kind(p.ty) else { return Err("struct pattern on something not a struct".into()) };
                let names: Vec<&str> = tcx.adts[&d].variants[v as usize].fields.iter().map(|f| f.name).collect();
                for &f in &kids[1..] {
                    if ast.kind(f) != Kind::FieldPat {
                        continue;
                    }
                    let fk = self.kids(f);
                    let (name, sub) = if fk.len() == 2 {
                        (ast.token(fk[0]).0, fk[1])
                    } else {
                        let sub = fk[0];
                        (ast.child(sub, Kind::Name).map_or("", |n| ast.token(n).0), sub)
                    };
                    let i = names.iter().position(|&x| x == name).ok_or_else(|| format!("no field `{name}`"))?;
                    let fp = self.field(p, v, i)?;
                    self.pat_match(sub, fp, fail)?;
                }
                Ok(())
            }
            Kind::PathPat => {
                let Some(res) = self.res(kids[0]) else { return Err("unresolved path pattern".into()) };
                self.path_test(pat, kids[0], res.clone(), p, fail)
            }
            Kind::LitPat => {
                let want = self.lit_value(kids[0], p.ty)?;
                let got = self.load(p)?;
                if matches!(tcx.kind(p.ty), TyKind::Ref(..)) {
                    return Err("string patterns are not supported yet".into());
                }
                let ok = self.compare("==", got, want)?;
                self.test(ok, fail)
            }
            Kind::RangePat => {
                let op_at = kids.iter().position(|&c| ast.kind(c) == Kind::Op).ok_or("range pattern without `..`")?;
                let inclusive = ast.text(kids[op_at]) != "..";
                let got = self.load(p)?;
                if let Some(&lo) = kids[..op_at].first() {
                    let lo = self.lit_value(lo, p.ty)?;
                    let ok = self.compare(">=", got, lo)?;
                    self.test(ok, fail)?;
                }
                if let Some(&hi) = kids.get(op_at + 1) {
                    let hi = self.lit_value(hi, p.ty)?;
                    let ok = self.compare(if inclusive { "<=" } else { "<" }, got, hi)?;
                    self.test(ok, fail)?;
                }
                Ok(())
            }
            Kind::OrPat => {
                // Each alternative binds the same names: into the same
                // slots, those of the first.
                let mut first: Vec<(String, NodeId)> = Vec::new();
                self.binding_names(kids[0], &mut first);
                for &alt in &kids[1..] {
                    let mut names = Vec::new();
                    self.binding_names(alt, &mut names);
                    for (name, node) in names {
                        if let Some((_, f)) = first.iter().find(|x| x.0 == name) {
                            let fkey = (self.body.k, self.body.f, *f);
                            let slot = match self.locals.get(&fkey) {
                                Some(&s) => s,
                                None => {
                                    let t = self.ty(*f)?;
                                    let s = self.temp(t)?;
                                    self.locals.insert(fkey, s);
                                    s
                                }
                            };
                            self.locals.insert((self.body.k, self.body.f, node), slot);
                        }
                    }
                }
                let matched = self.new_block("or.matched");
                for (i, &alt) in kids.iter().enumerate() {
                    let last = i + 1 == kids.len();
                    let next = if last { fail } else { Some(self.new_block("or.next")) };
                    self.pat_match(alt, p, next)?;
                    self.goto(matched);
                    match next {
                        Some(n) if !last => self.b.at(n),
                        _ => {
                            self.dead();
                            break;
                        }
                    };
                }
                self.b.at(matched);
                Ok(())
            }
            Kind::SlicePat => self.slice_pat(&kids, p, fail),
            Kind::NeverPat => Ok(()),
            k => Err(format!("pattern {k:?} is not supported yet")),
        }
    }

    /// The names bound in a pattern, with their nodes.
    fn binding_names(&self, pat: NodeId, out: &mut Vec<(String, NodeId)>) {
        let ast = self.body.ast;
        if ast.kind(pat) == Kind::IdentPat
            && self.res(pat).is_none()
            && let Some(n) = ast.child(pat, Kind::Name)
        {
            out.push((ast.token(n).0.to_string(), pat));
        }
        for &c in ast.children(pat) {
            if is_pattern(ast.kind(c)) || ast.kind(c) == Kind::FieldPat {
                self.binding_names(c, out);
            }
        }
    }

    /// How many fields variant `v` of the ADT at `p` has.
    fn variant_fields(&self, p: Place, v: u32) -> R<usize> {
        let tcx = self.cx.tcx;
        match tcx.kind(p.ty) {
            TyKind::Adt(d, _) => Ok(tcx.adts.get(&d).and_then(|a| a.variants.get(v as usize)).map_or(0, |x| x.fields.len())),
            _ => Err("constructor pattern on something not a struct or enum".into()),
        }
    }

    /// Tests the variant a constructor pattern's path names; gives it.
    fn pat_variant(&mut self, path: NodeId, p: Place, fail: Option<BlockId>) -> R<u32> {
        let tcx = self.cx.tcx;
        let TyKind::Adt(d, _) = tcx.kind(p.ty) else { return Err("constructor pattern on something not an ADT".into()) };
        let v = self.variant_of_path(path, d).ok_or("variant not found")?;
        if tcx.adts[&d].kind == AdtKind::Enum && fail.is_some() {
            let ok = self.test_variant(p, v)?;
            self.test(ok, fail)?;
        }
        Ok(v)
    }

    /// Positional patterns (with at most one `..`) against fields
    /// `0..n` of variant `v`.
    fn positional(&mut self, pats: &[NodeId], p: Place, v: u32, n: usize, fail: Option<BlockId>) -> R<()> {
        let ast = self.body.ast;
        let rest = pats.iter().position(|&c| ast.kind(c) == Kind::RestPat);
        for (i, &sp) in pats.iter().enumerate() {
            if Some(i) == rest {
                continue;
            }
            let fi = match rest {
                Some(r) if i > r => n - (pats.len() - i),
                _ => i,
            };
            let fp = self.field(p, v, fi)?;
            self.pat_match(sp, fp, fail)?;
        }
        Ok(())
    }

    /// A path in a pattern: a unit variant or struct, or a constant.
    fn path_test(&mut self, pat: NodeId, path: NodeId, res: Res, p: Place, fail: Option<BlockId>) -> R<()> {
        let tcx = self.cx.tcx;
        match res {
            Res::Def(d) if matches!(self.cx.r.defs[d.0 as usize].kind, DefKind::Variant | DefKind::Struct) => {
                if let TyKind::Adt(e, _) = tcx.kind(p.ty)
                    && tcx.adts[&e].kind == AdtKind::Enum
                    && fail.is_some()
                {
                    let v = self.variant_of_path(path, e).ok_or("variant not found")?;
                    let ok = self.test_variant(p, v)?;
                    self.test(ok, fail)?;
                }
                Ok(())
            }
            Res::Partial(..) | Res::SelfTy(..) => {
                if let TyKind::Adt(e, _) = tcx.kind(p.ty)
                    && tcx.adts[&e].kind == AdtKind::Enum
                    && fail.is_some()
                {
                    let v = self.variant_of_path(path, e).ok_or("variant not found")?;
                    let ok = self.test_variant(p, v)?;
                    self.test(ok, fail)?;
                }
                Ok(())
            }
            _ => {
                // A constant: compared as a scalar.
                let want = self.lit_value(pat, p.ty)?;
                let got = self.load(p)?;
                let ok = self.compare("==", got, want)?;
                self.test(ok, fail)
            }
        }
    }

    /// A literal (or negated literal, or constant path) in a pattern, as a
    /// value of type `t`.
    fn lit_value(&mut self, n: NodeId, t: crate::ty::Ty) -> R<Operand> {
        let ast = self.body.ast;
        let tcx = self.cx.tcx;
        match ast.kind(n) {
            Kind::LitPat => {
                let k = self.kids(n);
                self.lit_value(k[0], t)
            }
            Kind::Unary => {
                let e = ast.children(n)[1];
                let o = self.lit_value(e, t)?;
                let v = self.imm(o)?;
                let r = if matches!(tcx.kind(t), TyKind::Float(_)) {
                    self.b.fneg(v)
                } else {
                    let ty = self.b.type_of(v);
                    let z = self.b.const_int(ty, 0);
                    self.b.sub(z, v)
                };
                Ok(Operand { ty: t, v: Val::Imm(r) })
            }
            Kind::Literal => {
                let tok = ast.token(n).0;
                let Repr::Imm(it) = self.repr(t)? else { return Err("literal pattern of a type in memory".into()) };
                let v = match tcx.kind(t) {
                    TyKind::Bool => (tok == "true") as u128,
                    TyKind::Char => super::body::parse_char(tok).ok_or("bad character")? as u128,
                    TyKind::Float(_) => {
                        let digits: String = tok.trim_end_matches("f32").trim_end_matches("f64").chars().filter(|&c| c != '_').collect();
                        let x: f64 = digits.parse().map_err(|_| "bad float")?;
                        let c = self.b.const_float(it, x);
                        return Ok(Operand { ty: t, v: Val::Imm(c) });
                    }
                    _ if tok.starts_with("b'") => super::body::parse_char(&tok[1..]).ok_or("bad byte")? as u128,
                    _ => super::body::parse_int(tok).ok_or("bad integer")?,
                };
                let c = self.int_const(it, v);
                Ok(Operand { ty: t, v: Val::Imm(c) })
            }
            Kind::PathPat | Kind::PathExpr | Kind::IdentPat => match self.body.tables.consts.get(&n).copied() {
                Some((c, l)) => {
                    let l = self.sub_list(l);
                    let pv = self.constant(n, c, l)?;
                    let o = self.operand_of(pv)?;
                    Ok(Operand { ty: t, v: o.v })
                }
                None => Err("path in a pattern that is not a constant".into()),
            },
            k => Err(format!("{k:?} in a pattern")),
        }
    }

    /// `[a, b, rest @ .., z]` against an array or slice.
    fn slice_pat(&mut self, kids: &[NodeId], p: Place, fail: Option<BlockId>) -> R<()> {
        let ast = self.body.ast;
        let tcx = self.cx.tcx;
        let is_rest = |c: NodeId| {
            ast.kind(c) == Kind::RestPat || ast.kind(c) == Kind::IdentPat && ast.children(c).iter().any(|&x| ast.kind(x) == Kind::RestPat)
        };
        let rest = kids.iter().position(|&c| is_rest(c));
        let (elem, len) = match tcx.kind(p.ty) {
            TyKind::Array(e, c) => {
                let n = self.cx.lcx.eval_const(c, &Default::default()).ok_or("array length not known")?;
                (e, self.usize_const(n as u64))
            }
            TyKind::Slice(e) => (e, p.meta.ok_or("slice without a length")?),
            _ => return Err("slice pattern on something not an array or slice".into()),
        };
        let fixed = kids.len() - rest.is_some() as usize;
        let need = self.usize_const(fixed as u64);
        if matches!(tcx.kind(p.ty), TyKind::Slice(_)) && fail.is_some() {
            let ok = if rest.is_some() { self.b.icmp(ICmpPred::Uge, len, need) } else { self.b.icmp(ICmpPred::Eq, len, need) };
            self.test(ok, fail)?;
        }
        let stride = self.layout(elem)?.size;
        for (i, &sp) in kids.iter().enumerate() {
            let idx = match rest {
                Some(r) if i == r => {
                    if ast.kind(sp) == Kind::IdentPat {
                        // `rest @ ..`: the middle, as a slice.
                        let start = self.usize_const(i as u64);
                        let s = self.usize_const(stride);
                        let off = self.b.mul(start, s);
                        let i8t = self.i8t();
                        let addr = self.b.gep(i8t, p.addr, &[off]);
                        let mid = self.b.sub(len, need);
                        let st = tcx.mk(TyKind::Slice(elem));
                        let bt = self.ty(sp)?;
                        let mode = self.body.tables.bindings.get(&sp).copied().unwrap_or(BindingMode::Value);
                        let sub = Place { addr, meta: Some(mid), ty: st };
                        let o = match mode {
                            BindingMode::Ref(_) => self.addr_of(sub, bt)?,
                            BindingMode::Value => return Err("binding a slice by value".into()),
                        };
                        let slot = self.temp(bt)?;
                        self.store(o, slot)?;
                        self.locals.insert((self.body.k, self.body.f, sp), slot);
                    }
                    continue;
                }
                Some(r) if i > r => {
                    let from_end = self.usize_const((kids.len() - i) as u64);
                    self.b.sub(len, from_end)
                }
                _ => self.usize_const(i as u64),
            };
            let s = self.usize_const(stride);
            let off = self.b.mul(idx, s);
            let i8t = self.i8t();
            let addr = self.b.gep(i8t, p.addr, &[off]);
            self.pat_match(sp, Place { addr, meta: None, ty: elem }, fail)?;
        }
        Ok(())
    }
}
