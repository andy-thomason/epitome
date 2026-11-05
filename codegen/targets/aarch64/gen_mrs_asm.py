"""Writes 80-asm.ll: entries for assembly and disassembly only, for the A64
instructions (base, floating point and Advanced SIMD) that the rest of the
description does not have.

Two sources, both public or observed:

- Arm's A-profile Architecture Machine Readable Specification (AARCHMRS,
  BSD 3-clause): each instruction's bit layout, the features it needs, and
  its assembly syntax as a grammar (Instructions.json, "assembly_rules").
  The open-source package does not say how an operand is held in its
  fields (that `<pimm>` is `imm12` times 8), so:
- clang's assembler, on lines we write: for each template, a few lines with
  the operands varied are assembled, and each operand's fields are read
  back from the words. The relation between operand and fields is fitted
  from a few forms (the value, scaled, offset, negated, PC-relative, a
  logical immediate). Every entry written is one whose words, for every
  sample, are clang's.

Entries are "asmonly": the assembler reads them and the disassembler writes
them, but instruction selection does not use them and their bodies say
nothing. Run:

    python3 codegen/targets/aarch64/gen_mrs_asm.py path/to/Instructions.json

It needs clang (any version with the AArch64 target). The output is checked
in; the code generator reads only the .ll files.
"""
import collections
import itertools
import json
import os
import re
import struct
import subprocess
import sys
import tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
CLANG = ["clang", "--target=aarch64-linux-gnu", "-c"]
# Every extension the instructions here may need, for clang to accept them.
CLANG_FEATURES = ("+crc+crypto+sha3+sm4+rcpc+rcpc3+lse+lse128+rdm+dotprod+fp16+fp16fml+bf16+i8mm+f64mm+fp8+fp8fma"
                  "+fp8dot2+fp8dot4+faminmax+lut+cssc+mops+the+d128+gcs+rasv2+ite+flagm+predres+sb+ssbs+tme+ls64"
                  "+wfxt+hbc+brbe+pauth-lr+fprcvt+lsfe+lsui+occmo+cmpbr+pauth+bti+memtag")
# And SVE's and SME's, for 81-scalable.ll.
SCALABLE_FEATURES = ("+sve+sve2+sve2p1+sve2p2+sve-b16b16+sve-bitperm+sve-aes+sve-aes2+sve-sha3+sve-sm4+sve-bfscale"
                     "+sve-f16f32mm+sme+sme2+sme2p1+sme2p2+sme-f64f64+sme-i16i64+sme-f16f16+sme-b16b16+sme-f8f16"
                     "+sme-f8f32+sme-lutv2+sme-mop4+sme-tmop+ssve-fp8fma+ssve-fp8dot2+ssve-fp8dot4+ssve-aes"
                     "+ssve-bitperm+f8f16mm+f8f32mm+sme-fa64")

MAX_VARIANTS = 48
# Whether the instructions written are SVEs and SMEs (81-scalable.ll),
# or the rest (80-asm.ll).
SCALABLE = False


def max_variants():
    # (SVEs take more: ptrues patterns are 17 for each of 4 lane sizes.)
    return 4 * MAX_VARIANTS if SCALABLE else MAX_VARIANTS


# ---------------------------------------------------------------------------
# Arm's specification


def first_literal(asm):
    for s in (asm or {}).get("symbols", []):
        if s["_type"] == "Instruction.Symbols.Literal":
            return s["value"].split()[0].lower()
    return "?"


def load(path):
    spec = json.load(open(path))
    rules = spec["assembly_rules"]
    a64 = next(s for s in spec["instructions"] if s["name"] == "A64")
    out = []

    def feats(cond, acc):
        if isinstance(cond, dict):
            if cond.get("_type") == "AST.Identifier" and str(cond.get("value", "")).startswith("FEAT_"):
                acc.add(cond["value"])
            for v in cond.values():
                feats(v, acc)
        elif isinstance(cond, list):
            for v in cond:
                feats(v, acc)

    def walk(node, mask, value, fields, path, fs):
        fields = dict(fields)
        for f in (node.get("encoding") or {}).get("values", []):
            start, width = f["range"]["start"], f["range"]["width"]
            v = f["value"]["value"].strip("'")
            if f.get("name"):
                fields[f["name"]] = (start, width)
            for k, ch in enumerate(v):
                bit = start + width - 1 - k
                if ch in "01":
                    mask |= 1 << bit
                    if ch == "1":
                        value |= 1 << bit
        fs = set(fs)
        feats(node.get("condition"), fs)
        if node["_type"] == "Instruction.Instruction":
            p = "/".join(path).lower()
            scalable = any(x in p for x in ("sve", "sme", "mortlach"))
            if "reserved" not in p and scalable == SCALABLE:
                aliases = [c for c in node.get("children") or [] if c["_type"] == "Instruction.InstructionAlias"]
                mnemonics = sorted({first_literal(c["assembly"]) for c in aliases} - {first_literal(node["assembly"])})
                out.append(dict(name=node["name"], mask=mask, value=value, fields=fields, feats=fs, alias=False,
                                aliases=mnemonics, encoded_in=node.get("_meta", {}).get("encoded_in", {}),
                                assembly=node["assembly"]))
                # Its aliases: other ways of writing it (cinc, of csinc).
                for c in aliases:
                    out.append(dict(name=f"{node['name']}.{c['name']}", mask=mask, value=value, fields=fields,
                                    feats=fs, alias=True, aliases=[],
                                    encoded_in=c.get("_meta", {}).get("encoded_in", {}) or node.get("_meta", {}).get("encoded_in", {}),
                                    assembly=c["assembly"]))
            return
        for c in node.get("children") or []:
            walk(c, mask, value, fields, path + [c["name"]], fs)

    walk(a64, 0, 0, {}, ["A64"], set())
    return out, rules


