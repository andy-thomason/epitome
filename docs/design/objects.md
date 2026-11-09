# Object files, assembly and disassembly

This document covers how Epitome writes object files (Mach-O for macOS,
ELF for Linux) instead of assembly; `epitome-as`, which assembles
assembly into them; and `epitome-objdump`, which reads object files (ELF
and Mach-O) and disassembles them with the machine description.
For the description itself and instruction selection, see
[isel.md](isel.md).

**Status:** ELF relocatable files and Mach-O object files: **Done**, both
checked against clang's assembler over SQLite. Assembling Epitome's own
assembly: **Done**. Assembling other compilers' assembly: **Partial** (see
below). Programs and SQLite built
from our own Mach-O objects, linked by the system linker, run and answer as
clang's builds do. Unwind tables (`.eh_frame`): **Done**. Exception
tables: **Done** for Mach-O, run; **Done** for ELF, checked against the
assemblers but not run (nothing here runs Linux programs). Thread-local
variables: **Done** (Mach-O's descriptors; ELF's by TLSDESC on AArch64,
local-exec offsets on RISC-V). Compact unwind and debug info: **Later**.

## Sources

| Document | Used for |
| --- | --- |
| System V Application Binary Interface, *Object Files* (the generic ELF chapters) | File and section headers, symbol tables, `SHT_RELA`, string tables |
| Arm, *ELF for the Arm 64-bit Architecture* (AAELF64) | `EM_AARCH64`, relocation numbers and what each fills in, mapping symbols `$x` and `$d` |
| Apple's published `<mach-o/loader.h>`, `<mach-o/nlist.h>`, `<mach-o/reloc.h>`, `<mach-o/arm64/reloc.h>`, `<mach-o/fat.h>` | Writing and reading Mach-O: load commands, sections, symbols, arm64 relocation types, universal files |
| Arm Architecture Reference Manual | The `adrp` immediate (`immhi:immlo`, signed, in pages); the shift-by-immediate encodings (`immh:immb`) |
| clang's assembler on our own assembly | What an assembler does where the documents leave a choice; see below |

## The object model

`codegen/src/object.rs` describes an object file without committing to a
format: sections (`Text`, `Data`, `Bss`, `ReadOnly`, `RelRo`) with their
bytes and alignment, symbols (function, object or undefined; local or
global; with section, value and size), and relocations (offset, kind,
target, addend). `elf.rs` writes it as ELF and `macho.rs` as Mach-O.

Globals are described once, as `GlobalData`: their section, alignment and a
list of items (integers of 1, 2, 4 or 8 bytes, strings, zeros, and
addresses of symbols with an offset). Both the assembly printer and the
object builder read that list, so the two outputs cannot disagree about
what a global holds. Zero-initialised writable data goes in `.bss` (ELF
only so far), where only its size is stored.

### Relocations are data

The description says how each instruction with a symbol operand is fixed
up, in a `"reloc"` attribute naming one of the relocations the description
lists in `!target.relocs`, with their ELF and Mach-O numbers and how
assembly writes them ([targets.md](targets.md)). AArch64's:

| Instruction | `"reloc"` | ELF | Mach-O |
| --- | --- | --- | --- |
| `bl sym` | `call26` | `R_AARCH64_CALL26` | `ARM64_RELOC_BRANCH26` |
| `adrp x, sym` | `adrp` | `R_AARCH64_ADR_PREL_PG_HI21` | `ARM64_RELOC_PAGE21` |
| `add x, x, :lo12:sym` | `add_lo12` | `R_AARCH64_ADD_ABS_LO12_NC` | `ARM64_RELOC_PAGEOFF12` |
| `adrp x, :got:sym` | `adrp_got` | `R_AARCH64_ADR_GOT_PAGE` | `ARM64_RELOC_GOT_LOAD_PAGE21` |
| `ldr x, [x, :got_lo12:sym]` | `ld64_got_lo12` | `R_AARCH64_LD64_GOT_LO12_NC` | `ARM64_RELOC_GOT_LOAD_PAGEOFF12` |
| `.quad sym+off` (data) | `abs64` | `R_AARCH64_ABS64` | `ARM64_RELOC_UNSIGNED` |

