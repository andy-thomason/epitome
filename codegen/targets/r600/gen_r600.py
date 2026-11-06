"""Writes the R600 description: AMD's Evergreen family of shader cores (the
Radeon HD 5000 and 6000 series), for assembly and disassembly only so far.

A VLIW machine with no stack and no calls: up to five ALU operations issue
together as an instruction group, the last marked by a bit of its own;
programs are clauses of ALU or fetch instructions, started and sequenced by
control-flow (CF) instructions, whose loops are counted by the hardware.
See docs/design/partial-machines.md.

From AMD's "Evergreen Family Instruction Set Architecture: Instructions and
Microcode" (2011), chapter 10, its microcode formats: the fields of each
64-bit word and the opcodes. The assembly syntax is the one LLVM's r600
back end writes (`llc -mtriple=r600 -mcpu=cypress`), which has no
assembler of its own to compare with; tests/r600.rs checks these entries
against the encodings llc prints beside its assembly.

    python3 codegen/targets/r600/gen_r600.py
"""
import os

HERE = os.path.dirname(os.path.abspath(__file__))

# ALU_WORD1_OP2's opcodes, ALU_INST [17:7]: (opcode, name, sources).
# Names are AMD's, without the OP2_INST_ prefix. The 64-bit conversions and
# MUL_64 are listed twice in AMD's table (27-29 and 202, 205, 206); only the
# first are here. LDS_1A, LDS_1A1D and LDS_2A (220, 221, 223) are for the
# hardware's own use and are left out.
UNARY = set("""
FRACT TRUNC CEIL RNDNE FLOOR MOV NOT_INT FLT_TO_INT BFREV_INT FLT64_TO_FLT32
FLT32_TO_FLT64 EXP_IEEE LOG_CLAMPED LOG_IEEE RECIP_CLAMPED RECIP_FF
RECIP_IEEE RECIPSQRT_CLAMPED RECIPSQRT_FF RECIPSQRT_IEEE SQRT_IEEE SIN COS
RECIP_INT RECIP_UINT RECIP_64 RECIP_CLAMPED_64 RECIPSQRT_64
RECIPSQRT_CLAMPED_64 SQRT_64 FLT_TO_UINT INT_TO_FLT UINT_TO_FLT
FLT32_TO_FLT16 FLT16_TO_FLT32 UBYTE0_FLT UBYTE1_FLT UBYTE2_FLT UBYTE3_FLT
BCNT_INT FFBH_UINT FFBL_INT FFBH_INT FLT_TO_UINT4 FLT_TO_INT_RPI
FLT_TO_INT_FLOOR OFFSET_TO_FLT FREXP_64 FRACT_64 MOVA_INT
""".split())
NULLARY = {"NOP", "GROUP_BARRIER", "GROUP_SEQ_BEGIN", "GROUP_SEQ_END"}

