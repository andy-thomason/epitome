# Parallel passes, inlining and recursion removal

Epitome runs passes on many functions at once. This document describes how
the module is shared between threads, how work is ordered, and the first
two interprocedural passes built on that: the inliner and recursion
removal. The code is in the `epitome-opt` crate (`opt/`); the
`epitome-opt` binary runs the pipeline. For sources and status labels, see
the [index](README.md).

## 1. Goals

- **Use every core.** Functions are the unit of work. Any pass that can run
  on functions independently does so.
- **Same output for any number of threads.** The printed module must not
  depend on thread timing. This is tested on every example and program,
  with 1 thread against 4 or 8.
- **No locks on hot paths.** A thread owns the functions it changes
  outright. Shared tables are read far more than they are written.
- **`std` only.** Threads come from `std::thread::scope`; there is no
  dependency on a thread-pool crate.

## 2. Sharing a module between threads

A module is already split along the right line. Each function owns its
locals, blocks and instructions in index-based tables, so changing one
function never touches another. What is module-wide is:

- the type, constant and metadata tables, which are interned;
- the global headers: names, linkage, attributes, attribute groups.

`ctx::parallel_phase` takes the module apart for the length of a phase:

| Part | During the phase |
| --- | --- |
| Function bodies | Each moves into its own slot. A task `take`s the bodies it works on, which gives it `&mut Function` with no lock, and `publish`es them when done. A published body is read-only and any task may read it without locking. |
| Types and constants | Frozen for the phase, so any thread reads them with no lock. New entries go into an append-only list whose entries never move, so they are read without a lock too; only adding one takes a lock (`table.rs`). |
| Metadata | Behind a `RwLock`. Passes rarely touch it. |
| Global headers | Read-only, through a shared `&Module`. A pass that needs a new global (for example an intrinsic declaration) declares it before the phase starts. |

At the end of the phase the bodies go back into the module and identical
metadata nodes created by different threads are merged
(`metadata::unique_metadata`).

### Why the output does not depend on timing

Interned ids can be handed out in a different order from run to run. That
does not change the printed text:

- types and constants are printed by content, not by id;
- metadata is renumbered by walking the module when printed;
- identified structs are printed in table order, so a parallel pass must
  only create literal structs. The recursion pass follows this rule.

What a task computes depends only on the bodies it owns and on bodies
that were published before it started, which are final. Remarks are kept
per function and printed in module order.

## 3. Ordering the work

### Function passes

A pass that looks at one function at a time uses `par::map`: every
function is an independent task.

### Call-graph passes

The inliner needs the final form of a callee before it decides whether to
inline it, so it works bottom-up over the call graph:

1. `CallGraph::build` scans function bodies (in parallel) for direct calls
   and for uses of function addresses. An edge is a direct call; calls
   through pointers are not edges, because they cannot be inlined.
2. Tarjan's algorithm finds the strongly connected components (SCCs). A
   recursive SCC (more than one function, or a function that calls itself)
   is a cycle. `--print-cycles` lists one path through each.
3. The SCCs form a DAG. `par::run_dag` runs each SCC as a task once every
   SCC it calls is finished. Functions inside one SCC are handled by one
   task, one after another.
4. Among ready tasks, those with the longest chain of work waiting on them
   go first (`CallGraph::scc_priorities`), which shortens the critical
   path.

The task for an SCC takes its bodies, inlines calls into finished SCCs,
removes recursion if the SCC is a cycle, and publishes the bodies, which
may make the SCCs that call it ready.

### Measurements

`epitome-opt --stats` reports the time spent in each step, the total task
time, and the critical path: the longest chain of tasks that must run one
after another. Total task time divided by the critical path is the best
speed-up threads could give.

The synthetic modules come from `cargo run --release --example synth --
<functions> [layers]`. Timings are for the parallel step on a 12-core
machine with 8 performance cores.

| Module | 1 thread | 2 | 4 | 8 | 12 | Upper bound |
| --- | --- | --- | --- | --- | --- | --- |
| 50,000 functions in 16 layers (wide, like a real program) | 2952 ms | 1549 ms | 887 ms | 523 ms | 478 ms | 280x |
| 50,000 functions in long chains (`layers` = 0) | 3067 ms | 1597 ms | 1025 ms | 1066 ms | 1088 ms | 3.5x |

Times are the best of three runs. The wide module gets 5.6x on 8 threads;
beyond 8, the extra threads run on the slower efficiency cores. The chained
module stops at 3x, near its bound of 3.5x: each function waits for the one
below it, and inlining makes each link of the chain heavier. Either way the
output is the same, byte for byte, for every thread count.

