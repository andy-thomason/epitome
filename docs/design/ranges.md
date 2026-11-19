# Ranges: what branches imply, and what values can be

Bounds checks are the reason for this work. Rust checks every slice index:

```rust
fn sum(v: &[u64]) -> u64 { let mut s = 0; for i in 0..v.len() { s += v[i]; } s }
```

becomes, once `Range::next` is inlined, a loop whose test is `i < len`
and whose body tests `i < len` again before the load, with a branch to a
call that panics. The second test can never fail. Removing it needs to know
what holds where: here, that the loop's own test dominates the check.

Two pieces do this, from the LangRef and from LLVM's behaviour as a program
(what `opt` does to such loops, not how):

1. **What dominating branches imply** (`opt/src/implied.rs`, done): the
   comparisons a branch established on the path to a block, and what
   follows from them. LLVM calls its versions correlated value propagation
   and constraint elimination.
2. **Value ranges** (planned): what each integer can be, as an interval,
   from constants, masks, casts, loop counters and branch conditions.

For sources and status labels, see the [index](README.md).

## What dominating branches imply

A block reached only along the `true` edge of `br i1 %c` knows `%c` is
true, and when `%c = icmp ult %i, %n`, that `%i < %n`. So does every block
that block dominates. `implied` walks the dominator tree keeping the facts
in force: entering a child reached only along one edge of its parent's
conditional branch adds that edge's facts, and leaving it drops them.

A fact is `a rel b` with `rel` one of `<` and `<=` (unsigned or signed),
`==` and `!=`; `>` and `>=` are turned round. A condition gives:

- itself: `%c == true` (or `false` on the other edge);
- for `icmp`, its relation, or the negation on the `false` edge
  (`!(a < b)` is `b <= a`);
- for `and` true (or `select %a, %b, false`), both sides' facts; for `or`
  false (or `select %a, true, %b`), both sides' negations; through
  `xor %c, true`;
- for a `switch` case reached by one value only, `%x == value`.

It is sound because both sides of the fact are SSA values defined in blocks
that dominate the branch, which dominates the block that uses the fact: on
any path to that block, neither has been computed again since the branch.

### Bounds a fact brings with it

When a fact `a < b` (or `<=`) is added, so are `a < w` for the values `w`
that `b`'s shape alone puts at or above it (up to six):

| `b` | `w` |
| --- | --- |
| `lshr x, k`, `udiv x, k`, `urem x, k` | `x` |
| `and x, y` | `x` and `y` |
| `sub nuw x, y` | `x` |
| `umin(x, y)`, as `llvm.umin` or `select (x < y), x, y` | `x` and `y` |

So `i < len / 2` also gives `i < len`, and `i < a.len().min(b.len())`
gives `i < a.len()` and `i < b.len()`, and checking either index is a
lookup.

### Proving

A question `a rel b` is answered by a search, at most six deep and 96
steps per question:

- equal values, constants, `0 <= x`, `x <= all ones`;
- through a fact about `a` or `b` and a smaller question: `a < y` and
  `y <= b`, `a <= y` and `y < b`, `a == y` and `y < b`;
- by `a`'s shape: `x - y <= x` when `y <= x`; `x - k < x` when `x >= k`
  (for `k == 1`, when `x != 0`); `x / k`, `x >> k` and `x & y` are at
  most `x`, and the first two below it when `x != 0` and `k` divides or
  shifts; `x % y < y`; a select's two arms, each
  with its condition assumed; `umin` and `umax`;
- by `b`'s shape: `a <= x - 1` when `a < x`; a select's arms; `umin`, `umax`;
- `x != 0` when some fact puts a value below `x`, or a fact says so;
- signed `<` and not negative is unsigned `<`.

`rev` shows the search at work:

```rust
fn rev(v: &mut [u8]) { let n = v.len(); for i in 0..n / 2 { v.swap(i, n - 1 - i); } }
```

Under the loop's `i < n / 2` (and so `i < n`), the check of `n - 1 - i`
is `(n + -1) - i < n`: the subtraction does not wrap because `i <= n - 1`
(as `i < n`), and `n - 1 < n` because `n != 0` (something, `i`, is below
it). Both checks go.

### What it changes

For each block, with its facts:

- an `icmp` the facts decide becomes `true` or `false` everywhere (it is
  evaluated where the facts hold); the combiner then folds the branch, and
  simplifycfg removes the call that panicked;
- an operand that is `select %c, %x, %y`, where `%c` is known, becomes `%x`
  or `%y`. This is per use: `Range::next` builds `Option` as
  `select %more, %i, undef`, and below the loop's test that is `%i`. A phi
  entry is a use at the end of its predecessor, with that edge's facts;
- an `i1` operand a fact gives is replaced by the constant: a condition
  tested again below its own branch;
- `add x, 1` gets `nuw` when something is above `x`; `sub x, y` gets `nuw`
  when `y <= x`.

In the pipeline it runs after simplifycfg and before rotation, while a
loop's test is still at the top, dominating the body, followed by the
combiner and simplifycfg when it changed something. `--no-implied` turns it
off; `implied` names it in `--passes=`.

The combiner gained the rules that make the loop test plain: a `bool`
widened to a byte and compared with 0 or 1 is the `bool`
(`icmp ne (zext %c), 0` is `%c`), and `select %c, 1, 0` is `zext %c`.
Rotation then needed the `i + 1` the header computes moved to the latch
([loops](loops.md)), and induction variables no longer need `nuw` or
`nsw` on the increment to make the exit test exact: the guard and the test
bound the counter.

### Results

`opt/tests/examples/bounds_checks.ll` has `sum`, `dot` and `rev` in the
shape epitome-rustc2 gives them, and a function whose checks must stay. All
three checked loops lose their checks, rotate and get exact exit tests;
`sum` and `dot` then step by pointer:

```llvm
ok:                                               ; preds = %ok, %start
  %lsr.ptr = phi ptr [ %v, %start ], [ %lsr.ptr.next, %ok ]
  %s = phi i64 [ 0, %start ], [ %s.next, %ok ]
  %lsr.count = phi i64 [ %len, %start ], [ %lsr.count.next, %ok ]
  %x = load i64, ptr %lsr.ptr
  %s.next = add i64 %s, %x
  %lsr.count.next = add i64 %lsr.count, -1
  %lt = icmp ne i64 %lsr.count.next, 0
  %lsr.ptr.next = getelementptr i64, ptr %lsr.ptr, i64 1
  br i1 %lt, label %ok, label %done
```

`epitome-rustc2/tests/run/bounds.rs` runs the three, and loops whose
checks stay, over slices of every length.

### What it cannot do

- Facts that hold round a loop rather than down the dominator tree. After
  rotation, `i < len` in the body holds because it held on entry and the
  bottom test keeps it; `implied` does not see that, which is why it runs
  before rotation. Value ranges (below) cover induction variables.
- Relations not written in a branch: `v[i]` after `assert!(i + 4 <= len)`
  needs `i + 3 < len` from `i + 4 <= len`, arithmetic on facts it does not
  do. Nor sums of facts (`i < n` and `j < m` give `i + j < n + m`).
- Facts across calls or through memory; anything after a `switch` default.

## Value ranges (planned)

What `implied` cannot reach is what each value can be by itself: an index
`i & 15` into `[T; 16]`, a `u8` widened to index a 256-entry table,
`i % len`, a counter running from 0 to a constant. These need a range per
value.

### The lattice

Each integer value of at most 64 bits gets two closed intervals, unsigned
`[umin, umax]` and signed `[smin, smax]`. The value is in both. Two
intervals rather than one wrapped interval (LLVM's `!range` metadata allows
`[a, b)` wrapping round) keep the arithmetic simple and lose little:
an index is non-negative both ways, and a value whose range wraps in one
view is usually plain in the other. The full ranges mean "unknown"; an
empty one means the value is never computed (unreachable code).

### Where ranges come from

| Value | Range |
| --- | --- |
| constant `k` | `[k, k]` |
| `zext` from `w` bits | unsigned and signed `[0, 2^w - 1]` |
| `sext` from `w` bits | signed `[-2^(w-1), 2^(w-1) - 1]` |
| `trunc` | the operand's, if it fits; else full |
| `and x, k` | unsigned `[0, k]` (and `[0, umax(x)]`) |
| `or x, k` | unsigned at least `k` |
| `urem x, y` | unsigned `[0, umax(y) - 1]` |
| `udiv x, y`, `lshr x, y` | `umax(x) / umin(y)`, `umax(x) >> umin(y)` |
| `ashr x, k`, `sdiv x, k` | signed interval of `x` shrunk |
| `add`, `sub`, `mul`, `shl` | interval arithmetic, when no bound wraps (or the flag forbids it) |
| `select`, `umin`, `umax`, `smin`, `smax` | from the arms |
| `icmp`, `zext` of `i1` | `[0, 1]` |
| `ctpop`, `ctlz`, `cttz` of `w` bits | `[0, w]` |
| `load` or call with `!range`, call with a `range` return attribute | as given |
| induction variable (below) | `[start, bound]` |
| other phis | the union of the incoming ranges |

They are computed in reverse postorder, so each operand's range is known
first, except along back edges. A loop header's phis are handled as
induction variables when `indvars` recognises them, and otherwise get the
full range: one pass, no fixed-point iteration or widening.

### Induction variables

For a counter `i = phi [start, pre], [i + 1, latch]` whose loop goes round
while `i + 1 < n` (unsigned; signed alike), entered only when `start < n`
(the guard): `i` is in `[umin(start), umax(n) - 1]` and `i + 1` in
`[umin(start) + 1, umax(n)]`, and neither wraps. Counting down by 1 is the
mirror. With a constant bound this is a constant range: `for i in 0..16`
gives `i` in `[0, 15]`. This is what `implied` cannot see after rotation.

### Ranges at a block

A range is refined where it is used by the facts `implied` keeps: under
`x < k`, `umax(x)` is at most `k - 1`; under `x < y`, at most
`umax(y) - 1`; under `x == k`, it is `k`. The refinement is per block, as
the facts are; the value's own range stays as computed.

### What ranges change

- An `icmp` whose operands' ranges decide it folds: `i & 15 < 16`, `u8 as
  usize < 256`, `i % n < n`, a counter in `[0, 15]` against an array of 16.
- In `implied`'s search, `a < b` holds when `umax(a) < umin(b)`, so the two
  combine: `i < len` with `len` known to be at least 16 and `i` in `[0, 15]`.
- `add`, `sub`, `mul` and `shl` get `nuw` or `nsw` when the ranges show no
  wrap; `sext` of a value that is never negative becomes `zext nneg`.
- `udiv` and `urem` by a value never zero need no check by the code
  generator; a select whose condition's ranges decide it folds.

### What ranges cannot prove

- Relations between two unknown values. Ranges are about one value at a
  time; `i < len` with `len` a parameter is `implied`'s business, and
  facts like `i + j < n` are neither's.
- Through memory: a value loaded without `!range` is unknown, as is a
  call's result without a `range` attribute.
- Loops other than counters stepping by 1 with one exit test, and phis on
  back edges that are not induction variables (no iteration to a fixed
  point).
- Non-intervals: `x` in `{0, 255}` is `[0, 255]`; bits known to be zero in
  the middle of a value (`x << 4` has its low four bits clear) are not
  tracked. Known bits would be a separate analysis.
- Wider than 64 bits.
