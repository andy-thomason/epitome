# VLIW machines: bundles, repeat loops, rotating registers and streams

Epitome describes machines that issue one instruction at a time. This plan
covers machines that issue several at once, in a bundle, each to a unit of
its own, and some features such machines tend to have:

1. bundles: a fixed-size word with a header and one slot per unit, or a
   group of instructions whose last is marked;
2. scheduling into bundles, by unit and latency;
3. loops counted by hardware, over a range of bundles;
4. rotating registers, renamed by the hardware on each iteration of such a
   loop, for software pipelining;
5. channels: values that live in a FIFO between units, not in a register;
6. streams: one function compiled into several programs, for different
   units, one launching the others;
7. several machines in one object: streams whose units are different
   machines, each with a description of its own.

None of it names a machine. The public machines it is checked against are
Intel's Itanium (bundles with a template, rotating registers, `br.ctop`)
and AMD's TeraScale, R600 and Evergreen (groups, clauses, hardware loops;
[partial-machines.md](partial-machines.md), section 5). For sources and
status labels, see the [index](README.md).

## 1. Bundles

`!target.bundles` lists the bundle formats, as property pairs
(`codegen/src/bundle.rs`):

```llvm
!target.bundles = !{!20}
!20 = !{!"name", !"toy", !"section", !".text.toy", !"bytes", !"8",
        !"lanes", !"::", !"end", !"last=1",
        !"field", !"port", !"0:3 3:3", !"field", !"rot", !"6:1 7:1", !"field", !"len", !"8:3",
        !"field", !"rep", !"11:1", !"field", !"last", !"12:1",
        !"slot", !"16:16", !"X", !"slot", !"32:16", !"Y", !"slot", !"48:16", !"X,Y",
        !"prefix", !"rep {len} last", !"rep=1 last=1", !"prefix", !"rep {len}", !"rep=1"}
```

