//! Building IR in code: constructors for the key structs, and a
//! `FunctionBuilder` that adds instructions at a block's end, working out
//! each result's type and checking its operands as it goes.
//!
//! ```
//! use epitome_ir::module::{ICmpPred, Module};
//!
//! let mut m = Module::default();
//! let i64t = m.types.int(64);
//! // define i64 @max(i64 %a, i64 %b)
//! let mut b = m.function("max", i64t, &[("a", i64t), ("b", i64t)]);
//! let (x, y) = (b.param(0), b.param(1));
//! b.block("entry");
//! let gt = b.name("gt").icmp(ICmpPred::Sgt, x, y);
//! let r = b.name("r").select(gt, x, y);
//! b.ret(r);
//! b.finish().unwrap();
//! assert!(m.to_string().contains("%r = select i1 %gt, i64 %a, i64 %b"));
//! ```
//!
//! An operand of the wrong type is a mistake in the caller, as in LLVM's
//! IRBuilder, so it panics, saying what was expected; `finish` checks that
//! every block ends in exactly one terminator and returns an error if not.

use crate::apint::ApInt;
use crate::float::{self, FloatKind};
use crate::module::*;
use crate::types::{Type, TypeId, Types};

// ---------------------------------------------------------------------------
// Types

impl Types {
    pub fn half(&mut self) -> TypeId {
        self.intern(Type::Half)
    }
    pub fn float(&mut self) -> TypeId {
        self.intern(Type::Float)
    }
    pub fn double(&mut self) -> TypeId {
        self.intern(Type::Double)
    }
    /// `<len x elem>`
    pub fn vector(&mut self, len: u32, elem: TypeId) -> TypeId {
        self.intern(Type::Vector { scalable: false, len, elem })
    }
    /// `[len x elem]`
    pub fn array(&mut self, len: u64, elem: TypeId) -> TypeId {
        self.intern(Type::Array { len, elem })
    }
    /// The literal struct `{ fields... }`.
    pub fn struct_of(&mut self, fields: &[TypeId]) -> TypeId {
        self.intern(Type::Struct { packed: false, fields: fields.to_vec() })
    }
    /// `ret (params...)`, with `...` if `varargs`.
    pub fn function(&mut self, ret: TypeId, params: &[TypeId], varargs: bool) -> TypeId {
        self.intern(Type::Function { ret, params: params.to_vec(), varargs })
    }
}

// ---------------------------------------------------------------------------
// Constants, globals, calls and instructions

impl Constant {
    /// An integer of type `ty` (`iN`), truncated to its width.
    pub fn int(types: &Types, ty: TypeId, value: i64) -> Constant {
        let Type::Int(bits) = *types.get(ty) else { panic!("Constant::int of a type that is not an integer") };
        Constant { ty, kind: ConstKind::Int(ApInt::from_i64(bits, value)) }
    }

    /// A floating-point number of type `ty` (`half`, `float`, `double`...),
    /// rounded to it.
    pub fn float(types: &Types, ty: TypeId, value: f64) -> Constant {
        let kind = FloatKind::of(types.get(ty)).expect("Constant::float of a type that is not floating point");
        let bits = float::from_f64(kind, value).expect("a floating-point value the type can hold");
        Constant { ty, kind: ConstKind::Float(bits) }
    }
}

impl Call {
    /// A plain call: no attributes, tail marker, bundles or fast-math
    /// flags, the C calling convention.
    pub fn new(fn_ty: TypeId, callee: Value, args: &[Value]) -> Call {
        Call {
            tail: TailKind::None,
            fmf: FastMath::default(),
            cconv: CallConv::default(),
            ret_attrs: vec![],
            addrspace: None,
            fn_ty,
            callee,
            args: args.iter().map(|&value| CallArg { value, attrs: vec![] }).collect(),
            fn_attrs: vec![],
            bundles: vec![],
        }
    }
}

impl Inst {
    /// An instruction with no result, attachments or debug records.
    pub fn new(op: Op) -> Inst {
        Inst { result: None, op, attachments: vec![], debug_records: vec![] }
    }
}

impl Function {
    /// A declaration of type `ty` (a function type), with a parameter of
    /// each of its parameter types.
    pub fn declaration(types: &Types, ty: TypeId) -> Function {
        let Type::Function { params, .. } = types.get(ty).clone() else {
            panic!("Function::declaration of a type that is not a function type")
        };
        let mut f = Function { ty, ..Default::default() };
        for (k, p) in params.into_iter().enumerate() {
            let local = f.add_local(None, p, LocalDef::Param(k as u32));
            f.params.push(Param { local, attrs: vec![] });
        }
        f
    }

    /// The first block of a function with none yet, placed in its layout.
    /// (`add_block` takes the label type from the first block.)
    pub fn add_entry_block(&mut self, name: Option<String>, label_ty: TypeId) -> BlockId {
        assert!(self.blocks.is_empty(), "the function already has blocks");
        let id = BlockId(0);
        let local = self.add_local(name, label_ty, LocalDef::Block(id));
        self.blocks.push(Block { local, insts: Vec::new() });
        self.layout.push(id);
        id
    }
}

impl Global {
    /// A function with external linkage.
    pub fn function(name: &str, f: Function) -> Global {
        Global {
            name: Some(name.to_string()),
            props: GlobalProps::default(),
            attachments: vec![],
            kind: GlobalKind::Function(Box::new(f)),
        }
    }

    /// A global variable of type `ty`, with external linkage.
    pub fn variable(name: &str, ty: TypeId, init: Option<ConstId>, constant: bool) -> Global {
        Global {
            name: Some(name.to_string()),
            props: GlobalProps::default(),
            attachments: vec![],
            kind: GlobalKind::Variable(GlobalVar {
                constant,
                externally_initialized: false,
                ty,
                init,
                section: None,
                comdat: None,
                align: None,
                code_model: None,
                sanitizer: vec![],
                attrs: vec![],
            }),
        }
    }
}

