"""Writes the RISC-V RV64 description: the .ll files of this directory.

The description is data, read by epitome-llc at start-up like the
built-in AArch64 one, but not built in: point at it with
--target=codegen/targets/riscv64, or put codegen/targets on
EPITOME_TARGET_PATH and say --target=riscv64. This script only writes it;
run it from anywhere:

    python3 codegen/targets/riscv64/gen_riscv64.py

From the RISC-V Instruction Set Manual, Volume I (Unprivileged ISA): RV64I,
the M extension, F and D, and Zbb; and the RISC-V ELF psABI (the LP64D
calling convention, the relocations).
"""
import os

HERE = os.path.dirname(os.path.abspath(__file__))
files = {}
declared = set()


def declare(text):
    """Declarations (of intrinsics), each once: the files are one module."""
    lines = [l for l in text.strip().split("\n") if l not in declared]
    declared.update(lines)
    return "\n".join(lines) + "\n" if lines else ""


def out(name, text):
    files.setdefault(name, []).append(text)


HEADER = """; The RISC-V RV64 description: {what}. Written by gen_riscv64.py; the
; conventions are those of codegen/targets/aarch64/00-base.ll.
"""

# ---------------------------------------------------------------------------
# Encodings

def R(f7, rs2, rs1, f3, rd, op):
    return f"{f7} {rs2} {rs1} {f3} {rd} {op}"


def I(imm, rs1, f3, rd, op):
    return f"{imm} {rs1} {f3} {rd} {op}"


S_IMM = "hi:s7=off>>5 {rs2} {rs1} {f3} lo:5=off&31 {op}"
B_IMM = "b12:s1=dest>>12 b10:6=(dest>>5)&63 {rs2} {rs1} {f3} b4:4=(dest>>1)&15 b11:1=(dest>>11)&1 1100011"
J_IMM = "j20:s1=dest>>20 j10:10=(dest>>1)&1023 j11:1=(dest>>11)&1 j19:8=(dest>>12)&255 {rd} 1101111"

OP, OP32, OPI, OPI32 = "0110011", "0111011", "0010011", "0011011"
LOAD, STORE, LUI = "0000011", "0100011", "0110111"
OPFP, LOADFP, STOREFP = "1010011", "0000111", "0100111"


def define(ret, name, params, enc, asm, body, attrs=""):
    return f'define {ret} @{name}({params}) "enc"="{enc}" "asm"="{asm}"{attrs} {{\n{body}\n}}\n'


TY = {"x": "i64", "w": "i32"}

# ---------------------------------------------------------------------------
# 00-base.ll

