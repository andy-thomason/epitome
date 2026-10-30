//! The in-memory IR: modules, globals, functions, instructions, constants
//! and metadata.
//!
//! Everything lives in index-based tables. A module owns its types,
//! constants, metadata and globals. A function owns its local values,
//! blocks and instructions.

use std::collections::HashMap;
use std::num::NonZeroU8;

use crate::apint::ApInt;
use crate::types::{TypeId, Types};

macro_rules! id_type {
    ($($(#[$doc:meta])* $name:ident;)*) => {$(
        $(#[$doc])*
        #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, PartialOrd, Ord)]
        pub struct $name(pub u32);
        impl $name {
            pub fn index(self) -> usize { self.0 as usize }
        }
    )*};
}

id_type! {
    /// A global value: variable, function, alias or ifunc.
    GlobalId;
    /// An interned constant.
    ConstId;
    /// A metadata node or string.
    MdId;
    /// A local value of a function: argument, instruction result or block.
    LocalId;
    /// A basic block of a function.
    BlockId;
    /// An instruction of a function.
    InstId;
    /// A comdat.
    ComdatId;
}

/// An operand.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Value {
    Local(LocalId),
    Const(ConstId),
    /// Metadata used as a value, as in `metadata !5` in a call.
    Metadata(MdId),
}

#[derive(Clone, Debug, Default)]
pub struct Module {
    pub source_filename: Option<Vec<u8>>,
    pub data_layout: Option<Vec<u8>>,
    pub triple: Option<Vec<u8>>,
    /// Module-level assembly, a line each.
    pub module_asm: Vec<Vec<u8>>,
    /// Its options (`module asm(target_cpu: "apple-m1")`), in order.
    pub module_asm_options: Vec<(String, Vec<u8>)>,
    pub types: Types,
    pub consts: Constants,
    pub metadata: Vec<MdNode>,
    pub comdats: Vec<Comdat>,
    /// Indexed by `GlobalId`.
    pub globals: Vec<Global>,
    /// Order in which globals appear in the module.
    pub global_order: Vec<GlobalId>,
    pub attr_groups: Vec<AttrGroup>,
    pub named_metadata: Vec<NamedMetadata>,
    /// Attachment kinds that are not built in, in order of first use. See
    /// `metadata::FIXED_KINDS`.
    pub md_kinds: Vec<String>,
}

impl Module {
    pub fn global(&self, id: GlobalId) -> &Global {
        &self.globals[id.index()]
    }

    pub fn constant(&self, id: ConstId) -> &Constant {
        self.consts.get(id)
    }

    /// The functions in module order.
    pub fn functions(&self) -> impl Iterator<Item = (GlobalId, &Function)> {
        self.global_order.iter().filter_map(|&g| match &self.globals[g.index()].kind {
            GlobalKind::Function(f) => Some((g, &**f)),
            _ => None,
        })
    }

    /// The type of a value used in function `func`.
    pub fn value_type(&mut self, func: Option<&Function>, v: Value) -> TypeId {
        match v {
            Value::Local(l) => func.expect("local value outside a function").locals[l.index()].ty,
            Value::Const(c) => self.consts.get(c).ty,
            Value::Metadata(_) => self.types.metadata(),
        }
    }
}

// ---------------------------------------------------------------------------
// Constants

#[derive(Clone, Debug, Default)]
pub struct Constants {
    list: Vec<Constant>,
    map: HashMap<Constant, ConstId>,
}

impl Constants {
    pub fn intern(&mut self, c: Constant) -> ConstId {
        if let Some(&id) = self.map.get(&c) {
            return id;
        }
        let id = ConstId(self.list.len() as u32);
        self.list.push(c.clone());
        self.map.insert(c, id);
        id
    }

    pub fn get(&self, id: ConstId) -> &Constant {
        &self.list[id.index()]
    }

    /// The id of an already interned constant.
    pub fn lookup(&self, c: &Constant) -> Option<ConstId> {
        self.map.get(c).copied()
    }

    pub fn len(&self) -> usize {
        self.list.len()
    }

    pub fn is_empty(&self) -> bool {
        self.list.is_empty()
    }
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Constant {
    pub ty: TypeId,
    pub kind: ConstKind,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum ConstKind {
    Int(ApInt),
    /// Raw bits in the type's format (see `float`).
    Float(u128),
    Null,
    /// The `none` token.
    NoneToken,
    Undef,
    Poison,
    Zero,
    Array(Vec<ConstId>),
    Struct(Vec<ConstId>),
    Vector(Vec<ConstId>),
    /// An array of `i8` written as `c"..."`.
    Bytes(Vec<u8>),
    Splat(ConstId),
    /// The address of a global value.
    Global(GlobalId),
    BlockAddress {
        func: GlobalId,
        block: BlockId,
    },
    DsoLocalEquivalent(GlobalId),
    NoCfi(GlobalId),
    PtrAuth(Vec<ConstId>),
    Expr(ConstExpr),
    InlineAsm(InlineAsm),
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum ConstExpr {
    /// The result type is the constant's type.
    Cast {
        op: CastOp,
        value: ConstId,
    },
    Gep {
        flags: GepFlags,
        inrange: Option<(i64, i64)>,
        source_ty: TypeId,
        ptr: ConstId,
        indices: Vec<ConstId>,
    },
    Binary {
        op: BinOp,
        flags: IntFlags,
        lhs: ConstId,
        rhs: ConstId,
    },
    ExtractElement {
        vector: ConstId,
        index: ConstId,
    },
    InsertElement {
        vector: ConstId,
        element: ConstId,
        index: ConstId,
    },
    ShuffleVector {
        a: ConstId,
        b: ConstId,
        mask: ConstId,
    },
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct InlineAsm {
    pub fn_ty: TypeId,
    pub side_effect: bool,
    pub align_stack: bool,
    pub intel_dialect: bool,
    pub unwind: bool,
    pub asm: Vec<u8>,
    pub constraints: Vec<u8>,
}

// ---------------------------------------------------------------------------
// Metadata

#[derive(Clone, Debug)]
pub struct MdNode {
    /// The number in `!N = ...` in the source. Printing renumbers nodes, so
    /// this is only the name the node was read under.
    pub slot: Option<u32>,
    pub data: Metadata,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Metadata {
    /// `!{...}`. `None` operands are `null`.
    Tuple { distinct: bool, ops: Vec<Option<MdId>> },
    /// A specialized node such as `!DILocation(line: 3, scope: !5)`.
    Node { distinct: bool, kind: String, fields: Vec<MdField> },
    /// `!"..."`
    String(Vec<u8>),
    /// A typed value used as metadata, such as `i32 1` or `ptr %x`.
    Value(Value),
    /// A numbered node that was referenced but not yet defined. Only seen
    /// while parsing.
    Forward,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct MdField {
    /// `None` for positional operands, as in `!DIExpression(DW_OP_deref)`.
    pub name: Option<String>,
    pub value: MdFieldValue,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum MdFieldValue {
    Null,
    Md(MdId),
    Str(Vec<u8>),
    /// Integers, booleans, enumerators and flag sets such as
    /// `DIFlagPrototyped | DIFlagAllCallsDescribed`, kept as written.
    Word(String),
    Value(Value),
}

#[derive(Clone, Debug)]
pub struct NamedMetadata {
    pub name: String,
    pub ops: Vec<MdId>,
}

/// Metadata attached to an instruction, function or global: `!dbg !5`.
#[derive(Clone, Debug)]
pub struct Attachment {
    pub kind: String,
    pub node: MdId,
}

// ---------------------------------------------------------------------------
// Attributes

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Attr {
    /// A keyword attribute such as `nounwind`.
    Word(String),
    /// An integer attribute: `align 8`, `dereferenceable(24)` or `alignstack=16`.
    Int { name: String, value: u64, style: IntStyle },
    /// A type attribute such as `sret(%struct.S)`.
    Type { name: String, ty: TypeId },
    /// Any other attribute with an argument list, such as `memory(read)` or
    /// `range(i32 0, 10)`. The text between the parentheses is kept as written.
    Parens { name: String, text: String },
    /// `"key"` or `"key"="value"`.
    Str { key: Vec<u8>, value: Option<Vec<u8>> },
    /// A reference to an attribute group, `#N`.
    Group(u32),
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum IntStyle {
    Space,
    Paren,
    Eq,
}

#[derive(Clone, Debug)]
pub struct AttrGroup {
    pub id: u32,
    pub attrs: Vec<Attr>,
}

// ---------------------------------------------------------------------------
// Globals

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub enum Linkage {
    #[default]
    External,
    Private,
    Internal,
    AvailableExternally,
    LinkOnce,
    Weak,
    Common,
    Appending,
    ExternWeak,
    LinkOnceOdr,
    WeakOdr,
}

impl Linkage {
    pub const ALL: [(&'static str, Linkage); 11] = [
        ("private", Linkage::Private),
        ("internal", Linkage::Internal),
        ("available_externally", Linkage::AvailableExternally),
        ("linkonce", Linkage::LinkOnce),
        ("weak", Linkage::Weak),
        ("common", Linkage::Common),
        ("appending", Linkage::Appending),
        ("extern_weak", Linkage::ExternWeak),
        ("linkonce_odr", Linkage::LinkOnceOdr),
        ("weak_odr", Linkage::WeakOdr),
        ("external", Linkage::External),
    ];

    pub fn keyword(self) -> &'static str {
        Self::ALL.iter().find(|(_, l)| *l == self).unwrap().0
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Visibility {
    #[default]
    Default,
    Hidden,
    Protected,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum DllStorage {
    #[default]
    None,
    Import,
    Export,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ThreadLocal {
    GeneralDynamic,
    LocalDynamic,
    InitialExec,
    LocalExec,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum UnnamedAddr {
    #[default]
    None,
    Local,
    Global,
}

/// Properties shared by all global values.
#[derive(Clone, Debug, Default)]
pub struct GlobalProps {
    pub linkage: Linkage,
    /// The linkage keyword was written explicitly.
    pub explicit_linkage: bool,
    pub dso_local: bool,
    pub visibility: Visibility,
    pub dll_storage: DllStorage,
    pub thread_local: Option<ThreadLocal>,
    pub unnamed_addr: UnnamedAddr,
    pub addrspace: Option<u32>,
    pub partition: Option<Vec<u8>>,
}

#[derive(Clone, Debug)]
pub struct Global {
    /// `None` for a numbered global such as `@0`.
    pub name: Option<String>,
    pub props: GlobalProps,
    pub attachments: Vec<Attachment>,
    pub kind: GlobalKind,
}

#[derive(Clone, Debug)]
pub enum GlobalKind {
    Variable(GlobalVar),
    Function(Box<Function>),
    Alias {
        value_ty: TypeId,
        aliasee: ConstId,
    },
    IFunc {
        value_ty: TypeId,
        resolver: ConstId,
    },
    /// Referenced but not yet defined. Only seen while parsing.
    Forward,
}

#[derive(Clone, Debug)]
pub struct GlobalVar {
    pub constant: bool,
    pub externally_initialized: bool,
    pub ty: TypeId,
    pub init: Option<ConstId>,
    pub section: Option<Vec<u8>>,
    pub comdat: Option<ComdatRef>,
    pub align: Option<u64>,
    pub code_model: Option<Vec<u8>>,
    /// `no_sanitize_address` and the like, in source order.
    pub sanitizer: Vec<String>,
    pub attrs: Vec<Attr>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ComdatRef {
    /// `comdat`: the comdat with the global's own name.
    Own,
    /// `comdat($name)`
    Named(ComdatId),
}

#[derive(Clone, Debug)]
pub struct Comdat {
    pub name: String,
    /// `None` while only referenced.
    pub kind: Option<ComdatKind>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ComdatKind {
    Any,
    ExactMatch,
    Largest,
    NoDeduplicate,
    SameSize,
}

impl ComdatKind {
    pub const ALL: [(&'static str, ComdatKind); 5] = [
        ("any", ComdatKind::Any),
        ("exactmatch", ComdatKind::ExactMatch),
        ("largest", ComdatKind::Largest),
        ("nodeduplicate", ComdatKind::NoDeduplicate),
        ("samesize", ComdatKind::SameSize),
    ];
}

/// A calling convention, as written: a keyword such as `fastcc` or `cc 11`.
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub enum CallConv {
    #[default]
    C,
    Named(String),
    Numbered(u32),
}

/// The named calling conventions listed in the Language Reference. Others
/// can be written by number, as `cc <n>`.
pub const CALLING_CONVENTIONS: &[&str] = &[
    "ccc",
    "fastcc",
    "coldcc",
    "ghccc",
    "anyregcc",
    "preserve_mostcc",
    "preserve_allcc",
    "preserve_nonecc",
    "cxx_fast_tlscc",
    "tailcc",
    "swiftcc",
    "swifttailcc",
    "cfguard_checkcc",
];

// ---------------------------------------------------------------------------
// Functions

#[derive(Clone, Debug, Default)]
pub struct Function {
    /// The function type.
    pub ty: TypeId,
    pub cconv: CallConv,
    pub ret_attrs: Vec<Attr>,
    pub params: Vec<Param>,
    pub fn_attrs: Vec<Attr>,
    pub section: Option<Vec<u8>>,
    pub comdat: Option<ComdatRef>,
    pub align: Option<u64>,
    pub prefalign: Option<u64>,
    pub gc: Option<Vec<u8>>,
    pub prefix: Option<ConstId>,
    pub prologue: Option<ConstId>,
    pub personality: Option<ConstId>,
    /// False for a declaration.
    pub has_body: bool,
    /// Indexed by `LocalId`. Parameters come first.
    pub locals: Vec<Local>,
    /// Indexed by `BlockId`.
    pub blocks: Vec<Block>,
    /// Block order in the function body.
    pub layout: Vec<BlockId>,
    /// Indexed by `InstId`.
    pub insts: Vec<Inst>,
}

#[derive(Clone, Debug)]
pub struct Param {
    pub local: LocalId,
    pub attrs: Vec<Attr>,
}

#[derive(Clone, Debug)]
pub struct Local {
    /// `None` for a numbered value such as `%3`.
    pub name: Option<String>,
    pub ty: TypeId,
    pub def: LocalDef,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum LocalDef {
    Param(u32),
    Inst(InstId),
    Block(BlockId),
    /// Used before being defined. Only seen while parsing.
    Forward,
}

#[derive(Clone, Debug)]
pub struct Block {
    pub local: LocalId,
    pub insts: Vec<InstId>,
}

#[derive(Clone, Debug)]
pub struct Inst {
    pub result: Option<LocalId>,
    pub op: Op,
    pub attachments: Vec<Attachment>,
    /// Debug records written before this instruction.
    pub debug_records: Vec<DebugRecord>,
}

/// `#dbg_value(...)` and its relatives.
#[derive(Clone, Debug)]
pub struct DebugRecord {
    /// The part after `#dbg_`, such as `value` or `declare`.
    pub kind: String,
    pub args: Vec<MdId>,
}

// ---------------------------------------------------------------------------
// Instructions

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum BinOp {
    Add,
    FAdd,
    Sub,
    FSub,
    Mul,
    FMul,
    UDiv,
    SDiv,
    FDiv,
    URem,
    SRem,
    FRem,
    Shl,
    LShr,
    AShr,
    And,
    Or,
    Xor,
}

impl BinOp {
    pub const ALL: [(&'static str, BinOp); 18] = [
        ("add", BinOp::Add),
        ("fadd", BinOp::FAdd),
        ("sub", BinOp::Sub),
        ("fsub", BinOp::FSub),
        ("mul", BinOp::Mul),
        ("fmul", BinOp::FMul),
        ("udiv", BinOp::UDiv),
        ("sdiv", BinOp::SDiv),
        ("fdiv", BinOp::FDiv),
        ("urem", BinOp::URem),
        ("srem", BinOp::SRem),
        ("frem", BinOp::FRem),
        ("shl", BinOp::Shl),
        ("lshr", BinOp::LShr),
        ("ashr", BinOp::AShr),
        ("and", BinOp::And),
        ("or", BinOp::Or),
        ("xor", BinOp::Xor),
    ];

    pub fn keyword(self) -> &'static str {
        Self::ALL.iter().find(|(_, o)| *o == self).unwrap().0
    }

    pub fn is_float(self) -> bool {
        matches!(self, BinOp::FAdd | BinOp::FSub | BinOp::FMul | BinOp::FDiv | BinOp::FRem)
    }

    /// The integer flags this operation accepts.
    pub fn allowed_flags(self) -> IntFlags {
        match self {
            BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Shl => IntFlags::NUW | IntFlags::NSW,
            BinOp::UDiv | BinOp::SDiv | BinOp::LShr | BinOp::AShr => IntFlags::EXACT,
            BinOp::Or => IntFlags::DISJOINT,
            _ => IntFlags::NONE,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum CastOp {
    Trunc,
    ZExt,
    SExt,
    FPTrunc,
    FPExt,
    FPToUI,
    FPToSI,
    UIToFP,
    SIToFP,
    PtrToInt,
    PtrToAddr,
    IntToPtr,
    BitCast,
    AddrSpaceCast,
}

impl CastOp {
    pub const ALL: [(&'static str, CastOp); 14] = [
        ("trunc", CastOp::Trunc),
        ("zext", CastOp::ZExt),
        ("sext", CastOp::SExt),
        ("fptrunc", CastOp::FPTrunc),
        ("fpext", CastOp::FPExt),
        ("fptoui", CastOp::FPToUI),
        ("fptosi", CastOp::FPToSI),
        ("uitofp", CastOp::UIToFP),
        ("sitofp", CastOp::SIToFP),
        ("ptrtoint", CastOp::PtrToInt),
        ("ptrtoaddr", CastOp::PtrToAddr),
        ("inttoptr", CastOp::IntToPtr),
        ("bitcast", CastOp::BitCast),
        ("addrspacecast", CastOp::AddrSpaceCast),
    ];

    pub fn keyword(self) -> &'static str {
        Self::ALL.iter().find(|(_, o)| *o == self).unwrap().0
    }

    /// The integer flags this cast accepts.
    pub fn allowed_flags(self) -> IntFlags {
        match self {
            CastOp::Trunc => IntFlags::NUW | IntFlags::NSW,
            CastOp::ZExt | CastOp::UIToFP => IntFlags::NNEG,
            CastOp::AddrSpaceCast => IntFlags::NONNULL,
            _ => IntFlags::NONE,
        }
    }

    /// Casts that accept fast-math flags.
    pub fn allows_fmf(self) -> bool {
        matches!(self, CastOp::FPTrunc | CastOp::FPExt | CastOp::UIToFP | CastOp::SIToFP)
    }
}

/// Integer-style flags on instructions. Which are valid depends on the opcode.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub struct IntFlags(pub u8);

impl IntFlags {
    pub const NONE: IntFlags = IntFlags(0);
    pub const NUW: IntFlags = IntFlags(1);
    pub const NSW: IntFlags = IntFlags(2);
    pub const EXACT: IntFlags = IntFlags(4);
    pub const DISJOINT: IntFlags = IntFlags(8);
    pub const NNEG: IntFlags = IntFlags(16);
    pub const NONNULL: IntFlags = IntFlags(32);

    /// In print order.
    pub const NAMES: [(&'static str, IntFlags); 6] = [
        ("nuw", IntFlags::NUW),
        ("nsw", IntFlags::NSW),
        ("exact", IntFlags::EXACT),
        ("disjoint", IntFlags::DISJOINT),
        ("nneg", IntFlags::NNEG),
        ("nonnull", IntFlags::NONNULL),
    ];

    pub fn contains(self, other: IntFlags) -> bool {
        self.0 & other.0 == other.0
    }
}

impl std::ops::BitOr for IntFlags {
    type Output = IntFlags;
    fn bitor(self, rhs: IntFlags) -> IntFlags {
        IntFlags(self.0 | rhs.0)
    }
}

/// Fast-math flags.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub struct FastMath(pub u8);

impl FastMath {
    pub const ALL: FastMath = FastMath(0x7f);
    /// In print order.
    pub const NAMES: [(&'static str, u8); 7] =
        [("nnan", 1), ("ninf", 2), ("nsz", 4), ("arcp", 8), ("contract", 16), ("afn", 32), ("reassoc", 64)];

    pub fn is_empty(self) -> bool {
        self.0 == 0
    }
}

/// The alignment of a memory instruction: a power of two, in bytes. It is
/// kept as its log2 plus one, so that `Option<Align>` is one byte.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, PartialOrd, Ord)]
pub struct Align(NonZeroU8);

impl Align {
    /// The largest alignment LLVM allows, 2^32.
    pub const MAX_LOG2: u8 = 32;

    /// `None` unless `bytes` is a power of two no larger than 2^32.
    pub fn new(bytes: u64) -> Option<Align> {
        if !bytes.is_power_of_two() || bytes.trailing_zeros() > Self::MAX_LOG2 as u32 {
            return None;
        }
        Some(Self::from_log2(bytes.trailing_zeros() as u8))
    }

    /// `bytes`, which must be a power of two no larger than 2^32.
    pub const fn of(bytes: u64) -> Align {
        assert!(bytes.is_power_of_two() && bytes.trailing_zeros() <= Self::MAX_LOG2 as u32);
        Align(NonZeroU8::new(bytes.trailing_zeros() as u8 + 1).unwrap())
    }

    /// This alignment, or the largest power of two no larger than `bytes`
    /// if that is less.
    pub fn at_most(self, bytes: u64) -> Align {
        self.min(Self::from_log2(bytes.max(1).ilog2().min(Self::MAX_LOG2 as u32) as u8))
    }

    pub fn from_log2(log2: u8) -> Align {
        assert!(log2 <= Self::MAX_LOG2, "alignment 2^{log2} is too large");
        Align(NonZeroU8::new(log2 + 1).unwrap())
    }

    pub fn log2(self) -> u8 {
        self.0.get() - 1
    }

    pub fn bytes(self) -> u64 {
        1 << self.log2()
    }
}

impl std::fmt::Display for Align {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.bytes())
    }
}

/// `getelementptr` flags.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub struct GepFlags(pub u8);

impl GepFlags {
    pub const INBOUNDS: u8 = 1;
    pub const NUSW: u8 = 2;
    pub const NUW: u8 = 4;
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum ICmpPred {
    Eq,
    Ne,
    Ugt,
    Uge,
    Ult,
    Ule,
    Sgt,
    Sge,
    Slt,
    Sle,
}

impl ICmpPred {
    pub const ALL: [(&'static str, ICmpPred); 10] = [
        ("eq", ICmpPred::Eq),
        ("ne", ICmpPred::Ne),
        ("ugt", ICmpPred::Ugt),
        ("uge", ICmpPred::Uge),
        ("ult", ICmpPred::Ult),
        ("ule", ICmpPred::Ule),
        ("sgt", ICmpPred::Sgt),
        ("sge", ICmpPred::Sge),
        ("slt", ICmpPred::Slt),
        ("sle", ICmpPred::Sle),
    ];

    pub fn keyword(self) -> &'static str {
        Self::ALL.iter().find(|(_, p)| *p == self).unwrap().0
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum FCmpPred {
    False,
    Oeq,
    Ogt,
    Oge,
    Olt,
    Ole,
    One,
    Ord,
    Ueq,
    Ugt,
    Uge,
    Ult,
    Ule,
    Une,
    Uno,
    True,
}

impl FCmpPred {
    pub const ALL: [(&'static str, FCmpPred); 16] = [
        ("false", FCmpPred::False),
        ("oeq", FCmpPred::Oeq),
        ("ogt", FCmpPred::Ogt),
        ("oge", FCmpPred::Oge),
        ("olt", FCmpPred::Olt),
        ("ole", FCmpPred::Ole),
        ("one", FCmpPred::One),
        ("ord", FCmpPred::Ord),
        ("ueq", FCmpPred::Ueq),
        ("ugt", FCmpPred::Ugt),
        ("uge", FCmpPred::Uge),
        ("ult", FCmpPred::Ult),
        ("ule", FCmpPred::Ule),
        ("une", FCmpPred::Une),
        ("uno", FCmpPred::Uno),
        ("true", FCmpPred::True),
    ];

    pub fn keyword(self) -> &'static str {
        Self::ALL.iter().find(|(_, p)| *p == self).unwrap().0
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Ordering {
    Unordered,
    Monotonic,
    Acquire,
    Release,
    AcqRel,
    SeqCst,
}

impl Ordering {
    pub const ALL: [(&'static str, Ordering); 6] = [
        ("unordered", Ordering::Unordered),
        ("monotonic", Ordering::Monotonic),
        ("acquire", Ordering::Acquire),
        ("release", Ordering::Release),
        ("acq_rel", Ordering::AcqRel),
        ("seq_cst", Ordering::SeqCst),
    ];

    pub fn keyword(self) -> &'static str {
        Self::ALL.iter().find(|(_, o)| *o == self).unwrap().0
    }
}

/// The atomic part of a `load`, `store` or `fence`.
#[derive(Clone, Debug)]
pub struct Atomic {
    pub elementwise: bool,
    /// `syncscope("...")`
    pub scope: Option<Box<[u8]>>,
    pub ordering: Ordering,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum RmwOp {
    Xchg,
    Add,
    Sub,
    And,
    Nand,
    Or,
    Xor,
    Max,
    Min,
    UMax,
    UMin,
    FAdd,
    FSub,
    FMax,
    FMin,
    FMaximum,
    FMinimum,
    FMaximumNum,
    FMinimumNum,
    UIncWrap,
    UDecWrap,
    USubCond,
    USubSat,
}

impl RmwOp {
    pub const ALL: [(&'static str, RmwOp); 23] = [
        ("xchg", RmwOp::Xchg),
        ("add", RmwOp::Add),
        ("sub", RmwOp::Sub),
        ("and", RmwOp::And),
        ("nand", RmwOp::Nand),
        ("or", RmwOp::Or),
        ("xor", RmwOp::Xor),
        ("max", RmwOp::Max),
        ("min", RmwOp::Min),
        ("umax", RmwOp::UMax),
        ("umin", RmwOp::UMin),
        ("fadd", RmwOp::FAdd),
        ("fsub", RmwOp::FSub),
        ("fmax", RmwOp::FMax),
        ("fmin", RmwOp::FMin),
        ("fmaximum", RmwOp::FMaximum),
        ("fminimum", RmwOp::FMinimum),
        ("fmaximumnum", RmwOp::FMaximumNum),
        ("fminimumnum", RmwOp::FMinimumNum),
        ("uinc_wrap", RmwOp::UIncWrap),
        ("udec_wrap", RmwOp::UDecWrap),
        ("usub_cond", RmwOp::USubCond),
        ("usub_sat", RmwOp::USubSat),
    ];

    pub fn keyword(self) -> &'static str {
        Self::ALL.iter().find(|(_, o)| *o == self).unwrap().0
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum TailKind {
    #[default]
    None,
    Tail,
    MustTail,
    NoTail,
}

/// The parts shared by `call`, `invoke` and `callbr`.
#[derive(Clone, Debug)]
pub struct Call {
    pub tail: TailKind,
    pub fmf: FastMath,
    pub cconv: CallConv,
    pub ret_attrs: Vec<Attr>,
    pub addrspace: Option<u32>,
    /// The function type of the callee.
    pub fn_ty: TypeId,
    pub callee: Value,
    pub args: Vec<CallArg>,
    pub fn_attrs: Vec<Attr>,
    pub bundles: Vec<Bundle>,
}

#[derive(Clone, Debug)]
pub struct CallArg {
    pub value: Value,
    pub attrs: Vec<Attr>,
}

/// An operand bundle: `"deopt"(i32 10)`.
#[derive(Clone, Debug)]
pub struct Bundle {
    pub tag: Vec<u8>,
    pub inputs: Vec<Value>,
}

#[derive(Clone, Debug)]
pub enum Clause {
    Catch(Value),
    Filter(Value),
}

#[derive(Clone, Debug)]
pub enum Op {
    // Terminators
    Ret(Option<Value>),
    Br(BlockId),
    CondBr {
        cond: Value,
        then_dest: BlockId,
        else_dest: BlockId,
    },
    Switch {
        value: Value,
        default: BlockId,
        cases: Box<[(ConstId, BlockId)]>,
    },
    IndirectBr {
        addr: Value,
        dests: Box<[BlockId]>,
    },
    Invoke {
        call: Box<Call>,
        normal: BlockId,
        unwind: BlockId,
    },
    CallBr {
        call: Box<Call>,
        default: BlockId,
        indirect: Box<[BlockId]>,
    },
    Resume(Value),
    /// `parent` is `None` for `within none`; `unwind` is `None` for `unwind to caller`.
    CatchSwitch {
        parent: Option<Value>,
        handlers: Box<[BlockId]>,
        unwind: Option<BlockId>,
    },
    CatchRet {
        pad: Value,
        dest: BlockId,
    },
    CleanupRet {
        pad: Value,
        unwind: Option<BlockId>,
    },
    Unreachable,

    // Arithmetic
    FNeg {
        fmf: FastMath,
        value: Value,
    },
    Binary {
        op: BinOp,
        flags: IntFlags,
        fmf: FastMath,
        lhs: Value,
        rhs: Value,
    },
    BitExtract {
        value: Value,
        offset: Value,
    },
    BitInsert {
        base: Value,
        value: Value,
        offset: Value,
    },

    // Vectors and aggregates
    ExtractElement {
        vector: Value,
        index: Value,
    },
    InsertElement {
        vector: Value,
        element: Value,
        index: Value,
    },
    ShuffleVector {
        a: Value,
        b: Value,
        mask: ConstId,
    },
    ExtractValue {
        aggregate: Value,
        indices: Box<[u32]>,
    },
    InsertValue {
        aggregate: Value,
        element: Value,
        indices: Box<[u32]>,
    },

    // Memory
    Alloca {
        inalloca: bool,
        ty: TypeId,
        count: Option<Value>,
        align: Option<Align>,
        addrspace: Option<u32>,
    },
    Load {
        volatile: bool,
        atomic: Option<Atomic>,
        ptr: Value,
        align: Option<Align>,
    },
    Store {
        volatile: bool,
        atomic: Option<Atomic>,
        value: Value,
        ptr: Value,
        align: Option<Align>,
    },
    Fence {
        scope: Option<Box<[u8]>>,
        ordering: Ordering,
    },
    CmpXchg {
        weak: bool,
        volatile: bool,
        ptr: Value,
        cmp: Value,
        new: Value,
        scope: Option<Box<[u8]>>,
        success: Ordering,
        failure: Ordering,
        align: Option<Align>,
    },
    AtomicRmw {
        op: RmwOp,
        volatile: bool,
        elementwise: bool,
        ptr: Value,
        value: Value,
        scope: Option<Box<[u8]>>,
        ordering: Ordering,
        align: Option<Align>,
    },
    Gep {
        flags: GepFlags,
        source_ty: TypeId,
        ptr: Value,
        indices: Box<[Value]>,
    },

    // Other
    Cast {
        op: CastOp,
        flags: IntFlags,
        fmf: FastMath,
        value: Value,
    },
    ICmp {
        pred: ICmpPred,
        samesign: bool,
        lhs: Value,
        rhs: Value,
    },
    FCmp {
        pred: FCmpPred,
        fmf: FastMath,
        lhs: Value,
        rhs: Value,
    },
    Phi {
        fmf: FastMath,
        incoming: Vec<(Value, BlockId)>,
    },
    Select {
        fmf: FastMath,
        cond: Value,
        then_value: Value,
        else_value: Value,
    },
    Freeze(Value),
    Call(Box<Call>),
    VaArg {
        list: Value,
    },
    LandingPad {
        cleanup: bool,
        clauses: Box<[Clause]>,
    },
    /// `within` is the parent pad (a `catchswitch` for `catchpad`), or `None` for `within none`.
    CatchPad {
        within: Value,
        args: Box<[Value]>,
    },
    CleanupPad {
        within: Option<Value>,
        args: Box<[Value]>,
    },
}

impl Op {
    /// The call of a `call`, `invoke` or `callbr`.
    pub fn call(&self) -> Option<&Call> {
        match self {
            Op::Call(c) | Op::Invoke { call: c, .. } | Op::CallBr { call: c, .. } => Some(c),
            _ => None,
        }
    }

    pub fn call_mut(&mut self) -> Option<&mut Call> {
        match self {
            Op::Call(c) | Op::Invoke { call: c, .. } | Op::CallBr { call: c, .. } => Some(c),
            _ => None,
        }
    }

    pub fn is_terminator(&self) -> bool {
        matches!(
            self,
            Op::Ret(_)
                | Op::Br(_)
                | Op::CondBr { .. }
                | Op::Switch { .. }
                | Op::IndirectBr { .. }
                | Op::Invoke { .. }
                | Op::CallBr { .. }
                | Op::Resume(_)
                | Op::CatchSwitch { .. }
                | Op::CatchRet { .. }
                | Op::CleanupRet { .. }
                | Op::Unreachable
        )
    }

    /// The blocks this terminator may branch to, in operand order.
    pub fn successors(&self) -> Vec<BlockId> {
        match self {
            Op::Br(b) => vec![*b],
            Op::CondBr { then_dest, else_dest, .. } => vec![*then_dest, *else_dest],
            Op::Switch { default, cases, .. } => std::iter::once(*default).chain(cases.iter().map(|c| c.1)).collect(),
            Op::IndirectBr { dests, .. } => dests.to_vec(),
            Op::Invoke { normal, unwind, .. } => vec![*normal, *unwind],
            Op::CallBr { default, indirect, .. } => std::iter::once(*default).chain(indirect.iter().copied()).collect(),
            Op::CatchSwitch { handlers, unwind, .. } => handlers.iter().copied().chain(*unwind).collect(),
            Op::CatchRet { dest, .. } => vec![*dest],
            Op::CleanupRet { unwind, .. } => unwind.iter().copied().collect(),
            _ => Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every instruction holds an `Op`, so its size matters: keep rare,
    /// large operands behind a pointer.
    #[test]
    fn op_is_small() {
        assert!(size_of::<Op>() <= 64, "Op is {} bytes", size_of::<Op>());
        assert_eq!(size_of::<Option<Align>>(), 1);
    }

    #[test]
    fn alignments() {
        assert_eq!(Align::new(0), None);
        assert_eq!(Align::new(12), None);
        assert_eq!(Align::new(1 << 33), None);
        assert_eq!(Align::new(16).map(Align::bytes), Some(16));
        assert_eq!(Align::of(1 << 32).log2(), 32);
        assert_eq!(Align::of(16).at_most(12), Align::of(8));
        assert_eq!(Align::of(4).at_most(64), Align::of(4));
        assert_eq!(Align::of(8).to_string(), "8");
    }
}
