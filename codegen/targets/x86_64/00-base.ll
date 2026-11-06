; The x86-64 description: the files, and the machine as a whole. The
; conventions are those of codegen/targets/aarch64/00-base.ll; the plan is
; docs/design/x86.md.
;
; The description is these files, read in name order as one module:
;
;   00-base.ll      registers, parameters, relocations, and the encoding
;                   fragments (REX, ModRM, SIB, displacements)
;   02-xed.ll       every instruction llvm-mc 18 assembles, for assembly and
;                   disassembly ("asmonly"): integer, x87, MMX, SSE to 4.2,
;                   AVX, AVX2, FMA, AVX-512 with masks, broadcasts and
;                   rounding, AMX, and the rest; written by gen_xed.py from
;                   Intel XED's tables. First, so that the assembler prefers
;                   its entries, which encode as llvm-mc does
;   05-cond.ll      the flags, and the conditions that read them
;   10-integer.ll   integer instructions for selection, with bodies;
;                   written by gen_sel.py
;   15-frame.ll     what frames are made with: pushes, pops, rsp's
;                   adjustments
;   16-short.ll     short forms peepholes use in place of longer ones
;   70-abi.ll       the System V calling convention
;
; Assembly is AT&T's, as clang, llvm-mc and objdump write it on Linux.
; The fragments here follow the Intel SDM and the AMD APM; the entries are
; XED's, and every one is checked against llvm-mc (tests/x86.rs). Not yet:
; selection and the calling convention.

; ===========================================================================
; Registers. The integer file, r, by hardware number: rax, rcx, rdx, rbx,
; rsp, rbp, rsi, rdi, r8-r15, seen as 64, 32, 16 and 8 bits. The byte
; registers are those REX gives (spl, bpl, sil, dil), never ah-bh.

!target.registers = !{!1000, !1001, !1002, !1003, !1004, !1005, !1006, !1007, !1008, !1009, !1060, !1040, !1041, !1042, !1043, !1044, !1045, !1046, !1047, !1048, !1049, !1030, !1031, !1034, !1035, !1038, !1062}
!1000 = !{!"r64", !"%rax", !"%rcx", !"%rdx", !"%rbx", !"%rsp", !"%rbp", !"%rsi", !"%rdi", !"%r{8-15}"}
!1001 = !{!"r32", !"%eax", !"%ecx", !"%edx", !"%ebx", !"%esp", !"%ebp", !"%esi", !"%edi", !"%r{8-15}d"}
!1002 = !{!"r16", !"%ax", !"%cx", !"%dx", !"%bx", !"%sp", !"%bp", !"%si", !"%di", !"%r{8-15}w"}
!1003 = !{!"r8", !"%al", !"%cl", !"%dl", !"%bl", !"%spl", !"%bpl", !"%sil", !"%dil", !"%r{8-15}b"}
; Scalar floating point in the vector registers: doubles (d) and floats (s).
!1030 = !{!"d", !"%xmm{0-15}"}
!1031 = !{!"s", !"%xmm{0-15}"}
; Vectors for selection (SSE2): 128 bits (xv), and 64 (xh, in a register's
; low half, as LLVM widens them).
!1034 = !{!"xv", !"%xmm{0-15}"}
!1035 = !{!"xh", !"%xmm{0-15}"}
; And 256 bits (yv), with avx2.
!1038 = !{!"yv", !"%ymm{0-15}"}
; And 512 bits (zv), all 32 registers, with avx512f.
!1060 = !{!"zv", !"%zmm{0-31}"}
; The vector file, v: 16 registers with SSE and VEX (x, y), all 32 with
; EVEX (xe, ye, z). The mask file, k.
!1004 = !{!"x", !"%xmm{0-15}"}
!1005 = !{!"y", !"%ymm{0-15}"}
!1006 = !{!"xe", !"%xmm{0-31}"}
!1007 = !{!"ye", !"%ymm{0-31}"}
!1008 = !{!"z", !"%zmm{0-31}"}
!1009 = !{!"k", !"%k{0-7}"}
; Masks of 512-bit vectors (kw): k1-k7, as an instruction's mask cannot be k0.
!1062 = !{!"kw", !"-", !"%k{1-7}"}
; For assembly: MMX, the x87 stack, the segment, control and debug
; registers, and AMX's tiles.
!1040 = !{!"mm", !"%mm{0-7}"}
!1041 = !{!"st", !"%st({0-7})"}
!1042 = !{!"seg", !"%es", !"%cs", !"%ss", !"%ds", !"%fs", !"%gs"}
!1043 = !{!"cr", !"%cr{0-15}"}
!1044 = !{!"dr", !"%dr{0-15}"}
!1045 = !{!"tmm", !"%tmm{0-7}"}
; The registers some instructions take fixed: a division's rax and rdx, a
; shift's count in rcx (cl).
!1046 = !{!"rax", !"%rax", !"%rcx", !"%rdx", !"%rbx", !"%rsp", !"%rbp", !"%rsi", !"%rdi", !"%r{8-15}"}
!1047 = !{!"rdx", !"%rax", !"%rcx", !"%rdx", !"%rbx", !"%rsp", !"%rbp", !"%rsi", !"%rdi", !"%r{8-15}"}
!1048 = !{!"rcx", !"%rax", !"%rcx", !"%rdx", !"%rbx", !"%rsp", !"%rbp", !"%rsi", !"%rdi", !"%r{8-15}"}
; And xmm0, which SSE4.1's blends and SHA's rounds take unnamed.
!1049 = !{!"xmm0", !"%xmm{0-15}"}

