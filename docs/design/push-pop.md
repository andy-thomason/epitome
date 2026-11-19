# Push and pop on x86-64

The idea: x86's `push` and `pop` are a byte or two (`push imm8` two),
and the stack engine runs them as cheaply as moves, where a store to or
load from `disp(%rsp)` takes four to eight bytes. So stack temporaries
could be made with pushes of immediates, and spills and reloads done
with `push r` and `pop r`, if the code generator knew the depth of the
stack and what each pushed slot holds at the start of every block.

This note measures how much of our x86 code is stack traffic and of what
kind, sets out what a design for push and pop across blocks needs, and
says which parts pay. The short answer: the stack traffic is large, but
very little of it has the shape push and pop fit. One part pays now and
is done (the frame's last eight bytes, below); the rest of the gap is
better closed by the other changes listed at the end.

## What the stack traffic is

Two samples. bench/vs-llc's x86-64 scalar crates (regex-syntax,
serde_json, ryu and others: rustc's LLVM IR, compiled by `epitome-llc`
and by llc -O3), after the register allocation work of #199 and #200;
and std's hello world (`epitome-rustc2/tests/run/std_hello.rs`, built by
epitome-rustc2 on x86-64 Linux, as one object, with std). The counts are
`bench/vs-llc/stack.py`'s: every instruction addressing memory at `rsp`
(or `rbp`), with its bytes, and how many need a 32-bit displacement.

| bench/vs-llc, scalar crates | llc -O3 | Epitome | |
|---|---|---|---|
| all code | 293,764 bytes | 364,323 bytes | 1.240x |
| loads from the stack | 3,171 (17,438 B) | 4,790 (29,689 B) | |
| stores to it | 2,449 (13,092 B) | 4,245 (25,417 B) | |
| vector loads and stores (16-byte copies) | 2,322 (14,681 B) | 4,453 (32,730 B) | |
| immediates stored | 740 (6,386 B) | 0 | |
| `leaq` of a slot | 1,358 (7,834 B) | 1,861 (11,330 B) | |
| ... of those, 32-bit displacements | 2,610 | 8,206 | |
| allocator's spill loads / stores | | 2,223 / 1,213 | |

| std hello world | Epitome |
|---|---|
| all code | 214,913 bytes |
| loads from the stack | 3,501 (23,095 B) |
| stores to it | 4,565 (30,098 B) |
| vector loads and stores | 7,539 (58,902 B) |
| `leaq` of a slot | 1,457 (9,192 B) |
| ... of those, 32-bit displacements | 12,411 |
| loads of a value stored earlier in the same block | 3,382 (24,485 B) |

So stack traffic is a third of our bytes on the crates (about 100 KB of
364 KB, where llc's is 62 KB of 294 KB: 38 KB of the 70 KB gap), and
more than half of hello world's.

What it is:

- **Spills are the smaller part.** On the crates the allocator's own
  spill code is 2,223 loads and 1,213 stores of the 7,201 and 6,260 (and
  most of it is in loops of regex-syntax, where a value is reloaded at
  many places). The rest are allocas' loads and stores: Rust's
  aggregates, `Option`s and `Result`s, and arguments passed by
  reference.
- **Hello world's is almost all allocas the optimiser left.** In the
  optimised IR of `CharSearcher::next_match`, for instance, seventeen
  16- and 24-byte allocas are written field by field and copied to each
  other with `memcpy` (`store i64 0, ptr %11` and, next, `load ptr, ptr
  %11`; `memcpy(%8, %6, 16)` then `memcpy(%22, %8, 16)`), and each copy
  is a pair of `movups` through `xmm0`. 3,382 loads (24 KB, 11% of the
  code) read a slot that a store a few instructions before, in the same
  block, wrote from a register.
- **Frames are big, so offsets are long.** Our frames sum to 80 KB over
  the crates where llc's sum to 45 KB (every alloca has its own slot:
  `llvm.lifetime` is dropped, so slots are never shared), and slots are
  laid out smallest first, so the many accesses of 16- and 24-byte
  aggregates are at offsets past 127: 8,206 accesses take a 32-bit
  displacement where llc has 2,610 (3 bytes each, 17 KB).
- **No immediate is stored directly.** `movl $1, %eax; movb %al,
  0xe0(%rsp)` where llc writes `movb $1, 0xe0(%rsp)`: the x86
  description has the store-immediate forms only as `asmonly`. 674
  stores on the crates come through a register made only for them.
- **Smaller things.** 154 stores on the crates made dead by a second
  store to the same slot a few instructions later (a spilled value
  stored before and after a two-address instruction: `movq %rsi,
  (%rsp); addq $-1, %rsi; movq %rsi, (%rsp)`), and 179 reloads into a
  scratch register used once by the next instruction, which could take
  the slot as its memory operand (`cmpq 0x90(%rsp), %rax`).

## Where push and pop would fit

A push and its pop must nest: the slot a push makes is the top of the
stack until it is popped. A value fits if it is stored once and loaded
once, with every other push and pop between them nested inside. The
candidates, counted from the code as it is (`once` in `stack.py`: a slot
stored once and loaded once by 8-byte moves, nothing else touching it):

