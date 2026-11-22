//! Variance of generic parameters: how a type changes when an argument
//! does. `Foo<&'long T>` can be used as `Foo<&'short T>` only if `Foo` is
//! covariant in that parameter, which is for borrow checking to know.
//!
//! rustc's algorithm (`rustc_hir_analysis::variance`): every parameter of
//! a struct, enum, union or type alias starts bivariant (unused); walking
//! its fields, each use of a parameter adds a constraint, "at least this
//! variance", where the variance of the use is that of the position
//! (covariant at the top, flipped inside a function pointer's arguments,
//! invariant under `&mut`, `*mut` and trait arguments) composed with the
//! variance of the parameter of the type it is an argument of. That
//! composition is why the constraints are solved together, to a fixed
//! point. Bounds and where clauses add none.
//!
//! Lifetimes are not in `Ty`, so this walks the source types and uses name
//! resolution for the paths, as `ty::lower` does.

use crate::ast::{Ast, Kind, NodeId};
use crate::hash::FastMap;
use crate::resolve::{DefId, DefKind, Res, Resolver};
use crate::session::ParsedCrate;
use crate::ty::TyCtxt;

/// How a type varies with a parameter.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum Variance {
    /// Unused: any argument will do (`*`).
    Bi,
    /// Larger argument, larger type (`+`).
    Co,
    /// Larger argument, smaller type (`-`).
    Contra,
    /// Only the same argument (`o`).
    Inv,
}

impl Variance {
    /// The variance of a use in a position that varies as `self` of
    /// something that varies as `v`.
    pub fn xform(self, v: Variance) -> Variance {
        use Variance::*;
        match (self, v) {
            (Co, v) => v,
            (Inv, Bi) => Bi,
            (Inv, _) => Inv,
            (Contra, Co) => Contra,
            (Contra, Contra) => Co,
            (Contra, v) => v,
            (Bi, _) => Bi,
        }
    }

    /// The least variance at least both.
    pub fn join(self, o: Variance) -> Variance {
        use Variance::*;
        match (self, o) {
            (Bi, v) | (v, Bi) => v,
            (a, b) if a == b => a,
            _ => Inv,
        }
    }

    /// `+`, `-`, `o`, `*`, as rustc prints them.
    pub fn symbol(self) -> char {
        match self {
            Variance::Bi => '*',
            Variance::Co => '+',
            Variance::Contra => '-',
            Variance::Inv => 'o',
        }
    }

    /// The variance of what is flipped (a function's argument).
    pub fn flip(self) -> Variance {
        Variance::Contra.xform(self)
    }
}

/// What a parameter of an item is.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ParamKind {
    Lifetime,
    Type,
    Const,
}

/// A generic parameter of a struct, enum, union or alias, in the order
/// written.
#[derive(Clone, Debug)]
pub struct VarParam {
    pub name: String,
    pub kind: ParamKind,
    pub node: NodeId,
    pub variance: Variance,
}

/// A variance the constraints refer to: a constant, a parameter's, or one
/// applied to another.
#[derive(Clone, Debug)]
enum Term {
    Const(Variance),
    Var(DefId, usize),
    Xform(Box<Term>, Box<Term>),
}

impl Term {
    fn xform(self, inner: Term) -> Term {
        match (self, inner) {
            (Term::Const(Variance::Co), t) => t,
            (Term::Const(a), Term::Const(b)) => Term::Const(a.xform(b)),
            (a, b) => Term::Xform(Box::new(a), Box::new(b)),
        }
    }
}

/// The variances of the generic parameters of the items of a session.
#[derive(Default)]
pub struct Variances {
    items: FastMap<DefId, Vec<VarParam>>,
}

impl Variances {
    /// The parameters of a struct, enum, union or type alias, with their
    /// variances; none for other items.
    pub fn of(&self, d: DefId) -> Option<&[VarParam]> {
        self.items.get(&d).map(|v| v.as_slice())
    }

    /// `['a: +, T: o]`, as `#[rustc_dump_variances]` prints it.
    pub fn show(&self, d: DefId) -> Option<String> {
        let ps = self.items.get(&d)?;
        let v: Vec<String> = ps.iter().map(|p| format!("{}: {}", p.name, p.variance.symbol())).collect();
        Some(format!("[{}]", v.join(", ")))
    }
}

struct Constraint {
    item: DefId,
    dest: usize,
    term: Term,
}

struct Walker<'c, 's> {
    r: &'c Resolver<'s>,
    ast: &'c Ast<'s>,
    krate: usize,
    file: usize,
    item: DefId,
    params: &'c [VarParam],
    kinds: &'c FastMap<DefId, Vec<ParamKind>>,
    /// Lifetimes bound by `for<...>` around what is walked.
    bound: Vec<&'c str>,
    out: &'c mut Vec<Constraint>,
}

