//! Reading a machine description and building its selection patterns.
//!
//! See `docs/design/isel.md`. A description is an LLVM IR module; functions
//! with an `"enc"` attribute are instructions, functions with `"cond"` are
//! condition forms, and functions with `"recipe"` are recipes. Their bodies
//! become pattern trees, by evaluating each body with its parameters as
//! leaves: condition forms and flag-setting instructions are put in for
//! `"cond"` operands, and the trees are simplified as they are built.

use std::collections::HashMap;

use epitome_ir::module::*;
use epitome_ir::types::{Type, TypeId};

use crate::enc::{self, Encoding, Env};
use crate::machine::Holds;

/// A register class: its index in the description's list of them
/// (`machine.rs` reads them: their registers, file, width and what they
/// hold).
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub struct Class(pub u8);

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Kind {
    Reg(Class),
    Imm,
    Label,
    Sym,
    Cond,
    /// A value of the machine's enum with this index, written by name
    /// (`machine.rs`).
    Enum(u16),
}

#[derive(Clone, Debug)]
pub struct Operand {
    pub name: String,
    pub kind: Kind,
    /// Bits of the value's IR type: immediates are compared at this width.
    pub bits: u32,
    pub tied: bool,
    /// Read and written: the base register of a post-increment access.
    pub writeback: bool,
}

/// An instruction's encoding, from its text: parsed as the description is
/// read, or (with "lazy-encodings") when first used, so that a description
/// of tens of thousands of entries loads without parsing them all.
#[derive(Clone, Debug)]
pub struct LazyEncoding {
    name: String,
    text: String,
    fragments: std::sync::Arc<enc::Fragments>,
    cell: std::sync::OnceLock<Result<Encoding, String>>,
}

impl LazyEncoding {
    pub fn new(name: &str, text: &str, fragments: std::sync::Arc<enc::Fragments>) -> LazyEncoding {
        LazyEncoding { name: name.into(), text: text.into(), fragments, cell: Default::default() }
    }

    /// The encoding, or why its text does not parse.
    pub fn parsed(&self) -> Result<&Encoding, String> {
        self.cell
            .get_or_init(|| enc::parse_with(&self.text, &self.fragments).map_err(|e| format!("@{}: {e}", self.name)))
            .as_ref()
            .map_err(Clone::clone)
    }
}

impl std::ops::Deref for LazyEncoding {
    type Target = Encoding;
    /// (A description read lazily has its encodings checked by
    /// `--check-target`; one that does not parse is a bug in it.)
    fn deref(&self) -> &Encoding {
        self.parsed().unwrap_or_else(|e| panic!("{e}"))
    }
}

#[derive(Clone, Debug)]
pub struct Instr {
    pub name: String,
    /// The result first, called `d`, if there is one; then the parameters.
    pub operands: Vec<Operand>,
    pub has_result: bool,
    pub enc: LazyEncoding,
    pub asm: String,
    pub decode: bool,
    /// The result is zero above this many bits (64: nothing known).
    pub zext: u32,
    pub copy: Option<Class>,
    pub role: Option<String>,
    /// Which flags it sets, if it is a comparison.
    pub sets: Option<String>,
    pub matchable: bool,
    /// For assembly and disassembly only: its body says nothing (entries
    /// written from Arm's specification, `80-asm.ll`).
    pub asm_only: bool,
    /// The object format it is for, if only one (`"format"="macho"`: its
    /// relocation is that format's only).
    pub format: Option<crate::abi::Syntax>,
    /// Other mnemonics it may be written with, by Arm's specification
    /// (`cinc`, of `csinc`): for comparing with other disassemblers.
    pub aliases: Vec<String>,
    /// For a conditional branch: the branch taken when this one is not.
    pub inverse: Option<String>,
    /// How an object file fixes up its symbol operand.
    pub reloc: Option<crate::object::RelocKind>,
    /// Operand 1 (`d2`) is a second register result.
    pub second_result: bool,
    /// Registers it writes besides its operands (`"clobbers"`, as a
    /// convention lists registers: x86's division writes rdx, SME's
    /// smstart the d registers), and whether it writes the flags (`flags`
    /// in that list), where the machine's arithmetic does: the allocator
    /// keeps values out of them across it, and the frame saves those a
    /// function must keep.
    pub clobbers: Vec<u8>,
    pub clobbers_flags: bool,
    /// The features it needs (see `features.rs`).
    pub features: Vec<String>,
    /// Operands `(base, offset)` whose address it uses is `base + offset`,
    /// the offset an immediate: a load or store whose body starts
    /// `getelementptr i8, ptr %base, i64 %offset`, or an add of an
    /// immediate (`add %base, %offset`). Found from the body.
    pub base_offset: Option<(usize, usize)>,
    /// Its value is that address itself (`add` or `getelementptr` of base
    /// and offset, returned): an address computation, not an access.
    pub address_of: bool,
    /// It reads or writes memory, or calls: its body loads, stores, calls,
    /// fences or makes an atomic access.
    pub memory: bool,
    pub func: GlobalId,
}

impl Instr {
    /// The operands with fields of their own: a tied operand is held in
    /// the result's field.
    pub fn operand_names(&self) -> Vec<String> {
        self.operands.iter().filter(|o| !o.tied).map(|o| o.name.clone()).collect()
    }

    /// The symbol operands no field holds by itself (a relocation marking
    /// the instruction, as TLSDESC's call is marked, or a symbol added to a
    /// displacement): taken as 0 when decoding.
    pub fn free(&self) -> Vec<String> {
        if !self.operands.iter().any(|o| o.kind == Kind::Sym) {
            return Vec::new();
        }
        self.operands
            .iter()
            .filter(|o| {
                o.kind == Kind::Sym && !self.enc.fields().any(|f| f.names.as_slice() == std::slice::from_ref(&o.name))
            })
            .map(|o| o.name.clone())
            .collect()
    }

    /// The bytes for these operand values, and which alternative of the
    /// encoding they are, or `None` if they cannot be encoded.
    pub fn encode_alt(&self, env: &Env) -> Option<(Vec<u8>, usize)> {
        if let [only] = &self.enc.alts[..] {
            return only.encode(env).map(|b| (b, 0));
        }
        self.enc.encode_alt(env, &self.free())
    }

    /// As `encode_alt`, by an alternative `len` bytes long.
    pub fn encode_alt_of_len(&self, env: &Env, len: u32) -> Option<(Vec<u8>, usize)> {
        self.enc.encode_alt_of(env, &self.free(), Some(len))
    }

    /// The bytes for these operand values, or `None` if they cannot be
    /// encoded.
    pub fn encode(&self, env: &Env) -> Option<Vec<u8>> {
        self.encode_alt(env).map(|(b, _)| b)
    }

    /// Where in alternative `alt` the field holding operand `name` is (its
    /// piece's offset), and its value with these operands, `name` at 0: a
    /// relocation's place and addend.
    pub fn field_of(&self, alt: usize, name: &str, env: &Env) -> (usize, i64) {
        let a = &self.enc.alts[alt];
        match a.fields.iter().find(|x| x.names.iter().any(|n| n == name)) {
            Some(x) => {
                let mut env = env.clone();
                env.insert(name.to_string(), 0);
                (a.pieces[x.piece].offset as usize, enc::eval(&x.expr, &env).unwrap_or(0))
            }
            None => (0, 0),
        }
    }

    /// Decodes the start of `bytes` as this instruction, tied operands
    /// included, and gives its length.
    pub fn decode(&self, bytes: &[u8]) -> Option<(Env, usize)> {
        self.decode_with(bytes, None)
    }

    /// Decodes `bytes` as alternative `alt` of the encoding: what it
    /// encodes, where another alternative may read the same bytes
    /// otherwise (a commutative operation's sources swapped).
    pub fn decode_alt(&self, bytes: &[u8], alt: usize) -> Option<(Env, usize)> {
        self.decode_with(bytes, Some(alt))
    }

    fn decode_with(&self, bytes: &[u8], alt: Option<usize>) -> Option<(Env, usize)> {
        let free = self.free();
        let names: Vec<String> = self.operands.iter().filter(|o| !o.tied).map(|o| o.name.clone()).collect();
        let (mut env, len) = match alt {
            Some(k) => {
                let a = &self.enc.alts[k];
                (a.decode(bytes, &names, &free)?, a.len as usize)
            }
            None => self.enc.decode(bytes, &names, &free)?,
        };
        let d = env.get("d").copied();
        for o in self.operands.iter().filter(|o| o.tied) {
            env.insert(o.name.clone(), d?);
        }
        Some((env, len))
    }

    /// Whether these operand values can be encoded: each immediate must
    /// survive an encode and decode. `None` values are registers, labels or
    /// symbols, and are given placeholders.
    pub fn fits(&self, values: &[Option<i64>]) -> bool {
        let mut env = Env::new();
        for (o, v) in self.operands.iter().zip(values) {
            env.insert(o.name.clone(), v.unwrap_or(0));
        }
        let Some(bytes) = self.encode(&env) else { return false };
        let Some((back, _)) = self.decode(&bytes) else { return false };
        self.operands.iter().zip(values).all(|(o, v)| match (o.kind, v) {
            (Kind::Imm, Some(v)) => {
                let m = if o.bits >= 64 { u64::MAX } else { (1u64 << o.bits) - 1 };
                (back[&o.name] as u64) & m == (*v as u64) & m
            }
            _ => true,
        })
    }
}

#[derive(Clone, Debug)]
pub struct CondForm {
    pub name: String,
    pub value: i64,
    pub asm: String,
    pub reads: String,
    pub func: GlobalId,
    /// The type of the flags it reads: integer or floating-point
    /// comparisons set them differently.
    pub flags_ty: TypeId,
}

/// The operation at a pattern node or IR instruction, with the bit widths
/// that matter for it.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Head {
    Bin(BinOp, u32),
    ICmp(ICmpPred, u32),
    Select(u32),
    Cast(CastOp, u32, u32),
    Load(u32),
    Store(u32),
    /// `getelementptr i8, ptr, i64`.
    Gep,
    CondBr,
    Br,
    Ret,
    Unreachable,
    /// A load with acquire (or stronger) ordering.
    LoadAcq(u32),
    /// A store with release (or stronger) ordering.
    StoreRel(u32),
    /// A fence: `true` if it orders only loads (acquire).
    Fence(bool),
    /// An atomic read-modify-write of this width, its ordering as
    /// `order_rank` gives it.
    Rmw(RmwOp, u32, u8),
    /// A call to an intrinsic, by its index in `Target::intrinsics`.
    Intrinsic(u16, u32),
    FCmp(FCmpPred, u32),
    FNeg(u32),
    InsertElement(u32),
    ExtractElement(u32),
    /// A shuffle by the mask `Target::masks[k]`, giving the type of the
    /// width given.
    Shuffle(u16, u32),
}

impl Head {
    pub fn commutative(self) -> bool {
        matches!(
            self,
            Head::Bin(BinOp::Add | BinOp::Mul | BinOp::And | BinOp::Or | BinOp::Xor, _)
                | Head::ICmp(ICmpPred::Eq | ICmpPred::Ne, _)
        )
    }
}