The code generator only ever reaches globals with `adrp` and `add` (then
loads through the register), so the load and store page-offset
relocations are not needed. Code never carries an addend: an offset from a
symbol is added by a separate instruction. Data does (`&table[3]`).

### What an assembler does, and so what we do

Where the documents leave a choice, Epitome does what clang's assembler
does with the same program, so that the two object files can be compared.
The two formats differ:

| | ELF | Mach-O |
| --- | --- | --- |
| A call to a function in the same section | resolved in place if the function is local; a global one keeps `R_AARCH64_CALL26`, since the linker may bind it to another library's definition | resolved in place, global or local: symbols are bound to the library that defines them |
| Other references to a local symbol | against its section, with the symbol's offset in the addend (`.data+0x28`); through the GOT, against the symbol | against the symbol |
| A pointer's addend | in the relocation (RELA); the bytes are zero | in the pointer's bytes |
| Relocation order | by address | from the highest address down |
| Extra symbols | mapping symbols: `$x` at the start of a code section, `$d` at the start of each data section, `.bss` included; a section symbol for each section relocations refer to | none (the assembler adds `ltmp0` temporaries at section starts, which nothing needs) |
| Other | an empty `.note.GNU-stack`: the stack need not be executable | `LC_BUILD_VERSION` with the least macOS version, from the triple |

References to global and undefined symbols are made against them in both,
and an instruction's relocated field is zero (code never has an addend).

The assembly gained what the object has, so that the assembler's object
says the same: for ELF, `.size` for each function and object, `.bss` for
zeros, and the `.note.GNU-stack` section; for Mach-O, `.build_version
macos, 26, 0` from the triple (else 11.0), without which the assembler
uses the version of the machine it runs on.

One encoding had to change: `mov w0, #0xffff0000` fits both `movn w0,
#0xffff` and `movz w0, #0xffff, lsl #16`, and an assembler prefers `movz`.
Where a constant's only nonzero 16 bits are above the lowest, the selector
now writes the shifted `movz` out, so that our encoding and the
assembler's are the same.

## Instructions for assembly only

Instruction selection needs an entry to say what the instruction does,
but the assembler and disassembler use only its encoding and template.
So most of the A64 instruction set is described for them alone:
`80-asm.ll` has 3,927 `"asmonly"` entries, written by `gen_mrs_asm.py`.
Selection ignores them, and their bodies are empty.

The generator has two sources:

- **Arm's machine-readable specification:** each instruction's bit layout,
  and its assembly syntax as a grammar. The grammar is expanded into
  concrete templates: register families become operands of our classes;
  vector arrangements, conditions and shift kinds become one entry each; a
  rule used twice chooses alike. Arm's aliases (`cinc`, `sbfx`, `negs`,
  `mvn`) are expanded too. They become `"nodecode"` entries, which the
  assembler reads, and the instruction lists its alias mnemonics in an
  `"aliases"` attribute.
