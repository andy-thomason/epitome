//! `linalg` on buffers to a target's kernels: a `linalg.generic` that a
//! kernel computes is done a tile at a time by calls to the kernel's
//! function, as upstream lowers `linalg.matmul` to `arm_sme` outer products
//! on AArch64 with SME.
//!
//! Which operations a target has kernels for, and the kernels themselves,
//! are data: `mlir/targets/<name>.mlir` (the pass is
//! `linalg-to-kernels=<name>`, or `=<path>` for a file of one's own). Each
//! `epitome.kernel` there gives the indexing maps, iterators, body and
//! element type of the `linalg.generic` it computes, the size of a tile of
//! the output, and the function that computes one; the file defines the
//! function, which this pass copies into the module.
//!
//! So far kernels multiply matrices, `C[m, n] += A[m, k] B[k, n]`. The pass
//! packs A transposed, so that a column of a tile of A is contiguous, then
//! calls the function for each tile of C:
//! `(c, ldc, at, lda, b, ldb, k, cols)`, pointers and row strides in
//! elements. Rows of C past the last whole tile are left to a
//! `linalg.generic` on views of A and C; the last tile of columns may be
//! narrower, and `cols` says by how much. The pass runs after
//! `bufferize` and before `linalg-to-loops`.

use std::collections::HashMap;
use std::sync::OnceLock;

use crate::affine::Map;
use crate::build::{OpSpec, row_major_strides, strided_attr, strided_layout};
use crate::ir::*;

const BUILT_IN: &[(&str, &str)] = &[
    ("aarch64-sme", include_str!("../targets/aarch64-sme.mlir")),
    ("x86-avx512", include_str!("../targets/x86-avx512.mlir")),
];

/// Lowers what `kernels` (a name in `mlir/targets`, or a path) has kernels
/// for. `name:bf16` (or another format) also allows the kernels whose
/// constant weights are stored in that format (`weights = "bf16"`), which
/// round them, and so are used only when asked for.
pub fn run(src: &Module, kernels: &str) -> Result<Module, String> {
    let (name, weights) = match kernels.rsplit_once(':') {
        Some((n, w)) if !w.contains('/') => (n, Some(w.to_string())),
        _ => (kernels, None),
    };
    let set = load(name)?;
    let mut out = src.empty_like();
    let mut cx = Cx {
        set,
        weights,
        imported: HashMap::new(),
        resources: src.resources(),
        packed: HashMap::new(),
        blobs: Vec::new(),
        zero: None,
    };
    for top in src.top_ops().collect::<Vec<_>>() {
        if src.op_name(top) != "builtin.module" {
            return Err(format!("{}: expected a builtin.module", src.op_name(top)));
        }
        let o = &src.ops[top.index()];
        let mut spec = OpSpec::new("builtin.module").regions(1).loc(o.loc);
        for (k, v) in dict(src, o.properties).into_iter().chain(dict(src, o.attrs)) {
            spec = spec.prop(&k, v);
        }
        let module = spec.build(&mut out, None);
        let region = out.region_of(module, 0);
        let block = out.add_block(region, &[]);
        let mut map = HashMap::new();
        for &r in &o.regions {
            for &b in &src.regions[r.index()].blocks {
                for &op in &src.blocks[b.index()].ops {
                    if src.op_name(op) == "func.func" {
                        func(src, op, &mut out, block, &mut cx, &mut map)?;
                    } else {
                        out.clone_op(src, op, Some(block), &mut map)?;
                    }
                }
            }
        }
    }
    // The packed weights' blobs, before the file's own resources; the
    // weights no call reads any longer, gone.
    let at = out.items.iter().position(|i| matches!(i, Item::Resources(_))).unwrap_or(out.items.len());
    for (k, b) in cx.blobs.into_iter().enumerate() {
        out.items.insert(at + k, b);
    }
    drop_unused_globals(&mut out);
    Ok(out)
}

/// What the pass carries from one operation to the next.
struct Cx<'a> {
    set: &'static Set,
    /// The format of constant weights allowed, beyond f32.
    weights: Option<String>,
    /// The kernels' functions copied into the module so far.
    imported: HashMap<String, String>,
    /// The blobs of the module being lowered, for packing constants.
    resources: Resources<'a>,
    /// Constants packed so far: the global of each, by (global, panel,
    /// format), and the new blobs.
    packed: HashMap<(String, u64, String), String>,
    blobs: Vec<Item>,
    /// For the operation being lowered: the zero its output was just filled
    /// with, the fill left out, for the kernels to do or redo.
    zero: Option<ValueId>,
}

/// Removes `memref.get_global`s whose result nothing uses, then the
/// constant `memref.global`s nothing gets.
fn drop_unused_globals(m: &mut Module) {
    let used: std::collections::HashSet<ValueId> = m.ops.iter().flat_map(|o| o.operands.iter().copied()).collect();
    for b in 0..m.blocks.len() {
        let ops = std::mem::take(&mut m.blocks[b].ops);
        let kept = ops
            .into_iter()
            .filter(|&op| {
                let o = &m.ops[op.index()];
                !(m.name(o.name) == "memref.get_global" && !used.contains(&o.results[0]))
            })
            .collect();
        m.blocks[b].ops = kept;
    }
    let mut got = std::collections::HashSet::new();
    for b in &m.blocks {
        for &op in &b.ops {
            if m.op_name(op) == "memref.get_global"
                && let Some(Attr::SymbolRef(p)) = m.op_attr(op, "name").map(|a| m.attr(a))
            {
                got.insert(p[0].clone());
            }
        }
    }
    for b in 0..m.blocks.len() {
        let ops = std::mem::take(&mut m.blocks[b].ops);
        let kept = ops
            .into_iter()
            .filter(|&op| {
                m.op_name(op) != "memref.global"
                    || m.op_attr(op, "constant").is_none()
                    || m.op_attr(op, "sym_name").and_then(|a| m.string_of(a)).is_some_and(|n| got.contains(&n))
            })
            .collect();
        m.blocks[b].ops = kept;
    }
}

fn dict(m: &Module, d: Option<AttrId>) -> Vec<(String, AttrId)> {
    match d.map(|d| m.attr(d)) {
        Some(Attr::Dict(es)) => es.clone(),
        _ => Vec::new(),
    }
}

/// A target's kernels: the module of its file, and each kernel's
/// description.
struct Set {
    m: Module,
    kernels: Vec<Kernel>,
}