/// Width class of a binary operation: operations whose low bits depend only
/// on their operands' low bits work on any value narrower than 32 bits in
/// a W register, whatever is above it.
pub fn bin_bits(op: BinOp, bits: u32) -> u32 {
    match op {
        BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::And | BinOp::Or | BinOp::Xor | BinOp::Shl if bits <= 32 => 32,
        _ => bits,
    }
}

pub fn wide_bits(bits: u32) -> u32 {
    if bits <= 32 { 32 } else { bits }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Tree {
    Op(Head, Vec<Tree>),
    /// A pattern parameter.
    Leaf(usize),
    /// A constant, sign-extended, and its width.
    Lit(i64, u32),
    /// The block reached by falling through.
    Next,
    // Only while building:
    Flags(Box<Tree>, Box<Tree>),
    Poison,
}

impl Tree {
    fn nodes(&self) -> usize {
        match self {
            Tree::Op(_, kids) => 1 + kids.iter().map(Tree::nodes).sum::<usize>(),
            _ => 0,
        }
    }

    fn substitute(&self, p: usize, with: &Tree) -> Tree {
        match self {
            Tree::Leaf(q) if *q == p => with.clone(),
            Tree::Op(h, kids) => mk(*h, kids.iter().map(|k| k.substitute(p, with)).collect()),
            t => t.clone(),
        }
    }

    fn leaves(&self, out: &mut Vec<usize>) {
        match self {
            Tree::Leaf(p) => out.push(*p),
            Tree::Op(_, kids) => kids.iter().for_each(|k| k.leaves(out)),
            _ => {}
        }
    }
}

/// Builds a node, simplifying as it goes.
fn mk(h: Head, kids: Vec<Tree>) -> Tree {
    match (h, kids.as_slice()) {
        // A comparison of two values sign-extended from the same width is
        // a comparison of the values.
        (
            Head::ICmp(p, _),
            [Tree::Op(Head::Cast(CastOp::SExt, a, _), x), Tree::Op(Head::Cast(CastOp::SExt, b, _), y)],
        ) if a == b => mk(Head::ICmp(p, *a), vec![x[0].clone(), y[0].clone()]),
        (Head::ICmp(p, _), [Tree::Op(Head::Cast(CastOp::SExt, a, _), x), Tree::Lit(k, _)]) if fits_signed(*k, *a) => {
            mk(Head::ICmp(p, *a), vec![x[0].clone(), Tree::Lit(*k, *a)])
        }
        (Head::Cast(CastOp::SExt, _, to), [Tree::Lit(k, _)]) => Tree::Lit(*k, to),
        // Floating-point comparisons of two values widened from the same
        // width are comparisons of the values: widening is exact.
        (
            Head::FCmp(p, _),
            [Tree::Op(Head::Cast(CastOp::FPExt, a, _), x), Tree::Op(Head::Cast(CastOp::FPExt, b, _), y)],
        ) if a == b => mk(Head::FCmp(p, *a & !FP), vec![x[0].clone(), y[0].clone()]),
        // ... and with +0.0, which is +0.0 at any width.
        (Head::FCmp(p, _), [Tree::Op(Head::Cast(CastOp::FPExt, a, _), x), Tree::Lit(0, _)]) => {
            mk(Head::FCmp(p, *a & !FP), vec![x[0].clone(), Tree::Lit(0, *a)])
        }
        // Identities left by putting 0 in for an operand.
        (Head::Gep, [p, Tree::Lit(0, _)]) => p.clone(),
        (Head::Bin(BinOp::Add | BinOp::Shl | BinOp::Or | BinOp::Xor, _), [x, Tree::Lit(0, _)]) => x.clone(),
        _ => Tree::Op(h, kids),
    }
}

fn fits_signed(k: i64, bits: u32) -> bool {
    bits >= 64 || (-(1i64 << (bits - 1))..(1i64 << (bits - 1))).contains(&k)
}

pub fn sext(v: u64, bits: u32) -> i64 {
    if bits >= 64 { v as i64 } else { ((v << (64 - bits)) as i64) >> (64 - bits) }
}

/// Where an emitted instruction's operand comes from.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Src {
    /// The pattern's result.
    Result,
    Param(usize),
    /// The result of an earlier emitted instruction (by index).
    Temp(usize),
    Imm(i64),
}

#[derive(Clone, Debug)]
pub struct Emit {
    pub instr: usize,
    /// One per operand of the instruction.
    pub args: Vec<Src>,
}

#[derive(Clone, Debug)]
pub struct PParam {
    pub kind: Kind,
    pub bits: u32,
}

#[derive(Clone, Debug)]
pub struct Pattern {
    pub name: String,
    pub tree: Tree,
    pub params: Vec<PParam>,
    pub emit: Vec<Emit>,
    pub nodes: usize,
    /// The wildcards in preorder: a parameter, or `None` for the
    /// fall-through block.
    pub wild: Vec<Option<usize>>,
    /// Further results, for instructions that write back their base: a
    /// tree another IR instruction in the block must match, with the
    /// parameters shared, and the operand of `emit[0]` that receives it.
    pub extra: Vec<(Tree, usize)>,
}

pub struct Target {
    pub module: Module,
    /// Registers, relocations and parameters (see `machine.rs`).
    pub machine: crate::machine::Machine,
    pub instrs: Vec<Instr>,
    pub by_name: HashMap<String, usize>,
    pub conds: Vec<CondForm>,
    pub patterns: Vec<Pattern>,
    /// Instructions whose body is `ret %imm`: ways to make a constant.
    pub constants: Vec<usize>,
    pub copies: HashMap<Class, usize>,
    pub trie: Trie,
    /// Intrinsics the description's bodies call, such as `llvm.smax.i32`.
    pub intrinsics: Vec<String>,
    /// Calling conventions and platforms.
    pub abis: Vec<crate::abi::Abi>,
    fits_cache: std::sync::Mutex<HashMap<FitsKey, bool>>,
    /// Shuffle masks the description's patterns use.
    pub masks: std::sync::Mutex<Vec<Vec<i32>>>,
    decode_index: std::sync::OnceLock<Vec<DecodeBucket>>,
}

/// The disassembler's candidates for code starting with one byte: by the
/// byte at an offset (the first after it with all bits literal), and those
/// with no such byte.
#[derive(Default)]
struct DecodeBucket {
    keyed: HashMap<(u8, u8), Vec<Candidate>>,
    offsets: Vec<u8>,
    rest: Vec<Candidate>,
}

/// An alternative to try: its place in the order tried, the entry, which
/// alternative, and its literal bits.
type Candidate = (usize, usize, usize, LitHead);

/// An alternative's literal bits over its first bytes (up to 8).
#[derive(Clone, Copy, Default)]
struct LitHead {
    mask: u64,
    value: u64,
    len: u8,
}

fn attr<'a>(attrs: &'a [Attr], key: &str) -> Option<Option<&'a str>> {
    attrs.iter().find_map(|a| match a {
        Attr::Str { key: k, value } if k == key.as_bytes() => {
            Some(value.as_deref().map(|v| std::str::from_utf8(v).unwrap_or("")))
        }
        _ => None,
    })
}

/// The directories of `EPITOME_TARGET_PATH`.
fn target_dirs() -> Vec<String> {
    std::env::var("EPITOME_TARGET_PATH")
        .map(|p| p.split(':').filter(|d| !d.is_empty()).map(str::to_string).collect())
        .unwrap_or_default()
}

/// The directory of target `name` in `EPITOME_TARGET_PATH`.
fn search_path(name: &str) -> Option<String> {
    target_dirs().into_iter().map(|d| format!("{d}/{name}")).find(|p| std::path::Path::new(p).is_dir())
}

/// A description's files: one file, or the `.ll` files of a directory in
/// name order.
fn read_path(path: &str) -> Result<Vec<(String, String)>, String> {
    let p = std::path::Path::new(path);
    let read = |f: &std::path::Path| std::fs::read_to_string(f).map_err(|e| format!("{}: {e}", f.display()));
    if !p.is_dir() {
        return Ok(vec![(p.file_name().map_or(path.to_string(), |n| n.to_string_lossy().into_owned()), read(p)?)]);
    }
    let mut names: Vec<std::path::PathBuf> = std::fs::read_dir(p)
        .map_err(|e| format!("{path}: {e}"))?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|f| f.extension().is_some_and(|x| x == "ll"))
        .collect();
    names.sort();
    names.iter().map(|f| Ok((f.file_name().unwrap().to_string_lossy().into_owned(), read(f)?))).collect()
}

/// The built-in AArch64 description (`codegen/targets/aarch64/`), its files
/// in order. Together they are one module.
pub const AARCH64_FILES: &[(&str, &str)] = &[
    ("00-base.ll", include_str!("../targets/aarch64/00-base.ll")),
    ("01-target.ll", include_str!("../targets/aarch64/01-target.ll")),
    ("10-integer.ll", include_str!("../targets/aarch64/10-integer.ll")),
    ("15-extend.ll", include_str!("../targets/aarch64/15-extend.ll")),
    ("20-memory.ll", include_str!("../targets/aarch64/20-memory.ll")),
    ("22-atomics.ll", include_str!("../targets/aarch64/22-atomics.ll")),
    ("25-pairs.ll", include_str!("../targets/aarch64/25-pairs.ll")),
    ("30-bits.ll", include_str!("../targets/aarch64/30-bits.ll")),
    ("40-float.ll", include_str!("../targets/aarch64/40-float.ll")),
    ("45-crypto.ll", include_str!("../targets/aarch64/45-crypto.ll")),
    ("45-sme.ll", include_str!("../targets/aarch64/45-sme.ll")),
    ("50-vectors.ll", include_str!("../targets/aarch64/50-vectors.ll")),
    ("60-recipes.ll", include_str!("../targets/aarch64/60-recipes.ll")),
    ("65-runtime.ll", include_str!("../targets/aarch64/65-runtime.ll")),
    ("70-abi.ll", include_str!("../targets/aarch64/70-abi.ll")),
    ("80-asm.ll", include_str!("../targets/aarch64/80-asm.ll")),
    ("85-sysregs.ll", include_str!("../targets/aarch64/85-sysregs.ll")),
    ("86-system.ll", include_str!("../targets/aarch64/86-system.ll")),
];

/// The x86-64 description (see docs/design/x86.md): for assembly and
/// disassembly so far, its instructions generated from Intel XED's tables.
pub const X86_64_FILES: &[(&str, &str)] = &[
    ("00-base.ll", include_str!("../targets/x86_64/00-base.ll")),
    ("02-xed.ll", include_str!("../targets/x86_64/02-xed.ll")),
    ("05-cond.ll", include_str!("../targets/x86_64/05-cond.ll")),
    ("10-integer.ll", include_str!("../targets/x86_64/10-integer.ll")),
    ("15-frame.ll", include_str!("../targets/x86_64/15-frame.ll")),
    ("16-short.ll", include_str!("../targets/x86_64/16-short.ll")),
    ("70-abi.ll", include_str!("../targets/x86_64/70-abi.ll")),
];