impl Module {
    /// Adds a global, in order, and returns its id.
    pub fn add_global(&mut self, g: Global) -> GlobalId {
        let id = GlobalId(self.globals.len() as u32);
        self.globals.push(g);
        self.global_order.push(id);
        id
    }

    /// The global named `name`, if there is one.
    pub fn find_global(&self, name: &str) -> Option<GlobalId> {
        self.global_order.iter().copied().find(|&g| self.global(g).name.as_deref() == Some(name))
    }

    /// Declares function `name` (or finds it, if the module has it).
    pub fn declare(&mut self, name: &str, ret: TypeId, params: &[TypeId], varargs: bool) -> GlobalId {
        if let Some(g) = self.find_global(name) {
            return g;
        }
        let ty = self.types.function(ret, params, varargs);
        let f = Function::declaration(&self.types, ty);
        self.add_global(Global::function(name, f))
    }

    /// A global's address, as a value (`ptr @name`).
    pub fn global_value(&mut self, g: GlobalId) -> Value {
        let ptr = self.types.ptr(0);
        Value::Const(self.consts.intern(Constant { ty: ptr, kind: ConstKind::Global(g) }))
    }

    /// An integer constant of type `ty`: `iN`, or a vector of them (a
    /// splat).
    pub fn const_int(&mut self, ty: TypeId, value: i64) -> Value {
        self.scalar_or_splat(ty, |m, elem| Constant::int(&m.types, elem, value))
    }

    /// A floating-point constant of type `ty`, or a splat of one.
    pub fn const_float(&mut self, ty: TypeId, value: f64) -> Value {
        self.scalar_or_splat(ty, |m, elem| Constant::float(&m.types, elem, value))
    }

    /// `zeroinitializer` of type `ty` (`null` for a pointer).
    pub fn const_zero(&mut self, ty: TypeId) -> Value {
        let kind = if matches!(self.types.get(ty), Type::Ptr(_)) { ConstKind::Null } else { ConstKind::Zero };
        Value::Const(self.consts.intern(Constant { ty, kind }))
    }

    /// An array of bytes, `[N x i8] c"..."` (add a 0 for a C string).
    pub fn const_bytes(&mut self, bytes: &[u8]) -> Value {
        let i8t = self.types.int(8);
        let ty = self.types.array(bytes.len() as u64, i8t);
        Value::Const(self.consts.intern(Constant { ty, kind: ConstKind::Bytes(bytes.to_vec()) }))
    }

    pub fn poison(&mut self, ty: TypeId) -> Value {
        Value::Const(self.consts.intern(Constant { ty, kind: ConstKind::Poison }))
    }

    pub fn undef(&mut self, ty: TypeId) -> Value {
        Value::Const(self.consts.intern(Constant { ty, kind: ConstKind::Undef }))
    }

    fn scalar_or_splat(&mut self, ty: TypeId, make: impl Fn(&mut Module, TypeId) -> Constant) -> Value {
        match *self.types.get(ty) {
            Type::Vector { elem, .. } => {
                let c = make(self, elem);
                let e = self.consts.intern(c);
                Value::Const(self.consts.intern(Constant { ty, kind: ConstKind::Splat(e) }))
            }
            _ => {
                let c = make(self, ty);
                Value::Const(self.consts.intern(c))
            }
        }
    }

    /// Starts defining function `name`: `ret name(params...)`, the
    /// parameters named as given. `finish` adds it to the module, in place
    /// of a declaration of the same name if there is one. The short form of
    /// `Function::builder(name).returns(ret).param(..)...begin(m)`.
    pub fn function(&mut self, name: &str, ret: TypeId, params: &[(&str, TypeId)]) -> FunctionBuilder<'_> {
        params.iter().fold(Function::builder(name).returns(ret), |s, &(n, t)| s.param(n, t)).begin(self)
    }
}

// ---------------------------------------------------------------------------
// Fluent builders for functions and globals

impl Function {
    /// A function's signature and settings, chained:
    ///
    /// ```
    /// # use epitome_ir::module::{Function, Linkage, Module};
    /// let mut m = Module::default();
    /// let i64t = m.types.int(64);
    /// let mut b = Function::builder("twice")
    ///     .returns(i64t)
    ///     .param("x", i64t)
    ///     .linkage(Linkage::Internal)
    ///     .attr("nounwind")
    ///     .begin(&mut m);
    /// b.block("entry");
    /// let x = b.param(0);
    /// let r = b.add(x, x);
    /// b.ret(r);
    /// b.finish().unwrap();
    /// assert!(m.to_string().contains("define internal i64 @twice(i64 %x) nounwind {"));
    /// ```
    pub fn builder(name: &str) -> FunctionSpec {
        FunctionSpec {
            name: name.to_string(),
            ret: None,
            params: vec![],
            varargs: false,
            props: GlobalProps::default(),
            attrs: vec![],
        }
    }
}

/// A function's name, signature, linkage and attributes, chained; then
/// `begin` to define it (a `FunctionBuilder`) or `declare` to declare it.
#[derive(Clone, Debug)]
pub struct FunctionSpec {
    name: String,
    /// `void` when not given.
    ret: Option<TypeId>,
    params: Vec<(String, TypeId, Vec<Attr>)>,
    varargs: bool,
    props: GlobalProps,
    attrs: Vec<Attr>,
}

impl FunctionSpec {
    /// The return type (`void` if never given).
    pub fn returns(mut self, ty: TypeId) -> Self {
        self.ret = Some(ty);
        self
    }

    /// A parameter, named (or `""` for a numbered one).
    pub fn param(mut self, name: &str, ty: TypeId) -> Self {
        self.params.push((name.to_string(), ty, vec![]));
        self
    }

