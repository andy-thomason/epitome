//! Lowering one function body: expressions to values and places,
//! statements, control flow and calls. Patterns are in `pat.rs`, and
//! intrinsics in `intrinsic.rs`.

use super::{Cx, Decl, Repr, is_prim};
use crate::ast::{Ast, Kind, NodeId};
use crate::hash::FastMap;
use crate::layout::{Fields, Layout};
use crate::mono::{Item, Resolved};
use crate::resolve::{DefId, DefKind, Res};
use crate::ty::{AdtKind, FloatTy, IntTy, List, Mutbl, ParamId, Ty, TyKind, UintTy};
use crate::typeck::{Adjust, Solver, Tables};
use epitome_ir::build::FunctionBuilder;
use epitome_ir::module::{Align, BinOp, BlockId, CastOp, FCmpPred, Function as IrFunction, GlobalId, ICmpPred, Linkage, Module, Op, Value};
use epitome_ir::types::TypeId;
use std::sync::Arc;

pub(crate) type R<T> = Result<T, String>;

/// A place in memory: its address, and for an unsized type the length or
/// vtable.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Place {
    pub addr: Value,
    pub meta: Option<Value>,
    pub ty: Ty,
}

/// A value, held as its type's `Repr` says.
#[derive(Clone, Copy, Debug)]
pub(crate) enum Val {
    Imm(Value),
    /// In memory at this address. Whoever keeps the value copies it.
    Mem(Value),
    Zst,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct Operand {
    pub ty: Ty,
    pub v: Val,
}

/// What an expression gives: a place (for place expressions) or a value.
#[derive(Clone, Copy, Debug)]
pub(crate) enum Pv {
    Place(Place),
    Val(Operand),
}

/// The body being walked: its file, its tables, and the arguments of its
/// generic parameters.
#[derive(Clone)]
pub(crate) struct Body<'a, 's> {
    pub k: u32,
    pub f: u32,
    pub ast: &'a Ast<'s>,
    pub types: &'a [Option<Ty>],
    pub tables: &'a Tables,
    pub params: Vec<ParamId>,
    pub args: Vec<Ty>,
}

/// Where `return` puts the result.
#[derive(Clone, Copy)]
enum Ret {
    Void,
    Imm,
    Sret(Value),
}

/// A loop or labelled block that `break` can leave.
struct Breakable<'s> {
    label: Option<&'s str>,
    /// A labelled block, which only a labelled `break` leaves.
    block: bool,
    exit: BlockId,
    /// Where `continue` goes (none for a block).
    cont: Option<BlockId>,
    /// Where `break value` puts the value.
    result: Option<Place>,
}

pub(crate) struct Fx<'c, 'a, 's, 'm> {
    pub cx: &'c Cx<'a, 's>,
    pub b: FunctionBuilder<'m>,
    pub solver: Solver<'a, 's>,
    pub body: Body<'a, 's>,
    entry: BlockId,
    /// Each binding's slot, by crate, file and pattern node.
    pub locals: FastMap<(u32, u32, NodeId), Place>,
    /// In a closure: each captured local's place among the captures, and
    /// whether it is held by value.
    upvars: FastMap<(u32, u32, NodeId), (usize, bool)>,
    /// In a closure: its captures.
    env: Option<Place>,
    ret: Ret,
    ret_ty: Ty,
    loops: Vec<Breakable<'s>>,
    /// Source comments: block, position in the block, text.
    notes: Vec<(BlockId, usize, String)>,
    /// The function's symbol, and how many string constants it has made.
    fname: String,
    strings: usize,
    blocks: usize,
    /// What the function traps on where it is reached (see `trap_here`).
    pub traps: Vec<String>,
    /// The `try` blocks around: where each ends, and its value's slot.
    tries: Vec<(BlockId, Place)>,
}

/// Source comments for `--dump-ir`: block, position in the block, text.
pub(crate) type Notes = Vec<(BlockId, usize, String)>;

/// Lowers the body of function (or closure) `it` into the function `decl`
/// declares. Gives the source comments for `--dump-ir`, and what the
/// function traps on where it is reached (see `Fx::trap_here`).
pub(crate) fn lower_fn(cx: &Cx, m: &mut Module, it: Item, decl: &Decl) -> R<(Notes, Vec<String>)> {
    let tcx = cx.tcx;
    let (owner, l, closure) = match it {
        Item::Fn(d, l) => (d, l, None),
        Item::Closure(c, d, l) => (d, l, Some(c)),
        _ => return Err("not a function".into()),
    };
    let (k, f, ast, node) = match closure {
        Some(c) => {
            let (k, f, n) = tcx.closure_at(c);
            (k, f, &cx.crates[k as usize].files[f as usize].ast, n)
        }
        None => cx.def_ast(owner),
    };
    let body_node = match closure {
        Some(_) => *ast.children(node).last().ok_or("closure without a body")?,
        None => ast.child(node, Kind::Block).ok_or("no body")?,
    };
    let (ret, params) = Cx::ir_fn_type(m, &decl.abi);
    let mut spec = IrFunction::builder(&decl.name).returns(ret);
    for p in &params {
        spec = spec.param("", *p);
    }
    if decl.abi.variadic {
        spec = spec.varargs();
    }
    let linkage = m.global(decl.g).props.linkage;
    if linkage != Linkage::External {
        spec = spec.linkage(linkage);
    }
    let b = spec.begin(m);
    let body = Body {
        k,
        f,
        ast,
        types: &cx.typeck.types[k as usize][f as usize],
        tables: &cx.typeck.tables[k as usize][f as usize],
        params: tcx.all_params(owner),
        args: tcx.tys(l),
    };
    let mut fx = Fx::new(cx, b, body, decl.abi.output, &decl.name);
    let first = fx.b.current_block().expect("the body block");
    let mut ir_param = 0;
    fx.ret = match decl.abi.ret {
        Repr::Mem => {
            ir_param = 1;
            Ret::Sret(fx.b.param(0))
        }
        Repr::Imm(_) => Ret::Imm,
        Repr::Zst => Ret::Void,
    };
    // The parameters: each is copied into a slot of its own, then its
    // pattern binds from there. A closure's first is its captures.
    let pnodes: Vec<NodeId> = match closure {
        Some(_) => {
            let ps = ast.child(node, Kind::ClosureParams);
            ps.map(|ps| ast.children(ps).iter().copied().filter(|&c| ast.kind(c) == Kind::ClosureParam).collect()).unwrap_or_default()
        }
        None => {
            let pnode = ast.child(node, Kind::Params).ok_or("no parameters")?;
            ast.children(pnode).iter().copied().filter(|&c| matches!(ast.kind(c), Kind::SelfParam | Kind::Param)).collect()
        }
    };
    let skip = closure.is_some() as usize;
    if pnodes.len() + skip != decl.abi.inputs.len() {
        return Err("parameters and signature disagree".into());
    }
    if let Some(c) = closure {
        let env_ty = fx.pointee(decl.abi.inputs[0]).ok_or("closure without its captures")?;
        let p = match decl.abi.args[0] {
            Repr::Imm(_) => {
                let v = fx.b.param(ir_param);
                ir_param += 1;
                v
            }
            _ => return Err("closure captures not passed by address".into()),
        };
        fx.env = Some(Place { addr: p, meta: None, ty: env_ty });
        for (i, cap) in cx.typeck.captures.get(&c).into_iter().flatten().enumerate() {
            fx.upvars.insert((k, f, cap.local), (i, cap.by_value));
        }
    }
    for (i, &p) in pnodes.iter().enumerate() {
        let t = decl.abi.inputs[i + skip];
        let slot = fx.temp(t)?;
        match decl.abi.args[i + skip] {
            Repr::Imm(_) => {
                let v = fx.b.param(ir_param);
                ir_param += 1;
                fx.store(Operand { ty: t, v: Val::Imm(v) }, slot)?;
            }
            Repr::Mem => {
                let v = fx.b.param(ir_param);
                ir_param += 1;
                fx.store(Operand { ty: t, v: Val::Mem(v) }, slot)?;
            }
            Repr::Zst => {}
        }
        match ast.kind(p) {
            Kind::SelfParam => {
                fx.locals.insert((k, f, p), slot);
            }
            _ => {
                let pat = ast.children(p).iter().copied().find(|&c| super::pat::is_pattern(ast.kind(c))).ok_or("parameter without a pattern")?;
                fx.bind_irrefutable(pat, slot)?;
            }
        }
    }
    let v = fx.expr(body_node)?;
    fx.ret_value(v)?;
    // Every block needs a terminator: those left open are unreachable.
    fx.close_open_blocks();
    let entry = fx.entry;
    fx.b.at(entry);
    fx.b.br(first);
    let notes = std::mem::take(&mut fx.notes);
    let traps = std::mem::take(&mut fx.traps);
    fx.b.finish()?;
    Ok((notes, traps))
}

/// Replaces `decl`'s body (or declaration) with one that traps.
pub(crate) fn trap_body(m: &mut Module, decl: &Decl) {
    let (ret, params) = Cx::ir_fn_type(m, &decl.abi);
    let mut spec = IrFunction::builder(&decl.name).returns(ret);
    for p in &params {
        spec = spec.param("", *p);
    }
    if decl.abi.variadic {
        spec = spec.varargs();
    }
    let linkage = m.global(decl.g).props.linkage;
    if linkage != Linkage::External {
        spec = spec.linkage(linkage);
    }
    let mut b = spec.begin(m);
    b.block("start");
    let void = b.module().types.void();
    let trap = b.module().declare("llvm.trap", void, &[], false);
    b.call(trap, &[]);
    b.unreachable();
    let _ = b.finish();
}

/// Defines a `static`: a global holding its value. A value the evaluator
/// can work out (an integer, `bool` or `char`) is the global's
/// initialiser. Any other is not known until its initialiser runs: the
/// global starts as zeros, and the second result is true, for the caller
/// to lower the initialiser (`lower_static_init`) to run at start-up.
pub(crate) fn define_static(cx: &Cx, m: &mut Module, d: DefId, name: &str) -> R<(GlobalId, bool)> {
    let t = cx.tcx.type_of(d).ok_or("static without a type")?;
    let l = cx.layout(t)?;
    let value = cx.lcx.eval_item(d).and_then(|v| v.int());
    let bytes: Vec<u8> = match value {
        Some(value) => (0..l.size).map(|i| if i < 16 { (value as u128 >> (8 * i)) as u8 } else { 0 }).collect(),
        None => vec![0; l.size as usize],
    };
    let init = m.const_bytes(&bytes);
    let (_, _, ast, node) = cx.def_ast(d);
    let mutable = ast.children(node).iter().any(|&c| ast.kind(c) == Kind::Keyword && ast.token(c).0 == "mut");
    let mut g = epitome_ir::module::Global::builder(name).init(init).align(l.align.max(1));
    if !mutable && value.is_some() {
        g = g.constant();
    }
    if !crate::session::has_attribute(ast, node, "no_mangle") {
        g = g.linkage(Linkage::Internal);
    }
    Ok((g.build(m), value.is_none()))
}

/// The function `name` (`void()`, internal) that stores the value of
/// static `d`'s initialiser in its global `g`, for `llvm.global_ctors` to
/// run before `main`. A static's initialiser is a constant expression: it
/// has no effects to run in another order, and in a `no_std` program
/// nothing reads the static before `main`.
pub(crate) fn lower_static_init(cx: &Cx, m: &mut Module, d: DefId, g: GlobalId, name: &str) -> R<GlobalId> {
    let (k, f, ast, node) = cx.def_ast(d);
    let t = cx.tcx.type_of(d).ok_or("static without a type")?;
    let init = ast.children(node).iter().rev().copied().find(|&x| super::pat::is_expr(ast.kind(x))).ok_or("static without a value")?;
    let b = IrFunction::builder(name).linkage(Linkage::Internal).begin(m);
    let body = Body {
        k,
        f,
        ast,
        types: &cx.typeck.types[k as usize][f as usize],
        tables: &cx.typeck.tables[k as usize][f as usize],
        params: Vec::new(),
        args: Vec::new(),
    };
    let unit = cx.tcx.unit();
    let mut fx = Fx::new(cx, b, body, unit, name);
    let first = fx.b.current_block().expect("the body block");
    let o = fx.operand(init)?;
    let addr = fx.b.module().global_value(g);
    if !fx.terminated() {
        fx.store(Operand { ty: t, v: o.v }, Place { addr, meta: None, ty: t })?;
        fx.b.ret_void();
    }
    fx.close_open_blocks();
    let entry = fx.entry;
    fx.b.at(entry);
    fx.b.br(first);
    fx.b.finish()
}

impl<'c, 'a, 's, 'm> Fx<'c, 'a, 's, 'm> {
    /// A body to lower into the function `b` begins (called `name`,
    /// returning `ret_ty`): its entry block for stack slots, and the block
    /// its code starts in, which is current.
    fn new(cx: &'c Cx<'a, 's>, b: FunctionBuilder<'m>, body: Body<'a, 's>, ret_ty: Ty, name: &str) -> Self {
        let mut fx = Fx {
            cx,
            b,
            solver: cx.solver(),
            body,
            entry: BlockId(0),
            locals: FastMap::default(),
            upvars: FastMap::default(),
            env: None,
            ret: Ret::Void,
            ret_ty,
            loops: Vec::new(),
            notes: Vec::new(),
            fname: name.to_string(),
            strings: 0,
            blocks: 0,
            traps: Vec::new(),
            tries: Vec::new(),
        };
        fx.entry = fx.b.block("start");
        let first = fx.b.new_block("body");
        fx.b.at(first);
        fx
    }

    // ---- types and layouts ----

