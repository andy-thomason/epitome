//! Level 0: a module in the `llvm` dialect translated to Epitome's LLVM IR,
//! as `mlir-translate --mlir-to-llvmir` does. See `docs/design/mlir.md`,
//! section 5.1, and `docs/mlir/03-llvm-dialect.md`.
//!
//! Each `llvm` op is one LLVM instruction, intrinsic call or module-level
//! construct. The differences from LLVM IR are few:
//!
//! - Block arguments become phis. A branch that passes values to a block
//!   adds an incoming entry to each of its phis. When one terminator
//!   reaches the same block twice, the second edge goes through a new block,
//!   so that each phi has one entry per predecessor.
//! - Constants are ops (`llvm.mlir.constant`, `llvm.mlir.zero`,
//!   `llvm.mlir.addressof` and so on). They become constant operands.
//! - An op whose operands are all constants is folded into a constant, as
//!   LLVM's IR builder does: arithmetic on integers and floats, compares,
//!   casts, `getelementptr`, `insertvalue` and `extractvalue`. A global's
//!   initialiser region must fold completely.
//! - Dialect types are kept as text (`!llvm.struct<(ptr, i64)>`), so this
//!   module has a small parser for their bodies.
//!
//! The generic form keeps enums as integers. The encodings used here follow
//! LLVM's own enums, checked against the tools' output in
//! `docs/mlir/examples`: `icmp` predicates `eq ne slt sle sgt sge ult ule ugt
//! uge` from 0; `fcmp` predicates in `FCmpPred::ALL` order; atomic orderings
//! 0 (none), 1 unordered, 2 monotonic, 4 acquire, 5 release, 6 acq_rel, 7
//! seq_cst; overflow flags `nsw` = 1, `nuw` = 2; `getelementptr` flags
//! `inbounds` = 1, `nusw` = 2, `nuw` = 4.
//!
//! Not translated: `dlti.dl_spec` (only `llvm.data_layout` gives the data
//! layout), metadata, debug information, comdats, aliases, atomics other
//! than loads and stores, `invoke` and landing pads, inline assembly, and
//! operand bundles. They are reported as errors, or ignored where they do
//! not change the program.

use std::collections::{HashMap, HashSet};

use epitome_ir::apint::ApInt;
use epitome_ir::float::{self, FloatKind};
use epitome_ir::layout::DataLayout;
use epitome_ir::module as ll;
use epitome_ir::module::{
    BinOp, CastOp, ConstExpr, ConstId, ConstKind, Constant, FCmpPred, FastMath, GepFlags, GlobalId, ICmpPred, IntFlags,
    IntStyle, Linkage, Op,
};
use epitome_ir::types::{StructBody, Type as LType, TypeId as LTypeId};

use crate::ir::{Attr, AttrId, BlockId, Module, OpId, Type, TypeId, ValueId};
use crate::printer::type_text;

type Result<T> = std::result::Result<T, String>;

/// Translates a module in the `llvm` dialect to LLVM IR, as `mlir-translate --mlir-to-llvmir` does.
pub fn translate(m: &crate::Module) -> Result<ll::Module> {
    let tops: Vec<OpId> = m.top_ops().collect();
    let (module_op, body) = match tops.as_slice() {
        [op] if m.op_name(*op) == "builtin.module" => (Some(*op), region_ops(m, *op)),
        _ => (None, tops),
    };
    let mut t = Translator::new(m);
    if let Some(op) = module_op {
        if let Some(dl) = t.str_attr(op, "llvm.data_layout") {
            t.dl = DataLayout::parse(&dl).unwrap_or_default();
            t.out.data_layout = Some(dl.into_bytes());
        }
        if let Some(triple) = t.str_attr(op, "llvm.target_triple") {
            t.out.triple = Some(triple.into_bytes());
        }
    }
    // Globals first, then functions, as the tools print them. Everything is
    // declared before any initialiser or body is translated, so that any
    // symbol can refer to any other.
    for &op in &body {
        match m.op_name(op) {
            "llvm.mlir.global" => t.declare_global(op)?,
            "llvm.func" => {}
            name => return Err(format!("{name}: not supported yet")),
        }
    }
    for &op in &body {
        if m.op_name(op) == "llvm.func" {
            t.declare_function(op)?;
        }
    }
    for &op in &body {
        let name = t.str_attr(op, "sym_name").unwrap_or_default();
        let r = match m.op_name(op) {
            "llvm.mlir.global" => t.global_init(op),
            _ => t.define_function(op),
        };
        r.map_err(|e| format!("@{name}: {e}"))?;
    }
    Ok(t.out)
}

/// The ops of the first block of an op's first region.
fn region_ops(m: &Module, op: OpId) -> Vec<OpId> {
    let o = &m.ops[op.index()];
    o.regions
        .first()
        .and_then(|r| m.regions[r.index()].blocks.first())
        .map(|b| m.blocks[b.index()].ops.clone())
        .unwrap_or_default()
}

/// The predicates of `llvm.icmp`, by their number in the generic form.
const ICMP_PREDS: [ICmpPred; 10] = [
    ICmpPred::Eq,
    ICmpPred::Ne,
    ICmpPred::Slt,
    ICmpPred::Sle,
    ICmpPred::Sgt,
    ICmpPred::Sge,
    ICmpPred::Ult,
    ICmpPred::Ule,
    ICmpPred::Ugt,
    ICmpPred::Uge,
];

/// The dynamic-index marker in `rawConstantIndices`.
const DYNAMIC_INDEX: i64 = i32::MIN as i64;

/// The unit attributes of `llvm.func` that are LLVM function attributes
/// under another name.
const FUNC_FLAGS: &[(&str, &str)] = &[
    ("no_inline", "noinline"),
    ("always_inline", "alwaysinline"),
    ("inline_hint", "inlinehint"),
    ("no_unwind", "nounwind"),
    ("will_return", "willreturn"),
    ("noreturn", "noreturn"),
    ("optimize_none", "optnone"),
    ("returns_twice", "returns_twice"),
    ("hot", "hot"),
    ("cold", "cold"),
    ("noduplicate", "noduplicate"),
    ("convergent", "convergent"),
    ("optsize", "optsize"),
    ("minsize", "minsize"),
];

/// LLVM's keyword function attributes that `passthrough` may name; any
/// other name becomes a string attribute.
const KEYWORD_ATTRS: &[&str] = &[
    "alwaysinline",
    "builtin",
    "cold",
    "convergent",
    "hot",
    "inlinehint",
    "minsize",
    "mustprogress",
    "naked",
    "nobuiltin",
    "nocallback",
    "noduplicate",
    "nofree",
    "noimplicitfloat",
    "noinline",
    "nomerge",
    "nonlazybind",
    "noredzone",
    "norecurse",
    "noreturn",
    "nosync",
    "nounwind",
    "null_pointer_is_valid",
    "optnone",
    "optsize",
    "readnone",
    "readonly",
    "returns_twice",
    "speculatable",
    "ssp",
    "sspreq",
    "sspstrong",
    "strictfp",
    "willreturn",
    "writeonly",
];

/// Intrinsics overloaded on their result type alone.
const BY_RESULT: &[&str] = &[
    "fabs",
    "sqrt",
    "sin",
    "cos",
    "tan",
    "asin",
    "acos",
    "atan",
    "atan2",
    "sinh",
    "cosh",
    "tanh",
    "exp",
    "exp2",
    "exp10",
    "log",
    "log2",
    "log10",
    "pow",
    "fma",
    "fmuladd",
    "copysign",
    "floor",
    "ceil",
    "trunc",
    "round",
    "roundeven",
    "rint",
    "nearbyint",
    "minnum",
    "maxnum",
    "minimum",
    "maximum",
    "minimumnum",
    "maximumnum",
    "canonicalize",
    "arithmetic.fence",
    "smin",
    "smax",
    "umin",
    "umax",
    "bitreverse",
    "bswap",
    "ctpop",
    "fshl",
    "fshr",
    "sadd.sat",
    "uadd.sat",
    "ssub.sat",
    "usub.sat",
    "sshl.sat",
    "ushl.sat",
    "expect",
    "expect.with.probability",
    "ssa.copy",
    "vscale",
    "stepvector",
    "stacksave",
    "threadlocal.address",
    "launder.invariant.group",
];

/// Intrinsics that are not overloaded.
const NOT_OVERLOADED: &[&str] = &["assume", "trap", "debugtrap", "donothing"];

/// The function being translated, or the initialiser region of a global,
/// which has no function and whose every op must fold.
struct Body {
    f: Option<ll::Function>,
    values: HashMap<ValueId, ll::Value>,
    blocks: HashMap<BlockId, ll::BlockId>,
    /// The phis that stand for each block's arguments.
    phis: HashMap<BlockId, Vec<ll::InstId>>,
    /// The block instructions go into.
    cur: ll::BlockId,
    /// What an initialiser region returns.
    ret: Option<ll::Value>,
}

impl Body {
    fn new(f: Option<ll::Function>) -> Body {
        Body { f, values: HashMap::new(), blocks: HashMap::new(), phis: HashMap::new(), cur: ll::BlockId(0), ret: None }
    }

    fn val(&self, v: ValueId) -> Result<ll::Value> {
        self.values.get(&v).copied().ok_or_else(|| format!("value {} used before it is defined", v.0))
    }

    fn func(&mut self) -> Result<&mut ll::Function> {
        self.f.as_mut().ok_or_else(|| "not a constant expression".to_string())
    }
}

struct Translator<'a> {
    m: &'a Module,
    out: ll::Module,
    dl: DataLayout,
    symbols: HashMap<String, GlobalId>,
    /// The `dialect_resources` blobs, by key.
    resources: crate::ir::Resources<'a>,
}