    /// A keyword attribute on the last parameter (`noundef`, `nonnull`).
    pub fn param_attr(mut self, word: &str) -> Self {
        let last = self.params.last_mut().expect("param_attr: no parameter yet");
        last.2.push(Attr::Word(word.to_string()));
        self
    }

    /// Takes variable arguments (`...`).
    pub fn varargs(mut self) -> Self {
        self.varargs = true;
        self
    }

    pub fn linkage(mut self, linkage: Linkage) -> Self {
        self.props.linkage = linkage;
        self.props.explicit_linkage = true;
        self
    }

    /// `dso_local`
    pub fn dso_local(mut self) -> Self {
        self.props.dso_local = true;
        self
    }

    /// A keyword function attribute (`nounwind`, `noinline`).
    pub fn attr(mut self, word: &str) -> Self {
        self.attrs.push(Attr::Word(word.to_string()));
        self
    }

    fn function(&self, m: &mut Module, has_body: bool) -> Function {
        let ret = self.ret.unwrap_or_else(|| m.types.void());
        let tys: Vec<TypeId> = self.params.iter().map(|p| p.1).collect();
        let ty = m.types.function(ret, &tys, self.varargs);
        let mut f = Function { ty, has_body, fn_attrs: self.attrs.clone(), ..Default::default() };
        for (k, (pname, pty, attrs)) in self.params.iter().enumerate() {
            let name = (!pname.is_empty()).then(|| pname.clone());
            let local = f.add_local(name, *pty, LocalDef::Param(k as u32));
            f.params.push(Param { local, attrs: attrs.clone() });
        }
        f
    }

    /// Starts the definition: blocks and instructions go in with the
    /// builder, and its `finish` adds the function to the module.
    pub fn begin(self, m: &mut Module) -> FunctionBuilder<'_> {
        let f = self.function(m, true);
        FunctionBuilder { m, f, name: self.name, props: self.props, cur: None, next_name: None }
    }

    /// Declares the function (or finds the module's global of that name).
    pub fn declare(self, m: &mut Module) -> GlobalId {
        if let Some(g) = m.find_global(&self.name) {
            return g;
        }
        let f = self.function(m, false);
        let mut g = Global::function(&self.name, f);
        g.props = self.props;
        m.add_global(g)
    }
}

impl Global {
    /// A global variable's settings, chained, then `build`:
    ///
    /// ```
    /// # use epitome_ir::module::{Global, Linkage, Module};
    /// let mut m = Module::default();
    /// let i32t = m.types.int(32);
    /// let seven = m.const_int(i32t, 7);
    /// Global::builder("seven").ty(i32t).init(seven).constant().linkage(Linkage::Private).align(4).build(&mut m);
    /// assert!(m.to_string().contains("@seven = private constant i32 7, align 4"));
    /// ```
    pub fn builder(name: &str) -> GlobalSpec {
        GlobalSpec {
            name: name.to_string(),
            ty: None,
            init: None,
            constant: false,
            props: GlobalProps::default(),
            align: None,
            section: None,
        }
    }
}

/// A global variable's type, initial value, linkage and placement, chained.
#[derive(Clone, Debug)]
pub struct GlobalSpec {
    name: String,
    ty: Option<TypeId>,
    init: Option<Value>,
    constant: bool,
    props: GlobalProps,
    align: Option<u64>,
    section: Option<Vec<u8>>,
}

impl GlobalSpec {
    /// The type (needed unless `init` gives it).
    pub fn ty(mut self, ty: TypeId) -> Self {
        self.ty = Some(ty);
        self
    }

    /// The initial value: a constant (`Module::const_int` and the like).
    pub fn init(mut self, value: Value) -> Self {
        assert!(matches!(value, Value::Const(_)), "init: a global's initial value must be a constant");
        self.init = Some(value);
        self
    }

    /// `constant` rather than `global`.
    pub fn constant(mut self) -> Self {
        self.constant = true;
        self
    }

    pub fn linkage(mut self, linkage: Linkage) -> Self {
        self.props.linkage = linkage;
        self.props.explicit_linkage = true;
        self
    }

    pub fn align(mut self, align: u64) -> Self {
        self.align = Some(align);
        self
    }

    pub fn section(mut self, name: &str) -> Self {
        self.section = Some(name.as_bytes().to_vec());
        self
    }

    /// Adds the global to the module.
    pub fn build(self, m: &mut Module) -> GlobalId {
        let init = self.init.map(|v| match v {
            Value::Const(c) => c,
            _ => unreachable!(),
        });
        let ty =
            self.ty.or_else(|| init.map(|c| m.consts.get(c).ty)).expect("a global needs a type or an initial value");
        let mut g = Global::variable(&self.name, ty, init, self.constant);
        g.props = self.props;
        if let GlobalKind::Variable(v) = &mut g.kind {
            v.align = self.align;
            v.section = self.section;
        }
        m.add_global(g)
    }
}

// ---------------------------------------------------------------------------
// FunctionBuilder

/// Builds one function's body: blocks, and instructions added at the end of
/// the current block (see the module documentation).
pub struct FunctionBuilder<'m> {
    m: &'m mut Module,
    f: Function,
    name: String,
    props: GlobalProps,
    cur: Option<BlockId>,
    next_name: Option<String>,
}

impl<'m> FunctionBuilder<'m> {
    /// The module, for types and constants.
    pub fn module(&mut self) -> &mut Module {
        self.m
    }

    /// The function as built so far.
    pub fn function(&self) -> &Function {
        &self.f
    }

    /// Parameter `k`.
    pub fn param(&self, k: usize) -> Value {
        Value::Local(self.f.params[k].local)
    }

