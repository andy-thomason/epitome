; The RISC-V RV64 description: RV64I, the base integer instructions. Written by gen_riscv64.py; the
; conventions are those of codegen/targets/aarch64/00-base.ll.

define i64 @ADD(i64 "reg"="x" %n, i64 "reg"="x" %m) "enc"="0000000 m:5 n:5 000 d:5 0110011" "asm"="add {d}, {n}, {m}" "role"="add" {
  %r = add i64 %n, %m
  ret i64 %r
}

define i32 @ADDW(i32 "reg"="w" %n, i32 "reg"="w" %m) "enc"="0000000 m:5 n:5 000 d:5 0111011" "asm"="addw {d}, {n}, {m}" {
  %r = add i32 %n, %m
  ret i32 %r
}

define i64 @SUB(i64 "reg"="x" %n, i64 "reg"="x" %m) "enc"="0100000 m:5 n:5 000 d:5 0110011" "asm"="sub {d}, {n}, {m}" "role"="sub" {
  %r = sub i64 %n, %m
  ret i64 %r
}

define i32 @SUBW(i32 "reg"="w" %n, i32 "reg"="w" %m) "enc"="0100000 m:5 n:5 000 d:5 0111011" "asm"="subw {d}, {n}, {m}" {
  %r = sub i32 %n, %m
  ret i32 %r
}

define i64 @SLL(i64 "reg"="x" %n, i64 "reg"="x" %m) "enc"="0000000 m:5 n:5 001 d:5 0110011" "asm"="sll {d}, {n}, {m}" {
  %r = shl i64 %n, %m
  ret i64 %r
}

define i32 @SLLW(i32 "reg"="w" %n, i32 "reg"="w" %m) "enc"="0000000 m:5 n:5 001 d:5 0111011" "asm"="sllw {d}, {n}, {m}" {
  %r = shl i32 %n, %m
  ret i32 %r
}

define i64 @SRL(i64 "reg"="x" %n, i64 "reg"="x" %m) "enc"="0000000 m:5 n:5 101 d:5 0110011" "asm"="srl {d}, {n}, {m}" {
  %r = lshr i64 %n, %m
  ret i64 %r
}

define i32 @SRLW(i32 "reg"="w" %n, i32 "reg"="w" %m) "enc"="0000000 m:5 n:5 101 d:5 0111011" "asm"="srlw {d}, {n}, {m}" {
  %r = lshr i32 %n, %m
  ret i32 %r
}

define i64 @SRA(i64 "reg"="x" %n, i64 "reg"="x" %m) "enc"="0100000 m:5 n:5 101 d:5 0110011" "asm"="sra {d}, {n}, {m}" {
  %r = ashr i64 %n, %m
  ret i64 %r
}

define i32 @SRAW(i32 "reg"="w" %n, i32 "reg"="w" %m) "enc"="0100000 m:5 n:5 101 d:5 0111011" "asm"="sraw {d}, {n}, {m}" {
  %r = ashr i32 %n, %m
  ret i32 %r
}

define i64 @AND(i64 "reg"="x" %n, i64 "reg"="x" %m) "enc"="0000000 m:5 n:5 111 d:5 0110011" "asm"="and {d}, {n}, {m}" {
  %r = and i64 %n, %m
  ret i64 %r
}

define i32 @ANDw(i32 "reg"="w" %n, i32 "reg"="w" %m) "enc"="0000000 m:5 n:5 111 d:5 0110011" "asm"="and {d}, {n}, {m}" "nodecode" {
  %r = and i32 %n, %m
  ret i32 %r
}

define i1 @ANDb(i1 "reg"="w" %n, i1 "reg"="w" %m) "enc"="0000000 m:5 n:5 111 d:5 0110011" "asm"="and {d}, {n}, {m}" "nodecode" {
  %r = and i1 %n, %m
  ret i1 %r
}

define i64 @OR(i64 "reg"="x" %n, i64 "reg"="x" %m) "enc"="0000000 m:5 n:5 110 d:5 0110011" "asm"="or {d}, {n}, {m}" {
  %r = or i64 %n, %m
  ret i64 %r
}

define i32 @ORw(i32 "reg"="w" %n, i32 "reg"="w" %m) "enc"="0000000 m:5 n:5 110 d:5 0110011" "asm"="or {d}, {n}, {m}" "nodecode" {
  %r = or i32 %n, %m
  ret i32 %r
}

define i1 @ORb(i1 "reg"="w" %n, i1 "reg"="w" %m) "enc"="0000000 m:5 n:5 110 d:5 0110011" "asm"="or {d}, {n}, {m}" "nodecode" {
  %r = or i1 %n, %m
  ret i1 %r
}

define i64 @XOR(i64 "reg"="x" %n, i64 "reg"="x" %m) "enc"="0000000 m:5 n:5 100 d:5 0110011" "asm"="xor {d}, {n}, {m}" {
  %r = xor i64 %n, %m
  ret i64 %r
}

define i32 @XORw(i32 "reg"="w" %n, i32 "reg"="w" %m) "enc"="0000000 m:5 n:5 100 d:5 0110011" "asm"="xor {d}, {n}, {m}" "nodecode" {
  %r = xor i32 %n, %m
  ret i32 %r
}

define i1 @XORb(i1 "reg"="w" %n, i1 "reg"="w" %m) "enc"="0000000 m:5 n:5 100 d:5 0110011" "asm"="xor {d}, {n}, {m}" "nodecode" {
  %r = xor i1 %n, %m
  ret i1 %r
}

define ptr @ADDp(ptr "reg"="x" %n, i64 "reg"="x" %m) "enc"="0000000 m:5 n:5 000 d:5 0110011" "asm"="add {d}, {n}, {m}" "nodecode" {
  %r = getelementptr i8, ptr %n, i64 %m
  ret ptr %r
}

define i64 @ADDI(i64 "reg"="x" %n, i64 "imm" %imm) "enc"="imm:s12 n:5 000 d:5 0010011" "asm"="addi {d}, {n}, {imm}" "role"="addi" {
  %r = add i64 %n, %imm
  ret i64 %r
}

define ptr @ADDIp(ptr "reg"="x" %n, i64 "imm" %imm) "enc"="imm:s12 n:5 000 d:5 0010011" "asm"="addi {d}, {n}, {imm}" "nodecode" {
  %r = getelementptr i8, ptr %n, i64 %imm
  ret ptr %r
}

