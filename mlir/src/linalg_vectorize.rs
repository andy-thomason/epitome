//! `linalg.generic` on buffers to loops of vectors: as `linalg-to-loops`,
//! but the innermost loop takes `width` elements at a time, with
//! `vector.load`, `vector.store` and the body's operations on
//! `vector<width x f32>`. Upstream's vectorizer turns a whole operation
//! into n-dimensional vectors and leaves unrolling them to the native
//! width to later passes; here the operation becomes loops of vectors of
//! the native width at once, which is where that ends.
//!
//! An operation is vectorized when its innermost loop runs over the last,
//! contiguous dimension of every operand that it indexes (the others are
//! the same for each lane and broadcast), its size is a multiple of
//! `width` (or of half of it, and so on down to 4, then vectors of that
//! many lanes), the elements it loads in vectors are `f32`, and its body is
//! arithmetic that has a vector form. Failing that, another parallel loop
//! that is so may be made the innermost instead (the two swapped): a
//! matrix product's columns rather than its sum, each lane then summing
//! its own element as the scalar loops do. When the innermost loop is a
//! reduction, each output's value must be its old one combined (`addf`,
//! `mulf`, `maximumf`, `minimumf`, `maxnumf` or `minnumf`) with what the
//! rest of the body works out: the loop carries a vector of partial
//! results (`scf.for` with `iter_args`), and `vector.reduction` combines
//! its lanes at the end. That adds in a different order from one element
//! at a time, as any vectorized sum does.
//!
//! Anything else is left as it is, for `linalg-to-loops`. Runs on buffers,
//! after `outline-linalg`.

use std::collections::HashMap;

use crate::affine::{Expr, Map};
use crate::build::OpSpec;
use crate::ir::*;

/// Vectorizes what it can in `src`, `width` elements at a time.
pub fn run(src: &Module, width: u64) -> Result<Module, String> {
    if width < 2 {
        return Err("linalg-vectorize needs a width of 2 or more".into());
    }
    let mut out = src.empty_like();
    let mut map = HashMap::new();
    for top in src.top_ops().collect::<Vec<_>>() {
        copy(src, top, &mut out, None, &mut map, width)?;
    }
    Ok(out)
}

fn copy(
    src: &Module,
    op: OpId,
    out: &mut Module,
    block: Option<BlockId>,
    map: &mut HashMap<ValueId, ValueId>,
    width: u64,
) -> Result<(), String> {
    if src.op_name(op) == "linalg.generic"
        && let Some(block) = block
        && let Some(plan) = plan(src, op, width)
    {
        let width = plan.width;
        return Vectorize { src, out, map, width, swap: plan.swap }.lower(op, &plan, block);
    }
    if src.op_name(op) == "linalg.fill"
        && let Some(block) = block
        && let Some(w) = fill_width(src, op, width)
    {
        return Vectorize { src, out, map, width: w, swap: 0 }.fill(op, block);
    }
    let o = &src.ops[op.index()];
    if o.regions.is_empty() {
        out.clone_op(src, op, block, map)?;
        return Ok(());
    }
    // The operation with empty regions, its blocks then copied here.
    let operands = o
        .operands
        .iter()
        .map(|v| map.get(v).copied().ok_or("an operand from outside"))
        .collect::<Result<Vec<_>, _>>()?;
    let results: Vec<TypeId> = o.results.iter().map(|&r| src.value_type(r)).collect();
    let mut spec =
        OpSpec::new(src.op_name(op)).operands(&operands).results(&results).regions(o.regions.len()).loc(o.loc);
    for (k, v) in dict(src, o.properties) {
        spec = spec.prop(&k, v);
    }
    for (k, v) in dict(src, o.attrs) {
        spec = spec.attr(&k, v);
    }
    let new = spec.build(out, block);
    for (&old, &n) in o.results.iter().zip(&out.ops[new.index()].results.clone()) {
        map.insert(old, n);
    }
    for (k, &r) in o.regions.iter().enumerate() {
        let region = out.region_of(new, k);
        let blocks = src.regions[r.index()].blocks.clone();
        let mut new_blocks = Vec::new();
        for &b in &blocks {
            let types: Vec<TypeId> = src.blocks[b.index()].args.iter().map(|&a| src.value_type(a)).collect();
            let nb = out.add_block(region, &types);
            for (&a, &n) in src.blocks[b.index()].args.iter().zip(&out.blocks[nb.index()].args.clone()) {
                map.insert(a, n);
            }
            new_blocks.push(nb);
        }
        for (&b, &nb) in blocks.iter().zip(&new_blocks) {
            for &inner in &src.blocks[b.index()].ops {
                copy(src, inner, out, Some(nb), map, width)?;
            }
        }
    }
    Ok(())
}

