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
4.4). Every legal test parses, and the parse tests that should fail do,
for the reasons they give (section 6.4). Elaboration builds the design
IR of section 7: every legal test elaborates (apart from eight that need
one compilation unit per file),
and 654 of the 673 should-fail tests that elaboration can reach are
rejected for their stated reasons (section 7.10). Simulation is next.

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
 │          interpreter: bytecode,        netlist: hw / comb / seq   │
 │          scheduler (section 8.2)       as lanes (8.1, 8.3)        │
 │          runs everything; default      synthesisable logic, when  │
 │                                        run time matters           │
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
  Today: all 781 legal tests parse and all 341 parse should-fail tests
  but one are rejected, each for the reason the test gives.

The exception is listed in the test: `missing_parentheses_error.sv`
(`M u;`) is valid syntax, which elaboration rejects.

Legal tests found to break the standard were fixed in the suite
(issues IEEE1800-2023-tests#1 to #4, and PR #5, which also corrected 41
tests that elaboration found: `timeunit` after other items, initialised
variables in static blocks without `static`, and a UDP row of all x
inputs with a known output).

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
| Interpreter (section 8.2) | Everything, and the scheduler; the only home of testbench code | Bytecode per specialisation |

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

Each process also records its **read and write sets**: the objects it
may read and write, including through tasks and functions it calls.
The thread pool (section 8.5) uses them to decide which processes can
run at the same time.

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
  written by that process only. How their values are laid out in
  memory, and the two-state and four-state models, are in section 8.3.
- **Interpreter** (section 8.2): everything else, including any process
  that the netlist rules reject, which then reads and writes netlist
  values through the state struct between `eval` calls.

### 7.9 Printing and testing

`epitome-sv --design file.sv` prints the design as text: each
specialisation once, with its parameter values and instance count, then
the hierarchy. Code is printed in SystemVerilog's syntax with the
conversions of clause 11 written out: `zext<8>(x)`, `sext<8>(x)` and
`trunc<4>(x)` change an integral value's width, `cvt<T>(x)` any other
conversion. Processes carry their read, write and sensitivity sets:

```text
module counter#1 #(W = 32'sd4) {  // 2 instances
  ports (input clk, input rst, input en, output q)
  parameter W: int = 32'sd4
  input wire clk: logic
  ...
  output var q: logic[3:0]
  always_ff reads(clk, rst, en, q) writes(q)
    @(posedge clk)
      if (rst)
        q <= 4'd0;
      else
        if (en)
          q <= trunc<4>((zext<32>(q) + cvt<logic[31:0]>(32'sd1)));
}
```

**Changed from the plan:** the text form is this readable one, not the
generic MLIR form; an MLIR form can be added when the netlist lowering
needs it. The measure is `elaborate_test1800` in `sv/tests/test1800.rs`:
legal tests elaborate, and should-fail elaboration tests fail.

### 7.10 What is built

In `sv/src/design/` (`types.rs`, `value.rs`, `print.rs`, and `elab/`):

- **Values**: four-state vectors of any width with clause 11's operators,
  used by constant evaluation and to be used by the reference simulator.
- **Elaboration in three passes** (declare, declarations in order, code),
  so code may name anything in the hierarchy. Specialisations are shared
  by instances with the same parameter values, except where a design
  element refers upwards through the hierarchy.
- **Constant evaluation** over the IR, including constant functions run
  by a statement interpreter (13.4.3).
- **Checks**: about 190 rules beyond binding and typing, each citing its
  clause: name spaces and imports (3.13, 26), drivers and lifetimes (6.5,
  6.21), types and casts (6.19–6.24, 7), classes (8), processes (9),
  assignments (10), operators (11), ports and hierarchy (23), programs
  (24), interfaces and modports (25), generate (27), gates and UDPs (28,
  29), specify (30, 31) and DPI (35).

**Measured** (`test1800`): all 776 legal tests that are run elaborate;
654 should-fail tests are rejected (the parser rejects most of the
syntax ones, elaboration the rest). Nineteen should-fail tests are not
checked yet, and are listed as such: clocking blocks (14.3), checker
bodies (17), mailbox element types (15.4), clock inference (16.9),
configurations (33) and specialised virtual interface types (25.9).
Legal tests that broke the standard were fixed in the suite's PR #5.

