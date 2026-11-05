//! Types, and the signatures of items.
//!
//! A type is a `Ty`: an index into an interned table of `TyKind`s, so equal
//! types have equal ids and a type is compared, copied and hashed as one
//! `u32`. Everything inside a `TyKind` is an id too (a `List` of generic
//! arguments, a `ConstId`, a `SigId`, a `DynId`, a `ParamId`, a `DefId`),
//! so `TyKind` is `Copy`.
//!
//! `collect.rs` gives every item its signature, keyed by its `DefId`:
//! generic parameters, the predicates of its bounds and `where` clause,
//! fields and variants, function signatures, impl headers, trait items,
//! and the types of aliases, constants and statics. `lower.rs` turns a type
//! written in the source into a `Ty`, using what name resolution found.
//!
//! There are no lifetimes in types: they matter for borrow checking, which
//! this compiler does not do, not for code generation.

pub(crate) mod collect;
pub(crate) mod lower;
mod print;

use crate::ast::NodeId;
use crate::hash::FastMap;
use crate::intern::Interner;
use crate::resolve::{DefId, Resolver};
use crate::session::{ParsedCrate, Session};

macro_rules! id {
    ($name:ident) => {
        #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, PartialOrd, Ord)]
        pub struct $name(pub u32);
    };
}

id!(Ty);
id!(List);
id!(ConstId);
id!(SigId);
id!(DynId);
id!(ParamId);
id!(OpaqueId);
id!(ClosureId);

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum IntTy {
    I8,
    I16,
    I32,
    I64,
    I128,
    Isize,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum UintTy {
    U8,
    U16,
    U32,
    U64,
    U128,
    Usize,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum FloatTy {
    F16,
    F32,
    F64,
    F128,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Mutbl {
    Not,
    Mut,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum TyKind {
    Bool,
    Char,
    Str,
    Int(IntTy),
    Uint(UintTy),
    Float(FloatTy),
    Never,
    /// `(A, B)`; `()` is the empty tuple.
    Tuple(List),
    Ref(Mutbl, Ty),
    Ptr(Mutbl, Ty),
    Array(Ty, ConstId),
    Slice(Ty),
    /// A struct, enum or union and its generic arguments.
    Adt(DefId, List),
    /// A function item (each has a type of its own) and its arguments.
    FnDef(DefId, List),
    FnPtr(SigId),
    Dyn(DynId),
    Param(ParamId),
    /// `<Args[0] as Trait<Args[1..]>>::Assoc`: the associated type's
    /// `DefId`, and the trait's arguments with the self type first.
    Projection(DefId, List),
    /// `impl Trait`.
    Opaque(OpaqueId),
    /// A closure (by where it is written) and its signature.
    Closure(ClosureId, SigId),
    /// A type from an `extern` block.
    Foreign(DefId),
    /// An inference variable (only while checking a body).
    Infer(u32),
    /// A constant as a generic argument (`N`, `3`, `{ N + 1 }`), only in
    /// argument lists.
    ConstArg(ConstId),
    /// What a type that could not be worked out is; errors were reported.
    Error,
}

/// A constant in a type: an array length or a const generic argument.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Const {
    Value(u128),
    Param(ParamId),
    /// An expression not yet evaluated: crate, file and node.
    Expr(u32, u32, NodeId),
    Error,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Abi {
    Rust,
    C,
    Other,
}

/// A function's signature: parameter types (the receiver first for a
/// method) and result.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct FnSig {
    pub inputs: List,
    pub output: Ty,
    pub unsafety: bool,
    pub variadic: bool,
    pub abi: Abi,
}

/// A trait, its arguments (without the self type), and associated type
/// bindings (`Item = u8`).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct TraitRef {
    pub def: DefId,
    pub args: List,
    pub bindings: Vec<(DefId, Ty)>,
    /// Bounds on associated types: `Iterator<Item: Copy>`.
    pub assoc_bounds: Vec<(DefId, TraitRef)>,
}

/// `dyn A + B`: its traits (the first is the principal, the rest auto
/// traits, in any order).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct DynData {
    pub traits: Vec<TraitRef>,
}

/// A predicate an item requires.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Pred {
    /// `self_ty: Trait<args>`.
    Trait(Ty, TraitRef),
    /// `<self_ty as Trait<args>>::Assoc == ty`, from `Assoc = ty` bounds.
    Projection(Ty, DefId, List, Ty),
}

