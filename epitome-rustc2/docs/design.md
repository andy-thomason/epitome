# epitome-rustc2

This is the plan for `epitome-rustc2`, a Rust compiler written in Rust
that does not use rustc. It reads Rust source, expands macros, resolves
names, infers types and lowers the syntax tree straight to Epitome IR,
which Epitome's optimiser and code generator compile.

`epitome-rustc` (`src/bin/epitome-rustc.rs`) is a different program: it
wraps the real rustc and replaces only LLVM's code generation. It stays
useful as a reference. The same crate built by both must behave the
same.

**Status:** The front end up to macro expansion works.

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

Resolution fills `res: Vec<Option<Res>>` indexed by `NodeId` for every
path, and builds the module tree: `DefId`s for items, each module's
names in its type, value and macro namespaces, glob and named imports
resolved to a fixed point, the prelude, `extern crate`s and
`--extern` crates. Local variables resolve to the pattern node that binds
them. Hygiene for `macro_rules!` comes from each expanded token carrying
a syntax context (section 6.3), which resolution consults for locals and
labels.

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

A path finds the macro in the crate it names. Macros brought in by
`use` are found by name for now; name resolution (section 5) will
replace that.

### 6.5 Built-in macros and attributes

Done in expansion: `cfg_select!`, `cfg!`, `include!`, `stringify!`,
`concat!`, `line!`, `column!`, `file!`, `module_path!`, `env!`,
`option_env!`, `compile_error!`. A definition with
`#[rustc_builtin_macro]` dispatches to these by name.

Left as calls for type checking and lowering: `format_args!` (to a node
the type checker knows), `asm!`, `global_asm!`, `include_str!`,
`include_bytes!`, `offset_of!`, `assert!` and the like. Built-in derives
(`Clone`, `Copy`, `Debug`, `Default`, `PartialEq`, `Eq`, `PartialOrd`,
`Ord`, `Hash`) will be token generators, and attribute macros run after
`cfg`.

### 6.6 Procedural macros

Existing proc-macro crates must run unchanged. A proc-macro crate is a
dynamic library built by rustc whose interface to the compiler (the
`proc_macro` bridge) is private to each rustc version, so we do not load
it ourselves. Instead each rustup toolchain ships
`libexec/rust-analyzer-proc-macro-srv`, a program that loads proc-macro
libraries built by that toolchain and expands macros on request over
stdin and stdout. We run it as a program:

1. Cargo builds proc-macro crates with the real rustc (they run on the
   host; `epitome-rustc2` sees `--crate-type proc-macro` and passes the
   whole job to rustc, as `epitome-rustc` does).
2. When a crate uses one (`--extern serde_derive=...dylib`), we start the
   server from the same toolchain, ask it for the library's macros, and
   send each call's input as a token tree. Each token's span is an index
   into a table on our side, so tokens that come back with a span map to
   their original slices; new text is put in a source of its own in the
   `SourceMap`.
3. The output is parsed like a `macro_rules!` expansion.

The server's protocol is versioned and documented only by its behaviour,
so we pin to the versions of the toolchains we test with, and test
against `serde_derive`, `thiserror-impl` and a small proc macro of our
own in `tests/`. If that ever becomes unworkable, the fallback is a
small host program, built with rustc, that loads the library itself.

## 7. Types

### 7.1 Definitions

After expansion, a collection pass records for each item its generics,
predicates, field types, variant lists, signatures, associated items,
and impls (inherent impls by self type; trait impls by trait). This is
also what a crate writes as metadata (section 9).

### 7.2 The type table

Types are interned in one arena, `Ty(u32)`, so equality is comparing
indices:

- `Bool`, `Char`, `Int(I8..I128, Isize)`, `Uint(...)`, `Float(F16..F128)`,
  `Str`, `Never`;
- `Adt(DefId, Args)`, `Tuple(Tys)`, `Array(Ty, Const)`, `Slice(Ty)`,
  `Ref(Region, Mutability, Ty)`, `Ptr(Mutability, Ty)`;
- `FnDef(DefId, Args)`, `FnPtr(Sig)`, `Closure(NodeId, Args)`,
  `Dyn(Bounds)`, `Param(index)`, `Alias(projection or opaque)`,
  `Infer(var)`.

Regions are kept only as far as trait selection and `dyn` need them;
there is no borrow checker to feed.

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
2. `cfg`, `macro_rules!`, built-in macros; expansion matches rustc's
   (done; built-in derives and attribute macros to come).
3. Proc macros through the toolchain's server.
4. Name resolution and item collection.
5. Type checking for `#![no_std]` programs: integers, structs, enums,
   references, generics, traits, closures.
6. Lowering of those to Epitome IR; first run tests pass.
7. Drops, `dyn`, slices and strings, formatting through `core::fmt`.
8. `core`, then `alloc` and `std` from `rust-src`; Cargo builds.
9. Unwinding, `async`, and the long tail.