- **clang's assembler, on lines we write:** the open-source specification
  does not say how an operand is held in its fields. For each template, the
  generator assembles a base line, then lines with one operand varied, and
  reads which bits change and how. It fits each operand to one of a few
  forms: the value, shifted (a scaled offset, a PC-relative label), `c - v`
  or `v + c` (shift encodings), a logical immediate (`bitmask64`), or,
  for a field of several operands, their sum or difference and a constant
  (`sbfx`'s `imms = lsb + width - 1`). A line clang encodes another way
  (`add #-1` as `sub #1`) is discarded. So is a template whose fit fails.

Templates an earlier file already covers are left out, and so are other
spellings of an encoding already written: the first spelling is the one
the disassembler writes, and the rest are `"nodecode"`.

With them, the description covers 1,933 of Arm's 4,429 A64 encodings: 294
of 301 base, 910 of 1,116 FP and SIMD, and 729 of 824 optional
extensions. SVE and SME are not covered. Not yet: register lists
(`ld1 {v0.16b, v1.16b}`), floating-point immediates (`fmov d0, #1.0`), and
system registers beyond the first 48 per instruction (the
`s3_0_c1_c2_3` form covers the rest).

`epitome-llc --sample-asm` writes sample lines of every instruction:
13,959 lines, with several operands per instruction. `tests/assemblers.rs`
assembles them with `epitome-as` and with clang, and the code must be
the same word for word. On clang's own builds of the test programs,
`tests/objdump.rs` now decodes about 99% of the words, up from about 90%,
and all as objdump reads them.

## Assembly as people write it

Inline assembly is written by people, not by our templates, so the
assembler also reads:

- **Numeric local labels:** `1:`, referred to as `1b` (the last before) and
  `1f` (the next after). Each definition is renamed to a temporary label of
  its own.
- **Comments** in the machine's syntax: the `asm-comment` parameter, `//`
  or `#`. On Mach-O, `;` also starts a comment; on ELF it separates
  statements.
- **Other spellings**, from the description's `!target.asm-aliases`. Each
  is a pattern and the way the templates write it. A line that does not
  assemble as written is tried rewritten by them. AArch64's are `42` for
  `#42` and `[x0]` for `[x0, #0]`; RISC-V's are `beqz`, `bnez`, `bltz`,
  `bgez`, `not`, and `(a0)` for `0(a0)`.

## `epitome-llc -o file.o`

A name ending in `.o`, or `--filetype=obj`, writes an object file instead
of assembly. A module with inline assembly, whose text only an assembler
reads, is written by assembling its assembly with `epitome-as`. Otherwise
the object is written directly: Mach-O under Apple's convention (the default, and a Darwin
triple), ELF under AAPCS64 (a Linux triple, `--abi=aapcs64`) or with
`--syntax=elf`.

```sh
epitome-llc file.ll -o file.o && clang file.o main.c -o program && ./program
```

A Mach-O object has one unnamed `LC_SEGMENT_64` holding every section,
laid out one after another from address 0 (`__TEXT,__text`,
`__DATA,__data`, `__TEXT,__const`, `__DATA,__const`); then
`LC_BUILD_VERSION`, `LC_SYMTAB` and `LC_DYSYMTAB`. Its symbol table has the
local symbols, then the defined external ones by name, then the undefined
ones by name, as `LC_DYSYMTAB` says. Relocations name their symbols
(`r_extern`); one with an addend in an instruction would be preceded by an
`ARM64_RELOC_ADDEND`, though the code generator makes none.

An ELF file has the null section; the object's sections in order (`.text`,
`.data`, `.bss`, `.rodata`, `.data.rel.ro`); a `.rela` section for each with
relocations; `.note.GNU-stack`; then `.symtab`, `.strtab` and `.shstrtab`.
Local symbols come first, as ELF requires: section symbols, mapping
symbols, then the object's own; then global ones, defined and undefined.

## Loading code into the process

`codegen/src/load.rs` runs compiled code in the compiler's own process, for
clients that compile and run (epitome-sv's netlist tests): no assembly, no
object file, no linker, no `dlopen`.

```rust
let opts = Options { no_asm: true, ..Default::default() };
let c = epitome_codegen::compile(&t, &mut module, &opts)?;
let code = load::load(&t, &c, load::process_symbol)?;
let f: extern "C" fn(i64) -> i64 = unsafe { code.function("fib") }.unwrap();
```

`Options::no_asm` leaves `Compiled::asm` empty (`Compiled::assembly` writes
it when it is wanted); `epitome-llc` sets it when writing an object.

`load` takes the abstract object (`Compiled::object`, or the assembler's for
inline assembly) and puts its code, read-only data and writable data on
pages of their own. It fills in each relocation as a linker would: a
pointer in data; a 32-bit distance in an instruction's own 32-bit field
(x86); or, for a field the description encodes in pieces (AArch64's
`adrp`, `add :lo12:`, `ldr :got_lo12:`, `bl`), by decoding the instruction
(`Reloc::instr` says which), setting its symbol operand and encoding it
again. The operand is found by inverting the relocation's `target`
(`pc + v`, `(pc & -4096) + (v << 12)`), or for a relocation with no target,
the address's low bits below the largest page any relocation measures in.
References through the GOT use a table built next to the read-only data.
Symbols the object does not define come from the caller's resolver
(`process_symbol` is `dlsym` of `RTLD_DEFAULT`); calls to them go through
16-byte stubs after the code (`ldr x16, #8; br x16` and `jmp *2(%rip)`),
since the target may be out of a call's reach.

On an AArch64 Mac the code pages are `MAP_JIT`, written with
`pthread_jit_write_protect_np(0)` and then `sys_icache_invalidate`d; the
data pages are ordinary memory mapped over the rest of the region. On
x86-64 Linux the whole region is mapped writable and the code then
`mprotect`ed to read and execute. Unwind and exception tables are not
loaded, and thread-local variables cannot be.

For CVA6's `id_stage` (9,000 lines of IR, 10 functions; the
`file_timing` test in `codegen/tests/load.rs`, with `EPITOME_LOAD_IR`):