fn dict(m: &Module, d: Option<AttrId>) -> Vec<(String, AttrId)> {
    match d.map(|d| m.attr(d)) {
        Some(Attr::Dict(es)) => es.clone(),
        _ => Vec::new(),
    }
}

/// How an operation is vectorized.
struct Plan {
    maps: Vec<Map>,
    sizes: Vec<u64>,
    /// Whether each operand is loaded a vector at a time (it is indexed by
    /// the innermost loop) or an element at a time.
    vector: Vec<bool>,
    /// For a reduction: each output's combining operation in the body,
    /// whose other operand is what the rest of the body works out.
    combine: Option<Vec<OpId>>,
    /// Lanes in a vector.
    width: u64,
    /// The loop made the innermost, swapped with the last (in `maps`,
    /// already).
    swap: usize,
}

/// How many vectors of partial results a reduction keeps, at most.
const UNROLL: u64 = 4;

/// Operations whose vector form is the same operation on vectors.
const ELEMENTWISE: &[&str] = &[
    "arith.addf",
    "arith.subf",
    "arith.mulf",
    "arith.divf",
    "arith.negf",
    "arith.maximumf",
    "arith.minimumf",
    "arith.maxnumf",
    "arith.minnumf",
    "arith.cmpf",
    "arith.select",
    "math.exp",
    "math.tanh",
    "math.sqrt",
    "math.rsqrt",
    "math.absf",
];

/// Operations that combine a reduction's partial results.
const COMBINE: &[&str] =
    &["arith.addf", "arith.mulf", "arith.maximumf", "arith.minimumf", "arith.maxnumf", "arith.minnumf"];

/// Whether `op` can be vectorized, and how.
fn plan(src: &Module, op: OpId, width: u64) -> Option<Plan> {
    let o = &src.ops[op.index()];
    let Some(Attr::Array(ms)) = src.op_attr(op, "indexing_maps").map(|a| src.attr(src.resolve_attr(a))) else {
        return None;
    };
    let maps: Vec<Map> = ms
        .iter()
        .map(|&m| match src.attr(src.resolve_attr(m)) {
            Attr::Builtin { keyword: "affine_map", body, .. } => Map::parse(&body[1..body.len() - 1]).ok(),
            _ => None,
        })
        .collect::<Option<_>>()?;
    if maps.len() != o.operands.len() || maps.iter().any(|m| m.syms > 0) {
        return None;
    }
    let n = maps[0].dims;
    let d = n.checked_sub(1)?;
    let reduction = match src.op_attr(op, "iterator_types").map(|a| src.attr(src.resolve_attr(a))) {
        Some(Attr::Array(es)) => es
            .iter()
            .map(|&e| matches!(src.attr(e), Attr::Dialect { body: Some(b), .. } if b.contains("reduction")))
            .collect::<Vec<_>>(),
        _ => return None,
    };
    // The innermost loop, else a parallel one in its place; the widest
    // vectors that fit.
    let widths: Vec<u64> = std::iter::successors(Some(width), |&w| (w > 4 && w % 2 == 0).then_some(w / 2)).collect();
    // Widths that divide the loop first; then the widest that fits, the
    // elements past the last whole vector one at a time.
    for tail in [false, true] {
        for swap in std::iter::once(d).chain((0..d).rev().filter(|&k| !reduction[k])) {
            let swapped = |k: usize| {
                if k == swap {
                    d
                } else if k == d {
                    swap
                } else {
                    k
                }
            };
            let maps: Vec<Map> = maps
                .iter()
                .map(|m| Map {
                    dims: m.dims,
                    syms: 0,
                    results: m.results.iter().map(|e| rename(e, &swapped)).collect(),
                })
                .collect();
            let mut reduction = reduction.clone();
            reduction.swap(swap, d);
            for &w in &widths {
                if let Some(p) = plan_at(src, op, w, maps.clone(), &reduction, swap, tail) {
                    return Some(p);
                }
            }
        }
    }
    None
}

