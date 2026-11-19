//! Lowering to Epitome IR: every function instance monomorphisation
//! found becomes an IR function, its body walked straight from the syntax
//! tree with type checking's tables (section 8 of the design).
//!
//! How values are held:
//!
//! - A type whose layout is one scalar (an integer, `bool`, `char`, a
//!   float, a thin pointer, or a struct wrapping one) is an SSA value of
//!   the matching IR type: `iN` by size (`bool` is `i8`), `float` or
//!   `double`, or `ptr`.
//! - A type of size zero is no value at all.
//! - Everything else (structs, enums, tuples, arrays, and pointers to
//!   unsized types, which are an address and a length or vtable) is kept
//!   in memory, and handled by its address.
//!
//! Every local is a stack slot (an `alloca` in the entry block), and
//! places (`a.b[i]`, `*p`) are address computations, so Epitome's
//! `mem2reg` and SROA make registers of them.
//!
//! Our own functions call each other with this ABI: a scalar argument is
//! passed as itself, a zero-sized one not at all, and anything in memory
//! by the address of a copy the callee may change (the callee copies it
//! into its own slot first, so a caller may pass the address of a place
//! it still uses). A result in memory is written through a pointer passed
//! first. `extern "C"` functions take and give scalars only, for now.
//!
//! Panics are `core`'s and `std`'s own code (`panic=abort`): an index out
//! of bounds or a division by zero calls `core`'s lang item for it, and
//! `core`'s `panic_impl` is the program's `#[panic_handler]`. A
//! `#[track_caller]` function takes its caller's `Location` last.
//! Arithmetic wraps, as in a release build.
//!
//! A function whose body uses something lowering does not support yet
//! becomes a body that traps, and the reason is kept: `--lower-stats`
//! counts them.

mod asm;
mod body;
mod drop;
mod image;
pub(crate) use body::parse_str;
mod intrinsic;
mod pat;
mod quad;

use crate::hash::FastMap;
use crate::layout::{Abi, Layout, LayoutCx, Prim, TargetData};
use crate::mono::{Item, MonoItems};
use crate::resolve::{DefId, DefKind, Resolver};
use crate::session::{ParsedCrate, Session};
use crate::ty::{List, Ty, TyCtxt, TyKind};
use crate::typeck::{Index, Solver, TypeckResults};
use epitome_ir::module::{Function as IrFunction, GlobalId, Linkage, Module};
use epitome_ir::types::TypeId;
use std::sync::Arc;

/// The machine the code is for, as far as lowering cares.
#[derive(Clone, Copy, Debug)]
pub struct Target {
    pub triple: &'static str,
    pub data_layout: &'static str,
    /// The size of a pointer, in bytes.
    pub ptr_size: u64,
    /// The CPU every function is compiled for (`"target-cpu"`), as
    /// rustc's target gives it; none for the code generator's default.
    pub cpu: Option<&'static str>,
}

impl Target {
    pub fn x86_64_linux() -> Target {
        Target {
            triple: "x86_64-unknown-linux-gnu",
            data_layout: "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-i128:128-f80:128-n8:16:32:64-S128",
            ptr_size: 8,
            cpu: None,
        }
    }

    /// macOS on Apple silicon: every Mac with it has the M1's features
    /// (LSE atomics among them, so atomics are instructions, not calls of
    /// helpers macOS does not have).
    pub fn aarch64_darwin() -> Target {
        Target {
            triple: "arm64-apple-macosx14.0.0",
            data_layout: "e-m:o-i64:64-i128:128-n32:64-S128-Fn32",
            ptr_size: 8,
            cpu: Some("apple-m1"),
        }
    }

    /// The machine this compiler runs on.
    pub fn host() -> Target {
        if cfg!(all(target_arch = "aarch64", target_os = "macos")) { Target::aarch64_darwin() } else { Target::x86_64_linux() }
    }
}

/// How a value of a type is held.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Repr {
    /// Size zero (or no value at all): nothing.
    Zst,
    /// An SSA value of this IR type.
    Imm(TypeId),
    /// In memory, by address.
    Mem,
}

/// How a function takes its arguments and gives its result.
#[derive(Clone, Debug)]
pub(crate) struct FnAbi {
    pub inputs: Vec<Ty>,
    pub output: Ty,
    pub args: Vec<Repr>,
    pub ret: Repr,
    pub variadic: bool,
    /// Whether it takes its caller's `&'static Location` last, unseen
    /// (`#[track_caller]`).
    pub caller: bool,
}

/// A function of the module.
#[derive(Clone, Debug)]
pub(crate) struct Decl {
    pub g: GlobalId,
    pub abi: FnAbi,
    pub name: String,
}

/// A function (or static) lowering could not finish.
#[derive(Clone, Debug)]
pub struct Failure {
    pub item: Item,
    /// Its symbol.
    pub name: String,
    pub why: String,
}

/// What lowering made.
pub struct Lowered {
    pub module: Module,
    /// Functions defined.
    pub functions: usize,
    /// Each function lowering could not finish, with why: their bodies
    /// trap.
    pub failures: Vec<Failure>,
    /// Functions lowered but for something they use that is not supported
    /// yet (`dyn`, formatting), where they trap: only panics reach these
    /// in the programs so far.
    pub partial: Vec<Failure>,
    /// Comments for `--dump-ir`: the source of each statement, before its
    /// first instruction.
    notes: FastMap<GlobalId, Vec<(epitome_ir::module::BlockId, usize, String)>>,
    /// The Rust item each function is, for its heading.
    titles: FastMap<GlobalId, String>,
    /// The function each item is.
    decls: FastMap<Item, GlobalId>,
}

impl Lowered {
    /// Makes the functions `items` are external (those lowered), so that
    /// optimising keeps them, as for a library's.
    pub fn export(&mut self, items: &[Item]) {
        for it in items {
            if let Some(&g) = self.decls.get(it) {
                let p = &mut self.module.globals[g.index()].props;
                p.linkage = Linkage::External;
                p.explicit_linkage = false;
            }
        }
    }

    /// The module as text, each function headed by the item it is, and
    /// each statement's source as a comment before its instructions.
    pub fn dump(&self) -> String {
        use epitome_ir::printer::Place;
        let m = &self.module;
        let notes = |g: GlobalId, p: Place| -> Option<String> {
            let f = match &m.global(g).kind {
                epitome_ir::module::GlobalKind::Function(f) => f,
                _ => return None,
            };
            let mut out = Vec::new();
            if let Place::Block(b) = p
                && f.layout.first() == Some(&b)
                && let Some(t) = self.titles.get(&g)
            {
                out.push(t.clone());
            }
            if let Place::Inst(i) = p {
                for (b, k, text) in self.notes.get(&g).into_iter().flatten() {
                    if f.blocks[b.index()].insts.get(*k) == Some(&i) {
                        out.push(text.clone());
                    }
                }
            }
            (!out.is_empty()).then(|| out.join("\n"))
        };
        epitome_ir::printer::print_module_with(m, &notes)
    }
}

