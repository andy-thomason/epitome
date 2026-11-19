# Against llc

How Epitome's code compares with LLVM's `llc -O3` on real code: the LLVM
IR rustc makes for a sample of crates (`sample/`: regex-syntax,
serde_json, rand_chacha, memchr, ryu and others), compiled by both for
AArch64 macOS and x86-64 Linux. `run.sh` prints, for each, the
instructions, register moves and stack loads and stores summed over the
crates (and x86's bytes), as a ratio of llc's, and the register
allocator's own counts (`epitome-llc --stats`: spill loads and stores,
moves, and spill code weighted by loop depth, 8 a loop):

```sh
cargo build --release
bench/vs-llc/run.sh                          # tagged "ours"
bench/vs-llc/run.sh /tmp/epitome-llc before  # another build, side by side
```

The crates are fetched by Cargo (`cargo build` in `sample/`, with the
rustup toolchain's targets `aarch64-apple-darwin` and
`x86_64-unknown-linux-gnu` installed; both run on either machine, as only
objects are made). On x86-64 the sums leave out the crates whose vector
code is lowered through memory (see `docs/design/later.md`), which would
swamp the rest.