!target.classes = !{!1010, !1011, !1012, !1013, !1014, !1015, !1016, !1017, !1018, !1019, !1050, !1051, !1052, !1053, !1054, !1055, !1056, !1057, !1058, !1032, !1033, !1036, !1037, !1039, !1061, !1063, !1059}
!1010 = !{!"r64", !"file", !"r", !"bits", !"64", !"holds", !"int"}
!1011 = !{!"r32", !"file", !"r", !"bits", !"32", !"holds", !"int"}
!1012 = !{!"r16", !"file", !"r"}
!1013 = !{!"r8", !"file", !"r"}
!1032 = !{!"d", !"file", !"v", !"bits", !"64", !"holds", !"float"}
!1033 = !{!"s", !"file", !"v", !"bits", !"32", !"holds", !"float"}
!1036 = !{!"xv", !"file", !"v", !"bits", !"128", !"holds", !"vector"}
!1037 = !{!"xh", !"file", !"v", !"bits", !"64", !"holds", !"vector"}
!1039 = !{!"yv", !"file", !"v", !"bits", !"256", !"holds", !"vector"}
!1061 = !{!"zv", !"file", !"v", !"bits", !"512", !"holds", !"vector"}
!1014 = !{!"x", !"file", !"v", !"bits", !"128"}
!1015 = !{!"y", !"file", !"v", !"bits", !"256"}
!1016 = !{!"xe", !"file", !"v", !"bits", !"128"}
!1017 = !{!"ye", !"file", !"v", !"bits", !"256"}
!1018 = !{!"z", !"file", !"v", !"bits", !"512"}
!1019 = !{!"k", !"file", !"k", !"bits", !"64", !"holds", !"int"}
!1063 = !{!"kw", !"file", !"k", !"bits", !"64"}
!1050 = !{!"mm", !"file", !"mm"}
!1051 = !{!"st", !"file", !"st"}
!1052 = !{!"seg", !"file", !"seg"}
!1053 = !{!"cr", !"file", !"cr"}
!1054 = !{!"dr", !"file", !"dr"}
!1055 = !{!"tmm", !"file", !"tmm"}
!1056 = !{!"rax", !"file", !"r", !"fixed", !"%rax"}
!1057 = !{!"rdx", !"file", !"r", !"fixed", !"%rdx"}
!1058 = !{!"rcx", !"file", !"r", !"fixed", !"%rcx"}
!1059 = !{!"xmm0", !"file", !"v", !"fixed", !"%xmm0"}