/// What lowering a body reads.
pub(crate) struct Cx<'a, 's> {
    pub tcx: &'a TyCtxt<'s>,
    pub r: &'a Resolver<'s>,
    pub crates: &'a [ParsedCrate<'s>],
    pub typeck: &'a TypeckResults,
    pub index: &'a Index<'s>,
    pub lcx: LayoutCx<'a, 's>,
    pub fns: FastMap<Item, Decl>,
    /// Each `static`'s global.
    pub statics: FastMap<DefId, GlobalId>,
    /// Which types need dropping.
    pub drops: crate::drops::DropInfo,
    /// Each closure lowered so far, and whether it owns its captures (an
    /// `FnOnce` that moves out of them), so that its callers do not drop
    /// it after calling it.
    pub owning: std::cell::RefCell<FastMap<Item, bool>>,
    /// For each closure that captures nothing, a function of its
    /// signature that calls it (what it is as a function pointer).
    pub fnptr: FastMap<Item, Decl>,
    /// For each closure in a vtable of the `Fn` traits, its `call_once`
    /// there: it calls the closure, then drops its captures (unless the
    /// closure does).
    pub once: FastMap<Item, Decl>,
    /// Whether `core` and `std` are built with `panic = "immediate-abort"`,
    /// where each of their panic functions aborts before anything else.
    pub immediate_abort: bool,
}

impl<'a, 's> Cx<'a, 's> {
    /// Whether function `d` is `#[track_caller]`: itself, or (a trait
    /// impl's method) the trait's method it implements, which makes every
    /// impl's so (`SliceIndex::index`).
    pub fn track_caller(&self, d: DefId) -> bool {
        let (_, _, ast, node) = self.def_ast(d);
        if crate::session::has_attribute(ast, node, "track_caller") {
            return true;
        }
        let r = self.r;
        let def = &r.defs[d.0 as usize];
        let Some(imp) = def.parent.and_then(|m| r.modules[m.0 as usize].def) else { return false };
        let Some(tr) = self.tcx.impl_index.get(&imp).and_then(|&i| self.tcx.impls[i].trait_ref.as_ref()).map(|t| t.def) else { return false };
        match r.defs[tr.0 as usize].module.and_then(|m| r.modules[m.0 as usize].names.get(&(def.name, crate::resolve::Ns::Value))).map(|b| &b.res) {
            Some(crate::resolve::Res::Def(td)) => {
                let (_, _, ast, node) = self.def_ast(*td);
                crate::session::has_attribute(ast, node, "track_caller")
            }
            _ => false,
        }
    }

    pub fn solver(&self) -> Solver<'a, 's> {
        Solver::new(self.tcx, self.r, self.crates, self.index)
    }

    pub fn layout(&self, t: Ty) -> Result<Arc<Layout>, String> {
        self.lcx.layout_of(t).map_err(|e| format!("no layout for {}: {e:?}", self.tcx.display(self.r, t)))
    }

    /// How a value of type `t` is held.
    pub fn repr(&self, m: &mut Module, t: Ty) -> Result<Repr, String> {
        let l = self.layout(t)?;
        Ok(match l.abi {
            Abi::Uninhabited => Repr::Zst,
            _ if l.size == 0 && l.is_sized() => Repr::Zst,
            Abi::Scalar(s) => Repr::Imm(match s.prim {
                Prim::Int { size, .. } => m.types.int(size as u32 * 8),
                Prim::Float { size: 2 } => m.types.half(),
                Prim::Float { size: 4 } => m.types.float(),
                Prim::Float { size: 8 } => m.types.double(),
                Prim::Float { .. } => m.types.intern(epitome_ir::types::Type::Fp128),
                Prim::Pointer => m.types.ptr(0),
            }),
            _ => Repr::Mem,
        })
    }

    pub fn abi(&self, m: &mut Module, inputs: Vec<Ty>, output: Ty, variadic: bool) -> Result<FnAbi, String> {
        let args = inputs.iter().map(|&t| self.repr(m, t)).collect::<Result<Vec<_>, _>>()?;
        let ret = self.repr(m, output)?;
        Ok(FnAbi { inputs, output, args, ret, variadic, caller: false })
    }

    /// The IR type of a function with this ABI.
    pub fn ir_fn_type(m: &mut Module, abi: &FnAbi) -> (TypeId, Vec<TypeId>) {
        let ptr = m.types.ptr(0);
        let mut params = Vec::new();
        if abi.ret == Repr::Mem {
            params.push(ptr);
        }
        for a in &abi.args {
            match a {
                Repr::Zst => {}
                Repr::Imm(t) => params.push(*t),
                Repr::Mem => params.push(ptr),
            }
        }
        if abi.caller {
            params.push(ptr);
        }
        let ret = match abi.ret {
            Repr::Imm(t) => t,
            _ => m.types.void(),
        };
        (ret, params)
    }

    /// The signature of function `d` at `args`, made concrete.
    pub fn signature(&self, solver: &mut Solver, d: DefId, args: &[Ty]) -> Result<(Vec<Ty>, Ty, bool, bool), String> {
        let sig = self.tcx.fn_sig[d.0 as usize].ok_or_else(|| format!("{} has no signature", self.r.def_path(d)))?;
        let sig = self.tcx.sig_of(sig);
        let params = self.tcx.all_params(d);
        let sub = |solver: &mut Solver, t: Ty| crate::mono::sub(self.tcx, &self.typeck.hidden, solver, t, &params, args);
        let inputs: Vec<Ty> = self.tcx.tys(sig.inputs).into_iter().map(|t| sub(solver, t)).collect();
        let output = sub(solver, sig.output);
        Ok((inputs, output, sig.abi == crate::ty::Abi::C, sig.variadic))
    }

    /// The type of closure `c` in its owner `d` at `l`.
    pub fn closure_ty(&self, solver: &mut Solver, c: crate::ty::ClosureId, d: DefId, l: List) -> Result<Ty, String> {
        let (k, f, node) = self.tcx.closure_at(c);
        let t = self.typeck.types[k as usize][f as usize].get(node.0 as usize).copied().flatten().ok_or("closure without a type")?;
        Ok(crate::mono::sub(self.tcx, &self.typeck.hidden, solver, t, &self.tcx.all_params(d), &self.tcx.tys(l)))
    }