impl<'c, 's> Walker<'c, 's> {
    fn constrain(&mut self, dest: usize, term: Term) {
        self.out.push(Constraint { item: self.item, dest, term });
    }

    fn lifetime(&mut self, lt: NodeId, cur: &Term) {
        let name = self.ast.token(lt).0;
        if name == "'static" || name == "'_" || self.bound.iter().any(|b| super::sig::lname(b) == super::sig::lname(name)) {
            return;
        }
        if let Some(i) = self.params.iter().position(|p| p.kind == ParamKind::Lifetime && super::sig::lname(&p.name) == super::sig::lname(name)) {
            self.constrain(i, cur.clone());
        }
    }

    /// Every parameter, as in `Self`: `Adt<params>`.
    fn self_ty(&mut self, cur: &Term) {
        for i in 0..self.params.len() {
            let t = cur.clone().xform(Term::Var(self.item, i));
            self.constrain(i, t);
        }
    }

    fn generic(&mut self, n: NodeId, cur: &Term) {
        if let Some(i) = self.params.iter().position(|p| p.node == n) {
            self.constrain(i, cur.clone());
        }
    }

    fn inv() -> Term {
        Term::Const(Variance::Inv)
    }

    /// Everything named under `n` is used invariantly (a trait's arguments,
    /// a projection, `Fn(A) -> B` sugar).
    fn all_inv(&mut self, n: NodeId) {
        let ast = self.ast;
        let pushed = self.push_binder(n);
        match ast.kind(n) {
            Kind::Lifetime => self.lifetime(n, &Self::inv()),
            Kind::Path => {
                match self.r.res_of(self.krate, self.file, n).cloned() {
                    Some(res) => self.res_inv(&res),
                    None => {}
                }
            }
            _ => {}
        }
        for &c in ast.children(n) {
            self.all_inv(c);
        }
        self.bound.truncate(self.bound.len() - pushed);
    }

    fn res_inv(&mut self, res: &Res) {
        match res {
            Res::Generic(n) => self.generic(*n, &Self::inv()),
            Res::SelfTy(..) => self.self_ty(&Self::inv()),
            Res::Partial(b, _) => self.res_inv(b),
            _ => {}
        }
    }

    /// The names a `for<'a, 'b>` under `n` binds, pushed; how many.
    fn push_binder(&mut self, n: NodeId) -> usize {
        let ast = self.ast;
        let Some(f) = ast.child(n, Kind::ForLifetimes) else { return 0 };
        let mut k = 0;
        for &p in ast.children(f) {
            if let Some(l) = ast.child(p, Kind::Lifetime) {
                self.bound.push(ast.token(l).0);
                k += 1;
            }
        }
        k
    }

    /// `region` is what `dyn Trait` with no bound of its own defaults to.
    fn ty(&mut self, n: NodeId, cur: &Term, region: Option<NodeId>) {
        let ast = self.ast;
        let kids = ast.children(n);
        match ast.kind(n) {
            Kind::PathType => {
                if let Some(&p) = kids.first() {
                    self.path(p, cur, region);
                }
            }
            Kind::RefType => {
                let lt = kids.iter().copied().find(|&c| ast.kind(c) == Kind::Lifetime);
                let is_mut = kids.iter().any(|&c| ast.kind(c) == Kind::Keyword && ast.token(c).0 == "mut");
                if let Some(l) = lt {
                    self.lifetime(l, cur);
                }
                let inner = *kids.last().unwrap();
                let c = if is_mut { Term::Const(Variance::Inv) } else { cur.clone() };
                self.ty(inner, &c, lt);
            }
            Kind::PtrType => {
                let is_mut = ast.token(kids[0]).0 == "mut";
                let c = if is_mut { Term::Const(Variance::Inv) } else { cur.clone() };
                self.ty(kids[1], &c, None);
            }
            Kind::ArrayType => {
                self.ty(kids[0], cur, None);
                // What the length names is invariant.
                self.all_inv(kids[1]);
            }
            Kind::SliceType | Kind::PatternType => self.ty(kids[0], cur, None),
            Kind::TupleType => {
                for &c in kids {
                    self.ty(c, cur, None);
                }
            }
            Kind::ParenType => self.ty(kids[0], cur, region),
            Kind::FnPtrType => {
                let pushed = self.push_binder(n);
                let flipped = Term::Const(Variance::Contra).xform(cur.clone());
                for &c in kids {
                    match ast.kind(c) {
                        Kind::Params => {
                            for &p in ast.children(c) {
                                if let Some(&t) = ast.children(p).last() {
                                    self.ty(t, &flipped, None);
                                }
                            }
                        }
                        Kind::RetType => self.ty(ast.children(c)[0], cur, None),
                        _ => {}
                    }
                }
                self.bound.truncate(self.bound.len() - pushed);
            }
            Kind::DynTraitType | Kind::ImplTraitType => {
                let Some(b) = ast.child(n, Kind::Bounds) else { return };
                let mut has_region = false;
                for &c in ast.children(b) {
                    match ast.kind(c) {
                        Kind::Lifetime => {
                            has_region = true;
                            self.lifetime(c, cur);
                        }
                        Kind::TraitBound => self.all_inv(c),
                        _ => {}
                    }
                }
                // `&'a dyn Trait` is `&'a (dyn Trait + 'a)`.
                if !has_region && let Some(l) = region {
                    self.lifetime(l, cur);
                }
            }
            Kind::UnsafeBinderType => self.ty(*kids.last().unwrap(), cur, None),
            _ => {}
        }
    }

