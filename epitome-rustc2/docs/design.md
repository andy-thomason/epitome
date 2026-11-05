# epitome-rustc2

This is the plan for `epitome-rustc2`, a Rust compiler written in Rust
that does not use rustc. It reads Rust source, expands macros, resolves
names, infers types and lowers the syntax tree straight to Epitome IR,
which Epitome's optimiser and code generator compile.

`epitome-rustc` (`src/bin/epitome-rustc.rs`) is a different program: it
wraps the real rustc and replaces only LLVM's code generation. It stays
useful as a reference. The same crate built by both must behave the
same.

**Status:** The front end works up to name resolution, and type
checking is under way.

- **Parsing.** The lexer and the parser (sections 3 and 4) are written.
  Every `.rs` file of the toolchain's `rust-src` (2994 files, nightly
  syntax included) parses in about a second, and so does every file of
  the local crates.io cache (69108 files) except 37 that are not Rust
  that rustc would accept.
- **`cfg` and `macro_rules!` expansion** (section 6) are done.
  - `core`, `alloc` and `std` together expand about 12,000 macro calls,
    in 0.16 s on mandelbrot, in four rounds.
  - Afterwards they consist of exactly the files rustc reads for them,
    according to rustc's dep-info.
  - The test programs in `tests/expand/` expand to the same tokens as
    `rustc -Zunpretty=expanded`.

- **Name resolution** (section 5) is done: on `core`, `alloc` and `std`,
  99.6% of paths resolve, or resolve as far as a type, in 0.1 s. Macros
  are found through the same module tree and imports.
- **Types** (section 7).
  - Signatures and the type table (7.1, 7.2) are done: 54,523 signatures
    of `core`, `alloc` and `std` in 0.07 s. The 159 that have an error
    type all name crates outside the session.
  - Bodies (7.3) are checked in parallel, with inference variables,
    expected types, integer fallback, inherent methods and patterns'
    binding modes. The 32,797 bodies of `core`, `alloc` and `std` take
    0.2 s, and 70% of them type completely. Almost all of the rest need
    trait methods, operator traits or `Deref`, which come next.
    `--dump-types` prints every expression's type.

Everything after that is a plan.

## 1. Goals

- **Compile real crates.** `core`, `alloc` and `std` from the toolchain's
  `rust-src`, then crates from crates.io, built by Cargo with
  `epitome-rustc2` as `RUSTC`.
- **Run existing procedural macros.** Proc-macro crates (`serde_derive`,
  `thiserror`, ...) are built by the real rustc, as Cargo builds them for
  the host, and run unchanged (section 6.6).
- **No HIR or MIR.** The AST is the only tree. Later stages add facts
  about AST nodes in side tables indexed by `NodeId`: what a name refers
  to, an expression's type, which method a call picks. Lowering walks the
  AST with these tables and writes Epitome IR, as a C compiler's front
  end walks its AST.
- **Every stage can be printed.** The tree after parsing, after
  expansion, with resolutions, with types, and the IR each function
  becomes, so each step of lowering can be checked by eye against rustc's
  `-Zunpretty` output (section 12).
- **A library first.** The compiler is a crate with an API (section
  11); the `epitome-rustc2` program is a thin command line over it. A
  host can compile several crates in one process.
- **Parallel everywhere** (section 10): files, modules, items, function
  bodies and instances are jobs on one pool of threads, across all the
  crates of a run, with the same output for any number of threads.
- **Stay linear in the size of the input.**
- **Clean room** (section 13).

Not goals for now: borrow checking (a program rustc rejects for
lifetimes or borrows may compile here; we compile programs rustc has
accepted), incremental compilation, and diagnostics as good as rustc's.
Errors point at the line and column and say what was expected; that is
all.

## 2. Where it fits

```text
  .rs files (crate root + modules)
        │
        ▼
 ┌────────────────────────────── epitome-rustc2 ───────────────────────────┐
 │ source map ─► lex ─► parse ─► AST                                       │
 │                                 │                                       │
 │      expand: cfg, macro_rules!, built-in macros, derives,               │
 │              proc macros (rust-analyzer-proc-macro-srv) ─► more AST     │
 │                                 │                                       │
 │      resolve: modules, `use`, paths      ─► side table: NodeId → Def    │
 │      collect: items, impls, traits       ─► definitions                 │
 │      type check: inference, methods,     ─► side tables: NodeId → Ty,   │
 │                  traits, coercions          method, adjustments         │
 │                                 │                                       │
 │      lower: monomorphise on demand; AST + tables ─► epitome_ir::Module  │
 │      metadata: what a dependent crate needs (.rmeta, our own format)    │
 └─────────────────────────────────┬───────────────────────────────────────┘
                                   ▼
                   epitome-opt ─► epitome-codegen ─► .o / .rlib / binary
```