OP2 = """
0 ADD 1 MUL 2 MUL_IEEE 3 MAX 4 MIN 5 MAX_DX10 6 MIN_DX10 8 SETE 9 SETGT
10 SETGE 11 SETNE 12 SETE_DX10 13 SETGT_DX10 14 SETGE_DX10 15 SETNE_DX10
16 FRACT 17 TRUNC 18 CEIL 19 RNDNE 20 FLOOR 21 ASHR_INT 22 LSHR_INT
23 LSHL_INT 25 MOV 26 NOP 27 MUL_64 28 FLT64_TO_FLT32 29 FLT32_TO_FLT64
30 PRED_SETGT_UINT 31 PRED_SETGE_UINT 32 PRED_SETE 33 PRED_SETGT
34 PRED_SETGE 35 PRED_SETNE 36 PRED_SET_INV 37 PRED_SET_POP 38 PRED_SET_CLR
39 PRED_SET_RESTORE 40 PRED_SETE_PUSH 41 PRED_SETGT_PUSH 42 PRED_SETGE_PUSH
43 PRED_SETNE_PUSH 44 KILLE 45 KILLGT 46 KILLGE 47 KILLNE 48 AND_INT
49 OR_INT 50 XOR_INT 51 NOT_INT 52 ADD_INT 53 SUB_INT 54 MAX_INT 55 MIN_INT
56 MAX_UINT 57 MIN_UINT 58 SETE_INT 59 SETGT_INT 60 SETGE_INT 61 SETNE_INT
62 SETGT_UINT 63 SETGE_UINT 64 KILLGT_UINT 65 KILLGE_UINT 66 PRED_SETE_INT
67 PRED_SETGT_INT 68 PRED_SETGE_INT 69 PRED_SETNE_INT 70 KILLE_INT
71 KILLGT_INT 72 KILLGE_INT 73 KILLNE_INT 74 PRED_SETE_PUSH_INT
75 PRED_SETGT_PUSH_INT 76 PRED_SETGE_PUSH_INT 77 PRED_SETNE_PUSH_INT
78 PRED_SETLT_PUSH_INT 79 PRED_SETLE_PUSH_INT 80 FLT_TO_INT 81 BFREV_INT
82 ADDC_UINT 83 SUBB_UINT 84 GROUP_BARRIER 85 GROUP_SEQ_BEGIN
86 GROUP_SEQ_END 87 SET_MODE 88 SET_CF_IDX0 89 SET_CF_IDX1 90 SET_LDS_SIZE
129 EXP_IEEE 130 LOG_CLAMPED 131 LOG_IEEE 132 RECIP_CLAMPED 133 RECIP_FF
134 RECIP_IEEE 135 RECIPSQRT_CLAMPED 136 RECIPSQRT_FF 137 RECIPSQRT_IEEE
138 SQRT_IEEE 141 SIN 142 COS 143 MULLO_INT 144 MULHI_INT 145 MULLO_UINT
146 MULHI_UINT 147 RECIP_INT 148 RECIP_UINT 149 RECIP_64 150 RECIP_CLAMPED_64
151 RECIPSQRT_64 152 RECIPSQRT_CLAMPED_64 153 SQRT_64 154 FLT_TO_UINT
155 INT_TO_FLT 156 UINT_TO_FLT 160 BFM_INT 162 FLT32_TO_FLT16
163 FLT16_TO_FLT32 164 UBYTE0_FLT 165 UBYTE1_FLT 166 UBYTE2_FLT
167 UBYTE3_FLT 170 BCNT_INT 171 FFBH_UINT 172 FFBL_INT 173 FFBH_INT
174 FLT_TO_UINT4 175 DOT_IEEE 176 FLT_TO_INT_RPI 177 FLT_TO_INT_FLOOR
178 MULHI_UINT24 179 MBCNT_32HI_INT 180 OFFSET_TO_FLT 181 MUL_UINT24
182 BCNT_ACCUM_PREV_INT 183 MBCNT_32LO_ACCUM_PREV_INT 184 SETE_64
185 SETNE_64 186 SETGT_64 187 SETGE_64 188 MIN_64 189 MAX_64 190 DOT4
191 DOT4_IEEE 192 CUBE 193 MAX4 196 FREXP_64 197 LDEXP_64 198 FRACT_64
199 PRED_SETGT_64 200 PRED_SETE_64 201 PRED_SETGE_64 203 ADD_64
204 MOVA_INT 207 SAD_ACCUM_PREV_UINT 208 DOT 209 MUL_PREV 210 MUL_IEEE_PREV
211 ADD_PREV 212 MULADD_PREV 213 MULADD_IEEE_PREV 214 INTERP_XY
215 INTERP_ZW 216 INTERP_X 217 INTERP_Z 218 STORE_FLAGS 219 LOAD_STORE_FLAGS
224 INTERP_LOAD_P0 225 INTERP_LOAD_P10 226 INTERP_LOAD_P20
""".split()
OP2 = [(int(OP2[k]), OP2[k + 1]) for k in range(0, len(OP2), 2)]