    /// A new block at the end of the function, which instructions now go
    /// into.
    pub fn block(&mut self, name: &str) -> BlockId {
        let name = (!name.is_empty()).then(|| name.to_string());
        let b = if self.f.blocks.is_empty() {
            let label = self.m.types.label();
            self.f.add_entry_block(name, label)
        } else {
            let b = self.f.add_block(name);
            self.f.layout.push(b);
            b
        };
        self.cur = Some(b);
        b
    }

    /// A new block, not yet where instructions go (for a branch forward).
    pub fn new_block(&mut self, name: &str) -> BlockId {
        let here = self.cur;
        let b = self.block(name);
        self.cur = here;
        b
    }

    /// Instructions now go at the end of block `b`.
    pub fn at(&mut self, b: BlockId) -> &mut Self {
        self.cur = Some(b);
        self
    }

    pub fn current_block(&self) -> Option<BlockId> {
        self.cur
    }

    /// Names the next instruction's result (`%name`).
    pub fn name(&mut self, name: &str) -> &mut Self {
        self.next_name = Some(name.to_string());
        self
    }

    /// The type of value `v`.
    pub fn type_of(&mut self, v: Value) -> TypeId {
        self.m.value_type(Some(&self.f), v)
    }

    pub fn const_int(&mut self, ty: TypeId, value: i64) -> Value {
        self.m.const_int(ty, value)
    }

    pub fn const_float(&mut self, ty: TypeId, value: f64) -> Value {
        self.m.const_float(ty, value)
    }

    /// Adds `op` at the end of the current block, with a result of type `ty`
    /// if it has one; returns the result (or `None`).
    pub fn push(&mut self, op: Op, ty: Option<TypeId>) -> Option<Value> {
        let b = self.cur.expect("no current block: call block() first");
        if let Some(&last) = self.f.blocks[b.index()].insts.last() {
            assert!(!self.f.insts[last.index()].op.is_terminator(), "block {b:?} already ends in a terminator");
        }
        let name = self.next_name.take();
        let i = self.f.push_inst(b, op, ty.map(|t| (name, t)));
        ty.map(|_| self.f.result(i))
    }

    fn value(&mut self, op: Op, ty: TypeId) -> Value {
        self.push(op, Some(ty)).unwrap()
    }

    fn is_int(&mut self, ty: TypeId) -> bool {
        let s = self.m.types.scalar(ty);
        matches!(self.m.types.get(s), Type::Int(_))
    }

    fn is_float(&mut self, ty: TypeId) -> bool {
        let s = self.m.types.scalar(ty);
        FloatKind::of(self.m.types.get(s)).is_some()
    }

    fn same(&mut self, what: &str, a: Value, b: Value) -> TypeId {
        let (ta, tb) = (self.type_of(a), self.type_of(b));
        assert!(
            ta == tb,
            "{what}: operands of different types, {:?} and {:?}",
            self.m.types.get(ta),
            self.m.types.get(tb)
        );
        ta
    }

    // Arithmetic -------------------------------------------------------------

    /// A binary operation, with flags (`nuw`, `nsw`, `exact`, `disjoint`).
    pub fn binary_with(&mut self, op: BinOp, flags: IntFlags, a: Value, b: Value) -> Value {
        let ty = self.same(op.keyword(), a, b);
        if op.is_float() {
            assert!(self.is_float(ty), "{}: needs floating-point operands", op.keyword());
        } else {
            assert!(self.is_int(ty), "{}: needs integer operands", op.keyword());
        }
        self.value(Op::Binary { op, flags, fmf: FastMath::default(), lhs: a, rhs: b }, ty)
    }

    pub fn binary(&mut self, op: BinOp, a: Value, b: Value) -> Value {
        self.binary_with(op, IntFlags::NONE, a, b)
    }

    pub fn add(&mut self, a: Value, b: Value) -> Value {
        self.binary(BinOp::Add, a, b)
    }
    pub fn sub(&mut self, a: Value, b: Value) -> Value {
        self.binary(BinOp::Sub, a, b)
    }
    pub fn mul(&mut self, a: Value, b: Value) -> Value {
        self.binary(BinOp::Mul, a, b)
    }
    pub fn sdiv(&mut self, a: Value, b: Value) -> Value {
        self.binary(BinOp::SDiv, a, b)
    }
    pub fn udiv(&mut self, a: Value, b: Value) -> Value {
        self.binary(BinOp::UDiv, a, b)
    }
    pub fn srem(&mut self, a: Value, b: Value) -> Value {
        self.binary(BinOp::SRem, a, b)
    }
    pub fn urem(&mut self, a: Value, b: Value) -> Value {
        self.binary(BinOp::URem, a, b)
    }
    pub fn shl(&mut self, a: Value, b: Value) -> Value {
        self.binary(BinOp::Shl, a, b)
    }
    pub fn lshr(&mut self, a: Value, b: Value) -> Value {
        self.binary(BinOp::LShr, a, b)
    }
    pub fn ashr(&mut self, a: Value, b: Value) -> Value {
        self.binary(BinOp::AShr, a, b)
    }
    pub fn and(&mut self, a: Value, b: Value) -> Value {
        self.binary(BinOp::And, a, b)
    }
    pub fn or(&mut self, a: Value, b: Value) -> Value {
        self.binary(BinOp::Or, a, b)
    }
    pub fn xor(&mut self, a: Value, b: Value) -> Value {
        self.binary(BinOp::Xor, a, b)
    }
    pub fn fadd(&mut self, a: Value, b: Value) -> Value {
        self.binary(BinOp::FAdd, a, b)
    }
    pub fn fsub(&mut self, a: Value, b: Value) -> Value {
        self.binary(BinOp::FSub, a, b)
    }
    pub fn fmul(&mut self, a: Value, b: Value) -> Value {
        self.binary(BinOp::FMul, a, b)
    }
    pub fn fdiv(&mut self, a: Value, b: Value) -> Value {
        self.binary(BinOp::FDiv, a, b)
    }
    pub fn fneg(&mut self, a: Value) -> Value {
        let ty = self.type_of(a);
        assert!(self.is_float(ty), "fneg: needs a floating-point operand");
        self.value(Op::FNeg { fmf: FastMath::default(), value: a }, ty)
    }