impl<'a> Translator<'a> {
    fn new(m: &'a Module) -> Translator<'a> {
        let resources = m.resources();
        let out = ll::Module { source_filename: Some(b"LLVMDialectModule".to_vec()), ..Default::default() };
        Translator { m, out, dl: DataLayout::default(), symbols: HashMap::new(), resources }
    }

    // -----------------------------------------------------------------------
    // Attributes

    fn attr(&self, op: OpId, key: &str) -> Option<&'a Attr> {
        let m = self.m;
        m.op_attr(op, key).map(|a| m.attr(m.resolve_attr(a)))
    }

    fn str_attr(&self, op: OpId, key: &str) -> Option<String> {
        match self.attr(op, key)? {
            Attr::String { bytes, .. } => Some(String::from_utf8_lossy(bytes).into_owned()),
            _ => None,
        }
    }

    fn int_attr(&self, op: OpId, key: &str) -> Option<i64> {
        match self.attr(op, key)? {
            Attr::Int { text, .. } => int_text(text),
            Attr::Bool(b) => Some(*b as i64),
            _ => None,
        }
    }

    fn unit_attr(&self, op: OpId, key: &str) -> bool {
        matches!(self.attr(op, key), Some(Attr::Unit | Attr::Bool(true)))
    }

    /// The text inside the brackets of a dialect attribute such as
    /// `#llvm.linkage<internal>`.
    fn dialect_attr(&self, op: OpId, key: &str) -> Option<&'a str> {
        match self.attr(op, key)? {
            Attr::Dialect { body: Some(b), .. } => Some(strip_brackets(b)),
            _ => None,
        }
    }

    fn ints_attr(&self, op: OpId, key: &str) -> Option<Vec<i64>> {
        match self.attr(op, key)? {
            Attr::Builtin { keyword: "array", body, .. } => array_ints(body),
            Attr::Builtin { keyword: "dense", body, .. } => {
                dense_elems(body).ok()?.into_iter().map(|t| int_text(&t)).collect()
            }
            _ => None,
        }
    }

    fn type_attr(&mut self, op: OpId, key: &str) -> Result<Option<LTypeId>> {
        match self.attr(op, key) {
            Some(Attr::Type(t)) => Ok(Some(self.ty(*t)?)),
            Some(_) => Err(format!("{key}: not a type")),
            None => Ok(None),
        }
    }

    fn linkage(&self, op: OpId) -> Result<Linkage> {
        match self.dialect_attr(op, "linkage") {
            None => Ok(Linkage::External),
            Some(word) => Linkage::ALL
                .iter()
                .find(|(k, _)| *k == word)
                .map(|(_, l)| *l)
                .ok_or_else(|| format!("unknown linkage {word}")),
        }
    }

    fn cconv(&self, op: OpId) -> ll::CallConv {
        match self.dialect_attr(op, "CConv") {
            None | Some("ccc") => ll::CallConv::C,
            Some(cc) => match cc.strip_prefix("cc_").and_then(|n| n.parse().ok()) {
                Some(n) => ll::CallConv::Numbered(n),
                None => ll::CallConv::Named(cc.to_string()),
            },
        }
    }

    fn fastmath(&self, op: OpId) -> FastMath {
        let Some(body) = self.dialect_attr(op, "fastmathFlags") else { return FastMath::default() };
        let mut bits = 0;
        for word in body.split(',').map(str::trim) {
            if word == "fast" {
                bits = FastMath::ALL.0;
            } else if let Some((_, b)) = FastMath::NAMES.iter().find(|(n, _)| *n == word) {
                bits |= b;
            }
        }
        FastMath(bits)
    }

    /// `overflowFlags`, `isExact`, `isDisjoint` and `nonNeg`.
    fn int_flags(&self, op: OpId) -> IntFlags {
        let mut flags = IntFlags::NONE;
        let overflow = self.int_attr(op, "overflowFlags").unwrap_or(0);
        if overflow & 1 != 0 {
            flags = flags | IntFlags::NSW;
        }
        if overflow & 2 != 0 {
            flags = flags | IntFlags::NUW;
        }
        for (keys, flag) in [
            (["isExact", "exact"], IntFlags::EXACT),
            (["isDisjoint", "disjoint"], IntFlags::DISJOINT),
            (["nonNeg", "nneg"], IntFlags::NNEG),
        ] {
            if keys.iter().any(|k| self.unit_attr(op, k)) {
                flags = flags | flag;
            }
        }
        flags
    }

    /// LLVM attributes from an `arg_attrs` or `res_attrs` dictionary.
    fn value_attrs(&mut self, dict: AttrId) -> Vec<ll::Attr> {
        let m = self.m;
        let Attr::Dict(entries) = m.attr(m.resolve_attr(dict)) else { return vec![] };
        let mut out = Vec::new();
        for (key, value) in entries {
            let name = key.strip_prefix("llvm.").unwrap_or(key);
            match m.attr(*value) {
                Attr::Unit if name == "nocapture" => {
                    out.push(ll::Attr::Parens { name: "captures".into(), text: "none".into() })
                }
                Attr::Unit => out.push(ll::Attr::Word(name.to_string())),
                Attr::Int { text, .. } => {
                    let Some(v) = int_text(text) else { continue };
                    let style = match name {
                        "align" => IntStyle::Space,
                        "alignstack" => IntStyle::Eq,
                        _ => IntStyle::Paren,
                    };
                    out.push(ll::Attr::Int { name: name.to_string(), value: v as u64, style });
                }
                Attr::Type(t) => {
                    if let Ok(ty) = self.ty(*t) {
                        out.push(ll::Attr::Type { name: name.to_string(), ty });
                    }
                }
                _ => {}
            }
        }
        out
    }

    /// The parts of a global's properties that functions and variables share.
    fn props(&self, op: OpId) -> Result<ll::GlobalProps> {
        let linkage = self.linkage(op)?;
        let visibility = match self.int_attr(op, "visibility_").unwrap_or(0) {
            1 => ll::Visibility::Hidden,
            2 => ll::Visibility::Protected,
            _ => ll::Visibility::Default,
        };
        let unnamed_addr = match self.int_attr(op, "unnamed_addr").unwrap_or(0) {
            1 => ll::UnnamedAddr::Local,
            2 => ll::UnnamedAddr::Global,
            _ => ll::UnnamedAddr::None,
        };
        Ok(ll::GlobalProps {
            linkage,
            explicit_linkage: linkage != Linkage::External,
            dso_local: self.unit_attr(op, "dso_local"),
            visibility,
            unnamed_addr,
            ..Default::default()
        })
    }

    // -----------------------------------------------------------------------
    // Types

    fn ty(&mut self, t: TypeId) -> Result<LTypeId> {
        let m = self.m;
        let t = m.resolve_type(t);
        let types = &mut self.out.types;
        Ok(match m.ty(t) {
            Type::Int { width, .. } => types.int(*width),
            Type::Index => types.int(64),
            Type::Float(k) => match float_type(k) {
                Some(f) => types.intern(f),
                None => return Err(format!("type {k} has no LLVM equivalent")),
            },
            Type::Vector { dims, elem } if dims.len() == 1 => {
                let elem = self.ty(*elem)?;
                let (len, scalable) = dims[0];
                self.out.types.intern(LType::Vector { scalable, len: len as u32, elem })
            }
            Type::Dialect { name, body } if name.starts_with("llvm.") => {
                let text = format!("{}{}", &name[5..], body.as_deref().unwrap_or(""));
                let mut c = Cursor { s: &text, at: 0 };
                let ty = self.text_type(&mut c)?;
                c.ws();
                if c.at != text.len() {
                    return Err(format!("cannot read type !{name}{}", body.as_deref().unwrap_or("")));
                }
                ty
            }
            _ => return Err(format!("type {} has no LLVM equivalent", type_text(m, t))),
        })
    }

    /// Parses a type written inside a dialect type's body, such as
    /// `struct<(ptr, array<2 x i64>)>`.
    fn text_type(&mut self, c: &mut Cursor) -> Result<LTypeId> {
        c.ws();
        c.eat("!");
        c.eat("llvm.");
        let word = c.word();
        let types = &mut self.out.types;
        let ty = match word {
            "ptr" => {
                let space = if c.eat("<") {
                    let n = c.number()?;
                    c.expect(">")?;
                    n as u32
                } else {
                    0
                };
                types.ptr(space)
            }
            "void" => types.void(),
            "metadata" => types.metadata(),
            "token" => types.token(),
            "index" => types.int(64),
            "ppc_fp128" => types.intern(LType::PpcFp128),
            "x86_amx" => types.intern(LType::X86Amx),
            "array" => {
                c.expect("<")?;
                let len = c.number()?;
                c.expect("x")?;
                let elem = self.text_type(c)?;
                c.expect(">")?;
                self.out.types.array(len, elem)
            }
            "vector" => {
                c.expect("<")?;
                let scalable = c.eat("[");
                let len = c.number()?;
                if scalable {
                    c.expect("]")?;
                }
                c.expect("x")?;
                let elem = self.text_type(c)?;
                c.expect(">")?;
                self.out.types.intern(LType::Vector { scalable, len: len as u32, elem })
            }
            "struct" => self.text_struct(c)?,
            "func" => {
                c.expect("<")?;
                let ret = self.text_type(c)?;
                c.expect("(")?;
                let mut params = Vec::new();
                let mut varargs = false;
                if !c.eat(")") {
                    loop {
                        if c.eat("...") {
                            varargs = true;
                        } else {
                            params.push(self.text_type(c)?);
                        }
                        if c.eat(")") {
                            break;
                        }
                        c.expect(",")?;
                    }
                }
                c.expect(">")?;
                self.out.types.function(ret, &params, varargs)
            }
            w => {
                if let Some(bits) = w.strip_prefix('i').and_then(|n| n.parse().ok()) {
                    types.int(bits)
                } else if let Some(f) = float_type(w) {
                    types.intern(f)
                } else {
                    return Err(format!("cannot read type '{w}'"));
                }
            }
        };
        Ok(ty)
    }

    /// The rest of `struct<...>`: literal, packed or identified.
    fn text_struct(&mut self, c: &mut Cursor) -> Result<LTypeId> {
        c.expect("<")?;
        c.ws();
        let name = if c.peek() == Some('"') { Some(c.string()?) } else { None };
        if let Some(name) = &name {
            let ty = self.out.types.named_struct(name);
            if c.eat(">") {
                return Ok(ty);
            }
            c.expect(",")?;
            if c.eat("opaque") {
                c.expect(">")?;
                return Ok(ty);
            }
            let (packed, fields) = self.text_fields(c)?;
            c.expect(">")?;
            let LType::Named(i) = *self.out.types.get(ty) else { unreachable!() };
            let s = &mut self.out.types.structs[i as usize];
            if s.body.is_none() {
                s.body = Some(StructBody { packed, fields });
            }
            return Ok(ty);
        }
        let (packed, fields) = self.text_fields(c)?;
        c.expect(">")?;
        Ok(self.out.types.intern(LType::Struct { packed, fields }))
    }

    /// `(T, U)` or `packed (T, U)`.
    fn text_fields(&mut self, c: &mut Cursor) -> Result<(bool, Vec<LTypeId>)> {
        let packed = c.eat("packed");
        c.expect("(")?;
        let mut fields = Vec::new();
        if !c.eat(")") {
            loop {
                fields.push(self.text_type(c)?);
                if c.eat(")") {
                    break;
                }
                c.expect(",")?;
            }
        }
        Ok((packed, fields))
    }

    /// The type of a translated value.
    fn vty(&self, body: &Body, v: ll::Value) -> LTypeId {
        match v {
            ll::Value::Local(l) => body.f.as_ref().unwrap().locals[l.index()].ty,
            ll::Value::Const(c) => self.out.consts.get(c).ty,
            ll::Value::Metadata(_) => unreachable!("no metadata values are made"),
        }
    }

    // -----------------------------------------------------------------------
    // Constants

    fn konst(&mut self, ty: LTypeId, kind: ConstKind) -> ConstId {
        self.out.consts.intern(Constant { ty, kind })
    }

    fn int_const(&mut self, ty: LTypeId, v: u64) -> ConstId {
        let LType::Int(bits) = *self.out.types.get(ty) else { panic!("not an integer type") };
        self.konst(ty, ConstKind::Int(ApInt::from_u64(bits, v)))
    }

    /// A scalar of type `ty` from its text in an attribute.
    fn scalar(&mut self, ty: LTypeId, text: &str) -> Result<ConstId> {
        let bad = || format!("cannot read {text:?} as a constant");
        let kind = match self.out.types.get(ty) {
            &LType::Int(bits) => ConstKind::Int(match text {
                "true" => ApInt::from_u64(bits, 1),
                "false" => ApInt::from_u64(bits, 0),
                _ => match text.strip_prefix("0x") {
                    Some(hex) => ApInt::parse_hex(bits, hex, false),
                    None => ApInt::parse_decimal(bits, text),
                }
                .ok_or_else(bad)?,
            }),
            LType::Ptr(_) if text == "0" => ConstKind::Null,
            t => {
                let kind = FloatKind::of(t).ok_or_else(bad)?;
                // MLIR writes a float in hex as the bits of its own type.
                let bits = match text.strip_prefix("0x") {
                    Some(hex) => u128::from_str_radix(hex, 16).ok(),
                    None => float::parse(kind, text),
                };
                ConstKind::Float(bits.ok_or_else(bad)?)
            }
        };
        Ok(self.konst(ty, kind))
    }

    /// A constant of type `ty` from an attribute: a scalar, a string, a
    /// `dense` or `dense_resource` aggregate, or an array of attributes for
    /// a struct or array.
    fn attr_const(&mut self, attr: AttrId, ty: LTypeId) -> Result<ConstId> {
        let m = self.m;
        let attr = m.attr(m.resolve_attr(attr));
        match attr {
            Attr::Int { text, .. } | Attr::Float { text, .. } => self.leaves(ty, Elems::Text(vec![text.clone()])),
            Attr::Bool(b) => self.leaves(ty, Elems::Text(vec![b.to_string()])),
            Attr::String { bytes, .. } => match *self.out.types.get(ty) {
                LType::Array { len, elem }
                    if len == bytes.len() as u64 && *self.out.types.get(elem) == LType::Int(8) =>
                {
                    Ok(self.konst(ty, ConstKind::Bytes(bytes.clone())))
                }
                _ => Err("a string for a type that is not an array of i8 of its length".into()),
            },
            Attr::Builtin { keyword: "dense", body, .. } => {
                let text = strip_brackets(body).trim();
                let elems = match text.strip_prefix("\"0x").and_then(|t| t.strip_suffix('"')) {
                    Some(hex) => Elems::Raw(hex_bytes(hex)?),
                    None => Elems::Text(dense_elems(body)?),
                };
                self.leaves(ty, elems)
            }
            Attr::Builtin { keyword: "dense_resource", body, .. } => {
                let key = strip_brackets(body).trim();
                let data = self.resources.get(key).ok_or_else(|| format!("no resource {key}"))?.into_owned();
                self.leaves(ty, Elems::Raw(data))
            }
            Attr::Array(items) => {
                let mut members = Vec::new();
                for (k, &item) in items.iter().enumerate() {
                    let mty = self.out.types.member(ty, k as u64).ok_or("too many members for the type")?;
                    members.push(self.attr_const(item, mty)?);
                }
                let kind = match self.out.types.get(ty) {
                    LType::Struct { .. } | LType::Named(_) => ConstKind::Struct(members),
                    LType::Vector { .. } => ConstKind::Vector(members),
                    _ => ConstKind::Array(members),
                };
                Ok(self.konst(ty, kind))
            }
            Attr::Dialect { name, .. } if name == "llvm.zero" => Ok(self.zero(ty)),
            Attr::Dialect { name, .. } if name == "llvm.undef" => Ok(self.konst(ty, ConstKind::Undef)),
            Attr::Dialect { name, .. } if name == "llvm.poison" => Ok(self.konst(ty, ConstKind::Poison)),
            _ => Err("an attribute that is not a constant".into()),
        }
    }

    fn zero(&mut self, ty: LTypeId) -> ConstId {
        let kind = if self.out.types.is_ptr(ty) { ConstKind::Null } else { ConstKind::Zero };
        self.konst(ty, kind)
    }

    /// An array, vector or scalar of type `ty` filled, in row-major order,
    /// with the given elements; one element is a splat.
    fn leaves(&mut self, ty: LTypeId, elems: Elems) -> Result<ConstId> {
        let mut leaf = ty;
        let mut count = 1u64;
        while let Some((len, elem)) = self.leaf_step(leaf) {
            count *= len;
            leaf = elem;
        }
        // An array of numbers filled with raw data: the data as it is, in
        // memory order (a model's weights: millions of elements).
        if let Elems::Raw(bytes) = &elems
            && count > 1
            && matches!(self.out.types.get(ty), LType::Array { .. })
        {
            let bits = match self.out.types.get(leaf) {
                LType::Int(b) if *b >= 8 && b % 8 == 0 => Some(*b),
                t => FloatKind::of(t).map(|k| k.bits()),
            };
            if let Some(bits) = bits
                && bytes.len() as u64 == count * (bits as u64 / 8)
            {
                let bytes = bytes.clone();
                return Ok(self.konst(ty, ConstKind::Bytes(bytes)));
            }
        }
        let consts: Vec<ConstId> = match elems {
            Elems::Text(texts) => {
                let mut out = Vec::new();
                for t in &texts {
                    out.push(self.scalar(leaf, t)?);
                }
                out
            }
            Elems::Raw(bytes) => {
                let bits = match self.out.types.get(leaf) {
                    LType::Int(b) if *b >= 8 => *b,
                    t => FloatKind::of(t).map(|k| k.bits()).ok_or("raw data for a type it cannot fill")?,
                };
                let size = bits.div_ceil(8) as usize;
                if size > 16 || !bytes.len().is_multiple_of(size) {
                    return Err("raw data of the wrong size".into());
                }
                let mut out = Vec::new();
                for chunk in bytes.chunks(size) {
                    let mut v = 0u128;
                    for (k, &b) in chunk.iter().enumerate() {
                        v |= (b as u128) << (8 * k);
                    }
                    let kind = match *self.out.types.get(leaf) {
                        LType::Int(b) if b <= 64 => ConstKind::Int(ApInt::from_u64(b, v as u64)),
                        LType::Int(_) => return Err("raw data for integers wider than 64 bits".into()),
                        _ => ConstKind::Float(v),
                    };
                    out.push(self.konst(leaf, kind));
                }
                out
            }
        };
        let consts = match consts.len() as u64 {
            1 => vec![consts[0]; count as usize],
            n if n == count => consts,
            n => return Err(format!("{n} elements for {count} places")),
        };
        let mut it = consts.into_iter();
        Ok(self.build_aggregate(ty, &mut it))
    }

    /// The length and element type of an array or vector type.
    fn leaf_step(&self, ty: LTypeId) -> Option<(u64, LTypeId)> {
        match *self.out.types.get(ty) {
            LType::Array { len, elem } => Some((len, elem)),
            LType::Vector { len, elem, .. } => Some((len as u64, elem)),
            _ => None,
        }
    }

    fn build_aggregate(&mut self, ty: LTypeId, it: &mut impl Iterator<Item = ConstId>) -> ConstId {
        match *self.out.types.get(ty) {
            LType::Array { len, elem } => {
                let members: Vec<ConstId> = (0..len).map(|_| self.build_aggregate(elem, it)).collect();
                if *self.out.types.get(elem) == LType::Int(8) {
                    let bytes: Vec<u8> = members
                        .iter()
                        .map(|&c| match &self.out.consts.get(c).kind {
                            ConstKind::Int(a) => a.low_u64() as u8,
                            _ => unreachable!(),
                        })
                        .collect();
                    return self.konst(ty, ConstKind::Bytes(bytes));
                }
                self.konst(ty, ConstKind::Array(members))
            }
            LType::Vector { len, elem, .. } => {
                let members: Vec<ConstId> = (0..len).map(|_| self.build_aggregate(elem, it)).collect();
                self.konst(ty, ConstKind::Vector(members))
            }
            _ => it.next().expect("as many elements as places"),
        }
    }

    /// An integer constant of up to 64 bits: its width and value.
    fn int_of(&self, c: ConstId) -> Option<(u32, u64)> {
        match &self.out.consts.get(c).kind {
            ConstKind::Int(a) if a.bits() <= 64 => Some((a.bits(), a.low_u64())),
            _ => None,
        }
    }

    /// A `float` or `double` constant as a double.
    fn float_of(&self, c: ConstId) -> Option<(FloatKind, f64)> {
        let k = self.out.consts.get(c);
        let kind = FloatKind::of(self.out.types.get(k.ty))?;
        match (&k.kind, kind) {
            (ConstKind::Float(bits), FloatKind::Float | FloatKind::Double) => Some((kind, float::to_f64(kind, *bits)?)),
            _ => None,
        }
    }

    /// Member `k` of a constant aggregate.
    fn member_const(&mut self, c: ConstId, k: u64) -> Option<ConstId> {
        let Constant { ty, kind } = self.out.consts.get(c).clone();
        let mty = self.out.types.member(ty, k)?;
        Some(match kind {
            ConstKind::Struct(v) | ConstKind::Array(v) | ConstKind::Vector(v) => *v.get(k as usize)?,
            ConstKind::Bytes(b) => self.int_const(mty, *b.get(k as usize)? as u64),
            ConstKind::Splat(e) => e,
            ConstKind::Zero => self.zero(mty),
            ConstKind::Undef => self.konst(mty, ConstKind::Undef),
            ConstKind::Poison => self.konst(mty, ConstKind::Poison),
            _ => return None,
        })
    }

    /// A constant struct or array with the member at `path` replaced.
    fn insert_const(&mut self, agg: ConstId, path: &[u32], elem: ConstId) -> Option<ConstId> {
        let Some((&first, rest)) = path.split_first() else { return Some(elem) };
        let ty = self.out.consts.get(agg).ty;
        let (n, is_struct) = match self.out.types.get(ty) {
            LType::Struct { fields, .. } => (fields.len() as u64, true),
            LType::Array { len, .. } if *len <= 4096 => (*len, false),
            _ => return None,
        };
        let mut members = Vec::new();
        for k in 0..n {
            members.push(self.member_const(agg, k)?);
        }
        let slot = members.get_mut(first as usize)?;
        let old = *slot;
        members[first as usize] = self.insert_const(old, rest, elem)?;
        let kind = if is_struct { ConstKind::Struct(members) } else { ConstKind::Array(members) };
        Some(self.konst(ty, kind))
    }

    /// The byte offset a constant `getelementptr` adds, by the data layout.
    fn gep_offset(&self, source_ty: LTypeId, indices: &[ConstId]) -> Option<i64> {
        let types = &self.out.types;
        let mut ty = source_ty;
        let mut offset = 0i64;
        for (k, &i) in indices.iter().enumerate() {
            let (bits, v) = self.int_of(i)?;
            let v = sext(v, bits);
            if k == 0 {
                offset += v * self.dl.alloc_size(types, ty)? as i64;
                continue;
            }
            match types.get(ty) {
                LType::Struct { fields, packed } => {
                    let layout = self.dl.struct_layout(types, fields, *packed)?;
                    offset += *layout.offsets.get(v as usize)? as i64;
                    ty = fields[v as usize];
                }
                LType::Array { elem, .. } | LType::Vector { elem, .. } => {
                    offset += v * self.dl.alloc_size(types, *elem)? as i64;
                    ty = *elem;
                }
                _ => return None,
            }
        }
        Some(offset)
    }

    /// Folds an instruction whose operands are all constants into a
    /// constant of type `ty`, where LLVM's IR builder would.
    fn fold(&mut self, op: &Op, ty: LTypeId) -> Option<ConstId> {
        let c = |v: &ll::Value| match v {
            ll::Value::Const(c) => Some(*c),
            _ => None,
        };
        match op {
            Op::Binary { op, flags, lhs, rhs, .. } => {
                let (a, b) = (c(lhs)?, c(rhs)?);
                if let (Some((bits, x)), Some((_, y))) = (self.int_of(a), self.int_of(b)) {
                    let r = fold_int(*op, *flags, bits, x, y)?;
                    return Some(self.int_const(ty, r));
                }
                let ((kind, x), (_, y)) = (self.float_of(a)?, self.float_of(b)?);
                let r = match kind {
                    FloatKind::Float => fold_float(*op, x as f32, y as f32)? as f64,
                    _ => fold_float(*op, x, y)?,
                };
                Some(self.konst(ty, ConstKind::Float(float::from_f64(kind, r)?)))
            }
            Op::ICmp { pred, lhs, rhs, .. } => {
                let ((bits, x), (_, y)) = (self.int_of(c(lhs)?)?, self.int_of(c(rhs)?)?);
                let (sx, sy) = (sext(x, bits), sext(y, bits));
                let r = match pred {
                    ICmpPred::Eq => x == y,
                    ICmpPred::Ne => x != y,
                    ICmpPred::Ugt => x > y,
                    ICmpPred::Uge => x >= y,
                    ICmpPred::Ult => x < y,
                    ICmpPred::Ule => x <= y,
                    ICmpPred::Sgt => sx > sy,
                    ICmpPred::Sge => sx >= sy,
                    ICmpPred::Slt => sx < sy,
                    ICmpPred::Sle => sx <= sy,
                };
                Some(self.int_const(ty, r as u64))
            }
            Op::FCmp { pred, lhs, rhs, .. } => {
                let ((_, x), (_, y)) = (self.float_of(c(lhs)?)?, self.float_of(c(rhs)?)?);
                let r = fold_fcmp(*pred, x, y);
                Some(self.int_const(ty, r as u64))
            }
            Op::Select { cond, then_value, else_value, .. } => {
                let (_, k) = self.int_of(c(cond)?)?;
                let (t, e) = (c(then_value)?, c(else_value)?);
                Some(if k != 0 { t } else { e })
            }
            Op::Cast { op, flags, value, .. } => {
                let v = c(value)?;
                let from = self.out.consts.get(v).clone();
                match op {
                    CastOp::Trunc | CastOp::ZExt | CastOp::SExt => {
                        let (bits, x) = self.int_of(v)?;
                        let LType::Int(to) = *self.out.types.get(ty) else { return None };
                        if to > 64 || *flags != IntFlags::NONE {
                            return None;
                        }
                        let r = if *op == CastOp::SExt { sext(x, bits) as u64 } else { x };
                        Some(self.int_const(ty, r))
                    }
                    CastOp::IntToPtr => Some(self.konst(ty, ConstKind::Expr(ConstExpr::Cast { op: *op, value: v }))),
                    CastOp::PtrToInt => {
                        let LType::Int(to) = *self.out.types.get(ty) else { return None };
                        match &from.kind {
                            ConstKind::Null if to <= 64 => Some(self.int_const(ty, 0)),
                            ConstKind::Expr(ConstExpr::Gep { source_ty, ptr, indices, .. })
                                if self.out.consts.get(*ptr).kind == ConstKind::Null && to <= 64 =>
                            {
                                let off = self.gep_offset(*source_ty, indices)?;
                                Some(self.int_const(ty, off as u64))
                            }
                            _ => Some(self.konst(ty, ConstKind::Expr(ConstExpr::Cast { op: *op, value: v }))),
                        }
                    }
                    CastOp::BitCast if from.ty == ty => Some(v),
                    _ => None,
                }
            }
            Op::Gep { flags, source_ty, ptr, indices, .. } => {
                let base = c(ptr)?;
                let idx: Vec<ConstId> = indices.iter().map(c).collect::<Option<_>>()?;
                if !self.out.types.is_ptr(self.out.consts.get(base).ty) {
                    return None;
                }
                if idx.iter().all(|&i| self.int_of(i).is_some_and(|(_, v)| v == 0)) {
                    return Some(base);
                }
                let expr =
                    ConstExpr::Gep { flags: *flags, inrange: None, source_ty: *source_ty, ptr: base, indices: idx };
                Some(self.konst(ty, ConstKind::Expr(expr)))
            }
            Op::ExtractValue { aggregate, indices } => {
                let mut v = c(aggregate)?;
                for &k in indices {
                    v = self.member_const(v, k as u64)?;
                }
                Some(v)
            }
            Op::InsertValue { aggregate, element, indices } => self.insert_const(c(aggregate)?, indices, c(element)?),
            _ => None,
        }
    }

    // -----------------------------------------------------------------------
    // Module level

    fn declare_global(&mut self, op: OpId) -> Result<()> {
        let name = self.str_attr(op, "sym_name").ok_or("llvm.mlir.global: no sym_name")?;
        let err = |e: String| format!("@{name}: llvm.mlir.global: {e}");
        let ty = self.type_attr(op, "global_type").map_err(err)?.ok_or_else(|| err("no global_type".into()))?;
        let mut props = self.props(op).map_err(err)?;
        props.thread_local = match self.int_attr(op, "tls_mode").unwrap_or(0) {
            0 => None,
            1 => Some(ll::ThreadLocal::GeneralDynamic),
            2 => Some(ll::ThreadLocal::LocalDynamic),
            3 => Some(ll::ThreadLocal::InitialExec),
            4 => Some(ll::ThreadLocal::LocalExec),
            n => return Err(err(format!("unknown tls_mode {n}"))),
        };
        let space = self.int_attr(op, "addr_space").unwrap_or(0);
        props.addrspace = (space != 0).then_some(space as u32);
        let mut g = ll::Global::variable(&name, ty, None, self.unit_attr(op, "constant"));
        g.props = props;
        if let ll::GlobalKind::Variable(v) = &mut g.kind {
            v.align = self.int_attr(op, "alignment").map(|a| a as u64);
            v.section = self.str_attr(op, "section").map(String::into_bytes);
            v.externally_initialized = self.unit_attr(op, "externally_initialized");
        }
        let id = self.out.add_global(g);
        self.symbols.insert(name, id);
        Ok(())
    }

    fn global_init(&mut self, op: OpId) -> Result<()> {
        let id = self.symbols[&self.str_attr(op, "sym_name").unwrap()];
        let ll::GlobalKind::Variable(v) = &self.out.global(id).kind else { unreachable!() };
        let ty = v.ty;
        let blocks: Vec<BlockId> = self.m.ops[op.index()]
            .regions
            .first()
            .map(|r| self.m.regions[r.index()].blocks.clone())
            .unwrap_or_default();
        let init = if let Some(value) = self.m.op_attr(op, "value") {
            Some(self.attr_const(value, ty).map_err(|e| format!("llvm.mlir.global: {e}"))?)
        } else if let Some(&b) = blocks.first() {
            if blocks.len() > 1 {
                return Err("llvm.mlir.global: an initialiser region of more than one block".into());
            }
            let mut body = Body::new(None);
            for &o in &self.m.blocks[b.index()].ops {
                self.inst(&mut body, o).map_err(|e| format!("{}: {e}", self.m.op_name(o)))?;
            }
            match body.ret {
                Some(ll::Value::Const(c)) => Some(c),
                _ => return Err("llvm.mlir.global: the initialiser region does not return a constant".into()),
            }
        } else {
            None
        };
        let g = &mut self.out.globals[id.index()];
        if init.is_none() {
            g.props.explicit_linkage = true;
        }
        if let ll::GlobalKind::Variable(v) = &mut g.kind {
            v.init = init;
        }
        Ok(())
    }

    fn declare_function(&mut self, op: OpId) -> Result<()> {
        let name = self.str_attr(op, "sym_name").ok_or("llvm.func: no sym_name")?;
        let err = |e: String| format!("@{name}: llvm.func: {e}");
        let fty = self.type_attr(op, "function_type").map_err(err)?.ok_or_else(|| err("no function_type".into()))?;
        if !matches!(self.out.types.get(fty), LType::Function { .. }) {
            return Err(err("function_type is not a function type".into()));
        }
        let mut f = ll::Function::declaration(&self.out.types, fty);
        f.cconv = self.cconv(op);
        if let Some(Attr::Array(list)) = self.attr(op, "arg_attrs") {
            for (k, &d) in list.iter().enumerate() {
                let attrs = self.value_attrs(d);
                if let Some(p) = f.params.get_mut(k) {
                    p.attrs = attrs;
                }
            }
        }
        if let Some(Attr::Array(list)) = self.attr(op, "res_attrs")
            && let Some(&d) = list.first()
        {
            f.ret_attrs = self.value_attrs(d);
        }
        for (key, word) in FUNC_FLAGS {
            if self.unit_attr(op, key) {
                f.fn_attrs.push(ll::Attr::Word(word.to_string()));
            }
        }
        for (key, llvm) in [("target_cpu", "target-cpu"), ("tune_cpu", "tune-cpu")] {
            if let Some(v) = self.str_attr(op, key) {
                f.fn_attrs.push(ll::Attr::Str { key: llvm.as_bytes().to_vec(), value: Some(v.into_bytes()) });
            }
        }
        if let Some(Attr::Array(list)) = self.attr(op, "passthrough") {
            for &a in list {
                f.fn_attrs.extend(self.passthrough(a));
            }
        }
        f.section = self.str_attr(op, "section").map(String::into_bytes);
        f.align = self.int_attr(op, "alignment").map(|a| a as u64);
        f.gc = self.str_attr(op, "garbageCollector").map(String::into_bytes);
        let props = self.props(op).map_err(err)?;
        let g = ll::Global {
            name: Some(name.clone()),
            props,
            attachments: vec![],
            kind: ll::GlobalKind::Function(Box::new(f)),
        };
        let id = self.out.add_global(g);
        self.symbols.insert(name, id);
        Ok(())
    }

    /// One `passthrough` entry: `"name"` or `["name", "value"]`.
    fn passthrough(&self, a: AttrId) -> Option<ll::Attr> {
        let m = self.m;
        let text = |a: AttrId| match m.attr(a) {
            Attr::String { bytes, .. } => Some(String::from_utf8_lossy(bytes).into_owned()),
            _ => None,
        };
        match m.attr(a) {
            Attr::String { .. } => {
                let name = text(a)?;
                Some(if KEYWORD_ATTRS.contains(&name.as_str()) {
                    ll::Attr::Word(name)
                } else {
                    ll::Attr::Str { key: name.into_bytes(), value: None }
                })
            }
            Attr::Array(pair) if pair.len() == 2 => {
                let (key, value) = (text(pair[0])?, text(pair[1])?);
                match (key.as_str(), value.parse::<u64>()) {
                    ("alignstack", Ok(v)) => Some(ll::Attr::Int { name: key, value: v, style: IntStyle::Eq }),
                    _ => Some(ll::Attr::Str { key: key.into_bytes(), value: Some(value.into_bytes()) }),
                }
            }
            _ => None,
        }
    }

    fn define_function(&mut self, op: OpId) -> Result<()> {
        let m = self.m;
        let Some(region) = m.ops[op.index()].regions.first() else { return Ok(()) };
        let blocks = m.regions[region.index()].blocks.clone();
        if blocks.is_empty() {
            return Ok(());
        }
        let id = self.symbols[&self.str_attr(op, "sym_name").unwrap()];
        let ll::GlobalKind::Function(f) = &mut self.out.globals[id.index()].kind else { unreachable!() };
        let mut f = std::mem::take(&mut **f);
        f.has_body = true;
        let label = self.out.types.label();
        let mut body = Body::new(None);
        for (k, &b) in blocks.iter().enumerate() {
            let lb = if k == 0 {
                f.add_entry_block(None, label)
            } else {
                let lb = f.add_block(None);
                f.layout.push(lb);
                lb
            };
            body.blocks.insert(b, lb);
        }
        // The entry block's arguments are the parameters; every other
        // block's are phis.
        let entry_args = &m.blocks[blocks[0].index()].args;
        if entry_args.len() != f.params.len() {
            return Err(format!(
                "llvm.func: {} entry block arguments for {} parameters",
                entry_args.len(),
                f.params.len()
            ));
        }
        for (k, &a) in entry_args.iter().enumerate() {
            body.values.insert(a, ll::Value::Local(f.params[k].local));
        }
        for &b in &blocks[1..] {
            let lb = body.blocks[&b];
            let mut phis = Vec::new();
            for (k, &a) in m.blocks[b.index()].args.iter().enumerate() {
                let ty = self.ty(m.values[a.index()].ty)?;
                let i = f.add_inst(Op::Phi { fmf: FastMath::default(), incoming: vec![] }, Some((None, ty)));
                f.blocks[lb.index()].insts.insert(k, i);
                body.values.insert(a, f.result(i));
                phis.push(i);
            }
            body.phis.insert(b, phis);
        }
        body.f = Some(f);
        // Dominators before the blocks they dominate, so that every value is
        // translated before its uses; then any unreachable blocks.
        let mut order = reverse_postorder(m, &blocks);
        let reached: HashSet<BlockId> = order.iter().copied().collect();
        order.extend(blocks.iter().copied().filter(|b| !reached.contains(b)));
        for b in order {
            body.cur = body.blocks[&b];
            for &o in &m.blocks[b.index()].ops {
                self.inst(&mut body, o).map_err(|e| format!("{}: {e}", m.op_name(o)))?;
            }
        }
        let f = body.f.take().unwrap();
        let ll::GlobalKind::Function(slot) = &mut self.out.globals[id.index()].kind else { unreachable!() };
        **slot = f;
        Ok(())
    }

    // -----------------------------------------------------------------------
    // Operations

    /// Adds an instruction at the end of the current block.
    fn push(&mut self, body: &mut Body, op: Op, ty: Option<LTypeId>) -> Result<Option<ll::Value>> {
        let cur = body.cur;
        let f = body.func()?;
        let i = f.push_inst(cur, op, ty.map(|t| (None, t)));
        Ok(ty.map(|_| f.result(i)))
    }

    /// Defines `result` as the value of `op`: a constant if it folds, else
    /// an instruction.
    fn value(&mut self, body: &mut Body, op: Op, ty: LTypeId, result: ValueId) -> Result<()> {
        let mut all_const = true;
        op.for_each_value(|v| all_const &= matches!(v, ll::Value::Const(_)));
        let folded = if all_const { self.fold(&op, ty) } else { None };
        let v = match folded {
            Some(c) => ll::Value::Const(c),
            None => self.push(body, op, Some(ty))?.unwrap(),
        };
        body.values.insert(result, v);
        Ok(())
    }

    /// The block a terminator branches to for its successor `dest`, passing
    /// `args`; adds the phi entries. `seen` holds the successors already
    /// handled for this terminator: a second edge to one of them goes
    /// through a new block.
    fn edge(
        &mut self,
        body: &mut Body,
        dest: BlockId,
        args: &[ll::Value],
        seen: &mut Vec<BlockId>,
    ) -> Result<ll::BlockId> {
        let target = body.blocks[&dest];
        let phis = body.phis.get(&dest).cloned().unwrap_or_default();
        if phis.len() != args.len() {
            return Err(format!("{} values for {} block arguments", args.len(), phis.len()));
        }
        if phis.is_empty() {
            return Ok(target);
        }
        let split = seen.contains(&dest);
        seen.push(dest);
        let cur = body.cur;
        let f = body.func()?;
        let from = if split {
            let nb = f.add_block(None);
            f.layout.push(nb);
            f.push_inst(nb, Op::Br(target), None);
            nb
        } else {
            cur
        };
        for (&phi, &v) in phis.iter().zip(args) {
            if let Op::Phi { incoming, .. } = &mut f.insts[phi.index()].op {
                incoming.push((v, from));
            }
        }
        Ok(if split { from } else { target })
    }

    /// Splits a terminator's operands after the first `skip` into one list
    /// per successor, by the successors' argument counts.
    fn successor_args<'v>(
        &self,
        op: OpId,
        args: &'v [ll::Value],
        skip: usize,
    ) -> Result<Vec<(BlockId, &'v [ll::Value])>> {
        let m = self.m;
        let mut at = skip;
        let mut out = Vec::new();
        for &s in &m.ops[op.index()].successors {
            let n = m.blocks[s.index()].args.len();
            let list = args.get(at..at + n).ok_or("fewer operands than block arguments")?;
            out.push((s, list));
            at += n;
        }
        if at != args.len() {
            return Err("more operands than block arguments".into());
        }
        Ok(out)
    }

    fn inst(&mut self, body: &mut Body, op: OpId) -> Result<()> {
        let m = self.m;
        let name = m.op_name(op);
        let o = &m.ops[op.index()];
        let args: Vec<ll::Value> = o.operands.iter().map(|&v| body.val(v)).collect::<Result<_>>()?;
        let (in_tys, out_tys) = m.op_signature(op);
        let result = o.results.first().copied();
        let res_ty = match out_tys {
            [] => None,
            [t] => Some(self.ty(*t)?),
            _ => return Err("more than one result".into()),
        };
        let short = name.strip_prefix("llvm.").ok_or("not supported yet")?;
        let need = |n: usize| -> Result<()> {
            if args.len() == n { Ok(()) } else { Err(format!("{} operands, not {n}", args.len())) }
        };
        let res = || -> Result<(LTypeId, ValueId)> {
            match (res_ty, result) {
                (Some(t), Some(r)) => Ok((t, r)),
                _ => Err("no result".into()),
            }
        };
        if let Some(bop) = BinOp::ALL.iter().find(|(k, _)| *k == short).map(|(_, b)| *b) {
            need(2)?;
            let (ty, r) = res()?;
            let flags = IntFlags(self.int_flags(op).0 & bop.allowed_flags().0);
            let fmf = if bop.is_float() { self.fastmath(op) } else { FastMath::default() };
            return self.value(body, Op::Binary { op: bop, flags, fmf, lhs: args[0], rhs: args[1] }, ty, r);
        }
        if let Some(cop) = CastOp::ALL.iter().find(|(k, _)| *k == short).map(|(_, c)| *c) {
            need(1)?;
            let (ty, r) = res()?;
            let flags = IntFlags(self.int_flags(op).0 & cop.allowed_flags().0);
            let fmf = if cop.allows_fmf() { self.fastmath(op) } else { FastMath::default() };
            return self.value(body, Op::Cast { op: cop, flags, fmf, value: args[0] }, ty, r);
        }
        if let Some(intr) = short.strip_prefix("intr.") {
            return self.intrinsic(body, op, intr, &args, res_ty);
        }
        match short {
            "mlir.constant" => {
                let (ty, r) = res()?;
                let value = m.op_attr(op, "value").ok_or("no value")?;
                let c = self.attr_const(value, ty)?;
                body.values.insert(r, ll::Value::Const(c));
            }
            "mlir.zero" | "mlir.undef" | "mlir.poison" => {
                let (ty, r) = res()?;
                let c = match short {
                    "mlir.zero" => self.zero(ty),
                    "mlir.undef" => self.konst(ty, ConstKind::Undef),
                    _ => self.konst(ty, ConstKind::Poison),
                };
                body.values.insert(r, ll::Value::Const(c));
            }
            "mlir.addressof" => {
                let (_, r) = res()?;
                let Some(Attr::SymbolRef(path)) = self.attr(op, "global_name") else {
                    return Err("no global_name".into());
                };
                let g = *self.symbols.get(&path[0]).ok_or_else(|| format!("no symbol @{}", path[0]))?;
                body.values.insert(r, self.out.global_value(g));
            }
            "fneg" => {
                need(1)?;
                let (ty, r) = res()?;
                let fmf = self.fastmath(op);
                self.value(body, Op::FNeg { fmf, value: args[0] }, ty, r)?;
            }
            "icmp" | "fcmp" => {
                need(2)?;
                let (ty, r) = res()?;
                let n = self.int_attr(op, "predicate").ok_or("no predicate")? as usize;
                let bad = || format!("unknown predicate {n}");
                let cmp = if short == "icmp" {
                    let pred = *ICMP_PREDS.get(n).ok_or_else(bad)?;
                    Op::ICmp { pred, samesign: false, lhs: args[0], rhs: args[1] }
                } else {
                    let pred = FCmpPred::ALL.get(n).ok_or_else(bad)?.1;
                    Op::FCmp { pred, fmf: self.fastmath(op), lhs: args[0], rhs: args[1] }
                };
                self.value(body, cmp, ty, r)?;
            }
            "select" => {
                need(3)?;
                let (ty, r) = res()?;
                let sel =
                    Op::Select { fmf: self.fastmath(op), cond: args[0], then_value: args[1], else_value: args[2] };
                self.value(body, sel, ty, r)?;
            }
            "freeze" => {
                need(1)?;
                let (ty, r) = res()?;
                self.value(body, Op::Freeze(args[0]), ty, r)?;
            }
            "getelementptr" => {
                let (ty, r) = res()?;
                let source_ty = self.type_attr(op, "elem_type")?.ok_or("no elem_type")?;
                let raw = self.ints_attr(op, "rawConstantIndices").unwrap_or_default();
                let i32t = self.out.types.int(32);
                let mut dynamic = args.iter().skip(1);
                let mut indices = Vec::new();
                for k in raw {
                    indices.push(if k == DYNAMIC_INDEX {
                        *dynamic.next().ok_or("fewer dynamic indices than operands")?
                    } else {
                        ll::Value::Const(self.int_const(i32t, k as u32 as u64))
                    });
                }
                if dynamic.next().is_some() {
                    return Err("more dynamic indices than markers".into());
                }
                let mut flags = self.int_attr(op, "noWrapFlags").unwrap_or(0) as u8;
                if self.unit_attr(op, "inbounds") {
                    flags |= GepFlags::INBOUNDS;
                }
                let gep = Op::Gep { flags: GepFlags(flags), source_ty, ptr: args[0], indices: indices.into() };
                self.value(body, gep, ty, r)?;
            }
            "load" => {
                need(1)?;
                let (ty, r) = res()?;
                let (atomic, align) = self.atomic(op, ty)?;
                let volatile = self.unit_attr(op, "volatile_") || self.unit_attr(op, "volatile");
                let v = self.push(body, Op::Load { volatile, atomic, ptr: args[0], align }, Some(ty))?.unwrap();
                body.values.insert(r, v);
            }
            "store" => {
                need(2)?;
                let vt = self.vty(body, args[0]);
                let (atomic, align) = self.atomic(op, vt)?;
                let volatile = self.unit_attr(op, "volatile_") || self.unit_attr(op, "volatile");
                self.push(body, Op::Store { volatile, atomic, value: args[0], ptr: args[1], align }, None)?;
            }
            "alloca" => {
                need(1)?;
                let (ty, r) = res()?;
                let elem = self.type_attr(op, "elem_type")?.ok_or("no elem_type")?;
                let addrspace = match *self.out.types.get(ty) {
                    LType::Ptr(0) => None,
                    LType::Ptr(n) => Some(n),
                    _ => return Err("a result that is not a pointer".into()),
                };
                let a = Op::Alloca {
                    inalloca: self.unit_attr(op, "inalloca"),
                    ty: elem,
                    count: Some(args[0]),
                    align: self.alignment(op)?,
                    addrspace,
                };
                let v = self.push(body, a, Some(ty))?.unwrap();
                body.values.insert(r, v);
            }
            "call" => self.call(body, op, &args, in_tys, res_ty)?,
            "call_intrinsic" => {
                let intrin = self.str_attr(op, "intrin").ok_or("no intrin")?;
                let tys: Vec<LTypeId> = args.iter().map(|&a| self.vty(body, a)).collect();
                self.call_named(body, op, &intrin, &args, &tys, res_ty)?;
            }
            "extractvalue" => {
                need(1)?;
                let (ty, r) = res()?;
                let indices = self.position(op)?;
                self.value(body, Op::ExtractValue { aggregate: args[0], indices: indices.into() }, ty, r)?;
            }
            "insertvalue" => {
                need(2)?;
                let (ty, r) = res()?;
                let indices = self.position(op)?;
                self.value(
                    body,
                    Op::InsertValue { aggregate: args[0], element: args[1], indices: indices.into() },
                    ty,
                    r,
                )?;
            }
            "extractelement" => {
                need(2)?;
                let (ty, r) = res()?;
                self.value(body, Op::ExtractElement { vector: args[0], index: args[1] }, ty, r)?;
            }
            "insertelement" => {
                need(3)?;
                let (ty, r) = res()?;
                // The operand with the result's type is the vector; the
                // value comes before the position, as in the custom form.
                let tys: Vec<LTypeId> = args.iter().map(|&a| self.vty(body, a)).collect();
                let vk = tys.iter().position(|&t| t == ty).ok_or("no operand of the vector's type")?;
                let rest: Vec<ll::Value> = (0..3).filter(|&k| k != vk).map(|k| args[k]).collect();
                let ins = Op::InsertElement { vector: args[vk], element: rest[0], index: rest[1] };
                self.value(body, ins, ty, r)?;
            }
            "shufflevector" => {
                need(2)?;
                let (ty, r) = res()?;
                let mask = self.ints_attr(op, "mask").ok_or("no mask")?;
                let i32t = self.out.types.int(32);
                let lanes: Vec<ConstId> = mask
                    .iter()
                    .map(|&k| if k < 0 { self.konst(i32t, ConstKind::Poison) } else { self.int_const(i32t, k as u64) })
                    .collect();
                let mty = self.out.types.vector(lanes.len() as u32, i32t);
                let mask = self.konst(mty, ConstKind::Vector(lanes));
                let v = self.push(body, Op::ShuffleVector { a: args[0], b: args[1], mask }, Some(ty))?.unwrap();
                body.values.insert(r, v);
            }
            "return" => {
                if body.f.is_none() {
                    body.ret = args.first().copied();
                } else {
                    self.push(body, Op::Ret(args.first().copied()), None)?;
                }
            }
            "unreachable" => {
                self.push(body, Op::Unreachable, None)?;
            }
            "br" => {
                let succ = self.successor_args(op, &args, 0)?;
                let [(dest, list)] = succ.as_slice() else { return Err("not one successor".into()) };
                let to = self.edge(body, *dest, list, &mut vec![])?;
                self.push(body, Op::Br(to), None)?;
            }
            "cond_br" => {
                let succ = self.successor_args(op, &args, 1)?;
                let [(t, tl), (e, el)] = succ.as_slice() else { return Err("not two successors".into()) };
                let mut seen = vec![];
                let then_dest = self.edge(body, *t, tl, &mut seen)?;
                let else_dest = self.edge(body, *e, el, &mut seen)?;
                self.push(body, Op::CondBr { cond: args[0], then_dest, else_dest }, None)?;
            }
            "switch" => {
                let succ = self.successor_args(op, &args, 1)?;
                let values = self.ints_attr(op, "case_values").unwrap_or_default();
                if values.len() + 1 != succ.len() {
                    return Err(format!("{} case values for {} cases", values.len(), succ.len() - 1));
                }
                let vt = self.vty(body, args[0]);
                let mut seen = vec![];
                let default = self.edge(body, succ[0].0, succ[0].1, &mut seen)?;
                let mut cases = Vec::new();
                for (&k, &(dest, list)) in values.iter().zip(&succ[1..]) {
                    let to = self.edge(body, dest, list, &mut seen)?;
                    cases.push((self.int_const(vt, k as u64), to));
                }
                self.push(body, Op::Switch { value: args[0], default, cases: cases.into() }, None)?;
            }
            _ => return Err("not supported yet".into()),
        }
        Ok(())
    }

    /// `alignment`, where 0 means none given.
    fn alignment(&self, op: OpId) -> Result<Option<ll::Align>> {
        match self.int_attr(op, "alignment").filter(|&a| a > 0) {
            None => Ok(None),
            Some(a) => ll::Align::new(a as u64).map(Some).ok_or_else(|| format!("alignment {a} is not a power of two")),
        }
    }

    /// `position` of `extractvalue` and `insertvalue`.
    fn position(&self, op: OpId) -> Result<Vec<u32>> {
        let p = self.ints_attr(op, "position").ok_or("no position")?;
        Ok(p.into_iter().map(|k| k as u32).collect())
    }

    /// The atomic ordering and alignment of a load or store of type `ty`.
    fn atomic(&self, op: OpId, ty: LTypeId) -> Result<(Option<ll::Atomic>, Option<ll::Align>)> {
        let mut align = self.alignment(op)?;
        let ordering = match self.int_attr(op, "ordering").unwrap_or(0) {
            0 => return Ok((None, align)),
            1 => ll::Ordering::Unordered,
            2 => ll::Ordering::Monotonic,
            4 => ll::Ordering::Acquire,
            5 => ll::Ordering::Release,
            6 => ll::Ordering::AcqRel,
            7 => ll::Ordering::SeqCst,
            n => return Err(format!("unknown ordering {n}")),
        };
        if align.is_none() {
            align = self.dl.abi_align(&self.out.types, ty).and_then(ll::Align::new);
        }
        let scope = self.str_attr(op, "syncscope").map(String::into_bytes);
        Ok((Some(ll::Atomic { elementwise: false, scope: scope.map(Into::into), ordering }), align))
    }

    fn call(
        &mut self,
        body: &mut Body,
        op: OpId,
        args: &[ll::Value],
        in_tys: &[TypeId],
        res_ty: Option<LTypeId>,
    ) -> Result<()> {
        let segments = self.ints_attr(op, "operandSegmentSizes");
        let n = match segments.as_deref() {
            Some([n, rest @ ..]) => {
                if rest.iter().any(|&k| k != 0) {
                    return Err("operand bundles: not supported yet".into());
                }
                *n as usize
            }
            _ => args.len(),
        };
        let (callee, first) = match self.attr(op, "callee") {
            Some(Attr::SymbolRef(path)) => {
                let g = *self.symbols.get(&path[0]).ok_or_else(|| format!("no function @{}", path[0]))?;
                (self.out.global_value(g), 0)
            }
            _ => (*args.first().ok_or("no callee")?, 1),
        };
        let call_args = &args[first..n];
        let fn_ty = match self.type_attr(op, "var_callee_type")? {
            Some(t) => t,
            None => {
                let mut params = Vec::new();
                for &t in &in_tys[first..n] {
                    params.push(self.ty(t)?);
                }
                let ret = res_ty.unwrap_or_else(|| self.out.types.void());
                self.out.types.function(ret, &params, false)
            }
        };
        let mut call = ll::Call::new(fn_ty, callee, call_args);
        call.cconv = self.cconv(op);
        call.fmf = self.fastmath(op);
        call.tail = match self.dialect_attr(op, "TailCallKind") {
            Some("tail") => ll::TailKind::Tail,
            Some("musttail") => ll::TailKind::MustTail,
            Some("notail") => ll::TailKind::NoTail,
            _ => ll::TailKind::None,
        };
        let v = self.push(body, Op::Call(Box::new(call)), res_ty)?;
        if let (Some(v), Some(&r)) = (v, self.m.ops[op.index()].results.first()) {
            body.values.insert(r, v);
        }
        Ok(())
    }

    /// `llvm.intr.<name>`: a call of `@llvm.<name>` with the overload
    /// suffixes LLVM's Language Reference gives it, and the `i1` or `i32`
    /// arguments that the dialect keeps as attributes.
    fn intrinsic(
        &mut self,
        body: &mut Body,
        op: OpId,
        name: &str,
        args: &[ll::Value],
        res_ty: Option<LTypeId>,
    ) -> Result<()> {
        let mut args = args.to_vec();
        let mut tys: Vec<LTypeId> = args.iter().map(|&a| self.vty(body, a)).collect();
        let arg = |k: usize| tys.get(k).copied().ok_or_else(|| format!("too few operands for llvm.{name}"));
        let overloads: Vec<LTypeId> = match name {
            "memcpy" | "memmove" | "memcpy.inline" => vec![arg(0)?, arg(1)?, arg(2)?],
            "memset" | "memset.inline" => vec![arg(0)?, arg(2)?],
            "powi" | "ldexp" => vec![res_ty.ok_or("no result")?, arg(1)?],
            "lrint" | "llrint" | "lround" | "llround" | "scmp" | "ucmp" | "fptosi.sat" | "fptoui.sat" => {
                vec![res_ty.ok_or("no result")?, arg(0)?]
            }
            "ptrmask" => vec![arg(0)?, arg(1)?],
            "stackrestore" | "lifetime.start" | "lifetime.end" => vec![*tys.last().ok_or("no operands")?],
            "vector.reduce.fadd" | "vector.reduce.fmul" => vec![arg(1)?],
            _ if name.starts_with("vector.reduce.") => vec![arg(0)?],
            _ if name.ends_with(".with.overflow") => vec![arg(0)?],
            "abs" | "ctlz" | "cttz" | "is.fpclass" => vec![arg(0)?],
            _ if NOT_OVERLOADED.contains(&name) => vec![],
            _ if BY_RESULT.contains(&name) => vec![res_ty.ok_or("no result")?],
            _ => return Err("not supported yet".into()),
        };
        // Flags that LLVM passes as immediate arguments.
        let i1 = self.out.types.int(1);
        let extra: Option<(LTypeId, &str)> = match name {
            "memcpy" | "memmove" | "memset" | "memcpy.inline" | "memset.inline" if args.len() == 3 => {
                Some((i1, "isVolatile"))
            }
            "abs" if args.len() == 1 => Some((i1, "is_int_min_poison")),
            "ctlz" | "cttz" if args.len() == 1 => Some((i1, "is_zero_poison")),
            "is.fpclass" if args.len() == 1 => Some((self.out.types.int(32), "bit")),
            _ => None,
        };
        if let Some((ty, key)) = extra {
            let v = self.int_attr(op, key).unwrap_or(0);
            args.push(ll::Value::Const(self.int_const(ty, v as u64)));
            tys.push(ty);
        }
        let mut full = format!("llvm.{name}");
        for t in overloads {
            full.push('.');
            full.push_str(&self.mangle(t)?);
        }
        self.call_named(body, op, &full, &args, &tys, res_ty)
    }

    /// A call of the function `name`, declared with these types if the
    /// module does not have it yet.
    fn call_named(
        &mut self,
        body: &mut Body,
        op: OpId,
        name: &str,
        args: &[ll::Value],
        tys: &[LTypeId],
        res_ty: Option<LTypeId>,
    ) -> Result<()> {
        let ret = res_ty.unwrap_or_else(|| self.out.types.void());
        let g = match self.symbols.get(name) {
            Some(&g) => g,
            None => {
                let g = self.out.declare(name, ret, tys, false);
                self.symbols.insert(name.to_string(), g);
                g
            }
        };
        let fn_ty = self.out.types.function(ret, tys, false);
        let mut call = ll::Call::new(fn_ty, self.out.global_value(g), args);
        call.fmf = self.fastmath(op);
        let v = self.push(body, Op::Call(Box::new(call)), res_ty)?;
        if let (Some(v), Some(&r)) = (v, self.m.ops[op.index()].results.first()) {
            body.values.insert(r, v);
        }
        Ok(())
    }

    /// A type's name in an intrinsic's overload suffix: `i32`, `f64`,
    /// `p0`, `v4f32`, `nxv2i64`, `a4i8`, `sl_i32f32s`.
    fn mangle(&self, ty: LTypeId) -> Result<String> {
        let types = &self.out.types;
        Ok(match types.get(ty) {
            LType::Int(b) => format!("i{b}"),
            LType::Half => "f16".into(),
            LType::BFloat => "bf16".into(),
            LType::Float => "f32".into(),
            LType::Double => "f64".into(),
            LType::X86Fp80 => "f80".into(),
            LType::Fp128 => "f128".into(),
            LType::PpcFp128 => "ppcf128".into(),
            LType::Ptr(n) => format!("p{n}"),
            LType::Vector { scalable, len, elem } => {
                format!("{}v{len}{}", if *scalable { "nx" } else { "" }, self.mangle(*elem)?)
            }
            LType::Array { len, elem } => format!("a{len}{}", self.mangle(*elem)?),
            LType::Struct { fields, .. } => {
                let mut s = "sl_".to_string();
                for &f in fields {
                    s.push_str(&self.mangle(f)?);
                }
                s.push('s');
                s
            }
            _ => return Err("an intrinsic overloaded on a type it cannot name".into()),
        })
    }
}

