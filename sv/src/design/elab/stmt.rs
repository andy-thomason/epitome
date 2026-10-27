//! Statements (clauses 9–13), processes and subroutines.

use super::{Elab, R, Work, expr, lookup, ty};
use crate::ast::{Kind, NodeId};
use crate::design::types::{Type, TypeId, Types, UDim};
use crate::design::*;

// ---- subroutines ----

/// Pass 1: makes a subroutine for a `function` or `task` and binds its
/// name, so calls can come before it. A method declared outside its class
/// (`function C::f`) gives the body of the class's prototype (8.24).
pub(crate) fn declare_sub<'a>(e: &mut Elab<'a, '_>, sc: ScopeId, n: NodeId) -> R<'a> {
    if e.sub_of(sc, n).is_some() {
        return Ok(());
    }
    let kids = e.kids(n).to_vec();
    let Some(&name_node) = kids.iter().find(|&&k| matches!(e.kind(k), Kind::Name | Kind::Scope)) else {
        return Ok(());
    };
    let kind = if e.kind(n) == Kind::Task { SubKind::Task } else { SubKind::Function };
    if e.kind(name_node) == Kind::Scope && !e.ast.tokens_of(name_node).iter().any(|t| t.0 == "::") {
        // `task port.name` exports a task through an interface (25.7).
        return Ok(());
    }
    if e.kind(name_node) == Kind::Scope {
        let k = e.kids(name_node).to_vec();
        let cname = e.sym_of(k[0]);
        let mname = e.sym_of(k[1]);
        let Some(Binding::Class(c)) = lookup::lookup(e, sc, cname) else {
            return Err(e.err(k[0], format!("`{}` is not a class", e.name(cname))));
        };
        let cs = e.d.classes[c.0 as usize].scope;
        // The class may not have been elaborated yet; its prototype is
        // matched once it is.
        e.out_of_block.push((c, mname, n, cs));
        let _ = cs;
        return Ok(());
    }
    let s = e.sym_of(name_node);
    let quals = qualifiers(e, n);
    if e.name(s) == "new"
        && e.scope(sc).kind == ScopeKind::Class
        && let Some(&q) = quals.iter().find(|&&q| matches!(e.name(q), "static" | "virtual"))
    {
        return Err(e.err(name_node, format!("a constructor cannot be `{}` (8.7)", e.name(q))));
    }
    let lifetime = sub_lifetime(e, sc, n);
    let scope = e.new_scope(ScopeKind::Subroutine, Some(s), Some(sc), n);
    let id = SubId(e.d.subroutines.len() as u32);
    e.d.subroutines.push(Subroutine {
        kind,
        name: s,
        ret: Types::VOID,
        args: vec![],
        defaults: vec![],
        scope,
        body: vec![],
        lifetime,
        node: n,
        qualifiers: quals,
        elaborated: false,
    });
    // A prototype declared earlier in a class (`extern` or `pure virtual`).
    if let Some(Binding::Subroutine(old)) = e.scope(sc).names.get(&s).copied()
        && e.d.subroutines[old.0 as usize].body.is_empty()
        && matches!(e.kind(e.d.subroutines[old.0 as usize].node), Kind::Prototype)
    {
        e.scope_mut(sc).names.insert(s, Binding::Subroutine(id));
        return Ok(());
    }
    e.declare(sc, s, Binding::Subroutine(id), name_node)?;
    Ok(())
}

fn qualifiers(e: &mut Elab, n: NodeId) -> Vec<crate::design::Sym> {
    let mut out = Vec::new();
    if let Some(q) = e.child(n, Kind::Qualifiers) {
        for &k in e.kids(q).to_vec().iter() {
            out.push(e.sym_of(k));
        }
    }
    for &k in e.kids(n).to_vec().iter() {
        if e.kind(k) == Kind::Keyword {
            out.push(e.sym_of(k));
        }
    }
    out
}

fn sub_lifetime(e: &Elab, sc: ScopeId, n: NodeId) -> Lifetime {
    if e.has_keyword(n, "automatic") {
        return Lifetime::Automatic;
    }
    if e.has_keyword(n, "static") {
        return Lifetime::Static;
    }
    // Class methods are automatic (8.6); so are subroutines in a design
    // element declared automatic.
    let mut cur = Some(sc);
    while let Some(id) = cur {
        let s = e.scope(id);
        match s.kind {
            ScopeKind::Class => return Lifetime::Automatic,
            ScopeKind::Spec | ScopeKind::Package => {
                return if e.has_keyword(s.node, "automatic") { Lifetime::Automatic } else { Lifetime::Static };
            }
            _ => cur = s.parent,
        }
    }
    Lifetime::Static
}