    /// A closure's function takes a pointer to the closure (its captures)
    /// first, then its parameters.
    pub fn closure_abi(&self, solver: &mut Solver, m: &mut Module, c: crate::ty::ClosureId, d: DefId, l: List) -> Result<FnAbi, String> {
        let t = self.closure_ty(solver, c, d, l)?;
        let TyKind::Closure(_, s, _) = self.tcx.kind(t) else { return Err("closure of no closure type".into()) };
        let sig = self.tcx.sig_of(s);
        let mut inputs = vec![self.tcx.mk(TyKind::Ref(crate::ty::Mutbl::Mut, t))];
        inputs.extend(self.tcx.tys(sig.inputs));
        self.abi(m, inputs, sig.output, false)
    }

    /// A description of `t` that tells any two types apart and is the same
    /// on every run: items by path, closures by where they are written.
    pub fn type_key(&self, t: Ty, out: &mut String) {
        let tcx = self.tcx;
        let list = |cx: &Self, l: List, out: &mut String| {
            out.push('<');
            for x in tcx.tys(l) {
                cx.type_key(x, out);
                out.push(',');
            }
            out.push('>');
        };
        match tcx.kind(t) {
            TyKind::Adt(d, l) | TyKind::FnDef(d, l) => {
                let (k, f, _, n) = self.def_ast(d);
                out.push_str(&format!("{}@{k}:{f}:{}", self.r.def_path(d), n.0));
                list(self, l, out);
            }
            TyKind::Closure(c, _, l) => {
                let (k, f, n) = tcx.closure_at(c);
                out.push_str(&format!("closure@{k}:{f}:{}", n.0));
                list(self, l, out);
            }
            TyKind::Tuple(l) => {
                out.push_str("tuple");
                list(self, l, out);
            }
            TyKind::Ref(m, x) | TyKind::Ptr(m, x) => {
                out.push_str(match (tcx.kind(t), m) {
                    (TyKind::Ref(..), crate::ty::Mutbl::Not) => "&",
                    (TyKind::Ref(..), _) => "&mut ",
                    (_, crate::ty::Mutbl::Not) => "*const ",
                    _ => "*mut ",
                });
                self.type_key(x, out);
            }
            TyKind::Slice(x) => {
                out.push('[');
                self.type_key(x, out);
                out.push(']');
            }
            TyKind::Array(x, c) => {
                out.push('[');
                self.type_key(x, out);
                out.push_str("; ");
                self.const_key(c, out);
                out.push(']');
            }
            TyKind::ConstArg(c) => self.const_key(c, out),
            _ => out.push_str(&tcx.display(self.r, t)),
        }
    }

    /// A constant for `type_key`: its value, or, when the evaluator cannot
    /// work it out, where it is written, so that two such constants are
    /// not taken for the same one.
    fn const_key(&self, c: crate::ty::ConstId, out: &mut String) {
        match (self.lcx.eval_const(c, &Default::default()), self.tcx.const_of(c)) {
            (Some(v), _) => out.push_str(&v.to_string()),
            (None, crate::ty::Const::Expr(k, f, n)) => out.push_str(&format!("expr@{k}:{f}:{}", n.0)),
            (None, c) => out.push_str(&format!("{c:?}")),
        }
    }

    /// The node and file of a definition.
    pub fn def_ast(&self, d: DefId) -> (u32, u32, &'a crate::ast::Ast<'s>, crate::ast::NodeId) {
        let def = &self.r.defs[d.0 as usize];
        (def.krate, def.file, &self.crates[def.krate as usize].files[def.file as usize].ast, def.node)
    }

    /// The `"target-features"` of function `d` (`+avx,+avx2`), from its
    /// `#[target_feature(enable = "...")]`s: each feature and those it
    /// implies, in LLVM's names, as rustc gives them (x86-64 only, so far).
    pub fn target_features(&self, d: DefId, target: Target) -> Option<String> {
        let (_, _, ast, node) = self.def_ast(d);
        let table = match target.triple {
            t if t.starts_with("x86_64") => x86_64_features(),
            _ => return None,
        };
        let mut out: Vec<&str> = Vec::new();
        for (name, a) in crate::session::attributes(ast, node) {
            if name != "target_feature" {
                continue;
            }
            let inner = ast.child(a, crate::ast::Kind::TokenTree).map(|tt| ast.text(tt)).unwrap_or_default();
            // `(enable = "a,b")`: the string's features.
            let Some(list) = inner.split_once('"').and_then(|(_, rest)| rest.split_once('"')).map(|(l, _)| l) else {
                continue;
            };
            for f in list.split(',').map(str::trim) {
                for llvm in table.get(f).into_iter().flatten() {
                    if !out.contains(&llvm.as_str()) {
                        out.push(llvm);
                    }
                }
            }
        }
        (!out.is_empty()).then(|| out.join(","))
    }

    /// The name a function (or static) has in the object file: its own for
    /// `#[no_mangle]`, `#[export_name]` and foreign items, else its path and
    /// arguments, made safe for an assembler, with a hash that tells
    /// instances apart.
    pub fn symbol(&self, d: DefId, args: List) -> String {
        let (k, f, ast, node) = self.def_ast(d);
        let kind = self.r.defs[d.0 as usize].kind;
        if matches!(kind, DefKind::ForeignFn | DefKind::ForeignStatic) {
            return crate::session::attribute_value(ast, node, "link_name").unwrap_or_else(|| self.r.defs[d.0 as usize].name.to_string());
        }
        if let Some(n) = crate::session::attribute_value(ast, node, "export_name") {
            return n;
        }
        // `#[rustc_std_internal_symbol]`: a function one standard library
        // crate defines and another declares (`__rust_start_panic`), by its
        // name on both sides.
        if crate::session::has_attribute(ast, node, "no_mangle") || crate::session::has_attribute(ast, node, "rustc_std_internal_symbol") {
            return self.r.defs[d.0 as usize].name.to_string();
        }
        let mut text = crate::mono::item_path(self.r, self.tcx, d);
        let a = self.tcx.tys(args);
        if !a.is_empty() {
            let s: Vec<String> = a.iter().map(|&t| self.tcx.display(self.r, t)).collect();
            text = format!("{text}<{}>", s.join(", "));
        }
        // The hash is of where the item is written (two impls of a trait
        // for one type print alike) and of its arguments in full: paths,
        // not just names, and where each closure is.
        let mut key = format!("{k}:{f}:{}", node.0);
        for &t in &a {
            key.push('|');
            self.type_key(t, &mut key);
        }
        let h = crate::hash::fnv(key.as_bytes());
        let safe: String = text.chars().map(|c| if c.is_ascii_alphanumeric() || c == '_' { c } else { '.' }).collect();
        let mut safe = safe.replace("..", ".");
        safe.truncate(120);
        format!("_R{safe}.{h:016x}")
    }
}

