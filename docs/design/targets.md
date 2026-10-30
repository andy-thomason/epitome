# Targets, CPUs and features

A machine is its description: `.ll` files read at run time. Adding a
machine, a CPU or a relocation means writing a description file. You don't
change Epitome's Rust or rebuild `epitome-llc`. AArch64 is built in for
convenience. RISC-V RV64 (`codegen/targets/riscv64/`) is not: it is read
from disk like any description someone else might write.

This document covers:

- how a description is found;
- what it says about the machine as a whole;
- how CPUs and features gate its entries;
- what the code generator needs from it;
- what adding RISC-V took.

[isel.md](isel.md) describes the instruction entries themselves.

## Finding a description

`epitome-llc`, `epitome-as` and `epitome-objdump` take `--target`
and `--target-add`, and `epitome-llc` takes `--mcpu` and `--mattr` too:

| Option | Meaning |
| --- | --- |
| `--target=<path>` | a `.ll` file, or a directory whose `.ll` files are read in name order as one module |
| `--target=<name>` | `aarch64` (built in), or directory `<name>` in one of the directories of `EPITOME_TARGET_PATH` (colon-separated) |
| `--target-add=<path>` | more files (or directories) read after the description, as part of it; may be repeated |
| `--mcpu=<name>` | the CPU, as the description names it |
| `--mattr=+f,-g` | features turned on or off; may be repeated |

Without `--target`, `epitome-llc` looks at the module's target triple:

- `aarch64` and `arm64` use the built-in description;
- any other architecture name is looked up in `EPITOME_TARGET_PATH`, so
  `riscv64-unknown-linux-gnu` finds `<dir>/riscv64/`.

`epitome-objdump` finds the description for an ELF object by its
`e_machine`, among the built-in one and those in `EPITOME_TARGET_PATH`.

### Overlays

The files of a description are one module, so `--target-add` can extend a
description without copying it. A file can add:

- new instructions;
- recipes;
- a calling convention;
- CPUs (`!target.cpus`);
- relocations (`!target.relocs`).

Named metadata from the different files is joined, and of two equal
patterns the first one read wins. For example, to add a CPU without
Advanced SIMD to the built-in AArch64:

```llvm
; my-core.ll
!target.cpus = !{!9000}
!9000 = !{!"my-core", !"fp-armv8"}
```

```sh
epitome-llc --target-add=my-core.ll --mcpu=my-core sum.ll -o sum.s
```

The loop that is vectorised for `generic` (`v0.4s` and so on) is then
compiled lane by lane. One thing to watch: the module's own
`"target-features"` are applied after the CPU, as llc applies them. So
clang's IR, which says `+neon`, turns NEON back on. `--mattr=-neon`, which
comes last, is what turns it off for such IR.

## The machine as a whole

`codegen/src/machine.rs` reads four lists of named metadata. AArch64's are
in `targets/aarch64/01-target.ll`, and RISC-V's in
`targets/riscv64/00-base.ll`.

### Registers

`!target.registers` gives each register class the names of its registers,
in number order:

```llvm
!1000 = !{!"x", !"zero", !"ra", !"sp", !"gp", !"tp", !"t{0-2}", !"s{0-1}", !"a{0-7}", !"s{2-11}", !"t{3-6}"}
!1004 = !{!"d", !"ft{0-7}", !"fs{0-1}", !"fa{0-7}", !"fs{2-11}", !"ft{8-11}"}
```

`{0-30}` is a range. `!target.classes` says what each class is:

```llvm
!1010 = !{!"x", !"file", !"x", !"bits", !"64", !"holds", !"int"}
!1012 = !{!"xsp", !"file", !"x", !"bits", !"64", !"holds", !"int", !"sp", !"yes"}
!1014 = !{!"d", !"file", !"f", !"bits", !"64", !"holds", !"float"}
```

