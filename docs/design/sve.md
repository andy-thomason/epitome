# SVE and SME

How Epitome compiles AArch64's scalable vectors (SVE, and SME's streaming
mode): values of types such as `<vscale x 4 x i32>` and predicates
(`<vscale x 16 x i1>`) in the z and p registers, whose size is a multiple
of 128 bits (16 for predicates) that the machine sets, not the compiler.
Status labels as in the [index](README.md).

## The encodings (done)

Every SVE and SME instruction Arm's specification and clang agree on is
in `81-scalable.ll` for assembly and disassembly (`gen_mrs_asm.py
--scalable`; see `isel.md`): 1,241 of Arm's 1,346 SVE encodings, 654 of
SME's 842. Register lists are written with the registers after the first
following from it (`{lb}{zn}.s, {zn:+1}.s{rb}`).

## Testing (done)

Neither machine the tests run on has SVE: an M1 has neither SVE nor SME,
and the Linux machine is x86-64. Both builds of a test (ours, clang's) are
run under qemu (`qemu-aarch64 -cpu max`), static against Rust's musl
(rustup's `aarch64-unknown-linux-musl` target):

- `tests/sve.rs`: hand-written IR (`tests/sve/NAME.ll`) with a C driver
  (`arm_sve.h`), run at vector lengths of 128, 256, 512 and 2048 bits, so
  that code right at one length only is caught.
- `tests/intrinsics.rs`: the generic and AArch64 catalogues under qemu
  too, with its CPU's features (`*-linux-todo.txt`).

## Values in registers (partly done)

- Register classes hold a kind (`machine.rs`, `Holds`): z holds
  `scalable` (its `bits`, 128, the least vector length), p and pg
  `predicate` (16). A scalable type's width is its least
  (`target::type_bits`), and pattern heads mark scalable types
  (`target::SCALABLE`), so that `<vscale x 4 x i32>` and `<4 x i32>` are
  told apart.
- Selection puts a scalable vector in z, a scalable vector of i1 in p
  (`isel::class_of_ty`). Entries for selection are in `82-sve.ll`,
  written by `gen_sve.py` from tables with `81-scalable.ll`'s encodings
  (needs the feature `sve`): so far copies, whole registers to and from
  memory (`ldr`, `str`: little-endian, the lanes in order whatever their
  width), and unpredicated arithmetic and logic.
- Arguments and results: z0-z7 are d0-d7's registers, so the convention's
  floating-point registers already pass them (AAPCS64's SVE variant).

## Spills and frames (done)

A spill slot of a scalable value is a vector length (`Slot::scalable`),
whatever the machine's is. A function with such slots has them between
the rest and the saves (`frame::scalable_frame`): from sp up, arguments
for calls and the other slots, then the scalable slots, then the saves and
the frame record. The frame is made in three steps (the top, by bytes,
with x29 set and the CFA defined from it; the scalable slots, `addvl sp,
sp, #-n`; the rest, by bytes) and undone from x29. Scalable slots are
addressed in vector lengths from `sp` plus the fixed part (`ldr z0, [x16,
#k, mul vl]`), incoming arguments from x29. The spills are `82-sve.ll`'s
roles `load.z` and `store.z`, the steps `addvl`.

## Predicates (done)

Predicates are a third file in the calling convention (`pred-args` p0-p3,
`pred-result` p0, the rest clobbered), allocated as z values are and
spilt in predicate lengths (an eighth of a vector length: `ldr p0, [x16,
#k, mul vl]`). An all-true constant is `ptrue` of its lanes' width, a
false one `pfalse`.

## SVE's intrinsics (done)

`llvm.aarch64.sve.*` are found as x86's are (`discover_sve.py`, into
`sve-intrinsics.txt`): their types from Rust's `core::arch`, the
instruction from llc, the encoding from `81-scalable.ll`; `gen_sve.py`
makes an entry of each (1,606), and the AArch64 catalogue's SVE lines run
under qemu.

## Selection of the vectoriser's IR (done)

What clang's loop vectoriser makes, from `82-sve.ll` (`gen_sve.py
generic`):

- Operations SVE has only predicated (`mul`, divisions, shifts, min and
  max, `fdiv`, `fneg`, `fabs`, `sqrt`, `fma`, `minnum`, conversions) are
  recipes: the predicated instruction under `ptrue` of the lanes.
- Comparisons are into predicates (`cmpgt p0.s, p1/z, z0.s, z1.s`); those
  SVE has not (`slt`, `ult`, `olt`) are the others with their operands
  swapped. Selects are `sel`; a predicate extended to integers is a select
  of one (or minus one) and zero.
- Splats are `dup` (`mov z0.s, w0`), of a constant too (made in a
  register first); reductions are the across-lanes instructions (`uaddv`,
  `smaxv`, `fadda`; a byte's or half's read through the s register of the
  same number, which they clear above it); `llvm.vscale` is `rdvl` over
  16.
- Copies of z values are `mov z0.d, z1.d`, never the q register's copy,
  which would clear the lanes above 128 bits (`regalloc::lower_copies`
  takes a class's own copy first).

## Before selection (done in part)

`scalable.rs` runs first in `prepare`, before the passes that take lanes
as countable:

- **Values of more than a register** (`<vscale x 4 x i64>`, from widening
  `<vscale x 4 x i32>`) are parts of one register each, in lane order.
  Lanes cannot be counted, so the parts are not made by shuffles:
  extensions are the source unpacked (`sunpklo`, `sunpkhi`), truncations
  the parts' low halves zipped (`uzp1`), memory at `vscale` times 16 bytes
  a part, `llvm.vector.partial.reduce.add` the parts added in,
  `deinterleave2` (a loop over pairs) `uzp1` and `uzp2`, `interleave2`
  `zip1` and `zip2`.
- **Unpacked values** (`<vscale x 8 x i8>`, a byte in each 16-bit lane)
  are held in their containers (`<vscale x 8 x i16>`), their lanes' upper
  bits unknown, and extended where those matter (shifts right, divisions,
  comparisons, extensions); their loads and stores are `ld1b {z0.h}` and
  `st1b` (`llvm.epitome.sve.ld1.nxv8i8`).
- **Masked loads and stores** are `ld1` and `st1` under the mask (a load's
  passthru selected in, unless zero or poison).

The other passes leave scalable vectors alone where they would count
lanes (`prepare`'s ordered reductions and vector bitcasts, `legalize`'s
shuffles), and intrinsics' names mangle them with `nx`.

`tests/sve/loops.c` and `loops2.c` are C loops clang vectorises for SVE
(sums, maxima, dot products, conversions, widening and narrowing,
conditional loads and stores, structures of pairs); the test builds
clang's IR of them with both compilers.

Loops with their tails folded in (`tests/sve/folded.c`, whose first line
gives clang `-mllvm -prefer-predicate-over-epilogue=...`, as some CPUs'
tunings do by default) run each step under the predicate of the lanes
still to do: `llvm.get.active.lane.mask` is `whilelo`, and the test of its
first lane that ends the loop is that lane of the predicate made integers
(`mov z0.s, p0/z, #1`, then `fmov w0, s0`).

## SME (done in part)

- **Intrinsics.** Rust's `core::arch` has no SME, so `discover_sme.py`
  finds them from C's: clang's AST dump lists `arm_sme.h`'s functions
  (1,845), each is called from a streaming function sharing ZA and the IR
  clang makes says which `llvm.aarch64.sme.*` it is (839), and then, as for
  SVE (`discover_sve.py`'s matching), llc's code (clang's own llc, 22) is
  found among `81-scalable.ll`'s entries: 550, in `sme-intrinsics.txt`,
  which `gen_sve.py` makes entries of (`SME.*`). ZA's slices are chosen by
  w12-w15 (class `ws`) or SME2's w8-w11 (`wv`): classes with gaps
  (`!"-{0-11}", !"w{12-15}"`), into which selection copies a value whose
  class has registers they have not (as it now does for SVE's p0-p7).
- **Streaming mode and ZA** (`sme.rs`, from the functions' attributes):
  `aarch64_pstate_sm_body` switches on after the prologue and off before
  each return; `aarch64_new_za` commits a caller's lazily saved ZA
  (`__arm_tpidr2_save` when TPIDR2_EL0 says there is one), then turns ZA on
  and zeroes it, and off before each return; a call from a streaming body
  of a function that is not, or the other way, switches around it. The
  switches clear the vector and predicate registers ("clobbers"), so
  nothing stays in one across them, and d8-d15 are saved.
- `tests/sve/sme.c`: outer products into a tile, a transpose through ZA,
  bytes' outer products, and calls both ways, run at the four streaming
  lengths with FA64 off (as on Apple's M4), so that an instruction
  streaming mode does not allow would fault.

## Lists of registers (done, as operands)

`st2 {z0.d-z1.d}`, `tbl {z0.b-z1.b}`, SME2's `{z0.s-z3.s}`: a template
derives the members from the head (`{zt1:+1}`), and the intrinsic takes
them as separate values. `gen_sve.py` gives each member a parameter
`"list"="a0:+1"` (from discovery's `unused=` arguments after the head's),
for 246 entries (st2-st4, tbl2, SME2's lists into ZA).

- **Selection** copies each member, the head too, into a value of its
  own just before the instruction, so one value may be in two places of
  a list (`st2 {z3, z4}` of `a, a`) or in lists of different orders.
- **Allocation** (`regalloc.rs`) places these values first, a list at a
  time: the first register where all members are free and the
  instruction can encode it (SME2's lists of two or four start at a
  multiple of two or four: `Instr::list_starts` tries the encoding), the
  one most members' copied values are drawn to (an argument's register,
  an earlier list's), less the values live across the list that are
  drawn to its registers and would have to move (an argument still
  needed after), then one that need not be saved. They are never moved
  for other values, and are kept out of coalescing.
- **Back along the ranges**: the values copied into a list are placed
  later, and the copies' hints draw each to its member's register, so a
  value whose live range ends in a list takes the register the list
  wants where it is free all along, and the copy goes. A value may also
  share a register with its own copies in lists (nothing writes it
  between them and the instruction), so one still needed after a list
  stays where the list reads it: `st4 {z0-z3}` of the four arguments,
  in order, is the store alone.
- **Two-address shapes**: a result is drawn (more weakly than by a copy)
  to the register of an operand whose range ends at the instruction, so
  that it reads and writes one register as a two-address form would.
- `tests/sve/lists.ll`: members out of order, one value twice, values
  read after the list, a list in a loop, and forty values live into ten
  `st4`s (spilled ones loaded straight into a list's registers).
  `tests/sve/lists/sme2.ll`: SME2's lists from values arriving at odd
  registers, two lists in one instruction; no qemu here runs SME2, so
  clang assembles our assembly (refusing a list out of line) and its code
  must be ours.

## Still to do

- **Lists of registers as results**: intrinsics returning tuples of
  vectors (`ld2`-`ld4`, SME2's multi-vector results and their destructive
  forms), which `discover_sve.py` skips; a list as a result would be
  members `d`, `d2`... placed as lists of operands are. And the `vg4`
  writes of ZA's slices, which discovery pairs with the two-register
  `mova`.
- **SME**: calls with ZA live of functions without it (ZA's lazy save, a
  TPIDR2 block), streaming-compatible functions' calls of ones that are
  not (switches by a test of PSTATE.SM), the runtime's own calls in
  streaming mode (`memcpy`, which should be `__arm_sc_memcpy`), scalable
  vectors into or out of a locally streaming function, and scalable spill
  slots of one (sized `addvl` in the prologue, before the switch, where
  `addsvl` is wanted).

- **Unpacked floats** (`<vscale x 2 x float>`, from loops mixing floats
  and doubles), and vectors of more than a register in compares, selects
  and calls other than those above.
- **Gathers and scatters** (`llvm.masked.gather` of scalable vectors).