/// The blocks reachable from the first, in reverse postorder.
fn reverse_postorder(m: &Module, blocks: &[BlockId]) -> Vec<BlockId> {
    let succs = |b: BlockId| -> Vec<BlockId> {
        m.blocks[b.index()].ops.last().map(|&t| m.ops[t.index()].successors.clone()).unwrap_or_default()
    };
    let mut seen = HashSet::new();
    let mut post = Vec::new();
    let mut stack = vec![(blocks[0], succs(blocks[0]), 0usize)];
    seen.insert(blocks[0]);
    while let Some((b, ss, k)) = stack.last_mut() {
        if let Some(&s) = ss.get(*k) {
            *k += 1;
            if seen.insert(s) {
                let next = succs(s);
                stack.push((s, next, 0));
            }
        } else {
            post.push(*b);
            stack.pop();
        }
    }
    post.reverse();
    post
}

/// The elements of a `dense` attribute or `leaves` input.
enum Elems {
    /// Scalars as written.
    Text(Vec<String>),
    /// Little-endian raw data.
    Raw(Vec<u8>),
}

/// A cursor over the text of a dialect type's body.
struct Cursor<'s> {
    s: &'s str,
    at: usize,
}

impl<'s> Cursor<'s> {
    fn ws(&mut self) {
        while self.s[self.at..].starts_with(char::is_whitespace) {
            self.at += 1;
        }
    }

