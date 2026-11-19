//! Interprocedural optimisation after the bottom-up pipeline: what working
//! up the call graph one SCC at a time cannot see.
//!
//! Bottom-up, a function is simplified once, before its callers, and a
//! call to another member of its own cycle tells it nothing. Panics make
//! such cycles big: every function that can panic reaches the panic hook,
//! which formats, which can panic, so in a whole Rust program hundreds of
//! functions share one cycle. And bottom-up order never carries what a
//! caller knows into its callee. This pass works on the whole module, in
//! rounds:
//!
//! 1. Constant returns. A call to a function whose every `ret` returns the
//!    same constant has that constant as its result (std's
//!    `get_backtrace_style` when std is built without backtraces).
//! 2. Leaf functions inlined. A function that calls nothing and has at
//!    most `LEAF_SIZE` instructions besides its allocas and its `ret`s
//!    costs no more inline than called, so every call to it is inlined,
//!    within a cycle too.
//! 3. Argument promotion. A pointer parameter of an internal function that
//!    the function only loads from, at constant offsets, becomes the loaded
//!    values, loaded by the callers, when every caller passes memory of its
//!    own frame (an alloca) whose address goes nowhere else: only the
//!    caller and this parameter see it, so nothing can write it while the
//!    callee runs. The callers' loads then usually fold with their stores.
//! 4. Constant arguments. A parameter of an internal function for which
//!    every caller passes the same constant is that constant.
//!
//! The functions a round changed are cleaned up (SROA, mem2reg, the
//! combiner, CFG simplification) before the next round, which then sees,
//! say, the constant a promoted argument now loads. Steps 1 and 4 are the
//! part of LLVM's interprocedural sparse conditional constant propagation
//! that carries constants through calls, and step 3 is its argument
//! promotion, as those passes are documented and as `opt` behaves.
//!
//! The rewrites are sequential, in module order; the clean-up runs in
//! parallel, with no body published until every one is done, so the result
//! does not depend on the number of threads. Each rewrite is an elective
//! change (autochop) of the function it changes: the caller for steps 1
//! and 2, the callee for steps 3 and 4.

use std::collections::{HashMap as StdHashMap, HashSet};

use epitome_ir::edit::Names;
use epitome_ir::module::*;
use epitome_ir::types::{Type, TypeId};

use crate::attrs;
use crate::callgraph::CallGraph;
use crate::combine::tree::Matcher;
use crate::ctx::{Intrinsics, Shared, parallel_phase};
use crate::fx::FxHashMap as HashMap;
use crate::par;

/// The most rounds.
const ROUNDS: usize = 8;
/// The most instructions, allocas and `ret`s aside, of a leaf function
/// that is always inlined.
const LEAF_SIZE: usize = 4;
/// The most loads one pointer parameter is promoted into.
const MAX_PROMOTED: usize = 4;

#[derive(Clone, Debug, Default)]
pub struct Stats {
    /// Call results replaced by the constant the callee always returns.
    pub returns: usize,
    /// Calls to leaf functions inlined.
    pub inlined: usize,
    /// Pointer parameters promoted, and the parameters they became.
    pub promoted: usize,
    pub promoted_into: usize,
    /// Parameters replaced by the constant every caller passes.
    pub arguments: usize,
    pub rounds: usize,
}

impl Stats {
    pub fn total(&self) -> usize {
        self.returns + self.inlined + self.promoted + self.arguments
    }
}

/// What the pass needs from the pipeline.
pub struct Context<'a> {
    pub threads: usize,
    pub intrinsics: Intrinsics,
    pub matcher: &'a Matcher,
    pub memory: bool,
    pub simplifycfg: bool,
    /// Whether step 2 may inline.
    pub inline: bool,
    pub chop: &'a epitome_ir::chop::Session,
    /// Each function's autochop index, `usize::MAX` for none.
    pub chop_index: &'a [usize],
}

impl Context<'_> {
    fn counted(&self, g: GlobalId) -> Option<epitome_ir::chop::Guard<'_>> {
        let k = self.chop_index.get(g.index()).copied().unwrap_or(usize::MAX);
        (k != usize::MAX).then(|| self.chop.enter(k))
    }
}