#[derive(Clone, Debug)]
pub struct ParamDef<'s> {
    pub owner: DefId,
    /// Position in the owner's full parameter list (its parent's first).
    pub index: u32,
    pub name: &'s str,
    pub is_const: bool,
    /// Where it is declared (crate, file, node); `Self` of a trait has the
    /// trait's node.
    pub at: (u32, u32, NodeId),
}

/// An item's generic parameters (types and constants; lifetimes are not
/// kept).
#[derive(Clone, Debug, Default)]
pub struct Generics {
    pub parent: Option<DefId>,
    pub parent_count: u32,
    pub own: Vec<ParamId>,
    /// Defaults of the own parameters, where given.
    pub defaults: Vec<Option<Ty>>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CtorKind {
    Named,
    Tuple,
    Unit,
}

#[derive(Clone, Debug)]
pub struct FieldDef<'s> {
    /// The name, or the position for a tuple field.
    pub name: &'s str,
    pub ty: Ty,
}

#[derive(Clone, Debug)]
pub struct VariantDef<'s> {
    /// The variant's own definition (none for a struct or union).
    pub def: Option<DefId>,
    pub name: &'s str,
    pub kind: CtorKind,
    pub fields: Vec<FieldDef<'s>>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum AdtKind {
    Struct,
    Enum,
    Union,
}

#[derive(Clone, Debug)]
pub struct AdtDef<'s> {
    pub kind: AdtKind,
    pub variants: Vec<VariantDef<'s>>,
}

#[derive(Clone, Debug)]
pub struct ImplDef {
    pub def: DefId,
    pub self_ty: Ty,
    pub trait_ref: Option<TraitRef>,
    pub negative: bool,
}

#[derive(Clone, Debug, Default)]
pub struct TraitDef {
    /// Its supertraits, as predicates on `Self`.
    pub supertraits: Vec<TraitRef>,
    pub is_auto: bool,
}

/// An `impl Trait` type: who it belongs to, and its bounds.
#[derive(Clone, Debug)]
pub struct OpaqueDef {
    pub owner: DefId,
    pub bounds: Vec<TraitRef>,
}

/// The types and signatures of a session's crates.
pub struct TyCtxt<'s> {
    types: Interner<TyKind>,
    lists: Interner<Vec<Ty>>,
    consts: Interner<Const>,
    sigs: Interner<FnSig>,
    dyns: Interner<DynData>,
    closures: Interner<(u32, u32, NodeId)>,
    pub params: Vec<ParamDef<'s>>,
    /// Per `DefId`: generics, predicates, and (where it has one) its type.
    pub generics: Vec<Generics>,
    pub predicates: Vec<Vec<Pred>>,
    /// Aliases (their target), constants, statics, and an impl's
    /// associated types; set once, when first worked out.
    type_of: Vec<std::sync::OnceLock<Ty>>,
    pub fn_sig: Vec<Option<SigId>>,
    pub adts: FastMap<DefId, AdtDef<'s>>,
    pub traits: FastMap<DefId, TraitDef>,
    pub impls: Vec<ImplDef>,
    /// Items by where they are (crate, file, node).
    pub(crate) def_at: FastMap<lower::At, DefId>,
    /// Generic parameters by where they are declared.
    pub(crate) param_at: FastMap<lower::At, ParamId>,
    /// `Self` of each impl, trait and type definition, by its node.
    pub(crate) self_ty: FastMap<lower::At, Ty>,
    /// Lang items: `#[lang = "name"]` on an item.
    pub lang: FastMap<&'s str, DefId>,
    pub impl_index: FastMap<DefId, usize>,
    opaques: std::sync::Mutex<Vec<OpaqueDef>>,
    /// Why types came out as errors, with counts.
    errors: std::sync::Mutex<FastMap<String, usize>>,
    /// How many items have a signature, and how many contain an error.
    pub sig_count: usize,
    pub sig_errors: usize,
}

impl<'s> TyCtxt<'s> {
    fn new(defs: usize) -> Self {
        TyCtxt {
            types: Interner::default(),
            lists: Interner::default(),
            consts: Interner::default(),
            sigs: Interner::default(),
            dyns: Interner::default(),
            closures: Interner::default(),
            params: Vec::new(),
            generics: vec![Generics::default(); defs],
            predicates: vec![Vec::new(); defs],
            type_of: (0..defs).map(|_| std::sync::OnceLock::new()).collect(),
            fn_sig: vec![None; defs],
            adts: FastMap::default(),
            traits: FastMap::default(),
            impls: Vec::new(),
            def_at: FastMap::default(),
            param_at: FastMap::default(),
            self_ty: FastMap::default(),
            lang: FastMap::default(),
            impl_index: FastMap::default(),
            opaques: std::sync::Mutex::new(Vec::new()),
            errors: std::sync::Mutex::new(FastMap::default()),
            sig_count: 0,
            sig_errors: 0,
        }
    }