    fn peek(&self) -> Option<char> {
        self.s[self.at..].chars().next()
    }

    fn eat(&mut self, t: &str) -> bool {
        self.ws();
        if self.s[self.at..].starts_with(t) {
            self.at += t.len();
            true
        } else {
            false
        }
    }

    fn expect(&mut self, t: &str) -> Result<()> {
        if self.eat(t) { Ok(()) } else { Err(format!("expected '{t}' at {:?}", &self.s[self.at..])) }
    }

    /// Letters, digits and `_`.
    fn word(&mut self) -> &'s str {
        self.ws();
        let rest = &self.s[self.at..];
        let n = rest.find(|c: char| !(c.is_ascii_alphanumeric() || c == '_')).unwrap_or(rest.len());
        self.at += n;
        &rest[..n]
    }

    fn number(&mut self) -> Result<u64> {
        self.ws();
        let rest = &self.s[self.at..];
        let n = rest.find(|c: char| !c.is_ascii_digit()).unwrap_or(rest.len());
        self.at += n;
        rest[..n].parse().map_err(|_| format!("expected a number at {rest:?}"))
    }

    fn string(&mut self) -> Result<String> {
        self.expect("\"")?;
        let rest = &self.s[self.at..];
        let n = rest.find('"').ok_or("an unterminated string")?;
        self.at += n + 1;
        Ok(rest[..n].to_string())
    }
}