define i32 @ADDIW(i32 "reg"="w" %n, i32 "imm" %imm) "enc"="imm:s12 n:5 000 d:5 0011011" "asm"="addiw {d}, {n}, {imm}" "role"="addi.w" {
  %r = add i32 %n, %imm
  ret i32 %r
}

define i64 @ANDI(i64 "reg"="x" %n, i64 "imm" %imm) "enc"="imm:s12 n:5 111 d:5 0010011" "asm"="andi {d}, {n}, {imm}" {
  %r = and i64 %n, %imm
  ret i64 %r
}

define i32 @ANDIw(i32 "reg"="w" %n, i32 "imm" %imm) "enc"="imm:s12 n:5 111 d:5 0010011" "asm"="andi {d}, {n}, {imm}" "nodecode" {
  %r = and i32 %n, %imm
  ret i32 %r
}

define i1 @ANDIb(i1 "reg"="w" %n, i1 "imm" %imm) "enc"="imm:s12 n:5 111 d:5 0010011" "asm"="andi {d}, {n}, {imm}" "nodecode" {
  %r = and i1 %n, %imm
  ret i1 %r
}

define i64 @ORI(i64 "reg"="x" %n, i64 "imm" %imm) "enc"="imm:s12 n:5 110 d:5 0010011" "asm"="ori {d}, {n}, {imm}" {
  %r = or i64 %n, %imm
  ret i64 %r
}

define i32 @ORIw(i32 "reg"="w" %n, i32 "imm" %imm) "enc"="imm:s12 n:5 110 d:5 0010011" "asm"="ori {d}, {n}, {imm}" "nodecode" {
  %r = or i32 %n, %imm
  ret i32 %r
}

define i1 @ORIb(i1 "reg"="w" %n, i1 "imm" %imm) "enc"="imm:s12 n:5 110 d:5 0010011" "asm"="ori {d}, {n}, {imm}" "nodecode" {
  %r = or i1 %n, %imm
  ret i1 %r
}

define i64 @XORI(i64 "reg"="x" %n, i64 "imm" %imm) "enc"="imm:s12 n:5 100 d:5 0010011" "asm"="xori {d}, {n}, {imm}" {
  %r = xor i64 %n, %imm
  ret i64 %r
}

define i32 @XORIw(i32 "reg"="w" %n, i32 "imm" %imm) "enc"="imm:s12 n:5 100 d:5 0010011" "asm"="xori {d}, {n}, {imm}" "nodecode" {
  %r = xor i32 %n, %imm
  ret i32 %r
}

define i1 @XORIb(i1 "reg"="w" %n, i1 "imm" %imm) "enc"="imm:s12 n:5 100 d:5 0010011" "asm"="xori {d}, {n}, {imm}" "nodecode" {
  %r = xor i1 %n, %imm
  ret i1 %r
}

define i64 @SLLI(i64 "reg"="x" %n, i64 "imm" %sh) "enc"="000000 sh:6 n:5 001 d:5 0010011" "asm"="slli {d}, {n}, {sh}" "role"="shli" {
  %r = shl i64 %n, %sh
  ret i64 %r
}

define i32 @SLLIW(i32 "reg"="w" %n, i32 "imm" %sh) "enc"="0000000 sh:5 n:5 001 d:5 0011011" "asm"="slliw {d}, {n}, {sh}" {
  %r = shl i32 %n, %sh
  ret i32 %r
}

define i64 @SRLI(i64 "reg"="x" %n, i64 "imm" %sh) "enc"="000000 sh:6 n:5 101 d:5 0010011" "asm"="srli {d}, {n}, {sh}" "role"="shri" {
  %r = lshr i64 %n, %sh
  ret i64 %r
}

define i32 @SRLIW(i32 "reg"="w" %n, i32 "imm" %sh) "enc"="0000000 sh:5 n:5 101 d:5 0011011" "asm"="srliw {d}, {n}, {sh}" {
  %r = lshr i32 %n, %sh
  ret i32 %r
}

define i64 @SRAI(i64 "reg"="x" %n, i64 "imm" %sh) "enc"="010000 sh:6 n:5 101 d:5 0010011" "asm"="srai {d}, {n}, {sh}" "role"="sari" {
  %r = ashr i64 %n, %sh
  ret i64 %r
}

define i32 @SRAIW(i32 "reg"="w" %n, i32 "imm" %sh) "enc"="0100000 sh:5 n:5 101 d:5 0011011" "asm"="sraiw {d}, {n}, {sh}" {
  %r = ashr i32 %n, %sh
  ret i32 %r
}

define i64 @NEG(i64 "reg"="x" %n) "enc"="0100000 n:5 00000 000 d:5 0110011" "asm"="neg {d}, {n}" {
  %r = sub i64 0, %n
  ret i64 %r
}

define i32 @NEGW(i32 "reg"="w" %n) "enc"="0100000 n:5 00000 000 d:5 0111011" "asm"="negw {d}, {n}" {
  %r = sub i32 0, %n
  ret i32 %r
}

define i32 @SEXTW(i64 "reg"="x" %n) "enc"="000000000000 n:5 000 d:5 0011011" "asm"="sext.w {d}, {n}" {
  %r = trunc i64 %n to i32
  ret i32 %r
}

define i1 @SLT(i64 "reg"="x" %n, i64 "reg"="x" %m) "enc"="0000000 m:5 n:5 010 d:5 0110011" "asm"="slt {d}, {n}, {m}" "zext"="1" {
  %r = icmp slt i64 %n, %m
  ret i1 %r
}

define i1 @SLTs(i64 "reg"="x" %n, i64 "reg"="x" %m) "enc"="0000000 rb:5=n ra:5=m 010 d:5 0110011" "asm"="slt {d}, {m}, {n}" "zext"="1" "nodecode" {
  %r = icmp sgt i64 %n, %m
  ret i1 %r
}

define i1 @SLTI(i64 "reg"="x" %n, i64 "imm" %imm) "enc"="imm:s12 n:5 010 d:5 0010011" "asm"="slti {d}, {n}, {imm}" "zext"="1" {
  %r = icmp slt i64 %n, %imm
  ret i1 %r
}

define i1 @SLTw(i32 "reg"="w" %n, i32 "reg"="w" %m) "enc"="0000000 m:5 n:5 010 d:5 0110011" "asm"="slt {d}, {n}, {m}" "zext"="1" "nodecode" {
  %r = icmp slt i32 %n, %m
  ret i1 %r
}