# (AMD's table writes 66 as PREDE_INT; it is PRED_SETE_INT, as its neighbours
# and llc have it.)

# ALU_WORD1_OP3's opcodes, ALU_INST [17:13]. LDS_IDX_OP (17) has words of
# its own and is left out.
OP3 = """
4 BFE_UINT 5 BFE_INT 6 BFI_INT 7 FMA 9 CNDNE_64 10 FMA_64 11 LERP_UINT
12 BIT_ALIGN_INT 13 BYTE_ALIGN_INT 14 SAD_ACCUM_UINT 15 SAD_ACCUM_HI_UINT
16 MULADD_UINT24 20 MULADD 21 MULADD_M2 22 MULADD_M4 23 MULADD_D2
24 MULADD_IEEE 25 CNDE 26 CNDGT 27 CNDGE 28 CNDE_INT 29 CNDGT_INT
30 CNDGE_INT 31 MUL_LIT
""".split()
OP3 = [(int(OP3[k]), OP3[k + 1]) for k in range(0, len(OP3), 2)]

# How llc spells some mnemonics, and AMD's names for them. (MULHI is the
# unsigned high product: llc writes MULHI_INT as MULHI_INT.)
LLC_SPELLINGS = [("ASHR", "ASHR_INT"), ("LSHR", "LSHR_INT"), ("LSHL", "LSHL_INT"), ("MULHI", "MULHI_UINT")]

files = {}


def out(name, text):
    files.setdefault(name, []).append(text)


def bits(v, w):
    return format(v, f"0{w}b")


