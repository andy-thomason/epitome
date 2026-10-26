# SystemVerilog

This is the plan for `epitome-sv`, a SystemVerilog compiler. It reads
IEEE 1800-2023 source, elaborates the design, and compiles it with
Epitome's optimiser and code generator into a simulator, as the `hw`,
`comb` and `seq` path of [MLIR](mlir.md) does for circuits from CIRCT.

The work is measured against the
[IEEE 1800-2023 conformance tests](https://github.com/andy-thomason/IEEE1800-2023-tests)
(`test1800`): 1478 self-checking tests, one directory per clause of the
standard. For sources and status labels, see the [index](README.md).

**Status:** Partial. The lexer, the preprocessor and the parser are done.
Every legal `test1800` test preprocesses, and the output matches
Verilator's `-E` token for token on all 772 tests both accept (section
4.4). Every legal test parses except three that break the standard, and
the parse tests that should fail do, for the reasons they give (section
6.4). The design IR that elaboration produces is planned (section 7);
building elaboration is next.

## 1. Goals

- **Parse all of IEEE 1800-2023.** Every test in `test1800` that is
  legal must parse, and every test whose `:should_fail_because:` is a
  syntax error must be rejected, with a message that points at the line
  and column.
- **Elaborate and simulate** the synthesisable subset first, then the
  testbench language (classes, `fork`, constrained random, assertions) as
  `test1800` clause by clause asks for it.
- **Compile, not interpret.** A design becomes Epitome IR and then machine
  code. Clocked logic that settles in one pass becomes a cycle-based
  `eval` function, which is one very large basic block (see
  [parallel](parallel.md), "Big blocks").
- **Stay linear in the size of the input.** Large netlists have millions
  of lines. No pass may be worse than linear in a module's size, and
  modules are processed in parallel with the same output for any number
  of threads.
- **Follow the clean-room rules** in section 9.

Out of scope for now: SDF back-annotation (clause 32), protected envelopes
(clause 34), and the VPI, assertion and coverage C APIs (clauses 36–40).
They come after everything else passes.

## 2. Where it fits

```text
  .sv / .svh files
        │
        ▼
 ┌──────────────────────────── epitome-sv ───────────────────────────┐
 │ source map ─► preprocess ─► lex ─► parse ─► AST                   │
 │                                              │                    │
 │                         elaborate (parameters, generate, hierarchy)│
 │                                              ▼                    │
 │                  design IR (per specialisation; section 7)        │
 │                           │                       │               │
 │          netlist: hw / comb / seq      program: processes,        │
 │          ─► eval (section 8.1)         testbench (section 8.2)    │
 │                           └───────────┬───────────┘               │
 │                     Epitome IR + runtime calls                    │
 └──────────────────────────────────────────────┬────────────────────┘
                                                 ▼
                                  epitome_ir::Module
                                                 ▼
                        epitome-opt ─► epitome-codegen ─► .o
                                                 ▼
                         linked with the runtime (scheduler, $display,
                         strings, classes, queues) into a simulator
```

`epitome-sv` depends on `epitome-ir` only. The binary links the
optimiser and code generator.

## 3. Source text and positions

### 3.1 Tokens are slices

A token is a `&str` that points into the source text, as in the IR and
MLIR lexers:

```rust
pub struct Token<'a>(pub &'a str);
```

Its kind is worked out from its first characters when it is needed, and
its position from where the slice sits in memory: the byte offset is the
token's pointer minus the source's pointer, and the line and column come
from that offset. Tokens are two words, copy freely, and need no position
fields. The end-of-file token is the empty slice at the end of the text.

Error reports carry a token (or any subslice of the source), never a
line number worked out early.

### 3.2 The source map

The preprocessor produces text that is not in any one file: included
files, and macro bodies with their arguments substituted. Every piece of
text a token can point into is therefore held in one `SourceMap`:

- each file read, and each piece of text the preprocessor makes, is a
  `Source { name, text, origin }`, kept for the life of the compilation
  (text is never moved once added, so slices stay valid);
- `SourceMap::find(&str)` finds which source holds a slice from its
  address, and `SourceMap::render` gives `name:line:col`, the line and a
  caret. The search is linear; it only runs when reporting an error or
  printing `-E` output, and can become a binary search on address
  ranges if that is ever slow;