Profiles show no time waiting for locks. The rest is ordinary work that can
be made cheaper: hashing (`HashMap` with the default SipHash), allocation,
and `Function::replace_uses`, which scans the whole function because there
are no use lists yet.

Parsing and printing are still serial and now take longer than the passes
(about 0.5 s and 2.3 s for the 50,000-function module). **Open:** print
functions in parallel; each function's text depends only on its own slot
numbers and the metadata numbering, which can be computed first.

### Code generation

`epitome-llc` compiles functions on every core too (`codegen/src/lib.rs`,
`-j` to choose how many). Functions are independent once optimised, so
there is no ordering:

1. The bodies come out of the module. Each function keeps its header and
   parameters, which other functions' calls look at.
2. Each thread works on its own copy of what is left: types, constants and
   globals, which compiling a function adds to (intrinsic declarations,
   new constants). Copying it takes about 3 ms for a 9,000-function module.
3. Threads take functions largest first, so a big one does not start last,
   and the results are put back in module order.
4. Each function's assembly, and each function's machine code for an object
   file, are written in parallel too, and joined in order.

A function's machine IR names what it refers to by symbol, not by the
module's ids, so the output is the same, byte for byte, for any number of
threads. `tests/parallel.rs` checks this for the C programs and a module of
generated functions. The selector's statistics for `--stats` are counted per
thread and added up when a thread finishes.

For the synthetic module above (8,884 functions after optimisation, 1.5
million machine instructions):

| Threads | Compiling and printing |
| --- | --- |
| 1 | 4394 ms |
| 4 | 1282 ms |
| 8 | 761 ms |

That is 5.8x on 8 threads. Parsing (serial) is most of what remains of the
1.4 s the whole run takes.

### Big blocks

Nothing may take time growing faster than a block's size. Two stages did,
both found with one generated block of 4,000 to 128,000 instructions:

- **Selection.** A post-increment load or store looks for the step of its
  base later in the block (`find_extra`). It scanned the rest of the block
  for every load and store. The block's instructions are now indexed by
  head and first operand, so it reads only the instructions with the same
  base.
- **Register allocation.** Whether a register was free for a value was a
  scan over every interval already given to it, and whether a value lived
  across a call was a scan over every call. A register's intervals are now
  kept as their union (disjoint, in a `BTreeMap` by start), so a value's
  check is one lookup per segment. Calls are found by binary search.

At 32,000 instructions selection went from 5,266 ms to 40 ms and allocation
from 143 ms to 16 ms. The output is the same as before.

