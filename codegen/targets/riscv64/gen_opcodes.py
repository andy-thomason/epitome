"""Writes 80-asm.ll: entries for assembly and disassembly only, for the RV64
instructions of the extensions below that the rest of the description does
not have.

Two sources, both public or observed:

- riscv-opcodes (RISC-V International, BSD 3-clause): each instruction's
  fixed bits and the fields of its operands (extensions/*, arg_lut.csv).
  It does not give assembly syntax, nor which registers are integer and
  which floating-point. So scalar operands are written in RISC-V's order
  (rd, rs1, rs2, rs3, then immediates; loads `rd, imm(rs1)`, stores
  `rs2, imm(rs1)`, branches `rs1, rs2, label`, atomics `rd, rs2, (rs1)`),
  and immediates as the ISA manual lays them out (the I, S, B, J and U
  forms, and the compressed forms' scattered bits).
- LLVM's RISC-V assembler, as rustc has it (its riscv64gc-unknown-none-elf
  and riscv64imac-unknown-none-elf targets, which rustup installs), on
  lines we write: for each template, the register files that it encodes as
  this instruction are found by trying them, and for the vector and
  hypervisor instructions the order of the operands too, each operand a
  different register so that only the right order gives the right word.
  Every entry written is one whose words, for every line tried, are the
  assembler's.

The instructions are in three groups, each assembled on its own: the 32-bit
ones; the compressed (16-bit) ones of C and its parts; and Zcmp and Zcmt,
which take the encodings of C's double-precision loads and stores, so are
assembled for a machine without D, and are not used to disassemble.

Entries are "asmonly": the assembler reads them and the disassembler writes
them, but instruction selection does not use them. Run:

    python3 codegen/targets/riscv64/gen_opcodes.py path/to/riscv-opcodes

The output is checked in; the code generator reads only the .ll files.
"""
import itertools
import os
import re
import struct
import subprocess
import sys
import tempfile

HERE = os.path.dirname(os.path.abspath(__file__))

# What every group's assembler must be told the machine has.
SCALAR = ("+m,+a,+f,+d,+zicsr,+zifencei,+zba,+zbb,+zbs,+zbc,+zbkb,+zbkc,+zbkx,+zicond,+zfa,+zfh,+zawrs,"
          "+zacas,+zabha,+zicbom,+zicboz,+zicbop,+zihintntl,+zimop,+zknd,+zkne,+zknh,+zksed,+zksh,+svinval,"
          "+zicntr,+zalasr,+experimental-zicfilp,+experimental-zicfiss,+q,+zfbfmin,+h,+sdext,"
          "+smrnmi,+ssctr")
VECTOR = "+v,+zvbb,+zvbc,+zvkg,+zvkned,+zvknha,+zvknhb,+zvksed,+zvksh,+zvfh,+zvfbfmin,+zvfbfwma"

# The groups: the extensions, as riscv-opcodes names their files (rv_ and
# rv64_ ones), and the assembler that checks them.
GROUPS = [
    dict(name="32-bit", target="riscv64gc-unknown-none-elf", features=SCALAR + "," + VECTOR, rvc=False,
         decode=True,
         exts=["i", "m", "a", "f", "d", "zicsr", "zifencei", "zba", "zbb", "zbs", "zbc", "zbkb", "zbkc", "zbkx",
               "zicond", "zfa", "f_zfa", "d_zfa", "zfh", "zfh_zfa", "zfhmin", "d_zfhmin", "zawrs", "zacas",
               "zabha", "zabha_zacas", "zicbo", "zihintntl", "zimop", "zkn", "zknd", "zkne", "zknh", "zks",
               "zksed", "zksh", "zk", "system", "s", "svinval", "zicntr", "zalasr", "zicfilp", "zicfiss",
               "q", "q_zfa", "q_zfhmin", "zfbfmin", "h", "svinval_h", "sdext", "smrnmi", "ssctr",
               "v", "v_aliases", "zvbb", "zvbc", "zvkg", "zvkned", "zvknha", "zvknhb", "zvksed", "zvksh",
               "zvfbfmin", "zvfbfwma"]),
    dict(name="compressed", target="riscv64gc-unknown-none-elf", features=SCALAR + ",+zcb,+zcmop", rvc=True,
         decode=True, exts=["c", "c_d", "zcb", "zcmop", "c_zicfiss", "c_zihintntl"]),
    dict(name="zcmp", target="riscv64imac-unknown-none-elf", features="+zcb,+zcmp,+zcmt", rvc=True,
         decode=False, exts=["zcmp", "zcmt"]),
]
ROUNDING = [("rne", 0), ("rtz", 1), ("rdn", 2), ("rup", 3), ("rmm", 4)]
NOTICE_FILE = "LICENSE"


# ---------------------------------------------------------------------------
# riscv-opcodes