**Gaps, counted:** across the suite, 536 expressions have a type the
elaborator does not work out yet (members of interfaces reached through
ports, built-in class methods, tagged unions) and 248 items are kept as
syntax (assertions, covergroups, clocking blocks, specify blocks).
Neither is hidden: `Design::gaps` reports both.

**Not yet:** `defparam` is checked but its value is not applied; `bind`
is recognised but not elaborated; parameterised classes are elaborated
once, generically; one compilation unit per file (3.12.1 b) is not
supported; arguments and children are per-node vectors rather than
ranges of a shared list.

**Speed:** 2,000 specialisations (92,000 lines) elaborate in 1.25 s,
using 700 MB, most of it the per-node vectors. Packing them is the first
thing to do for memory.

## 8. Simulation

Clause 4's scheduler has regions; the nine that simulation uses are
Preponed, Active, Inactive, NBA, Observed, Reactive, Re-Inactive, Re-NBA
and Postponed. The PLI-only regions wait for VPI. Within a region the
order of events is free (clause 4.7), so a fixed order chosen at compile
time is a correct one; only the region boundaries must be kept.

There are two ways to run a design, and they share one scheduler:

- **The interpreter** (section 8.2) runs everything, and is the default.
  It is built for a short time to first result: 100,000 lines of
  SystemVerilog should be simulating within a second or two, with no
  code generation at all. It is also the reference that the compiled
  path is checked against.
- **The compiled netlist** (sections 8.1, 8.3) runs the synthesisable
  logic, laid out as a structure of arrays with instances as vector
  lanes, and compiled by Epitome's code generator when run time matters
  more than build time. Concurrent assertions, which are clocked and
  sample like registers, may go with it.

Everything else (testbench code, classes, `fork`, dynamic data, system
tasks) stays in the interpreter for good. Such code is rarely where a
simulation spends its time, and compiling it would cost build time for
little gain. (This replaces an earlier plan to compile it to Epitome IR.)

### 8.1 The netlist

The processes and nets that section 7.8 sends here become one `hw.module`
per specialisation, built in `epitome-mlir`'s in-memory form:
`always_ff` becomes `seq` registers with multiplexers and enables from
its `if` and `case` tree, `always_comb` and continuous assignments
become `comb`. The same form can be written out and given to CIRCT's
tools to check our netlists.

[MLIR](mlir.md) section 7 flattens a design and builds one scalar `eval`.
`epitome-sv` does not flatten: it builds one `eval` per specialisation
that runs its instances at once, a lane per instance, scalar code where
there is one and lanes side by side in an integer register where there
are several (section 8.3). The scalar path of MLIR stays for designs
from Chisel and for checking.

**Code size is a goal.** The compiled code grows with the design's
source, not with its instance count or hierarchy. Each specialisation
becomes one function whose argument is a structure holding the state of
its instances (section 8.3), or rarely a few, one for each number of
instances it runs with. With more lanes than a register holds, that
function loops over the registers rather than being unrolled across
them, so more instances means more calls or more passes of the loop,
never more code; a large design must not become millions of lines of
generated code. Choosing lanes over bit planes for
wide arithmetic (section 8.3) keeps the code small as well as fast. In
the `pmp_entry` mock (`sv/examples/soa_pmp_entry.rs`), `eval` with its
addresses in 32-bit lanes is 91 instructions with one loop; the same
module in bit planes is 890.

### 8.2 The interpreter

**Building it is linear and cheap.** After elaboration, three passes,
each linear in the size of the design:

1. **Layout.** Every static object (variable, net, the static locals of
   blocks and subroutines) of a specialisation gets a slot in that
   specialisation's frame. Each instance gets a frame base in one state
   store, so an object's slot is the instance's base plus the object's
   offset. Automatic variables live in per-call frames.
2. **Compile code once per specialisation.** Expressions and statements
   are turned into a flat bytecode for a stack machine, with every name
   already resolved to a frame offset, a constant, or an entry in the
   instance's link table (hierarchical references, interface ports),
   and every clause 11 conversion already explicit in the IR. Instances
   of a specialisation share its code.
