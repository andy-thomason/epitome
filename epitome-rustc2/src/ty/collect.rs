//! Building every item's signature.
//!
//! In order, over all definitions (an item's parent always comes before
//! it): generic parameters; the `Self` type of each impl, trait and type;
//! then predicates, fields, function signatures, impl trait refs, and the
//! types of aliases, constants and statics. Aliases are expanded when they
//! are first needed, so their order does not matter.

use super::lower::{At, Env, Lower, is_type};
use super::{Abi, Mutbl, TraitDef, TraitRef, Ty, TyCtxt, TyKind, VariantDef};
use super::{AdtDef, AdtKind, CtorKind, FieldDef, FnSig, Generics, ImplDef, ParamDef, ParamId, Pred};
use crate::ast::{Ast, Kind, NodeId};
use crate::hash::FastSet;
use crate::resolve::{DefId, DefKind, Ns, Res, Resolver};
use crate::session::ParsedCrate;

pub(super) fn collect<'s>(crates: &[ParsedCrate<'s>], r: &Resolver<'s>) -> TyCtxt<'s> {
    let mut tcx = TyCtxt::new(r.defs.len());
    let env = Env { r, crates };
    for (i, d) in r.defs.iter().enumerate() {
        if d.parent.is_some() {
            tcx.def_at.insert((d.krate, d.file, d.node), DefId(i as u32));
            // `#[lang = "name"]`.
            let ast = env.ast(d.krate, d.file);
            if let Some(name) = crate::session::attribute_value(ast, d.node, "lang") {
                let name = ast.children(d.node).iter().find_map(|&a| {
                    let v = *ast.children(a).last()?;
                    (ast.kind(a) == Kind::Attribute && ast.kind(v) == Kind::Literal && crate::cfg::string_value(ast.token(v).0) == name)
                        .then(|| crate::cfg::string_value(ast.token(v).0))
                });
                if let Some(name) = name {
                    tcx.lang.insert(name, DefId(i as u32));
                }
            }
        }
    }
    let defs: Vec<DefId> = (0..r.defs.len() as u32).map(DefId).filter(|&d| r.defs[d.0 as usize].parent.is_some()).collect();
    for &d in &defs {
        generics(&mut tcx, &env, d);
    }
    // `Self`: of types and traits first, then of impls (whose self types
    // may name types).
    for &d in &defs {
        let at = env.at_of(d);
        match r.defs[d.0 as usize].kind {
            DefKind::Struct | DefKind::Enum | DefKind::Union => {
                let args = tcx.identity_args(d);
                let t = tcx.mk(TyKind::Adt(d, args));
                tcx.self_ty.insert(at, t);
            }
            DefKind::Trait | DefKind::TraitAlias => {
                let p = tcx.generics[d.0 as usize].own[0];
                let t = tcx.mk(TyKind::Param(p));
                tcx.self_ty.insert(at, t);
            }
            _ => {}
        }
    }
    // Parameter defaults, which may name `Self` (`trait Add<Rhs = Self>`).
    for &d in &defs {
        let ds = defaults(&tcx, &env, d);
        tcx.generics[d.0 as usize].defaults = ds;
    }
    for &d in &defs {
        if r.defs[d.0 as usize].kind == DefKind::Impl {
            let (imp, failed) = impl_header(&tcx, &env, d);
            tcx.self_ty.insert(env.at_of(d), imp.self_ty);
            tcx.impl_index.insert(d, tcx.impls.len());
            tcx.impls.push(imp);
            tcx.sig_count += 1;
            tcx.sig_errors += failed as usize;
        }
    }
    for &d in &defs {
        let Some(out) = signature(&tcx, &env, d) else { continue };
        tcx.predicates[d.0 as usize] = out.preds;
        if let Some(a) = out.adt {
            tcx.adts.insert(d, a);
        }
        if let Some(t) = out.trait_def {
            tcx.traits.insert(d, t);
        }
        tcx.fn_sig[d.0 as usize] = out.fn_sig;
        if out.counted {
            tcx.sig_count += 1;
            tcx.sig_errors += out.failed as usize;
        }
    }
    tcx
}

