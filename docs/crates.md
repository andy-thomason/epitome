# Crates from the top 1,000

Progress towards building the 1,000 most downloaded crates on crates.io
(by all-time downloads, as listed on 2026-11-03) with Epitome, through
`epitome-rustc`.

## Where we are

- **224** of the top 1,000 are in the README's [Crates that build](../README.md#crates-that-build)
  (their own tests pass on an AArch64 Mac).
- **Batch 1** (2026-11-04, x86-64 Linux): the next 100 by rank that are not
  in that list, from syn (#2) to mime (#179). Results below, re-run
  after the fixes for [1](#1-storing-a-16-x-i1) and
  [2](#2-x86-intrinsics-not-supported-yet).
- So far **293** of the top 1,000 are known to build and pass: the 224, and
  batch 1's 69 (51 that pass and 18 with no tests).
- **Next**: work through [To fix](#to-fix), then the next 100 by rank.

| Result | Crates |
|---|---|
| **pass**: tests as under rustc, every crate in the build compiled by Epitome | 51 |
| **pass (no tests)**: builds, as under rustc, but has no tests to run | 18 |
| **falls back**: tests as under rustc, but Epitome could not compile some crate in the build, which kept rustc's object | 5 |
| **fail**: tests differ from rustc's | 1 |
| **untestable**: the tests do not build under plain rustc either | 25 |

## How each crate is tested

On x86-64 Linux with Rust 1.99.1, in the crate's published package:
`cargo test --release` with plain rustc, then again with
`RUSTC_WRAPPER=epitome-rustc`, both with `RUSTFLAGS=--cap-lints=warn` (old
crates deny warnings that new rustc gives). A crate passes when its test
counts match rustc's (its own failing tests are its own) and the
`epitome-rustc` log shows no `ours failed` for any crate in the build.

## To fix

The crates Epitome could not compile in batch 1 (each kept rustc's object),
by cause, so that one fix can close several. For each: the versions seen
in the batch's lock files, the function Epitome stopped at, and the crates
of the batch whose builds included it. Tick a crate off when its IR
compiles and the batch crates that use it still pass.

### 1. Storing a `<16 x i1>`

`no instruction for the store in … (v16i1 …)`. The most widespread: every
tokio, http and hashbrown build hit it. Fixed: a vector of booleans in
memory is now packed bits, lane k bit k, as LLVM lays it out; all of
these compile, and the crates that used them were re-run.

- [x] `tokio` 1.42.0, 1.53.1, 1.53.2, 1.54.0, 1.54.1: `runtime::blocking::pool::BlockingPool::shutdown`. In: h2, hyper, hyper-rustls, reqwest, syn, tokio, tokio-rustls, tokio-util, tower, webpki-roots
- [x] `http` 0.2.12, 1.0.0, 1.2.0, 1.4.2, 1.5.0: `extensions::Extensions::extend`. In: h2, http, http-body, hyper, hyper-rustls, reqwest, syn, tower, tower-service
- [x] `hashbrown` 0.12.3, 0.14.5, 0.16.0, 0.17.0, 0.17.1: `raw::RawIterHashInner::new`, `raw::RawIterHashIndices::new`, `RawIterHashInner::next`. In: generic-array, h2, hyper-rustls, regex, regex-automata, reqwest, rustix, syn
- [x] `criterion` 0.3.6, 0.5.1, 0.8.2: `html::Html::final_summary`. In: generic-array, litemap, tracing, unicode-ident
- [x] `similar` 3.1.1, 3.2.0: `algorithms::utils::unique::<OffsetLookup>`, also inlined into `similar_asserts` 2.0.0 and `snapbox` 1.2.2. In: chrono, clap_builder, toml_datetime
- [x] `darling_core` 0.24.1: `codegen::trait_impl::TraitImpl::used_type_params`. In: darling, darling_core
- [x] `futures` 0.1.31, 0.3.34: `future::shared::Notifier::notify`. In: futures, futures-util
- [x] `oid_registry` 0.8.1: `OidRegistry::iter`. In: rustls, webpki-roots
- [x] `clap` 2.34.0: `args::arg_matcher::ArgMatcher::iter`. In: tracing
- [x] `tester` 0.9.1: `run_tests::get_timed_out_tests`. In: idna

### 2. x86 intrinsics not supported yet

Fixed: x86's own intrinsics now have entries in the machine description,
found from what LLVM's llc makes of each (see
`codegen/targets/x86_64/discover_intrinsics.py`), and are checked against
llc by `tests/intrinsics.rs` (`x86.txt`); those that are more than one
instruction (`rdtsc`, `xgetbv`, the carries) are rewritten before code
generation. All of these compile, and the crates that used them were
re-run.


- [x] `rand_chacha` 0.3.1, 0.9.0: `llvm.x86.avx.vzeroupper`, in `guts::refill_wide::impl_avx`. In: generic-array, h2, litemap, mio, nix, nom, time, tokio, tower
- [x] `aho_corasick` 0.7.6, 0.7.20, 1.1.3, 1.1.4, 1.1.5: `llvm.x86.ssse3.pshuf.b.128` and `llvm.x86.avx2.pshuf.b`, in `packed::teddy` (Slim SSSE3 and AVX2 searchers). In: http, hyper, regex, regex-automata, reqwest, rustls, time, tower, zerocopy
- [x] `base64` 0.22.1, 0.23.1: `llvm.x86.avx2.pshuf.b`, in `engine::simd::avx2::decode_bulk`. In: reqwest, syn
- [x] `simdutf8` 0.1.5: `llvm.x86.avx2.pshuf.b`, in `implementation::x86::avx2::validate_utf8_basic`. In: litemap
- [x] `chacha20` 0.10.0, 0.10.2 and `core_detect` 1.0.0: `llvm.x86.xgetbv`, in `core::arch::x86::xsave::_xgetbv`. In: http, reqwest, syn, tokio, unicode-ident
- [x] `crc32fast` 1.5.0, 1.5.1, 1.5.2: `llvm.x86.pclmulqdq`, in `specialized::pclmulqdq::calculate`. In: proc-macro2, reqwest, rustix, syn
- [x] `simd_adler32` 0.3.10: `llvm.x86.sse2.psad.bw`, in `imp::sse2::update`. In: proc-macro2, reqwest, rustix, syn
- [x] `httparse` 1.10.1: `llvm.x86.avx.ldu.dq.256`, in `simd::avx2::match_uri_vectored`. In: hyper-rustls, reqwest, syn
- [x] `num_bigint` 0.4.8: `llvm.x86.subborrow.64`, in `BigUint::modpow`. In: rustls, webpki-roots
- [x] `sha2` 0.11.0: `llvm.x86.sha256rnds2`, in `sha256::x86_sha::compress`. In: digest
- [x] `aes` 0.8.4: `llvm.x86.aesni.aeskeygenassist`, in `_mm_aeskeygenassist_si128`. In: generic-array
- [x] `divan` 0.1.16: an intrinsic whose name the log cuts off (its timer reads the clock with `rdtscp`), in `FineDuration` initialisation for `Timer::sample_loop_overhead`. In: anstream, anstyle-parse

### 3. A vector of pointers to globals

- [ ] `h2` 0.4.20's examples and tests `akamai`, `client`, `server`: `proto::connection::DynConnection::recv_frame`. In: h2
- [ ] `hyper_util` 0.1.20, 0.1.21: `client::legacy::client::origin_form`. In: hyper-rustls, reqwest, syn
- [ ] `reqwest` 0.13.5: `get::<Url>` and `connect::with_timeout`. In: reqwest
- [ ] `rcgen` 0.14.10: `KeyPair::try_from(PrivateKeyDer)`. In: rustls

### 4. Others

- [ ] `encoding_rs` 0.8.40, 0.8.42: a cast from a vector narrower than a register to a split one, in `iso_2022_jp::is_mapped_for_two_byte_encode`. In: reqwest, syn
- [x] cpufeatures 0.3.1's test `x86`: Intel-syntax inline assembly (core's `cpuid` sequence) is written with AT&T registers, `mov %r8, rbx` and `xchg %r8, rbx`, which do not assemble. In: cpufeatures. Fixed: inline
  assembly in Intel's syntax is turned into AT&T's (`codegen/src/intel.rs`).
- [ ] autocfg 1.5.1's test `wrappers` and idna 1.1.0's test `tests`: `0 objects for the crate in the archive, not one`. In: autocfg, idna

- [ ] A `select` by a vector of booleans that came from bits (a load of a
  `<4 x i1>`, or a `bitcast` of an `i4`) fails with `a select of split
  vectors with a legal result`, on both targets. No crate in batch 1 hits
  it; found while testing the fix for 1.

### 5. Tests that differ

- [ ] clap_builder 4.6.7: its library tests die with SIGILL under Epitome (81 pass under rustc).

## Batch 1

| Rank | Crate | Version | rustc: pass / fail | Epitome: pass / fail | Result | Epitome could not compile |
|---:|---|---|---|---|---|---|
| 2 | syn | 3.0.9 | no build | no build | untestable | `encoding_rs`, `hyper_util`; its `syn-test-suite` (lib) does not parse: `expected item, found @` |
| 3 | getrandom | 0.4.3 | 16 / 0 | 16 / 0 | pass |  |
| 8 | quote | 1.0.47 | 63 / 0 | 63 / 0 | pass |  |
| 9 | proc-macro2 | 1.0.107 | 71 / 1 | 71 / 1 | pass |  |
| 11 | windows-sys | 0.61.2 | 0 / 0 | 0 / 0 | pass (no tests) |  |
| 14 | thiserror-impl | 2.0.21 | 0 / 0 | 0 / 0 | pass (no tests) |  |
| 19 | serde | 1.0.229 | 0 / 0 | 0 / 0 | pass (no tests) |  |
| 20 | unicode-ident | 1.0.27 | 0 / 1 | 0 / 1 | pass |  |
| 23 | serde_derive | 1.0.229 | 2 / 1 | 2 / 1 | pass |  |
| 26 | autocfg | 1.5.1 | 25 / 0 | 25 / 0 | falls back | `wrappers` |
| 29 | cc | 1.11.0 | 76 / 0 | 76 / 0 | pass |  |
| 30 | regex-automata | 0.4.18 | no build | no build | untestable | module `fuzz` missing from the package |
| 31 | socket2 | 0.6.5 | 17 / 0 | 17 / 0 | pass |  |
| 33 | rustix | 1.1.6 | 17 / 0 | 17 / 0 | pass |  |
| 34 | regex | 1.13.1 | no build | no build | untestable | module `fuzz` missing from the package |
| 36 | digest | 0.11.3 | 3 / 1 | 3 / 1 | pass |  |
| 37 | clap | 4.6.7 | 2 / 0 | 2 / 0 | pass |  |
| 40 | linux-raw-sys | 0.12.1 | 0 / 0 | 0 / 0 | pass (no tests) |  |
| 41 | parking_lot_core | 0.10.0 | 16 / 0 | 16 / 0 | pass |  |
| 42 | windows_x86_64_msvc | 0.53.1 | 0 / 0 | 0 / 0 | pass (no tests) |  |
| 46 | mio | 1.2.4 | 5 / 0 | 5 / 0 | pass |  |
| 47 | windows_x86_64_gnu | 0.53.1 | 0 / 0 | 0 / 0 | pass (no tests) |  |
| 48 | cpufeatures | 0.3.1 | 3 / 0 | 3 / 0 | pass |  |
| 49 | pin-project-lite | 0.2.17 | no build | no build | untestable | crate `macrotest` not a dependency |
| 50 | windows_aarch64_msvc | 0.53.1 | 0 / 0 | 0 / 0 | pass (no tests) |  |
| 51 | windows_i686_msvc | 0.53.1 | 0 / 0 | 0 / 0 | pass (no tests) |  |
| 52 | windows-targets | 0.53.5 | 0 / 1 | 0 / 1 | pass |  |
| 53 | http | 1.5.0 | 409 / 0 | 409 / 0 | pass |  |
| 54 | windows_i686_gnu | 0.53.1 | 0 / 0 | 0 / 0 | pass (no tests) |  |
| 60 | windows_aarch64_gnullvm | 0.53.1 | 0 / 0 | 0 / 0 | pass (no tests) |  |
| 61 | windows_x86_64_gnullvm | 0.53.1 | 0 / 0 | 0 / 0 | pass (no tests) |  |
| 63 | tokio | 1.54.1 | no build | no build | untestable | needs `--features full` |
| 64 | lazy_static | 1.5.1 | 9 / 1 | 9 / 1 | pass |  |
| 67 | idna | 1.1.0 | 12649 / 155 | 12649 / 155 | falls back | `tests` |
| 69 | futures-core | 0.3.34 | 0 / 2 | 0 / 2 | pass |  |
| 71 | zerocopy | 0.8.63 | 135 / 5 | 135 / 5 | pass |  |
| 74 | futures-util | 0.3.34 | 2 / 153 | 2 / 153 | pass |  |
| 75 | futures-task | 0.3.34 | 1 / 2 | 1 / 2 | pass |  |
| 76 | hyper | 1.12.0 | 10 / 1 | 10 / 1 | pass |  |
| 77 | rustls | 0.23.46 | no build | no build | untestable | `rcgen`; test data missing from the package |
| 83 | generic-array | 1.4.5 | 35 / 0 | 35 / 0 | pass |  |
| 84 | time | 0.3.55 | no build | no build | untestable | attribute `rstest` not found (needs features) |
| 85 | http-body | 1.1.0 | 4 / 0 | 4 / 0 | pass |  |
| 87 | futures-sink | 0.3.34 | 0 / 0 | 0 / 0 | pass (no tests) |  |
| 89 | futures-channel | 0.3.34 | no build | no build | untestable | crate `futures` not a dependency |
| 91 | clap_lex | 1.1.1 | 17 / 0 | 17 / 0 | pass |  |
| 92 | tracing-core | 0.1.36 | 19 / 1 | 19 / 1 | pass |  |
| 95 | tracing | 0.1.44 | no build | no build | untestable | crate `tracing_mock` not published |
| 96 | toml_datetime | 1.1.2+spec-1.1.0 | 0 / 0 | 0 / 0 | pass |  |
| 97 | futures-io | 0.3.34 | 0 / 0 | 0 / 0 | pass (no tests) |  |
| 100 | tempfile | 3.27.0 | 114 / 0 | 114 / 0 | pass |  |
| 103 | chrono | 0.4.45 | 280 / 8 | 280 / 8 | pass |  |
| 104 | h2 | 0.4.20 | 62 / 382 | 62 / 382 | falls back | `akamai`, `client`, `server` |
| 105 | nix | 0.31.3 | no build | no build | untestable | needs features (`nix::sys::aio`) |
| 106 | tokio-macros | 2.7.2 | 39 / 0 | 39 / 0 | pass |  |
| 107 | rustls-webpki | 0.103.16 | no build | no build | untestable | module `test_utils` missing from the package |
| 110 | futures-macro | 0.3.34 | 0 / 0 | 0 / 0 | pass (no tests) |  |
| 112 | rustversion | 1.0.23 | 7 / 0 | 7 / 0 | pass |  |
| 114 | tokio-util | 0.7.20 | no build | no build | untestable | needs features (`tokio_util::codec`) |
| 115 | futures | 0.3.34 | no build | no build | untestable | crate `futures_test` not a dependency |
| 116 | tokio-rustls | 0.26.6 | no build | no build | untestable | `tests/utils.rs` missing from the package |
| 117 | pkg-config | 0.3.34 | 34 / 0 | 34 / 0 | pass |  |
| 118 | clap_builder | 4.6.7 | 81 / 1 | 0 / 0 | fail | SIGILL in the library tests |
| 119 | clap_derive | 4.6.7 | 0 / 0 | 0 / 0 | pass (no tests) |  |
| 120 | webpki-roots | 1.0.11 | no build | no build | untestable | crate `webpki_ccadb` not a dependency |
| 122 | tracing-attributes | 0.1.31 | no build | no build | untestable | crate `tracing_mock` not published |
| 123 | darling_core | 0.24.1 | 130 / 0 | 130 / 0 | pass |  |
| 125 | darling_macro | 0.24.1 | 0 / 0 | 0 / 0 | pass (no tests) |  |
| 126 | darling | 0.24.1 | 124 / 0 | 124 / 0 | pass |  |
| 127 | futures-executor | 0.3.34 | no build | no build | untestable | crate `futures` not a dependency |
| 129 | reqwest | 0.13.5 | 45 / 1 | 45 / 1 | falls back | `encoding_rs`, `hyper_util`, `reqwest` |
| 130 | wasi | 0.14.7+wasi-0.2.4 | no build | no build | untestable | its tests do not link on Linux |
| 131 | ring | 0.17.14 | no build | no build | untestable | test data missing from the package |
| 132 | signal-hook-registry | 1.4.8 | 15 / 0 | 15 / 0 | pass |  |
| 134 | synstructure | 0.14.0 | 42 / 0 | 42 / 0 | pass |  |
| 135 | nom | 8.0.0 | no build | no build | untestable | crate `nom_language` not a dependency |
| 138 | redox_syscall | 0.9.5 | 1 / 0 | 1 / 0 | pass |  |
| 143 | time-macros | 0.2.32 | 0 / 0 | 0 / 0 | pass (no tests) |  |
| 144 | anstream | 1.0.0 | 30 / 0 | 30 / 0 | pass |  |
| 148 | tower | 0.5.3 | no build | no build | untestable | `tracing::subscriber::DefaultGuard` not found (needs features) |
| 149 | serde_core | 1.0.229 | 48 / 21 | 48 / 21 | pass |  |
| 150 | anstyle-parse | 1.0.0 | 2 / 1 | 2 / 1 | pass |  |
| 151 | iana-time-zone | 0.1.65 | 8 / 0 | 8 / 0 | pass |  |
| 153 | async-trait | 0.1.92 | 9 / 0 | 9 / 0 | pass |  |
| 154 | time-core | 0.1.9 | 1 / 3 | 1 / 3 | pass |  |
| 156 | anstyle-query | 1.1.5 | 3 / 0 | 3 / 0 | pass |  |
| 158 | hyper-rustls | 0.27.10 | 7 / 0 | 7 / 0 | falls back | `hyper_util` |
| 162 | sync_wrapper | 1.0.2 | 6 / 0 | 6 / 0 | pass |  |
| 163 | tower-service | 0.3.3 | 4 / 0 | 4 / 0 | pass |  |
| 164 | strum_macros | 0.28.0 | 12 / 9 | 12 / 9 | pass |  |
| 165 | want | 0.3.2 | no build | no build | untestable | uses `#![feature]`, nightly only |
| 166 | try-lock | 0.2.5 | 2 / 0 | 2 / 0 | pass |  |
| 167 | tinyvec_macros | 0.1.1 | 0 / 0 | 0 / 0 | pass (no tests) |  |
| 168 | strum | 0.28.0 | 0 / 4 | 0 / 4 | pass |  |
| 172 | displaydoc | 0.2.7 | 9 / 0 | 9 / 0 | pass |  |
| 173 | yoke | 0.8.4 | 57 / 2 | 57 / 2 | pass |  |
| 175 | zerocopy-derive | 0.8.63 | no build | no build | untestable | crate `zerocopy_renamed` not a dependency |
| 177 | litemap | 0.8.3 | no build | no build | untestable | crate `icu_locale_core` not a dependency |
| 178 | yoke-derive | 0.8.4 | no build | no build | untestable | crate `zerovec` not a dependency |
| 179 | mime | 0.3.17 | 16 / 1 | 16 / 1 | pass |  |