/// `e` with its dimensions renamed.
fn rename(e: &Expr, to: &dyn Fn(usize) -> usize) -> Expr {
    let two = |a: &Expr, b: &Expr| (Box::new(rename(a, to)), Box::new(rename(b, to)));
    match e {
        Expr::Dim(k) => Expr::Dim(to(*k)),
        Expr::Sym(_) | Expr::Const(_) => e.clone(),
        Expr::Add(a, b) => {
            let (a, b) = two(a, b);
            Expr::Add(a, b)
        }
        Expr::Mul(a, b) => {
            let (a, b) = two(a, b);
            Expr::Mul(a, b)
        }
        Expr::FloorDiv(a, b) => {
            let (a, b) = two(a, b);
            Expr::FloorDiv(a, b)
        }
        Expr::CeilDiv(a, b) => {
            let (a, b) = two(a, b);
            Expr::CeilDiv(a, b)
        }
        Expr::Mod(a, b) => {
            let (a, b) = two(a, b);
            Expr::Mod(a, b)
        }
    }
}

/// The plan for `op` with `maps` (its own, `swap` swapped with the last
/// loop) and vectors of `width`, if it can be vectorized so.
#[allow(clippy::too_many_arguments)]
fn plan_at(
    src: &Module,
    op: OpId,
    width: u64,
    maps: Vec<Map>,
    reduction: &[bool],
    swap: usize,
    tail: bool,
) -> Option<Plan> {
    let o = &src.ops[op.index()];
    let n = maps[0].dims;
    let d = n - 1;
    let n_ins = crate::bufferize::inputs_of(src, op);

    // The loops' sizes; each operand indexed by the innermost loop only in
    // its last, contiguous dimension.
    let mut sizes = vec![None; n];
    let mut vector = Vec::new();
    for (&v, m) in o.operands.iter().zip(&maps) {
        let (shape, elem) = src.shaped(src.value_type(v))?;
        if !matches!(src.ty(src.resolve_type(src.value_type(v))), Type::MemRef { .. }) || m.results.len() != shape.len()
        {
            return None;
        }
        for (r, &s) in shape.iter().enumerate() {
            if let Some(k) = m.dim_of(r) {
                sizes[k].get_or_insert(s);
            }
        }
        let uses: Vec<usize> = (0..m.results.len()).filter(|&r| uses_dim(&m.results[r], d)).collect();
        match uses[..] {
            [] => vector.push(false),
            [r] if r + 1 == shape.len() && m.results[r] == Expr::Dim(d) => {
                let last_stride = crate::build::strided_layout(src, src.value_type(v)).map_or(1, |(s, _)| s[r]);
                if last_stride != 1 || !matches!(src.ty(elem), Type::Float("f32")) {
                    return None;
                }
                vector.push(true);
            }
            _ => return None,
        }
    }
    let sizes: Vec<u64> = sizes.into_iter().collect::<Option<_>>()?;
    if sizes[d] < width || (!tail && sizes[d] % width != 0) {
        return None;
    }

    // The body: arithmetic with a vector form, or work on scalars that the
    // innermost loop does not change.
    let body = src.regions[o.regions[0].index()].blocks[0];
    let bargs = &src.blocks[body.index()].args;
    let mut varying: Vec<ValueId> = bargs.iter().zip(&vector).filter(|&(_, &v)| v).map(|(&a, _)| a).collect();
    let ops = &src.blocks[body.index()].ops;
    for &inner in ops {
        let i = &src.ops[inner.index()];
        let name = src.op_name(inner);
        if name == "linalg.yield" {
            continue;
        }
        if name == "linalg.index" {
            match src.op_attr(inner, "dim").and_then(|a| src.int_of(a)) {
                Some(k) if k as usize != swap && (k as usize) <= d => continue,
                _ => return None,
            }
        }
        let vary = i.operands.iter().any(|v| varying.contains(v));
        if vary {
            let ok = ELEMENTWISE.contains(&name) || (name == "math.fpowi" && small_power(src, i.operands[1]).is_some());
            if !ok
                || i.results
                    .iter()
                    .any(|&r| !matches!(src.ty(src.value_type(r)), Type::Float("f32") | Type::Int { width: 1, .. }))
            {
                return None;
            }
            varying.extend(i.results.iter().copied());
        } else if !i.regions.is_empty() {
            return None;
        }
    }

    if !reduction[d] {
        // Every output indexed by the innermost loop.
        if !vector[n_ins..].iter().all(|&v| v) {
            return None;
        }
        return Some(Plan { maps, sizes, vector, combine: None, width, swap });
    }
    // A reduction along the innermost loop only, each output its old value
    // combined with something else, and read nowhere else.
    if reduction[..d].iter().any(|&r| r) || vector[n_ins..].iter().any(|&v| v) {
        return None;
    }
    let &yield_op = ops.last()?;
    let mut combine = Vec::new();
    for (k, &y) in src.ops[yield_op.index()].operands.iter().enumerate() {
        let out_arg = *bargs.get(n_ins + k)?;
        let ValueDef::Result { op: c, .. } = src.values[y.index()].def else { return None };
        let co = &src.ops[c.index()];
        if !COMBINE.contains(&src.op_name(c)) || co.operands.len() != 2 || !co.operands.contains(&out_arg) {
            return None;
        }
        // The old value used by the combining operation only.
        let uses = ops.iter().filter(|&&p| src.ops[p.index()].operands.contains(&out_arg)).count();
        if uses != 1 || co.operands[0] == co.operands[1] {
            return None;
        }
        combine.push(c);
    }
    Some(Plan { maps, sizes, vector, combine: Some(combine), width, swap })
}