The crate depends on `epitome-ir` only. The binary links the optimiser
and code generator, as `epitome-sv` does.

## 3. Source text and tokens

### 3.1 Tokens are slices

As in the IR, MLIR and SystemVerilog lexers, a token is a slice of the
source:

```rust
pub struct Token<'a>(pub &'a str);
```

Its kind is worked out from its text when needed (`is_ident`,
`is_lifetime`, `is_literal`, ...), and its position from where the slice
sits: offset = token pointer − source pointer; line and column only when
an error is reported. The end-of-file token is the empty slice at the end
of the text.

All text lives in a `SourceMap` (as in `epitome-sv`): every file read,
and every piece of text made later (proc-macro output, `stringify!`,
`concat!`), at an address that never changes. A slice of any of it lives
as long as the map, and `SourceMap::find` says which source it is in.
The map is shared by all threads (adding a source takes a lock; reading
one does not).

### 3.2 What the lexer does

- White space and comments are skipped; block comments nest.
- Doc comments (`///`, `//!`, `/** */`, `/*! */`) are tokens, so the
  parser can turn them into `#[doc]` attributes.
- Identifiers, raw identifiers (`r#match`), lifetimes and labels (`'a`),
  and character literals (`'a'`, `'\n'`, `'\u{1F600}'`) are told apart
  by looking past the quote.
- Literals keep their suffix in the same token: `1u8`, `2.5f32`,
  `"x"suffix`. `1.foo()` and `1..2` lex as `1` `.` and `1` `..`; `1.` is
  a float only when the next character cannot start a name or a `.`.
- Strings: `"..."`, `b"..."`, `c"..."`, and raw `r#"..."#`, `br##"..."##`,
  `cr"..."` with any number of `#`.
- Punctuation is one character per token, as `proc_macro` sees it:
  `::` is two `:` tokens that touch. The parser joins touching tokens
  into the operator it wants (`at_op("::")`, longest first from a table),
  so nothing ever needs splitting: `Vec<Vec<u8>>` closes two generic
  lists with two `>` tokens, `&&x` is two borrows, and `||` is an empty
  closure parameter list or the operator depending on where the parser
  is. Whether two tokens touch is known from their addresses, which is
  also what a proc macro's `Spacing::Joint` means.
- After a lone `.` a number is read without a fraction, so `x.0.1` is
  two field accesses, not a field and a float.
- A `#!` at the very start that is not `#![` is a shebang line and is
  skipped.

Tokens are made on demand from a byte offset, so the parser (or the
macro expander) can lex from anywhere.

### 3.3 Token trees

Macro calls, attribute arguments and `macro_rules!` bodies are kept as
token trees: delimited groups whose contents are not parsed until a
macro needs them. A `TokenTree` node in the AST covers the tokens of the
group. Nothing is re-lexed: matchers and expansions work on the same
token list.

## 4. The syntax tree

### 4.1 Nodes are slices of the token list

As tokens are slices of the source, nodes are slices of the token list:

```rust
pub struct Node {
    pub kind: Kind,        // what it is
    pub first: u32,        // tokens covered: Ast::tokens[first..end]
    pub end: u32,
    child_start: u32,      // children: Ast::children[child_start..][..child_len]
    child_len: u32,
}
```

All nodes live in one arena, `Ast::nodes`, and refer to each other by
`NodeId`. Children are the parts that carry meaning (names, literals,
operators, the keywords that choose a form such as `mut` or `unsafe`, and
sub-constructs); punctuation is only in the token range. Each `Kind`'s
comment lists its children. This is the shape `epitome-sv` uses; it is
small, needs no allocation per node, and is easy to print.

### 4.2 What is in it

The tree follows the Rust Reference's grammar:

- **Items:** `fn`, `struct`, `enum`, `union`, `trait`, `impl`, `mod`,
  `use`, `const`, `static`, `type`, `extern crate`, `extern` blocks,
  `macro_rules!`, macro calls in item position; with attributes,
  visibility, generics and `where` clauses.
- **Types:** paths with generic arguments, references, raw pointers,
  arrays and slices, tuples, `fn` pointers, `impl Trait`, `dyn Trait`,
  `!`, `_`, macro calls.