/// Runs the rounds; returns what they did and the remarks of each function.
pub fn run(m: &mut Module, cx: &Context) -> (Stats, Vec<(GlobalId, Vec<String>)>) {
    let mut stats = Stats::default();
    let mut remarks: Vec<(GlobalId, Vec<String>)> = Vec::new();
    for _ in 0..ROUNDS {
        let cg = CallGraph::build(m, cx.threads);
        let (changed, r) = parallel_phase(m, cx.intrinsics, |sh| {
            let mut round = Round::new(sh, &cg, cx);
            round.constant_returns(&mut stats);
            if cx.inline {
                round.inline_leaves(&mut stats);
            }
            round.find_sites();
            round.promote_arguments(&mut stats);
            round.constant_arguments(&mut stats);
            round.finish()
        });
        for (g, list) in r {
            match remarks.iter_mut().find(|(h, _)| *h == g) {
                Some((_, l)) => l.extend(list),
                None => remarks.push((g, list)),
            }
        }
        stats.rounds += 1;
        if !changed {
            break;
        }
    }
    (stats, remarks)
}

/// One pointer parameter's loads: a new parameter for each offset and type.
struct Slot {
    offset: i64,
    ty: TypeId,
    align: Option<Align>,
    loads: Vec<InstId>,
}

struct Round<'a, 's, 'm> {
    sh: &'a Shared<'m>,
    cg: &'a CallGraph,
    cx: &'a Context<'s>,
    /// Functions with bodies, in module order.
    order: Vec<GlobalId>,
    /// Their bodies, taken for the round, by `GlobalId`.
    bodies: Vec<Option<Box<Function>>>,
    changed: Vec<bool>,
    /// The plain direct calls to each function: (caller, call).
    sites: Vec<Vec<(GlobalId, InstId)>>,
    /// Called some other way: an `invoke`, a `musttail` call, operand
    /// bundles, or a call whose type is not the function's.
    odd: Vec<bool>,
}

impl<'a, 's, 'm> Round<'a, 's, 'm> {
    fn new(sh: &'a Shared<'m>, cg: &'a CallGraph, cx: &'a Context<'s>) -> Self {
        let m = sh.module;
        let n = m.globals.len();
        let order: Vec<GlobalId> =
            m.global_order.iter().copied().filter(|&g| sh.is_function(g) && sh.published(g).is_none()).collect();
        let mut bodies: Vec<Option<Box<Function>>> = (0..n).map(|_| None).collect();
        for &g in &order {
            bodies[g.index()] = Some(sh.take(g));
        }
        Round { sh, cg, cx, order, bodies, changed: vec![false; n], sites: vec![], odd: vec![false; n] }
    }

    fn body(&self, g: GlobalId) -> &Function {
        self.bodies[g.index()].as_deref().unwrap()
    }

    fn body_mut(&mut self, g: GlobalId) -> &mut Function {
        self.bodies[g.index()].as_deref_mut().unwrap()
    }

    /// A function's type, from this round's body or its declaration.
    fn fn_ty(&self, g: GlobalId) -> Option<TypeId> {
        self.bodies[g.index()].as_deref().or_else(|| self.sh.published(g)).map(|f| f.ty)
    }

    fn has(&self, list: &[Attr], word: &str) -> bool {
        attrs::has(self.sh.module, list, word)
    }

    /// Publishes what did not change, so the clean-up sees it, then cleans
    /// up the rest in parallel and publishes it after. True if anything
    /// changed.
    fn finish(mut self) -> bool {
        let (sh, cx, cg) = (self.sh, self.cx, self.cg);
        let mut work = Vec::new();
        for &g in &self.order {
            let f = self.bodies[g.index()].take().unwrap();
            if self.changed[g.index()] {
                work.push((g, std::sync::Mutex::new(Some(f))));
            } else {
                sh.publish(g, f);
            }
        }
        let done = par::map(&work, cx.threads, |_, (g, f)| {
            let mut f = f.lock().unwrap().take().unwrap();
            let _counted = cx.counted(*g);
            let block_addresses = cg.node(*g).is_some_and(|k| cg.has_blockaddress[k]);
            clean_up(sh, cx, &mut f, block_addresses);
            f
        });
        for ((g, _), f) in work.iter().zip(done) {
            sh.publish(*g, f);
        }
        !work.is_empty()
    }

