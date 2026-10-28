//! Turning recursion into loops with an explicit stack.
//!
//! A self-recursive function becomes one activation loop. The old entry
//! block becomes a loop header whose phis hold the parameters, and each
//! recursive call becomes a jump back to the header:
//!
//! - A **tail call** (`%r = call @f(...)` followed by `ret %r`) needs
//!   nothing saved: it jumps to the header with the new arguments. This is
//!   tail-recursion elimination.
//! - Any **other call** first pushes a frame holding the values that are
//!   live after the call and the number of the call site, then jumps to the
//!   header. Every `ret` goes to a dispatch block: if the frame stack is
//!   empty the function really returns; otherwise it pops the top frame,
//!   reloads the saved values and continues after the call that pushed it,
//!   with the returned value standing in for the call's result.
//!
//! Frames live on the machine stack, in one of two ways:
//!
//! - **A frame pool**, for functions with no allocas of their own. The
//!   entry block allocates a chunk of frames once; a push stores into the
//!   next free frame and a pop reads it back. Only when a chunk is full does
//!   the header allocate another (`llvm.stacksave` and an `alloca`), linked
//!   to the last, and a pop that empties it frees it with
//!   `llvm.stackrestore`.
//! - **Linked frames**, for functions with allocas. Each activation's
//!   allocas must be freed when it returns, so every push is
//!   `llvm.stacksave` followed by an `alloca` of the frame, and every pop
//!   calls `llvm.stackrestore`, freeing the frame and everything the
//!   finished activation allocated above it. Tail calls push frames too,
//!   because jumping to the header would allocate again without freeing.
//!
//! Either way the stack needs no fixed size, no heap and no C library, and
//! uses memory in proportion to the recursion depth. A frame holds only the
//! live values: no return address, saved registers or spill slots.
//!
//! Allocas of the function stay in the old entry block, which is inside the
//! loop, so each activation gets its own.
//!
//! Splitting blocks at the calls leaves values with more than one
//! definition (the original, and a reload in each pop that restores it), so
//! the pass finishes by rebuilding SSA form with `ssa::SsaUpdater`.
//!
//! Mutual recursion is first reduced to self-recursion by inlining: see
//! `collapse_scc`.

use crate::fx::{FxHashMap as HashMap, FxHashSet as HashSet};

use epitome_ir::edit::Names;
use epitome_ir::module::*;
use epitome_ir::types::{Type, TypeId};

use crate::attrs;
use crate::callgraph::CallGraph;
use crate::cfg::DomTree;
use crate::ctx::Shared;
use crate::inline::{self, plain_call};
use crate::ssa::{self, SsaUpdater};

/// Which recursive calls to remove.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RecursionMode {
    Off,
    /// Tail calls and accumulator calls, which become plain loops with no
    /// stack. Other recursive calls stay calls. The default.
    Tail,
    /// All direct recursive calls, using an explicit stack where a loop is
    /// not enough. For targets without a call stack, or where the stack
    /// measurably pays.
    Stack,
}

/// Functions bigger than this are left alone.
const MAX_SIZE: usize = 5000;
/// A chunk of the frame pool holds at most this many frames...
const CHUNK_FRAMES: u64 = 64;
/// ...and is about this many bytes at most.
const CHUNK_BYTES: u64 = 2048;
/// Call sites that would save more values than this are left alone.
const MAX_FRAME_VALUES: usize = 64;

/// What `eliminate` did.
#[derive(Debug, Default)]
pub struct Outcome {
    /// Tail calls turned into jumps.
    pub tail_calls: usize,
    /// Accumulator calls turned into jumps, and the operation they fold.
    pub accumulator_calls: usize,
    pub accumulator_op: Option<BinOp>,
    /// Calls turned into frame pushes, with the number of values each saves.
    pub frame_calls: Vec<usize>,
    /// Recursive calls left as calls.
    pub kept: usize,
}

/// Removes the direct self-recursive calls of function `g`.
///
/// `block_addresses` says a `blockaddress` names one of `f`'s blocks; the
/// blocks are then left unmerged.
pub fn eliminate(
    sh: &Shared,
    g: GlobalId,
    f: &mut Function,
    mode: RecursionMode,
    block_addresses: bool,
) -> Result<Outcome, String> {
    let sites: Vec<InstId> = f
        .insts_in_order()
        .filter(|&i| f.insts[i.index()].op.call().is_some_and(|c| sh.direct_callee(c) == Some(g)))
        .collect();
    if sites.is_empty() || mode == RecursionMode::Off {
        return Ok(Outcome::default());
    }
    if !attrs::definition_is_final(sh.module.global(g).props.linkage) {
        return Err("its definition may be replaced at link time, so the recursive calls may not reach it".into());
    }
    check_legal(sh, g, f, &sites)?;

    let blocks = f.inst_blocks();
    let has_alloca = f.insts_in_order().any(|i| matches!(f.insts[i.index()].op, Op::Alloca { .. }));
    let is_tail = |i: InstId| returns_result(f, blocks[i.index()].unwrap(), i);
    let mut jumps = Vec::new();
    let mut accs: Vec<AccSite> = Vec::new();
    let mut acc_op: Option<BinOp> = None;
    let mut others = Vec::new();
    for &s in &sites {
        if f.insts[s.index()].op.call().unwrap().tail == TailKind::MustTail && has_alloca {
            return Err("musttail recursion in a function with allocas".into());
        }
        if has_alloca {
            // Jumping back would allocate again without freeing.
            others.push(s);
        } else if is_tail(s) {
            jumps.push(s);
        } else if let Some((op, u, x)) = accumulator_use(sh, f, blocks[s.index()].unwrap(), s)
            && acc_op.is_none_or(|o| o == op)
        {
            acc_op = Some(op);
            accs.push((s, u, x));
        } else {
            others.push(s);
        }
    }
    // With a stack, every call that is not a tail call pushes a frame; an
    // accumulator would have to be saved in each frame too, so it is not
    // used.
    let mut frames = Vec::new();
    let mut kept = 0;
    if mode == RecursionMode::Stack && !others.is_empty() {
        frames = others;
        frames.extend(accs.drain(..).map(|(s, _, _)| s));
        frames.sort_by_key(|s| sites.iter().position(|x| x == s));
        acc_op = None;
    } else {
        kept = others.len();
    }
    if jumps.is_empty() && frames.is_empty() && accs.is_empty() {
        return Ok(Outcome { kept, ..Default::default() });
    }
    if f.layout.iter().map(|b| f.blocks[b.index()].insts.len()).sum::<usize>() > MAX_SIZE {
        return Err("function too large".into());
    }

    // A parameter that every recursive call passes through unchanged has
    // the same value in every activation: it needs no phi and no saving.
    let invariant: Vec<bool> = (0..f.params.len())
        .map(|k| {
            let p = Value::Local(f.params[k].local);
            jumps
                .iter()
                .chain(&frames)
                .chain(accs.iter().map(|(s, _, _)| s))
                .all(|&s| f.insts[s.index()].op.call().unwrap().args[k].value == p)
        })
        .collect();
    let mut live = live_after(f, &frames);
    for set in &mut live {
        set.retain(|&l| !matches!(f.locals[l.index()].def, LocalDef::Param(k) if invariant[k as usize]));
    }
    for (k, set) in live.iter().enumerate() {
        if set.len() > MAX_FRAME_VALUES {
            return Err(format!("call site {k} would save {} values", set.len()));
        }
        for &l in set {
            let ty = sh.ty(f.locals[l.index()].ty);
            if matches!(ty, Type::Token | Type::Label | Type::Metadata | Type::TargetExt { .. } | Type::X86Amx) {
                return Err("a value of a type that cannot be stored is live across a recursive call".into());
            }
        }
    }

    // Without allocas of its own, the function can take frames from a pool
    // instead of allocating each one on the machine stack.
    let pool = !has_alloca;
    let mut t = Transform::new(sh, f);
    let acc = acc_op.map(|op| (op, accs.as_slice()));
    let outcome = t.run(f, &jumps, acc, &frames, &live, &invariant, pool, !block_addresses);
    Ok(Outcome { kept, ..outcome })
}