; ===========================================================================
; Parameters.

!target.params = !{!1100, !1101, !1102, !1103, !1104, !1105, !1106, !1107, !1108, !1109, !1110, !1111, !1112, !1113, !1114, !1115, !1116, !1117, !1118, !1119, !1120, !1121, !1122, !1123, !1124}
!1100 = !{!"sp", !"%rsp"}
!1101 = !{!"vector-bits", !"128"}
; 256 bits with AVX2; masks in the registers of the vectors in use.
!1118 = !{!"vector-bits", !"256", !"avx2"}
; 512 with AVX-512 (avx512f).
!1122 = !{!"vector-bits", !"512", !"avx512f"}
; Unaligned loads and stores cost what aligned ones do (within a line).
!1123 = !{!"unaligned-access", !"fast"}
!1119 = !{!"mask-class", !"128", !"xv"}
!1120 = !{!"mask-class", !"256", !"yv"}
; With avx512f, masks of 512-bit vectors in k registers (isel.rs says which
; are; the others stay in yv).
!1124 = !{!"mask-class", !"512", !"kw"}
; 64-bit vectors in xmm registers (their low halves) at any width.
!1121 = !{!"min-vector-bits", !"64"}
!1102 = !{!"elf-machine", !"62"}
!1103 = !{!"elf-format", !"elf64-x86-64"}
!1104 = !{!"asm-comment", !"#"}
!1105 = !{!"cpu", !"x86-64"}
!1106 = !{!"cfi-data-align", !"-8"}
; Symbols another module may define (not dso_local: -fPIC's, and what is
; only declared) are reached through the GOT, which a linker relaxes in a
; program.
!1107 = !{!"external-via-got", !"yes"}
; Instructions are 1 to 15 bytes; the entries are many, so their encodings
; are read when first used.
!1108 = !{!"instruction-unit", !"1"}
!1109 = !{!"lazy-encodings", !"1"}
; A 32-bit result clears the top half; branches compare whole registers.
!1110 = !{!"narrow-results", !"zero"}
!1111 = !{!"boolean-branches", !"compare"}
!1112 = !{!"flags", !"zcso"}
; Unwind tables: the CFA is rsp + 8 on entry, the return address at CFA - 8,
; in DWARF's column 16 (rip).
!1113 = !{!"cfi-cfa-offset", !"8"}
!1114 = !{!"cfi-return-column", !"16"}
; Floating-point comparisons no one condition tests: made of two.
!1115 = !{!"fcmp-split", !"oeq", !"une"}
; Loads of any ordering are plain moves; a sequentially consistent store is
; an xchg.
!1116 = !{!"seq-cst-stores", !"swap"}
; Inline assembly's immediates, as AT&T writes them.
!1117 = !{!"asm-imm", !"${i}"}

; ===========================================================================
; Inline assembly: what its constraint codes and template modifiers mean,
; from the LLVM Language Reference and GCC's x86 constraints. A register
; operand without a modifier is written as its value's register: %eax for
; an i32, %rax for an i64.

!target.asm-constraints = !{!1170, !1171, !1172, !1173, !1174, !1175, !1176, !1177, !1178, !1179, !1180, !1181, !1182, !1183}
!1170 = !{!"r", !"file", !"r"}
!1171 = !{!"q", !"file", !"r"}
!1172 = !{!"R", !"file", !"r"}
!1173 = !{!"l", !"file", !"r"}
!1174 = !{!"x", !"file", !"v"}
!1175 = !{!"v", !"file", !"v"}
!1176 = !{!"m", !"mem", !"({r})"}
!1177 = !{!"e", !"imm", !"-2147483648", !"2147483647"}
!1178 = !{!"Z", !"imm", !"0", !"4294967295"}
!1179 = !{!"I", !"imm", !"0", !"31"}
!1180 = !{!"J", !"imm", !"0", !"63"}
!1181 = !{!"K", !"imm", !"-128", !"127"}
!1182 = !{!"N", !"imm", !"0", !"255"}
!1183 = !{!"M", !"imm", !"0", !"3"}

