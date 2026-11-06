# epitome-rustc2

A Rust compiler written in Rust that does not use rustc. It reads Rust
source, expands macros, resolves names and infers types, and will lower
the syntax tree straight to Epitome IR for Epitome's optimiser and code
generator to compile.

It is a clean-room compiler: there is no HIR or MIR. Every stage works
on the one syntax tree, and records what it learns in side tables indexed
by node. The tree can be printed at any point, so you can see what each
stage did. Language elements (types, signatures, lists and so on) are
interned as `u32` ids, and maps use a fast hasher instead of the
standard one.

It is a library first: a `Session` loads several crates at once and
works on their files in parallel. Each stage runs as jobs on a pool of
threads, and jobs can add more jobs. The `epitome-rustc2` program is a
thin driver over the library.

The plan is in [`docs/design.md`](docs/design.md).

## Progress

| Stage | State | On `core` + `alloc` + `std` (rust-src, 64 cores) |
|---|---|---|
| Lexer and parser | Done | All 2,994 `.rs` files of rust-src parse in about a second, nightly syntax included. So do the 69,108 files of the local crates.io cache, apart from 37 that are not valid Rust. |
| Pretty printer | Done | Files print back to source that parses to the same tree. |
| `cfg` | Done | Matches `rustc --print cfg`. |
| `macro_rules!`, built-in macros and derives | Done | 13,249 calls expanded in 5 rounds. Exactly the files rustc's dep-info lists are loaded, and the tests match `rustc -Zunpretty=expanded`. |
| Proc macros | Working | Run through the toolchain's proc-macro server; in-process loading will come once we can compile proc-macro crates. |
| Name resolution | Done | 350,676 paths in 0.1 s. Only 1,313 are unresolved, all of them in crates outside the session (libc and others). |
| Signatures and the type table | Done | 54,641 signatures and 13,886 impls in 0.1 s. The 159 with error types all name external crates. |
| Type checking bodies | Done (first version) | 32,847 bodies in 0.5 s, 97% typed completely. Most of the rest use libc. |
| Layout and constants | Done | Sizes and alignments match rustc's for the 45 types tested. |
| Monomorphisation | Done | From every non-generic function of `core`: 28,430 items in 0.09 s. 99.9% of uses resolve; most of the rest are specialisations. |
| Lowering to Epitome IR | Working | The programs in `tests/run/` lower completely. Of the 17,222 functions `core` reaches outside SIMD, 97% lower; the rest trap, and `--lower-stats` says why. |
| Compiling and running | Working | The nine programs in `tests/run/` compile through Epitome's optimiser and code generator, run, and print what rustc's builds print. |

Everything up to and including type checking `core`, `alloc` and `std`
together takes 1.3 s of wall time.

## Trying it

```sh
cargo build --release -p epitome-rustc2
B=target/release/epitome-rustc2
L=$(rustc --print sysroot)/lib/rustlib/src/rust/library   # needs rust-src

$B --sexp file.rs          # the syntax tree, as an s-expression
$B --dump-ast file.rs      # the syntax tree, indented
$B --pretty file.rs        # the tree printed back as Rust, after expansion
$B --dump-res file.rs      # what each path resolves to
$B --dump-sigs $L/core/src/lib.rs file.rs    # item signatures (with core)
$B --dump-types $L/core/src/lib.rs file.rs   # the type of every expression
$B --dump-layout $L/core/src/lib.rs file.rs  # each type's size, alignment and fields
$B --dump-mono $L/core/src/lib.rs file.rs    # every function instance main needs
$B --dump-ir $L/core/src/lib.rs file.rs      # the Epitome IR, with the source as comments
$B --lower-stats $L/core/src/lib.rs file.rs  # what could not be lowered, and why
$B -o prog $L/core/src/lib.rs prog.rs        # an executable, through Epitome (tested on x86-64 Linux)

# The statistics for each stage, on the standard library.
$B --edition 2024 --expand-stats --resolve-stats --types-stats --typeck-stats \
   $L/core/src/lib.rs $L/alloc/src/lib.rs $L/std/src/lib.rs

# Examples of why bodies fail to type, for any reason the stats print.
EPITOME_TYPECK_WHY="method not found" $B --typeck-stats ...
```

Each file given is a crate root, and the crates see each other by name.

## A tour

These are the stages run on this small crate:

```rust
#![no_std]

macro_rules! square {
    ($x:expr) => { $x * $x };
}

#[derive(Clone, PartialEq)]
pub struct Point<T> { pub x: T, pub y: T }

impl<T: Copy + core::ops::Add<Output = T>> Point<T> {
    pub fn sum(&self) -> T { self.x + self.y }
}

pub fn dist2(p: &Point<i64>) -> i64 { square!(p.x) + square!(p.y) }

pub fn evens(v: &[u32]) -> impl Iterator<Item = u32> + '_ {
    v.iter().copied().filter(|n| n % 2 == 0)
}
```

### The syntax tree

The tree is an arena of nodes, and each token is a slice of the source.
`--sexp` and `--dump-ast` print it. Here is `fn add(a: u32, b: u32) -> u32
{ a + b * 2 }`:

```
(Crate (Fn (Name add)
  (Params (Param (IdentPat (Name a)) (PathType (Path (Segment (Name u32)))))
          (Param (IdentPat (Name b)) (PathType (Path (Segment (Name u32))))))
  (RetType (PathType (Path (Segment (Name u32)))))
  (Block (Binary (PathExpr (Path (Segment (Name a)))) (Op +)
                 (Binary (PathExpr (Path (Segment (Name b)))) (Op *) (Literal 2))))))
```

### Expansion

`--pretty` prints the tree after expansion. Both `square!` calls are
expanded, and the built-in derives have generated their impls:

```rust
#[automatically_derived]
impl<T> ::core::clone::Clone for Point<T> where T: ::core::clone::Clone {
    #[inline]
    fn clone(&self) -> Self {
        Self { x: ::core::clone::Clone::clone(&self.x), y: ::core::clone::Clone::clone(&self.y) }
    }
}
#[automatically_derived]
impl<T> ::core::cmp::PartialEq for Point<T> where T: ::core::cmp::PartialEq {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        *&self.x == *&other.x && *&self.y == *&other.y
    }
}
...
pub fn dist2(p: &Point<i64>) -> i64 {
    p.x * p.x + p.y * p.y
}
```

### Names

`--dump-res` shows what each path resolves to, through imports, globs
and preludes, with macro hygiene taken into account:

```
/tmp/demo.rs:10:9 Copy -> Trait core::marker::Copy
/tmp/demo.rs:10:16 core::ops::Add<Output = T> -> Trait core::ops::arith::Add
/tmp/demo.rs:10:44 Point<T> -> Struct demo::Point
/tmp/demo.rs:11:30 self -> local #72
/tmp/demo.rs:14:24 i64 -> builtin i64
```

### Signatures

`--dump-sigs` prints every item's signature as interned types:

```
struct demo::Point<T> { x: T, y: T }
fn demo::{impl}::clone(&Point<T>) -> Point<T>
fn demo::{impl}::eq(&Point<T>, &Point<T>) -> bool
fn demo::{impl}::sum(&Point<T>) -> T
fn demo::dist2(&Point<i64>) -> i64
fn demo::evens(&[u32]) -> impl Iterator<Item = u32>
```

Bounds turn into predicates, and `Fn` sugar turns into the trait it
stands for. This is one from the tests:

```
fn t::f<F>(&F, P<u16, 2>) -> Option<&str>
    where <F as FnOnce<(u8,)>>::Output == bool, F: Fn<(u8,), Output = bool>, F: Copy
```

### Types

`--dump-types` prints every expression and pattern with its type, and
the method each method call picks. In `evens`, `n` is `&u32`. That comes
from `filter`'s `P: FnMut(&Self::Item) -> bool` bound, with `Self::Item`
worked out through core's `impl Iterator for Copied<I>`. `n % 2` uses
`impl Rem<u32> for &u32`, so the literals become `u32` as well:

```
fn demo::evens
  16:14    v : &[u32]
  17:5     v.iter() : Iter<u32>  [core::slice::{impl}::iter]
  17:5     v.iter().copied() : Copied<Iter<u32>>  [core::iter::traits::iterator::Iterator::copied]
  17:5     v.iter().copied().filter(|n| n % 2 == 0) : Filter<Copied<Iter<u32>>, {closure}(&u32) -> bool>  [core::iter::traits::iterator::Iterator::filter]
  17:30    |n| n % 2 == 0 : {closure}(&u32) -> bool
  17:31    n : &u32
  17:34    n % 2 == 0 : bool
  17:34    n % 2 : u32
  17:38    2 : u32
  17:43    0 : u32
```

Some more from [`tests/typeck/traits.rs`](tests/typeck/traits.rs), which
is checked against `core`:

```
// match on &Shape: patterns bind by reference
  22:14    |s| match s { Shape::Circle(r) => 3 * r * r, ... : {closure}(&Shape) -> u32
  23:27    r : &u32
  23:33    3 * r * r : u32

// `parse` and `?` from the type wanted
  42:17    a.parse() : Result<u8, ParseIntError>  [core::str::{impl}::parse]
  42:17    a.parse().ok()? : u8

// generic code: bounds on an associated type
  53:12    Default::default() : <I as Iterator>::Item
  53:39    a + b : <I as Iterator>::Item

// trait objects and tuple patterns through references
  58:5     w.write_str(s) : Result<(), Error>  [core::fmt::Write::write_str]
  63:29    (x, y) : (&u8, &u16)
  63:37    u16::from : fn item core::convert::From::from<u16, u8>
```

## How it is put together

| Module | What it does |
|---|---|
| `lexer` | Tokens are `&str` slices of the source. Line and column are worked out only when an error is reported. |
| `ast`, `parse` | The arena tree, with `mark`/`finish` building nodes. |
| `pretty` | Prints the tree as Rust, as an s-expression, or indented. |
| `session`, `pool`, `source` | Crates and files, the job pool, and the source map. |
| `cfg`, `expand`, `proc_macro` | `cfg` stripping, then expansion in rounds; proc macros run through the toolchain's proc-macro server. |
| `resolve` | Definitions, modules, imports resolved to a fixed point, and hygiene. |
| `ty` | Interned types; signatures, ADTs, impls, predicates and lang items. |
| `typeck` | Bodies in parallel: inference (`infer.rs`), expressions and patterns (`check.rs`), and traits (`traits.rs`). |
| `layout`, `consteval` | Types' sizes, alignments, fields and niches; constants worked out from the typed tree. |
| `mono` | Monomorphisation: every function instance, closure, vtable and constant a program needs. |
| `lower` | Bodies to Epitome IR: values and places (`body.rs`), patterns and enums (`pat.rs`), and intrinsics (`intrinsic.rs`). |
| `hash`, `intern` | A fast hasher, and sharded interners for ids. |

## Tests

```sh
cargo test --release -p epitome-rustc2
```

The tests cover the parser, expansion against `rustc -Zunpretty=expanded`,
the proc-macro server, signatures, type checking, layout against rustc,
monomorphisation, lowering the programs in `tests/run/`, and running them
against rustc's builds of the same programs (with `core` from rust-src, if
it is installed; running on x86-64 Linux).