# An operand's value is its selector (SRC*_SEL, DST_GPR) times 4, plus its
# channel: T1.Y is 5. An inline constant's channel is X.
out("00-base.ll", """\
; The R600 (Evergreen) description: the files, and the machine as a whole.
; Written by gen_r600.py.
;
;   00-base.ll   what the machine lacks, its parameters, operand names
;   10-alu.ll    ALU instructions (ALU_WORD0 and ALU_WORD1_OP2/OP3)
;   20-cf.ll     control-flow instructions, memory exports, vertex fetches
;
; For assembly and disassembly only: instruction selection has nothing to
; select yet. Instructions are 64 bits.

!target.lacks = !{!1000}
!1000 = !{!"stack", !"calls", !"globals", !"indirect", !"trap"}

; The general-purpose registers, each four 32-bit channels (X, Y, Z, W).
; Instructions name a register and a channel as one operand (an enum,
; below), so the class is only a name for the file.
!target.registers = !{!1050}
!1050 = !{!"t", !"T{0-127}"}
!target.classes = !{!1060}
!1060 = !{!"t", !"file", !"t"}

!target.params = !{!1100, !1101, !1102, !1103}
!1100 = !{!"instruction-unit", !"8"}
!1101 = !{!"vector-bits", !"0"}
!1102 = !{!"asm-comment", !";"}
; EM_AMDGPU.
!1103 = !{!"elf-machine", !"224"}

!target.cpus = !{!1150}
!1150 = !{!"cypress"}

; Operands written by name. A source is a GPR (selectors 0-127), a constant
; of one of four constant-cache banks (128-159, 160-191, 256-287,
; 288-319), an inline constant (248-252), the literal constants that follow
; the instruction group (253), or the previous group's vector (254) or
; transcendental (255) result. Each value is the selector times 4 plus the
; channel.
!target.enums = !{!1200, !1201, !1202, !1203, !1204, !1205, !1206, !1207, !1208}
!1200 = !{!"src", !"0", !"T{0-127}.{X,Y,Z,W}", !"512", !"KC0[{0-31}].{X,Y,Z,W}", !"640", !"KC1[{0-31}].{X,Y,Z,W}", !"1024", !"KC2[{0-31}].{X,Y,Z,W}", !"1152", !"KC3[{0-31}].{X,Y,Z,W}", !"992", !"0.0", !"996", !"1.0", !"1000", !"1", !"1004", !"-1", !"1008", !"0.5", !"1012", !"literal.{x,y,z,w}", !"1016", !"PV.{X,Y,Z,W}", !"1020", !"PS"}
!1201 = !{!"dst", !"0", !"T{0-127}.{X,Y,Z,W}"}
; Negation, written before a source.
!1202 = !{!"neg", !"0", !"", !"1", !"-"}
; The last instruction of a group is written with a star.
!1203 = !{!"last", !"0", !"", !"1", !"* "}
; BANK_SWIZZLE: how the sources are read from the register banks; the
; first, the default, is not written.
!1204 = !{!"bs", !"0", !"", !"1", !" BS:VEC_021/SCL_122", !"2", !" BS:VEC_120/SCL_212", !"3", !" BS:VEC_102/SCL_221", !"4", !" BS:VEC_201", !"5", !" BS:VEC_210", !"0", !" BS:VEC_012/SCL_210"}

; A GPR as a whole, written as its X channel (a vertex fetch's
; destination, a memory export's registers).
!1205 = !{!"gprx", !"0", !"T{0-127}.X"}

; PRED_SEL: whether an instruction runs only where the predicate is 0 or 1.
!1206 = !{!"ps", !"0", !"", !"2", !" Pred_sel_zero", !"3", !" Pred_sel_one"}
; A predicate set that writes no register, by what it updates: bit 0 the
; active mask (UPDATE_EXEC_MASK), bit 1 the predicate (UPDATE_PRED).
!1207 = !{!"pdst", !"1", !"ExecMask,PredicateBit (MASKED)", !"2", !"Pred,PredicateBit (MASKED)"}
; A GPR and which of its channels a memory export writes (COMP_MASK): the
; register times 16, plus the mask.
!1208 = !{!"gprmask", !"0", !"T{0-127}.{,X,Y,XY,Z,XZ,YZ,XYZ,W,XW,YW,XYW,ZW,XZW,YZW,XYZW}"}

; How llc spells some mnemonics.
!target.asm-aliases = !{""" + ", ".join(f"!{1300 + k}" for k in range(len(LLC_SPELLINGS))) + "}\n" + "".join(
    f'!{1300 + k} = !{{!"{a} ", !"{b} "}}\n' for k, (a, b) in enumerate(LLC_SPELLINGS)
))

ENUM = {"vd": "gprx", "vs": "dst", "rw": "gprmask", "ps": "ps", "pdst": "pdst", "ix": "gprx", "last": "last", "dst": "dst", "bs": "bs", "n0": "neg", "n1": "neg", "n2": "neg", "s0": "src", "s1": "src", "s2": "src"}


def entry(name, ops, enc, asm):
    params = ", ".join(f'i32 "enum"="{ENUM[o]}" %{o}' for o in ops)
    return f'define void @{name}({params}) "enc"="{enc}" "asm"="{asm}" "asmonly" {{\n  ret void\n}}\n'


def src(k):
    """A source's fields in ALU_WORD0 (k 0 or 1) or ALU_WORD1_OP3 (k 2):
    NEG, CHAN, REL (0), SEL."""
    return f"n{k}:1 c{k}:2=s{k}&3 0 sel{k}:9=s{k}>>2"


# ALU_WORD0: LAST, PRED_SEL, INDEX_MODE (0), then src1 and src0. A
# one-source instruction has src1 all zeros.
def word0(nsrc):
    s1 = src(1) if nsrc >= 2 else "0" * 13
    s0 = src(0) if nsrc >= 1 else "0" * 13
    return f"last:1 ps:2 000 {s1} {s0}"


# The destination's fields: DST_CHAN, DST_REL (0), DST_GPR.
DST = "dc:2=dst&3 0 dg:7=dst>>2"