def load(root, exts):
    args = {}
    for line in open(os.path.join(root, "arg_lut.csv")):
        m = re.match(r'"([^"]+)",\s*(\d+),\s*(\d+)', line)
        if m:
            args[m.group(1)] = (int(m.group(2)), int(m.group(3)))
    files = {}
    for ext in exts:
        for prefix in ("rv_", "rv64_"):
            p = os.path.join(root, "extensions", prefix + ext)
            if os.path.exists(p):
                files[prefix + ext] = open(p).read().splitlines()
    insts = {}

    def parse(fields):
        mask = value = 0
        ops = []
        for f in fields:
            m = re.fullmatch(r"(\d+)\.\.(\d+)=(\w+)", f) or re.fullmatch(r"(\d+)()=(\w+)", f)
            if m:
                hi = int(m.group(1))
                lo = int(m.group(2)) if m.group(2) else hi
                v = int(m.group(3), 0)
                for b in range(lo, hi + 1):
                    mask |= 1 << b
                    value |= ((v >> (b - lo)) & 1) << b
            else:
                ops.append(f)
        return mask, value, ops

    pseudo = []
    for name, lines in files.items():
        for line in lines:
            line = line.split("#")[0].strip()
            if not line:
                continue
            parts = line.split()
            if parts[0] == "$import":
                continue
            if parts[0] == "$pseudo_op":
                pseudo.append((name, parts[2], parts[3:], parts[1].split("::")[1]))
                continue
            mask, value, ops = parse(parts[1:])
            insts[parts[0]] = dict(name=parts[0], mask=mask, value=value, ops=ops, pseudo=False, ext=name)
    for ext, mnem, parts, of in pseudo:
        if mnem in insts:
            continue
        mask, value, ops = parse(parts)
        insts[mnem] = dict(name=mnem, mask=mask, value=value, ops=ops, pseudo=True, ext=ext, of=of)
    # cm.jt is cm.jalt with an index below 32 (riscv-opcodes has only the
    # one, for both).
    if "cm.jalt" in insts:
        insts["cm.jt"] = dict(insts["cm.jalt"], name="cm.jt", pseudo=True, of="cm.jalt")
    for ins in insts.values():
        ins["width"] = 32 if ins["value"] & 3 == 3 else 16
    return insts, args


# ---------------------------------------------------------------------------
# Operands: how each is written and held.

REGS = ("rd", "rs1", "rs2", "rs3")
VREGS = ("vd", "vs3", "vs1", "vs2")
# Immediates of one field, and whether they are signed.
SIMPLE = {"imm12": True, "imm20": False, "shamtd": False, "shamtw": False, "csr": False, "zimm": False,
          "zimm5": False, "rnum": False, "bs": False, "imm5": False, "simm5": True}
# Fields set by the variant (a suffix, a keyword or the mnemonic), not by an
# operand.
VARIANT = {"aq", "rl", "aqrl", "rm", "pred", "succ", "fm", "vm", "nf", "zimm10", "zimm11", "c_rlist", "c_spimm"}
XNAMES = ["zero", "ra", "sp", "gp", "tp", "t0", "t1", "t2", "s0", "s1"] + [f"a{k}" for k in range(8)] + \
    [f"s{k}" for k in range(2, 12)] + [f"t{k}" for k in range(3, 7)]
FNAMES = [f"ft{k}" for k in range(8)] + ["fs0", "fs1"] + [f"fa{k}" for k in range(8)] + \
    [f"fs{k}" for k in range(2, 12)] + [f"ft{k}" for k in range(8, 12)]

# The compressed forms' register fields: the operand, its bits, and how it
# is held: all 32 registers ("full"), x8-x15 or f8-f15 ("prime"), or Zcmp's
# s0-s7 ("sreg"); and the registers it cannot be.
CREGS = {
    "rd_p": ("rd", 4, 2, "prime", []), "rs1_p": ("rs1", 9, 7, "prime", []), "rs2_p": ("rs2", 4, 2, "prime", []),
    "rd_rs1_p": ("rd", 9, 7, "prime", []), "rd_rs1_n0": ("rd", 11, 7, "full", [0]),
    "rd_rs1": ("rd", 11, 7, "full", []), "rd_n2": ("rd", 11, 7, "full", [0, 2]), "rd_n0": ("rd", 11, 7, "full", [0]),
    "rs1_n0": ("rs1", 11, 7, "full", [0]), "c_rs2_n0": ("rs2", 6, 2, "full", [0]),
    "c_rs1_n0": ("rs1", 11, 7, "full", [0]), "c_rs2": ("rs2", 6, 2, "full", []), "rd": ("rd", 11, 7, "full", []),
    "c_sreg1": ("r1", 9, 7, "sreg", []), "c_sreg2": ("r2", 4, 2, "sreg", []),
}
# The compressed forms' immediates, from the ISA manual: for each field, its
# instruction bits (top, bottom) and the immediate's bit in each, top first.
# What the immediate is (an immediate, or a label), whether it is signed,
# and whether it may be zero.
CIMMS = {
    "c_nzuimm10": [(12, 5, [5, 4, 9, 8, 7, 6, 2, 3])],
    "c_uimm7hi": [(12, 10, [5, 4, 3])], "c_uimm7lo": [(6, 5, [2, 6])],
    "c_uimm8hi": [(12, 10, [5, 4, 3])], "c_uimm8lo": [(6, 5, [7, 6])],
    "c_nzimm6hi": [(12, 12, [5])], "c_nzimm6lo": [(6, 2, [4, 3, 2, 1, 0])],
    "c_imm6hi": [(12, 12, [5])], "c_imm6lo": [(6, 2, [4, 3, 2, 1, 0])],
    "c_nzimm10hi": [(12, 12, [9])], "c_nzimm10lo": [(6, 2, [4, 6, 8, 7, 5])],
    "c_nzimm18hi": [(12, 12, [17])], "c_nzimm18lo": [(6, 2, [16, 15, 14, 13, 12])],
    "c_imm12": [(12, 2, [11, 4, 9, 8, 10, 6, 7, 3, 2, 1, 5])],
    "c_bimm9hi": [(12, 10, [8, 4, 3])], "c_bimm9lo": [(6, 2, [7, 6, 2, 1, 5])],
    "c_nzuimm6hi": [(12, 12, [5])], "c_nzuimm6lo": [(6, 2, [4, 3, 2, 1, 0])],
    "c_uimm8sphi": [(12, 12, [5])], "c_uimm8splo": [(6, 2, [4, 3, 2, 7, 6])],
    "c_uimm9sphi": [(12, 12, [5])], "c_uimm9splo": [(6, 2, [4, 3, 8, 7, 6])],
    "c_uimm8sp_s": [(12, 7, [5, 4, 3, 2, 7, 6])], "c_uimm9sp_s": [(12, 7, [5, 4, 3, 8, 7, 6])],
    "c_uimm2": [(6, 5, [0, 1])], "c_uimm1": [(5, 5, [1])],
    "c_index": [(9, 2, [7, 6, 5, 4, 3, 2, 1, 0])],
}
# Signed immediates, and those that may not be zero (by their fields' names).
CSIGNED = ("c_nzimm6", "c_imm6", "c_nzimm10", "c_nzimm18", "c_imm12", "c_bimm9")
CNONZERO = ("c_nzuimm10", "c_nzimm6", "c_nzimm10", "c_nzimm18", "c_nzuimm6")
# Zcmp's s-registers: s0-s1 are x8-x9, s2-s7 x18-x23.
SREGS = [8, 9, 18, 19, 20, 21, 22, 23]


