# IR and middle end

This document covers Epitome's intermediate representation and every pass that
runs on it. For sources and status labels, see the [index](README.md).

## 1. The IR

Epitome reads and writes LLVM's textual IR (`.ll`) as described in the
[Language Reference](https://llvm.org/docs/LangRef.html). The bitcode format
(`.bc`) comes later.

The IR is arranged as a hierarchy:

- **Module**: global variables, functions, aliases, metadata, data layout and
  target triple.
- **Function**: a list of basic blocks plus attributes and a calling
  convention.
- **Basic block**: a list of instructions ending in exactly one terminator.
- **Instruction**: produces at most one SSA value.

### Rules that the verifier enforces

These come from the `-verify` entry on the Passes page:

- Operand and result types agree.
- Every value is defined before it is used, as SSA requires.
- PHI nodes sit at the top of their block and have one entry per
  predecessor.

### Open questions

- **Open:** how values, uses and blocks are stored in Rust. The simplest
  choices are arenas with index handles, or reference counting. Arenas make
  replace-all-uses and deletion cheap and avoid borrow checker fights, so
  they are the likely choice.
- **Open:** whether to intern types and constants per context, as the
  documentation's `LLVMContext` model suggests.

## 2. Pass manager

Epitome follows the design of LLVM's new pass manager. The legacy pass manager
is not cloned.

### Levels and adaptors

Passes run at four levels, nested from outermost to innermost:

1. Module
2. CGSCC, an optional level that walks strongly connected components of the
   call graph bottom-up
3. Function
4. Loop

A pass manager is itself a pass at its own level. Adaptors run an inner
pipeline inside an outer one, for example module to function or function to
loop. Grouping passes under one adaptor runs them all on each unit before
moving on, which is better for cache locality.

### Analyses

- There is one analysis manager per level. Results are computed when first
  asked for, then cached.
- A pass can only ask for cached results from an outer level, and must not
  change them. Loop passes get a fixed set of function analyses.
- Each pass returns a preserved-analyses set: all, none, specific analyses,
  or the set of CFG-only analyses. Anything not preserved is invalidated.
- An analysis can decide for itself whether it stays valid, and must check
  the analyses it depends on.
- Cached results for deleted IR must be cleared, because the memory may be
  reused.

### Pipeline text

`epitome-opt -passes=` takes LLVM's textual pipeline syntax:

- Passes nest by level, for example `module(cgscc(function(loop(licm))))`.
- If the first pass is not a module pass, the outer levels are added
  automatically.
- `require<name>` forces an analysis to run.
- `--print-passes` lists every pass, and `-debug-pass-manager` prints the
  order passes run in.

### Default pipelines

The documentation shows that `-O2` comes from building a default per-module
pipeline, and that extension points let extra passes be added at fixed
places in it.

- **Open:** the documentation does not list what `-O1`, `-O2`, `-O3`, `-Os`,
  `-Oz` or the LTO pipelines contain. Epitome will design its own pipelines
  from the passes below and compare output quality, not pass order.

## 3. Analysis passes

| Pass | What it does | Status |
| --- | --- | --- |
| `domtree` | Builds the dominator tree. | Planned |
| `postdomtree` | Builds the post-dominator tree. | Planned |
| `domfrontier` | Computes dominance frontiers. The classic way to place phis for `mem2reg`; ours uses the on-demand SSA updater instead, which does not need them. | Later |
| `loops` | Finds natural loops and the loop depth of each block. Natural loops, latches and preheaders; see [loops](loops.md). Loop depth not yet. | Partial |
| `scalar-evolution` | Describes scalar expressions in loops, including induction variables and trip counts. | Planned |
| `basic-aa` | Stateless alias analysis from simple facts, such as two distinct globals never aliasing. See [alias analysis](aa.md). | Done |
| `globals-aa` | Alias and mod/ref facts for globals whose address is never taken, plus pure functions. | Later |
| `scev-aa` | Alias analysis using scalar evolution, within one loop iteration. | Later |
| `memdep` | Finds the earlier memory operations a memory operation depends on, computed lazily. Done as memory SSA with a walker that answers on demand; see [memory SSA](memssa.md). The combiner's memory rules and `licm` use it. | Done |
| `lazy-value-info` | Computes value range facts on demand. Used by jump threading. | Planned |
| `da` | Dependence analysis between memory accesses. | Later |
| `iv-users` | Tracks the uses of induction-variable expressions. Used by `loop-reduce`. | Later |
| `regions` | Finds single-entry, single-exit regions and builds a region tree. | Later |
| `basiccg` | Builds the call graph. The Passes page says this entry is not yet written. See [parallel](parallel.md). | Done |
| `stack-safety` | Works out whether stack allocations are safe from memory errors. Mainly for sanitizers. | Later |
| `instcount` | Counts instructions by kind. | Planned |
| `lint` | Warns about IR that is probably undefined or unintended. | Later |
| `aa-eval` | Checks every pointer pair and reports how precise alias analysis is. | Later |
| `kernel-info` | Reports statistics for GPU kernels. | Skip |

### Printers and viewers

These print or draw an analysis. They are cheap to write once the analysis
exists.

| Pass | What it does | Status |
| --- | --- | --- |
| `print<module-debuginfo>` | Prints module debug info in readable form. | Later |
| `print<alias-sets>` | Prints alias sets. The Passes page says this entry is not yet written. | Later |
| `print-callgraph` | Prints the call graph. | Later |
| `print-callgraph-sccs` | Prints the call graph's strongly connected components. `epitome-opt --print-cycles` lists the recursive ones. | Later |
| `print<cfg-sccs>` | Prints each function's CFG strongly connected components. | Later |
| `function(print)` | Prints each function as the pipeline reaches it. | Planned |
| `module(print)` | Prints the whole module. | Planned |
| `dot-callgraph` | Writes the call graph as a `.dot` file. | Later |
| `dot-cfg`, `dot-cfg-only` | Writes each function's CFG as a `.dot` file, with or without instructions. | Later |
| `dot-dom`, `dot-dom-only` | Writes the dominator tree as a `.dot` file. | Later |
| `dot-post-dom`, `dot-post-dom-only` | Writes the post-dominator tree as a `.dot` file. | Later |

## 4. Transform passes

### Scalar cleanup

| Pass | What it does | Status |
| --- | --- | --- |
| `instcombine` | Combines and simplifies instructions into a canonical form. The rule-driven combiner covers arithmetic, comparisons, selects, casts and constant folding; see [combines](combine.md). | Done |
| `aggressive-instcombine` | Matches larger expression patterns and may change the CFG. Factoring, distribution and the identities of `and`, `or` and `xor` of two values are combine rules; see [combines](combine.md). Rotates and funnel shifts written as two shifts and an `or` are recognised (`llvm.fshl`); recognising popcount written out is not done yet. | Partial |
| `simplifycfg` | Removes unreachable blocks, merges blocks, removes trivial PHIs and branch-only blocks. Branch folding is in the combiner; see [simplifying the CFG](simplifycfg.md). | Done |
| `dce` | Removes dead instructions, and anything that becomes dead as a result. A step of the combiner: an unused instruction without effects goes, and its operands go back on the list. | Done |
| `bdce` | Removes computation of bits nothing reads. The combiner's demanded-bits step reads a narrower use's bits from a simpler value already made; see [combines](combine.md). Bits demanded across several uses are not tracked. | Partial |
| `adce` | Aggressive dead code elimination: everything is dead until proven live. | Planned |
| `dse` | Removes stores that are overwritten before being read. A step of the combiner: overwritten on every path before any read (across blocks and loops, within a budget), or to a local never read again, or of the value already there; see [combines](combine.md). | Done |
| `sccp` | Sparse conditional constant propagation. Finds constants and dead blocks. | Planned |
| `reassociate` | Reorders commutative expressions to help constant folding, CSE and LICM. A step of the combiner, with sums collected into linear combinations; see [combines](combine.md). | Done |
| `early-cse` | Removes instructions and loads that repeat a dominating one, in one walk of the dominator tree. Not on the Passes page; see [CSE](cse.md). | Done |
| `gvn` | Removes fully and partially redundant instructions and loads. CSE covers the fully redundant ones within a dominator path; the combiner forwards stored values to loads and reuses loads under the same memory version, across joins ([combines](combine.md)). Partial redundancy is not done. | Partial |
| `jump-threading` | Routes edges past blocks whose branch result is already known. | Planned |
| `sink` | Moves instructions into the successor blocks that need them. | Later |
| `memcpyopt` | Removes `memcpy` calls and turns runs of stores into `memset`. Runs of stores become `memset` (`opt/src/memcpyopt.rs`), which code generation writes back as wide stores when small. Removing copies is not done. | Partial |
| `tailcallelim` | Turns self-recursive tail calls into loops. Part of recursion removal; see [parallel](parallel.md). Marking the calls that may reuse the caller's frame `tail`, in functions with no allocas, variable arguments or `byval` parameters, is `tailmark.rs`. | Done |
| `normalize` | Renames and reorders instructions into a form that diffs well. | Later |

### Memory to registers

| Pass | What it does | Status |
| --- | --- | --- |
| `mem2reg` | Promotes allocas that are only loaded and stored into SSA values. See [mem2reg](mem2reg.md). | Done |
| `sroa` | Splits aggregate allocas into one per field, then promotes them. See [SROA](sroa.md). | Done |
| `reg2mem` | Demotes values back to stack slots, the reverse of `mem2reg`. | Later |

### Loops

| Pass | What it does | Status |
| --- | --- | --- |
| `loop-simplify` | Gives every loop a preheader, dedicated exits and a single backedge. | Planned |
| `lcssa` | Adds PHIs at loop exits for values used outside the loop. | Planned |
| `loop-rotate` | Rotates loops into a simpler form. See [loops](loops.md). | Done |
| `licm` | Moves loop-invariant code out of loops and promotes memory to registers. Invariant arithmetic that cannot trap, and loads whose clobber is outside the loop from blocks that run on every way out (`opt/src/licm.rs`). Promotion to registers is not done. | Partial |
| `indvars` | Canonicalises induction variables and exit conditions. Exact exit tests, exit values, widening and trip counts for constant steps, without scalar evolution; see [loops](loops.md). | Partial |
| `loop-deletion` | Removes finite loops with no side effects and no used results. | Planned |
| `loop-unroll` | Unrolls loops. Works best after `indvars`. | Planned |
| `loop-reduce` | Strength-reduces array addressing that uses the induction variable. Pointers for arrays indexed by a unit-step counter, and counting down to zero; see [loops](loops.md). | Partial |
| `simple-loop-unswitch` | Moves loop-invariant branches out by copying the loop, within a size limit. | Later |
| `loop-unroll-and-jam` | Unrolls an outer loop and fuses the inner copies, using dependence analysis. | Later |
| `loop-fusion` | Fuses adjacent loops when that keeps the meaning. | Later |

### Interprocedural

| Pass | What it does | Status |
| --- | --- | --- |
| `inline` | Inlines calls, walking the call graph bottom-up. See [parallel](parallel.md). | Done |
| `always-inline` | Inlines only functions marked `alwaysinline`. The inliner always inlines these; there is no separate pass yet. | Planned |
| `partial-inliner` | Inlines the part of a function that wraps its body, such as an early-return `if`. | Later |
| `function-attrs` | Infers attributes such as `readnone`, `readonly` and `nocapture`. | Planned |
| `argpromotion` | Turns read-only pointer arguments into value arguments. | Later |
| `deadargelim` | Removes unused arguments of internal functions. | Planned |
| `ipsccp` | Interprocedural sparse conditional constant propagation. Only its simplest part: a call whose arguments are all constants is followed through the callee's body, and replaced by its result when that path only computes integers (`constcall.rs`). | Partial |
| `globalopt` | Optimises globals whose address is never taken. Loads from constant globals fold in the combiner; see [combines](combine.md). The rest is not done yet. | Partial |
| `globaldce` | Deletes unreachable internal globals, including cycles of them. The inliner deletes internal functions it leaves unused, but not variables or cycles. | Planned |
| `constmerge` | Merges identical global constants. Private and internal `unnamed_addr` ones, in `merge.rs`; see [merging functions](merge.md). | Done |
| `mergefunc` | Finds identical functions and folds them into one: the others become aliases, or thunks where their address is significant. See [merging functions](merge.md). | Done |
| `internalize` | Makes every global except `main` internal. | Later |

### Epitome additions

These have no entry on the Passes page.

| Pass | What it does | Status |
| --- | --- | --- |
| recursion removal | Collapses cycles of functions by inlining, then turns self-recursion into a loop with an explicit stack of frames. See [parallel](parallel.md). | Done |

### Lowering

These rewrite IR into simpler forms for code generation or for targets that
lack a feature.

| Pass | What it does | Status |
| --- | --- | --- |
| `codegenprepare` | Rewrites IR to suit instruction selection. Runs at the start of codegen. Address sinking is done; see [instruction selection](isel.md). | Partial |
| `lower-switch` | Rewrites `switch` as a chain of branches. | Planned |
| `lower-invoke` | Turns `invoke` into `call` for targets without unwinding. | Planned |
| `lower-atomic` | Turns atomics into plain operations for single-threaded targets. | Later |
| `lower-global-dtors` | Turns global destructors into constructors that register them with `__cxa_atexit`. | Later |
| `break-crit-edges` | Splits critical edges by adding empty blocks. | Planned |
| `rel-lookup-table-converter` | Turns lookup tables into position-independent relative tables. | Later |
| `block-placement` | Orders blocks so the likely branch falls through, using profile data. Epitome places cold blocks (panics, cold calls, unlikely branches by their weights) after the rest; see [codegen](codegen.md). Profile counts are not read. | Partial |

### Stripping

| Pass | What it does | Status |
| --- | --- | --- |
| `strip` | Removes value names, internal symbols and debug info. | Later |
| `strip-dead-debug-info` | Removes debug info for unused symbols only. | Later |
| `strip-dead-prototypes` | Removes unused declarations of functions with no body. | Later |
| `strip-debug-declare` | Removes `llvm.dbg.declare` calls and `#dbg_declare` records. | Later |
| `strip-nondebug` | Removes all symbols except debug info. | Later |

## 5. Utility passes

| Pass | What it does | Status |
| --- | --- | --- |
| `verify` | Checks that IR is well-formed. Runs before and after every pipeline in debug builds. | Planned |
| `instnamer` | Names unnamed instructions so diffs are readable. | Planned |
| `extract-blocks` | Moves each basic block into its own function. | Later |
| `transform-warning` | Warns when a forced transform, such as `#pragma omp simd`, did not happen. | Later |
| `view-cfg`, `view-cfg-only`, `view-dom`, `view-dom-only`, `view-post-dom`, `view-post-dom-only` | Open GraphViz windows. The `dot-*` passes cover the same need. | Skip |

## 6. Build order

The minimum useful path is: parse `.ll`, then verify, then run `mem2reg` and
`simplifycfg`, then print `.ll`. After that, passes are added by what they
depend on:

1. `domtree`, `domfrontier`, `loops`, `postdomtree`
2. `mem2reg`, `sroa`, `simplifycfg`, `dce`, `instcombine`
3. `basic-aa`, `memdep`, then `gvn`, `dse`, `licm`
4. `scalar-evolution`, then `indvars`, `loop-unroll`, `loop-deletion`
5. Call graph, then `inline`, `function-attrs`, `globalopt`, `globaldce`
