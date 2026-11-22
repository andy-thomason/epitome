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

## What is in the way

- No MIR, and no lifetimes in types: `Ty::Ref` has no region.
- What helps: typeck records autoref, deref and reborrow adjustments per node;
  `lower/drop.rs` already works out moves, partial moves and temporary scopes.

## Milestones

| | What | Needs regions | Target |
|---|---|---|---|
| 0 | The harness and interface above | no | done |
| 1 | A control-flow graph of a body, for borrow checking only (places: local, field, deref, index, downcast; moves, copies, borrows, assignments, drops, calls, scope ends), built on demand from the typed tree, reusing the place and move classification of `lower/body.rs` | no | builds for all of `core` |
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