pub fn type_bits(m: &Module, ty: TypeId) -> u32 {
    match m.types.get(ty) {
        Type::Int(b) => *b,
        Type::Ptr(_) => 64,
        Type::Double => 64,
        Type::Float => 32,
        // A vector of booleans is a mask, a whole q register of lanes that
        // are all ones or all zeros, as a vector comparison makes it.
        Type::Vector { scalable: false, elem, .. } if type_bits(m, *elem) == 1 => 128,
        Type::Vector { scalable: false, len, elem } => len * type_bits(m, *elem),
        _ => 0,
    }
}

/// Values of this type live in the floating-point and vector registers.
pub fn is_fp_type(m: &Module, ty: TypeId) -> bool {
    matches!(m.types.get(ty), Type::Double | Type::Float | Type::Vector { .. })
}

pub fn is_vector_type(m: &Module, ty: TypeId) -> bool {
    matches!(m.types.get(ty), Type::Vector { .. })
}

/// Set in the widths of pattern heads for vectors, with the lane count in
/// bits 16-23 and the element's width below: `<4 x i32>` and `<8 x i16>`
/// are told apart.
pub const VEC: u32 = 0x2000;

/// A literal standing for `poison` or `undef` in a vector pattern, such as
/// the unused operand of a splat's `shufflevector`.
pub const UNDEF: i64 = 0x7eee_eeee_eeee_eeee;

/// Set in the widths of pattern heads for floating-point values, so that a
/// `double` load and an `i64` load are told apart.
pub const FP: u32 = 0x1000;

/// The width a pattern head records for a value of type `ty`.
pub fn head_bits(m: &Module, ty: TypeId) -> u32 {
    match m.types.get(ty) {
        Type::Vector { len, elem, .. } => VEC | (len << 16) | head_bits(m, *elem),
        Type::Double | Type::Float => type_bits(m, ty) | FP,
        _ => type_bits(m, ty),
    }
}

/// A constant as a pattern literal: its value (sign-extended, or the raw
/// bits of a floating-point value) and the width of its pattern head.
/// Vectors whose lanes are all the same are that lane; `poison` and
/// `undef` vectors are `UNDEF`. Shared by descriptions and the selector so
/// the two always agree.
pub fn lit_of(m: &Module, c: ConstId) -> Option<(i64, u32)> {
    let k = m.constant(c);
    let hb = head_bits(m, k.ty);
    let bits = type_bits(m, k.ty);
    let lane = |e: ConstId| -> Option<i64> {
        let ek = m.constant(e);
        let eb = type_bits(m, ek.ty);
        match &ek.kind {
            ConstKind::Int(a) => Some(sext(a.low_u64(), eb)),
            ConstKind::Float(raw) => Some(*raw as i64),
            ConstKind::Zero | ConstKind::Null => Some(0),
            _ => None,
        }
    };
    match &k.kind {
        ConstKind::Int(a) => Some((sext(a.low_u64(), bits), hb)),
        ConstKind::Float(raw) => Some((*raw as i64, hb)),
        ConstKind::Zero | ConstKind::Null => Some((0, hb)),
        ConstKind::Poison | ConstKind::Undef if is_vector_type(m, k.ty) => Some((UNDEF, hb)),
        ConstKind::Splat(e) => Some((lane(*e)?, hb)),
        ConstKind::Vector(es) => {
            let first = lane(*es.first()?)?;
            es.iter().all(|&e| lane(e) == Some(first)).then_some((first, hb))
        }
        _ => None,
    }
}

/// The lanes of a `shufflevector` mask: -1 for `poison` lanes.
pub fn mask_lanes(m: &Module, c: ConstId) -> Option<Vec<i32>> {
    let k = m.constant(c);
    let Type::Vector { len, .. } = m.types.get(k.ty) else { return None };
    match &k.kind {
        ConstKind::Zero => Some(vec![0; *len as usize]),
        ConstKind::Poison | ConstKind::Undef => Some(vec![-1; *len as usize]),
        ConstKind::Splat(e) => {
            let v = match &m.constant(*e).kind {
                ConstKind::Int(a) => a.low_u64() as i32,
                _ => -1,
            };
            Some(vec![v; *len as usize])
        }
        ConstKind::Vector(es) => Some(
            es.iter()
                .map(|e| match &m.constant(*e).kind {
                    ConstKind::Int(a) => a.low_u64() as i32,
                    _ => -1,
                })
                .collect(),
        ),
        _ => None,
    }
}

impl Target {
    pub fn aarch64() -> Target {
        Target::load_files(AARCH64_FILES).expect("the built-in AArch64 description is valid")
    }

    pub fn x86_64() -> Target {
        Target::load_files(X86_64_FILES).expect("the built-in x86-64 description is valid")
    }

    /// A description in several files, read in order as one module; a
    /// parse error is reported by the file and line it is in.
    pub fn load_files(files: &[(&str, &str)]) -> Result<Target, String> {
        let text: String = files.iter().map(|(_, t)| *t).collect();
        match epitome_ir::parse_module(&text) {
            Ok(m) => Target::from_module(m),
            Err(e) => {
                let mut line = e.line;
                for (name, t) in files {
                    let n = t.lines().count();
                    if line <= n {
                        return Err(format!("{name}:{line}:{}: {}", e.column, e.message));
                    }
                    line -= n;
                }
                Err(e.to_string())
            }
        }
    }

    /// A target: by `spec`, a path (a `.ll` file, or a directory of them) or
    /// a name (`aarch64` is built in; others are directories of that name
    /// in `EPITOME_TARGET_PATH`, a list separated by colons); or with no
    /// `spec`, the one a module's `triple` names by its architecture
    /// (`riscv64-unknown-linux-gnu` is `riscv64`), else the built-in one.
    /// The files of `adds` (files, or directories of them) are read after
    /// the target's own, as more of the same description.
    pub fn resolve(spec: Option<&str>, triple: &str, adds: &[String]) -> Result<Target, String> {
        let mut files: Vec<(String, String)> = Vec::new();
        let built_in = |files: &mut Vec<(String, String)>| {
            files.extend(AARCH64_FILES.iter().map(|(n, t)| (n.to_string(), t.to_string())));
        };
        let x86 = |files: &mut Vec<(String, String)>| {
            files.extend(X86_64_FILES.iter().map(|(n, t)| (n.to_string(), t.to_string())));
        };
        let arch = triple.split('-').next().unwrap_or("");
        match spec {
            Some(p) if std::path::Path::new(p).exists() => files.extend(read_path(p)?),
            Some("aarch64" | "arm64") => built_in(&mut files),
            Some("x86_64" | "x86-64") => x86(&mut files),
            Some(name) => {
                let dir = search_path(name).ok_or_else(|| {
                    format!("no target {name}: not a path, not built in, and not in EPITOME_TARGET_PATH")
                })?;
                files.extend(read_path(&dir)?);
            }
            None if matches!(arch, "" | "aarch64" | "arm64" | "arm64e") => built_in(&mut files),
            None if arch == "x86_64" => x86(&mut files),
            None => match search_path(arch) {
                Some(dir) => files.extend(read_path(&dir)?),
                None => {
                    return Err(format!(
                        "no target for {arch}: name one with --target, or put it in EPITOME_TARGET_PATH"
                    ));
                }
            },
        }
        for a in adds {
            files.extend(read_path(a)?);
        }
        let refs: Vec<(&str, &str)> = files.iter().map(|(n, t)| (n.as_str(), t.as_str())).collect();
        Target::load_files(&refs)
    }

    /// The target whose ELF machine number is `machine`: the built-in one,
    /// or one in `EPITOME_TARGET_PATH`.
    pub fn for_elf_machine(machine: u16) -> Option<Target> {
        for t in [Target::aarch64(), Target::x86_64()] {
            if t.machine.elf_machine == machine {
                return Some(t);
            }
        }
        for dir in target_dirs() {
            let Ok(entries) = std::fs::read_dir(&dir) else { continue };
            for e in entries.flatten() {
                if e.path().is_dir()
                    && let Ok(t) = Target::load_path(&e.path().to_string_lossy())
                    && t.machine.elf_machine == machine
                {
                    return Some(t);
                }
            }
        }
        None
    }

    /// A description from a file, or from the `.ll` files in a directory,
    /// read in the order of their names.
    pub fn load_path(path: &str) -> Result<Target, String> {
        let files = read_path(path)?;
        let refs: Vec<(&str, &str)> = files.iter().map(|(n, t)| (n.as_str(), t.as_str())).collect();
        Target::load_files(&refs)
    }

    pub fn load(src: &str) -> Result<Target, String> {
        Target::load_files(&[("<description>", src)])
    }

    fn from_module(module: epitome_ir::Module) -> Result<Target, String> {
        let machine = crate::machine::Machine::read(&module)?;
        let mut t = Target {
            module,
            machine,
            instrs: Vec::new(),
            by_name: HashMap::new(),
            conds: Vec::new(),
            patterns: Vec::new(),
            constants: Vec::new(),
            copies: HashMap::new(),
            trie: Trie::default(),
            intrinsics: Vec::new(),
            abis: Vec::new(),
            fits_cache: Default::default(),
            masks: Default::default(),
            decode_index: Default::default(),
        };
        t.read()?;
        t.abis = crate::abi::read(&t.module, "abi.", &t.machine)?;
        t.build_patterns()?;
        let mut trie = Trie::default();
        for (i, p) in t.patterns.iter().enumerate() {
            trie.insert(p, i);
        }
        t.trie = trie;
        Ok(t)
    }

