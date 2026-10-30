# Combines

The combiner rewrites instructions into simpler equivalents: `add(x, 0)`
becomes `x`, `add(x, x)` becomes `shl(x, 1)`, `add(sub(x, 1), 1)` becomes
`x`. The rewrites are rules in a small language, kept in
`opt/rules/combine.rules`, and a discrimination tree finds the rules that
match an instruction without trying them one by one. The code is in
`opt/src/combine/`. For sources and status labels, see the
[index](README.md).

The design is meant to grow beyond arithmetic to loads and stores, control
flow and target-specific patterns. Section 6 says what that needs.

## 1. Rules

One rule per line:

```text
name: pattern -> replacement [if condition]

add_sub_const: add(sub(x, C1), C2) -> add(x, C2 - C1)
mul_pow2:      mul(x, C) -> shl(x, log2(C)) if pow2(C)
trunc_zext_same: trunc(zext(x)) -> x if sametype(x, @)
```

- **Patterns** name operations as IR text does (`add`, `zext`, `select`,
  `icmp.ult`). A lower-case leaf matches any value, and a variable used
  twice must match the same value. A leaf starting with `C` matches an
  integer constant, scalar or splat. Literals match exactly: `0`, `-1`,
  `true`, `poison`, `-0.0`. Integers compare at the constant's own width,
  so `-1` means all ones.
- **Replacements** have the same shape, and may compute constants from the
  pattern's: `C1 + C2`, `-C`, `~C`, `log2(C)`, `bw - 1`. Expressions are
  computed exactly and truncated to the width where they are used.
- **Conditions** are comparisons of expressions, `pow2(C)`,
  `sametype(x, @)` and `oneuse`, joined by `&&`. `@` is the instruction
  being replaced, and `bw` is its integer width (for a comparison, the
  width it compares). `oneuse` holds when every operation the pattern
  looks through has no other use, so that the rule does not keep an old
  value alive beside the new one: `icmp.ne(add(x, C1), C2)` becomes
  `icmp.ne(x, C2 - C1)` only then, since otherwise both `x` and the `add`
  stay live (and a counter tested after its decrement can no longer be
  compared with zero).

Rules are tried in file order, so more specific rules go first.
Commutative operations (`add`, `mul`, `and`, `or`, `xor`, `fadd`, `fmul`,
`icmp.eq`, `icmp.ne`) match with their operands either way round, so a rule
is written once. Replacements carry no flags such as `nuw`, which is always
sound and sometimes loses information (**Open:** keep flags where a rule can
prove them).

The standard set has 123 rules: identities and annihilators,
cancellation, reassociation of constants, strength reduction, comparisons,
selects, casts, and the float rules that hold for signed zeros and NaN.

## 2. Matching

Every pattern, with each commuted form, is flattened in preorder into a
path of symbols, and all the paths go into one trie, a *discrimination
tree*. `add(sub(x, C1), C2)` becomes `add sub * C C`, where `*` is
"anything" and `C` is "any constant". Each node has a hash map of
operations, a hash map of literals, and at most one "any constant" and one
"anything" edge.

To match an instruction, the matcher walks its operand tree in the same
preorder alongside the trie, following only edges that fit the value in
hand:

- its operation, looked up in the node's map;
- the literals it equals. An integer constant equals at most two, its value
  read as unsigned and as signed (`255` and `-1` for an `i8` of all ones),
  so this is two lookups;
- "any constant", if it is one;
- "anything", which skips the value's whole operand tree.

Each step is constant time, and the walk only goes as deep as the patterns.
The cost depends on pattern depth and on how many patterns share a shape
with the instruction, not on how many rules there are. The tree checks only
shape; repeated variables and conditions are checked afterwards, for the
few rules that reach a leaf.

`cargo run --release --example match_bench` matches a 100,000-instruction
expression DAG against the standard rules plus N random rules:

| Rules | Trie nodes | Tree, ns per instruction | One rule at a time, ns per instruction |
| --- | --- | --- | --- |
| 123 | 477 | 258 | 386 |
| 223 | 6,348 | 212 | 548 |
| 1,123 | 64,361 | 228 | 2,667 |
| 10,123 | 642,073 | 351 | 37,888 |
| 100,123 | 6,259,320 | 1,176 | 546,359 |