    /// Step 1: calls to functions that always return the same constant.
    fn constant_returns(&mut self, stats: &mut Stats) {
        let sh = self.sh;
        let mut returns: HashMap<GlobalId, Value> = HashMap::default();
        for &g in &self.order {
            if !attrs::definition_is_final(sh.module.global(g).props.linkage) {
                continue;
            }
            let f = self.body(g);
            let mut same: Option<Value> = None;
            let mut all = true;
            for &b in &f.layout {
                let Some(t) = f.terminator(b) else { continue };
                match f.insts[t.index()].op {
                    Op::Ret(Some(v @ Value::Const(_))) if same.is_none_or(|s| s == v) => same = Some(v),
                    Op::Ret(_) => all = false,
                    _ => {}
                }
            }
            if all && let Some(v) = same {
                returns.insert(g, v);
            }
        }
        if returns.is_empty() {
            return;
        }
        for k in 0..self.order.len() {
            let g = self.order[k];
            let f = self.body(g);
            // The results still in use of calls with a constant result.
            let mut map: StdHashMap<LocalId, Value> = StdHashMap::new();
            for i in f.insts_in_order() {
                let inst = &f.insts[i.index()];
                let (Op::Call(call) | Op::Invoke { call, .. }) = &inst.op else { continue };
                let (Some(r), Some(callee)) = (inst.result, sh.direct_callee(call)) else { continue };
                let Some(&v) = returns.get(&callee) else { continue };
                if call.tail != TailKind::MustTail && Some(call.fn_ty) == self.fn_ty(callee) {
                    map.insert(r, v);
                }
            }
            if map.is_empty() {
                continue;
            }
            let mut used: HashSet<LocalId> = HashSet::new();
            for i in f.insts_in_order() {
                f.insts[i.index()].op.for_each_value(|v| {
                    if let Value::Local(l) = v
                        && map.contains_key(l)
                    {
                        used.insert(*l);
                    }
                });
            }
            let mut keys: Vec<LocalId> = used.into_iter().collect();
            keys.sort();
            let _counted = self.cx.counted(g);
            let keep: StdHashMap<LocalId, Value> = keys
                .into_iter()
                .filter(|&l| {
                    epitome_ir::chop::allow(|| {
                        format!("ipo: the result of {} is the constant its callee returns", local_name(f, l))
                    })
                })
                .map(|l| (l, map[&l]))
                .collect();
            if keep.is_empty() {
                continue;
            }
            stats.returns += keep.len();
            sh.remark(g, format!("ipo: {} call results are the constant their callee always returns", keep.len()));
            let f = self.body_mut(g);
            f.replace_many(&keep);
            for (&l, &v) in &keep {
                crate::inline::replace_in_debug_records(sh, f, l, v);
            }
            self.changed[g.index()] = true;
        }
    }

    /// Whether `g` is a leaf function small enough to inline everywhere.
    fn is_leaf(&self, g: GlobalId) -> bool {
        let sh = self.sh;
        let f = self.body(g);
        if !attrs::definition_is_final(sh.module.global(g).props.linkage)
            || self.has(&f.fn_attrs, "noinline")
            || f.personality.is_some()
            || self.cg.node(g).is_some_and(|k| self.cg.has_blockaddress[k])
            || matches!(sh.ty(f.ty), Type::Function { varargs: true, .. })
        {
            return false;
        }
        let entry = f.entry();
        let mut size = 0;
        for &b in &f.layout {
            for &i in &f.blocks[b.index()].insts {
                match &f.insts[i.index()].op {
                    Op::Alloca { count, .. } => {
                        // Static allocas only: they move to the caller's entry.
                        let fixed = count.is_none_or(|c| matches!(c, Value::Const(_)));
                        if b != entry || !fixed {
                            return false;
                        }
                    }
                    Op::Ret(_) => {}
                    Op::Call(_)
                    | Op::Invoke { .. }
                    | Op::CallBr { .. }
                    | Op::IndirectBr { .. }
                    | Op::Resume(_)
                    | Op::VaArg { .. } => return false,
                    _ => size += 1,
                }
            }
        }
        size <= LEAF_SIZE
    }