define i1 @SLTsw(i32 "reg"="w" %n, i32 "reg"="w" %m) "enc"="0000000 rb:5=n ra:5=m 010 d:5 0110011" "asm"="slt {d}, {m}, {n}" "zext"="1" "nodecode" {
  %r = icmp sgt i32 %n, %m
  ret i1 %r
}

define i1 @SLTIw(i32 "reg"="w" %n, i32 "imm" %imm) "enc"="imm:s12 n:5 010 d:5 0010011" "asm"="slti {d}, {n}, {imm}" "zext"="1" "nodecode" {
  %r = icmp slt i32 %n, %imm
  ret i1 %r
}

define i1 @SLTU(i64 "reg"="x" %n, i64 "reg"="x" %m) "enc"="0000000 m:5 n:5 011 d:5 0110011" "asm"="sltu {d}, {n}, {m}" "zext"="1" {
  %r = icmp ult i64 %n, %m
  ret i1 %r
}

define i1 @SLTUs(i64 "reg"="x" %n, i64 "reg"="x" %m) "enc"="0000000 rb:5=n ra:5=m 011 d:5 0110011" "asm"="sltu {d}, {m}, {n}" "zext"="1" "nodecode" {
  %r = icmp ugt i64 %n, %m
  ret i1 %r
}

define i1 @SLTUI(i64 "reg"="x" %n, i64 "imm" %imm) "enc"="imm:s12 n:5 011 d:5 0010011" "asm"="sltiu {d}, {n}, {imm}" "zext"="1" {
  %r = icmp ult i64 %n, %imm
  ret i1 %r
}

define i1 @SLTUw(i32 "reg"="w" %n, i32 "reg"="w" %m) "enc"="0000000 m:5 n:5 011 d:5 0110011" "asm"="sltu {d}, {n}, {m}" "zext"="1" "nodecode" {
  %r = icmp ult i32 %n, %m
  ret i1 %r
}

define i1 @SLTUsw(i32 "reg"="w" %n, i32 "reg"="w" %m) "enc"="0000000 rb:5=n ra:5=m 011 d:5 0110011" "asm"="sltu {d}, {m}, {n}" "zext"="1" "nodecode" {
  %r = icmp ugt i32 %n, %m
  ret i1 %r
}

define i1 @SLTUIw(i32 "reg"="w" %n, i32 "imm" %imm) "enc"="imm:s12 n:5 011 d:5 0010011" "asm"="sltiu {d}, {n}, {imm}" "zext"="1" "nodecode" {
  %r = icmp ult i32 %n, %imm
  ret i1 %r
}

define i1 @SEQZ(i64 "reg"="x" %n) "enc"="000000000001 n:5 011 d:5 0010011" "asm"="seqz {d}, {n}" "zext"="1" {
  %r = icmp eq i64 %n, 0
  ret i1 %r
}

define i1 @SNEZ(i64 "reg"="x" %n) "enc"="0000000 n:5 00000 011 d:5 0110011" "asm"="snez {d}, {n}" "zext"="1" {
  %r = icmp ne i64 %n, 0
  ret i1 %r
}

define i1 @SEQZw(i32 "reg"="w" %n) "enc"="000000000001 n:5 011 d:5 0010011" "asm"="seqz {d}, {n}" "zext"="1" "nodecode" {
  %r = icmp eq i32 %n, 0
  ret i1 %r
}

define i1 @SNEZw(i32 "reg"="w" %n) "enc"="0000000 n:5 00000 011 d:5 0110011" "asm"="snez {d}, {n}" "zext"="1" "nodecode" {
  %r = icmp ne i32 %n, 0
  ret i1 %r
}

define i64 @MV(i64 "reg"="x" %n) "enc"="000000000000 n:5 000 d:5 0010011" "asm"="mv {d}, {n}" "copy"="x" {
  ret i64 %n
}

define i32 @MVw(i32 "reg"="w" %n) "enc"="000000000000 n:5 000 d:5 0010011" "asm"="mv {d}, {n}" "copy"="w" "nodecode" {
  ret i32 %n
}

define i64 @LI(i64 "imm" %imm) "enc"="imm:s12 00000 000 d:5 0010011" "asm"="li {d}, {imm}" {
  ret i64 %imm
}

define i32 @LIw(i32 "imm" %imm) "enc"="imm:s12 00000 000 d:5 0010011" "asm"="li {d}, {imm}" "nodecode" {
  ret i32 %imm
}

define i64 @LUI(i64 "imm" %imm) "enc"="imm20:s20=imm>>12 d:5 0110111" "asm"="lui {d}, {=(imm>>12)&1048575}" {
  ret i64 %imm
}

define i32 @LUIw(i32 "imm" %imm) "enc"="imm20:s20=imm>>12 d:5 0110111" "asm"="lui {d}, {=(imm>>12)&1048575}" "nodecode" {
  ret i32 %imm
}

define i64 @ZEXTBx(i8 "reg"="w" %n) "enc"="000011111111 n:5 111 d:5 0010011" "asm"="andi {d}, {n}, 255" "zext"="8" "nodecode" {
  %r = zext i8 %n to i64
  ret i64 %r
}

define i64 @ZEXT1x(i1 "reg"="w" %n) "enc"="000000000001 n:5 111 d:5 0010011" "asm"="andi {d}, {n}, 1" "zext"="1" "nodecode" {
  %r = zext i1 %n to i64
  ret i64 %r
}

define i32 @ZEXTBw(i8 "reg"="w" %n) "enc"="000011111111 n:5 111 d:5 0010011" "asm"="andi {d}, {n}, 255" "zext"="8" "nodecode" "role"="zext.8" {
  %r = zext i8 %n to i32
  ret i32 %r
}

define i32 @ZEXT1w(i1 "reg"="w" %n) "enc"="000000000001 n:5 111 d:5 0010011" "asm"="andi {d}, {n}, 1" "zext"="1" "nodecode" "role"="zext.1" {
  %r = zext i1 %n to i32
  ret i32 %r
}

define ptr @LUIsym(ptr "sym" %s) "enc"="imm20:20=s>>12 d:5 0110111" "asm"="lui {d}, {s}" "reloc"="hi20" "role"="addr.hi" "nomatch" "nodecode" {
  ret ptr %s
}

define ptr @ADDIsym(ptr "reg"="x" %n, ptr "sym" %s) "enc"="imm:12=s n:5 000 d:5 0010011" "asm"="addi {d}, {n}, {s}" "reloc"="lo12_i" "role"="addr.lo" "nomatch" "nodecode" {
  ret ptr %n
}

