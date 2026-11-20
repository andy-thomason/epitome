//! Merging identical functions and constants.
//!
//! Two functions are the same when their bodies are, up to the names and
//! numbering of their locals and blocks: the same instructions in the
//! same order, with the same flags, types, constants and callees, and the
//! same signature and attributes. Monomorphised generics that differ only
//! in a type of the same layout (`Vec<u64>` and `Vec<usize>`), and impls
//! that print integers of the same width, are the usual case: pointers
//! are opaque, so the IR of each is the same.
//!
//! Each function's body is written out in a canonical form (locals and
//! blocks numbered in layout order, names dropped, a call of the function
//! itself written as such), and functions with the same text are one
//! class. The first member of a class whose address must stay its own is
//! kept, or else the first member; each other member is then:
//!
//! - if its address is not significant (`unnamed_addr`, or
//!   `local_unnamed_addr` on an internal function), every use of it is
//!   made a use of the one kept, and it becomes an alias of it: an
//!   internal one is then unused and deleted, an external one stays, so
//!   other modules can still call it by its name;
//! - otherwise, its calls are made calls of the one kept (a call does not
//!   see the callee's address), and its body becomes a thunk that calls
//!   the one kept with its own arguments, so its address stays its own.
//!
//! Merging callees can make their callers the same, so the classes are
//! found again until nothing changes. Constant globals are merged the same
//! way, between rounds: private and internal `unnamed_addr` constants with
//! the same initialiser are one, with the greatest alignment of them
//! (strings, and vtables once the functions in them are merged).
//!
//! Only functions this module defines for good are merged: no `weak` or
//! `linkonce` definitions, nothing in a comdat, nothing whose blocks have
//! their addresses taken. Functions out of the module while it is
//! optimised (`epitome-opaque`, `"epitome-opt-level"="0"`) are
//! declarations here, so they are left alone.

use std::collections::HashMap;
use std::fmt::Write;

use epitome_ir::module::*;
use epitome_ir::types::Type;

use crate::attrs;

#[derive(Clone, Debug, Default)]
pub struct Stats {
    /// Functions made aliases of the same function.
    pub aliases: usize,
    /// Functions whose address is their own made thunks.
    pub thunks: usize,
    /// Address-significant functions left as they were, their calls made
    /// calls of the same function.
    pub calls_only: usize,
    /// Constant globals merged into the same constant.
    pub constants: usize,
    /// Rounds run.
    pub rounds: usize,
}

impl Stats {
    pub fn functions(&self) -> usize {
        self.aliases + self.thunks + self.calls_only
    }
}

/// The most rounds of merging.
const MAX_ROUNDS: usize = 16;

/// Merges `m`'s identical functions and constants. Returns what it did,
/// and a remark for each function merged into another.
pub fn run(
    m: &mut Module,
    chop: &epitome_ir::chop::Session,
    chop_index: &[usize],
) -> (Stats, Vec<(GlobalId, String)>) {
    let mut stats = Stats::default();
    let mut remarks = Vec::new();
    // Functions whose blocks have their addresses taken.
    let mut block_addresses = vec![false; m.globals.len()];
    for k in 0..m.consts.len() {
        if let ConstKind::BlockAddress { func, .. } = m.consts.get(ConstId(k as u32)).kind {
            block_addresses[func.index()] = true;
        }
    }
    // Each function's canonical text, kept between rounds until a
    // rewrite changes the function.
    let mut keys: Vec<Option<String>> = vec![None; m.globals.len()];
    // Globals already merged into another, which stay out of later rounds.
    let mut done = vec![false; m.globals.len()];
    for _ in 0..MAX_ROUNDS {
        stats.rounds += 1;
        let mut map = Rewrite::new(m.globals.len());
        merge_constants(m, &mut done, &mut map, &mut stats);
        let mut cx = FnCx { block_addresses: &block_addresses, keys: &mut keys, done: &mut done, chop, chop_index };
        merge_functions(m, &mut cx, &mut map, &mut stats, &mut remarks);
        if map.is_empty() {
            break;
        }
        for g in map.apply(m) {
            keys[g.index()] = None;
        }
    }
    (stats, remarks)
}

/// Uses of globals to make uses of others: of the address everywhere
/// (`full`), or only as the callee of a call (`calls`).
struct Rewrite {
    full: Vec<Option<GlobalId>>,
    calls: Vec<Option<GlobalId>>,
    any: bool,
}