- **Patterns:** identifiers with `ref`/`mut`/`@`, literals, ranges,
  tuples, structs, tuple structs, slices with `..`, references, or-
  patterns, paths, macro calls.
- **Expressions:** with precedence climbing; and the restriction that a
  struct literal cannot appear where `{` would start a block (`if`,
  `while`, `match` scrutinees, `for` iterators); blocks, `unsafe`,
  `async`, `const` blocks, closures, labels, `let` chains in `if` and
  `while`.
- **Statements:** `let` (with `else`), items, expression statements, and
  the rule that a block-like expression ends a statement without `;`.

The parser is hand-written recursive descent, building with a node stack
(`mark` / `finish(mark, kind)`), as `sv/src/parse` does. The first error
stops the parse. Where a fixed lookahead is not enough (a parameter that
is `pattern: Type` or only a `Type`; a generic argument that is `Item =
T` or a type) it tries one reading and backs up.

`epitome-rustc2 --parse-tree dir` parses every `.rs` file under a
directory and lists the failures: the parse corpus of section 12.

### 4.3 Printing

- `--dump-ast`: one node per line, indented, `Kind` or `Kind text`
  (done).
- `--sexp` / `Ast::sexp`: the same on one line, for tests (done):
  `(Binary (PathExpr (Path (Segment (Name a)))) (Op +) (Literal 1))`.
- `--pretty` (done, `src/pretty.rs`): the tree printed back as Rust
  source in a fixed style: four-space indents, one statement or field
  per line, spaces around binary operators. Parentheses are nodes, so
  the printer never adds any. Macro input (token trees) is printed as
  the source text it covers, comments and all. `--parse-tree dir
  --roundtrip` prints every file, parses the output and checks the tree
  is the same, node for node and token for token. All of `rust-src`
  and of the crates.io cache that parses round-trips.

## 5. Name resolution

**Done** for items, imports and paths (`src/resolve/`).
`epitome-rustc2 --dump-res` prints every path with what it resolved to,
and `--resolve-stats` counts them.

### 5.1 Definitions and modules

`collect.rs` makes a `Def` for every item, and a module for the crate
root, each `mod`, each `enum` (its variants), each `trait` (its items)
and each block that holds items. A module has names in three namespaces,
types, values and macros, each with a visibility: public, or seen only
inside a module (`pub(crate)`, `pub(super)`, private). A tuple or unit
struct or variant is in both the type and value namespaces.
`#[macro_export]` macros are also at the crate root.

### 5.2 Imports

`use` trees are flattened into single and glob imports, and resolved in
rounds until a round resolves nothing new (`imports.rs`).

- **Path starts.** A path starts at `crate`, `self`, `super`, `::crate`,
  a name in the module, or a crate in the extern prelude. The extern
  prelude is the crates of the session plus any `extern crate` at the
  crate root, renames included. 2015 crates start at the crate root.
- **Globs** are edges to other modules, followed when a name is looked
  up, so names a module defines or imports by name come first.
