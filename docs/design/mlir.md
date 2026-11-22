# MLIR

This is the plan for reading MLIR. Epitome would take a `.mlir` file and
turn it into its own IR, so that the optimiser and code generator can
compile it. The goal is to cover two paths that end in MLIR:

- **from PyTorch**: a model exported with `torch.export`, imported by
  torch-mlir, and lowered to `linalg`, `scf`, `arith` and `memref`;
- **from Chisel**: a circuit written as FIRRTL and lowered by CIRCT to
  `hw`, `comb` and `seq`, which Epitome compiles into a cycle-based
  simulator.

The second path matters most. A simulator's `eval` function is one very
large basic block, which Epitome was built to handle (see
[parallel](parallel.md), "Big blocks").

The specification Epitome works from is in [`docs/mlir`](../mlir/README.md),
written from the same kind of public documents as
[`docs/llvm-ir`](../llvm-ir/README.md). For sources and status labels, see
the [index](README.md).

**Status:** in the `mlir` crate (`epitome-mlir`):

- the generic form of any dialect is read and printed (section 4.1), and
  so are the custom forms torch-mlir writes (section 4.2);
- PyTorch models lower in stages, as upstream lowers them, from `torch`
  to `linalg`, buffers, loops and the `llvm` dialect (section 5.6);
- the `llvm` dialect translates to LLVM IR (section 5.1).

The MLP, matrix multiply, convolution and int8 examples compile with
Epitome alone and give PyTorch's results. Arcilator's `llvm` dialect
output translates too, and its GCD simulator runs (section 7.5).
Verification and the hardware dialects are planned.

## 1. Goals

- **Read MLIR text, in its generic form and in the custom forms of the
  dialects we support.** Write it back in the generic form, so that the
  output can always be read again by Epitome and by `mlir-opt`.
- **Lower it to Epitome's IR in Rust, with no MLIR library.** The lowering
  produces a `Module` like a parsed `.ll` file. `epitome_opt::pipeline::run`
  and `epitome_codegen::compile` run on it unchanged.
- **Describe dialects as data, as machines are described.** The meaning of
  an operation is written as a function in LLVM IR. The lowering inlines
  it, as instruction selection matches the bodies of machine instructions
  ([isel](isel.md)). Adding an operation means writing a description entry,
  not Rust.
- **Give the same output for any number of threads.** Functions and modules
  are lowered in parallel, as in [parallel](parallel.md).
- **Stay linear in the size of a region.** A `hw.module` of a large design
  holds hundreds of thousands of operations in one graph region. Nothing
  may take more than linear time in its size.
- **Follow the clean-room rules.** These are in section 9.

Out of scope:

- MLIR's pass infrastructure, pattern rewriter and dialect conversion
  framework. Epitome lowers to its own IR, then optimises that.
- Running MLIR passes that upstream already has, such as bufferization
  and tiling. When a model needs them, upstream `mlir-opt` or torch-mlir
  runs first and Epitome reads what they print (section 5).
- Writing SystemVerilog. `firtool` already does that.

## 2. Where it fits

```text
 PyTorch ─ torch.export ─ torch-mlir ─┐
                                      │  linalg / scf / arith / memref / func
                 upstream mlir-opt ◄──┤  (lowered further upstream, if needed)
                                      ▼
 Chisel ─ FIRRTL ─ firtool ─► hw / comb / seq        llvm dialect
                                  │                       │
                                  ▼                       ▼
                     ┌──────── epitome-mlir ──────────────────┐
                     │ parse ─► verify ─► lower (descriptions) │
                     └───────────────────┬────────────────────┘
                                         ▼
                                 epitome_ir::Module
                                         ▼
                        epitome-opt ─► epitome-llc ─► .o / .s
```

A new crate, `mlir` (`epitome-mlir`), sits beside `ir` and depends only on
it. It has:

| Part | What it does |
| --- | --- |
| `syntax` | the lexer and the parser of the generic form |
| `ir` | operations, regions, blocks, values, types and attributes, in index-based tables like `epitome_ir` |
| `dialect` | the description of each dialect: its types, its custom assembly formats, and its operations' meaning |
| `verify` | the checks every reader needs: dominance, terminators, symbol references, operand and result types |
| `lower` | MLIR to `epitome_ir::Module` |
| `sim` | `hw`, `comb` and `seq` to a cycle-based simulation model |

There are two new tools:

- `epitome-mlir-translate file.mlir -o file.ll`: lower and print LLVM IR.
  It is named after `mlir-translate --mlir-to-llvmir`, and does the same
  for the `llvm` dialect.
- `epitome-mlir-opt file.mlir`: parse, verify and print back. It is for
  tests: round trips, and printing the generic form.

`epitome-llc` and `epitome-opt` also take a `.mlir` file directly, by its
extension. `epitome-rustc-wrapper` is unaffected.

## 3. The in-memory form

MLIR is not LLVM IR with more instructions. An operation may hold regions
of blocks, and the set of operations is open. So the form is generic:

```rust
pub struct Operation {
    pub name: OpName,                 // interned "dialect.op"
    pub operands: Vec<ValueId>,
    pub results: Vec<ValueId>,        // each a local of the enclosing region tree
    pub properties: Option<AttrId>,   // `<{...}>`
    pub attrs: AttrDict,              // `{...}`, sorted by name
    pub successors: Vec<BlockId>,
    pub regions: Vec<RegionId>,
    pub loc: LocId,
}
```

Operations, blocks, regions and values of one top-level operation (a
`func.func`, `llvm.func` or `hw.module`) live in tables owned by it, as a
function's do in `epitome_ir`. So top-level operations can be parsed,
verified and lowered on separate threads. Types, attributes and locations
are interned module-wide, like `epitome_ir`'s types and constants.

**Built:** one set of tables for the whole file, not one per top-level
operation. Splitting them is left until lowering runs in parallel, and
needs to know which operations are isolated from above, which comes with
the dialect descriptions.

Types and attributes are trees:

```rust
pub enum TypeKind {
    Integer { width: u32, signedness: Signedness },
    Index,
    Float(FloatKind),             // f16, bf16, f32, f64, f80, f128, the f8 kinds, tf32
    Complex(TypeId),
    Vector { shape: Vec<u64>, scalable: Vec<bool>, elem: TypeId },
    Tensor { shape: Option<Vec<Dim>>, elem: TypeId, encoding: Option<AttrId> },
    MemRef { shape: Option<Vec<Dim>>, elem: TypeId, layout: Option<AttrId>, space: Option<AttrId> },
    Tuple(Vec<TypeId>),
    Function { inputs: Vec<TypeId>, results: Vec<TypeId> },
    None,
    /// `!dialect.name<...>`, parsed by the dialect's description, or kept as
    /// text if the dialect is unknown.
    Dialect { dialect: Sym, name: Sym, params: Vec<Param> },
    Opaque { dialect: Sym, text: String },
}
```

Attributes have the same shape: the builtin ones (integer, float, string,
array, dictionary, dense elements, dense resources, symbol references,
types, unit, affine maps, integer sets, strided layouts, locations), plus
dialect and opaque ones.