/// Elaborates a subroutine's return type and formal arguments, once.
pub(crate) fn sub_signature<'a>(e: &mut Elab<'a, '_>, sub: SubId) -> R<'a> {
    if !e.sig_done.insert(sub) {
        return Ok(());
    }
    let n = e.d.subroutines[sub.0 as usize].node;
    let scope = e.d.subroutines[sub.0 as usize].scope;
    let parent = e.scope(scope).parent.unwrap();
    let kids = e.kids(n).to_vec();
    let is_fn = e.d.subroutines[sub.0 as usize].kind == SubKind::Function;
    // The return type comes before the name.
    let name_pos = kids.iter().position(|&k| matches!(e.kind(k), Kind::Name | Kind::Scope)).unwrap_or(0);
    let ret = if is_fn {
        match kids[..name_pos].iter().rev().find(|&&k| ty::is_type(e.kind(k))) {
            Some(&t) if e.kind(t) == Kind::ImplicitType && e.kids(t).is_empty() => Types::LOGIC,
            Some(&t) => ty::data_type(e, parent, t)?,
            None => {
                // A constructor, `function new`, returns nothing.
                if e.name(e.d.subroutines[sub.0 as usize].name) == "new" { Types::VOID } else { Types::LOGIC }
            }
        }
    } else {
        Types::VOID
    };
    e.d.subroutines[sub.0 as usize].ret = ret;
    let mut args = Vec::new();
    let mut defaults = Vec::new();
    if let Some(ports) = kids.iter().copied().find(|&k| e.kind(k) == Kind::TfPorts) {
        let mut dir = Dir::Input;
        let mut prev_ty: Option<TypeId> = None;
        for &p in e.kids(ports).to_vec().iter() {
            if e.kind(p) != Kind::TfPort {
                continue;
            }
            // `default` in a constructor's arguments stands for the base
            // class constructor's arguments (8.17).
            if e.ast.tokens_of(p).len() == 1 && e.ast.tokens_of(p)[0].0 == "default" {
                for (o, d) in base_ctor_args(e, scope)? {
                    let obj = e.obj(o);
                    let (name, ty, node, dir) = (obj.name, obj.ty, obj.node, obj.dir);
                    let copy = e.new_object(Object {
                        name,
                        kind: ObjKind::Variable,
                        ty,
                        scope,
                        node,
                        lifetime: Lifetime::Automatic,
                        dir,
                        init: None,
                        value: None,
                        qualifiers: vec![],
                    });
                    e.declare(scope, name, Binding::Object(copy), p)?;
                    args.push(copy);
                    defaults.push(d);
                }
                continue;
            }
            let pk = e.kids(p).to_vec();
            let kws: Vec<&str> = pk.iter().filter(|&&k| e.kind(k) == Kind::Keyword).map(|&k| e.text(k)).collect();
            let explicit = kws.iter().find_map(|&k| match k {
                "input" => Some(Dir::Input),
                "output" => Some(Dir::Output),
                "inout" => Some(Dir::Inout),
                "ref" => Some(Dir::Ref),
                _ => None,
            });
            let is_const = kws.contains(&"const");
            if let Some(d) = explicit {
                dir = if d == Dir::Ref && is_const { Dir::ConstRef } else { d };
            }
            let tn = pk.iter().copied().find(|&k| ty::is_type(e.kind(k)));
            let implicit = tn.is_none_or(|t| e.kind(t) == Kind::ImplicitType && e.kids(t).is_empty());
            let t = match tn {
                Some(t) if !implicit => ty::data_type(e, scope, t)?,
                Some(t) if e.kind(t) == Kind::ImplicitType && !e.kids(t).is_empty() => ty::data_type(e, scope, t)?,
                _ => {
                    if explicit.is_some() || prev_ty.is_none() {
                        Types::LOGIC
                    } else {
                        prev_ty.unwrap()
                    }
                }
            };
            let Some(d) = pk.iter().copied().find(|&k| e.kind(k) == Kind::Declarator) else { continue };
            let dk = e.kids(d).to_vec();
            let s = e.sym_of(dk[0]);
            let dims: Vec<NodeId> = dk[1..].iter().copied().filter(|&k| ty::is_dim(e.kind(k))).collect();
            let default = dk[1..].iter().copied().find(|&k| !ty::is_dim(e.kind(k)));
            let at = ty::unpacked(e, scope, t, &dims)?;
            prev_ty = Some(t);
            let lifetime = e.d.subroutines[sub.0 as usize].lifetime;
            if matches!(dir, Dir::Ref | Dir::ConstRef) && lifetime == Lifetime::Static {
                return Err(e.err(dk[0], "a static subroutine cannot have a `ref` argument (13.5.2)"));
            }
            let sym_const = e.sym("const");
            let o = e.new_object(Object {
                name: s,
                kind: ObjKind::Variable,
                ty: at,
                scope,
                node: d,
                lifetime,
                dir: Some(dir),
                init: None,
                value: None,
                qualifiers: if is_const { vec![sym_const] } else { vec![] },
            });
            e.declare(scope, s, Binding::Object(o), dk[0])?;
            args.push(o);
            defaults.push(default);
        }
    }
    // Old-style declarations in the body (`input a;`).
    for &k in &kids {
        if e.kind(k) == Kind::TfPortDecl {
            let pk = e.kids(k).to_vec();
            let dir = match e.text(pk[0]) {
                "output" => Dir::Output,
                "inout" => Dir::Inout,
                "ref" => Dir::Ref,
                _ => Dir::Input,
            };
            let tn = pk.iter().copied().find(|&x| ty::is_type(e.kind(x)));
            let t = match tn {
                Some(t) => ty::data_type(e, scope, t)?,
                None => Types::LOGIC,
            };
            for d in pk.iter().copied().filter(|&x| e.kind(x) == Kind::Declarator).collect::<Vec<_>>() {
                let dk = e.kids(d).to_vec();
                let s = e.sym_of(dk[0]);
                let dims: Vec<NodeId> = dk[1..].iter().copied().filter(|&k| ty::is_dim(e.kind(k))).collect();
                let at = ty::unpacked(e, scope, t, &dims)?;
                let lifetime = e.d.subroutines[sub.0 as usize].lifetime;
                let o = e.new_object(Object {
                    name: s,
                    kind: ObjKind::Variable,
                    ty: at,
                    scope,
                    node: d,
                    lifetime,
                    dir: Some(dir),
                    init: None,
                    value: None,
                    qualifiers: vec![],
                });
                e.declare(scope, s, Binding::Object(o), dk[0])?;
                args.push(o);
                defaults.push(None);
            }
        }
    }
    // A function's result is a variable named after it (13.4.1).
    if is_fn && ret != Types::VOID {
        let s = e.d.subroutines[sub.0 as usize].name;
        let sym_result = e.sym("result");
        let o = e.new_object(Object {
            name: s,
            kind: ObjKind::Variable,
            ty: ret,
            scope,
            node: n,
            lifetime: e.d.subroutines[sub.0 as usize].lifetime,
            dir: None,
            init: None,
            value: None,
            qualifiers: vec![sym_result],
        });
        e.scope_mut(scope).names.insert(s, Binding::Object(o));
    }
    let sref = &mut e.d.subroutines[sub.0 as usize];
    sref.args = args;
    sref.defaults = defaults;
    Ok(())
}

/// The arguments of the base class's constructor, for `new(default)`.
fn base_ctor_args<'a>(e: &mut Elab<'a, '_>, scope: ScopeId) -> R<'a, Vec<(ObjId, Option<NodeId>)>> {
    let class_scope = e.scope(scope).parent;
    let class = class_scope.and_then(|c| lookup::class_of_scope(e, c));
    let base = class.and_then(|c| e.d.classes[c.0 as usize].base);
    let Some(base) = base else {
        let n = e.scope(scope).node;
        return Err(e.err(n, "`default` in a constructor's arguments needs a base class to take them from (8.17)"));
    };
    let new = e.sym("new");
    let bs = e.d.classes[base.0 as usize].scope;
    let Some(Binding::Subroutine(ctor)) = e.scope(bs).names.get(&new).copied() else { return Ok(vec![]) };
    sub_signature(e, ctor)?;
    let s = &e.d.subroutines[ctor.0 as usize];
    Ok(s.args.iter().copied().zip(s.defaults.iter().copied()).collect())
}