/// An item's AST node and file's tree.
fn node_of<'c, 's>(env: &Env<'c, 's>, d: DefId) -> (At, &'c Ast<'s>) {
    let at = env.at_of(d);
    (at, env.ast(at.0, at.1))
}

/// The item whose generics an item's generics extend: an impl's or
/// trait's, for their items.
fn generics_parent(env: &Env, d: DefId) -> Option<DefId> {
    let r = env.r;
    let def = &r.defs[d.0 as usize];
    if !matches!(def.kind, DefKind::AssocFn | DefKind::AssocConst | DefKind::AssocType | DefKind::Variant) {
        return None;
    }
    let m = def.parent?;
    let p = r.modules[m.0 as usize].def?;
    matches!(r.defs[p.0 as usize].kind, DefKind::Impl | DefKind::Trait | DefKind::Enum).then_some(p)
}

fn new_param<'s>(
    tcx: &mut TyCtxt<'s>,
    owner: DefId,
    index: u32,
    name: &'s str,
    is_const: bool,
    at: At,
) -> ParamId {
    let id = ParamId(tcx.params.len() as u32);
    tcx.params.push(ParamDef { owner, index, name, is_const, at });
    tcx.param_at.insert(at, id);
    id
}

fn generics<'s>(tcx: &mut TyCtxt<'s>, env: &Env<'_, 's>, d: DefId) {
    let parent = generics_parent(env, d);
    let parent_count = parent.map_or(0, |p| {
        let g = &tcx.generics[p.0 as usize];
        g.parent_count + g.own.len() as u32
    });
    let (at, ast) = node_of(env, d);
    let mut own = Vec::new();
    let mut index = parent_count;
    if matches!(env.r.defs[d.0 as usize].kind, DefKind::Trait | DefKind::TraitAlias) {
        // A trait's `Self` is its parameter 0.
        own.push(new_param(tcx, d, index, "Self", false, at));
        index += 1;
    }
    if env.r.defs[d.0 as usize].kind != DefKind::Variant
        && let Some(g) = ast.child(at.2, Kind::Generics)
    {
        for &p in ast.children(g) {
            let kind = ast.kind(p);
            if !matches!(kind, Kind::TypeParam | Kind::ConstParam) {
                continue;
            }
            let Some(n) = ast.child(p, Kind::Name) else { continue };
            let name = ast.token(n).ident_name();
            own.push(new_param(tcx, d, index, name, kind == Kind::ConstParam, (at.0, at.1, p)));
            index += 1;
        }
    }
    let n = own.len();
    tcx.generics[d.0 as usize] = Generics { parent, parent_count, own, defaults: vec![None; n] };
}

/// The defaults of an item's own parameters.
fn defaults<'s>(tcx: &TyCtxt<'s>, env: &Env<'_, 's>, d: DefId) -> Vec<Option<Ty>> {
    let own = tcx.generics[d.0 as usize].own.clone();
    let mut out = vec![None; own.len()];
    for (i, p) in own.into_iter().enumerate() {
        let at = tcx.params[p.0 as usize].at;
        let ast = env.ast(at.0, at.1);
        let mut l = Lower { tcx, env, expanding: FastSet::default(), impl_self: None, holes: false, k: at.0, file: at.1, owner: d, failed: false };
        out[i] = match ast.kind(at.2) {
            Kind::TypeParam => ast.children(at.2).iter().copied().find(|&c| is_type(ast.kind(c))).map(|t| l.ty(t)),
            // `const N: usize = 4`: the value after the type.
            Kind::ConstParam => ast.children(at.2).get(2).map(|&v| {
                let c = l.konst(v);
                tcx.mk(TyKind::ConstArg(c))
            }),
            _ => None,
        };
    }
    out
}