fn check_legal(sh: &Shared, g: GlobalId, f: &Function, sites: &[InstId]) -> Result<(), String> {
    let m = sh.module;
    let Type::Function { varargs, .. } = sh.ty(f.ty) else { unreachable!() };
    if varargs {
        return Err("variadic".into());
    }
    if let Some(dl) = &m.data_layout {
        let dl = String::from_utf8_lossy(dl);
        if dl.split('-').any(|part| part.starts_with('A') && part != "A0") {
            return Err("allocas are not in address space 0".into());
        }
    }
    for p in &f.params {
        for a in ["byval", "inalloca", "preallocated", "swiftself", "swifterror", "swiftasync"] {
            if attrs::has_named(m, &p.attrs, a) {
                return Err(format!("{a} parameter"));
            }
        }
    }
    for &s in sites {
        let inst = &f.insts[s.index()];
        let Op::Call(call) = &inst.op else { return Err("recursive call through invoke or callbr".into()) };
        if call.fn_ty != f.ty || call.cconv != f.cconv || !call.bundles.is_empty() {
            return Err("recursive call does not match the function's own type".into());
        }
    }
    for i in f.insts_in_order() {
        let op = &f.insts[i.index()].op;
        match op {
            Op::LandingPad { .. }
            | Op::Invoke { .. }
            | Op::Resume(_)
            | Op::CatchSwitch { .. }
            | Op::CatchPad { .. }
            | Op::CleanupPad { .. }
            | Op::CatchRet { .. }
            | Op::CleanupRet { .. } => return Err("exception handling".into()),
            Op::IndirectBr { .. } | Op::CallBr { .. } => return Err("indirect branches".into()),
            Op::VaArg { .. } => return Err("va_arg".into()),
            Op::Alloca { inalloca: true, .. } => return Err("inalloca".into()),
            _ => {}
        }
        if let Some(call) = op.call() {
            // A `musttail` call must stay right before its `ret`, and the
            // transform reroutes every `ret`.
            if call.tail == TailKind::MustTail && sh.direct_callee(call) != Some(g) {
                return Err("musttail call to another function".into());
            }
            if attrs::has(m, &call.fn_attrs, "returns_twice") {
                return Err("calls a returns_twice function".into());
            }
            if let Some(t) = sh.direct_callee(call) {
                if sh.published(t).is_some_and(|t| attrs::has(m, &t.fn_attrs, "returns_twice")) {
                    return Err("calls a returns_twice function".into());
                }
                let name = m.global(t).name.as_deref().unwrap_or("");
                // `llvm.stacksave` and `llvm.stackrestore` are fine: frames
                // are pushed and popped in LIFO order with the function's own
                // saves and restores, including those of an earlier run of
                // this pass.
                let frame_sensitive = [
                    "llvm.frameaddress",
                    "llvm.returnaddress",
                    "llvm.localescape",
                    "llvm.sponentry",
                    "llvm.va_start",
                    "llvm.va_copy",
                ];
                if frame_sensitive.iter().any(|p| name.starts_with(p)) {
                    return Err(format!("calls {name}"));
                }
            }
        }
    }
    Ok(())
}

/// An accumulator call: the call, the instruction that folds its result,
/// and the other operand.
type AccSite = (InstId, InstId, Value);

/// If recursive call `site` is an accumulator call, the operation, the
/// instruction that applies it, and the other operand: the call's only use
/// is `op(r, x)` with `op` associative and commutative, in the same block,
/// with only side-effect-free arithmetic between them, and that result is
/// returned. Then `return x op f(...)` can be computed as a loop that
/// folds each `x` into an accumulator, the way `n * fact(n - 1)` becomes a
/// running product.
fn accumulator_use(sh: &Shared, f: &Function, block: BlockId, site: InstId) -> Option<(BinOp, InstId, Value)> {
    let r = f.insts[site.index()].result?;
    if !matches!(sh.ty(f.locals[r.index()].ty), Type::Int(_)) {
        return None;
    }
    let me = Value::Local(r);
    let mut users = Vec::new();
    for i in f.insts_in_order() {
        let mut n = 0;
        f.insts[i.index()].op.for_each_value(|v| n += (*v == me) as usize);
        users.extend(std::iter::repeat_n(i, n));
    }
    let [u] = users[..] else { return None };
    let Op::Binary { op, lhs, rhs, .. } = f.insts[u.index()].op else { return None };
    if !matches!(op, BinOp::Add | BinOp::Mul | BinOp::And | BinOp::Or | BinOp::Xor) {
        return None;
    }
    let x = if lhs == me { rhs } else { lhs };
    if x == me {
        return None;
    }
    // Only pure arithmetic between the call and its use: those run before
    // the next activation instead of after it.
    let insts = &f.blocks[block.index()].insts;
    let from = insts.iter().position(|&i| i == site)?;
    let to = insts.iter().position(|&i| i == u)?;
    if to <= from {
        return None;
    }
    let pure = insts[from + 1..to].iter().all(|&i| {
        matches!(
            f.insts[i.index()].op,
            Op::Binary { .. } | Op::ICmp { .. } | Op::Select { .. } | Op::Cast { .. } | Op::Freeze(_) | Op::Gep { .. }
        )
    });
    (pure && returns_result(f, block, u)).then_some((op, u, x))
}

/// True if what follows call `site` in `block` only returns its result:
/// `ret` straight after it, or branches to blocks that do nothing but pass
/// the result through phis to a `ret`.
fn returns_result(f: &Function, block: BlockId, site: InstId) -> bool {
    let result = f.insts[site.index()].result.map(Value::Local);
    let insts = &f.blocks[block.index()].insts;
    let k = insts.iter().position(|&x| x == site).unwrap();
    // The value that carries the result, and the block it is carried from.
    let (mut carried, mut from) = (result, block);
    let mut next = insts.get(k + 1).copied();
    for _ in 0..8 {
        let Some(i) = next else { return false };
        match &f.insts[i.index()].op {
            Op::Ret(None) => return true,
            Op::Ret(Some(v)) => return carried == Some(*v),
            Op::Br(target) => {
                let body = &f.blocks[target.index()].insts;
                let mut phis = 0;
                let mut new_carried = None;
                for &p in body {
                    let Op::Phi { incoming, .. } = &f.insts[p.index()].op else { break };
                    phis += 1;
                    let mine = incoming.iter().find(|(_, b)| *b == from).map(|(v, _)| *v);
                    if carried.is_some() && mine == carried {
                        new_carried = f.insts[p.index()].result.map(Value::Local);
                    }
                }
                // Anything other than phis and the terminator is real work.
                if body.len() != phis + 1 {
                    return false;
                }
                // The result may also reach the `ret` directly, without a phi.
                if new_carried.is_some() {
                    carried = new_carried;
                }
                from = *target;
                next = body.last().copied();
            }
            _ => return false,
        }
    }
    false
}