3. **Link instances.** Each instance fills its link table and its port
   connections, which become continuous drivers (4.9.6).

No pass looks at the source text again, and nothing waits on a code
generator.

**Values** are four-state vectors stored inline up to 64 bits (value and
unknown words) and on the heap above; reals, strings and handles have
their own variants. Two-state objects carry no unknown word.

**Processes** are coroutines over the bytecode: a program counter, an
operand stack and a frame, so any number of them wait at no cost to the
native stack. A process that waits records why: a time (in the time
wheel), an event expression (on the waiting lists of the slots it
reads), or a condition (re-evaluated when its slots change).

**Continuous assignments and port connections** are small processes
sensitive to what they read; nets with several drivers or strengths keep
one value per driver and resolve them (clause 28).

**The scheduler** follows clause 4.5: the active and reactive region
sets iterated to quiescence, nonblocking updates applied in order, then
Postponed (`$strobe`, `$monitor`). It is the one the compiled netlist
joins later.

**Measures:** the share of `test1800`'s simulation tests that print
`PASS`, and the time from source to first simulated cycle on a large
generated design.

### 8.3 Instances as lanes

The netlist's state is kept as a structure of arrays, indexed by
instance. Each instance of a specialisation is a **lane** of its
function, and how many lanes the function has decides the code it is:

- **Lanes.** A simulation can hold independent copies of the whole
  design, `S` of them (with different seeds or stimulus, so a regression
  of many runs costs little more than one); `S` is a choice of the build,
  and one by default. The top has `S` lanes. A single instance of a child
  has its parent's lanes, and an array of `k` instances made by a
  generate loop is one call of the child with `k` times as many, where
  that widens it: pmp's eight `pmp_entry`s are one call with eight lanes
  for one simulation. So for few simulations a specialisation's lanes
  are the times it is instantiated in one, times `S`; for many, `S`.
- **Shapes.** The lanes, rounded up, set the code's shape: one lane is
  **scalar code** (plain integers, below); 2, 4 or 8 lanes are **SWAR
  code**, the lanes side by side in one `i64` (below); more are a loop
  over registers of 8 lanes, padding lanes running on zeros. One core
  running one program runs scalar code for each module it has once, and
  SWAR code only where it has several, as for the eight `pmp_entry`s of
  a `pmp`. (Vectors of 32-bit lanes, 4, 8 or 16 to a 128-, 256- or
  512-bit register, are kept behind an option, `Options::vectors`, to
  come back to; they paid off only for many simulations.)
- **Many simulations fit a core's cache.** Each function is a loop over
  all of its state, so with more state than a core's second-level cache
  every pass goes to the third level: CVA6's issue stage, 1.6 MB for 512
  simulations, ran three times as slowly on a Zen 5 core (1 MB of L2) as
  for 128. A netlist for many simulations holds as many of them (down to
  16) as keep its state within 512 KB, and the caller runs several
  states.

**Scalar code.** A function of one lane works on plain integers: each
value is one 64-bit piece up to 64 bits, and 64-bit pieces above. It
shares its lowering with vector code (folding, layouts, rolled loops,
recognised idioms) and differs where a branch beats a multiplexer: an arm
of an `if` or a `case` big enough to be worth a jump (a handful of
instructions) is behind a real branch, the values either arm changes
joined after it, as a compiler for software would; smaller arms are
both run and selected between. A `break` or `return` leaves: the rest of
the loop, block or function is behind a branch on not having taken one,
so a loop's variable stays a constant in each iteration. Values are
stored as narrow as they fit (a byte, a half, a word, 64-bit pieces).

**SWAR code.** A register of `n` lanes is an `i64` cut into `n` fields
of `64 / n` bits, 32, 16 or 8, and a value is computed on in pieces of
that many bits: a 56-bit address for eight lanes is seven `i64`s, each
holding a byte of each lane's. A constant is its piece repeated in every
field, and a mask (a comparison's result, an `if`'s condition) is all
ones or all zeros a field, so a multiplexer is `y ^ ((x ^ y) & m)`.
Logic is one instruction. A sum keeps its carries in their fields by
adding below the fields' top bits and putting those back with an
exclusive or; a difference does the same with the top bits set in the
minuend, and the borrow out of a field's top bit is an unsigned
comparison, which with the top bits flipped is a signed one. Shifts by
a constant are a shift and a mask, by a value the same in every lane a
mask worked out from it, and by one a lane a shift by each power of two
chosen by that bit of the amount. What has no such form (products,
counts of leading zeros) is done a field at a time. Every value is in
general-purpose registers, and no vector instructions are needed.