    // Comparisons and selects ------------------------------------------------

    /// The type of a comparison's result: `i1`, or `<N x i1>` for vectors.
    fn bool_like(&mut self, ty: TypeId) -> TypeId {
        let i1 = self.m.types.int(1);
        self.m.types.with_elem(ty, i1)
    }

    pub fn icmp(&mut self, pred: ICmpPred, a: Value, b: Value) -> Value {
        let ty = self.same("icmp", a, b);
        let s = self.m.types.scalar(ty);
        assert!(matches!(self.m.types.get(s), Type::Int(_) | Type::Ptr(_)), "icmp: needs integer or pointer operands");
        let r = self.bool_like(ty);
        self.value(Op::ICmp { pred, samesign: false, lhs: a, rhs: b }, r)
    }

    pub fn fcmp(&mut self, pred: FCmpPred, a: Value, b: Value) -> Value {
        let ty = self.same("fcmp", a, b);
        assert!(self.is_float(ty), "fcmp: needs floating-point operands");
        let r = self.bool_like(ty);
        self.value(Op::FCmp { pred, fmf: FastMath::default(), lhs: a, rhs: b }, r)
    }

    pub fn select(&mut self, cond: Value, a: Value, b: Value) -> Value {
        let ty = self.same("select", a, b);
        let ct = self.type_of(cond);
        let cs = self.m.types.scalar(ct);
        assert!(matches!(self.m.types.get(cs), Type::Int(1)), "select: the condition must be i1 (or <N x i1>)");
        self.value(Op::Select { fmf: FastMath::default(), cond, then_value: a, else_value: b }, ty)
    }

    // Casts --------------------------------------------------------------------

    pub fn cast(&mut self, op: CastOp, v: Value, to: TypeId) -> Value {
        self.value(Op::Cast { op, flags: IntFlags::NONE, fmf: FastMath::default(), value: v }, to)
    }
    pub fn zext(&mut self, v: Value, to: TypeId) -> Value {
        self.cast(CastOp::ZExt, v, to)
    }
    pub fn sext(&mut self, v: Value, to: TypeId) -> Value {
        self.cast(CastOp::SExt, v, to)
    }
    pub fn trunc(&mut self, v: Value, to: TypeId) -> Value {
        self.cast(CastOp::Trunc, v, to)
    }
    pub fn bitcast(&mut self, v: Value, to: TypeId) -> Value {
        self.cast(CastOp::BitCast, v, to)
    }

    // Memory -------------------------------------------------------------------

    /// `alloca ty`: a pointer to a new stack slot.
    pub fn alloca(&mut self, ty: TypeId) -> Value {
        let ptr = self.m.types.ptr(0);
        self.value(Op::Alloca { inalloca: false, ty, count: None, align: None, addrspace: None }, ptr)
    }

    /// `load ty, ptr p`
    pub fn load(&mut self, ty: TypeId, p: Value) -> Value {
        self.value(Op::Load { volatile: false, atomic: None, ptr: p, align: None }, ty)
    }

    /// `store v, ptr p`
    pub fn store(&mut self, v: Value, p: Value) {
        self.push(Op::Store { volatile: false, atomic: None, value: v, ptr: p, align: None }, None);
    }

    /// `getelementptr source_ty, ptr p, indices...`
    pub fn gep(&mut self, source_ty: TypeId, p: Value, indices: &[Value]) -> Value {
        let ptr = self.m.types.ptr(0);
        self.value(Op::Gep { flags: GepFlags::default(), source_ty, ptr: p, indices: indices.into() }, ptr)
    }

    // Aggregates and vectors ---------------------------------------------------

    pub fn extract_value(&mut self, agg: Value, indices: &[u32]) -> Value {
        let mut ty = self.type_of(agg);
        for &k in indices {
            ty = self.m.types.member(ty, k as u64).expect("extractvalue: an index past the aggregate's members");
        }
        self.value(Op::ExtractValue { aggregate: agg, indices: indices.into() }, ty)
    }

    pub fn insert_value(&mut self, agg: Value, element: Value, indices: &[u32]) -> Value {
        let ty = self.type_of(agg);
        self.value(Op::InsertValue { aggregate: agg, element, indices: indices.into() }, ty)
    }

    pub fn extract_element(&mut self, vector: Value, index: Value) -> Value {
        let ty = self.type_of(vector);
        let Type::Vector { elem, .. } = *self.m.types.get(ty) else { panic!("extractelement: not a vector") };
        self.value(Op::ExtractElement { vector, index }, elem)
    }

    pub fn insert_element(&mut self, vector: Value, element: Value, index: Value) -> Value {
        let ty = self.type_of(vector);
        self.value(Op::InsertElement { vector, element, index }, ty)
    }

    // Calls --------------------------------------------------------------------

    /// A call of function `callee` (a function of the module); its value,
    /// unless it returns `void`.
    pub fn call(&mut self, callee: GlobalId, args: &[Value]) -> Option<Value> {
        let GlobalKind::Function(f) = &self.m.global(callee).kind else { panic!("call: not a function") };
        let fn_ty = f.ty;
        let target = self.m.global_value(callee);
        self.call_indirect(fn_ty, target, args)
    }

