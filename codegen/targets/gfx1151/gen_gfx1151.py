"""Writes the gfx1151 description: AMD's RDNA 3.5 GPUs (Strix Halo's
Radeon 8060S), for assembly and disassembly only so far.

From AMD's machine-readable ISA specification of RDNA 3.5
(`amdgpu_isa_rdna3_5.xml`, from AMD GPUOpen's "Machine-Readable GPU ISA",
MIT): its encodings, the fields of each, and every instruction's opcode in
each encoding it has, with its operands' fields, types and sizes. The
assembly syntax is LLVM's (`llvm-mc -triple=amdgcn-amd-amdhsa
-mcpu=gfx1151`, wave32). The specification has no syntax, so it is found
with llvm-mc's disassembler: each operand's values are put in a word one
at a time, and what llvm-mc writes for each is that value's name; where in
the line it writes them makes the template. Each entry is then checked on
random operand values against llvm-mc.

    python3 codegen/targets/gfx1151/gen_gfx1151.py amdgpu_isa_rdna3_5.xml

It needs llvm-mc with the AMDGPU target (LLVM 22; `LLVM_MC` names it),
and runs llvm-mc in up to 16 processes at once: about two hours for the
whole.
`GFX_ONLY`, a regular expression, keeps the encodings whose names it
matches (`GFX_ONLY='ENC_SOP.*'`), and `GFX_OUT` names another directory to
write to.
"""

import collections
import concurrent.futures
import itertools
import os
import random
import re
import subprocess
import sys
import xml.etree.ElementTree as ET

HERE = os.environ.get("GFX_OUT") or os.path.dirname(os.path.abspath(__file__))
# A pattern of encoding names, to generate only those (for working on one).
ONLY = os.environ.get("GFX_ONLY")
LLVM_MC = os.environ.get("LLVM_MC", "llvm-mc-22")


# ---------------------------------------------------------------------------
# The specification


class Encoding:
    def __init__(self, e):
        self.name = e.findtext("EncodingName")
        self.bits = int(e.findtext("BitCount"))
        self.mask = int(e.findtext("EncodingIdentifierMask"), 2)
        self.ids = [int(x.text, 2) for x in e.findall("EncodingIdentifiers/EncodingIdentifier")]
        # Each field's bit ranges, the value's low bits first.
        self.fields = {}
        for f in e.findall("MicrocodeFormat/BitMap/Field"):
            self.fields[f.findtext("FieldName")] = [
                (int(r.findtext("BitOffset")), int(r.findtext("BitCount"))) for r in f.findall("BitLayout/Range")
            ]


class Operand:
    def __init__(self, o):
        self.field = o.findtext("FieldName")
        self.type = o.findtext("OperandType")
        self.size = int(o.findtext("OperandSize") or 0)
        self.format = o.findtext("DataFormatName")
        self.implicit = o.get("IsImplicit") == "true"
        self.order = int(o.get("Order"))


class Form:
    """An instruction in one of its encodings."""

    def __init__(self, name, ie, encodings):
        self.name = name
        self.enc = encodings[ie.findtext("EncodingName")]
        self.cond = ie.findtext("EncodingCondition")
        self.opcode = int(ie.findtext("Opcode"))
        self.operands = sorted((Operand(o) for o in ie.findall("Operands/Operand")), key=lambda o: o.order)


def load(path):
    root = ET.parse(path).getroot()
    encodings = {}
    for e in root.findall(".//Encodings/Encoding"):
        enc = Encoding(e)
        encodings[enc.name] = enc
    forms = []
    for i in root.findall(".//Instructions/Instruction"):
        name = i.findtext("InstructionName")
        for ie in i.findall("InstructionEncodings/InstructionEncoding"):
            forms.append(Form(name, ie, encodings))
    return encodings, forms


def put(word, ranges, v):
    """`word` with field `ranges` set to `v`."""
    for lo, n in ranges:
        m = (1 << n) - 1
        word = (word & ~(m << lo)) | ((v & m) << lo)
        v >>= n
    return word


def get(word, ranges):
    v, at = 0, 0
    for lo, n in ranges:
        v |= ((word >> lo) & ((1 << n) - 1)) << at
        at += n
    return v


# ---------------------------------------------------------------------------
# llvm-mc

CACHE = {}
# While collecting, the oracle answers only from its cache, and records what
# it was asked that it does not have; `settle` fetches those in one batch and
# runs the pass again, until nothing is missing.
COLLECT = [False]
PENDING, PENDING_ASM = set(), set()
# Few enough not to take a shared machine.
WORKERS = min(16, os.cpu_count() or 4)


def _marker(k, j):
    """`s_nop 0x8000+4k+j`, the j-th of four after the k-th word of a batch:
    llvm-mc reads the input as one stream, so a word short of its literal
    takes the next bytes, and the numbered markers show where that happened
    (and the ones after it keep the next word to itself)."""
    n = 4 * k + j
    return bytes([n & 0xFF, 0x80 | (n >> 8), 0x80, 0xBF])


def _hexs(w):
    return ",".join(f"0x{b:02x}" for b in w)


def _mc(text):
    return subprocess.run(
        [LLVM_MC, "-triple=amdgcn-amd-amdhsa", "-mcpu=gfx1151", "-disassemble", "-show-encoding"],
        input=text,
        capture_output=True,
        text=True,
    )


def _alone(w):
    """One word, disassembled by itself."""
    out = _mc(_hexs(w) + "\n")
    lines = [l for l in out.stdout.splitlines() if "; encoding: [" in l]
    if len(lines) != 1 or "invalid" in out.stderr or "undefined" in out.stderr:
        return None
    t, enc = lines[0].split("; encoding: [")
    t = t.strip()
    return None if "/*" in t else (t, enc.rstrip("]").replace(" ", "") == _hexs(w))