/// For each call site, the values live just after it, not counting the
/// call's own result. Parameters and instruction results only.
fn live_after(f: &Function, sites: &[InstId]) -> Vec<Vec<LocalId>> {
    if sites.is_empty() {
        return Vec::new();
    }
    let n = f.locals.len();
    let words = n.div_ceil(64);
    let is_value = |l: LocalId| matches!(f.locals[l.index()].def, LocalDef::Param(_) | LocalDef::Inst(_));
    let set = |s: &mut [u64], l: LocalId| s[l.index() / 64] |= 1 << (l.index() % 64);
    let clear = |s: &mut [u64], l: LocalId| s[l.index() / 64] &= !(1 << (l.index() % 64));

    let nb = f.blocks.len();
    let mut uses = vec![vec![0u64; words]; nb];
    let mut defs = vec![vec![0u64; words]; nb];
    let mut phi_out = vec![vec![0u64; words]; nb];
    for &b in &f.layout {
        for &i in &f.blocks[b.index()].insts {
            let inst = &f.insts[i.index()];
            if let Op::Phi { incoming, .. } = &inst.op {
                for (v, p) in incoming {
                    if let Value::Local(l) = v
                        && is_value(*l)
                    {
                        set(&mut phi_out[p.index()], *l);
                    }
                }
            } else {
                let (u, d) = (&mut uses[b.index()], &defs[b.index()]);
                inst.op.for_each_value(|v| {
                    if let Value::Local(l) = v
                        && is_value(*l)
                        && d[l.index() / 64] & (1 << (l.index() % 64)) == 0
                    {
                        set(u, *l);
                    }
                });
            }
            if let Some(r) = inst.result {
                set(&mut defs[b.index()], r);
            }
        }
    }
    let mut live_in = vec![vec![0u64; words]; nb];
    let mut live_out = vec![vec![0u64; words]; nb];
    let mut changed = true;
    while changed {
        changed = false;
        for &b in f.layout.iter().rev() {
            let mut out = phi_out[b.index()].clone();
            for s in f.successors(b) {
                for (o, x) in out.iter_mut().zip(&live_in[s.index()]) {
                    *o |= x;
                }
            }
            let inn: Vec<u64> = (0..words).map(|w| uses[b.index()][w] | (out[w] & !defs[b.index()][w])).collect();
            if out != live_out[b.index()] || inn != live_in[b.index()] {
                live_out[b.index()] = out;
                live_in[b.index()] = inn;
                changed = true;
            }
        }
    }

    let blocks = f.inst_blocks();
    sites
        .iter()
        .map(|&site| {
            let b = blocks[site.index()].unwrap();
            let mut live = live_out[b.index()].clone();
            for &i in f.blocks[b.index()].insts.iter().rev() {
                let inst = &f.insts[i.index()];
                if let Some(r) = inst.result {
                    clear(&mut live, r);
                }
                if i == site {
                    break;
                }
                if !inst.op.is_phi() {
                    inst.op.for_each_value(|v| {
                        if let Value::Local(l) = v
                            && is_value(*l)
                        {
                            set(&mut live, *l);
                        }
                    });
                }
            }
            (0..n as u32).map(LocalId).filter(|l| live[l.index() / 64] & (1 << (l.index() % 64)) != 0).collect()
        })
        .collect()
}

/// Types and constants the transform uses, interned once.
struct Transform<'s, 'm> {
    sh: &'s Shared<'m>,
    names: Names,
    ptr: TypeId,
    i32: TypeId,
    void: TypeId,
}

impl<'s, 'm> Transform<'s, 'm> {
    fn new(sh: &'s Shared<'m>, f: &Function) -> Self {
        Transform {
            sh,
            names: Names::new(f),
            ptr: sh.intern_type(Type::Ptr(0)),
            i32: sh.intern_type(Type::Int(32)),
            void: sh.intern_type(Type::Void),
        }
    }

    fn name(&mut self, base: &str) -> Option<String> {
        Some(self.names.fresh(base))
    }

    fn gep(&mut self, f: &mut Function, b: BlockId, ty: TypeId, base: Value, field: u32, name: &str) -> Value {
        if field == 0 {
            return base;
        }
        let indices = vec![self.sh.int(32, 0), self.sh.int(32, field as i64)];
        let op = Op::Gep { flags: GepFlags(GepFlags::INBOUNDS), source_ty: ty, ptr: base, indices: indices.into() };
        let name = self.name(name);
        let i = f.push_inst(b, op, Some((name, self.ptr)));
        f.result(i)
    }

    fn load(&mut self, f: &mut Function, b: BlockId, ty: TypeId, ptr: Value, name: Option<String>) -> Value {
        let i = f.push_inst(b, Op::Load { volatile: false, atomic: None, ptr, align: None }, Some((name, ty)));
        f.result(i)
    }

    fn store(&mut self, f: &mut Function, b: BlockId, value: Value, ptr: Value) {
        f.push_inst(b, Op::Store { volatile: false, atomic: None, value, ptr, align: None }, None);
    }

    fn call_intrinsic(&mut self, f: &mut Function, b: BlockId, save: bool, arg: Option<Value>) -> Option<Value> {
        let sh = self.sh;
        if save {
            let fn_ty = sh.intern_type(Type::Function { ret: self.ptr, params: vec![], varargs: false });
            let callee = sh.global_ref(sh.intrinsics.stacksave.expect("llvm.stacksave not declared"));
            let name = self.name("sp");
            let i = f.push_inst(b, Op::Call(Box::new(plain_call(fn_ty, callee, vec![]))), Some((name, self.ptr)));
            Some(f.result(i))
        } else {
            let fn_ty = sh.intern_type(Type::Function { ret: self.void, params: vec![self.ptr], varargs: false });
            let callee = sh.global_ref(sh.intrinsics.stackrestore.expect("llvm.stackrestore not declared"));
            f.push_inst(b, Op::Call(Box::new(plain_call(fn_ty, callee, arg.into_iter().collect()))), None);
            None
        }
    }

    /// An address inside `base`, which points to a `ty`.
    fn gep_path(
        &mut self,
        f: &mut Function,
        b: BlockId,
        ty: TypeId,
        base: Value,
        path: Vec<Value>,
        name: &str,
    ) -> Value {
        let op = Op::Gep { flags: GepFlags(GepFlags::INBOUNDS), source_ty: ty, ptr: base, indices: path.into() };
        let name = self.name(name);
        let i = f.push_inst(b, op, Some((name, self.ptr)));
        f.result(i)
    }

    fn phi(
        &mut self,
        f: &mut Function,
        b: BlockId,
        at: usize,
        ty: TypeId,
        name: &str,
        incoming: Vec<(Value, BlockId)>,
    ) -> InstId {
        let name = self.name(name);
        let phi = f.add_inst(Op::Phi { fmf: FastMath::default(), incoming }, Some((name, ty)));
        f.blocks[b.index()].insts.insert(at, phi);
        phi
    }

    fn icmp_eq(&mut self, f: &mut Function, b: BlockId, lhs: Value, rhs: Value, name: &str) -> Value {
        let i1 = self.sh.intern_type(Type::Int(1));
        let name = self.name(name);
        let i = f.push_inst(b, Op::ICmp { pred: ICmpPred::Eq, samesign: false, lhs, rhs }, Some((name, i1)));
        f.result(i)
    }