| | M-series Mac | x86-64, AVX-512 (a shared, busy machine) |
| --- | --- | --- |
| Compiling, with the assembly | 230 ms | 169 ms |
| The assembly alone (left out by `no_asm`) | 6 ms | 0.7 ms |
| Building the object (encoding) | 4.9 ms | 7.5 ms |
| Loading that | 0.04 ms | 0.4 ms |
| Instead: writing it, `cc -shared`, `dlopen` | 250 ms | 52 ms |

The text is written in parallel and is little of the time; what the
loader saves is the file, the linker and `dlopen`.

## `epitome-as`

```sh
epitome-as file.s -o file.o        # Mach-O or ELF, as the assembly is written
epitome-as --format=elf file.s     # say which
```

The assembler (`codegen/src/assembler.rs`) reads instructions with the
machine description, run backwards: each entry's `"asm"` template says how
it is written. The templates whose first word is a line's mnemonic are
tried (or whose literal start is, for `b.{c}`). A template is literal text
and holes; the literal text must be in the line as it is, and each hole
takes what comes before the next piece of literal text. A hole's text is
then read loosely, as what its operand is: a register (`w3`, `sp`,
`v2.4s`), a number, a condition, a symbol (`_x@PAGEOFF`, `:lo12:x`), or a
label. A hole that is an expression (`#{=0-imm}`) is solved for its operand
with the same inversion the decoder uses.

A reading is kept only if the template, filled in with the values read,
gives back the line exactly, and if the values encode and decode back to
themselves (so an immediate the encoding cannot hold does not fit). What is
accepted is exactly what the printer writes. Where two entries both fit
(`mov x0, x1` is `orr` from `xzr`, and `add` to the stack pointer), the one
using fewer stack-pointer register classes wins, which is what the
architecture's alias rules choose; then the first in the description.

Assembly takes two passes: the first lays out sections, labels and data,
an instruction being 4 bytes; the second encodes, now that labels have
addresses. References to symbols are then resolved by the same code as
compiled output (`object::resolve`), so the object file follows each
format's rules. The directives are those the code generator writes:
sections, `.globl`, `.p2align`, `.type`, `.size`, `.byte`, `.short`,
`.long`, `.quad` (of numbers or `symbol+offset`), `.ascii`, `.asciz`,
`.space`, `.zero`, `.build_version`. Every instruction that cannot be read
is reported, with its line (`--max-errors` says how many).

It is fast: SQLite's 430,000 lines assemble in 2.2 seconds, where clang's
assembler takes 4.5.

### Other compilers' assembly

clang's own assembly for SQLite assembles about 81% of its instruction
lines once the things below are set aside. What it uses that ours does not:

- **Directives:** the `.cfi_*` directives we don't write
  (`.cfi_remember_state`, other encodings for `.cfi_personality`), `.loh` (linker optimisation hints, which can be
  ignored), `.zerofill`, literal and C-string sections (`__literal8`,
  `__cstring`), jump tables of label differences
  (`.byte (LBB29_26-LBB29_15)>>2`), and `.indirect_symbol` and
  `.data_region`.