impl Rewrite {
    fn new(n: usize) -> Self {
        Rewrite { full: vec![None; n], calls: vec![None; n], any: false }
    }

    fn is_empty(&self) -> bool {
        !self.any
    }

    /// Makes the rewrites, and returns the functions whose bodies changed.
    fn apply(&self, m: &mut Module) -> Vec<GlobalId> {
        let mut consts = ConstRewriter { full: &self.full, memo: HashMap::new() };
        let mut changed = Vec::new();
        for k in 0..m.globals.len() {
            let g = GlobalId(k as u32);
            // The constants this global's own definition names.
            let mut own: Vec<&mut ConstId> = Vec::new();
            let (globals, cs) = (&mut m.globals, &mut m.consts);
            match &mut globals[k].kind {
                GlobalKind::Variable(v) => own.extend(v.init.as_mut()),
                GlobalKind::Alias { aliasee, .. } => own.push(aliasee),
                GlobalKind::IFunc { resolver, .. } => own.push(resolver),
                GlobalKind::Function(f) => {
                    own.extend([f.prefix.as_mut(), f.prologue.as_mut(), f.personality.as_mut()].into_iter().flatten());
                }
                GlobalKind::Forward => {}
            }
            let mut touched = false;
            for c in own {
                let new = consts.rewrite(cs, *c);
                touched |= new != *c;
                *c = new;
            }
            let GlobalKind::Function(f) = &mut globals[k].kind else { continue };
            for i in 0..f.insts.len() {
                let op = &mut f.insts[i].op;
                if let Some(call) = op.call_mut()
                    && let Value::Const(c) = call.callee
                    && let ConstKind::Global(callee) = cs.get(c).kind
                    && let Some(to) = self.calls[callee.index()]
                {
                    let ty = cs.get(c).ty;
                    call.callee = Value::Const(cs.intern(Constant { ty, kind: ConstKind::Global(to) }));
                    touched = true;
                }
                op.for_each_value_mut(|v| {
                    if let Value::Const(c) = v {
                        let new = consts.rewrite(cs, *c);
                        if new != *c {
                            *c = new;
                            touched = true;
                        }
                    }
                });
            }
            if touched {
                changed.push(g);
            }
        }
        // Metadata that names a global's address.
        let cs = &mut m.consts;
        for node in &mut m.metadata {
            match &mut node.data {
                Metadata::Value(Value::Const(c)) => *c = consts.rewrite(cs, *c),
                Metadata::Node { fields, .. } => {
                    for f in fields {
                        if let MdFieldValue::Value(Value::Const(c)) = &mut f.value {
                            *c = consts.rewrite(cs, *c);
                        }
                    }
                }
                _ => {}
            }
        }
        changed
    }
}

/// Rewrites constants that mention globals in `full`, memoised.
struct ConstRewriter<'a> {
    full: &'a [Option<GlobalId>],
    memo: HashMap<ConstId, ConstId>,
}

