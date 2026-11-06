//! Runs of stores become `memset`: part of LLVM's `memcpyopt`.
//!
//! Within a block, constant stores of the same byte value (`0`, `-1`, any
//! integer whose bytes are all one byte) to contiguous addresses from one
//! base are one `llvm.memset`, when they cover at least `MIN_BYTES` in at
//! least `MIN_STORES` stores. Code generation writes a small memset back as
//! the widest stores the alignment allows (`codegen/src/prepare.rs`), so
//! eight `i8` zeros become one 8-byte store.
//!
//! The order of writes is kept. A run is looked for between instructions
//! that touch memory other than such stores; stores to the same base that
//! overlap leave that base alone (their order would matter); and a store
//! between a run's first and last that is not one of them must not alias
//! the run's bytes. The memset goes where the run's first store was, with
//! that store's alignment.

use epitome_ir::module::*;
use epitome_ir::types::Type;

use crate::aa::{AliasResult, BasicAa, Location};
use crate::ctx::Shared;

/// The fewest bytes, and stores, a run must have.
const MIN_BYTES: u64 = 16;
const MIN_STORES: usize = 3;

#[derive(Clone, Copy, Debug, Default)]
pub struct Stats {
    /// Memsets made, and the stores they replaced.
    pub memsets: usize,
    pub stores: usize,
}

/// A store this pass can merge.
#[derive(Clone, Copy, Debug)]
struct Piece {
    inst: InstId,
    /// Where in the block.
    at: usize,
    base: Value,
    offset: i64,
    size: u64,
    byte: u8,
    align: Option<Align>,
}

pub fn run(sh: &Shared, f: &mut Function) -> Stats {
    let mut stats = Stats::default();
    if !f.has_body || sh.intrinsics.memset.is_none() {
        return stats;
    }
    let aa = BasicAa::new(sh, f);
    let mut runs: Vec<(BlockId, Vec<Piece>)> = Vec::new();
    for &b in &f.layout {
        let mut window: Vec<Piece> = Vec::new();
        let insts = f.blocks[b.index()].insts.clone();
        for (at, &i) in insts.iter().enumerate() {
            if let Some(p) = piece(sh, f, i, at) {
                window.push(p);
                continue;
            }
            let e = aa.effects(&f.insts[i.index()]);
            if e.mods || e.refs || f.insts[i.index()].op.is_terminator() {
                runs.extend(find_runs(&aa, f, &window).into_iter().map(|r| (b, r)));
                window.clear();
            }
        }
        runs.extend(find_runs(&aa, f, &window).into_iter().map(|r| (b, r)));
    }
    drop(aa);
    for (b, run) in runs {
        if !epitome_ir::chop::allow(|| {
            format!("memcpyopt: {} stores to {} as a memset", run.len(), epitome_ir::printer::inst_name(f, run[0].inst))
        }) {
            continue;
        }
        stats.memsets += 1;
        stats.stores += run.len();
        make_memset(sh, f, b, &run);
    }
    stats
}

/// A store of a constant whose bytes are all one value, to a base plus a
/// constant offset.
fn piece(sh: &Shared, f: &Function, i: InstId, at: usize) -> Option<Piece> {
    let Op::Store { volatile: false, atomic: None, value: Value::Const(c), ptr, align } = f.insts[i.index()].op else {
        return None;
    };
    let k = sh.consts().get(c).clone();
    let bits = match sh.types().get(k.ty) {
        Type::Int(b) => *b,
        Type::Ptr(0) => 64,
        _ => return None,
    };
    if bits % 8 != 0 || bits > 64 {
        return None;
    }
    let value: u64 = match &k.kind {
        ConstKind::Int(v) => v.low_u64(),
        ConstKind::Zero | ConstKind::Null => 0,
        _ => return None,
    };
    let byte = (value & 0xff) as u8;
    let n = bits / 8;
    if (0..n).any(|j| ((value >> (8 * j)) & 0xff) as u8 != byte) {
        return None;
    }
    let (base, offset) = split_address(sh, f, ptr)?;
    Some(Piece { inst: i, at, base, offset, size: n as u64, byte, align })
}

/// A pointer as a base and a constant offset, through `getelementptr`s
/// with constant indices.
fn split_address(sh: &Shared, f: &Function, mut v: Value) -> Option<(Value, i64)> {
    let mut offset = 0i64;
    for _ in 0..16 {
        let Value::Local(l) = v else { break };
        let LocalDef::Inst(d) = f.locals[l.index()].def else { break };
        let Op::Gep { source_ty, ptr, indices, .. } = &f.insts[d.index()].op else { break };
        let Some(o) = crate::combine::constload::gep_offset(sh, *source_ty, indices) else { break };
        offset = offset.checked_add(o)?;
        v = *ptr;
    }
    Some((v, offset))
}