- **Instructions the description lacks:** `ldp` and `stp` (a third of the
  rest), `ldur`/`stur`, pre-indexed addressing (`[x1, #16]!`), extended
  registers (`ldrb w0, [x1, w2, uxtw]`, `add x0, x1, w2, sxtw`), `ccmp`,
  `smaddl`/`umaddl`, `ld1` to a lane.
- **Spellings:** `mov` for a bitmask `orr`. (`[x1]` for `[x1, #0]` is now
  read, through the description's `!target.asm-aliases`.)
- **Symbols with addends in code** (`_x@PAGEOFF+128`), and page-offset
  relocations on loads and stores (`ldr x0, [x1, _x@PAGEOFF]`), which need
  the load and store page-offset relocations.

## `epitome-objdump`

```sh
epitome-objdump -d file.o        # disassemble (the default)
epitome-objdump -dr file.o       # with each instruction's relocation
epitome-objdump -h -t file.o     # section headers, symbols
epitome-objdump -s file.o        # contents in hex
epitome-objdump --canonical a.o  # sorted text, for diffing two objects
```

It reads ELF (relocatable or linked) and 64-bit Mach-O (objects and linked
images, and the arm64 slice of a universal file) into one form, an
`Image`: sections with addresses and bytes, symbols, and relocations
against names. Disassembly then works the same for both.

Each word is decoded as the description's entry whose literal bits it has
(see isel.md). Where a relocation fills the word in, the word is decoded as
the entry that carries that kind of relocation, and the symbol is printed in
its place, as assembly writes it in that format:

```
      30: 91000000     	add x0, x0, :lo12:counter
		0000000000000030:  R_AARCH64_ADD_ABS_LO12_NC	.data+0x28
```

(A reference against a section is printed with the symbol at that
address, if there is one.) In Mach-O, the same is `add x0, x0,
_counter@PAGEOFF`. Branch targets are printed with the symbol they are in,
`0x98 <use+0x90>`; `adrp` and `bl` in a linked image without relocations
are printed as the addresses they reach. Mapping symbols decide what is
code: words after a `$d` are printed as `.word`. So are words the
description does not have.

`--canonical` prints an ELF file as sorted text without offsets or order:
each section's type, flags, size and bytes, its relocations (with section
symbols named by their section), and the symbols other than section
symbols. Two files that say the same print the same.

### Decoding fixes

Reading code we did not generate found two kinds of wrong decode, both in
the description's encodings:

- **`adrp`** split its operand as `immlo:2=s%4` and `immhi:19=s/4`.
  Decoding used `immhi` alone and dropped `immlo`; and the immediate is
  signed (`immhi:immlo` is a signed count of pages), which `/` and `%`
  round the wrong way for. It is now `immlo:2=s&3` and `immhi:s19=s>>2`,
  and the decoder puts back the low bits of an operand split over a
  `name>>k` field and a `name&m` field (or `name/k` and `name%k`).
- **Shifts by immediate** (`shl`, `ushr`, `sshr`) had the whole of
  `immh:immb` as a field holding `2*esize - shift`, so a word with `immh`
  zero, which is `movi` or `mvni`, decoded as a shift by a too-large amount.
  The element size is now literal bits (`immh`'s leading one) and only the
  shift is a field.

## Checks

- `codegen/tests/programs.rs`, for each test program in each of its four
  builds (step 8 is for both formats):
  - **3b.** clang's assembly of our Mach-O assembly, read back by
    `epitome-objdump`, gives our assembly: every mnemonic (or an alias),
    and every instruction a relocation fills in, whole.
  - **5.** Our ELF object for the Linux build says what clang's assembler
    makes of our Linux assembly, compared in canonical form: the same
    sections with the same bytes, the same relocations against the same
    targets with the same addends, and the same symbols with the same
    bindings, types, values and sizes.
  - **6.** Our ELF object, read back by `epitome-objdump`, gives our
    assembly.
  - **7.** Our Mach-O object says what clang's assembler makes of our
    assembly, and linked with the main by the system linker, the program
    prints what clang's build prints.
  - **8.** `epitome-as` makes, of our assembly, byte for byte the object
    `epitome-llc` writes directly. So three ways to the object file
    agree: clang's assembler, ours, and none.
- `codegen/tests/sqlite.rs`: SQLite's three builds (clang `-O2` IR with
  and without vectorising, and `-O0` IR through `epitome-opt`), each
  linked from our assembly and from our own Mach-O object, run the SQL
  workloads and must answer as clang's build does; each Mach-O object must
  say what the assembler's says, and be byte for byte what `epitome-as`
  makes of the assembly (as must the ELF objects). The same as 5 and 6 for ELF, from the
  `-O2` IR with and without vectorising: about 357,000 and 375,000
  instructions, with every relocation and symbol.