    /// `t` at this instance's arguments.
    pub fn sub(&mut self, t: Ty) -> Ty {
        crate::mono::sub(self.cx.tcx, &self.cx.typeck.hidden, &mut self.solver, t, &self.body.params, &self.body.args)
    }

    pub fn sub_list(&mut self, l: List) -> Vec<Ty> {
        self.cx.tcx.tys(l).into_iter().map(|t| self.sub(t)).collect()
    }

    /// The type of node `n`, at this instance.
    pub fn ty(&mut self, n: NodeId) -> R<Ty> {
        let t = self.body.types.get(n.0 as usize).copied().flatten().ok_or_else(|| format!("no type for `{}`", self.text(n)))?;
        let t = self.sub(t);
        if crate::mono::has_param_or_infer(self.cx.tcx, t) || self.cx.tcx.has_error(t) {
            return Err(format!("type not concrete: `{}`: {}", self.text(n), self.cx.tcx.display(self.cx.r, t)));
        }
        Ok(t)
    }

    pub fn layout(&self, t: Ty) -> R<Arc<Layout>> {
        self.cx.layout(t)
    }

    pub fn repr(&mut self, t: Ty) -> R<Repr> {
        self.cx.repr(self.b.module(), t)
    }

    pub fn text(&self, n: NodeId) -> String {
        let mut t = self.body.ast.text(n).replace('\n', " ");
        if t.len() > 70 {
            let cut = (0..=70).rev().find(|&i| t.is_char_boundary(i)).unwrap_or(0);
            t.truncate(cut);
            t.push_str("...");
        }
        t
    }

    pub fn kids(&self, n: NodeId) -> Vec<NodeId> {
        self.body.ast.children(n).iter().copied().filter(|&c| !matches!(self.body.ast.kind(c), Kind::Attribute | Kind::DocComment)).collect()
    }