# ---------------------------------------------------------------------------
# Templates: each instruction's grammar, expanded into the lines it allows,
# with holes for operands.

# A piece of a template: ("lit", text) or ("hole", kind, display), where kind
# is "num", or a register class ("x", "xsp", "w", "wsp", "v", "b", "h", "s",
# "d", "q").
REG_FAMILIES = {
    frozenset({("x", "num"), ("xzr",)}): "x",
    frozenset({("x", "num")}): "x",
    frozenset({("x", "num"), ("sp",)}): "xsp",
    frozenset({("w", "num"), ("wzr",)}): "w",
    frozenset({("w", "num")}): "w",
    frozenset({("w", "num"), ("wsp",)}): "wsp",
    frozenset({("v", "num")}): "v",
    frozenset({("b", "num")}): "b",
    frozenset({("h", "num")}): "h",
    frozenset({("s", "num")}): "s",
    frozenset({("d", "num")}): "d",
    frozenset({("q", "num")}): "q",
}


def shape(variant):
    """A variant's shape for register families: its literals and holes."""
    return tuple("num" if p[0] == "hole" else p[1] for p in variant)


def expand(asm, rules, display=None, bound=None):
    """The variants of a grammar: lists of pieces, each with the choices it
    made (a rule used twice, such as an arrangement, chooses alike). None
    if there are too many."""
    variants = [([], dict(bound or {}))]
    for s in (asm or {}).get("symbols", []):
        new = []
        for pieces, b in variants:
            for alt, b2 in alternatives(s, rules, display, b):
                new.append((pieces + alt, b2))
                if len(new) > max_variants():
                    return None
        variants = new
    return variants


def alternatives(s, rules, display, bound):
    if s["_type"] == "Instruction.Symbols.Literal":
        return [([("lit", s["value"].lower())], bound)]
    rid = s["rule_id"]
    r = rules[rid]
    t = r["_type"]
    d = r.get("display") or display
    if t == "Instruction.Rules.Token":
        lit = {"SPACE": ("lit", " "), "COMMA": ("lit", ", "), "DOT": ("lit", "."), "BANG": ("lit", "!")}
        return [([lit.get(rid, ("hole", "num", d))], bound)]
    if t == "Instruction.Rules.Choice":
        # `#` is optional: write it.
        if rid.startswith("hash"):
            return [([("lit", "#")], bound)]
        out = []
        choices = list(enumerate(r["choices"]))
        if rid in bound:
            choices = [choices[bound[rid]]]
        for k, c in choices:
            b = dict(bound)
            b[rid] = k
            if c is None:
                out.append(([], b))
                continue
            sub = expand(c, rules, d, b)
            if sub is None:
                return [] if len(out) > max_variants() else out
            out.extend(sub)
    else:
        out = expand(r.get("symbols"), rules, d, bound)
        if out is None:
            return []
        if not out:
            out = [([], bound)]
    # A register: its alternatives are a register family.
    if r.get("display"):
        fam = REG_FAMILIES.get(frozenset(shape(a) for a, _ in out))
        if fam:
            return [([("hole", fam, r["display"])], bound)]
    return out


# ---------------------------------------------------------------------------
# Lines for clang, and its words.


# clang's words, by line, for the lines assembled so far; and lines asked
# for but not yet assembled (see `run_batches`).
CACHE = {}
PENDING = set()


def assemble(lines):
    """clang's word for each line, or None where clang refuses it or has not
    assembled it yet (then it is noted, for the next batch)."""
    out = []
    for line in lines:
        if line in CACHE:
            out.append(CACHE[line])
        else:
            PENDING.add(line)
            out.append(None)
    return out


def run_batch():
    lines = sorted(PENDING)
    PENDING.clear()
    for k in range(0, len(lines), 20000):
        part = lines[k:k + 20000]
        for line, w in zip(part, assemble_now(part)):
            CACHE[line] = w


def assemble_now(lines):
    """clang's word for each line, or None where clang refuses it."""
    words = [None] * len(lines)
    # SVE's movprfx makes clang check the next instruction against it, and
    # it reports one such error at a time: each is assembled alone, last.
    prefixes = [k for k, line in enumerate(lines) if line.startswith("movprfx")]
    if prefixes and len(lines) > 1:
        rest = [k for k in range(len(lines)) if k not in set(prefixes)]
        for k, w in zip(rest, assemble_now([lines[k] for k in rest])):
            words[k] = w
        for k in prefixes:
            words[k] = assemble_now([lines[k]])[0]
        return words
    todo = list(range(len(lines)))
    with tempfile.TemporaryDirectory() as tmp:
        # (Until clang takes what is left: it reports only some errors at a
        # time.)
        for _ in range(64):
            if not todo:
                break
            src = os.path.join(tmp, "t.s")
            with open(src, "w") as f:
                f.write("".join(lines[k] + "\n" for k in todo))
            obj = os.path.join(tmp, "t.o")
            march = "-march=armv9.4-a" + accepted(CLANG_FEATURES + (SCALABLE_FEATURES if SCALABLE else ""))
            r = subprocess.run(CLANG + [march, "-ferror-limit=0", src, "-o", obj],
                               capture_output=True, text=True)
            if r.returncode != 0:
                bad = {int(m.group(1)) - 1 for m in re.finditer(r"t\.s:(\d+):\d+: error", r.stderr)}
                crashed = "Stack dump" in r.stderr or "PLEASE submit" in r.stderr or r.returncode > 1
                if not bad or crashed:
                    # clang crashed (on some SVE and SME lines it does): the
                    # halves apart, down to the line it crashes on.
                    if len(todo) == 1:
                        break
                    half = len(todo) // 2
                    for part in (todo[:half], todo[half:]):
                        for k, w in zip(part, assemble_now([lines[k] for k in part])):
                            words[k] = w
                    break
                todo = [k for i, k in enumerate(todo) if i not in bad]
                continue
            text = elf_text(open(obj, "rb").read())
            # (A line that is not one word, an alias of two instructions,
            # would shift the rest: the halves apart, down to it.)
            if len(text) != 4 * len(todo):
                if len(todo) > 1:
                    half = len(todo) // 2
                    for part in (todo[:half], todo[half:]):
                        for k, w in zip(part, assemble_now([lines[k] for k in part])):
                            words[k] = w
                break
            for i, k in enumerate(todo):
                words[k] = struct.unpack_from("<I", text, 4 * i)[0]
            break
    return words


