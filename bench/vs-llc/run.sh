#!/bin/bash
# Epitome's code against llc -O3's, over the crates of sample/ (their
# LLVM IR from rustc), on AArch64 macOS and x86-64 Linux: instructions,
# register moves, stack loads and stores, and x86's bytes, summed; and the
# allocator's own counts (epitome-llc --stats).
#
#   bench/vs-llc/run.sh [EPITOME_LLC] [TAG]     (from the repository's
#                                                root, after cargo build
#                                                --release)
#
# Objects go to target/vs-llc/<triple>/<tag>-<crate>.o (llc's once, as
# llc-<crate>.o); the table compares every tag there, so runs of
# different builds (with different TAGs) are compared side by side. On
# x86-64 the crates whose vectors are lowered through memory (memchr,
# base64, crc32fast, rand_chacha, hashbrown) are left out of the sums.
set -e
here=$(cd "$(dirname "$0")" && pwd)
root=$(cd "$here/../.." && pwd)
ours=${1:-$root/target/release/epitome-llc}
tag=${2:-ours}
host=$(rustc -vV | sed -n 's/^host: //p')
bin=$(rustc --print sysroot)/lib/rustlib/$host/bin
out=$root/target/vs-llc
triples="aarch64-apple-darwin x86_64-unknown-linux-gnu"
for t in $triples; do
  [ -d "$out/sample/$t/release/deps" ] && ls "$out/sample/$t/release/deps"/*.ll >/dev/null 2>&1 && continue
  (cd "$here/sample" && CARGO_TARGET_DIR="$out/sample" RUSTFLAGS="--emit=link,llvm-ir" \
    cargo build -q --release --target "$t")
done
for t in $triples; do
  mkdir -p "$out/$t"
  for ll in "$out/sample/$t/release/deps"/*.ll; do
    n=$(basename "$ll" .ll | sed 's/-.*//')
    [ -f "$out/$t/llc-$n.o" ] || "$bin/llc" -O3 -filetype=obj "$ll" -o "$out/$t/llc-$n.o"
    "$ours" --stats "$ll" -o "$out/$t/$tag-$n.o" 2>&1 | grep '; allocation' | sed "s/^/$t /" || true
  done | awk -v t="$tag" '{l[$1]+=$4; s[$1]+=$7; m[$1]+=$10; w[$1]+=$13}
    END {for (k in l) print k, t, "allocator: spill loads", l[k], "stores", s[k], "moves", m[k], "weighted by loop depth", w[k]}'
done
python3 "$here/count.py" "$out" "$bin/llvm-objdump"