/// The width a `linalg.fill` is done at: of `f32`, into a buffer whose
/// last dimension is contiguous and at least 4.
fn fill_width(src: &Module, op: OpId, width: u64) -> Option<u64> {
    let o = &src.ops[op.index()];
    let [value, mem] = o.operands[..] else { return None };
    if !matches!(src.ty(src.value_type(value)), Type::Float("f32")) {
        return None;
    }
    // A constant whose four bytes are one byte (0.0, all ones) is left to
    // linalg-to-loops: epitome-opt makes that loop a memset.
    if let ValueDef::Result { op: c, .. } = src.values[value.index()].def
        && src.op_name(c) == "arith.constant"
        && let Some(Attr::Float { text, .. }) = src.op_attr(c, "value").map(|a| src.attr(a))
        && let Some(bits) = parse_f32_bits(text)
        && bits.to_le_bytes().iter().all(|&b| b == bits as u8)
    {
        return None;
    }
    let ty = src.value_type(mem);
    if !matches!(src.ty(src.resolve_type(ty)), Type::MemRef { .. }) {
        return None;
    }
    let (shape, _) = src.shaped(ty)?;
    let last = *shape.last()?;
    let stride = crate::build::strided_layout(src, ty).map_or(1, |(s, _)| *s.last().unwrap_or(&1));
    if stride != 1 {
        return None;
    }
    std::iter::successors(Some(width), |&w| (w > 4 && w % 2 == 0).then_some(w / 2)).find(|&w| last >= w)
}

/// An `f32` attribute's bits, from its text (`0.000000e+00`, `0xFF800000`).
fn parse_f32_bits(text: &str) -> Option<u32> {
    match text.strip_prefix("0x").or_else(|| text.strip_prefix("0X")) {
        Some(hex) => u32::from_str_radix(hex, 16).ok(),
        None => text.parse::<f32>().ok().map(f32::to_bits),
    }
}

/// A constant integer exponent from 0 to 16.
fn small_power(src: &Module, v: ValueId) -> Option<i64> {
    let ValueDef::Result { op, .. } = src.values[v.index()].def else { return None };
    if src.op_name(op) != "arith.constant" {
        return None;
    }
    src.op_attr(op, "value").and_then(|a| src.int_of(a)).filter(|n| (0..=16).contains(n))
}

fn uses_dim(e: &Expr, d: usize) -> bool {
    match e {
        Expr::Dim(k) => *k == d,
        Expr::Sym(_) | Expr::Const(_) => false,
        Expr::Add(a, b) | Expr::Mul(a, b) | Expr::FloorDiv(a, b) | Expr::CeilDiv(a, b) | Expr::Mod(a, b) => {
            uses_dim(a, d) || uses_dim(b, d)
        }
    }
}

struct Vectorize<'a> {
    src: &'a Module,
    out: &'a mut Module,
    map: &'a mut HashMap<ValueId, ValueId>,
    width: u64,
    /// The plan's loop made the innermost.
    swap: usize,
}

