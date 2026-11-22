# Borrow checking: work plan

The design (`design.md` section 1) lists borrow checking as a non-goal: the
compiler compiles programs rustc has accepted. This is the plan for adding it
anyway, as a check that finds what rustc finds and never more.

## The rule

**It must never reject what rustc accepts.** A missed error costs nothing a
program can see; a wrong one stops a program that builds. So every milestone
ships behind `--borrowck` until these hold:

- every `check-pass` UI test, and every program that builds and runs, has no
  diagnostics (`check_pass_tests_are_accepted` fails a test that has one);
- `core`, `alloc` and `std` have none (`borrowck_accepts_the_standard_library`).

## What there is

- `src/borrowck/mod.rs`: `Session::borrowck` and `Diagnostic` (a stub that
  finds nothing), and `CODES`, the error codes that are this checker's.
- `epitome-rustc --borrowck file.rs`: prints `file:line:col: error[E0382]:
  message`, and fails if there are any.
- `borrowck_expected_errors_are_found` (`tests/ui_check_pass.rs`): the UI tests
  that expect borrow-checking errors only, each of which must get those errors
  (code and line, from its `.stderr`) and no others. `tests/ui_borrowck_todo.txt`
  lists those that do not yet, and why (`no diagnostics (missing E0382@12)`,
  `wrong diagnostics (missing ..; extra ..)`, `before borrowck: <earlier stage>`).
  `EPITOME_UI_BLESS=1` rewrites it; a test that passes must come off it.

```sh
export EPITOME_RUST=~/rust-lang        # a checkout with tests/ui and library
cargo test --release -p epitome-rustc --test ui_check_pass borrowck_expected -- --nocapture
EPITOME_UI_FILTER=/borrowck/ ...       # only some tests
```

Starting point: 1,097 tests (1,078 get as far as the checker).

## The control-flow graph (milestone 1)

`src/borrowck/cfg.rs` is the graph, `src/borrowck/build.rs` builds it from the
typed tree: a body is basic blocks of statements (`Assign`, `StorageLive`,
`StorageDead`, `Drop`) ending in a terminator (`Goto`, `Switch`, `Call`,
`Return`, `Unreachable`); a place is a local and a path of projections
(`Deref`, `Field`, `Downcast`, `Index`, `ConstIndex`, `Subslice`). Typeck's
adjustments (autoderef, autoref, unsizing) are applied to the places;
patterns are tested, then bind; a block's variables and a statement's
temporaries leave scope with a `StorageDead` (and a `Drop`, unless the type is
plainly trivial) on every way out; a `let` that borrows a temporary keeps it
for the block; closures get bodies of their own, their captures fields of
the environment.

```sh
epitome-rustc --dump-cfg  core.rs alloc.rs std.rs prog.rs   # the last crate's bodies, as Rust
epitome-rustc --dump-cfg --cfg-compilable ...               # locals initialised, so rustc takes it
epitome-rustc --cfg-stats core.rs alloc.rs std.rs prog.rs   # how many built, and why not
```

A body shows as a Rust function, a state machine with one `match` arm for each
block (`bb = ..;` goes on, `return _0;` ends). It is Rust that `rustc` takes
for numbers and references (`borrowck::tests::rendered_bodies_compile`);
downcasts (`(_3 as variant#1).0`) are the one thing in it that is not Rust.

What it leaves for the analyses, on purpose:

- **Copy or move**: an operand is a `Use` of a place. Which it is needs the
  body's trait environment, so the move analysis (milestone 2) says.
- **Drops**: every variable and temporary that is not plainly trivial is
  dropped on leaving scope; whether its type needs it is for the analyses.
- **Unwinding**: a call has one successor.
- **Closure captures by reference** are shared borrows: typeck does not record
  which are changed through.
- **`asm!`**: its operands are not followed. A static is a constant, not a
  place.

Coverage: 26,306 of 26,324 bodies of `core` (99.93%), 33,426 of 33,943 of
`core`, `alloc` and `std`; all are valid graphs (`Cfg::validate`). Of those not
built, most have an error type from type checking (the external `libc` crate,
chiefly), which `--cfg-stats` lists with an example of each.

## What is in the way

- No MIR, and no lifetimes in types: `Ty::Ref` has no region.
- What helps: typeck records autoref, deref and reborrow adjustments per node;
  `lower/drop.rs` already works out moves, partial moves and temporary scopes.

## Milestones

| | What | Needs regions | Target |
|---|---|---|---|
| 0 | The harness and interface above | no | done |
| 1 | A control-flow graph of a body, for borrow checking only (places: local, field, deref, index, downcast; uses, borrows, assignments, drops, calls, scope ends), built on demand from the typed tree | no | **done**: 99.9% of `core` (see below) |
| 2 | Moves and initialisation: E0381 E0382 E0384 E0505 E0507 E0509, closure captures; maybe-uninit and moved dataflow over place paths | no | most of the ~360 tests |
| 3 | Mutability: E0596 E0594 E0384, from binding modes, `&` against `&mut`, closures, `ref` patterns | no | most of the ~210 tests |
| 4 | Regions in signatures: keep what `ty/lower` drops (lifetime parameters, `'a: 'b`, `T: 'a`, elision, `'static`, `dyn Trait + 'a` defaults, higher-ranked bounds); variance of ADT parameters; region-annotated types for borrowck only, leaving the shared `Ty` alone | yes | signatures and variance match rustc |
| 5 | Loans (NLL): region variables per body, subtyping constraints with variance, liveness, outlives solving, loan scopes and kills, two-phase borrows, reborrows, closure captures: E0499 E0502 E0503 E0505 E0506 E0597 E0515 E0716 E0521 | yes | most of the borrowck and nll tests |
| 6 | Associated types and `impl Trait` with regions, implied bounds, the higher-ranked leak check, unsafe and raw-pointer boundaries | yes | the rest |

Milestones 2 and 3 need no regions and cover most of the E0381, E0382, E0384,
E0507, E0594 and E0596 tests: a useful checker on its own. Milestones 4 and 5 are
a project of their own. The leak check of milestone 6 is what the
`leak-check-in-selection-*` UI tests need (they fail type checking today for want
of regions).

## Risks

False positives from macro spans and hygiene, two-phase borrows, closure capture
precision, and async and generator bodies; typeck's coercion and reborrow
adjustments are the main input to milestone 5, so any imprecision there becomes a
wrong error. Each body is an independent job, as in type checking; dataflow stays
linear in the body, and region solving gets a budget.