/// An impl's self type and trait, and whether either had an error.
fn impl_header<'s>(tcx: &TyCtxt<'s>, env: &Env<'_, 's>, d: DefId) -> (ImplDef, bool) {
    let (at, ast) = node_of(env, d);
    let kids = ast.children(at.2).to_vec();
    let self_node = kids.iter().rev().copied().find(|&c| is_type(ast.kind(c)));
    let mut l = Lower { tcx, env, expanding: FastSet::default(), impl_self: None, holes: false, k: at.0, file: at.1, owner: d, failed: false };
    let self_ty = match self_node {
        Some(t) => l.ty(t),
        None => l.tcx.error(),
    };
    l.impl_self = Some((at, self_ty));
    let mut negative = false;
    let trait_ref = kids.iter().copied().find(|&c| ast.kind(c) == Kind::ImplTrait).and_then(|it| {
        negative = ast.children(it).iter().any(|&c| ast.kind(c) == Kind::Op);
        let tp = ast.children(it).iter().copied().find(|&c| ast.kind(c) == Kind::PathType)?;
        let path = *ast.children(tp).first()?;
        let t = l.trait_ref(path);
        if t.is_none() {
            l.failed = true;
            l.tcx.note_error("impl of something that is not a trait");
        }
        t
    });
    (ImplDef { def: d, self_ty, trait_ref, negative }, l.failed)
}

/// The predicates of an item's generic parameters and `where` clause.
fn predicates(l: &mut Lower, item: NodeId, preds: &mut Vec<Pred>) {
    let ast = l.env.ast(l.k, l.file);
    if let Some(g) = ast.child(item, Kind::Generics) {
        for &p in ast.children(g) {
            if ast.kind(p) != Kind::TypeParam {
                continue;
            }
            let Some(&pid) = l.tcx.param_at.get(&(l.k, l.file, p)) else { continue };
            let t = l.tcx.mk(TyKind::Param(pid));
            if let Some(b) = ast.child(p, Kind::Bounds) {
                for tr in l.bounds(b) {
                    push_trait(l.tcx, preds, t, tr);
                }
            }
        }
    }
    if let Some(w) = ast.child(item, Kind::WhereClause) {
        for &pred in ast.children(w) {
            let kids = ast.children(pred);
            let ty = kids.iter().copied().find(|&c| is_type(ast.kind(c)));
            let b = kids.iter().copied().find(|&c| ast.kind(c) == Kind::Bounds);
            if let (Some(ty), Some(b)) = (ty, b) {
                let t = l.ty(ty);
                for tr in l.bounds(b) {
                    push_trait(l.tcx, preds, t, tr);
                }
            }
        }
    }
}

/// `t: tr`, and `<t as tr>::A == B` for each of its bindings.
fn push_trait(tcx: &TyCtxt, preds: &mut Vec<Pred>, t: Ty, tr: TraitRef) {
    for &(assoc, b) in &tr.bindings {
        let mut args = vec![t];
        args.extend(tcx.tys(tr.args));
        preds.push(Pred::Projection(t, assoc, tcx.list(args), b));
    }
    preds.push(Pred::Trait(t, tr));
}