| Property | Meaning |
| --- | --- |
| `file` | the register file its registers are in; the classes of one file name the same registers (`w3` is `x3`, `s3` is `d3`) |
| `bits` | how wide a value it holds |
| `holds` | `int`, `float` or `vector`; without it, the class is a *view* that only names registers for templates (`{n:b}` is `b3`; `lanes` gives the `v3` of `{n:4s}`, `v3.4s`) |
| `sp` | `yes` if the number it shares with the zero register is the stack pointer in this class (AArch64's `xsp`), so the zero register cannot be named in it |

Inside the code generator, registers are numbered file after file, in the
order the files are first named. AArch64's integer registers are 0–31 and
its vector registers 32–63. A machine with 16 integer registers and three
other files is numbered the same way, up to 255 registers in all. A class
is an index into this list, and the Rust names none.

Of the classes that hold a kind of value, the narrowest wide enough is
where such values are put, and the first declared wins between equals.
So an `i32` goes in `w` and a `<2 x i64>` in `q`. Everything else is
derived from the class and its file:

- which registers a value may be allocated to;
- which instruction copies it (its class's `"copy"`, or that of a wider
  class of its file: AArch64 copies 64-bit vectors as `d` registers);
- how it is moved between files (role `move.<to>.<from>`, for example
  `move.d.x`);
- how it is spilled, saved and restored (`load.<class>`, `store.<class>`).
  A function saves a register as its file's widest scalar class (`x`,
  `d`), and a value wider than that, such as a `q`, is not kept in a
  saved register across a call.

Templates print the names, and the assembler and the conventions' lists
read them. A template writes a register as another class names it with
`{n:w}`, `{n:s}`, `{n:b}`.

### Parameters

`!target.params` holds:

| Parameter | AArch64 | RISC-V | Meaning |
| --- | --- | --- | --- |
| `sp`, `zero` | `sp`, `xzr` | `sp`, `zero` | the stack pointer and the zero register |
| `vector-bits` | `128`, `neon` | `0` | a vector register's width, and the feature that brings the vector registers |
| `narrow-results` | `zero` | `sign` | how a 32-bit result sits in a 64-bit register |
| `external-via-got` | `yes` | `no` | whether symbols defined elsewhere are reached through the GOT |
| `boolean-branches` | `bit` | `compare` | whether a branch can test a boolean's low bit, or must compare whole registers |
| `unaligned-access` | `fast` | (`no`) | whether loads and stores need no alignment, at no cost, so small copies may use wide ones |
| `flags` | `nzcv` | (none) | the flags a comparison sets and a condition reads, where an entry has no `"sets"` or `"reads"` |
| `elf-machine`, `elf-flags`, `macho-cpu` | 183, 0, `0x0100000c` | 243, 4 | the object files' identity |
| `cpu` | `generic` | `generic-rv64` | the CPU when none is named |

`narrow-results` decides which casts are free:

- **`zero`:** zero extension from 32 bits, and truncation to 32 bits, cost
  nothing.
- **`sign`:** sign extension costs nothing, but truncation needs
  `sext.w`.

`boolean-branches = compare` makes `prepare` rewrite a branch on a boolean
that isn't a comparison into a comparison with zero.

### Operands written by name

Some operands are neither a register of a file nor a number: R600's
source operand is one of 128 registers, or a constant from one of four
banks, or an inline constant, each with a channel (`T5.Y`, `KC0[2].X`,
`0.5`, `PV.W`). `!target.enums` names such sets of values, and an operand
with `"enum"="src"` is one of them, written by name:

```llvm
!1200 = !{!"src", !"0", !"T{0-127}.{X,Y,Z,W}", !"512", !"KC0[{0-31}].{X,Y,Z,W}", !"992", !"0.0", ...}
!1202 = !{!"neg", !"0", !"", !"1", !"-"}
```

Each pattern's names take values in order from the number before it;
braces are a range or alternatives, the leftmost varying slowest. So
`T1.Y` is 5, and the encoding takes it apart with expressions
(`sel0:9=s0>>2`, `c0:2=s0&3`). A name may be empty: `{n0}{s0}` reads
`-T1.Y` and `T1.Y` alike. The assembler tries an enum's names in order
until the rest of the line matches, so enum operands may be side by
side, and `-1` (an inline constant) is not read as `-` and `1`. Where two
names have one value, the first is how it is written.

### CPUs

`!target.cpus` lists CPUs by LLVM's names for them, each followed by its
features:

```llvm
!1150 = !{!"generic", !"fp-armv8", !"neon"}
!1151 = !{!"apple-m1", !"fp-armv8", !"neon", !"lse"}
```

### Relocations

`!target.relocs` lists every relocation as property pairs:

```llvm
!1201 = !{!"name", !"adrp", !"elf", !"275", !"elf-name", !"R_AARCH64_ADR_PREL_PG_HI21",
          !"macho", !"3", !"macho-name", !"ARM64_RELOC_PAGE21", !"macho-pcrel", !"yes",
          !"macho-length", !"2", !"elf-syntax", !"{s}", !"macho-syntax", !"{s}@PAGE",
          !"target", !"(pc & -4096) + (v << 12)"}
```