out("00-base.ll", HEADER.format(what="the files, and the machine as a whole") + """
; The description is these files, read in name order as one module:
;
;   00-base.ll      registers, parameters, CPUs, relocations
;   10-integer.ll   RV64I: arithmetic, logic, shifts, comparisons, moves,
;                   constants, loads and stores, branches, calls
;   20-m.ll         the M extension: multiplication and division
;   25-a.ll         the A extension (atomics), and fences
;   30-zbb.ll       Zbb (basic bit manipulation), used when it is on
;   40-float.ll     the F and D extensions
;   60-recipes.ll   recipes: IR with no single instruction
;   70-abi.ll       the LP64D calling convention
;
; Compressed (C) and vector (V) instructions are for assembly and disassembly
; only (80-asm.ll); the code generator does not use them. Not yet: PIC.

; ===========================================================================
; Registers, by their ABI names, in two files: the integer registers
; x0-x31 (x0 reads as zero) and the floating-point ones f0-f31. The w
; classes are the integer registers holding 32-bit values, which are kept
; sign-extended to 64 bits ("narrow-results"="sign"); s and d are the
; floating-point registers holding floats and doubles.

!target.registers = !{!1000, !1001, !1002, !1003, !1004, !1005, !1006, !1007, !1008}
!1000 = !{!"x", !"zero", !"ra", !"sp", !"gp", !"tp", !"t{0-2}", !"s{0-1}", !"a{0-7}", !"s{2-11}", !"t{3-6}"}
!1001 = !{!"w", !"zero", !"ra", !"sp", !"gp", !"tp", !"t{0-2}", !"s{0-1}", !"a{0-7}", !"s{2-11}", !"t{3-6}"}
!1002 = !{!"xsp", !"zero", !"ra", !"sp", !"gp", !"tp", !"t{0-2}", !"s{0-1}", !"a{0-7}", !"s{2-11}", !"t{3-6}"}
!1003 = !{!"wsp", !"zero", !"ra", !"sp", !"gp", !"tp", !"t{0-2}", !"s{0-1}", !"a{0-7}", !"s{2-11}", !"t{3-6}"}
!1004 = !{!"d", !"ft{0-7}", !"fs{0-1}", !"fa{0-7}", !"fs{2-11}", !"ft{8-11}"}
!1005 = !{!"s", !"ft{0-7}", !"fs{0-1}", !"fa{0-7}", !"fs{2-11}", !"ft{8-11}"}
; The registers by number too, as assembly may name them (x10, f8).
!1006 = !{!"xn", !"x{0-31}"}
!1007 = !{!"fn", !"f{0-31}"}
; The vector registers, which only assembly names (V is not used by the code
; generator).
!1008 = !{!"v", !"v{0-31}"}

!target.classes = !{!1010, !1011, !1012, !1013, !1014, !1015, !1016, !1017, !1018}
!1010 = !{!"x", !"file", !"x", !"bits", !"64", !"holds", !"int"}
!1011 = !{!"w", !"file", !"x", !"bits", !"32", !"holds", !"int"}
!1012 = !{!"xsp", !"file", !"x", !"bits", !"64", !"holds", !"int", !"sp", !"yes"}
!1013 = !{!"wsp", !"file", !"x", !"bits", !"32", !"holds", !"int", !"sp", !"yes"}
!1014 = !{!"d", !"file", !"f", !"bits", !"64", !"holds", !"float"}
!1015 = !{!"s", !"file", !"f", !"bits", !"32", !"holds", !"float"}
!1016 = !{!"xn", !"file", !"x", !"names-of", !"x"}
!1017 = !{!"fn", !"file", !"f", !"names-of", !"d"}
!1018 = !{!"v", !"file", !"v"}

; ===========================================================================
; Parameters. No vector registers (V is not described): vectors are done
; lane by lane. Globals are reached by their absolute addresses (the medlow
; code model), not through a GOT. Branches compare whole registers.

!target.params = !{!1100, !1101, !1102, !1103, !1104, !1105, !1106, !1107, !1108, !1109, !1110, !1111, !1112}
!1100 = !{!"sp", !"sp"}
!1101 = !{!"zero", !"zero"}
!1102 = !{!"vector-bits", !"0"}
!1103 = !{!"narrow-results", !"sign"}
!1104 = !{!"external-via-got", !"no"}
!1105 = !{!"elf-machine", !"243"}
; EF_RISCV_FLOAT_ABI_DOUBLE: the LP64D convention.
!1106 = !{!"elf-flags", !"4"}
!1107 = !{!"cpu", !"generic-rv64"}
!1108 = !{!"boolean-branches", !"compare"}
!1109 = !{!"elf-format", !"elf64-littleriscv"}
!1110 = !{!"asm-comment", !"#"}
; The data alignment factor of unwind tables.
!1111 = !{!"cfi-data-align", !"-8"}
; Assembly may use li of any value, la and lla (asm_text.rs).
!1112 = !{!"asm-pseudos", !"riscv"}
; Code is padded with nop (and with C, c.nop), as the assemblers pad it.
!target.nops = !{!1113}
!1113 = !{!"00", !"0100", !"000000", !"13000000"}

; ===========================================================================
; Inline assembly: what its constraint codes and template modifiers mean,
; from the LLVM Language Reference (z, the zero register for a constant 0,
; is the code generator's own).

!target.asm-constraints = !{!1300, !1301, !1302, !1303, !1304, !1305, !1306}
!1300 = !{!"r", !"file", !"x", !"print", !"x"}
!1301 = !{!"f", !"file", !"f", !"print", !"d"}
!1302 = !{!"I", !"imm", !"-2048", !"2047"}
!1303 = !{!"J", !"imm", !"0", !"0"}
!1304 = !{!"K", !"imm", !"0", !"31"}
!1305 = !{!"m", !"mem", !"0({r})"}
!1306 = !{!"A", !"mem", !"({r})"}

!target.asm-modifiers = !{!1310}
!1310 = !{!"i", !"if-imm", !"i"}

; Pseudo-instructions and operands as people write them, and as the
; templates do.
!target.asm-aliases = !{!1320, !1321, !1322, !1323, !1324, !1325, !1354, !1355, !1356, !1326, !1327, !1328, !1329, !1330, !1331, !1332, !1333, !1334, !1335, !1336, !1337, !1338, !1339, !1340, !1341, !1342, !1343, !1344, !1345, !1346, !1347, !1348, !1349, !1350, !1351, !1352, !1353, !1357, !1358, !1359, !1360, !1361, !1362, !1363, !1364, !1365, !1366, !1367, !1368, !1369, !1370, !1371, !1372, !1373, !1374, !1375, !1376, !1377, !1378, !1379, !1380, !1381, !1382, !1383, !1384, !1385, !1386, !1387, !1388, !1389, !1390, !1391, !1392, !1393, !1402, !1403, !1404, !1405, !1406, !1407, !1408, !1409, !1410, !1411, !1412, !1413, !1414, !1415, !1416}
!1320 = !{!"beqz {r}, {l}", !"beq {r}, zero, {l}"}
!1321 = !{!"bnez {r}, {l}", !"bne {r}, zero, {l}"}
!1322 = !{!"bltz {r}, {l}", !"blt {r}, zero, {l}"}
!1323 = !{!"bgez {r}, {l}", !"bge {r}, zero, {l}"}
!1324 = !{!", ({r})", !", 0({r})"}
!1325 = !{!"not {d}, {r}", !"xori {d}, {r}, -1"}
; The CSR instructions' pseudo-instructions, with CSRs by number (fcsr is 3,
; cycle 0xc00), and the branches with their operands the other way round.
!1326 = !{!"csrw {c}, {r}", !"csrrw zero, {c}, {r}"}
!1327 = !{!"csrs {c}, {r}", !"csrrs zero, {c}, {r}"}
!1328 = !{!"csrc {c}, {r}", !"csrrc zero, {c}, {r}"}
!1329 = !{!"csrwi {c}, {i}", !"csrrwi zero, {c}, {i}"}
!1330 = !{!"csrsi {c}, {i}", !"csrrsi zero, {c}, {i}"}
!1331 = !{!"csrci {c}, {i}", !"csrrci zero, {c}, {i}"}
!1332 = !{!"frcsr {d}", !"csrrs {d}, 3, zero"}
!1333 = !{!"fscsr {d}, {r}", !"csrrw {d}, 3, {r}"}
!1334 = !{!"fscsr {r}", !"csrrw zero, 3, {r}"}
!1335 = !{!"frsr {d}", !"csrrs {d}, 3, zero"}
!1336 = !{!"fssr {d}, {r}", !"csrrw {d}, 3, {r}"}
!1337 = !{!"fssr {r}", !"csrrw zero, 3, {r}"}
!1338 = !{!"frflags {d}", !"csrrs {d}, 1, zero"}
!1339 = !{!"fsflags {d}, {r}", !"csrrw {d}, 1, {r}"}
!1340 = !{!"fsflags {r}", !"csrrw zero, 1, {r}"}
!1341 = !{!"frrm {d}", !"csrrs {d}, 2, zero"}
!1342 = !{!"fsrm {d}, {r}", !"csrrw {d}, 2, {r}"}
!1343 = !{!"fsrm {r}", !"csrrw zero, 2, {r}"}
!1344 = !{!"rdcycle {d}", !"csrrs {d}, 3072, zero"}
!1345 = !{!"rdtime {d}", !"csrrs {d}, 3073, zero"}
!1346 = !{!"rdinstret {d}", !"csrrs {d}, 3074, zero"}
!1347 = !{!"unimp", !"csrrw zero, 3072, zero"}
!1348 = !{!"bgt {a}, {b}, {l}", !"blt {b}, {a}, {l}"}
!1349 = !{!"ble {a}, {b}, {l}", !"bge {b}, {a}, {l}"}
!1350 = !{!"bgtu {a}, {b}, {l}", !"bltu {b}, {a}, {l}"}
!1351 = !{!"bleu {a}, {b}, {l}", !"bgeu {b}, {a}, {l}"}
!1352 = !{!"bgtz {r}, {l}", !"blt zero, {r}, {l}"}
!1353 = !{!"blez {r}, {l}", !"bge zero, {r}, {l}"}
; (CSR instructions of an immediate, their operand a number, before those of a
; register; a register-register instruction of an immediate is its immediate
; form, as GNU assemblers take it; Zalasr's addresses with an offset of 0.)
!1354 = !{!"csrw {c}, {i}", !"csrrwi zero, {c}, {i}"}
!1355 = !{!"csrs {c}, {i}", !"csrrsi zero, {c}, {i}"}
!1356 = !{!"csrc {c}, {i}", !"csrrci zero, {c}, {i}"}
!1357 = !{!"fence", !"fence iorw, iorw"}
!1358 = !{!"jalr {d}, {r}, {i}", !"jalr {d}, {i}({r})"}
!1359 = !{!"add {d}, {r}, {i}", !"addi {d}, {r}, {i}"}
!1360 = !{!"and {d}, {r}, {i}", !"andi {d}, {r}, {i}"}
!1361 = !{!"or {d}, {r}, {i}", !"ori {d}, {r}, {i}"}
!1362 = !{!"xor {d}, {r}, {i}", !"xori {d}, {r}, {i}"}
!1363 = !{!"sll {d}, {r}, {i}", !"slli {d}, {r}, {i}"}
!1364 = !{!"srl {d}, {r}, {i}", !"srli {d}, {r}, {i}"}
!1365 = !{!"sra {d}, {r}, {i}", !"srai {d}, {r}, {i}"}
!1366 = !{!"slt {d}, {r}, {i}", !"slti {d}, {r}, {i}"}
!1367 = !{!"sltu {d}, {r}, {i}", !"sltiu {d}, {r}, {i}"}
!1368 = !{!"addw {d}, {r}, {i}", !"addiw {d}, {r}, {i}"}
!1369 = !{!"sllw {d}, {r}, {i}", !"slliw {d}, {r}, {i}"}
!1370 = !{!"srlw {d}, {r}, {i}", !"srliw {d}, {r}, {i}"}
!1371 = !{!"sraw {d}, {r}, {i}", !"sraiw {d}, {r}, {i}"}
!1372 = !{!"ror {d}, {r}, {i}", !"rori {d}, {r}, {i}"}
!1373 = !{!"rorw {d}, {r}, {i}", !"roriw {d}, {r}, {i}"}
!1374 = !{!"bset {d}, {r}, {i}", !"bseti {d}, {r}, {i}"}
!1375 = !{!"bclr {d}, {r}, {i}", !"bclri {d}, {r}, {i}"}
!1376 = !{!"binv {d}, {r}, {i}", !"binvi {d}, {r}, {i}"}
!1377 = !{!"bext {d}, {r}, {i}", !"bexti {d}, {r}, {i}"}
!1378 = !{!"lb.aq {d}, 0({r})", !"lb.aq {d}, ({r})"}
!1379 = !{!"lb.aqrl {d}, 0({r})", !"lb.aqrl {d}, ({r})"}
!1380 = !{!"lh.aq {d}, 0({r})", !"lh.aq {d}, ({r})"}
!1381 = !{!"lh.aqrl {d}, 0({r})", !"lh.aqrl {d}, ({r})"}
!1382 = !{!"lw.aq {d}, 0({r})", !"lw.aq {d}, ({r})"}
!1383 = !{!"lw.aqrl {d}, 0({r})", !"lw.aqrl {d}, ({r})"}
!1384 = !{!"ld.aq {d}, 0({r})", !"ld.aq {d}, ({r})"}
!1385 = !{!"ld.aqrl {d}, 0({r})", !"ld.aqrl {d}, ({r})"}
!1386 = !{!"sb.rl {d}, 0({r})", !"sb.rl {d}, ({r})"}
!1387 = !{!"sb.aqrl {d}, 0({r})", !"sb.aqrl {d}, ({r})"}
!1388 = !{!"sh.rl {d}, 0({r})", !"sh.rl {d}, ({r})"}
!1389 = !{!"sh.aqrl {d}, 0({r})", !"sh.aqrl {d}, ({r})"}
!1390 = !{!"sw.rl {d}, 0({r})", !"sw.rl {d}, ({r})"}
!1391 = !{!"sw.aqrl {d}, 0({r})", !"sw.aqrl {d}, ({r})"}
!1392 = !{!"sd.rl {d}, 0({r})", !"sd.rl {d}, ({r})"}
!1393 = !{!"sd.aqrl {d}, 0({r})", !"sd.aqrl {d}, ({r})"}
; (Hypervisor loads and stores with an offset of 0, and jr of one.)
!1402 = !{!"jr {r}, {i}", !"jalr zero, {i}({r})"}
!1403 = !{!"hlv.b {d}, 0({r})", !"hlv.b {d}, ({r})"}
!1404 = !{!"hlv.bu {d}, 0({r})", !"hlv.bu {d}, ({r})"}
!1405 = !{!"hlv.h {d}, 0({r})", !"hlv.h {d}, ({r})"}
!1406 = !{!"hlv.hu {d}, 0({r})", !"hlv.hu {d}, ({r})"}
!1407 = !{!"hlvx.hu {d}, 0({r})", !"hlvx.hu {d}, ({r})"}
!1408 = !{!"hlv.w {d}, 0({r})", !"hlv.w {d}, ({r})"}
!1409 = !{!"hlvx.wu {d}, 0({r})", !"hlvx.wu {d}, ({r})"}
!1410 = !{!"hlv.wu {d}, 0({r})", !"hlv.wu {d}, ({r})"}
!1411 = !{!"hlv.d {d}, 0({r})", !"hlv.d {d}, ({r})"}
!1412 = !{!"hsv.b {d}, 0({r})", !"hsv.b {d}, ({r})"}
!1413 = !{!"hsv.h {d}, 0({r})", !"hsv.h {d}, ({r})"}
!1414 = !{!"hsv.w {d}, 0({r})", !"hsv.w {d}, ({r})"}
!1415 = !{!"hsv.d {d}, 0({r})", !"hsv.d {d}, ({r})"}
; (An atomic's address with an offset of 0.)
!1416 = !{!"{m} {d}, {s}, 0({r})", !"{m} {d}, {s}, ({r})"}
; Branches too far for their 12 bits: the other way round, over a jump.
!target.asm-relax = !{!1500, !1501, !1502, !1503, !1504, !1505}
!1500 = !{!"beq {a}, {b}, {l}", !"bne {a}, {b}, .+8; j {l}"}
!1501 = !{!"bne {a}, {b}, {l}", !"beq {a}, {b}, .+8; j {l}"}
!1502 = !{!"blt {a}, {b}, {l}", !"bge {a}, {b}, .+8; j {l}"}
!1503 = !{!"bge {a}, {b}, {l}", !"blt {a}, {b}, .+8; j {l}"}
!1504 = !{!"bltu {a}, {b}, {l}", !"bgeu {a}, {b}, .+8; j {l}"}
!1505 = !{!"bgeu {a}, {b}, {l}", !"bltu {a}, {b}, .+8; j {l}"}

; ===========================================================================
; CPUs, by LLVM's names for them and their features.

!target.cpus = !{!1150}
!1150 = !{!"generic-rv64", !"m", !"a", !"f", !"d"}

; ===========================================================================
; Relocations, from the RISC-V ELF psABI.

!target.relocs = !{!1200, !1201, !1202, !1203, !1210, !1211, !1212, !1213, !1214, !1215, !1216, !1217, !1218, !1219, !1224}
!1200 = !{!"name", !"jal", !"elf", !"17", !"elf-name", !"R_RISCV_JAL", !"use", !"call", !"target", !"pc + v"}
!1201 = !{!"name", !"hi20", !"elf", !"26", !"elf-name", !"R_RISCV_HI20", !"elf-syntax", !"%hi({s})", !"target", !"v << 12"}
!1202 = !{!"name", !"lo12_i", !"elf", !"27", !"elf-name", !"R_RISCV_LO12_I", !"elf-syntax", !"%lo({s})"}
!1203 = !{!"name", !"abs64", !"elf", !"2", !"elf-name", !"R_RISCV_64", !"use", !"data8"}
!1210 = !{!"name", !"none", !"elf", !"0", !"elf-name", !"R_RISCV_NONE"}
!1211 = !{!"name", !"abs32", !"elf", !"1", !"elf-name", !"R_RISCV_32"}
; How .eh_frame says where a function is.
!1219 = !{!"name", !"pcrel32", !"elf", !"57", !"elf-name", !"R_RISCV_32_PCREL", !"use", !"eh-pcrel"}

; DWARF's numbers for the registers, in unwind tables: x0-x31 are 0-31,
; f0-f31 32-63.
!target.dwarf = !{!1400, !1401}
!1400 = !{!"x", !"0"}
!1401 = !{!"f", !"32"}
!1212 = !{!"name", !"branch", !"elf", !"16", !"elf-name", !"R_RISCV_BRANCH"}
!1213 = !{!"name", !"call", !"elf", !"18", !"elf-name", !"R_RISCV_CALL"}
!1214 = !{!"name", !"call_plt", !"elf", !"19", !"elf-name", !"R_RISCV_CALL_PLT"}
!1215 = !{!"name", !"got_hi20", !"elf", !"20", !"elf-name", !"R_RISCV_GOT_HI20"}
!1216 = !{!"name", !"pcrel_hi20", !"elf", !"23", !"elf-name", !"R_RISCV_PCREL_HI20", !"elf-syntax", !"%pcrel_hi({s})"}
; (A %pcrel_lo's symbol is the label of the auipc with the %pcrel_hi.)
!1217 = !{!"name", !"pcrel_lo12_i", !"elf", !"24", !"elf-name", !"R_RISCV_PCREL_LO12_I", !"elf-syntax", !"%pcrel_lo({s})", !"use", !"pcrel-lo"}
!1224 = !{!"name", !"pcrel_lo12_s", !"elf", !"25", !"elf-name", !"R_RISCV_PCREL_LO12_S", !"elf-syntax", !"%pcrel_lo({s})", !"use", !"pcrel-lo"}
!1218 = !{!"name", !"lo12_s", !"elf", !"28", !"elf-name", !"R_RISCV_LO12_S"}
""")

