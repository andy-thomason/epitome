//! Code for other machines in the same program. A function marked
//! `#[epitome::target("riscv64")]` (a kernel) is compiled for that machine,
//! with everything it calls, in a pass of its own over the same source:
//! `core` and `alloc` from rust-src with the machine's `cfg`, then the
//! program's crate again. Monomorphisation starts from the kernels, so
//! code only the host uses (`std`'s, `println!`) is never type checked or
//! lowered there, and may name what the machine does not have.
//!
//! A kernel is named by its path on both sides (`kernel_symbol`), and is
//! external in the machine's module.

use crate::ast::{Ast, Kind, NodeId};
use crate::cfg::CfgSet;
use crate::lexer::Edition;
use crate::lower::{Lowered, Target};
use crate::mono::Item;
use crate::resolve::{DefId, DefKind, Resolver};
use crate::session::{CrateRoot, Options, ParsedCrate, Session};
use std::path::Path;

/// A machine kernels may be compiled for.
#[derive(Clone, Debug)]
pub struct Machine {
    /// Its name in `#[epitome::target(..)]`.
    pub name: &'static str,
    pub cfg: CfgSet,
    pub target: Target,
}

/// The machine called `name`, if there is one.
pub fn machine(name: &str) -> Option<Machine> {
    match name {
        "riscv64" => Some(Machine { name: "riscv64", cfg: CfgSet::riscv64_none(), target: Target::riscv64_none() }),
        _ => None,
    }
}

/// The machine `#[epitome::target("name")]` on item `node` names.
pub fn kernel_target(ast: &Ast, node: NodeId) -> Option<String> {
    let (_, a) = crate::session::attributes(ast, node).find(|(n, _)| n == "epitome::target")?;
    let tt = ast.child(a, Kind::TokenTree)?;
    let lit = ast.tokens_of(tt).iter().find(|t| t.0.starts_with('"'))?;
    Some(crate::cfg::string_value(lit.0).to_string())
}

/// A kernel's symbol, from its path (`prog::add` is `kernel.prog.add`).
pub fn kernel_symbol(path: &str) -> String {
    let safe: String = path.chars().map(|c| if c.is_ascii_alphanumeric() || c == '_' { c } else { '.' }).collect();
    format!("kernel.{}", safe.replace("..", "."))
}

/// The kernels of crate `k`, with the machine each is for.
pub fn kernels(r: &Resolver, crates: &[ParsedCrate], k: u32) -> Vec<(DefId, String)> {
    (0..r.defs.len() as u32)
        .map(DefId)
        .filter_map(|d| {
            let def = &r.defs[d.0 as usize];
            if def.krate != k || !matches!(def.kind, DefKind::Fn) {
                return None;
            }
            let ast = &crates[def.krate as usize].files[def.file as usize].ast;
            kernel_target(ast, def.node).map(|t| (d, t))
        })
        .collect()
}

/// Compiles the kernels of the crate whose root is `program` for
/// `machine`: lowered into one module, `core`'s and `alloc`'s functions at
/// optimisation level 2 and the program's at `level`. An error if a kernel
/// or what it calls in the program's crate cannot be lowered.
pub fn compile(program: &Path, name: &str, edition: Edition, machine: &Machine, threads: usize, level: &str) -> Result<Lowered, String> {
    let library = crate::sysroot::rust_src().ok_or("no rust-src: `rustup component add rust-src`")?;
    let session = Session::new(Options { threads, cfg: Some(machine.cfg.clone()), ..Default::default() });
    let roots = vec![
        crate::sysroot::library_crate(&library, "core"),
        crate::sysroot::library_crate(&library, "alloc"),
        CrateRoot::new(name, program).edition(edition),
    ];
    let mut crates = session.parse_crates(roots);
    if let Some(c) = crates.iter().find(|c| c.files.is_empty()) {
        let errors: Vec<String> = c.errors.iter().map(|e| e.render_in(&session.sources)).collect();
        return Err(format!("for {}: {}", machine.name, errors.join("")));
    }
    session.expand(&mut crates);
    let r = session.resolve(&crates);
    let tcx = session.collect_types(&crates, &r);
    let index = crate::typeck::Index::new(&tcx, &r);
    let mut res = crate::typeck::TypeckResults::empty(&crates);
    let k = crates.len() as u32 - 1;
    let empty = tcx.list(Vec::new());
    let roots: Vec<Item> = kernels(&r, &crates, k).into_iter().filter(|(_, t)| t == machine.name).map(|(d, _)| Item::Fn(d, empty)).collect();
    if roots.is_empty() {
        return Err(format!("no kernels for {}", machine.name));
    }
    let mono = session.collect_mono_items_lazily(&crates, &r, &tcx, &index, &mut res, roots);
    let mut lowered = session.lower(&crates, &r, &tcx, &index, &res, &mono, machine.target);
    let own: Vec<String> = lowered
        .failures
        .iter()
        .filter(|f| match f.item {
            Item::Fn(d, _) | Item::Closure(_, d, _) | Item::Static(d) | Item::Const(d, _) => r.defs[d.0 as usize].krate == k,
            _ => false,
        })
        .map(|f| format!("could not lower {} for {}: {}", f.name, machine.name, f.why))
        .collect();
    if !own.is_empty() {
        return Err(own.join("\n"));
    }
    lowered.set_opt_levels(&r, |c| Some(if c == k { level.to_string() } else { "2".to_string() }));
    Ok(lowered)
}