    /// A call through pointer `target`, of function type `fn_ty`.
    pub fn call_indirect(&mut self, fn_ty: TypeId, target: Value, args: &[Value]) -> Option<Value> {
        let Type::Function { ret, params, varargs } = self.m.types.get(fn_ty).clone() else {
            panic!("call: not a function type")
        };
        assert!(
            args.len() == params.len() || (varargs && args.len() > params.len()),
            "call: {} arguments for {} parameters",
            args.len(),
            params.len()
        );
        for (k, (&a, &p)) in args.iter().zip(&params).enumerate() {
            let t = self.type_of(a);
            assert!(t == p, "call: argument {k} is {:?}, the parameter {:?}", self.m.types.get(t), self.m.types.get(p));
        }
        let void = matches!(self.m.types.get(ret), Type::Void);
        self.push(Op::Call(Box::new(Call::new(fn_ty, target, args))), (!void).then_some(ret))
    }

    // Phis and control flow ----------------------------------------------------

    /// A phi of type `ty` at the start of the current block (after its other
    /// phis), with these entries; more can be added with `add_incoming`.
    pub fn phi(&mut self, ty: TypeId, incoming: &[(Value, BlockId)]) -> Value {
        let b = self.cur.expect("no current block: call block() first");
        let name = self.next_name.take();
        let i = self.f.add_inst(Op::Phi { fmf: FastMath::default(), incoming: incoming.to_vec() }, Some((name, ty)));
        let insts = &self.f.blocks[b.index()].insts;
        let at = insts.iter().take_while(|&&x| self.f.insts[x.index()].op.is_phi()).count();
        self.f.blocks[b.index()].insts.insert(at, i);
        self.f.result(i)
    }

    /// Adds an entry to phi `phi`, made by `phi`.
    pub fn add_incoming(&mut self, phi: Value, v: Value, from: BlockId) {
        let Value::Local(l) = phi else { panic!("add_incoming: not a phi") };
        let LocalDef::Inst(i) = self.f.locals[l.index()].def else { panic!("add_incoming: not a phi") };
        let Op::Phi { incoming, .. } = &mut self.f.insts[i.index()].op else { panic!("add_incoming: not a phi") };
        incoming.push((v, from));
    }

    pub fn br(&mut self, dest: BlockId) {
        self.push(Op::Br(dest), None);
    }

    pub fn cond_br(&mut self, cond: Value, then_dest: BlockId, else_dest: BlockId) {
        self.push(Op::CondBr { cond, then_dest, else_dest }, None);
    }

    /// `switch v, default [cases]`, each case a constant of `v`'s type.
    pub fn switch(&mut self, v: Value, default: BlockId, cases: &[(i64, BlockId)]) {
        let ty = self.type_of(v);
        let cases = cases
            .iter()
            .map(|&(k, b)| {
                let c = Constant::int(&self.m.types, ty, k);
                (self.m.consts.intern(c), b)
            })
            .collect();
        self.push(Op::Switch { value: v, default, cases }, None);
    }

    pub fn ret(&mut self, v: Value) {
        let want = match self.m.types.get(self.f.ty) {
            Type::Function { ret, .. } => *ret,
            _ => unreachable!(),
        };
        let got = self.type_of(v);
        assert!(
            got == want,
            "ret: a {:?} from a function returning {:?}",
            self.m.types.get(got),
            self.m.types.get(want)
        );
        self.push(Op::Ret(Some(v)), None);
    }

    pub fn ret_void(&mut self) {
        self.push(Op::Ret(None), None);
    }

    pub fn unreachable(&mut self) {
        self.push(Op::Unreachable, None);
    }

    // Instructions with settings, chained ------------------------------------

    /// A binary operation with flags, chained: `b.build_binary(BinOp::Add,
    /// x, y).nsw().nuw().build()`.
    pub fn build_binary(&mut self, op: BinOp, a: Value, b: Value) -> BinarySpec<'_, 'm> {
        BinarySpec { b: self, op, lhs: a, rhs: b, flags: IntFlags::NONE }
    }

    /// A call of function `callee`, chained: `b.build_call(f).arg(x).arg(y)
    /// .tail().build()`.
    pub fn build_call(&mut self, callee: GlobalId) -> CallSpec<'_, 'm> {
        let GlobalKind::Function(f) = &self.m.global(callee).kind else { panic!("build_call: not a function") };
        let fn_ty = f.ty;
        let target = self.m.global_value(callee);
        CallSpec { b: self, fn_ty, target, args: vec![], tail: TailKind::None, attrs: vec![] }
    }

    /// A call through pointer `target`, of function type `fn_ty`, chained.
    pub fn build_call_indirect(&mut self, fn_ty: TypeId, target: Value) -> CallSpec<'_, 'm> {
        CallSpec { b: self, fn_ty, target, args: vec![], tail: TailKind::None, attrs: vec![] }
    }

    /// `load ty, ptr p`, chained: `.align(8).volatile().build()`.
    pub fn build_load(&mut self, ty: TypeId, p: Value) -> LoadSpec<'_, 'm> {
        LoadSpec { b: self, ty, ptr: p, align: None, volatile: false }
    }

    /// `store v, ptr p`, chained: `.align(8).volatile().build()`.
    pub fn build_store(&mut self, v: Value, p: Value) -> StoreSpec<'_, 'm> {
        StoreSpec { b: self, value: v, ptr: p, align: None, volatile: false }
    }

    /// `getelementptr source_ty, ptr p`, chained: `.index(i).inbounds()
    /// .build()`.
    pub fn build_gep(&mut self, source_ty: TypeId, p: Value) -> GepSpec<'_, 'm> {
        GepSpec { b: self, source_ty, ptr: p, indices: vec![], flags: 0 }
    }