/// Pass 3: a subroutine's body.
pub(crate) fn sub_body<'a>(e: &mut Elab<'a, '_>, sub: SubId) -> R<'a> {
    if e.d.subroutines[sub.0 as usize].elaborated {
        return Ok(());
    }
    sub_signature(e, sub)?;
    e.d.subroutines[sub.0 as usize].elaborated = true;
    let n = e.d.subroutines[sub.0 as usize].node;
    if e.kind(n) == Kind::Prototype {
        return Ok(());
    }
    let scope = e.d.subroutines[sub.0 as usize].scope;
    let kids = e.kids(n).to_vec();
    let saved = e.current_sub.replace(sub);
    let saved_loop = std::mem::replace(&mut e.loop_depth, 0);
    let saved_forks = std::mem::take(&mut e.forks);
    let r = (|| -> R<'a> {
        let mut body = Vec::new();
        let name_pos = kids.iter().position(|&k| matches!(e.kind(k), Kind::Name | Kind::Scope)).unwrap_or(0);
        let rest: Vec<NodeId> = kids[name_pos + 1..]
            .iter()
            .copied()
            .filter(|&k| !matches!(e.kind(k), Kind::TfPorts | Kind::TfPortDecl | Kind::EndLabel | Kind::Attributes))
            .collect();
        e.declare_pass(scope, &rest)?;
        for k in rest {
            if is_statement(e.kind(k)) {
                body.push(stmt(e, scope, k)?);
            } else {
                e.item(scope, k)?;
            }
        }
        e.d.subroutines[sub.0 as usize].body = body;
        Ok(())
    })();
    e.current_sub = saved;
    e.loop_depth = saved_loop;
    e.forks = saved_forks;
    r?;
    super::checks::sub_done(e, sub)?;
    super::checks::constructor_body(e, sub)?;
    Ok(())
}

/// A subroutine prototype: `extern`, `pure virtual`, or a DPI import.
pub(crate) fn prototype<'a>(e: &mut Elab<'a, '_>, sc: ScopeId, n: NodeId) -> R<'a> {
    // `import "DPI-C"` and `extern [forkjoin]` wrap the prototype itself.
    let proto = e.child(n, Kind::Prototype).unwrap_or(n);
    let kids = e.kids(proto).to_vec();
    let Some(&name_node) = kids.iter().find(|&&k| e.kind(k) == Kind::Name) else { return Ok(()) };
    // A DPI import may give a C name first; the SystemVerilog name is last.
    let name_node = *kids.iter().filter(|&&k| e.kind(k) == Kind::Name).next_back().unwrap_or(&name_node);
    let s = e.sym_of(name_node);
    let kind = if kids.iter().any(|&k| e.kind(k) == Kind::Keyword && e.text(k) == "task") { SubKind::Task } else { SubKind::Function };
    let mut quals = qualifiers(e, proto);
    if proto != n {
        quals.extend(qualifiers(e, n));
    }
    if e.kind(n) == Kind::DpiImport {
        quals.push(e.sym("import"));
        super::checks::dpi_import(e, sc, n)?;
    }
    let scope = e.new_scope(ScopeKind::Subroutine, Some(s), Some(sc), proto);
    let id = SubId(e.d.subroutines.len() as u32);
    e.d.subroutines.push(Subroutine {
        kind,
        name: s,
        ret: Types::VOID,
        args: vec![],
        defaults: vec![],
        scope,
        body: vec![],
        lifetime: if e.kind(n) == Kind::DpiImport { Lifetime::Automatic } else { sub_lifetime(e, sc, proto) },
        node: proto,
        qualifiers: quals,
        elaborated: true,
    });
    // An out-of-block body declared earlier takes this prototype's place.
    if let Some(Binding::Subroutine(old)) = e.scope(sc).names.get(&s).copied() {
        let on = e.d.subroutines[old.0 as usize].node;
        if e.kind(on) == Kind::Function || e.kind(on) == Kind::Task {
            return Err(e.err(name_node, format!("`{}` is already declared in this scope", e.name(s))));
        }
    }
    e.declare(sc, s, Binding::Subroutine(id), name_node)?;
    e.push_item(sc, Item::Subroutine(id));
    sub_signature(e, id)?;
    // Bodies written outside the class for this prototype (8.24).
    let pending: Vec<(ClassId, crate::design::Sym, NodeId, ScopeId)> =
        e.out_of_block.iter().copied().filter(|(c, m, _, _)| e.d.classes[c.0 as usize].scope == sc && *m == s).collect();
    for (_, _, body, _) in pending {
        attach_body(e, id, body)?;
    }
    Ok(())
}

/// Gives prototype `id` the out-of-block body `body`.
pub(crate) fn attach_body<'a>(e: &mut Elab<'a, '_>, id: SubId, body: NodeId) -> R<'a> {
    let scope = e.d.subroutines[id.0 as usize].scope;
    if e.kind(e.d.subroutines[id.0 as usize].node) != Kind::Prototype {
        return Err(e.err(body, format!("`{}` already has a body outside its class (8.24)", e.name(e.d.subroutines[id.0 as usize].name))));
    }
    let proto_args: Vec<(TypeId, Option<Dir>)> =
        e.d.subroutines[id.0 as usize].args.iter().map(|&a| (e.obj(a).ty, e.obj(a).dir)).collect();
    let proto_ret = e.d.subroutines[id.0 as usize].ret;
    e.d.subroutines[id.0 as usize].node = body;
    e.d.subroutines[id.0 as usize].elaborated = false;
    e.scope_mut(scope).node = body;
    e.sig_done.remove(&id);
    // Rebuild the formals from the body's header.
    let names: Vec<crate::design::Sym> = e.d.subroutines[id.0 as usize].args.iter().map(|&o| e.obj(o).name).collect();
    for s in names {
        e.scope_mut(scope).names.remove(&s);
    }
    let fname = e.d.subroutines[id.0 as usize].name;
    e.scope_mut(scope).names.remove(&fname);
    sub_signature(e, id)?;
    let body_args: Vec<(TypeId, Option<Dir>)> =
        e.d.subroutines[id.0 as usize].args.iter().map(|&a| (e.obj(a).ty, e.obj(a).dir)).collect();
    if body_args != proto_args || e.d.subroutines[id.0 as usize].ret != proto_ret {
        return Err(e.err(body, format!("the body of `{}` does not match its prototype in the class (8.24)", e.name(fname))));
    }
    e.defer(Work::Body(id));
    Ok(())
}