impl ConstRewriter<'_> {
    fn rewrite(&mut self, cs: &mut Constants, c: ConstId) -> ConstId {
        if let Some(&r) = self.memo.get(&c) {
            return r;
        }
        // In progress: a malformed cycle stays as it is.
        self.memo.insert(c, c);
        let old = cs.get(c).clone();
        let kids = |list: &[ConstId], this: &mut Self, cs: &mut Constants| -> Vec<ConstId> {
            list.iter().map(|&k| this.rewrite(cs, k)).collect()
        };
        let kind = match &old.kind {
            ConstKind::Global(g) => ConstKind::Global(self.full[g.index()].unwrap_or(*g)),
            ConstKind::DsoLocalEquivalent(g) => ConstKind::DsoLocalEquivalent(self.full[g.index()].unwrap_or(*g)),
            ConstKind::NoCfi(g) => ConstKind::NoCfi(self.full[g.index()].unwrap_or(*g)),
            ConstKind::Array(v) => ConstKind::Array(kids(v, self, cs)),
            ConstKind::Struct(v) => ConstKind::Struct(kids(v, self, cs)),
            ConstKind::Vector(v) => ConstKind::Vector(kids(v, self, cs)),
            ConstKind::PtrAuth(v) => ConstKind::PtrAuth(kids(v, self, cs)),
            ConstKind::Splat(x) => ConstKind::Splat(self.rewrite(cs, *x)),
            ConstKind::Expr(e) => ConstKind::Expr(match e {
                ConstExpr::Cast { op, value } => ConstExpr::Cast { op: *op, value: self.rewrite(cs, *value) },
                ConstExpr::Gep { flags, inrange, source_ty, ptr, indices } => ConstExpr::Gep {
                    flags: *flags,
                    inrange: *inrange,
                    source_ty: *source_ty,
                    ptr: self.rewrite(cs, *ptr),
                    indices: kids(indices, self, cs),
                },
                ConstExpr::Binary { op, flags, lhs, rhs } => {
                    ConstExpr::Binary { op: *op, flags: *flags, lhs: self.rewrite(cs, *lhs), rhs: self.rewrite(cs, *rhs) }
                }
                ConstExpr::ExtractElement { vector, index } => {
                    ConstExpr::ExtractElement { vector: self.rewrite(cs, *vector), index: self.rewrite(cs, *index) }
                }
                ConstExpr::InsertElement { vector, element, index } => ConstExpr::InsertElement {
                    vector: self.rewrite(cs, *vector),
                    element: self.rewrite(cs, *element),
                    index: self.rewrite(cs, *index),
                },
                ConstExpr::ShuffleVector { a, b, mask } => ConstExpr::ShuffleVector {
                    a: self.rewrite(cs, *a),
                    b: self.rewrite(cs, *b),
                    mask: self.rewrite(cs, *mask),
                },
            }),
            _ => return c,
        };
        let new = if kind == old.kind { c } else { cs.intern(Constant { ty: old.ty, kind }) };
        self.memo.insert(c, new);
        new
    }
}

/// True if uses of `g`'s address may be uses of another's: its address is
/// not significant.
fn address_free(gl: &Global) -> bool {
    match gl.props.unnamed_addr {
        UnnamedAddr::Global => true,
        UnnamedAddr::Local => matches!(gl.props.linkage, Linkage::Internal | Linkage::Private),
        UnnamedAddr::None => false,
    }
}

/// True if `gl`'s definition is this module's for good, and plain enough
/// to merge with another.
fn mergeable_props(gl: &Global) -> bool {
    let p = &gl.props;
    matches!(p.linkage, Linkage::External | Linkage::Internal | Linkage::Private)
        && p.dll_storage == DllStorage::None
        && p.thread_local.is_none()
        && p.partition.is_none()
}

/// Every attribute in `list`, with `#N` groups expanded.
fn expand(m: &Module, list: &[Attr]) -> Vec<Attr> {
    let mut out = Vec::new();
    for a in list {
        match a {
            Attr::Group(n) => {
                m.attr_groups.iter().filter(|g| g.id == *n).for_each(|g| out.extend(expand(m, &g.attrs)))
            }
            a => out.push(a.clone()),
        }
    }
    out
}

/// Attachments that say what an instruction or global means, debug
/// locations left out.
fn meaningful(list: &[Attachment]) -> Vec<(&str, MdId)> {
    list.iter().filter(|a| a.kind != "dbg").map(|a| (a.kind.as_str(), a.node)).collect()
}

/// The canonical text of function `g`: equal for two functions exactly
/// when one can stand for the other.
fn function_key(m: &Module, g: GlobalId, f: &Function) -> String {
    let gl = m.global(g);
    let mut s = String::new();
    let params: Vec<Vec<Attr>> = f.params.iter().map(|p| expand(m, &p.attrs)).collect();
    let _ = write!(
        s,
        "{:?}",
        (
            f.ty,
            &f.cconv,
            expand(m, &f.ret_attrs),
            params,
            expand(m, &f.fn_attrs),
            &f.section,
            f.align,
            f.prefalign,
            &f.gc,
            (f.prefix, f.prologue, f.personality),
            (gl.props.addrspace, gl.props.visibility),
            meaningful(&gl.attachments),
        )
    );
    // Locals: parameters, then each block and its results in layout order.
    let mut local = vec![u32::MAX; f.locals.len()];
    let mut block = vec![u32::MAX; f.blocks.len()];
    for (k, p) in f.params.iter().enumerate() {
        local[p.local.index()] = k as u32;
    }
    let mut next = f.params.len() as u32;
    for (k, &b) in f.layout.iter().enumerate() {
        block[b.index()] = k as u32;
        local[f.blocks[b.index()].local.index()] = next;
        next += 1;
        for &i in &f.blocks[b.index()].insts {
            if let Some(r) = f.insts[i.index()].result {
                local[r.index()] = next;
                next += 1;
            }
        }
    }
    // A call of the function itself is the same in each.
    for &b in &f.layout {
        s.push_str("\n;");
        for &i in &f.blocks[b.index()].insts {
            let inst = &f.insts[i.index()];
            let mut op = inst.op.clone();
            op.for_each_value_mut(|v| match v {
                Value::Local(l) => *v = Value::Local(LocalId(local[l.index()])),
                Value::Const(c) if matches!(m.consts.get(*c).kind, ConstKind::Global(x) if x == g) => {
                    *v = Value::Const(ConstId(u32::MAX))
                }
                _ => {}
            });
            op.for_each_block_mut(|b| *b = BlockId(block[b.index()]));
            let ty = inst.result.map(|r| f.locals[r.index()].ty);
            let _ = write!(s, "\n{ty:?} {op:?} {:?}", meaningful(&inst.attachments));
        }
    }
    s
}