class Operand:
    """An operand of a template: a register of file "x", "f" or "v", an
    immediate, or a label; the tokens of the encoding it fills, by top bit;
    the guards it needs; and, for Zcmp's s-registers, the alternatives
    (guard, tokens) it is held by."""

    def __init__(self, name, kind, tokens, guards=(), alts=None):
        self.name, self.kind, self.tokens = name, kind, tokens
        self.guards, self.alts = list(guards), alts
        self.first = SREGS if alts else None

    def text(self, v):
        if self.kind == "x":
            return XNAMES[v]
        if self.kind == "f":
            return FNAMES[v]
        if self.kind == "v":
            return f"v{v}"
        if self.kind == "label":
            return f".{'+' if v >= 0 else ''}{v}"
        if self.kind == "lui":
            return str((v >> 12) & 0xfffff)
        return str(v)


def slices(name, pieces, signed, prefix):
    """The tokens of an immediate scattered over instruction bits: each run
    of instruction bits holding a run of the immediate's, by top bit."""
    out = {}
    top = max(b for _, _, bits in pieces for b in bits)
    k = 0
    for hi, lo, bits in pieces:
        b = hi
        j = 0
        while j < len(bits):
            # A run: the immediate's bits falling by one.
            n = 1
            while j + n < len(bits) and bits[j + n] == bits[j + n - 1] - 1:
                n += 1
            low = bits[j + n - 1]
            if signed and bits[j] == top:
                expr = f"{name}>>{low}"
                tok = f"{prefix}{k}:s{n}={expr}"
            else:
                expr = f"({name}>>{low})&{(1 << n) - 1}" if low else f"{name}&{(1 << n) - 1}"
                tok = f"{prefix}{k}:{n}={expr}"
            out[b] = (tok, n)
            k += 1
            b -= n
            j += n
    return out


def compressed_operands(ins, args, files):
    """The operands of a compressed instruction, with register files
    `files` in order; None if it has an argument we cannot write."""
    out = []
    ops = ins["ops"]
    k = 0
    for f in ops:
        if f not in CREGS:
            continue
        name, hi, lo, how, never = CREGS[f]
        kind = files[k]
        k += 1
        width = hi - lo + 1
        if how == "prime":
            out.append(Operand(name, kind, {hi: (f"{name}_p:3={name}-8", 3)}))
        elif how == "sreg":
            out.append(Operand(name, kind, {}, alts=[
                (f"if({name}<10)", {hi: (f"{name}_s:3={name}-8", 3)}),
                (f"if({name}>=18)", {hi: (f"{name}_s:3={name}-16", 3)})]))
        else:
            out.append(Operand(name, kind, {hi: (f"{name}:{width}", width)}, [f"if({name}!={n})" for n in never]))
    imm = [f for f in ops if f in CIMMS]
    if imm:
        base = re.sub(r"(hi|lo|sphi|splo|sp_s)$", "", imm[0])
        if base.endswith("sp"):
            base = base[:-2]
        pieces = [p for f in imm for p in CIMMS[f]]
        signed = any(base.startswith(s) for s in CSIGNED)
        label = base in ("c_imm12", "c_bimm9")
        name = "dest" if label else "imm"
        kind = "label" if label else ("lui" if base == "c_nzimm18" else "imm")
        guards = [f"if({name}!=0)"] if any(base.startswith(s) for s in CNONZERO) else []
        # A table jump's index: cm.jt's below 32, cm.jalt's from 32.
        guards += {"cm.jt": [f"if({name}<32)"], "cm.jalt": [f"if({name}>=32)"]}.get(ins["name"], [])
        out.append(Operand(name, kind, slices(name, pieces, signed, "i"), guards))
    known = set(CREGS) | set(CIMMS) | VARIANT
    if set(ops) - known:
        return None
    return out


def operands(ins, args, files):
    """The operands of an instruction, with register files `files` for its
    scalar registers in order; None if it has an argument we cannot
    write."""
    if ins["width"] == 16:
        return compressed_operands(ins, args, files)
    out = []
    ops = [f for f in ins["ops"] if not (f == "rs1" and fli(ins))]
    k = 0
    for r in REGS:
        if r in ops:
            hi, lo = args[r]
            out.append(Operand(r, files[k], {hi: (f"{r}:5", hi - lo + 1)}))
            k += 1
    for r in VREGS:
        if r in ops:
            hi, lo = args[r]
            out.append(Operand(r, "v", {hi: (f"{r}:5", hi - lo + 1)}))
    s = set(ops)
    if {"bimm12hi", "bimm12lo"} <= s:
        out.append(Operand("dest", "label", {31: ("o_12:s1=dest>>12", 1), 30: ("o_10:6=(dest>>5)&63", 6),
                                             11: ("o_4:4=(dest>>1)&15", 4), 7: ("o_11:1=(dest>>11)&1", 1)}))
    elif "jimm20" in s:
        out.append(Operand("dest", "label", {31: ("o_20:s1=dest>>20", 1), 30: ("o_10:10=(dest>>1)&1023", 10),
                                             20: ("o_11:1=(dest>>11)&1", 1), 19: ("o_19:8=(dest>>12)&255", 8)}))
    elif {"imm12hi", "imm12lo"} <= s:
        out.append(Operand("off", "imm", {31: ("o_hi:s7=off>>5", 7), 11: ("o_lo:5=off&31", 5)}))
    if {"zimm6hi", "zimm6lo"} <= s:
        out.append(Operand("zimm6", "imm", {26: ("i_hi:1=(zimm6>>5)&1", 1), 19: ("i_lo:5=zimm6&31", 5)}))
    for f, signed in SIMPLE.items():
        if f in s:
            hi, lo = args[f]
            out.append(Operand(f, "imm", {hi: (f"{f}:{'s' if signed else ''}{hi - lo + 1}", hi - lo + 1)}))
    known = set(REGS) | set(VREGS) | set(SIMPLE) | VARIANT | {"bimm12hi", "bimm12lo", "jimm20", "imm12hi",
                                                               "imm12lo", "zimm6hi", "zimm6lo"}
    if s - known:
        return None
    return out