impl Session {
    /// Lowers the function instances of `mono` to one IR module for
    /// `target`.
    #[allow(clippy::too_many_arguments)]
    pub fn lower<'s>(
        &'s self,
        crates: &[ParsedCrate<'s>],
        r: &Resolver<'s>,
        tcx: &TyCtxt<'s>,
        index: &Index<'s>,
        typeck: &TypeckResults,
        mono: &MonoItems,
        target: Target,
    ) -> Lowered {
        let mut lcx = LayoutCx::new(tcx, r, crates, index, TargetData { ptr_size: target.ptr_size });
        lcx.typeck = Some(typeck);
        let mut m = Module {
            triple: Some(target.triple.as_bytes().to_vec()),
            data_layout: Some(target.data_layout.as_bytes().to_vec()),
            ..Default::default()
        };
        let drops = crate::drops::DropInfo::new(tcx);
        let mut cx =
            Cx { tcx, r, crates, typeck, index, lcx, fns: FastMap::default(), statics: FastMap::default(), drops, owning: Default::default(), fnptr: FastMap::default(), once: FastMap::default(), immediate_abort: false };
        cx.immediate_abort = self.cfg_of(0).is_some_and(|c| c.has_value("panic", "immediate-abort"));
        let mut failures = Vec::new();
        let mut solver = cx.solver();

        // Every function is declared first, so that bodies can call any of
        // them; in order of name, so the module does not depend on the
        // order monomorphisation found them in.
        let mut named: Vec<(String, Item)> = Vec::new();
        for &it in &mono.items {
            match it {
                Item::Fn(d, l) => named.push((cx.symbol(d, l), it)),
                Item::Foreign(d) if matches!(r.defs[d.0 as usize].kind, DefKind::ForeignFn | DefKind::AssocFn | DefKind::Fn) => {
                    named.push((cx.symbol(d, tcx.list(Vec::new())), it))
                }
                Item::Static(d) => named.push((cx.symbol(d, tcx.list(Vec::new())), it)),
                Item::Closure(c, d, l) => {
                    let (_, _, node) = tcx.closure_at(c);
                    let owner = cx.symbol(d, l);
                    named.push((format!("{owner}.closure.{}", node.0), it))
                }
                Item::DropGlue(t) => named.push((drop::glue_symbol(&cx, t), it)),
                _ => {}
            }
        }
        named.sort_by(|a, b| a.0.cmp(&b.0));
        // Items of one name are one instance (constant arguments written
        // apart, of the same value): the first is declared, and the
        // others share its declaration, whichever order they came in.
        let mut same = Vec::new();
        named.dedup_by(|a, b| {
            let dup = a.0 == b.0;
            if dup {
                same.push((a.1, b.1));
            }
            dup
        });
        let mut static_inits = Vec::new();
        for (name, it) in &named {
            match *it {
                Item::Fn(d, _) | Item::Foreign(d) => {
                    let l = if let Item::Fn(_, l) = *it { l } else { tcx.list(Vec::new()) };
                    let args = tcx.tys(l);
                    let abi = cx.signature(&mut solver, d, &args).and_then(|(i, o, _, v)| cx.abi(&mut m, i, o, v));
                    let mut abi = match abi {
                        Ok(a) => a,
                        Err(e) => {
                            failures.push(Failure { item: *it, name: name.clone(), why: e });
                            continue;
                        }
                    };
                    let foreign = matches!(it, Item::Foreign(_));
                    abi.caller = !foreign && cx.track_caller(d);
                    // An LLVM intrinsic of vectors (`core::arch`'s, by
                    // `link_name`) or aggregates takes them as values, where
                    // this ABI passes them in memory: SIMD is not lowered
                    // yet, so it is not declared, and calls of it fail.
                    if foreign && name.starts_with("llvm.") && (abi.ret == Repr::Mem || abi.args.contains(&Repr::Mem)) {
                        continue;
                    }
                    let (ret, params) = Cx::ir_fn_type(&mut m, &abi);
                    let (_, _, ast, node) = cx.def_ast(d);
                    let exported = foreign
                        || crate::session::has_attribute(ast, node, "no_mangle")
                        || crate::session::has_attribute(ast, node, "rustc_std_internal_symbol")
                        || crate::session::attribute_value(ast, node, "export_name").is_some();
                    let mut spec = IrFunction::builder(name).returns(ret);
                    for p in params {
                        spec = spec.param("", p);
                    }
                    if abi.variadic {
                        spec = spec.varargs();
                    }
                    if !exported {
                        spec = spec.linkage(Linkage::Internal);
                    }
                    let g = spec.declare(&mut m);
                    cx.fns.insert(*it, Decl { g, abi, name: name.clone() });
                }
                Item::Closure(c, d, l) => {
                    let abi = cx.closure_abi(&mut solver, &mut m, c, d, l);
                    let abi = match abi {
                        Ok(a) => a,
                        Err(e) => {
                            failures.push(Failure { item: *it, name: name.clone(), why: e });
                            continue;
                        }
                    };
                    let (ret, params) = Cx::ir_fn_type(&mut m, &abi);
                    let mut spec = IrFunction::builder(name).returns(ret).linkage(Linkage::Internal);
                    for p in params {
                        spec = spec.param("", p);
                    }
                    let g = spec.declare(&mut m);
                    cx.fns.insert(*it, Decl { g, abi, name: name.clone() });
                    // One that captures nothing can be a function pointer:
                    // a function of its signature that calls it.
                    if typeck.captures.get(&c).is_none_or(|v| v.is_empty())
                        && let Ok(TyKind::Closure(_, s, _)) = cx.closure_ty(&mut solver, c, d, l).map(|t| tcx.kind(t))
                        && let sig = tcx.sig_of(s)
                        && let Ok(abi) = cx.abi(&mut m, tcx.tys(sig.inputs), sig.output, false)
                    {
                        let fname = format!("{name}.fnptr");
                        let (ret, params) = Cx::ir_fn_type(&mut m, &abi);
                        let mut spec = IrFunction::builder(&fname).returns(ret).linkage(Linkage::Internal);
                        for p in params {
                            spec = spec.param("", p);
                        }
                        let g = spec.declare(&mut m);
                        cx.fnptr.insert(*it, Decl { g, abi, name: fname });
                    }
                }
                Item::DropGlue(t) => {
                    // `fn(*mut T)`.
                    let unit = tcx.mk(TyKind::Tuple(tcx.list(Vec::new())));
                    let pt = tcx.mk(TyKind::Ptr(crate::ty::Mutbl::Mut, t));
                    let abi = match cx.abi(&mut m, vec![pt], unit, false) {
                        Ok(a) => a,
                        Err(e) => {
                            failures.push(Failure { item: *it, name: name.clone(), why: e });
                            continue;
                        }
                    };
                    let (ret, params) = Cx::ir_fn_type(&mut m, &abi);
                    let mut spec = IrFunction::builder(name).returns(ret).linkage(Linkage::Internal);
                    for p in params {
                        spec = spec.param("", p);
                    }
                    let g = spec.declare(&mut m);
                    cx.fns.insert(*it, Decl { g, abi, name: name.clone() });
                }
                Item::Static(d) => match body::define_static(&cx, &mut m, d, name) {
                    Ok((g, needs_init)) => {
                        cx.statics.insert(d, g);
                        if needs_init {
                            static_inits.push((*it, d, g, name.clone()));
                        }
                    }
                    Err(e) => failures.push(Failure { item: *it, name: name.clone(), why: e }),
                },
                _ => {}
            }
        }

        for (it, first) in same {
            if let Some(d) = cx.fns.get(&first).cloned() {
                cx.fns.insert(it, d);
            }
        }
        // Each closure in a vtable of the `Fn` traits: its `call_once`.
        let fn_traits: Vec<DefId> = ["fn", "fn_mut", "fn_once"].iter().filter_map(|l| tcx.lang.get(l).copied()).collect();
        for &it in &mono.items {
            let Item::VTable(t, tr, _) = it else { continue };
            let TyKind::Closure(c, _, l) = tcx.kind(t) else { continue };
            if !fn_traits.contains(&tr) {
                continue;
            }
            let Some(&owner) = typeck.closure_owners.get(&c) else { continue };
            let closure = Item::Closure(c, owner, l);
            if cx.once.contains_key(&closure) {
                continue;
            }
            let Some(decl) = cx.fns.get(&closure).cloned() else { continue };
            let name = format!("{}.once", decl.name);
            let (ret, params) = Cx::ir_fn_type(&mut m, &decl.abi);
            let mut spec = IrFunction::builder(&name).returns(ret).linkage(Linkage::Internal);
            for p in params {
                spec = spec.param("", p);
            }
            let g = spec.declare(&mut m);
            cx.once.insert(closure, Decl { g, abi: decl.abi.clone(), name });
        }

        let mut notes = FastMap::default();
        let mut titles = FastMap::default();
        let mut functions = 0;
        let mut partial = Vec::new();
        // Closures first, so that every call of one by `call_once` knows
        // whether the closure drops its own captures: each is lowered not
        // owning them; those that move out of one (`FnOnce`), and those
        // that called a closure not lowered yet, are lowered again.
        let closures: Vec<&(String, Item)> = named.iter().filter(|x| matches!(x.1, Item::Closure(..))).collect();
        let mut again = Vec::new();
        for (name, it) in closures {
            let Some(decl) = cx.fns.get(it).cloned() else { continue };
            functions += 1;
            if let Item::Closure(c, d, l) = *it {
                titles.insert(decl.g, format!("closure at node {} in {}", tcx.closure_at(c).2.0, item_title(&cx, d, l)));
            }
            let (moved, unknown) = lower_item(&cx, &mut m, *it, &decl, name, false, &mut notes, &mut partial, &mut failures);
            cx.owning.borrow_mut().insert(*it, moved);
            if moved || unknown {
                again.push((name.clone(), *it, decl, moved));
            }
        }
        for (name, it, decl, owns) in again {
            if let epitome_ir::module::GlobalKind::Function(f) = &mut m.globals[decl.g.index()].kind {
                f.has_body = false;
            }
            failures.retain(|f| f.item != it);
            partial.retain(|f| f.item != it);
            lower_item(&cx, &mut m, it, &decl, &name, owns, &mut notes, &mut partial, &mut failures);
        }
        // Closures' `call_once` in vtables.
        let mut once: Vec<(Item, Decl)> = cx.once.iter().map(|(&i, d)| (i, d.clone())).collect();
        once.sort_by(|a, b| a.1.name.cmp(&b.1.name));
        for (it, shim) in once {
            let Some(closure) = cx.fns.get(&it).cloned() else { continue };
            let owns = cx.owning.borrow().get(&it).copied().unwrap_or(false);
            let drop = match (owns, it) {
                (false, Item::Closure(c, d, l)) => cx
                    .closure_ty(&mut solver, c, d, l)
                    .ok()
                    .filter(|&t| cx.drops.needs_drop(tcx, typeck, &mut solver, t))
                    .and_then(|t| cx.fns.get(&Item::DropGlue(t)).cloned()),
                _ => None,
            };
            once_shim(&mut m, &shim, &closure, drop.as_ref());
        }
        // Closures as function pointers: each calls its closure, with no
        // captures.
        for (it, thunk) in &cx.fnptr {
            let Some(closure) = cx.fns.get(it) else { continue };
            fnptr_thunk(&mut m, thunk, closure);
        }
        for (name, it) in &named {
            if let Item::DropGlue(t) = *it {
                let Some(decl) = cx.fns.get(it).cloned() else { continue };
                functions += 1;
                titles.insert(decl.g, format!("drop glue of {}", tcx.display(r, t)));
                let got = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| drop::lower_drop_glue(&cx, &mut m, t, &decl)));
                let why = match got {
                    Ok(Ok(())) => continue,
                    Ok(Err(e)) => e,
                    Err(_) => "internal error".into(),
                };
                body::trap_body(&mut m, &decl);
                failures.push(Failure { item: *it, name: name.clone(), why });
                continue;
            }
            let Item::Fn(d, l) = *it else { continue };
            let Some(decl) = cx.fns.get(it).cloned() else { continue };
            functions += 1;
            titles.insert(decl.g, item_title(&cx, d, l));
            lower_item(&cx, &mut m, *it, &decl, name, false, &mut notes, &mut partial, &mut failures);
        }
        // Statics whose values the evaluator does not know: a function
        // each that computes the value, run here (`image.rs`) for the
        // static's initialiser, or if that cannot be, before `main`.
        let mut ctors = Vec::new();
        for (it, d, g, name) in static_inits {
            let fname = format!("{name}.init");
            let got = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| body::lower_static_init(&cx, &mut m, d, g, &fname)));
            match got {
                Ok(Ok(f)) => {
                    functions += 1;
                    titles.insert(f, format!("the value of static {}", r.def_path(d)));
                    let size = tcx.type_of(d).and_then(|t| cx.layout(t).ok()).map(|l| l.size);
                    let img = size.and_then(|n| std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| image::run(&m, f, g, n))).ok().flatten());
                    match img {
                        Some(img) => {
                            let c = image::constant(&mut m, &img);
                            let ty = m.consts.get(c).ty;
                            if let epitome_ir::module::GlobalKind::Variable(v) = &mut m.globals[g.index()].kind {
                                v.ty = ty;
                                v.init = Some(c);
                            }
                        }
                        None => ctors.push(f),
                    }
                }
                Ok(Err(e)) => failures.push(Failure { item: it, name: fname, why: e }),
                Err(_) => failures.push(Failure { item: it, name: fname, why: "internal error".into() }),
            }
        }
        if !ctors.is_empty() {
            global_ctors(&mut m, &ctors);
        }
        default_allocator(&cx, &mut m);
        // With `std`: the C `main`, which runs the program's `main`
        // through `std`'s `lang_start`.
        if let Some(e) = entry_point(&cx, &mut m, mono) {
            failures.push(Failure { item: e.0, name: "main".into(), why: e.1 });
        }
        failures.sort_by(|a, b| a.name.cmp(&b.name));
        partial.sort_by(|a, b| a.name.cmp(&b.name));
        // Each function's target features (once its body is in: a body
        // replaces its declaration); a closure has its function's.
        for (&it, decl) in &cx.fns {
            if let Item::Fn(d, _) | Item::Closure(_, d, _) = it
                && let Some(fs) = cx.target_features(d, target)
            {
                add_target_features(&mut m, decl.g, fs);
            }
        }
        // Every function defined: the target's CPU.
        if let Some(cpu) = target.cpu {
            for g in &mut m.globals {
                if let epitome_ir::module::GlobalKind::Function(f) = &mut g.kind
                    && f.has_body
                    && !f.fn_attrs.iter().any(|a| matches!(a, epitome_ir::module::Attr::Str { key, .. } if key.as_slice() == b"target-cpu"))
                {
                    f.fn_attrs.push(epitome_ir::module::Attr::Str { key: b"target-cpu".to_vec(), value: Some(cpu.as_bytes().to_vec()) });
                }
            }
        }
        let decls = cx.fns.iter().map(|(&it, d)| (it, d.g)).collect();
        Lowered { module: m, functions, failures, partial, notes, titles, decls }
    }
}