    /// Step 2: calls to small leaf functions, inlined.
    fn inline_leaves(&mut self, stats: &mut Stats) {
        let sh = self.sh;
        let n = self.bodies.len();
        let mut leaf = vec![false; n];
        for &g in &self.order {
            leaf[g.index()] = self.is_leaf(g);
        }
        let same = |a: &Function, b: &Function, key: &str| {
            attrs::string(sh.module, &a.fn_attrs, key) == attrs::string(sh.module, &b.fn_attrs, key)
        };
        for k in 0..self.order.len() {
            let g = self.order[k];
            let mut f = self.bodies[g.index()].take().unwrap();
            if self.has(&f.fn_attrs, "optnone") {
                self.bodies[g.index()] = Some(f);
                continue;
            }
            let sites: Vec<(InstId, GlobalId)> = f
                .insts_in_order()
                .filter_map(|i| {
                    let Op::Call(call) = &f.insts[i.index()].op else { return None };
                    let c = sh.direct_callee(call)?;
                    let callee = self.bodies[c.index()].as_deref().filter(|_| leaf[c.index()])?;
                    let fits = call.tail != TailKind::MustTail
                        && call.bundles.is_empty()
                        && call.fn_ty == callee.ty
                        && same(&f, callee, "target-cpu")
                        && same(&f, callee, "target-features");
                    fits.then_some((i, c))
                })
                .collect();
            if !sites.is_empty() {
                let _counted = self.cx.counted(g);
                let mut names = Names::new(&f);
                let mut memo = crate::inline::DebugMemo::default();
                let mut count = 0;
                for (site, c) in sites {
                    let name = sh.name(c);
                    if !epitome_ir::chop::allow(|| format!("ipo: inline @{name} into @{}", sh.name(g))) {
                        continue;
                    }
                    let block = f.inst_blocks()[site.index()].expect("call site not in a block");
                    let callee = self.bodies[c.index()].as_deref().unwrap();
                    crate::inline::inline_call(sh, &mut f, &mut names, &mut memo, site, block, callee, &name, false);
                    sh.remark(g, format!("ipo: inlined leaf @{name}"));
                    count += 1;
                }
                if count > 0 {
                    stats.inlined += count;
                    self.changed[g.index()] = true;
                }
            }
            self.bodies[g.index()] = Some(f);
        }
    }

    /// The call sites of every function, after step 2.
    fn find_sites(&mut self) {
        let sh = self.sh;
        let n = self.bodies.len();
        let mut sites = vec![Vec::new(); n];
        let mut odd = vec![false; n];
        for &g in &self.order {
            let f = self.body(g);
            for i in f.insts_in_order() {
                let Some(call) = f.insts[i.index()].op.call() else { continue };
                let Some(c) = sh.direct_callee(call) else { continue };
                let plain = matches!(f.insts[i.index()].op, Op::Call(_))
                    && call.tail != TailKind::MustTail
                    && call.bundles.is_empty()
                    && Some(call.fn_ty) == self.fn_ty(c);
                if plain {
                    sites[c.index()].push((g, i));
                } else {
                    odd[c.index()] = true;
                }
            }
        }
        self.sites = sites;
        self.odd = odd;
    }