- `codegen/tests/objdump.rs`: clang's own `-O2` builds of the test programs
  and of SQLite (237,143 words), disassembled by `epitome-objdump` and by
  the system's objdump. Every word we decode must be, to objdump, the same
  instruction (mnemonic, or an alias). About 89% of SQLite's words decode;
  the rest are instructions the description does not have yet (`ldp`,
  `stp`, and others).

What is not checked here: linking and running the ELF objects, which needs
Linux. The Mach-O objects are linked and run, and the ELF writer shares
the object model with them, but an arm64 Linux machine (or a CI runner)
linking the ELF objects with glibc would be the end-to-end proof for ELF.

## Unwind tables

Each function's prologue carries its unwind rules as `Opcode::Cfi`
pseudo-instructions, which take no space (`unwind.rs`):

- after each move of `sp`, where the CFA (`sp` on entry) is;
- after the saves, where each register is saved, from the CFA;
- where `sp` moves, the CFA as the base register plus the frame size.

Assembly writes them as `.cfi_startproc`, `.cfi_def_cfa_offset`,
`.cfi_def_cfa`, `.cfi_offset` and `.cfi_endproc`, and `epitome-as`
reads those back.

The object writer and the assembler both make the same tables of them:
one CIE, and one FDE a function, in DWARF's call frame format. The forms
are the platforms' assemblers', observed with clang and rustc:

- augmentation `zR`, code alignment 1;
- the data alignment the description gives for the object format
  (`cfi-data-align`: AArch64's is −4 in ELF and −8 in Mach-O), and the
  link register as the return address column (DWARF's register numbers are description data,
  `!target.dwarf`);
- the CFA starting at `sp`;
- entries padded with `DW_CFA_nop` to 4 bytes, and the last to 8.

A function's address is PC-relative. Under ELF it is 4 bytes, with the
relocation whose `use` is `eh-pcrel` (`R_AARCH64_PREL32`,
`R_RISCV_32_PCREL`). Under Mach-O it is 8 bytes: the function less a
symbol at the table's start, a pair of relocations (`eh-sub`, then
`eh-unsigned`) on a field holding minus its own offset.

The checks:

- **AArch64 ELF:** the tables are clang's, relocations and all, for
  every program `tests/programs.rs` builds for Linux.