def nregs(ins):
    """How many of its registers are scalar ones, of a file to be found."""
    if ins["width"] == 16:
        return sum(f in CREGS for f in ins["ops"])
    return sum(r in ins["ops"] for r in REGS) - fli(ins)


def fli(ins):
    """Whether an instruction is Zfa's fli, whose rs1 field is the index of
    a constant."""
    return ins["name"].startswith("fli.")


# fli's constants, by index (the Zfa specification's table).
FLI = ["-1.0", "min", repr(2.0 ** -16), repr(2.0 ** -15), repr(2.0 ** -8), repr(2.0 ** -7), "0.0625", "0.125",
       "0.25", "0.3125", "0.375", "0.4375", "0.5", "0.625", "0.75", "0.875", "1.0", "1.25", "1.5", "1.75", "2.0",
       "2.5", "3.0", "4.0", "8.0", "16.0", "128.0", "256.0", "32768.0", "65536.0", "inf", "nan"]


SEW = ["e8", "e16", "e32", "e64"]
LMUL = [("m1", 0), ("m2", 1), ("m4", 2), ("m8", 3), ("mf8", 5), ("mf4", 6), ("mf2", 7)]
# Segment loads and stores: the mnemonic for each number of fields.
SEGMENT = re.compile(r"v(l|s)(e|se|uxei|oxei)(\d+)(ff)?\.v")


def rlist(r):
    """Zcmp's register list for an rlist of 4-15, and the stack it takes
    at least on RV64."""
    n = r - 3 if r < 15 else 13
    regs = "ra" if r == 4 else ("ra, s0" if r == 5 else (f"ra, s0-s{r - 5}" if r < 15 else "ra, s0-s11"))
    return "{" + regs + "}", (n * 8 + 15) // 16 * 16


def variants(ins):
    """The mnemonics, suffixes and keyword operands an instruction is
    written with: (mnemonic, tail)."""
    ops = set(ins["ops"])
    name = ins["name"]
    # sspush.x1 is `sspush ra`.
    m = re.fullmatch(r"(.*)\.x(1|5)", name)
    if m:
        return [(m.group(1), " " + XNAMES[int(m.group(2))])]
    if fli(ins):
        return [(name, f", {c}") for c in FLI]
    # (fcvtmod.w.d rounds only toward zero, and says so.)
    if name == "fcvtmod.w.d":
        return [(name, ", rtz")]
    names = [name]
    if re.search(r"\.(aq|rl)$", name):
        # Zalasr's lb.aq and sb.rl, and their .aqrl.
        names = [name, name[:-3] + ".aqrl"]
    elif "aq" in ops or "aqrl" in ops:
        names = [name + s for s in ("", ".aq", ".rl", ".aqrl")]
    if "nf" in ops:
        seg = SEGMENT.fullmatch(name)
        if seg:
            l, how, w, ff = seg.groups()
            mid = {"e": "seg{}e", "se": "sseg{}e", "uxei": "uxseg{}ei", "oxei": "oxseg{}ei"}[how]
            names = [name] + [f"v{l}{mid.format(k)}{w}{ff or ''}.v" for k in range(2, 9)]
    tail = [""]
    if "rm" in ops:
        tail = [""] + [f", {r}" for r, _ in ROUNDING] + [", dyn"]
    if "pred" in ops:
        sets = ["iorw", "rw", "r", "w"]
        return [(n, f" {p}, {q}") for n in names for p in sets for q in sets]
    if "zimm10" in ops or "zimm11" in ops:
        tail = [f", {s}, {l}, {t}, {a}" for s in SEW for l, _ in LMUL for t in ("tu", "ta") for a in ("mu", "ma")]
    if "vm" in ops:
        tail = [t + m for t in tail for m in ("", ", v0.t")]
    elif any(r in ops for r in VREGS) and re.search(r"\.v[vxif]m$", name):
        tail = [t + ", v0" for t in tail]
    if "c_rlist" in ops:
        out = []
        for r in range(4, 16):
            regs, base = rlist(r)
            sign = "-" if name == "cm.push" else ""
            out += [(name, f" {regs}, {sign}{base + 16 * k}") for k in range(4)]
        return out
    return [(n, t) for n in names for t in tail]