    /// Whether every call to `c` is a plain call seen in `sites`, so its
    /// parameters may change.
    fn all_calls_known(&self, c: GlobalId) -> bool {
        let sh = self.sh;
        let Some(f) = self.bodies[c.index()].as_deref() else { return false };
        let internal = matches!(sh.module.global(c).props.linkage, Linkage::Internal | Linkage::Private);
        let address_taken = self.cg.node(c).is_none_or(|k| self.cg.address_taken[k]);
        internal
            && !address_taken
            && !self.odd[c.index()]
            && !self.sites[c.index()].is_empty()
            && !self.has(&f.fn_attrs, "optnone")
            && !self.has(&f.fn_attrs, "naked")
            && !matches!(sh.ty(f.ty), Type::Function { varargs: true, .. })
    }

    /// Step 3: pointer parameters only loaded from, promoted.
    fn promote_arguments(&mut self, stats: &mut Stats) {
        let sh = self.sh;
        for k in 0..self.order.len() {
            let c = self.order[k];
            if !self.all_calls_known(c) {
                continue;
            }
            let f = self.body(c);
            // Changing the parameters would break a `musttail` call out.
            let musttail = f.insts_in_order().any(|i| f.insts[i.index()].op.call().is_some_and(|c| c.tail == TailKind::MustTail));
            if musttail {
                continue;
            }
            let users = users(f);
            let mut promoted: Vec<(usize, Vec<Slot>, Vec<InstId>)> = Vec::new();
            for (j, p) in f.params.iter().enumerate() {
                if !matches!(sh.ty(f.locals[p.local.index()].ty), Type::Ptr(_)) || special_param(sh.module, &p.attrs) {
                    continue;
                }
                let Some((slots, geps)) = self.loads_of(f, &users, p.local) else { continue };
                if self.sites[c.index()].iter().all(|&(g, i)| self.caller_allows(g, i, c, j, &slots)) {
                    promoted.push((j, slots, geps));
                }
            }
            if promoted.is_empty() {
                continue;
            }
            let _counted = self.cx.counted(c);
            promoted.retain(|(j, _, _)| {
                epitome_ir::chop::allow(|| format!("ipo: promote parameter {j} of @{}", sh.name(c)))
            });
            if promoted.is_empty() {
                continue;
            }
            let new_ty = self.promote_in_callee(c, &promoted);
            for s in 0..self.sites[c.index()].len() {
                let (g, i) = self.sites[c.index()][s];
                self.promote_at_site(g, i, new_ty, &promoted);
                self.changed[g.index()] = true;
            }
            let into: usize = promoted.iter().map(|(_, s, _)| s.len()).sum();
            sh.remark(c, format!("ipo: {} pointer parameters promoted into {into} loaded values", promoted.len()));
            stats.promoted += promoted.len();
            stats.promoted_into += into;
            self.changed[c.index()] = true;
        }
    }

    /// The loads that are the only uses of pointer parameter `p`, directly
    /// or through a constant `getelementptr`, by offset and type; and those
    /// `getelementptr`s. `None` if `p` has any other use.
    fn loads_of(
        &self,
        f: &Function,
        users: &HashMap<LocalId, Vec<InstId>>,
        p: LocalId,
    ) -> Option<(Vec<Slot>, Vec<InstId>)> {
        let sh = self.sh;
        let mut slots: Vec<Slot> = Vec::new();
        let mut geps = Vec::new();
        let add = |slots: &mut Vec<Slot>, offset: i64, ptr: Value, load: InstId| -> Option<()> {
            let inst = &f.insts[load.index()];
            let Op::Load { volatile: false, atomic: None, ptr: q, align } = inst.op else { return None };
            if q != ptr {
                return None;
            }
            let ty = f.locals[inst.result?.index()].ty;
            if !first_class(&sh.ty(ty)) || offset < 0 {
                return None;
            }
            match slots.iter_mut().find(|s| s.offset == offset && s.ty == ty) {
                Some(s) => {
                    s.align = s.align.zip(align).map(|(a, b)| if a.bytes() <= b.bytes() { a } else { b });
                    s.loads.push(load);
                }
                None => slots.push(Slot { offset, ty, align, loads: vec![load] }),
            }
            Some(())
        };
        let me = Value::Local(p);
        for &u in users.get(&p)? {
            let inst = &f.insts[u.index()];
            match &inst.op {
                Op::Load { .. } => add(&mut slots, 0, me, u)?,
                Op::Gep { source_ty, ptr, indices, .. } if *ptr == me && !indices.contains(&me) => {
                    let offset = crate::combine::constload::gep_offset(sh, *source_ty, indices)?;
                    let r = inst.result?;
                    for &v in users.get(&r).map(|v| v.as_slice()).unwrap_or(&[]) {
                        add(&mut slots, offset, Value::Local(r), v)?;
                    }
                    geps.push(u);
                }
                _ => return None,
            }
        }
        (!slots.is_empty() && slots.len() <= MAX_PROMOTED).then_some((slots, geps))
    }