# ---------------------------------------------------------------------------
# 10-integer.ll

I_ = []

def w(s):
    I_.append(s)

w(HEADER.format(what="RV64I, the base integer instructions"))

# Register-register operations, 64 and 32 bits.
for name, ir, f7, f3, x_ok, w_name in [
    ("ADD", "add", "0000000", "000", True, "ADDW"),
    ("SUB", "sub", "0100000", "000", True, "SUBW"),
    ("SLL", "shl", "0000000", "001", True, "SLLW"),
    ("SRL", "lshr", "0000000", "101", True, "SRLW"),
    ("SRA", "ashr", "0100000", "101", True, "SRAW"),
]:
    role = {"ADD": ' "role"="add"', "SUB": ' "role"="sub"'}.get(name, "")
    w(define("i64", name, 'i64 "reg"="x" %n, i64 "reg"="x" %m', R(f7, "m:5", "n:5", f3, "d:5", OP),
             f"{name.lower()} {{d}}, {{n}}, {{m}}", f"  %r = {ir} i64 %n, %m\n  ret i64 %r", role))
    w(define("i32", w_name, 'i32 "reg"="w" %n, i32 "reg"="w" %m', R(f7, "m:5", "n:5", f3, "d:5", OP32),
             f"{w_name.lower()} {{d}}, {{n}}, {{m}}", f"  %r = {ir} i32 %n, %m\n  ret i32 %r"))
for name, ir, f3 in [("AND", "and", "111"), ("OR", "or", "110"), ("XOR", "xor", "100")]:
    for c, ty, extra in [("x", "i64", ""), ("w", "i32", ' "nodecode"'), ("b", "i1", ' "nodecode"')]:
        cls = "w" if c != "x" else "x"
        w(define(ty, f"{name}{'' if c == 'x' else c}", f'{ty} "reg"="{cls}" %n, {ty} "reg"="{cls}" %m',
                 R("0000000", "m:5", "n:5", f3, "d:5", OP), f"{name.lower()} {{d}}, {{n}}, {{m}}",
                 f"  %r = {ir} {ty} %n, %m\n  ret {ty} %r", extra))

# Adding to a pointer.
w(define("ptr", "ADDp", 'ptr "reg"="x" %n, i64 "reg"="x" %m', R("0000000", "m:5", "n:5", "000", "d:5", OP),
         "add {d}, {n}, {m}", "  %r = getelementptr i8, ptr %n, i64 %m\n  ret ptr %r", ' "nodecode"'))

# Immediates: 12 bits, signed.
w(define("i64", "ADDI", 'i64 "reg"="x" %n, i64 "imm" %imm', I("imm:s12", "n:5", "000", "d:5", OPI),
         "addi {d}, {n}, {imm}", "  %r = add i64 %n, %imm\n  ret i64 %r", ' "role"="addi"'))
w(define("ptr", "ADDIp", 'ptr "reg"="x" %n, i64 "imm" %imm', I("imm:s12", "n:5", "000", "d:5", OPI),
         "addi {d}, {n}, {imm}", "  %r = getelementptr i8, ptr %n, i64 %imm\n  ret ptr %r", ' "nodecode"'))
w(define("i32", "ADDIW", 'i32 "reg"="w" %n, i32 "imm" %imm', I("imm:s12", "n:5", "000", "d:5", OPI32),
         "addiw {d}, {n}, {imm}", "  %r = add i32 %n, %imm\n  ret i32 %r", ' "role"="addi.w"'))
for name, ir, f3 in [("ANDI", "and", "111"), ("ORI", "or", "110"), ("XORI", "xor", "100")]:
    for c, ty, extra in [("x", "i64", ""), ("w", "i32", ' "nodecode"'), ("b", "i1", ' "nodecode"')]:
        cls = "x" if c == "x" else "w"
        w(define(ty, f"{name}{'' if c == 'x' else c}", f'{ty} "reg"="{cls}" %n, {ty} "imm" %imm',
                 I("imm:s12", "n:5", f3, "d:5", OPI), f"{name.lower()} {{d}}, {{n}}, {{imm}}",
                 f"  %r = {ir} {ty} %n, %imm\n  ret {ty} %r", extra))

# Shifts by an immediate: 6 bits (64-bit), 5 bits (32-bit, the w forms).
for name, ir, top, f3 in [("SLLI", "shl", "000000", "001"), ("SRLI", "lshr", "000000", "101"), ("SRAI", "ashr", "010000", "101")]:
    role = ' "role"="shli"' if name == "SLLI" else ""
    w(define("i64", name, 'i64 "reg"="x" %n, i64 "imm" %sh', f"{top} sh:6 n:5 {f3} d:5 {OPI}",
             f"{name.lower()} {{d}}, {{n}}, {{sh}}", f"  %r = {ir} i64 %n, %sh\n  ret i64 %r", role))
    w(define("i32", name + "W", 'i32 "reg"="w" %n, i32 "imm" %sh', f"{top}0 sh:5 n:5 {f3} d:5 {OPI32}",
             f"{name.lower()}w {{d}}, {{n}}, {{sh}}", f"  %r = {ir} i32 %n, %sh\n  ret i32 %r"))

# Negation, and sext.w (a 64-bit value's low 32 bits, sign-extended).
w(define("i64", "NEG", 'i64 "reg"="x" %n', R("0100000", "n:5", "00000", "000", "d:5", OP),
         "neg {d}, {n}", "  %r = sub i64 0, %n\n  ret i64 %r"))
w(define("i32", "NEGW", 'i32 "reg"="w" %n', R("0100000", "n:5", "00000", "000", "d:5", OP32),
         "negw {d}, {n}", "  %r = sub i32 0, %n\n  ret i32 %r"))
w(define("i32", "SEXTW", 'i64 "reg"="x" %n', I("000000000000", "n:5", "000", "d:5", OPI32),
         "sext.w {d}, {n}", "  %r = trunc i64 %n to i32\n  ret i32 %r"))

# Comparisons that give a boolean, 0 or 1.
for name, pred, f3 in [("SLT", "slt", "010"), ("SLTU", "ult", "011")]:
    for c, ty, extra in [("x", "i64", ""), ("w", "i32", ' "nodecode"')]:
        sfx = "" if c == "x" else "w"
        w(define("i1", name + sfx, f'{ty} "reg"="{c}" %n, {ty} "reg"="{c}" %m', R("0000000", "m:5", "n:5", f3, "d:5", OP),
                 f"{name.lower()} {{d}}, {{n}}, {{m}}", f"  %r = icmp {pred} {ty} %n, %m\n  ret i1 %r", ' "zext"="1"' + extra))
        swapped = {"slt": "sgt", "ult": "ugt"}[pred]
        w(define("i1", name + "s" + sfx, f'{ty} "reg"="{c}" %n, {ty} "reg"="{c}" %m',
                 R("0000000", "rb:5=n", "ra:5=m", f3, "d:5", OP),
                 f"{name.lower()} {{d}}, {{m}}, {{n}}", f"  %r = icmp {swapped} {ty} %n, %m\n  ret i1 %r", ' "zext"="1" "nodecode"'))
        w(define("i1", name + "I" + sfx, f'{ty} "reg"="{c}" %n, {ty} "imm" %imm', I("imm:s12", "n:5", f3, "d:5", OPI),
                 f"{ {'SLT': 'slti', 'SLTU': 'sltiu'}[name]} {{d}}, {{n}}, {{imm}}", f"  %r = icmp {pred} {ty} %n, %imm\n  ret i1 %r",
                 ' "zext"="1"' + extra))