**Epitome note:** an unknown dialect is not an error when reading. Its
operations are kept in their generic form and printed back. It is an
error when lowering, which says which operation it could not lower and
where. This is how `epitome-rustc-wrapper` treats a crate it cannot compile: say
why, and stop.

## 4. Parsing

### 4.1 The generic form

Every operation can be written in the generic form:

```mlir
%0:2 = "dialect.op"(%a, %b) <{prop = 1 : i32}> ({
^bb0(%x: i32):
  "dialect.yield"(%x) : (i32) -> ()
}) {attr = "s"} : (i32, i32) -> (i32, i32) loc("f.py":3:7)
```

The grammar is short and fixed ([language](../mlir/01-language.md)), so
the parser is written by hand, as `epitome_ir`'s is. The lexer returns
slices of the source, as `ir/src/lexer.rs` does, but makes them on
demand from a byte offset rather than all at once: how MLIR text divides
depends on where the parser is. `4x8xf32` in a shape is read as
characters, and the body of a dialect type or attribute
(`!llvm.struct<(i32, ptr)>`) is kept as the text written, its brackets
matched. Value and block names are scoped by region, and a forward
reference left when a region closes moves out to the region around it,
since a graph region may use a value defined later, further out. The
parser also handles:

- aliases for types and attributes (`!t = ...`, `#a = ...`), resolved as
  they are read;
- locations, including `#loc` aliases defined after their use, which need
  a fix-up at the end;
- the file's metadata block `{-# dialect_resources: {...} #-}`, which holds
  the blobs that `dense_resource` attributes name, such as a model's
  weights. These can be hundreds of megabytes, so they are kept as slices
  of the mapped file and decoded only when lowered.

### 4.2 Custom forms

Most tools print custom forms, such as:

```mlir
%2 = arith.addi %0, %1 overflow<nsw> : i32
scf.for %i = %c0 to %n step %c1 iter_args(%acc = %z) -> (f32) { ... }
%r = comb.add bin %a, %b, %c : i8
```

Each operation has its own form. MLIR's operation definitions give it as a
declarative format: a sequence of literals, operand and attribute
directives, `type(...)`, `attr-dict` and optional groups. The format is
published in each dialect's documentation page only by example, so we
write our own format for each operation, in the description (section 6),
in a language close to the declarative one:

```text
arith.addi:  $lhs `,` $rhs (`overflow` `<` $overflowFlags^ `>`)? attr-dict `:` type($result)
```

One engine reads and prints every operation described this way. A few
operations need more than the language, such as `func.func`, `scf.for`,
`hw.module` (its port list) and `linalg.generic`. These have hand-written
Rust parsers in `dialect/custom/`, one file each.

**Not documented:** for most operations the exact custom form is shown
only in examples. Where the examples leave a case open, we run `mlir-opt`
or `firtool` on our own inputs and record what they print in
[observed behaviour](../mlir/06-observed-behaviour.md), as for LLVM.

### 4.3 Verification

The verifier checks what every dialect relies on:

- each block in an SSACFG region ends in a terminator, and definitions
  dominate uses (the dominance code of `opt/src/cfg.rs`, reused through a
  small trait over "a graph of blocks");
- graph regions, such as a `hw.module`'s body, have one block and any
  order of operations;
- operations that are isolated from above, such as `func.func`, use no
  value from outside;
- symbol references resolve within the nearest symbol table;
- each operation's operand and result types match its description.

## 5. Lowering

Lowering goes from the higher dialects down, one level at a time, to
`epitome_ir`. The levels, in build order:

| Level | Dialects | Becomes | Status |
| --- | --- | --- | --- |
| 0 | `builtin`, `llvm` | LLVM IR, one instruction per operation | Done (`translate.rs`) |
| 1 | `func`, `arith`, `math`, `cf`, `index`, `ub` | the `llvm` dialect | Partial: `func`, `arith` (`to_llvm.rs`) |
| 2 | `scf`, `memref`, `affine` (loops and loads), `vector` (the common subset) | the `llvm` dialect, memrefs as bare pointers | Partial: `scf.for`, static `memref`s, `affine.apply` (`to_llvm.rs`) |
| 3 | `linalg` on buffers | loop nests | Done for `linalg.generic` and `linalg.fill` (`linalg_to_loops.rs`), and for named operations once generalized (`linalg_generalize.rs`) |
| 4 | `tensor`, `bufferization`, `linalg` on tensors | buffers | Partial: `tensor.empty`, `tensor.pad`, constants (`bufferize.rs`) |
| 5 | `torch`, `tosa`, `stablehlo` | `linalg` | Partial: `torch`, from descriptions (`torch_to_linalg.rs`) |
| H | `hw`, `comb`, `seq` | a simulation model (section 7) | Planned |
| H+ | `firrtl` | `hw`, `comb`, `seq` | Later: `firtool` does this |

### 5.1 Level 0: the `llvm` dialect

This comes first because it makes Epitome useful at once. Any MLIR
pipeline can end in the `llvm` dialect, using upstream `mlir-opt` with
`--convert-to-llvm` or arcilator's own lowering, and every operation there
is one LLVM IR instruction:

| MLIR | Epitome IR |
| --- | --- |
| `llvm.func @f(%a: i32) -> i32 attributes {...}` | `define i32 @f(i32 %a) #n` |
| `llvm.add %a, %b overflow<nsw> : i32` | `add nsw i32 %a, %b` |
| `llvm.getelementptr %p[%i] : (!llvm.ptr, i64) -> !llvm.ptr, f32` | `getelementptr float, ptr %p, i64 %i` |
| `llvm.mlir.global internal @g(0 : i32) : i32` | `@g = internal global i32 0` |
| `llvm.mlir.addressof @g : !llvm.ptr` | the constant `@g` |
| `llvm.intr.fma(%a, %b, %c) : (f32, f32, f32) -> f32` | `call float @llvm.fma.f32(...)` |
| block arguments | phis in the successor, one incoming value per branch |
| `loc("f.mlir":3:7)` | `!dbg !DILocation(line: 3, column: 7, ...)`, with `-g` |
| `llvm.data_layout`, `llvm.target_triple` on the module | `target datalayout`, `target triple` |

Block arguments are the only real difference from LLVM IR. A branch that
passes `%x` to `^bb2(%a)` adds `[%x, %from]` to a phi at the head of
`^bb2`. When two edges come from the same block with different values (a
`cf.cond_br` to the same block twice), the edge is split with a new block,
as LLVM's phis require one value per predecessor.

**Epitome note:** with level 0 alone, arcilator's output and upstream's
fully lowered pipelines compile. This is the first milestone, and the
first test: the same program through `mlir-translate` and clang, and
through `epitome-mlir-translate` and `epitome-llc`, must give the same
results.

### 5.2 Level 1: scalars and control flow

Each operation in `arith`, `math`, `index` and `ub` has an entry in the
description whose body is the LLVM IR it means (section 6). `func.func`,
`func.call`, `func.return`, `cf.br`, `cf.cond_br`, `cf.switch` and
`cf.assert` are built in, because they make blocks, not values.

The types map as follows:

| MLIR | LLVM IR |
| --- | --- |
| `iN`, `siN`, `uiN` | `iN`. Signedness is used only to choose the operation. |
| `index` | `i64`, or the width from `dlti.dl_spec` if the module has one |
| `f16`, `bf16`, `f32`, `f64`, `f80`, `f128` | `half`, `bfloat`, `float`, `double`, `x86_fp80`, `fp128` |
| the `f8` kinds, `tf32` | **Open.** `i8` storage with calls to a small runtime, or lowering to `float` |
| `complex<T>` | `{ T, T }` |
| `vector<4x8xf32>` | `[4 x <8 x float>]`, as upstream does |
| `vector<[4]xf32>` | `<vscale x 4 x float>` |
| `memref<...>` | a descriptor (section 5.3) |
| `tuple<...>` | a literal struct |

### 5.3 Level 2: structured control flow and memory

- **`scf`** becomes blocks: `scf.for` a rotated loop with its
  `iter_args` as phis, `scf.while` its two regions as blocks,
  `scf.if` a diamond, and `scf.parallel` and `scf.forall` plain nests of
  `scf.for`. **Later:** `scf.parallel` on Epitome's threads, through a
  small runtime.
- **`memref`** values use the descriptor that upstream's `llvm` lowering
  uses, so that code built by Epitome can call code built by upstream and
  the other way round:

  ```llvm
  { ptr, ptr, i64, [N x i64], [N x i64] }   ; allocated, aligned, offset, sizes, strides
  ```

  `memref.load %m[%i, %j]` becomes the address
  `aligned + (offset + i*stride0 + j*stride1) * sizeof(T)` and a `load`.
  Static sizes and strides are constants, so for the common case the
  combiner reduces it to one `getelementptr`. `memref.alloc` calls
  `malloc` (or `aligned_alloc` with an `alignment`), and `memref.alloca`
  is an `alloca`. A function's memref arguments are passed in both of the
  conventions upstream has: the expanded fields, and a pointer to a
  descriptor for `_mlir_ciface_` wrappers (the `llvm.emit_c_interface`
  attribute).
  **Epitome note:** the descriptor is a first-class struct in upstream's
  output, built field by field with `insertvalue`, returned whole, and in
  places stored whole. `epitome-llc` takes none of these today (section
  5.5). Epitome's own lowering keeps each field as a separate value inside
  a function and builds the struct only at a call or return, so the
  optimiser sees plain integers and pointers.
- **`affine`** maps are evaluated as `index` arithmetic: `affine.for`,
  `affine.if`, `affine.load`, `affine.store` and `affine.apply`.
  Nothing polyhedral is done.
- **`vector`**: `transfer_read` and `transfer_write` without masks,
  `broadcast`, `extract`, `insert`, `splat`, `reduction`, `fma`,
  `contract` in its matrix-multiply form, `load`, `store`, `maskedload`
  and `maskedstore`. These map to what [isel](isel.md) already handles for
  LLVM IR vectors.

### 5.4 Levels 3 and 4: `linalg` and tensors

`linalg` operations on buffers become loop nests by reading their
`indexing_maps` and `iterator_types`: one `scf.for` per loop, the maps as
`affine.apply`, the body region inlined. Named operations
(`linalg.matmul`, `linalg.conv_2d_nhwc_hwcf` and so on) are first
rewritten as `linalg.generic` using the maps their documentation gives.

Tensor code needs bufferization first, which is a large analysis
(one-shot bufferize). Epitome does not reimplement it. The supported path
for tensor programs is upstream's, ending in `linalg` on buffers or in
the `llvm` dialect:

```sh
mlir-opt model.linalg.mlir \
  --one-shot-bufferize="bufferize-function-boundaries" \
  --convert-linalg-to-loops -o model.loops.mlir
epitome-llc model.loops.mlir -o model.o
```

**Later:** a simple bufferization of our own, where every tensor value is
a new buffer and copies are removed by the optimiser's `memcpyopt`. It
would be correct and slow, which is enough for tests.

### 5.5 What works already

From the PyTorch chapter's experiments ([from PyTorch](../mlir/04-from-pytorch.md)),
with torch 2.16.0.dev20261020 and torch-mlir 20261001: each model was
exported, lowered by upstream passes through buffers and loops to the
`llvm` dialect, and translated to `.ll`.

| Example | `epitome` reads the `.ll` | `epitome-llc` compiles it |
| --- | --- | --- |
| MLP | yes, with its `f0x` float constants | no: a struct value made by `insertvalue` |
| matmul with bias | yes | no: a struct value made by `insertvalue` |
| conv2d | yes | no: a store of a whole struct |

So the PyTorch path through upstream's `llvm` dialect needs first-class
aggregates in the code generator: `insertvalue` and `extractvalue` on
small structs, and loads, stores and returns of them. These are split into
their fields in `prepare`, before selection, as `wide.rs` splits wide
integers. Upstream's `.ll` also has no target triple or data layout, so
`epitome-llc` must take them from its options, and it calls `malloc` and,
for conv2d, `memrefCopy` from MLIR's C runner library, which a test
links or Epitome provides.

### 5.6 The PyTorch pipeline, as built

Each stage is a pass that builds a new module, in the dialects upstream
uses at that point, so that every stage can be printed, read back, and
checked by upstream's own verifier. `--pass=torch-to-llvm` runs them all
from `torch`; `--pass=linalg-to-llvm` runs them from `linalg` on tensors as
upstream writes it, generalizing its named operations first:

| Pass | Upstream's equivalent | Gives |
| --- | --- | --- |
| `torch-to-linalg` | `torch-backend-to-linalg-on-tensors-backend-pipeline` | `linalg.generic`, `linalg.fill`, `tensor.empty`, `tensor.pad`, `arith.constant` on tensors |
| `linalg-generalize-named-ops` | the same | `linalg.generic` for `matmul`, `batch_matmul`, `transpose`, `broadcast`, `map` and `conv_2d_nchw_fchw` |
| `bufferize` | `one-shot-bufferize{bufferize-function-boundaries}`, `buffer-results-to-out-params` | `memref.alloc`, `memref.global`, `memref.subview`, `memref.copy`, `memref.dealloc`; results become arguments |
| `linalg-to-loops` | `convert-linalg-to-loops` | `scf.for`, `memref.load`, `memref.store`, `affine.apply` |
| `math-polynomial-approximation` | upstream's polynomial approximations (`populateMathPolynomialApproximationPatterns`) | `math.exp` of `f32` (or vectors of them) as `math.fma`, `arith` and `arith.bitcast` |
| `to-llvm` | `lower-affine`, `convert-scf-to-cf`, `convert-to-llvm{use-bare-ptr-memref-call-conv}` | `llvm.func`, `llvm.br`, `llvm.getelementptr`, calls to `malloc` and `free` |

```sh
epitome-mlir-opt model.torch.mlir --pass=torch-to-linalg,bufferize -o model.buf.mlir
epitome-mlir-translate model.torch.mlir --pass=torch-to-llvm -o model.ll
epitome-opt -O2 model.ll -o model.opt.ll && epitome-llc model.opt.ll -o model.o
```

A function `@mlp(tensor<1x4xf32>) -> tensor<1x2xf32>` becomes, in C,
`void mlp(const float *arg0, float *arg1)`: inputs, then a buffer for
each result, each a pointer to packed, row-major elements. Bare pointers
avoid the memref descriptor structs that upstream's default convention
passes and returns (section 5.5).