    /// The type of an alias, constant or static, once worked out.
    pub fn type_of(&self, d: DefId) -> Option<Ty> {
        self.type_of[d.0 as usize].get().copied()
    }

    pub(crate) fn set_type_of(&self, d: DefId, t: Ty) -> Ty {
        *self.type_of[d.0 as usize].get_or_init(|| t)
    }

    pub fn opaque(&self, o: OpaqueId) -> OpaqueDef {
        self.opaques.lock().unwrap()[o.0 as usize].clone()
    }

    pub(crate) fn new_opaque(&self, o: OpaqueDef) -> OpaqueId {
        let mut v = self.opaques.lock().unwrap();
        v.push(o);
        OpaqueId(v.len() as u32 - 1)
    }

    /// Counts a type that came out as an error, by reason.
    pub(crate) fn note_error(&self, why: &str) {
        *self.errors.lock().unwrap().entry(why.to_string()).or_default() += 1;
    }

    pub fn mk(&self, k: TyKind) -> Ty {
        Ty(self.types.intern(k))
    }

    pub fn kind(&self, t: Ty) -> TyKind {
        self.types.get(t.0)
    }

    pub fn list(&self, tys: Vec<Ty>) -> List {
        List(self.lists.intern(tys))
    }

    pub fn tys(&self, l: List) -> Vec<Ty> {
        self.lists.get(l.0)
    }

    pub fn konst(&self, c: Const) -> ConstId {
        ConstId(self.consts.intern(c))
    }

    pub fn const_of(&self, c: ConstId) -> Const {
        self.consts.get(c.0)
    }

    pub fn sig(&self, s: FnSig) -> SigId {
        SigId(self.sigs.intern(s))
    }

    pub fn sig_of(&self, s: SigId) -> FnSig {
        self.sigs.get(s.0)
    }

    pub fn dyn_(&self, d: DynData) -> DynId {
        DynId(self.dyns.intern(d))
    }

    pub fn dyn_of(&self, d: DynId) -> DynData {
        self.dyns.get(d.0)
    }

    /// The id of the closure written at node `at` (crate, file, node).
    pub fn closure_id(&self, at: (u32, u32, NodeId)) -> ClosureId {
        ClosureId(self.closures.intern(at))
    }

    pub fn closure_at(&self, c: ClosureId) -> (u32, u32, NodeId) {
        self.closures.get(c.0)
    }

    pub fn unit(&self) -> Ty {
        let empty = self.list(Vec::new());
        self.mk(TyKind::Tuple(empty))
    }

    pub fn error(&self) -> Ty {
        self.mk(TyKind::Error)
    }

    /// How many distinct types there are.
    pub fn type_count(&self) -> usize {
        self.types.len()
    }

    /// Whether `t` has an error type anywhere in it.
    pub fn has_error(&self, t: Ty) -> bool {
        match self.kind(t) {
            TyKind::Error => true,
            TyKind::Tuple(l) | TyKind::Adt(_, l) | TyKind::FnDef(_, l) | TyKind::Projection(_, l) => {
                self.tys(l).into_iter().any(|x| self.has_error(x))
            }
            TyKind::Ref(_, x) | TyKind::Ptr(_, x) | TyKind::Slice(x) => self.has_error(x),
            TyKind::Array(x, c) => self.has_error(x) || self.const_of(c) == Const::Error,
            TyKind::FnPtr(s) | TyKind::Closure(_, s) => {
                let s = self.sig_of(s);
                self.has_error(s.output) || self.tys(s.inputs).into_iter().any(|x| self.has_error(x))
            }
            TyKind::Dyn(d) => self.dyn_of(d).traits.iter().any(|t| self.trait_ref_has_error(t)),
            _ => false,
        }
    }

    pub fn trait_ref_has_error(&self, t: &TraitRef) -> bool {
        self.tys(t.args).into_iter().any(|x| self.has_error(x)) || t.bindings.iter().any(|b| self.has_error(b.1))
    }