/// The number of instructions in `f`'s body.
fn size(f: &Function) -> usize {
    f.layout.iter().map(|b| f.blocks[b.index()].insts.len()).sum()
}

/// What merging functions keeps between rounds.
struct FnCx<'a> {
    block_addresses: &'a [bool],
    keys: &'a mut [Option<String>],
    done: &'a mut [bool],
    chop: &'a epitome_ir::chop::Session,
    chop_index: &'a [usize],
}

fn merge_functions(
    m: &mut Module,
    cx: &mut FnCx,
    map: &mut Rewrite,
    stats: &mut Stats,
    remarks: &mut Vec<(GlobalId, String)>,
) {
    let FnCx { block_addresses, keys, done, chop, chop_index } = cx;
    let mut classes: HashMap<&str, Vec<GlobalId>> = HashMap::new();
    let mut order: Vec<GlobalId> = Vec::new();
    for &g in &m.global_order {
        let gl = m.global(g);
        let GlobalKind::Function(f) = &gl.kind else { continue };
        if !f.has_body
            || done[g.index()]
            || f.comdat.is_some()
            || block_addresses[g.index()]
            || !mergeable_props(gl)
            || gl.name.as_deref().is_none_or(|n| n.starts_with("llvm."))
        {
            continue;
        }
        if keys[g.index()].is_none() {
            keys[g.index()] = Some(function_key(m, g, f));
        }
        order.push(g);
    }
    for &g in &order {
        classes.entry(keys[g.index()].as_deref().unwrap()).or_default().push(g);
    }
    let mut sets: Vec<Vec<GlobalId>> = classes.into_values().filter(|c| c.len() > 1).collect();
    // In module order, so the result does not depend on hashing.
    sets.sort_unstable_by_key(|c| c[0]);
    let position: HashMap<GlobalId, usize> = order.iter().enumerate().map(|(k, &g)| (g, k)).collect();
    for set in &mut sets {
        set.sort_unstable_by_key(|g| position[g]);
    }
    for set in sets {
        // Keep one whose address is its own, if any is.
        let keep = set.iter().copied().find(|&g| !address_free(m.global(g))).unwrap_or(set[0]);
        let keep_name = m.global(keep).name.clone().unwrap_or_default();
        for &g in &set {
            if g == keep {
                continue;
            }
            if let Some(&k) = chop_index.get(g.index())
                && k != usize::MAX
            {
                let _counted = chop.enter(k);
                let name = m.global(g).name.clone().unwrap_or_default();
                if !epitome_ir::chop::allow(|| format!("merge: @{name} into @{keep_name}")) {
                    continue;
                }
            }
            map.any = true;
            map.calls[g.index()] = Some(keep);
            if address_free(m.global(g)) {
                map.full[g.index()] = Some(keep);
                make_alias(m, g, keep);
                stats.aliases += 1;
                remarks.push((g, format!("merge: the same as @{keep_name}, now an alias of it")));
            } else if make_thunk(m, g, keep) {
                stats.thunks += 1;
                remarks.push((g, format!("merge: the same as @{keep_name}, now a call of it")));
            } else {
                stats.calls_only += 1;
                remarks.push((g, format!("merge: the same as @{keep_name}, its calls now call that")));
            }
            keys[g.index()] = None;
            done[g.index()] = true;
        }
    }
}