    pub fn res(&self, n: NodeId) -> Option<&'a Res> {
        self.cx.r.res_of(self.body.k as usize, self.body.f as usize, n)
    }

    pub fn unit(&self) -> Operand {
        Operand { ty: self.cx.tcx.unit(), v: Val::Zst }
    }

    fn note(&mut self, n: NodeId) {
        let Some(bb) = self.b.current_block() else { return };
        let at = self.b.function().blocks[bb.index()].insts.len();
        let t = self.body.ast.token(n);
        let line = self.cx.crates[self.body.k as usize]
            .files
            .get(self.body.f as usize)
            .and_then(|file| file.source.text.find(t.0).filter(|_| self.body.ast.is_original(n)).map(|_| file))
            .map(|file| {
                let off = t.0.as_ptr() as usize - file.source.text.as_ptr() as usize;
                file.source.text[..off.min(file.source.text.len())].lines().count().max(1)
            });
        let text = self.text(n);
        let note = match line {
            Some(l) => format!("{l}: {text}"),
            None => text,
        };
        self.notes.push((bb, at, note));
    }

    // ---- IR helpers ----

    pub fn i8t(&mut self) -> TypeId {
        self.b.module().types.int(8)
    }

    pub fn isize_t(&mut self) -> TypeId {
        let bits = self.cx.lcx.ptr_bits();
        self.b.module().types.int(bits)
    }

    pub fn ptr_t(&mut self) -> TypeId {
        self.b.module().types.ptr(0)
    }

    pub fn usize_const(&mut self, v: u64) -> Value {
        let t = self.isize_t();
        self.b.const_int(t, v as i64)
    }

    /// An integer constant of type `t` (any width).
    pub fn int_const(&mut self, t: TypeId, v: u128) -> Value {
        let bits = match self.b.module().types.get(t) {
            epitome_ir::types::Type::Int(b) => *b,
            _ => 64,
        };
        if bits <= 64 {
            return self.b.const_int(t, v as u64 as i64);
        }
        let ap = epitome_ir::apint::ApInt::parse_decimal(bits, &v.to_string()).expect("a decimal integer");
        let c = epitome_ir::module::Constant { ty: t, kind: epitome_ir::module::ConstKind::Int(ap) };
        epitome_ir::module::Value::Const(self.b.module().consts.intern(c))
    }

    /// A float literal of type `f`: through `f64` for the narrower types,
    /// and rounded exactly for `f128`, which holds more digits.
    pub fn float_const(&mut self, tok: &str, f: FloatTy) -> R<Value> {
        let ft = match f {
            FloatTy::F16 => self.b.module().types.half(),
            FloatTy::F32 => self.b.module().types.float(),
            FloatTy::F64 => self.b.module().types.double(),
            FloatTy::F128 => {
                let body = ["f16", "f32", "f64", "f128"].iter().fold(tok, |t, s| t.strip_suffix(s).unwrap_or(t));
                let bits = super::quad::decimal_to_quad(body).ok_or_else(|| format!("bad float literal {tok}"))?;
                let ty = self.b.module().types.intern(epitome_ir::types::Type::Fp128);
                let c = epitome_ir::module::Constant { ty, kind: epitome_ir::module::ConstKind::Float(bits) };
                return Ok(Value::Const(self.b.module().consts.intern(c)));
            }
        };
        let x = parse_float(tok).ok_or_else(|| format!("bad float literal {tok}"))?;
        Ok(self.b.const_float(ft, x))
    }

    /// A stack slot for a value of type `t`, in the entry block.
    pub fn temp(&mut self, t: Ty) -> R<Place> {
        let l = self.layout(t)?;
        if !l.is_sized() {
            return Err(format!("a local of unsized type {}", self.cx.tcx.display(self.cx.r, t)));
        }
        let here = self.b.current_block();
        let i8t = self.i8t();
        let arr = self.b.module().types.array(l.size.max(1), i8t);
        self.b.at(self.entry);
        let ptr = self.ptr_t();
        let addr = self
            .b
            .push(Op::Alloca { inalloca: false, ty: arr, count: None, align: Align::new(l.align.max(1)), addrspace: None }, Some(ptr))
            .unwrap();
        if let Some(h) = here {
            self.b.at(h);
        }
        Ok(Place { addr, meta: None, ty: t })
    }

    /// `addr + off` bytes.
    pub fn offset(&mut self, addr: Value, off: u64) -> Value {
        if off == 0 {
            return addr;
        }
        let i8t = self.i8t();
        let o = self.usize_const(off);
        self.b.gep(i8t, addr, &[o])
    }

    pub fn load_imm(&mut self, t: TypeId, addr: Value, align: u64) -> Value {
        self.b.build_load(t, addr).align(align.max(1)).build()
    }

    pub fn store_imm(&mut self, v: Value, addr: Value, align: u64) {
        self.b.build_store(v, addr).align(align.max(1)).build()
    }

    pub fn memcpy(&mut self, dst: Value, src: Value, size: u64) {
        if size == 0 || dst == src {
            return;
        }
        let ptr = self.ptr_t();
        let i64t = self.b.module().types.int(64);
        let i1 = self.b.module().types.int(1);
        let void = self.b.module().types.void();
        let f = self.b.module().declare("llvm.memcpy.p0.p0.i64", void, &[ptr, ptr, i64t, i1], false);
        let n = self.b.const_int(i64t, size as i64);
        let no = self.b.const_int(i1, 0);
        self.b.call(f, &[dst, src, n, no]);
    }

    /// Whether the current block already ends in a terminator.
    pub fn terminated(&self) -> bool {
        let Some(bb) = self.b.current_block() else { return true };
        let f = self.b.function();
        f.blocks[bb.index()].insts.last().is_some_and(|i| f.insts[i.index()].op.is_terminator())
    }

    pub fn goto(&mut self, bb: BlockId) {
        if !self.terminated() {
            self.b.br(bb);
        }
    }

    /// A new block, named `name` and a number.
    pub fn new_block(&mut self, name: &str) -> BlockId {
        self.blocks += 1;
        let n = format!("{name}.{}", self.blocks);
        self.b.new_block(&n)
    }

    /// After a jump away: later code goes in a block nothing reaches.
    pub fn dead(&mut self) {
        let bb = self.new_block("dead");
        self.b.at(bb);
    }

    fn close_open_blocks(&mut self) {
        let layout = self.b.function().layout.clone();
        for bb in layout {
            if bb == self.entry {
                continue;
            }
            self.b.at(bb);
            if !self.terminated() {
                self.b.unreachable();
            }
        }
    }

    /// `llvm.trap`, then nothing more in this block.
    pub fn trap(&mut self) {
        let void = self.b.module().types.void();
        let trap = self.b.module().declare("llvm.trap", void, &[], false);
        self.b.call(trap, &[]);
        self.b.unreachable();
        self.dead();
    }

    /// Something not supported yet, used where only a panic would reach
    /// it (formatting a message, `dyn Debug`): it traps when reached, and
    /// the rest of the function is lowered. `--lower-stats` lists these.
    pub fn trap_here(&mut self, t: Ty, what: &str) -> Operand {
        if !self.traps.iter().any(|x| x == what) {
            self.traps.push(what.to_string());
        }
        self.trap();
        Operand { ty: t, v: Val::Zst }
    }

    /// Branches to a trap unless `ok` (an `i1`) holds.
    pub fn check(&mut self, ok: Value) {
        let good = self.new_block("ok");
        let bad = self.new_block("panic");
        self.b.cond_br(ok, good, bad);
        self.b.at(bad);
        self.trap();
        self.b.at(good);
    }

    /// An `i8` `bool` as an `i1`.
    pub fn truthy(&mut self, v: Value) -> Value {
        let t = self.b.type_of(v);
        let zero = self.b.const_int(t, 0);
        self.b.icmp(ICmpPred::Ne, v, zero)
    }

    /// An `i1` as an `i8` `bool`.
    pub fn bool_of(&mut self, c: Value) -> Value {
        let i8t = self.i8t();
        self.b.zext(c, i8t)
    }

    // ---- values and places ----

    pub fn load(&mut self, p: Place) -> R<Operand> {
        let v = match self.repr(p.ty)? {
            Repr::Zst => Val::Zst,
            Repr::Imm(t) => {
                let a = self.layout(p.ty)?.align;
                Val::Imm(self.load_imm(t, p.addr, a))
            }
            Repr::Mem => Val::Mem(p.addr),
        };
        Ok(Operand { ty: p.ty, v })
    }

    pub fn store(&mut self, o: Operand, dst: Place) -> R<()> {
        match o.v {
            Val::Zst => {}
            Val::Imm(v) => {
                let a = self.layout(dst.ty)?.align;
                self.store_imm(v, dst.addr, a);
            }
            Val::Mem(src) => {
                let size = self.layout(o.ty)?.size;
                self.memcpy(dst.addr, src, size);
            }
        }
        Ok(())
    }

    pub fn operand_of(&mut self, pv: Pv) -> R<Operand> {
        match pv {
            Pv::Val(o) => Ok(o),
            Pv::Place(p) => self.load(p),
        }
    }

    pub fn place_of(&mut self, pv: Pv) -> R<Place> {
        match pv {
            Pv::Place(p) => Ok(p),
            Pv::Val(o) => {
                if let Val::Mem(a) = o.v {
                    // A value in memory already has a place, which nothing
                    // else uses.
                    return Ok(Place { addr: a, meta: None, ty: o.ty });
                }
                let p = self.temp(o.ty)?;
                self.store(o, p)?;
                Ok(p)
            }
        }
    }

    /// An operand's value as an SSA value (it must be a scalar).
    pub fn imm(&mut self, o: Operand) -> R<Value> {
        match o.v {
            Val::Imm(v) => Ok(v),
            Val::Mem(a) => match self.repr(o.ty)? {
                Repr::Imm(t) => {
                    let al = self.layout(o.ty)?.align;
                    Ok(self.load_imm(t, a, al))
                }
                _ => Err(format!("a scalar was wanted, not {}", self.cx.tcx.display(self.cx.r, o.ty))),
            },
            // After a trap or a call that does not return: the code is
            // dead, and any value will do.
            Val::Zst => match self.repr(o.ty)? {
                Repr::Imm(t) => Ok(self.b.module().undef(t)),
                _ => Err("a scalar was wanted, not a zero-sized value".into()),
            },
        }
    }

    /// The address an operand of a type held in memory is at (spilling a
    /// scalar).
    pub fn mem(&mut self, o: Operand) -> R<Value> {
        match o.v {
            Val::Mem(a) => Ok(a),
            _ => Ok(self.place_of(Pv::Val(o))?.addr),
        }
    }

    /// The pointee of a reference, raw pointer or `Box` type.
    pub fn pointee(&self, t: Ty) -> Option<Ty> {
        let tcx = self.cx.tcx;
        match tcx.kind(t) {
            TyKind::Ref(_, x) | TyKind::Ptr(_, x) => Some(x),
            TyKind::Adt(d, l) if tcx.lang.get("owned_box") == Some(&d) => tcx.tys(l).first().copied(),
            _ => None,
        }
    }

    /// Whether a pointer to `t` carries metadata (a length or vtable).
    pub fn is_unsized(&self, t: Ty) -> R<bool> {
        let tcx = self.cx.tcx;
        Ok(match tcx.kind(t) {
            TyKind::Slice(_) | TyKind::Str | TyKind::Dyn(_) => true,
            TyKind::Adt(..) | TyKind::Tuple(_) => !self.layout(t)?.is_sized(),
            _ => false,
        })
    }

    /// `*o`: the place a pointer points at.
    pub fn deref(&mut self, o: Operand) -> R<Place> {
        let target = self.pointee(o.ty).ok_or_else(|| format!("dereference of {}", self.cx.tcx.display(self.cx.r, o.ty)))?;
        if self.cx.tcx.kind(o.ty) != TyKind::Ref(Mutbl::Not, target)
            && self.cx.tcx.kind(o.ty) != TyKind::Ref(Mutbl::Mut, target)
            && !matches!(self.cx.tcx.kind(o.ty), TyKind::Ptr(..))
        {
            // `Box<T>`: its first field holds the pointer.
            return Err("dereference of a `Box`".into());
        }
        if self.is_unsized(target)? {
            let a = self.mem(o)?;
            let ptr = self.ptr_t();
            let isz = self.isize_t();
            let ps = self.cx.lcx.target.ptr_size;
            let addr = self.load_imm(ptr, a, ps);
            let ma = self.offset(a, ps);
            let meta = if matches!(self.cx.tcx.kind(target), TyKind::Dyn(_)) { self.load_imm(ptr, ma, ps) } else { self.load_imm(isz, ma, ps) };
            return Ok(Place { addr, meta: Some(meta), ty: target });
        }
        let addr = self.imm(o)?;
        Ok(Place { addr, meta: None, ty: target })
    }

    /// `&p` (or `&raw p`) as a value of pointer type `pt`.
    pub fn addr_of(&mut self, p: Place, pt: Ty) -> R<Operand> {
        match p.meta {
            None => Ok(Operand { ty: pt, v: Val::Imm(p.addr) }),
            Some(meta) => {
                let pair = self.temp(pt)?;
                let ps = self.cx.lcx.target.ptr_size;
                self.store_imm(p.addr, pair.addr, ps);
                let ma = self.offset(pair.addr, ps);
                self.store_imm(meta, ma, ps);
                Ok(Operand { ty: pt, v: Val::Mem(pair.addr) })
            }
        }
    }

    /// A fat pointer of type `pt` made from an address and metadata.
    pub fn fat(&mut self, pt: Ty, addr: Value, meta: Value) -> R<Operand> {
        let target = self.pointee(pt).ok_or("not a pointer")?;
        self.addr_of(Place { addr, meta: Some(meta), ty: target }, pt)
    }

    /// The type of field `i` of variant `v` of `t`.
    pub fn field_ty(&mut self, t: Ty, v: u32, i: usize) -> R<Ty> {
        let tcx = self.cx.tcx;
        match tcx.kind(t) {
            TyKind::Adt(d, l) => {
                let adt = tcx.adts.get(&d).ok_or("not an ADT")?;
                let f = adt.variants.get(v as usize).and_then(|x| x.fields.get(i)).ok_or("no such field")?;
                let ft = crate::ty::collect::subst(tcx, f.ty, &tcx.all_params(d), &tcx.tys(l));
                let ft = crate::mono::reveal(tcx, &self.cx.typeck.hidden, ft);
                Ok(self.solver.normalize(ft))
            }
            TyKind::Tuple(l) => tcx.tys(l).get(i).copied().ok_or_else(|| "no such tuple field".into()),
            TyKind::Array(e, _) | TyKind::Slice(e) => Ok(e),
            TyKind::Pat(x, ..) if i == 0 => Ok(x),
            TyKind::Closure(c, _, l) => {
                let tys = self.cx.lcx.upvar_tys(c, l).ok_or("closure captures not known")?;
                tys.get(i).copied().ok_or_else(|| "no such capture".into())
            }
            _ => Err(format!("field of {}", tcx.display(self.cx.r, t))),
        }
    }

    /// Field `i` of variant `v` (0 for a struct) of the value at `p`.
    pub fn field(&mut self, p: Place, v: u32, i: usize) -> R<Place> {
        let l = self.layout(p.ty)?;
        let ft = self.field_ty(p.ty, v, i)?;
        let off = match &l.variants {
            crate::layout::Variants::Multiple { variants, .. } => variants.get(v as usize).map(|x| x.offset(i)).unwrap_or(0),
            _ => l.offset(i),
        };
        let addr = self.offset(p.addr, off);
        let meta = if self.is_unsized(ft)? { p.meta } else { None };
        Ok(Place { addr, meta, ty: ft })
    }

    /// The variant index of `name` in ADT `d`.
    pub fn variant_named(&self, d: DefId, name: &str) -> Option<u32> {
        self.cx.tcx.adts.get(&d)?.variants.iter().position(|v| v.name == name).map(|i| i as u32)
    }

    /// The index of the variant that is lang item `lang` (`Some`, `Break`).
    pub fn lang_variant(&self, t: Ty, lang: &str) -> R<u32> {
        let tcx = self.cx.tcx;
        let TyKind::Adt(d, _) = tcx.kind(t) else { return Err(format!("`{lang}` of a type that is not an enum")) };
        let vd = tcx.lang.get(lang).ok_or_else(|| format!("no lang item `{lang}`"))?;
        tcx.adts
            .get(&d)
            .and_then(|a| a.variants.iter().position(|v| v.def == Some(*vd)))
            .map(|i| i as u32)
            .ok_or_else(|| format!("no `{lang}` variant"))
    }

    // ---- expressions ----

    /// Expression `n` with its adjustments.
    pub fn expr(&mut self, n: NodeId) -> R<Pv> {
        let pv = self.expr_raw(n)?;
        match self.body.tables.adjustments.get(&n) {
            Some(adj) if !adj.is_empty() => {
                let adj = adj.clone();
                self.adjust(n, pv, &adj)
            }
            _ => Ok(pv),
        }
    }

    pub fn operand(&mut self, n: NodeId) -> R<Operand> {
        let pv = self.expr(n)?;
        self.operand_of(pv)
    }

    pub fn place(&mut self, n: NodeId) -> R<Place> {
        let pv = self.expr(n)?;
        self.place_of(pv)
    }

    fn pv_ty(pv: &Pv) -> Ty {
        match pv {
            Pv::Place(p) => p.ty,
            Pv::Val(o) => o.ty,
        }
    }

    fn adjust(&mut self, n: NodeId, mut pv: Pv, adj: &[Adjust]) -> R<Pv> {
        let tcx = self.cx.tcx;
        for (i, a) in adj.iter().enumerate() {
            match a {
                Adjust::Deref => {
                    let o = self.operand_of(pv)?;
                    pv = Pv::Place(self.deref(o)?);
                }
                Adjust::OverloadedDeref => {
                    let cur = Self::pv_ty(&pv);
                    let mutable = matches!(adj.get(i + 1), Some(Adjust::Borrow(Mutbl::Mut)));
                    let (lang, name, m) = if mutable { ("deref_mut", "deref_mut", Mutbl::Mut) } else { ("deref", "deref", Mutbl::Not) };
                    let tr = *tcx.lang.get(lang).ok_or("no `Deref`")?;
                    let method = self.trait_method(tr, name).ok_or("no `deref` method")?;
                    let p = self.place_of(pv)?;
                    let rt = tcx.mk(TyKind::Ref(m, cur));
                    let recv = self.addr_of(p, rt)?;
                    let r = self.call_def(method, vec![cur], vec![recv], None)?;
                    let o = self.operand_of(r)?;
                    pv = Pv::Place(self.deref(o)?);
                }
                Adjust::Borrow(m) => {
                    let p = self.place_of(pv)?;
                    let rt = tcx.mk(TyKind::Ref(*m, p.ty));
                    pv = Pv::Val(self.addr_of(p, rt)?);
                }
                Adjust::Unsize => {
                    let to = match self.body.tables.adjusted.get(&n) {
                        Some(&t) if i + 1 == adj.len() => self.sub(t),
                        _ => return Err("unsizing that is not the last adjustment".into()),
                    };
                    let o = self.operand_of(pv)?;
                    pv = Pv::Val(self.unsize(o, to)?);
                }
                Adjust::NeverToAny => {
                    let to = match self.body.tables.adjusted.get(&n) {
                        Some(&t) => self.sub(t),
                        None => tcx.mk(TyKind::Never),
                    };
                    pv = Pv::Val(Operand { ty: to, v: Val::Zst });
                }
                Adjust::ReifyFnPointer => {
                    let o = self.operand_of(pv)?;
                    let TyKind::FnDef(d, l) = tcx.kind(o.ty) else { return Err("reify of something not a function".into()) };
                    let to = match self.body.tables.adjusted.get(&n) {
                        Some(&t) => self.sub(t),
                        None => return Err("function pointer without a type".into()),
                    };
                    let f = self.fn_address(d, tcx.tys(l))?;
                    pv = Pv::Val(Operand { ty: to, v: Val::Imm(f) });
                }
                Adjust::ClosureFnPointer => return Err("closures are not supported yet".into()),
            }
        }
        Ok(pv)
    }

    /// `o` (a pointer to a sized type) made a pointer to the unsized `to`.
    fn unsize(&mut self, o: Operand, to: Ty) -> R<Operand> {
        let tcx = self.cx.tcx;
        let (Some(src), Some(dst)) = (self.pointee(o.ty), self.pointee(to)) else { return Err("unsizing of something not a pointer".into()) };
        if let TyKind::Dyn(_) = tcx.kind(dst) {
            return Ok(self.trap_here(to, "`dyn` (unsizing)"));
        }
        let n = self.unsized_len(src, dst).ok_or_else(|| format!("unsizing {} to {}", tcx.display(self.cx.r, o.ty), tcx.display(self.cx.r, to)))?;
        let addr = self.imm(o)?;
        let len = self.usize_const(n);
        self.fat(to, addr, len)
    }

    /// The length a pointer gets when `src` is unsized to `dst`: an
    /// array's, at the end of structs and tuples (`S<[T; 4]>` to `S<[T]>`).
    fn unsized_len(&mut self, src: Ty, dst: Ty) -> Option<u64> {
        let tcx = self.cx.tcx;
        match (tcx.kind(src), tcx.kind(dst)) {
            (TyKind::Array(_, c), TyKind::Slice(_)) => self.cx.lcx.eval_const(c, &Default::default()).map(|n| n as u64),
            (TyKind::Adt(d, _), TyKind::Adt(e, _)) if d == e => {
                let n = tcx.adts.get(&d)?.variants.first()?.fields.len();
                let a = self.field_ty(src, 0, n.checked_sub(1)?).ok()?;
                let b = self.field_ty(dst, 0, n - 1).ok()?;
                self.unsized_len(a, b)
            }
            (TyKind::Tuple(l), TyKind::Tuple(m)) => {
                let (a, b) = (*tcx.tys(l).last()?, *tcx.tys(m).last()?);
                self.unsized_len(a, b)
            }
            _ => None,
        }
    }

    /// The method `name` of trait `tr`.
    pub fn trait_method(&self, tr: DefId, name: &str) -> Option<DefId> {
        let r = self.cx.r;
        let m = r.defs[tr.0 as usize].module?;
        match r.modules[m.0 as usize].names.get(&(name, crate::resolve::Ns::Value)).map(|b| &b.res) {
            Some(Res::Def(d)) => Some(*d),
            _ => None,
        }
    }

    fn expr_raw(&mut self, n: NodeId) -> R<Pv> {
        let ast = self.body.ast;
        let tcx = self.cx.tcx;
        let kids = self.kids(n);
        match ast.kind(n) {
            Kind::Literal => Ok(Pv::Val(self.literal(n)?)),
            Kind::PathExpr => self.path_expr(n, kids[0]),
            Kind::Paren | Kind::AttrExpr => self.expr(*kids.last().unwrap()),
            Kind::Tuple | Kind::Array => {
                let t = self.ty(n)?;
                let p = self.temp(t)?;
                for (i, &e) in kids.iter().enumerate() {
                    let o = self.operand(e)?;
                    let fp = self.field(p, 0, i)?;
                    self.store(o, fp)?;
                }
                Ok(Pv::Val(Operand { ty: t, v: self.load(p)?.v }))
            }
            Kind::ArrayRepeat => {
                let t = self.ty(n)?;
                let TyKind::Array(_, c) = tcx.kind(t) else { return Err("repeat of something not an array".into()) };
                let count = self.cx.lcx.eval_const(c, &Default::default()).ok_or("array length not known")? as u64;
                let p = self.temp(t)?;
                let o = self.operand(kids[0])?;
                // A loop over the elements.
                let stride = match self.layout(t)?.fields {
                    Fields::Array { stride, .. } => stride,
                    _ => 0,
                };
                if count > 0 && stride > 0 {
                    let isz = self.isize_t();
                    let i = self.temp(tcx.mk(TyKind::Uint(UintTy::Usize)))?;
                    let zero = self.usize_const(0);
                    self.store_imm(zero, i.addr, 8);
                    let head = self.new_block("repeat");
                    let done = self.new_block("repeat.end");
                    self.b.br(head);
                    self.b.at(head);
                    let iv = self.load_imm(isz, i.addr, 8);
                    let s = self.usize_const(stride);
                    let off = self.b.mul(iv, s);
                    let i8t = self.i8t();
                    let ea = self.b.gep(i8t, p.addr, &[off]);
                    let et = self.field_ty(t, 0, 0)?;
                    self.store(o, Place { addr: ea, meta: None, ty: et })?;
                    let one = self.usize_const(1);
                    let next = self.b.add(iv, one);
                    self.store_imm(next, i.addr, 8);
                    let lim = self.usize_const(count);
                    let more = self.b.icmp(ICmpPred::Ult, next, lim);
                    self.b.cond_br(more, head, done);
                    self.b.at(done);
                }
                Ok(Pv::Val(Operand { ty: t, v: self.load(p)?.v }))
            }
            Kind::Block => self.block(n),
            Kind::ConstBlock => self.expr(kids[0]),
            Kind::Unary => self.unary(n, &kids),
            Kind::Borrow => {
                let t = self.ty(n)?;
                let e = *kids.last().unwrap();
                let p = self.place(e)?;
                Ok(Pv::Val(self.addr_of(p, t)?))
            }
            Kind::Cast => {
                let t = self.ty(n)?;
                let o = self.operand(kids[0])?;
                Ok(Pv::Val(self.cast(o, t)?))
            }
            Kind::Binary => self.binary(n, &kids),
            Kind::Range => {
                let t = self.ty(n)?;
                let p = self.temp(t)?;
                let TyKind::Adt(d, _) = tcx.kind(t) else { return Err("range of no range type".into()) };
                let op_at = kids.iter().position(|&c| ast.kind(c) == Kind::Op).unwrap_or(0);
                let start = kids[..op_at].first().copied();
                let end = kids.get(op_at + 1).copied();
                let adt = tcx.adts.get(&d).ok_or("range type not found")?;
                let names: Vec<&str> = adt.variants[0].fields.iter().map(|f| f.name).collect();
                for (name, e) in [("start", start), ("end", end)] {
                    if let Some(e) = e {
                        let i = names.iter().position(|&x| x == name).ok_or("range field not found")?;
                        let o = self.operand(e)?;
                        let fp = self.field(p, 0, i)?;
                        self.store(o, fp)?;
                    }
                }
                if let Some(i) = names.iter().position(|&x| x == "exhausted") {
                    let fp = self.field(p, 0, i)?;
                    let i8t = self.i8t();
                    let z = self.b.const_int(i8t, 0);
                    self.store_imm(z, fp.addr, 1);
                }
                Ok(Pv::Val(Operand { ty: t, v: self.load(p)?.v }))
            }
            Kind::Call => self.call_expr(n, &kids),
            Kind::MethodCall => {
                let (m, margs) = *self.body.tables.methods.get(&n).ok_or("method call without a method")?;
                let margs = self.sub_list(margs);
                let mut args = vec![self.operand(kids[0])?];
                for a in self.kids(kids[2]) {
                    args.push(self.operand(a)?);
                }
                self.call_def(m, margs, args, Some(n))
            }
            Kind::Field => {
                let base = self.place(kids[0])?;
                let name = ast.token(kids[1]).0;
                let i = match tcx.kind(base.ty) {
                    TyKind::Adt(d, _) => {
                        let adt = tcx.adts.get(&d).ok_or("not an ADT")?;
                        adt.variants[0].fields.iter().position(|f| f.name == name).ok_or("no such field")?
                    }
                    TyKind::Tuple(_) => name.parse::<usize>().map_err(|_| "bad tuple field")?,
                    _ => return Err(format!("field of {}", tcx.display(self.cx.r, base.ty))),
                };
                Ok(Pv::Place(self.field(base, 0, i)?))
            }
            Kind::Index => self.index(n, &kids),
            Kind::StructExpr => self.struct_expr(n, &kids),
            Kind::If => self.if_expr(n, &kids),
            Kind::Match => self.match_expr(n, &kids),
            Kind::Loop => {
                let label = kids.first().filter(|&&c| ast.kind(c) == Kind::Lifetime).map(|&c| ast.token(c).0);
                let t = self.ty(n)?;
                let result = if self.repr(t)? == Repr::Zst { None } else { Some(self.temp(t)?) };
                let head = self.new_block("loop");
                let exit = self.new_block("loop.end");
                self.goto(head);
                self.b.at(head);
                self.loops.push(Breakable { label, block: false, exit, cont: Some(head), result });
                let body = *kids.last().unwrap();
                self.expr(body)?;
                self.loops.pop();
                self.goto(head);
                self.b.at(exit);
                match result {
                    Some(p) => Ok(Pv::Place(p)),
                    None => Ok(Pv::Val(Operand { ty: t, v: Val::Zst })),
                }
            }
            Kind::While => {
                let label = kids.first().filter(|&&c| ast.kind(c) == Kind::Lifetime).map(|&c| ast.token(c).0);
                let rest: Vec<NodeId> = kids.iter().copied().filter(|&c| ast.kind(c) != Kind::Lifetime).collect();
                let head = self.new_block("while");
                let exit = self.new_block("while.end");
                self.goto(head);
                self.b.at(head);
                self.note(rest[0]);
                self.cond(rest[0], exit)?;
                self.loops.push(Breakable { label, block: false, exit, cont: Some(head), result: None });
                self.expr(rest[1])?;
                self.loops.pop();
                self.goto(head);
                self.b.at(exit);
                Ok(Pv::Val(self.unit()))
            }
            Kind::For => self.for_expr(n, &kids),
            Kind::Break => {
                let label = kids.first().filter(|&&c| ast.kind(c) == Kind::Lifetime).map(|&c| ast.token(c).0);
                let value = kids.iter().copied().find(|&c| ast.kind(c) != Kind::Lifetime);
                let target = match label {
                    Some(l) => self.loops.iter().rposition(|x| x.label == Some(l)),
                    None => self.loops.iter().rposition(|x| !x.block),
                }
                .ok_or("`break` outside a loop")?;
                if let Some(v) = value {
                    let o = self.operand(v)?;
                    if let Some(p) = self.loops[target].result {
                        self.store(o, p)?;
                    }
                }
                let exit = self.loops[target].exit;
                self.goto(exit);
                self.dead();
                Ok(Pv::Val(Operand { ty: tcx.mk(TyKind::Never), v: Val::Zst }))
            }
            Kind::Continue => {
                let label = kids.first().filter(|&&c| ast.kind(c) == Kind::Lifetime).map(|&c| ast.token(c).0);
                let target = match label {
                    Some(l) => self.loops.iter().rposition(|x| x.label == Some(l)),
                    None => self.loops.iter().rposition(|x| !x.block),
                }
                .ok_or("`continue` outside a loop")?;
                let cont = self.loops[target].cont.ok_or("`continue` to a block")?;
                self.goto(cont);
                self.dead();
                Ok(Pv::Val(Operand { ty: tcx.mk(TyKind::Never), v: Val::Zst }))
            }
            Kind::Return => {
                let v = match kids.first() {
                    Some(&e) => self.expr(e)?,
                    None => Pv::Val(self.unit()),
                };
                self.ret_value(v)?;
                self.dead();
                Ok(Pv::Val(Operand { ty: tcx.mk(TyKind::Never), v: Val::Zst }))
            }
            Kind::Try => self.try_expr(n, &kids),
            Kind::MacroCall => {
                let name = ast.children(kids[0]).iter().rev().find(|&&s| ast.kind(s) == Kind::Segment).map_or("", |&s| ast.token(s).0);
                match name {
                    "panic" | "core_panic" | "std_panic" | "unreachable_2015" | "unreachable_2021" | "const_panic" => {
                        self.trap();
                        Ok(Pv::Val(Operand { ty: tcx.mk(TyKind::Never), v: Val::Zst }))
                    }
                    "asm" => self.inline_asm(n, &kids),
                    // `core::fmt` comes later: where formatting is
                    // reached, the program traps.
                    "format_args" | "const_format_args" | "format_args_nl" => {
                        let t = self.ty(n)?;
                        Ok(Pv::Val(self.trap_here(t, "`format_args!`")))
                    }
                    _ => Err(format!("macro `{name}!` is not supported")),
                }
            }
            Kind::Closure => self.closure_expr(n),
            Kind::Let => Err("`let` outside a condition".into()),
            k => Err(format!("{k:?} is not supported yet")),
        }
    }

    /// Puts `v` where `return` puts the result, and returns.
    fn ret_value(&mut self, v: Pv) -> R<()> {
        if self.terminated() {
            return Ok(());
        }
        let o = self.operand_of(v)?;
        match self.ret {
            Ret::Void => self.b.ret_void(),
            Ret::Imm => {
                if matches!(o.v, Val::Zst) {
                    // A value that never exists (after a diverging call).
                    self.b.unreachable();
                } else {
                    let v = self.imm(o)?;
                    self.b.ret(v);
                }
            }
            Ret::Sret(p) => {
                let rt = self.ret_ty;
                self.store(Operand { ty: rt, v: o.v }, Place { addr: p, meta: None, ty: rt })?;
                self.b.ret_void();
            }
        }
        Ok(())
    }

    fn block(&mut self, n: NodeId) -> R<Pv> {
        let ast = self.body.ast;
        let kids = self.kids(n);
        if kids.iter().any(|&c| ast.kind(c) == Kind::Keyword && matches!(ast.token(c).0, "async" | "gen")) {
            return Err("`async` and `gen` blocks are not supported yet".into());
        }
        if kids.iter().any(|&c| ast.kind(c) == Kind::Keyword && ast.token(c).0 == "try") {
            return self.try_block(n, &kids);
        }
        let label = kids.first().filter(|&&c| ast.kind(c) == Kind::Lifetime).map(|&c| ast.token(c).0);
        let stmts: Vec<NodeId> = kids.iter().copied().filter(|&c| !matches!(ast.kind(c), Kind::Lifetime | Kind::Keyword | Kind::InnerAttribute)).collect();
        let labelled = match label {
            Some(l) => {
                let t = self.ty(n)?;
                let result = if self.repr(t)? == Repr::Zst { None } else { Some(self.temp(t)?) };
                let exit = self.new_block("block.end");
                self.loops.push(Breakable { label: Some(l), block: true, exit, cont: None, result });
                Some((t, exit, result))
            }
            None => None,
        };
        let mut value = Pv::Val(self.unit());
        let last = stmts.len().checked_sub(1);
        for (i, &s) in stmts.iter().enumerate() {
            match ast.kind(s) {
                Kind::LetStmt => {
                    self.note(s);
                    self.let_stmt(s)?;
                }
                Kind::ExprStmt => {
                    let Some(&e) = self.kids(s).last() else { continue };
                    self.note(s);
                    self.expr(e)?;
                }
                Kind::EmptyStmt => {}
                k if super::pat::is_expr(k) => {
                    self.note(s);
                    let v = self.expr(s)?;
                    if Some(i) == last {
                        value = v;
                    }
                }
                // Items in a block are lowered as items of their own.
                _ => {}
            }
        }
        if let Some((t, exit, result)) = labelled {
            if let Some(p) = result {
                let o = self.operand_of(value)?;
                if !self.terminated() {
                    self.store(o, p)?;
                }
            }
            self.loops.pop();
            self.goto(exit);
            self.b.at(exit);
            return Ok(match result {
                Some(p) => Pv::Place(p),
                None => Pv::Val(Operand { ty: t, v: Val::Zst }),
            });
        }
        // A block whose statements diverge has type `!`: its "value" is
        // never used.
        Ok(value)
    }

    /// `try { stmts; e }`: `Try::from_output(e)`, and each `?` in it that
    /// fails gives `FromResidual::from_residual` of its residual as the
    /// block's value instead of returning.
    fn try_block(&mut self, n: NodeId, kids: &[NodeId]) -> R<Pv> {
        let ast = self.body.ast;
        let tcx = self.cx.tcx;
        let t = self.ty(n)?;
        let result = self.temp(t)?;
        let exit = self.new_block("try.end");
        self.tries.push((exit, result));
        let stmts: Vec<NodeId> = kids.iter().copied().filter(|&c| !matches!(ast.kind(c), Kind::Lifetime | Kind::Keyword | Kind::InnerAttribute)).collect();
        let mut value = Pv::Val(self.unit());
        let last = stmts.len().checked_sub(1);
        for (i, &s) in stmts.iter().enumerate() {
            match ast.kind(s) {
                Kind::LetStmt => {
                    self.note(s);
                    self.let_stmt(s)?;
                }
                Kind::ExprStmt => {
                    let Some(&e) = self.kids(s).last() else { continue };
                    self.note(s);
                    self.expr(e)?;
                }
                k if super::pat::is_expr(k) => {
                    self.note(s);
                    let v = self.expr(s)?;
                    if Some(i) == last {
                        value = v;
                    }
                }
                _ => {}
            }
        }
        self.tries.pop();
        if !self.terminated() {
            let tr = *tcx.lang.get("Try").ok_or("no `Try`")?;
            let from_output = self.trait_method(tr, "from_output").ok_or("no `from_output`")?;
            let v = self.operand_of(value)?;
            let r = self.call_def(from_output, vec![t], vec![v], None)?;
            let o = self.operand_of(r)?;
            self.store(Operand { ty: t, v: o.v }, result)?;
        }
        self.goto(exit);
        self.b.at(exit);
        Ok(Pv::Place(result))
    }

    fn let_stmt(&mut self, s: NodeId) -> R<()> {
        let ast = self.body.ast;
        let sk: Vec<NodeId> = self.kids(s).into_iter().filter(|&c| ast.kind(c) != Kind::Keyword).collect();
        let pat = sk[0];
        let mut rest = &sk[1..];
        if rest.first().is_some_and(|&t| super::pat::is_type(ast.kind(t))) {
            rest = &rest[1..];
        }
        let Some(&init) = rest.first() else {
            // Declared, assigned later.
            return self.declare_bindings(pat);
        };
        if let Some(&els) = rest.get(1) {
            let p = self.place(init)?;
            let fail = self.new_block("let.else");
            let ok = self.new_block("let.ok");
            self.pat_match(pat, p, Some(fail))?;
            self.goto(ok);
            self.b.at(fail);
            self.expr(els)?;
            if !self.terminated() {
                self.b.unreachable();
            }
            self.b.at(ok);
            return Ok(());
        }
        if self.simple_binding(pat) {
            let o = self.operand(init)?;
            let t = self.ty(pat)?;
            let slot = self.temp(t)?;
            self.store(Operand { ty: t, v: o.v }, slot)?;
            self.locals.insert((self.body.k, self.body.f, pat), slot);
            return Ok(());
        }
        let p = self.place(init)?;
        self.bind_irrefutable(pat, p)
    }

    /// Whether `pat` is `x` or `mut x`, binding by value.
    fn simple_binding(&self, pat: NodeId) -> bool {
        let ast = self.body.ast;
        ast.kind(pat) == Kind::IdentPat
            && self.res(pat).is_none()
            && self.body.tables.bindings.get(&pat) == Some(&crate::typeck::BindingMode::Value)
            && !self.body.tables.pat_derefs.contains_key(&pat)
            && ast.children(pat).iter().all(|&c| matches!(ast.kind(c), Kind::Keyword | Kind::Name))
    }

    /// Slots for the bindings of a pattern with no value yet.
    fn declare_bindings(&mut self, pat: NodeId) -> R<()> {
        let ast = self.body.ast;
        if ast.kind(pat) == Kind::IdentPat && self.res(pat).is_none() {
            let t = self.ty(pat)?;
            let slot = self.temp(t)?;
            self.locals.insert((self.body.k, self.body.f, pat), slot);
            return Ok(());
        }
        for c in self.kids(pat) {
            if super::pat::is_pattern(ast.kind(c)) {
                self.declare_bindings(c)?;
            }
        }
        Ok(())
    }

    pub fn bind_irrefutable(&mut self, pat: NodeId, p: Place) -> R<()> {
        if self.simple_binding(pat) {
            let t = self.ty(pat)?;
            if t == p.ty && p.meta.is_none() {
                // A slot of its own already (a parameter's).
                self.locals.insert((self.body.k, self.body.f, pat), p);
                return Ok(());
            }
        }
        self.pat_match(pat, p, None)
    }

    // ---- literals and paths ----

    fn literal(&mut self, n: NodeId) -> R<Operand> {
        let t = self.ty(n)?;
        let tok = self.body.ast.token(n).0;
        let tcx = self.cx.tcx;
        match tcx.kind(t) {
            TyKind::Bool => {
                let i8t = self.i8t();
                let v = self.b.const_int(i8t, (tok == "true") as i64);
                Ok(Operand { ty: t, v: Val::Imm(v) })
            }
            TyKind::Char => {
                let c = parse_char(tok).ok_or_else(|| format!("bad character literal {tok}"))?;
                let i32t = self.b.module().types.int(32);
                let v = self.b.const_int(i32t, c as i64);
                Ok(Operand { ty: t, v: Val::Imm(v) })
            }
            TyKind::Int(_) | TyKind::Uint(_) => {
                let v = if tok.starts_with("b'") {
                    parse_char(&tok[1..]).ok_or_else(|| format!("bad byte literal {tok}"))? as u128
                } else {
                    parse_int(tok).ok_or_else(|| format!("bad integer literal {tok}"))?
                };
                let Repr::Imm(it) = self.repr(t)? else { return Err("integer not a scalar".into()) };
                let v = self.int_const(it, v);
                Ok(Operand { ty: t, v: Val::Imm(v) })
            }
            TyKind::Float(f) => {
                let v = self.float_const(tok, f)?;
                Ok(Operand { ty: t, v: Val::Imm(v) })
            }
            TyKind::Ref(_, inner) => {
                let bytes = parse_str(tok).ok_or_else(|| format!("bad string literal {tok}"))?;
                let len = bytes.len() as u64;
                let init = self.b.module().const_bytes(&bytes);
                let name = format!("{}.str.{}", self.fname, self.strings);
                self.strings += 1;
                let g = epitome_ir::module::Global::builder(&name).init(init).constant().linkage(Linkage::Private).align(1).build(self.b.module());
                let addr = self.b.module().global_value(g);
                match tcx.kind(inner) {
                    TyKind::Str => {
                        let l = self.usize_const(len);
                        Ok(self.fat(t, addr, l)?)
                    }
                    TyKind::Array(..) => Ok(Operand { ty: t, v: Val::Imm(addr) }),
                    _ => Err("string literal of an unexpected type".into()),
                }
            }
            _ => Err(format!("literal {tok} of type {}", tcx.display(self.cx.r, t))),
        }
    }

    fn path_expr(&mut self, n: NodeId, path: NodeId) -> R<Pv> {
        let tcx = self.cx.tcx;
        if let Some(&(c, cargs)) = self.body.tables.consts.get(&n) {
            let cargs = self.sub_list(cargs);
            return self.constant(n, c, cargs);
        }
        match self.res(path) {
            Some(Res::Local(p)) => {
                let key = (self.body.k, self.body.f, *p);
                return self.local_place(key).map(Pv::Place);
            }
            Some(Res::Generic(gp)) => {
                // A const parameter: its argument's value.
                let pid = *tcx.param_at.get(&(self.body.k, self.body.f, *gp)).ok_or("generic parameter not found")?;
                let i = self.body.params.iter().position(|&x| x == pid).ok_or("const parameter not in scope")?;
                let TyKind::ConstArg(c) = tcx.kind(self.body.args[i]) else { return Err("const parameter without a value".into()) };
                let v = self.cx.lcx.eval_const(c, &Default::default()).ok_or_else(|| match tcx.const_of(c) {
                    crate::ty::Const::Expr(k, f, e) => format!("const argument the evaluator cannot work out: {}", self.cx.crates[k as usize].files[f as usize].ast.text(e)),
                    c => format!("const argument the evaluator cannot work out: {c:?}"),
                })?;
                let t = self.ty(n)?;
                let Repr::Imm(it) = self.repr(t)? else { return Err("const parameter of a type in memory".into()) };
                let v = self.int_const(it, v as u128);
                return Ok(Pv::Val(Operand { ty: t, v: Val::Imm(v) }));
            }
            Some(Res::Def(d)) if matches!(self.cx.r.defs[d.0 as usize].kind, DefKind::Static) => {
                let g = *self.cx.statics.get(d).ok_or("static not defined")?;
                let addr = self.b.module().global_value(g);
                let t = self.ty(n)?;
                return Ok(Pv::Place(Place { addr, meta: None, ty: t }));
            }
            Some(Res::Def(d)) if matches!(self.cx.r.defs[d.0 as usize].kind, DefKind::ForeignStatic) => {
                let name = self.cx.symbol(*d, tcx.list(Vec::new()));
                let ptr = self.ptr_t();
                let _ = ptr;
                let t = self.ty(n)?;
                let g = match self.b.module().find_global(&name) {
                    Some(g) => g,
                    None => {
                        let i8t = self.i8t();
                        let mut g = epitome_ir::module::Global::variable(&name, i8t, None, false);
                        g.props.linkage = Linkage::External;
                        self.b.module().add_global(g)
                    }
                };
                let addr = self.b.module().global_value(g);
                return Ok(Pv::Place(Place { addr, meta: None, ty: t }));
            }
            _ => {}
        }
        let t = self.ty(n)?;
        match tcx.kind(t) {
            // A function item, or a tuple constructor: no value.
            TyKind::FnDef(..) => Ok(Pv::Val(Operand { ty: t, v: Val::Zst })),
            TyKind::Adt(d, _) => {
                // A unit struct or variant.
                let v = self.variant_of_path(path, d).unwrap_or(0);
                let p = self.temp(t)?;
                self.set_variant(p, v)?;
                Ok(Pv::Val(Operand { ty: t, v: self.load(p)?.v }))
            }
            _ if self.repr(t)? == Repr::Zst => Ok(Pv::Val(Operand { ty: t, v: Val::Zst })),
            _ => Err(format!("path `{}` is not supported", self.text(n))),
        }
    }

    /// The place of a local: its slot, or in a closure, what was captured.
    pub fn local_place(&mut self, key: (u32, u32, NodeId)) -> R<Place> {
        if let Some(&p) = self.locals.get(&key) {
            return Ok(p);
        }
        let (i, by_value) = *self.upvars.get(&key).ok_or("local has no slot")?;
        let env = self.env.ok_or("captured local outside a closure")?;
        let fp = self.field(env, 0, i)?;
        if by_value {
            return Ok(fp);
        }
        let o = self.load(fp)?;
        self.deref(o)
    }

    /// A closure's value: its captures.
    fn closure_expr(&mut self, n: NodeId) -> R<Pv> {
        let t = self.ty(n)?;
        let TyKind::Closure(c, _, _) = self.cx.tcx.kind(t) else { return Err("closure of no closure type".into()) };
        let caps = self.cx.typeck.captures.get(&c).cloned().unwrap_or_default();
        let p = self.temp(t)?;
        for (i, cap) in caps.iter().enumerate() {
            let src = self.local_place((self.body.k, self.body.f, cap.local))?;
            let fp = self.field(p, 0, i)?;
            let o = if cap.by_value { self.load(src)? } else { self.addr_of(src, fp.ty)? };
            self.store(Operand { ty: fp.ty, v: o.v }, fp)?;
        }
        Ok(Pv::Val(Operand { ty: t, v: self.load(p)?.v }))
    }

    /// The variant a path names, in enum `d`.
    pub fn variant_of_path(&self, path: NodeId, d: DefId) -> Option<u32> {
        let tcx = self.cx.tcx;
        let adt = tcx.adts.get(&d)?;
        if adt.kind != AdtKind::Enum {
            return Some(0);
        }
        match self.res(path) {
            Some(Res::Def(v)) if self.cx.r.defs[v.0 as usize].kind == DefKind::Variant => {
                adt.variants.iter().position(|x| x.def == Some(*v)).map(|i| i as u32)
            }
            _ => {
                let ast = self.body.ast;
                let name = if ast.kind(path) == Kind::Path {
                    ast.children(path).iter().rev().find(|&&s| ast.kind(s) == Kind::Segment).map(|&s| ast.token(s).0)
                } else {
                    Some(ast.token(path).0)
                }?;
                self.variant_named(d, name)
            }
        }
    }

    /// The value of constant `c` at `cargs`: an integer the evaluator
    /// works out, or its initialiser lowered here.
    pub fn constant(&mut self, n: NodeId, c: DefId, cargs: Vec<Ty>) -> R<Pv> {
        let tcx = self.cx.tcx;
        let item = match crate::mono::instance(self.cx.r, self.cx.crates, tcx, &mut self.solver, c, cargs) {
            Resolved::Item(Item::Const(d, l)) => (d, l),
            _ => return Err(format!("constant `{}` not resolved", self.text(n))),
        };
        let t = self.ty(n)?;
        if let Repr::Imm(it) = self.repr(t)?
            && tcx.tys(item.1).is_empty()
            && matches!(tcx.kind(t), TyKind::Int(_) | TyKind::Uint(_) | TyKind::Bool | TyKind::Char)
            && let Some(v) = self.cx.lcx.eval_item(item.0).and_then(|v| v.int())
        {
            let v = self.int_const(it, v as u128);
            return Ok(Pv::Val(Operand { ty: t, v: Val::Imm(v) }));
        }
        // Its initialiser, lowered in its own body.
        let (k, f, ast, node) = self.cx.def_ast(item.0);
        let init = ast.children(node).iter().rev().copied().find(|&x| super::pat::is_expr(ast.kind(x))).ok_or("constant without a value")?;
        let saved = self.body.clone();
        self.body = Body {
            k,
            f,
            ast,
            types: &self.cx.typeck.types[k as usize][f as usize],
            tables: &self.cx.typeck.tables[k as usize][f as usize],
            params: tcx.all_params(item.0),
            args: tcx.tys(item.1),
        };
        let got = self.operand(init);
        self.body = saved;
        let o = got?;
        Ok(Pv::Val(Operand { ty: t, v: o.v }))
    }

    // ---- operators ----

    fn unary(&mut self, n: NodeId, kids: &[NodeId]) -> R<Pv> {
        let tcx = self.cx.tcx;
        let ast = self.body.ast;
        if ast.kind(kids[0]) == Kind::Keyword {
            return Err("`box` is not supported".into());
        }
        let op = ast.token(kids[0]).0;
        if op == "*" {
            if let Some(&(m, margs)) = self.body.tables.methods.get(&n) {
                // `*x` through `Deref::deref(&x)`.
                let margs = self.sub_list(margs);
                let p = self.place(kids[1])?;
                let rt = tcx.mk(TyKind::Ref(Mutbl::Not, p.ty));
                let recv = self.addr_of(p, rt)?;
                let r = self.call_def(m, margs, vec![recv], None)?;
                let o = self.operand_of(r)?;
                return Ok(Pv::Place(self.deref(o)?));
            }
            let o = self.operand(kids[1])?;
            return Ok(Pv::Place(self.deref(o)?));
        }
        let o = self.operand(kids[1])?;
        if !is_prim(tcx, o.ty)
            && let Some(&(m, margs)) = self.body.tables.methods.get(&n)
        {
            let margs = self.sub_list(margs);
            return self.call_def(m, margs, vec![o], None);
        }
        let v = self.imm(o)?;
        let r = match (op, tcx.kind(o.ty)) {
            ("-", TyKind::Float(_)) => self.b.fneg(v),
            ("-", _) => {
                let t = self.b.type_of(v);
                let z = self.b.const_int(t, 0);
                self.b.sub(z, v)
            }
            ("!", TyKind::Bool) => {
                let t = self.b.type_of(v);
                let one = self.b.const_int(t, 1);
                self.b.xor(v, one)
            }
            ("!", _) => {
                let t = self.b.type_of(v);
                let ones = self.b.const_int(t, -1);
                self.b.xor(v, ones)
            }
            _ => return Err(format!("unary `{op}`")),
        };
        Ok(Pv::Val(Operand { ty: o.ty, v: Val::Imm(r) }))
    }

    fn binary(&mut self, n: NodeId, kids: &[NodeId]) -> R<Pv> {
        let tcx = self.cx.tcx;
        let op = self.body.ast.text(kids[1]);
        let bool_t = tcx.mk(TyKind::Bool);
        match op.as_str() {
            "&&" | "||" => {
                let result = self.temp(bool_t)?;
                let a = self.operand(kids[0])?;
                let av = self.imm(a)?;
                self.store_imm(av, result.addr, 1);
                let rhs = self.new_block("rhs");
                let done = self.new_block("logic.end");
                let c = self.truthy(av);
                if op == "&&" {
                    self.b.cond_br(c, rhs, done);
                } else {
                    self.b.cond_br(c, done, rhs);
                }
                self.b.at(rhs);
                let bo = self.operand(kids[2])?;
                if !self.terminated() {
                    let bv = self.imm(bo)?;
                    self.store_imm(bv, result.addr, 1);
                }
                self.goto(done);
                self.b.at(done);
                return Ok(Pv::Place(result));
            }
            "=" => {
                // The right first, then the place.
                let o = self.operand(kids[2])?;
                let o = if let Val::Mem(_) = o.v {
                    // A copy, in case the place overlaps what it reads.
                    let t = self.temp(o.ty)?;
                    self.store(o, t)?;
                    self.load(t)?
                } else {
                    o
                };
                self.assign(kids[0], o)?;
                return Ok(Pv::Val(self.unit()));
            }
            _ => {}
        }
        let assign = op.ends_with('=') && !matches!(op.as_str(), "==" | "!=" | "<=" | ">=");
        let base = if assign { op.trim_end_matches('=') } else { op.as_str() };
        if assign {
            let rhs = self.operand(kids[2])?;
            let p = self.place(kids[0])?;
            if let Some(&(m, margs)) = self.body.tables.methods.get(&n)
                && !(is_prim(tcx, p.ty) && is_prim(tcx, rhs.ty))
            {
                let margs = self.sub_list(margs);
                let rt = tcx.mk(TyKind::Ref(Mutbl::Mut, p.ty));
                let recv = self.addr_of(p, rt)?;
                self.call_def(m, margs, vec![recv, rhs], None)?;
                return Ok(Pv::Val(self.unit()));
            }
            let a = self.load(p)?;
            let r = self.arith(base, a, rhs)?;
            self.store(r, p)?;
            return Ok(Pv::Val(self.unit()));
        }
        let compare = matches!(base, "==" | "!=" | "<" | ">" | "<=" | ">=");
        let a = self.operand(kids[0])?;
        let b = self.operand(kids[2])?;
        if let Some(&(m, margs)) = self.body.tables.methods.get(&n)
            && !(is_prim(tcx, a.ty) && is_prim(tcx, b.ty))
        {
            let margs = self.sub_list(margs);
            if compare {
                // `PartialEq::eq(&a, &b)` and its relatives.
                let pa = self.place_of(Pv::Val(a))?;
                let pb = self.place_of(Pv::Val(b))?;
                let ra = tcx.mk(TyKind::Ref(Mutbl::Not, a.ty));
                let rb = tcx.mk(TyKind::Ref(Mutbl::Not, b.ty));
                let ra = self.addr_of(pa, ra)?;
                let rb = self.addr_of(pb, rb)?;
                return self.call_def(m, margs, vec![ra, rb], None);
            }
            return self.call_def(m, margs, vec![a, b], None);
        }
        if compare {
            let c = self.compare(base, a, b)?;
            let v = self.bool_of(c);
            return Ok(Pv::Val(Operand { ty: bool_t, v: Val::Imm(v) }));
        }
        Ok(Pv::Val(self.arith(base, a, b)?))
    }

    /// `lhs = o`, where `lhs` may destructure: `(a, b) = ...`, `[x, _] =
    /// ...`, `_ = ...`.
    fn assign(&mut self, lhs: NodeId, o: Operand) -> R<()> {
        let ast = self.body.ast;
        match ast.kind(lhs) {
            Kind::Underscore => Ok(()),
            Kind::Paren => {
                let k = self.kids(lhs);
                self.assign(k[0], o)
            }
            Kind::Tuple | Kind::Array => {
                let src = self.place_of(Pv::Val(o))?;
                let elems = self.kids(lhs);
                let n = match self.cx.tcx.kind(src.ty) {
                    TyKind::Tuple(l) => self.cx.tcx.tys(l).len(),
                    TyKind::Array(_, c) => self.cx.lcx.eval_const(c, &Default::default()).ok_or("array length not known")? as usize,
                    _ => return Err("destructuring something not a tuple or array".into()),
                };
                let rest = elems.iter().position(|&e| ast.kind(e) == Kind::Range && self.kids(e).len() == 1);
                for (i, &e) in elems.iter().enumerate() {
                    if Some(i) == rest {
                        continue;
                    }
                    let fi = match rest {
                        Some(r) if i > r => n - (elems.len() - i),
                        _ => i,
                    };
                    let fp = self.field(src, 0, fi)?;
                    let v = self.load(fp)?;
                    self.assign(e, v)?;
                }
                Ok(())
            }
            Kind::StructExpr | Kind::Call => Err("destructuring assignment of a struct is not supported yet".into()),
            _ => {
                let p = self.place(lhs)?;
                self.store(Operand { ty: p.ty, v: o.v }, p)
            }
        }
    }

    pub fn signed(&self, t: Ty) -> bool {
        matches!(self.cx.tcx.kind(t), TyKind::Int(_))
    }

    /// `a op b` on numbers (or `bool`s), wrapping.
    pub fn arith(&mut self, op: &str, a: Operand, b: Operand) -> R<Operand> {
        let tcx = self.cx.tcx;
        let x = self.imm(a)?;
        let mut y = self.imm(b)?;
        let float = matches!(tcx.kind(a.ty), TyKind::Float(_));
        let signed = self.signed(a.ty);
        if matches!(op, "<<" | ">>") {
            // The shift amount, at the left's width, masked as a release
            // build does.
            let (tx, ty) = (self.b.type_of(x), self.b.type_of(y));
            let bits = |b: &mut FunctionBuilder, t: TypeId| match b.module().types.get(t) {
                epitome_ir::types::Type::Int(n) => *n,
                _ => 64,
            };
            let (bx, by) = (bits(&mut self.b, tx), bits(&mut self.b, ty));
            if by > bx {
                y = self.b.trunc(y, tx);
            } else if by < bx {
                y = self.b.zext(y, tx);
            }
            let mask = self.b.const_int(tx, bx as i64 - 1);
            y = self.b.and(y, mask);
        }
        let v = match op {
            "+" if float => self.b.fadd(x, y),
            "-" if float => self.b.fsub(x, y),
            "*" if float => self.b.fmul(x, y),
            "/" if float => self.b.fdiv(x, y),
            "%" if float => self.b.binary(BinOp::FRem, x, y),
            "+" => self.b.add(x, y),
            "-" => self.b.sub(x, y),
            "*" => self.b.mul(x, y),
            "/" | "%" => {
                let t = self.b.type_of(y);
                let z = self.b.const_int(t, 0);
                let ok = self.b.icmp(ICmpPred::Ne, y, z);
                self.check(ok);
                match (op, signed) {
                    ("/", true) => self.b.sdiv(x, y),
                    ("/", false) => self.b.udiv(x, y),
                    (_, true) => self.b.srem(x, y),
                    _ => self.b.urem(x, y),
                }
            }
            "&" => self.b.and(x, y),
            "|" => self.b.or(x, y),
            "^" => self.b.xor(x, y),
            "<<" => self.b.shl(x, y),
            ">>" if signed => self.b.ashr(x, y),
            ">>" => self.b.lshr(x, y),
            _ => return Err(format!("operator `{op}`")),
        };
        Ok(Operand { ty: a.ty, v: Val::Imm(v) })
    }

    /// `a op b` for a comparison, as an `i1`.
    pub fn compare(&mut self, op: &str, a: Operand, b: Operand) -> R<Value> {
        let tcx = self.cx.tcx;
        let x = self.imm(a)?;
        let y = self.imm(b)?;
        if matches!(tcx.kind(a.ty), TyKind::Float(_)) {
            let p = match op {
                "==" => FCmpPred::Oeq,
                "!=" => FCmpPred::Une,
                "<" => FCmpPred::Olt,
                ">" => FCmpPred::Ogt,
                "<=" => FCmpPred::Ole,
                _ => FCmpPred::Oge,
            };
            return Ok(self.b.fcmp(p, x, y));
        }
        let signed = self.signed(a.ty);
        let p = match (op, signed) {
            ("==", _) => ICmpPred::Eq,
            ("!=", _) => ICmpPred::Ne,
            ("<", true) => ICmpPred::Slt,
            ("<", false) => ICmpPred::Ult,
            (">", true) => ICmpPred::Sgt,
            (">", false) => ICmpPred::Ugt,
            ("<=", true) => ICmpPred::Sle,
            ("<=", false) => ICmpPred::Ule,
            (_, true) => ICmpPred::Sge,
            _ => ICmpPred::Uge,
        };
        Ok(self.b.icmp(p, x, y))
    }

    /// `o as to`.
    pub fn cast(&mut self, o: Operand, to: Ty) -> R<Operand> {
        let tcx = self.cx.tcx;
        if o.ty == to {
            return Ok(o);
        }
        let from_k = tcx.kind(o.ty);
        let to_k = tcx.kind(to);
        // A field-less enum: its discriminant.
        if let TyKind::Adt(d, _) = from_k
            && tcx.adts.get(&d).is_some_and(|a| a.kind == AdtKind::Enum)
        {
            let p = self.place_of(Pv::Val(o))?;
            let dv = self.discriminant(p)?;
            let i64t = self.b.module().types.int(64);
            let dop = Operand { ty: tcx.mk(TyKind::Int(IntTy::I64)), v: Val::Imm(self.int_to(dv, i64t, true)) };
            return self.cast(dop, to);
        }
        if let TyKind::FnDef(d, l) = from_k {
            let f = self.fn_address(d, tcx.tys(l))?;
            let fo = Operand { ty: tcx.mk(TyKind::Ptr(Mutbl::Not, tcx.unit())), v: Val::Imm(f) };
            return self.cast(fo, to);
        }
        let from_r = self.repr(o.ty)?;
        let to_r = self.repr(to)?;
        match (from_r, to_r) {
            (Repr::Imm(ft), Repr::Imm(tt)) => {
                let v = self.imm(o)?;
                let ptr = self.ptr_t();
                let is_float = |k: TyKind| matches!(k, TyKind::Float(_));
                let r = if ft == tt && !is_float(from_k) && !is_float(to_k) || ft == ptr && tt == ptr {
                    v
                } else if ft == ptr {
                    self.b.cast(CastOp::PtrToInt, v, tt)
                } else if tt == ptr {
                    self.b.cast(CastOp::IntToPtr, v, tt)
                } else if is_float(from_k) && is_float(to_k) {
                    let (fs, ts) = (self.layout(o.ty)?.size, self.layout(to)?.size);
                    if fs < ts {
                        self.b.cast(CastOp::FPExt, v, tt)
                    } else if fs > ts {
                        self.b.cast(CastOp::FPTrunc, v, tt)
                    } else {
                        v
                    }
                } else if is_float(from_k) && from_k == TyKind::Float(FloatTy::F128) {
                    self.f128_to_int(v, tt, self.signed(to))
                } else if is_float(to_k) && to_k == TyKind::Float(FloatTy::F128) {
                    // Through 128 bits, which the runtime converts.
                    let i128t = self.b.module().types.int(128);
                    let signed = self.signed(o.ty) || from_k == TyKind::Bool;
                    let w = self.int_to(v, i128t, self.signed(o.ty));
                    let op = if signed { CastOp::SIToFP } else { CastOp::UIToFP };
                    self.b.cast(op, w, tt)
                } else if is_float(from_k) {
                    // Saturating, as Rust's `as` is: out-of-range values
                    // give the nearest the integer holds, NaN gives 0.
                    let fbits = self.layout(o.ty)?.size * 8;
                    let ibits = self.layout(to)?.size * 8;
                    let name = format!("llvm.{}.sat.i{ibits}.f{fbits}", if self.signed(to) { "fptosi" } else { "fptoui" });
                    let g = self.b.module().declare(&name, tt, &[ft], false);
                    self.b.call(g, &[v]).unwrap()
                } else if is_float(to_k) {
                    let op = if self.signed(o.ty) { CastOp::SIToFP } else { CastOp::UIToFP };
                    self.b.cast(op, v, tt)
                } else {
                    let signed = self.signed(o.ty);
                    self.int_to(v, tt, signed)
                };
                Ok(Operand { ty: to, v: Val::Imm(r) })
            }
            (Repr::Mem, Repr::Imm(tt)) => {
                // A fat pointer made thin (or an address): its first word.
                let a = self.mem(o)?;
                let ptr = self.ptr_t();
                let v = self.load_imm(ptr, a, 8);
                let v = if tt == ptr { v } else { self.b.cast(CastOp::PtrToInt, v, tt) };
                Ok(Operand { ty: to, v: Val::Imm(v) })
            }
            (Repr::Mem, Repr::Mem) => Ok(Operand { ty: to, v: o.v }),
            (Repr::Imm(_), Repr::Mem) if self.pointee(o.ty).is_some() && self.pointee(to).is_some() => self.unsize(o, to),
            _ => Err(format!("cast from {} to {}", tcx.display(self.cx.r, o.ty), tcx.display(self.cx.r, to))),
        }
    }

    /// An `f128` cast to integer type `to`, saturating as Rust's `as` does
    /// (NaN gives 0), built from comparisons: the runtime converts through
    /// 128 bits, and the value is in range wherever that result is used.
    fn f128_to_int(&mut self, v: Value, to: TypeId, signed: bool) -> Value {
        let bits = match self.b.module().types.get(to) {
            epitome_ir::types::Type::Int(n) => *n,
            _ => 64,
        };
        let qt = self.b.type_of(v);
        let quad = |fx: &mut Self, b: u128| {
            let c = epitome_ir::module::Constant { ty: qt, kind: epitome_ir::module::ConstKind::Float(b) };
            Value::Const(fx.b.module().consts.intern(c))
        };
        // 2^k as a binary128, and -2^k.
        let pow2 = |k: u32| (k as u128 + 16383) << 112;
        let (lo, hi) = if signed { (pow2(bits - 1) | 1 << 127, pow2(bits - 1)) } else { (0, pow2(bits)) };
        let (lo, hi) = (quad(self, lo), quad(self, hi));
        let i128t = self.b.module().types.int(128);
        let op = if signed { CastOp::FPToSI } else { CastOp::FPToUI };
        let r = self.b.cast(op, v, i128t);
        let r = self.int_to(r, to, signed);
        let min = if signed { 1u128 << (bits - 1) } else { 0 };
        let max = if signed { (1u128 << (bits - 1)) - 1 } else if bits == 128 { u128::MAX } else { (1u128 << bits) - 1 };
        let (min, max) = (self.int_const(to, min), self.int_const(to, max));
        let zero = self.int_const(to, 0);
        let below = self.b.fcmp(FCmpPred::Olt, v, lo);
        let above = self.b.fcmp(FCmpPred::Oge, v, hi);
        let nan = self.b.fcmp(FCmpPred::Uno, v, v);
        let r = self.b.select(below, min, r);
        let r = self.b.select(above, max, r);
        self.b.select(nan, zero, r)
    }

    /// An integer made `to` wide (truncated, or extended by `signed`).
    pub fn int_to(&mut self, v: Value, to: TypeId, signed: bool) -> Value {
        let from = self.b.type_of(v);
        let bits = |b: &mut FunctionBuilder, t: TypeId| match b.module().types.get(t) {
            epitome_ir::types::Type::Int(n) => *n,
            _ => 64,
        };
        let (fb, tb) = (bits(&mut self.b, from), bits(&mut self.b, to));
        if fb == tb {
            v
        } else if fb > tb {
            self.b.trunc(v, to)
        } else if signed {
            self.b.sext(v, to)
        } else {
            self.b.zext(v, to)
        }
    }

    // ---- indexing, structs ----

    fn index(&mut self, n: NodeId, kids: &[NodeId]) -> R<Pv> {
        let tcx = self.cx.tcx;
        if let Some(&(m, margs)) = self.body.tables.methods.get(&n) {
            // `*Index::index(&base, i)`.
            let margs = self.sub_list(margs);
            let p = self.place(kids[0])?;
            let rt = tcx.mk(TyKind::Ref(Mutbl::Not, p.ty));
            let recv = self.addr_of(p, rt)?;
            let i = self.operand(kids[1])?;
            let r = self.call_def(m, margs, vec![recv, i], None)?;
            let o = self.operand_of(r)?;
            return Ok(Pv::Place(self.deref(o)?));
        }
        let base = self.place(kids[0])?;
        let i = self.operand(kids[1])?;
        let iv = self.imm(i)?;
        Ok(Pv::Place(self.element(base, iv)?))
    }

    /// Element `i` of the array or slice at `base`, checked against its
    /// length.
    pub fn element(&mut self, base: Place, iv: Value) -> R<Place> {
        let tcx = self.cx.tcx;
        let (elem, len) = match tcx.kind(base.ty) {
            TyKind::Array(e, c) => {
                let n = self.cx.lcx.eval_const(c, &Default::default()).ok_or("array length not known")?;
                (e, self.usize_const(n as u64))
            }
            TyKind::Slice(e) => (e, base.meta.ok_or("slice without a length")?),
            TyKind::Str => (tcx.mk(TyKind::Uint(UintTy::U8)), base.meta.ok_or("str without a length")?),
            _ => return Err(format!("index of {}", tcx.display(self.cx.r, base.ty))),
        };
        let ok = self.b.icmp(ICmpPred::Ult, iv, len);
        self.check(ok);
        let stride = self.layout(elem)?.size;
        let s = self.usize_const(stride);
        let off = self.b.mul(iv, s);
        let i8t = self.i8t();
        let addr = self.b.gep(i8t, base.addr, &[off]);
        Ok(Place { addr, meta: None, ty: elem })
    }

    fn struct_expr(&mut self, n: NodeId, kids: &[NodeId]) -> R<Pv> {
        let tcx = self.cx.tcx;
        let ast = self.body.ast;
        let t = self.ty(n)?;
        let TyKind::Adt(d, _) = tcx.kind(t) else { return Err("struct literal of something not a struct".into()) };
        let adt = tcx.adts.get(&d).ok_or("not an ADT")?;
        let v = self.variant_of_path(kids[0], d).unwrap_or(0);
        let fields: Vec<&str> = adt.variants[v as usize].fields.iter().map(|f| f.name).collect();
        let union = adt.kind == AdtKind::Union;
        let p = self.temp(t)?;
        let mut given = vec![false; fields.len()];
        for &f in &kids[1..] {
            let fk = self.kids(f);
            match ast.kind(f) {
                Kind::FieldInit => {
                    let name = ast.token(fk[0]).0;
                    let i = fields.iter().position(|&x| x == name).ok_or_else(|| format!("no field `{name}`"))?;
                    let o = match fk.get(1) {
                        Some(&e) => self.operand(e)?,
                        None => {
                            // `S { a }`: the local `a`.
                            let Some(Res::Local(lp)) = self.res(fk[0]) else { return Err("shorthand field without a local".into()) };
                            let key = (self.body.k, self.body.f, *lp);
                            let lp = self.local_place(key)?;
                            self.load(lp)?
                        }
                    };
                    let fp = if union { Place { addr: p.addr, meta: None, ty: self.field_ty(t, 0, i)? } } else { self.field(p, v, i)? };
                    self.store(Operand { ty: fp.ty, v: o.v }, fp)?;
                    given[i] = true;
                }
                Kind::StructBase => {
                    let Some(&b) = fk.first() else { return Err("`..` without a base".into()) };
                    let bp = self.place(b)?;
                    for (i, g) in given.iter().enumerate() {
                        if !g {
                            let src = self.field(bp, v, i)?;
                            let o = self.load(src)?;
                            let dst = self.field(p, v, i)?;
                            self.store(o, dst)?;
                        }
                    }
                }
                _ => {}
            }
        }
        if !union {
            self.set_variant(p, v)?;
        }
        Ok(Pv::Val(Operand { ty: t, v: self.load(p)?.v }))
    }

    /// A tuple struct or variant built from its fields.
    fn construct(&mut self, t: Ty, d: DefId, v: u32, args: Vec<Operand>) -> R<Pv> {
        let _ = d;
        let p = self.temp(t)?;
        for (i, o) in args.into_iter().enumerate() {
            let fp = self.field(p, v, i)?;
            self.store(Operand { ty: fp.ty, v: o.v }, fp)?;
        }
        self.set_variant(p, v)?;
        Ok(Pv::Val(Operand { ty: t, v: self.load(p)?.v }))
    }

    // ---- control flow ----

    /// Lowers condition `n`, falling through when it holds and jumping to
    /// `els` when not; `let` patterns in it bind as they match.
    pub fn cond(&mut self, n: NodeId, els: BlockId) -> R<()> {
        let ast = self.body.ast;
        let kids = self.kids(n);
        match ast.kind(n) {
            Kind::Binary if ast.text(kids[1]) == "&&" => {
                self.cond(kids[0], els)?;
                self.cond(kids[2], els)
            }
            Kind::Paren => self.cond(kids[0], els),
            Kind::Let => {
                let p = self.place(kids[1])?;
                self.pat_match(kids[0], p, Some(els))
            }
            _ => {
                let o = self.operand(n)?;
                if self.terminated() {
                    return Ok(());
                }
                let v = self.imm(o)?;
                let c = self.truthy(v);
                let then = self.new_block("then");
                self.b.cond_br(c, then, els);
                self.b.at(then);
                Ok(())
            }
        }
    }

    fn if_expr(&mut self, n: NodeId, kids: &[NodeId]) -> R<Pv> {
        let t = self.ty(n)?;
        let result = if self.repr(t)? == Repr::Zst { None } else { Some(self.temp(t)?) };
        let els = self.new_block("else");
        let join = self.new_block("if.end");
        self.cond(kids[0], els)?;
        let v = self.operand(kids[1])?;
        if let Some(p) = result
            && !self.terminated()
        {
            self.store(Operand { ty: t, v: v.v }, p)?;
        }
        self.goto(join);
        self.b.at(els);
        if let Some(&e) = kids.get(2) {
            let v = self.operand(e)?;
            if let Some(p) = result
                && !self.terminated()
            {
                self.store(Operand { ty: t, v: v.v }, p)?;
            }
        }
        self.goto(join);
        self.b.at(join);
        Ok(match result {
            Some(p) => Pv::Place(p),
            None => Pv::Val(Operand { ty: t, v: Val::Zst }),
        })
    }

    fn match_expr(&mut self, n: NodeId, kids: &[NodeId]) -> R<Pv> {
        let ast = self.body.ast;
        let t = self.ty(n)?;
        let scrut = self.place(kids[0])?;
        let result = if self.repr(t)? == Repr::Zst { None } else { Some(self.temp(t)?) };
        let join = self.new_block("match.end");
        for &arm in &kids[1..] {
            if ast.kind(arm) != Kind::MatchArm {
                continue;
            }
            let ak = self.kids(arm);
            let next = self.new_block("arm.next");
            self.note(ak[0]);
            self.pat_match(ak[0], scrut, Some(next))?;
            let mut i = 1;
            if let Some(&g) = ak.get(1)
                && ast.kind(g) == Kind::Guard
            {
                let ge = self.body.ast.children(g)[0];
                self.cond(ge, next)?;
                i = 2;
            }
            if let Some(&body) = ak.get(i) {
                let v = self.operand(body)?;
                if let Some(p) = result
                    && !self.terminated()
                {
                    self.store(Operand { ty: t, v: v.v }, p)?;
                }
            }
            self.goto(join);
            self.b.at(next);
        }
        // No arm matched: cannot happen.
        if !self.terminated() {
            self.b.unreachable();
        }
        self.b.at(join);
        Ok(match result {
            Some(p) => Pv::Place(p),
            None => Pv::Val(Operand { ty: t, v: Val::Zst }),
        })
    }

    fn for_expr(&mut self, n: NodeId, kids: &[NodeId]) -> R<Pv> {
        let ast = self.body.ast;
        let tcx = self.cx.tcx;
        let rest: Vec<NodeId> = kids.iter().copied().filter(|&c| !matches!(ast.kind(c), Kind::Lifetime | Kind::Keyword)).collect();
        let label = kids.first().filter(|&&c| ast.kind(c) == Kind::Lifetime).map(|&c| ast.token(c).0);
        let calls = self.body.tables.implicit.get(&n).cloned().ok_or("`for` without its calls")?;
        let [(into_iter, l1), (next, l2)] = calls[..] else { return Err("`for` without its calls".into()) };
        let l1 = self.sub_list(l1);
        let l2 = self.sub_list(l2);
        let src = self.operand(rest[1])?;
        let it = self.call_def(into_iter, l1, vec![src], None)?;
        let it = self.operand_of(it)?;
        let slot = self.temp(it.ty)?;
        self.store(it, slot)?;
        let head = self.new_block("for");
        let exit = self.new_block("for.end");
        self.goto(head);
        self.b.at(head);
        let rt = tcx.mk(TyKind::Ref(Mutbl::Mut, it.ty));
        let recv = self.addr_of(slot, rt)?;
        let item = self.call_def(next, l2, vec![recv], None)?;
        let item = self.place_of(item)?;
        let some = self.lang_variant(item.ty, "Some")?;
        let ok = self.test_variant(item, some)?;
        let body = self.new_block("for.body");
        self.b.cond_br(ok, body, exit);
        self.b.at(body);
        let x = self.field(item, some, 0)?;
        self.bind_irrefutable(rest[0], x)?;
        self.loops.push(Breakable { label, block: false, exit, cont: Some(head), result: None });
        self.expr(rest[2])?;
        self.loops.pop();
        self.goto(head);
        self.b.at(exit);
        Ok(Pv::Val(self.unit()))
    }

    fn try_expr(&mut self, n: NodeId, kids: &[NodeId]) -> R<Pv> {
        let calls = self.body.tables.implicit.get(&n).cloned().ok_or("`?` without its calls")?;
        let [(branch, l1), (from_residual, l2)] = calls[..] else { return Err("`?` without its calls".into()) };
        let l1 = self.sub_list(l1);
        let l2 = self.sub_list(l2);
        let v = self.operand(kids[0])?;
        let cf = self.call_def(branch, l1, vec![v], None)?;
        let cf = self.place_of(cf)?;
        let cont = self.lang_variant(cf.ty, "Continue")?;
        let brk = self.lang_variant(cf.ty, "Break")?;
        let ok = self.test_variant(cf, cont)?;
        let go_on = self.new_block("try.ok");
        let bail = self.new_block("try.err");
        self.b.cond_br(ok, go_on, bail);
        self.b.at(bail);
        let residual = self.field(cf, brk, 0)?;
        let r = self.load(residual)?;
        let out = self.call_def(from_residual, l2, vec![r], None)?;
        match self.tries.last().copied() {
            // In a `try` block: its value.
            Some((exit, result)) => {
                if !self.terminated() {
                    let o = self.operand_of(out)?;
                    self.store(Operand { ty: result.ty, v: o.v }, result)?;
                }
                self.goto(exit);
            }
            None => self.ret_value(out)?,
        }
        self.b.at(go_on);
        Ok(Pv::Place(self.field(cf, cont, 0)?))
    }

    // ---- calls ----

    fn call_expr(&mut self, n: NodeId, kids: &[NodeId]) -> R<Pv> {
        let tcx = self.cx.tcx;
        let arg_nodes = self.kids(kids[1]);
        if let Some(&(m, margs)) = self.body.tables.methods.get(&n) {
            // A call through the `Fn` traits: `f(a, b)` is `Fn::call(&f,
            // (a, b))`.
            let margs = self.sub_list(margs);
            let callee = self.expr(kids[0])?;
            let mut args = Vec::new();
            for &a in &arg_nodes {
                args.push(self.operand(a)?);
            }
            let _ = m;
            return self.call_value(callee, margs, args);
        }
        let ct = self.ty(kids[0])?;
        let mut args = Vec::new();
        for &a in &arg_nodes {
            args.push(self.operand(a)?);
        }
        match tcx.kind(ct) {
            TyKind::FnDef(d, l) => {
                let l = tcx.tys(l);
                match self.cx.r.defs[d.0 as usize].kind {
                    DefKind::Struct => {
                        let t = self.ty(n)?;
                        self.construct(t, d, 0, args)
                    }
                    DefKind::Variant => {
                        let t = self.ty(n)?;
                        let TyKind::Adt(e, _) = tcx.kind(t) else { return Err("variant of no enum".into()) };
                        let v = tcx.adts.get(&e).and_then(|a| a.variants.iter().position(|x| x.def == Some(d))).ok_or("no such variant")? as u32;
                        self.construct(t, d, v, args)
                    }
                    _ => self.call_def(d, l, args, Some(n)),
                }
            }
            TyKind::FnPtr(_) => {
                let callee = self.operand(kids[0])?;
                self.call_fn_ptr(callee, args)
            }
            // A closure called directly: through its place, so that what
            // an `FnMut` changes stays changed.
            TyKind::Closure(..) | TyKind::Ref(..) => {
                let callee = self.expr(kids[0])?;
                self.call_value(callee, Vec::new(), args)
            }
            _ => Err(format!("call of {}", tcx.display(self.cx.r, ct))),
        }
    }

    /// A call through the `Fn` traits of a value of type `callee.ty`.
    fn call_value(&mut self, callee: Pv, margs: Vec<Ty>, args: Vec<Operand>) -> R<Pv> {
        let tcx = self.cx.tcx;
        let _ = margs;
        let mut place = self.place_of(callee)?;
        while let TyKind::Ref(..) = tcx.kind(place.ty) {
            let o = self.load(place)?;
            place = self.deref(o)?;
        }
        match tcx.kind(place.ty) {
            TyKind::FnDef(d, l) => self.call_def(d, tcx.tys(l), args, None),
            TyKind::FnPtr(_) => {
                let c = self.load(place)?;
                self.call_fn_ptr(c, args)
            }
            TyKind::Closure(c, _, l) => {
                let owner = *self.cx.typeck.closure_owners.get(&c).ok_or("closure without an owner")?;
                let decl = self.cx.fns.get(&Item::Closure(c, owner, l)).cloned().ok_or("closure was not declared")?;
                let rt = tcx.mk(TyKind::Ref(Mutbl::Mut, place.ty));
                let mut all = vec![Operand { ty: rt, v: Val::Imm(place.addr) }];
                all.extend(args);
                let g = decl.g;
                self.emit_call(&decl.abi, |b, a| b.call(g, a), all)
            }
            _ => Err(format!("call of {}", tcx.display(self.cx.r, place.ty))),
        }
    }

    fn call_fn_ptr(&mut self, callee: Operand, args: Vec<Operand>) -> R<Pv> {
        let tcx = self.cx.tcx;
        let TyKind::FnPtr(s) = tcx.kind(callee.ty) else { return Err("not a function pointer".into()) };
        let sig = tcx.sig_of(s);
        let inputs: Vec<Ty> = tcx.tys(sig.inputs);
        let abi = self.cx.abi(self.b.module(), inputs, sig.output, sig.variadic)?;
        let (ret, params) = Cx::ir_fn_type(self.b.module(), &abi);
        let fty = self.b.module().types.function(ret, &params, abi.variadic);
        let target = self.imm(callee)?;
        self.emit_call(&abi, |b, a| b.call_indirect(fty, target, a), args)
    }

    /// The address of function `d` at `args`, as a value.
    pub fn fn_address(&mut self, d: DefId, args: Vec<Ty>) -> R<Value> {
        match crate::mono::instance(self.cx.r, self.cx.crates, self.cx.tcx, &mut self.solver, d, args) {
            Resolved::Item(it) => {
                let decl = self.cx.fns.get(&it).ok_or_else(|| format!("`{}` was not declared", self.cx.r.def_path(d)))?;
                let g = decl.g;
                Ok(self.b.module().global_value(g))
            }
            _ => Err(format!("address of `{}`", self.cx.r.def_path(d))),
        }
    }

    /// A call of `d` at `targs` with arguments `args` (each already of its
    /// parameter's type).
    pub fn call_def(&mut self, d: DefId, targs: Vec<Ty>, args: Vec<Operand>, _at: Option<NodeId>) -> R<Pv> {
        let tcx = self.cx.tcx;
        let r = self.cx.r;
        if crate::mono::has_param_or_infer_list(tcx, &targs) {
            let a: Vec<String> = targs.iter().map(|&t| tcx.display(r, t)).collect();
            return Err(format!("call of `{}` with arguments not concrete: <{}>", r.def_path(d), a.join(", ")));
        }
        let resolved = crate::mono::instance(r, self.cx.crates, tcx, &mut self.solver, d, targs.clone());
        match resolved {
            Resolved::Item(it @ (Item::Fn(f, _) | Item::Foreign(f))) => {
                if r.def_path(f).starts_with("core::panicking::") {
                    self.trap();
                    return Ok(Pv::Val(Operand { ty: tcx.mk(TyKind::Never), v: Val::Zst }));
                }
                let decl = self.cx.fns.get(&it).cloned().ok_or_else(|| {
                    let a: Vec<String> = match it {
                        Item::Fn(_, l) => tcx.tys(l).into_iter().map(|t| tcx.display(r, t)).collect(),
                        _ => Vec::new(),
                    };
                    format!("`{}` was not declared at {}", r.def_path(f), a.join(", "))
                })?;
                let g = decl.g;
                self.emit_call(&decl.abi, |b, a| b.call(g, a), args)
            }
            Resolved::Item(Item::Intrinsic(f, l)) => {
                let l = tcx.tys(l);
                self.intrinsic(f, l, args)
            }
            Resolved::Builtin => self.builtin(d, targs, args),
            _ => Err(format!("call of `{}` not resolved", r.def_path(d))),
        }
    }

    /// A trait method with a built-in impl: `Clone` of a `Copy` type, the
    /// `Fn` traits of function items and pointers.
    fn builtin(&mut self, d: DefId, targs: Vec<Ty>, args: Vec<Operand>) -> R<Pv> {
        let tcx = self.cx.tcx;
        let name = self.cx.r.defs[d.0 as usize].name;
        let tr = crate::mono::trait_of(self.cx.r, d);
        if tr.is_some() && tr == tcx.lang.get("clone").copied() && name == "clone" {
            // `*self`, copied.
            let p = self.deref(args[0])?;
            let o = self.load(p)?;
            let tmp = self.temp(o.ty)?;
            self.store(o, tmp)?;
            return Ok(Pv::Place(tmp));
        }
        if matches!(name, "call" | "call_mut" | "call_once") && args.len() == 2 {
            // The arguments come as a tuple.
            let tuple = self.place_of(Pv::Val(args[1]))?;
            let TyKind::Tuple(l) = tcx.kind(tuple.ty) else { return Err("`Fn` arguments not a tuple".into()) };
            let mut inner = Vec::new();
            for i in 0..tcx.tys(l).len() {
                let fp = self.field(tuple, 0, i)?;
                inner.push(self.load(fp)?);
            }
            return self.call_value(Pv::Val(args[0]), targs, inner);
        }
        if targs.first().is_some_and(|&t| matches!(tcx.kind(t), TyKind::Dyn(_))) {
            let out = self.cx.signature(&mut self.solver, d, &targs)?.1;
            return Ok(Pv::Val(self.trap_here(out, "`dyn` (a method)")));
        }
        Err(format!("built-in `{}` on {}", self.cx.r.def_path(d), targs.first().map(|&t| tcx.display(self.cx.r, t)).unwrap_or_default()))
    }

    /// Emits a call with `abi`, passing `args`.
    fn emit_call(&mut self, abi: &super::FnAbi, call: impl FnOnce(&mut FunctionBuilder<'m>, &[Value]) -> Option<Value>, args: Vec<Operand>) -> R<Pv> {
        if self.terminated() {
            return Ok(Pv::Val(Operand { ty: abi.output, v: Val::Zst }));
        }
        let mut ir_args = Vec::new();
        let dst = match abi.ret {
            Repr::Mem => {
                let p = self.temp(abi.output)?;
                ir_args.push(p.addr);
                Some(p)
            }
            _ => None,
        };
        if args.len() < abi.args.len() || (!abi.variadic && args.len() != abi.args.len()) {
            return Err("wrong number of arguments".into());
        }
        for (i, o) in args.iter().enumerate() {
            match abi.args.get(i) {
                Some(Repr::Zst) => {}
                Some(Repr::Imm(_)) => ir_args.push(self.imm(*o)?),
                Some(Repr::Mem) => ir_args.push(self.mem(*o)?),
                // A variadic argument: as itself.
                None => match self.repr(o.ty)? {
                    Repr::Imm(_) => ir_args.push(self.imm(*o)?),
                    _ => return Err("variadic argument that is not a scalar".into()),
                },
            }
        }
        let got = call(&mut self.b, &ir_args);
        let tcx = self.cx.tcx;
        if self.layout(abi.output)?.is_uninhabited() {
            self.b.unreachable();
            self.dead();
            return Ok(Pv::Val(Operand { ty: tcx.mk(TyKind::Never), v: Val::Zst }));
        }
        Ok(match (abi.ret, dst) {
            (Repr::Imm(_), _) => Pv::Val(Operand { ty: abi.output, v: Val::Imm(got.ok_or("call without a value")?) }),
            (Repr::Mem, Some(p)) => Pv::Place(p),
            _ => Pv::Val(Operand { ty: abi.output, v: Val::Zst }),
        })
    }
}