; Operands as people write them, and as the templates do: memory without a
; displacement or an index's scale (with what comes before the operand, so
; that 8(%rax) is left alone), and conditions by their other names.
!target.asm-aliases = !{!1600, !1601, !1602, !1603, !1604, !1605, !1606, !1607, !1608, !1609, !1610, !1611, !1612, !1613, !1614, !1615, !1616, !1617, !1618, !1619, !1620, !1621, !1622, !1623, !1624, !1625, !1626, !1627, !1628, !1629, !1630, !1631, !1632, !1633, !1634, !1635, !1636, !1637, !1638, !1639, !1640, !1641, !1642, !1643, !1644, !1645, !1646, !1647, !1648, !1649, !1650, !1651, !1652, !1653, !1654, !1655, !1656, !1657, !1658, !1659, !1660, !1661, !1662, !1663, !1664, !1665, !1666, !1667, !1668, !1669, !1670, !1671, !1672, !1673, !1674, !1675, !1676, !1677}
!1600 = !{!" (%{r})", !" 0(%{r})"}
!1601 = !{!",(%{r})", !",0(%{r})"}
!1602 = !{!" (%{r},%{x})", !" 0(%{r},%{x},1)"}
!1603 = !{!",(%{r},%{x})", !",0(%{r},%{x},1)"}
!1604 = !{!" (%{r},%{x},{i})", !" 0(%{r},%{x},{i})"}
!1605 = !{!",(%{r},%{x},{i})", !",0(%{r},%{x},{i})"}
!1606 = !{!" {i}(%{r},%{x})", !" {i}(%{r},%{x},1)"}
!1607 = !{!",{i}(%{r},%{x})", !",{i}(%{r},%{x},1)"}
!1608 = !{!"jz ", !"je "}
!1609 = !{!"jnz ", !"jne "}
!1610 = !{!"jc ", !"jb "}
!1611 = !{!"jnc ", !"jae "}
!1612 = !{!"jnae ", !"jb "}
!1613 = !{!"jnb ", !"jae "}
!1614 = !{!"jna ", !"jbe "}
!1615 = !{!"jnbe ", !"ja "}
!1616 = !{!"jnge ", !"jl "}
!1617 = !{!"jnl ", !"jge "}
!1618 = !{!"jng ", !"jle "}
!1619 = !{!"jnle ", !"jg "}
!1620 = !{!"jpe ", !"jp "}
!1621 = !{!"jpo ", !"jnp "}
!1622 = !{!"setz ", !"sete "}
!1623 = !{!"setnz ", !"setne "}
!1624 = !{!"setc ", !"setb "}
!1625 = !{!"setnc ", !"setae "}
!1626 = !{!"setnae ", !"setb "}
!1627 = !{!"setnb ", !"setae "}
!1628 = !{!"setna ", !"setbe "}
!1629 = !{!"setnbe ", !"seta "}
!1630 = !{!"setnge ", !"setl "}
!1631 = !{!"setnl ", !"setge "}
!1632 = !{!"setng ", !"setle "}
!1633 = !{!"setnle ", !"setg "}
!1634 = !{!"setpe ", !"setp "}
!1635 = !{!"setpo ", !"setnp "}
!1636 = !{!"cmovzw ", !"cmovew "}
!1637 = !{!"cmovzl ", !"cmovel "}
!1638 = !{!"cmovzq ", !"cmoveq "}
!1639 = !{!"cmovnzw ", !"cmovnew "}
!1640 = !{!"cmovnzl ", !"cmovnel "}
!1641 = !{!"cmovnzq ", !"cmovneq "}
!1642 = !{!"cmovcw ", !"cmovbw "}
!1643 = !{!"cmovcl ", !"cmovbl "}
!1644 = !{!"cmovcq ", !"cmovbq "}
!1645 = !{!"cmovncw ", !"cmovaew "}
!1646 = !{!"cmovncl ", !"cmovael "}
!1647 = !{!"cmovncq ", !"cmovaeq "}
!1648 = !{!"cmovnaew ", !"cmovbw "}
!1649 = !{!"cmovnael ", !"cmovbl "}
!1650 = !{!"cmovnaeq ", !"cmovbq "}
!1651 = !{!"cmovnbw ", !"cmovaew "}
!1652 = !{!"cmovnbl ", !"cmovael "}
!1653 = !{!"cmovnbq ", !"cmovaeq "}
!1654 = !{!"cmovnaw ", !"cmovbew "}
!1655 = !{!"cmovnal ", !"cmovbel "}
!1656 = !{!"cmovnaq ", !"cmovbeq "}
!1657 = !{!"cmovnbew ", !"cmovaw "}
!1658 = !{!"cmovnbel ", !"cmoval "}
!1659 = !{!"cmovnbeq ", !"cmovaq "}
!1660 = !{!"cmovngew ", !"cmovlw "}
!1661 = !{!"cmovngel ", !"cmovll "}
!1662 = !{!"cmovngeq ", !"cmovlq "}
!1663 = !{!"cmovnlw ", !"cmovgew "}
!1664 = !{!"cmovnll ", !"cmovgel "}
!1665 = !{!"cmovnlq ", !"cmovgeq "}
!1666 = !{!"cmovngw ", !"cmovlew "}
!1667 = !{!"cmovngl ", !"cmovlel "}
!1668 = !{!"cmovngq ", !"cmovleq "}
!1669 = !{!"cmovnlew ", !"cmovgw "}
!1670 = !{!"cmovnlel ", !"cmovgl "}
!1671 = !{!"cmovnleq ", !"cmovgq "}
!1672 = !{!"cmovpew ", !"cmovpw "}
!1673 = !{!"cmovpel ", !"cmovpl "}
!1674 = !{!"cmovpeq ", !"cmovpq "}
!1675 = !{!"cmovpow ", !"cmovnpw "}
!1676 = !{!"cmovpol ", !"cmovnpl "}
!1677 = !{!"cmovpoq ", !"cmovnpq "}