- **The prelude** is what `#[prelude_import]` names, else
  `std::prelude::rust_20xx` (or `core`'s with `no_std`).

### 5.3 Paths

`paths.rs` walks every file in parallel. It keeps scopes for generic
parameters and `Self`, function and closure parameters, `let` bindings,
match arms, `if let` / `while let` (and `let` chains) and `for`
patterns. An item inside a function sees none of the function's locals.

- **One name** looks in the scopes, then the items of the enclosing
  blocks and the module, the extern prelude, the prelude, and the
  built-in types. A generic argument read as a type may be a constant.
- **A longer path** follows modules, enums and traits. When it reaches a
  type (`Vec::new`, `T::Item`, `u32::MAX`, `Ordering::is_lt`), it stops
  with `Res::Partial`, leaving the rest for type checking. A primitive
  name shadowed by a module (`char` after `use core::char`) falls back to
  the primitive, as rustc does.
- **An identifier pattern** that names a unit struct, unit variant or
  constant in scope is that, not a new binding.
- **Results** go in a side table from the path's node to its `Res`; the
  tree does not change.

### 5.4 Hygiene

- **Items.** A path written in a macro 2.0's body resolves in the
  macro's module first (definition-site), then at the call site, for the
  items the expansion itself defines. `macro_rules!` expansions resolve
  items at the call site, as rustc's mixed-site hygiene does.
- **Local variables.** Every token has a syntax context: the macro body
  (`macro_rules!` or macro 2.0, as written in a file) whose text it is
  in, or none. The spans of the bodies are known, so a token's address
  gives its context. A local binding records its context, and a name
  sees only bindings of its own context. A name from a macro body falls
  back to bindings outside any macro (an approximation of the locals at
  the macro's definition). So `m!(x)` with `let x = 2; $e + x` in `m`
  sees the caller's `x` for `$e` and the macro's for its own `x`.
  Generic parameters are not hygienic.
- **Visibility.** `pub(in path)` names an ancestor module, which already
  exists when the item is collected, so it is resolved then.
- **Maps.** Side tables keyed by node are `Vec`s indexed by `NodeId`; the
  few hash maps use our own fast hasher (`src/hash.rs`), not the standard
  library's default.

### 5.5 Results on `core`, `alloc` and `std`

On mandelbrot, `core`, `alloc` and `std` resolve in 0.1 s: 54,278
definitions, 14,733 modules and 6,177 imports.

| Paths (336,574 in all) | Share |
|---|---|
| Resolved | 94.3% |
| Partial (finished in type checking) | 5.3% |
| Unresolved | 0.4% |

The 128 imports that fail and nearly all of the unresolved paths name
crates outside the session (`libc`, `hashbrown`, `std_detect`,
`unwind`, `cfg_if`).

## 6. Macro expansion

**Done** for `cfg`, `macro_rules!`, macros 2.0 and the built-ins listed
below (`src/cfg.rs`, `src/expand/`).

### 6.1 Changing the tree

An expansion is a new list of tokens: slices of the definition's text
with the call's arguments put in, so most expanded tokens still point
into real files. They are appended to `Ast::tokens`, with the position
of the call they came from (`Ast::origin_of`), and parsed as the
fragment the call's place needs (items, statements, an expression, a
type or a pattern). The new nodes are grafted into the file's arena
(`Ast::graft`) and the call's parent is given a new child list
(`Ast::set_children`). Nodes never change otherwise, so a `NodeId` stays
valid through expansion, and the arena is append-only.

An expansion in an expression's place is kept one expression, as rustc
keeps it one node: if it is not a single operand, it goes in a
parenthesis node of its own. The printer shows such parentheses only
where precedence needs them, as rustc's does.

### 6.2 `cfg`

`CfgSet::host()` is the configuration rustc uses for the machine we run
on, including what it prints only with `-Zunstable-options`; a test
compares it with `rustc --print cfg`. Stripping (`cfg::strip`) runs on
each file as it is parsed, before its `mod` items are followed, and again
on everything expansion makes. `cfg_attr` becomes the attributes it
gives.

### 6.3 `macro_rules!`

A definition's rules are compiled into matchers and transcribers
(`expand/mbe.rs`).

- **Matching** tries each rule in order and backtracks over repetition
  counts. Fragments (`expr`, `ty`, `pat`, `pat_param`, `path`, `block`,
  `stmt`, `item`, `meta`, `vis`, by edition) are read by the ordinary
  parser from where the matcher is (`FragmentParser`, one per delimited
  group of the input). `ident`, `lifetime`, `literal` and `tt` are read
  directly. Doc comments in the input become `#[doc = "..."]` first, as
  macros see them.
- **Transcription** copies tokens and handles `$(...) sep op` repetition
  by the depth of the metavariables in it. It also handles `$crate`
  (`crate` in the defining crate, `::name` elsewhere) and
  `${count()}`, `${index()}`, `${ignore()}` and `${concat()}`.
  - A transcribed `$e:expr` that is not a single operand is put in
    parentheses made of synthetic tokens (not slices of any source), so
    `$e * 2` keeps `$e` whole.
- **Macros 2.0** (`macro m(...) { ... }`, `macro m { ..., ... }`) are the
  same rules.

### 6.4 Rounds and scope

Expansion runs in rounds, each in parallel over every file of every
crate:

1. Collect the definitions.
2. Expand every call whose macro is found, and the calls in its output,
   up to the recursion limit of 128.
3. Load the files of any new `mod` items.

Rounds repeat until nothing changes. This handles macros that define
macros, and modules declared inside `cfg_select!`.

A definition's position is its file's position in the crate's text
(`ParsedFile::pos`, built from each `mod` item's place in its parent),
then its own. A call `m!` finds, in order:

- the last `macro_rules! m` before it in its module or an enclosing one
  (`#[macro_use]` modules pass their macros to their parent);
- a macro exported by a crate in its macro prelude (`#[macro_use] extern
  crate`, and `std` or `core`);
- a built-in.

Paths and macros brought in by `use` are found by name resolution's
module tree and imports (section 5), rebuilt at the start of each round,
so an import made by one round's expansions is seen by the next. A
macro 2.0's body resolves where the macro is defined (5.4). Only proc
macros of crates outside the session are still found by name.

### 6.5 Built-in macros and attributes

Done in expansion: `cfg_select!`, `cfg!`, `include!`, `stringify!`,
`concat!`, `line!`, `column!`, `file!`, `module_path!`, `env!`,
`option_env!`, `compile_error!`. A definition with
`#[rustc_builtin_macro]` dispatches to these by name.

Left as calls for type checking and lowering: `format_args!` (to a node
the type checker knows), `asm!`, `global_asm!`, `include_str!`,
`include_bytes!`, `offset_of!`, `assert!` and the like.

**Built-in derives** (`src/expand/derive.rs`, done): `Clone`, `Copy`,
`Debug`, `Default`, `PartialEq`, `Eq`, `PartialOrd`, `Ord` and `Hash`.

- **What each gives.** Each reads the item's shape (name, generic
  parameters, `where` clause, fields or variants) and writes an `impl` as
  source text, which is parsed like any macro's output. The impls do
  what each trait's documentation says a derive does: fields in order,
  and variants compared by their order, then their fields.
- **Generics.** The impl keeps the item's generic parameters, without
  defaults, and bounds each type parameter by the trait.
- **Paths** start at `::core`, which every crate can name (`core` itself
  through `extern crate self as core`).
- **How they are found.** A derive is found as rustc finds it, through
  the prelude's `#[rustc_builtin_macro]` definitions, or by name when
  `core` is not in the session.
- **Results.** In `core`, `alloc` and `std`, 1,188 derives expand with no
  errors, and every path in them resolves.

### 6.6 Procedural macros

**Working as an experiment** (`src/proc_macro.rs`, `src/json.rs`). Proc-macro
libraries built by rustc run through the toolchain's
`libexec/rust-analyzer-proc-macro-srv`, a separate process. Its protocol
was learnt by running it (its error messages name the fields it wants),
and is written down at the top of `src/proc_macro.rs`.

- **Function-like macros** are found by name or path, like any macro.
- **Attribute macros** replace their item with their output.
- **Derives** add their output after the item. The item is passed
  `cfg`-stripped and without its `#[derive]`.
- **Test.** `tests/proc_macro/hello.rs` (a derive, a function-like and an
  attribute macro) expands `tests/proc_macro/use_hello.rs` to what rustc
  gives, including the exact tokens the macro receives.

This is not how rustc does it. rustc loads the library into its own
process and calls it through `proc_macro`'s bridge; the server and its
JSON are rust-analyzer's way round not being rustc. The server also
declares its API "highly unstable" and runs only with
`RUST_ANALYZER_INTERNALS_DO_NOT_USE` set. The alternative is to load the
library ourselves, which we will do once we compile proc-macro crates
(section 6.7); the token conversion and the expansion hooks stay.

### 6.7 Loading proc-macro libraries ourselves (later)

Decided (2026-11-05): the server stays until `epitome-rustc2` can
compile proc-macro crates itself. Then we build both sides of the
bridge: the macro library, and the `proc_macro` crate it links against.
The bridge becomes an interface of our own, not the private one of each
rustc version, and libraries are loaded in-process with no JSON and no
second process. Until then, proc macros must come from the same
toolchain as the server.

## 7. Types

### 7.1 Definitions

**Done** (`src/ty/collect.rs`). Every item gets its signature, keyed by its
`DefId`, in one pass over the definitions (an item's parent, an impl or
trait, always comes before it). It records:

- **Generic parameters**, with a trait's `Self` as parameter 0, an impl's
  or trait's parameters first in its items' lists, and defaults (which
  may name `Self`).
- **Predicates** from bounds and `where` clauses. `Item = T` bindings
  become projection predicates; `?Sized` is dropped.
- **Fields and variants**, function signatures (the receiver first),
  impl headers, supertraits and auto traits.
- **The types of aliases, constants and statics.** An alias is expanded
  where it is used, worked out when first needed.

`--dump-sigs` prints every signature, one per line:

```text
fn core::option::{impl}::map<U, F>(Option<T>, F) -> Option<U> where <F as FnOnce<(T,)>>::Output == U, F: FnOnce<(T,), Output = U>
fn core::iter::traits::iterator::Iterator::next(&mut Self) -> Option<<Self as Iterator>::Item>
```

On `core`, `alloc` and `std`, the run takes 0.07 s and gives 54,523
signatures and 13,818 impls. 0.3% of them contain an error type, every
one naming a crate outside the session (`libc`, `hashbrown`, `unwind`).

### 7.2 The type table

**Done** (`src/ty/mod.rs`, `src/intern.rs`). A type is a `Ty(u32)`, an
index into an interned table of `TyKind`s, so equal types have equal ids.
Everything inside a `TyKind` is an id as well, so `TyKind` is `Copy`. The
kinds:

- **Primitives:** `bool`, `char`, `str`, the integers and floats, `!`.
- **Built-up types:** `Tuple(List)`, `Ref` and `Ptr` (with mutability),
  `Array(Ty, ConstId)`, `Slice`.
- **Items:** `Adt(DefId, List)`, `FnDef(DefId, List)`, `FnPtr(SigId)`,
  `Dyn(DynId)`, `Foreign(DefId)`.
- **Generic and associated:** `Param(ParamId)`, `Projection(assoc DefId,
  List)` with the self type first in the list, `Opaque(OpaqueId)` for
  `impl Trait`, and `ConstArg(ConstId)` for a constant in an argument
  list.
- **For checking:** `Infer(var)` and `Error`.

Lifetimes are not kept: they matter for borrow checking, not code
generation.

The interner is split into 16 shards by hash, each with its own locks,
so parallel jobs can add types; an id's low bits name its shard. Maps use
our own hasher (`src/hash.rs`).

Turning written types into `Ty`s (`src/ty/lower.rs`) uses name
resolution's results:

- Missing generic arguments take their defaults.
- `T::Item` and `Self::Item` become projections through a trait bounding
  `T`. The bounds searched are those of the current item and its
  parents, so a method's `where T: Deref` bounds its impl's `T`.
- `<T as Tr>::Item` is a projection directly.
- `Fn(A) -> B` becomes `Fn<(A,)>` with `Output = B`.
- A module never stands for a type: `u8` in `core` is the primitive,
  though a module `u8` is in scope.

### 7.3 Checking a function

Each function body is checked on its own, in parallel: inference
variables, unification, expected-type propagation, integer and float
fallback, coercions (deref, unsize, reborrow, never, closures to `fn`
pointers), auto-deref and auto-ref in method lookup, operator overloading
through the lang-item traits, closures' capture analysis and their `Fn*`
kind, and patterns' default binding modes. The results go in side tables
keyed by `NodeId`: `ty`, `adjustments` (the derefs, borrows and
coercions applied to an expression), `method` (the picked method and its
generic arguments), `field` indices, and pattern binding modes.

Trait solving is a goal-based search over impls and where clauses with
a cache, including associated type projection and normalisation, auto
traits, `Sized`, and the built-in impls (`Copy` for primitives, `Fn` for
closures, and so on).

So far (`src/typeck/`):

- `infer.rs` holds a body's variables: general, integer or float, bound
  by unification, with snapshots to undo a probe (trying an impl).
- `check.rs` checks expressions against the type expected of them, and
  patterns against the type they match, through references with default
  binding modes. The types go in a table by node. Method calls find
  inherent methods through `&` and `Box`, borrowing the receiver for
  `&self`, and record the method. Impls are indexed by the head of their
  self type (an ADT, a primitive, `[T]`, `&T`, ...), so lookup does not
  scan every impl.
- The coercions so far: `!` to anything, `&mut T` to `&T`, `&[T; N]` to
  `&[T]`, and function items and closures to `fn` pointers.
- Unsuffixed integers and floats left unknown become `i32` and `f64`.

Const evaluation (array lengths, `const` items, const generics) is an
interpreter over the typed AST, the same walk as lowering but producing
values.

## 8. Lowering to Epitome IR

Lowering is demand-driven monomorphisation: start from `main` (or every
exported item of a library), and lower each `(DefId, generic args)`
instance once, walking its AST with the side tables and substituting
the arguments as it goes. Instances that are found go on a worklist;
functions are lowered in parallel.

- **Layout:** our own algorithm for `repr(Rust)` (fields reordered by
  alignment, niche-filling enums); `repr(C)`, `repr(transparent)`,
  `repr(u8)`, ... as the Reference specifies. Everything in a program is
  compiled by us, so our layout need not match rustc's.
- **Places and values:** every local is an `alloca`; Epitome's mem2reg
  and SROA turn them into registers. Expressions lower to an operand or
  a place, following the adjustments table.
- **Control flow:** blocks, `if`, `loop`, `while`, `for` (through
  `IntoIterator`/`Iterator::next`), labelled `break` with values, `?`
  (through `Try`), `let else`.
- **Matching:** each `match` becomes a decision tree over the scrutinee's
  places, tested column by column, with guards and or-patterns; the
  same code serves `if let`, `while let`, `let else` and function
  parameters.
- **Drops:** a scope stack records what needs dropping; every exit
  (fall-through, `break`, `return`, `?`, unwinding) runs the drops in
  reverse. A value moved on some paths only gets a drop flag. Drop glue
  for each type is generated on demand.
- **Panics:** `panic=abort` first; unwinding (`invoke` and landing pads)
  later, when Epitome's code generator supports it.
- **Closures** become a struct of captures plus a function; **`dyn`**
  becomes a vtable (drop, size, align, then methods); **`async` and
  generators** become state machines (late).
- **Symbols:** v0 mangling, as documented in the rustc book.
- **Intrinsics and lang items** are recognised by attribute and lowered
  to IR instructions or calls.

`--dump-ir` prints each function's IR with a comment at the start of the
instructions for each statement and expression, quoting the source, so
the IR can be read against the AST.

## 9. Crates and Cargo

Libraries are written as `.rlib` archives holding the object and a
metadata file in our own format: the crate's public definitions (as
collected in 7.1), and the AST of every generic and `#[inline]` function,
so a dependent crate can monomorphise them. The simplest form of the
latter is the source text of those items plus their resolved paths.