ACCEPTED = {}


def accepted(features):
    """The features of `features` (`+a+b`) this clang knows (newer ones
    drop some: tme)."""
    if features not in ACCEPTED:
        keep = []
        for f in filter(None, features.split("+")):
            r = subprocess.run(CLANG + [f"-march=armv9.4-a+{f}", "-x", "assembler", "-", "-o", os.devnull],
                               input="", capture_output=True, text=True)
            if r.returncode == 0:
                keep.append(f)
        ACCEPTED[features] = "".join("+" + f for f in keep)
    return ACCEPTED[features]


def elf_text(b):
    shoff, = struct.unpack_from("<Q", b, 0x28)
    shentsize, shnum, shstrndx = struct.unpack_from("<HHH", b, 0x3A)
    secs = [struct.unpack_from("<IIQQQQIIQQ", b, shoff + k * shentsize) for k in range(shnum)]
    strtab = secs[shstrndx]
    for s in secs:
        name = b[strtab[4] + s[0]:].split(b"\0")[0]
        if name == b".text":
            return b[s[4]:s[4] + s[5]]
    return b""


# ---------------------------------------------------------------------------
# Fitting: how each operand of a template is held in the word.

# The values tried for an immediate (and a label's offset).
NUMS = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 12, 15, 16, 24, 31, 32, 33, 48, 63, 64, 100, 127, 128, 255, 256, 504,
        1000, 1020, 4095, 4096, 8188, 32760, 65535, -1, -2, -3, -4, -8, -16, -64, -256, -512, -4096,
        0xff, 0xff00, 0xf0f0, 0x5555, 0x5555_5555, 0x0f0f_0f0f_0f0f_0f0f, 0x5555_5555_5555_5555, 0xffff_0000]
REG_PREFIX = {"x": "x", "xsp": "x", "w": "w", "wsp": "w", "v": "v", "b": "b", "h": "h", "s": "s", "d": "d", "q": "q",
              "z": "z", "p": "p", "pn": "pn"}
# The registers of each kind, and those tried first for a line of a
# template (lists start at even or fourth registers, ZA's slices are chosen
# by w12-w15, a counter is pn8-pn15).
REG_COUNT = {"p": 16, "pn": 16}
REG_TRIES = [None, 4, 8, 12, 0, 16, 13, 2, 24, 14, 15, 9]


def bitmask(v, width):
    """AArch64's logical immediate N:immr:imms for v, or None."""
    v &= (1 << width) - 1
    if v in (0, (1 << width) - 1):
        return None
    size = width
    while size > 2:
        half = size // 2
        if (v & ((1 << half) - 1)) != (v >> half) & ((1 << half) - 1):
            break
        size = half
    elem = v & ((1 << size) - 1)
    for r in range(size):
        rot = ((elem << r) | (elem >> (size - r))) & ((1 << size) - 1)  # rotate left by r
        ones = bin(rot).count("1")
        if rot == (1 << ones) - 1:
            immr = r
            n = 1 if size == 64 else 0
            imms = ((~(size * 2 - 1) & 0x3f) | (ones - 1)) & 0x3f if size < 64 else ones - 1
            return (n << 12) | (immr << 6) | imms
    return None


def name_of(display, taken):
    base = re.sub(r"[^a-z0-9]+", "_", display.lower()).strip("_") or "op"
    if base[0].isdigit():
        base = "o" + base
    name, k = base, 2
    while name in taken:
        name, k = f"{base}{k}", k + 1
    return name


def merge_registers(variant, encoded_in):
    """A number after a register letter (`b{<d>}`, `v{<m>}`, SVE's `z`, `p`
    and `pn`) is a register; one in a list that no field holds (`<Zd2>`) is
    one after the list's first."""
    out = []
    for p in variant:
        lit = out[-1][1] if out and out[-1][0] == "lit" else ""
        listed = SCALABLE and re.fullmatch(r"<Z[a-z]*\d>", p[2] or "") if p[0] == "hole" else False
        if p[0] == "hole" and p[1] == "num" and lit and (p[2] in encoded_in or listed):
            letter = None
            if SCALABLE and lit.endswith("pn") and (len(lit) == 2 or not lit[-3].isalnum()):
                letter = "pn"
            elif lit[-1] in ("bhsdqvwx" + ("zp" if SCALABLE else "")) and (len(lit) == 1 or not lit[-2].isalnum()):
                letter = lit[-1]
            if letter:
                out[-1] = ("lit", lit[:-len(letter)])
                out.append(("hole", letter, p[2]))
                continue
        out.append(p)
    # (Literals together, and SVE's `mul vl`, whose words come without
    # the space between them.)
    joined = []
    for p in out:
        if p[0] == "lit" and joined and joined[-1][0] == "lit":
            joined[-1] = ("lit", joined[-1][1] + p[1])
        else:
            joined.append(p)
    return [(p[0], p[1].replace("mulvl", "mul vl")) if p[0] == "lit" else p for p in joined if p != ("lit", "")]