    #[allow(clippy::too_many_arguments)]
    fn run(
        &mut self,
        f: &mut Function,
        jumps: &[InstId],
        acc: Option<(BinOp, &[AccSite])>,
        frames: &[InstId],
        live: &[Vec<LocalId>],
        invariant: &[bool],
        pool: bool,
        merge: bool,
    ) -> Outcome {
        let sh = self.sh;
        let Type::Function { ret: ret_ty, .. } = sh.ty(f.ty) else { unreachable!() };
        let returns_value = !matches!(sh.ty(ret_ty), Type::Void);
        let use_stack = !frames.is_empty();
        let pool = pool && use_stack;
        let many_sites = frames.len() > 1;

        // Blocks. A new entry block jumps to the header; the header starts
        // each activation. With linked frames the header is the old entry
        // block itself. With a pool, the header first makes sure there is
        // room for a frame, then goes on to the old entry block.
        let body = f.entry();
        if f.block_name(body).is_none() {
            let name = self.name("recurse");
            let l = f.blocks[body.index()].local;
            f.locals[l.index()].name = name;
        }
        let new_entry = f.add_block(self.name("entry.rec"));
        f.layout.insert(0, new_entry);
        let header = if pool {
            let h = f.add_block(self.name("rec.header"));
            f.layout.insert(1, h);
            h
        } else {
            body
        };

        // Parameters become header phis. The phi takes the parameter's name.
        let mut param_phis: Vec<Option<InstId>> = Vec::new();
        for (k, &invariant) in invariant.iter().enumerate() {
            if invariant {
                param_phis.push(None);
                continue;
            }
            let p = f.params[k].local;
            let name = f.locals[p.index()].name.take();
            if let Some(n) = &name {
                f.locals[p.index()].name = self.name(&format!("{n}.arg"));
            }
            let ty = f.locals[p.index()].ty;
            let phi = f.add_inst(Op::Phi { fmf: FastMath::default(), incoming: vec![] }, Some((name, ty)));
            let pv = f.result(phi);
            f.replace_uses(Value::Local(p), pv);
            inline::replace_in_debug_records(sh, f, p, pv);
            if let Op::Phi { incoming, .. } = &mut f.insts[phi.index()].op {
                incoming.push((Value::Local(p), new_entry));
            }
            param_phis.push(Some(phi));
        }
        let header_phis: Vec<InstId> = param_phis.iter().flatten().copied().collect();
        for (k, &phi) in header_phis.iter().enumerate() {
            f.blocks[header.index()].insts.insert(k, phi);
        }
        let param_value = |f: &Function, l: LocalId| -> LocalId {
            match f.locals[l.index()].def {
                LocalDef::Param(k) => match param_phis[k as usize] {
                    Some(phi) => f.insts[phi.index()].result.unwrap(),
                    None => l,
                },
                _ => l,
            }
        };

        // What each call site saves.
        let saved: Vec<Vec<LocalId>> =
            live.iter().map(|set| set.iter().map(|&l| param_value(f, l)).collect()).collect();

        // The state of the frame stack is held in SSA variables, each
        // represented by a placeholder local until SSA is rebuilt:
        // - linked frames: the top frame, or null;
        // - a pool: the current chunk, and the index of its next free frame.
        // Each has a phi in the header, fed by the new entry block, the
        // pushes and the tail calls.
        let mut state: Vec<LocalId> = Vec::new();
        let mut state_phis: Vec<InstId> = Vec::new();
        let mut state_defs: Vec<(usize, BlockId, Value)> = Vec::new();
        // Pool layout: one frame type for every call site, and the chunk type.
        let mut pool_frame_ty = self.i32;
        let mut chunk_ty = self.i32;
        let mut site_fields: Vec<Vec<u32>> = Vec::new();
        let mut capacity = 0i64;
        if pool {
            // Fields are shared between call sites by type: the frame has,
            // for each type, as many fields as the most any site saves.
            let mut fields: Vec<TypeId> = if many_sites { vec![self.i32] } else { vec![] };
            for set in &saved {
                let mut used: HashMap<TypeId, usize> = HashMap::default();
                let mut mine = Vec::new();
                for &l in set {
                    let ty = f.locals[l.index()].ty;
                    let n = used.entry(ty).or_default();
                    let start = if many_sites { 1 } else { 0 };
                    let existing = (start..fields.len()).filter(|&i| fields[i] == ty).nth(*n);
                    let at = existing.unwrap_or_else(|| {
                        fields.push(ty);
                        fields.len() - 1
                    });
                    *n += 1;
                    mine.push(at as u32);
                }
                site_fields.push(mine);
            }
            if fields.is_empty() {
                fields.push(self.i32);
            }
            let bytes: u64 = fields.iter().map(|&t| size_estimate(sh, t)).sum::<u64>().max(1);
            capacity = (CHUNK_BYTES / bytes).clamp(4, CHUNK_FRAMES) as i64;
            pool_frame_ty = sh.intern_type(Type::Struct { packed: false, fields });
            let frames_ty = sh.intern_type(Type::Array { len: capacity as u64, elem: pool_frame_ty });
            chunk_ty = sh.intern_type(Type::Struct { packed: false, fields: vec![self.ptr, self.ptr, frames_ty] });

            // The first chunk is static, in the entry block.
            let name = self.name("frames");
            let first = f.push_inst(
                new_entry,
                Op::Alloca { inalloca: false, ty: chunk_ty, count: None, align: None, addrspace: None },
                Some((name, self.ptr)),
            );
            let first = f.result(first);
            self.store(f, new_entry, sh.null_ptr(), first);

            let chunk = f.add_local(Some("chunk".into()), self.ptr, LocalDef::Forward);
            let index = f.add_local(Some("index".into()), self.i32, LocalDef::Forward);
            state = vec![chunk, index];
            let at = header_phis.len();
            let chunk_h = self.phi(f, header, at, self.ptr, "chunk.top", vec![(first, new_entry)]);
            let index_h = self.phi(f, header, at + 1, self.i32, "index.top", vec![(sh.int(32, 0), new_entry)]);
            state_phis = vec![chunk_h, index_h];
            let (chunk_h, index_h) = (f.result(chunk_h), f.result(index_h));

            // Room for one more frame, or a new chunk.
            let grow = f.add_block(self.name("rec.grow"));
            let pos = f.layout.iter().position(|&b| b == header).unwrap();
            f.layout.insert(pos + 1, grow);
            let full = self.icmp_eq(f, header, index_h, sh.int(32, capacity), "chunk.full");
            f.push_inst(header, Op::CondBr { cond: full, then_dest: grow, else_dest: body }, None);
            let sp = self.call_intrinsic(f, grow, true, None).unwrap();
            let name = self.name("chunk.new");
            let new_chunk = f.push_inst(
                grow,
                Op::Alloca { inalloca: false, ty: chunk_ty, count: None, align: None, addrspace: None },
                Some((name, self.ptr)),
            );
            let new_chunk = f.result(new_chunk);
            self.store(f, grow, chunk_h, new_chunk);
            let at = self.gep(f, grow, chunk_ty, new_chunk, 1, "chunk.sp");
            self.store(f, grow, sp, at);
            f.push_inst(grow, Op::Br(body), None);
            let chunk_b = self.phi(f, body, 0, self.ptr, "chunk", vec![(chunk_h, header), (new_chunk, grow)]);
            let index_b = self.phi(f, body, 1, self.i32, "index", vec![(index_h, header), (sh.int(32, 0), grow)]);
            state_phis.extend([chunk_b, index_b]);
            state_defs.push((0, body, f.result(chunk_b)));
            state_defs.push((1, body, f.result(index_b)));
        } else if use_stack {
            let top = f.add_local(Some("frame".into()), self.ptr, LocalDef::Forward);
            state = vec![top];
            let phi = self.phi(f, header, header_phis.len(), self.ptr, "frame.top", vec![(sh.null_ptr(), new_entry)]);
            state_phis = vec![phi];
            state_defs.push((0, header, f.result(phi)));
        }
        f.push_inst(new_entry, Op::Br(header), None);
        let n_header_state = if pool { 2 } else { state.len() };
        let add_header_incoming = |f: &mut Function, from: BlockId, args: &[Value], state_values: &[Value]| {
            for (k, phi) in param_phis.iter().enumerate() {
                if let Some(phi) = phi
                    && let Op::Phi { incoming, .. } = &mut f.insts[phi.index()].op
                {
                    incoming.push((args[k], from));
                }
            }
            for (phi, &v) in state_phis[..n_header_state].iter().zip(state_values) {
                if let Op::Phi { incoming, .. } = &mut f.insts[phi.index()].op {
                    incoming.push((v, from));
                }
            }
        };
        let state_values: Vec<Value> = state.iter().map(|&l| Value::Local(l)).collect();

        // An accumulator, for calls whose result is folded with an
        // associative operation and returned. It starts at the operation's
        // identity, each such call folds its other operand in, and every
        // return combines it with the returned value.
        let mut acc_phi = None;
        if let Some((op, _)) = acc {
            let Type::Function { ret, .. } = sh.ty(f.ty) else { unreachable!() };
            let Type::Int(bits) = sh.ty(ret) else { unreachable!() };
            let identity = match op {
                BinOp::Mul => 1,
                BinOp::And => -1,
                _ => 0,
            };
            let at = header_phis.len() + state_phis.len().min(n_header_state);
            let phi = self.phi(f, header, at, ret, "acc", vec![(sh.int(bits, identity), new_entry)]);
            let acc_value = f.result(phi);
            acc_phi = Some(phi);
            let rets: Vec<(BlockId, InstId)> = f
                .layout
                .iter()
                .filter_map(|&b| f.terminator(b).map(|t| (b, t)))
                .filter(|(_, t)| matches!(f.insts[t.index()].op, Op::Ret(Some(_))))
                .collect();
            for (b, t) in rets {
                let Op::Ret(Some(v)) = f.insts[t.index()].op else { unreachable!() };
                let name = self.name("acc.ret");
                let combined = f.add_inst(
                    Op::Binary { op, flags: IntFlags::NONE, fmf: FastMath::default(), lhs: acc_value, rhs: v },
                    Some((name, ret)),
                );
                let k = f.blocks[b.index()].insts.len() - 1;
                f.blocks[b.index()].insts.insert(k, combined);
                f.insts[t.index()].op = Op::Ret(Some(f.result(combined)));
            }
        }
        let add_acc_incoming = |f: &mut Function, from: BlockId, value: Value| {
            if let Some(phi) = acc_phi
                && let Op::Phi { incoming, .. } = &mut f.insts[phi.index()].op
            {
                incoming.push((value, from));
            }
        };

        // Tail calls: jump to the header, leaving the stack as it is.
        for &site in jumps {
            let b = f.inst_blocks()[site.index()].unwrap();
            let args: Vec<Value> = f.insts[site.index()].op.call().unwrap().args.iter().map(|a| a.value).collect();
            let mut old_succs = f.successors(b);
            old_succs.dedup();
            let insts = &mut f.blocks[b.index()].insts;
            let k = insts.iter().position(|&x| x == site).unwrap();
            let removed = insts.split_off(k);
            // Debug records on the call and the `ret` move to the jump.
            let records: Vec<DebugRecord> =
                removed.iter().flat_map(|i| f.insts[i.index()].debug_records.clone()).collect();
            let jump = f.push_inst(b, Op::Br(header), None);
            f.insts[jump.index()].debug_records = records;
            for s in old_succs {
                f.remove_phi_entries(s, b);
            }
            add_header_incoming(f, b, &args, &state_values);
            if let Some(phi) = acc_phi {
                let unchanged = f.result(phi);
                add_acc_incoming(f, b, unchanged);
            }
        }

        // Accumulator calls: fold the other operand in and jump.
        if let (Some((op, sites)), Some(phi)) = (acc, acc_phi) {
            let acc_value = f.result(phi);
            for &(site, u, x) in sites {
                // A parameter is now the header phi that replaced it.
                let x = match x {
                    Value::Local(l) => Value::Local(param_value(f, l)),
                    v => v,
                };
                let b = f.inst_blocks()[site.index()].unwrap();
                let args: Vec<Value> = f.insts[site.index()].op.call().unwrap().args.iter().map(|a| a.value).collect();
                let mut old_succs = f.successors(b);
                old_succs.dedup();
                // The call goes; its debug records move to what follows it.
                let records = std::mem::take(&mut f.insts[site.index()].debug_records);
                let insts = &mut f.blocks[b.index()].insts;
                let k = insts.iter().position(|&i| i == site).unwrap();
                insts.remove(k);
                let next = insts[k];
                f.insts[next.index()].debug_records.splice(0..0, records);
                // The use becomes the new accumulator, without the flags
                // that held for the old order of operations.
                f.insts[u.index()].op =
                    Op::Binary { op, flags: IntFlags::NONE, fmf: FastMath::default(), lhs: acc_value, rhs: x };
                let insts = &mut f.blocks[b.index()].insts;
                let at = insts.iter().position(|&i| i == u).unwrap();
                let removed = insts.split_off(at + 1);
                let records: Vec<DebugRecord> =
                    removed.iter().flat_map(|i| f.insts[i.index()].debug_records.clone()).collect();
                let jump = f.push_inst(b, Op::Br(header), None);
                f.insts[jump.index()].debug_records = records;
                for s in old_succs {
                    f.remove_phi_entries(s, b);
                }
                add_header_incoming(f, b, &args, &state_values);
                let folded = f.result(u);
                add_acc_incoming(f, b, folded);
            }
        }

        // Other calls: push a frame and jump to the header.
        struct Site {
            cont: BlockId,
            result: Option<LocalId>,
            frame_ty: TypeId,
            /// Each saved value and the frame field that holds it.
            saved: Vec<(LocalId, u32)>,
        }
        let link_fields: u32 = if many_sites { 3 } else { 2 };
        let mut pushed: Vec<Site> = Vec::new();
        for (k, &site) in frames.iter().enumerate() {
            let b = f.inst_blocks()[site.index()].unwrap();
            let call_inst = f.insts[site.index()].clone();
            let args: Vec<Value> = call_inst.op.call().unwrap().args.iter().map(|a| a.value).collect();
            let pos = f.blocks[b.index()].insts.iter().position(|&x| x == site).unwrap();
            let cont_name = self.name(&format!("after.call{k}"));
            let cont = f.split_block(b, pos + 1, cont_name);
            f.blocks[b.index()].insts.pop();
            let records = call_inst.debug_records.clone();

            if pool {
                let (chunk, index) = (state_values[0], state_values[1]);
                let path = vec![sh.int(32, 0), sh.int(32, 2), index];
                let frame = self.gep_path(f, b, chunk_ty, chunk, path, &format!("frame{k}"));
                if many_sites {
                    self.store(f, b, sh.int(32, k as i64), frame);
                }
                let saved: Vec<(LocalId, u32)> = saved[k].iter().copied().zip(site_fields[k].iter().copied()).collect();
                for &(l, field) in &saved {
                    let at = self.gep(f, b, pool_frame_ty, frame, field, "frame.slot");
                    self.store(f, b, Value::Local(l), at);
                }
                let name = self.name("index.next");
                let next = f.push_inst(
                    b,
                    Op::Binary {
                        op: BinOp::Add,
                        flags: IntFlags::NUW,
                        fmf: FastMath::default(),
                        lhs: index,
                        rhs: sh.int(32, 1),
                    },
                    Some((name, self.i32)),
                );
                let next = f.result(next);
                let jump = f.push_inst(b, Op::Br(header), None);
                f.insts[jump.index()].debug_records = records;
                add_header_incoming(f, b, &args, &[chunk, next]);
                pushed.push(Site { cont, result: call_inst.result, frame_ty: pool_frame_ty, saved });
            } else {
                let mut fields = vec![self.ptr, self.ptr];
                if many_sites {
                    fields.push(self.i32);
                }
                fields.extend(saved[k].iter().map(|l| f.locals[l.index()].ty));
                let frame_ty = sh.intern_type(Type::Struct { packed: false, fields });
                let sp = self.call_intrinsic(f, b, true, None).unwrap();
                let name = self.name(&format!("frame{k}"));
                let frame = f.push_inst(
                    b,
                    Op::Alloca { inalloca: false, ty: frame_ty, count: None, align: None, addrspace: None },
                    Some((name, self.ptr)),
                );
                let frame = f.result(frame);
                self.store(f, b, state_values[0], frame);
                let at = self.gep(f, b, frame_ty, frame, 1, "frame.sp");
                self.store(f, b, sp, at);
                if many_sites {
                    let at = self.gep(f, b, frame_ty, frame, 2, "frame.site");
                    self.store(f, b, sh.int(32, k as i64), at);
                }
                let saved: Vec<(LocalId, u32)> =
                    saved[k].iter().enumerate().map(|(j, &l)| (l, link_fields + j as u32)).collect();
                for &(l, field) in &saved {
                    let at = self.gep(f, b, frame_ty, frame, field, "frame.slot");
                    self.store(f, b, Value::Local(l), at);
                }
                let jump = f.push_inst(b, Op::Br(header), None);
                f.insts[jump.index()].debug_records = records;
                add_header_incoming(f, b, &args, &[frame]);
                pushed.push(Site { cont, result: call_inst.result, frame_ty, saved });
            }
        }

        // Returns go through a dispatch block that pops a frame or returns.
        let mut pops: Vec<(BlockId, Vec<(LocalId, Value)>)> = Vec::new();
        let mut rv = None;
        let mut dispatch_phis: Vec<(BlockId, InstId)> = Vec::new();
        if use_stack {
            let rets: Vec<(BlockId, InstId)> = f
                .layout
                .iter()
                .filter_map(|&b| f.terminator(b).map(|t| (b, t)))
                .filter(|(_, t)| matches!(f.insts[t.index()].op, Op::Ret(_)))
                .collect();
            let dispatch = f.add_block(self.name("rec.return"));
            let exit = f.add_block(self.name("return"));
            let pop = f.add_block(self.name("rec.pop"));
            let mut ret_incoming = Vec::new();
            for &(b, t) in &rets {
                if let Op::Ret(Some(v)) = f.insts[t.index()].op {
                    ret_incoming.push((v, b));
                }
                f.insts[t.index()].op = Op::Br(dispatch);
            }
            if returns_value {
                let phi = f.push_inst(
                    dispatch,
                    Op::Phi { fmf: FastMath::default(), incoming: ret_incoming },
                    Some((self.name("result"), ret_ty)),
                );
                dispatch_phis.push((dispatch, phi));
                rv = Some(f.result(phi));
            }
            f.push_inst(exit, Op::Ret(rv), None);

            // The frame to pop, in `pop`.
            let frame;
            if pool {
                let (chunk, index) = (state_values[0], state_values[1]);
                let underflow = f.add_block(self.name("rec.underflow"));
                let shrink = f.add_block(self.name("rec.shrink"));
                f.layout.extend([dispatch, underflow, exit, shrink, pop]);
                let at_start = self.icmp_eq(f, dispatch, index, sh.int(32, 0), "chunk.empty");
                f.push_inst(dispatch, Op::CondBr { cond: at_start, then_dest: underflow, else_dest: pop }, None);
                let name = self.name("chunk.prev");
                let prev = self.load(f, underflow, self.ptr, chunk, name);
                let empty = self.icmp_eq(f, underflow, prev, sh.null_ptr(), "stack.empty");
                f.push_inst(underflow, Op::CondBr { cond: empty, then_dest: exit, else_dest: shrink }, None);
                let at = self.gep(f, shrink, chunk_ty, chunk, 1, "chunk.sp");
                let name = self.name("saved.sp");
                let sp = self.load(f, shrink, self.ptr, at, name);
                self.call_intrinsic(f, shrink, false, Some(sp));
                f.push_inst(shrink, Op::Br(pop), None);
                let c = self.phi(f, pop, 0, self.ptr, "chunk.pop", vec![(chunk, dispatch), (prev, shrink)]);
                let i =
                    self.phi(f, pop, 1, self.i32, "index.pop", vec![(index, dispatch), (sh.int(32, capacity), shrink)]);
                dispatch_phis.extend([(pop, c), (pop, i)]);
                let (c, i) = (f.result(c), f.result(i));
                let name = self.name("index.prev");
                let prev_index = f.push_inst(
                    pop,
                    Op::Binary {
                        op: BinOp::Sub,
                        flags: IntFlags::NUW,
                        fmf: FastMath::default(),
                        lhs: i,
                        rhs: sh.int(32, 1),
                    },
                    Some((name, self.i32)),
                );
                let prev_index = f.result(prev_index);
                frame = self.gep_path(f, pop, chunk_ty, c, vec![sh.int(32, 0), sh.int(32, 2), prev_index], "frame");
                state_defs.push((0, pop, c));
                state_defs.push((1, pop, prev_index));
            } else {
                f.layout.extend([dispatch, exit, pop]);
                let frame_incoming: Vec<(Value, BlockId)> = rets.iter().map(|&(b, _)| (state_values[0], b)).collect();
                let frame_phi = f.push_inst(
                    dispatch,
                    Op::Phi { fmf: FastMath::default(), incoming: frame_incoming },
                    Some((self.name("frame"), self.ptr)),
                );
                dispatch_phis.push((dispatch, frame_phi));
                frame = f.result(frame_phi);
                let empty = self.icmp_eq(f, dispatch, frame, sh.null_ptr(), "stack.empty");
                f.push_inst(dispatch, Op::CondBr { cond: empty, then_dest: exit, else_dest: pop }, None);
                let name = self.name("frame.prev");
                let prev = self.load(f, pop, self.ptr, frame, name);
                state_defs.push((0, pop, prev));
            }
            let sp = if pool {
                None
            } else {
                let link_ty = {
                    let mut fields = vec![self.ptr, self.ptr];
                    if many_sites {
                        fields.push(self.i32);
                    }
                    sh.intern_type(Type::Struct { packed: false, fields })
                };
                let at = self.gep(f, pop, link_ty, frame, 1, "frame.sp");
                let name = self.name("saved.sp");
                Some((self.load(f, pop, self.ptr, at, name), link_ty))
            };
            let mut pop_blocks = Vec::new();
            for (k, s) in pushed.iter().enumerate() {
                let pb = f.add_block(self.name(&format!("rec.pop{k}")));
                f.layout.push(pb);
                let mut reloads = Vec::new();
                for &(l, field) in &s.saved {
                    let at = self.gep(f, pb, s.frame_ty, frame, field, "frame.slot");
                    let ty = f.locals[l.index()].ty;
                    let name = f.locals[l.index()].name.clone().map(|n| self.names.fresh(&format!("{n}.saved")));
                    let v = self.load(f, pb, ty, at, name);
                    reloads.push((l, v));
                }
                if let Some((sp, _)) = sp {
                    self.call_intrinsic(f, pb, false, Some(sp));
                }
                f.push_inst(pb, Op::Br(s.cont), None);
                pop_blocks.push(pb);
                pops.push((pb, reloads));
            }
            if pushed.len() == 1 {
                f.push_inst(pop, Op::Br(pop_blocks[0]), None);
            } else {
                let at = match sp {
                    Some((_, link_ty)) => self.gep(f, pop, link_ty, frame, 2, "frame.site"),
                    None => frame,
                };
                let name = self.name("site");
                let site = self.load(f, pop, self.i32, at, name);
                let last = *pop_blocks.last().unwrap();
                let cases = pop_blocks[..pop_blocks.len() - 1]
                    .iter()
                    .enumerate()
                    .map(|(k, &pb)| {
                        let Value::Const(c) = sh.int(32, k as i64) else { unreachable!() };
                        (c, pb)
                    })
                    .collect();
                f.push_inst(pop, Op::Switch { value: site, default: last, cases }, None);
            }
        }

        // Rebuild SSA form.
        let preds = f.predecessors();
        let mut up = SsaUpdater::new(preds);
        let mut var_of: HashMap<LocalId, usize> = HashMap::default();
        let def_blocks = f.inst_blocks();
        let mut new_var = |up: &mut SsaUpdater, f: &Function, l: LocalId| -> usize {
            *var_of.entry(l).or_insert_with(|| {
                let ty = f.locals[l.index()].ty;
                up.add_var(ty, f.locals[l.index()].name.clone(), sh.poison(ty))
            })
        };
        // The stack's state.
        let state_vars: Vec<usize> = state.iter().map(|&l| new_var(&mut up, f, l)).collect();
        for &(k, b, v) in &state_defs {
            up.def(state_vars[k], b, v);
        }
        // Call results: the returned value, at the continuation. The call
        // itself is gone.
        let mut call_results = HashSet::default();
        for s in &pushed {
            if let (Some(r), Some(v)) = (s.result, rv) {
                let var = new_var(&mut up, f, r);
                up.def(var, s.cont, v);
                call_results.insert(r);
            }
        }
        // Saved values: their own definition, plus each reload.
        for (pb, reloads) in &pops {
            for &(l, v) in reloads {
                let var = new_var(&mut up, f, l);
                if !call_results.contains(&l) {
                    let LocalDef::Inst(def) = f.locals[l.index()].def else { unreachable!() };
                    let db = def_blocks[def.index()].expect("saved value is not defined in the body");
                    up.def(var, db, Value::Local(l));
                }
                up.def(var, *pb, v);
            }
        }
        let var_of = var_of;

        // Rewrite every use of a variable.
        let snapshot: Vec<(BlockId, InstId)> =
            f.layout.iter().flat_map(|&b| f.blocks[b.index()].insts.iter().map(move |&i| (b, i))).collect();
        for &(b, i) in &snapshot {
            let mut wanted: Vec<Option<(usize, BlockId)>> = Vec::new();
            match &f.insts[i.index()].op {
                Op::Phi { incoming, .. } => {
                    for (v, p) in incoming {
                        wanted.push(match v {
                            Value::Local(l) => var_of.get(l).map(|&x| (x, *p)),
                            _ => None,
                        });
                    }
                }
                op => op.for_each_value(|v| {
                    wanted.push(match v {
                        Value::Local(l) => var_of.get(l).map(|&x| (x, b)),
                        _ => None,
                    })
                }),
            }
            if wanted.iter().all(Option::is_none) {
                continue;
            }
            let values: Vec<Option<Value>> =
                wanted.iter().map(|w| w.map(|(var, at)| up.value_at(f, &mut self.names, var, at))).collect();
            let mut it = values.into_iter();
            f.insts[i.index()].op.for_each_value_mut(|v| {
                if let Some(Some(new)) = it.next() {
                    *v = new;
                }
            });
        }

        // Tidy: drop phis that turned out to have one value.
        let mut candidates = up.phis.clone();
        candidates.extend(header_phis.iter().map(|&p| (header, p)));
        candidates.extend(acc_phi.map(|p| (header, p)));
        let inst_blocks = f.inst_blocks();
        candidates.extend(state_phis.iter().map(|&p| (inst_blocks[p.index()].unwrap(), p)));
        candidates.extend(dispatch_phis);
        let replaced = ssa::remove_trivial_phis(f, &candidates);

        // Debug records: point them at the value that reaches them, if one
        // is known without new phis; otherwise mark them optimised out.
        fix_debug_records(sh, f, &up, &var_of, &replaced);
        if merge {
            crate::cleanup::merge_blocks(f);
        }

        Outcome {
            tail_calls: jumps.len(),
            accumulator_calls: acc.map_or(0, |(_, s)| s.len()),
            accumulator_op: acc.map(|(op, _)| op),
            frame_calls: pushed.iter().map(|s| s.saved.len()).collect(),
            kept: 0,
        }
    }
}

