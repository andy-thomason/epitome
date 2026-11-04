# Crates from the top 1,000

Progress towards building the 1,000 most downloaded crates on crates.io
(by all-time downloads, as listed on 2026-11-03) with Epitome, through
`epitome-rustc`.

## Where we are

- **224** of the top 1,000 are in the README's [Crates that build](../README.md#crates-that-build)
  (their own tests pass on an AArch64 Mac).
- **Batch 1** (2026-11-04, x86-64 Linux): the next 100 by rank that are not
  in that list, from syn (#2) to mime (#179). Results below; no Epitome
  changes were made for them.
- So far **274** of the top 1,000 are known to build and pass: the 224, and
  batch 1's 50 (32 that pass and 18 with no tests).
- **Next**: batch 1's failures below, then the next 100 by rank.

| Result | Crates |
|---|---|
| **pass**: tests as under rustc, every crate in the build compiled by Epitome | 32 |
| **pass (no tests)**: builds, as under rustc, but has no tests to run | 18 |
| **falls back**: tests as under rustc, but Epitome could not compile some crate in the build, which kept rustc's object | 24 |
| **fail**: tests differ from rustc's | 1 |
| **untestable**: the tests do not build under plain rustc either | 25 |

## How each crate is tested

On x86-64 Linux with Rust 1.99.1, in the crate's published package:
`cargo test --release` with plain rustc, then again with
`RUSTC_WRAPPER=epitome-rustc`, both with `RUSTFLAGS=--cap-lints=warn` (old
crates deny warnings that new rustc gives). A crate passes when its test
counts match rustc's (its own failing tests are its own) and the
`epitome-rustc` log shows no `ours failed` for any crate in the build.

## Epitome's failures

What stopped Epitome compiling a crate (it kept rustc's object), with the
crates of the batch whose builds hit it.

- **store of <16 x i1>**
  - `clap`, in: tracing
  - `criterion`, in: generic-array, litemap, tracing, unicode-ident
  - `darling_core`, in: darling, darling_core
  - `futures`, in: futures, futures-util
  - `hashbrown`, in: generic-array, h2, hyper-rustls, regex, regex-automata, reqwest, rustix, syn
  - `http`, in: h2, http, http-body, hyper, hyper-rustls, reqwest, syn, tower, tower-service
  - `oid_registry`, in: rustls, webpki-roots
  - `similar`, in: chrono, clap_builder, toml_datetime
  - `similar_asserts`, in: chrono
  - `snapbox`, in: clap_builder, toml_datetime
  - `tester`, in: idna
  - `tokio`, in: h2, hyper, hyper-rustls, reqwest, syn, tokio, tokio-rustls, tokio-util, tower, webpki-roots
- **intrinsic llvm.x86.avx.vzeroupper**
  - `rand_chacha`, in: generic-array, h2, litemap, mio, nix, nom, time, tokio, tower
- **vector of pointers to globals**
  - `akamai`, in: h2
  - `client`, in: h2
  - `hyper_util`, in: hyper-rustls, reqwest, syn
  - `rcgen`, in: rustls
  - `reqwest`, in: reqwest
  - `server`, in: h2
- **intrinsic llvm.x86.avx2.pshuf.b**
  - `aho_corasick`, in: time, tower, zerocopy
  - `base64`, in: reqwest, syn
  - `simdutf8`, in: litemap
- **intrinsic llvm.x86.ssse3.pshuf.b.128**
  - `aho_corasick`, in: http, hyper, regex, regex-automata, reqwest, rustls
- **intrinsic llvm.x86.xgetbv**
  - `chacha20`, in: http, tokio, unicode-ident
  - `core_detect`, in: reqwest, syn, tokio
- **intrinsic llvm.x86.pclmulqdq**
  - `crc32fast`, in: proc-macro2, reqwest, rustix, syn
- **intrinsic llvm.x86.sse2.psad.bw**
  - `simd_adler32`, in: proc-macro2, reqwest, rustix, syn
- **intrinsic llvm.x86.avx.ldu.dq.256**
  - `httparse`, in: hyper-rustls, reqwest, syn
- **0 objects for the crate in the archive**
  - `tests`, in: idna
  - `wrappers`, in: autocfg
- **cast of a narrow vector to a split one**
  - `encoding_rs`, in: reqwest, syn
- **intrinsic (name cut off in the log; divan reads the clock with `rdtscp`)**
  - `divan`, in: anstream, anstyle-parse
- **intrinsic llvm.x86.subborrow.64**
  - `num_bigint`, in: rustls, webpki-roots
- **Intel-syntax inline asm written with AT&T registers**
  - `x86`, in: cpufeatures
- **intrinsic llvm.x86.aesni.aeskeygenassist**
  - `aes`, in: generic-array
- **intrinsic llvm.x86.sha256rnds2**
  - `sha2`, in: digest

Tests that differ:

- **clap_builder 4.6.7**: its library tests die with SIGILL (81 pass under rustc).

## Batch 1

| Rank | Crate | Version | rustc: pass / fail | Epitome: pass / fail | Result | Epitome could not compile |
|---:|---|---|---|---|---|---|
| 2 | syn | 3.0.9 | no build | no build | untestable | `base64`, `core_detect`, `crc32fast`, `encoding_rs`, `hashbrown`, `http`, `httparse`, `hyper_util`, `simd_adler32`, `tokio`; its `syn-test-suite` (lib) does not parse: `expected item, found @` |
| 3 | getrandom | 0.4.3 | 16 / 0 | 16 / 0 | pass |  |
| 8 | quote | 1.0.47 | 63 / 0 | 63 / 0 | pass |  |
| 9 | proc-macro2 | 1.0.107 | 71 / 1 | 71 / 1 | falls back | `crc32fast`, `simd_adler32` |
| 11 | windows-sys | 0.61.2 | 0 / 0 | 0 / 0 | pass (no tests) |  |
| 14 | thiserror-impl | 2.0.21 | 0 / 0 | 0 / 0 | pass (no tests) |  |
| 19 | serde | 1.0.229 | 0 / 0 | 0 / 0 | pass (no tests) |  |
| 20 | unicode-ident | 1.0.27 | 0 / 1 | 0 / 1 | falls back | `chacha20`, `criterion` |
| 23 | serde_derive | 1.0.229 | 2 / 1 | 2 / 1 | pass |  |
| 26 | autocfg | 1.5.1 | 25 / 0 | 25 / 0 | falls back | `wrappers` |
| 29 | cc | 1.11.0 | 76 / 0 | 76 / 0 | pass |  |
| 30 | regex-automata | 0.4.18 | no build | no build | untestable | `aho_corasick`, `hashbrown`; module `fuzz` missing from the package |
| 31 | socket2 | 0.6.5 | 17 / 0 | 17 / 0 | pass |  |
| 33 | rustix | 1.1.6 | 17 / 0 | 17 / 0 | falls back | `crc32fast`, `hashbrown`, `simd_adler32` |
| 34 | regex | 1.13.1 | no build | no build | untestable | `aho_corasick`, `hashbrown`; module `fuzz` missing from the package |
| 36 | digest | 0.11.3 | 3 / 1 | 3 / 1 | falls back | `sha2` |
| 37 | clap | 4.6.7 | 2 / 0 | 2 / 0 | pass |  |
| 40 | linux-raw-sys | 0.12.1 | 0 / 0 | 0 / 0 | pass (no tests) |  |
| 41 | parking_lot_core | 0.10.0 | 16 / 0 | 16 / 0 | pass |  |
| 42 | windows_x86_64_msvc | 0.53.1 | 0 / 0 | 0 / 0 | pass (no tests) |  |
| 46 | mio | 1.2.4 | 5 / 0 | 5 / 0 | falls back | `rand_chacha` |
| 47 | windows_x86_64_gnu | 0.53.1 | 0 / 0 | 0 / 0 | pass (no tests) |  |
| 48 | cpufeatures | 0.3.1 | 3 / 0 | 3 / 0 | falls back | `x86` |
| 49 | pin-project-lite | 0.2.17 | no build | no build | untestable | crate `macrotest` not a dependency |
| 50 | windows_aarch64_msvc | 0.53.1 | 0 / 0 | 0 / 0 | pass (no tests) |  |
| 51 | windows_i686_msvc | 0.53.1 | 0 / 0 | 0 / 0 | pass (no tests) |  |
| 52 | windows-targets | 0.53.5 | 0 / 1 | 0 / 1 | pass |  |
| 53 | http | 1.5.0 | 409 / 0 | 409 / 0 | falls back | `aho_corasick`, `chacha20`, `http` |
| 54 | windows_i686_gnu | 0.53.1 | 0 / 0 | 0 / 0 | pass (no tests) |  |
| 60 | windows_aarch64_gnullvm | 0.53.1 | 0 / 0 | 0 / 0 | pass (no tests) |  |
| 61 | windows_x86_64_gnullvm | 0.53.1 | 0 / 0 | 0 / 0 | pass (no tests) |  |
| 63 | tokio | 1.54.1 | no build | no build | untestable | `chacha20`, `core_detect`, `rand_chacha`, `tokio`; needs `--features full` |
| 64 | lazy_static | 1.5.1 | 9 / 1 | 9 / 1 | pass |  |
| 67 | idna | 1.1.0 | 12649 / 155 | 12649 / 155 | falls back | `tester`, `tests` |
| 69 | futures-core | 0.3.34 | 0 / 2 | 0 / 2 | pass |  |
| 71 | zerocopy | 0.8.63 | 135 / 5 | 135 / 5 | falls back | `aho_corasick` |
| 74 | futures-util | 0.3.34 | 2 / 153 | 2 / 153 | falls back | `futures` |
| 75 | futures-task | 0.3.34 | 1 / 2 | 1 / 2 | pass |  |
| 76 | hyper | 1.12.0 | 10 / 1 | 10 / 1 | falls back | `aho_corasick`, `http`, `tokio` |
| 77 | rustls | 0.23.46 | no build | no build | untestable | `aho_corasick`, `num_bigint`, `oid_registry`, `rcgen`; test data missing from the package |
| 83 | generic-array | 1.4.5 | 35 / 0 | 35 / 0 | falls back | `aes`, `criterion`, `hashbrown`, `rand_chacha` |
| 84 | time | 0.3.55 | no build | no build | untestable | `aho_corasick`, `rand_chacha`; attribute `rstest` not found (needs features) |
| 85 | http-body | 1.1.0 | 4 / 0 | 4 / 0 | falls back | `http` |
| 87 | futures-sink | 0.3.34 | 0 / 0 | 0 / 0 | pass (no tests) |  |
| 89 | futures-channel | 0.3.34 | no build | no build | untestable | crate `futures` not a dependency |
| 91 | clap_lex | 1.1.1 | 17 / 0 | 17 / 0 | pass |  |
| 92 | tracing-core | 0.1.36 | 19 / 1 | 19 / 1 | pass |  |
| 95 | tracing | 0.1.44 | no build | no build | untestable | `clap`, `criterion`; crate `tracing_mock` not published |
| 96 | toml_datetime | 1.1.2+spec-1.1.0 | 0 / 0 | 0 / 0 | falls back | `similar`, `snapbox` |
| 97 | futures-io | 0.3.34 | 0 / 0 | 0 / 0 | pass (no tests) |  |
| 100 | tempfile | 3.27.0 | 114 / 0 | 114 / 0 | pass |  |
| 103 | chrono | 0.4.45 | 280 / 8 | 280 / 8 | falls back | `similar`, `similar_asserts` |
| 104 | h2 | 0.4.20 | 62 / 382 | 62 / 382 | falls back | `akamai`, `client`, `hashbrown`, `http`, `rand_chacha`, `server`, `tokio` |
| 105 | nix | 0.31.3 | no build | no build | untestable | `rand_chacha`; needs features (`nix::sys::aio`) |
| 106 | tokio-macros | 2.7.2 | 39 / 0 | 39 / 0 | pass |  |
| 107 | rustls-webpki | 0.103.16 | no build | no build | untestable | module `test_utils` missing from the package |
| 110 | futures-macro | 0.3.34 | 0 / 0 | 0 / 0 | pass (no tests) |  |
| 112 | rustversion | 1.0.23 | 7 / 0 | 7 / 0 | pass |  |
| 114 | tokio-util | 0.7.20 | no build | no build | untestable | `tokio`; needs features (`tokio_util::codec`) |
| 115 | futures | 0.3.34 | no build | no build | untestable | `futures`; crate `futures_test` not a dependency |
| 116 | tokio-rustls | 0.26.6 | no build | no build | untestable | `tokio`; `tests/utils.rs` missing from the package |
| 117 | pkg-config | 0.3.34 | 34 / 0 | 34 / 0 | pass |  |
| 118 | clap_builder | 4.6.7 | 81 / 1 | 0 / 0 | fail | `similar`, `snapbox`; SIGILL in the library tests |
| 119 | clap_derive | 4.6.7 | 0 / 0 | 0 / 0 | pass (no tests) |  |
| 120 | webpki-roots | 1.0.11 | no build | no build | untestable | `num_bigint`, `oid_registry`, `tokio`; crate `webpki_ccadb` not a dependency |
| 122 | tracing-attributes | 0.1.31 | no build | no build | untestable | crate `tracing_mock` not published |
| 123 | darling_core | 0.24.1 | 130 / 0 | 130 / 0 | falls back | `darling_core` |
| 125 | darling_macro | 0.24.1 | 0 / 0 | 0 / 0 | pass (no tests) |  |
| 126 | darling | 0.24.1 | 124 / 0 | 124 / 0 | falls back | `darling_core` |
| 127 | futures-executor | 0.3.34 | no build | no build | untestable | crate `futures` not a dependency |
| 129 | reqwest | 0.13.5 | 45 / 1 | 45 / 1 | falls back | `aho_corasick`, `base64`, `core_detect`, `crc32fast`, `encoding_rs`, `hashbrown`, `http`, `httparse`, `hyper_util`, `reqwest`, `simd_adler32`, `tokio` |
| 130 | wasi | 0.14.7+wasi-0.2.4 | no build | no build | untestable | its tests do not link on Linux |
| 131 | ring | 0.17.14 | no build | no build | untestable | test data missing from the package |
| 132 | signal-hook-registry | 1.4.8 | 15 / 0 | 15 / 0 | pass |  |
| 134 | synstructure | 0.14.0 | 42 / 0 | 42 / 0 | pass |  |
| 135 | nom | 8.0.0 | no build | no build | untestable | `rand_chacha`; crate `nom_language` not a dependency |
| 138 | redox_syscall | 0.9.5 | 1 / 0 | 1 / 0 | pass |  |
| 143 | time-macros | 0.2.32 | 0 / 0 | 0 / 0 | pass (no tests) |  |
| 144 | anstream | 1.0.0 | 30 / 0 | 30 / 0 | falls back | `divan` |
| 148 | tower | 0.5.3 | no build | no build | untestable | `aho_corasick`, `http`, `rand_chacha`, `tokio`; `tracing::subscriber::DefaultGuard` not found (needs features) |
| 149 | serde_core | 1.0.229 | 48 / 21 | 48 / 21 | pass |  |
| 150 | anstyle-parse | 1.0.0 | 2 / 1 | 2 / 1 | falls back | `divan` |
| 151 | iana-time-zone | 0.1.65 | 8 / 0 | 8 / 0 | pass |  |
| 153 | async-trait | 0.1.92 | 9 / 0 | 9 / 0 | pass |  |
| 154 | time-core | 0.1.9 | 1 / 3 | 1 / 3 | pass |  |
| 156 | anstyle-query | 1.1.5 | 3 / 0 | 3 / 0 | pass |  |
| 158 | hyper-rustls | 0.27.10 | 7 / 0 | 7 / 0 | falls back | `hashbrown`, `http`, `httparse`, `hyper_util`, `tokio` |
| 162 | sync_wrapper | 1.0.2 | 6 / 0 | 6 / 0 | pass |  |
| 163 | tower-service | 0.3.3 | 4 / 0 | 4 / 0 | falls back | `http` |
| 164 | strum_macros | 0.28.0 | 12 / 9 | 12 / 9 | pass |  |
| 165 | want | 0.3.2 | no build | no build | untestable | uses `#![feature]`, nightly only |
| 166 | try-lock | 0.2.5 | 2 / 0 | 2 / 0 | pass |  |
| 167 | tinyvec_macros | 0.1.1 | 0 / 0 | 0 / 0 | pass (no tests) |  |
| 168 | strum | 0.28.0 | 0 / 4 | 0 / 4 | pass |  |
| 172 | displaydoc | 0.2.7 | 9 / 0 | 9 / 0 | pass |  |
| 173 | yoke | 0.8.4 | 57 / 2 | 57 / 2 | pass |  |
| 175 | zerocopy-derive | 0.8.63 | no build | no build | untestable | crate `zerocopy_renamed` not a dependency |
| 177 | litemap | 0.8.3 | no build | no build | untestable | `criterion`, `rand_chacha`, `simdutf8`; crate `icu_locale_core` not a dependency |
| 178 | yoke-derive | 0.8.4 | no build | no build | untestable | crate `zerovec` not a dependency |
| 179 | mime | 0.3.17 | 16 / 1 | 16 / 1 | pass |  |