def template(ins, ops, mnem, tail):
    """The assembly of a scalar instruction, by RISC-V's operand order;
    `{name}` for operands."""
    by = {o.name: o for o in ops}
    regs = [o for o in ops if o.kind in "xfv"]
    imms = [o for o in ops if o.kind in ("imm", "label", "lui")]
    names = [o.name for o in regs]
    opcode = (ins["value"] >> 2) & 31
    name = ins["name"]
    if ins["width"] == 16:
        mem = "{imm}(sp)" if "sp" in name else "{imm}({rs1})"
        if re.fullmatch(r"c\.f?l[bhwdq]u?(sp)?", name):
            body = "{rd}, " + mem
        elif re.fullmatch(r"c\.f?s[bhwdq](sp)?", name):
            body = "{rs2}, " + mem
        elif name == "c.addi4spn":
            body = "{rd}, sp, {imm}"
        elif name == "c.addi16sp":
            body = "sp, {imm}"
        elif name in ("cm.mvsa01", "cm.mva01s"):
            body = "{r1}, {r2}"
        else:
            body = ", ".join(["{" + o.name + "}" for o in regs + imms])
    elif "off" in by and "rs1" in by and "rs2" in by:
        body = "{rs2}, {off}({rs1})"
    elif "dest" in by and "rs1" in by and "rs2" in by:
        body = "{rs1}, {rs2}, {dest}"
    elif "imm12" in by and "rs1" in by and "rd" in by and (opcode in (0, 1) or name == "jalr"):
        body = "{rd}, {imm12}({rs1})"
    elif (opcode == 0x0B or name.startswith("cbo.")) and "rs1" in by:
        body = ", ".join(["{" + n + "}" for n in names if n not in ("rs1",)] + ["({rs1})"])
    elif "csr" in by:
        body = ", ".join(["{rd}", "{csr}"] + ["{" + o.name + "}" for o in ops if o.name not in ("rd", "csr")])
    else:
        body = ", ".join(["{" + o.name + "}" for o in regs + imms])
    text = (mnem + " " + body).strip() + tail
    return text if body else mnem + tail


def searched(ins):
    """Whether an instruction's operand order is found by trying each one:
    the vector instructions, and the hypervisor's loads and stores."""
    return any(r in ins["ops"] for r in VREGS) or ins["ext"].endswith("_h") or ins["ext"] in ("rv_h", "rv64_h")


def orders(ins, ops, mnem, tail):
    """The templates of an instruction whose operand order is searched:
    every order of its operands, with an address register written `(rs1)`
    first, then plain."""
    out = []
    for perm in itertools.permutations(ops):
        for paren in ((True, False) if "rs1" in [o.name for o in ops] else (False,)):
            parts = ["({rs1})" if o.name == "rs1" and paren else "{" + o.name + "}" for o in perm]
            body = ", ".join(parts)
            out.append((mnem + " " + body).strip() + tail if body else mnem + tail)
    return out


def fill(text, ops, values):
    out = text
    for o, v in zip(ops, values):
        out = out.replace("{" + o.name + "}", o.text(v))
    return out


def encode(enc, ops, values):
    """Our encoding of an entry, for values of its operands; None where no
    alternative's guards hold."""
    env = {o.name: v for o, v in zip(ops, values)}
    for guards, toks, width in enc:
        if all(eval(g[3:-1], {}, dict(env)) for g in guards):
            return encode_toks(toks, env, width)
    return None


def encode_toks(toks, env, width):
    word = 0
    pos = width
    for t in toks:
        if re.fullmatch(r"[01]+", t):
            for ch in t:
                pos -= 1
                word |= (ch == "1") << pos
            continue
        m = re.fullmatch(r"([A-Za-z0-9_]+):(s?)(\d+)(?:=(.*))?", t)
        w = int(m.group(3))
        pos -= w
        v = eval(m.group(4) or m.group(1), {}, dict(env))
        word |= (v & ((1 << w) - 1)) << pos
    return word


def tokens(ins, owned, base):
    """The encoding: fixed bits, operands' fields, and the variant's bits
    from the base word."""
    toks = []
    b = ins["width"] - 1
    while b >= 0:
        if b in owned:
            tok, width = owned[b]
            toks.append(tok)
            b -= width
            continue
        bit = str(base >> b & 1)
        if toks and re.fullmatch(r"[01]+", toks[-1]):
            toks[-1] += bit
        else:
            toks.append(bit)
        b -= 1
    return toks


def encoding(ins, ops, base):
    """The alternatives of an entry's encoding: (guards, tokens, width),
    one for each choice of Zcmp's s-registers' alternatives."""
    plain = {}
    guards = []
    choices = []
    for o in ops:
        plain.update(o.tokens)
        guards += o.guards
        if o.alts:
            choices.append(o.alts)
    out = []
    for pick in itertools.product(*choices):
        owned = dict(plain)
        gs = list(guards)
        for g, toks in pick:
            gs.append(g)
            owned.update(toks)
        out.append((gs, tokens(ins, owned, base), ins["width"]))
    return out


def relax(enc, ops, good):
    """The encoding without the guards the assembler does not keep: those
    some line it takes breaks (a hint, as `c.addi zero, 3` is), when every
    line it takes is then ours."""
    env = lambda vals: {o.name: v for o, v in zip(ops, vals)}
    # (Only those of every alternative: those choosing one stay.)
    common = set.intersection(*[set(gs) for gs, _, _ in enc])
    broken = {g for g in common if any(not eval(g[3:-1], {}, env(vals)) for vals, _ in good)}
    loose = [([x for x in gs if x not in broken], toks, w) for gs, toks, w in enc]
    return loose if broken and all(encode(loose, ops, vals) == w for vals, w in good) else enc