    /// `Instr::fits`, remembered: the same few immediates come up again
    /// and again.
    /// The entries the disassembler tries for `bytes`, and which of their
    /// alternatives, most literal bits first: those with an alternative whose first byte's literal bits are
    /// those of `bytes`, and whose first byte after that with all its bits
    /// literal (the opcode, after VEX's or EVEX's prefix), if it has one, is
    /// there too.
    pub fn decode_candidates(&self, bytes: &[u8]) -> Vec<(usize, usize)> {
        let index = self.decode_index.get_or_init(|| {
            // Every encoding is needed: those not yet parsed are, on every
            // core.
            let threads = std::thread::available_parallelism().map_or(1, |n| n.get());
            let chunk = self.instrs.len().div_ceil(threads).max(1);
            std::thread::scope(|scope| {
                for part in self.instrs.chunks(chunk) {
                    scope.spawn(move || part.iter().for_each(|i| drop(i.enc.parsed())));
                }
            });
            // (Entries not to be read as themselves are here too, for reading
            // an instruction a relocation fills in: `decode` is the caller's.)
            let mut rank: Vec<usize> = (0..self.instrs.len()).filter(|&i| !self.instrs[i].enc.is_empty()).collect();
            rank.sort_by_key(|&i| std::cmp::Reverse(self.instrs[i].enc.fixed_bits()));
            let mut index: Vec<DecodeBucket> = (0..256).map(|_| DecodeBucket::default()).collect();
            for (order, &i) in rank.iter().enumerate() {
                for (alt, a) in self.instrs[i].enc.alts.iter().enumerate() {
                    // Each byte's literal bits, over the first 8.
                    let mut lit = [(0u8, 0u8); 8];
                    for p in &a.pieces {
                        for k in 0..p.bytes.min(8) {
                            let at = (p.offset + k) as usize;
                            if at < 8 {
                                lit[at] = (((p.mask >> (8 * k)) & 0xff) as u8, ((p.value >> (8 * k)) & 0xff) as u8);
                            }
                        }
                    }
                    let key = (1..a.len.min(8) as usize).find(|&k| lit[k].0 == 0xff);
                    // Its literal bits over the first 8 bytes, to check at once.
                    let n = (a.len as usize).min(8);
                    let head = lit[..n].iter().enumerate().fold(LitHead::default(), |h, (k, &(m, v))| LitHead {
                        mask: h.mask | (m as u64) << (8 * k),
                        value: h.value | (v as u64) << (8 * k),
                        len: n as u8,
                    });
                    for (b0, bucket) in index.iter_mut().enumerate() {
                        if (b0 as u8) & lit[0].0 != lit[0].1 {
                            continue;
                        }
                        let list = match key {
                            Some(k) => bucket.keyed.entry((k as u8, lit[k].1)).or_default(),
                            None => &mut bucket.rest,
                        };
                        list.push((order, i, alt, head));
                    }
                }
            }
            for b in &mut index {
                b.offsets = b.keyed.keys().map(|&(k, _)| k).collect();
                b.offsets.sort();
                b.offsets.dedup();
            }
            index
        });
        let Some(&b0) = bytes.first() else { return Vec::new() };
        let bucket = &index[b0 as usize];
        let word = enc::word(&bytes[..bytes.len().min(8)]);
        let fits = |h: &LitHead| bytes.len() >= h.len as usize && word & h.mask == h.value;
        let mut out: Vec<(usize, usize, usize)> =
            bucket.rest.iter().filter(|(_, _, _, h)| fits(h)).map(|&(o, i, a, _)| (o, i, a)).collect();
        for &k in &bucket.offsets {
            if let Some(&b) = bytes.get(k as usize)
                && let Some(list) = bucket.keyed.get(&(k, b))
            {
                out.extend(list.iter().filter(|(_, _, _, h)| fits(h)).map(|&(o, i, a, _)| (o, i, a)));
            }
        }
        out.sort();
        out.dedup();
        out.into_iter().map(|(_, i, a)| (i, a)).collect()
    }

    /// The shortest instruction's length in bytes: 4 for a target whose
    /// instructions are all one word.
    pub fn unit(&self) -> usize {
        self.machine.unit as usize
    }

    pub fn fits_cached(&self, i: usize, vals: Vec<Option<i64>>) -> bool {
        let key = (i, vals);
        if let Some(&ok) = self.fits_cache.lock().unwrap().get(&key) {
            return ok;
        }
        let ok = self.instrs[i].fits(&key.1);
        self.fits_cache.lock().unwrap().insert(key, ok);
        ok
    }

    /// The condition that holds when condition `value` does not: the form
    /// whose comparison is the inverse of this one's.
    pub fn inverse_cond(&self, value: i64) -> Option<i64> {
        #[derive(PartialEq)]
        enum P {
            I(ICmpPred),
            F(FCmpPred),
        }
        let pred_of = |form: &CondForm| -> Option<P> {
            let f = self.func(form.func);
            f.insts.iter().find_map(|i| match i.op {
                Op::ICmp { pred, .. } => Some(P::I(pred)),
                Op::FCmp { pred, .. } => Some(P::F(pred)),
                _ => None,
            })
        };
        let form = self.conds.iter().find(|c| c.value == value)?;
        let want = match pred_of(form)? {
            P::I(p) => P::I(inverse(p)),
            P::F(p) => P::F(inverse_f(p)?),
        };
        self.conds.iter().find(|c| pred_of(c).as_ref() == Some(&want)).map(|c| c.value)
    }

    pub fn abi(&self, name: &str) -> Option<&crate::abi::Abi> {
        self.abis.iter().find(|a| a.name == name)
    }

    /// The convention for a module's target triple: the first whose
    /// substrings it contains.
    pub fn abi_for_triple(&self, triple: &str) -> Option<&crate::abi::Abi> {
        self.abis.iter().find(|a| a.triples.iter().any(|s| triple.contains(s.as_str())))
    }

    /// The number of a shuffle mask, adding it if new.
    pub fn mask_id(&self, lanes: &[i32]) -> u16 {
        let mut ms = self.masks.lock().unwrap();
        match ms.iter().position(|m| m == lanes) {
            Some(i) => i as u16,
            None => {
                ms.push(lanes.to_vec());
                (ms.len() - 1) as u16
            }
        }
    }

    /// Is there a pattern whose root is a shuffle by this mask, to a
    /// result of head bits `hb` (lanes and their type), of a first operand
    /// of `src_bits` bits, or (if `of_insert`) one an `insertelement` makes
    /// (a splat of a scalar)?
    /// (`hb` `None`, a mask's: masks take their comparison's lane width
    /// later, so any pattern of the mask will do.)
    pub fn has_shuffle(&self, lanes: &[i32], hb: Option<u32>, src_bits: u32, of_insert: bool) -> bool {
        let Some(id) = self.find_mask(lanes) else { return false };
        if hb.is_none() {
            return self.patterns.iter().any(|p| matches!(&p.tree, Tree::Op(Head::Shuffle(k, _), _) if *k == id));
        }
        self.patterns.iter().any(|p| match &p.tree {
            Tree::Op(Head::Shuffle(k, b), kids) if *k == id && Some(*b) == hb => match kids.first() {
                Some(Tree::Op(Head::InsertElement(_), _)) => of_insert,
                Some(Tree::Leaf(q)) => p.params.get(*q).is_some_and(|x| x.bits == src_bits),
                _ => true,
            },
            _ => false,
        })
    }

    pub fn find_mask(&self, lanes: &[i32]) -> Option<u16> {
        self.masks.lock().unwrap().iter().position(|m| m == lanes).map(|i| i as u16)
    }

    pub fn instr(&self, name: &str) -> &Instr {
        &self.instrs[self.by_name[name]]
    }

    pub fn func(&self, g: GlobalId) -> &Function {
        match &self.module.global(g).kind {
            GlobalKind::Function(f) => f,
            _ => unreachable!(),
        }
    }

    pub fn by_role(&self, role: &str) -> usize {
        self.role(role).unwrap_or_else(|| panic!("no instruction has role {role}"))
    }

    /// The instruction with this role, if one has it. (`"role"` may list
    /// several, with commas.)
    pub fn role(&self, role: &str) -> Option<usize> {
        self.instrs.iter().position(|i| i.role.as_deref().is_some_and(|r| r.split(',').any(|x| x == role)))
    }

    /// Does instruction `i` have this role?
    pub fn has_role(&self, i: usize, role: &str) -> bool {
        self.instrs[i].role.as_deref().is_some_and(|r| r.split(',').any(|x| x == role))
    }