    /// Checks that every block ends in a terminator and has no other, then
    /// adds the function to the module, in place of a declaration of the
    /// same name if there is one.
    pub fn finish(self) -> Result<GlobalId, String> {
        let f = self.f;
        if f.layout.is_empty() {
            return Err(format!("@{}: no blocks", self.name));
        }
        for &b in &f.layout {
            let insts = &f.blocks[b.index()].insts;
            let ends = insts.last().is_some_and(|i| f.insts[i.index()].op.is_terminator());
            if !ends {
                let name = f.block_name(b).unwrap_or("?").to_string();
                return Err(format!("@{}: block %{name} does not end in a terminator", self.name));
            }
        }
        match self.m.find_global(&self.name) {
            Some(g) => {
                let GlobalKind::Function(old) = &mut self.m.globals[g.index()].kind else {
                    return Err(format!("@{}: already a global that is not a function", self.name));
                };
                if old.has_body {
                    return Err(format!("@{}: already defined", self.name));
                }
                **old = f;
                self.m.globals[g.index()].props = self.props;
                Ok(g)
            }
            None => {
                let mut g = Global::function(&self.name, f);
                g.props = self.props;
                Ok(self.m.add_global(g))
            }
        }
    }
}

/// A binary operation's flags, then `build`.
pub struct BinarySpec<'b, 'm> {
    b: &'b mut FunctionBuilder<'m>,
    op: BinOp,
    lhs: Value,
    rhs: Value,
    flags: IntFlags,
}

impl BinarySpec<'_, '_> {
    /// `nsw`: signed overflow is poison.
    pub fn nsw(mut self) -> Self {
        self.flags = self.flags | IntFlags::NSW;
        self
    }
    /// `nuw`: unsigned overflow is poison.
    pub fn nuw(mut self) -> Self {
        self.flags = self.flags | IntFlags::NUW;
        self
    }
    /// `exact`: a division or right shift that loses no bits.
    pub fn exact(mut self) -> Self {
        self.flags = self.flags | IntFlags::EXACT;
        self
    }
    pub fn build(self) -> Value {
        assert!(self.op.allowed_flags().contains(self.flags), "{}: flags it does not take", self.op.keyword());
        self.b.binary_with(self.op, self.flags, self.lhs, self.rhs)
    }
}

/// A call's arguments and settings, then `build`.
pub struct CallSpec<'b, 'm> {
    b: &'b mut FunctionBuilder<'m>,
    fn_ty: TypeId,
    target: Value,
    args: Vec<Value>,
    tail: TailKind,
    attrs: Vec<Attr>,
}

impl CallSpec<'_, '_> {
    pub fn arg(mut self, v: Value) -> Self {
        self.args.push(v);
        self
    }
    pub fn args(mut self, vs: &[Value]) -> Self {
        self.args.extend_from_slice(vs);
        self
    }
    /// `tail`: the callee does not use the caller's stack.
    pub fn tail(mut self) -> Self {
        self.tail = TailKind::Tail;
        self
    }
    /// `musttail`
    pub fn must_tail(mut self) -> Self {
        self.tail = TailKind::MustTail;
        self
    }
    /// A keyword attribute on the call (`nounwind`).
    pub fn attr(mut self, word: &str) -> Self {
        self.attrs.push(Attr::Word(word.to_string()));
        self
    }
    /// Adds the call; its value, unless it returns `void`.
    pub fn build(self) -> Option<Value> {
        let r = self.b.call_indirect(self.fn_ty, self.target, &self.args);
        let b = self.b.cur.unwrap();
        let i = *self.b.f.blocks[b.index()].insts.last().unwrap();
        if let Op::Call(c) = &mut self.b.f.insts[i.index()].op {
            c.tail = self.tail;
            c.fn_attrs = self.attrs;
        }
        r
    }
}

/// A load's alignment and volatility, then `build`.
pub struct LoadSpec<'b, 'm> {
    b: &'b mut FunctionBuilder<'m>,
    ty: TypeId,
    ptr: Value,
    align: Option<Align>,
    volatile: bool,
}

impl LoadSpec<'_, '_> {
    pub fn align(mut self, align: u64) -> Self {
        self.align = Some(Align::new(align).expect("an alignment must be a power of two"));
        self
    }
    pub fn volatile(mut self) -> Self {
        self.volatile = true;
        self
    }
    pub fn build(self) -> Value {
        let op = Op::Load { volatile: self.volatile, atomic: None, ptr: self.ptr, align: self.align };
        self.b.value(op, self.ty)
    }
}

/// A store's alignment and volatility, then `build`.
pub struct StoreSpec<'b, 'm> {
    b: &'b mut FunctionBuilder<'m>,
    value: Value,
    ptr: Value,
    align: Option<Align>,
    volatile: bool,
}

impl StoreSpec<'_, '_> {
    pub fn align(mut self, align: u64) -> Self {
        self.align = Some(Align::new(align).expect("an alignment must be a power of two"));
        self
    }
    pub fn volatile(mut self) -> Self {
        self.volatile = true;
        self
    }
    pub fn build(self) {
        let op =
            Op::Store { volatile: self.volatile, atomic: None, value: self.value, ptr: self.ptr, align: self.align };
        self.b.push(op, None);
    }
}

/// A `getelementptr`'s indices and flags, then `build`.
pub struct GepSpec<'b, 'm> {
    b: &'b mut FunctionBuilder<'m>,
    source_ty: TypeId,
    ptr: Value,
    indices: Vec<Value>,
    flags: u8,
}