/// The value of a float literal, without its suffix (`2.5f16`, `1e3`).
pub(crate) fn parse_float(tok: &str) -> Option<f64> {
    let t = ["f16", "f32", "f64", "f128"].iter().fold(tok, |t, s| t.strip_suffix(s).unwrap_or(t));
    let digits: String = t.chars().filter(|&c| c != '_').collect();
    digits.parse().ok()
}

/// The value of an integer literal, without its suffix.
pub(crate) fn parse_int(tok: &str) -> Option<u128> {
    let t: String = tok.chars().filter(|&c| c != '_').collect();
    let (radix, body) = if let Some(r) = t.strip_prefix("0x") {
        (16, r)
    } else if let Some(r) = t.strip_prefix("0o") {
        (8, r)
    } else if let Some(r) = t.strip_prefix("0b") {
        (2, r)
    } else {
        (10, t.as_str())
    };
    let end = body.find(|c: char| !c.is_digit(radix)).unwrap_or(body.len());
    u128::from_str_radix(&body[..end], radix).ok()
}

/// One escape or character at the start of `s`: the character and the
/// rest.
fn unescape_one(s: &str) -> Option<(u32, &str)> {
    let mut it = s.chars();
    let c = it.next()?;
    if c != '\\' {
        return Some((c as u32, it.as_str()));
    }
    let e = it.next()?;
    let rest = it.as_str();
    Some(match e {
        'n' => (10, rest),
        'r' => (13, rest),
        't' => (9, rest),
        '\\' => (92, rest),
        '0' => (0, rest),
        '\'' => (39, rest),
        '"' => (34, rest),
        'x' => (u32::from_str_radix(rest.get(..2)?, 16).ok()?, &rest[2..]),
        'u' => {
            let close = rest.find('}')?;
            let hex: String = rest[1..close].chars().filter(|&c| c != '_').collect();
            (u32::from_str_radix(&hex, 16).ok()?, &rest[close + 1..])
        }
        _ => return None,
    })
}

