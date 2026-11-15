"""Checks an iCE40 `.asc` file by running IceStorm's `icepack` on it as a
black box, and says what the bitstream it makes holds.

    python3 sv/tools/ice40_asc_check.py design.asc [--bin design.bin] [--board icesugar-nano]

`icepack` is built on first use, from a pinned commit of IceStorm
(github.com/YosysHQ/icestorm), into ~/.cache/epitome (or $XDG_CACHE_HOME);
it needs git, make and a C++ compiler. --icepack PATH uses another build.

The checks:

- `icepack` packs the file into a bitstream without an error. It is
  lenient: it reads only the 1s of a tile's rows, ignores columns past the
  tile's width and tiles with fewer than 16 rows, and ORs repeated tiles
  together, so packing alone proves little.
- The bitstream is unpacked again (`icepack -u`) and compared with the input
  bit by bit. Every bit set in one and not the other is a bit the input asked
  for that the device cannot hold, or one it did not ask for. The tile widths
  are learnt from the unpacked file, and input rows of another width, or with
  characters other than 0 and 1, are reported.
- Packing twice gives the same bitstream.
- With --board, the device suits the board (checked first).

Then a summary: the device, the tiles of each kind and how many are used,
the bits set, the block RAMs initialised and the bitstream's size.

The exit status is 1 if a check fails.
"""
import argparse
import collections
import filecmp
import os
import shutil
import subprocess
import sys
import tempfile

ICESTORM_URL = "https://github.com/YosysHQ/icestorm"
ICESTORM_COMMIT = "00da9c3e6ceff056248e3451dd6fd3210d6ceec9"

TILE_KINDS = ("logic", "io", "ramb", "ramt", "dsp0", "dsp1", "dsp2", "dsp3", "ipcon")

# The device each board carries (`.device` names the die, not the package).
BOARDS = {
    "icesugar-nano": ("1k", "iCE40LP1K-CM36"),
    "icestick": ("1k", "iCE40HX1K-TQ144"),
    "icebreaker": ("5k", "iCE40UP5K-SG48"),
    "icesugar": ("5k", "iCE40UP5K-SG48"),
}

# Differing bits listed per tile before the rest are counted.
SHOW_BITS = 8


def cache_dir():
    base = os.environ.get("XDG_CACHE_HOME") or os.path.expanduser("~/.cache")
    return os.path.join(base, "epitome", "icestorm-" + ICESTORM_COMMIT[:12])


def install_icepack():
    """Builds `icepack` from the pinned IceStorm commit, unless it has been
    already, and returns its path."""
    src = cache_dir()
    exe = os.path.join(src, "icepack", "icepack")
    if os.access(exe, os.X_OK):
        return exe
    for tool in ("git", "make"):
        if shutil.which(tool) is None:
            sys.exit(f"error: building icepack needs {tool}, which is not on PATH")
    print(f"Installing icepack: IceStorm {ICESTORM_COMMIT[:12]} into {src}", file=sys.stderr)
    # Build in a temporary directory beside the cache and move it into place,
    # so an interrupted build is never taken for a finished one.
    os.makedirs(os.path.dirname(src), exist_ok=True)
    tmp = tempfile.mkdtemp(prefix="icestorm-", dir=os.path.dirname(src))
    try:
        def run(*cmd):
            subprocess.run(cmd, cwd=tmp, check=True, stdout=subprocess.DEVNULL)

        run("git", "init", "-q")
        run("git", "fetch", "-q", "--depth", "1", ICESTORM_URL, ICESTORM_COMMIT)
        run("git", "checkout", "-q", "FETCH_HEAD")
        run("make", "-s", "-C", "icepack", f"-j{os.cpu_count() or 1}")
        if os.path.exists(src):
            shutil.rmtree(src)
        os.rename(tmp, src)
    except subprocess.CalledProcessError as e:
        shutil.rmtree(tmp, ignore_errors=True)
        sys.exit(f"error: installing icepack failed: {' '.join(e.cmd)} exited with {e.returncode}")
    except BaseException:
        shutil.rmtree(tmp, ignore_errors=True)
        raise
    return exe


