; The RISC-V RV64 description: the files, and the machine as a whole. Written by gen_riscv64.py; the
; conventions are those of codegen/targets/aarch64/00-base.ll.

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

!target.relocs = !{!1200, !1201, !1202, !1203, !1210, !1211, !1212, !1213, !1214, !1215, !1216, !1217, !1218, !1219, !1220, !1221, !1222, !1223, !1224}
!1200 = !{!"name", !"jal", !"elf", !"17", !"elf-name", !"R_RISCV_JAL", !"use", !"call", !"target", !"pc + v"}
!1201 = !{!"name", !"hi20", !"elf", !"26", !"elf-name", !"R_RISCV_HI20", !"elf-syntax", !"%hi({s})", !"target", !"v << 12"}
!1202 = !{!"name", !"lo12_i", !"elf", !"27", !"elf-name", !"R_RISCV_LO12_I", !"elf-syntax", !"%lo({s})"}
!1203 = !{!"name", !"abs64", !"elf", !"2", !"elf-name", !"R_RISCV_64", !"use", !"data8"}
!1210 = !{!"name", !"none", !"elf", !"0", !"elf-name", !"R_RISCV_NONE"}
!1211 = !{!"name", !"abs32", !"elf", !"1", !"elf-name", !"R_RISCV_32"}
; How .eh_frame says where a function is.
!1219 = !{!"name", !"pcrel32", !"elf", !"57", !"elf-name", !"R_RISCV_32_PCREL", !"use", !"eh-pcrel"}
; A thread-local variable's offset from the thread pointer, in a program
; (the local-exec model, as the rest of this code is a program's): its
; high part, the add of tp that marks the sequence, and its low part.
!1220 = !{!"name", !"tprel_hi20", !"elf", !"29", !"elf-name", !"R_RISCV_TPREL_HI20", !"elf-syntax", !"%tprel_hi({s})", !"use", !"tls", !"target", !"v << 12"}
!1221 = !{!"name", !"tprel_add", !"elf", !"32", !"elf-name", !"R_RISCV_TPREL_ADD", !"elf-syntax", !"%tprel_add({s})", !"use", !"tls"}
!1222 = !{!"name", !"tprel_lo12_i", !"elf", !"30", !"elf-name", !"R_RISCV_TPREL_LO12_I", !"elf-syntax", !"%tprel_lo({s})", !"use", !"tls"}
!1223 = !{!"name", !"tprel_lo12_s", !"elf", !"31", !"elf-name", !"R_RISCV_TPREL_LO12_S"}

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