    fn read(&mut self) -> Result<(), String> {
        let m = &self.module;
        for (g, f) in m.functions() {
            if let Some(name) = &m.global(g).name
                && name.starts_with("llvm.")
                && !f.has_body
            {
                self.intrinsics.push(name.clone());
            }
        }
        let mut instrs = Vec::new();
        let mut conds = Vec::new();
        // Named pieces of encodings, `!{!"name", !"params", !"encoding"}`.
        let mut fragments = enc::Fragments::new();
        for t in crate::machine::tuples(m, "target.encodings") {
            let [name, params, body] = &t[..] else { return Err(format!("bad encoding fragment {t:?}")) };
            let params = params.split([',', ' ']).filter(|p| !p.is_empty()).map(String::from).collect();
            fragments.insert(name.clone(), (params, body.clone()));
        }
        let fragments = std::sync::Arc::new(fragments);
        for (g, f) in m.functions() {
            let name = m.global(g).name.clone().unwrap_or_default();
            if let Some(Some(v)) = attr(&f.fn_attrs, "cond") {
                conds.push(CondForm {
                    name: name.clone(),
                    value: v.parse().map_err(|_| format!("@{name}: bad cond value"))?,
                    asm: attr(&f.fn_attrs, "asm").flatten().unwrap_or("").to_string(),
                    reads: match attr(&f.fn_attrs, "reads").flatten() {
                        Some(r) => r.to_string(),
                        None => self
                            .machine
                            .flags
                            .clone()
                            .ok_or_else(|| format!("@{name}: no \"reads\", and no target parameter flags"))?,
                    },
                    func: g,
                    flags_ty: f.locals[f.params[0].local.index()].ty,
                });
                continue;
            }
            let Some(Some(enc_text)) = attr(&f.fn_attrs, "enc") else { continue };
            let enc = LazyEncoding::new(&name, enc_text, fragments.clone());
            if !self.machine.lazy_encodings {
                enc.parsed()?;
            }
            let fn_ty = m.types.get(f.ty).clone();
            let Type::Function { ret, .. } = fn_ty else { unreachable!() };
            // A struct result is the flags, for a comparison ("sets"), or
            // the value and the written-back base of a post-increment load.
            let aggregate = matches!(m.types.get(ret), Type::Named(_) | Type::Struct { .. });
            let flags = aggregate && attr(&f.fn_attrs, "sets").is_some();
            let res_ty = if aggregate && !flags {
                let fields = match m.types.get(ret) {
                    Type::Struct { fields, .. } => fields.clone(),
                    Type::Named(n) => {
                        m.types.structs[*n as usize].body.as_ref().map(|b| b.fields.clone()).unwrap_or_default()
                    }
                    _ => vec![],
                };
                fields.first().copied()
            } else {
                Some(ret)
            };
            let ret_bits = res_ty.map_or(0, |t| type_bits(m, t));
            let res_fp = res_ty.is_some_and(|t| is_fp_type(m, t));
            let writeback_ret = f.params.iter().any(|p| attr(&p.attrs, "writeback").is_some()) && !aggregate;
            let has_result = ret_bits > 0 && !flags && !writeback_ret;
            let mut operands = Vec::new();
            if has_result {
                let class = match attr(&f.fn_attrs, "dclass").flatten() {
                    Some(c) => self.machine.class(c).ok_or_else(|| format!("@{name}: no register class {c}"))?,
                    None => {
                        let holds = if res_ty.is_some_and(|t| is_vector_type(m, t)) {
                            Holds::Vector
                        } else if res_fp {
                            Holds::Float
                        } else {
                            Holds::Int
                        };
                        self.machine
                            .value_class(holds, ret_bits)
                            .ok_or_else(|| format!("@{name}: no register class holds its result"))?
                    }
                };
                operands.push(Operand {
                    name: "d".into(),
                    kind: Kind::Reg(class),
                    bits: ret_bits,
                    tied: false,
                    writeback: false,
                });
            }
            // A second register result, `d2`: the second field of the
            // result (a load pair's second register).
            let second_result = attr(&f.fn_attrs, "result2").is_some();
            if second_result {
                let field = match m.types.get(ret) {
                    Type::Struct { fields, .. } => fields.get(1).copied(),
                    _ => None,
                }
                .ok_or_else(|| format!("@{name}: \"result2\" without a second result"))?;
                let bits = type_bits(m, field);
                let class = match operands.first().map(|o| o.kind) {
                    Some(Kind::Reg(c)) => c,
                    _ => return Err(format!("@{name}: \"result2\" without a first result")),
                };
                operands.push(Operand {
                    name: "d2".into(),
                    kind: Kind::Reg(class),
                    bits,
                    tied: false,
                    writeback: false,
                });
            }
            for p in &f.params {
                let local = &f.locals[p.local.index()];
                let pname = local.name.clone().unwrap_or_default();
                let kind = if let Some(Some(c)) = attr(&p.attrs, "reg") {
                    Kind::Reg(self.machine.class(c).ok_or_else(|| format!("@{name}: no register class {c}"))?)
                } else if attr(&p.attrs, "imm").is_some() {
                    Kind::Imm
                } else if attr(&p.attrs, "label").is_some() {
                    Kind::Label
                } else if attr(&p.attrs, "sym").is_some() {
                    Kind::Sym
                } else if attr(&p.attrs, "cond").is_some() {
                    Kind::Cond
                } else if let Some(Some(e)) = attr(&p.attrs, "enum") {
                    Kind::Enum(self.machine.enum_index(e).ok_or_else(|| format!("@{name}: no enum {e}"))? as u16)
                } else {
                    return Err(format!("@{name}: operand %{pname} has no kind"));
                };
                operands.push(Operand {
                    name: pname,
                    kind,
                    bits: type_bits(m, local.ty),
                    tied: attr(&p.attrs, "tied").is_some(),
                    writeback: attr(&p.attrs, "writeback").is_some(),
                });
            }
            // A 32-bit result is zero above 32 bits where the machine zeroes
            // the top half of the register; where it sign-extends, nothing
            // is known.
            let default_zext = if has_result && ret_bits <= 32 && self.machine.narrow == crate::machine::Narrow::Zero {
                32
            } else {
                64
            };
            let reloc = match attr(&f.fn_attrs, "reloc").flatten() {
                Some(r) => Some(crate::object::RelocKind(
                    self.machine.reloc(r).ok_or_else(|| format!("@{name}: no relocation {r} in the description"))?
                        as u16,
                )),
                None => None,
            };
            instrs.push(Instr {
                name: name.clone(),
                operands,
                has_result,
                enc,
                asm: attr(&f.fn_attrs, "asm").flatten().unwrap_or("").to_string(),
                decode: attr(&f.fn_attrs, "nodecode").is_none(),
                zext: attr(&f.fn_attrs, "zext").flatten().and_then(|z| z.parse().ok()).unwrap_or(default_zext),
                copy: match attr(&f.fn_attrs, "copy").flatten() {
                    Some(c) => Some(self.machine.class(c).ok_or_else(|| format!("@{name}: no register class {c}"))?),
                    None => None,
                },
                role: attr(&f.fn_attrs, "role").flatten().map(str::to_string),
                sets: if flags {
                    Some(match attr(&f.fn_attrs, "sets").flatten() {
                        Some(s) => s.to_string(),
                        None => self
                            .machine
                            .flags
                            .clone()
                            .ok_or_else(|| format!("@{name}: no \"sets\", and no target parameter flags"))?,
                    })
                } else {
                    None
                },
                matchable: attr(&f.fn_attrs, "nomatch").is_none() && attr(&f.fn_attrs, "asmonly").is_none(),
                asm_only: attr(&f.fn_attrs, "asmonly").is_some(),
                format: match attr(&f.fn_attrs, "format").flatten() {
                    None => None,
                    Some("macho") => Some(crate::abi::Syntax::MachO),
                    Some("elf") => Some(crate::abi::Syntax::Elf),
                    Some(o) => return Err(format!("@{name}: \"format\"=\"{o}\": not elf or macho")),
                },
                aliases: attr(&f.fn_attrs, "aliases")
                    .flatten()
                    .map_or(vec![], |a| a.split(',').map(str::to_string).collect()),
                inverse: attr(&f.fn_attrs, "inverse").flatten().map(str::to_string),
                reloc,
                second_result,
                features: attr(&f.fn_attrs, "feature")
                    .flatten()
                    .map(|s| s.split(',').map(|x| x.trim().to_string()).filter(|x| !x.is_empty()).collect())
                    .unwrap_or_default(),
                base_offset: base_offset_of(m, f, has_result),
                address_of: has_result && base_offset_of(m, f, has_result).is_some() && returns_first(f),
                memory: f.blocks.iter().flat_map(|b| &b.insts).any(|i| match &f.insts[i.index()].op {
                    Op::Load { .. }
                    | Op::Store { .. }
                    | Op::Fence { .. }
                    | Op::AtomicRmw { .. }
                    | Op::CmpXchg { .. } => true,
                    // A call of a function, or of an intrinsic that may
                    // touch memory or order it (exclusive loads, barriers,
                    // hints, traps), not one that only computes.
                    Op::Call(c) => crate::prepare::callee_name(m, c).is_none_or(|n| {
                        !n.starts_with("llvm.")
                            || [
                                "ld", "st", "dmb", "dsb", "isb", "clrex", "hint", "prefetch", "barrier", "break",
                                "hlt", "trap",
                            ]
                            .iter()
                            .any(|k| n.contains(k))
                    }),
                    _ => false,
                }),
                clobbers: {
                    let list: Vec<String> = attr(&f.fn_attrs, "clobbers")
                        .flatten()
                        .map_or(vec![], |c| c.split(',').map(|x| x.trim().to_string()).collect());
                    let regs: Vec<String> = list.iter().filter(|x| *x != "flags").cloned().collect();
                    self.machine.regs(&regs).map_err(|e| format!("@{name}: clobbers: {e}"))?
                },
                clobbers_flags: attr(&f.fn_attrs, "clobbers")
                    .flatten()
                    .is_some_and(|c| c.split(',').any(|x| x.trim() == "flags")),
                func: g,
            });
        }
        for (i, ins) in instrs.iter().enumerate() {
            self.by_name.insert(ins.name.clone(), i);
            if let Some(c) = ins.copy {
                self.copies.insert(c, i);
            }
        }
        self.instrs = instrs;
        self.conds = conds;
        Ok(())
    }

    // -----------------------------------------------------------------------
    // Building trees from bodies

    /// Evaluates the body of `f` symbolically: `env` gives trees for some
    /// parameters, and the others become leaves numbered from `base`.
    fn eval_value(&self, f: &Function, v: Value, env: &HashMap<LocalId, Tree>, base: usize) -> Result<Tree, String> {
        match v {
            Value::Const(c) => {
                let k = self.module.constant(c);
                let bits = type_bits(&self.module, k.ty);
                if let Some((v, hb)) = lit_of(&self.module, c) {
                    return Ok(Tree::Lit(v, hb));
                }
                let _ = bits;
                Ok(match &k.kind {
                    ConstKind::Poison | ConstKind::Undef => Tree::Poison,
                    other => return Err(format!("unsupported constant {other:?} in a body")),
                })
            }
            Value::Local(x) => {
                if let Some(t) = env.get(&x) {
                    return Ok(t.clone());
                }
                match f.locals[x.index()].def {
                    LocalDef::Param(i) => Ok(Tree::Leaf(base + i as usize)),
                    LocalDef::Inst(i) => self.eval_inst(f, i, env, base),
                    _ => Err("a block used as a value".into()),
                }
            }
            Value::Metadata(_) => Err("metadata in a body".into()),
        }
    }

    fn eval_inst(&self, f: &Function, i: InstId, env: &HashMap<LocalId, Tree>, base: usize) -> Result<Tree, String> {
        let m = &self.module;
        let ev = |v: Value| self.eval_value(f, v, env, base);
        let ty_bits = |v: Value| match v {
            Value::Local(x) => type_bits(m, f.locals[x.index()].ty),
            Value::Const(c) => type_bits(m, m.constant(c).ty),
            _ => 0,
        };
        let res_bits = f.insts[i.index()].result.map(|r| type_bits(m, f.locals[r.index()].ty)).unwrap_or(0);
        let res_hbits = f.insts[i.index()].result.map(|r| head_bits(m, f.locals[r.index()].ty)).unwrap_or(0);
        let hb = |v: Value| match v {
            Value::Local(x) => head_bits(m, f.locals[x.index()].ty),
            Value::Const(c) => head_bits(m, m.constant(c).ty),
            _ => 0,
        };
        Ok(match &f.insts[i.index()].op {
            Op::Binary { op, lhs, rhs, .. } => {
                let b = if res_hbits & VEC != 0 { res_hbits } else { bin_bits(*op, res_bits) };
                mk(Head::Bin(*op, b), vec![ev(*lhs)?, ev(*rhs)?])
            }
            Op::ICmp { pred, lhs, rhs, .. } => {
                let b = if hb(*lhs) & VEC != 0 { hb(*lhs) } else { ty_bits(*lhs) };
                mk(Head::ICmp(*pred, b), vec![ev(*lhs)?, ev(*rhs)?])
            }
            Op::InsertElement { vector, element, index } => {
                mk(Head::InsertElement(res_hbits), vec![ev(*vector)?, ev(*element)?, ev(*index)?])
            }
            Op::ExtractElement { vector, index } => {
                mk(Head::ExtractElement(hb(*vector)), vec![ev(*vector)?, ev(*index)?])
            }
            Op::ShuffleVector { a, b, mask } => {
                let lanes = mask_lanes(m, *mask).ok_or("a shuffle mask that is not constant")?;
                let id = self.mask_id(&lanes);
                mk(Head::Shuffle(id, res_hbits), vec![ev(*a)?, ev(*b)?])
            }
            Op::Select { cond, then_value, else_value, .. } => {
                mk(Head::Select(wide_bits(res_hbits)), vec![ev(*cond)?, ev(*then_value)?, ev(*else_value)?])
            }
            Op::Cast { op, value, .. } => mk(Head::Cast(*op, hb(*value), wide_bits(res_hbits)), vec![ev(*value)?]),
            // (A vector's by its lanes and their type, as isel.rs keys it.)
            Op::FCmp { pred, lhs, rhs, .. } => {
                let b = if hb(*lhs) & VEC != 0 { hb(*lhs) } else { ty_bits(*lhs) };
                mk(Head::FCmp(*pred, b), vec![ev(*lhs)?, ev(*rhs)?])
            }
            Op::FNeg { value, .. } => mk(Head::FNeg(res_hbits), vec![ev(*value)?]),
            Op::Load { ptr, atomic, .. } if acquires(atomic) => mk(Head::LoadAcq(res_hbits), vec![ev(*ptr)?]),
            Op::AtomicRmw { op, ptr, value, ordering, .. } => {
                mk(Head::Rmw(*op, res_hbits, order_rank(*ordering)), vec![ev(*ptr)?, ev(*value)?])
            }
            Op::Load { ptr, .. } => mk(Head::Load(res_hbits), vec![ev(*ptr)?]),
            Op::Gep { ptr, indices, source_ty, .. }
                if indices.len() == 1 && matches!(m.types.get(*source_ty), Type::Int(8)) =>
            {
                mk(Head::Gep, vec![ev(*ptr)?, ev(indices[0])?])
            }
            Op::ExtractValue { aggregate, indices } => match (ev(*aggregate)?, indices.as_slice()) {
                (Tree::Flags(a, _), [0]) => *a,
                (Tree::Flags(_, b), [1]) => *b,
                (t, _) => return Err(format!("extractvalue from {t:?}")),
            },
            Op::InsertValue { aggregate, element, indices } => {
                let (a, b) = match ev(*aggregate)? {
                    Tree::Poison => (Tree::Poison, Tree::Poison),
                    Tree::Flags(a, b) => (*a, *b),
                    t => return Err(format!("insertvalue into {t:?}")),
                };
                let e = ev(*element)?;
                match indices.as_slice() {
                    [0] => Tree::Flags(Box::new(e), Box::new(b)),
                    [1] => Tree::Flags(Box::new(a), Box::new(e)),
                    _ => return Err("insertvalue index".into()),
                }
            }
            Op::Call(call) => {
                let Value::Const(c) = call.callee else { return Err("indirect call in a body".into()) };
                let ConstKind::Global(g) = self.module.constant(c).kind else { return Err("call".into()) };
                if let Some(id) =
                    self.module.global(g).name.as_ref().and_then(|n| self.intrinsics.iter().position(|x| x == n))
                {
                    let args = call.args.iter().map(|a| ev(a.value)).collect::<Result<Vec<_>, _>>()?;
                    return Ok(mk(Head::Intrinsic(id as u16, res_bits), args));
                }
                // An instruction, in an expansion: what its body says.
                if let Some(&idx) = self.module.global(g).name.as_ref().and_then(|n| self.by_name.get(n)) {
                    let callee = self.func(self.instrs[idx].func);
                    let mut inner = HashMap::new();
                    for (p, a) in callee.params.iter().zip(&call.args) {
                        inner.insert(p.local, ev(a.value)?);
                    }
                    let ret = self.ret_value(callee).ok_or("an instruction without a value")?;
                    return self.eval_value(callee, ret, &inner, 0);
                }
                // A condition form, applied to flags.
                let form =
                    self.conds.iter().find(|k| k.func == g).ok_or("call to something other than a condition form")?;
                let flags = ev(call.args[0].value)?;
                self.eval_cond(form, flags)?
            }
            op => return Err(format!("unsupported operation in a body: {op:?}")),
        })
    }