/// The text between the outer brackets of `<...>`.
fn strip_brackets(body: &str) -> &str {
    let b = body.trim();
    b.strip_prefix('<').and_then(|b| b.strip_suffix('>')).unwrap_or(b)
}

/// The integers of an `array<i32: 1, 2>` body.
fn array_ints(body: &str) -> Option<Vec<i64>> {
    let inner = strip_brackets(body);
    let Some((_, list)) = inner.split_once(':') else { return Some(vec![]) };
    list.split(',').map(|t| int_text(t.trim())).collect()
}

/// The scalars of a `dense<...>` body, flattened in row-major order.
fn dense_elems(body: &str) -> Result<Vec<String>> {
    let inner = strip_brackets(body);
    if inner.contains('(') {
        return Err("complex dense elements: not supported yet".into());
    }
    Ok(inner.split(['[', ']', ',']).map(str::trim).filter(|t| !t.is_empty()).map(str::to_string).collect())
}

fn hex_bytes(hex: &str) -> Result<Vec<u8>> {
    if !hex.len().is_multiple_of(2) {
        return Err("hex data of odd length".into());
    }
    (0..hex.len())
        .step_by(2)
        .map(|k| u8::from_str_radix(&hex[k..k + 2], 16).map_err(|_| "bad hex data".to_string()))
        .collect()
}