!target.asm-modifiers = !{!1185, !1186, !1187, !1188, !1189, !1191, !1192}
!1185 = !{!"k", !"class", !"r32"}
!1186 = !{!"q", !"class", !"r64"}
!1187 = !{!"w", !"class", !"r16"}
!1188 = !{!"b", !"class", !"r8"}
!1189 = !{!"x", !"class", !"x"}
!1191 = !{!"t", !"class", !"y"}
!1192 = !{!"g", !"class", !"z"}

!target.cpus = !{!1150}
; x86-64 itself has SSE and SSE2, by LLVM's names (which rustc and clang write
; in "target-features", with those they imply).
!1150 = !{!"x86-64", !"sse", !"sse2"}

; No-ops to pad code with, by length, as llvm-mc pads: nop, and nopw and nopl
; of a memory operand with prefixes.
!target.nops = !{!1160}
!1160 = !{!"90", !"6690", !"0f1f00", !"0f1f4000", !"0f1f440000", !"660f1f440000", !"0f1f8000000000", !"0f1f840000000000", !"660f1f840000000000", !"662e0f1f840000000000", !"66662e0f1f840000000000", !"6666662e0f1f840000000000", !"666666662e0f1f840000000000", !"66666666662e0f1f840000000000", !"6666666666662e0f1f840000000000"}

; ===========================================================================
; Relocations, from the System V x86-64 psABI. A field measures a symbol
; from its instruction's start; a PC-relative relocation measures from the
; field, so its addend is what the field's expression adds (the
; instruction's end subtracted) plus the field's place in the instruction.