    fn eval_cond(&self, form: &CondForm, flags: Tree) -> Result<Tree, String> {
        let f = self.func(form.func);
        let mut env = HashMap::new();
        env.insert(f.params[0].local, flags);
        let ret = self.ret_value(f).ok_or("condition form without a value")?;
        self.eval_value(f, ret, &env, 0)
    }

    fn ret_value(&self, f: &Function) -> Option<Value> {
        let b = f.entry();
        let t = f.terminator(b)?;
        match f.insts[t.index()].op {
            Op::Ret(v) => v,
            _ => None,
        }
    }

    /// The pattern tree of a whole body.
    fn body_tree(&self, f: &Function, env: &HashMap<LocalId, Tree>, base: usize) -> Result<Option<Tree>, String> {
        let entry = f.entry();
        let t = f.terminator(entry).ok_or("body without a terminator")?;
        let label_of = |v: Value| -> Result<Tree, String> {
            match self.eval_value(f, v, env, base)? {
                Tree::Leaf(p) => Ok(Tree::Leaf(p)),
                t => Err(format!("jump to {t:?}")),
            }
        };
        // What one successor block of a conditional branch does.
        let arm = |b: BlockId| -> Result<Tree, String> {
            let insts = &f.blocks[b.index()].insts;
            match insts.as_slice() {
                [i] => match &f.insts[i.index()].op {
                    Op::IndirectBr { addr, .. } => label_of(*addr),
                    Op::Ret(None) => Ok(Tree::Next),
                    _ => Err("a branch arm must be indirectbr or ret void".into()),
                },
                _ => Err("a branch arm must be one instruction".into()),
            }
        };
        Ok(Some(match &f.insts[t.index()].op {
            Op::Ret(Some(v)) => self.eval_value(f, *v, env, base)?,
            Op::Ret(None) => {
                let store = f.blocks[entry.index()]
                    .insts
                    .iter()
                    .rev()
                    .find(|&&i| matches!(f.insts[i.index()].op, Op::Store { .. } | Op::Fence { .. } | Op::Call(_)));
                match store {
                    // An intrinsic without a value (`llvm.aarch64.isb`).
                    Some(&s) if matches!(f.insts[s.index()].op, Op::Call(_)) => {
                        let Op::Call(call) = &f.insts[s.index()].op else { unreachable!() };
                        let id = match call.callee {
                            Value::Const(c) => match self.module.constant(c).kind {
                                ConstKind::Global(g) => self
                                    .module
                                    .global(g)
                                    .name
                                    .as_ref()
                                    .and_then(|n| self.intrinsics.iter().position(|x| x == n)),
                                _ => None,
                            },
                            _ => None,
                        };
                        let id = id.ok_or("a call of something not an intrinsic in a body")?;
                        let args = call
                            .args
                            .iter()
                            .map(|a| self.eval_value(f, a.value, env, base))
                            .collect::<Result<Vec<_>, _>>()?;
                        mk(Head::Intrinsic(id as u16, 0), args)
                    }
                    Some(&s) if matches!(f.insts[s.index()].op, Op::Fence { .. }) => {
                        let Op::Fence { ordering, .. } = f.insts[s.index()].op else { unreachable!() };
                        Tree::Op(Head::Fence(ordering == Ordering::Acquire), vec![])
                    }
                    Some(&s) => {
                        let Op::Store { value, ptr, ref atomic, .. } = f.insts[s.index()].op else { unreachable!() };
                        let release = releases(atomic);
                        let bits = match value {
                            Value::Local(x) => head_bits(&self.module, f.locals[x.index()].ty),
                            Value::Const(c) => head_bits(&self.module, self.module.constant(c).ty),
                            _ => 0,
                        };
                        mk(
                            if release { Head::StoreRel(bits) } else { Head::Store(bits) },
                            vec![self.eval_value(f, value, env, base)?, self.eval_value(f, ptr, env, base)?],
                        )
                    }
                    None if f.blocks[entry.index()].insts.len() == 1 => Tree::Op(Head::Ret, vec![]),
                    None => return Ok(None),
                }
            }
            Op::Unreachable => Tree::Op(Head::Unreachable, vec![]),
            Op::IndirectBr { addr, .. } => Tree::Op(Head::Br, vec![label_of(*addr)?]),
            Op::CondBr { cond, then_dest, else_dest } => {
                Tree::Op(Head::CondBr, vec![self.eval_value(f, *cond, env, base)?, arm(*then_dest)?, arm(*else_dest)?])
            }
            op => return Err(format!("unsupported terminator in a body: {op:?}")),
        }))
    }

    // -----------------------------------------------------------------------
    // Patterns

    fn params_of(&self, ins: &Instr) -> Vec<PParam> {
        ins.operands
            .iter()
            .filter(|o| o.name != "d" || !ins.has_result)
            .map(|o| PParam { kind: o.kind, bits: o.bits })
            .collect()
    }

    /// The emitted instruction for `ins` with its parameters numbered from
    /// `base`, the condition operand given, and its result going to `res`.
    fn emit_of(&self, idx: usize, base: usize, cond: Option<i64>, res: Src) -> Emit {
        let ins = &self.instrs[idx];
        let mut args = Vec::new();
        let mut k = base;
        for o in &ins.operands {
            if o.name == "d" && ins.has_result {
                args.push(res);
            } else {
                args.push(match (o.kind, cond) {
                    (Kind::Cond, Some(v)) => Src::Imm(v),
                    _ => Src::Param(k),
                });
                k += 1;
            }
        }
        Emit { instr: idx, args }
    }

    fn build_patterns(&mut self) -> Result<(), String> {
        let mut out: Vec<Pattern> = Vec::new();
        // (Those not to be matched are only for recipes: x86's fucompi.)
        let producers: Vec<usize> =
            (0..self.instrs.len()).filter(|&i| self.instrs[i].sets.is_some() && self.instrs[i].matchable).collect();
        for idx in 0..self.instrs.len() {
            let ins = &self.instrs[idx];
            if !ins.matchable || ins.copy.is_some() || ins.sets.is_some() {
                continue;
            }
            let f = self.func(ins.func);
            let params = self.params_of(ins);
            let cond_param = params.iter().position(|p| p.kind == Kind::Cond);
            match cond_param {
                None => {
                    let Some(tree) =
                        self.body_tree(f, &HashMap::new(), 0).map_err(|e| format!("@{}: {e}", ins.name))?
                    else {
                        continue;
                    };
                    match tree {
                        Tree::Leaf(p) if params[p].kind == Kind::Imm => {
                            self.constants.push(idx);
                            continue;
                        }
                        Tree::Leaf(_) => continue,
                        _ => {}
                    }
                    let emit = vec![self.emit_of(idx, 0, None, Src::Result)];
                    // A post-increment access: the access is the pattern,
                    // and the stepped base is a second root to find.
                    let mut extra = Vec::new();
                    let mut tree = tree;
                    if let Some(wb) = ins.operands.iter().position(|o| o.writeback) {
                        let entry = f.entry();
                        let store = f.blocks[entry.index()]
                            .insts
                            .iter()
                            .find(|&&i| matches!(f.insts[i.index()].op, Op::Store { .. }));
                        match (store, tree) {
                            (Some(&st), next) => {
                                let Op::Store { value, ptr, .. } = f.insts[st.index()].op else { unreachable!() };
                                let bits = match value {
                                    Value::Local(x) => head_bits(&self.module, f.locals[x.index()].ty),
                                    _ => 0,
                                };
                                let ev = |v| self.eval_value(f, v, &HashMap::new(), 0);
                                tree = mk(Head::Store(bits), vec![ev(value)?, ev(ptr)?]);
                                extra.push((next, wb));
                            }
                            (None, Tree::Flags(value, next)) => {
                                tree = *value;
                                extra.push((*next, wb));
                            }
                            (None, t) => {
                                return Err(format!(
                                    "@{}: a write-back instruction must return its base: {t:?}",
                                    ins.name
                                ));
                            }
                        }
                    }
                    out.push(Pattern { name: ins.name.clone(), tree, params, emit, nodes: 0, wild: vec![], extra });
                }
                Some(cp) => {
                    // Each condition form, on the flags of each comparison
                    // that sets the flags it reads.
                    for form in &self.conds {
                        for &pi in &producers {
                            let prod = &self.instrs[pi];
                            let sets = prod.sets.as_deref().unwrap_or("");
                            if !form.reads.chars().all(|c| sets.contains(c)) {
                                continue;
                            }
                            let Type::Function { ret, .. } = self.module.types.get(self.func(prod.func).ty) else {
                                continue;
                            };
                            if *ret != form.flags_ty {
                                continue;
                            }
                            let base = params.len();
                            let pf = self.func(prod.func);
                            let ret = self.ret_value(pf).ok_or("comparison without a value")?;
                            let flags = self
                                .eval_value(pf, ret, &HashMap::new(), base)
                                .map_err(|e| format!("@{}: {e}", prod.name))?;
                            let cond = self.eval_cond(form, flags).map_err(|e| format!("@{}: {e}", form.name))?;
                            let mut env = HashMap::new();
                            env.insert(f.params[cp].local, cond);
                            let Some(tree) = self.body_tree(f, &env, 0).map_err(|e| format!("@{}: {e}", ins.name))?
                            else {
                                continue;
                            };
                            let mut all = params.clone();
                            all.extend(self.params_of(prod));
                            let emit = vec![
                                self.emit_of(pi, base, None, Src::Result),
                                self.emit_of(idx, 0, Some(form.value), Src::Result),
                            ];
                            out.push(Pattern {
                                name: format!("{}+{}.{}", prod.name, ins.name, form.asm),
                                tree,
                                params: all,
                                emit,
                                nodes: 0,
                                wild: vec![],
                                extra: vec![],
                            });
                        }
                    }
                }
            }
        }
        // Recipes.
        let m = &self.module;
        for (_, f) in m.functions() {
            let Some(Some(expand)) = attr(&f.fn_attrs, "recipe") else { continue };
            let tree = self.body_tree(f, &HashMap::new(), 0)?.ok_or("recipe without a value")?;
            let mut params = Vec::new();
            for p in &f.params {
                let kind = match attr(&p.attrs, "reg") {
                    Some(Some(c)) => Kind::Reg(self.machine.class(c).ok_or_else(|| format!("no register class {c}"))?),
                    _ => Kind::Imm,
                };
                params.push(PParam { kind, bits: type_bits(m, f.locals[p.local.index()].ty) });
            }
            let g = m
                .global_order
                .iter()
                .copied()
                .find(|&g| m.global(g).name.as_deref() == Some(expand))
                .ok_or_else(|| format!("no function @{expand}"))?;
            let emit = self.expansion(self.func(g)).map_err(|e| format!("@{expand}: {e}"))?;
            out.push(Pattern {
                name: expand.trim_end_matches(".expand").to_string(),
                tree,
                params,
                emit,
                nodes: 0,
                wild: vec![],
                extra: vec![],
            });
        }

        // Specialise immediates that can be zero, and give conditional
        // branches their inverted twin.
        let mut more = Vec::new();
        for p in &out {
            for (i, pp) in p.params.iter().enumerate() {
                if pp.kind != Kind::Imm {
                    continue;
                }
                let zero = Tree::Lit(0, pp.bits);
                let t = p.tree.substitute(i, &zero);
                let mut leaves = Vec::new();
                t.leaves(&mut leaves);
                // Only when an identity went: `gep p, 0` is `p`.
                if t.nodes() >= p.tree.nodes() || t.nodes() == 0 || leaves.contains(&i) {
                    continue;
                }
                let mut q = p.clone();
                q.tree = t;
                for e in &mut q.emit {
                    for a in &mut e.args {
                        if *a == Src::Param(i) {
                            *a = Src::Imm(0);
                        }
                    }
                }
                if self.pattern_fits(&q) {
                    q.name = format!("{} ({}=0)", q.name, i);
                    more.push(q);
                }
            }
        }
        out.extend(more);
        let mut more = Vec::new();
        for p in &out {
            if let Tree::Op(Head::CondBr, kids) = &p.tree
                && let Tree::Op(Head::ICmp(pred, bits), cmp) = &kids[0]
            {
                let mut q = p.clone();
                q.tree = Tree::Op(
                    Head::CondBr,
                    vec![Tree::Op(Head::ICmp(inverse(*pred), *bits), cmp.clone()), kids[2].clone(), kids[1].clone()],
                );
                q.name = format!("{} (inverted)", q.name);
                more.push(q);
            }
        }
        out.extend(more);
        for p in &mut out {
            p.nodes = p.tree.nodes() + p.extra.iter().map(|(t, _)| t.nodes()).sum::<usize>();
            let mut wild = Vec::new();
            preorder_wild(&p.tree, &mut wild);
            p.wild = wild;
            if let Some(t) = find_build_only(&p.tree) {
                return Err(format!("{}: pattern still has {t:?} in it", p.name));
            }
        }
        self.patterns = out;
        Ok(())
    }