| Property | Meaning |
| --- | --- |
| `name` | what an instruction's `"reloc"` attribute names |
| `elf`, `elf-name` | the ELF number and name |
| `macho`, `macho-name`, `macho-pcrel`, `macho-length` | Mach-O's number, name, PC-relative flag and log2 size |
| `elf-syntax`, `macho-syntax` | how assembly writes the operand, with `{s}` for the symbol (`%lo({s})`, `{s}@PAGEOFF`) |
| `use` | `call` (resolved in place within a section), `got` (through the GOT), `data8` (a pointer in data), `eh-pcrel`, `eh-sub` and `eh-unsigned` (unwind tables' addresses), or `got-pcrel32` (a 4-byte PC-relative reference to a GOT entry: personality routines, types caught) |
| `target` | for the disassembler: the address reached, from the instruction's address `pc` and the operand's value `v` |

Code uses `"reloc"` attributes. The object writers use the numbers and
`use`. The assembly printer and the assembler use the syntax, as a template
and in reverse. The objdump reader uses the names and `target`.

Relocations that have only a number and a name aren't used by any
instruction. They're listed so that other compilers' objects can be read.
Inside the code generator a relocation is an index into this list
(`RelocKind`), so no relocation is named in Rust.

## Features

An entry needs features through `"feature"="zbb"` (comma-separated when it
needs several). AArch64's generated vector entries all carry
`"feature"="neon"`.

`codegen/src/features.rs` works out each function's features, in this
order:

1. The CPU's features. The CPU is the one `--mcpu` names; failing that,
   the IR's `"target-cpu"` if the description knows it; failing that, the
   default `cpu`.
2. Then the IR's `"target-features"` (`+neon,-lse`) are applied.
3. Then `--mattr` is applied.

Features the description never mentions are ignored, which is why the long
`target-features` strings clang writes cause no trouble. A description that
names no CPUs starts with every feature on.

When a feature is off:

- Patterns that would emit an instruction needing it are skipped, and so
  are constants built with such an instruction.
- Recipes that use such an instruction are skipped, so a cheaper or a
  more general route is chosen instead.
- If it is the vector feature, `vector-bits` counts as 0. Legalisation then
  splits every vector into scalars, partial reductions included.

So RISC-V without Zbb computes `smax` with a branch-free recipe, and with
`+zbb` it uses `max`. Intrinsics with no instruction that is on are
expanded generically in IR by `prepare`:

- **Funnel shifts and rotations:** two shifts and an or. With a variable
  amount, one of the shifts is done in two steps, so it is never by the
  full width.
- **Bit counts and byte swaps:** done at a wider width the description has
  an instruction for. For example, RISC-V's `llvm.bswap.i32` becomes Zbb's
  64-bit `rev8` and a shift. With no such width, they are done with shifts
  and masks. A population count adds up its bytes with a multiply, or with
  shifts when the machine has no multiply.

## Inline assembly

LLVM's Language Reference documents inline assembly's constraints and
template modifiers, for each target. The generic parts are the code
generator's:

- `=` (an output), `&` (early clobber), `*` (indirect: memory through a
  pointer);
- a digit (an input tied to that output);
- `{reg}` (that register, by any name the description gives it:
  RISC-V's `{x10}` is `a0`);
- `~{reg}` and `~{memory}` (clobbers);
- `i` and `n` (immediates);
- the modifiers `c`, `n` and `z`, and `$$`, `${:uid}`, `${:comment}` and
  `${:private}`.

The rest is the description's, in three lists:

```llvm
!target.asm-constraints = !{...}
!{!"r", !"file", !"r", !"print", !"x"}    ; a register of file r, written as x names it
!{!"I", !"imm", !"0", !"4095"}             ; an immediate in this range
!{!"m", !"mem", !"[{r}]"}                  ; memory at a register, written so
!{!"Z", !"zero"}                           ; the constant zero, as the zero register
!target.asm-modifiers = !{...}
!{!"w", !"class", !"w"}                    ; ${0:w}: the register as class w names it
!{!"i", !"if-imm", !"i"}                   ; ${2:i}: this text if the operand is an immediate
!target.asm-aliases = !{...}               ; spellings the assembler reads (objects.md)
```

| | AArch64 | RISC-V |
| --- | --- | --- |
| Registers | `r` (written `x3`), `w` (written `v3`) | `r` (written `a0`), `f` (written `fa0`) |
| Immediates | `I` (0 to 4095), `J` (-4095 to 0) | `I` (12 bits, signed), `K` (5 bits), `J` (zero) |
| Memory | `m`, `Q`: `[x0]` | `m`: `0(a0)`; `A`: `(a0)` |
| Zero register | `Z` | `J` with `${n:z}` |
| Modifiers | `w`, `x`, `b`, `h`, `s`, `d`, `q` | `i`, and the generic `z` |

How each part is compiled:

- **Selection:** a block becomes one instruction of its own, with its
  outputs first and its inputs after. A value whose constraint names a
  register is copied in or out. A tied input is copied into its output's
  register. A clobbered register is written by the instruction.
- **Register allocation:** an early-clobber output is taken to be written
  while the inputs are read, so it never shares their registers. The frame
  saves a saved register the block clobbers.
- **Output:** the template is written with its operands, between
  `APP` and `NO_APP` comments, as clang writes them. For branch distances,
  each statement counts as one instruction.

`tests/inline_asm.rs` builds about 20 functions at `-O0` and `-O2` and
runs them on this machine. Clang must assemble our assembly to the same
code. `tests/riscv.rs` runs the same kinds, written as IR, on the RV64
machine.

Not yet:

- `asm goto` (`callbr`) and `module asm`;
- constraints for part of a file (AArch64's `x`, `v0`–`v15`), and
  alternatives (`r|m`) beyond the first;
- instructions the description lacks: a block using `dmb` or a CSR
  instruction fails to assemble, with an error naming the line.

## What the code generator asks of a description

The Rust code refers to no instruction by name. Where it must emit an
instruction that no pattern selected, such as a spill, a frame adjustment,
a call or a jump, it asks for one by **role**. An instruction can have
several roles: `"role"="addi,addi12"`.

| Roles | Used for | Needed |
| --- | --- | --- |
| `load.<class>`, `store.<class>`, and `load.b/h`, `store.b/h` | spills, saves, arguments on the stack | for each file's widest scalar class (`x`, `d`), for classes wider than that (`q`), and for the sizes of arguments passed |
| `addi`, `subi`, `addi.neg`, `addi12`, `subi12`, `addi.w`, `add`, `shli` | frame adjustment, addresses of stack slots, building large constants | `addi` and `add` |
| `call`, `callr`, `jump`, `ret`, `trap` | calls, branches, returns, `unreachable` | yes |
| `addr.hi`, `addr.lo`, `got.hi`, `got.lo` | the address of a global: the two halves of its address, and of its GOT entry's | `addr.*`; `got.*` with `external-via-got` |
| `movz.*`, `movk.*`, `movi.zero` | AArch64's constant building | no: otherwise `lui`/`addi`/`shli` pieces are found from the constants the description allows |
| `zext.1/8/16`, `sext.1/8/16` | extending `zeroext`/`signext` arguments and results | for the widths a program passes so |
| `move.<to>.<from>` (`move.d.x`) | copies between register files: floating-point constants, bit-casts | for each floating-point class, from the integer class of its width |
| `dup.*`, `ins.*` | building vectors | where the machine has vectors |
| `load.pair.<class>`, `store.pair.<class>` | saving two registers at once | no |

The calling convention adds a few registers, and the stack's alignment:

- `stack-align`: the stack pointer's alignment at calls, in bytes (16 for
  all three conventions). Frames and the stack arguments' area are rounded
  to it;
- `scratch` and `fp-scratch`: for spill code;
- `far-scratch`: for the address of a stack slot further away than an
  instruction's offset can reach (beyond 2,047 bytes on RISC-V). Without
  one, a spill scratch register that is free at that instruction is used
  (AArch64, whose offsets reach 4KB for bytes and 32KB for doublewords).
  The address is made with the target's `far-add` instruction where it
  has one, an add that may take the stack pointer, else `add`.

## Runtime calls

What a machine has no instructions for, it may do by calling its runtime:
the description says which runtime functions do what, and how a value no
register holds is held. AArch64's (`65-runtime.ll`) does quad precision
(`fp128`) so; a description without such entries cannot compile it, and
says so.

- **`"libcall"="f"`**: a function whose body is one instruction of its
  parameters (and constants), as an instruction's is, done by calling
  runtime function `f` (declared in the description) on the parameters in
  order:

  ```llvm
  declare fp128 @__addtf3(fp128, fp128)

  define fp128 @fadd.f128(fp128 %a, fp128 %b) "libcall"="__addtf3" {
    %r = fadd fp128 %a, %b
    ret fp128 %r
  }
  ```

- **`"lower"="g"`**: the same, but the instruction becomes the body of
  description function `g` (of the same parameters, one block), which may
  call the runtime and do other IR: a comparison as a call and a test of
  its result (`fcmp olt` is `__lttf2(a, b) < 0`), a negation as an
  exclusive or of the sign bit. A body may use arithmetic, comparisons,
  casts, selects and calls.

- **`!target.held`**: pairs of types: the type no register holds, and the
  type it is held as once only moves of it are left (loads, stores,
  selects, phis, bit-casts, calls' arguments and results, constants). On
  AArch64 `fp128` is `<2 x i64>`, in a q register, where AAPCS64 passes
  it; a convention passing it in two integer registers (RISC-V's) would
  hold it as `i128`. A constant is held as its bits, from the low end.

An entry is used only where its `"feature"`s are on and its `"format"`
(`macho`, `elf`), if it has one, is the convention's: Apple's C library
has no quad-precision `sqrtl`, so `llvm.sqrt.f128` calls it only on ELF.
Whatever of a held type is left that is not a move is an error naming it
(`llvm.sqrt.f128 of fp128: no instruction does it, and the description
names no runtime function for it here`), as it would otherwise be taken
for an operation on the holding type. The runtime is the platform's
(libgcc's or compiler-rt's names; Rust's `compiler_builtins` has them on
Apple's systems, whose own library lacks most).

## What is still built in

The Rust names no instruction, register, class, relocation or CPU. What it
still assumes about a machine:

- **Instructions are 32-bit words, little-endian.** That rules out x86,
  RISC-V's C extension and Thumb.
- **Pointers are 64 bits.** That rules out RV32 and Arm32.
- **Object files are ELF64 and Mach-O.**
- **The vocabularies are fixed:** of conventions (`args`, `saved`,
  `stack-args`, ...), parameters and roles. A convention passes integers
  and floating-point values in two lists of registers. A machine needing
  something they cannot say (delay slots, register windows) needs Rust. So
  does AAPCS64's `va_list` (`va-list` = `aapcs64` only turns it on), and
  the even-register rule for `i128` (`wide-pairs` = `even`).
- **Flags are a lazy comparison** (`%nzcv`, the two values last compared),
  which suits AArch64's flags and machines without them alike.
- **Some limits are found from the description, not named in it.** How
  far a frame may be is what the save instructions' offsets reach (a frame
  beyond that is made in two steps, the saves near `sp`). A large
  adjustment of `sp` is split where the shifted add (`addi12`) takes its
  smallest step. Constants are built a chunk at a time, the chunk being the
  widest immediate `movz.x` holds unshifted.
- **No state between instructions** beyond registers and flags, such as
  RVV's vector length.

## Checking a description

`epitome-llc --check-target` checks a description, with the same
`--target` and `--target-add` options, and exits with failure if it finds
errors (`--verbose` lists every message):

```
$ epitome-llc --check-target --target=codegen/targets/riscv64
208 instructions, 0 condition forms, 300 patterns, 1 convention (lp64d), 1 CPU, 13 relocations
warn structure  33 requirements
       warning: feature zbb is needed by entries, but no CPU has it (only --mattr=+zbb turns it on)
ok   encodings  4863 words encoded, decoded and disassembled
ok   assembly   622 instructions written and read back
ok   recipes    54 recipes evaluated on edge values
ok   smoke      5 compilations of the smoke program
       note: without m, 10 of the 38 smoke functions cannot be compiled: arith32, ...
0 errors, 1 warnings
```

It takes well under a second. `codegen/src/check/` has the checks:

| Check | Finds |
| --- | --- |
| **structure** | Missing roles, copies or register names. A `"sym"` operand without a `"reloc"`. A relocation with no number in a format a convention writes. A convention whose registers contradict each other (saved and clobbered, scratch and argument). A default CPU that isn't listed. |
| **encodings** | Each instruction, over a spread of operands, must encode, decode back to the same operands, and be read by the disassembler as an entry that encodes the same word. It is an error for two entries to share words when neither accepts fewer words than the other, unless both write each word alike. The allowed cases are an alias like `cmp` of `subs`, and one instruction described once per IR type. |
| **assembly** | Each template must write every operand the encoding holds, and the assembler must read the written instruction back as the same word. Two entries that read back as one another are reported as a warning. |
| **recipes** | Each recipe's instructions, by their bodies, must compute what the IR it replaces computes. This is tested on edge values, including NaN, -0.0, the extremes and the sign boundaries. |
| **smoke** | The smoke program (`smoke.c`, about 40 functions: every width, division, constants, loops, switches, globals, many arguments, a variadic call, a 12KB frame, register pressure, floating point, vectorised loops, structs, traps) must compile under every convention and CPU. The object written must equal what the assembler makes of the assembly, and every word must disassemble. It also reports which functions each optional feature is needed for. |

It cannot show that an instruction's body says what the hardware does, or
that a template is written as other assemblers write it. A template whose
operands are in the wrong order still reads back as itself. Only comparing
with other tools, or running code, can show those. `tests/riscv.rs` and
`tests/programs.rs` run code. `codegen/tests/describe.rs` checks both
descriptions, and checks that the checks find the mistakes they are for:
it breaks the RISC-V description in six ways, one at a time.

## Descriptions for assembly only

An entry can be marked `"asmonly"`. It has an encoding and a template, and
an empty body. The assembler reads it and the disassembler writes it, but
selection never uses it. That makes it cheap to cover a whole instruction
set from a machine-readable source, and to check the entries against
another assembler rather than write their semantics. AArch64's `80-asm.ll`
is made this way ([objects.md](objects.md)). So is RISC-V's: `gen_opcodes.py`
writes 760 entries from riscv-opcodes (RISC-V International, BSD 3-clause).
That source gives each instruction's fixed bits and the fields of its
operands, but neither its assembly syntax nor which of its registers are
integer and which floating-point. So:

- operands are written in RISC-V's order (`rd, rs1, rs2, rs3`; loads
  `rd, imm(rs1)`; stores `rs2, imm(rs1)`; branches `rs1, rs2, label`;
  atomics `rd, rs2, (rs1)`);
- immediates are laid out as the ISA manual lays them out (the I, S, B, J
  and U forms);
- the register files, and the bits of rounding modes (`rtz`) and orderings
  (`.aqrl`), are found with LLVM's RISC-V assembler. rustc has it, in its
  `riscv64gc-unknown-none-elf` target, which assembles a crate's
  `global_asm!`.

With it, the description has 300 of the 329 RV64 instructions of I, M, A,
F, D, Zicsr, Zifencei, the Zb* and Zk* extensions, Zfh, Zfa, Zacas, Zabha,
Zicbo and others. The rest: Zfa's `fli` (a table of constants), Zalasr
(which this LLVM rejects), Zimop's numbered operations, and a few
privileged ones. `tests/assemblers.rs` assembles RISC-V's 2,900 sample
lines with `epitome-as` and with rustc, and they must agree word for
word. That check found that our own `fcvt.d.w`, `fcvt.d.wu` and `fcvt.d.s`
wrote a rounding mode other than LLVM's (they are exact, so it never
mattered).

The checks apply to these entries as to any other: encodings, assembly
round trips, and ambiguity. A register the encoding cannot hold (an odd
one, for a pair; `v16` in a 4-bit field) does not fit, as an immediate out
of range does not.

## RISC-V RV64

`targets/riscv64/` is written by `gen_riscv64.py`, from the RISC-V
Instruction Set Manual and the ELF psABI. It has 208 instructions in these
files:

| File | Contents |
| --- | --- |
| `00-base.ll` | registers, parameters, CPUs, relocations |
| `10-integer.ll` | RV64I |
| `20-m.ll` | M |
| `30-zbb.ll` | Zbb, behind `"feature"="zbb"` |
| `40-float.ll` | F and D |
| `60-recipes.ll` | 54 recipes: comparisons into registers, selects, minimum and maximum, extensions, `abs`, floating-point predicates |
| `70-abi.ll` | the LP64D convention |

Globals use the medlow model (`lui %hi`, `addi %lo`) and no GOT.

RISC-V showed where the Rust still assumed AArch64. Each of these is now
general, or is driven by a parameter:

- **Operands split across fields.** B- and J-type immediates are scattered
  across the word. Encoding concatenates slices (`imm[12|10:5]`), and
  decoding now joins them back. Fields that repeat the same bits, such as
  the two halves of `fmv.d`'s `rs1 = rs2`, are kept apart rather than
  summed.
- **Sign-extended 32-bit values** (`narrow-results`).
- **Branches that compare registers** (`boolean-branches`).
- **Large constants.** Built from the constants the description allows,
  as `lui` + `addi` + `slli` pieces, instead of AArch64's `movz`/`movk`.
- **Short offsets.**
  - Frames larger than an `addi` can reach are allocated in two steps.
  - Slots beyond 2,047 bytes are reached through `far-scratch`.
  - Branches out of range are relaxed by inserting a `.far` block that
    holds a `jal`.
- **No vector registers.** Every vector is done lane by lane.

`codegen/tests/riscv.rs` tests it without a RISC-V toolchain:

- **Compiling.** `basic`, `bits`, `divs`, `loops`, `calls`, `floats` and
  `vectors` from `tests/programs/` are built from clang's IR (LP64,
  unsigned `char`). There are four configurations: `-O2` and vectorised
  `-O2`, each with and without `+zbb`. `bits` is also built with `-m`.
- **Round trip.** `epitome-as` must make the same bytes from our
  assembly, and `epitome-objdump` must read it back.
- **Running.** The objects are linked and run on a small RV64IMFD+Zbb
  emulator (`tests/riscv/emu.rs`). Its static linker resolves the four
  relocations, and its C library provides a `printf` that prints as
  glibc's does. Each program must print exactly what the native build
  prints.

All 30 runs pass. The test was also what found the bugs listed above.

Not yet:

- compressed instructions (C) and vectors (V);
- thread-local variables;
- position-independent code and the GOT;
- `varargs.c`, whose IR's `va_list` is AArch64's because clang here has
  no RISC-V target.

Adding V would mean:

1. a third register file (`v0`–`v31`), with its classes;
2. `vector-bits` with the feature `v`;
3. vector entries tagged `"feature"="v"`.

The files and classes are now data, but V's vector length is set at run
time (`vsetvli`). That is state the code generator would have to track
between instructions, and it has nothing for that yet.

## AMD R600 (Evergreen)

`codegen/targets/r600` describes AMD's Evergreen shader cores (Radeon HD
5000 and 6000) for assembly and disassembly. `gen_r600.py` writes it from
the microcode formats of AMD's "Evergreen Family Instruction Set
Architecture" (2011). The assembly syntax is what LLVM's r600 back end
writes, since that is the only assembly for it there is, and
`tests/r600.rs` compares each line `llc -show-mc-encoding` writes, as we
assemble it, with the bytes llc gives for it.