def constrain(ins, enc, ops, got):
    """The encoding with the guards on registers the assembler keeps,
    found from the lines it refuses: a pair's evenness (`amocas.q`), a
    vector register group's alignment
    (`vl4r.v` takes v0, v4, ...), the destination not being v0 (with a
    mask, or `vadc`) or a source (`vrgather`, widening), and a segment's
    registers not holding its index (`vloxseg8ei64.v`). Each refused line
    is one register changed from a line it takes; the guards are those that
    the most such lines break and no line it takes does."""
    vregs = [o.name for o in ops if o.kind == "v"]
    pairs = [o.name for o in ops if o.kind in "xf"]
    if ins["width"] == 16 or not vregs + pairs:
        return enc
    env = lambda vals: {o.name: v for o, v in zip(ops, vals)}
    cands = [f"if({a}%2==0)" for a in pairs] + [f"if({a}%{k}==0)" for a in vregs for k in (2, 4, 8)] + [f"if({a}!=0)" for a in vregs] + \
        [f"if({a}!={b})" for a, b in itertools.permutations(vregs, 2) if a < b] + \
        [f"if(({b}<{a})|({b}>={a}+{n}))" for a, b in itertools.permutations(vregs, 2) for n in range(2, 9)]
    taken = [env(vals) for vals, w in got if w is not None]
    refused = [env(vals) for vals, w in got if w is None]
    cands = [g for g in cands if all(eval(g[3:-1], {}, e) for e in taken)]
    chosen = []
    while True:
        left = [e for e in refused if all(eval(g[3:-1], {}, e) for g in chosen)]
        # (Of those that break as many, the strongest: the longest range.)
        best = max(cands, key=lambda g: (sum(not eval(g[3:-1], {}, e) for e in left),
                                         int(re.search(r"\+(\d)\)\)$", g).group(1)) if "+" in g else 1), default=None)
        if best is None or not any(not eval(best[3:-1], {}, e) for e in left):
            break
        chosen.append(best)
    # (An alignment of 2 is implied by one of 4.)
    align = lambda g: re.fullmatch(r"if\((\w+)%(\d+)==0\)", g)
    chosen = [g for g in chosen if not (align(g) and any(
        align(h) and align(h).group(1) == align(g).group(1) and int(align(h).group(2)) > int(align(g).group(2))
        for h in chosen))]
    return [(gs + chosen, toks, w) for gs, toks, w in enc]


def enc_text(enc):
    """The "enc" attribute of an encoding's alternatives."""
    if len(enc) == 1 and not enc[0][0]:
        return " ".join(enc[0][1])
    return "{ " + " ; ".join(" ".join(g + t) for g, t, _ in [(a[0], a[1], a[2]) for a in enc]) + " }"


# Values tried for each operand kind: registers, immediates, labels.
SAMPLES = {"x": [1, 5, 10, 31, 0, 8, 9, 15, 18, 23], "f": [1, 5, 10, 31, 0, 8, 15],
           "v": [1, 5, 10, 31, 0, 8, 16, 24, 2, 4, 6, 9, 12],
           "imm": [1, 0, 3, 31, 63, 100, 2047, -1, -2048, 4095, 0xfffff, 1000, 2, 4, 8, 16, 32, 64, 128, 256, 512,
                   -16, -32, -64, -512, 496, 1008, 248, 504, -15, 15, 7],
           "lui": [1 << 12, 31 << 12, -1 << 12, -32 << 12, 5 << 12, 16 << 12, -16 << 12],
           "label": [8, 0, -8, 2048, -4096, 4094, 1048574, -1048576, 2, 6, 18, 66, 130, 254, -256, 2046, -2048]}
# A first value for each, for finding the operands' files and order: each
# register a different one (x8-x15 and f8-f15 for the compressed forms; and
# v8, v16 and v24, which any group of vector registers may start at).
FIRST = {"x": [10, 11, 12, 13], "f": [8, 9, 10, 11], "v": [8, 16, 24, 4], "imm": [3, 5], "lui": [5 << 12],
         "label": [8]}


# Immediates of the compressed forms are scaled (by 4, 8 or 16), or as
# small as 0-3: these are tried in turn.
FIRST_IMMS = [3, 16, 1, 2, 32]


def first(ops, k=0):
    """The first values of operands: different registers, and the k-th
    choice of immediate."""
    seen = {}
    out = []
    for o in ops:
        seen[o.kind] = seen.get(o.kind, 0) + 1
        pick = o.first or FIRST[o.kind]
        out.append(FIRST_IMMS[k] if o.kind == "imm" and k else pick[(seen[o.kind] - 1) % len(pick)])
    return out


# ---------------------------------------------------------------------------
# The assembler: rustc's.

CACHE = {}