def _run(chunk):
    text = "".join(_hexs(w) + "\n" + "".join(_hexs(_marker(k, j)) + "\n" for j in range(4)) for k, w in enumerate(chunk))
    out = _mc(text)
    bad = set()
    for l in out.stderr.splitlines():
        if l.startswith("<stdin>:") and ("invalid" in l or "undefined" in l):
            line = int(l.split(":")[1]) - 1
            if line % 5 == 0:
                bad.add(line // 5)
    groups, cur, first = {}, [], set()
    for l in out.stdout.splitlines():
        if "; encoding: [" not in l:
            continue
        t, enc = l.split("; encoding: [")
        t, enc = t.strip(), enc.rstrip("]").replace(" ", "")
        m = re.fullmatch(r"s_nop 0x([0-9a-f]+)", t)
        n = int(m.group(1), 16) - 0x8000 if m else -1
        if m and n >= 0 and enc == _hexs(_marker(n // 4, n % 4)):
            k = n // 4
            if k not in groups:
                groups[k] = cur
            if n % 4 == 0:
                first.add(k)
            cur = []
        else:
            cur.append((t, enc))
    res = []
    for k, w in enumerate(chunk):
        g = groups.get(k)
        # A word that took a marker is longer than itself: invalid alone.
        if g is None or k not in first or k in bad or len(g) != 1 or "/*" in g[0][0]:
            res.append(None)
        else:
            res.append((g[0][0], g[0][1] == _hexs(w)))
    return res


def prefetch(words):
    """Disassembles the words not yet seen, in parallel."""
    if COLLECT[0]:
        PENDING.update(w for w in words if w not in CACHE)
        return
    todo = list({w for w in words if w not in CACHE})
    # Words that look like markers: alone.
    for w in [w for w in todo if len(w) == 4 and w[2:] == b"\x80\xbf" and w[1] & 0x80]:
        todo.remove(w)
        CACHE[w] = _alone(w)
    if not todo:
        return
    size = min(2000, max(200, len(todo) // (4 * WORKERS) + 1))
    chunks = [todo[k : k + size] for k in range(0, len(todo), size)]
    with concurrent.futures.ThreadPoolExecutor(max_workers=WORKERS) as ex:
        for chunk, res in zip(chunks, ex.map(_run, chunks)):
            for w, r in zip(chunk, res):
                CACHE[w] = r


ASM = {}


def _assemble(chunk):
    """Each line's bytes as llvm-mc assembles it, or None."""
    out = subprocess.run(
        [LLVM_MC, "-triple=amdgcn-amd-amdhsa", "-mcpu=gfx1151", "-show-encoding"],
        input="".join(l + "\n" for l in chunk),
        capture_output=True,
        text=True,
    )
    bad = set()
    for l in out.stderr.splitlines():
        if l.startswith("<stdin>:"):
            bad.add(int(l.split(":")[1]) - 1)
    got = []
    for l in out.stdout.splitlines():
        if "; encoding: [" in l:
            enc = l.split("; encoding: [")[1].rstrip("]")
            got.append(bytes(int(b, 16) for b in enc.split(",") if b.strip()))
    it = iter(got)
    return [None if k in bad else next(it, None) for k in range(len(chunk))]


def assemble(lines):
    """llvm-mc's bytes for each line (cached), in parallel."""
    if COLLECT[0]:
        PENDING_ASM.update(l for l in lines if l not in ASM)
        return [ASM.get(l) for l in lines]
    todo = list({l for l in lines if l not in ASM})
    if todo:
        size = min(2000, max(200, len(todo) // (4 * WORKERS) + 1))
        chunks = [todo[k : k + size] for k in range(0, len(todo), size)]
        with concurrent.futures.ThreadPoolExecutor(max_workers=WORKERS) as ex:
            for chunk, res in zip(chunks, ex.map(_assemble, chunks)):
                for l, r in zip(chunk, res):
                    ASM[l] = r
    return [ASM[l] for l in lines]


def dis(word, nbytes):
    w = word.to_bytes(nbytes, "little")
    if w not in CACHE:
        if COLLECT[0]:
            PENDING.add(w)
            return None
        prefetch([w])
    return CACHE[w]


# ---------------------------------------------------------------------------
# Entries
#
# An entry is a form with some fields fixed, and the rest grouped into its
# operands. A group is the fields one operand holds, each a slice of the
# operand's value (`(field, shift)`: the field holds bits from `shift`), and
# the values it may take. A field `NEG.1` is bit 1 of field NEG.


def field_ranges(fields, name):
    if name in fields:
        return fields[name]
    m = re.fullmatch(r"(\w+)\[(\d+):(\d+)\]", name)
    if m:
        # Bits lo to lo+n of a field.
        base, lo, n = m.group(1), int(m.group(2)), int(m.group(3))
        bits = [b for r0, w in fields[base] for b in range(r0, r0 + w)][lo : lo + n]
        out = []
        for b in bits:
            if out and out[-1][0] + out[-1][1] == b:
                out[-1] = (out[-1][0], out[-1][1] + 1)
            else:
                out.append((b, 1))
        return out
    base, k = name.rsplit(".", 1)
    k, out, at = int(k), [], 0
    for lo, n in fields[base]:
        if at <= k < at + n:
            return [(lo + k - at, 1)]
        at += n
    raise KeyError(name)


class Group:
    def __init__(self, name, parts, values, kind="enum", signed=False, fmt="d", hidden=False, omit=None, bias=0, sub=0):
        self.name = name
        self.parts = parts
        self.values = values
        self.kind = kind  # enum or imm
        self.signed = signed
        # How an immediate is written: decimal, hex, or signed hex (-0x8).
        self.fmt = fmt
        # The value with which it is not written at all (an offset of 0, a
        # wait count at its most): an entry without it, and one with it.
        self.omit = omit
        # Written as its value plus this (hwreg's size, held less 1).
        self.bias = bias
        # Held less this (VOPD's second destination: its low bit is not
        # held, but is the other of the first's).
        self.sub = sub
        # An immediate with no hole of its own, written only by a computed
        # hole (an op_sel bit of a literal source).
        self.hidden = hidden
        self.names = {}  # value -> text, for enums
        self.fixed_only = False

    def width(self, entry):
        return sum(n for f, _ in self.parts for _, n in entry.ranges(f))

    def copy(self):
        g = Group(self.name, self.parts, self.values, self.kind, self.signed, self.fmt, self.hidden, self.omit, self.bias, self.sub)
        return g


class Entry:
    def __init__(self, form, tag, fixed, groups, guards=(), opsel=False):
        self.form = form
        self.tag = tag
        self.fixed = fixed  # the word with every fixed field set
        self.groups = groups
        self.guards = list(guards)  # (python expression, description expression)
        self.nbits = form.enc.bits
        # The op_sel bits are written again after the operands, as
        # `op_sel:[1,0,0]`, which this entry writes from its operands.
        self.opsel = opsel
        self.opsel_digits = None
        self.template = None
        self.why = None
        self.prefer = {}

    def ranges(self, f):
        return field_ranges(self.form.enc.fields, f)

    @property
    def nbytes(self):
        return self.nbits // 8

    def word(self, env):
        w = self.fixed
        for g in self.groups:
            v = env[g.name] - g.sub
            for f, shift in g.parts:
                r = self.ranges(f)
                width = sum(n for _, n in r)
                w = put(w, r, (v >> shift) & ((1 << width) - 1))
        return w

    def ok(self, env):
        return all(eval(compiled(p), {}, env) for p, _ in self.guards)


COMPILED = {}


def compiled(p):
    c = COMPILED.get(p)
    if c is None:
        c = COMPILED[p] = compile(p, "<guard>", "eval")
    return c


def baseline(entry):
    """Values for every group that llvm-mc reads back as the same word, from
    each group's preferred values in turn."""

    def good(env):
        r = entry.ok(env) and dis(entry.word(env), entry.nbytes)
        # (Valid, and read back: llvm-mc writes a carry-in in a VGPR, say,
        # that it does not read.)
        return bool(r) and r[1] and assemble([r[0]])[0] == entry.word(env).to_bytes(entry.nbytes, "little")

    tries = []
    for g in entry.groups:
        own = set(g.values)
        tries.append((g, [v for v in entry.prefer.get(g.name, []) + PREFER.get(g.name, []) if v in own] + g.values[:4]))
    # Each at its first preference: several guards may tie operands, so that
    # no one change from arbitrary values would satisfy them.
    env = {g.name: choices[0] for g, choices in tries}
    for _ in range(2):
        for g, choices in tries:
            keep = env[g.name]
            for v in choices:
                env[g.name] = v
                if good(env):
                    break
            else:
                env[g.name] = keep
        if good(env):
            return env
    return None


def middles(texts):
    """The part of each text that differs from the others."""
    ts = list(texts.values())
    p = os.path.commonprefix(ts)
    s = os.path.commonprefix([t[::-1] for t in ts])[::-1]
    # (The prefix and suffix may overlap where texts are short.)
    s = s[: max(0, min(len(t) for t in ts) - len(p))]
    return p, s, {v: t[len(p) : len(t) - len(s)] for v, t in texts.items()}


OPSEL = re.compile(r" op_sel:\[[01](?:,[01])*\]")


def strip(entry, text):
    return OPSEL.sub("", text, count=1) if entry.opsel else text


NAMES = {}


def learn(entry, base):
    """Each group's names and place in the line, from llvm-mc; the template."""
    r0 = dis(entry.word(base), entry.nbytes)
    if not r0:
        entry.why = "the baseline is not read"
        return False
    full0 = r0[0]
    texts0 = strip(entry, full0)
    regions = []
    for g in entry.groups:
        g.fixed_only = False
    for g in entry.groups:
        if g.kind != "enum":
            continue
        key = (entry.form.name, entry.form.cond, entry.tag, entry.fixed, g.name, tuple(sorted((k, v) for k, v in base.items() if k != g.name)))
        if key in NAMES:
            texts = NAMES[key]
        else:
            words = {}
            for v in g.values:
                env = dict(base)
                env[g.name] = v
                if entry.ok(env):
                    words[v] = entry.word(env)
            prefetch([w.to_bytes(entry.nbytes, "little") for w in words.values()])
            texts, missing = {}, False
            for v, w in words.items():
                r = dis(w, entry.nbytes)
                if r is None and w.to_bytes(entry.nbytes, "little") not in CACHE:
                    missing = True
                # Only values llvm-mc writes back as the same word: others
                # it reads as some other value of the group.
                if r and r[1]:
                    texts[v] = strip(entry, r[0])
            if not missing:
                NAMES[key] = texts
        if base[g.name] not in texts:
            entry.why = f"{g.name}: the baseline value is not read back"
            return False
        if len(set(texts.values())) < 2:
            # Only one way of writing it: no operand, the field fixed (at 0
            # if it may be: llvm-mc reads exp's `off` as 0).
            g.names = {0 if 0 in texts else base[g.name]: ""}
            g.fixed_only = True
            continue
        p, s, mids = middles(texts)
        if all(mids.values()):
            # Whole tokens: the name runs back to a space or comma.
            while p and p[-1] not in " ,":
                for v in mids:
                    mids[v] = p[-1] + mids[v]
                p = p[:-1]
            while s and s[0] not in " ,":
                for v in mids:
                    mids[v] = mids[v] + s[0]
                s = s[1:]
        g.names = mids
        more_names(entry, base, g)
        b = texts[base[g.name]]
        regions.append((len(p), len(b) - len(s), "{" + g.name + "}"))
    for g in entry.groups:
        if g.kind == "enum" or g.hidden:
            continue
        if not any(entry.ok(dict(base, **{g.name: v})) for v in g.values if v != base[g.name]):
            # Only one value it may take: written as fixed text.
            g.fixed_only = True
            continue
        spots = imm_spots(entry, base, g, texts0)
        if spots is None:
            entry.why = f"{g.name}: not found in the line"
            return False
        hole = hole_of(g)
        regions.extend((lo, hi, hole) for lo, hi in spots)
    if entry.opsel:
        m = OPSEL.search(full0)
        digits = opsel_digits(entry, base)
        if m and digits is not None:
            regions.append((m.start(), m.start(), " op_sel:[" + ",".join(d if d == "0" else "{=" + d + "}" for d in digits) + "]"))
            entry.opsel_digits = digits
        elif m or "op_sel" in full0:
            entry.why = "op_sel not understood"
            return False
        # (Else the halves are only in the operands' names: v_cndmask_b16
        # v1.h, v2.l, v3.l, s4.)
    # A literal source's op_sel bit not written in op_sel (in an operation
    # that has none) is fixed, at the baseline's 0.
    for g in entry.groups:
        if g.hidden and not any(re.search(rf"\b{g.name}\b", d) for d in entry.opsel_digits or []):
            g.fixed_only = True
    regions.sort(key=lambda r: (r[0], r[1]))
    regions = order_empty(entry, base, regions)
    tpl, at = "", 0
    for lo, hi, hole in regions:
        if lo < at:
            entry.why = f"{hole} overlaps another operand"
            return False
        tpl += escape(texts0[at:lo]) + hole
        at = hi
    tpl += escape(texts0[at:])
    entry.template = tpl
    # A group with one way of writing it is a fixed field.
    for g in [g for g in entry.groups if g.fixed_only]:
        v = next(iter(g.names)) if g.kind == "enum" else base[g.name]
        entry.fixed = fix(entry, g, v)
        # Its value in the guards and op_sel's digits that read it.
        sub = lambda t: re.sub(rf"\b{g.name}\b", str(v), t)
        entry.guards = [(sub(p), sub(d)) for p, d in entry.guards]
        if entry.opsel_digits:
            new = [sub(d) for d in entry.opsel_digits]
            for d, n in zip(entry.opsel_digits, new):
                entry.template = entry.template.replace("{=" + d + "}", "{=" + n + "}" if re.search(r"[a-z]", n) else str(eval(n)))
            entry.opsel_digits = [n for n in new if re.search(r"[a-z]", n)]
    entry.groups = [g for g in entry.groups if not g.fixed_only]
    # Only the names llvm-mc also reads: its disassembler writes some it
    # does not take (src_flat_scratch_base_lo).
    # Whether llvm-mc takes a name in a place depends on the place, not the
    # opcode: so what one entry finds is kept for others of the same
    # encoding and operand types, and each entry checks only a sample of
    # its names against that, all of them if the sample disagrees.
    sig = (entry.form.enc.name, tuple((o.field, o.type, o.size, o.format) for o in entry.form.operands))
    for g in entry.groups:
        if g.kind != "enum":
            continue
        vals = sorted(g.names)
        sample = set(vals[:: max(1, len(vals) // 16)]) | {base[g.name]}
        key = lambda v: (sig, g.name, g.names[v])
        # (A name asked about this round, by another entry, is not asked
        # again until it is answered.)
        todo = [v for v in vals if v in sample or (key(v) not in TAKEN and not (COLLECT[0] and key(v) in REQUESTED))]
        verdict = validate(entry, base, g, todo)
        checked = set(todo)
        if any(key(v) in TAKEN and verdict[v] in (True, False) and TAKEN[key(v)] != verdict[v] for v in sample if v in verdict):
            verdict.update(validate(entry, base, g, [v for v in vals if v not in verdict]))
            checked = set(vals)
        else:
            for v in vals:
                verdict.setdefault(v, TAKEN.get(key(v), True))
        for v, ok in verdict.items():
            # (Only that a name is taken is shared, and only as llvm-mc
            # said: a line may be refused for its other operands, VOPD's
            # banks or the constant bus.)
            if ok is True and v in checked:
                TAKEN.setdefault(key(v), ok)
            elif ok is None and v in todo:
                REQUESTED.add(key(v))
        for v in vals:
            if verdict.get(v) is False:
                del g.names[v]
        if base[g.name] not in g.names:
            entry.why = f"{g.name}: llvm-mc does not read the baseline back"
            return False
    # And the line itself (an entry may have no names to check).
    if assemble([render(entry, base)])[0] != entry.word(base).to_bytes(entry.nbytes, "little"):
        entry.why = "llvm-mc does not read the line back"
        return False
    return True


def order_empty(entry, base, regions):
    """Operands written nowhere at the baseline, at one place: in the order
    llvm-mc writes them when each is written (`format:70 dlc`)."""
    out, k = [], 0
    while k < len(regions):
        same = [r for r in regions[k:] if r[0] == regions[k][0] and r[1] == r[0]]
        if len(same) < 2 or regions[k][1] != regions[k][0]:
            out.append(regions[k])
            k += 1
            continue
        groups = {"{" + g.name + "}": g for g in entry.groups}
        env = dict(base)
        for _, _, hole in same:
            g = groups.get(hole)
            if g is None or g.kind != "enum":
                break
            # A value it is written with.
            v = next((v for v, n in g.names.items() if n and entry.ok(dict(env, **{g.name: v}))), None)
            if v is None:
                break
            env[g.name] = v
        else:
            r = dis(entry.word(env), entry.nbytes)
            if r:
                for perm in itertools.permutations(same):
                    t = strip(entry, r[0])
                    at = perm[0][0]
                    if t[at:].startswith("".join(groups[h].names[env[groups[h].name]] for _, _, h in perm)):
                        same = list(perm)
                        break
        out.extend(same)
        k += len(same)
    return out


TAKEN = {}
REQUESTED = set()
UNREAD = "unread"


def validate(entry, base, g, vals):
    """Whether llvm-mc reads each value's line back as its word (None while
    collecting, for lines not yet assembled)."""
    envs = [guarded(entry, dict(base, **{g.name: v}), g) for v in vals]
    lines = [render(entry, env) for env in envs if env]
    got = iter(assemble(lines))
    out = {}
    for v, env in zip(vals, envs):
        if env is None:
            # No line the guards allow has it, so none is read: kept, but
            # it says nothing of the name.
            out[v] = UNREAD
            continue
        l, b = render(entry, env), next(got)
        if b is None and l not in ASM:
            out[v] = None
        else:
            out[v] = b == entry.word(env).to_bytes(entry.nbytes, "little")
    return out


def more_names(entry, base, g):
    """Names of values the guards leave out beside the baseline's other
    operands (VOPD's source in the bank of the other half's): read beside
    others that allow them, as the token in the place of a named value's.
    (Only VOPD's guards tie one operand's values to another's.)"""
    if not entry.form.enc.name.startswith("VOPD") or any(c in n for n in g.names.values() for c in " ,"):
        return
    contexts = {}
    for v in g.values:
        if v not in g.names and not entry.ok(dict(base, **{g.name: v})):
            env = guarded(entry, dict(base, **{g.name: v}), g)
            if env:
                contexts.setdefault(tuple(sorted((k, x) for k, x in env.items() if k != g.name)), []).append(v)
    for key, vs in contexts.items():
        ctx = dict(key)
        ref = next((u for u in sorted(g.names) if entry.ok(dict(ctx, **{g.name: u}))), None)
        if ref is None:
            continue
        words = {v: entry.word(dict(ctx, **{g.name: v})) for v in vs + [ref]}
        prefetch([w.to_bytes(entry.nbytes, "little") for w in words.values()])
        texts = {}
        for v, w in words.items():
            r = dis(w, entry.nbytes)
            if r and r[1]:
                texts[v] = re.split(r"(\s+|,)", strip(entry, r[0]))
        if ref not in texts:
            continue
        for v in vs:
            t = texts.get(v)
            if t is None or len(t) != len(texts[ref]):
                continue
            diff = {k for k, (x, y) in enumerate(zip(t, texts[ref])) if x != y}
            if diff and all(texts[ref][k] == g.names[ref] for k in diff) and len({t[k] for k in diff}) == 1:
                g.names[v] = t[next(iter(diff))]


def guarded(entry, env, g):
    """`env`, with another operand changed where the guards need it (VOPD's
    halves' sources are in different banks), or None."""
    if entry.ok(env):
        return env
    for h in entry.groups:
        if h is not g:
            for v in PREFER.get(h.name, []) + h.values[:16] + h.values[:: max(1, len(h.values) // 64)]:
                e2 = dict(env, **{h.name: v})
                if v in h.values and entry.ok(e2):
                    return e2
    return None


def fix(entry, g, v):
    """The entry's fixed word with group `g` set to `v`."""
    w = entry.fixed
    v -= g.sub
    for f, shift in g.parts:
        r = entry.ranges(f)
        width = sum(n for _, n in r)
        w = put(w, r, (v >> shift) & ((1 << width) - 1))
    return w


def opsel_digits(entry, base):
    """What each digit of `op_sel:[...]` is, as an expression of the
    operands: found by setting each op_sel bit in turn."""
    holders = []
    for g in entry.groups:
        for f, shift in g.parts:
            if f.startswith("OP_SEL."):
                holders.append((g, shift, int(f.split(".")[1])))
    holders.sort(key=lambda h: h[2])
    zero = dict(base)
    for g, shift, _ in holders:
        zero[g.name] &= ~(1 << shift)
    by_digit = {}
    n = None
    for g, shift, bit in holders:
        env = dict(zero)
        env[g.name] |= 1 << shift
        if not entry.ok(env):
            continue
        r = dis(entry.word(env), entry.nbytes)
        if not r or not r[1]:
            continue
        m = OPSEL.search(r[0])
        if not m:
            continue
        ds = m.group(0).split("[")[1].rstrip("]").split(",")
        n = len(ds)
        if ds.count("1") != 1:
            return None
        by_digit[ds.index("1")] = f"({g.name}>>{shift})&1" if shift else f"{g.name}&1"
    if n is None or not by_digit:
        return None
    # (A digit no bit sets is 0: a compare's, for its scalar destination.)
    return [by_digit.get(k, "0") for k in range(n)]


def numeral(entry, g, v):
    width = g.width(entry)
    if g.bias:
        return str(v + g.bias)
    n = numerals(v, width, g.signed)
    if g.fmt == "sx":
        return n.get("sh", n["h"])
    return n["h"] if g.fmt == "h32" else n["d"]


def replacements(text, a, b, other):
    """The fewest occurrences of numeral `a` in `text` whose replacement by
    `b` makes it `other`, or None. (`v0 dpp8:[0,0,0]` writes 0 in several
    places, of which only some are the operand's.)"""
    at = [m.start() for m in re.finditer(re.escape(a), text)][:12]
    for n in range(1, len(at) + 1):
        for pick in itertools.combinations(at, n):
            t, last = "", 0
            for k in pick:
                t += text[last:k] + b
                last = k + len(a)
            t += text[last:]
            if t == other:
                return pick
    return None


def imm_spots(entry, base, g, text):
    """Where in `text` (the baseline's line) immediate `g` is written: the
    occurrences of its numeral that another value's line has its own
    numeral in place of."""
    other = dict(base)
    width = g.width(entry)
    for v in (0x2A, 0x1D, 0x31, 0x2B5, 0x3C, 0x12345679, 5, 3, 6, 2, 1, 0):
        if v < (1 << width) and v != base[g.name] and (v <= 64) == (base[g.name] <= 64):
            other[g.name] = v
            if entry.ok(other):
                break
    else:
        return None
    r = dis(entry.word(other), entry.nbytes)
    if not r:
        return None
    a, b = numeral(entry, g, base[g.name]), numeral(entry, g, other[g.name])
    pick = replacements(text, a, b, strip(entry, r[0]))
    return None if pick is None else [(k, k + len(a)) for k in pick]


def hole_of(g):
    if g.bias:
        return "{=" + g.name + f"+{g.bias}" + "}"
    return "{" + g.name + ("" if g.fmt == "d" else ":" + g.fmt) + "}"


def escape(s):
    return s.replace("{", "{lb}").replace("}", "{rb}")


def render(entry, env):
    out = entry.template
    for g in entry.groups:
        if g.kind == "enum":
            out = out.replace("{" + g.name + "}", g.names.get(env[g.name], "?"))
        elif not g.hidden:
            out = out.replace(hole_of(g), numeral(entry, g, env[g.name]))
    for d in entry.opsel_digits or []:
        out = out.replace("{=" + d + "}", str(eval(d, {}, dict(env))))
    return out.replace("{lb}", "{").replace("{rb}", "}")


def check(entry, n, rng):
    """Random operand values: the line llvm-mc writes for each must be ours."""
    envs = []
    choices = {g.name: [v for v in g.values if g.kind != "enum" or v in g.names] for g in entry.groups}
    for _ in range(n * 8):
        env = {k: rng.choice(v) for k, v in choices.items()}
        if entry.ok(env):
            envs.append(env)
        if len(envs) == n:
            break
    words = [entry.word(e) for e in envs]
    prefetch([w.to_bytes(entry.nbytes, "little") for w in words])
    for env, w in zip(envs, words):
        r = dis(w, entry.nbytes)
        want = render(entry, env)
        if not r or r[0] != want:
            entry.why = f"{env}: llvm-mc {r[0] if r else 'invalid'!r}, ours {want!r}"
            return False
    return True


# ---------------------------------------------------------------------------
# Immediates: where llvm-mc writes one, and how (decimal, or hex, or decimal
# up to 64 and hex above, as it writes an inline constant's range).


def numerals(v, width, signed):
    """A value's numerals; `v` is the operand's (signed, for a signed field)."""
    s = v - (1 << width) if not signed and v < 0 else v
    u = v & ((1 << width) - 1)
    out = {"d": str(s), "h": hex(u)}
    if s < 0:
        out["sh"] = "-" + hex(-s)
    return out


def learn_imm(entry, base, g, probes):
    """How llvm-mc writes imm group `g` at each probed value: the numeral
    format whose replacement turns the line into the line of a neighbouring
    value (the numeral may be written more than once)."""
    width = g.width(entry)
    formats = {}
    for v in probes:
        env = dict(base)
        env[g.name] = v
        if not entry.ok(env):
            continue
        r = dis(entry.word(env), entry.nbytes)
        if not r or not r[1]:
            formats[v] = None
            continue
        u = v ^ 1 if (v ^ 1) < (1 << width) else v - 1
        env2 = dict(env)
        env2[g.name] = u
        r2 = dis(entry.word(env2), entry.nbytes) if entry.ok(env2) else None
        found = None
        if r2:
            a, b = numerals(v, width, g.signed), numerals(u, width, g.signed)
            for f in ("h", "d", "sh"):
                if f in a and f in b and replacements(r[0], a[f], b[f], r2[0]) is not None:
                    found = f
                    break
        formats[v] = found
    return formats


def split_imms(e, base):
    """The entry's immediates learnt; an entry for each way llvm-mc writes
    them (decimal up to 64 and hex above are two)."""
    out = [e]
    for g in [g for g in e.groups if g.kind == "imm" and not g.hidden]:
        width = g.width(e)
        probes = sorted(v for v in {0, 1, 2, 5, 16, 63, 64, 65, 100, 255, 256, 0x1234, (1 << width) - 1,
                                    1 << (width - 1), (1 << (width - 1)) - 1} if 0 <= v < 1 << width)
        if g.signed:
            probes = [v - (1 << width) if v >> (width - 1) else v for v in probes]
        probes = sorted(set(probes) | {base[g.name]})
        formats = learn_imm(e, base, g, probes)
        seen = {f for f in formats.values() if f}
        if seen <= {"h"} and "h" in seen:
            g.fmt = "h32"
        elif seen <= {"d"} and "d" in seen:
            g.fmt = "d"
        elif seen == {"h", "sh"}:
            g.fmt = "sx"
        elif seen == {"d", "h"} and all(f == "d" for v, f in formats.items() if f and v <= 64) and all(
            f == "h" for v, f in formats.items() if f and v > 64
        ):
            # Two entries: decimal up to 64, hex above.
            lo = clone(e, e.tag + ".lo", [(f"{g.name} <= 64", f"{g.name}<=64")])
            hi = clone(e, e.tag + ".hi", [(f"{g.name} > 64", f"{g.name}>64")])
            [x for x in lo.groups if x.name == g.name][0].fmt = "d"
            [x for x in hi.groups if x.name == g.name][0].fmt = "h32"
            return [lo, hi], None
        else:
            return [], f"{g.name}: written {formats}"
    return out, None


def clone_fixed(e, tag, fixed):
    c = clone(e, tag)
    c.fixed = fixed
    return c


def clone(e, tag, guards=()):
    c = Entry(e.form, tag, e.fixed, [g.copy() for g in e.groups], e.guards + list(guards), e.opsel)
    c.prefer = dict(e.prefer)
    return c


# ---------------------------------------------------------------------------
# The families of encodings

SDST7 = list(range(128))


def literal_values(rng, n=24):
    vals = {0x12345678, 0x41200000, 0xDEADBEEF, 0x8000, 0x7FFF0000, 65, 0xFFFFFFEF, 100, 0x1234}
    while len(vals) < n:
        vals.add(rng.randrange(1 << 32))
    return sorted(vals)


def form_fixed(form):
    """The form's identifying bits and opcode."""
    enc = form.enc
    opf = enc.fields.get("OP")
    if opf is None:
        # One instruction in the encoding (exp): its identifier.
        return enc.ids[0] if len(enc.ids) == 1 else None
    want = put(0, opf, form.opcode)
    ids = [x for x in enc.ids if x & put(0, opf, -1) == want]
    return ids[0] if ids else None


SOURCES = {"SRC0": 0, "SRC1": 1, "SRC2": 2, "SSRC0": 0, "SSRC1": 1}
# Operand types written as numbers, not by name.
NUMBERS = {"OPR_SIMM16", "OPR_SIMM32", "OPR_SIMM8", "OPR_LABEL", "OPR_CLAUSE", "OPR_WAIT_EVENT", "OPR_SMEM_OFFSET"}
# Structured immediates: their parts are operands of their own.
STRUCTURED = {"OPR_HWREG", "OPR_SENDMSG", "OPR_WAITCNT", "OPR_DELAY", "OPR_WAITCNT_DEPCTR", "OPR_VERSION"}


def structured(o, f):
    """The groups of a structured immediate in field `f`, by llvm-mc's way
    of writing it; a list of alternatives, each (groups, guards, fixed
    parts)."""
    sub = lambda lo, n: f"{f}[{lo}:{n}]"
    if o.type == "OPR_HWREG":
        # hwreg(id, offset, size), the size held less 1; hwreg(id) for the
        # whole register.
        hid = Group("hwreg", [(sub(0, 6), 0)], list(range(64)))
        off = Group("hw_offset", [(sub(6, 5), 0)], list(range(32)), kind="imm")
        size = Group("hw_size", [(sub(11, 5), 0)], list(range(32)), kind="imm", bias=1)
        whole = "(hw_offset == 0) and (hw_size == 31)"
        return [
            ([hid], [], [(sub(6, 5), 0), (sub(11, 5), 31)]),
            ([hid, off, size], [(f"not ({whole})", "(hw_offset!=0)|(hw_size!=31)")], []),
        ]
    if o.type == "OPR_WAITCNT":
        # Each count left out at its most.
        return [([Group("expcnt", [(sub(0, 3), 0)], list(range(8)), kind="imm", omit=7),
                  Group("lgkmcnt", [(sub(4, 6), 0)], list(range(64)), kind="imm", omit=63),
                  Group("vmcnt", [(sub(10, 6), 0)], list(range(64)), kind="imm", omit=63)], [], [])]
    if o.type == "OPR_DELAY":
        return [([Group("instid0", [(sub(0, 4), 0)], list(range(16)), omit=0),
                  Group("instskip", [(sub(4, 3), 0)], list(range(8)), omit=0),
                  Group("instid1", [(sub(7, 4), 0)], list(range(16)), omit=0)], [], [])]
    if o.type == "OPR_WAITCNT_DEPCTR":
        parts = [("sa_sdst", 0, 1), ("va_vcc", 1, 1), ("vm_vsrc", 2, 3), ("hold_cnt", 7, 1), ("va_ssrc", 8, 1),
                 ("va_sdst", 9, 3), ("va_vdst", 12, 4)]
        return [([Group(n, [(sub(lo, w), 0)], list(range(1 << w)), kind="imm", omit=(1 << w) - 1) for n, lo, w in parts],
                 [], [])]
    if o.type == "OPR_SENDMSG":
        # sendmsg(MSG_...) by the low byte; a number above it, but where the
        # low byte is a message llvm-mc names, which it writes by the name
        # alone, losing the rest.
        named = (1, 2, 3, 5, 6, 9, 128, 129, 130, 131, 132, 133, 134)
        raw = [("simm16 >= 256", "simm16>=256")] + [(f"(simm16 & 255) != {m}", f"(simm16&255)!={m}") for m in named]
        return [
            ([Group("msg", [(sub(0, 8), 0)], list(range(256)))], [], [(sub(8, 8), 0)]),
            ([Group("simm16", [(f, 0)], list(range(256, 1 << 16)), kind="imm")], raw, []),
        ]
    return None
# Encodings whose op_sel bits are each a source's (or the destination's)
# half, written again after the operands.
OPSEL_ENCODINGS = {"ENC_VOP3", "VOP3_INST_LITERAL", "ENC_VINTERP", "VOP3_VOP_DPP8", "VOP3_VOP_DPP16"}


def alu(form, rng):
    """The scalar and vector ALU encodings and their literal forms: a group
    per operand the form names; in VOP3, each source with its negation,
    absolute value and op_sel bit, the destination with its op_sel bit, and
    the clamp and output modifier together; in VOP3P, every modifier
    together."""
    fixed = form_fixed(form)
    if fixed is None:
        return None
    enc = form.enc
    fields = enc.fields
    lits = re.findall(r"(?<!Not)has_lit_(\d)", form.cond or "") or (["0"] if form.cond == "has_lit" else [])
    packed = enc.name.startswith("ENC_VOP3P") or enc.name.startswith("VOP3P")
    opsel = enc.name in OPSEL_ENCODINGS and "OP_SEL" in fields
    groups, seen, guards = [], set(), []
    lit_srcs = set()
    structured_alts = [([], [], [])]
    for o in form.operands:
        if o.implicit or not o.field or o.field in seen:
            continue
        seen.add(o.field)
        f = o.field
        if o.type in STRUCTURED:
            alts = structured(o, f)
            if alts is None:
                return None
            structured_alts = alts
            continue
        if f == "LITERAL":
            if o.type == "OPR_SRC" or o.type == "OPR_SSRC":
                lit_srcs.add(len(lit_srcs))
            groups.append(Group("lit", [("LITERAL", 0)], literal_values(rng), kind="imm", fmt="h32"))
            if o.size == 16:
                guards.append(("lit < 65536", "lit<65536"))
            continue
        width = sum(n for _, n in fields[f])
        if o.type in NUMBERS or width > 9:
            # (llvm-mc writes a branch's offset unsigned.)
            vals = list(range(1 << width)) if width <= 16 else literal_values(rng)
            groups.append(Group(f.lower(), [(f, 0)], vals, kind="imm"))
            continue
        parts = [(f, 0)]
        bits = [width]
        k = SOURCES.get(f)
        if k is not None and not packed and "SSRC" not in f:
            for m, b in (("NEG", 9), ("ABS", 10)):
                if m in fields:
                    parts.append((f"{m}.{k}", b))
            if opsel:
                parts.append((f"OP_SEL.{k}", 11))
        if f == "VDST" and opsel:
            parts.append(("OP_SEL.3", 8))
        if f in ("VSRC0", "VSRC1") and ("DPP_CTRL" in fields or "LANE_SEL_0" in fields):
            k = int(f[-1])
            if "NEG" in fields and f == "VSRC0" and not packed:
                # VOP3's: its source 0's modifiers.
                for m, b in (("NEG", 9), ("ABS", 10)):
                    if m in fields:
                        parts.append((f"{m}.0", b))
                if opsel:
                    parts.append(("OP_SEL.0", 11))
            elif f"SRC{k}_NEG" in fields and "NEG" not in fields:
                parts += [(f"SRC{k}_NEG", 9), (f"SRC{k}_ABS", 10)]
        values = [0]
        for p, b in parts:
            n = sum(x for _, x in field_ranges(fields, p))
            values = [v | (x << b) for v in values for x in range(1 << n)]
        if k is not None:
            # (255 is the literal, which has forms of its own.)
            values = [v for v in values if v & ((1 << width) - 1) != 255]
        if o.type == "OPR_EXEC":
            # v_cmpx's: exec_lo, not written.
            values = [126]
        if o.type == "OPR_SDST_NULL":
            # (llvm-mc reads any register where the specification has only
            # null, but writes only null.)
            values = [124]
        groups.append(Group(f.lower(), parts, sorted(values)))
    # The sources the literal is in: fixed at 255; their modifiers fixed
    # (an entry for each), and their op_sel bits written only in op_sel.
    srcf = "SSRC" if "SSRC0" in fields else "SRC"
    fixed_mods = []
    for k in lits:
        if f"{srcf}{k}" not in fields:
            return None
        fixed = put(fixed, fields[f"{srcf}{k}"], 255)
        if not packed:
            for m in ("NEG", "ABS"):
                if m in fields:
                    fixed_mods.append(f"{m}.{k}")
            if opsel:
                groups.append(Group(f"h{k}", [(f"OP_SEL.{k}", 0)], [0, 1], kind="imm", hidden=True))
    # The constant bus: at most two scalar values (a register read twice is
    # one; the literal is one), which llvm-mc keeps to.
    scalars = [g.name for g in groups if g.name in ("src0", "src1", "src2")]
    # (One, in the 64-bit shifts.)
    bus = 1 if re.fullmatch(r"V_(LSHLREV|LSHRREV|ASHRREV)_[BI]64", form.name) else 2
    if len(scalars) + bool(lits) > bus:
        terms = []
        for k, n in enumerate(scalars):
            t = f"(({n}&511)<128)"
            for m in scalars[:k]:
                t += f"*(({n}&511)!=({m}&511))"
            terms.append(t)
        expr = "+".join(terms) + f"+{int(bool(lits))}<={bus}"
        guards.append((expr, expr))
    # DPP's controls; source 0 is 250 for DPP16 and 233 for DPP8 (234 with
    # fetch-inactive).
    dpp8 = []
    if "DPP_CTRL" in fields:
        fixed = put(fixed, fields["SRC0"], 250)
        groups.append(Group("dpp_ctrl", [("DPP_CTRL", 0)], list(range(512))))
        groups.append(Group("row_mask", [("ROW_MASK", 0)], list(range(16)), kind="imm"))
        groups.append(Group("bank_mask", [("BANK_MASK", 0)], list(range(16)), kind="imm"))
        groups.append(Group("dpp_flags", [("BOUND_CTRL", 0), ("FI", 1)], [0, 1, 2, 3]))
        if "NEG" in fields:
            for x in ("SRC0_NEG", "SRC0_ABS", "SRC1_NEG", "SRC1_ABS"):
                fixed = put(fixed, fields[x], 0)
    if "LANE_SEL_0" in fields:
        dpp8 = [233, 234]
        for k in range(8):
            groups.append(Group(f"lane{k}", [(f"LANE_SEL_{k}", 0)], list(range(8)), kind="imm"))
    # The modifiers written after the operands.
    if packed:
        mods = [("CLAMP", 0), ("NEG", 1), ("NEG_HI", 4), ("OP_SEL", 7), ("OP_SEL_HI", 10)]
    else:
        mods = [("CLAMP", 0), ("OMOD", 1), ("WAIT_EXP", 3)]
    mods = [(m, b) for m, b in mods if m in fields]
    if mods:
        values = [0]
        for p, b in mods:
            n = sum(x for _, x in fields[p])
            values = [v | (x << b) for v in values for x in range(1 << n)]
        groups.append(Group("mods", mods, sorted(values)))
    out = []
    for combo in itertools.product((0, 1), repeat=len(fixed_mods)):
        fx = fixed
        tag = enc.name
        if dpp8:
            fx = put(fx, fields["SRC0"], dpp8[0])
        for m, v in zip(fixed_mods, combo):
            fx = put(fx, field_ranges(fields, m), v)
            if v:
                tag += "." + m.replace(".", "").lower()
        if opsel:
            bits = [f"(({g.name}>>{s})&1)" if s else f"({g.name}&1)" for g in groups for p, s in g.parts if p.startswith("OP_SEL.")]
            if bits:
                none = "+".join(bits) + "==0"
                some = "+".join(bits) + "!=0"
                gs = [g.copy() for g in groups]
                out.append(Entry(form, tag, fx, gs, guards + [(none, none)]))
                e = Entry(form, tag + ".opsel", fx, [g.copy() for g in groups], guards + [(some, some)], opsel=True)
                # A baseline with op_sel bits set: every operand's at first,
                # as a compare's scalar destination has none.
                for g in e.groups:
                    for p, s in g.parts:
                        if p.startswith("OP_SEL."):
                            e.prefer[g.name] = [v | (1 << s) for v in PREFER.get(g.name, [])] + [1 << s]
                            break
                out.append(e)
                continue
        out.append(Entry(form, tag, fx, [g.copy() for g in groups], list(guards)))
    if dpp8:
        # The same with fetch-inactive: source 0 is 234.
        out += [clone_fixed(e, e.tag + ".fi", put(e.fixed, fields["SRC0"], dpp8[1])) for e in out]
    more = []
    for e in out:
        for k, (gs, gd, fixes) in enumerate(structured_alts):
            x = clone(e, e.tag + (f".{k}" if len(structured_alts) > 1 else ""), gd)
            x.groups += [g.copy() for g in gs]
            for p, v in fixes:
                x.fixed = put(x.fixed, field_ranges(fields, p), v)
            more.append(x)
    return more


# Flags that change the operands as well as being written: fixed, an entry
# for each (glc only in atomics, where it makes them return).
MEM_SPLIT = ("IDXEN", "OFFEN", "TFE", "SVE", "LWE")
MEM_FLAGS = ("GLC", "SLC", "DLC", "GDS")
SIGNED_OFFSET = ("ENC_SMEM", "ENC_FLAT_GLOBAL", "ENC_FLAT_SCRATCH")


def mem(form, rng):
    """Scalar memory, LDS, buffers, flat, global and scratch: registers by
    name; offsets as numbers, left out when 0; the cache flags together."""
    fixed = form_fixed(form)
    if fixed is None:
        return None
    enc = form.enc
    fields = enc.fields
    groups, seen = [], set()
    for o in form.operands:
        if o.implicit or not o.field or o.field in seen:
            continue
        seen.add(o.field)
        f = o.field
        width = sum(n for _, n in fields[f])
        if width > 9:
            return None
        groups.append(Group(f.lower(), [(f, 0)], list(range(1 << width))))
    split = [f for f in MEM_SPLIT if f in fields]
    flags = [f for f in MEM_FLAGS if f in fields]
    if "ATOMIC" in form.name and "GLC" in flags:
        flags.remove("GLC")
        split.append("GLC")
    if flags:
        groups.append(Group("cpol", [(f, k) for k, f in enumerate(flags)], list(range(1 << len(flags)))))
    if "FORMAT" in fields:
        groups.append(Group("format", [("FORMAT", 0)], list(range(128))))
    if "OFFSET" in fields:
        width = sum(n for _, n in fields["OFFSET"])
        signed = enc.name in SIGNED_OFFSET
        lo = -(1 << (width - 1)) if signed else 0
        hi = (1 << (width - 1)) if signed else 1 << width
        vals = sorted({lo, hi - 1, 1, 2, 4, 16, 255, 256, -1 if signed else 3, -8 if signed else 8} | {rng.randrange(lo, hi) for _ in range(20)})
        groups.append(Group("offset", [("OFFSET", 0)], vals, kind="imm", signed=signed, omit=0))
    if "OFFSET0" in fields:
        if "2ADDR" in form.name:
            for k in (0, 1):
                groups.append(Group(f"offset{k}", [(f"OFFSET{k}", 0)], list(range(256)), kind="imm", omit=0))
        else:
            groups.append(Group("offset", [("OFFSET0", 0), ("OFFSET1", 8)], list(range(1 << 16)), kind="imm", omit=0))
    if enc.name == "ENC_FLAT" and "saddr" not in [g.name for g in groups]:
        fixed = put(fixed, fields["SADDR"], 124)
    out = []
    for combo in itertools.product((0, 1), repeat=len(split)):
        fx, tag = fixed, enc.name
        for f, v in zip(split, combo):
            fx = put(fx, fields[f], v)
            if v:
                tag += "." + f.lower()
        gs = [g.copy() for g in groups]
        if enc.name == "ENC_FLAT_GLOBAL" and any(g.name == "saddr" for g in gs):
            # The address a 64-bit register without saddr, 32-bit with.
            off = Entry(form, tag + ".off", put(fx, fields["SADDR"], 124), [g for g in gs if g.name != "saddr"])
            out.append(off)
            out.append(Entry(form, tag + ".saddr", fx, [g.copy() for g in groups], [("saddr != 124", "saddr!=124")]))
            continue
        out.append(Entry(form, tag, fx, gs))
    return out


def fields_as_operands(form, rng, split=(), together=()):
    """Every field an operand by name, but `split` (fixed, an entry for each
    value) and `together` (lists of fields that are one operand)."""
    fixed = form_fixed(form)
    if fixed is None:
        return None
    fields = form.enc.fields
    joined = {f for fs in together for f in fs}
    groups = []
    for fs in together:
        if all(f in fields for f in fs):
            parts, at = [], 0
            for f in fs:
                parts.append((f, at))
                at += sum(n for _, n in fields[f])
            groups.append(Group(fs[0].lower(), parts, list(range(1 << at))))
    for f, r in fields.items():
        if f in ("ENCODING", "OP") or f in joined or f in split:
            continue
        width = sum(n for _, n in r)
        if width > 9:
            return None
        groups.append(Group(f.lower(), [(f, 0)], list(range(1 << width))))
    out = []
    split = [f for f in split if f in fields]
    for combo in itertools.product(*[range(1 << sum(n for _, n in fields[f])) for f in split]):
        fx, tag = fixed, form.enc.name
        for f, v in zip(split, combo):
            fx = put(fx, fields[f], v)
            tag += f".{f.lower()}{v}"
        out.append(Entry(form, tag, fx, [g.copy() for g in groups]))
    return out


FAMILIES = {name: mem for name in ("ENC_SMEM", "ENC_DS", "ENC_MUBUF", "ENC_MTBUF", "ENC_FLAT", "ENC_FLAT_GLOBAL", "ENC_FLAT_SCRATCH")}
FAMILIES["ENC_EXP"] = lambda form, rng: fields_as_operands(form, rng, split=("EN",), together=(("DONE", "ROW_EN"),))
FAMILIES["ENC_LDSDIR"] = lambda form, rng: fields_as_operands(form, rng, together=(("ATTR", "ATTR_CHAN"),))
FAMILIES.update({name: alu for name in (
    "ENC_SOP1", "ENC_SOP2", "ENC_SOPC", "ENC_SOPK",
    "SOP1_INST_LITERAL", "SOP2_INST_LITERAL", "SOPC_INST_LITERAL", "SOPK_INST_LITERAL",
    "ENC_VOP1", "ENC_VOP2", "ENC_VOPC", "VOP1_INST_LITERAL", "VOP2_INST_LITERAL", "VOPC_INST_LITERAL",
    "ENC_VOP3", "VOP3_INST_LITERAL", "VOP3_SDST_ENC", "VOP3_SDST_ENC_INST_LITERAL",
    "ENC_VOP3P", "VOP3P_INST_LITERAL", "ENC_SOPP", "ENC_VINTERP",
    "VOP1_VOP_DPP16", "VOP1_VOP_DPP8", "VOP2_VOP_DPP16", "VOP2_VOP_DPP8", "VOPC_VOP_DPP16", "VOPC_VOP_DPP8",
    "VOP3_VOP_DPP16", "VOP3_VOP_DPP8", "VOP3_SDST_ENC_VOP_DPP16", "VOP3_SDST_ENC_VOP_DPP8",
    "VOP3P_VOP_DPP16", "VOP3P_VOP_DPP8",
)})

# Preferred baseline values, by group name: distinct registers, so that
# each operand can be told apart in the line.
PREFER = {
    "sdst": [5, 4, 6, 8, 124, 106, 0],
    "ssrc0": [2, 4, 0, 128, 106],
    "ssrc1": [3, 6, 8, 129, 106],
    "vdst": [1, 4, 8, 0, 106],
    "src0": [258, 260, 2, 4, 128],
    "src1": [259, 262, 3, 6, 129],
    "src2": [260, 264, 4, 8, 130],
    "vsrc1": [3, 6, 2],
    "lit": [0x12345678, 0x41200000, 0x1234],
    "simm16": [0x1234, 100, 5, 0],
    # (VOP3P's op_sel_hi is all ones unless written.)
    "mods": [0, 7 << 10],
    "vsrc0": [2, 4, 6],
    "vdata": [8, 4, 6, 0],
    "vaddr": [2, 4, 0],
    "addr": [2, 4, 0],
    "data": [6, 8, 4],
    "data0": [6, 8, 4],
    "data1": [8, 10, 12],
    "srsrc": [1, 2, 0],
    "soffset": [3, 5, 124, 128],
    "saddr": [2, 4, 124],
    "sbase": [1, 2, 0],
    "sdata": [4, 8, 6, 0],
    "cpol": [0],
    "format": [0, 1],
    "offset": [16, 8, 4, 1],
    "offset0": [16, 8, 1],
    "offset1": [32, 9, 2],
    "dpp_ctrl": [0xE4, 0x101, 0],
    "row_mask": [15, 0xA],
    "bank_mask": [15, 0xB],
    "dpp_flags": [0],
    "hwreg": [1, 2, 3],
    "hw_offset": [3, 1],
    "hw_size": [4, 1],
    "msg": [1, 2, 3],
    "expcnt": [1, 2],
    "lgkmcnt": [5, 2],
    "vmcnt": [9, 3],
    "instid0": [1, 2],
    "instskip": [1, 2],
    "instid1": [2, 3],
    "tgt": [0, 1],
    "vsrc2": [4, 6],
    "vsrc3": [5, 7],
    "done": [0],
    "attr": [0, 1],
    "wait_vdst": [0, 1],
    "vdstx": [0, 1, 2],
    "vdsty": [3, 2, 5],
    "srcx0": [257, 2, 128],
    "vsrcx1": [2, 6],
    "srcy0": [260, 3, 129],
    "vsrcy1": [5, 7],
}


# Values a literal could hold that an inline constant also can: the
# integers -16 to 64, and 0.5, 1.0, 2.0, 4.0 (each signed) and 1/(2*pi) in
# f16, bf16, f32 and the high half of f64. llvm-mc writes such a literal
# `lit(...)`, to keep it a literal; these entries leave them out.
INLINE = sorted(
    set(range(65))
    | set(range((1 << 32) - 16, 1 << 32))
    | set(range(0xFFF0, 0x10000))
    | {0x3800, 0xB800, 0x3C00, 0xBC00, 0x4000, 0xC000, 0x4400, 0xC400, 0x3118}
    | {0x3F00, 0xBF00, 0x3F80, 0xBF80, 0x4000, 0xC000, 0x4080, 0xC080, 0x3E22}
    | {0x3F000000, 0xBF000000, 0x3F800000, 0xBF800000, 0x40000000, 0xC0000000, 0x40800000, 0xC0800000, 0x3E22F983}
    | {0x3FE00000, 0xBFE00000, 0x3FF00000, 0xBFF00000, 0x40000000, 0xC0000000, 0x40100000, 0xC0100000, 0x3FC45F30}
)


def literal_guards(e, base, g):
    """Guards leaving out the literals llvm-mc writes as `lit(...)`, or reads
    back as inline constants."""
    out = []
    for v in INLINE:
        env = dict(base)
        env[g.name] = v
        if not e.ok(env):
            continue
        r = dis(e.word(env), e.nbytes)
        # (Or that llvm-mc reads back as an inline constant: a 64-bit
        # operation's `0x1`.)
        if not r or not r[1] or "lit(" in r[0] or assemble([r[0]])[0] != e.word(env).to_bytes(e.nbytes, "little"):
            out.append(v)
    out = set(out)
    guards = []
    for lo, hi in ((0, 64), ((1 << 32) - 16, (1 << 32) - 1), (0xFFF0, 0xFFFF)):
        run = set(range(lo, hi + 1))
        if run <= out:
            guards.append(f"({g.name}<{lo})|({g.name}>{hi})" if lo else f"{g.name}>{hi}")
            out -= run
    guards += [f"{g.name}!={v}" for v in sorted(out)]
    return [(p.replace("|", " or ").replace("!=", " != "), p) for p in guards]


def omitted(e):
    """An entry for each way of writing its immediates that are left out
    when 0: without (the field 0), and with (not 0)."""
    out = [e]
    for g in [g for g in e.groups if g.omit is not None]:
        nxt = []
        for x in out:
            without = clone(x, x.tag)
            without.fixed = fix(without, g, g.omit)
            without.groups = [h for h in without.groups if h.name != g.name]
            with_ = clone(x, x.tag + "." + g.name, [(f"{g.name} != {g.omit}", f"{g.name}!={g.omit}")])
            nxt += [without, with_]
        out = nxt
    return out


def build(form, rng):
    """The form's entries, each with its baseline, ready to learn."""
    fam = FAMILIES.get(form.enc.name)
    if fam is None:
        return [], f"{form.enc.name} not yet"
    made = fam(form, rng)
    if made is None:
        return [], "operands not understood"
    out, why = [], None
    made = [v for e in made for v in omitted(e)]
    for e in made:
        base = baseline(e)
        if base is None:
            why = "no baseline llvm-mc reads"
            continue
        for g in e.groups:
            if g.name == "lit":
                e.guards += literal_guards(e, base, g)
        es, w = split_imms(e, base)
        why = why or w
        out.extend(es)
    return out, (why if not out else None)


# ---------------------------------------------------------------------------
# Writing the description


def encoding(entry):
    """The `"enc"` attribute: the word most significant bit first, in pieces
    of up to 64 bits (a literal after the instruction is a piece of its
    own), each field holding a slice of its operand."""
    # Each bit's owner: (token, its top bit) for fields; None for fixed bits.
    owner = {}
    for g in entry.groups:
        for f, shift in g.parts:
            at = shift
            r = entry.ranges(f)
            for lo, n in r:
                tok = re.sub(r"\W", "", f.lower()) + f"_{lo}"
                nm = f"({g.name}-{g.sub})" if g.sub else g.name
                expr = g.name if (at == 0 and not g.sub and len(g.parts) == 1 and len(r) == 1 and n == g.width(entry)) else (
                    f"({nm}>>{at})&{(1 << n) - 1}" if at else f"{nm}&{(1 << n) - 1}")
                signed = g.signed and at + n == g.width(entry)
                for b in range(lo, lo + n):
                    owner[b] = (tok, lo + n - 1, n, expr, signed)
                at += n
    pieces = []
    for top in range(0, entry.nbits, 64):
        hi = min(entry.nbits, top + 64)
        toks, bits = [], ""
        b = hi - 1
        while b >= top:
            o = owner.get(b)
            if o is None:
                bits += str((entry.fixed >> b) & 1)
                b -= 1
                continue
            if bits:
                toks.append(bits)
                bits = ""
            tok, t, n, expr, signed = o
            w = f"s{n}" if signed else str(n)
            toks.append(f"{tok}:{w}" if expr == tok else f"{tok}:{w}={expr}")
            b -= n
        if bits:
            toks.append(bits)
        pieces.append(" ".join(toks))
    body = " | ".join(pieces)
    if entry.guards:
        return "{ " + " ".join(f"if({d})" for _, d in entry.guards) + " " + body + " }"
    return body


def ll_name(s):
    return re.sub(r"[^A-Za-z0-9_.]", "_", s)


class Enums:
    """The enums the entries use, one per distinct set of names."""

    def __init__(self):
        self.by_names = {}
        self.order = []

    def get(self, names):
        key = tuple(sorted(names.items()))
        if key not in self.by_names:
            n = f"e{len(self.order)}"
            self.by_names[key] = n
            self.order.append((n, names))
        return self.by_names[key]


def merge_enums(made, most=8):
    """Each enum operand's names replaced by a larger table's that agrees
    with them, the values it lacks left out by guards (at most `most` runs
    of them): a source without 255 or without `vcc_lo` then shares its
    table with one that has them, and a table is written once, not once for
    each small difference."""
    used = [(e, g) for e in made for g in e.groups if g.kind == "enum" and not g.fixed_only]
    tables = sorted({tuple(sorted(g.names.items())) for _, g in used}, key=len, reverse=True)
    heads, canon = [], {}
    for t in tables:
        for h in heads:
            if len(h) >= len(t) and all(h.get(v) == n for v, n in t):
                rs = runs(sorted(set(h) - {v for v, _ in t}))
                if len(rs) <= most:
                    canon[t] = (h, rs)
                    break
        else:
            heads.append(dict(t))
            canon[t] = (heads[-1], [])
    for e, g in used:
        h, rs = canon[tuple(sorted(g.names.items()))]
        g.names = h
        for lo, hi in rs:
            p = f"{g.name}!={lo}" if lo == hi else f"({g.name}<{lo})|({g.name}>{hi})"
            e.guards.append((p.replace("|", " or ").replace("!=", " != "), p))


def runs(values):
    """Sorted values as (first, last) runs."""
    out = []
    for v in values:
        if out and v == out[-1][1] + 1:
            out[-1] = (out[-1][0], v)
        else:
            out.append((v, v))
    return out


def pattern_escape(name):
    # Braces in a name are a pattern's; none of these names has them.
    assert "{" not in name and "}" not in name, name
    return name.replace("\\", "\\5C").replace('"', "\\22")


def write(entries, enums, path, header):
    lines = [header]
    used = collections.Counter()
    for e in entries:
        tag = e.tag.replace("ENC_", "").replace("_INST_LITERAL", ".lit")
        if e.form.cond and e.form.cond not in ("default",):
            tag += "." + e.form.cond.replace("has_", "")
        name = ll_name(e.form.name + "." + tag)
        used[name] += 1
        if used[name] > 1:
            name += f".{used[name]}"
        ops = []
        for g in e.groups:
            if g.kind == "enum":
                if getattr(g, "fixed_only", False):
                    continue
                ops.append(f'i32 "enum"="{enums.get(g.names)}" %{g.name}')
            elif g.kind == "label":
                ops.append(f'ptr "label" %{g.name}')
            else:
                ops.append(f'i64 "imm" %{g.name}')
        asm = e.template.replace("\\", "\\5C").replace('"', "\\22")
        enc = encoding(e)
        lines.append(f'define void @{name}({", ".join(ops)}) "enc"="{enc}" "asm"="{asm}" "asmonly" {{\n  ret void\n}}')
        for suffix, ops2, enc2, asm2 in spellings(e, ops, enc, asm):
            lines.append(
                f'define void @{name}.{suffix}({", ".join(ops2)}) "enc"="{enc2}" "asm"="{asm2}" "asmonly" "nodecode" {{\n  ret void\n}}'
            )
    open(path, "w").write("\n".join(lines) + "\n")


def fixed_in_guards(enc, name, v):
    """`enc` with operand `name` fixed at `v`: its guards, now constant,
    left out (they hold for `v`)."""
    def keep(m):
        p = re.sub(rf"\b{name}\b", str(v), m.group(1))
        if re.search(r"[a-z]", p):
            return f"if({p}) "
        assert eval(p.replace("|", " or ")), p
        return ""
    enc = re.sub(r"if\(((?:[^()]|\([^()]*\))*)\) ", keep, enc)
    return enc[2:-2] if enc.startswith("{ ") and "if(" not in enc else enc


def spellings(e, ops, enc, asm):
    """Other ways the assembler reads an entry's lines, which the
    disassembler does not write: a branch to a label (`s_branch .LBB0_2`;
    the offset is in words from the next instruction), and a scalar load's
    zero offset written `0x0` where llvm-mc writes `null`, as clang does;
    and a 16-bit operation's halves written only in its operands' names
    (`v_cmp_gt_f16_e64 s0, v2.h, v3.h`), without the op_sel after them."""
    out = []
    m = re.search(r" op_sel:\[[^\]]*\]", asm)
    if m and "{=" in m.group(0) and not any(g.hidden for g in e.groups):
        out.append(("names", ops, enc, asm.replace(m.group(0), "")))
    for o in e.form.operands:
        if o.type == "OPR_LABEL" and o.field:
            f = o.field.lower()
            field = re.search(rf"\b{f}_(\d+):16={f}\b", enc)
            if field and "{" + f + "}" in asm:
                out.append((
                    "label",
                    [x if not x.endswith(f" %{f}") else 'ptr "label" %dest' for x in ops],
                    enc.replace(field.group(0), f"{f}_{field.group(1)}:s16=(dest-end)/4"),
                    asm.replace("{" + f + "}", "{dest}"),
                ))
    if e.form.enc.name == "ENC_SMEM" and "{soffset}{" in asm and "{offset" not in asm:
        g = next(g for g in e.groups if g.name == "soffset")
        if g.names.get(124) == "null" and "soffset_57:7=soffset" in enc:
            out.append((
                "zero",
                [x for x in ops if not x.endswith(" %soffset")],
                fixed_in_guards(enc.replace("soffset_57:7=soffset", "1111100"), "soffset", 124),
                asm.replace("{soffset}", "0x0"),
            ))
    return out


def write_enums(enums, path, header):
    lines = [header]
    ids = []
    for k, (n, names) in enumerate(enums.order):
        ids.append(f"!{2000 + k}")
    lines.append("!target.enums = !{" + ", ".join(ids) + "}")
    for k, (n, names) in enumerate(enums.order):
        parts = [f'!"{n}"']
        for v in sorted(names):
            parts.append(f'!"{v}"')
            parts.append(f'!"{pattern_escape(names[v])}"')
        lines.append(f"!{2000 + k} = !{{{', '.join(parts)}}}")
    open(path, "w").write("\n".join(lines) + "\n")


BASE = """; The gfx1151 (RDNA 3.5) description: the files, and the machine as a whole.
; Written by gen_gfx1151.py.
;
;   00-base.ll    what the machine lacks, its parameters
;   05-enums.ll   operands written by name: registers, constants, modifiers
;   10-scalar.ll  scalar ALU, scalar memory and program control
;   20-vector.ll  vector ALU
;   30-memory.ll  vector memory: LDS, buffers, flat, global, scratch, images
;
; For assembly and disassembly only: instruction selection has nothing to
; select yet. Instructions are 32 bits or more, in 32-bit units; wave32.
!target.lacks = !{!1000}
!1000 = !{!"stack", !"calls", !"globals", !"indirect", !"trap"}
; The scalar registers. Instructions name registers, and ranges of them
; (s[4:7], v[0:3]), as operands written by name (enums), so the class is
; only a name for the file.
!target.registers = !{!1050}
!1050 = !{!"s", !"s{0-105}"}
!target.classes = !{!1060}
!1060 = !{!"s", !"file", !"s"}
!target.params = !{!1100, !1101, !1102, !1103, !1104}
!1100 = !{!"instruction-unit", !"4"}
!1101 = !{!"vector-bits", !"0"}
!1102 = !{!"asm-comment", !";"}
; EM_AMDGPU, and EF_AMDGPU_MACH_AMDGCN_GFX1151.
!1103 = !{!"elf-machine", !"224"}
!1104 = !{!"elf-flags", !"74"}
!target.cpus = !{!1150}
!1150 = !{!"gfx1151"}
"""


class Pair:
    """VOPD: two forms, an X and a Y, as one form."""

    def __init__(self, x, y, enc):
        self.name = f"{x.name}__{y.name}"
        self.enc = enc
        self.cond = None
        self.x, self.y = x, y
        self.operands = x.operands + y.operands


def vopd(forms, encodings):
    """Every X and Y pair of the dual-issue instructions, each destination
    parity, and each half with or without the literal."""
    xs = [f for f in forms if f.enc.name.startswith("VOPDXY") and any(o.field == "VDSTX" for o in f.operands)]
    ys = [f for f in forms if f.enc.name.startswith("VOPDXY") and any(o.field == "VDSTY" for o in f.operands)]
    plain, lit = encodings["VOPDXY"], encodings["VOPDXY_INST_LITERAL"]
    out = []
    for x in xs:
        for y in ys:
            uses = x.enc is lit or y.enc is lit
            enc = lit if uses else plain
            p = Pair(x, y, enc)
            f = enc.fields
            ids = [i for i in enc.ids if get(i, f["OPX"]) == x.opcode]
            if not ids:
                continue
            fixed = put(put(ids[0] & enc.mask, f["OPX"], x.opcode), f["OPY"], y.opcode)
            if x.cond == "has_lit":
                fixed = put(fixed, f["SRCX0"], 255)
            if y.cond == "has_lit":
                fixed = put(fixed, f["SRCY0"], 255)
            named = {o.field for o in p.operands if not o.implicit}
            for parity in (0, 1):
                groups, guards = [], []
                for fld in ("VDSTX", "SRCX0", "VSRCX1", "SRCY0", "VSRCY1"):
                    if fld in named and not (fld == "SRCX0" and x.cond == "has_lit") and not (fld == "SRCY0" and y.cond == "has_lit"):
                        width = sum(n for _, n in f[fld])
                        vals = [v for v in range(1 << width) if (fld != "VDSTX" or v % 2 == parity) and not (fld.startswith("SRC") and v == 255)]
                        groups.append(Group(fld.lower(), [(fld, 0)], vals))
                # The second destination's low bit is the other of the first's.
                groups.append(Group("vdsty", [("VDSTY", 1)], [2 * k + 1 - parity for k in range(128)], sub=1 - parity))
                if "LITERAL" in named:
                    groups.append(Group("lit", [("LITERAL", 0)], literal_values(random.Random(p.name)), kind="imm", fmt="h32"))
                gn = {g.name for g in groups}
                # Sources in VGPRs must be in different banks.
                if {"srcx0", "srcy0"} <= gn:
                    guards.append(("srcx0 < 256 or srcy0 < 256 or ((srcx0 ^ srcy0) & 3) != 0",
                                   "(srcx0<256)|(srcy0<256)|(((srcx0^srcy0)&3)!=0)"))
                if {"vsrcx1", "vsrcy1"} <= gn:
                    guards.append(("((vsrcx1 ^ vsrcy1) & 3) != 0", "((vsrcx1^vsrcy1)&3)!=0"))
                # And their third sources, of which there are two banks: an
                # accumulator's destination, or fmamk's second source.
                def third(form, half):
                    if "FMAC" in form.name or "DOT2ACC" in form.name:
                        return f"vdst{half}"
                    if "FMAMK" in form.name:
                        return f"vsrc{half}1"
                    return None
                tx, ty = third(x, "x"), third(y, "y")
                if tx in gn and ty in gn:
                    guards.append((f"(({tx} ^ {ty}) & 1) != 0", f"(({tx}^{ty})&1)!=0"))
                # The constant bus: two scalar values at most, a scalar
                # register read by both halves once, the literal once, and
                # vcc for a cndmask.
                srcs = [n for n in ("srcx0", "srcy0") if n in gn]
                fixed_scalars = ("LITERAL" in named) + any("CNDMASK" in h.name for h in (x, y))
                if len(srcs) + fixed_scalars > 2:
                    terms = [f"({n}<128)" for n in srcs]
                    if len(srcs) == 2:
                        terms.append("-((srcx0==srcy0)*(srcx0<128))")
                    expr = "+".join(terms).replace("+-", "-") + f"+{fixed_scalars}<=2"
                    guards.append((expr.replace("==", " == "), expr))
                # (The first destination's parity, which the second's field
                # depends on, as a guard: the disassembler tells the two
                # entries apart by it.)
                guards.append((f"vdstx % 2 == {parity}", f"(vdstx&1)=={parity}"))
                e = Entry(p, f"{enc.name}.{'odd' if parity else 'even'}", fixed, groups, guards)
                out.append(e)
    return out


def generate(forms):
    """Every form's entries, learnt and checked; and why the others are not."""
    made, why = [], collections.Counter()
    pairs = [f for f in forms if f.enc.name.startswith("VOPD")]
    forms = [f for f in forms if not f.enc.name.startswith("VOPD")]
    if pairs:
        for e in vopd(pairs, ENCODINGS):
            base = baseline(e)
            if base is not None:
                for g in e.groups:
                    if g.name == "lit":
                        e.guards += literal_guards(e, base, g)
                base = baseline(e)
            if base is None or not learn(e, base) or not check(e, 12, random.Random(e.form.name + e.tag)):
                why[(e.form.enc.name, "failed")] += 1
                if not COLLECT[0]:
                    print(f"{e.form.name} {e.tag}: {e.why}", file=sys.stderr)
                continue
            made.append(e)
    for form in forms:
        rng = random.Random(f"{form.name}.{form.enc.name}.{form.cond}")
        es, reason = build(form, rng)
        if reason:
            why[(form.enc.name, reason)] += 1
            if not COLLECT[0] and not reason.endswith("not yet"):
                print(f"{form.name} {form.enc.name}: {reason}", file=sys.stderr)
            continue
        for e in es:
            try:
                base = baseline(e)
                good = base is not None and learn(e, base) and check(e, 12, rng)
            except Exception as x:
                e.why, good = f"{type(x).__name__}: {x}", False
            if not good:
                why[(form.enc.name, "failed")] += 1
                if not COLLECT[0]:
                    print(f"{form.name} {e.tag}: {e.why}", file=sys.stderr)
                continue
            made.append(e)
    return made, why


def settle(fn):
    """`fn()` run until the oracle has everything it asks for."""
    for round in range(40):
        PENDING.clear()
        PENDING_ASM.clear()
        REQUESTED.clear()
        COLLECT[0] = True
        out = fn()
        COLLECT[0] = False
        print(f"  round {round}: {len(PENDING)} words, {len(PENDING_ASM)} lines", file=sys.stderr)
        if not PENDING and not PENDING_ASM:
            return out
        prefetch(list(PENDING))
        assemble(list(PENDING_ASM))
    return fn()


ENCODINGS = {}
FILES = [
    ("10-scalar.ll", "Scalar ALU, scalar memory and program control", lambda n: n.startswith(("ENC_SOP", "SOP", "ENC_SMEM"))),
    ("20-vector.ll", "Vector ALU", lambda n: n.startswith(("ENC_VOP", "VOP", "ENC_VINTERP", "VOPD"))),
    ("30-memory.ll", "Vector memory: LDS, buffers, flat, global, scratch, images, exports",
     lambda n: n.startswith(("ENC_DS", "ENC_MUBUF", "ENC_MTBUF", "ENC_FLAT", "ENC_MIMG", "MIMG", "ENC_EXP", "ENC_LDSDIR"))),
]


def main():
    encs, forms = load(sys.argv[1])
    ENCODINGS.update(encs)
    if ONLY:
        forms = [f for f in forms if re.fullmatch(ONLY, f.enc.name)]
    made, why = settle(lambda: generate(forms))
    made, why = generate(forms)
    merge_enums(made)
    enums = Enums()
    for e in made:
        for g in e.groups:
            if g.kind == "enum" and not getattr(g, "fixed_only", False):
                enums.get(g.names)
    open(os.path.join(HERE, "00-base.ll"), "w").write(BASE)
    for name, what, test in FILES:
        write([e for e in made if test(e.form.enc.name)], enums, os.path.join(HERE, name), f"; {what}. Written by gen_gfx1151.py.")
    write_enums(enums, os.path.join(HERE, "05-enums.ll"), "; Operands written by name. Written by gen_gfx1151.py.")
    print(f"{len(made)} entries, {len(enums.order)} enums")
    for (enc, reason), n in sorted(why.items()):
        print(f"  {enc}: {n} {reason}")


if __name__ == "__main__":
    main()
