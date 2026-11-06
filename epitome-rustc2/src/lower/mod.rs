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
//! Panics become a trap (`panic=abort`, with no message: `core::fmt`
//! comes later), and so does an index out of bounds or a division by zero.
//! Arithmetic wraps, as in a release build.
//!
//! A function whose body uses something lowering does not support yet
//! becomes a body that traps, and the reason is kept: `--lower-stats`
//! counts them.

mod body;
mod intrinsic;
mod pat;

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
}

impl Target {
    pub fn x86_64_linux() -> Target {
        Target {
            triple: "x86_64-unknown-linux-gnu",
            data_layout: "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-i128:128-f80:128-n8:16:32:64-S128",
            ptr_size: 8,
        }
    }

    pub fn aarch64_darwin() -> Target {
        Target { triple: "arm64-apple-macosx14.0.0", data_layout: "e-m:o-i64:64-i128:128-n32:64-S128-Fn32", ptr_size: 8 }
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
}

impl Lowered {
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
}

impl<'a, 's> Cx<'a, 's> {
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
                Prim::Float { size: 4 } => m.types.float(),
                Prim::Float { size: 8 } => m.types.double(),
                Prim::Float { .. } => return Err("f16 and f128 are not supported".into()),
                Prim::Pointer => m.types.ptr(0),
            }),
            _ => Repr::Mem,
        })
    }

    pub fn abi(&self, m: &mut Module, inputs: Vec<Ty>, output: Ty, variadic: bool) -> Result<FnAbi, String> {
        let args = inputs.iter().map(|&t| self.repr(m, t)).collect::<Result<Vec<_>, _>>()?;
        let ret = self.repr(m, output)?;
        Ok(FnAbi { inputs, output, args, ret, variadic })
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
                out.push_str(&format!("; {:?}]", self.lcx.eval_const(c, &Default::default())));
            }
            _ => out.push_str(&tcx.display(self.r, t)),
        }
    }

    /// The node and file of a definition.
    pub fn def_ast(&self, d: DefId) -> (u32, u32, &'a crate::ast::Ast<'s>, crate::ast::NodeId) {
        let def = &self.r.defs[d.0 as usize];
        (def.krate, def.file, &self.crates[def.krate as usize].files[def.file as usize].ast, def.node)
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
        if crate::session::has_attribute(ast, node, "no_mangle") {
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
        let mut cx = Cx { tcx, r, crates, typeck, index, lcx, fns: FastMap::default(), statics: FastMap::default() };
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
                _ => {}
            }
        }
        named.sort_by(|a, b| a.0.cmp(&b.0));
        named.dedup_by(|a, b| a.0 == b.0);
        for (name, it) in &named {
            match *it {
                Item::Fn(d, _) | Item::Foreign(d) => {
                    let l = if let Item::Fn(_, l) = *it { l } else { tcx.list(Vec::new()) };
                    let args = tcx.tys(l);
                    let abi = cx.signature(&mut solver, d, &args).and_then(|(i, o, _, v)| cx.abi(&mut m, i, o, v));
                    let abi = match abi {
                        Ok(a) => a,
                        Err(e) => {
                            failures.push(Failure { item: *it, name: name.clone(), why: e });
                            continue;
                        }
                    };
                    let (ret, params) = Cx::ir_fn_type(&mut m, &abi);
                    let foreign = matches!(it, Item::Foreign(_));
                    let (_, _, ast, node) = cx.def_ast(d);
                    let exported = foreign
                        || crate::session::has_attribute(ast, node, "no_mangle")
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
                }
                Item::Static(d) => match body::define_static(&cx, &mut m, d, name) {
                    Ok(g) => {
                        cx.statics.insert(d, g);
                    }
                    Err(e) => failures.push(Failure { item: *it, name: name.clone(), why: e }),
                },
                _ => {}
            }
        }

        let mut notes = FastMap::default();
        let mut titles = FastMap::default();
        let mut functions = 0;
        let mut partial = Vec::new();
        for (name, it) in &named {
            let (d, l) = match *it {
                Item::Fn(d, l) | Item::Closure(_, d, l) => (d, l),
                _ => continue,
            };
            let Some(decl) = cx.fns.get(it).cloned() else { continue };
            functions += 1;
            let mut title = item_title(&cx, d, l);
            if let Item::Closure(c, ..) = *it {
                title = format!("closure at node {} in {title}", tcx.closure_at(c).2.0);
            }
            titles.insert(decl.g, title);
            let got = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| body::lower_fn(&cx, &mut m, *it, &decl)));
            let why = match got {
                Ok(Ok((n, t))) => {
                    notes.insert(decl.g, n);
                    if !t.is_empty() {
                        partial.push(Failure { item: *it, name: name.clone(), why: format!("traps where it uses {}", t.join(", ")) });
                    }
                    continue;
                }
                Ok(Err(e)) => e,
                Err(p) => {
                    let msg = p.downcast_ref::<String>().cloned().or_else(|| p.downcast_ref::<&str>().map(|s| s.to_string()));
                    format!("internal error: {}", msg.unwrap_or_default())
                }
            };
            body::trap_body(&mut m, &decl);
            failures.push(Failure { item: *it, name: name.clone(), why });
        }
        failures.sort_by(|a, b| a.name.cmp(&b.name));
        partial.sort_by(|a, b| a.name.cmp(&b.name));
        Lowered { module: m, functions, failures, partial, notes, titles }
    }
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
            let session = Session::new(Options { threads: 4, cfg: Some(crate::cfg::CfgSet::host()), ..Default::default() });
            let mut crates = session.parse_crates(vec![CrateRoot::new("core", &core), CrateRoot::new("program", &file)]);
            session.expand(&mut crates);
            let r = session.resolve(&crates);
            let tcx = session.collect_types(&crates, &r);
            let index = crate::typeck::Index::new(&tcx, &r);
            let res = session.typeck_with(&crates, &r, &tcx, &index);
            let roots = crate::mono::roots(&r, &tcx, 1);
            let mono = session.collect_mono_items(&crates, &r, &tcx, &index, &res, roots);
            let lowered = session.lower(&crates, &r, &tcx, &index, &res, &mono, super::Target::x86_64_linux());
            let own: Vec<String> = lowered
                .failures
                .iter()
                .filter(|f| {
                    let d = match f.item {
                        Item::Fn(d, _) | Item::Closure(_, d, _) | Item::Static(d) | Item::Foreign(d) | Item::Intrinsic(d, _) | Item::Const(d, _) => d,
                        Item::VTable(..) => return false,
                    };
                    r.defs[d.0 as usize].krate == 1
                })
                .map(|f| format!("{}: {}", f.name, f.why))
                .collect();
            assert!(own.is_empty(), "{}: not lowered: {own:#?}", file.display());
            let text = lowered.module.to_string();
            assert!(text.contains("define i32 @main()"), "{}: no main", file.display());
            if let Err(e) = epitome_ir::parse_module(&text) {
                panic!("{}: the IR does not parse: {e}", file.display());
            }
            // The dump quotes the source.
            assert!(lowered.dump().contains("; fn "), "{}: no headings in the dump", file.display());
        }
    }
}