| Property | Meaning |
| --- | --- |
| `name` | the format's name |
| `section` | the sections whose code is in bundles of this format, by name; a name ending `*` is a prefix (`.text.b .b.text*`) |
| `bytes` | the bundle's size; bits no field or slot holds are zero |
| `field` | a header field, `lsb:width`; several places (`5:3 8:3`) make a family, `port[0]` and `port[1]` |
| `slot` | a slot, `lsb:width`, and the units whose entries may go in it |
| `prefix` | a way of writing header fields before a bundle's `{`: a template of fields, and fields it sets besides |
| `end` | fields the last bundle of a section sets (a program's end) |
| `lanes` | the text between an instruction line's lanes: `a :: b` is `a` in lane 0 and `b` in lane 1 |

Bits are numbered from bit 0 of byte 0. An entry goes in bundles when it
has `"unit"` (a list); `"lane"` says which lane it is written in (either,
when absent). Such an entry is assembled and disassembled only inside a
bundle, so it may share a mnemonic with an instruction of another stream.

### Header fields set by operands

An entry's `"header"` sets header fields from its operands:

```llvm
"header"="port[s>>4]=s&7; rot[s>>4]=1 if (s>>3)&1"
```

Each assignment is `field[index]=value`, with an optional `if guard`,
in the expressions encodings use. Every assignment to a field in a bundle
must give it the same value, and fields nothing sets are zero. This is how
a register file read through one port per bundle is described: the
operand is an enum whose values carry the file, the register and whether
it is the rotating one; the slot holds only which file
(`s:1=s>>4`), and the header holds the register. Two slots reading
different registers of one file in a bundle is an error, as the hardware
cannot do it.

The disassembler decodes the header and each slot that is not zero, by
the entries of the slot's units. An operand that the slot holds only
part of is found from its enum: of the values that encode the same slot,
the one whose header assignments agree with the header and explain most
of it.

### Assembly

A bundle is written between `{` and `}`, its prefix (if any) before the
`{`, one instruction per line, or a line's lanes separated:

```
rep 3 last {
    add a1, a2, b3 :: add a3, a2, b3
    mov a0, bR
}
```

Each instruction goes in the first free slot of its unit. A bundle's
prefix is not read as a label (a prefix written `2: {` would be one in
GNU's syntax). Comments may start with any of the description's
`asm-comment` markers (`# ;`).
Names of numbers (`.set n, 4`, `.equ n, 4`) are put in where they are
used, in prefixes as well as operands.

### Checking

`--check-target` checks each entry of a bundle in a bundle of its own:
each sample of its operands, every value of its enums among them, is put
in its slot, and the bundle must decode as the entry (or as another
writing the same bundle: an alias, or a register written two ways). The
assembly check leaves such entries to that.

`tests/bundles.rs` does this for a made-up machine (`tests/bundles/toy.ll`):
bytes worked out by hand, a port's conflict, and disassembly read back.

Not yet: groups whose last instruction is marked (R600's), and fields
computed from which units the slots hold (Itanium's templates).

## 2. Scheduling into bundles

An entry's `"latency"` is the number of bundles after its own before its
result can be read (1 when absent). Within a bundle every slot reads
before any writes, so a value written in a bundle is not read in it.

After selection and before register allocation, each block is scheduled
as a list: a dependence graph of its instructions (registers, channels in
order, memory in order, side effects in order), with latencies on the
edges; then bundle by bundle, the ready instructions with the longest
path to the end of the block first, each in a free slot of its unit,
while the ports agree. The allocator then works on bundles: values
written in one bundle are live from the next. Copies the allocator adds
go in bundles of their own, or in free slots of a neighbour where their
operands are ready.

Machines without interlocks need the latencies kept after allocation too:
an empty bundle (all `fill`) is put where nothing else can go.

## 3. Repeat loops

`!target.loops` says how the hardware repeats a range of bundles:

```llvm
!target.loops = !{!1310}
!1310 = !{!"kind", !"header", !"start", !"rep", !"length", !"len",
          !"count", !"count", !"count-from", !"cnt", !"count-reg", !"lc",
          !"count-bias", !"1"}
```

| Property | Meaning |
| --- | --- |
| `kind` | `header` (the first bundle's header says so) or `instruction` (an entry with role `loop.start` begins it, `loop.end` ends it: R600's `LOOP_START_DX10` and `END_LOOP`, Itanium's `br.ctop`) |
| `start`, `length`, `count` | the header fields: a loop starts here, the bundles in it (less `length-bias`), and the times it runs (less `count-bias`) |
| `count-from`, `count-reg` | a header field choosing a count from a register instead, and the register (set by an entry with role `loop.count`) |

A loop in IR becomes a repeat loop when its body is one block (after
if-conversion, where the machine predicates), its trip count is known on
entry, it has no calls, and the body fits `length`. A count that fits
`count` is written there; one that does not, or is not constant, goes
through `count-reg`. A loop whose count may be zero gets a test before it.
Nested repeat loops are allowed as deep as the machine says
(`"depth"`).

## 4. Rotating registers

A register file can rotate: `"rotating"="r32-r127"` on its class says
which of its registers do. A rotating register's name is an offset from
a base that the hardware moves by one on each iteration of a repeat loop
(or each `br.ctop`), so what one iteration writes to `r33`, the next reads
as `r34`. Whether an instruction's register is rotating may be fixed by
its number (Itanium: `r32` and above), or chosen per bundle by a header
mode (section 1, ports), or per instruction by an operand marked
`"rotate"`. Entries with roles `rotate.reset` and `rotate.offset` set the
base and its step, where the machine has them.

Rotation is how a software-pipelined loop keeps a value alive for longer
than one iteration without unrolling. The plan:

- **Modulo scheduling** of a repeat loop's body: the initiation interval
  is the larger of the units' bound (instructions per unit) and the
  recurrences' bound (latency around each cycle of the dependence graph);
  instructions are placed in a table of that many bundles, by stage.
- **Allocation**: a value whose lifetime spans `k` iterations takes `k`
  consecutive rotating registers, named by its stage. Values that span
  one iteration or none are allocated as before, to registers that do not
  rotate.
- **Prologue and epilogue**: on a machine with stage predicates
  (Itanium's `p16` and up, rotating with the registers) the kernel alone
  runs, its count increased by the stages less one; otherwise explicit
  prologue and epilogue bundles are written around it.

Without rotation, the same schedule is kept by unrolling the kernel `k`
times (modulo variable expansion). That is the fallback, and is also what
a non-repeat loop gets.

## 5. Channels

Some units talk through FIFOs: one pushes values, another pops them in
the same order. Such values are not in any register the program can name.
A target extension type named in `!target.channels` is a channel value:

```llvm
!target.channels = !{!1320}
!1320 = !{!"type", !"stream", !"depth", !"8"}
```

Operations on channels are calls to functions the description declares,
and entries match the calls:

```llvm
declare target("stream") @fifo.push(<64 x float>)
declare <64 x float> @fifo.pop(target("stream"))
```

A channel value is used once, and pushes and pops of the channels between
two units stay in the order written. The optimiser treats the calls as
having side effects on an inaccessible location (they are not moved
across each other, merged or removed), unless their attributes say less.
Register allocation gives channel values no register. `fits` (in
partial-machines.md) checks that a channel value is not kept in a phi, a
select or memory.

## 6. Streams

A machine may have several units that each run a program of their own,
one starting the others: R600's control-flow program, which runs ALU and
fetch clauses, is one. A function for it is compiled into one program per
unit, from the one IR function. The description says which:

```llvm
!target.streams = !{!1330, !1331}
!1330 = !{!"name", !"cf", !"section", !".text"}
!1331 = !{!"name", !"alu", !"section", !".text.alu", !"bundle", !"group",
          !"launch", !"alu.launch", !"end", !"alu.end"}
```

An entry's `"stream"` says which stream it is in (the first stream when
absent). After selection, the instructions of each stream other than the
first are taken out of the function in regions: a region is a run of a
stream's instructions in one block (or a repeat loop of them, as a whole),
and becomes a program of that stream, with a symbol of its own
(`<function>.<stream>.<n>`). Where it was, the first stream gets the
entry with role `launch` of that stream, whose `"sym"` operand is the
region's symbol, with a relocation for the program's place in its
section. The region's last instruction gets the stream's `end` (as a
header field or an entry).

Values cross between streams only:

- through channels (section 5), which is how the streams' units pass
  data;
- as parameters: a scalar the first stream computes and the region reads
  is passed with the launch (role `launch.param`), and read in the region
  with role `param`.

Anything else crossing, such as a value a region computes that the first
stream then uses in a register, is an error naming it. Instructions of
different streams can be side by side in one IR block; only the order of
their channel operations ties them together.

Each stream's code goes to its section in the object file. A section's
code is in bundles of the stream's format, and its symbols are
measured in the units its launch operand counts (`"unit-bytes"`), so a
relocation can resolve to a bundle number.

## 7. Several machines in one object

The streams of section 6 are units of one description. Sometimes they are
different machines: a host processor and a DSP, or a control core and the
accelerator whose programs it starts. Each has a description of its own,
and one object holds the code of all of them.

### What LLVM does

LLVM IR gives each function `"target-cpu"` and `"target-features"`
attributes, so functions of one module can be compiled for different CPUs
of one architecture (C's `__attribute__((target("avx2")))`, Rust's
`#[target_feature]`, ARM and Thumb functions side by side). The triple and
the data layout are the module's, though, so a module is for one
architecture. Code for another (CUDA, HIP, OpenMP offloading) is a module
of its own, compiled separately, and its object is embedded in the host's
object as data (`.llvm.offloading`, a fat binary), to be loaded at run
time. Cell's SPU code was embedded in the PPU's object in the same way.

### Here: a stream is a description

A stream may name a target instead of being a part of this one:

```llvm
!target.streams = !{!1340, !1341}
!1340 = !{!"name", !"host", !"section", !".text"}
!1341 = !{!"name", !"dsp", !"target", !"toy-dsp", !"section", !".text.dsp .dsp.text*"}
```

The description holding `!target.streams` is the first stream's, and is
the object's machine (`e_machine`). A stream with `"target"` is compiled
by that description (found as `--target` finds one): its own selection,
allocation, bundles and encodings. Such a description has no streams of
its own.

Which code goes to which stream is said by sections, a convention every
front end already has:

- a function whose `section` is one of a stream's sections is compiled
  whole for that stream (Rust's `#[link_section = ".text.dsp"]`, C's
  `__attribute__((section(".text.dsp")))`); it may not be called from
  another stream, only launched;
- otherwise, regions of a function are taken out for a stream by the
  partition of section 6, by the streams of the entries selected.

The object is one ELF file with the first stream's `e_machine`. Each other
stream's sections are its code, as they would be in an object of its own,
and a note, `.note.epitome.machines`, lists which sections are which
machine's (section name pattern, `e_machine`, bundle format), so that
`epitome-objdump` disassembles each by its own description. Other tools
see code they cannot read, as they would see data.

A launch is the first stream's instruction, so its relocation is in the
first stream's machine's numbering, with the symbol measured in the
launched stream's `unit-bytes`. Machines without numbers to spare have a
way to add some: RISC-V's `R_RISCV_VENDOR` (191) names a vendor for the
relocation that follows it, whose type is from 192 to 255.

Embedding a whole object, as LLVM does, is kept for later, for a loader
that wants each machine's code as a file of its own: the sections and the
note are what it would be made from.

## Status

| Item | Status |
| --- | --- |
| `!target.bundles`: fixed bundles, header fields and families, slots, units, lanes | **Done** |
| Header fields set by operands (`"header"`), ports | **Done** |
| Prefixes and the last bundle's fields | **Done** |
| Assembly and disassembly of bundles | **Done** |
| Bundle checks in `--check-target` | **Done**: entries in bundles; not yet the format itself (overlaps) |
| Numeric `.set` and `.equ` | **Done** |
| Groups (R600), templates (Itanium) | **Later** |
| `"latency"` and list scheduling into bundles | **Planned** |
| Repeat loops (`!target.loops`) | **Planned** |
| Rotating registers and modulo scheduling | **Later** |
| Channels | **Planned** |
| Streams, launch and parameters | **Planned** |
| Several machines in one object: streams with `"target"`, functions by section, the machines note | **Planned** |
| Itanium description, for assembly and disassembly | **Later** |
