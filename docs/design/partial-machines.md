# Partial machines: a plan

Every description so far is of a whole computer: memory, a stack, calls,
globals, 64-bit integers and floating point. Many machines worth compiling
for are less than that. An accelerator's control processor may have
registers and branches but no memory it can load from, no stack and no
calls. A GPU core may have no stack, and its loops counted by hardware. A
DSP may have no floating point. Code for them is still written as loops and
arithmetic, and lowering a model from JAX or PyTorch through `linalg` gives
exactly that kind of IR.

Today such a machine cannot be described. `--check-target` demands the
roles a whole computer needs (`load.x`, `store.x`, `call`, `trap`,
`addr.hi`), a 64-bit integer class, and a convention with a stack and
floating-point registers. The work-around, inventing a stack pointer and
spill instructions the hardware does not have, makes code that cannot run.

This plan is for four things:

1. a description says what its machine lacks;
2. a program is checked against that before it is lowered, and rejected
   with a reason if it does not fit;
3. passes that would add what the machine lacks do not;
4. parts of a program can be fenced off, so that Epitome neither optimises
   nor lowers them, and something else does.

None of it names a machine. For sources and status labels, see the
[index](README.md).

## 1. Saying what a machine lacks

`!target.lacks` lists what is missing, from a fixed vocabulary:

```llvm
!target.lacks = !{!1400}
!1400 = !{!"memory", !"calls", !"float", !"i64"}
```

| Name | The machine has no | So the description need not have | And a program may not use |
| --- | --- | --- | --- |
| `memory` | loads or stores | `load.*`, `store.*`, `addr.*`, `got.*`, data relocations | `load`, `store`, atomics, `alloca` that survives `mem2reg`, globals other than constants folded away |
| `stack` | stack (it may have other memory) | `sp`, `frame`, `saved`, `stack-align`, spill roles | `alloca` that survives, calls that pass arguments on the stack, spills |
| `calls` | calls or returns to a caller | `call`, `callr`, `tailcall`; `ret` is the end of the program | calls that are not inlined, recursion not turned into loops |
| `globals` | addresses of symbols | `addr.*`, `got.*`, relocations | globals, function pointers, jump tables |
| `float` | floating-point registers | the `fp-*` lists of a convention | floating-point types and operations |
| `i64` | integers wider than its widest class | a 64-bit class | integers wider than the widest class that cannot be narrowed (section 3) |
| `trap` | way to stop on an error | `trap` | `unreachable` that is reached, `llvm.trap` |
| `indirect` | branch through a register | `jumpr` | `indirectbr`, jump tables |

`memory` implies `stack` and `globals`. Pointers stay 64-bit in IR; with
`memory` there are none left after the check to lower.

The structure check stops asking for what a lacked thing would need, and
says so: `structure  lacks memory, calls: 9 requirements waived`. A
convention of a machine that lacks `stack` needs only `args` and `result`;
one that lacks `float` needs none of the `fp-*` lists.

The smoke program is compiled one function at a time, and a function that
does not fit (section 2) is reported as such and not compiled, in the way
functions needing an optional feature are reported now.

**Open:** whether `lacks` is per CPU, like features, or for the whole
description. Per description is enough for every machine considered so
far.

## 2. Does the program fit?

A pass, `fits`, walks the IR before anything is lowered and lists each use
of something the machine lacks, with the function, the block and the
instruction:

```
error: kernel: block loop: %p = alloca [16 x float]: the machine has no stack
error: kernel: call @expf: the machine has no calls, and @expf is not defined here
```

It runs twice:

- in `epitome-llc`, on each function after narrowing (section 3) and
  before it is lowered, and again on what `prepare` made of it, so that
  what that pass adds (a library call, a 64-bit product) is reported too.
  A failure here is an error, and nothing is written;
- in `epitome-opt`, when given `--target`, at the end of the pipeline, as a
  warning, so that a program that will not fit is found where the
  optimiser can still be asked to do something about it (inline harder,
  unroll fully, promote).

Selection, register allocation and frame layout then assume the program
fits, and an assertion that fails in them is a bug in `fits` or in a pass,
not in the program.

## 3. Passes stay inside

Some passes add what was not in the program. With a lacks list they must
not, or must do it another way:

| Pass | Adds | Instead |
| --- | --- | --- |
| `prepare`: `memcpy`, `memmove`, `memset` | calls to the library | loops of byte loads and stores (a `memmove` copying down where its ranges overlap) |
| `prepare`: division, float runtime calls | calls to the library | recipes only; an error from `fits` if there is none |
| switch lowering | jump tables (`globals`, `indirect`) | compare-and-branch trees (Epitome makes no jump tables) |
| constant materialisation | constant pools (`memory`) | building in registers only (Epitome makes no constant pools) |
| legalisation of wide integers | pairs of registers, which is fine; stack slots for `i128` | pairs only |
| register allocation | spills (`stack`) | an error naming the function and the number of registers it needed |
| frame layout | a frame (`stack`) | none, and none needed |
| the inliner | nothing, but its threshold matters | inline everything that is called once or is small, and report what is left (`calls`) |

Integers wider than the machine's widest class need care. Index arithmetic
from JAX and PyTorch is `i64` even when every index fits in 32 bits, and so
is Rust's `usize` on a 64-bit target. A narrowing pass (`ir/src/narrow.rs`),
before `fits` and `prepare`, rewrites a wide computation in the machine's
widest integers where only the low bits of its values are needed. It
bounds each wide value's bits from what it is made of (extensions,
constants, arithmetic that cannot wrap, shifts) and bounds loop counters
by their exit tests: up by one while below `n`, up by one until equal to
`n` (`add nuw`, or from 0 with `n` known not to be 0 on the way in), and
down by one to zero from a start known not to be zero. A value is
narrowed when every use needs only its low bits, or compares it with
another value where both fit. What it cannot narrow (the high half of a
product, a counter whose end may not fit) is left wide, and `fits`
reports it.

## 4. Fencing off part of a program

Some operations have no IR form: a matrix product done by a fixed-function
unit, a send to a neighbouring core, a wait on a barrier. And some
functions are for a different machine altogether, compiled by something
else and launched from this one. Epitome has to carry these through
without optimising them away or trying to lower them.

