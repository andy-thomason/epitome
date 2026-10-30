; Entries for assembly and disassembly only ("asmonly"), written by
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

define void @AUIPC.0(i64 "reg"="x" %rd, i64 "imm" %imm20) "enc"="imm20:20 rd:5 0010111" "asm"="auipc {rd}, {imm20}" "asmonly" {
  ret void
}

define void @JAL.0(i64 "reg"="x" %rd, ptr "label" %dest) "enc"="o_20:s1=dest>>20 o_10:10=(dest>>1)&1023 o_11:1=(dest>>11)&1 o_19:8=(dest>>12)&255 rd:5 1101111" "asm"="jal {rd}, {dest}" "asmonly" {
  ret void
}

define void @JALR.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1, i64 "imm" %imm12) "enc"="imm12:s12 rs1:5 000 rd:5 1100111" "asm"="jalr {rd}, {imm12}({rs1})" "asmonly" {
  ret void
}

define void @FENCE.0() "enc"="00001111111100000000000000001111" "asm"="fence iorw, iorw" "asmonly" {
  ret void
}

define void @FENCE.1() "enc"="00001111001100000000000000001111" "asm"="fence iorw, rw" "asmonly" {
  ret void
}

define void @FENCE.2() "enc"="00001111001000000000000000001111" "asm"="fence iorw, r" "asmonly" {
  ret void
}

define void @FENCE.3() "enc"="00001111000100000000000000001111" "asm"="fence iorw, w" "asmonly" {
  ret void
}

define void @FENCE.4() "enc"="00000011111100000000000000001111" "asm"="fence rw, iorw" "asmonly" {
  ret void
}

define void @FENCE.5() "enc"="00000011001000000000000000001111" "asm"="fence rw, r" "asmonly" {
  ret void
}

define void @FENCE.6() "enc"="00000011000100000000000000001111" "asm"="fence rw, w" "asmonly" {
  ret void
}

define void @FENCE.7() "enc"="00000010111100000000000000001111" "asm"="fence r, iorw" "asmonly" {
  ret void
}

define void @FENCE.8() "enc"="00000010001000000000000000001111" "asm"="fence r, r" "asmonly" {
  ret void
}

define void @FENCE.9() "enc"="00000010000100000000000000001111" "asm"="fence r, w" "asmonly" {
  ret void
}

define void @FENCE.10() "enc"="00000001111100000000000000001111" "asm"="fence w, iorw" "asmonly" {
  ret void
}

define void @FENCE.11() "enc"="00000001001100000000000000001111" "asm"="fence w, rw" "asmonly" {
  ret void
}

define void @FENCE.12() "enc"="00000001001000000000000000001111" "asm"="fence w, r" "asmonly" {
  ret void
}

define void @FENCE.13() "enc"="00000001000100000000000000001111" "asm"="fence w, w" "asmonly" {
  ret void
}

define void @ECALL.0() "enc"="00000000000000000000000001110011" "asm"="ecall" "asmonly" {
  ret void
}

define void @MULHSU.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1, i64 "reg"="x" %rs2) "enc"="0000001 rs2:5 rs1:5 010 rd:5 0110011" "asm"="mulhsu {rd}, {rs1}, {rs2}" "asmonly" {
  ret void
}

define void @LR_W_RL.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1) "enc"="000100100000 rs1:5 010 rd:5 0101111" "asm"="lr.w.rl {rd}, ({rs1})" "asmonly" {
  ret void
}

define void @SC_W_AQ.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1, i64 "reg"="x" %rs2) "enc"="0001110 rs2:5 rs1:5 010 rd:5 0101111" "asm"="sc.w.aq {rd}, {rs2}, ({rs1})" "asmonly" {
  ret void
}

define void @SC_W_AQRL.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1, i64 "reg"="x" %rs2) "enc"="0001111 rs2:5 rs1:5 010 rd:5 0101111" "asm"="sc.w.aqrl {rd}, {rs2}, ({rs1})" "asmonly" {
  ret void
}

define void @LR_D_RL.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1) "enc"="000100100000 rs1:5 011 rd:5 0101111" "asm"="lr.d.rl {rd}, ({rs1})" "asmonly" {
  ret void
}

define void @SC_D_AQ.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1, i64 "reg"="x" %rs2) "enc"="0001110 rs2:5 rs1:5 011 rd:5 0101111" "asm"="sc.d.aq {rd}, {rs2}, ({rs1})" "asmonly" {
  ret void
}

define void @SC_D_AQRL.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1, i64 "reg"="x" %rs2) "enc"="0001111 rs2:5 rs1:5 011 rd:5 0101111" "asm"="sc.d.aqrl {rd}, {rs2}, ({rs1})" "asmonly" {
  ret void
}

define void @FMADD_S.0(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2, i64 "reg"="d" %rs3) "enc"="rs3:5 00 rs2:5 rs1:5 000 rd:5 1000011" "asm"="fmadd.s {rd}, {rs1}, {rs2}, {rs3}, rne" "asmonly" {
  ret void
}

define void @FMADD_S.1(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2, i64 "reg"="d" %rs3) "enc"="rs3:5 00 rs2:5 rs1:5 001 rd:5 1000011" "asm"="fmadd.s {rd}, {rs1}, {rs2}, {rs3}, rtz" "asmonly" {
  ret void
}

define void @FMADD_S.2(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2, i64 "reg"="d" %rs3) "enc"="rs3:5 00 rs2:5 rs1:5 010 rd:5 1000011" "asm"="fmadd.s {rd}, {rs1}, {rs2}, {rs3}, rdn" "asmonly" {
  ret void
}

define void @FMADD_S.3(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2, i64 "reg"="d" %rs3) "enc"="rs3:5 00 rs2:5 rs1:5 011 rd:5 1000011" "asm"="fmadd.s {rd}, {rs1}, {rs2}, {rs3}, rup" "asmonly" {
  ret void
}

define void @FMADD_S.4(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2, i64 "reg"="d" %rs3) "enc"="rs3:5 00 rs2:5 rs1:5 100 rd:5 1000011" "asm"="fmadd.s {rd}, {rs1}, {rs2}, {rs3}, rmm" "asmonly" {
  ret void
}

define void @FMSUB_S.0(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2, i64 "reg"="d" %rs3) "enc"="rs3:5 00 rs2:5 rs1:5 111 rd:5 1000111" "asm"="fmsub.s {rd}, {rs1}, {rs2}, {rs3}" "asmonly" {
  ret void
}

define void @FMSUB_S.1(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2, i64 "reg"="d" %rs3) "enc"="rs3:5 00 rs2:5 rs1:5 000 rd:5 1000111" "asm"="fmsub.s {rd}, {rs1}, {rs2}, {rs3}, rne" "asmonly" {
  ret void
}

define void @FMSUB_S.2(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2, i64 "reg"="d" %rs3) "enc"="rs3:5 00 rs2:5 rs1:5 001 rd:5 1000111" "asm"="fmsub.s {rd}, {rs1}, {rs2}, {rs3}, rtz" "asmonly" {
  ret void
}

define void @FMSUB_S.3(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2, i64 "reg"="d" %rs3) "enc"="rs3:5 00 rs2:5 rs1:5 010 rd:5 1000111" "asm"="fmsub.s {rd}, {rs1}, {rs2}, {rs3}, rdn" "asmonly" {
  ret void
}

define void @FMSUB_S.4(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2, i64 "reg"="d" %rs3) "enc"="rs3:5 00 rs2:5 rs1:5 011 rd:5 1000111" "asm"="fmsub.s {rd}, {rs1}, {rs2}, {rs3}, rup" "asmonly" {
  ret void
}

define void @FMSUB_S.5(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2, i64 "reg"="d" %rs3) "enc"="rs3:5 00 rs2:5 rs1:5 100 rd:5 1000111" "asm"="fmsub.s {rd}, {rs1}, {rs2}, {rs3}, rmm" "asmonly" {
  ret void
}

define void @FMSUB_S.6(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2, i64 "reg"="d" %rs3) "enc"="rs3:5 00 rs2:5 rs1:5 111 rd:5 1000111" "asm"="fmsub.s {rd}, {rs1}, {rs2}, {rs3}, dyn" "asmonly" "nodecode" {
  ret void
}

define void @FNMSUB_S.0(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2, i64 "reg"="d" %rs3) "enc"="rs3:5 00 rs2:5 rs1:5 111 rd:5 1001011" "asm"="fnmsub.s {rd}, {rs1}, {rs2}, {rs3}" "asmonly" {
  ret void
}

define void @FNMSUB_S.1(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2, i64 "reg"="d" %rs3) "enc"="rs3:5 00 rs2:5 rs1:5 000 rd:5 1001011" "asm"="fnmsub.s {rd}, {rs1}, {rs2}, {rs3}, rne" "asmonly" {
  ret void
}

define void @FNMSUB_S.2(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2, i64 "reg"="d" %rs3) "enc"="rs3:5 00 rs2:5 rs1:5 001 rd:5 1001011" "asm"="fnmsub.s {rd}, {rs1}, {rs2}, {rs3}, rtz" "asmonly" {
  ret void
}

define void @FNMSUB_S.3(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2, i64 "reg"="d" %rs3) "enc"="rs3:5 00 rs2:5 rs1:5 010 rd:5 1001011" "asm"="fnmsub.s {rd}, {rs1}, {rs2}, {rs3}, rdn" "asmonly" {
  ret void
}

define void @FNMSUB_S.4(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2, i64 "reg"="d" %rs3) "enc"="rs3:5 00 rs2:5 rs1:5 011 rd:5 1001011" "asm"="fnmsub.s {rd}, {rs1}, {rs2}, {rs3}, rup" "asmonly" {
  ret void
}

define void @FNMSUB_S.5(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2, i64 "reg"="d" %rs3) "enc"="rs3:5 00 rs2:5 rs1:5 100 rd:5 1001011" "asm"="fnmsub.s {rd}, {rs1}, {rs2}, {rs3}, rmm" "asmonly" {
  ret void
}

define void @FNMSUB_S.6(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2, i64 "reg"="d" %rs3) "enc"="rs3:5 00 rs2:5 rs1:5 111 rd:5 1001011" "asm"="fnmsub.s {rd}, {rs1}, {rs2}, {rs3}, dyn" "asmonly" "nodecode" {
  ret void
}

define void @FNMADD_S.0(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2, i64 "reg"="d" %rs3) "enc"="rs3:5 00 rs2:5 rs1:5 111 rd:5 1001111" "asm"="fnmadd.s {rd}, {rs1}, {rs2}, {rs3}" "asmonly" {
  ret void
}

define void @FNMADD_S.1(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2, i64 "reg"="d" %rs3) "enc"="rs3:5 00 rs2:5 rs1:5 000 rd:5 1001111" "asm"="fnmadd.s {rd}, {rs1}, {rs2}, {rs3}, rne" "asmonly" {
  ret void
}

define void @FNMADD_S.2(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2, i64 "reg"="d" %rs3) "enc"="rs3:5 00 rs2:5 rs1:5 001 rd:5 1001111" "asm"="fnmadd.s {rd}, {rs1}, {rs2}, {rs3}, rtz" "asmonly" {
  ret void
}

define void @FNMADD_S.3(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2, i64 "reg"="d" %rs3) "enc"="rs3:5 00 rs2:5 rs1:5 010 rd:5 1001111" "asm"="fnmadd.s {rd}, {rs1}, {rs2}, {rs3}, rdn" "asmonly" {
  ret void
}

define void @FNMADD_S.4(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2, i64 "reg"="d" %rs3) "enc"="rs3:5 00 rs2:5 rs1:5 011 rd:5 1001111" "asm"="fnmadd.s {rd}, {rs1}, {rs2}, {rs3}, rup" "asmonly" {
  ret void
}

define void @FNMADD_S.5(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2, i64 "reg"="d" %rs3) "enc"="rs3:5 00 rs2:5 rs1:5 100 rd:5 1001111" "asm"="fnmadd.s {rd}, {rs1}, {rs2}, {rs3}, rmm" "asmonly" {
  ret void
}

define void @FNMADD_S.6(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2, i64 "reg"="d" %rs3) "enc"="rs3:5 00 rs2:5 rs1:5 111 rd:5 1001111" "asm"="fnmadd.s {rd}, {rs1}, {rs2}, {rs3}, dyn" "asmonly" "nodecode" {
  ret void
}

define void @FADD_S.0(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2) "enc"="0000000 rs2:5 rs1:5 000 rd:5 1010011" "asm"="fadd.s {rd}, {rs1}, {rs2}, rne" "asmonly" {
  ret void
}

define void @FADD_S.1(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2) "enc"="0000000 rs2:5 rs1:5 001 rd:5 1010011" "asm"="fadd.s {rd}, {rs1}, {rs2}, rtz" "asmonly" {
  ret void
}

define void @FADD_S.2(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2) "enc"="0000000 rs2:5 rs1:5 010 rd:5 1010011" "asm"="fadd.s {rd}, {rs1}, {rs2}, rdn" "asmonly" {
  ret void
}

define void @FADD_S.3(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2) "enc"="0000000 rs2:5 rs1:5 011 rd:5 1010011" "asm"="fadd.s {rd}, {rs1}, {rs2}, rup" "asmonly" {
  ret void
}

define void @FADD_S.4(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2) "enc"="0000000 rs2:5 rs1:5 100 rd:5 1010011" "asm"="fadd.s {rd}, {rs1}, {rs2}, rmm" "asmonly" {
  ret void
}

define void @FSUB_S.0(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2) "enc"="0000100 rs2:5 rs1:5 000 rd:5 1010011" "asm"="fsub.s {rd}, {rs1}, {rs2}, rne" "asmonly" {
  ret void
}

define void @FSUB_S.1(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2) "enc"="0000100 rs2:5 rs1:5 001 rd:5 1010011" "asm"="fsub.s {rd}, {rs1}, {rs2}, rtz" "asmonly" {
  ret void
}

define void @FSUB_S.2(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2) "enc"="0000100 rs2:5 rs1:5 010 rd:5 1010011" "asm"="fsub.s {rd}, {rs1}, {rs2}, rdn" "asmonly" {
  ret void
}

define void @FSUB_S.3(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2) "enc"="0000100 rs2:5 rs1:5 011 rd:5 1010011" "asm"="fsub.s {rd}, {rs1}, {rs2}, rup" "asmonly" {
  ret void
}

define void @FSUB_S.4(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2) "enc"="0000100 rs2:5 rs1:5 100 rd:5 1010011" "asm"="fsub.s {rd}, {rs1}, {rs2}, rmm" "asmonly" {
  ret void
}

define void @FMUL_S.0(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2) "enc"="0001000 rs2:5 rs1:5 000 rd:5 1010011" "asm"="fmul.s {rd}, {rs1}, {rs2}, rne" "asmonly" {
  ret void
}

define void @FMUL_S.1(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2) "enc"="0001000 rs2:5 rs1:5 001 rd:5 1010011" "asm"="fmul.s {rd}, {rs1}, {rs2}, rtz" "asmonly" {
  ret void
}

define void @FMUL_S.2(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2) "enc"="0001000 rs2:5 rs1:5 010 rd:5 1010011" "asm"="fmul.s {rd}, {rs1}, {rs2}, rdn" "asmonly" {
  ret void
}

define void @FMUL_S.3(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2) "enc"="0001000 rs2:5 rs1:5 011 rd:5 1010011" "asm"="fmul.s {rd}, {rs1}, {rs2}, rup" "asmonly" {
  ret void
}

define void @FMUL_S.4(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2) "enc"="0001000 rs2:5 rs1:5 100 rd:5 1010011" "asm"="fmul.s {rd}, {rs1}, {rs2}, rmm" "asmonly" {
  ret void
}

define void @FDIV_S.0(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2) "enc"="0001100 rs2:5 rs1:5 000 rd:5 1010011" "asm"="fdiv.s {rd}, {rs1}, {rs2}, rne" "asmonly" {
  ret void
}

define void @FDIV_S.1(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2) "enc"="0001100 rs2:5 rs1:5 001 rd:5 1010011" "asm"="fdiv.s {rd}, {rs1}, {rs2}, rtz" "asmonly" {
  ret void
}

define void @FDIV_S.2(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2) "enc"="0001100 rs2:5 rs1:5 010 rd:5 1010011" "asm"="fdiv.s {rd}, {rs1}, {rs2}, rdn" "asmonly" {
  ret void
}

define void @FDIV_S.3(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2) "enc"="0001100 rs2:5 rs1:5 011 rd:5 1010011" "asm"="fdiv.s {rd}, {rs1}, {rs2}, rup" "asmonly" {
  ret void
}

define void @FDIV_S.4(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2) "enc"="0001100 rs2:5 rs1:5 100 rd:5 1010011" "asm"="fdiv.s {rd}, {rs1}, {rs2}, rmm" "asmonly" {
  ret void
}

define void @FSQRT_S.0(i64 "reg"="d" %rd, i64 "reg"="d" %rs1) "enc"="010110000000 rs1:5 000 rd:5 1010011" "asm"="fsqrt.s {rd}, {rs1}, rne" "asmonly" {
  ret void
}

define void @FSQRT_S.1(i64 "reg"="d" %rd, i64 "reg"="d" %rs1) "enc"="010110000000 rs1:5 001 rd:5 1010011" "asm"="fsqrt.s {rd}, {rs1}, rtz" "asmonly" {
  ret void
}

define void @FSQRT_S.2(i64 "reg"="d" %rd, i64 "reg"="d" %rs1) "enc"="010110000000 rs1:5 010 rd:5 1010011" "asm"="fsqrt.s {rd}, {rs1}, rdn" "asmonly" {
  ret void
}

define void @FSQRT_S.3(i64 "reg"="d" %rd, i64 "reg"="d" %rs1) "enc"="010110000000 rs1:5 011 rd:5 1010011" "asm"="fsqrt.s {rd}, {rs1}, rup" "asmonly" {
  ret void
}

define void @FSQRT_S.4(i64 "reg"="d" %rd, i64 "reg"="d" %rs1) "enc"="010110000000 rs1:5 100 rd:5 1010011" "asm"="fsqrt.s {rd}, {rs1}, rmm" "asmonly" {
  ret void
}

define void @FSGNJ_S.0(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2) "enc"="0010000 rs2:5 rs1:5 000 rd:5 1010011" "asm"="fsgnj.s {rd}, {rs1}, {rs2}" "asmonly" {
  ret void
}

define void @FSGNJN_S.0(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2) "enc"="0010000 rs2:5 rs1:5 001 rd:5 1010011" "asm"="fsgnjn.s {rd}, {rs1}, {rs2}" "asmonly" {
  ret void
}

define void @FSGNJX_S.0(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2) "enc"="0010000 rs2:5 rs1:5 010 rd:5 1010011" "asm"="fsgnjx.s {rd}, {rs1}, {rs2}" "asmonly" {
  ret void
}

define void @FMIN_S.0(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2) "enc"="0010100 rs2:5 rs1:5 000 rd:5 1010011" "asm"="fmin.s {rd}, {rs1}, {rs2}" "asmonly" {
  ret void
}

define void @FMAX_S.0(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2) "enc"="0010100 rs2:5 rs1:5 001 rd:5 1010011" "asm"="fmax.s {rd}, {rs1}, {rs2}" "asmonly" {
  ret void
}

define void @FCVT_W_S.0(i64 "reg"="x" %rd, i64 "reg"="d" %rs1) "enc"="110000000000 rs1:5 111 rd:5 1010011" "asm"="fcvt.w.s {rd}, {rs1}" "asmonly" {
  ret void
}

define void @FCVT_W_S.1(i64 "reg"="x" %rd, i64 "reg"="d" %rs1) "enc"="110000000000 rs1:5 000 rd:5 1010011" "asm"="fcvt.w.s {rd}, {rs1}, rne" "asmonly" {
  ret void
}

define void @FCVT_W_S.2(i64 "reg"="x" %rd, i64 "reg"="d" %rs1) "enc"="110000000000 rs1:5 010 rd:5 1010011" "asm"="fcvt.w.s {rd}, {rs1}, rdn" "asmonly" {
  ret void
}

define void @FCVT_W_S.3(i64 "reg"="x" %rd, i64 "reg"="d" %rs1) "enc"="110000000000 rs1:5 011 rd:5 1010011" "asm"="fcvt.w.s {rd}, {rs1}, rup" "asmonly" {
  ret void
}

define void @FCVT_W_S.4(i64 "reg"="x" %rd, i64 "reg"="d" %rs1) "enc"="110000000000 rs1:5 100 rd:5 1010011" "asm"="fcvt.w.s {rd}, {rs1}, rmm" "asmonly" {
  ret void
}

define void @FCVT_W_S.5(i64 "reg"="x" %rd, i64 "reg"="d" %rs1) "enc"="110000000000 rs1:5 111 rd:5 1010011" "asm"="fcvt.w.s {rd}, {rs1}, dyn" "asmonly" "nodecode" {
  ret void
}

define void @FCVT_WU_S.0(i64 "reg"="x" %rd, i64 "reg"="d" %rs1) "enc"="110000000001 rs1:5 111 rd:5 1010011" "asm"="fcvt.wu.s {rd}, {rs1}" "asmonly" {
  ret void
}

define void @FCVT_WU_S.1(i64 "reg"="x" %rd, i64 "reg"="d" %rs1) "enc"="110000000001 rs1:5 000 rd:5 1010011" "asm"="fcvt.wu.s {rd}, {rs1}, rne" "asmonly" {
  ret void
}

define void @FCVT_WU_S.2(i64 "reg"="x" %rd, i64 "reg"="d" %rs1) "enc"="110000000001 rs1:5 010 rd:5 1010011" "asm"="fcvt.wu.s {rd}, {rs1}, rdn" "asmonly" {
  ret void
}

define void @FCVT_WU_S.3(i64 "reg"="x" %rd, i64 "reg"="d" %rs1) "enc"="110000000001 rs1:5 011 rd:5 1010011" "asm"="fcvt.wu.s {rd}, {rs1}, rup" "asmonly" {
  ret void
}

define void @FCVT_WU_S.4(i64 "reg"="x" %rd, i64 "reg"="d" %rs1) "enc"="110000000001 rs1:5 100 rd:5 1010011" "asm"="fcvt.wu.s {rd}, {rs1}, rmm" "asmonly" {
  ret void
}

define void @FCVT_WU_S.5(i64 "reg"="x" %rd, i64 "reg"="d" %rs1) "enc"="110000000001 rs1:5 111 rd:5 1010011" "asm"="fcvt.wu.s {rd}, {rs1}, dyn" "asmonly" "nodecode" {
  ret void
}

define void @FCLASS_S.0(i64 "reg"="x" %rd, i64 "reg"="d" %rs1) "enc"="111000000000 rs1:5 001 rd:5 1010011" "asm"="fclass.s {rd}, {rs1}" "asmonly" {
  ret void
}

define void @FCVT_S_W.0(i64 "reg"="d" %rd, i64 "reg"="x" %rs1) "enc"="110100000000 rs1:5 000 rd:5 1010011" "asm"="fcvt.s.w {rd}, {rs1}, rne" "asmonly" {
  ret void
}

define void @FCVT_S_W.1(i64 "reg"="d" %rd, i64 "reg"="x" %rs1) "enc"="110100000000 rs1:5 001 rd:5 1010011" "asm"="fcvt.s.w {rd}, {rs1}, rtz" "asmonly" {
  ret void
}

define void @FCVT_S_W.2(i64 "reg"="d" %rd, i64 "reg"="x" %rs1) "enc"="110100000000 rs1:5 010 rd:5 1010011" "asm"="fcvt.s.w {rd}, {rs1}, rdn" "asmonly" {
  ret void
}

define void @FCVT_S_W.3(i64 "reg"="d" %rd, i64 "reg"="x" %rs1) "enc"="110100000000 rs1:5 011 rd:5 1010011" "asm"="fcvt.s.w {rd}, {rs1}, rup" "asmonly" {
  ret void
}

define void @FCVT_S_W.4(i64 "reg"="d" %rd, i64 "reg"="x" %rs1) "enc"="110100000000 rs1:5 100 rd:5 1010011" "asm"="fcvt.s.w {rd}, {rs1}, rmm" "asmonly" {
  ret void
}

define void @FCVT_S_WU.0(i64 "reg"="d" %rd, i64 "reg"="x" %rs1) "enc"="110100000001 rs1:5 000 rd:5 1010011" "asm"="fcvt.s.wu {rd}, {rs1}, rne" "asmonly" {
  ret void
}

define void @FCVT_S_WU.1(i64 "reg"="d" %rd, i64 "reg"="x" %rs1) "enc"="110100000001 rs1:5 001 rd:5 1010011" "asm"="fcvt.s.wu {rd}, {rs1}, rtz" "asmonly" {
  ret void
}

define void @FCVT_S_WU.2(i64 "reg"="d" %rd, i64 "reg"="x" %rs1) "enc"="110100000001 rs1:5 010 rd:5 1010011" "asm"="fcvt.s.wu {rd}, {rs1}, rdn" "asmonly" {
  ret void
}

define void @FCVT_S_WU.3(i64 "reg"="d" %rd, i64 "reg"="x" %rs1) "enc"="110100000001 rs1:5 011 rd:5 1010011" "asm"="fcvt.s.wu {rd}, {rs1}, rup" "asmonly" {
  ret void
}

define void @FCVT_S_WU.4(i64 "reg"="d" %rd, i64 "reg"="x" %rs1) "enc"="110100000001 rs1:5 100 rd:5 1010011" "asm"="fcvt.s.wu {rd}, {rs1}, rmm" "asmonly" {
  ret void
}

define void @FCVT_L_S.0(i64 "reg"="x" %rd, i64 "reg"="d" %rs1) "enc"="110000000010 rs1:5 111 rd:5 1010011" "asm"="fcvt.l.s {rd}, {rs1}" "asmonly" {
  ret void
}

define void @FCVT_L_S.1(i64 "reg"="x" %rd, i64 "reg"="d" %rs1) "enc"="110000000010 rs1:5 000 rd:5 1010011" "asm"="fcvt.l.s {rd}, {rs1}, rne" "asmonly" {
  ret void
}

define void @FCVT_L_S.2(i64 "reg"="x" %rd, i64 "reg"="d" %rs1) "enc"="110000000010 rs1:5 010 rd:5 1010011" "asm"="fcvt.l.s {rd}, {rs1}, rdn" "asmonly" {
  ret void
}

define void @FCVT_L_S.3(i64 "reg"="x" %rd, i64 "reg"="d" %rs1) "enc"="110000000010 rs1:5 011 rd:5 1010011" "asm"="fcvt.l.s {rd}, {rs1}, rup" "asmonly" {
  ret void
}

define void @FCVT_L_S.4(i64 "reg"="x" %rd, i64 "reg"="d" %rs1) "enc"="110000000010 rs1:5 100 rd:5 1010011" "asm"="fcvt.l.s {rd}, {rs1}, rmm" "asmonly" {
  ret void
}

define void @FCVT_L_S.5(i64 "reg"="x" %rd, i64 "reg"="d" %rs1) "enc"="110000000010 rs1:5 111 rd:5 1010011" "asm"="fcvt.l.s {rd}, {rs1}, dyn" "asmonly" "nodecode" {
  ret void
}

define void @FCVT_LU_S.0(i64 "reg"="x" %rd, i64 "reg"="d" %rs1) "enc"="110000000011 rs1:5 111 rd:5 1010011" "asm"="fcvt.lu.s {rd}, {rs1}" "asmonly" {
  ret void
}

define void @FCVT_LU_S.1(i64 "reg"="x" %rd, i64 "reg"="d" %rs1) "enc"="110000000011 rs1:5 000 rd:5 1010011" "asm"="fcvt.lu.s {rd}, {rs1}, rne" "asmonly" {
  ret void
}

define void @FCVT_LU_S.2(i64 "reg"="x" %rd, i64 "reg"="d" %rs1) "enc"="110000000011 rs1:5 010 rd:5 1010011" "asm"="fcvt.lu.s {rd}, {rs1}, rdn" "asmonly" {
  ret void
}

define void @FCVT_LU_S.3(i64 "reg"="x" %rd, i64 "reg"="d" %rs1) "enc"="110000000011 rs1:5 011 rd:5 1010011" "asm"="fcvt.lu.s {rd}, {rs1}, rup" "asmonly" {
  ret void
}

define void @FCVT_LU_S.4(i64 "reg"="x" %rd, i64 "reg"="d" %rs1) "enc"="110000000011 rs1:5 100 rd:5 1010011" "asm"="fcvt.lu.s {rd}, {rs1}, rmm" "asmonly" {
  ret void
}

define void @FCVT_LU_S.5(i64 "reg"="x" %rd, i64 "reg"="d" %rs1) "enc"="110000000011 rs1:5 111 rd:5 1010011" "asm"="fcvt.lu.s {rd}, {rs1}, dyn" "asmonly" "nodecode" {
  ret void
}

define void @FCVT_S_L.0(i64 "reg"="d" %rd, i64 "reg"="x" %rs1) "enc"="110100000010 rs1:5 000 rd:5 1010011" "asm"="fcvt.s.l {rd}, {rs1}, rne" "asmonly" {
  ret void
}

define void @FCVT_S_L.1(i64 "reg"="d" %rd, i64 "reg"="x" %rs1) "enc"="110100000010 rs1:5 001 rd:5 1010011" "asm"="fcvt.s.l {rd}, {rs1}, rtz" "asmonly" {
  ret void
}

define void @FCVT_S_L.2(i64 "reg"="d" %rd, i64 "reg"="x" %rs1) "enc"="110100000010 rs1:5 010 rd:5 1010011" "asm"="fcvt.s.l {rd}, {rs1}, rdn" "asmonly" {
  ret void
}

define void @FCVT_S_L.3(i64 "reg"="d" %rd, i64 "reg"="x" %rs1) "enc"="110100000010 rs1:5 011 rd:5 1010011" "asm"="fcvt.s.l {rd}, {rs1}, rup" "asmonly" {
  ret void
}

define void @FCVT_S_L.4(i64 "reg"="d" %rd, i64 "reg"="x" %rs1) "enc"="110100000010 rs1:5 100 rd:5 1010011" "asm"="fcvt.s.l {rd}, {rs1}, rmm" "asmonly" {
  ret void
}

define void @FCVT_S_LU.0(i64 "reg"="d" %rd, i64 "reg"="x" %rs1) "enc"="110100000011 rs1:5 000 rd:5 1010011" "asm"="fcvt.s.lu {rd}, {rs1}, rne" "asmonly" {
  ret void
}

define void @FCVT_S_LU.1(i64 "reg"="d" %rd, i64 "reg"="x" %rs1) "enc"="110100000011 rs1:5 001 rd:5 1010011" "asm"="fcvt.s.lu {rd}, {rs1}, rtz" "asmonly" {
  ret void
}

define void @FCVT_S_LU.2(i64 "reg"="d" %rd, i64 "reg"="x" %rs1) "enc"="110100000011 rs1:5 010 rd:5 1010011" "asm"="fcvt.s.lu {rd}, {rs1}, rdn" "asmonly" {
  ret void
}

define void @FCVT_S_LU.3(i64 "reg"="d" %rd, i64 "reg"="x" %rs1) "enc"="110100000011 rs1:5 011 rd:5 1010011" "asm"="fcvt.s.lu {rd}, {rs1}, rup" "asmonly" {
  ret void
}

define void @FCVT_S_LU.4(i64 "reg"="d" %rd, i64 "reg"="x" %rs1) "enc"="110100000011 rs1:5 100 rd:5 1010011" "asm"="fcvt.s.lu {rd}, {rs1}, rmm" "asmonly" {
  ret void
}

define void @FMADD_D.0(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2, i64 "reg"="d" %rs3) "enc"="rs3:5 01 rs2:5 rs1:5 000 rd:5 1000011" "asm"="fmadd.d {rd}, {rs1}, {rs2}, {rs3}, rne" "asmonly" {
  ret void
}

define void @FMADD_D.1(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2, i64 "reg"="d" %rs3) "enc"="rs3:5 01 rs2:5 rs1:5 001 rd:5 1000011" "asm"="fmadd.d {rd}, {rs1}, {rs2}, {rs3}, rtz" "asmonly" {
  ret void
}

define void @FMADD_D.2(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2, i64 "reg"="d" %rs3) "enc"="rs3:5 01 rs2:5 rs1:5 010 rd:5 1000011" "asm"="fmadd.d {rd}, {rs1}, {rs2}, {rs3}, rdn" "asmonly" {
  ret void
}

define void @FMADD_D.3(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2, i64 "reg"="d" %rs3) "enc"="rs3:5 01 rs2:5 rs1:5 011 rd:5 1000011" "asm"="fmadd.d {rd}, {rs1}, {rs2}, {rs3}, rup" "asmonly" {
  ret void
}

define void @FMADD_D.4(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2, i64 "reg"="d" %rs3) "enc"="rs3:5 01 rs2:5 rs1:5 100 rd:5 1000011" "asm"="fmadd.d {rd}, {rs1}, {rs2}, {rs3}, rmm" "asmonly" {
  ret void
}

define void @FMSUB_D.0(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2, i64 "reg"="d" %rs3) "enc"="rs3:5 01 rs2:5 rs1:5 111 rd:5 1000111" "asm"="fmsub.d {rd}, {rs1}, {rs2}, {rs3}" "asmonly" {
  ret void
}

define void @FMSUB_D.1(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2, i64 "reg"="d" %rs3) "enc"="rs3:5 01 rs2:5 rs1:5 000 rd:5 1000111" "asm"="fmsub.d {rd}, {rs1}, {rs2}, {rs3}, rne" "asmonly" {
  ret void
}

define void @FMSUB_D.2(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2, i64 "reg"="d" %rs3) "enc"="rs3:5 01 rs2:5 rs1:5 001 rd:5 1000111" "asm"="fmsub.d {rd}, {rs1}, {rs2}, {rs3}, rtz" "asmonly" {
  ret void
}

define void @FMSUB_D.3(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2, i64 "reg"="d" %rs3) "enc"="rs3:5 01 rs2:5 rs1:5 010 rd:5 1000111" "asm"="fmsub.d {rd}, {rs1}, {rs2}, {rs3}, rdn" "asmonly" {
  ret void
}

define void @FMSUB_D.4(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2, i64 "reg"="d" %rs3) "enc"="rs3:5 01 rs2:5 rs1:5 011 rd:5 1000111" "asm"="fmsub.d {rd}, {rs1}, {rs2}, {rs3}, rup" "asmonly" {
  ret void
}

define void @FMSUB_D.5(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2, i64 "reg"="d" %rs3) "enc"="rs3:5 01 rs2:5 rs1:5 100 rd:5 1000111" "asm"="fmsub.d {rd}, {rs1}, {rs2}, {rs3}, rmm" "asmonly" {
  ret void
}

define void @FMSUB_D.6(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2, i64 "reg"="d" %rs3) "enc"="rs3:5 01 rs2:5 rs1:5 111 rd:5 1000111" "asm"="fmsub.d {rd}, {rs1}, {rs2}, {rs3}, dyn" "asmonly" "nodecode" {
  ret void
}

define void @FNMSUB_D.0(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2, i64 "reg"="d" %rs3) "enc"="rs3:5 01 rs2:5 rs1:5 111 rd:5 1001011" "asm"="fnmsub.d {rd}, {rs1}, {rs2}, {rs3}" "asmonly" {
  ret void
}

define void @FNMSUB_D.1(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2, i64 "reg"="d" %rs3) "enc"="rs3:5 01 rs2:5 rs1:5 000 rd:5 1001011" "asm"="fnmsub.d {rd}, {rs1}, {rs2}, {rs3}, rne" "asmonly" {
  ret void
}

define void @FNMSUB_D.2(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2, i64 "reg"="d" %rs3) "enc"="rs3:5 01 rs2:5 rs1:5 001 rd:5 1001011" "asm"="fnmsub.d {rd}, {rs1}, {rs2}, {rs3}, rtz" "asmonly" {
  ret void
}

define void @FNMSUB_D.3(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2, i64 "reg"="d" %rs3) "enc"="rs3:5 01 rs2:5 rs1:5 010 rd:5 1001011" "asm"="fnmsub.d {rd}, {rs1}, {rs2}, {rs3}, rdn" "asmonly" {
  ret void
}

define void @FNMSUB_D.4(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2, i64 "reg"="d" %rs3) "enc"="rs3:5 01 rs2:5 rs1:5 011 rd:5 1001011" "asm"="fnmsub.d {rd}, {rs1}, {rs2}, {rs3}, rup" "asmonly" {
  ret void
}

define void @FNMSUB_D.5(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2, i64 "reg"="d" %rs3) "enc"="rs3:5 01 rs2:5 rs1:5 100 rd:5 1001011" "asm"="fnmsub.d {rd}, {rs1}, {rs2}, {rs3}, rmm" "asmonly" {
  ret void
}

define void @FNMSUB_D.6(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2, i64 "reg"="d" %rs3) "enc"="rs3:5 01 rs2:5 rs1:5 111 rd:5 1001011" "asm"="fnmsub.d {rd}, {rs1}, {rs2}, {rs3}, dyn" "asmonly" "nodecode" {
  ret void
}

define void @FNMADD_D.0(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2, i64 "reg"="d" %rs3) "enc"="rs3:5 01 rs2:5 rs1:5 111 rd:5 1001111" "asm"="fnmadd.d {rd}, {rs1}, {rs2}, {rs3}" "asmonly" {
  ret void
}

define void @FNMADD_D.1(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2, i64 "reg"="d" %rs3) "enc"="rs3:5 01 rs2:5 rs1:5 000 rd:5 1001111" "asm"="fnmadd.d {rd}, {rs1}, {rs2}, {rs3}, rne" "asmonly" {
  ret void
}

define void @FNMADD_D.2(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2, i64 "reg"="d" %rs3) "enc"="rs3:5 01 rs2:5 rs1:5 001 rd:5 1001111" "asm"="fnmadd.d {rd}, {rs1}, {rs2}, {rs3}, rtz" "asmonly" {
  ret void
}

define void @FNMADD_D.3(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2, i64 "reg"="d" %rs3) "enc"="rs3:5 01 rs2:5 rs1:5 010 rd:5 1001111" "asm"="fnmadd.d {rd}, {rs1}, {rs2}, {rs3}, rdn" "asmonly" {
  ret void
}

define void @FNMADD_D.4(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2, i64 "reg"="d" %rs3) "enc"="rs3:5 01 rs2:5 rs1:5 011 rd:5 1001111" "asm"="fnmadd.d {rd}, {rs1}, {rs2}, {rs3}, rup" "asmonly" {
  ret void
}

define void @FNMADD_D.5(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2, i64 "reg"="d" %rs3) "enc"="rs3:5 01 rs2:5 rs1:5 100 rd:5 1001111" "asm"="fnmadd.d {rd}, {rs1}, {rs2}, {rs3}, rmm" "asmonly" {
  ret void
}

define void @FNMADD_D.6(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2, i64 "reg"="d" %rs3) "enc"="rs3:5 01 rs2:5 rs1:5 111 rd:5 1001111" "asm"="fnmadd.d {rd}, {rs1}, {rs2}, {rs3}, dyn" "asmonly" "nodecode" {
  ret void
}

define void @FADD_D.0(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2) "enc"="0000001 rs2:5 rs1:5 000 rd:5 1010011" "asm"="fadd.d {rd}, {rs1}, {rs2}, rne" "asmonly" {
  ret void
}

define void @FADD_D.1(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2) "enc"="0000001 rs2:5 rs1:5 001 rd:5 1010011" "asm"="fadd.d {rd}, {rs1}, {rs2}, rtz" "asmonly" {
  ret void
}

define void @FADD_D.2(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2) "enc"="0000001 rs2:5 rs1:5 010 rd:5 1010011" "asm"="fadd.d {rd}, {rs1}, {rs2}, rdn" "asmonly" {
  ret void
}

define void @FADD_D.3(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2) "enc"="0000001 rs2:5 rs1:5 011 rd:5 1010011" "asm"="fadd.d {rd}, {rs1}, {rs2}, rup" "asmonly" {
  ret void
}

define void @FADD_D.4(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2) "enc"="0000001 rs2:5 rs1:5 100 rd:5 1010011" "asm"="fadd.d {rd}, {rs1}, {rs2}, rmm" "asmonly" {
  ret void
}

define void @FSUB_D.0(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2) "enc"="0000101 rs2:5 rs1:5 000 rd:5 1010011" "asm"="fsub.d {rd}, {rs1}, {rs2}, rne" "asmonly" {
  ret void
}

define void @FSUB_D.1(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2) "enc"="0000101 rs2:5 rs1:5 001 rd:5 1010011" "asm"="fsub.d {rd}, {rs1}, {rs2}, rtz" "asmonly" {
  ret void
}

define void @FSUB_D.2(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2) "enc"="0000101 rs2:5 rs1:5 010 rd:5 1010011" "asm"="fsub.d {rd}, {rs1}, {rs2}, rdn" "asmonly" {
  ret void
}

define void @FSUB_D.3(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2) "enc"="0000101 rs2:5 rs1:5 011 rd:5 1010011" "asm"="fsub.d {rd}, {rs1}, {rs2}, rup" "asmonly" {
  ret void
}

define void @FSUB_D.4(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2) "enc"="0000101 rs2:5 rs1:5 100 rd:5 1010011" "asm"="fsub.d {rd}, {rs1}, {rs2}, rmm" "asmonly" {
  ret void
}

define void @FMUL_D.0(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2) "enc"="0001001 rs2:5 rs1:5 000 rd:5 1010011" "asm"="fmul.d {rd}, {rs1}, {rs2}, rne" "asmonly" {
  ret void
}

define void @FMUL_D.1(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2) "enc"="0001001 rs2:5 rs1:5 001 rd:5 1010011" "asm"="fmul.d {rd}, {rs1}, {rs2}, rtz" "asmonly" {
  ret void
}

define void @FMUL_D.2(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2) "enc"="0001001 rs2:5 rs1:5 010 rd:5 1010011" "asm"="fmul.d {rd}, {rs1}, {rs2}, rdn" "asmonly" {
  ret void
}

define void @FMUL_D.3(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2) "enc"="0001001 rs2:5 rs1:5 011 rd:5 1010011" "asm"="fmul.d {rd}, {rs1}, {rs2}, rup" "asmonly" {
  ret void
}

define void @FMUL_D.4(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2) "enc"="0001001 rs2:5 rs1:5 100 rd:5 1010011" "asm"="fmul.d {rd}, {rs1}, {rs2}, rmm" "asmonly" {
  ret void
}

define void @FDIV_D.0(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2) "enc"="0001101 rs2:5 rs1:5 000 rd:5 1010011" "asm"="fdiv.d {rd}, {rs1}, {rs2}, rne" "asmonly" {
  ret void
}

define void @FDIV_D.1(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2) "enc"="0001101 rs2:5 rs1:5 001 rd:5 1010011" "asm"="fdiv.d {rd}, {rs1}, {rs2}, rtz" "asmonly" {
  ret void
}

define void @FDIV_D.2(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2) "enc"="0001101 rs2:5 rs1:5 010 rd:5 1010011" "asm"="fdiv.d {rd}, {rs1}, {rs2}, rdn" "asmonly" {
  ret void
}

define void @FDIV_D.3(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2) "enc"="0001101 rs2:5 rs1:5 011 rd:5 1010011" "asm"="fdiv.d {rd}, {rs1}, {rs2}, rup" "asmonly" {
  ret void
}

define void @FDIV_D.4(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2) "enc"="0001101 rs2:5 rs1:5 100 rd:5 1010011" "asm"="fdiv.d {rd}, {rs1}, {rs2}, rmm" "asmonly" {
  ret void
}

define void @FSQRT_D.0(i64 "reg"="d" %rd, i64 "reg"="d" %rs1) "enc"="010110100000 rs1:5 000 rd:5 1010011" "asm"="fsqrt.d {rd}, {rs1}, rne" "asmonly" {
  ret void
}

define void @FSQRT_D.1(i64 "reg"="d" %rd, i64 "reg"="d" %rs1) "enc"="010110100000 rs1:5 001 rd:5 1010011" "asm"="fsqrt.d {rd}, {rs1}, rtz" "asmonly" {
  ret void
}

define void @FSQRT_D.2(i64 "reg"="d" %rd, i64 "reg"="d" %rs1) "enc"="010110100000 rs1:5 010 rd:5 1010011" "asm"="fsqrt.d {rd}, {rs1}, rdn" "asmonly" {
  ret void
}

define void @FSQRT_D.3(i64 "reg"="d" %rd, i64 "reg"="d" %rs1) "enc"="010110100000 rs1:5 011 rd:5 1010011" "asm"="fsqrt.d {rd}, {rs1}, rup" "asmonly" {
  ret void
}

define void @FSQRT_D.4(i64 "reg"="d" %rd, i64 "reg"="d" %rs1) "enc"="010110100000 rs1:5 100 rd:5 1010011" "asm"="fsqrt.d {rd}, {rs1}, rmm" "asmonly" {
  ret void
}

define void @FSGNJ_D.0(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2) "enc"="0010001 rs2:5 rs1:5 000 rd:5 1010011" "asm"="fsgnj.d {rd}, {rs1}, {rs2}" "asmonly" {
  ret void
}

define void @FSGNJN_D.0(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2) "enc"="0010001 rs2:5 rs1:5 001 rd:5 1010011" "asm"="fsgnjn.d {rd}, {rs1}, {rs2}" "asmonly" {
  ret void
}

define void @FSGNJX_D.0(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2) "enc"="0010001 rs2:5 rs1:5 010 rd:5 1010011" "asm"="fsgnjx.d {rd}, {rs1}, {rs2}" "asmonly" {
  ret void
}

define void @FMIN_D.0(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2) "enc"="0010101 rs2:5 rs1:5 000 rd:5 1010011" "asm"="fmin.d {rd}, {rs1}, {rs2}" "asmonly" {
  ret void
}

define void @FMAX_D.0(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2) "enc"="0010101 rs2:5 rs1:5 001 rd:5 1010011" "asm"="fmax.d {rd}, {rs1}, {rs2}" "asmonly" {
  ret void
}

define void @FCVT_S_D.0(i64 "reg"="d" %rd, i64 "reg"="d" %rs1) "enc"="010000000001 rs1:5 000 rd:5 1010011" "asm"="fcvt.s.d {rd}, {rs1}, rne" "asmonly" {
  ret void
}

define void @FCVT_S_D.1(i64 "reg"="d" %rd, i64 "reg"="d" %rs1) "enc"="010000000001 rs1:5 001 rd:5 1010011" "asm"="fcvt.s.d {rd}, {rs1}, rtz" "asmonly" {
  ret void
}

define void @FCVT_S_D.2(i64 "reg"="d" %rd, i64 "reg"="d" %rs1) "enc"="010000000001 rs1:5 010 rd:5 1010011" "asm"="fcvt.s.d {rd}, {rs1}, rdn" "asmonly" {
  ret void
}

define void @FCVT_S_D.3(i64 "reg"="d" %rd, i64 "reg"="d" %rs1) "enc"="010000000001 rs1:5 011 rd:5 1010011" "asm"="fcvt.s.d {rd}, {rs1}, rup" "asmonly" {
  ret void
}

define void @FCVT_S_D.4(i64 "reg"="d" %rd, i64 "reg"="d" %rs1) "enc"="010000000001 rs1:5 100 rd:5 1010011" "asm"="fcvt.s.d {rd}, {rs1}, rmm" "asmonly" {
  ret void
}

define void @FCVT_D_S.0(i64 "reg"="d" %rd, i64 "reg"="d" %rs1) "enc"="010000100000 rs1:5 001 rd:5 1010011" "asm"="fcvt.d.s {rd}, {rs1}, rtz" "asmonly" {
  ret void
}

define void @FCVT_D_S.1(i64 "reg"="d" %rd, i64 "reg"="d" %rs1) "enc"="010000100000 rs1:5 010 rd:5 1010011" "asm"="fcvt.d.s {rd}, {rs1}, rdn" "asmonly" {
  ret void
}

define void @FCVT_D_S.2(i64 "reg"="d" %rd, i64 "reg"="d" %rs1) "enc"="010000100000 rs1:5 011 rd:5 1010011" "asm"="fcvt.d.s {rd}, {rs1}, rup" "asmonly" {
  ret void
}

define void @FCVT_D_S.3(i64 "reg"="d" %rd, i64 "reg"="d" %rs1) "enc"="010000100000 rs1:5 100 rd:5 1010011" "asm"="fcvt.d.s {rd}, {rs1}, rmm" "asmonly" {
  ret void
}

define void @FCVT_D_S.4(i64 "reg"="d" %rd, i64 "reg"="d" %rs1) "enc"="010000100000 rs1:5 111 rd:5 1010011" "asm"="fcvt.d.s {rd}, {rs1}, dyn" "asmonly" {
  ret void
}

define void @FCLASS_D.0(i64 "reg"="x" %rd, i64 "reg"="d" %rs1) "enc"="111000100000 rs1:5 001 rd:5 1010011" "asm"="fclass.d {rd}, {rs1}" "asmonly" {
  ret void
}

define void @FCVT_W_D.0(i64 "reg"="x" %rd, i64 "reg"="d" %rs1) "enc"="110000100000 rs1:5 111 rd:5 1010011" "asm"="fcvt.w.d {rd}, {rs1}" "asmonly" {
  ret void
}

define void @FCVT_W_D.1(i64 "reg"="x" %rd, i64 "reg"="d" %rs1) "enc"="110000100000 rs1:5 000 rd:5 1010011" "asm"="fcvt.w.d {rd}, {rs1}, rne" "asmonly" {
  ret void
}

define void @FCVT_W_D.2(i64 "reg"="x" %rd, i64 "reg"="d" %rs1) "enc"="110000100000 rs1:5 010 rd:5 1010011" "asm"="fcvt.w.d {rd}, {rs1}, rdn" "asmonly" {
  ret void
}

define void @FCVT_W_D.3(i64 "reg"="x" %rd, i64 "reg"="d" %rs1) "enc"="110000100000 rs1:5 011 rd:5 1010011" "asm"="fcvt.w.d {rd}, {rs1}, rup" "asmonly" {
  ret void
}

define void @FCVT_W_D.4(i64 "reg"="x" %rd, i64 "reg"="d" %rs1) "enc"="110000100000 rs1:5 100 rd:5 1010011" "asm"="fcvt.w.d {rd}, {rs1}, rmm" "asmonly" {
  ret void
}

define void @FCVT_W_D.5(i64 "reg"="x" %rd, i64 "reg"="d" %rs1) "enc"="110000100000 rs1:5 111 rd:5 1010011" "asm"="fcvt.w.d {rd}, {rs1}, dyn" "asmonly" "nodecode" {
  ret void
}

define void @FCVT_WU_D.0(i64 "reg"="x" %rd, i64 "reg"="d" %rs1) "enc"="110000100001 rs1:5 111 rd:5 1010011" "asm"="fcvt.wu.d {rd}, {rs1}" "asmonly" {
  ret void
}

define void @FCVT_WU_D.1(i64 "reg"="x" %rd, i64 "reg"="d" %rs1) "enc"="110000100001 rs1:5 000 rd:5 1010011" "asm"="fcvt.wu.d {rd}, {rs1}, rne" "asmonly" {
  ret void
}

define void @FCVT_WU_D.2(i64 "reg"="x" %rd, i64 "reg"="d" %rs1) "enc"="110000100001 rs1:5 010 rd:5 1010011" "asm"="fcvt.wu.d {rd}, {rs1}, rdn" "asmonly" {
  ret void
}

define void @FCVT_WU_D.3(i64 "reg"="x" %rd, i64 "reg"="d" %rs1) "enc"="110000100001 rs1:5 011 rd:5 1010011" "asm"="fcvt.wu.d {rd}, {rs1}, rup" "asmonly" {
  ret void
}

define void @FCVT_WU_D.4(i64 "reg"="x" %rd, i64 "reg"="d" %rs1) "enc"="110000100001 rs1:5 100 rd:5 1010011" "asm"="fcvt.wu.d {rd}, {rs1}, rmm" "asmonly" {
  ret void
}

define void @FCVT_WU_D.5(i64 "reg"="x" %rd, i64 "reg"="d" %rs1) "enc"="110000100001 rs1:5 111 rd:5 1010011" "asm"="fcvt.wu.d {rd}, {rs1}, dyn" "asmonly" "nodecode" {
  ret void
}

define void @FCVT_D_W.0(i64 "reg"="d" %rd, i64 "reg"="x" %rs1) "enc"="110100100000 rs1:5 001 rd:5 1010011" "asm"="fcvt.d.w {rd}, {rs1}, rtz" "asmonly" {
  ret void
}

define void @FCVT_D_W.1(i64 "reg"="d" %rd, i64 "reg"="x" %rs1) "enc"="110100100000 rs1:5 010 rd:5 1010011" "asm"="fcvt.d.w {rd}, {rs1}, rdn" "asmonly" {
  ret void
}

define void @FCVT_D_W.2(i64 "reg"="d" %rd, i64 "reg"="x" %rs1) "enc"="110100100000 rs1:5 011 rd:5 1010011" "asm"="fcvt.d.w {rd}, {rs1}, rup" "asmonly" {
  ret void
}

define void @FCVT_D_W.3(i64 "reg"="d" %rd, i64 "reg"="x" %rs1) "enc"="110100100000 rs1:5 100 rd:5 1010011" "asm"="fcvt.d.w {rd}, {rs1}, rmm" "asmonly" {
  ret void
}

define void @FCVT_D_W.4(i64 "reg"="d" %rd, i64 "reg"="x" %rs1) "enc"="110100100000 rs1:5 111 rd:5 1010011" "asm"="fcvt.d.w {rd}, {rs1}, dyn" "asmonly" {
  ret void
}

define void @FCVT_D_WU.0(i64 "reg"="d" %rd, i64 "reg"="x" %rs1) "enc"="110100100001 rs1:5 001 rd:5 1010011" "asm"="fcvt.d.wu {rd}, {rs1}, rtz" "asmonly" {
  ret void
}

define void @FCVT_D_WU.1(i64 "reg"="d" %rd, i64 "reg"="x" %rs1) "enc"="110100100001 rs1:5 010 rd:5 1010011" "asm"="fcvt.d.wu {rd}, {rs1}, rdn" "asmonly" {
  ret void
}

define void @FCVT_D_WU.2(i64 "reg"="d" %rd, i64 "reg"="x" %rs1) "enc"="110100100001 rs1:5 011 rd:5 1010011" "asm"="fcvt.d.wu {rd}, {rs1}, rup" "asmonly" {
  ret void
}

define void @FCVT_D_WU.3(i64 "reg"="d" %rd, i64 "reg"="x" %rs1) "enc"="110100100001 rs1:5 100 rd:5 1010011" "asm"="fcvt.d.wu {rd}, {rs1}, rmm" "asmonly" {
  ret void
}

define void @FCVT_D_WU.4(i64 "reg"="d" %rd, i64 "reg"="x" %rs1) "enc"="110100100001 rs1:5 111 rd:5 1010011" "asm"="fcvt.d.wu {rd}, {rs1}, dyn" "asmonly" {
  ret void
}

define void @FCVT_L_D.0(i64 "reg"="x" %rd, i64 "reg"="d" %rs1) "enc"="110000100010 rs1:5 111 rd:5 1010011" "asm"="fcvt.l.d {rd}, {rs1}" "asmonly" {
  ret void
}

define void @FCVT_L_D.1(i64 "reg"="x" %rd, i64 "reg"="d" %rs1) "enc"="110000100010 rs1:5 000 rd:5 1010011" "asm"="fcvt.l.d {rd}, {rs1}, rne" "asmonly" {
  ret void
}

define void @FCVT_L_D.2(i64 "reg"="x" %rd, i64 "reg"="d" %rs1) "enc"="110000100010 rs1:5 010 rd:5 1010011" "asm"="fcvt.l.d {rd}, {rs1}, rdn" "asmonly" {
  ret void
}

define void @FCVT_L_D.3(i64 "reg"="x" %rd, i64 "reg"="d" %rs1) "enc"="110000100010 rs1:5 011 rd:5 1010011" "asm"="fcvt.l.d {rd}, {rs1}, rup" "asmonly" {
  ret void
}

define void @FCVT_L_D.4(i64 "reg"="x" %rd, i64 "reg"="d" %rs1) "enc"="110000100010 rs1:5 100 rd:5 1010011" "asm"="fcvt.l.d {rd}, {rs1}, rmm" "asmonly" {
  ret void
}

define void @FCVT_L_D.5(i64 "reg"="x" %rd, i64 "reg"="d" %rs1) "enc"="110000100010 rs1:5 111 rd:5 1010011" "asm"="fcvt.l.d {rd}, {rs1}, dyn" "asmonly" "nodecode" {
  ret void
}

define void @FCVT_LU_D.0(i64 "reg"="x" %rd, i64 "reg"="d" %rs1) "enc"="110000100011 rs1:5 111 rd:5 1010011" "asm"="fcvt.lu.d {rd}, {rs1}" "asmonly" {
  ret void
}

define void @FCVT_LU_D.1(i64 "reg"="x" %rd, i64 "reg"="d" %rs1) "enc"="110000100011 rs1:5 000 rd:5 1010011" "asm"="fcvt.lu.d {rd}, {rs1}, rne" "asmonly" {
  ret void
}

define void @FCVT_LU_D.2(i64 "reg"="x" %rd, i64 "reg"="d" %rs1) "enc"="110000100011 rs1:5 010 rd:5 1010011" "asm"="fcvt.lu.d {rd}, {rs1}, rdn" "asmonly" {
  ret void
}

define void @FCVT_LU_D.3(i64 "reg"="x" %rd, i64 "reg"="d" %rs1) "enc"="110000100011 rs1:5 011 rd:5 1010011" "asm"="fcvt.lu.d {rd}, {rs1}, rup" "asmonly" {
  ret void
}

define void @FCVT_LU_D.4(i64 "reg"="x" %rd, i64 "reg"="d" %rs1) "enc"="110000100011 rs1:5 100 rd:5 1010011" "asm"="fcvt.lu.d {rd}, {rs1}, rmm" "asmonly" {
  ret void
}

define void @FCVT_LU_D.5(i64 "reg"="x" %rd, i64 "reg"="d" %rs1) "enc"="110000100011 rs1:5 111 rd:5 1010011" "asm"="fcvt.lu.d {rd}, {rs1}, dyn" "asmonly" "nodecode" {
  ret void
}

define void @FCVT_D_L.0(i64 "reg"="d" %rd, i64 "reg"="x" %rs1) "enc"="110100100010 rs1:5 000 rd:5 1010011" "asm"="fcvt.d.l {rd}, {rs1}, rne" "asmonly" {
  ret void
}

define void @FCVT_D_L.1(i64 "reg"="d" %rd, i64 "reg"="x" %rs1) "enc"="110100100010 rs1:5 001 rd:5 1010011" "asm"="fcvt.d.l {rd}, {rs1}, rtz" "asmonly" {
  ret void
}

define void @FCVT_D_L.2(i64 "reg"="d" %rd, i64 "reg"="x" %rs1) "enc"="110100100010 rs1:5 010 rd:5 1010011" "asm"="fcvt.d.l {rd}, {rs1}, rdn" "asmonly" {
  ret void
}

define void @FCVT_D_L.3(i64 "reg"="d" %rd, i64 "reg"="x" %rs1) "enc"="110100100010 rs1:5 011 rd:5 1010011" "asm"="fcvt.d.l {rd}, {rs1}, rup" "asmonly" {
  ret void
}

define void @FCVT_D_L.4(i64 "reg"="d" %rd, i64 "reg"="x" %rs1) "enc"="110100100010 rs1:5 100 rd:5 1010011" "asm"="fcvt.d.l {rd}, {rs1}, rmm" "asmonly" {
  ret void
}

define void @FCVT_D_LU.0(i64 "reg"="d" %rd, i64 "reg"="x" %rs1) "enc"="110100100011 rs1:5 000 rd:5 1010011" "asm"="fcvt.d.lu {rd}, {rs1}, rne" "asmonly" {
  ret void
}

define void @FCVT_D_LU.1(i64 "reg"="d" %rd, i64 "reg"="x" %rs1) "enc"="110100100011 rs1:5 001 rd:5 1010011" "asm"="fcvt.d.lu {rd}, {rs1}, rtz" "asmonly" {
  ret void
}

define void @FCVT_D_LU.2(i64 "reg"="d" %rd, i64 "reg"="x" %rs1) "enc"="110100100011 rs1:5 010 rd:5 1010011" "asm"="fcvt.d.lu {rd}, {rs1}, rdn" "asmonly" {
  ret void
}

define void @FCVT_D_LU.3(i64 "reg"="d" %rd, i64 "reg"="x" %rs1) "enc"="110100100011 rs1:5 011 rd:5 1010011" "asm"="fcvt.d.lu {rd}, {rs1}, rup" "asmonly" {
  ret void
}

define void @FCVT_D_LU.4(i64 "reg"="d" %rd, i64 "reg"="x" %rs1) "enc"="110100100011 rs1:5 100 rd:5 1010011" "asm"="fcvt.d.lu {rd}, {rs1}, rmm" "asmonly" {
  ret void
}

define void @CSRRW.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1, i64 "imm" %csr) "enc"="csr:12 rs1:5 001 rd:5 1110011" "asm"="csrrw {rd}, {csr}, {rs1}" "asmonly" {
  ret void
}

define void @CSRRS.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1, i64 "imm" %csr) "enc"="csr:12 rs1:5 010 rd:5 1110011" "asm"="csrrs {rd}, {csr}, {rs1}" "asmonly" {
  ret void
}

define void @CSRRC.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1, i64 "imm" %csr) "enc"="csr:12 rs1:5 011 rd:5 1110011" "asm"="csrrc {rd}, {csr}, {rs1}" "asmonly" {
  ret void
}

define void @CSRRWI.0(i64 "reg"="x" %rd, i64 "imm" %csr, i64 "imm" %zimm5) "enc"="csr:12 zimm5:5 101 rd:5 1110011" "asm"="csrrwi {rd}, {csr}, {zimm5}" "asmonly" {
  ret void
}

define void @CSRRSI.0(i64 "reg"="x" %rd, i64 "imm" %csr, i64 "imm" %zimm5) "enc"="csr:12 zimm5:5 110 rd:5 1110011" "asm"="csrrsi {rd}, {csr}, {zimm5}" "asmonly" {
  ret void
}

define void @CSRRCI.0(i64 "reg"="x" %rd, i64 "imm" %csr, i64 "imm" %zimm5) "enc"="csr:12 zimm5:5 111 rd:5 1110011" "asm"="csrrci {rd}, {csr}, {zimm5}" "asmonly" {
  ret void
}

define void @FENCE_I.0() "enc"="00000000000000000001000000001111" "asm"="fence.i" "asmonly" {
  ret void
}

define void @SH1ADD.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1, i64 "reg"="x" %rs2) "enc"="0010000 rs2:5 rs1:5 010 rd:5 0110011" "asm"="sh1add {rd}, {rs1}, {rs2}" "asmonly" {
  ret void
}

define void @SH2ADD.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1, i64 "reg"="x" %rs2) "enc"="0010000 rs2:5 rs1:5 100 rd:5 0110011" "asm"="sh2add {rd}, {rs1}, {rs2}" "asmonly" {
  ret void
}

define void @SH3ADD.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1, i64 "reg"="x" %rs2) "enc"="0010000 rs2:5 rs1:5 110 rd:5 0110011" "asm"="sh3add {rd}, {rs1}, {rs2}" "asmonly" {
  ret void
}

define void @ADD_UW.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1, i64 "reg"="x" %rs2) "enc"="0000100 rs2:5 rs1:5 000 rd:5 0111011" "asm"="add.uw {rd}, {rs1}, {rs2}" "asmonly" {
  ret void
}

define void @SH1ADD_UW.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1, i64 "reg"="x" %rs2) "enc"="0010000 rs2:5 rs1:5 010 rd:5 0111011" "asm"="sh1add.uw {rd}, {rs1}, {rs2}" "asmonly" {
  ret void
}

define void @SH2ADD_UW.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1, i64 "reg"="x" %rs2) "enc"="0010000 rs2:5 rs1:5 100 rd:5 0111011" "asm"="sh2add.uw {rd}, {rs1}, {rs2}" "asmonly" {
  ret void
}

define void @SH3ADD_UW.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1, i64 "reg"="x" %rs2) "enc"="0010000 rs2:5 rs1:5 110 rd:5 0111011" "asm"="sh3add.uw {rd}, {rs1}, {rs2}" "asmonly" {
  ret void
}

define void @SLLI_UW.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1, i64 "imm" %shamtd) "enc"="000010 shamtd:6 rs1:5 001 rd:5 0011011" "asm"="slli.uw {rd}, {rs1}, {shamtd}" "asmonly" {
  ret void
}

define void @XNOR.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1, i64 "reg"="x" %rs2) "enc"="0100000 rs2:5 rs1:5 100 rd:5 0110011" "asm"="xnor {rd}, {rs1}, {rs2}" "asmonly" {
  ret void
}

define void @ROL.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1, i64 "reg"="x" %rs2) "enc"="0110000 rs2:5 rs1:5 001 rd:5 0110011" "asm"="rol {rd}, {rs1}, {rs2}" "asmonly" {
  ret void
}

define void @ROR.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1, i64 "reg"="x" %rs2) "enc"="0110000 rs2:5 rs1:5 101 rd:5 0110011" "asm"="ror {rd}, {rs1}, {rs2}" "asmonly" {
  ret void
}

define void @ORC_B.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1) "enc"="001010000111 rs1:5 101 rd:5 0010011" "asm"="orc.b {rd}, {rs1}" "asmonly" {
  ret void
}

define void @CLZW.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1) "enc"="011000000000 rs1:5 001 rd:5 0011011" "asm"="clzw {rd}, {rs1}" "asmonly" {
  ret void
}

define void @CTZW.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1) "enc"="011000000001 rs1:5 001 rd:5 0011011" "asm"="ctzw {rd}, {rs1}" "asmonly" {
  ret void
}

define void @CPOPW.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1) "enc"="011000000010 rs1:5 001 rd:5 0011011" "asm"="cpopw {rd}, {rs1}" "asmonly" {
  ret void
}

define void @ROLW.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1, i64 "reg"="x" %rs2) "enc"="0110000 rs2:5 rs1:5 001 rd:5 0111011" "asm"="rolw {rd}, {rs1}, {rs2}" "asmonly" {
  ret void
}

define void @RORW.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1, i64 "reg"="x" %rs2) "enc"="0110000 rs2:5 rs1:5 101 rd:5 0111011" "asm"="rorw {rd}, {rs1}, {rs2}" "asmonly" {
  ret void
}

define void @RORIW.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1, i64 "imm" %shamtw) "enc"="0110000 shamtw:5 rs1:5 101 rd:5 0011011" "asm"="roriw {rd}, {rs1}, {shamtw}" "asmonly" {
  ret void
}

define void @RORI.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1, i64 "imm" %shamtd) "enc"="011000 shamtd:6 rs1:5 101 rd:5 0010011" "asm"="rori {rd}, {rs1}, {shamtd}" "asmonly" {
  ret void
}

define void @BCLR.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1, i64 "reg"="x" %rs2) "enc"="0100100 rs2:5 rs1:5 001 rd:5 0110011" "asm"="bclr {rd}, {rs1}, {rs2}" "asmonly" {
  ret void
}

define void @BEXT.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1, i64 "reg"="x" %rs2) "enc"="0100100 rs2:5 rs1:5 101 rd:5 0110011" "asm"="bext {rd}, {rs1}, {rs2}" "asmonly" {
  ret void
}

define void @BINV.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1, i64 "reg"="x" %rs2) "enc"="0110100 rs2:5 rs1:5 001 rd:5 0110011" "asm"="binv {rd}, {rs1}, {rs2}" "asmonly" {
  ret void
}

define void @BSET.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1, i64 "reg"="x" %rs2) "enc"="0010100 rs2:5 rs1:5 001 rd:5 0110011" "asm"="bset {rd}, {rs1}, {rs2}" "asmonly" {
  ret void
}

define void @BCLRI.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1, i64 "imm" %shamtd) "enc"="010010 shamtd:6 rs1:5 001 rd:5 0010011" "asm"="bclri {rd}, {rs1}, {shamtd}" "asmonly" {
  ret void
}

define void @BEXTI.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1, i64 "imm" %shamtd) "enc"="010010 shamtd:6 rs1:5 101 rd:5 0010011" "asm"="bexti {rd}, {rs1}, {shamtd}" "asmonly" {
  ret void
}

define void @BINVI.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1, i64 "imm" %shamtd) "enc"="011010 shamtd:6 rs1:5 001 rd:5 0010011" "asm"="binvi {rd}, {rs1}, {shamtd}" "asmonly" {
  ret void
}

define void @BSETI.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1, i64 "imm" %shamtd) "enc"="001010 shamtd:6 rs1:5 001 rd:5 0010011" "asm"="bseti {rd}, {rs1}, {shamtd}" "asmonly" {
  ret void
}

define void @CLMUL.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1, i64 "reg"="x" %rs2) "enc"="0000101 rs2:5 rs1:5 001 rd:5 0110011" "asm"="clmul {rd}, {rs1}, {rs2}" "asmonly" {
  ret void
}

define void @CLMULR.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1, i64 "reg"="x" %rs2) "enc"="0000101 rs2:5 rs1:5 010 rd:5 0110011" "asm"="clmulr {rd}, {rs1}, {rs2}" "asmonly" {
  ret void
}

define void @CLMULH.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1, i64 "reg"="x" %rs2) "enc"="0000101 rs2:5 rs1:5 011 rd:5 0110011" "asm"="clmulh {rd}, {rs1}, {rs2}" "asmonly" {
  ret void
}

define void @PACK.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1, i64 "reg"="x" %rs2) "enc"="0000100 rs2:5 rs1:5 100 rd:5 0110011" "asm"="pack {rd}, {rs1}, {rs2}" "asmonly" {
  ret void
}

define void @PACKH.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1, i64 "reg"="x" %rs2) "enc"="0000100 rs2:5 rs1:5 111 rd:5 0110011" "asm"="packh {rd}, {rs1}, {rs2}" "asmonly" {
  ret void
}

define void @BREV8.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1) "enc"="011010000111 rs1:5 101 rd:5 0010011" "asm"="brev8 {rd}, {rs1}" "asmonly" {
  ret void
}

define void @PACKW.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1, i64 "reg"="x" %rs2) "enc"="0000100 rs2:5 rs1:5 100 rd:5 0111011" "asm"="packw {rd}, {rs1}, {rs2}" "asmonly" {
  ret void
}

define void @XPERM4.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1, i64 "reg"="x" %rs2) "enc"="0010100 rs2:5 rs1:5 010 rd:5 0110011" "asm"="xperm4 {rd}, {rs1}, {rs2}" "asmonly" {
  ret void
}

define void @XPERM8.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1, i64 "reg"="x" %rs2) "enc"="0010100 rs2:5 rs1:5 100 rd:5 0110011" "asm"="xperm8 {rd}, {rs1}, {rs2}" "asmonly" {
  ret void
}

define void @CZERO_EQZ.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1, i64 "reg"="x" %rs2) "enc"="0000111 rs2:5 rs1:5 101 rd:5 0110011" "asm"="czero.eqz {rd}, {rs1}, {rs2}" "asmonly" {
  ret void
}

define void @CZERO_NEZ.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1, i64 "reg"="x" %rs2) "enc"="0000111 rs2:5 rs1:5 111 rd:5 0110011" "asm"="czero.nez {rd}, {rs1}, {rs2}" "asmonly" {
  ret void
}

define void @FLI_S.0(i64 "reg"="d" %rd) "enc"="11110000000100000000 rd:5 1010011" "asm"="fli.s {rd}, -1.0" "asmonly" {
  ret void
}

define void @FLI_S.1(i64 "reg"="d" %rd) "enc"="11110000000100001000 rd:5 1010011" "asm"="fli.s {rd}, min" "asmonly" {
  ret void
}

define void @FLI_S.2(i64 "reg"="d" %rd) "enc"="11110000000100010000 rd:5 1010011" "asm"="fli.s {rd}, 1.52587890625e-05" "asmonly" {
  ret void
}

define void @FLI_S.3(i64 "reg"="d" %rd) "enc"="11110000000100011000 rd:5 1010011" "asm"="fli.s {rd}, 3.0517578125e-05" "asmonly" {
  ret void
}

define void @FLI_S.4(i64 "reg"="d" %rd) "enc"="11110000000100100000 rd:5 1010011" "asm"="fli.s {rd}, 0.00390625" "asmonly" {
  ret void
}

define void @FLI_S.5(i64 "reg"="d" %rd) "enc"="11110000000100101000 rd:5 1010011" "asm"="fli.s {rd}, 0.0078125" "asmonly" {
  ret void
}

define void @FLI_S.6(i64 "reg"="d" %rd) "enc"="11110000000100110000 rd:5 1010011" "asm"="fli.s {rd}, 0.0625" "asmonly" {
  ret void
}

define void @FLI_S.7(i64 "reg"="d" %rd) "enc"="11110000000100111000 rd:5 1010011" "asm"="fli.s {rd}, 0.125" "asmonly" {
  ret void
}

define void @FLI_S.8(i64 "reg"="d" %rd) "enc"="11110000000101000000 rd:5 1010011" "asm"="fli.s {rd}, 0.25" "asmonly" {
  ret void
}

define void @FLI_S.9(i64 "reg"="d" %rd) "enc"="11110000000101001000 rd:5 1010011" "asm"="fli.s {rd}, 0.3125" "asmonly" {
  ret void
}

define void @FLI_S.10(i64 "reg"="d" %rd) "enc"="11110000000101010000 rd:5 1010011" "asm"="fli.s {rd}, 0.375" "asmonly" {
  ret void
}

define void @FLI_S.11(i64 "reg"="d" %rd) "enc"="11110000000101011000 rd:5 1010011" "asm"="fli.s {rd}, 0.4375" "asmonly" {
  ret void
}

define void @FLI_S.12(i64 "reg"="d" %rd) "enc"="11110000000101100000 rd:5 1010011" "asm"="fli.s {rd}, 0.5" "asmonly" {
  ret void
}

define void @FLI_S.13(i64 "reg"="d" %rd) "enc"="11110000000101101000 rd:5 1010011" "asm"="fli.s {rd}, 0.625" "asmonly" {
  ret void
}

define void @FLI_S.14(i64 "reg"="d" %rd) "enc"="11110000000101110000 rd:5 1010011" "asm"="fli.s {rd}, 0.75" "asmonly" {
  ret void
}

define void @FLI_S.15(i64 "reg"="d" %rd) "enc"="11110000000101111000 rd:5 1010011" "asm"="fli.s {rd}, 0.875" "asmonly" {
  ret void
}

define void @FLI_S.16(i64 "reg"="d" %rd) "enc"="11110000000110000000 rd:5 1010011" "asm"="fli.s {rd}, 1.0" "asmonly" {
  ret void
}

define void @FLI_S.17(i64 "reg"="d" %rd) "enc"="11110000000110001000 rd:5 1010011" "asm"="fli.s {rd}, 1.25" "asmonly" {
  ret void
}

define void @FLI_S.18(i64 "reg"="d" %rd) "enc"="11110000000110010000 rd:5 1010011" "asm"="fli.s {rd}, 1.5" "asmonly" {
  ret void
}

define void @FLI_S.19(i64 "reg"="d" %rd) "enc"="11110000000110011000 rd:5 1010011" "asm"="fli.s {rd}, 1.75" "asmonly" {
  ret void
}

define void @FLI_S.20(i64 "reg"="d" %rd) "enc"="11110000000110100000 rd:5 1010011" "asm"="fli.s {rd}, 2.0" "asmonly" {
  ret void
}

define void @FLI_S.21(i64 "reg"="d" %rd) "enc"="11110000000110101000 rd:5 1010011" "asm"="fli.s {rd}, 2.5" "asmonly" {
  ret void
}

define void @FLI_S.22(i64 "reg"="d" %rd) "enc"="11110000000110110000 rd:5 1010011" "asm"="fli.s {rd}, 3.0" "asmonly" {
  ret void
}

define void @FLI_S.23(i64 "reg"="d" %rd) "enc"="11110000000110111000 rd:5 1010011" "asm"="fli.s {rd}, 4.0" "asmonly" {
  ret void
}

define void @FLI_S.24(i64 "reg"="d" %rd) "enc"="11110000000111000000 rd:5 1010011" "asm"="fli.s {rd}, 8.0" "asmonly" {
  ret void
}

define void @FLI_S.25(i64 "reg"="d" %rd) "enc"="11110000000111001000 rd:5 1010011" "asm"="fli.s {rd}, 16.0" "asmonly" {
  ret void
}

define void @FLI_S.26(i64 "reg"="d" %rd) "enc"="11110000000111010000 rd:5 1010011" "asm"="fli.s {rd}, 128.0" "asmonly" {
  ret void
}

define void @FLI_S.27(i64 "reg"="d" %rd) "enc"="11110000000111011000 rd:5 1010011" "asm"="fli.s {rd}, 256.0" "asmonly" {
  ret void
}

define void @FLI_S.28(i64 "reg"="d" %rd) "enc"="11110000000111100000 rd:5 1010011" "asm"="fli.s {rd}, 32768.0" "asmonly" {
  ret void
}

define void @FLI_S.29(i64 "reg"="d" %rd) "enc"="11110000000111101000 rd:5 1010011" "asm"="fli.s {rd}, 65536.0" "asmonly" {
  ret void
}

define void @FLI_S.30(i64 "reg"="d" %rd) "enc"="11110000000111110000 rd:5 1010011" "asm"="fli.s {rd}, inf" "asmonly" {
  ret void
}

define void @FLI_S.31(i64 "reg"="d" %rd) "enc"="11110000000111111000 rd:5 1010011" "asm"="fli.s {rd}, nan" "asmonly" {
  ret void
}

define void @FMINM_S.0(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2) "enc"="0010100 rs2:5 rs1:5 010 rd:5 1010011" "asm"="fminm.s {rd}, {rs1}, {rs2}" "asmonly" {
  ret void
}

define void @FMAXM_S.0(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2) "enc"="0010100 rs2:5 rs1:5 011 rd:5 1010011" "asm"="fmaxm.s {rd}, {rs1}, {rs2}" "asmonly" {
  ret void
}

define void @FROUND_S.0(i64 "reg"="d" %rd, i64 "reg"="d" %rs1) "enc"="010000000100 rs1:5 111 rd:5 1010011" "asm"="fround.s {rd}, {rs1}" "asmonly" {
  ret void
}

define void @FROUND_S.1(i64 "reg"="d" %rd, i64 "reg"="d" %rs1) "enc"="010000000100 rs1:5 000 rd:5 1010011" "asm"="fround.s {rd}, {rs1}, rne" "asmonly" {
  ret void
}

define void @FROUND_S.2(i64 "reg"="d" %rd, i64 "reg"="d" %rs1) "enc"="010000000100 rs1:5 001 rd:5 1010011" "asm"="fround.s {rd}, {rs1}, rtz" "asmonly" {
  ret void
}

define void @FROUND_S.3(i64 "reg"="d" %rd, i64 "reg"="d" %rs1) "enc"="010000000100 rs1:5 010 rd:5 1010011" "asm"="fround.s {rd}, {rs1}, rdn" "asmonly" {
  ret void
}

define void @FROUND_S.4(i64 "reg"="d" %rd, i64 "reg"="d" %rs1) "enc"="010000000100 rs1:5 011 rd:5 1010011" "asm"="fround.s {rd}, {rs1}, rup" "asmonly" {
  ret void
}

define void @FROUND_S.5(i64 "reg"="d" %rd, i64 "reg"="d" %rs1) "enc"="010000000100 rs1:5 100 rd:5 1010011" "asm"="fround.s {rd}, {rs1}, rmm" "asmonly" {
  ret void
}

define void @FROUND_S.6(i64 "reg"="d" %rd, i64 "reg"="d" %rs1) "enc"="010000000100 rs1:5 111 rd:5 1010011" "asm"="fround.s {rd}, {rs1}, dyn" "asmonly" "nodecode" {
  ret void
}

define void @FROUNDNX_S.0(i64 "reg"="d" %rd, i64 "reg"="d" %rs1) "enc"="010000000101 rs1:5 111 rd:5 1010011" "asm"="froundnx.s {rd}, {rs1}" "asmonly" {
  ret void
}

define void @FROUNDNX_S.1(i64 "reg"="d" %rd, i64 "reg"="d" %rs1) "enc"="010000000101 rs1:5 000 rd:5 1010011" "asm"="froundnx.s {rd}, {rs1}, rne" "asmonly" {
  ret void
}

define void @FROUNDNX_S.2(i64 "reg"="d" %rd, i64 "reg"="d" %rs1) "enc"="010000000101 rs1:5 001 rd:5 1010011" "asm"="froundnx.s {rd}, {rs1}, rtz" "asmonly" {
  ret void
}

define void @FROUNDNX_S.3(i64 "reg"="d" %rd, i64 "reg"="d" %rs1) "enc"="010000000101 rs1:5 010 rd:5 1010011" "asm"="froundnx.s {rd}, {rs1}, rdn" "asmonly" {
  ret void
}

define void @FROUNDNX_S.4(i64 "reg"="d" %rd, i64 "reg"="d" %rs1) "enc"="010000000101 rs1:5 011 rd:5 1010011" "asm"="froundnx.s {rd}, {rs1}, rup" "asmonly" {
  ret void
}

define void @FROUNDNX_S.5(i64 "reg"="d" %rd, i64 "reg"="d" %rs1) "enc"="010000000101 rs1:5 100 rd:5 1010011" "asm"="froundnx.s {rd}, {rs1}, rmm" "asmonly" {
  ret void
}

define void @FROUNDNX_S.6(i64 "reg"="d" %rd, i64 "reg"="d" %rs1) "enc"="010000000101 rs1:5 111 rd:5 1010011" "asm"="froundnx.s {rd}, {rs1}, dyn" "asmonly" "nodecode" {
  ret void
}

define void @FLEQ_S.0(i64 "reg"="x" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2) "enc"="1010000 rs2:5 rs1:5 100 rd:5 1010011" "asm"="fleq.s {rd}, {rs1}, {rs2}" "asmonly" {
  ret void
}

define void @FLTQ_S.0(i64 "reg"="x" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2) "enc"="1010000 rs2:5 rs1:5 101 rd:5 1010011" "asm"="fltq.s {rd}, {rs1}, {rs2}" "asmonly" {
  ret void
}

define void @FLI_D.0(i64 "reg"="d" %rd) "enc"="11110010000100000000 rd:5 1010011" "asm"="fli.d {rd}, -1.0" "asmonly" {
  ret void
}

define void @FLI_D.1(i64 "reg"="d" %rd) "enc"="11110010000100001000 rd:5 1010011" "asm"="fli.d {rd}, min" "asmonly" {
  ret void
}

define void @FLI_D.2(i64 "reg"="d" %rd) "enc"="11110010000100010000 rd:5 1010011" "asm"="fli.d {rd}, 1.52587890625e-05" "asmonly" {
  ret void
}

define void @FLI_D.3(i64 "reg"="d" %rd) "enc"="11110010000100011000 rd:5 1010011" "asm"="fli.d {rd}, 3.0517578125e-05" "asmonly" {
  ret void
}

define void @FLI_D.4(i64 "reg"="d" %rd) "enc"="11110010000100100000 rd:5 1010011" "asm"="fli.d {rd}, 0.00390625" "asmonly" {
  ret void
}

define void @FLI_D.5(i64 "reg"="d" %rd) "enc"="11110010000100101000 rd:5 1010011" "asm"="fli.d {rd}, 0.0078125" "asmonly" {
  ret void
}

define void @FLI_D.6(i64 "reg"="d" %rd) "enc"="11110010000100110000 rd:5 1010011" "asm"="fli.d {rd}, 0.0625" "asmonly" {
  ret void
}

define void @FLI_D.7(i64 "reg"="d" %rd) "enc"="11110010000100111000 rd:5 1010011" "asm"="fli.d {rd}, 0.125" "asmonly" {
  ret void
}

define void @FLI_D.8(i64 "reg"="d" %rd) "enc"="11110010000101000000 rd:5 1010011" "asm"="fli.d {rd}, 0.25" "asmonly" {
  ret void
}

define void @FLI_D.9(i64 "reg"="d" %rd) "enc"="11110010000101001000 rd:5 1010011" "asm"="fli.d {rd}, 0.3125" "asmonly" {
  ret void
}

define void @FLI_D.10(i64 "reg"="d" %rd) "enc"="11110010000101010000 rd:5 1010011" "asm"="fli.d {rd}, 0.375" "asmonly" {
  ret void
}

define void @FLI_D.11(i64 "reg"="d" %rd) "enc"="11110010000101011000 rd:5 1010011" "asm"="fli.d {rd}, 0.4375" "asmonly" {
  ret void
}

define void @FLI_D.12(i64 "reg"="d" %rd) "enc"="11110010000101100000 rd:5 1010011" "asm"="fli.d {rd}, 0.5" "asmonly" {
  ret void
}

define void @FLI_D.13(i64 "reg"="d" %rd) "enc"="11110010000101101000 rd:5 1010011" "asm"="fli.d {rd}, 0.625" "asmonly" {
  ret void
}

define void @FLI_D.14(i64 "reg"="d" %rd) "enc"="11110010000101110000 rd:5 1010011" "asm"="fli.d {rd}, 0.75" "asmonly" {
  ret void
}

define void @FLI_D.15(i64 "reg"="d" %rd) "enc"="11110010000101111000 rd:5 1010011" "asm"="fli.d {rd}, 0.875" "asmonly" {
  ret void
}

define void @FLI_D.16(i64 "reg"="d" %rd) "enc"="11110010000110000000 rd:5 1010011" "asm"="fli.d {rd}, 1.0" "asmonly" {
  ret void
}

define void @FLI_D.17(i64 "reg"="d" %rd) "enc"="11110010000110001000 rd:5 1010011" "asm"="fli.d {rd}, 1.25" "asmonly" {
  ret void
}

define void @FLI_D.18(i64 "reg"="d" %rd) "enc"="11110010000110010000 rd:5 1010011" "asm"="fli.d {rd}, 1.5" "asmonly" {
  ret void
}

define void @FLI_D.19(i64 "reg"="d" %rd) "enc"="11110010000110011000 rd:5 1010011" "asm"="fli.d {rd}, 1.75" "asmonly" {
  ret void
}

define void @FLI_D.20(i64 "reg"="d" %rd) "enc"="11110010000110100000 rd:5 1010011" "asm"="fli.d {rd}, 2.0" "asmonly" {
  ret void
}

define void @FLI_D.21(i64 "reg"="d" %rd) "enc"="11110010000110101000 rd:5 1010011" "asm"="fli.d {rd}, 2.5" "asmonly" {
  ret void
}

define void @FLI_D.22(i64 "reg"="d" %rd) "enc"="11110010000110110000 rd:5 1010011" "asm"="fli.d {rd}, 3.0" "asmonly" {
  ret void
}

define void @FLI_D.23(i64 "reg"="d" %rd) "enc"="11110010000110111000 rd:5 1010011" "asm"="fli.d {rd}, 4.0" "asmonly" {
  ret void
}

define void @FLI_D.24(i64 "reg"="d" %rd) "enc"="11110010000111000000 rd:5 1010011" "asm"="fli.d {rd}, 8.0" "asmonly" {
  ret void
}

define void @FLI_D.25(i64 "reg"="d" %rd) "enc"="11110010000111001000 rd:5 1010011" "asm"="fli.d {rd}, 16.0" "asmonly" {
  ret void
}

define void @FLI_D.26(i64 "reg"="d" %rd) "enc"="11110010000111010000 rd:5 1010011" "asm"="fli.d {rd}, 128.0" "asmonly" {
  ret void
}

define void @FLI_D.27(i64 "reg"="d" %rd) "enc"="11110010000111011000 rd:5 1010011" "asm"="fli.d {rd}, 256.0" "asmonly" {
  ret void
}

define void @FLI_D.28(i64 "reg"="d" %rd) "enc"="11110010000111100000 rd:5 1010011" "asm"="fli.d {rd}, 32768.0" "asmonly" {
  ret void
}

define void @FLI_D.29(i64 "reg"="d" %rd) "enc"="11110010000111101000 rd:5 1010011" "asm"="fli.d {rd}, 65536.0" "asmonly" {
  ret void
}

define void @FLI_D.30(i64 "reg"="d" %rd) "enc"="11110010000111110000 rd:5 1010011" "asm"="fli.d {rd}, inf" "asmonly" {
  ret void
}

define void @FLI_D.31(i64 "reg"="d" %rd) "enc"="11110010000111111000 rd:5 1010011" "asm"="fli.d {rd}, nan" "asmonly" {
  ret void
}

define void @FMINM_D.0(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2) "enc"="0010101 rs2:5 rs1:5 010 rd:5 1010011" "asm"="fminm.d {rd}, {rs1}, {rs2}" "asmonly" {
  ret void
}

define void @FMAXM_D.0(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2) "enc"="0010101 rs2:5 rs1:5 011 rd:5 1010011" "asm"="fmaxm.d {rd}, {rs1}, {rs2}" "asmonly" {
  ret void
}

define void @FROUND_D.0(i64 "reg"="d" %rd, i64 "reg"="d" %rs1) "enc"="010000100100 rs1:5 111 rd:5 1010011" "asm"="fround.d {rd}, {rs1}" "asmonly" {
  ret void
}

define void @FROUND_D.1(i64 "reg"="d" %rd, i64 "reg"="d" %rs1) "enc"="010000100100 rs1:5 000 rd:5 1010011" "asm"="fround.d {rd}, {rs1}, rne" "asmonly" {
  ret void
}

define void @FROUND_D.2(i64 "reg"="d" %rd, i64 "reg"="d" %rs1) "enc"="010000100100 rs1:5 001 rd:5 1010011" "asm"="fround.d {rd}, {rs1}, rtz" "asmonly" {
  ret void
}

define void @FROUND_D.3(i64 "reg"="d" %rd, i64 "reg"="d" %rs1) "enc"="010000100100 rs1:5 010 rd:5 1010011" "asm"="fround.d {rd}, {rs1}, rdn" "asmonly" {
  ret void
}

define void @FROUND_D.4(i64 "reg"="d" %rd, i64 "reg"="d" %rs1) "enc"="010000100100 rs1:5 011 rd:5 1010011" "asm"="fround.d {rd}, {rs1}, rup" "asmonly" {
  ret void
}

define void @FROUND_D.5(i64 "reg"="d" %rd, i64 "reg"="d" %rs1) "enc"="010000100100 rs1:5 100 rd:5 1010011" "asm"="fround.d {rd}, {rs1}, rmm" "asmonly" {
  ret void
}

define void @FROUND_D.6(i64 "reg"="d" %rd, i64 "reg"="d" %rs1) "enc"="010000100100 rs1:5 111 rd:5 1010011" "asm"="fround.d {rd}, {rs1}, dyn" "asmonly" "nodecode" {
  ret void
}

define void @FROUNDNX_D.0(i64 "reg"="d" %rd, i64 "reg"="d" %rs1) "enc"="010000100101 rs1:5 111 rd:5 1010011" "asm"="froundnx.d {rd}, {rs1}" "asmonly" {
  ret void
}

define void @FROUNDNX_D.1(i64 "reg"="d" %rd, i64 "reg"="d" %rs1) "enc"="010000100101 rs1:5 000 rd:5 1010011" "asm"="froundnx.d {rd}, {rs1}, rne" "asmonly" {
  ret void
}

define void @FROUNDNX_D.2(i64 "reg"="d" %rd, i64 "reg"="d" %rs1) "enc"="010000100101 rs1:5 001 rd:5 1010011" "asm"="froundnx.d {rd}, {rs1}, rtz" "asmonly" {
  ret void
}

define void @FROUNDNX_D.3(i64 "reg"="d" %rd, i64 "reg"="d" %rs1) "enc"="010000100101 rs1:5 010 rd:5 1010011" "asm"="froundnx.d {rd}, {rs1}, rdn" "asmonly" {
  ret void
}

define void @FROUNDNX_D.4(i64 "reg"="d" %rd, i64 "reg"="d" %rs1) "enc"="010000100101 rs1:5 011 rd:5 1010011" "asm"="froundnx.d {rd}, {rs1}, rup" "asmonly" {
  ret void
}

define void @FROUNDNX_D.5(i64 "reg"="d" %rd, i64 "reg"="d" %rs1) "enc"="010000100101 rs1:5 100 rd:5 1010011" "asm"="froundnx.d {rd}, {rs1}, rmm" "asmonly" {
  ret void
}

define void @FROUNDNX_D.6(i64 "reg"="d" %rd, i64 "reg"="d" %rs1) "enc"="010000100101 rs1:5 111 rd:5 1010011" "asm"="froundnx.d {rd}, {rs1}, dyn" "asmonly" "nodecode" {
  ret void
}

define void @FCVTMOD_W_D.0(i64 "reg"="x" %rd, i64 "reg"="d" %rs1) "enc"="110000101000 rs1:5 001 rd:5 1010011" "asm"="fcvtmod.w.d {rd}, {rs1}, rtz" "asmonly" {
  ret void
}

define void @FLEQ_D.0(i64 "reg"="x" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2) "enc"="1010001 rs2:5 rs1:5 100 rd:5 1010011" "asm"="fleq.d {rd}, {rs1}, {rs2}" "asmonly" {
  ret void
}

define void @FLTQ_D.0(i64 "reg"="x" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2) "enc"="1010001 rs2:5 rs1:5 101 rd:5 1010011" "asm"="fltq.d {rd}, {rs1}, {rs2}" "asmonly" {
  ret void
}

define void @FMADD_H.0(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2, i64 "reg"="d" %rs3) "enc"="rs3:5 10 rs2:5 rs1:5 111 rd:5 1000011" "asm"="fmadd.h {rd}, {rs1}, {rs2}, {rs3}" "asmonly" {
  ret void
}

define void @FMADD_H.1(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2, i64 "reg"="d" %rs3) "enc"="rs3:5 10 rs2:5 rs1:5 000 rd:5 1000011" "asm"="fmadd.h {rd}, {rs1}, {rs2}, {rs3}, rne" "asmonly" {
  ret void
}

define void @FMADD_H.2(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2, i64 "reg"="d" %rs3) "enc"="rs3:5 10 rs2:5 rs1:5 001 rd:5 1000011" "asm"="fmadd.h {rd}, {rs1}, {rs2}, {rs3}, rtz" "asmonly" {
  ret void
}

define void @FMADD_H.3(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2, i64 "reg"="d" %rs3) "enc"="rs3:5 10 rs2:5 rs1:5 010 rd:5 1000011" "asm"="fmadd.h {rd}, {rs1}, {rs2}, {rs3}, rdn" "asmonly" {
  ret void
}

define void @FMADD_H.4(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2, i64 "reg"="d" %rs3) "enc"="rs3:5 10 rs2:5 rs1:5 011 rd:5 1000011" "asm"="fmadd.h {rd}, {rs1}, {rs2}, {rs3}, rup" "asmonly" {
  ret void
}

define void @FMADD_H.5(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2, i64 "reg"="d" %rs3) "enc"="rs3:5 10 rs2:5 rs1:5 100 rd:5 1000011" "asm"="fmadd.h {rd}, {rs1}, {rs2}, {rs3}, rmm" "asmonly" {
  ret void
}

define void @FMADD_H.6(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2, i64 "reg"="d" %rs3) "enc"="rs3:5 10 rs2:5 rs1:5 111 rd:5 1000011" "asm"="fmadd.h {rd}, {rs1}, {rs2}, {rs3}, dyn" "asmonly" "nodecode" {
  ret void
}

define void @FMSUB_H.0(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2, i64 "reg"="d" %rs3) "enc"="rs3:5 10 rs2:5 rs1:5 111 rd:5 1000111" "asm"="fmsub.h {rd}, {rs1}, {rs2}, {rs3}" "asmonly" {
  ret void
}

define void @FMSUB_H.1(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2, i64 "reg"="d" %rs3) "enc"="rs3:5 10 rs2:5 rs1:5 000 rd:5 1000111" "asm"="fmsub.h {rd}, {rs1}, {rs2}, {rs3}, rne" "asmonly" {
  ret void
}

define void @FMSUB_H.2(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2, i64 "reg"="d" %rs3) "enc"="rs3:5 10 rs2:5 rs1:5 001 rd:5 1000111" "asm"="fmsub.h {rd}, {rs1}, {rs2}, {rs3}, rtz" "asmonly" {
  ret void
}

define void @FMSUB_H.3(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2, i64 "reg"="d" %rs3) "enc"="rs3:5 10 rs2:5 rs1:5 010 rd:5 1000111" "asm"="fmsub.h {rd}, {rs1}, {rs2}, {rs3}, rdn" "asmonly" {
  ret void
}

define void @FMSUB_H.4(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2, i64 "reg"="d" %rs3) "enc"="rs3:5 10 rs2:5 rs1:5 011 rd:5 1000111" "asm"="fmsub.h {rd}, {rs1}, {rs2}, {rs3}, rup" "asmonly" {
  ret void
}

define void @FMSUB_H.5(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2, i64 "reg"="d" %rs3) "enc"="rs3:5 10 rs2:5 rs1:5 100 rd:5 1000111" "asm"="fmsub.h {rd}, {rs1}, {rs2}, {rs3}, rmm" "asmonly" {
  ret void
}

define void @FMSUB_H.6(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2, i64 "reg"="d" %rs3) "enc"="rs3:5 10 rs2:5 rs1:5 111 rd:5 1000111" "asm"="fmsub.h {rd}, {rs1}, {rs2}, {rs3}, dyn" "asmonly" "nodecode" {
  ret void
}

define void @FNMSUB_H.0(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2, i64 "reg"="d" %rs3) "enc"="rs3:5 10 rs2:5 rs1:5 111 rd:5 1001011" "asm"="fnmsub.h {rd}, {rs1}, {rs2}, {rs3}" "asmonly" {
  ret void
}

define void @FNMSUB_H.1(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2, i64 "reg"="d" %rs3) "enc"="rs3:5 10 rs2:5 rs1:5 000 rd:5 1001011" "asm"="fnmsub.h {rd}, {rs1}, {rs2}, {rs3}, rne" "asmonly" {
  ret void
}

define void @FNMSUB_H.2(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2, i64 "reg"="d" %rs3) "enc"="rs3:5 10 rs2:5 rs1:5 001 rd:5 1001011" "asm"="fnmsub.h {rd}, {rs1}, {rs2}, {rs3}, rtz" "asmonly" {
  ret void
}

define void @FNMSUB_H.3(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2, i64 "reg"="d" %rs3) "enc"="rs3:5 10 rs2:5 rs1:5 010 rd:5 1001011" "asm"="fnmsub.h {rd}, {rs1}, {rs2}, {rs3}, rdn" "asmonly" {
  ret void
}

define void @FNMSUB_H.4(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2, i64 "reg"="d" %rs3) "enc"="rs3:5 10 rs2:5 rs1:5 011 rd:5 1001011" "asm"="fnmsub.h {rd}, {rs1}, {rs2}, {rs3}, rup" "asmonly" {
  ret void
}

define void @FNMSUB_H.5(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2, i64 "reg"="d" %rs3) "enc"="rs3:5 10 rs2:5 rs1:5 100 rd:5 1001011" "asm"="fnmsub.h {rd}, {rs1}, {rs2}, {rs3}, rmm" "asmonly" {
  ret void
}

define void @FNMSUB_H.6(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2, i64 "reg"="d" %rs3) "enc"="rs3:5 10 rs2:5 rs1:5 111 rd:5 1001011" "asm"="fnmsub.h {rd}, {rs1}, {rs2}, {rs3}, dyn" "asmonly" "nodecode" {
  ret void
}

define void @FNMADD_H.0(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2, i64 "reg"="d" %rs3) "enc"="rs3:5 10 rs2:5 rs1:5 111 rd:5 1001111" "asm"="fnmadd.h {rd}, {rs1}, {rs2}, {rs3}" "asmonly" {
  ret void
}

define void @FNMADD_H.1(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2, i64 "reg"="d" %rs3) "enc"="rs3:5 10 rs2:5 rs1:5 000 rd:5 1001111" "asm"="fnmadd.h {rd}, {rs1}, {rs2}, {rs3}, rne" "asmonly" {
  ret void
}

define void @FNMADD_H.2(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2, i64 "reg"="d" %rs3) "enc"="rs3:5 10 rs2:5 rs1:5 001 rd:5 1001111" "asm"="fnmadd.h {rd}, {rs1}, {rs2}, {rs3}, rtz" "asmonly" {
  ret void
}

define void @FNMADD_H.3(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2, i64 "reg"="d" %rs3) "enc"="rs3:5 10 rs2:5 rs1:5 010 rd:5 1001111" "asm"="fnmadd.h {rd}, {rs1}, {rs2}, {rs3}, rdn" "asmonly" {
  ret void
}

define void @FNMADD_H.4(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2, i64 "reg"="d" %rs3) "enc"="rs3:5 10 rs2:5 rs1:5 011 rd:5 1001111" "asm"="fnmadd.h {rd}, {rs1}, {rs2}, {rs3}, rup" "asmonly" {
  ret void
}

define void @FNMADD_H.5(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2, i64 "reg"="d" %rs3) "enc"="rs3:5 10 rs2:5 rs1:5 100 rd:5 1001111" "asm"="fnmadd.h {rd}, {rs1}, {rs2}, {rs3}, rmm" "asmonly" {
  ret void
}

define void @FNMADD_H.6(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2, i64 "reg"="d" %rs3) "enc"="rs3:5 10 rs2:5 rs1:5 111 rd:5 1001111" "asm"="fnmadd.h {rd}, {rs1}, {rs2}, {rs3}, dyn" "asmonly" "nodecode" {
  ret void
}

define void @FADD_H.0(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2) "enc"="0000010 rs2:5 rs1:5 111 rd:5 1010011" "asm"="fadd.h {rd}, {rs1}, {rs2}" "asmonly" {
  ret void
}

define void @FADD_H.1(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2) "enc"="0000010 rs2:5 rs1:5 000 rd:5 1010011" "asm"="fadd.h {rd}, {rs1}, {rs2}, rne" "asmonly" {
  ret void
}

define void @FADD_H.2(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2) "enc"="0000010 rs2:5 rs1:5 001 rd:5 1010011" "asm"="fadd.h {rd}, {rs1}, {rs2}, rtz" "asmonly" {
  ret void
}

define void @FADD_H.3(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2) "enc"="0000010 rs2:5 rs1:5 010 rd:5 1010011" "asm"="fadd.h {rd}, {rs1}, {rs2}, rdn" "asmonly" {
  ret void
}

define void @FADD_H.4(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2) "enc"="0000010 rs2:5 rs1:5 011 rd:5 1010011" "asm"="fadd.h {rd}, {rs1}, {rs2}, rup" "asmonly" {
  ret void
}

define void @FADD_H.5(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2) "enc"="0000010 rs2:5 rs1:5 100 rd:5 1010011" "asm"="fadd.h {rd}, {rs1}, {rs2}, rmm" "asmonly" {
  ret void
}

define void @FADD_H.6(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2) "enc"="0000010 rs2:5 rs1:5 111 rd:5 1010011" "asm"="fadd.h {rd}, {rs1}, {rs2}, dyn" "asmonly" "nodecode" {
  ret void
}

define void @FSUB_H.0(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2) "enc"="0000110 rs2:5 rs1:5 111 rd:5 1010011" "asm"="fsub.h {rd}, {rs1}, {rs2}" "asmonly" {
  ret void
}

define void @FSUB_H.1(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2) "enc"="0000110 rs2:5 rs1:5 000 rd:5 1010011" "asm"="fsub.h {rd}, {rs1}, {rs2}, rne" "asmonly" {
  ret void
}

define void @FSUB_H.2(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2) "enc"="0000110 rs2:5 rs1:5 001 rd:5 1010011" "asm"="fsub.h {rd}, {rs1}, {rs2}, rtz" "asmonly" {
  ret void
}

define void @FSUB_H.3(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2) "enc"="0000110 rs2:5 rs1:5 010 rd:5 1010011" "asm"="fsub.h {rd}, {rs1}, {rs2}, rdn" "asmonly" {
  ret void
}

define void @FSUB_H.4(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2) "enc"="0000110 rs2:5 rs1:5 011 rd:5 1010011" "asm"="fsub.h {rd}, {rs1}, {rs2}, rup" "asmonly" {
  ret void
}

define void @FSUB_H.5(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2) "enc"="0000110 rs2:5 rs1:5 100 rd:5 1010011" "asm"="fsub.h {rd}, {rs1}, {rs2}, rmm" "asmonly" {
  ret void
}

define void @FSUB_H.6(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2) "enc"="0000110 rs2:5 rs1:5 111 rd:5 1010011" "asm"="fsub.h {rd}, {rs1}, {rs2}, dyn" "asmonly" "nodecode" {
  ret void
}

define void @FMUL_H.0(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2) "enc"="0001010 rs2:5 rs1:5 111 rd:5 1010011" "asm"="fmul.h {rd}, {rs1}, {rs2}" "asmonly" {
  ret void
}

define void @FMUL_H.1(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2) "enc"="0001010 rs2:5 rs1:5 000 rd:5 1010011" "asm"="fmul.h {rd}, {rs1}, {rs2}, rne" "asmonly" {
  ret void
}

define void @FMUL_H.2(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2) "enc"="0001010 rs2:5 rs1:5 001 rd:5 1010011" "asm"="fmul.h {rd}, {rs1}, {rs2}, rtz" "asmonly" {
  ret void
}

define void @FMUL_H.3(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2) "enc"="0001010 rs2:5 rs1:5 010 rd:5 1010011" "asm"="fmul.h {rd}, {rs1}, {rs2}, rdn" "asmonly" {
  ret void
}

define void @FMUL_H.4(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2) "enc"="0001010 rs2:5 rs1:5 011 rd:5 1010011" "asm"="fmul.h {rd}, {rs1}, {rs2}, rup" "asmonly" {
  ret void
}

define void @FMUL_H.5(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2) "enc"="0001010 rs2:5 rs1:5 100 rd:5 1010011" "asm"="fmul.h {rd}, {rs1}, {rs2}, rmm" "asmonly" {
  ret void
}

define void @FMUL_H.6(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2) "enc"="0001010 rs2:5 rs1:5 111 rd:5 1010011" "asm"="fmul.h {rd}, {rs1}, {rs2}, dyn" "asmonly" "nodecode" {
  ret void
}

define void @FDIV_H.0(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2) "enc"="0001110 rs2:5 rs1:5 111 rd:5 1010011" "asm"="fdiv.h {rd}, {rs1}, {rs2}" "asmonly" {
  ret void
}

define void @FDIV_H.1(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2) "enc"="0001110 rs2:5 rs1:5 000 rd:5 1010011" "asm"="fdiv.h {rd}, {rs1}, {rs2}, rne" "asmonly" {
  ret void
}

define void @FDIV_H.2(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2) "enc"="0001110 rs2:5 rs1:5 001 rd:5 1010011" "asm"="fdiv.h {rd}, {rs1}, {rs2}, rtz" "asmonly" {
  ret void
}

define void @FDIV_H.3(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2) "enc"="0001110 rs2:5 rs1:5 010 rd:5 1010011" "asm"="fdiv.h {rd}, {rs1}, {rs2}, rdn" "asmonly" {
  ret void
}

define void @FDIV_H.4(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2) "enc"="0001110 rs2:5 rs1:5 011 rd:5 1010011" "asm"="fdiv.h {rd}, {rs1}, {rs2}, rup" "asmonly" {
  ret void
}

define void @FDIV_H.5(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2) "enc"="0001110 rs2:5 rs1:5 100 rd:5 1010011" "asm"="fdiv.h {rd}, {rs1}, {rs2}, rmm" "asmonly" {
  ret void
}

define void @FDIV_H.6(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2) "enc"="0001110 rs2:5 rs1:5 111 rd:5 1010011" "asm"="fdiv.h {rd}, {rs1}, {rs2}, dyn" "asmonly" "nodecode" {
  ret void
}

define void @FSQRT_H.0(i64 "reg"="d" %rd, i64 "reg"="d" %rs1) "enc"="010111000000 rs1:5 111 rd:5 1010011" "asm"="fsqrt.h {rd}, {rs1}" "asmonly" {
  ret void
}

define void @FSQRT_H.1(i64 "reg"="d" %rd, i64 "reg"="d" %rs1) "enc"="010111000000 rs1:5 000 rd:5 1010011" "asm"="fsqrt.h {rd}, {rs1}, rne" "asmonly" {
  ret void
}

define void @FSQRT_H.2(i64 "reg"="d" %rd, i64 "reg"="d" %rs1) "enc"="010111000000 rs1:5 001 rd:5 1010011" "asm"="fsqrt.h {rd}, {rs1}, rtz" "asmonly" {
  ret void
}

define void @FSQRT_H.3(i64 "reg"="d" %rd, i64 "reg"="d" %rs1) "enc"="010111000000 rs1:5 010 rd:5 1010011" "asm"="fsqrt.h {rd}, {rs1}, rdn" "asmonly" {
  ret void
}

define void @FSQRT_H.4(i64 "reg"="d" %rd, i64 "reg"="d" %rs1) "enc"="010111000000 rs1:5 011 rd:5 1010011" "asm"="fsqrt.h {rd}, {rs1}, rup" "asmonly" {
  ret void
}

define void @FSQRT_H.5(i64 "reg"="d" %rd, i64 "reg"="d" %rs1) "enc"="010111000000 rs1:5 100 rd:5 1010011" "asm"="fsqrt.h {rd}, {rs1}, rmm" "asmonly" {
  ret void
}

define void @FSQRT_H.6(i64 "reg"="d" %rd, i64 "reg"="d" %rs1) "enc"="010111000000 rs1:5 111 rd:5 1010011" "asm"="fsqrt.h {rd}, {rs1}, dyn" "asmonly" "nodecode" {
  ret void
}

define void @FSGNJ_H.0(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2) "enc"="0010010 rs2:5 rs1:5 000 rd:5 1010011" "asm"="fsgnj.h {rd}, {rs1}, {rs2}" "asmonly" {
  ret void
}

define void @FSGNJN_H.0(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2) "enc"="0010010 rs2:5 rs1:5 001 rd:5 1010011" "asm"="fsgnjn.h {rd}, {rs1}, {rs2}" "asmonly" {
  ret void
}

define void @FSGNJX_H.0(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2) "enc"="0010010 rs2:5 rs1:5 010 rd:5 1010011" "asm"="fsgnjx.h {rd}, {rs1}, {rs2}" "asmonly" {
  ret void
}

define void @FMIN_H.0(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2) "enc"="0010110 rs2:5 rs1:5 000 rd:5 1010011" "asm"="fmin.h {rd}, {rs1}, {rs2}" "asmonly" {
  ret void
}

define void @FMAX_H.0(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2) "enc"="0010110 rs2:5 rs1:5 001 rd:5 1010011" "asm"="fmax.h {rd}, {rs1}, {rs2}" "asmonly" {
  ret void
}

define void @FEQ_H.0(i64 "reg"="x" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2) "enc"="1010010 rs2:5 rs1:5 010 rd:5 1010011" "asm"="feq.h {rd}, {rs1}, {rs2}" "asmonly" {
  ret void
}

define void @FLT_H.0(i64 "reg"="x" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2) "enc"="1010010 rs2:5 rs1:5 001 rd:5 1010011" "asm"="flt.h {rd}, {rs1}, {rs2}" "asmonly" {
  ret void
}

define void @FLE_H.0(i64 "reg"="x" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2) "enc"="1010010 rs2:5 rs1:5 000 rd:5 1010011" "asm"="fle.h {rd}, {rs1}, {rs2}" "asmonly" {
  ret void
}

define void @FCLASS_H.0(i64 "reg"="x" %rd, i64 "reg"="d" %rs1) "enc"="111001000000 rs1:5 001 rd:5 1010011" "asm"="fclass.h {rd}, {rs1}" "asmonly" {
  ret void
}

define void @FCVT_W_H.0(i64 "reg"="x" %rd, i64 "reg"="d" %rs1) "enc"="110001000000 rs1:5 111 rd:5 1010011" "asm"="fcvt.w.h {rd}, {rs1}" "asmonly" {
  ret void
}

define void @FCVT_W_H.1(i64 "reg"="x" %rd, i64 "reg"="d" %rs1) "enc"="110001000000 rs1:5 000 rd:5 1010011" "asm"="fcvt.w.h {rd}, {rs1}, rne" "asmonly" {
  ret void
}

define void @FCVT_W_H.2(i64 "reg"="x" %rd, i64 "reg"="d" %rs1) "enc"="110001000000 rs1:5 001 rd:5 1010011" "asm"="fcvt.w.h {rd}, {rs1}, rtz" "asmonly" {
  ret void
}

define void @FCVT_W_H.3(i64 "reg"="x" %rd, i64 "reg"="d" %rs1) "enc"="110001000000 rs1:5 010 rd:5 1010011" "asm"="fcvt.w.h {rd}, {rs1}, rdn" "asmonly" {
  ret void
}

define void @FCVT_W_H.4(i64 "reg"="x" %rd, i64 "reg"="d" %rs1) "enc"="110001000000 rs1:5 011 rd:5 1010011" "asm"="fcvt.w.h {rd}, {rs1}, rup" "asmonly" {
  ret void
}

define void @FCVT_W_H.5(i64 "reg"="x" %rd, i64 "reg"="d" %rs1) "enc"="110001000000 rs1:5 100 rd:5 1010011" "asm"="fcvt.w.h {rd}, {rs1}, rmm" "asmonly" {
  ret void
}

define void @FCVT_W_H.6(i64 "reg"="x" %rd, i64 "reg"="d" %rs1) "enc"="110001000000 rs1:5 111 rd:5 1010011" "asm"="fcvt.w.h {rd}, {rs1}, dyn" "asmonly" "nodecode" {
  ret void
}

define void @FCVT_WU_H.0(i64 "reg"="x" %rd, i64 "reg"="d" %rs1) "enc"="110001000001 rs1:5 111 rd:5 1010011" "asm"="fcvt.wu.h {rd}, {rs1}" "asmonly" {
  ret void
}

define void @FCVT_WU_H.1(i64 "reg"="x" %rd, i64 "reg"="d" %rs1) "enc"="110001000001 rs1:5 000 rd:5 1010011" "asm"="fcvt.wu.h {rd}, {rs1}, rne" "asmonly" {
  ret void
}

define void @FCVT_WU_H.2(i64 "reg"="x" %rd, i64 "reg"="d" %rs1) "enc"="110001000001 rs1:5 001 rd:5 1010011" "asm"="fcvt.wu.h {rd}, {rs1}, rtz" "asmonly" {
  ret void
}

define void @FCVT_WU_H.3(i64 "reg"="x" %rd, i64 "reg"="d" %rs1) "enc"="110001000001 rs1:5 010 rd:5 1010011" "asm"="fcvt.wu.h {rd}, {rs1}, rdn" "asmonly" {
  ret void
}

define void @FCVT_WU_H.4(i64 "reg"="x" %rd, i64 "reg"="d" %rs1) "enc"="110001000001 rs1:5 011 rd:5 1010011" "asm"="fcvt.wu.h {rd}, {rs1}, rup" "asmonly" {
  ret void
}

define void @FCVT_WU_H.5(i64 "reg"="x" %rd, i64 "reg"="d" %rs1) "enc"="110001000001 rs1:5 100 rd:5 1010011" "asm"="fcvt.wu.h {rd}, {rs1}, rmm" "asmonly" {
  ret void
}

define void @FCVT_WU_H.6(i64 "reg"="x" %rd, i64 "reg"="d" %rs1) "enc"="110001000001 rs1:5 111 rd:5 1010011" "asm"="fcvt.wu.h {rd}, {rs1}, dyn" "asmonly" "nodecode" {
  ret void
}

define void @FCVT_H_W.0(i64 "reg"="d" %rd, i64 "reg"="x" %rs1) "enc"="110101000000 rs1:5 111 rd:5 1010011" "asm"="fcvt.h.w {rd}, {rs1}" "asmonly" {
  ret void
}

define void @FCVT_H_W.1(i64 "reg"="d" %rd, i64 "reg"="x" %rs1) "enc"="110101000000 rs1:5 000 rd:5 1010011" "asm"="fcvt.h.w {rd}, {rs1}, rne" "asmonly" {
  ret void
}

define void @FCVT_H_W.2(i64 "reg"="d" %rd, i64 "reg"="x" %rs1) "enc"="110101000000 rs1:5 001 rd:5 1010011" "asm"="fcvt.h.w {rd}, {rs1}, rtz" "asmonly" {
  ret void
}

define void @FCVT_H_W.3(i64 "reg"="d" %rd, i64 "reg"="x" %rs1) "enc"="110101000000 rs1:5 010 rd:5 1010011" "asm"="fcvt.h.w {rd}, {rs1}, rdn" "asmonly" {
  ret void
}

define void @FCVT_H_W.4(i64 "reg"="d" %rd, i64 "reg"="x" %rs1) "enc"="110101000000 rs1:5 011 rd:5 1010011" "asm"="fcvt.h.w {rd}, {rs1}, rup" "asmonly" {
  ret void
}

define void @FCVT_H_W.5(i64 "reg"="d" %rd, i64 "reg"="x" %rs1) "enc"="110101000000 rs1:5 100 rd:5 1010011" "asm"="fcvt.h.w {rd}, {rs1}, rmm" "asmonly" {
  ret void
}

define void @FCVT_H_W.6(i64 "reg"="d" %rd, i64 "reg"="x" %rs1) "enc"="110101000000 rs1:5 111 rd:5 1010011" "asm"="fcvt.h.w {rd}, {rs1}, dyn" "asmonly" "nodecode" {
  ret void
}

define void @FCVT_H_WU.0(i64 "reg"="d" %rd, i64 "reg"="x" %rs1) "enc"="110101000001 rs1:5 111 rd:5 1010011" "asm"="fcvt.h.wu {rd}, {rs1}" "asmonly" {
  ret void
}

define void @FCVT_H_WU.1(i64 "reg"="d" %rd, i64 "reg"="x" %rs1) "enc"="110101000001 rs1:5 000 rd:5 1010011" "asm"="fcvt.h.wu {rd}, {rs1}, rne" "asmonly" {
  ret void
}

define void @FCVT_H_WU.2(i64 "reg"="d" %rd, i64 "reg"="x" %rs1) "enc"="110101000001 rs1:5 001 rd:5 1010011" "asm"="fcvt.h.wu {rd}, {rs1}, rtz" "asmonly" {
  ret void
}

define void @FCVT_H_WU.3(i64 "reg"="d" %rd, i64 "reg"="x" %rs1) "enc"="110101000001 rs1:5 010 rd:5 1010011" "asm"="fcvt.h.wu {rd}, {rs1}, rdn" "asmonly" {
  ret void
}

define void @FCVT_H_WU.4(i64 "reg"="d" %rd, i64 "reg"="x" %rs1) "enc"="110101000001 rs1:5 011 rd:5 1010011" "asm"="fcvt.h.wu {rd}, {rs1}, rup" "asmonly" {
  ret void
}

define void @FCVT_H_WU.5(i64 "reg"="d" %rd, i64 "reg"="x" %rs1) "enc"="110101000001 rs1:5 100 rd:5 1010011" "asm"="fcvt.h.wu {rd}, {rs1}, rmm" "asmonly" {
  ret void
}

define void @FCVT_H_WU.6(i64 "reg"="d" %rd, i64 "reg"="x" %rs1) "enc"="110101000001 rs1:5 111 rd:5 1010011" "asm"="fcvt.h.wu {rd}, {rs1}, dyn" "asmonly" "nodecode" {
  ret void
}

define void @FCVT_L_H.0(i64 "reg"="x" %rd, i64 "reg"="d" %rs1) "enc"="110001000010 rs1:5 111 rd:5 1010011" "asm"="fcvt.l.h {rd}, {rs1}" "asmonly" {
  ret void
}

define void @FCVT_L_H.1(i64 "reg"="x" %rd, i64 "reg"="d" %rs1) "enc"="110001000010 rs1:5 000 rd:5 1010011" "asm"="fcvt.l.h {rd}, {rs1}, rne" "asmonly" {
  ret void
}

define void @FCVT_L_H.2(i64 "reg"="x" %rd, i64 "reg"="d" %rs1) "enc"="110001000010 rs1:5 001 rd:5 1010011" "asm"="fcvt.l.h {rd}, {rs1}, rtz" "asmonly" {
  ret void
}

define void @FCVT_L_H.3(i64 "reg"="x" %rd, i64 "reg"="d" %rs1) "enc"="110001000010 rs1:5 010 rd:5 1010011" "asm"="fcvt.l.h {rd}, {rs1}, rdn" "asmonly" {
  ret void
}

define void @FCVT_L_H.4(i64 "reg"="x" %rd, i64 "reg"="d" %rs1) "enc"="110001000010 rs1:5 011 rd:5 1010011" "asm"="fcvt.l.h {rd}, {rs1}, rup" "asmonly" {
  ret void
}

define void @FCVT_L_H.5(i64 "reg"="x" %rd, i64 "reg"="d" %rs1) "enc"="110001000010 rs1:5 100 rd:5 1010011" "asm"="fcvt.l.h {rd}, {rs1}, rmm" "asmonly" {
  ret void
}

define void @FCVT_L_H.6(i64 "reg"="x" %rd, i64 "reg"="d" %rs1) "enc"="110001000010 rs1:5 111 rd:5 1010011" "asm"="fcvt.l.h {rd}, {rs1}, dyn" "asmonly" "nodecode" {
  ret void
}

define void @FCVT_LU_H.0(i64 "reg"="x" %rd, i64 "reg"="d" %rs1) "enc"="110001000011 rs1:5 111 rd:5 1010011" "asm"="fcvt.lu.h {rd}, {rs1}" "asmonly" {
  ret void
}

define void @FCVT_LU_H.1(i64 "reg"="x" %rd, i64 "reg"="d" %rs1) "enc"="110001000011 rs1:5 000 rd:5 1010011" "asm"="fcvt.lu.h {rd}, {rs1}, rne" "asmonly" {
  ret void
}

define void @FCVT_LU_H.2(i64 "reg"="x" %rd, i64 "reg"="d" %rs1) "enc"="110001000011 rs1:5 001 rd:5 1010011" "asm"="fcvt.lu.h {rd}, {rs1}, rtz" "asmonly" {
  ret void
}

define void @FCVT_LU_H.3(i64 "reg"="x" %rd, i64 "reg"="d" %rs1) "enc"="110001000011 rs1:5 010 rd:5 1010011" "asm"="fcvt.lu.h {rd}, {rs1}, rdn" "asmonly" {
  ret void
}

define void @FCVT_LU_H.4(i64 "reg"="x" %rd, i64 "reg"="d" %rs1) "enc"="110001000011 rs1:5 011 rd:5 1010011" "asm"="fcvt.lu.h {rd}, {rs1}, rup" "asmonly" {
  ret void
}

define void @FCVT_LU_H.5(i64 "reg"="x" %rd, i64 "reg"="d" %rs1) "enc"="110001000011 rs1:5 100 rd:5 1010011" "asm"="fcvt.lu.h {rd}, {rs1}, rmm" "asmonly" {
  ret void
}

define void @FCVT_LU_H.6(i64 "reg"="x" %rd, i64 "reg"="d" %rs1) "enc"="110001000011 rs1:5 111 rd:5 1010011" "asm"="fcvt.lu.h {rd}, {rs1}, dyn" "asmonly" "nodecode" {
  ret void
}

define void @FCVT_H_L.0(i64 "reg"="d" %rd, i64 "reg"="x" %rs1) "enc"="110101000010 rs1:5 111 rd:5 1010011" "asm"="fcvt.h.l {rd}, {rs1}" "asmonly" {
  ret void
}

define void @FCVT_H_L.1(i64 "reg"="d" %rd, i64 "reg"="x" %rs1) "enc"="110101000010 rs1:5 000 rd:5 1010011" "asm"="fcvt.h.l {rd}, {rs1}, rne" "asmonly" {
  ret void
}

define void @FCVT_H_L.2(i64 "reg"="d" %rd, i64 "reg"="x" %rs1) "enc"="110101000010 rs1:5 001 rd:5 1010011" "asm"="fcvt.h.l {rd}, {rs1}, rtz" "asmonly" {
  ret void
}

define void @FCVT_H_L.3(i64 "reg"="d" %rd, i64 "reg"="x" %rs1) "enc"="110101000010 rs1:5 010 rd:5 1010011" "asm"="fcvt.h.l {rd}, {rs1}, rdn" "asmonly" {
  ret void
}

define void @FCVT_H_L.4(i64 "reg"="d" %rd, i64 "reg"="x" %rs1) "enc"="110101000010 rs1:5 011 rd:5 1010011" "asm"="fcvt.h.l {rd}, {rs1}, rup" "asmonly" {
  ret void
}

define void @FCVT_H_L.5(i64 "reg"="d" %rd, i64 "reg"="x" %rs1) "enc"="110101000010 rs1:5 100 rd:5 1010011" "asm"="fcvt.h.l {rd}, {rs1}, rmm" "asmonly" {
  ret void
}

define void @FCVT_H_L.6(i64 "reg"="d" %rd, i64 "reg"="x" %rs1) "enc"="110101000010 rs1:5 111 rd:5 1010011" "asm"="fcvt.h.l {rd}, {rs1}, dyn" "asmonly" "nodecode" {
  ret void
}

define void @FCVT_H_LU.0(i64 "reg"="d" %rd, i64 "reg"="x" %rs1) "enc"="110101000011 rs1:5 111 rd:5 1010011" "asm"="fcvt.h.lu {rd}, {rs1}" "asmonly" {
  ret void
}

define void @FCVT_H_LU.1(i64 "reg"="d" %rd, i64 "reg"="x" %rs1) "enc"="110101000011 rs1:5 000 rd:5 1010011" "asm"="fcvt.h.lu {rd}, {rs1}, rne" "asmonly" {
  ret void
}

define void @FCVT_H_LU.2(i64 "reg"="d" %rd, i64 "reg"="x" %rs1) "enc"="110101000011 rs1:5 001 rd:5 1010011" "asm"="fcvt.h.lu {rd}, {rs1}, rtz" "asmonly" {
  ret void
}

define void @FCVT_H_LU.3(i64 "reg"="d" %rd, i64 "reg"="x" %rs1) "enc"="110101000011 rs1:5 010 rd:5 1010011" "asm"="fcvt.h.lu {rd}, {rs1}, rdn" "asmonly" {
  ret void
}

define void @FCVT_H_LU.4(i64 "reg"="d" %rd, i64 "reg"="x" %rs1) "enc"="110101000011 rs1:5 011 rd:5 1010011" "asm"="fcvt.h.lu {rd}, {rs1}, rup" "asmonly" {
  ret void
}

define void @FCVT_H_LU.5(i64 "reg"="d" %rd, i64 "reg"="x" %rs1) "enc"="110101000011 rs1:5 100 rd:5 1010011" "asm"="fcvt.h.lu {rd}, {rs1}, rmm" "asmonly" {
  ret void
}

define void @FCVT_H_LU.6(i64 "reg"="d" %rd, i64 "reg"="x" %rs1) "enc"="110101000011 rs1:5 111 rd:5 1010011" "asm"="fcvt.h.lu {rd}, {rs1}, dyn" "asmonly" "nodecode" {
  ret void
}

define void @FLI_H.0(i64 "reg"="d" %rd) "enc"="11110100000100000000 rd:5 1010011" "asm"="fli.h {rd}, -1.0" "asmonly" {
  ret void
}

define void @FLI_H.1(i64 "reg"="d" %rd) "enc"="11110100000100001000 rd:5 1010011" "asm"="fli.h {rd}, min" "asmonly" {
  ret void
}

define void @FLI_H.2(i64 "reg"="d" %rd) "enc"="11110100000100010000 rd:5 1010011" "asm"="fli.h {rd}, 1.52587890625e-05" "asmonly" {
  ret void
}

define void @FLI_H.3(i64 "reg"="d" %rd) "enc"="11110100000100011000 rd:5 1010011" "asm"="fli.h {rd}, 3.0517578125e-05" "asmonly" {
  ret void
}

define void @FLI_H.4(i64 "reg"="d" %rd) "enc"="11110100000100100000 rd:5 1010011" "asm"="fli.h {rd}, 0.00390625" "asmonly" {
  ret void
}

define void @FLI_H.5(i64 "reg"="d" %rd) "enc"="11110100000100101000 rd:5 1010011" "asm"="fli.h {rd}, 0.0078125" "asmonly" {
  ret void
}

define void @FLI_H.6(i64 "reg"="d" %rd) "enc"="11110100000100110000 rd:5 1010011" "asm"="fli.h {rd}, 0.0625" "asmonly" {
  ret void
}

define void @FLI_H.7(i64 "reg"="d" %rd) "enc"="11110100000100111000 rd:5 1010011" "asm"="fli.h {rd}, 0.125" "asmonly" {
  ret void
}

define void @FLI_H.8(i64 "reg"="d" %rd) "enc"="11110100000101000000 rd:5 1010011" "asm"="fli.h {rd}, 0.25" "asmonly" {
  ret void
}

define void @FLI_H.9(i64 "reg"="d" %rd) "enc"="11110100000101001000 rd:5 1010011" "asm"="fli.h {rd}, 0.3125" "asmonly" {
  ret void
}

define void @FLI_H.10(i64 "reg"="d" %rd) "enc"="11110100000101010000 rd:5 1010011" "asm"="fli.h {rd}, 0.375" "asmonly" {
  ret void
}

define void @FLI_H.11(i64 "reg"="d" %rd) "enc"="11110100000101011000 rd:5 1010011" "asm"="fli.h {rd}, 0.4375" "asmonly" {
  ret void
}

define void @FLI_H.12(i64 "reg"="d" %rd) "enc"="11110100000101100000 rd:5 1010011" "asm"="fli.h {rd}, 0.5" "asmonly" {
  ret void
}

define void @FLI_H.13(i64 "reg"="d" %rd) "enc"="11110100000101101000 rd:5 1010011" "asm"="fli.h {rd}, 0.625" "asmonly" {
  ret void
}

define void @FLI_H.14(i64 "reg"="d" %rd) "enc"="11110100000101110000 rd:5 1010011" "asm"="fli.h {rd}, 0.75" "asmonly" {
  ret void
}

define void @FLI_H.15(i64 "reg"="d" %rd) "enc"="11110100000101111000 rd:5 1010011" "asm"="fli.h {rd}, 0.875" "asmonly" {
  ret void
}

define void @FLI_H.16(i64 "reg"="d" %rd) "enc"="11110100000110000000 rd:5 1010011" "asm"="fli.h {rd}, 1.0" "asmonly" {
  ret void
}

define void @FLI_H.17(i64 "reg"="d" %rd) "enc"="11110100000110001000 rd:5 1010011" "asm"="fli.h {rd}, 1.25" "asmonly" {
  ret void
}

define void @FLI_H.18(i64 "reg"="d" %rd) "enc"="11110100000110010000 rd:5 1010011" "asm"="fli.h {rd}, 1.5" "asmonly" {
  ret void
}

define void @FLI_H.19(i64 "reg"="d" %rd) "enc"="11110100000110011000 rd:5 1010011" "asm"="fli.h {rd}, 1.75" "asmonly" {
  ret void
}

define void @FLI_H.20(i64 "reg"="d" %rd) "enc"="11110100000110100000 rd:5 1010011" "asm"="fli.h {rd}, 2.0" "asmonly" {
  ret void
}

define void @FLI_H.21(i64 "reg"="d" %rd) "enc"="11110100000110101000 rd:5 1010011" "asm"="fli.h {rd}, 2.5" "asmonly" {
  ret void
}

define void @FLI_H.22(i64 "reg"="d" %rd) "enc"="11110100000110110000 rd:5 1010011" "asm"="fli.h {rd}, 3.0" "asmonly" {
  ret void
}

define void @FLI_H.23(i64 "reg"="d" %rd) "enc"="11110100000110111000 rd:5 1010011" "asm"="fli.h {rd}, 4.0" "asmonly" {
  ret void
}

define void @FLI_H.24(i64 "reg"="d" %rd) "enc"="11110100000111000000 rd:5 1010011" "asm"="fli.h {rd}, 8.0" "asmonly" {
  ret void
}

define void @FLI_H.25(i64 "reg"="d" %rd) "enc"="11110100000111001000 rd:5 1010011" "asm"="fli.h {rd}, 16.0" "asmonly" {
  ret void
}

define void @FLI_H.26(i64 "reg"="d" %rd) "enc"="11110100000111010000 rd:5 1010011" "asm"="fli.h {rd}, 128.0" "asmonly" {
  ret void
}

define void @FLI_H.27(i64 "reg"="d" %rd) "enc"="11110100000111011000 rd:5 1010011" "asm"="fli.h {rd}, 256.0" "asmonly" {
  ret void
}

define void @FLI_H.28(i64 "reg"="d" %rd) "enc"="11110100000111100000 rd:5 1010011" "asm"="fli.h {rd}, 32768.0" "asmonly" {
  ret void
}

define void @FLI_H.29(i64 "reg"="d" %rd) "enc"="11110100000111101000 rd:5 1010011" "asm"="fli.h {rd}, 65536.0" "asmonly" {
  ret void
}

define void @FLI_H.30(i64 "reg"="d" %rd) "enc"="11110100000111110000 rd:5 1010011" "asm"="fli.h {rd}, inf" "asmonly" {
  ret void
}

define void @FLI_H.31(i64 "reg"="d" %rd) "enc"="11110100000111111000 rd:5 1010011" "asm"="fli.h {rd}, nan" "asmonly" {
  ret void
}

define void @FMINM_H.0(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2) "enc"="0010110 rs2:5 rs1:5 010 rd:5 1010011" "asm"="fminm.h {rd}, {rs1}, {rs2}" "asmonly" {
  ret void
}

define void @FMAXM_H.0(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2) "enc"="0010110 rs2:5 rs1:5 011 rd:5 1010011" "asm"="fmaxm.h {rd}, {rs1}, {rs2}" "asmonly" {
  ret void
}

define void @FROUND_H.0(i64 "reg"="d" %rd, i64 "reg"="d" %rs1) "enc"="010001000100 rs1:5 111 rd:5 1010011" "asm"="fround.h {rd}, {rs1}" "asmonly" {
  ret void
}

define void @FROUND_H.1(i64 "reg"="d" %rd, i64 "reg"="d" %rs1) "enc"="010001000100 rs1:5 000 rd:5 1010011" "asm"="fround.h {rd}, {rs1}, rne" "asmonly" {
  ret void
}

define void @FROUND_H.2(i64 "reg"="d" %rd, i64 "reg"="d" %rs1) "enc"="010001000100 rs1:5 001 rd:5 1010011" "asm"="fround.h {rd}, {rs1}, rtz" "asmonly" {
  ret void
}

define void @FROUND_H.3(i64 "reg"="d" %rd, i64 "reg"="d" %rs1) "enc"="010001000100 rs1:5 010 rd:5 1010011" "asm"="fround.h {rd}, {rs1}, rdn" "asmonly" {
  ret void
}

define void @FROUND_H.4(i64 "reg"="d" %rd, i64 "reg"="d" %rs1) "enc"="010001000100 rs1:5 011 rd:5 1010011" "asm"="fround.h {rd}, {rs1}, rup" "asmonly" {
  ret void
}

define void @FROUND_H.5(i64 "reg"="d" %rd, i64 "reg"="d" %rs1) "enc"="010001000100 rs1:5 100 rd:5 1010011" "asm"="fround.h {rd}, {rs1}, rmm" "asmonly" {
  ret void
}

define void @FROUND_H.6(i64 "reg"="d" %rd, i64 "reg"="d" %rs1) "enc"="010001000100 rs1:5 111 rd:5 1010011" "asm"="fround.h {rd}, {rs1}, dyn" "asmonly" "nodecode" {
  ret void
}

define void @FROUNDNX_H.0(i64 "reg"="d" %rd, i64 "reg"="d" %rs1) "enc"="010001000101 rs1:5 111 rd:5 1010011" "asm"="froundnx.h {rd}, {rs1}" "asmonly" {
  ret void
}

define void @FROUNDNX_H.1(i64 "reg"="d" %rd, i64 "reg"="d" %rs1) "enc"="010001000101 rs1:5 000 rd:5 1010011" "asm"="froundnx.h {rd}, {rs1}, rne" "asmonly" {
  ret void
}

define void @FROUNDNX_H.2(i64 "reg"="d" %rd, i64 "reg"="d" %rs1) "enc"="010001000101 rs1:5 001 rd:5 1010011" "asm"="froundnx.h {rd}, {rs1}, rtz" "asmonly" {
  ret void
}

define void @FROUNDNX_H.3(i64 "reg"="d" %rd, i64 "reg"="d" %rs1) "enc"="010001000101 rs1:5 010 rd:5 1010011" "asm"="froundnx.h {rd}, {rs1}, rdn" "asmonly" {
  ret void
}

define void @FROUNDNX_H.4(i64 "reg"="d" %rd, i64 "reg"="d" %rs1) "enc"="010001000101 rs1:5 011 rd:5 1010011" "asm"="froundnx.h {rd}, {rs1}, rup" "asmonly" {
  ret void
}

define void @FROUNDNX_H.5(i64 "reg"="d" %rd, i64 "reg"="d" %rs1) "enc"="010001000101 rs1:5 100 rd:5 1010011" "asm"="froundnx.h {rd}, {rs1}, rmm" "asmonly" {
  ret void
}

define void @FROUNDNX_H.6(i64 "reg"="d" %rd, i64 "reg"="d" %rs1) "enc"="010001000101 rs1:5 111 rd:5 1010011" "asm"="froundnx.h {rd}, {rs1}, dyn" "asmonly" "nodecode" {
  ret void
}

define void @FLEQ_H.0(i64 "reg"="x" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2) "enc"="1010010 rs2:5 rs1:5 100 rd:5 1010011" "asm"="fleq.h {rd}, {rs1}, {rs2}" "asmonly" {
  ret void
}

define void @FLTQ_H.0(i64 "reg"="x" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2) "enc"="1010010 rs2:5 rs1:5 101 rd:5 1010011" "asm"="fltq.h {rd}, {rs1}, {rs2}" "asmonly" {
  ret void
}

define void @FLH.0(i64 "reg"="d" %rd, i64 "reg"="x" %rs1, i64 "imm" %imm12) "enc"="imm12:s12 rs1:5 001 rd:5 0000111" "asm"="flh {rd}, {imm12}({rs1})" "asmonly" {
  ret void
}

define void @FSH.0(i64 "reg"="x" %rs1, i64 "reg"="d" %rs2, i64 "imm" %off) "enc"="o_hi:s7=off>>5 rs2:5 rs1:5 001 o_lo:5=off&31 0100111" "asm"="fsh {rs2}, {off}({rs1})" "asmonly" {
  ret void
}

define void @FCVT_S_H.0(i64 "reg"="d" %rd, i64 "reg"="d" %rs1) "enc"="010000000010 rs1:5 000 rd:5 1010011" "asm"="fcvt.s.h {rd}, {rs1}" "asmonly" {
  ret void
}

define void @FCVT_S_H.1(i64 "reg"="d" %rd, i64 "reg"="d" %rs1) "enc"="010000000010 rs1:5 000 rd:5 1010011" "asm"="fcvt.s.h {rd}, {rs1}, rne" "asmonly" "nodecode" {
  ret void
}

define void @FCVT_S_H.2(i64 "reg"="d" %rd, i64 "reg"="d" %rs1) "enc"="010000000010 rs1:5 001 rd:5 1010011" "asm"="fcvt.s.h {rd}, {rs1}, rtz" "asmonly" {
  ret void
}

define void @FCVT_S_H.3(i64 "reg"="d" %rd, i64 "reg"="d" %rs1) "enc"="010000000010 rs1:5 010 rd:5 1010011" "asm"="fcvt.s.h {rd}, {rs1}, rdn" "asmonly" {
  ret void
}

define void @FCVT_S_H.4(i64 "reg"="d" %rd, i64 "reg"="d" %rs1) "enc"="010000000010 rs1:5 011 rd:5 1010011" "asm"="fcvt.s.h {rd}, {rs1}, rup" "asmonly" {
  ret void
}

define void @FCVT_S_H.5(i64 "reg"="d" %rd, i64 "reg"="d" %rs1) "enc"="010000000010 rs1:5 100 rd:5 1010011" "asm"="fcvt.s.h {rd}, {rs1}, rmm" "asmonly" {
  ret void
}

define void @FCVT_S_H.6(i64 "reg"="d" %rd, i64 "reg"="d" %rs1) "enc"="010000000010 rs1:5 111 rd:5 1010011" "asm"="fcvt.s.h {rd}, {rs1}, dyn" "asmonly" {
  ret void
}

define void @FCVT_H_S.0(i64 "reg"="d" %rd, i64 "reg"="d" %rs1) "enc"="010001000000 rs1:5 111 rd:5 1010011" "asm"="fcvt.h.s {rd}, {rs1}" "asmonly" {
  ret void
}

define void @FCVT_H_S.1(i64 "reg"="d" %rd, i64 "reg"="d" %rs1) "enc"="010001000000 rs1:5 000 rd:5 1010011" "asm"="fcvt.h.s {rd}, {rs1}, rne" "asmonly" {
  ret void
}

define void @FCVT_H_S.2(i64 "reg"="d" %rd, i64 "reg"="d" %rs1) "enc"="010001000000 rs1:5 001 rd:5 1010011" "asm"="fcvt.h.s {rd}, {rs1}, rtz" "asmonly" {
  ret void
}

define void @FCVT_H_S.3(i64 "reg"="d" %rd, i64 "reg"="d" %rs1) "enc"="010001000000 rs1:5 010 rd:5 1010011" "asm"="fcvt.h.s {rd}, {rs1}, rdn" "asmonly" {
  ret void
}

define void @FCVT_H_S.4(i64 "reg"="d" %rd, i64 "reg"="d" %rs1) "enc"="010001000000 rs1:5 011 rd:5 1010011" "asm"="fcvt.h.s {rd}, {rs1}, rup" "asmonly" {
  ret void
}

define void @FCVT_H_S.5(i64 "reg"="d" %rd, i64 "reg"="d" %rs1) "enc"="010001000000 rs1:5 100 rd:5 1010011" "asm"="fcvt.h.s {rd}, {rs1}, rmm" "asmonly" {
  ret void
}

define void @FCVT_H_S.6(i64 "reg"="d" %rd, i64 "reg"="d" %rs1) "enc"="010001000000 rs1:5 111 rd:5 1010011" "asm"="fcvt.h.s {rd}, {rs1}, dyn" "asmonly" "nodecode" {
  ret void
}

define void @FMV_X_H.0(i64 "reg"="x" %rd, i64 "reg"="d" %rs1) "enc"="111001000000 rs1:5 000 rd:5 1010011" "asm"="fmv.x.h {rd}, {rs1}" "asmonly" {
  ret void
}

define void @FMV_H_X.0(i64 "reg"="d" %rd, i64 "reg"="x" %rs1) "enc"="111101000000 rs1:5 000 rd:5 1010011" "asm"="fmv.h.x {rd}, {rs1}" "asmonly" {
  ret void
}

define void @FCVT_D_H.0(i64 "reg"="d" %rd, i64 "reg"="d" %rs1) "enc"="010000100010 rs1:5 000 rd:5 1010011" "asm"="fcvt.d.h {rd}, {rs1}" "asmonly" {
  ret void
}

define void @FCVT_D_H.1(i64 "reg"="d" %rd, i64 "reg"="d" %rs1) "enc"="010000100010 rs1:5 000 rd:5 1010011" "asm"="fcvt.d.h {rd}, {rs1}, rne" "asmonly" "nodecode" {
  ret void
}

define void @FCVT_D_H.2(i64 "reg"="d" %rd, i64 "reg"="d" %rs1) "enc"="010000100010 rs1:5 001 rd:5 1010011" "asm"="fcvt.d.h {rd}, {rs1}, rtz" "asmonly" {
  ret void
}

define void @FCVT_D_H.3(i64 "reg"="d" %rd, i64 "reg"="d" %rs1) "enc"="010000100010 rs1:5 010 rd:5 1010011" "asm"="fcvt.d.h {rd}, {rs1}, rdn" "asmonly" {
  ret void
}

define void @FCVT_D_H.4(i64 "reg"="d" %rd, i64 "reg"="d" %rs1) "enc"="010000100010 rs1:5 011 rd:5 1010011" "asm"="fcvt.d.h {rd}, {rs1}, rup" "asmonly" {
  ret void
}

define void @FCVT_D_H.5(i64 "reg"="d" %rd, i64 "reg"="d" %rs1) "enc"="010000100010 rs1:5 100 rd:5 1010011" "asm"="fcvt.d.h {rd}, {rs1}, rmm" "asmonly" {
  ret void
}

define void @FCVT_D_H.6(i64 "reg"="d" %rd, i64 "reg"="d" %rs1) "enc"="010000100010 rs1:5 111 rd:5 1010011" "asm"="fcvt.d.h {rd}, {rs1}, dyn" "asmonly" {
  ret void
}

define void @FCVT_H_D.0(i64 "reg"="d" %rd, i64 "reg"="d" %rs1) "enc"="010001000001 rs1:5 111 rd:5 1010011" "asm"="fcvt.h.d {rd}, {rs1}" "asmonly" {
  ret void
}

define void @FCVT_H_D.1(i64 "reg"="d" %rd, i64 "reg"="d" %rs1) "enc"="010001000001 rs1:5 000 rd:5 1010011" "asm"="fcvt.h.d {rd}, {rs1}, rne" "asmonly" {
  ret void
}

define void @FCVT_H_D.2(i64 "reg"="d" %rd, i64 "reg"="d" %rs1) "enc"="010001000001 rs1:5 001 rd:5 1010011" "asm"="fcvt.h.d {rd}, {rs1}, rtz" "asmonly" {
  ret void
}

define void @FCVT_H_D.3(i64 "reg"="d" %rd, i64 "reg"="d" %rs1) "enc"="010001000001 rs1:5 010 rd:5 1010011" "asm"="fcvt.h.d {rd}, {rs1}, rdn" "asmonly" {
  ret void
}

define void @FCVT_H_D.4(i64 "reg"="d" %rd, i64 "reg"="d" %rs1) "enc"="010001000001 rs1:5 011 rd:5 1010011" "asm"="fcvt.h.d {rd}, {rs1}, rup" "asmonly" {
  ret void
}

define void @FCVT_H_D.5(i64 "reg"="d" %rd, i64 "reg"="d" %rs1) "enc"="010001000001 rs1:5 100 rd:5 1010011" "asm"="fcvt.h.d {rd}, {rs1}, rmm" "asmonly" {
  ret void
}

define void @FCVT_H_D.6(i64 "reg"="d" %rd, i64 "reg"="d" %rs1) "enc"="010001000001 rs1:5 111 rd:5 1010011" "asm"="fcvt.h.d {rd}, {rs1}, dyn" "asmonly" "nodecode" {
  ret void
}

define void @WRS_NTO.0() "enc"="00000000110100000000000001110011" "asm"="wrs.nto" "asmonly" {
  ret void
}

define void @WRS_STO.0() "enc"="00000001110100000000000001110011" "asm"="wrs.sto" "asmonly" {
  ret void
}

define void @AMOCAS_W.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1, i64 "reg"="x" %rs2) "enc"="0010100 rs2:5 rs1:5 010 rd:5 0101111" "asm"="amocas.w {rd}, {rs2}, ({rs1})" "asmonly" {
  ret void
}

define void @AMOCAS_W_AQ.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1, i64 "reg"="x" %rs2) "enc"="0010110 rs2:5 rs1:5 010 rd:5 0101111" "asm"="amocas.w.aq {rd}, {rs2}, ({rs1})" "asmonly" {
  ret void
}

define void @AMOCAS_W_RL.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1, i64 "reg"="x" %rs2) "enc"="0010101 rs2:5 rs1:5 010 rd:5 0101111" "asm"="amocas.w.rl {rd}, {rs2}, ({rs1})" "asmonly" {
  ret void
}

define void @AMOCAS_W_AQRL.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1, i64 "reg"="x" %rs2) "enc"="0010111 rs2:5 rs1:5 010 rd:5 0101111" "asm"="amocas.w.aqrl {rd}, {rs2}, ({rs1})" "asmonly" {
  ret void
}

define void @AMOCAS_D.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1, i64 "reg"="x" %rs2) "enc"="0010100 rs2:5 rs1:5 011 rd:5 0101111" "asm"="amocas.d {rd}, {rs2}, ({rs1})" "asmonly" {
  ret void
}

define void @AMOCAS_D_AQ.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1, i64 "reg"="x" %rs2) "enc"="0010110 rs2:5 rs1:5 011 rd:5 0101111" "asm"="amocas.d.aq {rd}, {rs2}, ({rs1})" "asmonly" {
  ret void
}

define void @AMOCAS_D_RL.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1, i64 "reg"="x" %rs2) "enc"="0010101 rs2:5 rs1:5 011 rd:5 0101111" "asm"="amocas.d.rl {rd}, {rs2}, ({rs1})" "asmonly" {
  ret void
}

define void @AMOCAS_D_AQRL.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1, i64 "reg"="x" %rs2) "enc"="0010111 rs2:5 rs1:5 011 rd:5 0101111" "asm"="amocas.d.aqrl {rd}, {rs2}, ({rs1})" "asmonly" {
  ret void
}

define void @AMOCAS_Q.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1, i64 "reg"="x" %rs2) "enc"="{ if(rd%2==0) if(rs2%2==0) 0010100 rs2:5 rs1:5 100 rd:5 0101111 }" "asm"="amocas.q {rd}, {rs2}, ({rs1})" "asmonly" {
  ret void
}

define void @AMOCAS_Q_AQ.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1, i64 "reg"="x" %rs2) "enc"="{ if(rd%2==0) if(rs2%2==0) 0010110 rs2:5 rs1:5 100 rd:5 0101111 }" "asm"="amocas.q.aq {rd}, {rs2}, ({rs1})" "asmonly" {
  ret void
}

define void @AMOCAS_Q_RL.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1, i64 "reg"="x" %rs2) "enc"="{ if(rd%2==0) if(rs2%2==0) 0010101 rs2:5 rs1:5 100 rd:5 0101111 }" "asm"="amocas.q.rl {rd}, {rs2}, ({rs1})" "asmonly" {
  ret void
}

define void @AMOCAS_Q_AQRL.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1, i64 "reg"="x" %rs2) "enc"="{ if(rd%2==0) if(rs2%2==0) 0010111 rs2:5 rs1:5 100 rd:5 0101111 }" "asm"="amocas.q.aqrl {rd}, {rs2}, ({rs1})" "asmonly" {
  ret void
}

define void @AMOSWAP_B.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1, i64 "reg"="x" %rs2) "enc"="0000100 rs2:5 rs1:5 000 rd:5 0101111" "asm"="amoswap.b {rd}, {rs2}, ({rs1})" "asmonly" {
  ret void
}

define void @AMOSWAP_B_AQ.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1, i64 "reg"="x" %rs2) "enc"="0000110 rs2:5 rs1:5 000 rd:5 0101111" "asm"="amoswap.b.aq {rd}, {rs2}, ({rs1})" "asmonly" {
  ret void
}

define void @AMOSWAP_B_RL.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1, i64 "reg"="x" %rs2) "enc"="0000101 rs2:5 rs1:5 000 rd:5 0101111" "asm"="amoswap.b.rl {rd}, {rs2}, ({rs1})" "asmonly" {
  ret void
}

define void @AMOSWAP_B_AQRL.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1, i64 "reg"="x" %rs2) "enc"="0000111 rs2:5 rs1:5 000 rd:5 0101111" "asm"="amoswap.b.aqrl {rd}, {rs2}, ({rs1})" "asmonly" {
  ret void
}

define void @AMOADD_B.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1, i64 "reg"="x" %rs2) "enc"="0000000 rs2:5 rs1:5 000 rd:5 0101111" "asm"="amoadd.b {rd}, {rs2}, ({rs1})" "asmonly" {
  ret void
}

define void @AMOADD_B_AQ.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1, i64 "reg"="x" %rs2) "enc"="0000010 rs2:5 rs1:5 000 rd:5 0101111" "asm"="amoadd.b.aq {rd}, {rs2}, ({rs1})" "asmonly" {
  ret void
}

define void @AMOADD_B_RL.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1, i64 "reg"="x" %rs2) "enc"="0000001 rs2:5 rs1:5 000 rd:5 0101111" "asm"="amoadd.b.rl {rd}, {rs2}, ({rs1})" "asmonly" {
  ret void
}

define void @AMOADD_B_AQRL.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1, i64 "reg"="x" %rs2) "enc"="0000011 rs2:5 rs1:5 000 rd:5 0101111" "asm"="amoadd.b.aqrl {rd}, {rs2}, ({rs1})" "asmonly" {
  ret void
}

define void @AMOXOR_B.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1, i64 "reg"="x" %rs2) "enc"="0010000 rs2:5 rs1:5 000 rd:5 0101111" "asm"="amoxor.b {rd}, {rs2}, ({rs1})" "asmonly" {
  ret void
}

define void @AMOXOR_B_AQ.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1, i64 "reg"="x" %rs2) "enc"="0010010 rs2:5 rs1:5 000 rd:5 0101111" "asm"="amoxor.b.aq {rd}, {rs2}, ({rs1})" "asmonly" {
  ret void
}

define void @AMOXOR_B_RL.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1, i64 "reg"="x" %rs2) "enc"="0010001 rs2:5 rs1:5 000 rd:5 0101111" "asm"="amoxor.b.rl {rd}, {rs2}, ({rs1})" "asmonly" {
  ret void
}

define void @AMOXOR_B_AQRL.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1, i64 "reg"="x" %rs2) "enc"="0010011 rs2:5 rs1:5 000 rd:5 0101111" "asm"="amoxor.b.aqrl {rd}, {rs2}, ({rs1})" "asmonly" {
  ret void
}

define void @AMOAND_B.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1, i64 "reg"="x" %rs2) "enc"="0110000 rs2:5 rs1:5 000 rd:5 0101111" "asm"="amoand.b {rd}, {rs2}, ({rs1})" "asmonly" {
  ret void
}

define void @AMOAND_B_AQ.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1, i64 "reg"="x" %rs2) "enc"="0110010 rs2:5 rs1:5 000 rd:5 0101111" "asm"="amoand.b.aq {rd}, {rs2}, ({rs1})" "asmonly" {
  ret void
}

define void @AMOAND_B_RL.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1, i64 "reg"="x" %rs2) "enc"="0110001 rs2:5 rs1:5 000 rd:5 0101111" "asm"="amoand.b.rl {rd}, {rs2}, ({rs1})" "asmonly" {
  ret void
}

define void @AMOAND_B_AQRL.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1, i64 "reg"="x" %rs2) "enc"="0110011 rs2:5 rs1:5 000 rd:5 0101111" "asm"="amoand.b.aqrl {rd}, {rs2}, ({rs1})" "asmonly" {
  ret void
}

define void @AMOOR_B.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1, i64 "reg"="x" %rs2) "enc"="0100000 rs2:5 rs1:5 000 rd:5 0101111" "asm"="amoor.b {rd}, {rs2}, ({rs1})" "asmonly" {
  ret void
}

define void @AMOOR_B_AQ.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1, i64 "reg"="x" %rs2) "enc"="0100010 rs2:5 rs1:5 000 rd:5 0101111" "asm"="amoor.b.aq {rd}, {rs2}, ({rs1})" "asmonly" {
  ret void
}

define void @AMOOR_B_RL.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1, i64 "reg"="x" %rs2) "enc"="0100001 rs2:5 rs1:5 000 rd:5 0101111" "asm"="amoor.b.rl {rd}, {rs2}, ({rs1})" "asmonly" {
  ret void
}

define void @AMOOR_B_AQRL.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1, i64 "reg"="x" %rs2) "enc"="0100011 rs2:5 rs1:5 000 rd:5 0101111" "asm"="amoor.b.aqrl {rd}, {rs2}, ({rs1})" "asmonly" {
  ret void
}

define void @AMOMIN_B.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1, i64 "reg"="x" %rs2) "enc"="1000000 rs2:5 rs1:5 000 rd:5 0101111" "asm"="amomin.b {rd}, {rs2}, ({rs1})" "asmonly" {
  ret void
}

define void @AMOMIN_B_AQ.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1, i64 "reg"="x" %rs2) "enc"="1000010 rs2:5 rs1:5 000 rd:5 0101111" "asm"="amomin.b.aq {rd}, {rs2}, ({rs1})" "asmonly" {
  ret void
}

define void @AMOMIN_B_RL.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1, i64 "reg"="x" %rs2) "enc"="1000001 rs2:5 rs1:5 000 rd:5 0101111" "asm"="amomin.b.rl {rd}, {rs2}, ({rs1})" "asmonly" {
  ret void
}

define void @AMOMIN_B_AQRL.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1, i64 "reg"="x" %rs2) "enc"="1000011 rs2:5 rs1:5 000 rd:5 0101111" "asm"="amomin.b.aqrl {rd}, {rs2}, ({rs1})" "asmonly" {
  ret void
}

define void @AMOMAX_B.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1, i64 "reg"="x" %rs2) "enc"="1010000 rs2:5 rs1:5 000 rd:5 0101111" "asm"="amomax.b {rd}, {rs2}, ({rs1})" "asmonly" {
  ret void
}

define void @AMOMAX_B_AQ.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1, i64 "reg"="x" %rs2) "enc"="1010010 rs2:5 rs1:5 000 rd:5 0101111" "asm"="amomax.b.aq {rd}, {rs2}, ({rs1})" "asmonly" {
  ret void
}

define void @AMOMAX_B_RL.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1, i64 "reg"="x" %rs2) "enc"="1010001 rs2:5 rs1:5 000 rd:5 0101111" "asm"="amomax.b.rl {rd}, {rs2}, ({rs1})" "asmonly" {
  ret void
}

define void @AMOMAX_B_AQRL.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1, i64 "reg"="x" %rs2) "enc"="1010011 rs2:5 rs1:5 000 rd:5 0101111" "asm"="amomax.b.aqrl {rd}, {rs2}, ({rs1})" "asmonly" {
  ret void
}

define void @AMOMINU_B.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1, i64 "reg"="x" %rs2) "enc"="1100000 rs2:5 rs1:5 000 rd:5 0101111" "asm"="amominu.b {rd}, {rs2}, ({rs1})" "asmonly" {
  ret void
}

define void @AMOMINU_B_AQ.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1, i64 "reg"="x" %rs2) "enc"="1100010 rs2:5 rs1:5 000 rd:5 0101111" "asm"="amominu.b.aq {rd}, {rs2}, ({rs1})" "asmonly" {
  ret void
}

define void @AMOMINU_B_RL.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1, i64 "reg"="x" %rs2) "enc"="1100001 rs2:5 rs1:5 000 rd:5 0101111" "asm"="amominu.b.rl {rd}, {rs2}, ({rs1})" "asmonly" {
  ret void
}

define void @AMOMINU_B_AQRL.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1, i64 "reg"="x" %rs2) "enc"="1100011 rs2:5 rs1:5 000 rd:5 0101111" "asm"="amominu.b.aqrl {rd}, {rs2}, ({rs1})" "asmonly" {
  ret void
}

define void @AMOMAXU_B.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1, i64 "reg"="x" %rs2) "enc"="1110000 rs2:5 rs1:5 000 rd:5 0101111" "asm"="amomaxu.b {rd}, {rs2}, ({rs1})" "asmonly" {
  ret void
}

define void @AMOMAXU_B_AQ.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1, i64 "reg"="x" %rs2) "enc"="1110010 rs2:5 rs1:5 000 rd:5 0101111" "asm"="amomaxu.b.aq {rd}, {rs2}, ({rs1})" "asmonly" {
  ret void
}

define void @AMOMAXU_B_RL.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1, i64 "reg"="x" %rs2) "enc"="1110001 rs2:5 rs1:5 000 rd:5 0101111" "asm"="amomaxu.b.rl {rd}, {rs2}, ({rs1})" "asmonly" {
  ret void
}

define void @AMOMAXU_B_AQRL.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1, i64 "reg"="x" %rs2) "enc"="1110011 rs2:5 rs1:5 000 rd:5 0101111" "asm"="amomaxu.b.aqrl {rd}, {rs2}, ({rs1})" "asmonly" {
  ret void
}

define void @AMOSWAP_H.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1, i64 "reg"="x" %rs2) "enc"="0000100 rs2:5 rs1:5 001 rd:5 0101111" "asm"="amoswap.h {rd}, {rs2}, ({rs1})" "asmonly" {
  ret void
}

define void @AMOSWAP_H_AQ.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1, i64 "reg"="x" %rs2) "enc"="0000110 rs2:5 rs1:5 001 rd:5 0101111" "asm"="amoswap.h.aq {rd}, {rs2}, ({rs1})" "asmonly" {
  ret void
}

define void @AMOSWAP_H_RL.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1, i64 "reg"="x" %rs2) "enc"="0000101 rs2:5 rs1:5 001 rd:5 0101111" "asm"="amoswap.h.rl {rd}, {rs2}, ({rs1})" "asmonly" {
  ret void
}

define void @AMOSWAP_H_AQRL.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1, i64 "reg"="x" %rs2) "enc"="0000111 rs2:5 rs1:5 001 rd:5 0101111" "asm"="amoswap.h.aqrl {rd}, {rs2}, ({rs1})" "asmonly" {
  ret void
}

define void @AMOADD_H.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1, i64 "reg"="x" %rs2) "enc"="0000000 rs2:5 rs1:5 001 rd:5 0101111" "asm"="amoadd.h {rd}, {rs2}, ({rs1})" "asmonly" {
  ret void
}

define void @AMOADD_H_AQ.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1, i64 "reg"="x" %rs2) "enc"="0000010 rs2:5 rs1:5 001 rd:5 0101111" "asm"="amoadd.h.aq {rd}, {rs2}, ({rs1})" "asmonly" {
  ret void
}

define void @AMOADD_H_RL.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1, i64 "reg"="x" %rs2) "enc"="0000001 rs2:5 rs1:5 001 rd:5 0101111" "asm"="amoadd.h.rl {rd}, {rs2}, ({rs1})" "asmonly" {
  ret void
}

define void @AMOADD_H_AQRL.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1, i64 "reg"="x" %rs2) "enc"="0000011 rs2:5 rs1:5 001 rd:5 0101111" "asm"="amoadd.h.aqrl {rd}, {rs2}, ({rs1})" "asmonly" {
  ret void
}

define void @AMOXOR_H.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1, i64 "reg"="x" %rs2) "enc"="0010000 rs2:5 rs1:5 001 rd:5 0101111" "asm"="amoxor.h {rd}, {rs2}, ({rs1})" "asmonly" {
  ret void
}

define void @AMOXOR_H_AQ.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1, i64 "reg"="x" %rs2) "enc"="0010010 rs2:5 rs1:5 001 rd:5 0101111" "asm"="amoxor.h.aq {rd}, {rs2}, ({rs1})" "asmonly" {
  ret void
}

define void @AMOXOR_H_RL.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1, i64 "reg"="x" %rs2) "enc"="0010001 rs2:5 rs1:5 001 rd:5 0101111" "asm"="amoxor.h.rl {rd}, {rs2}, ({rs1})" "asmonly" {
  ret void
}

define void @AMOXOR_H_AQRL.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1, i64 "reg"="x" %rs2) "enc"="0010011 rs2:5 rs1:5 001 rd:5 0101111" "asm"="amoxor.h.aqrl {rd}, {rs2}, ({rs1})" "asmonly" {
  ret void
}

define void @AMOAND_H.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1, i64 "reg"="x" %rs2) "enc"="0110000 rs2:5 rs1:5 001 rd:5 0101111" "asm"="amoand.h {rd}, {rs2}, ({rs1})" "asmonly" {
  ret void
}

define void @AMOAND_H_AQ.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1, i64 "reg"="x" %rs2) "enc"="0110010 rs2:5 rs1:5 001 rd:5 0101111" "asm"="amoand.h.aq {rd}, {rs2}, ({rs1})" "asmonly" {
  ret void
}

define void @AMOAND_H_RL.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1, i64 "reg"="x" %rs2) "enc"="0110001 rs2:5 rs1:5 001 rd:5 0101111" "asm"="amoand.h.rl {rd}, {rs2}, ({rs1})" "asmonly" {
  ret void
}

define void @AMOAND_H_AQRL.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1, i64 "reg"="x" %rs2) "enc"="0110011 rs2:5 rs1:5 001 rd:5 0101111" "asm"="amoand.h.aqrl {rd}, {rs2}, ({rs1})" "asmonly" {
  ret void
}

define void @AMOOR_H.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1, i64 "reg"="x" %rs2) "enc"="0100000 rs2:5 rs1:5 001 rd:5 0101111" "asm"="amoor.h {rd}, {rs2}, ({rs1})" "asmonly" {
  ret void
}

define void @AMOOR_H_AQ.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1, i64 "reg"="x" %rs2) "enc"="0100010 rs2:5 rs1:5 001 rd:5 0101111" "asm"="amoor.h.aq {rd}, {rs2}, ({rs1})" "asmonly" {
  ret void
}

define void @AMOOR_H_RL.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1, i64 "reg"="x" %rs2) "enc"="0100001 rs2:5 rs1:5 001 rd:5 0101111" "asm"="amoor.h.rl {rd}, {rs2}, ({rs1})" "asmonly" {
  ret void
}

define void @AMOOR_H_AQRL.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1, i64 "reg"="x" %rs2) "enc"="0100011 rs2:5 rs1:5 001 rd:5 0101111" "asm"="amoor.h.aqrl {rd}, {rs2}, ({rs1})" "asmonly" {
  ret void
}

define void @AMOMIN_H.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1, i64 "reg"="x" %rs2) "enc"="1000000 rs2:5 rs1:5 001 rd:5 0101111" "asm"="amomin.h {rd}, {rs2}, ({rs1})" "asmonly" {
  ret void
}

define void @AMOMIN_H_AQ.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1, i64 "reg"="x" %rs2) "enc"="1000010 rs2:5 rs1:5 001 rd:5 0101111" "asm"="amomin.h.aq {rd}, {rs2}, ({rs1})" "asmonly" {
  ret void
}

define void @AMOMIN_H_RL.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1, i64 "reg"="x" %rs2) "enc"="1000001 rs2:5 rs1:5 001 rd:5 0101111" "asm"="amomin.h.rl {rd}, {rs2}, ({rs1})" "asmonly" {
  ret void
}

define void @AMOMIN_H_AQRL.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1, i64 "reg"="x" %rs2) "enc"="1000011 rs2:5 rs1:5 001 rd:5 0101111" "asm"="amomin.h.aqrl {rd}, {rs2}, ({rs1})" "asmonly" {
  ret void
}

define void @AMOMAX_H.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1, i64 "reg"="x" %rs2) "enc"="1010000 rs2:5 rs1:5 001 rd:5 0101111" "asm"="amomax.h {rd}, {rs2}, ({rs1})" "asmonly" {
  ret void
}

define void @AMOMAX_H_AQ.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1, i64 "reg"="x" %rs2) "enc"="1010010 rs2:5 rs1:5 001 rd:5 0101111" "asm"="amomax.h.aq {rd}, {rs2}, ({rs1})" "asmonly" {
  ret void
}

define void @AMOMAX_H_RL.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1, i64 "reg"="x" %rs2) "enc"="1010001 rs2:5 rs1:5 001 rd:5 0101111" "asm"="amomax.h.rl {rd}, {rs2}, ({rs1})" "asmonly" {
  ret void
}

define void @AMOMAX_H_AQRL.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1, i64 "reg"="x" %rs2) "enc"="1010011 rs2:5 rs1:5 001 rd:5 0101111" "asm"="amomax.h.aqrl {rd}, {rs2}, ({rs1})" "asmonly" {
  ret void
}

define void @AMOMINU_H.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1, i64 "reg"="x" %rs2) "enc"="1100000 rs2:5 rs1:5 001 rd:5 0101111" "asm"="amominu.h {rd}, {rs2}, ({rs1})" "asmonly" {
  ret void
}

define void @AMOMINU_H_AQ.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1, i64 "reg"="x" %rs2) "enc"="1100010 rs2:5 rs1:5 001 rd:5 0101111" "asm"="amominu.h.aq {rd}, {rs2}, ({rs1})" "asmonly" {
  ret void
}

define void @AMOMINU_H_RL.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1, i64 "reg"="x" %rs2) "enc"="1100001 rs2:5 rs1:5 001 rd:5 0101111" "asm"="amominu.h.rl {rd}, {rs2}, ({rs1})" "asmonly" {
  ret void
}

define void @AMOMINU_H_AQRL.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1, i64 "reg"="x" %rs2) "enc"="1100011 rs2:5 rs1:5 001 rd:5 0101111" "asm"="amominu.h.aqrl {rd}, {rs2}, ({rs1})" "asmonly" {
  ret void
}

define void @AMOMAXU_H.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1, i64 "reg"="x" %rs2) "enc"="1110000 rs2:5 rs1:5 001 rd:5 0101111" "asm"="amomaxu.h {rd}, {rs2}, ({rs1})" "asmonly" {
  ret void
}

define void @AMOMAXU_H_AQ.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1, i64 "reg"="x" %rs2) "enc"="1110010 rs2:5 rs1:5 001 rd:5 0101111" "asm"="amomaxu.h.aq {rd}, {rs2}, ({rs1})" "asmonly" {
  ret void
}

define void @AMOMAXU_H_RL.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1, i64 "reg"="x" %rs2) "enc"="1110001 rs2:5 rs1:5 001 rd:5 0101111" "asm"="amomaxu.h.rl {rd}, {rs2}, ({rs1})" "asmonly" {
  ret void
}

define void @AMOMAXU_H_AQRL.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1, i64 "reg"="x" %rs2) "enc"="1110011 rs2:5 rs1:5 001 rd:5 0101111" "asm"="amomaxu.h.aqrl {rd}, {rs2}, ({rs1})" "asmonly" {
  ret void
}

define void @AMOCAS_B.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1, i64 "reg"="x" %rs2) "enc"="0010100 rs2:5 rs1:5 000 rd:5 0101111" "asm"="amocas.b {rd}, {rs2}, ({rs1})" "asmonly" {
  ret void
}

define void @AMOCAS_B_AQ.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1, i64 "reg"="x" %rs2) "enc"="0010110 rs2:5 rs1:5 000 rd:5 0101111" "asm"="amocas.b.aq {rd}, {rs2}, ({rs1})" "asmonly" {
  ret void
}

define void @AMOCAS_B_RL.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1, i64 "reg"="x" %rs2) "enc"="0010101 rs2:5 rs1:5 000 rd:5 0101111" "asm"="amocas.b.rl {rd}, {rs2}, ({rs1})" "asmonly" {
  ret void
}

define void @AMOCAS_B_AQRL.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1, i64 "reg"="x" %rs2) "enc"="0010111 rs2:5 rs1:5 000 rd:5 0101111" "asm"="amocas.b.aqrl {rd}, {rs2}, ({rs1})" "asmonly" {
  ret void
}

define void @AMOCAS_H.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1, i64 "reg"="x" %rs2) "enc"="0010100 rs2:5 rs1:5 001 rd:5 0101111" "asm"="amocas.h {rd}, {rs2}, ({rs1})" "asmonly" {
  ret void
}

define void @AMOCAS_H_AQ.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1, i64 "reg"="x" %rs2) "enc"="0010110 rs2:5 rs1:5 001 rd:5 0101111" "asm"="amocas.h.aq {rd}, {rs2}, ({rs1})" "asmonly" {
  ret void
}

define void @AMOCAS_H_RL.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1, i64 "reg"="x" %rs2) "enc"="0010101 rs2:5 rs1:5 001 rd:5 0101111" "asm"="amocas.h.rl {rd}, {rs2}, ({rs1})" "asmonly" {
  ret void
}

define void @AMOCAS_H_AQRL.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1, i64 "reg"="x" %rs2) "enc"="0010111 rs2:5 rs1:5 001 rd:5 0101111" "asm"="amocas.h.aqrl {rd}, {rs2}, ({rs1})" "asmonly" {
  ret void
}

define void @CBO_CLEAN.0(i64 "reg"="x" %rs1) "enc"="000000000001 rs1:5 010000000001111" "asm"="cbo.clean ({rs1})" "asmonly" {
  ret void
}

define void @CBO_FLUSH.0(i64 "reg"="x" %rs1) "enc"="000000000010 rs1:5 010000000001111" "asm"="cbo.flush ({rs1})" "asmonly" {
  ret void
}

define void @CBO_INVAL.0(i64 "reg"="x" %rs1) "enc"="000000000000 rs1:5 010000000001111" "asm"="cbo.inval ({rs1})" "asmonly" {
  ret void
}

define void @CBO_ZERO.0(i64 "reg"="x" %rs1) "enc"="000000000100 rs1:5 010000000001111" "asm"="cbo.zero ({rs1})" "asmonly" {
  ret void
}

define void @AES64DSM.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1, i64 "reg"="x" %rs2) "enc"="0011111 rs2:5 rs1:5 000 rd:5 0110011" "asm"="aes64dsm {rd}, {rs1}, {rs2}" "asmonly" {
  ret void
}

define void @AES64DS.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1, i64 "reg"="x" %rs2) "enc"="0011101 rs2:5 rs1:5 000 rd:5 0110011" "asm"="aes64ds {rd}, {rs1}, {rs2}" "asmonly" {
  ret void
}

define void @AES64KS1I.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1, i64 "imm" %rnum) "enc"="00110001 rnum:4 rs1:5 001 rd:5 0010011" "asm"="aes64ks1i {rd}, {rs1}, {rnum}" "asmonly" {
  ret void
}

define void @AES64IM.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1) "enc"="001100000000 rs1:5 001 rd:5 0010011" "asm"="aes64im {rd}, {rs1}" "asmonly" {
  ret void
}

define void @AES64KS2.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1, i64 "reg"="x" %rs2) "enc"="0111111 rs2:5 rs1:5 000 rd:5 0110011" "asm"="aes64ks2 {rd}, {rs1}, {rs2}" "asmonly" {
  ret void
}

define void @AES64ESM.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1, i64 "reg"="x" %rs2) "enc"="0011011 rs2:5 rs1:5 000 rd:5 0110011" "asm"="aes64esm {rd}, {rs1}, {rs2}" "asmonly" {
  ret void
}

define void @AES64ES.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1, i64 "reg"="x" %rs2) "enc"="0011001 rs2:5 rs1:5 000 rd:5 0110011" "asm"="aes64es {rd}, {rs1}, {rs2}" "asmonly" {
  ret void
}

define void @SHA256SUM0.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1) "enc"="000100000000 rs1:5 001 rd:5 0010011" "asm"="sha256sum0 {rd}, {rs1}" "asmonly" {
  ret void
}

define void @SHA256SUM1.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1) "enc"="000100000001 rs1:5 001 rd:5 0010011" "asm"="sha256sum1 {rd}, {rs1}" "asmonly" {
  ret void
}

define void @SHA256SIG0.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1) "enc"="000100000010 rs1:5 001 rd:5 0010011" "asm"="sha256sig0 {rd}, {rs1}" "asmonly" {
  ret void
}

define void @SHA256SIG1.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1) "enc"="000100000011 rs1:5 001 rd:5 0010011" "asm"="sha256sig1 {rd}, {rs1}" "asmonly" {
  ret void
}

define void @SHA512SUM0.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1) "enc"="000100000100 rs1:5 001 rd:5 0010011" "asm"="sha512sum0 {rd}, {rs1}" "asmonly" {
  ret void
}

define void @SHA512SUM1.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1) "enc"="000100000101 rs1:5 001 rd:5 0010011" "asm"="sha512sum1 {rd}, {rs1}" "asmonly" {
  ret void
}

define void @SHA512SIG0.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1) "enc"="000100000110 rs1:5 001 rd:5 0010011" "asm"="sha512sig0 {rd}, {rs1}" "asmonly" {
  ret void
}

define void @SHA512SIG1.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1) "enc"="000100000111 rs1:5 001 rd:5 0010011" "asm"="sha512sig1 {rd}, {rs1}" "asmonly" {
  ret void
}

define void @SM4ED.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1, i64 "reg"="x" %rs2, i64 "imm" %bs) "enc"="bs:2 11000 rs2:5 rs1:5 000 rd:5 0110011" "asm"="sm4ed {rd}, {rs1}, {rs2}, {bs}" "asmonly" {
  ret void
}

define void @SM4KS.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1, i64 "reg"="x" %rs2, i64 "imm" %bs) "enc"="bs:2 11010 rs2:5 rs1:5 000 rd:5 0110011" "asm"="sm4ks {rd}, {rs1}, {rs2}, {bs}" "asmonly" {
  ret void
}

define void @SM3P0.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1) "enc"="000100001000 rs1:5 001 rd:5 0010011" "asm"="sm3p0 {rd}, {rs1}" "asmonly" {
  ret void
}

define void @SM3P1.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1) "enc"="000100001001 rs1:5 001 rd:5 0010011" "asm"="sm3p1 {rd}, {rs1}" "asmonly" {
  ret void
}

define void @MRET.0() "enc"="00110000001000000000000001110011" "asm"="mret" "asmonly" {
  ret void
}

define void @WFI.0() "enc"="00010000010100000000000001110011" "asm"="wfi" "asmonly" {
  ret void
}

define void @SFENCE_VMA.0() "enc"="00010010000000000000000001110011" "asm"="sfence.vma" "asmonly" {
  ret void
}

define void @SFENCE_VMA.1(i64 "reg"="x" %rs1, i64 "reg"="x" %rs2) "enc"="0001001 rs2:5 rs1:5 000000001110011" "asm"="sfence.vma {rs1}, {rs2}" "asmonly" {
  ret void
}

define void @SRET.0() "enc"="00010000001000000000000001110011" "asm"="sret" "asmonly" {
  ret void
}

define void @SINVAL_VMA.0(i64 "reg"="x" %rs1, i64 "reg"="x" %rs2) "enc"="0001011 rs2:5 rs1:5 000000001110011" "asm"="sinval.vma {rs1}, {rs2}" "asmonly" {
  ret void
}

define void @SFENCE_W_INVAL.0() "enc"="00011000000000000000000001110011" "asm"="sfence.w.inval" "asmonly" {
  ret void
}

define void @SFENCE_INVAL_IR.0() "enc"="00011000000100000000000001110011" "asm"="sfence.inval.ir" "asmonly" {
  ret void
}

define void @LB_AQ.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1) "enc"="001101000000 rs1:5 000 rd:5 0101111" "asm"="lb.aq {rd}, ({rs1})" "asmonly" {
  ret void
}

define void @LB_AQRL.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1) "enc"="001101100000 rs1:5 000 rd:5 0101111" "asm"="lb.aqrl {rd}, ({rs1})" "asmonly" {
  ret void
}

define void @LH_AQ.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1) "enc"="001101000000 rs1:5 001 rd:5 0101111" "asm"="lh.aq {rd}, ({rs1})" "asmonly" {
  ret void
}

define void @LH_AQRL.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1) "enc"="001101100000 rs1:5 001 rd:5 0101111" "asm"="lh.aqrl {rd}, ({rs1})" "asmonly" {
  ret void
}

define void @LW_AQ.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1) "enc"="001101000000 rs1:5 010 rd:5 0101111" "asm"="lw.aq {rd}, ({rs1})" "asmonly" {
  ret void
}

define void @LW_AQRL.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1) "enc"="001101100000 rs1:5 010 rd:5 0101111" "asm"="lw.aqrl {rd}, ({rs1})" "asmonly" {
  ret void
}

define void @LD_AQ.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1) "enc"="001101000000 rs1:5 011 rd:5 0101111" "asm"="ld.aq {rd}, ({rs1})" "asmonly" {
  ret void
}

define void @LD_AQRL.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1) "enc"="001101100000 rs1:5 011 rd:5 0101111" "asm"="ld.aqrl {rd}, ({rs1})" "asmonly" {
  ret void
}

define void @SB_RL.0(i64 "reg"="x" %rs1, i64 "reg"="x" %rs2) "enc"="0011101 rs2:5 rs1:5 000000000101111" "asm"="sb.rl {rs2}, ({rs1})" "asmonly" {
  ret void
}

define void @SB_AQRL.0(i64 "reg"="x" %rs1, i64 "reg"="x" %rs2) "enc"="0011111 rs2:5 rs1:5 000000000101111" "asm"="sb.aqrl {rs2}, ({rs1})" "asmonly" {
  ret void
}

define void @SH_RL.0(i64 "reg"="x" %rs1, i64 "reg"="x" %rs2) "enc"="0011101 rs2:5 rs1:5 001000000101111" "asm"="sh.rl {rs2}, ({rs1})" "asmonly" {
  ret void
}

define void @SH_AQRL.0(i64 "reg"="x" %rs1, i64 "reg"="x" %rs2) "enc"="0011111 rs2:5 rs1:5 001000000101111" "asm"="sh.aqrl {rs2}, ({rs1})" "asmonly" {
  ret void
}

define void @SW_RL.0(i64 "reg"="x" %rs1, i64 "reg"="x" %rs2) "enc"="0011101 rs2:5 rs1:5 010000000101111" "asm"="sw.rl {rs2}, ({rs1})" "asmonly" {
  ret void
}

define void @SW_AQRL.0(i64 "reg"="x" %rs1, i64 "reg"="x" %rs2) "enc"="0011111 rs2:5 rs1:5 010000000101111" "asm"="sw.aqrl {rs2}, ({rs1})" "asmonly" {
  ret void
}

define void @SD_RL.0(i64 "reg"="x" %rs1, i64 "reg"="x" %rs2) "enc"="0011101 rs2:5 rs1:5 011000000101111" "asm"="sd.rl {rs2}, ({rs1})" "asmonly" {
  ret void
}

define void @SD_AQRL.0(i64 "reg"="x" %rs1, i64 "reg"="x" %rs2) "enc"="0011111 rs2:5 rs1:5 011000000101111" "asm"="sd.aqrl {rs2}, ({rs1})" "asmonly" {
  ret void
}

define void @SSAMOSWAP_W.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1, i64 "reg"="x" %rs2) "enc"="0100100 rs2:5 rs1:5 010 rd:5 0101111" "asm"="ssamoswap.w {rd}, {rs2}, ({rs1})" "asmonly" {
  ret void
}

define void @SSAMOSWAP_W_AQ.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1, i64 "reg"="x" %rs2) "enc"="0100110 rs2:5 rs1:5 010 rd:5 0101111" "asm"="ssamoswap.w.aq {rd}, {rs2}, ({rs1})" "asmonly" {
  ret void
}

define void @SSAMOSWAP_W_RL.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1, i64 "reg"="x" %rs2) "enc"="0100101 rs2:5 rs1:5 010 rd:5 0101111" "asm"="ssamoswap.w.rl {rd}, {rs2}, ({rs1})" "asmonly" {
  ret void
}

define void @SSAMOSWAP_W_AQRL.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1, i64 "reg"="x" %rs2) "enc"="0100111 rs2:5 rs1:5 010 rd:5 0101111" "asm"="ssamoswap.w.aqrl {rd}, {rs2}, ({rs1})" "asmonly" {
  ret void
}

define void @SSAMOSWAP_D.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1, i64 "reg"="x" %rs2) "enc"="0100100 rs2:5 rs1:5 011 rd:5 0101111" "asm"="ssamoswap.d {rd}, {rs2}, ({rs1})" "asmonly" {
  ret void
}

define void @SSAMOSWAP_D_AQ.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1, i64 "reg"="x" %rs2) "enc"="0100110 rs2:5 rs1:5 011 rd:5 0101111" "asm"="ssamoswap.d.aq {rd}, {rs2}, ({rs1})" "asmonly" {
  ret void
}

define void @SSAMOSWAP_D_RL.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1, i64 "reg"="x" %rs2) "enc"="0100101 rs2:5 rs1:5 011 rd:5 0101111" "asm"="ssamoswap.d.rl {rd}, {rs2}, ({rs1})" "asmonly" {
  ret void
}

define void @SSAMOSWAP_D_AQRL.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1, i64 "reg"="x" %rs2) "enc"="0100111 rs2:5 rs1:5 011 rd:5 0101111" "asm"="ssamoswap.d.aqrl {rd}, {rs2}, ({rs1})" "asmonly" {
  ret void
}

define void @FLQ.0(i64 "reg"="d" %rd, i64 "reg"="x" %rs1, i64 "imm" %imm12) "enc"="imm12:s12 rs1:5 100 rd:5 0000111" "asm"="flq {rd}, {imm12}({rs1})" "asmonly" {
  ret void
}

define void @FSQ.0(i64 "reg"="x" %rs1, i64 "reg"="d" %rs2, i64 "imm" %off) "enc"="o_hi:s7=off>>5 rs2:5 rs1:5 100 o_lo:5=off&31 0100111" "asm"="fsq {rs2}, {off}({rs1})" "asmonly" {
  ret void
}

define void @FMADD_Q.0(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2, i64 "reg"="d" %rs3) "enc"="rs3:5 11 rs2:5 rs1:5 111 rd:5 1000011" "asm"="fmadd.q {rd}, {rs1}, {rs2}, {rs3}" "asmonly" {
  ret void
}

define void @FMADD_Q.1(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2, i64 "reg"="d" %rs3) "enc"="rs3:5 11 rs2:5 rs1:5 000 rd:5 1000011" "asm"="fmadd.q {rd}, {rs1}, {rs2}, {rs3}, rne" "asmonly" {
  ret void
}

define void @FMADD_Q.2(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2, i64 "reg"="d" %rs3) "enc"="rs3:5 11 rs2:5 rs1:5 001 rd:5 1000011" "asm"="fmadd.q {rd}, {rs1}, {rs2}, {rs3}, rtz" "asmonly" {
  ret void
}

define void @FMADD_Q.3(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2, i64 "reg"="d" %rs3) "enc"="rs3:5 11 rs2:5 rs1:5 010 rd:5 1000011" "asm"="fmadd.q {rd}, {rs1}, {rs2}, {rs3}, rdn" "asmonly" {
  ret void
}

define void @FMADD_Q.4(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2, i64 "reg"="d" %rs3) "enc"="rs3:5 11 rs2:5 rs1:5 011 rd:5 1000011" "asm"="fmadd.q {rd}, {rs1}, {rs2}, {rs3}, rup" "asmonly" {
  ret void
}

define void @FMADD_Q.5(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2, i64 "reg"="d" %rs3) "enc"="rs3:5 11 rs2:5 rs1:5 100 rd:5 1000011" "asm"="fmadd.q {rd}, {rs1}, {rs2}, {rs3}, rmm" "asmonly" {
  ret void
}

define void @FMADD_Q.6(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2, i64 "reg"="d" %rs3) "enc"="rs3:5 11 rs2:5 rs1:5 111 rd:5 1000011" "asm"="fmadd.q {rd}, {rs1}, {rs2}, {rs3}, dyn" "asmonly" "nodecode" {
  ret void
}

define void @FMSUB_Q.0(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2, i64 "reg"="d" %rs3) "enc"="rs3:5 11 rs2:5 rs1:5 111 rd:5 1000111" "asm"="fmsub.q {rd}, {rs1}, {rs2}, {rs3}" "asmonly" {
  ret void
}

define void @FMSUB_Q.1(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2, i64 "reg"="d" %rs3) "enc"="rs3:5 11 rs2:5 rs1:5 000 rd:5 1000111" "asm"="fmsub.q {rd}, {rs1}, {rs2}, {rs3}, rne" "asmonly" {
  ret void
}

define void @FMSUB_Q.2(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2, i64 "reg"="d" %rs3) "enc"="rs3:5 11 rs2:5 rs1:5 001 rd:5 1000111" "asm"="fmsub.q {rd}, {rs1}, {rs2}, {rs3}, rtz" "asmonly" {
  ret void
}

define void @FMSUB_Q.3(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2, i64 "reg"="d" %rs3) "enc"="rs3:5 11 rs2:5 rs1:5 010 rd:5 1000111" "asm"="fmsub.q {rd}, {rs1}, {rs2}, {rs3}, rdn" "asmonly" {
  ret void
}

define void @FMSUB_Q.4(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2, i64 "reg"="d" %rs3) "enc"="rs3:5 11 rs2:5 rs1:5 011 rd:5 1000111" "asm"="fmsub.q {rd}, {rs1}, {rs2}, {rs3}, rup" "asmonly" {
  ret void
}

define void @FMSUB_Q.5(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2, i64 "reg"="d" %rs3) "enc"="rs3:5 11 rs2:5 rs1:5 100 rd:5 1000111" "asm"="fmsub.q {rd}, {rs1}, {rs2}, {rs3}, rmm" "asmonly" {
  ret void
}

define void @FMSUB_Q.6(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2, i64 "reg"="d" %rs3) "enc"="rs3:5 11 rs2:5 rs1:5 111 rd:5 1000111" "asm"="fmsub.q {rd}, {rs1}, {rs2}, {rs3}, dyn" "asmonly" "nodecode" {
  ret void
}

define void @FNMSUB_Q.0(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2, i64 "reg"="d" %rs3) "enc"="rs3:5 11 rs2:5 rs1:5 111 rd:5 1001011" "asm"="fnmsub.q {rd}, {rs1}, {rs2}, {rs3}" "asmonly" {
  ret void
}

define void @FNMSUB_Q.1(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2, i64 "reg"="d" %rs3) "enc"="rs3:5 11 rs2:5 rs1:5 000 rd:5 1001011" "asm"="fnmsub.q {rd}, {rs1}, {rs2}, {rs3}, rne" "asmonly" {
  ret void
}

define void @FNMSUB_Q.2(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2, i64 "reg"="d" %rs3) "enc"="rs3:5 11 rs2:5 rs1:5 001 rd:5 1001011" "asm"="fnmsub.q {rd}, {rs1}, {rs2}, {rs3}, rtz" "asmonly" {
  ret void
}

define void @FNMSUB_Q.3(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2, i64 "reg"="d" %rs3) "enc"="rs3:5 11 rs2:5 rs1:5 010 rd:5 1001011" "asm"="fnmsub.q {rd}, {rs1}, {rs2}, {rs3}, rdn" "asmonly" {
  ret void
}

define void @FNMSUB_Q.4(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2, i64 "reg"="d" %rs3) "enc"="rs3:5 11 rs2:5 rs1:5 011 rd:5 1001011" "asm"="fnmsub.q {rd}, {rs1}, {rs2}, {rs3}, rup" "asmonly" {
  ret void
}

define void @FNMSUB_Q.5(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2, i64 "reg"="d" %rs3) "enc"="rs3:5 11 rs2:5 rs1:5 100 rd:5 1001011" "asm"="fnmsub.q {rd}, {rs1}, {rs2}, {rs3}, rmm" "asmonly" {
  ret void
}

define void @FNMSUB_Q.6(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2, i64 "reg"="d" %rs3) "enc"="rs3:5 11 rs2:5 rs1:5 111 rd:5 1001011" "asm"="fnmsub.q {rd}, {rs1}, {rs2}, {rs3}, dyn" "asmonly" "nodecode" {
  ret void
}

define void @FNMADD_Q.0(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2, i64 "reg"="d" %rs3) "enc"="rs3:5 11 rs2:5 rs1:5 111 rd:5 1001111" "asm"="fnmadd.q {rd}, {rs1}, {rs2}, {rs3}" "asmonly" {
  ret void
}

define void @FNMADD_Q.1(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2, i64 "reg"="d" %rs3) "enc"="rs3:5 11 rs2:5 rs1:5 000 rd:5 1001111" "asm"="fnmadd.q {rd}, {rs1}, {rs2}, {rs3}, rne" "asmonly" {
  ret void
}

define void @FNMADD_Q.2(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2, i64 "reg"="d" %rs3) "enc"="rs3:5 11 rs2:5 rs1:5 001 rd:5 1001111" "asm"="fnmadd.q {rd}, {rs1}, {rs2}, {rs3}, rtz" "asmonly" {
  ret void
}

define void @FNMADD_Q.3(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2, i64 "reg"="d" %rs3) "enc"="rs3:5 11 rs2:5 rs1:5 010 rd:5 1001111" "asm"="fnmadd.q {rd}, {rs1}, {rs2}, {rs3}, rdn" "asmonly" {
  ret void
}

define void @FNMADD_Q.4(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2, i64 "reg"="d" %rs3) "enc"="rs3:5 11 rs2:5 rs1:5 011 rd:5 1001111" "asm"="fnmadd.q {rd}, {rs1}, {rs2}, {rs3}, rup" "asmonly" {
  ret void
}

define void @FNMADD_Q.5(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2, i64 "reg"="d" %rs3) "enc"="rs3:5 11 rs2:5 rs1:5 100 rd:5 1001111" "asm"="fnmadd.q {rd}, {rs1}, {rs2}, {rs3}, rmm" "asmonly" {
  ret void
}

define void @FNMADD_Q.6(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2, i64 "reg"="d" %rs3) "enc"="rs3:5 11 rs2:5 rs1:5 111 rd:5 1001111" "asm"="fnmadd.q {rd}, {rs1}, {rs2}, {rs3}, dyn" "asmonly" "nodecode" {
  ret void
}

define void @FADD_Q.0(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2) "enc"="0000011 rs2:5 rs1:5 111 rd:5 1010011" "asm"="fadd.q {rd}, {rs1}, {rs2}" "asmonly" {
  ret void
}

define void @FADD_Q.1(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2) "enc"="0000011 rs2:5 rs1:5 000 rd:5 1010011" "asm"="fadd.q {rd}, {rs1}, {rs2}, rne" "asmonly" {
  ret void
}

define void @FADD_Q.2(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2) "enc"="0000011 rs2:5 rs1:5 001 rd:5 1010011" "asm"="fadd.q {rd}, {rs1}, {rs2}, rtz" "asmonly" {
  ret void
}

define void @FADD_Q.3(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2) "enc"="0000011 rs2:5 rs1:5 010 rd:5 1010011" "asm"="fadd.q {rd}, {rs1}, {rs2}, rdn" "asmonly" {
  ret void
}

define void @FADD_Q.4(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2) "enc"="0000011 rs2:5 rs1:5 011 rd:5 1010011" "asm"="fadd.q {rd}, {rs1}, {rs2}, rup" "asmonly" {
  ret void
}

define void @FADD_Q.5(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2) "enc"="0000011 rs2:5 rs1:5 100 rd:5 1010011" "asm"="fadd.q {rd}, {rs1}, {rs2}, rmm" "asmonly" {
  ret void
}

define void @FADD_Q.6(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2) "enc"="0000011 rs2:5 rs1:5 111 rd:5 1010011" "asm"="fadd.q {rd}, {rs1}, {rs2}, dyn" "asmonly" "nodecode" {
  ret void
}

define void @FSUB_Q.0(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2) "enc"="0000111 rs2:5 rs1:5 111 rd:5 1010011" "asm"="fsub.q {rd}, {rs1}, {rs2}" "asmonly" {
  ret void
}

define void @FSUB_Q.1(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2) "enc"="0000111 rs2:5 rs1:5 000 rd:5 1010011" "asm"="fsub.q {rd}, {rs1}, {rs2}, rne" "asmonly" {
  ret void
}

define void @FSUB_Q.2(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2) "enc"="0000111 rs2:5 rs1:5 001 rd:5 1010011" "asm"="fsub.q {rd}, {rs1}, {rs2}, rtz" "asmonly" {
  ret void
}

define void @FSUB_Q.3(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2) "enc"="0000111 rs2:5 rs1:5 010 rd:5 1010011" "asm"="fsub.q {rd}, {rs1}, {rs2}, rdn" "asmonly" {
  ret void
}

define void @FSUB_Q.4(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2) "enc"="0000111 rs2:5 rs1:5 011 rd:5 1010011" "asm"="fsub.q {rd}, {rs1}, {rs2}, rup" "asmonly" {
  ret void
}

define void @FSUB_Q.5(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2) "enc"="0000111 rs2:5 rs1:5 100 rd:5 1010011" "asm"="fsub.q {rd}, {rs1}, {rs2}, rmm" "asmonly" {
  ret void
}

define void @FSUB_Q.6(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2) "enc"="0000111 rs2:5 rs1:5 111 rd:5 1010011" "asm"="fsub.q {rd}, {rs1}, {rs2}, dyn" "asmonly" "nodecode" {
  ret void
}

define void @FMUL_Q.0(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2) "enc"="0001011 rs2:5 rs1:5 111 rd:5 1010011" "asm"="fmul.q {rd}, {rs1}, {rs2}" "asmonly" {
  ret void
}

define void @FMUL_Q.1(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2) "enc"="0001011 rs2:5 rs1:5 000 rd:5 1010011" "asm"="fmul.q {rd}, {rs1}, {rs2}, rne" "asmonly" {
  ret void
}

define void @FMUL_Q.2(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2) "enc"="0001011 rs2:5 rs1:5 001 rd:5 1010011" "asm"="fmul.q {rd}, {rs1}, {rs2}, rtz" "asmonly" {
  ret void
}

define void @FMUL_Q.3(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2) "enc"="0001011 rs2:5 rs1:5 010 rd:5 1010011" "asm"="fmul.q {rd}, {rs1}, {rs2}, rdn" "asmonly" {
  ret void
}

define void @FMUL_Q.4(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2) "enc"="0001011 rs2:5 rs1:5 011 rd:5 1010011" "asm"="fmul.q {rd}, {rs1}, {rs2}, rup" "asmonly" {
  ret void
}

define void @FMUL_Q.5(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2) "enc"="0001011 rs2:5 rs1:5 100 rd:5 1010011" "asm"="fmul.q {rd}, {rs1}, {rs2}, rmm" "asmonly" {
  ret void
}

define void @FMUL_Q.6(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2) "enc"="0001011 rs2:5 rs1:5 111 rd:5 1010011" "asm"="fmul.q {rd}, {rs1}, {rs2}, dyn" "asmonly" "nodecode" {
  ret void
}

define void @FDIV_Q.0(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2) "enc"="0001111 rs2:5 rs1:5 111 rd:5 1010011" "asm"="fdiv.q {rd}, {rs1}, {rs2}" "asmonly" {
  ret void
}

define void @FDIV_Q.1(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2) "enc"="0001111 rs2:5 rs1:5 000 rd:5 1010011" "asm"="fdiv.q {rd}, {rs1}, {rs2}, rne" "asmonly" {
  ret void
}

define void @FDIV_Q.2(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2) "enc"="0001111 rs2:5 rs1:5 001 rd:5 1010011" "asm"="fdiv.q {rd}, {rs1}, {rs2}, rtz" "asmonly" {
  ret void
}

define void @FDIV_Q.3(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2) "enc"="0001111 rs2:5 rs1:5 010 rd:5 1010011" "asm"="fdiv.q {rd}, {rs1}, {rs2}, rdn" "asmonly" {
  ret void
}

define void @FDIV_Q.4(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2) "enc"="0001111 rs2:5 rs1:5 011 rd:5 1010011" "asm"="fdiv.q {rd}, {rs1}, {rs2}, rup" "asmonly" {
  ret void
}

define void @FDIV_Q.5(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2) "enc"="0001111 rs2:5 rs1:5 100 rd:5 1010011" "asm"="fdiv.q {rd}, {rs1}, {rs2}, rmm" "asmonly" {
  ret void
}

define void @FDIV_Q.6(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2) "enc"="0001111 rs2:5 rs1:5 111 rd:5 1010011" "asm"="fdiv.q {rd}, {rs1}, {rs2}, dyn" "asmonly" "nodecode" {
  ret void
}

define void @FSQRT_Q.0(i64 "reg"="d" %rd, i64 "reg"="d" %rs1) "enc"="010111100000 rs1:5 111 rd:5 1010011" "asm"="fsqrt.q {rd}, {rs1}" "asmonly" {
  ret void
}

define void @FSQRT_Q.1(i64 "reg"="d" %rd, i64 "reg"="d" %rs1) "enc"="010111100000 rs1:5 000 rd:5 1010011" "asm"="fsqrt.q {rd}, {rs1}, rne" "asmonly" {
  ret void
}

define void @FSQRT_Q.2(i64 "reg"="d" %rd, i64 "reg"="d" %rs1) "enc"="010111100000 rs1:5 001 rd:5 1010011" "asm"="fsqrt.q {rd}, {rs1}, rtz" "asmonly" {
  ret void
}

define void @FSQRT_Q.3(i64 "reg"="d" %rd, i64 "reg"="d" %rs1) "enc"="010111100000 rs1:5 010 rd:5 1010011" "asm"="fsqrt.q {rd}, {rs1}, rdn" "asmonly" {
  ret void
}

define void @FSQRT_Q.4(i64 "reg"="d" %rd, i64 "reg"="d" %rs1) "enc"="010111100000 rs1:5 011 rd:5 1010011" "asm"="fsqrt.q {rd}, {rs1}, rup" "asmonly" {
  ret void
}

define void @FSQRT_Q.5(i64 "reg"="d" %rd, i64 "reg"="d" %rs1) "enc"="010111100000 rs1:5 100 rd:5 1010011" "asm"="fsqrt.q {rd}, {rs1}, rmm" "asmonly" {
  ret void
}

define void @FSQRT_Q.6(i64 "reg"="d" %rd, i64 "reg"="d" %rs1) "enc"="010111100000 rs1:5 111 rd:5 1010011" "asm"="fsqrt.q {rd}, {rs1}, dyn" "asmonly" "nodecode" {
  ret void
}

define void @FSGNJ_Q.0(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2) "enc"="0010011 rs2:5 rs1:5 000 rd:5 1010011" "asm"="fsgnj.q {rd}, {rs1}, {rs2}" "asmonly" {
  ret void
}

define void @FSGNJN_Q.0(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2) "enc"="0010011 rs2:5 rs1:5 001 rd:5 1010011" "asm"="fsgnjn.q {rd}, {rs1}, {rs2}" "asmonly" {
  ret void
}

define void @FSGNJX_Q.0(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2) "enc"="0010011 rs2:5 rs1:5 010 rd:5 1010011" "asm"="fsgnjx.q {rd}, {rs1}, {rs2}" "asmonly" {
  ret void
}

define void @FMIN_Q.0(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2) "enc"="0010111 rs2:5 rs1:5 000 rd:5 1010011" "asm"="fmin.q {rd}, {rs1}, {rs2}" "asmonly" {
  ret void
}

define void @FMAX_Q.0(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2) "enc"="0010111 rs2:5 rs1:5 001 rd:5 1010011" "asm"="fmax.q {rd}, {rs1}, {rs2}" "asmonly" {
  ret void
}

define void @FCVT_S_Q.0(i64 "reg"="d" %rd, i64 "reg"="d" %rs1) "enc"="010000000011 rs1:5 111 rd:5 1010011" "asm"="fcvt.s.q {rd}, {rs1}" "asmonly" {
  ret void
}

define void @FCVT_S_Q.1(i64 "reg"="d" %rd, i64 "reg"="d" %rs1) "enc"="010000000011 rs1:5 000 rd:5 1010011" "asm"="fcvt.s.q {rd}, {rs1}, rne" "asmonly" {
  ret void
}

define void @FCVT_S_Q.2(i64 "reg"="d" %rd, i64 "reg"="d" %rs1) "enc"="010000000011 rs1:5 001 rd:5 1010011" "asm"="fcvt.s.q {rd}, {rs1}, rtz" "asmonly" {
  ret void
}

define void @FCVT_S_Q.3(i64 "reg"="d" %rd, i64 "reg"="d" %rs1) "enc"="010000000011 rs1:5 010 rd:5 1010011" "asm"="fcvt.s.q {rd}, {rs1}, rdn" "asmonly" {
  ret void
}

define void @FCVT_S_Q.4(i64 "reg"="d" %rd, i64 "reg"="d" %rs1) "enc"="010000000011 rs1:5 011 rd:5 1010011" "asm"="fcvt.s.q {rd}, {rs1}, rup" "asmonly" {
  ret void
}

define void @FCVT_S_Q.5(i64 "reg"="d" %rd, i64 "reg"="d" %rs1) "enc"="010000000011 rs1:5 100 rd:5 1010011" "asm"="fcvt.s.q {rd}, {rs1}, rmm" "asmonly" {
  ret void
}

define void @FCVT_S_Q.6(i64 "reg"="d" %rd, i64 "reg"="d" %rs1) "enc"="010000000011 rs1:5 111 rd:5 1010011" "asm"="fcvt.s.q {rd}, {rs1}, dyn" "asmonly" "nodecode" {
  ret void
}

define void @FCVT_Q_S.0(i64 "reg"="d" %rd, i64 "reg"="d" %rs1) "enc"="010001100000 rs1:5 000 rd:5 1010011" "asm"="fcvt.q.s {rd}, {rs1}" "asmonly" {
  ret void
}

define void @FCVT_Q_S.1(i64 "reg"="d" %rd, i64 "reg"="d" %rs1) "enc"="010001100000 rs1:5 000 rd:5 1010011" "asm"="fcvt.q.s {rd}, {rs1}, rne" "asmonly" "nodecode" {
  ret void
}

define void @FCVT_Q_S.2(i64 "reg"="d" %rd, i64 "reg"="d" %rs1) "enc"="010001100000 rs1:5 001 rd:5 1010011" "asm"="fcvt.q.s {rd}, {rs1}, rtz" "asmonly" {
  ret void
}

define void @FCVT_Q_S.3(i64 "reg"="d" %rd, i64 "reg"="d" %rs1) "enc"="010001100000 rs1:5 010 rd:5 1010011" "asm"="fcvt.q.s {rd}, {rs1}, rdn" "asmonly" {
  ret void
}

define void @FCVT_Q_S.4(i64 "reg"="d" %rd, i64 "reg"="d" %rs1) "enc"="010001100000 rs1:5 011 rd:5 1010011" "asm"="fcvt.q.s {rd}, {rs1}, rup" "asmonly" {
  ret void
}

define void @FCVT_Q_S.5(i64 "reg"="d" %rd, i64 "reg"="d" %rs1) "enc"="010001100000 rs1:5 100 rd:5 1010011" "asm"="fcvt.q.s {rd}, {rs1}, rmm" "asmonly" {
  ret void
}

define void @FCVT_Q_S.6(i64 "reg"="d" %rd, i64 "reg"="d" %rs1) "enc"="010001100000 rs1:5 111 rd:5 1010011" "asm"="fcvt.q.s {rd}, {rs1}, dyn" "asmonly" {
  ret void
}

define void @FCVT_D_Q.0(i64 "reg"="d" %rd, i64 "reg"="d" %rs1) "enc"="010000100011 rs1:5 111 rd:5 1010011" "asm"="fcvt.d.q {rd}, {rs1}" "asmonly" {
  ret void
}

define void @FCVT_D_Q.1(i64 "reg"="d" %rd, i64 "reg"="d" %rs1) "enc"="010000100011 rs1:5 000 rd:5 1010011" "asm"="fcvt.d.q {rd}, {rs1}, rne" "asmonly" {
  ret void
}

define void @FCVT_D_Q.2(i64 "reg"="d" %rd, i64 "reg"="d" %rs1) "enc"="010000100011 rs1:5 001 rd:5 1010011" "asm"="fcvt.d.q {rd}, {rs1}, rtz" "asmonly" {
  ret void
}

define void @FCVT_D_Q.3(i64 "reg"="d" %rd, i64 "reg"="d" %rs1) "enc"="010000100011 rs1:5 010 rd:5 1010011" "asm"="fcvt.d.q {rd}, {rs1}, rdn" "asmonly" {
  ret void
}

define void @FCVT_D_Q.4(i64 "reg"="d" %rd, i64 "reg"="d" %rs1) "enc"="010000100011 rs1:5 011 rd:5 1010011" "asm"="fcvt.d.q {rd}, {rs1}, rup" "asmonly" {
  ret void
}

define void @FCVT_D_Q.5(i64 "reg"="d" %rd, i64 "reg"="d" %rs1) "enc"="010000100011 rs1:5 100 rd:5 1010011" "asm"="fcvt.d.q {rd}, {rs1}, rmm" "asmonly" {
  ret void
}

define void @FCVT_D_Q.6(i64 "reg"="d" %rd, i64 "reg"="d" %rs1) "enc"="010000100011 rs1:5 111 rd:5 1010011" "asm"="fcvt.d.q {rd}, {rs1}, dyn" "asmonly" "nodecode" {
  ret void
}

define void @FCVT_Q_D.0(i64 "reg"="d" %rd, i64 "reg"="d" %rs1) "enc"="010001100001 rs1:5 000 rd:5 1010011" "asm"="fcvt.q.d {rd}, {rs1}" "asmonly" {
  ret void
}

define void @FCVT_Q_D.1(i64 "reg"="d" %rd, i64 "reg"="d" %rs1) "enc"="010001100001 rs1:5 000 rd:5 1010011" "asm"="fcvt.q.d {rd}, {rs1}, rne" "asmonly" "nodecode" {
  ret void
}

define void @FCVT_Q_D.2(i64 "reg"="d" %rd, i64 "reg"="d" %rs1) "enc"="010001100001 rs1:5 001 rd:5 1010011" "asm"="fcvt.q.d {rd}, {rs1}, rtz" "asmonly" {
  ret void
}

define void @FCVT_Q_D.3(i64 "reg"="d" %rd, i64 "reg"="d" %rs1) "enc"="010001100001 rs1:5 010 rd:5 1010011" "asm"="fcvt.q.d {rd}, {rs1}, rdn" "asmonly" {
  ret void
}

define void @FCVT_Q_D.4(i64 "reg"="d" %rd, i64 "reg"="d" %rs1) "enc"="010001100001 rs1:5 011 rd:5 1010011" "asm"="fcvt.q.d {rd}, {rs1}, rup" "asmonly" {
  ret void
}

define void @FCVT_Q_D.5(i64 "reg"="d" %rd, i64 "reg"="d" %rs1) "enc"="010001100001 rs1:5 100 rd:5 1010011" "asm"="fcvt.q.d {rd}, {rs1}, rmm" "asmonly" {
  ret void
}

define void @FCVT_Q_D.6(i64 "reg"="d" %rd, i64 "reg"="d" %rs1) "enc"="010001100001 rs1:5 111 rd:5 1010011" "asm"="fcvt.q.d {rd}, {rs1}, dyn" "asmonly" {
  ret void
}

define void @FEQ_Q.0(i64 "reg"="x" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2) "enc"="1010011 rs2:5 rs1:5 010 rd:5 1010011" "asm"="feq.q {rd}, {rs1}, {rs2}" "asmonly" {
  ret void
}

define void @FLT_Q.0(i64 "reg"="x" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2) "enc"="1010011 rs2:5 rs1:5 001 rd:5 1010011" "asm"="flt.q {rd}, {rs1}, {rs2}" "asmonly" {
  ret void
}

define void @FLE_Q.0(i64 "reg"="x" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2) "enc"="1010011 rs2:5 rs1:5 000 rd:5 1010011" "asm"="fle.q {rd}, {rs1}, {rs2}" "asmonly" {
  ret void
}

define void @FCLASS_Q.0(i64 "reg"="x" %rd, i64 "reg"="d" %rs1) "enc"="111001100000 rs1:5 001 rd:5 1010011" "asm"="fclass.q {rd}, {rs1}" "asmonly" {
  ret void
}

define void @FCVT_W_Q.0(i64 "reg"="x" %rd, i64 "reg"="d" %rs1) "enc"="110001100000 rs1:5 111 rd:5 1010011" "asm"="fcvt.w.q {rd}, {rs1}" "asmonly" {
  ret void
}

define void @FCVT_W_Q.1(i64 "reg"="x" %rd, i64 "reg"="d" %rs1) "enc"="110001100000 rs1:5 000 rd:5 1010011" "asm"="fcvt.w.q {rd}, {rs1}, rne" "asmonly" {
  ret void
}

define void @FCVT_W_Q.2(i64 "reg"="x" %rd, i64 "reg"="d" %rs1) "enc"="110001100000 rs1:5 001 rd:5 1010011" "asm"="fcvt.w.q {rd}, {rs1}, rtz" "asmonly" {
  ret void
}

define void @FCVT_W_Q.3(i64 "reg"="x" %rd, i64 "reg"="d" %rs1) "enc"="110001100000 rs1:5 010 rd:5 1010011" "asm"="fcvt.w.q {rd}, {rs1}, rdn" "asmonly" {
  ret void
}

define void @FCVT_W_Q.4(i64 "reg"="x" %rd, i64 "reg"="d" %rs1) "enc"="110001100000 rs1:5 011 rd:5 1010011" "asm"="fcvt.w.q {rd}, {rs1}, rup" "asmonly" {
  ret void
}

define void @FCVT_W_Q.5(i64 "reg"="x" %rd, i64 "reg"="d" %rs1) "enc"="110001100000 rs1:5 100 rd:5 1010011" "asm"="fcvt.w.q {rd}, {rs1}, rmm" "asmonly" {
  ret void
}

define void @FCVT_W_Q.6(i64 "reg"="x" %rd, i64 "reg"="d" %rs1) "enc"="110001100000 rs1:5 111 rd:5 1010011" "asm"="fcvt.w.q {rd}, {rs1}, dyn" "asmonly" "nodecode" {
  ret void
}

define void @FCVT_WU_Q.0(i64 "reg"="x" %rd, i64 "reg"="d" %rs1) "enc"="110001100001 rs1:5 111 rd:5 1010011" "asm"="fcvt.wu.q {rd}, {rs1}" "asmonly" {
  ret void
}

define void @FCVT_WU_Q.1(i64 "reg"="x" %rd, i64 "reg"="d" %rs1) "enc"="110001100001 rs1:5 000 rd:5 1010011" "asm"="fcvt.wu.q {rd}, {rs1}, rne" "asmonly" {
  ret void
}

define void @FCVT_WU_Q.2(i64 "reg"="x" %rd, i64 "reg"="d" %rs1) "enc"="110001100001 rs1:5 001 rd:5 1010011" "asm"="fcvt.wu.q {rd}, {rs1}, rtz" "asmonly" {
  ret void
}

define void @FCVT_WU_Q.3(i64 "reg"="x" %rd, i64 "reg"="d" %rs1) "enc"="110001100001 rs1:5 010 rd:5 1010011" "asm"="fcvt.wu.q {rd}, {rs1}, rdn" "asmonly" {
  ret void
}

define void @FCVT_WU_Q.4(i64 "reg"="x" %rd, i64 "reg"="d" %rs1) "enc"="110001100001 rs1:5 011 rd:5 1010011" "asm"="fcvt.wu.q {rd}, {rs1}, rup" "asmonly" {
  ret void
}

define void @FCVT_WU_Q.5(i64 "reg"="x" %rd, i64 "reg"="d" %rs1) "enc"="110001100001 rs1:5 100 rd:5 1010011" "asm"="fcvt.wu.q {rd}, {rs1}, rmm" "asmonly" {
  ret void
}

define void @FCVT_WU_Q.6(i64 "reg"="x" %rd, i64 "reg"="d" %rs1) "enc"="110001100001 rs1:5 111 rd:5 1010011" "asm"="fcvt.wu.q {rd}, {rs1}, dyn" "asmonly" "nodecode" {
  ret void
}

define void @FCVT_Q_W.0(i64 "reg"="d" %rd, i64 "reg"="x" %rs1) "enc"="110101100000 rs1:5 000 rd:5 1010011" "asm"="fcvt.q.w {rd}, {rs1}" "asmonly" {
  ret void
}

define void @FCVT_Q_W.1(i64 "reg"="d" %rd, i64 "reg"="x" %rs1) "enc"="110101100000 rs1:5 000 rd:5 1010011" "asm"="fcvt.q.w {rd}, {rs1}, rne" "asmonly" "nodecode" {
  ret void
}

define void @FCVT_Q_W.2(i64 "reg"="d" %rd, i64 "reg"="x" %rs1) "enc"="110101100000 rs1:5 001 rd:5 1010011" "asm"="fcvt.q.w {rd}, {rs1}, rtz" "asmonly" {
  ret void
}

define void @FCVT_Q_W.3(i64 "reg"="d" %rd, i64 "reg"="x" %rs1) "enc"="110101100000 rs1:5 010 rd:5 1010011" "asm"="fcvt.q.w {rd}, {rs1}, rdn" "asmonly" {
  ret void
}

define void @FCVT_Q_W.4(i64 "reg"="d" %rd, i64 "reg"="x" %rs1) "enc"="110101100000 rs1:5 011 rd:5 1010011" "asm"="fcvt.q.w {rd}, {rs1}, rup" "asmonly" {
  ret void
}

define void @FCVT_Q_W.5(i64 "reg"="d" %rd, i64 "reg"="x" %rs1) "enc"="110101100000 rs1:5 100 rd:5 1010011" "asm"="fcvt.q.w {rd}, {rs1}, rmm" "asmonly" {
  ret void
}

define void @FCVT_Q_W.6(i64 "reg"="d" %rd, i64 "reg"="x" %rs1) "enc"="110101100000 rs1:5 111 rd:5 1010011" "asm"="fcvt.q.w {rd}, {rs1}, dyn" "asmonly" {
  ret void
}

define void @FCVT_Q_WU.0(i64 "reg"="d" %rd, i64 "reg"="x" %rs1) "enc"="110101100001 rs1:5 000 rd:5 1010011" "asm"="fcvt.q.wu {rd}, {rs1}" "asmonly" {
  ret void
}

define void @FCVT_Q_WU.1(i64 "reg"="d" %rd, i64 "reg"="x" %rs1) "enc"="110101100001 rs1:5 000 rd:5 1010011" "asm"="fcvt.q.wu {rd}, {rs1}, rne" "asmonly" "nodecode" {
  ret void
}

define void @FCVT_Q_WU.2(i64 "reg"="d" %rd, i64 "reg"="x" %rs1) "enc"="110101100001 rs1:5 001 rd:5 1010011" "asm"="fcvt.q.wu {rd}, {rs1}, rtz" "asmonly" {
  ret void
}

define void @FCVT_Q_WU.3(i64 "reg"="d" %rd, i64 "reg"="x" %rs1) "enc"="110101100001 rs1:5 010 rd:5 1010011" "asm"="fcvt.q.wu {rd}, {rs1}, rdn" "asmonly" {
  ret void
}

define void @FCVT_Q_WU.4(i64 "reg"="d" %rd, i64 "reg"="x" %rs1) "enc"="110101100001 rs1:5 011 rd:5 1010011" "asm"="fcvt.q.wu {rd}, {rs1}, rup" "asmonly" {
  ret void
}

define void @FCVT_Q_WU.5(i64 "reg"="d" %rd, i64 "reg"="x" %rs1) "enc"="110101100001 rs1:5 100 rd:5 1010011" "asm"="fcvt.q.wu {rd}, {rs1}, rmm" "asmonly" {
  ret void
}

define void @FCVT_Q_WU.6(i64 "reg"="d" %rd, i64 "reg"="x" %rs1) "enc"="110101100001 rs1:5 111 rd:5 1010011" "asm"="fcvt.q.wu {rd}, {rs1}, dyn" "asmonly" {
  ret void
}

define void @FCVT_L_Q.0(i64 "reg"="x" %rd, i64 "reg"="d" %rs1) "enc"="110001100010 rs1:5 111 rd:5 1010011" "asm"="fcvt.l.q {rd}, {rs1}" "asmonly" {
  ret void
}

define void @FCVT_L_Q.1(i64 "reg"="x" %rd, i64 "reg"="d" %rs1) "enc"="110001100010 rs1:5 000 rd:5 1010011" "asm"="fcvt.l.q {rd}, {rs1}, rne" "asmonly" {
  ret void
}

define void @FCVT_L_Q.2(i64 "reg"="x" %rd, i64 "reg"="d" %rs1) "enc"="110001100010 rs1:5 001 rd:5 1010011" "asm"="fcvt.l.q {rd}, {rs1}, rtz" "asmonly" {
  ret void
}

define void @FCVT_L_Q.3(i64 "reg"="x" %rd, i64 "reg"="d" %rs1) "enc"="110001100010 rs1:5 010 rd:5 1010011" "asm"="fcvt.l.q {rd}, {rs1}, rdn" "asmonly" {
  ret void
}

define void @FCVT_L_Q.4(i64 "reg"="x" %rd, i64 "reg"="d" %rs1) "enc"="110001100010 rs1:5 011 rd:5 1010011" "asm"="fcvt.l.q {rd}, {rs1}, rup" "asmonly" {
  ret void
}

define void @FCVT_L_Q.5(i64 "reg"="x" %rd, i64 "reg"="d" %rs1) "enc"="110001100010 rs1:5 100 rd:5 1010011" "asm"="fcvt.l.q {rd}, {rs1}, rmm" "asmonly" {
  ret void
}

define void @FCVT_L_Q.6(i64 "reg"="x" %rd, i64 "reg"="d" %rs1) "enc"="110001100010 rs1:5 111 rd:5 1010011" "asm"="fcvt.l.q {rd}, {rs1}, dyn" "asmonly" "nodecode" {
  ret void
}

define void @FCVT_LU_Q.0(i64 "reg"="x" %rd, i64 "reg"="d" %rs1) "enc"="110001100011 rs1:5 111 rd:5 1010011" "asm"="fcvt.lu.q {rd}, {rs1}" "asmonly" {
  ret void
}

define void @FCVT_LU_Q.1(i64 "reg"="x" %rd, i64 "reg"="d" %rs1) "enc"="110001100011 rs1:5 000 rd:5 1010011" "asm"="fcvt.lu.q {rd}, {rs1}, rne" "asmonly" {
  ret void
}

define void @FCVT_LU_Q.2(i64 "reg"="x" %rd, i64 "reg"="d" %rs1) "enc"="110001100011 rs1:5 001 rd:5 1010011" "asm"="fcvt.lu.q {rd}, {rs1}, rtz" "asmonly" {
  ret void
}

define void @FCVT_LU_Q.3(i64 "reg"="x" %rd, i64 "reg"="d" %rs1) "enc"="110001100011 rs1:5 010 rd:5 1010011" "asm"="fcvt.lu.q {rd}, {rs1}, rdn" "asmonly" {
  ret void
}

define void @FCVT_LU_Q.4(i64 "reg"="x" %rd, i64 "reg"="d" %rs1) "enc"="110001100011 rs1:5 011 rd:5 1010011" "asm"="fcvt.lu.q {rd}, {rs1}, rup" "asmonly" {
  ret void
}

define void @FCVT_LU_Q.5(i64 "reg"="x" %rd, i64 "reg"="d" %rs1) "enc"="110001100011 rs1:5 100 rd:5 1010011" "asm"="fcvt.lu.q {rd}, {rs1}, rmm" "asmonly" {
  ret void
}

define void @FCVT_LU_Q.6(i64 "reg"="x" %rd, i64 "reg"="d" %rs1) "enc"="110001100011 rs1:5 111 rd:5 1010011" "asm"="fcvt.lu.q {rd}, {rs1}, dyn" "asmonly" "nodecode" {
  ret void
}

define void @FCVT_Q_L.0(i64 "reg"="d" %rd, i64 "reg"="x" %rs1) "enc"="110101100010 rs1:5 000 rd:5 1010011" "asm"="fcvt.q.l {rd}, {rs1}" "asmonly" {
  ret void
}

define void @FCVT_Q_L.1(i64 "reg"="d" %rd, i64 "reg"="x" %rs1) "enc"="110101100010 rs1:5 000 rd:5 1010011" "asm"="fcvt.q.l {rd}, {rs1}, rne" "asmonly" "nodecode" {
  ret void
}

define void @FCVT_Q_L.2(i64 "reg"="d" %rd, i64 "reg"="x" %rs1) "enc"="110101100010 rs1:5 001 rd:5 1010011" "asm"="fcvt.q.l {rd}, {rs1}, rtz" "asmonly" {
  ret void
}

define void @FCVT_Q_L.3(i64 "reg"="d" %rd, i64 "reg"="x" %rs1) "enc"="110101100010 rs1:5 010 rd:5 1010011" "asm"="fcvt.q.l {rd}, {rs1}, rdn" "asmonly" {
  ret void
}

define void @FCVT_Q_L.4(i64 "reg"="d" %rd, i64 "reg"="x" %rs1) "enc"="110101100010 rs1:5 011 rd:5 1010011" "asm"="fcvt.q.l {rd}, {rs1}, rup" "asmonly" {
  ret void
}

define void @FCVT_Q_L.5(i64 "reg"="d" %rd, i64 "reg"="x" %rs1) "enc"="110101100010 rs1:5 100 rd:5 1010011" "asm"="fcvt.q.l {rd}, {rs1}, rmm" "asmonly" {
  ret void
}

define void @FCVT_Q_L.6(i64 "reg"="d" %rd, i64 "reg"="x" %rs1) "enc"="110101100010 rs1:5 111 rd:5 1010011" "asm"="fcvt.q.l {rd}, {rs1}, dyn" "asmonly" {
  ret void
}

define void @FCVT_Q_LU.0(i64 "reg"="d" %rd, i64 "reg"="x" %rs1) "enc"="110101100011 rs1:5 000 rd:5 1010011" "asm"="fcvt.q.lu {rd}, {rs1}" "asmonly" {
  ret void
}

define void @FCVT_Q_LU.1(i64 "reg"="d" %rd, i64 "reg"="x" %rs1) "enc"="110101100011 rs1:5 000 rd:5 1010011" "asm"="fcvt.q.lu {rd}, {rs1}, rne" "asmonly" "nodecode" {
  ret void
}

define void @FCVT_Q_LU.2(i64 "reg"="d" %rd, i64 "reg"="x" %rs1) "enc"="110101100011 rs1:5 001 rd:5 1010011" "asm"="fcvt.q.lu {rd}, {rs1}, rtz" "asmonly" {
  ret void
}

define void @FCVT_Q_LU.3(i64 "reg"="d" %rd, i64 "reg"="x" %rs1) "enc"="110101100011 rs1:5 010 rd:5 1010011" "asm"="fcvt.q.lu {rd}, {rs1}, rdn" "asmonly" {
  ret void
}

define void @FCVT_Q_LU.4(i64 "reg"="d" %rd, i64 "reg"="x" %rs1) "enc"="110101100011 rs1:5 011 rd:5 1010011" "asm"="fcvt.q.lu {rd}, {rs1}, rup" "asmonly" {
  ret void
}

define void @FCVT_Q_LU.5(i64 "reg"="d" %rd, i64 "reg"="x" %rs1) "enc"="110101100011 rs1:5 100 rd:5 1010011" "asm"="fcvt.q.lu {rd}, {rs1}, rmm" "asmonly" {
  ret void
}

define void @FCVT_Q_LU.6(i64 "reg"="d" %rd, i64 "reg"="x" %rs1) "enc"="110101100011 rs1:5 111 rd:5 1010011" "asm"="fcvt.q.lu {rd}, {rs1}, dyn" "asmonly" {
  ret void
}

define void @FLI_Q.0(i64 "reg"="d" %rd) "enc"="11110110000100000000 rd:5 1010011" "asm"="fli.q {rd}, -1.0" "asmonly" {
  ret void
}

define void @FLI_Q.1(i64 "reg"="d" %rd) "enc"="11110110000100001000 rd:5 1010011" "asm"="fli.q {rd}, min" "asmonly" {
  ret void
}

define void @FLI_Q.2(i64 "reg"="d" %rd) "enc"="11110110000100010000 rd:5 1010011" "asm"="fli.q {rd}, 1.52587890625e-05" "asmonly" {
  ret void
}

define void @FLI_Q.3(i64 "reg"="d" %rd) "enc"="11110110000100011000 rd:5 1010011" "asm"="fli.q {rd}, 3.0517578125e-05" "asmonly" {
  ret void
}

define void @FLI_Q.4(i64 "reg"="d" %rd) "enc"="11110110000100100000 rd:5 1010011" "asm"="fli.q {rd}, 0.00390625" "asmonly" {
  ret void
}

define void @FLI_Q.5(i64 "reg"="d" %rd) "enc"="11110110000100101000 rd:5 1010011" "asm"="fli.q {rd}, 0.0078125" "asmonly" {
  ret void
}

define void @FLI_Q.6(i64 "reg"="d" %rd) "enc"="11110110000100110000 rd:5 1010011" "asm"="fli.q {rd}, 0.0625" "asmonly" {
  ret void
}

define void @FLI_Q.7(i64 "reg"="d" %rd) "enc"="11110110000100111000 rd:5 1010011" "asm"="fli.q {rd}, 0.125" "asmonly" {
  ret void
}

define void @FLI_Q.8(i64 "reg"="d" %rd) "enc"="11110110000101000000 rd:5 1010011" "asm"="fli.q {rd}, 0.25" "asmonly" {
  ret void
}

define void @FLI_Q.9(i64 "reg"="d" %rd) "enc"="11110110000101001000 rd:5 1010011" "asm"="fli.q {rd}, 0.3125" "asmonly" {
  ret void
}

define void @FLI_Q.10(i64 "reg"="d" %rd) "enc"="11110110000101010000 rd:5 1010011" "asm"="fli.q {rd}, 0.375" "asmonly" {
  ret void
}

define void @FLI_Q.11(i64 "reg"="d" %rd) "enc"="11110110000101011000 rd:5 1010011" "asm"="fli.q {rd}, 0.4375" "asmonly" {
  ret void
}

define void @FLI_Q.12(i64 "reg"="d" %rd) "enc"="11110110000101100000 rd:5 1010011" "asm"="fli.q {rd}, 0.5" "asmonly" {
  ret void
}

define void @FLI_Q.13(i64 "reg"="d" %rd) "enc"="11110110000101101000 rd:5 1010011" "asm"="fli.q {rd}, 0.625" "asmonly" {
  ret void
}

define void @FLI_Q.14(i64 "reg"="d" %rd) "enc"="11110110000101110000 rd:5 1010011" "asm"="fli.q {rd}, 0.75" "asmonly" {
  ret void
}

define void @FLI_Q.15(i64 "reg"="d" %rd) "enc"="11110110000101111000 rd:5 1010011" "asm"="fli.q {rd}, 0.875" "asmonly" {
  ret void
}

define void @FLI_Q.16(i64 "reg"="d" %rd) "enc"="11110110000110000000 rd:5 1010011" "asm"="fli.q {rd}, 1.0" "asmonly" {
  ret void
}

define void @FLI_Q.17(i64 "reg"="d" %rd) "enc"="11110110000110001000 rd:5 1010011" "asm"="fli.q {rd}, 1.25" "asmonly" {
  ret void
}

define void @FLI_Q.18(i64 "reg"="d" %rd) "enc"="11110110000110010000 rd:5 1010011" "asm"="fli.q {rd}, 1.5" "asmonly" {
  ret void
}

define void @FLI_Q.19(i64 "reg"="d" %rd) "enc"="11110110000110011000 rd:5 1010011" "asm"="fli.q {rd}, 1.75" "asmonly" {
  ret void
}

define void @FLI_Q.20(i64 "reg"="d" %rd) "enc"="11110110000110100000 rd:5 1010011" "asm"="fli.q {rd}, 2.0" "asmonly" {
  ret void
}

define void @FLI_Q.21(i64 "reg"="d" %rd) "enc"="11110110000110101000 rd:5 1010011" "asm"="fli.q {rd}, 2.5" "asmonly" {
  ret void
}

define void @FLI_Q.22(i64 "reg"="d" %rd) "enc"="11110110000110110000 rd:5 1010011" "asm"="fli.q {rd}, 3.0" "asmonly" {
  ret void
}

define void @FLI_Q.23(i64 "reg"="d" %rd) "enc"="11110110000110111000 rd:5 1010011" "asm"="fli.q {rd}, 4.0" "asmonly" {
  ret void
}

define void @FLI_Q.24(i64 "reg"="d" %rd) "enc"="11110110000111000000 rd:5 1010011" "asm"="fli.q {rd}, 8.0" "asmonly" {
  ret void
}

define void @FLI_Q.25(i64 "reg"="d" %rd) "enc"="11110110000111001000 rd:5 1010011" "asm"="fli.q {rd}, 16.0" "asmonly" {
  ret void
}

define void @FLI_Q.26(i64 "reg"="d" %rd) "enc"="11110110000111010000 rd:5 1010011" "asm"="fli.q {rd}, 128.0" "asmonly" {
  ret void
}

define void @FLI_Q.27(i64 "reg"="d" %rd) "enc"="11110110000111011000 rd:5 1010011" "asm"="fli.q {rd}, 256.0" "asmonly" {
  ret void
}

define void @FLI_Q.28(i64 "reg"="d" %rd) "enc"="11110110000111100000 rd:5 1010011" "asm"="fli.q {rd}, 32768.0" "asmonly" {
  ret void
}

define void @FLI_Q.29(i64 "reg"="d" %rd) "enc"="11110110000111101000 rd:5 1010011" "asm"="fli.q {rd}, 65536.0" "asmonly" {
  ret void
}

define void @FLI_Q.30(i64 "reg"="d" %rd) "enc"="11110110000111110000 rd:5 1010011" "asm"="fli.q {rd}, inf" "asmonly" {
  ret void
}

define void @FLI_Q.31(i64 "reg"="d" %rd) "enc"="11110110000111111000 rd:5 1010011" "asm"="fli.q {rd}, nan" "asmonly" {
  ret void
}

define void @FMINM_Q.0(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2) "enc"="0010111 rs2:5 rs1:5 010 rd:5 1010011" "asm"="fminm.q {rd}, {rs1}, {rs2}" "asmonly" {
  ret void
}

define void @FMAXM_Q.0(i64 "reg"="d" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2) "enc"="0010111 rs2:5 rs1:5 011 rd:5 1010011" "asm"="fmaxm.q {rd}, {rs1}, {rs2}" "asmonly" {
  ret void
}

define void @FROUND_Q.0(i64 "reg"="d" %rd, i64 "reg"="d" %rs1) "enc"="010001100100 rs1:5 111 rd:5 1010011" "asm"="fround.q {rd}, {rs1}" "asmonly" {
  ret void
}

define void @FROUND_Q.1(i64 "reg"="d" %rd, i64 "reg"="d" %rs1) "enc"="010001100100 rs1:5 000 rd:5 1010011" "asm"="fround.q {rd}, {rs1}, rne" "asmonly" {
  ret void
}

define void @FROUND_Q.2(i64 "reg"="d" %rd, i64 "reg"="d" %rs1) "enc"="010001100100 rs1:5 001 rd:5 1010011" "asm"="fround.q {rd}, {rs1}, rtz" "asmonly" {
  ret void
}

define void @FROUND_Q.3(i64 "reg"="d" %rd, i64 "reg"="d" %rs1) "enc"="010001100100 rs1:5 010 rd:5 1010011" "asm"="fround.q {rd}, {rs1}, rdn" "asmonly" {
  ret void
}

define void @FROUND_Q.4(i64 "reg"="d" %rd, i64 "reg"="d" %rs1) "enc"="010001100100 rs1:5 011 rd:5 1010011" "asm"="fround.q {rd}, {rs1}, rup" "asmonly" {
  ret void
}

define void @FROUND_Q.5(i64 "reg"="d" %rd, i64 "reg"="d" %rs1) "enc"="010001100100 rs1:5 100 rd:5 1010011" "asm"="fround.q {rd}, {rs1}, rmm" "asmonly" {
  ret void
}

define void @FROUND_Q.6(i64 "reg"="d" %rd, i64 "reg"="d" %rs1) "enc"="010001100100 rs1:5 111 rd:5 1010011" "asm"="fround.q {rd}, {rs1}, dyn" "asmonly" "nodecode" {
  ret void
}

define void @FROUNDNX_Q.0(i64 "reg"="d" %rd, i64 "reg"="d" %rs1) "enc"="010001100101 rs1:5 111 rd:5 1010011" "asm"="froundnx.q {rd}, {rs1}" "asmonly" {
  ret void
}

define void @FROUNDNX_Q.1(i64 "reg"="d" %rd, i64 "reg"="d" %rs1) "enc"="010001100101 rs1:5 000 rd:5 1010011" "asm"="froundnx.q {rd}, {rs1}, rne" "asmonly" {
  ret void
}

define void @FROUNDNX_Q.2(i64 "reg"="d" %rd, i64 "reg"="d" %rs1) "enc"="010001100101 rs1:5 001 rd:5 1010011" "asm"="froundnx.q {rd}, {rs1}, rtz" "asmonly" {
  ret void
}

define void @FROUNDNX_Q.3(i64 "reg"="d" %rd, i64 "reg"="d" %rs1) "enc"="010001100101 rs1:5 010 rd:5 1010011" "asm"="froundnx.q {rd}, {rs1}, rdn" "asmonly" {
  ret void
}

define void @FROUNDNX_Q.4(i64 "reg"="d" %rd, i64 "reg"="d" %rs1) "enc"="010001100101 rs1:5 011 rd:5 1010011" "asm"="froundnx.q {rd}, {rs1}, rup" "asmonly" {
  ret void
}

define void @FROUNDNX_Q.5(i64 "reg"="d" %rd, i64 "reg"="d" %rs1) "enc"="010001100101 rs1:5 100 rd:5 1010011" "asm"="froundnx.q {rd}, {rs1}, rmm" "asmonly" {
  ret void
}

define void @FROUNDNX_Q.6(i64 "reg"="d" %rd, i64 "reg"="d" %rs1) "enc"="010001100101 rs1:5 111 rd:5 1010011" "asm"="froundnx.q {rd}, {rs1}, dyn" "asmonly" "nodecode" {
  ret void
}

define void @FLEQ_Q.0(i64 "reg"="x" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2) "enc"="1010011 rs2:5 rs1:5 100 rd:5 1010011" "asm"="fleq.q {rd}, {rs1}, {rs2}" "asmonly" {
  ret void
}

define void @FLTQ_Q.0(i64 "reg"="x" %rd, i64 "reg"="d" %rs1, i64 "reg"="d" %rs2) "enc"="1010011 rs2:5 rs1:5 101 rd:5 1010011" "asm"="fltq.q {rd}, {rs1}, {rs2}" "asmonly" {
  ret void
}

define void @FMVH_X_Q.0(i64 "reg"="x" %rd, i64 "reg"="d" %rs1) "enc"="111001100001 rs1:5 000 rd:5 1010011" "asm"="fmvh.x.q {rd}, {rs1}" "asmonly" {
  ret void
}

define void @FMVP_Q_X.0(i64 "reg"="d" %rd, i64 "reg"="x" %rs1, i64 "reg"="x" %rs2) "enc"="1011011 rs2:5 rs1:5 000 rd:5 1010011" "asm"="fmvp.q.x {rd}, {rs1}, {rs2}" "asmonly" {
  ret void
}

define void @FCVT_BF16_S.0(i64 "reg"="d" %rd, i64 "reg"="d" %rs1) "enc"="010001001000 rs1:5 111 rd:5 1010011" "asm"="fcvt.bf16.s {rd}, {rs1}" "asmonly" {
  ret void
}

define void @FCVT_BF16_S.1(i64 "reg"="d" %rd, i64 "reg"="d" %rs1) "enc"="010001001000 rs1:5 000 rd:5 1010011" "asm"="fcvt.bf16.s {rd}, {rs1}, rne" "asmonly" {
  ret void
}

define void @FCVT_BF16_S.2(i64 "reg"="d" %rd, i64 "reg"="d" %rs1) "enc"="010001001000 rs1:5 001 rd:5 1010011" "asm"="fcvt.bf16.s {rd}, {rs1}, rtz" "asmonly" {
  ret void
}

define void @FCVT_BF16_S.3(i64 "reg"="d" %rd, i64 "reg"="d" %rs1) "enc"="010001001000 rs1:5 010 rd:5 1010011" "asm"="fcvt.bf16.s {rd}, {rs1}, rdn" "asmonly" {
  ret void
}

define void @FCVT_BF16_S.4(i64 "reg"="d" %rd, i64 "reg"="d" %rs1) "enc"="010001001000 rs1:5 011 rd:5 1010011" "asm"="fcvt.bf16.s {rd}, {rs1}, rup" "asmonly" {
  ret void
}

define void @FCVT_BF16_S.5(i64 "reg"="d" %rd, i64 "reg"="d" %rs1) "enc"="010001001000 rs1:5 100 rd:5 1010011" "asm"="fcvt.bf16.s {rd}, {rs1}, rmm" "asmonly" {
  ret void
}

define void @FCVT_BF16_S.6(i64 "reg"="d" %rd, i64 "reg"="d" %rs1) "enc"="010001001000 rs1:5 111 rd:5 1010011" "asm"="fcvt.bf16.s {rd}, {rs1}, dyn" "asmonly" "nodecode" {
  ret void
}

define void @FCVT_S_BF16.0(i64 "reg"="d" %rd, i64 "reg"="d" %rs1) "enc"="010000000110 rs1:5 000 rd:5 1010011" "asm"="fcvt.s.bf16 {rd}, {rs1}" "asmonly" {
  ret void
}

define void @FCVT_S_BF16.1(i64 "reg"="d" %rd, i64 "reg"="d" %rs1) "enc"="010000000110 rs1:5 000 rd:5 1010011" "asm"="fcvt.s.bf16 {rd}, {rs1}, rne" "asmonly" "nodecode" {
  ret void
}

define void @FCVT_S_BF16.2(i64 "reg"="d" %rd, i64 "reg"="d" %rs1) "enc"="010000000110 rs1:5 001 rd:5 1010011" "asm"="fcvt.s.bf16 {rd}, {rs1}, rtz" "asmonly" {
  ret void
}

define void @FCVT_S_BF16.3(i64 "reg"="d" %rd, i64 "reg"="d" %rs1) "enc"="010000000110 rs1:5 010 rd:5 1010011" "asm"="fcvt.s.bf16 {rd}, {rs1}, rdn" "asmonly" {
  ret void
}

define void @FCVT_S_BF16.4(i64 "reg"="d" %rd, i64 "reg"="d" %rs1) "enc"="010000000110 rs1:5 011 rd:5 1010011" "asm"="fcvt.s.bf16 {rd}, {rs1}, rup" "asmonly" {
  ret void
}

define void @FCVT_S_BF16.5(i64 "reg"="d" %rd, i64 "reg"="d" %rs1) "enc"="010000000110 rs1:5 100 rd:5 1010011" "asm"="fcvt.s.bf16 {rd}, {rs1}, rmm" "asmonly" {
  ret void
}

define void @FCVT_S_BF16.6(i64 "reg"="d" %rd, i64 "reg"="d" %rs1) "enc"="010000000110 rs1:5 111 rd:5 1010011" "asm"="fcvt.s.bf16 {rd}, {rs1}, dyn" "asmonly" {
  ret void
}

define void @HFENCE_VVMA.0() "enc"="00100010000000000000000001110011" "asm"="hfence.vvma" "asmonly" {
  ret void
}

define void @HFENCE_VVMA.1(i64 "reg"="x" %rs1, i64 "reg"="x" %rs2) "enc"="0010001 rs2:5 rs1:5 000000001110011" "asm"="hfence.vvma {rs1}, {rs2}" "asmonly" {
  ret void
}

define void @HFENCE_GVMA.0() "enc"="01100010000000000000000001110011" "asm"="hfence.gvma" "asmonly" {
  ret void
}

define void @HFENCE_GVMA.1(i64 "reg"="x" %rs1, i64 "reg"="x" %rs2) "enc"="0110001 rs2:5 rs1:5 000000001110011" "asm"="hfence.gvma {rs1}, {rs2}" "asmonly" {
  ret void
}

define void @HLV_B.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1) "enc"="011000000000 rs1:5 100 rd:5 1110011" "asm"="hlv.b {rd}, ({rs1})" "asmonly" {
  ret void
}

define void @HLV_BU.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1) "enc"="011000000001 rs1:5 100 rd:5 1110011" "asm"="hlv.bu {rd}, ({rs1})" "asmonly" {
  ret void
}

define void @HLV_H.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1) "enc"="011001000000 rs1:5 100 rd:5 1110011" "asm"="hlv.h {rd}, ({rs1})" "asmonly" {
  ret void
}

define void @HLV_HU.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1) "enc"="011001000001 rs1:5 100 rd:5 1110011" "asm"="hlv.hu {rd}, ({rs1})" "asmonly" {
  ret void
}

define void @HLVX_HU.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1) "enc"="011001000011 rs1:5 100 rd:5 1110011" "asm"="hlvx.hu {rd}, ({rs1})" "asmonly" {
  ret void
}

define void @HLV_W.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1) "enc"="011010000000 rs1:5 100 rd:5 1110011" "asm"="hlv.w {rd}, ({rs1})" "asmonly" {
  ret void
}

define void @HLVX_WU.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1) "enc"="011010000011 rs1:5 100 rd:5 1110011" "asm"="hlvx.wu {rd}, ({rs1})" "asmonly" {
  ret void
}

define void @HSV_B.0(i64 "reg"="x" %rs1, i64 "reg"="x" %rs2) "enc"="0110001 rs2:5 rs1:5 100000001110011" "asm"="hsv.b {rs2}, ({rs1})" "asmonly" {
  ret void
}

define void @HSV_H.0(i64 "reg"="x" %rs1, i64 "reg"="x" %rs2) "enc"="0110011 rs2:5 rs1:5 100000001110011" "asm"="hsv.h {rs2}, ({rs1})" "asmonly" {
  ret void
}

define void @HSV_W.0(i64 "reg"="x" %rs1, i64 "reg"="x" %rs2) "enc"="0110101 rs2:5 rs1:5 100000001110011" "asm"="hsv.w {rs2}, ({rs1})" "asmonly" {
  ret void
}

define void @HLV_WU.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1) "enc"="011010000001 rs1:5 100 rd:5 1110011" "asm"="hlv.wu {rd}, ({rs1})" "asmonly" {
  ret void
}

define void @HLV_D.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1) "enc"="011011000000 rs1:5 100 rd:5 1110011" "asm"="hlv.d {rd}, ({rs1})" "asmonly" {
  ret void
}

define void @HSV_D.0(i64 "reg"="x" %rs1, i64 "reg"="x" %rs2) "enc"="0110111 rs2:5 rs1:5 100000001110011" "asm"="hsv.d {rs2}, ({rs1})" "asmonly" {
  ret void
}

define void @HINVAL_VVMA.0(i64 "reg"="x" %rs1, i64 "reg"="x" %rs2) "enc"="0010011 rs2:5 rs1:5 000000001110011" "asm"="hinval.vvma {rs1}, {rs2}" "asmonly" {
  ret void
}

define void @HINVAL_GVMA.0(i64 "reg"="x" %rs1, i64 "reg"="x" %rs2) "enc"="0110011 rs2:5 rs1:5 000000001110011" "asm"="hinval.gvma {rs1}, {rs2}" "asmonly" {
  ret void
}

define void @DRET.0() "enc"="01111011001000000000000001110011" "asm"="dret" "asmonly" {
  ret void
}

define void @MNRET.0() "enc"="01110000001000000000000001110011" "asm"="mnret" "asmonly" {
  ret void
}

define void @SCTRCLR.0() "enc"="00010000010000000000000001110011" "asm"="sctrclr" "asmonly" {
  ret void
}

define void @VSETIVLI.0(i64 "reg"="x" %rd, i64 "imm" %zimm5) "enc"="110000000000 zimm5:5 111 rd:5 1010111" "asm"="vsetivli {rd}, {zimm5}, e8, m1, tu, mu" "asmonly" {
  ret void
}

define void @VSETIVLI.1(i64 "reg"="x" %rd, i64 "imm" %zimm5) "enc"="110010000000 zimm5:5 111 rd:5 1010111" "asm"="vsetivli {rd}, {zimm5}, e8, m1, tu, ma" "asmonly" {
  ret void
}

define void @VSETIVLI.2(i64 "reg"="x" %rd, i64 "imm" %zimm5) "enc"="110001000000 zimm5:5 111 rd:5 1010111" "asm"="vsetivli {rd}, {zimm5}, e8, m1, ta, mu" "asmonly" {
  ret void
}

define void @VSETIVLI.3(i64 "reg"="x" %rd, i64 "imm" %zimm5) "enc"="110011000000 zimm5:5 111 rd:5 1010111" "asm"="vsetivli {rd}, {zimm5}, e8, m1, ta, ma" "asmonly" {
  ret void
}

define void @VSETIVLI.4(i64 "reg"="x" %rd, i64 "imm" %zimm5) "enc"="110000000001 zimm5:5 111 rd:5 1010111" "asm"="vsetivli {rd}, {zimm5}, e8, m2, tu, mu" "asmonly" {
  ret void
}

define void @VSETIVLI.5(i64 "reg"="x" %rd, i64 "imm" %zimm5) "enc"="110010000001 zimm5:5 111 rd:5 1010111" "asm"="vsetivli {rd}, {zimm5}, e8, m2, tu, ma" "asmonly" {
  ret void
}

define void @VSETIVLI.6(i64 "reg"="x" %rd, i64 "imm" %zimm5) "enc"="110001000001 zimm5:5 111 rd:5 1010111" "asm"="vsetivli {rd}, {zimm5}, e8, m2, ta, mu" "asmonly" {
  ret void
}

define void @VSETIVLI.7(i64 "reg"="x" %rd, i64 "imm" %zimm5) "enc"="110011000001 zimm5:5 111 rd:5 1010111" "asm"="vsetivli {rd}, {zimm5}, e8, m2, ta, ma" "asmonly" {
  ret void
}

define void @VSETIVLI.8(i64 "reg"="x" %rd, i64 "imm" %zimm5) "enc"="110000000010 zimm5:5 111 rd:5 1010111" "asm"="vsetivli {rd}, {zimm5}, e8, m4, tu, mu" "asmonly" {
  ret void
}

define void @VSETIVLI.9(i64 "reg"="x" %rd, i64 "imm" %zimm5) "enc"="110010000010 zimm5:5 111 rd:5 1010111" "asm"="vsetivli {rd}, {zimm5}, e8, m4, tu, ma" "asmonly" {
  ret void
}

define void @VSETIVLI.10(i64 "reg"="x" %rd, i64 "imm" %zimm5) "enc"="110001000010 zimm5:5 111 rd:5 1010111" "asm"="vsetivli {rd}, {zimm5}, e8, m4, ta, mu" "asmonly" {
  ret void
}

define void @VSETIVLI.11(i64 "reg"="x" %rd, i64 "imm" %zimm5) "enc"="110011000010 zimm5:5 111 rd:5 1010111" "asm"="vsetivli {rd}, {zimm5}, e8, m4, ta, ma" "asmonly" {
  ret void
}

define void @VSETIVLI.12(i64 "reg"="x" %rd, i64 "imm" %zimm5) "enc"="110000000011 zimm5:5 111 rd:5 1010111" "asm"="vsetivli {rd}, {zimm5}, e8, m8, tu, mu" "asmonly" {
  ret void
}

define void @VSETIVLI.13(i64 "reg"="x" %rd, i64 "imm" %zimm5) "enc"="110010000011 zimm5:5 111 rd:5 1010111" "asm"="vsetivli {rd}, {zimm5}, e8, m8, tu, ma" "asmonly" {
  ret void
}

define void @VSETIVLI.14(i64 "reg"="x" %rd, i64 "imm" %zimm5) "enc"="110001000011 zimm5:5 111 rd:5 1010111" "asm"="vsetivli {rd}, {zimm5}, e8, m8, ta, mu" "asmonly" {
  ret void
}

define void @VSETIVLI.15(i64 "reg"="x" %rd, i64 "imm" %zimm5) "enc"="110011000011 zimm5:5 111 rd:5 1010111" "asm"="vsetivli {rd}, {zimm5}, e8, m8, ta, ma" "asmonly" {
  ret void
}

define void @VSETIVLI.16(i64 "reg"="x" %rd, i64 "imm" %zimm5) "enc"="110000000101 zimm5:5 111 rd:5 1010111" "asm"="vsetivli {rd}, {zimm5}, e8, mf8, tu, mu" "asmonly" {
  ret void
}

define void @VSETIVLI.17(i64 "reg"="x" %rd, i64 "imm" %zimm5) "enc"="110010000101 zimm5:5 111 rd:5 1010111" "asm"="vsetivli {rd}, {zimm5}, e8, mf8, tu, ma" "asmonly" {
  ret void
}

define void @VSETIVLI.18(i64 "reg"="x" %rd, i64 "imm" %zimm5) "enc"="110001000101 zimm5:5 111 rd:5 1010111" "asm"="vsetivli {rd}, {zimm5}, e8, mf8, ta, mu" "asmonly" {
  ret void
}

define void @VSETIVLI.19(i64 "reg"="x" %rd, i64 "imm" %zimm5) "enc"="110011000101 zimm5:5 111 rd:5 1010111" "asm"="vsetivli {rd}, {zimm5}, e8, mf8, ta, ma" "asmonly" {
  ret void
}

define void @VSETIVLI.20(i64 "reg"="x" %rd, i64 "imm" %zimm5) "enc"="110000000110 zimm5:5 111 rd:5 1010111" "asm"="vsetivli {rd}, {zimm5}, e8, mf4, tu, mu" "asmonly" {
  ret void
}

define void @VSETIVLI.21(i64 "reg"="x" %rd, i64 "imm" %zimm5) "enc"="110010000110 zimm5:5 111 rd:5 1010111" "asm"="vsetivli {rd}, {zimm5}, e8, mf4, tu, ma" "asmonly" {
  ret void
}

define void @VSETIVLI.22(i64 "reg"="x" %rd, i64 "imm" %zimm5) "enc"="110001000110 zimm5:5 111 rd:5 1010111" "asm"="vsetivli {rd}, {zimm5}, e8, mf4, ta, mu" "asmonly" {
  ret void
}

define void @VSETIVLI.23(i64 "reg"="x" %rd, i64 "imm" %zimm5) "enc"="110011000110 zimm5:5 111 rd:5 1010111" "asm"="vsetivli {rd}, {zimm5}, e8, mf4, ta, ma" "asmonly" {
  ret void
}

define void @VSETIVLI.24(i64 "reg"="x" %rd, i64 "imm" %zimm5) "enc"="110000000111 zimm5:5 111 rd:5 1010111" "asm"="vsetivli {rd}, {zimm5}, e8, mf2, tu, mu" "asmonly" {
  ret void
}

define void @VSETIVLI.25(i64 "reg"="x" %rd, i64 "imm" %zimm5) "enc"="110010000111 zimm5:5 111 rd:5 1010111" "asm"="vsetivli {rd}, {zimm5}, e8, mf2, tu, ma" "asmonly" {
  ret void
}

define void @VSETIVLI.26(i64 "reg"="x" %rd, i64 "imm" %zimm5) "enc"="110001000111 zimm5:5 111 rd:5 1010111" "asm"="vsetivli {rd}, {zimm5}, e8, mf2, ta, mu" "asmonly" {
  ret void
}

define void @VSETIVLI.27(i64 "reg"="x" %rd, i64 "imm" %zimm5) "enc"="110011000111 zimm5:5 111 rd:5 1010111" "asm"="vsetivli {rd}, {zimm5}, e8, mf2, ta, ma" "asmonly" {
  ret void
}

define void @VSETIVLI.28(i64 "reg"="x" %rd, i64 "imm" %zimm5) "enc"="110000001000 zimm5:5 111 rd:5 1010111" "asm"="vsetivli {rd}, {zimm5}, e16, m1, tu, mu" "asmonly" {
  ret void
}

define void @VSETIVLI.29(i64 "reg"="x" %rd, i64 "imm" %zimm5) "enc"="110010001000 zimm5:5 111 rd:5 1010111" "asm"="vsetivli {rd}, {zimm5}, e16, m1, tu, ma" "asmonly" {
  ret void
}

define void @VSETIVLI.30(i64 "reg"="x" %rd, i64 "imm" %zimm5) "enc"="110001001000 zimm5:5 111 rd:5 1010111" "asm"="vsetivli {rd}, {zimm5}, e16, m1, ta, mu" "asmonly" {
  ret void
}

define void @VSETIVLI.31(i64 "reg"="x" %rd, i64 "imm" %zimm5) "enc"="110011001000 zimm5:5 111 rd:5 1010111" "asm"="vsetivli {rd}, {zimm5}, e16, m1, ta, ma" "asmonly" {
  ret void
}

define void @VSETIVLI.32(i64 "reg"="x" %rd, i64 "imm" %zimm5) "enc"="110000001001 zimm5:5 111 rd:5 1010111" "asm"="vsetivli {rd}, {zimm5}, e16, m2, tu, mu" "asmonly" {
  ret void
}

define void @VSETIVLI.33(i64 "reg"="x" %rd, i64 "imm" %zimm5) "enc"="110010001001 zimm5:5 111 rd:5 1010111" "asm"="vsetivli {rd}, {zimm5}, e16, m2, tu, ma" "asmonly" {
  ret void
}

define void @VSETIVLI.34(i64 "reg"="x" %rd, i64 "imm" %zimm5) "enc"="110001001001 zimm5:5 111 rd:5 1010111" "asm"="vsetivli {rd}, {zimm5}, e16, m2, ta, mu" "asmonly" {
  ret void
}

define void @VSETIVLI.35(i64 "reg"="x" %rd, i64 "imm" %zimm5) "enc"="110011001001 zimm5:5 111 rd:5 1010111" "asm"="vsetivli {rd}, {zimm5}, e16, m2, ta, ma" "asmonly" {
  ret void
}

define void @VSETIVLI.36(i64 "reg"="x" %rd, i64 "imm" %zimm5) "enc"="110000001010 zimm5:5 111 rd:5 1010111" "asm"="vsetivli {rd}, {zimm5}, e16, m4, tu, mu" "asmonly" {
  ret void
}

define void @VSETIVLI.37(i64 "reg"="x" %rd, i64 "imm" %zimm5) "enc"="110010001010 zimm5:5 111 rd:5 1010111" "asm"="vsetivli {rd}, {zimm5}, e16, m4, tu, ma" "asmonly" {
  ret void
}

define void @VSETIVLI.38(i64 "reg"="x" %rd, i64 "imm" %zimm5) "enc"="110001001010 zimm5:5 111 rd:5 1010111" "asm"="vsetivli {rd}, {zimm5}, e16, m4, ta, mu" "asmonly" {
  ret void
}

define void @VSETIVLI.39(i64 "reg"="x" %rd, i64 "imm" %zimm5) "enc"="110011001010 zimm5:5 111 rd:5 1010111" "asm"="vsetivli {rd}, {zimm5}, e16, m4, ta, ma" "asmonly" {
  ret void
}

define void @VSETIVLI.40(i64 "reg"="x" %rd, i64 "imm" %zimm5) "enc"="110000001011 zimm5:5 111 rd:5 1010111" "asm"="vsetivli {rd}, {zimm5}, e16, m8, tu, mu" "asmonly" {
  ret void
}

define void @VSETIVLI.41(i64 "reg"="x" %rd, i64 "imm" %zimm5) "enc"="110010001011 zimm5:5 111 rd:5 1010111" "asm"="vsetivli {rd}, {zimm5}, e16, m8, tu, ma" "asmonly" {
  ret void
}

define void @VSETIVLI.42(i64 "reg"="x" %rd, i64 "imm" %zimm5) "enc"="110001001011 zimm5:5 111 rd:5 1010111" "asm"="vsetivli {rd}, {zimm5}, e16, m8, ta, mu" "asmonly" {
  ret void
}

define void @VSETIVLI.43(i64 "reg"="x" %rd, i64 "imm" %zimm5) "enc"="110011001011 zimm5:5 111 rd:5 1010111" "asm"="vsetivli {rd}, {zimm5}, e16, m8, ta, ma" "asmonly" {
  ret void
}

define void @VSETIVLI.44(i64 "reg"="x" %rd, i64 "imm" %zimm5) "enc"="110000001110 zimm5:5 111 rd:5 1010111" "asm"="vsetivli {rd}, {zimm5}, e16, mf4, tu, mu" "asmonly" {
  ret void
}

define void @VSETIVLI.45(i64 "reg"="x" %rd, i64 "imm" %zimm5) "enc"="110010001110 zimm5:5 111 rd:5 1010111" "asm"="vsetivli {rd}, {zimm5}, e16, mf4, tu, ma" "asmonly" {
  ret void
}

define void @VSETIVLI.46(i64 "reg"="x" %rd, i64 "imm" %zimm5) "enc"="110001001110 zimm5:5 111 rd:5 1010111" "asm"="vsetivli {rd}, {zimm5}, e16, mf4, ta, mu" "asmonly" {
  ret void
}

define void @VSETIVLI.47(i64 "reg"="x" %rd, i64 "imm" %zimm5) "enc"="110011001110 zimm5:5 111 rd:5 1010111" "asm"="vsetivli {rd}, {zimm5}, e16, mf4, ta, ma" "asmonly" {
  ret void
}

define void @VSETIVLI.48(i64 "reg"="x" %rd, i64 "imm" %zimm5) "enc"="110000001111 zimm5:5 111 rd:5 1010111" "asm"="vsetivli {rd}, {zimm5}, e16, mf2, tu, mu" "asmonly" {
  ret void
}

define void @VSETIVLI.49(i64 "reg"="x" %rd, i64 "imm" %zimm5) "enc"="110010001111 zimm5:5 111 rd:5 1010111" "asm"="vsetivli {rd}, {zimm5}, e16, mf2, tu, ma" "asmonly" {
  ret void
}

define void @VSETIVLI.50(i64 "reg"="x" %rd, i64 "imm" %zimm5) "enc"="110001001111 zimm5:5 111 rd:5 1010111" "asm"="vsetivli {rd}, {zimm5}, e16, mf2, ta, mu" "asmonly" {
  ret void
}

define void @VSETIVLI.51(i64 "reg"="x" %rd, i64 "imm" %zimm5) "enc"="110011001111 zimm5:5 111 rd:5 1010111" "asm"="vsetivli {rd}, {zimm5}, e16, mf2, ta, ma" "asmonly" {
  ret void
}

define void @VSETIVLI.52(i64 "reg"="x" %rd, i64 "imm" %zimm5) "enc"="110000010000 zimm5:5 111 rd:5 1010111" "asm"="vsetivli {rd}, {zimm5}, e32, m1, tu, mu" "asmonly" {
  ret void
}

define void @VSETIVLI.53(i64 "reg"="x" %rd, i64 "imm" %zimm5) "enc"="110010010000 zimm5:5 111 rd:5 1010111" "asm"="vsetivli {rd}, {zimm5}, e32, m1, tu, ma" "asmonly" {
  ret void
}

define void @VSETIVLI.54(i64 "reg"="x" %rd, i64 "imm" %zimm5) "enc"="110001010000 zimm5:5 111 rd:5 1010111" "asm"="vsetivli {rd}, {zimm5}, e32, m1, ta, mu" "asmonly" {
  ret void
}

define void @VSETIVLI.55(i64 "reg"="x" %rd, i64 "imm" %zimm5) "enc"="110011010000 zimm5:5 111 rd:5 1010111" "asm"="vsetivli {rd}, {zimm5}, e32, m1, ta, ma" "asmonly" {
  ret void
}

define void @VSETIVLI.56(i64 "reg"="x" %rd, i64 "imm" %zimm5) "enc"="110000010001 zimm5:5 111 rd:5 1010111" "asm"="vsetivli {rd}, {zimm5}, e32, m2, tu, mu" "asmonly" {
  ret void
}

define void @VSETIVLI.57(i64 "reg"="x" %rd, i64 "imm" %zimm5) "enc"="110010010001 zimm5:5 111 rd:5 1010111" "asm"="vsetivli {rd}, {zimm5}, e32, m2, tu, ma" "asmonly" {
  ret void
}

define void @VSETIVLI.58(i64 "reg"="x" %rd, i64 "imm" %zimm5) "enc"="110001010001 zimm5:5 111 rd:5 1010111" "asm"="vsetivli {rd}, {zimm5}, e32, m2, ta, mu" "asmonly" {
  ret void
}

define void @VSETIVLI.59(i64 "reg"="x" %rd, i64 "imm" %zimm5) "enc"="110011010001 zimm5:5 111 rd:5 1010111" "asm"="vsetivli {rd}, {zimm5}, e32, m2, ta, ma" "asmonly" {
  ret void
}

define void @VSETIVLI.60(i64 "reg"="x" %rd, i64 "imm" %zimm5) "enc"="110000010010 zimm5:5 111 rd:5 1010111" "asm"="vsetivli {rd}, {zimm5}, e32, m4, tu, mu" "asmonly" {
  ret void
}

define void @VSETIVLI.61(i64 "reg"="x" %rd, i64 "imm" %zimm5) "enc"="110010010010 zimm5:5 111 rd:5 1010111" "asm"="vsetivli {rd}, {zimm5}, e32, m4, tu, ma" "asmonly" {
  ret void
}

define void @VSETIVLI.62(i64 "reg"="x" %rd, i64 "imm" %zimm5) "enc"="110001010010 zimm5:5 111 rd:5 1010111" "asm"="vsetivli {rd}, {zimm5}, e32, m4, ta, mu" "asmonly" {
  ret void
}

define void @VSETIVLI.63(i64 "reg"="x" %rd, i64 "imm" %zimm5) "enc"="110011010010 zimm5:5 111 rd:5 1010111" "asm"="vsetivli {rd}, {zimm5}, e32, m4, ta, ma" "asmonly" {
  ret void
}

define void @VSETIVLI.64(i64 "reg"="x" %rd, i64 "imm" %zimm5) "enc"="110000010011 zimm5:5 111 rd:5 1010111" "asm"="vsetivli {rd}, {zimm5}, e32, m8, tu, mu" "asmonly" {
  ret void
}

define void @VSETIVLI.65(i64 "reg"="x" %rd, i64 "imm" %zimm5) "enc"="110010010011 zimm5:5 111 rd:5 1010111" "asm"="vsetivli {rd}, {zimm5}, e32, m8, tu, ma" "asmonly" {
  ret void
}

define void @VSETIVLI.66(i64 "reg"="x" %rd, i64 "imm" %zimm5) "enc"="110001010011 zimm5:5 111 rd:5 1010111" "asm"="vsetivli {rd}, {zimm5}, e32, m8, ta, mu" "asmonly" {
  ret void
}

define void @VSETIVLI.67(i64 "reg"="x" %rd, i64 "imm" %zimm5) "enc"="110011010011 zimm5:5 111 rd:5 1010111" "asm"="vsetivli {rd}, {zimm5}, e32, m8, ta, ma" "asmonly" {
  ret void
}

define void @VSETIVLI.68(i64 "reg"="x" %rd, i64 "imm" %zimm5) "enc"="110000010111 zimm5:5 111 rd:5 1010111" "asm"="vsetivli {rd}, {zimm5}, e32, mf2, tu, mu" "asmonly" {
  ret void
}

define void @VSETIVLI.69(i64 "reg"="x" %rd, i64 "imm" %zimm5) "enc"="110010010111 zimm5:5 111 rd:5 1010111" "asm"="vsetivli {rd}, {zimm5}, e32, mf2, tu, ma" "asmonly" {
  ret void
}

define void @VSETIVLI.70(i64 "reg"="x" %rd, i64 "imm" %zimm5) "enc"="110001010111 zimm5:5 111 rd:5 1010111" "asm"="vsetivli {rd}, {zimm5}, e32, mf2, ta, mu" "asmonly" {
  ret void
}

define void @VSETIVLI.71(i64 "reg"="x" %rd, i64 "imm" %zimm5) "enc"="110011010111 zimm5:5 111 rd:5 1010111" "asm"="vsetivli {rd}, {zimm5}, e32, mf2, ta, ma" "asmonly" {
  ret void
}

define void @VSETIVLI.72(i64 "reg"="x" %rd, i64 "imm" %zimm5) "enc"="110000011000 zimm5:5 111 rd:5 1010111" "asm"="vsetivli {rd}, {zimm5}, e64, m1, tu, mu" "asmonly" {
  ret void
}

define void @VSETIVLI.73(i64 "reg"="x" %rd, i64 "imm" %zimm5) "enc"="110010011000 zimm5:5 111 rd:5 1010111" "asm"="vsetivli {rd}, {zimm5}, e64, m1, tu, ma" "asmonly" {
  ret void
}

define void @VSETIVLI.74(i64 "reg"="x" %rd, i64 "imm" %zimm5) "enc"="110001011000 zimm5:5 111 rd:5 1010111" "asm"="vsetivli {rd}, {zimm5}, e64, m1, ta, mu" "asmonly" {
  ret void
}

define void @VSETIVLI.75(i64 "reg"="x" %rd, i64 "imm" %zimm5) "enc"="110011011000 zimm5:5 111 rd:5 1010111" "asm"="vsetivli {rd}, {zimm5}, e64, m1, ta, ma" "asmonly" {
  ret void
}

define void @VSETIVLI.76(i64 "reg"="x" %rd, i64 "imm" %zimm5) "enc"="110000011001 zimm5:5 111 rd:5 1010111" "asm"="vsetivli {rd}, {zimm5}, e64, m2, tu, mu" "asmonly" {
  ret void
}

define void @VSETIVLI.77(i64 "reg"="x" %rd, i64 "imm" %zimm5) "enc"="110010011001 zimm5:5 111 rd:5 1010111" "asm"="vsetivli {rd}, {zimm5}, e64, m2, tu, ma" "asmonly" {
  ret void
}

define void @VSETIVLI.78(i64 "reg"="x" %rd, i64 "imm" %zimm5) "enc"="110001011001 zimm5:5 111 rd:5 1010111" "asm"="vsetivli {rd}, {zimm5}, e64, m2, ta, mu" "asmonly" {
  ret void
}

define void @VSETIVLI.79(i64 "reg"="x" %rd, i64 "imm" %zimm5) "enc"="110011011001 zimm5:5 111 rd:5 1010111" "asm"="vsetivli {rd}, {zimm5}, e64, m2, ta, ma" "asmonly" {
  ret void
}

define void @VSETIVLI.80(i64 "reg"="x" %rd, i64 "imm" %zimm5) "enc"="110000011010 zimm5:5 111 rd:5 1010111" "asm"="vsetivli {rd}, {zimm5}, e64, m4, tu, mu" "asmonly" {
  ret void
}

define void @VSETIVLI.81(i64 "reg"="x" %rd, i64 "imm" %zimm5) "enc"="110010011010 zimm5:5 111 rd:5 1010111" "asm"="vsetivli {rd}, {zimm5}, e64, m4, tu, ma" "asmonly" {
  ret void
}

define void @VSETIVLI.82(i64 "reg"="x" %rd, i64 "imm" %zimm5) "enc"="110001011010 zimm5:5 111 rd:5 1010111" "asm"="vsetivli {rd}, {zimm5}, e64, m4, ta, mu" "asmonly" {
  ret void
}

define void @VSETIVLI.83(i64 "reg"="x" %rd, i64 "imm" %zimm5) "enc"="110011011010 zimm5:5 111 rd:5 1010111" "asm"="vsetivli {rd}, {zimm5}, e64, m4, ta, ma" "asmonly" {
  ret void
}

define void @VSETIVLI.84(i64 "reg"="x" %rd, i64 "imm" %zimm5) "enc"="110000011011 zimm5:5 111 rd:5 1010111" "asm"="vsetivli {rd}, {zimm5}, e64, m8, tu, mu" "asmonly" {
  ret void
}

define void @VSETIVLI.85(i64 "reg"="x" %rd, i64 "imm" %zimm5) "enc"="110010011011 zimm5:5 111 rd:5 1010111" "asm"="vsetivli {rd}, {zimm5}, e64, m8, tu, ma" "asmonly" {
  ret void
}

define void @VSETIVLI.86(i64 "reg"="x" %rd, i64 "imm" %zimm5) "enc"="110001011011 zimm5:5 111 rd:5 1010111" "asm"="vsetivli {rd}, {zimm5}, e64, m8, ta, mu" "asmonly" {
  ret void
}

define void @VSETIVLI.87(i64 "reg"="x" %rd, i64 "imm" %zimm5) "enc"="110011011011 zimm5:5 111 rd:5 1010111" "asm"="vsetivli {rd}, {zimm5}, e64, m8, ta, ma" "asmonly" {
  ret void
}

define void @VSETVLI.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1) "enc"="000000000000 rs1:5 111 rd:5 1010111" "asm"="vsetvli {rd}, {rs1}, e8, m1, tu, mu" "asmonly" {
  ret void
}

define void @VSETVLI.1(i64 "reg"="x" %rd, i64 "reg"="x" %rs1) "enc"="000010000000 rs1:5 111 rd:5 1010111" "asm"="vsetvli {rd}, {rs1}, e8, m1, tu, ma" "asmonly" {
  ret void
}

define void @VSETVLI.2(i64 "reg"="x" %rd, i64 "reg"="x" %rs1) "enc"="000001000000 rs1:5 111 rd:5 1010111" "asm"="vsetvli {rd}, {rs1}, e8, m1, ta, mu" "asmonly" {
  ret void
}

define void @VSETVLI.3(i64 "reg"="x" %rd, i64 "reg"="x" %rs1) "enc"="000011000000 rs1:5 111 rd:5 1010111" "asm"="vsetvli {rd}, {rs1}, e8, m1, ta, ma" "asmonly" {
  ret void
}

define void @VSETVLI.4(i64 "reg"="x" %rd, i64 "reg"="x" %rs1) "enc"="000000000001 rs1:5 111 rd:5 1010111" "asm"="vsetvli {rd}, {rs1}, e8, m2, tu, mu" "asmonly" {
  ret void
}

define void @VSETVLI.5(i64 "reg"="x" %rd, i64 "reg"="x" %rs1) "enc"="000010000001 rs1:5 111 rd:5 1010111" "asm"="vsetvli {rd}, {rs1}, e8, m2, tu, ma" "asmonly" {
  ret void
}

define void @VSETVLI.6(i64 "reg"="x" %rd, i64 "reg"="x" %rs1) "enc"="000001000001 rs1:5 111 rd:5 1010111" "asm"="vsetvli {rd}, {rs1}, e8, m2, ta, mu" "asmonly" {
  ret void
}

define void @VSETVLI.7(i64 "reg"="x" %rd, i64 "reg"="x" %rs1) "enc"="000011000001 rs1:5 111 rd:5 1010111" "asm"="vsetvli {rd}, {rs1}, e8, m2, ta, ma" "asmonly" {
  ret void
}

define void @VSETVLI.8(i64 "reg"="x" %rd, i64 "reg"="x" %rs1) "enc"="000000000010 rs1:5 111 rd:5 1010111" "asm"="vsetvli {rd}, {rs1}, e8, m4, tu, mu" "asmonly" {
  ret void
}

define void @VSETVLI.9(i64 "reg"="x" %rd, i64 "reg"="x" %rs1) "enc"="000010000010 rs1:5 111 rd:5 1010111" "asm"="vsetvli {rd}, {rs1}, e8, m4, tu, ma" "asmonly" {
  ret void
}

define void @VSETVLI.10(i64 "reg"="x" %rd, i64 "reg"="x" %rs1) "enc"="000001000010 rs1:5 111 rd:5 1010111" "asm"="vsetvli {rd}, {rs1}, e8, m4, ta, mu" "asmonly" {
  ret void
}

define void @VSETVLI.11(i64 "reg"="x" %rd, i64 "reg"="x" %rs1) "enc"="000011000010 rs1:5 111 rd:5 1010111" "asm"="vsetvli {rd}, {rs1}, e8, m4, ta, ma" "asmonly" {
  ret void
}

define void @VSETVLI.12(i64 "reg"="x" %rd, i64 "reg"="x" %rs1) "enc"="000000000011 rs1:5 111 rd:5 1010111" "asm"="vsetvli {rd}, {rs1}, e8, m8, tu, mu" "asmonly" {
  ret void
}

define void @VSETVLI.13(i64 "reg"="x" %rd, i64 "reg"="x" %rs1) "enc"="000010000011 rs1:5 111 rd:5 1010111" "asm"="vsetvli {rd}, {rs1}, e8, m8, tu, ma" "asmonly" {
  ret void
}

define void @VSETVLI.14(i64 "reg"="x" %rd, i64 "reg"="x" %rs1) "enc"="000001000011 rs1:5 111 rd:5 1010111" "asm"="vsetvli {rd}, {rs1}, e8, m8, ta, mu" "asmonly" {
  ret void
}

define void @VSETVLI.15(i64 "reg"="x" %rd, i64 "reg"="x" %rs1) "enc"="000011000011 rs1:5 111 rd:5 1010111" "asm"="vsetvli {rd}, {rs1}, e8, m8, ta, ma" "asmonly" {
  ret void
}

define void @VSETVLI.16(i64 "reg"="x" %rd, i64 "reg"="x" %rs1) "enc"="000000000101 rs1:5 111 rd:5 1010111" "asm"="vsetvli {rd}, {rs1}, e8, mf8, tu, mu" "asmonly" {
  ret void
}

define void @VSETVLI.17(i64 "reg"="x" %rd, i64 "reg"="x" %rs1) "enc"="000010000101 rs1:5 111 rd:5 1010111" "asm"="vsetvli {rd}, {rs1}, e8, mf8, tu, ma" "asmonly" {
  ret void
}

define void @VSETVLI.18(i64 "reg"="x" %rd, i64 "reg"="x" %rs1) "enc"="000001000101 rs1:5 111 rd:5 1010111" "asm"="vsetvli {rd}, {rs1}, e8, mf8, ta, mu" "asmonly" {
  ret void
}

define void @VSETVLI.19(i64 "reg"="x" %rd, i64 "reg"="x" %rs1) "enc"="000011000101 rs1:5 111 rd:5 1010111" "asm"="vsetvli {rd}, {rs1}, e8, mf8, ta, ma" "asmonly" {
  ret void
}

define void @VSETVLI.20(i64 "reg"="x" %rd, i64 "reg"="x" %rs1) "enc"="000000000110 rs1:5 111 rd:5 1010111" "asm"="vsetvli {rd}, {rs1}, e8, mf4, tu, mu" "asmonly" {
  ret void
}

define void @VSETVLI.21(i64 "reg"="x" %rd, i64 "reg"="x" %rs1) "enc"="000010000110 rs1:5 111 rd:5 1010111" "asm"="vsetvli {rd}, {rs1}, e8, mf4, tu, ma" "asmonly" {
  ret void
}

define void @VSETVLI.22(i64 "reg"="x" %rd, i64 "reg"="x" %rs1) "enc"="000001000110 rs1:5 111 rd:5 1010111" "asm"="vsetvli {rd}, {rs1}, e8, mf4, ta, mu" "asmonly" {
  ret void
}

define void @VSETVLI.23(i64 "reg"="x" %rd, i64 "reg"="x" %rs1) "enc"="000011000110 rs1:5 111 rd:5 1010111" "asm"="vsetvli {rd}, {rs1}, e8, mf4, ta, ma" "asmonly" {
  ret void
}

define void @VSETVLI.24(i64 "reg"="x" %rd, i64 "reg"="x" %rs1) "enc"="000000000111 rs1:5 111 rd:5 1010111" "asm"="vsetvli {rd}, {rs1}, e8, mf2, tu, mu" "asmonly" {
  ret void
}

define void @VSETVLI.25(i64 "reg"="x" %rd, i64 "reg"="x" %rs1) "enc"="000010000111 rs1:5 111 rd:5 1010111" "asm"="vsetvli {rd}, {rs1}, e8, mf2, tu, ma" "asmonly" {
  ret void
}

define void @VSETVLI.26(i64 "reg"="x" %rd, i64 "reg"="x" %rs1) "enc"="000001000111 rs1:5 111 rd:5 1010111" "asm"="vsetvli {rd}, {rs1}, e8, mf2, ta, mu" "asmonly" {
  ret void
}

define void @VSETVLI.27(i64 "reg"="x" %rd, i64 "reg"="x" %rs1) "enc"="000011000111 rs1:5 111 rd:5 1010111" "asm"="vsetvli {rd}, {rs1}, e8, mf2, ta, ma" "asmonly" {
  ret void
}

define void @VSETVLI.28(i64 "reg"="x" %rd, i64 "reg"="x" %rs1) "enc"="000000001000 rs1:5 111 rd:5 1010111" "asm"="vsetvli {rd}, {rs1}, e16, m1, tu, mu" "asmonly" {
  ret void
}

define void @VSETVLI.29(i64 "reg"="x" %rd, i64 "reg"="x" %rs1) "enc"="000010001000 rs1:5 111 rd:5 1010111" "asm"="vsetvli {rd}, {rs1}, e16, m1, tu, ma" "asmonly" {
  ret void
}

define void @VSETVLI.30(i64 "reg"="x" %rd, i64 "reg"="x" %rs1) "enc"="000001001000 rs1:5 111 rd:5 1010111" "asm"="vsetvli {rd}, {rs1}, e16, m1, ta, mu" "asmonly" {
  ret void
}

define void @VSETVLI.31(i64 "reg"="x" %rd, i64 "reg"="x" %rs1) "enc"="000011001000 rs1:5 111 rd:5 1010111" "asm"="vsetvli {rd}, {rs1}, e16, m1, ta, ma" "asmonly" {
  ret void
}

define void @VSETVLI.32(i64 "reg"="x" %rd, i64 "reg"="x" %rs1) "enc"="000000001001 rs1:5 111 rd:5 1010111" "asm"="vsetvli {rd}, {rs1}, e16, m2, tu, mu" "asmonly" {
  ret void
}

define void @VSETVLI.33(i64 "reg"="x" %rd, i64 "reg"="x" %rs1) "enc"="000010001001 rs1:5 111 rd:5 1010111" "asm"="vsetvli {rd}, {rs1}, e16, m2, tu, ma" "asmonly" {
  ret void
}

define void @VSETVLI.34(i64 "reg"="x" %rd, i64 "reg"="x" %rs1) "enc"="000001001001 rs1:5 111 rd:5 1010111" "asm"="vsetvli {rd}, {rs1}, e16, m2, ta, mu" "asmonly" {
  ret void
}

define void @VSETVLI.35(i64 "reg"="x" %rd, i64 "reg"="x" %rs1) "enc"="000011001001 rs1:5 111 rd:5 1010111" "asm"="vsetvli {rd}, {rs1}, e16, m2, ta, ma" "asmonly" {
  ret void
}

define void @VSETVLI.36(i64 "reg"="x" %rd, i64 "reg"="x" %rs1) "enc"="000000001010 rs1:5 111 rd:5 1010111" "asm"="vsetvli {rd}, {rs1}, e16, m4, tu, mu" "asmonly" {
  ret void
}

define void @VSETVLI.37(i64 "reg"="x" %rd, i64 "reg"="x" %rs1) "enc"="000010001010 rs1:5 111 rd:5 1010111" "asm"="vsetvli {rd}, {rs1}, e16, m4, tu, ma" "asmonly" {
  ret void
}

define void @VSETVLI.38(i64 "reg"="x" %rd, i64 "reg"="x" %rs1) "enc"="000001001010 rs1:5 111 rd:5 1010111" "asm"="vsetvli {rd}, {rs1}, e16, m4, ta, mu" "asmonly" {
  ret void
}

define void @VSETVLI.39(i64 "reg"="x" %rd, i64 "reg"="x" %rs1) "enc"="000011001010 rs1:5 111 rd:5 1010111" "asm"="vsetvli {rd}, {rs1}, e16, m4, ta, ma" "asmonly" {
  ret void
}

define void @VSETVLI.40(i64 "reg"="x" %rd, i64 "reg"="x" %rs1) "enc"="000000001011 rs1:5 111 rd:5 1010111" "asm"="vsetvli {rd}, {rs1}, e16, m8, tu, mu" "asmonly" {
  ret void
}

define void @VSETVLI.41(i64 "reg"="x" %rd, i64 "reg"="x" %rs1) "enc"="000010001011 rs1:5 111 rd:5 1010111" "asm"="vsetvli {rd}, {rs1}, e16, m8, tu, ma" "asmonly" {
  ret void
}

define void @VSETVLI.42(i64 "reg"="x" %rd, i64 "reg"="x" %rs1) "enc"="000001001011 rs1:5 111 rd:5 1010111" "asm"="vsetvli {rd}, {rs1}, e16, m8, ta, mu" "asmonly" {
  ret void
}

define void @VSETVLI.43(i64 "reg"="x" %rd, i64 "reg"="x" %rs1) "enc"="000011001011 rs1:5 111 rd:5 1010111" "asm"="vsetvli {rd}, {rs1}, e16, m8, ta, ma" "asmonly" {
  ret void
}

define void @VSETVLI.44(i64 "reg"="x" %rd, i64 "reg"="x" %rs1) "enc"="000000001110 rs1:5 111 rd:5 1010111" "asm"="vsetvli {rd}, {rs1}, e16, mf4, tu, mu" "asmonly" {
  ret void
}

define void @VSETVLI.45(i64 "reg"="x" %rd, i64 "reg"="x" %rs1) "enc"="000010001110 rs1:5 111 rd:5 1010111" "asm"="vsetvli {rd}, {rs1}, e16, mf4, tu, ma" "asmonly" {
  ret void
}

define void @VSETVLI.46(i64 "reg"="x" %rd, i64 "reg"="x" %rs1) "enc"="000001001110 rs1:5 111 rd:5 1010111" "asm"="vsetvli {rd}, {rs1}, e16, mf4, ta, mu" "asmonly" {
  ret void
}

define void @VSETVLI.47(i64 "reg"="x" %rd, i64 "reg"="x" %rs1) "enc"="000011001110 rs1:5 111 rd:5 1010111" "asm"="vsetvli {rd}, {rs1}, e16, mf4, ta, ma" "asmonly" {
  ret void
}

define void @VSETVLI.48(i64 "reg"="x" %rd, i64 "reg"="x" %rs1) "enc"="000000001111 rs1:5 111 rd:5 1010111" "asm"="vsetvli {rd}, {rs1}, e16, mf2, tu, mu" "asmonly" {
  ret void
}

define void @VSETVLI.49(i64 "reg"="x" %rd, i64 "reg"="x" %rs1) "enc"="000010001111 rs1:5 111 rd:5 1010111" "asm"="vsetvli {rd}, {rs1}, e16, mf2, tu, ma" "asmonly" {
  ret void
}

define void @VSETVLI.50(i64 "reg"="x" %rd, i64 "reg"="x" %rs1) "enc"="000001001111 rs1:5 111 rd:5 1010111" "asm"="vsetvli {rd}, {rs1}, e16, mf2, ta, mu" "asmonly" {
  ret void
}

define void @VSETVLI.51(i64 "reg"="x" %rd, i64 "reg"="x" %rs1) "enc"="000011001111 rs1:5 111 rd:5 1010111" "asm"="vsetvli {rd}, {rs1}, e16, mf2, ta, ma" "asmonly" {
  ret void
}

define void @VSETVLI.52(i64 "reg"="x" %rd, i64 "reg"="x" %rs1) "enc"="000000010000 rs1:5 111 rd:5 1010111" "asm"="vsetvli {rd}, {rs1}, e32, m1, tu, mu" "asmonly" {
  ret void
}

define void @VSETVLI.53(i64 "reg"="x" %rd, i64 "reg"="x" %rs1) "enc"="000010010000 rs1:5 111 rd:5 1010111" "asm"="vsetvli {rd}, {rs1}, e32, m1, tu, ma" "asmonly" {
  ret void
}

define void @VSETVLI.54(i64 "reg"="x" %rd, i64 "reg"="x" %rs1) "enc"="000001010000 rs1:5 111 rd:5 1010111" "asm"="vsetvli {rd}, {rs1}, e32, m1, ta, mu" "asmonly" {
  ret void
}

define void @VSETVLI.55(i64 "reg"="x" %rd, i64 "reg"="x" %rs1) "enc"="000011010000 rs1:5 111 rd:5 1010111" "asm"="vsetvli {rd}, {rs1}, e32, m1, ta, ma" "asmonly" {
  ret void
}

define void @VSETVLI.56(i64 "reg"="x" %rd, i64 "reg"="x" %rs1) "enc"="000000010001 rs1:5 111 rd:5 1010111" "asm"="vsetvli {rd}, {rs1}, e32, m2, tu, mu" "asmonly" {
  ret void
}

define void @VSETVLI.57(i64 "reg"="x" %rd, i64 "reg"="x" %rs1) "enc"="000010010001 rs1:5 111 rd:5 1010111" "asm"="vsetvli {rd}, {rs1}, e32, m2, tu, ma" "asmonly" {
  ret void
}

define void @VSETVLI.58(i64 "reg"="x" %rd, i64 "reg"="x" %rs1) "enc"="000001010001 rs1:5 111 rd:5 1010111" "asm"="vsetvli {rd}, {rs1}, e32, m2, ta, mu" "asmonly" {
  ret void
}

define void @VSETVLI.59(i64 "reg"="x" %rd, i64 "reg"="x" %rs1) "enc"="000011010001 rs1:5 111 rd:5 1010111" "asm"="vsetvli {rd}, {rs1}, e32, m2, ta, ma" "asmonly" {
  ret void
}

define void @VSETVLI.60(i64 "reg"="x" %rd, i64 "reg"="x" %rs1) "enc"="000000010010 rs1:5 111 rd:5 1010111" "asm"="vsetvli {rd}, {rs1}, e32, m4, tu, mu" "asmonly" {
  ret void
}

define void @VSETVLI.61(i64 "reg"="x" %rd, i64 "reg"="x" %rs1) "enc"="000010010010 rs1:5 111 rd:5 1010111" "asm"="vsetvli {rd}, {rs1}, e32, m4, tu, ma" "asmonly" {
  ret void
}

define void @VSETVLI.62(i64 "reg"="x" %rd, i64 "reg"="x" %rs1) "enc"="000001010010 rs1:5 111 rd:5 1010111" "asm"="vsetvli {rd}, {rs1}, e32, m4, ta, mu" "asmonly" {
  ret void
}

define void @VSETVLI.63(i64 "reg"="x" %rd, i64 "reg"="x" %rs1) "enc"="000011010010 rs1:5 111 rd:5 1010111" "asm"="vsetvli {rd}, {rs1}, e32, m4, ta, ma" "asmonly" {
  ret void
}

define void @VSETVLI.64(i64 "reg"="x" %rd, i64 "reg"="x" %rs1) "enc"="000000010011 rs1:5 111 rd:5 1010111" "asm"="vsetvli {rd}, {rs1}, e32, m8, tu, mu" "asmonly" {
  ret void
}

define void @VSETVLI.65(i64 "reg"="x" %rd, i64 "reg"="x" %rs1) "enc"="000010010011 rs1:5 111 rd:5 1010111" "asm"="vsetvli {rd}, {rs1}, e32, m8, tu, ma" "asmonly" {
  ret void
}

define void @VSETVLI.66(i64 "reg"="x" %rd, i64 "reg"="x" %rs1) "enc"="000001010011 rs1:5 111 rd:5 1010111" "asm"="vsetvli {rd}, {rs1}, e32, m8, ta, mu" "asmonly" {
  ret void
}

define void @VSETVLI.67(i64 "reg"="x" %rd, i64 "reg"="x" %rs1) "enc"="000011010011 rs1:5 111 rd:5 1010111" "asm"="vsetvli {rd}, {rs1}, e32, m8, ta, ma" "asmonly" {
  ret void
}

define void @VSETVLI.68(i64 "reg"="x" %rd, i64 "reg"="x" %rs1) "enc"="000000010111 rs1:5 111 rd:5 1010111" "asm"="vsetvli {rd}, {rs1}, e32, mf2, tu, mu" "asmonly" {
  ret void
}

define void @VSETVLI.69(i64 "reg"="x" %rd, i64 "reg"="x" %rs1) "enc"="000010010111 rs1:5 111 rd:5 1010111" "asm"="vsetvli {rd}, {rs1}, e32, mf2, tu, ma" "asmonly" {
  ret void
}

define void @VSETVLI.70(i64 "reg"="x" %rd, i64 "reg"="x" %rs1) "enc"="000001010111 rs1:5 111 rd:5 1010111" "asm"="vsetvli {rd}, {rs1}, e32, mf2, ta, mu" "asmonly" {
  ret void
}

define void @VSETVLI.71(i64 "reg"="x" %rd, i64 "reg"="x" %rs1) "enc"="000011010111 rs1:5 111 rd:5 1010111" "asm"="vsetvli {rd}, {rs1}, e32, mf2, ta, ma" "asmonly" {
  ret void
}

define void @VSETVLI.72(i64 "reg"="x" %rd, i64 "reg"="x" %rs1) "enc"="000000011000 rs1:5 111 rd:5 1010111" "asm"="vsetvli {rd}, {rs1}, e64, m1, tu, mu" "asmonly" {
  ret void
}

define void @VSETVLI.73(i64 "reg"="x" %rd, i64 "reg"="x" %rs1) "enc"="000010011000 rs1:5 111 rd:5 1010111" "asm"="vsetvli {rd}, {rs1}, e64, m1, tu, ma" "asmonly" {
  ret void
}

define void @VSETVLI.74(i64 "reg"="x" %rd, i64 "reg"="x" %rs1) "enc"="000001011000 rs1:5 111 rd:5 1010111" "asm"="vsetvli {rd}, {rs1}, e64, m1, ta, mu" "asmonly" {
  ret void
}

define void @VSETVLI.75(i64 "reg"="x" %rd, i64 "reg"="x" %rs1) "enc"="000011011000 rs1:5 111 rd:5 1010111" "asm"="vsetvli {rd}, {rs1}, e64, m1, ta, ma" "asmonly" {
  ret void
}

define void @VSETVLI.76(i64 "reg"="x" %rd, i64 "reg"="x" %rs1) "enc"="000000011001 rs1:5 111 rd:5 1010111" "asm"="vsetvli {rd}, {rs1}, e64, m2, tu, mu" "asmonly" {
  ret void
}

define void @VSETVLI.77(i64 "reg"="x" %rd, i64 "reg"="x" %rs1) "enc"="000010011001 rs1:5 111 rd:5 1010111" "asm"="vsetvli {rd}, {rs1}, e64, m2, tu, ma" "asmonly" {
  ret void
}

define void @VSETVLI.78(i64 "reg"="x" %rd, i64 "reg"="x" %rs1) "enc"="000001011001 rs1:5 111 rd:5 1010111" "asm"="vsetvli {rd}, {rs1}, e64, m2, ta, mu" "asmonly" {
  ret void
}

define void @VSETVLI.79(i64 "reg"="x" %rd, i64 "reg"="x" %rs1) "enc"="000011011001 rs1:5 111 rd:5 1010111" "asm"="vsetvli {rd}, {rs1}, e64, m2, ta, ma" "asmonly" {
  ret void
}

define void @VSETVLI.80(i64 "reg"="x" %rd, i64 "reg"="x" %rs1) "enc"="000000011010 rs1:5 111 rd:5 1010111" "asm"="vsetvli {rd}, {rs1}, e64, m4, tu, mu" "asmonly" {
  ret void
}

define void @VSETVLI.81(i64 "reg"="x" %rd, i64 "reg"="x" %rs1) "enc"="000010011010 rs1:5 111 rd:5 1010111" "asm"="vsetvli {rd}, {rs1}, e64, m4, tu, ma" "asmonly" {
  ret void
}

define void @VSETVLI.82(i64 "reg"="x" %rd, i64 "reg"="x" %rs1) "enc"="000001011010 rs1:5 111 rd:5 1010111" "asm"="vsetvli {rd}, {rs1}, e64, m4, ta, mu" "asmonly" {
  ret void
}

define void @VSETVLI.83(i64 "reg"="x" %rd, i64 "reg"="x" %rs1) "enc"="000011011010 rs1:5 111 rd:5 1010111" "asm"="vsetvli {rd}, {rs1}, e64, m4, ta, ma" "asmonly" {
  ret void
}

define void @VSETVLI.84(i64 "reg"="x" %rd, i64 "reg"="x" %rs1) "enc"="000000011011 rs1:5 111 rd:5 1010111" "asm"="vsetvli {rd}, {rs1}, e64, m8, tu, mu" "asmonly" {
  ret void
}

define void @VSETVLI.85(i64 "reg"="x" %rd, i64 "reg"="x" %rs1) "enc"="000010011011 rs1:5 111 rd:5 1010111" "asm"="vsetvli {rd}, {rs1}, e64, m8, tu, ma" "asmonly" {
  ret void
}

define void @VSETVLI.86(i64 "reg"="x" %rd, i64 "reg"="x" %rs1) "enc"="000001011011 rs1:5 111 rd:5 1010111" "asm"="vsetvli {rd}, {rs1}, e64, m8, ta, mu" "asmonly" {
  ret void
}

define void @VSETVLI.87(i64 "reg"="x" %rd, i64 "reg"="x" %rs1) "enc"="000011011011 rs1:5 111 rd:5 1010111" "asm"="vsetvli {rd}, {rs1}, e64, m8, ta, ma" "asmonly" {
  ret void
}

define void @VSETVL.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1, i64 "reg"="x" %rs2) "enc"="1000000 rs2:5 rs1:5 111 rd:5 1010111" "asm"="vsetvl {rd}, {rs1}, {rs2}" "asmonly" {
  ret void
}

define void @VLM_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd) "enc"="000000101011 rs1:5 000 vd:5 0000111" "asm"="vlm.v {vd}, ({rs1})" "asmonly" {
  ret void
}

define void @VSM_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3) "enc"="000000101011 rs1:5 000 vs3:5 0100111" "asm"="vsm.v {vs3}, ({rs1})" "asmonly" {
  ret void
}

define void @VLE8_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd) "enc"="000000100000 rs1:5 000 vd:5 0000111" "asm"="vle8.v {vd}, ({rs1})" "asmonly" {
  ret void
}

define void @VLE8_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd) "enc"="{ if(vd!=0) 000000000000 rs1:5 000 vd:5 0000111 }" "asm"="vle8.v {vd}, ({rs1}), v0.t" "asmonly" {
  ret void
}

define void @VLSEG2E8_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd) "enc"="001000100000 rs1:5 000 vd:5 0000111" "asm"="vlseg2e8.v {vd}, ({rs1})" "asmonly" {
  ret void
}

define void @VLSEG2E8_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd) "enc"="{ if(vd!=0) 001000000000 rs1:5 000 vd:5 0000111 }" "asm"="vlseg2e8.v {vd}, ({rs1}), v0.t" "asmonly" {
  ret void
}

define void @VLSEG3E8_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd) "enc"="010000100000 rs1:5 000 vd:5 0000111" "asm"="vlseg3e8.v {vd}, ({rs1})" "asmonly" {
  ret void
}

define void @VLSEG3E8_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd) "enc"="{ if(vd!=0) 010000000000 rs1:5 000 vd:5 0000111 }" "asm"="vlseg3e8.v {vd}, ({rs1}), v0.t" "asmonly" {
  ret void
}

define void @VLSEG4E8_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd) "enc"="011000100000 rs1:5 000 vd:5 0000111" "asm"="vlseg4e8.v {vd}, ({rs1})" "asmonly" {
  ret void
}

define void @VLSEG4E8_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd) "enc"="{ if(vd!=0) 011000000000 rs1:5 000 vd:5 0000111 }" "asm"="vlseg4e8.v {vd}, ({rs1}), v0.t" "asmonly" {
  ret void
}

define void @VLSEG5E8_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd) "enc"="100000100000 rs1:5 000 vd:5 0000111" "asm"="vlseg5e8.v {vd}, ({rs1})" "asmonly" {
  ret void
}

define void @VLSEG5E8_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd) "enc"="{ if(vd!=0) 100000000000 rs1:5 000 vd:5 0000111 }" "asm"="vlseg5e8.v {vd}, ({rs1}), v0.t" "asmonly" {
  ret void
}

define void @VLSEG6E8_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd) "enc"="101000100000 rs1:5 000 vd:5 0000111" "asm"="vlseg6e8.v {vd}, ({rs1})" "asmonly" {
  ret void
}

define void @VLSEG6E8_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd) "enc"="{ if(vd!=0) 101000000000 rs1:5 000 vd:5 0000111 }" "asm"="vlseg6e8.v {vd}, ({rs1}), v0.t" "asmonly" {
  ret void
}

define void @VLSEG7E8_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd) "enc"="110000100000 rs1:5 000 vd:5 0000111" "asm"="vlseg7e8.v {vd}, ({rs1})" "asmonly" {
  ret void
}

define void @VLSEG7E8_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd) "enc"="{ if(vd!=0) 110000000000 rs1:5 000 vd:5 0000111 }" "asm"="vlseg7e8.v {vd}, ({rs1}), v0.t" "asmonly" {
  ret void
}

define void @VLSEG8E8_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd) "enc"="111000100000 rs1:5 000 vd:5 0000111" "asm"="vlseg8e8.v {vd}, ({rs1})" "asmonly" {
  ret void
}

define void @VLSEG8E8_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd) "enc"="{ if(vd!=0) 111000000000 rs1:5 000 vd:5 0000111 }" "asm"="vlseg8e8.v {vd}, ({rs1}), v0.t" "asmonly" {
  ret void
}

define void @VLE16_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd) "enc"="000000100000 rs1:5 101 vd:5 0000111" "asm"="vle16.v {vd}, ({rs1})" "asmonly" {
  ret void
}

define void @VLE16_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd) "enc"="{ if(vd!=0) 000000000000 rs1:5 101 vd:5 0000111 }" "asm"="vle16.v {vd}, ({rs1}), v0.t" "asmonly" {
  ret void
}

define void @VLSEG2E16_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd) "enc"="001000100000 rs1:5 101 vd:5 0000111" "asm"="vlseg2e16.v {vd}, ({rs1})" "asmonly" {
  ret void
}

define void @VLSEG2E16_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd) "enc"="{ if(vd!=0) 001000000000 rs1:5 101 vd:5 0000111 }" "asm"="vlseg2e16.v {vd}, ({rs1}), v0.t" "asmonly" {
  ret void
}

define void @VLSEG3E16_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd) "enc"="010000100000 rs1:5 101 vd:5 0000111" "asm"="vlseg3e16.v {vd}, ({rs1})" "asmonly" {
  ret void
}

define void @VLSEG3E16_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd) "enc"="{ if(vd!=0) 010000000000 rs1:5 101 vd:5 0000111 }" "asm"="vlseg3e16.v {vd}, ({rs1}), v0.t" "asmonly" {
  ret void
}

define void @VLSEG4E16_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd) "enc"="011000100000 rs1:5 101 vd:5 0000111" "asm"="vlseg4e16.v {vd}, ({rs1})" "asmonly" {
  ret void
}

define void @VLSEG4E16_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd) "enc"="{ if(vd!=0) 011000000000 rs1:5 101 vd:5 0000111 }" "asm"="vlseg4e16.v {vd}, ({rs1}), v0.t" "asmonly" {
  ret void
}

define void @VLSEG5E16_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd) "enc"="100000100000 rs1:5 101 vd:5 0000111" "asm"="vlseg5e16.v {vd}, ({rs1})" "asmonly" {
  ret void
}

define void @VLSEG5E16_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd) "enc"="{ if(vd!=0) 100000000000 rs1:5 101 vd:5 0000111 }" "asm"="vlseg5e16.v {vd}, ({rs1}), v0.t" "asmonly" {
  ret void
}

define void @VLSEG6E16_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd) "enc"="101000100000 rs1:5 101 vd:5 0000111" "asm"="vlseg6e16.v {vd}, ({rs1})" "asmonly" {
  ret void
}

define void @VLSEG6E16_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd) "enc"="{ if(vd!=0) 101000000000 rs1:5 101 vd:5 0000111 }" "asm"="vlseg6e16.v {vd}, ({rs1}), v0.t" "asmonly" {
  ret void
}

define void @VLSEG7E16_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd) "enc"="110000100000 rs1:5 101 vd:5 0000111" "asm"="vlseg7e16.v {vd}, ({rs1})" "asmonly" {
  ret void
}

define void @VLSEG7E16_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd) "enc"="{ if(vd!=0) 110000000000 rs1:5 101 vd:5 0000111 }" "asm"="vlseg7e16.v {vd}, ({rs1}), v0.t" "asmonly" {
  ret void
}

define void @VLSEG8E16_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd) "enc"="111000100000 rs1:5 101 vd:5 0000111" "asm"="vlseg8e16.v {vd}, ({rs1})" "asmonly" {
  ret void
}

define void @VLSEG8E16_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd) "enc"="{ if(vd!=0) 111000000000 rs1:5 101 vd:5 0000111 }" "asm"="vlseg8e16.v {vd}, ({rs1}), v0.t" "asmonly" {
  ret void
}

define void @VLE32_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd) "enc"="000000100000 rs1:5 110 vd:5 0000111" "asm"="vle32.v {vd}, ({rs1})" "asmonly" {
  ret void
}

define void @VLE32_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd) "enc"="{ if(vd!=0) 000000000000 rs1:5 110 vd:5 0000111 }" "asm"="vle32.v {vd}, ({rs1}), v0.t" "asmonly" {
  ret void
}

define void @VLSEG2E32_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd) "enc"="001000100000 rs1:5 110 vd:5 0000111" "asm"="vlseg2e32.v {vd}, ({rs1})" "asmonly" {
  ret void
}

define void @VLSEG2E32_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd) "enc"="{ if(vd!=0) 001000000000 rs1:5 110 vd:5 0000111 }" "asm"="vlseg2e32.v {vd}, ({rs1}), v0.t" "asmonly" {
  ret void
}

define void @VLSEG3E32_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd) "enc"="010000100000 rs1:5 110 vd:5 0000111" "asm"="vlseg3e32.v {vd}, ({rs1})" "asmonly" {
  ret void
}

define void @VLSEG3E32_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd) "enc"="{ if(vd!=0) 010000000000 rs1:5 110 vd:5 0000111 }" "asm"="vlseg3e32.v {vd}, ({rs1}), v0.t" "asmonly" {
  ret void
}

define void @VLSEG4E32_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd) "enc"="011000100000 rs1:5 110 vd:5 0000111" "asm"="vlseg4e32.v {vd}, ({rs1})" "asmonly" {
  ret void
}

define void @VLSEG4E32_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd) "enc"="{ if(vd!=0) 011000000000 rs1:5 110 vd:5 0000111 }" "asm"="vlseg4e32.v {vd}, ({rs1}), v0.t" "asmonly" {
  ret void
}

define void @VLSEG5E32_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd) "enc"="100000100000 rs1:5 110 vd:5 0000111" "asm"="vlseg5e32.v {vd}, ({rs1})" "asmonly" {
  ret void
}

define void @VLSEG5E32_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd) "enc"="{ if(vd!=0) 100000000000 rs1:5 110 vd:5 0000111 }" "asm"="vlseg5e32.v {vd}, ({rs1}), v0.t" "asmonly" {
  ret void
}

define void @VLSEG6E32_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd) "enc"="101000100000 rs1:5 110 vd:5 0000111" "asm"="vlseg6e32.v {vd}, ({rs1})" "asmonly" {
  ret void
}

define void @VLSEG6E32_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd) "enc"="{ if(vd!=0) 101000000000 rs1:5 110 vd:5 0000111 }" "asm"="vlseg6e32.v {vd}, ({rs1}), v0.t" "asmonly" {
  ret void
}

define void @VLSEG7E32_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd) "enc"="110000100000 rs1:5 110 vd:5 0000111" "asm"="vlseg7e32.v {vd}, ({rs1})" "asmonly" {
  ret void
}

define void @VLSEG7E32_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd) "enc"="{ if(vd!=0) 110000000000 rs1:5 110 vd:5 0000111 }" "asm"="vlseg7e32.v {vd}, ({rs1}), v0.t" "asmonly" {
  ret void
}

define void @VLSEG8E32_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd) "enc"="111000100000 rs1:5 110 vd:5 0000111" "asm"="vlseg8e32.v {vd}, ({rs1})" "asmonly" {
  ret void
}

define void @VLSEG8E32_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd) "enc"="{ if(vd!=0) 111000000000 rs1:5 110 vd:5 0000111 }" "asm"="vlseg8e32.v {vd}, ({rs1}), v0.t" "asmonly" {
  ret void
}

define void @VLE64_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd) "enc"="000000100000 rs1:5 111 vd:5 0000111" "asm"="vle64.v {vd}, ({rs1})" "asmonly" {
  ret void
}

define void @VLE64_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd) "enc"="{ if(vd!=0) 000000000000 rs1:5 111 vd:5 0000111 }" "asm"="vle64.v {vd}, ({rs1}), v0.t" "asmonly" {
  ret void
}

define void @VLSEG2E64_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd) "enc"="001000100000 rs1:5 111 vd:5 0000111" "asm"="vlseg2e64.v {vd}, ({rs1})" "asmonly" {
  ret void
}

define void @VLSEG2E64_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd) "enc"="{ if(vd!=0) 001000000000 rs1:5 111 vd:5 0000111 }" "asm"="vlseg2e64.v {vd}, ({rs1}), v0.t" "asmonly" {
  ret void
}

define void @VLSEG3E64_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd) "enc"="010000100000 rs1:5 111 vd:5 0000111" "asm"="vlseg3e64.v {vd}, ({rs1})" "asmonly" {
  ret void
}

define void @VLSEG3E64_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd) "enc"="{ if(vd!=0) 010000000000 rs1:5 111 vd:5 0000111 }" "asm"="vlseg3e64.v {vd}, ({rs1}), v0.t" "asmonly" {
  ret void
}

define void @VLSEG4E64_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd) "enc"="011000100000 rs1:5 111 vd:5 0000111" "asm"="vlseg4e64.v {vd}, ({rs1})" "asmonly" {
  ret void
}

define void @VLSEG4E64_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd) "enc"="{ if(vd!=0) 011000000000 rs1:5 111 vd:5 0000111 }" "asm"="vlseg4e64.v {vd}, ({rs1}), v0.t" "asmonly" {
  ret void
}

define void @VLSEG5E64_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd) "enc"="100000100000 rs1:5 111 vd:5 0000111" "asm"="vlseg5e64.v {vd}, ({rs1})" "asmonly" {
  ret void
}

define void @VLSEG5E64_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd) "enc"="{ if(vd!=0) 100000000000 rs1:5 111 vd:5 0000111 }" "asm"="vlseg5e64.v {vd}, ({rs1}), v0.t" "asmonly" {
  ret void
}

define void @VLSEG6E64_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd) "enc"="101000100000 rs1:5 111 vd:5 0000111" "asm"="vlseg6e64.v {vd}, ({rs1})" "asmonly" {
  ret void
}

define void @VLSEG6E64_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd) "enc"="{ if(vd!=0) 101000000000 rs1:5 111 vd:5 0000111 }" "asm"="vlseg6e64.v {vd}, ({rs1}), v0.t" "asmonly" {
  ret void
}

define void @VLSEG7E64_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd) "enc"="110000100000 rs1:5 111 vd:5 0000111" "asm"="vlseg7e64.v {vd}, ({rs1})" "asmonly" {
  ret void
}

define void @VLSEG7E64_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd) "enc"="{ if(vd!=0) 110000000000 rs1:5 111 vd:5 0000111 }" "asm"="vlseg7e64.v {vd}, ({rs1}), v0.t" "asmonly" {
  ret void
}

define void @VLSEG8E64_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd) "enc"="111000100000 rs1:5 111 vd:5 0000111" "asm"="vlseg8e64.v {vd}, ({rs1})" "asmonly" {
  ret void
}

define void @VLSEG8E64_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd) "enc"="{ if(vd!=0) 111000000000 rs1:5 111 vd:5 0000111 }" "asm"="vlseg8e64.v {vd}, ({rs1}), v0.t" "asmonly" {
  ret void
}

define void @VSE8_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3) "enc"="000000100000 rs1:5 000 vs3:5 0100111" "asm"="vse8.v {vs3}, ({rs1})" "asmonly" {
  ret void
}

define void @VSE8_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3) "enc"="000000000000 rs1:5 000 vs3:5 0100111" "asm"="vse8.v {vs3}, ({rs1}), v0.t" "asmonly" {
  ret void
}

define void @VSSEG2E8_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3) "enc"="001000100000 rs1:5 000 vs3:5 0100111" "asm"="vsseg2e8.v {vs3}, ({rs1})" "asmonly" {
  ret void
}

define void @VSSEG2E8_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3) "enc"="001000000000 rs1:5 000 vs3:5 0100111" "asm"="vsseg2e8.v {vs3}, ({rs1}), v0.t" "asmonly" {
  ret void
}

define void @VSSEG3E8_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3) "enc"="010000100000 rs1:5 000 vs3:5 0100111" "asm"="vsseg3e8.v {vs3}, ({rs1})" "asmonly" {
  ret void
}

define void @VSSEG3E8_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3) "enc"="010000000000 rs1:5 000 vs3:5 0100111" "asm"="vsseg3e8.v {vs3}, ({rs1}), v0.t" "asmonly" {
  ret void
}

define void @VSSEG4E8_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3) "enc"="011000100000 rs1:5 000 vs3:5 0100111" "asm"="vsseg4e8.v {vs3}, ({rs1})" "asmonly" {
  ret void
}

define void @VSSEG4E8_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3) "enc"="011000000000 rs1:5 000 vs3:5 0100111" "asm"="vsseg4e8.v {vs3}, ({rs1}), v0.t" "asmonly" {
  ret void
}

define void @VSSEG5E8_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3) "enc"="100000100000 rs1:5 000 vs3:5 0100111" "asm"="vsseg5e8.v {vs3}, ({rs1})" "asmonly" {
  ret void
}

define void @VSSEG5E8_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3) "enc"="100000000000 rs1:5 000 vs3:5 0100111" "asm"="vsseg5e8.v {vs3}, ({rs1}), v0.t" "asmonly" {
  ret void
}

define void @VSSEG6E8_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3) "enc"="101000100000 rs1:5 000 vs3:5 0100111" "asm"="vsseg6e8.v {vs3}, ({rs1})" "asmonly" {
  ret void
}

define void @VSSEG6E8_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3) "enc"="101000000000 rs1:5 000 vs3:5 0100111" "asm"="vsseg6e8.v {vs3}, ({rs1}), v0.t" "asmonly" {
  ret void
}

define void @VSSEG7E8_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3) "enc"="110000100000 rs1:5 000 vs3:5 0100111" "asm"="vsseg7e8.v {vs3}, ({rs1})" "asmonly" {
  ret void
}

define void @VSSEG7E8_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3) "enc"="110000000000 rs1:5 000 vs3:5 0100111" "asm"="vsseg7e8.v {vs3}, ({rs1}), v0.t" "asmonly" {
  ret void
}

define void @VSSEG8E8_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3) "enc"="111000100000 rs1:5 000 vs3:5 0100111" "asm"="vsseg8e8.v {vs3}, ({rs1})" "asmonly" {
  ret void
}

define void @VSSEG8E8_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3) "enc"="111000000000 rs1:5 000 vs3:5 0100111" "asm"="vsseg8e8.v {vs3}, ({rs1}), v0.t" "asmonly" {
  ret void
}

define void @VSE16_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3) "enc"="000000100000 rs1:5 101 vs3:5 0100111" "asm"="vse16.v {vs3}, ({rs1})" "asmonly" {
  ret void
}

define void @VSE16_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3) "enc"="000000000000 rs1:5 101 vs3:5 0100111" "asm"="vse16.v {vs3}, ({rs1}), v0.t" "asmonly" {
  ret void
}

define void @VSSEG2E16_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3) "enc"="001000100000 rs1:5 101 vs3:5 0100111" "asm"="vsseg2e16.v {vs3}, ({rs1})" "asmonly" {
  ret void
}

define void @VSSEG2E16_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3) "enc"="001000000000 rs1:5 101 vs3:5 0100111" "asm"="vsseg2e16.v {vs3}, ({rs1}), v0.t" "asmonly" {
  ret void
}

define void @VSSEG3E16_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3) "enc"="010000100000 rs1:5 101 vs3:5 0100111" "asm"="vsseg3e16.v {vs3}, ({rs1})" "asmonly" {
  ret void
}

define void @VSSEG3E16_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3) "enc"="010000000000 rs1:5 101 vs3:5 0100111" "asm"="vsseg3e16.v {vs3}, ({rs1}), v0.t" "asmonly" {
  ret void
}

define void @VSSEG4E16_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3) "enc"="011000100000 rs1:5 101 vs3:5 0100111" "asm"="vsseg4e16.v {vs3}, ({rs1})" "asmonly" {
  ret void
}

define void @VSSEG4E16_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3) "enc"="011000000000 rs1:5 101 vs3:5 0100111" "asm"="vsseg4e16.v {vs3}, ({rs1}), v0.t" "asmonly" {
  ret void
}

define void @VSSEG5E16_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3) "enc"="100000100000 rs1:5 101 vs3:5 0100111" "asm"="vsseg5e16.v {vs3}, ({rs1})" "asmonly" {
  ret void
}

define void @VSSEG5E16_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3) "enc"="100000000000 rs1:5 101 vs3:5 0100111" "asm"="vsseg5e16.v {vs3}, ({rs1}), v0.t" "asmonly" {
  ret void
}

define void @VSSEG6E16_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3) "enc"="101000100000 rs1:5 101 vs3:5 0100111" "asm"="vsseg6e16.v {vs3}, ({rs1})" "asmonly" {
  ret void
}

define void @VSSEG6E16_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3) "enc"="101000000000 rs1:5 101 vs3:5 0100111" "asm"="vsseg6e16.v {vs3}, ({rs1}), v0.t" "asmonly" {
  ret void
}

define void @VSSEG7E16_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3) "enc"="110000100000 rs1:5 101 vs3:5 0100111" "asm"="vsseg7e16.v {vs3}, ({rs1})" "asmonly" {
  ret void
}

define void @VSSEG7E16_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3) "enc"="110000000000 rs1:5 101 vs3:5 0100111" "asm"="vsseg7e16.v {vs3}, ({rs1}), v0.t" "asmonly" {
  ret void
}

define void @VSSEG8E16_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3) "enc"="111000100000 rs1:5 101 vs3:5 0100111" "asm"="vsseg8e16.v {vs3}, ({rs1})" "asmonly" {
  ret void
}

define void @VSSEG8E16_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3) "enc"="111000000000 rs1:5 101 vs3:5 0100111" "asm"="vsseg8e16.v {vs3}, ({rs1}), v0.t" "asmonly" {
  ret void
}

define void @VSE32_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3) "enc"="000000100000 rs1:5 110 vs3:5 0100111" "asm"="vse32.v {vs3}, ({rs1})" "asmonly" {
  ret void
}

define void @VSE32_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3) "enc"="000000000000 rs1:5 110 vs3:5 0100111" "asm"="vse32.v {vs3}, ({rs1}), v0.t" "asmonly" {
  ret void
}

define void @VSSEG2E32_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3) "enc"="001000100000 rs1:5 110 vs3:5 0100111" "asm"="vsseg2e32.v {vs3}, ({rs1})" "asmonly" {
  ret void
}

define void @VSSEG2E32_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3) "enc"="001000000000 rs1:5 110 vs3:5 0100111" "asm"="vsseg2e32.v {vs3}, ({rs1}), v0.t" "asmonly" {
  ret void
}

define void @VSSEG3E32_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3) "enc"="010000100000 rs1:5 110 vs3:5 0100111" "asm"="vsseg3e32.v {vs3}, ({rs1})" "asmonly" {
  ret void
}

define void @VSSEG3E32_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3) "enc"="010000000000 rs1:5 110 vs3:5 0100111" "asm"="vsseg3e32.v {vs3}, ({rs1}), v0.t" "asmonly" {
  ret void
}

define void @VSSEG4E32_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3) "enc"="011000100000 rs1:5 110 vs3:5 0100111" "asm"="vsseg4e32.v {vs3}, ({rs1})" "asmonly" {
  ret void
}

define void @VSSEG4E32_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3) "enc"="011000000000 rs1:5 110 vs3:5 0100111" "asm"="vsseg4e32.v {vs3}, ({rs1}), v0.t" "asmonly" {
  ret void
}

define void @VSSEG5E32_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3) "enc"="100000100000 rs1:5 110 vs3:5 0100111" "asm"="vsseg5e32.v {vs3}, ({rs1})" "asmonly" {
  ret void
}

define void @VSSEG5E32_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3) "enc"="100000000000 rs1:5 110 vs3:5 0100111" "asm"="vsseg5e32.v {vs3}, ({rs1}), v0.t" "asmonly" {
  ret void
}

define void @VSSEG6E32_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3) "enc"="101000100000 rs1:5 110 vs3:5 0100111" "asm"="vsseg6e32.v {vs3}, ({rs1})" "asmonly" {
  ret void
}

define void @VSSEG6E32_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3) "enc"="101000000000 rs1:5 110 vs3:5 0100111" "asm"="vsseg6e32.v {vs3}, ({rs1}), v0.t" "asmonly" {
  ret void
}

define void @VSSEG7E32_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3) "enc"="110000100000 rs1:5 110 vs3:5 0100111" "asm"="vsseg7e32.v {vs3}, ({rs1})" "asmonly" {
  ret void
}

define void @VSSEG7E32_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3) "enc"="110000000000 rs1:5 110 vs3:5 0100111" "asm"="vsseg7e32.v {vs3}, ({rs1}), v0.t" "asmonly" {
  ret void
}

define void @VSSEG8E32_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3) "enc"="111000100000 rs1:5 110 vs3:5 0100111" "asm"="vsseg8e32.v {vs3}, ({rs1})" "asmonly" {
  ret void
}

define void @VSSEG8E32_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3) "enc"="111000000000 rs1:5 110 vs3:5 0100111" "asm"="vsseg8e32.v {vs3}, ({rs1}), v0.t" "asmonly" {
  ret void
}

define void @VSE64_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3) "enc"="000000100000 rs1:5 111 vs3:5 0100111" "asm"="vse64.v {vs3}, ({rs1})" "asmonly" {
  ret void
}

define void @VSE64_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3) "enc"="000000000000 rs1:5 111 vs3:5 0100111" "asm"="vse64.v {vs3}, ({rs1}), v0.t" "asmonly" {
  ret void
}

define void @VSSEG2E64_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3) "enc"="001000100000 rs1:5 111 vs3:5 0100111" "asm"="vsseg2e64.v {vs3}, ({rs1})" "asmonly" {
  ret void
}

define void @VSSEG2E64_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3) "enc"="001000000000 rs1:5 111 vs3:5 0100111" "asm"="vsseg2e64.v {vs3}, ({rs1}), v0.t" "asmonly" {
  ret void
}

define void @VSSEG3E64_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3) "enc"="010000100000 rs1:5 111 vs3:5 0100111" "asm"="vsseg3e64.v {vs3}, ({rs1})" "asmonly" {
  ret void
}

define void @VSSEG3E64_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3) "enc"="010000000000 rs1:5 111 vs3:5 0100111" "asm"="vsseg3e64.v {vs3}, ({rs1}), v0.t" "asmonly" {
  ret void
}

define void @VSSEG4E64_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3) "enc"="011000100000 rs1:5 111 vs3:5 0100111" "asm"="vsseg4e64.v {vs3}, ({rs1})" "asmonly" {
  ret void
}

define void @VSSEG4E64_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3) "enc"="011000000000 rs1:5 111 vs3:5 0100111" "asm"="vsseg4e64.v {vs3}, ({rs1}), v0.t" "asmonly" {
  ret void
}

define void @VSSEG5E64_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3) "enc"="100000100000 rs1:5 111 vs3:5 0100111" "asm"="vsseg5e64.v {vs3}, ({rs1})" "asmonly" {
  ret void
}

define void @VSSEG5E64_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3) "enc"="100000000000 rs1:5 111 vs3:5 0100111" "asm"="vsseg5e64.v {vs3}, ({rs1}), v0.t" "asmonly" {
  ret void
}

define void @VSSEG6E64_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3) "enc"="101000100000 rs1:5 111 vs3:5 0100111" "asm"="vsseg6e64.v {vs3}, ({rs1})" "asmonly" {
  ret void
}

define void @VSSEG6E64_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3) "enc"="101000000000 rs1:5 111 vs3:5 0100111" "asm"="vsseg6e64.v {vs3}, ({rs1}), v0.t" "asmonly" {
  ret void
}

define void @VSSEG7E64_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3) "enc"="110000100000 rs1:5 111 vs3:5 0100111" "asm"="vsseg7e64.v {vs3}, ({rs1})" "asmonly" {
  ret void
}

define void @VSSEG7E64_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3) "enc"="110000000000 rs1:5 111 vs3:5 0100111" "asm"="vsseg7e64.v {vs3}, ({rs1}), v0.t" "asmonly" {
  ret void
}

define void @VSSEG8E64_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3) "enc"="111000100000 rs1:5 111 vs3:5 0100111" "asm"="vsseg8e64.v {vs3}, ({rs1})" "asmonly" {
  ret void
}

define void @VSSEG8E64_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3) "enc"="111000000000 rs1:5 111 vs3:5 0100111" "asm"="vsseg8e64.v {vs3}, ({rs1}), v0.t" "asmonly" {
  ret void
}

define void @VLUXEI8_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="0000011 vs2:5 rs1:5 000 vd:5 0000111" "asm"="vluxei8.v {vd}, ({rs1}), {vs2}" "asmonly" {
  ret void
}

define void @VLUXEI8_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if(vd!=0) 0000010 vs2:5 rs1:5 000 vd:5 0000111 }" "asm"="vluxei8.v {vd}, ({rs1}), {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VLUXSEG2EI8_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if((vs2<vd)|(vs2>=vd+2)) 0010011 vs2:5 rs1:5 000 vd:5 0000111 }" "asm"="vluxseg2ei8.v {vd}, ({rs1}), {vs2}" "asmonly" {
  ret void
}

define void @VLUXSEG2EI8_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if((vs2<vd)|(vs2>=vd+2)) if(vd!=0) 0010010 vs2:5 rs1:5 000 vd:5 0000111 }" "asm"="vluxseg2ei8.v {vd}, ({rs1}), {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VLUXSEG3EI8_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if((vs2<vd)|(vs2>=vd+4)) 0100011 vs2:5 rs1:5 000 vd:5 0000111 }" "asm"="vluxseg3ei8.v {vd}, ({rs1}), {vs2}" "asmonly" {
  ret void
}

define void @VLUXSEG3EI8_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if((vs2<vd)|(vs2>=vd+4)) if(vd!=0) 0100010 vs2:5 rs1:5 000 vd:5 0000111 }" "asm"="vluxseg3ei8.v {vd}, ({rs1}), {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VLUXSEG4EI8_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if((vs2<vd)|(vs2>=vd+4)) 0110011 vs2:5 rs1:5 000 vd:5 0000111 }" "asm"="vluxseg4ei8.v {vd}, ({rs1}), {vs2}" "asmonly" {
  ret void
}

define void @VLUXSEG4EI8_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if((vs2<vd)|(vs2>=vd+4)) if(vd!=0) 0110010 vs2:5 rs1:5 000 vd:5 0000111 }" "asm"="vluxseg4ei8.v {vd}, ({rs1}), {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VLUXSEG5EI8_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if((vs2<vd)|(vs2>=vd+6)) 1000011 vs2:5 rs1:5 000 vd:5 0000111 }" "asm"="vluxseg5ei8.v {vd}, ({rs1}), {vs2}" "asmonly" {
  ret void
}

define void @VLUXSEG5EI8_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if((vs2<vd)|(vs2>=vd+6)) if(vd!=0) 1000010 vs2:5 rs1:5 000 vd:5 0000111 }" "asm"="vluxseg5ei8.v {vd}, ({rs1}), {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VLUXSEG6EI8_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if((vs2<vd)|(vs2>=vd+6)) 1010011 vs2:5 rs1:5 000 vd:5 0000111 }" "asm"="vluxseg6ei8.v {vd}, ({rs1}), {vs2}" "asmonly" {
  ret void
}

define void @VLUXSEG6EI8_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if((vs2<vd)|(vs2>=vd+6)) if(vd!=0) 1010010 vs2:5 rs1:5 000 vd:5 0000111 }" "asm"="vluxseg6ei8.v {vd}, ({rs1}), {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VLUXSEG7EI8_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if((vs2<vd)|(vs2>=vd+7)) 1100011 vs2:5 rs1:5 000 vd:5 0000111 }" "asm"="vluxseg7ei8.v {vd}, ({rs1}), {vs2}" "asmonly" {
  ret void
}

define void @VLUXSEG7EI8_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if((vs2<vd)|(vs2>=vd+7)) if(vd!=0) 1100010 vs2:5 rs1:5 000 vd:5 0000111 }" "asm"="vluxseg7ei8.v {vd}, ({rs1}), {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VLUXSEG8EI8_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if((vs2<vd)|(vs2>=vd+8)) 1110011 vs2:5 rs1:5 000 vd:5 0000111 }" "asm"="vluxseg8ei8.v {vd}, ({rs1}), {vs2}" "asmonly" {
  ret void
}

define void @VLUXSEG8EI8_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if((vs2<vd)|(vs2>=vd+8)) if(vd!=0) 1110010 vs2:5 rs1:5 000 vd:5 0000111 }" "asm"="vluxseg8ei8.v {vd}, ({rs1}), {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VLUXEI16_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="0000011 vs2:5 rs1:5 101 vd:5 0000111" "asm"="vluxei16.v {vd}, ({rs1}), {vs2}" "asmonly" {
  ret void
}

define void @VLUXEI16_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if(vd!=0) 0000010 vs2:5 rs1:5 101 vd:5 0000111 }" "asm"="vluxei16.v {vd}, ({rs1}), {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VLUXSEG2EI16_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if((vs2<vd)|(vs2>=vd+2)) 0010011 vs2:5 rs1:5 101 vd:5 0000111 }" "asm"="vluxseg2ei16.v {vd}, ({rs1}), {vs2}" "asmonly" {
  ret void
}

define void @VLUXSEG2EI16_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if((vs2<vd)|(vs2>=vd+2)) if(vd!=0) 0010010 vs2:5 rs1:5 101 vd:5 0000111 }" "asm"="vluxseg2ei16.v {vd}, ({rs1}), {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VLUXSEG3EI16_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if((vs2<vd)|(vs2>=vd+4)) 0100011 vs2:5 rs1:5 101 vd:5 0000111 }" "asm"="vluxseg3ei16.v {vd}, ({rs1}), {vs2}" "asmonly" {
  ret void
}

define void @VLUXSEG3EI16_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if((vs2<vd)|(vs2>=vd+4)) if(vd!=0) 0100010 vs2:5 rs1:5 101 vd:5 0000111 }" "asm"="vluxseg3ei16.v {vd}, ({rs1}), {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VLUXSEG4EI16_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if((vs2<vd)|(vs2>=vd+4)) 0110011 vs2:5 rs1:5 101 vd:5 0000111 }" "asm"="vluxseg4ei16.v {vd}, ({rs1}), {vs2}" "asmonly" {
  ret void
}

define void @VLUXSEG4EI16_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if((vs2<vd)|(vs2>=vd+4)) if(vd!=0) 0110010 vs2:5 rs1:5 101 vd:5 0000111 }" "asm"="vluxseg4ei16.v {vd}, ({rs1}), {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VLUXSEG5EI16_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if((vs2<vd)|(vs2>=vd+6)) 1000011 vs2:5 rs1:5 101 vd:5 0000111 }" "asm"="vluxseg5ei16.v {vd}, ({rs1}), {vs2}" "asmonly" {
  ret void
}

define void @VLUXSEG5EI16_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if((vs2<vd)|(vs2>=vd+6)) if(vd!=0) 1000010 vs2:5 rs1:5 101 vd:5 0000111 }" "asm"="vluxseg5ei16.v {vd}, ({rs1}), {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VLUXSEG6EI16_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if((vs2<vd)|(vs2>=vd+6)) 1010011 vs2:5 rs1:5 101 vd:5 0000111 }" "asm"="vluxseg6ei16.v {vd}, ({rs1}), {vs2}" "asmonly" {
  ret void
}

define void @VLUXSEG6EI16_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if((vs2<vd)|(vs2>=vd+6)) if(vd!=0) 1010010 vs2:5 rs1:5 101 vd:5 0000111 }" "asm"="vluxseg6ei16.v {vd}, ({rs1}), {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VLUXSEG7EI16_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if((vs2<vd)|(vs2>=vd+7)) 1100011 vs2:5 rs1:5 101 vd:5 0000111 }" "asm"="vluxseg7ei16.v {vd}, ({rs1}), {vs2}" "asmonly" {
  ret void
}

define void @VLUXSEG7EI16_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if((vs2<vd)|(vs2>=vd+7)) if(vd!=0) 1100010 vs2:5 rs1:5 101 vd:5 0000111 }" "asm"="vluxseg7ei16.v {vd}, ({rs1}), {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VLUXSEG8EI16_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if((vs2<vd)|(vs2>=vd+8)) 1110011 vs2:5 rs1:5 101 vd:5 0000111 }" "asm"="vluxseg8ei16.v {vd}, ({rs1}), {vs2}" "asmonly" {
  ret void
}

define void @VLUXSEG8EI16_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if((vs2<vd)|(vs2>=vd+8)) if(vd!=0) 1110010 vs2:5 rs1:5 101 vd:5 0000111 }" "asm"="vluxseg8ei16.v {vd}, ({rs1}), {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VLUXEI32_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="0000011 vs2:5 rs1:5 110 vd:5 0000111" "asm"="vluxei32.v {vd}, ({rs1}), {vs2}" "asmonly" {
  ret void
}

define void @VLUXEI32_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if(vd!=0) 0000010 vs2:5 rs1:5 110 vd:5 0000111 }" "asm"="vluxei32.v {vd}, ({rs1}), {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VLUXSEG2EI32_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if((vs2<vd)|(vs2>=vd+2)) 0010011 vs2:5 rs1:5 110 vd:5 0000111 }" "asm"="vluxseg2ei32.v {vd}, ({rs1}), {vs2}" "asmonly" {
  ret void
}

define void @VLUXSEG2EI32_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if((vs2<vd)|(vs2>=vd+2)) if(vd!=0) 0010010 vs2:5 rs1:5 110 vd:5 0000111 }" "asm"="vluxseg2ei32.v {vd}, ({rs1}), {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VLUXSEG3EI32_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if((vs2<vd)|(vs2>=vd+4)) 0100011 vs2:5 rs1:5 110 vd:5 0000111 }" "asm"="vluxseg3ei32.v {vd}, ({rs1}), {vs2}" "asmonly" {
  ret void
}

define void @VLUXSEG3EI32_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if((vs2<vd)|(vs2>=vd+4)) if(vd!=0) 0100010 vs2:5 rs1:5 110 vd:5 0000111 }" "asm"="vluxseg3ei32.v {vd}, ({rs1}), {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VLUXSEG4EI32_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if((vs2<vd)|(vs2>=vd+4)) 0110011 vs2:5 rs1:5 110 vd:5 0000111 }" "asm"="vluxseg4ei32.v {vd}, ({rs1}), {vs2}" "asmonly" {
  ret void
}

define void @VLUXSEG4EI32_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if((vs2<vd)|(vs2>=vd+4)) if(vd!=0) 0110010 vs2:5 rs1:5 110 vd:5 0000111 }" "asm"="vluxseg4ei32.v {vd}, ({rs1}), {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VLUXSEG5EI32_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if((vs2<vd)|(vs2>=vd+6)) 1000011 vs2:5 rs1:5 110 vd:5 0000111 }" "asm"="vluxseg5ei32.v {vd}, ({rs1}), {vs2}" "asmonly" {
  ret void
}

define void @VLUXSEG5EI32_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if((vs2<vd)|(vs2>=vd+6)) if(vd!=0) 1000010 vs2:5 rs1:5 110 vd:5 0000111 }" "asm"="vluxseg5ei32.v {vd}, ({rs1}), {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VLUXSEG6EI32_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if((vs2<vd)|(vs2>=vd+6)) 1010011 vs2:5 rs1:5 110 vd:5 0000111 }" "asm"="vluxseg6ei32.v {vd}, ({rs1}), {vs2}" "asmonly" {
  ret void
}

define void @VLUXSEG6EI32_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if((vs2<vd)|(vs2>=vd+6)) if(vd!=0) 1010010 vs2:5 rs1:5 110 vd:5 0000111 }" "asm"="vluxseg6ei32.v {vd}, ({rs1}), {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VLUXSEG7EI32_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if((vs2<vd)|(vs2>=vd+7)) 1100011 vs2:5 rs1:5 110 vd:5 0000111 }" "asm"="vluxseg7ei32.v {vd}, ({rs1}), {vs2}" "asmonly" {
  ret void
}

define void @VLUXSEG7EI32_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if((vs2<vd)|(vs2>=vd+7)) if(vd!=0) 1100010 vs2:5 rs1:5 110 vd:5 0000111 }" "asm"="vluxseg7ei32.v {vd}, ({rs1}), {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VLUXSEG8EI32_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if((vs2<vd)|(vs2>=vd+8)) 1110011 vs2:5 rs1:5 110 vd:5 0000111 }" "asm"="vluxseg8ei32.v {vd}, ({rs1}), {vs2}" "asmonly" {
  ret void
}

define void @VLUXSEG8EI32_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if((vs2<vd)|(vs2>=vd+8)) if(vd!=0) 1110010 vs2:5 rs1:5 110 vd:5 0000111 }" "asm"="vluxseg8ei32.v {vd}, ({rs1}), {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VLUXEI64_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="0000011 vs2:5 rs1:5 111 vd:5 0000111" "asm"="vluxei64.v {vd}, ({rs1}), {vs2}" "asmonly" {
  ret void
}

define void @VLUXEI64_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if(vd!=0) 0000010 vs2:5 rs1:5 111 vd:5 0000111 }" "asm"="vluxei64.v {vd}, ({rs1}), {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VLUXSEG2EI64_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if((vs2<vd)|(vs2>=vd+2)) 0010011 vs2:5 rs1:5 111 vd:5 0000111 }" "asm"="vluxseg2ei64.v {vd}, ({rs1}), {vs2}" "asmonly" {
  ret void
}

define void @VLUXSEG2EI64_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if((vs2<vd)|(vs2>=vd+2)) if(vd!=0) 0010010 vs2:5 rs1:5 111 vd:5 0000111 }" "asm"="vluxseg2ei64.v {vd}, ({rs1}), {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VLUXSEG3EI64_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if((vs2<vd)|(vs2>=vd+4)) 0100011 vs2:5 rs1:5 111 vd:5 0000111 }" "asm"="vluxseg3ei64.v {vd}, ({rs1}), {vs2}" "asmonly" {
  ret void
}

define void @VLUXSEG3EI64_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if((vs2<vd)|(vs2>=vd+4)) if(vd!=0) 0100010 vs2:5 rs1:5 111 vd:5 0000111 }" "asm"="vluxseg3ei64.v {vd}, ({rs1}), {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VLUXSEG4EI64_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if((vs2<vd)|(vs2>=vd+4)) 0110011 vs2:5 rs1:5 111 vd:5 0000111 }" "asm"="vluxseg4ei64.v {vd}, ({rs1}), {vs2}" "asmonly" {
  ret void
}

define void @VLUXSEG4EI64_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if((vs2<vd)|(vs2>=vd+4)) if(vd!=0) 0110010 vs2:5 rs1:5 111 vd:5 0000111 }" "asm"="vluxseg4ei64.v {vd}, ({rs1}), {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VLUXSEG5EI64_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if((vs2<vd)|(vs2>=vd+6)) 1000011 vs2:5 rs1:5 111 vd:5 0000111 }" "asm"="vluxseg5ei64.v {vd}, ({rs1}), {vs2}" "asmonly" {
  ret void
}

define void @VLUXSEG5EI64_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if((vs2<vd)|(vs2>=vd+6)) if(vd!=0) 1000010 vs2:5 rs1:5 111 vd:5 0000111 }" "asm"="vluxseg5ei64.v {vd}, ({rs1}), {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VLUXSEG6EI64_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if((vs2<vd)|(vs2>=vd+6)) 1010011 vs2:5 rs1:5 111 vd:5 0000111 }" "asm"="vluxseg6ei64.v {vd}, ({rs1}), {vs2}" "asmonly" {
  ret void
}

define void @VLUXSEG6EI64_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if((vs2<vd)|(vs2>=vd+6)) if(vd!=0) 1010010 vs2:5 rs1:5 111 vd:5 0000111 }" "asm"="vluxseg6ei64.v {vd}, ({rs1}), {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VLUXSEG7EI64_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if((vs2<vd)|(vs2>=vd+7)) 1100011 vs2:5 rs1:5 111 vd:5 0000111 }" "asm"="vluxseg7ei64.v {vd}, ({rs1}), {vs2}" "asmonly" {
  ret void
}

define void @VLUXSEG7EI64_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if((vs2<vd)|(vs2>=vd+7)) if(vd!=0) 1100010 vs2:5 rs1:5 111 vd:5 0000111 }" "asm"="vluxseg7ei64.v {vd}, ({rs1}), {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VLUXSEG8EI64_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if((vs2<vd)|(vs2>=vd+8)) 1110011 vs2:5 rs1:5 111 vd:5 0000111 }" "asm"="vluxseg8ei64.v {vd}, ({rs1}), {vs2}" "asmonly" {
  ret void
}

define void @VLUXSEG8EI64_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if((vs2<vd)|(vs2>=vd+8)) if(vd!=0) 1110010 vs2:5 rs1:5 111 vd:5 0000111 }" "asm"="vluxseg8ei64.v {vd}, ({rs1}), {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VSUXEI8_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3, i64 "reg"="v" %vs2) "enc"="0000011 vs2:5 rs1:5 000 vs3:5 0100111" "asm"="vsuxei8.v {vs3}, ({rs1}), {vs2}" "asmonly" {
  ret void
}

define void @VSUXEI8_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3, i64 "reg"="v" %vs2) "enc"="0000010 vs2:5 rs1:5 000 vs3:5 0100111" "asm"="vsuxei8.v {vs3}, ({rs1}), {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VSUXSEG2EI8_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3, i64 "reg"="v" %vs2) "enc"="0010011 vs2:5 rs1:5 000 vs3:5 0100111" "asm"="vsuxseg2ei8.v {vs3}, ({rs1}), {vs2}" "asmonly" {
  ret void
}

define void @VSUXSEG2EI8_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3, i64 "reg"="v" %vs2) "enc"="0010010 vs2:5 rs1:5 000 vs3:5 0100111" "asm"="vsuxseg2ei8.v {vs3}, ({rs1}), {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VSUXSEG3EI8_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3, i64 "reg"="v" %vs2) "enc"="0100011 vs2:5 rs1:5 000 vs3:5 0100111" "asm"="vsuxseg3ei8.v {vs3}, ({rs1}), {vs2}" "asmonly" {
  ret void
}

define void @VSUXSEG3EI8_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3, i64 "reg"="v" %vs2) "enc"="0100010 vs2:5 rs1:5 000 vs3:5 0100111" "asm"="vsuxseg3ei8.v {vs3}, ({rs1}), {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VSUXSEG4EI8_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3, i64 "reg"="v" %vs2) "enc"="0110011 vs2:5 rs1:5 000 vs3:5 0100111" "asm"="vsuxseg4ei8.v {vs3}, ({rs1}), {vs2}" "asmonly" {
  ret void
}

define void @VSUXSEG4EI8_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3, i64 "reg"="v" %vs2) "enc"="0110010 vs2:5 rs1:5 000 vs3:5 0100111" "asm"="vsuxseg4ei8.v {vs3}, ({rs1}), {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VSUXSEG5EI8_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3, i64 "reg"="v" %vs2) "enc"="1000011 vs2:5 rs1:5 000 vs3:5 0100111" "asm"="vsuxseg5ei8.v {vs3}, ({rs1}), {vs2}" "asmonly" {
  ret void
}

define void @VSUXSEG5EI8_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3, i64 "reg"="v" %vs2) "enc"="1000010 vs2:5 rs1:5 000 vs3:5 0100111" "asm"="vsuxseg5ei8.v {vs3}, ({rs1}), {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VSUXSEG6EI8_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3, i64 "reg"="v" %vs2) "enc"="1010011 vs2:5 rs1:5 000 vs3:5 0100111" "asm"="vsuxseg6ei8.v {vs3}, ({rs1}), {vs2}" "asmonly" {
  ret void
}

define void @VSUXSEG6EI8_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3, i64 "reg"="v" %vs2) "enc"="1010010 vs2:5 rs1:5 000 vs3:5 0100111" "asm"="vsuxseg6ei8.v {vs3}, ({rs1}), {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VSUXSEG7EI8_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3, i64 "reg"="v" %vs2) "enc"="1100011 vs2:5 rs1:5 000 vs3:5 0100111" "asm"="vsuxseg7ei8.v {vs3}, ({rs1}), {vs2}" "asmonly" {
  ret void
}

define void @VSUXSEG7EI8_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3, i64 "reg"="v" %vs2) "enc"="1100010 vs2:5 rs1:5 000 vs3:5 0100111" "asm"="vsuxseg7ei8.v {vs3}, ({rs1}), {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VSUXSEG8EI8_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3, i64 "reg"="v" %vs2) "enc"="1110011 vs2:5 rs1:5 000 vs3:5 0100111" "asm"="vsuxseg8ei8.v {vs3}, ({rs1}), {vs2}" "asmonly" {
  ret void
}

define void @VSUXSEG8EI8_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3, i64 "reg"="v" %vs2) "enc"="1110010 vs2:5 rs1:5 000 vs3:5 0100111" "asm"="vsuxseg8ei8.v {vs3}, ({rs1}), {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VSUXEI16_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3, i64 "reg"="v" %vs2) "enc"="0000011 vs2:5 rs1:5 101 vs3:5 0100111" "asm"="vsuxei16.v {vs3}, ({rs1}), {vs2}" "asmonly" {
  ret void
}

define void @VSUXEI16_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3, i64 "reg"="v" %vs2) "enc"="0000010 vs2:5 rs1:5 101 vs3:5 0100111" "asm"="vsuxei16.v {vs3}, ({rs1}), {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VSUXSEG2EI16_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3, i64 "reg"="v" %vs2) "enc"="0010011 vs2:5 rs1:5 101 vs3:5 0100111" "asm"="vsuxseg2ei16.v {vs3}, ({rs1}), {vs2}" "asmonly" {
  ret void
}

define void @VSUXSEG2EI16_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3, i64 "reg"="v" %vs2) "enc"="0010010 vs2:5 rs1:5 101 vs3:5 0100111" "asm"="vsuxseg2ei16.v {vs3}, ({rs1}), {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VSUXSEG3EI16_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3, i64 "reg"="v" %vs2) "enc"="0100011 vs2:5 rs1:5 101 vs3:5 0100111" "asm"="vsuxseg3ei16.v {vs3}, ({rs1}), {vs2}" "asmonly" {
  ret void
}

define void @VSUXSEG3EI16_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3, i64 "reg"="v" %vs2) "enc"="0100010 vs2:5 rs1:5 101 vs3:5 0100111" "asm"="vsuxseg3ei16.v {vs3}, ({rs1}), {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VSUXSEG4EI16_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3, i64 "reg"="v" %vs2) "enc"="0110011 vs2:5 rs1:5 101 vs3:5 0100111" "asm"="vsuxseg4ei16.v {vs3}, ({rs1}), {vs2}" "asmonly" {
  ret void
}

define void @VSUXSEG4EI16_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3, i64 "reg"="v" %vs2) "enc"="0110010 vs2:5 rs1:5 101 vs3:5 0100111" "asm"="vsuxseg4ei16.v {vs3}, ({rs1}), {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VSUXSEG5EI16_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3, i64 "reg"="v" %vs2) "enc"="1000011 vs2:5 rs1:5 101 vs3:5 0100111" "asm"="vsuxseg5ei16.v {vs3}, ({rs1}), {vs2}" "asmonly" {
  ret void
}

define void @VSUXSEG5EI16_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3, i64 "reg"="v" %vs2) "enc"="1000010 vs2:5 rs1:5 101 vs3:5 0100111" "asm"="vsuxseg5ei16.v {vs3}, ({rs1}), {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VSUXSEG6EI16_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3, i64 "reg"="v" %vs2) "enc"="1010011 vs2:5 rs1:5 101 vs3:5 0100111" "asm"="vsuxseg6ei16.v {vs3}, ({rs1}), {vs2}" "asmonly" {
  ret void
}

define void @VSUXSEG6EI16_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3, i64 "reg"="v" %vs2) "enc"="1010010 vs2:5 rs1:5 101 vs3:5 0100111" "asm"="vsuxseg6ei16.v {vs3}, ({rs1}), {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VSUXSEG7EI16_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3, i64 "reg"="v" %vs2) "enc"="1100011 vs2:5 rs1:5 101 vs3:5 0100111" "asm"="vsuxseg7ei16.v {vs3}, ({rs1}), {vs2}" "asmonly" {
  ret void
}

define void @VSUXSEG7EI16_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3, i64 "reg"="v" %vs2) "enc"="1100010 vs2:5 rs1:5 101 vs3:5 0100111" "asm"="vsuxseg7ei16.v {vs3}, ({rs1}), {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VSUXSEG8EI16_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3, i64 "reg"="v" %vs2) "enc"="1110011 vs2:5 rs1:5 101 vs3:5 0100111" "asm"="vsuxseg8ei16.v {vs3}, ({rs1}), {vs2}" "asmonly" {
  ret void
}

define void @VSUXSEG8EI16_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3, i64 "reg"="v" %vs2) "enc"="1110010 vs2:5 rs1:5 101 vs3:5 0100111" "asm"="vsuxseg8ei16.v {vs3}, ({rs1}), {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VSUXEI32_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3, i64 "reg"="v" %vs2) "enc"="0000011 vs2:5 rs1:5 110 vs3:5 0100111" "asm"="vsuxei32.v {vs3}, ({rs1}), {vs2}" "asmonly" {
  ret void
}

define void @VSUXEI32_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3, i64 "reg"="v" %vs2) "enc"="0000010 vs2:5 rs1:5 110 vs3:5 0100111" "asm"="vsuxei32.v {vs3}, ({rs1}), {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VSUXSEG2EI32_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3, i64 "reg"="v" %vs2) "enc"="0010011 vs2:5 rs1:5 110 vs3:5 0100111" "asm"="vsuxseg2ei32.v {vs3}, ({rs1}), {vs2}" "asmonly" {
  ret void
}

define void @VSUXSEG2EI32_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3, i64 "reg"="v" %vs2) "enc"="0010010 vs2:5 rs1:5 110 vs3:5 0100111" "asm"="vsuxseg2ei32.v {vs3}, ({rs1}), {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VSUXSEG3EI32_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3, i64 "reg"="v" %vs2) "enc"="0100011 vs2:5 rs1:5 110 vs3:5 0100111" "asm"="vsuxseg3ei32.v {vs3}, ({rs1}), {vs2}" "asmonly" {
  ret void
}

define void @VSUXSEG3EI32_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3, i64 "reg"="v" %vs2) "enc"="0100010 vs2:5 rs1:5 110 vs3:5 0100111" "asm"="vsuxseg3ei32.v {vs3}, ({rs1}), {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VSUXSEG4EI32_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3, i64 "reg"="v" %vs2) "enc"="0110011 vs2:5 rs1:5 110 vs3:5 0100111" "asm"="vsuxseg4ei32.v {vs3}, ({rs1}), {vs2}" "asmonly" {
  ret void
}

define void @VSUXSEG4EI32_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3, i64 "reg"="v" %vs2) "enc"="0110010 vs2:5 rs1:5 110 vs3:5 0100111" "asm"="vsuxseg4ei32.v {vs3}, ({rs1}), {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VSUXSEG5EI32_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3, i64 "reg"="v" %vs2) "enc"="1000011 vs2:5 rs1:5 110 vs3:5 0100111" "asm"="vsuxseg5ei32.v {vs3}, ({rs1}), {vs2}" "asmonly" {
  ret void
}

define void @VSUXSEG5EI32_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3, i64 "reg"="v" %vs2) "enc"="1000010 vs2:5 rs1:5 110 vs3:5 0100111" "asm"="vsuxseg5ei32.v {vs3}, ({rs1}), {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VSUXSEG6EI32_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3, i64 "reg"="v" %vs2) "enc"="1010011 vs2:5 rs1:5 110 vs3:5 0100111" "asm"="vsuxseg6ei32.v {vs3}, ({rs1}), {vs2}" "asmonly" {
  ret void
}

define void @VSUXSEG6EI32_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3, i64 "reg"="v" %vs2) "enc"="1010010 vs2:5 rs1:5 110 vs3:5 0100111" "asm"="vsuxseg6ei32.v {vs3}, ({rs1}), {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VSUXSEG7EI32_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3, i64 "reg"="v" %vs2) "enc"="1100011 vs2:5 rs1:5 110 vs3:5 0100111" "asm"="vsuxseg7ei32.v {vs3}, ({rs1}), {vs2}" "asmonly" {
  ret void
}

define void @VSUXSEG7EI32_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3, i64 "reg"="v" %vs2) "enc"="1100010 vs2:5 rs1:5 110 vs3:5 0100111" "asm"="vsuxseg7ei32.v {vs3}, ({rs1}), {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VSUXSEG8EI32_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3, i64 "reg"="v" %vs2) "enc"="1110011 vs2:5 rs1:5 110 vs3:5 0100111" "asm"="vsuxseg8ei32.v {vs3}, ({rs1}), {vs2}" "asmonly" {
  ret void
}

define void @VSUXSEG8EI32_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3, i64 "reg"="v" %vs2) "enc"="1110010 vs2:5 rs1:5 110 vs3:5 0100111" "asm"="vsuxseg8ei32.v {vs3}, ({rs1}), {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VSUXEI64_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3, i64 "reg"="v" %vs2) "enc"="0000011 vs2:5 rs1:5 111 vs3:5 0100111" "asm"="vsuxei64.v {vs3}, ({rs1}), {vs2}" "asmonly" {
  ret void
}

define void @VSUXEI64_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3, i64 "reg"="v" %vs2) "enc"="0000010 vs2:5 rs1:5 111 vs3:5 0100111" "asm"="vsuxei64.v {vs3}, ({rs1}), {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VSUXSEG2EI64_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3, i64 "reg"="v" %vs2) "enc"="0010011 vs2:5 rs1:5 111 vs3:5 0100111" "asm"="vsuxseg2ei64.v {vs3}, ({rs1}), {vs2}" "asmonly" {
  ret void
}

define void @VSUXSEG2EI64_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3, i64 "reg"="v" %vs2) "enc"="0010010 vs2:5 rs1:5 111 vs3:5 0100111" "asm"="vsuxseg2ei64.v {vs3}, ({rs1}), {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VSUXSEG3EI64_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3, i64 "reg"="v" %vs2) "enc"="0100011 vs2:5 rs1:5 111 vs3:5 0100111" "asm"="vsuxseg3ei64.v {vs3}, ({rs1}), {vs2}" "asmonly" {
  ret void
}

define void @VSUXSEG3EI64_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3, i64 "reg"="v" %vs2) "enc"="0100010 vs2:5 rs1:5 111 vs3:5 0100111" "asm"="vsuxseg3ei64.v {vs3}, ({rs1}), {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VSUXSEG4EI64_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3, i64 "reg"="v" %vs2) "enc"="0110011 vs2:5 rs1:5 111 vs3:5 0100111" "asm"="vsuxseg4ei64.v {vs3}, ({rs1}), {vs2}" "asmonly" {
  ret void
}

define void @VSUXSEG4EI64_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3, i64 "reg"="v" %vs2) "enc"="0110010 vs2:5 rs1:5 111 vs3:5 0100111" "asm"="vsuxseg4ei64.v {vs3}, ({rs1}), {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VSUXSEG5EI64_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3, i64 "reg"="v" %vs2) "enc"="1000011 vs2:5 rs1:5 111 vs3:5 0100111" "asm"="vsuxseg5ei64.v {vs3}, ({rs1}), {vs2}" "asmonly" {
  ret void
}

define void @VSUXSEG5EI64_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3, i64 "reg"="v" %vs2) "enc"="1000010 vs2:5 rs1:5 111 vs3:5 0100111" "asm"="vsuxseg5ei64.v {vs3}, ({rs1}), {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VSUXSEG6EI64_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3, i64 "reg"="v" %vs2) "enc"="1010011 vs2:5 rs1:5 111 vs3:5 0100111" "asm"="vsuxseg6ei64.v {vs3}, ({rs1}), {vs2}" "asmonly" {
  ret void
}

define void @VSUXSEG6EI64_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3, i64 "reg"="v" %vs2) "enc"="1010010 vs2:5 rs1:5 111 vs3:5 0100111" "asm"="vsuxseg6ei64.v {vs3}, ({rs1}), {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VSUXSEG7EI64_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3, i64 "reg"="v" %vs2) "enc"="1100011 vs2:5 rs1:5 111 vs3:5 0100111" "asm"="vsuxseg7ei64.v {vs3}, ({rs1}), {vs2}" "asmonly" {
  ret void
}

define void @VSUXSEG7EI64_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3, i64 "reg"="v" %vs2) "enc"="1100010 vs2:5 rs1:5 111 vs3:5 0100111" "asm"="vsuxseg7ei64.v {vs3}, ({rs1}), {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VSUXSEG8EI64_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3, i64 "reg"="v" %vs2) "enc"="1110011 vs2:5 rs1:5 111 vs3:5 0100111" "asm"="vsuxseg8ei64.v {vs3}, ({rs1}), {vs2}" "asmonly" {
  ret void
}

define void @VSUXSEG8EI64_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3, i64 "reg"="v" %vs2) "enc"="1110010 vs2:5 rs1:5 111 vs3:5 0100111" "asm"="vsuxseg8ei64.v {vs3}, ({rs1}), {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VLSE8_V.0(i64 "reg"="x" %rs1, i64 "reg"="x" %rs2, i64 "reg"="v" %vd) "enc"="0000101 rs2:5 rs1:5 000 vd:5 0000111" "asm"="vlse8.v {vd}, ({rs1}), {rs2}" "asmonly" {
  ret void
}

define void @VLSE8_V.1(i64 "reg"="x" %rs1, i64 "reg"="x" %rs2, i64 "reg"="v" %vd) "enc"="{ if(vd!=0) 0000100 rs2:5 rs1:5 000 vd:5 0000111 }" "asm"="vlse8.v {vd}, ({rs1}), {rs2}, v0.t" "asmonly" {
  ret void
}

define void @VLSSEG2E8_V.0(i64 "reg"="x" %rs1, i64 "reg"="x" %rs2, i64 "reg"="v" %vd) "enc"="0010101 rs2:5 rs1:5 000 vd:5 0000111" "asm"="vlsseg2e8.v {vd}, ({rs1}), {rs2}" "asmonly" {
  ret void
}

define void @VLSSEG2E8_V.1(i64 "reg"="x" %rs1, i64 "reg"="x" %rs2, i64 "reg"="v" %vd) "enc"="{ if(vd!=0) 0010100 rs2:5 rs1:5 000 vd:5 0000111 }" "asm"="vlsseg2e8.v {vd}, ({rs1}), {rs2}, v0.t" "asmonly" {
  ret void
}

define void @VLSSEG3E8_V.0(i64 "reg"="x" %rs1, i64 "reg"="x" %rs2, i64 "reg"="v" %vd) "enc"="0100101 rs2:5 rs1:5 000 vd:5 0000111" "asm"="vlsseg3e8.v {vd}, ({rs1}), {rs2}" "asmonly" {
  ret void
}

define void @VLSSEG3E8_V.1(i64 "reg"="x" %rs1, i64 "reg"="x" %rs2, i64 "reg"="v" %vd) "enc"="{ if(vd!=0) 0100100 rs2:5 rs1:5 000 vd:5 0000111 }" "asm"="vlsseg3e8.v {vd}, ({rs1}), {rs2}, v0.t" "asmonly" {
  ret void
}

define void @VLSSEG4E8_V.0(i64 "reg"="x" %rs1, i64 "reg"="x" %rs2, i64 "reg"="v" %vd) "enc"="0110101 rs2:5 rs1:5 000 vd:5 0000111" "asm"="vlsseg4e8.v {vd}, ({rs1}), {rs2}" "asmonly" {
  ret void
}

define void @VLSSEG4E8_V.1(i64 "reg"="x" %rs1, i64 "reg"="x" %rs2, i64 "reg"="v" %vd) "enc"="{ if(vd!=0) 0110100 rs2:5 rs1:5 000 vd:5 0000111 }" "asm"="vlsseg4e8.v {vd}, ({rs1}), {rs2}, v0.t" "asmonly" {
  ret void
}

define void @VLSSEG5E8_V.0(i64 "reg"="x" %rs1, i64 "reg"="x" %rs2, i64 "reg"="v" %vd) "enc"="1000101 rs2:5 rs1:5 000 vd:5 0000111" "asm"="vlsseg5e8.v {vd}, ({rs1}), {rs2}" "asmonly" {
  ret void
}

define void @VLSSEG5E8_V.1(i64 "reg"="x" %rs1, i64 "reg"="x" %rs2, i64 "reg"="v" %vd) "enc"="{ if(vd!=0) 1000100 rs2:5 rs1:5 000 vd:5 0000111 }" "asm"="vlsseg5e8.v {vd}, ({rs1}), {rs2}, v0.t" "asmonly" {
  ret void
}

define void @VLSSEG6E8_V.0(i64 "reg"="x" %rs1, i64 "reg"="x" %rs2, i64 "reg"="v" %vd) "enc"="1010101 rs2:5 rs1:5 000 vd:5 0000111" "asm"="vlsseg6e8.v {vd}, ({rs1}), {rs2}" "asmonly" {
  ret void
}

define void @VLSSEG6E8_V.1(i64 "reg"="x" %rs1, i64 "reg"="x" %rs2, i64 "reg"="v" %vd) "enc"="{ if(vd!=0) 1010100 rs2:5 rs1:5 000 vd:5 0000111 }" "asm"="vlsseg6e8.v {vd}, ({rs1}), {rs2}, v0.t" "asmonly" {
  ret void
}

define void @VLSSEG7E8_V.0(i64 "reg"="x" %rs1, i64 "reg"="x" %rs2, i64 "reg"="v" %vd) "enc"="1100101 rs2:5 rs1:5 000 vd:5 0000111" "asm"="vlsseg7e8.v {vd}, ({rs1}), {rs2}" "asmonly" {
  ret void
}

define void @VLSSEG7E8_V.1(i64 "reg"="x" %rs1, i64 "reg"="x" %rs2, i64 "reg"="v" %vd) "enc"="{ if(vd!=0) 1100100 rs2:5 rs1:5 000 vd:5 0000111 }" "asm"="vlsseg7e8.v {vd}, ({rs1}), {rs2}, v0.t" "asmonly" {
  ret void
}

define void @VLSSEG8E8_V.0(i64 "reg"="x" %rs1, i64 "reg"="x" %rs2, i64 "reg"="v" %vd) "enc"="1110101 rs2:5 rs1:5 000 vd:5 0000111" "asm"="vlsseg8e8.v {vd}, ({rs1}), {rs2}" "asmonly" {
  ret void
}

define void @VLSSEG8E8_V.1(i64 "reg"="x" %rs1, i64 "reg"="x" %rs2, i64 "reg"="v" %vd) "enc"="{ if(vd!=0) 1110100 rs2:5 rs1:5 000 vd:5 0000111 }" "asm"="vlsseg8e8.v {vd}, ({rs1}), {rs2}, v0.t" "asmonly" {
  ret void
}

define void @VLSE16_V.0(i64 "reg"="x" %rs1, i64 "reg"="x" %rs2, i64 "reg"="v" %vd) "enc"="0000101 rs2:5 rs1:5 101 vd:5 0000111" "asm"="vlse16.v {vd}, ({rs1}), {rs2}" "asmonly" {
  ret void
}

define void @VLSE16_V.1(i64 "reg"="x" %rs1, i64 "reg"="x" %rs2, i64 "reg"="v" %vd) "enc"="{ if(vd!=0) 0000100 rs2:5 rs1:5 101 vd:5 0000111 }" "asm"="vlse16.v {vd}, ({rs1}), {rs2}, v0.t" "asmonly" {
  ret void
}

define void @VLSSEG2E16_V.0(i64 "reg"="x" %rs1, i64 "reg"="x" %rs2, i64 "reg"="v" %vd) "enc"="0010101 rs2:5 rs1:5 101 vd:5 0000111" "asm"="vlsseg2e16.v {vd}, ({rs1}), {rs2}" "asmonly" {
  ret void
}

define void @VLSSEG2E16_V.1(i64 "reg"="x" %rs1, i64 "reg"="x" %rs2, i64 "reg"="v" %vd) "enc"="{ if(vd!=0) 0010100 rs2:5 rs1:5 101 vd:5 0000111 }" "asm"="vlsseg2e16.v {vd}, ({rs1}), {rs2}, v0.t" "asmonly" {
  ret void
}

define void @VLSSEG3E16_V.0(i64 "reg"="x" %rs1, i64 "reg"="x" %rs2, i64 "reg"="v" %vd) "enc"="0100101 rs2:5 rs1:5 101 vd:5 0000111" "asm"="vlsseg3e16.v {vd}, ({rs1}), {rs2}" "asmonly" {
  ret void
}

define void @VLSSEG3E16_V.1(i64 "reg"="x" %rs1, i64 "reg"="x" %rs2, i64 "reg"="v" %vd) "enc"="{ if(vd!=0) 0100100 rs2:5 rs1:5 101 vd:5 0000111 }" "asm"="vlsseg3e16.v {vd}, ({rs1}), {rs2}, v0.t" "asmonly" {
  ret void
}

define void @VLSSEG4E16_V.0(i64 "reg"="x" %rs1, i64 "reg"="x" %rs2, i64 "reg"="v" %vd) "enc"="0110101 rs2:5 rs1:5 101 vd:5 0000111" "asm"="vlsseg4e16.v {vd}, ({rs1}), {rs2}" "asmonly" {
  ret void
}

define void @VLSSEG4E16_V.1(i64 "reg"="x" %rs1, i64 "reg"="x" %rs2, i64 "reg"="v" %vd) "enc"="{ if(vd!=0) 0110100 rs2:5 rs1:5 101 vd:5 0000111 }" "asm"="vlsseg4e16.v {vd}, ({rs1}), {rs2}, v0.t" "asmonly" {
  ret void
}

define void @VLSSEG5E16_V.0(i64 "reg"="x" %rs1, i64 "reg"="x" %rs2, i64 "reg"="v" %vd) "enc"="1000101 rs2:5 rs1:5 101 vd:5 0000111" "asm"="vlsseg5e16.v {vd}, ({rs1}), {rs2}" "asmonly" {
  ret void
}

define void @VLSSEG5E16_V.1(i64 "reg"="x" %rs1, i64 "reg"="x" %rs2, i64 "reg"="v" %vd) "enc"="{ if(vd!=0) 1000100 rs2:5 rs1:5 101 vd:5 0000111 }" "asm"="vlsseg5e16.v {vd}, ({rs1}), {rs2}, v0.t" "asmonly" {
  ret void
}

define void @VLSSEG6E16_V.0(i64 "reg"="x" %rs1, i64 "reg"="x" %rs2, i64 "reg"="v" %vd) "enc"="1010101 rs2:5 rs1:5 101 vd:5 0000111" "asm"="vlsseg6e16.v {vd}, ({rs1}), {rs2}" "asmonly" {
  ret void
}

define void @VLSSEG6E16_V.1(i64 "reg"="x" %rs1, i64 "reg"="x" %rs2, i64 "reg"="v" %vd) "enc"="{ if(vd!=0) 1010100 rs2:5 rs1:5 101 vd:5 0000111 }" "asm"="vlsseg6e16.v {vd}, ({rs1}), {rs2}, v0.t" "asmonly" {
  ret void
}

define void @VLSSEG7E16_V.0(i64 "reg"="x" %rs1, i64 "reg"="x" %rs2, i64 "reg"="v" %vd) "enc"="1100101 rs2:5 rs1:5 101 vd:5 0000111" "asm"="vlsseg7e16.v {vd}, ({rs1}), {rs2}" "asmonly" {
  ret void
}

define void @VLSSEG7E16_V.1(i64 "reg"="x" %rs1, i64 "reg"="x" %rs2, i64 "reg"="v" %vd) "enc"="{ if(vd!=0) 1100100 rs2:5 rs1:5 101 vd:5 0000111 }" "asm"="vlsseg7e16.v {vd}, ({rs1}), {rs2}, v0.t" "asmonly" {
  ret void
}

define void @VLSSEG8E16_V.0(i64 "reg"="x" %rs1, i64 "reg"="x" %rs2, i64 "reg"="v" %vd) "enc"="1110101 rs2:5 rs1:5 101 vd:5 0000111" "asm"="vlsseg8e16.v {vd}, ({rs1}), {rs2}" "asmonly" {
  ret void
}

define void @VLSSEG8E16_V.1(i64 "reg"="x" %rs1, i64 "reg"="x" %rs2, i64 "reg"="v" %vd) "enc"="{ if(vd!=0) 1110100 rs2:5 rs1:5 101 vd:5 0000111 }" "asm"="vlsseg8e16.v {vd}, ({rs1}), {rs2}, v0.t" "asmonly" {
  ret void
}

define void @VLSE32_V.0(i64 "reg"="x" %rs1, i64 "reg"="x" %rs2, i64 "reg"="v" %vd) "enc"="0000101 rs2:5 rs1:5 110 vd:5 0000111" "asm"="vlse32.v {vd}, ({rs1}), {rs2}" "asmonly" {
  ret void
}

define void @VLSE32_V.1(i64 "reg"="x" %rs1, i64 "reg"="x" %rs2, i64 "reg"="v" %vd) "enc"="{ if(vd!=0) 0000100 rs2:5 rs1:5 110 vd:5 0000111 }" "asm"="vlse32.v {vd}, ({rs1}), {rs2}, v0.t" "asmonly" {
  ret void
}

define void @VLSSEG2E32_V.0(i64 "reg"="x" %rs1, i64 "reg"="x" %rs2, i64 "reg"="v" %vd) "enc"="0010101 rs2:5 rs1:5 110 vd:5 0000111" "asm"="vlsseg2e32.v {vd}, ({rs1}), {rs2}" "asmonly" {
  ret void
}

define void @VLSSEG2E32_V.1(i64 "reg"="x" %rs1, i64 "reg"="x" %rs2, i64 "reg"="v" %vd) "enc"="{ if(vd!=0) 0010100 rs2:5 rs1:5 110 vd:5 0000111 }" "asm"="vlsseg2e32.v {vd}, ({rs1}), {rs2}, v0.t" "asmonly" {
  ret void
}

define void @VLSSEG3E32_V.0(i64 "reg"="x" %rs1, i64 "reg"="x" %rs2, i64 "reg"="v" %vd) "enc"="0100101 rs2:5 rs1:5 110 vd:5 0000111" "asm"="vlsseg3e32.v {vd}, ({rs1}), {rs2}" "asmonly" {
  ret void
}

define void @VLSSEG3E32_V.1(i64 "reg"="x" %rs1, i64 "reg"="x" %rs2, i64 "reg"="v" %vd) "enc"="{ if(vd!=0) 0100100 rs2:5 rs1:5 110 vd:5 0000111 }" "asm"="vlsseg3e32.v {vd}, ({rs1}), {rs2}, v0.t" "asmonly" {
  ret void
}

define void @VLSSEG4E32_V.0(i64 "reg"="x" %rs1, i64 "reg"="x" %rs2, i64 "reg"="v" %vd) "enc"="0110101 rs2:5 rs1:5 110 vd:5 0000111" "asm"="vlsseg4e32.v {vd}, ({rs1}), {rs2}" "asmonly" {
  ret void
}

define void @VLSSEG4E32_V.1(i64 "reg"="x" %rs1, i64 "reg"="x" %rs2, i64 "reg"="v" %vd) "enc"="{ if(vd!=0) 0110100 rs2:5 rs1:5 110 vd:5 0000111 }" "asm"="vlsseg4e32.v {vd}, ({rs1}), {rs2}, v0.t" "asmonly" {
  ret void
}

define void @VLSSEG5E32_V.0(i64 "reg"="x" %rs1, i64 "reg"="x" %rs2, i64 "reg"="v" %vd) "enc"="1000101 rs2:5 rs1:5 110 vd:5 0000111" "asm"="vlsseg5e32.v {vd}, ({rs1}), {rs2}" "asmonly" {
  ret void
}

define void @VLSSEG5E32_V.1(i64 "reg"="x" %rs1, i64 "reg"="x" %rs2, i64 "reg"="v" %vd) "enc"="{ if(vd!=0) 1000100 rs2:5 rs1:5 110 vd:5 0000111 }" "asm"="vlsseg5e32.v {vd}, ({rs1}), {rs2}, v0.t" "asmonly" {
  ret void
}

define void @VLSSEG6E32_V.0(i64 "reg"="x" %rs1, i64 "reg"="x" %rs2, i64 "reg"="v" %vd) "enc"="1010101 rs2:5 rs1:5 110 vd:5 0000111" "asm"="vlsseg6e32.v {vd}, ({rs1}), {rs2}" "asmonly" {
  ret void
}

define void @VLSSEG6E32_V.1(i64 "reg"="x" %rs1, i64 "reg"="x" %rs2, i64 "reg"="v" %vd) "enc"="{ if(vd!=0) 1010100 rs2:5 rs1:5 110 vd:5 0000111 }" "asm"="vlsseg6e32.v {vd}, ({rs1}), {rs2}, v0.t" "asmonly" {
  ret void
}

define void @VLSSEG7E32_V.0(i64 "reg"="x" %rs1, i64 "reg"="x" %rs2, i64 "reg"="v" %vd) "enc"="1100101 rs2:5 rs1:5 110 vd:5 0000111" "asm"="vlsseg7e32.v {vd}, ({rs1}), {rs2}" "asmonly" {
  ret void
}

define void @VLSSEG7E32_V.1(i64 "reg"="x" %rs1, i64 "reg"="x" %rs2, i64 "reg"="v" %vd) "enc"="{ if(vd!=0) 1100100 rs2:5 rs1:5 110 vd:5 0000111 }" "asm"="vlsseg7e32.v {vd}, ({rs1}), {rs2}, v0.t" "asmonly" {
  ret void
}

define void @VLSSEG8E32_V.0(i64 "reg"="x" %rs1, i64 "reg"="x" %rs2, i64 "reg"="v" %vd) "enc"="1110101 rs2:5 rs1:5 110 vd:5 0000111" "asm"="vlsseg8e32.v {vd}, ({rs1}), {rs2}" "asmonly" {
  ret void
}

define void @VLSSEG8E32_V.1(i64 "reg"="x" %rs1, i64 "reg"="x" %rs2, i64 "reg"="v" %vd) "enc"="{ if(vd!=0) 1110100 rs2:5 rs1:5 110 vd:5 0000111 }" "asm"="vlsseg8e32.v {vd}, ({rs1}), {rs2}, v0.t" "asmonly" {
  ret void
}

define void @VLSE64_V.0(i64 "reg"="x" %rs1, i64 "reg"="x" %rs2, i64 "reg"="v" %vd) "enc"="0000101 rs2:5 rs1:5 111 vd:5 0000111" "asm"="vlse64.v {vd}, ({rs1}), {rs2}" "asmonly" {
  ret void
}

define void @VLSE64_V.1(i64 "reg"="x" %rs1, i64 "reg"="x" %rs2, i64 "reg"="v" %vd) "enc"="{ if(vd!=0) 0000100 rs2:5 rs1:5 111 vd:5 0000111 }" "asm"="vlse64.v {vd}, ({rs1}), {rs2}, v0.t" "asmonly" {
  ret void
}

define void @VLSSEG2E64_V.0(i64 "reg"="x" %rs1, i64 "reg"="x" %rs2, i64 "reg"="v" %vd) "enc"="0010101 rs2:5 rs1:5 111 vd:5 0000111" "asm"="vlsseg2e64.v {vd}, ({rs1}), {rs2}" "asmonly" {
  ret void
}

define void @VLSSEG2E64_V.1(i64 "reg"="x" %rs1, i64 "reg"="x" %rs2, i64 "reg"="v" %vd) "enc"="{ if(vd!=0) 0010100 rs2:5 rs1:5 111 vd:5 0000111 }" "asm"="vlsseg2e64.v {vd}, ({rs1}), {rs2}, v0.t" "asmonly" {
  ret void
}

define void @VLSSEG3E64_V.0(i64 "reg"="x" %rs1, i64 "reg"="x" %rs2, i64 "reg"="v" %vd) "enc"="0100101 rs2:5 rs1:5 111 vd:5 0000111" "asm"="vlsseg3e64.v {vd}, ({rs1}), {rs2}" "asmonly" {
  ret void
}

define void @VLSSEG3E64_V.1(i64 "reg"="x" %rs1, i64 "reg"="x" %rs2, i64 "reg"="v" %vd) "enc"="{ if(vd!=0) 0100100 rs2:5 rs1:5 111 vd:5 0000111 }" "asm"="vlsseg3e64.v {vd}, ({rs1}), {rs2}, v0.t" "asmonly" {
  ret void
}

define void @VLSSEG4E64_V.0(i64 "reg"="x" %rs1, i64 "reg"="x" %rs2, i64 "reg"="v" %vd) "enc"="0110101 rs2:5 rs1:5 111 vd:5 0000111" "asm"="vlsseg4e64.v {vd}, ({rs1}), {rs2}" "asmonly" {
  ret void
}

define void @VLSSEG4E64_V.1(i64 "reg"="x" %rs1, i64 "reg"="x" %rs2, i64 "reg"="v" %vd) "enc"="{ if(vd!=0) 0110100 rs2:5 rs1:5 111 vd:5 0000111 }" "asm"="vlsseg4e64.v {vd}, ({rs1}), {rs2}, v0.t" "asmonly" {
  ret void
}

define void @VLSSEG5E64_V.0(i64 "reg"="x" %rs1, i64 "reg"="x" %rs2, i64 "reg"="v" %vd) "enc"="1000101 rs2:5 rs1:5 111 vd:5 0000111" "asm"="vlsseg5e64.v {vd}, ({rs1}), {rs2}" "asmonly" {
  ret void
}

define void @VLSSEG5E64_V.1(i64 "reg"="x" %rs1, i64 "reg"="x" %rs2, i64 "reg"="v" %vd) "enc"="{ if(vd!=0) 1000100 rs2:5 rs1:5 111 vd:5 0000111 }" "asm"="vlsseg5e64.v {vd}, ({rs1}), {rs2}, v0.t" "asmonly" {
  ret void
}

define void @VLSSEG6E64_V.0(i64 "reg"="x" %rs1, i64 "reg"="x" %rs2, i64 "reg"="v" %vd) "enc"="1010101 rs2:5 rs1:5 111 vd:5 0000111" "asm"="vlsseg6e64.v {vd}, ({rs1}), {rs2}" "asmonly" {
  ret void
}

define void @VLSSEG6E64_V.1(i64 "reg"="x" %rs1, i64 "reg"="x" %rs2, i64 "reg"="v" %vd) "enc"="{ if(vd!=0) 1010100 rs2:5 rs1:5 111 vd:5 0000111 }" "asm"="vlsseg6e64.v {vd}, ({rs1}), {rs2}, v0.t" "asmonly" {
  ret void
}

define void @VLSSEG7E64_V.0(i64 "reg"="x" %rs1, i64 "reg"="x" %rs2, i64 "reg"="v" %vd) "enc"="1100101 rs2:5 rs1:5 111 vd:5 0000111" "asm"="vlsseg7e64.v {vd}, ({rs1}), {rs2}" "asmonly" {
  ret void
}

define void @VLSSEG7E64_V.1(i64 "reg"="x" %rs1, i64 "reg"="x" %rs2, i64 "reg"="v" %vd) "enc"="{ if(vd!=0) 1100100 rs2:5 rs1:5 111 vd:5 0000111 }" "asm"="vlsseg7e64.v {vd}, ({rs1}), {rs2}, v0.t" "asmonly" {
  ret void
}

define void @VLSSEG8E64_V.0(i64 "reg"="x" %rs1, i64 "reg"="x" %rs2, i64 "reg"="v" %vd) "enc"="1110101 rs2:5 rs1:5 111 vd:5 0000111" "asm"="vlsseg8e64.v {vd}, ({rs1}), {rs2}" "asmonly" {
  ret void
}

define void @VLSSEG8E64_V.1(i64 "reg"="x" %rs1, i64 "reg"="x" %rs2, i64 "reg"="v" %vd) "enc"="{ if(vd!=0) 1110100 rs2:5 rs1:5 111 vd:5 0000111 }" "asm"="vlsseg8e64.v {vd}, ({rs1}), {rs2}, v0.t" "asmonly" {
  ret void
}

define void @VSSE8_V.0(i64 "reg"="x" %rs1, i64 "reg"="x" %rs2, i64 "reg"="v" %vs3) "enc"="0000101 rs2:5 rs1:5 000 vs3:5 0100111" "asm"="vsse8.v {vs3}, ({rs1}), {rs2}" "asmonly" {
  ret void
}

define void @VSSE8_V.1(i64 "reg"="x" %rs1, i64 "reg"="x" %rs2, i64 "reg"="v" %vs3) "enc"="0000100 rs2:5 rs1:5 000 vs3:5 0100111" "asm"="vsse8.v {vs3}, ({rs1}), {rs2}, v0.t" "asmonly" {
  ret void
}

define void @VSSSEG2E8_V.0(i64 "reg"="x" %rs1, i64 "reg"="x" %rs2, i64 "reg"="v" %vs3) "enc"="0010101 rs2:5 rs1:5 000 vs3:5 0100111" "asm"="vssseg2e8.v {vs3}, ({rs1}), {rs2}" "asmonly" {
  ret void
}

define void @VSSSEG2E8_V.1(i64 "reg"="x" %rs1, i64 "reg"="x" %rs2, i64 "reg"="v" %vs3) "enc"="0010100 rs2:5 rs1:5 000 vs3:5 0100111" "asm"="vssseg2e8.v {vs3}, ({rs1}), {rs2}, v0.t" "asmonly" {
  ret void
}

define void @VSSSEG3E8_V.0(i64 "reg"="x" %rs1, i64 "reg"="x" %rs2, i64 "reg"="v" %vs3) "enc"="0100101 rs2:5 rs1:5 000 vs3:5 0100111" "asm"="vssseg3e8.v {vs3}, ({rs1}), {rs2}" "asmonly" {
  ret void
}

define void @VSSSEG3E8_V.1(i64 "reg"="x" %rs1, i64 "reg"="x" %rs2, i64 "reg"="v" %vs3) "enc"="0100100 rs2:5 rs1:5 000 vs3:5 0100111" "asm"="vssseg3e8.v {vs3}, ({rs1}), {rs2}, v0.t" "asmonly" {
  ret void
}

define void @VSSSEG4E8_V.0(i64 "reg"="x" %rs1, i64 "reg"="x" %rs2, i64 "reg"="v" %vs3) "enc"="0110101 rs2:5 rs1:5 000 vs3:5 0100111" "asm"="vssseg4e8.v {vs3}, ({rs1}), {rs2}" "asmonly" {
  ret void
}

define void @VSSSEG4E8_V.1(i64 "reg"="x" %rs1, i64 "reg"="x" %rs2, i64 "reg"="v" %vs3) "enc"="0110100 rs2:5 rs1:5 000 vs3:5 0100111" "asm"="vssseg4e8.v {vs3}, ({rs1}), {rs2}, v0.t" "asmonly" {
  ret void
}

define void @VSSSEG5E8_V.0(i64 "reg"="x" %rs1, i64 "reg"="x" %rs2, i64 "reg"="v" %vs3) "enc"="1000101 rs2:5 rs1:5 000 vs3:5 0100111" "asm"="vssseg5e8.v {vs3}, ({rs1}), {rs2}" "asmonly" {
  ret void
}

define void @VSSSEG5E8_V.1(i64 "reg"="x" %rs1, i64 "reg"="x" %rs2, i64 "reg"="v" %vs3) "enc"="1000100 rs2:5 rs1:5 000 vs3:5 0100111" "asm"="vssseg5e8.v {vs3}, ({rs1}), {rs2}, v0.t" "asmonly" {
  ret void
}

define void @VSSSEG6E8_V.0(i64 "reg"="x" %rs1, i64 "reg"="x" %rs2, i64 "reg"="v" %vs3) "enc"="1010101 rs2:5 rs1:5 000 vs3:5 0100111" "asm"="vssseg6e8.v {vs3}, ({rs1}), {rs2}" "asmonly" {
  ret void
}

define void @VSSSEG6E8_V.1(i64 "reg"="x" %rs1, i64 "reg"="x" %rs2, i64 "reg"="v" %vs3) "enc"="1010100 rs2:5 rs1:5 000 vs3:5 0100111" "asm"="vssseg6e8.v {vs3}, ({rs1}), {rs2}, v0.t" "asmonly" {
  ret void
}

define void @VSSSEG7E8_V.0(i64 "reg"="x" %rs1, i64 "reg"="x" %rs2, i64 "reg"="v" %vs3) "enc"="1100101 rs2:5 rs1:5 000 vs3:5 0100111" "asm"="vssseg7e8.v {vs3}, ({rs1}), {rs2}" "asmonly" {
  ret void
}

define void @VSSSEG7E8_V.1(i64 "reg"="x" %rs1, i64 "reg"="x" %rs2, i64 "reg"="v" %vs3) "enc"="1100100 rs2:5 rs1:5 000 vs3:5 0100111" "asm"="vssseg7e8.v {vs3}, ({rs1}), {rs2}, v0.t" "asmonly" {
  ret void
}

define void @VSSSEG8E8_V.0(i64 "reg"="x" %rs1, i64 "reg"="x" %rs2, i64 "reg"="v" %vs3) "enc"="1110101 rs2:5 rs1:5 000 vs3:5 0100111" "asm"="vssseg8e8.v {vs3}, ({rs1}), {rs2}" "asmonly" {
  ret void
}

define void @VSSSEG8E8_V.1(i64 "reg"="x" %rs1, i64 "reg"="x" %rs2, i64 "reg"="v" %vs3) "enc"="1110100 rs2:5 rs1:5 000 vs3:5 0100111" "asm"="vssseg8e8.v {vs3}, ({rs1}), {rs2}, v0.t" "asmonly" {
  ret void
}

define void @VSSE16_V.0(i64 "reg"="x" %rs1, i64 "reg"="x" %rs2, i64 "reg"="v" %vs3) "enc"="0000101 rs2:5 rs1:5 101 vs3:5 0100111" "asm"="vsse16.v {vs3}, ({rs1}), {rs2}" "asmonly" {
  ret void
}

define void @VSSE16_V.1(i64 "reg"="x" %rs1, i64 "reg"="x" %rs2, i64 "reg"="v" %vs3) "enc"="0000100 rs2:5 rs1:5 101 vs3:5 0100111" "asm"="vsse16.v {vs3}, ({rs1}), {rs2}, v0.t" "asmonly" {
  ret void
}

define void @VSSSEG2E16_V.0(i64 "reg"="x" %rs1, i64 "reg"="x" %rs2, i64 "reg"="v" %vs3) "enc"="0010101 rs2:5 rs1:5 101 vs3:5 0100111" "asm"="vssseg2e16.v {vs3}, ({rs1}), {rs2}" "asmonly" {
  ret void
}

define void @VSSSEG2E16_V.1(i64 "reg"="x" %rs1, i64 "reg"="x" %rs2, i64 "reg"="v" %vs3) "enc"="0010100 rs2:5 rs1:5 101 vs3:5 0100111" "asm"="vssseg2e16.v {vs3}, ({rs1}), {rs2}, v0.t" "asmonly" {
  ret void
}

define void @VSSSEG3E16_V.0(i64 "reg"="x" %rs1, i64 "reg"="x" %rs2, i64 "reg"="v" %vs3) "enc"="0100101 rs2:5 rs1:5 101 vs3:5 0100111" "asm"="vssseg3e16.v {vs3}, ({rs1}), {rs2}" "asmonly" {
  ret void
}

define void @VSSSEG3E16_V.1(i64 "reg"="x" %rs1, i64 "reg"="x" %rs2, i64 "reg"="v" %vs3) "enc"="0100100 rs2:5 rs1:5 101 vs3:5 0100111" "asm"="vssseg3e16.v {vs3}, ({rs1}), {rs2}, v0.t" "asmonly" {
  ret void
}

define void @VSSSEG4E16_V.0(i64 "reg"="x" %rs1, i64 "reg"="x" %rs2, i64 "reg"="v" %vs3) "enc"="0110101 rs2:5 rs1:5 101 vs3:5 0100111" "asm"="vssseg4e16.v {vs3}, ({rs1}), {rs2}" "asmonly" {
  ret void
}

define void @VSSSEG4E16_V.1(i64 "reg"="x" %rs1, i64 "reg"="x" %rs2, i64 "reg"="v" %vs3) "enc"="0110100 rs2:5 rs1:5 101 vs3:5 0100111" "asm"="vssseg4e16.v {vs3}, ({rs1}), {rs2}, v0.t" "asmonly" {
  ret void
}

define void @VSSSEG5E16_V.0(i64 "reg"="x" %rs1, i64 "reg"="x" %rs2, i64 "reg"="v" %vs3) "enc"="1000101 rs2:5 rs1:5 101 vs3:5 0100111" "asm"="vssseg5e16.v {vs3}, ({rs1}), {rs2}" "asmonly" {
  ret void
}

define void @VSSSEG5E16_V.1(i64 "reg"="x" %rs1, i64 "reg"="x" %rs2, i64 "reg"="v" %vs3) "enc"="1000100 rs2:5 rs1:5 101 vs3:5 0100111" "asm"="vssseg5e16.v {vs3}, ({rs1}), {rs2}, v0.t" "asmonly" {
  ret void
}

define void @VSSSEG6E16_V.0(i64 "reg"="x" %rs1, i64 "reg"="x" %rs2, i64 "reg"="v" %vs3) "enc"="1010101 rs2:5 rs1:5 101 vs3:5 0100111" "asm"="vssseg6e16.v {vs3}, ({rs1}), {rs2}" "asmonly" {
  ret void
}

define void @VSSSEG6E16_V.1(i64 "reg"="x" %rs1, i64 "reg"="x" %rs2, i64 "reg"="v" %vs3) "enc"="1010100 rs2:5 rs1:5 101 vs3:5 0100111" "asm"="vssseg6e16.v {vs3}, ({rs1}), {rs2}, v0.t" "asmonly" {
  ret void
}

define void @VSSSEG7E16_V.0(i64 "reg"="x" %rs1, i64 "reg"="x" %rs2, i64 "reg"="v" %vs3) "enc"="1100101 rs2:5 rs1:5 101 vs3:5 0100111" "asm"="vssseg7e16.v {vs3}, ({rs1}), {rs2}" "asmonly" {
  ret void
}

define void @VSSSEG7E16_V.1(i64 "reg"="x" %rs1, i64 "reg"="x" %rs2, i64 "reg"="v" %vs3) "enc"="1100100 rs2:5 rs1:5 101 vs3:5 0100111" "asm"="vssseg7e16.v {vs3}, ({rs1}), {rs2}, v0.t" "asmonly" {
  ret void
}

define void @VSSSEG8E16_V.0(i64 "reg"="x" %rs1, i64 "reg"="x" %rs2, i64 "reg"="v" %vs3) "enc"="1110101 rs2:5 rs1:5 101 vs3:5 0100111" "asm"="vssseg8e16.v {vs3}, ({rs1}), {rs2}" "asmonly" {
  ret void
}

define void @VSSSEG8E16_V.1(i64 "reg"="x" %rs1, i64 "reg"="x" %rs2, i64 "reg"="v" %vs3) "enc"="1110100 rs2:5 rs1:5 101 vs3:5 0100111" "asm"="vssseg8e16.v {vs3}, ({rs1}), {rs2}, v0.t" "asmonly" {
  ret void
}

define void @VSSE32_V.0(i64 "reg"="x" %rs1, i64 "reg"="x" %rs2, i64 "reg"="v" %vs3) "enc"="0000101 rs2:5 rs1:5 110 vs3:5 0100111" "asm"="vsse32.v {vs3}, ({rs1}), {rs2}" "asmonly" {
  ret void
}

define void @VSSE32_V.1(i64 "reg"="x" %rs1, i64 "reg"="x" %rs2, i64 "reg"="v" %vs3) "enc"="0000100 rs2:5 rs1:5 110 vs3:5 0100111" "asm"="vsse32.v {vs3}, ({rs1}), {rs2}, v0.t" "asmonly" {
  ret void
}

define void @VSSSEG2E32_V.0(i64 "reg"="x" %rs1, i64 "reg"="x" %rs2, i64 "reg"="v" %vs3) "enc"="0010101 rs2:5 rs1:5 110 vs3:5 0100111" "asm"="vssseg2e32.v {vs3}, ({rs1}), {rs2}" "asmonly" {
  ret void
}

define void @VSSSEG2E32_V.1(i64 "reg"="x" %rs1, i64 "reg"="x" %rs2, i64 "reg"="v" %vs3) "enc"="0010100 rs2:5 rs1:5 110 vs3:5 0100111" "asm"="vssseg2e32.v {vs3}, ({rs1}), {rs2}, v0.t" "asmonly" {
  ret void
}

define void @VSSSEG3E32_V.0(i64 "reg"="x" %rs1, i64 "reg"="x" %rs2, i64 "reg"="v" %vs3) "enc"="0100101 rs2:5 rs1:5 110 vs3:5 0100111" "asm"="vssseg3e32.v {vs3}, ({rs1}), {rs2}" "asmonly" {
  ret void
}

define void @VSSSEG3E32_V.1(i64 "reg"="x" %rs1, i64 "reg"="x" %rs2, i64 "reg"="v" %vs3) "enc"="0100100 rs2:5 rs1:5 110 vs3:5 0100111" "asm"="vssseg3e32.v {vs3}, ({rs1}), {rs2}, v0.t" "asmonly" {
  ret void
}

define void @VSSSEG4E32_V.0(i64 "reg"="x" %rs1, i64 "reg"="x" %rs2, i64 "reg"="v" %vs3) "enc"="0110101 rs2:5 rs1:5 110 vs3:5 0100111" "asm"="vssseg4e32.v {vs3}, ({rs1}), {rs2}" "asmonly" {
  ret void
}

define void @VSSSEG4E32_V.1(i64 "reg"="x" %rs1, i64 "reg"="x" %rs2, i64 "reg"="v" %vs3) "enc"="0110100 rs2:5 rs1:5 110 vs3:5 0100111" "asm"="vssseg4e32.v {vs3}, ({rs1}), {rs2}, v0.t" "asmonly" {
  ret void
}

define void @VSSSEG5E32_V.0(i64 "reg"="x" %rs1, i64 "reg"="x" %rs2, i64 "reg"="v" %vs3) "enc"="1000101 rs2:5 rs1:5 110 vs3:5 0100111" "asm"="vssseg5e32.v {vs3}, ({rs1}), {rs2}" "asmonly" {
  ret void
}

define void @VSSSEG5E32_V.1(i64 "reg"="x" %rs1, i64 "reg"="x" %rs2, i64 "reg"="v" %vs3) "enc"="1000100 rs2:5 rs1:5 110 vs3:5 0100111" "asm"="vssseg5e32.v {vs3}, ({rs1}), {rs2}, v0.t" "asmonly" {
  ret void
}

define void @VSSSEG6E32_V.0(i64 "reg"="x" %rs1, i64 "reg"="x" %rs2, i64 "reg"="v" %vs3) "enc"="1010101 rs2:5 rs1:5 110 vs3:5 0100111" "asm"="vssseg6e32.v {vs3}, ({rs1}), {rs2}" "asmonly" {
  ret void
}

define void @VSSSEG6E32_V.1(i64 "reg"="x" %rs1, i64 "reg"="x" %rs2, i64 "reg"="v" %vs3) "enc"="1010100 rs2:5 rs1:5 110 vs3:5 0100111" "asm"="vssseg6e32.v {vs3}, ({rs1}), {rs2}, v0.t" "asmonly" {
  ret void
}

define void @VSSSEG7E32_V.0(i64 "reg"="x" %rs1, i64 "reg"="x" %rs2, i64 "reg"="v" %vs3) "enc"="1100101 rs2:5 rs1:5 110 vs3:5 0100111" "asm"="vssseg7e32.v {vs3}, ({rs1}), {rs2}" "asmonly" {
  ret void
}

define void @VSSSEG7E32_V.1(i64 "reg"="x" %rs1, i64 "reg"="x" %rs2, i64 "reg"="v" %vs3) "enc"="1100100 rs2:5 rs1:5 110 vs3:5 0100111" "asm"="vssseg7e32.v {vs3}, ({rs1}), {rs2}, v0.t" "asmonly" {
  ret void
}

define void @VSSSEG8E32_V.0(i64 "reg"="x" %rs1, i64 "reg"="x" %rs2, i64 "reg"="v" %vs3) "enc"="1110101 rs2:5 rs1:5 110 vs3:5 0100111" "asm"="vssseg8e32.v {vs3}, ({rs1}), {rs2}" "asmonly" {
  ret void
}

define void @VSSSEG8E32_V.1(i64 "reg"="x" %rs1, i64 "reg"="x" %rs2, i64 "reg"="v" %vs3) "enc"="1110100 rs2:5 rs1:5 110 vs3:5 0100111" "asm"="vssseg8e32.v {vs3}, ({rs1}), {rs2}, v0.t" "asmonly" {
  ret void
}

define void @VSSE64_V.0(i64 "reg"="x" %rs1, i64 "reg"="x" %rs2, i64 "reg"="v" %vs3) "enc"="0000101 rs2:5 rs1:5 111 vs3:5 0100111" "asm"="vsse64.v {vs3}, ({rs1}), {rs2}" "asmonly" {
  ret void
}

define void @VSSE64_V.1(i64 "reg"="x" %rs1, i64 "reg"="x" %rs2, i64 "reg"="v" %vs3) "enc"="0000100 rs2:5 rs1:5 111 vs3:5 0100111" "asm"="vsse64.v {vs3}, ({rs1}), {rs2}, v0.t" "asmonly" {
  ret void
}

define void @VSSSEG2E64_V.0(i64 "reg"="x" %rs1, i64 "reg"="x" %rs2, i64 "reg"="v" %vs3) "enc"="0010101 rs2:5 rs1:5 111 vs3:5 0100111" "asm"="vssseg2e64.v {vs3}, ({rs1}), {rs2}" "asmonly" {
  ret void
}

define void @VSSSEG2E64_V.1(i64 "reg"="x" %rs1, i64 "reg"="x" %rs2, i64 "reg"="v" %vs3) "enc"="0010100 rs2:5 rs1:5 111 vs3:5 0100111" "asm"="vssseg2e64.v {vs3}, ({rs1}), {rs2}, v0.t" "asmonly" {
  ret void
}

define void @VSSSEG3E64_V.0(i64 "reg"="x" %rs1, i64 "reg"="x" %rs2, i64 "reg"="v" %vs3) "enc"="0100101 rs2:5 rs1:5 111 vs3:5 0100111" "asm"="vssseg3e64.v {vs3}, ({rs1}), {rs2}" "asmonly" {
  ret void
}

define void @VSSSEG3E64_V.1(i64 "reg"="x" %rs1, i64 "reg"="x" %rs2, i64 "reg"="v" %vs3) "enc"="0100100 rs2:5 rs1:5 111 vs3:5 0100111" "asm"="vssseg3e64.v {vs3}, ({rs1}), {rs2}, v0.t" "asmonly" {
  ret void
}

define void @VSSSEG4E64_V.0(i64 "reg"="x" %rs1, i64 "reg"="x" %rs2, i64 "reg"="v" %vs3) "enc"="0110101 rs2:5 rs1:5 111 vs3:5 0100111" "asm"="vssseg4e64.v {vs3}, ({rs1}), {rs2}" "asmonly" {
  ret void
}

define void @VSSSEG4E64_V.1(i64 "reg"="x" %rs1, i64 "reg"="x" %rs2, i64 "reg"="v" %vs3) "enc"="0110100 rs2:5 rs1:5 111 vs3:5 0100111" "asm"="vssseg4e64.v {vs3}, ({rs1}), {rs2}, v0.t" "asmonly" {
  ret void
}

define void @VSSSEG5E64_V.0(i64 "reg"="x" %rs1, i64 "reg"="x" %rs2, i64 "reg"="v" %vs3) "enc"="1000101 rs2:5 rs1:5 111 vs3:5 0100111" "asm"="vssseg5e64.v {vs3}, ({rs1}), {rs2}" "asmonly" {
  ret void
}

define void @VSSSEG5E64_V.1(i64 "reg"="x" %rs1, i64 "reg"="x" %rs2, i64 "reg"="v" %vs3) "enc"="1000100 rs2:5 rs1:5 111 vs3:5 0100111" "asm"="vssseg5e64.v {vs3}, ({rs1}), {rs2}, v0.t" "asmonly" {
  ret void
}

define void @VSSSEG6E64_V.0(i64 "reg"="x" %rs1, i64 "reg"="x" %rs2, i64 "reg"="v" %vs3) "enc"="1010101 rs2:5 rs1:5 111 vs3:5 0100111" "asm"="vssseg6e64.v {vs3}, ({rs1}), {rs2}" "asmonly" {
  ret void
}

define void @VSSSEG6E64_V.1(i64 "reg"="x" %rs1, i64 "reg"="x" %rs2, i64 "reg"="v" %vs3) "enc"="1010100 rs2:5 rs1:5 111 vs3:5 0100111" "asm"="vssseg6e64.v {vs3}, ({rs1}), {rs2}, v0.t" "asmonly" {
  ret void
}

define void @VSSSEG7E64_V.0(i64 "reg"="x" %rs1, i64 "reg"="x" %rs2, i64 "reg"="v" %vs3) "enc"="1100101 rs2:5 rs1:5 111 vs3:5 0100111" "asm"="vssseg7e64.v {vs3}, ({rs1}), {rs2}" "asmonly" {
  ret void
}

define void @VSSSEG7E64_V.1(i64 "reg"="x" %rs1, i64 "reg"="x" %rs2, i64 "reg"="v" %vs3) "enc"="1100100 rs2:5 rs1:5 111 vs3:5 0100111" "asm"="vssseg7e64.v {vs3}, ({rs1}), {rs2}, v0.t" "asmonly" {
  ret void
}

define void @VSSSEG8E64_V.0(i64 "reg"="x" %rs1, i64 "reg"="x" %rs2, i64 "reg"="v" %vs3) "enc"="1110101 rs2:5 rs1:5 111 vs3:5 0100111" "asm"="vssseg8e64.v {vs3}, ({rs1}), {rs2}" "asmonly" {
  ret void
}

define void @VSSSEG8E64_V.1(i64 "reg"="x" %rs1, i64 "reg"="x" %rs2, i64 "reg"="v" %vs3) "enc"="1110100 rs2:5 rs1:5 111 vs3:5 0100111" "asm"="vssseg8e64.v {vs3}, ({rs1}), {rs2}, v0.t" "asmonly" {
  ret void
}

define void @VLOXEI8_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="0000111 vs2:5 rs1:5 000 vd:5 0000111" "asm"="vloxei8.v {vd}, ({rs1}), {vs2}" "asmonly" {
  ret void
}

define void @VLOXEI8_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if(vd!=0) 0000110 vs2:5 rs1:5 000 vd:5 0000111 }" "asm"="vloxei8.v {vd}, ({rs1}), {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VLOXSEG2EI8_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if((vs2<vd)|(vs2>=vd+2)) 0010111 vs2:5 rs1:5 000 vd:5 0000111 }" "asm"="vloxseg2ei8.v {vd}, ({rs1}), {vs2}" "asmonly" {
  ret void
}

define void @VLOXSEG2EI8_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if((vs2<vd)|(vs2>=vd+2)) if(vd!=0) 0010110 vs2:5 rs1:5 000 vd:5 0000111 }" "asm"="vloxseg2ei8.v {vd}, ({rs1}), {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VLOXSEG3EI8_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if((vs2<vd)|(vs2>=vd+4)) 0100111 vs2:5 rs1:5 000 vd:5 0000111 }" "asm"="vloxseg3ei8.v {vd}, ({rs1}), {vs2}" "asmonly" {
  ret void
}

define void @VLOXSEG3EI8_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if((vs2<vd)|(vs2>=vd+4)) if(vd!=0) 0100110 vs2:5 rs1:5 000 vd:5 0000111 }" "asm"="vloxseg3ei8.v {vd}, ({rs1}), {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VLOXSEG4EI8_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if((vs2<vd)|(vs2>=vd+4)) 0110111 vs2:5 rs1:5 000 vd:5 0000111 }" "asm"="vloxseg4ei8.v {vd}, ({rs1}), {vs2}" "asmonly" {
  ret void
}

define void @VLOXSEG4EI8_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if((vs2<vd)|(vs2>=vd+4)) if(vd!=0) 0110110 vs2:5 rs1:5 000 vd:5 0000111 }" "asm"="vloxseg4ei8.v {vd}, ({rs1}), {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VLOXSEG5EI8_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if((vs2<vd)|(vs2>=vd+6)) 1000111 vs2:5 rs1:5 000 vd:5 0000111 }" "asm"="vloxseg5ei8.v {vd}, ({rs1}), {vs2}" "asmonly" {
  ret void
}

define void @VLOXSEG5EI8_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if((vs2<vd)|(vs2>=vd+6)) if(vd!=0) 1000110 vs2:5 rs1:5 000 vd:5 0000111 }" "asm"="vloxseg5ei8.v {vd}, ({rs1}), {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VLOXSEG6EI8_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if((vs2<vd)|(vs2>=vd+6)) 1010111 vs2:5 rs1:5 000 vd:5 0000111 }" "asm"="vloxseg6ei8.v {vd}, ({rs1}), {vs2}" "asmonly" {
  ret void
}

define void @VLOXSEG6EI8_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if((vs2<vd)|(vs2>=vd+6)) if(vd!=0) 1010110 vs2:5 rs1:5 000 vd:5 0000111 }" "asm"="vloxseg6ei8.v {vd}, ({rs1}), {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VLOXSEG7EI8_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if((vs2<vd)|(vs2>=vd+7)) 1100111 vs2:5 rs1:5 000 vd:5 0000111 }" "asm"="vloxseg7ei8.v {vd}, ({rs1}), {vs2}" "asmonly" {
  ret void
}

define void @VLOXSEG7EI8_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if((vs2<vd)|(vs2>=vd+7)) if(vd!=0) 1100110 vs2:5 rs1:5 000 vd:5 0000111 }" "asm"="vloxseg7ei8.v {vd}, ({rs1}), {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VLOXSEG8EI8_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if((vs2<vd)|(vs2>=vd+8)) 1110111 vs2:5 rs1:5 000 vd:5 0000111 }" "asm"="vloxseg8ei8.v {vd}, ({rs1}), {vs2}" "asmonly" {
  ret void
}

define void @VLOXSEG8EI8_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if((vs2<vd)|(vs2>=vd+8)) if(vd!=0) 1110110 vs2:5 rs1:5 000 vd:5 0000111 }" "asm"="vloxseg8ei8.v {vd}, ({rs1}), {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VLOXEI16_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="0000111 vs2:5 rs1:5 101 vd:5 0000111" "asm"="vloxei16.v {vd}, ({rs1}), {vs2}" "asmonly" {
  ret void
}

define void @VLOXEI16_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if(vd!=0) 0000110 vs2:5 rs1:5 101 vd:5 0000111 }" "asm"="vloxei16.v {vd}, ({rs1}), {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VLOXSEG2EI16_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if((vs2<vd)|(vs2>=vd+2)) 0010111 vs2:5 rs1:5 101 vd:5 0000111 }" "asm"="vloxseg2ei16.v {vd}, ({rs1}), {vs2}" "asmonly" {
  ret void
}

define void @VLOXSEG2EI16_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if((vs2<vd)|(vs2>=vd+2)) if(vd!=0) 0010110 vs2:5 rs1:5 101 vd:5 0000111 }" "asm"="vloxseg2ei16.v {vd}, ({rs1}), {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VLOXSEG3EI16_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if((vs2<vd)|(vs2>=vd+4)) 0100111 vs2:5 rs1:5 101 vd:5 0000111 }" "asm"="vloxseg3ei16.v {vd}, ({rs1}), {vs2}" "asmonly" {
  ret void
}

define void @VLOXSEG3EI16_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if((vs2<vd)|(vs2>=vd+4)) if(vd!=0) 0100110 vs2:5 rs1:5 101 vd:5 0000111 }" "asm"="vloxseg3ei16.v {vd}, ({rs1}), {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VLOXSEG4EI16_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if((vs2<vd)|(vs2>=vd+4)) 0110111 vs2:5 rs1:5 101 vd:5 0000111 }" "asm"="vloxseg4ei16.v {vd}, ({rs1}), {vs2}" "asmonly" {
  ret void
}

define void @VLOXSEG4EI16_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if((vs2<vd)|(vs2>=vd+4)) if(vd!=0) 0110110 vs2:5 rs1:5 101 vd:5 0000111 }" "asm"="vloxseg4ei16.v {vd}, ({rs1}), {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VLOXSEG5EI16_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if((vs2<vd)|(vs2>=vd+6)) 1000111 vs2:5 rs1:5 101 vd:5 0000111 }" "asm"="vloxseg5ei16.v {vd}, ({rs1}), {vs2}" "asmonly" {
  ret void
}

define void @VLOXSEG5EI16_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if((vs2<vd)|(vs2>=vd+6)) if(vd!=0) 1000110 vs2:5 rs1:5 101 vd:5 0000111 }" "asm"="vloxseg5ei16.v {vd}, ({rs1}), {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VLOXSEG6EI16_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if((vs2<vd)|(vs2>=vd+6)) 1010111 vs2:5 rs1:5 101 vd:5 0000111 }" "asm"="vloxseg6ei16.v {vd}, ({rs1}), {vs2}" "asmonly" {
  ret void
}

define void @VLOXSEG6EI16_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if((vs2<vd)|(vs2>=vd+6)) if(vd!=0) 1010110 vs2:5 rs1:5 101 vd:5 0000111 }" "asm"="vloxseg6ei16.v {vd}, ({rs1}), {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VLOXSEG7EI16_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if((vs2<vd)|(vs2>=vd+7)) 1100111 vs2:5 rs1:5 101 vd:5 0000111 }" "asm"="vloxseg7ei16.v {vd}, ({rs1}), {vs2}" "asmonly" {
  ret void
}

define void @VLOXSEG7EI16_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if((vs2<vd)|(vs2>=vd+7)) if(vd!=0) 1100110 vs2:5 rs1:5 101 vd:5 0000111 }" "asm"="vloxseg7ei16.v {vd}, ({rs1}), {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VLOXSEG8EI16_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if((vs2<vd)|(vs2>=vd+8)) 1110111 vs2:5 rs1:5 101 vd:5 0000111 }" "asm"="vloxseg8ei16.v {vd}, ({rs1}), {vs2}" "asmonly" {
  ret void
}

define void @VLOXSEG8EI16_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if((vs2<vd)|(vs2>=vd+8)) if(vd!=0) 1110110 vs2:5 rs1:5 101 vd:5 0000111 }" "asm"="vloxseg8ei16.v {vd}, ({rs1}), {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VLOXEI32_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="0000111 vs2:5 rs1:5 110 vd:5 0000111" "asm"="vloxei32.v {vd}, ({rs1}), {vs2}" "asmonly" {
  ret void
}

define void @VLOXEI32_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if(vd!=0) 0000110 vs2:5 rs1:5 110 vd:5 0000111 }" "asm"="vloxei32.v {vd}, ({rs1}), {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VLOXSEG2EI32_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if((vs2<vd)|(vs2>=vd+2)) 0010111 vs2:5 rs1:5 110 vd:5 0000111 }" "asm"="vloxseg2ei32.v {vd}, ({rs1}), {vs2}" "asmonly" {
  ret void
}

define void @VLOXSEG2EI32_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if((vs2<vd)|(vs2>=vd+2)) if(vd!=0) 0010110 vs2:5 rs1:5 110 vd:5 0000111 }" "asm"="vloxseg2ei32.v {vd}, ({rs1}), {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VLOXSEG3EI32_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if((vs2<vd)|(vs2>=vd+4)) 0100111 vs2:5 rs1:5 110 vd:5 0000111 }" "asm"="vloxseg3ei32.v {vd}, ({rs1}), {vs2}" "asmonly" {
  ret void
}

define void @VLOXSEG3EI32_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if((vs2<vd)|(vs2>=vd+4)) if(vd!=0) 0100110 vs2:5 rs1:5 110 vd:5 0000111 }" "asm"="vloxseg3ei32.v {vd}, ({rs1}), {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VLOXSEG4EI32_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if((vs2<vd)|(vs2>=vd+4)) 0110111 vs2:5 rs1:5 110 vd:5 0000111 }" "asm"="vloxseg4ei32.v {vd}, ({rs1}), {vs2}" "asmonly" {
  ret void
}

define void @VLOXSEG4EI32_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if((vs2<vd)|(vs2>=vd+4)) if(vd!=0) 0110110 vs2:5 rs1:5 110 vd:5 0000111 }" "asm"="vloxseg4ei32.v {vd}, ({rs1}), {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VLOXSEG5EI32_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if((vs2<vd)|(vs2>=vd+6)) 1000111 vs2:5 rs1:5 110 vd:5 0000111 }" "asm"="vloxseg5ei32.v {vd}, ({rs1}), {vs2}" "asmonly" {
  ret void
}

define void @VLOXSEG5EI32_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if((vs2<vd)|(vs2>=vd+6)) if(vd!=0) 1000110 vs2:5 rs1:5 110 vd:5 0000111 }" "asm"="vloxseg5ei32.v {vd}, ({rs1}), {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VLOXSEG6EI32_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if((vs2<vd)|(vs2>=vd+6)) 1010111 vs2:5 rs1:5 110 vd:5 0000111 }" "asm"="vloxseg6ei32.v {vd}, ({rs1}), {vs2}" "asmonly" {
  ret void
}

define void @VLOXSEG6EI32_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if((vs2<vd)|(vs2>=vd+6)) if(vd!=0) 1010110 vs2:5 rs1:5 110 vd:5 0000111 }" "asm"="vloxseg6ei32.v {vd}, ({rs1}), {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VLOXSEG7EI32_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if((vs2<vd)|(vs2>=vd+7)) 1100111 vs2:5 rs1:5 110 vd:5 0000111 }" "asm"="vloxseg7ei32.v {vd}, ({rs1}), {vs2}" "asmonly" {
  ret void
}

define void @VLOXSEG7EI32_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if((vs2<vd)|(vs2>=vd+7)) if(vd!=0) 1100110 vs2:5 rs1:5 110 vd:5 0000111 }" "asm"="vloxseg7ei32.v {vd}, ({rs1}), {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VLOXSEG8EI32_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if((vs2<vd)|(vs2>=vd+8)) 1110111 vs2:5 rs1:5 110 vd:5 0000111 }" "asm"="vloxseg8ei32.v {vd}, ({rs1}), {vs2}" "asmonly" {
  ret void
}

define void @VLOXSEG8EI32_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if((vs2<vd)|(vs2>=vd+8)) if(vd!=0) 1110110 vs2:5 rs1:5 110 vd:5 0000111 }" "asm"="vloxseg8ei32.v {vd}, ({rs1}), {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VLOXEI64_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="0000111 vs2:5 rs1:5 111 vd:5 0000111" "asm"="vloxei64.v {vd}, ({rs1}), {vs2}" "asmonly" {
  ret void
}

define void @VLOXEI64_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if(vd!=0) 0000110 vs2:5 rs1:5 111 vd:5 0000111 }" "asm"="vloxei64.v {vd}, ({rs1}), {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VLOXSEG2EI64_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if((vs2<vd)|(vs2>=vd+2)) 0010111 vs2:5 rs1:5 111 vd:5 0000111 }" "asm"="vloxseg2ei64.v {vd}, ({rs1}), {vs2}" "asmonly" {
  ret void
}

define void @VLOXSEG2EI64_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if((vs2<vd)|(vs2>=vd+2)) if(vd!=0) 0010110 vs2:5 rs1:5 111 vd:5 0000111 }" "asm"="vloxseg2ei64.v {vd}, ({rs1}), {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VLOXSEG3EI64_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if((vs2<vd)|(vs2>=vd+4)) 0100111 vs2:5 rs1:5 111 vd:5 0000111 }" "asm"="vloxseg3ei64.v {vd}, ({rs1}), {vs2}" "asmonly" {
  ret void
}

define void @VLOXSEG3EI64_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if((vs2<vd)|(vs2>=vd+4)) if(vd!=0) 0100110 vs2:5 rs1:5 111 vd:5 0000111 }" "asm"="vloxseg3ei64.v {vd}, ({rs1}), {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VLOXSEG4EI64_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if((vs2<vd)|(vs2>=vd+4)) 0110111 vs2:5 rs1:5 111 vd:5 0000111 }" "asm"="vloxseg4ei64.v {vd}, ({rs1}), {vs2}" "asmonly" {
  ret void
}

define void @VLOXSEG4EI64_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if((vs2<vd)|(vs2>=vd+4)) if(vd!=0) 0110110 vs2:5 rs1:5 111 vd:5 0000111 }" "asm"="vloxseg4ei64.v {vd}, ({rs1}), {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VLOXSEG5EI64_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if((vs2<vd)|(vs2>=vd+6)) 1000111 vs2:5 rs1:5 111 vd:5 0000111 }" "asm"="vloxseg5ei64.v {vd}, ({rs1}), {vs2}" "asmonly" {
  ret void
}

define void @VLOXSEG5EI64_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if((vs2<vd)|(vs2>=vd+6)) if(vd!=0) 1000110 vs2:5 rs1:5 111 vd:5 0000111 }" "asm"="vloxseg5ei64.v {vd}, ({rs1}), {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VLOXSEG6EI64_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if((vs2<vd)|(vs2>=vd+6)) 1010111 vs2:5 rs1:5 111 vd:5 0000111 }" "asm"="vloxseg6ei64.v {vd}, ({rs1}), {vs2}" "asmonly" {
  ret void
}

define void @VLOXSEG6EI64_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if((vs2<vd)|(vs2>=vd+6)) if(vd!=0) 1010110 vs2:5 rs1:5 111 vd:5 0000111 }" "asm"="vloxseg6ei64.v {vd}, ({rs1}), {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VLOXSEG7EI64_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if((vs2<vd)|(vs2>=vd+7)) 1100111 vs2:5 rs1:5 111 vd:5 0000111 }" "asm"="vloxseg7ei64.v {vd}, ({rs1}), {vs2}" "asmonly" {
  ret void
}

define void @VLOXSEG7EI64_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if((vs2<vd)|(vs2>=vd+7)) if(vd!=0) 1100110 vs2:5 rs1:5 111 vd:5 0000111 }" "asm"="vloxseg7ei64.v {vd}, ({rs1}), {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VLOXSEG8EI64_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if((vs2<vd)|(vs2>=vd+8)) 1110111 vs2:5 rs1:5 111 vd:5 0000111 }" "asm"="vloxseg8ei64.v {vd}, ({rs1}), {vs2}" "asmonly" {
  ret void
}

define void @VLOXSEG8EI64_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if((vs2<vd)|(vs2>=vd+8)) if(vd!=0) 1110110 vs2:5 rs1:5 111 vd:5 0000111 }" "asm"="vloxseg8ei64.v {vd}, ({rs1}), {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VSOXEI8_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3, i64 "reg"="v" %vs2) "enc"="0000111 vs2:5 rs1:5 000 vs3:5 0100111" "asm"="vsoxei8.v {vs3}, ({rs1}), {vs2}" "asmonly" {
  ret void
}

define void @VSOXEI8_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3, i64 "reg"="v" %vs2) "enc"="0000110 vs2:5 rs1:5 000 vs3:5 0100111" "asm"="vsoxei8.v {vs3}, ({rs1}), {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VSOXSEG2EI8_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3, i64 "reg"="v" %vs2) "enc"="0010111 vs2:5 rs1:5 000 vs3:5 0100111" "asm"="vsoxseg2ei8.v {vs3}, ({rs1}), {vs2}" "asmonly" {
  ret void
}

define void @VSOXSEG2EI8_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3, i64 "reg"="v" %vs2) "enc"="0010110 vs2:5 rs1:5 000 vs3:5 0100111" "asm"="vsoxseg2ei8.v {vs3}, ({rs1}), {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VSOXSEG3EI8_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3, i64 "reg"="v" %vs2) "enc"="0100111 vs2:5 rs1:5 000 vs3:5 0100111" "asm"="vsoxseg3ei8.v {vs3}, ({rs1}), {vs2}" "asmonly" {
  ret void
}

define void @VSOXSEG3EI8_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3, i64 "reg"="v" %vs2) "enc"="0100110 vs2:5 rs1:5 000 vs3:5 0100111" "asm"="vsoxseg3ei8.v {vs3}, ({rs1}), {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VSOXSEG4EI8_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3, i64 "reg"="v" %vs2) "enc"="0110111 vs2:5 rs1:5 000 vs3:5 0100111" "asm"="vsoxseg4ei8.v {vs3}, ({rs1}), {vs2}" "asmonly" {
  ret void
}

define void @VSOXSEG4EI8_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3, i64 "reg"="v" %vs2) "enc"="0110110 vs2:5 rs1:5 000 vs3:5 0100111" "asm"="vsoxseg4ei8.v {vs3}, ({rs1}), {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VSOXSEG5EI8_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3, i64 "reg"="v" %vs2) "enc"="1000111 vs2:5 rs1:5 000 vs3:5 0100111" "asm"="vsoxseg5ei8.v {vs3}, ({rs1}), {vs2}" "asmonly" {
  ret void
}

define void @VSOXSEG5EI8_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3, i64 "reg"="v" %vs2) "enc"="1000110 vs2:5 rs1:5 000 vs3:5 0100111" "asm"="vsoxseg5ei8.v {vs3}, ({rs1}), {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VSOXSEG6EI8_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3, i64 "reg"="v" %vs2) "enc"="1010111 vs2:5 rs1:5 000 vs3:5 0100111" "asm"="vsoxseg6ei8.v {vs3}, ({rs1}), {vs2}" "asmonly" {
  ret void
}

define void @VSOXSEG6EI8_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3, i64 "reg"="v" %vs2) "enc"="1010110 vs2:5 rs1:5 000 vs3:5 0100111" "asm"="vsoxseg6ei8.v {vs3}, ({rs1}), {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VSOXSEG7EI8_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3, i64 "reg"="v" %vs2) "enc"="1100111 vs2:5 rs1:5 000 vs3:5 0100111" "asm"="vsoxseg7ei8.v {vs3}, ({rs1}), {vs2}" "asmonly" {
  ret void
}

define void @VSOXSEG7EI8_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3, i64 "reg"="v" %vs2) "enc"="1100110 vs2:5 rs1:5 000 vs3:5 0100111" "asm"="vsoxseg7ei8.v {vs3}, ({rs1}), {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VSOXSEG8EI8_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3, i64 "reg"="v" %vs2) "enc"="1110111 vs2:5 rs1:5 000 vs3:5 0100111" "asm"="vsoxseg8ei8.v {vs3}, ({rs1}), {vs2}" "asmonly" {
  ret void
}

define void @VSOXSEG8EI8_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3, i64 "reg"="v" %vs2) "enc"="1110110 vs2:5 rs1:5 000 vs3:5 0100111" "asm"="vsoxseg8ei8.v {vs3}, ({rs1}), {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VSOXEI16_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3, i64 "reg"="v" %vs2) "enc"="0000111 vs2:5 rs1:5 101 vs3:5 0100111" "asm"="vsoxei16.v {vs3}, ({rs1}), {vs2}" "asmonly" {
  ret void
}

define void @VSOXEI16_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3, i64 "reg"="v" %vs2) "enc"="0000110 vs2:5 rs1:5 101 vs3:5 0100111" "asm"="vsoxei16.v {vs3}, ({rs1}), {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VSOXSEG2EI16_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3, i64 "reg"="v" %vs2) "enc"="0010111 vs2:5 rs1:5 101 vs3:5 0100111" "asm"="vsoxseg2ei16.v {vs3}, ({rs1}), {vs2}" "asmonly" {
  ret void
}

define void @VSOXSEG2EI16_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3, i64 "reg"="v" %vs2) "enc"="0010110 vs2:5 rs1:5 101 vs3:5 0100111" "asm"="vsoxseg2ei16.v {vs3}, ({rs1}), {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VSOXSEG3EI16_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3, i64 "reg"="v" %vs2) "enc"="0100111 vs2:5 rs1:5 101 vs3:5 0100111" "asm"="vsoxseg3ei16.v {vs3}, ({rs1}), {vs2}" "asmonly" {
  ret void
}

define void @VSOXSEG3EI16_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3, i64 "reg"="v" %vs2) "enc"="0100110 vs2:5 rs1:5 101 vs3:5 0100111" "asm"="vsoxseg3ei16.v {vs3}, ({rs1}), {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VSOXSEG4EI16_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3, i64 "reg"="v" %vs2) "enc"="0110111 vs2:5 rs1:5 101 vs3:5 0100111" "asm"="vsoxseg4ei16.v {vs3}, ({rs1}), {vs2}" "asmonly" {
  ret void
}

define void @VSOXSEG4EI16_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3, i64 "reg"="v" %vs2) "enc"="0110110 vs2:5 rs1:5 101 vs3:5 0100111" "asm"="vsoxseg4ei16.v {vs3}, ({rs1}), {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VSOXSEG5EI16_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3, i64 "reg"="v" %vs2) "enc"="1000111 vs2:5 rs1:5 101 vs3:5 0100111" "asm"="vsoxseg5ei16.v {vs3}, ({rs1}), {vs2}" "asmonly" {
  ret void
}

define void @VSOXSEG5EI16_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3, i64 "reg"="v" %vs2) "enc"="1000110 vs2:5 rs1:5 101 vs3:5 0100111" "asm"="vsoxseg5ei16.v {vs3}, ({rs1}), {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VSOXSEG6EI16_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3, i64 "reg"="v" %vs2) "enc"="1010111 vs2:5 rs1:5 101 vs3:5 0100111" "asm"="vsoxseg6ei16.v {vs3}, ({rs1}), {vs2}" "asmonly" {
  ret void
}

define void @VSOXSEG6EI16_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3, i64 "reg"="v" %vs2) "enc"="1010110 vs2:5 rs1:5 101 vs3:5 0100111" "asm"="vsoxseg6ei16.v {vs3}, ({rs1}), {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VSOXSEG7EI16_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3, i64 "reg"="v" %vs2) "enc"="1100111 vs2:5 rs1:5 101 vs3:5 0100111" "asm"="vsoxseg7ei16.v {vs3}, ({rs1}), {vs2}" "asmonly" {
  ret void
}

define void @VSOXSEG7EI16_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3, i64 "reg"="v" %vs2) "enc"="1100110 vs2:5 rs1:5 101 vs3:5 0100111" "asm"="vsoxseg7ei16.v {vs3}, ({rs1}), {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VSOXSEG8EI16_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3, i64 "reg"="v" %vs2) "enc"="1110111 vs2:5 rs1:5 101 vs3:5 0100111" "asm"="vsoxseg8ei16.v {vs3}, ({rs1}), {vs2}" "asmonly" {
  ret void
}

define void @VSOXSEG8EI16_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3, i64 "reg"="v" %vs2) "enc"="1110110 vs2:5 rs1:5 101 vs3:5 0100111" "asm"="vsoxseg8ei16.v {vs3}, ({rs1}), {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VSOXEI32_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3, i64 "reg"="v" %vs2) "enc"="0000111 vs2:5 rs1:5 110 vs3:5 0100111" "asm"="vsoxei32.v {vs3}, ({rs1}), {vs2}" "asmonly" {
  ret void
}

define void @VSOXEI32_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3, i64 "reg"="v" %vs2) "enc"="0000110 vs2:5 rs1:5 110 vs3:5 0100111" "asm"="vsoxei32.v {vs3}, ({rs1}), {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VSOXSEG2EI32_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3, i64 "reg"="v" %vs2) "enc"="0010111 vs2:5 rs1:5 110 vs3:5 0100111" "asm"="vsoxseg2ei32.v {vs3}, ({rs1}), {vs2}" "asmonly" {
  ret void
}

define void @VSOXSEG2EI32_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3, i64 "reg"="v" %vs2) "enc"="0010110 vs2:5 rs1:5 110 vs3:5 0100111" "asm"="vsoxseg2ei32.v {vs3}, ({rs1}), {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VSOXSEG3EI32_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3, i64 "reg"="v" %vs2) "enc"="0100111 vs2:5 rs1:5 110 vs3:5 0100111" "asm"="vsoxseg3ei32.v {vs3}, ({rs1}), {vs2}" "asmonly" {
  ret void
}

define void @VSOXSEG3EI32_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3, i64 "reg"="v" %vs2) "enc"="0100110 vs2:5 rs1:5 110 vs3:5 0100111" "asm"="vsoxseg3ei32.v {vs3}, ({rs1}), {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VSOXSEG4EI32_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3, i64 "reg"="v" %vs2) "enc"="0110111 vs2:5 rs1:5 110 vs3:5 0100111" "asm"="vsoxseg4ei32.v {vs3}, ({rs1}), {vs2}" "asmonly" {
  ret void
}

define void @VSOXSEG4EI32_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3, i64 "reg"="v" %vs2) "enc"="0110110 vs2:5 rs1:5 110 vs3:5 0100111" "asm"="vsoxseg4ei32.v {vs3}, ({rs1}), {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VSOXSEG5EI32_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3, i64 "reg"="v" %vs2) "enc"="1000111 vs2:5 rs1:5 110 vs3:5 0100111" "asm"="vsoxseg5ei32.v {vs3}, ({rs1}), {vs2}" "asmonly" {
  ret void
}

define void @VSOXSEG5EI32_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3, i64 "reg"="v" %vs2) "enc"="1000110 vs2:5 rs1:5 110 vs3:5 0100111" "asm"="vsoxseg5ei32.v {vs3}, ({rs1}), {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VSOXSEG6EI32_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3, i64 "reg"="v" %vs2) "enc"="1010111 vs2:5 rs1:5 110 vs3:5 0100111" "asm"="vsoxseg6ei32.v {vs3}, ({rs1}), {vs2}" "asmonly" {
  ret void
}

define void @VSOXSEG6EI32_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3, i64 "reg"="v" %vs2) "enc"="1010110 vs2:5 rs1:5 110 vs3:5 0100111" "asm"="vsoxseg6ei32.v {vs3}, ({rs1}), {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VSOXSEG7EI32_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3, i64 "reg"="v" %vs2) "enc"="1100111 vs2:5 rs1:5 110 vs3:5 0100111" "asm"="vsoxseg7ei32.v {vs3}, ({rs1}), {vs2}" "asmonly" {
  ret void
}

define void @VSOXSEG7EI32_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3, i64 "reg"="v" %vs2) "enc"="1100110 vs2:5 rs1:5 110 vs3:5 0100111" "asm"="vsoxseg7ei32.v {vs3}, ({rs1}), {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VSOXSEG8EI32_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3, i64 "reg"="v" %vs2) "enc"="1110111 vs2:5 rs1:5 110 vs3:5 0100111" "asm"="vsoxseg8ei32.v {vs3}, ({rs1}), {vs2}" "asmonly" {
  ret void
}

define void @VSOXSEG8EI32_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3, i64 "reg"="v" %vs2) "enc"="1110110 vs2:5 rs1:5 110 vs3:5 0100111" "asm"="vsoxseg8ei32.v {vs3}, ({rs1}), {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VSOXEI64_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3, i64 "reg"="v" %vs2) "enc"="0000111 vs2:5 rs1:5 111 vs3:5 0100111" "asm"="vsoxei64.v {vs3}, ({rs1}), {vs2}" "asmonly" {
  ret void
}

define void @VSOXEI64_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3, i64 "reg"="v" %vs2) "enc"="0000110 vs2:5 rs1:5 111 vs3:5 0100111" "asm"="vsoxei64.v {vs3}, ({rs1}), {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VSOXSEG2EI64_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3, i64 "reg"="v" %vs2) "enc"="0010111 vs2:5 rs1:5 111 vs3:5 0100111" "asm"="vsoxseg2ei64.v {vs3}, ({rs1}), {vs2}" "asmonly" {
  ret void
}

define void @VSOXSEG2EI64_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3, i64 "reg"="v" %vs2) "enc"="0010110 vs2:5 rs1:5 111 vs3:5 0100111" "asm"="vsoxseg2ei64.v {vs3}, ({rs1}), {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VSOXSEG3EI64_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3, i64 "reg"="v" %vs2) "enc"="0100111 vs2:5 rs1:5 111 vs3:5 0100111" "asm"="vsoxseg3ei64.v {vs3}, ({rs1}), {vs2}" "asmonly" {
  ret void
}

define void @VSOXSEG3EI64_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3, i64 "reg"="v" %vs2) "enc"="0100110 vs2:5 rs1:5 111 vs3:5 0100111" "asm"="vsoxseg3ei64.v {vs3}, ({rs1}), {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VSOXSEG4EI64_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3, i64 "reg"="v" %vs2) "enc"="0110111 vs2:5 rs1:5 111 vs3:5 0100111" "asm"="vsoxseg4ei64.v {vs3}, ({rs1}), {vs2}" "asmonly" {
  ret void
}

define void @VSOXSEG4EI64_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3, i64 "reg"="v" %vs2) "enc"="0110110 vs2:5 rs1:5 111 vs3:5 0100111" "asm"="vsoxseg4ei64.v {vs3}, ({rs1}), {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VSOXSEG5EI64_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3, i64 "reg"="v" %vs2) "enc"="1000111 vs2:5 rs1:5 111 vs3:5 0100111" "asm"="vsoxseg5ei64.v {vs3}, ({rs1}), {vs2}" "asmonly" {
  ret void
}

define void @VSOXSEG5EI64_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3, i64 "reg"="v" %vs2) "enc"="1000110 vs2:5 rs1:5 111 vs3:5 0100111" "asm"="vsoxseg5ei64.v {vs3}, ({rs1}), {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VSOXSEG6EI64_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3, i64 "reg"="v" %vs2) "enc"="1010111 vs2:5 rs1:5 111 vs3:5 0100111" "asm"="vsoxseg6ei64.v {vs3}, ({rs1}), {vs2}" "asmonly" {
  ret void
}

define void @VSOXSEG6EI64_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3, i64 "reg"="v" %vs2) "enc"="1010110 vs2:5 rs1:5 111 vs3:5 0100111" "asm"="vsoxseg6ei64.v {vs3}, ({rs1}), {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VSOXSEG7EI64_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3, i64 "reg"="v" %vs2) "enc"="1100111 vs2:5 rs1:5 111 vs3:5 0100111" "asm"="vsoxseg7ei64.v {vs3}, ({rs1}), {vs2}" "asmonly" {
  ret void
}

define void @VSOXSEG7EI64_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3, i64 "reg"="v" %vs2) "enc"="1100110 vs2:5 rs1:5 111 vs3:5 0100111" "asm"="vsoxseg7ei64.v {vs3}, ({rs1}), {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VSOXSEG8EI64_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3, i64 "reg"="v" %vs2) "enc"="1110111 vs2:5 rs1:5 111 vs3:5 0100111" "asm"="vsoxseg8ei64.v {vs3}, ({rs1}), {vs2}" "asmonly" {
  ret void
}

define void @VSOXSEG8EI64_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3, i64 "reg"="v" %vs2) "enc"="1110110 vs2:5 rs1:5 111 vs3:5 0100111" "asm"="vsoxseg8ei64.v {vs3}, ({rs1}), {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VLE8FF_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd) "enc"="000000110000 rs1:5 000 vd:5 0000111" "asm"="vle8ff.v {vd}, ({rs1})" "asmonly" {
  ret void
}

define void @VLE8FF_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd) "enc"="{ if(vd!=0) 000000010000 rs1:5 000 vd:5 0000111 }" "asm"="vle8ff.v {vd}, ({rs1}), v0.t" "asmonly" {
  ret void
}

define void @VLSEG2E8FF_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd) "enc"="001000110000 rs1:5 000 vd:5 0000111" "asm"="vlseg2e8ff.v {vd}, ({rs1})" "asmonly" {
  ret void
}

define void @VLSEG2E8FF_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd) "enc"="{ if(vd!=0) 001000010000 rs1:5 000 vd:5 0000111 }" "asm"="vlseg2e8ff.v {vd}, ({rs1}), v0.t" "asmonly" {
  ret void
}

define void @VLSEG3E8FF_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd) "enc"="010000110000 rs1:5 000 vd:5 0000111" "asm"="vlseg3e8ff.v {vd}, ({rs1})" "asmonly" {
  ret void
}

define void @VLSEG3E8FF_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd) "enc"="{ if(vd!=0) 010000010000 rs1:5 000 vd:5 0000111 }" "asm"="vlseg3e8ff.v {vd}, ({rs1}), v0.t" "asmonly" {
  ret void
}

define void @VLSEG4E8FF_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd) "enc"="011000110000 rs1:5 000 vd:5 0000111" "asm"="vlseg4e8ff.v {vd}, ({rs1})" "asmonly" {
  ret void
}

define void @VLSEG4E8FF_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd) "enc"="{ if(vd!=0) 011000010000 rs1:5 000 vd:5 0000111 }" "asm"="vlseg4e8ff.v {vd}, ({rs1}), v0.t" "asmonly" {
  ret void
}

define void @VLSEG5E8FF_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd) "enc"="100000110000 rs1:5 000 vd:5 0000111" "asm"="vlseg5e8ff.v {vd}, ({rs1})" "asmonly" {
  ret void
}

define void @VLSEG5E8FF_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd) "enc"="{ if(vd!=0) 100000010000 rs1:5 000 vd:5 0000111 }" "asm"="vlseg5e8ff.v {vd}, ({rs1}), v0.t" "asmonly" {
  ret void
}

define void @VLSEG6E8FF_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd) "enc"="101000110000 rs1:5 000 vd:5 0000111" "asm"="vlseg6e8ff.v {vd}, ({rs1})" "asmonly" {
  ret void
}

define void @VLSEG6E8FF_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd) "enc"="{ if(vd!=0) 101000010000 rs1:5 000 vd:5 0000111 }" "asm"="vlseg6e8ff.v {vd}, ({rs1}), v0.t" "asmonly" {
  ret void
}

define void @VLSEG7E8FF_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd) "enc"="110000110000 rs1:5 000 vd:5 0000111" "asm"="vlseg7e8ff.v {vd}, ({rs1})" "asmonly" {
  ret void
}

define void @VLSEG7E8FF_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd) "enc"="{ if(vd!=0) 110000010000 rs1:5 000 vd:5 0000111 }" "asm"="vlseg7e8ff.v {vd}, ({rs1}), v0.t" "asmonly" {
  ret void
}

define void @VLSEG8E8FF_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd) "enc"="111000110000 rs1:5 000 vd:5 0000111" "asm"="vlseg8e8ff.v {vd}, ({rs1})" "asmonly" {
  ret void
}

define void @VLSEG8E8FF_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd) "enc"="{ if(vd!=0) 111000010000 rs1:5 000 vd:5 0000111 }" "asm"="vlseg8e8ff.v {vd}, ({rs1}), v0.t" "asmonly" {
  ret void
}

define void @VLE16FF_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd) "enc"="000000110000 rs1:5 101 vd:5 0000111" "asm"="vle16ff.v {vd}, ({rs1})" "asmonly" {
  ret void
}

define void @VLE16FF_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd) "enc"="{ if(vd!=0) 000000010000 rs1:5 101 vd:5 0000111 }" "asm"="vle16ff.v {vd}, ({rs1}), v0.t" "asmonly" {
  ret void
}

define void @VLSEG2E16FF_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd) "enc"="001000110000 rs1:5 101 vd:5 0000111" "asm"="vlseg2e16ff.v {vd}, ({rs1})" "asmonly" {
  ret void
}

define void @VLSEG2E16FF_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd) "enc"="{ if(vd!=0) 001000010000 rs1:5 101 vd:5 0000111 }" "asm"="vlseg2e16ff.v {vd}, ({rs1}), v0.t" "asmonly" {
  ret void
}

define void @VLSEG3E16FF_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd) "enc"="010000110000 rs1:5 101 vd:5 0000111" "asm"="vlseg3e16ff.v {vd}, ({rs1})" "asmonly" {
  ret void
}

define void @VLSEG3E16FF_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd) "enc"="{ if(vd!=0) 010000010000 rs1:5 101 vd:5 0000111 }" "asm"="vlseg3e16ff.v {vd}, ({rs1}), v0.t" "asmonly" {
  ret void
}

define void @VLSEG4E16FF_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd) "enc"="011000110000 rs1:5 101 vd:5 0000111" "asm"="vlseg4e16ff.v {vd}, ({rs1})" "asmonly" {
  ret void
}

define void @VLSEG4E16FF_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd) "enc"="{ if(vd!=0) 011000010000 rs1:5 101 vd:5 0000111 }" "asm"="vlseg4e16ff.v {vd}, ({rs1}), v0.t" "asmonly" {
  ret void
}

define void @VLSEG5E16FF_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd) "enc"="100000110000 rs1:5 101 vd:5 0000111" "asm"="vlseg5e16ff.v {vd}, ({rs1})" "asmonly" {
  ret void
}

define void @VLSEG5E16FF_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd) "enc"="{ if(vd!=0) 100000010000 rs1:5 101 vd:5 0000111 }" "asm"="vlseg5e16ff.v {vd}, ({rs1}), v0.t" "asmonly" {
  ret void
}

define void @VLSEG6E16FF_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd) "enc"="101000110000 rs1:5 101 vd:5 0000111" "asm"="vlseg6e16ff.v {vd}, ({rs1})" "asmonly" {
  ret void
}

define void @VLSEG6E16FF_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd) "enc"="{ if(vd!=0) 101000010000 rs1:5 101 vd:5 0000111 }" "asm"="vlseg6e16ff.v {vd}, ({rs1}), v0.t" "asmonly" {
  ret void
}

define void @VLSEG7E16FF_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd) "enc"="110000110000 rs1:5 101 vd:5 0000111" "asm"="vlseg7e16ff.v {vd}, ({rs1})" "asmonly" {
  ret void
}

define void @VLSEG7E16FF_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd) "enc"="{ if(vd!=0) 110000010000 rs1:5 101 vd:5 0000111 }" "asm"="vlseg7e16ff.v {vd}, ({rs1}), v0.t" "asmonly" {
  ret void
}

define void @VLSEG8E16FF_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd) "enc"="111000110000 rs1:5 101 vd:5 0000111" "asm"="vlseg8e16ff.v {vd}, ({rs1})" "asmonly" {
  ret void
}

define void @VLSEG8E16FF_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd) "enc"="{ if(vd!=0) 111000010000 rs1:5 101 vd:5 0000111 }" "asm"="vlseg8e16ff.v {vd}, ({rs1}), v0.t" "asmonly" {
  ret void
}

define void @VLE32FF_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd) "enc"="000000110000 rs1:5 110 vd:5 0000111" "asm"="vle32ff.v {vd}, ({rs1})" "asmonly" {
  ret void
}

define void @VLE32FF_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd) "enc"="{ if(vd!=0) 000000010000 rs1:5 110 vd:5 0000111 }" "asm"="vle32ff.v {vd}, ({rs1}), v0.t" "asmonly" {
  ret void
}

define void @VLSEG2E32FF_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd) "enc"="001000110000 rs1:5 110 vd:5 0000111" "asm"="vlseg2e32ff.v {vd}, ({rs1})" "asmonly" {
  ret void
}

define void @VLSEG2E32FF_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd) "enc"="{ if(vd!=0) 001000010000 rs1:5 110 vd:5 0000111 }" "asm"="vlseg2e32ff.v {vd}, ({rs1}), v0.t" "asmonly" {
  ret void
}

define void @VLSEG3E32FF_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd) "enc"="010000110000 rs1:5 110 vd:5 0000111" "asm"="vlseg3e32ff.v {vd}, ({rs1})" "asmonly" {
  ret void
}

define void @VLSEG3E32FF_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd) "enc"="{ if(vd!=0) 010000010000 rs1:5 110 vd:5 0000111 }" "asm"="vlseg3e32ff.v {vd}, ({rs1}), v0.t" "asmonly" {
  ret void
}

define void @VLSEG4E32FF_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd) "enc"="011000110000 rs1:5 110 vd:5 0000111" "asm"="vlseg4e32ff.v {vd}, ({rs1})" "asmonly" {
  ret void
}

define void @VLSEG4E32FF_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd) "enc"="{ if(vd!=0) 011000010000 rs1:5 110 vd:5 0000111 }" "asm"="vlseg4e32ff.v {vd}, ({rs1}), v0.t" "asmonly" {
  ret void
}

define void @VLSEG5E32FF_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd) "enc"="100000110000 rs1:5 110 vd:5 0000111" "asm"="vlseg5e32ff.v {vd}, ({rs1})" "asmonly" {
  ret void
}

define void @VLSEG5E32FF_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd) "enc"="{ if(vd!=0) 100000010000 rs1:5 110 vd:5 0000111 }" "asm"="vlseg5e32ff.v {vd}, ({rs1}), v0.t" "asmonly" {
  ret void
}

define void @VLSEG6E32FF_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd) "enc"="101000110000 rs1:5 110 vd:5 0000111" "asm"="vlseg6e32ff.v {vd}, ({rs1})" "asmonly" {
  ret void
}

define void @VLSEG6E32FF_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd) "enc"="{ if(vd!=0) 101000010000 rs1:5 110 vd:5 0000111 }" "asm"="vlseg6e32ff.v {vd}, ({rs1}), v0.t" "asmonly" {
  ret void
}

define void @VLSEG7E32FF_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd) "enc"="110000110000 rs1:5 110 vd:5 0000111" "asm"="vlseg7e32ff.v {vd}, ({rs1})" "asmonly" {
  ret void
}

define void @VLSEG7E32FF_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd) "enc"="{ if(vd!=0) 110000010000 rs1:5 110 vd:5 0000111 }" "asm"="vlseg7e32ff.v {vd}, ({rs1}), v0.t" "asmonly" {
  ret void
}

define void @VLSEG8E32FF_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd) "enc"="111000110000 rs1:5 110 vd:5 0000111" "asm"="vlseg8e32ff.v {vd}, ({rs1})" "asmonly" {
  ret void
}

define void @VLSEG8E32FF_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd) "enc"="{ if(vd!=0) 111000010000 rs1:5 110 vd:5 0000111 }" "asm"="vlseg8e32ff.v {vd}, ({rs1}), v0.t" "asmonly" {
  ret void
}

define void @VLE64FF_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd) "enc"="000000110000 rs1:5 111 vd:5 0000111" "asm"="vle64ff.v {vd}, ({rs1})" "asmonly" {
  ret void
}

define void @VLE64FF_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd) "enc"="{ if(vd!=0) 000000010000 rs1:5 111 vd:5 0000111 }" "asm"="vle64ff.v {vd}, ({rs1}), v0.t" "asmonly" {
  ret void
}

define void @VLSEG2E64FF_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd) "enc"="001000110000 rs1:5 111 vd:5 0000111" "asm"="vlseg2e64ff.v {vd}, ({rs1})" "asmonly" {
  ret void
}

define void @VLSEG2E64FF_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd) "enc"="{ if(vd!=0) 001000010000 rs1:5 111 vd:5 0000111 }" "asm"="vlseg2e64ff.v {vd}, ({rs1}), v0.t" "asmonly" {
  ret void
}

define void @VLSEG3E64FF_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd) "enc"="010000110000 rs1:5 111 vd:5 0000111" "asm"="vlseg3e64ff.v {vd}, ({rs1})" "asmonly" {
  ret void
}

define void @VLSEG3E64FF_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd) "enc"="{ if(vd!=0) 010000010000 rs1:5 111 vd:5 0000111 }" "asm"="vlseg3e64ff.v {vd}, ({rs1}), v0.t" "asmonly" {
  ret void
}

define void @VLSEG4E64FF_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd) "enc"="011000110000 rs1:5 111 vd:5 0000111" "asm"="vlseg4e64ff.v {vd}, ({rs1})" "asmonly" {
  ret void
}

define void @VLSEG4E64FF_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd) "enc"="{ if(vd!=0) 011000010000 rs1:5 111 vd:5 0000111 }" "asm"="vlseg4e64ff.v {vd}, ({rs1}), v0.t" "asmonly" {
  ret void
}

define void @VLSEG5E64FF_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd) "enc"="100000110000 rs1:5 111 vd:5 0000111" "asm"="vlseg5e64ff.v {vd}, ({rs1})" "asmonly" {
  ret void
}

define void @VLSEG5E64FF_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd) "enc"="{ if(vd!=0) 100000010000 rs1:5 111 vd:5 0000111 }" "asm"="vlseg5e64ff.v {vd}, ({rs1}), v0.t" "asmonly" {
  ret void
}

define void @VLSEG6E64FF_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd) "enc"="101000110000 rs1:5 111 vd:5 0000111" "asm"="vlseg6e64ff.v {vd}, ({rs1})" "asmonly" {
  ret void
}

define void @VLSEG6E64FF_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd) "enc"="{ if(vd!=0) 101000010000 rs1:5 111 vd:5 0000111 }" "asm"="vlseg6e64ff.v {vd}, ({rs1}), v0.t" "asmonly" {
  ret void
}

define void @VLSEG7E64FF_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd) "enc"="110000110000 rs1:5 111 vd:5 0000111" "asm"="vlseg7e64ff.v {vd}, ({rs1})" "asmonly" {
  ret void
}

define void @VLSEG7E64FF_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd) "enc"="{ if(vd!=0) 110000010000 rs1:5 111 vd:5 0000111 }" "asm"="vlseg7e64ff.v {vd}, ({rs1}), v0.t" "asmonly" {
  ret void
}

define void @VLSEG8E64FF_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd) "enc"="111000110000 rs1:5 111 vd:5 0000111" "asm"="vlseg8e64ff.v {vd}, ({rs1})" "asmonly" {
  ret void
}

define void @VLSEG8E64FF_V.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd) "enc"="{ if(vd!=0) 111000010000 rs1:5 111 vd:5 0000111 }" "asm"="vlseg8e64ff.v {vd}, ({rs1}), v0.t" "asmonly" {
  ret void
}

define void @VL1RE8_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd) "enc"="000000101000 rs1:5 000 vd:5 0000111" "asm"="vl1re8.v {vd}, ({rs1})" "asmonly" {
  ret void
}

define void @VL1RE16_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd) "enc"="000000101000 rs1:5 101 vd:5 0000111" "asm"="vl1re16.v {vd}, ({rs1})" "asmonly" {
  ret void
}

define void @VL1RE32_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd) "enc"="000000101000 rs1:5 110 vd:5 0000111" "asm"="vl1re32.v {vd}, ({rs1})" "asmonly" {
  ret void
}

define void @VL1RE64_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd) "enc"="000000101000 rs1:5 111 vd:5 0000111" "asm"="vl1re64.v {vd}, ({rs1})" "asmonly" {
  ret void
}

define void @VL2RE8_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd) "enc"="{ if(vd%2==0) 001000101000 rs1:5 000 vd:5 0000111 }" "asm"="vl2re8.v {vd}, ({rs1})" "asmonly" {
  ret void
}

define void @VL2RE16_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd) "enc"="{ if(vd%2==0) 001000101000 rs1:5 101 vd:5 0000111 }" "asm"="vl2re16.v {vd}, ({rs1})" "asmonly" {
  ret void
}

define void @VL2RE32_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd) "enc"="{ if(vd%2==0) 001000101000 rs1:5 110 vd:5 0000111 }" "asm"="vl2re32.v {vd}, ({rs1})" "asmonly" {
  ret void
}

define void @VL2RE64_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd) "enc"="{ if(vd%2==0) 001000101000 rs1:5 111 vd:5 0000111 }" "asm"="vl2re64.v {vd}, ({rs1})" "asmonly" {
  ret void
}

define void @VL4RE8_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd) "enc"="{ if(vd%4==0) 011000101000 rs1:5 000 vd:5 0000111 }" "asm"="vl4re8.v {vd}, ({rs1})" "asmonly" {
  ret void
}

define void @VL4RE16_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd) "enc"="{ if(vd%4==0) 011000101000 rs1:5 101 vd:5 0000111 }" "asm"="vl4re16.v {vd}, ({rs1})" "asmonly" {
  ret void
}

define void @VL4RE32_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd) "enc"="{ if(vd%4==0) 011000101000 rs1:5 110 vd:5 0000111 }" "asm"="vl4re32.v {vd}, ({rs1})" "asmonly" {
  ret void
}

define void @VL4RE64_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd) "enc"="{ if(vd%4==0) 011000101000 rs1:5 111 vd:5 0000111 }" "asm"="vl4re64.v {vd}, ({rs1})" "asmonly" {
  ret void
}

define void @VL8RE8_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd) "enc"="{ if(vd%8==0) 111000101000 rs1:5 000 vd:5 0000111 }" "asm"="vl8re8.v {vd}, ({rs1})" "asmonly" {
  ret void
}

define void @VL8RE16_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd) "enc"="{ if(vd%8==0) 111000101000 rs1:5 101 vd:5 0000111 }" "asm"="vl8re16.v {vd}, ({rs1})" "asmonly" {
  ret void
}

define void @VL8RE32_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd) "enc"="{ if(vd%8==0) 111000101000 rs1:5 110 vd:5 0000111 }" "asm"="vl8re32.v {vd}, ({rs1})" "asmonly" {
  ret void
}

define void @VL8RE64_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd) "enc"="{ if(vd%8==0) 111000101000 rs1:5 111 vd:5 0000111 }" "asm"="vl8re64.v {vd}, ({rs1})" "asmonly" {
  ret void
}

define void @VS1R_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3) "enc"="000000101000 rs1:5 000 vs3:5 0100111" "asm"="vs1r.v {vs3}, ({rs1})" "asmonly" {
  ret void
}

define void @VS2R_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3) "enc"="{ if(vs3%2==0) 001000101000 rs1:5 000 vs3:5 0100111 }" "asm"="vs2r.v {vs3}, ({rs1})" "asmonly" {
  ret void
}

define void @VS4R_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3) "enc"="{ if(vs3%4==0) 011000101000 rs1:5 000 vs3:5 0100111 }" "asm"="vs4r.v {vs3}, ({rs1})" "asmonly" {
  ret void
}

define void @VS8R_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3) "enc"="{ if(vs3%8==0) 111000101000 rs1:5 000 vs3:5 0100111 }" "asm"="vs8r.v {vs3}, ({rs1})" "asmonly" {
  ret void
}

define void @VFADD_VF.0(i64 "reg"="d" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="0000001 vs2:5 rs1:5 101 vd:5 1010111" "asm"="vfadd.vf {vd}, {vs2}, {rs1}" "asmonly" {
  ret void
}

define void @VFADD_VF.1(i64 "reg"="d" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if(vd!=0) 0000000 vs2:5 rs1:5 101 vd:5 1010111 }" "asm"="vfadd.vf {vd}, {vs2}, {rs1}, v0.t" "asmonly" {
  ret void
}

define void @VFSUB_VF.0(i64 "reg"="d" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="0000101 vs2:5 rs1:5 101 vd:5 1010111" "asm"="vfsub.vf {vd}, {vs2}, {rs1}" "asmonly" {
  ret void
}

define void @VFSUB_VF.1(i64 "reg"="d" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if(vd!=0) 0000100 vs2:5 rs1:5 101 vd:5 1010111 }" "asm"="vfsub.vf {vd}, {vs2}, {rs1}, v0.t" "asmonly" {
  ret void
}

define void @VFMIN_VF.0(i64 "reg"="d" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="0001001 vs2:5 rs1:5 101 vd:5 1010111" "asm"="vfmin.vf {vd}, {vs2}, {rs1}" "asmonly" {
  ret void
}

define void @VFMIN_VF.1(i64 "reg"="d" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if(vd!=0) 0001000 vs2:5 rs1:5 101 vd:5 1010111 }" "asm"="vfmin.vf {vd}, {vs2}, {rs1}, v0.t" "asmonly" {
  ret void
}

define void @VFMAX_VF.0(i64 "reg"="d" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="0001101 vs2:5 rs1:5 101 vd:5 1010111" "asm"="vfmax.vf {vd}, {vs2}, {rs1}" "asmonly" {
  ret void
}

define void @VFMAX_VF.1(i64 "reg"="d" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if(vd!=0) 0001100 vs2:5 rs1:5 101 vd:5 1010111 }" "asm"="vfmax.vf {vd}, {vs2}, {rs1}, v0.t" "asmonly" {
  ret void
}

define void @VFSGNJ_VF.0(i64 "reg"="d" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="0010001 vs2:5 rs1:5 101 vd:5 1010111" "asm"="vfsgnj.vf {vd}, {vs2}, {rs1}" "asmonly" {
  ret void
}

define void @VFSGNJ_VF.1(i64 "reg"="d" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if(vd!=0) 0010000 vs2:5 rs1:5 101 vd:5 1010111 }" "asm"="vfsgnj.vf {vd}, {vs2}, {rs1}, v0.t" "asmonly" {
  ret void
}

define void @VFSGNJN_VF.0(i64 "reg"="d" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="0010011 vs2:5 rs1:5 101 vd:5 1010111" "asm"="vfsgnjn.vf {vd}, {vs2}, {rs1}" "asmonly" {
  ret void
}

define void @VFSGNJN_VF.1(i64 "reg"="d" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if(vd!=0) 0010010 vs2:5 rs1:5 101 vd:5 1010111 }" "asm"="vfsgnjn.vf {vd}, {vs2}, {rs1}, v0.t" "asmonly" {
  ret void
}

define void @VFSGNJX_VF.0(i64 "reg"="d" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="0010101 vs2:5 rs1:5 101 vd:5 1010111" "asm"="vfsgnjx.vf {vd}, {vs2}, {rs1}" "asmonly" {
  ret void
}

define void @VFSGNJX_VF.1(i64 "reg"="d" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if(vd!=0) 0010100 vs2:5 rs1:5 101 vd:5 1010111 }" "asm"="vfsgnjx.vf {vd}, {vs2}, {rs1}, v0.t" "asmonly" {
  ret void
}

define void @VFSLIDE1UP_VF.0(i64 "reg"="d" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if((vd<vs2)|(vd>=vs2+2)) 0011101 vs2:5 rs1:5 101 vd:5 1010111 }" "asm"="vfslide1up.vf {vd}, {vs2}, {rs1}" "asmonly" {
  ret void
}

define void @VFSLIDE1UP_VF.1(i64 "reg"="d" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if((vd<vs2)|(vd>=vs2+2)) if(vd!=0) 0011100 vs2:5 rs1:5 101 vd:5 1010111 }" "asm"="vfslide1up.vf {vd}, {vs2}, {rs1}, v0.t" "asmonly" {
  ret void
}

define void @VFSLIDE1DOWN_VF.0(i64 "reg"="d" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="0011111 vs2:5 rs1:5 101 vd:5 1010111" "asm"="vfslide1down.vf {vd}, {vs2}, {rs1}" "asmonly" {
  ret void
}

define void @VFSLIDE1DOWN_VF.1(i64 "reg"="d" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if(vd!=0) 0011110 vs2:5 rs1:5 101 vd:5 1010111 }" "asm"="vfslide1down.vf {vd}, {vs2}, {rs1}, v0.t" "asmonly" {
  ret void
}

define void @VFMV_S_F.0(i64 "reg"="d" %rs1, i64 "reg"="v" %vd) "enc"="010000100000 rs1:5 101 vd:5 1010111" "asm"="vfmv.s.f {vd}, {rs1}" "asmonly" {
  ret void
}

define void @VFMERGE_VFM.0(i64 "reg"="d" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if(vd!=0) 0101110 vs2:5 rs1:5 101 vd:5 1010111 }" "asm"="vfmerge.vfm {vd}, {vs2}, {rs1}, v0" "asmonly" {
  ret void
}

define void @VFMV_V_F.0(i64 "reg"="d" %rs1, i64 "reg"="v" %vd) "enc"="010111100000 rs1:5 101 vd:5 1010111" "asm"="vfmv.v.f {vd}, {rs1}" "asmonly" {
  ret void
}

define void @VMFEQ_VF.0(i64 "reg"="d" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="0110001 vs2:5 rs1:5 101 vd:5 1010111" "asm"="vmfeq.vf {vd}, {vs2}, {rs1}" "asmonly" {
  ret void
}

define void @VMFEQ_VF.1(i64 "reg"="d" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="0110000 vs2:5 rs1:5 101 vd:5 1010111" "asm"="vmfeq.vf {vd}, {vs2}, {rs1}, v0.t" "asmonly" {
  ret void
}

define void @VMFLE_VF.0(i64 "reg"="d" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="0110011 vs2:5 rs1:5 101 vd:5 1010111" "asm"="vmfle.vf {vd}, {vs2}, {rs1}" "asmonly" {
  ret void
}

define void @VMFLE_VF.1(i64 "reg"="d" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="0110010 vs2:5 rs1:5 101 vd:5 1010111" "asm"="vmfle.vf {vd}, {vs2}, {rs1}, v0.t" "asmonly" {
  ret void
}

define void @VMFLT_VF.0(i64 "reg"="d" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="0110111 vs2:5 rs1:5 101 vd:5 1010111" "asm"="vmflt.vf {vd}, {vs2}, {rs1}" "asmonly" {
  ret void
}

define void @VMFLT_VF.1(i64 "reg"="d" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="0110110 vs2:5 rs1:5 101 vd:5 1010111" "asm"="vmflt.vf {vd}, {vs2}, {rs1}, v0.t" "asmonly" {
  ret void
}

define void @VMFNE_VF.0(i64 "reg"="d" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="0111001 vs2:5 rs1:5 101 vd:5 1010111" "asm"="vmfne.vf {vd}, {vs2}, {rs1}" "asmonly" {
  ret void
}

define void @VMFNE_VF.1(i64 "reg"="d" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="0111000 vs2:5 rs1:5 101 vd:5 1010111" "asm"="vmfne.vf {vd}, {vs2}, {rs1}, v0.t" "asmonly" {
  ret void
}

define void @VMFGT_VF.0(i64 "reg"="d" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="0111011 vs2:5 rs1:5 101 vd:5 1010111" "asm"="vmfgt.vf {vd}, {vs2}, {rs1}" "asmonly" {
  ret void
}

define void @VMFGT_VF.1(i64 "reg"="d" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="0111010 vs2:5 rs1:5 101 vd:5 1010111" "asm"="vmfgt.vf {vd}, {vs2}, {rs1}, v0.t" "asmonly" {
  ret void
}

define void @VMFGE_VF.0(i64 "reg"="d" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="0111111 vs2:5 rs1:5 101 vd:5 1010111" "asm"="vmfge.vf {vd}, {vs2}, {rs1}" "asmonly" {
  ret void
}

define void @VMFGE_VF.1(i64 "reg"="d" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="0111110 vs2:5 rs1:5 101 vd:5 1010111" "asm"="vmfge.vf {vd}, {vs2}, {rs1}, v0.t" "asmonly" {
  ret void
}

define void @VFDIV_VF.0(i64 "reg"="d" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="1000001 vs2:5 rs1:5 101 vd:5 1010111" "asm"="vfdiv.vf {vd}, {vs2}, {rs1}" "asmonly" {
  ret void
}

define void @VFDIV_VF.1(i64 "reg"="d" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if(vd!=0) 1000000 vs2:5 rs1:5 101 vd:5 1010111 }" "asm"="vfdiv.vf {vd}, {vs2}, {rs1}, v0.t" "asmonly" {
  ret void
}

define void @VFRDIV_VF.0(i64 "reg"="d" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="1000011 vs2:5 rs1:5 101 vd:5 1010111" "asm"="vfrdiv.vf {vd}, {vs2}, {rs1}" "asmonly" {
  ret void
}

define void @VFRDIV_VF.1(i64 "reg"="d" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if(vd!=0) 1000010 vs2:5 rs1:5 101 vd:5 1010111 }" "asm"="vfrdiv.vf {vd}, {vs2}, {rs1}, v0.t" "asmonly" {
  ret void
}

define void @VFMUL_VF.0(i64 "reg"="d" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="1001001 vs2:5 rs1:5 101 vd:5 1010111" "asm"="vfmul.vf {vd}, {vs2}, {rs1}" "asmonly" {
  ret void
}

define void @VFMUL_VF.1(i64 "reg"="d" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if(vd!=0) 1001000 vs2:5 rs1:5 101 vd:5 1010111 }" "asm"="vfmul.vf {vd}, {vs2}, {rs1}, v0.t" "asmonly" {
  ret void
}

define void @VFRSUB_VF.0(i64 "reg"="d" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="1001111 vs2:5 rs1:5 101 vd:5 1010111" "asm"="vfrsub.vf {vd}, {vs2}, {rs1}" "asmonly" {
  ret void
}

define void @VFRSUB_VF.1(i64 "reg"="d" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if(vd!=0) 1001110 vs2:5 rs1:5 101 vd:5 1010111 }" "asm"="vfrsub.vf {vd}, {vs2}, {rs1}, v0.t" "asmonly" {
  ret void
}

define void @VFMADD_VF.0(i64 "reg"="d" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="1010001 vs2:5 rs1:5 101 vd:5 1010111" "asm"="vfmadd.vf {vd}, {rs1}, {vs2}" "asmonly" {
  ret void
}

define void @VFMADD_VF.1(i64 "reg"="d" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if(vd!=0) 1010000 vs2:5 rs1:5 101 vd:5 1010111 }" "asm"="vfmadd.vf {vd}, {rs1}, {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VFNMADD_VF.0(i64 "reg"="d" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="1010011 vs2:5 rs1:5 101 vd:5 1010111" "asm"="vfnmadd.vf {vd}, {rs1}, {vs2}" "asmonly" {
  ret void
}

define void @VFNMADD_VF.1(i64 "reg"="d" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if(vd!=0) 1010010 vs2:5 rs1:5 101 vd:5 1010111 }" "asm"="vfnmadd.vf {vd}, {rs1}, {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VFMSUB_VF.0(i64 "reg"="d" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="1010101 vs2:5 rs1:5 101 vd:5 1010111" "asm"="vfmsub.vf {vd}, {rs1}, {vs2}" "asmonly" {
  ret void
}

define void @VFMSUB_VF.1(i64 "reg"="d" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if(vd!=0) 1010100 vs2:5 rs1:5 101 vd:5 1010111 }" "asm"="vfmsub.vf {vd}, {rs1}, {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VFNMSUB_VF.0(i64 "reg"="d" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="1010111 vs2:5 rs1:5 101 vd:5 1010111" "asm"="vfnmsub.vf {vd}, {rs1}, {vs2}" "asmonly" {
  ret void
}

define void @VFNMSUB_VF.1(i64 "reg"="d" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if(vd!=0) 1010110 vs2:5 rs1:5 101 vd:5 1010111 }" "asm"="vfnmsub.vf {vd}, {rs1}, {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VFMACC_VF.0(i64 "reg"="d" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="1011001 vs2:5 rs1:5 101 vd:5 1010111" "asm"="vfmacc.vf {vd}, {rs1}, {vs2}" "asmonly" {
  ret void
}

define void @VFMACC_VF.1(i64 "reg"="d" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if(vd!=0) 1011000 vs2:5 rs1:5 101 vd:5 1010111 }" "asm"="vfmacc.vf {vd}, {rs1}, {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VFNMACC_VF.0(i64 "reg"="d" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="1011011 vs2:5 rs1:5 101 vd:5 1010111" "asm"="vfnmacc.vf {vd}, {rs1}, {vs2}" "asmonly" {
  ret void
}

define void @VFNMACC_VF.1(i64 "reg"="d" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if(vd!=0) 1011010 vs2:5 rs1:5 101 vd:5 1010111 }" "asm"="vfnmacc.vf {vd}, {rs1}, {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VFMSAC_VF.0(i64 "reg"="d" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="1011101 vs2:5 rs1:5 101 vd:5 1010111" "asm"="vfmsac.vf {vd}, {rs1}, {vs2}" "asmonly" {
  ret void
}

define void @VFMSAC_VF.1(i64 "reg"="d" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if(vd!=0) 1011100 vs2:5 rs1:5 101 vd:5 1010111 }" "asm"="vfmsac.vf {vd}, {rs1}, {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VFNMSAC_VF.0(i64 "reg"="d" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="1011111 vs2:5 rs1:5 101 vd:5 1010111" "asm"="vfnmsac.vf {vd}, {rs1}, {vs2}" "asmonly" {
  ret void
}

define void @VFNMSAC_VF.1(i64 "reg"="d" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if(vd!=0) 1011110 vs2:5 rs1:5 101 vd:5 1010111 }" "asm"="vfnmsac.vf {vd}, {rs1}, {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VFWADD_VF.0(i64 "reg"="d" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if((vd<vs2)|(vd>=vs2+2)) 1100001 vs2:5 rs1:5 101 vd:5 1010111 }" "asm"="vfwadd.vf {vd}, {vs2}, {rs1}" "asmonly" {
  ret void
}

define void @VFWADD_VF.1(i64 "reg"="d" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if((vd<vs2)|(vd>=vs2+2)) if(vd!=0) 1100000 vs2:5 rs1:5 101 vd:5 1010111 }" "asm"="vfwadd.vf {vd}, {vs2}, {rs1}, v0.t" "asmonly" {
  ret void
}

define void @VFWSUB_VF.0(i64 "reg"="d" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if((vd<vs2)|(vd>=vs2+2)) 1100101 vs2:5 rs1:5 101 vd:5 1010111 }" "asm"="vfwsub.vf {vd}, {vs2}, {rs1}" "asmonly" {
  ret void
}

define void @VFWSUB_VF.1(i64 "reg"="d" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if((vd<vs2)|(vd>=vs2+2)) if(vd!=0) 1100100 vs2:5 rs1:5 101 vd:5 1010111 }" "asm"="vfwsub.vf {vd}, {vs2}, {rs1}, v0.t" "asmonly" {
  ret void
}

define void @VFWADD_WF.0(i64 "reg"="d" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="1101001 vs2:5 rs1:5 101 vd:5 1010111" "asm"="vfwadd.wf {vd}, {vs2}, {rs1}" "asmonly" {
  ret void
}

define void @VFWADD_WF.1(i64 "reg"="d" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if(vd!=0) 1101000 vs2:5 rs1:5 101 vd:5 1010111 }" "asm"="vfwadd.wf {vd}, {vs2}, {rs1}, v0.t" "asmonly" {
  ret void
}

define void @VFWSUB_WF.0(i64 "reg"="d" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="1101101 vs2:5 rs1:5 101 vd:5 1010111" "asm"="vfwsub.wf {vd}, {vs2}, {rs1}" "asmonly" {
  ret void
}

define void @VFWSUB_WF.1(i64 "reg"="d" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if(vd!=0) 1101100 vs2:5 rs1:5 101 vd:5 1010111 }" "asm"="vfwsub.wf {vd}, {vs2}, {rs1}, v0.t" "asmonly" {
  ret void
}

define void @VFWMUL_VF.0(i64 "reg"="d" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if((vd<vs2)|(vd>=vs2+2)) 1110001 vs2:5 rs1:5 101 vd:5 1010111 }" "asm"="vfwmul.vf {vd}, {vs2}, {rs1}" "asmonly" {
  ret void
}

define void @VFWMUL_VF.1(i64 "reg"="d" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if((vd<vs2)|(vd>=vs2+2)) if(vd!=0) 1110000 vs2:5 rs1:5 101 vd:5 1010111 }" "asm"="vfwmul.vf {vd}, {vs2}, {rs1}, v0.t" "asmonly" {
  ret void
}

define void @VFWMACC_VF.0(i64 "reg"="d" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if((vd<vs2)|(vd>=vs2+2)) 1111001 vs2:5 rs1:5 101 vd:5 1010111 }" "asm"="vfwmacc.vf {vd}, {rs1}, {vs2}" "asmonly" {
  ret void
}

define void @VFWMACC_VF.1(i64 "reg"="d" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if((vd<vs2)|(vd>=vs2+2)) if(vd!=0) 1111000 vs2:5 rs1:5 101 vd:5 1010111 }" "asm"="vfwmacc.vf {vd}, {rs1}, {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VFWNMACC_VF.0(i64 "reg"="d" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if((vd<vs2)|(vd>=vs2+2)) 1111011 vs2:5 rs1:5 101 vd:5 1010111 }" "asm"="vfwnmacc.vf {vd}, {rs1}, {vs2}" "asmonly" {
  ret void
}

define void @VFWNMACC_VF.1(i64 "reg"="d" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if((vd<vs2)|(vd>=vs2+2)) if(vd!=0) 1111010 vs2:5 rs1:5 101 vd:5 1010111 }" "asm"="vfwnmacc.vf {vd}, {rs1}, {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VFWMSAC_VF.0(i64 "reg"="d" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if((vd<vs2)|(vd>=vs2+2)) 1111101 vs2:5 rs1:5 101 vd:5 1010111 }" "asm"="vfwmsac.vf {vd}, {rs1}, {vs2}" "asmonly" {
  ret void
}

define void @VFWMSAC_VF.1(i64 "reg"="d" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if((vd<vs2)|(vd>=vs2+2)) if(vd!=0) 1111100 vs2:5 rs1:5 101 vd:5 1010111 }" "asm"="vfwmsac.vf {vd}, {rs1}, {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VFWNMSAC_VF.0(i64 "reg"="d" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if((vd<vs2)|(vd>=vs2+2)) 1111111 vs2:5 rs1:5 101 vd:5 1010111 }" "asm"="vfwnmsac.vf {vd}, {rs1}, {vs2}" "asmonly" {
  ret void
}

define void @VFWNMSAC_VF.1(i64 "reg"="d" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if((vd<vs2)|(vd>=vs2+2)) if(vd!=0) 1111110 vs2:5 rs1:5 101 vd:5 1010111 }" "asm"="vfwnmsac.vf {vd}, {rs1}, {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VFADD_VV.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="0000001 vs2:5 vs1:5 001 vd:5 1010111" "asm"="vfadd.vv {vd}, {vs2}, {vs1}" "asmonly" {
  ret void
}

define void @VFADD_VV.1(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="{ if(vd!=0) 0000000 vs2:5 vs1:5 001 vd:5 1010111 }" "asm"="vfadd.vv {vd}, {vs2}, {vs1}, v0.t" "asmonly" {
  ret void
}

define void @VFREDUSUM_VS.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="0000011 vs2:5 vs1:5 001 vd:5 1010111" "asm"="vfredusum.vs {vd}, {vs2}, {vs1}" "asmonly" {
  ret void
}

define void @VFREDUSUM_VS.1(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="0000010 vs2:5 vs1:5 001 vd:5 1010111" "asm"="vfredusum.vs {vd}, {vs2}, {vs1}, v0.t" "asmonly" {
  ret void
}

define void @VFSUB_VV.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="0000101 vs2:5 vs1:5 001 vd:5 1010111" "asm"="vfsub.vv {vd}, {vs2}, {vs1}" "asmonly" {
  ret void
}

define void @VFSUB_VV.1(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="{ if(vd!=0) 0000100 vs2:5 vs1:5 001 vd:5 1010111 }" "asm"="vfsub.vv {vd}, {vs2}, {vs1}, v0.t" "asmonly" {
  ret void
}

define void @VFREDOSUM_VS.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="0000111 vs2:5 vs1:5 001 vd:5 1010111" "asm"="vfredosum.vs {vd}, {vs2}, {vs1}" "asmonly" {
  ret void
}

define void @VFREDOSUM_VS.1(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="0000110 vs2:5 vs1:5 001 vd:5 1010111" "asm"="vfredosum.vs {vd}, {vs2}, {vs1}, v0.t" "asmonly" {
  ret void
}

define void @VFMIN_VV.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="0001001 vs2:5 vs1:5 001 vd:5 1010111" "asm"="vfmin.vv {vd}, {vs2}, {vs1}" "asmonly" {
  ret void
}

define void @VFMIN_VV.1(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="{ if(vd!=0) 0001000 vs2:5 vs1:5 001 vd:5 1010111 }" "asm"="vfmin.vv {vd}, {vs2}, {vs1}, v0.t" "asmonly" {
  ret void
}

define void @VFREDMIN_VS.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="0001011 vs2:5 vs1:5 001 vd:5 1010111" "asm"="vfredmin.vs {vd}, {vs2}, {vs1}" "asmonly" {
  ret void
}

define void @VFREDMIN_VS.1(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="0001010 vs2:5 vs1:5 001 vd:5 1010111" "asm"="vfredmin.vs {vd}, {vs2}, {vs1}, v0.t" "asmonly" {
  ret void
}

define void @VFMAX_VV.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="0001101 vs2:5 vs1:5 001 vd:5 1010111" "asm"="vfmax.vv {vd}, {vs2}, {vs1}" "asmonly" {
  ret void
}

define void @VFMAX_VV.1(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="{ if(vd!=0) 0001100 vs2:5 vs1:5 001 vd:5 1010111 }" "asm"="vfmax.vv {vd}, {vs2}, {vs1}, v0.t" "asmonly" {
  ret void
}

define void @VFREDMAX_VS.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="0001111 vs2:5 vs1:5 001 vd:5 1010111" "asm"="vfredmax.vs {vd}, {vs2}, {vs1}" "asmonly" {
  ret void
}

define void @VFREDMAX_VS.1(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="0001110 vs2:5 vs1:5 001 vd:5 1010111" "asm"="vfredmax.vs {vd}, {vs2}, {vs1}, v0.t" "asmonly" {
  ret void
}

define void @VFSGNJ_VV.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="0010001 vs2:5 vs1:5 001 vd:5 1010111" "asm"="vfsgnj.vv {vd}, {vs2}, {vs1}" "asmonly" {
  ret void
}

define void @VFSGNJ_VV.1(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="{ if(vd!=0) 0010000 vs2:5 vs1:5 001 vd:5 1010111 }" "asm"="vfsgnj.vv {vd}, {vs2}, {vs1}, v0.t" "asmonly" {
  ret void
}

define void @VFSGNJN_VV.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="0010011 vs2:5 vs1:5 001 vd:5 1010111" "asm"="vfsgnjn.vv {vd}, {vs2}, {vs1}" "asmonly" {
  ret void
}

define void @VFSGNJN_VV.1(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="{ if(vd!=0) 0010010 vs2:5 vs1:5 001 vd:5 1010111 }" "asm"="vfsgnjn.vv {vd}, {vs2}, {vs1}, v0.t" "asmonly" {
  ret void
}

define void @VFSGNJX_VV.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="0010101 vs2:5 vs1:5 001 vd:5 1010111" "asm"="vfsgnjx.vv {vd}, {vs2}, {vs1}" "asmonly" {
  ret void
}

define void @VFSGNJX_VV.1(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="{ if(vd!=0) 0010100 vs2:5 vs1:5 001 vd:5 1010111 }" "asm"="vfsgnjx.vv {vd}, {vs2}, {vs1}, v0.t" "asmonly" {
  ret void
}

define void @VFMV_F_S.0(i64 "reg"="d" %rd, i64 "reg"="v" %vs2) "enc"="0100001 vs2:5 00000001 rd:5 1010111" "asm"="vfmv.f.s {rd}, {vs2}" "asmonly" {
  ret void
}

define void @VMFEQ_VV.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="0110001 vs2:5 vs1:5 001 vd:5 1010111" "asm"="vmfeq.vv {vd}, {vs2}, {vs1}" "asmonly" {
  ret void
}

define void @VMFEQ_VV.1(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="0110000 vs2:5 vs1:5 001 vd:5 1010111" "asm"="vmfeq.vv {vd}, {vs2}, {vs1}, v0.t" "asmonly" {
  ret void
}

define void @VMFLE_VV.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="0110011 vs2:5 vs1:5 001 vd:5 1010111" "asm"="vmfle.vv {vd}, {vs2}, {vs1}" "asmonly" {
  ret void
}

define void @VMFLE_VV.1(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="0110010 vs2:5 vs1:5 001 vd:5 1010111" "asm"="vmfle.vv {vd}, {vs2}, {vs1}, v0.t" "asmonly" {
  ret void
}

define void @VMFLT_VV.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="0110111 vs2:5 vs1:5 001 vd:5 1010111" "asm"="vmflt.vv {vd}, {vs2}, {vs1}" "asmonly" {
  ret void
}

define void @VMFLT_VV.1(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="0110110 vs2:5 vs1:5 001 vd:5 1010111" "asm"="vmflt.vv {vd}, {vs2}, {vs1}, v0.t" "asmonly" {
  ret void
}

define void @VMFNE_VV.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="0111001 vs2:5 vs1:5 001 vd:5 1010111" "asm"="vmfne.vv {vd}, {vs2}, {vs1}" "asmonly" {
  ret void
}

define void @VMFNE_VV.1(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="0111000 vs2:5 vs1:5 001 vd:5 1010111" "asm"="vmfne.vv {vd}, {vs2}, {vs1}, v0.t" "asmonly" {
  ret void
}

define void @VFDIV_VV.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="1000001 vs2:5 vs1:5 001 vd:5 1010111" "asm"="vfdiv.vv {vd}, {vs2}, {vs1}" "asmonly" {
  ret void
}

define void @VFDIV_VV.1(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="{ if(vd!=0) 1000000 vs2:5 vs1:5 001 vd:5 1010111 }" "asm"="vfdiv.vv {vd}, {vs2}, {vs1}, v0.t" "asmonly" {
  ret void
}

define void @VFMUL_VV.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="1001001 vs2:5 vs1:5 001 vd:5 1010111" "asm"="vfmul.vv {vd}, {vs2}, {vs1}" "asmonly" {
  ret void
}

define void @VFMUL_VV.1(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="{ if(vd!=0) 1001000 vs2:5 vs1:5 001 vd:5 1010111 }" "asm"="vfmul.vv {vd}, {vs2}, {vs1}, v0.t" "asmonly" {
  ret void
}

define void @VFMADD_VV.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="1010001 vs2:5 vs1:5 001 vd:5 1010111" "asm"="vfmadd.vv {vd}, {vs1}, {vs2}" "asmonly" {
  ret void
}

define void @VFMADD_VV.1(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="{ if(vd!=0) 1010000 vs2:5 vs1:5 001 vd:5 1010111 }" "asm"="vfmadd.vv {vd}, {vs1}, {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VFNMADD_VV.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="1010011 vs2:5 vs1:5 001 vd:5 1010111" "asm"="vfnmadd.vv {vd}, {vs1}, {vs2}" "asmonly" {
  ret void
}

define void @VFNMADD_VV.1(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="{ if(vd!=0) 1010010 vs2:5 vs1:5 001 vd:5 1010111 }" "asm"="vfnmadd.vv {vd}, {vs1}, {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VFMSUB_VV.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="1010101 vs2:5 vs1:5 001 vd:5 1010111" "asm"="vfmsub.vv {vd}, {vs1}, {vs2}" "asmonly" {
  ret void
}

define void @VFMSUB_VV.1(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="{ if(vd!=0) 1010100 vs2:5 vs1:5 001 vd:5 1010111 }" "asm"="vfmsub.vv {vd}, {vs1}, {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VFNMSUB_VV.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="1010111 vs2:5 vs1:5 001 vd:5 1010111" "asm"="vfnmsub.vv {vd}, {vs1}, {vs2}" "asmonly" {
  ret void
}

define void @VFNMSUB_VV.1(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="{ if(vd!=0) 1010110 vs2:5 vs1:5 001 vd:5 1010111 }" "asm"="vfnmsub.vv {vd}, {vs1}, {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VFMACC_VV.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="1011001 vs2:5 vs1:5 001 vd:5 1010111" "asm"="vfmacc.vv {vd}, {vs1}, {vs2}" "asmonly" {
  ret void
}

define void @VFMACC_VV.1(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="{ if(vd!=0) 1011000 vs2:5 vs1:5 001 vd:5 1010111 }" "asm"="vfmacc.vv {vd}, {vs1}, {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VFNMACC_VV.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="1011011 vs2:5 vs1:5 001 vd:5 1010111" "asm"="vfnmacc.vv {vd}, {vs1}, {vs2}" "asmonly" {
  ret void
}

define void @VFNMACC_VV.1(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="{ if(vd!=0) 1011010 vs2:5 vs1:5 001 vd:5 1010111 }" "asm"="vfnmacc.vv {vd}, {vs1}, {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VFMSAC_VV.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="1011101 vs2:5 vs1:5 001 vd:5 1010111" "asm"="vfmsac.vv {vd}, {vs1}, {vs2}" "asmonly" {
  ret void
}

define void @VFMSAC_VV.1(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="{ if(vd!=0) 1011100 vs2:5 vs1:5 001 vd:5 1010111 }" "asm"="vfmsac.vv {vd}, {vs1}, {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VFNMSAC_VV.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="1011111 vs2:5 vs1:5 001 vd:5 1010111" "asm"="vfnmsac.vv {vd}, {vs1}, {vs2}" "asmonly" {
  ret void
}

define void @VFNMSAC_VV.1(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="{ if(vd!=0) 1011110 vs2:5 vs1:5 001 vd:5 1010111 }" "asm"="vfnmsac.vv {vd}, {vs1}, {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VFCVT_XU_F_V.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="0100101 vs2:5 00000001 vd:5 1010111" "asm"="vfcvt.xu.f.v {vd}, {vs2}" "asmonly" {
  ret void
}

define void @VFCVT_XU_F_V.1(i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if(vd!=0) 0100100 vs2:5 00000001 vd:5 1010111 }" "asm"="vfcvt.xu.f.v {vd}, {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VFCVT_X_F_V.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="0100101 vs2:5 00001001 vd:5 1010111" "asm"="vfcvt.x.f.v {vd}, {vs2}" "asmonly" {
  ret void
}

define void @VFCVT_X_F_V.1(i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if(vd!=0) 0100100 vs2:5 00001001 vd:5 1010111 }" "asm"="vfcvt.x.f.v {vd}, {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VFCVT_F_XU_V.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="0100101 vs2:5 00010001 vd:5 1010111" "asm"="vfcvt.f.xu.v {vd}, {vs2}" "asmonly" {
  ret void
}

define void @VFCVT_F_XU_V.1(i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if(vd!=0) 0100100 vs2:5 00010001 vd:5 1010111 }" "asm"="vfcvt.f.xu.v {vd}, {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VFCVT_F_X_V.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="0100101 vs2:5 00011001 vd:5 1010111" "asm"="vfcvt.f.x.v {vd}, {vs2}" "asmonly" {
  ret void
}

define void @VFCVT_F_X_V.1(i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if(vd!=0) 0100100 vs2:5 00011001 vd:5 1010111 }" "asm"="vfcvt.f.x.v {vd}, {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VFCVT_RTZ_XU_F_V.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="0100101 vs2:5 00110001 vd:5 1010111" "asm"="vfcvt.rtz.xu.f.v {vd}, {vs2}" "asmonly" {
  ret void
}

define void @VFCVT_RTZ_XU_F_V.1(i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if(vd!=0) 0100100 vs2:5 00110001 vd:5 1010111 }" "asm"="vfcvt.rtz.xu.f.v {vd}, {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VFCVT_RTZ_X_F_V.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="0100101 vs2:5 00111001 vd:5 1010111" "asm"="vfcvt.rtz.x.f.v {vd}, {vs2}" "asmonly" {
  ret void
}

define void @VFCVT_RTZ_X_F_V.1(i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if(vd!=0) 0100100 vs2:5 00111001 vd:5 1010111 }" "asm"="vfcvt.rtz.x.f.v {vd}, {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VFWCVT_XU_F_V.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if((vd<vs2)|(vd>=vs2+2)) 0100101 vs2:5 01000001 vd:5 1010111 }" "asm"="vfwcvt.xu.f.v {vd}, {vs2}" "asmonly" {
  ret void
}

define void @VFWCVT_XU_F_V.1(i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if((vd<vs2)|(vd>=vs2+2)) if(vd!=0) 0100100 vs2:5 01000001 vd:5 1010111 }" "asm"="vfwcvt.xu.f.v {vd}, {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VFWCVT_X_F_V.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if((vd<vs2)|(vd>=vs2+2)) 0100101 vs2:5 01001001 vd:5 1010111 }" "asm"="vfwcvt.x.f.v {vd}, {vs2}" "asmonly" {
  ret void
}

define void @VFWCVT_X_F_V.1(i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if((vd<vs2)|(vd>=vs2+2)) if(vd!=0) 0100100 vs2:5 01001001 vd:5 1010111 }" "asm"="vfwcvt.x.f.v {vd}, {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VFWCVT_F_XU_V.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if((vd<vs2)|(vd>=vs2+2)) 0100101 vs2:5 01010001 vd:5 1010111 }" "asm"="vfwcvt.f.xu.v {vd}, {vs2}" "asmonly" {
  ret void
}

define void @VFWCVT_F_XU_V.1(i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if((vd<vs2)|(vd>=vs2+2)) if(vd!=0) 0100100 vs2:5 01010001 vd:5 1010111 }" "asm"="vfwcvt.f.xu.v {vd}, {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VFWCVT_F_X_V.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if((vd<vs2)|(vd>=vs2+2)) 0100101 vs2:5 01011001 vd:5 1010111 }" "asm"="vfwcvt.f.x.v {vd}, {vs2}" "asmonly" {
  ret void
}

define void @VFWCVT_F_X_V.1(i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if((vd<vs2)|(vd>=vs2+2)) if(vd!=0) 0100100 vs2:5 01011001 vd:5 1010111 }" "asm"="vfwcvt.f.x.v {vd}, {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VFWCVT_F_F_V.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if((vd<vs2)|(vd>=vs2+2)) 0100101 vs2:5 01100001 vd:5 1010111 }" "asm"="vfwcvt.f.f.v {vd}, {vs2}" "asmonly" {
  ret void
}

define void @VFWCVT_F_F_V.1(i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if((vd<vs2)|(vd>=vs2+2)) if(vd!=0) 0100100 vs2:5 01100001 vd:5 1010111 }" "asm"="vfwcvt.f.f.v {vd}, {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VFWCVT_RTZ_XU_F_V.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if((vd<vs2)|(vd>=vs2+2)) 0100101 vs2:5 01110001 vd:5 1010111 }" "asm"="vfwcvt.rtz.xu.f.v {vd}, {vs2}" "asmonly" {
  ret void
}

define void @VFWCVT_RTZ_XU_F_V.1(i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if((vd<vs2)|(vd>=vs2+2)) if(vd!=0) 0100100 vs2:5 01110001 vd:5 1010111 }" "asm"="vfwcvt.rtz.xu.f.v {vd}, {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VFWCVT_RTZ_X_F_V.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if((vd<vs2)|(vd>=vs2+2)) 0100101 vs2:5 01111001 vd:5 1010111 }" "asm"="vfwcvt.rtz.x.f.v {vd}, {vs2}" "asmonly" {
  ret void
}

define void @VFWCVT_RTZ_X_F_V.1(i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if((vd<vs2)|(vd>=vs2+2)) if(vd!=0) 0100100 vs2:5 01111001 vd:5 1010111 }" "asm"="vfwcvt.rtz.x.f.v {vd}, {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VFNCVT_XU_F_W.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="0100101 vs2:5 10000001 vd:5 1010111" "asm"="vfncvt.xu.f.w {vd}, {vs2}" "asmonly" {
  ret void
}

define void @VFNCVT_XU_F_W.1(i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if(vd!=0) 0100100 vs2:5 10000001 vd:5 1010111 }" "asm"="vfncvt.xu.f.w {vd}, {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VFNCVT_X_F_W.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="0100101 vs2:5 10001001 vd:5 1010111" "asm"="vfncvt.x.f.w {vd}, {vs2}" "asmonly" {
  ret void
}

define void @VFNCVT_X_F_W.1(i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if(vd!=0) 0100100 vs2:5 10001001 vd:5 1010111 }" "asm"="vfncvt.x.f.w {vd}, {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VFNCVT_F_XU_W.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="0100101 vs2:5 10010001 vd:5 1010111" "asm"="vfncvt.f.xu.w {vd}, {vs2}" "asmonly" {
  ret void
}

define void @VFNCVT_F_XU_W.1(i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if(vd!=0) 0100100 vs2:5 10010001 vd:5 1010111 }" "asm"="vfncvt.f.xu.w {vd}, {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VFNCVT_F_X_W.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="0100101 vs2:5 10011001 vd:5 1010111" "asm"="vfncvt.f.x.w {vd}, {vs2}" "asmonly" {
  ret void
}

define void @VFNCVT_F_X_W.1(i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if(vd!=0) 0100100 vs2:5 10011001 vd:5 1010111 }" "asm"="vfncvt.f.x.w {vd}, {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VFNCVT_F_F_W.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="0100101 vs2:5 10100001 vd:5 1010111" "asm"="vfncvt.f.f.w {vd}, {vs2}" "asmonly" {
  ret void
}

define void @VFNCVT_F_F_W.1(i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if(vd!=0) 0100100 vs2:5 10100001 vd:5 1010111 }" "asm"="vfncvt.f.f.w {vd}, {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VFNCVT_ROD_F_F_W.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="0100101 vs2:5 10101001 vd:5 1010111" "asm"="vfncvt.rod.f.f.w {vd}, {vs2}" "asmonly" {
  ret void
}

define void @VFNCVT_ROD_F_F_W.1(i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if(vd!=0) 0100100 vs2:5 10101001 vd:5 1010111 }" "asm"="vfncvt.rod.f.f.w {vd}, {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VFNCVT_RTZ_XU_F_W.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="0100101 vs2:5 10110001 vd:5 1010111" "asm"="vfncvt.rtz.xu.f.w {vd}, {vs2}" "asmonly" {
  ret void
}

define void @VFNCVT_RTZ_XU_F_W.1(i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if(vd!=0) 0100100 vs2:5 10110001 vd:5 1010111 }" "asm"="vfncvt.rtz.xu.f.w {vd}, {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VFNCVT_RTZ_X_F_W.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="0100101 vs2:5 10111001 vd:5 1010111" "asm"="vfncvt.rtz.x.f.w {vd}, {vs2}" "asmonly" {
  ret void
}

define void @VFNCVT_RTZ_X_F_W.1(i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if(vd!=0) 0100100 vs2:5 10111001 vd:5 1010111 }" "asm"="vfncvt.rtz.x.f.w {vd}, {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VFSQRT_V.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="0100111 vs2:5 00000001 vd:5 1010111" "asm"="vfsqrt.v {vd}, {vs2}" "asmonly" {
  ret void
}

define void @VFSQRT_V.1(i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if(vd!=0) 0100110 vs2:5 00000001 vd:5 1010111 }" "asm"="vfsqrt.v {vd}, {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VFRSQRT7_V.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="0100111 vs2:5 00100001 vd:5 1010111" "asm"="vfrsqrt7.v {vd}, {vs2}" "asmonly" {
  ret void
}

define void @VFRSQRT7_V.1(i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if(vd!=0) 0100110 vs2:5 00100001 vd:5 1010111 }" "asm"="vfrsqrt7.v {vd}, {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VFREC7_V.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="0100111 vs2:5 00101001 vd:5 1010111" "asm"="vfrec7.v {vd}, {vs2}" "asmonly" {
  ret void
}

define void @VFREC7_V.1(i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if(vd!=0) 0100110 vs2:5 00101001 vd:5 1010111 }" "asm"="vfrec7.v {vd}, {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VFCLASS_V.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="0100111 vs2:5 10000001 vd:5 1010111" "asm"="vfclass.v {vd}, {vs2}" "asmonly" {
  ret void
}

define void @VFCLASS_V.1(i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if(vd!=0) 0100110 vs2:5 10000001 vd:5 1010111 }" "asm"="vfclass.v {vd}, {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VFWADD_VV.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="{ if((vd<vs1)|(vd>=vs1+2)) if((vd<vs2)|(vd>=vs2+2)) 1100001 vs2:5 vs1:5 001 vd:5 1010111 }" "asm"="vfwadd.vv {vd}, {vs2}, {vs1}" "asmonly" {
  ret void
}

define void @VFWADD_VV.1(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="{ if((vd<vs1)|(vd>=vs1+2)) if((vd<vs2)|(vd>=vs2+2)) if(vd!=0) 1100000 vs2:5 vs1:5 001 vd:5 1010111 }" "asm"="vfwadd.vv {vd}, {vs2}, {vs1}, v0.t" "asmonly" {
  ret void
}

define void @VFWREDUSUM_VS.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="1100011 vs2:5 vs1:5 001 vd:5 1010111" "asm"="vfwredusum.vs {vd}, {vs2}, {vs1}" "asmonly" {
  ret void
}

define void @VFWREDUSUM_VS.1(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="1100010 vs2:5 vs1:5 001 vd:5 1010111" "asm"="vfwredusum.vs {vd}, {vs2}, {vs1}, v0.t" "asmonly" {
  ret void
}

define void @VFWSUB_VV.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="{ if((vd<vs1)|(vd>=vs1+2)) if((vd<vs2)|(vd>=vs2+2)) 1100101 vs2:5 vs1:5 001 vd:5 1010111 }" "asm"="vfwsub.vv {vd}, {vs2}, {vs1}" "asmonly" {
  ret void
}

define void @VFWSUB_VV.1(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="{ if((vd<vs1)|(vd>=vs1+2)) if((vd<vs2)|(vd>=vs2+2)) if(vd!=0) 1100100 vs2:5 vs1:5 001 vd:5 1010111 }" "asm"="vfwsub.vv {vd}, {vs2}, {vs1}, v0.t" "asmonly" {
  ret void
}

define void @VFWREDOSUM_VS.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="1100111 vs2:5 vs1:5 001 vd:5 1010111" "asm"="vfwredosum.vs {vd}, {vs2}, {vs1}" "asmonly" {
  ret void
}

define void @VFWREDOSUM_VS.1(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="1100110 vs2:5 vs1:5 001 vd:5 1010111" "asm"="vfwredosum.vs {vd}, {vs2}, {vs1}, v0.t" "asmonly" {
  ret void
}

define void @VFWADD_WV.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="{ if((vd<vs1)|(vd>=vs1+2)) 1101001 vs2:5 vs1:5 001 vd:5 1010111 }" "asm"="vfwadd.wv {vd}, {vs2}, {vs1}" "asmonly" {
  ret void
}

define void @VFWADD_WV.1(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="{ if((vd<vs1)|(vd>=vs1+2)) if(vd!=0) 1101000 vs2:5 vs1:5 001 vd:5 1010111 }" "asm"="vfwadd.wv {vd}, {vs2}, {vs1}, v0.t" "asmonly" {
  ret void
}

define void @VFWSUB_WV.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="{ if((vd<vs1)|(vd>=vs1+2)) 1101101 vs2:5 vs1:5 001 vd:5 1010111 }" "asm"="vfwsub.wv {vd}, {vs2}, {vs1}" "asmonly" {
  ret void
}

define void @VFWSUB_WV.1(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="{ if((vd<vs1)|(vd>=vs1+2)) if(vd!=0) 1101100 vs2:5 vs1:5 001 vd:5 1010111 }" "asm"="vfwsub.wv {vd}, {vs2}, {vs1}, v0.t" "asmonly" {
  ret void
}

define void @VFWMUL_VV.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="{ if((vd<vs1)|(vd>=vs1+2)) if((vd<vs2)|(vd>=vs2+2)) 1110001 vs2:5 vs1:5 001 vd:5 1010111 }" "asm"="vfwmul.vv {vd}, {vs2}, {vs1}" "asmonly" {
  ret void
}

define void @VFWMUL_VV.1(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="{ if((vd<vs1)|(vd>=vs1+2)) if((vd<vs2)|(vd>=vs2+2)) if(vd!=0) 1110000 vs2:5 vs1:5 001 vd:5 1010111 }" "asm"="vfwmul.vv {vd}, {vs2}, {vs1}, v0.t" "asmonly" {
  ret void
}

define void @VFWMACC_VV.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="{ if((vd<vs1)|(vd>=vs1+2)) if((vd<vs2)|(vd>=vs2+2)) 1111001 vs2:5 vs1:5 001 vd:5 1010111 }" "asm"="vfwmacc.vv {vd}, {vs1}, {vs2}" "asmonly" {
  ret void
}

define void @VFWMACC_VV.1(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="{ if((vd<vs1)|(vd>=vs1+2)) if((vd<vs2)|(vd>=vs2+2)) if(vd!=0) 1111000 vs2:5 vs1:5 001 vd:5 1010111 }" "asm"="vfwmacc.vv {vd}, {vs1}, {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VFWNMACC_VV.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="{ if((vd<vs1)|(vd>=vs1+2)) if((vd<vs2)|(vd>=vs2+2)) 1111011 vs2:5 vs1:5 001 vd:5 1010111 }" "asm"="vfwnmacc.vv {vd}, {vs1}, {vs2}" "asmonly" {
  ret void
}

define void @VFWNMACC_VV.1(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="{ if((vd<vs1)|(vd>=vs1+2)) if((vd<vs2)|(vd>=vs2+2)) if(vd!=0) 1111010 vs2:5 vs1:5 001 vd:5 1010111 }" "asm"="vfwnmacc.vv {vd}, {vs1}, {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VFWMSAC_VV.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="{ if((vd<vs1)|(vd>=vs1+2)) if((vd<vs2)|(vd>=vs2+2)) 1111101 vs2:5 vs1:5 001 vd:5 1010111 }" "asm"="vfwmsac.vv {vd}, {vs1}, {vs2}" "asmonly" {
  ret void
}

define void @VFWMSAC_VV.1(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="{ if((vd<vs1)|(vd>=vs1+2)) if((vd<vs2)|(vd>=vs2+2)) if(vd!=0) 1111100 vs2:5 vs1:5 001 vd:5 1010111 }" "asm"="vfwmsac.vv {vd}, {vs1}, {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VFWNMSAC_VV.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="{ if((vd<vs1)|(vd>=vs1+2)) if((vd<vs2)|(vd>=vs2+2)) 1111111 vs2:5 vs1:5 001 vd:5 1010111 }" "asm"="vfwnmsac.vv {vd}, {vs1}, {vs2}" "asmonly" {
  ret void
}

define void @VFWNMSAC_VV.1(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="{ if((vd<vs1)|(vd>=vs1+2)) if((vd<vs2)|(vd>=vs2+2)) if(vd!=0) 1111110 vs2:5 vs1:5 001 vd:5 1010111 }" "asm"="vfwnmsac.vv {vd}, {vs1}, {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VADD_VX.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="0000001 vs2:5 rs1:5 100 vd:5 1010111" "asm"="vadd.vx {vd}, {vs2}, {rs1}" "asmonly" {
  ret void
}

define void @VADD_VX.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if(vd!=0) 0000000 vs2:5 rs1:5 100 vd:5 1010111 }" "asm"="vadd.vx {vd}, {vs2}, {rs1}, v0.t" "asmonly" {
  ret void
}

define void @VSUB_VX.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="0000101 vs2:5 rs1:5 100 vd:5 1010111" "asm"="vsub.vx {vd}, {vs2}, {rs1}" "asmonly" {
  ret void
}

define void @VSUB_VX.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if(vd!=0) 0000100 vs2:5 rs1:5 100 vd:5 1010111 }" "asm"="vsub.vx {vd}, {vs2}, {rs1}, v0.t" "asmonly" {
  ret void
}

define void @VRSUB_VX.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="0000111 vs2:5 rs1:5 100 vd:5 1010111" "asm"="vrsub.vx {vd}, {vs2}, {rs1}" "asmonly" {
  ret void
}

define void @VRSUB_VX.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if(vd!=0) 0000110 vs2:5 rs1:5 100 vd:5 1010111 }" "asm"="vrsub.vx {vd}, {vs2}, {rs1}, v0.t" "asmonly" {
  ret void
}

define void @VMINU_VX.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="0001001 vs2:5 rs1:5 100 vd:5 1010111" "asm"="vminu.vx {vd}, {vs2}, {rs1}" "asmonly" {
  ret void
}

define void @VMINU_VX.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if(vd!=0) 0001000 vs2:5 rs1:5 100 vd:5 1010111 }" "asm"="vminu.vx {vd}, {vs2}, {rs1}, v0.t" "asmonly" {
  ret void
}

define void @VMIN_VX.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="0001011 vs2:5 rs1:5 100 vd:5 1010111" "asm"="vmin.vx {vd}, {vs2}, {rs1}" "asmonly" {
  ret void
}

define void @VMIN_VX.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if(vd!=0) 0001010 vs2:5 rs1:5 100 vd:5 1010111 }" "asm"="vmin.vx {vd}, {vs2}, {rs1}, v0.t" "asmonly" {
  ret void
}

define void @VMAXU_VX.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="0001101 vs2:5 rs1:5 100 vd:5 1010111" "asm"="vmaxu.vx {vd}, {vs2}, {rs1}" "asmonly" {
  ret void
}

define void @VMAXU_VX.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if(vd!=0) 0001100 vs2:5 rs1:5 100 vd:5 1010111 }" "asm"="vmaxu.vx {vd}, {vs2}, {rs1}, v0.t" "asmonly" {
  ret void
}

define void @VMAX_VX.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="0001111 vs2:5 rs1:5 100 vd:5 1010111" "asm"="vmax.vx {vd}, {vs2}, {rs1}" "asmonly" {
  ret void
}

define void @VMAX_VX.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if(vd!=0) 0001110 vs2:5 rs1:5 100 vd:5 1010111 }" "asm"="vmax.vx {vd}, {vs2}, {rs1}, v0.t" "asmonly" {
  ret void
}

define void @VAND_VX.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="0010011 vs2:5 rs1:5 100 vd:5 1010111" "asm"="vand.vx {vd}, {vs2}, {rs1}" "asmonly" {
  ret void
}

define void @VAND_VX.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if(vd!=0) 0010010 vs2:5 rs1:5 100 vd:5 1010111 }" "asm"="vand.vx {vd}, {vs2}, {rs1}, v0.t" "asmonly" {
  ret void
}

define void @VOR_VX.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="0010101 vs2:5 rs1:5 100 vd:5 1010111" "asm"="vor.vx {vd}, {vs2}, {rs1}" "asmonly" {
  ret void
}

define void @VOR_VX.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if(vd!=0) 0010100 vs2:5 rs1:5 100 vd:5 1010111 }" "asm"="vor.vx {vd}, {vs2}, {rs1}, v0.t" "asmonly" {
  ret void
}

define void @VXOR_VX.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="0010111 vs2:5 rs1:5 100 vd:5 1010111" "asm"="vxor.vx {vd}, {vs2}, {rs1}" "asmonly" {
  ret void
}

define void @VXOR_VX.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if(vd!=0) 0010110 vs2:5 rs1:5 100 vd:5 1010111 }" "asm"="vxor.vx {vd}, {vs2}, {rs1}, v0.t" "asmonly" {
  ret void
}

define void @VRGATHER_VX.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if((vd<vs2)|(vd>=vs2+2)) 0011001 vs2:5 rs1:5 100 vd:5 1010111 }" "asm"="vrgather.vx {vd}, {vs2}, {rs1}" "asmonly" {
  ret void
}

define void @VRGATHER_VX.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if((vd<vs2)|(vd>=vs2+2)) if(vd!=0) 0011000 vs2:5 rs1:5 100 vd:5 1010111 }" "asm"="vrgather.vx {vd}, {vs2}, {rs1}, v0.t" "asmonly" {
  ret void
}

define void @VSLIDEUP_VX.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if((vd<vs2)|(vd>=vs2+2)) 0011101 vs2:5 rs1:5 100 vd:5 1010111 }" "asm"="vslideup.vx {vd}, {vs2}, {rs1}" "asmonly" {
  ret void
}

define void @VSLIDEUP_VX.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if((vd<vs2)|(vd>=vs2+2)) if(vd!=0) 0011100 vs2:5 rs1:5 100 vd:5 1010111 }" "asm"="vslideup.vx {vd}, {vs2}, {rs1}, v0.t" "asmonly" {
  ret void
}

define void @VSLIDEDOWN_VX.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="0011111 vs2:5 rs1:5 100 vd:5 1010111" "asm"="vslidedown.vx {vd}, {vs2}, {rs1}" "asmonly" {
  ret void
}

define void @VSLIDEDOWN_VX.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if(vd!=0) 0011110 vs2:5 rs1:5 100 vd:5 1010111 }" "asm"="vslidedown.vx {vd}, {vs2}, {rs1}, v0.t" "asmonly" {
  ret void
}

define void @VADC_VXM.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if(vd!=0) 0100000 vs2:5 rs1:5 100 vd:5 1010111 }" "asm"="vadc.vxm {vd}, {vs2}, {rs1}, v0" "asmonly" {
  ret void
}

define void @VMADC_VXM.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="0100010 vs2:5 rs1:5 100 vd:5 1010111" "asm"="vmadc.vxm {vd}, {vs2}, {rs1}, v0" "asmonly" {
  ret void
}

define void @VMADC_VX.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="0100011 vs2:5 rs1:5 100 vd:5 1010111" "asm"="vmadc.vx {vd}, {vs2}, {rs1}" "asmonly" {
  ret void
}

define void @VSBC_VXM.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if(vd!=0) 0100100 vs2:5 rs1:5 100 vd:5 1010111 }" "asm"="vsbc.vxm {vd}, {vs2}, {rs1}, v0" "asmonly" {
  ret void
}

define void @VMSBC_VXM.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="0100110 vs2:5 rs1:5 100 vd:5 1010111" "asm"="vmsbc.vxm {vd}, {vs2}, {rs1}, v0" "asmonly" {
  ret void
}

define void @VMSBC_VX.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="0100111 vs2:5 rs1:5 100 vd:5 1010111" "asm"="vmsbc.vx {vd}, {vs2}, {rs1}" "asmonly" {
  ret void
}

define void @VMERGE_VXM.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if(vd!=0) 0101110 vs2:5 rs1:5 100 vd:5 1010111 }" "asm"="vmerge.vxm {vd}, {vs2}, {rs1}, v0" "asmonly" {
  ret void
}

define void @VMV_V_X.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd) "enc"="010111100000 rs1:5 100 vd:5 1010111" "asm"="vmv.v.x {vd}, {rs1}" "asmonly" {
  ret void
}

define void @VMSEQ_VX.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="0110001 vs2:5 rs1:5 100 vd:5 1010111" "asm"="vmseq.vx {vd}, {vs2}, {rs1}" "asmonly" {
  ret void
}

define void @VMSEQ_VX.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="0110000 vs2:5 rs1:5 100 vd:5 1010111" "asm"="vmseq.vx {vd}, {vs2}, {rs1}, v0.t" "asmonly" {
  ret void
}

define void @VMSNE_VX.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="0110011 vs2:5 rs1:5 100 vd:5 1010111" "asm"="vmsne.vx {vd}, {vs2}, {rs1}" "asmonly" {
  ret void
}

define void @VMSNE_VX.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="0110010 vs2:5 rs1:5 100 vd:5 1010111" "asm"="vmsne.vx {vd}, {vs2}, {rs1}, v0.t" "asmonly" {
  ret void
}

define void @VMSLTU_VX.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="0110101 vs2:5 rs1:5 100 vd:5 1010111" "asm"="vmsltu.vx {vd}, {vs2}, {rs1}" "asmonly" {
  ret void
}

define void @VMSLTU_VX.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="0110100 vs2:5 rs1:5 100 vd:5 1010111" "asm"="vmsltu.vx {vd}, {vs2}, {rs1}, v0.t" "asmonly" {
  ret void
}

define void @VMSLT_VX.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="0110111 vs2:5 rs1:5 100 vd:5 1010111" "asm"="vmslt.vx {vd}, {vs2}, {rs1}" "asmonly" {
  ret void
}

define void @VMSLT_VX.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="0110110 vs2:5 rs1:5 100 vd:5 1010111" "asm"="vmslt.vx {vd}, {vs2}, {rs1}, v0.t" "asmonly" {
  ret void
}

define void @VMSLEU_VX.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="0111001 vs2:5 rs1:5 100 vd:5 1010111" "asm"="vmsleu.vx {vd}, {vs2}, {rs1}" "asmonly" {
  ret void
}

define void @VMSLEU_VX.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="0111000 vs2:5 rs1:5 100 vd:5 1010111" "asm"="vmsleu.vx {vd}, {vs2}, {rs1}, v0.t" "asmonly" {
  ret void
}

define void @VMSLE_VX.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="0111011 vs2:5 rs1:5 100 vd:5 1010111" "asm"="vmsle.vx {vd}, {vs2}, {rs1}" "asmonly" {
  ret void
}

define void @VMSLE_VX.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="0111010 vs2:5 rs1:5 100 vd:5 1010111" "asm"="vmsle.vx {vd}, {vs2}, {rs1}, v0.t" "asmonly" {
  ret void
}

define void @VMSGTU_VX.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="0111101 vs2:5 rs1:5 100 vd:5 1010111" "asm"="vmsgtu.vx {vd}, {vs2}, {rs1}" "asmonly" {
  ret void
}

define void @VMSGTU_VX.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="0111100 vs2:5 rs1:5 100 vd:5 1010111" "asm"="vmsgtu.vx {vd}, {vs2}, {rs1}, v0.t" "asmonly" {
  ret void
}

define void @VMSGT_VX.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="0111111 vs2:5 rs1:5 100 vd:5 1010111" "asm"="vmsgt.vx {vd}, {vs2}, {rs1}" "asmonly" {
  ret void
}

define void @VMSGT_VX.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="0111110 vs2:5 rs1:5 100 vd:5 1010111" "asm"="vmsgt.vx {vd}, {vs2}, {rs1}, v0.t" "asmonly" {
  ret void
}

define void @VSADDU_VX.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="1000001 vs2:5 rs1:5 100 vd:5 1010111" "asm"="vsaddu.vx {vd}, {vs2}, {rs1}" "asmonly" {
  ret void
}

define void @VSADDU_VX.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if(vd!=0) 1000000 vs2:5 rs1:5 100 vd:5 1010111 }" "asm"="vsaddu.vx {vd}, {vs2}, {rs1}, v0.t" "asmonly" {
  ret void
}

define void @VSADD_VX.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="1000011 vs2:5 rs1:5 100 vd:5 1010111" "asm"="vsadd.vx {vd}, {vs2}, {rs1}" "asmonly" {
  ret void
}

define void @VSADD_VX.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if(vd!=0) 1000010 vs2:5 rs1:5 100 vd:5 1010111 }" "asm"="vsadd.vx {vd}, {vs2}, {rs1}, v0.t" "asmonly" {
  ret void
}

define void @VSSUBU_VX.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="1000101 vs2:5 rs1:5 100 vd:5 1010111" "asm"="vssubu.vx {vd}, {vs2}, {rs1}" "asmonly" {
  ret void
}

define void @VSSUBU_VX.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if(vd!=0) 1000100 vs2:5 rs1:5 100 vd:5 1010111 }" "asm"="vssubu.vx {vd}, {vs2}, {rs1}, v0.t" "asmonly" {
  ret void
}

define void @VSSUB_VX.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="1000111 vs2:5 rs1:5 100 vd:5 1010111" "asm"="vssub.vx {vd}, {vs2}, {rs1}" "asmonly" {
  ret void
}

define void @VSSUB_VX.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if(vd!=0) 1000110 vs2:5 rs1:5 100 vd:5 1010111 }" "asm"="vssub.vx {vd}, {vs2}, {rs1}, v0.t" "asmonly" {
  ret void
}

define void @VSLL_VX.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="1001011 vs2:5 rs1:5 100 vd:5 1010111" "asm"="vsll.vx {vd}, {vs2}, {rs1}" "asmonly" {
  ret void
}

define void @VSLL_VX.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if(vd!=0) 1001010 vs2:5 rs1:5 100 vd:5 1010111 }" "asm"="vsll.vx {vd}, {vs2}, {rs1}, v0.t" "asmonly" {
  ret void
}

define void @VSMUL_VX.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="1001111 vs2:5 rs1:5 100 vd:5 1010111" "asm"="vsmul.vx {vd}, {vs2}, {rs1}" "asmonly" {
  ret void
}

define void @VSMUL_VX.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if(vd!=0) 1001110 vs2:5 rs1:5 100 vd:5 1010111 }" "asm"="vsmul.vx {vd}, {vs2}, {rs1}, v0.t" "asmonly" {
  ret void
}

define void @VSRL_VX.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="1010001 vs2:5 rs1:5 100 vd:5 1010111" "asm"="vsrl.vx {vd}, {vs2}, {rs1}" "asmonly" {
  ret void
}

define void @VSRL_VX.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if(vd!=0) 1010000 vs2:5 rs1:5 100 vd:5 1010111 }" "asm"="vsrl.vx {vd}, {vs2}, {rs1}, v0.t" "asmonly" {
  ret void
}

define void @VSRA_VX.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="1010011 vs2:5 rs1:5 100 vd:5 1010111" "asm"="vsra.vx {vd}, {vs2}, {rs1}" "asmonly" {
  ret void
}

define void @VSRA_VX.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if(vd!=0) 1010010 vs2:5 rs1:5 100 vd:5 1010111 }" "asm"="vsra.vx {vd}, {vs2}, {rs1}, v0.t" "asmonly" {
  ret void
}

define void @VSSRL_VX.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="1010101 vs2:5 rs1:5 100 vd:5 1010111" "asm"="vssrl.vx {vd}, {vs2}, {rs1}" "asmonly" {
  ret void
}

define void @VSSRL_VX.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if(vd!=0) 1010100 vs2:5 rs1:5 100 vd:5 1010111 }" "asm"="vssrl.vx {vd}, {vs2}, {rs1}, v0.t" "asmonly" {
  ret void
}

define void @VSSRA_VX.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="1010111 vs2:5 rs1:5 100 vd:5 1010111" "asm"="vssra.vx {vd}, {vs2}, {rs1}" "asmonly" {
  ret void
}

define void @VSSRA_VX.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if(vd!=0) 1010110 vs2:5 rs1:5 100 vd:5 1010111 }" "asm"="vssra.vx {vd}, {vs2}, {rs1}, v0.t" "asmonly" {
  ret void
}

define void @VNSRL_WX.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="1011001 vs2:5 rs1:5 100 vd:5 1010111" "asm"="vnsrl.wx {vd}, {vs2}, {rs1}" "asmonly" {
  ret void
}

define void @VNSRL_WX.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if(vd!=0) 1011000 vs2:5 rs1:5 100 vd:5 1010111 }" "asm"="vnsrl.wx {vd}, {vs2}, {rs1}, v0.t" "asmonly" {
  ret void
}

define void @VNSRA_WX.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="1011011 vs2:5 rs1:5 100 vd:5 1010111" "asm"="vnsra.wx {vd}, {vs2}, {rs1}" "asmonly" {
  ret void
}

define void @VNSRA_WX.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if(vd!=0) 1011010 vs2:5 rs1:5 100 vd:5 1010111 }" "asm"="vnsra.wx {vd}, {vs2}, {rs1}, v0.t" "asmonly" {
  ret void
}

define void @VNCLIPU_WX.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="1011101 vs2:5 rs1:5 100 vd:5 1010111" "asm"="vnclipu.wx {vd}, {vs2}, {rs1}" "asmonly" {
  ret void
}

define void @VNCLIPU_WX.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if(vd!=0) 1011100 vs2:5 rs1:5 100 vd:5 1010111 }" "asm"="vnclipu.wx {vd}, {vs2}, {rs1}, v0.t" "asmonly" {
  ret void
}

define void @VNCLIP_WX.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="1011111 vs2:5 rs1:5 100 vd:5 1010111" "asm"="vnclip.wx {vd}, {vs2}, {rs1}" "asmonly" {
  ret void
}

define void @VNCLIP_WX.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if(vd!=0) 1011110 vs2:5 rs1:5 100 vd:5 1010111 }" "asm"="vnclip.wx {vd}, {vs2}, {rs1}, v0.t" "asmonly" {
  ret void
}

define void @VADD_VV.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="0000001 vs2:5 vs1:5 000 vd:5 1010111" "asm"="vadd.vv {vd}, {vs2}, {vs1}" "asmonly" {
  ret void
}

define void @VADD_VV.1(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="{ if(vd!=0) 0000000 vs2:5 vs1:5 000 vd:5 1010111 }" "asm"="vadd.vv {vd}, {vs2}, {vs1}, v0.t" "asmonly" {
  ret void
}

define void @VSUB_VV.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="0000101 vs2:5 vs1:5 000 vd:5 1010111" "asm"="vsub.vv {vd}, {vs2}, {vs1}" "asmonly" {
  ret void
}

define void @VSUB_VV.1(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="{ if(vd!=0) 0000100 vs2:5 vs1:5 000 vd:5 1010111 }" "asm"="vsub.vv {vd}, {vs2}, {vs1}, v0.t" "asmonly" {
  ret void
}

define void @VMINU_VV.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="0001001 vs2:5 vs1:5 000 vd:5 1010111" "asm"="vminu.vv {vd}, {vs2}, {vs1}" "asmonly" {
  ret void
}

define void @VMINU_VV.1(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="{ if(vd!=0) 0001000 vs2:5 vs1:5 000 vd:5 1010111 }" "asm"="vminu.vv {vd}, {vs2}, {vs1}, v0.t" "asmonly" {
  ret void
}

define void @VMIN_VV.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="0001011 vs2:5 vs1:5 000 vd:5 1010111" "asm"="vmin.vv {vd}, {vs2}, {vs1}" "asmonly" {
  ret void
}

define void @VMIN_VV.1(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="{ if(vd!=0) 0001010 vs2:5 vs1:5 000 vd:5 1010111 }" "asm"="vmin.vv {vd}, {vs2}, {vs1}, v0.t" "asmonly" {
  ret void
}

define void @VMAXU_VV.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="0001101 vs2:5 vs1:5 000 vd:5 1010111" "asm"="vmaxu.vv {vd}, {vs2}, {vs1}" "asmonly" {
  ret void
}

define void @VMAXU_VV.1(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="{ if(vd!=0) 0001100 vs2:5 vs1:5 000 vd:5 1010111 }" "asm"="vmaxu.vv {vd}, {vs2}, {vs1}, v0.t" "asmonly" {
  ret void
}

define void @VMAX_VV.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="0001111 vs2:5 vs1:5 000 vd:5 1010111" "asm"="vmax.vv {vd}, {vs2}, {vs1}" "asmonly" {
  ret void
}

define void @VMAX_VV.1(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="{ if(vd!=0) 0001110 vs2:5 vs1:5 000 vd:5 1010111 }" "asm"="vmax.vv {vd}, {vs2}, {vs1}, v0.t" "asmonly" {
  ret void
}

define void @VAND_VV.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="0010011 vs2:5 vs1:5 000 vd:5 1010111" "asm"="vand.vv {vd}, {vs2}, {vs1}" "asmonly" {
  ret void
}

define void @VAND_VV.1(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="{ if(vd!=0) 0010010 vs2:5 vs1:5 000 vd:5 1010111 }" "asm"="vand.vv {vd}, {vs2}, {vs1}, v0.t" "asmonly" {
  ret void
}

define void @VOR_VV.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="0010101 vs2:5 vs1:5 000 vd:5 1010111" "asm"="vor.vv {vd}, {vs2}, {vs1}" "asmonly" {
  ret void
}

define void @VOR_VV.1(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="{ if(vd!=0) 0010100 vs2:5 vs1:5 000 vd:5 1010111 }" "asm"="vor.vv {vd}, {vs2}, {vs1}, v0.t" "asmonly" {
  ret void
}

define void @VXOR_VV.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="0010111 vs2:5 vs1:5 000 vd:5 1010111" "asm"="vxor.vv {vd}, {vs2}, {vs1}" "asmonly" {
  ret void
}

define void @VXOR_VV.1(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="{ if(vd!=0) 0010110 vs2:5 vs1:5 000 vd:5 1010111 }" "asm"="vxor.vv {vd}, {vs2}, {vs1}, v0.t" "asmonly" {
  ret void
}

define void @VRGATHER_VV.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="{ if((vd<vs1)|(vd>=vs1+2)) if((vd<vs2)|(vd>=vs2+2)) 0011001 vs2:5 vs1:5 000 vd:5 1010111 }" "asm"="vrgather.vv {vd}, {vs2}, {vs1}" "asmonly" {
  ret void
}

define void @VRGATHER_VV.1(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="{ if((vd<vs1)|(vd>=vs1+2)) if((vd<vs2)|(vd>=vs2+2)) if(vd!=0) 0011000 vs2:5 vs1:5 000 vd:5 1010111 }" "asm"="vrgather.vv {vd}, {vs2}, {vs1}, v0.t" "asmonly" {
  ret void
}

define void @VRGATHEREI16_VV.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="{ if((vd<vs1)|(vd>=vs1+2)) if((vd<vs2)|(vd>=vs2+2)) 0011101 vs2:5 vs1:5 000 vd:5 1010111 }" "asm"="vrgatherei16.vv {vd}, {vs2}, {vs1}" "asmonly" {
  ret void
}

define void @VRGATHEREI16_VV.1(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="{ if((vd<vs1)|(vd>=vs1+2)) if((vd<vs2)|(vd>=vs2+2)) if(vd!=0) 0011100 vs2:5 vs1:5 000 vd:5 1010111 }" "asm"="vrgatherei16.vv {vd}, {vs2}, {vs1}, v0.t" "asmonly" {
  ret void
}

define void @VADC_VVM.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="{ if(vd!=0) 0100000 vs2:5 vs1:5 000 vd:5 1010111 }" "asm"="vadc.vvm {vd}, {vs2}, {vs1}, v0" "asmonly" {
  ret void
}

define void @VMADC_VVM.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="0100010 vs2:5 vs1:5 000 vd:5 1010111" "asm"="vmadc.vvm {vd}, {vs2}, {vs1}, v0" "asmonly" {
  ret void
}

define void @VMADC_VV.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="0100011 vs2:5 vs1:5 000 vd:5 1010111" "asm"="vmadc.vv {vd}, {vs2}, {vs1}" "asmonly" {
  ret void
}

define void @VSBC_VVM.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="{ if(vd!=0) 0100100 vs2:5 vs1:5 000 vd:5 1010111 }" "asm"="vsbc.vvm {vd}, {vs2}, {vs1}, v0" "asmonly" {
  ret void
}

define void @VMSBC_VVM.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="0100110 vs2:5 vs1:5 000 vd:5 1010111" "asm"="vmsbc.vvm {vd}, {vs2}, {vs1}, v0" "asmonly" {
  ret void
}

define void @VMSBC_VV.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="0100111 vs2:5 vs1:5 000 vd:5 1010111" "asm"="vmsbc.vv {vd}, {vs2}, {vs1}" "asmonly" {
  ret void
}

define void @VMERGE_VVM.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="{ if(vd!=0) 0101110 vs2:5 vs1:5 000 vd:5 1010111 }" "asm"="vmerge.vvm {vd}, {vs2}, {vs1}, v0" "asmonly" {
  ret void
}

define void @VMV_V_V.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs1) "enc"="010111100000 vs1:5 000 vd:5 1010111" "asm"="vmv.v.v {vd}, {vs1}" "asmonly" {
  ret void
}

define void @VMSEQ_VV.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="0110001 vs2:5 vs1:5 000 vd:5 1010111" "asm"="vmseq.vv {vd}, {vs2}, {vs1}" "asmonly" {
  ret void
}

define void @VMSEQ_VV.1(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="0110000 vs2:5 vs1:5 000 vd:5 1010111" "asm"="vmseq.vv {vd}, {vs2}, {vs1}, v0.t" "asmonly" {
  ret void
}

define void @VMSNE_VV.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="0110011 vs2:5 vs1:5 000 vd:5 1010111" "asm"="vmsne.vv {vd}, {vs2}, {vs1}" "asmonly" {
  ret void
}

define void @VMSNE_VV.1(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="0110010 vs2:5 vs1:5 000 vd:5 1010111" "asm"="vmsne.vv {vd}, {vs2}, {vs1}, v0.t" "asmonly" {
  ret void
}

define void @VMSLTU_VV.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="0110101 vs2:5 vs1:5 000 vd:5 1010111" "asm"="vmsltu.vv {vd}, {vs2}, {vs1}" "asmonly" {
  ret void
}

define void @VMSLTU_VV.1(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="0110100 vs2:5 vs1:5 000 vd:5 1010111" "asm"="vmsltu.vv {vd}, {vs2}, {vs1}, v0.t" "asmonly" {
  ret void
}

define void @VMSLT_VV.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="0110111 vs2:5 vs1:5 000 vd:5 1010111" "asm"="vmslt.vv {vd}, {vs2}, {vs1}" "asmonly" {
  ret void
}

define void @VMSLT_VV.1(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="0110110 vs2:5 vs1:5 000 vd:5 1010111" "asm"="vmslt.vv {vd}, {vs2}, {vs1}, v0.t" "asmonly" {
  ret void
}

define void @VMSLEU_VV.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="0111001 vs2:5 vs1:5 000 vd:5 1010111" "asm"="vmsleu.vv {vd}, {vs2}, {vs1}" "asmonly" {
  ret void
}

define void @VMSLEU_VV.1(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="0111000 vs2:5 vs1:5 000 vd:5 1010111" "asm"="vmsleu.vv {vd}, {vs2}, {vs1}, v0.t" "asmonly" {
  ret void
}

define void @VMSLE_VV.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="0111011 vs2:5 vs1:5 000 vd:5 1010111" "asm"="vmsle.vv {vd}, {vs2}, {vs1}" "asmonly" {
  ret void
}

define void @VMSLE_VV.1(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="0111010 vs2:5 vs1:5 000 vd:5 1010111" "asm"="vmsle.vv {vd}, {vs2}, {vs1}, v0.t" "asmonly" {
  ret void
}

define void @VSADDU_VV.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="1000001 vs2:5 vs1:5 000 vd:5 1010111" "asm"="vsaddu.vv {vd}, {vs2}, {vs1}" "asmonly" {
  ret void
}

define void @VSADDU_VV.1(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="{ if(vd!=0) 1000000 vs2:5 vs1:5 000 vd:5 1010111 }" "asm"="vsaddu.vv {vd}, {vs2}, {vs1}, v0.t" "asmonly" {
  ret void
}

define void @VSADD_VV.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="1000011 vs2:5 vs1:5 000 vd:5 1010111" "asm"="vsadd.vv {vd}, {vs2}, {vs1}" "asmonly" {
  ret void
}

define void @VSADD_VV.1(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="{ if(vd!=0) 1000010 vs2:5 vs1:5 000 vd:5 1010111 }" "asm"="vsadd.vv {vd}, {vs2}, {vs1}, v0.t" "asmonly" {
  ret void
}

define void @VSSUBU_VV.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="1000101 vs2:5 vs1:5 000 vd:5 1010111" "asm"="vssubu.vv {vd}, {vs2}, {vs1}" "asmonly" {
  ret void
}

define void @VSSUBU_VV.1(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="{ if(vd!=0) 1000100 vs2:5 vs1:5 000 vd:5 1010111 }" "asm"="vssubu.vv {vd}, {vs2}, {vs1}, v0.t" "asmonly" {
  ret void
}

define void @VSSUB_VV.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="1000111 vs2:5 vs1:5 000 vd:5 1010111" "asm"="vssub.vv {vd}, {vs2}, {vs1}" "asmonly" {
  ret void
}

define void @VSSUB_VV.1(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="{ if(vd!=0) 1000110 vs2:5 vs1:5 000 vd:5 1010111 }" "asm"="vssub.vv {vd}, {vs2}, {vs1}, v0.t" "asmonly" {
  ret void
}

define void @VSLL_VV.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="1001011 vs2:5 vs1:5 000 vd:5 1010111" "asm"="vsll.vv {vd}, {vs2}, {vs1}" "asmonly" {
  ret void
}

define void @VSLL_VV.1(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="{ if(vd!=0) 1001010 vs2:5 vs1:5 000 vd:5 1010111 }" "asm"="vsll.vv {vd}, {vs2}, {vs1}, v0.t" "asmonly" {
  ret void
}

define void @VSMUL_VV.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="1001111 vs2:5 vs1:5 000 vd:5 1010111" "asm"="vsmul.vv {vd}, {vs2}, {vs1}" "asmonly" {
  ret void
}

define void @VSMUL_VV.1(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="{ if(vd!=0) 1001110 vs2:5 vs1:5 000 vd:5 1010111 }" "asm"="vsmul.vv {vd}, {vs2}, {vs1}, v0.t" "asmonly" {
  ret void
}

define void @VSRL_VV.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="1010001 vs2:5 vs1:5 000 vd:5 1010111" "asm"="vsrl.vv {vd}, {vs2}, {vs1}" "asmonly" {
  ret void
}

define void @VSRL_VV.1(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="{ if(vd!=0) 1010000 vs2:5 vs1:5 000 vd:5 1010111 }" "asm"="vsrl.vv {vd}, {vs2}, {vs1}, v0.t" "asmonly" {
  ret void
}

define void @VSRA_VV.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="1010011 vs2:5 vs1:5 000 vd:5 1010111" "asm"="vsra.vv {vd}, {vs2}, {vs1}" "asmonly" {
  ret void
}

define void @VSRA_VV.1(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="{ if(vd!=0) 1010010 vs2:5 vs1:5 000 vd:5 1010111 }" "asm"="vsra.vv {vd}, {vs2}, {vs1}, v0.t" "asmonly" {
  ret void
}

define void @VSSRL_VV.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="1010101 vs2:5 vs1:5 000 vd:5 1010111" "asm"="vssrl.vv {vd}, {vs2}, {vs1}" "asmonly" {
  ret void
}

define void @VSSRL_VV.1(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="{ if(vd!=0) 1010100 vs2:5 vs1:5 000 vd:5 1010111 }" "asm"="vssrl.vv {vd}, {vs2}, {vs1}, v0.t" "asmonly" {
  ret void
}

define void @VSSRA_VV.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="1010111 vs2:5 vs1:5 000 vd:5 1010111" "asm"="vssra.vv {vd}, {vs2}, {vs1}" "asmonly" {
  ret void
}

define void @VSSRA_VV.1(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="{ if(vd!=0) 1010110 vs2:5 vs1:5 000 vd:5 1010111 }" "asm"="vssra.vv {vd}, {vs2}, {vs1}, v0.t" "asmonly" {
  ret void
}

define void @VNSRL_WV.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="1011001 vs2:5 vs1:5 000 vd:5 1010111" "asm"="vnsrl.wv {vd}, {vs2}, {vs1}" "asmonly" {
  ret void
}

define void @VNSRL_WV.1(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="{ if(vd!=0) 1011000 vs2:5 vs1:5 000 vd:5 1010111 }" "asm"="vnsrl.wv {vd}, {vs2}, {vs1}, v0.t" "asmonly" {
  ret void
}

define void @VNSRA_WV.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="1011011 vs2:5 vs1:5 000 vd:5 1010111" "asm"="vnsra.wv {vd}, {vs2}, {vs1}" "asmonly" {
  ret void
}

define void @VNSRA_WV.1(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="{ if(vd!=0) 1011010 vs2:5 vs1:5 000 vd:5 1010111 }" "asm"="vnsra.wv {vd}, {vs2}, {vs1}, v0.t" "asmonly" {
  ret void
}

define void @VNCLIPU_WV.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="1011101 vs2:5 vs1:5 000 vd:5 1010111" "asm"="vnclipu.wv {vd}, {vs2}, {vs1}" "asmonly" {
  ret void
}

define void @VNCLIPU_WV.1(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="{ if(vd!=0) 1011100 vs2:5 vs1:5 000 vd:5 1010111 }" "asm"="vnclipu.wv {vd}, {vs2}, {vs1}, v0.t" "asmonly" {
  ret void
}

define void @VNCLIP_WV.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="1011111 vs2:5 vs1:5 000 vd:5 1010111" "asm"="vnclip.wv {vd}, {vs2}, {vs1}" "asmonly" {
  ret void
}

define void @VNCLIP_WV.1(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="{ if(vd!=0) 1011110 vs2:5 vs1:5 000 vd:5 1010111 }" "asm"="vnclip.wv {vd}, {vs2}, {vs1}, v0.t" "asmonly" {
  ret void
}

define void @VWREDSUMU_VS.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="1100001 vs2:5 vs1:5 000 vd:5 1010111" "asm"="vwredsumu.vs {vd}, {vs2}, {vs1}" "asmonly" {
  ret void
}

define void @VWREDSUMU_VS.1(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="1100000 vs2:5 vs1:5 000 vd:5 1010111" "asm"="vwredsumu.vs {vd}, {vs2}, {vs1}, v0.t" "asmonly" {
  ret void
}

define void @VWREDSUM_VS.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="1100011 vs2:5 vs1:5 000 vd:5 1010111" "asm"="vwredsum.vs {vd}, {vs2}, {vs1}" "asmonly" {
  ret void
}

define void @VWREDSUM_VS.1(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="1100010 vs2:5 vs1:5 000 vd:5 1010111" "asm"="vwredsum.vs {vd}, {vs2}, {vs1}, v0.t" "asmonly" {
  ret void
}

define void @VADD_VI.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs2, i64 "imm" %simm5) "enc"="0000001 vs2:5 simm5:s5 011 vd:5 1010111" "asm"="vadd.vi {vd}, {vs2}, {simm5}" "asmonly" {
  ret void
}

define void @VADD_VI.1(i64 "reg"="v" %vd, i64 "reg"="v" %vs2, i64 "imm" %simm5) "enc"="{ if(vd!=0) 0000000 vs2:5 simm5:s5 011 vd:5 1010111 }" "asm"="vadd.vi {vd}, {vs2}, {simm5}, v0.t" "asmonly" {
  ret void
}

define void @VRSUB_VI.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs2, i64 "imm" %simm5) "enc"="0000111 vs2:5 simm5:s5 011 vd:5 1010111" "asm"="vrsub.vi {vd}, {vs2}, {simm5}" "asmonly" {
  ret void
}

define void @VRSUB_VI.1(i64 "reg"="v" %vd, i64 "reg"="v" %vs2, i64 "imm" %simm5) "enc"="{ if(vd!=0) 0000110 vs2:5 simm5:s5 011 vd:5 1010111 }" "asm"="vrsub.vi {vd}, {vs2}, {simm5}, v0.t" "asmonly" {
  ret void
}

define void @VAND_VI.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs2, i64 "imm" %simm5) "enc"="0010011 vs2:5 simm5:s5 011 vd:5 1010111" "asm"="vand.vi {vd}, {vs2}, {simm5}" "asmonly" {
  ret void
}

define void @VAND_VI.1(i64 "reg"="v" %vd, i64 "reg"="v" %vs2, i64 "imm" %simm5) "enc"="{ if(vd!=0) 0010010 vs2:5 simm5:s5 011 vd:5 1010111 }" "asm"="vand.vi {vd}, {vs2}, {simm5}, v0.t" "asmonly" {
  ret void
}

define void @VOR_VI.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs2, i64 "imm" %simm5) "enc"="0010101 vs2:5 simm5:s5 011 vd:5 1010111" "asm"="vor.vi {vd}, {vs2}, {simm5}" "asmonly" {
  ret void
}

define void @VOR_VI.1(i64 "reg"="v" %vd, i64 "reg"="v" %vs2, i64 "imm" %simm5) "enc"="{ if(vd!=0) 0010100 vs2:5 simm5:s5 011 vd:5 1010111 }" "asm"="vor.vi {vd}, {vs2}, {simm5}, v0.t" "asmonly" {
  ret void
}

define void @VXOR_VI.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs2, i64 "imm" %simm5) "enc"="0010111 vs2:5 simm5:s5 011 vd:5 1010111" "asm"="vxor.vi {vd}, {vs2}, {simm5}" "asmonly" {
  ret void
}

define void @VXOR_VI.1(i64 "reg"="v" %vd, i64 "reg"="v" %vs2, i64 "imm" %simm5) "enc"="{ if(vd!=0) 0010110 vs2:5 simm5:s5 011 vd:5 1010111 }" "asm"="vxor.vi {vd}, {vs2}, {simm5}, v0.t" "asmonly" {
  ret void
}

define void @VRGATHER_VI.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs2, i64 "imm" %zimm5) "enc"="{ if((vd<vs2)|(vd>=vs2+2)) 0011001 vs2:5 zimm5:5 011 vd:5 1010111 }" "asm"="vrgather.vi {vd}, {vs2}, {zimm5}" "asmonly" {
  ret void
}

define void @VRGATHER_VI.1(i64 "reg"="v" %vd, i64 "reg"="v" %vs2, i64 "imm" %zimm5) "enc"="{ if((vd<vs2)|(vd>=vs2+2)) if(vd!=0) 0011000 vs2:5 zimm5:5 011 vd:5 1010111 }" "asm"="vrgather.vi {vd}, {vs2}, {zimm5}, v0.t" "asmonly" {
  ret void
}

define void @VSLIDEUP_VI.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs2, i64 "imm" %zimm5) "enc"="{ if((vd<vs2)|(vd>=vs2+2)) 0011101 vs2:5 zimm5:5 011 vd:5 1010111 }" "asm"="vslideup.vi {vd}, {vs2}, {zimm5}" "asmonly" {
  ret void
}

define void @VSLIDEUP_VI.1(i64 "reg"="v" %vd, i64 "reg"="v" %vs2, i64 "imm" %zimm5) "enc"="{ if((vd<vs2)|(vd>=vs2+2)) if(vd!=0) 0011100 vs2:5 zimm5:5 011 vd:5 1010111 }" "asm"="vslideup.vi {vd}, {vs2}, {zimm5}, v0.t" "asmonly" {
  ret void
}

define void @VSLIDEDOWN_VI.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs2, i64 "imm" %zimm5) "enc"="0011111 vs2:5 zimm5:5 011 vd:5 1010111" "asm"="vslidedown.vi {vd}, {vs2}, {zimm5}" "asmonly" {
  ret void
}

define void @VSLIDEDOWN_VI.1(i64 "reg"="v" %vd, i64 "reg"="v" %vs2, i64 "imm" %zimm5) "enc"="{ if(vd!=0) 0011110 vs2:5 zimm5:5 011 vd:5 1010111 }" "asm"="vslidedown.vi {vd}, {vs2}, {zimm5}, v0.t" "asmonly" {
  ret void
}

define void @VADC_VIM.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs2, i64 "imm" %simm5) "enc"="{ if(vd!=0) 0100000 vs2:5 simm5:s5 011 vd:5 1010111 }" "asm"="vadc.vim {vd}, {vs2}, {simm5}, v0" "asmonly" {
  ret void
}

define void @VMADC_VIM.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs2, i64 "imm" %simm5) "enc"="0100010 vs2:5 simm5:s5 011 vd:5 1010111" "asm"="vmadc.vim {vd}, {vs2}, {simm5}, v0" "asmonly" {
  ret void
}

define void @VMADC_VI.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs2, i64 "imm" %simm5) "enc"="0100011 vs2:5 simm5:s5 011 vd:5 1010111" "asm"="vmadc.vi {vd}, {vs2}, {simm5}" "asmonly" {
  ret void
}

define void @VMERGE_VIM.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs2, i64 "imm" %simm5) "enc"="{ if(vd!=0) 0101110 vs2:5 simm5:s5 011 vd:5 1010111 }" "asm"="vmerge.vim {vd}, {vs2}, {simm5}, v0" "asmonly" {
  ret void
}

define void @VMV_V_I.0(i64 "reg"="v" %vd, i64 "imm" %simm5) "enc"="010111100000 simm5:s5 011 vd:5 1010111" "asm"="vmv.v.i {vd}, {simm5}" "asmonly" {
  ret void
}

define void @VMSEQ_VI.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs2, i64 "imm" %simm5) "enc"="0110001 vs2:5 simm5:s5 011 vd:5 1010111" "asm"="vmseq.vi {vd}, {vs2}, {simm5}" "asmonly" {
  ret void
}

define void @VMSEQ_VI.1(i64 "reg"="v" %vd, i64 "reg"="v" %vs2, i64 "imm" %simm5) "enc"="0110000 vs2:5 simm5:s5 011 vd:5 1010111" "asm"="vmseq.vi {vd}, {vs2}, {simm5}, v0.t" "asmonly" {
  ret void
}

define void @VMSNE_VI.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs2, i64 "imm" %simm5) "enc"="0110011 vs2:5 simm5:s5 011 vd:5 1010111" "asm"="vmsne.vi {vd}, {vs2}, {simm5}" "asmonly" {
  ret void
}

define void @VMSNE_VI.1(i64 "reg"="v" %vd, i64 "reg"="v" %vs2, i64 "imm" %simm5) "enc"="0110010 vs2:5 simm5:s5 011 vd:5 1010111" "asm"="vmsne.vi {vd}, {vs2}, {simm5}, v0.t" "asmonly" {
  ret void
}

define void @VMSLEU_VI.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs2, i64 "imm" %simm5) "enc"="0111001 vs2:5 simm5:s5 011 vd:5 1010111" "asm"="vmsleu.vi {vd}, {vs2}, {simm5}" "asmonly" {
  ret void
}

define void @VMSLEU_VI.1(i64 "reg"="v" %vd, i64 "reg"="v" %vs2, i64 "imm" %simm5) "enc"="0111000 vs2:5 simm5:s5 011 vd:5 1010111" "asm"="vmsleu.vi {vd}, {vs2}, {simm5}, v0.t" "asmonly" {
  ret void
}

define void @VMSLE_VI.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs2, i64 "imm" %simm5) "enc"="0111011 vs2:5 simm5:s5 011 vd:5 1010111" "asm"="vmsle.vi {vd}, {vs2}, {simm5}" "asmonly" {
  ret void
}

define void @VMSLE_VI.1(i64 "reg"="v" %vd, i64 "reg"="v" %vs2, i64 "imm" %simm5) "enc"="0111010 vs2:5 simm5:s5 011 vd:5 1010111" "asm"="vmsle.vi {vd}, {vs2}, {simm5}, v0.t" "asmonly" {
  ret void
}

define void @VMSGTU_VI.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs2, i64 "imm" %simm5) "enc"="0111101 vs2:5 simm5:s5 011 vd:5 1010111" "asm"="vmsgtu.vi {vd}, {vs2}, {simm5}" "asmonly" {
  ret void
}

define void @VMSGTU_VI.1(i64 "reg"="v" %vd, i64 "reg"="v" %vs2, i64 "imm" %simm5) "enc"="0111100 vs2:5 simm5:s5 011 vd:5 1010111" "asm"="vmsgtu.vi {vd}, {vs2}, {simm5}, v0.t" "asmonly" {
  ret void
}

define void @VMSGT_VI.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs2, i64 "imm" %simm5) "enc"="0111111 vs2:5 simm5:s5 011 vd:5 1010111" "asm"="vmsgt.vi {vd}, {vs2}, {simm5}" "asmonly" {
  ret void
}

define void @VMSGT_VI.1(i64 "reg"="v" %vd, i64 "reg"="v" %vs2, i64 "imm" %simm5) "enc"="0111110 vs2:5 simm5:s5 011 vd:5 1010111" "asm"="vmsgt.vi {vd}, {vs2}, {simm5}, v0.t" "asmonly" {
  ret void
}

define void @VSADDU_VI.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs2, i64 "imm" %simm5) "enc"="1000001 vs2:5 simm5:s5 011 vd:5 1010111" "asm"="vsaddu.vi {vd}, {vs2}, {simm5}" "asmonly" {
  ret void
}

define void @VSADDU_VI.1(i64 "reg"="v" %vd, i64 "reg"="v" %vs2, i64 "imm" %simm5) "enc"="{ if(vd!=0) 1000000 vs2:5 simm5:s5 011 vd:5 1010111 }" "asm"="vsaddu.vi {vd}, {vs2}, {simm5}, v0.t" "asmonly" {
  ret void
}

define void @VSADD_VI.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs2, i64 "imm" %simm5) "enc"="1000011 vs2:5 simm5:s5 011 vd:5 1010111" "asm"="vsadd.vi {vd}, {vs2}, {simm5}" "asmonly" {
  ret void
}

define void @VSADD_VI.1(i64 "reg"="v" %vd, i64 "reg"="v" %vs2, i64 "imm" %simm5) "enc"="{ if(vd!=0) 1000010 vs2:5 simm5:s5 011 vd:5 1010111 }" "asm"="vsadd.vi {vd}, {vs2}, {simm5}, v0.t" "asmonly" {
  ret void
}

define void @VSLL_VI.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs2, i64 "imm" %zimm5) "enc"="1001011 vs2:5 zimm5:5 011 vd:5 1010111" "asm"="vsll.vi {vd}, {vs2}, {zimm5}" "asmonly" {
  ret void
}

define void @VSLL_VI.1(i64 "reg"="v" %vd, i64 "reg"="v" %vs2, i64 "imm" %zimm5) "enc"="{ if(vd!=0) 1001010 vs2:5 zimm5:5 011 vd:5 1010111 }" "asm"="vsll.vi {vd}, {vs2}, {zimm5}, v0.t" "asmonly" {
  ret void
}

define void @VMV1R_V.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="1001111 vs2:5 00000011 vd:5 1010111" "asm"="vmv1r.v {vd}, {vs2}" "asmonly" {
  ret void
}

define void @VMV2R_V.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if(vd%2==0) if(vs2%2==0) 1001111 vs2:5 00001011 vd:5 1010111 }" "asm"="vmv2r.v {vd}, {vs2}" "asmonly" {
  ret void
}

define void @VMV4R_V.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if(vd%4==0) if(vs2%4==0) 1001111 vs2:5 00011011 vd:5 1010111 }" "asm"="vmv4r.v {vd}, {vs2}" "asmonly" {
  ret void
}

define void @VMV8R_V.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if(vd%8==0) if(vs2%8==0) 1001111 vs2:5 00111011 vd:5 1010111 }" "asm"="vmv8r.v {vd}, {vs2}" "asmonly" {
  ret void
}

define void @VSRL_VI.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs2, i64 "imm" %zimm5) "enc"="1010001 vs2:5 zimm5:5 011 vd:5 1010111" "asm"="vsrl.vi {vd}, {vs2}, {zimm5}" "asmonly" {
  ret void
}

define void @VSRL_VI.1(i64 "reg"="v" %vd, i64 "reg"="v" %vs2, i64 "imm" %zimm5) "enc"="{ if(vd!=0) 1010000 vs2:5 zimm5:5 011 vd:5 1010111 }" "asm"="vsrl.vi {vd}, {vs2}, {zimm5}, v0.t" "asmonly" {
  ret void
}

define void @VSRA_VI.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs2, i64 "imm" %zimm5) "enc"="1010011 vs2:5 zimm5:5 011 vd:5 1010111" "asm"="vsra.vi {vd}, {vs2}, {zimm5}" "asmonly" {
  ret void
}

define void @VSRA_VI.1(i64 "reg"="v" %vd, i64 "reg"="v" %vs2, i64 "imm" %zimm5) "enc"="{ if(vd!=0) 1010010 vs2:5 zimm5:5 011 vd:5 1010111 }" "asm"="vsra.vi {vd}, {vs2}, {zimm5}, v0.t" "asmonly" {
  ret void
}

define void @VSSRL_VI.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs2, i64 "imm" %zimm5) "enc"="1010101 vs2:5 zimm5:5 011 vd:5 1010111" "asm"="vssrl.vi {vd}, {vs2}, {zimm5}" "asmonly" {
  ret void
}

define void @VSSRL_VI.1(i64 "reg"="v" %vd, i64 "reg"="v" %vs2, i64 "imm" %zimm5) "enc"="{ if(vd!=0) 1010100 vs2:5 zimm5:5 011 vd:5 1010111 }" "asm"="vssrl.vi {vd}, {vs2}, {zimm5}, v0.t" "asmonly" {
  ret void
}

define void @VSSRA_VI.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs2, i64 "imm" %zimm5) "enc"="1010111 vs2:5 zimm5:5 011 vd:5 1010111" "asm"="vssra.vi {vd}, {vs2}, {zimm5}" "asmonly" {
  ret void
}

define void @VSSRA_VI.1(i64 "reg"="v" %vd, i64 "reg"="v" %vs2, i64 "imm" %zimm5) "enc"="{ if(vd!=0) 1010110 vs2:5 zimm5:5 011 vd:5 1010111 }" "asm"="vssra.vi {vd}, {vs2}, {zimm5}, v0.t" "asmonly" {
  ret void
}

define void @VNSRL_WI.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs2, i64 "imm" %zimm5) "enc"="1011001 vs2:5 zimm5:5 011 vd:5 1010111" "asm"="vnsrl.wi {vd}, {vs2}, {zimm5}" "asmonly" {
  ret void
}

define void @VNSRL_WI.1(i64 "reg"="v" %vd, i64 "reg"="v" %vs2, i64 "imm" %zimm5) "enc"="{ if(vd!=0) 1011000 vs2:5 zimm5:5 011 vd:5 1010111 }" "asm"="vnsrl.wi {vd}, {vs2}, {zimm5}, v0.t" "asmonly" {
  ret void
}

define void @VNSRA_WI.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs2, i64 "imm" %zimm5) "enc"="1011011 vs2:5 zimm5:5 011 vd:5 1010111" "asm"="vnsra.wi {vd}, {vs2}, {zimm5}" "asmonly" {
  ret void
}

define void @VNSRA_WI.1(i64 "reg"="v" %vd, i64 "reg"="v" %vs2, i64 "imm" %zimm5) "enc"="{ if(vd!=0) 1011010 vs2:5 zimm5:5 011 vd:5 1010111 }" "asm"="vnsra.wi {vd}, {vs2}, {zimm5}, v0.t" "asmonly" {
  ret void
}

define void @VNCLIPU_WI.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs2, i64 "imm" %zimm5) "enc"="1011101 vs2:5 zimm5:5 011 vd:5 1010111" "asm"="vnclipu.wi {vd}, {vs2}, {zimm5}" "asmonly" {
  ret void
}

define void @VNCLIPU_WI.1(i64 "reg"="v" %vd, i64 "reg"="v" %vs2, i64 "imm" %zimm5) "enc"="{ if(vd!=0) 1011100 vs2:5 zimm5:5 011 vd:5 1010111 }" "asm"="vnclipu.wi {vd}, {vs2}, {zimm5}, v0.t" "asmonly" {
  ret void
}

define void @VNCLIP_WI.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs2, i64 "imm" %zimm5) "enc"="1011111 vs2:5 zimm5:5 011 vd:5 1010111" "asm"="vnclip.wi {vd}, {vs2}, {zimm5}" "asmonly" {
  ret void
}

define void @VNCLIP_WI.1(i64 "reg"="v" %vd, i64 "reg"="v" %vs2, i64 "imm" %zimm5) "enc"="{ if(vd!=0) 1011110 vs2:5 zimm5:5 011 vd:5 1010111 }" "asm"="vnclip.wi {vd}, {vs2}, {zimm5}, v0.t" "asmonly" {
  ret void
}

define void @VREDSUM_VS.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="0000001 vs2:5 vs1:5 010 vd:5 1010111" "asm"="vredsum.vs {vd}, {vs2}, {vs1}" "asmonly" {
  ret void
}

define void @VREDSUM_VS.1(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="0000000 vs2:5 vs1:5 010 vd:5 1010111" "asm"="vredsum.vs {vd}, {vs2}, {vs1}, v0.t" "asmonly" {
  ret void
}

define void @VREDAND_VS.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="0000011 vs2:5 vs1:5 010 vd:5 1010111" "asm"="vredand.vs {vd}, {vs2}, {vs1}" "asmonly" {
  ret void
}

define void @VREDAND_VS.1(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="0000010 vs2:5 vs1:5 010 vd:5 1010111" "asm"="vredand.vs {vd}, {vs2}, {vs1}, v0.t" "asmonly" {
  ret void
}

define void @VREDOR_VS.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="0000101 vs2:5 vs1:5 010 vd:5 1010111" "asm"="vredor.vs {vd}, {vs2}, {vs1}" "asmonly" {
  ret void
}

define void @VREDOR_VS.1(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="0000100 vs2:5 vs1:5 010 vd:5 1010111" "asm"="vredor.vs {vd}, {vs2}, {vs1}, v0.t" "asmonly" {
  ret void
}

define void @VREDXOR_VS.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="0000111 vs2:5 vs1:5 010 vd:5 1010111" "asm"="vredxor.vs {vd}, {vs2}, {vs1}" "asmonly" {
  ret void
}

define void @VREDXOR_VS.1(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="0000110 vs2:5 vs1:5 010 vd:5 1010111" "asm"="vredxor.vs {vd}, {vs2}, {vs1}, v0.t" "asmonly" {
  ret void
}

define void @VREDMINU_VS.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="0001001 vs2:5 vs1:5 010 vd:5 1010111" "asm"="vredminu.vs {vd}, {vs2}, {vs1}" "asmonly" {
  ret void
}

define void @VREDMINU_VS.1(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="0001000 vs2:5 vs1:5 010 vd:5 1010111" "asm"="vredminu.vs {vd}, {vs2}, {vs1}, v0.t" "asmonly" {
  ret void
}

define void @VREDMIN_VS.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="0001011 vs2:5 vs1:5 010 vd:5 1010111" "asm"="vredmin.vs {vd}, {vs2}, {vs1}" "asmonly" {
  ret void
}

define void @VREDMIN_VS.1(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="0001010 vs2:5 vs1:5 010 vd:5 1010111" "asm"="vredmin.vs {vd}, {vs2}, {vs1}, v0.t" "asmonly" {
  ret void
}

define void @VREDMAXU_VS.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="0001101 vs2:5 vs1:5 010 vd:5 1010111" "asm"="vredmaxu.vs {vd}, {vs2}, {vs1}" "asmonly" {
  ret void
}

define void @VREDMAXU_VS.1(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="0001100 vs2:5 vs1:5 010 vd:5 1010111" "asm"="vredmaxu.vs {vd}, {vs2}, {vs1}, v0.t" "asmonly" {
  ret void
}

define void @VREDMAX_VS.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="0001111 vs2:5 vs1:5 010 vd:5 1010111" "asm"="vredmax.vs {vd}, {vs2}, {vs1}" "asmonly" {
  ret void
}

define void @VREDMAX_VS.1(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="0001110 vs2:5 vs1:5 010 vd:5 1010111" "asm"="vredmax.vs {vd}, {vs2}, {vs1}, v0.t" "asmonly" {
  ret void
}

define void @VAADDU_VV.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="0010001 vs2:5 vs1:5 010 vd:5 1010111" "asm"="vaaddu.vv {vd}, {vs2}, {vs1}" "asmonly" {
  ret void
}

define void @VAADDU_VV.1(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="{ if(vd!=0) 0010000 vs2:5 vs1:5 010 vd:5 1010111 }" "asm"="vaaddu.vv {vd}, {vs2}, {vs1}, v0.t" "asmonly" {
  ret void
}

define void @VAADD_VV.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="0010011 vs2:5 vs1:5 010 vd:5 1010111" "asm"="vaadd.vv {vd}, {vs2}, {vs1}" "asmonly" {
  ret void
}

define void @VAADD_VV.1(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="{ if(vd!=0) 0010010 vs2:5 vs1:5 010 vd:5 1010111 }" "asm"="vaadd.vv {vd}, {vs2}, {vs1}, v0.t" "asmonly" {
  ret void
}

define void @VASUBU_VV.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="0010101 vs2:5 vs1:5 010 vd:5 1010111" "asm"="vasubu.vv {vd}, {vs2}, {vs1}" "asmonly" {
  ret void
}

define void @VASUBU_VV.1(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="{ if(vd!=0) 0010100 vs2:5 vs1:5 010 vd:5 1010111 }" "asm"="vasubu.vv {vd}, {vs2}, {vs1}, v0.t" "asmonly" {
  ret void
}

define void @VASUB_VV.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="0010111 vs2:5 vs1:5 010 vd:5 1010111" "asm"="vasub.vv {vd}, {vs2}, {vs1}" "asmonly" {
  ret void
}

define void @VASUB_VV.1(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="{ if(vd!=0) 0010110 vs2:5 vs1:5 010 vd:5 1010111 }" "asm"="vasub.vv {vd}, {vs2}, {vs1}, v0.t" "asmonly" {
  ret void
}

define void @VMV_X_S.0(i64 "reg"="x" %rd, i64 "reg"="v" %vs2) "enc"="0100001 vs2:5 00000010 rd:5 1010111" "asm"="vmv.x.s {rd}, {vs2}" "asmonly" {
  ret void
}

define void @VZEXT_VF8.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="0100101 vs2:5 00010010 vd:5 1010111" "asm"="vzext.vf8 {vd}, {vs2}" "asmonly" {
  ret void
}

define void @VZEXT_VF8.1(i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if(vd!=0) 0100100 vs2:5 00010010 vd:5 1010111 }" "asm"="vzext.vf8 {vd}, {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VSEXT_VF8.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="0100101 vs2:5 00011010 vd:5 1010111" "asm"="vsext.vf8 {vd}, {vs2}" "asmonly" {
  ret void
}

define void @VSEXT_VF8.1(i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if(vd!=0) 0100100 vs2:5 00011010 vd:5 1010111 }" "asm"="vsext.vf8 {vd}, {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VZEXT_VF4.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="0100101 vs2:5 00100010 vd:5 1010111" "asm"="vzext.vf4 {vd}, {vs2}" "asmonly" {
  ret void
}

define void @VZEXT_VF4.1(i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if(vd!=0) 0100100 vs2:5 00100010 vd:5 1010111 }" "asm"="vzext.vf4 {vd}, {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VSEXT_VF4.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="0100101 vs2:5 00101010 vd:5 1010111" "asm"="vsext.vf4 {vd}, {vs2}" "asmonly" {
  ret void
}

define void @VSEXT_VF4.1(i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if(vd!=0) 0100100 vs2:5 00101010 vd:5 1010111 }" "asm"="vsext.vf4 {vd}, {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VZEXT_VF2.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="0100101 vs2:5 00110010 vd:5 1010111" "asm"="vzext.vf2 {vd}, {vs2}" "asmonly" {
  ret void
}

define void @VZEXT_VF2.1(i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if(vd!=0) 0100100 vs2:5 00110010 vd:5 1010111 }" "asm"="vzext.vf2 {vd}, {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VSEXT_VF2.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="0100101 vs2:5 00111010 vd:5 1010111" "asm"="vsext.vf2 {vd}, {vs2}" "asmonly" {
  ret void
}

define void @VSEXT_VF2.1(i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if(vd!=0) 0100100 vs2:5 00111010 vd:5 1010111 }" "asm"="vsext.vf2 {vd}, {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VCOMPRESS_VM.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="{ if((vd<vs1)|(vd>=vs1+2)) if((vd<vs2)|(vd>=vs2+2)) 0101111 vs2:5 vs1:5 010 vd:5 1010111 }" "asm"="vcompress.vm {vd}, {vs2}, {vs1}" "asmonly" {
  ret void
}

define void @VMANDN_MM.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="0110001 vs2:5 vs1:5 010 vd:5 1010111" "asm"="vmandn.mm {vd}, {vs2}, {vs1}" "asmonly" {
  ret void
}

define void @VMAND_MM.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="0110011 vs2:5 vs1:5 010 vd:5 1010111" "asm"="vmand.mm {vd}, {vs2}, {vs1}" "asmonly" {
  ret void
}

define void @VMOR_MM.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="0110101 vs2:5 vs1:5 010 vd:5 1010111" "asm"="vmor.mm {vd}, {vs2}, {vs1}" "asmonly" {
  ret void
}

define void @VMXOR_MM.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="0110111 vs2:5 vs1:5 010 vd:5 1010111" "asm"="vmxor.mm {vd}, {vs2}, {vs1}" "asmonly" {
  ret void
}

define void @VMORN_MM.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="0111001 vs2:5 vs1:5 010 vd:5 1010111" "asm"="vmorn.mm {vd}, {vs2}, {vs1}" "asmonly" {
  ret void
}

define void @VMNAND_MM.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="0111011 vs2:5 vs1:5 010 vd:5 1010111" "asm"="vmnand.mm {vd}, {vs2}, {vs1}" "asmonly" {
  ret void
}

define void @VMNOR_MM.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="0111101 vs2:5 vs1:5 010 vd:5 1010111" "asm"="vmnor.mm {vd}, {vs2}, {vs1}" "asmonly" {
  ret void
}

define void @VMXNOR_MM.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="0111111 vs2:5 vs1:5 010 vd:5 1010111" "asm"="vmxnor.mm {vd}, {vs2}, {vs1}" "asmonly" {
  ret void
}

define void @VMSBF_M.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if((vd<vs2)|(vd>=vs2+2)) 0101001 vs2:5 00001010 vd:5 1010111 }" "asm"="vmsbf.m {vd}, {vs2}" "asmonly" {
  ret void
}

define void @VMSBF_M.1(i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if((vd<vs2)|(vd>=vs2+2)) if(vd!=0) 0101000 vs2:5 00001010 vd:5 1010111 }" "asm"="vmsbf.m {vd}, {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VMSOF_M.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if((vd<vs2)|(vd>=vs2+2)) 0101001 vs2:5 00010010 vd:5 1010111 }" "asm"="vmsof.m {vd}, {vs2}" "asmonly" {
  ret void
}

define void @VMSOF_M.1(i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if((vd<vs2)|(vd>=vs2+2)) if(vd!=0) 0101000 vs2:5 00010010 vd:5 1010111 }" "asm"="vmsof.m {vd}, {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VMSIF_M.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if((vd<vs2)|(vd>=vs2+2)) 0101001 vs2:5 00011010 vd:5 1010111 }" "asm"="vmsif.m {vd}, {vs2}" "asmonly" {
  ret void
}

define void @VMSIF_M.1(i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if((vd<vs2)|(vd>=vs2+2)) if(vd!=0) 0101000 vs2:5 00011010 vd:5 1010111 }" "asm"="vmsif.m {vd}, {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VIOTA_M.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if((vd<vs2)|(vd>=vs2+2)) 0101001 vs2:5 10000010 vd:5 1010111 }" "asm"="viota.m {vd}, {vs2}" "asmonly" {
  ret void
}

define void @VIOTA_M.1(i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if((vd<vs2)|(vd>=vs2+2)) if(vd!=0) 0101000 vs2:5 10000010 vd:5 1010111 }" "asm"="viota.m {vd}, {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VID_V.0(i64 "reg"="v" %vd) "enc"="01010010000010001010 vd:5 1010111" "asm"="vid.v {vd}" "asmonly" {
  ret void
}

define void @VID_V.1(i64 "reg"="v" %vd) "enc"="{ if(vd!=0) 01010000000010001010 vd:5 1010111 }" "asm"="vid.v {vd}, v0.t" "asmonly" {
  ret void
}

define void @VCPOP_M.0(i64 "reg"="x" %rd, i64 "reg"="v" %vs2) "enc"="0100001 vs2:5 10000010 rd:5 1010111" "asm"="vcpop.m {rd}, {vs2}" "asmonly" {
  ret void
}

define void @VCPOP_M.1(i64 "reg"="x" %rd, i64 "reg"="v" %vs2) "enc"="0100000 vs2:5 10000010 rd:5 1010111" "asm"="vcpop.m {rd}, {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VFIRST_M.0(i64 "reg"="x" %rd, i64 "reg"="v" %vs2) "enc"="0100001 vs2:5 10001010 rd:5 1010111" "asm"="vfirst.m {rd}, {vs2}" "asmonly" {
  ret void
}

define void @VFIRST_M.1(i64 "reg"="x" %rd, i64 "reg"="v" %vs2) "enc"="0100000 vs2:5 10001010 rd:5 1010111" "asm"="vfirst.m {rd}, {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VDIVU_VV.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="1000001 vs2:5 vs1:5 010 vd:5 1010111" "asm"="vdivu.vv {vd}, {vs2}, {vs1}" "asmonly" {
  ret void
}

define void @VDIVU_VV.1(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="{ if(vd!=0) 1000000 vs2:5 vs1:5 010 vd:5 1010111 }" "asm"="vdivu.vv {vd}, {vs2}, {vs1}, v0.t" "asmonly" {
  ret void
}

define void @VDIV_VV.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="1000011 vs2:5 vs1:5 010 vd:5 1010111" "asm"="vdiv.vv {vd}, {vs2}, {vs1}" "asmonly" {
  ret void
}

define void @VDIV_VV.1(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="{ if(vd!=0) 1000010 vs2:5 vs1:5 010 vd:5 1010111 }" "asm"="vdiv.vv {vd}, {vs2}, {vs1}, v0.t" "asmonly" {
  ret void
}

define void @VREMU_VV.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="1000101 vs2:5 vs1:5 010 vd:5 1010111" "asm"="vremu.vv {vd}, {vs2}, {vs1}" "asmonly" {
  ret void
}

define void @VREMU_VV.1(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="{ if(vd!=0) 1000100 vs2:5 vs1:5 010 vd:5 1010111 }" "asm"="vremu.vv {vd}, {vs2}, {vs1}, v0.t" "asmonly" {
  ret void
}

define void @VREM_VV.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="1000111 vs2:5 vs1:5 010 vd:5 1010111" "asm"="vrem.vv {vd}, {vs2}, {vs1}" "asmonly" {
  ret void
}

define void @VREM_VV.1(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="{ if(vd!=0) 1000110 vs2:5 vs1:5 010 vd:5 1010111 }" "asm"="vrem.vv {vd}, {vs2}, {vs1}, v0.t" "asmonly" {
  ret void
}

define void @VMULHU_VV.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="1001001 vs2:5 vs1:5 010 vd:5 1010111" "asm"="vmulhu.vv {vd}, {vs2}, {vs1}" "asmonly" {
  ret void
}

define void @VMULHU_VV.1(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="{ if(vd!=0) 1001000 vs2:5 vs1:5 010 vd:5 1010111 }" "asm"="vmulhu.vv {vd}, {vs2}, {vs1}, v0.t" "asmonly" {
  ret void
}

define void @VMUL_VV.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="1001011 vs2:5 vs1:5 010 vd:5 1010111" "asm"="vmul.vv {vd}, {vs2}, {vs1}" "asmonly" {
  ret void
}

define void @VMUL_VV.1(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="{ if(vd!=0) 1001010 vs2:5 vs1:5 010 vd:5 1010111 }" "asm"="vmul.vv {vd}, {vs2}, {vs1}, v0.t" "asmonly" {
  ret void
}

define void @VMULHSU_VV.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="1001101 vs2:5 vs1:5 010 vd:5 1010111" "asm"="vmulhsu.vv {vd}, {vs2}, {vs1}" "asmonly" {
  ret void
}

define void @VMULHSU_VV.1(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="{ if(vd!=0) 1001100 vs2:5 vs1:5 010 vd:5 1010111 }" "asm"="vmulhsu.vv {vd}, {vs2}, {vs1}, v0.t" "asmonly" {
  ret void
}

define void @VMULH_VV.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="1001111 vs2:5 vs1:5 010 vd:5 1010111" "asm"="vmulh.vv {vd}, {vs2}, {vs1}" "asmonly" {
  ret void
}

define void @VMULH_VV.1(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="{ if(vd!=0) 1001110 vs2:5 vs1:5 010 vd:5 1010111 }" "asm"="vmulh.vv {vd}, {vs2}, {vs1}, v0.t" "asmonly" {
  ret void
}

define void @VMADD_VV.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="1010011 vs2:5 vs1:5 010 vd:5 1010111" "asm"="vmadd.vv {vd}, {vs1}, {vs2}" "asmonly" {
  ret void
}

define void @VMADD_VV.1(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="{ if(vd!=0) 1010010 vs2:5 vs1:5 010 vd:5 1010111 }" "asm"="vmadd.vv {vd}, {vs1}, {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VNMSUB_VV.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="1010111 vs2:5 vs1:5 010 vd:5 1010111" "asm"="vnmsub.vv {vd}, {vs1}, {vs2}" "asmonly" {
  ret void
}

define void @VNMSUB_VV.1(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="{ if(vd!=0) 1010110 vs2:5 vs1:5 010 vd:5 1010111 }" "asm"="vnmsub.vv {vd}, {vs1}, {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VMACC_VV.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="1011011 vs2:5 vs1:5 010 vd:5 1010111" "asm"="vmacc.vv {vd}, {vs1}, {vs2}" "asmonly" {
  ret void
}

define void @VMACC_VV.1(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="{ if(vd!=0) 1011010 vs2:5 vs1:5 010 vd:5 1010111 }" "asm"="vmacc.vv {vd}, {vs1}, {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VNMSAC_VV.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="1011111 vs2:5 vs1:5 010 vd:5 1010111" "asm"="vnmsac.vv {vd}, {vs1}, {vs2}" "asmonly" {
  ret void
}

define void @VNMSAC_VV.1(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="{ if(vd!=0) 1011110 vs2:5 vs1:5 010 vd:5 1010111 }" "asm"="vnmsac.vv {vd}, {vs1}, {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VWADDU_VV.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="{ if((vd<vs1)|(vd>=vs1+2)) if((vd<vs2)|(vd>=vs2+2)) 1100001 vs2:5 vs1:5 010 vd:5 1010111 }" "asm"="vwaddu.vv {vd}, {vs2}, {vs1}" "asmonly" {
  ret void
}

define void @VWADDU_VV.1(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="{ if((vd<vs1)|(vd>=vs1+2)) if((vd<vs2)|(vd>=vs2+2)) if(vd!=0) 1100000 vs2:5 vs1:5 010 vd:5 1010111 }" "asm"="vwaddu.vv {vd}, {vs2}, {vs1}, v0.t" "asmonly" {
  ret void
}

define void @VWADD_VV.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="{ if((vd<vs1)|(vd>=vs1+2)) if((vd<vs2)|(vd>=vs2+2)) 1100011 vs2:5 vs1:5 010 vd:5 1010111 }" "asm"="vwadd.vv {vd}, {vs2}, {vs1}" "asmonly" {
  ret void
}

define void @VWADD_VV.1(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="{ if((vd<vs1)|(vd>=vs1+2)) if((vd<vs2)|(vd>=vs2+2)) if(vd!=0) 1100010 vs2:5 vs1:5 010 vd:5 1010111 }" "asm"="vwadd.vv {vd}, {vs2}, {vs1}, v0.t" "asmonly" {
  ret void
}

define void @VWSUBU_VV.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="{ if((vd<vs1)|(vd>=vs1+2)) if((vd<vs2)|(vd>=vs2+2)) 1100101 vs2:5 vs1:5 010 vd:5 1010111 }" "asm"="vwsubu.vv {vd}, {vs2}, {vs1}" "asmonly" {
  ret void
}

define void @VWSUBU_VV.1(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="{ if((vd<vs1)|(vd>=vs1+2)) if((vd<vs2)|(vd>=vs2+2)) if(vd!=0) 1100100 vs2:5 vs1:5 010 vd:5 1010111 }" "asm"="vwsubu.vv {vd}, {vs2}, {vs1}, v0.t" "asmonly" {
  ret void
}

define void @VWSUB_VV.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="{ if((vd<vs1)|(vd>=vs1+2)) if((vd<vs2)|(vd>=vs2+2)) 1100111 vs2:5 vs1:5 010 vd:5 1010111 }" "asm"="vwsub.vv {vd}, {vs2}, {vs1}" "asmonly" {
  ret void
}

define void @VWSUB_VV.1(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="{ if((vd<vs1)|(vd>=vs1+2)) if((vd<vs2)|(vd>=vs2+2)) if(vd!=0) 1100110 vs2:5 vs1:5 010 vd:5 1010111 }" "asm"="vwsub.vv {vd}, {vs2}, {vs1}, v0.t" "asmonly" {
  ret void
}

define void @VWADDU_WV.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="{ if((vd<vs1)|(vd>=vs1+2)) 1101001 vs2:5 vs1:5 010 vd:5 1010111 }" "asm"="vwaddu.wv {vd}, {vs2}, {vs1}" "asmonly" {
  ret void
}

define void @VWADDU_WV.1(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="{ if((vd<vs1)|(vd>=vs1+2)) if(vd!=0) 1101000 vs2:5 vs1:5 010 vd:5 1010111 }" "asm"="vwaddu.wv {vd}, {vs2}, {vs1}, v0.t" "asmonly" {
  ret void
}

define void @VWADD_WV.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="{ if((vd<vs1)|(vd>=vs1+2)) 1101011 vs2:5 vs1:5 010 vd:5 1010111 }" "asm"="vwadd.wv {vd}, {vs2}, {vs1}" "asmonly" {
  ret void
}

define void @VWADD_WV.1(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="{ if((vd<vs1)|(vd>=vs1+2)) if(vd!=0) 1101010 vs2:5 vs1:5 010 vd:5 1010111 }" "asm"="vwadd.wv {vd}, {vs2}, {vs1}, v0.t" "asmonly" {
  ret void
}

define void @VWSUBU_WV.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="{ if((vd<vs1)|(vd>=vs1+2)) 1101101 vs2:5 vs1:5 010 vd:5 1010111 }" "asm"="vwsubu.wv {vd}, {vs2}, {vs1}" "asmonly" {
  ret void
}

define void @VWSUBU_WV.1(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="{ if((vd<vs1)|(vd>=vs1+2)) if(vd!=0) 1101100 vs2:5 vs1:5 010 vd:5 1010111 }" "asm"="vwsubu.wv {vd}, {vs2}, {vs1}, v0.t" "asmonly" {
  ret void
}

define void @VWSUB_WV.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="{ if((vd<vs1)|(vd>=vs1+2)) 1101111 vs2:5 vs1:5 010 vd:5 1010111 }" "asm"="vwsub.wv {vd}, {vs2}, {vs1}" "asmonly" {
  ret void
}

define void @VWSUB_WV.1(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="{ if((vd<vs1)|(vd>=vs1+2)) if(vd!=0) 1101110 vs2:5 vs1:5 010 vd:5 1010111 }" "asm"="vwsub.wv {vd}, {vs2}, {vs1}, v0.t" "asmonly" {
  ret void
}

define void @VWMULU_VV.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="{ if((vd<vs1)|(vd>=vs1+2)) if((vd<vs2)|(vd>=vs2+2)) 1110001 vs2:5 vs1:5 010 vd:5 1010111 }" "asm"="vwmulu.vv {vd}, {vs2}, {vs1}" "asmonly" {
  ret void
}

define void @VWMULU_VV.1(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="{ if((vd<vs1)|(vd>=vs1+2)) if((vd<vs2)|(vd>=vs2+2)) if(vd!=0) 1110000 vs2:5 vs1:5 010 vd:5 1010111 }" "asm"="vwmulu.vv {vd}, {vs2}, {vs1}, v0.t" "asmonly" {
  ret void
}

define void @VWMULSU_VV.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="{ if((vd<vs1)|(vd>=vs1+2)) if((vd<vs2)|(vd>=vs2+2)) 1110101 vs2:5 vs1:5 010 vd:5 1010111 }" "asm"="vwmulsu.vv {vd}, {vs2}, {vs1}" "asmonly" {
  ret void
}

define void @VWMULSU_VV.1(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="{ if((vd<vs1)|(vd>=vs1+2)) if((vd<vs2)|(vd>=vs2+2)) if(vd!=0) 1110100 vs2:5 vs1:5 010 vd:5 1010111 }" "asm"="vwmulsu.vv {vd}, {vs2}, {vs1}, v0.t" "asmonly" {
  ret void
}

define void @VWMUL_VV.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="{ if((vd<vs1)|(vd>=vs1+2)) if((vd<vs2)|(vd>=vs2+2)) 1110111 vs2:5 vs1:5 010 vd:5 1010111 }" "asm"="vwmul.vv {vd}, {vs2}, {vs1}" "asmonly" {
  ret void
}

define void @VWMUL_VV.1(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="{ if((vd<vs1)|(vd>=vs1+2)) if((vd<vs2)|(vd>=vs2+2)) if(vd!=0) 1110110 vs2:5 vs1:5 010 vd:5 1010111 }" "asm"="vwmul.vv {vd}, {vs2}, {vs1}, v0.t" "asmonly" {
  ret void
}

define void @VWMACCU_VV.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="{ if((vd<vs1)|(vd>=vs1+2)) if((vd<vs2)|(vd>=vs2+2)) 1111001 vs2:5 vs1:5 010 vd:5 1010111 }" "asm"="vwmaccu.vv {vd}, {vs1}, {vs2}" "asmonly" {
  ret void
}

define void @VWMACCU_VV.1(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="{ if((vd<vs1)|(vd>=vs1+2)) if((vd<vs2)|(vd>=vs2+2)) if(vd!=0) 1111000 vs2:5 vs1:5 010 vd:5 1010111 }" "asm"="vwmaccu.vv {vd}, {vs1}, {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VWMACC_VV.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="{ if((vd<vs1)|(vd>=vs1+2)) if((vd<vs2)|(vd>=vs2+2)) 1111011 vs2:5 vs1:5 010 vd:5 1010111 }" "asm"="vwmacc.vv {vd}, {vs1}, {vs2}" "asmonly" {
  ret void
}

define void @VWMACC_VV.1(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="{ if((vd<vs1)|(vd>=vs1+2)) if((vd<vs2)|(vd>=vs2+2)) if(vd!=0) 1111010 vs2:5 vs1:5 010 vd:5 1010111 }" "asm"="vwmacc.vv {vd}, {vs1}, {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VWMACCSU_VV.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="{ if((vd<vs1)|(vd>=vs1+2)) if((vd<vs2)|(vd>=vs2+2)) 1111111 vs2:5 vs1:5 010 vd:5 1010111 }" "asm"="vwmaccsu.vv {vd}, {vs1}, {vs2}" "asmonly" {
  ret void
}

define void @VWMACCSU_VV.1(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="{ if((vd<vs1)|(vd>=vs1+2)) if((vd<vs2)|(vd>=vs2+2)) if(vd!=0) 1111110 vs2:5 vs1:5 010 vd:5 1010111 }" "asm"="vwmaccsu.vv {vd}, {vs1}, {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VAADDU_VX.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="0010001 vs2:5 rs1:5 110 vd:5 1010111" "asm"="vaaddu.vx {vd}, {vs2}, {rs1}" "asmonly" {
  ret void
}

define void @VAADDU_VX.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if(vd!=0) 0010000 vs2:5 rs1:5 110 vd:5 1010111 }" "asm"="vaaddu.vx {vd}, {vs2}, {rs1}, v0.t" "asmonly" {
  ret void
}

define void @VAADD_VX.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="0010011 vs2:5 rs1:5 110 vd:5 1010111" "asm"="vaadd.vx {vd}, {vs2}, {rs1}" "asmonly" {
  ret void
}

define void @VAADD_VX.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if(vd!=0) 0010010 vs2:5 rs1:5 110 vd:5 1010111 }" "asm"="vaadd.vx {vd}, {vs2}, {rs1}, v0.t" "asmonly" {
  ret void
}

define void @VASUBU_VX.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="0010101 vs2:5 rs1:5 110 vd:5 1010111" "asm"="vasubu.vx {vd}, {vs2}, {rs1}" "asmonly" {
  ret void
}

define void @VASUBU_VX.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if(vd!=0) 0010100 vs2:5 rs1:5 110 vd:5 1010111 }" "asm"="vasubu.vx {vd}, {vs2}, {rs1}, v0.t" "asmonly" {
  ret void
}

define void @VASUB_VX.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="0010111 vs2:5 rs1:5 110 vd:5 1010111" "asm"="vasub.vx {vd}, {vs2}, {rs1}" "asmonly" {
  ret void
}

define void @VASUB_VX.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if(vd!=0) 0010110 vs2:5 rs1:5 110 vd:5 1010111 }" "asm"="vasub.vx {vd}, {vs2}, {rs1}, v0.t" "asmonly" {
  ret void
}

define void @VMV_S_X.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd) "enc"="010000100000 rs1:5 110 vd:5 1010111" "asm"="vmv.s.x {vd}, {rs1}" "asmonly" {
  ret void
}

define void @VSLIDE1UP_VX.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if((vd<vs2)|(vd>=vs2+2)) 0011101 vs2:5 rs1:5 110 vd:5 1010111 }" "asm"="vslide1up.vx {vd}, {vs2}, {rs1}" "asmonly" {
  ret void
}

define void @VSLIDE1UP_VX.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if((vd<vs2)|(vd>=vs2+2)) if(vd!=0) 0011100 vs2:5 rs1:5 110 vd:5 1010111 }" "asm"="vslide1up.vx {vd}, {vs2}, {rs1}, v0.t" "asmonly" {
  ret void
}

define void @VSLIDE1DOWN_VX.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="0011111 vs2:5 rs1:5 110 vd:5 1010111" "asm"="vslide1down.vx {vd}, {vs2}, {rs1}" "asmonly" {
  ret void
}

define void @VSLIDE1DOWN_VX.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if(vd!=0) 0011110 vs2:5 rs1:5 110 vd:5 1010111 }" "asm"="vslide1down.vx {vd}, {vs2}, {rs1}, v0.t" "asmonly" {
  ret void
}

define void @VDIVU_VX.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="1000001 vs2:5 rs1:5 110 vd:5 1010111" "asm"="vdivu.vx {vd}, {vs2}, {rs1}" "asmonly" {
  ret void
}

define void @VDIVU_VX.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if(vd!=0) 1000000 vs2:5 rs1:5 110 vd:5 1010111 }" "asm"="vdivu.vx {vd}, {vs2}, {rs1}, v0.t" "asmonly" {
  ret void
}

define void @VDIV_VX.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="1000011 vs2:5 rs1:5 110 vd:5 1010111" "asm"="vdiv.vx {vd}, {vs2}, {rs1}" "asmonly" {
  ret void
}

define void @VDIV_VX.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if(vd!=0) 1000010 vs2:5 rs1:5 110 vd:5 1010111 }" "asm"="vdiv.vx {vd}, {vs2}, {rs1}, v0.t" "asmonly" {
  ret void
}

define void @VREMU_VX.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="1000101 vs2:5 rs1:5 110 vd:5 1010111" "asm"="vremu.vx {vd}, {vs2}, {rs1}" "asmonly" {
  ret void
}

define void @VREMU_VX.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if(vd!=0) 1000100 vs2:5 rs1:5 110 vd:5 1010111 }" "asm"="vremu.vx {vd}, {vs2}, {rs1}, v0.t" "asmonly" {
  ret void
}

define void @VREM_VX.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="1000111 vs2:5 rs1:5 110 vd:5 1010111" "asm"="vrem.vx {vd}, {vs2}, {rs1}" "asmonly" {
  ret void
}

define void @VREM_VX.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if(vd!=0) 1000110 vs2:5 rs1:5 110 vd:5 1010111 }" "asm"="vrem.vx {vd}, {vs2}, {rs1}, v0.t" "asmonly" {
  ret void
}

define void @VMULHU_VX.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="1001001 vs2:5 rs1:5 110 vd:5 1010111" "asm"="vmulhu.vx {vd}, {vs2}, {rs1}" "asmonly" {
  ret void
}

define void @VMULHU_VX.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if(vd!=0) 1001000 vs2:5 rs1:5 110 vd:5 1010111 }" "asm"="vmulhu.vx {vd}, {vs2}, {rs1}, v0.t" "asmonly" {
  ret void
}

define void @VMUL_VX.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="1001011 vs2:5 rs1:5 110 vd:5 1010111" "asm"="vmul.vx {vd}, {vs2}, {rs1}" "asmonly" {
  ret void
}

define void @VMUL_VX.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if(vd!=0) 1001010 vs2:5 rs1:5 110 vd:5 1010111 }" "asm"="vmul.vx {vd}, {vs2}, {rs1}, v0.t" "asmonly" {
  ret void
}

define void @VMULHSU_VX.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="1001101 vs2:5 rs1:5 110 vd:5 1010111" "asm"="vmulhsu.vx {vd}, {vs2}, {rs1}" "asmonly" {
  ret void
}

define void @VMULHSU_VX.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if(vd!=0) 1001100 vs2:5 rs1:5 110 vd:5 1010111 }" "asm"="vmulhsu.vx {vd}, {vs2}, {rs1}, v0.t" "asmonly" {
  ret void
}

define void @VMULH_VX.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="1001111 vs2:5 rs1:5 110 vd:5 1010111" "asm"="vmulh.vx {vd}, {vs2}, {rs1}" "asmonly" {
  ret void
}

define void @VMULH_VX.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if(vd!=0) 1001110 vs2:5 rs1:5 110 vd:5 1010111 }" "asm"="vmulh.vx {vd}, {vs2}, {rs1}, v0.t" "asmonly" {
  ret void
}

define void @VMADD_VX.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="1010011 vs2:5 rs1:5 110 vd:5 1010111" "asm"="vmadd.vx {vd}, {rs1}, {vs2}" "asmonly" {
  ret void
}

define void @VMADD_VX.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if(vd!=0) 1010010 vs2:5 rs1:5 110 vd:5 1010111 }" "asm"="vmadd.vx {vd}, {rs1}, {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VNMSUB_VX.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="1010111 vs2:5 rs1:5 110 vd:5 1010111" "asm"="vnmsub.vx {vd}, {rs1}, {vs2}" "asmonly" {
  ret void
}

define void @VNMSUB_VX.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if(vd!=0) 1010110 vs2:5 rs1:5 110 vd:5 1010111 }" "asm"="vnmsub.vx {vd}, {rs1}, {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VMACC_VX.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="1011011 vs2:5 rs1:5 110 vd:5 1010111" "asm"="vmacc.vx {vd}, {rs1}, {vs2}" "asmonly" {
  ret void
}

define void @VMACC_VX.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if(vd!=0) 1011010 vs2:5 rs1:5 110 vd:5 1010111 }" "asm"="vmacc.vx {vd}, {rs1}, {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VNMSAC_VX.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="1011111 vs2:5 rs1:5 110 vd:5 1010111" "asm"="vnmsac.vx {vd}, {rs1}, {vs2}" "asmonly" {
  ret void
}

define void @VNMSAC_VX.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if(vd!=0) 1011110 vs2:5 rs1:5 110 vd:5 1010111 }" "asm"="vnmsac.vx {vd}, {rs1}, {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VWADDU_VX.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if((vd<vs2)|(vd>=vs2+2)) 1100001 vs2:5 rs1:5 110 vd:5 1010111 }" "asm"="vwaddu.vx {vd}, {vs2}, {rs1}" "asmonly" {
  ret void
}

define void @VWADDU_VX.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if((vd<vs2)|(vd>=vs2+2)) if(vd!=0) 1100000 vs2:5 rs1:5 110 vd:5 1010111 }" "asm"="vwaddu.vx {vd}, {vs2}, {rs1}, v0.t" "asmonly" {
  ret void
}

define void @VWADD_VX.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if((vd<vs2)|(vd>=vs2+2)) 1100011 vs2:5 rs1:5 110 vd:5 1010111 }" "asm"="vwadd.vx {vd}, {vs2}, {rs1}" "asmonly" {
  ret void
}

define void @VWADD_VX.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if((vd<vs2)|(vd>=vs2+2)) if(vd!=0) 1100010 vs2:5 rs1:5 110 vd:5 1010111 }" "asm"="vwadd.vx {vd}, {vs2}, {rs1}, v0.t" "asmonly" {
  ret void
}

define void @VWSUBU_VX.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if((vd<vs2)|(vd>=vs2+2)) 1100101 vs2:5 rs1:5 110 vd:5 1010111 }" "asm"="vwsubu.vx {vd}, {vs2}, {rs1}" "asmonly" {
  ret void
}

define void @VWSUBU_VX.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if((vd<vs2)|(vd>=vs2+2)) if(vd!=0) 1100100 vs2:5 rs1:5 110 vd:5 1010111 }" "asm"="vwsubu.vx {vd}, {vs2}, {rs1}, v0.t" "asmonly" {
  ret void
}

define void @VWSUB_VX.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if((vd<vs2)|(vd>=vs2+2)) 1100111 vs2:5 rs1:5 110 vd:5 1010111 }" "asm"="vwsub.vx {vd}, {vs2}, {rs1}" "asmonly" {
  ret void
}

define void @VWSUB_VX.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if((vd<vs2)|(vd>=vs2+2)) if(vd!=0) 1100110 vs2:5 rs1:5 110 vd:5 1010111 }" "asm"="vwsub.vx {vd}, {vs2}, {rs1}, v0.t" "asmonly" {
  ret void
}

define void @VWADDU_WX.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="1101001 vs2:5 rs1:5 110 vd:5 1010111" "asm"="vwaddu.wx {vd}, {vs2}, {rs1}" "asmonly" {
  ret void
}

define void @VWADDU_WX.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if(vd!=0) 1101000 vs2:5 rs1:5 110 vd:5 1010111 }" "asm"="vwaddu.wx {vd}, {vs2}, {rs1}, v0.t" "asmonly" {
  ret void
}

define void @VWADD_WX.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="1101011 vs2:5 rs1:5 110 vd:5 1010111" "asm"="vwadd.wx {vd}, {vs2}, {rs1}" "asmonly" {
  ret void
}

define void @VWADD_WX.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if(vd!=0) 1101010 vs2:5 rs1:5 110 vd:5 1010111 }" "asm"="vwadd.wx {vd}, {vs2}, {rs1}, v0.t" "asmonly" {
  ret void
}

define void @VWSUBU_WX.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="1101101 vs2:5 rs1:5 110 vd:5 1010111" "asm"="vwsubu.wx {vd}, {vs2}, {rs1}" "asmonly" {
  ret void
}

define void @VWSUBU_WX.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if(vd!=0) 1101100 vs2:5 rs1:5 110 vd:5 1010111 }" "asm"="vwsubu.wx {vd}, {vs2}, {rs1}, v0.t" "asmonly" {
  ret void
}

define void @VWSUB_WX.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="1101111 vs2:5 rs1:5 110 vd:5 1010111" "asm"="vwsub.wx {vd}, {vs2}, {rs1}" "asmonly" {
  ret void
}

define void @VWSUB_WX.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if(vd!=0) 1101110 vs2:5 rs1:5 110 vd:5 1010111 }" "asm"="vwsub.wx {vd}, {vs2}, {rs1}, v0.t" "asmonly" {
  ret void
}

define void @VWMULU_VX.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if((vd<vs2)|(vd>=vs2+2)) 1110001 vs2:5 rs1:5 110 vd:5 1010111 }" "asm"="vwmulu.vx {vd}, {vs2}, {rs1}" "asmonly" {
  ret void
}

define void @VWMULU_VX.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if((vd<vs2)|(vd>=vs2+2)) if(vd!=0) 1110000 vs2:5 rs1:5 110 vd:5 1010111 }" "asm"="vwmulu.vx {vd}, {vs2}, {rs1}, v0.t" "asmonly" {
  ret void
}

define void @VWMULSU_VX.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if((vd<vs2)|(vd>=vs2+2)) 1110101 vs2:5 rs1:5 110 vd:5 1010111 }" "asm"="vwmulsu.vx {vd}, {vs2}, {rs1}" "asmonly" {
  ret void
}

define void @VWMULSU_VX.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if((vd<vs2)|(vd>=vs2+2)) if(vd!=0) 1110100 vs2:5 rs1:5 110 vd:5 1010111 }" "asm"="vwmulsu.vx {vd}, {vs2}, {rs1}, v0.t" "asmonly" {
  ret void
}

define void @VWMUL_VX.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if((vd<vs2)|(vd>=vs2+2)) 1110111 vs2:5 rs1:5 110 vd:5 1010111 }" "asm"="vwmul.vx {vd}, {vs2}, {rs1}" "asmonly" {
  ret void
}

define void @VWMUL_VX.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if((vd<vs2)|(vd>=vs2+2)) if(vd!=0) 1110110 vs2:5 rs1:5 110 vd:5 1010111 }" "asm"="vwmul.vx {vd}, {vs2}, {rs1}, v0.t" "asmonly" {
  ret void
}

define void @VWMACCU_VX.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if((vd<vs2)|(vd>=vs2+2)) 1111001 vs2:5 rs1:5 110 vd:5 1010111 }" "asm"="vwmaccu.vx {vd}, {rs1}, {vs2}" "asmonly" {
  ret void
}

define void @VWMACCU_VX.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if((vd<vs2)|(vd>=vs2+2)) if(vd!=0) 1111000 vs2:5 rs1:5 110 vd:5 1010111 }" "asm"="vwmaccu.vx {vd}, {rs1}, {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VWMACC_VX.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if((vd<vs2)|(vd>=vs2+2)) 1111011 vs2:5 rs1:5 110 vd:5 1010111 }" "asm"="vwmacc.vx {vd}, {rs1}, {vs2}" "asmonly" {
  ret void
}

define void @VWMACC_VX.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if((vd<vs2)|(vd>=vs2+2)) if(vd!=0) 1111010 vs2:5 rs1:5 110 vd:5 1010111 }" "asm"="vwmacc.vx {vd}, {rs1}, {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VWMACCUS_VX.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if((vd<vs2)|(vd>=vs2+2)) 1111101 vs2:5 rs1:5 110 vd:5 1010111 }" "asm"="vwmaccus.vx {vd}, {rs1}, {vs2}" "asmonly" {
  ret void
}

define void @VWMACCUS_VX.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if((vd<vs2)|(vd>=vs2+2)) if(vd!=0) 1111100 vs2:5 rs1:5 110 vd:5 1010111 }" "asm"="vwmaccus.vx {vd}, {rs1}, {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VWMACCSU_VX.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if((vd<vs2)|(vd>=vs2+2)) 1111111 vs2:5 rs1:5 110 vd:5 1010111 }" "asm"="vwmaccsu.vx {vd}, {rs1}, {vs2}" "asmonly" {
  ret void
}

define void @VWMACCSU_VX.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if((vd<vs2)|(vd>=vs2+2)) if(vd!=0) 1111110 vs2:5 rs1:5 110 vd:5 1010111 }" "asm"="vwmaccsu.vx {vd}, {rs1}, {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VANDN_VV.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="0000011 vs2:5 vs1:5 000 vd:5 1010111" "asm"="vandn.vv {vd}, {vs2}, {vs1}" "asmonly" {
  ret void
}

define void @VANDN_VV.1(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="{ if(vd!=0) 0000010 vs2:5 vs1:5 000 vd:5 1010111 }" "asm"="vandn.vv {vd}, {vs2}, {vs1}, v0.t" "asmonly" {
  ret void
}

define void @VANDN_VX.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="0000011 vs2:5 rs1:5 100 vd:5 1010111" "asm"="vandn.vx {vd}, {vs2}, {rs1}" "asmonly" {
  ret void
}

define void @VANDN_VX.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if(vd!=0) 0000010 vs2:5 rs1:5 100 vd:5 1010111 }" "asm"="vandn.vx {vd}, {vs2}, {rs1}, v0.t" "asmonly" {
  ret void
}

define void @VBREV_V.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="0100101 vs2:5 01010010 vd:5 1010111" "asm"="vbrev.v {vd}, {vs2}" "asmonly" {
  ret void
}

define void @VBREV_V.1(i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if(vd!=0) 0100100 vs2:5 01010010 vd:5 1010111 }" "asm"="vbrev.v {vd}, {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VBREV8_V.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="0100101 vs2:5 01000010 vd:5 1010111" "asm"="vbrev8.v {vd}, {vs2}" "asmonly" {
  ret void
}

define void @VBREV8_V.1(i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if(vd!=0) 0100100 vs2:5 01000010 vd:5 1010111 }" "asm"="vbrev8.v {vd}, {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VREV8_V.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="0100101 vs2:5 01001010 vd:5 1010111" "asm"="vrev8.v {vd}, {vs2}" "asmonly" {
  ret void
}

define void @VREV8_V.1(i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if(vd!=0) 0100100 vs2:5 01001010 vd:5 1010111 }" "asm"="vrev8.v {vd}, {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VCLZ_V.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="0100101 vs2:5 01100010 vd:5 1010111" "asm"="vclz.v {vd}, {vs2}" "asmonly" {
  ret void
}

define void @VCLZ_V.1(i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if(vd!=0) 0100100 vs2:5 01100010 vd:5 1010111 }" "asm"="vclz.v {vd}, {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VCTZ_V.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="0100101 vs2:5 01101010 vd:5 1010111" "asm"="vctz.v {vd}, {vs2}" "asmonly" {
  ret void
}

define void @VCTZ_V.1(i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if(vd!=0) 0100100 vs2:5 01101010 vd:5 1010111 }" "asm"="vctz.v {vd}, {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VCPOP_V.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="0100101 vs2:5 01110010 vd:5 1010111" "asm"="vcpop.v {vd}, {vs2}" "asmonly" {
  ret void
}

define void @VCPOP_V.1(i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if(vd!=0) 0100100 vs2:5 01110010 vd:5 1010111 }" "asm"="vcpop.v {vd}, {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VROL_VV.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="0101011 vs2:5 vs1:5 000 vd:5 1010111" "asm"="vrol.vv {vd}, {vs2}, {vs1}" "asmonly" {
  ret void
}

define void @VROL_VV.1(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="{ if(vd!=0) 0101010 vs2:5 vs1:5 000 vd:5 1010111 }" "asm"="vrol.vv {vd}, {vs2}, {vs1}, v0.t" "asmonly" {
  ret void
}

define void @VROL_VX.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="0101011 vs2:5 rs1:5 100 vd:5 1010111" "asm"="vrol.vx {vd}, {vs2}, {rs1}" "asmonly" {
  ret void
}

define void @VROL_VX.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if(vd!=0) 0101010 vs2:5 rs1:5 100 vd:5 1010111 }" "asm"="vrol.vx {vd}, {vs2}, {rs1}, v0.t" "asmonly" {
  ret void
}

define void @VROR_VV.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="0101001 vs2:5 vs1:5 000 vd:5 1010111" "asm"="vror.vv {vd}, {vs2}, {vs1}" "asmonly" {
  ret void
}

define void @VROR_VV.1(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="{ if(vd!=0) 0101000 vs2:5 vs1:5 000 vd:5 1010111 }" "asm"="vror.vv {vd}, {vs2}, {vs1}, v0.t" "asmonly" {
  ret void
}

define void @VROR_VX.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="0101001 vs2:5 rs1:5 100 vd:5 1010111" "asm"="vror.vx {vd}, {vs2}, {rs1}" "asmonly" {
  ret void
}

define void @VROR_VX.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if(vd!=0) 0101000 vs2:5 rs1:5 100 vd:5 1010111 }" "asm"="vror.vx {vd}, {vs2}, {rs1}, v0.t" "asmonly" {
  ret void
}

define void @VROR_VI.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs2, i64 "imm" %zimm6) "enc"="01010 i_hi:1=(zimm6>>5)&1 1 vs2:5 i_lo:5=zimm6&31 011 vd:5 1010111" "asm"="vror.vi {vd}, {vs2}, {zimm6}" "asmonly" {
  ret void
}

define void @VROR_VI.1(i64 "reg"="v" %vd, i64 "reg"="v" %vs2, i64 "imm" %zimm6) "enc"="{ if(vd!=0) 01010 i_hi:1=(zimm6>>5)&1 0 vs2:5 i_lo:5=zimm6&31 011 vd:5 1010111 }" "asm"="vror.vi {vd}, {vs2}, {zimm6}, v0.t" "asmonly" {
  ret void
}

define void @VWSLL_VV.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="{ if((vd<vs1)|(vd>=vs1+2)) if((vd<vs2)|(vd>=vs2+2)) 1101011 vs2:5 vs1:5 000 vd:5 1010111 }" "asm"="vwsll.vv {vd}, {vs2}, {vs1}" "asmonly" {
  ret void
}

define void @VWSLL_VV.1(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="{ if((vd<vs1)|(vd>=vs1+2)) if((vd<vs2)|(vd>=vs2+2)) if(vd!=0) 1101010 vs2:5 vs1:5 000 vd:5 1010111 }" "asm"="vwsll.vv {vd}, {vs2}, {vs1}, v0.t" "asmonly" {
  ret void
}

define void @VWSLL_VX.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if((vd<vs2)|(vd>=vs2+2)) 1101011 vs2:5 rs1:5 100 vd:5 1010111 }" "asm"="vwsll.vx {vd}, {vs2}, {rs1}" "asmonly" {
  ret void
}

define void @VWSLL_VX.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if((vd<vs2)|(vd>=vs2+2)) if(vd!=0) 1101010 vs2:5 rs1:5 100 vd:5 1010111 }" "asm"="vwsll.vx {vd}, {vs2}, {rs1}, v0.t" "asmonly" {
  ret void
}

define void @VWSLL_VI.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs2, i64 "imm" %zimm5) "enc"="{ if((vd<vs2)|(vd>=vs2+2)) 1101011 vs2:5 zimm5:5 011 vd:5 1010111 }" "asm"="vwsll.vi {vd}, {vs2}, {zimm5}" "asmonly" {
  ret void
}

define void @VWSLL_VI.1(i64 "reg"="v" %vd, i64 "reg"="v" %vs2, i64 "imm" %zimm5) "enc"="{ if((vd<vs2)|(vd>=vs2+2)) if(vd!=0) 1101010 vs2:5 zimm5:5 011 vd:5 1010111 }" "asm"="vwsll.vi {vd}, {vs2}, {zimm5}, v0.t" "asmonly" {
  ret void
}

define void @VCLMUL_VV.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="0011001 vs2:5 vs1:5 010 vd:5 1010111" "asm"="vclmul.vv {vd}, {vs2}, {vs1}" "asmonly" {
  ret void
}

define void @VCLMUL_VV.1(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="{ if(vd!=0) 0011000 vs2:5 vs1:5 010 vd:5 1010111 }" "asm"="vclmul.vv {vd}, {vs2}, {vs1}, v0.t" "asmonly" {
  ret void
}

define void @VCLMUL_VX.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="0011001 vs2:5 rs1:5 110 vd:5 1010111" "asm"="vclmul.vx {vd}, {vs2}, {rs1}" "asmonly" {
  ret void
}

define void @VCLMUL_VX.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if(vd!=0) 0011000 vs2:5 rs1:5 110 vd:5 1010111 }" "asm"="vclmul.vx {vd}, {vs2}, {rs1}, v0.t" "asmonly" {
  ret void
}

define void @VCLMULH_VV.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="0011011 vs2:5 vs1:5 010 vd:5 1010111" "asm"="vclmulh.vv {vd}, {vs2}, {vs1}" "asmonly" {
  ret void
}

define void @VCLMULH_VV.1(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="{ if(vd!=0) 0011010 vs2:5 vs1:5 010 vd:5 1010111 }" "asm"="vclmulh.vv {vd}, {vs2}, {vs1}, v0.t" "asmonly" {
  ret void
}

define void @VCLMULH_VX.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="0011011 vs2:5 rs1:5 110 vd:5 1010111" "asm"="vclmulh.vx {vd}, {vs2}, {rs1}" "asmonly" {
  ret void
}

define void @VCLMULH_VX.1(i64 "reg"="x" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if(vd!=0) 0011010 vs2:5 rs1:5 110 vd:5 1010111 }" "asm"="vclmulh.vx {vd}, {vs2}, {rs1}, v0.t" "asmonly" {
  ret void
}

define void @VGMUL_VV.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="1010001 vs2:5 10001010 vd:5 1110111" "asm"="vgmul.vv {vd}, {vs2}" "asmonly" {
  ret void
}

define void @VGHSH_VV.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="1011001 vs2:5 vs1:5 010 vd:5 1110111" "asm"="vghsh.vv {vd}, {vs2}, {vs1}" "asmonly" {
  ret void
}

define void @VAESDF_VV.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="1010001 vs2:5 00001010 vd:5 1110111" "asm"="vaesdf.vv {vd}, {vs2}" "asmonly" {
  ret void
}

define void @VAESDF_VS.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if((vd<vs2)|(vd>=vs2+2)) 1010011 vs2:5 00001010 vd:5 1110111 }" "asm"="vaesdf.vs {vd}, {vs2}" "asmonly" {
  ret void
}

define void @VAESDM_VV.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="1010001 vs2:5 00000010 vd:5 1110111" "asm"="vaesdm.vv {vd}, {vs2}" "asmonly" {
  ret void
}

define void @VAESDM_VS.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if((vd<vs2)|(vd>=vs2+2)) 1010011 vs2:5 00000010 vd:5 1110111 }" "asm"="vaesdm.vs {vd}, {vs2}" "asmonly" {
  ret void
}

define void @VAESEF_VV.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="1010001 vs2:5 00011010 vd:5 1110111" "asm"="vaesef.vv {vd}, {vs2}" "asmonly" {
  ret void
}

define void @VAESEF_VS.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if((vd<vs2)|(vd>=vs2+2)) 1010011 vs2:5 00011010 vd:5 1110111 }" "asm"="vaesef.vs {vd}, {vs2}" "asmonly" {
  ret void
}

define void @VAESEM_VV.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="1010001 vs2:5 00010010 vd:5 1110111" "asm"="vaesem.vv {vd}, {vs2}" "asmonly" {
  ret void
}

define void @VAESEM_VS.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if((vd<vs2)|(vd>=vs2+2)) 1010011 vs2:5 00010010 vd:5 1110111 }" "asm"="vaesem.vs {vd}, {vs2}" "asmonly" {
  ret void
}

define void @VAESZ_VS.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if((vd<vs2)|(vd>=vs2+2)) 1010011 vs2:5 00111010 vd:5 1110111 }" "asm"="vaesz.vs {vd}, {vs2}" "asmonly" {
  ret void
}

define void @VAESKF1_VI.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs2, i64 "imm" %zimm5) "enc"="1000101 vs2:5 zimm5:5 010 vd:5 1110111" "asm"="vaeskf1.vi {vd}, {vs2}, {zimm5}" "asmonly" {
  ret void
}

define void @VAESKF2_VI.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs2, i64 "imm" %zimm5) "enc"="1010101 vs2:5 zimm5:5 010 vd:5 1110111" "asm"="vaeskf2.vi {vd}, {vs2}, {zimm5}" "asmonly" {
  ret void
}

define void @VSHA2MS_VV.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="{ if((vd<vs1)|(vd>=vs1+2)) if((vd<vs2)|(vd>=vs2+2)) 1011011 vs2:5 vs1:5 010 vd:5 1110111 }" "asm"="vsha2ms.vv {vd}, {vs2}, {vs1}" "asmonly" {
  ret void
}

define void @VSHA2CH_VV.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="{ if((vd<vs1)|(vd>=vs1+2)) if((vd<vs2)|(vd>=vs2+2)) 1011101 vs2:5 vs1:5 010 vd:5 1110111 }" "asm"="vsha2ch.vv {vd}, {vs2}, {vs1}" "asmonly" {
  ret void
}

define void @VSHA2CL_VV.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="{ if((vd<vs1)|(vd>=vs1+2)) if((vd<vs2)|(vd>=vs2+2)) 1011111 vs2:5 vs1:5 010 vd:5 1110111 }" "asm"="vsha2cl.vv {vd}, {vs2}, {vs1}" "asmonly" {
  ret void
}

define void @VSM4K_VI.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs2, i64 "imm" %zimm5) "enc"="1000011 vs2:5 zimm5:5 010 vd:5 1110111" "asm"="vsm4k.vi {vd}, {vs2}, {zimm5}" "asmonly" {
  ret void
}

define void @VSM4R_VV.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="1010001 vs2:5 10000010 vd:5 1110111" "asm"="vsm4r.vv {vd}, {vs2}" "asmonly" {
  ret void
}

define void @VSM4R_VS.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if((vd<vs2)|(vd>=vs2+2)) 1010011 vs2:5 10000010 vd:5 1110111 }" "asm"="vsm4r.vs {vd}, {vs2}" "asmonly" {
  ret void
}

define void @VSM3C_VI.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs2, i64 "imm" %zimm5) "enc"="{ if((vd<vs2)|(vd>=vs2+2)) 1010111 vs2:5 zimm5:5 010 vd:5 1110111 }" "asm"="vsm3c.vi {vd}, {vs2}, {zimm5}" "asmonly" {
  ret void
}

define void @VSM3ME_VV.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="{ if((vd<vs2)|(vd>=vs2+2)) 1000001 vs2:5 vs1:5 010 vd:5 1110111 }" "asm"="vsm3me.vv {vd}, {vs2}, {vs1}" "asmonly" {
  ret void
}

define void @VFNCVTBF16_F_F_W.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="0100101 vs2:5 11101001 vd:5 1010111" "asm"="vfncvtbf16.f.f.w {vd}, {vs2}" "asmonly" {
  ret void
}

define void @VFNCVTBF16_F_F_W.1(i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if(vd!=0) 0100100 vs2:5 11101001 vd:5 1010111 }" "asm"="vfncvtbf16.f.f.w {vd}, {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VFWCVTBF16_F_F_V.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if((vd<vs2)|(vd>=vs2+2)) 0100101 vs2:5 01101001 vd:5 1010111 }" "asm"="vfwcvtbf16.f.f.v {vd}, {vs2}" "asmonly" {
  ret void
}

define void @VFWCVTBF16_F_F_V.1(i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if((vd<vs2)|(vd>=vs2+2)) if(vd!=0) 0100100 vs2:5 01101001 vd:5 1010111 }" "asm"="vfwcvtbf16.f.f.v {vd}, {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VFWMACCBF16_VV.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="{ if((vd<vs1)|(vd>=vs1+2)) if((vd<vs2)|(vd>=vs2+2)) 1110111 vs2:5 vs1:5 001 vd:5 1010111 }" "asm"="vfwmaccbf16.vv {vd}, {vs1}, {vs2}" "asmonly" {
  ret void
}

define void @VFWMACCBF16_VV.1(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="{ if((vd<vs1)|(vd>=vs1+2)) if((vd<vs2)|(vd>=vs2+2)) if(vd!=0) 1110110 vs2:5 vs1:5 001 vd:5 1010111 }" "asm"="vfwmaccbf16.vv {vd}, {vs1}, {vs2}, v0.t" "asmonly" {
  ret void
}

define void @VFWMACCBF16_VF.0(i64 "reg"="d" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if((vd<vs2)|(vd>=vs2+2)) 1110111 vs2:5 rs1:5 101 vd:5 1010111 }" "asm"="vfwmaccbf16.vf {vd}, {rs1}, {vs2}" "asmonly" {
  ret void
}

define void @VFWMACCBF16_VF.1(i64 "reg"="d" %rs1, i64 "reg"="v" %vd, i64 "reg"="v" %vs2) "enc"="{ if((vd<vs2)|(vd>=vs2+2)) if(vd!=0) 1110110 vs2:5 rs1:5 101 vd:5 1010111 }" "asm"="vfwmaccbf16.vf {vd}, {rs1}, {vs2}, v0.t" "asmonly" {
  ret void
}

define void @FENCE_TSO.0() "enc"="10000011001100000000000000001111" "asm"="fence.tso" "asmonly" "nodecode" {
  ret void
}

define void @PAUSE.0() "enc"="00000001000000000000000000001111" "asm"="pause" "asmonly" "nodecode" {
  ret void
}

define void @SCALL.0() "enc"="00000000000000000000000001110011" "asm"="scall" "asmonly" "nodecode" {
  ret void
}

define void @SBREAK.0() "enc"="00000000000100000000000001110011" "asm"="sbreak" "asmonly" "nodecode" {
  ret void
}

define void @NOP.0() "enc"="00000000000000000000000000010011" "asm"="nop" "asmonly" "nodecode" {
  ret void
}

define void @ZEXT_B.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1) "enc"="000011111111 rs1:5 111 rd:5 0010011" "asm"="zext.b {rd}, {rs1}" "asmonly" "nodecode" {
  ret void
}

define void @BGEZ.0(i64 "reg"="x" %rs1, ptr "label" %dest) "enc"="o_12:s1=dest>>12 o_10:6=(dest>>5)&63 00000 rs1:5 101 o_4:4=(dest>>1)&15 o_11:1=(dest>>11)&1 1100011" "asm"="bgez {rs1}, {dest}" "asmonly" "nodecode" {
  ret void
}

define void @BLEZ.0(i64 "reg"="x" %rs2, ptr "label" %dest) "enc"="o_12:s1=dest>>12 o_10:6=(dest>>5)&63 rs2:5 00000101 o_4:4=(dest>>1)&15 o_11:1=(dest>>11)&1 1100011" "asm"="blez {rs2}, {dest}" "asmonly" "nodecode" {
  ret void
}

define void @BGTZ.0(i64 "reg"="x" %rs2, ptr "label" %dest) "enc"="o_12:s1=dest>>12 o_10:6=(dest>>5)&63 rs2:5 00000100 o_4:4=(dest>>1)&15 o_11:1=(dest>>11)&1 1100011" "asm"="bgtz {rs2}, {dest}" "asmonly" "nodecode" {
  ret void
}

define void @BLTZ.0(i64 "reg"="x" %rs1, ptr "label" %dest) "enc"="o_12:s1=dest>>12 o_10:6=(dest>>5)&63 00000 rs1:5 100 o_4:4=(dest>>1)&15 o_11:1=(dest>>11)&1 1100011" "asm"="bltz {rs1}, {dest}" "asmonly" "nodecode" {
  ret void
}

define void @BNEZ.0(i64 "reg"="x" %rs1, ptr "label" %dest) "enc"="o_12:s1=dest>>12 o_10:6=(dest>>5)&63 00000 rs1:5 001 o_4:4=(dest>>1)&15 o_11:1=(dest>>11)&1 1100011" "asm"="bnez {rs1}, {dest}" "asmonly" "nodecode" {
  ret void
}

define void @BEQZ.0(i64 "reg"="x" %rs1, ptr "label" %dest) "enc"="o_12:s1=dest>>12 o_10:6=(dest>>5)&63 00000 rs1:5 000 o_4:4=(dest>>1)&15 o_11:1=(dest>>11)&1 1100011" "asm"="beqz {rs1}, {dest}" "asmonly" "nodecode" {
  ret void
}

define void @SLTZ.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1) "enc"="000000000000 rs1:5 010 rd:5 0110011" "asm"="sltz {rd}, {rs1}" "asmonly" "nodecode" {
  ret void
}

define void @SGTZ.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs2) "enc"="0000000 rs2:5 00000010 rd:5 0110011" "asm"="sgtz {rd}, {rs2}" "asmonly" "nodecode" {
  ret void
}

define void @JR.0(i64 "reg"="x" %rs1) "enc"="000000000000 rs1:5 000000001100111" "asm"="jr {rs1}" "asmonly" "nodecode" {
  ret void
}

define void @FMV_X_S.0(i64 "reg"="x" %rd, i64 "reg"="d" %rs1) "enc"="111000000000 rs1:5 000 rd:5 1010011" "asm"="fmv.x.s {rd}, {rs1}" "asmonly" "nodecode" {
  ret void
}

define void @FMV_S_X.0(i64 "reg"="d" %rd, i64 "reg"="x" %rs1) "enc"="111100000000 rs1:5 000 rd:5 1010011" "asm"="fmv.s.x {rd}, {rs1}" "asmonly" "nodecode" {
  ret void
}

define void @FRFLAGS.0(i64 "reg"="x" %rd) "enc"="00000000000100000010 rd:5 1110011" "asm"="frflags {rd}" "asmonly" "nodecode" {
  ret void
}

define void @FSFLAGS.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1) "enc"="000000000001 rs1:5 001 rd:5 1110011" "asm"="fsflags {rd}, {rs1}" "asmonly" "nodecode" {
  ret void
}

define void @FSFLAGSI.0(i64 "reg"="x" %rd, i64 "imm" %zimm5) "enc"="000000000001 zimm5:5 101 rd:5 1110011" "asm"="fsflagsi {rd}, {zimm5}" "asmonly" "nodecode" {
  ret void
}

define void @FRRM.0(i64 "reg"="x" %rd) "enc"="00000000001000000010 rd:5 1110011" "asm"="frrm {rd}" "asmonly" "nodecode" {
  ret void
}

define void @FSRM.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1) "enc"="000000000010 rs1:5 001 rd:5 1110011" "asm"="fsrm {rd}, {rs1}" "asmonly" "nodecode" {
  ret void
}

define void @FSRMI.0(i64 "reg"="x" %rd, i64 "imm" %zimm5) "enc"="000000000010 zimm5:5 101 rd:5 1110011" "asm"="fsrmi {rd}, {zimm5}" "asmonly" "nodecode" {
  ret void
}

define void @FSCSR.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1) "enc"="000000000011 rs1:5 001 rd:5 1110011" "asm"="fscsr {rd}, {rs1}" "asmonly" "nodecode" {
  ret void
}

define void @FRCSR.0(i64 "reg"="x" %rd) "enc"="00000000001100000010 rd:5 1110011" "asm"="frcsr {rd}" "asmonly" "nodecode" {
  ret void
}

define void @CSRR.0(i64 "reg"="x" %rd, i64 "imm" %csr) "enc"="csr:12 00000010 rd:5 1110011" "asm"="csrr {rd}, {csr}" "asmonly" "nodecode" {
  ret void
}

define void @ZEXT_W.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1) "enc"="000010000000 rs1:5 000 rd:5 0111011" "asm"="zext.w {rd}, {rs1}" "asmonly" "nodecode" {
  ret void
}

define void @NTL_P1.0() "enc"="00000000001000000000000000110011" "asm"="ntl.p1" "asmonly" "nodecode" {
  ret void
}

define void @NTL_PALL.0() "enc"="00000000001100000000000000110011" "asm"="ntl.pall" "asmonly" "nodecode" {
  ret void
}

define void @NTL_S1.0() "enc"="00000000010000000000000000110011" "asm"="ntl.s1" "asmonly" "nodecode" {
  ret void
}

define void @NTL_ALL.0() "enc"="00000000010100000000000000110011" "asm"="ntl.all" "asmonly" "nodecode" {
  ret void
}

define void @MOP_R_0.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1) "enc"="100000011100 rs1:5 100 rd:5 1110011" "asm"="mop.r.0 {rd}, {rs1}" "asmonly" {
  ret void
}

define void @MOP_R_1.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1) "enc"="100000011101 rs1:5 100 rd:5 1110011" "asm"="mop.r.1 {rd}, {rs1}" "asmonly" {
  ret void
}

define void @MOP_R_2.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1) "enc"="100000011110 rs1:5 100 rd:5 1110011" "asm"="mop.r.2 {rd}, {rs1}" "asmonly" {
  ret void
}

define void @MOP_R_3.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1) "enc"="100000011111 rs1:5 100 rd:5 1110011" "asm"="mop.r.3 {rd}, {rs1}" "asmonly" {
  ret void
}

define void @MOP_R_4.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1) "enc"="100001011100 rs1:5 100 rd:5 1110011" "asm"="mop.r.4 {rd}, {rs1}" "asmonly" {
  ret void
}

define void @MOP_R_5.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1) "enc"="100001011101 rs1:5 100 rd:5 1110011" "asm"="mop.r.5 {rd}, {rs1}" "asmonly" {
  ret void
}

define void @MOP_R_6.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1) "enc"="100001011110 rs1:5 100 rd:5 1110011" "asm"="mop.r.6 {rd}, {rs1}" "asmonly" {
  ret void
}

define void @MOP_R_7.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1) "enc"="100001011111 rs1:5 100 rd:5 1110011" "asm"="mop.r.7 {rd}, {rs1}" "asmonly" {
  ret void
}

define void @MOP_R_8.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1) "enc"="100010011100 rs1:5 100 rd:5 1110011" "asm"="mop.r.8 {rd}, {rs1}" "asmonly" {
  ret void
}

define void @MOP_R_9.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1) "enc"="100010011101 rs1:5 100 rd:5 1110011" "asm"="mop.r.9 {rd}, {rs1}" "asmonly" {
  ret void
}

define void @MOP_R_10.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1) "enc"="100010011110 rs1:5 100 rd:5 1110011" "asm"="mop.r.10 {rd}, {rs1}" "asmonly" {
  ret void
}

define void @MOP_R_11.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1) "enc"="100010011111 rs1:5 100 rd:5 1110011" "asm"="mop.r.11 {rd}, {rs1}" "asmonly" {
  ret void
}

define void @MOP_R_12.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1) "enc"="100011011100 rs1:5 100 rd:5 1110011" "asm"="mop.r.12 {rd}, {rs1}" "asmonly" {
  ret void
}

define void @MOP_R_13.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1) "enc"="100011011101 rs1:5 100 rd:5 1110011" "asm"="mop.r.13 {rd}, {rs1}" "asmonly" {
  ret void
}

define void @MOP_R_14.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1) "enc"="100011011110 rs1:5 100 rd:5 1110011" "asm"="mop.r.14 {rd}, {rs1}" "asmonly" {
  ret void
}

define void @MOP_R_15.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1) "enc"="100011011111 rs1:5 100 rd:5 1110011" "asm"="mop.r.15 {rd}, {rs1}" "asmonly" {
  ret void
}

define void @MOP_R_16.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1) "enc"="110000011100 rs1:5 100 rd:5 1110011" "asm"="mop.r.16 {rd}, {rs1}" "asmonly" {
  ret void
}

define void @MOP_R_17.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1) "enc"="110000011101 rs1:5 100 rd:5 1110011" "asm"="mop.r.17 {rd}, {rs1}" "asmonly" {
  ret void
}

define void @MOP_R_18.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1) "enc"="110000011110 rs1:5 100 rd:5 1110011" "asm"="mop.r.18 {rd}, {rs1}" "asmonly" {
  ret void
}

define void @MOP_R_19.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1) "enc"="110000011111 rs1:5 100 rd:5 1110011" "asm"="mop.r.19 {rd}, {rs1}" "asmonly" {
  ret void
}

define void @MOP_R_20.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1) "enc"="110001011100 rs1:5 100 rd:5 1110011" "asm"="mop.r.20 {rd}, {rs1}" "asmonly" {
  ret void
}

define void @MOP_R_21.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1) "enc"="110001011101 rs1:5 100 rd:5 1110011" "asm"="mop.r.21 {rd}, {rs1}" "asmonly" {
  ret void
}

define void @MOP_R_22.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1) "enc"="110001011110 rs1:5 100 rd:5 1110011" "asm"="mop.r.22 {rd}, {rs1}" "asmonly" {
  ret void
}

define void @MOP_R_23.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1) "enc"="110001011111 rs1:5 100 rd:5 1110011" "asm"="mop.r.23 {rd}, {rs1}" "asmonly" {
  ret void
}

define void @MOP_R_24.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1) "enc"="110010011100 rs1:5 100 rd:5 1110011" "asm"="mop.r.24 {rd}, {rs1}" "asmonly" {
  ret void
}

define void @MOP_R_25.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1) "enc"="110010011101 rs1:5 100 rd:5 1110011" "asm"="mop.r.25 {rd}, {rs1}" "asmonly" {
  ret void
}

define void @MOP_R_26.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1) "enc"="110010011110 rs1:5 100 rd:5 1110011" "asm"="mop.r.26 {rd}, {rs1}" "asmonly" {
  ret void
}

define void @MOP_R_27.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1) "enc"="110010011111 rs1:5 100 rd:5 1110011" "asm"="mop.r.27 {rd}, {rs1}" "asmonly" {
  ret void
}

define void @MOP_R_28.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1) "enc"="110011011100 rs1:5 100 rd:5 1110011" "asm"="mop.r.28 {rd}, {rs1}" "asmonly" {
  ret void
}

define void @MOP_R_29.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1) "enc"="110011011101 rs1:5 100 rd:5 1110011" "asm"="mop.r.29 {rd}, {rs1}" "asmonly" {
  ret void
}

define void @MOP_R_30.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1) "enc"="110011011110 rs1:5 100 rd:5 1110011" "asm"="mop.r.30 {rd}, {rs1}" "asmonly" {
  ret void
}

define void @MOP_R_31.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1) "enc"="110011011111 rs1:5 100 rd:5 1110011" "asm"="mop.r.31 {rd}, {rs1}" "asmonly" {
  ret void
}

define void @MOP_RR_0.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1, i64 "reg"="x" %rs2) "enc"="1000001 rs2:5 rs1:5 100 rd:5 1110011" "asm"="mop.rr.0 {rd}, {rs1}, {rs2}" "asmonly" {
  ret void
}

define void @MOP_RR_1.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1, i64 "reg"="x" %rs2) "enc"="1000011 rs2:5 rs1:5 100 rd:5 1110011" "asm"="mop.rr.1 {rd}, {rs1}, {rs2}" "asmonly" {
  ret void
}

define void @MOP_RR_2.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1, i64 "reg"="x" %rs2) "enc"="1000101 rs2:5 rs1:5 100 rd:5 1110011" "asm"="mop.rr.2 {rd}, {rs1}, {rs2}" "asmonly" {
  ret void
}

define void @MOP_RR_3.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1, i64 "reg"="x" %rs2) "enc"="1000111 rs2:5 rs1:5 100 rd:5 1110011" "asm"="mop.rr.3 {rd}, {rs1}, {rs2}" "asmonly" {
  ret void
}

define void @MOP_RR_4.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1, i64 "reg"="x" %rs2) "enc"="1100001 rs2:5 rs1:5 100 rd:5 1110011" "asm"="mop.rr.4 {rd}, {rs1}, {rs2}" "asmonly" {
  ret void
}

define void @MOP_RR_5.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1, i64 "reg"="x" %rs2) "enc"="1100011 rs2:5 rs1:5 100 rd:5 1110011" "asm"="mop.rr.5 {rd}, {rs1}, {rs2}" "asmonly" {
  ret void
}

define void @MOP_RR_6.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1, i64 "reg"="x" %rs2) "enc"="1100101 rs2:5 rs1:5 100 rd:5 1110011" "asm"="mop.rr.6 {rd}, {rs1}, {rs2}" "asmonly" {
  ret void
}

define void @MOP_RR_7.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1, i64 "reg"="x" %rs2) "enc"="1100111 rs2:5 rs1:5 100 rd:5 1110011" "asm"="mop.rr.7 {rd}, {rs1}, {rs2}" "asmonly" {
  ret void
}

define void @RDCYCLE.0(i64 "reg"="x" %rd) "enc"="11000000000000000010 rd:5 1110011" "asm"="rdcycle {rd}" "asmonly" "nodecode" {
  ret void
}

define void @RDTIME.0(i64 "reg"="x" %rd) "enc"="11000000000100000010 rd:5 1110011" "asm"="rdtime {rd}" "asmonly" "nodecode" {
  ret void
}

define void @RDINSTRET.0(i64 "reg"="x" %rd) "enc"="11000000001000000010 rd:5 1110011" "asm"="rdinstret {rd}" "asmonly" "nodecode" {
  ret void
}

define void @LPAD.0(i64 "imm" %imm20) "enc"="imm20:20 000000010111" "asm"="lpad {imm20}" "asmonly" "nodecode" {
  ret void
}

define void @SSPOPCHK.0() "enc"="11001101110000001100000001110011" "asm"="sspopchk ra" "asmonly" {
  ret void
}

define void @SSPOPCHK.1() "enc"="11001101110000101100000001110011" "asm"="sspopchk t0" "asmonly" {
  ret void
}

define void @SSPUSH.0() "enc"="11001110000100000100000001110011" "asm"="sspush ra" "asmonly" {
  ret void
}

define void @SSPUSH.1() "enc"="11001110010100000100000001110011" "asm"="sspush t0" "asmonly" {
  ret void
}

define void @VL1R_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd) "enc"="000000101000 rs1:5 000 vd:5 0000111" "asm"="vl1r.v {vd}, ({rs1})" "asmonly" "nodecode" {
  ret void
}

define void @VL2R_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd) "enc"="{ if(vd%2==0) 001000101000 rs1:5 000 vd:5 0000111 }" "asm"="vl2r.v {vd}, ({rs1})" "asmonly" "nodecode" {
  ret void
}

define void @VL4R_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd) "enc"="{ if(vd%4==0) 011000101000 rs1:5 000 vd:5 0000111 }" "asm"="vl4r.v {vd}, ({rs1})" "asmonly" "nodecode" {
  ret void
}

define void @VL8R_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd) "enc"="{ if(vd%8==0) 111000101000 rs1:5 000 vd:5 0000111 }" "asm"="vl8r.v {vd}, ({rs1})" "asmonly" "nodecode" {
  ret void
}

define void @VLE1_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vd) "enc"="000000101011 rs1:5 000 vd:5 0000111" "asm"="vle1.v {vd}, ({rs1})" "asmonly" "nodecode" {
  ret void
}

define void @VSE1_V.0(i64 "reg"="x" %rs1, i64 "reg"="v" %vs3) "enc"="000000101011 rs1:5 000 vs3:5 0100111" "asm"="vse1.v {vs3}, ({rs1})" "asmonly" "nodecode" {
  ret void
}

define void @VFREDSUM_VS.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="0000011 vs2:5 vs1:5 001 vd:5 1010111" "asm"="vfredsum.vs {vd}, {vs2}, {vs1}" "asmonly" "nodecode" {
  ret void
}

define void @VFREDSUM_VS.1(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="0000010 vs2:5 vs1:5 001 vd:5 1010111" "asm"="vfredsum.vs {vd}, {vs2}, {vs1}, v0.t" "asmonly" "nodecode" {
  ret void
}

define void @VFWREDSUM_VS.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="1100011 vs2:5 vs1:5 001 vd:5 1010111" "asm"="vfwredsum.vs {vd}, {vs2}, {vs1}" "asmonly" "nodecode" {
  ret void
}

define void @VFWREDSUM_VS.1(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="1100010 vs2:5 vs1:5 001 vd:5 1010111" "asm"="vfwredsum.vs {vd}, {vs2}, {vs1}, v0.t" "asmonly" "nodecode" {
  ret void
}

define void @VPOPC_M.0(i64 "reg"="x" %rd, i64 "reg"="v" %vs2) "enc"="0100001 vs2:5 10000010 rd:5 1010111" "asm"="vpopc.m {rd}, {vs2}" "asmonly" "nodecode" {
  ret void
}

define void @VPOPC_M.1(i64 "reg"="x" %rd, i64 "reg"="v" %vs2) "enc"="0100000 vs2:5 10000010 rd:5 1010111" "asm"="vpopc.m {rd}, {vs2}, v0.t" "asmonly" "nodecode" {
  ret void
}

define void @VMORNOT_MM.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="0111001 vs2:5 vs1:5 010 vd:5 1010111" "asm"="vmornot.mm {vd}, {vs2}, {vs1}" "asmonly" "nodecode" {
  ret void
}

define void @VMANDNOT_MM.0(i64 "reg"="v" %vd, i64 "reg"="v" %vs1, i64 "reg"="v" %vs2) "enc"="0110001 vs2:5 vs1:5 010 vd:5 1010111" "asm"="vmandnot.mm {vd}, {vs2}, {vs1}" "asmonly" "nodecode" {
  ret void
}

define void @C_ADDI4SPN.0(i64 "reg"="x" %rd, i64 "imm" %imm) "enc"="{ if(imm!=0) 000 i0:2=(imm>>4)&3 i1:4=(imm>>6)&15 i2:1=(imm>>2)&1 i3:1=(imm>>3)&1 rd_p:3=rd-8 00 }" "asm"="c.addi4spn {rd}, sp, {imm}" "asmonly" {
  ret void
}

define void @C_LW.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1, i64 "imm" %imm) "enc"="010 i2:3=(imm>>3)&7 rs1_p:3=rs1-8 i0:1=(imm>>2)&1 i1:1=(imm>>6)&1 rd_p:3=rd-8 00" "asm"="c.lw {rd}, {imm}({rs1})" "asmonly" {
  ret void
}

define void @C_SW.0(i64 "reg"="x" %rs1, i64 "reg"="x" %rs2, i64 "imm" %imm) "enc"="110 i2:3=(imm>>3)&7 rs1_p:3=rs1-8 i0:1=(imm>>2)&1 i1:1=(imm>>6)&1 rs2_p:3=rs2-8 00" "asm"="c.sw {rs2}, {imm}({rs1})" "asmonly" {
  ret void
}

define void @C_NOP.0() "enc"="0000000000000001" "asm"="c.nop" "asmonly" {
  ret void
}

define void @C_NOP.1(i64 "imm" %imm) "enc"="{ if(imm!=0) 000 i0:s1=imm>>5 00000 i1:5=imm&31 01 }" "asm"="c.nop {imm}" "asmonly" {
  ret void
}

define void @C_ADDI.0(i64 "reg"="x" %rd, i64 "imm" %imm) "enc"="000 i1:s1=imm>>5 rd:5 i0:5=imm&31 01" "asm"="c.addi {rd}, {imm}" "asmonly" {
  ret void
}

define void @C_LI.0(i64 "reg"="x" %rd, i64 "imm" %imm) "enc"="010 i1:s1=imm>>5 rd:5 i0:5=imm&31 01" "asm"="c.li {rd}, {imm}" "asmonly" {
  ret void
}

define void @C_ADDI16SP.0(i64 "imm" %imm) "enc"="{ if(imm!=0) 011 i0:s1=imm>>9 00010 i1:1=(imm>>4)&1 i2:1=(imm>>6)&1 i3:2=(imm>>7)&3 i4:1=(imm>>5)&1 01 }" "asm"="c.addi16sp sp, {imm}" "asmonly" {
  ret void
}

define void @C_LUI.0(i64 "reg"="x" %rd, i64 "imm" %imm) "enc"="{ if(rd!=2) if(imm!=0) 011 i0:s1=imm>>17 rd:5 i1:5=(imm>>12)&31 01 }" "asm"="c.lui {rd}, {=(imm>>12)&1048575}" "asmonly" {
  ret void
}

define void @C_ANDI.0(i64 "reg"="x" %rd, i64 "imm" %imm) "enc"="100 i0:s1=imm>>5 10 rd_p:3=rd-8 i1:5=imm&31 01" "asm"="c.andi {rd}, {imm}" "asmonly" {
  ret void
}

define void @C_SUB.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs2) "enc"="100011 rd_p:3=rd-8 00 rs2_p:3=rs2-8 01" "asm"="c.sub {rd}, {rs2}" "asmonly" {
  ret void
}

define void @C_XOR.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs2) "enc"="100011 rd_p:3=rd-8 01 rs2_p:3=rs2-8 01" "asm"="c.xor {rd}, {rs2}" "asmonly" {
  ret void
}

define void @C_OR.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs2) "enc"="100011 rd_p:3=rd-8 10 rs2_p:3=rs2-8 01" "asm"="c.or {rd}, {rs2}" "asmonly" {
  ret void
}

define void @C_AND.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs2) "enc"="100011 rd_p:3=rd-8 11 rs2_p:3=rs2-8 01" "asm"="c.and {rd}, {rs2}" "asmonly" {
  ret void
}

define void @C_J.0(ptr "label" %dest) "enc"="101 i0:s1=dest>>11 i1:1=(dest>>4)&1 i2:2=(dest>>8)&3 i3:1=(dest>>10)&1 i4:1=(dest>>6)&1 i5:1=(dest>>7)&1 i6:3=(dest>>1)&7 i7:1=(dest>>5)&1 01" "asm"="c.j {dest}" "asmonly" {
  ret void
}

define void @C_BEQZ.0(i64 "reg"="x" %rs1, ptr "label" %dest) "enc"="110 i3:s1=dest>>8 i4:2=(dest>>3)&3 rs1_p:3=rs1-8 i0:2=(dest>>6)&3 i1:2=(dest>>1)&3 i2:1=(dest>>5)&1 01" "asm"="c.beqz {rs1}, {dest}" "asmonly" {
  ret void
}

define void @C_BNEZ.0(i64 "reg"="x" %rs1, ptr "label" %dest) "enc"="111 i3:s1=dest>>8 i4:2=(dest>>3)&3 rs1_p:3=rs1-8 i0:2=(dest>>6)&3 i1:2=(dest>>1)&3 i2:1=(dest>>5)&1 01" "asm"="c.bnez {rs1}, {dest}" "asmonly" {
  ret void
}

define void @C_LWSP.0(i64 "reg"="x" %rd, i64 "imm" %imm) "enc"="{ if(rd!=0) 010 i0:1=(imm>>5)&1 rd:5 i1:3=(imm>>2)&7 i2:2=(imm>>6)&3 10 }" "asm"="c.lwsp {rd}, {imm}(sp)" "asmonly" {
  ret void
}

define void @C_JR.0(i64 "reg"="x" %rs1) "enc"="{ if(rs1!=0) 1000 rs1:5 0000010 }" "asm"="c.jr {rs1}" "asmonly" {
  ret void
}

define void @C_MV.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs2) "enc"="{ if(rs2!=0) 1000 rd:5 rs2:5 10 }" "asm"="c.mv {rd}, {rs2}" "asmonly" {
  ret void
}

define void @C_EBREAK.0() "enc"="1001000000000010" "asm"="c.ebreak" "asmonly" {
  ret void
}

define void @C_JALR.0(i64 "reg"="x" %rs1) "enc"="{ if(rs1!=0) 1001 rs1:5 0000010 }" "asm"="c.jalr {rs1}" "asmonly" {
  ret void
}

define void @C_ADD.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs2) "enc"="{ if(rs2!=0) 1001 rd:5 rs2:5 10 }" "asm"="c.add {rd}, {rs2}" "asmonly" {
  ret void
}

define void @C_SWSP.0(i64 "reg"="x" %rs2, i64 "imm" %imm) "enc"="110 i0:4=(imm>>2)&15 i1:2=(imm>>6)&3 rs2:5 10" "asm"="c.swsp {rs2}, {imm}(sp)" "asmonly" {
  ret void
}

define void @C_LD.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1, i64 "imm" %imm) "enc"="011 i1:3=(imm>>3)&7 rs1_p:3=rs1-8 i0:2=(imm>>6)&3 rd_p:3=rd-8 00" "asm"="c.ld {rd}, {imm}({rs1})" "asmonly" {
  ret void
}

define void @C_SD.0(i64 "reg"="x" %rs1, i64 "reg"="x" %rs2, i64 "imm" %imm) "enc"="111 i0:3=(imm>>3)&7 rs1_p:3=rs1-8 i1:2=(imm>>6)&3 rs2_p:3=rs2-8 00" "asm"="c.sd {rs2}, {imm}({rs1})" "asmonly" {
  ret void
}

define void @C_ADDIW.0(i64 "reg"="x" %rd, i64 "imm" %imm) "enc"="{ if(rd!=0) 001 i1:s1=imm>>5 rd:5 i0:5=imm&31 01 }" "asm"="c.addiw {rd}, {imm}" "asmonly" {
  ret void
}

define void @C_SRLI.0(i64 "reg"="x" %rd, i64 "imm" %imm) "enc"="100 i1:1=(imm>>5)&1 00 rd_p:3=rd-8 i0:5=imm&31 01" "asm"="c.srli {rd}, {imm}" "asmonly" {
  ret void
}

define void @C_SRAI.0(i64 "reg"="x" %rd, i64 "imm" %imm) "enc"="100 i1:1=(imm>>5)&1 01 rd_p:3=rd-8 i0:5=imm&31 01" "asm"="c.srai {rd}, {imm}" "asmonly" {
  ret void
}

define void @C_SUBW.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs2) "enc"="100111 rd_p:3=rd-8 00 rs2_p:3=rs2-8 01" "asm"="c.subw {rd}, {rs2}" "asmonly" {
  ret void
}

define void @C_ADDW.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs2) "enc"="100111 rd_p:3=rd-8 01 rs2_p:3=rs2-8 01" "asm"="c.addw {rd}, {rs2}" "asmonly" {
  ret void
}

define void @C_SLLI.0(i64 "reg"="x" %rd, i64 "imm" %imm) "enc"="000 i0:1=(imm>>5)&1 rd:5 i1:5=imm&31 10" "asm"="c.slli {rd}, {imm}" "asmonly" {
  ret void
}

define void @C_LDSP.0(i64 "reg"="x" %rd, i64 "imm" %imm) "enc"="{ if(rd!=0) 011 i0:1=(imm>>5)&1 rd:5 i1:2=(imm>>3)&3 i2:3=(imm>>6)&7 10 }" "asm"="c.ldsp {rd}, {imm}(sp)" "asmonly" {
  ret void
}

define void @C_SDSP.0(i64 "reg"="x" %rs2, i64 "imm" %imm) "enc"="111 i0:3=(imm>>3)&7 i1:3=(imm>>6)&7 rs2:5 10" "asm"="c.sdsp {rs2}, {imm}(sp)" "asmonly" {
  ret void
}

define void @C_FLD.0(i64 "reg"="d" %rd, i64 "reg"="x" %rs1, i64 "imm" %imm) "enc"="001 i1:3=(imm>>3)&7 rs1_p:3=rs1-8 i0:2=(imm>>6)&3 rd_p:3=rd-8 00" "asm"="c.fld {rd}, {imm}({rs1})" "asmonly" {
  ret void
}

define void @C_FSD.0(i64 "reg"="x" %rs1, i64 "reg"="d" %rs2, i64 "imm" %imm) "enc"="101 i1:3=(imm>>3)&7 rs1_p:3=rs1-8 i0:2=(imm>>6)&3 rs2_p:3=rs2-8 00" "asm"="c.fsd {rs2}, {imm}({rs1})" "asmonly" {
  ret void
}

define void @C_FLDSP.0(i64 "reg"="d" %rd, i64 "imm" %imm) "enc"="001 i0:1=(imm>>5)&1 rd:5 i1:2=(imm>>3)&3 i2:3=(imm>>6)&7 10" "asm"="c.fldsp {rd}, {imm}(sp)" "asmonly" {
  ret void
}

define void @C_FSDSP.0(i64 "reg"="d" %rs2, i64 "imm" %imm) "enc"="101 i0:3=(imm>>3)&7 i1:3=(imm>>6)&7 rs2:5 10" "asm"="c.fsdsp {rs2}, {imm}(sp)" "asmonly" {
  ret void
}

define void @C_LBU.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1, i64 "imm" %imm) "enc"="100000 rs1_p:3=rs1-8 i0:1=imm&1 i1:1=(imm>>1)&1 rd_p:3=rd-8 00" "asm"="c.lbu {rd}, {imm}({rs1})" "asmonly" {
  ret void
}

define void @C_LHU.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1, i64 "imm" %imm) "enc"="100001 rs1_p:3=rs1-8 0 i0:1=(imm>>1)&1 rd_p:3=rd-8 00" "asm"="c.lhu {rd}, {imm}({rs1})" "asmonly" {
  ret void
}

define void @C_LH.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs1, i64 "imm" %imm) "enc"="100001 rs1_p:3=rs1-8 1 i0:1=(imm>>1)&1 rd_p:3=rd-8 00" "asm"="c.lh {rd}, {imm}({rs1})" "asmonly" {
  ret void
}

define void @C_SB.0(i64 "reg"="x" %rs2, i64 "reg"="x" %rs1, i64 "imm" %imm) "enc"="100010 rs1_p:3=rs1-8 i0:1=imm&1 i1:1=(imm>>1)&1 rs2_p:3=rs2-8 00" "asm"="c.sb {rs2}, {imm}({rs1})" "asmonly" {
  ret void
}

define void @C_SH.0(i64 "reg"="x" %rs2, i64 "reg"="x" %rs1, i64 "imm" %imm) "enc"="100011 rs1_p:3=rs1-8 0 i0:1=(imm>>1)&1 rs2_p:3=rs2-8 00" "asm"="c.sh {rs2}, {imm}({rs1})" "asmonly" {
  ret void
}

define void @C_ZEXT_B.0(i64 "reg"="x" %rd) "enc"="100111 rd_p:3=rd-8 1100001" "asm"="c.zext.b {rd}" "asmonly" {
  ret void
}

define void @C_SEXT_B.0(i64 "reg"="x" %rd) "enc"="100111 rd_p:3=rd-8 1100101" "asm"="c.sext.b {rd}" "asmonly" {
  ret void
}

define void @C_ZEXT_H.0(i64 "reg"="x" %rd) "enc"="100111 rd_p:3=rd-8 1101001" "asm"="c.zext.h {rd}" "asmonly" {
  ret void
}

define void @C_SEXT_H.0(i64 "reg"="x" %rd) "enc"="100111 rd_p:3=rd-8 1101101" "asm"="c.sext.h {rd}" "asmonly" {
  ret void
}

define void @C_NOT.0(i64 "reg"="x" %rd) "enc"="100111 rd_p:3=rd-8 1110101" "asm"="c.not {rd}" "asmonly" {
  ret void
}

define void @C_MUL.0(i64 "reg"="x" %rd, i64 "reg"="x" %rs2) "enc"="100111 rd_p:3=rd-8 10 rs2_p:3=rs2-8 01" "asm"="c.mul {rd}, {rs2}" "asmonly" {
  ret void
}

define void @C_ZEXT_W.0(i64 "reg"="x" %rd) "enc"="100111 rd_p:3=rd-8 1110001" "asm"="c.zext.w {rd}" "asmonly" {
  ret void
}

define void @C_MOP_1.0() "enc"="0110000010000001" "asm"="c.mop.1" "asmonly" {
  ret void
}

define void @C_MOP_3.0() "enc"="0110000110000001" "asm"="c.mop.3" "asmonly" {
  ret void
}

define void @C_MOP_5.0() "enc"="0110001010000001" "asm"="c.mop.5" "asmonly" {
  ret void
}

define void @C_MOP_7.0() "enc"="0110001110000001" "asm"="c.mop.7" "asmonly" {
  ret void
}

define void @C_MOP_9.0() "enc"="0110010010000001" "asm"="c.mop.9" "asmonly" {
  ret void
}

define void @C_MOP_11.0() "enc"="0110010110000001" "asm"="c.mop.11" "asmonly" {
  ret void
}

define void @C_MOP_13.0() "enc"="0110011010000001" "asm"="c.mop.13" "asmonly" {
  ret void
}

define void @C_MOP_15.0() "enc"="0110011110000001" "asm"="c.mop.15" "asmonly" {
  ret void
}

define void @C_SSPUSH.0() "enc"="0110000010000001" "asm"="c.sspush ra" "asmonly" "nodecode" {
  ret void
}

define void @C_SSPOPCHK.0() "enc"="0110001010000001" "asm"="c.sspopchk t0" "asmonly" "nodecode" {
  ret void
}

define void @C_NTL_P1.0() "enc"="1001000000001010" "asm"="c.ntl.p1" "asmonly" "nodecode" {
  ret void
}

define void @C_NTL_PALL.0() "enc"="1001000000001110" "asm"="c.ntl.pall" "asmonly" "nodecode" {
  ret void
}

define void @C_NTL_S1.0() "enc"="1001000000010010" "asm"="c.ntl.s1" "asmonly" "nodecode" {
  ret void
}

define void @C_NTL_ALL.0() "enc"="1001000000010110" "asm"="c.ntl.all" "asmonly" "nodecode" {
  ret void
}

define void @CM_PUSH.0() "enc"="1011100001000010" "asm"="cm.push {lb}ra{rb}, -16" "asmonly" "nodecode" {
  ret void
}

define void @CM_PUSH.1() "enc"="1011100001000110" "asm"="cm.push {lb}ra{rb}, -32" "asmonly" "nodecode" {
  ret void
}

define void @CM_PUSH.2() "enc"="1011100001001010" "asm"="cm.push {lb}ra{rb}, -48" "asmonly" "nodecode" {
  ret void
}

define void @CM_PUSH.3() "enc"="1011100001001110" "asm"="cm.push {lb}ra{rb}, -64" "asmonly" "nodecode" {
  ret void
}

define void @CM_PUSH.4() "enc"="1011100001010010" "asm"="cm.push {lb}ra, s0{rb}, -16" "asmonly" "nodecode" {
  ret void
}

define void @CM_PUSH.5() "enc"="1011100001010110" "asm"="cm.push {lb}ra, s0{rb}, -32" "asmonly" "nodecode" {
  ret void
}

define void @CM_PUSH.6() "enc"="1011100001011010" "asm"="cm.push {lb}ra, s0{rb}, -48" "asmonly" "nodecode" {
  ret void
}

define void @CM_PUSH.7() "enc"="1011100001011110" "asm"="cm.push {lb}ra, s0{rb}, -64" "asmonly" "nodecode" {
  ret void
}

define void @CM_PUSH.8() "enc"="1011100001100010" "asm"="cm.push {lb}ra, s0-s1{rb}, -32" "asmonly" "nodecode" {
  ret void
}

define void @CM_PUSH.9() "enc"="1011100001100110" "asm"="cm.push {lb}ra, s0-s1{rb}, -48" "asmonly" "nodecode" {
  ret void
}

define void @CM_PUSH.10() "enc"="1011100001101010" "asm"="cm.push {lb}ra, s0-s1{rb}, -64" "asmonly" "nodecode" {
  ret void
}

define void @CM_PUSH.11() "enc"="1011100001101110" "asm"="cm.push {lb}ra, s0-s1{rb}, -80" "asmonly" "nodecode" {
  ret void
}

define void @CM_PUSH.12() "enc"="1011100001110010" "asm"="cm.push {lb}ra, s0-s2{rb}, -32" "asmonly" "nodecode" {
  ret void
}

define void @CM_PUSH.13() "enc"="1011100001110110" "asm"="cm.push {lb}ra, s0-s2{rb}, -48" "asmonly" "nodecode" {
  ret void
}

define void @CM_PUSH.14() "enc"="1011100001111010" "asm"="cm.push {lb}ra, s0-s2{rb}, -64" "asmonly" "nodecode" {
  ret void
}

define void @CM_PUSH.15() "enc"="1011100001111110" "asm"="cm.push {lb}ra, s0-s2{rb}, -80" "asmonly" "nodecode" {
  ret void
}

define void @CM_PUSH.16() "enc"="1011100010000010" "asm"="cm.push {lb}ra, s0-s3{rb}, -48" "asmonly" "nodecode" {
  ret void
}

define void @CM_PUSH.17() "enc"="1011100010000110" "asm"="cm.push {lb}ra, s0-s3{rb}, -64" "asmonly" "nodecode" {
  ret void
}

define void @CM_PUSH.18() "enc"="1011100010001010" "asm"="cm.push {lb}ra, s0-s3{rb}, -80" "asmonly" "nodecode" {
  ret void
}

define void @CM_PUSH.19() "enc"="1011100010001110" "asm"="cm.push {lb}ra, s0-s3{rb}, -96" "asmonly" "nodecode" {
  ret void
}

define void @CM_PUSH.20() "enc"="1011100010010010" "asm"="cm.push {lb}ra, s0-s4{rb}, -48" "asmonly" "nodecode" {
  ret void
}

define void @CM_PUSH.21() "enc"="1011100010010110" "asm"="cm.push {lb}ra, s0-s4{rb}, -64" "asmonly" "nodecode" {
  ret void
}

define void @CM_PUSH.22() "enc"="1011100010011010" "asm"="cm.push {lb}ra, s0-s4{rb}, -80" "asmonly" "nodecode" {
  ret void
}

define void @CM_PUSH.23() "enc"="1011100010011110" "asm"="cm.push {lb}ra, s0-s4{rb}, -96" "asmonly" "nodecode" {
  ret void
}

define void @CM_PUSH.24() "enc"="1011100010100010" "asm"="cm.push {lb}ra, s0-s5{rb}, -64" "asmonly" "nodecode" {
  ret void
}

define void @CM_PUSH.25() "enc"="1011100010100110" "asm"="cm.push {lb}ra, s0-s5{rb}, -80" "asmonly" "nodecode" {
  ret void
}

define void @CM_PUSH.26() "enc"="1011100010101010" "asm"="cm.push {lb}ra, s0-s5{rb}, -96" "asmonly" "nodecode" {
  ret void
}

define void @CM_PUSH.27() "enc"="1011100010101110" "asm"="cm.push {lb}ra, s0-s5{rb}, -112" "asmonly" "nodecode" {
  ret void
}

define void @CM_PUSH.28() "enc"="1011100010110010" "asm"="cm.push {lb}ra, s0-s6{rb}, -64" "asmonly" "nodecode" {
  ret void
}

define void @CM_PUSH.29() "enc"="1011100010110110" "asm"="cm.push {lb}ra, s0-s6{rb}, -80" "asmonly" "nodecode" {
  ret void
}

define void @CM_PUSH.30() "enc"="1011100010111010" "asm"="cm.push {lb}ra, s0-s6{rb}, -96" "asmonly" "nodecode" {
  ret void
}

define void @CM_PUSH.31() "enc"="1011100010111110" "asm"="cm.push {lb}ra, s0-s6{rb}, -112" "asmonly" "nodecode" {
  ret void
}

define void @CM_PUSH.32() "enc"="1011100011000010" "asm"="cm.push {lb}ra, s0-s7{rb}, -80" "asmonly" "nodecode" {
  ret void
}

define void @CM_PUSH.33() "enc"="1011100011000110" "asm"="cm.push {lb}ra, s0-s7{rb}, -96" "asmonly" "nodecode" {
  ret void
}

define void @CM_PUSH.34() "enc"="1011100011001010" "asm"="cm.push {lb}ra, s0-s7{rb}, -112" "asmonly" "nodecode" {
  ret void
}

define void @CM_PUSH.35() "enc"="1011100011001110" "asm"="cm.push {lb}ra, s0-s7{rb}, -128" "asmonly" "nodecode" {
  ret void
}

define void @CM_PUSH.36() "enc"="1011100011010010" "asm"="cm.push {lb}ra, s0-s8{rb}, -80" "asmonly" "nodecode" {
  ret void
}

define void @CM_PUSH.37() "enc"="1011100011010110" "asm"="cm.push {lb}ra, s0-s8{rb}, -96" "asmonly" "nodecode" {
  ret void
}

define void @CM_PUSH.38() "enc"="1011100011011010" "asm"="cm.push {lb}ra, s0-s8{rb}, -112" "asmonly" "nodecode" {
  ret void
}

define void @CM_PUSH.39() "enc"="1011100011011110" "asm"="cm.push {lb}ra, s0-s8{rb}, -128" "asmonly" "nodecode" {
  ret void
}

define void @CM_PUSH.40() "enc"="1011100011100010" "asm"="cm.push {lb}ra, s0-s9{rb}, -96" "asmonly" "nodecode" {
  ret void
}

define void @CM_PUSH.41() "enc"="1011100011100110" "asm"="cm.push {lb}ra, s0-s9{rb}, -112" "asmonly" "nodecode" {
  ret void
}

define void @CM_PUSH.42() "enc"="1011100011101010" "asm"="cm.push {lb}ra, s0-s9{rb}, -128" "asmonly" "nodecode" {
  ret void
}

define void @CM_PUSH.43() "enc"="1011100011101110" "asm"="cm.push {lb}ra, s0-s9{rb}, -144" "asmonly" "nodecode" {
  ret void
}

define void @CM_PUSH.44() "enc"="1011100011110010" "asm"="cm.push {lb}ra, s0-s11{rb}, -112" "asmonly" "nodecode" {
  ret void
}

define void @CM_PUSH.45() "enc"="1011100011110110" "asm"="cm.push {lb}ra, s0-s11{rb}, -128" "asmonly" "nodecode" {
  ret void
}

define void @CM_PUSH.46() "enc"="1011100011111010" "asm"="cm.push {lb}ra, s0-s11{rb}, -144" "asmonly" "nodecode" {
  ret void
}

define void @CM_PUSH.47() "enc"="1011100011111110" "asm"="cm.push {lb}ra, s0-s11{rb}, -160" "asmonly" "nodecode" {
  ret void
}

define void @CM_POP.0() "enc"="1011101001000010" "asm"="cm.pop {lb}ra{rb}, 16" "asmonly" "nodecode" {
  ret void
}

define void @CM_POP.1() "enc"="1011101001000110" "asm"="cm.pop {lb}ra{rb}, 32" "asmonly" "nodecode" {
  ret void
}

define void @CM_POP.2() "enc"="1011101001001010" "asm"="cm.pop {lb}ra{rb}, 48" "asmonly" "nodecode" {
  ret void
}

define void @CM_POP.3() "enc"="1011101001001110" "asm"="cm.pop {lb}ra{rb}, 64" "asmonly" "nodecode" {
  ret void
}

define void @CM_POP.4() "enc"="1011101001010010" "asm"="cm.pop {lb}ra, s0{rb}, 16" "asmonly" "nodecode" {
  ret void
}

define void @CM_POP.5() "enc"="1011101001010110" "asm"="cm.pop {lb}ra, s0{rb}, 32" "asmonly" "nodecode" {
  ret void
}

define void @CM_POP.6() "enc"="1011101001011010" "asm"="cm.pop {lb}ra, s0{rb}, 48" "asmonly" "nodecode" {
  ret void
}

define void @CM_POP.7() "enc"="1011101001011110" "asm"="cm.pop {lb}ra, s0{rb}, 64" "asmonly" "nodecode" {
  ret void
}

define void @CM_POP.8() "enc"="1011101001100010" "asm"="cm.pop {lb}ra, s0-s1{rb}, 32" "asmonly" "nodecode" {
  ret void
}

define void @CM_POP.9() "enc"="1011101001100110" "asm"="cm.pop {lb}ra, s0-s1{rb}, 48" "asmonly" "nodecode" {
  ret void
}

define void @CM_POP.10() "enc"="1011101001101010" "asm"="cm.pop {lb}ra, s0-s1{rb}, 64" "asmonly" "nodecode" {
  ret void
}

define void @CM_POP.11() "enc"="1011101001101110" "asm"="cm.pop {lb}ra, s0-s1{rb}, 80" "asmonly" "nodecode" {
  ret void
}

define void @CM_POP.12() "enc"="1011101001110010" "asm"="cm.pop {lb}ra, s0-s2{rb}, 32" "asmonly" "nodecode" {
  ret void
}

define void @CM_POP.13() "enc"="1011101001110110" "asm"="cm.pop {lb}ra, s0-s2{rb}, 48" "asmonly" "nodecode" {
  ret void
}

define void @CM_POP.14() "enc"="1011101001111010" "asm"="cm.pop {lb}ra, s0-s2{rb}, 64" "asmonly" "nodecode" {
  ret void
}

define void @CM_POP.15() "enc"="1011101001111110" "asm"="cm.pop {lb}ra, s0-s2{rb}, 80" "asmonly" "nodecode" {
  ret void
}

define void @CM_POP.16() "enc"="1011101010000010" "asm"="cm.pop {lb}ra, s0-s3{rb}, 48" "asmonly" "nodecode" {
  ret void
}

define void @CM_POP.17() "enc"="1011101010000110" "asm"="cm.pop {lb}ra, s0-s3{rb}, 64" "asmonly" "nodecode" {
  ret void
}

define void @CM_POP.18() "enc"="1011101010001010" "asm"="cm.pop {lb}ra, s0-s3{rb}, 80" "asmonly" "nodecode" {
  ret void
}

define void @CM_POP.19() "enc"="1011101010001110" "asm"="cm.pop {lb}ra, s0-s3{rb}, 96" "asmonly" "nodecode" {
  ret void
}

define void @CM_POP.20() "enc"="1011101010010010" "asm"="cm.pop {lb}ra, s0-s4{rb}, 48" "asmonly" "nodecode" {
  ret void
}

define void @CM_POP.21() "enc"="1011101010010110" "asm"="cm.pop {lb}ra, s0-s4{rb}, 64" "asmonly" "nodecode" {
  ret void
}

define void @CM_POP.22() "enc"="1011101010011010" "asm"="cm.pop {lb}ra, s0-s4{rb}, 80" "asmonly" "nodecode" {
  ret void
}

define void @CM_POP.23() "enc"="1011101010011110" "asm"="cm.pop {lb}ra, s0-s4{rb}, 96" "asmonly" "nodecode" {
  ret void
}

define void @CM_POP.24() "enc"="1011101010100010" "asm"="cm.pop {lb}ra, s0-s5{rb}, 64" "asmonly" "nodecode" {
  ret void
}

define void @CM_POP.25() "enc"="1011101010100110" "asm"="cm.pop {lb}ra, s0-s5{rb}, 80" "asmonly" "nodecode" {
  ret void
}

define void @CM_POP.26() "enc"="1011101010101010" "asm"="cm.pop {lb}ra, s0-s5{rb}, 96" "asmonly" "nodecode" {
  ret void
}

define void @CM_POP.27() "enc"="1011101010101110" "asm"="cm.pop {lb}ra, s0-s5{rb}, 112" "asmonly" "nodecode" {
  ret void
}

define void @CM_POP.28() "enc"="1011101010110010" "asm"="cm.pop {lb}ra, s0-s6{rb}, 64" "asmonly" "nodecode" {
  ret void
}

define void @CM_POP.29() "enc"="1011101010110110" "asm"="cm.pop {lb}ra, s0-s6{rb}, 80" "asmonly" "nodecode" {
  ret void
}

define void @CM_POP.30() "enc"="1011101010111010" "asm"="cm.pop {lb}ra, s0-s6{rb}, 96" "asmonly" "nodecode" {
  ret void
}

define void @CM_POP.31() "enc"="1011101010111110" "asm"="cm.pop {lb}ra, s0-s6{rb}, 112" "asmonly" "nodecode" {
  ret void
}

define void @CM_POP.32() "enc"="1011101011000010" "asm"="cm.pop {lb}ra, s0-s7{rb}, 80" "asmonly" "nodecode" {
  ret void
}

define void @CM_POP.33() "enc"="1011101011000110" "asm"="cm.pop {lb}ra, s0-s7{rb}, 96" "asmonly" "nodecode" {
  ret void
}

define void @CM_POP.34() "enc"="1011101011001010" "asm"="cm.pop {lb}ra, s0-s7{rb}, 112" "asmonly" "nodecode" {
  ret void
}

define void @CM_POP.35() "enc"="1011101011001110" "asm"="cm.pop {lb}ra, s0-s7{rb}, 128" "asmonly" "nodecode" {
  ret void
}

define void @CM_POP.36() "enc"="1011101011010010" "asm"="cm.pop {lb}ra, s0-s8{rb}, 80" "asmonly" "nodecode" {
  ret void
}

define void @CM_POP.37() "enc"="1011101011010110" "asm"="cm.pop {lb}ra, s0-s8{rb}, 96" "asmonly" "nodecode" {
  ret void
}

define void @CM_POP.38() "enc"="1011101011011010" "asm"="cm.pop {lb}ra, s0-s8{rb}, 112" "asmonly" "nodecode" {
  ret void
}

define void @CM_POP.39() "enc"="1011101011011110" "asm"="cm.pop {lb}ra, s0-s8{rb}, 128" "asmonly" "nodecode" {
  ret void
}

define void @CM_POP.40() "enc"="1011101011100010" "asm"="cm.pop {lb}ra, s0-s9{rb}, 96" "asmonly" "nodecode" {
  ret void
}

define void @CM_POP.41() "enc"="1011101011100110" "asm"="cm.pop {lb}ra, s0-s9{rb}, 112" "asmonly" "nodecode" {
  ret void
}

define void @CM_POP.42() "enc"="1011101011101010" "asm"="cm.pop {lb}ra, s0-s9{rb}, 128" "asmonly" "nodecode" {
  ret void
}

define void @CM_POP.43() "enc"="1011101011101110" "asm"="cm.pop {lb}ra, s0-s9{rb}, 144" "asmonly" "nodecode" {
  ret void
}

define void @CM_POP.44() "enc"="1011101011110010" "asm"="cm.pop {lb}ra, s0-s11{rb}, 112" "asmonly" "nodecode" {
  ret void
}

define void @CM_POP.45() "enc"="1011101011110110" "asm"="cm.pop {lb}ra, s0-s11{rb}, 128" "asmonly" "nodecode" {
  ret void
}

define void @CM_POP.46() "enc"="1011101011111010" "asm"="cm.pop {lb}ra, s0-s11{rb}, 144" "asmonly" "nodecode" {
  ret void
}

define void @CM_POP.47() "enc"="1011101011111110" "asm"="cm.pop {lb}ra, s0-s11{rb}, 160" "asmonly" "nodecode" {
  ret void
}

define void @CM_POPRETZ.0() "enc"="1011110001000010" "asm"="cm.popretz {lb}ra{rb}, 16" "asmonly" "nodecode" {
  ret void
}

define void @CM_POPRETZ.1() "enc"="1011110001000110" "asm"="cm.popretz {lb}ra{rb}, 32" "asmonly" "nodecode" {
  ret void
}

define void @CM_POPRETZ.2() "enc"="1011110001001010" "asm"="cm.popretz {lb}ra{rb}, 48" "asmonly" "nodecode" {
  ret void
}

define void @CM_POPRETZ.3() "enc"="1011110001001110" "asm"="cm.popretz {lb}ra{rb}, 64" "asmonly" "nodecode" {
  ret void
}

define void @CM_POPRETZ.4() "enc"="1011110001010010" "asm"="cm.popretz {lb}ra, s0{rb}, 16" "asmonly" "nodecode" {
  ret void
}

define void @CM_POPRETZ.5() "enc"="1011110001010110" "asm"="cm.popretz {lb}ra, s0{rb}, 32" "asmonly" "nodecode" {
  ret void
}

define void @CM_POPRETZ.6() "enc"="1011110001011010" "asm"="cm.popretz {lb}ra, s0{rb}, 48" "asmonly" "nodecode" {
  ret void
}

define void @CM_POPRETZ.7() "enc"="1011110001011110" "asm"="cm.popretz {lb}ra, s0{rb}, 64" "asmonly" "nodecode" {
  ret void
}

define void @CM_POPRETZ.8() "enc"="1011110001100010" "asm"="cm.popretz {lb}ra, s0-s1{rb}, 32" "asmonly" "nodecode" {
  ret void
}

define void @CM_POPRETZ.9() "enc"="1011110001100110" "asm"="cm.popretz {lb}ra, s0-s1{rb}, 48" "asmonly" "nodecode" {
  ret void
}

define void @CM_POPRETZ.10() "enc"="1011110001101010" "asm"="cm.popretz {lb}ra, s0-s1{rb}, 64" "asmonly" "nodecode" {
  ret void
}

define void @CM_POPRETZ.11() "enc"="1011110001101110" "asm"="cm.popretz {lb}ra, s0-s1{rb}, 80" "asmonly" "nodecode" {
  ret void
}

define void @CM_POPRETZ.12() "enc"="1011110001110010" "asm"="cm.popretz {lb}ra, s0-s2{rb}, 32" "asmonly" "nodecode" {
  ret void
}

define void @CM_POPRETZ.13() "enc"="1011110001110110" "asm"="cm.popretz {lb}ra, s0-s2{rb}, 48" "asmonly" "nodecode" {
  ret void
}

define void @CM_POPRETZ.14() "enc"="1011110001111010" "asm"="cm.popretz {lb}ra, s0-s2{rb}, 64" "asmonly" "nodecode" {
  ret void
}

define void @CM_POPRETZ.15() "enc"="1011110001111110" "asm"="cm.popretz {lb}ra, s0-s2{rb}, 80" "asmonly" "nodecode" {
  ret void
}

define void @CM_POPRETZ.16() "enc"="1011110010000010" "asm"="cm.popretz {lb}ra, s0-s3{rb}, 48" "asmonly" "nodecode" {
  ret void
}

define void @CM_POPRETZ.17() "enc"="1011110010000110" "asm"="cm.popretz {lb}ra, s0-s3{rb}, 64" "asmonly" "nodecode" {
  ret void
}

define void @CM_POPRETZ.18() "enc"="1011110010001010" "asm"="cm.popretz {lb}ra, s0-s3{rb}, 80" "asmonly" "nodecode" {
  ret void
}

define void @CM_POPRETZ.19() "enc"="1011110010001110" "asm"="cm.popretz {lb}ra, s0-s3{rb}, 96" "asmonly" "nodecode" {
  ret void
}

define void @CM_POPRETZ.20() "enc"="1011110010010010" "asm"="cm.popretz {lb}ra, s0-s4{rb}, 48" "asmonly" "nodecode" {
  ret void
}

define void @CM_POPRETZ.21() "enc"="1011110010010110" "asm"="cm.popretz {lb}ra, s0-s4{rb}, 64" "asmonly" "nodecode" {
  ret void
}

define void @CM_POPRETZ.22() "enc"="1011110010011010" "asm"="cm.popretz {lb}ra, s0-s4{rb}, 80" "asmonly" "nodecode" {
  ret void
}

define void @CM_POPRETZ.23() "enc"="1011110010011110" "asm"="cm.popretz {lb}ra, s0-s4{rb}, 96" "asmonly" "nodecode" {
  ret void
}

define void @CM_POPRETZ.24() "enc"="1011110010100010" "asm"="cm.popretz {lb}ra, s0-s5{rb}, 64" "asmonly" "nodecode" {
  ret void
}

define void @CM_POPRETZ.25() "enc"="1011110010100110" "asm"="cm.popretz {lb}ra, s0-s5{rb}, 80" "asmonly" "nodecode" {
  ret void
}

define void @CM_POPRETZ.26() "enc"="1011110010101010" "asm"="cm.popretz {lb}ra, s0-s5{rb}, 96" "asmonly" "nodecode" {
  ret void
}

define void @CM_POPRETZ.27() "enc"="1011110010101110" "asm"="cm.popretz {lb}ra, s0-s5{rb}, 112" "asmonly" "nodecode" {
  ret void
}

define void @CM_POPRETZ.28() "enc"="1011110010110010" "asm"="cm.popretz {lb}ra, s0-s6{rb}, 64" "asmonly" "nodecode" {
  ret void
}

define void @CM_POPRETZ.29() "enc"="1011110010110110" "asm"="cm.popretz {lb}ra, s0-s6{rb}, 80" "asmonly" "nodecode" {
  ret void
}

define void @CM_POPRETZ.30() "enc"="1011110010111010" "asm"="cm.popretz {lb}ra, s0-s6{rb}, 96" "asmonly" "nodecode" {
  ret void
}

define void @CM_POPRETZ.31() "enc"="1011110010111110" "asm"="cm.popretz {lb}ra, s0-s6{rb}, 112" "asmonly" "nodecode" {
  ret void
}

define void @CM_POPRETZ.32() "enc"="1011110011000010" "asm"="cm.popretz {lb}ra, s0-s7{rb}, 80" "asmonly" "nodecode" {
  ret void
}

define void @CM_POPRETZ.33() "enc"="1011110011000110" "asm"="cm.popretz {lb}ra, s0-s7{rb}, 96" "asmonly" "nodecode" {
  ret void
}

define void @CM_POPRETZ.34() "enc"="1011110011001010" "asm"="cm.popretz {lb}ra, s0-s7{rb}, 112" "asmonly" "nodecode" {
  ret void
}

define void @CM_POPRETZ.35() "enc"="1011110011001110" "asm"="cm.popretz {lb}ra, s0-s7{rb}, 128" "asmonly" "nodecode" {
  ret void
}

define void @CM_POPRETZ.36() "enc"="1011110011010010" "asm"="cm.popretz {lb}ra, s0-s8{rb}, 80" "asmonly" "nodecode" {
  ret void
}

define void @CM_POPRETZ.37() "enc"="1011110011010110" "asm"="cm.popretz {lb}ra, s0-s8{rb}, 96" "asmonly" "nodecode" {
  ret void
}

define void @CM_POPRETZ.38() "enc"="1011110011011010" "asm"="cm.popretz {lb}ra, s0-s8{rb}, 112" "asmonly" "nodecode" {
  ret void
}

define void @CM_POPRETZ.39() "enc"="1011110011011110" "asm"="cm.popretz {lb}ra, s0-s8{rb}, 128" "asmonly" "nodecode" {
  ret void
}

define void @CM_POPRETZ.40() "enc"="1011110011100010" "asm"="cm.popretz {lb}ra, s0-s9{rb}, 96" "asmonly" "nodecode" {
  ret void
}

define void @CM_POPRETZ.41() "enc"="1011110011100110" "asm"="cm.popretz {lb}ra, s0-s9{rb}, 112" "asmonly" "nodecode" {
  ret void
}

define void @CM_POPRETZ.42() "enc"="1011110011101010" "asm"="cm.popretz {lb}ra, s0-s9{rb}, 128" "asmonly" "nodecode" {
  ret void
}

define void @CM_POPRETZ.43() "enc"="1011110011101110" "asm"="cm.popretz {lb}ra, s0-s9{rb}, 144" "asmonly" "nodecode" {
  ret void
}

define void @CM_POPRETZ.44() "enc"="1011110011110010" "asm"="cm.popretz {lb}ra, s0-s11{rb}, 112" "asmonly" "nodecode" {
  ret void
}

define void @CM_POPRETZ.45() "enc"="1011110011110110" "asm"="cm.popretz {lb}ra, s0-s11{rb}, 128" "asmonly" "nodecode" {
  ret void
}

define void @CM_POPRETZ.46() "enc"="1011110011111010" "asm"="cm.popretz {lb}ra, s0-s11{rb}, 144" "asmonly" "nodecode" {
  ret void
}

define void @CM_POPRETZ.47() "enc"="1011110011111110" "asm"="cm.popretz {lb}ra, s0-s11{rb}, 160" "asmonly" "nodecode" {
  ret void
}

define void @CM_POPRET.0() "enc"="1011111001000010" "asm"="cm.popret {lb}ra{rb}, 16" "asmonly" "nodecode" {
  ret void
}

define void @CM_POPRET.1() "enc"="1011111001000110" "asm"="cm.popret {lb}ra{rb}, 32" "asmonly" "nodecode" {
  ret void
}

define void @CM_POPRET.2() "enc"="1011111001001010" "asm"="cm.popret {lb}ra{rb}, 48" "asmonly" "nodecode" {
  ret void
}

define void @CM_POPRET.3() "enc"="1011111001001110" "asm"="cm.popret {lb}ra{rb}, 64" "asmonly" "nodecode" {
  ret void
}

define void @CM_POPRET.4() "enc"="1011111001010010" "asm"="cm.popret {lb}ra, s0{rb}, 16" "asmonly" "nodecode" {
  ret void
}

define void @CM_POPRET.5() "enc"="1011111001010110" "asm"="cm.popret {lb}ra, s0{rb}, 32" "asmonly" "nodecode" {
  ret void
}

define void @CM_POPRET.6() "enc"="1011111001011010" "asm"="cm.popret {lb}ra, s0{rb}, 48" "asmonly" "nodecode" {
  ret void
}

define void @CM_POPRET.7() "enc"="1011111001011110" "asm"="cm.popret {lb}ra, s0{rb}, 64" "asmonly" "nodecode" {
  ret void
}

define void @CM_POPRET.8() "enc"="1011111001100010" "asm"="cm.popret {lb}ra, s0-s1{rb}, 32" "asmonly" "nodecode" {
  ret void
}

define void @CM_POPRET.9() "enc"="1011111001100110" "asm"="cm.popret {lb}ra, s0-s1{rb}, 48" "asmonly" "nodecode" {
  ret void
}

define void @CM_POPRET.10() "enc"="1011111001101010" "asm"="cm.popret {lb}ra, s0-s1{rb}, 64" "asmonly" "nodecode" {
  ret void
}

define void @CM_POPRET.11() "enc"="1011111001101110" "asm"="cm.popret {lb}ra, s0-s1{rb}, 80" "asmonly" "nodecode" {
  ret void
}

define void @CM_POPRET.12() "enc"="1011111001110010" "asm"="cm.popret {lb}ra, s0-s2{rb}, 32" "asmonly" "nodecode" {
  ret void
}

define void @CM_POPRET.13() "enc"="1011111001110110" "asm"="cm.popret {lb}ra, s0-s2{rb}, 48" "asmonly" "nodecode" {
  ret void
}

define void @CM_POPRET.14() "enc"="1011111001111010" "asm"="cm.popret {lb}ra, s0-s2{rb}, 64" "asmonly" "nodecode" {
  ret void
}

define void @CM_POPRET.15() "enc"="1011111001111110" "asm"="cm.popret {lb}ra, s0-s2{rb}, 80" "asmonly" "nodecode" {
  ret void
}

define void @CM_POPRET.16() "enc"="1011111010000010" "asm"="cm.popret {lb}ra, s0-s3{rb}, 48" "asmonly" "nodecode" {
  ret void
}

define void @CM_POPRET.17() "enc"="1011111010000110" "asm"="cm.popret {lb}ra, s0-s3{rb}, 64" "asmonly" "nodecode" {
  ret void
}

define void @CM_POPRET.18() "enc"="1011111010001010" "asm"="cm.popret {lb}ra, s0-s3{rb}, 80" "asmonly" "nodecode" {
  ret void
}

define void @CM_POPRET.19() "enc"="1011111010001110" "asm"="cm.popret {lb}ra, s0-s3{rb}, 96" "asmonly" "nodecode" {
  ret void
}

define void @CM_POPRET.20() "enc"="1011111010010010" "asm"="cm.popret {lb}ra, s0-s4{rb}, 48" "asmonly" "nodecode" {
  ret void
}

define void @CM_POPRET.21() "enc"="1011111010010110" "asm"="cm.popret {lb}ra, s0-s4{rb}, 64" "asmonly" "nodecode" {
  ret void
}

define void @CM_POPRET.22() "enc"="1011111010011010" "asm"="cm.popret {lb}ra, s0-s4{rb}, 80" "asmonly" "nodecode" {
  ret void
}

define void @CM_POPRET.23() "enc"="1011111010011110" "asm"="cm.popret {lb}ra, s0-s4{rb}, 96" "asmonly" "nodecode" {
  ret void
}

define void @CM_POPRET.24() "enc"="1011111010100010" "asm"="cm.popret {lb}ra, s0-s5{rb}, 64" "asmonly" "nodecode" {
  ret void
}

define void @CM_POPRET.25() "enc"="1011111010100110" "asm"="cm.popret {lb}ra, s0-s5{rb}, 80" "asmonly" "nodecode" {
  ret void
}

define void @CM_POPRET.26() "enc"="1011111010101010" "asm"="cm.popret {lb}ra, s0-s5{rb}, 96" "asmonly" "nodecode" {
  ret void
}

define void @CM_POPRET.27() "enc"="1011111010101110" "asm"="cm.popret {lb}ra, s0-s5{rb}, 112" "asmonly" "nodecode" {
  ret void
}

define void @CM_POPRET.28() "enc"="1011111010110010" "asm"="cm.popret {lb}ra, s0-s6{rb}, 64" "asmonly" "nodecode" {
  ret void
}

define void @CM_POPRET.29() "enc"="1011111010110110" "asm"="cm.popret {lb}ra, s0-s6{rb}, 80" "asmonly" "nodecode" {
  ret void
}

define void @CM_POPRET.30() "enc"="1011111010111010" "asm"="cm.popret {lb}ra, s0-s6{rb}, 96" "asmonly" "nodecode" {
  ret void
}

define void @CM_POPRET.31() "enc"="1011111010111110" "asm"="cm.popret {lb}ra, s0-s6{rb}, 112" "asmonly" "nodecode" {
  ret void
}

define void @CM_POPRET.32() "enc"="1011111011000010" "asm"="cm.popret {lb}ra, s0-s7{rb}, 80" "asmonly" "nodecode" {
  ret void
}

define void @CM_POPRET.33() "enc"="1011111011000110" "asm"="cm.popret {lb}ra, s0-s7{rb}, 96" "asmonly" "nodecode" {
  ret void
}

define void @CM_POPRET.34() "enc"="1011111011001010" "asm"="cm.popret {lb}ra, s0-s7{rb}, 112" "asmonly" "nodecode" {
  ret void
}

define void @CM_POPRET.35() "enc"="1011111011001110" "asm"="cm.popret {lb}ra, s0-s7{rb}, 128" "asmonly" "nodecode" {
  ret void
}

define void @CM_POPRET.36() "enc"="1011111011010010" "asm"="cm.popret {lb}ra, s0-s8{rb}, 80" "asmonly" "nodecode" {
  ret void
}

define void @CM_POPRET.37() "enc"="1011111011010110" "asm"="cm.popret {lb}ra, s0-s8{rb}, 96" "asmonly" "nodecode" {
  ret void
}

define void @CM_POPRET.38() "enc"="1011111011011010" "asm"="cm.popret {lb}ra, s0-s8{rb}, 112" "asmonly" "nodecode" {
  ret void
}

define void @CM_POPRET.39() "enc"="1011111011011110" "asm"="cm.popret {lb}ra, s0-s8{rb}, 128" "asmonly" "nodecode" {
  ret void
}

define void @CM_POPRET.40() "enc"="1011111011100010" "asm"="cm.popret {lb}ra, s0-s9{rb}, 96" "asmonly" "nodecode" {
  ret void
}

define void @CM_POPRET.41() "enc"="1011111011100110" "asm"="cm.popret {lb}ra, s0-s9{rb}, 112" "asmonly" "nodecode" {
  ret void
}

define void @CM_POPRET.42() "enc"="1011111011101010" "asm"="cm.popret {lb}ra, s0-s9{rb}, 128" "asmonly" "nodecode" {
  ret void
}

define void @CM_POPRET.43() "enc"="1011111011101110" "asm"="cm.popret {lb}ra, s0-s9{rb}, 144" "asmonly" "nodecode" {
  ret void
}

define void @CM_POPRET.44() "enc"="1011111011110010" "asm"="cm.popret {lb}ra, s0-s11{rb}, 112" "asmonly" "nodecode" {
  ret void
}

define void @CM_POPRET.45() "enc"="1011111011110110" "asm"="cm.popret {lb}ra, s0-s11{rb}, 128" "asmonly" "nodecode" {
  ret void
}

define void @CM_POPRET.46() "enc"="1011111011111010" "asm"="cm.popret {lb}ra, s0-s11{rb}, 144" "asmonly" "nodecode" {
  ret void
}

define void @CM_POPRET.47() "enc"="1011111011111110" "asm"="cm.popret {lb}ra, s0-s11{rb}, 160" "asmonly" "nodecode" {
  ret void
}

define void @CM_MVSA01.0(i64 "reg"="x" %r1, i64 "reg"="x" %r2) "enc"="{ if(r1<10) if(r2<10) 101011 r1_s:3=r1-8 01 r2_s:3=r2-8 10 ; if(r1<10) if(r2>=18) 101011 r1_s:3=r1-8 01 r2_s:3=r2-16 10 ; if(r1>=18) if(r2<10) 101011 r1_s:3=r1-16 01 r2_s:3=r2-8 10 ; if(r1>=18) if(r2>=18) 101011 r1_s:3=r1-16 01 r2_s:3=r2-16 10 }" "asm"="cm.mvsa01 {r1}, {r2}" "asmonly" "nodecode" {
  ret void
}

define void @CM_MVA01S.0(i64 "reg"="x" %r1, i64 "reg"="x" %r2) "enc"="{ if(r1<10) if(r2<10) 101011 r1_s:3=r1-8 11 r2_s:3=r2-8 10 ; if(r1<10) if(r2>=18) 101011 r1_s:3=r1-8 11 r2_s:3=r2-16 10 ; if(r1>=18) if(r2<10) 101011 r1_s:3=r1-16 11 r2_s:3=r2-8 10 ; if(r1>=18) if(r2>=18) 101011 r1_s:3=r1-16 11 r2_s:3=r2-16 10 }" "asm"="cm.mva01s {r1}, {r2}" "asmonly" "nodecode" {
  ret void
}

define void @CM_JALT.0(i64 "imm" %imm) "enc"="{ if(imm>=32) 101000 i0:8=imm&255 10 }" "asm"="cm.jalt {imm}" "asmonly" "nodecode" {
  ret void
}

define void @CM_JT.0(i64 "imm" %imm) "enc"="{ if(imm<32) 101000 i0:8=imm&255 10 }" "asm"="cm.jt {imm}" "asmonly" "nodecode" {
  ret void
}

; The CSRs by name, in the operands of the CSR instructions (csrs.csv).
!target.asm-names = !{!9000, !9001, !9002, !9003, !9004, !9005, !9006, !9007, !9008, !9009, !9010, !9011, !9012, !9013, !9014, !9015, !9016, !9017, !9018, !9019, !9020, !9021, !9022, !9023, !9024, !9025, !9026, !9027, !9028, !9029, !9030, !9031, !9032, !9033, !9034, !9035, !9036, !9037, !9038, !9039, !9040, !9041, !9042, !9043, !9044, !9045, !9046, !9047, !9048, !9049, !9050, !9051, !9052, !9053, !9054, !9055, !9056, !9057, !9058, !9059, !9060, !9061, !9062, !9063, !9064, !9065, !9066, !9067, !9068, !9069, !9070, !9071, !9072, !9073, !9074, !9075, !9076, !9077, !9078, !9079, !9080, !9081, !9082, !9083, !9084, !9085, !9086, !9087, !9088, !9089, !9090, !9091, !9092, !9093, !9094, !9095, !9096, !9097, !9098, !9099, !9100, !9101, !9102, !9103, !9104, !9105, !9106, !9107, !9108, !9109, !9110, !9111, !9112, !9113, !9114, !9115, !9116, !9117, !9118, !9119, !9120, !9121, !9122, !9123, !9124, !9125, !9126, !9127, !9128, !9129, !9130, !9131, !9132, !9133, !9134, !9135, !9136, !9137, !9138, !9139, !9140, !9141, !9142, !9143, !9144, !9145, !9146, !9147, !9148, !9149, !9150, !9151, !9152, !9153, !9154, !9155, !9156, !9157, !9158, !9159, !9160, !9161, !9162, !9163, !9164, !9165, !9166, !9167, !9168, !9169, !9170, !9171, !9172, !9173, !9174, !9175, !9176, !9177, !9178, !9179, !9180, !9181, !9182, !9183, !9184, !9185, !9186, !9187, !9188, !9189, !9190, !9191, !9192, !9193, !9194, !9195, !9196, !9197, !9198, !9199, !9200, !9201, !9202, !9203, !9204, !9205, !9206, !9207, !9208, !9209, !9210, !9211, !9212, !9213, !9214, !9215, !9216, !9217, !9218, !9219, !9220, !9221, !9222, !9223, !9224, !9225, !9226, !9227, !9228, !9229, !9230, !9231, !9232, !9233, !9234, !9235, !9236, !9237, !9238, !9239, !9240, !9241, !9242, !9243, !9244, !9245, !9246, !9247, !9248, !9249, !9250, !9251, !9252, !9253, !9254, !9255, !9256, !9257, !9258, !9259, !9260, !9261, !9262, !9263, !9264, !9265, !9266, !9267, !9268, !9269, !9270, !9271, !9272, !9273, !9274, !9275, !9276, !9277, !9278, !9279, !9280, !9281, !9282, !9283, !9284, !9285, !9286, !9287, !9288, !9289, !9290, !9291, !9292, !9293, !9294, !9295, !9296, !9297, !9298, !9299, !9300, !9301, !9302, !9303, !9304, !9305, !9306, !9307, !9308, !9309, !9310, !9311, !9312, !9313, !9314, !9315, !9316, !9317, !9318, !9319, !9320, !9321, !9322, !9323, !9324, !9325, !9326, !9327, !9328, !9329}
!9000 = !{!"csr", !"fflags", !"0x001"}
!9001 = !{!"csr", !"frm", !"0x002"}
!9002 = !{!"csr", !"fcsr", !"0x003"}
!9003 = !{!"csr", !"vstart", !"0x008"}
!9004 = !{!"csr", !"vxsat", !"0x009"}
!9005 = !{!"csr", !"vxrm", !"0x00A"}
!9006 = !{!"csr", !"vcsr", !"0x00F"}
!9007 = !{!"csr", !"ssp", !"0x011"}
!9008 = !{!"csr", !"seed", !"0x015"}
!9009 = !{!"csr", !"jvt", !"0x017"}
!9010 = !{!"csr", !"cycle", !"0xC00"}
!9011 = !{!"csr", !"time", !"0xC01"}
!9012 = !{!"csr", !"instret", !"0xC02"}
!9013 = !{!"csr", !"hpmcounter3", !"0xC03"}
!9014 = !{!"csr", !"hpmcounter4", !"0xC04"}
!9015 = !{!"csr", !"hpmcounter5", !"0xC05"}
!9016 = !{!"csr", !"hpmcounter6", !"0xC06"}
!9017 = !{!"csr", !"hpmcounter7", !"0xC07"}
!9018 = !{!"csr", !"hpmcounter8", !"0xC08"}
!9019 = !{!"csr", !"hpmcounter9", !"0xC09"}
!9020 = !{!"csr", !"hpmcounter10", !"0xC0A"}
!9021 = !{!"csr", !"hpmcounter11", !"0xC0B"}
!9022 = !{!"csr", !"hpmcounter12", !"0xC0C"}
!9023 = !{!"csr", !"hpmcounter13", !"0xC0D"}
!9024 = !{!"csr", !"hpmcounter14", !"0xC0E"}
!9025 = !{!"csr", !"hpmcounter15", !"0xC0F"}
!9026 = !{!"csr", !"hpmcounter16", !"0xC10"}
!9027 = !{!"csr", !"hpmcounter17", !"0xC11"}
!9028 = !{!"csr", !"hpmcounter18", !"0xC12"}
!9029 = !{!"csr", !"hpmcounter19", !"0xC13"}
!9030 = !{!"csr", !"hpmcounter20", !"0xC14"}
!9031 = !{!"csr", !"hpmcounter21", !"0xC15"}
!9032 = !{!"csr", !"hpmcounter22", !"0xC16"}
!9033 = !{!"csr", !"hpmcounter23", !"0xC17"}
!9034 = !{!"csr", !"hpmcounter24", !"0xC18"}
!9035 = !{!"csr", !"hpmcounter25", !"0xC19"}
!9036 = !{!"csr", !"hpmcounter26", !"0xC1A"}
!9037 = !{!"csr", !"hpmcounter27", !"0xC1B"}
!9038 = !{!"csr", !"hpmcounter28", !"0xC1C"}
!9039 = !{!"csr", !"hpmcounter29", !"0xC1D"}
!9040 = !{!"csr", !"hpmcounter30", !"0xC1E"}
!9041 = !{!"csr", !"hpmcounter31", !"0xC1F"}
!9042 = !{!"csr", !"vl", !"0xC20"}
!9043 = !{!"csr", !"vtype", !"0xC21"}
!9044 = !{!"csr", !"vlenb", !"0xC22"}
!9045 = !{!"csr", !"mtype", !"0xC23"}
!9046 = !{!"csr", !"sstatus", !"0x100"}
!9047 = !{!"csr", !"sie", !"0x104"}
!9048 = !{!"csr", !"stvec", !"0x105"}
!9049 = !{!"csr", !"scounteren", !"0x106"}
!9050 = !{!"csr", !"senvcfg", !"0x10A"}
!9051 = !{!"csr", !"sstateen0", !"0x10C"}
!9052 = !{!"csr", !"sstateen1", !"0x10D"}
!9053 = !{!"csr", !"sstateen2", !"0x10E"}
!9054 = !{!"csr", !"sstateen3", !"0x10F"}
!9055 = !{!"csr", !"scountinhibit", !"0x120"}
!9056 = !{!"csr", !"sscratch", !"0x140"}
!9057 = !{!"csr", !"sepc", !"0x141"}
!9058 = !{!"csr", !"scause", !"0x142"}
!9059 = !{!"csr", !"stval", !"0x143"}
!9060 = !{!"csr", !"sip", !"0x144"}
!9061 = !{!"csr", !"stimecmp", !"0x14D"}
!9062 = !{!"csr", !"sctrctl", !"0x14E"}
!9063 = !{!"csr", !"sctrstatus", !"0x14F"}
!9064 = !{!"csr", !"siselect", !"0x150"}
!9065 = !{!"csr", !"sireg", !"0x151"}
!9066 = !{!"csr", !"sireg2", !"0x152"}
!9067 = !{!"csr", !"sireg3", !"0x153"}
!9068 = !{!"csr", !"sireg4", !"0x155"}
!9069 = !{!"csr", !"sireg5", !"0x156"}
!9070 = !{!"csr", !"sireg6", !"0x157"}
!9071 = !{!"csr", !"stopei", !"0x15C"}
!9072 = !{!"csr", !"sctrdepth", !"0x15F"}
!9073 = !{!"csr", !"satp", !"0x180"}
!9074 = !{!"csr", !"srmcfg", !"0x181"}
!9075 = !{!"csr", !"spmpen", !"0x183"}
!9076 = !{!"csr", !"scontext", !"0x5A8"}
!9077 = !{!"csr", !"vsstatus", !"0x200"}
!9078 = !{!"csr", !"vsie", !"0x204"}
!9079 = !{!"csr", !"vstvec", !"0x205"}
!9080 = !{!"csr", !"vsscratch", !"0x240"}
!9081 = !{!"csr", !"vsepc", !"0x241"}
!9082 = !{!"csr", !"vscause", !"0x242"}
!9083 = !{!"csr", !"vstval", !"0x243"}
!9084 = !{!"csr", !"vsip", !"0x244"}
!9085 = !{!"csr", !"vstimecmp", !"0x24D"}
!9086 = !{!"csr", !"vsctrctl", !"0x24E"}
!9087 = !{!"csr", !"vsiselect", !"0x250"}
!9088 = !{!"csr", !"vsireg", !"0x251"}
!9089 = !{!"csr", !"vsireg2", !"0x252"}
!9090 = !{!"csr", !"vsireg3", !"0x253"}
!9091 = !{!"csr", !"vsireg4", !"0x255"}
!9092 = !{!"csr", !"vsireg5", !"0x256"}
!9093 = !{!"csr", !"vsireg6", !"0x257"}
!9094 = !{!"csr", !"vstopei", !"0x25C"}
!9095 = !{!"csr", !"vsatp", !"0x280"}
!9096 = !{!"csr", !"hstatus", !"0x600"}
!9097 = !{!"csr", !"hedeleg", !"0x602"}
!9098 = !{!"csr", !"hideleg", !"0x603"}
!9099 = !{!"csr", !"hie", !"0x604"}
!9100 = !{!"csr", !"htimedelta", !"0x605"}
!9101 = !{!"csr", !"hcounteren", !"0x606"}
!9102 = !{!"csr", !"hgeie", !"0x607"}
!9103 = !{!"csr", !"hvien", !"0x608"}
!9104 = !{!"csr", !"hvictl", !"0x609"}
!9105 = !{!"csr", !"henvcfg", !"0x60A"}
!9106 = !{!"csr", !"hstateen0", !"0x60C"}
!9107 = !{!"csr", !"hstateen1", !"0x60D"}
!9108 = !{!"csr", !"hstateen2", !"0x60E"}
!9109 = !{!"csr", !"hstateen3", !"0x60F"}
!9110 = !{!"csr", !"htval", !"0x643"}
!9111 = !{!"csr", !"hip", !"0x644"}
!9112 = !{!"csr", !"hvip", !"0x645"}
!9113 = !{!"csr", !"hviprio1", !"0x646"}
!9114 = !{!"csr", !"hviprio2", !"0x647"}
!9115 = !{!"csr", !"htinst", !"0x64A"}
!9116 = !{!"csr", !"hgatp", !"0x680"}
!9117 = !{!"csr", !"hcontext", !"0x6A8"}
!9118 = !{!"csr", !"hgeip", !"0xE12"}
!9119 = !{!"csr", !"vstopi", !"0xEB0"}
!9120 = !{!"csr", !"scountovf", !"0xDA0"}
!9121 = !{!"csr", !"stopi", !"0xDB0"}
!9122 = !{!"csr", !"utvt", !"0x007"}
!9123 = !{!"csr", !"unxti", !"0x045"}
!9124 = !{!"csr", !"uintstatus", !"0x046"}
!9125 = !{!"csr", !"uscratchcsw", !"0x048"}
!9126 = !{!"csr", !"uscratchcswl", !"0x049"}
!9127 = !{!"csr", !"stvt", !"0x107"}
!9128 = !{!"csr", !"snxti", !"0x145"}
!9129 = !{!"csr", !"sintstatus", !"0x146"}
!9130 = !{!"csr", !"sscratchcsw", !"0x148"}
!9131 = !{!"csr", !"sscratchcswl", !"0x149"}
!9132 = !{!"csr", !"mtvt", !"0x307"}
!9133 = !{!"csr", !"mpmpdeleg", !"0x316"}
!9134 = !{!"csr", !"mnxti", !"0x345"}
!9135 = !{!"csr", !"mintstatus", !"0x346"}
!9136 = !{!"csr", !"mscratchcsw", !"0x348"}
!9137 = !{!"csr", !"mscratchcswl", !"0x349"}
!9138 = !{!"csr", !"mstatus", !"0x300"}
!9139 = !{!"csr", !"misa", !"0x301"}
!9140 = !{!"csr", !"medeleg", !"0x302"}
!9141 = !{!"csr", !"mideleg", !"0x303"}
!9142 = !{!"csr", !"mie", !"0x304"}
!9143 = !{!"csr", !"mtvec", !"0x305"}
!9144 = !{!"csr", !"mcounteren", !"0x306"}
!9145 = !{!"csr", !"mvien", !"0x308"}
!9146 = !{!"csr", !"mvip", !"0x309"}
!9147 = !{!"csr", !"menvcfg", !"0x30a"}
!9148 = !{!"csr", !"mstateen0", !"0x30C"}
!9149 = !{!"csr", !"mstateen1", !"0x30D"}
!9150 = !{!"csr", !"mstateen2", !"0x30E"}
!9151 = !{!"csr", !"mstateen3", !"0x30F"}
!9152 = !{!"csr", !"mcountinhibit", !"0x320"}
!9153 = !{!"csr", !"mscratch", !"0x340"}
!9154 = !{!"csr", !"mepc", !"0x341"}
!9155 = !{!"csr", !"mcause", !"0x342"}
!9156 = !{!"csr", !"mtval", !"0x343"}
!9157 = !{!"csr", !"mip", !"0x344"}
!9158 = !{!"csr", !"mtinst", !"0x34a"}
!9159 = !{!"csr", !"mtval2", !"0x34b"}
!9160 = !{!"csr", !"mctrctl", !"0x34E"}
!9161 = !{!"csr", !"miselect", !"0x350"}
!9162 = !{!"csr", !"mireg", !"0x351"}
!9163 = !{!"csr", !"mireg2", !"0x352"}
!9164 = !{!"csr", !"mireg3", !"0x353"}
!9165 = !{!"csr", !"mireg4", !"0x355"}
!9166 = !{!"csr", !"mireg5", !"0x356"}
!9167 = !{!"csr", !"mireg6", !"0x357"}
!9168 = !{!"csr", !"mtopei", !"0x35c"}
!9169 = !{!"csr", !"pmpcfg0", !"0x3a0"}
!9170 = !{!"csr", !"pmpcfg1", !"0x3a1"}
!9171 = !{!"csr", !"pmpcfg2", !"0x3a2"}
!9172 = !{!"csr", !"pmpcfg3", !"0x3a3"}
!9173 = !{!"csr", !"pmpcfg4", !"0x3a4"}
!9174 = !{!"csr", !"pmpcfg5", !"0x3a5"}
!9175 = !{!"csr", !"pmpcfg6", !"0x3a6"}
!9176 = !{!"csr", !"pmpcfg7", !"0x3a7"}
!9177 = !{!"csr", !"pmpcfg8", !"0x3a8"}
!9178 = !{!"csr", !"pmpcfg9", !"0x3a9"}
!9179 = !{!"csr", !"pmpcfg10", !"0x3aa"}
!9180 = !{!"csr", !"pmpcfg11", !"0x3ab"}
!9181 = !{!"csr", !"pmpcfg12", !"0x3ac"}
!9182 = !{!"csr", !"pmpcfg13", !"0x3ad"}
!9183 = !{!"csr", !"pmpcfg14", !"0x3ae"}
!9184 = !{!"csr", !"pmpcfg15", !"0x3af"}
!9185 = !{!"csr", !"pmpaddr0", !"0x3b0"}
!9186 = !{!"csr", !"pmpaddr1", !"0x3b1"}
!9187 = !{!"csr", !"pmpaddr2", !"0x3b2"}
!9188 = !{!"csr", !"pmpaddr3", !"0x3b3"}
!9189 = !{!"csr", !"pmpaddr4", !"0x3b4"}
!9190 = !{!"csr", !"pmpaddr5", !"0x3b5"}
!9191 = !{!"csr", !"pmpaddr6", !"0x3b6"}
!9192 = !{!"csr", !"pmpaddr7", !"0x3b7"}
!9193 = !{!"csr", !"pmpaddr8", !"0x3b8"}
!9194 = !{!"csr", !"pmpaddr9", !"0x3b9"}
!9195 = !{!"csr", !"pmpaddr10", !"0x3ba"}
!9196 = !{!"csr", !"pmpaddr11", !"0x3bb"}
!9197 = !{!"csr", !"pmpaddr12", !"0x3bc"}
!9198 = !{!"csr", !"pmpaddr13", !"0x3bd"}
!9199 = !{!"csr", !"pmpaddr14", !"0x3be"}
!9200 = !{!"csr", !"pmpaddr15", !"0x3bf"}
!9201 = !{!"csr", !"pmpaddr16", !"0x3c0"}
!9202 = !{!"csr", !"pmpaddr17", !"0x3c1"}
!9203 = !{!"csr", !"pmpaddr18", !"0x3c2"}
!9204 = !{!"csr", !"pmpaddr19", !"0x3c3"}
!9205 = !{!"csr", !"pmpaddr20", !"0x3c4"}
!9206 = !{!"csr", !"pmpaddr21", !"0x3c5"}
!9207 = !{!"csr", !"pmpaddr22", !"0x3c6"}
!9208 = !{!"csr", !"pmpaddr23", !"0x3c7"}
!9209 = !{!"csr", !"pmpaddr24", !"0x3c8"}
!9210 = !{!"csr", !"pmpaddr25", !"0x3c9"}
!9211 = !{!"csr", !"pmpaddr26", !"0x3ca"}
!9212 = !{!"csr", !"pmpaddr27", !"0x3cb"}
!9213 = !{!"csr", !"pmpaddr28", !"0x3cc"}
!9214 = !{!"csr", !"pmpaddr29", !"0x3cd"}
!9215 = !{!"csr", !"pmpaddr30", !"0x3ce"}
!9216 = !{!"csr", !"pmpaddr31", !"0x3cf"}
!9217 = !{!"csr", !"pmpaddr32", !"0x3d0"}
!9218 = !{!"csr", !"pmpaddr33", !"0x3d1"}
!9219 = !{!"csr", !"pmpaddr34", !"0x3d2"}
!9220 = !{!"csr", !"pmpaddr35", !"0x3d3"}
!9221 = !{!"csr", !"pmpaddr36", !"0x3d4"}
!9222 = !{!"csr", !"pmpaddr37", !"0x3d5"}
!9223 = !{!"csr", !"pmpaddr38", !"0x3d6"}
!9224 = !{!"csr", !"pmpaddr39", !"0x3d7"}
!9225 = !{!"csr", !"pmpaddr40", !"0x3d8"}
!9226 = !{!"csr", !"pmpaddr41", !"0x3d9"}
!9227 = !{!"csr", !"pmpaddr42", !"0x3da"}
!9228 = !{!"csr", !"pmpaddr43", !"0x3db"}
!9229 = !{!"csr", !"pmpaddr44", !"0x3dc"}
!9230 = !{!"csr", !"pmpaddr45", !"0x3dd"}
!9231 = !{!"csr", !"pmpaddr46", !"0x3de"}
!9232 = !{!"csr", !"pmpaddr47", !"0x3df"}
!9233 = !{!"csr", !"pmpaddr48", !"0x3e0"}
!9234 = !{!"csr", !"pmpaddr49", !"0x3e1"}
!9235 = !{!"csr", !"pmpaddr50", !"0x3e2"}
!9236 = !{!"csr", !"pmpaddr51", !"0x3e3"}
!9237 = !{!"csr", !"pmpaddr52", !"0x3e4"}
!9238 = !{!"csr", !"pmpaddr53", !"0x3e5"}
!9239 = !{!"csr", !"pmpaddr54", !"0x3e6"}
!9240 = !{!"csr", !"pmpaddr55", !"0x3e7"}
!9241 = !{!"csr", !"pmpaddr56", !"0x3e8"}
!9242 = !{!"csr", !"pmpaddr57", !"0x3e9"}
!9243 = !{!"csr", !"pmpaddr58", !"0x3ea"}
!9244 = !{!"csr", !"pmpaddr59", !"0x3eb"}
!9245 = !{!"csr", !"pmpaddr60", !"0x3ec"}
!9246 = !{!"csr", !"pmpaddr61", !"0x3ed"}
!9247 = !{!"csr", !"pmpaddr62", !"0x3ee"}
!9248 = !{!"csr", !"pmpaddr63", !"0x3ef"}
!9249 = !{!"csr", !"mseccfg", !"0x747"}
!9250 = !{!"csr", !"tselect", !"0x7a0"}
!9251 = !{!"csr", !"tdata1", !"0x7a1"}
!9252 = !{!"csr", !"tdata2", !"0x7a2"}
!9253 = !{!"csr", !"tdata3", !"0x7a3"}
!9254 = !{!"csr", !"tinfo", !"0x7a4"}
!9255 = !{!"csr", !"tcontrol", !"0x7a5"}
!9256 = !{!"csr", !"mcontext", !"0x7a8"}
!9257 = !{!"csr", !"mscontext", !"0x7aa"}
!9258 = !{!"csr", !"dcsr", !"0x7b0"}
!9259 = !{!"csr", !"dpc", !"0x7b1"}
!9260 = !{!"csr", !"dscratch0", !"0x7b2"}
!9261 = !{!"csr", !"dscratch1", !"0x7b3"}
!9262 = !{!"csr", !"mcycle", !"0xB00"}
!9263 = !{!"csr", !"minstret", !"0xB02"}
!9264 = !{!"csr", !"mhpmcounter3", !"0xB03"}
!9265 = !{!"csr", !"mhpmcounter4", !"0xB04"}
!9266 = !{!"csr", !"mhpmcounter5", !"0xB05"}
!9267 = !{!"csr", !"mhpmcounter6", !"0xB06"}
!9268 = !{!"csr", !"mhpmcounter7", !"0xB07"}
!9269 = !{!"csr", !"mhpmcounter8", !"0xB08"}
!9270 = !{!"csr", !"mhpmcounter9", !"0xB09"}
!9271 = !{!"csr", !"mhpmcounter10", !"0xB0A"}
!9272 = !{!"csr", !"mhpmcounter11", !"0xB0B"}
!9273 = !{!"csr", !"mhpmcounter12", !"0xB0C"}
!9274 = !{!"csr", !"mhpmcounter13", !"0xB0D"}
!9275 = !{!"csr", !"mhpmcounter14", !"0xB0E"}
!9276 = !{!"csr", !"mhpmcounter15", !"0xB0F"}
!9277 = !{!"csr", !"mhpmcounter16", !"0xB10"}
!9278 = !{!"csr", !"mhpmcounter17", !"0xB11"}
!9279 = !{!"csr", !"mhpmcounter18", !"0xB12"}
!9280 = !{!"csr", !"mhpmcounter19", !"0xB13"}
!9281 = !{!"csr", !"mhpmcounter20", !"0xB14"}
!9282 = !{!"csr", !"mhpmcounter21", !"0xB15"}
!9283 = !{!"csr", !"mhpmcounter22", !"0xB16"}
!9284 = !{!"csr", !"mhpmcounter23", !"0xB17"}
!9285 = !{!"csr", !"mhpmcounter24", !"0xB18"}
!9286 = !{!"csr", !"mhpmcounter25", !"0xB19"}
!9287 = !{!"csr", !"mhpmcounter26", !"0xB1A"}
!9288 = !{!"csr", !"mhpmcounter27", !"0xB1B"}
!9289 = !{!"csr", !"mhpmcounter28", !"0xB1C"}
!9290 = !{!"csr", !"mhpmcounter29", !"0xB1D"}
!9291 = !{!"csr", !"mhpmcounter30", !"0xB1E"}
!9292 = !{!"csr", !"mhpmcounter31", !"0xB1F"}
!9293 = !{!"csr", !"mcyclecfg", !"0x321"}
!9294 = !{!"csr", !"minstretcfg", !"0x322"}
!9295 = !{!"csr", !"mhpmevent3", !"0x323"}
!9296 = !{!"csr", !"mhpmevent4", !"0x324"}
!9297 = !{!"csr", !"mhpmevent5", !"0x325"}
!9298 = !{!"csr", !"mhpmevent6", !"0x326"}
!9299 = !{!"csr", !"mhpmevent7", !"0x327"}
!9300 = !{!"csr", !"mhpmevent8", !"0x328"}
!9301 = !{!"csr", !"mhpmevent9", !"0x329"}
!9302 = !{!"csr", !"mhpmevent10", !"0x32A"}
!9303 = !{!"csr", !"mhpmevent11", !"0x32B"}
!9304 = !{!"csr", !"mhpmevent12", !"0x32C"}
!9305 = !{!"csr", !"mhpmevent13", !"0x32D"}
!9306 = !{!"csr", !"mhpmevent14", !"0x32E"}
!9307 = !{!"csr", !"mhpmevent15", !"0x32F"}
!9308 = !{!"csr", !"mhpmevent16", !"0x330"}
!9309 = !{!"csr", !"mhpmevent17", !"0x331"}
!9310 = !{!"csr", !"mhpmevent18", !"0x332"}
!9311 = !{!"csr", !"mhpmevent19", !"0x333"}
!9312 = !{!"csr", !"mhpmevent20", !"0x334"}
!9313 = !{!"csr", !"mhpmevent21", !"0x335"}
!9314 = !{!"csr", !"mhpmevent22", !"0x336"}
!9315 = !{!"csr", !"mhpmevent23", !"0x337"}
!9316 = !{!"csr", !"mhpmevent24", !"0x338"}
!9317 = !{!"csr", !"mhpmevent25", !"0x339"}
!9318 = !{!"csr", !"mhpmevent26", !"0x33A"}
!9319 = !{!"csr", !"mhpmevent27", !"0x33B"}
!9320 = !{!"csr", !"mhpmevent28", !"0x33C"}
!9321 = !{!"csr", !"mhpmevent29", !"0x33D"}
!9322 = !{!"csr", !"mhpmevent30", !"0x33E"}
!9323 = !{!"csr", !"mhpmevent31", !"0x33F"}
!9324 = !{!"csr", !"mvendorid", !"0xF11"}
!9325 = !{!"csr", !"marchid", !"0xF12"}
!9326 = !{!"csr", !"mimpid", !"0xF13"}
!9327 = !{!"csr", !"mhartid", !"0xF14"}
!9328 = !{!"csr", !"mconfigptr", !"0xF15"}
!9329 = !{!"csr", !"mtopi", !"0xFB0"}
