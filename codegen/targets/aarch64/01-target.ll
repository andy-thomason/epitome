; The AArch64 description: the machine as a whole. The conventions are in
; 00-base.ll, and the forms of these lists in codegen/src/machine.rs.

; ===========================================================================
; Registers: for each class, the names of its registers in number order.
; Number 31 is the zero register in w and x, the stack pointer in wsp and
; xsp ("sp"="yes"). There are two files: the integer registers, and the
; floating-point and vector registers (v0-v31), which d, s, q and v name as
; 64-bit, 32-bit and 128-bit values and 64-bit vectors.

!target.registers = !{!1000, !1001, !1002, !1003, !1004, !1005, !1006, !1007, !1008, !1009, !1010, !1011, !1012, !1013, !1014, !1015, !1016}
!1000 = !{!"w", !"w{0-30}", !"wzr"}
!1001 = !{!"x", !"x{0-30}", !"xzr"}
!1002 = !{!"wsp", !"w{0-30}", !"wsp"}
!1003 = !{!"xsp", !"x{0-30}", !"sp"}
!1004 = !{!"d", !"d{0-31}"}
!1005 = !{!"s", !"s{0-31}"}
!1006 = !{!"q", !"q{0-31}"}
!1007 = !{!"v", !"d{0-31}"}
; Views of the vector registers for templates: {n:b} is b3, {n:4s} is v3.4s.
!1008 = !{!"b", !"b{0-31}"}
!1009 = !{!"h", !"h{0-31}"}
!1010 = !{!"lanes", !"v{0-31}"}
; SVE's: its scalable vectors z0-z31 (at least 128 bits, a multiple of
; them), whose low 128 bits are v0-v31, and a
; third file, the predicates p0-p15; pg is the eight a three-bit field
; holds (an instruction's governing predicate), pn the predicates as
; counters (SVE2.1's and SME2's pn8-pn15).
!1011 = !{!"z", !"z{0-31}"}
!1012 = !{!"p", !"p{0-15}"}
!1013 = !{!"pg", !"p{0-7}"}
!1014 = !{!"pn", !"pn{0-15}"}
; SME's choosers of ZA's slices: ws, w12-w15 (a tile's), and wv, w8-w11
; (SME2's vectors of the array), each in a two-bit field.
!1015 = !{!"ws", !"-{0-11}", !"w{12-15}"}
!1016 = !{!"wv", !"-{0-7}", !"w{8-11}"}

; What each class holds, and in which file. The first of a kind and width
; is where values of it are put; b, h and lanes only name registers.
!target.classes = !{!1020, !1021, !1022, !1023, !1024, !1025, !1026, !1027, !1028, !1029, !1030, !1031, !1032, !1033, !1034, !1035, !1036}
!1020 = !{!"w", !"file", !"r", !"bits", !"32", !"holds", !"int"}
!1021 = !{!"x", !"file", !"r", !"bits", !"64", !"holds", !"int"}
!1022 = !{!"wsp", !"file", !"r", !"bits", !"32", !"holds", !"int", !"sp", !"yes"}
!1023 = !{!"xsp", !"file", !"r", !"bits", !"64", !"holds", !"int", !"sp", !"yes"}
!1024 = !{!"d", !"file", !"v", !"bits", !"64", !"holds", !"float"}
!1025 = !{!"s", !"file", !"v", !"bits", !"32", !"holds", !"float"}
!1026 = !{!"q", !"file", !"v", !"bits", !"128", !"holds", !"vector"}
!1027 = !{!"v", !"file", !"v", !"bits", !"64", !"holds", !"vector"}
!1028 = !{!"b", !"file", !"v"}
!1029 = !{!"h", !"file", !"v"}
!1030 = !{!"lanes", !"file", !"v"}
!1031 = !{!"z", !"file", !"v", !"bits", !"128", !"holds", !"scalable"}
!1032 = !{!"p", !"file", !"p", !"bits", !"16", !"holds", !"predicate"}
!1033 = !{!"pg", !"file", !"p", !"bits", !"16", !"holds", !"predicate"}
!1034 = !{!"pn", !"file", !"p"}
!1035 = !{!"ws", !"file", !"r", !"bits", !"32", !"holds", !"int"}
!1036 = !{!"wv", !"file", !"r", !"bits", !"32", !"holds", !"int"}

; ===========================================================================
; Parameters

!target.params = !{!1100, !1101, !1102, !1103, !1104, !1105, !1106, !1107, !1108, !1109, !1110, !1111, !1112, !1113, !1114}
!1100 = !{!"sp", !"sp"}
!1101 = !{!"zero", !"xzr"}
!1102 = !{!"vector-bits", !"128", !"neon"}
!1103 = !{!"narrow-results", !"zero"}
!1104 = !{!"external-via-got", !"yes"}
!1105 = !{!"elf-machine", !"183"}
!1106 = !{!"elf-flags", !"0"}
!1107 = !{!"macho-cpu", !"0x0100000c"}
!1108 = !{!"cpu", !"generic"}
!1109 = !{!"elf-format", !"elf64-littleaarch64"}
!1110 = !{!"asm-comment", !"//"}
; The data alignment factor of unwind tables, in ELF and in Mach-O, as
; their assemblers write it.
!1111 = !{!"cfi-data-align", !"-4", !"-8"}
; Loads and stores of normal memory need no alignment, at no cost.
!1112 = !{!"unaligned-access", !"fast"}
; The flags a comparison sets and a condition reads, unless an entry says
; otherwise ("sets", "reads").
!1113 = !{!"flags", !"nzcv"}
; Atomics without FEAT_LSE: libgcc's and compiler-rt's outline helpers.
!1114 = !{!"atomic-helpers", !"__aarch64_{op}{bytes}_{order}"}

; ===========================================================================
; Inline assembly: what its constraint codes and template modifiers mean,
; from the LLVM Language Reference. A register operand without a modifier
; is written as its whole register: x3, or v3 for w.

!target.asm-constraints = !{!1300, !1301, !1302, !1303, !1304, !1305, !1306}
!1300 = !{!"r", !"file", !"r", !"print", !"x"}
!1301 = !{!"w", !"file", !"v", !"print", !"lanes"}
!1302 = !{!"I", !"imm", !"0", !"4095"}
!1303 = !{!"J", !"imm", !"-4095", !"0"}
!1304 = !{!"Z", !"zero"}
!1305 = !{!"m", !"mem", !"[{r}]"}
!1306 = !{!"Q", !"mem", !"[{r}]"}

; Operands as people write them, and as the templates do: an immediate
; without its #, and an address without its offset.
!target.asm-aliases = !{!1320, !1321}
!1320 = !{!", {i}", !", #{i}"}
!1321 = !{!"[{r}]", !"[{r}, #0]"}

!target.asm-modifiers = !{!1310, !1311, !1312, !1313, !1314, !1315, !1316}
!1310 = !{!"w", !"class", !"w"}
!1311 = !{!"x", !"class", !"x"}
!1312 = !{!"b", !"class", !"b"}
!1313 = !{!"h", !"class", !"h"}
!1314 = !{!"s", !"class", !"s"}
!1315 = !{!"d", !"class", !"d"}
!1316 = !{!"q", !"class", !"q"}

; ===========================================================================
; CPUs and their features, by LLVM's names for them (which clang's IR uses
; in "target-cpu" and "target-features"). Only those the description uses
; matter: neon (Advanced SIMD; the vector entries need it), fp-armv8, lse
; (atomics), and crc, aes, sha2 and sha3 (45-crypto.ll).

!target.cpus = !{!1150, !1151, !1152}
!1150 = !{!"generic", !"fp-armv8", !"neon"}
!1151 = !{!"apple-m1", !"fp-armv8", !"neon", !"lse", !"crc", !"aes", !"sha2", !"sha3"}
; sme: the Scalable Matrix Extension (45-sme.ll), with 512-bit streaming
; vectors on M4.
!1152 = !{!"apple-m4", !"fp-armv8", !"neon", !"lse", !"crc", !"aes", !"sha2", !"sha3", !"sme"}

; ===========================================================================
; Relocations: those instructions name in "reloc" (and data's pointers),
; with how each object format numbers them and assembly writes them; then
; others, by number and name only, for reading other compilers' objects.
; From Arm's ELF for the Arm 64-bit Architecture, and Apple's
; <mach-o/arm64/reloc.h>.

!target.relocs = !{!1200, !1201, !1202, !1203, !1204, !1205, !1210, !1211, !1212, !1213, !1214, !1215, !1216, !1217, !1218, !1219, !1220, !1221, !1222, !1223, !1224, !1225, !1226, !1227, !1228, !1229, !1230, !1231, !1232, !1233, !1234, !1235}
!1200 = !{!"name", !"call26", !"elf", !"283", !"elf-name", !"R_AARCH64_CALL26", !"macho", !"2", !"macho-name", !"ARM64_RELOC_BRANCH26", !"macho-pcrel", !"yes", !"macho-length", !"2", !"use", !"call", !"target", !"pc + v"}
!1201 = !{!"name", !"adrp", !"elf", !"275", !"elf-name", !"R_AARCH64_ADR_PREL_PG_HI21", !"macho", !"3", !"macho-name", !"ARM64_RELOC_PAGE21", !"macho-pcrel", !"yes", !"macho-length", !"2", !"elf-syntax", !"{s}", !"macho-syntax", !"{s}@PAGE", !"target", !"(pc & -4096) + (v << 12)"}
!1202 = !{!"name", !"add_lo12", !"elf", !"277", !"elf-name", !"R_AARCH64_ADD_ABS_LO12_NC", !"macho", !"4", !"macho-name", !"ARM64_RELOC_PAGEOFF12", !"macho-length", !"2", !"elf-syntax", !":lo12:{s}", !"macho-syntax", !"{s}@PAGEOFF"}
!1203 = !{!"name", !"adrp_got", !"elf", !"311", !"elf-name", !"R_AARCH64_ADR_GOT_PAGE", !"macho", !"5", !"macho-name", !"ARM64_RELOC_GOT_LOAD_PAGE21", !"macho-pcrel", !"yes", !"macho-length", !"2", !"elf-syntax", !":got:{s}", !"macho-syntax", !"{s}@GOTPAGE", !"use", !"got", !"target", !"(pc & -4096) + (v << 12)"}
!1204 = !{!"name", !"ld64_got_lo12", !"elf", !"312", !"elf-name", !"R_AARCH64_LD64_GOT_LO12_NC", !"macho", !"6", !"macho-name", !"ARM64_RELOC_GOT_LOAD_PAGEOFF12", !"macho-length", !"2", !"elf-syntax", !":got_lo12:{s}", !"macho-syntax", !"{s}@GOTPAGEOFF", !"use", !"got"}
!1205 = !{!"name", !"abs64", !"elf", !"257", !"elf-name", !"R_AARCH64_ABS64", !"macho", !"0", !"macho-name", !"ARM64_RELOC_UNSIGNED", !"macho-length", !"3", !"use", !"data8"}
!1210 = !{!"name", !"none", !"elf", !"0", !"elf-name", !"R_AARCH64_NONE"}
!1211 = !{!"name", !"abs32", !"elf", !"258", !"elf-name", !"R_AARCH64_ABS32"}
!1212 = !{!"name", !"abs16", !"elf", !"259", !"elf-name", !"R_AARCH64_ABS16"}
!1213 = !{!"name", !"prel64", !"elf", !"260", !"elf-name", !"R_AARCH64_PREL64"}
!1214 = !{!"name", !"prel32", !"elf", !"261", !"elf-name", !"R_AARCH64_PREL32", !"use", !"eh-pcrel"}
!1215 = !{!"name", !"prel16", !"elf", !"262", !"elf-name", !"R_AARCH64_PREL16"}
!1216 = !{!"name", !"adr_prel_lo21", !"elf", !"274", !"elf-name", !"R_AARCH64_ADR_PREL_LO21"}
!1217 = !{!"name", !"adr_prel_pg_hi21_nc", !"elf", !"276", !"elf-name", !"R_AARCH64_ADR_PREL_PG_HI21_NC"}
!1218 = !{!"name", !"ldst8_lo12", !"elf", !"278", !"elf-name", !"R_AARCH64_LDST8_ABS_LO12_NC"}
!1219 = !{!"name", !"tstbr14", !"elf", !"279", !"elf-name", !"R_AARCH64_TSTBR14"}
!1220 = !{!"name", !"condbr19", !"elf", !"280", !"elf-name", !"R_AARCH64_CONDBR19"}
!1221 = !{!"name", !"jump26", !"elf", !"282", !"elf-name", !"R_AARCH64_JUMP26", !"macho", !"2", !"macho-name", !"ARM64_RELOC_BRANCH26", !"macho-pcrel", !"yes", !"macho-length", !"2", !"use", !"call", !"target", !"pc + v"}
!1222 = !{!"name", !"ldst16_lo12", !"elf", !"284", !"elf-name", !"R_AARCH64_LDST16_ABS_LO12_NC"}
!1223 = !{!"name", !"ldst32_lo12", !"elf", !"285", !"elf-name", !"R_AARCH64_LDST32_ABS_LO12_NC"}
!1224 = !{!"name", !"ldst64_lo12", !"elf", !"286", !"elf-name", !"R_AARCH64_LDST64_ABS_LO12_NC"}
!1225 = !{!"name", !"ldst128_lo12", !"elf", !"299", !"elf-name", !"R_AARCH64_LDST128_ABS_LO12_NC"}
!1226 = !{!"name", !"macho_subtractor", !"macho", !"1", !"macho-name", !"ARM64_RELOC_SUBTRACTOR", !"macho-length", !"3", !"use", !"eh-sub"}
!1227 = !{!"name", !"macho_pointer_to_got", !"macho", !"7", !"macho-name", !"ARM64_RELOC_POINTER_TO_GOT", !"macho-pcrel", !"yes", !"macho-length", !"2", !"use", !"got-pcrel32"}
!1228 = !{!"name", !"macho_addend", !"macho", !"10", !"macho-name", !"ARM64_RELOC_ADDEND"}
; With macho_subtractor before it, the distance from one symbol to another:
; how __eh_frame says where a function is.
!1229 = !{!"name", !"macho_unsigned64", !"macho", !"0", !"macho-name", !"ARM64_RELOC_UNSIGNED", !"macho-length", !"3", !"use", !"eh-unsigned"}
; A thread-local variable's descriptor (Mach-O).
!1230 = !{!"name", !"adrp_tlv", !"macho", !"8", !"macho-name", !"ARM64_RELOC_TLVP_LOAD_PAGE21", !"macho-pcrel", !"yes", !"macho-length", !"2", !"macho-syntax", !"{s}@TLVPPAGE", !"use", !"got", !"target", !"(pc & -4096) + (v << 12)"}
!1231 = !{!"name", !"ld64_tlv_lo12", !"macho", !"9", !"macho-name", !"ARM64_RELOC_TLVP_LOAD_PAGEOFF12", !"macho-length", !"2", !"macho-syntax", !"{s}@TLVPPAGEOFF", !"use", !"got"}
; A thread-local variable's descriptor (ELF's TLSDESC): its page, the
; resolver in its first word, its address, and the call of the resolver,
; which returns the variable's offset from the thread pointer. A linker
; may rewrite the four in place, so they are always the same sequence.
!1232 = !{!"name", !"tlsdesc_adr_page21", !"elf", !"562", !"elf-name", !"R_AARCH64_TLSDESC_ADR_PAGE21", !"elf-syntax", !":tlsdesc:{s}", !"use", !"tls", !"target", !"(pc & -4096) + (v << 12)"}
!1233 = !{!"name", !"tlsdesc_ld64_lo12", !"elf", !"563", !"elf-name", !"R_AARCH64_TLSDESC_LD64_LO12", !"elf-syntax", !":tlsdesc_lo12:{s}", !"use", !"tls"}
!1234 = !{!"name", !"tlsdesc_add_lo12", !"elf", !"564", !"elf-name", !"R_AARCH64_TLSDESC_ADD_LO12", !"elf-syntax", !":tlsdesc_lo12:{s}", !"use", !"tls"}
!1235 = !{!"name", !"tlsdesc_call", !"elf", !"569", !"elf-name", !"R_AARCH64_TLSDESC_CALL", !"use", !"tls"}

; ===========================================================================
; DWARF's numbers for the registers, in unwind tables: each file's from
; this. (x0-x30 are 0-30, sp 31; v0-v31 are 64-95.)

!target.dwarf = !{!1400, !1401}
!1400 = !{!"r", !"0"}
!1401 = !{!"v", !"64"}