class Hole:
    def __init__(self, kind, display, name, label):
        self.kind, self.display, self.name, self.label = kind, display, name, label
        # A register after a list's first: that hole, and how far after it.
        self.after = None

    def text(self, v):
        if self.kind == "num":
            return str(v)
        if self.kind in ("xsp", "wsp") and v == 31:
            return "sp" if self.kind == "xsp" else "wsp"
        if self.kind in ("x", "w") and v == 31:
            return self.kind + "zr"
        return f"{REG_PREFIX[self.kind]}{v}"


def line_of(variant, holes, values):
    out, k = [], 0
    for p in variant:
        if p[0] == "lit":
            out.append(p[1])
        else:
            h = holes[k]
            if h.after is not None:
                j, off = h.after
                v = values[j] if values[j] is not None else values[holes[j].after[0]] + holes[j].after[1]
                out.append(h.text(v + off if h.kind == "num" else (v + off) % REG_COUNT.get(h.kind, 32)))
            else:
                out.append(h.text(values[k]))
            k += 1
    return "".join(out)


def fits(samples, width):
    """An expression for how a hole's bits hold its value, from samples
    (value, the bits' value): `v>>a`, `c-v`, `v+c`, `bitmask64(v)`, ...;
    as (kind, constant)."""
    m = (1 << width) - 1
    forms = []
    for a in range(0, 5):
        if all(v % (1 << a) == 0 and (v >> a) & m == b for v, b in samples):
            forms.append(("shr", a))
            break
    # Constants as they come, unwrapped, so that values decode as they are
    # written (fbits = 64 - scale, not -scale).
    v0, b0 = samples[0]
    c = v0 + b0
    if all((c - v) & m == b for v, b in samples):
        forms.append(("sub_from", c))
    c = b0 - v0
    if all((v + c) & m == b for v, b in samples):
        forms.append(("add", c))
    for bw in (64, 32):
        if all(bitmask(v, bw) is not None and bitmask(v, bw) & m == b for v, b in samples):
            forms.append(("bitmask", bw))
    return forms


def reg_forms(samples, width):
    """How a register's field holds it: its number, halved or quartered
    (a list of two or four), or offset (w8-w11 in two bits, pn8-pn15 in
    three); never truncated, so that it decodes as itself."""
    m = (1 << width) - 1
    out = []
    for f in fits(samples, width):
        if f[0] == "shr" and all(v % (1 << f[1]) == 0 and v >> f[1] <= m for v, _ in samples):
            out.append(f)
        elif f[0] == "add" and all(0 <= v + f[1] <= m for v, _ in samples):
            out.append(f)
    # Offset, then halved or quartered (SME2's z16-z30, even, in three
    # bits).
    if not out:
        v0, b0 = samples[0]
        for a in (1, 2):
            c = (b0 << a) - v0
            if all((v + c) % (1 << a) == 0 and (v + c) >> a == b and 0 <= (v + c) >> a <= m for v, b in samples):
                out.append(("add_shr", (c, a)))
                break
    return out