`epitome-rustc2` accepts rustc's command line as far as Cargo uses it
(`--crate-name`, `--crate-type`, `--edition`, `--cfg`, `--extern`, `-L`,
`--emit`, `-C`, `--out-dir`, `--error-format=json` messages), so it can
stand in for `RUSTC`.

`core` from `rust-src` uses many unstable features, intrinsics and lang
items; it is the main test of completeness. Until it compiles, programs
are tested with `#![no_std]` and a small `core` of our own in `tests/`.

## 10. Parallelism

Every stage is split into jobs small enough to keep all cores busy and
large enough that handing them out costs little: a file, a module, an
item, a function body, an instance. All the crates of a run share one
pool (`src/pool.rs`), so a small crate's jobs fill the gaps in a large
one's.

### 10.1 Rules

- **One writer per piece of data.** Each file's AST, each body's
  inference tables and each instance's IR belong to the job that makes
  them. Nothing is shared and written at once except a few append-only or
  insert-only tables (the source map, interners, caches), which are
  behind locks split into shards.
- **Rounds where a stage needs a fixed point.** Name resolution and
  macro expansion need what other modules define. They run in rounds:
  in each round every module's job reads only what the previous round
  published, and the round's results are published together at its end.
  Inside a round, jobs never see each other's work, so the order they
  run in cannot matter.