; An address PC-relative: auipc's high part, then the low part, whose symbol
; is the auipc's label (la, lla).
define ptr @AUIPCpcrel(ptr "sym" %s) "enc"="imm20:20=s>>12 d:5 0010111" "asm"="auipc {d}, {s}" "reloc"="pcrel_hi20" "nomatch" "nodecode" {
  ret ptr %s
}

define ptr @ADDIpcrel(ptr "reg"="x" %n, ptr "sym" %s) "enc"="imm:12=s n:5 000 d:5 0010011" "asm"="addi {d}, {n}, {s}" "reloc"="pcrel_lo12_i" "nomatch" "nodecode" {
  ret ptr %n
}

; Loads and stores at a PC-relative address's low part, after its auipc
; (`lw a0, sym` and `sw a0, sym, t0`).
define void @LBpcrel(i64 "reg"="x" %rd, i64 "reg"="x" %n, ptr "sym" %s) "enc"="imm:12=s n:5 000 rd:5 0000011" "asm"="lb {rd}, {s}({n})" "reloc"="pcrel_lo12_i" "asmonly" "nodecode" {
  ret void
}

define void @LHpcrel(i64 "reg"="x" %rd, i64 "reg"="x" %n, ptr "sym" %s) "enc"="imm:12=s n:5 001 rd:5 0000011" "asm"="lh {rd}, {s}({n})" "reloc"="pcrel_lo12_i" "asmonly" "nodecode" {
  ret void
}

define void @LWpcrel(i64 "reg"="x" %rd, i64 "reg"="x" %n, ptr "sym" %s) "enc"="imm:12=s n:5 010 rd:5 0000011" "asm"="lw {rd}, {s}({n})" "reloc"="pcrel_lo12_i" "asmonly" "nodecode" {
  ret void
}

define void @LDpcrel(i64 "reg"="x" %rd, i64 "reg"="x" %n, ptr "sym" %s) "enc"="imm:12=s n:5 011 rd:5 0000011" "asm"="ld {rd}, {s}({n})" "reloc"="pcrel_lo12_i" "asmonly" "nodecode" {
  ret void
}

define void @LBUpcrel(i64 "reg"="x" %rd, i64 "reg"="x" %n, ptr "sym" %s) "enc"="imm:12=s n:5 100 rd:5 0000011" "asm"="lbu {rd}, {s}({n})" "reloc"="pcrel_lo12_i" "asmonly" "nodecode" {
  ret void
}

define void @LHUpcrel(i64 "reg"="x" %rd, i64 "reg"="x" %n, ptr "sym" %s) "enc"="imm:12=s n:5 101 rd:5 0000011" "asm"="lhu {rd}, {s}({n})" "reloc"="pcrel_lo12_i" "asmonly" "nodecode" {
  ret void
}

define void @LWUpcrel(i64 "reg"="x" %rd, i64 "reg"="x" %n, ptr "sym" %s) "enc"="imm:12=s n:5 110 rd:5 0000011" "asm"="lwu {rd}, {s}({n})" "reloc"="pcrel_lo12_i" "asmonly" "nodecode" {
  ret void
}

define void @FLHpcrel(i64 "reg"="d" %rd, i64 "reg"="x" %n, ptr "sym" %s) "enc"="imm:12=s n:5 001 rd:5 0000111" "asm"="flh {rd}, {s}({n})" "reloc"="pcrel_lo12_i" "asmonly" "nodecode" {
  ret void
}

define void @FLWpcrel(i64 "reg"="d" %rd, i64 "reg"="x" %n, ptr "sym" %s) "enc"="imm:12=s n:5 010 rd:5 0000111" "asm"="flw {rd}, {s}({n})" "reloc"="pcrel_lo12_i" "asmonly" "nodecode" {
  ret void
}

define void @FLDpcrel(i64 "reg"="d" %rd, i64 "reg"="x" %n, ptr "sym" %s) "enc"="imm:12=s n:5 011 rd:5 0000111" "asm"="fld {rd}, {s}({n})" "reloc"="pcrel_lo12_i" "asmonly" "nodecode" {
  ret void
}

define void @FLQpcrel(i64 "reg"="d" %rd, i64 "reg"="x" %n, ptr "sym" %s) "enc"="imm:12=s n:5 100 rd:5 0000111" "asm"="flq {rd}, {s}({n})" "reloc"="pcrel_lo12_i" "asmonly" "nodecode" {
  ret void
}

define void @SBpcrel(i64 "reg"="x" %rs2, i64 "reg"="x" %n, ptr "sym" %s) "enc"="hi:7=s>>5 rs2:5 n:5 000 lo:5=s&31 0100011" "asm"="sb {rs2}, {s}({n})" "reloc"="pcrel_lo12_s" "asmonly" "nodecode" {
  ret void
}

define void @SHpcrel(i64 "reg"="x" %rs2, i64 "reg"="x" %n, ptr "sym" %s) "enc"="hi:7=s>>5 rs2:5 n:5 001 lo:5=s&31 0100011" "asm"="sh {rs2}, {s}({n})" "reloc"="pcrel_lo12_s" "asmonly" "nodecode" {
  ret void
}

define void @SWpcrel(i64 "reg"="x" %rs2, i64 "reg"="x" %n, ptr "sym" %s) "enc"="hi:7=s>>5 rs2:5 n:5 010 lo:5=s&31 0100011" "asm"="sw {rs2}, {s}({n})" "reloc"="pcrel_lo12_s" "asmonly" "nodecode" {
  ret void
}

define void @SDpcrel(i64 "reg"="x" %rs2, i64 "reg"="x" %n, ptr "sym" %s) "enc"="hi:7=s>>5 rs2:5 n:5 011 lo:5=s&31 0100011" "asm"="sd {rs2}, {s}({n})" "reloc"="pcrel_lo12_s" "asmonly" "nodecode" {
  ret void
}

define void @FSHpcrel(i64 "reg"="d" %rs2, i64 "reg"="x" %n, ptr "sym" %s) "enc"="hi:7=s>>5 rs2:5 n:5 001 lo:5=s&31 0100111" "asm"="fsh {rs2}, {s}({n})" "reloc"="pcrel_lo12_s" "asmonly" "nodecode" {
  ret void
}