fn fields<'s>(l: &mut Lower<'_, '_, 's>, node: NodeId) -> (CtorKind, Vec<FieldDef<'s>>) {
    let ast = l.env.ast(l.k, l.file);
    if let Some(f) = ast.child(node, Kind::NamedFields) {
        let fs = ast
            .children(f)
            .iter()
            .filter_map(|&d| {
                let name = ast.token(ast.child(d, Kind::Name)?).ident_name();
                let t = ast.children(d).iter().copied().find(|&c| is_type(ast.kind(c)))?;
                Some((name, t))
            })
            .collect::<Vec<_>>();
        let fs = fs.into_iter().map(|(name, t)| FieldDef { name, ty: l.ty(t) }).collect();
        (CtorKind::Named, fs)
    } else if let Some(f) = ast.child(node, Kind::TupleFields) {
        const NUMS: [&str; 16] = ["0", "1", "2", "3", "4", "5", "6", "7", "8", "9", "10", "11", "12", "13", "14", "15"];
        let ts: Vec<NodeId> = ast
            .children(f)
            .iter()
            .filter_map(|&d| ast.children(d).iter().copied().find(|&c| is_type(ast.kind(c))))
            .collect();
        let fs = ts
            .into_iter()
            .enumerate()
            .map(|(i, t)| FieldDef { name: NUMS.get(i).copied().unwrap_or("_"), ty: l.ty(t) })
            .collect();
        (CtorKind::Tuple, fs)
    } else {
        (CtorKind::Unit, Vec::new())
    }
}

/// The type `self` has in a method of the impl or trait `parent`.
fn self_param_ty(l: &mut Lower, p: NodeId, self_ty: Ty) -> Ty {
    let ast = l.env.ast(l.k, l.file);
    let kids = ast.children(p);
    if let Some(&t) = kids.iter().find(|&&c| is_type(ast.kind(c))) {
        return l.ty(t);
    }
    let by_ref = kids.iter().any(|&c| ast.kind(c) == Kind::Op);
    let mutbl = kids.iter().any(|&c| ast.kind(c) == Kind::Keyword && ast.token(c).0 == "mut");
    if by_ref { l.tcx.mk(TyKind::Ref(if mutbl { Mutbl::Mut } else { Mutbl::Not }, self_ty)) } else { self_ty }
}

/// What `signature` works out for one item.
struct SigOut<'s> {
    preds: Vec<Pred>,
    adt: Option<AdtDef<'s>>,
    trait_def: Option<TraitDef>,
    fn_sig: Option<super::SigId>,
    /// Whether it counts as a signature (an impl's is counted with its
    /// header), and whether it had an error type.
    counted: bool,
    failed: bool,
}