struct Kernel {
    maps: Vec<Map>,
    iterators: Vec<String>,
    body: Vec<String>,
    element: TypeId,
    tile: (u64, u64),
    function: String,
    /// `batch_matmul`: a matrix product for each index of the first
    /// dimension, each done as a `matmul`.
    batch: bool,
    /// The function does every column of a block of rows, its last
    /// argument the number of columns (rather than one tile, its last
    /// argument the tile's width).
    whole_rows: bool,
    /// The function takes one more argument, how far apart B's panels of
    /// this many columns are, and B may be a constant packed into such
    /// panels, each k rows of them, contiguous.
    pack_b: Option<u64>,
    /// The function takes A as it is, with a buffer of k x (the tile's rows)
    /// to transpose it into, rather than A transposed.
    transpose_a: bool,
    /// B must be a constant, which the pass stores in this format (`bf16`)
    /// in panels of `pack_b` columns.
    weights: Option<String>,
    /// The function's twin for C that starts zeroed: its sums start at
    /// zero, and C is not read.
    zeroed: Option<String>,
}

fn load(name: &str) -> Result<&'static Set, String> {
    static SETS: OnceLock<std::sync::Mutex<HashMap<String, &'static Set>>> = OnceLock::new();
    let sets = SETS.get_or_init(Default::default);
    if let Some(&s) = sets.lock().unwrap().get(name) {
        return Ok(s);
    }
    let text = match BUILT_IN.iter().find(|(n, _)| *n == name) {
        Some((_, t)) => t.to_string(),
        None => std::fs::read_to_string(name).map_err(|e| {
            let known: Vec<&str> = BUILT_IN.iter().map(|(n, _)| *n).collect();
            format!("{name}: {e} (built in: {})", known.join(", "))
        })?,
    };
    let m = crate::parse_module(&text).map_err(|e| format!("{name}:{e}"))?;
    let mut kernels = Vec::new();
    for op in m.top_ops() {
        if m.op_name(op) != "epitome.kernel" {
            continue;
        }
        let get = |k: &str| m.op_attr(op, k).ok_or_else(|| format!("{name}: a kernel without '{k}'"));
        let strings = |a: AttrId| -> Vec<String> {
            match m.attr(a) {
                Attr::Array(es) => es.iter().filter_map(|&e| m.string_of(e)).collect(),
                _ => Vec::new(),
            }
        };
        let maps = match m.attr(get("indexing_maps")?) {
            Attr::Array(es) => es
                .iter()
                .map(|&e| match m.attr(e) {
                    Attr::Builtin { keyword: "affine_map", body, .. } => Map::parse(&body[1..body.len() - 1]),
                    _ => Err("indexing_maps holds something else".to_string()),
                })
                .collect::<Result<Vec<_>, _>>()?,
            _ => return Err(format!("{name}: indexing_maps is not a list")),
        };
        let element = match m.attr(get("element")?) {
            Attr::Type(t) => *t,
            _ => return Err(format!("{name}: element is not a type")),
        };
        let tile = match m.int_list(get("tile")?).as_deref() {
            Some(&[r, c]) => (r as u64, c as u64),
            _ => return Err(format!("{name}: tile should be [rows, columns]")),
        };
        let function = match m.attr(get("function")?) {
            Attr::SymbolRef(p) => p[0].clone(),
            _ => return Err(format!("{name}: function is not a symbol")),
        };
        let batch = match m.op_attr(op, "op").and_then(|a| m.string_of(a)).as_deref() {
            Some("matmul") => false,
            Some("batch_matmul") => true,
            _ => return Err(format!("{name}: only matmul and batch_matmul kernels are supported")),
        };
        kernels.push(Kernel {
            maps,
            iterators: strings(get("iterator_types")?),
            body: strings(get("body")?),
            element,
            tile,
            function,
            batch,
            whole_rows: m.op_attr(op, "whole_rows").is_some(),
            pack_b: m.op_attr(op, "pack_b").and_then(|a| m.int_of(a)).map(|p| p as u64),
            transpose_a: m.op_attr(op, "transpose_a").is_some(),
            weights: m.op_attr(op, "weights").and_then(|a| m.string_of(a)),
            zeroed: match m.op_attr(op, "zeroed").map(|a| m.attr(a)) {
                Some(Attr::SymbolRef(p)) => Some(p[0].clone()),
                _ => None,
            },
        });
    }
    let set: &'static Set = Box::leak(Box::new(Set { m, kernels }));
    sets.lock().unwrap().insert(name.to_string(), set);
    Ok(set)
}

/// A function, its `linalg.generic`s that a kernel computes replaced.
fn func(
    src: &Module,
    f: OpId,
    out: &mut Module,
    parent: BlockId,
    cx: &mut Cx<'_>,
    map: &mut HashMap<ValueId, ValueId>,
) -> Result<(), String> {
    let set = cx.set;
    let o = &src.ops[f.index()];
    let mut spec = OpSpec::new("func.func").regions(o.regions.len()).loc(o.loc);
    for (k, v) in dict(src, o.properties) {
        spec = spec.prop(&k, v);
    }
    for (k, v) in dict(src, o.attrs) {
        spec = spec.attr(&k, v);
    }
    let new = spec.build(out, Some(parent));
    for (k, &r) in o.regions.iter().enumerate() {
        let region = out.region_of(new, k);
        for &b in &src.regions[r.index()].blocks {
            let types: Vec<TypeId> = src.blocks[b.index()].args.iter().map(|&a| src.value_type(a)).collect();
            let nb = out.add_block(region, &types);
            for (&a, &n) in src.blocks[b.index()].args.iter().zip(&out.blocks[nb.index()].args.clone()) {
                map.insert(a, n);
            }
            let weights = cx.weights.clone();
            let kernels_for = |op: OpId| -> Vec<&Kernel> {
                if src.op_name(op) != "linalg.generic" {
                    return Vec::new();
                }
                set.kernels
                    .iter()
                    .filter(|k| k.weights.is_none() || k.weights == weights)
                    .filter(|k| matches(src, op, &set.m, k))
                    .collect()
            };
            let ops = &src.blocks[b.index()].ops;
            // A fill of zeros into a matrix product's output just before it,
            // left for the product's lowering: a kernel whose sums start at
            // zero makes it unneeded.
            let mut deferred: Option<OpId> = None;
            for (i, &op) in ops.iter().enumerate() {
                if let Some(&next) = ops.get(i + 1)
                    && zero_fill(src, op).is_some_and(|mem| src.ops[next.index()].operands.get(2) == Some(&mem))
                    && kernels_for(next).first().is_some_and(|k| !k.batch)
                {
                    deferred = Some(op);
                    continue;
                }
                let ks = kernels_for(op);
                let fill = deferred.take();
                let done = match ks.first() {
                    Some(k) if k.batch => batched(src, op, out, nb, parent, cx, &ks, map)?,
                    Some(_) => {
                        let o = &src.ops[op.index()];
                        let operands = [o.operands[0], o.operands[1], o.operands[2]].map(|v| map[&v]);
                        let props = dict(src, o.properties);
                        cx.zero = fill.map(|f| map[&src.ops[f.index()].operands[0]]);
                        let done = tiles(src, op, operands, &props, out, nb, parent, cx, &ks, map)?;
                        if done && cx.zero.is_some() {
                            return Err("a fill left out and not done".into());
                        }
                        cx.zero = None;
                        done
                    }
                    None => false,
                };
                if !done {
                    if let Some(f) = fill {
                        out.clone_op(src, f, Some(nb), map)?;
                    }
                    out.clone_op(src, op, Some(nb), map)?;
                }
            }
        }
    }
    Ok(())
}