// ---- processes ----

pub(crate) fn process<'a>(e: &mut Elab<'a, '_>, sc: ScopeId, n: NodeId) -> R<'a> {
    let kids = e.kids(n).to_vec();
    let kind = match e.text(kids[0]) {
        "initial" => ProcKind::Initial,
        "always" => ProcKind::Always,
        "always_comb" => ProcKind::AlwaysComb,
        "always_latch" => ProcKind::AlwaysLatch,
        "always_ff" => ProcKind::AlwaysFf,
        _ => ProcKind::Final,
    };
    let body_node = *kids.iter().skip(1).find(|&&k| e.kind(k) != Kind::Attributes).unwrap();
    e.proc_kind = Some(kind);
    e.ff_event_seen = false;
    let r = stmt(e, sc, body_node);
    e.proc_kind = None;
    let body = r?;
    let region = if e.in_program_or_interface(sc) == Some(DefKind::Program) || lookup::enclosing(e, sc, ScopeKind::Other).is_some() {
        RegionSet::Reactive
    } else {
        RegionSet::Active
    };
    let (reads, writes) = super::checks::read_write_sets(e, body);
    let sensitivity = match kind {
        ProcKind::AlwaysComb | ProcKind::AlwaysLatch => {
            // What the process reads, less what it only uses locally
            // (9.2.2.2).
            reads.iter().copied().filter(|o| e.obj(*o).lifetime == Lifetime::Static && !local_to(e, *o, sc)).collect()
        }
        _ => vec![],
    };
    super::checks::process_body(e, sc, kind, body, &writes, n)?;
    let id = ProcId(e.d.processes.len() as u32);
    e.d.processes.push(Process { kind, body, scope: sc, region, node: n, reads, writes, sensitivity });
    e.push_item(sc, Item::Process(id));
    Ok(())
}

/// Whether `o` is declared inside the process (in a block of its own).
fn local_to(e: &Elab, o: ObjId, sc: ScopeId) -> bool {
    let os = e.obj(o).scope;
    os != sc && e.scope(os).kind == ScopeKind::Block
}

// ---- statements ----

pub(crate) fn is_statement(k: Kind) -> bool {
    matches!(
        k,
        Kind::Labeled
            | Kind::NullStatement
            | Kind::Block
            | Kind::Assign
            | Kind::ExprStatement
            | Kind::ProceduralContinuous
            | Kind::If
            | Kind::Case
            | Kind::Randcase
            | Kind::Loop
            | Kind::DoWhile
            | Kind::For
            | Kind::Foreach
            | Kind::Jump
            | Kind::Timed
            | Kind::Wait
            | Kind::EventTrigger
            | Kind::Disable
            | Kind::ClockingDrive
            | Kind::Randsequence
            | Kind::ImmediateAssertion
            | Kind::ConcurrentAssertion
            | Kind::Expect
    )
}

pub(crate) fn stmt<'a>(e: &mut Elab<'a, '_>, sc: ScopeId, n: NodeId) -> R<'a, StmtId> {
    e.depth += 1;
    if e.depth > 2000 {
        e.depth -= 1;
        return Err(e.err(n, "the statement is nested too deeply"));
    }
    let r = stmt_inner(e, sc, n);
    e.depth -= 1;
    r
}