impl Vectorize<'_> {
    fn index_const(&mut self, block: BlockId, v: i64) -> ValueId {
        let index = self.out.index_type();
        let attr = self.out.int_attr(v, index);
        let c = OpSpec::new("arith.constant").results(&[index]).prop("value", attr).build(self.out, Some(block));
        self.out.result(c)
    }

    fn vector_of(&mut self, elem: TypeId) -> TypeId {
        self.out.intern_type(Type::Vector { dims: vec![(self.width, false)], elem })
    }

    /// `scf.for %iv = 0 to n step s`, carrying `carried`, at the end of
    /// `block`: the loop, its body, the induction variable and the carried
    /// values' arguments.
    fn for_loop(
        &mut self,
        block: BlockId,
        n: u64,
        step: u64,
        carried: &[ValueId],
    ) -> (OpId, BlockId, ValueId, Vec<ValueId>) {
        self.for_range(block, 0, n, step, carried)
    }

    /// `scf.for %iv = from to n step s`, as `for_loop`.
    fn for_range(
        &mut self,
        block: BlockId,
        from: u64,
        n: u64,
        step: u64,
        carried: &[ValueId],
    ) -> (OpId, BlockId, ValueId, Vec<ValueId>) {
        let lb = self.index_const(block, from as i64);
        let ub = self.index_const(block, n as i64);
        let st = self.index_const(block, step as i64);
        let types: Vec<TypeId> = carried.iter().map(|&v| self.out.value_type(v)).collect();
        let mut operands = vec![lb, ub, st];
        operands.extend_from_slice(carried);
        let f = OpSpec::new("scf.for").operands(&operands).results(&types).regions(1).build(self.out, Some(block));
        let index = self.out.index_type();
        let region = self.out.region_of(f, 0);
        let mut args = vec![index];
        args.extend(&types);
        let body = self.out.add_block(region, &args);
        let a = self.out.blocks[body.index()].args.clone();
        (f, body, a[0], a[1..].to_vec())
    }

    /// The indices map `m` gives at the induction variables.
    fn indices(&mut self, block: BlockId, m: &Map, ivs: &[ValueId]) -> Vec<ValueId> {
        let mut out = Vec::new();
        for e in &m.results {
            out.push(match e {
                Expr::Dim(d) => ivs[*d],
                Expr::Const(c) => self.index_const(block, *c),
                _ => {
                    let attr = self
                        .out
                        .affine_map_attr(&Map { dims: ivs.len(), syms: 0, results: vec![e.clone()] }.to_string());
                    let index = self.out.index_type();
                    let a = OpSpec::new("affine.apply")
                        .operands(ivs)
                        .results(&[index])
                        .prop("map", attr)
                        .build(self.out, Some(block));
                    self.out.result(a)
                }
            });
        }
        out
    }

    fn load(&mut self, block: BlockId, mem: ValueId, idx: &[ValueId], vector: bool) -> ValueId {
        let elem = self.out.shaped(self.out.value_type(mem)).unwrap().1;
        let mut operands = vec![mem];
        operands.extend_from_slice(idx);
        let op = if vector {
            let ty = self.vector_of(elem);
            let f = self.out.bool_attr(false);
            OpSpec::new("vector.load")
                .operands(&operands)
                .results(&[ty])
                .prop("nontemporal", f)
                .build(self.out, Some(block))
        } else {
            OpSpec::new("memref.load").operands(&operands).results(&[elem]).build(self.out, Some(block))
        };
        self.out.result(op)
    }

    fn store(&mut self, block: BlockId, v: ValueId, mem: ValueId, idx: &[ValueId], vector: bool) {
        let mut operands = vec![v, mem];
        operands.extend_from_slice(idx);
        if vector {
            let f = self.out.bool_attr(false);
            OpSpec::new("vector.store").operands(&operands).prop("nontemporal", f).build(self.out, Some(block));
        } else {
            OpSpec::new("memref.store").operands(&operands).build(self.out, Some(block));
        }
    }

    fn is_vector(&self, v: ValueId) -> bool {
        matches!(self.out.ty(self.out.value_type(v)), Type::Vector { .. })
    }

    /// `v` as a vector: itself, or broadcast.
    fn splat(&mut self, block: BlockId, v: ValueId) -> ValueId {
        if self.is_vector(v) {
            return v;
        }
        let ty = self.vector_of(self.out.value_type(v));
        let b = OpSpec::new("vector.broadcast").operands(&[v]).results(&[ty]).build(self.out, Some(block));
        self.out.result(b)
    }

    /// The body's operation `inner`, on vectors if any operand is one.
    fn body_op(&mut self, block: BlockId, inner: OpId, ivs: &[ValueId]) -> Result<(), String> {
        let src = self.src;
        let i = &src.ops[inner.index()];
        let name = src.op_name(inner);
        if name == "linalg.index" {
            let d = src.op_attr(inner, "dim").and_then(|a| src.int_of(a)).ok_or("linalg.index without dim")?;
            // (Its loop's place, after the swap.)
            let (d, last) = (d as usize, ivs.len() - 1);
            let d = if d == self.swap {
                last
            } else if d == last {
                self.swap
            } else {
                d
            };
            self.map.insert(i.results[0], ivs[d]);
            return Ok(());
        }
        let args: Vec<ValueId> = i
            .operands
            .iter()
            .map(|v| self.map.get(v).copied().ok_or("an operand from outside"))
            .collect::<Result<_, _>>()?;
        if !args.iter().any(|&a| self.is_vector(a)) {
            self.out.clone_op(src, inner, Some(block), self.map)?;
            return Ok(());
        }
        if name == "math.fpowi" {
            // x to a small constant power: multiplications.
            let n = small_power(src, i.operands[1]).ok_or("fpowi of a power that is not a small constant")?;
            let ty = self.out.value_type(args[0]);
            let one =
                self.out.intern_attr(Attr::Builtin { keyword: "dense", body: "<1.000000e+00>".into(), ty: Some(ty) });
            let c = OpSpec::new("arith.constant").results(&[ty]).prop("value", one).build(self.out, Some(block));
            let mut r = self.out.result(c);
            let fm = self.out.dialect_attr("arith.fastmath", Some("<none>"));
            for _ in 0..n {
                let m = OpSpec::new("arith.mulf")
                    .operands(&[r, args[0]])
                    .results(&[ty])
                    .prop("fastmath", fm)
                    .build(self.out, Some(block));
                r = self.out.result(m);
            }
            self.map.insert(i.results[0], r);
            return Ok(());
        }
        let args: Vec<ValueId> = if name == "arith.select" {
            // The condition stays a scalar if it is one.
            let mut a = vec![args[0]];
            a.push(self.splat(block, args[1]));
            a.push(self.splat(block, args[2]));
            a
        } else {
            args.iter().map(|&a| self.splat(block, a)).collect()
        };
        let results: Vec<TypeId> = i.results.iter().map(|&r| self.vector_of(src.value_type(r))).collect();
        let mut spec = OpSpec::new(name).operands(&args).results(&results).loc(i.loc);
        for (k, v) in dict(src, i.properties) {
            spec = spec.prop(&k, v);
        }
        let new = spec.build(self.out, Some(block));
        for (&old, &n) in i.results.iter().zip(&self.out.ops[new.index()].results.clone()) {
            self.map.insert(old, n);
        }
        Ok(())
    }

    /// A `linalg.fill`: each row's whole vectors stored a splat at a time,
    /// then the elements past them one at a time.
    fn fill(&mut self, op: OpId, block: BlockId) -> Result<(), String> {
        let src = self.src;
        let o = &src.ops[op.index()];
        let value = *self.map.get(&o.operands[0]).ok_or("an operand from outside")?;
        let mem = *self.map.get(&o.operands[1]).ok_or("an operand from outside")?;
        let (shape, _) = self.out.shaped(self.out.value_type(mem)).ok_or("a fill of no shape")?;
        let (last, outer) = shape.split_last().ok_or("a fill of a scalar")?;
        let mut at = block;
        let mut ivs = Vec::new();
        let mut loops = Vec::new();
        for &s in outer {
            if s == 1 {
                ivs.push(self.index_const(at, 0));
                continue;
            }
            let (f, b, iv, _) = self.for_loop(at, s, 1, &[]);
            loops.push((f, b));
            ivs.push(iv);
            at = b;
        }
        let splat = self.splat(at, value);
        let whole = last / self.width * self.width;
        for (from, to, step, vector) in [(0, whole, self.width, true), (whole, *last, 1, false)] {
            if from == to {
                continue;
            }
            let (_, inner, iv, _) = self.for_range(at, from, to, step, &[]);
            let mut idx = ivs.clone();
            idx.push(iv);
            self.store(inner, if vector { splat } else { value }, mem, &idx, vector);
            OpSpec::new("scf.yield").build(self.out, Some(inner));
        }
        for &(_, b) in loops.iter().rev() {
            OpSpec::new("scf.yield").build(self.out, Some(b));
        }
        Ok(())
    }

    fn lower(&mut self, op: OpId, plan: &Plan, block: BlockId) -> Result<(), String> {
        let src = self.src;
        let o = &src.ops[op.index()];
        let operands: Vec<ValueId> = o
            .operands
            .iter()
            .map(|v| self.map.get(v).copied().ok_or("an operand from outside"))
            .collect::<Result<_, _>>()?;
        let n_ins = crate::bufferize::inputs_of(src, op);
        let n = plan.sizes.len();
        let d = n - 1;
        let body = src.regions[o.regions[0].index()].blocks[0];
        let bargs = src.blocks[body.index()].args.clone();
        let ops = src.blocks[body.index()].ops.clone();

        // The outer loops; a dimension of one is index 0, not a loop.
        let mut at = block;
        let mut ivs = Vec::new();
        let mut loops = Vec::new();
        for &s in &plan.sizes[..d] {
            if s == 1 {
                let zero = self.index_const(at, 0);
                ivs.push(zero);
                continue;
            }
            let (f, b, iv, _) = self.for_loop(at, s, 1, &[]);
            loops.push((f, b));
            ivs.push(iv);
            at = b;
        }

        match &plan.combine {
            None => {
                // Whole vectors, then the elements past them one at a time.
                let whole = plan.sizes[d] / self.width * self.width;
                for (from, to, step, vector) in [(0, whole, self.width, true), (whole, plan.sizes[d], 1, false)] {
                    if from == to {
                        continue;
                    }
                    let (_, inner, iv, _) = self.for_range(at, from, to, step, &[]);
                    let mut ivs = ivs.clone();
                    ivs.push(iv);
                    for (k, &v) in operands.iter().enumerate() {
                        if k >= n_ins && !ops.iter().any(|&p| src.ops[p.index()].operands.contains(&bargs[k])) {
                            continue;
                        }
                        let idx = self.indices(inner, &plan.maps[k], &ivs);
                        let x = self.load(inner, v, &idx, vector && plan.vector[k]);
                        self.map.insert(bargs[k], x);
                    }
                    for &p in &ops {
                        if src.op_name(p) == "linalg.yield" {
                            for (k, y) in src.ops[p.index()].operands.iter().enumerate() {
                                let mut v = *self.map.get(y).ok_or("a yield of an unknown value")?;
                                if vector {
                                    v = self.splat(inner, v);
                                }
                                let idx = self.indices(inner, &plan.maps[n_ins + k], &ivs);
                                self.store(inner, v, operands[n_ins + k], &idx, vector);
                            }
                            break;
                        }
                        self.body_op(inner, p, &ivs)?;
                    }
                    OpSpec::new("scf.yield").build(self.out, Some(inner));
                }
            }
            Some(combine) => {
                // Each output's old value, and a vector of partial results:
                // the old value in every lane for a maximum or minimum (which
                // it does not change), the identity for a sum or product.
                let mut olds = Vec::new();
                let mut inits = Vec::new();
                for (k, &c) in combine.iter().enumerate() {
                    let idx = self.indices(at, &plan.maps[n_ins + k], &ivs);
                    let old = self.load(at, operands[n_ins + k], &idx, false);
                    olds.push((old, idx));
                    let init = match src.op_name(c) {
                        "arith.addf" | "arith.mulf" => {
                            let ty = self.vector_of(self.out.value_type(old));
                            let one = if src.op_name(c) == "arith.addf" { "0.000000e+00" } else { "1.000000e+00" };
                            let a = self.out.intern_attr(Attr::Builtin {
                                keyword: "dense",
                                body: format!("<{one}>"),
                                ty: Some(ty),
                            });
                            let c =
                                OpSpec::new("arith.constant").results(&[ty]).prop("value", a).build(self.out, Some(at));
                            self.out.result(c)
                        }
                        _ => self.splat(at, old),
                    };
                    inits.push(init);
                }
                // UNROLL vectors of partial results for each output, each
                // its own chain of combining: one would make each step wait
                // for the one before (a sum: the latency of an addition).
                let vectors = plan.sizes[d] / self.width;
                let unroll = [UNROLL, 2, 1].into_iter().find(|&u| vectors % u == 0 && vectors >= u).unwrap();
                let whole = vectors * self.width;
                let carried: Vec<ValueId> = (0..unroll as usize).flat_map(|_| inits.iter().copied()).collect();
                let (f, inner, iv, accs) = self.for_loop(at, whole, self.width * unroll, &carried);
                let mut next = Vec::new();
                for u in 0..unroll as usize {
                    let ivu = if u == 0 {
                        iv
                    } else {
                        let off = self.index_const(inner, (u as u64 * self.width) as i64);
                        let index = self.out.index_type();
                        let a = OpSpec::new("arith.addi")
                            .operands(&[iv, off])
                            .results(&[index])
                            .build(self.out, Some(inner));
                        self.out.result(a)
                    };
                    let mut ivs_u = ivs.clone();
                    ivs_u.push(ivu);
                    for (k, &v) in operands[..n_ins].iter().enumerate() {
                        let idx = self.indices(inner, &plan.maps[k], &ivs_u);
                        let x = self.load(inner, v, &idx, plan.vector[k]);
                        self.map.insert(bargs[k], x);
                    }
                    // The body, each output's old value standing for this
                    // copy's vector of partial results.
                    for k in 0..combine.len() {
                        self.map.insert(bargs[n_ins + k], accs[u * combine.len() + k]);
                    }
                    for &p in &ops {
                        if src.op_name(p) == "linalg.yield" {
                            for y in &src.ops[p.index()].operands {
                                next.push(*self.map.get(y).ok_or("a yield of an unknown value")?);
                            }
                            break;
                        }
                        self.body_op(inner, p, &ivs_u)?;
                    }
                }
                OpSpec::new("scf.yield").operands(&next).build(self.out, Some(inner));
                // Each output's vectors combined, in pairs.
                let results = self.out.ops[f.index()].results.clone();
                let fm = self.out.dialect_attr("arith.fastmath", Some("<none>"));
                let mut partial = Vec::new();
                for (k, &c) in combine.iter().enumerate() {
                    let mut vs: Vec<ValueId> = (0..unroll as usize).map(|u| results[u * combine.len() + k]).collect();
                    while vs.len() > 1 {
                        let ty = self.out.value_type(vs[0]);
                        let mut pairs = Vec::new();
                        for pair in vs.chunks(2) {
                            let v = OpSpec::new(src.op_name(c))
                                .operands(pair)
                                .results(&[ty])
                                .prop("fastmath", fm)
                                .build(self.out, Some(at));
                            pairs.push(self.out.result(v));
                        }
                        vs = pairs;
                    }
                    partial.push(vs[0]);
                }
                // The lanes combined, then with the old value.
                let mut finals = Vec::new();
                for (k, &c) in combine.iter().enumerate() {
                    let kind = match src.op_name(c) {
                        "arith.addf" => "add",
                        "arith.mulf" => "mul",
                        "arith.maximumf" => "maximumf",
                        "arith.minimumf" => "minimumf",
                        "arith.maxnumf" => "maxnumf",
                        _ => "minnumf",
                    };
                    let (old, idx) = olds[k].clone();
                    let elem = self.out.value_type(old);
                    let kind = self.out.dialect_attr("vector.kind", Some(&format!("<{kind}>")));
                    let fm = self.out.dialect_attr("arith.fastmath", Some("<none>"));
                    let r = OpSpec::new("vector.reduction")
                        .operands(&[partial[k]])
                        .results(&[elem])
                        .prop("fastmath", fm)
                        .prop("kind", kind)
                        .build(self.out, Some(at));
                    let r = self.out.result(r);
                    let both = OpSpec::new(src.op_name(c))
                        .operands(&[r, old])
                        .results(&[elem])
                        .prop("fastmath", fm)
                        .build(self.out, Some(at));
                    let both = self.out.result(both);
                    finals.push((both, idx, k));
                }
                // The elements past the last whole vector, one at a time.
                if whole < plan.sizes[d] {
                    let carried: Vec<ValueId> = finals.iter().map(|(v, _, _)| *v).collect();
                    let (f, inner, iv, accs) = self.for_range(at, whole, plan.sizes[d], 1, &carried);
                    let mut ivs_t = ivs.clone();
                    ivs_t.push(iv);
                    for (k, &v) in operands[..n_ins].iter().enumerate() {
                        let idx = self.indices(inner, &plan.maps[k], &ivs_t);
                        let x = self.load(inner, v, &idx, false);
                        self.map.insert(bargs[k], x);
                    }
                    for k in 0..combine.len() {
                        self.map.insert(bargs[n_ins + k], accs[k]);
                    }
                    let mut next = Vec::new();
                    for &p in &ops {
                        if src.op_name(p) == "linalg.yield" {
                            for y in &src.ops[p.index()].operands {
                                next.push(*self.map.get(y).ok_or("a yield of an unknown value")?);
                            }
                            break;
                        }
                        self.body_op(inner, p, &ivs_t)?;
                    }
                    OpSpec::new("scf.yield").operands(&next).build(self.out, Some(inner));
                    let results = self.out.ops[f.index()].results.clone();
                    for (fin, r) in finals.iter_mut().zip(results) {
                        fin.0 = r;
                    }
                }
                for (v, idx, k) in finals {
                    self.store(at, v, operands[n_ins + k], &idx, false);
                }
            }
        }
        // The outer loops' bodies end in `scf.yield`.
        for &(_, b) in loops.iter().rev() {
            OpSpec::new("scf.yield").build(self.out, Some(b));
        }
        Ok(())
    }
}