Blocks where many values are live at once (operands from the last couple
of hundred values, as a simulator's `eval` has) found more, from 40,000 to
640,000 instructions:

- **Allocation, moving values out of a register.** Taking a value off a
  register rebuilt the register's whole map of busy intervals, and the
  search for values to move listed every value along a long one's range on
  every register before judging any. A moved value's intervals are now
  taken out by lookup, and a register is given up at the first value in
  the way that cannot move.
- **Preparation.** Three passes (`const_multiplies`, `constant_addresses`,
  `promote_odd`) inserted instructions into the middle of a block's list,
  and one replaced each value's uses with a pass over the function. Blocks
  are now made again in one pass, and the replacements put in place
  together (`Function::replace_many`).
- **Frames.** Whether an offset fits an instruction is now cached
  (`fits_cached`): a big frame asks it for every spill.

At 640,000 instructions allocation went from 52.9 s to 4.1 s, preparation
from 2.9 s to 0.5 s, and the whole compile from 50 s to 7.8 s; for a block
four times bigger, allocation and preparation now take 4 to 5.5 times as
long, where allocation took 20 times, and the output is the same as
before. Frames still take a step up (about 11x from
40,000 to 160,000) where the frame grows past what an instruction's offset
reaches and accesses need their address made first; past that it is
linear again. The optimiser was
already linear on the same blocks, and on blocks of unoptimised code (locals
in allocas) and of loads and stores through pointers that may alias.
`tests/parallel.rs` checks every stage of code generation: a block four
times bigger must take less than ten times as long (linear is four).

The optimiser is checked the same way (`opt/tests/scaling.rs`): a chain of
diamonds, a chain of blocks, thousands of calls in one block, and a big
block of loads and stores. Reading every pass for [autochop](chop.md)
found several things quadratic in a function's size there, listed in
that document.

## 4. The inliner

The inliner (`inline.rs`) runs on each function after its callees are
final. It only considers calls to functions in other SCCs; calls inside a
cycle are left to the recursion pass. Calls exposed by inlining are not
reconsidered: the callee's own inliner already made those decisions.

### Legality

A call is never inlined when:

- the callee has no body, is variadic, is `noinline` or `optnone`, or the
  call site is `noinline`;
- the callee's definition may be replaced at link time: `weak`,
  `linkonce`, `common` or `extern_weak` linkage (the `_odr` forms are
  fine, because every definition must be the same);
- the callee still calls into its own SCC, so inlining would only unroll a
  cycle;
- the callee uses `indirectbr`, `callbr`, a `musttail` call, funclet-based
  exception handling, `inalloca`, `llvm.localescape`, calls a
  `returns_twice` function, or has `byval`, `inalloca` or `preallocated`
  parameters;
- `blockaddress` names one of the callee's blocks;
- the call is `musttail`, `callbr`, has operand bundles, or its calling
  convention or function type does not match the callee;
- the call is an `invoke` and the callee may unwind;
- the callee needs a different personality function, garbage collector,
  `"target-cpu"` or `"target-features"` from the caller.

### Cost model

Each call compares what inlining adds in code size with what it saves.

**Cost** is the callee's size in weighted instructions. These count 0,
because they disappear or fold into addressing: `phi`, `br`, `ret`,
`unreachable`, static allocas in the entry block, `bitcast`, `getelementptr`
with constant indices, and calls to debug, lifetime, assume and alias-scope
intrinsics. A `switch` costs 1 plus half its cases; a call to a real
function costs `CALL_COST` (5) plus one per argument; anything else costs 1.

**Bonus** is what inlining removes or makes possible at this call:

| Bonus | Amount | Why |
| --- | --- | --- |
| The call itself | `CALL_COST` + 1 per argument | No call, return, or argument moves. |
| Constant argument used in a comparison, branch, switch or select | 10 per use | The branch folds away once constants are propagated. |
| Constant argument used elsewhere | 2 per use | The arithmetic folds. |
| Pointer to a caller alloca, used for a load, store or address | 3 per use, up to 8 uses | The memory can be promoted to registers. |
| Only call of an internal function whose address is never taken | callee cost + 5 | The function is deleted afterwards, so code size does not grow. |

A call is inlined when `cost - bonus <= threshold`.

| Level | Threshold | Growth factor | Growth slack |
| --- | --- | --- | --- |
| `-O1` | 40 | 2 | 100 |
| `-O2` | 100 | 4 | 400 |
| `-O3` | 250 | 8 | 1000 |
| `-Os` | 25 | 2 | 50 |
| `-Oz` | 0 | 1 | 20 |

The threshold drops to at most 0 for callers marked `optsize` or `minsize`,
for `cold` callees or call sites, and for calls in a block that leads only
to `unreachable`, which stands in for profile data. At 0, a call is
inlined only if it makes the code no bigger. An `inlinehint` callee gets
half the threshold again. `alwaysinline` skips the comparison but not the
legality checks.

A caller may grow to at most `growth factor × starting size + growth
slack` instructions, whatever the individual decisions say. Inlining the
only call of a function that is then deleted is exempt, because the module
does not grow, but no function grows past 50,000 instructions.

These numbers are our own starting points. **Open:** tune them against
code size and run time on real programs once constant propagation and CFG
simplification exist. Without those, the constant-argument bonus pays for
folding that has not happened yet.

`--remarks` prints every decision, with its cost, bonus and threshold, or
the reason it was refused. `tests/examples/inline_cost.ll` shows each rule.

### Doing it

`inline_call` splits the caller's block after the call, copies the
callee's blocks in between, turns each `ret` into a branch to the second
half, and joins the returned values in a phi. It also:

- moves the callee's static allocas to the caller's entry block, so
  inlining into a loop does not grow the stack on every iteration. If the
  callee also has dynamic allocas, the stack is saved before the copied
  body and restored after it;
- clears `tail` on copied calls, because the callee's allocas are now the
  caller's and the caller's allocas may be in the arguments;
- for an `invoke` of a callee that cannot unwind, continues at the normal
  destination and drops the unwind edge;
- adds the call site's location to the end of the `inlinedAt` chain of
  every copied debug location, and copies debug records with their values
  mapped. If the call site has no location, the copies get none either;
- gives each copy of a loop its own `!llvm.loop` node, with `inlinedAt`
  added to the locations inside it;
- moves debug records attached to the call onto the branch into the copied
  body, so they are not lost;
- renames copied values with a `.i` suffix where names would clash.

Blocks are merged afterwards to keep the result readable, except in
functions whose block addresses are taken.

After inlining, internal functions that nothing refers to are deleted,
working through a list so that functions only they called are deleted
too.

**Open:**

- inlining an `invoke` whose callee may unwind, which needs the callee's
  `resume`s turned into branches to the caller's landing pad;
- `byval` parameters (copy into a new alloca) and funclet-based exception
  handling;
- reconsidering calls exposed by inlining, with a history to avoid
  inlining through a cycle again and again;
- fresh `DIAssignID`s and `!llvm.loop` nodes for each copy.

## 5. Cycles

Every recursive SCC is reported. The recursion pass then tries to remove
the cycle.

A cycle of several functions is first reduced to self-recursion by
inlining (`recursion::collapse_scc`). An *entry* is a member that can be
called from outside the SCC: it is not internal, its address is taken, or
a function outside the SCC calls it. Each entry, working on a copy of the
original bodies, inlines its calls to the other members (never `noinline`
or `optnone` ones, nor ones whose definition may be replaced) until it calls
only itself and functions outside the SCC. Members that are not entries
are then unused and get deleted. With several entries, each gets its own
copy of the cycle, so a size limit (four times the cycle's size plus 200
instructions) keeps this to small cycles.

Self-calls inside members are removed before collapsing. Otherwise
inlining a member that calls itself would bring the self-call along every
time and never finish. Once a cycle is gone, its function is an ordinary
callee, and the SCCs above it can inline it.

**Open:** a cycle among non-entry members that does not pass through the
entry (A calls B, B and C call each other) cannot be collapsed this way;
it needs a second level of SCCs inside the first. A general alternative is
to merge the whole SCC into one function with a selector argument and
thin entry functions.

## 6. Recursion with an explicit stack

`recursion::eliminate` turns a self-recursive function into a loop. The
old entry block becomes a loop header whose phis hold the parameters, and
a new entry block branches to it.

- A **tail call**, whose result is returned without further work (through
  branches and phis if need be), jumps to the header with the new
  arguments. This is tail-recursion elimination.
- An **accumulator call**, whose result is used once, by `add`, `mul`,
  `and`, `or` or `xor`, and that result is returned, also jumps. An
  accumulator phi in the header starts at the operation's identity (0, 1
  or all ones), each such call folds its other operand into it, and every
  `ret v` returns `acc op v`. `n * fact(n - 1)` becomes a running product.
  Integer arithmetic wraps, so these operations are associative and
  commutative and the reordering is exact; flags such as `nsw` are
  dropped, because they held only for the old order. Only side-effect-free
  arithmetic may sit between the call and its use, since it now runs
  before the next activation instead of after it.