/// The buffer `op` fills, if it is a `linalg.fill` of +0.0.
fn zero_fill(src: &Module, op: OpId) -> Option<ValueId> {
    if src.op_name(op) != "linalg.fill" {
        return None;
    }
    let [value, mem] = src.ops[op.index()].operands[..] else { return None };
    let ValueDef::Result { op: c, .. } = src.values[value.index()].def else { return None };
    if src.op_name(c) != "arith.constant" {
        return None;
    }
    match src.op_attr(c, "value").map(|a| src.attr(a)) {
        Some(Attr::Float { text, .. }) if text.parse::<f64>().is_ok_and(|v| v.to_bits() == 0) => Some(mem),
        _ => None,
    }
}

/// `linalg.fill` of `value` into `mem`, at the end of `block`.
fn fill(out: &mut Module, block: BlockId, value: ValueId, mem: ValueId) {
    let elem = out.value_type(value);
    let segments = out.dense_array_attr("i32", &[1, 1]);
    let f = OpSpec::new("linalg.fill")
        .operands(&[value, mem])
        .prop("operandSegmentSizes", segments)
        .regions(1)
        .build(out, Some(block));
    let region = out.region_of(f, 0);
    let body = out.add_block(region, &[elem, elem]);
    let x = out.blocks[body.index()].args[0];
    OpSpec::new("linalg.yield").operands(&[x]).build(out, Some(body));
}

/// Whether `op` is the `linalg.generic` kernel `k` computes: the same maps,
/// iterators, element type and body.
fn matches(src: &Module, op: OpId, km: &Module, k: &Kernel) -> bool {
    let o = &src.ops[op.index()];
    if crate::bufferize::inputs_of(src, op) != 2 || o.operands.len() != 3 || !o.results.is_empty() {
        return false;
    }
    let Some(Attr::Array(ms)) = src.op_attr(op, "indexing_maps").map(|a| src.attr(src.resolve_attr(a))) else {
        return false;
    };
    let maps: Vec<Option<Map>> = ms
        .iter()
        .map(|&m| match src.attr(src.resolve_attr(m)) {
            Attr::Builtin { keyword: "affine_map", body, .. } => Map::parse(&body[1..body.len() - 1]).ok(),
            _ => None,
        })
        .collect();
    if maps.len() != k.maps.len() || maps.iter().zip(&k.maps).any(|(a, b)| a.as_ref() != Some(b)) {
        return false;
    }
    let iterators: Vec<String> = match src.op_attr(op, "iterator_types").map(|a| src.attr(src.resolve_attr(a))) {
        Some(Attr::Array(es)) => es
            .iter()
            .filter_map(|&e| match src.attr(e) {
                Attr::Dialect { body: Some(b), .. } => Some(b.trim_matches(|c| c == '<' || c == '>').to_string()),
                _ => None,
            })
            .collect(),
        _ => return false,
    };
    if iterators != k.iterators {
        return false;
    }
    // Every operand's elements are of the kernel's type.
    let elem = crate::printer::type_text(km, k.element);
    if o.operands
        .iter()
        .any(|&v| src.shaped(src.value_type(v)).is_none_or(|(_, e)| crate::printer::type_text(src, e) != elem))
    {
        return false;
    }
    // The body: a product of the inputs, added to the output.
    let body = src.regions[o.regions[0].index()].blocks[0];
    let args = &src.blocks[body.index()].args;
    let ops = &src.blocks[body.index()].ops;
    let names: Vec<&str> = ops.iter().map(|&b| src.op_name(b)).collect();
    if names.len() != k.body.len() + 1
        || names[..k.body.len()] != k.body.iter().map(String::as_str).collect::<Vec<_>>()[..]
    {
        return false;
    }
    let (mul, add, ret) = (&src.ops[ops[0].index()], &src.ops[ops[1].index()], &src.ops[ops[2].index()]);
    let product = mul.operands.len() == 2 && mul.operands.contains(&args[0]) && mul.operands.contains(&args[1]);
    let sum = add.operands.len() == 2 && add.operands.contains(&args[2]) && add.operands.contains(&mul.results[0]);
    product && sum && ret.operands == add.results
}

/// A row-major or strided memref's strides and offset.
fn layout(m: &Module, ty: TypeId) -> Option<(Vec<u64>, u64)> {
    strided_layout(m, ty).or_else(|| m.shaped(ty).map(|(s, _)| (row_major_strides(&s), 0)))
}