fn stmt_inner<'a>(e: &mut Elab<'a, '_>, sc: ScopeId, n: NodeId) -> R<'a, StmtId> {
    let kids: Vec<NodeId> = e.kids(n).iter().copied().filter(|&k| e.kind(k) != Kind::Attributes).collect();
    let kind = match e.kind(n) {
        Kind::NullStatement | Kind::Empty => StmtKind::Null,
        Kind::Labeled => {
            // `label: stmt` names a block (9.3.5).
            let inner = kids[1];
            if e.kind(inner) == Kind::Block {
                let s = e.sym_of(kids[0]);
                return block(e, sc, inner, Some((s, kids[0])));
            }
            return stmt(e, sc, inner);
        }
        Kind::Block => return block(e, sc, n, None),
        Kind::Assign => {
            let op = e.text(kids[1]);
            let timing_node = kids.iter().copied().find(|&k| matches!(e.kind(k), Kind::Delay | Kind::Event | Kind::RepeatEvent | Kind::CycleDelay));
            let rhs_node = *kids.last().unwrap();
            let lhs = expr::expr(e, sc, kids[0])?;
            let nonblocking = op == "<=";
            super::checks::lvalue(e, sc, lhs, kids[0], nonblocking)?;
            let lt = e.ex(lhs).ty;
            let timing = match timing_node {
                Some(t) => Some(timing(e, sc, t)?),
                None => None,
            };
            let akind = if op == "=" {
                AssignKind::Blocking
            } else if nonblocking {
                AssignKind::NonBlocking
            } else {
                match expr::compound_op(op) {
                    Some(b) => AssignKind::Compound(b),
                    None => return Err(e.err(kids[1], format!("`{op}` is not an assignment operator"))),
                }
            };
            let rhs = match akind {
                AssignKind::Compound(op) => {
                    // `a op= b` is `a = a op b` (11.4.1).
                    let r = expr::bind(e, sc, rhs_node)?;
                    if matches!(op, BinOp::Shl | BinOp::Shr | BinOp::AShl | BinOp::AShr) {
                        expr::selfdet(e, r)
                    } else {
                        let r = expr::context_to(e, r, lt);
                        expr::assign_conv(e, r, lt)
                    }
                }
                _ => expr::rvalue(e, sc, rhs_node, lt)?,
            };
            super::checks::instance_types(e, lhs, rhs, n)?;
            super::checks::stream_widths(e, lhs, rhs, n)?;
            StmtKind::Assign { kind: akind, lhs, rhs, timing }
        }
        Kind::ExprStatement => {
            let x = expr::expr(e, sc, kids[0])?;
            super::checks::expr_statement(e, x, kids[0])?;
            StmtKind::Expr(x)
        }
        Kind::ProceduralContinuous => {
            let kw = e.sym_of(kids[0]);
            let target = kids[1];
            if e.kind(target) == Kind::Assign {
                let ak = e.kids(target).to_vec();
                let lhs = expr::expr(e, sc, ak[0])?;
                let lt = e.ex(lhs).ty;
                let rhs = expr::rvalue(e, sc, *ak.last().unwrap(), lt)?;
                e.current_rhs = Some(rhs);
                let r = super::checks::proc_continuous(e, sc, e.name(kw), lhs, ak[0]);
                e.current_rhs = None;
                r?;
                StmtKind::ProcContinuous { keyword: kw, lhs, rhs: Some(rhs) }
            } else {
                let lhs = expr::expr(e, sc, target)?;
                super::checks::proc_continuous(e, sc, e.name(kw), lhs, target)?;
                StmtKind::ProcContinuous { keyword: kw, lhs, rhs: None }
            }
        }
        Kind::If => {
            let check = check_of(e, &kids);
            // The condition is the first child after `unique` or `priority`.
            let pred = *kids.iter().find(|&&k| e.kind(k) != Kind::Keyword).unwrap();
            let (cond, inner) = if e.kind(pred) == Kind::CondPredicate {
                cond_predicate(e, sc, pred)?
            } else {
                (expr::expr(e, sc, pred)?, sc)
            };
            let rest: Vec<NodeId> = kids.iter().copied().skip_while(|&k| k != pred).skip(1).collect();
            let then = stmt(e, inner, rest[0])?;
            let else_ = match rest.get(1) {
                Some(&s) => Some(stmt(e, sc, s)?),
                None => None,
            };
            StmtKind::If { check, cond, then, else_ }
        }
        Kind::Case => case(e, sc, n, &kids)?,
        Kind::Loop => {
            let kw = e.text(kids[0]);
            e.loop_depth += 1;
            let r = (|| -> R<'a, StmtKind> {
                Ok(match kw {
                    "forever" => StmtKind::Forever(stmt(e, sc, kids[1])?),
                    "repeat" => {
                        let c = expr::expr(e, sc, kids[1])?;
                        StmtKind::Repeat(c, stmt(e, sc, kids[2])?)
                    }
                    _ => {
                        let c = expr::expr(e, sc, kids[1])?;
                        StmtKind::While(c, stmt(e, sc, kids[2])?)
                    }
                })
            })();
            e.loop_depth -= 1;
            r?
        }
        Kind::DoWhile => {
            e.loop_depth += 1;
            let body = stmt(e, sc, kids[0]);
            e.loop_depth -= 1;
            let c = expr::expr(e, sc, kids[1])?;
            StmtKind::DoWhile(body?, c)
        }
        Kind::For => for_loop(e, sc, n, &kids)?,
        Kind::Foreach => foreach(e, sc, n, &kids)?,
        Kind::Jump => {
            let kw = e.text(kids[0]);
            match kw {
                "return" => {
                    let v = kids.get(1).copied();
                    if !e.forks.is_empty() {
                        return Err(e.err(n, "`return` cannot be used inside a fork (9.3.2)"));
                    }
                    super::checks::return_stmt(e, v, n)?;
                    let x = match (v, e.current_sub) {
                        (Some(v), Some(s)) => {
                            let rt = e.d.subroutines[s.0 as usize].ret;
                            Some(expr::rvalue(e, sc, v, rt)?)
                        }
                        (Some(v), None) => Some(expr::expr(e, sc, v)?),
                        _ => None,
                    };
                    StmtKind::Return(x)
                }
                "break" | "continue" => {
                    if e.loop_depth == 0 {
                        return Err(e.err(n, format!("`{kw}` can only be used inside a loop (12.8)")));
                    }
                    if kw == "break" { StmtKind::Break } else { StmtKind::Continue }
                }
                _ => StmtKind::Opaque,
            }
        }
        Kind::Timed => {
            super::checks::timing_allowed(e, sc, kids[0])?;
            let t = timing(e, sc, kids[0])?;
            let s = stmt(e, sc, kids[1])?;
            StmtKind::Timed(t, s)
        }
        Kind::Wait => {
            let kw = e.text(kids[0]);
            super::checks::timing_allowed(e, sc, n)?;
            match kw {
                "wait" if e.ast.tokens_of(n).get(1).is_some_and(|t| t.0 == "fork") => StmtKind::WaitFork,
                "wait" => {
                    let c = expr::expr(e, sc, kids[1])?;
                    let s = match kids.get(2) {
                        Some(&s) => stmt(e, sc, s)?,
                        None => e.new_stmt(StmtKind::Null, n),
                    };
                    StmtKind::Wait(c, s)
                }
                _ => StmtKind::Opaque,
            }
        }
        Kind::EventTrigger => {
            let nonblocking = e.text(kids[0]) == "->>";
            let target = *kids.last().unwrap();
            let x = expr::expr(e, sc, target)?;
            StmtKind::Trigger { nonblocking, event: x }
        }
        Kind::Disable => {
            if e.kind(kids[0]) == Kind::Keyword {
                StmtKind::DisableFork
            } else {
                let x = disable_target(e, sc, kids[0])?;
                StmtKind::Disable(x)
            }
        }
        Kind::ImmediateAssertion => {
            let kw = e.sym_of(kids[0]);
            let cond_node = *kids.iter().find(|&&k| !matches!(e.kind(k), Kind::Keyword | Kind::Op | Kind::ActionBlock)).unwrap();
            let cond = expr::expr(e, sc, cond_node)?;
            let deferred = kids.iter().any(|&k| matches!(e.kind(k), Kind::Op) || (e.kind(k) == Kind::Keyword && e.text(k) == "final"));
            let (mut pass, mut fail) = (None, None);
            if let Some(ab) = kids.iter().copied().find(|&k| e.kind(k) == Kind::ActionBlock) {
                super::checks::action_block(e, ab, deferred)?;
                let toks = e.ast.tokens_of(ab);
                let has_else = toks.iter().any(|t| t.0 == "else");
                let abk: Vec<NodeId> = e.kids(ab).to_vec();
                match (abk.len(), has_else) {
                    (2, _) => {
                        if e.kind(abk[0]) != Kind::Empty {
                            pass = Some(stmt(e, sc, abk[0])?);
                        }
                        fail = Some(stmt(e, sc, abk[1])?);
                    }
                    (1, true) if toks.first().is_some_and(|t| t.0 == "else") => fail = Some(stmt(e, sc, abk[0])?),
                    (1, _) => pass = Some(stmt(e, sc, abk[0])?),
                    _ => {}
                }
            }
            StmtKind::Assert { keyword: kw, cond, pass, fail }
        }
        Kind::Randcase | Kind::Randsequence | Kind::ClockingDrive | Kind::ConcurrentAssertion | Kind::Expect => {
            if e.kind(n) == Kind::ClockingDrive {
                super::checks::clocking_drive(e, sc, n)?;
            }
            StmtKind::Opaque
        }
        _ => {
            // A declaration among statements: elaborate it as an item.
            e.item(sc, n)?;
            StmtKind::Null
        }
    };
    Ok(e.new_stmt(kind, n))
}