for c, ty, extra in [("x", "i64", ""), ("w", "i32", ' "nodecode"')]:
    sfx = "" if c == "x" else "w"
    w(define("i1", "SEQZ" + sfx, f'{ty} "reg"="{c}" %n', I("000000000001", "n:5", "011", "d:5", OPI),
             "seqz {d}, {n}", f"  %r = icmp eq {ty} %n, 0\n  ret i1 %r", ' "zext"="1"' + extra))
    w(define("i1", "SNEZ" + sfx, f'{ty} "reg"="{c}" %n', R("0000000", "n:5", "00000", "011", "d:5", OP),
             "snez {d}, {n}", f"  %r = icmp ne {ty} %n, 0\n  ret i1 %r", ' "zext"="1"' + extra))

# Moves and constants.
w(define("i64", "MV", 'i64 "reg"="x" %n', I("000000000000", "n:5", "000", "d:5", OPI), "mv {d}, {n}",
         "  ret i64 %n", ' "copy"="x"'))
w(define("i32", "MVw", 'i32 "reg"="w" %n', I("000000000000", "n:5", "000", "d:5", OPI), "mv {d}, {n}",
         "  ret i32 %n", ' "copy"="w" "nodecode"'))
w(define("i64", "LI", 'i64 "imm" %imm', I("imm:s12", "00000", "000", "d:5", OPI), "li {d}, {imm}", "  ret i64 %imm"))
w(define("i32", "LIw", 'i32 "imm" %imm', I("imm:s12", "00000", "000", "d:5", OPI), "li {d}, {imm}", "  ret i32 %imm",
         ' "nodecode"'))
w(define("i64", "LUI", 'i64 "imm" %imm', f"imm20:s20=imm>>12 d:5 {LUI}", "lui {d}, {=(imm>>12)&1048575}", "  ret i64 %imm"))
w(define("i32", "LUIw", 'i32 "imm" %imm', f"imm20:s20=imm>>12 d:5 {LUI}", "lui {d}, {=(imm>>12)&1048575}", "  ret i32 %imm",
         ' "nodecode"'))

# Zero extension of a byte or a boolean.
for ty, c in [("i64", "x"), ("i32", "w")]:
    w(define(ty, f"ZEXTB{c}", 'i8 "reg"="w" %n', I("000011111111", "n:5", "111", "d:5", OPI), "andi {d}, {n}, 255",
             f"  %r = zext i8 %n to {ty}\n  ret {ty} %r", ' "zext"="8" "nodecode"' + (' "role"="zext.8"' if c == "w" else "")))
    w(define(ty, f"ZEXT1{c}", 'i1 "reg"="w" %n', I("000000000001", "n:5", "111", "d:5", OPI), "andi {d}, {n}, 1",
             f"  %r = zext i1 %n to {ty}\n  ret {ty} %r", ' "zext"="1" "nodecode"' + (' "role"="zext.1"' if c == "w" else "")))

# The address of a global: its upper 20 bits, then the rest (medlow).
w('define ptr @LUIsym(ptr "sym" %s) "enc"="imm20:20=s>>12 d:5 0110111" "asm"="lui {d}, {s}" "reloc"="hi20" "role"="addr.hi" "nomatch" "nodecode" {\n  ret ptr %s\n}\n')
w('define ptr @ADDIsym(ptr "reg"="x" %n, ptr "sym" %s) "enc"="imm:12=s n:5 000 d:5 0010011" "asm"="addi {d}, {n}, {s}" "reloc"="lo12_i" "role"="addr.lo" "nomatch" "nodecode" {\n  ret ptr %n\n}\n')
w("; An address PC-relative: auipc's high part, then the low part, whose symbol\n; is the auipc's label (la, lla).\n")
w('define ptr @AUIPCpcrel(ptr "sym" %s) "enc"="imm20:20=s>>12 d:5 0010111" "asm"="auipc {d}, {s}" "reloc"="pcrel_hi20" "nomatch" "nodecode" {\n  ret ptr %s\n}\n')
w('define ptr @ADDIpcrel(ptr "reg"="x" %n, ptr "sym" %s) "enc"="imm:12=s n:5 000 d:5 0010011" "asm"="addi {d}, {n}, {s}" "reloc"="pcrel_lo12_i" "nomatch" "nodecode" {\n  ret ptr %n\n}\n')
w('; Loads and stores at a PC-relative address\'s low part, after its auipc\n; (`lw a0, sym` and `sw a0, sym, t0`).\ndefine void @LBpcrel(i64 "reg"="x" %rd, i64 "reg"="x" %n, ptr "sym" %s) "enc"="imm:12=s n:5 000 rd:5 0000011" "asm"="lb {rd}, {s}({n})" "reloc"="pcrel_lo12_i" "asmonly" "nodecode" {\n  ret void\n}\n\ndefine void @LHpcrel(i64 "reg"="x" %rd, i64 "reg"="x" %n, ptr "sym" %s) "enc"="imm:12=s n:5 001 rd:5 0000011" "asm"="lh {rd}, {s}({n})" "reloc"="pcrel_lo12_i" "asmonly" "nodecode" {\n  ret void\n}\n\ndefine void @LWpcrel(i64 "reg"="x" %rd, i64 "reg"="x" %n, ptr "sym" %s) "enc"="imm:12=s n:5 010 rd:5 0000011" "asm"="lw {rd}, {s}({n})" "reloc"="pcrel_lo12_i" "asmonly" "nodecode" {\n  ret void\n}\n\ndefine void @LDpcrel(i64 "reg"="x" %rd, i64 "reg"="x" %n, ptr "sym" %s) "enc"="imm:12=s n:5 011 rd:5 0000011" "asm"="ld {rd}, {s}({n})" "reloc"="pcrel_lo12_i" "asmonly" "nodecode" {\n  ret void\n}\n\ndefine void @LBUpcrel(i64 "reg"="x" %rd, i64 "reg"="x" %n, ptr "sym" %s) "enc"="imm:12=s n:5 100 rd:5 0000011" "asm"="lbu {rd}, {s}({n})" "reloc"="pcrel_lo12_i" "asmonly" "nodecode" {\n  ret void\n}\n\ndefine void @LHUpcrel(i64 "reg"="x" %rd, i64 "reg"="x" %n, ptr "sym" %s) "enc"="imm:12=s n:5 101 rd:5 0000011" "asm"="lhu {rd}, {s}({n})" "reloc"="pcrel_lo12_i" "asmonly" "nodecode" {\n  ret void\n}\n\ndefine void @LWUpcrel(i64 "reg"="x" %rd, i64 "reg"="x" %n, ptr "sym" %s) "enc"="imm:12=s n:5 110 rd:5 0000011" "asm"="lwu {rd}, {s}({n})" "reloc"="pcrel_lo12_i" "asmonly" "nodecode" {\n  ret void\n}\n\ndefine void @FLHpcrel(i64 "reg"="d" %rd, i64 "reg"="x" %n, ptr "sym" %s) "enc"="imm:12=s n:5 001 rd:5 0000111" "asm"="flh {rd}, {s}({n})" "reloc"="pcrel_lo12_i" "asmonly" "nodecode" {\n  ret void\n}\n\ndefine void @FLWpcrel(i64 "reg"="d" %rd, i64 "reg"="x" %n, ptr "sym" %s) "enc"="imm:12=s n:5 010 rd:5 0000111" "asm"="flw {rd}, {s}({n})" "reloc"="pcrel_lo12_i" "asmonly" "nodecode" {\n  ret void\n}\n\ndefine void @FLDpcrel(i64 "reg"="d" %rd, i64 "reg"="x" %n, ptr "sym" %s) "enc"="imm:12=s n:5 011 rd:5 0000111" "asm"="fld {rd}, {s}({n})" "reloc"="pcrel_lo12_i" "asmonly" "nodecode" {\n  ret void\n}\n\ndefine void @FLQpcrel(i64 "reg"="d" %rd, i64 "reg"="x" %n, ptr "sym" %s) "enc"="imm:12=s n:5 100 rd:5 0000111" "asm"="flq {rd}, {s}({n})" "reloc"="pcrel_lo12_i" "asmonly" "nodecode" {\n  ret void\n}\n\ndefine void @SBpcrel(i64 "reg"="x" %rs2, i64 "reg"="x" %n, ptr "sym" %s) "enc"="hi:7=s>>5 rs2:5 n:5 000 lo:5=s&31 0100011" "asm"="sb {rs2}, {s}({n})" "reloc"="pcrel_lo12_s" "asmonly" "nodecode" {\n  ret void\n}\n\ndefine void @SHpcrel(i64 "reg"="x" %rs2, i64 "reg"="x" %n, ptr "sym" %s) "enc"="hi:7=s>>5 rs2:5 n:5 001 lo:5=s&31 0100011" "asm"="sh {rs2}, {s}({n})" "reloc"="pcrel_lo12_s" "asmonly" "nodecode" {\n  ret void\n}\n\ndefine void @SWpcrel(i64 "reg"="x" %rs2, i64 "reg"="x" %n, ptr "sym" %s) "enc"="hi:7=s>>5 rs2:5 n:5 010 lo:5=s&31 0100011" "asm"="sw {rs2}, {s}({n})" "reloc"="pcrel_lo12_s" "asmonly" "nodecode" {\n  ret void\n}\n\ndefine void @SDpcrel(i64 "reg"="x" %rs2, i64 "reg"="x" %n, ptr "sym" %s) "enc"="hi:7=s>>5 rs2:5 n:5 011 lo:5=s&31 0100011" "asm"="sd {rs2}, {s}({n})" "reloc"="pcrel_lo12_s" "asmonly" "nodecode" {\n  ret void\n}\n\ndefine void @FSHpcrel(i64 "reg"="d" %rs2, i64 "reg"="x" %n, ptr "sym" %s) "enc"="hi:7=s>>5 rs2:5 n:5 001 lo:5=s&31 0100111" "asm"="fsh {rs2}, {s}({n})" "reloc"="pcrel_lo12_s" "asmonly" "nodecode" {\n  ret void\n}\n\ndefine void @FSWpcrel(i64 "reg"="d" %rs2, i64 "reg"="x" %n, ptr "sym" %s) "enc"="hi:7=s>>5 rs2:5 n:5 010 lo:5=s&31 0100111" "asm"="fsw {rs2}, {s}({n})" "reloc"="pcrel_lo12_s" "asmonly" "nodecode" {\n  ret void\n}\n\ndefine void @FSDpcrel(i64 "reg"="d" %rs2, i64 "reg"="x" %n, ptr "sym" %s) "enc"="hi:7=s>>5 rs2:5 n:5 011 lo:5=s&31 0100111" "asm"="fsd {rs2}, {s}({n})" "reloc"="pcrel_lo12_s" "asmonly" "nodecode" {\n  ret void\n}\n\ndefine void @FSQpcrel(i64 "reg"="d" %rs2, i64 "reg"="x" %n, ptr "sym" %s) "enc"="hi:7=s>>5 rs2:5 n:5 100 lo:5=s&31 0100111" "asm"="fsq {rs2}, {s}({n})" "reloc"="pcrel_lo12_s" "asmonly" "nodecode" {\n  ret void\n}\n\n')