These random rules are like real ones, mostly literal leaves. When most
leaves are variables instead, many rules genuinely match every instruction,
and the tree's time follows the number of matches it reports (about 27 ns
each); trying rules one at a time is still 20 to 50 times slower.

## 3. The pass

`combine::run` works on one function, from a list that starts with every
instruction in program order. For each it:

1. deletes it if nothing uses it and it has no side effects;
2. moves a constant left operand of a commutative operation or comparison
   to the right (swapping the predicate of a comparison);
3. folds it: all-constant integer operations, comparisons, casts, selects,
   float arithmetic without fast-math flags, operations on `poison`, phis
   whose entries are all one value, and loads from constant globals (see
   below);
4. otherwise tries the matching rules, best first. A rule applies when its
   repeated variables agree, its condition holds, its replacement
   type-checks, and it makes no more new instructions than the match
   leaves unused;
5. otherwise tries the built-in steps about bits: known bits, funnel
   shifts, `extractvalue` of `insertvalue`, and demanded bits (see below);
6. otherwise collects it as a linear combination, or reassociates it, if
   it is the top of a sum or a chain (see below).

### Reassociation

This is what LLVM's `reassociate` pass does, as a step of the combiner.
At the top of a chain of one associative, commutative integer operation
(`add`, `mul`, `and`, `or`, `xor`), the leaves are collected through
operands that are the same operation and have no other use. The chain is
then rebuilt leaning left, with its leaves sorted by rank:

- parameters first, in order;
- then instructions, in the order they were made;
- constants last.

Two things follow:

- Constants gather at the outside, where the constant rules fold them:
  `(x + 3) + y + 5` becomes `(x + y) + 8`.
- Chains with the same leaves come out the same however they were
  written, so CSE merges them. In `tests/examples/reassociate.ll`, `(c + a)
  + b` and `(a + b) + c` merge, their `xor` becomes 0, and the function
  becomes `a + b + 8`.

A chain already in that shape is left alone, and only its top is rebuilt,
so the step always ends. It is a built-in step, not a rule. A rule's
variables can match the chain's inner nodes as well as its leaves, and
those carry ever newer ranks, so a rule would keep moving them. Flags such
as `nsw` are dropped, since they were only true of the order written.

**Open:** a chain is taken at most 256 leaves deep from its top. Nodes
below that are left in the order written: they are part of a longer chain,
so they leave the work to its top.

### Linear combinations

A sum is often written so that a value appears in it more than once:
`f(x, x + 1, x + 2)` inlined into `a - b + c` gives `x - (x + 1) + (x +
2)`. At the top of an `add` or `sub`, the step walks down through operands
that are `add`, `sub`, or `shl` or `mul` by a constant, with no other use,
giving each leaf a coefficient: `+1`, `-1` under a `sub`, times `2^k` under
a `shl`. A leaf's coefficients are added modulo the width, and constants
add up into one. The sum is rebuilt only when that takes fewer
instructions than it did: the positive terms in rank order, then the
negative ones subtracted, then the constant. A term whose coefficient is
not 1 is a `shl` (a power of two) or a `mul`. With no positive term the
constant comes first, `sub K, x`.

`tests/examples/linear.ll` has examples: `(x + 1) - (x + 3) + x - 2 * (x +
7)` becomes `-16 - x`. Since a rebuild must be smaller, the step ends, and
reassociation still sorts what it leaves. `tests/linear.rs` checks the two
steps against an evaluator: 400 random functions of `add`, `sub`, `mul` and
`shl` at 8, 32 and 64 bits, each on 144 pairs of inputs.

### Bits

SROA packs a small struct into an integer and takes it apart again with
shifts, masks and truncations (see [sroa](sroa.md)). After inlining, most of
that packing is undone by four steps. They work from `ones`, the bits of a
value that may be set: a constant's own bits, through `and`, `or`, `xor`,
shifts by constants and `zext`, and every bit otherwise. The search stops six
levels down.

- **Known bits.** An `and` with a constant that keeps every bit that may be
  set is its operand. A shift left and back right by the same amount (or
  right and back left) that loses none of them is the value shifted.
  `zext (trunc y)` is `y` when `y` fits in the narrow type. A value with no
  bit that may be set is 0. A comparison with a constant is decided when
  the value's range, `[0, ones]`, decides it: `(x & 15) < 0` is false, and
  `(x & 15) > 50` is false. Clamps of a masked value then fold away.