- **Any other call**, in `stack` mode, pushes a frame and jumps to the header. Every `ret`
  goes to a dispatch block: if the frame stack is empty the function
  returns; otherwise it pops a frame, reloads the saved values, and
  continues after the call that pushed the frame, with the returned value
  in place of the call's result.

Frames are kept on the machine stack, in one of two ways.

**A frame pool**, for functions with no allocas of their own (the usual
case). The new entry block statically allocates a chunk of up to 64 frames:

| Chunk field | Contents |
| --- | --- |
| 0 | the previous chunk, or null for the first |
| 1 | the stack pointer from before this chunk was allocated |
| 2 | the frames |

Every call site uses the same frame type: the call site's number (when
there is more than one), then the saved values. Fields are shared between
call sites by type, so the frame has, for each type, as many fields as the
most any one site saves. The stack's state is the current chunk and the
index of its next free frame. A push stores the frame at that index and
adds one. The header checks for room once, for whichever push comes next,
since every push goes straight to the header. When the chunk is full it
allocates a new chunk with `llvm.stacksave` and an `alloca`, and links it to
the old one. A pop reads the frame below the index. When the index reaches
the start of a chunk, the chunk's `llvm.stackrestore` frees it and popping
continues in the previous chunk, unless there is none, in which case the
function returns. The common path has no calls and no allocation.

**Linked frames**, for functions with allocas. Each activation has its own
allocas, which must be freed when it returns, so each frame records the
stack pointer anyway. A frame is a literal struct per call site:

| Frame field | Contents |
| --- | --- |
| 0 | the previous frame, or null |
| 1 | the stack pointer before the frame was allocated |
| 2 | the call site's number, when there is more than one site |
| 3 and up | the values live after the call |