/// Makes `g` an alias of function `keep`.
fn make_alias(m: &mut Module, g: GlobalId, keep: GlobalId) {
    let GlobalKind::Function(f) = &m.global(g).kind else { unreachable!() };
    let value_ty = f.ty;
    let ptr = m.types.ptr(m.global(keep).props.addrspace.unwrap_or(0));
    let aliasee = m.consts.intern(Constant { ty: ptr, kind: ConstKind::Global(keep) });
    let gl = &mut m.globals[g.index()];
    gl.kind = GlobalKind::Alias { value_ty, aliasee };
    gl.attachments.clear();
}

/// Makes `g`'s body a tail call of `keep` with its own arguments, if that
/// is smaller and can pass them as they came. Returns whether it did.
fn make_thunk(m: &mut Module, g: GlobalId, keep: GlobalId) -> bool {
    let GlobalKind::Function(f) = &m.global(g).kind else { unreachable!() };
    let Type::Function { ret, varargs, .. } = m.types.get(f.ty).clone() else { return false };
    let by_memory = f.params.iter().any(|p| {
        ["byval", "inalloca", "preallocated"].iter().any(|n| attrs::has_named(m, &p.attrs, n))
    });
    if varargs || by_memory || size(f) <= 2 {
        return false;
    }
    let ptr = m.types.ptr(m.global(keep).props.addrspace.unwrap_or(0));
    let callee = Value::Const(m.consts.intern(Constant { ty: ptr, kind: ConstKind::Global(keep) }));
    let label = m.types.label();
    let void = matches!(m.types.get(ret), Type::Void);
    let GlobalKind::Function(f) = &mut m.globals[g.index()].kind else { unreachable!() };
    let mut body = Function { locals: f.locals[..f.params.len()].to_vec(), ..Default::default() };
    let entry = body.add_entry_block(None, label);
    let call = Call {
        tail: TailKind::Tail,
        cconv: f.cconv.clone(),
        ret_attrs: f.ret_attrs.clone(),
        args: f.params.iter().map(|p| CallArg { value: Value::Local(p.local), attrs: p.attrs.clone() }).collect(),
        ..Call::new(f.ty, callee, &[])
    };
    let result = (!void).then_some((None, ret));
    let i = body.push_inst(entry, Op::Call(Box::new(call)), result);
    let value = (!void).then(|| body.result(i));
    body.push_inst(entry, Op::Ret(value), None);
    f.locals = body.locals;
    f.blocks = body.blocks;
    f.layout = body.layout;
    f.insts = body.insts;
    true
}

fn merge_constants(m: &mut Module, done: &mut [bool], map: &mut Rewrite, stats: &mut Stats) {
    let mut classes: HashMap<String, Vec<GlobalId>> = HashMap::new();
    for &g in &m.global_order {
        let gl = m.global(g);
        let GlobalKind::Variable(v) = &gl.kind else { continue };
        if !v.constant
            || done[g.index()]
            || v.externally_initialized
            || v.init.is_none()
            || v.comdat.is_some()
            || !matches!(gl.props.linkage, Linkage::Internal | Linkage::Private)
            || !address_free(gl)
            || !mergeable_props(gl)
        {
            continue;
        }
        let key = format!(
            "{:?}",
            (
                v.ty,
                v.init,
                &v.section,
                &v.code_model,
                &v.sanitizer,
                &v.attrs,
                v.align.is_some(),
                gl.props.addrspace,
                meaningful(&gl.attachments)
            )
        );
        classes.entry(key).or_default().push(g);
    }
    let mut sets: Vec<Vec<GlobalId>> = classes.into_values().filter(|c| c.len() > 1).collect();
    sets.sort_unstable_by_key(|c| c[0]);
    for set in sets {
        let keep = set[0];
        let align = set
            .iter()
            .filter_map(|g| match &m.global(*g).kind {
                GlobalKind::Variable(v) => v.align,
                _ => None,
            })
            .max();
        if let GlobalKind::Variable(v) = &mut m.globals[keep.index()].kind {
            v.align = align;
        }
        for &g in &set[1..] {
            map.any = true;
            map.full[g.index()] = Some(keep);
            done[g.index()] = true;
            stats.constants += 1;
        }
    }
}