    fn path(&mut self, p: NodeId, cur: &Term, region: Option<NodeId>) {
        let ast = self.ast;
        if ast.child(p, Kind::QSelf).is_some() {
            self.all_inv(p);
            return;
        }
        let Some(res) = self.r.res_of(self.krate, self.file, p).cloned() else { return };
        let last = ast.children(p).iter().rev().copied().find(|&s| ast.kind(s) == Kind::Segment);
        match res {
            Res::Generic(n) => self.generic(n, cur),
            Res::SelfTy(..) => self.self_ty(cur),
            Res::Partial(..) => self.all_inv(p),
            Res::Def(d) => {
                let kind = self.r.defs[d.0 as usize].kind;
                if matches!(kind, DefKind::Struct | DefKind::Enum | DefKind::Union | DefKind::TypeAlias) {
                    let args = last.and_then(|s| ast.child(s, Kind::GenericArgs));
                    let (mut lts, mut tys) = (Vec::new(), Vec::new());
                    for &a in args.map(|g| ast.children(g)).unwrap_or(&[]) {
                        match ast.kind(a) {
                            Kind::Lifetime => lts.push(a),
                            Kind::AssocConstraint => {}
                            _ => tys.push(a),
                        }
                    }
                    let (mut li, mut ti) = (0, 0);
                    // The item's parameters, from what it declared.
                    let decl = self.kinds.get(&d).map_or(&[][..], |v| v.as_slice());
                    for (i, kind) in decl.iter().enumerate() {
                        let inner = cur.clone().xform(Term::Var(d, i));
                        match kind {
                            ParamKind::Lifetime => {
                                if let Some(&l) = lts.get(li) {
                                    self.lifetime(l, &inner);
                                }
                                li += 1;
                            }
                            ParamKind::Type => {
                                if let Some(&t) = tys.get(ti) {
                                    // A type argument's own `dyn Trait` defaults
                                    // to `'static` (or its bound).
                                    self.ty(t, &inner, None);
                                }
                                ti += 1;
                            }
                            ParamKind::Const => {
                                if let Some(&c) = tys.get(ti) {
                                    self.all_inv(c);
                                }
                                ti += 1;
                            }
                        }
                    }
                } else {
                    self.all_inv(p);
                }
                let _ = region;
            }
            _ => {}
        }
    }
}

fn collect_decl<'s>(ast: &Ast<'s>, node: NodeId) -> Vec<(ParamKind, String, NodeId)> {
    let Some(g) = ast.child(node, Kind::Generics) else { return Vec::new() };
    let mut v = Vec::new();
    for &p in ast.children(g) {
        match ast.kind(p) {
            Kind::LifetimeParam => {
                if let Some(l) = ast.child(p, Kind::Lifetime) {
                    v.push((ParamKind::Lifetime, ast.token(l).0.to_string(), p));
                }
            }
            Kind::TypeParam | Kind::ConstParam => {
                if let Some(n) = ast.child(p, Kind::Name) {
                    let k = if ast.kind(p) == Kind::TypeParam { ParamKind::Type } else { ParamKind::Const };
                    v.push((k, ast.token(n).ident_name().to_string(), p));
                }
            }
            _ => {}
        }
    }
    v
}