fn signature<'s>(tcx: &TyCtxt<'s>, env: &Env<'_, 's>, d: DefId) -> Option<SigOut<'s>> {
    let kind = env.r.defs[d.0 as usize].kind;
    if matches!(kind, DefKind::Mod | DefKind::Macro | DefKind::Variant) {
        return None;
    }
    let (at, ast) = node_of(env, d);
    let node = at.2;
    let mut out = SigOut { preds: Vec::new(), adt: None, trait_def: None, fn_sig: None, counted: kind != DefKind::Impl, failed: false };
    let mut l = Lower { tcx, env, expanding: FastSet::default(), impl_self: None, holes: false, k: at.0, file: at.1, owner: d, failed: false };
    predicates(&mut l, node, &mut out.preds);
    match kind {
        DefKind::Struct | DefKind::Union => {
            let (ctor, fs) = fields(&mut l, node);
            out.adt = Some(AdtDef {
                kind: if kind == DefKind::Struct { AdtKind::Struct } else { AdtKind::Union },
                variants: vec![VariantDef { def: None, name: env.r.defs[d.0 as usize].name, kind: ctor, fields: fs }],
            });
        }
        DefKind::Enum => {
            let vs: Vec<NodeId> = ast.children(node).iter().copied().filter(|&v| ast.kind(v) == Kind::Variant).collect();
            let mut variants = Vec::new();
            for v in vs {
                let (ctor, fs) = fields(&mut l, v);
                let vdef = tcx.def_at.get(&(at.0, at.1, v)).copied();
                let name = ast.child(v, Kind::Name).map_or("", |n| ast.token(n).ident_name());
                variants.push(VariantDef { def: vdef, name, kind: ctor, fields: fs });
            }
            out.adt = Some(AdtDef { kind: AdtKind::Enum, variants });
        }
        DefKind::Fn | DefKind::AssocFn | DefKind::ForeignFn => {
            let parent = tcx.generics[d.0 as usize].parent;
            let self_ty = parent.and_then(|p| tcx.self_ty.get(&env.at_of(p)).copied());
            let mut inputs = Vec::new();
            let mut output = None;
            let mut variadic = false;
            let mut unsafety = false;
            let mut abi = if kind == DefKind::ForeignFn { Abi::C } else { Abi::Rust };
            for &c in ast.children(node) {
                match ast.kind(c) {
                    Kind::Keyword if ast.token(c).0 == "unsafe" => unsafety = true,
                    Kind::Abi => {
                        abi = if crate::cfg::string_value(ast.token(c).0) == "C" { Abi::C } else { Abi::Other }
                    }
                    Kind::Params => {
                        for &p in ast.children(c) {
                            match ast.kind(p) {
                                Kind::SelfParam => {
                                    let st = self_ty.unwrap_or_else(|| tcx.error());
                                    inputs.push(self_param_ty(&mut l, p, st));
                                }
                                Kind::Param => {
                                    let t = *ast.children(p).last().unwrap();
                                    inputs.push(l.ty(t));
                                }
                                Kind::Variadic => variadic = true,
                                _ => {}
                            }
                        }
                    }
                    Kind::RetType => output = Some(l.ty(ast.children(c)[0])),
                    _ => {}
                }
            }
            let output = output.unwrap_or_else(|| tcx.unit());
            let inputs = tcx.list(inputs);
            out.fn_sig = Some(tcx.sig(FnSig { inputs, output, unsafety, variadic, abi }));
        }
        DefKind::TypeAlias | DefKind::AssocType => {
            if kind == DefKind::AssocType
                && let Some(b) = ast.child(node, Kind::Bounds)
            {
                // `type Item: Bound;` in a trait: bounds on the projection.
                let _ = l.bounds(b);
            }
            let t = alias_ty(tcx, env, &mut l.expanding, d);
            l.failed |= tcx.has_error(t);
        }
        DefKind::Const | DefKind::Static | DefKind::AssocConst | DefKind::ForeignStatic => {
            if let Some(t) = ast.children(node).iter().copied().find(|&c| is_type(ast.kind(c))) {
                let t = l.ty(t);
                tcx.set_type_of(d, t);
            }
        }
        DefKind::Trait | DefKind::TraitAlias => {
            let supertraits = ast.child(node, Kind::Bounds).map(|b| l.bounds(b)).unwrap_or_default();
            let self_ty = tcx.self_ty[&at];
            for st in &supertraits {
                push_trait(tcx, &mut out.preds, self_ty, st.clone());
            }
            let is_auto = ast.children(node).iter().any(|&c| ast.kind(c) == Kind::Keyword && ast.token(c).0 == "auto");
            out.trait_def = Some(TraitDef { supertraits, is_auto });
        }
        _ => {}
    }
    out.failed = l.failed;
    Some(out)
}

/// The type an alias (or an impl's associated type) stands for, worked
/// out when first needed. A trait's associated type is its projection on
/// `Self` (or its default, if written).
pub(crate) fn alias_ty<'s>(tcx: &TyCtxt<'s>, env: &Env<'_, 's>, expanding: &mut FastSet<DefId>, d: DefId) -> Ty {
    if let Some(t) = tcx.type_of(d) {
        return t;
    }
    if !expanding.insert(d) {
        tcx.note_error("type alias that refers to itself");
        return tcx.error();
    }
    let (at, ast) = node_of(env, d);
    let value = ast.children(at.2).iter().copied().find(|&c| is_type(ast.kind(c)));
    let t = match value {
        Some(v) => {
            let mut l = Lower { tcx, env, expanding: std::mem::take(expanding), impl_self: None, holes: false, k: at.0, file: at.1, owner: d, failed: false };
            let t = l.ty(v);
            *expanding = l.expanding;
            t
        }
        None => {
            // A trait's associated type without a default: itself, as a
            // projection on `Self`.
            match generics_parent(env, d) {
                Some(t) if env.r.defs[t.0 as usize].kind == DefKind::Trait => {
                    let args = tcx.identity_args(t);
                    tcx.mk(TyKind::Projection(d, args))
                }
                _ => tcx.error(),
            }
        }
    };
    expanding.remove(&d);
    tcx.set_type_of(d, t)
}

