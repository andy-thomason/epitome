# Code size on real crates

How big epitome-rustc2's programs are against rustc's, on twelve small
crates from crates.io, on the Mac (AArch64) and on x86-64 Linux, and where
the difference is. `bench/crate-sizes/` reproduces it.

The short answer: whole programs are smaller than rustc's (rustc's
precompiled `std` carries 144 KB of backtrace support), but the code a
crate itself brings is **3.2× rustc -O's on the Mac and 3.7× on x86-64**
(the eight crates whose own code lowers completely), far worse than the
1.8× a hello world's functions suggested. Three things make most of it,
in this order:

1. **Aggregates through memory.** `Result`, `Option`, `Layout`, slices and
   small structs are kept in allocas and copied slot to slot; frames of
   1–3 KB in the functions that matter. Stack loads and stores are 29% of
   our instructions on the Mac (16.2k, rustc 2.6k) and 37% on x86-64
   (26.0k, rustc 2.6k): over half of the whole excess.
2. **AArch64 frame addressing.** 16-byte slots sit at offsets ≡ 8 (mod 16),
   and half the offsets are past 4 KB, so a `q`-register access takes
   `mov x15, #off; add x15, sp, x15; ldr q0, [x15]`. That is **17% of all
   our instructions** in the crates' code on the Mac (9.5k of 56k), and a
   cheap fix.
3. **`#[derive(Debug)]` expands to builder chains**
   (`f.debug_tuple("N").field(..).finish()`) where rustc's derive calls
   `Formatter::debug_tuple_field1_finish` and kin: every derived `Debug`
   is 3–13× rustc's, in core and std as well as the crates.

Then x86 vector code lowered lane by lane (memchr: 5.9×), inlining
choices, and missed CSE. Details and follow-ups below.

## Method

Each crate has a demo program (`bench/crate-sizes/progs/<crate>_demo.rs`)
that exercises its main API and prints the results. The inputs go through
`std::env::args().len()` (1 when run), so neither compiler can fold the
program into its output: rustc -O folded the first version of itoa's
demo to 856 bytes of `puts`-like code. `build.sh` builds each program:

- epitome-rustc2, `--std`, as it builds by default (**ep default**: the
  crates keep `debug_assertions`), with `-C debug-assertions=off` (**ep
  ndebug**, the like-for-like build: rustc's `-O` turns them off), and with
  that and `-Cpanic=immediate-abort` (**ep abort**);
- rustc 1.99.1, the crate as an rlib and then the program, `-C
  opt-level=3` (**rustc -O**) and `-C opt-level=s`, both `-C panic=abort`,
  with v0 symbols; `std` is rustc's precompiled one;
- a hello world (`progs/hello.rs`) the same ways, as a baseline.

It runs every build and checks it prints what rustc -O's does.
`sizes.py` then measures:

- **text**: the size of `__text` / `.text`;
- **text over hello world**: that less hello world's, what using the crate
  costs, `std`'s generic code instantiated for it included;
- **the crate's code**: the bytes of the functions written in the crate or
  in its demo program. A function's size is the distance to the next
  symbol. Ours are put down to crates by the new `epitome-rustc2
  --fn-crates FILE` (a symbol of an impl's method, `_R.Buffer..format…`,
  doesn't name its crate); rustc's by the first crate root in the v0
  symbol. Generic instances of `std` functions count as `std`'s on both
  sides;
- functions matched by name without paths, generic arguments or hashes
  (`<Buffer>::format`, `<f32 as Sealed>::is_nonfinite`, `main::{closure}`);
- with `--mix`, the instruction mix of the crate's code (`mix.py`): stack
  loads and stores (`sp`/`x29`/`x15` or `rsp`/`rbp`), AArch64 frame
  addresses made in `x15`, vector instructions, calls, traps, moves;
- with `--new`, the biggest functions the program has beyond hello world.

```sh
cargo build --release -p epitome-rustc2
cargo fetch --manifest-path bench/crate-sizes/fetch/Cargo.toml   # if crates are missing
bench/crate-sizes/build.sh             # target/crate-sizes; or build.sh OUT crate...
bench/crate-sizes/sizes.py --mix --new > sizes.md
bench/crate-sizes/mix.py target/crate-sizes/smallvec/ep-ndebug.exe try_grow --code
```

The numbers here are from main at 6907286 plus this branch, on an Apple M1
and on mandelbrot.

### Caveats

- Where a program misbehaves (below), it is because a function could not
  be lowered and traps; a trapping function is small, so those crates'
  numbers flatter us. fnv, memchr (Mac), unicode-width and strsim lose
  real code that way and are left out of the totals; arrayvec, hex,
  base64 and heck trap in `std` functions they call, with their own code
  whole.
- Matching by name is approximate where the two compilers inline
  differently: rustc keeps `ryu::pretty::format64` and inlines
  `Buffer::format_finite` into `main`; we do the opposite. The crate
  totals don't depend on it.
- Two demos are written around epitome-rustc2 bugs (each place says so
  in a comment): semver's and fnv's.

## Results

Hello world's text: Mac ours 135,400 (immediate-abort 45,904), rustc
216,892; x86-64 ours 116,599 (40,649), rustc 241,827.

### Mac (AArch64)

| crate | output | text over hello: ep ndebug | rustc -O | crate's code: ep default | ep ndebug | ep abort | rustc -O | rustc -Os | ndebug / rustc -O |
|---|---|---|---|---|---|---|---|---|---|
| itoa | ok | 7,484 | 4,096 | 9,132 | 8,576 | 8,076 | 3,340 | 3,200 | 2.57 |
| ryu | ok | 10,448 | 6,784 | 12,192 | 11,384 | 11,008 | 6,968 | 5,508 | 1.63 |
| fnv | traps | — | 14,080 | 1,000 | 1,000 | 616 | 2,860 | 1,332 | — |
| smallvec | ok | 36,776 | 11,328 | 37,524 | 32,852 | 31,880 | 6,320 | 5,920 | 5.20 |
| arrayvec | traps | 5,620 | 2,688 | 5,556 | 5,472 | 3,704 | 1,860 | 2,060 | 2.94 |
| hex | traps | 16,104 | 5,760 | 9,372 | 9,372 | 8,520 | 3,244 | 2,960 | 2.89 |
| base64 | traps | 45,216 | 7,616 | 38,896 | 38,608 | 34,992 | 6,048 | 5,700 | 6.38 |
| memchr | traps | — | 17,792 | 1,280 | 1,280 | 1,144 | 16,116 | 9,092 | — |
| semver | ok | 59,228 | 24,320 | 48,504 | 51,280 | 49,448 | 15,912 | 11,900 | 3.22 |
| unicode-width | traps | 4,936 | 10,880 | 5,492 | 5,492 | 4,952 | 10,576 | 8,076 | — |
| heck | wrong | 55,012 | 20,608 | 41,852 | 41,852 | 41,092 | 18,248 | 13,020 | 2.29 |
| strsim | traps | 10,868 | 29,824 | 18,632 | 18,632 | 17,012 | 10,780 | 6,880 | — |
| the eight | | 235,888 | 83,200 | 203,028 | 199,396 | 188,720 | 61,940 | 50,268 | **3.22** |

### x86-64 Linux

| crate | output | text over hello: ep ndebug | rustc -O | crate's code: ep default | ep ndebug | ep abort | rustc -O | rustc -Os | ndebug / rustc -O |
|---|---|---|---|---|---|---|---|---|---|
| itoa | ok | 9,440 | 3,379 | 11,876 | 10,756 | 9,968 | 3,312 | 3,080 | 3.25 |
| ryu | ok | 12,516 | 7,232 | 14,708 | 13,652 | 13,068 | 7,401 | 5,639 | 1.84 |
| fnv | traps | — | 16,752 | 1,292 | 1,292 | 772 | 4,080 | 1,504 | — |
| smallvec | ok | 68,812 | 11,840 | 44,192 | 41,136 | 38,128 | 6,584 | 6,031 | 6.25 |
| arrayvec | traps | 6,240 | 2,672 | 6,104 | 6,012 | 4,064 | 2,150 | 2,112 | 2.80 |
| hex | traps | 43,680 | 5,840 | 11,952 | 11,952 | 10,404 | 3,585 | 3,287 | 3.33 |
| base64 | traps | 72,044 | 8,960 | 40,508 | 40,196 | 36,544 | 7,226 | 6,466 | 5.56 |
| memchr | wrong | 149,656 | 21,280 | 128,952 | 115,308 | 111,612 | 19,492 | 14,231 | 5.92 |
| semver | ok | 88,676 | 26,832 | 56,508 | 59,072 | 57,700 | 16,640 | 12,300 | 3.55 |
| unicode-width | traps | 25,980 | 8,944 | 6,052 | 6,052 | 5,376 | 8,865 | 6,809 | — |
| heck | wrong | 89,484 | 19,376 | 51,904 | 51,904 | 52,108 | 17,296 | 14,293 | 3.00 |
| strsim | traps | 14,204 | 40,544 | 20,608 | 20,608 | 21,356 | 12,752 | 8,007 | — |
| the eight | | 390,892 | 86,131 | 237,752 | 234,680 | 221,984 | 64,194 | 53,208 | **3.66** |

("The eight" are itoa, ryu, smallvec, arrayvec, hex, base64, semver and
heck; memchr builds on x86-64, but only prints nothing.)

`debug_assertions` cost little (1.8% of the crates' code) and
immediate-abort 5% more: almost none of the excess is panic code.

### The instruction mix

The crates' code, all twelve crates, ep ndebug against rustc -O:

| | insts | stack loads | stack stores | frame addresses | other memory | calls | branches | traps | vector | moves |
|---|---|---|---|---|---|---|---|---|---|---|
| Mac, ours | 56,450 | 7,755 | 8,492 | 9,534 | 2,484 | 989 | 5,222 | 238 | 7,404 | 1,504 |
| Mac, rustc | 25,357 | 1,382 | 1,177 | 9 | 1,916 | 890 | 4,351 | 0 | 1,088 | 1,740 |
| x86-64, ours | 69,736 | 12,482 | 13,477 | — | 2,067 | 1,614 | 6,598 | 305 | 11,202 | 4,283 |
| x86-64, rustc | 27,389 | 1,195 | 1,420 | — | 1,731 | 854 | 4,200 | 806 | 1,372 | 2,591 |

Of the Mac's 31k extra instructions, 13.7k are stack loads and stores and
9.5k frame addresses; most of the 6.3k extra vector instructions are
16-byte copies between slots (`ldr q`/`str q`), so the same thing again.
Calls and branches are close to rustc's: control flow is not the problem.

## Per crate

The crate's code in each build, and the functions where ours is biggest
(Mac, with the same functions on x86-64). "—" is a function the other
compiler inlined.

### itoa

| | ep default | ep ndebug | ep abort | rustc -O | rustc -Os | ndebug / -O |
|---|---|---|---|---|---|---|
| Mac | 9,132 | 8,576 | 8,076 | 3,340 | 3,200 | 2.57 |
| x86-64 | 11,876 | 10,756 | 9,968 | 3,312 | 3,080 | 3.25 |

Everything is inlined into `main` on both sides (8.6 KB against 3.3 KB on
the Mac). Ours has 300 stack accesses against 58 and 184 frame-address
instructions; on x86-64, 366 register moves against 106. The default
build also keeps itoa's `debug_assert_eq!`s in `udiv128` (eight calls of
`assert_failed::<u128, u128>`).

### ryu

| | ep default | ep ndebug | ep abort | rustc -O | rustc -Os | ndebug / -O |
|---|---|---|---|---|---|---|
| Mac | 12,192 | 11,384 | 11,008 | 6,968 | 5,508 | 1.63 |
| x86-64 | 14,708 | 13,652 | 13,068 | 7,401 | 5,639 | 1.84 |

| function | Mac ep | Mac rustc | x86 ep | x86 rustc |
|---|---|---|---|---|
| `main` | 6,552 | 1,140 | 7,312 | 1,568 |
| `<Buffer>::format_finite` | 5,408 | — | 6,340 | — |
| `format32` + `format64` + `write_mantissa_long` | — | 5,828 | — | 5,833 |

The best of the twelve, and nearly free of stack traffic: the excess is
selection and folds. `format_finite::<f64>` is 1,352 instructions against
rustc's `format64` 606: `write_mantissa_long` is inlined into each of
`format64`'s branches (rustc calls it), `x / 100` is computed twice
(`umull; lsr #37` repeated), and constants (`#10000`, `#100`) are made
again in each block. `main` has `format_finite::<f32>` inlined where
rustc calls `format32`.

### smallvec

| | ep default | ep ndebug | ep abort | rustc -O | rustc -Os | ndebug / -O |
|---|---|---|---|---|---|---|
| Mac | 37,524 | 32,852 | 31,880 | 6,320 | 5,920 | 5.20 |
| x86-64 | 44,192 | 41,136 | 38,128 | 6,584 | 6,031 | 6.25 |

| function | Mac ep | Mac rustc | x86 ep | x86 rustc |
|---|---|---|---|---|
| `main` | 18,860 | 2,260 | 23,480 | 2,672 |
| `<SmallVec>::try_grow` | 9,288 | 952 | 12,556 | 992 |
| `<SmallVec as Debug>::fmt` | 3,080 | 412 | 3,464 | 472 |
| `<SmallVec as Deref>::deref` | 328 | — | 328 | — |
| `<SmallVec>::len`, `set_len`, `as_mut_ptr`, `infallible` | 520 | — | 456 | — |

The worst ratio. `try_grow::<[u32; 4]>` is 1,104 instructions against
111 (479 stack accesses, 250 frame addresses), with a 2.5 KB frame: `Result<(), CollectionAllocErr>`, `Layout` and
`Result<Layout, LayoutError>` are each built in a slot, copied to
another, reloaded and copied again; `str x23, [sp, #0x98]; ldr x1, [sp,
#0x98]` back to back; and every 16-byte copy at an offset ≡ 8 (mod 16)
past 2 KB takes three instructions. The tiny accessors (`deref`, `len`,
`set_len`, `as_mut_ptr`: 100–330 bytes each) stay out of line, where
rustc inlines them all: before folding, smallvec's "inline or spilled"
test makes them look big to the inliner.

### arrayvec

| | ep default | ep ndebug | ep abort | rustc -O | rustc -Os | ndebug / -O |
|---|---|---|---|---|---|---|
| Mac | 5,556 | 5,472 | 3,704 | 1,860 | 2,060 | 2.94 |
| x86-64 | 6,104 | 6,012 | 4,064 | 2,150 | 2,112 | 2.80 |

| function | Mac ep | Mac rustc | x86 ep | x86 rustc |
|---|---|---|---|---|
| `main` | 4,224 | 1,612 | 4,572 | 1,888 |
| `<ArrayVec as Debug>::fmt` | 896 | 116 | 972 | 144 |
| `<CapacityError as Debug>::fmt` | 352 | 64 | 468 | 64 |

The derived `Debug` impls are 7.7× and 5.5× rustc's (below).

### hex

| | ep default | ep ndebug | ep abort | rustc -O | rustc -Os | ndebug / -O |
|---|---|---|---|---|---|---|
| Mac | 9,372 | 9,372 | 8,520 | 3,244 | 2,960 | 2.89 |
| x86-64 | 11,952 | 11,952 | 10,404 | 3,585 | 3,287 | 3.33 |

`main` 8,144 against 2,392 (Mac). Beyond hello world the program's
biggest growth on our side is `core`'s `<Result as Debug>::fmt` (4,376
against 252) and `<u8 as Debug>::fmt` (2,040): derived and hand-written
`Debug` in core, inlined builder chains.

### base64

| | ep default | ep ndebug | ep abort | rustc -O | rustc -Os | ndebug / -O |
|---|---|---|---|---|---|---|
| Mac | 38,896 | 38,608 | 34,992 | 6,048 | 5,700 | 6.38 |
| x86-64 | 40,508 | 40,196 | 36,544 | 7,226 | 6,466 | 5.56 |

| function | Mac ep | Mac rustc | x86 ep | x86 rustc |
|---|---|---|---|---|
| `encode`'s `inner` (with `internal_encode`) | 18,136 | 1,868 | 20,028 | 2,122 |
| `main` (with `internal_decode`) | 17,252 | 3,932 | 16,564 | 4,880 |
| `<DecodeError as Debug>::fmt` | 3,220 | 248 | 3,604 | 224 |

`inner::<GeneralPurpose>` is 4,032 instructions with 1,564 stack
accesses, 1,460 frame addresses and 1,020 vector instructions: the encoder's 8-byte and 6-byte chunks go
through slots, and `[u8; N]` temporaries are copied by `q` registers.

### semver

| | ep default | ep ndebug | ep abort | rustc -O | rustc -Os | ndebug / -O |
|---|---|---|---|---|---|---|
| Mac | 48,504 | 51,280 | 49,448 | 15,912 | 11,900 | 3.22 |
| x86-64 | 56,508 | 59,072 | 57,700 | 16,640 | 12,300 | 3.55 |

| function | Mac ep | Mac rustc | x86 ep | x86 rustc |
|---|---|---|---|---|
| `version_req` | 14,200 | 1,012 | 15,484 | 1,056 |
| `main` | 9,720 | 1,864 | 12,060 | 2,464 |
| `<Version>::parse` | 8,944 | 4 | 10,332 | 16 |
| `<Prerelease as PartialOrd>::partial_cmp` | 2,708 | — | 3,120 | — |
| `<VersionReq>::parse` | 2,572 | 4 | 3,032 | 16 |
| `<Error as Display>::fmt` | 3,212 | 712 | 3,856 | 720 |

The parser returns `Result<(T, Position, &str), Error>` from every
function. `version_req` is 3,550 instructions: 1,250 stack accesses,
1,102 frame addresses and 531 vector copies, against 253 instructions in
all for rustc (which keeps `comparator` out of line). rustc's `Version::parse`
is a 4-byte tail call into `FromStr`; ours has the parser inlined.
(Building semver without `debug_assertions` made it *bigger*, 51.3 KB
against 48.5 KB: inlining decisions moved.)

### heck

| | ep default | ep ndebug | ep abort | rustc -O | rustc -Os | ndebug / -O |
|---|---|---|---|---|---|---|
| Mac | 41,852 | 41,852 | 41,092 | 18,248 | 13,020 | 2.29 |
| x86-64 | 51,904 | 51,904 | 52,108 | 17,296 | 14,293 | 3.00 |

| function | Mac ep | Mac rustc |
|---|---|---|
| `main` (all seven conversions inlined) | 35,708 | 708 |
| the seven `to_*_case` (rustc: out of line) | — | 16,288 |
| `lowercase`, `uppercase`, `capitalize` | 5,464 | 1,252 |

Each conversion is used once, so inlining them all is fair; each is 2×
rustc's inside `main` (1,286 stack loads, 1,057 stores, 736 frame
addresses in `main`).

