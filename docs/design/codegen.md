# Code generation

This document covers how Epitome turns IR into machine code. For sources and
status labels, see the [index](README.md).

The code generator documentation marks several parts as "To Be Written". Those
parts are marked **Open** here, and Epitome designs them itself.

## 1. Phases

The code generator runs these phases in order:

1. **Instruction selection**: turns IR into target instructions that use
   virtual registers in SSA form.
2. **Scheduling and formation**: puts the selected instructions in order and
   builds machine instructions.
3. **SSA machine code optimisations**: optional. **Open**, not described in
   the documentation.
4. **Register allocation**: maps virtual registers to physical ones and adds
   spill code.
5. **Prologue and epilogue insertion**: builds the stack frame and replaces
   abstract stack references with real offsets. **Open**, not described.
6. **Late machine code optimisations**: **Open**, not described.
7. **Code emission**: writes assembly text or an object file.

IR passes `codegenprepare`, `lower-switch`, `lower-invoke` and
`break-crit-edges` (see [ir.md](ir.md)) run before phase 1.

## 2. Machine code data structures

| Structure | Role |
| --- | --- |
| Machine function | One per IR function. Holds its blocks, constant pool, frame info, target-specific info and register info. |
| Machine basic block | A list of machine instructions. One IR block can become several machine blocks. |
| Machine instruction | An opcode and operands. Operands that define values come first. |
| Bundle | A group of instructions under a `BUNDLE` header, with each member flagged as inside it. Used for VLIW packets and Thumb2 `IT` blocks. |
| Register info | Tracks virtual registers, their classes and, for GlobalISel, their types. |
| Register mask operand | Lists the registers a call clobbers. |

### Target description

A target provides these pieces. Only the target machine and data layout are
required.