/// Defines `shim`, closure `closure`'s `call_once` in a vtable: it calls
/// the closure with its own arguments, then drops the captures (through
/// `drop`, the closure's drop glue, if it does not drop them itself).
fn once_shim(m: &mut Module, shim: &Decl, closure: &Decl, drop: Option<&Decl>) {
    let (ret, params) = Cx::ir_fn_type(m, &shim.abi);
    let mut spec = IrFunction::builder(&shim.name).returns(ret).linkage(Linkage::Internal);
    for &p in &params {
        spec = spec.param("", p);
    }
    let mut b = spec.begin(m);
    b.block("start");
    let args: Vec<_> = (0..params.len()).map(|i| b.param(i)).collect();
    let r = b.call(closure.g, &args);
    if let Some(d) = drop {
        // The captures' address: the first argument after the result's.
        let env = args[(shim.abi.ret == Repr::Mem) as usize];
        b.call(d.g, &[env]);
    }
    match r {
        Some(v) if matches!(shim.abi.ret, Repr::Imm(_)) => b.ret(v),
        _ => b.ret_void(),
    }
    let _ = b.finish();
}

/// Defines `thunk`, a closure that captures nothing as a function: it calls
/// the closure (`closure`) with no captures (a null pointer for them) and
/// its own arguments.
fn fnptr_thunk(m: &mut Module, thunk: &Decl, closure: &Decl) {
    let (ret, params) = Cx::ir_fn_type(m, &thunk.abi);
    let mut spec = IrFunction::builder(&thunk.name).returns(ret).linkage(Linkage::Internal);
    for &p in &params {
        spec = spec.param("", p);
    }
    let mut b = spec.begin(m);
    b.block("start");
    let ptr = b.module().types.ptr(0);
    let null = b.module().const_zero(ptr);
    let mut args: Vec<_> = (0..params.len()).map(|i| b.param(i)).collect();
    // After the result's address, if it is returned in memory.
    let at = (thunk.abi.ret == Repr::Mem) as usize;
    if closure.abi.args.first().is_some_and(|r| *r != Repr::Zst) {
        args.insert(at, null);
    }
    match b.call(closure.g, &args) {
        Some(v) if matches!(thunk.abi.ret, Repr::Imm(_)) => b.ret(v),
        _ => b.ret_void(),
    }
    let _ = b.finish();
}