define void @FSWpcrel(i64 "reg"="d" %rs2, i64 "reg"="x" %n, ptr "sym" %s) "enc"="hi:7=s>>5 rs2:5 n:5 010 lo:5=s&31 0100111" "asm"="fsw {rs2}, {s}({n})" "reloc"="pcrel_lo12_s" "asmonly" "nodecode" {
  ret void
}

define void @FSDpcrel(i64 "reg"="d" %rs2, i64 "reg"="x" %n, ptr "sym" %s) "enc"="hi:7=s>>5 rs2:5 n:5 011 lo:5=s&31 0100111" "asm"="fsd {rs2}, {s}({n})" "reloc"="pcrel_lo12_s" "asmonly" "nodecode" {
  ret void
}

define void @FSQpcrel(i64 "reg"="d" %rs2, i64 "reg"="x" %n, ptr "sym" %s) "enc"="hi:7=s>>5 rs2:5 n:5 100 lo:5=s&31 0100111" "asm"="fsq {rs2}, {s}({n})" "reloc"="pcrel_lo12_s" "asmonly" "nodecode" {
  ret void
}

; A thread-local variable's address: the thread pointer plus its offset
; (see `prepare::thread_locals`).
define ptr @LUItprel(ptr "sym" %s) "enc"="imm20:20=s>>12 d:5 0110111" "asm"="lui {d}, {s}" "reloc"="tprel_hi20" "role"="tprel.hi" "nomatch" "nodecode" {
  ret ptr %s
}

define ptr @ADDtprel(ptr "reg"="x" %n, ptr "sym" %s) "enc"="0000000 00100 n:5 000 d:5 0110011" "asm"="add {d}, {n}, tp, {s}" "reloc"="tprel_add" "role"="tprel.add" "nomatch" "nodecode" {
  ret ptr %n
}

define ptr @ADDItprel(ptr "reg"="x" %n, ptr "sym" %s) "enc"="imm:12=s n:5 000 d:5 0010011" "asm"="addi {d}, {n}, {s}" "reloc"="tprel_lo12_i" "role"="tprel.lo" "nomatch" "nodecode" {
  ret ptr %n
}

; The thread pointer (llvm.thread.pointer).
define ptr @MVtp() "enc"="000000000000 00100 000 d:5 0010011" "asm"="mv {d}, tp" "role"="thread.pointer" "nodecode" {
  %p = call ptr @llvm.thread.pointer.p0()
  ret ptr %p
}

declare ptr @llvm.thread.pointer.p0()

; Loads and stores, at a signed 12-bit offset from a register.

define i8 @LBU(ptr "reg"="x" %n, i64 "imm" %off) "enc"="off:s12 n:5 100 d:5 0000011" "asm"="lbu {d}, {off}({n})" "role"="load.b" "zext"="8" {
  %p = getelementptr i8, ptr %n, i64 %off
  %v = load i8, ptr %p
  ret i8 %v
}

define i16 @LHU(ptr "reg"="x" %n, i64 "imm" %off) "enc"="off:s12 n:5 101 d:5 0000011" "asm"="lhu {d}, {off}({n})" "role"="load.h" "zext"="16" {
  %p = getelementptr i8, ptr %n, i64 %off
  %v = load i16, ptr %p
  ret i16 %v
}

define i32 @LW(ptr "reg"="x" %n, i64 "imm" %off) "enc"="off:s12 n:5 010 d:5 0000011" "asm"="lw {d}, {off}({n})" "role"="load.w" {
  %p = getelementptr i8, ptr %n, i64 %off
  %v = load i32, ptr %p
  ret i32 %v
}

define i64 @LD(ptr "reg"="x" %n, i64 "imm" %off) "enc"="off:s12 n:5 011 d:5 0000011" "asm"="ld {d}, {off}({n})" "role"="load.x" {
  %p = getelementptr i8, ptr %n, i64 %off
  %v = load i64, ptr %p
  ret i64 %v
}

define ptr @LDp(ptr "reg"="x" %n, i64 "imm" %off) "enc"="off:s12 n:5 011 d:5 0000011" "asm"="ld {d}, {off}({n})" "nodecode" {
  %p = getelementptr i8, ptr %n, i64 %off
  %v = load ptr, ptr %p
  ret ptr %v
}

define i32 @LBs32(ptr "reg"="x" %n, i64 "imm" %off) "enc"="off:s12 n:5 000 d:5 0000011" "asm"="lb {d}, {off}({n})" "nodecode" {
  %p = getelementptr i8, ptr %n, i64 %off
  %v = load i8, ptr %p
  %r = sext i8 %v to i32
  ret i32 %r
}

define i64 @LBs64(ptr "reg"="x" %n, i64 "imm" %off) "enc"="off:s12 n:5 000 d:5 0000011" "asm"="lb {d}, {off}({n})" {
  %p = getelementptr i8, ptr %n, i64 %off
  %v = load i8, ptr %p
  %r = sext i8 %v to i64
  ret i64 %r
}

define i32 @LHs32(ptr "reg"="x" %n, i64 "imm" %off) "enc"="off:s12 n:5 001 d:5 0000011" "asm"="lh {d}, {off}({n})" "nodecode" {
  %p = getelementptr i8, ptr %n, i64 %off
  %v = load i16, ptr %p
  %r = sext i16 %v to i32
  ret i32 %r
}

define i64 @LHs64(ptr "reg"="x" %n, i64 "imm" %off) "enc"="off:s12 n:5 001 d:5 0000011" "asm"="lh {d}, {off}({n})" {
  %p = getelementptr i8, ptr %n, i64 %off
  %v = load i16, ptr %p
  %r = sext i16 %v to i64
  ret i64 %r
}

define i64 @LWs64(ptr "reg"="x" %n, i64 "imm" %off) "enc"="off:s12 n:5 010 d:5 0000011" "asm"="lw {d}, {off}({n})" "nodecode" {
  %p = getelementptr i8, ptr %n, i64 %off
  %v = load i32, ptr %p
  %r = sext i32 %v to i64
  ret i64 %r
}

define i32 @LBUz32(ptr "reg"="x" %n, i64 "imm" %off) "enc"="off:s12 n:5 100 d:5 0000011" "asm"="lbu {d}, {off}({n})" "nodecode" {
  %p = getelementptr i8, ptr %n, i64 %off
  %v = load i8, ptr %p
  %r = zext i8 %v to i32
  ret i32 %r
}

define i64 @LBUz64(ptr "reg"="x" %n, i64 "imm" %off) "enc"="off:s12 n:5 100 d:5 0000011" "asm"="lbu {d}, {off}({n})" "nodecode" {
  %p = getelementptr i8, ptr %n, i64 %off
  %v = load i8, ptr %p
  %r = zext i8 %v to i64
  ret i64 %r
}