def operand(k, ab):
    """Source k as written: negation, then the source, in bars for its
    absolute value."""
    return f"{{n{k}}}|{{s{k}}}|" if ab else f"{{n{k}}}{{s{k}}}"


out("10-alu.ll", "; ALU instructions: ALU_WORD0 and ALU_WORD1_OP2 or ALU_WORD1_OP3, a 64-bit\n; word each. Written by gen_r600.py.\n\n")
for op, name in OP2:
    nsrc = 0 if name in NULLARY else 1 if name in UNARY else 2
    # SRC0_ABS and SRC1_ABS: one entry for each, as the bars are text.
    for a1 in ([0, 1] if nsrc >= 2 else [0]):
        for a0 in ([0, 1] if nsrc >= 1 else [0]):
            # ALU_WORD1_OP2: CLAMP (0), the destination, BANK_SWIZZLE,
            # ALU_INST, OMOD (0), WRITE_MASK (1), UPDATE_PRED and
            # UPDATE_EXEC_MASK (0), SRC1_ABS, SRC0_ABS.
            w1 = f"0 {DST} bs:3 {bits(op, 11)} 00 1 0 0 {a1} {a0}"
            ops = ["last", "dst"] + [x for k in range(nsrc) for x in (f"n{k}", f"s{k}")] + ["ps", "bs"]
            srcs = "".join(f", {operand(k, (a0, a1)[k])}" for k in range(nsrc))
            asm = f"{name} {{last}}{{dst}}{srcs},{{ps}}{{bs}}"
            out("10-alu.ll", entry(f"{name}.{a0}{a1}", ops, f"{w1} {word0(nsrc)}", asm))
            # A predicate set that writes no register (no WRITE_MASK), only
            # the active mask or the predicate.
            if name.startswith("PRED_SET") and nsrc == 2:
                w1 = f"0 00 0 0000000 bs:3 {bits(op, 11)} 00 0 up:1=pdst>>1 uem:1=pdst&1 {a1} {a0}"
                asm = f"{name} {{last}}{{pdst}}{srcs},{{ps}}{{bs}}"
                ops = ["last", "pdst"] + ops[2:]
                out("10-alu.ll", entry(f"{name}.pred.{a0}{a1}", ops, f"{w1} {word0(nsrc)}", asm))

for op, name in OP3:
    # ALU_WORD1_OP3: CLAMP (0), the destination, BANK_SWIZZLE, ALU_INST,
    # then src2.
    w1 = f"0 {DST} bs:3 {bits(op, 5)} {src(2)}"
    ops = ["last", "dst", "n0", "s0", "n1", "s1", "n2", "s2", "ps", "bs"]
    asm = f"{name} {{last}}{{dst}}, {operand(0, 0)}, {operand(1, 0)}, {operand(2, 0)},{{ps}}{{bs}}"
    out("10-alu.ll", entry(name, ops, f"{w1} {word0(2)}", asm))

# The literal constants of an instruction group follow its last
# instruction, two to a 64-bit word: literal.x and literal.y, then
# literal.z and literal.w. (llc writes each with its value as a float,
# `2(2.802597e-45)`; that is a comment, and not read.)
out("10-alu.ll", """
define void @LITERALS(i32 "imm" %x, i32 "imm" %y) "enc"="y:s32 x:s32" "asm"="LITERALS {x}, {y}" "asmonly" {
  ret void
}
""")