/// Lowers function (or closure) `it` into `decl`: its notes, or a failure
/// (and a body that traps). Gives whether (a closure not owning its
/// captures) it moved out of one, and whether it called a closure whose
/// kind was not known yet.
#[allow(clippy::too_many_arguments)]
fn lower_item(
    cx: &Cx,
    m: &mut Module,
    it: Item,
    decl: &Decl,
    name: &str,
    owns_env: bool,
    notes: &mut FastMap<GlobalId, Vec<(epitome_ir::module::BlockId, usize, String)>>,
    partial: &mut Vec<Failure>,
    failures: &mut Vec<Failure>,
) -> (bool, bool) {
    let got = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| body::lower_fn(cx, m, it, decl, owns_env)));
    let why = match got {
        Ok(Ok((n, t, moved, unknown))) => {
            notes.insert(decl.g, n);
            if !t.is_empty() {
                partial.push(Failure { item: it, name: name.to_string(), why: format!("traps where it uses {}", t.join(", ")) });
            }
            return (moved, unknown);
        }
        Ok(Err(e)) => e,
        Err(p) => {
            let msg = p.downcast_ref::<String>().cloned().or_else(|| p.downcast_ref::<&str>().map(|s| s.to_string()));
            format!("internal error: {}", msg.unwrap_or_default())
        }
    };
    body::trap_body(m, decl);
    failures.push(Failure { item: it, name: name.to_string(), why });
    (false, false)
}