Values are stored as they are in registers, an `i64` a piece, except
one-bit values, which are bit planes: a bit a lane, so eight lanes'
enable is one byte. Loaded, a plane is spread to the lowest bit of each
field (three shifts, ors and masks for eight lanes), and stored, gathered
back.

**Layout of a value.** Each object has one of two layouts, chosen at
lowering from the operations that use it:

- **Bit planes**: one vector per bit; bit `i` of lane `n` is in vector
  `i`. A `w`-bit two-state value for 512 instances takes `w` vectors (3
  for a 3-bit value).
- **Lanes** of 8, 16 or 32 bits, rounding the width up; wider values are
  lanes of 32-bit pieces. A `w`-bit value takes the rounded width in
  vectors.

A four-state value has a second array of the same shape for the unknown
bits: `0`, `1`, `z` and `x` are value and unknown bits `00`, `10`, `01`
and `11`. So a 1-bit four-state signal for 512 instances is two
vectors, and a 3-bit one is six.

**Operations decide.** From 8 bits up both layouts take the same space,
and below 8 bit planes are smaller, so space rarely decides. The work
does:

| Operation | Bit planes | Lanes |
| --- | --- | --- |
| bitwise operators, multiplexers | one instruction per vector, or fewer with three-input logic | one per vector |
| bit and part selects, concatenation, shifts by a constant | free: they choose which vectors to use | shifts and masks |
| `+`, `-` | a ripple carry: a sum and a carry per bit | one per vector |
| comparisons, `*`, `/`, shifts by a variable | many per bit | one or a few per vector |

So an object that only meets the first two rows, such as a 32-bit bus
that is masked, selected and multiplexed, stays in bit planes whatever
its width; one that meets the last two, such as a 4-bit counter, goes to
lanes. The choice is made per object by adding up the cost of its uses
in each layout. Converting between layouts is a transpose, so each object
keeps one layout, and where the operands of an operation differ the
cheaper side is converted.

**Three-input logic.** Bit planes are all bitwise logic, and both targets
have instructions that take three inputs:

- x86-64 with AVX-512 has `vpternlogd`/`q`, which computes *any*
  function of three inputs, chosen by an 8-bit truth table in its
  immediate. The sum and the carry are one instruction each, so an add is
  two per bit (16 for 8 bits, against 8 on lanes); a multiplexer is one;
  and a four-state operator, whose value and unknown results each depend
  on four input planes, is a few.
- AArch64 with SVE2, which streaming SME mode includes, has fixed
  three-input forms: `eor3` (the sum), `bsl` and its variants (a
  multiplexer, and the carry as a select on `a ^ b`) and `bcax`.

Code generation does not use them yet. The plan, owned by the combiner
because it helps all code, not only simulators, is in
[combines](combine.md), "Three-input bit functions".

**No branches in vector code.** `if` and `case` are already
multiplexers in the netlist (save costly arms, run only where some lane
of a register takes them). Each lane's clock edge is a mask, so instances
on different clocks, or with enables, share one `eval`: a register takes
its next value where the mask is set and keeps its old one elsewhere.