    /// All of an item's parameters, its parent's first.
    pub fn all_params(&self, def: DefId) -> Vec<ParamId> {
        let g = &self.generics[def.0 as usize];
        let mut out = g.parent.map(|p| self.all_params(p)).unwrap_or_default();
        out.extend(&g.own);
        out
    }

    /// The arguments that name an item's own parameters: `T, U, N`.
    pub fn identity_args(&self, def: DefId) -> List {
        let tys = self.all_params(def).into_iter().map(|p| self.mk(TyKind::Param(p))).collect();
        self.list(tys)
    }
}

/// Type statistics, for progress reports.
#[derive(Debug, Default)]
pub struct TypeStats {
    pub signatures: usize,
    pub with_errors: usize,
    pub distinct_types: usize,
    pub impls: usize,
    /// Why types came out as errors, most common first.
    pub reasons: Vec<(String, usize)>,
}

impl Session {
    /// The signatures of every item of `crates`.
    pub fn collect_types<'s>(&'s self, crates: &[ParsedCrate<'s>], r: &Resolver<'s>) -> TyCtxt<'s> {
        collect::collect(crates, r)
    }
}

impl TyCtxt<'_> {
    pub fn stats(&self) -> TypeStats {
        let mut reasons: Vec<(String, usize)> = self.errors.lock().unwrap().iter().map(|(k, v)| (k.clone(), *v)).collect();
        reasons.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
        TypeStats {
            signatures: self.sig_count,
            with_errors: self.sig_errors,
            distinct_types: self.type_count(),
            impls: self.impls.len(),
            reasons,
        }
    }
}

pub use print::dump_signatures;

#[cfg(test)]
mod tests {
    use crate::session::{CrateRoot, Options, Session};

    /// The signatures of `src` as a crate `t` (one file).
    fn sigs(src: &str) -> String {
        let dir = std::env::temp_dir().join(format!("epitome-rustc2-ty-{}-{}", std::process::id(), src.len()));
        std::fs::create_dir_all(&dir).unwrap();
        let file = dir.join("lib.rs");
        std::fs::write(&file, src).unwrap();
        let session = Session::new(Options { threads: 2, ..Default::default() });
        let mut crates = session.parse_crates(vec![CrateRoot::new("t", &file)]);
        session.expand(&mut crates);
        let r = session.resolve(&crates);
        let tcx = session.collect_types(&crates, &r);
        let _ = std::fs::remove_dir_all(dir);
        super::dump_signatures(&tcx, &r)
    }

    #[test]
    fn items_and_their_signatures() {
        let s = sigs(
            "pub trait FnOnce<A> { type Output; }\n\
             pub trait Fn<A>: FnOnce<A> {}\n\
             pub trait Copy {}\n\
             pub trait Tr { type Out; fn get(&self) -> Self::Out; }\n\
             pub struct P<T, const N: usize = 4> { xs: [T; N], n: usize }\n\
             pub enum E<T> { A, B(T, u8), C { x: Box<E<T>> } }\n\
             pub struct Box<T>(*const T);\n\
             type Pair<T> = (T, T);\n\
             impl<T: Tr> Tr for P<T> { type Out = Pair<T::Out>; fn get(&self) -> Self::Out { loop {} } }\n\
             pub fn f<F: Fn(u8) -> bool>(f: &F, p: P<u16, 2>) -> Option<&'static str> where F: Copy { None }\n\
             pub enum Option<T> { None, Some(T) }\n\
             const K: [u8; 3] = [1, 2, 3];",
        );
        for want in [
            "trait t::Tr",
            "type t::Tr::Out = <Self as Tr>::Out",
            "fn t::Tr::get(&Self) -> <Self as Tr>::Out",
            "struct t::P<T, const N> { xs: [T; N], n: usize }",
            "enum t::E<T> { A, B(T, u8), C { x: Box<E<T>> } }",
            "type t::Pair<T> = (T, T)",
            "impl<T> Tr for P<T, 4> where T: Tr",
            "type t::{impl}::Out = (<T as Tr>::Out, <T as Tr>::Out)",
            "fn t::{impl}::get(&P<T, 4>) -> (<T as Tr>::Out, <T as Tr>::Out)",
            "fn t::f<F>(&F, P<u16, 2>) -> Option<&str> where <F as FnOnce<(u8,)>>::Output == bool, F: Fn<(u8,), Output = bool>, F: Copy",
            "const t::K: [u8; 3]",
        ] {
            assert!(s.lines().any(|l| l == want), "no `{want}` in:\n{s}");
        }
        assert!(!s.contains("{error}"), "{s}");
    }
}