# Loads and stores, at a 12-bit signed offset.
w("; Loads and stores, at a signed 12-bit offset from a register.\n")
for name, ty, f3, ext, role, z in [
    ("LBU", "i8", "100", None, "load.b", "8"), ("LHU", "i16", "101", None, "load.h", "16"),
    ("LW", "i32", "010", None, "load.w", None), ("LD", "i64", "011", None, "load.x", None),
]:
    attrs = f' "role"="{role}"' + (f' "zext"="{z}"' if z else "")
    w(define(ty, name, 'ptr "reg"="x" %n, i64 "imm" %off', I("off:s12", "n:5", f3, "d:5", LOAD),
             f"{name.lower()} {{d}}, {{off}}({{n}})",
             f"  %p = getelementptr i8, ptr %n, i64 %off\n  %v = load {ty}, ptr %p\n  ret {ty} %v", attrs))
w(define("ptr", "LDp", 'ptr "reg"="x" %n, i64 "imm" %off', I("off:s12", "n:5", "011", "d:5", LOAD),
         "ld {d}, {off}({n})", "  %p = getelementptr i8, ptr %n, i64 %off\n  %v = load ptr, ptr %p\n  ret ptr %v",
         ' "nodecode"'))
for name, src, f3 in [("LB", "i8", "000"), ("LH", "i16", "001"), ("LW", "i32", "010")]:
    for ty in (["i32", "i64"] if src != "i32" else ["i64"]):
        w(define(ty, f"{name}s{ty[1:]}", 'ptr "reg"="x" %n, i64 "imm" %off', I("off:s12", "n:5", f3, "d:5", LOAD),
                 f"{name.lower()} {{d}}, {{off}}({{n}})",
                 f"  %p = getelementptr i8, ptr %n, i64 %off\n  %v = load {src}, ptr %p\n  %r = sext {src} %v to {ty}\n  ret {ty} %r",
                 "" if (ty == "i64" and src != "i32") else ' "nodecode"'))
for name, src, f3 in [("LBU", "i8", "100"), ("LHU", "i16", "101"), ("LWU", "i32", "110")]:
    for ty in (["i32", "i64"] if src != "i32" else ["i64"]):
        w(define(ty, f"{name}z{ty[1:]}", 'ptr "reg"="x" %n, i64 "imm" %off', I("off:s12", "n:5", f3, "d:5", LOAD),
                 f"{name.lower()} {{d}}, {{off}}({{n}})",
                 f"  %p = getelementptr i8, ptr %n, i64 %off\n  %v = load {src}, ptr %p\n  %r = zext {src} %v to {ty}\n  ret {ty} %r",
                 ' "nodecode"' if name != "LWU" else ""))
for name, ty, f3, role, cls in [("SB", "i8", "000", "store.b", "w"), ("SH", "i16", "001", "store.h", "w"),
                                ("SW", "i32", "010", "store.w", "w"), ("SD", "i64", "011", "store.x", "x")]:
    w(define("void", name, f'{ty} "reg"="{cls}" %t, ptr "reg"="x" %n, i64 "imm" %off',
             S_IMM.format(rs2="t:5", rs1="n:5", f3=f3, op=STORE), f"{name.lower()} {{t}}, {{off}}({{n}})",
             f"  %p = getelementptr i8, ptr %n, i64 %off\n  store {ty} %t, ptr %p\n  ret void", f' "role"="{role}"'))
w(define("void", "SDp", 'ptr "reg"="x" %t, ptr "reg"="x" %n, i64 "imm" %off',
         S_IMM.format(rs2="t:5", rs1="n:5", f3="011", op=STORE), "sd {t}, {off}({n})",
         "  %p = getelementptr i8, ptr %n, i64 %off\n  store ptr %t, ptr %p\n  ret void", ' "nodecode"'))

# Branches: compare two registers, then branch.
w("; Branches, comparing two registers. The swapped forms (bgt is blt with\n; its operands swapped) are for the predicates RISC-V has no instruction for.\n")
BR_BODY = "  %c = icmp {pred} {ty} %n, %m\n  br i1 %c, label %taken, label %next\ntaken:\n  indirectbr ptr %dest, []\nnext:\n  ret void"
branches = [("BEQ", "eq", "000", "BNE", False), ("BNE", "ne", "001", "BEQ", False),
            ("BLT", "slt", "100", "BGE", False), ("BGE", "sge", "101", "BLT", False),
            ("BLTU", "ult", "110", "BGEU", False), ("BGEU", "uge", "111", "BLTU", False),
            ("BGT", "sgt", "100", "BLE", True), ("BLE", "sle", "101", "BGT", True),
            ("BGTU", "ugt", "110", "BLEU", True), ("BLEU", "ule", "111", "BGTU", True)]
for c, ty in [("x", "i64"), ("w", "i32"), ("p", "ptr")]:
    for name, pred, f3, inv, swapped in branches:
        sfx = "" if c == "x" else c
        cls = "w" if c == "w" else "x"
        if swapped:
            mnem = {"BGT": "blt", "BLE": "bge", "BGTU": "bltu", "BLEU": "bgeu"}[name]
            enc = B_IMM.format(rs2="rb:5=n", rs1="ra:5=m", f3=f3)
            asm = f"{mnem} {{m}}, {{n}}, {{dest}}"
        else:
            enc = B_IMM.format(rs2="m:5", rs1="n:5", f3=f3)
            asm = f"{name.lower()} {{n}}, {{m}}, {{dest}}"
        extra = f' "inverse"="{inv}{sfx}"' + ("" if (c == "x" and not swapped) else ' "nodecode"')
        if c == "p" and pred not in ("eq", "ne", "ult", "uge", "ugt", "ule"):
            continue
        w(define("void", name + sfx, f'{ty} "reg"="{cls}" %n, {ty} "reg"="{cls}" %m, ptr "label" %dest', enc, asm,
                 BR_BODY.format(pred=pred, ty=ty), extra))

# Jumps, calls and returns.
w(define("void", "J", 'ptr "label" %dest', J_IMM.format(rd="00000"), "j {dest}",
         "  indirectbr ptr %dest, []", ' "role"="jump"'))
w('define void @CALL(ptr "sym" %s) "enc"="j20:s1=s>>20 j10:10=(s>>1)&1023 j11:1=(s>>11)&1 j19:8=(s>>12)&255 00001 1101111" "asm"="jal ra, {s}" "reloc"="jal" "role"="call" "nomatch" {\n  ret void\n}\n')
# A tail call: the callee returns to this function's caller.
w('define void @TAIL(ptr "sym" %s) "enc"="j20:s1=s>>20 j10:10=(s>>1)&1023 j11:1=(s>>11)&1 j19:8=(s>>12)&255 00000 1101111" "asm"="jal zero, {s}" "reloc"="jal" "role"="tailcall" "nomatch" "nodecode" {\n  ret void\n}\n\n')
w('define void @CALLR(ptr "reg"="x" %n) "enc"="000000000000 n:5 000 00001 1100111" "asm"="jalr ra, 0({n})" "role"="callr" "nomatch" {\n  ret void\n}\n')
w('define void @RET() "enc"="00000000000000001000000001100111" "asm"="ret" "role"="ret" {\n  ret void\n}\n')
w('define void @EBREAK() "enc"="00000000000100000000000001110011" "asm"="ebreak" "role"="trap" {\n  unreachable\n}\n')
out("10-integer.ll", "\n".join(I_))

# ---------------------------------------------------------------------------
# 20-m.ll

M = [HEADER.format(what="the M extension, multiplication and division")]
for name, ir, f3, wname in [("MUL", "mul", "000", "MULW"), ("DIV", "sdiv", "100", "DIVW"), ("DIVU", "udiv", "101", "DIVUW"),
                            ("REM", "srem", "110", "REMW"), ("REMU", "urem", "111", "REMUW")]:
    M.append(define("i64", name, 'i64 "reg"="x" %n, i64 "reg"="x" %m', R("0000001", "m:5", "n:5", f3, "d:5", OP),
                    f"{name.lower()} {{d}}, {{n}}, {{m}}", f"  %r = {ir} i64 %n, %m\n  ret i64 %r", ' "feature"="m"'))
    M.append(define("i32", wname, 'i32 "reg"="w" %n, i32 "reg"="w" %m', R("0000001", "m:5", "n:5", f3, "d:5", OP32),
                    f"{wname.lower()} {{d}}, {{n}}, {{m}}", f"  %r = {ir} i32 %n, %m\n  ret i32 %r", ' "feature"="m"'))
for name, ext, f3 in [("MULH", "sext", "001"), ("MULHU", "zext", "011")]:
    M.append(define("i64", name, 'i64 "reg"="x" %n, i64 "reg"="x" %m', R("0000001", "m:5", "n:5", f3, "d:5", OP),
                    f"{name.lower()} {{d}}, {{n}}, {{m}}",
                    f"  %a = {ext} i64 %n to i128\n  %b = {ext} i64 %m to i128\n  %p = mul i128 %a, %b\n  %h = lshr i128 %p, 64\n  %r = trunc i128 %h to i64\n  ret i64 %r",
                    ' "feature"="m"'))
