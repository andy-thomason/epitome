# Epitome design documents

These documents describe what Epitome of synaptic excellence (Epitome for
short) has to do to work like LLVM. They are split into two halves:

- [IR and middle end](ir.md): the intermediate representation, the pass
  manager, and the analysis and transform passes that run on IR.
- [Code generation](codegen.md): lowering IR to machine code, from
  instruction selection to object file emission.
- [Debug info](debug-info.md): the plan for a typed debug-info model,
  parser, printer and verifier.
- [Parallel passes](parallel.md): how passes run on many functions at once,
  the inliner and its cost model, and turning recursion into loops with an
  explicit stack.
- [Alias analysis](aa.md): the data layout, and basic alias analysis.
- [Autochop](chop.md): every elective change counted, so a miscompilation
  is found by binary search to the change that causes it.
- [Memory SSA](memssa.md): memory versions, and the walker that finds the
  write reaching a load on demand, within a budget.
- [SROA](sroa.md): splitting allocas into pieces mem2reg can promote.
- [mem2reg](mem2reg.md): promoting allocas to SSA values.
- [Simplifying the CFG](simplifycfg.md): branch rules in the combiner, and
  the passes that change the graph's shape.
- [Machine descriptions and instruction selection](isel.md): targets
  described in LLVM IR, longest-match selection with a trie, register
  allocation, and assembly, binary and disassembly from one description.
- [Targets, CPUs and features](targets.md): adding a machine or a CPU
  without rebuilding, as description files found at run time; feature
  gates; relocations, registers and object-file identities as data; and
  RISC-V RV64, the second machine; inline assembly's constraints and
  modifiers as data; operands written by name; and AMD's R600
  (Evergreen), for assembly.
- [x86-64: a plan](x86.md): how x86 instructions are written in a
  description, and what the code generator must learn first.
- [Partial machines: a plan](partial-machines.md): machines without
  memory, a stack, calls or floating point; checking that a program fits
  one; fencing off what Epitome must not optimise or lower; and a VLIW
  GPU (TeraScale) to test it on.
- [Object files, assembly and disassembly](objects.md): writing Mach-O
  and ELF object files that match clang's assembler; `epitome-as`, which
  assembles with the machine description run backwards; and
  `epitome-objdump`, which disassembles with it.
- [Loops](loops.md): finding loops, rotation, induction variables and
  trip counts.
- [CSE](cse.md): common-subexpression elimination by hashing a canonical
  key for each instruction.
- [Combines](combine.md): the rule language, the discrimination tree that
  matches rules, and the combiner pass.
- [Code generation: what is left](backend-plan.md): the gap to clang,
  measured, and the plan for allocation, scheduling and machine passes.
- [Correct, but not yet good](later.md): code generation that gives the
  right answers the long way (through memory, lane by lane), and what
  would be better.
- [MLIR](mlir.md): reading MLIR, lowering it to Epitome's IR through
  dialect descriptions written in LLVM IR, and compiling `hw`, `comb` and
  `seq` into cycle-based simulators; the paths from PyTorch and from
  Chisel. Its specification is in [`docs/mlir`](../mlir/README.md).

## Clean-room rules

Everything here comes from LLVM's published documentation only. No LLVM
source code was read to write these documents, and none may be read when
implementing them.

When the documentation leaves something out, the gap is marked **Open** and
Epitome decides for itself. We do not fill a gap by looking at LLVM's source.

We may also run LLVM's tools, such as `clang -S -emit-llvm -x ir`, on our
own inputs and record what they print. Rules found this way are listed with
how they were found in
[observed behaviour](../llvm-ir/08-observed-behaviour.md).

### Sources

| Document | Used for |
| --- | --- |
| [LLVM's Analysis and Transform Passes](https://llvm.org/docs/Passes.html) | Every IR pass listed in `ir.md` |
| [The RISC-V Instruction Set Manual, Volume I](https://github.com/riscv/riscv-isa-manual) and the [RISC-V ELF psABI](https://github.com/riscv-non-isa/riscv-elf-psabi-doc) | The RISC-V RV64 description: encodings, the LP64D convention and relocations (`targets.md`) |
| [Intel 64 and IA-32 Architectures Software Developer's Manual](https://www.intel.com/content/www/us/en/developer/articles/technical/intel-sdm.html), [AMD64 Architecture Programmer's Manual](https://www.amd.com/en/search/documentation/hub.html) and the [System V x86-64 psABI](https://gitlab.com/x86-psABIs/x86-64-ABI) | The planned x86-64 description: encodings, semantics, the convention and relocations (`x86.md`) |
| Arm's A-profile Architecture Machine Readable Specification (AARCHMRS, open-source package, BSD 3-clause) | Checking the AArch64 description's encodings and mnemonics, and the bit layouts of entries generated from it (`isel.md`) |
| [AMD Evergreen Family Instruction Set Architecture: Instructions and Microcode](https://www.x.org/docs/AMD/old/AMD_Evergreen-Family_Instruction_Set_Architecture.pdf) (2011) | The R600 (Evergreen) description: microcode formats and opcodes (`targets.md`) |
| [Using the New Pass Manager](https://llvm.org/docs/NewPassManager.html) | Pass manager design and pipeline syntax |
| [The LLVM Target-Independent Code Generator](https://llvm.org/docs/CodeGenerator.html) | Codegen phases, data structures, SelectionDAG, register allocation, MC layer |
| [Global Instruction Selection](https://llvm.org/docs/GlobalISel/index.html) and [Core Pipeline](https://llvm.org/docs/GlobalISel/Pipeline.html) | GlobalISel passes and generic MIR |
| [Writing an LLVM Backend](https://llvm.org/docs/WritingAnLLVMBackend.html) | Backend components and pass hooks |
| [LLVM Language Reference](https://llvm.org/docs/LangRef.html) | IR semantics, to be cited per feature as it is built |
| Braun et al., "Simple and Efficient Construction of Static Single Assignment Form", CC 2013 | Rebuilding SSA form in `parallel.md` |
| Cooper, Harvey and Kennedy, "A Simple, Fast Dominance Algorithm", 2001 | The dominator tree |
| Tarjan, "Depth-First Search and Linear Graph Algorithms", 1972 | Call-graph SCCs |
| [MLIR's](https://mlir.llvm.org/docs/), [CIRCT's](https://circt.llvm.org/docs/) and torch-mlir's documentation, the FIRRTL specification; see [`docs/mlir`](../mlir/README.md) | `mlir.md` |

The pass list on the Passes page says it may be incomplete. Where we learn
about a pass from another public document, we add that source to the entry.

## Status labels

Each pass or component has one of these labels:

- **Planned**: in scope, not started.
- **Later**: in scope, but after the core works end to end.
- **Skip**: not needed for an alternative code generator that takes LLVM
  IR, such as GraphViz viewers.
- **Partial**: started; the entry says what is missing.
- **Done**: implemented and tested.