    /// Whether the call `i` in `g` passes, as parameter `j` of `c`, memory
    /// of `g`'s frame that holds every slot and whose address goes nowhere
    /// but to that parameter.
    fn caller_allows(&self, g: GlobalId, i: InstId, c: GlobalId, j: usize, slots: &[Slot]) -> bool {
        let sh = self.sh;
        let f = self.body(g);
        let Some(call) = f.insts[i.index()].op.call() else { return false };
        let Some(arg) = call.args.get(j).map(|a| a.value) else { return false };
        // The alloca under the argument, and the argument's offset into it.
        let mut v = arg;
        let mut base = 0i64;
        let alloca = loop {
            let Value::Local(l) = v else { return false };
            let LocalDef::Inst(d) = f.locals[l.index()].def else { return false };
            match &f.insts[d.index()].op {
                Op::Alloca { .. } => break l,
                Op::Gep { source_ty, ptr, indices, .. } => {
                    let Some(o) = crate::combine::constload::gep_offset(sh, *source_ty, indices) else { return false };
                    base += o;
                    v = *ptr;
                }
                _ => return false,
            }
        };
        let LocalDef::Inst(d) = f.locals[alloca.index()].def else { return false };
        let Op::Alloca { ty, count, .. } = &f.insts[d.index()].op else { return false };
        let count = match count {
            None => 1,
            Some(Value::Const(k)) => match crate::combine::eval::int_const(sh.consts(), sh.types(), *k) {
                Some(k) => k.value,
                None => return false,
            },
            Some(_) => return false,
        };
        let Some(size) = sh.layout.alloc_size(sh.types(), *ty).and_then(|s| s.checked_mul(count)) else { return false };
        for s in slots {
            let Some(len) = sh.layout.store_size(sh.types(), s.ty) else { return false };
            let start = base + s.offset;
            if start < 0 || start as u64 + len > size {
                return false;
            }
        }
        !escapes(sh, f, &users(f), alloca, c, j)
    }

    /// Rewrites `c` for the promoted parameters; returns its new type.
    fn promote_in_callee(&mut self, c: GlobalId, promoted: &[(usize, Vec<Slot>, Vec<InstId>)]) -> TypeId {
        let sh = self.sh;
        let f = self.body_mut(c);
        let ptr_ty = sh.intern_type(Type::Ptr(0));
        let mut map: StdHashMap<LocalId, Value> = StdHashMap::new();
        let mut gone: HashSet<InstId> = HashSet::new();
        let mut params = Vec::new();
        let mut tys = Vec::new();
        for (j, p) in f.params.clone().into_iter().enumerate() {
            let Some((_, slots, geps)) = promoted.iter().find(|(k, _, _)| *k == j) else {
                tys.push(f.locals[p.local.index()].ty);
                params.push(p);
                continue;
            };
            for s in slots {
                let l = f.add_local(None, s.ty, LocalDef::Param(0));
                params.push(Param { local: l, attrs: vec![] });
                tys.push(s.ty);
                for &load in &s.loads {
                    map.insert(f.insts[load.index()].result.unwrap(), Value::Local(l));
                    gone.insert(load);
                }
            }
            gone.extend(geps.iter().copied());
            crate::inline::replace_in_debug_records(sh, f, p.local, sh.poison(ptr_ty));
        }
        for b in f.layout.clone() {
            f.blocks[b.index()].insts.retain(|i| !gone.contains(i));
        }
        f.replace_many(&map);
        for (&l, &v) in &map {
            crate::inline::replace_in_debug_records(sh, f, l, v);
        }
        for (k, p) in params.iter().enumerate() {
            f.locals[p.local.index()].def = LocalDef::Param(k as u32);
        }
        f.params = params;
        let Type::Function { ret, varargs, .. } = sh.ty(f.ty) else { unreachable!("a function's type") };
        f.ty = sh.intern_type(Type::Function { ret, params: tys, varargs });
        f.ty
    }

