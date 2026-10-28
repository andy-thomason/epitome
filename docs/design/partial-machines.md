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

- in `epitome-llc`, after `prepare` and legalisation (section 3), so that
  what those passes remove is not reported. A failure here is an error,
  and nothing is written;
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
| `prepare`: `memcpy`, `memset` | calls to the library | loops, or an error from `fits` if the size is not known |
| `prepare`: division, float runtime calls | calls to the library | recipes only; an error if there is none |
| switch lowering | jump tables (`globals`, `indirect`) | compare-and-branch trees |
| constant materialisation | constant pools (`memory`) | building in registers only |
| legalisation of wide integers | pairs of registers, which is fine; stack slots for `i128` | pairs only |
| register allocation | spills (`stack`) | an error naming the function and the number of registers it needed |
| frame layout | a frame (`stack`) | none, and none needed |
| the inliner | nothing, but its threshold matters | inline everything that is called once or is small, and report what is left (`calls`) |

Integers wider than the machine's widest class need care. Index arithmetic
from JAX and PyTorch is `i64` even when every index fits in 32 bits. A
narrowing pass, before legalisation, rewrites an `i64` computation as `i32`
where known bits and ranges (trip counts, `zext` from `i32`, constant
shapes) show that the high half is never needed. What it cannot narrow is
split into pairs, and with `i64` lacked, `fits` reports it.

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

**Operations.** Inside a function, inline assembly naming entries of the
description already passes through untouched. An entry marked `"asmonly"`
needs only an encoding and a template, so a description can cover a whole
instruction set without saying what each instruction computes. To the
optimiser an `asm` call is opaque: it is not moved across another `asm`
with side effects, not removed if it has them, and not duplicated if it
is `convergent`. Lowering from MLIR can write such calls for operations
it knows the machine has, keeping the operation's operands as SSA values
the optimiser still works on.

**Open:** whether opaque functions should be written as IR text or kept in
a section of the object file. Text is simpler, and nothing reads the
section yet.

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

## Status

| Item | Status |
| --- | --- |
| An empty `result` or `fp-result` list is an error, not a panic | **Done** |
| `!target.lacks`, and the structure check waiving what it covers | **Partial**: read; `stack` makes `sp` optional; the structure check waives nothing yet |
| Conventions without `stack` or `fp-*` entries | **Planned** |
| `fits`, in `epitome-llc` and `epitome-opt` | **Planned** |
| Smoke program compiled per function, reporting what does not fit | **Planned** |
| Passes inside the lacks list (section 3) | **Planned** |
| `i64` narrowing | **Planned** |
| `"epitome-opaque"` | **Planned** |
| `"role"="launch"` | **Planned** |
| TeraScale description, for assembly and disassembly | **Done**: ALU, control flow, a memory export and a vertex fetch, checked against `llc` |
| TeraScale compilation: groups, clauses, hardware loops | **Later** |
