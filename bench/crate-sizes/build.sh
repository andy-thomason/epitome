#!/bin/bash
# Builds each crate's demo program (progs/<crate>_demo.rs) with
# epitome-rustc2 (by default, -C debug-assertions=off, and that with
# -Cpanic=immediate-abort) and with rustc
# (-O -C panic=abort, and opt-level=s), runs them, and checks that they
# print the same.
#
#   bench/crate-sizes/build.sh [OUT] [crate...]
#
# OUT defaults to target/crate-sizes. Crates come from crates.txt and the
# Cargo registry (~/.cargo/registry/src/*); if any are missing,
# `cargo fetch --manifest-path bench/crate-sizes/fetch/Cargo.toml`.
# EPITOME_RUSTC2 overrides target/release/epitome-rustc2.

HERE=$(cd "$(dirname "$0")" && pwd)
ROOT=$(cd "$HERE/../.." && pwd)
OUT=${1:-$ROOT/target/crate-sizes}; shift || true
export RUST_MIN_STACK=${RUST_MIN_STACK:-268435456}   # deep programs overflow its stack
B=${EPITOME_RUSTC2:-$ROOT/target/release/epitome-rustc2}
REG=$(ls -d "$HOME"/.cargo/registry/src/*/ | head -1)
mkdir -p "$OUT/crates"
OUT=$(cd "$OUT" && pwd)
STATUS="$OUT/status.txt"
: > "$STATUS"

# The demangler sizes.py uses, from rustc-demangle in the registry.
DM=$(ls -d "$REG"rustc-demangle-0.1.* | tail -1)
mkdir -p "$OUT/tools"
[ -x "$OUT/tools/demangle" ] ||
    { rustc --edition 2015 --crate-type rlib --crate-name rustc_demangle -O --cap-lints allow --out-dir "$OUT/tools" "$DM/src/lib.rs" &&
      rustc --edition 2021 -O --extern rustc_demangle="$OUT/tools/librustc_demangle.rlib" -o "$OUT/tools/demangle" "$HERE/demangle.rs"; } ||
    echo "warning: could not build the demangler from $DM"

# The baseline: a hello world, built the same ways.
h="$OUT/hello"; mkdir -p "$h"
for lvl in 3 s; do
    rustc --edition 2021 -C opt-level=$lvl -C panic=abort -C symbol-mangling-version=v0 -o "$h/rustc-O$lvl.exe" "$HERE/progs/hello.rs"
done
"$B" --std --edition 2021 --fn-crates "$h/ep-default.crates" -o "$h/ep-default.exe" "$HERE/progs/hello.rs"
"$B" --std --edition 2021 -C debug-assertions=off --fn-crates "$h/ep-ndebug.crates" -o "$h/ep-ndebug.exe" "$HERE/progs/hello.rs"
"$B" --std --edition 2021 -C debug-assertions=off -Cpanic=immediate-abort --fn-crates "$h/ep-abort.crates" -o "$h/ep-abort.exe" "$HERE/progs/hello.rs"

grep -v '^#' "$HERE/crates.txt" | while read -r name dir edition cfgs; do
    [ -n "$name" ] || continue
    if [ $# -gt 0 ] && ! printf '%s\n' "$@" | grep -qx "$name"; then continue; fi
    src="$REG$dir"
    if [ ! -d "$src" ]; then echo "$name missing $dir" | tee -a "$STATUS"; continue; fi
    # epitome-rustc2 names a crate after the directory above src/.
    ln -sfn "$src" "$OUT/crates/$name"
    d="$OUT/$name"; mkdir -p "$d"
    prog="$HERE/progs/${name}_demo.rs"
    lib=src/lib.rs; [ -f "$src/$lib" ] || lib=lib.rs   # fnv's is at the top
    rm -f "$d"/*.exe "$d"/*.txt
    epcfg=(); rcfg=()
    if [ "$cfgs" != "-" ]; then
        IFS=, read -ra cs <<< "$cfgs"
        for c in "${cs[@]}"; do epcfg+=(--cfg "$c"); rcfg+=(--cfg "$c"); done
    fi
    # rustc: the crate as an rlib, then the program; v0 symbols.
    for lvl in 3 s; do
        rd="$d/rustc-O$lvl"; mkdir -p "$rd"
        rustc --edition "$edition" --crate-type rlib --crate-name "$name" -C opt-level=$lvl -C panic=abort \
            -C symbol-mangling-version=v0 "${rcfg[@]}" --cap-lints allow --out-dir "$rd" "$src/$lib" 2> "$rd/build.log" &&
        rustc --edition "$edition" -C opt-level=$lvl -C panic=abort -C symbol-mangling-version=v0 \
            --extern "$name=$rd/lib$name.rlib" -L "$rd" -o "$d/rustc-O$lvl.exe" "$prog" 2>> "$rd/build.log" ||
            echo "$name rustc-O$lvl build-failed" >> "$STATUS"
    done
    # default: as epitome-rustc2 builds by default (the crates keep
    # debug_assertions); ndebug: without them, as rustc -O; abort: that
    # and -Cpanic=immediate-abort.
    for mode in default ndebug abort; do
        case $mode in
            default) extra=() ;;
            ndebug) extra=(-C debug-assertions=off) ;;
            abort) extra=(-C debug-assertions=off -Cpanic=immediate-abort) ;;
        esac
        if "$B" --std --edition "$edition" "${epcfg[@]}" "${extra[@]}" --fn-crates "$d/ep-$mode.crates" -o "$d/ep-$mode.exe" \
            "$OUT/crates/$name/$lib" "$prog" > "$d/ep-$mode.log" 2>&1; then :; else
            echo "$name ep-$mode build-failed: $(grep -m1 error "$d/ep-$mode.log" | cut -c1-200)" >> "$STATUS"
        fi
    done
    [ -x "$d/rustc-O3.exe" ] && "$d/rustc-O3.exe" > "$d/expected.txt" 2>&1
    for exe in rustc-Os ep-default ep-ndebug ep-abort; do
        [ -x "$d/$exe.exe" ] || continue
        "$d/$exe.exe" > "$d/$exe.txt" 2>&1
        if cmp -s "$d/expected.txt" "$d/$exe.txt"; then echo "$name $exe ok" >> "$STATUS"
        else echo "$name $exe differs" >> "$STATUS"; fi
    done
    grep "^$name " "$STATUS"
done
