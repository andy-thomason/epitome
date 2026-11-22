# Code size on real crates

How big epitome-rustc's programs are against rustc's, on twelve small
crates from crates.io, on the Mac (AArch64) and on x86-64 Linux, and where
the difference is. `bench/crate-sizes/` reproduces it.

The short answer, on main at 3347eba: the code a crate brings is **2.0×
rustc -O's on the Mac and 2.5× on x86-64** (the eight crates whose own
code lowers completely). Whole programs are much smaller than rustc's
(a hello world is 59 KB of code against 217 KB; rustc's precompiled `std`
carries its backtrace support).

The first measurement, on main at 6907286, found 3.2× and 3.7×, and three
causes: aggregates kept in allocas and copied slot to slot, AArch64 frame
addressing of 16-byte slots, and `#[derive(Debug)]` expanded as builder
chains. PRs #210 (alloca forwarding and SROA), #211 (frame slots shared
and ordered), #215 (inlining growth) and #218 (derive through
`debug_*_finish`) took out most of all three:

| | 6907286, Mac | 3347eba, Mac | 6907286, x86-64 | 3347eba, x86-64 | rustc -O, Mac / x86-64 |
|---|---|---|---|---|---|
| hello world's text | 135,400 | 58,972 | 116,599 | 64,038 | 216,892 / 241,827 |
| the eight crates' code | 199,396 | 124,096 | 234,680 | 158,540 | 61,940 / 64,194 |
| against rustc -O | 3.22× | **2.00×** | 3.66× | **2.47×** | |
| stack loads and stores, all twelve | 16,247 | 4,984 | 25,959 | 10,569 | 2,559 / 2,615 |
| AArch64 frame addresses in `x15` | 9,534 | 797 | — | — | 9 |