def assemble(lines, group):
    """The assembler's word for each line, or None where it refuses one."""
    key = group["name"]
    todo = sorted({l for l in lines if (key, l) not in CACHE})
    chunks = [todo[part:part + 20000] for part in range(0, len(todo), 20000)]
    while chunks:
        chunk = chunks.pop()
        words, bad = run(chunk, group)
        if words is not None:
            CACHE.update(((key, l), w) for l, w in zip(chunk, words))
            continue
        if bad is None:
            # The assembler failed without saying where: halves, until the
            # line it fails on is found.
            if len(chunk) == 1:
                CACHE[(key, chunk[0])] = None
            else:
                chunks += [chunk[:len(chunk) // 2], chunk[len(chunk) // 2:]]
            continue
        for k in bad:
            CACHE[(key, chunk[k])] = None
        chunk = [l for k, l in enumerate(chunk) if k not in bad]
        if chunk:
            chunks.append(chunk)
    return [CACHE.get((key, l)) for l in lines]


def run(lines, group):
    """Assembles lines, each under a label of its own: each one's word, by
    its label (None for a line that is not one instruction of the group's
    length); or the lines the assembler refused."""
    size = 2 if group["rvc"] else 4
    with tempfile.TemporaryDirectory() as tmp:
        with open(os.path.join(tmp, "t.s"), "w") as f:
            f.write(f".option {'rvc' if group['rvc'] else 'norvc'}\n.option norelax\n"
                    + "".join(f"l{k}: {l}\n" for k, l in enumerate(lines)) + f"l{len(lines)}:\n")
        with open(os.path.join(tmp, "t.rs"), "w") as f:
            f.write('#![no_std]\ncore::arch::global_asm!(include_str!("t.s"), options(raw));\n')
        obj = os.path.join(tmp, "t.o")
        r = subprocess.run(["rustc", "--target", group["target"], "--crate-type", "lib", "--emit", "obj",
                            "-C", f"target-feature={group['features']}", "-o", obj, os.path.join(tmp, "t.rs")],
                           capture_output=True, text=True)
        if r.returncode != 0:
            bad = {int(m.group(1)) - 3 for m in re.finditer(r"<inline asm>:(\d+):", r.stderr)}
            return None, bad or None
        b = open(obj, "rb").read()
        text, at = elf_text(b), elf_labels(b)
        out = []
        for k in range(len(lines)):
            a, z = at.get(f"l{k}"), at.get(f"l{k + 1}")
            ok = a is not None and z == a + size
            out.append(struct.unpack_from("<I" if size == 4 else "<H", text, a)[0] if ok else None)
        return out, set()


def elf_labels(b):
    """The values of the object's symbols, by name."""
    shoff, = struct.unpack_from("<Q", b, 0x28)
    shentsize, shnum, _ = struct.unpack_from("<HHH", b, 0x3A)
    secs = [struct.unpack_from("<IIQQQQIIQQ", b, shoff + k * shentsize) for k in range(shnum)]
    out = {}
    for s in secs:
        if s[1] != 2:  # SHT_SYMTAB
            continue
        strtab = secs[s[6]]
        for k in range(s[5] // 24):
            name, _, _, _, value, _ = struct.unpack_from("<IBBHQQ", b, s[4] + 24 * k)
            n = b[strtab[4] + name:].split(b"\0")[0].decode()
            out[n] = value
    return out


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
# Main


def existing():
    """The description's own entries (not this file's): literal bits, the
    mnemonic, and the fields (name, bottom bit, width)."""
    out = []
    for path in sorted(os.listdir(HERE)):
        if not path.endswith(".ll") or path == "80-asm.ll":
            continue
        for line in open(os.path.join(HERE, path)):
            m = re.match(r'define .*?@([A-Za-z0-9_.]+)\(.*?"enc"="([^"]*)" .*?"asm"="([a-z0-9.]+)', line)
            # (Those of a symbol are not those of a number: lui's of %hi.)
            if not m or '"reloc"=' in line:
                continue
            mask = value = 0
            pos = 32
            fields = []
            for tok in m.group(2).split():
                if re.fullmatch(r"[01]+", tok):
                    for ch in tok:
                        pos -= 1
                        mask |= 1 << pos
                        value |= (ch == "1") << pos
                else:
                    # (A field that is another operand, `rb:5=n`, is a field
                    # of that one.)
                    f = re.fullmatch(r"([A-Za-z0-9_]+):s?(\d+)(?:=(.*))?", tok)
                    pos -= int(f.group(2))
                    if f.group(3) is None or re.fullmatch(r"[a-z_]\w*", f.group(3)):
                        fields.append((f.group(3) or f.group(1), pos, int(f.group(2))))
            out.append((mask, value, m.group(3), fields))
    return out


def main():
    if len(sys.argv) != 2:
        sys.exit(__doc__)
    root = sys.argv[1]
    ours = existing()
    entries = []
    seen_enc = set()
    written = set()
    for group in GROUPS:
        insts, args = load(root, group["exts"])
        stats = {}

        def count(k):
            stats[k] = stats.get(k, 0) + 1

        # 1. For each instruction and variant: the register files (and for
        #    some, the order of the operands) the assembler encodes as it,
        #    from a first line of each choice.
        tries = []
        for ins in insts.values():
            # Written without operands, if its operands are fields that must
            # be zero (fence.i's rd, rs1 and imm12).
            if ins["ops"] and operands(ins, args, "x" * 4) is not None:
                for mnem, tail in variants(ins):
                    text = template(ins, [], mnem, tail)
                    tries.append((ins, [], text, [], text, (mnem, tail)))
            files_of = ["xf"] * nregs(ins)
            for files in itertools.product(*files_of):
                ops = operands(ins, args, files)
                if ops is None:
                    count("an operand we cannot write")
                    break
                imms = len(FIRST_IMMS) if ins["width"] == 16 and any(o.kind == "imm" for o in ops) else 1
                for base in [first(ops, k) for k in range(imms)]:
                    for mnem, tail in variants(ins):
                        texts = orders(ins, ops, mnem, tail) if searched(ins) else [template(ins, ops, mnem, tail)]
                        for text in texts:
                            tries.append((ins, ops, text, base, fill(text, ops, base), (mnem, tail)))
        words = assemble([t[4] for t in tries], group)
        chosen = {}
        for (ins, ops, text, base, line, var), w in zip(tries, words):
            if w is None or w & ins["mask"] != ins["value"]:
                continue
            enc = encoding(ins, ops, w)
            if ops and encode(enc, ops, base) != w:
                continue
            # One entry for each variant, written with operands or without:
            # the first files (and order) that give the word.
            key = (ins["name"], var, tuple(o.kind for o in ops) if searched(ins) else bool(ops))
            if key not in chosen:
                chosen[key] = (ins, ops, text, base, w, enc)
        # 2. Each entry, checked on lines with every operand varied.
        checks = []
        for key, (ins, ops, text, base, w0, enc) in chosen.items():
            lines = [(list(base), fill(text, ops, base))]
            for k, o in enumerate(ops):
                for v in SAMPLES[o.kind]:
                    vals = list(base)
                    vals[k] = v
                    lines.append((vals, fill(text, ops, vals)))
            checks.append((ins, ops, text, enc, lines))
        words = assemble([l for c in checks for _, l in c[4]], group)
        it = iter(words)
        for ins, ops, text, enc, lines in checks:
            got = [(vals, next(it)) for vals, _ in lines]
            good = [(vals, w) for vals, w in got if w is not None]
            enc = constrain(ins, relax(enc, ops, good), ops, got)
            bad = [(vals, w) for vals, w in good if encode(enc, ops, vals) != w]
            # Lines the assembler refuses must be ones we cannot encode.
            loose = [vals for vals, w in got if w is None and encode(enc, ops, vals) is not None and
                     ins["width"] == 16 and any(o.guards or o.alts for o in ops)]
            if bad:
                count("the assembler encodes it otherwise")
                if os.environ.get("SHOW"):
                    vals, w = bad[0]
                    print(f"  {fill(text, ops, vals)}: ours {encode(enc, ops, vals)}, theirs {w:08x}; "
                          f"{enc_text(enc)}", file=sys.stderr)
                continue
            if loose and os.environ.get("SHOW"):
                print(f"  {fill(text, ops, loose[0])}: refused", file=sys.stderr)
            if len(good) < len(ops):
                count("too few lines assemble")
                continue
            # The description's entries that read these words: those whose
            # fixed bits they have, and whose fields of one operand agree
            # (`fmv.d` is `fsgnj.d` only with its sources the same).
            mnem = text.split(" ")[0]
            agree = lambda w, fields: all(len({(w >> b) & ((1 << k) - 1) for f, b, k in fields if f == g}) <= 1
                                          for g in {f for f, _, _ in fields})
            reads = [n for m, v, n, fields in ours if ins["width"] == 32 and
                     all(w & m == v and agree(w, fields) for _, w in good)]
            if mnem in reads:
                count("the description has it")
                continue
            shape = re.sub(r"[a-z_][a-z0-9_]*", "o", enc_text(enc))
            # A pseudo-instruction is read as itself only where its
            # instruction has no entry (mop.r.5, of mop.r.N).
            # (Nor one the description reads by another name: sbreak, of
            # ebreak.)
            real = (not ins["pseudo"] or ins["of"] not in written) and not reads
            decode = group["decode"] and shape not in seen_enc and real
            seen_enc.add(shape)
            if not decode:
                count("another spelling, a pseudo-instruction, or not to disassemble")
            entries.append(entry(ins, ops, text, enc, decode))
            written.add(ins["name"])
            count("entries")
        print(f"{group['name']}:", file=sys.stderr)
        for k, v in sorted(stats.items(), key=lambda x: -x[1]):
            print(f"{v:6} {k}", file=sys.stderr)
    with open(os.path.join(HERE, "80-asm.ll"), "w") as f:
        f.write(NOTICE + "".join(entries) + csr_names(root))


def csr_names(root):
    """The CSRs' names, for the operands of the CSR instructions
    (`!target.asm-names`), from riscv-opcodes' csrs.csv."""
    names = []
    for line in open(os.path.join(root, "csrs.csv")):
        m = re.match(r'\s*(0x[0-9a-fA-F]+),\s*"([a-z0-9_.]+)"', line)
        if m:
            names.append((m.group(2), m.group(1)))
    ids = [f"!{9000 + k}" for k in range(len(names))]
    out = "; The CSRs by name, in the operands of the CSR instructions (csrs.csv).\n"
    out += "!target.asm-names = !{" + ", ".join(ids) + "}\n"
    return out + "".join(f'{i} = !{{!"csr", !"{n}", !"{v}"}}\n' for i, (n, v) in zip(ids, names))


CLASS = {"x": "x", "f": "d", "v": "v"}


NAMES = {}


def entry(ins, ops, text, enc, decode):
    params = []
    for o in ops:
        if o.kind == "label":
            params.append(f'ptr "label" %{o.name}')
        elif o.kind in ("imm", "lui"):
            params.append(f'i64 "imm" %{o.name}')
        else:
            params.append(f'i64 "reg"="{CLASS[o.kind]}" %{o.name}')
    for o in ops:
        if o.kind == "lui":
            text = text.replace("{" + o.name + "}", "{=(" + o.name + ">>12)&1048575}")
    if "c_rlist" in ins["ops"]:
        text = re.sub(r"[{}]", lambda m: "{lb}" if m.group() == "{" else "{rb}", text)
    base = re.sub(r"[^A-Za-z0-9_]", "_", text.split(" ")[0].upper())
    NAMES[base] = NAMES.get(base, -1) + 1
    name = f"{base}.{NAMES[base]}"
    attrs = '"asmonly"' + ("" if decode else ' "nodecode"')
    return (f'define void @{name}({", ".join(params)}) "enc"="{enc_text(enc)}" '
            f'"asm"="{text}" {attrs} {{\n  ret void\n}}\n\n')


NOTICE = """; Entries for assembly and disassembly only ("asmonly"), written by
; gen_opcodes.py: the RV64 instructions of the extensions it names that the
; rest of the description does not have, 32-bit and compressed. Their fixed
; bits and the fields of their operands are from riscv-opcodes, under this
; license:
;
; Copyright (c) 2022 RISC-V International
;
; Redistribution and use in source and binary forms, with or without
; modification, are permitted provided that the following conditions are met:
; 1. Redistributions of source code must retain the above copyright
;    notice, this list of conditions and the following disclaimer.
; 2. Redistributions in binary form must reproduce the above copyright
;    notice, this list of conditions and the following disclaimer in the
;    documentation and/or other materials provided with the distribution.
; 3. Neither the name of the copyright holder nor the names of its contributors
;    may be used to endorse or promote products derived from this software without
;    specific prior written permission.
;
; THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS" AND
; ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE IMPLIED
; WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE
; DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE LIABLE
; FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL
; DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR
; SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER
; CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY,
; OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
; OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
;
; Operands are written in RISC-V's order, or for vector instructions, in the
; order the assembler reads them. Which registers are integer and which
; floating-point, and the bits of rounding modes, orderings, vector types and
; register lists, were found with LLVM's RISC-V assembler (rustc's); each
; entry's words, for every line tried, are its.

"""


if __name__ == "__main__":
    main()