/// The functions `alloc` calls to allocate, and `std`'s own allocator's
/// functions that they call when no `#[global_allocator]` defines them.
pub const DEFAULT_ALLOCATOR: [(&str, &str); 4] = [
    ("__rust_alloc", "__rdl_alloc"),
    ("__rust_dealloc", "__rdl_dealloc"),
    ("__rust_realloc", "__rdl_realloc"),
    ("__rust_alloc_zeroed", "__rdl_alloc_zeroed"),
];

/// The allocator shim with no `#[global_allocator]` (as rustc makes it):
/// each of `alloc`'s allocation functions that is declared and not
/// defined calls `std`'s (`__rdl_alloc` and the rest), passing its
/// arguments on; `__rust_no_alloc_shim_is_unstable_v2` does nothing, and
/// `__rust_alloc_error_handler` aborts.
fn default_allocator(cx: &Cx, m: &mut Module) {
    let rdl = |name: &str| {
        cx.fns.iter().find_map(|(it, d)| match *it {
            Item::Fn(f, _) if cx.r.defs[f.0 as usize].name == name => Some(d.g),
            _ => None,
        })
    };
    let declared = |m: &Module, name: &str| -> Option<(TypeId, Vec<TypeId>)> {
        let g = m.find_global(name)?;
        match &m.global(g).kind {
            epitome_ir::module::GlobalKind::Function(f) if !f.has_body => {
                let epitome_ir::types::Type::Function { ret, params, .. } = m.types.get(f.ty).clone() else { return None };
                Some((ret, params.to_vec()))
            }
            _ => None,
        }
    };
    if rdl(DEFAULT_ALLOCATOR[0].1).is_none() {
        return;
    }
    for (shim, target) in DEFAULT_ALLOCATOR {
        let (Some((ret, params)), Some(t)) = (declared(m, shim), rdl(target)) else { continue };
        let mut spec = IrFunction::builder(shim).returns(ret);
        for &p in &params {
            spec = spec.param("", p);
        }
        let mut b = spec.begin(m);
        b.block("start");
        let args: Vec<_> = (0..params.len()).map(|i| b.param(i)).collect();
        match b.call(t, &args) {
            Some(v) => b.ret(v),
            None => b.ret_void(),
        }
        let _ = b.finish();
    }
    if let Some((ret, params)) = declared(m, "__rust_no_alloc_shim_is_unstable_v2") {
        let mut spec = IrFunction::builder("__rust_no_alloc_shim_is_unstable_v2").returns(ret);
        for &p in &params {
            spec = spec.param("", p);
        }
        let mut b = spec.begin(m);
        b.block("start");
        b.ret_void();
        let _ = b.finish();
    }
    if let Some((ret, params)) = declared(m, "__rust_alloc_error_handler") {
        let mut spec = IrFunction::builder("__rust_alloc_error_handler").returns(ret);
        for &p in &params {
            spec = spec.param("", p);
        }
        let mut b = spec.begin(m);
        b.block("start");
        let void = b.module().types.void();
        let trap = b.module().declare("llvm.trap", void, &[], false);
        b.call(trap, &[]);
        b.unreachable();
        let _ = b.finish();
    }
}

/// The C `main(argc, argv)` of a program with `std`: `lang_start::<T>`
/// (`std::rt`'s, the `start` lang item) of the program's `main`, `argc`,
/// `argv` and the default `SIGPIPE` handling, its result the exit status.
/// Nothing if there is no such `main` or no `std`; a failure if it cannot
/// be made.
fn entry_point(cx: &Cx, m: &mut Module, mono: &MonoItems) -> Option<(Item, String)> {
    let tcx = cx.tcx;
    let start = *tcx.lang.get("start")?;
    let empty = tcx.list(Vec::new());
    // The program's `main`: the last crate's.
    let main = mono
        .items
        .iter()
        .filter_map(|&it| match it {
            Item::Fn(d, l) if l == empty && cx.r.defs[d.0 as usize].name == "main" && cx.r.defs[d.0 as usize].kind == DefKind::Fn => Some(d),
            _ => None,
        })
        .max_by_key(|&d| cx.r.defs[d.0 as usize].krate)?;
    let main_it = Item::Fn(main, empty);
    let (_, _, ast, node) = cx.def_ast(main);
    if crate::session::has_attribute(ast, node, "no_mangle") {
        return None;
    }
    let out = tcx.sig_of(tcx.fn_sig[main.0 as usize]?).output;
    let start_it = Item::Fn(start, tcx.list(vec![out]));
    let (Some(md), Some(sd)) = (cx.fns.get(&main_it), cx.fns.get(&start_it)) else {
        return Some((main_it, "`main` or `lang_start` was not declared".into()));
    };
    let ints: Vec<TypeId> = sd.abi.args.iter().map(|r| match r {
        Repr::Imm(t) => Some(*t),
        _ => None,
    }).collect::<Option<Vec<_>>>().unwrap_or_default();
    if ints.len() != 4 || !matches!(sd.abi.ret, Repr::Imm(_)) {
        return Some((start_it, "`lang_start` has an unexpected signature".into()));
    }
    let i32t = m.types.int(32);
    let ptr = m.types.ptr(0);
    let mut b = IrFunction::builder("main").returns(i32t).param("argc", i32t).param("argv", ptr).begin(m);
    b.block("start");
    let f = b.module().global_value(md.g);
    let argc = b.param(0);
    let argc = b.sext(argc, ints[1]);
    let argv = b.param(1);
    let sigpipe = b.const_int(ints[3], 0);
    let r = b.call(sd.g, &[f, argc, argv, sigpipe]).expect("`lang_start` gives a value");
    let r = b.trunc(r, i32t);
    b.ret(r);
    b.finish().err().map(|e| (start_it, e.to_string()))
}

/// Each Rust target feature of x86-64, and the LLVM features rustc turns
/// on for it (`x86_64_features.txt`, from `gen_target_features.py`).
fn x86_64_features() -> &'static FastMap<String, Vec<String>> {
    static TABLE: std::sync::OnceLock<FastMap<String, Vec<String>>> = std::sync::OnceLock::new();
    TABLE.get_or_init(|| {
        include_str!("x86_64_features.txt")
            .lines()
            .filter(|l| !l.starts_with('#'))
            .filter_map(|l| l.split_once(' '))
            .map(|(rust, llvm)| (rust.to_string(), llvm.split(',').map(str::to_string).collect()))
            .collect()
    })
}

/// Gives function `g` the attribute `"target-features"="fs"`.
fn add_target_features(m: &mut Module, g: GlobalId, fs: String) {
    if let epitome_ir::module::GlobalKind::Function(f) = &mut m.globals[g.index()].kind {
        f.fn_attrs.push(epitome_ir::module::Attr::Str { key: b"target-features".to_vec(), value: Some(fs.into_bytes()) });
    }
}