define i32 @LHUz32(ptr "reg"="x" %n, i64 "imm" %off) "enc"="off:s12 n:5 101 d:5 0000011" "asm"="lhu {d}, {off}({n})" "nodecode" {
  %p = getelementptr i8, ptr %n, i64 %off
  %v = load i16, ptr %p
  %r = zext i16 %v to i32
  ret i32 %r
}

define i64 @LHUz64(ptr "reg"="x" %n, i64 "imm" %off) "enc"="off:s12 n:5 101 d:5 0000011" "asm"="lhu {d}, {off}({n})" "nodecode" {
  %p = getelementptr i8, ptr %n, i64 %off
  %v = load i16, ptr %p
  %r = zext i16 %v to i64
  ret i64 %r
}

define i64 @LWUz64(ptr "reg"="x" %n, i64 "imm" %off) "enc"="off:s12 n:5 110 d:5 0000011" "asm"="lwu {d}, {off}({n})" {
  %p = getelementptr i8, ptr %n, i64 %off
  %v = load i32, ptr %p
  %r = zext i32 %v to i64
  ret i64 %r
}

define void @SB(i8 "reg"="w" %t, ptr "reg"="x" %n, i64 "imm" %off) "enc"="hi:s7=off>>5 t:5 n:5 000 lo:5=off&31 0100011" "asm"="sb {t}, {off}({n})" "role"="store.b" {
  %p = getelementptr i8, ptr %n, i64 %off
  store i8 %t, ptr %p
  ret void
}

define void @SH(i16 "reg"="w" %t, ptr "reg"="x" %n, i64 "imm" %off) "enc"="hi:s7=off>>5 t:5 n:5 001 lo:5=off&31 0100011" "asm"="sh {t}, {off}({n})" "role"="store.h" {
  %p = getelementptr i8, ptr %n, i64 %off
  store i16 %t, ptr %p
  ret void
}

define void @SW(i32 "reg"="w" %t, ptr "reg"="x" %n, i64 "imm" %off) "enc"="hi:s7=off>>5 t:5 n:5 010 lo:5=off&31 0100011" "asm"="sw {t}, {off}({n})" "role"="store.w" {
  %p = getelementptr i8, ptr %n, i64 %off
  store i32 %t, ptr %p
  ret void
}

define void @SD(i64 "reg"="x" %t, ptr "reg"="x" %n, i64 "imm" %off) "enc"="hi:s7=off>>5 t:5 n:5 011 lo:5=off&31 0100011" "asm"="sd {t}, {off}({n})" "role"="store.x" {
  %p = getelementptr i8, ptr %n, i64 %off
  store i64 %t, ptr %p
  ret void
}

define void @SDp(ptr "reg"="x" %t, ptr "reg"="x" %n, i64 "imm" %off) "enc"="hi:s7=off>>5 t:5 n:5 011 lo:5=off&31 0100011" "asm"="sd {t}, {off}({n})" "nodecode" {
  %p = getelementptr i8, ptr %n, i64 %off
  store ptr %t, ptr %p
  ret void
}

; Branches, comparing two registers. The swapped forms (bgt is blt with
; its operands swapped) are for the predicates RISC-V has no instruction for.

define void @BEQ(i64 "reg"="x" %n, i64 "reg"="x" %m, ptr "label" %dest) "enc"="b12:s1=dest>>12 b10:6=(dest>>5)&63 m:5 n:5 000 b4:4=(dest>>1)&15 b11:1=(dest>>11)&1 1100011" "asm"="beq {n}, {m}, {dest}" "inverse"="BNE" {
  %c = icmp eq i64 %n, %m
  br i1 %c, label %taken, label %next
taken:
  indirectbr ptr %dest, []
next:
  ret void
}

define void @BNE(i64 "reg"="x" %n, i64 "reg"="x" %m, ptr "label" %dest) "enc"="b12:s1=dest>>12 b10:6=(dest>>5)&63 m:5 n:5 001 b4:4=(dest>>1)&15 b11:1=(dest>>11)&1 1100011" "asm"="bne {n}, {m}, {dest}" "inverse"="BEQ" {
  %c = icmp ne i64 %n, %m
  br i1 %c, label %taken, label %next
taken:
  indirectbr ptr %dest, []
next:
  ret void
}

define void @BLT(i64 "reg"="x" %n, i64 "reg"="x" %m, ptr "label" %dest) "enc"="b12:s1=dest>>12 b10:6=(dest>>5)&63 m:5 n:5 100 b4:4=(dest>>1)&15 b11:1=(dest>>11)&1 1100011" "asm"="blt {n}, {m}, {dest}" "inverse"="BGE" {
  %c = icmp slt i64 %n, %m
  br i1 %c, label %taken, label %next
taken:
  indirectbr ptr %dest, []
next:
  ret void
}

define void @BGE(i64 "reg"="x" %n, i64 "reg"="x" %m, ptr "label" %dest) "enc"="b12:s1=dest>>12 b10:6=(dest>>5)&63 m:5 n:5 101 b4:4=(dest>>1)&15 b11:1=(dest>>11)&1 1100011" "asm"="bge {n}, {m}, {dest}" "inverse"="BLT" {
  %c = icmp sge i64 %n, %m
  br i1 %c, label %taken, label %next
taken:
  indirectbr ptr %dest, []
next:
  ret void
}

define void @BLTU(i64 "reg"="x" %n, i64 "reg"="x" %m, ptr "label" %dest) "enc"="b12:s1=dest>>12 b10:6=(dest>>5)&63 m:5 n:5 110 b4:4=(dest>>1)&15 b11:1=(dest>>11)&1 1100011" "asm"="bltu {n}, {m}, {dest}" "inverse"="BGEU" {
  %c = icmp ult i64 %n, %m
  br i1 %c, label %taken, label %next
taken:
  indirectbr ptr %dest, []
next:
  ret void
}

define void @BGEU(i64 "reg"="x" %n, i64 "reg"="x" %m, ptr "label" %dest) "enc"="b12:s1=dest>>12 b10:6=(dest>>5)&63 m:5 n:5 111 b4:4=(dest>>1)&15 b11:1=(dest>>11)&1 1100011" "asm"="bgeu {n}, {m}, {dest}" "inverse"="BLTU" {
  %c = icmp uge i64 %n, %m
  br i1 %c, label %taken, label %next
taken:
  indirectbr ptr %dest, []
next:
  ret void
}

