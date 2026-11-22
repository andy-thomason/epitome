# Code size on real crates

epitome-rustc's programs against rustc's, on twelve crates.io crates
(`crates.txt`, each with a demo program in `progs/`). The write-up and the
results are in [`docs/design/crate-sizes.md`](../../docs/design/crate-sizes.md).

```sh
cargo build --release -p epitome-rustc
cargo fetch --manifest-path bench/crate-sizes/fetch/Cargo.toml   # if crates are missing
bench/crate-sizes/build.sh                  # builds and runs everything into target/crate-sizes
bench/crate-sizes/sizes.py --mix --new      # the tables, as Markdown
bench/crate-sizes/mix.py EXE SYMBOL --code  # one function's instruction mix and code
```

- `build.sh [OUT] [crate...]`: epitome-rustc (default, `-C
  debug-assertions=off`, and that with `-Cpanic=immediate-abort`) and
  rustc (`-C opt-level=3` and `s`, `-C panic=abort`, v0 symbols), plus a
  hello world as the baseline; `OUT/status.txt` says whether each build
  prints what rustc's does.
- `sizes.py [OUT] [--crate NAME] [--top N] [--all] [--mix] [--new]`: text
  sizes, the crate's own code (ours put down to crates by
  `epitome-rustc --fn-crates`, rustc's by the v0 symbol), and functions
  matched by name.
- `demangle.rs`: a v0/legacy demangler over rustc-demangle, which
  `build.sh` builds for `sizes.py`.