### memchr, unicode-width, strsim, fnv

These trap where a function couldn't be lowered, so their sizes say
little (below). On x86-64, where memchr's SSE2 and AVX2 paths do lower,
its code is 5.9× rustc's: `<Searcher>::new` 10,364 against 1,256,
`find_raw_sse2`/`avx2` of `Two` and `Three` 4–6 KB each where rustc's
whole `find_avx2` is 4.5 KB, and 9.1k stack accesses: the vector code is
lowered through memory, lane by lane (`docs/design/later.md`), and
`std_detect`'s `avx2` check is 9.6 KB against rustc's 1.8 KB
`detect_and_initialize`.

## Where we're biggest, by cause

| | Mac | x86-64 | for |
|---|---|---|---|
| Aggregates through memory: allocas the optimiser leaves, copied slot to slot, stores reloaded at once | 13.7k extra stack loads/stores of 31k extra instructions (+ most of 6.3k vector copies) | 23.3k of 42k | opt (SROA through memcpy, store-to-load forwarding: **opt-forward-and-remove-l**, PR #210), frame slots (**frames-share-alloca-slot**) |
| AArch64 frame addresses: 16-byte slots at offsets ≡ 8 (mod 16), half past 4 KB | 9.5k instructions, 17% | — | codegen: align 16-byte slots to 16, `add x15, sp, #hi, lsl #12` for the rest, smaller frames |
| `#[derive(Debug)]` as builder chains | `Debug` impls 3–13× | same | epitome-rustc2's derive (follow-up 1) |
| x86 vector code lane by lane | — | memchr 5.9×, `std_detect` 5× | x86-vector-lanes |
| Inlining: tiny accessors left out of line; once-used big functions duplicated into branches | smallvec accessors ~0.5 KB; ryu's `write_mantissa_long` ×4 | same | **inline-size-policy** |
| Missed CSE and constants remade per block | ryu: `x / 100` twice, `mov #10000` per block | 4.3k register moves vs 2.6k | opt CSE across blocks, remat |
| `brk` after calls that don't return | 238 instructions | 305 | codegen (opt-dead-cycles found the same) |

Cold placement (**cold-placement**) and function merging
(**merge-functions**) don't show here: these programs have little panic
code (immediate-abort saves 5% of the crates' code) and few identical
instances.

## What doesn't build or run, and why

Fixed on this branch (small):

- **itoa** didn't build: `i128::MAX_STR_LEN` in an array length, an
  associated constant of a trait impl on a primitive (`impl Integer for
  i128`); the evaluator searched only inherent impls (`consteval.rs`).
- **hex, base64**: `println!("{}", a == b)` failed to expand, "expected an
  expression, found `=`": `a`, `=`, `=` was taken for a named argument
  (`expand/format.rs`). `tests/run/std_hello.rs` now covers it.

Not fixed (proposed as follow-ups):

- **char case mapping is wrong**: `'Ä'.to_lowercase()` gives `Ä` (heck's
  only difference). core's `conversions::to_lower` finds nothing in
  `LOWERCASE_LUT` (a `static` struct of nested tables). A two-line
  program shows it.
- **`HashMap::entry`**: no layout for hashbrown's `Entry` (its type has an
  `{error}`), and `Group::static_empty` has "no layout for AlignedTags:
  BadConst" (`[Tag; Group::WIDTH]`). fnv's demo uses `get_mut`/`insert`
  instead and still traps; strsim's Damerau–Levenshtein uses a `HashMap`.
- **Closure argument coercion**: `.map(|s| Version::parse(s))` over
  `&&str` fails, "method call without a method" (semver's demo uses a
  loop).
- **Specialised and blanket trait methods**: `<[u8] as TryInto<[u8;
  8]>>` (base64's `read_u64`), `SpecFromIter` for `GenericShunt` (hex's
  `decode`), `StableSmallSortTypeImpl::small_sort_threshold` and
  `IsFreeze` (sorting a `Vec<Version>`): "built-in … on …", so they trap.
- **arrayvec**: `extend_from_iter`'s `ScopeExitGuard` closure has
  untyped captures.
- **unicode-width**: a `match` whose arms are `&` arrays of different
  lengths (`&EMOJI_MODIFIER_LEAF_0`, …) has no type; also
  `isize::from(add)` in a closure.
- **memchr on the Mac**: NEON intrinsics (`simd_splat`, `simd_eq`,
  `vshrn_n_u16` with a const generic, `umaxp`): no SIMD lowering on
  AArch64 yet.
- **Compiler stack**: fnv's program overflows epitome-rustc2's stack;
  `build.sh` sets `RUST_MIN_STACK=256M` (inline-hints found the same).

## Follow-ups, by size of win

1. AArch64: align 16-byte frame slots to 16 and make far offsets with one
   `add x15, sp, #hi, lsl #12`: up to 17% of the crates' code, and the
   same in `std` (`set_current_info`). Small and local to frame layout.
2. Aggregates through memory: SROA through `memcpy` between allocas and
   store-to-load forwarding (PR #210's direction), then slot sharing.
   The biggest, but already in hand.
3. `#[derive(Debug)]` via `Formatter::debug_{struct,tuple}_field{1..5}_finish`
   and `debug_struct_fields_finish`, as rustc's derive expands: shrinks
   every derived `Debug` in core, std and the crates 3–10×.
4. char case mapping (correctness), then `HashMap::entry` and
   `Group::WIDTH`-length arrays, which most real programs need.
5. Inlining: treat a function whose body folds to a few instructions
   once its argument is known (smallvec's accessors) as small; don't
   duplicate a big once-called function into several branches.