/// A `batch_matmul` as a `matmul` for each index of its first dimension,
/// on views of its operands; false (and nothing added) where its shapes do
/// not suit the kernel.
#[allow(clippy::too_many_arguments)]
fn batched(
    src: &Module,
    op: OpId,
    out: &mut Module,
    block: BlockId,
    module_block: BlockId,
    cx: &mut Cx<'_>,
    ks: &[&Kernel],
    map: &mut HashMap<ValueId, ValueId>,
) -> Result<bool, String> {
    let o = &src.ops[op.index()];
    let operands = [o.operands[0], o.operands[1], o.operands[2]].map(|v| map[&v]);
    let fewest = ks.iter().map(|k| k.tile.0).min().unwrap_or(u64::MAX);
    let mut shapes = Vec::new();
    for &v in &operands {
        let ty = out.value_type(v);
        let (Some((shape, elem)), Some(l)) = (out.shaped(ty), layout(out, ty)) else { return Ok(false) };
        shapes.push((shape, elem, l));
    }
    let batch = shapes[0].0[0];
    if shapes.iter().any(|s| s.0.len() != 3 || s.0[0] != batch) || shapes[0].0[1] < fewest {
        return Ok(false);
    }
    // A matmul's maps and iterators, for the rows past the last whole tile.
    let maps = ["(d0, d1, d2) -> (d0, d2)", "(d0, d1, d2) -> (d2, d1)", "(d0, d1, d2) -> (d0, d1)"]
        .iter()
        .map(|m| out.affine_map_attr(m))
        .collect();
    let maps = out.array_attr(maps);
    let par = out.dialect_attr("linalg.iterator_type", Some("<parallel>"));
    let red = out.dialect_attr("linalg.iterator_type", Some("<reduction>"));
    let iters = out.array_attr(vec![par, par, red]);
    let segments = out.dense_array_attr("i32", &[2, 1]);
    let props = vec![
        ("indexing_maps".to_string(), maps),
        ("iterator_types".to_string(), iters),
        ("operandSegmentSizes".to_string(), segments),
    ];
    for i in 0..batch {
        // Each operand's matrix i, a rank-reduced view.
        let mut views = [operands[0]; 3];
        for (j, (v, (shape, elem, (strides, offset)))) in operands.iter().zip(&shapes).enumerate() {
            let layout = strided_attr(out, &strides[1..], offset + i * strides[0]);
            let ty = out.memref_type(&shape[1..], *elem, Some(layout));
            let segs = out.dense_array_attr("i32", &[1, 0, 0, 0]);
            let offsets = out.dense_array_attr("i64", &[i as i64, 0, 0]);
            let sizes = out.dense_array_attr("i64", &[1, shape[1] as i64, shape[2] as i64]);
            let ones = out.dense_array_attr("i64", &[1, 1, 1]);
            let s = OpSpec::new("memref.subview")
                .operands(&[*v])
                .results(&[ty])
                .prop("operandSegmentSizes", segs)
                .prop("static_offsets", offsets)
                .prop("static_sizes", sizes)
                .prop("static_strides", ones)
                .build(out, Some(block));
            views[j] = out.result(s);
        }
        if !tiles(src, op, views, &props, out, block, module_block, cx, ks, map)? {
            return Err("a batch_matmul whose matrices do not suit the kernel".into());
        }
    }
    Ok(true)
}

