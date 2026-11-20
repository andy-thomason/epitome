# Simplifying the CFG

Control-flow simplification is split in two. Branches are instructions,
so the [combiner](combine.md) simplifies them with rules like any other.
What changes the shape of the graph is `opt/src/simplifycfg.rs`. For
sources and status labels, see the [index](README.md).

## Branches, in the combiner

The rule language has two branch operations, `br(^a)` and
`condbr(c, ^a, ^b)`. Block operands are variables written `^name`, which
the matcher treats like any other wildcard: a block operand is the block's
label. A rule rooted at a branch must rewrite it to a branch:

```text
br_true:  condbr(true, ^a, ^b) -> br(^a)
br_false: condbr(false, ^a, ^b) -> br(^b)
br_same:  condbr(c, ^a, ^a) -> br(^a)
br_not:   condbr(xor(c, true), ^a, ^b) -> condbr(c, ^b, ^a)
```

Applying one is an edit rather than a replacement of a value. Each edge
the new branch no longer has takes one entry out of the target's phis
(phis have one entry per edge), and those phis go back on the worklist,
since they may now fold. A `switch` on a constant, or whose targets are all
the same, becomes a `br`. This is built in, because a switch has any number
of operands.

A branch's weights (`!prof` `branch_weights`, one per successor) stay
right through these edits: `br_not` swaps them with the targets, and a
branch that becomes plain, or whose targets change otherwise, loses them
(here and wherever a pass makes a branch plain). The weights come from the
front end, or from `llvm.expect`, which each function's task turns into
weights before anything else (after promoting its locals, so the branch
testing the expected value is in sight), as LLVM's `lower-expect` does:
the side expected gets 2000 and the other 1 (or the given probability, for
`llvm.expect.with.probability`), seen through casts, `icmp eq`/`ne` and
`xor` with constants; then the call is replaced by its value
(`opt/src/expect.rs`, `ir/src/prof.rs`). The code generator places
blocks by them ([codegen](codegen.md)).

When a comparison folds to a constant, the branch using it is one of its
users, so it goes back on the list and the branch rule fires: constants
flow into control flow without a separate pass.

The soundness checker covers branch rules too. Branching on poison is
undefined behaviour; wherever the pattern's behaviour is defined, the
replacement must go to the same block.

## The graph

`simplifycfg::run` repeats these until nothing changes:

- **Unreachable blocks** are deleted, with their phi entries.
- **Empty blocks**, holding only `br label %next`, are bypassed: their
  predecessors branch to `%next` directly, and `%next`'s phis get an entry
  for each new edge. This is not done when a predecessor already branches
  to `%next` with a different phi value, since one block cannot send two
  values along edges to the same place. Only `br`, `condbr` and `switch`
  predecessors are retargeted.
- **Diamonds and triangles** become `select`s. When the arms of an `if`
  are empty blocks (or one arm is missing), the branch only chooses values
  for the phis where the arms meet. Those become selects in the branching
  block, which then jumps straight to the join, for at most 8 phis per
  join. An arm may also compute up to two values that are safe to compute
  when it would not have run: pure, and unable to trap, as for `licm`
  (division only by a constant that cannot trap). Those move before the
  branch first. A clamp, `x < lo ? lo : x > hi ? hi : x`, then becomes two
  selects with no branch.
- **Straight lines** merge: a block that is the only successor of its only
  predecessor joins it.
- **Edges into `unreachable`** go. A block that is nothing but
  `unreachable` cannot be reached without undefined behaviour, so a
  conditional branch to one becomes a plain branch to its other target,
  and a switch loses its cases that lead there. A default that leads there
  is replaced by the first live case's target, and that target's cases
  then go too. A branch whose every edge leads there becomes `unreachable`
  itself, so this works back through chains of tests. A block that does
  something first, such as calling a function that does not return, is a
  real path and stays. `Option::unwrap_unchecked`, a test of the tag and
  a branch to `unreachable` for `None`, becomes a single move.

Functions whose blocks have their address taken are left alone, since a
`blockaddress` may name any block.

In the pipeline, each function is combined, then simplified, then
combined again from the selects and phis the simplification touched, for
up to three rounds, before CSE.

`opt/tests/examples/simplifycfg.ll` shows each case, and
`unreachable_edges.ll` the edges into `unreachable`.

Before the call graph is built, `opt/src/prefold.rs` folds branches and
switches on constants (and on comparisons of two constants) and deletes
the blocks left unreachable, so that calls that can never run do not tie
functions into call-graph cycles (see [parallel.md](parallel.md)).
`prefold.ll` shows it.

## Effect

On the test programs built by clang without its own passes:

| | Blocks | Branches on a constant | Instructions |
| --- | --- | --- | --- |
| inlining, before | 107 | 16 | 186 |
| inlining, now | 22 | 0 | 136 |
| inlining, LLVM `-O2` | 10 | 0 | 124 |
| recursion, before | 151 | 0 | 427 |
| recursion, now | 122 | 0 | 427 |
| recursion, LLVM `-O2` | 104 | 0 | 459 |