/// An integer written in an attribute: decimal, or hex after `0x`.
fn int_text(text: &str) -> Option<i64> {
    let (neg, digits) = match text.strip_prefix('-') {
        Some(d) => (true, d),
        None => (false, text),
    };
    let v = match digits.strip_prefix("0x") {
        Some(hex) => u64::from_str_radix(hex, 16).ok()? as i64,
        None => digits.parse::<u64>().ok()? as i64,
    };
    Some(if neg { v.wrapping_neg() } else { v })
}

fn float_type(keyword: &str) -> Option<LType> {
    Some(match keyword {
        "f16" => LType::Half,
        "bf16" => LType::BFloat,
        "f32" => LType::Float,
        "f64" => LType::Double,
        "f80" => LType::X86Fp80,
        "f128" => LType::Fp128,
        _ => return None,
    })
}

/// `v`, `bits` wide, sign-extended to 64 bits.
fn sext(v: u64, bits: u32) -> i64 {
    if bits >= 64 { v as i64 } else { ((v << (64 - bits)) as i64) >> (64 - bits) }
}

/// An integer operation on constants, or `None` where the result is
/// poison or undefined, or the flags cannot be checked.
fn fold_int(op: BinOp, flags: IntFlags, bits: u32, x: u64, y: u64) -> Option<u64> {
    let mask = if bits == 64 { u64::MAX } else { (1u64 << bits) - 1 };
    let (sx, sy) = (sext(x, bits) as i128, sext(y, bits) as i128);
    let (ux, uy) = (x as i128, y as i128);
    let (exact_s, exact_u) = match op {
        BinOp::Add => (sx + sy, ux + uy),
        BinOp::Sub => (sx - sy, ux - uy),
        BinOp::Mul => (sx * sy, ux * uy),
        _ => (0, 0),
    };
    let r = match op {
        BinOp::Add | BinOp::Sub | BinOp::Mul => {
            let r = exact_u as u64 & mask;
            let fits_s = exact_s == sext(r, bits) as i128;
            let fits_u = exact_u == r as i128;
            if (flags.contains(IntFlags::NSW) && !fits_s) || (flags.contains(IntFlags::NUW) && !fits_u) {
                return None;
            }
            r
        }
        _ if flags != IntFlags::NONE => return None,
        BinOp::And => x & y,
        BinOp::Or => x | y,
        BinOp::Xor => x ^ y,
        BinOp::Shl if y < bits as u64 => x << y,
        BinOp::LShr if y < bits as u64 => x >> y,
        BinOp::AShr if y < bits as u64 => (sx >> y) as u64,
        BinOp::UDiv if y != 0 => x / y,
        BinOp::URem if y != 0 => x % y,
        BinOp::SDiv if sy != 0 => (sx / sy) as u64,
        BinOp::SRem if sy != 0 => (sx % sy) as u64,
        _ => return None,
    };
    Some(r & mask)
}