# Control flow. CF_WORD1's opcodes, CF_INST [29:22], by llc's names where it
# has them (AMD's LOOP_END is END_LOOP, TC and VC are TEX and VTX), with
# their forms: a clause (a count of instructions, less one, and its
# address), or a jump (an address, and how many entries to pop from the
# stack). BARRIER is set, as llc sets it.
CF = [
    (1, "TEX", "clause"), (2, "VTX", "clause"), (3, "GDS", "clause"), (4, "LOOP_START", "jump"),
    (5, "END_LOOP", "jump"), (6, "LOOP_START_DX10", "jump"), (7, "LOOP_START_NO_AL", "jump"),
    (8, "LOOP_CONTINUE", "jump"), (9, "LOOP_BREAK", "jump"), (10, "JUMP", "jump"), (11, "PUSH", "jump"),
    (13, "ELSE", "jump"), (14, "POP", "jump"), (18, "CALL", "jump"), (20, "RETURN", "jump"),
    (24, "KILL", "jump"), (26, "WAIT_ACK", "jump"), (27, "TEX_ACK", "clause"), (28, "VTX_ACK", "clause"),
]
out("20-cf.ll", "; Control-flow instructions (CF_WORD0/1, CF_ALU_WORD0/1), memory exports\n; (CF_ALLOC_EXPORT_WORD0_RAT/1_BUF) and vertex fetches (VTX_WORD0/1/2). Written by\n; gen_r600.py.\n\n")


def imm_entry(name, ops, enc, asm):
    params = ", ".join(f'i32 "imm" %{o}' if o not in ENUM else f'i32 "enum"="{ENUM[o]}" %{o}' for o in ops)
    return f'define void @{name}({params}) "enc"="{enc}" "asm"="{asm}" "asmonly" {{\n  ret void\n}}\n'


for op, name, form in CF:
    # CF_WORD1: BARRIER, WHOLE_QUAD_MODE (0), CF_INST, END_OF_PROGRAM (0),
    # VALID_PIXEL_MODE (0), reserved, COUNT, COND (0), CF_CONST (0),
    # POP_COUNT. CF_WORD0: reserved, JUMPTABLE_SEL (0), ADDR.
    if form == "clause":
        enc = f"1 0 {bits(op, 8)} 0 0 0000 count:6 00 00000 000 00000 000 addr:24"
        out("20-cf.ll", imm_entry(name, ["count", "addr"], enc, f"{name} {{count}} @{{addr}}"))
    else:
        enc = f"1 0 {bits(op, 8)} 0 0 0000 000000 00 00000 pop:3 00000 000 addr:24"
        out("20-cf.ll", imm_entry(name, ["addr"], enc.replace("pop:3", "000"), f"{name} @{{addr}}"))
        out("20-cf.ll", imm_entry(f"{name}.pop", ["addr", "pop"], enc, f"{name} @{{addr}} POP:{{pop}}"))

# A NOP that ends the program (END_OF_PROGRAM), and one with no barrier, to
# pad.
out("20-cf.ll", """
define void @CF_END() "enc"="1 0 00000000 1 0 0000 000000 00 00000 000 00000 000 000000000000000000000000" "asm"="CF_END" "asmonly" {
  ret void
}

define void @PAD() "enc"="0000000000000000000000000000000000000000000000000000000000000000" "asm"="PAD" "asmonly" {
  ret void
}

""")

# ALU clauses: CF_ALU_WORD1's opcodes, CF_INST [29:26], with the clause's
# instruction count less one, its address, and the constant-cache lines it
# locks for KC0 and KC1: none (KCACHE_MODE 0, written KC0[]) or two lines
# from line ka of buffer kb (mode 2, written KC0[CBkb:16ka-16ka+32]).
CF_ALU = [(8, "ALU"), (9, "ALU_PUSH_BEFORE"), (10, "ALU_POP_AFTER"), (11, "ALU_POP2_AFTER"), (13, "ALU_CONTINUE"),
          (14, "ALU_BREAK"), (15, "ALU_ELSE_AFTER")]