out("20-m.ll", "\n".join(M))

# ---------------------------------------------------------------------------
# 25-a.ll

A = [HEADER.format(what="the A extension, atomics, and fences") + """
; Each read-modify-write (amoadd and the rest) loads the old value into its
; result and writes the new one; .aq acquires, .rl releases, .aqrl does
; both and is sequentially consistent (as the psABI's mapping has it). The
; others (8 and 16 bits, compare-and-swap, nand and the rest) are loops of
; lr and sc (prepare.rs, atomics.rs), through
;
;   llvm.epitome.lr.iN(ptr, rank)        lr: the value, reserved
;   llvm.epitome.sc.iN(ptr, value, rank) sc: 0 if it was stored
;
; with rank 1 for .aq, 2 for .rl and 3 for .aqrl. Ordered loads and stores
; are plain ones with fences around them.
""" + declare("""
declare i32 @llvm.epitome.lr.i32(ptr, i32)
declare i64 @llvm.epitome.lr.i64(ptr, i32)
declare i32 @llvm.epitome.sc.i32(ptr, i32, i32)
declare i32 @llvm.epitome.sc.i64(ptr, i64, i32)
""")]
ORDERS = [("", "00", "monotonic"), (".aq", "10", "acquire"), (".rl", "01", "release"), (".aqrl", "11", "seq_cst")]
for name, f5, op in [("SWAP", "00001", "xchg"), ("ADD", "00000", "add"), ("XOR", "00100", "xor"), ("AND", "01100", "and"),
                     ("OR", "01000", "or"), ("MIN", "10000", "min"), ("MAX", "10100", "max"), ("MINU", "11000", "umin"),
                     ("MAXU", "11100", "umax")]:
    for c, ty, f3 in [("W", "i32", "010"), ("D", "i64", "011")]:
        cls = "w" if ty == "i32" else "x"
        for suffix, aqrl, order in ORDERS:
            A.append(define(ty, f"AMO{name}{c}{suffix.replace('.', '').upper()}", f'{ty} "reg"="{cls}" %m, ptr "reg"="x" %n',
                            f"{f5} {aqrl} m:5 n:5 {f3} d:5 0101111", f"amo{name.lower()}.{c.lower()}{suffix} {{d}}, {{m}}, ({{n}})",
                            f"  %r = atomicrmw {op} ptr %n, {ty} %m {order}\n  ret {ty} %r", ' "feature"="a"'))
for c, ty, f3 in [("W", "i32", "010"), ("D", "i64", "011")]:
    cls = "w" if ty == "i32" else "x"
    for suffix, aqrl, rank in [("", "00", 0), (".aq", "10", 1), (".aqrl", "11", 3)]:
        A.append(define(ty, f"LR{c}{suffix.replace('.', '').upper()}", 'ptr "reg"="x" %n',
                        f"00010 {aqrl} 00000 n:5 {f3} d:5 0101111", f"lr.{c.lower()}{suffix} {{d}}, ({{n}})",
                        f"  %r = call {ty} @llvm.epitome.lr.{ty}(ptr %n, i32 {rank})\n  ret {ty} %r", ' "feature"="a"'))
    for suffix, aqrl, rank in [("", "00", 0), (".rl", "01", 2)]:
        A.append(define("i32", f"SC{c}{suffix.replace('.', '').upper()}", f'{ty} "reg"="{cls}" %m, ptr "reg"="x" %n',
                        f"00011 {aqrl} m:5 n:5 {f3} d:5 0101111", f"sc.{c.lower()}{suffix} {{d}}, {{m}}, ({{n}})",
                        f"  %r = call i32 @llvm.epitome.sc.{ty}(ptr %n, {ty} %m, i32 {rank})\n  ret i32 %r", ' "feature"="a"'))
# Fences (RV64I): an acquire orders loads before it with everything after;
# the others order everything.
A.append(define("void", "FENCERRW", "", "0000 0010 0011 00000 000 00000 0001111", "fence r, rw", "  fence acquire\n  ret void"))
A.append(define("void", "FENCERWRW", "", "0000 0011 0011 00000 000 00000 0001111", "fence rw, rw", "  fence seq_cst\n  ret void"))
out("25-a.ll", "\n".join(A))

# ---------------------------------------------------------------------------
# 30-zbb.ll

Z = [HEADER.format(what="Zbb, basic bit manipulation (feature zbb)")]
for name, intr, f7, f3 in [("MAX", "smax", "0000101", "110"), ("MAXU", "umax", "0000101", "111"),
                           ("MIN", "smin", "0000101", "100"), ("MINU", "umin", "0000101", "101")]:
    for ty, c, extra in [("i64", "x", ""), ("i32", "w", ' "nodecode"')]:
        Z.append(declare(f"declare {ty} @llvm.{intr}.{ty}({ty}, {ty})\n"))
        Z.append(define(ty, name + ("" if c == "x" else "w"), f'{ty} "reg"="{c}" %n, {ty} "reg"="{c}" %m',
                        R(f7, "m:5", "n:5", f3, "d:5", OP), f"{name.lower()} {{d}}, {{n}}, {{m}}",
                        f"  %r = call {ty} @llvm.{intr}.{ty}({ty} %n, {ty} %m)\n  ret {ty} %r", ' "feature"="zbb"' + extra))
for name, ir, f3 in [("ANDN", "and", "111"), ("ORN", "or", "110")]:
    Z.append(define("i64", name, 'i64 "reg"="x" %n, i64 "reg"="x" %m', R("0100000", "m:5", "n:5", f3, "d:5", OP),
                    f"{name.lower()} {{d}}, {{n}}, {{m}}",
                    f"  %nm = xor i64 %m, -1\n  %r = {ir} i64 %n, %nm\n  ret i64 %r", ' "feature"="zbb"'))
for name, intr, rs2, op in [("CLZ", "ctlz", "00000", OPI), ("CTZ", "cttz", "00001", OPI), ("CPOP", "ctpop", "00010", OPI)]:
    args = "i64 %n, i1 false" if intr != "ctpop" else "i64 %n"
    decl = "i64, i1" if intr != "ctpop" else "i64"
    Z.append(declare(f"declare i64 @llvm.{intr}.i64({decl})\n"))
    Z.append(define("i64", name, 'i64 "reg"="x" %n', R("0110000", rs2, "n:5", "001", "d:5", op),
                    f"{name.lower()} {{d}}, {{n}}", f"  %r = call i64 @llvm.{intr}.i64({args})\n  ret i64 %r", ' "feature"="zbb"'))
for name, src, rs2 in [("SEXTB", "i8", "00100"), ("SEXTH", "i16", "00101")]:
    for ty, extra in [("i64", ""), ("i32", ' "nodecode"')]:
        Z.append(define(ty, name + ty[1:], f'{src} "reg"="w" %n', R("0110000", rs2, "n:5", "001", "d:5", OPI),
                        f"{name[:4].lower()}.{name[4].lower()} {{d}}, {{n}}", f"  %r = sext {src} %n to {ty}\n  ret {ty} %r",
                        ' "feature"="zbb"' + extra))
for ty, extra in [("i64", ""), ("i32", ' "nodecode"')]:
    Z.append(define(ty, "ZEXTH" + ty[1:], 'i16 "reg"="w" %n', R("0000100", "00000", "n:5", "100", "d:5", OP32),
                    "zext.h {d}, {n}", f"  %r = zext i16 %n to {ty}\n  ret {ty} %r", ' "feature"="zbb" "zext"="16"' + extra))
Z.append("declare i64 @llvm.bswap.i64(i64)\n")
Z.append(define("i64", "REV8", 'i64 "reg"="x" %n', I("011010111000", "n:5", "101", "d:5", OPI), "rev8 {d}, {n}",
                "  %r = call i64 @llvm.bswap.i64(i64 %n)\n  ret i64 %r", ' "feature"="zbb"'))
out("30-zbb.ll", "\n".join(Z))

# ---------------------------------------------------------------------------
# 40-float.ll