/// `@llvm.global_ctors`: each of `fs` run before `main`, in order.
fn global_ctors(m: &mut Module, fs: &[GlobalId]) {
    use epitome_ir::module::{ConstKind, Constant, Value};
    let i32t = m.types.int(32);
    let ptr = m.types.ptr(0);
    let entry = m.types.struct_of(&[i32t, ptr, ptr]);
    let mut entries = Vec::new();
    for &f in fs {
        let prio = Constant::int(&m.types, i32t, 65535);
        let prio = m.consts.intern(prio);
        let Value::Const(fc) = m.global_value(f) else { unreachable!("a global's address is a constant") };
        let null = m.consts.intern(Constant { ty: ptr, kind: ConstKind::Null });
        entries.push(m.consts.intern(Constant { ty: entry, kind: ConstKind::Struct(vec![prio, fc, null]) }));
    }
    let ty = m.types.array(entries.len() as u64, entry);
    let init = Value::Const(m.consts.intern(Constant { ty, kind: ConstKind::Array(entries) }));
    epitome_ir::module::Global::builder("llvm.global_ctors").init(init).linkage(Linkage::Appending).build(m);
}

/// `fn path<args>`, with the source location.
fn item_title(cx: &Cx, d: DefId, l: List) -> String {
    let a = cx.tcx.tys(l);
    let args = if a.is_empty() {
        String::new()
    } else {
        let s: Vec<String> = a.into_iter().map(|t| cx.tcx.display(cx.r, t)).collect();
        format!("<{}>", s.join(", "))
    };
    let (k, f, _, _) = cx.def_ast(d);
    let file = &cx.crates[k as usize].files[f as usize].source.name;
    let file = file.rsplit('/').next().unwrap_or(file);
    format!("fn {}{args} ({file})", crate::mono::item_path(cx.r, cx.tcx, d))
}

/// Whether `t` is a type lowering treats as a primitive for operators:
/// numbers, `bool`, `char` and raw pointers.
pub(crate) fn is_prim(tcx: &TyCtxt, t: Ty) -> bool {
    matches!(tcx.kind(t), TyKind::Int(_) | TyKind::Uint(_) | TyKind::Float(_) | TyKind::Bool | TyKind::Char | TyKind::Ptr(..) | TyKind::FnPtr(_))
}

#[cfg(test)]
mod tests {
    use crate::mono::Item;
    use crate::session::{CrateRoot, Options, Session};

    /// `core` from the toolchain's rust-src, if it is installed.
    fn core_src() -> Option<std::path::PathBuf> {
        let out = std::process::Command::new("rustc").args(["--print", "sysroot"]).output().ok()?;
        let root = std::path::PathBuf::from(String::from_utf8(out.stdout).ok()?.trim());
        let core = root.join("lib/rustlib/src/rust/library/core/src/lib.rs");
        core.exists().then_some(core)
    }

    /// Each program of `tests/run` lowers with `core`: none of its own
    /// functions fails, and the module prints as IR that parses back.
    /// `core` is configured for x86-64 Linux, the target lowered for,
    /// whatever the host, so `core::arch::x86_64` is there on any machine.
    #[test]
    fn run_programs_lower() {
        let Some(core) = core_src() else {
            eprintln!("no rust-src: skipped");
            return;
        };
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/run");
        let mut programs: Vec<_> = std::fs::read_dir(&dir).unwrap().flatten().map(|e| e.path()).filter(|p| p.extension().is_some_and(|x| x == "rs")).collect();
        programs.sort();
        assert!(programs.len() >= 5, "programs: {programs:?}");
        for file in programs {
            let mut cfg = crate::cfg::CfgSet::x86_64_linux();
            let mut options = Options { threads: 4, ..Default::default() };
            if !std::fs::read_to_string(&file).unwrap().contains("#![no_std]") {
                for (n, v) in crate::sysroot::std_cfg() {
                    match v {
                        Some(v) => cfg.set_value(n, v),
                        None => cfg.set(n),
                    };
                }
                options.env = crate::sysroot::std_env("x86_64");
            }
            options.cfg = Some(cfg);
            let session = Session::new(options);
            let text = std::fs::read_to_string(&file).unwrap();
            let with_std = !text.contains("#![no_std]");
            let mut roots = if with_std {
                crate::sysroot::std_crates(core.ancestors().nth(3).unwrap()).unwrap()
            } else {
                vec![CrateRoot::new("core", &core)]
            };
            let alloc = core.ancestors().nth(3).unwrap().join("alloc/src/lib.rs");
            if !with_std && text.contains("extern crate alloc;") {
                roots.push(CrateRoot::new("alloc", &alloc));
            }
            roots.push(CrateRoot::new("program", &file));
            let program = roots.len() as u32 - 1;
            let mut crates = session.parse_crates(roots);
            session.expand(&mut crates);
            let r = session.resolve(&crates);
            let tcx = session.collect_types(&crates, &r);
            let index = crate::typeck::Index::new(&tcx, &r);
            let res = session.typeck_with(&crates, &r, &tcx, &index);
            let roots = crate::mono::roots(&r, &tcx, &crates, program, false);
            let mono = session.collect_mono_items(&crates, &r, &tcx, &index, &res, roots);
            let lowered = session.lower(&crates, &r, &tcx, &index, &res, &mono, super::Target::x86_64_linux());
            let own: Vec<String> = lowered
                .failures
                .iter()
                .filter(|f| {
                    let d = match f.item {
                        Item::Fn(d, _) | Item::Closure(_, d, _) | Item::Static(d) | Item::Foreign(d) | Item::Intrinsic(d, _) | Item::Const(d, _) => d,
                        Item::VTable(..) => return false,
                        // Drop glue the program needs, of whatever type.
                        Item::DropGlue(_) => return true,
                    };
                    r.defs[d.0 as usize].krate == program
                })
                .map(|f| format!("{}: {}", f.name, f.why))
                .collect();
            assert!(own.is_empty(), "{}: not lowered: {own:#?}", file.display());
            let text = lowered.module.to_string();
            assert!(text.contains("define i32 @main("), "{}: no main", file.display());
            if let Err(e) = epitome_ir::parse_module(&text) {
                panic!("{}: the IR does not parse: {e}", file.display());
            }
            // The dump quotes the source.
            assert!(lowered.dump().contains("; fn "), "{}: no headings in the dump", file.display());
        }
    }
}