    /// Whether a pattern with fixed immediates can be encoded.
    fn pattern_fits(&self, p: &Pattern) -> bool {
        p.emit.iter().all(|e| {
            let vals: Vec<Option<i64>> =
                e.args.iter().map(|a| if let Src::Imm(v) = a { Some(*v) } else { None }).collect();
            self.instrs[e.instr].fits(&vals)
        })
    }

    /// The instructions a recipe's expansion emits.
    fn expansion(&self, f: &Function) -> Result<Vec<Emit>, String> {
        let m = &self.module;
        // Each local of the expansion: a parameter, an earlier result, or a
        // condition form's value.
        #[derive(Clone, Copy)]
        enum Is {
            Src(Src),
            Cond(i64),
            Flags,
        }
        let mut is: HashMap<LocalId, Is> = HashMap::new();
        for (i, p) in f.params.iter().enumerate() {
            is.insert(p.local, Is::Src(Src::Param(i)));
        }
        let mut emit = Vec::new();
        let void = matches!(m.types.get(f.ty), Type::Function { ret, .. } if matches!(m.types.get(*ret), Type::Void));
        let insts: Vec<InstId> = f.blocks[f.entry().index()].insts.clone();
        let src_of = |is: &HashMap<LocalId, Is>, v: Value| -> Result<Is, String> {
            match v {
                Value::Local(x) => is.get(&x).copied().ok_or_else(|| "unknown value".to_string()),
                Value::Const(c) => match &m.constant(c).kind {
                    ConstKind::Int(a) => Ok(Is::Src(Src::Imm(sext(a.low_u64(), type_bits(m, m.constant(c).ty))))),
                    _ => Err("unsupported constant".into()),
                },
                _ => Err("metadata".into()),
            }
        };
        let mut result = None;
        for i in insts {
            let inst = &f.insts[i.index()];
            match &inst.op {
                // Casts between types in the same register are free.
                Op::Cast { value, .. } => {
                    let s = src_of(&is, *value)?;
                    is.insert(inst.result.unwrap(), s);
                }
                Op::Call(call) => {
                    let Value::Const(c) = call.callee else { return Err("indirect call".into()) };
                    let ConstKind::Global(g) = m.constant(c).kind else { return Err("call".into()) };
                    if let Some(form) = self.conds.iter().find(|k| k.func == g) {
                        is.insert(inst.result.unwrap(), Is::Cond(form.value));
                        continue;
                    }
                    let name = m.global(g).name.clone().unwrap_or_default();
                    let idx = *self.by_name.get(&name).ok_or_else(|| format!("no instruction @{name}"))?;
                    let ins = &self.instrs[idx];
                    let mut args = Vec::new();
                    let mut k = 0;
                    for o in &ins.operands {
                        if o.name == "d" && ins.has_result {
                            args.push(Src::Temp(emit.len()));
                            continue;
                        }
                        args.push(match src_of(&is, call.args[k].value)? {
                            Is::Src(s) => s,
                            Is::Cond(v) => Src::Imm(v),
                            Is::Flags => return Err("flags as an operand".into()),
                        });
                        k += 1;
                    }
                    if let Some(r) = inst.result {
                        is.insert(r, if ins.sets.is_some() { Is::Flags } else { Is::Src(Src::Temp(emit.len())) });
                    }
                    emit.push(Emit { instr: idx, args });
                }
                Op::Ret(Some(v)) => {
                    let Is::Src(Src::Temp(k)) = src_of(&is, *v)? else {
                        return Err("must return an instruction's result".into());
                    };
                    result = Some(k);
                }
                // A recipe for an operation with no result, such as a call
                // to an intrinsic that only changes the machine's state.
                Op::Ret(None) if void => {}
                op => return Err(format!("unsupported in an expansion: {op:?}")),
            }
        }
        // The returned temporary is the pattern's result.
        if !void {
            let r = result.ok_or("expansion without a result")?;
            for e in &mut emit {
                for a in &mut e.args {
                    if *a == Src::Temp(r) {
                        *a = Src::Result;
                    }
                }
            }
        }
        Ok(emit)
    }
}

/// An atomic access ordered at least as strongly as acquire: it needs
/// `ldar`. Weaker atomic loads are plain loads, which are single-copy
/// atomic when aligned.
pub fn acquires(a: &Option<Atomic>) -> bool {
    a.as_ref().is_some_and(|a| matches!(a.ordering, Ordering::Acquire | Ordering::AcqRel | Ordering::SeqCst))
}

/// An atomic store at least as strong as release: it needs `stlr`.
pub fn releases(a: &Option<Atomic>) -> bool {
    a.as_ref().is_some_and(|a| matches!(a.ordering, Ordering::Release | Ordering::AcqRel | Ordering::SeqCst))
}

/// The floating-point comparison true exactly when `p` is false.
pub fn inverse_f(p: FCmpPred) -> Option<FCmpPred> {
    use FCmpPred::*;
    Some(match p {
        Oeq => Une,
        Une => Oeq,
        Ogt => Ule,
        Ule => Ogt,
        Oge => Ult,
        Ult => Oge,
        Olt => Uge,
        Uge => Olt,
        Ole => Ugt,
        Ugt => Ole,
        One => Ueq,
        Ueq => One,
        Ord => Uno,
        Uno => Ord,
        False => True,
        True => False,
    })
}

pub fn inverse(p: ICmpPred) -> ICmpPred {
    use ICmpPred::*;
    match p {
        Eq => Ne,
        Ne => Eq,
        Ugt => Ule,
        Uge => Ult,
        Ult => Uge,
        Ule => Ugt,
        Sgt => Sle,
        Sge => Slt,
        Slt => Sge,
        Sle => Sgt,
    }
}

fn find_build_only(t: &Tree) -> Option<Tree> {
    match t {
        Tree::Flags(..) | Tree::Poison => Some(t.clone()),
        Tree::Op(_, kids) => kids.iter().find_map(find_build_only),
        _ => None,
    }
}

fn preorder_wild(t: &Tree, out: &mut Vec<Option<usize>>) {
    match t {
        Tree::Leaf(p) => out.push(Some(*p)),
        Tree::Next => out.push(None),
        Tree::Op(_, kids) => kids.iter().for_each(|k| preorder_wild(k, out)),
        _ => {}
    }
}

// ---------------------------------------------------------------------------
// Checking

/// An instruction and the immediates it is asked to hold.
type FitsKey = (usize, Vec<Option<i64>>);

/// A recipe, for checking: its name, its parameters' widths, the tree of
/// the IR it replaces, and the tree of what its instructions do.
pub type RecipeCheck = (String, Vec<u32>, Tree, Tree);