/// A rough size in bytes, for choosing how many frames fit in a chunk.
fn size_estimate(sh: &Shared, ty: TypeId) -> u64 {
    match sh.ty(ty) {
        Type::Int(bits) | Type::Byte(bits) => u64::from(bits.div_ceil(8)).next_power_of_two(),
        Type::Half | Type::BFloat => 2,
        Type::Float => 4,
        Type::Double | Type::Ptr(_) => 8,
        Type::X86Fp80 | Type::Fp128 | Type::PpcFp128 => 16,
        Type::Vector { len, elem, .. } => u64::from(len) * size_estimate(sh, elem),
        Type::Array { len, elem } => len * size_estimate(sh, elem),
        Type::Struct { fields, .. } => fields.iter().map(|&t| size_estimate(sh, t)).sum(),
        Type::Named(i) => {
            let body = sh.types().structs()[i as usize].body.clone();
            body.map_or(8, |b| b.fields.iter().map(|&t| size_estimate(sh, t)).sum())
        }
        _ => 8,
    }
}

/// Updates the values in debug records after SSA has been rebuilt. A
/// record that names a variable gets the value reaching it; a record whose
/// value no longer dominates it gets `poison`, which means "optimised out".
pub(crate) fn fix_debug_records(
    sh: &Shared,
    f: &mut Function,
    up: &SsaUpdater,
    var_of: &HashMap<LocalId, usize>,
    replaced: &HashMap<LocalId, Value>,
) {
    let dom = DomTree::new(f);
    let mut def_at: Vec<Option<(BlockId, usize)>> = vec![None; f.locals.len()];
    for p in &f.params {
        def_at[p.local.index()] = Some((f.entry(), 0));
    }
    for &b in &f.layout {
        for (k, &i) in f.blocks[b.index()].insts.iter().enumerate() {
            if let Some(r) = f.insts[i.index()].result {
                def_at[r.index()] = Some((b, k + 1));
            }
        }
    }
    let resolve = |mut v: Value| {
        while let Value::Local(l) = v {
            match replaced.get(&l) {
                Some(&next) if next != v => v = next,
                _ => break,
            }
        }
        v
    };
    let layout = f.layout.clone();
    for b in layout {
        for k in 0..f.blocks[b.index()].insts.len() {
            let i = f.blocks[b.index()].insts[k];
            if f.insts[i.index()].debug_records.is_empty() {
                continue;
            }
            let mut records = std::mem::take(&mut f.insts[i.index()].debug_records);
            for r in &mut records {
                for arg in &mut r.args {
                    let data = sh.metadata()[arg.index()].data.clone();
                    let fix = |v: Value| -> Value {
                        let Value::Local(l) = v else { return v };
                        let v = match var_of.get(&l) {
                            Some(&var) => up.peek(var, b).map(resolve).unwrap_or(v),
                            None => resolve(v),
                        };
                        let Value::Local(l2) = v else { return v };
                        let ok = match def_at[l2.index()] {
                            Some((db, pos)) => {
                                if db == b {
                                    pos <= k
                                } else {
                                    dom.dominates(db, b)
                                }
                            }
                            None => false,
                        };
                        if ok { v } else { sh.poison(f.locals[l2.index()].ty) }
                    };
                    match data {
                        Metadata::Value(v @ Value::Local(_)) => {
                            let new = fix(v);
                            if new != v {
                                *arg = sh.md_value(new);
                            }
                        }
                        Metadata::Node { distinct, kind, fields }
                            if fields.iter().any(|x| matches!(x.value, MdFieldValue::Value(Value::Local(_)))) =>
                        {
                            let new_fields: Vec<MdField> = fields
                                .iter()
                                .map(|x| match x.value {
                                    MdFieldValue::Value(v) => {
                                        MdField { name: x.name.clone(), value: MdFieldValue::Value(fix(v)) }
                                    }
                                    _ => x.clone(),
                                })
                                .collect();
                            if new_fields != fields {
                                *arg = sh.add_md(Metadata::Node { distinct, kind, fields: new_fields });
                            }
                        }
                        _ => {}
                    }
                }
            }
            f.insts[i.index()].debug_records = records;
        }
    }
}