A push is `llvm.stacksave` followed by an `alloca` of the frame. A pop
reloads the frame and calls `llvm.stackrestore`, which frees the frame and
everything the finished activation allocated above it.

Either way the stack needs no fixed size, no heap and no C library, and its
size is proportional to the recursion depth.

Details:

- **Liveness** decides what goes in a frame: the values live just after
  each call, not counting the call's own result.
- **Invariant parameters**, passed through unchanged by every recursive
  call, have the same value in every activation. They get no phi and are
  never saved. `queens(n, row + 1, cols)` saves neither `n` nor `cols`.
- **Allocas** stay in the old entry block, which is now inside the loop, so
  each activation gets its own. Popping a linked frame restores the stack
  pointer and frees them. In a function with allocas, tail calls push
  frames too, because jumping to the header would allocate again without
  freeing.
- **SSA form.** Splitting blocks at the calls leaves values with several
  definitions: the original, and a reload in each pop that restores it.
  `ssa::SsaUpdater` rebuilds SSA form with the on-demand method of Braun
  et al., "Simple and Efficient Construction of Static Single Assignment
  Form" (CC 2013), and phis that turn out to have one value are removed.
- **Debug records** get the value that reaches them if it is known without
  new phis; otherwise they are marked optimised out (`poison`). Debug
  information never adds code.
- A function that has `llvm.stacksave` or `llvm.stackrestore` calls,
  including frames from an earlier run of this pass, can be transformed:
  frames are pushed and popped in LIFO order with its own saves and
  restores.

The pass refuses functions whose definition may be replaced at link time
(the recursive calls might then reach another definition), and functions
that are variadic, have a `musttail` call to another function (it must
stay right before its `ret`, and the pass reroutes every `ret`), use
exception handling,
`indirectbr`, `callbr`, `va_arg` or `inalloca`, call `returns_twice`
functions or `llvm.frameaddress`, `llvm.returnaddress`, `llvm.sponentry`,
`llvm.localescape` or `llvm.va_*`, have `byval`-like or `swift*`
parameters, put allocas outside address space 0, recurse through `invoke`
or `callbr`, or would save a value of a type that cannot be stored. A
`musttail` self-call in a function with allocas is refused, because it
would need a frame and so lose `musttail`'s guarantee of bounded stack.

`--recursion=off|tail|stack` chooses what to remove:

- `tail`, the default, removes tail and accumulator calls, which needs no
  stack and never makes code slower. Other recursive calls stay calls, so
  the function stays recursive and the inliner leaves it alone. This is
  what LLVM's `tailcallelim` does.
- `stack` also turns every other recursive call into a frame push. It is
  for targets without a call stack or that forbid recursion, and for
  functions where it measurably pays (Ackermann below). With frames, an
  accumulator would have to be saved in each frame, so accumulator calls
  push frames too.

Functions marked `minsize` get `tail` at most, and `optnone` functions are
left alone.

**Open:** in unoptimised code every local lives in an alloca, so a call's
result goes through memory and no call looks like a tail or accumulator
call. This needs `mem2reg` or `sroa` to run first, as LLVM does.

**Open:** a per-function choice of `stack`, from a cost model or a target
hook, instead of one setting for the module.

### Why use the stack

- A cycle stops blocking the inliner: the function becomes an ordinary
  callee.
- Targets without a call stack, or where recursion is not allowed, such as
  many accelerators and shader models, can still run the code.
- A frame holds only the live values: no return address, saved registers
  or spill slots.

### Cost

On Apple M-series, with clang `-O2` compiling both the original and the
transformed IR, compared with native recursion:

| Benchmark | Native | Linked frames | Frame pool |
| --- | --- | --- | --- |
| `fib(35)` | 0.014 s | 0.024 s | 0.012 s |
| Ackermann(2, 3000) + Ackermann(3, 8) | 0.071 s | 0.027 s | 0.018 s |
| Sum of a 4M-node tree, 20 times | 0.006 s | 0.008 s | 0.006 s |

Linked frames are slower on small functions because each push is a dynamic
`alloca`, and clang marks functions with `"probe-stack"`, so each one calls
the stack probe `__chkstk_darwin`. The pool avoids that on all but one push
in 64. Ackermann gains most: its native recursion is deep, and each native
frame holds a return address and saved registers as well as the two values
the pool saves.

**Open:** a cost model for when to apply the transform. Today it is applied
to every recursive function that qualifies. Linked frames, needed when the
function has allocas, are the case where it can cost speed.