| Piece | Role |
| --- | --- |
| Target machine | Entry point. Gives access to every other piece. |
| Data layout | Type sizes, alignment, endianness and pointer size. |
| Lowering | How IR operations and calls map onto the target. |
| Register info | Registers, register classes and aliases. |
| Instruction info | Instructions, plus helpers such as copying, spilling and analysing branches. |
| Frame lowering | Stack layout, prologue and epilogue. |
| Subtarget | CPU and feature flags, from `-mcpu=` and `-mattr=`. |
| Calling convention | Where arguments and return values go. Epitome describes conventions as named metadata in the target description; see [isel](isel.md#calling-conventions). |
| Assembly printer | Turns machine instructions into text or MC instructions. |

LLVM describes most of this in TableGen files. Epitome describes targets in
LLVM IR instead: each instruction is a function whose body says what it does,
with its encoding and assembly in attributes, read as data at start-up. See
[machine descriptions and instruction selection](isel.md).

## 3. Instruction selection

Epitome will have both selectors, as LLVM does. SelectionDAG comes first
because it is the most fully documented.

### SelectionDAG

The DAG is made of nodes connected by value edges. Value edges carry types.
Chain edges, of type `Other`, keep side effects in order.

Phases, in order:

1. **Build**: translate IR into a naive DAG.
2. **Combine**: simplify the DAG.
3. **Legalise types**: promote or expand types the target lacks. Vectors can
   also be widened, split or scalarised.
4. **Combine** again.
5. **Legalise operations**: expand, promote or custom-lower operations the
   target lacks.
6. **Combine** again.
7. **Select**: match DAG patterns to target instructions.
8. **Schedule**: put the DAG in a line and emit machine instructions.

The combiner runs after the build and after each legalisation step.

### GlobalISel

GlobalISel works on generic machine IR (gMIR). gMIR uses the same structures
as machine IR but with looser rules: a virtual register can have a type
without a register class. Types are kept in a table that is dropped after
selection.

| Pass | What it does | Status |
| --- | --- | --- |
| IRTranslator | Translates IR into gMIR. | Later |
| Legalizer | Replaces operations the target lacks with ones it has. | Later |
| RegBankSelect | Puts each virtual register in a register bank, grouping code to avoid copies between banks. | Later |
| InstructionSelect | Selects target instructions. No gMIR may remain afterwards. | Later |
| Combiner | Optional. Replaces instruction patterns with better ones. | Later |

**Open:** the details of generic opcodes, low-level types and the known-bits
analysis are on GlobalISel subpages we have not read yet.

## 4. Register allocation

### Before allocation

| Pass | What it does | Status |
| --- | --- | --- |
| Live variables | Finds where each register is killed or dead. Uses SSA for virtual registers and tracks physical registers within a block. | Planned |
| PHI elimination | Replaces PHI instructions with copies. Copies go at the end of each predecessor, through temporaries; see [isel](isel.md). | Done |
| Two-address | Rewrites three-address instructions into two-address form where the target needs it. | Planned |
| Live intervals | Numbers instructions and builds a live range for each virtual register. The documentation is incomplete here. Epitome keeps one segment per block; see [isel](isel.md). | Done |

### Allocators

Chosen with `-regalloc=`.

| Allocator | What it does | Status |
| --- | --- | --- |
| `fast` | Works one block at a time. The default for debug builds. | Planned |
| `basic` | Allocates live ranges one by one in a heuristic order. Useful for finding bugs. | Planned |
| linear scan | Epitome's first allocator: live ranges in order of start, copy hints, callee-saved registers for values live across calls, eviction of lighter values, spills through scratch registers; a spilled value's range split by blocks (below). Before it, copies into values nothing reads go, and the sides of a copy that never meet or hold the same value where they do are joined (coalescing). See [isel](isel.md). | Done |
| `greedy` | The default. Like `basic`, plus splitting live ranges across the function. | Later |

Epitome's splitting is by blocks, after linear scan (`split_by_blocks`): a
spilled value is stored after each definition, so memory always has it,
and a register free for all of its stretch in a block may hold it there.
For each spilled value, heaviest first, and each register of its class,
the blocks are chosen where the reads it saves (each block weighted 8 for
each loop it is in) outweigh the loads entering them, which go on the
edges: at the start of a block of one predecessor, or before the
terminator of a block of one successor. Leaving needs nothing. The
register that saves most is taken, then up to two more for the rest. In
its other blocks the value is reloaded before each use, or once for a
run of uses where a register is free between them. Within a block ranges
are not split (around one call, say), and there is no spill placement
across the function as greedy's has.
| `pbqp` | Solves allocation as a partitioned boolean quadratic problem. | Later |

### Allocation support

- **Pre-coloured registers**: physical registers already in the code, either
  as operands or as implicit uses and definitions.
- **Rewriting**: either change operands directly, or record a
  virtual-to-physical map and let a spiller rewrite the code.
- **Spiller**: adds loads and stores for spilled registers.
- **Folding**: folds a spill load or store into the instruction that uses it,
  to remove copies. Epitome folds reloads on x86: an instruction whose
  memory form's body is its own with a load for one register operand
  (`ADD64rm` of `ADD64rr`) reads a spilled value from its slot.

## 5. Machine passes after selection

The documentation names only a few of these. **Open:** the rest of the
pipeline is ours to design.

| Pass | Source | What it does | Status |
| --- | --- | --- | --- |
| Branch folder | Writing an LLVM Backend | Merges and simplifies branches using the target's branch analysis. Epitome skips blocks that only branch and drops branches to the next block. | Partial |
| If-converter | Writing an LLVM Backend | Turns branches into predicated instructions using branch analysis. | Later |
| VLIW packetiser | Code generator | Groups instructions into packets using a resource state machine. | Later |
| Tail call optimisation | Code generator | Guaranteed tail calls for `fastcc`, `tailcc`, or with `-tailcallopt`. | Later |
| Sibling call optimisation | Code generator | A narrower tail call that needs no flag, when conventions and stack arguments allow. | Planned |

Targets can add passes at fixed points in the pipeline. The documented hooks
are "add instruction selector" and "add before emission".

## 6. Code emission (MC layer)

| Piece | Role |
| --- | --- |
| Streamer | An assembler-like interface with one method per directive. It writes either a `.s` file or a `.o` file. Targets can add their own directives. |
| Context | Owns symbols and sections and makes sure each exists once. |
| Symbol | A label. Temporary labels are dropped from object files. |
| Section | An object-format section: ELF, Mach-O or COFF. |
| MC instruction | An opcode plus operands. Each operand is an immediate, a register or an expression. |
| Expression | A symbolic value used as an operand. |

To emit code, a target needs:

- An assembly printer.
- An instruction printer.
- Lowering from machine instructions to MC instructions. This also expands
  pseudo-instructions.
- An encoder, which turns MC instructions into bytes and relocations. Only
  needed for object files.

The assembler matches mnemonic aliases first, then instruction aliases.

Epitome has no streamer: assembly and object files are both written from
the allocated machine IR and one description of each global's data. See
[objects.md](objects.md) for object files (Mach-O and ELF: **Done**) and
`epitome-objdump`.

### Optional sections

| Flag | What it adds |
| --- | --- |
| `-stack-size-section` | Pairs of function symbol and stack size. Only counts the prologue, and leaves out functions with dynamic stack allocations. |
| `--call-graph-section` | Call graph information. The layout has its own document. |

## 7. Targets

**Open:** which target comes first. The suggestion is AArch64 or RISC-V,
because their encodings are regular and they are well documented by their
vendors. x86-64 should follow.

The code generator documentation also has notes on X86, PowerPC, NVPTX, eBPF
and AMDGPU, including x86-64 compact unwind encoding. These get their own
documents when we reach those targets.

## 8. Debugging options

These options show the DAG at each step. Epitome should support them as text
or `.dot` output: `-view-dag-combine1-dags`, `-view-legalize-dags`,
`-view-dag-combine2-dags`, `-view-isel-dags`, `-view-sched-dags`,
`-view-sunit-dags` and `-filter-view-dags`. `-debug-only=isel` and
`-filter-print-funcs=` control text dumps.

## 9. Build order

1. Machine IR data structures and a text printer for them.
2. A first target description: registers, instructions and calling
   convention.
3. SelectionDAG: build, then select, for already-legal code only.
4. Scheduling in source order.
5. PHI elimination, two-address, then the `fast` allocator.
6. Prologue and epilogue, then assembly output.
7. Type and operation legalisation, and the DAG combiner.
8. Live intervals, then `basic`, then `greedy`.
9. Object file output.
10. GlobalISel.