Where it is simpler than upstream:

- **Bufferization** writes into an output's buffer only when that is the
  output's last use and the buffer is the function's own; otherwise it
  allocates, copying first if the operation reads the output. Upstream's
  analysis finds more chances to write in place, so it allocates less.
- **Every `torch` operation becomes a `linalg.generic`**, where
  torch-mlir uses named operations (`linalg.matmul`,
  `linalg.conv_2d_nchw_fchw`, `linalg.broadcast`). The generic form is
  what upstream's `linalg-generalize-named-ops` gives for them.
- **Shapes are static.** Dynamic dimensions (`torch.bind_symbolic_shape`)
  and views (`aten.view`) are not lowered yet.

The checks (`mlir/tests/torch.rs`):

- each stage's output prints, reads back and prints the same;
- upstream's verifier (torch-mlir's Python bindings) accepts every stage
  for every example (a manual check, as the tests do not need Python);
- the examples, compiled by Epitome alone, linked with a C driver made
  from each function's signature and run, give what PyTorch computes for
  the same weights and inputs (`tests/torch/reference.py`): the MLP,
  matrix multiply with bias, a padded convolution, and an `int8` matrix
  multiply into `int32`, each from `torch` and from torch-mlir's own
  `linalg`; and, from torch-mlir's `linalg` only, a quantize-dequantize
  matrix multiply (`math.roundeven`, `arith.maximumf` and `minimumf`,
  which needed `frintn`, `fmax` and `fmin` in the AArch64 description).

**A language model.** `examples/llm.rs` takes TinyStories-1M, a
GPT-Neo model from Hugging Face (8 layers, 16 heads, a vocabulary of
50,257), from torch-mlir's `linalg` for a window of 64 tokens (480
`linalg.generic`s, 49 `linalg.matmul`s, views, slices, a scan) through
every stage to a Mach-O object, and links it with a Rust driver that
tokenizes a prompt and chooses each next token greedily. The story it
writes is PyTorch's to the token. Reading and lowering the 97 MB of MLIR
takes under 0.1 s; translation, optimisation and code generation about
5 s. It needed, beyond the examples above: views (`tensor.expand_shape`,
`collapse_shape`, `extract_slice`, which share their source's buffer, so
writing in place now checks every tensor that shares it), `tensor.extract`
in bodies (the embedding), `tensor.concat`, `linalg.index`, outputs of more
than one (`aten.max.dim`), torch-mlir's `tm_tensor.scan` (the positions),
`cf.assert`, and `math.rsqrt`, `fpowi`, `exp` and `tanh`; the last two call
`libm`, as Epitome's code generator has no instructions for them. It runs at
2 tokens a second: the driver runs the whole window for each token, and the
loops are scalar and untiled.

`math` and `arith`'s floating-point minimum and maximum become the LLVM
intrinsics of the same meaning (`llvm.intr.roundeven`, `llvm.intr.maximum`).
Dynamic shapes are still not lowered: torch-mlir's `linalg` for a model
with a dynamic batch uses `tensor.dim`.

### 5.7 Parallel and linear

- The lowering of one top-level operation reads only that operation's
  tables, the module's interned tables and the symbol table. Each
  top-level operation is lowered on a thread of its own, using
  `ctx::parallel_phase` as the optimiser does.
- New functions, constants and globals are asked for by name, and
  numbered at the end, in module order, so that the output does not
  depend on timing.
- Lowering is mandatory work, so it asks [autochop](chop.md) for nothing.
  Elective choices made while lowering, such as keeping a `scf.for` as a
  loop rather than unrolling one with a constant trip count of two, are
  steps.

**A larger one.** SmolLM2-135M (Llama: 30 layers, RMSNorm, rotary
embeddings, grouped-query attention; 540 MB of weights, 1.08 GB of MLIR)
needs no operation the examples above did not, and writes PyTorch's
tokens. What it needed was scale:

- the translation keeps a blob of weights as one array of bytes
  (`ConstKind::Bytes`), not an interned constant per element;
- `linalg-fold-constants` transposes constant weights when compiling, as
  upstream's `populateConstantFoldLinalgOperations` does: torch-mlir writes
  each `nn.Linear` as a `linalg.transpose` of the weight, so every call
  transposed all of them. The new weights are blobs a pass made
  (`Item::Blob`, read with `Module::resources`);
- `outline-linalg` makes each `linalg` operation a function of its own,
  named after what it does (`forward.17.matmul`): the optimiser's time on
  the model falls from 8.8 s to 2.2 s, and a profiler can tell them apart.

A window is about 17 GFLOP, nearly all of it matrix products, so the aim
is that little else runs. On an M4 Pro, a forward pass of the window
takes 33 ms (about 520 GFLOP/s, 30 tokens a second), with PyTorch's
tokens:

| Step | ms a window |
| --- | --- |
| the 16-row SME kernel for `matmul`, the rest scalar loops | 625 |
| weights as bytes, not one constant per element | 500 |
| the SME kernel for `batch_matmul` too (attention) | 115 |
| `outline-linalg` keeping scalar constants (`x^2` a multiply, not `powf`) | 96 |
| the kernel 64 columns at a time, so that a weight streams in contiguous 256-byte rows (section 5.8) | 63 |
| `linalg-vectorize=4`, with `exp` as arithmetic (`math-polynomial-approximation`) | 41 |
| results nothing uses dropped (the argmax of softmax's `max`) | 37 |
| vector constants made once outside loops; `exp` with fused multiply-adds | 33 |

`linalg-vectorize=<width>` (`mlir/src/linalg_vectorize.rs`) lowers a
`linalg.generic` on buffers to loops whose innermost one takes `width`
elements at a time, with `vector.load`, `vector.store`,
`vector.broadcast` and the body on `vector<width x f32>`, when the
innermost loop is the last, contiguous dimension of each operand it
indexes. A reduction along it carries up to four vectors of partial results in
the loop (`scf.for` with `iter_args`; one would make each step wait for the
addition before), combines them, and then the lanes
(`vector.reduction`). Upstream vectorizes to n-dimensional vectors and
unrolls them to the native width later; this goes to the native width at
once. What it cannot vectorize, it leaves for `linalg-to-loops`.
`linalg-fold-constants` also drops the results of a `linalg.generic` that
nothing uses, and the work in its body for them, as upstream's
canonicalization does.

The kernel is now about 75% of the time. In a microbenchmark it reaches
1.0 to 1.2 TFLOP/s on weights streamed from memory (30 to 37 GB/s), and
more if a weight is first packed into 64-column panels, each contiguous:
the LM head, whose rows are 196 KB apart, gains half again. What is left
outside it: transposing A for the kernel, the logits' fill and copy, and
the attention's small transposes.

### 5.8 Target kernels: matrix products on SME

`linalg-to-kernels=<target>` replaces a `linalg.generic` that a target has
a kernel for with calls to the kernel, a tile at a time, as upstream lowers
`linalg.matmul` to `arm_sme` outer products. It runs on buffers, after
`bufferize` and before `linalg-to-loops`. What a target has, and the
kernels themselves, are data in `mlir/targets/<target>.mlir`:

```mlir
"epitome.kernel"() <{op = "matmul", element = f32,
    indexing_maps = [affine_map<(d0, d1, d2) -> (d0, d2)>, affine_map<(d0, d1, d2) -> (d2, d1)>,
                     affine_map<(d0, d1, d2) -> (d0, d1)>],
    iterator_types = ["parallel", "parallel", "reduction"],
    body = ["arith.mulf", "arith.addf"],
    tile = array<i64: 16, 16>,
    function = @__epitome_sme_matmul_f32}> : () -> ()

func.func private @__epitome_sme_matmul_f32(%c: !llvm.ptr, %ldc: i64, %at: !llvm.ptr, ...) {
  "llvm.call_intrinsic"() <{intrin = "llvm.epitome.sme.start"}> : () -> ()
  ...  // the tile of C into za0.s; za0.s += column of A x row of B, for each k; za0.s back to C
}
```

A matching `linalg.generic` (the same maps, iterators, element type, and a
body that multiplies the inputs and adds to the output) becomes:

- the first input packed transposed, `AT[k, m]`, by a `linalg.generic`,
  so that the kernel reads a tile's column of A as a contiguous vector;
- two `scf.for`s over the tiles of C, each calling the kernel with
  pointers to the tiles (`memref.extract_aligned_pointer_as_index`, then
  `llvm.inttoptr`), the row strides from the memrefs' layouts, and how many
  of the tile's columns are in C (`arith.minsi`);
- for rows past the last whole tile, the original `linalg.generic` on
  views of A and C, so loops.

The kernel for AArch64 with SME (`aarch64-sme`) is written with Epitome's
SME intrinsics, which the code generator's description implements
(`codegen/targets/aarch64/45-sme.ll`, feature `sme`, CPU `apple-m4`):
`smstart`, `ptrue`, `whilelt` for the last tile's columns, `ld1w` of a row
of C into a horizontal slice of `za0.s`, `fmopa` of a column of A and a row
of B (a recipe: two `ld1w` into fixed `z0` and `z1`, then `fmopa`), `st1w`
back, and `smstop`. The kernel is a function of its own, so streaming mode
covers only code that is legal in it (integer arithmetic and SME), and the
frame saves `d8`-`d15`, which `smstart` zeroes (the entries say
`"clobbers"`). A tile of 32-bit elements is 16 x 16: M4's streaming vector
length is 512 bits.

`__epitome_sme_matmul_f32_64` does 64 rows at a time with all four
tiles of ZA. For each 64 columns of C it takes the four blocks of 16 rows
in turn, each with the tiles side by side (16 x 64 of C): for each k, a
column of 16 from AT and a row of 64 from B, which is 256 contiguous bytes
(`ld1w` of `z4` to `z7`, with `#1, mul vl` to `#3, mul vl`), then four
`fmopa`s (the recipe `sme.fmopa4w`). Streaming a weight from memory, that
keeps four cache lines in flight for each k rather than one; prefetching
(`prfm`) made it slower. Columns past the last 64 are done 16 at a time
with the tiles one above another. Rows past the last 64 go to the 16-row
kernel, then loops.

On TinyStories-1M, `--sme` (`examples/llm.rs`) takes generation
from 2.1 to 32 tokens a second, with the same tokens. `mlir/tests/sme.rs`
checks a 70 x 40 by 40 x 50 product, which has rows past the last whole
tile and a narrow last tile of columns, against a plain one on an M4.

### 5.9 Weights in BF16, int8 and FP8

`linalg-to-kernels=aarch64-sme:<format>` also allows the kernels whose
`epitome.kernel` says `weights = "<format>"`. Such a kernel takes B only
when B is a constant, which the pass converts and packs when compiling
(`pack` in `linalg_kernels.rs`); otherwise the f32 kernels are used, as
they are for attention. The formats round, so they are used only when
asked for (`examples/llm.rs --weights=bf16` builds `llm-bf16`).

| Format | Weights packed as | A, in the kernel | Products |
| --- | --- | --- | --- |
| `bf16` | pairs of rows, rounded to nearest even | rounded to BF16 as it is transposed (`ld2w`, `bfcvt`, `bfcvtnt`, `mova`) | `bfmopa` into f32 |
| `i8` | four rows a word, each column -127..127 times a scale | each row quantised by its largest magnitude (`fmaxv`, `fdivr`, `frintn`, `fcvtzs`, `st1b`) | `smopa` into int32, then scaled into C (`mova`, `scvtf`, `fmul`) |
| `fp8` | E4M3 bytes as BF16's pairs, each column scaled by a power of two | as for `bf16` | each panel made BF16, then `bfmopa` |

The FP8 conversion is the table of (FP8 value, scale) to BF16, worked out:
`(t << 4) + bias + (t & 0x80) * 0xF0`, the bias `(120 + e) << 7` for a
column scaled by 2^e (zero and subnormal codes are never stored). A gather
from a table is not possible here: SVE's gathers fault in streaming mode
on a machine without `FEAT_SME_FA64`, as M4 is, and M4 has no SVE outside
streaming mode.

SmolLM2-135M on an M4 Pro, a window of 64 tokens; perplexity of PyTorch's
own continuation of the prompt, and of a paragraph of other text:

| Weights | ms a window | kernel ms | tokens/s generating | binary | continuation, chosen / perplexity | other text |
| --- | --- | --- | --- | --- | --- | --- |
| f32 | 25.8 | 17.5 | 35.9 | 657 MB | 53 of 53 / 2.461 | 27.37 |
| bf16 | 21.7 | 12.8 | 42.6 | 386 MB | 52 of 53 / 2.458 | 27.38 |
| int8 | 19.1 | 11.0 | 47.4 | 252 MB | 45 of 53 / 2.592 | 37.57 |
| fp8 | 27.5 | 19.0 | 34.0 | 252 MB | 50 of 53 / 2.431 | 28.34 |

What decides it is that M4's SME unit does about the same multiply-adds a
cycle whatever the format: a step of four outer products takes about 4.6
cycles with `fmopa` (1 k), 9.4 with `bfmopa` (2 k) and 14.5 with `smopa`
(4 k), so in cache all reach 1.7 to 2.2 TFLOP/s. Smaller weights only save
memory: streaming them, f32 reaches 1.1 TFLOP/s at 35 GB/s, BF16 1.4 at
22 GB/s, int8 1.5 to 2.0. FP8's conversion costs more than the memory it
saves (it is instructions on the same unit: about 0.2 a weight). Int8's
accuracy suffers from quantising each row of activations to one scale:
Llama's activations have channels far larger than the rest.

Streaming SVE instructions wait for the one before when it is their input,
so the recipes for quantising, scaling and conversion work on four vectors
in registers of their own (z20-z31); int8 went from 32 to 19 ms a window
when its output step did.

### 5.10 From JAX: StableHLO

JAX exports StableHLO (`jax.jit(f).lower(x).compiler_ir("stablehlo")`,
printed in the generic form). `stablehlo-to-linalg`
(`mlir/src/stablehlo_to_linalg.rs`, written from the StableHLO
specification's definitions) takes it to `linalg` on tensors, after which
the pipeline is the one a PyTorch model takes:

| StableHLO | `linalg` on tensors |
| --- | --- |
| element-wise operations (`add`, `multiply`, `exponential`, `logistic`, `tanh`, `maximum`, ...) | a `linalg.generic`, its body from `mlir/dialects/stablehlo.mlir`, data as for `torch` |
| `constant` | `arith.constant`; data over 64 bytes becomes a `dense_resource` blob, as weights are |
| `broadcast_in_dim` | a generic reading the operand through the dimensions it maps to; of a scalar constant, a `linalg.fill` |
| `dot_general` | a generic over batch, free and contracting dimensions, operands transposed first where needed so that a product has `linalg.matmul`'s or `linalg.batch_matmul`'s maps (a constant's transpose is folded away; the SME kernels match) |
| `reduce` (one operand, one combining operation, a constant start) | `linalg.fill` and a generic with reduction iterators |
| `transpose`, `reshape` | a generic; `tensor.collapse_shape` then `tensor.expand_shape` |
| `convert`, `compare`, `select`, `iota` | generics with `arith` casts, `cmpf`/`cmpi`, `select`, `linalg.index` |
| `func.call` of a private function (JAX makes one of `jnp.where`) | inlined |

`examples/jax.rs` compiles an attention block that `examples/jax/export.py`
exports (64 positions: a projection, GELU, a leaky ReLU, four heads, a causal
softmax, an output projection, 32 KB of weights), links it with a Rust
driver and compares it with JAX's own output, which
`mlir/tests/stablehlo.rs` also does from a copy of the export
(`mlir/tests/jax`). On an M4 Pro:

| Built with | Largest difference from JAX (of the largest value) | A call |
| --- | --- | --- |
| loops (NEON) | 4.0e-7 | 2065 us |
| `--sme` | 3.5e-7 | 85 us |
| `--sme --weights=bf16` | 1.6e-3 | 86 us |
| JAX itself (XLA on the CPU, from Python) | | 167 us |

Not yet: `slice`, `concatenate`, `gather`, `scatter`, reductions of several
operands, and dynamic shapes. JAX writes each element-wise operation as its
own; fusing them (upstream's `linalg-fuse-elementwise-ops`) would save
memory traffic for both front ends.

### 5.11 x86-64 with AVX-512 BF16

On x86-64 (`examples/llm.rs` on an x86-64 Linux machine) the code
generator targets AVX2 and FMA, `linalg-vectorize=8`, and `--avx512`
takes matrix products to `mlir/targets/x86-avx512.mlir` (written by
`gen_x86_avx512.py`): BF16 kernels for 4 rows and for 1, which take B
packed as section 5.9's BF16 kernel does. A block of rows of A is rounded
to BF16 into `at` (`vcvtneps2bf16`); for each 64-column panel, the block's
rows x 64 of C stay in registers (16 of the 32 for four rows) while each
row of pairs of the panel is multiplied in: four vectors of B, a pair of A
broadcast for each row (`vpbroadcastd`), and `vdpbf16ps`, which adds both
products of a pair into each lane. A's rows are then interleaved, each step's pairs
side by side, so that one pointer and constant offsets reach every row's;
the inner loop is four loads of B, four broadcasts and sixteen
`vdpbf16ps`, nothing spilled. The kernels' functions alone have AVX-512
(`"target-features"`, in `llvm.passthrough`); the description's 512-bit
class, `zv`, is used only by functions with `avx512f`.

On TinyStories-1M, on one core of an AMD EPYC 9B45 (Zen 5), with the
same tokens at each step for f32 and for BF16 respectively:

| Step | ms a window |
| --- | --- |
| AVX2 loops, f32 | 906 |
| the BF16 kernel for the layers' products | 820 |
| the LM head too: a kernel of packed weights does the whole panels, `linalg.generic` the 17 columns past them | 113 |
| the causal mask (2048 x 2048, compared with zero) worked out only where it is sliced | 95 |
| FMA turned on (it was off, so `math.fma` of vectors was a call of `fmaf` a lane) | 48 |
| IEEE `maximum` and `minimum` of vectors as compares and selects, not `fmaximumf` a lane | 32 |
| a contraction vectorized along a parallel loop when the innermost is not contiguous (attention's) | 21 |
| `tanh` as arithmetic (GELU's), not a call of `tanhf` | 17 |
| the kernel's accumulators in zmm16-31 too, not spilled (the allocator kept each value to its class's registers, so the vector file's 32 could be offered); A's pairs interleaved, one pointer for every row | 14 |
| the logits' fill a memset (`loopidiom`), and vectors with tails for what is not a multiple of the width | 13.4 |
| the logits worked out in the output argument, not a buffer copied there (`bufferize`) | 10.6 |
| no fill: the kernel's twin whose sums start at zero, fills only of the columns loops do | 10.3 |

`linalg-fold-constants` now also works out an element-wise
`linalg.generic` whose result is only sliced (`tensor.extract_slice`, a
slice of a slice) on slices of its inputs. `linalg-vectorize` falls back
to vectors of half its width, and so down to 4, for a dimension that is
not a multiple of the width, and swaps a parallel loop into the innermost
place when that makes every operand it indexes contiguous: a matrix
product's columns rather than its sum, each lane summing its own element
in the order the scalar loops do. `math-polynomial-approximation` expands
`math.tanh`: below 1/4 the odd Taylor series to x^9, above it 1 - 2 /
(e^2|x| + 1), then x's sign.

## 6. Dialect descriptions

A description is written in the dialect it lowers to. `torch` lowers to
`linalg`, so its descriptions are MLIR (`mlir/dialects/torch.mlir`): for
each operation and kind of element, the `linalg.generic` body in `arith`,
the roles of the operation's operands, and its indexing maps when it is
not element by element.

```mlir
// Matrix products: the sum over d2 of a[d0, d2] * b[d2, d1].
"epitome.lowering"() <{op = "torch.aten.mm", element = "float", operands = ["tensor", "tensor"],
    indexing_maps = [affine_map<(d0, d1, d2) -> (d0, d2)>, affine_map<(d0, d1, d2) -> (d2, d1)>,
                     affine_map<(d0, d1, d2) -> (d0, d1)>],
    iterator_types = ["parallel", "parallel", "reduction"]}> ({
^bb0(%a: f32, %b: f32, %acc: f32):
  %p = arith.mulf %a, %b : f32
  %s = arith.addf %acc, %p : f32
  linalg.yield %s : f32
}) : () -> ()
```

`f32` stands for the result's element type, and the pass converts inputs
of other types to it (`arith.extsi`, `arith.sitofp`), so one entry serves
`f16` to `f64`, and an `int` entry `i8` to `i64`. A role says what an
operand is: an input, a constant given to the body (`alpha`), the
output's starting value (a bias), a list read by symbols in the maps
(strides, dilations), a list of padding, a permutation, dimensions to
reduce, or a constant that must have a value (`groups` = 1). Adding an
element-by-element operation is an entry in this file, not Rust.

Where the level below is LLVM IR itself, as for `comb` (section 7), the
plan is descriptions in LLVM IR, as machines are described. Each
operation is a function named after it, whose body says what it
computes:

```llvm
; dialects/arith.ll
define i32 @arith.addi(i32 %lhs, i32 %rhs)
    "format"="$lhs `,` $rhs (`overflow` `<` $overflowFlags^ `>`)? attr-dict `:` type($result)"
    "flags"="overflowFlags" {
  %r = add i32 %lhs, %rhs
  ret i32 %r
}

define i1 @arith.cmpi.slt(i32 %a, i32 %b) "format"="..." {
  %r = icmp slt i32 %a, %b
  ret i1 %r
}

define i32 @arith.ceildivsi(i32 %a, i32 %b) "format"="..." {
  ; the rounding rule from the dialect's documentation, as IR
  ...
}
```

- **Widths.** A body is written for `i32` and stands for every width, as
  the selector's patterns are generic over register classes
  ([isel](isel.md)). The lowering instantiates it at the width it meets:
  `i1`, `i8`, `i64`, `i512`.
- **Attributes.** An attribute that picks a variant, such as `cmpi`'s
  predicate, selects an entry by suffix (`@arith.cmpi.slt`). One that
  adds flags, such as `overflow<nsw>`, names the IR flags it maps to
  (`"flags"`).
- **Checked.** A body is IR, so it can be evaluated. The descriptions are
  checked against `mlir-opt`'s own lowering, when it is installed: random
  inputs through `epitome-mlir-translate`, and through
  `mlir-opt --convert-arith-to-llvm` then `mlir-translate`, must agree.
  This is the same check [isel](isel.md) does for recipes.
- **Overlays.** As with targets ([targets](targets.md)), `--dialect-add`
  reads more description files, so a user can describe their own
  dialect's simple operations without changing Epitome.

Operations that make blocks, regions or memory (`scf`, `memref`,
`func`, `linalg`, `seq`) are Rust in `lower/`. The descriptions cover
operations that compute values, which are most of them.

## 7. Hardware: `hw`, `comb` and `seq`

### 7.1 The model

A circuit is one top `hw.module` with instances of others. Epitome turns
it into a simulation model with the same interface as arcilator's, so that
existing test benches can use either:

- a **state** struct, holding every register, memory and port of the
  flattened design, laid out by Epitome;
- an **`eval`** function, `void top_eval(top_state *s)`, that reads the
  inputs in the state, computes the combinational logic, and on each
  clock edge it sees updates the registers;
- a **header** describing the layout, so that C, C++ or Rust test benches
  can set inputs and read outputs by name.

**Not documented:** arcilator's exact state layout and header format.
They are recorded from its output in
[observed behaviour](../mlir/06-observed-behaviour.md); where we cannot
match them, Epitome writes its own header and says so.

### 7.2 Building `eval`

1. **Flatten** instances into the top module. Each instance's values get
   new ids; ports become plain values. This is linear in the size of the
   flattened design. **Later:** keep instances of large modules used many
   times as functions, to keep code size down.
2. **Order.** A `hw.module` body is a graph region, so its operations may
   appear in any order. They are sorted into a topological order of their
   combinational dependences, with registers' outputs as roots. A cycle that
   does not pass through a register is a combinational loop, which is an
   error naming the operations in it. The sort is Kahn's algorithm,
   linear in the number of operations and uses.
3. **Lower each operation** to `epitome_ir` through the description of
   `comb` and `hw` (section 6), into one block:

   | `comb` | LLVM IR |
   | --- | --- |
   | `add`, `mul`, `sub`, `and`, `or`, `xor` (variadic) | a chain of the binary operation |
   | `divu`, `divs`, `modu`, `mods` | `udiv`, `sdiv`, `urem`, `srem`, guarded: arcilator divides by 1 when the divisor is 0 (observed), and Epitome does the same |
   | `shl`, `shru`, `shrs` | `shl`, `lshr`, `ashr`, guarded: an amount of the width or more gives 0, or the sign bits for `shrs` (observed) |
   | `icmp` (`eq`, `ne`, `slt`, `ult`, ...) | `icmp` |
   | `extract %x from 4 : (i8) -> i2` | `lshr` then `trunc` |
   | `concat` | `zext`, `shl`, `or` |
   | `replicate` | a constant multiply of the zero-extended value, or `concat` |
   | `mux` | `select` |
   | `parity` | `llvm.ctpop` then `and 1` |
   | `hw.array_get`, `hw.array_create`, `hw.struct_extract`, `hw.struct_create`, `hw.bitcast` | `extractvalue` and `insertvalue` on arrays and structs, or shifts on the packed bits |

   `comb` is two-state and has no poison, so the guards in the table are
   what keep the LLVM IR defined. The combiner removes them where the
   amount is a constant.
4. **Registers.** `seq.compreg` and `seq.firreg` read their current value
   from the state at the top of `eval`. Their next value is computed in
   the block, and written back when the clock rose since the last call:
   `(clk & ~old_clk)`, with the old clock kept in the state. Reset is a
   `select` on the next value. Writing back happens after every register's
   next value is computed, so registers update together.
5. **Memories** (`seq.firmem`) are arrays in the state. Reads are loads;
   writes are stores at the end, with their enables, like registers.

The result is one function with one very large block, and a few
branches for clock edges. That is the shape [parallel](parallel.md) and
`opt/tests/scaling.rs` test for.

### 7.3 Wide integers

Hardware values are often wider than 128 bits: a 512-bit bus, a
1024-bit cache line. `codegen/src/wide.rs` splits integers of 65 to 128
bits into two `i64` halves. Hardware needs any width:

- **Generalise `wide.rs` to `N` words.** `add` and `sub` carry through
  the words; `and`, `or`, `xor` and `select` act word by word; shifts by a
  constant move words and bits; `icmp` compares from the top word;
  `extract` and `concat` with constant positions become word moves and
  shifts. These are linear in the number of words.
- **Shifts by a variable amount, multiplication and division** call a
  small runtime (`__epitome_shl_n`, `__epitome_mul_n`, ...) on arrays of
  words above 128 bits. They are rare in data paths.
- Before that, in the optimiser: **narrow** each value to the bits it
  uses. `comb.extract` of the low 8 bits of a 512-bit `add` needs only an
  8-bit add. This is the combiner's known-bits work, run backwards
  (demanded bits). It is the biggest single saving for simulation.

### 7.4 Odd widths

Hardware is full of integers that are not 8, 16, 32 or 64 bits wide.
Arcilator's output for the examples (section 8) has `zext` to `i10` to
`i13`, and loads and stores of `i4`, `i12`, `i20` and `i24` in its state
buffer. `epitome-llc` stops on both today:

```text
epitome-llc: @Fir_eval: no instruction for `Cast { op: ZExt, ... }`
epitome-llc: loads and stores of odd-sized integers are not supported yet
```

These are fixed in `prepare`, before selection, by legalising types as
the LangRef defines them:

- an `iN` value of 1 to 64 bits lives in the smallest register class that
  holds it, with the bits above `N` undefined, as `wide.rs` already does
  for the high half of an `i65` to `i127`;
- `zext` and `sext` from `iN` clear or copy the bits above `N` (an `and`
  with a mask, or a pair of shifts), and `trunc` to `iN` does nothing;
- a load of `iN` loads its store size (the bytes `N` rounds up to, as the
  data layout says) and a store of `iN` stores the value zero-extended to
  that size. A store size of 3, 5, 6 or 7 bytes is split into
  power-of-two pieces.

This is the first piece of work for the hardware path. It needs no MLIR
reader, and it lets every arcilator example compile.

### 7.5 What works already

From the hardware chapter's experiments
([hardware](../mlir/05-hardware.md)), with firtool 1.162.0 and Chisel
7.16.0:

| Example | `epitome` reads the `.ll` | `epitome-llc` compiles it | Runs |
| --- | --- | --- | --- |
| counter | yes | yes | |
| GCD | yes | yes | `gcd(1071, 462) = 21 after 12 cycles`, as with clang |
| SystemVerilog counter, through `circt-verilog` | yes | yes | |
| 4-tap and 64-tap FIR | yes | no: `zext` to an odd width | |
| 16 x 32 memory, every FIRRTL primitive, every `comb` operation | yes | no: odd-width loads and stores | |

The 64-tap FIR's `eval` has 690 instructions in 8 blocks, the largest of
354. Larger designs make larger blocks, as section 7.2 expects.

### 7.6 What to measure

Simulation speed is what a user sees. The comparisons are:

- Epitome against arcilator's own `.ll` compiled by clang `-O2`, and
  arcilator's `.ll` compiled by Epitome (level 0), on the same design;
- against Verilator, on the SystemVerilog `firtool` writes;
- compile time against the number of operations, which must stay
  linear, from one 8-bit counter to a design of a million operations,
  generated by repeating an array of multiply-accumulate cells.

## 8. Examples

[`docs/mlir/examples`](../mlir/examples) holds real inputs, each produced
by the tools named beside it, with their versions:

| Example | Path | Tests |
| --- | --- | --- |
| `pytorch/`: an MLP, a matmul with bias, a conv2d; also an int8 matmul, a quantised matmul, a dynamic batch | PyTorch → `torch` → `linalg` on tensors (also `tosa`, `stablehlo`) → (upstream) buffers → loops → `llvm` → `.ll` | levels 0 to 3 |
| `chisel/`: a counter, GCD, a 4-tap and a 64-tap FIR filter, a 16 x 32 memory | Chisel → FIRRTL → `firrtl` → `hw`/`comb`/`seq` → SystemVerilog, and (arcilator) `arc` → `llvm` → `.ll` | levels 0 and H |
| `firrtl/primops/`, `comb/` | every FIRRTL primitive operation, and every `comb` operation, lowered | the description of `comb` (section 6) |
| `verilog/counter/` | SystemVerilog → `circt-verilog` → `moore` → `hw` | level H, from Verilog |

Each becomes a test in `mlir/tests/`, in the style of
`opt/tests/examples.rs`:

1. **Round trip**: parse and print the generic form; parsing that again
   gives the same.
2. **Lower and run**: `epitome-llc` the file, link a small C driver, run,
   and compare with the expected output, and with the upstream tools'
   result where they are installed.
3. **Threads**: 1 thread against 8 gives the same output.
4. **Scaling**: the generated arrays of cells, as in
   `opt/tests/scaling.rs`.

## 9. Clean-room rules for MLIR

The [rules](README.md) for LLVM apply to MLIR, CIRCT and torch-mlir in the
same way:

- **Allowed:** the published documentation (mlir.llvm.org/docs, including
  the dialect pages generated from operation definitions;
  circt.llvm.org/docs; the torch-mlir `docs/` pages; pytorch.org's
  documentation; the FIRRTL specification; chisel-lang.org), published
  papers, and the output of running the tools on our own inputs.
- **Not allowed:** the C++ sources, the TableGen operation definitions,
  and the `.mlir` test files of llvm-project, CIRCT and torch-mlir. The
  test files are source, even though they are written in MLIR.
- Where the documents leave something out, it is marked **Not documented**
  in [`docs/mlir`](../mlir/README.md), and decided by observing the tools
  or by Epitome for itself.

The tools are run only to see what they print. Their output on our own
inputs, such as the examples in section 8, may be kept in the repository.

## 10. Build order

0. **Done for PyTorch:** the stages of section 5.6 and the translation of
   section 5.1 compile the PyTorch examples with Epitome alone, and
   their results match PyTorch's. Next: views and dynamic shapes, more
   `torch` operations (each an entry in `torch.mlir`), and reading the
   custom forms of `linalg`, `tensor` and `memref`, so that upstream's
   own intermediate files can be read.
1. **What upstream's `.ll` needs from the code generator**, before any
   MLIR is read: odd integer widths (section 7.4) and first-class
   aggregates (section 5.5). Milestone: every `.ll` in
   `docs/mlir/examples` compiles and runs, from arcilator and from
   torch-mlir.
2. **The generic form.** `ir`, `syntax`, `verify` and `epitome-mlir-opt`:
   parse and print the generic form of any dialect, with round-trip tests
   on every example. **Partial:** parsing, printing and the tool are done
   (`mlir/src`), and the 74 examples, printed generically with and without
   locations by `circt-opt` and torch-mlir, round-trip exactly
   (`mlir/tests/corpus`); `verify` is not started.
3. **Level 0, the `llvm` dialect**, and `epitome-mlir-translate`. With
   it, arcilator's output and fully lowered upstream pipelines compile.
   Milestone: the Chisel examples simulate from arcilator's `llvm`
   dialect output, with no `mlir-translate`. **Done:** `translate.rs`; arcilator's
   GCD runs, and the translation of every example matches the tools'
   `.ll` in its functions, globals and instructions.
4. **Description files**, and the format engine for custom forms. Level 1:
   `func`, `arith`, `math`, `cf`, `index`, with the check against
   upstream's lowering.
5. **Level 2:** `scf`, `memref`, `affine`, `vector`. Milestone: the
   PyTorch examples run, after upstream bufferization and
   `--convert-linalg-to-loops`.
6. **Level H:** `hw`, `comb`, `seq`, flattening, ordering and the state
   model. Milestone: the Chisel examples simulate with no arcilator, and
   match it.
7. **Wide integers** of any width, and narrowing by demanded bits.
   Measured against arcilator and Verilator on the generated arrays.
8. **Level 3:** `linalg` on buffers. Then, if wanted, level 4 and
   `firrtl`.

## 11. Open questions

- **Bytecode.** MLIR also has a binary form. torch-mlir and firtool write
  text by default, so text comes first. The container is documented
  ([language](../mlir/01-language.md)); the encoding of each dialect's
  attributes and types is not.
- **Four-state logic.** `comb` is two-state. The `moore` and `llhd`
  dialects, from circt-verilog, have `x` and `z`. Supporting them means
  two bits per bit and a different description of each operation.
- **Should the optimiser understand `comb` directly?** Lowering first
  loses nothing for the operations above, since each is a short sequence
  of LLVM IR. It may matter for very wide values, where narrowing is
  easier before splitting into words. Narrowing is planned on LLVM IR
  first (section 7.3), and this is decided by what it misses.
- **Floating-point formats with no LLVM type**, the `f8` kinds among
  them, for the PyTorch path.
