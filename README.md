# Epitome of synaptic excellence

Epitome of synaptic excellence (Epitome for short) is an alternative code
generation tool, written in Rust, that takes LLVM IR as its input. It is
not a clone of LLVM: it reads and writes the same IR, and its tools take
the same kinds of input and give the same kinds of output, but how it gets
there is its own.

**256 crates' own test suites pass with their code compiled by Epitome**
(on an AArch64 Mac; see [Crates that build](#crates-that-build)).

> **Experimental.** Epitome is young and changes quickly, and it is
> suitable only for the adventurous at the moment: expect missing features,
> rough edges and bugs, and check its output before relying on it.

- **Its own passes and algorithms.** The pass structure differs from
  LLVM's. Much of what LLVM does in separate passes (instcombine,
  reassociate, DSE, parts of GVN and memcpyopt) is one rule-driven
  combiner here, with its rules in a small language
  (`opt/rules/combine.rules`), and memory dependences are found on demand
  rather than kept up to date.
- **Data-driven code generation.** A machine is a description written as
  LLVM IR: each instruction is a function whose body says what it
  computes, with its encoding and assembly as attributes. Selection,
  encoding, assembly, disassembly and the checks are all made from it, so a
  new machine or CPU is a set of description files, not new code
  (`codegen/targets/`).
- **Multithreaded throughout.** Optimisation runs on many functions at
  once, in call-graph order, and so does code generation; the output is
  the same for any number of threads.
- **Large basic blocks.** Special attention is paid to compiling very large
  basic blocks, such as those generated from SystemVerilog by simulators:
  every pass is kept close to linear in a block's size, with budgets on the
  searches that could grow, and tests check that time grows in proportion
  as blocks and functions grow (`opt/tests/scaling.rs`,
  `codegen/tests/parallel.rs`).

## Clean-room policy

Epitome is built only from public specifications and documented behaviour,
such as the LLVM Language Reference, the IR and bitcode format documentation,
and the observable behaviour of the tools. No LLVM source code is read,
copied or translated into this project.

## Building

```sh
cargo build --release --workspace
cargo test --release --workspace
```

The tests in `opt/tests/programs.rs` and `opt/tests/examples.rs` also build
and run their output with `clang` when it is installed.

## On x86-64 Linux

Everything below was run on an x86-64 Linux machine (AMD EPYC, Zen 5) with
clang and Rust installed. Build first, in this directory, then work in a
scratch one:

```sh
cargo build --release --workspace --bins --examples
EPI=$PWD E=$PWD/target/release
mkdir -p /tmp/epitome-demo && cd /tmp/epitome-demo
```

### Hello, world in C

clang makes the IR (unoptimised, so that Epitome's optimiser does the
work), and Epitome optimises it and compiles it to an ELF object, which
clang links:

```sh
cat > hello.c <<'C'
#include <stdio.h>

int main(void) {
    printf("Hello, world!\n");
    return 0;
}
C
clang -O1 -Xclang -disable-llvm-passes -S -emit-llvm hello.c -o hello.ll
$E/epitome-opt -O2 hello.ll -o hello.opt.ll
$E/epitome-llc --target=x86_64 hello.opt.ll -o hello.o    # or -o hello.s
clang hello.o -o hello && ./hello
```

### Hello, world in Rust

`epitome-rustc` compiles each crate's code in place of LLVM (see
[Building Rust crates with Epitome](#building-rust-crates-with-epitome)):

```sh
cargo new hello-rs && cd hello-rs
RUSTC_WRAPPER=$E/epitome-rustc cargo run --release
cat target/release/epitome-rustc.log    # hello_rs: ours (binary)
cd ..
```

### A PyTorch model

`docs/mlir/examples/pytorch` has small models as torch-mlir imports them.
Epitome lowers one through every stage (`torch` to `linalg`, buffers,
loops, the `llvm` dialect) to LLVM IR, then optimises and compiles it. The
model's tensors become plain pointers, its result the last:

```sh
P=$EPI/docs/mlir/examples/pytorch
$E/epitome-mlir-opt $P/matmul_bias.torch.mlir --pass=torch-to-linalg,bufferize   # see a stage
$E/epitome-mlir-translate $P/matmul_bias.torch.mlir --pass=torch-to-llvm -o matmul_bias.ll
$E/epitome-opt -O2 matmul_bias.ll -o matmul_bias.opt.ll
$E/epitome-llc --target=x86_64 matmul_bias.opt.ll -o matmul_bias.o
cat > main.c <<'C'
#include <stdio.h>

// The model: out = a (2x3) @ b (3x4) + bias (4).
void matmul_bias(const float *a, const float *b, const float *bias, float *out);

int main(void) {
    float a[6] = {1, 2, 3, 4, 5, 6};
    float b[12] = {1, 0, 0, 1, 0, 1, 0, 1, 0, 0, 1, 1};
    float bias[4] = {10, 20, 30, 40}, out[8];
    matmul_bias(a, b, bias, out);
    for (int i = 0; i < 8; i++)
        printf("%g%s", out[i], i % 4 == 3 ? "\n" : " ");
    return 0;
}
C
clang main.c matmul_bias.o -o matmul_bias && ./matmul_bias   # 11 22 33 46 / 14 25 36 55, as PyTorch
```

A whole language model from Hugging Face, exported from PyTorch as MLIR
(with torch-mlir's Python environment and `transformers`), and compiled
into an executable with a Rust driver. Without a flag, matrix products are
loops vectorised for AVX2 and FMA; `--avx512 --weights=bf16` uses the
AVX-512 BF16 kernels of `mlir/targets/x86-avx512.mlir`:

```sh
cd $EPI
python examples/llm/export.py target/tinystories          # roneneldan/TinyStories-1M
cargo run --release --example llm -- target/tinystories
target/tinystories/llm "Once upon a time, there was a little dog called"
target/tinystories/llm --bench 5                          # 80 ms a forward pass
cargo run --release --example llm -- target/tinystories --avx512 --weights=bf16
target/tinystories/llm-bf16 --bench 5                     # 10.4 ms a forward pass
```

### A JAX model

`examples/jax/export.py` exports a block of a model written in JAX
(attention over 64 positions with four heads, GELU and a causal softmax)
as StableHLO, with an input and JAX's output. `examples/jax.rs` takes it
from StableHLO to machine code in Epitome and links a driver that checks
the result against JAX's:

```sh
cd $EPI
pip install "jax[cpu]"
python examples/jax/export.py target/jax
cargo run --release --example jax -- target/jax
target/jax/jax              # largest difference from JAX 7.2e-7; 414 us a call
cargo run --release --example jax -- target/jax --avx512 --weights=bf16
target/jax/jax-bf16         # BF16 weights: 1.9e-3; 376 us a call
```

## Status

What each target can do:

- **AArch64** (macOS Mach-O, and ELF): the main target. It compiles C,
  C++ with exceptions, Rust (256 crates' own test suites pass), SQLite,
  and the PyTorch and JAX examples, with NEON and SME kernels (Apple M4).
- **x86-64** (Linux ELF): compiles C, C++ with exceptions, Rust, SQLite
  and the PyTorch and JAX examples, with SSE2, AVX2 and FMA, AVX-512 BF16
  kernels and the x87 for `long double`. It assembles and disassembles
  every XED form llvm-mc 18 takes.
- **RISC-V RV64** (ELF): compiles scalar code (RV64IMFD and Zbb). It
  assembles and disassembles 866 of the 871 RV64 instructions, C and V
  included, and runs riscv-tests on QEMU.
- **AMD R600** (Evergreen): assembly and disassembly only, for ALU
  instructions, control flow, a memory export and a vertex fetch.
- **AMD gfx1151** (RDNA 3.5): assembly and disassembly only, of everything
  but images. It agrees with llvm-mc 22 on 34,597 lines.

Optimisations in `epitome-opt` (each call-graph SCC bottom-up, on many
threads; `opt/src/pipeline.rs`):

- `sroa` and `mem2reg` (allocas to SSA values), again after inlining;
- inlining with a cost model, and recursion removal (self-recursion to
  loops or an explicit stack; tail calls eliminated);
- the combiner (`opt/rules/combine.rules`), which does the work of
  instcombine, DCE, DSE, reassociate, store-to-load forwarding over memory
  SSA, and parts of bdce, aggressive-instcombine and globalopt;
- constant calls folded (part of ipsccp), and simplifycfg;
- loop rotation, LICM (no promotion of memory to registers), induction
  variables (exit values, widening and trip counts, without SCEV), loop
  idioms to `memset`, and strength reduction (`loop-reduce`, in part);
- early CSE and GVN (no partial redundancy elimination), memcpyopt (store
  runs to `memset`), tail-call marking, and dead internal functions
  removed.

Optimisations in `epitome-llc`:

- selection by the longest, cheapest match in a trie built from the
  description;
- address sinking (part of codegenprepare), constants and addresses used
  again within a block and hoisted out of loops, post-increment loads and
  stores, and division by constants without a divide;
- linear-scan register allocation with copy hints and eviction by weight,
  then copy propagation;
- peepholes: frame addresses, redundant compares, load and store pairs
  (`ldp`/`stp`), and single-bit branches (`tbz`);
- block placement with branch inversion, branch relaxation, part of the
  branch folder, and tail calls as jumps.

## Crates

- `epitome` (the root package): `epitome-rustc`, which builds Rust crates
  with Epitome (see below), and `examples/factorial.rs`.
- `epitome-rustc2`: a Rust compiler that does not use rustc. Parsing,
  macro expansion, name resolution and type checking work, and run on
  `core`, `alloc` and `std` in 1.3 s; lowering to Epitome IR is next. See
  `epitome-rustc2/README.md`.
- `ir` (`epitome-ir`): the IR, with a parser and printer for `.ll` files,
  and builders for making it in code (`epitome_ir::build`).
- `opt` (`epitome-opt`): optimisation passes, run on many functions in
  parallel. So far: the call graph, an inliner with a cost model,
  recursion removal, and a rule-driven combiner whose rules are in
  `opt/rules/combine.rules`. See `docs/design/parallel.md` and
  `docs/design/combine.md`.
- `codegen` (`epitome-codegen`): code generation from machine
  descriptions written in LLVM IR: instruction selection, register
  allocation, frames, and assembly, object files (Mach-O and ELF) and
  disassembly; `epitome-as` assembles, and `epitome-objdump`
  disassembles. AArch64 and x86-64 (`codegen/targets/aarch64/`,
  `codegen/targets/x86_64/`) are built in; other machines and CPUs are
  description files read at run time, such as RISC-V RV64
  (`codegen/targets/riscv64/`). See `docs/design/isel.md`,
  `docs/design/targets.md` and `docs/design/objects.md`.

- `mlir` (`epitome-mlir`): MLIR. A parser and printer for the generic
  form of any dialect and the custom forms torch-mlir writes; passes that
  lower a PyTorch model, as torch-mlir imports it, in upstream's stages
  (`torch` to `linalg`, buffers, loops and the `llvm` dialect); and the
  translation of the `llvm` dialect to LLVM IR. `epitome-mlir-opt` runs
  passes and prints MLIR; `epitome-mlir-translate` writes LLVM IR. See
  `docs/design/mlir.md` and the specification in `docs/mlir`.

## Running

```sh
# Build a factorial function and a main in code, optimise and compile them,
# then link and run the program (needs clang).
cargo run --release --example factorial -- --run

# Run a Rust crate's tests with its code compiled by Epitome (from this
# directory; see "Building Rust crates with Epitome").
cargo build --release --bin epitome-rustc
W=$PWD/target/release/epitome-rustc
(cd /path/to/some-crate && RUSTC_WRAPPER=$W cargo test)

# Read a .ll file and print it back.
cargo run --release -- file.ll

# Inline and remove recursion, using every core, explaining each decision.
cargo run --release -p epitome-opt -- -O2 --remarks --print-cycles --stats file.ll
```

```sh
# Compile to AArch64 assembly (Darwin), or show the machine code.
cargo run --release -p epitome-codegen --bin epitome-llc -- file.ll -o file.s
cargo run --release -p epitome-codegen --bin epitome-llc -- file.ll --disasm

# Compile to an object file (Mach-O here; ELF with --abi=aapcs64), link it
# and run it, and disassemble it with its relocations.
cargo run --release -p epitome-codegen --bin epitome-llc -- file.ll -o file.o
clang file.o main.c -o program && ./program
cargo run --release -p epitome-codegen --bin epitome-objdump -- -dr file.o

# Assemble assembly (epitome-llc's) into an object file.
cargo run --release -p epitome-codegen --bin epitome-as -- file.s -o file.o

# Another machine, from its description's files; with a CPU and features.
cargo run --release -p epitome-codegen --bin epitome-llc -- \
    --target=codegen/targets/riscv64 --mattr=+zbb file.ll -o file.o
cargo run --release -p epitome-codegen --bin epitome-llc -- --mcpu=apple-m1 --mattr=-neon file.ll

# Or by name, from the directories in EPITOME_TARGET_PATH (and objdump
# finds a description there by the object's ELF machine).
export EPITOME_TARGET_PATH=$PWD/codegen/targets
cargo run --release -p epitome-codegen --bin epitome-objdump -- -d file.o
```

```sh
# A PyTorch model, as torch-mlir imports it, lowered stage by stage and
# printed as MLIR, then compiled with Epitome alone.
cargo run --release -p epitome-mlir --bin epitome-mlir-opt -- \
    docs/mlir/examples/pytorch/mlp.torch.mlir --pass=torch-to-linalg,bufferize
cargo run --release -p epitome-mlir --bin epitome-mlir-translate -- \
    docs/mlir/examples/pytorch/mlp.torch.mlir --pass=torch-to-llvm -o mlp.ll
cargo run --release -p epitome-codegen --bin epitome-llc -- mlp.ll -o mlp.o
```

```sh
# A language model from Hugging Face, exported from PyTorch as MLIR,
# compiled through every stage by Epitome into a Mach-O executable with a
# Rust driver (needs torch-mlir's Python environment with transformers for
# the export, and an AArch64 Mac). SmolLM2-135M (Llama, 30 layers) or
# TinyStories-1M (GPT-Neo, 8 layers); --sme puts matrix products on SME
# (Apple M4).
python examples/llm/export.py --model HuggingFaceTB/SmolLM2-135M target/smollm2
cargo run --release --example llm -- target/smollm2 --sme
target/smollm2/llm "Once upon a time, there was a little dog called"
```

`epitome-opt --help` lists the options. `opt/tests/examples` has small
inputs with their expected output.

## Building IR in code

`epitome_ir::build` has chained builders for functions and globals
(`Function::builder`, `Global::builder`), and a `FunctionBuilder` that adds
instructions at the end of the current block, working out result types and
checking operands. Instructions with settings are chained too
(`build_call`, `build_load`, `build_store`, `build_gep`, `build_binary`):

```rust
use epitome_ir::module::{BinOp, Function, ICmpPred, Linkage, Module};

let mut m = Module::default();
let i64t = m.types.int(64);
let mut b = Function::builder("max")
    .returns(i64t)
    .param("a", i64t)
    .param("b", i64t)
    .linkage(Linkage::Internal)
    .attr("nounwind")
    .begin(&mut m);
let (x, y) = (b.param(0), b.param(1));
b.block("entry");
let gt = b.icmp(ICmpPred::Sgt, x, y);
let r = b.select(gt, x, y);
let r1 = b.build_binary(BinOp::Add, r, x).nsw().build();
b.ret(r1);
b.finish()?; // every block must end in a terminator
```

There are plain constructors as well (`Types::function`, `Constant::int`,
`Call::new`, `Global::variable`, `Module::declare`, `Module::const_int`).
The module can then go to `epitome_opt::pipeline::run` and
`epitome_codegen::compile` like a parsed one. `examples/factorial.rs`
builds a factorial function and a `main` that prints some factorials,
optimises and compiles them, and with `--run` links and runs the program:

```sh
cargo run --release --example factorial -- --run
```

## Building Rust crates with Epitome

`epitome-rustc` (in this package) is a `RUSTC_WRAPPER`: Cargo runs it with
each rustc command line. It runs rustc as asked, also writing the crate's
LLVM IR in one codegen unit, compiles that IR with Epitome, and puts our
object in place of rustc's: in the `.rlib` for a library, and at the link
(it is given as rustc's linker) for a binary or test harness.

```sh
cargo build --release --bin epitome-rustc
cd some-crate
RUSTC_WRAPPER=/path/to/epitome/target/release/epitome-rustc cargo test
cat target/debug/epitome-rustc.log   # which crates are ours, and why not
```

Rust's standard library comes built and stays LLVM's, and so do
proc-macro crates and build scripts (`EPITOME_RUSTC_BUILD_SCRIPTS=1` takes
build scripts too). A crate whose IR Epitome cannot compile yet keeps
rustc's object, and the log says why. `EPITOME_RUSTC_OPT=1` runs Epitome's
optimiser on rustc's unoptimised IR as well; `EPITOME_RUSTC_SKIP` and
`EPITOME_RUSTC_ONLY` take comma-separated crate names. This is a wrapper,
not a rustc codegen backend (`-Zcodegen-backend`), which would have to be
built against rustc's internals and turn its MIR into our IR.

## License

Copyright (c) 2026 Amy Thomason. No license has been chosen yet; see
[LICENSE](LICENSE).

## Crates that build

These crates' own tests (`cargo test --release`) pass with their code,
and their dependencies', compiled by Epitome through `epitome-rustc` (see
[Building Rust crates with Epitome](#building-rust-crates-with-epitome)),
on an AArch64 Mac with Rust 1.99.1: 256 crates, at the versions given.
chrono's pass too, but for 8 tests of time-zone files its package leaves
out. Crates whose tests fail for reasons of their own (files or crates
missing from the package, or failing under plain rustc) are not listed.

- **a**: adler2 2.0.1, ahash 0.8.12, aho-corasick 1.1.5, allocator-api2 0.4.0, anstyle 1.0.14, anyhow 1.0.104, approx 0.5.1, arbitrary 1.4.2, arrayref 0.3.9, arrayvec 0.7.8, assert_matches 1.5.0, async-channel 2.5.0, async-lock 3.4.2, async-task 4.7.1, atomic-waker 1.1.2
- **b**: base64 0.22.1, bincode 1.3.3, bit-set 0.9.1, bit-vec 0.9.1, bit_field 0.10.1, bitflags 2.13.2, bitstream-io 4.9.0, bitvec 1.0.1, block-buffer 0.12.1, bumpalo 3.20.3, bytecount 0.6.9, bytemuck 1.25.2, byteorder 1.5.0, bytes 1.12.1
- **c**: castaway 0.2.4, cfg-if 1.0.5, chacha20 0.10.2, cipher 0.5.2, cobs 0.3.0, color_quant 1.1.0, colorchoice 1.0.5, combine 4.6.8, compact_str 0.9.1, concurrent-queue 2.5.0, const-oid 0.10.2, constant_time_eq 0.4.2, crc32fast 1.5.3, crossbeam-channel 0.5.17, crossbeam-deque 0.8.8, crossbeam-epoch 0.9.21, crossbeam-queue 0.3.14, crossbeam-utils 0.8.23, crunchy 0.2.4, crypto-common 0.2.2, csv 1.4.0, csv-core 0.1.13, ctor 1.0.13
- **d**: dary_heap 0.3.9, data-encoding 2.11.1, deranged 0.5.9, diff 0.1.13, dissimilar 1.0.11, downcast-rs 2.0.2, dunce 1.0.5
- **e**: either 1.19.0, encode_unicode 1.0.0, encoding_rs 0.8.42, env_logger 0.11.11, equivalent 1.0.3, erased-serde 0.4.10, errno 0.3.14, euclid 0.22.14, event-listener 5.4.2
- **f**: fallible-iterator 0.3.0, fallible-streaming-iterator 0.1.9, fancy-regex 0.18.0, faster-hex 0.5.0, fastrand 2.5.0, fixedbitset 0.5.7, flate2 1.1.10, float-ord 0.3.0, flume 0.12.0, fnv 1.0.7, foldhash 0.2.0, foreign-types-shared 0.3.1, form_urlencoded 1.2.2, futures-lite 2.6.1, fxhash 0.2.1
- **g**: gethostname 1.1.0, gimli 0.34.0, glam 0.32.1, glob 0.3.4, globset 0.4.20
- **h**: half 2.7.1, hash32 0.3.1, hashbrown 0.17.1, hashlink 0.9.1, heapless 0.9.3, heck 0.5.0, hex 0.4.3, hexf-parse 0.2.1, hmac 0.13.0, httparse 1.10.1, httpdate 1.0.3, humantime 2.4.0
- **i**: ident_case 1.0.1, indexmap 2.14.2, inout 0.2.2, inventory 0.3.24, ipnet 2.12.2, itertools 0.14.0, itoa 1.0.18
- **j**: jobserver 0.1.35
- **l**: lexopt 0.3.1, libc 0.2.191, lock_api 0.4.14, log 0.4.34, lru-slab 0.1.3, lz4_flex 0.13.1
- **m**: matchers 0.2.0, matches 0.1.10, matrixmultiply 0.3.11, memchr 2.8.3, memmap2 0.9.11, memoffset 0.6.5, mime_guess 2.0.5, minimal-lexical 0.2.1, miniserde 0.1.45, miniz_oxide 0.9.1, multimap 0.10.1
- **n**: nb 1.1.0, nonmax 0.5.5, num-bigint 0.5.1, num-complex 0.4.6, num-conv 0.2.2, num-integer 0.1.47, num-iter 0.1.46, num-rational 0.4.2, num-traits 0.2.19
- **o**: once_cell 1.21.4, once_cell_polyfill 1.70.2, oneshot 0.1.13, oorandom 11.1.5, option-ext 0.2.0, ordered-float 5.5.0, os_str_bytes 6.6.1, owning_ref 0.4.1
- **p**: page_size 0.6.0, parking 2.2.1, parking_lot 0.12.5, pathdiff 0.2.3, percent-encoding 2.3.2, pest 2.8.0, petgraph 0.8.3, phf_shared 0.12.1, pico-args 0.5.0, pin-utils 0.1.0, piper 0.2.5, postcard 1.1.3, powerfmt 0.2.1, ppv-lite86 0.2.21, pulldown-cmark 0.9.6
- **q**: quick-error 2.0.1, quick-xml 0.41.0
- **r**: radium 0.7.0, radsort 0.1.1, rand 0.10.3, rand_chacha 0.9.0, rand_core 0.10.1, rand_distr 0.6.0, rand_isaac 0.3.0, rand_pcg 0.10.2, rand_xorshift 0.5.0, rand_xoshiro 0.8.0, rawpointer 0.2.1, rayon 1.12.0, regex-syntax 0.8.11, rgb 0.8.53, ron 0.12.2, rustc-demangle 0.1.27, rustc-hash 2.1.3, rustc-hex 2.1.0, rustc_version 0.4.1, ryu 1.0.23
- **s**: same-file 1.0.6, scoped-tls 1.0.1, scopeguard 1.2.0, seahash 4.1.0, semver 1.0.28, semver-parser 0.7.0, send_wrapper 0.6.0, serde-value 0.7.0, serde_bytes 0.11.19, serde_cbor 0.11.2, serde_json 1.0.154, serde_plain 1.0.2, serde_repr 0.1.21, serde_urlencoded 0.7.1, sha1 0.11.0, sha2 0.11.0, sharded-slab 0.1.7, shellexpand 3.1.2, shlex 2.0.1, simba 0.9.1, simdutf8 0.1.5, siphasher 1.0.3, slab 0.4.12, slotmap 1.1.1, smallvec 1.16.3, smawk 0.3.2, smol_str 0.2.2, snap 1.1.1, spin 0.10.1, spin_on 0.1.1, stable_deref_trait 1.2.1, static_assertions 1.1.0, statrs 0.18.0, strsim 0.11.1, subtle 2.6.1, symlink 0.1.0
- **t**: tap 1.0.1, target-lexicon 0.13.5, termcolor 1.4.1, textwrap 0.16.1, thiserror 2.0.21, thousands 0.2.0, thread_local 1.1.10, tinystr 0.8.4, tinytemplate 1.2.1, tinyvec 1.13.3, toml 1.1.8+spec-1.1.0, toml_edit 0.25.15+spec-1.1.0, twox-hash 2.1.5, typed-path 0.12.4, typeid 1.0.3, typenum 1.20.1, typetag 0.2.14
- **u**: unicase 2.10.0, unicode-bidi 0.3.18, unicode-linebreak 0.1.5, unicode-normalization 0.1.25, unicode-segmentation 1.13.3, unicode-width 0.2.2, unicode-xid 0.2.6, unicode_categories 0.1.1, untrusted 0.9.0, url 2.5.8, urlencoding 2.1.3, utf16_iter 1.0.5, utf8-zero 0.8.1, utf8_iter 1.0.4, utf8parse 0.2.2, uuid 1.26.1
- **v**: vec_map 0.8.2, version_check 0.9.5
- **w**: walkdir 2.5.0, wide 0.7.33, winnow 1.0.4
- **y**: yansi 1.0.1
- **z**: zeroize 1.9.1, zmij 1.0.23

## Gap to LLVM

Where Epitome stands against clang and LLVM, from the measurements in
`docs/design` (`backend-plan.md`, `later.md`, `x86.md`, `isel.md`,
`ir.md`), and what it would take to catch up.

**Code size and speed now:**

- **AArch64 test programs.** Ours: 3,323 instructions (1.08×). Clang's IR
  and our code generator: 3,412 (1.11×). Clang `-O2`: 3,085. The start was
  1.24× and 1.20×.
- **AArch64 vectorised programs** are further behind. Instructions, LLVM's
  against ours:
  - `vectors.c`: 1,719 against 2,381
  - `loops.c`: 619 against 885
  - `basic.c`: 901 against 1,147
- **Rust crates (AArch64)**, against llc:

  | | llc | Ours | Ratio |
  | --- | --- | --- | --- |
  | Instructions | 68,885 | 79,899 | 1.16× |
  | Moves | 11,769 | 15,210 | |
  | Spill loads | 4,871 | 7,356 | |
  | Spill stores | 2,674 | 4,675 | |
- **x86-64.** Code is 1.31× llc's size, with 30,013 moves to llc's 17,880.
  SQLite runs in 1.47× clang's build's time, down from 1.85×.
- **SQLite on AArch64** runs in about 2× clang's build's time.
- **The LLM examples** have no LLVM-compiled build to compare with.
- **Compile speed.** Assembling SQLite (430,000 lines) takes 2.2 s, where
  clang takes 4.5 s. Every pass is kept close to linear in block size, and
  runs on many threads. No other compile times are measured against LLVM's.

**What we need to do**, in order of what it would gain:

1. **Register allocation.** This is most of the gap that remains:
   - rematerialise constants instead of spilling them;
   - split live ranges, especially around calls;
   - coalesce copies before allocation;
   - colour stack slots (a frame takes 1,088 bytes of slots where llc's
     takes 336);
   - on x86, fold reloads into memory operands.
2. **Vectorisation in the IR optimiser.** There is no loop or SLP
   vectoriser. The MLIR examples vectorise before they reach LLVM IR
   (`linalg-vectorize`), but C, C++ and Rust code does not, and many
   vector operations are still done a lane at a time (`later.md`).
3. **Scheduling.** Instructions come out in source order, and there is no
   machine model. On M4, emitted NEON loops take 1.56 against 0.70 cycles
   per four elements for an element-wise multiply, and 2.42 against 0.68
   for a row sum (as emitted against a hand-written order).
4. **Machine passes.** Missing: a rule-driven peephole pass, shrink
   wrapping, general `cbz` folding, if-conversion, conditional compares
   (`ccmp`: llc uses 126 in regex-syntax alone), sibling calls and jump
   tables. Also missing: the flags model for `i128` arithmetic (`adds`
   and `adc`), and on x86, AVX-512 mask registers, `vpternlog` and general
   shuffles.
5. **The middle end.**
   - Planned:
     - analyses: dominator trees, scalar evolution, lazy value info;
     - scalar passes: sccp, adce, jump threading;
     - loops: loop-simplify, LCSSA, loop deletion, loop unrolling;
     - interprocedural: function attributes, dead-argument elimination,
       globaldce, constmerge;
     - the verifier.
   - Later: sink, loop unswitching, partial inlining, argument promotion,
     mergefunc, and dependence analysis.
   - Only in part: GVN, which has no PRE, and LICM, which doesn't promote
     memory to registers.
6. **Targets.**
   - RISC-V: compressed instructions and V in the code generator,
     thread-locals, and position-independent code.
   - R600 and gfx1151: instruction selection, which needs the
     partial-machine work (`docs/design/partial-machines.md`).
   - x86-64: the APX, XOP and VSIB encodings.