define void @BGT(i64 "reg"="x" %n, i64 "reg"="x" %m, ptr "label" %dest) "enc"="b12:s1=dest>>12 b10:6=(dest>>5)&63 rb:5=n ra:5=m 100 b4:4=(dest>>1)&15 b11:1=(dest>>11)&1 1100011" "asm"="blt {m}, {n}, {dest}" "inverse"="BLE" "nodecode" {
  %c = icmp sgt i64 %n, %m
  br i1 %c, label %taken, label %next
taken:
  indirectbr ptr %dest, []
next:
  ret void
}

define void @BLE(i64 "reg"="x" %n, i64 "reg"="x" %m, ptr "label" %dest) "enc"="b12:s1=dest>>12 b10:6=(dest>>5)&63 rb:5=n ra:5=m 101 b4:4=(dest>>1)&15 b11:1=(dest>>11)&1 1100011" "asm"="bge {m}, {n}, {dest}" "inverse"="BGT" "nodecode" {
  %c = icmp sle i64 %n, %m
  br i1 %c, label %taken, label %next
taken:
  indirectbr ptr %dest, []
next:
  ret void
}

define void @BGTU(i64 "reg"="x" %n, i64 "reg"="x" %m, ptr "label" %dest) "enc"="b12:s1=dest>>12 b10:6=(dest>>5)&63 rb:5=n ra:5=m 110 b4:4=(dest>>1)&15 b11:1=(dest>>11)&1 1100011" "asm"="bltu {m}, {n}, {dest}" "inverse"="BLEU" "nodecode" {
  %c = icmp ugt i64 %n, %m
  br i1 %c, label %taken, label %next
taken:
  indirectbr ptr %dest, []
next:
  ret void
}

define void @BLEU(i64 "reg"="x" %n, i64 "reg"="x" %m, ptr "label" %dest) "enc"="b12:s1=dest>>12 b10:6=(dest>>5)&63 rb:5=n ra:5=m 111 b4:4=(dest>>1)&15 b11:1=(dest>>11)&1 1100011" "asm"="bgeu {m}, {n}, {dest}" "inverse"="BGTU" "nodecode" {
  %c = icmp ule i64 %n, %m
  br i1 %c, label %taken, label %next
taken:
  indirectbr ptr %dest, []
next:
  ret void
}

define void @BEQw(i32 "reg"="w" %n, i32 "reg"="w" %m, ptr "label" %dest) "enc"="b12:s1=dest>>12 b10:6=(dest>>5)&63 m:5 n:5 000 b4:4=(dest>>1)&15 b11:1=(dest>>11)&1 1100011" "asm"="beq {n}, {m}, {dest}" "inverse"="BNEw" "nodecode" {
  %c = icmp eq i32 %n, %m
  br i1 %c, label %taken, label %next
taken:
  indirectbr ptr %dest, []
next:
  ret void
}

define void @BNEw(i32 "reg"="w" %n, i32 "reg"="w" %m, ptr "label" %dest) "enc"="b12:s1=dest>>12 b10:6=(dest>>5)&63 m:5 n:5 001 b4:4=(dest>>1)&15 b11:1=(dest>>11)&1 1100011" "asm"="bne {n}, {m}, {dest}" "inverse"="BEQw" "nodecode" {
  %c = icmp ne i32 %n, %m
  br i1 %c, label %taken, label %next
taken:
  indirectbr ptr %dest, []
next:
  ret void
}

define void @BLTw(i32 "reg"="w" %n, i32 "reg"="w" %m, ptr "label" %dest) "enc"="b12:s1=dest>>12 b10:6=(dest>>5)&63 m:5 n:5 100 b4:4=(dest>>1)&15 b11:1=(dest>>11)&1 1100011" "asm"="blt {n}, {m}, {dest}" "inverse"="BGEw" "nodecode" {
  %c = icmp slt i32 %n, %m
  br i1 %c, label %taken, label %next
taken:
  indirectbr ptr %dest, []
next:
  ret void
}

define void @BGEw(i32 "reg"="w" %n, i32 "reg"="w" %m, ptr "label" %dest) "enc"="b12:s1=dest>>12 b10:6=(dest>>5)&63 m:5 n:5 101 b4:4=(dest>>1)&15 b11:1=(dest>>11)&1 1100011" "asm"="bge {n}, {m}, {dest}" "inverse"="BLTw" "nodecode" {
  %c = icmp sge i32 %n, %m
  br i1 %c, label %taken, label %next
taken:
  indirectbr ptr %dest, []
next:
  ret void
}

define void @BLTUw(i32 "reg"="w" %n, i32 "reg"="w" %m, ptr "label" %dest) "enc"="b12:s1=dest>>12 b10:6=(dest>>5)&63 m:5 n:5 110 b4:4=(dest>>1)&15 b11:1=(dest>>11)&1 1100011" "asm"="bltu {n}, {m}, {dest}" "inverse"="BGEUw" "nodecode" {
  %c = icmp ult i32 %n, %m
  br i1 %c, label %taken, label %next
taken:
  indirectbr ptr %dest, []
next:
  ret void
}

define void @BGEUw(i32 "reg"="w" %n, i32 "reg"="w" %m, ptr "label" %dest) "enc"="b12:s1=dest>>12 b10:6=(dest>>5)&63 m:5 n:5 111 b4:4=(dest>>1)&15 b11:1=(dest>>11)&1 1100011" "asm"="bgeu {n}, {m}, {dest}" "inverse"="BLTUw" "nodecode" {
  %c = icmp uge i32 %n, %m
  br i1 %c, label %taken, label %next
taken:
  indirectbr ptr %dest, []
next:
  ret void
}

define void @BGTw(i32 "reg"="w" %n, i32 "reg"="w" %m, ptr "label" %dest) "enc"="b12:s1=dest>>12 b10:6=(dest>>5)&63 rb:5=n ra:5=m 100 b4:4=(dest>>1)&15 b11:1=(dest>>11)&1 1100011" "asm"="blt {m}, {n}, {dest}" "inverse"="BLEw" "nodecode" {
  %c = icmp sgt i32 %n, %m
  br i1 %c, label %taken, label %next
taken:
  indirectbr ptr %dest, []
next:
  ret void
}