fn fold_float<F>(op: BinOp, x: F, y: F) -> Option<F>
where
    F: std::ops::Add<Output = F>
        + std::ops::Sub<Output = F>
        + std::ops::Mul<Output = F>
        + std::ops::Div<Output = F>
        + std::ops::Rem<Output = F>,
{
    Some(match op {
        BinOp::FAdd => x + y,
        BinOp::FSub => x - y,
        BinOp::FMul => x * y,
        BinOp::FDiv => x / y,
        BinOp::FRem => x % y,
        _ => return None,
    })
}

fn fold_fcmp(pred: FCmpPred, x: f64, y: f64) -> bool {
    let uno = x.is_nan() || y.is_nan();
    match pred {
        FCmpPred::False => false,
        FCmpPred::True => true,
        FCmpPred::Ord => !uno,
        FCmpPred::Uno => uno,
        FCmpPred::Oeq => x == y,
        FCmpPred::Ogt => x > y,
        FCmpPred::Oge => x >= y,
        FCmpPred::Olt => x < y,
        FCmpPred::Ole => x <= y,
        FCmpPred::One => !uno && x != y,
        FCmpPred::Ueq => uno || x == y,
        FCmpPred::Ugt => uno || x > y,
        FCmpPred::Uge => uno || x >= y,
        FCmpPred::Ult => uno || x < y,
        FCmpPred::Ule => uno || x <= y,
        FCmpPred::Une => x != y,
    }
}
