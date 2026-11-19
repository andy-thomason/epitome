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
2. **Value ranges** (`opt/src/ranges.rs`, done): what each integer can
   be, as an interval, from constants, masks, casts, arithmetic, loop
   counters and, where a value is used, the branch conditions in force.
   `implied` asks them as part of its search.

For sources and status labels, see the [index](README.md).

## What dominating branches imply

A block reached only along the `true` edge of `br i1 %c` knows `%c` is
true, and when `%c = icmp ult %i, %n`, that `%i < %n`. So does every block
that block dominates. `implied` walks the dominator tree keeping the facts
in force: entering a child reached only along one edge of its parent's
conditional branch adds that edge's facts, and leaving it drops them. The
child may have other predecessors if they are back edges from blocks it
dominates: a loop entered only from that edge (an inner loop, already
rotated, inlined as the first thing in an outer loop's body).

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
- `x + k < b` when a fact says `x < b - k` and `k <= b` (`v[i + 1]` in
  `for i in 0..len - 1`);
- by `b`'s shape: `a <= x - 1` when `a < x`; a select's arms; `umin`, `umax`;
- by ranges: `a < b` when everything `a` can be here is below everything
  `b` can be (below);
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
  when `y <= x`; `add`, `sub` and `mul` get `nuw` and `nsw` when their
  operands' ranges here cannot make them wrap.

A select whose condition a fact gives is looked through wherever the
search meets it, not only where it is rewritten.

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
- Relations not written in a branch, beyond `x + k < b` from
  `x < b - k`: `v[i]` after `assert!(i + 4 <= len)` needs `i + 3 < len`
  from `i + 4 <= len`, which it does not work out. Nor sums of facts
  (`i < n` and `j < m` give `i + j < n + m`).
- Facts across calls or through memory; anything after a `switch` default.

## Value ranges

What `implied`'s facts cannot reach is what each value can be by itself:
an index `2 * i + 1` with `i` counting to 8, a parameter with a `range`
attribute, `i % len`, a counter running from 0 to a constant. These need a
range per value. (Comparing a mask, a shift or a `zext` with a constant was
already done by the combiner, from the bits that can be set.)

### The lattice

Each integer value of at most 64 bits gets two closed intervals, unsigned
`[umin, umax]` and signed `[smin, smax]`. The value is in both. Two
intervals rather than one interval that may wrap round (as a `range`
attribute's `[a, b)` may) keep the arithmetic simple and lose little: an
index is non-negative both ways, and a value whose range wraps in one view
is usually plain in the other. Each view narrows the other: unsigned
`[0, 9]` is signed `[0, 9]`; signed `[-3, -1]` is unsigned
`[2^w - 3, 2^w - 1]`. Full intervals mean "unknown".

### Where ranges come from

`ranges::Ranges::compute` makes one pass over a function in reverse
postorder, so each operand's range is known before its users', except along
back edges:

| Value | Range |
| --- | --- |
| constant `k` | `[k, k]` |
| parameter, call with a `range(iN a, b)` attribute | `[a, b - 1]` |
| `zext` | the operand's, unsigned |
| `sext` | the operand's, signed |
| `trunc` | the operand's, if it fits |
| `add`, `sub`, `mul`, `shl` by a constant range | interval arithmetic in 128 bits, kept when no bound leaves the type, or clamped when `nuw`/`nsw` says leaving it would be poison |
| `and` | `[0, min(umax)]` |
| `or`, `xor` | up to every bit either could set |
| `udiv`, `lshr` | `umax(x) / umin(y)`, `umax(x) >> umin(y)` |
| `urem x, y` | `[0, min(umax(x), umax(y) - 1)]` |
| `srem x, y`, `y > 0` | `x`'s sign, smaller than `y` |
| `ashr` | `x`'s signed interval shifted |
| `select` | both arms |
| `llvm.umin`, `umax`, `smin`, `smax` | from the operands |
| `llvm.ctpop`, `ctlz`, `cttz` of `w` bits | `[0, w]` |
| `icmp` | `[0, 1]` |
| counter (below) | `[start, bound]` |
| other phis | both (all) incoming ranges |

A phi with an entry along a back edge that is not a counter gets the full
range: there is no iteration to a fixed point, and no widening.

### Counters

A loop header phi `i = phi [start, outside], [i + 1, latch]` (or
`select %c, i + 1, i`, as `Range::next` leaves it) whose header ends in
`br (i < n), body, exit`, where `body` is entered only from the header and
dominates every latch, and `n` does not change in the loop: `i` goes up
only after the test passed, so it never passes `max(start, n)` and never
wraps. Its range is `[umin(start), max(umax(start), umax(n))]`. The same
holds signed, with `<=` (when `n + 1` does not wrap), with `!=` when `start
<= n`, and counting down by one while `> x`, `>= x` or `!= x`.

That is the loop shape before rotation, which is where `implied` runs.
With `for i in 0..8` it gives `[0, 8]`, and in the body, under `i < 8`,
`[0, 7]`.

### Ranges at a block

Where `implied` asks about a value, its range is narrowed by the facts in
force about it: under `x < y`, `umax(x)` is at most `umax(y) - 1`; under
`y < x`, `umin(x)` at least `umin(y) + 1`; signed alike; under `x == y`,
`y`'s range; under `x != 0`, at least 1. Then, up to two operations deep,
it is computed again from its operands' ranges at the block, so the facts
about `i` reach `2 * i + 1`. A select whose condition a fact gives is its
arm. Facts that contradict each other (an empty range) mean the block
cannot run, and nothing is concluded from them.

`a < b` then holds when `umax(a) < umin(b)` here (signed alike), and
`a != b` when the ranges do not overlap. This is one more way into
`implied`'s search, so the two combine: `i < len` from `i` in `[0, 7]` and
`len` at least 8.

### What ranges change

`opt/tests/examples/ranges.ll`:

- `for i in 0..8 { s += a[2 * i + 1] }` over `[u64; 16]`: the check goes,
  `a[2 * i + 2]`'s stays;
- an index with a `range(i64 0, 10)` attribute into 16 entries: the check
  goes; `x & 31` into 16 entries stays.

`add`, `sub` and `mul` get `nuw` and `nsw` where the ranges allow, as
`i * i` for a counter from 3 to 12 does in `loops.ll`.

In std's hello world, `implied` with ranges folds 503 comparisons and
sets flags on 449 instructions; without ranges, 346 and 149.

### What ranges cannot prove

- Relations between two unknown values. Ranges are about one value at a
  time; `i < len` with `len` a parameter is `implied`'s business.
- Through memory: a value loaded is unknown (`!range` metadata on loads is
  not read yet), as is a call's result without a `range` attribute.
- Loops other than counters stepping by one under their header's test;
  other phis on back edges are unknown. A counter is not followed after
  rotation, as `implied` runs before it.
- A loop bounded by something other than its counter: `put_u64`'s digit
  loop `loop { buf[i] = ..; i += 1; n /= 10; if n == 0 { break } }` keeps
  its check, as nothing bounds the number of digits.
- Non-intervals: `x` in `{0, 255}` is `[0, 255]`; bits known to be zero in
  the middle of a value (`x << 4`) are the combiner's known bits, not
  ranges.
- Wider than 64 bits.
- Not yet: `zext nneg` for a `sext` of a value never negative, and
  division by a value never zero.
