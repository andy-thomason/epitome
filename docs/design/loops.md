# Loops: rotation, induction variables and trip counts

Three modules work on loops: `opt/src/loops.rs` finds them,
`opt/src/rotate.rs` rotates them, and `opt/src/indvars.rs` simplifies
their counters and works out how many times they run. For sources and
status labels, see the [index](README.md).

## Finding loops

A *back edge* goes from a block to one that dominates it, its *header*. The
loop is the header and every block that reaches a back edge without passing
through the header. The blocks with back edges are its *latches*. Loops
whose header does not dominate them (irreducible control flow) are not
natural loops, are not found and are left alone.

`loops::ensure_preheader` gives a loop a *preheader*: one block outside the
loop that is the header's only predecessor from outside, and branches only
to the header. If there is none, it adds an empty one; header phis' entries
from outside move to a phi in it.

## Rotation

Front ends test at the top:

```llvm
preheader:  br label %header
header:     %i = phi i32 [ 0, %preheader ], [ %i.next, %latch ]
            %c = icmp slt i32 %i, %n
            br i1 %c, label %body, label %exit
body ...    latch: %i.next = add nsw i32 %i, 1
                   br label %header
```

That takes two branches per iteration. Rotation copies the header's
instructions twice:

- into the preheader as a *guard* (does the loop run at all?), with each
  header phi replaced by its value from the preheader;
- into a new block after the latch as the *bottom test* (does it run
  again?), with each phi replaced by its value from the latch.

The old header goes, and `%body` becomes the header:

```llvm
preheader:  %c1 = icmp slt i32 0, %n
            br i1 %c1, label %body, label %exit
body:       %i = phi i32 [ 0, %preheader ], [ %i.next, %body ]
            ...
            %i.next = add nsw i32 %i, 1
            %c = icmp slt i32 %i.next, %n
            br i1 %c, label %body, label %exit
```

Each run of the old header is exactly one run of one of the copies, so
nothing runs that did not before. Values the header defined now have two
definitions, so SSA form is rebuilt with the SSA updater, which adds phis
where they meet. The bottom copies and those phis take the header values'
names. The combiner and simplifycfg then fold the guard when it is constant
and merge the bottom test into the latch.

A loop is rotated when:

- it has one latch, other than the header, ending in `br label %header`;
- the header ends in a conditional branch with one target inside the loop
  and one outside, and the inside target is reached only from the header
  and has no phis;
- the header has at most 16 instructions, none of them allocas, exception
  pads or calls marked `convergent` or `noduplicate`, whose copies would
  not mean the same;
- no header phi's value from the latch is computed in the header.

The last rule would stop the loops an inlined `Range::next` makes, which
compute `i + 1` before the test. So rotation first moves a header
instruction that only gives a phi its next value (an arithmetic operation,
cast, comparison, select or address, used once, by that phi) to the end of
the latch, where it runs only when the loop goes round again.

## Induction variables

An *induction variable* is a header phi `%i = phi [start, preheader],
[%i.next, latch]` with `%i.next = add %i, step`, for a constant step.
When the latch's branch compares the counter with a value that does not
change in the loop, `indvars`:

- **Works out the trip count** for steps of 1 and -1: a number when start
  and bound are constants, otherwise an expression such as `%n` or
  `%n - %start`. It is reported as a remark (`--remarks`). It assumes the
  loop is entered; the guard decides that.
- **Makes the exit test exact.** `%i.next < %n` with a step of 1 becomes
  `%i.next != %n`, if the loop only starts when `start < %n`. That is shown
  by constants, or by the guard rotation left in the preheader (for a
  start of 0 and `<` unsigned, the guard `%n != 0` the combiner makes of
  `0 < %n`). Counting down by 1 with `>` works the same way. Without the
  guard the rewrite would be wrong: a loop entered with `start >= %n` would
  run until it wraps. With it, the increment needs no `nuw` or `nsw`: the
  counter is below `%n` on entry and the test keeps it there, so `%i + 1`
  is at most `%n` and cannot wrap. The increment is given the flag (`nuw`
  for `<` unsigned, `nsw` for signed) that says so.
- **Replaces exit values.** With an exact test, the loop leaves with
  `%i.next == %n`, so an exit phi that takes `%i.next` from the latch takes
  `%n` instead.