- `origin` says which macro made a piece of text and where it was used,
  so an error inside an expansion ends with "note: in expansion of
  `` `FOO ``" at the use;
- `` `line `` directives are kept with the source they appear in, and
  applied when a position is reported.

Most tokens come straight from a file, so the common case costs nothing.

## 4. Preprocessing (clause 22)

**Done**, in `sv/src/preprocess.rs`. `epitome-sv -E` prints its output.

### 4.1 Frames and tokens

The preprocessor reads tokens from a stack of frames, each a lexer over
some text in the source map: the files named on the command line, files
reached by `` `include ``, and the text of macro expansions. Any token
that is not a directive is passed through unchanged, still a slice of the
text it came from. Directives are acted on and dropped.

The output is `Output { tokens, states }`. Each token carries the index
of a `State`: the keyword set (`` `begin_keywords ``), `` `timescale ``,
`` `default_nettype ``, `` `unconnected_drive `` and `` `celldefine `` in
force when it was read. The parser asks `Output::is_keyword`, which uses
the token's own keyword set, rather than `Token::is_keyword`.

All the files of a run form one compilation unit, so macros carry from
one file to the next. Macros given with `-D name=text` or
`+define+name=text`, and the coverage constants of 40.3.1, are defined
first, from a source named `<command line>`.

### 4.2 Macros

A use of a macro is expanded by putting the actual arguments (or the
defaults) into the macro's text, as text, and pushing the result as a new
frame. Its source records the macro's name and where it was used, so an
error inside it ends with "note: in expansion of `` `M `` at file:line:col".
Because the text is read again, macros in the arguments and in the text
expand after substitution, and a macro passed as an argument cannot pick
up the outer macro's formal names (22.5.1).

The text of a definition runs to the first newline that is not escaped
by a backslash and is not inside a block comment or a `"""` string.
Building the expansion works on characters:

- comments are dropped; a backslash-newline becomes a newline, except
  inside a string literal, where both go;
- a formal's name, as a whole word, becomes the argument; nothing is
  replaced inside a string literal, even one inside `` `" `` or
  `` `""" ``;
- `` `" `` and `` `""" `` become `"` and `"""`, with arguments still
  replaced between them, and macros used between them expanded into the
  string; `` `\`" `` becomes `\"`; ` `` ` disappears, joining what is on
  either side.

A macro with no arguments and none of these forms in its text is read
straight from its definition, so its tokens point into the defining file.