fn check_of(e: &Elab, kids: &[NodeId]) -> Check {
    match kids.first().map(|&k| (e.kind(k), e.text(k))) {
        Some((Kind::Keyword, "unique")) => Check::Unique,
        Some((Kind::Keyword, "unique0")) => Check::Unique0,
        Some((Kind::Keyword, "priority")) => Check::Priority,
        _ => Check::None,
    }
}

/// A condition with `matches` patterns and `&&&` guards (12.6): its
/// pattern variables are declared in a scope of their own, which the
/// guards and the statement it controls see.
fn cond_predicate<'a>(e: &mut Elab<'a, '_>, sc: ScopeId, p: NodeId) -> R<'a, (ExprId, ScopeId)> {
    let k = e.kids(p).to_vec();
    let scope = e.new_scope(ScopeKind::Block, None, Some(sc), p);
    pattern_vars(e, scope, p)?;
    let mut parts = Vec::new();
    for &x in &k {
        if e.kind(x) == Kind::CondPattern {
            let v = e.kids(x)[0];
            parts.push(expr::expr(e, sc, v)?);
        } else {
            parts.push(expr::expr(e, scope, x)?);
        }
    }
    Ok((e.new_expr(ExprKind::Opaque, Types::BIT, p, parts), scope))
}

/// Declares the variables a pattern binds: `.x` (12.6.1). Their types
/// come from the tagged union being matched, which is not modelled yet.
pub(crate) fn pattern_vars<'a>(e: &mut Elab<'a, '_>, scope: ScopeId, n: NodeId) -> R<'a> {
    for &k in e.kids(n).to_vec().iter() {
        if e.kind(k) == Kind::Pattern {
            let dot = e.ast.tokens_of(k).first().is_some_and(|t| t.0 == ".");
            let kids = e.kids(k).to_vec();
            if dot && kids.len() == 1 && e.kind(kids[0]) == Kind::Name {
                let s = e.sym_of(kids[0]);
                let o = e.new_object(Object {
                    name: s,
                    kind: ObjKind::Variable,
                    ty: Types::UNKNOWN,
                    scope,
                    node: kids[0],
                    lifetime: Lifetime::Automatic,
                    dir: None,
                    init: None,
                    value: None,
                    qualifiers: vec![],
                });
                e.declare(scope, s, Binding::Object(o), kids[0])?;
                continue;
            }
        }
        if matches!(e.kind(k), Kind::Pattern | Kind::CondPattern | Kind::AssignmentPattern | Kind::PatternKey) {
            pattern_vars(e, scope, k)?;
        }
    }
    Ok(())
}

fn block<'a>(e: &mut Elab<'a, '_>, sc: ScopeId, n: NodeId, label: Option<(crate::design::Sym, NodeId)>) -> R<'a, StmtId> {
    let kids: Vec<NodeId> = e.kids(n).iter().copied().filter(|&k| e.kind(k) != Kind::Attributes).collect();
    let fork = e.text(kids[0]) == "fork";
    let join = if fork {
        let last_kw = kids.iter().rev().find(|&&k| e.kind(k) == Kind::Keyword).map(|&k| e.text(k));
        Some(match last_kw {
            Some("join_any") => Join::Any,
            Some("join_none") => Join::None,
            _ => Join::All,
        })
    } else {
        None
    };
    if fork {
        super::checks::fork_allowed(e, sc, n)?;
    }
    // Processes forked with `join_none` may wait, even in a function.
    let saved_sub = if join == Some(Join::None) { e.current_sub.take() } else { e.current_sub };
    if let Some(j) = join {
        e.forks.push(j);
    }
    let r = block_body(e, sc, n, label, &kids, join);
    if join.is_some() {
        e.forks.pop();
    }
    e.current_sub = saved_sub;
    r
}

fn block_body<'a>(
    e: &mut Elab<'a, '_>,
    sc: ScopeId,
    n: NodeId,
    label: Option<(crate::design::Sym, NodeId)>,
    kids: &[NodeId],
    join: Option<Join>,
) -> R<'a, StmtId> {
    let name = match (label, kids.get(1).filter(|&&k| e.kind(k) == Kind::Name)) {
        (Some((s, _)), _) => Some(s),
        (None, Some(&nm)) => Some(e.sym_of(nm)),
        _ => None,
    };
    let decls: Vec<NodeId> =
        kids.iter().copied().filter(|&k| !is_statement(e.kind(k)) && !matches!(e.kind(k), Kind::Keyword | Kind::Name | Kind::EndLabel)).collect();
    let scope = if name.is_some() || !decls.is_empty() {
        let s = e.new_scope(ScopeKind::Block, name, Some(sc), n);
        if let Some(nm) = name {
            let at = label.map(|l| l.1).or(kids.get(1).copied()).unwrap_or(n);
            if !e.scope(sc).names.contains_key(&nm) {
                e.declare(sc, nm, Binding::Scope(s), at)?;
            }
        }
        Some(s)
    } else {
        None
    };
    let inner = scope.unwrap_or(sc);
    e.declare_pass(inner, &decls)?;
    let mut stmts = Vec::new();
    for &k in kids {
        if is_statement(e.kind(k)) {
            stmts.push(stmt(e, inner, k)?);
        } else if !matches!(e.kind(k), Kind::Keyword | Kind::Name | Kind::EndLabel) {
            e.item(inner, k)?;
        }
    }
    Ok(e.new_stmt(StmtKind::Block { join, scope, stmts }, n))
}