- **Widens counters.** A counter narrower than 64 bits whose `nsw`
  increment cannot overflow equals the sign extension of a 64-bit counter
  with the same start and step. Each `sext` of the counter to `i64` in the
  loop is replaced by a 64-bit twin, so array indexing needs no `sext` per
  iteration. If the narrow counter is then used only by its increment and
  the exit test, the test moves to the twin, comparing with `sext %n` made
  in the preheader, and the narrow counter goes. Debug records showing it
  become `poison`.

The combiner adds `icmp ugt x, 0 -> icmp ne x, 0` and related rules, so an
unsigned countdown's test is already exact.

`opt/tests/examples/loops.ll` has a summing loop (exact test, widened
counter), a loop whose result is its counter (exit value), an unsigned
countdown and a loop with a constant trip count. For

```c
long sum_to(int *a, int n) { long s = 0; for (int i = 0; i < n; i++) s += a[i]; return s; }
```

the result is

```llvm
entry:
  %cmp1 = icmp sgt i32 %n, 0
  %wide.bound = sext i32 %n to i64
  br i1 %cmp1, label %for.body, label %for.end

for.body:
  %i.wide = phi i64 [ 0, %entry ], [ %i.wide.next, %for.body ]
  %s = phi i64 [ 0, %entry ], [ %add, %for.body ]
  %arrayidx = getelementptr inbounds i32, ptr %a, i64 %i.wide
  ...
  %i.wide.next = add nsw i64 %i.wide, 1
  %cmp = icmp ne i64 %i.wide.next, %wide.bound
  br i1 %cmp, label %for.body, label %for.end
```

with the remark `loop %for.body runs %n times when entered`.

## Strength reduction

`opt/src/lsr.rs`. In a rotated loop whose counter `i` steps by 1 and has
an exact exit test, when every other use of `i` is an index into an array
whose base does not change in the loop, `getelementptr T, ptr %a, i64 %i`,
used only as an address for loads and stores:

- each array gets a pointer of its own, `p = phi [&a[start], preheader],
  [p + 1, latch]`, and the indexing is replaced by it;
- the counter becomes `bound - i`, so the exit test is `c.next != 0`.

```llvm
; before                                  ; after
%i = phi i64 [ 0, %pre ], [ %i.next, %l ]   %p = phi ptr [ %a, %pre ], [ %p.next, %l ]
%q = getelementptr i32, ptr %a, i64 %i      %c = phi i64 [ %n, %pre ], [ %c.next, %l ]
%v = load i32, ptr %q                       %v = load i32, ptr %p
%i.next = add nsw i64 %i, 1                 %c.next = add i64 %c, -1
%t = icmp ne i64 %i.next, %n                %t = icmp ne i64 %c.next, 0
                                            %p.next = getelementptr i32, ptr %p, i64 1
```

At most two arrays become pointers, since each is a register live across
the loop. The rewrite pays off on machines with post-increment addressing,
where stepping the pointer is part of the load (`ldr w0, [x1], #4`), and
where a comparison with zero folds into the branch (`cbnz`): on AArch64
`sum_array`'s loop goes from five instructions to four. LLVM does this in
its code generator; here it is an IR pass, so it also shapes the IR other
back ends see.

## In the pipeline

After simplifycfg and [`implied`](ranges.md) (which needs the loop test
still at the top, dominating the body), before CSE, on each function:
rotate; then, if any loop
rotated, the combiner and simplifycfg again; then indvars and the combiner;
then strength reduction and the combiner.
`--no-loops` turns all of this off, and `loops` names it in `--passes=`.

On the inlining test program (clang without its own passes, then ours,
against clang `-O2 -fno-vectorize -fno-slp-vectorize`; entry blocks not
counted):

| | Blocks | Instructions |
| --- | --- | --- |
| Ours, `--no-loops` | 21 | 157 |
| Ours | 17 | 151 |
| LLVM | 9 | 129 |

Most of the remaining blocks are clamps, `x < lo ? lo : min(x, hi)`,
left as a branch and a select where LLVM makes `smax` and `smin`.

## Not yet

- Scalar evolution: counters other than `add` of a constant, steps other
  than ±1, trip counts of nested and multi-exit loops, exit values of other
  expressions (sums of the counter, for example).
- `lcssa` form, LICM, loop deletion, unrolling.
- Rotating loops with several latches or a latch ending in a conditional
  branch.