!target.relocs = !{!1200, !1201, !1202, !1203, !1204, !1205, !1206}
!1200 = !{!"name", !"abs64", !"elf", !"1", !"elf-name", !"R_X86_64_64", !"use", !"data8"}
!1201 = !{!"name", !"pc32", !"elf", !"2", !"elf-name", !"R_X86_64_PC32", !"use", !"eh-pcrel", !"target", !"pc + v"}
!1202 = !{!"name", !"plt32", !"elf", !"4", !"elf-name", !"R_X86_64_PLT32", !"use", !"call", !"target", !"pc + v"}
; (A mov's: relaxable, as assemblers write it for one with REX.)
!1203 = !{!"name", !"gotpcrel", !"elf", !"42", !"elf-name", !"R_X86_64_REX_GOTPCRELX", !"elf-syntax", !"{s}@GOTPCREL", !"use", !"got", !"target", !"pc + v"}
!1204 = !{!"name", !"abs32s", !"elf", !"11", !"elf-name", !"R_X86_64_32S"}
!1205 = !{!"name", !"pc32x", !"elf", !"2", !"elf-name", !"R_X86_64_PC32", !"target", !"pc + v"}
; A thread-local variable's offset from the thread pointer, in the GOT
; (initial exec).
!1206 = !{!"name", !"gottpoff", !"elf", !"22", !"elf-name", !"R_X86_64_GOTTPOFF", !"elf-syntax", !"{s}@GOTTPOFF", !"use", !"tls", !"target", !"pc + v"}

; DWARF's numbers for the registers, by hardware number: DWARF orders them
; rax, rdx, rcx, rbx, rsi, rdi, rbp, rsp, then r8-r15; xmm0-xmm31 from 17.
!target.dwarf = !{!1400, !1401}
!1400 = !{!"r", !"0", !"2", !"1", !"3", !"7", !"6", !"4", !"5", !"8", !"9", !"10", !"11", !"12", !"13", !"14", !"15"}
!1401 = !{!"v", !"17"}

; ===========================================================================
; Encodings. An instruction is a list of bytes (pieces, written most
; significant bit first); these fragments are the parts most share.
;
; rex(w, r, x, b): the REX prefix, 0100WRXB, where W is a 64-bit operand
; and R, X and B the fourth bits of ModRM.reg, SIB.index and ModRM.rm (or
; SIB.base). Left out when nothing needs it.
;
; rexm(w, r, base, index): REX for a memory operand: a RIP-relative one
; (base 16) has no base or index bits.
;
; rr(r, b): ModRM for two registers, mod 11.
;
; rm(r, base, index, scale, disp, sym, n): ModRM, SIB and displacement for
; a memory operand [base + index*scale + disp]: base 0-15, or 16 for RIP
; (then sym + disp, from the instruction's end); index 4 (rsp) for none. A
; byte displacement counts units of n bytes (EVEX's disp8*N; else 1).
; The shortest form that holds it is the one used:
;   - RIP-relative: mod 00, rm 101, disp32;
;   - no index: mod 00 (not rbp or r13, which mean a displacement; not rsp
;     or r12, which mean a SIB byte), mod 01 with disp8, mod 10 with disp32;
;   - with a SIB byte (an index, or a base of rsp or r12): the same three.

!target.encodings = !{!1500, !1501, !1502, !1503, !1504, !1505, !1506, !1507, !1508}
!1500 = !{!"rex", !"w r x b", !"{ if(w==0) if(r<8) if(x<8) if(b<8) ; 0100 W:1=w R:1=r>>3 X:1=x>>3 B:1=b>>3 }"}
!1501 = !{!"rexm", !"w r base index", !"{ if(base==16) rex(w, r, 0, 0) ; rex(w, r, index, base) }"}
!1502 = !{!"rr", !"r b", !"11 reg:3=r&7 rm:3=b&7"}
!1503 = !{!"rm", !"r base index scale disp sym n", !"{ rip:0=base-16 i:0=index-4 s:0=scale-1 00 reg:3=r&7 101 | disp:s32=sym+disp-end ; if((base&7)!=4) if((base&7)!=5) i:0=index-4 s:0=scale-1 z:0=disp 00 reg:3=r&7 rm:3=base&7 ; if((base&7)!=4) i:0=index-4 s:0=scale-1 01 reg:3=r&7 rm:3=base&7 | disp:s8=disp/n ; if((base&7)!=4) i:0=index-4 s:0=scale-1 10 reg:3=r&7 rm:3=base&7 | disp:s32=disp ; if((base&7)!=5) z:0=disp 00 reg:3=r&7 100 | ss:2=log2(scale) idx:3=index&7 bs:3=base&7 ; 01 reg:3=r&7 100 | ss:2=log2(scale) idx:3=index&7 bs:3=base&7 | disp:s8=disp/n ; 10 reg:3=r&7 100 | ss:2=log2(scale) idx:3=index&7 bs:3=base&7 | disp:s32=disp }"}

; vex(r, x, b, map, w, v, l, pp): the VEX prefix. Two bytes (C5) when it
; can be: map 0F, W 0, no X or B; else three (C4). R, X, B and vvvv are
; stored inverted. map: 1 0F, 2 0F38, 3 0F3A; pp: 0 none, 1 66, 2 F3, 3 F2;
; l: 0 128 bits, 1 256.
;
; vexm(r, base, index, map, w, v, l, pp): VEX for a memory operand.
;
; evex(r, x, b, map, w, v, ll, pp, bc, aaa, z): the EVEX prefix (62), for
; a memory operand's index and base registers, x and b; evexr(r, b, ...)
; for a register b (0-31: its fifth bit is in X). R' and V' are the fifth
; bits of ModRM.reg and vvvv, inverted as the others are; ll: 0 128 bits,
; 1 256, 2 512 (with bc, of registers: the rounding); bc broadcast, or of
; registers, rounding or {sae}; aaa the mask register; z zeroing.
!1504 = !{!"vex", !"r x b map w v l pp", !"{ if(x<8) if(b<8) m1:0=map-1 w0:0=w 11000101 | R:1=(r>>3)^1 vv:4=v^15 L:1=l pp:2=pp ; 11000100 | R:1=(r>>3)^1 X:1=(x>>3)^1 B:1=(b>>3)^1 mm:5=map | W:1=w vv:4=v^15 L:1=l pp:2=pp }"}
!1505 = !{!"vexm", !"r base index map w v l pp", !"{ if(base==16) vex(r, 0, 0, map, w, v, l, pp) ; vex(r, index, base, map, w, v, l, pp) }"}
!1506 = !{!"evex", !"r x b map w v ll pp bc aaa z", !"01100010 | R:1=((r>>3)&1)^1 X:1=((x>>3)&1)^1 B:1=((b>>3)&1)^1 R1:1=(r>>4)^1 0 mm:3=map | W:1=w vv:4=(v&15)^15 1 pp:2=pp | Z:1=z LL:2=ll bc:1=bc V1:1=(v>>4)^1 aaa:3=aaa"}
!1507 = !{!"evexr", !"r b map w v ll pp bc aaa z", !"01100010 | R:1=((r>>3)&1)^1 X:1=(b>>4)^1 B:1=((b>>3)&1)^1 R1:1=(r>>4)^1 0 mm:3=map | W:1=w vv:4=(v&15)^15 1 pp:2=pp | Z:1=z LL:2=ll bc:1=bc V1:1=(v>>4)^1 aaa:3=aaa"}
!1508 = !{!"evexm", !"r base index map w v ll pp bc aaa z", !"{ if(base==16) evex(r, 0, 0, map, w, v, ll, pp, bc, aaa, z) ; evex(r, index, base, map, w, v, ll, pp, bc, aaa, z) }"}