fn case<'a>(e: &mut Elab<'a, '_>, sc: ScopeId, n: NodeId, kids: &[NodeId]) -> R<'a, StmtKind> {
    let check = check_of(e, kids);
    let kw_node = *kids.iter().find(|&&k| e.kind(k) == Kind::Keyword && e.text(k).starts_with("case")).unwrap();
    let mut kind = match e.text(kw_node) {
        "casez" => CaseKind::CaseZ,
        "casex" => CaseKind::CaseX,
        _ => CaseKind::Case,
    };
    let after: Vec<NodeId> = kids.iter().copied().skip_while(|&k| k != kw_node).skip(1).collect();
    let value_node = after[0];
    if after.iter().any(|&k| e.kind(k) == Kind::Keyword && e.text(k) == "inside") {
        kind = CaseKind::Inside;
    }
    let matches = after.iter().any(|&k| e.kind(k) == Kind::Keyword && e.text(k) == "matches");
    let v = expr::bind(e, sc, value_node)?;
    let item_nodes: Vec<NodeId> = after.iter().copied().filter(|&k| e.kind(k) == Kind::CaseItem).collect();
    // The case expression and the item expressions are sized together
    // (12.5).
    let mut w = e.d.types.width(e.ex(v).ty);
    let mut signed = e.d.types.is_signed(e.ex(v).ty);
    let mut four = e.d.types.is_four_state(e.ex(v).ty);
    let mut item_exprs: Vec<(Vec<ExprId>, NodeId, ScopeId)> = Vec::new();
    let mut default = None;
    let mut defaults = 0;
    for &it in &item_nodes {
        let ik: Vec<NodeId> = e.kids(it).iter().copied().filter(|&k| e.kind(k) != Kind::Attributes).collect();
        let body = *ik.last().unwrap();
        if ik.len() == 1 || (ik.len() == 2 && e.kind(ik[0]) == Kind::Keyword) {
            defaults += 1;
            if defaults > 1 {
                return Err(e.err(it, "a case statement can have only one default item (12.5)"));
            }
            default = Some(body);
            continue;
        }
        let mut xs = Vec::new();
        let item_scope = if matches {
            let s2 = e.new_scope(ScopeKind::Block, None, Some(sc), it);
            pattern_vars(e, s2, it)?;
            s2
        } else {
            sc
        };
        let sc = item_scope;
        for &x in &ik[..ik.len() - 1] {
            if matches || kind == CaseKind::Inside {
                let b = if e.kind(x) == Kind::ValueRange || e.kind(x) == Kind::RangeList {
                    expr::bind(e, sc, x)?
                } else if matches {
                    let k2 = e.kids(x).to_vec();
                    match k2.first() {
                        Some(&f) if e.kind(x) == Kind::Pattern => {
                            let _ = f;
                            e.new_expr(ExprKind::Opaque, Types::UNKNOWN, x, vec![])
                        }
                        _ => expr::bind(e, sc, x)?,
                    }
                } else {
                    expr::bind(e, sc, x)?
                };
                xs.push(b);
                continue;
            }
            let b = expr::bind(e, sc, x)?;
            let t = e.ex(b).ty;
            if e.d.types.is_integral(t) {
                w = w.max(e.d.types.width(t));
                signed &= e.d.types.is_signed(t);
                four |= e.d.types.is_four_state(t);
            }
            xs.push(b);
        }
        item_exprs.push((xs, body, item_scope));
    }
    let integral = e.d.types.is_integral(e.ex(v).ty);
    let t = if integral { e.d.types.vector(w.max(1), signed, four) } else { e.ex(v).ty };
    let v = if integral { expr::ctx(e, v, t) } else { expr::selfdet(e, v) };
    let mut items = Vec::new();
    for (xs, body, isc) in item_exprs {
        let xs: Vec<ExprId> = xs
            .into_iter()
            .map(|x| if integral && e.d.types.is_integral(e.ex(x).ty) && kind != CaseKind::Inside { expr::ctx(e, x, t) } else { expr::selfdet(e, x) })
            .collect();
        let s = stmt(e, isc, body)?;
        items.push((xs, s));
    }
    let default = match default {
        Some(d) => Some(stmt(e, sc, d)?),
        None => None,
    };
    let _ = n;
    Ok(StmtKind::Case { check, kind, expr: v, items, default })
}

fn for_loop<'a>(e: &mut Elab<'a, '_>, sc: ScopeId, n: NodeId, kids: &[NodeId]) -> R<'a, StmtKind> {
    let init_node = kids.iter().copied().find(|&k| e.kind(k) == Kind::ForInit);
    let step_node = kids.iter().copied().find(|&k| e.kind(k) == Kind::ForStep);
    let body_node = *kids.last().unwrap();
    let cond_node = kids.iter().copied().find(|&k| !matches!(e.kind(k), Kind::ForInit | Kind::ForStep) && k != body_node);
    let declares = init_node.is_some_and(|i| e.kids(i).iter().any(|&k| e.kind(k) == Kind::VarDecl));
    let scope = if declares { Some(e.new_scope(ScopeKind::Block, None, Some(sc), n)) } else { None };
    let inner = scope.unwrap_or(sc);
    let mut init = Vec::new();
    if let Some(i) = init_node {
        for &k in e.kids(i).to_vec().iter() {
            match e.kind(k) {
                Kind::VarDecl => {
                    // `for (int i = 0; ...)`: each variable is automatic
                    // and set before the loop (12.7.1).
                    let tn = *e.kids(k).iter().find(|&&x| ty::is_type(e.kind(x))).unwrap();
                    let t = ty::data_type(e, inner, tn)?;
                    for d in e.kids(k).to_vec().iter().copied().filter(|&x| e.kind(x) == Kind::Declarator).collect::<Vec<_>>() {
                        let dk = e.kids(d).to_vec();
                        let s = e.sym_of(dk[0]);
                        let o = e.new_object(Object {
                            name: s,
                            kind: ObjKind::Variable,
                            ty: t,
                            scope: inner,
                            node: d,
                            lifetime: Lifetime::Automatic,
                            dir: None,
                            init: None,
                            value: None,
                            qualifiers: vec![],
                        });
                        e.declare(inner, s, Binding::Object(o), dk[0])?;
                        e.push_item(inner, Item::Object(o));
                        if let Some(&v) = dk.get(1) {
                            let rhs = expr::rvalue(e, inner, v, t)?;
                            let lhs = e.new_expr(ExprKind::Object(o), t, dk[0], vec![]);
                            init.push(e.new_stmt(StmtKind::Assign { kind: AssignKind::Blocking, lhs, rhs, timing: None }, d));
                        }
                    }
                }
                _ => init.push(stmt(e, inner, k)?),
            }
        }
    }
    let cond = match cond_node {
        Some(c) if e.kind(c) != Kind::Empty => Some(expr::expr(e, inner, c)?),
        _ => None,
    };
    let mut step = Vec::new();
    if let Some(s) = step_node {
        for &k in e.kids(s).to_vec().iter() {
            if e.kind(k) == Kind::Assign {
                step.push(stmt(e, inner, k)?);
            } else {
                let x = expr::expr(e, inner, k)?;
                step.push(e.new_stmt(StmtKind::Expr(x), k));
            }
        }
    }
    e.loop_depth += 1;
    let body = stmt(e, inner, body_node);
    e.loop_depth -= 1;
    Ok(StmtKind::For { scope, init, cond, step, body: body? })
}