**Functions.** A function with `"epitome-opaque"` is not optimised, not
inlined and not lowered. It is written to the output as it came in (its IR
text, with the module's types and the declarations it uses) beside the
object file, for another tool. A call to it is kept, and is treated as
reading and writing all memory unless its own memory attributes say less.
`optnone` already keeps a function out of the inliner and the pipeline
(`opt/src/inline.rs`, `opt/src/pipeline.rs`); `"epitome-opaque"` adds
keeping it out of code generation. On a machine that lacks `calls`, a call
to an opaque function must have a role that turns it into something the
machine does do (`"role"="launch"`, an instruction that starts another
unit's program by its symbol), or `fits` reports it.

**Operations.** A description may declare the machine's own intrinsics:
declarations marked `"intrinsic"` (`declare void @tiny.signal(i32, i32)
"intrinsic"`), which its entries' bodies call as they call LLVM's. A
program calls them to have an instruction the IR has no operation for (a
matrix product on a fixed-function unit, a signal to another unit); the
call is selected as that instruction, a constant argument as an
immediate, and `fits` does not count it as a call. An operand that must be
in one register (one of a pair the instruction names by its first) is of
a `"fixed"` class, and selection copies the value there just before it.
`tests/partial.rs` (`machine_intrinsics_are_instructions`) has both.

Inside a function, inline assembly naming entries of the
description already passes through untouched. An entry marked `"asmonly"`
needs only an encoding and a template, so a description can cover a whole
instruction set without saying what each instruction computes. To the
optimiser an `asm` call is opaque: it is not moved across another `asm`
with side effects, not removed if it has them, and not duplicated if it
is `convergent`. Lowering from MLIR can write such calls for operations
it knows the machine has, keeping the operation's operands as SSA values
the optimiser still works on.

`epitome-llc` writes the opaque functions to `<output>.opaque.ll`: the
module with every other function a declaration (an internal one made
external). The optimiser takes their bodies out of the module while it
runs, so that no pass reads, changes or inlines them, and puts them back.
A call to one on a machine with a `launch` entry is that instruction, the
arguments where a call's go and a relocation naming the function.

**Open:** whether opaque functions should be kept in a section of the
object file instead. Text is simpler, and nothing reads the section yet.

## 5. A public target to test it on

All of this should be exercised by a machine anyone can check: AMD's
TeraScale (R600 and Evergreen) shader cores, from AMD's published
instruction-set document. It suits because it is partial in the ways
above (no stack, no calls, no general memory in the ALU clauses) and
because it is VLIW: up to five operations issued together in a group, and
loops counted by hardware.

`codegen/targets/r600` describes it for assembly and disassembly, from the
Evergreen document's microcode formats (`targets.md`, "AMD R600
(Evergreen)"). LLVM's r600 back end has no assembler, so `llvm-mc` cannot
check it; but `llc -show-mc-encoding` writes each instruction's bytes
beside its assembly, and `tests/r600.rs` assembles every such line and
compares.

To compile for it, the code generator needs what it does not yet have,
which belongs in a plan of its own:

- groups: which instructions may go in which slot (four vector, one
  transcendental), the reads each group may make of the register banks
  and constants, and the last of a group marked; the packetiser of
  `codegen.md` (**Later** there);
- clauses: ALU, fetch and control-flow instructions in streams of their
  own, the control flow pointing into the others;
- loops whose count is set by an instruction and whose body is a range of
  bundles;
- scheduling by latency, since results are not ready in the next group.

## A toy machine, and Rust

`codegen/tests/partial/tiny.ll` describes a made-up control core: sixteen
32-bit registers, arithmetic, compare-and-branch and `halt`, lacking
memory, calls, floating point, `i64`, `indirect` and `trap`.
`tests/partial.rs` checks it passes `--check-target`, compiles Rust's
`fib` (`tests/partial/fib.rs`) for it from rustc's IR and runs it on an
interpreter of the toy's instructions, and checks that programs using what
it lacks are turned away with each use named.

A no_std Rust function is compiled for such a machine by way of rustc's IR
for a bare-metal target with 32-bit `u32` arithmetic, optimised (at
`opt-level=0` the IR keeps calls and stack slots, which do not fit):

    rustc --crate-type=lib --target riscv64imac-unknown-none-elf \
          -C opt-level=2 -C panic=abort --emit=llvm-ir fib.rs
    epitome-llc --target=codegen/tests/partial/tiny.ll fib.ll -o fib.o

The convention names the triple (`riscv64`) among its own, so the module's
triple selects it.

## Status

| Item | Status |
| --- | --- |
| An empty `result` or `fp-result` list is an error, not a panic | **Done** |
| `!target.lacks`, and the structure check waiving what it covers | **Done**: `load.x`, `store.x`, `add`, `addi` (stack), `call` (calls), `trap`, `addr.*` (globals), the 64-bit class (i64); `structure` notes how many were waived |
| Conventions without `stack` or `fp-*` entries | **Done**: without a stack, `args` and `result` are enough; without calls, every register is free to use |
| `fits`, in `epitome-llc` and `epitome-opt` | **Done**, in `epitome_ir::fits`: run on each function before it is lowered and again after `prepare`, an error in `epitome-llc`, a warning in `epitome-opt --target` |
| Smoke program compiled per function, reporting what does not fit | **Done**: noted, not failed |
| Passes inside the lacks list (section 3) | **Done** for the passes of section 3: memory operations as loops without calls; division by a constant keeps the division without `i64`; spills without a stack are an error naming the function; no unwind tables without calls; no data without globals; what `prepare` still adds (the overflow check of a multiplication, made with a 64-bit product) is reported by `fits` |
| `ret` ends the program without `calls`; `unreachable` ends it without `trap` | **Done** |
| `i64` narrowing | **Done**: `epitome_ir::narrow`, in `epitome-llc` and before `epitome-opt --target`'s check |
| `"epitome-opaque"` | **Done**: kept out of the optimiser and of code generation, written to `<output>.opaque.ll` |
| `"role"="launch"` | **Done**: a call to an opaque function, on a machine with a launch entry; `fits` reports one that returns a value or passes arguments on a stack the machine lacks |
| TeraScale description, for assembly and disassembly | **Done**: ALU, control flow, a memory export and a vertex fetch, checked against `llc` |
| TeraScale compilation: groups, clauses, hardware loops | **Later** |