A base and its digits may come from different macros (`` `W`B`D `` for
`12'hfA`). The preprocessor remembers that the last token was a base and
tells the next frame's lexer to read digits.

A macro that uses itself is caught when expansions nest 256 deep.
`` `include `` nests to 64 files.

### 4.3 The other directives

- `` `ifdef ``, `` `ifndef ``, `` `elsif ``, `` `else ``, `` `endif ``:
  a stack of conditions, with the 2023 expressions (`!`, `&&`, `||`,
  `->`, `<->`, parentheses). Ignored text is still lexed, so it must be
  lexically valid, and a `` `define `` in it is read past whole, so a
  directive in its text cannot end the block. A conditional must end in
  the file it started in.
- `` `include "file" `` looks in the including file's directory, then the
  `-I` directories, then the current directory. `` `include <file> ``
  looks only in `Options::system_include_dirs`, which is empty: Epitome
  has none of the files the standard defines. Only white space and
  comments may follow on the line.
- `` `timescale ``, `` `default_nettype ``, `` `unconnected_drive ``,
  `` `nounconnected_drive ``, `` `resetall ``, `` `begin_keywords `` and
  `` `end_keywords `` must be outside design elements. The preprocessor
  counts design elements from the keywords it passes through (`module`
  to `endmodule` and so on, but not `extern module`, `virtual interface`,
  an `interface` port or `interface class`).
- `` `line `` remaps the following lines of its file, for error messages
  and for `` `__LINE__ `` and `` `__FILE__ ``.
- `` `pragma `` needs a name and is otherwise ignored, as the standard
  allows for pragmas a tool does not know. Annex E's directives are
  accepted and ignored.

### 4.4 Testing

- Unit tests in `preprocess.rs` cover each rule above with small
  examples of our own.
- `tests/test1800.rs` preprocesses every test of `test1800`. All legal
  tests must preprocess; the six should-fail tests whose error is the
  preprocessor's must be rejected.
- `tests/reference.rs` runs Verilator as a program (`-E -P`) over the
  same tests, lexes its output, and compares the token streams. Set
  `SV_REFERENCE_PP` to the `verilator` binary. All 772 tests that both
  accept give the same tokens.

Where the reference and the standard differ, we follow the standard:

- In `` `define TQ(x) `"""a "x" b`""" ``, the `"x"` is a string literal
  (a `"` in macro text always starts one, 22.5.1), so `x` is not
  replaced. Verilator agrees. `test1800`'s
  `22.05-define/triple_quote_stringification.sv` expects `"a \"go\" b"`
  from `` `TQ(go) ``, which looks like a bug in the test
  ([IEEE1800-2023-tests#1](https://github.com/andy-thomason/IEEE1800-2023-tests/issues/1)).
- A macro with an escaped name takes formal arguments after the single
  space that ends the name (22.5.1). Verilator reads the parenthesis as
  part of the macro's text.

## 5. Lexing (clause 5)

The lexer turns text into tokens on demand, from a byte offset, as the
MLIR lexer does. The parts that need care:

- **Comments** `//` and `/* */` are dropped; `/*` does not nest.
- **Identifiers** are `[a-zA-Z_][a-zA-Z0-9_$]*`. **Escaped identifiers**
  run from `\` to the next white space, which ends them and is not part
  of the name. `\bus+index ` and `bus+index` are different names; `\cpu3 `
  and `cpu3` are the same (5.6.1).
- **System names** `$display`, and **directives** `` `name ``.
- **Keywords** are identifiers that the current keyword set reserves
  (Annex B). The lexer does not classify them; `Token::is_keyword` asks.
- **Numbers**: decimal, `'b`/`'o`/`'d`/`'h` with an optional size and `s`,
  `x`/`z`/`?` digits, underscores (not first), the unbased unsized
  literals `'0 '1 'x 'z`, reals with exponents, and time literals `10ns`,
  `1.5us`, `1step`. Spaces may separate the size from the `'` and the
  base from the digits (`8 'h FF`), but not the `'` from the base
  (`8' hFF` is an error). The lexer makes the size, the `'b` part and the
  digits separate tokens; the parser joins them into one literal.
- **Strings**: `"..."` with the escapes of 5.9.1 and a backslash-newline
  continuation, and the triple-quoted `"""..."""` of 1800-2023.
- **Operators** by longest match: `<<<=`, `>>>=`, `<->`, `|->`, `|=>`,
  `#-#`, `#=#`, `##`, `->>`, `:=`, `:/`, `::`, `.*`, `'{`, `@@`, `(*`
  and `*)` are not tokens: `@(*)` is a sensitivity list, so attributes
  are recognised by the parser as `(` immediately followed by `*`.
- **Context**: inside a `table` of a UDP, `0`, `1`, `x`, `?`, `b`, `r`,
  `f`, `p`, `n`, `*` and `-` are single-character entries. The parser
  asks the lexer for "table" tokens there, which is why tokens are made
  on demand rather than all at once.

## 6. Parsing (Annex A)

**Done** for all of Annex A, in `sv/src/parse/` and `sv/src/ast.rs`.
`epitome-sv --ast` prints the tree.

### 6.1 The tree

The tree follows the lexer's lead: a node holds no text, only where its
text is.

```rust
pub struct Node { kind: Kind, first: u32, end: u32, child_start: u32, child_len: u32 }
```

`first..end` is a range of the token list (each token a `&str` into its
source), and the children are a range of one shared list of `NodeId`s.
All nodes live in one arena, `Ast::nodes`; a node is 20 bytes. A leaf is
a node of one token (`Name`, `Keyword`, `Op`, `Number`, ...); a sized
literal such as `8'hFF` is one `Number` node over its three tokens.

Children are what carries meaning: names, literals, operators, the
keywords that pick a form (a port's direction, `posedge`), and
sub-constructs. Punctuation and keywords that only introduce a
construct are not children, but are still in the node's token range.
Children are always in source order. Attributes, `(* ... *)`, are an
`Attributes` node just before the item or operand they apply to. Each
`Kind` documents its children; where an optional child would leave two
forms looking alike, it has a wrapper (`Label`, `Iff`, `DisableIff`,
`NamedConnection`, `NamedArg`).

### 6.2 How it parses

A hand-written recursive-descent parser, one function per construct,
structured after Annex A. Each function pushes the node it builds onto
a stack; `finish(mark, kind)` turns everything pushed since `mark` into
the children of a new node. Wrapping something already built (the left
side of a binary operator, a select after a name) is another `finish`
with the earlier mark, so left recursion costs nothing. Expressions use
precedence climbing with the strengths of table 11-2, and properties
and sequences the same with table 16-3.

Keywords are those of the token's own `` `begin_keywords `` set (section
4.1), so `logic` is a name under `"1364-2005"`.

SystemVerilog cannot always be read without knowing which names are
types. The parser does not keep a table of type names; it looks at the
shape of what follows instead:

- a type followed by a name starts a declaration: `T x;`, `p::T #(8)
  x[4];`. A name before a name and `(` is an instantiation, `M u(...);`.
  So `M u;` is a variable of type `M` until elaboration finds that `M` is
  a module;
- a name followed by `.name name` in a module body is an interface port;
- in a sequence, `(` is tried as an expression first, then as a
  parenthesised sequence (`(a ##1 b)`, `(s, x = 1)`);
- in `@(...)`, a parenthesis is tried as a nested event expression, then
  as part of an expression.

Backing up is cheap: it truncates the arena and the stack. The input of
150 000 lines of assignments and `if`s parses (with preprocessing) in
0.6 s.

The first error stops the parse. When readings were abandoned along the
way, the error reported is the one that got furthest through the
tokens.

### 6.3 Rules checked while parsing

Beyond the grammar, the parser rejects what the standard forbids and
the tokens alone show, among them: keywords as names; a literal of size
0; bad digits for a base; `x`, `z` or `?` in a string's octal or hex
escape; nested attributes; packed dimensions without a range, or on an
unpacked struct or union; a signed unpacked struct; `rand` in a packed
struct; an implicit data type without `var`; `type(...)` starting a
variable without `var` (except `type(this)`, as 6.23 writes it); drive
strengths without an assignment, charge strengths on anything but
`trireg`, `(highz0, highz1)`; `vectored` without a range; `wire reg`;
the restrictions on `interconnect`; a `localparam` without a value in a
parameter port list; declarations after statements in a block; a block
with both a label and a name, or an end label that differs; a for loop
that declares only some of its variables; positional arguments after
named ones; `ref` with a direction; mixed ordered and named connections
or parameter values; ports in a generate block, or in the body of a
module with ANSI ports; a generate region inside a generate block;
`import` in a class; `local protected`; a static class method; `cover`
with `else`, `restrict` with an action; delays on `tran` and `pullup`;
UDPs inside modules, with two outputs, vector or `inout` ports, `reg`
on a combinational output, bad table symbols, edges in combinational
tables, two edges in a row, wide or delayed initial values; specify
blocks outside modules, `ifnone` on edge paths, system tasks in specify
blocks and timing checks outside them; `$width` with a notifier but no
threshold; a DPI import without a return type.

### 6.4 Testing

- Unit tests in `parse/tests.rs` check the tree for each kind of
  construct, and the message and position of each rule above.
- `tests/test1800.rs` parses every test of `test1800`. Every legal test
  must parse and every `:type: parse` should-fail test must be rejected.
  Today: 778 of 781 legal tests parse and all 340 parse should-fail
  tests but one are rejected, each for the reason the test gives.

The exceptions are listed in the test with their reasons:

- `missing_parentheses_error.sv` (`M u;`) is valid syntax; elaboration
  will reject it.
- Three legal tests break the standard, and issues are raised:
  `escape_sequences_octal.sv` (`\7z`, IEEE1800-2023-tests#4),
  `nettype_valid_data_types.sv` (dimensions after a nettype's name,
  #3) and `coverage_save_then_merge.sv` (`before` as a name, #2).

## 7. Elaboration and the design IR (clauses 3.12, 23, 27)

Elaboration turns the AST into a **design**: it binds names, evaluates
parameters and constant expressions, unrolls generate blocks, resolves
the hierarchy from the top modules downwards, and gives every expression
its type and width (clause 11's rules for self- and context-determined
operands). The design is Epitome's own IR, the first of three layers:

| Layer | Holds | Form |
| --- | --- | --- |
| Design (this section) | Everything the language means, with names and types resolved | Rust arenas, below |
| Netlist (section 8.1) | The synthesisable part: cells, wires, registers, memories | MLIR `hw`, `comb` and `seq`, in `epitome-mlir`'s form |
| Program (section 8.2) | Everything event-driven: processes, testbench code, the scheduler | `epitome_ir` calling a Rust runtime |

`llhd` is not used for now. It has delta and epsilon steps but no
reactive set, Preponed or Postponed region (clause 4.4), so the runtime
needs its own scheduler anyway. CIRCT's `moore` dialect is not used as a
model: it is another tool's reading of the standard (section 9).

The design keeps what the netlist cannot: four-state values, strengths,
the hierarchy, regions, classes and dynamic data. It is shaped by RTLIL's
split between wires, cells, connections and processes, but it is a
language-level IR, not a netlist.

### 7.1 Principles

- **Arenas and ids.** Like the AST (section 6.1), every table is a `Vec`
  indexed by a `u32` id, with children as ranges of a shared list. No
  `Rc`, no pointers between entries, no strings except interned symbols.
- **Every entry points back to the AST.** Each object, expression and
  statement stores the `NodeId` it came from, so an error at any later
  stage has a line and column through the token's `&str`.
- **Elaborated once per specialisation.** A module, interface, program
  or checker with one set of parameter values, type parameters and
  interface-port bindings is a **specialisation**, elaborated once and
  shared by all its instances. Specialisations are independent, so they
  are elaborated in parallel once their parameters are known.
- **The hierarchy is kept, not flattened.** Instances refer to
  specialisations. The netlist flattens (as [MLIR](mlir.md) section 7.2
  does) or keeps large modules as functions; the design does neither.
- **Clause 11 is applied once.** After elaboration every operand has its
  final width and signedness, and every extension, truncation and
  conversion is an explicit node. No later layer re-derives clause 11.
- **Structured statements.** Procedural code stays as a tree (`if`,
  `case`, loops), not a control-flow graph. The netlist needs the
  structure to turn an `always_ff` into multiplexers and enables, as
  RTLIL's processes do with their switch rules; the program layer builds
  the graph when it lowers.

### 7.2 Values and types

Types are interned (`TypeId`). Packed types all reduce to a **bit
vector** with a width, a signedness and a state count (two or four),
plus a layout naming its fields and dimensions so that member and part
selects can be turned into offsets (clauses 6.9–6.11, 7.2.1, 7.4.1).
The other kinds are:

- `real`, `shortreal`, `realtime`;
- unpacked arrays: fixed, dynamic, queue and associative (clause 7);
- unpacked structs and unions;
- `string`, `chandle`, `event`;
- enums (a base vector type and its named values; clause 6.19);
- class handles, interface classes, virtual interfaces;
- nets of a user-defined nettype (clause 6.6.7), with its resolution
  function.

Constants are interned too (`ConstId`): a four-state vector of any width
stored as two word arrays, value bits and unknown bits, so that `0`,
`1`, `x` and `z` are the four combinations of one bit from each (clause
6.3.1). Two-state vectors have no unknown words. Reals, strings and
unpacked aggregates of constants are constants as well.

### 7.3 Scopes and objects

A **scope** is a specialisation, a generate block, a named or unnamed
block, a task or function, a class, a package, the compilation unit, or a
clocking block. Each has a parent, a symbol table, and the objects,
processes and instances declared in it. Unnamed generate blocks get the
names clause 27.6 gives them.

An **object** is what RTLIL calls a wire, generalised:

- **variable**: type, lifetime (static or automatic; clause 6.21),
  initial value (run before any process starts; clause 10.5);
- **net**: type, net kind (`wire`, `tri`, `wand`, `wor`, `tri0`, `tri1`,
  `trireg`, `supply0`, `supply1`, `uwire`, or a user nettype), its
  drivers (below) and, for `trireg`, its charge strength;
- **parameter**, **localparam**, **specparam**: a constant, kept for
  hierarchical references, `$bits` and the like;
- **port**: a direction and the object inside it (clause 23.2.2).

References across the hierarchy (`a.b.c`, upwards names, `$root`; clause
23.6–23.8) are resolved during elaboration to an instance path and an
object id. A reference into a class or a virtual interface stays a typed
member access, resolved when the program runs.

### 7.4 Drivers, connections and cells

The structural part follows clause 4.9:

- A **continuous assignment** (clause 10.3) is a driver: target,
  expression, drive strength, delay. So is an output port that drives an
  outside net, and an input port driven from outside (clause 4.9.6).
- An **inout port** connecting two nets is an **alias**, a tran with no
  strength reduction. Where both sides are nets of the same type with no
  delay, the two nets are merged, which is what RTLIL's connections do.
- A **cell** is a gate, a switch, a UDP or a pull (clause 28, 29):
  kind, terminals (each a one-bit net or net expression), strengths and
  delays. A UDP's table is kept as data.
- An **instance** names a specialisation and binds each of its ports to
  an expression or a net, with `.*` and implicit names already expanded
  (clause 23.3.2).

A net's value is resolved from its drivers, cells and aliases by the
rules of clause 28.12 for built-in net kinds and by the resolution
function for a user nettype. The design records which kind each net
needs: one driver and no strengths (the common case, which the netlist
can take), several drivers of a wired kind, or strengths and switches.

### 7.5 Expressions and statements

Expressions are typed nodes in an arena, with explicit conversions:
constant, object reference, hierarchical reference, bit and part select
(indices normalised to offsets from the least significant bit, with the
out-of-range rule of clause 11.5.1), member, the unary, binary and
conditional operators with their four-state meaning (clause 11.4),
concatenation and replication, streaming (clause 11.4.14), `inside`,
casts, calls to tasks, functions, methods and system functions, `new`,
and assignment patterns (clause 10.9) expanded to their elements.

Statements mirror clause 9–13: blocks (sequential, or `fork` with its
join kind), assignments (blocking, nonblocking, `assign`, `deassign`,
`force`, `release`, each with any intra-assignment timing), timing
controls (`#`, `@`, `wait`, `##`), `if` and `case` with `unique`,
`unique0` and `priority`, loops, `disable`, `return`, `break`,
`continue`, event triggers, immediate and deferred assertions, and
calls.

### 7.6 Processes

A **process** is RTLIL's process generalised to clause 9: its kind
(`initial`, `always`, `always_comb`, `always_latch`, `always_ff`,
`final`), its body, and two facts the later layers need:

- **its region set**: active, or reactive for code in a program or a
  checker and for assertion action blocks (clauses 4.4.2.6, 24);
- **its sensitivity**: for `always_comb`, `always_latch` and `@*`, the
  set clause 9.2.2.2 defines (what the body reads, including through
  called functions, less what it writes first); for an `always_ff` or a
  leading `@(posedge …)`, the clock edges and any asynchronous ones.

Tasks, functions and class methods are bodies in their scopes, not
processes. Classes keep their members, methods, virtual slots,
constraints and covergroups for the program layer. Concurrent
assertions keep their sequences and properties as expression trees, with
the clock and `disable iff` each one resolves to (clauses 16.15, 16.16).

### 7.7 Constant evaluation, and an interpreter

Parameters, generate conditions, array bounds and constant functions
(clause 13.4.3) are evaluated during elaboration by an interpreter over
expressions and statements in this IR. Simulation uses the same
four-state rules, so the interpreter is written once and grows into a
**reference simulator**: a slow, event-driven run of the design, with the
regions of clause 4.5. It lets `test1800`'s simulation tests pass before
the compiled path does, and every compiled result is checked against it.

### 7.8 Which code goes where

Elaboration sorts processes and nets between the two lower layers:

- **Netlist** (section 8.1): `always_ff`, `always_comb`, `always_latch`
  and continuous assignments with no delays and no calls into the
  runtime, writing nets with one driver and no strengths, or variables
  written by that process only. Four-state vectors become two `iN`
  values, one for the value bits and one for the unknown bits; a design
  that asks for two-state simulation keeps only the first.
- **Program** (section 8.2): everything else, including any process
  that the netlist rules reject, which then reads and writes netlist
  values through the state struct between `eval` calls.

### 7.9 Printing and testing

`epitome-sv --design file.sv` prints the design as text, one entry per
line, in the generic MLIR form that `epitome-mlir` reads, so that it can
be diffed and round-tripped like the AST's `sexp`. The 347 `:type:
elaboration` tests in `test1800` are the measure: legal ones elaborate,
and the others fail for the reason they give.

## 8. Simulation

Clause 4's scheduler has regions; the nine that simulation uses are
Preponed, Active, Inactive, NBA, Observed, Reactive, Re-Inactive, Re-NBA
and Postponed. The PLI-only regions wait for VPI. Within a region the
order of events is free (clause 4.7), so a fixed order chosen at compile
time is a correct one; only the region boundaries must be kept.

### 8.1 The netlist

The processes and nets that section 7.8 sends here become one `hw.module`
per specialisation, built in `epitome-mlir`'s in-memory form:
`always_ff` becomes `seq` registers with multiplexers and enables from
its `if` and `case` tree, `always_comb` and continuous assignments
become `comb`. From there it is the [MLIR](mlir.md) section 7 path:
flatten, sort, and build one `eval` function, which Epitome compiles as a
big block. The same form can be written out and given to CIRCT's tools to
check our netlists.

### 8.2 The program

Everything else is lowered to Epitome IR:

- **Processes become functions.** Each remaining `always`, `initial`,
  `final` and forked thread is a function. A process that waits (`@`,
  `#`, `wait`) is a state machine: its locals live in a frame and each
  wait point is a resume label, so threads cost no stack.
- **A runtime in Rust** holds the regions, the time wheel, NBA updates,
  net resolution for nets with several drivers or strengths, strings,
  dynamic arrays, queues, associative arrays, class objects, mailboxes,
  semaphores and `$display`-style formatting. Generated code calls it
  through `extern "C"` functions.
- **The netlist runs inside the Active region.** When a value it reads
  changes, the runtime calls `eval`; registers update at the clock edges
  `eval` sees, as nonblocking assignments would.

## 9. Clean-room rules

`epitome-sv` is a clean-room build from IEEE 1800-2023 alone. The
standard's prose, grammar (Annex A) and keyword list (Annex B) say what
the compiler must do. We do not read the source code of other
SystemVerilog tools (Verilator, Icarus, slang, Surelog, sv-parser, Yosys,
or any commercial simulator). Running them as programs, as `test1800`
does, to see what they accept and print is fair; reading how they do it
is not.

The standard is copyright IEEE, so **nothing is copied from it**: no
prose, no BNF productions, no tables, no examples, in code, comments,
tests or documents. We cite the clause number (`// 5.7.1`) and describe
the rule in our own words, as `test1800` does. The parser is structured
after Annex A, but its comments say what a function accepts in our words
rather than quoting the production. The PDF itself is never committed.

## 10. Build order and testing

`epitome-sv` gets a `test1800` profile of the `generic` kind: it is run as
`epitome-sv [flags] test.sv`, and its exit status and output are judged
the same way as Verilator's and Icarus's. Each step below ends when its
pass rate on the clauses it targets stops climbing.

1. **Lexer** (Done in skeleton): every `.sv` file in `test1800` tokenises
   without error, except tests that `:should_fail_because:` of a lexical
   rule. A cargo test walks the suite when `IEEE1800_TESTS` points at a
   checkout of it.
2. **Parser** (Done): every `:type: parse` test (341 of them) gives the
   right answer, and every legal test in the suite parses; see section
   6.4.
3. **Preprocessor** (Done): clause 22 and every test that uses
   `` `define ``; see section 4.4.
4. **Elaboration**: the design IR of section 7, and its interpreter for
   constant expressions; `:type: elaboration` tests, clauses 6, 7, 11,
   23, 26, 27.
5. **Simulation of the synthesisable subset**: the runtime with
   `$display`, `$finish`, `initial`, `always_ff`, `always_comb`, NBAs and
   `#` delays. Every test prints `PASS` this way, so this unlocks the
   bulk of clauses 3–13.
6. **Testbench language**: classes, queues, `fork`/`join`, mailboxes,
   semaphores, constrained random, then assertions and coverage.

The aim at each stage is to pass more of `test1800` than Verilator and
Icarus (66.8% and 68.6% today).

## 11. Open questions

- **Decided:** the preprocessor runs over the whole compilation unit
  before parsing, and each token records the keyword set it was read
  under, so `` `begin_keywords `` needs nothing lazy.
- **Open:** how much of four-state arithmetic to do inline in IR and how
  much to call the runtime for, for wide vectors.
- **Decided:** three layers (section 7). The design is Epitome's own IR;
  the synthesisable part lowers to MLIR `hw`/`comb`/`seq` and reuses
  that path; the rest goes straight to Epitome IR and the runtime. `llhd`
  is not used, and `moore` is not a model.
- **Open:** whether the reference interpreter (section 7.7) runs whole
  simulations in `test1800`, or only constant functions, once the
  compiled path works.