fn foreach<'a>(e: &mut Elab<'a, '_>, sc: ScopeId, n: NodeId, kids: &[NodeId]) -> R<'a, StmtKind> {
    let array = expr::expr(e, sc, kids[0])?;
    let body_node = *kids.last().unwrap();
    let var_nodes = &kids[1..kids.len() - 1];
    let scope = e.new_scope(ScopeKind::Block, None, Some(sc), n);
    // The index types, one per dimension, outermost first.
    let mut dims: Vec<TypeId> = Vec::new();
    let mut t = e.ex(array).ty;
    let mut guard = 0;
    loop {
        guard += 1;
        if guard > 32 {
            break;
        }
        match e.d.types.get(t).clone() {
            Type::Unpacked { elem, dim } => {
                match dim {
                    UDim::Assoc(it) => dims.push(it),
                    UDim::Wildcard => {
                        return Err(e.err(kids[0], "`foreach` cannot iterate over a wildcard-indexed associative array (12.7.3)"));
                    }
                    _ => dims.push(Types::INT),
                }
                t = elem;
            }
            Type::Vector { dims: ref d, .. } if !d.is_empty() => {
                for _ in d {
                    dims.push(Types::INT);
                }
                break;
            }
            // An integer type's bits are not a dimension to iterate.
            Type::Vector { .. } => break,
            Type::PackedArray { elem, .. } => {
                dims.push(Types::INT);
                t = elem;
            }
            Type::String => {
                dims.push(Types::INT);
                break;
            }
            _ => break,
        }
    }
    if var_nodes.len() > dims.len() && e.ex(array).ty != Types::UNKNOWN {
        return Err(e.err(
            var_nodes[dims.len()],
            format!("`foreach` names {} loop variables, but the array has {} dimensions (12.7.3)", var_nodes.len(), dims.len()),
        ));
    }
    let mut vars = Vec::new();
    for (i, &v) in var_nodes.iter().enumerate() {
        if e.kind(v) != Kind::Name {
            vars.push(None);
            continue;
        }
        let s = e.sym_of(v);
        let vt = dims.get(i).copied().unwrap_or(Types::INT);
        let sym_foreach = e.sym("foreach");
        let o = e.new_object(Object {
            name: s,
            kind: ObjKind::Variable,
            ty: vt,
            scope,
            node: v,
            lifetime: Lifetime::Automatic,
            dir: None,
            init: None,
            value: None,
            qualifiers: vec![sym_foreach],
        });
        e.declare(scope, s, Binding::Object(o), v)?;
        vars.push(Some(o));
    }
    e.loop_depth += 1;
    let body = stmt(e, scope, body_node);
    e.loop_depth -= 1;
    Ok(StmtKind::Foreach { scope, array, vars, body: body? })
}

fn disable_target<'a>(e: &mut Elab<'a, '_>, sc: ScopeId, n: NodeId) -> R<'a, ExprId> {
    // A block or task name, maybe hierarchical.
    if e.kind(n) == Kind::Name {
        let s = e.sym_of(n);
        return match lookup::lookup(e, sc, s) {
            Some(Binding::Scope(_) | Binding::Subroutine(_) | Binding::Other(_)) => {
                Ok(e.new_expr(ExprKind::Hier { path: vec![s], object: None }, Types::UNKNOWN, n, vec![]))
            }
            _ => Err(e.err(n, format!("`{}` is not a block or task that can be disabled (9.6.2)", e.name(s)))),
        };
    }
    if let Some(x) = expr::hier(e, sc, n)? {
        return Ok(x);
    }
    expr::expr(e, sc, n)
}

/// A delay or event control.
pub(crate) fn timing<'a>(e: &mut Elab<'a, '_>, sc: ScopeId, n: NodeId) -> R<'a, Timing> {
    let kids = e.kids(n).to_vec();
    match e.kind(n) {
        Kind::Delay => {
            let mut xs = Vec::new();
            for &k in &kids {
                xs.push(expr::expr(e, sc, k)?);
            }
            Ok(Timing::Delay(xs))
        }
        Kind::Event => {
            if kids.is_empty() {
                return Ok(Timing::Implicit(vec![]));
            }
            let mut terms = Vec::new();
            event_terms(e, sc, kids[0], &mut terms)?;
            Ok(Timing::Event(terms))
        }
        Kind::CycleDelay => {
            let x = match kids.first() {
                Some(&k) => expr::expr(e, sc, k)?,
                None => e.new_expr(ExprKind::Opaque, Types::UNKNOWN, n, vec![]),
            };
            Ok(Timing::Cycle(x))
        }
        Kind::RepeatEvent => {
            let c = expr::expr(e, sc, kids[0])?;
            let mut terms = Vec::new();
            if let Some(&ev) = kids.get(1)
                && let Some(&inner) = e.kids(ev).first()
            {
                event_terms(e, sc, inner, &mut terms)?;
            }
            Ok(Timing::Repeat(c, terms))
        }
        _ => Ok(Timing::Delay(vec![])),
    }
}

fn event_terms<'a>(e: &mut Elab<'a, '_>, sc: ScopeId, n: NodeId, out: &mut Vec<EventTerm>) -> R<'a> {
    match e.kind(n) {
        Kind::EventOr => {
            for &k in e.kids(n).to_vec().iter() {
                event_terms(e, sc, k, out)?;
            }
        }
        Kind::EventTerm => {
            let k = e.kids(n).to_vec();
            let (edge, xi) = match e.kind(k[0]) {
                Kind::Keyword => (
                    match e.text(k[0]) {
                        "posedge" => Edge::Pos,
                        "negedge" => Edge::Neg,
                        _ => Edge::Both,
                    },
                    1,
                ),
                _ => (Edge::Any, 0),
            };
            let x = expr::expr(e, sc, k[xi])?;
            super::checks::event_operand(e, x, edge != Edge::Any, k[xi])?;
            let iff = match k.get(xi + 1) {
                Some(&i) if e.kind(i) == Kind::Iff => Some(expr::expr(e, sc, e.kids(i)[0])?),
                _ => None,
            };
            out.push(EventTerm { edge, expr: x, iff });
        }
        Kind::Paren => event_terms(e, sc, e.kids(n)[0], out)?,
        _ => {
            let x = expr::expr(e, sc, n)?;
            out.push(EventTerm { edge: Edge::Any, expr: x, iff: None });
        }
    }
    Ok(())
}