It is a partial machine ([partial-machines.md](partial-machines.md)):
`!target.lacks` says it has no stack, calls, globals, indirect branches or
traps. Its instructions are 64 bits (`instruction-unit` 8), and a vertex
fetch is 128:

- **ALU instructions** (`ALU_WORD0` with `ALU_WORD1_OP2` or `_OP3`): one
  entry per opcode and absolute-value combination. Sources, the
  destination, negation, the end of a group (`* `), predication
  (`Pred_sel_zero`) and the bank swizzle (`BS:VEC_021/SCL_122`) are
  enum operands. A group's literal constants follow it, two to a word,
  written `LITERALS 2, 0` (llc writes `2(2.802597e-45), 0(0.000000e+00)`,
  which the test reads so);
- **control flow** (`CF_WORD0/1`, `CF_ALU_WORD0/1`): clauses (`ALU 36, @4,
  KC0[CB0:0-32], KC1[]`, `TEX 0 @14`), jumps and pops (`JUMP @6 POP:1`),
  the hardware loops (`LOOP_START_DX10`, `END_LOOP`, `LOOP_BREAK`),
  `CF_END` and `PAD`;
- **a memory export** (`MEM_RAT_CACHELESS STORE_RAW`) and **a 32-bit vertex
  fetch** (`VTX_READ_32`), in the forms llc writes.

