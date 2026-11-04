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
   units, one launching the others.

None of it names a machine. The public machines it is checked against are
Intel's Itanium (bundles with a template, rotating registers, `br.ctop`)
and AMD's TeraScale, R600 and Evergreen (groups, clauses, hardware loops;
[partial-machines.md](partial-machines.md), section 5). For sources and
status labels, see the [index](README.md).

## 1. Bundles

`!target.bundles` lists the bundle formats, as property pairs:

```llvm
!target.bundles = !{!1300}
!1300 = !{!"name", !"b", !"bytes", !"16",
          !"header", !"template:5",
          !"slot", !"0", !"5", !"41", !"M",
          !"slot", !"1", !"46", !"41", !"M,I",
          !"slot", !"2", !"87", !"41", !"I,B"}
```

| Property | Meaning |
| --- | --- |
| `name` | the format's name, used by `"bundle"` on entries and by sections |
| `bytes` | the bundle's size; bytes after its last field are zero |
| `header` | an encoding, from bit 0, of the bundle's own fields |
| `slot` | a slot's number, its lowest bit, its width and the units whose entries may go in it |
| `fill` | for each unit, the entry put in a slot of that unit that has nothing in it (a `nop`); else zeros |
| `group` | `last=<operand>`: a bundle is not a fixed word but a run of instructions, the last with that operand set (R600's groups) |

An entry says which units can issue it with `"unit"="M,I"`. An entry with
no `"unit"` is not part of any bundle; on a machine with bundles it is
an instruction of a stream that has none (section 6).

### Header fields

A header field is one of three things:

- **A bundle operand**, written in the assembly before the bundle: a
  repeat count, the end of a program. The header encoding names it, and
  `"prefix"` gives its template (`{len}:{count}`).
- **A template**: a field computed from which unit each slot holds
  (Itanium's 5-bit template). The field's expression may use `unit0`,
  `unit1`, ... (the index, in the slot's list, of the unit used) and
  `stop0`, `stop1`, ... (whether a stop follows the slot).
- **A port**: a register file read through one port per bundle. The
  header holds the register read; the slots that read the file name only
  the file. An entry's operand with `"port"="p"` is encoded in the header
  field `p`, not in the slot, and every slot in the bundle reading the
  port must read the same register. A port can also carry a mode, such
  as whether the register is rotating (section 4): `"port-mode"="pm"`.

### Assembly

A bundle is written between `{` and `}`, one instruction per line or
separated by `;`, its prefix (if any) before the `{`:

```
3:7 {
    add r1 = r2, r3
    ld8 r4 = [r5]
}
```

Slots are filled in the order the instructions are written, each going in
the first free slot whose units include its own. A bundle with no
instruction for a slot gets the slot's `fill`. The disassembler decodes
the header, then each slot by the entries of the unit the slot (and the
template) says it holds, and writes the bundle the same way.

### Checking

`--check-target` checks each format: slots inside `bytes` and not
overlapping the header or each other; every unit in a slot list having
entries; every entry's encoding the width of its slots; ports naming
header fields; a `fill` for each unit, or zeros decoding as nothing.
The encodings check encodes and decodes whole bundles: each entry in each
slot it can go in, the others filled.

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
!1310 = !{!"kind", !"header", !"start", !"rpt", !"length", !"len",
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

## Status

| Item | Status |
| --- | --- |
| `!target.bundles`: fixed bundles, header, slots, units, fill | **Planned** |
| Bundle operands, templates and ports in the header | **Planned** |
| Assembly and disassembly of bundles | **Planned** |
| Bundle checks in `--check-target` | **Planned** |
| Groups (R600) | **Later** |
| `"latency"` and list scheduling into bundles | **Planned** |
| Repeat loops (`!target.loops`) | **Planned** |
| Rotating registers and modulo scheduling | **Later** |
| Channels | **Planned** |
| Streams, launch and parameters | **Planned** |
| Itanium description, for assembly and disassembly | **Later** |