    /// Loads the promoted slots before call `i` in `g` and passes them.
    fn promote_at_site(&mut self, g: GlobalId, i: InstId, new_ty: TypeId, promoted: &[(usize, Vec<Slot>, Vec<InstId>)]) {
        let sh = self.sh;
        let i8t = sh.intern_type(Type::Int(8));
        let ptr_ty = sh.intern_type(Type::Ptr(0));
        let f = self.body_mut(g);
        let block = f.inst_blocks()[i.index()].expect("call site not in a block");
        let old = f.insts[i.index()].op.call().unwrap().args.clone();
        let mut before = Vec::new();
        let mut args = Vec::new();
        for (j, a) in old.into_iter().enumerate() {
            let Some((_, slots, _)) = promoted.iter().find(|(k, _, _)| *k == j) else {
                args.push(a);
                continue;
            };
            for s in slots {
                let ptr = if s.offset == 0 {
                    a.value
                } else {
                    let gep = f.add_inst(
                        Op::Gep {
                            flags: GepFlags(GepFlags::INBOUNDS),
                            source_ty: i8t,
                            ptr: a.value,
                            indices: vec![sh.int(64, s.offset)].into_boxed_slice(),
                        },
                        Some((None, ptr_ty)),
                    );
                    before.push(gep);
                    f.result(gep)
                };
                let load =
                    f.add_inst(Op::Load { volatile: false, atomic: None, ptr, align: s.align }, Some((None, s.ty)));
                before.push(load);
                args.push(CallArg { value: f.result(load), attrs: vec![] });
            }
        }
        let call = f.insts[i.index()].op.call_mut().unwrap();
        call.args = args;
        call.fn_ty = new_ty;
        let insts = &mut f.blocks[block.index()].insts;
        let at = insts.iter().position(|&x| x == i).unwrap();
        insts.splice(at..at, before);
    }

    /// Step 4: parameters every caller passes the same constant for.
    fn constant_arguments(&mut self, stats: &mut Stats) {
        let sh = self.sh;
        for k in 0..self.order.len() {
            let c = self.order[k];
            if !self.all_calls_known(c) {
                continue;
            }
            let f = self.body(c);
            let used = users(f);
            let mut found: Vec<(LocalId, Value)> = Vec::new();
            for (j, p) in f.params.iter().enumerate() {
                if special_param(sh.module, &p.attrs) || !used.contains_key(&p.local) {
                    continue;
                }
                let mut same: Option<Value> = None;
                let all = self.sites[c.index()].iter().all(|&(g, i)| {
                    let call = self.body(g).insts[i.index()].op.call().unwrap();
                    match call.args.get(j).map(|a| a.value) {
                        Some(v @ Value::Const(_)) if same.is_none_or(|s| s == v) => {
                            same = Some(v);
                            true
                        }
                        _ => false,
                    }
                });
                if all && let Some(v) = same {
                    found.push((p.local, v));
                }
            }
            if found.is_empty() {
                continue;
            }
            let _counted = self.cx.counted(c);
            found.retain(|&(l, _)| {
                epitome_ir::chop::allow(|| {
                    format!("ipo: {} of @{} is the constant every caller passes", local_name(f, l), sh.name(c))
                })
            });
            if found.is_empty() {
                continue;
            }
            stats.arguments += found.len();
            sh.remark(c, format!("ipo: {} parameters are the constant every caller passes", found.len()));
            let f = self.body_mut(c);
            for (l, v) in found {
                f.replace_uses(Value::Local(l), v);
                crate::inline::replace_in_debug_records(sh, f, l, v);
            }
            self.changed[c.index()] = true;
        }
    }
}