class Asc:
    """What an `.asc` file says: its device, each tile's set bits (as (x, y)
    pairs, x the column, y the row), its block RAM contents, its extra bits,
    and the lines a strict reading finds wrong."""

    def __init__(self, path):
        self.device = None
        self.comment = []
        self.tiles = {}  # (kind, x, y) -> set of (bit_x, bit_y)
        self.rows = {}  # (kind, x, y) -> list of row strings, as written
        self.ram = {}  # (x, y) -> list of hex row strings
        self.extra = set()  # (bank, x, y)
        self.syms = 0
        self.problems = []  # (line number, what)
        self._read(path)

    def _read(self, path):
        with open(path) as f:
            lines = f.read().split("\n")
        i = 0
        while i < len(lines):
            line, at = lines[i].strip(), i + 1
            i += 1
            if not line:
                continue
            words = line.split()
            cmd = words[0]
            if cmd == ".comment":
                while i < len(lines) and not lines[i].startswith("."):
                    self.comment.append(lines[i])
                    i += 1
            elif cmd == ".device":
                if self.device is not None:
                    self.problems.append((at, "a second .device"))
                self.device = words[1] if len(words) > 1 else ""
            elif cmd.endswith("_tile") or cmd.startswith(".dsp"):
                kind = cmd[1:].removesuffix("_tile")
                key = (kind, int(words[1]), int(words[2]))
                body = self._body(lines, i)
                i += len(body)
                if len(body) != 16:
                    self.problems.append((at, f"{cmd} {key[1]} {key[2]} has {len(body)} rows, not 16"))
                if key in self.tiles:
                    self.problems.append((at, f"{cmd} {key[1]} {key[2]} appears again"))
                bits = self.tiles.setdefault(key, set())
                for y, row in enumerate(body):
                    bits.update((x, y) for x, c in enumerate(row) if c == "1")
                    bad = sorted(set(row) - {"0", "1"})
                    if bad:
                        self.problems.append((at + 1 + y, f"{cmd} {key[1]} {key[2]} row {y} has {''.join(bad)!r}"))
                self.rows[key] = body
            elif cmd == ".ram_data":
                key = (int(words[1]), int(words[2]))
                body = self._body(lines, i)
                i += len(body)
                self.ram[key] = body
            elif cmd == ".extra_bit":
                self.extra.add(tuple(int(w) for w in words[1:4]))
            elif cmd == ".sym":
                self.syms += 1
            elif cmd in (".warmboot",):
                pass
            else:
                self.problems.append((at, f"unknown line {line[:40]!r}"))

    @staticmethod
    def _body(lines, i):
        """The data rows from line i (counted from 0) up to the next
        statement, at most 16."""
        body = []
        while i < len(lines) and len(body) < 16 and lines[i].strip() and not lines[i].startswith("."):
            body.append(lines[i].strip())
            i += 1
        return body

    def used(self, key):
        return bool(self.tiles.get(key))

    def ram_used(self, key):
        return any(c != "0" for row in self.ram.get(key, []) for c in row)


def run_icepack(icepack, args, what):
    """Runs icepack; on failure, says so and returns None."""
    r = subprocess.run([icepack, *args], capture_output=True, text=True)
    if r.returncode != 0:
        how = f"crashed (signal {-r.returncode})" if r.returncode < 0 else f"exited with {r.returncode}"
        print(f"FAIL  {what}: icepack {how}")
        for line in (r.stderr or r.stdout).strip().splitlines():
            print(f"      {line}")
        return None
    return r


def compare(given, packed):
    """The differences between the input and the round-tripped file, as lines
    of the report, and whether there were any."""
    out = []
    widths = {}
    for (kind, _, _), body in packed.rows.items():
        if body:
            widths[kind] = len(body[0])
    for key, body in sorted(given.rows.items()):
        width = widths.get(key[0])
        wrong = sorted({len(row) for row in body if width is not None and len(row) != width})
        if wrong:
            out.append(f"{key[0]} tile {key[1]} {key[2]}: rows {wrong} wide, the device's are {width}")
    for key in sorted(set(given.tiles) | set(packed.tiles)):
        want, got = given.tiles.get(key, set()), packed.tiles.get(key, set())
        if key not in packed.tiles and want:
            out.append(f"{key[0]} tile {key[1]} {key[2]}: no such tile on this device ({len(want)} bits set)")
            continue
        lost, gained = sorted(want - got), sorted(got - want)
        for name, bits in (("not in the bitstream", lost), ("in the bitstream, not the input", gained)):
            if bits:
                shown = " ".join(f"B{y}[{x}]" for x, y in bits[:SHOW_BITS])
                more = f" and {len(bits) - SHOW_BITS} more" if len(bits) > SHOW_BITS else ""
                out.append(f"{key[0]} tile {key[1]} {key[2]}: {len(bits)} bits {name}: {shown}{more}")
    for key in sorted(set(given.ram) | set(packed.ram)):
        a = "".join(given.ram.get(key, [])).lower().ljust(16 * 64, "0")
        b = "".join(packed.ram.get(key, [])).lower().ljust(16 * 64, "0")
        if a != b:
            out.append(f"ram_data {key[0]} {key[1]}: contents differ")
    if given.extra != packed.extra:
        out.append(f"extra bits differ: input {sorted(given.extra)}, bitstream {sorted(packed.extra)}")
    return out