def fit_variant(ins, variant, rules_text):
    """The entry for a template: its enc tokens and asm, or why not."""
    taken = set()
    holes = []
    for p in variant:
        if p[0] != "hole":
            continue
        label = p[1] == "num" and p[2] == "<label>"
        name = name_of(p[2], taken)
        taken.add(name)
        holes.append(Hole(p[1], p[2], name, label))
    if any("(" in h.display or re.search(r"\d>$", h.display) and h.kind == "v" for h in holes):
        return None, "a register list"
    # The registers after a list's first: which, and how far after it (the
    # next, or strided: SME2's {z0.s, z8.s}), as clang takes them.
    # An operand written twice (a destructive operation's `<Zdn>`): the
    # one register, the second written as the first.
    same = {}
    for k, h in enumerate(holes):
        if h.display in [holes[j].display for j in range(k)]:
            same[k] = next(j for j in range(k) if holes[j].display == h.display)
    # In a list in braces, the registers after the first follow from it;
    # so does a range's end (ZA's slices `<offs1>:<offs4>`).
    group, depth, groups = 0, 0, []
    k = 0
    for p in variant:
        if p[0] == "lit":
            for ch in p[1]:
                if ch == "{":
                    depth += 1
                    group += 1
                elif ch == "}":
                    depth -= 1
        else:
            groups.append(group if depth else None)
            k += 1
    derived, firsts = [], {}
    for k, h in enumerate(holes):
        if k in same:
            continue
        if h.kind == "z" and groups[k] is not None:
            if groups[k] in firsts:
                derived.append(k)
                holes[k].after = (firsts[groups[k]], None)
            else:
                firsts[groups[k]] = k
        m = re.fullmatch(r"<([a-z]+)([2-9])>", h.display)
        if h.kind == "num" and m:
            first = next((j for j in range(k) if holes[j].display == f"<{m.group(1)}1>"), None)
            if first is not None:
                derived.append(k)
                holes[k].after = (first, None)
    # (How far after the first: by the number in its name, <Zd4> the
    # fourth, times the list's stride.)
    position = {}
    for k in derived:
        m = re.search(r"(\d)>$", holes[k].display)
        if holes[k].kind == "num":
            position[k] = int(m.group(1)) - 1
            continue
        # (Else by its place in the list.)
        place = sum(1 for j in derived if j <= k and holes[j].after[0] == holes[k].after[0])
        position[k] = int(m.group(1)) - 1 if m and int(m.group(1)) > 1 else place
    for k, j in same.items():
        holes[k].name = holes[j].name
    regs = [k for k, h in enumerate(holes) if h.kind != "num" and h.after is None and k not in same]
    nums = [k for k, h in enumerate(holes) if h.kind == "num" and k not in same]
    # A base: registers 1, 2, 3, ... (else others), lists' strides, and
    # immediates the first values that work.
    import random
    rng = random.Random(len(holes) * 7919 + len(variant))
    reg_sets = [[k + 1 for k in regs]]
    for t in REG_TRIES[1:]:
        reg_sets.append([(t + 4 * i) % 32 if holes[k].kind != "pn" else 8 + (i % 8) for i, k in enumerate(regs)])
    for _ in range(64):
        reg_sets.append([rng.choice([1, 2, 4, 8, 9, 10, 11, 12, 13, 14, 15, 16, 20, 24, 28, 0]) for _ in regs])
    strides = [1, 4, 8] if derived else [1]
    lines, combos = [], []
    for regset in reg_sets:
        for stride in strides:
            # (Fewer immediates with the later registers.)
            n_imms = 16 if len(lines) < 800 else 4
            for combo in itertools.islice(itertools.product(*[[1, 0, 4, 8, 16, 2, 3, 32, 64]] * len(nums)), n_imms):
                vals = [None] * len(holes)
                for k, v in zip(regs, regset):
                    vals[k] = v
                for k, v in zip(nums, combo):
                    vals[k] = v
                for k in derived:
                    holes[k].after = (holes[k].after[0], (1 if holes[k].kind == "num" else stride) * position[k])
                for k, j in same.items():
                    holes[k].after = (j, 0) if holes[j].after is None else holes[j].after
                lines.append(line_of(variant, holes, vals))
                combos.append((vals, stride))
    words = assemble(lines)
    # The base must be this encoding (clang may choose another: ldur for
    # an offset ldr cannot scale).
    found = next(((w, v) for w, v in zip(words, combos) if w is not None and w & ins["mask"] == ins["value"]), None)
    if found is None:
        return None, "no line of it assembles"
    w0, (base, stride) = found
    for k in derived:
        holes[k].after = (holes[k].after[0], (1 if holes[k].kind == "num" else stride) * position[k])
    for k, j in same.items():
        holes[k].after = (j, 0) if holes[j].after is None else holes[j].after
        holes[k].twice = True
    # Each hole varied alone.
    tries = []
    for k, h in enumerate(holes):
        if h.after is not None:
            continue
        vals = NUMS if h.kind == "num" else [v for v in range(REG_COUNT.get(h.kind, 32)) if v != base[k]]
        for v in vals:
            if v == base[k]:
                continue
            line_vals = list(base)
            line_vals[k] = v
            tries.append((k, v, line_of(variant, holes, line_vals)))
    words = assemble([t[2] for t in tries])
    # The bits each operand may change: its fields, as Arm says it is
    # encoded (else any Arm leaves to fields). A line that changes others
    # is another encoding clang chose (add #-1 is sub #1).
    allowed = []
    for h in holes:
        a = 0
        for f in ins["encoded_in"].get(h.display, []):
            fname = f.get("value") if isinstance(f, dict) else None
            if fname in ins["fields"]:
                start, width = ins["fields"][fname]
                a |= ((1 << width) - 1) << start
        allowed.append(a or (~ins["mask"] & 0xFFFF_FFFF))
    # An alias may hold an operand in more fields than Arm lists (cinc's n
    # in Rm too): with no line within them, any of the instruction's.
    for k in range(len(holes)):
        if not any(w is not None and w != w0 and (w ^ w0) & ~allowed[k] == 0
                   for (j, _, _), w in zip(tries, words) if j == k):
            allowed[k] = ~ins["mask"] & 0xFFFF_FFFF
    bits = [0] * len(holes)
    seen = [[] for _ in holes]
    # (A register after a list's first changes nothing of its own.)
    for k in derived:
        allowed[k] = 0
    for (k, v, _), w in zip(tries, words):
        if w is not None and (w ^ w0) & ~allowed[k] == 0:
            seen[k].append((v, w))
            bits[k] |= w ^ w0
    # Only bits Arm leaves to fields.
    for k, h in enumerate(holes):
        if h.after is not None:
            continue
        if bits[k] & ins["mask"]:
            return None, "an operand changes fixed bits"
        if not bits[k]:
            return None, f"operand {h.display} changes no bits"

    def express(form, v):
        if form[0] == "add_shr":
            c, a = form[1]
            return f"({v}+{c})>>{a}" if c >= 0 else f"({v}-{-c})>>{a}"
        return {"shr": f"{v}>>{form[1]}" if form[1] else v, "sub_from": f"({form[1]}-{v})",
                "add": f"({v}+{form[1]})" if form[1] >= 0 else f"({v}-{-form[1]})",
                "bitmask": f"bitmask{form[1]}({v})"}[form[0]]

    # First, an operand whose bits no other changes, as one value in them
    # (slices of it, if split: adr's immhi and immlo).
    tokens_for = {}
    by_field = []
    for k, h in enumerate(holes):
        if h.after is not None:
            continue
        if any(bits[k] & bits[j] for j in range(len(holes)) if j != k):
            by_field.append(k)
            continue
        order = []
        for f in ins["encoded_in"].get(h.display, []):
            fname = f.get("value") if isinstance(f, dict) else None
            if fname in ins["fields"]:
                start, width = ins["fields"][fname]
                order.extend(range(start + width - 1, start - 1, -1))
        order += [b for b in range(31, -1, -1) if b not in order]
        positions = [b for b in order if bits[k] >> b & 1]
        width = len(positions)
        samples = [(base[k], gather(w0, positions))] + [(v, gather(w, positions)) for v, w in seen[k]]
        if h.kind != "num":
            form = ("shr", 0) if all(b == v for v, b in samples) else None
            if form is None and SCALABLE:
                form = next(iter(reg_forms(samples, width)), None)
        else:
            forms = fits(samples, width)
            form = forms[0] if forms else None
        if form is None or (form[0] != "shr" and len(set(order[:width]) ^ set(positions)) == 0 and False):
            by_field.append(k)
            continue
        tokens_for[k] = (positions, form, width, samples)
    # Then the others, field by field: a field of one operand, in one of
    # the forms; of several, their sum or differences and a constant
    # (sbfx's imms = lsb + width - 1); a register in two fields (cinc's Rn
    # and Rm) is in each.
    field_tokens = {}  # top bit -> (bits, token)
    forms_of = {k: t[1] for k, t in tokens_for.items()}
    left = 0
    for k in by_field:
        left |= bits[k]
    for fname, (start, width) in sorted(ins["fields"].items(), key=lambda x: -x[1][0]):
        positions = [b for b in range(start + width - 1, start - 1, -1) if left >> b & 1]
        if not positions:
            continue
        if positions != list(range(positions[0], positions[0] - len(positions), -1)):
            return None, f"field {fname} is changed in pieces"
        pm = sum(1 << b for b in positions)
        deps = [k for k in by_field if bits[k] & pm]
        n = len(positions)
        m = (1 << n) - 1
        g0 = gather(w0, positions)
        if len(deps) == 1:
            k = deps[0]
            h = holes[k]
            samples = [(base[k], g0)] + [(v, gather(w, positions)) for v, w in seen[k]]
            forms = [("shr", 0)] if h.kind != "num" and all(b == v for v, b in samples) else fits(samples, n)
            if h.kind != "num" and SCALABLE and not (forms and forms[0] == ("shr", 0) and all(b == v for v, b in samples)):
                forms = reg_forms(samples, n)
            if not forms:
                return None, f"no fit for {h.display} in {fname}"
            form = forms[0]
            forms_of.setdefault(k, form)
            signed = h.kind == "num" and form[0] == "shr" and any(v < 0 for v, _ in samples)
            expr = express(form, h.name)
        else:
            terms = []
            const = g0
            for k in deps:
                c = next((c for c in (1, -1) if all((gather(w, positions) - g0 - c * (v - base[k])) & m == 0
                                                    for v, w in seen[k])), None)
                if c is None:
                    return None, f"no fit for field {fname} of several operands"
                terms.append(("+" if c > 0 else "-") + holes[k].name)
                const -= c * base[k]
            expr = "(" + "".join(terms).lstrip("+") + (f"+{const}" if const > 0 else f"{const}" if const < 0 else "") + ")"
            signed = False
        field_tokens[positions[0]] = (positions, f"{fname.lower()}_f:{'s' if signed else ''}{n}={expr}")
        left &= ~pm
    if left:
        return None, "an operand changes bits in no field"
    # The tokens, most significant bit first: literal bits from the base
    # word, runs of a hole's bits as fields.
    owner = {}
    for k, (positions, form, width, _) in tokens_for.items():
        for i, b in enumerate(positions):
            owner[b] = (k, width - 1 - i)  # bit b holds bit (width-1-i) of the stored value
    toks = []
    b = 31
    nfield = collections.Counter()
    while b >= 0:
        if b in field_tokens:
            positions, tok = field_tokens[b]
            toks.append(tok)
            b -= len(positions)
            continue
        if b not in owner:
            toks.append(str(w0 >> b & 1))
            b -= 1
            continue
        k, hi = owner[b]
        lo_b = b
        while lo_b - 1 >= 0 and owner.get(lo_b - 1) == (k, owner[lo_b][1] - 1):
            lo_b -= 1
        run = b - lo_b + 1
        lo = hi - run + 1
        h = holes[k]
        positions, form, width, samples = tokens_for[k]
        nfield[k] += 1
        fname = f"{h.name}{nfield[k]}" if run != width else h.name + "_f"
        v = h.name
        signed = h.kind == "num" and any(s[0] < 0 for s in samples) and form[0] == "shr" and hi == width - 1
        if run == width:
            expr = express(form, v)
        elif form[0] == "shr":
            expr = f"({v}>>{form[1] + lo})&{(1 << run) - 1}" if lo + form[1] else f"{v}&{(1 << run) - 1}"
            if signed:
                expr = f"{v}>>{form[1] + lo}"
        else:
            return None, f"{h.display}: a split field of a form that cannot be sliced"
        sw = f"{'s' if signed else ''}{run}"
        toks.append(f"{h.name}:{sw}" if expr == h.name else f"{fname}:{sw}={expr}")
        b = lo_b - 1
    # Literal bits together.
    merged = []
    for t in toks:
        if t in "01" and merged and re.fullmatch(r"[01]+", merged[-1]):
            merged[-1] += t
        else:
            merged.append(t)
    clean = [w0] + [w for k in range(len(holes)) for _, w in seen[k]]
    return (holes, merged, w0, clean, forms_of, base), None