impl GepSpec<'_, '_> {
    pub fn index(mut self, v: Value) -> Self {
        self.indices.push(v);
        self
    }
    /// A constant `i64` index.
    pub fn const_index(mut self, k: i64) -> Self {
        let i64t = self.b.m.types.int(64);
        let v = self.b.m.const_int(i64t, k);
        self.indices.push(v);
        self
    }
    /// `inbounds`: the address stays within the object.
    pub fn inbounds(mut self) -> Self {
        self.flags |= GepFlags::INBOUNDS;
        self
    }
    /// `nuw`: the offset additions do not wrap.
    pub fn nuw(mut self) -> Self {
        self.flags |= GepFlags::NUW;
        self
    }
    pub fn build(self) -> Value {
        let ptr = self.b.m.types.ptr(0);
        let op = Op::Gep {
            flags: GepFlags(self.flags),
            source_ty: self.source_ty,
            ptr: self.ptr,
            indices: self.indices.into(),
        };
        self.b.value(op, ptr)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A loop with a phi, a call to a declared function and a store: the
    /// built module prints as the IR it stands for, and parses back the same.
    #[test]
    fn a_loop_prints_and_parses_back() {
        let mut m = Module::default();
        let i32t = m.types.int(32);
        let ptr = m.types.ptr(0);
        let void = m.types.void();
        let report = m.declare("report", void, &[i32t], false);
        let mut b = m.function("sum_to", i32t, &[("n", i32t), ("out", ptr)]);
        let (n, out) = (b.param(0), b.param(1));
        let entry = b.block("entry");
        let lp = b.new_block("loop");
        let done = b.new_block("done");
        b.br(lp);
        b.at(lp);
        let zero = b.const_int(i32t, 0);
        let i = b.name("i").phi(i32t, &[(zero, entry)]);
        let s = b.name("s").phi(i32t, &[(zero, entry)]);
        let one = b.const_int(i32t, 1);
        let i1 = b.name("i1").add(i, one);
        let s1 = b.name("s1").add(s, i1);
        b.add_incoming(i, i1, lp);
        b.add_incoming(s, s1, lp);
        let more = b.name("more").icmp(ICmpPred::Slt, i1, n);
        b.cond_br(more, lp, done);
        b.at(done);
        b.store(s1, out);
        b.call(report, &[s1]);
        b.ret(s1);
        b.finish().unwrap();
        let text = m.to_string();
        let want = "define i32 @sum_to(i32 %n, ptr %out) {\n\
                    entry:\n  br label %loop\n\n\
                    loop:                                             ; preds = %loop, %entry\n\
                    \x20 %i = phi i32 [ 0, %entry ], [ %i1, %loop ]\n\
                    \x20 %s = phi i32 [ 0, %entry ], [ %s1, %loop ]\n\
                    \x20 %i1 = add i32 %i, 1\n  %s1 = add i32 %s, %i1\n\
                    \x20 %more = icmp slt i32 %i1, %n\n  br i1 %more, label %loop, label %done\n\n\
                    done:                                             ; preds = %loop\n\
                    \x20 store i32 %s1, ptr %out\n  call void @report(i32 %s1)\n  ret i32 %s1\n}";
        assert!(text.contains(want), "{text}");
        let back = crate::parse_module(&text).unwrap();
        assert_eq!(back.to_string(), text);
    }

    #[test]
    #[should_panic(expected = "operands of different types")]
    fn mismatched_operands_panic() {
        let mut m = Module::default();
        let (i32t, i64t) = (m.types.int(32), m.types.int(64));
        let mut b = m.function("f", i32t, &[("a", i32t), ("b", i64t)]);
        b.block("entry");
        let (x, y) = (b.param(0), b.param(1));
        b.add(x, y);
    }

    #[test]
    fn a_block_without_a_terminator_is_an_error() {
        let mut m = Module::default();
        let i32t = m.types.int(32);
        let mut b = m.function("f", i32t, &[("a", i32t)]);
        b.block("entry");
        let x = b.param(0);
        b.add(x, x);
        assert!(b.finish().unwrap_err().contains("does not end in a terminator"));
    }

    /// The chained builders: a function's linkage and attributes, a
    /// global, a call's settings, a load's and a store's, a gep's, a
    /// binary operation's flags.
    #[test]
    fn chained_settings_print() {
        let mut m = Module::default();
        let (i32t, i64t, ptr) = (m.types.int(32), m.types.int(64), m.types.ptr(0));
        let arr = m.types.array(4, i32t);
        let zero = m.const_zero(arr);
        let table = Global::builder("table").ty(arr).init(zero).linkage(Linkage::Internal).align(16).build(&mut m);
        let log = Function::builder("log").param("v", i32t).param_attr("noundef").declare(&mut m);
        let mut b = Function::builder("get")
            .returns(i32t)
            .param("k", i64t)
            .linkage(Linkage::Internal)
            .attr("nounwind")
            .begin(&mut m);
        let k = b.param(0);
        b.block("entry");
        let t = b.module().global_value(table);
        let at = b.name("at").build_gep(arr, t).const_index(0).index(k).inbounds().build();
        let v = b.name("v").build_load(i32t, at).align(4).volatile().build();
        let one = b.const_int(i32t, 1);
        let w = b.name("w").build_binary(BinOp::Add, v, one).nsw().build();
        b.build_store(w, at).align(4).build();
        b.build_call(log).arg(w).tail().build();
        b.ret(w);
        b.finish().unwrap();
        let _ = ptr;
        let text = m.to_string();
        for want in [
            "@table = internal global [4 x i32] zeroinitializer, align 16",
            "declare void @log(i32 noundef)",
            "define internal i32 @get(i64 %k) nounwind {",
            "%at = getelementptr inbounds [4 x i32], ptr @table, i64 0, i64 %k",
            "%v = load volatile i32, ptr %at, align 4",
            "%w = add nsw i32 %v, 1",
            "store i32 %w, ptr %at, align 4",
            "tail call void @log(i32 %w)",
        ] {
            assert!(text.contains(want), "no {want:?} in\n{text}");
        }
        assert_eq!(crate::parse_module(&text).unwrap().to_string(), text);
    }

    #[test]
    fn a_declaration_is_replaced_by_its_definition() {
        let mut m = Module::default();
        let i64t = m.types.int(64);
        let g = m.declare("twice", i64t, &[i64t], false);
        let mut b = m.function("twice", i64t, &[("x", i64t)]);
        b.block("entry");
        let x = b.param(0);
        let r = b.add(x, x);
        b.ret(r);
        assert_eq!(b.finish().unwrap(), g);
        assert!(m.to_string().contains("define i64 @twice(i64 %x)"));
    }
}
