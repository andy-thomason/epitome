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
  (`isel::class_of_ty`). Entries for selection are in `82-sve.ll` (needs
  the feature `sve`).
- Arguments and results: z0-z7 are d0-d7's registers, so the convention's
  floating-point registers already pass them (AAPCS64's SVE variant).

## Still to do

In order:

1. **Spills and frames.** A slot of a scalable value is sized in vector
   lengths (`mul vl`): laid out apart from the rest, made with `addvl`,
   addressed `[sp, #k, mul vl]`; the CFA from the frame pointer once sp
   moves by a vector length.
2. **Predicates.** A third file in the calling convention (p0-p3 for
   arguments, p0 for a result), allocatable; spills in predicate lengths.
3. **Selection**: what clang's vectoriser makes (loads and stores with an
   all-true predicate, splats, `llvm.vscale`, arithmetic, comparisons into
   predicates, selects, `llvm.fmuladd`, reductions, `whilelo` loops), as a
   generated `82-sve.ll` (from tables, with `81-scalable.ll`'s
   encodings).
4. **The passes before selection** leave scalable vectors alone, or say
   what they cannot do with them (`intrinsics::lower`'s lane-by-lane
   expansion, the shuffles and masks of `legalize`, `mangle`'s `nx`).
5. **SVE's intrinsics** (`llvm.aarch64.sve.*`), found as x86's are
   (`x86_64/discover_intrinsics.py`): their types from Rust's
   `core::arch`, the instruction from llc, the encoding from
   `81-scalable.ll`; and the catalogue's SVE lines run under qemu.
6. **SME**: streaming mode and ZA as values, beyond `45-sme.ll`'s fixed
   registers.