def summary(asc, packed, bin_path):
    print(f"\nDevice {asc.device}, bitstream {os.path.getsize(bin_path)} bytes")
    if asc.comment:
        print(f"Comment: {asc.comment[0][:70]}" + (" ..." if len(asc.comment) > 1 else ""))
    counts = collections.Counter(k[0] for k in packed.tiles)
    print(f"{'tiles':<8}{'on die':>8}{'used':>8}{'bits set':>10}")
    for kind in TILE_KINDS:
        if counts[kind]:
            keys = [k for k in packed.tiles if k[0] == kind]
            used = sum(packed.used(k) for k in keys)
            bits = sum(len(packed.tiles[k]) for k in keys)
            print(f"{kind:<8}{counts[kind]:>8}{used:>8}{bits:>10}")
    if packed.ram:
        used = sum(packed.ram_used(k) for k in packed.ram)
        print(f"Block RAMs initialised: {used} of {len(packed.ram)}")
    if packed.extra:
        print(f"Extra bits: {len(packed.extra)}")
    if asc.syms:
        print(f"Symbols (.sym, not in the bitstream): {asc.syms}")


def main():
    ap = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    ap.add_argument("asc", nargs="?", help="the .asc file to check")
    ap.add_argument("--bin", help="where to keep the bitstream (by default it is thrown away)")
    ap.add_argument("--board", choices=sorted(BOARDS), help="check the device suits this board")
    ap.add_argument("--icepack", help="the icepack to use, instead of building one")
    ap.add_argument("--install-only", action="store_true", help="install icepack and stop")
    args = ap.parse_args()

    icepack = args.icepack or install_icepack()
    if args.install_only:
        print(icepack)
        return 0
    if args.asc is None:
        ap.error("the .asc file is needed")

    failed = False
    asc = Asc(args.asc)
    if args.board:
        device, part = BOARDS[args.board]
        if asc.device == device:
            print(f"ok    device {device} suits the {args.board} ({part})")
        else:
            print(f"FAIL  device {asc.device}: the {args.board} carries an {part}, .device {device}")
            failed = True

    with tempfile.TemporaryDirectory(prefix="ice40-asc-check-") as tmp:
        bin_path = os.path.join(tmp, "out.bin")
        again = os.path.join(tmp, "again.bin")
        unpacked = os.path.join(tmp, "unpacked.asc")

        if run_icepack(icepack, [args.asc, bin_path], "pack") is None:
            return 1
        print("ok    pack")

        if run_icepack(icepack, [args.asc, again], "pack again") is None:
            return 1
        if filecmp.cmp(bin_path, again, shallow=False):
            print("ok    packing twice gives the same bitstream")
        else:
            print("FAIL  packing twice gives different bitstreams")
            failed = True

        if run_icepack(icepack, ["-u", bin_path, unpacked], "unpack") is None:
            return 1
        packed = Asc(unpacked)
        diffs = compare(asc, packed)
        if asc.problems or diffs:
            print("FAIL  the bitstream is not what the file says")
            for at, what in asc.problems:
                print(f"      line {at}: {what}")
            for line in diffs:
                print(f"      {line}")
            failed = True
        else:
            print("ok    unpacking the bitstream gives the same bits")

        summary(asc, packed, bin_path)
        if args.bin:
            shutil.copyfile(bin_path, args.bin)
            print(f"Bitstream written to {args.bin}")
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