/// A matmul of `operands` (A, B and C, already in `out`) as tiles computed
/// by kernel `k`, with `props` for a `linalg.generic` of the rows past the
/// last whole tile (whose body is `op`'s); false (and nothing added) where
/// its shapes do not suit the kernel.
#[allow(clippy::too_many_arguments)]
fn tiles(
    src: &Module,
    op: OpId,
    operands: [ValueId; 3],
    props: &[(String, AttrId)],
    out: &mut Module,
    block: BlockId,
    module_block: BlockId,
    cx: &mut Cx<'_>,
    ks: &[&Kernel],
    map: &mut HashMap<ValueId, ValueId>,
) -> Result<bool, String> {
    let set = cx.set;
    let [a, b, c] = operands;
    let (Some((sa, elem)), Some((sb, _)), Some((sc, _))) =
        (out.shaped(out.value_type(a)), out.shaped(out.value_type(b)), out.shaped(out.value_type(c)))
    else {
        return Ok(false);
    };
    let (m, kk, n) = (sa[0], sa[1], sb[1]);
    let (Some(la), Some(lb), Some(lc)) =
        (layout(out, out.value_type(a)), layout(out, out.value_type(b)), layout(out, out.value_type(c)))
    else {
        return Ok(false);
    };
    // The first kernel with a tile no taller than C (and, for one that
    // stores constant weights in another format, B such a constant, in whole
    // panels); rows past its last whole tile go to the next.
    let mut chosen = None;
    for (i, k) in ks.iter().enumerate() {
        if k.tile.0 > m {
            continue;
        }
        if let (Some(w), Some(p)) = (&k.weights, k.pack_b) {
            let k_multiple = if w == "i8" { 64 } else { 32 };
            if !k.whole_rows || n < p || kk % k_multiple != 0 {
                continue;
            }
            match pack(out, b, p, w, module_block, block, cx)? {
                Some(packed) => chosen = Some((i, Some(packed))),
                None => continue,
            }
        } else {
            chosen = Some((i, None));
        }
        break;
    }
    let Some((ki, early)) = chosen else { return Ok(false) };
    let k = ks[ki];
    // The kernel reads rows of B and C: their elements must be contiguous.
    let full = m / k.tile.0 * k.tile.0;
    if full == 0 || kk == 0 || n == 0 || lb.0[1] != 1 || lc.0[1] != 1 || sc != [m, n] {
        return Ok(false);
    }
    // A kernel that transposes A itself reads its rows too.
    if k.transpose_a && (la.0[1] != 1 || !k.whole_rows) {
        return Ok(false);
    }

    // A fill of zeros left out for this product: the kernel's twin whose
    // sums start at zero, and fills of only what loops do; or the fill.
    let zeroed = match (cx.zero.take(), &k.zeroed) {
        (Some(z), Some(_)) if k.whole_rows => Some(z),
        (Some(z), _) => {
            fill(out, block, z, c);
            None
        }
        (None, _) => None,
    };
    let function = match (&zeroed, &k.zeroed) {
        (Some(_), Some(z)) => z.clone(),
        _ => k.function.clone(),
    };

    // The kernel's function, once in the module.
    let fname = match cx.imported.get(&function) {
        Some(n) => n.clone(),
        None => {
            let f = set
                .m
                .top_ops()
                .find(|&f| set.m.op_attr(f, "sym_name").and_then(|a| set.m.string_of(a)).as_deref() == Some(&function))
                .ok_or_else(|| format!("no function @{function} in the kernels"))?;
            out.import_op(&set.m, f, Some(module_block), &mut HashMap::new())?;
            // Before the functions that call it, as upstream places them.
            let ops = &mut out.blocks[module_block.index()].ops;
            let new = ops.pop().unwrap();
            ops.insert(0, new);
            cx.imported.insert(function.clone(), function.clone());
            function.clone()
        }
    };

    let i64t = out.int_type(64);
    let index = out.index_type();
    let ptr = out.dialect_type("llvm.ptr", None);
    let add = |out: &mut Module, spec: OpSpec| -> ValueId {
        let op = spec.build(out, Some(block));
        out.result(op)
    };
    let c64 = |out: &mut Module, v: i64| -> ValueId {
        let attr = out.int_attr(v, i64t);
        add(out, OpSpec::new("llvm.mlir.constant").results(&[i64t]).prop("value", attr))
    };
    let cidx = |out: &mut Module, v: i64| -> ValueId {
        let attr = out.int_attr(v, index);
        add(out, OpSpec::new("arith.constant").results(&[index]).prop("value", attr))
    };

    // A transposed: AT[k, m] = A[m, k], packed; or, for a kernel that
    // transposes A itself, a buffer for a block of rows of it.
    let at_rows = if k.transpose_a { k.tile.0 } else { m };
    let at_ty = out.memref_type(&[kk, at_rows], elem, None);
    let i64a = out.int_attr(64, i64t);
    let segments = out.dense_array_attr("i32", &[0, 0]);
    let at = add(
        out,
        OpSpec::new("memref.alloc").results(&[at_ty]).prop("alignment", i64a).prop("operandSegmentSizes", segments),
    );
    if !k.transpose_a {
        let maps = [Map::parse("(d0, d1) -> (d1, d0)")?, Map::identity(2)];
        let maps: Vec<AttrId> = maps.iter().map(|m| out.affine_map_attr(&m.to_string())).collect();
        let maps = out.array_attr(maps);
        let par = out.dialect_attr("linalg.iterator_type", Some("<parallel>"));
        let iters = out.array_attr(vec![par, par]);
        let segments = out.dense_array_attr("i32", &[1, 1]);
        let g = OpSpec::new("linalg.generic")
            .operands(&[a, at])
            .prop("indexing_maps", maps)
            .prop("iterator_types", iters)
            .prop("operandSegmentSizes", segments)
            .regions(1)
            .build(out, Some(block));
        let region = out.region_of(g, 0);
        let body = out.add_block(region, &[elem, elem]);
        let x = out.blocks[body.index()].args[0];
        OpSpec::new("linalg.yield").operands(&[x]).build(out, Some(body));
    }

    // Each operand's address, as a pointer.
    let pointer = |out: &mut Module, v: ValueId| -> ValueId {
        let i = add(out, OpSpec::new("memref.extract_aligned_pointer_as_index").operands(&[v]).results(&[index]));
        let i = add(out, OpSpec::new("arith.index_cast").operands(&[i]).results(&[i64t]));
        add(out, OpSpec::new("llvm.inttoptr").operands(&[i]).results(&[ptr]))
    };
    // B packed into panels, if the kernel takes it so and B is a constant
    // whose columns are whole panels; else as it is.
    let packed = match k.pack_b {
        _ if early.is_some() => early,
        Some(p) if k.whole_rows && n % p == 0 => pack(out, b, p, "f32", module_block, block, cx)?,
        _ => None,
    };
    let (packed, scales) = match packed {
        Some((v, s)) => (Some(v), s),
        None => (None, None),
    };
    // A packed panel's size in words.
    let bits = match k.weights.as_deref() {
        Some("bf16") => 16,
        Some(_) => 8,
        None => 32,
    };
    let (pat, pc) = (pointer(out, at), pointer(out, c));
    let pa = if k.transpose_a { pointer(out, a) } else { pat };
    let pb = pointer(out, packed.unwrap_or(b));
    let scale_ptr = scales.map(|sc| pointer(out, sc));
    let ldc = c64(out, lc.0[0] as i64);
    let (lda, offa) = if k.transpose_a {
        (c64(out, la.0[0] as i64), c64(out, la.1 as i64))
    } else {
        (c64(out, m as i64), c64(out, 0))
    };
    let (ldb, offb, panel) = match (packed, k.pack_b) {
        (Some(_), Some(p)) => (c64(out, p as i64), c64(out, 0), Some(c64(out, (p * kk * bits / 32) as i64))),
        (None, Some(p)) => (c64(out, lb.0[0] as i64), c64(out, lb.1 as i64), Some(c64(out, p as i64))),
        _ => (c64(out, lb.0[0] as i64), c64(out, lb.1 as i64), None),
    };
    let (kv, offc) = (c64(out, kk as i64), c64(out, lc.1 as i64));
    // A kernel of packed weights does the whole panels; the columns past
    // them are left to a linalg.generic.
    let nk = match (&k.weights, k.pack_b) {
        (Some(_), Some(p)) => n / p * p,
        _ => n,
    };
    let nv = c64(out, nk as i64);

    // for i in 0..full by tile rows, for j in 0..n by tile columns.
    let zero = cidx(out, 0);
    let (rows, cols) = (cidx(out, k.tile.0 as i64), cidx(out, k.tile.1 as i64));
    let (mend, nend) = (cidx(out, full as i64), cidx(out, n as i64));
    let gep = |out: &mut Module, blk: BlockId, p: ValueId, off: ValueId| -> ValueId {
        let f32t = elem;
        let et = out.type_attr(f32t);
        let i32t = out.int_type(32);
        let nw = out.int_attr(0, i32t);
        let raw = out.dense_array_attr("i32", &[i32::MIN as i64]);
        let g = OpSpec::new("llvm.getelementptr")
            .operands(&[p, off])
            .results(&[ptr])
            .prop("elem_type", et)
            .prop("noWrapFlags", nw)
            .prop("rawConstantIndices", raw)
            .build(out, Some(blk));
        out.result(g)
    };
    let int = |out: &mut Module, blk: BlockId, name: &str, x: ValueId, y: ValueId| -> ValueId {
        let i32t = out.int_type(32);
        let flags = out.int_attr(0, i32t);
        let r = OpSpec::new(name).operands(&[x, y]).results(&[i64t]).prop("overflowFlags", flags).build(out, Some(blk));
        out.result(r)
    };
    let outer = OpSpec::new("scf.for").operands(&[zero, mend, rows]).regions(1).build(out, Some(block));
    let r1 = out.region_of(outer, 0);
    let ob = out.add_block(r1, &[index]);
    let i = out.blocks[ob.index()].args[0];
    if k.whole_rows {
        // One call for each block of rows, for all its columns.
        let at_ob = |out: &mut Module, spec: OpSpec| -> ValueId {
            let op = spec.build(out, Some(ob));
            out.result(op)
        };
        let i64i = at_ob(out, OpSpec::new("arith.index_cast").operands(&[i]).results(&[i64t]));
        let rowc = int(out, ob, "llvm.mul", i64i, ldc);
        let offs = int(out, ob, "llvm.add", rowc, offc);
        let tc = gep(out, ob, pc, offs);
        // A's block of rows: its first row, or its first column of AT.
        let ta = if k.transpose_a {
            let rowa = int(out, ob, "llvm.mul", i64i, lda);
            let offs = int(out, ob, "llvm.add", rowa, offa);
            gep(out, ob, pa, offs)
        } else {
            gep(out, ob, pa, i64i)
        };
        let tb = gep(out, ob, pb, offb);
        let callee = out.symbol_attr(&fname);
        let mut args = vec![tc, ldc, ta, lda];
        if k.transpose_a {
            args.push(pat);
        }
        args.extend([tb, ldb, kv, nv]);
        args.extend(panel);
        args.extend(scale_ptr);
        OpSpec::new("func.call").operands(&args).prop("callee", callee).build(out, Some(ob));
        OpSpec::new("scf.yield").build(out, Some(ob));
        if nk < n {
            edge_columns(src, op, [a, b, c], props, out, block, map, full, nk, zeroed)?;
        }
        return rest_rows(
            src,
            op,
            [a, b, c],
            props,
            out,
            block,
            module_block,
            cx,
            &ks[ki + 1..],
            map,
            full,
            at,
            zeroed,
        )
        .map(|()| true);
    }
    let inner = OpSpec::new("scf.for").operands(&[zero, nend, cols]).regions(1).build(out, Some(ob));
    OpSpec::new("scf.yield").build(out, Some(ob));
    let r2 = out.region_of(inner, 0);
    let ib = out.add_block(r2, &[index]);
    let j = out.blocks[ib.index()].args[0];
    let at_ib = |out: &mut Module, spec: OpSpec| -> ValueId {
        let op = spec.build(out, Some(ib));
        out.result(op)
    };
    let i64i = at_ib(out, OpSpec::new("arith.index_cast").operands(&[i]).results(&[i64t]));
    let i64j = at_ib(out, OpSpec::new("arith.index_cast").operands(&[j]).results(&[i64t]));
    // The tile's columns: those of the matrix left, at most a tile's.
    let left = at_ib(out, OpSpec::new("arith.subi").operands(&[nend, j]).results(&[index]));
    let width = at_ib(out, OpSpec::new("arith.minsi").operands(&[left, cols]).results(&[index]));
    let width = at_ib(out, OpSpec::new("arith.index_cast").operands(&[width]).results(&[i64t]));
    let rowc = int(out, ib, "llvm.mul", i64i, ldc);
    let offs = int(out, ib, "llvm.add", rowc, i64j);
    let offs = int(out, ib, "llvm.add", offs, offc);
    let tc = gep(out, ib, pc, offs);
    let ta = gep(out, ib, pa, i64i);
    let offs = int(out, ib, "llvm.add", i64j, offb);
    let tb = gep(out, ib, pb, offs);
    let callee = out.symbol_attr(&fname);
    let mut args = vec![tc, ldc, ta, lda, tb, ldb, kv, width];
    args.extend(panel);
    OpSpec::new("func.call").operands(&args).prop("callee", callee).build(out, Some(ib));
    OpSpec::new("scf.yield").build(out, Some(ib));

    rest_rows(src, op, [a, b, c], props, out, block, module_block, cx, &ks[ki + 1..], map, full, at, zeroed)?;
    Ok(true)
}