What is left is different in kind. On the Mac, stack traffic is now 2.4k
of the 10.9k extra instructions; the biggest part is **booleans made into
values**: compares turned into `cset`/`csel` bytes, zero-extended with
`uxtb`, combined with `orr` and tested again, where rustc branches on the
flags (`cset` +1.2k, `csel` +1.4k, `uxtb` +1.0k, `cmp` +2.1k, `orr`
+0.7k), with folds missed among them. Then inlining (each crate's `main`
is several times rustc's), x86-64's register moves and stack traffic, and
x86 vector code lowered lane by lane (memchr). Details and follow-ups
below.

## Method

Each crate has a demo program (`bench/crate-sizes/progs/<crate>_demo.rs`)
that exercises its main API and prints the results. The inputs go through
`std::env::args().len()` (1 when run), so neither compiler can fold the
program into its output: rustc -O folded the first version of itoa's
demo to 856 bytes. `build.sh` builds each program:

- epitome-rustc, `--std`, as it builds by default (**ep default**: the
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
  symbol. Ours are put down to crates by `epitome-rustc --fn-crates
  FILE` (new here: a symbol of an impl's method, `_R.Buffer..format…`,
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
cargo build --release -p epitome-rustc
cargo fetch --manifest-path bench/crate-sizes/fetch/Cargo.toml   # if crates are missing
bench/crate-sizes/build.sh             # target/crate-sizes; or build.sh OUT crate...
bench/crate-sizes/sizes.py --mix --new > sizes.md
bench/crate-sizes/mix.py target/crate-sizes/smallvec/ep-ndebug.exe try_grow --code
```

The numbers are from an Apple M1 and from mandelbrot.

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
- Two demos are written around epitome-rustc bugs (each place says so
  in a comment): semver's and fnv's.

## Results

### Mac (AArch64)

| crate | output | text over hello: ep ndebug | rustc -O | crate's code: ep default | ep ndebug | ep abort | rustc -O | rustc -Os | ndebug / rustc -O |
|---|---|---|---|---|---|---|---|---|---|
| itoa | ok | 6,456 | 4,096 | 8,288 | 7,648 | 7,580 | 3,340 | 3,200 | 2.29 |
| ryu | ok | 10,060 | 6,784 | 12,052 | 11,256 | 11,272 | 6,968 | 5,508 | 1.62 |
| fnv | traps | — | 14,080 | 1,268 | 1,268 | 1,284 | 2,860 | 1,332 | — |
| smallvec | ok | 42,656 | 11,328 | 20,188 | 18,216 | 17,168 | 6,320 | 5,920 | 2.88 |
| arrayvec | traps | 2,520 | 2,688 | 3,520 | 3,432 | 3,004 | 1,860 | 2,060 | 1.85 |
| hex | traps | 10,740 | 5,760 | 3,244 | 3,244 | 3,192 | 3,244 | 2,960 | 1.00 |
| base64 | traps | 29,632 | 7,616 | 17,812 | 17,268 | 16,308 | 6,048 | 5,700 | 2.86 |
| memchr | traps | — | 17,792 | 1,804 | 1,804 | 1,848 | 16,116 | 9,092 | — |
| semver | ok | 52,388 | 24,320 | 28,500 | 28,500 | 27,116 | 15,912 | 11,900 | 1.79 |
| unicode-width | traps | 13,956 | 10,880 | 3,716 | 3,716 | 3,640 | 10,576 | 8,076 | — |
| heck | wrong | 54,944 | 20,608 | 34,532 | 34,532 | 34,132 | 18,248 | 13,020 | 1.89 |
| strsim | traps | 10,236 | 29,824 | 13,960 | 13,960 | 14,092 | 10,780 | 6,880 | — |
| the eight | | 209,396 | 83,200 | 128,136 | 124,096 | 119,772 | 61,940 | 50,268 | **2.00** |

### x86-64 Linux

| crate | output | text over hello: ep ndebug | rustc -O | crate's code: ep default | ep ndebug | ep abort | rustc -O | rustc -Os | ndebug / rustc -O |
|---|---|---|---|---|---|---|---|---|---|
| itoa | ok | 8,200 | 3,379 | 11,000 | 9,860 | 9,764 | 3,312 | 3,080 | 2.98 |
| ryu | ok | 11,740 | 7,232 | 14,372 | 13,408 | 13,400 | 7,401 | 5,639 | 1.81 |
| fnv | traps | — | 16,752 | 1,600 | 1,600 | 1,588 | 4,080 | 1,504 | — |
| smallvec | ok | 53,488 | 11,840 | 25,660 | 24,020 | 22,464 | 6,584 | 6,031 | 3.65 |
| arrayvec | traps | 2,668 | 2,672 | 4,308 | 4,204 | 3,580 | 2,150 | 2,112 | 1.96 |
| hex | traps | 13,224 | 5,840 | 4,240 | 4,240 | 4,368 | 3,585 | 3,287 | 1.18 |
| base64 | traps | 36,984 | 8,960 | 23,272 | 22,948 | 20,400 | 7,226 | 6,466 | 3.18 |
| memchr | wrong | 95,460 | 21,280 | 83,176 | 76,720 | 74,464 | 19,492 | 14,231 | 3.94 |
| semver | ok | 64,396 | 26,832 | 36,516 | 36,388 | 34,160 | 16,640 | 12,300 | 2.19 |
| unicode-width | traps | 16,300 | 8,944 | 4,240 | 4,240 | 4,108 | 8,865 | 6,809 | — |
| heck | wrong | 67,712 | 19,376 | 43,472 | 43,472 | 43,420 | 17,296 | 14,293 | 2.51 |
| strsim | traps | 13,196 | 40,544 | 17,972 | 17,972 | 18,372 | 12,752 | 8,007 | — |
| the eight | | 258,412 | 86,131 | 162,840 | 158,540 | 151,556 | 64,194 | 53,208 | **2.47** |

("The eight" are itoa, ryu, smallvec, arrayvec, hex, base64, semver and
heck; memchr builds on x86-64, but prints nothing.)

`debug_assertions` cost 3% of the crates' code and immediate-abort saves
3–4% more: almost none of the excess is panic code. "Text over hello
world" is now well above the crate's code on our side (Mac 209 KB against
124 KB): `std`'s generic code grows with the crate, `core`'s `quicksort`
(6.5 KB for smallvec's `sort_unstable_by`, against rustc's 2.2 KB),
`<char>::escape_debug_ext` (4.7 KB) and `unicode_data`'s `lookup_slow`
(2.7 KB) for `{:?}` of strings, all in rustc's hello world already.

### The instruction mix

The crates' code, all twelve crates, ep ndebug against rustc -O:

| | insts | stack loads | stack stores | frame addresses | other memory | calls | branches | traps | vector | moves |
|---|---|---|---|---|---|---|---|---|---|---|
| Mac, ours | 36,211 | 2,312 | 2,672 | 797 | 1,452 | 910 | 4,693 | 192 | 1,035 | 1,507 |
| Mac, rustc | 25,357 | 1,382 | 1,177 | 9 | 1,916 | 890 | 4,351 | 0 | 1,088 | 1,740 |
| x86-64, ours | 57,911 | 4,876 | 5,693 | — | 2,018 | 1,405 | 5,895 | 275 | 2,557 | 4,716 |
| x86-64, rustc | 27,389 | 1,195 | 1,420 | — | 1,731 | 854 | 4,200 | 806 | 1,372 | 2,591 |

(x86-64's include memchr's 77 KB; without it the ratio is the 2.47 above.)

By opcode, the Mac's eight crates (31.0k instructions against 15.3k):

| opcode | ours | rustc | + |
|---|---|---|---|
| `mov` | 4,851 | 2,392 | 2,459 |
| `cmp` | 3,283 | 1,196 | 2,087 |
| `str` | 1,904 | 383 | 1,521 |
| `add` | 3,295 | 1,896 | 1,399 |
| `csel` | 1,486 | 118 | 1,368 |
| `cset` | 1,279 | 67 | 1,212 |
| `ldr` | 1,838 | 712 | 1,126 |
| `uxtb` | 995 | 0 | 995 |
| `b` | 2,988 | 2,004 | 984 |
| `orr` | 1,008 | 267 | 741 |
| `cbnz` / `cbz` / `tst` | 989 | 277 | 712 |
| `brk` | 152 | 0 | 152 |

## Per crate

The crate's code in each build, and the functions where ours is biggest
(Mac, with the same functions on x86-64). "—" is a function the other
compiler inlined.

### itoa

| | ep default | ep ndebug | ep abort | rustc -O | rustc -Os | ndebug / -O |
|---|---|---|---|---|---|---|
| Mac | 8,288 | 7,648 | 7,580 | 3,340 | 3,200 | 2.29 |
| x86-64 | 11,000 | 9,860 | 9,764 | 3,312 | 3,080 | 2.98 |

Everything is inlined into `main` on both sides (7.6 KB against 3.3 KB on
the Mac). The default build also keeps itoa's `debug_assert_eq!`s in
`udiv128`.

### ryu

| | ep default | ep ndebug | ep abort | rustc -O | rustc -Os | ndebug / -O |
|---|---|---|---|---|---|---|
| Mac | 12,052 | 11,256 | 11,272 | 6,968 | 5,508 | 1.62 |
| x86-64 | 14,372 | 13,408 | 13,400 | 7,401 | 5,639 | 1.81 |

| function | Mac ep | Mac rustc | x86 ep | x86 rustc |
|---|---|---|---|---|
| `main` | 6,060 | 1,140 | 7,168 | 1,568 |
| `<Buffer>::format_finite` | 5,196 | — | 6,240 | — |
| `format32` + `format64` + `write_mantissa_long` | — | 5,828 | — | 5,833 |

Nearly free of stack traffic from the start: the excess is selection and
folds. In `format_finite::<f64>`, `write_mantissa_long` is inlined into
each of `format64`'s branches (rustc calls it), `x / 100` is computed
twice (`umull; lsr #37` repeated) and constants (`#10000`, `#100`) are
made again in each block. `main` has `format_finite::<f32>` inlined
where rustc calls `format32`.

### smallvec

| | ep default | ep ndebug | ep abort | rustc -O | rustc -Os | ndebug / -O |
|---|---|---|---|---|---|---|
| Mac | 20,188 | 18,216 | 17,168 | 6,320 | 5,920 | 2.88 |
| x86-64 | 25,660 | 24,020 | 22,464 | 6,584 | 6,031 | 3.65 |

| function | Mac ep | Mac rustc | x86 ep | x86 rustc |
|---|---|---|---|---|
| `main` | 11,772 | 2,260 | 15,992 | 2,672 |
| `<SmallVec>::try_grow` | 3,292 | 952 | 4,720 | 992 |
| `<SmallVec as Debug>::fmt` | 2,248 | 412 | 2,456 | 472 |
| `<SmallVec as Deref>::deref` | 256 | — | 224 | — |
| `<SmallVec>::reserve_one_unchecked` | 392 | 280 | 400 | 288 |

`try_grow::<[u32; 4]>` went from 1,104 instructions (479 stack accesses,
a 2.5 KB frame) to 412, against rustc's 111; it has 26 stack accesses
now. What is left is booleans and missed folds:

```
add  x1, x20, #0xc         ; a pointer into *self: never null
cmp  x1, xzr
csel x23, xzr, x1, eq      ; ... tested for null anyway
cmp  x23, xzr
cset w1, ne
cmp  x23, #0x1
cset w2, eq
orr  w1, w1, w2            ; x != 0 || x == 1, which is x != 0
tbz  w1, #0, ...
```

and `Layout::array::<u32>(n)`'s overflow test made with `umulh` and three
`csel`s, where `n >> 62` would do. `<SmallVec as Debug>::fmt` is
`debug_list().entries(self.iter())`: the iterator inlined. The small
accessors (`deref`, `infallible`) still stay out of line.

### arrayvec

| | ep default | ep ndebug | ep abort | rustc -O | rustc -Os | ndebug / -O |
|---|---|---|---|---|---|---|
| Mac | 3,520 | 3,432 | 3,004 | 1,860 | 2,060 | 1.85 |
| x86-64 | 4,308 | 4,204 | 3,580 | 2,150 | 2,112 | 1.96 |

| function | Mac ep | Mac rustc | x86 ep | x86 rustc |
|---|---|---|---|---|
| `main` | 2,640 | 1,612 | 3,352 | 1,888 |
| `<ArrayVec as Debug>::fmt` | 660 | 116 | 720 | 144 |
| `<CapacityError as Debug>::fmt` | 132 | 64 | 132 | 64 |

### hex

| | ep default | ep ndebug | ep abort | rustc -O | rustc -Os | ndebug / -O |
|---|---|---|---|---|---|---|
| Mac | 3,244 | 3,244 | 3,192 | 3,244 | 2,960 | 1.00 |
| x86-64 | 4,240 | 4,240 | 4,368 | 3,585 | 3,287 | 1.18 |

At parity in its own code (`main` 2,264 against 2,392 on the Mac), but
`decode` traps in `std`'s `Vec::from_iter` (below), so `from_hex`'s code
is missing on our side.

### base64

| | ep default | ep ndebug | ep abort | rustc -O | rustc -Os | ndebug / -O |
|---|---|---|---|---|---|---|
| Mac | 17,812 | 17,268 | 16,308 | 6,048 | 5,700 | 2.86 |
| x86-64 | 23,272 | 22,948 | 20,400 | 7,226 | 6,466 | 3.18 |

| function | Mac ep | Mac rustc | x86 ep | x86 rustc |
|---|---|---|---|---|
| `main` (with `internal_decode`) | 9,256 | 3,932 | 11,960 | 4,880 |
| `encode`'s `inner` (with `internal_encode`) | 6,888 | 1,868 | 9,384 | 2,122 |
| `<GeneralPurpose>::new` | 644 | — | 1,152 | — |

`inner::<GeneralPurpose>` went from 4,032 instructions (1,564 stack
accesses, 1,460 frame addresses, 1,020 vector copies of the 8-byte and
6-byte chunks) to 1,331, with 214 stack accesses and no vector copies.

### semver

| | ep default | ep ndebug | ep abort | rustc -O | rustc -Os | ndebug / -O |
|---|---|---|---|---|---|---|
| Mac | 28,500 | 28,500 | 27,116 | 15,912 | 11,900 | 1.79 |
| x86-64 | 36,516 | 36,388 | 34,160 | 16,640 | 12,300 | 2.19 |

| function | Mac ep | Mac rustc | x86 ep | x86 rustc |
|---|---|---|---|---|
| `main` | 7,632 | 1,864 | 10,028 | 2,464 |
| `version_req` | 6,556 | 1,012 | 8,856 | 1,056 |
| `<Version>::parse` | 3,268 | 4 | 4,296 | 16 |
| `<Prerelease as PartialOrd>::partial_cmp` | 2,256 | — | 2,828 | — |
| `<Version as Display>::fmt` | 1,996 | 48 | 2,392 | 32 |
| `<VersionReq>::parse` | 1,624 | 4 | 2,104 | 16 |

The parser returns `Result<(T, Position, &str), Error>` from every
function. `version_req` went from 3,550 instructions (1,250 stack
accesses, 1,102 frame addresses) to 1,639 (119 stack accesses), against
253 for rustc, which keeps `comparator` out of line. rustc's
`Version::parse` is a tail call into `FromStr`; ours has the parser
inlined.

### heck

| | ep default | ep ndebug | ep abort | rustc -O | rustc -Os | ndebug / -O |
|---|---|---|---|---|---|---|
| Mac | 34,532 | 34,532 | 34,132 | 18,248 | 13,020 | 1.89 |
| x86-64 | 43,472 | 43,472 | 43,420 | 17,296 | 14,293 | 2.51 |

| function | Mac ep | Mac rustc | x86 ep | x86 rustc |
|---|---|---|---|---|
| `main` (all seven conversions inlined) | 31,188 | 708 | 39,972 | 1,024 |
| the seven `to_*_case` (rustc: out of line) | — | 16,288 | — | |
| `lowercase` | 1,772 | 560 | 1,840 | 592 |

Each conversion is used once, so inlining them all is fair; `main` is
7,797 instructions with 1,358 branches against rustc's ~4,250 and ~940
for the same work, and on x86-64 1,076 register moves against 395.

### memchr, unicode-width, strsim, fnv

These trap where a function couldn't be lowered, so their sizes say
little (below). On x86-64, where memchr's SSE2 and AVX2 paths do lower,
its code is 3.9× rustc's (5.9× before): its vector code is lowered
through memory, lane by lane (`docs/design/later.md`).

## Where we're biggest, by cause

| | Mac | x86-64 | for |
|---|---|---|---|
| Booleans as values: `cmp; cset; uxtb; orr; tst; b` where rustc branches on flags, with missed folds (`x != 0 \|\| x == 1`, a null test of `p + 12`, overflow of `n * 4` by `umulh`) | `cset`/`csel`/`uxtb`/`orr` +4.3k, `cmp` +2.1k, of +15.7k | similar | opt (combine: `or` of compares, `nonnull` of inbounds GEPs, `umul.with.overflow` by a power of 2), isel (fuse compare and branch through `zext`/`or`) |
| Stack traffic still left | +2.4k loads and stores | +8.0k | opt (forwarding across blocks), x86 regalloc |
| Inlining: once-used functions duplicated into branches, small accessors kept out of line; `main` 3–40× rustc's where rustc keeps the crate's functions out of line | ryu, heck, semver | same | inliner |
| x86-64 register moves | — | 4.7k against 2.6k | regalloc coalescing |
| x86 vector code lane by lane | — | memchr 3.9× | x86-vector-lanes |
| `std` generics instantiated for the crate | `quicksort` 6.5 KB against 2.2 KB | | the same causes, in core |
| `brk` after calls that don't return | 152 | 275 | codegen |

## What doesn't build or run, and why

Fixed on this branch (small):

- **itoa** didn't build: `i128::MAX_STR_LEN` in an array length, an
  associated constant of a trait impl on a primitive (`impl Integer for
  i128`); the evaluator searched only inherent impls (`consteval.rs`).
- `-C debug-assertions=yes|no`, so the comparison with rustc -O is like
  for like, and `--fn-crates FILE`.

(The same branch fixed `println!("{}", a == b)`, which #212 also fixed on
main; main's fix is kept.)

Not fixed (follow-ups):

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
  `decode`): "built-in … on …", so they trap.
- **arrayvec**: `extend_from_iter`'s `ScopeExitGuard` closure has
  untyped captures.
- **unicode-width**: a `match` whose arms are `&` arrays of different
  lengths (`&EMOJI_MODIFIER_LEAF_0`, …) has no type; also
  `isize::from(add)` in a closure.
- **memchr on the Mac**: NEON intrinsics (`simd_splat`, `simd_eq`,
  `vshrn_n_u16` with a const generic, `umaxp`): no SIMD lowering on
  AArch64 yet.
- **Compiler stack**: fnv's program overflows epitome-rustc's stack;
  `build.sh` sets `RUST_MIN_STACK=256M`.

## Follow-ups, by size of win

1. Booleans: fold `or`/`and` of compares of one value, `icmp ne (gep
   inbounds nonnull p, k), null`, and `umul.with.overflow` by a power of
   two; select compare-and-branch through `zext i1`/`or i1` rather than
   making bytes. About 6k of the Mac's 15.7k extra instructions are this
   shape.
2. Inlining: don't duplicate a big function called once into several
   branches (ryu), keep a crate's large once-used functions (`to_*_case`,
   `comparator`) out of line at `-O` as rustc does when that is smaller,
   and inline accessors whose bodies fold away.
3. x86-64: the remaining stack traffic (8k extra loads and stores) and
   register moves (2.1k extra).
4. Correctness: char case mapping, then `HashMap::entry` and
   `Group::WIDTH`-length arrays, which most real programs need.