F = [HEADER.format(what="the F and D extensions, floating point")]
for t, ty, fmt, feat, cls in [("D", "double", "1", "d", "d"), ("S", "float", "0", "f", "s")]:
    fa = f' "feature"="{feat}"'
    for name, ir, f5 in [("FADD", "fadd", "00000"), ("FSUB", "fsub", "00001"), ("FMUL", "fmul", "00010"), ("FDIV", "fdiv", "00011")]:
        F.append(define(ty, f"{name}{t}", f'{ty} "reg"="{cls}" %n, {ty} "reg"="{cls}" %m',
                        R(f"{f5}0{fmt}", "m:5", "n:5", "111", "d:5", OPFP), f"{name.lower()}.{t.lower()} {{d}}, {{n}}, {{m}}",
                        f"  %r = {ir} {ty} %n, %m\n  ret {ty} %r", fa))
    sfx = "f64" if ty == "double" else "f32"
    F.append(declare(f"declare {ty} @llvm.sqrt.{sfx}({ty})\ndeclare {ty} @llvm.fabs.{sfx}({ty})\ndeclare {ty} @llvm.fmuladd.{sfx}({ty}, {ty}, {ty})\ndeclare {ty} @llvm.fma.{sfx}({ty}, {ty}, {ty})\n"))
    F.append(define(ty, f"FSQRT{t}", f'{ty} "reg"="{cls}" %n', R(f"01011" + f"0{fmt}", "00000", "n:5", "111", "d:5", OPFP),
                    f"fsqrt.{t.lower()} {{d}}, {{n}}", f"  %r = call {ty} @llvm.sqrt.{sfx}({ty} %n)\n  ret {ty} %r", fa))
    # Sign injection: fneg, fabs, and fmv (the copy) are fsgnjn, fsgnjx and fsgnj of a register with itself.
    F.append(define(ty, f"FNEG{t}", f'{ty} "reg"="{cls}" %n', R(f"00100" + f"0{fmt}", "rb:5=n", "n:5", "001", "d:5", OPFP),
                    f"fneg.{t.lower()} {{d}}, {{n}}", f"  %r = fneg {ty} %n\n  ret {ty} %r", fa))
    F.append(define(ty, f"FABS{t}", f'{ty} "reg"="{cls}" %n', R(f"00100" + f"0{fmt}", "rb:5=n", "n:5", "010", "d:5", OPFP),
                    f"fabs.{t.lower()} {{d}}, {{n}}", f"  %r = call {ty} @llvm.fabs.{sfx}({ty} %n)\n  ret {ty} %r", fa))
    F.append(define(ty, f"FMV{t}", f'{ty} "reg"="{cls}" %n', R(f"00100" + f"0{fmt}", "rb:5=n", "n:5", "000", "d:5", OPFP),
                    f"fmv.{t.lower()} {{d}}, {{n}}", f"  ret {ty} %n", fa + f' "copy"="{cls}"'))
    for intr in ["fmuladd", "fma"]:
        F.append(define(ty, f"FMADD{t}{'' if intr == 'fmuladd' else 'f'}", f'{ty} "reg"="{cls}" %n, {ty} "reg"="{cls}" %m, {ty} "reg"="{cls}" %a',
                        f"a:5 0{fmt} m:5 n:5 111 d:5 1000011", f"fmadd.{t.lower()} {{d}}, {{n}}, {{m}}, {{a}}",
                        f"  %r = call {ty} @llvm.{intr}.{sfx}({ty} %n, {ty} %m, {ty} %a)\n  ret {ty} %r",
                        fa + ("" if intr == "fmuladd" else ' "nodecode"')))
    # Comparisons: a boolean in an integer register.
    for name, pred, f3, swap in [("FEQ", "oeq", "010", False), ("FLT", "olt", "001", False), ("FLE", "ole", "000", False),
                                 ("FGT", "ogt", "001", True), ("FGE", "oge", "000", True)]:
        if swap:
            enc = R(f"10100" + f"0{fmt}", "rb:5=n", "ra:5=m", f3, "d:5", OPFP)
            asm = f"{ {'FGT': 'flt', 'FGE': 'fle'}[name]}.{t.lower()} {{d}}, {{m}}, {{n}}"
        else:
            enc = R(f"10100" + f"0{fmt}", "m:5", "n:5", f3, "d:5", OPFP)
            asm = f"{name.lower()}.{t.lower()} {{d}}, {{n}}, {{m}}"
        F.append(define("i1", f"{name}{t}", f'{ty} "reg"="{cls}" %n, {ty} "reg"="{cls}" %m', enc, asm,
                        f"  %r = fcmp {pred} {ty} %n, %m\n  ret i1 %r", fa + ' "zext"="1"' + (' "nodecode"' if swap else "")))
    # Conversions to and from integers: to them, rounding towards zero.
    for it, ic, rs2u, rs2s in [("i64", "x", "00011", "00010"), ("i32", "w", "00001", "00000")]:
        nm = "L" if it == "i64" else "W"
        F.append(define(it, f"FCVT{nm}{t}", f'{ty} "reg"="{cls}" %n', R(f"11000" + f"0{fmt}", rs2s, "n:5", "001", "d:5", OPFP),
                        f"fcvt.{nm.lower()}.{t.lower()} {{d}}, {{n}}, rtz", f"  %r = fptosi {ty} %n to {it}\n  ret {it} %r", fa))
        F.append(define(it, f"FCVT{nm}U{t}", f'{ty} "reg"="{cls}" %n', R(f"11000" + f"0{fmt}", rs2u, "n:5", "001", "d:5", OPFP),
                        f"fcvt.{nm.lower()}u.{t.lower()} {{d}}, {{n}}, rtz", f"  %r = fptoui {ty} %n to {it}\n  ret {it} %r", fa))
        # A 32-bit integer to a double is exact: its rounding mode is
        # written rne (000), as LLVM's assembler writes it; the others dyn.
        rm = "000" if t == "D" and nm == "W" else "111"
        F.append(define(ty, f"FCVT{t}{nm}", f'{it} "reg"="{ic}" %n', R(f"11010" + f"0{fmt}", rs2s, "n:5", rm, "d:5", OPFP),
                        f"fcvt.{t.lower()}.{nm.lower()} {{d}}, {{n}}", f"  %r = sitofp {it} %n to {ty}\n  ret {ty} %r", fa))
        F.append(define(ty, f"FCVT{t}{nm}U", f'{it} "reg"="{ic}" %n', R(f"11010" + f"0{fmt}", rs2u, "n:5", rm, "d:5", OPFP),
                        f"fcvt.{t.lower()}.{nm.lower()}u {{d}}, {{n}}", f"  %r = uitofp {it} %n to {ty}\n  ret {ty} %r", fa))
    # Moving bits between the register files.
    # fmv.x.d and fmv.d.x; for floats, fmv.x.w and fmv.w.x.
    it, ic, fx = ("i64", "x", "d") if t == "D" else ("i32", "w", "w")
    F.append(define(it, f"FMVX{t}", f'{ty} "reg"="{cls}" %n', R(f"11100" + f"0{fmt}", "00000", "n:5", "000", "d:5", OPFP),
                    f"fmv.x.{fx} {{d}}, {{n}}", f"  %r = bitcast {ty} %n to {it}\n  ret {it} %r",
                    fa + f' "role"="move.{ic}.{cls}"'))
    F.append(define(ty, f"FMV{t}X", f'{it} "reg"="{ic}" %n', R(f"11110" + f"0{fmt}", "00000", "n:5", "000", "d:5", OPFP),
                    f"fmv.{fx}.x {{d}}, {{n}}", f"  %r = bitcast {it} %n to {ty}\n  ret {ty} %r",
                    fa + f' "role"="move.{cls}.{ic}"'))
    # Loads and stores.
    f3m = "011" if t == "D" else "010"
    F.append(define(ty, f"FL{t}", 'ptr "reg"="x" %n, i64 "imm" %off', I("off:s12", "n:5", f3m, "d:5", LOADFP),
                    f"fl{'d' if t == 'D' else 'w'} {{d}}, {{off}}({{n}})",
                    f"  %p = getelementptr i8, ptr %n, i64 %off\n  %v = load {ty}, ptr %p\n  ret {ty} %v", fa + f' "role"="load.{cls}"'))
    F.append(define("void", f"FS{t}", f'{ty} "reg"="{cls}" %t, ptr "reg"="x" %n, i64 "imm" %off',
                    S_IMM.format(rs2="t:5", rs1="n:5", f3=f3m, op=STOREFP), f"fs{'d' if t == 'D' else 'w'} {{t}}, {{off}}({{n}})",
                    f"  %p = getelementptr i8, ptr %n, i64 %off\n  store {ty} %t, ptr %p\n  ret void", fa + f' "role"="store.{cls}"'))
F.append(define("double", "FCVTDS", 'float "reg"="s" %n', R("0100001", "00000", "n:5", "000", "d:5", OPFP),
                "fcvt.d.s {d}, {n}", "  %r = fpext float %n to double\n  ret double %r", ' "feature"="d"'))
F.append(define("float", "FCVTSD", 'double "reg"="d" %n', R("0100000", "00001", "n:5", "111", "d:5", OPFP),
                "fcvt.s.d {d}, {n}", "  %r = fptrunc double %n to float\n  ret float %r", ' "feature"="d"'))
out("40-float.ll", "\n".join(F))

# ---------------------------------------------------------------------------
# 60-recipes.ll

RC = [HEADER.format(what="recipes, for IR with no single instruction")]


def recipe(name, ret, params, body, expand_params, expand):
    RC.append(f'define {ret} @{name}({params}) "recipe"="{name}.expand" {{\n{body}\n}}\n')
    RC.append(f"define {ret} @{name}.expand({expand_params}) {{\n{expand}\n}}\n")


# Equality and the predicates with no instruction, from the ones there are.
for c, ty, sfx in [("x", "i64", ""), ("w", "i32", "w")]:
    for pred, inner in [("eq", "SEQZ"), ("ne", "SNEZ")]:
        xor = "XOR" if c == "x" else "XORw"
        recipe(f"icmp.{pred}.{c}", "i1", f'{ty} "reg"="{c}" %a, {ty} "reg"="{c}" %b',
               f"  %r = icmp {pred} {ty} %a, %b\n  ret i1 %r", f"{ty} %a, {ty} %b",
               f"  %x = call {ty} @{xor}({ty} %a, {ty} %b)\n  %r = call i1 @{inner}{sfx}({ty} %x)\n  ret i1 %r")
    for pred, inner in [("sge", "SLT"), ("uge", "SLTU"), ("sle", "SLTs"), ("ule", "SLTUs")]:
        recipe(f"icmp.{pred}.{c}", "i1", f'{ty} "reg"="{c}" %a, {ty} "reg"="{c}" %b',
               f"  %r = icmp {pred} {ty} %a, %b\n  ret i1 %r", f"{ty} %a, {ty} %b",
               f"  %l = call i1 @{inner}{sfx}({ty} %a, {ty} %b)\n  %r = call i1 @XORIb(i1 %l, i1 1)\n  ret i1 %r")

# A select on a boolean: b ^ ((a ^ b) & -c), with c's other bits cleared.
for c, ty in [("x", "i64"), ("w", "i32"), ("p", "ptr")]:
    cls = "w" if c == "w" else "x"
    RC.append(f'define {ty} @select.{c}(i1 "reg"="w" %c, {ty} "reg"="{cls}" %a, {ty} "reg"="{cls}" %b) "recipe"="select.{c}.expand" {{\n'
              f"  %r = select i1 %c, {ty} %a, {ty} %b\n  ret {ty} %r\n}}\n")
    RC.append(f"define {ty} @select.{c}.expand(i1 %c, {ty} %a, {ty} %b) {{\n"
              "  %c1 = call i64 @ZEXT1x(i1 %c)\n  %m = call i64 @NEG(i64 %c1)\n"
              f"  %ai = bitcast {ty} %a to i64\n  %bi = bitcast {ty} %b to i64\n"
              "  %x = call i64 @XOR(i64 %ai, i64 %bi)\n  %y = call i64 @AND(i64 %x, i64 %m)\n  %r = call i64 @XOR(i64 %bi, i64 %y)\n"
              f"  %rt = bitcast i64 %r to {ty}\n  ret {ty} %rt\n}}\n")