/// Reduces a cycle of several functions to self-recursive functions by
/// inlining.
///
/// An entry is a member that can be called from outside the SCC: it is not
/// internal, its address is taken, or a function outside the SCC calls it.
/// Each entry, working on a copy of the original bodies, inlines its calls
/// to the other members, repeatedly, until it calls only itself and
/// functions outside the SCC. Members that are not entries are then unused
/// and are deleted later. With several entries, each gets its own copy of
/// the cycle, so the size limit keeps this to small cycles.
///
/// Self-recursion inside members should be removed first (`eliminate`), so
/// that inlining a member does not bring its self-calls along forever.
///
/// Returns, for each entry (an index into `members`), whether it was
/// collapsed. A failed entry keeps its original body.
pub fn collapse_scc(
    sh: &Shared,
    cg: &CallGraph,
    members: &mut [(GlobalId, Box<Function>)],
    max_inlines: usize,
) -> Vec<(usize, Result<(), String>)> {
    let m = sh.module;
    let entries: Vec<usize> = members
        .iter()
        .enumerate()
        .filter(|(_, (g, _))| {
            let n = cg.node(*g).unwrap();
            let local = matches!(m.global(*g).props.linkage, Linkage::Internal | Linkage::Private);
            let outside = cg.call_sites[n] > cg.sites_in_scc[n];
            !local || cg.address_taken[n] || outside
        })
        .map(|(k, _)| k)
        .collect();
    let size = |f: &Function| f.layout.iter().map(|b| f.blocks[b.index()].insts.len()).sum::<usize>();
    let start: usize = members.iter().map(|(_, f)| size(f)).sum();
    let limit = start * 4 + 200;
    let originals: Vec<(GlobalId, Function)> = members.iter().map(|(g, f)| (*g, (**f).clone())).collect();

    let mut results = Vec::new();
    for &e in &entries {
        let head_g = originals[e].0;
        if attrs::has(m, &originals[e].1.fn_attrs, "optnone") {
            results.push((e, Err("optnone".into())));
            continue;
        }
        let mut head = originals[e].1.clone();
        let bodies: HashMap<GlobalId, &Function> =
            originals.iter().filter(|(g, _)| *g != head_g).map(|(g, f)| (*g, f)).collect();
        let mut names = Names::new(&head);
        let mut memo = inline::DebugMemo::default();
        let mut done = 0;
        let result = loop {
            let site = head.insts_in_order().find(|&i| {
                head.insts[i.index()]
                    .op
                    .call()
                    .and_then(|c| sh.direct_callee(c))
                    .is_some_and(|t| bodies.contains_key(&t))
            });
            let Some(site) = site else { break Ok(()) };
            if done == max_inlines {
                break Err(format!("still calls other members after {max_inlines} inlines"));
            }
            if size(&head) > limit {
                break Err("collapsing would grow the function too much".into());
            }
            let call = head.insts[site.index()].op.call().unwrap();
            let callee_g = sh.direct_callee(call).unwrap();
            let callee = bodies[&callee_g];
            if !matches!(head.insts[site.index()].op, Op::Call(_))
                || call.fn_ty != callee.ty
                || call.cconv != callee.cconv
                || !call.bundles.is_empty()
                || call.tail == TailKind::MustTail
            {
                break Err(format!("cannot inline the call to @{}", sh.name(callee_g)));
            }
            if attrs::has(m, &callee.fn_attrs, "noinline")
                || attrs::has(m, &callee.fn_attrs, "optnone")
                || attrs::has(m, &call.fn_attrs, "noinline")
            {
                break Err(format!("@{} is noinline", sh.name(callee_g)));
            }
            if !attrs::definition_is_final(m.global(callee_g).props.linkage) {
                break Err(format!("@{} may be replaced at link time", sh.name(callee_g)));
            }
            let Type::Function { varargs, .. } = sh.ty(callee.ty) else { unreachable!() };
            if varargs
                || callee.insts_in_order().any(|i| matches!(f_op(callee, i), Op::IndirectBr { .. } | Op::CallBr { .. }))
            {
                break Err(format!("cannot inline @{}", sh.name(callee_g)));
            }
            let dynamic = callee.insts_in_order().any(|i| {
                let in_entry = callee.blocks[callee.entry().index()].insts.contains(&i);
                matches!(f_op(callee, i), Op::Alloca { count, .. } if !in_entry || count.is_some_and(|c| !matches!(c, Value::Const(_))))
            });
            let block = head.inst_blocks()[site.index()].unwrap();
            let name = sh.name(callee_g);
            inline::inline_call(sh, &mut head, &mut names, &mut memo, site, block, callee, &name, dynamic);
            done += 1;
        };
        if result.is_ok() {
            let n = cg.node(head_g).unwrap();
            if !cg.has_blockaddress[n] {
                crate::cleanup::merge_blocks(&mut head);
            }
            *members[e].1 = head;
        }
        results.push((e, result));
    }
    if entries.is_empty() {
        results.push((0, Err("the cycle is never entered".into())));
    }
    results
}

fn f_op(f: &Function, i: InstId) -> &Op {
    &f.insts[i.index()].op
}