/// Columns of C from `from` on, of its first `rows` rows: the original
/// operation on views of A, B and C.
#[allow(clippy::too_many_arguments)]
fn edge_columns(
    src: &Module,
    op: OpId,
    [a, b, c]: [ValueId; 3],
    props: &[(String, AttrId)],
    out: &mut Module,
    block: BlockId,
    map: &mut HashMap<ValueId, ValueId>,
    rows: u64,
    from: u64,
    zero: Option<ValueId>,
) -> Result<(), String> {
    let (sa, elem) = out.shaped(out.value_type(a)).ok_or("an operand of unknown shape")?;
    let (sb, _) = out.shaped(out.value_type(b)).ok_or("an operand of unknown shape")?;
    let (kk, n) = (sa[1], sb[1]);
    // A view of `v`: `nrows` rows from `row`, `ncols` columns from `col`.
    let view = |out: &mut Module, v: ValueId, row: u64, nrows: u64, col: u64, ncols: u64| -> Result<ValueId, String> {
        let (strides, offset) = layout(out, out.value_type(v)).ok_or("an operand of unknown layout")?;
        let layout = strided_attr(out, &strides, offset + row * strides[0] + col * strides[1]);
        let ty = out.memref_type(&[nrows, ncols], elem, Some(layout));
        let segments = out.dense_array_attr("i32", &[1, 0, 0, 0]);
        let offsets = out.dense_array_attr("i64", &[row as i64, col as i64]);
        let sizes = out.dense_array_attr("i64", &[nrows as i64, ncols as i64]);
        let ones = out.dense_array_attr("i64", &[1, 1]);
        let s = OpSpec::new("memref.subview")
            .operands(&[v])
            .results(&[ty])
            .prop("operandSegmentSizes", segments)
            .prop("static_offsets", offsets)
            .prop("static_sizes", sizes)
            .prop("static_strides", ones)
            .build(out, Some(block));
        Ok(out.result(s))
    };
    let va = view(out, a, 0, rows, 0, kk)?;
    let vb = view(out, b, 0, kk, from, n - from)?;
    let vc = view(out, c, 0, rows, from, n - from)?;
    if let Some(z) = zero {
        fill(out, block, z, vc);
    }
    let o = &src.ops[op.index()];
    let mut spec = OpSpec::new("linalg.generic").operands(&[va, vb, vc]).regions(1).loc(o.loc);
    for (key, v) in props {
        spec = spec.prop(key, *v);
    }
    let g = spec.build(out, Some(block));
    let region = out.region_of(g, 0);
    out.clone_region_into(src, o.regions[0], region, map)?;
    Ok(())
}

/// Rows of C past `full`, the last whole tile's end: by the kernels left,
/// else as the original operation on views; then the packed A freed.
#[allow(clippy::too_many_arguments)]
fn rest_rows(
    src: &Module,
    op: OpId,
    [a, b, c]: [ValueId; 3],
    props: &[(String, AttrId)],
    out: &mut Module,
    block: BlockId,
    module_block: BlockId,
    cx: &mut Cx<'_>,
    ks: &[&Kernel],
    map: &mut HashMap<ValueId, ValueId>,
    full: u64,
    at: ValueId,
    zero: Option<ValueId>,
) -> Result<(), String> {
    let o = &src.ops[op.index()];
    let (sa, elem) = out.shaped(out.value_type(a)).ok_or("an operand of unknown shape")?;
    let (sc, _) = out.shaped(out.value_type(c)).ok_or("an operand of unknown shape")?;
    let (m, kk, n) = (sa[0], sa[1], sc[1]);
    let lc = layout(out, out.value_type(c)).ok_or("an operand of unknown layout")?;
    if full < m {
        let rest = m - full;
        let view = |out: &mut Module, v: ValueId, (strides, offset): &(Vec<u64>, u64), cols: u64| -> ValueId {
            let off = offset + full * strides[0];
            let layout = strided_attr(out, strides, off);
            let ty = out.memref_type(&[rest, cols], elem, Some(layout));
            let segments = out.dense_array_attr("i32", &[1, 0, 0, 0]);
            let offsets = out.dense_array_attr("i64", &[full as i64, 0]);
            let sizes = out.dense_array_attr("i64", &[rest as i64, cols as i64]);
            let ones = out.dense_array_attr("i64", &[1, 1]);
            let s = OpSpec::new("memref.subview")
                .operands(&[v])
                .results(&[ty])
                .prop("operandSegmentSizes", segments)
                .prop("static_offsets", offsets)
                .prop("static_sizes", sizes)
                .prop("static_strides", ones)
                .build(out, Some(block));
            out.result(s)
        };
        let va = view(out, a, &layout(out, out.value_type(a)).unwrap(), kk);
        let vc = view(out, c, &lc, n);
        cx.zero = zero;
        let done = tiles(src, op, [va, b, vc], props, out, block, module_block, cx, ks, map)?;
        if let Some(z) = cx.zero.take() {
            fill(out, block, z, vc);
        }
        if !done {
            let mut spec = OpSpec::new("linalg.generic").operands(&[va, b, vc]).regions(1).loc(o.loc);
            for (key, v) in props {
                spec = spec.prop(key, *v);
            }
            let g = spec.build(out, Some(block));
            let region = out.region_of(g, 0);
            out.clone_region_into(src, o.regions[0], region, map)?;
        }
    }
    OpSpec::new("memref.dealloc").operands(&[at]).build(out, Some(block));
    Ok(())
}