define void @BLEw(i32 "reg"="w" %n, i32 "reg"="w" %m, ptr "label" %dest) "enc"="b12:s1=dest>>12 b10:6=(dest>>5)&63 rb:5=n ra:5=m 101 b4:4=(dest>>1)&15 b11:1=(dest>>11)&1 1100011" "asm"="bge {m}, {n}, {dest}" "inverse"="BGTw" "nodecode" {
  %c = icmp sle i32 %n, %m
  br i1 %c, label %taken, label %next
taken:
  indirectbr ptr %dest, []
next:
  ret void
}

define void @BGTUw(i32 "reg"="w" %n, i32 "reg"="w" %m, ptr "label" %dest) "enc"="b12:s1=dest>>12 b10:6=(dest>>5)&63 rb:5=n ra:5=m 110 b4:4=(dest>>1)&15 b11:1=(dest>>11)&1 1100011" "asm"="bltu {m}, {n}, {dest}" "inverse"="BLEUw" "nodecode" {
  %c = icmp ugt i32 %n, %m
  br i1 %c, label %taken, label %next
taken:
  indirectbr ptr %dest, []
next:
  ret void
}

define void @BLEUw(i32 "reg"="w" %n, i32 "reg"="w" %m, ptr "label" %dest) "enc"="b12:s1=dest>>12 b10:6=(dest>>5)&63 rb:5=n ra:5=m 111 b4:4=(dest>>1)&15 b11:1=(dest>>11)&1 1100011" "asm"="bgeu {m}, {n}, {dest}" "inverse"="BGTUw" "nodecode" {
  %c = icmp ule i32 %n, %m
  br i1 %c, label %taken, label %next
taken:
  indirectbr ptr %dest, []
next:
  ret void
}

define void @BEQp(ptr "reg"="x" %n, ptr "reg"="x" %m, ptr "label" %dest) "enc"="b12:s1=dest>>12 b10:6=(dest>>5)&63 m:5 n:5 000 b4:4=(dest>>1)&15 b11:1=(dest>>11)&1 1100011" "asm"="beq {n}, {m}, {dest}" "inverse"="BNEp" "nodecode" {
  %c = icmp eq ptr %n, %m
  br i1 %c, label %taken, label %next
taken:
  indirectbr ptr %dest, []
next:
  ret void
}

define void @BNEp(ptr "reg"="x" %n, ptr "reg"="x" %m, ptr "label" %dest) "enc"="b12:s1=dest>>12 b10:6=(dest>>5)&63 m:5 n:5 001 b4:4=(dest>>1)&15 b11:1=(dest>>11)&1 1100011" "asm"="bne {n}, {m}, {dest}" "inverse"="BEQp" "nodecode" {
  %c = icmp ne ptr %n, %m
  br i1 %c, label %taken, label %next
taken:
  indirectbr ptr %dest, []
next:
  ret void
}

define void @BLTUp(ptr "reg"="x" %n, ptr "reg"="x" %m, ptr "label" %dest) "enc"="b12:s1=dest>>12 b10:6=(dest>>5)&63 m:5 n:5 110 b4:4=(dest>>1)&15 b11:1=(dest>>11)&1 1100011" "asm"="bltu {n}, {m}, {dest}" "inverse"="BGEUp" "nodecode" {
  %c = icmp ult ptr %n, %m
  br i1 %c, label %taken, label %next
taken:
  indirectbr ptr %dest, []
next:
  ret void
}

define void @BGEUp(ptr "reg"="x" %n, ptr "reg"="x" %m, ptr "label" %dest) "enc"="b12:s1=dest>>12 b10:6=(dest>>5)&63 m:5 n:5 111 b4:4=(dest>>1)&15 b11:1=(dest>>11)&1 1100011" "asm"="bgeu {n}, {m}, {dest}" "inverse"="BLTUp" "nodecode" {
  %c = icmp uge ptr %n, %m
  br i1 %c, label %taken, label %next
taken:
  indirectbr ptr %dest, []
next:
  ret void
}

define void @BGTUp(ptr "reg"="x" %n, ptr "reg"="x" %m, ptr "label" %dest) "enc"="b12:s1=dest>>12 b10:6=(dest>>5)&63 rb:5=n ra:5=m 110 b4:4=(dest>>1)&15 b11:1=(dest>>11)&1 1100011" "asm"="bltu {m}, {n}, {dest}" "inverse"="BLEUp" "nodecode" {
  %c = icmp ugt ptr %n, %m
  br i1 %c, label %taken, label %next
taken:
  indirectbr ptr %dest, []
next:
  ret void
}

define void @BLEUp(ptr "reg"="x" %n, ptr "reg"="x" %m, ptr "label" %dest) "enc"="b12:s1=dest>>12 b10:6=(dest>>5)&63 rb:5=n ra:5=m 111 b4:4=(dest>>1)&15 b11:1=(dest>>11)&1 1100011" "asm"="bgeu {m}, {n}, {dest}" "inverse"="BGTUp" "nodecode" {
  %c = icmp ule ptr %n, %m
  br i1 %c, label %taken, label %next
taken:
  indirectbr ptr %dest, []
next:
  ret void
}

define void @J(ptr "label" %dest) "enc"="j20:s1=dest>>20 j10:10=(dest>>1)&1023 j11:1=(dest>>11)&1 j19:8=(dest>>12)&255 00000 1101111" "asm"="j {dest}" "role"="jump" {
  indirectbr ptr %dest, []
}

define void @CALL(ptr "sym" %s) "enc"="j20:s1=s>>20 j10:10=(s>>1)&1023 j11:1=(s>>11)&1 j19:8=(s>>12)&255 00001 1101111" "asm"="jal ra, {s}" "reloc"="jal" "role"="call" "nomatch" {
  ret void
}

define void @TAIL(ptr "sym" %s) "enc"="j20:s1=s>>20 j10:10=(s>>1)&1023 j11:1=(s>>11)&1 j19:8=(s>>12)&255 00000 1101111" "asm"="jal zero, {s}" "reloc"="jal" "role"="tailcall" "nomatch" "nodecode" {
  ret void
}


define void @CALLR(ptr "reg"="x" %n) "enc"="000000000000 n:5 000 00001 1100111" "asm"="jalr ra, 0({n})" "role"="callr" "nomatch" {
  ret void
}

define void @RET() "enc"="00000000000000001000000001100111" "asm"="ret" "role"="ret" {
  ret void
}

define void @EBREAK() "enc"="00000000000100000000000001110011" "asm"="ebreak" "role"="trap" {
  unreachable
}