for op, name in CF_ALU:
    for m0 in (0, 2):
        for m1 in (0, 2):
            kc = []
            ops = ["count", "addr"]
            for k, m in ((0, m0), (1, m1)):
                if m == 2:
                    kc.append(f"KC{k}[CB{{kb{k}}}:{{=ka{k}<<4}}-{{=(ka{k}<<4)+32}}]")
                    ops += [f"kb{k}", f"ka{k}"]
                else:
                    kc.append(f"KC{k}[]")
            f = lambda k, m, w, name: f"{name}{k}:{w}" if m == 2 else "0" * w
            # CF_ALU_WORD1: BARRIER, WHOLE_QUAD_MODE (0), CF_INST, ALT_CONST
            # (0), COUNT, KCACHE_ADDR1, KCACHE_ADDR0, KCACHE_MODE1.
            # CF_ALU_WORD0: KCACHE_MODE0, KCACHE_BANK1, KCACHE_BANK0, ADDR.
            w1 = f"1 0 {bits(op, 4)} 0 count:7 {f(1, m1, 8, 'ka')} {f(0, m0, 8, 'ka')} {bits(m1, 2)}"
            w0 = f"{bits(m0, 2)} {f(1, m1, 4, 'kb')} {f(0, m0, 4, 'kb')} addr:22"
            out("20-cf.ll", imm_entry(f"{name}.{m0}{m1}", ops, f"{w1} {w0}", f"{name} {{count}}, @{{addr}}, {kc[0]}, {kc[1]}"))

# A store to memory through a random-access target without the cache
# (CF_INST 87, MEM_RAT_CACHELESS; RAT_INST 2, STORE_RAW): the value's
# register rw with the channels written (COMP_MASK), the index's register
# ix, written as its X channel, and whether it ends the program. CF_ALLOC_EXPORT_WORD1_BUF: BARRIER, MARK (0), CF_INST,
# END_OF_PROGRAM, VALID_PIXEL_MODE (0), BURST_COUNT (0), COMP_MASK,
# ARRAY_SIZE (0). CF_ALLOC_EXPORT_WORD0_RAT: ELEM_SIZE (0), INDEX_GPR,
# RW_REL (0), RW_GPR, TYPE (1, EXPORT_WRITE_IND), RAT_INDEX_MODE (0), a
# reserved bit, RAT_INST, RAT_ID (0).
out("20-cf.ll", imm_entry("MEM_RAT_CACHELESS_STORE_RAW", ["rw", "ix", "eop"],
                          "1 0 01010111 eop:1 0 0000 cm:4=rw&15 000000000000 00 ix:7 0 rg:7=rw>>4 01 00 0 000010 0000",
                          "MEM_RAT_CACHELESS STORE_RAW {rw}, {ix}, {eop}"))

# A 32-bit fetch through the vertex cache (VC_INST 0, FETCH), into the X
# channel of vd, from buffer buf at an offset from the index in vs's
# channel: 16 bytes, the last 4 zero. VTX_WORD1_GPR: SRF_MODE_ALL (0),
# FORMAT_COMP_ALL (0), NUM_FORMAT_ALL (1), DATA_FORMAT (13, FMT_32),
# USE_CONST_FIELDS (0), DST_SEL_W, _Z, _Y (7, masked), DST_SEL_X (0), a
# reserved bit, DST_REL (0), DST_GPR. VTX_WORD0: MEGA_FETCH_COUNT (4),
# SRC_SEL_X, SRC_REL (0), SRC_GPR, BUFFER_ID, FETCH_WHOLE_QUAD (0),
# FETCH_TYPE (2), VC_INST. VTX_WORD2: reserved, MEGA_FETCH (1),
# the rest 0, OFFSET.
out("20-cf.ll", imm_entry("VTX_READ_32", ["vd", "vs", "off", "buf"],
                          "0 0 01 001101 0 111 111 111 000 0 0 vd:7 000100 sc:2=vs&3 0 sg:7=vs>>2 buf:8 0 10 00000"
                          " | 00000000000000000000000000000000 000000000000 1 000 off:16",
                          "VTX_READ_32 {vd}, {vs}, {off}, #{buf}"))

for name, parts in files.items():
    with open(os.path.join(HERE, name), "w") as f:
        f.write("".join(parts))