- **RISC-V:** `.text` and `.eh_frame` are byte for byte what LLVM's
  assembler (rustc's) makes of our assembly (`tests/unwind.rs`). The
  `.text` comparison found `lui` written with a negative immediate, which
  LLVM's assembler rejects.
- **Mach-O:** clang's assembler makes compact unwind entries where it can,
  where we write `__eh_frame` always, and the linker takes either. So
  `tests/unwind.rs` runs it: a C++ exception thrown from a callback our
  functions call (recursing, saving registers, with a variable-length
  array) must reach `main`'s `catch`. Without the tables the C++ runtime
  calls `terminate`.

## Exception tables

A function with landing pads has a language-specific data area (LSDA) in
`__TEXT,__gcc_except_tab`, which its personality routine
(`___gxx_personality_v0`, `_rust_eh_personality`) reads while unwinding
(`eh.rs`). It is built after layout, from the call instructions' offsets.
The form is the Itanium ABI's, as clang and rustc write it:

- **The header:** the landing pads' base is omitted (it is the
  function's start). The type table's encoding is `0x9b` (indirect,
  PC-relative, signed, 4 bytes: through the GOT), and its end is a
  ULEB128. The call sites' encoding is ULEB128.
- **Call sites:** one for every call, so that unwinding through any of them
  is allowed. Each gives its offset, its length (4), its landing pad's
  offset (0 for none), and its action (0 to clean up only, else 1 plus the
  offset of its chain of action records).
- **Action records:** each is a filter and the offset to the next. A filter
  is a type's index in the type table, from 1 (catch it); 0 (a cleanup);
  or negative, for an exception filter (−1 less its list's offset).
- **The type table:** last type first, aligned to 4. Each entry is a
  GOT-relative reference to the type's information, or 0 to catch all.
  After it come the filters' lists of type indices, each ending in 0.

The unwind tables then name the routine and the LSDA. A CIE for each
personality has augmentation `zPLR`. The personality goes through its GOT
entry: 4 bytes, the relocation whose `use` is `got-pcrel32`
(`ARM64_RELOC_POINTER_TO_GOT`). The LSDA's encoding is `0x10` and the
FDEs' is `0x10`. A function's FDE adds its LSDA's address, 8 bytes, a pair
of relocations as its own address is.

Assembly writes `.cfi_personality 155, sym` and `.cfi_lsda 16, label`,
and the table as `.byte` lists, with `.long sym@GOT-label` for the types.
`epitome-as` reads all of these.

ELF has no 4-byte reference to a GOT entry that every linker takes in
data, so ELF's tables reach the personality and the types through words
of ours instead, as clang's do: `sym.DW.stub`, a local 8-byte word in
`.data` holding the symbol's address (`asm::eh_stubs`, named by
`Abi::eh_ref`). The encodings are the same, `0x9b`, with the relocation
whose `use` is `eh-pcrel` (`R_AARCH64_PREL32`, `R_RISCV_32_PCREL`) to
the word; the LSDA's is `0x1b`, 4 bytes, PC-relative to the table's
place. Assembly writes `.cfi_personality 155, sym.DW.stub`, `.cfi_lsda
27, label` and `.long sym.DW.stub-label`. Assemblers put each CIE before
its FDEs, the CIE without a personality first, and so do we. The objects
are checked against the assemblers (`tests/unwind.rs`): on AArch64 they
are what clang's assembler and ours make of our assembly; on RISC-V their
code, tables and words are LLVM's, byte for byte. Linking and running
them needs Linux. Two things were found by running
programs, not by comparing bytes:

- **Mach-O's 4-byte GOT reference** holds `0xc0000000` less its own offset
  in the section. That is what clang's assembler writes there, and the
  relocation gives the rest.
- **The LSDA's pair of relocations names a symbol at the LSDA** (we write
  `GCC_except_tableN`, as clang does). With a section symbol and the
  offset in the field, which is what clang's assembler makes of a
  temporary label, the linker gives the unwinder the section's start
  *less* the offset. Every table but the first is then wrong. Clang's own
  builds don't meet this because they use compact unwind entries.

The checks run programs. `tests/unwind.rs` covers C++ code of ours that
catches by type and all, runs destructors, rethrows, and nests `try`s,
driven by a clang-built `main`. `tests/rust.rs` covers whole Rust programs:
rustc writes their IR, and their crate's object is ours, linked against
rustc's standard library. Their panics unwind through drops, and
`catch_unwind`, `resume_unwind` and overflow checks all work. Each program
must print what rustc's build of it does, at `-C opt-level=0` and `2`.

## Not yet

- Compact unwind entries for Mach-O (smaller than DWARF's, and what Apple's
  assembler writes where a frame fits them), and debug info.
- `.subsections_via_symbols` (Mach-O): without it the linker keeps each
  section whole, so it cannot drop unused functions.
- Zero-filled sections in Mach-O (`__DATA,__bss`): zeros are stored.
- Weak and `linkonce` definitions (written as global), common symbols,
  `.tbss` (zero-initialised thread-local variables are stored as zeros in
  `.tdata`), and sections chosen by attribute.
- Disassembling instructions the description does not have (`ldp`, `stp`,
  pointer authentication, system instructions): they print as `.word`.