def spelling(toks):
    """An encoding's shape: its tokens with the operands numbered in order."""
    ids = {}
    return re.sub(r"[a-z_][a-z0-9_]*", lambda m: ids.setdefault(m.group(0), f"o{len(ids)}"), " ".join(toks))


def normal(t):
    """A line of assembly as two spellings of it compare: lower case, one
    space, numbers in decimal, lists without inner spaces."""
    t = re.sub(r"\s+", " ", t.split("//")[0].replace("\t", " ")).strip().lower()
    t = re.sub(r"0x[0-9a-f]+", lambda m: str(int(m.group(0), 16)), t)
    t = re.sub(r"\{\s*", "{", t)
    t = re.sub(r"\s*\}", "}", t)
    t = re.sub(r"\s*-\s*", "-", t)
    return re.sub(r"\s*,\s*", ", ", t)


def same_text(a, b):
    return normal(a) == normal(b)


def objdump_words(words):
    """llvm-objdump's text for each word (None where it reads none)."""
    out = [None] * len(words)
    with tempfile.TemporaryDirectory() as tmp:
        src, obj = os.path.join(tmp, "w.s"), os.path.join(tmp, "w.o")
        with open(src, "w") as f:
            f.write("".join(f".inst 0x{w:08x}\n" for w in words))
        march = "-march=armv9.4-a" + accepted(CLANG_FEATURES + SCALABLE_FEATURES)
        subprocess.run(CLANG + [march, src, "-o", obj], check=True, capture_output=True)
        dump = subprocess.run([os.environ.get("LLVM_OBJDUMP", "llvm-objdump"), "-d", "--no-show-raw-insn", "--mattr=+all", obj],
                              capture_output=True, text=True).stdout
    for line in dump.splitlines():
        m = re.match(r"\s*([0-9a-f]+):\s+(.*)", line)
        if m and not m.group(2).startswith("<"):
            k = int(m.group(1), 16) // 4
            if k < len(out) and not m.group(2).startswith(("<unknown>", ".word", "udf")):
                out[k] = m.group(2)
    return out