/// `b`, if it is a constant (a `memref.get_global` of a constant global
/// of `dense_resource` data, row-major), packed into panels of `panel`
/// columns: panel p holds columns `p panel` to `(p + 1) panel - 1`, k rows
/// of `panel` elements, the panels one after another. A new global, made
/// once for each constant, width and format, got at the end of `block`.
///
/// `format` is `f32`, as B is; or `bf16`, each word of a panel's row two
/// rows of a column rounded to BF16 (row 2i in the bottom half, 2i + 1 in
/// the top), so k/2 rows of words. Other formats are B as bytes.
fn pack(
    out: &mut Module,
    b: ValueId,
    panel: u64,
    format: &str,
    module_block: BlockId,
    block: BlockId,
    cx: &mut Cx<'_>,
) -> Result<Option<(ValueId, Option<ValueId>)>, String> {
    let ValueDef::Result { op: get, .. } = out.values[b.index()].def else { return Ok(None) };
    if out.op_name(get) != "memref.get_global" {
        return Ok(None);
    }
    let Some(Attr::SymbolRef(p)) = out.op_attr(get, "name").map(|a| out.attr(a)).cloned() else { return Ok(None) };
    let name = p[0].clone();
    let ty = out.value_type(b);
    let Some((shape, elem)) = out.shaped(ty) else { return Ok(None) };
    if strided_layout(out, ty).is_some() || shape.len() != 2 || !matches!(out.ty(elem), Type::Float("f32")) {
        return Ok(None);
    }
    let (kk, n) = (shape[0], shape[1]);
    let cache = (name.clone(), panel, format.to_string());
    let new_name = match cx.packed.get(&cache) {
        Some(g) => g.clone(),
        None => {
            // The global, constant, and its data.
            let global = out.blocks[module_block.index()].ops.iter().copied().find(|&g| {
                out.op_name(g) == "memref.global"
                    && out.op_attr(g, "sym_name").and_then(|a| out.string_of(a)).as_deref() == Some(&name)
            });
            let Some(global) = global.filter(|&g| out.op_attr(g, "constant").is_some()) else { return Ok(None) };
            let key = match out.op_attr(global, "initial_value").map(|a| out.attr(out.resolve_attr(a))) {
                Some(Attr::Builtin { keyword: "dense_resource", body, .. }) => {
                    body[1..body.len() - 1].trim().to_string()
                }
                _ => return Ok(None),
            };
            let Some(data) = cx.resources.get(&key) else { return Ok(None) };
            if data.len() as u64 != kk * n * 4 {
                return Ok(None);
            }
            let (rows, cols, w) = (kk as usize, n as usize, panel as usize);
            let mut scale_blob: Option<Vec<u8>> = None;
            let f = |r: usize, c: usize| f32::from_le_bytes(data[(r * cols + c) * 4..][..4].try_into().unwrap());
            let bytes = match format {
                "f32" => {
                    let mut bytes = vec![0u8; data.len()];
                    for p in 0..cols / w {
                        for r in 0..rows {
                            let from = (r * cols + p * w) * 4;
                            let to = ((p * rows + r) * w) * 4;
                            bytes[to..to + w * 4].copy_from_slice(&data[from..from + w * 4]);
                        }
                    }
                    bytes
                }
                "i8" if rows % 4 == 0 => {
                    // Each column's scale: its largest magnitude over 127.
                    let mut scale = vec![0f32; cols];
                    for (c, s) in scale.iter_mut().enumerate() {
                        *s = (0..rows).map(|r| f(r, c).abs()).fold(0.0, f32::max) / 127.0;
                    }
                    let q = |r: usize, c: usize| -> u8 {
                        if scale[c] == 0.0 { 0 } else { (f(r, c) / scale[c]).round().clamp(-127.0, 127.0) as i8 as u8 }
                    };
                    let mut bytes = Vec::with_capacity(data.len() / 4);
                    for p in 0..cols / w {
                        for r in (0..rows).step_by(4) {
                            for c in p * w..(p + 1) * w {
                                bytes.extend([q(r, c), q(r + 1, c), q(r + 2, c), q(r + 3, c)]);
                            }
                        }
                    }
                    // The kernel's row scales are a row's largest magnitude,
                    // not over 127: the column's take that too.
                    let scales: Vec<u8> = scale.iter().flat_map(|s| (s / 127.0).to_le_bytes()).collect();
                    scale_blob = Some(scales);
                    bytes
                }
                "fp8" if rows % 2 == 0 => {
                    // Each column scaled by a power of two, its largest
                    // magnitude at most 448 (E4M3's largest), kept as the
                    // BF16 exponent bias the kernel adds, twice (a word).
                    let mut exps = vec![0i32; cols];
                    let mut biases = Vec::with_capacity(cols * 4);
                    for (c, e) in exps.iter_mut().enumerate() {
                        let m = (0..rows).map(|r| f(r, c).abs()).fold(0.0, f32::max);
                        *e = if m > 0.0 { (m / 448.0).log2().ceil() as i32 } else { 0 };
                        let bias = (((120 + *e) as u16) << 7).to_le_bytes();
                        biases.extend(bias);
                        biases.extend(bias);
                    }
                    let mut bytes = Vec::with_capacity(data.len() / 4);
                    for p in 0..cols / w {
                        for r in (0..rows).step_by(2) {
                            for c in p * w..(p + 1) * w {
                                let scale = (-exps[c] as f32).exp2();
                                bytes.push(e4m3(f(r, c) * scale));
                                bytes.push(e4m3(f(r + 1, c) * scale));
                            }
                        }
                    }
                    scale_blob = Some(biases);
                    bytes
                }
                "bf16" if rows % 2 == 0 => {
                    let mut bytes = Vec::with_capacity(data.len() / 2);
                    for p in 0..cols / w {
                        for r in (0..rows).step_by(2) {
                            for c in p * w..(p + 1) * w {
                                bytes.extend(bf16(f(r, c)).to_le_bytes());
                                bytes.extend(bf16(f(r + 1, c)).to_le_bytes());
                            }
                        }
                    }
                    bytes
                }
                _ => return Ok(None),
            };
            let suffix = if format == "f32" { String::new() } else { format!("_{format}") };
            let new_key = format!("{key}.panels{panel}{suffix}");
            let new_name = format!("{name}_panels{panel}{suffix}");
            // The same global, of the packed data, after the original: of B's
            // type for f32, else of bytes.
            let (shape, elem) =
                if format == "f32" { (shape.clone(), elem) } else { (vec![bytes.len() as u64], out.int_type(8)) };
            cx.blobs.push(Item::Blob { key: new_key.clone(), align: 64, bytes: std::sync::Arc::new(bytes) });
            let tensor = out.tensor_type(&shape, elem);
            let mty = out.memref_type(&shape, elem, None);
            let mty_attr = out.type_attr(mty);
            let value = out.intern_attr(Attr::Builtin {
                keyword: "dense_resource",
                body: format!("<{new_key}>"),
                ty: Some(tensor),
            });
            let sym = out.string_attr(&new_name);
            let mut spec = OpSpec::new("memref.global");
            for (k, v) in dict(out, out.ops[global.index()].properties) {
                spec = match k.as_str() {
                    "initial_value" => spec.prop(&k, value),
                    "sym_name" => spec.prop(&k, sym),
                    "type" => spec.prop(&k, mty_attr),
                    _ => spec.prop(&k, v),
                };
            }
            let g = spec.build(out, Some(module_block));
            let ops = &mut out.blocks[module_block.index()].ops;
            ops.pop();
            let at = ops.iter().position(|&o| o == global).unwrap() + 1;
            ops.insert(at, g);
            if let Some(scales) = scale_blob {
                // The columns' scales, a global of f32 after the panels'.
                let f32t = out.float_type("f32");
                let len = scales.len() as u64 / 4;
                let skey = format!("{new_key}.scales");
                cx.blobs.push(Item::Blob { key: skey.clone(), align: 64, bytes: std::sync::Arc::new(scales) });
                let tensor = out.tensor_type(&[len], f32t);
                let value = out.intern_attr(Attr::Builtin {
                    keyword: "dense_resource",
                    body: format!("<{skey}>"),
                    ty: Some(tensor),
                });
                let mty = out.memref_type(&[len], f32t, None);
                let mty_attr = out.type_attr(mty);
                let sym = out.string_attr(&format!("{new_name}_scales"));
                let mut spec = OpSpec::new("memref.global");
                for (k, v) in dict(out, out.ops[global.index()].properties) {
                    spec = match k.as_str() {
                        "initial_value" => spec.prop(&k, value),
                        "sym_name" => spec.prop(&k, sym),
                        "type" => spec.prop(&k, mty_attr),
                        _ => spec.prop(&k, v),
                    };
                }
                let g = spec.build(out, Some(module_block));
                let ops = &mut out.blocks[module_block.index()].ops;
                ops.pop();
                let at = ops.iter().position(|&o| o == global).unwrap() + 1;
                ops.insert(at, g);
            }
            cx.packed.insert(cache, new_name.clone());
            new_name
        }
    };
    // Of the packed global's type: B's, though not its layout, or bytes.
    // Each global got by its own type: B's, though not its layout, or bytes;
    // and the columns' scales, if there are any.
    let get = |out: &mut Module, name: &str| -> Option<ValueId> {
        let gty = out.blocks[module_block.index()]
            .ops
            .iter()
            .find(|&&g| {
                out.op_name(g) == "memref.global"
                    && out.op_attr(g, "sym_name").and_then(|a| out.string_of(a)).as_deref() == Some(name)
            })
            .and_then(|&g| match out.op_attr(g, "type").map(|a| out.attr(a)) {
                Some(Attr::Type(t)) => Some(*t),
                _ => None,
            })?;
        let sym = out.symbol_attr(name);
        let g = OpSpec::new("memref.get_global").results(&[gty]).prop("name", sym).build(out, Some(block));
        Some(out.result(g))
    };
    let packed = get(out, &new_name).ok_or("the packed global is missing")?;
    let scales = get(out, &format!("{new_name}_scales"));
    Ok(Some((packed, scales)))
}