/// SROA and mem2reg, the combiner, and CFG simplification with the
/// combiner on what it touched.
fn clean_up(sh: &Shared, cx: &Context, f: &mut Function, block_addresses: bool) {
    crate::sroa::run(sh, f);
    crate::mem2reg::run(sh, f);
    crate::combine::run_with(sh, f, cx.matcher, None, cx.memory);
    if !cx.simplifycfg {
        return;
    }
    for _ in 0..3 {
        let out = crate::simplifycfg::run(sh, f, block_addresses);
        if out.stats.total() == 0 || out.touched.is_empty() {
            break;
        }
        crate::combine::run_with(sh, f, cx.matcher, Some(&out.touched), cx.memory);
    }
}

/// The instructions in the body that use each local.
fn users(f: &Function) -> HashMap<LocalId, Vec<InstId>> {
    let mut users: HashMap<LocalId, Vec<InstId>> = HashMap::default();
    for i in f.insts_in_order() {
        f.insts[i.index()].op.for_each_value(|v| {
            if let Value::Local(l) = v {
                users.entry(*l).or_default().push(i);
            }
        });
    }
    users
}

/// Whether the address of `alloca`, or one derived from it, goes anywhere
/// but to loads and stores through it, lifetime markers, memory
/// intrinsics, and parameter `j` of calls to `c`.
fn escapes(sh: &Shared, f: &Function, users: &HashMap<LocalId, Vec<InstId>>, alloca: LocalId, c: GlobalId, j: usize) -> bool {
    let mut work = vec![alloca];
    let mut seen: HashSet<LocalId> = HashSet::new();
    while let Some(p) = work.pop() {
        if !seen.insert(p) {
            continue;
        }
        let me = Value::Local(p);
        for &u in users.get(&p).map(|v| v.as_slice()).unwrap_or(&[]) {
            let inst = &f.insts[u.index()];
            match &inst.op {
                Op::Load { ptr, .. } if *ptr == me => {}
                Op::Store { ptr, value, .. } if *ptr == me && *value != me => {}
                Op::Gep { ptr, indices, .. } if *ptr == me && !indices.contains(&me) => work.push(inst.result.unwrap()),
                Op::Call(call) if call.callee != me && call.bundles.iter().all(|b| !b.inputs.contains(&me)) => {
                    let callee = sh.direct_callee(call);
                    let name = callee.and_then(|g| sh.module.global(g).name.as_deref()).unwrap_or("");
                    let at = |k: usize| call.args.iter().enumerate().all(|(n, a)| a.value != me || n == k);
                    let fine = if callee == Some(c) {
                        at(j)
                    } else if name.starts_with("llvm.lifetime.") {
                        true
                    } else if name.starts_with("llvm.memcpy") || name.starts_with("llvm.memmove") || name.starts_with("llvm.memset") {
                        call.args.iter().skip(2).all(|a| a.value != me)
                    } else {
                        false
                    };
                    if !fine {
                        return true;
                    }
                }
                _ => return true,
            }
        }
    }
    false
}

/// Parameter attributes that change what the argument is: a copy made for
/// the call, or memory with a meaning of its own.
fn special_param(m: &Module, list: &[Attr]) -> bool {
    ["byval", "byref", "inalloca", "preallocated", "sret", "swifterror", "swiftself", "swiftasync", "nest"]
        .iter()
        .any(|a| attrs::has_named(m, list, a))
}

/// A type a load can produce and a parameter can take as one value.
fn first_class(t: &Type) -> bool {
    matches!(
        t,
        Type::Int(_) | Type::Half | Type::BFloat | Type::Float | Type::Double | Type::Fp128 | Type::Ptr(_) | Type::Vector { .. }
    )
}

fn local_name(f: &Function, l: LocalId) -> String {
    match &f.locals[l.index()].name {
        Some(n) => format!("%{n}"),
        None => format!("%{}", l.0),
    }
}