def gather(w, positions):
    """The bits of `w` at `positions`, as a number (the first the top)."""
    x = 0
    for b in positions:
        x = (x << 1) | (w >> b & 1)
    return x
# ---------------------------------------------------------------------------
# Entries

CLASS = {"x": "x", "xsp": "xsp", "w": "w", "wsp": "wsp", "v": "q", "b": "b", "h": "h", "s": "s", "d": "d", "q": "q",
         "z": "z", "p": "p", "pn": "pn"}


def template(variant, holes, forms):
    """Our "asm" template for a variant: `{name}`, `{name:4s}` for a vector
    with its arrangement, `{name:.h}` for a lane, `{name:h64}` for a
    logical immediate, labels without `#`."""
    text, k = "", 0
    for p in variant:
        if p[0] == "lit":
            # (Braces, a register list's, as templates write them.)
            text += p[1].replace("{", "\x01").replace("}", "\x02")
        else:
            text += f"\x00{k}\x00"
            k += 1
    for k, h in enumerate(holes):
        mark = f"\x00{k}\x00"
        if h.after is not None:
            j, off = h.after
            if h.kind == "num":
                text = text.replace(mark, f"{{={holes[j].name}+{off}}}" if off else f"{{{holes[j].name}}}")
            else:
                text = text.replace(mark, f"{{{holes[j].name}:+{off}}}" if off else f"{{{holes[j].name}}}")
            continue
        if h.label:
            text = text.replace("#" + mark, mark)
        if h.kind == "v":
            text = re.sub(re.escape(mark) + r"\.(\d+[bhsdq])\b", f"{{{h.name}:\\1}}", text)
            text = re.sub(re.escape(mark) + r"\.([bhsdq])\b", f"{{{h.name}:.\\1}}", text)
        form = forms.get(k)
        mod = f":h{form[1]}" if form and form[0] == "bitmask" else ""
        text = text.replace(mark, f"{{{h.name}{mod}}}")
    return re.sub(r" +", " ", text).strip().replace("\x01", "{lb}").replace("\x02", "{rb}")


def entry(name, holes, toks, asm, decode, aliases):
    params = []
    for h in holes:
        if h.after is not None:
            continue
        if h.label:
            params.append(f'ptr "label" %{h.name}')
        elif h.kind == "num":
            params.append(f'i64 "imm" %{h.name}')
        else:
            cls = CLASS[h.kind]
            # (A predicate in three bits: p0-p7.)
            if cls == "p" and re.search(rf"\b{re.escape(h.name)}(_f)?:3\b", " ".join(toks)):
                cls = "pg"
            params.append(f'i64 "reg"="{cls}" %{h.name}')
    return (f'define void @{name}({", ".join(params)}) "enc"="{" ".join(toks)}" "asm"="{asm}" "asmonly"{"" if decode else ' "nodecode"'}'
            f'{f" \"aliases\"=\"{",".join(aliases)}\"" if aliases else ""} {{\n'
            f"  ret void\n}}\n")


def existing():
    """The description's own entries (not this file's): literal bits and
    mnemonic, so that what they already cover is left out."""
    out = []
    for path in sorted(os.listdir(HERE)):
        if not path.endswith(".ll") or path in ("80-asm.ll", "81-scalable.ll", "82-sve.ll"):
            continue
        for line in open(os.path.join(HERE, path)):
            m = re.match(r'define .*?@([A-Za-z0-9_.]+)\(.*?"enc"="([^"]*)"', line)
            if not m:
                continue
            mask = value = 0
            pos = 32
            for tok in m.group(2).split():
                if re.fullmatch(r"[01]+", tok):
                    for ch in tok:
                        pos -= 1
                        mask |= 1 << pos
                        value |= (ch == "1") << pos
                else:
                    pos -= int(re.match(r"[A-Za-z0-9_]+:s?(\d+)", tok).group(1))
            out.append((mask, value))
    return out