| bench/vs-llc, scalar crates | slots | bytes now |
|---|---|---|
| in one block, other stack accesses between | 97 | 1,522 |
| in one block, a call between | 3 | 48 |
| across blocks, no call between | 56 | 544 |
| across blocks, a call between | 211 | 2,328 |
| all | 367 | 4,442 (1.2% of the code) |

Turning all of them into pushes and pops saves at most about 3 KB
(0.8%), before the costs below, which take a good part of it. On hello
world, the 623 slots stored and loaded once with nothing between, and the
1,019 with other accesses between, are allocas stored and read back at
once: removing both (forwarding the stored register) beats a push and a
pop, and leaves nothing for them.

Push imm for constant temporaries fares no better: the temporaries in
our code are fields of allocas, written in whatever order the IR has,
into slots at fixed places in the frame, not values made at the top of
the stack just before they are used; and a single constant is better
stored directly (`movq $c, disp(%rsp)`, which we do not select yet) or
kept in a register. (`push imm8; pop r` makes a constant in three bytes
where `movl` takes five, as llc does at -Oz, but it costs a store and a
load on the critical path, so not for speed.)

## What a design across blocks needs

For the record, should allocation later make values that do fit (the
register allocation notes suggest some: a value reloaded once for a run
of uses across a call), these are the pieces; none is built.

- **A stack state per block.** At each block's start: the depth below
  the fixed frame, and which value (or constant) each pushed slot holds,
  innermost last. It is found forwards, as a JVM verifier does: a block's
  state is its predecessors' end state, which must agree; where two
  edges disagree, the edge with more is fixed with an `addq $8k, %rsp`
  (or pops into a dead register) on a block of its own, or the value is
  not pushed. Pushes nest like the lifetimes they hold, so a value is
  pushed only where its lifetime is inside every lifetime pushed before
  it; a loop's back edge must bring the depth back to the header's, so a
  push in a loop is popped in it.
- **Addressing the rest of the frame.** The frame's slots are at fixed
  offsets from the frame's bottom; with `d` bytes pushed, a slot at `o`
  is at `o + d` from rsp. Frame lowering adds the depth at each
  instruction (known from the state) to the offset, which is cheap and
  keeps rbp free; a frame pointer instead would cost a register and four
  bytes of prologue in every such function.
- **Calls.** rsp must be 16-aligned at each call: an odd number of pushed
  slots needs another push (a byte) or eight bytes of `subq` around the
  call. Arguments passed on the stack are at fixed offsets from rsp at
  the call, so the depth is added there too.
- **Unwinding.** Our tables are synchronous (correct at calls,
  `unwind.rs`), so a push needs a `DW_CFA_def_cfa_offset` row only when a
  call comes before its pop: two or three bytes of `.eh_frame` each, at
  the push and at the pop, which takes back most of what a single push
  and pop across a call save in code.
- **The stack engine.** Pushes and pops update rsp in the front end; the
  first instruction after them that reads rsp explicitly (`movq
  8(%rsp), %rax`, `leaq`, a call's own use is fine) waits for a sync
  µop. A run of pushes or pops then an rsp-relative access costs one
  µop; interleaving them costs one per access. So pushes and pops belong
  in runs, away from rsp-relative accesses, which the in-block candidates
  above mostly are not (97 of 100 have other stack accesses between).

## What pays, and what is done

**The frame's last eight bytes** (done, `frame.rs`). A function's frame
below the return address and the pushed saves is often just the eight
bytes that align rsp for calls; it was `subq $8, %rsp` and `addq $8,
%rsp` before each way out (four bytes each). Now it is `pushq %rax` (one
byte; whatever rax holds), and, before each return, a `pop` of a
call-clobbered register the return does not read (`popq %rcx` where rax
holds the result: one byte), or before a tail call, of one that is
neither an argument nor the result (`popq %r11`: two bytes). The CFA rows
are those the subtraction had. This is what llc does (its prologues have
`pushq %rax`), and the stack engine makes it as cheap as the subtraction.

| | bytes, bench/vs-llc scalar crates | std hello world |
|---|---|---|
| before | 364,323 (1.240x llc) | 214,913 |
| after | 363,338 (1.237x llc) | see the PR |

**Not done: push and pop of spills and constant temporaries.** At most
0.8% of the crates' code before the costs above (alignment at calls,
unwind rows, sync µops), nothing on hello world, for a dataflow analysis
and a change of every frame offset. Worth looking at again only if the
allocator comes to produce many values reloaded once across a region.

**What would pay more** (each its own piece of work, outside push and
pop):

1. Forwarding and removing allocas' stores and loads, and copies between
   allocas (SROA through `memcpy`, store-to-load forwarding in a block):
   up to 24 KB of hello world's 215 KB in the loads alone, more with the
   stores and the `movups` copies they feed. In the optimiser, before
   code generation.
2. Smaller frames with the busiest slots near rsp: sharing slots whose
   allocas' lifetimes do not overlap (keeping `llvm.lifetime` to frame
   layout), and ordering slots by accesses per byte, so most offsets fit
   in a byte: up to 3 bytes for each of 8,206 accesses on the crates
   (17 KB, 5%) and 12,411 on hello world.
3. Storing immediates directly (`movq $c, disp(base)` and its narrower
   forms, selectable): 674 stores on the crates.
4. In the allocator or a peephole: dropping a store made dead by the next
   store to the same slot (154 on the crates), and folding a reload used
   once into its user's memory operand (179).
