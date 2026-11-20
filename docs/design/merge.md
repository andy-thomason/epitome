# Merging identical functions and constants

`opt/src/merge.rs` keeps identical functions once, and identical constant
globals once. It is the last pass of the pipeline: it runs after the
bottom-up passes and the interprocedural ones, once each function is as
simple as it will get and the dead ones are deleted, and the functions
and constants it merges are then deleted in turn. `--no-merge` turns it
off; in `--passes=` it is `merge`.

It plays the part of LLVM's `mergefunc` and `constmerge`, designed from
the Language Reference's rules for `unnamed_addr` and aliases and from
what rustc's output shows (rustc marks every function and every private
constant `unnamed_addr`, and asks LLVM to merge functions into aliases).

## When two functions are the same

Two functions are the same when one can stand for the other: the same
instructions in the same order, with the same flags, types, constants
and callees, up to the names and numbering of their locals and blocks;
and the same signature, calling convention, parameter, return and
function attributes (attribute groups expanded), section, alignment,
GC, prefix, prologue and personality. A call of the function itself is
the same in each, so two copies of a recursive function are the same.
Debug locations (`!dbg`) are left out of the comparison; other
attachments (`!range`, `!noalias`) are not.

Each function's body is written out in a canonical form: locals and
blocks are numbered in layout order (parameters first, then each block
and the results of its instructions), names are dropped, and each
instruction is written with its operands so renumbered. Constants and
types are interned in the module, so the same constant or type is the
same id, and pointers are opaque, so `Vec<u64>` and `Vec<usize>`, or
`Option<&T>` for different `T`, come out the same. Functions with the
same text are a class; the text is the hash key and the exact comparison
at once.

Only functions this module defines for good take part: `external`,
`internal` or `private` linkage (a `weak` or `linkonce` definition may be
replaced by the linker), nothing in a comdat, no DLL storage class,
nothing whose blocks have their addresses taken (`blockaddress`), no
`llvm.` intrinsics. Functions out of the module while it is optimised,
`"epitome-opaque"` and `"epitome-opt-level"="0"`, are declarations while
the pass runs, so they are left alone.

## What becomes of the others

In each class, the first function (in module order) whose address must
stay its own is kept, or else the first. Each other member `@d` of the
class, kept `@k`:

- **Its address is not significant** (`unnamed_addr`; or
  `local_unnamed_addr` on an internal or private function, which no
  other module can see): every use of `@d` (calls, constants such as
  vtables, metadata, other aliases) becomes a use of `@k`, and `@d`
  becomes an alias of `@k` (`@d = alias <fn type>, ptr @k`). An internal
  alias is then unused and deleted with the dead functions; an external
  one stays, so other modules still find `@d` by its name, at `@k`'s
  address.
- **Its address is significant** (no `unnamed_addr`, or
  `local_unnamed_addr` on an external function): only its calls become
  calls of `@k` (a call does not see the callee's address), and its body
  becomes a thunk, `tail call` of `@k` with its own arguments (and their
  attributes, which say how they are passed: `sret`, `zeroext`) and
  `ret`, so `@d`'s address stays its own. A body of two instructions or
  fewer is no bigger than the thunk, and a variadic function or one with
  arguments in memory (`byval`, `inalloca`, `preallocated`) cannot pass
  them on so simply: those keep their bodies, only their calls redirected.

Following the Language Reference, only `@d` needs to be `unnamed_addr`
for its uses to become uses of `@k`: comparing the two was unspecified
already.

Merging is an elective change of `@d` (autochop): one autochop refuses
is left as it was.

## Rounds, and constants

Merging callees makes callers the same (`<u64 as Display>::fmt` and
`<usize as Display>::fmt` merge, then whatever calls only them), and
merging constants makes functions that use them the same. So the pass
runs in rounds until one merges nothing (at most 16): each round merges
constants, then functions, then rewrites every use. Only functions a
rewrite changed are written out again.

A constant global merges with another when both are `private` or
`internal` and `unnamed_addr` (or `local_unnamed_addr`), constant, not
externally initialised, with the same type, initialiser (interned, so the
same id), section and other properties: string literals, promoted
constants, and vtables once the functions in them have merged. The one
kept takes the greatest alignment of its class. The others are then
unused, and deleted.

## epitome-rustc2

Merging only reaches what may be merged, so epitome-rustc2 marks every
function it defines `unnamed_addr`, and every private constant (string
literals, vtables, promoted constants), as rustc does: Rust promises
neither functions nor constants a unique address. `static`s are internal
or external, not private, and stay without it, as in rustc.

## Measurements

A `std` hello world (`epitome-rustc2 --std`), whose 204 functions are
what is left after inlining:

| | `__text`/`.text` | constants | file |
| --- | --- | --- | --- |
| AArch64 macOS, before | 133,156 | 10,500 (`__const`) | 250,640 |
| AArch64 macOS, after | 131,956 | 8,775 | 244,160 |
| x86-64 Linux, before | 111,159 | 7,617 (`.rodata`) | 187,008 |
| x86-64 Linux, after | 109,983 | 6,606 | 183,096 |

17 functions merged on the Mac (16 on x86-64), all as aliases: the
`Display` impls of `u64` and `usize`; `drop_in_place` of types with the
same drop (`Adapter<Stderr>`, `Adapter<StdoutLock>`, `Adapter<Vec<u8>>`;
`Vec<u8>` and `String`; `PoisonError<MutexGuard<T>>` and
`MutexGuard<T>`); `Write::flush` and `is_write_vectored` of `Vec<u8>` and
`Stderr`; `__rust_alloc` and friends with their `__rdl_` defaults; and
closures and their `call_once` shims. 69 constants merged on the Mac
(34 on x86-64). The pass takes about 20 ms (of about 1.7 s for the whole
pipeline).

That is every exact duplicate left: comparing the optimised functions
with globals' names blanked out finds five more pairs, each differing in
a constant it uses (`LowerHex` and `UpperHex` for `u32` differ in their
digits), which merging would need a parameter for.

Most duplicates in a Rust program are inlined before this pass sees
them, each copy into its callers; merging before inlining would let the
callers share a callee, and is a possible next step.