/// The runs worth a memset among the stores of a window.
fn find_runs(aa: &BasicAa, f: &Function, window: &[Piece]) -> Vec<Vec<Piece>> {
    let mut out = Vec::new();
    // The stores of each base, bases in the order they first appear.
    let mut index: crate::fx::FxHashMap<Value, usize> = Default::default();
    let mut groups: Vec<Vec<Piece>> = Vec::new();
    for p in window {
        let k = *index.entry(p.base).or_insert_with(|| {
            groups.push(Vec::new());
            groups.len() - 1
        });
        groups[k].push(*p);
    }
    for mut mine in groups {
        mine.sort_by_key(|p| (p.offset, p.at));
        // Overlapping stores: their order matters; leave the base alone.
        if mine.windows(2).any(|w| w[0].offset + w[0].size as i64 > w[1].offset) {
            continue;
        }
        let mut k = 0;
        while k < mine.len() {
            let mut j = k + 1;
            while j < mine.len()
                && mine[j].byte == mine[k].byte
                && mine[j].offset == mine[j - 1].offset + mine[j - 1].size as i64
            {
                j += 1;
            }
            let run = &mine[k..j];
            let bytes: u64 = run.iter().map(|p| p.size).sum();
            if run.len() >= MIN_STORES && bytes >= MIN_BYTES && no_alias_between(aa, f, window, run, bytes) {
                out.push(run.to_vec());
            }
            k = j;
        }
    }
    out
}

/// Whether every other store between the run's first and last leaves the
/// run's bytes alone.
fn no_alias_between(aa: &BasicAa, f: &Function, window: &[Piece], run: &[Piece], bytes: u64) -> bool {
    let first = run.iter().map(|p| p.at).min().unwrap();
    let last = run.iter().map(|p| p.at).max().unwrap();
    let start = run.iter().min_by_key(|p| p.offset).unwrap();
    let Op::Store { ptr: start_ptr, .. } = f.insts[start.inst.index()].op else { return false };
    let region = Location { ptr: start_ptr, size: Some(bytes) };
    // The window is in block order: only the stores within the run's span.
    let lo = window.partition_point(|p| p.at <= first);
    let hi = window.partition_point(|p| p.at < last);
    let between = &window[lo..hi.max(lo)];
    if between.len() > run.len() + 64 {
        return false;
    }
    let mine: crate::fx::FxHashSet<InstId> = run.iter().map(|r| r.inst).collect();
    between.iter().filter(|p| !mine.contains(&p.inst)).all(|p| {
        let Op::Store { ptr, .. } = f.insts[p.inst.index()].op else { return false };
        aa.alias(f, region, Location { ptr, size: Some(p.size) }) == AliasResult::NoAlias
    })
}

fn make_memset(sh: &Shared, f: &mut Function, b: BlockId, run: &[Piece]) {
    let start = *run.iter().min_by_key(|p| p.offset).unwrap();
    let bytes: u64 = run.iter().map(|p| p.size).sum();
    let first = run.iter().map(|p| p.at).min().unwrap();
    let at_inst = f.blocks[b.index()].insts.iter().position(|&x| x == run.iter().find(|p| p.at == first).unwrap().inst);
    let mut at = at_inst.unwrap();
    let i8t = sh.intern_type(Type::Int(8));
    let ptr_ty = sh.intern_type(Type::Ptr(0));
    let address = if start.offset == 0 {
        start.base
    } else {
        let g = f.add_inst(
            Op::Gep {
                flags: GepFlags::default(),
                source_ty: i8t,
                ptr: start.base,
                indices: vec![sh.int(64, start.offset)].into(),
            },
            Some((None, ptr_ty)),
        );
        f.blocks[b.index()].insts.insert(at, g);
        at += 1;
        f.result(g)
    };
    let memset = sh.global_ref(sh.intrinsics.memset.unwrap());
    // (During a phase, declarations are read from where they are published.)
    let fn_ty = sh.published(sh.intrinsics.memset.unwrap()).expect("llvm.memset is declared").ty;
    let mut call = crate::inline::plain_call(
        fn_ty,
        memset,
        vec![address, sh.int(8, start.byte as i64), sh.int(64, bytes as i64), {
            let i1 = sh.intern_type(Type::Int(1));
            Value::Const(
                sh.intern_const(Constant { ty: i1, kind: ConstKind::Int(epitome_ir::apint::ApInt::from_u64(1, 0)) }),
            )
        }],
    );
    if let Some(a) = start.align {
        call.args[0].attrs.push(Attr::Int { name: "align".into(), value: a.bytes(), style: IntStyle::Space });
    }
    let c = f.add_inst(Op::Call(Box::new(call)), None);
    f.blocks[b.index()].insts.insert(at, c);
    let gone: crate::fx::FxHashSet<InstId> = run.iter().map(|p| p.inst).collect();
    f.blocks[b.index()].insts.retain(|x| !gone.contains(x));
}