**Ports between specialisations.** A child's lanes are ordered by
parent. A single instance has its parent's lanes, child lane `n` its
parent's lane `n`, and ports connected to a value whole connect with no
data movement: the child reads and writes the parent's value where it
is. An array of `k` instances in a parent of one register, `w` lanes
wide (its register's width, padding included), is one call: iteration
`i` has the child's lanes `i * w` to `(i + 1) * w`, and the parent
gathers each iteration's inputs into its block and scatters its outputs
back, a lane at a time, once a call. In SWAR code each lane's value is
taken apart into 32-bit digits and put together again as the other
side's pieces, in registers, and each of the child's registers stored
whole. A parent of more than one register gains no width from that and
would lose ports that connect in place, so it calls each instance with
its own lanes. An array whose call would wait on itself (each
iteration's input made from the last one's output, as a chain of adders
is) is called an instance at a time too.

A parent's code may loop through a child without any value depending on
itself: an output of the child feeds logic that feeds another of its
inputs, on which that output does not depend (CVA6's scoreboard and its
issue logic each read the other's outputs). Which outputs depend on which
inputs is worked out statement by statement, and the child is called
twice: first for the outputs that need none of the inputs the loop goes
through, then for the rest. Its code is then two functions, the first
holding only what those outputs need.

**Indexed arrays.** A select by a value differs from lane to lane, so it
is a choice among the elements, each the same constant select, where
that costs less than shifting the whole value (it costs the value's
pieces squared). An array of elements wider than a piece, indexed so, is
laid out with each element starting a piece, so that the same bits of
every element are the same shift, done once. Such arrays, and other wide
values indexed by a value or registers no clocked process reads, stay in
the state: a select loads the pieces it reads, an assignment stores those
it changes in the lanes that run it, and the arms of an `if` need no
multiplexers for them.

**Two models.** The same netlist is lowered twice:

- **four-state**: value and unknown arrays, and the four-state rules of
  clause 11 for every operator;
- **two-state**: value arrays only, and most operators one instruction.
  `x` and `z` become `0` on the way in, as a conversion to a two-state
  type does.

Two-state halves the state and removes most of the work of four-state
logic. Which one runs is chosen per simulation; tests that look at `x`
need four-state. We keep both if two-state is measurably faster, which is
the expectation.

### 8.4 Fast mode

`epitome-sv --fast` builds a simulator that keeps only the netlist of
section 8.1, for speed when a design is trusted and only its outputs
matter: long runs, regressions, software running on a simulated
processor. The standard defines no synthesisable subset, so "synthesisable"
here means what section 7.8 sends to the netlist, plus these changes,
which are what a synthesis tool would do with the same source:

- **Dropped:** immediate, deferred and concurrent assertions, `assume`,
  `cover`, `expect`, covergroups and checkers; system tasks with no effect
  on the design's state (`$display` and its family, `$monitor`,
  `$strobe`, `$info`, `$warning`, `$error`, `$fatal`); `final`
  procedures; specify blocks and timing checks.
- **Ignored:** delays (`#` on assignments, nets, gates and continuous
  assignments are zero), and drive strengths where a net has one driver.
- **Kept as initial values:** an `initial` procedure, or a declaration
  initialiser, that only assigns constants to variables (register reset
  values as an FPGA flow would load them). Any other `initial` is
  dropped.
- **Two-state by default:** `--fast` implies the two-state model of
  section 8.3; `--fast --four-state` keeps unknown bits.

Every other process is outside the fast design. What drives it is chosen
by where the design's boundary is:

- **The whole design** (the default): the top module's ports are the
  interface, set and read by a harness in Rust or C through the
  generated state header and `eval`, as with arcilator's models
  ([MLIR](mlir.md) section 7.1). Testbench modules are not compiled.
- **`--dut <path>`**: only the subtree under that instance is made fast;
  everything above it, the testbench, still runs in full through the
  program layer and the runtime, and calls the fast subtree's `eval` in
  the Active region (section 8.2). So an existing SystemVerilog
  testbench drives a fast design.

Dropping code changes what a run prints and when it stops, so a fast
build reports what it dropped: one line per kind of construct with a
count, and with `--explain`, each construct's position. A design whose
fast netlist still holds a process the netlist rules reject (a delay that
matters, a call into the runtime from inside the DUT) is an error naming
it, never a silent change.

**Testing.** Fast mode is checked against full mode: for every
`test1800` test and example whose design is synthesisable, both builds
run the same stimulus and must give the same values on the top-level
outputs at every clock edge (with `x` compared as `0` against
two-state).

### 8.5 Running on a thread pool, without locks

A simulation runs on a pool of worker threads, one per core, that share
no locks. The work of a time slot is cut into **tasks** before the run
starts, and each step of the scheduler is a **phase**: a list of tasks
that may run in any order and at the same time.

**Taking work.** A phase is a slice of task ids and one atomic index.
Each worker takes the next task with `fetch_add(1)` on the index, runs
it, and takes another, until the index passes the end. A second atomic
counts finished tasks; the worker that finishes the last one starts the
next phase by publishing it and bumping an epoch counter, which the
others wait on (spinning briefly, then parking). No task ever waits for
another inside a phase, so there is nothing to lock.

**Why tasks need no locks.** Every word of simulation state has exactly
one task that may write it in a given phase, decided when the design is
partitioned, and readers of that word run in later phases or read a copy
that does not change during the phase:

- **The netlist** (section 8.3) is two phases per clock edge.
  *Evaluate*: each task is one specialisation's lane group, or a slice
  of a large one; it reads registers' current values and the inputs, and
  writes its combinational values and its registers' next values. Where
  one task's combinational output is another's input (a port between
  specialisations), the tasks are levelised and each level is a phase.
  *Commit*: next values become current, as nonblocking assignments
  would (clause 4.9.4); each task commits what it wrote.
- **Processes** in the program layer run in parallel when the read and
  write sets elaboration gives them (section 7.6) show they cannot
  touch the same variable; processes that can are put in the same task
  and run one after another. Clause 4.7 lets events in one region run in
  any order, so this is a legal schedule.
- **Nonblocking assignments** from a task go to that task's own list.
  The NBA region applies the lists in task order, so a process's updates
  stay in the order it made them (clause 4.6) and the result does not
  depend on which thread ran which task.
- **Events and wakeups** are written to per-task lists too, and merged
  in task order at the end of the phase.
- **Nets with several drivers** have one slot per driver; a resolution
  task per net, in a later phase, reads them all.

**Determinism.** A task's output depends only on what it reads, and
merges are in task order, so a run gives the same result on 1 thread as
on 64. Tests run each simulation both ways and compare.

**Sizing tasks.** A task should be large enough to cost more than taking
it (one atomic add); a lane group of a few hundred operations is. Small
specialisations are batched together; large ones are cut at registers,
which break combinational dependence.

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
4. **Elaboration** (Done, with the gaps in section 7.10): the design IR
   of section 7, and its interpreter for constant expressions;
   `:type: elaboration` tests.
5. **The interpreter** (section 8.2): the scheduler, processes,
   continuous assignments and `$display`/`$finish` first, since every
   simulation test prints `PASS`; then the testbench language (classes,
   queues, `fork`, mailboxes, constrained random), then assertions and
   coverage. Time from source to first cycle is measured throughout.
6. **The compiled netlist** (sections 8.1, 8.3): synthesisable logic as a
   structure of arrays with instances as lanes, checked against the
   interpreter; then fast mode (8.4) and the thread pool (8.5).

The aim at each stage is to pass more of `test1800` than Verilator and
Icarus (66.8% and 68.6% today).

## 11. Open questions

- **Decided:** the preprocessor runs over the whole compilation unit
  before parsing, and each token records the keyword set it was read
  under, so `` `begin_keywords `` needs nothing lazy.
- **Decided:** netlist state is a structure of arrays, one lane per
  instance; each object in bit planes or in 8/16/32-bit lanes, chosen by
  the operations that use it; a two-state and a four-state model
  (section 8.3).
- **Decided:** the simulation runs on a thread pool with no locks:
  phases of tasks taken by an atomic index, one writer per word of state,
  per-task lists merged in task order (section 8.5).
- **Decided:** a fast mode that keeps only the netlist, drops
  assertions, coverage and other non-synthesisable code, and is driven by
  a harness or, with `--dut`, by the full testbench (section 8.4).
- **Open:** whether 64-bit lanes beat 32-bit pieces for wide values on
  machines with 64-bit vector multiplies.
- **Open:** whether to skip groups of lanes whose inputs did not change,
  and at what size of design that pays for its checks.
- **Decided:** three layers (section 7). The design is Epitome's own IR;
  the synthesisable part lowers to MLIR `hw`/`comb`/`seq` and reuses
  that path; the rest is interpreted (section 8.2). `llhd` is not used,
  and `moore` is not a model.
- **Decided:** the interpreter runs whole simulations and is the
  default; testbench code is never compiled; the compiled netlist is an
  accelerator for synthesisable logic (section 8).