- **Demanded bits.** Some instructions read only some bits of their
  operand: `trunc` the low ones, `lshr` and `ashr` by a constant the high
  ones, `shl` by a constant the low ones, and `and` with a constant those
  set. Such an instruction may read them from a simpler value, already
  made, that has the same bits there. Through an `or`, one side can go if
  it cannot set any of those bits. Through an `and` with a constant, the
  constant must keep them all. Through `zext (trunc y)`, only the narrow
  bits may be wanted. The operand is changed in place, and the
  instruction's value stays the same. The low half of
  `(x & 0xffffffff00000000) | zext y` is read from `y`.
- **Funnel shifts.** `(x << a) | (y >> (w - a))` is `llvm.fshl(x, y, a)`,
  a rotation when `x` and `y` are the same. Code generation selects it as
  one instruction (`extr`, `ror`) where there is one, and otherwise writes
  it back as the two shifts and the `or`. The pipeline declares
  `llvm.fshl` for 8, 16, 32 and 64 bits before its parallel phase, and
  removes any declaration nothing uses.
- **`extractvalue` of `insertvalue`.** It is the inserted element, if
  inserted at the same place. Below insertions at other places, the
  extraction reads the aggregate underneath instead.

A float constant's `bitcast` to an integer of its width also folds, to its
raw bits.

`tests/linear.rs` checks these steps against an evaluator, as it does
linear combinations: 500 random functions on `i64` of masks, `or`, `xor`,
shifts, `add` and `trunc`/`zext` pairs, each on 81 pairs of inputs. A third test
is exhaustive for comparisons: every predicate, against constants around
each of five masks, on every value the mask lets through. Weakening
a condition of any of these steps (the `and` in demanded bits, the
widths of a funnel shift, the mask in known bits) makes it fail.

### Bit maps (planned)

The steps above each know one shape. Most code that moves bits around,
rather than computing with them, can be handled by one analysis instead:
say, for each bit of a value, where it comes from, and then find the
cheapest operation that puts the bits there.

**The map.** A value of any width, scalar or vector (up to 512 bits), gets
a *bit map*: for each of its bits, one of

- `0` or `1`, a known constant bit;
- `s[i]`, bit `i` of a source value `s`, with up to four sources;
- `?`, a bit that is poison or undefined, which matches anything.

A vector's bits are numbered lane by lane, low lane first, as the data
layout lays them out in memory (little-endian), so that a `bitcast`
between a vector and an integer of the same width leaves the map as it is.

The map is built by walking down from a root, at most eight levels, through
instructions whose output bits are each a copy of one input bit or a
constant:

- `trunc`, `zext` and `sext` (whose top bits are copies of the sign bit);
- `shl`, `lshr` and `ashr` by a constant, and rotates (`llvm.fshl` and
  `llvm.fshr` by a constant);