/// `x` rounded to BF16, to nearest with ties to even, as `bfcvt` does.
fn bf16(x: f32) -> u16 {
    let b = x.to_bits();
    if x.is_nan() {
        return ((b >> 16) | 0x40) as u16;
    }
    ((b + 0x7fff + ((b >> 16) & 1)) >> 16) as u16
}

/// `x` as an E4M3 byte (sign, four bits of exponent biased by 7, three of
/// mantissa), rounded to nearest, at most 448; never zero or subnormal (the
/// kernel's conversion has no case for them), so magnitudes below 2^-6
/// become 2^-6.
fn e4m3(x: f32) -> u8 {
    let sign = if x.is_sign_negative() { 0x80 } else { 0 };
    let v = x.abs().clamp(1.0 / 64.0, 448.0);
    let mut e = v.log2().floor() as i32;
    let mut m = ((v / (e as f32).exp2() - 1.0) * 8.0).round() as i32;
    if m == 8 {
        e += 1;
        m = 0;
    }
    // 448 is the largest: exponent 8, mantissa 6 (7 there is NaN).
    if e > 8 || (e == 8 && m > 6) {
        (e, m) = (8, 6);
    }
    sign | (((e + 7) as u8) << 3) | m as u8
}

#[cfg(test)]
mod tests {
    #[test]
    fn e4m3_bytes() {
        use super::e4m3;
        assert_eq!(e4m3(1.0), 0x38);
        assert_eq!(e4m3(-2.0), 0xC0);
        assert_eq!(e4m3(1.125), 0x39);
        assert_eq!(e4m3(448.0), 0x7E);
        assert_eq!(e4m3(1000.0), 0x7E, "clamped to the largest");
        assert_eq!(e4m3(1.0 / 64.0), 0x08, "the smallest normal");
        assert_eq!(e4m3(0.0), 0x08, "no zero: the smallest normal");
        assert_eq!(e4m3(1.97), 0x40, "rounds up into the next binade");
    }
}
