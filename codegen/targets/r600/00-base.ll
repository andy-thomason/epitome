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
!target.asm-aliases = !{!1300, !1301, !1302, !1303}
!1300 = !{!"ASHR ", !"ASHR_INT "}
!1301 = !{!"LSHR ", !"LSHR_INT "}
!1302 = !{!"LSHL ", !"LSHL_INT "}
!1303 = !{!"MULHI ", !"MULHI_UINT "}