- `and` and `or` with a constant (each bit is the input's, or `0` or `1`),
  and `or` of two maps that never both have a source bit in the same place
  (a `disjoint` or, or one the maps show is);
- `bitcast`, `llvm.bswap` and `llvm.bitreverse`;
- `shufflevector`, and `extractelement` and `insertelement` at constant
  indices;
- a load, from the store that reaches it (its clobber in memory SSA), when
  the two addresses differ by a known constant: the load reads bits
  `[8o, 8o + w)` of the stored value, at offset `o` bytes into it. A load
  that several narrower stores cover, together, gets a map with a source
  for each.

Anything else is a leaf: a source whose bits are taken as they are.

**Matching.** The map then goes down a table of operations, cheapest first,
each a test of the map's shape:

| Shape of the map | Operation |
| --- | --- |
| `s[0..w)` in place | `s` itself, through a `bitcast` if the type differs |
| the low bits of one source | `trunc` |
| a source's bits, then zeros, or copies of its top bit | `zext`, `sext` |
| a source's bits moved up or down by `k`, zeros or sign copies in the rest | `shl`, `lshr`, `ashr` |
| moved round modulo the width | a rotate |
| a contiguous field of a source, moved to bit 0 | `lshr` and `and`, which selection makes `ubfx` |
| a field of one source written into another, in place | `and`/`or` with masks (`bfi`) |
| bytes reversed, or bits | `llvm.bswap`, `llvm.bitreverse` |
| whole lanes of one or two sources, rearranged | `shufflevector`, its mask looked up in the description's shuffles |

For the last line the map is cut into lanes, trying the widest lane width
first: a byte permute that moves whole 32-bit lanes is a `<4 x i32>`
shuffle. The mask is then looked up in the shuffles the description has
patterns for, at that lane width (`rev16`, `rev32`, `rev64`, `zip`,
`uzp`, `trn`, `ext`, `dup`, `ins` on AArch64). It is the same table
`legalize.rs` asks with `has_shuffle`, so the result is one instruction.
Where no single shuffle fits, two that do may later be tried, before the
fallback of building the vector lane by lane.

The rewrite is made only when the operation found takes fewer instructions
than the instructions it replaces with no other use. It is an elective
change (autochop).

**What it covers.** Examples, each now written as several instructions or
as memory traffic:

- A 64-bit store followed by an 8-bit load of its byte 3 is
  `trunc (lshr v, 24)`, with no load. Four 16-bit stores followed by one
  64-bit load is the four values zero-extended, shifted and or-ed together,
  which the map then may find to be a single source (a store of a value
  taken apart).
- A stack slot used as a bitcast, `store <16 x i8>` then `load i128` (what
  code generation now does for bitcasts it has no instruction for), is
  the `bitcast` again, where the description has one.
- `(x >> 8 & 0xff) | (x << 8 & 0xff00)` on an `i16` is `llvm.bswap`;
  shifting bytes into place one by one, as a hand-written byte reversal
  does, is too.
- A shuffle written out lane by lane through `extractelement` and
  `insertelement` is one `shufflevector`, and if its mask is in the table,
  one instruction.
- SROA's packing (see [sroa](sroa.md)) and the known-bits and
  demanded-bits steps above become special cases. They can stay as cheap
  first tries.

This is the *store-to-load bridge*: forwarding a stored value to a load of
another width or type, which `forward_store` cannot do since it requires
the same type.

**Checking.** As `tests/linear.rs` does for the steps above: random trees of
the instructions the map walks through, on scalars and vectors of every
width up to 128 bits, are evaluated on random inputs before and after the
rewrite, and must agree on every bit that is not `?` in the map. Memory
cases are checked against the model of memory in `tests/combine_rules.rs`,
widened to bytes, so that a load sees the bytes the last stores to them
wrote.

### Three-input bit functions (planned)

Any function of three bits is described by its truth table, eight bits
long. AVX-512's `vpternlogd` and `vpternlogq` compute any of the 256 in
one instruction, the table in its immediate; SVE2 has a few of them as
fixed instructions (`eor3`, `bcax`, and `bsl`, `bsl1n`, `bsl2n` and
`nbsl`). A chain of `and`, `or`, `xor` and `not` that reads only three
values can therefore be one instruction where it is now several. Code
that does logic a bit plane at a time (the simulators of
[SystemVerilog](sv.md) section 8.3, bit-sliced hashing and ciphers)
is made of such chains: a full adder's sum and carry, a multiplexer, the
majority of three.

**Finding them.** From a root `and`, `or` or `xor` on integers or integer
vectors, walk down through `and`, `or`, `xor` (a `not` is an `xor` with
all ones) and constants of all zeros or all ones, collecting the other
operands as leaves, and stop if a fourth distinct leaf appears. The table
is the tree evaluated on the bytes `0xf0`, `0xcc` and `0xaa` in place of
the three leaves: bit `k` of the result is the function's value when the
inputs are bits `k` of those three bytes. A table that ignores one
input is a two-input function and is left to the rules above.

**The canonical form** is a call:

```llvm
%r = call <16 x i32> @llvm.epitome.bitfn3.v16i32(i8 150, <16 x i32> %a, <16 x i32> %b, <16 x i32> %c)  ; a ^ b ^ c
```

The rewrite is made when the description has an instruction for that
table at that type, as `has_shuffle` asks for shuffles, and when it
removes at least one instruction (intermediate values with other uses
stay). It is an elective change (autochop). Two `bitfn3` calls where
one feeds the other with no other use, together reading at most three
values, fold into one.

**Selection.** In the x86 description, `VPTERNLOGD` and `VPTERNLOGQ` are
`"asmonly"` today. Their bodies become the call above with the immediate
as the table, so every table selects; with AVX-512VL the same holds at
128 and 256 bits. The masked forms (`{k}`, `{z}`) come later, with the
rest of AVX-512's mask selection. On AArch64, SVE2's entries each give
their fixed table; a table with no entry is not formed, so nothing needs
expanding back. (The AArch64 description has no SVE2 entries yet.)

**Checking.** As for the rules above: random trees of the bitwise
operations over three leaves, at widths up to 512 bits, evaluated on
random inputs before and after the rewrite. Each instruction that
claims a table is checked exhaustively: on the three bytes `0xf0`,
`0xcc` and `0xaa` it must give the table back.

**Later:** a function of four inputs is two `bitfn3` and a select on the
fourth (`f = d ? f1 : f0`, which is itself a `bitfn3`), so three
instructions; four-state logic has many of these.

### Loads from constant globals

This is the local part of LLVM's `globalopt` (`combine/constload.rs`). A
load from a constant global, at a constant offset reached through
`getelementptr`s (as instructions or constant expressions), folds to the
value in the global's initialiser. The value is read out through arrays,
vectors, structs, byte strings (an integer may be read from any offset)
and zero initialisers.

The global must be `constant`, have an initialiser, and not be
`externally_initialized`. Its definition must be one that cannot be
replaced at link time: not `weak`, `linkonce`, `common` or `extern_weak`.
The `_odr` forms are fine, since they promise every definition is the same.

### Memory

With memory on (the pipeline's default; `--no-memory` turns it off), a run
of the combiner builds memory SSA and its walker (`memssa.rs`) and has rules
and steps about memory. The walker's dominator tree and blocks are from the
start of the run. The combiner's edits only make its answers more cautious:
replacing values with equal ones and removing loads or pure instructions
change no write, and folding branches only takes paths away. A store the
combiner deletes is told to the walker, which then walks past it.

**Rules.** A load pattern may name the write that reaches it, its clobber,
with `@`:

```text
forward_store:    load(p) @ store(v, q) -> v if mustalias(p, q) && sametype(v, @)
```

The rule applies when the walker finds the load's clobber exactly and it is
a store that matches. `mustalias(p, q)` asks alias analysis whether the
two addresses are the same. `tests/combine_rules.rs` checks memory rules
against a model of memory: a load sees the stored value exactly when the
addresses and the types are the same. So a rule must require both, and its
replacement must be the stored value. Loads may only be the root of a
pattern and stores only follow `@`, so other rules are unchanged.

**Load reuse.** An earlier load of the same address and type that comes
before this one on every path, and has the same exact clobber, gives its
value. Nothing between them can have written the bytes. Unlike CSE's, this
works across joins: a load after a diamond reuses one from before it when
neither arm writes. Up to 16 candidates are tried.

**Dead stores.** A store is deleted when either holds:

- **It stores the value already there:** the value of a load of the same
  bytes, with the same exact clobber.
- **Nothing can read it:** a search forward over the CFG finds, on every
  path, a store covering the same bytes (from the same address, at least
  as wide) before anything that may read them, or the end of a path where
  nothing can read them:
  - an `unreachable`;
  - a return, when the memory is a local of the function that never
    escapes;
  - coming back round a loop to the store itself.

  A call that may unwind ends the search, unless the memory is local, since
  an exception handler could read it. The search looks at 256 instructions
  at most.

This is the "is this store read before it is overwritten?" question that
`postdomtree` was planned for. A forward search answers it directly,
without one.

### Larger patterns

These are the expression patterns LLVM's `aggressive-instcombine` looks
for, as ordinary rules (the section of `combine.rules` named after it):

- factoring out a common operand: `x*y + x*z` becomes `x*(y + z)`, and
  `x*C + x` becomes `x*(C+1)`;
- distributing: `(x & y) | (x & z)` becomes `x & (y | z)`, and likewise
  for `xor` of `and`s and `and` of `or`s;
- the identities of the `and`, `or` and `xor` of the same two values:
  `(x & y) + (x | y) = x + y`, `(x ^ y) + (x & y) = x | y`,
  `(x | y) - (x & y) = x ^ y`, and so on.

All of them pass the brute-force soundness checker.

The pass keeps a use list for every value, so replacing a value costs the
number of its users, not the size of the function. A replaced
instruction's users go back on the list, since they may now match
something, and so do its operands, which may now be unused. New
instructions are queued to go before the one they replace, and the blocks
are rebuilt once at the end. Debug records follow replaced values; records
of deleted values become `poison` (optimised out).

A work limit (20 visits per instruction) stops a set of rules that undo
each other from looping forever; hitting it shows in the statistics.

In the pipeline, each function is combined after inlining and recursion
removal, before it is published, so callers see the simpler body when they
decide what to inline. `epitome-opt --passes=combine` runs it alone;
`--remarks` lists the rules that fired per function, and `--stats` totals
them for the module.

## 4. Checking the rules

`opt/tests/combine_rules.rs` checks every rule by brute force. It
evaluates the pattern and the replacement with the folding code, which
follows the LangRef: shifts by the width or more give poison, division by
zero and signed overflow in division are undefined behaviour, and so on.
Wherever the pattern gives a value that is neither poison nor undefined,
the replacement must give the same one.

- Integer rules are tried on every input at widths 1 to 4, on every input
  at width 8 when they have at most two variables (a sample otherwise), and
  on a sample at width 64.
- Float rules are tried at `float` and `double` precision on signed zeros,
  infinities, NaN, denormals and ordinary values.
- Rules with casts or `freeze` change widths or are nondeterministic, so
  they are listed by name and covered by the example and program tests. A
  new rule of that kind that is not listed fails the test.
- The test also feeds the checker known-wrong rules (`x + 0.0 -> x`, a
  shift that overflows at small widths, an off-by-one constant) to make
  sure it catches them.

The checker found a bug on its first run: `add(x, x) -> shl(x, 1)` is wrong
for `i1`, where shifting by 1 is poison but `x + x` is 0. The rule now has
`if bw > 1`, and a separate rule handles `i1`.

**Open:** variables are never poison or undef in the checker, so rules
whose soundness depends on how poison propagates (such as turning a select
into `and`) need care. None of the standard rules do this.

## 5. Parallel performance

The combiner reads constants and types on every instruction it visits.
With the tables behind a read-write lock, every read was an atomic update
of the lock's cache line, and eight threads ran slower than one. The
tables (`opt/src/table.rs`) are now frozen at the start of a phase, so
existing entries are read with no synchronisation. Entries added during
the phase go into an append-only list whose entries never move, so they are
read without a lock too, and only adding an entry takes a lock.

On the 50,000-function synthetic module (16 layers):

| Threads | 1 | 2 | 4 | 8 |
| --- | --- | --- | --- | --- |
| Combine only | 437 ms | 235 ms | 148 ms | 99 ms |
| Whole pipeline | 4431 ms | | | 699 ms |

The output is identical for every thread count.

## 6. Growing it

The matcher knows nothing about the IR beyond the `Subject` trait: a
value's operation and operands, the literals a constant equals, and whether
it is an integer constant. Extending combines means adding operations to
`Head`, and to the rule language and the soundness checker.

- **Loads and stores.** Add `load` and `store` heads, with operands for the
  address and value. Store-to-load forwarding and dead-store rules need
  more than shape: that two addresses are the same or cannot alias, and
  that nothing in between writes memory. That takes a new kind of
  condition, answered by alias analysis, and a way to say "the previous
  store to this address", which is a new kind of edge in the operand tree.
- **Control flow.** Done for `br` and `condbr`: block operands are `^name`
  variables, and a rule rooted at a branch is applied as an edit to the
  branch and the target phis. See [simplifying the CFG](simplifycfg.md).
  `switch` and other terminators with any number of operands would need
  variadic patterns.
- **Target rules.** Instruction selection is the same problem with
  patterns over generic operations and target instructions as
  replacements. The tree can be reused as it is.
- **Generated matchers.** The tree is built at start-up from the text, which
  takes well under a millisecond for the standard rules. A build script
  could generate Rust code from the same rules if start-up ever matters.

Other open work: rules over vectors with different elements (only splats
match `C`), integers wider than 64 bits, keeping `nuw`/`nsw` where sound,
and `undef`.
