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
epitome-rustc --dump-cfg      core.rs alloc.rs std.rs prog.rs   # the source, annotated with the graph
epitome-rustc --dump-cfg-rust core.rs alloc.rs std.rs prog.rs   # the graph, as a Rust function
epitome-rustc --cfg-stats     core.rs alloc.rs std.rs prog.rs   # how many built, and why not
epitome-rustc --borrowck      core.rs alloc.rs std.rs prog.rs   # the errors, of the last crate
epitome-rustc --borrowck-all  core.rs alloc.rs std.rs prog.rs   # of every crate (a gate on the library)
```

`--dump-cfg` prints each body's source as it is (as the AST has it: after
expansion, so what a macro made is listed after the function) and, under each
line, what the graph does for it: the blocks (`bbN:`), their statements and where
they go. A scope's ends (`StorageDead`, `Drop`, `return`) are on the line that
closes it.

```text
// cfg_small::abs: _0: i32 (result), _1: i32 (x)
pub fn abs(x: i32) -> i32 {
    if x < 0 { -x } else { x }
        //  bb0: _2 = _1 < 0
        //       _3 = _2
        //       if _3: bb3 else bb1
        //  bb1: _0 = _1
        //       goto bb2
        //  bb3: _0 = -_1
        //       goto bb2
}
    //  bb2: return
```

`--dump-cfg-rust` is the other view: a body as a Rust function, a state machine
with one `match` arm for each block (`bb = ..;` goes on, `return _0;` ends).
With `--cfg-compilable` its locals are initialised, and it is Rust that `rustc`
takes for numbers and references (`borrowck::tests::rendered_bodies_compile`);
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

## Moves and initialisation (milestone 2)

`src/borrowck/moves.rs`. Each place the body names has a *move path* (a
variable, a field of one, the contents of a `Box`). Three sets of paths are
followed through the graph, each "may" (on any way here): not initialised,
moved, initialised. A use of a path asks about it, what it is part of and what is
part of it. The errors:

| | |
|---|---|
| E0381 | used before it has a value |
| E0382 | used (or borrowed, or assigned a part of) after it moved, as a whole or in part |
| E0384 | a variable that is not `mut` assigned when it may have a value |
| E0507 | moved out of a borrow |
| E0508 | moved out of an array or slice (by an index, or behind a reference) |
| E0509 | a field moved out of a type with a destructor |

Like rustc, an uninitialised or moved variable is reported once, at its first
use in the file; a use in a macro is at the macro's call.

**Never wrong, sometimes silent.** Where it is not sure, it says nothing:

- `may_be_copy`: a type is taken as `Copy` unless it is known not to be.
- A `&mut T` (and a `Pin<&mut T>`, and a type that is `Reborrow`) used by value
  is a reborrow, as it is where rustc passes it on; typeck does not record
  every one. So a `&mut` moved twice is not found.
- A closure captures whole variables (typeck's capture analysis), as edition
  2018 did: one that only uses fields, or is a `use ||`, does not move the rest,
  and a move out of what a closure has by reference is not its error.
- A call that cannot return (`-> !`, or an uninhabited type) ends the path.

**Measured**: `core`, `alloc` and `std` and all 4,300 `check-pass` UI tests get no
errors (`borrowck_accepts_the_standard_library`, `check_pass_tests_are_accepted`);
of the 345 UI tests that expect only the errors above, about 230 get exactly
rustc's (code and line). The rest are mostly: moves out of arrays by pattern or
index (paths do not go into elements), a move out of a `static`, `FnOnce`
closures moved twice, and a `&mut` moved.

`tests/borrowck/moves.rs` has the cases, annotated with `//~ E0382` and the like.

## Mutability (milestone 3)

`src/borrowck/mutability.rs`. A place can change by rustc's rule: a variable if
it is `mut` (a temporary always is), a field or element if what it is part of
is, `*r` of a `&mut T` however `r` is declared (but not if `r` is itself behind a
`&`), never `*r` of a `&T` or `*const T`, always of a `*mut T`, a `Box`'s contents
if the box is. Then:

| | |
|---|---|
| E0596 | `&mut p` (or `&raw mut p`, or an autoref'd method receiver, or a `ref mut` binding) of a place that cannot change |
| E0594 | `p.f = v`, `*r = v`, `a[i] = v`, `p.f += v` where it cannot |

(A whole variable assigned twice is E0384, in `moves`.) What decides is how a
place is *used*, which typeck does not say: it records `Deref::deref` and
`Index::index` and leaves `DerefMut` and `IndexMut` to the context. So the
builder knows when a place is to be changed (assigned to, `&mut`, a compound
assignment, a `ref mut` pattern, a callee called through `FnMut`) and picks the
`_mut` method then, where the type has one: `*rc = 1` is an error because `Rc`
has no `DerefMut`.

- **Closures.** A closure's captured variable is the place in its environment
  (the field, and the `*` after it if held by reference); whether it can change
  is whether the variable it came from was declared `mut`. A closure whose body
  changes what it captured is `FnMut`, and calling one needs its variable to be
  mutable.
- **Statics** are places of their own: not `static mut`, they cannot change
  (and cannot be moved out of: E0507).
- **Parameters**: one that is not just a name (`(ref mut a, b): (A, B)`) is a
  `mut` local, as in rustc.

`tests/borrowck/mutability.rs` has the cases.

## Regions in signatures (milestone 4)

`src/regions/`. `Ty` still has no lifetimes, and that does not change: the
shared type table is for type checking and code generation, which never need
them. Lifetimes are read again from the source where they are written, and
kept beside `Ty`, for borrow checking only.

- **Variance** (`variance.rs`): `Session::variances`, rustc's algorithm. Each
  parameter of a struct, enum, union, alias or free function starts bivariant;
  each use in a field adds "at least this", composed with the variance of the
  parameter of the type it is an argument of (so the constraints are solved
  together, to a fixed point). `&mut T`, `*mut T`, trait arguments, projections
  and array lengths are invariant, a function pointer's arguments flip.
  `PhantomData` is covariant and `UnsafeCell` invariant by lang item. Bounds add
  nothing. `Session::dump_variances` prints what `#[rustc_dump_variances]` does.
- **Signatures** (`sig.rs`): `fn_regions(def)` gives a function's lifetime
  parameters (early or late bound, and those of its impl or trait), its
  parameter and return types as `RTy` (a `Region` in every `&`, lifetime
  argument and `dyn Trait + 'a`), `'a: 'b` and `T: 'a`, and trait bounds with
  their arguments (`F: for<'x> Fn(&'x T) -> &'x U`). `adt_fields(def)` gives
  the fields of a type likewise.
- **Elision**: each elided lifetime in an argument is its own (`Elided(i)`,
  hidden lifetime arguments of paths too); the result's is `&self`'s, or the
  only one in the arguments. A function pointer or `Fn(A) -> B` binds its own.
  An elided lifetime in an impl header (`impl Tr for &T`) is a parameter of
  the impl (`ImplAnon`). Type aliases are expanded.
- **Not yet**: `impl Trait` and associated types with regions (milestone 6);
  `Region::Unknown` is left for what has nothing to be.

Measured: the 12 variance UI tests that do not involve `impl Trait` print
exactly what rustc does (`EPITOME_UI_WANT=variance`,
`tests/ui_variance_todo.txt`; the rest print the variances of opaque types);
every function of `core` has its signature read, none with a lifetime left
out; and so has every function of the 4,300 check-pass programs.

## Loans (milestone 5)

`src/borrowck/loans.rs`. `Ty` has no regions, so a value's regions are kept as
*slots*: one for each place a lifetime can be in its type (`&'a Vec<&'b T>` has two;
a struct has one for each lifetime parameter, then those of its type arguments).
Each local has, for each slot, the set of loans its region may hold: a flow-sensitive
stand-in for rustc's region inference, computed by dataflow over the graph.

- **Loans.** `r = &place` is a loan; `r`'s outer slot holds it and the rest hold what
  `place` holds. A reborrow `&*q` also holds what `q`'s outer slot holds, back to the
  first shared reference. A closure capture is a loan of each part of the variable the
  closure uses (a field path, `(*p).a`), mutable if the closure changes it.
- **Flow.** Copies and aggregates move sets between slots (a field's slots are found
  from `regions::sig::adt_fields`). A call's result gets what the signature ties to it:
  an input slot and an output slot with the same lifetime or type parameter. A `&mut`
  argument gets what other arguments tie to what it points to (`v.push(&x)`), and so
  do the places it borrows. What a signature cannot say (an associated type, an
  `impl Trait`) takes the slots the type has; `Fn(A) -> B` bounds tie nothing.
- **Live.** A loan is live where a local that holds it is live (used later; the
  arguments and the result are used at the return). An assignment to a variable ends
  the loans of what it pointed to; so does the end of its scope.
- **Conflicts.** An access to a place (read, write, move, `&`, `&mut`, a `&mut`
  argument reborrowed, the end of the variable's scope) conflicts with a live loan of
  an overlapping place: E0499, E0502, E0503, E0505, E0506, and E0597 (E0716 for a
  temporary, E0515 if what is returned holds it, E0373 if a closure's capture). A
  two-phase borrow (an autoref receiver) only reserves, as a shared borrow, until its
  call. Slice-pattern elements and fields are disjoint where rustc's are. A call that
  wants `'static` (`&'static T`, `F: 'static`) of an argument that holds a borrow of the
  function's own data is E0597 or E0373.
- **Left out so that nothing is wrongly rejected:** what a constant or static borrows
  (promoted to `'static`; so is `&[]`, `&0`, `&Foo { x: 1 }`); borrows behind a shared
  reference; the locals macros declare; what a closure returns of its arguments.

The gate is the same: nothing in `core`, `alloc` and `std`, and in the check-pass
programs. The rest, in the order they appear in the tests that fail: drops as uses
(E0597 and E0505 for types with `Drop`), annotated lifetimes in bodies (`let x: &'static
T`), universal regions (E0521, E0621), closures called through `FnMut` and what they
return (E0373 from `move` closures), borrows that outlive a `match` (E0510, E0713).

`tests/borrowck/loans.rs` has the cases.

## What is in the way

- No MIR, and no lifetimes in `Ty::Ref`: see above for where they are kept.
- What helps: typeck records autoref, deref and reborrow adjustments per node;
  `lower/drop.rs` already works out moves, partial moves and temporary scopes.

## Milestones

| | What | Needs regions | Target |
|---|---|---|---|
| 0 | The harness and interface above | no | done |
| 1 | A control-flow graph of a body, for borrow checking only (places: local, field, deref, index, downcast; uses, borrows, assignments, drops, calls, scope ends), built on demand from the typed tree | no | **done**: 99.9% of `core` (see below) |
| 2 | Moves and initialisation: E0381 E0382 E0384 E0507 E0508 E0509, closure captures; maybe-uninit and moved dataflow over place paths | no | **done**: 67% of the 345 tests that expect only these (see below) |
| 3 | Mutability: E0596 E0594 E0384, from binding modes, `&` against `&mut`, closures, `ref` patterns | no | **done**: 74% of the 156 tests that expect only mutability errors |
| 4 | Regions in signatures: keep what `ty/lower` drops (lifetime parameters, `'a: 'b`, `T: 'a`, elision, `'static`, `dyn Trait + 'a` defaults, higher-ranked bounds); variance of ADT parameters; region-annotated types for borrowck only, leaving the shared `Ty` alone | yes | **done**: variance matches rustc's on its tests; `core` and the check-pass programs read |
| 5 | Loans (NLL): region variables per body, subtyping constraints with variance, liveness, outlives solving, loan scopes and kills, two-phase borrows, reborrows, closure captures: E0499 E0502 E0503 E0505 E0506 E0597 E0515 E0716 E0521 | yes | **started**: loans, flow, conflicts, closure captures, `'static` calls (see below) |
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