/// The types an item's variance comes from, each with the variance of
/// its position: the fields of a struct, union or enum, the aliased type,
/// a function's arguments (contravariant) and result.
fn field_types(ast: &Ast, node: NodeId, kind: DefKind) -> Vec<(NodeId, Variance)> {
    let mut out = Vec::new();
    let from = |owner: NodeId, out: &mut Vec<(NodeId, Variance)>| {
        for &c in ast.children(owner) {
            if matches!(ast.kind(c), Kind::NamedFields | Kind::TupleFields) {
                for &f in ast.children(c) {
                    if let Some(&t) = ast.children(f).iter().find(|&&t| crate::ty::lower::is_type(ast.kind(t))) {
                        out.push((t, Variance::Co));
                    }
                }
            }
        }
    };
    match kind {
        DefKind::Struct | DefKind::Union => from(node, &mut out),
        DefKind::Enum => {
            for &v in ast.children(node) {
                if ast.kind(v) == Kind::Variant {
                    from(v, &mut out);
                }
            }
        }
        DefKind::TypeAlias => {
            if let Some(&t) = ast.children(node).iter().find(|&&t| crate::ty::lower::is_type(ast.kind(t))) {
                out.push((t, Variance::Co));
            }
        }
        DefKind::Fn => {
            if let Some(ps) = ast.child(node, Kind::Params) {
                for &p in ast.children(ps) {
                    if let Some(&t) = ast.children(p).last().filter(|_| ast.kind(p) == Kind::Param) {
                        out.push((t, Variance::Contra));
                    }
                }
            }
            if let Some(rt) = ast.child(node, Kind::RetType) {
                out.push((ast.children(rt)[0], Variance::Co));
            }
        }
        _ => {}
    }
    out
}

/// The variances of the parameters of every struct, enum, union and type
/// alias of `crates`.
pub fn compute<'s>(crates: &[ParsedCrate<'s>], r: &Resolver<'s>, tcx: &TyCtxt<'s>) -> Variances {
    let mut items: FastMap<DefId, Vec<VarParam>> = FastMap::default();
    let mut nodes: Vec<(DefId, DefKind)> = Vec::new();
    let mut kinds: FastMap<DefId, Vec<ParamKind>> = FastMap::default();
    for (i, def) in r.defs.iter().enumerate() {
        if !matches!(def.kind, DefKind::Struct | DefKind::Enum | DefKind::Union | DefKind::TypeAlias | DefKind::Fn) {
            continue;
        }
        let d = DefId(i as u32);
        let ast = &crates[def.krate as usize].files[def.file as usize].ast;
        let mut decl = collect_decl(ast, def.node);
        if def.kind == DefKind::Fn {
            // Only the lifetimes that are parameters of the function's type.
            decl.retain(|p| p.0 != ParamKind::Lifetime || !super::sig::is_late(ast, def.node, &p.1, p.2));
        }
        kinds.insert(d, decl.iter().map(|p| p.0).collect());
        let ps = decl
            .into_iter()
            .map(|(kind, name, node)| VarParam {
                name,
                kind,
                node,
                // A constant is invariant; the rest start unused.
                variance: if kind == ParamKind::Const { Variance::Inv } else { Variance::Bi },
            })
            .collect();
        items.insert(d, ps);
        nodes.push((d, def.kind));
    }
    let mut cons: Vec<Constraint> = Vec::new();
    for &(d, kind) in &nodes {
        let def = &r.defs[d.0 as usize];
        let ast = &crates[def.krate as usize].files[def.file as usize].ast;
        // (Type parameters are found by the node of their declaration.)
        let params = items[&d].clone();
        let mut w = Walker {
            r,
            ast,
            krate: def.krate as usize,
            file: def.file as usize,
            item: d,
            params: &params,
            kinds: &kinds,
            bound: Vec::new(),
            out: &mut cons,
        };
        for (t, v) in field_types(ast, def.node, kind) {
            w.ty(t, &Term::Const(v), None);
        }
    }
    // Special cases: `PhantomData<T>` is covariant, `UnsafeCell<T>` and
    // `*mut`-like cells are invariant.
    let fixed = |name: &str, v: Variance, items: &mut FastMap<DefId, Vec<VarParam>>| {
        if let Some(d) = tcx.lang.get(name)
            && let Some(ps) = items.get_mut(d)
        {
            for p in ps.iter_mut().filter(|p| p.kind != ParamKind::Lifetime) {
                p.variance = v;
            }
            return Some(*d);
        }
        None
    };
    let pinned: Vec<DefId> =
        [fixed("phantom_data", Variance::Co, &mut items), fixed("unsafe_cell", Variance::Inv, &mut items)].into_iter().flatten().collect();
    cons.retain(|c| !pinned.contains(&c.item));
    // To a fixed point.
    loop {
        let mut changed = false;
        for c in &cons {
            let v = eval(&c.term, &items);
            let p = &mut items.get_mut(&c.item).unwrap()[c.dest];
            let j = p.variance.join(v);
            if j != p.variance {
                p.variance = j;
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }
    Variances { items }
}

fn eval(t: &Term, items: &FastMap<DefId, Vec<VarParam>>) -> Variance {
    match t {
        Term::Const(v) => *v,
        Term::Var(d, i) => items.get(d).and_then(|p| p.get(*i)).map_or(Variance::Inv, |p| p.variance),
        Term::Xform(a, b) => eval(a, items).xform(eval(b, items)),
    }
}