- **Output does not depend on scheduling.** Results are placed by a key
  that comes from the input (a file's place in the module tree, an item's
  path, an instance's symbol name), not by the order jobs finished. Ids
  handed out by a shared interner can differ between runs; nothing may
  depend on their order, and anything printed or emitted is sorted by
  content. A test runs the same input on 1 and on many threads and
  compares.
- **No global state.** All of a run's state is in a `Session`, so two
  sessions in one process do not interact.

### 10.2 Stage by stage

| Stage | Job | Shared state | Done |
|---|---|---|---|
| Read and parse | a file; finding `mod foo;` queues `foo.rs` | source map (append-only) | yes |
| `cfg` stripping | a file | none (the cfg set is read-only) | |
| Expansion | a module, per round | macro tables, published per round | |
| Resolution | a module, per round | per-module name tables, published per round | |
| Item collection | an item | definition table (insert-only) | |
| Type checking | a function body | type interner and trait cache (sharded) | |
| Const evaluation | a constant, on demand, once | results (one cell per constant, with cycle detection) | |
| Lowering | an instance; finding a call queues its callee | instance set (insert-only) | |
| Optimisation, code generation | a function | none | (Epitome's own) |

Expansion adds AST to the file that held the macro call, and that file
belongs to one job per round, so its arenas need no locks. Type checking
of a body needs only signatures, which are fixed before any body is
checked; bodies are checked in any order. Closures are checked with the
body they are in.

### 10.3 Crates

In one process, a crate's dependents need its signatures, not its code.
A dependency's collected items (what its metadata holds) are the only
barrier: once they are published, dependents start type checking while
the dependency is still lowering and generating code. Crates with no
path between them in the dependency graph run side by side.

### 10.4 How far this goes

On this machine (12 cores), loading `core`, `alloc` and `std` from
`rust-src` (849 files, every module followed) takes 0.40 s on one thread
and 0.06 s on twelve. Lexing and parsing use no shared state at all, so
they scale until reading the files becomes the limit.

## 11. The library

`epitome_rustc2` is a library crate; the program is `src/main.rs` over
it. The entry point is a `Session` (`src/session.rs`):

```rust
use epitome_rustc2::lexer::Edition;
use epitome_rustc2::session::{CrateRoot, Options, Session};

let session = Session::new(Options { threads: 0 }); // 0: one per core
let crates = session.parse_crates(vec![
    CrateRoot::new("app", "app/src/main.rs").edition(Edition::E2021),
    CrateRoot::new("util", "util/src/lib.rs"),
]);
for c in &crates {
    for e in &c.errors {
        eprint!("{}", e.render_in(&session.sources));
    }
    for f in &c.files {
        // f.module is the module path, f.ast the tree, f.parent the file
        // and `mod` item that declared it.
    }
}
```

Each later stage is a method on the session that takes the previous
stage's results and returns its own, which a host can inspect or print:
`parse_crates`, then `expand`, `resolve`, `check`, `lower` (to an
`epitome_ir::Module` per crate), `compile` (to object files). A host
that wants only part of the pipeline (an IDE wanting types, a tool
wanting the expanded AST) stops where it likes. Results borrow from the
session (tokens are slices of its source map), so a session lives as long
as anything taken from it.

Errors are values, not output: every stage returns them with its
results, and the program decides how to print them. A stage carries on
past an error in one file or body where it can, so that one run reports
everything it can find.

The library has no dependencies besides Epitome's own crates; the pool
is plain `std` threads.

## 12. Testing

- **Unit tests** in each module, on small strings, comparing `sexp`.
- **Parse corpus:** every `.rs` file in the toolchain's `rust-src`
  (`core`, `alloc`, `std`, ...) and in `~/.cargo/registry/src` must
  parse; `--pretty` output must parse again to the same tree. Files that
  rustc rejects are skipped by a list.
- **Expansion:** `-Zunpretty=expanded` from the real rustc (run as a
  program) against our `--pretty` after expansion, compared as token
  streams.
- **Run tests:** programs in `epitome-rustc2/tests/run/` with their
  expected output, compiled by both rustc and us and run; the outputs
  must match.
- **Crates:** `cargo test` of crates with `RUSTC=epitome-rustc2`,
  compared with `RUSTC_WRAPPER=epitome-rustc`.

Every test has a tight time limit, and everything is built `--release`.
Tests of parallel stages run on one thread and on several, and compare.

## 13. Clean room

- No rustc code is read or copied: not the lexer (`rustc_lexer`), parser,
  AST, resolver, type checker, trait solver, MIR, or codegen. Knowing
  rustc's overall stages (as in this document) is fine; its data
  structures are not copied, and there is no HIR or MIR here.
- Sources: the Rust Reference, the rustc book (command line, v0 symbol
  mangling), the standard library's documentation, and the behaviour of
  `rustc` and `rust-analyzer-proc-macro-srv` run as programs.
- The standard library's source is compiled by us (it is the input, not
  a reference for how to write a compiler).

## 14. Order of work

1. Lexer, parser, AST dump and pretty printer (done; `rust-src` and the
   crates.io cache parse and round-trip). The session and pool: whole
   crates, several at once, loaded in parallel (done).
2. `cfg`, `macro_rules!`, built-in macros and derives; expansion matches
   rustc's (done).
3. Proc macros (working through the toolchain's server; in-process
   loading once we compile proc-macro crates, section 6.7).
4. Name resolution and item collection (done).
5. Type checking: signatures and the type table (done); then bodies, for
   `#![no_std]` programs: integers, structs, enums, references, generics
   (done); then traits, methods, closures.
6. Lowering of those to Epitome IR; first run tests pass.
7. Drops, `dyn`, slices and strings, formatting through `core::fmt`.
8. `core`, then `alloc` and `std` from `rust-src`; Cargo builds.
9. Unwinding, `async`, and the long tail.