/// The associated item `name` of trait `t` or one of its supertraits:
/// the item and the trait it is in.
pub(crate) fn find_assoc(env: &Env, t: DefId, name: &str) -> Option<(DefId, DefId)> {
    let mut seen = FastSet::default();
    find_assoc_in(env, t, name, &mut seen)
}

fn find_assoc_in(env: &Env, t: DefId, name: &str, seen: &mut FastSet<DefId>) -> Option<(DefId, DefId)> {
    if !seen.insert(t) {
        return None;
    }
    let r = env.r;
    if let Some(m) = r.defs[t.0 as usize].module
        && let Some(Res::Def(a)) = r.lookup(m, name, Ns::Type, m)
        && r.defs[a.0 as usize].kind == DefKind::AssocType
    {
        return Some((a, t));
    }
    // Supertraits, from the trait's bounds as written.
    let (at, ast) = node_of(env, t);
    let mut supers = Vec::new();
    if let Some(b) = ast.child(at.2, Kind::Bounds) {
        for &tb in ast.children(b) {
            if let Some(p) = ast.child(tb, Kind::Path)
                && let Some(Res::Def(s)) = r.res_of(at.0 as usize, at.1 as usize, p)
            {
                supers.push(*s);
            }
        }
    }
    supers.into_iter().find_map(|s| find_assoc_in(env, s, name, seen))
}

/// `t` with each of `params` replaced by the matching `args`.
pub(crate) fn subst(tcx: &TyCtxt, t: Ty, params: &[ParamId], args: &[Ty]) -> Ty {
    if params.is_empty() {
        return t;
    }
    let sub = |x: Ty| subst(tcx, x, params, args);
    let sub_list = |l: super::List| {
        let v: Vec<Ty> = tcx.tys(l).into_iter().map(sub).collect();
        tcx.list(v)
    };
    match tcx.kind(t) {
        TyKind::Param(p) => match params.iter().position(|&q| q == p) {
            Some(i) if i < args.len() => args[i],
            _ => t,
        },
        TyKind::Tuple(l) => tcx.mk(TyKind::Tuple(sub_list(l))),
        TyKind::Adt(d, l) => tcx.mk(TyKind::Adt(d, sub_list(l))),
        TyKind::FnDef(d, l) => tcx.mk(TyKind::FnDef(d, sub_list(l))),
        TyKind::Projection(d, l) => tcx.mk(TyKind::Projection(d, sub_list(l))),
        TyKind::Ref(m, x) => tcx.mk(TyKind::Ref(m, sub(x))),
        TyKind::Ptr(m, x) => tcx.mk(TyKind::Ptr(m, sub(x))),
        TyKind::Slice(x) => tcx.mk(TyKind::Slice(sub(x))),
        TyKind::Array(x, c) => tcx.mk(TyKind::Array(sub(x), c)),
        TyKind::FnPtr(s) => {
            let mut sig = tcx.sig_of(s);
            sig.inputs = sub_list(sig.inputs);
            sig.output = sub(sig.output);
            tcx.mk(TyKind::FnPtr(tcx.sig(sig)))
        }
        TyKind::Dyn(d) => {
            let mut data = tcx.dyn_of(d);
            for tr in &mut data.traits {
                tr.args = sub_list(tr.args);
                for b in &mut tr.bindings {
                    b.1 = sub(b.1);
                }
            }
            tcx.mk(TyKind::Dyn(tcx.dyn_(data)))
        }
        _ => t,
    }
}