impl Target {
    /// For each recipe: the tree of the IR it replaces, and the tree of
    /// what its instructions do, with their bodies put in for the calls.
    /// Recipes taking vectors are left out: the evaluator holds a value in
    /// 64 bits. (The vector programs in `tests/programs` run them.)
    pub fn recipe_trees(&self) -> Result<Vec<RecipeCheck>, String> {
        let m = &self.module;
        let mut out = Vec::new();
        for (_, f) in m.functions() {
            let Some(Some(expand)) = attr(&f.fn_attrs, "recipe") else { continue };
            if f.params.iter().any(|p| matches!(m.types.get(f.locals[p.local.index()].ty), Type::Vector { .. })) {
                continue;
            }
            // Nor scalar ones made of vector instructions (a population
            // count): the program tests check those.
            let through_vectors =
                m.global_order.iter().copied().find(|&g| m.global(g).name.as_deref() == Some(expand)).is_some_and(
                    |g| {
                        let e = self.func(g);
                        e.locals.iter().any(|l| matches!(m.types.get(l.ty), Type::Vector { .. }))
                    },
                );
            // Nor those that touch memory (atomic read-modify-writes, and
            // intrinsics given pointers: x86's x87 comparisons): there is
            // no memory to evaluate them on.
            let touches_memory = f.blocks.iter().flat_map(|b| &b.insts).any(|i| match &f.insts[i.index()].op {
                Op::AtomicRmw { .. } | Op::CmpXchg { .. } | Op::Load { .. } | Op::Store { .. } => true,
                Op::Call(c) => c.args.iter().any(|a| match a.value {
                    Value::Local(x) => matches!(m.types.get(f.locals[x.index()].ty), Type::Ptr(_)),
                    _ => false,
                }),
                _ => false,
            });
            // Nor those with no value, which only change the machine's
            // state (SME's ZA and predicates): nothing to evaluate.
            let void =
                matches!(m.types.get(f.ty), Type::Function { ret, .. } if matches!(m.types.get(*ret), Type::Void));
            if through_vectors || touches_memory || void {
                continue;
            }
            let want = self.body_tree(f, &HashMap::new(), 0)?.ok_or("recipe without a value")?;
            let g = m
                .global_order
                .iter()
                .copied()
                .find(|&g| m.global(g).name.as_deref() == Some(expand))
                .ok_or("no expansion")?;
            let e = self.func(g);
            let ret = self.ret_value(e).ok_or("expansion without a value")?;
            let got = self.eval_value(e, ret, &HashMap::new(), 0).map_err(|err| format!("@{expand}: {err}"))?;
            let bits = f.params.iter().map(|p| type_bits(m, f.locals[p.local.index()].ty)).collect();
            out.push((expand.to_string(), bits, want, got));
        }
        Ok(out)
    }

    /// Evaluates a tree on parameter values (each held in the low bits of
    /// a `u64`). `None` if the result is undefined, as for division by zero.
    pub fn eval_tree(&self, t: &Tree, args: &[u64]) -> Option<u64> {
        let mask = |bits: u32| if bits >= 64 { u64::MAX } else { (1u64 << bits) - 1 };
        Some(match t {
            Tree::Leaf(p) => args[*p],
            Tree::Lit(v, bits) => *v as u64 & mask(*bits),
            Tree::Op(h, kids) => {
                let k: Vec<u64> = kids.iter().map(|c| self.eval_tree(c, args)).collect::<Option<_>>()?;
                match *h {
                    Head::Bin(op, bits) => {
                        let m = mask(bits);
                        let (a, b) = (k[0] & m, k[1] & m);
                        let (sa, sb) = (sext(a, bits), sext(b, bits));
                        (match op {
                            BinOp::Add => a.wrapping_add(b),
                            BinOp::Sub => a.wrapping_sub(b),
                            BinOp::Mul => a.wrapping_mul(b),
                            BinOp::And => a & b,
                            BinOp::Or => a | b,
                            BinOp::Xor => a ^ b,
                            BinOp::Shl if b < bits as u64 => a << b,
                            BinOp::LShr if b < bits as u64 => a >> b,
                            BinOp::AShr if b < bits as u64 => (sa >> b) as u64,
                            BinOp::UDiv if b != 0 => a / b,
                            BinOp::URem if b != 0 => a % b,
                            BinOp::SDiv if b != 0 && !(sa == sext(1 << (bits - 1), bits) && sb == -1) => {
                                (sa / sb) as u64
                            }
                            BinOp::SRem if b != 0 && !(sa == sext(1 << (bits - 1), bits) && sb == -1) => {
                                (sa % sb) as u64
                            }
                            _ => return None,
                        }) & m
                    }
                    Head::ICmp(p, bits) => {
                        let m = mask(bits);
                        let (a, b) = (k[0] & m, k[1] & m);
                        let (sa, sb) = (sext(a, bits), sext(b, bits));
                        u64::from(match p {
                            ICmpPred::Eq => a == b,
                            ICmpPred::Ne => a != b,
                            ICmpPred::Ugt => a > b,
                            ICmpPred::Uge => a >= b,
                            ICmpPred::Ult => a < b,
                            ICmpPred::Ule => a <= b,
                            ICmpPred::Sgt => sa > sb,
                            ICmpPred::Sge => sa >= sb,
                            ICmpPred::Slt => sa < sb,
                            ICmpPred::Sle => sa <= sb,
                        })
                    }
                    Head::Select(_) => {
                        if k[0] & 1 != 0 {
                            k[1]
                        } else {
                            k[2]
                        }
                    }
                    Head::Cast(op, from, to) => {
                        let a = k[0] & mask(from);
                        (match op {
                            CastOp::ZExt | CastOp::Trunc | CastOp::BitCast => a,
                            CastOp::SExt => sext(a, from) as u64,
                            _ => return None,
                        }) & mask(to)
                    }
                    Head::Intrinsic(id, bits) => {
                        let m = mask(bits);
                        let (a, b) = (k[0] & m, k.get(1).copied().unwrap_or(0) & m);
                        let (sa, sb) = (sext(a, bits), sext(b, bits));
                        let name = &self.intrinsics[id as usize];
                        let pick = |x: bool| if x { a } else { b };
                        if name.starts_with("llvm.smax.") {
                            pick(sa >= sb)
                        } else if name.starts_with("llvm.smin.") {
                            pick(sa <= sb)
                        } else if name.starts_with("llvm.umax.") {
                            pick(a >= b)
                        } else if name.starts_with("llvm.umin.") {
                            pick(a <= b)
                        } else if name.starts_with("llvm.abs.") {
                            sa.unsigned_abs() & m
                        } else {
                            return None;
                        }
                    }
                    Head::FCmp(p, bits) => {
                        let (a, b) = if bits & !FP == 32 {
                            (f64::from(f32::from_bits(k[0] as u32)), f64::from(f32::from_bits(k[1] as u32)))
                        } else {
                            (f64::from_bits(k[0]), f64::from_bits(k[1]))
                        };
                        let uno = a.is_nan() || b.is_nan();
                        u64::from(match p {
                            FCmpPred::False => false,
                            FCmpPred::True => true,
                            FCmpPred::Oeq => !uno && a == b,
                            FCmpPred::Ogt => !uno && a > b,
                            FCmpPred::Oge => !uno && a >= b,
                            FCmpPred::Olt => !uno && a < b,
                            FCmpPred::Ole => !uno && a <= b,
                            FCmpPred::One => !uno && a != b,
                            FCmpPred::Ord => !uno,
                            FCmpPred::Ueq => uno || a == b,
                            FCmpPred::Ugt => uno || a > b,
                            FCmpPred::Uge => uno || a >= b,
                            FCmpPred::Ult => uno || a < b,
                            FCmpPred::Ule => uno || a <= b,
                            FCmpPred::Une => uno || a != b,
                            FCmpPred::Uno => uno,
                        })
                    }
                    _ => return None,
                }
            }
            _ => return None,
        })
    }
}

// ---------------------------------------------------------------------------
// The trie

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Key {
    Head(Head),
    Lit(i64, u32),
    Wild,
}

#[derive(Default, Debug)]
pub struct TrieNode {
    /// Children by key. Nodes have few, so a list searched in order beats
    /// a hash map.
    pub next: Vec<(Key, usize)>,
    pub accept: Vec<usize>,
}

impl TrieNode {
    #[inline]
    pub fn get(&self, k: &Key) -> Option<usize> {
        self.next.iter().find(|(x, _)| x == k).map(|&(_, c)| c)
    }
}

/// A discrimination trie of patterns, keyed by their preorder traversal.
#[derive(Default, Debug)]
pub struct Trie {
    pub nodes: Vec<TrieNode>,
}

fn preorder_keys(t: &Tree, out: &mut Vec<Key>) {
    match t {
        Tree::Op(h, kids) => {
            out.push(Key::Head(*h));
            kids.iter().for_each(|k| preorder_keys(k, out));
        }
        Tree::Lit(v, b) => out.push(Key::Lit(*v, *b)),
        Tree::Leaf(_) | Tree::Next => out.push(Key::Wild),
        Tree::Flags(..) | Tree::Poison => unreachable!(),
    }
}

impl Trie {
    fn insert(&mut self, p: &Pattern, id: usize) {
        if self.nodes.is_empty() {
            self.nodes.push(TrieNode::default());
        }
        let mut keys = Vec::new();
        preorder_keys(&p.tree, &mut keys);
        let mut n = 0;
        for k in keys {
            n = match self.nodes[n].get(&k) {
                Some(c) => c,
                None => {
                    self.nodes.push(TrieNode::default());
                    let c = self.nodes.len() - 1;
                    self.nodes[n].next.push((k, c));
                    c
                }
            };
        }
        self.nodes[n].accept.push(id);
    }
}

/// The operands `(base, offset)` of an entry whose body computes
/// `base + offset` first (see `Instr::base_offset`), `offset` an immediate
/// parameter and `base` not written back.
fn base_offset_of(m: &Module, f: &Function, has_result: bool) -> Option<(usize, usize)> {
    if !f.has_body {
        return None;
    }
    let first = *f.blocks[f.entry().index()].insts.first()?;
    let param = |v: Value| match v {
        Value::Local(l) => match f.locals[l.index()].def {
            LocalDef::Param(k) => Some(k as usize),
            _ => None,
        },
        _ => None,
    };
    let (base, off) = match &f.insts[first.index()].op {
        Op::Gep { source_ty, ptr, indices, .. }
            if matches!(m.types.get(*source_ty), Type::Int(8)) && indices.len() == 1 =>
        {
            (param(*ptr)?, param(indices[0])?)
        }
        Op::Binary { op: BinOp::Add, lhs, rhs, .. } => (param(*lhs)?, param(*rhs)?),
        _ => return None,
    };
    let is = |k: usize, name: &str| attr(&f.params[k].attrs, name).is_some();
    if !is(off, "imm") || is(base, "writeback") || attr(&f.params[base].attrs, "reg").is_none() {
        return None;
    }
    let at = has_result as usize;
    Some((base + at, off + at))
}

/// Whether the body's value is its first instruction's result: `%r = ...;
/// ret %r`.
fn returns_first(f: &Function) -> bool {
    let insts = &f.blocks[f.entry().index()].insts;
    let (Some(&first), Some(&last)) = (insts.first(), insts.last()) else { return false };
    insts.len() == 2
        && matches!(f.insts[last.index()].op, Op::Ret(Some(Value::Local(l))) if Some(l) == f.insts[first.index()].result)
}

/// An atomic ordering as instructions offer it: 0 none (monotonic,
/// unordered), 1 acquire, 2 release, 3 both (acq_rel and seq_cst, which on
/// the machines described are the same instructions).
pub fn order_rank(o: Ordering) -> u8 {
    match o {
        Ordering::Unordered | Ordering::Monotonic => 0,
        Ordering::Acquire => 1,
        Ordering::Release => 2,
        Ordering::AcqRel | Ordering::SeqCst => 3,
    }
}

/// A compare-and-swap's rank: its success ordering, with acquire too if the
/// failure ordering acquires.
pub fn cmpxchg_rank(success: Ordering, failure: Ordering) -> u8 {
    order_rank(success) | (order_rank(failure) & 1)
}