# ---------------------------------------------------------------------------
# Main


def main():
    global SCALABLE
    if len(sys.argv) < 2:
        sys.exit(__doc__)
    SCALABLE = "--scalable" in sys.argv[2:]
    insts, rules = load(sys.argv[1])
    stats = collections.Counter()
    jobs = []
    for ins in insts:
        try:
            vs = expand(ins["assembly"], rules)
        except RecursionError:
            vs = None
        if vs is None:
            stats["too many variants"] += 1
            continue
        seen = set()
        for var, _ in vs:
            var = merge_registers(var, ins["encoded_in"])
            key = re.sub(r"\s+", "", "".join(p[1] if p[0] == "lit" else f"<{p[2]}>" for p in var))
            if key in seen:
                continue
            seen.add(key)
            if not SCALABLE and any(p[0] == "lit" and ("{" in p[1] or "}" in p[1]) for p in var):
                stats["register list"] += 1
                continue
            jobs.append((ins, var))
    print(f"{len(insts)} instructions, {len(jobs)} templates", file=sys.stderr)
    for n in range(2):
        for ins, var in jobs:
            fit_variant(ins, var, None)
        print(f"pass {n + 1}: {len(PENDING)} lines for clang", file=sys.stderr)
        run_batch()
    ours = existing()
    entries = []
    covered = set()
    names = collections.Counter()
    # Encodings already written: another spelling of one (b.cs, b.hs) is
    # read, but not written by the disassembler.
    decoded = set()
    fitted = []
    for ins, var in jobs:
        r, why = fit_variant(ins, var, None)
        if r is None:
            stats[re.sub(r"<[^>]*>", "<..>", why)] += 1
            continue
        fitted.append((ins, var, r))
    # Of an encoding's spellings, the disassembler writes the one
    # llvm-objdump does (SVE's: `{ z0.s, z1.s }` but `{ z0.s - z3.s }`, mov
    # for cpy), where one is.
    preferred = set()
    if SCALABLE:
        texts = objdump_words([r[2] for _, _, r in fitted])
        for n, ((ins, var, r), text) in enumerate(zip(fitted, texts)):
            holes, base = r[0], r[5]
            if text is not None and same_text(line_of(var, holes, base), text):
                preferred.add(n)
    for n, (ins, var, r) in enumerate(fitted):
        holes, toks, w0, words, forms, _ = r
        if any(all(w & m == v for w in words) for m, v in ours):
            stats["the description has it"] += 1
            covered.add(ins["name"])
            continue
        k = names[ins["name"]]
        names[ins["name"]] += 1
        # The encoding, its operands numbered in order.
        ids = {}
        shape_ = re.sub(r"[a-z_][a-z0-9_]*", lambda m: ids.setdefault(m.group(0), f"o{len(ids)}"), " ".join(toks))
        if SCALABLE:
            # (The spelling llvm-objdump writes, else the first.)
            others = [m for m, (i2, v2, r2) in enumerate(fitted) if m in preferred and spelling(r2[1]) == shape_]
            decode = shape_ not in decoded and (n in preferred or not others and not ins["alias"])
        else:
            decode = shape_ not in decoded and not ins["alias"]
        if decode:
            decoded.add(shape_)
        if not decode:
            stats["another spelling"] += 1
        entries.append(entry(f"{ins['name']}.{k}", holes, toks, template(var, holes, forms), decode, ins["aliases"]))
        covered.add(ins["name"])
    stats["entries"] = len(entries)
    stats["instructions with an entry"] = len(covered)
    for k, v in sorted(stats.items(), key=lambda x: -x[1]):
        print(f"{v:6} {k}", file=sys.stderr)
    ref = json.load(open(sys.argv[1]))["_meta"]["version"]
    with open(os.path.join(HERE, "81-scalable.ll" if SCALABLE else "80-asm.ll"), "w") as f:
        f.write(NOTICE.format(ref=ref['ref'], what=SCALABLE_WHAT if SCALABLE else BASE_WHAT))
        f.write("\n".join(entries))


BASE_WHAT = """the A64 instructions (base, floating point and Advanced
; SIMD) the rest of the description does not have"""
SCALABLE_WHAT = """SVE's and SME's instructions (with --scalable): those
; of every SVE and SME extension clang knows"""
NOTICE = """; Entries for assembly and disassembly only ("asmonly"), written by
; gen_mrs_asm.py: {what}. Their bit layouts and
; assembly syntax are from Arm's A-profile Architecture Machine Readable
; Specification (AARCHMRS, {ref}), under this license:
;
; Copyright 2010-2026 Arm Limited or its affiliates.
;
; Redistribution and use in source and binary forms, with or without
; modification, are permitted provided that the following conditions are met:
;
; 1. Redistributions of source code must retain the above copyright notice,
;    this list of conditions and the following disclaimer.
; 2. Redistributions in binary form must reproduce the above copyright
;    notice, this list of conditions and the following disclaimer in the
;    documentation and/or other materials provided with the distribution.
; 3. Neither the name of the copyright holder nor the names of its
;    contributors may be used to endorse or promote products derived from
;    this software without specific prior written permission.
;
; THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS"
; AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE
; IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE
; ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE
; LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR
; CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF
; SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS
; INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN
; CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE)
; ARISING IN ANY WAY OUT OF THE USE OF THIS SOFTWARE, EVEN IF ADVISED OF THE
; POSSIBILITY OF SUCH DAMAGE.
;
; How each operand is held in its fields is not in the open-source
; specification: it was found by assembling lines with clang and reading
; the fields back. Each entry's words, for every line tried, are clang's.

"""


if __name__ == "__main__":
    main()