for t, ty, cls, it, ic in [("D", "double", "d", "i64", "x"), ("S", "float", "s", "i32", "w")]:
    RC.append(f'define {ty} @select.{cls}(i1 "reg"="w" %c, {ty} "reg"="{cls}" %a, {ty} "reg"="{cls}" %b) "recipe"="select.{cls}.expand" {{\n'
              f"  %r = select i1 %c, {ty} %a, {ty} %b\n  ret {ty} %r\n}}\n")
    RC.append(f"define {ty} @select.{cls}.expand(i1 %c, {ty} %a, {ty} %b) {{\n"
              f"  %ai = call {it} @FMVX{t}({ty} %a)\n  %bi = call {it} @FMVX{t}({ty} %b)\n"
              "  %c1 = call i64 @ZEXT1x(i1 %c)\n  %m = call i64 @NEG(i64 %c1)\n"
              f"  %a64 = bitcast {it} %ai to i64\n  %b64 = bitcast {it} %bi to i64\n"
              "  %x = call i64 @XOR(i64 %a64, i64 %b64)\n  %y = call i64 @AND(i64 %x, i64 %m)\n  %r = call i64 @XOR(i64 %b64, i64 %y)\n"
              f"  %rt = bitcast i64 %r to {it}\n  %f = call {ty} @FMV{t}X({it} %rt)\n  ret {ty} %f\n}}\n")

# Minimum and maximum, by a comparison and a select.
for intr, cmp, keep_a_if in [("smax", "SLT", "b"), ("smin", "SLTs", "b"), ("umax", "SLTU", "b"), ("umin", "SLTUs", "b")]:
    for c, ty, sfx in [("x", "i64", ""), ("w", "i32", "w")]:
        RC.append(declare(f"declare {ty} @llvm.{intr}.{ty}({ty}, {ty})\n"))
        xor, and_ = ("XOR", "AND") if c == "x" else ("XORw", "ANDw")
        neg = "NEG" if c == "x" else "NEGW"
        zext = "ZEXT1x" if c == "x" else "ZEXT1w"
        recipe(f"{intr}.{c}", ty, f'{ty} "reg"="{c}" %a, {ty} "reg"="{c}" %b',
               f"  %r = call {ty} @llvm.{intr}.{ty}({ty} %a, {ty} %b)\n  ret {ty} %r", f"{ty} %a, {ty} %b",
               f"  %l = call i1 @{cmp}{sfx}({ty} %a, {ty} %b)\n  %c1 = call {ty} @{zext}(i1 %l)\n  %m = call {ty} @{neg}({ty} %c1)\n"
               f"  %x = call {ty} @{xor}({ty} %a, {ty} %b)\n  %y = call {ty} @{and_}({ty} %x, {ty} %m)\n  %r = call {ty} @{xor}({ty} %a, {ty} %y)\n  ret {ty} %r")

# Extensions with no instruction: shifted up, then back down.
for src, bits in [("i1", 1), ("i8", 8), ("i16", 16), ("i32", 32)]:
    for ty in ["i64", "i32"]:
        if src == "i32" and ty == "i32":
            continue
        for ext, down in [("sext", "SRAI"), ("zext", "SRLI")]:
            if ext == "zext" and src in ("i1", "i8"):
                continue
            if ext == "sext" and src == "i32":
                continue
            sh = 64 - bits
            recipe(f"{ext}.{src}.{ty}", ty, f'{src} "reg"="{"x" if src == "i32" else "w"}" %n',
                   f"  %r = {ext} {src} %n to {ty}\n  ret {ty} %r", f"{src} %n",
                   f"  %w = zext {src} %n to i64\n  %u = call i64 @SLLI(i64 %w, i64 {sh})\n  %d = call i64 @{down}(i64 %u, i64 {sh})\n"
                   f"  %r = trunc i64 %d to {ty}\n  ret {ty} %r")

# Absolute value: (n ^ s) - s, with s the sign spread across.
for c, ty, srai, xor, sub in [("x", "i64", "SRAI", "XOR", "SUB"), ("w", "i32", "SRAIW", "XORw", "SUBW")]:
    RC.append(declare(f"declare {ty} @llvm.abs.{ty}({ty}, i1)\n"))
    recipe(f"abs.{c}", ty, f'{ty} "reg"="{c}" %n', f"  %r = call {ty} @llvm.abs.{ty}({ty} %n, i1 false)\n  ret {ty} %r",
           f"{ty} %n", f"  %s = call {ty} @{srai}({ty} %n, {ty} {int(ty[1:]) - 1})\n  %x = call {ty} @{xor}({ty} %n, {ty} %s)\n"
           f"  %r = call {ty} @{sub}({ty} %x, {ty} %s)\n  ret {ty} %r")

# Floating-point comparisons with no instruction: unordered ones are the
# ordered opposite, negated; ordered and unordered tests compare each
# value with itself.
for t, ty, cls in [("D", "double", "d"), ("S", "float", "s")]:
    for pred, inner, swap in [("une", "FEQ", False), ("uge", "FLT", False), ("ugt", "FLE", False),
                              ("ule", "FLT", True), ("ult", "FLE", True)]:
        a, b = ("%b", "%a") if swap else ("%a", "%b")
        recipe(f"fcmp.{pred}.{cls}", "i1", f'{ty} "reg"="{cls}" %a, {ty} "reg"="{cls}" %b',
               f"  %r = fcmp {pred} {ty} %a, %b\n  ret i1 %r", f"{ty} %a, {ty} %b",
               f"  %l = call i1 @{inner}{t}({ty} {a}, {ty} {b})\n  %r = call i1 @XORIb(i1 %l, i1 1)\n  ret i1 %r")
    recipe(f"fcmp.ord.{cls}", "i1", f'{ty} "reg"="{cls}" %a, {ty} "reg"="{cls}" %b',
           f"  %r = fcmp ord {ty} %a, %b\n  ret i1 %r", f"{ty} %a, {ty} %b",
           f"  %x = call i1 @FEQ{t}({ty} %a, {ty} %a)\n  %y = call i1 @FEQ{t}({ty} %b, {ty} %b)\n  %r = call i1 @ANDb(i1 %x, i1 %y)\n  ret i1 %r")
    recipe(f"fcmp.uno.{cls}", "i1", f'{ty} "reg"="{cls}" %a, {ty} "reg"="{cls}" %b',
           f"  %r = fcmp uno {ty} %a, %b\n  ret i1 %r", f"{ty} %a, {ty} %b",
           f"  %x = call i1 @FEQ{t}({ty} %a, {ty} %a)\n  %y = call i1 @FEQ{t}({ty} %b, {ty} %b)\n  %z = call i1 @ANDb(i1 %x, i1 %y)\n  %r = call i1 @XORIb(i1 %z, i1 1)\n  ret i1 %r")
    recipe(f"fcmp.one.{cls}", "i1", f'{ty} "reg"="{cls}" %a, {ty} "reg"="{cls}" %b',
           f"  %r = fcmp one {ty} %a, %b\n  ret i1 %r", f"{ty} %a, {ty} %b",
           f"  %x = call i1 @FLT{t}({ty} %a, {ty} %b)\n  %y = call i1 @FLT{t}({ty} %b, {ty} %a)\n  %r = call i1 @ORb(i1 %x, i1 %y)\n  ret i1 %r")
    recipe(f"fcmp.ueq.{cls}", "i1", f'{ty} "reg"="{cls}" %a, {ty} "reg"="{cls}" %b',
           f"  %r = fcmp ueq {ty} %a, %b\n  ret i1 %r", f"{ty} %a, {ty} %b",
           f"  %x = call i1 @FLT{t}({ty} %a, {ty} %b)\n  %y = call i1 @FLT{t}({ty} %b, {ty} %a)\n  %z = call i1 @ORb(i1 %x, i1 %y)\n  %r = call i1 @XORIb(i1 %z, i1 1)\n  ret i1 %r")
out("60-recipes.ll", "\n".join(RC))

# ---------------------------------------------------------------------------
# 70-abi.ll

out("70-abi.ll", HEADER.format(what="the LP64D calling convention") + """
; The RISC-V ELF psABI's LP64D: integer arguments in a0-a7, floating-point
; ones in fa0-fa7, stack arguments in 8-byte slots, and variadic arguments
; where named ones would go but in the integer registers, floating-point
; ones too. s0 is the frame pointer; gp and tp are the platform's. t5 and
; t6 (and ft9-ft11) are kept for spill code, and t4 for the addresses of
; stack slots too far for an offset ("far-scratch").

!abi.lp64d = !{!100, !101, !102, !103, !104, !105, !106, !107, !108, !109, !110, !111, !112, !113, !114, !115, !116, !117, !118}
!100 = !{!"args", !"a0-a7"}
!101 = !{!"result", !"a0"}
!102 = !{!"clobbers", !"ra", !"t0-t2", !"a0-a7", !"t3-t6"}
!103 = !{!"saved", !"s1", !"s2-s11"}
!104 = !{!"reserved", !"gp", !"tp"}
!105 = !{!"scratch", !"t5-t6"}
!106 = !{!"frame", !"s0", !"ra"}
!107 = !{!"stack-args", !"slots"}
!108 = !{!"variadic", !"integer"}
!109 = !{!"syntax", !"elf"}
!110 = !{!"triples", !"riscv64", !"linux", !"elf", !"none"}
!111 = !{!"fp-args", !"fa0-fa7"}
!112 = !{!"fp-result", !"fa0"}
!113 = !{!"fp-clobbers", !"ft0-ft7", !"fa0-fa7", !"ft8-ft11"}
!114 = !{!"fp-saved", !"fs0-fs1", !"fs2-fs11"}
!115 = !{!"fp-scratch", !"ft9-ft11"}
!116 = !{!"far-scratch", !"t4"}
!117 = !{!"base", !"s1"}
!118 = !{!"stack-align", !"16"}
""")

for name, parts in files.items():
    with open(os.path.join(HERE, name), "w") as f:
        f.write("\n".join(parts).rstrip("\n") + "\n")