/// A character literal `'c'` (or byte literal without its `b`).
pub(crate) fn parse_char(tok: &str) -> Option<u32> {
    let body = tok.strip_prefix('\'')?;
    let end = body.rfind('\'')?;
    unescape_one(&body[..end]).map(|x| x.0)
}

/// The bytes of a string or byte string literal (raw or not).
pub(crate) fn parse_str(tok: &str) -> Option<Vec<u8>> {
    let byte = tok.starts_with('b');
    let t = tok.strip_prefix('b').unwrap_or(tok);
    if let Some(r) = t.strip_prefix('r') {
        let hashes = r.len() - r.trim_start_matches('#').len();
        let inner = &r[hashes..];
        let inner = inner.strip_prefix('"')?;
        let end = inner.rfind('"')?;
        return Some(inner.as_bytes()[..end].to_vec());
    }
    let inner = t.strip_prefix('"')?;
    let end = inner.rfind('"')?;
    let mut s = &inner[..end];
    let mut out = Vec::new();
    while !s.is_empty() {
        if let Some(rest) = s.strip_prefix("\\\n").or_else(|| s.strip_prefix("\\\r\n")) {
            // A line continuation: the newline and the next line's
            // leading whitespace go.
            s = rest.trim_start();
            continue;
        }
        let (c, rest) = unescape_one(s)?;
        if byte {
            out.push(c as u8);
        } else {
            let ch = char::from_u32(c)?;
            let mut buf = [0u8; 4];
            out.extend_from_slice(ch.encode_utf8(&mut buf).as_bytes());
        }
        s = rest;
    }
    Some(out)
}