Where llc's names differ from AMD's (`LSHR` for `LSHR_INT`, `MULHI` for
`MULHI_UINT`, `END_LOOP` for `LOOP_END`), the entries take llc's for
control flow and AMD's for ALU instructions, with llc's as
`!target.asm-aliases`. AMD's table writes opcode 66 as `PREDE_INT`; it is
`PRED_SETE_INT`.

Not yet: the other memory exports, fetches other than `VTX_READ_32`,
`LDS_IDX_OP`, the extended ALU clause (`CF_ALU_WORD0_EXT`), and
instruction selection, which needs groups and clauses
([partial-machines.md](partial-machines.md), section 5).

## AMD gfx1151 (RDNA 3.5)

`codegen/targets/gfx1151` describes AMD's RDNA 3.5 GPUs (gfx1151, Strix
Halo's Radeon 8060S) for assembly and disassembly, in wave32.
`gen_gfx1151.py` writes it from AMD's machine-readable ISA specification of
RDNA 3.5 (`amdgpu_isa_rdna3_5.xml`, from AMD GPUOpen's "Machine-Readable GPU
ISA", MIT). That gives every encoding's fields and every instruction's
opcode in each encoding it has, with its operands' fields, types and sizes,
but no assembly syntax. The syntax is LLVM's (`llvm-mc
-triple=amdgcn-amd-amdhsa -mcpu=gfx1151`), and the generator finds it with
llvm-mc, as RISC-V's does with rustc:

- each operand's values are put in a word one at a time, the others held,
  and what llvm-mc's disassembler writes for each value is its name (so a
  source's 512 values are `s0` to `s105`, `vcc_lo`, `ttmp0`, `null`, `m0`,
  `exec_lo`, the inline constants `0` to `64`, `-1` to `-16`, `0.5` and
  the like, `src_scc`, and `v0` to `v255`; and `v0.l` and `v0.h` in a
  16-bit operand);
- where in the line those names are written makes the template;
- values llvm-mc writes but does not read back as the same word are left
  out: odd register pairs (`s5` read as `s[4:5]`), names its assembler does
  not take (`src_flat_scratch_base_lo`), a literal it writes `lit(...)`
  because an inline constant could hold it;
- each entry is then checked on random operand values: the line llvm-mc
  writes for each must be the one the template writes.

It writes 15,796 entries, with 79 enums. It leaves out MIMG's 168 forms and
255 others, which it lists: most are VOP3 forms with a literal and op_sel or
source modifiers, which llvm-mc writes in ways the generator does not yet
follow, or does not read back.

It is a partial machine ([partial-machines.md](partial-machines.md)), like
R600: `!target.lacks` says it has no stack, calls, globals, indirect
branches or traps. Its instructions are multiples of 32 bits
(`instruction-unit` 4), with a 32-bit literal after some.

An operand is one or more fields. A VOP3 source is its 9-bit field, its
negation and absolute-value bits (`-|v2|`, `neg(2)`) and its op_sel bit
(`v2.h`); a destination is its register and op_sel bit. The clamp and
output modifier are one operand (` clamp mul:2`), and so are VOP3P's
modifiers, and the cache flags (` glc slc dlc`). A 16-bit VOP3 operation
writes its op_sel bits twice, in its operands' halves and again after them
(`v_add_f16_e64 v1.l, v2.h, v3.l op_sel:[1,0,0]`); its entry writes the
second from the first, `op_sel:[{=(src0>>11)&1},...]`, a computed hole the
assembler checks against the operands it has read. Some fields have
entries of their own for each way of writing them:

- an offset, which is not written when 0 (`offset:16`), and wait counts,
  which are not written at their most (`s_waitcnt vmcnt(0) lgkmcnt(0)`);
- flags that change the operands as well as being written: `idxen` and
  `offen` (the address is `off` without them), `tfe` (one more data
  register), `sve`, and `glc` in atomics (which then return a value);
- a global access's address register, 64-bit without `saddr` and 32-bit
  with it;
- an immediate llvm-mc writes in decimal up to 64 and in hex above
  (`s_movk_i32 s0, 64`, `s_movk_i32 s0, 0x41`), and `hwreg(id)` beside
  `hwreg(id, offset, size)`.

Three ways of writing a line are read but not written (`"nodecode"`
entries), as clang writes them: a branch to a label (`s_cbranch_execz
.LBB0_2`; the field is the distance in words from the next instruction,
which llvm-mc writes as a number); a scalar load's zero offset written
`0x0`, where llvm-mc's disassembler writes `null`; and a 16-bit operation's
halves written only in its operands' names (`v_cmp_gt_f16_e64 s0, v2.h,
v3.h`), without the op_sel after them.

An enum is written once for the operands that share it: an operand whose
names are a larger table's, but for a few runs of values (a source without
`vcc_lo`, say), uses that table, with guards leaving the others out.

A VOPD (dual-issue) entry is an X and a Y operation together, for each
parity of the first destination: the second's low bit is not held, but is
the other of the first's. The register banks the two halves' sources must
be in are guards. So is the constant bus, in VOPD and VOP3 alike: at most
two scalar values (a register read twice is one, and the literal is one),
and one in the 64-bit shifts.

`tests/gfx1151.rs` assembles every entry's sample lines with `epitome-as`
and with llvm-mc, which must give the same bytes, and disassembles each of
llvm-mc's instructions, which must be read as llvm-mc's disassembler writes
it (34,597 lines; llvm-mc's disassembler crashes on a few words, which the
test lists and leaves out). It is skipped where there is no llvm-mc 22 or
later with the AMDGPU target (the syntax is LLVM 22's; LLVM 18's assembler
refuses some of it).

Not yet: images (MIMG), whose data and address registers' sizes depend on
`dmask`, `d16`, `tfe`, `dim` and `a16` together; `s_version`, which
llvm-mc writes but does not read back; buffer stores with `tfe`, which
llvm-mc writes but does not take; `ds_swizzle_b32`'s offset, which
llvm-mc writes as a pattern (`offset:swizzle(BITMASK_PERM,"ppppp")`), but
for 0; wave64; and instruction selection.
