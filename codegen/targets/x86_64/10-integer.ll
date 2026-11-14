; x86-64's integer instructions for selection. Written by gen_sel.py; do
; not edit. The conventions are those of codegen/targets/aarch64/00-base.ll,
; and docs/design/x86.md has the plan.

define i64 @ADD64rr(i64 "reg"="r64" "tied" %n, i64 "reg"="r64" %m) "enc"="rex(1, m, 0, d) | 00000001 | rr(m, d)" "asm"="addq {m}, {d}" "clobbers"="flags" "role"="add2.r64" "nodecode" {
  %r = add i64 %n, %m
  ret i64 %r
}

define i64 @ADD64ri(i64 "reg"="r64" "tied" %n, i64 "imm" %imm) "enc"="{ rex(1, 0, 0, d) | 10000011 | rr(0, d) | ib:s8=imm ; acc:0=d rex(1, 0, 0, 0) | 00000101 | iv:s32=imm ; rex(1, 0, 0, d) | 10000001 | rr(0, d) | iv:s32=imm }" "asm"="addq ${imm}, {d}" "clobbers"="flags" "role"="add2i.r64" "nodecode" {
  %r = add i64 %n, %imm
  ret i64 %r
}

define i64 @OR64rr(i64 "reg"="r64" "tied" %n, i64 "reg"="r64" %m) "enc"="rex(1, m, 0, d) | 00001001 | rr(m, d)" "asm"="orq {m}, {d}" "clobbers"="flags" "nodecode" {
  %r = or i64 %n, %m
  ret i64 %r
}

define i64 @OR64ri(i64 "reg"="r64" "tied" %n, i64 "imm" %imm) "enc"="{ rex(1, 0, 0, d) | 10000011 | rr(1, d) | ib:s8=imm ; acc:0=d rex(1, 0, 0, 0) | 00001101 | iv:s32=imm ; rex(1, 0, 0, d) | 10000001 | rr(1, d) | iv:s32=imm }" "asm"="orq ${imm}, {d}" "clobbers"="flags" "nodecode" {
  %r = or i64 %n, %imm
  ret i64 %r
}

define i64 @AND64rr(i64 "reg"="r64" "tied" %n, i64 "reg"="r64" %m) "enc"="rex(1, m, 0, d) | 00100001 | rr(m, d)" "asm"="andq {m}, {d}" "clobbers"="flags" "nodecode" {
  %r = and i64 %n, %m
  ret i64 %r
}

define i64 @AND64ri(i64 "reg"="r64" "tied" %n, i64 "imm" %imm) "enc"="{ rex(1, 0, 0, d) | 10000011 | rr(4, d) | ib:s8=imm ; acc:0=d rex(1, 0, 0, 0) | 00100101 | iv:s32=imm ; rex(1, 0, 0, d) | 10000001 | rr(4, d) | iv:s32=imm }" "asm"="andq ${imm}, {d}" "clobbers"="flags" "nodecode" {
  %r = and i64 %n, %imm
  ret i64 %r
}

define i64 @SUB64rr(i64 "reg"="r64" "tied" %n, i64 "reg"="r64" %m) "enc"="rex(1, m, 0, d) | 00101001 | rr(m, d)" "asm"="subq {m}, {d}" "clobbers"="flags" "role"="sub" "nodecode" {
  %r = sub i64 %n, %m
  ret i64 %r
}

define i64 @SUB64ri(i64 "reg"="r64" "tied" %n, i64 "imm" %imm) "enc"="{ rex(1, 0, 0, d) | 10000011 | rr(5, d) | ib:s8=imm ; acc:0=d rex(1, 0, 0, 0) | 00101101 | iv:s32=imm ; rex(1, 0, 0, d) | 10000001 | rr(5, d) | iv:s32=imm }" "asm"="subq ${imm}, {d}" "clobbers"="flags" "nodecode" {
  %r = sub i64 %n, %imm
  ret i64 %r
}

define i64 @XOR64rr(i64 "reg"="r64" "tied" %n, i64 "reg"="r64" %m) "enc"="rex(1, m, 0, d) | 00110001 | rr(m, d)" "asm"="xorq {m}, {d}" "clobbers"="flags" "nodecode" {
  %r = xor i64 %n, %m
  ret i64 %r
}

define i64 @XOR64ri(i64 "reg"="r64" "tied" %n, i64 "imm" %imm) "enc"="{ rex(1, 0, 0, d) | 10000011 | rr(6, d) | ib:s8=imm ; acc:0=d rex(1, 0, 0, 0) | 00110101 | iv:s32=imm ; rex(1, 0, 0, d) | 10000001 | rr(6, d) | iv:s32=imm }" "asm"="xorq ${imm}, {d}" "clobbers"="flags" "nodecode" {
  %r = xor i64 %n, %imm
  ret i64 %r
}

define i64 @IMUL64rr(i64 "reg"="r64" "tied" %n, i64 "reg"="r64" %m) "enc"="rex(1, d, 0, m) | 00001111 | 10101111 | rr(d, m)" "asm"="imulq {m}, {d}" "clobbers"="flags" "nodecode" {
  %r = mul i64 %n, %m
  ret i64 %r
}

define i64 @IMUL64rri(i64 "reg"="r64" %n, i64 "imm" %imm) "enc"="{ rex(1, d, 0, n) | 01101011 | rr(d, n) | ib:s8=imm ; rex(1, d, 0, n) | 01101001 | rr(d, n) | iv:s32=imm }" "asm"="imulq ${imm}, {n}, {d}" "clobbers"="flags" "nodecode" {
  %r = mul i64 %n, %imm
  ret i64 %r
}

define i64 @SHL64ri(i64 "reg"="r64" "tied" %n, i64 "imm" %imm) "enc"="{ one:0=imm-1 rex(1, 0, 0, d) | 11010001 | rr(4, d) ; rex(1, 0, 0, d) | 11000001 | rr(4, d) | ib:8=imm }" "asm"="shlq ${imm}, {d}" "clobbers"="flags" "nodecode" {
  %r = shl i64 %n, %imm
  ret i64 %r
}

define i64 @SHL64rCL(i64 "reg"="r64" "tied" %n, i64 "reg"="rcx" %c) "enc"="rex(1, 0, 0, d) | 11010011 | rr(4, d)" "asm"="shlq %cl, {d}" "clobbers"="flags" "nodecode" {
  %r = shl i64 %n, %c
  ret i64 %r
}

define i64 @SHR64ri(i64 "reg"="r64" "tied" %n, i64 "imm" %imm) "enc"="{ one:0=imm-1 rex(1, 0, 0, d) | 11010001 | rr(5, d) ; rex(1, 0, 0, d) | 11000001 | rr(5, d) | ib:8=imm }" "asm"="shrq ${imm}, {d}" "clobbers"="flags" "nodecode" {
  %r = lshr i64 %n, %imm
  ret i64 %r
}

define i64 @SHR64rCL(i64 "reg"="r64" "tied" %n, i64 "reg"="rcx" %c) "enc"="rex(1, 0, 0, d) | 11010011 | rr(5, d)" "asm"="shrq %cl, {d}" "clobbers"="flags" "nodecode" {
  %r = lshr i64 %n, %c
  ret i64 %r
}

define i64 @SAR64ri(i64 "reg"="r64" "tied" %n, i64 "imm" %imm) "enc"="{ one:0=imm-1 rex(1, 0, 0, d) | 11010001 | rr(7, d) ; rex(1, 0, 0, d) | 11000001 | rr(7, d) | ib:8=imm }" "asm"="sarq ${imm}, {d}" "clobbers"="flags" "nodecode" {
  %r = ashr i64 %n, %imm
  ret i64 %r
}

define i64 @SAR64rCL(i64 "reg"="r64" "tied" %n, i64 "reg"="rcx" %c) "enc"="rex(1, 0, 0, d) | 11010011 | rr(7, d)" "asm"="sarq %cl, {d}" "clobbers"="flags" "nodecode" {
  %r = ashr i64 %n, %c
  ret i64 %r
}

define i64 @NEG64r(i64 "reg"="r64" "tied" %n) "enc"="rex(1, 0, 0, d) | 11110111 | rr(3, d)" "asm"="negq {d}" "clobbers"="flags" "nodecode" {
  %r = sub i64 0, %n
  ret i64 %r
}

define i64 @NOT64r(i64 "reg"="r64" "tied" %n) "enc"="rex(1, 0, 0, d) | 11110111 | rr(2, d)" "asm"="notq {d}" "nodecode" {
  %r = xor i64 %n, -1
  ret i64 %r
}

define i32 @ADD32rr(i32 "reg"="r32" "tied" %n, i32 "reg"="r32" %m) "enc"="rex(0, m, 0, d) | 00000001 | rr(m, d)" "asm"="addl {m}, {d}" "clobbers"="flags" "zext"="32" "nodecode" {
  %r = add i32 %n, %m
  ret i32 %r
}

define i32 @ADD32ri(i32 "reg"="r32" "tied" %n, i32 "imm" %imm) "enc"="{ rex(0, 0, 0, d) | 10000011 | rr(0, d) | ib:s8=imm ; acc:0=d rex(0, 0, 0, 0) | 00000101 | iv:s32=imm ; rex(0, 0, 0, d) | 10000001 | rr(0, d) | iv:s32=imm }" "asm"="addl ${imm}, {d}" "clobbers"="flags" "zext"="32" "nodecode" {
  %r = add i32 %n, %imm
  ret i32 %r
}

define i32 @OR32rr(i32 "reg"="r32" "tied" %n, i32 "reg"="r32" %m) "enc"="rex(0, m, 0, d) | 00001001 | rr(m, d)" "asm"="orl {m}, {d}" "clobbers"="flags" "zext"="32" "nodecode" {
  %r = or i32 %n, %m
  ret i32 %r
}

define i32 @OR32ri(i32 "reg"="r32" "tied" %n, i32 "imm" %imm) "enc"="{ rex(0, 0, 0, d) | 10000011 | rr(1, d) | ib:s8=imm ; acc:0=d rex(0, 0, 0, 0) | 00001101 | iv:s32=imm ; rex(0, 0, 0, d) | 10000001 | rr(1, d) | iv:s32=imm }" "asm"="orl ${imm}, {d}" "clobbers"="flags" "zext"="32" "nodecode" {
  %r = or i32 %n, %imm
  ret i32 %r
}

define i32 @AND32rr(i32 "reg"="r32" "tied" %n, i32 "reg"="r32" %m) "enc"="rex(0, m, 0, d) | 00100001 | rr(m, d)" "asm"="andl {m}, {d}" "clobbers"="flags" "zext"="32" "nodecode" {
  %r = and i32 %n, %m
  ret i32 %r
}

define i32 @AND32ri(i32 "reg"="r32" "tied" %n, i32 "imm" %imm) "enc"="{ rex(0, 0, 0, d) | 10000011 | rr(4, d) | ib:s8=imm ; acc:0=d rex(0, 0, 0, 0) | 00100101 | iv:s32=imm ; rex(0, 0, 0, d) | 10000001 | rr(4, d) | iv:s32=imm }" "asm"="andl ${imm}, {d}" "clobbers"="flags" "zext"="32" "nodecode" {
  %r = and i32 %n, %imm
  ret i32 %r
}

define i32 @SUB32rr(i32 "reg"="r32" "tied" %n, i32 "reg"="r32" %m) "enc"="rex(0, m, 0, d) | 00101001 | rr(m, d)" "asm"="subl {m}, {d}" "clobbers"="flags" "zext"="32" "nodecode" {
  %r = sub i32 %n, %m
  ret i32 %r
}

define i32 @SUB32ri(i32 "reg"="r32" "tied" %n, i32 "imm" %imm) "enc"="{ rex(0, 0, 0, d) | 10000011 | rr(5, d) | ib:s8=imm ; acc:0=d rex(0, 0, 0, 0) | 00101101 | iv:s32=imm ; rex(0, 0, 0, d) | 10000001 | rr(5, d) | iv:s32=imm }" "asm"="subl ${imm}, {d}" "clobbers"="flags" "zext"="32" "nodecode" {
  %r = sub i32 %n, %imm
  ret i32 %r
}

define i32 @XOR32rr(i32 "reg"="r32" "tied" %n, i32 "reg"="r32" %m) "enc"="rex(0, m, 0, d) | 00110001 | rr(m, d)" "asm"="xorl {m}, {d}" "clobbers"="flags" "zext"="32" "nodecode" {
  %r = xor i32 %n, %m
  ret i32 %r
}

define i32 @XOR32ri(i32 "reg"="r32" "tied" %n, i32 "imm" %imm) "enc"="{ rex(0, 0, 0, d) | 10000011 | rr(6, d) | ib:s8=imm ; acc:0=d rex(0, 0, 0, 0) | 00110101 | iv:s32=imm ; rex(0, 0, 0, d) | 10000001 | rr(6, d) | iv:s32=imm }" "asm"="xorl ${imm}, {d}" "clobbers"="flags" "zext"="32" "nodecode" {
  %r = xor i32 %n, %imm
  ret i32 %r
}

define i32 @IMUL32rr(i32 "reg"="r32" "tied" %n, i32 "reg"="r32" %m) "enc"="rex(0, d, 0, m) | 00001111 | 10101111 | rr(d, m)" "asm"="imull {m}, {d}" "clobbers"="flags" "zext"="32" "nodecode" {
  %r = mul i32 %n, %m
  ret i32 %r
}

define i32 @IMUL32rri(i32 "reg"="r32" %n, i32 "imm" %imm) "enc"="{ rex(0, d, 0, n) | 01101011 | rr(d, n) | ib:s8=imm ; rex(0, d, 0, n) | 01101001 | rr(d, n) | iv:s32=imm }" "asm"="imull ${imm}, {n}, {d}" "clobbers"="flags" "zext"="32" "nodecode" {
  %r = mul i32 %n, %imm
  ret i32 %r
}

define i32 @SHL32ri(i32 "reg"="r32" "tied" %n, i32 "imm" %imm) "enc"="{ one:0=imm-1 rex(0, 0, 0, d) | 11010001 | rr(4, d) ; rex(0, 0, 0, d) | 11000001 | rr(4, d) | ib:8=imm }" "asm"="shll ${imm}, {d}" "clobbers"="flags" "zext"="32" "nodecode" {
  %r = shl i32 %n, %imm
  ret i32 %r
}

define i32 @SHL32rCL(i32 "reg"="r32" "tied" %n, i32 "reg"="rcx" %c) "enc"="rex(0, 0, 0, d) | 11010011 | rr(4, d)" "asm"="shll %cl, {d}" "clobbers"="flags" "zext"="32" "nodecode" {
  %r = shl i32 %n, %c
  ret i32 %r
}

define i32 @SHR32ri(i32 "reg"="r32" "tied" %n, i32 "imm" %imm) "enc"="{ one:0=imm-1 rex(0, 0, 0, d) | 11010001 | rr(5, d) ; rex(0, 0, 0, d) | 11000001 | rr(5, d) | ib:8=imm }" "asm"="shrl ${imm}, {d}" "clobbers"="flags" "zext"="32" "nodecode" {
  %r = lshr i32 %n, %imm
  ret i32 %r
}

define i32 @SHR32rCL(i32 "reg"="r32" "tied" %n, i32 "reg"="rcx" %c) "enc"="rex(0, 0, 0, d) | 11010011 | rr(5, d)" "asm"="shrl %cl, {d}" "clobbers"="flags" "zext"="32" "nodecode" {
  %r = lshr i32 %n, %c
  ret i32 %r
}

define i32 @SAR32ri(i32 "reg"="r32" "tied" %n, i32 "imm" %imm) "enc"="{ one:0=imm-1 rex(0, 0, 0, d) | 11010001 | rr(7, d) ; rex(0, 0, 0, d) | 11000001 | rr(7, d) | ib:8=imm }" "asm"="sarl ${imm}, {d}" "clobbers"="flags" "zext"="32" "nodecode" {
  %r = ashr i32 %n, %imm
  ret i32 %r
}

define i32 @SAR32rCL(i32 "reg"="r32" "tied" %n, i32 "reg"="rcx" %c) "enc"="rex(0, 0, 0, d) | 11010011 | rr(7, d)" "asm"="sarl %cl, {d}" "clobbers"="flags" "zext"="32" "nodecode" {
  %r = ashr i32 %n, %c
  ret i32 %r
}

define i32 @NEG32r(i32 "reg"="r32" "tied" %n) "enc"="rex(0, 0, 0, d) | 11110111 | rr(3, d)" "asm"="negl {d}" "clobbers"="flags" "zext"="32" "nodecode" {
  %r = sub i32 0, %n
  ret i32 %r
}

define i32 @NOT32r(i32 "reg"="r32" "tied" %n) "enc"="rex(0, 0, 0, d) | 11110111 | rr(2, d)" "asm"="notl {d}" "zext"="32" "nodecode" {
  %r = xor i32 %n, -1
  ret i32 %r
}

define i64 @MOV64rr(i64 "reg"="r64" %n) "enc"="rex(1, n, 0, d) | 10001001 | rr(n, d)" "asm"="movq {n}, {d}" "copy"="r64" "nodecode" {
  ret i64 %n
}

define i32 @MOV32rr(i32 "reg"="r32" %n) "enc"="rex(0, n, 0, d) | 10001001 | rr(n, d)" "asm"="movl {n}, {d}" "copy"="r32" "zext"="32" "nodecode" {
  ret i32 %n
}

define i32 @MOV32ri(i32 "imm" %imm) "enc"="{ rex(0, 0, 0, d) | 10111 o:3=d&7 | iv:s32=imm ; rex(0, 0, 0, d) | 10111 o:3=d&7 | iv:32=imm }" "asm"="movl ${imm}, {d}" "zext"="32" "nodecode" {
  ret i32 %imm
}

define i64 @MOV64ri32(i64 "imm" %imm) "enc"="rex(0, 0, 0, d) | 10111 o:3=d&7 | iv:32=imm" "asm"="movl ${imm}, {d:r32}" "nodecode" {
  ret i64 %imm
}

define i64 @MOV64ri(i64 "imm" %imm) "enc"="rex(1, 0, 0, d) | 11000111 | rr(0, d) | iv:s32=imm" "asm"="movq ${imm}, {d}" "nodecode" {
  ret i64 %imm
}

define i64 @MOVABS64ri(i64 "imm" %imm) "enc"="rex(1, 0, 0, d) | 10111 o:3=d&7 | io:s64=imm" "asm"="movabsq ${imm}, {d}" "nodecode" {
  ret i64 %imm
}

define i32 @MOVZBL32(i8 "reg"="r32" %n) "enc"="{ if(n<4) rex(0, d, 0, n) ; 0100 W:1=0 R:1=(d)>>3 X:1=0 B:1=(n)>>3 } | 00001111 | 10110110 | rr(d, n)" "asm"="movzbl {n:r8}, {d:r32}" "zext"="8" "role"="zext.8" "nodecode" {
  %r = zext i8 %n to i32
  ret i32 %r
}

define i32 @MOVZWL32(i16 "reg"="r32" %n) "enc"="rex(0, d, 0, n) | 00001111 | 10110111 | rr(d, n)" "asm"="movzwl {n:r16}, {d:r32}" "zext"="16" "role"="zext.16" "nodecode" {
  %r = zext i16 %n to i32
  ret i32 %r
}

define i32 @MOVSBL32(i8 "reg"="r32" %n) "enc"="{ if(n<4) rex(0, d, 0, n) ; 0100 W:1=0 R:1=(d)>>3 X:1=0 B:1=(n)>>3 } | 00001111 | 10111110 | rr(d, n)" "asm"="movsbl {n:r8}, {d:r32}" "role"="sext.8" "nodecode" {
  %r = sext i8 %n to i32
  ret i32 %r
}

define i32 @MOVSWL32(i16 "reg"="r32" %n) "enc"="rex(0, d, 0, n) | 00001111 | 10111111 | rr(d, n)" "asm"="movswl {n:r16}, {d:r32}" "role"="sext.16" "nodecode" {
  %r = sext i16 %n to i32
  ret i32 %r
}

define i64 @MOVZBL64(i8 "reg"="r32" %n) "enc"="{ if(n<4) rex(0, d, 0, n) ; 0100 W:1=0 R:1=(d)>>3 X:1=0 B:1=(n)>>3 } | 00001111 | 10110110 | rr(d, n)" "asm"="movzbl {n:r8}, {d:r32}" "zext"="8" "nodecode" {
  %r = zext i8 %n to i64
  ret i64 %r
}

define i64 @MOVZWL64(i16 "reg"="r32" %n) "enc"="rex(0, d, 0, n) | 00001111 | 10110111 | rr(d, n)" "asm"="movzwl {n:r16}, {d:r32}" "zext"="16" "nodecode" {
  %r = zext i16 %n to i64
  ret i64 %r
}

define i64 @MOVSBQ64(i8 "reg"="r32" %n) "enc"="rex(1, d, 0, n) | 00001111 | 10111110 | rr(d, n)" "asm"="movsbq {n:r8}, {d}" "nodecode" {
  %r = sext i8 %n to i64
  ret i64 %r
}

define i64 @MOVSWQ64(i16 "reg"="r32" %n) "enc"="rex(1, d, 0, n) | 00001111 | 10111111 | rr(d, n)" "asm"="movswq {n:r16}, {d}" "nodecode" {
  %r = sext i16 %n to i64
  ret i64 %r
}

define i32 @ZEXT1(i1 "reg"="r32" "tied" %n) "enc"="rex(0, 0, 0, d) | 10000011 | rr(4, d) | 00000001" "asm"="andl $1, {d}" "zext"="1" "role"="zext.1" "clobbers"="flags" "nodecode" {
  %r = zext i1 %n to i32
  ret i32 %r
}

define i64 @ZEXT1q(i1 "reg"="r32" "tied" %n) "enc"="rex(0, 0, 0, d) | 10000011 | rr(4, d) | 00000001" "asm"="andl $1, {d:r32}" "zext"="1" "dclass"="r64" "clobbers"="flags" "nodecode" {
  %r = zext i1 %n to i64
  ret i64 %r
}

define i64 @MOVSLQ(i32 "reg"="r32" %n) "enc"="rex(1, d, 0, n) | 01100011 | rr(d, n)" "asm"="movslq {n}, {d}" "role"="sext.32" "nodecode" {
  %r = sext i32 %n to i64
  ret i64 %r
}

define ptr @LEA64ri(ptr "reg"="r64" %n, i64 "imm" %imm) "enc"="rex(1, d, 4, n) | 10001101 | rm(d, n, 4, 1, imm, 0, 1)" "asm"="leaq {imm}({n}), {d}" "role"="addi" "nodecode" {
  %r = getelementptr i8, ptr %n, i64 %imm
  ret ptr %r
}

define ptr @LEA64rr(ptr "reg"="r64" %n, i64 "reg"="r64" %m) "enc"="{ if(m!=4) rex(1, d, m, n) | 10001101 | rm(d, n, m, 1, 0, 0, 1) }" "asm"="leaq ({n},{m}), {d}" "role"="add" "nodecode" {
  %r = getelementptr i8, ptr %n, i64 %m
  ret ptr %r
}

define i64 @LEA64add(i64 "reg"="r64" %n, i64 "reg"="r64" %m) "enc"="{ if(m!=4) rex(1, d, m, n) | 10001101 | rm(d, n, m, 1, 0, 0, 1) }" "asm"="leaq ({n},{m}), {d}" "nodecode" {
  %r = add i64 %n, %m
  ret i64 %r
}

define i64 @LEA64addi(i64 "reg"="r64" %n, i64 "imm" %imm) "enc"="rex(1, d, 4, n) | 10001101 | rm(d, n, 4, 1, imm, 0, 1)" "asm"="leaq {imm}({n}), {d}" "nodecode" {
  %r = add i64 %n, %imm
  ret i64 %r
}

define ptr @LEArip(ptr "sym" %s) "enc"="rex(1, d, 0, 0) | 10001101 | rm(d, 16, 4, 1, 0, s, 1)" "asm"="leaq {s}(%rip), {d}" "reloc"="pc32x" "role"="addr" "nomatch" "nodecode" {
  ret ptr %s
}

define ptr @MOVgot(ptr "sym" %s) "enc"="rex(1, d, 0, 0) | 10001011 | rm(d, 16, 4, 1, 0, s, 1)" "asm"="movq {s}(%rip), {d}" "reloc"="gotpcrel" "role"="got" "nomatch" "nodecode" {
  ret ptr %s
}

define i64 @MOVgottpoff(ptr "sym" %s) "enc"="rex(1, d, 0, 0) | 10001011 | rm(d, 16, 4, 1, 0, s, 1)" "asm"="movq {s}(%rip), {d}" "reloc"="gottpoff" "role"="tls.ie" "nomatch" "nodecode" {
  ret i64 0
}

define ptr @MOVfs0() "enc"="01100100 | rex(1, d, 0, 0) | 10001011 | 00 r:3=d&7 100 | 00100101 | 00000000 | 00000000 | 00000000 | 00000000" "asm"="movq %fs:0, {d}" "role"="thread.pointer" {
  %p = call ptr @llvm.thread.pointer.p0()
  ret ptr %p
}

declare ptr @llvm.thread.pointer.p0()

define i64 @LDI64(ptr "reg"="r64" %n, i64 "imm" %off) "enc"="rex(1, d, 4, n) | 10001011 | rm(d, n, 4, 1, off, 0, 1)" "asm"="movq {off}({n}), {d}" "role"="load.x,load.r64" "dclass"="r64" "nodecode" {
  %a = getelementptr i8, ptr %n, i64 %off
  %r = load i64, ptr %a
  ret i64 %r
}

define i32 @LDI32(ptr "reg"="r64" %n, i64 "imm" %off) "enc"="rex(0, d, 4, n) | 10001011 | rm(d, n, 4, 1, off, 0, 1)" "asm"="movl {off}({n}), {d}" "role"="load.w" "zext"="32" "dclass"="r32" "nodecode" {
  %a = getelementptr i8, ptr %n, i64 %off
  %r = load i32, ptr %a
  ret i32 %r
}

define i8 @LDI8(ptr "reg"="r64" %n, i64 "imm" %off) "enc"="rex(0, d, 4, n) | 00001111 | 10110110 | rm(d, n, 4, 1, off, 0, 1)" "asm"="movzbl {off}({n}), {d:r32}" "role"="load.b" "zext"="8" "dclass"="r32" "nodecode" {
  %a = getelementptr i8, ptr %n, i64 %off
  %r = load i8, ptr %a
  ret i8 %r
}

define i16 @LDI16(ptr "reg"="r64" %n, i64 "imm" %off) "enc"="rex(0, d, 4, n) | 00001111 | 10110111 | rm(d, n, 4, 1, off, 0, 1)" "asm"="movzwl {off}({n}), {d:r32}" "role"="load.h" "zext"="16" "dclass"="r32" "nodecode" {
  %a = getelementptr i8, ptr %n, i64 %off
  %r = load i16, ptr %a
  ret i16 %r
}

define ptr @LDP(ptr "reg"="r64" %n, i64 "imm" %off) "enc"="rex(1, d, 4, n) | 10001011 | rm(d, n, 4, 1, off, 0, 1)" "asm"="movq {off}({n}), {d}" "nodecode" {
  %a = getelementptr i8, ptr %n, i64 %off
  %r = load ptr, ptr %a
  ret ptr %r
}

define i32 @LDI8S32(ptr "reg"="r64" %n, i64 "imm" %off) "enc"="rex(0, d, 4, n) | 00001111 | 10111110 | rm(d, n, 4, 1, off, 0, 1)" "asm"="movsbl {off}({n}), {d}" "nodecode" {
  %a = getelementptr i8, ptr %n, i64 %off
  %v = load i8, ptr %a
  %r = sext i8 %v to i32
  ret i32 %r
}

define i32 @LDI16S32(ptr "reg"="r64" %n, i64 "imm" %off) "enc"="rex(0, d, 4, n) | 00001111 | 10111111 | rm(d, n, 4, 1, off, 0, 1)" "asm"="movswl {off}({n}), {d}" "nodecode" {
  %a = getelementptr i8, ptr %n, i64 %off
  %v = load i16, ptr %a
  %r = sext i16 %v to i32
  ret i32 %r
}

define i64 @LDI8S64(ptr "reg"="r64" %n, i64 "imm" %off) "enc"="rex(1, d, 4, n) | 00001111 | 10111110 | rm(d, n, 4, 1, off, 0, 1)" "asm"="movsbq {off}({n}), {d}" "nodecode" {
  %a = getelementptr i8, ptr %n, i64 %off
  %v = load i8, ptr %a
  %r = sext i8 %v to i64
  ret i64 %r
}

define i64 @LDI16S64(ptr "reg"="r64" %n, i64 "imm" %off) "enc"="rex(1, d, 4, n) | 00001111 | 10111111 | rm(d, n, 4, 1, off, 0, 1)" "asm"="movswq {off}({n}), {d}" "nodecode" {
  %a = getelementptr i8, ptr %n, i64 %off
  %v = load i16, ptr %a
  %r = sext i16 %v to i64
  ret i64 %r
}

define i64 @LDI32S64(ptr "reg"="r64" %n, i64 "imm" %off) "enc"="rex(1, d, 4, n) | 01100011 | rm(d, n, 4, 1, off, 0, 1)" "asm"="movslq {off}({n}), {d}" "nodecode" {
  %a = getelementptr i8, ptr %n, i64 %off
  %v = load i32, ptr %a
  %r = sext i32 %v to i64
  ret i64 %r
}

define void @STI64(i64 "reg"="r64" %t, ptr "reg"="r64" %n, i64 "imm" %off) "enc"="rex(1, t, 4, n) | 10001001 | rm(t, n, 4, 1, off, 0, 1)" "asm"="movq {t}, {off}({n})" "role"="store.x,store.r64" "nodecode" {
  %a = getelementptr i8, ptr %n, i64 %off
  store i64 %t, ptr %a
  ret void
}

define void @STI32(i32 "reg"="r32" %t, ptr "reg"="r64" %n, i64 "imm" %off) "enc"="rex(0, t, 4, n) | 10001001 | rm(t, n, 4, 1, off, 0, 1)" "asm"="movl {t}, {off}({n})" "role"="store.w" "nodecode" {
  %a = getelementptr i8, ptr %n, i64 %off
  store i32 %t, ptr %a
  ret void
}

define void @STI16(i16 "reg"="r32" %t, ptr "reg"="r64" %n, i64 "imm" %off) "enc"="01100110 | rex(0, t, 4, n) | 10001001 | rm(t, n, 4, 1, off, 0, 1)" "asm"="movw {t:r16}, {off}({n})" "role"="store.h" "nodecode" {
  %a = getelementptr i8, ptr %n, i64 %off
  store i16 %t, ptr %a
  ret void
}

define void @STI8(i8 "reg"="r32" %t, ptr "reg"="r64" %n, i64 "imm" %off) "enc"="{ if(t<4) rex(0, t, 4, n) ; 0100 0 R:1=(t)>>3 X:1=(4)>>3 B:1=(n)>>3 } | 10001000 | rm(t, n, 4, 1, off, 0, 1)" "asm"="movb {t:r8}, {off}({n})" "role"="store.b" "nodecode" {
  %a = getelementptr i8, ptr %n, i64 %off
  store i8 %t, ptr %a
  ret void
}

define void @STP(ptr "reg"="r64" %t, ptr "reg"="r64" %n, i64 "imm" %off) "enc"="rex(1, t, 4, n) | 10001001 | rm(t, n, 4, 1, off, 0, 1)" "asm"="movq {t}, {off}({n})" "nodecode" {
  %a = getelementptr i8, ptr %n, i64 %off
  store ptr %t, ptr %a
  ret void
}

define i64 @LDI64X1(ptr "reg"="r64" %n, i64 "reg"="r64" %m.index) "enc"="rex(1, d, m.index, n) | 10001011 | rm(d, n, m.index, 1, 0, 0, 1)" "asm"="movq ({n},{m.index},1), {d}" "dclass"="r64" "nodecode" {
  %a = getelementptr i8, ptr %n, i64 %m.index
  %r = load i64, ptr %a
  ret i64 %r
}

define i32 @LDI32X1(ptr "reg"="r64" %n, i64 "reg"="r64" %m.index) "enc"="rex(0, d, m.index, n) | 10001011 | rm(d, n, m.index, 1, 0, 0, 1)" "asm"="movl ({n},{m.index},1), {d}" "dclass"="r32" "zext"="32" "nodecode" {
  %a = getelementptr i8, ptr %n, i64 %m.index
  %r = load i32, ptr %a
  ret i32 %r
}

define i8 @LDI8X1(ptr "reg"="r64" %n, i64 "reg"="r64" %m.index) "enc"="rex(0, d, m.index, n) | 00001111 | 10110110 | rm(d, n, m.index, 1, 0, 0, 1)" "asm"="movzbl ({n},{m.index},1), {d:r32}" "dclass"="r32" "zext"="8" "nodecode" {
  %a = getelementptr i8, ptr %n, i64 %m.index
  %r = load i8, ptr %a
  ret i8 %r
}

define i16 @LDI16X1(ptr "reg"="r64" %n, i64 "reg"="r64" %m.index) "enc"="rex(0, d, m.index, n) | 00001111 | 10110111 | rm(d, n, m.index, 1, 0, 0, 1)" "asm"="movzwl ({n},{m.index},1), {d:r32}" "dclass"="r32" "zext"="16" "nodecode" {
  %a = getelementptr i8, ptr %n, i64 %m.index
  %r = load i16, ptr %a
  ret i16 %r
}

define ptr @LDPTRX1(ptr "reg"="r64" %n, i64 "reg"="r64" %m.index) "enc"="rex(1, d, m.index, n) | 10001011 | rm(d, n, m.index, 1, 0, 0, 1)" "asm"="movq ({n},{m.index},1), {d}" "dclass"="r64" "nodecode" {
  %a = getelementptr i8, ptr %n, i64 %m.index
  %r = load ptr, ptr %a
  ret ptr %r
}

define i32 @LDI8S32X1(ptr "reg"="r64" %n, i64 "reg"="r64" %m.index) "enc"="rex(0, d, m.index, n) | 00001111 | 10111110 | rm(d, n, m.index, 1, 0, 0, 1)" "asm"="movsbl ({n},{m.index},1), {d}" "nodecode" {
  %a = getelementptr i8, ptr %n, i64 %m.index
  %v = load i8, ptr %a
  %r = sext i8 %v to i32
  ret i32 %r
}

define i32 @LDI16S32X1(ptr "reg"="r64" %n, i64 "reg"="r64" %m.index) "enc"="rex(0, d, m.index, n) | 00001111 | 10111111 | rm(d, n, m.index, 1, 0, 0, 1)" "asm"="movswl ({n},{m.index},1), {d}" "nodecode" {
  %a = getelementptr i8, ptr %n, i64 %m.index
  %v = load i16, ptr %a
  %r = sext i16 %v to i32
  ret i32 %r
}

define i64 @LDI8S64X1(ptr "reg"="r64" %n, i64 "reg"="r64" %m.index) "enc"="rex(1, d, m.index, n) | 00001111 | 10111110 | rm(d, n, m.index, 1, 0, 0, 1)" "asm"="movsbq ({n},{m.index},1), {d}" "nodecode" {
  %a = getelementptr i8, ptr %n, i64 %m.index
  %v = load i8, ptr %a
  %r = sext i8 %v to i64
  ret i64 %r
}

define i64 @LDI16S64X1(ptr "reg"="r64" %n, i64 "reg"="r64" %m.index) "enc"="rex(1, d, m.index, n) | 00001111 | 10111111 | rm(d, n, m.index, 1, 0, 0, 1)" "asm"="movswq ({n},{m.index},1), {d}" "nodecode" {
  %a = getelementptr i8, ptr %n, i64 %m.index
  %v = load i16, ptr %a
  %r = sext i16 %v to i64
  ret i64 %r
}

define i64 @LDI32S64X1(ptr "reg"="r64" %n, i64 "reg"="r64" %m.index) "enc"="rex(1, d, m.index, n) | 01100011 | rm(d, n, m.index, 1, 0, 0, 1)" "asm"="movslq ({n},{m.index},1), {d}" "nodecode" {
  %a = getelementptr i8, ptr %n, i64 %m.index
  %v = load i32, ptr %a
  %r = sext i32 %v to i64
  ret i64 %r
}

define double @LDDX1(ptr "reg"="r64" %n, i64 "reg"="r64" %m.index) "enc"="11110010 | rex(0, d, m.index, n) | 00001111 | 00010000 | rm(d, n, m.index, 1, 0, 0, 1)" "asm"="movsd ({n},{m.index},1), {d}" "dclass"="d" "nodecode" {
  %a = getelementptr i8, ptr %n, i64 %m.index
  %r = load double, ptr %a
  ret double %r
}

define float @LDSX1(ptr "reg"="r64" %n, i64 "reg"="r64" %m.index) "enc"="11110011 | rex(0, d, m.index, n) | 00001111 | 00010000 | rm(d, n, m.index, 1, 0, 0, 1)" "asm"="movss ({n},{m.index},1), {d}" "dclass"="s" "nodecode" {
  %a = getelementptr i8, ptr %n, i64 %m.index
  %r = load float, ptr %a
  ret float %r
}

define i64 @LDI64X2(ptr "reg"="r64" %n, i64 "reg"="r64" %m.index) "enc"="rex(1, d, m.index, n) | 10001011 | rm(d, n, m.index, 2, 0, 0, 1)" "asm"="movq ({n},{m.index},2), {d}" "dclass"="r64" "nodecode" {
  %o = shl i64 %m.index, 1
  %a = getelementptr i8, ptr %n, i64 %o
  %r = load i64, ptr %a
  ret i64 %r
}

define i32 @LDI32X2(ptr "reg"="r64" %n, i64 "reg"="r64" %m.index) "enc"="rex(0, d, m.index, n) | 10001011 | rm(d, n, m.index, 2, 0, 0, 1)" "asm"="movl ({n},{m.index},2), {d}" "dclass"="r32" "zext"="32" "nodecode" {
  %o = shl i64 %m.index, 1
  %a = getelementptr i8, ptr %n, i64 %o
  %r = load i32, ptr %a
  ret i32 %r
}

define i8 @LDI8X2(ptr "reg"="r64" %n, i64 "reg"="r64" %m.index) "enc"="rex(0, d, m.index, n) | 00001111 | 10110110 | rm(d, n, m.index, 2, 0, 0, 1)" "asm"="movzbl ({n},{m.index},2), {d:r32}" "dclass"="r32" "zext"="8" "nodecode" {
  %o = shl i64 %m.index, 1
  %a = getelementptr i8, ptr %n, i64 %o
  %r = load i8, ptr %a
  ret i8 %r
}

define i16 @LDI16X2(ptr "reg"="r64" %n, i64 "reg"="r64" %m.index) "enc"="rex(0, d, m.index, n) | 00001111 | 10110111 | rm(d, n, m.index, 2, 0, 0, 1)" "asm"="movzwl ({n},{m.index},2), {d:r32}" "dclass"="r32" "zext"="16" "nodecode" {
  %o = shl i64 %m.index, 1
  %a = getelementptr i8, ptr %n, i64 %o
  %r = load i16, ptr %a
  ret i16 %r
}

define ptr @LDPTRX2(ptr "reg"="r64" %n, i64 "reg"="r64" %m.index) "enc"="rex(1, d, m.index, n) | 10001011 | rm(d, n, m.index, 2, 0, 0, 1)" "asm"="movq ({n},{m.index},2), {d}" "dclass"="r64" "nodecode" {
  %o = shl i64 %m.index, 1
  %a = getelementptr i8, ptr %n, i64 %o
  %r = load ptr, ptr %a
  ret ptr %r
}

define i32 @LDI8S32X2(ptr "reg"="r64" %n, i64 "reg"="r64" %m.index) "enc"="rex(0, d, m.index, n) | 00001111 | 10111110 | rm(d, n, m.index, 2, 0, 0, 1)" "asm"="movsbl ({n},{m.index},2), {d}" "nodecode" {
  %o = shl i64 %m.index, 1
  %a = getelementptr i8, ptr %n, i64 %o
  %v = load i8, ptr %a
  %r = sext i8 %v to i32
  ret i32 %r
}

define i32 @LDI16S32X2(ptr "reg"="r64" %n, i64 "reg"="r64" %m.index) "enc"="rex(0, d, m.index, n) | 00001111 | 10111111 | rm(d, n, m.index, 2, 0, 0, 1)" "asm"="movswl ({n},{m.index},2), {d}" "nodecode" {
  %o = shl i64 %m.index, 1
  %a = getelementptr i8, ptr %n, i64 %o
  %v = load i16, ptr %a
  %r = sext i16 %v to i32
  ret i32 %r
}

define i64 @LDI8S64X2(ptr "reg"="r64" %n, i64 "reg"="r64" %m.index) "enc"="rex(1, d, m.index, n) | 00001111 | 10111110 | rm(d, n, m.index, 2, 0, 0, 1)" "asm"="movsbq ({n},{m.index},2), {d}" "nodecode" {
  %o = shl i64 %m.index, 1
  %a = getelementptr i8, ptr %n, i64 %o
  %v = load i8, ptr %a
  %r = sext i8 %v to i64
  ret i64 %r
}

define i64 @LDI16S64X2(ptr "reg"="r64" %n, i64 "reg"="r64" %m.index) "enc"="rex(1, d, m.index, n) | 00001111 | 10111111 | rm(d, n, m.index, 2, 0, 0, 1)" "asm"="movswq ({n},{m.index},2), {d}" "nodecode" {
  %o = shl i64 %m.index, 1
  %a = getelementptr i8, ptr %n, i64 %o
  %v = load i16, ptr %a
  %r = sext i16 %v to i64
  ret i64 %r
}

define i64 @LDI32S64X2(ptr "reg"="r64" %n, i64 "reg"="r64" %m.index) "enc"="rex(1, d, m.index, n) | 01100011 | rm(d, n, m.index, 2, 0, 0, 1)" "asm"="movslq ({n},{m.index},2), {d}" "nodecode" {
  %o = shl i64 %m.index, 1
  %a = getelementptr i8, ptr %n, i64 %o
  %v = load i32, ptr %a
  %r = sext i32 %v to i64
  ret i64 %r
}

define double @LDDX2(ptr "reg"="r64" %n, i64 "reg"="r64" %m.index) "enc"="11110010 | rex(0, d, m.index, n) | 00001111 | 00010000 | rm(d, n, m.index, 2, 0, 0, 1)" "asm"="movsd ({n},{m.index},2), {d}" "dclass"="d" "nodecode" {
  %o = shl i64 %m.index, 1
  %a = getelementptr i8, ptr %n, i64 %o
  %r = load double, ptr %a
  ret double %r
}

define float @LDSX2(ptr "reg"="r64" %n, i64 "reg"="r64" %m.index) "enc"="11110011 | rex(0, d, m.index, n) | 00001111 | 00010000 | rm(d, n, m.index, 2, 0, 0, 1)" "asm"="movss ({n},{m.index},2), {d}" "dclass"="s" "nodecode" {
  %o = shl i64 %m.index, 1
  %a = getelementptr i8, ptr %n, i64 %o
  %r = load float, ptr %a
  ret float %r
}

define ptr @LEA64X2(ptr "reg"="r64" %n, i64 "reg"="r64" %m.index) "enc"="rex(1, d, m.index, n) | 10001101 | rm(d, n, m.index, 2, 0, 0, 1)" "asm"="leaq ({n},{m.index},2), {d}" "nodecode" {
  %o = shl i64 %m.index, 1
  %a = getelementptr i8, ptr %n, i64 %o
  ret ptr %a
}

define i64 @LDI64X4(ptr "reg"="r64" %n, i64 "reg"="r64" %m.index) "enc"="rex(1, d, m.index, n) | 10001011 | rm(d, n, m.index, 4, 0, 0, 1)" "asm"="movq ({n},{m.index},4), {d}" "dclass"="r64" "nodecode" {
  %o = shl i64 %m.index, 2
  %a = getelementptr i8, ptr %n, i64 %o
  %r = load i64, ptr %a
  ret i64 %r
}

define i32 @LDI32X4(ptr "reg"="r64" %n, i64 "reg"="r64" %m.index) "enc"="rex(0, d, m.index, n) | 10001011 | rm(d, n, m.index, 4, 0, 0, 1)" "asm"="movl ({n},{m.index},4), {d}" "dclass"="r32" "zext"="32" "nodecode" {
  %o = shl i64 %m.index, 2
  %a = getelementptr i8, ptr %n, i64 %o
  %r = load i32, ptr %a
  ret i32 %r
}

define i8 @LDI8X4(ptr "reg"="r64" %n, i64 "reg"="r64" %m.index) "enc"="rex(0, d, m.index, n) | 00001111 | 10110110 | rm(d, n, m.index, 4, 0, 0, 1)" "asm"="movzbl ({n},{m.index},4), {d:r32}" "dclass"="r32" "zext"="8" "nodecode" {
  %o = shl i64 %m.index, 2
  %a = getelementptr i8, ptr %n, i64 %o
  %r = load i8, ptr %a
  ret i8 %r
}

define i16 @LDI16X4(ptr "reg"="r64" %n, i64 "reg"="r64" %m.index) "enc"="rex(0, d, m.index, n) | 00001111 | 10110111 | rm(d, n, m.index, 4, 0, 0, 1)" "asm"="movzwl ({n},{m.index},4), {d:r32}" "dclass"="r32" "zext"="16" "nodecode" {
  %o = shl i64 %m.index, 2
  %a = getelementptr i8, ptr %n, i64 %o
  %r = load i16, ptr %a
  ret i16 %r
}

define ptr @LDPTRX4(ptr "reg"="r64" %n, i64 "reg"="r64" %m.index) "enc"="rex(1, d, m.index, n) | 10001011 | rm(d, n, m.index, 4, 0, 0, 1)" "asm"="movq ({n},{m.index},4), {d}" "dclass"="r64" "nodecode" {
  %o = shl i64 %m.index, 2
  %a = getelementptr i8, ptr %n, i64 %o
  %r = load ptr, ptr %a
  ret ptr %r
}

define i32 @LDI8S32X4(ptr "reg"="r64" %n, i64 "reg"="r64" %m.index) "enc"="rex(0, d, m.index, n) | 00001111 | 10111110 | rm(d, n, m.index, 4, 0, 0, 1)" "asm"="movsbl ({n},{m.index},4), {d}" "nodecode" {
  %o = shl i64 %m.index, 2
  %a = getelementptr i8, ptr %n, i64 %o
  %v = load i8, ptr %a
  %r = sext i8 %v to i32
  ret i32 %r
}

define i32 @LDI16S32X4(ptr "reg"="r64" %n, i64 "reg"="r64" %m.index) "enc"="rex(0, d, m.index, n) | 00001111 | 10111111 | rm(d, n, m.index, 4, 0, 0, 1)" "asm"="movswl ({n},{m.index},4), {d}" "nodecode" {
  %o = shl i64 %m.index, 2
  %a = getelementptr i8, ptr %n, i64 %o
  %v = load i16, ptr %a
  %r = sext i16 %v to i32
  ret i32 %r
}

define i64 @LDI8S64X4(ptr "reg"="r64" %n, i64 "reg"="r64" %m.index) "enc"="rex(1, d, m.index, n) | 00001111 | 10111110 | rm(d, n, m.index, 4, 0, 0, 1)" "asm"="movsbq ({n},{m.index},4), {d}" "nodecode" {
  %o = shl i64 %m.index, 2
  %a = getelementptr i8, ptr %n, i64 %o
  %v = load i8, ptr %a
  %r = sext i8 %v to i64
  ret i64 %r
}

define i64 @LDI16S64X4(ptr "reg"="r64" %n, i64 "reg"="r64" %m.index) "enc"="rex(1, d, m.index, n) | 00001111 | 10111111 | rm(d, n, m.index, 4, 0, 0, 1)" "asm"="movswq ({n},{m.index},4), {d}" "nodecode" {
  %o = shl i64 %m.index, 2
  %a = getelementptr i8, ptr %n, i64 %o
  %v = load i16, ptr %a
  %r = sext i16 %v to i64
  ret i64 %r
}

define i64 @LDI32S64X4(ptr "reg"="r64" %n, i64 "reg"="r64" %m.index) "enc"="rex(1, d, m.index, n) | 01100011 | rm(d, n, m.index, 4, 0, 0, 1)" "asm"="movslq ({n},{m.index},4), {d}" "nodecode" {
  %o = shl i64 %m.index, 2
  %a = getelementptr i8, ptr %n, i64 %o
  %v = load i32, ptr %a
  %r = sext i32 %v to i64
  ret i64 %r
}

define double @LDDX4(ptr "reg"="r64" %n, i64 "reg"="r64" %m.index) "enc"="11110010 | rex(0, d, m.index, n) | 00001111 | 00010000 | rm(d, n, m.index, 4, 0, 0, 1)" "asm"="movsd ({n},{m.index},4), {d}" "dclass"="d" "nodecode" {
  %o = shl i64 %m.index, 2
  %a = getelementptr i8, ptr %n, i64 %o
  %r = load double, ptr %a
  ret double %r
}

define float @LDSX4(ptr "reg"="r64" %n, i64 "reg"="r64" %m.index) "enc"="11110011 | rex(0, d, m.index, n) | 00001111 | 00010000 | rm(d, n, m.index, 4, 0, 0, 1)" "asm"="movss ({n},{m.index},4), {d}" "dclass"="s" "nodecode" {
  %o = shl i64 %m.index, 2
  %a = getelementptr i8, ptr %n, i64 %o
  %r = load float, ptr %a
  ret float %r
}

define ptr @LEA64X4(ptr "reg"="r64" %n, i64 "reg"="r64" %m.index) "enc"="rex(1, d, m.index, n) | 10001101 | rm(d, n, m.index, 4, 0, 0, 1)" "asm"="leaq ({n},{m.index},4), {d}" "nodecode" {
  %o = shl i64 %m.index, 2
  %a = getelementptr i8, ptr %n, i64 %o
  ret ptr %a
}

define i64 @LDI64X8(ptr "reg"="r64" %n, i64 "reg"="r64" %m.index) "enc"="rex(1, d, m.index, n) | 10001011 | rm(d, n, m.index, 8, 0, 0, 1)" "asm"="movq ({n},{m.index},8), {d}" "dclass"="r64" "nodecode" {
  %o = shl i64 %m.index, 3
  %a = getelementptr i8, ptr %n, i64 %o
  %r = load i64, ptr %a
  ret i64 %r
}

define i32 @LDI32X8(ptr "reg"="r64" %n, i64 "reg"="r64" %m.index) "enc"="rex(0, d, m.index, n) | 10001011 | rm(d, n, m.index, 8, 0, 0, 1)" "asm"="movl ({n},{m.index},8), {d}" "dclass"="r32" "zext"="32" "nodecode" {
  %o = shl i64 %m.index, 3
  %a = getelementptr i8, ptr %n, i64 %o
  %r = load i32, ptr %a
  ret i32 %r
}

define i8 @LDI8X8(ptr "reg"="r64" %n, i64 "reg"="r64" %m.index) "enc"="rex(0, d, m.index, n) | 00001111 | 10110110 | rm(d, n, m.index, 8, 0, 0, 1)" "asm"="movzbl ({n},{m.index},8), {d:r32}" "dclass"="r32" "zext"="8" "nodecode" {
  %o = shl i64 %m.index, 3
  %a = getelementptr i8, ptr %n, i64 %o
  %r = load i8, ptr %a
  ret i8 %r
}

define i16 @LDI16X8(ptr "reg"="r64" %n, i64 "reg"="r64" %m.index) "enc"="rex(0, d, m.index, n) | 00001111 | 10110111 | rm(d, n, m.index, 8, 0, 0, 1)" "asm"="movzwl ({n},{m.index},8), {d:r32}" "dclass"="r32" "zext"="16" "nodecode" {
  %o = shl i64 %m.index, 3
  %a = getelementptr i8, ptr %n, i64 %o
  %r = load i16, ptr %a
  ret i16 %r
}

define ptr @LDPTRX8(ptr "reg"="r64" %n, i64 "reg"="r64" %m.index) "enc"="rex(1, d, m.index, n) | 10001011 | rm(d, n, m.index, 8, 0, 0, 1)" "asm"="movq ({n},{m.index},8), {d}" "dclass"="r64" "nodecode" {
  %o = shl i64 %m.index, 3
  %a = getelementptr i8, ptr %n, i64 %o
  %r = load ptr, ptr %a
  ret ptr %r
}

define i32 @LDI8S32X8(ptr "reg"="r64" %n, i64 "reg"="r64" %m.index) "enc"="rex(0, d, m.index, n) | 00001111 | 10111110 | rm(d, n, m.index, 8, 0, 0, 1)" "asm"="movsbl ({n},{m.index},8), {d}" "nodecode" {
  %o = shl i64 %m.index, 3
  %a = getelementptr i8, ptr %n, i64 %o
  %v = load i8, ptr %a
  %r = sext i8 %v to i32
  ret i32 %r
}

define i32 @LDI16S32X8(ptr "reg"="r64" %n, i64 "reg"="r64" %m.index) "enc"="rex(0, d, m.index, n) | 00001111 | 10111111 | rm(d, n, m.index, 8, 0, 0, 1)" "asm"="movswl ({n},{m.index},8), {d}" "nodecode" {
  %o = shl i64 %m.index, 3
  %a = getelementptr i8, ptr %n, i64 %o
  %v = load i16, ptr %a
  %r = sext i16 %v to i32
  ret i32 %r
}

define i64 @LDI8S64X8(ptr "reg"="r64" %n, i64 "reg"="r64" %m.index) "enc"="rex(1, d, m.index, n) | 00001111 | 10111110 | rm(d, n, m.index, 8, 0, 0, 1)" "asm"="movsbq ({n},{m.index},8), {d}" "nodecode" {
  %o = shl i64 %m.index, 3
  %a = getelementptr i8, ptr %n, i64 %o
  %v = load i8, ptr %a
  %r = sext i8 %v to i64
  ret i64 %r
}

define i64 @LDI16S64X8(ptr "reg"="r64" %n, i64 "reg"="r64" %m.index) "enc"="rex(1, d, m.index, n) | 00001111 | 10111111 | rm(d, n, m.index, 8, 0, 0, 1)" "asm"="movswq ({n},{m.index},8), {d}" "nodecode" {
  %o = shl i64 %m.index, 3
  %a = getelementptr i8, ptr %n, i64 %o
  %v = load i16, ptr %a
  %r = sext i16 %v to i64
  ret i64 %r
}

define i64 @LDI32S64X8(ptr "reg"="r64" %n, i64 "reg"="r64" %m.index) "enc"="rex(1, d, m.index, n) | 01100011 | rm(d, n, m.index, 8, 0, 0, 1)" "asm"="movslq ({n},{m.index},8), {d}" "nodecode" {
  %o = shl i64 %m.index, 3
  %a = getelementptr i8, ptr %n, i64 %o
  %v = load i32, ptr %a
  %r = sext i32 %v to i64
  ret i64 %r
}

define double @LDDX8(ptr "reg"="r64" %n, i64 "reg"="r64" %m.index) "enc"="11110010 | rex(0, d, m.index, n) | 00001111 | 00010000 | rm(d, n, m.index, 8, 0, 0, 1)" "asm"="movsd ({n},{m.index},8), {d}" "dclass"="d" "nodecode" {
  %o = shl i64 %m.index, 3
  %a = getelementptr i8, ptr %n, i64 %o
  %r = load double, ptr %a
  ret double %r
}

define float @LDSX8(ptr "reg"="r64" %n, i64 "reg"="r64" %m.index) "enc"="11110011 | rex(0, d, m.index, n) | 00001111 | 00010000 | rm(d, n, m.index, 8, 0, 0, 1)" "asm"="movss ({n},{m.index},8), {d}" "dclass"="s" "nodecode" {
  %o = shl i64 %m.index, 3
  %a = getelementptr i8, ptr %n, i64 %o
  %r = load float, ptr %a
  ret float %r
}

define ptr @LEA64X8(ptr "reg"="r64" %n, i64 "reg"="r64" %m.index) "enc"="rex(1, d, m.index, n) | 10001101 | rm(d, n, m.index, 8, 0, 0, 1)" "asm"="leaq ({n},{m.index},8), {d}" "nodecode" {
  %o = shl i64 %m.index, 3
  %a = getelementptr i8, ptr %n, i64 %o
  ret ptr %a
}

define i64 @LDI64rip(ptr "sym" %s) "enc"="rex(1, d, 0, 0) | 10001011 | rm(d, 16, 4, 1, 0, s, 1)" "asm"="movq {s}(%rip), {d}" "reloc"="pc32x" "dclass"="r64" "nodecode" {
  %r = load i64, ptr %s
  ret i64 %r
}

define i32 @LDI32rip(ptr "sym" %s) "enc"="rex(0, d, 0, 0) | 10001011 | rm(d, 16, 4, 1, 0, s, 1)" "asm"="movl {s}(%rip), {d}" "reloc"="pc32x" "dclass"="r32" "zext"="32" "nodecode" {
  %r = load i32, ptr %s
  ret i32 %r
}

define i8 @LDI8rip(ptr "sym" %s) "enc"="rex(0, d, 0, 0) | 00001111 | 10110110 | rm(d, 16, 4, 1, 0, s, 1)" "asm"="movzbl {s}(%rip), {d:r32}" "reloc"="pc32x" "dclass"="r32" "zext"="8" "nodecode" {
  %r = load i8, ptr %s
  ret i8 %r
}

define i16 @LDI16rip(ptr "sym" %s) "enc"="rex(0, d, 0, 0) | 00001111 | 10110111 | rm(d, 16, 4, 1, 0, s, 1)" "asm"="movzwl {s}(%rip), {d:r32}" "reloc"="pc32x" "dclass"="r32" "zext"="16" "nodecode" {
  %r = load i16, ptr %s
  ret i16 %r
}

define ptr @LDPTRrip(ptr "sym" %s) "enc"="rex(1, d, 0, 0) | 10001011 | rm(d, 16, 4, 1, 0, s, 1)" "asm"="movq {s}(%rip), {d}" "reloc"="pc32x" "dclass"="r64" "nodecode" {
  %r = load ptr, ptr %s
  ret ptr %r
}

define i32 @LDI8S32rip(ptr "sym" %s) "enc"="rex(0, d, 0, 0) | 00001111 | 10111110 | rm(d, 16, 4, 1, 0, s, 1)" "asm"="movsbl {s}(%rip), {d}" "reloc"="pc32x" "nodecode" {
  %v = load i8, ptr %s
  %r = sext i8 %v to i32
  ret i32 %r
}

define i32 @LDI16S32rip(ptr "sym" %s) "enc"="rex(0, d, 0, 0) | 00001111 | 10111111 | rm(d, 16, 4, 1, 0, s, 1)" "asm"="movswl {s}(%rip), {d}" "reloc"="pc32x" "nodecode" {
  %v = load i16, ptr %s
  %r = sext i16 %v to i32
  ret i32 %r
}

define i64 @LDI8S64rip(ptr "sym" %s) "enc"="rex(1, d, 0, 0) | 00001111 | 10111110 | rm(d, 16, 4, 1, 0, s, 1)" "asm"="movsbq {s}(%rip), {d}" "reloc"="pc32x" "nodecode" {
  %v = load i8, ptr %s
  %r = sext i8 %v to i64
  ret i64 %r
}

define i64 @LDI16S64rip(ptr "sym" %s) "enc"="rex(1, d, 0, 0) | 00001111 | 10111111 | rm(d, 16, 4, 1, 0, s, 1)" "asm"="movswq {s}(%rip), {d}" "reloc"="pc32x" "nodecode" {
  %v = load i16, ptr %s
  %r = sext i16 %v to i64
  ret i64 %r
}

define i64 @LDI32S64rip(ptr "sym" %s) "enc"="rex(1, d, 0, 0) | 01100011 | rm(d, 16, 4, 1, 0, s, 1)" "asm"="movslq {s}(%rip), {d}" "reloc"="pc32x" "nodecode" {
  %v = load i32, ptr %s
  %r = sext i32 %v to i64
  ret i64 %r
}

define void @STI64rip(i64 "reg"="r64" %t, ptr "sym" %s) "enc"="rex(1, t, 0, 0) | 10001001 | rm(t, 16, 4, 1, 0, s, 1)" "asm"="movq {t}, {s}(%rip)" "reloc"="pc32x" "nodecode" {
  store i64 %t, ptr %s
  ret void
}

define void @STI32rip(i32 "reg"="r32" %t, ptr "sym" %s) "enc"="rex(0, t, 0, 0) | 10001001 | rm(t, 16, 4, 1, 0, s, 1)" "asm"="movl {t}, {s}(%rip)" "reloc"="pc32x" "nodecode" {
  store i32 %t, ptr %s
  ret void
}

define void @STI16rip(i16 "reg"="r32" %t, ptr "sym" %s) "enc"="01100110 | rex(0, t, 0, 0) | 10001001 | rm(t, 16, 4, 1, 0, s, 1)" "asm"="movw {t:r16}, {s}(%rip)" "reloc"="pc32x" "nodecode" {
  store i16 %t, ptr %s
  ret void
}

define void @STI8rip(i8 "reg"="r32" %t, ptr "sym" %s) "enc"="{ if(t<4) rex(0, t, 0, 0) ; 0100 0 R:1=(t)>>3 0 0 } | 10001000 | rm(t, 16, 4, 1, 0, s, 1)" "asm"="movb {t:r8}, {s}(%rip)" "reloc"="pc32x" "nodecode" {
  store i8 %t, ptr %s
  ret void
}

define void @STPTRrip(ptr "reg"="r64" %t, ptr "sym" %s) "enc"="rex(1, t, 0, 0) | 10001001 | rm(t, 16, 4, 1, 0, s, 1)" "asm"="movq {t}, {s}(%rip)" "reloc"="pc32x" "nodecode" {
  store ptr %t, ptr %s
  ret void
}

define double @LDDrip(ptr "sym" %s) "enc"="11110010 | rex(0, d, 0, 0) | 00001111 | 00010000 | rm(d, 16, 4, 1, 0, s, 1)" "asm"="movsd {s}(%rip), {d}" "reloc"="pc32x" "dclass"="d" "nodecode" {
  %r = load double, ptr %s
  ret double %r
}

define void @STDrip(double "reg"="d" %t, ptr "sym" %s) "enc"="11110010 | rex(0, t, 0, 0) | 00001111 | 00010001 | rm(t, 16, 4, 1, 0, s, 1)" "asm"="movsd {t}, {s}(%rip)" "reloc"="pc32x" "nodecode" {
  store double %t, ptr %s
  ret void
}

define float @LDSrip(ptr "sym" %s) "enc"="11110011 | rex(0, d, 0, 0) | 00001111 | 00010000 | rm(d, 16, 4, 1, 0, s, 1)" "asm"="movss {s}(%rip), {d}" "reloc"="pc32x" "dclass"="s" "nodecode" {
  %r = load float, ptr %s
  ret float %r
}

define void @STSrip(float "reg"="s" %t, ptr "sym" %s) "enc"="11110011 | rex(0, t, 0, 0) | 00001111 | 00010001 | rm(t, 16, 4, 1, 0, s, 1)" "asm"="movss {t}, {s}(%rip)" "reloc"="pc32x" "nodecode" {
  store float %t, ptr %s
  ret void
}

define %flags @CMP64rr(i64 "reg"="r64" %n, i64 "reg"="r64" %m) "enc"="rex(1, m, 0, n) | 00111001 | rr(m, n)" "asm"="cmpq {m}, {n}" "sets"="zcso" "nodecode" {
  %f0 = insertvalue %flags poison, i64 %n, 0
  %f = insertvalue %flags %f0, i64 %m, 1
  ret %flags %f
}

define %flags @CMP64ri(i64 "reg"="r64" %n, i64 "imm" %imm) "enc"="{ rex(1, 0, 0, n) | 10000011 | rr(7, n) | ib:s8=imm ; acc:0=n rex(1, 0, 0, 0) | 00111101 | iv:s32=imm ; rex(1, 0, 0, n) | 10000001 | rr(7, n) | iv:s32=imm }" "asm"="cmpq ${imm}, {n}" "sets"="zcso" "role"="cmpi.r64" "nodecode" {
  %f0 = insertvalue %flags poison, i64 %n, 0
  %f = insertvalue %flags %f0, i64 %imm, 1
  ret %flags %f
}

define %flags @TEST64rr(i64 "reg"="r64" %n, i64 "reg"="r64" %m) "enc"="rex(1, m, 0, n) | 10000101 | rr(m, n)" "asm"="testq {m}, {n}" "sets"="zcso" "role"="test.r64" "nodecode" {
  %r = and i64 %n, %m
  %f0 = insertvalue %flags poison, i64 %r, 0
  %f = insertvalue %flags %f0, i64 0, 1
  ret %flags %f
}

define %flags @CMP32rr(i32 "reg"="r32" %n, i32 "reg"="r32" %m) "enc"="rex(0, m, 0, n) | 00111001 | rr(m, n)" "asm"="cmpl {m}, {n}" "sets"="zcso" "nodecode" {
  %a = sext i32 %n to i64
  %b = sext i32 %m to i64
  %f0 = insertvalue %flags poison, i64 %a, 0
  %f = insertvalue %flags %f0, i64 %b, 1
  ret %flags %f
}

define %flags @CMP32ri(i32 "reg"="r32" %n, i32 "imm" %imm) "enc"="{ rex(0, 0, 0, n) | 10000011 | rr(7, n) | ib:s8=imm ; acc:0=n rex(0, 0, 0, 0) | 00111101 | iv:s32=imm ; rex(0, 0, 0, n) | 10000001 | rr(7, n) | iv:s32=imm }" "asm"="cmpl ${imm}, {n}" "sets"="zcso" "role"="cmpi.r32" "nodecode" {
  %a = sext i32 %n to i64
  %b = sext i32 %imm to i64
  %f0 = insertvalue %flags poison, i64 %a, 0
  %f = insertvalue %flags %f0, i64 %b, 1
  ret %flags %f
}

define %flags @TEST32rr(i32 "reg"="r32" %n, i32 "reg"="r32" %m) "enc"="rex(0, m, 0, n) | 10000101 | rr(m, n)" "asm"="testl {m}, {n}" "sets"="zcso" "role"="test.r32" "nodecode" {
  %r = and i32 %n, %m
  %x = sext i32 %r to i64
  %f0 = insertvalue %flags poison, i64 %x, 0
  %f = insertvalue %flags %f0, i64 0, 1
  ret %flags %f
}

define void @JMP(ptr "label" %dest) "enc"="{ 11101011 | rel:s8=dest-end ; 11101001 | rel:s32=dest-end }" "asm"="jmp {dest}" "role"="jump" "nodecode" {
  indirectbr ptr %dest, []
}

define void @JCC(i1 "cond" %c, ptr "label" %dest) "enc"="{ 0111 cc:4=c | rel:s8=dest-end ; 00001111 | 1000 cc:4=c | rel:s32=dest-end }" "asm"="j{c} {dest}" "inverse"="JCC" "nodecode" {
  br i1 %c, label %taken, label %next
  taken:
  indirectbr ptr %dest, []
  next:
  ret void
}

define void @CALL(ptr "sym" %s) "enc"="11101000 | rel:s32=s-end" "asm"="callq {s}" "reloc"="plt32" "role"="call" "nomatch" "nodecode" {
  ret void
}

define void @TAILJMP(ptr "sym" %s) "enc"="11101001 | rel:s32=s-end" "asm"="jmp {s}" "reloc"="plt32" "role"="tailcall" "nomatch" "nodecode" {
  ret void
}

define void @CALLr(ptr "reg"="r64" %n) "enc"="rex(0, 0, 0, n) | 11111111 | rr(2, n)" "asm"="callq *{n}" "role"="callr" "nomatch" "nodecode" {
  ret void
}

define void @RET() "enc"="11000011" "asm"="retq" "role"="ret" "nodecode" {
  ret void
}

define void @UD2() "enc"="00001111 | 00001011" "asm"="ud2" "role"="trap" "nodecode" {
  unreachable
}

define void @PAUSE() "enc"="11110011 | 10010000" "asm"="pause" "nodecode" {
  call void @llvm.x86.sse2.pause()
  ret void
}

declare void @llvm.x86.sse2.pause()

define i1 @SETCC(i1 "cond" %c) "enc"="{ if(d<4) 00001111 | 1001 cc:4=c | 11 000 rm:3=d&7 ; 0100 0 0 0 B:1=d>>3 | 00001111 | 1001 cc:4=c | 11 000 rm:3=d&7 }" "asm"="set{c} {d:r8}" "dclass"="r32" "nodecode" {
  ret i1 %c
}

define i64 @CMOV64(i1 "cond" %c, i64 "reg"="r64" %n, i64 "reg"="r64" "tied" %m) "enc"="rex(1, d, 0, n) | 00001111 | 0100 cc:4=c | rr(d, n)" "asm"="cmov{c}q {n}, {d}" "nodecode" {
  %r = select i1 %c, i64 %n, i64 %m
  ret i64 %r
}

define i32 @CMOV32(i1 "cond" %c, i32 "reg"="r32" %n, i32 "reg"="r32" "tied" %m) "enc"="rex(0, d, 0, n) | 00001111 | 0100 cc:4=c | rr(d, n)" "asm"="cmov{c}l {n}, {d}" "zext"="32" "nodecode" {
  %r = select i1 %c, i32 %n, i32 %m
  ret i32 %r
}

define ptr @CMOVP(i1 "cond" %c, ptr "reg"="r64" %n, ptr "reg"="r64" "tied" %m) "enc"="rex(1, d, 0, n) | 00001111 | 0100 cc:4=c | rr(d, n)" "asm"="cmov{c}q {n}, {d}" "nodecode" {
  %r = select i1 %c, ptr %n, ptr %m
  ret ptr %r
}

define i64 @SIGN64(i64 "reg"="rax" %a) "enc"="01001000 | 10011001" "asm"="cqto" "dclass"="rdx" "nomatch" "nodecode" {
  %r = ashr i64 %a, 63
  ret i64 %r
}

define i64 @IDIV64q(i64 "reg"="rax" %lo, i64 "reg"="rdx" %hi, i64 "reg"="r64" %b) "enc"="rex(1, 0, 0, b) | 11110111 | rr(7, b)" "asm"="idivq {b}" "dclass"="rax" "clobbers"="%rdx,flags" "nomatch" "nodecode" {
  %r = sdiv i64 %lo, %b
  ret i64 %r
}

define i64 @IDIV64r(i64 "reg"="rax" %lo, i64 "reg"="rdx" %hi, i64 "reg"="r64" %b) "enc"="rex(1, 0, 0, b) | 11110111 | rr(7, b)" "asm"="idivq {b}" "dclass"="rdx" "clobbers"="%rax,flags" "nomatch" "nodecode" {
  %r = srem i64 %lo, %b
  ret i64 %r
}

define i64 @DIV64q(i64 "reg"="rax" %lo, i64 "reg"="rdx" %hi, i64 "reg"="r64" %b) "enc"="rex(1, 0, 0, b) | 11110111 | rr(6, b)" "asm"="divq {b}" "dclass"="rax" "clobbers"="%rdx,flags" "nomatch" "nodecode" {
  %r = udiv i64 %lo, %b
  ret i64 %r
}

define i64 @DIV64r(i64 "reg"="rax" %lo, i64 "reg"="rdx" %hi, i64 "reg"="r64" %b) "enc"="rex(1, 0, 0, b) | 11110111 | rr(6, b)" "asm"="divq {b}" "dclass"="rdx" "clobbers"="%rax,flags" "nomatch" "nodecode" {
  %r = urem i64 %lo, %b
  ret i64 %r
}

define i32 @SIGN32(i32 "reg"="rax" %a) "enc"="10011001" "asm"="cltd" "dclass"="rdx" "nomatch" "nodecode" {
  %r = ashr i32 %a, 31
  ret i32 %r
}

define i32 @IDIV32q(i32 "reg"="rax" %lo, i32 "reg"="rdx" %hi, i32 "reg"="r32" %b) "enc"="rex(0, 0, 0, b) | 11110111 | rr(7, b)" "asm"="idivl {b}" "dclass"="rax" "clobbers"="%rdx,flags" "nomatch" "nodecode" {
  %r = sdiv i32 %lo, %b
  ret i32 %r
}

define i32 @IDIV32r(i32 "reg"="rax" %lo, i32 "reg"="rdx" %hi, i32 "reg"="r32" %b) "enc"="rex(0, 0, 0, b) | 11110111 | rr(7, b)" "asm"="idivl {b}" "dclass"="rdx" "clobbers"="%rax,flags" "nomatch" "nodecode" {
  %r = srem i32 %lo, %b
  ret i32 %r
}

define i32 @DIV32q(i32 "reg"="rax" %lo, i32 "reg"="rdx" %hi, i32 "reg"="r32" %b) "enc"="rex(0, 0, 0, b) | 11110111 | rr(6, b)" "asm"="divl {b}" "dclass"="rax" "clobbers"="%rdx,flags" "nomatch" "nodecode" {
  %r = udiv i32 %lo, %b
  ret i32 %r
}

define i32 @DIV32r(i32 "reg"="rax" %lo, i32 "reg"="rdx" %hi, i32 "reg"="r32" %b) "enc"="rex(0, 0, 0, b) | 11110111 | rr(6, b)" "asm"="divl {b}" "dclass"="rdx" "clobbers"="%rax,flags" "nomatch" "nodecode" {
  %r = urem i32 %lo, %b
  ret i32 %r
}

define i64 @sdiv.64(i64 "reg"="r64" %a, i64 "reg"="r64" %b) "recipe"="sdiv.64.expand" {
  %r = sdiv i64 %a, %b
  ret i64 %r
}

define i64 @sdiv.64.expand(i64 %a, i64 %b) {
  %h = call i64 @SIGN64(i64 %a)
  %r = call i64 @IDIV64q(i64 %a, i64 %h, i64 %b)
  ret i64 %r
}

define i64 @srem.64(i64 "reg"="r64" %a, i64 "reg"="r64" %b) "recipe"="srem.64.expand" {
  %r = srem i64 %a, %b
  ret i64 %r
}

define i64 @srem.64.expand(i64 %a, i64 %b) {
  %h = call i64 @SIGN64(i64 %a)
  %r = call i64 @IDIV64r(i64 %a, i64 %h, i64 %b)
  ret i64 %r
}

define i64 @udiv.64(i64 "reg"="r64" %a, i64 "reg"="r64" %b) "recipe"="udiv.64.expand" {
  %r = udiv i64 %a, %b
  ret i64 %r
}

define i64 @udiv.64.expand(i64 %a, i64 %b) {
  %h = call i64 @MOV64zero()
  %r = call i64 @DIV64q(i64 %a, i64 %h, i64 %b)
  ret i64 %r
}

define i64 @urem.64(i64 "reg"="r64" %a, i64 "reg"="r64" %b) "recipe"="urem.64.expand" {
  %r = urem i64 %a, %b
  ret i64 %r
}

define i64 @urem.64.expand(i64 %a, i64 %b) {
  %h = call i64 @MOV64zero()
  %r = call i64 @DIV64r(i64 %a, i64 %h, i64 %b)
  ret i64 %r
}

define i64 @MOV64zero() "enc"="rex(0, 0, 0, d) | 10111 o:3=d&7 | 00000000 | 00000000 | 00000000 | 00000000" "asm"="movl $0, {d:r32}" "nomatch" "nodecode" {
  ret i64 0
}

define i32 @sdiv.32(i32 "reg"="r32" %a, i32 "reg"="r32" %b) "recipe"="sdiv.32.expand" {
  %r = sdiv i32 %a, %b
  ret i32 %r
}

define i32 @sdiv.32.expand(i32 %a, i32 %b) {
  %h = call i32 @SIGN32(i32 %a)
  %r = call i32 @IDIV32q(i32 %a, i32 %h, i32 %b)
  ret i32 %r
}

define i32 @srem.32(i32 "reg"="r32" %a, i32 "reg"="r32" %b) "recipe"="srem.32.expand" {
  %r = srem i32 %a, %b
  ret i32 %r
}

define i32 @srem.32.expand(i32 %a, i32 %b) {
  %h = call i32 @SIGN32(i32 %a)
  %r = call i32 @IDIV32r(i32 %a, i32 %h, i32 %b)
  ret i32 %r
}

define i32 @udiv.32(i32 "reg"="r32" %a, i32 "reg"="r32" %b) "recipe"="udiv.32.expand" {
  %r = udiv i32 %a, %b
  ret i32 %r
}

define i32 @udiv.32.expand(i32 %a, i32 %b) {
  %h = call i32 @MOV32zero()
  %r = call i32 @DIV32q(i32 %a, i32 %h, i32 %b)
  ret i32 %r
}

define i32 @urem.32(i32 "reg"="r32" %a, i32 "reg"="r32" %b) "recipe"="urem.32.expand" {
  %r = urem i32 %a, %b
  ret i32 %r
}

define i32 @urem.32.expand(i32 %a, i32 %b) {
  %h = call i32 @MOV32zero()
  %r = call i32 @DIV32r(i32 %a, i32 %h, i32 %b)
  ret i32 %r
}

define i32 @MOV32zero() "enc"="rex(0, 0, 0, d) | 10111 o:3=d&7 | 00000000 | 00000000 | 00000000 | 00000000" "asm"="movl $0, {d:r32}" "nomatch" "nodecode" {
  ret i32 0
}

define i64 @smin.64(i64 "reg"="r64" %a, i64 "reg"="r64" %b) "recipe"="smin.64.expand" {
  %r = call i64 @llvm.smin.i64(i64 %a, i64 %b)
  ret i64 %r
}

define i64 @smin.64.expand(i64 %a, i64 %b) {
  %f = call %flags @CMP64rr(i64 %a, i64 %b)
  %c = call i1 @cc.l(%flags %f)
  %r = call i64 @CMOV64(i1 %c, i64 %a, i64 %b)
  ret i64 %r
}

define i64 @smax.64(i64 "reg"="r64" %a, i64 "reg"="r64" %b) "recipe"="smax.64.expand" {
  %r = call i64 @llvm.smax.i64(i64 %a, i64 %b)
  ret i64 %r
}

define i64 @smax.64.expand(i64 %a, i64 %b) {
  %f = call %flags @CMP64rr(i64 %a, i64 %b)
  %c = call i1 @cc.g(%flags %f)
  %r = call i64 @CMOV64(i1 %c, i64 %a, i64 %b)
  ret i64 %r
}

define i64 @umin.64(i64 "reg"="r64" %a, i64 "reg"="r64" %b) "recipe"="umin.64.expand" {
  %r = call i64 @llvm.umin.i64(i64 %a, i64 %b)
  ret i64 %r
}

define i64 @umin.64.expand(i64 %a, i64 %b) {
  %f = call %flags @CMP64rr(i64 %a, i64 %b)
  %c = call i1 @cc.b(%flags %f)
  %r = call i64 @CMOV64(i1 %c, i64 %a, i64 %b)
  ret i64 %r
}

define i64 @umax.64(i64 "reg"="r64" %a, i64 "reg"="r64" %b) "recipe"="umax.64.expand" {
  %r = call i64 @llvm.umax.i64(i64 %a, i64 %b)
  ret i64 %r
}

define i64 @umax.64.expand(i64 %a, i64 %b) {
  %f = call %flags @CMP64rr(i64 %a, i64 %b)
  %c = call i1 @cc.a(%flags %f)
  %r = call i64 @CMOV64(i1 %c, i64 %a, i64 %b)
  ret i64 %r
}

define i32 @smin.32(i32 "reg"="r32" %a, i32 "reg"="r32" %b) "recipe"="smin.32.expand" {
  %r = call i32 @llvm.smin.i32(i32 %a, i32 %b)
  ret i32 %r
}

define i32 @smin.32.expand(i32 %a, i32 %b) {
  %f = call %flags @CMP32rr(i32 %a, i32 %b)
  %c = call i1 @cc.l(%flags %f)
  %r = call i32 @CMOV32(i1 %c, i32 %a, i32 %b)
  ret i32 %r
}

define i32 @smax.32(i32 "reg"="r32" %a, i32 "reg"="r32" %b) "recipe"="smax.32.expand" {
  %r = call i32 @llvm.smax.i32(i32 %a, i32 %b)
  ret i32 %r
}

define i32 @smax.32.expand(i32 %a, i32 %b) {
  %f = call %flags @CMP32rr(i32 %a, i32 %b)
  %c = call i1 @cc.g(%flags %f)
  %r = call i32 @CMOV32(i1 %c, i32 %a, i32 %b)
  ret i32 %r
}

define i32 @umin.32(i32 "reg"="r32" %a, i32 "reg"="r32" %b) "recipe"="umin.32.expand" {
  %r = call i32 @llvm.umin.i32(i32 %a, i32 %b)
  ret i32 %r
}

define i32 @umin.32.expand(i32 %a, i32 %b) {
  %f = call %flags @CMP32rr(i32 %a, i32 %b)
  %c = call i1 @cc.b(%flags %f)
  %r = call i32 @CMOV32(i1 %c, i32 %a, i32 %b)
  ret i32 %r
}

define i32 @umax.32(i32 "reg"="r32" %a, i32 "reg"="r32" %b) "recipe"="umax.32.expand" {
  %r = call i32 @llvm.umax.i32(i32 %a, i32 %b)
  ret i32 %r
}

define i32 @umax.32.expand(i32 %a, i32 %b) {
  %f = call %flags @CMP32rr(i32 %a, i32 %b)
  %c = call i1 @cc.a(%flags %f)
  %r = call i32 @CMOV32(i1 %c, i32 %a, i32 %b)
  ret i32 %r
}

declare i64 @llvm.smin.i64(i64, i64)

declare i64 @llvm.smax.i64(i64, i64)

declare i64 @llvm.umin.i64(i64, i64)

declare i64 @llvm.umax.i64(i64, i64)

declare i32 @llvm.smin.i32(i32, i32)

declare i32 @llvm.smax.i32(i32, i32)

declare i32 @llvm.umin.i32(i32, i32)

declare i32 @llvm.umax.i32(i32, i32)

define %flags @TESTB1(i1 "reg"="r32" %n) "enc"="{ acc:0=n 10101000 | 00000001 ; if(n<4) 11110110 | rr(0, n) | 00000001 ; 0100 0 0 0 B:1=n>>3 | 11110110 | rr(0, n) | 00000001 }" "asm"="testb $1, {n:r8}" "sets"="zcso" "nomatch" "nodecode" {
  %x = zext i1 %n to i64
  %f0 = insertvalue %flags poison, i64 %x, 0
  %f = insertvalue %flags %f0, i64 0, 1
  ret %flags %f
}

define i64 @select.64(i1 "reg"="r32" %c, i64 "reg"="r64" %a, i64 "reg"="r64" %b) "recipe"="select.64.expand" {
  %r = select i1 %c, i64 %a, i64 %b
  ret i64 %r
}

define i64 @select.64.expand(i1 %c, i64 %a, i64 %b) {
  %f = call %flags @TESTB1(i1 %c)
  %ne = call i1 @cc.ne(%flags %f)
  %r = call i64 @CMOV64(i1 %ne, i64 %a, i64 %b)
  ret i64 %r
}

define i32 @select.32(i1 "reg"="r32" %c, i32 "reg"="r32" %a, i32 "reg"="r32" %b) "recipe"="select.32.expand" {
  %r = select i1 %c, i32 %a, i32 %b
  ret i32 %r
}

define i32 @select.32.expand(i1 %c, i32 %a, i32 %b) {
  %f = call %flags @TESTB1(i1 %c)
  %ne = call i1 @cc.ne(%flags %f)
  %r = call i32 @CMOV32(i1 %ne, i32 %a, i32 %b)
  ret i32 %r
}

define ptr @select.p(i1 "reg"="r32" %c, ptr "reg"="r64" %a, ptr "reg"="r64" %b) "recipe"="select.p.expand" {
  %r = select i1 %c, ptr %a, ptr %b
  ret ptr %r
}

define ptr @select.p.expand(i1 %c, ptr %a, ptr %b) {
  %f = call %flags @TESTB1(i1 %c)
  %ne = call i1 @cc.ne(%flags %f)
  %r = call ptr @CMOVP(i1 %ne, ptr %a, ptr %b)
  ret ptr %r
}

define i8 @ZEXT1toi8(i1 "reg"="r32" "tied" %n) "enc"="rex(0, 0, 0, d) | 10000011 | rr(4, d) | 00000001" "asm"="andl $1, {d:r32}" "clobbers"="flags" "nomatch" "dclass"="r32" "nodecode" {
  %r = zext i1 %n to i8
  ret i8 %r
}

define i8 @NEGxi8(i8 "reg"="r32" "tied" %n) "enc"="rex(0, 0, 0, d) | 11110111 | rr(3, d)" "asm"="negl {d}" "clobbers"="flags" "nomatch" "dclass"="r32" "nodecode" {
  %r = sub i8 0, %n
  ret i8 %r
}

define i8 @sext.1.i8(i1 "reg"="r32" %n) "recipe"="sext.1.i8.expand" {
  %r = sext i1 %n to i8
  ret i8 %r
}

define i8 @sext.1.i8.expand(i1 %n) {
  %z = call i8 @ZEXT1toi8(i1 %n)
  %r = call i8 @NEGxi8(i8 %z)
  ret i8 %r
}

define i16 @ZEXT1toi16(i1 "reg"="r32" "tied" %n) "enc"="rex(0, 0, 0, d) | 10000011 | rr(4, d) | 00000001" "asm"="andl $1, {d:r32}" "clobbers"="flags" "nomatch" "dclass"="r32" "nodecode" {
  %r = zext i1 %n to i16
  ret i16 %r
}

define i16 @NEGxi16(i16 "reg"="r32" "tied" %n) "enc"="rex(0, 0, 0, d) | 11110111 | rr(3, d)" "asm"="negl {d}" "clobbers"="flags" "nomatch" "dclass"="r32" "nodecode" {
  %r = sub i16 0, %n
  ret i16 %r
}

define i16 @sext.1.i16(i1 "reg"="r32" %n) "recipe"="sext.1.i16.expand" {
  %r = sext i1 %n to i16
  ret i16 %r
}

define i16 @sext.1.i16.expand(i1 %n) {
  %z = call i16 @ZEXT1toi16(i1 %n)
  %r = call i16 @NEGxi16(i16 %z)
  ret i16 %r
}

define i32 @ZEXT1toi32(i1 "reg"="r32" "tied" %n) "enc"="rex(0, 0, 0, d) | 10000011 | rr(4, d) | 00000001" "asm"="andl $1, {d:r32}" "clobbers"="flags" "nomatch" "nodecode" {
  %r = zext i1 %n to i32
  ret i32 %r
}

define i32 @NEGxi32(i32 "reg"="r32" "tied" %n) "enc"="rex(0, 0, 0, d) | 11110111 | rr(3, d)" "asm"="negl {d}" "clobbers"="flags" "nomatch" "nodecode" {
  %r = sub i32 0, %n
  ret i32 %r
}

define i32 @sext.1.i32(i1 "reg"="r32" %n) "recipe"="sext.1.i32.expand" {
  %r = sext i1 %n to i32
  ret i32 %r
}

define i32 @sext.1.i32.expand(i1 %n) {
  %z = call i32 @ZEXT1toi32(i1 %n)
  %r = call i32 @NEGxi32(i32 %z)
  ret i32 %r
}

define i64 @ZEXT1toi64(i1 "reg"="r32" "tied" %n) "enc"="rex(0, 0, 0, d) | 10000011 | rr(4, d) | 00000001" "asm"="andl $1, {d:r32}" "clobbers"="flags" "nomatch" "dclass"="r64" "nodecode" {
  %r = zext i1 %n to i64
  ret i64 %r
}

define i64 @NEGxi64(i64 "reg"="r64" "tied" %n) "enc"="rex(1, 0, 0, d) | 11110111 | rr(3, d)" "asm"="negq {d}" "clobbers"="flags" "nomatch" "nodecode" {
  %r = sub i64 0, %n
  ret i64 %r
}

define i64 @sext.1.i64(i1 "reg"="r32" %n) "recipe"="sext.1.i64.expand" {
  %r = sext i1 %n to i64
  ret i64 %r
}

define i64 @sext.1.i64.expand(i1 %n) {
  %z = call i64 @ZEXT1toi64(i1 %n)
  %r = call i64 @NEGxi64(i64 %z)
  ret i64 %r
}

define i64 @IMULH64(i64 "reg"="rax" %n, i64 "reg"="r64" %m) "enc"="rex(1, 0, 0, m) | 11110111 | rr(5, m)" "asm"="imulq {m}" "dclass"="rdx" "clobbers"="%rax,flags" "nodecode" {
  %a = sext i64 %n to i128
  %b = sext i64 %m to i128
  %p = mul i128 %a, %b
  %h = lshr i128 %p, 64
  %r = trunc i128 %h to i64
  ret i64 %r
}

define i64 @MULH64(i64 "reg"="rax" %n, i64 "reg"="r64" %m) "enc"="rex(1, 0, 0, m) | 11110111 | rr(4, m)" "asm"="mulq {m}" "dclass"="rdx" "clobbers"="%rax,flags" "nodecode" {
  %a = zext i64 %n to i128
  %b = zext i64 %m to i128
  %p = mul i128 %a, %b
  %h = lshr i128 %p, 64
  %r = trunc i128 %h to i64
  ret i64 %r
}

define i64 @MOVZX32(i32 "reg"="r32" %n) "enc"="rex(0, n, 0, d) | 10001001 | rr(n, d)" "asm"="movl {n}, {d:r32}" "zext"="32" "nodecode" {
  %r = zext i32 %n to i64
  ret i64 %r
}

; Floating point, SSE2's scalar instructions. The flags of a comparison
; (ucomisd) are ZF, PF and CF: unordered sets all three, less than CF,
; equal ZF. Each of the eight predicates a condition tests is one condition
; code; oeq and une are two (prepare splits them: "fcmp-split").
%fflags = type { double, double }
; The flags of the operands the other way round: ucomisd n, m compares m
; with n. Its conditions test the predicates whose operands are swapped:
; olt is ogt the other way round, ugt ult.
%fflagsr = type { double, double }

define i1 @cc.fr.olt(%fflagsr %f) "cond"="7" "asm"="a" "reads"="zcp" {
  %a = extractvalue %fflagsr %f, 0
  %b = extractvalue %fflagsr %f, 1
  %c = fcmp olt double %a, %b
  ret i1 %c
}

define i1 @cc.fr.ole(%fflagsr %f) "cond"="3" "asm"="ae" "reads"="zcp" {
  %a = extractvalue %fflagsr %f, 0
  %b = extractvalue %fflagsr %f, 1
  %c = fcmp ole double %a, %b
  ret i1 %c
}

define i1 @cc.fr.ugt(%fflagsr %f) "cond"="2" "asm"="b" "reads"="zcp" {
  %a = extractvalue %fflagsr %f, 0
  %b = extractvalue %fflagsr %f, 1
  %c = fcmp ugt double %a, %b
  ret i1 %c
}

define i1 @cc.fr.uge(%fflagsr %f) "cond"="6" "asm"="be" "reads"="zcp" {
  %a = extractvalue %fflagsr %f, 0
  %b = extractvalue %fflagsr %f, 1
  %c = fcmp uge double %a, %b
  ret i1 %c
}

define i1 @cc.f.ogt(%fflags %f) "cond"="7" "asm"="a" "reads"="zcp" {
  %a = extractvalue %fflags %f, 0
  %b = extractvalue %fflags %f, 1
  %c = fcmp ogt double %a, %b
  ret i1 %c
}

define i1 @cc.f.oge(%fflags %f) "cond"="3" "asm"="ae" "reads"="zcp" {
  %a = extractvalue %fflags %f, 0
  %b = extractvalue %fflags %f, 1
  %c = fcmp oge double %a, %b
  ret i1 %c
}

define i1 @cc.f.ult(%fflags %f) "cond"="2" "asm"="b" "reads"="zcp" {
  %a = extractvalue %fflags %f, 0
  %b = extractvalue %fflags %f, 1
  %c = fcmp ult double %a, %b
  ret i1 %c
}

define i1 @cc.f.ule(%fflags %f) "cond"="6" "asm"="be" "reads"="zcp" {
  %a = extractvalue %fflags %f, 0
  %b = extractvalue %fflags %f, 1
  %c = fcmp ule double %a, %b
  ret i1 %c
}

define i1 @cc.f.ueq(%fflags %f) "cond"="4" "asm"="e" "reads"="zcp" {
  %a = extractvalue %fflags %f, 0
  %b = extractvalue %fflags %f, 1
  %c = fcmp ueq double %a, %b
  ret i1 %c
}

define i1 @cc.f.one(%fflags %f) "cond"="5" "asm"="ne" "reads"="zcp" {
  %a = extractvalue %fflags %f, 0
  %b = extractvalue %fflags %f, 1
  %c = fcmp one double %a, %b
  ret i1 %c
}

define i1 @cc.f.uno(%fflags %f) "cond"="10" "asm"="p" "reads"="zcp" {
  %a = extractvalue %fflags %f, 0
  %b = extractvalue %fflags %f, 1
  %c = fcmp uno double %a, %b
  ret i1 %c
}

define i1 @cc.f.ord(%fflags %f) "cond"="11" "asm"="np" "reads"="zcp" {
  %a = extractvalue %fflags %f, 0
  %b = extractvalue %fflags %f, 1
  %c = fcmp ord double %a, %b
  ret i1 %c
}

define %fflags @UCOMISD(double "reg"="d" %n, double "reg"="d" %m) "enc"="01100110 | rex(0, n, 0, m) | 00001111 | 00101110 | rr(n, m)" "asm"="ucomisd {m}, {n}" "sets"="zcp" "nodecode" {
  %f0 = insertvalue %fflags poison, double %n, 0
  %f = insertvalue %fflags %f0, double %m, 1
  ret %fflags %f
}

define %fflagsr @UCOMISDrev(double "reg"="d" %n, double "reg"="d" %m) "enc"="01100110 | rex(0, m, 0, n) | 00001111 | 00101110 | rr(m, n)" "asm"="ucomisd {n}, {m}" "sets"="zcp" "nodecode" {
  %f0 = insertvalue %fflagsr poison, double %n, 0
  %f = insertvalue %fflagsr %f0, double %m, 1
  ret %fflagsr %f
}

define double @ADDSD(double "reg"="d" "tied" %n, double "reg"="d" %m) "enc"="11110010 | rex(0, d, 0, m) | 00001111 | 01011000 | rr(d, m)" "asm"="addsd {m}, {d}" "nodecode" {
  %r = fadd double %n, %m
  ret double %r
}

define double @SUBSD(double "reg"="d" "tied" %n, double "reg"="d" %m) "enc"="11110010 | rex(0, d, 0, m) | 00001111 | 01011100 | rr(d, m)" "asm"="subsd {m}, {d}" "nodecode" {
  %r = fsub double %n, %m
  ret double %r
}

define double @MULSD(double "reg"="d" "tied" %n, double "reg"="d" %m) "enc"="11110010 | rex(0, d, 0, m) | 00001111 | 01011001 | rr(d, m)" "asm"="mulsd {m}, {d}" "nodecode" {
  %r = fmul double %n, %m
  ret double %r
}

define double @DIVSD(double "reg"="d" "tied" %n, double "reg"="d" %m) "enc"="11110010 | rex(0, d, 0, m) | 00001111 | 01011110 | rr(d, m)" "asm"="divsd {m}, {d}" "nodecode" {
  %r = fdiv double %n, %m
  ret double %r
}

define double @SQRTSD(double "reg"="d" %n) "enc"="11110010 | rex(0, d, 0, n) | 00001111 | 01010001 | rr(d, n)" "asm"="sqrtsd {n}, {d}" "nodecode" {
  %r = call double @llvm.sqrt.f64(double %n)
  ret double %r
}

declare double @llvm.sqrt.f64(double)

define double @LDSD(ptr "reg"="r64" %n, i64 "imm" %off) "enc"="11110010 | rex(0, d, 4, n) | 00001111 | 00010000 | rm(d, n, 4, 1, off, 0, 1)" "asm"="movsd {off}({n}), {d}" "role"="load.d" "nodecode" {
  %a = getelementptr i8, ptr %n, i64 %off
  %r = load double, ptr %a
  ret double %r
}

define void @STSD(double "reg"="d" %t, ptr "reg"="r64" %n, i64 "imm" %off) "enc"="11110010 | rex(0, t, 4, n) | 00001111 | 00010001 | rm(t, n, 4, 1, off, 0, 1)" "asm"="movsd {t}, {off}({n})" "role"="store.d" "nodecode" {
  %a = getelementptr i8, ptr %n, i64 %off
  store double %t, ptr %a
  ret void
}

define double @MOVSDfromi64(i64 "reg"="r64" %n) "enc"="01100110 | rex(1, d, 0, n) | 00001111 | 01101110 | rr(d, n)" "asm"="movq {n}, {d}" "role"="move.d.r64" "nodecode" {
  %r = bitcast i64 %n to double
  ret double %r
}

define i64 @MOVi64fromSD(double "reg"="d" %n) "enc"="01100110 | rex(1, n, 0, d) | 00001111 | 01111110 | rr(n, d)" "asm"="movq {n}, {d}" "role"="move.r64.d" "nodecode" {
  %r = bitcast double %n to i64
  ret i64 %r
}

define double @CVTSITOFPI32SD(i32 "reg"="r32" %n) "enc"="11110010 | rex(0, d, 0, n) | 00001111 | 00101010 | rr(d, n)" "asm"="cvtsi2sd {n}, {d}" "nodecode" {
  %r = sitofp i32 %n to double
  ret double %r
}

define double @CVTSITOFPI64SD(i64 "reg"="r64" %n) "enc"="11110010 | rex(1, d, 0, n) | 00001111 | 00101010 | rr(d, n)" "asm"="cvtsi2sd {n}, {d}" "nodecode" {
  %r = sitofp i64 %n to double
  ret double %r
}

define double @CVTUITOFPI32SD(i32 "reg"="r32" %n) "enc"="11110010 | rex(1, d, 0, n) | 00001111 | 00101010 | rr(d, n)" "asm"="cvtsi2sd {n:r64}, {d}" "nodecode" {
  %r = uitofp i32 %n to double
  ret double %r
}

define i32 @CVTTFPTOSISDI32(double "reg"="d" %n) "enc"="11110010 | rex(0, d, 0, n) | 00001111 | 00101100 | rr(d, n)" "asm"="cvttsd2si {n}, {d}" "dclass"="r32" "nodecode" {
  %r = fptosi double %n to i32
  ret i32 %r
}

define i64 @CVTTFPTOSISDI64(double "reg"="d" %n) "enc"="11110010 | rex(1, d, 0, n) | 00001111 | 00101100 | rr(d, n)" "asm"="cvttsd2si {n}, {d}" "dclass"="r64" "nodecode" {
  %r = fptosi double %n to i64
  ret i64 %r
}

define i32 @CVTTFPTOUISDI32(double "reg"="d" %n) "enc"="11110010 | rex(1, d, 0, n) | 00001111 | 00101100 | rr(d, n)" "asm"="cvttsd2si {n}, {d:r64}" "dclass"="r32" "nodecode" {
  %r = fptoui double %n to i32
  ret i32 %r
}

define double @select.d(i1 "reg"="r32" %c, double "reg"="d" %a, double "reg"="d" %b) "recipe"="select.d.expand" {
  %r = select i1 %c, double %a, double %b
  ret double %r
}

define double @select.d.expand(i1 %c, double %a, double %b) {
  %ai = call i64 @MOVi64fromSD(double %a)
  %bi = call i64 @MOVi64fromSD(double %b)
  %f = call %flags @TESTB1(i1 %c)
  %ne = call i1 @cc.ne(%flags %f)
  %ri = call i64 @CMOV64(i1 %ne, i64 %ai, i64 %bi)
  %r = call double @MOVSDfromi64(i64 %ri)
  ret double %r
}

define double @fneg.d(double "reg"="d" %a) "recipe"="fneg.d.expand" {
  %r = fneg double %a
  ret double %r
}

define double @fneg.d.expand(double %a) {
  %ai = call i64 @MOVi64fromSD(double %a)
  %k = call i64 @MOVABS64ri(i64 -9223372036854775808)
  %x = call i64 @XOR64rr(i64 %ai, i64 %k)
  %r = call double @MOVSDfromi64(i64 %x)
  ret double %r
}

define double @fabs.d(double "reg"="d" %a) "recipe"="fabs.d.expand" {
  %r = call double @llvm.fabs.f64(double %a)
  ret double %r
}

define double @fabs.d.expand(double %a) {
  %ai = call i64 @MOVi64fromSD(double %a)
  %k = call i64 @MOVABS64ri(i64 9223372036854775807)
  %x = call i64 @AND64rr(i64 %ai, i64 %k)
  %r = call double @MOVSDfromi64(i64 %x)
  ret double %r
}

declare double @llvm.fabs.f64(double)

declare double @llvm.fmuladd.f64(double, double, double)

define double @fmuladd.d(double "reg"="d" %a, double "reg"="d" %b, double "reg"="d" %c) "recipe"="fmuladd.d.expand" {
  %r = call double @llvm.fmuladd.f64(double %a, double %b, double %c)
  ret double %r
}

define double @fmuladd.d.expand(double %a, double %b, double %c) {
  %p = call double @MULSD(double %a, double %b)
  %r = call double @ADDSD(double %p, double %c)
  ret double %r
}

define double @uitofp.64.d(i64 "reg"="r64" %x) "recipe"="uitofp.64.d.expand" {
  %r = uitofp i64 %x to double
  ret double %r
}

define double @uitofp.64.d.expand(i64 %x) {
  %h = call i64 @SHR64ri(i64 %x, i64 1)
  %l = call i64 @AND64ri(i64 %x, i64 1)
  %o = call i64 @OR64rr(i64 %h, i64 %l)
  %f1 = call %flags @TEST64rr(i64 %x, i64 %x)
  %n1 = call i1 @cc.l(%flags %f1)
  %t = call i64 @CMOV64(i1 %n1, i64 %o, i64 %x)
  %c = call double @CVTSITOFPI64SD(i64 %t)
  %c2 = call double @ADDSD(double %c, double %c)
  %ci = call i64 @MOVi64fromSD(double %c)
  %c2i = call i64 @MOVi64fromSD(double %c2)
  %f2 = call %flags @TEST64rr(i64 %x, i64 %x)
  %n2 = call i1 @cc.l(%flags %f2)
  %ri = call i64 @CMOV64(i1 %n2, i64 %c2i, i64 %ci)
  %r = call double @MOVSDfromi64(i64 %ri)
  ret double %r
}

define i64 @fptoui.64.d(double "reg"="d" %x) "recipe"="fptoui.64.d.expand" {
  %r = fptoui double %x to i64
  ret i64 %r
}

define i64 @fptoui.64.d.expand(double %x) {
  %a = call i64 @CVTTFPTOSISDI64(double %x)
  %ki = call i64 @MOVABS64ri(i64 4890909195324358656)
  %k = call double @MOVSDfromi64(i64 %ki)
  %y = call double @SUBSD(double %x, double %k)
  %b = call i64 @CVTTFPTOSISDI64(double %y)
  %s = call i64 @SAR64ri(i64 %a, i64 63)
  %m = call i64 @AND64rr(i64 %b, i64 %s)
  %r = call i64 @OR64rr(i64 %a, i64 %m)
  ret i64 %r
}

define %fflags @UCOMISS(float "reg"="s" %n, float "reg"="s" %m) "enc"="rex(0, n, 0, m) | 00001111 | 00101110 | rr(n, m)" "asm"="ucomiss {m}, {n}" "sets"="zcp" "nodecode" {
  %a = fpext float %n to double
  %b = fpext float %m to double
  %f0 = insertvalue %fflags poison, double %a, 0
  %f = insertvalue %fflags %f0, double %b, 1
  ret %fflags %f
}

define %fflagsr @UCOMISSrev(float "reg"="s" %n, float "reg"="s" %m) "enc"="rex(0, m, 0, n) | 00001111 | 00101110 | rr(m, n)" "asm"="ucomiss {n}, {m}" "sets"="zcp" "nodecode" {
  %a = fpext float %n to double
  %b = fpext float %m to double
  %f0 = insertvalue %fflagsr poison, double %a, 0
  %f = insertvalue %fflagsr %f0, double %b, 1
  ret %fflagsr %f
}

define float @ADDSS(float "reg"="s" "tied" %n, float "reg"="s" %m) "enc"="11110011 | rex(0, d, 0, m) | 00001111 | 01011000 | rr(d, m)" "asm"="addss {m}, {d}" "nodecode" {
  %r = fadd float %n, %m
  ret float %r
}

define float @SUBSS(float "reg"="s" "tied" %n, float "reg"="s" %m) "enc"="11110011 | rex(0, d, 0, m) | 00001111 | 01011100 | rr(d, m)" "asm"="subss {m}, {d}" "nodecode" {
  %r = fsub float %n, %m
  ret float %r
}

define float @MULSS(float "reg"="s" "tied" %n, float "reg"="s" %m) "enc"="11110011 | rex(0, d, 0, m) | 00001111 | 01011001 | rr(d, m)" "asm"="mulss {m}, {d}" "nodecode" {
  %r = fmul float %n, %m
  ret float %r
}

define float @DIVSS(float "reg"="s" "tied" %n, float "reg"="s" %m) "enc"="11110011 | rex(0, d, 0, m) | 00001111 | 01011110 | rr(d, m)" "asm"="divss {m}, {d}" "nodecode" {
  %r = fdiv float %n, %m
  ret float %r
}

define float @SQRTSS(float "reg"="s" %n) "enc"="11110011 | rex(0, d, 0, n) | 00001111 | 01010001 | rr(d, n)" "asm"="sqrtss {n}, {d}" "nodecode" {
  %r = call float @llvm.sqrt.f32(float %n)
  ret float %r
}

declare float @llvm.sqrt.f32(float)

define float @LDSS(ptr "reg"="r64" %n, i64 "imm" %off) "enc"="11110011 | rex(0, d, 4, n) | 00001111 | 00010000 | rm(d, n, 4, 1, off, 0, 1)" "asm"="movss {off}({n}), {d}" "role"="load.s" "nodecode" {
  %a = getelementptr i8, ptr %n, i64 %off
  %r = load float, ptr %a
  ret float %r
}

define void @STSS(float "reg"="s" %t, ptr "reg"="r64" %n, i64 "imm" %off) "enc"="11110011 | rex(0, t, 4, n) | 00001111 | 00010001 | rm(t, n, 4, 1, off, 0, 1)" "asm"="movss {t}, {off}({n})" "role"="store.s" "nodecode" {
  %a = getelementptr i8, ptr %n, i64 %off
  store float %t, ptr %a
  ret void
}

define float @MOVSSfromi32(i32 "reg"="r32" %n) "enc"="01100110 | rex(0, d, 0, n) | 00001111 | 01101110 | rr(d, n)" "asm"="movd {n}, {d}" "role"="move.s.r32" "nodecode" {
  %r = bitcast i32 %n to float
  ret float %r
}

define i32 @MOVi32fromSS(float "reg"="s" %n) "enc"="01100110 | rex(0, n, 0, d) | 00001111 | 01111110 | rr(n, d)" "asm"="movd {n}, {d}" "role"="move.r32.s" "nodecode" {
  %r = bitcast float %n to i32
  ret i32 %r
}

define float @CVTSITOFPI32SS(i32 "reg"="r32" %n) "enc"="11110011 | rex(0, d, 0, n) | 00001111 | 00101010 | rr(d, n)" "asm"="cvtsi2ss {n}, {d}" "nodecode" {
  %r = sitofp i32 %n to float
  ret float %r
}

define float @CVTSITOFPI64SS(i64 "reg"="r64" %n) "enc"="11110011 | rex(1, d, 0, n) | 00001111 | 00101010 | rr(d, n)" "asm"="cvtsi2ss {n}, {d}" "nodecode" {
  %r = sitofp i64 %n to float
  ret float %r
}

define float @CVTUITOFPI32SS(i32 "reg"="r32" %n) "enc"="11110011 | rex(1, d, 0, n) | 00001111 | 00101010 | rr(d, n)" "asm"="cvtsi2ss {n:r64}, {d}" "nodecode" {
  %r = uitofp i32 %n to float
  ret float %r
}

define i32 @CVTTFPTOSISSI32(float "reg"="s" %n) "enc"="11110011 | rex(0, d, 0, n) | 00001111 | 00101100 | rr(d, n)" "asm"="cvttss2si {n}, {d}" "dclass"="r32" "nodecode" {
  %r = fptosi float %n to i32
  ret i32 %r
}

define i64 @CVTTFPTOSISSI64(float "reg"="s" %n) "enc"="11110011 | rex(1, d, 0, n) | 00001111 | 00101100 | rr(d, n)" "asm"="cvttss2si {n}, {d}" "dclass"="r64" "nodecode" {
  %r = fptosi float %n to i64
  ret i64 %r
}

define i32 @CVTTFPTOUISSI32(float "reg"="s" %n) "enc"="11110011 | rex(1, d, 0, n) | 00001111 | 00101100 | rr(d, n)" "asm"="cvttss2si {n}, {d:r64}" "dclass"="r32" "nodecode" {
  %r = fptoui float %n to i32
  ret i32 %r
}

define float @select.s(i1 "reg"="r32" %c, float "reg"="s" %a, float "reg"="s" %b) "recipe"="select.s.expand" {
  %r = select i1 %c, float %a, float %b
  ret float %r
}

define float @select.s.expand(i1 %c, float %a, float %b) {
  %ai = call i32 @MOVi32fromSS(float %a)
  %bi = call i32 @MOVi32fromSS(float %b)
  %f = call %flags @TESTB1(i1 %c)
  %ne = call i1 @cc.ne(%flags %f)
  %ri = call i32 @CMOV32(i1 %ne, i32 %ai, i32 %bi)
  %r = call float @MOVSSfromi32(i32 %ri)
  ret float %r
}

define float @fneg.s(float "reg"="s" %a) "recipe"="fneg.s.expand" {
  %r = fneg float %a
  ret float %r
}

define float @fneg.s.expand(float %a) {
  %ai = call i32 @MOVi32fromSS(float %a)
  %k = call i32 @MOV32ri(i32 -2147483648)
  %x = call i32 @XOR32rr(i32 %ai, i32 %k)
  %r = call float @MOVSSfromi32(i32 %x)
  ret float %r
}

define float @fabs.s(float "reg"="s" %a) "recipe"="fabs.s.expand" {
  %r = call float @llvm.fabs.f32(float %a)
  ret float %r
}

define float @fabs.s.expand(float %a) {
  %ai = call i32 @MOVi32fromSS(float %a)
  %k = call i32 @MOV32ri(i32 2147483647)
  %x = call i32 @AND32rr(i32 %ai, i32 %k)
  %r = call float @MOVSSfromi32(i32 %x)
  ret float %r
}

declare float @llvm.fabs.f32(float)

declare float @llvm.fmuladd.f32(float, float, float)

define float @fmuladd.s(float "reg"="s" %a, float "reg"="s" %b, float "reg"="s" %c) "recipe"="fmuladd.s.expand" {
  %r = call float @llvm.fmuladd.f32(float %a, float %b, float %c)
  ret float %r
}

define float @fmuladd.s.expand(float %a, float %b, float %c) {
  %p = call float @MULSS(float %a, float %b)
  %r = call float @ADDSS(float %p, float %c)
  ret float %r
}

define float @uitofp.64.s(i64 "reg"="r64" %x) "recipe"="uitofp.64.s.expand" {
  %r = uitofp i64 %x to float
  ret float %r
}

define float @uitofp.64.s.expand(i64 %x) {
  %h = call i64 @SHR64ri(i64 %x, i64 1)
  %l = call i64 @AND64ri(i64 %x, i64 1)
  %o = call i64 @OR64rr(i64 %h, i64 %l)
  %f1 = call %flags @TEST64rr(i64 %x, i64 %x)
  %n1 = call i1 @cc.l(%flags %f1)
  %t = call i64 @CMOV64(i1 %n1, i64 %o, i64 %x)
  %c = call float @CVTSITOFPI64SS(i64 %t)
  %c2 = call float @ADDSS(float %c, float %c)
  %ci = call i32 @MOVi32fromSS(float %c)
  %c2i = call i32 @MOVi32fromSS(float %c2)
  %f2 = call %flags @TEST64rr(i64 %x, i64 %x)
  %n2 = call i1 @cc.l(%flags %f2)
  %ri = call i32 @CMOV32(i1 %n2, i32 %c2i, i32 %ci)
  %r = call float @MOVSSfromi32(i32 %ri)
  ret float %r
}

define i64 @fptoui.64.s(float "reg"="s" %x) "recipe"="fptoui.64.s.expand" {
  %r = fptoui float %x to i64
  ret i64 %r
}

define i64 @fptoui.64.s.expand(float %x) {
  %a = call i64 @CVTTFPTOSISSI64(float %x)
  %ki = call i32 @MOV32ri(i32 1593835520)
  %k = call float @MOVSSfromi32(i32 %ki)
  %y = call float @SUBSS(float %x, float %k)
  %b = call i64 @CVTTFPTOSISSI64(float %y)
  %s = call i64 @SAR64ri(i64 %a, i64 63)
  %m = call i64 @AND64rr(i64 %b, i64 %s)
  %r = call i64 @OR64rr(i64 %a, i64 %m)
  ret i64 %r
}

define float @MOVDh(i16 "reg"="r32" %n) "enc"="01100110 | rex(0, d, 0, n) | 00001111 | 01101110 | rr(d, n)" "asm"="movd {n}, {d}" "role"="move.h.w" "nomatch" "nodecode" {
  %r = bitcast i32 0 to float
  ret float %r
}

define i16 @PEXTRWh(float "reg"="s" %n) "enc"="01100110 | rex(0, d, 0, n) | 00001111 | 11000101 | rr(d, n) | 00000000" "asm"="pextrw $0, {n}, {d}" "dclass"="r32" "zext"="16" "role"="move.w.h" "nomatch" "nodecode" {
  ret i16 0
}

define double @CVTSS2SD(float "reg"="s" %n) "enc"="11110011 | rex(0, d, 0, n) | 00001111 | 01011010 | rr(d, n)" "asm"="cvtss2sd {n}, {d}" "nodecode" {
  %r = fpext float %n to double
  ret double %r
}

define float @CVTSD2SS(double "reg"="d" %n) "enc"="11110010 | rex(0, d, 0, n) | 00001111 | 01011010 | rr(d, n)" "asm"="cvtsd2ss {n}, {d}" "nodecode" {
  %r = fptrunc double %n to float
  ret float %r
}

define i64 @ADD64rm(i64 "reg"="r64" "tied" %x, ptr "reg"="r64" %n, i64 "imm" %off) "enc"="rex(1, d, 4, n) | 00000011 | rm(d, n, 4, 1, off, 0, 1)" "asm"="addq {off}({n}), {d}" "clobbers"="flags" "nodecode" {
  %a = getelementptr i8, ptr %n, i64 %off
  %v = load i64, ptr %a
  %r = add i64 %x, %v
  ret i64 %r
}

define i64 @OR64rm(i64 "reg"="r64" "tied" %x, ptr "reg"="r64" %n, i64 "imm" %off) "enc"="rex(1, d, 4, n) | 00001011 | rm(d, n, 4, 1, off, 0, 1)" "asm"="orq {off}({n}), {d}" "clobbers"="flags" "nodecode" {
  %a = getelementptr i8, ptr %n, i64 %off
  %v = load i64, ptr %a
  %r = or i64 %x, %v
  ret i64 %r
}

define i64 @AND64rm(i64 "reg"="r64" "tied" %x, ptr "reg"="r64" %n, i64 "imm" %off) "enc"="rex(1, d, 4, n) | 00100011 | rm(d, n, 4, 1, off, 0, 1)" "asm"="andq {off}({n}), {d}" "clobbers"="flags" "nodecode" {
  %a = getelementptr i8, ptr %n, i64 %off
  %v = load i64, ptr %a
  %r = and i64 %x, %v
  ret i64 %r
}

define i64 @SUB64rm(i64 "reg"="r64" "tied" %x, ptr "reg"="r64" %n, i64 "imm" %off) "enc"="rex(1, d, 4, n) | 00101011 | rm(d, n, 4, 1, off, 0, 1)" "asm"="subq {off}({n}), {d}" "clobbers"="flags" "nodecode" {
  %a = getelementptr i8, ptr %n, i64 %off
  %v = load i64, ptr %a
  %r = sub i64 %x, %v
  ret i64 %r
}

define i64 @XOR64rm(i64 "reg"="r64" "tied" %x, ptr "reg"="r64" %n, i64 "imm" %off) "enc"="rex(1, d, 4, n) | 00110011 | rm(d, n, 4, 1, off, 0, 1)" "asm"="xorq {off}({n}), {d}" "clobbers"="flags" "nodecode" {
  %a = getelementptr i8, ptr %n, i64 %off
  %v = load i64, ptr %a
  %r = xor i64 %x, %v
  ret i64 %r
}

define i64 @IMUL64rm(i64 "reg"="r64" "tied" %x, ptr "reg"="r64" %n, i64 "imm" %off) "enc"="rex(1, d, 4, n) | 00001111 | 10101111 | rm(d, n, 4, 1, off, 0, 1)" "asm"="imulq {off}({n}), {d}" "clobbers"="flags" "nodecode" {
  %a = getelementptr i8, ptr %n, i64 %off
  %v = load i64, ptr %a
  %r = mul i64 %x, %v
  ret i64 %r
}

define %flags @CMP64rm(i64 "reg"="r64" %x, ptr "reg"="r64" %n, i64 "imm" %off) "enc"="rex(1, x, 4, n) | 00111011 | rm(x, n, 4, 1, off, 0, 1)" "asm"="cmpq {off}({n}), {x}" "sets"="zcso" "nodecode" {
  %a = getelementptr i8, ptr %n, i64 %off
  %v = load i64, ptr %a
  %f0 = insertvalue %flags poison, i64 %x, 0
  %f = insertvalue %flags %f0, i64 %v, 1
  ret %flags %f
}

define %flags @CMP64mi(ptr "reg"="r64" %n, i64 "imm" %off, i64 "imm" %imm) "enc"="{ rex(1, 0, 4, n) | 10000011 | rm(7, n, 4, 1, off, 0, 1) | ib:s8=imm ; rex(1, 0, 4, n) | 10000001 | rm(7, n, 4, 1, off, 0, 1) | iv:s32=imm }" "asm"="cmpq ${imm}, {off}({n})" "sets"="zcso" "nodecode" {
  %a = getelementptr i8, ptr %n, i64 %off
  %v = load i64, ptr %a
  %f0 = insertvalue %flags poison, i64 %v, 0
  %f = insertvalue %flags %f0, i64 %imm, 1
  ret %flags %f
}

define i32 @ADD32rm(i32 "reg"="r32" "tied" %x, ptr "reg"="r64" %n, i64 "imm" %off) "enc"="rex(0, d, 4, n) | 00000011 | rm(d, n, 4, 1, off, 0, 1)" "asm"="addl {off}({n}), {d}" "clobbers"="flags" "zext"="32" "nodecode" {
  %a = getelementptr i8, ptr %n, i64 %off
  %v = load i32, ptr %a
  %r = add i32 %x, %v
  ret i32 %r
}

define i32 @OR32rm(i32 "reg"="r32" "tied" %x, ptr "reg"="r64" %n, i64 "imm" %off) "enc"="rex(0, d, 4, n) | 00001011 | rm(d, n, 4, 1, off, 0, 1)" "asm"="orl {off}({n}), {d}" "clobbers"="flags" "zext"="32" "nodecode" {
  %a = getelementptr i8, ptr %n, i64 %off
  %v = load i32, ptr %a
  %r = or i32 %x, %v
  ret i32 %r
}

define i32 @AND32rm(i32 "reg"="r32" "tied" %x, ptr "reg"="r64" %n, i64 "imm" %off) "enc"="rex(0, d, 4, n) | 00100011 | rm(d, n, 4, 1, off, 0, 1)" "asm"="andl {off}({n}), {d}" "clobbers"="flags" "zext"="32" "nodecode" {
  %a = getelementptr i8, ptr %n, i64 %off
  %v = load i32, ptr %a
  %r = and i32 %x, %v
  ret i32 %r
}

define i32 @SUB32rm(i32 "reg"="r32" "tied" %x, ptr "reg"="r64" %n, i64 "imm" %off) "enc"="rex(0, d, 4, n) | 00101011 | rm(d, n, 4, 1, off, 0, 1)" "asm"="subl {off}({n}), {d}" "clobbers"="flags" "zext"="32" "nodecode" {
  %a = getelementptr i8, ptr %n, i64 %off
  %v = load i32, ptr %a
  %r = sub i32 %x, %v
  ret i32 %r
}

define i32 @XOR32rm(i32 "reg"="r32" "tied" %x, ptr "reg"="r64" %n, i64 "imm" %off) "enc"="rex(0, d, 4, n) | 00110011 | rm(d, n, 4, 1, off, 0, 1)" "asm"="xorl {off}({n}), {d}" "clobbers"="flags" "zext"="32" "nodecode" {
  %a = getelementptr i8, ptr %n, i64 %off
  %v = load i32, ptr %a
  %r = xor i32 %x, %v
  ret i32 %r
}

define i32 @IMUL32rm(i32 "reg"="r32" "tied" %x, ptr "reg"="r64" %n, i64 "imm" %off) "enc"="rex(0, d, 4, n) | 00001111 | 10101111 | rm(d, n, 4, 1, off, 0, 1)" "asm"="imull {off}({n}), {d}" "clobbers"="flags" "zext"="32" "nodecode" {
  %a = getelementptr i8, ptr %n, i64 %off
  %v = load i32, ptr %a
  %r = mul i32 %x, %v
  ret i32 %r
}

define %flags @CMP32rm(i32 "reg"="r32" %x, ptr "reg"="r64" %n, i64 "imm" %off) "enc"="rex(0, x, 4, n) | 00111011 | rm(x, n, 4, 1, off, 0, 1)" "asm"="cmpl {off}({n}), {x}" "sets"="zcso" "nodecode" {
  %a = getelementptr i8, ptr %n, i64 %off
  %v = load i32, ptr %a
  %ea = sext i32 %x to i64
  %eb = sext i32 %v to i64
  %f0 = insertvalue %flags poison, i64 %ea, 0
  %f = insertvalue %flags %f0, i64 %eb, 1
  ret %flags %f
}

define %flags @CMP32mi(ptr "reg"="r64" %n, i64 "imm" %off, i32 "imm" %imm) "enc"="{ rex(0, 0, 4, n) | 10000011 | rm(7, n, 4, 1, off, 0, 1) | ib:s8=imm ; rex(0, 0, 4, n) | 10000001 | rm(7, n, 4, 1, off, 0, 1) | iv:s32=imm }" "asm"="cmpl ${imm}, {off}({n})" "sets"="zcso" "nodecode" {
  %a = getelementptr i8, ptr %n, i64 %off
  %v = load i32, ptr %a
  %ea = sext i32 %v to i64
  %eb = sext i32 %imm to i64
  %f0 = insertvalue %flags poison, i64 %ea, 0
  %f = insertvalue %flags %f0, i64 %eb, 1
  ret %flags %f
}

define double @ADDSDrm(double "reg"="d" "tied" %x, ptr "reg"="r64" %n, i64 "imm" %off) "enc"="11110010 | rex(0, d, 4, n) | 00001111 | 01011000 | rm(d, n, 4, 1, off, 0, 1)" "asm"="addsd {off}({n}), {d}" "dclass"="d" "nodecode" {
  %a = getelementptr i8, ptr %n, i64 %off
  %v = load double, ptr %a
  %r = fadd double %x, %v
  ret double %r
}

define double @SUBSDrm(double "reg"="d" "tied" %x, ptr "reg"="r64" %n, i64 "imm" %off) "enc"="11110010 | rex(0, d, 4, n) | 00001111 | 01011100 | rm(d, n, 4, 1, off, 0, 1)" "asm"="subsd {off}({n}), {d}" "dclass"="d" "nodecode" {
  %a = getelementptr i8, ptr %n, i64 %off
  %v = load double, ptr %a
  %r = fsub double %x, %v
  ret double %r
}

define double @MULSDrm(double "reg"="d" "tied" %x, ptr "reg"="r64" %n, i64 "imm" %off) "enc"="11110010 | rex(0, d, 4, n) | 00001111 | 01011001 | rm(d, n, 4, 1, off, 0, 1)" "asm"="mulsd {off}({n}), {d}" "dclass"="d" "nodecode" {
  %a = getelementptr i8, ptr %n, i64 %off
  %v = load double, ptr %a
  %r = fmul double %x, %v
  ret double %r
}

define double @DIVSDrm(double "reg"="d" "tied" %x, ptr "reg"="r64" %n, i64 "imm" %off) "enc"="11110010 | rex(0, d, 4, n) | 00001111 | 01011110 | rm(d, n, 4, 1, off, 0, 1)" "asm"="divsd {off}({n}), {d}" "dclass"="d" "nodecode" {
  %a = getelementptr i8, ptr %n, i64 %off
  %v = load double, ptr %a
  %r = fdiv double %x, %v
  ret double %r
}

define float @ADDSSrm(float "reg"="s" "tied" %x, ptr "reg"="r64" %n, i64 "imm" %off) "enc"="11110011 | rex(0, d, 4, n) | 00001111 | 01011000 | rm(d, n, 4, 1, off, 0, 1)" "asm"="addss {off}({n}), {d}" "dclass"="s" "nodecode" {
  %a = getelementptr i8, ptr %n, i64 %off
  %v = load float, ptr %a
  %r = fadd float %x, %v
  ret float %r
}

define float @SUBSSrm(float "reg"="s" "tied" %x, ptr "reg"="r64" %n, i64 "imm" %off) "enc"="11110011 | rex(0, d, 4, n) | 00001111 | 01011100 | rm(d, n, 4, 1, off, 0, 1)" "asm"="subss {off}({n}), {d}" "dclass"="s" "nodecode" {
  %a = getelementptr i8, ptr %n, i64 %off
  %v = load float, ptr %a
  %r = fsub float %x, %v
  ret float %r
}

define float @MULSSrm(float "reg"="s" "tied" %x, ptr "reg"="r64" %n, i64 "imm" %off) "enc"="11110011 | rex(0, d, 4, n) | 00001111 | 01011001 | rm(d, n, 4, 1, off, 0, 1)" "asm"="mulss {off}({n}), {d}" "dclass"="s" "nodecode" {
  %a = getelementptr i8, ptr %n, i64 %off
  %v = load float, ptr %a
  %r = fmul float %x, %v
  ret float %r
}

define float @DIVSSrm(float "reg"="s" "tied" %x, ptr "reg"="r64" %n, i64 "imm" %off) "enc"="11110011 | rex(0, d, 4, n) | 00001111 | 01011110 | rm(d, n, 4, 1, off, 0, 1)" "asm"="divss {off}({n}), {d}" "dclass"="s" "nodecode" {
  %a = getelementptr i8, ptr %n, i64 %off
  %v = load float, ptr %a
  %r = fdiv float %x, %v
  ret float %r
}

define i8 @LDAI8(ptr "reg"="r64" %n) "enc"="rex(0, d, 4, n) | 00001111 | 10110110 | rm(d, n, 4, 1, 0, 0, 1)" "asm"="movzbl 0({n}), {d:r32}" "dclass"="r32" "nodecode" "zext"="8" {
  %r = load atomic i8, ptr %n acquire, align 1
  ret i8 %r
}

define void @STRI8(i8 "reg"="r32" %t, ptr "reg"="r64" %n) "enc"="{ if(t<4) rex(0, t, 4, n) ; 0100 0 R:1=(t)>>3 X:1=0 B:1=(n)>>3 } | 10001000 | rm(t, n, 4, 1, 0, 0, 1)" "asm"="movb {t:r8}, 0({n})" "nodecode" {
  store atomic i8 %t, ptr %n release, align 1
  ret void
}

define i8 @XCHGI80(i8 "reg"="r32" "tied" %m, ptr "reg"="r64" %n) "enc"="{ if(d<4) rex(0, d, 4, n) ; 0100 0 R:1=(d)>>3 X:1=0 B:1=(n)>>3 } | 10000110 | rm(d, n, 4, 1, 0, 0, 1)" "asm"="xchgb {d:r8}, 0({n})" "nodecode" {
  %r = atomicrmw xchg ptr %n, i8 %m monotonic
  ret i8 %r
}

define i8 @XADDI80(i8 "reg"="r32" "tied" %m, ptr "reg"="r64" %n) "enc"="11110000 | { if(d<4) rex(0, d, 4, n) ; 0100 0 R:1=(d)>>3 X:1=0 B:1=(n)>>3 } | 00001111 | 11000000 | rm(d, n, 4, 1, 0, 0, 1)" "asm"="lock xaddb {d:r8}, 0({n})" "clobbers"="flags" "nodecode" {
  %r = atomicrmw add ptr %n, i8 %m monotonic
  ret i8 %r
}

define i8 @CMPXCHGI80(i8 "reg"="rax" "tied" %old, i8 "reg"="r32" %new, ptr "reg"="r64" %n) "enc"="11110000 | { if(new<4) rex(0, new, 4, n) ; 0100 0 R:1=(new)>>3 X:1=0 B:1=(n)>>3 } | 00001111 | 10110000 | rm(new, n, 4, 1, 0, 0, 1)" "asm"="lock cmpxchgb {new:r8}, 0({n})" "dclass"="rax" "clobbers"="flags" "nodecode" {
  %r = call i8 @llvm.epitome.cmpxchg.i8(ptr %n, i8 %old, i8 %new, i32 0)
  ret i8 %r
}

define i8 @XCHGI81(i8 "reg"="r32" "tied" %m, ptr "reg"="r64" %n) "enc"="{ if(d<4) rex(0, d, 4, n) ; 0100 0 R:1=(d)>>3 X:1=0 B:1=(n)>>3 } | 10000110 | rm(d, n, 4, 1, 0, 0, 1)" "asm"="xchgb {d:r8}, 0({n})" "nodecode" {
  %r = atomicrmw xchg ptr %n, i8 %m acquire
  ret i8 %r
}

define i8 @XADDI81(i8 "reg"="r32" "tied" %m, ptr "reg"="r64" %n) "enc"="11110000 | { if(d<4) rex(0, d, 4, n) ; 0100 0 R:1=(d)>>3 X:1=0 B:1=(n)>>3 } | 00001111 | 11000000 | rm(d, n, 4, 1, 0, 0, 1)" "asm"="lock xaddb {d:r8}, 0({n})" "clobbers"="flags" "nodecode" {
  %r = atomicrmw add ptr %n, i8 %m acquire
  ret i8 %r
}

define i8 @CMPXCHGI81(i8 "reg"="rax" "tied" %old, i8 "reg"="r32" %new, ptr "reg"="r64" %n) "enc"="11110000 | { if(new<4) rex(0, new, 4, n) ; 0100 0 R:1=(new)>>3 X:1=0 B:1=(n)>>3 } | 00001111 | 10110000 | rm(new, n, 4, 1, 0, 0, 1)" "asm"="lock cmpxchgb {new:r8}, 0({n})" "dclass"="rax" "clobbers"="flags" "nodecode" {
  %r = call i8 @llvm.epitome.cmpxchg.i8(ptr %n, i8 %old, i8 %new, i32 1)
  ret i8 %r
}

define i8 @XCHGI82(i8 "reg"="r32" "tied" %m, ptr "reg"="r64" %n) "enc"="{ if(d<4) rex(0, d, 4, n) ; 0100 0 R:1=(d)>>3 X:1=0 B:1=(n)>>3 } | 10000110 | rm(d, n, 4, 1, 0, 0, 1)" "asm"="xchgb {d:r8}, 0({n})" "nodecode" {
  %r = atomicrmw xchg ptr %n, i8 %m release
  ret i8 %r
}

define i8 @XADDI82(i8 "reg"="r32" "tied" %m, ptr "reg"="r64" %n) "enc"="11110000 | { if(d<4) rex(0, d, 4, n) ; 0100 0 R:1=(d)>>3 X:1=0 B:1=(n)>>3 } | 00001111 | 11000000 | rm(d, n, 4, 1, 0, 0, 1)" "asm"="lock xaddb {d:r8}, 0({n})" "clobbers"="flags" "nodecode" {
  %r = atomicrmw add ptr %n, i8 %m release
  ret i8 %r
}

define i8 @CMPXCHGI82(i8 "reg"="rax" "tied" %old, i8 "reg"="r32" %new, ptr "reg"="r64" %n) "enc"="11110000 | { if(new<4) rex(0, new, 4, n) ; 0100 0 R:1=(new)>>3 X:1=0 B:1=(n)>>3 } | 00001111 | 10110000 | rm(new, n, 4, 1, 0, 0, 1)" "asm"="lock cmpxchgb {new:r8}, 0({n})" "dclass"="rax" "clobbers"="flags" "nodecode" {
  %r = call i8 @llvm.epitome.cmpxchg.i8(ptr %n, i8 %old, i8 %new, i32 2)
  ret i8 %r
}

define i8 @XCHGI83(i8 "reg"="r32" "tied" %m, ptr "reg"="r64" %n) "enc"="{ if(d<4) rex(0, d, 4, n) ; 0100 0 R:1=(d)>>3 X:1=0 B:1=(n)>>3 } | 10000110 | rm(d, n, 4, 1, 0, 0, 1)" "asm"="xchgb {d:r8}, 0({n})" "nodecode" {
  %r = atomicrmw xchg ptr %n, i8 %m seq_cst
  ret i8 %r
}

define i8 @XADDI83(i8 "reg"="r32" "tied" %m, ptr "reg"="r64" %n) "enc"="11110000 | { if(d<4) rex(0, d, 4, n) ; 0100 0 R:1=(d)>>3 X:1=0 B:1=(n)>>3 } | 00001111 | 11000000 | rm(d, n, 4, 1, 0, 0, 1)" "asm"="lock xaddb {d:r8}, 0({n})" "clobbers"="flags" "nodecode" {
  %r = atomicrmw add ptr %n, i8 %m seq_cst
  ret i8 %r
}

define i8 @CMPXCHGI83(i8 "reg"="rax" "tied" %old, i8 "reg"="r32" %new, ptr "reg"="r64" %n) "enc"="11110000 | { if(new<4) rex(0, new, 4, n) ; 0100 0 R:1=(new)>>3 X:1=0 B:1=(n)>>3 } | 00001111 | 10110000 | rm(new, n, 4, 1, 0, 0, 1)" "asm"="lock cmpxchgb {new:r8}, 0({n})" "dclass"="rax" "clobbers"="flags" "nodecode" {
  %r = call i8 @llvm.epitome.cmpxchg.i8(ptr %n, i8 %old, i8 %new, i32 3)
  ret i8 %r
}

declare i8 @llvm.epitome.cmpxchg.i8(ptr, i8, i8, i32)

define i16 @LDAI16(ptr "reg"="r64" %n) "enc"="rex(0, d, 4, n) | 00001111 | 10110111 | rm(d, n, 4, 1, 0, 0, 1)" "asm"="movzwl 0({n}), {d:r32}" "dclass"="r32" "nodecode" "zext"="16" {
  %r = load atomic i16, ptr %n acquire, align 2
  ret i16 %r
}

define void @STRI16(i16 "reg"="r32" %t, ptr "reg"="r64" %n) "enc"="01100110 | rex(0, t, 4, n) | 10001001 | rm(t, n, 4, 1, 0, 0, 1)" "asm"="movw {t:r16}, 0({n})" "nodecode" {
  store atomic i16 %t, ptr %n release, align 2
  ret void
}

define i16 @XCHGI160(i16 "reg"="r32" "tied" %m, ptr "reg"="r64" %n) "enc"="01100110 | rex(0, d, 4, n) | 10000111 | rm(d, n, 4, 1, 0, 0, 1)" "asm"="xchgw {d:r16}, 0({n})" "nodecode" {
  %r = atomicrmw xchg ptr %n, i16 %m monotonic
  ret i16 %r
}

define i16 @XADDI160(i16 "reg"="r32" "tied" %m, ptr "reg"="r64" %n) "enc"="01100110 | 11110000 | rex(0, d, 4, n) | 00001111 | 11000001 | rm(d, n, 4, 1, 0, 0, 1)" "asm"="lock xaddw {d:r16}, 0({n})" "clobbers"="flags" "nodecode" {
  %r = atomicrmw add ptr %n, i16 %m monotonic
  ret i16 %r
}

define i16 @CMPXCHGI160(i16 "reg"="rax" "tied" %old, i16 "reg"="r32" %new, ptr "reg"="r64" %n) "enc"="01100110 | 11110000 | rex(0, new, 4, n) | 00001111 | 10110001 | rm(new, n, 4, 1, 0, 0, 1)" "asm"="lock cmpxchgw {new:r16}, 0({n})" "dclass"="rax" "clobbers"="flags" "nodecode" {
  %r = call i16 @llvm.epitome.cmpxchg.i16(ptr %n, i16 %old, i16 %new, i32 0)
  ret i16 %r
}

define i16 @XCHGI161(i16 "reg"="r32" "tied" %m, ptr "reg"="r64" %n) "enc"="01100110 | rex(0, d, 4, n) | 10000111 | rm(d, n, 4, 1, 0, 0, 1)" "asm"="xchgw {d:r16}, 0({n})" "nodecode" {
  %r = atomicrmw xchg ptr %n, i16 %m acquire
  ret i16 %r
}

define i16 @XADDI161(i16 "reg"="r32" "tied" %m, ptr "reg"="r64" %n) "enc"="01100110 | 11110000 | rex(0, d, 4, n) | 00001111 | 11000001 | rm(d, n, 4, 1, 0, 0, 1)" "asm"="lock xaddw {d:r16}, 0({n})" "clobbers"="flags" "nodecode" {
  %r = atomicrmw add ptr %n, i16 %m acquire
  ret i16 %r
}

define i16 @CMPXCHGI161(i16 "reg"="rax" "tied" %old, i16 "reg"="r32" %new, ptr "reg"="r64" %n) "enc"="01100110 | 11110000 | rex(0, new, 4, n) | 00001111 | 10110001 | rm(new, n, 4, 1, 0, 0, 1)" "asm"="lock cmpxchgw {new:r16}, 0({n})" "dclass"="rax" "clobbers"="flags" "nodecode" {
  %r = call i16 @llvm.epitome.cmpxchg.i16(ptr %n, i16 %old, i16 %new, i32 1)
  ret i16 %r
}

define i16 @XCHGI162(i16 "reg"="r32" "tied" %m, ptr "reg"="r64" %n) "enc"="01100110 | rex(0, d, 4, n) | 10000111 | rm(d, n, 4, 1, 0, 0, 1)" "asm"="xchgw {d:r16}, 0({n})" "nodecode" {
  %r = atomicrmw xchg ptr %n, i16 %m release
  ret i16 %r
}

define i16 @XADDI162(i16 "reg"="r32" "tied" %m, ptr "reg"="r64" %n) "enc"="01100110 | 11110000 | rex(0, d, 4, n) | 00001111 | 11000001 | rm(d, n, 4, 1, 0, 0, 1)" "asm"="lock xaddw {d:r16}, 0({n})" "clobbers"="flags" "nodecode" {
  %r = atomicrmw add ptr %n, i16 %m release
  ret i16 %r
}

define i16 @CMPXCHGI162(i16 "reg"="rax" "tied" %old, i16 "reg"="r32" %new, ptr "reg"="r64" %n) "enc"="01100110 | 11110000 | rex(0, new, 4, n) | 00001111 | 10110001 | rm(new, n, 4, 1, 0, 0, 1)" "asm"="lock cmpxchgw {new:r16}, 0({n})" "dclass"="rax" "clobbers"="flags" "nodecode" {
  %r = call i16 @llvm.epitome.cmpxchg.i16(ptr %n, i16 %old, i16 %new, i32 2)
  ret i16 %r
}

define i16 @XCHGI163(i16 "reg"="r32" "tied" %m, ptr "reg"="r64" %n) "enc"="01100110 | rex(0, d, 4, n) | 10000111 | rm(d, n, 4, 1, 0, 0, 1)" "asm"="xchgw {d:r16}, 0({n})" "nodecode" {
  %r = atomicrmw xchg ptr %n, i16 %m seq_cst
  ret i16 %r
}

define i16 @XADDI163(i16 "reg"="r32" "tied" %m, ptr "reg"="r64" %n) "enc"="01100110 | 11110000 | rex(0, d, 4, n) | 00001111 | 11000001 | rm(d, n, 4, 1, 0, 0, 1)" "asm"="lock xaddw {d:r16}, 0({n})" "clobbers"="flags" "nodecode" {
  %r = atomicrmw add ptr %n, i16 %m seq_cst
  ret i16 %r
}

define i16 @CMPXCHGI163(i16 "reg"="rax" "tied" %old, i16 "reg"="r32" %new, ptr "reg"="r64" %n) "enc"="01100110 | 11110000 | rex(0, new, 4, n) | 00001111 | 10110001 | rm(new, n, 4, 1, 0, 0, 1)" "asm"="lock cmpxchgw {new:r16}, 0({n})" "dclass"="rax" "clobbers"="flags" "nodecode" {
  %r = call i16 @llvm.epitome.cmpxchg.i16(ptr %n, i16 %old, i16 %new, i32 3)
  ret i16 %r
}

declare i16 @llvm.epitome.cmpxchg.i16(ptr, i16, i16, i32)

define i32 @LDAI32(ptr "reg"="r64" %n) "enc"="rex(0, d, 4, n) | 10001011 | rm(d, n, 4, 1, 0, 0, 1)" "asm"="movl 0({n}), {d}" "dclass"="r32" "nodecode" {
  %r = load atomic i32, ptr %n acquire, align 4
  ret i32 %r
}

define void @STRI32(i32 "reg"="r32" %t, ptr "reg"="r64" %n) "enc"="rex(0, t, 4, n) | 10001001 | rm(t, n, 4, 1, 0, 0, 1)" "asm"="movl {t}, 0({n})" "nodecode" {
  store atomic i32 %t, ptr %n release, align 4
  ret void
}

define i32 @XCHGI320(i32 "reg"="r32" "tied" %m, ptr "reg"="r64" %n) "enc"="rex(0, d, 4, n) | 10000111 | rm(d, n, 4, 1, 0, 0, 1)" "asm"="xchgl {d}, 0({n})" "nodecode" {
  %r = atomicrmw xchg ptr %n, i32 %m monotonic
  ret i32 %r
}

define i32 @XADDI320(i32 "reg"="r32" "tied" %m, ptr "reg"="r64" %n) "enc"="11110000 | rex(0, d, 4, n) | 00001111 | 11000001 | rm(d, n, 4, 1, 0, 0, 1)" "asm"="lock xaddl {d}, 0({n})" "clobbers"="flags" "nodecode" {
  %r = atomicrmw add ptr %n, i32 %m monotonic
  ret i32 %r
}

define i32 @CMPXCHGI320(i32 "reg"="rax" "tied" %old, i32 "reg"="r32" %new, ptr "reg"="r64" %n) "enc"="11110000 | rex(0, new, 4, n) | 00001111 | 10110001 | rm(new, n, 4, 1, 0, 0, 1)" "asm"="lock cmpxchgl {new}, 0({n})" "dclass"="rax" "clobbers"="flags" "nodecode" {
  %r = call i32 @llvm.epitome.cmpxchg.i32(ptr %n, i32 %old, i32 %new, i32 0)
  ret i32 %r
}

define i32 @XCHGI321(i32 "reg"="r32" "tied" %m, ptr "reg"="r64" %n) "enc"="rex(0, d, 4, n) | 10000111 | rm(d, n, 4, 1, 0, 0, 1)" "asm"="xchgl {d}, 0({n})" "nodecode" {
  %r = atomicrmw xchg ptr %n, i32 %m acquire
  ret i32 %r
}

define i32 @XADDI321(i32 "reg"="r32" "tied" %m, ptr "reg"="r64" %n) "enc"="11110000 | rex(0, d, 4, n) | 00001111 | 11000001 | rm(d, n, 4, 1, 0, 0, 1)" "asm"="lock xaddl {d}, 0({n})" "clobbers"="flags" "nodecode" {
  %r = atomicrmw add ptr %n, i32 %m acquire
  ret i32 %r
}

define i32 @CMPXCHGI321(i32 "reg"="rax" "tied" %old, i32 "reg"="r32" %new, ptr "reg"="r64" %n) "enc"="11110000 | rex(0, new, 4, n) | 00001111 | 10110001 | rm(new, n, 4, 1, 0, 0, 1)" "asm"="lock cmpxchgl {new}, 0({n})" "dclass"="rax" "clobbers"="flags" "nodecode" {
  %r = call i32 @llvm.epitome.cmpxchg.i32(ptr %n, i32 %old, i32 %new, i32 1)
  ret i32 %r
}

define i32 @XCHGI322(i32 "reg"="r32" "tied" %m, ptr "reg"="r64" %n) "enc"="rex(0, d, 4, n) | 10000111 | rm(d, n, 4, 1, 0, 0, 1)" "asm"="xchgl {d}, 0({n})" "nodecode" {
  %r = atomicrmw xchg ptr %n, i32 %m release
  ret i32 %r
}

define i32 @XADDI322(i32 "reg"="r32" "tied" %m, ptr "reg"="r64" %n) "enc"="11110000 | rex(0, d, 4, n) | 00001111 | 11000001 | rm(d, n, 4, 1, 0, 0, 1)" "asm"="lock xaddl {d}, 0({n})" "clobbers"="flags" "nodecode" {
  %r = atomicrmw add ptr %n, i32 %m release
  ret i32 %r
}

define i32 @CMPXCHGI322(i32 "reg"="rax" "tied" %old, i32 "reg"="r32" %new, ptr "reg"="r64" %n) "enc"="11110000 | rex(0, new, 4, n) | 00001111 | 10110001 | rm(new, n, 4, 1, 0, 0, 1)" "asm"="lock cmpxchgl {new}, 0({n})" "dclass"="rax" "clobbers"="flags" "nodecode" {
  %r = call i32 @llvm.epitome.cmpxchg.i32(ptr %n, i32 %old, i32 %new, i32 2)
  ret i32 %r
}

define i32 @XCHGI323(i32 "reg"="r32" "tied" %m, ptr "reg"="r64" %n) "enc"="rex(0, d, 4, n) | 10000111 | rm(d, n, 4, 1, 0, 0, 1)" "asm"="xchgl {d}, 0({n})" "nodecode" {
  %r = atomicrmw xchg ptr %n, i32 %m seq_cst
  ret i32 %r
}

define i32 @XADDI323(i32 "reg"="r32" "tied" %m, ptr "reg"="r64" %n) "enc"="11110000 | rex(0, d, 4, n) | 00001111 | 11000001 | rm(d, n, 4, 1, 0, 0, 1)" "asm"="lock xaddl {d}, 0({n})" "clobbers"="flags" "nodecode" {
  %r = atomicrmw add ptr %n, i32 %m seq_cst
  ret i32 %r
}

define i32 @CMPXCHGI323(i32 "reg"="rax" "tied" %old, i32 "reg"="r32" %new, ptr "reg"="r64" %n) "enc"="11110000 | rex(0, new, 4, n) | 00001111 | 10110001 | rm(new, n, 4, 1, 0, 0, 1)" "asm"="lock cmpxchgl {new}, 0({n})" "dclass"="rax" "clobbers"="flags" "nodecode" {
  %r = call i32 @llvm.epitome.cmpxchg.i32(ptr %n, i32 %old, i32 %new, i32 3)
  ret i32 %r
}

declare i32 @llvm.epitome.cmpxchg.i32(ptr, i32, i32, i32)

define i64 @LDAI64(ptr "reg"="r64" %n) "enc"="rex(1, d, 4, n) | 10001011 | rm(d, n, 4, 1, 0, 0, 1)" "asm"="movq 0({n}), {d}" "dclass"="r64" "nodecode" {
  %r = load atomic i64, ptr %n acquire, align 8
  ret i64 %r
}

define void @STRI64(i64 "reg"="r64" %t, ptr "reg"="r64" %n) "enc"="rex(1, t, 4, n) | 10001001 | rm(t, n, 4, 1, 0, 0, 1)" "asm"="movq {t}, 0({n})" "nodecode" {
  store atomic i64 %t, ptr %n release, align 8
  ret void
}

define i64 @XCHGI640(i64 "reg"="r64" "tied" %m, ptr "reg"="r64" %n) "enc"="rex(1, d, 4, n) | 10000111 | rm(d, n, 4, 1, 0, 0, 1)" "asm"="xchgq {d}, 0({n})" "nodecode" {
  %r = atomicrmw xchg ptr %n, i64 %m monotonic
  ret i64 %r
}

define i64 @XADDI640(i64 "reg"="r64" "tied" %m, ptr "reg"="r64" %n) "enc"="11110000 | rex(1, d, 4, n) | 00001111 | 11000001 | rm(d, n, 4, 1, 0, 0, 1)" "asm"="lock xaddq {d}, 0({n})" "clobbers"="flags" "nodecode" {
  %r = atomicrmw add ptr %n, i64 %m monotonic
  ret i64 %r
}

define i64 @CMPXCHGI640(i64 "reg"="rax" "tied" %old, i64 "reg"="r64" %new, ptr "reg"="r64" %n) "enc"="11110000 | rex(1, new, 4, n) | 00001111 | 10110001 | rm(new, n, 4, 1, 0, 0, 1)" "asm"="lock cmpxchgq {new}, 0({n})" "dclass"="rax" "clobbers"="flags" "nodecode" {
  %r = call i64 @llvm.epitome.cmpxchg.i64(ptr %n, i64 %old, i64 %new, i32 0)
  ret i64 %r
}

define i64 @XCHGI641(i64 "reg"="r64" "tied" %m, ptr "reg"="r64" %n) "enc"="rex(1, d, 4, n) | 10000111 | rm(d, n, 4, 1, 0, 0, 1)" "asm"="xchgq {d}, 0({n})" "nodecode" {
  %r = atomicrmw xchg ptr %n, i64 %m acquire
  ret i64 %r
}

define i64 @XADDI641(i64 "reg"="r64" "tied" %m, ptr "reg"="r64" %n) "enc"="11110000 | rex(1, d, 4, n) | 00001111 | 11000001 | rm(d, n, 4, 1, 0, 0, 1)" "asm"="lock xaddq {d}, 0({n})" "clobbers"="flags" "nodecode" {
  %r = atomicrmw add ptr %n, i64 %m acquire
  ret i64 %r
}

define i64 @CMPXCHGI641(i64 "reg"="rax" "tied" %old, i64 "reg"="r64" %new, ptr "reg"="r64" %n) "enc"="11110000 | rex(1, new, 4, n) | 00001111 | 10110001 | rm(new, n, 4, 1, 0, 0, 1)" "asm"="lock cmpxchgq {new}, 0({n})" "dclass"="rax" "clobbers"="flags" "nodecode" {
  %r = call i64 @llvm.epitome.cmpxchg.i64(ptr %n, i64 %old, i64 %new, i32 1)
  ret i64 %r
}

define i64 @XCHGI642(i64 "reg"="r64" "tied" %m, ptr "reg"="r64" %n) "enc"="rex(1, d, 4, n) | 10000111 | rm(d, n, 4, 1, 0, 0, 1)" "asm"="xchgq {d}, 0({n})" "nodecode" {
  %r = atomicrmw xchg ptr %n, i64 %m release
  ret i64 %r
}

define i64 @XADDI642(i64 "reg"="r64" "tied" %m, ptr "reg"="r64" %n) "enc"="11110000 | rex(1, d, 4, n) | 00001111 | 11000001 | rm(d, n, 4, 1, 0, 0, 1)" "asm"="lock xaddq {d}, 0({n})" "clobbers"="flags" "nodecode" {
  %r = atomicrmw add ptr %n, i64 %m release
  ret i64 %r
}

define i64 @CMPXCHGI642(i64 "reg"="rax" "tied" %old, i64 "reg"="r64" %new, ptr "reg"="r64" %n) "enc"="11110000 | rex(1, new, 4, n) | 00001111 | 10110001 | rm(new, n, 4, 1, 0, 0, 1)" "asm"="lock cmpxchgq {new}, 0({n})" "dclass"="rax" "clobbers"="flags" "nodecode" {
  %r = call i64 @llvm.epitome.cmpxchg.i64(ptr %n, i64 %old, i64 %new, i32 2)
  ret i64 %r
}

define i64 @XCHGI643(i64 "reg"="r64" "tied" %m, ptr "reg"="r64" %n) "enc"="rex(1, d, 4, n) | 10000111 | rm(d, n, 4, 1, 0, 0, 1)" "asm"="xchgq {d}, 0({n})" "nodecode" {
  %r = atomicrmw xchg ptr %n, i64 %m seq_cst
  ret i64 %r
}

define i64 @XADDI643(i64 "reg"="r64" "tied" %m, ptr "reg"="r64" %n) "enc"="11110000 | rex(1, d, 4, n) | 00001111 | 11000001 | rm(d, n, 4, 1, 0, 0, 1)" "asm"="lock xaddq {d}, 0({n})" "clobbers"="flags" "nodecode" {
  %r = atomicrmw add ptr %n, i64 %m seq_cst
  ret i64 %r
}

define i64 @CMPXCHGI643(i64 "reg"="rax" "tied" %old, i64 "reg"="r64" %new, ptr "reg"="r64" %n) "enc"="11110000 | rex(1, new, 4, n) | 00001111 | 10110001 | rm(new, n, 4, 1, 0, 0, 1)" "asm"="lock cmpxchgq {new}, 0({n})" "dclass"="rax" "clobbers"="flags" "nodecode" {
  %r = call i64 @llvm.epitome.cmpxchg.i64(ptr %n, i64 %old, i64 %new, i32 3)
  ret i64 %r
}

declare i64 @llvm.epitome.cmpxchg.i64(ptr, i64, i64, i32)

define ptr @LDAP(ptr "reg"="r64" %n) "enc"="rex(1, d, 4, n) | 10001011 | rm(d, n, 4, 1, 0, 0, 1)" "asm"="movq 0({n}), {d}" "nodecode" {
  %r = load atomic ptr, ptr %n acquire, align 8
  ret ptr %r
}

define void @STRP(ptr "reg"="r64" %t, ptr "reg"="r64" %n) "enc"="rex(1, t, 4, n) | 10001001 | rm(t, n, 4, 1, 0, 0, 1)" "asm"="movq {t}, 0({n})" "nodecode" {
  store atomic ptr %t, ptr %n release, align 8
  ret void
}

define void @MFENCE() "enc"="00001111 | 10101110 | 11110000" "asm"="mfence" "nodecode" {
  fence seq_cst
  ret void
}

define void @MFENCEA() "enc"="00001111 | 10101110 | 11110000" "asm"="mfence" "nodecode" {
  fence acquire
  ret void
}

define void @FLDT(ptr "reg"="r64" %n) "enc"="rex(0, 0, 4, n) | 11011011 | rm(5, n, 4, 1, 0, 0, 1)" "asm"="fldt 0({n})" "nodecode" {
  call void @llvm.epitome.x87.fldt(ptr %n)
  ret void
}

declare void @llvm.epitome.x87.fldt(ptr)

define void @FSTPT(ptr "reg"="r64" %n) "enc"="rex(0, 0, 4, n) | 11011011 | rm(7, n, 4, 1, 0, 0, 1)" "asm"="fstpt 0({n})" "nodecode" {
  call void @llvm.epitome.x87.fstpt(ptr %n)
  ret void
}

declare void @llvm.epitome.x87.fstpt(ptr)

define void @FLDL(ptr "reg"="r64" %n) "enc"="rex(0, 0, 4, n) | 11011101 | rm(0, n, 4, 1, 0, 0, 1)" "asm"="fldl 0({n})" "nodecode" {
  call void @llvm.epitome.x87.fldl(ptr %n)
  ret void
}

declare void @llvm.epitome.x87.fldl(ptr)

define void @FSTPL(ptr "reg"="r64" %n) "enc"="rex(0, 0, 4, n) | 11011101 | rm(3, n, 4, 1, 0, 0, 1)" "asm"="fstpl 0({n})" "nodecode" {
  call void @llvm.epitome.x87.fstpl(ptr %n)
  ret void
}

declare void @llvm.epitome.x87.fstpl(ptr)

define void @FLDS(ptr "reg"="r64" %n) "enc"="rex(0, 0, 4, n) | 11011001 | rm(0, n, 4, 1, 0, 0, 1)" "asm"="flds 0({n})" "nodecode" {
  call void @llvm.epitome.x87.flds(ptr %n)
  ret void
}

declare void @llvm.epitome.x87.flds(ptr)

define void @FSTPS(ptr "reg"="r64" %n) "enc"="rex(0, 0, 4, n) | 11011001 | rm(3, n, 4, 1, 0, 0, 1)" "asm"="fstps 0({n})" "nodecode" {
  call void @llvm.epitome.x87.fstps(ptr %n)
  ret void
}

declare void @llvm.epitome.x87.fstps(ptr)

define void @FILDLL(ptr "reg"="r64" %n) "enc"="rex(0, 0, 4, n) | 11011111 | rm(5, n, 4, 1, 0, 0, 1)" "asm"="fildll 0({n})" "nodecode" {
  call void @llvm.epitome.x87.fildll(ptr %n)
  ret void
}

declare void @llvm.epitome.x87.fildll(ptr)

define void @FISTPLL(ptr "reg"="r64" %n) "enc"="rex(0, 0, 4, n) | 11011111 | rm(7, n, 4, 1, 0, 0, 1)" "asm"="fistpll 0({n})" "nodecode" {
  call void @llvm.epitome.x87.fistpll(ptr %n)
  ret void
}

declare void @llvm.epitome.x87.fistpll(ptr)

define void @FNSTCW(ptr "reg"="r64" %n) "enc"="rex(0, 0, 4, n) | 11011001 | rm(7, n, 4, 1, 0, 0, 1)" "asm"="fnstcw 0({n})" "nodecode" {
  call void @llvm.epitome.x87.fnstcw(ptr %n)
  ret void
}

declare void @llvm.epitome.x87.fnstcw(ptr)

define void @FLDCW(ptr "reg"="r64" %n) "enc"="rex(0, 0, 4, n) | 11011001 | rm(5, n, 4, 1, 0, 0, 1)" "asm"="fldcw 0({n})" "nodecode" {
  call void @llvm.epitome.x87.fldcw(ptr %n)
  ret void
}

declare void @llvm.epitome.x87.fldcw(ptr)

define void @FADDP() "enc"="11011110 | 11000001" "asm"="faddp %st, %st(1)" "nodecode" {
  call void @llvm.epitome.x87.fadd()
  ret void
}

declare void @llvm.epitome.x87.fadd()

define void @FSUBP() "enc"="11011110 | 11101001" "asm"="fsubrp %st, %st(1)" "nodecode" {
  call void @llvm.epitome.x87.fsub()
  ret void
}

declare void @llvm.epitome.x87.fsub()

define void @FMULP() "enc"="11011110 | 11001001" "asm"="fmulp %st, %st(1)" "nodecode" {
  call void @llvm.epitome.x87.fmul()
  ret void
}

declare void @llvm.epitome.x87.fmul()

define void @FDIVP() "enc"="11011110 | 11111001" "asm"="fdivrp %st, %st(1)" "nodecode" {
  call void @llvm.epitome.x87.fdiv()
  ret void
}

declare void @llvm.epitome.x87.fdiv()

define void @FCHS() "enc"="11011001 | 11100000" "asm"="fchs" "nodecode" {
  call void @llvm.epitome.x87.fchs()
  ret void
}

declare void @llvm.epitome.x87.fchs()

define void @FABS() "enc"="11011001 | 11100001" "asm"="fabs" "nodecode" {
  call void @llvm.epitome.x87.fabs()
  ret void
}

declare void @llvm.epitome.x87.fabs()

define void @FPOP() "enc"="11011101 | 11011000" "asm"="fstp %st(0)" "nodecode" {
  call void @llvm.epitome.x87.pop()
  ret void
}

declare void @llvm.epitome.x87.pop()

define %fflags @FUCOMPI() "enc"="11011111 | 11101001" "asm"="fucompi %st(1), %st" "sets"="zcp" "nomatch" "nodecode" {
  ret %fflags poison
}

declare i1 @llvm.epitome.x87.cmp.ogt(ptr, ptr)

define i1 @x87.cmp.ogt(ptr "reg"="r64" %a, ptr "reg"="r64" %b) "recipe"="x87.cmp.ogt.expand" {
  %r = call i1 @llvm.epitome.x87.cmp.ogt(ptr %a, ptr %b)
  ret i1 %r
}

define i1 @x87.cmp.ogt.expand(ptr %a, ptr %b) {
  call void @FLDT(ptr %b)
  call void @FLDT(ptr %a)
  %f = call %fflags @FUCOMPI()
  call void @FPOP()
  %c = call i1 @cc.f.ogt(%fflags %f)
  %r = call i1 @SETCC(i1 %c)
  ret i1 %r
}

declare i1 @llvm.epitome.x87.cmp.oge(ptr, ptr)

define i1 @x87.cmp.oge(ptr "reg"="r64" %a, ptr "reg"="r64" %b) "recipe"="x87.cmp.oge.expand" {
  %r = call i1 @llvm.epitome.x87.cmp.oge(ptr %a, ptr %b)
  ret i1 %r
}

define i1 @x87.cmp.oge.expand(ptr %a, ptr %b) {
  call void @FLDT(ptr %b)
  call void @FLDT(ptr %a)
  %f = call %fflags @FUCOMPI()
  call void @FPOP()
  %c = call i1 @cc.f.oge(%fflags %f)
  %r = call i1 @SETCC(i1 %c)
  ret i1 %r
}

declare i1 @llvm.epitome.x87.cmp.ult(ptr, ptr)

define i1 @x87.cmp.ult(ptr "reg"="r64" %a, ptr "reg"="r64" %b) "recipe"="x87.cmp.ult.expand" {
  %r = call i1 @llvm.epitome.x87.cmp.ult(ptr %a, ptr %b)
  ret i1 %r
}

define i1 @x87.cmp.ult.expand(ptr %a, ptr %b) {
  call void @FLDT(ptr %b)
  call void @FLDT(ptr %a)
  %f = call %fflags @FUCOMPI()
  call void @FPOP()
  %c = call i1 @cc.f.ult(%fflags %f)
  %r = call i1 @SETCC(i1 %c)
  ret i1 %r
}

declare i1 @llvm.epitome.x87.cmp.ule(ptr, ptr)

define i1 @x87.cmp.ule(ptr "reg"="r64" %a, ptr "reg"="r64" %b) "recipe"="x87.cmp.ule.expand" {
  %r = call i1 @llvm.epitome.x87.cmp.ule(ptr %a, ptr %b)
  ret i1 %r
}

define i1 @x87.cmp.ule.expand(ptr %a, ptr %b) {
  call void @FLDT(ptr %b)
  call void @FLDT(ptr %a)
  %f = call %fflags @FUCOMPI()
  call void @FPOP()
  %c = call i1 @cc.f.ule(%fflags %f)
  %r = call i1 @SETCC(i1 %c)
  ret i1 %r
}

declare i1 @llvm.epitome.x87.cmp.ueq(ptr, ptr)

define i1 @x87.cmp.ueq(ptr "reg"="r64" %a, ptr "reg"="r64" %b) "recipe"="x87.cmp.ueq.expand" {
  %r = call i1 @llvm.epitome.x87.cmp.ueq(ptr %a, ptr %b)
  ret i1 %r
}

define i1 @x87.cmp.ueq.expand(ptr %a, ptr %b) {
  call void @FLDT(ptr %b)
  call void @FLDT(ptr %a)
  %f = call %fflags @FUCOMPI()
  call void @FPOP()
  %c = call i1 @cc.f.ueq(%fflags %f)
  %r = call i1 @SETCC(i1 %c)
  ret i1 %r
}

declare i1 @llvm.epitome.x87.cmp.one(ptr, ptr)

define i1 @x87.cmp.one(ptr "reg"="r64" %a, ptr "reg"="r64" %b) "recipe"="x87.cmp.one.expand" {
  %r = call i1 @llvm.epitome.x87.cmp.one(ptr %a, ptr %b)
  ret i1 %r
}

define i1 @x87.cmp.one.expand(ptr %a, ptr %b) {
  call void @FLDT(ptr %b)
  call void @FLDT(ptr %a)
  %f = call %fflags @FUCOMPI()
  call void @FPOP()
  %c = call i1 @cc.f.one(%fflags %f)
  %r = call i1 @SETCC(i1 %c)
  ret i1 %r
}

declare i1 @llvm.epitome.x87.cmp.uno(ptr, ptr)

define i1 @x87.cmp.uno(ptr "reg"="r64" %a, ptr "reg"="r64" %b) "recipe"="x87.cmp.uno.expand" {
  %r = call i1 @llvm.epitome.x87.cmp.uno(ptr %a, ptr %b)
  ret i1 %r
}

define i1 @x87.cmp.uno.expand(ptr %a, ptr %b) {
  call void @FLDT(ptr %b)
  call void @FLDT(ptr %a)
  %f = call %fflags @FUCOMPI()
  call void @FPOP()
  %c = call i1 @cc.f.uno(%fflags %f)
  %r = call i1 @SETCC(i1 %c)
  ret i1 %r
}

declare i1 @llvm.epitome.x87.cmp.ord(ptr, ptr)

define i1 @x87.cmp.ord(ptr "reg"="r64" %a, ptr "reg"="r64" %b) "recipe"="x87.cmp.ord.expand" {
  %r = call i1 @llvm.epitome.x87.cmp.ord(ptr %a, ptr %b)
  ret i1 %r
}

define i1 @x87.cmp.ord.expand(ptr %a, ptr %b) {
  call void @FLDT(ptr %b)
  call void @FLDT(ptr %a)
  %f = call %fflags @FUCOMPI()
  call void @FPOP()
  %c = call i1 @cc.f.ord(%fflags %f)
  %r = call i1 @SETCC(i1 %c)
  ret i1 %r
}

define <16 x i8> @LDV.16xi8(ptr "reg"="r64" %n, i64 "imm" %off) "enc"="rex(0, d, 4, n) | 00001111 | 00010000 | rm(d, n, 4, 1, off, 0, 1)" "asm"="movups {off}({n}), {d}" "dclass"="xv" "nodecode" {
  %a = getelementptr i8, ptr %n, i64 %off
  %r = load <16 x i8>, ptr %a
  ret <16 x i8> %r
}

define void @STV.16xi8(<16 x i8> "reg"="xv" %t, ptr "reg"="r64" %n, i64 "imm" %off) "enc"="rex(0, t, 4, n) | 00001111 | 00010001 | rm(t, n, 4, 1, off, 0, 1)" "asm"="movups {t}, {off}({n})" "nodecode" {
  %a = getelementptr i8, ptr %n, i64 %off
  store <16 x i8> %t, ptr %a
  ret void
}

define <8 x i16> @LDV.8xi16(ptr "reg"="r64" %n, i64 "imm" %off) "enc"="rex(0, d, 4, n) | 00001111 | 00010000 | rm(d, n, 4, 1, off, 0, 1)" "asm"="movups {off}({n}), {d}" "dclass"="xv" "nodecode" {
  %a = getelementptr i8, ptr %n, i64 %off
  %r = load <8 x i16>, ptr %a
  ret <8 x i16> %r
}

define void @STV.8xi16(<8 x i16> "reg"="xv" %t, ptr "reg"="r64" %n, i64 "imm" %off) "enc"="rex(0, t, 4, n) | 00001111 | 00010001 | rm(t, n, 4, 1, off, 0, 1)" "asm"="movups {t}, {off}({n})" "nodecode" {
  %a = getelementptr i8, ptr %n, i64 %off
  store <8 x i16> %t, ptr %a
  ret void
}

define <4 x i32> @LDV.4xi32(ptr "reg"="r64" %n, i64 "imm" %off) "enc"="rex(0, d, 4, n) | 00001111 | 00010000 | rm(d, n, 4, 1, off, 0, 1)" "asm"="movups {off}({n}), {d}" "dclass"="xv" "nodecode" {
  %a = getelementptr i8, ptr %n, i64 %off
  %r = load <4 x i32>, ptr %a
  ret <4 x i32> %r
}

define void @STV.4xi32(<4 x i32> "reg"="xv" %t, ptr "reg"="r64" %n, i64 "imm" %off) "enc"="rex(0, t, 4, n) | 00001111 | 00010001 | rm(t, n, 4, 1, off, 0, 1)" "asm"="movups {t}, {off}({n})" "nodecode" {
  %a = getelementptr i8, ptr %n, i64 %off
  store <4 x i32> %t, ptr %a
  ret void
}

define <2 x i64> @LDV.2xi64(ptr "reg"="r64" %n, i64 "imm" %off) "enc"="rex(0, d, 4, n) | 00001111 | 00010000 | rm(d, n, 4, 1, off, 0, 1)" "asm"="movups {off}({n}), {d}" "dclass"="xv" "role"="load.xv,load.q" "nodecode" {
  %a = getelementptr i8, ptr %n, i64 %off
  %r = load <2 x i64>, ptr %a
  ret <2 x i64> %r
}

define void @STV.2xi64(<2 x i64> "reg"="xv" %t, ptr "reg"="r64" %n, i64 "imm" %off) "enc"="rex(0, t, 4, n) | 00001111 | 00010001 | rm(t, n, 4, 1, off, 0, 1)" "asm"="movups {t}, {off}({n})" "role"="store.xv,store.q" "nodecode" {
  %a = getelementptr i8, ptr %n, i64 %off
  store <2 x i64> %t, ptr %a
  ret void
}

define <8 x i8> @LDV.8xi8(ptr "reg"="r64" %n, i64 "imm" %off) "enc"="11110011 | rex(0, d, 4, n) | 00001111 | 01111110 | rm(d, n, 4, 1, off, 0, 1)" "asm"="movq {off}({n}), {d}" "dclass"="xh" "nodecode" {
  %a = getelementptr i8, ptr %n, i64 %off
  %r = load <8 x i8>, ptr %a
  ret <8 x i8> %r
}

define void @STV.8xi8(<8 x i8> "reg"="xh" %t, ptr "reg"="r64" %n, i64 "imm" %off) "enc"="01100110 | rex(0, t, 4, n) | 00001111 | 11010110 | rm(t, n, 4, 1, off, 0, 1)" "asm"="movq {t}, {off}({n})" "nodecode" {
  %a = getelementptr i8, ptr %n, i64 %off
  store <8 x i8> %t, ptr %a
  ret void
}

define <4 x i16> @LDV.4xi16(ptr "reg"="r64" %n, i64 "imm" %off) "enc"="11110011 | rex(0, d, 4, n) | 00001111 | 01111110 | rm(d, n, 4, 1, off, 0, 1)" "asm"="movq {off}({n}), {d}" "dclass"="xh" "nodecode" {
  %a = getelementptr i8, ptr %n, i64 %off
  %r = load <4 x i16>, ptr %a
  ret <4 x i16> %r
}

define void @STV.4xi16(<4 x i16> "reg"="xh" %t, ptr "reg"="r64" %n, i64 "imm" %off) "enc"="01100110 | rex(0, t, 4, n) | 00001111 | 11010110 | rm(t, n, 4, 1, off, 0, 1)" "asm"="movq {t}, {off}({n})" "nodecode" {
  %a = getelementptr i8, ptr %n, i64 %off
  store <4 x i16> %t, ptr %a
  ret void
}

define <2 x i32> @LDV.2xi32(ptr "reg"="r64" %n, i64 "imm" %off) "enc"="11110011 | rex(0, d, 4, n) | 00001111 | 01111110 | rm(d, n, 4, 1, off, 0, 1)" "asm"="movq {off}({n}), {d}" "dclass"="xh" "role"="load.xh" "nodecode" {
  %a = getelementptr i8, ptr %n, i64 %off
  %r = load <2 x i32>, ptr %a
  ret <2 x i32> %r
}

define void @STV.2xi32(<2 x i32> "reg"="xh" %t, ptr "reg"="r64" %n, i64 "imm" %off) "enc"="01100110 | rex(0, t, 4, n) | 00001111 | 11010110 | rm(t, n, 4, 1, off, 0, 1)" "asm"="movq {t}, {off}({n})" "role"="store.xh" "nodecode" {
  %a = getelementptr i8, ptr %n, i64 %off
  store <2 x i32> %t, ptr %a
  ret void
}

define <4 x float> @LDV.4xf32(ptr "reg"="r64" %n, i64 "imm" %off) "enc"="rex(0, d, 4, n) | 00001111 | 00010000 | rm(d, n, 4, 1, off, 0, 1)" "asm"="movups {off}({n}), {d}" "dclass"="xv" "nodecode" {
  %a = getelementptr i8, ptr %n, i64 %off
  %r = load <4 x float>, ptr %a
  ret <4 x float> %r
}

define void @STV.4xf32(<4 x float> "reg"="xv" %t, ptr "reg"="r64" %n, i64 "imm" %off) "enc"="rex(0, t, 4, n) | 00001111 | 00010001 | rm(t, n, 4, 1, off, 0, 1)" "asm"="movups {t}, {off}({n})" "nodecode" {
  %a = getelementptr i8, ptr %n, i64 %off
  store <4 x float> %t, ptr %a
  ret void
}

define <2 x double> @LDV.2xf64(ptr "reg"="r64" %n, i64 "imm" %off) "enc"="rex(0, d, 4, n) | 00001111 | 00010000 | rm(d, n, 4, 1, off, 0, 1)" "asm"="movups {off}({n}), {d}" "dclass"="xv" "nodecode" {
  %a = getelementptr i8, ptr %n, i64 %off
  %r = load <2 x double>, ptr %a
  ret <2 x double> %r
}

define void @STV.2xf64(<2 x double> "reg"="xv" %t, ptr "reg"="r64" %n, i64 "imm" %off) "enc"="rex(0, t, 4, n) | 00001111 | 00010001 | rm(t, n, 4, 1, off, 0, 1)" "asm"="movups {t}, {off}({n})" "nodecode" {
  %a = getelementptr i8, ptr %n, i64 %off
  store <2 x double> %t, ptr %a
  ret void
}

define <2 x float> @LDV.2xf32(ptr "reg"="r64" %n, i64 "imm" %off) "enc"="11110011 | rex(0, d, 4, n) | 00001111 | 01111110 | rm(d, n, 4, 1, off, 0, 1)" "asm"="movq {off}({n}), {d}" "dclass"="xh" "nodecode" {
  %a = getelementptr i8, ptr %n, i64 %off
  %r = load <2 x float>, ptr %a
  ret <2 x float> %r
}

define void @STV.2xf32(<2 x float> "reg"="xh" %t, ptr "reg"="r64" %n, i64 "imm" %off) "enc"="01100110 | rex(0, t, 4, n) | 00001111 | 11010110 | rm(t, n, 4, 1, off, 0, 1)" "asm"="movq {t}, {off}({n})" "nodecode" {
  %a = getelementptr i8, ptr %n, i64 %off
  store <2 x float> %t, ptr %a
  ret void
}

define <2 x i64> @MOVAPSxv(<2 x i64> "reg"="xv" %n) "enc"="rex(0, d, 0, n) | 00001111 | 00101000 | rr(d, n)" "asm"="movaps {n}, {d}" "copy"="xv" "dclass"="xv" "nodecode" {
  ret <2 x i64> %n
}

define <2 x i32> @MOVAPSxh(<2 x i32> "reg"="xh" %n) "enc"="rex(0, d, 0, n) | 00001111 | 00101000 | rr(d, n)" "asm"="movaps {n}, {d}" "copy"="xh" "dclass"="xh" "nodecode" {
  ret <2 x i32> %n
}

define <2 x i64> @PXORzero() "enc"="01100110 | rex(0, d, 0, d) | 00001111 | 11101111 | rr(d, d)" "asm"="pxor {d}, {d}" "dclass"="xv" "role"="movi.zero" "nomatch" "nodecode" {
  ret <2 x i64> zeroinitializer
}

define <16 x i8> @PADDB.16xi8(<16 x i8> "reg"="xv" "tied" %n, <16 x i8> "reg"="xv" %m) "enc"="01100110 | rex(0, d, 0, m) | 00001111 | 11111100 | rr(d, m)" "asm"="paddb {m}, {d}" "dclass"="xv" "nodecode" {
  %r = add <16 x i8> %n, %m
  ret <16 x i8> %r
}

define <16 x i8> @PSUBB.16xi8(<16 x i8> "reg"="xv" "tied" %n, <16 x i8> "reg"="xv" %m) "enc"="01100110 | rex(0, d, 0, m) | 00001111 | 11111000 | rr(d, m)" "asm"="psubb {m}, {d}" "dclass"="xv" "nodecode" {
  %r = sub <16 x i8> %n, %m
  ret <16 x i8> %r
}

define <16 x i8> @PAND.16xi8(<16 x i8> "reg"="xv" "tied" %n, <16 x i8> "reg"="xv" %m) "enc"="01100110 | rex(0, d, 0, m) | 00001111 | 11011011 | rr(d, m)" "asm"="pand {m}, {d}" "dclass"="xv" "nodecode" {
  %r = and <16 x i8> %n, %m
  ret <16 x i8> %r
}

define <16 x i8> @POR.16xi8(<16 x i8> "reg"="xv" "tied" %n, <16 x i8> "reg"="xv" %m) "enc"="01100110 | rex(0, d, 0, m) | 00001111 | 11101011 | rr(d, m)" "asm"="por {m}, {d}" "dclass"="xv" "nodecode" {
  %r = or <16 x i8> %n, %m
  ret <16 x i8> %r
}

define <16 x i8> @PXOR.16xi8(<16 x i8> "reg"="xv" "tied" %n, <16 x i8> "reg"="xv" %m) "enc"="01100110 | rex(0, d, 0, m) | 00001111 | 11101111 | rr(d, m)" "asm"="pxor {m}, {d}" "dclass"="xv" "nodecode" {
  %r = xor <16 x i8> %n, %m
  ret <16 x i8> %r
}

define <16 x i1> @PCMPEQB.16xi8(<16 x i8> "reg"="xv" "tied" %n, <16 x i8> "reg"="xv" %m) "enc"="01100110 | rex(0, d, 0, m) | 00001111 | 01110100 | rr(d, m)" "asm"="pcmpeqb {m}, {d}" "dclass"="xv" "nodecode" {
  %r = icmp eq <16 x i8> %n, %m
  ret <16 x i1> %r
}

define <16 x i1> @PCMPGTB.16xi8(<16 x i8> "reg"="xv" "tied" %n, <16 x i8> "reg"="xv" %m) "enc"="01100110 | rex(0, d, 0, m) | 00001111 | 01100100 | rr(d, m)" "asm"="pcmpgtb {m}, {d}" "dclass"="xv" "nodecode" {
  %r = icmp sgt <16 x i8> %n, %m
  ret <16 x i1> %r
}

define <8 x i16> @PADDW.8xi16(<8 x i16> "reg"="xv" "tied" %n, <8 x i16> "reg"="xv" %m) "enc"="01100110 | rex(0, d, 0, m) | 00001111 | 11111101 | rr(d, m)" "asm"="paddw {m}, {d}" "dclass"="xv" "nodecode" {
  %r = add <8 x i16> %n, %m
  ret <8 x i16> %r
}

define <8 x i16> @PSUBW.8xi16(<8 x i16> "reg"="xv" "tied" %n, <8 x i16> "reg"="xv" %m) "enc"="01100110 | rex(0, d, 0, m) | 00001111 | 11111001 | rr(d, m)" "asm"="psubw {m}, {d}" "dclass"="xv" "nodecode" {
  %r = sub <8 x i16> %n, %m
  ret <8 x i16> %r
}

define <8 x i16> @PMULLW.8xi16(<8 x i16> "reg"="xv" "tied" %n, <8 x i16> "reg"="xv" %m) "enc"="01100110 | rex(0, d, 0, m) | 00001111 | 11010101 | rr(d, m)" "asm"="pmullw {m}, {d}" "dclass"="xv" "nodecode" {
  %r = mul <8 x i16> %n, %m
  ret <8 x i16> %r
}
define <4 x i32> @PMULLD.4xi32(<4 x i32> "reg"="xv" "tied" %n, <4 x i32> "reg"="xv" %m) "enc"="01100110 | rex(0, d, 0, m) | 00001111 | 00111000 | 01000000 | rr(d, m)" "asm"="pmulld {m}, {d}" "dclass"="xv" "feature"="sse4.1" "nodecode" {
  %r = mul <4 x i32> %n, %m
  ret <4 x i32> %r
}

define <8 x i16> @PAND.8xi16(<8 x i16> "reg"="xv" "tied" %n, <8 x i16> "reg"="xv" %m) "enc"="01100110 | rex(0, d, 0, m) | 00001111 | 11011011 | rr(d, m)" "asm"="pand {m}, {d}" "dclass"="xv" "nodecode" {
  %r = and <8 x i16> %n, %m
  ret <8 x i16> %r
}

define <8 x i16> @POR.8xi16(<8 x i16> "reg"="xv" "tied" %n, <8 x i16> "reg"="xv" %m) "enc"="01100110 | rex(0, d, 0, m) | 00001111 | 11101011 | rr(d, m)" "asm"="por {m}, {d}" "dclass"="xv" "nodecode" {
  %r = or <8 x i16> %n, %m
  ret <8 x i16> %r
}

define <8 x i16> @PXOR.8xi16(<8 x i16> "reg"="xv" "tied" %n, <8 x i16> "reg"="xv" %m) "enc"="01100110 | rex(0, d, 0, m) | 00001111 | 11101111 | rr(d, m)" "asm"="pxor {m}, {d}" "dclass"="xv" "nodecode" {
  %r = xor <8 x i16> %n, %m
  ret <8 x i16> %r
}

define <8 x i16> @PSLLWi.8xi16(<8 x i16> "reg"="xv" "tied" %n, <8 x i16> "imm" %sh) "enc"="01100110 | rex(0, 0, 0, d) | 00001111 | 01110001 | rr(6, d) | ib:8=sh" "asm"="psllw ${sh}, {d}" "dclass"="xv" "nodecode" {
  %r = shl <8 x i16> %n, %sh
  ret <8 x i16> %r
}

define <8 x i16> @PSRLWi.8xi16(<8 x i16> "reg"="xv" "tied" %n, <8 x i16> "imm" %sh) "enc"="01100110 | rex(0, 0, 0, d) | 00001111 | 01110001 | rr(2, d) | ib:8=sh" "asm"="psrlw ${sh}, {d}" "dclass"="xv" "nodecode" {
  %r = lshr <8 x i16> %n, %sh
  ret <8 x i16> %r
}

define <8 x i16> @PSRAWi.8xi16(<8 x i16> "reg"="xv" "tied" %n, <8 x i16> "imm" %sh) "enc"="01100110 | rex(0, 0, 0, d) | 00001111 | 01110001 | rr(4, d) | ib:8=sh" "asm"="psraw ${sh}, {d}" "dclass"="xv" "nodecode" {
  %r = ashr <8 x i16> %n, %sh
  ret <8 x i16> %r
}

define <8 x i1> @PCMPEQW.8xi16(<8 x i16> "reg"="xv" "tied" %n, <8 x i16> "reg"="xv" %m) "enc"="01100110 | rex(0, d, 0, m) | 00001111 | 01110101 | rr(d, m)" "asm"="pcmpeqw {m}, {d}" "dclass"="xv" "nodecode" {
  %r = icmp eq <8 x i16> %n, %m
  ret <8 x i1> %r
}

define <8 x i1> @PCMPGTW.8xi16(<8 x i16> "reg"="xv" "tied" %n, <8 x i16> "reg"="xv" %m) "enc"="01100110 | rex(0, d, 0, m) | 00001111 | 01100101 | rr(d, m)" "asm"="pcmpgtw {m}, {d}" "dclass"="xv" "nodecode" {
  %r = icmp sgt <8 x i16> %n, %m
  ret <8 x i1> %r
}

define <8 x i16> @MASK.8xi16(<8 x i1> "reg"="xv" "tied" %n) "enc"="01100110 | rex(0, 0, 0, d) | 00001111 | 01110001 | rr(2, d) | ib:8=15" "asm"="psrlw $15, {d}" "dclass"="xv" "nodecode" {
  %r = zext <8 x i1> %n to <8 x i16>
  ret <8 x i16> %r
}

define <4 x i32> @PADDD.4xi32(<4 x i32> "reg"="xv" "tied" %n, <4 x i32> "reg"="xv" %m) "enc"="01100110 | rex(0, d, 0, m) | 00001111 | 11111110 | rr(d, m)" "asm"="paddd {m}, {d}" "dclass"="xv" "nodecode" {
  %r = add <4 x i32> %n, %m
  ret <4 x i32> %r
}

define <4 x i32> @PSUBD.4xi32(<4 x i32> "reg"="xv" "tied" %n, <4 x i32> "reg"="xv" %m) "enc"="01100110 | rex(0, d, 0, m) | 00001111 | 11111010 | rr(d, m)" "asm"="psubd {m}, {d}" "dclass"="xv" "nodecode" {
  %r = sub <4 x i32> %n, %m
  ret <4 x i32> %r
}

define <4 x i32> @PAND.4xi32(<4 x i32> "reg"="xv" "tied" %n, <4 x i32> "reg"="xv" %m) "enc"="01100110 | rex(0, d, 0, m) | 00001111 | 11011011 | rr(d, m)" "asm"="pand {m}, {d}" "dclass"="xv" "nodecode" {
  %r = and <4 x i32> %n, %m
  ret <4 x i32> %r
}

define <4 x i32> @POR.4xi32(<4 x i32> "reg"="xv" "tied" %n, <4 x i32> "reg"="xv" %m) "enc"="01100110 | rex(0, d, 0, m) | 00001111 | 11101011 | rr(d, m)" "asm"="por {m}, {d}" "dclass"="xv" "nodecode" {
  %r = or <4 x i32> %n, %m
  ret <4 x i32> %r
}

define <4 x i32> @PXOR.4xi32(<4 x i32> "reg"="xv" "tied" %n, <4 x i32> "reg"="xv" %m) "enc"="01100110 | rex(0, d, 0, m) | 00001111 | 11101111 | rr(d, m)" "asm"="pxor {m}, {d}" "dclass"="xv" "nodecode" {
  %r = xor <4 x i32> %n, %m
  ret <4 x i32> %r
}

define <4 x i32> @PSLLDi.4xi32(<4 x i32> "reg"="xv" "tied" %n, <4 x i32> "imm" %sh) "enc"="01100110 | rex(0, 0, 0, d) | 00001111 | 01110010 | rr(6, d) | ib:8=sh" "asm"="pslld ${sh}, {d}" "dclass"="xv" "nodecode" {
  %r = shl <4 x i32> %n, %sh
  ret <4 x i32> %r
}

define <4 x i32> @PSRLDi.4xi32(<4 x i32> "reg"="xv" "tied" %n, <4 x i32> "imm" %sh) "enc"="01100110 | rex(0, 0, 0, d) | 00001111 | 01110010 | rr(2, d) | ib:8=sh" "asm"="psrld ${sh}, {d}" "dclass"="xv" "nodecode" {
  %r = lshr <4 x i32> %n, %sh
  ret <4 x i32> %r
}

define <4 x i32> @PSRADi.4xi32(<4 x i32> "reg"="xv" "tied" %n, <4 x i32> "imm" %sh) "enc"="01100110 | rex(0, 0, 0, d) | 00001111 | 01110010 | rr(4, d) | ib:8=sh" "asm"="psrad ${sh}, {d}" "dclass"="xv" "nodecode" {
  %r = ashr <4 x i32> %n, %sh
  ret <4 x i32> %r
}

define <4 x i1> @PCMPEQD.4xi32(<4 x i32> "reg"="xv" "tied" %n, <4 x i32> "reg"="xv" %m) "enc"="01100110 | rex(0, d, 0, m) | 00001111 | 01110110 | rr(d, m)" "asm"="pcmpeqd {m}, {d}" "dclass"="xv" "nodecode" {
  %r = icmp eq <4 x i32> %n, %m
  ret <4 x i1> %r
}

define <4 x i1> @PCMPGTD.4xi32(<4 x i32> "reg"="xv" "tied" %n, <4 x i32> "reg"="xv" %m) "enc"="01100110 | rex(0, d, 0, m) | 00001111 | 01100110 | rr(d, m)" "asm"="pcmpgtd {m}, {d}" "dclass"="xv" "nodecode" {
  %r = icmp sgt <4 x i32> %n, %m
  ret <4 x i1> %r
}

define <4 x i32> @MASK.4xi32(<4 x i1> "reg"="xv" "tied" %n) "enc"="01100110 | rex(0, 0, 0, d) | 00001111 | 01110010 | rr(2, d) | ib:8=31" "asm"="psrld $31, {d}" "dclass"="xv" "nodecode" {
  %r = zext <4 x i1> %n to <4 x i32>
  ret <4 x i32> %r
}

define <2 x i64> @PADDQ.2xi64(<2 x i64> "reg"="xv" "tied" %n, <2 x i64> "reg"="xv" %m) "enc"="01100110 | rex(0, d, 0, m) | 00001111 | 11010100 | rr(d, m)" "asm"="paddq {m}, {d}" "dclass"="xv" "nodecode" {
  %r = add <2 x i64> %n, %m
  ret <2 x i64> %r
}

define <2 x i64> @PSUBQ.2xi64(<2 x i64> "reg"="xv" "tied" %n, <2 x i64> "reg"="xv" %m) "enc"="01100110 | rex(0, d, 0, m) | 00001111 | 11111011 | rr(d, m)" "asm"="psubq {m}, {d}" "dclass"="xv" "nodecode" {
  %r = sub <2 x i64> %n, %m
  ret <2 x i64> %r
}

define <2 x i64> @PAND.2xi64(<2 x i64> "reg"="xv" "tied" %n, <2 x i64> "reg"="xv" %m) "enc"="01100110 | rex(0, d, 0, m) | 00001111 | 11011011 | rr(d, m)" "asm"="pand {m}, {d}" "dclass"="xv" "nodecode" {
  %r = and <2 x i64> %n, %m
  ret <2 x i64> %r
}

define <2 x i64> @POR.2xi64(<2 x i64> "reg"="xv" "tied" %n, <2 x i64> "reg"="xv" %m) "enc"="01100110 | rex(0, d, 0, m) | 00001111 | 11101011 | rr(d, m)" "asm"="por {m}, {d}" "dclass"="xv" "nodecode" {
  %r = or <2 x i64> %n, %m
  ret <2 x i64> %r
}

define <2 x i64> @PXOR.2xi64(<2 x i64> "reg"="xv" "tied" %n, <2 x i64> "reg"="xv" %m) "enc"="01100110 | rex(0, d, 0, m) | 00001111 | 11101111 | rr(d, m)" "asm"="pxor {m}, {d}" "dclass"="xv" "nodecode" {
  %r = xor <2 x i64> %n, %m
  ret <2 x i64> %r
}

define <2 x i64> @PSLLQi.2xi64(<2 x i64> "reg"="xv" "tied" %n, <2 x i64> "imm" %sh) "enc"="01100110 | rex(0, 0, 0, d) | 00001111 | 01110011 | rr(6, d) | ib:8=sh" "asm"="psllq ${sh}, {d}" "dclass"="xv" "nodecode" {
  %r = shl <2 x i64> %n, %sh
  ret <2 x i64> %r
}

define <2 x i64> @PSRLQi.2xi64(<2 x i64> "reg"="xv" "tied" %n, <2 x i64> "imm" %sh) "enc"="01100110 | rex(0, 0, 0, d) | 00001111 | 01110011 | rr(2, d) | ib:8=sh" "asm"="psrlq ${sh}, {d}" "dclass"="xv" "nodecode" {
  %r = lshr <2 x i64> %n, %sh
  ret <2 x i64> %r
}

define <2 x i64> @MASK.2xi64(<2 x i1> "reg"="xv" "tied" %n) "enc"="01100110 | rex(0, 0, 0, d) | 00001111 | 01110011 | rr(2, d) | ib:8=63" "asm"="psrlq $63, {d}" "dclass"="xv" "nodecode" {
  %r = zext <2 x i1> %n to <2 x i64>
  ret <2 x i64> %r
}

define <8 x i8> @PADDB.8xi8(<8 x i8> "reg"="xh" "tied" %n, <8 x i8> "reg"="xh" %m) "enc"="01100110 | rex(0, d, 0, m) | 00001111 | 11111100 | rr(d, m)" "asm"="paddb {m}, {d}" "dclass"="xh" "nodecode" {
  %r = add <8 x i8> %n, %m
  ret <8 x i8> %r
}

define <8 x i8> @PSUBB.8xi8(<8 x i8> "reg"="xh" "tied" %n, <8 x i8> "reg"="xh" %m) "enc"="01100110 | rex(0, d, 0, m) | 00001111 | 11111000 | rr(d, m)" "asm"="psubb {m}, {d}" "dclass"="xh" "nodecode" {
  %r = sub <8 x i8> %n, %m
  ret <8 x i8> %r
}

define <8 x i8> @PAND.8xi8(<8 x i8> "reg"="xh" "tied" %n, <8 x i8> "reg"="xh" %m) "enc"="01100110 | rex(0, d, 0, m) | 00001111 | 11011011 | rr(d, m)" "asm"="pand {m}, {d}" "dclass"="xh" "nodecode" {
  %r = and <8 x i8> %n, %m
  ret <8 x i8> %r
}

define <8 x i8> @POR.8xi8(<8 x i8> "reg"="xh" "tied" %n, <8 x i8> "reg"="xh" %m) "enc"="01100110 | rex(0, d, 0, m) | 00001111 | 11101011 | rr(d, m)" "asm"="por {m}, {d}" "dclass"="xh" "nodecode" {
  %r = or <8 x i8> %n, %m
  ret <8 x i8> %r
}

define <8 x i8> @PXOR.8xi8(<8 x i8> "reg"="xh" "tied" %n, <8 x i8> "reg"="xh" %m) "enc"="01100110 | rex(0, d, 0, m) | 00001111 | 11101111 | rr(d, m)" "asm"="pxor {m}, {d}" "dclass"="xh" "nodecode" {
  %r = xor <8 x i8> %n, %m
  ret <8 x i8> %r
}

define <8 x i1> @PCMPEQB.8xi8(<8 x i8> "reg"="xh" "tied" %n, <8 x i8> "reg"="xh" %m) "enc"="01100110 | rex(0, d, 0, m) | 00001111 | 01110100 | rr(d, m)" "asm"="pcmpeqb {m}, {d}" "dclass"="xv" "nodecode" {
  %r = icmp eq <8 x i8> %n, %m
  ret <8 x i1> %r
}

define <8 x i1> @PCMPGTB.8xi8(<8 x i8> "reg"="xh" "tied" %n, <8 x i8> "reg"="xh" %m) "enc"="01100110 | rex(0, d, 0, m) | 00001111 | 01100100 | rr(d, m)" "asm"="pcmpgtb {m}, {d}" "dclass"="xv" "nodecode" {
  %r = icmp sgt <8 x i8> %n, %m
  ret <8 x i1> %r
}

define <4 x i16> @PADDW.4xi16(<4 x i16> "reg"="xh" "tied" %n, <4 x i16> "reg"="xh" %m) "enc"="01100110 | rex(0, d, 0, m) | 00001111 | 11111101 | rr(d, m)" "asm"="paddw {m}, {d}" "dclass"="xh" "nodecode" {
  %r = add <4 x i16> %n, %m
  ret <4 x i16> %r
}

define <4 x i16> @PSUBW.4xi16(<4 x i16> "reg"="xh" "tied" %n, <4 x i16> "reg"="xh" %m) "enc"="01100110 | rex(0, d, 0, m) | 00001111 | 11111001 | rr(d, m)" "asm"="psubw {m}, {d}" "dclass"="xh" "nodecode" {
  %r = sub <4 x i16> %n, %m
  ret <4 x i16> %r
}

define <4 x i16> @PMULLW.4xi16(<4 x i16> "reg"="xh" "tied" %n, <4 x i16> "reg"="xh" %m) "enc"="01100110 | rex(0, d, 0, m) | 00001111 | 11010101 | rr(d, m)" "asm"="pmullw {m}, {d}" "dclass"="xh" "nodecode" {
  %r = mul <4 x i16> %n, %m
  ret <4 x i16> %r
}

define <4 x i16> @PAND.4xi16(<4 x i16> "reg"="xh" "tied" %n, <4 x i16> "reg"="xh" %m) "enc"="01100110 | rex(0, d, 0, m) | 00001111 | 11011011 | rr(d, m)" "asm"="pand {m}, {d}" "dclass"="xh" "nodecode" {
  %r = and <4 x i16> %n, %m
  ret <4 x i16> %r
}

define <4 x i16> @POR.4xi16(<4 x i16> "reg"="xh" "tied" %n, <4 x i16> "reg"="xh" %m) "enc"="01100110 | rex(0, d, 0, m) | 00001111 | 11101011 | rr(d, m)" "asm"="por {m}, {d}" "dclass"="xh" "nodecode" {
  %r = or <4 x i16> %n, %m
  ret <4 x i16> %r
}

define <4 x i16> @PXOR.4xi16(<4 x i16> "reg"="xh" "tied" %n, <4 x i16> "reg"="xh" %m) "enc"="01100110 | rex(0, d, 0, m) | 00001111 | 11101111 | rr(d, m)" "asm"="pxor {m}, {d}" "dclass"="xh" "nodecode" {
  %r = xor <4 x i16> %n, %m
  ret <4 x i16> %r
}

define <4 x i16> @PSLLWi.4xi16(<4 x i16> "reg"="xh" "tied" %n, <4 x i16> "imm" %sh) "enc"="01100110 | rex(0, 0, 0, d) | 00001111 | 01110001 | rr(6, d) | ib:8=sh" "asm"="psllw ${sh}, {d}" "dclass"="xh" "nodecode" {
  %r = shl <4 x i16> %n, %sh
  ret <4 x i16> %r
}

define <4 x i16> @PSRLWi.4xi16(<4 x i16> "reg"="xh" "tied" %n, <4 x i16> "imm" %sh) "enc"="01100110 | rex(0, 0, 0, d) | 00001111 | 01110001 | rr(2, d) | ib:8=sh" "asm"="psrlw ${sh}, {d}" "dclass"="xh" "nodecode" {
  %r = lshr <4 x i16> %n, %sh
  ret <4 x i16> %r
}

define <4 x i16> @PSRAWi.4xi16(<4 x i16> "reg"="xh" "tied" %n, <4 x i16> "imm" %sh) "enc"="01100110 | rex(0, 0, 0, d) | 00001111 | 01110001 | rr(4, d) | ib:8=sh" "asm"="psraw ${sh}, {d}" "dclass"="xh" "nodecode" {
  %r = ashr <4 x i16> %n, %sh
  ret <4 x i16> %r
}

define <4 x i1> @PCMPEQW.4xi16(<4 x i16> "reg"="xh" "tied" %n, <4 x i16> "reg"="xh" %m) "enc"="01100110 | rex(0, d, 0, m) | 00001111 | 01110101 | rr(d, m)" "asm"="pcmpeqw {m}, {d}" "dclass"="xv" "nodecode" {
  %r = icmp eq <4 x i16> %n, %m
  ret <4 x i1> %r
}

define <4 x i1> @PCMPGTW.4xi16(<4 x i16> "reg"="xh" "tied" %n, <4 x i16> "reg"="xh" %m) "enc"="01100110 | rex(0, d, 0, m) | 00001111 | 01100101 | rr(d, m)" "asm"="pcmpgtw {m}, {d}" "dclass"="xv" "nodecode" {
  %r = icmp sgt <4 x i16> %n, %m
  ret <4 x i1> %r
}

define <4 x i16> @MASK.4xi16(<4 x i1> "reg"="xv" "tied" %n) "enc"="01100110 | rex(0, 0, 0, d) | 00001111 | 01110001 | rr(2, d) | ib:8=15" "asm"="psrlw $15, {d}" "dclass"="xh" "nodecode" {
  %r = zext <4 x i1> %n to <4 x i16>
  ret <4 x i16> %r
}

define <2 x i32> @PADDD.2xi32(<2 x i32> "reg"="xh" "tied" %n, <2 x i32> "reg"="xh" %m) "enc"="01100110 | rex(0, d, 0, m) | 00001111 | 11111110 | rr(d, m)" "asm"="paddd {m}, {d}" "dclass"="xh" "nodecode" {
  %r = add <2 x i32> %n, %m
  ret <2 x i32> %r
}

define <2 x i32> @PSUBD.2xi32(<2 x i32> "reg"="xh" "tied" %n, <2 x i32> "reg"="xh" %m) "enc"="01100110 | rex(0, d, 0, m) | 00001111 | 11111010 | rr(d, m)" "asm"="psubd {m}, {d}" "dclass"="xh" "nodecode" {
  %r = sub <2 x i32> %n, %m
  ret <2 x i32> %r
}

define <2 x i32> @PAND.2xi32(<2 x i32> "reg"="xh" "tied" %n, <2 x i32> "reg"="xh" %m) "enc"="01100110 | rex(0, d, 0, m) | 00001111 | 11011011 | rr(d, m)" "asm"="pand {m}, {d}" "dclass"="xh" "nodecode" {
  %r = and <2 x i32> %n, %m
  ret <2 x i32> %r
}

define <2 x i32> @POR.2xi32(<2 x i32> "reg"="xh" "tied" %n, <2 x i32> "reg"="xh" %m) "enc"="01100110 | rex(0, d, 0, m) | 00001111 | 11101011 | rr(d, m)" "asm"="por {m}, {d}" "dclass"="xh" "nodecode" {
  %r = or <2 x i32> %n, %m
  ret <2 x i32> %r
}

define <2 x i32> @PXOR.2xi32(<2 x i32> "reg"="xh" "tied" %n, <2 x i32> "reg"="xh" %m) "enc"="01100110 | rex(0, d, 0, m) | 00001111 | 11101111 | rr(d, m)" "asm"="pxor {m}, {d}" "dclass"="xh" "nodecode" {
  %r = xor <2 x i32> %n, %m
  ret <2 x i32> %r
}

define <2 x i32> @PSLLDi.2xi32(<2 x i32> "reg"="xh" "tied" %n, <2 x i32> "imm" %sh) "enc"="01100110 | rex(0, 0, 0, d) | 00001111 | 01110010 | rr(6, d) | ib:8=sh" "asm"="pslld ${sh}, {d}" "dclass"="xh" "nodecode" {
  %r = shl <2 x i32> %n, %sh
  ret <2 x i32> %r
}

define <2 x i32> @PSRLDi.2xi32(<2 x i32> "reg"="xh" "tied" %n, <2 x i32> "imm" %sh) "enc"="01100110 | rex(0, 0, 0, d) | 00001111 | 01110010 | rr(2, d) | ib:8=sh" "asm"="psrld ${sh}, {d}" "dclass"="xh" "nodecode" {
  %r = lshr <2 x i32> %n, %sh
  ret <2 x i32> %r
}

define <2 x i32> @PSRADi.2xi32(<2 x i32> "reg"="xh" "tied" %n, <2 x i32> "imm" %sh) "enc"="01100110 | rex(0, 0, 0, d) | 00001111 | 01110010 | rr(4, d) | ib:8=sh" "asm"="psrad ${sh}, {d}" "dclass"="xh" "nodecode" {
  %r = ashr <2 x i32> %n, %sh
  ret <2 x i32> %r
}

define <2 x i1> @PCMPEQD.2xi32(<2 x i32> "reg"="xh" "tied" %n, <2 x i32> "reg"="xh" %m) "enc"="01100110 | rex(0, d, 0, m) | 00001111 | 01110110 | rr(d, m)" "asm"="pcmpeqd {m}, {d}" "dclass"="xv" "nodecode" {
  %r = icmp eq <2 x i32> %n, %m
  ret <2 x i1> %r
}

define <2 x i1> @PCMPGTD.2xi32(<2 x i32> "reg"="xh" "tied" %n, <2 x i32> "reg"="xh" %m) "enc"="01100110 | rex(0, d, 0, m) | 00001111 | 01100110 | rr(d, m)" "asm"="pcmpgtd {m}, {d}" "dclass"="xv" "nodecode" {
  %r = icmp sgt <2 x i32> %n, %m
  ret <2 x i1> %r
}

define <2 x i32> @MASK.2xi32(<2 x i1> "reg"="xv" "tied" %n) "enc"="01100110 | rex(0, 0, 0, d) | 00001111 | 01110010 | rr(2, d) | ib:8=31" "asm"="psrld $31, {d}" "dclass"="xh" "nodecode" {
  %r = zext <2 x i1> %n to <2 x i32>
  ret <2 x i32> %r
}

define <16 x i1> @PAND.m16(<16 x i1> "reg"="xv" "tied" %n, <16 x i1> "reg"="xv" %m) "enc"="01100110 | rex(0, d, 0, m) | 00001111 | 11011011 | rr(d, m)" "asm"="pand {m}, {d}" "dclass"="xv" "nodecode" {
  %r = and <16 x i1> %n, %m
  ret <16 x i1> %r
}

define <16 x i1> @POR.m16(<16 x i1> "reg"="xv" "tied" %n, <16 x i1> "reg"="xv" %m) "enc"="01100110 | rex(0, d, 0, m) | 00001111 | 11101011 | rr(d, m)" "asm"="por {m}, {d}" "dclass"="xv" "nodecode" {
  %r = or <16 x i1> %n, %m
  ret <16 x i1> %r
}

define <16 x i1> @PXOR.m16(<16 x i1> "reg"="xv" "tied" %n, <16 x i1> "reg"="xv" %m) "enc"="01100110 | rex(0, d, 0, m) | 00001111 | 11101111 | rr(d, m)" "asm"="pxor {m}, {d}" "dclass"="xv" "nodecode" {
  %r = xor <16 x i1> %n, %m
  ret <16 x i1> %r
}

define <8 x i1> @PAND.m8(<8 x i1> "reg"="xv" "tied" %n, <8 x i1> "reg"="xv" %m) "enc"="01100110 | rex(0, d, 0, m) | 00001111 | 11011011 | rr(d, m)" "asm"="pand {m}, {d}" "dclass"="xv" "nodecode" {
  %r = and <8 x i1> %n, %m
  ret <8 x i1> %r
}

define <8 x i1> @POR.m8(<8 x i1> "reg"="xv" "tied" %n, <8 x i1> "reg"="xv" %m) "enc"="01100110 | rex(0, d, 0, m) | 00001111 | 11101011 | rr(d, m)" "asm"="por {m}, {d}" "dclass"="xv" "nodecode" {
  %r = or <8 x i1> %n, %m
  ret <8 x i1> %r
}

define <8 x i1> @PXOR.m8(<8 x i1> "reg"="xv" "tied" %n, <8 x i1> "reg"="xv" %m) "enc"="01100110 | rex(0, d, 0, m) | 00001111 | 11101111 | rr(d, m)" "asm"="pxor {m}, {d}" "dclass"="xv" "nodecode" {
  %r = xor <8 x i1> %n, %m
  ret <8 x i1> %r
}

define <4 x i1> @PAND.m4(<4 x i1> "reg"="xv" "tied" %n, <4 x i1> "reg"="xv" %m) "enc"="01100110 | rex(0, d, 0, m) | 00001111 | 11011011 | rr(d, m)" "asm"="pand {m}, {d}" "dclass"="xv" "nodecode" {
  %r = and <4 x i1> %n, %m
  ret <4 x i1> %r
}

define <4 x i1> @POR.m4(<4 x i1> "reg"="xv" "tied" %n, <4 x i1> "reg"="xv" %m) "enc"="01100110 | rex(0, d, 0, m) | 00001111 | 11101011 | rr(d, m)" "asm"="por {m}, {d}" "dclass"="xv" "nodecode" {
  %r = or <4 x i1> %n, %m
  ret <4 x i1> %r
}

define <4 x i1> @PXOR.m4(<4 x i1> "reg"="xv" "tied" %n, <4 x i1> "reg"="xv" %m) "enc"="01100110 | rex(0, d, 0, m) | 00001111 | 11101111 | rr(d, m)" "asm"="pxor {m}, {d}" "dclass"="xv" "nodecode" {
  %r = xor <4 x i1> %n, %m
  ret <4 x i1> %r
}

define <2 x i1> @PAND.m2(<2 x i1> "reg"="xv" "tied" %n, <2 x i1> "reg"="xv" %m) "enc"="01100110 | rex(0, d, 0, m) | 00001111 | 11011011 | rr(d, m)" "asm"="pand {m}, {d}" "dclass"="xv" "nodecode" {
  %r = and <2 x i1> %n, %m
  ret <2 x i1> %r
}

define <2 x i1> @POR.m2(<2 x i1> "reg"="xv" "tied" %n, <2 x i1> "reg"="xv" %m) "enc"="01100110 | rex(0, d, 0, m) | 00001111 | 11101011 | rr(d, m)" "asm"="por {m}, {d}" "dclass"="xv" "nodecode" {
  %r = or <2 x i1> %n, %m
  ret <2 x i1> %r
}

define <2 x i1> @PXOR.m2(<2 x i1> "reg"="xv" "tied" %n, <2 x i1> "reg"="xv" %m) "enc"="01100110 | rex(0, d, 0, m) | 00001111 | 11101111 | rr(d, m)" "asm"="pxor {m}, {d}" "dclass"="xv" "nodecode" {
  %r = xor <2 x i1> %n, %m
  ret <2 x i1> %r
}

define <2 x i64> @PANDN.xv(<2 x i64> "reg"="xv" "tied" %n, <2 x i64> "reg"="xv" %m) "enc"="01100110 | rex(0, d, 0, m) | 00001111 | 11011111 | rr(d, m)" "asm"="pandn {m}, {d}" "dclass"="xv" "nodecode" {
  %x = xor <2 x i64> %n, splat (i64 -1)
  %r = and <2 x i64> %x, %m
  ret <2 x i64> %r
}

define <2 x i32> @PANDN.xh(<2 x i32> "reg"="xh" "tied" %n, <2 x i32> "reg"="xh" %m) "enc"="01100110 | rex(0, d, 0, m) | 00001111 | 11011111 | rr(d, m)" "asm"="pandn {m}, {d}" "dclass"="xh" "nodecode" {
  %x = xor <2 x i32> %n, splat (i32 -1)
  %r = and <2 x i32> %x, %m
  ret <2 x i32> %r
}

define <4 x float> @FADDPS.4xf32(<4 x float> "reg"="xv" "tied" %n, <4 x float> "reg"="xv" %m) "enc"="rex(0, d, 0, m) | 00001111 | 01011000 | rr(d, m)" "asm"="addps {m}, {d}" "dclass"="xv" "nodecode" {
  %r = fadd <4 x float> %n, %m
  ret <4 x float> %r
}

define <4 x float> @FSUBPS.4xf32(<4 x float> "reg"="xv" "tied" %n, <4 x float> "reg"="xv" %m) "enc"="rex(0, d, 0, m) | 00001111 | 01011100 | rr(d, m)" "asm"="subps {m}, {d}" "dclass"="xv" "nodecode" {
  %r = fsub <4 x float> %n, %m
  ret <4 x float> %r
}

define <4 x float> @FMULPS.4xf32(<4 x float> "reg"="xv" "tied" %n, <4 x float> "reg"="xv" %m) "enc"="rex(0, d, 0, m) | 00001111 | 01011001 | rr(d, m)" "asm"="mulps {m}, {d}" "dclass"="xv" "nodecode" {
  %r = fmul <4 x float> %n, %m
  ret <4 x float> %r
}

define <4 x float> @FDIVPS.4xf32(<4 x float> "reg"="xv" "tied" %n, <4 x float> "reg"="xv" %m) "enc"="rex(0, d, 0, m) | 00001111 | 01011110 | rr(d, m)" "asm"="divps {m}, {d}" "dclass"="xv" "nodecode" {
  %r = fdiv <4 x float> %n, %m
  ret <4 x float> %r
}

define <4 x float> @SQRTPS.4xf32(<4 x float> "reg"="xv" %n) "enc"="rex(0, d, 0, n) | 00001111 | 01010001 | rr(d, n)" "asm"="sqrtps {n}, {d}" "dclass"="xv" "nodecode" {
  %r = call <4 x float> @llvm.sqrt.v4f32(<4 x float> %n)
  ret <4 x float> %r
}

declare <4 x float> @llvm.sqrt.v4f32(<4 x float>)

define <4 x i1> @CMPOEQPS.4xf32(<4 x float> "reg"="xv" "tied" %n, <4 x float> "reg"="xv" %m) "enc"="rex(0, d, 0, m) | 00001111 | 11000010 | rr(d, m) | 00000000" "asm"="cmpps $0, {m}, {d}" "dclass"="xv" "nodecode" {
  %r = fcmp oeq <4 x float> %n, %m
  ret <4 x i1> %r
}

define <4 x i1> @CMPOLTPS.4xf32(<4 x float> "reg"="xv" "tied" %n, <4 x float> "reg"="xv" %m) "enc"="rex(0, d, 0, m) | 00001111 | 11000010 | rr(d, m) | 00000001" "asm"="cmpps $1, {m}, {d}" "dclass"="xv" "nodecode" {
  %r = fcmp olt <4 x float> %n, %m
  ret <4 x i1> %r
}

define <4 x i1> @CMPOLEPS.4xf32(<4 x float> "reg"="xv" "tied" %n, <4 x float> "reg"="xv" %m) "enc"="rex(0, d, 0, m) | 00001111 | 11000010 | rr(d, m) | 00000010" "asm"="cmpps $2, {m}, {d}" "dclass"="xv" "nodecode" {
  %r = fcmp ole <4 x float> %n, %m
  ret <4 x i1> %r
}

define <4 x i1> @CMPUNOPS.4xf32(<4 x float> "reg"="xv" "tied" %n, <4 x float> "reg"="xv" %m) "enc"="rex(0, d, 0, m) | 00001111 | 11000010 | rr(d, m) | 00000011" "asm"="cmpps $3, {m}, {d}" "dclass"="xv" "nodecode" {
  %r = fcmp uno <4 x float> %n, %m
  ret <4 x i1> %r
}

define <4 x i1> @CMPUNEPS.4xf32(<4 x float> "reg"="xv" "tied" %n, <4 x float> "reg"="xv" %m) "enc"="rex(0, d, 0, m) | 00001111 | 11000010 | rr(d, m) | 00000100" "asm"="cmpps $4, {m}, {d}" "dclass"="xv" "nodecode" {
  %r = fcmp une <4 x float> %n, %m
  ret <4 x i1> %r
}

define <4 x i1> @CMPUGEPS.4xf32(<4 x float> "reg"="xv" "tied" %n, <4 x float> "reg"="xv" %m) "enc"="rex(0, d, 0, m) | 00001111 | 11000010 | rr(d, m) | 00000101" "asm"="cmpps $5, {m}, {d}" "dclass"="xv" "nodecode" {
  %r = fcmp uge <4 x float> %n, %m
  ret <4 x i1> %r
}

define <4 x i1> @CMPUGTPS.4xf32(<4 x float> "reg"="xv" "tied" %n, <4 x float> "reg"="xv" %m) "enc"="rex(0, d, 0, m) | 00001111 | 11000010 | rr(d, m) | 00000110" "asm"="cmpps $6, {m}, {d}" "dclass"="xv" "nodecode" {
  %r = fcmp ugt <4 x float> %n, %m
  ret <4 x i1> %r
}

define <4 x i1> @CMPORDPS.4xf32(<4 x float> "reg"="xv" "tied" %n, <4 x float> "reg"="xv" %m) "enc"="rex(0, d, 0, m) | 00001111 | 11000010 | rr(d, m) | 00000111" "asm"="cmpps $7, {m}, {d}" "dclass"="xv" "nodecode" {
  %r = fcmp ord <4 x float> %n, %m
  ret <4 x i1> %r
}

define <4 x i1> @CMPOGTPS.4xf32(<4 x float> "reg"="xv" "tied" %n, <4 x float> "reg"="xv" %m) "enc"="rex(0, d, 0, m) | 00001111 | 11000010 | rr(d, m) | 00000001" "asm"="cmpps $1, {m}, {d}" "dclass"="xv" "nodecode" {
  %r = fcmp ogt <4 x float> %m, %n
  ret <4 x i1> %r
}

define <4 x i1> @CMPOGEPS.4xf32(<4 x float> "reg"="xv" "tied" %n, <4 x float> "reg"="xv" %m) "enc"="rex(0, d, 0, m) | 00001111 | 11000010 | rr(d, m) | 00000010" "asm"="cmpps $2, {m}, {d}" "dclass"="xv" "nodecode" {
  %r = fcmp oge <4 x float> %m, %n
  ret <4 x i1> %r
}

define <4 x i1> @CMPULTPS.4xf32(<4 x float> "reg"="xv" "tied" %n, <4 x float> "reg"="xv" %m) "enc"="rex(0, d, 0, m) | 00001111 | 11000010 | rr(d, m) | 00000110" "asm"="cmpps $6, {m}, {d}" "dclass"="xv" "nodecode" {
  %r = fcmp ult <4 x float> %m, %n
  ret <4 x i1> %r
}

define <4 x i1> @CMPULEPS.4xf32(<4 x float> "reg"="xv" "tied" %n, <4 x float> "reg"="xv" %m) "enc"="rex(0, d, 0, m) | 00001111 | 11000010 | rr(d, m) | 00000101" "asm"="cmpps $5, {m}, {d}" "dclass"="xv" "nodecode" {
  %r = fcmp ule <4 x float> %m, %n
  ret <4 x i1> %r
}

define <2 x double> @FADDPD.2xf64(<2 x double> "reg"="xv" "tied" %n, <2 x double> "reg"="xv" %m) "enc"="01100110 | rex(0, d, 0, m) | 00001111 | 01011000 | rr(d, m)" "asm"="addpd {m}, {d}" "dclass"="xv" "nodecode" {
  %r = fadd <2 x double> %n, %m
  ret <2 x double> %r
}

define <2 x double> @FSUBPD.2xf64(<2 x double> "reg"="xv" "tied" %n, <2 x double> "reg"="xv" %m) "enc"="01100110 | rex(0, d, 0, m) | 00001111 | 01011100 | rr(d, m)" "asm"="subpd {m}, {d}" "dclass"="xv" "nodecode" {
  %r = fsub <2 x double> %n, %m
  ret <2 x double> %r
}

define <2 x double> @FMULPD.2xf64(<2 x double> "reg"="xv" "tied" %n, <2 x double> "reg"="xv" %m) "enc"="01100110 | rex(0, d, 0, m) | 00001111 | 01011001 | rr(d, m)" "asm"="mulpd {m}, {d}" "dclass"="xv" "nodecode" {
  %r = fmul <2 x double> %n, %m
  ret <2 x double> %r
}

define <2 x double> @FDIVPD.2xf64(<2 x double> "reg"="xv" "tied" %n, <2 x double> "reg"="xv" %m) "enc"="01100110 | rex(0, d, 0, m) | 00001111 | 01011110 | rr(d, m)" "asm"="divpd {m}, {d}" "dclass"="xv" "nodecode" {
  %r = fdiv <2 x double> %n, %m
  ret <2 x double> %r
}

define <2 x double> @SQRTPD.2xf64(<2 x double> "reg"="xv" %n) "enc"="01100110 | rex(0, d, 0, n) | 00001111 | 01010001 | rr(d, n)" "asm"="sqrtpd {n}, {d}" "dclass"="xv" "nodecode" {
  %r = call <2 x double> @llvm.sqrt.v2f64(<2 x double> %n)
  ret <2 x double> %r
}

declare <2 x double> @llvm.sqrt.v2f64(<2 x double>)

define <2 x i1> @CMPOEQPD.2xf64(<2 x double> "reg"="xv" "tied" %n, <2 x double> "reg"="xv" %m) "enc"="01100110 | rex(0, d, 0, m) | 00001111 | 11000010 | rr(d, m) | 00000000" "asm"="cmppd $0, {m}, {d}" "dclass"="xv" "nodecode" {
  %r = fcmp oeq <2 x double> %n, %m
  ret <2 x i1> %r
}

define <2 x i1> @CMPOLTPD.2xf64(<2 x double> "reg"="xv" "tied" %n, <2 x double> "reg"="xv" %m) "enc"="01100110 | rex(0, d, 0, m) | 00001111 | 11000010 | rr(d, m) | 00000001" "asm"="cmppd $1, {m}, {d}" "dclass"="xv" "nodecode" {
  %r = fcmp olt <2 x double> %n, %m
  ret <2 x i1> %r
}

define <2 x i1> @CMPOLEPD.2xf64(<2 x double> "reg"="xv" "tied" %n, <2 x double> "reg"="xv" %m) "enc"="01100110 | rex(0, d, 0, m) | 00001111 | 11000010 | rr(d, m) | 00000010" "asm"="cmppd $2, {m}, {d}" "dclass"="xv" "nodecode" {
  %r = fcmp ole <2 x double> %n, %m
  ret <2 x i1> %r
}

define <2 x i1> @CMPUNOPD.2xf64(<2 x double> "reg"="xv" "tied" %n, <2 x double> "reg"="xv" %m) "enc"="01100110 | rex(0, d, 0, m) | 00001111 | 11000010 | rr(d, m) | 00000011" "asm"="cmppd $3, {m}, {d}" "dclass"="xv" "nodecode" {
  %r = fcmp uno <2 x double> %n, %m
  ret <2 x i1> %r
}

define <2 x i1> @CMPUNEPD.2xf64(<2 x double> "reg"="xv" "tied" %n, <2 x double> "reg"="xv" %m) "enc"="01100110 | rex(0, d, 0, m) | 00001111 | 11000010 | rr(d, m) | 00000100" "asm"="cmppd $4, {m}, {d}" "dclass"="xv" "nodecode" {
  %r = fcmp une <2 x double> %n, %m
  ret <2 x i1> %r
}

define <2 x i1> @CMPUGEPD.2xf64(<2 x double> "reg"="xv" "tied" %n, <2 x double> "reg"="xv" %m) "enc"="01100110 | rex(0, d, 0, m) | 00001111 | 11000010 | rr(d, m) | 00000101" "asm"="cmppd $5, {m}, {d}" "dclass"="xv" "nodecode" {
  %r = fcmp uge <2 x double> %n, %m
  ret <2 x i1> %r
}

define <2 x i1> @CMPUGTPD.2xf64(<2 x double> "reg"="xv" "tied" %n, <2 x double> "reg"="xv" %m) "enc"="01100110 | rex(0, d, 0, m) | 00001111 | 11000010 | rr(d, m) | 00000110" "asm"="cmppd $6, {m}, {d}" "dclass"="xv" "nodecode" {
  %r = fcmp ugt <2 x double> %n, %m
  ret <2 x i1> %r
}

define <2 x i1> @CMPORDPD.2xf64(<2 x double> "reg"="xv" "tied" %n, <2 x double> "reg"="xv" %m) "enc"="01100110 | rex(0, d, 0, m) | 00001111 | 11000010 | rr(d, m) | 00000111" "asm"="cmppd $7, {m}, {d}" "dclass"="xv" "nodecode" {
  %r = fcmp ord <2 x double> %n, %m
  ret <2 x i1> %r
}

define <2 x i1> @CMPOGTPD.2xf64(<2 x double> "reg"="xv" "tied" %n, <2 x double> "reg"="xv" %m) "enc"="01100110 | rex(0, d, 0, m) | 00001111 | 11000010 | rr(d, m) | 00000001" "asm"="cmppd $1, {m}, {d}" "dclass"="xv" "nodecode" {
  %r = fcmp ogt <2 x double> %m, %n
  ret <2 x i1> %r
}

define <2 x i1> @CMPOGEPD.2xf64(<2 x double> "reg"="xv" "tied" %n, <2 x double> "reg"="xv" %m) "enc"="01100110 | rex(0, d, 0, m) | 00001111 | 11000010 | rr(d, m) | 00000010" "asm"="cmppd $2, {m}, {d}" "dclass"="xv" "nodecode" {
  %r = fcmp oge <2 x double> %m, %n
  ret <2 x i1> %r
}

define <2 x i1> @CMPULTPD.2xf64(<2 x double> "reg"="xv" "tied" %n, <2 x double> "reg"="xv" %m) "enc"="01100110 | rex(0, d, 0, m) | 00001111 | 11000010 | rr(d, m) | 00000110" "asm"="cmppd $6, {m}, {d}" "dclass"="xv" "nodecode" {
  %r = fcmp ult <2 x double> %m, %n
  ret <2 x i1> %r
}

define <2 x i1> @CMPULEPD.2xf64(<2 x double> "reg"="xv" "tied" %n, <2 x double> "reg"="xv" %m) "enc"="01100110 | rex(0, d, 0, m) | 00001111 | 11000010 | rr(d, m) | 00000101" "asm"="cmppd $5, {m}, {d}" "dclass"="xv" "nodecode" {
  %r = fcmp ule <2 x double> %m, %n
  ret <2 x i1> %r
}

define <2 x float> @FADDPS.2xf32(<2 x float> "reg"="xh" "tied" %n, <2 x float> "reg"="xh" %m) "enc"="rex(0, d, 0, m) | 00001111 | 01011000 | rr(d, m)" "asm"="addps {m}, {d}" "dclass"="xh" "nodecode" {
  %r = fadd <2 x float> %n, %m
  ret <2 x float> %r
}

define <2 x float> @FSUBPS.2xf32(<2 x float> "reg"="xh" "tied" %n, <2 x float> "reg"="xh" %m) "enc"="rex(0, d, 0, m) | 00001111 | 01011100 | rr(d, m)" "asm"="subps {m}, {d}" "dclass"="xh" "nodecode" {
  %r = fsub <2 x float> %n, %m
  ret <2 x float> %r
}

define <2 x float> @FMULPS.2xf32(<2 x float> "reg"="xh" "tied" %n, <2 x float> "reg"="xh" %m) "enc"="rex(0, d, 0, m) | 00001111 | 01011001 | rr(d, m)" "asm"="mulps {m}, {d}" "dclass"="xh" "nodecode" {
  %r = fmul <2 x float> %n, %m
  ret <2 x float> %r
}

define <2 x float> @FDIVPS.2xf32(<2 x float> "reg"="xh" "tied" %n, <2 x float> "reg"="xh" %m) "enc"="rex(0, d, 0, m) | 00001111 | 01011110 | rr(d, m)" "asm"="divps {m}, {d}" "dclass"="xh" "nodecode" {
  %r = fdiv <2 x float> %n, %m
  ret <2 x float> %r
}

define <2 x float> @SQRTPS.2xf32(<2 x float> "reg"="xh" %n) "enc"="rex(0, d, 0, n) | 00001111 | 01010001 | rr(d, n)" "asm"="sqrtps {n}, {d}" "dclass"="xh" "nodecode" {
  %r = call <2 x float> @llvm.sqrt.v2f32(<2 x float> %n)
  ret <2 x float> %r
}

declare <2 x float> @llvm.sqrt.v2f32(<2 x float>)

define <2 x i1> @CMPOEQPS.2xf32(<2 x float> "reg"="xh" "tied" %n, <2 x float> "reg"="xh" %m) "enc"="rex(0, d, 0, m) | 00001111 | 11000010 | rr(d, m) | 00000000" "asm"="cmpps $0, {m}, {d}" "dclass"="xv" "nodecode" {
  %r = fcmp oeq <2 x float> %n, %m
  ret <2 x i1> %r
}

define <2 x i1> @CMPOLTPS.2xf32(<2 x float> "reg"="xh" "tied" %n, <2 x float> "reg"="xh" %m) "enc"="rex(0, d, 0, m) | 00001111 | 11000010 | rr(d, m) | 00000001" "asm"="cmpps $1, {m}, {d}" "dclass"="xv" "nodecode" {
  %r = fcmp olt <2 x float> %n, %m
  ret <2 x i1> %r
}

define <2 x i1> @CMPOLEPS.2xf32(<2 x float> "reg"="xh" "tied" %n, <2 x float> "reg"="xh" %m) "enc"="rex(0, d, 0, m) | 00001111 | 11000010 | rr(d, m) | 00000010" "asm"="cmpps $2, {m}, {d}" "dclass"="xv" "nodecode" {
  %r = fcmp ole <2 x float> %n, %m
  ret <2 x i1> %r
}

define <2 x i1> @CMPUNOPS.2xf32(<2 x float> "reg"="xh" "tied" %n, <2 x float> "reg"="xh" %m) "enc"="rex(0, d, 0, m) | 00001111 | 11000010 | rr(d, m) | 00000011" "asm"="cmpps $3, {m}, {d}" "dclass"="xv" "nodecode" {
  %r = fcmp uno <2 x float> %n, %m
  ret <2 x i1> %r
}

define <2 x i1> @CMPUNEPS.2xf32(<2 x float> "reg"="xh" "tied" %n, <2 x float> "reg"="xh" %m) "enc"="rex(0, d, 0, m) | 00001111 | 11000010 | rr(d, m) | 00000100" "asm"="cmpps $4, {m}, {d}" "dclass"="xv" "nodecode" {
  %r = fcmp une <2 x float> %n, %m
  ret <2 x i1> %r
}

define <2 x i1> @CMPUGEPS.2xf32(<2 x float> "reg"="xh" "tied" %n, <2 x float> "reg"="xh" %m) "enc"="rex(0, d, 0, m) | 00001111 | 11000010 | rr(d, m) | 00000101" "asm"="cmpps $5, {m}, {d}" "dclass"="xv" "nodecode" {
  %r = fcmp uge <2 x float> %n, %m
  ret <2 x i1> %r
}

define <2 x i1> @CMPUGTPS.2xf32(<2 x float> "reg"="xh" "tied" %n, <2 x float> "reg"="xh" %m) "enc"="rex(0, d, 0, m) | 00001111 | 11000010 | rr(d, m) | 00000110" "asm"="cmpps $6, {m}, {d}" "dclass"="xv" "nodecode" {
  %r = fcmp ugt <2 x float> %n, %m
  ret <2 x i1> %r
}

define <2 x i1> @CMPORDPS.2xf32(<2 x float> "reg"="xh" "tied" %n, <2 x float> "reg"="xh" %m) "enc"="rex(0, d, 0, m) | 00001111 | 11000010 | rr(d, m) | 00000111" "asm"="cmpps $7, {m}, {d}" "dclass"="xv" "nodecode" {
  %r = fcmp ord <2 x float> %n, %m
  ret <2 x i1> %r
}

define <2 x i1> @CMPOGTPS.2xf32(<2 x float> "reg"="xh" "tied" %n, <2 x float> "reg"="xh" %m) "enc"="rex(0, d, 0, m) | 00001111 | 11000010 | rr(d, m) | 00000001" "asm"="cmpps $1, {m}, {d}" "dclass"="xv" "nodecode" {
  %r = fcmp ogt <2 x float> %m, %n
  ret <2 x i1> %r
}

define <2 x i1> @CMPOGEPS.2xf32(<2 x float> "reg"="xh" "tied" %n, <2 x float> "reg"="xh" %m) "enc"="rex(0, d, 0, m) | 00001111 | 11000010 | rr(d, m) | 00000010" "asm"="cmpps $2, {m}, {d}" "dclass"="xv" "nodecode" {
  %r = fcmp oge <2 x float> %m, %n
  ret <2 x i1> %r
}

define <2 x i1> @CMPULTPS.2xf32(<2 x float> "reg"="xh" "tied" %n, <2 x float> "reg"="xh" %m) "enc"="rex(0, d, 0, m) | 00001111 | 11000010 | rr(d, m) | 00000110" "asm"="cmpps $6, {m}, {d}" "dclass"="xv" "nodecode" {
  %r = fcmp ult <2 x float> %m, %n
  ret <2 x i1> %r
}

define <2 x i1> @CMPULEPS.2xf32(<2 x float> "reg"="xh" "tied" %n, <2 x float> "reg"="xh" %m) "enc"="rex(0, d, 0, m) | 00001111 | 11000010 | rr(d, m) | 00000101" "asm"="cmpps $5, {m}, {d}" "dclass"="xv" "nodecode" {
  %r = fcmp ule <2 x float> %m, %n
  ret <2 x i1> %r
}

define <4 x float> @CVTDQ2PS.4xf32(<4 x i32> "reg"="xv" %n) "enc"="rex(0, d, 0, n) | 00001111 | 01011011 | rr(d, n)" "asm"="cvtdq2ps {n}, {d}" "dclass"="xv" "nodecode" {
  %r = sitofp <4 x i32> %n to <4 x float>
  ret <4 x float> %r
}

define <4 x i32> @CVTTPS2DQ.4xi32(<4 x float> "reg"="xv" %n) "enc"="11110011 | rex(0, d, 0, n) | 00001111 | 01011011 | rr(d, n)" "asm"="cvttps2dq {n}, {d}" "dclass"="xv" "nodecode" {
  %r = fptosi <4 x float> %n to <4 x i32>
  ret <4 x i32> %r
}

define <2 x float> @CVTDQ2PS.2xf32(<2 x i32> "reg"="xh" %n) "enc"="rex(0, d, 0, n) | 00001111 | 01011011 | rr(d, n)" "asm"="cvtdq2ps {n}, {d}" "dclass"="xh" "nodecode" {
  %r = sitofp <2 x i32> %n to <2 x float>
  ret <2 x float> %r
}

define <2 x i32> @CVTTPS2DQ.2xi32(<2 x float> "reg"="xh" %n) "enc"="11110011 | rex(0, d, 0, n) | 00001111 | 01011011 | rr(d, n)" "asm"="cvttps2dq {n}, {d}" "dclass"="xh" "nodecode" {
  %r = fptosi <2 x float> %n to <2 x i32>
  ret <2 x i32> %r
}

define <2 x double> @CVTPS2PD(<2 x float> "reg"="xh" %n) "enc"="rex(0, d, 0, n) | 00001111 | 01011010 | rr(d, n)" "asm"="cvtps2pd {n}, {d}" "dclass"="xv" "nodecode" {
  %r = fpext <2 x float> %n to <2 x double>
  ret <2 x double> %r
}

define <2 x float> @CVTPD2PS(<2 x double> "reg"="xv" %n) "enc"="01100110 | rex(0, d, 0, n) | 00001111 | 01011010 | rr(d, n)" "asm"="cvtpd2ps {n}, {d}" "dclass"="xh" "nodecode" {
  %r = fptrunc <2 x double> %n to <2 x float>
  ret <2 x float> %r
}

define <2 x double> @CVTDQ2PD(<2 x i32> "reg"="xh" %n) "enc"="11110011 | rex(0, d, 0, n) | 00001111 | 11100110 | rr(d, n)" "asm"="cvtdq2pd {n}, {d}" "dclass"="xv" "nodecode" {
  %r = sitofp <2 x i32> %n to <2 x double>
  ret <2 x double> %r
}

define <2 x i32> @CVTTPD2DQ(<2 x double> "reg"="xv" %n) "enc"="01100110 | rex(0, d, 0, n) | 00001111 | 11100110 | rr(d, n)" "asm"="cvttpd2dq {n}, {d}" "dclass"="xh" "nodecode" {
  %r = fptosi <2 x double> %n to <2 x i32>
  ret <2 x i32> %r
}

define <16 x i8> @vselect.16xi8(<16 x i1> "reg"="xv" %c, <16 x i8> "reg"="xv" %a, <16 x i8> "reg"="xv" %b) "recipe"="vselect.16xi8.expand" {
  %r = select <16 x i1> %c, <16 x i8> %a, <16 x i8> %b
  ret <16 x i8> %r
}

define <16 x i8> @vselect.16xi8.expand(<16 x i1> %c, <16 x i8> %a, <16 x i8> %b) {
  %cs = sext <16 x i1> %c to <16 x i8>
  %ci = bitcast <16 x i8> %cs to <2 x i64>
  %ti = bitcast <16 x i8> %a to <2 x i64>
  %fi = bitcast <16 x i8> %b to <2 x i64>
  %x = call <2 x i64> @PAND.2xi64(<2 x i64> %ci, <2 x i64> %ti)
  %y = call <2 x i64> @PANDN.xv(<2 x i64> %ci, <2 x i64> %fi)
  %z = call <2 x i64> @POR.2xi64(<2 x i64> %x, <2 x i64> %y)
  %r = bitcast <2 x i64> %z to <16 x i8>
  ret <16 x i8> %r
}

define <8 x i16> @vselect.8xi16(<8 x i1> "reg"="xv" %c, <8 x i16> "reg"="xv" %a, <8 x i16> "reg"="xv" %b) "recipe"="vselect.8xi16.expand" {
  %r = select <8 x i1> %c, <8 x i16> %a, <8 x i16> %b
  ret <8 x i16> %r
}

define <8 x i16> @vselect.8xi16.expand(<8 x i1> %c, <8 x i16> %a, <8 x i16> %b) {
  %cs = sext <8 x i1> %c to <8 x i16>
  %ci = bitcast <8 x i16> %cs to <2 x i64>
  %ti = bitcast <8 x i16> %a to <2 x i64>
  %fi = bitcast <8 x i16> %b to <2 x i64>
  %x = call <2 x i64> @PAND.2xi64(<2 x i64> %ci, <2 x i64> %ti)
  %y = call <2 x i64> @PANDN.xv(<2 x i64> %ci, <2 x i64> %fi)
  %z = call <2 x i64> @POR.2xi64(<2 x i64> %x, <2 x i64> %y)
  %r = bitcast <2 x i64> %z to <8 x i16>
  ret <8 x i16> %r
}

define <4 x i32> @vselect.4xi32(<4 x i1> "reg"="xv" %c, <4 x i32> "reg"="xv" %a, <4 x i32> "reg"="xv" %b) "recipe"="vselect.4xi32.expand" {
  %r = select <4 x i1> %c, <4 x i32> %a, <4 x i32> %b
  ret <4 x i32> %r
}

define <4 x i32> @vselect.4xi32.expand(<4 x i1> %c, <4 x i32> %a, <4 x i32> %b) {
  %cs = sext <4 x i1> %c to <4 x i32>
  %ci = bitcast <4 x i32> %cs to <2 x i64>
  %ti = bitcast <4 x i32> %a to <2 x i64>
  %fi = bitcast <4 x i32> %b to <2 x i64>
  %x = call <2 x i64> @PAND.2xi64(<2 x i64> %ci, <2 x i64> %ti)
  %y = call <2 x i64> @PANDN.xv(<2 x i64> %ci, <2 x i64> %fi)
  %z = call <2 x i64> @POR.2xi64(<2 x i64> %x, <2 x i64> %y)
  %r = bitcast <2 x i64> %z to <4 x i32>
  ret <4 x i32> %r
}

define <2 x i64> @vselect.2xi64(<2 x i1> "reg"="xv" %c, <2 x i64> "reg"="xv" %a, <2 x i64> "reg"="xv" %b) "recipe"="vselect.2xi64.expand" {
  %r = select <2 x i1> %c, <2 x i64> %a, <2 x i64> %b
  ret <2 x i64> %r
}

define <2 x i64> @vselect.2xi64.expand(<2 x i1> %c, <2 x i64> %a, <2 x i64> %b) {
  %cs = sext <2 x i1> %c to <2 x i64>
  %ci = bitcast <2 x i64> %cs to <2 x i64>
  %ti = bitcast <2 x i64> %a to <2 x i64>
  %fi = bitcast <2 x i64> %b to <2 x i64>
  %x = call <2 x i64> @PAND.2xi64(<2 x i64> %ci, <2 x i64> %ti)
  %y = call <2 x i64> @PANDN.xv(<2 x i64> %ci, <2 x i64> %fi)
  %z = call <2 x i64> @POR.2xi64(<2 x i64> %x, <2 x i64> %y)
  %r = bitcast <2 x i64> %z to <2 x i64>
  ret <2 x i64> %r
}

define <8 x i8> @vselect.8xi8(<8 x i1> "reg"="xv" %c, <8 x i8> "reg"="xh" %a, <8 x i8> "reg"="xh" %b) "recipe"="vselect.8xi8.expand" {
  %r = select <8 x i1> %c, <8 x i8> %a, <8 x i8> %b
  ret <8 x i8> %r
}

define <8 x i8> @vselect.8xi8.expand(<8 x i1> %c, <8 x i8> %a, <8 x i8> %b) {
  %cs = sext <8 x i1> %c to <8 x i8>
  %ci = bitcast <8 x i8> %cs to <2 x i32>
  %ti = bitcast <8 x i8> %a to <2 x i32>
  %fi = bitcast <8 x i8> %b to <2 x i32>
  %x = call <2 x i32> @PAND.2xi32(<2 x i32> %ci, <2 x i32> %ti)
  %y = call <2 x i32> @PANDN.xh(<2 x i32> %ci, <2 x i32> %fi)
  %z = call <2 x i32> @POR.2xi32(<2 x i32> %x, <2 x i32> %y)
  %r = bitcast <2 x i32> %z to <8 x i8>
  ret <8 x i8> %r
}

define <4 x i16> @vselect.4xi16(<4 x i1> "reg"="xv" %c, <4 x i16> "reg"="xh" %a, <4 x i16> "reg"="xh" %b) "recipe"="vselect.4xi16.expand" {
  %r = select <4 x i1> %c, <4 x i16> %a, <4 x i16> %b
  ret <4 x i16> %r
}

define <4 x i16> @vselect.4xi16.expand(<4 x i1> %c, <4 x i16> %a, <4 x i16> %b) {
  %cs = sext <4 x i1> %c to <4 x i16>
  %ci = bitcast <4 x i16> %cs to <2 x i32>
  %ti = bitcast <4 x i16> %a to <2 x i32>
  %fi = bitcast <4 x i16> %b to <2 x i32>
  %x = call <2 x i32> @PAND.2xi32(<2 x i32> %ci, <2 x i32> %ti)
  %y = call <2 x i32> @PANDN.xh(<2 x i32> %ci, <2 x i32> %fi)
  %z = call <2 x i32> @POR.2xi32(<2 x i32> %x, <2 x i32> %y)
  %r = bitcast <2 x i32> %z to <4 x i16>
  ret <4 x i16> %r
}

define <2 x i32> @vselect.2xi32(<2 x i1> "reg"="xv" %c, <2 x i32> "reg"="xh" %a, <2 x i32> "reg"="xh" %b) "recipe"="vselect.2xi32.expand" {
  %r = select <2 x i1> %c, <2 x i32> %a, <2 x i32> %b
  ret <2 x i32> %r
}

define <2 x i32> @vselect.2xi32.expand(<2 x i1> %c, <2 x i32> %a, <2 x i32> %b) {
  %cs = sext <2 x i1> %c to <2 x i32>
  %ci = bitcast <2 x i32> %cs to <2 x i32>
  %ti = bitcast <2 x i32> %a to <2 x i32>
  %fi = bitcast <2 x i32> %b to <2 x i32>
  %x = call <2 x i32> @PAND.2xi32(<2 x i32> %ci, <2 x i32> %ti)
  %y = call <2 x i32> @PANDN.xh(<2 x i32> %ci, <2 x i32> %fi)
  %z = call <2 x i32> @POR.2xi32(<2 x i32> %x, <2 x i32> %y)
  %r = bitcast <2 x i32> %z to <2 x i32>
  ret <2 x i32> %r
}

define <4 x float> @vselect.4xf32(<4 x i1> "reg"="xv" %c, <4 x float> "reg"="xv" %a, <4 x float> "reg"="xv" %b) "recipe"="vselect.4xf32.expand" {
  %r = select <4 x i1> %c, <4 x float> %a, <4 x float> %b
  ret <4 x float> %r
}

define <4 x float> @vselect.4xf32.expand(<4 x i1> %c, <4 x float> %a, <4 x float> %b) {
  %cs = sext <4 x i1> %c to <4 x i32>
  %ci = bitcast <4 x i32> %cs to <2 x i64>
  %ti = bitcast <4 x float> %a to <2 x i64>
  %fi = bitcast <4 x float> %b to <2 x i64>
  %x = call <2 x i64> @PAND.2xi64(<2 x i64> %ci, <2 x i64> %ti)
  %y = call <2 x i64> @PANDN.xv(<2 x i64> %ci, <2 x i64> %fi)
  %z = call <2 x i64> @POR.2xi64(<2 x i64> %x, <2 x i64> %y)
  %r = bitcast <2 x i64> %z to <4 x float>
  ret <4 x float> %r
}

define <2 x double> @vselect.2xf64(<2 x i1> "reg"="xv" %c, <2 x double> "reg"="xv" %a, <2 x double> "reg"="xv" %b) "recipe"="vselect.2xf64.expand" {
  %r = select <2 x i1> %c, <2 x double> %a, <2 x double> %b
  ret <2 x double> %r
}

define <2 x double> @vselect.2xf64.expand(<2 x i1> %c, <2 x double> %a, <2 x double> %b) {
  %cs = sext <2 x i1> %c to <2 x i64>
  %ci = bitcast <2 x i64> %cs to <2 x i64>
  %ti = bitcast <2 x double> %a to <2 x i64>
  %fi = bitcast <2 x double> %b to <2 x i64>
  %x = call <2 x i64> @PAND.2xi64(<2 x i64> %ci, <2 x i64> %ti)
  %y = call <2 x i64> @PANDN.xv(<2 x i64> %ci, <2 x i64> %fi)
  %z = call <2 x i64> @POR.2xi64(<2 x i64> %x, <2 x i64> %y)
  %r = bitcast <2 x i64> %z to <2 x double>
  ret <2 x double> %r
}

define <2 x float> @vselect.2xf32(<2 x i1> "reg"="xv" %c, <2 x float> "reg"="xh" %a, <2 x float> "reg"="xh" %b) "recipe"="vselect.2xf32.expand" {
  %r = select <2 x i1> %c, <2 x float> %a, <2 x float> %b
  ret <2 x float> %r
}

define <2 x float> @vselect.2xf32.expand(<2 x i1> %c, <2 x float> %a, <2 x float> %b) {
  %cs = sext <2 x i1> %c to <2 x i32>
  %ci = bitcast <2 x i32> %cs to <2 x i32>
  %ti = bitcast <2 x float> %a to <2 x i32>
  %fi = bitcast <2 x float> %b to <2 x i32>
  %x = call <2 x i32> @PAND.2xi32(<2 x i32> %ci, <2 x i32> %ti)
  %y = call <2 x i32> @PANDN.xh(<2 x i32> %ci, <2 x i32> %fi)
  %z = call <2 x i32> @POR.2xi32(<2 x i32> %x, <2 x i32> %y)
  %r = bitcast <2 x i32> %z to <2 x float>
  ret <2 x float> %r
}

define i16 @PEXTRW.8xi16(<8 x i16> "reg"="xv" %v, i64 "imm" %k) "enc"="01100110 | rex(0, d, 0, v) | 00001111 | 11000101 | rr(d, v) | ib:8=k" "asm"="pextrw ${k}, {v}, {d:r32}" "dclass"="r32" "zext"="16" "nodecode" {
  %r = extractelement <8 x i16> %v, i64 %k
  ret i16 %r
}

define <8 x i16> @PINSRW.8xi16(<8 x i16> "reg"="xv" "tied" %v, i16 "reg"="r32" %e, i64 "imm" %k) "enc"="01100110 | rex(0, d, 0, e) | 00001111 | 11000100 | rr(d, e) | ib:8=k" "asm"="pinsrw ${k}, {e:r32}, {d}" "dclass"="xv" "nodecode" {
  %r = insertelement <8 x i16> %v, i16 %e, i64 %k
  ret <8 x i16> %r
}

define i16 @PEXTRW.4xi16(<4 x i16> "reg"="xh" %v, i64 "imm" %k) "enc"="01100110 | rex(0, d, 0, v) | 00001111 | 11000101 | rr(d, v) | ib:8=k" "asm"="pextrw ${k}, {v}, {d:r32}" "dclass"="r32" "zext"="16" "nodecode" {
  %r = extractelement <4 x i16> %v, i64 %k
  ret i16 %r
}

define <4 x i16> @PINSRW.4xi16(<4 x i16> "reg"="xh" "tied" %v, i16 "reg"="r32" %e, i64 "imm" %k) "enc"="01100110 | rex(0, d, 0, e) | 00001111 | 11000100 | rr(d, e) | ib:8=k" "asm"="pinsrw ${k}, {e:r32}, {d}" "dclass"="xh" "nodecode" {
  %r = insertelement <4 x i16> %v, i16 %e, i64 %k
  ret <4 x i16> %r
}

define i32 @MOVDlane0.4xi32(<4 x i32> "reg"="xv" %v) "enc"="01100110 | rex(0, v, 0, d) | 00001111 | 01111110 | rr(v, d)" "asm"="movd {v}, {d}" "dclass"="r32" "nodecode" {
  %r = extractelement <4 x i32> %v, i64 0
  ret i32 %r
}

define <4 x i32> @PSHUFDlane.4xi32(<4 x i32> "reg"="xv" %v, i64 "imm" %k) "enc"="01100110 | rex(0, d, 0, v) | 00001111 | 01110000 | rr(d, v) | ib:8=k" "asm"="pshufd ${k}, {v}, {d}" "dclass"="xv" "nomatch" "nodecode" {
  ret <4 x i32> %v
}

define i32 @lane.4xi32(<4 x i32> "reg"="xv" %v, i64 %k) "recipe"="lane.4xi32.expand" {
  %r = extractelement <4 x i32> %v, i64 %k
  ret i32 %r
}

define i32 @lane.4xi32.expand(<4 x i32> %v, i64 %k) {
  %s = call <4 x i32> @PSHUFDlane.4xi32(<4 x i32> %v, i64 %k)
  %r = call i32 @MOVDlane0.4xi32(<4 x i32> %s)
  ret i32 %r
}

define i32 @MOVDlane0.2xi32(<2 x i32> "reg"="xh" %v) "enc"="01100110 | rex(0, v, 0, d) | 00001111 | 01111110 | rr(v, d)" "asm"="movd {v}, {d}" "dclass"="r32" "nodecode" {
  %r = extractelement <2 x i32> %v, i64 0
  ret i32 %r
}

define <2 x i32> @PSHUFDlane.2xi32(<2 x i32> "reg"="xh" %v, i64 "imm" %k) "enc"="01100110 | rex(0, d, 0, v) | 00001111 | 01110000 | rr(d, v) | ib:8=k" "asm"="pshufd ${k}, {v}, {d}" "dclass"="xh" "nomatch" "nodecode" {
  ret <2 x i32> %v
}

define i32 @lane.2xi32(<2 x i32> "reg"="xh" %v, i64 %k) "recipe"="lane.2xi32.expand" {
  %r = extractelement <2 x i32> %v, i64 %k
  ret i32 %r
}

define i32 @lane.2xi32.expand(<2 x i32> %v, i64 %k) {
  %s = call <2 x i32> @PSHUFDlane.2xi32(<2 x i32> %v, i64 %k)
  %r = call i32 @MOVDlane0.2xi32(<2 x i32> %s)
  ret i32 %r
}

define i64 @MOVQlane0(<2 x i64> "reg"="xv" %v) "enc"="01100110 | rex(1, v, 0, d) | 00001111 | 01111110 | rr(v, d)" "asm"="movq {v}, {d}" "dclass"="r64" "nodecode" {
  %r = extractelement <2 x i64> %v, i64 0
  ret i64 %r
}

define <2 x i64> @PUNPCKHQDQself(<2 x i64> "reg"="xv" "tied" %v) "enc"="01100110 | rex(0, d, 0, d) | 00001111 | 01101101 | rr(d, d)" "asm"="punpckhqdq {d}, {d}" "dclass"="xv" "nomatch" "nodecode" {
  ret <2 x i64> %v
}

define i64 @lane1.2xi64(<2 x i64> "reg"="xv" %v) "recipe"="lane1.2xi64.expand" {
  %r = extractelement <2 x i64> %v, i64 1
  ret i64 %r
}

define i64 @lane1.2xi64.expand(<2 x i64> %v) {
  %s = call <2 x i64> @PUNPCKHQDQself(<2 x i64> %v)
  %r = call i64 @MOVQlane0(<2 x i64> %s)
  ret i64 %r
}

define float @PSHUFDf.4xf32(<4 x float> "reg"="xv" %v, i64 "imm" %k) "enc"="01100110 | rex(0, d, 0, v) | 00001111 | 01110000 | rr(d, v) | ib:8=k" "asm"="pshufd ${k}, {v}, {d}" "dclass"="s" "nodecode" {
  %r = extractelement <4 x float> %v, i64 %k
  ret float %r
}

define float @PSHUFDf.2xf32(<2 x float> "reg"="xh" %v, i64 "imm" %k) "enc"="01100110 | rex(0, d, 0, v) | 00001111 | 01110000 | rr(d, v) | ib:8=k" "asm"="pshufd ${k}, {v}, {d}" "dclass"="s" "nodecode" {
  %r = extractelement <2 x float> %v, i64 %k
  ret float %r
}

define double @MOVAPDlane0(<2 x double> "reg"="xv" %v) "enc"="rex(0, d, 0, v) | 00001111 | 00101000 | rr(d, v)" "asm"="movaps {v}, {d}" "dclass"="d" "nodecode" {
  %r = extractelement <2 x double> %v, i64 0
  ret double %r
}

define double @UNPCKHPDlane1(<2 x double> "reg"="xv" "tied" %v) "enc"="01100110 | rex(0, d, 0, d) | 00001111 | 00010101 | rr(d, d)" "asm"="unpckhpd {d}, {d}" "dclass"="d" "nodecode" {
  %r = extractelement <2 x double> %v, i64 1
  ret double %r
}

define <4 x i32> @MOVDto(i32 "reg"="r32" %n) "enc"="01100110 | rex(0, d, 0, n) | 00001111 | 01101110 | rr(d, n)" "asm"="movd {n}, {d}" "dclass"="xv" "nodecode" {
  %r = insertelement <4 x i32> zeroinitializer, i32 %n, i64 0
  ret <4 x i32> %r
}

define <2 x i64> @MOVQto(i64 "reg"="r64" %n) "enc"="01100110 | rex(1, d, 0, n) | 00001111 | 01101110 | rr(d, n)" "asm"="movq {n}, {d}" "dclass"="xv" "role"="vec.from.x" "nodecode" {
  %r = insertelement <2 x i64> zeroinitializer, i64 %n, i64 0
  ret <2 x i64> %r
}

define <8 x i8> @MOVQtoh(i64 "reg"="r64" %n) "enc"="01100110 | rex(1, d, 0, n) | 00001111 | 01101110 | rr(d, n)" "asm"="movq {n}, {d}" "dclass"="xh" "nodecode" {
  %r = bitcast i64 %n to <8 x i8>
  ret <8 x i8> %r
}

define i64 @MOVQfromh(<8 x i8> "reg"="xh" %v) "enc"="01100110 | rex(1, v, 0, d) | 00001111 | 01111110 | rr(v, d)" "asm"="movq {v}, {d}" "dclass"="r64" "nodecode" {
  %r = bitcast <8 x i8> %v to i64
  ret i64 %r
}

define <2 x i64> @PUNPCKLQDQ(<2 x i64> "reg"="xv" "tied" %n, <2 x i64> "reg"="xv" %m) "enc"="01100110 | rex(0, d, 0, m) | 00001111 | 01101100 | rr(d, m)" "asm"="punpcklqdq {m}, {d}" "dclass"="xv" "role"="vec.join.d" "nomatch" "nodecode" {
  ret <2 x i64> %n
}

define <16 x i8> @BITS.16x8(<16 x i1> "reg"="xv" %m) "enc"="rex(0, d, 0, m) | 00001111 | 00101000 | rr(d, m)" "asm"="movaps {m}, {d}" "dclass"="xv" "nodecode" {
  %r = call <16 x i8> @llvm.epitome.mask.bits.v16i8(<16 x i1> %m)
  ret <16 x i8> %r
}

define <16 x i1> @MASK.16x8(<16 x i8> "reg"="xv" %v) "enc"="rex(0, d, 0, v) | 00001111 | 00101000 | rr(d, v)" "asm"="movaps {v}, {d}" "dclass"="xv" "nodecode" {
  %r = call <16 x i1> @llvm.epitome.bits.mask.v16i8(<16 x i8> %v)
  ret <16 x i1> %r
}

declare <16 x i8> @llvm.epitome.mask.bits.v16i8(<16 x i1>)
declare <16 x i1> @llvm.epitome.bits.mask.v16i8(<16 x i8>)

define <8 x i16> @BITS.8x16(<8 x i1> "reg"="xv" %m) "enc"="rex(0, d, 0, m) | 00001111 | 00101000 | rr(d, m)" "asm"="movaps {m}, {d}" "dclass"="xv" "nodecode" {
  %r = call <8 x i16> @llvm.epitome.mask.bits.v8i16(<8 x i1> %m)
  ret <8 x i16> %r
}

define <8 x i1> @MASK.8x16(<8 x i16> "reg"="xv" %v) "enc"="rex(0, d, 0, v) | 00001111 | 00101000 | rr(d, v)" "asm"="movaps {v}, {d}" "dclass"="xv" "nodecode" {
  %r = call <8 x i1> @llvm.epitome.bits.mask.v8i16(<8 x i16> %v)
  ret <8 x i1> %r
}

declare <8 x i16> @llvm.epitome.mask.bits.v8i16(<8 x i1>)
declare <8 x i1> @llvm.epitome.bits.mask.v8i16(<8 x i16>)

define <4 x i32> @BITS.4x32(<4 x i1> "reg"="xv" %m) "enc"="rex(0, d, 0, m) | 00001111 | 00101000 | rr(d, m)" "asm"="movaps {m}, {d}" "dclass"="xv" "nodecode" {
  %r = call <4 x i32> @llvm.epitome.mask.bits.v4i32(<4 x i1> %m)
  ret <4 x i32> %r
}

define <4 x i1> @MASK.4x32(<4 x i32> "reg"="xv" %v) "enc"="rex(0, d, 0, v) | 00001111 | 00101000 | rr(d, v)" "asm"="movaps {v}, {d}" "dclass"="xv" "nodecode" {
  %r = call <4 x i1> @llvm.epitome.bits.mask.v4i32(<4 x i32> %v)
  ret <4 x i1> %r
}

declare <4 x i32> @llvm.epitome.mask.bits.v4i32(<4 x i1>)
declare <4 x i1> @llvm.epitome.bits.mask.v4i32(<4 x i32>)

define <2 x i64> @BITS.2x64(<2 x i1> "reg"="xv" %m) "enc"="rex(0, d, 0, m) | 00001111 | 00101000 | rr(d, m)" "asm"="movaps {m}, {d}" "dclass"="xv" "nodecode" {
  %r = call <2 x i64> @llvm.epitome.mask.bits.v2i64(<2 x i1> %m)
  ret <2 x i64> %r
}

define <2 x i1> @MASK.2x64(<2 x i64> "reg"="xv" %v) "enc"="rex(0, d, 0, v) | 00001111 | 00101000 | rr(d, v)" "asm"="movaps {v}, {d}" "dclass"="xv" "nodecode" {
  %r = call <2 x i1> @llvm.epitome.bits.mask.v2i64(<2 x i64> %v)
  ret <2 x i1> %r
}

declare <2 x i64> @llvm.epitome.mask.bits.v2i64(<2 x i1>)
declare <2 x i1> @llvm.epitome.bits.mask.v2i64(<2 x i64>)

define <8 x i8> @BITS.8x8(<8 x i1> "reg"="xv" %m) "enc"="rex(0, d, 0, m) | 00001111 | 00101000 | rr(d, m)" "asm"="movaps {m}, {d}" "dclass"="xh" "nodecode" {
  %r = call <8 x i8> @llvm.epitome.mask.bits.v8i8(<8 x i1> %m)
  ret <8 x i8> %r
}

define <8 x i1> @MASK.8x8(<8 x i8> "reg"="xh" %v) "enc"="rex(0, d, 0, v) | 00001111 | 00101000 | rr(d, v)" "asm"="movaps {v}, {d}" "dclass"="xv" "nodecode" {
  %r = call <8 x i1> @llvm.epitome.bits.mask.v8i8(<8 x i8> %v)
  ret <8 x i1> %r
}

declare <8 x i8> @llvm.epitome.mask.bits.v8i8(<8 x i1>)
declare <8 x i1> @llvm.epitome.bits.mask.v8i8(<8 x i8>)

define <4 x i16> @BITS.4x16(<4 x i1> "reg"="xv" %m) "enc"="rex(0, d, 0, m) | 00001111 | 00101000 | rr(d, m)" "asm"="movaps {m}, {d}" "dclass"="xh" "nodecode" {
  %r = call <4 x i16> @llvm.epitome.mask.bits.v4i16(<4 x i1> %m)
  ret <4 x i16> %r
}

define <4 x i1> @MASK.4x16(<4 x i16> "reg"="xh" %v) "enc"="rex(0, d, 0, v) | 00001111 | 00101000 | rr(d, v)" "asm"="movaps {v}, {d}" "dclass"="xv" "nodecode" {
  %r = call <4 x i1> @llvm.epitome.bits.mask.v4i16(<4 x i16> %v)
  ret <4 x i1> %r
}

declare <4 x i16> @llvm.epitome.mask.bits.v4i16(<4 x i1>)
declare <4 x i1> @llvm.epitome.bits.mask.v4i16(<4 x i16>)

define <2 x i32> @BITS.2x32(<2 x i1> "reg"="xv" %m) "enc"="rex(0, d, 0, m) | 00001111 | 00101000 | rr(d, m)" "asm"="movaps {m}, {d}" "dclass"="xh" "nodecode" {
  %r = call <2 x i32> @llvm.epitome.mask.bits.v2i32(<2 x i1> %m)
  ret <2 x i32> %r
}

define <2 x i1> @MASK.2x32(<2 x i32> "reg"="xh" %v) "enc"="rex(0, d, 0, v) | 00001111 | 00101000 | rr(d, v)" "asm"="movaps {v}, {d}" "dclass"="xv" "nodecode" {
  %r = call <2 x i1> @llvm.epitome.bits.mask.v2i32(<2 x i32> %v)
  ret <2 x i1> %r
}

declare <2 x i32> @llvm.epitome.mask.bits.v2i32(<2 x i1>)
declare <2 x i1> @llvm.epitome.bits.mask.v2i32(<2 x i32>)

define <4 x i32> @PSHUFD(<4 x i32> "reg"="xv" %v, i64 "imm" %k) "enc"="01100110 | rex(0, d, 0, v) | 00001111 | 01110000 | rr(d, v) | ib:8=k" "asm"="pshufd ${k}, {v}, {d}" "dclass"="xv" "nomatch" "nodecode" {
  ret <4 x i32> %v
}

define <8 x i16> @PSHUFLW(<8 x i16> "reg"="xv" %v, i64 "imm" %k) "enc"="11110010 | rex(0, d, 0, v) | 00001111 | 01110000 | rr(d, v) | ib:8=k" "asm"="pshuflw ${k}, {v}, {d}" "dclass"="xv" "nomatch" "nodecode" {
  ret <8 x i16> %v
}

define <16 x i8> @PUNPCKLBWself(<16 x i8> "reg"="xv" "tied" %v) "enc"="01100110 | rex(0, d, 0, d) | 00001111 | 01100000 | rr(d, d)" "asm"="punpcklbw {d}, {d}" "dclass"="xv" "nomatch" "nodecode" {
  ret <16 x i8> %v
}

define <4 x i32> @splat.4xi32(i32 "reg"="r32" %x) "recipe"="splat.4xi32.expand" {
  %i = insertelement <4 x i32> poison, i32 %x, i64 0
  %r = shufflevector <4 x i32> %i, <4 x i32> poison, <4 x i32> zeroinitializer
  ret <4 x i32> %r
}

define <4 x i32> @splat.4xi32.expand(i32 %x) {
  %v = call <4 x i32> @MOVDto(i32 %x)
  %s = call <4 x i32> @PSHUFD(<4 x i32> %v, i64 0)
  %r = bitcast <4 x i32> %s to <4 x i32>
  ret <4 x i32> %r
}

define <2 x i32> @splat.2xi32(i32 "reg"="r32" %x) "recipe"="splat.2xi32.expand" {
  %i = insertelement <2 x i32> poison, i32 %x, i64 0
  %r = shufflevector <2 x i32> %i, <2 x i32> poison, <2 x i32> zeroinitializer
  ret <2 x i32> %r
}

define <2 x i32> @splat.2xi32.expand(i32 %x) {
  %v = call <4 x i32> @MOVDto(i32 %x)
  %s = call <4 x i32> @PSHUFD(<4 x i32> %v, i64 0)
  %r = bitcast <4 x i32> %s to <2 x i32>
  ret <2 x i32> %r
}

define <2 x i64> @splat.2xi64(i64 "reg"="r64" %x) "recipe"="splat.2xi64.expand" {
  %i = insertelement <2 x i64> poison, i64 %x, i64 0
  %r = shufflevector <2 x i64> %i, <2 x i64> poison, <2 x i32> zeroinitializer
  ret <2 x i64> %r
}

define <2 x i64> @splat.2xi64.expand(i64 %x) {
  %v = call <2 x i64> @MOVQto(i64 %x)
  %vi = bitcast <2 x i64> %v to <4 x i32>
  %s = call <4 x i32> @PSHUFD(<4 x i32> %vi, i64 68)
  %r = bitcast <4 x i32> %s to <2 x i64>
  ret <2 x i64> %r
}

define <8 x i16> @splat.8xi16(i16 "reg"="r32" %x) "recipe"="splat.8xi16.expand" {
  %i = insertelement <8 x i16> poison, i16 %x, i64 0
  %r = shufflevector <8 x i16> %i, <8 x i16> poison, <8 x i32> zeroinitializer
  ret <8 x i16> %r
}

define <8 x i16> @splat.8xi16.expand(i16 %x) {
  %w = zext i16 %x to i32
  %v = call <4 x i32> @MOVDto(i32 %w)
  %h = bitcast <4 x i32> %v to <8 x i16>
  %l = call <8 x i16> @PSHUFLW(<8 x i16> %h, i64 0)
  %v3 = bitcast <8 x i16> %l to <4 x i32>
  %s = call <4 x i32> @PSHUFD(<4 x i32> %v3, i64 0)
  %r = bitcast <4 x i32> %s to <8 x i16>
  ret <8 x i16> %r
}

define <4 x i16> @splat.4xi16(i16 "reg"="r32" %x) "recipe"="splat.4xi16.expand" {
  %i = insertelement <4 x i16> poison, i16 %x, i64 0
  %r = shufflevector <4 x i16> %i, <4 x i16> poison, <4 x i32> zeroinitializer
  ret <4 x i16> %r
}

define <4 x i16> @splat.4xi16.expand(i16 %x) {
  %w = zext i16 %x to i32
  %v = call <4 x i32> @MOVDto(i32 %w)
  %h = bitcast <4 x i32> %v to <8 x i16>
  %l = call <8 x i16> @PSHUFLW(<8 x i16> %h, i64 0)
  %v3 = bitcast <8 x i16> %l to <4 x i32>
  %s = call <4 x i32> @PSHUFD(<4 x i32> %v3, i64 0)
  %r = bitcast <4 x i32> %s to <4 x i16>
  ret <4 x i16> %r
}

define <16 x i8> @splat.16xi8(i8 "reg"="r32" %x) "recipe"="splat.16xi8.expand" {
  %i = insertelement <16 x i8> poison, i8 %x, i64 0
  %r = shufflevector <16 x i8> %i, <16 x i8> poison, <16 x i32> zeroinitializer
  ret <16 x i8> %r
}

define <16 x i8> @splat.16xi8.expand(i8 %x) {
  %w = zext i8 %x to i32
  %v = call <4 x i32> @MOVDto(i32 %w)
  %vb = bitcast <4 x i32> %v to <16 x i8>
  %u = call <16 x i8> @PUNPCKLBWself(<16 x i8> %vb)
  %v2 = bitcast <16 x i8> %u to <4 x i32>
  %h = bitcast <4 x i32> %v2 to <8 x i16>
  %l = call <8 x i16> @PSHUFLW(<8 x i16> %h, i64 0)
  %v3 = bitcast <8 x i16> %l to <4 x i32>
  %s = call <4 x i32> @PSHUFD(<4 x i32> %v3, i64 0)
  %r = bitcast <4 x i32> %s to <16 x i8>
  ret <16 x i8> %r
}

define <8 x i8> @splat.8xi8(i8 "reg"="r32" %x) "recipe"="splat.8xi8.expand" {
  %i = insertelement <8 x i8> poison, i8 %x, i64 0
  %r = shufflevector <8 x i8> %i, <8 x i8> poison, <8 x i32> zeroinitializer
  ret <8 x i8> %r
}

define <8 x i8> @splat.8xi8.expand(i8 %x) {
  %w = zext i8 %x to i32
  %v = call <4 x i32> @MOVDto(i32 %w)
  %vb = bitcast <4 x i32> %v to <16 x i8>
  %u = call <16 x i8> @PUNPCKLBWself(<16 x i8> %vb)
  %v2 = bitcast <16 x i8> %u to <4 x i32>
  %h = bitcast <4 x i32> %v2 to <8 x i16>
  %l = call <8 x i16> @PSHUFLW(<8 x i16> %h, i64 0)
  %v3 = bitcast <8 x i16> %l to <4 x i32>
  %s = call <4 x i32> @PSHUFD(<4 x i32> %v3, i64 0)
  %r = bitcast <4 x i32> %s to <8 x i8>
  ret <8 x i8> %r
}

define <4 x float> @splat.4xf32(float "reg"="s" %x) "recipe"="splat.4xf32.expand" {
  %i = insertelement <4 x float> poison, float %x, i64 0
  %r = shufflevector <4 x float> %i, <4 x float> poison, <4 x i32> zeroinitializer
  ret <4 x float> %r
}

define <4 x float> @splat.4xf32.expand(float %x) {
  %vi = bitcast float %x to <4 x i32>
  %s = call <4 x i32> @PSHUFD(<4 x i32> %vi, i64 0)
  %r = bitcast <4 x i32> %s to <4 x float>
  ret <4 x float> %r
}

define <2 x float> @splat.2xf32(float "reg"="s" %x) "recipe"="splat.2xf32.expand" {
  %i = insertelement <2 x float> poison, float %x, i64 0
  %r = shufflevector <2 x float> %i, <2 x float> poison, <2 x i32> zeroinitializer
  ret <2 x float> %r
}

define <2 x float> @splat.2xf32.expand(float %x) {
  %vi = bitcast float %x to <4 x i32>
  %s = call <4 x i32> @PSHUFD(<4 x i32> %vi, i64 0)
  %r = bitcast <4 x i32> %s to <2 x float>
  ret <2 x float> %r
}

define <2 x double> @splat.2xf64(double "reg"="d" %x) "recipe"="splat.2xf64.expand" {
  %i = insertelement <2 x double> poison, double %x, i64 0
  %r = shufflevector <2 x double> %i, <2 x double> poison, <2 x i32> zeroinitializer
  ret <2 x double> %r
}

define <2 x double> @splat.2xf64.expand(double %x) {
  %vi = bitcast double %x to <4 x i32>
  %s = call <4 x i32> @PSHUFD(<4 x i32> %vi, i64 68)
  %r = bitcast <4 x i32> %s to <2 x double>
  ret <2 x double> %r
}

define <8 x i8> @LOW.16xi8(<16 x i8> "reg"="xv" %v) "enc"="rex(0, d, 0, v) | 00001111 | 00101000 | rr(d, v)" "asm"="movaps {v}, {d}" "dclass"="xh" "nodecode" {
  %r = shufflevector <16 x i8> %v, <16 x i8> poison, <8 x i32> <i32 0, i32 1, i32 2, i32 3, i32 4, i32 5, i32 6, i32 7>
  ret <8 x i8> %r
}

define <8 x i8> @HIGH.16xi8(<16 x i8> "reg"="xv" %v) "enc"="01100110 | rex(0, d, 0, v) | 00001111 | 01110000 | rr(d, v) | 11101110" "asm"="pshufd $238, {v}, {d}" "dclass"="xh" "nodecode" {
  %r = shufflevector <16 x i8> %v, <16 x i8> poison, <8 x i32> <i32 8, i32 9, i32 10, i32 11, i32 12, i32 13, i32 14, i32 15>
  ret <8 x i8> %r
}

define <16 x i8> @JOIN.16xi8(<8 x i8> "reg"="xh" "tied" %a, <8 x i8> "reg"="xh" %b) "enc"="01100110 | rex(0, d, 0, b) | 00001111 | 01101100 | rr(d, b)" "asm"="punpcklqdq {b}, {d}" "dclass"="xv" "nodecode" {
  %r = shufflevector <8 x i8> %a, <8 x i8> %b, <16 x i32> <i32 0, i32 1, i32 2, i32 3, i32 4, i32 5, i32 6, i32 7, i32 8, i32 9, i32 10, i32 11, i32 12, i32 13, i32 14, i32 15>
  ret <16 x i8> %r
}

define <4 x i16> @LOW.8xi16(<8 x i16> "reg"="xv" %v) "enc"="rex(0, d, 0, v) | 00001111 | 00101000 | rr(d, v)" "asm"="movaps {v}, {d}" "dclass"="xh" "nodecode" {
  %r = shufflevector <8 x i16> %v, <8 x i16> poison, <4 x i32> <i32 0, i32 1, i32 2, i32 3>
  ret <4 x i16> %r
}

define <4 x i16> @HIGH.8xi16(<8 x i16> "reg"="xv" %v) "enc"="01100110 | rex(0, d, 0, v) | 00001111 | 01110000 | rr(d, v) | 11101110" "asm"="pshufd $238, {v}, {d}" "dclass"="xh" "nodecode" {
  %r = shufflevector <8 x i16> %v, <8 x i16> poison, <4 x i32> <i32 4, i32 5, i32 6, i32 7>
  ret <4 x i16> %r
}

define <8 x i16> @JOIN.8xi16(<4 x i16> "reg"="xh" "tied" %a, <4 x i16> "reg"="xh" %b) "enc"="01100110 | rex(0, d, 0, b) | 00001111 | 01101100 | rr(d, b)" "asm"="punpcklqdq {b}, {d}" "dclass"="xv" "nodecode" {
  %r = shufflevector <4 x i16> %a, <4 x i16> %b, <8 x i32> <i32 0, i32 1, i32 2, i32 3, i32 4, i32 5, i32 6, i32 7>
  ret <8 x i16> %r
}

define <2 x i32> @LOW.4xi32(<4 x i32> "reg"="xv" %v) "enc"="rex(0, d, 0, v) | 00001111 | 00101000 | rr(d, v)" "asm"="movaps {v}, {d}" "dclass"="xh" "nodecode" {
  %r = shufflevector <4 x i32> %v, <4 x i32> poison, <2 x i32> <i32 0, i32 1>
  ret <2 x i32> %r
}

define <2 x i32> @HIGH.4xi32(<4 x i32> "reg"="xv" %v) "enc"="01100110 | rex(0, d, 0, v) | 00001111 | 01110000 | rr(d, v) | 11101110" "asm"="pshufd $238, {v}, {d}" "dclass"="xh" "nodecode" {
  %r = shufflevector <4 x i32> %v, <4 x i32> poison, <2 x i32> <i32 2, i32 3>
  ret <2 x i32> %r
}

define <4 x i32> @JOIN.4xi32(<2 x i32> "reg"="xh" "tied" %a, <2 x i32> "reg"="xh" %b) "enc"="01100110 | rex(0, d, 0, b) | 00001111 | 01101100 | rr(d, b)" "asm"="punpcklqdq {b}, {d}" "dclass"="xv" "nodecode" {
  %r = shufflevector <2 x i32> %a, <2 x i32> %b, <4 x i32> <i32 0, i32 1, i32 2, i32 3>
  ret <4 x i32> %r
}

define <2 x float> @LOW.4xf32(<4 x float> "reg"="xv" %v) "enc"="rex(0, d, 0, v) | 00001111 | 00101000 | rr(d, v)" "asm"="movaps {v}, {d}" "dclass"="xh" "nodecode" {
  %r = shufflevector <4 x float> %v, <4 x float> poison, <2 x i32> <i32 0, i32 1>
  ret <2 x float> %r
}

define <2 x float> @HIGH.4xf32(<4 x float> "reg"="xv" %v) "enc"="01100110 | rex(0, d, 0, v) | 00001111 | 01110000 | rr(d, v) | 11101110" "asm"="pshufd $238, {v}, {d}" "dclass"="xh" "nodecode" {
  %r = shufflevector <4 x float> %v, <4 x float> poison, <2 x i32> <i32 2, i32 3>
  ret <2 x float> %r
}

define <4 x float> @JOIN.4xf32(<2 x float> "reg"="xh" "tied" %a, <2 x float> "reg"="xh" %b) "enc"="01100110 | rex(0, d, 0, b) | 00001111 | 01101100 | rr(d, b)" "asm"="punpcklqdq {b}, {d}" "dclass"="xv" "nodecode" {
  %r = shufflevector <2 x float> %a, <2 x float> %b, <4 x i32> <i32 0, i32 1, i32 2, i32 3>
  ret <4 x float> %r
}

define <8 x i16> @UNPCKLz.8xi8(<8 x i8> "reg"="xh" "tied" %v, <2 x i64> "reg"="xv" %z) "enc"="01100110 | rex(0, d, 0, z) | 00001111 | 01100000 | rr(d, z)" "asm"="punpcklbw {z}, {d}" "dclass"="xv" "nomatch" "nodecode" {
  ret <8 x i16> poison
}

define <8 x i16> @UNPCKLs.8xi8(<8 x i8> "reg"="xh" "tied" %v) "enc"="01100110 | rex(0, d, 0, d) | 00001111 | 01100000 | rr(d, d)" "asm"="punpcklbw {d}, {d}" "dclass"="xv" "nomatch" "nodecode" {
  ret <8 x i16> poison
}

define <8 x i16> @zext.8xi8(<8 x i8> "reg"="xh" %v) "recipe"="zext.8xi8.expand" {
  %r = zext <8 x i8> %v to <8 x i16>
  ret <8 x i16> %r
}

define <8 x i16> @zext.8xi8.expand(<8 x i8> %v) {
  %z = call <2 x i64> @PXORzero()
  %r = call <8 x i16> @UNPCKLz.8xi8(<8 x i8> %v, <2 x i64> %z)
  ret <8 x i16> %r
}

define <8 x i16> @sext.8xi8(<8 x i8> "reg"="xh" %v) "recipe"="sext.8xi8.expand" {
  %r = sext <8 x i8> %v to <8 x i16>
  ret <8 x i16> %r
}

define <8 x i16> @sext.8xi8.expand(<8 x i8> %v) {
  %u = call <8 x i16> @UNPCKLs.8xi8(<8 x i8> %v)
  %r = call <8 x i16> @PSRAWi.8xi16(<8 x i16> %u, i16 8)
  ret <8 x i16> %r
}

define <4 x i32> @UNPCKLz.4xi16(<4 x i16> "reg"="xh" "tied" %v, <2 x i64> "reg"="xv" %z) "enc"="01100110 | rex(0, d, 0, z) | 00001111 | 01100001 | rr(d, z)" "asm"="punpcklwd {z}, {d}" "dclass"="xv" "nomatch" "nodecode" {
  ret <4 x i32> poison
}

define <4 x i32> @UNPCKLs.4xi16(<4 x i16> "reg"="xh" "tied" %v) "enc"="01100110 | rex(0, d, 0, d) | 00001111 | 01100001 | rr(d, d)" "asm"="punpcklwd {d}, {d}" "dclass"="xv" "nomatch" "nodecode" {
  ret <4 x i32> poison
}

define <4 x i32> @zext.4xi16(<4 x i16> "reg"="xh" %v) "recipe"="zext.4xi16.expand" {
  %r = zext <4 x i16> %v to <4 x i32>
  ret <4 x i32> %r
}

define <4 x i32> @zext.4xi16.expand(<4 x i16> %v) {
  %z = call <2 x i64> @PXORzero()
  %r = call <4 x i32> @UNPCKLz.4xi16(<4 x i16> %v, <2 x i64> %z)
  ret <4 x i32> %r
}

define <4 x i32> @sext.4xi16(<4 x i16> "reg"="xh" %v) "recipe"="sext.4xi16.expand" {
  %r = sext <4 x i16> %v to <4 x i32>
  ret <4 x i32> %r
}

define <4 x i32> @sext.4xi16.expand(<4 x i16> %v) {
  %u = call <4 x i32> @UNPCKLs.4xi16(<4 x i16> %v)
  %r = call <4 x i32> @PSRADi.4xi32(<4 x i32> %u, i32 16)
  ret <4 x i32> %r
}

define <2 x i64> @UNPCKLz.2xi32(<2 x i32> "reg"="xh" "tied" %v, <2 x i64> "reg"="xv" %z) "enc"="01100110 | rex(0, d, 0, z) | 00001111 | 01100010 | rr(d, z)" "asm"="punpckldq {z}, {d}" "dclass"="xv" "nomatch" "nodecode" {
  ret <2 x i64> poison
}

define <2 x i64> @UNPCKLs.2xi32(<2 x i32> "reg"="xh" "tied" %v) "enc"="01100110 | rex(0, d, 0, d) | 00001111 | 01100010 | rr(d, d)" "asm"="punpckldq {d}, {d}" "dclass"="xv" "nomatch" "nodecode" {
  ret <2 x i64> poison
}

define <2 x i64> @zext.2xi32(<2 x i32> "reg"="xh" %v) "recipe"="zext.2xi32.expand" {
  %r = zext <2 x i32> %v to <2 x i64>
  ret <2 x i64> %r
}

define <2 x i64> @zext.2xi32.expand(<2 x i32> %v) {
  %z = call <2 x i64> @PXORzero()
  %r = call <2 x i64> @UNPCKLz.2xi32(<2 x i32> %v, <2 x i64> %z)
  ret <2 x i64> %r
}

define <2 x i64> @sext.2xi32(<2 x i32> "reg"="xh" %v) "recipe"="sext.2xi32.expand" {
  %r = sext <2 x i32> %v to <2 x i64>
  ret <2 x i64> %r
}

define <2 x i64> @sext.2xi32.expand(<2 x i32> %v) {
  %z = call <2 x i64> @PXORzero()
  %zi = bitcast <2 x i64> %z to <2 x i32>
  %s = call <2 x i1> @PCMPGTD.2xi32(<2 x i32> %zi, <2 x i32> %v)
  %si = sext <2 x i1> %s to <2 x i32>
  %sw = bitcast <2 x i32> %si to <2 x i64>
  %r = call <2 x i64> @UNPCKLz.2xi32(<2 x i32> %v, <2 x i64> %sw)
  ret <2 x i64> %r
}

define <2 x i64> @ALLONES() "enc"="01100110 | rex(0, d, 0, d) | 00001111 | 01110110 | rr(d, d)" "asm"="pcmpeqd {d}, {d}" "dclass"="xv" "nomatch" "nodecode" {
  ret <2 x i64> splat (i64 -1)
}

define <16 x i1> @slt.16xi8(<16 x i8> "reg"="xv" %a, <16 x i8> "reg"="xv" %b) "recipe"="slt.16xi8.expand" {
  %r = icmp slt <16 x i8> %a, %b
  ret <16 x i1> %r
}

define <16 x i1> @slt.16xi8.expand(<16 x i8> %a, <16 x i8> %b) {
  %r = call <16 x i1> @PCMPGTB.16xi8(<16 x i8> %b, <16 x i8> %a)
  ret <16 x i1> %r
}

define <16 x i1> @ne.16xi8(<16 x i8> "reg"="xv" %a, <16 x i8> "reg"="xv" %b) "recipe"="ne.16xi8.expand" {
  %r = icmp ne <16 x i8> %a, %b
  ret <16 x i1> %r
}

define <16 x i1> @ne.16xi8.expand(<16 x i8> %a, <16 x i8> %b) {
  %c = call <16 x i1> @PCMPEQB.16xi8(<16 x i8> %a, <16 x i8> %b)
  %xi = sext <16 x i1> %c to <16 x i8>
  %xw = bitcast <16 x i8> %xi to <2 x i64>
  %on = call <2 x i64> @ALLONES()
  %nw = call <2 x i64> @PXOR.2xi64(<2 x i64> %xw, <2 x i64> %on)
  %ni = bitcast <2 x i64> %nw to <16 x i8>
  %r = trunc <16 x i8> %ni to <16 x i1>
  ret <16 x i1> %r
}

define <16 x i1> @sge.16xi8(<16 x i8> "reg"="xv" %a, <16 x i8> "reg"="xv" %b) "recipe"="sge.16xi8.expand" {
  %r = icmp sge <16 x i8> %a, %b
  ret <16 x i1> %r
}

define <16 x i1> @sge.16xi8.expand(<16 x i8> %a, <16 x i8> %b) {
  %c = call <16 x i1> @PCMPGTB.16xi8(<16 x i8> %b, <16 x i8> %a)
  %xi = sext <16 x i1> %c to <16 x i8>
  %xw = bitcast <16 x i8> %xi to <2 x i64>
  %on = call <2 x i64> @ALLONES()
  %nw = call <2 x i64> @PXOR.2xi64(<2 x i64> %xw, <2 x i64> %on)
  %ni = bitcast <2 x i64> %nw to <16 x i8>
  %r = trunc <16 x i8> %ni to <16 x i1>
  ret <16 x i1> %r
}

define <16 x i1> @sle.16xi8(<16 x i8> "reg"="xv" %a, <16 x i8> "reg"="xv" %b) "recipe"="sle.16xi8.expand" {
  %r = icmp sle <16 x i8> %a, %b
  ret <16 x i1> %r
}

define <16 x i1> @sle.16xi8.expand(<16 x i8> %a, <16 x i8> %b) {
  %c = call <16 x i1> @PCMPGTB.16xi8(<16 x i8> %a, <16 x i8> %b)
  %xi = sext <16 x i1> %c to <16 x i8>
  %xw = bitcast <16 x i8> %xi to <2 x i64>
  %on = call <2 x i64> @ALLONES()
  %nw = call <2 x i64> @PXOR.2xi64(<2 x i64> %xw, <2 x i64> %on)
  %ni = bitcast <2 x i64> %nw to <16 x i8>
  %r = trunc <16 x i8> %ni to <16 x i1>
  ret <16 x i1> %r
}

define <16 x i8> @PMAXUB.16xi8(<16 x i8> "reg"="xv" "tied" %n, <16 x i8> "reg"="xv" %m) "enc"="01100110 | rex(0, d, 0, m) | 00001111 | 11011110 | rr(d, m)" "asm"="pmaxub {m}, {d}" "dclass"="xv" "nodecode" {
  %r = call <16 x i8> @llvm.umax.v16i8(<16 x i8> %n, <16 x i8> %m)
  ret <16 x i8> %r
}

define <16 x i8> @PMINUB.16xi8(<16 x i8> "reg"="xv" "tied" %n, <16 x i8> "reg"="xv" %m) "enc"="01100110 | rex(0, d, 0, m) | 00001111 | 11011010 | rr(d, m)" "asm"="pminub {m}, {d}" "dclass"="xv" "nodecode" {
  %r = call <16 x i8> @llvm.umin.v16i8(<16 x i8> %n, <16 x i8> %m)
  ret <16 x i8> %r
}

declare <16 x i8> @llvm.umax.v16i8(<16 x i8>, <16 x i8>)
declare <16 x i8> @llvm.umin.v16i8(<16 x i8>, <16 x i8>)

define <16 x i1> @uge.16xi8(<16 x i8> "reg"="xv" %a, <16 x i8> "reg"="xv" %b) "recipe"="uge.16xi8.expand" {
  %r = icmp uge <16 x i8> %a, %b
  ret <16 x i1> %r
}

define <16 x i1> @uge.16xi8.expand(<16 x i8> %a, <16 x i8> %b) {
  %m = call <16 x i8> @PMAXUB.16xi8(<16 x i8> %a, <16 x i8> %b)
  %c = call <16 x i1> @PCMPEQB.16xi8(<16 x i8> %m, <16 x i8> %a)
  ret <16 x i1> %c
}

define <16 x i1> @ule.16xi8(<16 x i8> "reg"="xv" %a, <16 x i8> "reg"="xv" %b) "recipe"="ule.16xi8.expand" {
  %r = icmp ule <16 x i8> %a, %b
  ret <16 x i1> %r
}

define <16 x i1> @ule.16xi8.expand(<16 x i8> %a, <16 x i8> %b) {
  %m = call <16 x i8> @PMAXUB.16xi8(<16 x i8> %b, <16 x i8> %a)
  %c = call <16 x i1> @PCMPEQB.16xi8(<16 x i8> %m, <16 x i8> %b)
  ret <16 x i1> %c
}

define <16 x i1> @ult.16xi8(<16 x i8> "reg"="xv" %a, <16 x i8> "reg"="xv" %b) "recipe"="ult.16xi8.expand" {
  %r = icmp ult <16 x i8> %a, %b
  ret <16 x i1> %r
}

define <16 x i1> @ult.16xi8.expand(<16 x i8> %a, <16 x i8> %b) {
  %m = call <16 x i8> @PMAXUB.16xi8(<16 x i8> %a, <16 x i8> %b)
  %c = call <16 x i1> @PCMPEQB.16xi8(<16 x i8> %m, <16 x i8> %a)
  %xi = sext <16 x i1> %c to <16 x i8>
  %xw = bitcast <16 x i8> %xi to <2 x i64>
  %on = call <2 x i64> @ALLONES()
  %nw = call <2 x i64> @PXOR.2xi64(<2 x i64> %xw, <2 x i64> %on)
  %ni = bitcast <2 x i64> %nw to <16 x i8>
  %r = trunc <16 x i8> %ni to <16 x i1>
  ret <16 x i1> %r
}

define <16 x i1> @ugt.16xi8(<16 x i8> "reg"="xv" %a, <16 x i8> "reg"="xv" %b) "recipe"="ugt.16xi8.expand" {
  %r = icmp ugt <16 x i8> %a, %b
  ret <16 x i1> %r
}

define <16 x i1> @ugt.16xi8.expand(<16 x i8> %a, <16 x i8> %b) {
  %m = call <16 x i8> @PMAXUB.16xi8(<16 x i8> %b, <16 x i8> %a)
  %c = call <16 x i1> @PCMPEQB.16xi8(<16 x i8> %m, <16 x i8> %b)
  %xi = sext <16 x i1> %c to <16 x i8>
  %xw = bitcast <16 x i8> %xi to <2 x i64>
  %on = call <2 x i64> @ALLONES()
  %nw = call <2 x i64> @PXOR.2xi64(<2 x i64> %xw, <2 x i64> %on)
  %ni = bitcast <2 x i64> %nw to <16 x i8>
  %r = trunc <16 x i8> %ni to <16 x i1>
  ret <16 x i1> %r
}

define <8 x i1> @slt.8xi16(<8 x i16> "reg"="xv" %a, <8 x i16> "reg"="xv" %b) "recipe"="slt.8xi16.expand" {
  %r = icmp slt <8 x i16> %a, %b
  ret <8 x i1> %r
}

define <8 x i1> @slt.8xi16.expand(<8 x i16> %a, <8 x i16> %b) {
  %r = call <8 x i1> @PCMPGTW.8xi16(<8 x i16> %b, <8 x i16> %a)
  ret <8 x i1> %r
}

define <8 x i1> @ne.8xi16(<8 x i16> "reg"="xv" %a, <8 x i16> "reg"="xv" %b) "recipe"="ne.8xi16.expand" {
  %r = icmp ne <8 x i16> %a, %b
  ret <8 x i1> %r
}

define <8 x i1> @ne.8xi16.expand(<8 x i16> %a, <8 x i16> %b) {
  %c = call <8 x i1> @PCMPEQW.8xi16(<8 x i16> %a, <8 x i16> %b)
  %xi = sext <8 x i1> %c to <8 x i16>
  %xw = bitcast <8 x i16> %xi to <2 x i64>
  %on = call <2 x i64> @ALLONES()
  %nw = call <2 x i64> @PXOR.2xi64(<2 x i64> %xw, <2 x i64> %on)
  %ni = bitcast <2 x i64> %nw to <8 x i16>
  %r = trunc <8 x i16> %ni to <8 x i1>
  ret <8 x i1> %r
}

define <8 x i1> @sge.8xi16(<8 x i16> "reg"="xv" %a, <8 x i16> "reg"="xv" %b) "recipe"="sge.8xi16.expand" {
  %r = icmp sge <8 x i16> %a, %b
  ret <8 x i1> %r
}

define <8 x i1> @sge.8xi16.expand(<8 x i16> %a, <8 x i16> %b) {
  %c = call <8 x i1> @PCMPGTW.8xi16(<8 x i16> %b, <8 x i16> %a)
  %xi = sext <8 x i1> %c to <8 x i16>
  %xw = bitcast <8 x i16> %xi to <2 x i64>
  %on = call <2 x i64> @ALLONES()
  %nw = call <2 x i64> @PXOR.2xi64(<2 x i64> %xw, <2 x i64> %on)
  %ni = bitcast <2 x i64> %nw to <8 x i16>
  %r = trunc <8 x i16> %ni to <8 x i1>
  ret <8 x i1> %r
}

define <8 x i1> @sle.8xi16(<8 x i16> "reg"="xv" %a, <8 x i16> "reg"="xv" %b) "recipe"="sle.8xi16.expand" {
  %r = icmp sle <8 x i16> %a, %b
  ret <8 x i1> %r
}

define <8 x i1> @sle.8xi16.expand(<8 x i16> %a, <8 x i16> %b) {
  %c = call <8 x i1> @PCMPGTW.8xi16(<8 x i16> %a, <8 x i16> %b)
  %xi = sext <8 x i1> %c to <8 x i16>
  %xw = bitcast <8 x i16> %xi to <2 x i64>
  %on = call <2 x i64> @ALLONES()
  %nw = call <2 x i64> @PXOR.2xi64(<2 x i64> %xw, <2 x i64> %on)
  %ni = bitcast <2 x i64> %nw to <8 x i16>
  %r = trunc <8 x i16> %ni to <8 x i1>
  ret <8 x i1> %r
}

define <8 x i16> @PSUBUSW.8xi16(<8 x i16> "reg"="xv" "tied" %n, <8 x i16> "reg"="xv" %m) "enc"="01100110 | rex(0, d, 0, m) | 00001111 | 11011001 | rr(d, m)" "asm"="psubusw {m}, {d}" "dclass"="xv" "nodecode" {
  %r = call <8 x i16> @llvm.usub.sat.v8i16(<8 x i16> %n, <8 x i16> %m)
  ret <8 x i16> %r
}

declare <8 x i16> @llvm.usub.sat.v8i16(<8 x i16>, <8 x i16>)

define <8 x i16> @PMAXSW.8xi16(<8 x i16> "reg"="xv" "tied" %n, <8 x i16> "reg"="xv" %m) "enc"="01100110 | rex(0, d, 0, m) | 00001111 | 11101110 | rr(d, m)" "asm"="pmaxsw {m}, {d}" "dclass"="xv" "nodecode" {
  %r = call <8 x i16> @llvm.smax.v8i16(<8 x i16> %n, <8 x i16> %m)
  ret <8 x i16> %r
}

declare <8 x i16> @llvm.smax.v8i16(<8 x i16>, <8 x i16>)

define <8 x i16> @PMINSW.8xi16(<8 x i16> "reg"="xv" "tied" %n, <8 x i16> "reg"="xv" %m) "enc"="01100110 | rex(0, d, 0, m) | 00001111 | 11101010 | rr(d, m)" "asm"="pminsw {m}, {d}" "dclass"="xv" "nodecode" {
  %r = call <8 x i16> @llvm.smin.v8i16(<8 x i16> %n, <8 x i16> %m)
  ret <8 x i16> %r
}

declare <8 x i16> @llvm.smin.v8i16(<8 x i16>, <8 x i16>)

define <8 x i1> @ule.8xi16(<8 x i16> "reg"="xv" %a, <8 x i16> "reg"="xv" %b) "recipe"="ule.8xi16.expand" {
  %r = icmp ule <8 x i16> %a, %b
  ret <8 x i1> %r
}

define <8 x i1> @ule.8xi16.expand(<8 x i16> %a, <8 x i16> %b) {
  %z = call <2 x i64> @PXORzero()
  %zt = bitcast <2 x i64> %z to <8 x i16>
  %d = call <8 x i16> @PSUBUSW.8xi16(<8 x i16> %a, <8 x i16> %b)
  %c = call <8 x i1> @PCMPEQW.8xi16(<8 x i16> %d, <8 x i16> %zt)
  ret <8 x i1> %c
}

define <8 x i1> @uge.8xi16(<8 x i16> "reg"="xv" %a, <8 x i16> "reg"="xv" %b) "recipe"="uge.8xi16.expand" {
  %r = icmp uge <8 x i16> %a, %b
  ret <8 x i1> %r
}

define <8 x i1> @uge.8xi16.expand(<8 x i16> %a, <8 x i16> %b) {
  %z = call <2 x i64> @PXORzero()
  %zt = bitcast <2 x i64> %z to <8 x i16>
  %d = call <8 x i16> @PSUBUSW.8xi16(<8 x i16> %b, <8 x i16> %a)
  %c = call <8 x i1> @PCMPEQW.8xi16(<8 x i16> %d, <8 x i16> %zt)
  ret <8 x i1> %c
}

define <8 x i1> @ugt.8xi16(<8 x i16> "reg"="xv" %a, <8 x i16> "reg"="xv" %b) "recipe"="ugt.8xi16.expand" {
  %r = icmp ugt <8 x i16> %a, %b
  ret <8 x i1> %r
}

define <8 x i1> @ugt.8xi16.expand(<8 x i16> %a, <8 x i16> %b) {
  %z = call <2 x i64> @PXORzero()
  %zt = bitcast <2 x i64> %z to <8 x i16>
  %d = call <8 x i16> @PSUBUSW.8xi16(<8 x i16> %a, <8 x i16> %b)
  %c = call <8 x i1> @PCMPEQW.8xi16(<8 x i16> %d, <8 x i16> %zt)
  %xi = sext <8 x i1> %c to <8 x i16>
  %xw = bitcast <8 x i16> %xi to <2 x i64>
  %on = call <2 x i64> @ALLONES()
  %nw = call <2 x i64> @PXOR.2xi64(<2 x i64> %xw, <2 x i64> %on)
  %ni = bitcast <2 x i64> %nw to <8 x i16>
  %r = trunc <8 x i16> %ni to <8 x i1>
  ret <8 x i1> %r
}

define <8 x i1> @ult.8xi16(<8 x i16> "reg"="xv" %a, <8 x i16> "reg"="xv" %b) "recipe"="ult.8xi16.expand" {
  %r = icmp ult <8 x i16> %a, %b
  ret <8 x i1> %r
}

define <8 x i1> @ult.8xi16.expand(<8 x i16> %a, <8 x i16> %b) {
  %z = call <2 x i64> @PXORzero()
  %zt = bitcast <2 x i64> %z to <8 x i16>
  %d = call <8 x i16> @PSUBUSW.8xi16(<8 x i16> %b, <8 x i16> %a)
  %c = call <8 x i1> @PCMPEQW.8xi16(<8 x i16> %d, <8 x i16> %zt)
  %xi = sext <8 x i1> %c to <8 x i16>
  %xw = bitcast <8 x i16> %xi to <2 x i64>
  %on = call <2 x i64> @ALLONES()
  %nw = call <2 x i64> @PXOR.2xi64(<2 x i64> %xw, <2 x i64> %on)
  %ni = bitcast <2 x i64> %nw to <8 x i16>
  %r = trunc <8 x i16> %ni to <8 x i1>
  ret <8 x i1> %r
}

define <4 x i1> @slt.4xi32(<4 x i32> "reg"="xv" %a, <4 x i32> "reg"="xv" %b) "recipe"="slt.4xi32.expand" {
  %r = icmp slt <4 x i32> %a, %b
  ret <4 x i1> %r
}

define <4 x i1> @slt.4xi32.expand(<4 x i32> %a, <4 x i32> %b) {
  %r = call <4 x i1> @PCMPGTD.4xi32(<4 x i32> %b, <4 x i32> %a)
  ret <4 x i1> %r
}

define <4 x i1> @ne.4xi32(<4 x i32> "reg"="xv" %a, <4 x i32> "reg"="xv" %b) "recipe"="ne.4xi32.expand" {
  %r = icmp ne <4 x i32> %a, %b
  ret <4 x i1> %r
}

define <4 x i1> @ne.4xi32.expand(<4 x i32> %a, <4 x i32> %b) {
  %c = call <4 x i1> @PCMPEQD.4xi32(<4 x i32> %a, <4 x i32> %b)
  %xi = sext <4 x i1> %c to <4 x i32>
  %xw = bitcast <4 x i32> %xi to <2 x i64>
  %on = call <2 x i64> @ALLONES()
  %nw = call <2 x i64> @PXOR.2xi64(<2 x i64> %xw, <2 x i64> %on)
  %ni = bitcast <2 x i64> %nw to <4 x i32>
  %r = trunc <4 x i32> %ni to <4 x i1>
  ret <4 x i1> %r
}

define <4 x i1> @sge.4xi32(<4 x i32> "reg"="xv" %a, <4 x i32> "reg"="xv" %b) "recipe"="sge.4xi32.expand" {
  %r = icmp sge <4 x i32> %a, %b
  ret <4 x i1> %r
}

define <4 x i1> @sge.4xi32.expand(<4 x i32> %a, <4 x i32> %b) {
  %c = call <4 x i1> @PCMPGTD.4xi32(<4 x i32> %b, <4 x i32> %a)
  %xi = sext <4 x i1> %c to <4 x i32>
  %xw = bitcast <4 x i32> %xi to <2 x i64>
  %on = call <2 x i64> @ALLONES()
  %nw = call <2 x i64> @PXOR.2xi64(<2 x i64> %xw, <2 x i64> %on)
  %ni = bitcast <2 x i64> %nw to <4 x i32>
  %r = trunc <4 x i32> %ni to <4 x i1>
  ret <4 x i1> %r
}

define <4 x i1> @sle.4xi32(<4 x i32> "reg"="xv" %a, <4 x i32> "reg"="xv" %b) "recipe"="sle.4xi32.expand" {
  %r = icmp sle <4 x i32> %a, %b
  ret <4 x i1> %r
}

define <4 x i1> @sle.4xi32.expand(<4 x i32> %a, <4 x i32> %b) {
  %c = call <4 x i1> @PCMPGTD.4xi32(<4 x i32> %a, <4 x i32> %b)
  %xi = sext <4 x i1> %c to <4 x i32>
  %xw = bitcast <4 x i32> %xi to <2 x i64>
  %on = call <2 x i64> @ALLONES()
  %nw = call <2 x i64> @PXOR.2xi64(<2 x i64> %xw, <2 x i64> %on)
  %ni = bitcast <2 x i64> %nw to <4 x i32>
  %r = trunc <4 x i32> %ni to <4 x i1>
  ret <4 x i1> %r
}

define <4 x i1> @ugt.4xi32(<4 x i32> "reg"="xv" %a, <4 x i32> "reg"="xv" %b) "recipe"="ugt.4xi32.expand" {
  %r = icmp ugt <4 x i32> %a, %b
  ret <4 x i1> %r
}

define <4 x i1> @ugt.4xi32.expand(<4 x i32> %a, <4 x i32> %b) {
  %o = call <2 x i64> @ALLONES()
  %ot = bitcast <2 x i64> %o to <4 x i32>
  %sg = call <4 x i32> @PSLLDi.4xi32(<4 x i32> %ot, i32 31)
  %fa = call <4 x i32> @PXOR.4xi32(<4 x i32> %a, <4 x i32> %sg)
  %fb = call <4 x i32> @PXOR.4xi32(<4 x i32> %b, <4 x i32> %sg)
  %c = call <4 x i1> @PCMPGTD.4xi32(<4 x i32> %fa, <4 x i32> %fb)
  ret <4 x i1> %c
}

define <4 x i1> @ult.4xi32(<4 x i32> "reg"="xv" %a, <4 x i32> "reg"="xv" %b) "recipe"="ult.4xi32.expand" {
  %r = icmp ult <4 x i32> %a, %b
  ret <4 x i1> %r
}

define <4 x i1> @ult.4xi32.expand(<4 x i32> %a, <4 x i32> %b) {
  %o = call <2 x i64> @ALLONES()
  %ot = bitcast <2 x i64> %o to <4 x i32>
  %sg = call <4 x i32> @PSLLDi.4xi32(<4 x i32> %ot, i32 31)
  %fa = call <4 x i32> @PXOR.4xi32(<4 x i32> %a, <4 x i32> %sg)
  %fb = call <4 x i32> @PXOR.4xi32(<4 x i32> %b, <4 x i32> %sg)
  %c = call <4 x i1> @PCMPGTD.4xi32(<4 x i32> %fb, <4 x i32> %fa)
  ret <4 x i1> %c
}

define <4 x i1> @ule.4xi32(<4 x i32> "reg"="xv" %a, <4 x i32> "reg"="xv" %b) "recipe"="ule.4xi32.expand" {
  %r = icmp ule <4 x i32> %a, %b
  ret <4 x i1> %r
}

define <4 x i1> @ule.4xi32.expand(<4 x i32> %a, <4 x i32> %b) {
  %o = call <2 x i64> @ALLONES()
  %ot = bitcast <2 x i64> %o to <4 x i32>
  %sg = call <4 x i32> @PSLLDi.4xi32(<4 x i32> %ot, i32 31)
  %fa = call <4 x i32> @PXOR.4xi32(<4 x i32> %a, <4 x i32> %sg)
  %fb = call <4 x i32> @PXOR.4xi32(<4 x i32> %b, <4 x i32> %sg)
  %c = call <4 x i1> @PCMPGTD.4xi32(<4 x i32> %fa, <4 x i32> %fb)
  %xi = sext <4 x i1> %c to <4 x i32>
  %xw = bitcast <4 x i32> %xi to <2 x i64>
  %on = call <2 x i64> @ALLONES()
  %nw = call <2 x i64> @PXOR.2xi64(<2 x i64> %xw, <2 x i64> %on)
  %ni = bitcast <2 x i64> %nw to <4 x i32>
  %r = trunc <4 x i32> %ni to <4 x i1>
  ret <4 x i1> %r
}

define <4 x i1> @uge.4xi32(<4 x i32> "reg"="xv" %a, <4 x i32> "reg"="xv" %b) "recipe"="uge.4xi32.expand" {
  %r = icmp uge <4 x i32> %a, %b
  ret <4 x i1> %r
}

define <4 x i1> @uge.4xi32.expand(<4 x i32> %a, <4 x i32> %b) {
  %o = call <2 x i64> @ALLONES()
  %ot = bitcast <2 x i64> %o to <4 x i32>
  %sg = call <4 x i32> @PSLLDi.4xi32(<4 x i32> %ot, i32 31)
  %fa = call <4 x i32> @PXOR.4xi32(<4 x i32> %a, <4 x i32> %sg)
  %fb = call <4 x i32> @PXOR.4xi32(<4 x i32> %b, <4 x i32> %sg)
  %c = call <4 x i1> @PCMPGTD.4xi32(<4 x i32> %fb, <4 x i32> %fa)
  %xi = sext <4 x i1> %c to <4 x i32>
  %xw = bitcast <4 x i32> %xi to <2 x i64>
  %on = call <2 x i64> @ALLONES()
  %nw = call <2 x i64> @PXOR.2xi64(<2 x i64> %xw, <2 x i64> %on)
  %ni = bitcast <2 x i64> %nw to <4 x i32>
  %r = trunc <4 x i32> %ni to <4 x i1>
  ret <4 x i1> %r
}

define <8 x i1> @slt.8xi8(<8 x i8> "reg"="xh" %a, <8 x i8> "reg"="xh" %b) "recipe"="slt.8xi8.expand" {
  %r = icmp slt <8 x i8> %a, %b
  ret <8 x i1> %r
}

define <8 x i1> @slt.8xi8.expand(<8 x i8> %a, <8 x i8> %b) {
  %r = call <8 x i1> @PCMPGTB.8xi8(<8 x i8> %b, <8 x i8> %a)
  ret <8 x i1> %r
}

define <8 x i1> @ne.8xi8(<8 x i8> "reg"="xh" %a, <8 x i8> "reg"="xh" %b) "recipe"="ne.8xi8.expand" {
  %r = icmp ne <8 x i8> %a, %b
  ret <8 x i1> %r
}

define <8 x i1> @ne.8xi8.expand(<8 x i8> %a, <8 x i8> %b) {
  %c = call <8 x i1> @PCMPEQB.8xi8(<8 x i8> %a, <8 x i8> %b)
  %xi = sext <8 x i1> %c to <8 x i8>
  %xw = bitcast <8 x i8> %xi to <2 x i64>
  %on = call <2 x i64> @ALLONES()
  %nw = call <2 x i64> @PXOR.2xi64(<2 x i64> %xw, <2 x i64> %on)
  %ni = bitcast <2 x i64> %nw to <8 x i8>
  %r = trunc <8 x i8> %ni to <8 x i1>
  ret <8 x i1> %r
}

define <8 x i1> @sge.8xi8(<8 x i8> "reg"="xh" %a, <8 x i8> "reg"="xh" %b) "recipe"="sge.8xi8.expand" {
  %r = icmp sge <8 x i8> %a, %b
  ret <8 x i1> %r
}

define <8 x i1> @sge.8xi8.expand(<8 x i8> %a, <8 x i8> %b) {
  %c = call <8 x i1> @PCMPGTB.8xi8(<8 x i8> %b, <8 x i8> %a)
  %xi = sext <8 x i1> %c to <8 x i8>
  %xw = bitcast <8 x i8> %xi to <2 x i64>
  %on = call <2 x i64> @ALLONES()
  %nw = call <2 x i64> @PXOR.2xi64(<2 x i64> %xw, <2 x i64> %on)
  %ni = bitcast <2 x i64> %nw to <8 x i8>
  %r = trunc <8 x i8> %ni to <8 x i1>
  ret <8 x i1> %r
}

define <8 x i1> @sle.8xi8(<8 x i8> "reg"="xh" %a, <8 x i8> "reg"="xh" %b) "recipe"="sle.8xi8.expand" {
  %r = icmp sle <8 x i8> %a, %b
  ret <8 x i1> %r
}

define <8 x i1> @sle.8xi8.expand(<8 x i8> %a, <8 x i8> %b) {
  %c = call <8 x i1> @PCMPGTB.8xi8(<8 x i8> %a, <8 x i8> %b)
  %xi = sext <8 x i1> %c to <8 x i8>
  %xw = bitcast <8 x i8> %xi to <2 x i64>
  %on = call <2 x i64> @ALLONES()
  %nw = call <2 x i64> @PXOR.2xi64(<2 x i64> %xw, <2 x i64> %on)
  %ni = bitcast <2 x i64> %nw to <8 x i8>
  %r = trunc <8 x i8> %ni to <8 x i1>
  ret <8 x i1> %r
}

define <8 x i8> @PMAXUB.8xi8(<8 x i8> "reg"="xh" "tied" %n, <8 x i8> "reg"="xh" %m) "enc"="01100110 | rex(0, d, 0, m) | 00001111 | 11011110 | rr(d, m)" "asm"="pmaxub {m}, {d}" "dclass"="xh" "nodecode" {
  %r = call <8 x i8> @llvm.umax.v8i8(<8 x i8> %n, <8 x i8> %m)
  ret <8 x i8> %r
}

define <8 x i8> @PMINUB.8xi8(<8 x i8> "reg"="xh" "tied" %n, <8 x i8> "reg"="xh" %m) "enc"="01100110 | rex(0, d, 0, m) | 00001111 | 11011010 | rr(d, m)" "asm"="pminub {m}, {d}" "dclass"="xh" "nodecode" {
  %r = call <8 x i8> @llvm.umin.v8i8(<8 x i8> %n, <8 x i8> %m)
  ret <8 x i8> %r
}

declare <8 x i8> @llvm.umax.v8i8(<8 x i8>, <8 x i8>)
declare <8 x i8> @llvm.umin.v8i8(<8 x i8>, <8 x i8>)

define <8 x i1> @uge.8xi8(<8 x i8> "reg"="xh" %a, <8 x i8> "reg"="xh" %b) "recipe"="uge.8xi8.expand" {
  %r = icmp uge <8 x i8> %a, %b
  ret <8 x i1> %r
}

define <8 x i1> @uge.8xi8.expand(<8 x i8> %a, <8 x i8> %b) {
  %m = call <8 x i8> @PMAXUB.8xi8(<8 x i8> %a, <8 x i8> %b)
  %c = call <8 x i1> @PCMPEQB.8xi8(<8 x i8> %m, <8 x i8> %a)
  ret <8 x i1> %c
}

define <8 x i1> @ule.8xi8(<8 x i8> "reg"="xh" %a, <8 x i8> "reg"="xh" %b) "recipe"="ule.8xi8.expand" {
  %r = icmp ule <8 x i8> %a, %b
  ret <8 x i1> %r
}

define <8 x i1> @ule.8xi8.expand(<8 x i8> %a, <8 x i8> %b) {
  %m = call <8 x i8> @PMAXUB.8xi8(<8 x i8> %b, <8 x i8> %a)
  %c = call <8 x i1> @PCMPEQB.8xi8(<8 x i8> %m, <8 x i8> %b)
  ret <8 x i1> %c
}

define <8 x i1> @ult.8xi8(<8 x i8> "reg"="xh" %a, <8 x i8> "reg"="xh" %b) "recipe"="ult.8xi8.expand" {
  %r = icmp ult <8 x i8> %a, %b
  ret <8 x i1> %r
}

define <8 x i1> @ult.8xi8.expand(<8 x i8> %a, <8 x i8> %b) {
  %m = call <8 x i8> @PMAXUB.8xi8(<8 x i8> %a, <8 x i8> %b)
  %c = call <8 x i1> @PCMPEQB.8xi8(<8 x i8> %m, <8 x i8> %a)
  %xi = sext <8 x i1> %c to <8 x i8>
  %xw = bitcast <8 x i8> %xi to <2 x i64>
  %on = call <2 x i64> @ALLONES()
  %nw = call <2 x i64> @PXOR.2xi64(<2 x i64> %xw, <2 x i64> %on)
  %ni = bitcast <2 x i64> %nw to <8 x i8>
  %r = trunc <8 x i8> %ni to <8 x i1>
  ret <8 x i1> %r
}

define <8 x i1> @ugt.8xi8(<8 x i8> "reg"="xh" %a, <8 x i8> "reg"="xh" %b) "recipe"="ugt.8xi8.expand" {
  %r = icmp ugt <8 x i8> %a, %b
  ret <8 x i1> %r
}

define <8 x i1> @ugt.8xi8.expand(<8 x i8> %a, <8 x i8> %b) {
  %m = call <8 x i8> @PMAXUB.8xi8(<8 x i8> %b, <8 x i8> %a)
  %c = call <8 x i1> @PCMPEQB.8xi8(<8 x i8> %m, <8 x i8> %b)
  %xi = sext <8 x i1> %c to <8 x i8>
  %xw = bitcast <8 x i8> %xi to <2 x i64>
  %on = call <2 x i64> @ALLONES()
  %nw = call <2 x i64> @PXOR.2xi64(<2 x i64> %xw, <2 x i64> %on)
  %ni = bitcast <2 x i64> %nw to <8 x i8>
  %r = trunc <8 x i8> %ni to <8 x i1>
  ret <8 x i1> %r
}

define <4 x i1> @slt.4xi16(<4 x i16> "reg"="xh" %a, <4 x i16> "reg"="xh" %b) "recipe"="slt.4xi16.expand" {
  %r = icmp slt <4 x i16> %a, %b
  ret <4 x i1> %r
}

define <4 x i1> @slt.4xi16.expand(<4 x i16> %a, <4 x i16> %b) {
  %r = call <4 x i1> @PCMPGTW.4xi16(<4 x i16> %b, <4 x i16> %a)
  ret <4 x i1> %r
}

define <4 x i1> @ne.4xi16(<4 x i16> "reg"="xh" %a, <4 x i16> "reg"="xh" %b) "recipe"="ne.4xi16.expand" {
  %r = icmp ne <4 x i16> %a, %b
  ret <4 x i1> %r
}

define <4 x i1> @ne.4xi16.expand(<4 x i16> %a, <4 x i16> %b) {
  %c = call <4 x i1> @PCMPEQW.4xi16(<4 x i16> %a, <4 x i16> %b)
  %xi = sext <4 x i1> %c to <4 x i16>
  %xw = bitcast <4 x i16> %xi to <2 x i64>
  %on = call <2 x i64> @ALLONES()
  %nw = call <2 x i64> @PXOR.2xi64(<2 x i64> %xw, <2 x i64> %on)
  %ni = bitcast <2 x i64> %nw to <4 x i16>
  %r = trunc <4 x i16> %ni to <4 x i1>
  ret <4 x i1> %r
}

define <4 x i1> @sge.4xi16(<4 x i16> "reg"="xh" %a, <4 x i16> "reg"="xh" %b) "recipe"="sge.4xi16.expand" {
  %r = icmp sge <4 x i16> %a, %b
  ret <4 x i1> %r
}

define <4 x i1> @sge.4xi16.expand(<4 x i16> %a, <4 x i16> %b) {
  %c = call <4 x i1> @PCMPGTW.4xi16(<4 x i16> %b, <4 x i16> %a)
  %xi = sext <4 x i1> %c to <4 x i16>
  %xw = bitcast <4 x i16> %xi to <2 x i64>
  %on = call <2 x i64> @ALLONES()
  %nw = call <2 x i64> @PXOR.2xi64(<2 x i64> %xw, <2 x i64> %on)
  %ni = bitcast <2 x i64> %nw to <4 x i16>
  %r = trunc <4 x i16> %ni to <4 x i1>
  ret <4 x i1> %r
}

define <4 x i1> @sle.4xi16(<4 x i16> "reg"="xh" %a, <4 x i16> "reg"="xh" %b) "recipe"="sle.4xi16.expand" {
  %r = icmp sle <4 x i16> %a, %b
  ret <4 x i1> %r
}

define <4 x i1> @sle.4xi16.expand(<4 x i16> %a, <4 x i16> %b) {
  %c = call <4 x i1> @PCMPGTW.4xi16(<4 x i16> %a, <4 x i16> %b)
  %xi = sext <4 x i1> %c to <4 x i16>
  %xw = bitcast <4 x i16> %xi to <2 x i64>
  %on = call <2 x i64> @ALLONES()
  %nw = call <2 x i64> @PXOR.2xi64(<2 x i64> %xw, <2 x i64> %on)
  %ni = bitcast <2 x i64> %nw to <4 x i16>
  %r = trunc <4 x i16> %ni to <4 x i1>
  ret <4 x i1> %r
}

define <4 x i16> @PSUBUSW.4xi16(<4 x i16> "reg"="xh" "tied" %n, <4 x i16> "reg"="xh" %m) "enc"="01100110 | rex(0, d, 0, m) | 00001111 | 11011001 | rr(d, m)" "asm"="psubusw {m}, {d}" "dclass"="xh" "nodecode" {
  %r = call <4 x i16> @llvm.usub.sat.v4i16(<4 x i16> %n, <4 x i16> %m)
  ret <4 x i16> %r
}

declare <4 x i16> @llvm.usub.sat.v4i16(<4 x i16>, <4 x i16>)

define <4 x i16> @PMAXSW.4xi16(<4 x i16> "reg"="xh" "tied" %n, <4 x i16> "reg"="xh" %m) "enc"="01100110 | rex(0, d, 0, m) | 00001111 | 11101110 | rr(d, m)" "asm"="pmaxsw {m}, {d}" "dclass"="xh" "nodecode" {
  %r = call <4 x i16> @llvm.smax.v4i16(<4 x i16> %n, <4 x i16> %m)
  ret <4 x i16> %r
}

declare <4 x i16> @llvm.smax.v4i16(<4 x i16>, <4 x i16>)

define <4 x i16> @PMINSW.4xi16(<4 x i16> "reg"="xh" "tied" %n, <4 x i16> "reg"="xh" %m) "enc"="01100110 | rex(0, d, 0, m) | 00001111 | 11101010 | rr(d, m)" "asm"="pminsw {m}, {d}" "dclass"="xh" "nodecode" {
  %r = call <4 x i16> @llvm.smin.v4i16(<4 x i16> %n, <4 x i16> %m)
  ret <4 x i16> %r
}

declare <4 x i16> @llvm.smin.v4i16(<4 x i16>, <4 x i16>)

define <4 x i1> @ule.4xi16(<4 x i16> "reg"="xh" %a, <4 x i16> "reg"="xh" %b) "recipe"="ule.4xi16.expand" {
  %r = icmp ule <4 x i16> %a, %b
  ret <4 x i1> %r
}

define <4 x i1> @ule.4xi16.expand(<4 x i16> %a, <4 x i16> %b) {
  %z = call <2 x i64> @PXORzero()
  %zt = bitcast <2 x i64> %z to <4 x i16>
  %d = call <4 x i16> @PSUBUSW.4xi16(<4 x i16> %a, <4 x i16> %b)
  %c = call <4 x i1> @PCMPEQW.4xi16(<4 x i16> %d, <4 x i16> %zt)
  ret <4 x i1> %c
}

define <4 x i1> @uge.4xi16(<4 x i16> "reg"="xh" %a, <4 x i16> "reg"="xh" %b) "recipe"="uge.4xi16.expand" {
  %r = icmp uge <4 x i16> %a, %b
  ret <4 x i1> %r
}

define <4 x i1> @uge.4xi16.expand(<4 x i16> %a, <4 x i16> %b) {
  %z = call <2 x i64> @PXORzero()
  %zt = bitcast <2 x i64> %z to <4 x i16>
  %d = call <4 x i16> @PSUBUSW.4xi16(<4 x i16> %b, <4 x i16> %a)
  %c = call <4 x i1> @PCMPEQW.4xi16(<4 x i16> %d, <4 x i16> %zt)
  ret <4 x i1> %c
}

define <4 x i1> @ugt.4xi16(<4 x i16> "reg"="xh" %a, <4 x i16> "reg"="xh" %b) "recipe"="ugt.4xi16.expand" {
  %r = icmp ugt <4 x i16> %a, %b
  ret <4 x i1> %r
}

define <4 x i1> @ugt.4xi16.expand(<4 x i16> %a, <4 x i16> %b) {
  %z = call <2 x i64> @PXORzero()
  %zt = bitcast <2 x i64> %z to <4 x i16>
  %d = call <4 x i16> @PSUBUSW.4xi16(<4 x i16> %a, <4 x i16> %b)
  %c = call <4 x i1> @PCMPEQW.4xi16(<4 x i16> %d, <4 x i16> %zt)
  %xi = sext <4 x i1> %c to <4 x i16>
  %xw = bitcast <4 x i16> %xi to <2 x i64>
  %on = call <2 x i64> @ALLONES()
  %nw = call <2 x i64> @PXOR.2xi64(<2 x i64> %xw, <2 x i64> %on)
  %ni = bitcast <2 x i64> %nw to <4 x i16>
  %r = trunc <4 x i16> %ni to <4 x i1>
  ret <4 x i1> %r
}

define <4 x i1> @ult.4xi16(<4 x i16> "reg"="xh" %a, <4 x i16> "reg"="xh" %b) "recipe"="ult.4xi16.expand" {
  %r = icmp ult <4 x i16> %a, %b
  ret <4 x i1> %r
}

define <4 x i1> @ult.4xi16.expand(<4 x i16> %a, <4 x i16> %b) {
  %z = call <2 x i64> @PXORzero()
  %zt = bitcast <2 x i64> %z to <4 x i16>
  %d = call <4 x i16> @PSUBUSW.4xi16(<4 x i16> %b, <4 x i16> %a)
  %c = call <4 x i1> @PCMPEQW.4xi16(<4 x i16> %d, <4 x i16> %zt)
  %xi = sext <4 x i1> %c to <4 x i16>
  %xw = bitcast <4 x i16> %xi to <2 x i64>
  %on = call <2 x i64> @ALLONES()
  %nw = call <2 x i64> @PXOR.2xi64(<2 x i64> %xw, <2 x i64> %on)
  %ni = bitcast <2 x i64> %nw to <4 x i16>
  %r = trunc <4 x i16> %ni to <4 x i1>
  ret <4 x i1> %r
}

define <2 x i1> @slt.2xi32(<2 x i32> "reg"="xh" %a, <2 x i32> "reg"="xh" %b) "recipe"="slt.2xi32.expand" {
  %r = icmp slt <2 x i32> %a, %b
  ret <2 x i1> %r
}

define <2 x i1> @slt.2xi32.expand(<2 x i32> %a, <2 x i32> %b) {
  %r = call <2 x i1> @PCMPGTD.2xi32(<2 x i32> %b, <2 x i32> %a)
  ret <2 x i1> %r
}

define <2 x i1> @ne.2xi32(<2 x i32> "reg"="xh" %a, <2 x i32> "reg"="xh" %b) "recipe"="ne.2xi32.expand" {
  %r = icmp ne <2 x i32> %a, %b
  ret <2 x i1> %r
}

define <2 x i1> @ne.2xi32.expand(<2 x i32> %a, <2 x i32> %b) {
  %c = call <2 x i1> @PCMPEQD.2xi32(<2 x i32> %a, <2 x i32> %b)
  %xi = sext <2 x i1> %c to <2 x i32>
  %xw = bitcast <2 x i32> %xi to <2 x i64>
  %on = call <2 x i64> @ALLONES()
  %nw = call <2 x i64> @PXOR.2xi64(<2 x i64> %xw, <2 x i64> %on)
  %ni = bitcast <2 x i64> %nw to <2 x i32>
  %r = trunc <2 x i32> %ni to <2 x i1>
  ret <2 x i1> %r
}

define <2 x i1> @sge.2xi32(<2 x i32> "reg"="xh" %a, <2 x i32> "reg"="xh" %b) "recipe"="sge.2xi32.expand" {
  %r = icmp sge <2 x i32> %a, %b
  ret <2 x i1> %r
}

define <2 x i1> @sge.2xi32.expand(<2 x i32> %a, <2 x i32> %b) {
  %c = call <2 x i1> @PCMPGTD.2xi32(<2 x i32> %b, <2 x i32> %a)
  %xi = sext <2 x i1> %c to <2 x i32>
  %xw = bitcast <2 x i32> %xi to <2 x i64>
  %on = call <2 x i64> @ALLONES()
  %nw = call <2 x i64> @PXOR.2xi64(<2 x i64> %xw, <2 x i64> %on)
  %ni = bitcast <2 x i64> %nw to <2 x i32>
  %r = trunc <2 x i32> %ni to <2 x i1>
  ret <2 x i1> %r
}

define <2 x i1> @sle.2xi32(<2 x i32> "reg"="xh" %a, <2 x i32> "reg"="xh" %b) "recipe"="sle.2xi32.expand" {
  %r = icmp sle <2 x i32> %a, %b
  ret <2 x i1> %r
}

define <2 x i1> @sle.2xi32.expand(<2 x i32> %a, <2 x i32> %b) {
  %c = call <2 x i1> @PCMPGTD.2xi32(<2 x i32> %a, <2 x i32> %b)
  %xi = sext <2 x i1> %c to <2 x i32>
  %xw = bitcast <2 x i32> %xi to <2 x i64>
  %on = call <2 x i64> @ALLONES()
  %nw = call <2 x i64> @PXOR.2xi64(<2 x i64> %xw, <2 x i64> %on)
  %ni = bitcast <2 x i64> %nw to <2 x i32>
  %r = trunc <2 x i32> %ni to <2 x i1>
  ret <2 x i1> %r
}

define <2 x i1> @ugt.2xi32(<2 x i32> "reg"="xh" %a, <2 x i32> "reg"="xh" %b) "recipe"="ugt.2xi32.expand" {
  %r = icmp ugt <2 x i32> %a, %b
  ret <2 x i1> %r
}

define <2 x i1> @ugt.2xi32.expand(<2 x i32> %a, <2 x i32> %b) {
  %o = call <2 x i64> @ALLONES()
  %ot = bitcast <2 x i64> %o to <2 x i32>
  %sg = call <2 x i32> @PSLLDi.2xi32(<2 x i32> %ot, i32 31)
  %fa = call <2 x i32> @PXOR.2xi32(<2 x i32> %a, <2 x i32> %sg)
  %fb = call <2 x i32> @PXOR.2xi32(<2 x i32> %b, <2 x i32> %sg)
  %c = call <2 x i1> @PCMPGTD.2xi32(<2 x i32> %fa, <2 x i32> %fb)
  ret <2 x i1> %c
}

define <2 x i1> @ult.2xi32(<2 x i32> "reg"="xh" %a, <2 x i32> "reg"="xh" %b) "recipe"="ult.2xi32.expand" {
  %r = icmp ult <2 x i32> %a, %b
  ret <2 x i1> %r
}

define <2 x i1> @ult.2xi32.expand(<2 x i32> %a, <2 x i32> %b) {
  %o = call <2 x i64> @ALLONES()
  %ot = bitcast <2 x i64> %o to <2 x i32>
  %sg = call <2 x i32> @PSLLDi.2xi32(<2 x i32> %ot, i32 31)
  %fa = call <2 x i32> @PXOR.2xi32(<2 x i32> %a, <2 x i32> %sg)
  %fb = call <2 x i32> @PXOR.2xi32(<2 x i32> %b, <2 x i32> %sg)
  %c = call <2 x i1> @PCMPGTD.2xi32(<2 x i32> %fb, <2 x i32> %fa)
  ret <2 x i1> %c
}

define <2 x i1> @ule.2xi32(<2 x i32> "reg"="xh" %a, <2 x i32> "reg"="xh" %b) "recipe"="ule.2xi32.expand" {
  %r = icmp ule <2 x i32> %a, %b
  ret <2 x i1> %r
}

define <2 x i1> @ule.2xi32.expand(<2 x i32> %a, <2 x i32> %b) {
  %o = call <2 x i64> @ALLONES()
  %ot = bitcast <2 x i64> %o to <2 x i32>
  %sg = call <2 x i32> @PSLLDi.2xi32(<2 x i32> %ot, i32 31)
  %fa = call <2 x i32> @PXOR.2xi32(<2 x i32> %a, <2 x i32> %sg)
  %fb = call <2 x i32> @PXOR.2xi32(<2 x i32> %b, <2 x i32> %sg)
  %c = call <2 x i1> @PCMPGTD.2xi32(<2 x i32> %fa, <2 x i32> %fb)
  %xi = sext <2 x i1> %c to <2 x i32>
  %xw = bitcast <2 x i32> %xi to <2 x i64>
  %on = call <2 x i64> @ALLONES()
  %nw = call <2 x i64> @PXOR.2xi64(<2 x i64> %xw, <2 x i64> %on)
  %ni = bitcast <2 x i64> %nw to <2 x i32>
  %r = trunc <2 x i32> %ni to <2 x i1>
  ret <2 x i1> %r
}

define <2 x i1> @uge.2xi32(<2 x i32> "reg"="xh" %a, <2 x i32> "reg"="xh" %b) "recipe"="uge.2xi32.expand" {
  %r = icmp uge <2 x i32> %a, %b
  ret <2 x i1> %r
}

define <2 x i1> @uge.2xi32.expand(<2 x i32> %a, <2 x i32> %b) {
  %o = call <2 x i64> @ALLONES()
  %ot = bitcast <2 x i64> %o to <2 x i32>
  %sg = call <2 x i32> @PSLLDi.2xi32(<2 x i32> %ot, i32 31)
  %fa = call <2 x i32> @PXOR.2xi32(<2 x i32> %a, <2 x i32> %sg)
  %fb = call <2 x i32> @PXOR.2xi32(<2 x i32> %b, <2 x i32> %sg)
  %c = call <2 x i1> @PCMPGTD.2xi32(<2 x i32> %fb, <2 x i32> %fa)
  %xi = sext <2 x i1> %c to <2 x i32>
  %xw = bitcast <2 x i32> %xi to <2 x i64>
  %on = call <2 x i64> @ALLONES()
  %nw = call <2 x i64> @PXOR.2xi64(<2 x i64> %xw, <2 x i64> %on)
  %ni = bitcast <2 x i64> %nw to <2 x i32>
  %r = trunc <2 x i32> %ni to <2 x i1>
  ret <2 x i1> %r
}

define <2 x i64> @PMULUDQ(<2 x i64> "reg"="xv" "tied" %n, <2 x i64> "reg"="xv" %m) "enc"="01100110 | rex(0, d, 0, m) | 00001111 | 11110100 | rr(d, m)" "asm"="pmuludq {m}, {d}" "dclass"="xv" "nomatch" "nodecode" {
  ret <2 x i64> %n
}

define <4 x i32> @PUNPCKLDQ(<4 x i32> "reg"="xv" "tied" %n, <4 x i32> "reg"="xv" %m) "enc"="01100110 | rex(0, d, 0, m) | 00001111 | 01100010 | rr(d, m)" "asm"="punpckldq {m}, {d}" "dclass"="xv" "nomatch" "nodecode" {
  ret <4 x i32> %n
}

define <4 x i32> @mul.4xi32(<4 x i32> "reg"="xv" %a, <4 x i32> "reg"="xv" %b) "recipe"="mul.4xi32.expand" {
  %r = mul <4 x i32> %a, %b
  ret <4 x i32> %r
}

define <4 x i32> @mul.4xi32.expand(<4 x i32> %a, <4 x i32> %b) {
  %aw = bitcast <4 x i32> %a to <2 x i64>
  %bw = bitcast <4 x i32> %b to <2 x i64>
  %p0 = call <2 x i64> @PMULUDQ(<2 x i64> %aw, <2 x i64> %bw)
  %a1 = call <2 x i64> @PSRLQi.2xi64(<2 x i64> %aw, i64 32)
  %b1 = call <2 x i64> @PSRLQi.2xi64(<2 x i64> %bw, i64 32)
  %p1 = call <2 x i64> @PMULUDQ(<2 x i64> %a1, <2 x i64> %b1)
  %q0 = bitcast <2 x i64> %p0 to <4 x i32>
  %q1 = bitcast <2 x i64> %p1 to <4 x i32>
  %s0 = call <4 x i32> @PSHUFD(<4 x i32> %q0, i64 8)
  %s1 = call <4 x i32> @PSHUFD(<4 x i32> %q1, i64 8)
  %u = call <4 x i32> @PUNPCKLDQ(<4 x i32> %s0, <4 x i32> %s1)
  %r = bitcast <4 x i32> %u to <4 x i32>
  ret <4 x i32> %r
}

define <2 x i32> @mul.2xi32(<2 x i32> "reg"="xh" %a, <2 x i32> "reg"="xh" %b) "recipe"="mul.2xi32.expand" {
  %r = mul <2 x i32> %a, %b
  ret <2 x i32> %r
}

define <2 x i32> @mul.2xi32.expand(<2 x i32> %a, <2 x i32> %b) {
  %aw = bitcast <2 x i32> %a to <2 x i64>
  %bw = bitcast <2 x i32> %b to <2 x i64>
  %p0 = call <2 x i64> @PMULUDQ(<2 x i64> %aw, <2 x i64> %bw)
  %a1 = call <2 x i64> @PSRLQi.2xi64(<2 x i64> %aw, i64 32)
  %b1 = call <2 x i64> @PSRLQi.2xi64(<2 x i64> %bw, i64 32)
  %p1 = call <2 x i64> @PMULUDQ(<2 x i64> %a1, <2 x i64> %b1)
  %q0 = bitcast <2 x i64> %p0 to <4 x i32>
  %q1 = bitcast <2 x i64> %p1 to <4 x i32>
  %s0 = call <4 x i32> @PSHUFD(<4 x i32> %q0, i64 8)
  %s1 = call <4 x i32> @PSHUFD(<4 x i32> %q1, i64 8)
  %u = call <4 x i32> @PUNPCKLDQ(<4 x i32> %s0, <4 x i32> %s1)
  %r = bitcast <4 x i32> %u to <2 x i32>
  ret <2 x i32> %r
}

define <4 x float> @fneg.4xf32(<4 x float> "reg"="xv" %a) "recipe"="fneg.4xf32.expand" {
  %r = fneg <4 x float> %a
  ret <4 x float> %r
}

define <4 x float> @fneg.4xf32.expand(<4 x float> %a) {
  %o = call <2 x i64> @ALLONES()
  %oi = bitcast <2 x i64> %o to <4 x i32>
  %s = call <4 x i32> @PSLLDi.4xi32(<4 x i32> %oi, i32 31)
  %sw = bitcast <4 x i32> %s to <2 x i64>
  %aw = bitcast <4 x float> %a to <2 x i64>
  %x = call <2 x i64> @PXOR.2xi64(<2 x i64> %aw, <2 x i64> %sw)
  %r = bitcast <2 x i64> %x to <4 x float>
  ret <4 x float> %r
}

define <4 x float> @fabs.4xf32(<4 x float> "reg"="xv" %a) "recipe"="fabs.4xf32.expand" {
  %r = call <4 x float> @llvm.fabs.v4f32(<4 x float> %a)
  ret <4 x float> %r
}

define <4 x float> @fabs.4xf32.expand(<4 x float> %a) {
  %o = call <2 x i64> @ALLONES()
  %oi = bitcast <2 x i64> %o to <4 x i32>
  %s = call <4 x i32> @PSRLDi.4xi32(<4 x i32> %oi, i32 1)
  %sw = bitcast <4 x i32> %s to <2 x i64>
  %aw = bitcast <4 x float> %a to <2 x i64>
  %x = call <2 x i64> @PAND.2xi64(<2 x i64> %aw, <2 x i64> %sw)
  %r = bitcast <2 x i64> %x to <4 x float>
  ret <4 x float> %r
}

declare <4 x float> @llvm.fabs.v4f32(<4 x float>)

define <2 x double> @fneg.2xf64(<2 x double> "reg"="xv" %a) "recipe"="fneg.2xf64.expand" {
  %r = fneg <2 x double> %a
  ret <2 x double> %r
}

define <2 x double> @fneg.2xf64.expand(<2 x double> %a) {
  %o = call <2 x i64> @ALLONES()
  %oi = bitcast <2 x i64> %o to <2 x i64>
  %s = call <2 x i64> @PSLLQi.2xi64(<2 x i64> %oi, i32 63)
  %sw = bitcast <2 x i64> %s to <2 x i64>
  %aw = bitcast <2 x double> %a to <2 x i64>
  %x = call <2 x i64> @PXOR.2xi64(<2 x i64> %aw, <2 x i64> %sw)
  %r = bitcast <2 x i64> %x to <2 x double>
  ret <2 x double> %r
}

define <2 x double> @fabs.2xf64(<2 x double> "reg"="xv" %a) "recipe"="fabs.2xf64.expand" {
  %r = call <2 x double> @llvm.fabs.v2f64(<2 x double> %a)
  ret <2 x double> %r
}

define <2 x double> @fabs.2xf64.expand(<2 x double> %a) {
  %o = call <2 x i64> @ALLONES()
  %oi = bitcast <2 x i64> %o to <2 x i64>
  %s = call <2 x i64> @PSRLQi.2xi64(<2 x i64> %oi, i32 1)
  %sw = bitcast <2 x i64> %s to <2 x i64>
  %aw = bitcast <2 x double> %a to <2 x i64>
  %x = call <2 x i64> @PAND.2xi64(<2 x i64> %aw, <2 x i64> %sw)
  %r = bitcast <2 x i64> %x to <2 x double>
  ret <2 x double> %r
}

declare <2 x double> @llvm.fabs.v2f64(<2 x double>)

define <2 x float> @fneg.2xf32(<2 x float> "reg"="xh" %a) "recipe"="fneg.2xf32.expand" {
  %r = fneg <2 x float> %a
  ret <2 x float> %r
}

define <2 x float> @fneg.2xf32.expand(<2 x float> %a) {
  %o = call <2 x i64> @ALLONES()
  %oi = bitcast <2 x i64> %o to <4 x i32>
  %s = call <4 x i32> @PSLLDi.4xi32(<4 x i32> %oi, i32 31)
  %sw = bitcast <4 x i32> %s to <2 x i64>
  %aw = bitcast <2 x float> %a to <2 x i64>
  %x = call <2 x i64> @PXOR.2xi64(<2 x i64> %aw, <2 x i64> %sw)
  %r = bitcast <2 x i64> %x to <2 x float>
  ret <2 x float> %r
}

define <2 x float> @fabs.2xf32(<2 x float> "reg"="xh" %a) "recipe"="fabs.2xf32.expand" {
  %r = call <2 x float> @llvm.fabs.v2f32(<2 x float> %a)
  ret <2 x float> %r
}

define <2 x float> @fabs.2xf32.expand(<2 x float> %a) {
  %o = call <2 x i64> @ALLONES()
  %oi = bitcast <2 x i64> %o to <4 x i32>
  %s = call <4 x i32> @PSRLDi.4xi32(<4 x i32> %oi, i32 1)
  %sw = bitcast <4 x i32> %s to <2 x i64>
  %aw = bitcast <2 x float> %a to <2 x i64>
  %x = call <2 x i64> @PAND.2xi64(<2 x i64> %aw, <2 x i64> %sw)
  %r = bitcast <2 x i64> %x to <2 x float>
  ret <2 x float> %r
}

declare <2 x float> @llvm.fabs.v2f32(<2 x float>)

define <32 x i8> @VLD.32xi8(ptr "reg"="r64" %n, i64 "imm" %off) "enc"="{ vexm(d, n, 4, 1, 0, 0, 1, 0) | 00010000 | rm(d, n, 4, 1, off, 0, 1) ; evexm(d, n, 4, 1, 0, 0, 1, 0, 0, 0, 0) | 00010000 | rm(d, n, 4, 1, off, 0, 32) }" "asm"="vmovups {off}({n}), {d}" "dclass"="yv" "feature"="avx2" "nodecode" {
  %a = getelementptr i8, ptr %n, i64 %off
  %r = load <32 x i8>, ptr %a
  ret <32 x i8> %r
}

define void @VST.32xi8(<32 x i8> "reg"="yv" %t, ptr "reg"="r64" %n, i64 "imm" %off) "enc"="{ vexm(t, n, 4, 1, 0, 0, 1, 0) | 00010001 | rm(t, n, 4, 1, off, 0, 1) ; evexm(t, n, 4, 1, 0, 0, 1, 0, 0, 0, 0) | 00010001 | rm(t, n, 4, 1, off, 0, 32) }" "asm"="vmovups {t}, {off}({n})" "feature"="avx2" "nodecode" {
  %a = getelementptr i8, ptr %n, i64 %off
  store <32 x i8> %t, ptr %a
  ret void
}

define <16 x i16> @VLD.16xi16(ptr "reg"="r64" %n, i64 "imm" %off) "enc"="{ vexm(d, n, 4, 1, 0, 0, 1, 0) | 00010000 | rm(d, n, 4, 1, off, 0, 1) ; evexm(d, n, 4, 1, 0, 0, 1, 0, 0, 0, 0) | 00010000 | rm(d, n, 4, 1, off, 0, 32) }" "asm"="vmovups {off}({n}), {d}" "dclass"="yv" "feature"="avx2" "nodecode" {
  %a = getelementptr i8, ptr %n, i64 %off
  %r = load <16 x i16>, ptr %a
  ret <16 x i16> %r
}

define void @VST.16xi16(<16 x i16> "reg"="yv" %t, ptr "reg"="r64" %n, i64 "imm" %off) "enc"="{ vexm(t, n, 4, 1, 0, 0, 1, 0) | 00010001 | rm(t, n, 4, 1, off, 0, 1) ; evexm(t, n, 4, 1, 0, 0, 1, 0, 0, 0, 0) | 00010001 | rm(t, n, 4, 1, off, 0, 32) }" "asm"="vmovups {t}, {off}({n})" "feature"="avx2" "nodecode" {
  %a = getelementptr i8, ptr %n, i64 %off
  store <16 x i16> %t, ptr %a
  ret void
}

define <8 x i32> @VLD.8xi32(ptr "reg"="r64" %n, i64 "imm" %off) "enc"="{ vexm(d, n, 4, 1, 0, 0, 1, 0) | 00010000 | rm(d, n, 4, 1, off, 0, 1) ; evexm(d, n, 4, 1, 0, 0, 1, 0, 0, 0, 0) | 00010000 | rm(d, n, 4, 1, off, 0, 32) }" "asm"="vmovups {off}({n}), {d}" "dclass"="yv" "feature"="avx2" "nodecode" {
  %a = getelementptr i8, ptr %n, i64 %off
  %r = load <8 x i32>, ptr %a
  ret <8 x i32> %r
}

define void @VST.8xi32(<8 x i32> "reg"="yv" %t, ptr "reg"="r64" %n, i64 "imm" %off) "enc"="{ vexm(t, n, 4, 1, 0, 0, 1, 0) | 00010001 | rm(t, n, 4, 1, off, 0, 1) ; evexm(t, n, 4, 1, 0, 0, 1, 0, 0, 0, 0) | 00010001 | rm(t, n, 4, 1, off, 0, 32) }" "asm"="vmovups {t}, {off}({n})" "feature"="avx2" "nodecode" {
  %a = getelementptr i8, ptr %n, i64 %off
  store <8 x i32> %t, ptr %a
  ret void
}

define <4 x i64> @VLD.4xi64(ptr "reg"="r64" %n, i64 "imm" %off) "enc"="{ vexm(d, n, 4, 1, 0, 0, 1, 0) | 00010000 | rm(d, n, 4, 1, off, 0, 1) ; evexm(d, n, 4, 1, 0, 0, 1, 0, 0, 0, 0) | 00010000 | rm(d, n, 4, 1, off, 0, 32) }" "asm"="vmovups {off}({n}), {d}" "dclass"="yv" "role"="load.yv" "feature"="avx2" "nodecode" {
  %a = getelementptr i8, ptr %n, i64 %off
  %r = load <4 x i64>, ptr %a
  ret <4 x i64> %r
}

define void @VST.4xi64(<4 x i64> "reg"="yv" %t, ptr "reg"="r64" %n, i64 "imm" %off) "enc"="{ vexm(t, n, 4, 1, 0, 0, 1, 0) | 00010001 | rm(t, n, 4, 1, off, 0, 1) ; evexm(t, n, 4, 1, 0, 0, 1, 0, 0, 0, 0) | 00010001 | rm(t, n, 4, 1, off, 0, 32) }" "asm"="vmovups {t}, {off}({n})" "role"="store.yv" "feature"="avx2" "nodecode" {
  %a = getelementptr i8, ptr %n, i64 %off
  store <4 x i64> %t, ptr %a
  ret void
}

define <8 x float> @VLD.8xf32(ptr "reg"="r64" %n, i64 "imm" %off) "enc"="{ vexm(d, n, 4, 1, 0, 0, 1, 0) | 00010000 | rm(d, n, 4, 1, off, 0, 1) ; evexm(d, n, 4, 1, 0, 0, 1, 0, 0, 0, 0) | 00010000 | rm(d, n, 4, 1, off, 0, 32) }" "asm"="vmovups {off}({n}), {d}" "dclass"="yv" "feature"="avx2" "nodecode" {
  %a = getelementptr i8, ptr %n, i64 %off
  %r = load <8 x float>, ptr %a
  ret <8 x float> %r
}

define void @VST.8xf32(<8 x float> "reg"="yv" %t, ptr "reg"="r64" %n, i64 "imm" %off) "enc"="{ vexm(t, n, 4, 1, 0, 0, 1, 0) | 00010001 | rm(t, n, 4, 1, off, 0, 1) ; evexm(t, n, 4, 1, 0, 0, 1, 0, 0, 0, 0) | 00010001 | rm(t, n, 4, 1, off, 0, 32) }" "asm"="vmovups {t}, {off}({n})" "feature"="avx2" "nodecode" {
  %a = getelementptr i8, ptr %n, i64 %off
  store <8 x float> %t, ptr %a
  ret void
}

define <4 x double> @VLD.4xf64(ptr "reg"="r64" %n, i64 "imm" %off) "enc"="{ vexm(d, n, 4, 1, 0, 0, 1, 0) | 00010000 | rm(d, n, 4, 1, off, 0, 1) ; evexm(d, n, 4, 1, 0, 0, 1, 0, 0, 0, 0) | 00010000 | rm(d, n, 4, 1, off, 0, 32) }" "asm"="vmovups {off}({n}), {d}" "dclass"="yv" "feature"="avx2" "nodecode" {
  %a = getelementptr i8, ptr %n, i64 %off
  %r = load <4 x double>, ptr %a
  ret <4 x double> %r
}

define void @VST.4xf64(<4 x double> "reg"="yv" %t, ptr "reg"="r64" %n, i64 "imm" %off) "enc"="{ vexm(t, n, 4, 1, 0, 0, 1, 0) | 00010001 | rm(t, n, 4, 1, off, 0, 1) ; evexm(t, n, 4, 1, 0, 0, 1, 0, 0, 0, 0) | 00010001 | rm(t, n, 4, 1, off, 0, 32) }" "asm"="vmovups {t}, {off}({n})" "feature"="avx2" "nodecode" {
  %a = getelementptr i8, ptr %n, i64 %off
  store <4 x double> %t, ptr %a
  ret void
}

define <4 x i64> @VMOVAPSyv(<4 x i64> "reg"="yv" %n) "enc"="{ if(n<8) vex(d, 0, n, 1, 0, 0, 1, 0) | 00101000 | 11 reg:3=d&7 rm:3=n&7 ; if(d<8) vex(n, 0, d, 1, 0, 0, 1, 0) | 00101001 | 11 reg:3=n&7 rm:3=d&7 ; vex(d, 0, n, 1, 0, 0, 1, 0) | 00101000 | 11 reg:3=d&7 rm:3=n&7 ; vex(n, 0, d, 1, 0, 0, 1, 0) | 00101001 | 11 reg:3=n&7 rm:3=d&7 ; evexr(d, n, 1, 0, 0, 1, 0, 0, 0, 0) | 00101000 | 11 reg:3=d&7 rm:3=n&7 ; evexr(n, d, 1, 0, 0, 1, 0, 0, 0, 0) | 00101001 | 11 reg:3=n&7 rm:3=d&7 }" "asm"="vmovaps {n}, {d}" "copy"="yv" "dclass"="yv" "nodecode" {
  ret <4 x i64> %n
}

define <4 x i64> @VPXORzero() "enc"="{ if(d<8) vex(d, 0, d, 1, 0, d, 1, 1) | 11101111 | 11 reg:3=d&7 rm:3=d&7 ; if(d<8) vex(d, 0, d, 1, 0, d, 1, 1) | 11101111 | 11 reg:3=d&7 rm:3=d&7 ; vex(d, 0, d, 1, 0, d, 1, 1) | 11101111 | 11 reg:3=d&7 rm:3=d&7 ; vex(d, 0, d, 1, 0, d, 1, 1) | 11101111 | 11 reg:3=d&7 rm:3=d&7 }" "asm"="vpxor {d}, {d}, {d}" "dclass"="yv" "role"="movi.zero.yv" "nomatch" "nodecode" {
  ret <4 x i64> zeroinitializer
}

define <4 x i64> @VINSERTI128hi(<4 x i64> "reg"="yv" "tied" %n, <2 x i64> "reg"="xv" %h) "enc"="vex(d, 0, h, 3, 0, n, 1, 1) | 00111000 | 11 reg:3=d&7 rm:3=h&7 | 00000001" "asm"="vinserti128 $1, {h}, {d}, {d}" "dclass"="yv" "role"="vec.join.q" "nomatch" "nodecode" {
  ret <4 x i64> %n
}

define <32 x i8> @VPADDB.32xi8(<32 x i8> "reg"="yv" %n, <32 x i8> "reg"="yv" %m) "enc"="{ if(m<8) vex(d, 0, m, 1, 0, n, 1, 1) | 11111100 | 11 reg:3=d&7 rm:3=m&7 ; if(n<8) vex(d, 0, n, 1, 0, m, 1, 1) | 11111100 | 11 reg:3=d&7 rm:3=n&7 ; vex(d, 0, m, 1, 0, n, 1, 1) | 11111100 | 11 reg:3=d&7 rm:3=m&7 ; evexr(d, m, 1, 0, n, 1, 1, 0, 0, 0) | 11111100 | 11 reg:3=d&7 rm:3=m&7 ; vex(d, 0, n, 1, 0, m, 1, 1) | 11111100 | 11 reg:3=d&7 rm:3=n&7 }" "asm"="vpaddb {m}, {n}, {d}" "dclass"="yv" "feature"="avx2" "nodecode" {
  %r = add <32 x i8> %n, %m
  ret <32 x i8> %r
}

define <32 x i8> @VPSUBB.32xi8(<32 x i8> "reg"="yv" %n, <32 x i8> "reg"="yv" %m) "enc"="{ vex(d, 0, m, 1, 0, n, 1, 1) | 11111000 | 11 reg:3=d&7 rm:3=m&7 ; evexr(d, m, 1, 0, n, 1, 1, 0, 0, 0) | 11111000 | 11 reg:3=d&7 rm:3=m&7 }" "asm"="vpsubb {m}, {n}, {d}" "dclass"="yv" "feature"="avx2" "nodecode" {
  %r = sub <32 x i8> %n, %m
  ret <32 x i8> %r
}

define <32 x i8> @VPAND.32xi8(<32 x i8> "reg"="yv" %n, <32 x i8> "reg"="yv" %m) "enc"="{ if(m<8) vex(d, 0, m, 1, 0, n, 1, 1) | 11011011 | 11 reg:3=d&7 rm:3=m&7 ; if(n<8) vex(d, 0, n, 1, 0, m, 1, 1) | 11011011 | 11 reg:3=d&7 rm:3=n&7 ; vex(d, 0, m, 1, 0, n, 1, 1) | 11011011 | 11 reg:3=d&7 rm:3=m&7 ; vex(d, 0, n, 1, 0, m, 1, 1) | 11011011 | 11 reg:3=d&7 rm:3=n&7 }" "asm"="vpand {m}, {n}, {d}" "dclass"="yv" "feature"="avx2" "nodecode" {
  %r = and <32 x i8> %n, %m
  ret <32 x i8> %r
}

define <32 x i8> @VPOR.32xi8(<32 x i8> "reg"="yv" %n, <32 x i8> "reg"="yv" %m) "enc"="{ if(m<8) vex(d, 0, m, 1, 0, n, 1, 1) | 11101011 | 11 reg:3=d&7 rm:3=m&7 ; if(n<8) vex(d, 0, n, 1, 0, m, 1, 1) | 11101011 | 11 reg:3=d&7 rm:3=n&7 ; vex(d, 0, m, 1, 0, n, 1, 1) | 11101011 | 11 reg:3=d&7 rm:3=m&7 ; vex(d, 0, n, 1, 0, m, 1, 1) | 11101011 | 11 reg:3=d&7 rm:3=n&7 }" "asm"="vpor {m}, {n}, {d}" "dclass"="yv" "feature"="avx2" "nodecode" {
  %r = or <32 x i8> %n, %m
  ret <32 x i8> %r
}

define <32 x i8> @VPXOR.32xi8(<32 x i8> "reg"="yv" %n, <32 x i8> "reg"="yv" %m) "enc"="{ if(m<8) vex(d, 0, m, 1, 0, n, 1, 1) | 11101111 | 11 reg:3=d&7 rm:3=m&7 ; if(n<8) vex(d, 0, n, 1, 0, m, 1, 1) | 11101111 | 11 reg:3=d&7 rm:3=n&7 ; vex(d, 0, m, 1, 0, n, 1, 1) | 11101111 | 11 reg:3=d&7 rm:3=m&7 ; vex(d, 0, n, 1, 0, m, 1, 1) | 11101111 | 11 reg:3=d&7 rm:3=n&7 }" "asm"="vpxor {m}, {n}, {d}" "dclass"="yv" "feature"="avx2" "nodecode" {
  %r = xor <32 x i8> %n, %m
  ret <32 x i8> %r
}

; Masks in 256-bit registers (AVX2), as the comparisons of 256-bit
; vectors make them: the same instructions as on their lanes.
define <32 x i1> @VPAND.ym32(<32 x i1> "reg"="yv" %n, <32 x i1> "reg"="yv" %m) "enc"="{ if(m<8) vex(d, 0, m, 1, 0, n, 1, 1) | 11011011 | 11 reg:3=d&7 rm:3=m&7 ; if(n<8) vex(d, 0, n, 1, 0, m, 1, 1) | 11011011 | 11 reg:3=d&7 rm:3=n&7 ; vex(d, 0, m, 1, 0, n, 1, 1) | 11011011 | 11 reg:3=d&7 rm:3=m&7 ; vex(d, 0, n, 1, 0, m, 1, 1) | 11011011 | 11 reg:3=d&7 rm:3=n&7 }" "asm"="vpand {m}, {n}, {d}" "dclass"="yv" "feature"="avx2" "nodecode" {
  %r = and <32 x i1> %n, %m
  ret <32 x i1> %r
}

define <32 x i1> @VPOR.ym32(<32 x i1> "reg"="yv" %n, <32 x i1> "reg"="yv" %m) "enc"="{ if(m<8) vex(d, 0, m, 1, 0, n, 1, 1) | 11101011 | 11 reg:3=d&7 rm:3=m&7 ; if(n<8) vex(d, 0, n, 1, 0, m, 1, 1) | 11101011 | 11 reg:3=d&7 rm:3=n&7 ; vex(d, 0, m, 1, 0, n, 1, 1) | 11101011 | 11 reg:3=d&7 rm:3=m&7 ; vex(d, 0, n, 1, 0, m, 1, 1) | 11101011 | 11 reg:3=d&7 rm:3=n&7 }" "asm"="vpor {m}, {n}, {d}" "dclass"="yv" "feature"="avx2" "nodecode" {
  %r = or <32 x i1> %n, %m
  ret <32 x i1> %r
}

define <32 x i1> @VPXOR.ym32(<32 x i1> "reg"="yv" %n, <32 x i1> "reg"="yv" %m) "enc"="{ if(m<8) vex(d, 0, m, 1, 0, n, 1, 1) | 11101111 | 11 reg:3=d&7 rm:3=m&7 ; if(n<8) vex(d, 0, n, 1, 0, m, 1, 1) | 11101111 | 11 reg:3=d&7 rm:3=n&7 ; vex(d, 0, m, 1, 0, n, 1, 1) | 11101111 | 11 reg:3=d&7 rm:3=m&7 ; vex(d, 0, n, 1, 0, m, 1, 1) | 11101111 | 11 reg:3=d&7 rm:3=n&7 }" "asm"="vpxor {m}, {n}, {d}" "dclass"="yv" "feature"="avx2" "nodecode" {
  %r = xor <32 x i1> %n, %m
  ret <32 x i1> %r
}

define <16 x i1> @VPAND.ym16(<16 x i1> "reg"="yv" %n, <16 x i1> "reg"="yv" %m) "enc"="{ if(m<8) vex(d, 0, m, 1, 0, n, 1, 1) | 11011011 | 11 reg:3=d&7 rm:3=m&7 ; if(n<8) vex(d, 0, n, 1, 0, m, 1, 1) | 11011011 | 11 reg:3=d&7 rm:3=n&7 ; vex(d, 0, m, 1, 0, n, 1, 1) | 11011011 | 11 reg:3=d&7 rm:3=m&7 ; vex(d, 0, n, 1, 0, m, 1, 1) | 11011011 | 11 reg:3=d&7 rm:3=n&7 }" "asm"="vpand {m}, {n}, {d}" "dclass"="yv" "feature"="avx2" "nodecode" {
  %r = and <16 x i1> %n, %m
  ret <16 x i1> %r
}

define <16 x i1> @VPOR.ym16(<16 x i1> "reg"="yv" %n, <16 x i1> "reg"="yv" %m) "enc"="{ if(m<8) vex(d, 0, m, 1, 0, n, 1, 1) | 11101011 | 11 reg:3=d&7 rm:3=m&7 ; if(n<8) vex(d, 0, n, 1, 0, m, 1, 1) | 11101011 | 11 reg:3=d&7 rm:3=n&7 ; vex(d, 0, m, 1, 0, n, 1, 1) | 11101011 | 11 reg:3=d&7 rm:3=m&7 ; vex(d, 0, n, 1, 0, m, 1, 1) | 11101011 | 11 reg:3=d&7 rm:3=n&7 }" "asm"="vpor {m}, {n}, {d}" "dclass"="yv" "feature"="avx2" "nodecode" {
  %r = or <16 x i1> %n, %m
  ret <16 x i1> %r
}

define <16 x i1> @VPXOR.ym16(<16 x i1> "reg"="yv" %n, <16 x i1> "reg"="yv" %m) "enc"="{ if(m<8) vex(d, 0, m, 1, 0, n, 1, 1) | 11101111 | 11 reg:3=d&7 rm:3=m&7 ; if(n<8) vex(d, 0, n, 1, 0, m, 1, 1) | 11101111 | 11 reg:3=d&7 rm:3=n&7 ; vex(d, 0, m, 1, 0, n, 1, 1) | 11101111 | 11 reg:3=d&7 rm:3=m&7 ; vex(d, 0, n, 1, 0, m, 1, 1) | 11101111 | 11 reg:3=d&7 rm:3=n&7 }" "asm"="vpxor {m}, {n}, {d}" "dclass"="yv" "feature"="avx2" "nodecode" {
  %r = xor <16 x i1> %n, %m
  ret <16 x i1> %r
}

define <8 x i1> @VPAND.ym8(<8 x i1> "reg"="yv" %n, <8 x i1> "reg"="yv" %m) "enc"="{ if(m<8) vex(d, 0, m, 1, 0, n, 1, 1) | 11011011 | 11 reg:3=d&7 rm:3=m&7 ; if(n<8) vex(d, 0, n, 1, 0, m, 1, 1) | 11011011 | 11 reg:3=d&7 rm:3=n&7 ; vex(d, 0, m, 1, 0, n, 1, 1) | 11011011 | 11 reg:3=d&7 rm:3=m&7 ; vex(d, 0, n, 1, 0, m, 1, 1) | 11011011 | 11 reg:3=d&7 rm:3=n&7 }" "asm"="vpand {m}, {n}, {d}" "dclass"="yv" "feature"="avx2" "nodecode" {
  %r = and <8 x i1> %n, %m
  ret <8 x i1> %r
}

define <8 x i1> @VPOR.ym8(<8 x i1> "reg"="yv" %n, <8 x i1> "reg"="yv" %m) "enc"="{ if(m<8) vex(d, 0, m, 1, 0, n, 1, 1) | 11101011 | 11 reg:3=d&7 rm:3=m&7 ; if(n<8) vex(d, 0, n, 1, 0, m, 1, 1) | 11101011 | 11 reg:3=d&7 rm:3=n&7 ; vex(d, 0, m, 1, 0, n, 1, 1) | 11101011 | 11 reg:3=d&7 rm:3=m&7 ; vex(d, 0, n, 1, 0, m, 1, 1) | 11101011 | 11 reg:3=d&7 rm:3=n&7 }" "asm"="vpor {m}, {n}, {d}" "dclass"="yv" "feature"="avx2" "nodecode" {
  %r = or <8 x i1> %n, %m
  ret <8 x i1> %r
}

define <8 x i1> @VPXOR.ym8(<8 x i1> "reg"="yv" %n, <8 x i1> "reg"="yv" %m) "enc"="{ if(m<8) vex(d, 0, m, 1, 0, n, 1, 1) | 11101111 | 11 reg:3=d&7 rm:3=m&7 ; if(n<8) vex(d, 0, n, 1, 0, m, 1, 1) | 11101111 | 11 reg:3=d&7 rm:3=n&7 ; vex(d, 0, m, 1, 0, n, 1, 1) | 11101111 | 11 reg:3=d&7 rm:3=m&7 ; vex(d, 0, n, 1, 0, m, 1, 1) | 11101111 | 11 reg:3=d&7 rm:3=n&7 }" "asm"="vpxor {m}, {n}, {d}" "dclass"="yv" "feature"="avx2" "nodecode" {
  %r = xor <8 x i1> %n, %m
  ret <8 x i1> %r
}

define <4 x i1> @VPAND.ym4(<4 x i1> "reg"="yv" %n, <4 x i1> "reg"="yv" %m) "enc"="{ if(m<8) vex(d, 0, m, 1, 0, n, 1, 1) | 11011011 | 11 reg:3=d&7 rm:3=m&7 ; if(n<8) vex(d, 0, n, 1, 0, m, 1, 1) | 11011011 | 11 reg:3=d&7 rm:3=n&7 ; vex(d, 0, m, 1, 0, n, 1, 1) | 11011011 | 11 reg:3=d&7 rm:3=m&7 ; vex(d, 0, n, 1, 0, m, 1, 1) | 11011011 | 11 reg:3=d&7 rm:3=n&7 }" "asm"="vpand {m}, {n}, {d}" "dclass"="yv" "feature"="avx2" "nodecode" {
  %r = and <4 x i1> %n, %m
  ret <4 x i1> %r
}

define <4 x i1> @VPOR.ym4(<4 x i1> "reg"="yv" %n, <4 x i1> "reg"="yv" %m) "enc"="{ if(m<8) vex(d, 0, m, 1, 0, n, 1, 1) | 11101011 | 11 reg:3=d&7 rm:3=m&7 ; if(n<8) vex(d, 0, n, 1, 0, m, 1, 1) | 11101011 | 11 reg:3=d&7 rm:3=n&7 ; vex(d, 0, m, 1, 0, n, 1, 1) | 11101011 | 11 reg:3=d&7 rm:3=m&7 ; vex(d, 0, n, 1, 0, m, 1, 1) | 11101011 | 11 reg:3=d&7 rm:3=n&7 }" "asm"="vpor {m}, {n}, {d}" "dclass"="yv" "feature"="avx2" "nodecode" {
  %r = or <4 x i1> %n, %m
  ret <4 x i1> %r
}

define <4 x i1> @VPXOR.ym4(<4 x i1> "reg"="yv" %n, <4 x i1> "reg"="yv" %m) "enc"="{ if(m<8) vex(d, 0, m, 1, 0, n, 1, 1) | 11101111 | 11 reg:3=d&7 rm:3=m&7 ; if(n<8) vex(d, 0, n, 1, 0, m, 1, 1) | 11101111 | 11 reg:3=d&7 rm:3=n&7 ; vex(d, 0, m, 1, 0, n, 1, 1) | 11101111 | 11 reg:3=d&7 rm:3=m&7 ; vex(d, 0, n, 1, 0, m, 1, 1) | 11101111 | 11 reg:3=d&7 rm:3=n&7 }" "asm"="vpxor {m}, {n}, {d}" "dclass"="yv" "feature"="avx2" "nodecode" {
  %r = xor <4 x i1> %n, %m
  ret <4 x i1> %r
}

define <32 x i1> @VPCMPEQB.32xi8(<32 x i8> "reg"="yv" %n, <32 x i8> "reg"="yv" %m) "enc"="{ if(m<8) vex(d, 0, m, 1, 0, n, 1, 1) | 01110100 | 11 reg:3=d&7 rm:3=m&7 ; if(n<8) vex(d, 0, n, 1, 0, m, 1, 1) | 01110100 | 11 reg:3=d&7 rm:3=n&7 ; vex(d, 0, m, 1, 0, n, 1, 1) | 01110100 | 11 reg:3=d&7 rm:3=m&7 ; vex(d, 0, n, 1, 0, m, 1, 1) | 01110100 | 11 reg:3=d&7 rm:3=n&7 }" "asm"="vpcmpeqb {m}, {n}, {d}" "dclass"="yv" "feature"="avx2" "nodecode" {
  %r = icmp eq <32 x i8> %n, %m
  ret <32 x i1> %r
}

define <32 x i1> @VPCMPGTB.32xi8(<32 x i8> "reg"="yv" %n, <32 x i8> "reg"="yv" %m) "enc"="vex(d, 0, m, 1, 0, n, 1, 1) | 01100100 | 11 reg:3=d&7 rm:3=m&7" "asm"="vpcmpgtb {m}, {n}, {d}" "dclass"="yv" "feature"="avx2" "nodecode" {
  %r = icmp sgt <32 x i8> %n, %m
  ret <32 x i1> %r
}

define <32 x i8> @VPMINSB.32xi8(<32 x i8> "reg"="yv" %n, <32 x i8> "reg"="yv" %m) "enc"="{ vex(d, 0, m, 2, 0, n, 1, 1) | 00111000 | 11 reg:3=d&7 rm:3=m&7 ; evexr(d, m, 2, 0, n, 1, 1, 0, 0, 0) | 00111000 | 11 reg:3=d&7 rm:3=m&7 }" "asm"="vpminsb {m}, {n}, {d}" "dclass"="yv" "feature"="avx2" "nodecode" {
  %r = call <32 x i8> @llvm.smin.v32i8(<32 x i8> %n, <32 x i8> %m)
  ret <32 x i8> %r
}

declare <32 x i8> @llvm.smin.v32i8(<32 x i8>, <32 x i8>)

define <32 x i8> @VPMAXSB.32xi8(<32 x i8> "reg"="yv" %n, <32 x i8> "reg"="yv" %m) "enc"="{ vex(d, 0, m, 2, 0, n, 1, 1) | 00111100 | 11 reg:3=d&7 rm:3=m&7 ; evexr(d, m, 2, 0, n, 1, 1, 0, 0, 0) | 00111100 | 11 reg:3=d&7 rm:3=m&7 }" "asm"="vpmaxsb {m}, {n}, {d}" "dclass"="yv" "feature"="avx2" "nodecode" {
  %r = call <32 x i8> @llvm.smax.v32i8(<32 x i8> %n, <32 x i8> %m)
  ret <32 x i8> %r
}

declare <32 x i8> @llvm.smax.v32i8(<32 x i8>, <32 x i8>)

define <32 x i8> @VPMINUB.32xi8(<32 x i8> "reg"="yv" %n, <32 x i8> "reg"="yv" %m) "enc"="{ if(m<8) vex(d, 0, m, 1, 0, n, 1, 1) | 11011010 | 11 reg:3=d&7 rm:3=m&7 ; if(n<8) vex(d, 0, n, 1, 0, m, 1, 1) | 11011010 | 11 reg:3=d&7 rm:3=n&7 ; vex(d, 0, m, 1, 0, n, 1, 1) | 11011010 | 11 reg:3=d&7 rm:3=m&7 ; evexr(d, m, 1, 0, n, 1, 1, 0, 0, 0) | 11011010 | 11 reg:3=d&7 rm:3=m&7 ; vex(d, 0, n, 1, 0, m, 1, 1) | 11011010 | 11 reg:3=d&7 rm:3=n&7 }" "asm"="vpminub {m}, {n}, {d}" "dclass"="yv" "feature"="avx2" "nodecode" {
  %r = call <32 x i8> @llvm.umin.v32i8(<32 x i8> %n, <32 x i8> %m)
  ret <32 x i8> %r
}

declare <32 x i8> @llvm.umin.v32i8(<32 x i8>, <32 x i8>)

define <32 x i8> @VPMAXUB.32xi8(<32 x i8> "reg"="yv" %n, <32 x i8> "reg"="yv" %m) "enc"="{ if(m<8) vex(d, 0, m, 1, 0, n, 1, 1) | 11011110 | 11 reg:3=d&7 rm:3=m&7 ; if(n<8) vex(d, 0, n, 1, 0, m, 1, 1) | 11011110 | 11 reg:3=d&7 rm:3=n&7 ; vex(d, 0, m, 1, 0, n, 1, 1) | 11011110 | 11 reg:3=d&7 rm:3=m&7 ; evexr(d, m, 1, 0, n, 1, 1, 0, 0, 0) | 11011110 | 11 reg:3=d&7 rm:3=m&7 ; vex(d, 0, n, 1, 0, m, 1, 1) | 11011110 | 11 reg:3=d&7 rm:3=n&7 }" "asm"="vpmaxub {m}, {n}, {d}" "dclass"="yv" "feature"="avx2" "nodecode" {
  %r = call <32 x i8> @llvm.umax.v32i8(<32 x i8> %n, <32 x i8> %m)
  ret <32 x i8> %r
}

declare <32 x i8> @llvm.umax.v32i8(<32 x i8>, <32 x i8>)

define <32 x i8> @VPBLENDVB.32xi8(<32 x i1> "reg"="yv" %c, <32 x i8> "reg"="yv" %a, <32 x i8> "reg"="yv" %b) "enc"="vex(d, 0, a, 3, 0, b, 1, 1) | 01001100 | 11 reg:3=d&7 rm:3=a&7 | is4:4=c 0000" "asm"="vpblendvb {c}, {a}, {b}, {d}" "dclass"="yv" "feature"="avx2" "nodecode" {
  %r = select <32 x i1> %c, <32 x i8> %a, <32 x i8> %b
  ret <32 x i8> %r
}

define <16 x i16> @VPADDW.16xi16(<16 x i16> "reg"="yv" %n, <16 x i16> "reg"="yv" %m) "enc"="{ if(m<8) vex(d, 0, m, 1, 0, n, 1, 1) | 11111101 | 11 reg:3=d&7 rm:3=m&7 ; if(n<8) vex(d, 0, n, 1, 0, m, 1, 1) | 11111101 | 11 reg:3=d&7 rm:3=n&7 ; vex(d, 0, m, 1, 0, n, 1, 1) | 11111101 | 11 reg:3=d&7 rm:3=m&7 ; evexr(d, m, 1, 0, n, 1, 1, 0, 0, 0) | 11111101 | 11 reg:3=d&7 rm:3=m&7 ; vex(d, 0, n, 1, 0, m, 1, 1) | 11111101 | 11 reg:3=d&7 rm:3=n&7 }" "asm"="vpaddw {m}, {n}, {d}" "dclass"="yv" "feature"="avx2" "nodecode" {
  %r = add <16 x i16> %n, %m
  ret <16 x i16> %r
}

define <16 x i16> @VPSUBW.16xi16(<16 x i16> "reg"="yv" %n, <16 x i16> "reg"="yv" %m) "enc"="{ vex(d, 0, m, 1, 0, n, 1, 1) | 11111001 | 11 reg:3=d&7 rm:3=m&7 ; evexr(d, m, 1, 0, n, 1, 1, 0, 0, 0) | 11111001 | 11 reg:3=d&7 rm:3=m&7 }" "asm"="vpsubw {m}, {n}, {d}" "dclass"="yv" "feature"="avx2" "nodecode" {
  %r = sub <16 x i16> %n, %m
  ret <16 x i16> %r
}

define <16 x i16> @VPMULLW.16xi16(<16 x i16> "reg"="yv" %n, <16 x i16> "reg"="yv" %m) "enc"="{ if(m<8) vex(d, 0, m, 1, 0, n, 1, 1) | 11010101 | 11 reg:3=d&7 rm:3=m&7 ; if(n<8) vex(d, 0, n, 1, 0, m, 1, 1) | 11010101 | 11 reg:3=d&7 rm:3=n&7 ; vex(d, 0, m, 1, 0, n, 1, 1) | 11010101 | 11 reg:3=d&7 rm:3=m&7 ; evexr(d, m, 1, 0, n, 1, 1, 0, 0, 0) | 11010101 | 11 reg:3=d&7 rm:3=m&7 ; vex(d, 0, n, 1, 0, m, 1, 1) | 11010101 | 11 reg:3=d&7 rm:3=n&7 }" "asm"="vpmullw {m}, {n}, {d}" "dclass"="yv" "feature"="avx2" "nodecode" {
  %r = mul <16 x i16> %n, %m
  ret <16 x i16> %r
}

define <16 x i16> @VPAND.16xi16(<16 x i16> "reg"="yv" %n, <16 x i16> "reg"="yv" %m) "enc"="{ if(m<8) vex(d, 0, m, 1, 0, n, 1, 1) | 11011011 | 11 reg:3=d&7 rm:3=m&7 ; if(n<8) vex(d, 0, n, 1, 0, m, 1, 1) | 11011011 | 11 reg:3=d&7 rm:3=n&7 ; vex(d, 0, m, 1, 0, n, 1, 1) | 11011011 | 11 reg:3=d&7 rm:3=m&7 ; vex(d, 0, n, 1, 0, m, 1, 1) | 11011011 | 11 reg:3=d&7 rm:3=n&7 }" "asm"="vpand {m}, {n}, {d}" "dclass"="yv" "feature"="avx2" "nodecode" {
  %r = and <16 x i16> %n, %m
  ret <16 x i16> %r
}

define <16 x i16> @VPOR.16xi16(<16 x i16> "reg"="yv" %n, <16 x i16> "reg"="yv" %m) "enc"="{ if(m<8) vex(d, 0, m, 1, 0, n, 1, 1) | 11101011 | 11 reg:3=d&7 rm:3=m&7 ; if(n<8) vex(d, 0, n, 1, 0, m, 1, 1) | 11101011 | 11 reg:3=d&7 rm:3=n&7 ; vex(d, 0, m, 1, 0, n, 1, 1) | 11101011 | 11 reg:3=d&7 rm:3=m&7 ; vex(d, 0, n, 1, 0, m, 1, 1) | 11101011 | 11 reg:3=d&7 rm:3=n&7 }" "asm"="vpor {m}, {n}, {d}" "dclass"="yv" "feature"="avx2" "nodecode" {
  %r = or <16 x i16> %n, %m
  ret <16 x i16> %r
}

define <16 x i16> @VPXOR.16xi16(<16 x i16> "reg"="yv" %n, <16 x i16> "reg"="yv" %m) "enc"="{ if(m<8) vex(d, 0, m, 1, 0, n, 1, 1) | 11101111 | 11 reg:3=d&7 rm:3=m&7 ; if(n<8) vex(d, 0, n, 1, 0, m, 1, 1) | 11101111 | 11 reg:3=d&7 rm:3=n&7 ; vex(d, 0, m, 1, 0, n, 1, 1) | 11101111 | 11 reg:3=d&7 rm:3=m&7 ; vex(d, 0, n, 1, 0, m, 1, 1) | 11101111 | 11 reg:3=d&7 rm:3=n&7 }" "asm"="vpxor {m}, {n}, {d}" "dclass"="yv" "feature"="avx2" "nodecode" {
  %r = xor <16 x i16> %n, %m
  ret <16 x i16> %r
}

define <16 x i1> @VPCMPEQW.16xi16(<16 x i16> "reg"="yv" %n, <16 x i16> "reg"="yv" %m) "enc"="{ if(m<8) vex(d, 0, m, 1, 0, n, 1, 1) | 01110101 | 11 reg:3=d&7 rm:3=m&7 ; if(n<8) vex(d, 0, n, 1, 0, m, 1, 1) | 01110101 | 11 reg:3=d&7 rm:3=n&7 ; vex(d, 0, m, 1, 0, n, 1, 1) | 01110101 | 11 reg:3=d&7 rm:3=m&7 ; vex(d, 0, n, 1, 0, m, 1, 1) | 01110101 | 11 reg:3=d&7 rm:3=n&7 }" "asm"="vpcmpeqw {m}, {n}, {d}" "dclass"="yv" "feature"="avx2" "nodecode" {
  %r = icmp eq <16 x i16> %n, %m
  ret <16 x i1> %r
}

define <16 x i1> @VPCMPGTW.16xi16(<16 x i16> "reg"="yv" %n, <16 x i16> "reg"="yv" %m) "enc"="vex(d, 0, m, 1, 0, n, 1, 1) | 01100101 | 11 reg:3=d&7 rm:3=m&7" "asm"="vpcmpgtw {m}, {n}, {d}" "dclass"="yv" "feature"="avx2" "nodecode" {
  %r = icmp sgt <16 x i16> %n, %m
  ret <16 x i1> %r
}

define <16 x i16> @VPMINSW.16xi16(<16 x i16> "reg"="yv" %n, <16 x i16> "reg"="yv" %m) "enc"="{ if(m<8) vex(d, 0, m, 1, 0, n, 1, 1) | 11101010 | 11 reg:3=d&7 rm:3=m&7 ; if(n<8) vex(d, 0, n, 1, 0, m, 1, 1) | 11101010 | 11 reg:3=d&7 rm:3=n&7 ; vex(d, 0, m, 1, 0, n, 1, 1) | 11101010 | 11 reg:3=d&7 rm:3=m&7 ; evexr(d, m, 1, 0, n, 1, 1, 0, 0, 0) | 11101010 | 11 reg:3=d&7 rm:3=m&7 ; vex(d, 0, n, 1, 0, m, 1, 1) | 11101010 | 11 reg:3=d&7 rm:3=n&7 }" "asm"="vpminsw {m}, {n}, {d}" "dclass"="yv" "feature"="avx2" "nodecode" {
  %r = call <16 x i16> @llvm.smin.v16i16(<16 x i16> %n, <16 x i16> %m)
  ret <16 x i16> %r
}

declare <16 x i16> @llvm.smin.v16i16(<16 x i16>, <16 x i16>)

define <16 x i16> @VPMAXSW.16xi16(<16 x i16> "reg"="yv" %n, <16 x i16> "reg"="yv" %m) "enc"="{ if(m<8) vex(d, 0, m, 1, 0, n, 1, 1) | 11101110 | 11 reg:3=d&7 rm:3=m&7 ; if(n<8) vex(d, 0, n, 1, 0, m, 1, 1) | 11101110 | 11 reg:3=d&7 rm:3=n&7 ; vex(d, 0, m, 1, 0, n, 1, 1) | 11101110 | 11 reg:3=d&7 rm:3=m&7 ; evexr(d, m, 1, 0, n, 1, 1, 0, 0, 0) | 11101110 | 11 reg:3=d&7 rm:3=m&7 ; vex(d, 0, n, 1, 0, m, 1, 1) | 11101110 | 11 reg:3=d&7 rm:3=n&7 }" "asm"="vpmaxsw {m}, {n}, {d}" "dclass"="yv" "feature"="avx2" "nodecode" {
  %r = call <16 x i16> @llvm.smax.v16i16(<16 x i16> %n, <16 x i16> %m)
  ret <16 x i16> %r
}

declare <16 x i16> @llvm.smax.v16i16(<16 x i16>, <16 x i16>)

define <16 x i16> @VPMINUW.16xi16(<16 x i16> "reg"="yv" %n, <16 x i16> "reg"="yv" %m) "enc"="{ vex(d, 0, m, 2, 0, n, 1, 1) | 00111010 | 11 reg:3=d&7 rm:3=m&7 ; evexr(d, m, 2, 0, n, 1, 1, 0, 0, 0) | 00111010 | 11 reg:3=d&7 rm:3=m&7 }" "asm"="vpminuw {m}, {n}, {d}" "dclass"="yv" "feature"="avx2" "nodecode" {
  %r = call <16 x i16> @llvm.umin.v16i16(<16 x i16> %n, <16 x i16> %m)
  ret <16 x i16> %r
}

declare <16 x i16> @llvm.umin.v16i16(<16 x i16>, <16 x i16>)

define <16 x i16> @VPMAXUW.16xi16(<16 x i16> "reg"="yv" %n, <16 x i16> "reg"="yv" %m) "enc"="{ vex(d, 0, m, 2, 0, n, 1, 1) | 00111110 | 11 reg:3=d&7 rm:3=m&7 ; evexr(d, m, 2, 0, n, 1, 1, 0, 0, 0) | 00111110 | 11 reg:3=d&7 rm:3=m&7 }" "asm"="vpmaxuw {m}, {n}, {d}" "dclass"="yv" "feature"="avx2" "nodecode" {
  %r = call <16 x i16> @llvm.umax.v16i16(<16 x i16> %n, <16 x i16> %m)
  ret <16 x i16> %r
}

declare <16 x i16> @llvm.umax.v16i16(<16 x i16>, <16 x i16>)

define <16 x i16> @VPSLLWi.16xi16(<16 x i16> "reg"="yv" %n, <16 x i16> "imm" %sh) "enc"="{ vex(6, 0, n, 1, 0, d, 1, 1) | 01110001 | 11 110 rm:3=n&7 | i1:8=sh ; evexr(6, n, 1, 0, d, 1, 1, 0, 0, 0) | 01110001 | 11 110 rm:3=n&7 | i1:8=sh }" "asm"="vpsllw ${sh}, {n}, {d}" "dclass"="yv" "feature"="avx2" "nodecode" {
  %r = shl <16 x i16> %n, %sh
  ret <16 x i16> %r
}

define <16 x i16> @VPSRLWi.16xi16(<16 x i16> "reg"="yv" %n, <16 x i16> "imm" %sh) "enc"="{ vex(2, 0, n, 1, 0, d, 1, 1) | 01110001 | 11 010 rm:3=n&7 | i1:8=sh ; evexr(2, n, 1, 0, d, 1, 1, 0, 0, 0) | 01110001 | 11 010 rm:3=n&7 | i1:8=sh }" "asm"="vpsrlw ${sh}, {n}, {d}" "dclass"="yv" "feature"="avx2" "nodecode" {
  %r = lshr <16 x i16> %n, %sh
  ret <16 x i16> %r
}

define <16 x i16> @VPSRAWi.16xi16(<16 x i16> "reg"="yv" %n, <16 x i16> "imm" %sh) "enc"="{ vex(4, 0, n, 1, 0, d, 1, 1) | 01110001 | 11 100 rm:3=n&7 | i1:8=sh ; evexr(4, n, 1, 0, d, 1, 1, 0, 0, 0) | 01110001 | 11 100 rm:3=n&7 | i1:8=sh }" "asm"="vpsraw ${sh}, {n}, {d}" "dclass"="yv" "feature"="avx2" "nodecode" {
  %r = ashr <16 x i16> %n, %sh
  ret <16 x i16> %r
}

define <16 x i16> @VMASK.16xi16(<16 x i1> "reg"="yv" %n) "enc"="{ vex(2, 0, n, 1, 0, d, 1, 1) | 01110001 | 11 010 rm:3=n&7 | 00001111 ; evexr(2, n, 1, 0, d, 1, 1, 0, 0, 0) | 01110001 | 11 010 rm:3=n&7 | 00001111 }" "asm"="vpsrlw $15, {n}, {d}" "dclass"="yv" "feature"="avx2" "nodecode" {
  %r = zext <16 x i1> %n to <16 x i16>
  ret <16 x i16> %r
}

define <16 x i16> @VPBLENDVB.16xi16(<16 x i1> "reg"="yv" %c, <16 x i16> "reg"="yv" %a, <16 x i16> "reg"="yv" %b) "enc"="vex(d, 0, a, 3, 0, b, 1, 1) | 01001100 | 11 reg:3=d&7 rm:3=a&7 | is4:4=c 0000" "asm"="vpblendvb {c}, {a}, {b}, {d}" "dclass"="yv" "feature"="avx2" "nodecode" {
  %r = select <16 x i1> %c, <16 x i16> %a, <16 x i16> %b
  ret <16 x i16> %r
}

define <8 x i32> @VPADDD.8xi32(<8 x i32> "reg"="yv" %n, <8 x i32> "reg"="yv" %m) "enc"="{ if(m<8) vex(d, 0, m, 1, 0, n, 1, 1) | 11111110 | 11 reg:3=d&7 rm:3=m&7 ; if(n<8) vex(d, 0, n, 1, 0, m, 1, 1) | 11111110 | 11 reg:3=d&7 rm:3=n&7 ; vex(d, 0, m, 1, 0, n, 1, 1) | 11111110 | 11 reg:3=d&7 rm:3=m&7 ; evexr(d, m, 1, 0, n, 1, 1, 0, 0, 0) | 11111110 | 11 reg:3=d&7 rm:3=m&7 ; vex(d, 0, n, 1, 0, m, 1, 1) | 11111110 | 11 reg:3=d&7 rm:3=n&7 }" "asm"="vpaddd {m}, {n}, {d}" "dclass"="yv" "feature"="avx2" "nodecode" {
  %r = add <8 x i32> %n, %m
  ret <8 x i32> %r
}

define <8 x i32> @VPSUBD.8xi32(<8 x i32> "reg"="yv" %n, <8 x i32> "reg"="yv" %m) "enc"="{ vex(d, 0, m, 1, 0, n, 1, 1) | 11111010 | 11 reg:3=d&7 rm:3=m&7 ; evexr(d, m, 1, 0, n, 1, 1, 0, 0, 0) | 11111010 | 11 reg:3=d&7 rm:3=m&7 }" "asm"="vpsubd {m}, {n}, {d}" "dclass"="yv" "feature"="avx2" "nodecode" {
  %r = sub <8 x i32> %n, %m
  ret <8 x i32> %r
}

define <8 x i32> @VPMULLD.8xi32(<8 x i32> "reg"="yv" %n, <8 x i32> "reg"="yv" %m) "enc"="{ vex(d, 0, m, 2, 0, n, 1, 1) | 01000000 | 11 reg:3=d&7 rm:3=m&7 ; evexr(d, m, 2, 0, n, 1, 1, 0, 0, 0) | 01000000 | 11 reg:3=d&7 rm:3=m&7 }" "asm"="vpmulld {m}, {n}, {d}" "dclass"="yv" "feature"="avx2" "nodecode" {
  %r = mul <8 x i32> %n, %m
  ret <8 x i32> %r
}

define <8 x i32> @VPAND.8xi32(<8 x i32> "reg"="yv" %n, <8 x i32> "reg"="yv" %m) "enc"="{ if(m<8) vex(d, 0, m, 1, 0, n, 1, 1) | 11011011 | 11 reg:3=d&7 rm:3=m&7 ; if(n<8) vex(d, 0, n, 1, 0, m, 1, 1) | 11011011 | 11 reg:3=d&7 rm:3=n&7 ; vex(d, 0, m, 1, 0, n, 1, 1) | 11011011 | 11 reg:3=d&7 rm:3=m&7 ; vex(d, 0, n, 1, 0, m, 1, 1) | 11011011 | 11 reg:3=d&7 rm:3=n&7 }" "asm"="vpand {m}, {n}, {d}" "dclass"="yv" "feature"="avx2" "nodecode" {
  %r = and <8 x i32> %n, %m
  ret <8 x i32> %r
}

define <8 x i32> @VPOR.8xi32(<8 x i32> "reg"="yv" %n, <8 x i32> "reg"="yv" %m) "enc"="{ if(m<8) vex(d, 0, m, 1, 0, n, 1, 1) | 11101011 | 11 reg:3=d&7 rm:3=m&7 ; if(n<8) vex(d, 0, n, 1, 0, m, 1, 1) | 11101011 | 11 reg:3=d&7 rm:3=n&7 ; vex(d, 0, m, 1, 0, n, 1, 1) | 11101011 | 11 reg:3=d&7 rm:3=m&7 ; vex(d, 0, n, 1, 0, m, 1, 1) | 11101011 | 11 reg:3=d&7 rm:3=n&7 }" "asm"="vpor {m}, {n}, {d}" "dclass"="yv" "feature"="avx2" "nodecode" {
  %r = or <8 x i32> %n, %m
  ret <8 x i32> %r
}

define <8 x i32> @VPXOR.8xi32(<8 x i32> "reg"="yv" %n, <8 x i32> "reg"="yv" %m) "enc"="{ if(m<8) vex(d, 0, m, 1, 0, n, 1, 1) | 11101111 | 11 reg:3=d&7 rm:3=m&7 ; if(n<8) vex(d, 0, n, 1, 0, m, 1, 1) | 11101111 | 11 reg:3=d&7 rm:3=n&7 ; vex(d, 0, m, 1, 0, n, 1, 1) | 11101111 | 11 reg:3=d&7 rm:3=m&7 ; vex(d, 0, n, 1, 0, m, 1, 1) | 11101111 | 11 reg:3=d&7 rm:3=n&7 }" "asm"="vpxor {m}, {n}, {d}" "dclass"="yv" "feature"="avx2" "nodecode" {
  %r = xor <8 x i32> %n, %m
  ret <8 x i32> %r
}

define <8 x i1> @VPCMPEQD.8xi32(<8 x i32> "reg"="yv" %n, <8 x i32> "reg"="yv" %m) "enc"="{ if(m<8) vex(d, 0, m, 1, 0, n, 1, 1) | 01110110 | 11 reg:3=d&7 rm:3=m&7 ; if(n<8) vex(d, 0, n, 1, 0, m, 1, 1) | 01110110 | 11 reg:3=d&7 rm:3=n&7 ; vex(d, 0, m, 1, 0, n, 1, 1) | 01110110 | 11 reg:3=d&7 rm:3=m&7 ; vex(d, 0, n, 1, 0, m, 1, 1) | 01110110 | 11 reg:3=d&7 rm:3=n&7 }" "asm"="vpcmpeqd {m}, {n}, {d}" "dclass"="yv" "feature"="avx2" "nodecode" {
  %r = icmp eq <8 x i32> %n, %m
  ret <8 x i1> %r
}

define <8 x i1> @VPCMPGTD.8xi32(<8 x i32> "reg"="yv" %n, <8 x i32> "reg"="yv" %m) "enc"="vex(d, 0, m, 1, 0, n, 1, 1) | 01100110 | 11 reg:3=d&7 rm:3=m&7" "asm"="vpcmpgtd {m}, {n}, {d}" "dclass"="yv" "feature"="avx2" "nodecode" {
  %r = icmp sgt <8 x i32> %n, %m
  ret <8 x i1> %r
}

define <8 x i32> @VPMINSD.8xi32(<8 x i32> "reg"="yv" %n, <8 x i32> "reg"="yv" %m) "enc"="{ vex(d, 0, m, 2, 0, n, 1, 1) | 00111001 | 11 reg:3=d&7 rm:3=m&7 ; evexr(d, m, 2, 0, n, 1, 1, 0, 0, 0) | 00111001 | 11 reg:3=d&7 rm:3=m&7 }" "asm"="vpminsd {m}, {n}, {d}" "dclass"="yv" "feature"="avx2" "nodecode" {
  %r = call <8 x i32> @llvm.smin.v8i32(<8 x i32> %n, <8 x i32> %m)
  ret <8 x i32> %r
}

declare <8 x i32> @llvm.smin.v8i32(<8 x i32>, <8 x i32>)

define <8 x i32> @VPMAXSD.8xi32(<8 x i32> "reg"="yv" %n, <8 x i32> "reg"="yv" %m) "enc"="{ vex(d, 0, m, 2, 0, n, 1, 1) | 00111101 | 11 reg:3=d&7 rm:3=m&7 ; evexr(d, m, 2, 0, n, 1, 1, 0, 0, 0) | 00111101 | 11 reg:3=d&7 rm:3=m&7 }" "asm"="vpmaxsd {m}, {n}, {d}" "dclass"="yv" "feature"="avx2" "nodecode" {
  %r = call <8 x i32> @llvm.smax.v8i32(<8 x i32> %n, <8 x i32> %m)
  ret <8 x i32> %r
}

declare <8 x i32> @llvm.smax.v8i32(<8 x i32>, <8 x i32>)

define <8 x i32> @VPMINUD.8xi32(<8 x i32> "reg"="yv" %n, <8 x i32> "reg"="yv" %m) "enc"="{ vex(d, 0, m, 2, 0, n, 1, 1) | 00111011 | 11 reg:3=d&7 rm:3=m&7 ; evexr(d, m, 2, 0, n, 1, 1, 0, 0, 0) | 00111011 | 11 reg:3=d&7 rm:3=m&7 }" "asm"="vpminud {m}, {n}, {d}" "dclass"="yv" "feature"="avx2" "nodecode" {
  %r = call <8 x i32> @llvm.umin.v8i32(<8 x i32> %n, <8 x i32> %m)
  ret <8 x i32> %r
}

declare <8 x i32> @llvm.umin.v8i32(<8 x i32>, <8 x i32>)

define <8 x i32> @VPMAXUD.8xi32(<8 x i32> "reg"="yv" %n, <8 x i32> "reg"="yv" %m) "enc"="{ vex(d, 0, m, 2, 0, n, 1, 1) | 00111111 | 11 reg:3=d&7 rm:3=m&7 ; evexr(d, m, 2, 0, n, 1, 1, 0, 0, 0) | 00111111 | 11 reg:3=d&7 rm:3=m&7 }" "asm"="vpmaxud {m}, {n}, {d}" "dclass"="yv" "feature"="avx2" "nodecode" {
  %r = call <8 x i32> @llvm.umax.v8i32(<8 x i32> %n, <8 x i32> %m)
  ret <8 x i32> %r
}

declare <8 x i32> @llvm.umax.v8i32(<8 x i32>, <8 x i32>)

define <8 x i32> @VPSLLDi.8xi32(<8 x i32> "reg"="yv" %n, <8 x i32> "imm" %sh) "enc"="{ vex(6, 0, n, 1, 0, d, 1, 1) | 01110010 | 11 110 rm:3=n&7 | i1:8=sh ; evexr(6, n, 1, 0, d, 1, 1, 0, 0, 0) | 01110010 | 11 110 rm:3=n&7 | i1:8=sh }" "asm"="vpslld ${sh}, {n}, {d}" "dclass"="yv" "feature"="avx2" "nodecode" {
  %r = shl <8 x i32> %n, %sh
  ret <8 x i32> %r
}

define <8 x i32> @VPSLLVD.8xi32(<8 x i32> "reg"="yv" %n, <8 x i32> "reg"="yv" %m) "enc"="{ vex(d, 0, m, 2, 0, n, 1, 1) | 01000111 | 11 reg:3=d&7 rm:3=m&7 ; evexr(d, m, 2, 0, n, 1, 1, 0, 0, 0) | 01000111 | 11 reg:3=d&7 rm:3=m&7 }" "asm"="vpsllvd {m}, {n}, {d}" "dclass"="yv" "feature"="avx2" "nodecode" {
  %r = shl <8 x i32> %n, %m
  ret <8 x i32> %r
}

define <8 x i32> @VPSRLDi.8xi32(<8 x i32> "reg"="yv" %n, <8 x i32> "imm" %sh) "enc"="{ vex(2, 0, n, 1, 0, d, 1, 1) | 01110010 | 11 010 rm:3=n&7 | i1:8=sh ; evexr(2, n, 1, 0, d, 1, 1, 0, 0, 0) | 01110010 | 11 010 rm:3=n&7 | i1:8=sh }" "asm"="vpsrld ${sh}, {n}, {d}" "dclass"="yv" "feature"="avx2" "nodecode" {
  %r = lshr <8 x i32> %n, %sh
  ret <8 x i32> %r
}

define <8 x i32> @VPSRLVD.8xi32(<8 x i32> "reg"="yv" %n, <8 x i32> "reg"="yv" %m) "enc"="{ vex(d, 0, m, 2, 0, n, 1, 1) | 01000101 | 11 reg:3=d&7 rm:3=m&7 ; evexr(d, m, 2, 0, n, 1, 1, 0, 0, 0) | 01000101 | 11 reg:3=d&7 rm:3=m&7 }" "asm"="vpsrlvd {m}, {n}, {d}" "dclass"="yv" "feature"="avx2" "nodecode" {
  %r = lshr <8 x i32> %n, %m
  ret <8 x i32> %r
}

define <8 x i32> @VPSRADi.8xi32(<8 x i32> "reg"="yv" %n, <8 x i32> "imm" %sh) "enc"="{ vex(4, 0, n, 1, 0, d, 1, 1) | 01110010 | 11 100 rm:3=n&7 | i1:8=sh ; evexr(4, n, 1, 0, d, 1, 1, 0, 0, 0) | 01110010 | 11 100 rm:3=n&7 | i1:8=sh }" "asm"="vpsrad ${sh}, {n}, {d}" "dclass"="yv" "feature"="avx2" "nodecode" {
  %r = ashr <8 x i32> %n, %sh
  ret <8 x i32> %r
}

define <8 x i32> @VPSRAVD.8xi32(<8 x i32> "reg"="yv" %n, <8 x i32> "reg"="yv" %m) "enc"="{ vex(d, 0, m, 2, 0, n, 1, 1) | 01000110 | 11 reg:3=d&7 rm:3=m&7 ; evexr(d, m, 2, 0, n, 1, 1, 0, 0, 0) | 01000110 | 11 reg:3=d&7 rm:3=m&7 }" "asm"="vpsravd {m}, {n}, {d}" "dclass"="yv" "feature"="avx2" "nodecode" {
  %r = ashr <8 x i32> %n, %m
  ret <8 x i32> %r
}

define <8 x i32> @VMASK.8xi32(<8 x i1> "reg"="yv" %n) "enc"="{ vex(2, 0, n, 1, 0, d, 1, 1) | 01110010 | 11 010 rm:3=n&7 | 00011111 ; evexr(2, n, 1, 0, d, 1, 1, 0, 0, 0) | 01110010 | 11 010 rm:3=n&7 | 00011111 }" "asm"="vpsrld $31, {n}, {d}" "dclass"="yv" "feature"="avx2" "nodecode" {
  %r = zext <8 x i1> %n to <8 x i32>
  ret <8 x i32> %r
}

define <8 x i32> @VPBLENDVB.8xi32(<8 x i1> "reg"="yv" %c, <8 x i32> "reg"="yv" %a, <8 x i32> "reg"="yv" %b) "enc"="vex(d, 0, a, 3, 0, b, 1, 1) | 01001100 | 11 reg:3=d&7 rm:3=a&7 | is4:4=c 0000" "asm"="vpblendvb {c}, {a}, {b}, {d}" "dclass"="yv" "feature"="avx2" "nodecode" {
  %r = select <8 x i1> %c, <8 x i32> %a, <8 x i32> %b
  ret <8 x i32> %r
}

define <4 x i64> @VPADDQ.4xi64(<4 x i64> "reg"="yv" %n, <4 x i64> "reg"="yv" %m) "enc"="{ if(m<8) vex(d, 0, m, 1, 0, n, 1, 1) | 11010100 | 11 reg:3=d&7 rm:3=m&7 ; if(n<8) vex(d, 0, n, 1, 0, m, 1, 1) | 11010100 | 11 reg:3=d&7 rm:3=n&7 ; vex(d, 0, m, 1, 0, n, 1, 1) | 11010100 | 11 reg:3=d&7 rm:3=m&7 ; evexr(d, m, 1, 1, n, 1, 1, 0, 0, 0) | 11010100 | 11 reg:3=d&7 rm:3=m&7 ; vex(d, 0, n, 1, 0, m, 1, 1) | 11010100 | 11 reg:3=d&7 rm:3=n&7 }" "asm"="vpaddq {m}, {n}, {d}" "dclass"="yv" "feature"="avx2" "nodecode" {
  %r = add <4 x i64> %n, %m
  ret <4 x i64> %r
}

define <4 x i64> @VPSUBQ.4xi64(<4 x i64> "reg"="yv" %n, <4 x i64> "reg"="yv" %m) "enc"="{ vex(d, 0, m, 1, 0, n, 1, 1) | 11111011 | 11 reg:3=d&7 rm:3=m&7 ; evexr(d, m, 1, 1, n, 1, 1, 0, 0, 0) | 11111011 | 11 reg:3=d&7 rm:3=m&7 }" "asm"="vpsubq {m}, {n}, {d}" "dclass"="yv" "feature"="avx2" "nodecode" {
  %r = sub <4 x i64> %n, %m
  ret <4 x i64> %r
}

define <4 x i64> @VPAND.4xi64(<4 x i64> "reg"="yv" %n, <4 x i64> "reg"="yv" %m) "enc"="{ if(m<8) vex(d, 0, m, 1, 0, n, 1, 1) | 11011011 | 11 reg:3=d&7 rm:3=m&7 ; if(n<8) vex(d, 0, n, 1, 0, m, 1, 1) | 11011011 | 11 reg:3=d&7 rm:3=n&7 ; vex(d, 0, m, 1, 0, n, 1, 1) | 11011011 | 11 reg:3=d&7 rm:3=m&7 ; vex(d, 0, n, 1, 0, m, 1, 1) | 11011011 | 11 reg:3=d&7 rm:3=n&7 }" "asm"="vpand {m}, {n}, {d}" "dclass"="yv" "feature"="avx2" "nodecode" {
  %r = and <4 x i64> %n, %m
  ret <4 x i64> %r
}

define <4 x i64> @VPOR.4xi64(<4 x i64> "reg"="yv" %n, <4 x i64> "reg"="yv" %m) "enc"="{ if(m<8) vex(d, 0, m, 1, 0, n, 1, 1) | 11101011 | 11 reg:3=d&7 rm:3=m&7 ; if(n<8) vex(d, 0, n, 1, 0, m, 1, 1) | 11101011 | 11 reg:3=d&7 rm:3=n&7 ; vex(d, 0, m, 1, 0, n, 1, 1) | 11101011 | 11 reg:3=d&7 rm:3=m&7 ; vex(d, 0, n, 1, 0, m, 1, 1) | 11101011 | 11 reg:3=d&7 rm:3=n&7 }" "asm"="vpor {m}, {n}, {d}" "dclass"="yv" "feature"="avx2" "nodecode" {
  %r = or <4 x i64> %n, %m
  ret <4 x i64> %r
}

define <4 x i64> @VPXOR.4xi64(<4 x i64> "reg"="yv" %n, <4 x i64> "reg"="yv" %m) "enc"="{ if(m<8) vex(d, 0, m, 1, 0, n, 1, 1) | 11101111 | 11 reg:3=d&7 rm:3=m&7 ; if(n<8) vex(d, 0, n, 1, 0, m, 1, 1) | 11101111 | 11 reg:3=d&7 rm:3=n&7 ; vex(d, 0, m, 1, 0, n, 1, 1) | 11101111 | 11 reg:3=d&7 rm:3=m&7 ; vex(d, 0, n, 1, 0, m, 1, 1) | 11101111 | 11 reg:3=d&7 rm:3=n&7 }" "asm"="vpxor {m}, {n}, {d}" "dclass"="yv" "feature"="avx2" "nodecode" {
  %r = xor <4 x i64> %n, %m
  ret <4 x i64> %r
}

define <4 x i1> @VPCMPEQQ.4xi64(<4 x i64> "reg"="yv" %n, <4 x i64> "reg"="yv" %m) "enc"="vex(d, 0, m, 2, 0, n, 1, 1) | 00101001 | 11 reg:3=d&7 rm:3=m&7" "asm"="vpcmpeqq {m}, {n}, {d}" "dclass"="yv" "feature"="avx2" "nodecode" {
  %r = icmp eq <4 x i64> %n, %m
  ret <4 x i1> %r
}

define <4 x i1> @VPCMPGTQ.4xi64(<4 x i64> "reg"="yv" %n, <4 x i64> "reg"="yv" %m) "enc"="vex(d, 0, m, 2, 0, n, 1, 1) | 00110111 | 11 reg:3=d&7 rm:3=m&7" "asm"="vpcmpgtq {m}, {n}, {d}" "dclass"="yv" "feature"="avx2" "nodecode" {
  %r = icmp sgt <4 x i64> %n, %m
  ret <4 x i1> %r
}

define <4 x i64> @VPSLLQi.4xi64(<4 x i64> "reg"="yv" %n, <4 x i64> "imm" %sh) "enc"="{ vex(6, 0, n, 1, 0, d, 1, 1) | 01110011 | 11 110 rm:3=n&7 | i1:8=sh ; evexr(6, n, 1, 1, d, 1, 1, 0, 0, 0) | 01110011 | 11 110 rm:3=n&7 | i1:8=sh }" "asm"="vpsllq ${sh}, {n}, {d}" "dclass"="yv" "feature"="avx2" "nodecode" {
  %r = shl <4 x i64> %n, %sh
  ret <4 x i64> %r
}

define <4 x i64> @VPSLLVQ.4xi64(<4 x i64> "reg"="yv" %n, <4 x i64> "reg"="yv" %m) "enc"="{ vex(d, 0, m, 2, 1, n, 1, 1) | 01000111 | 11 reg:3=d&7 rm:3=m&7 ; evexr(d, m, 2, 1, n, 1, 1, 0, 0, 0) | 01000111 | 11 reg:3=d&7 rm:3=m&7 }" "asm"="vpsllvq {m}, {n}, {d}" "dclass"="yv" "feature"="avx2" "nodecode" {
  %r = shl <4 x i64> %n, %m
  ret <4 x i64> %r
}

define <4 x i64> @VPSRLQi.4xi64(<4 x i64> "reg"="yv" %n, <4 x i64> "imm" %sh) "enc"="{ vex(2, 0, n, 1, 0, d, 1, 1) | 01110011 | 11 010 rm:3=n&7 | i1:8=sh ; evexr(2, n, 1, 1, d, 1, 1, 0, 0, 0) | 01110011 | 11 010 rm:3=n&7 | i1:8=sh }" "asm"="vpsrlq ${sh}, {n}, {d}" "dclass"="yv" "feature"="avx2" "nodecode" {
  %r = lshr <4 x i64> %n, %sh
  ret <4 x i64> %r
}

define <4 x i64> @VPSRLVQ.4xi64(<4 x i64> "reg"="yv" %n, <4 x i64> "reg"="yv" %m) "enc"="{ vex(d, 0, m, 2, 1, n, 1, 1) | 01000101 | 11 reg:3=d&7 rm:3=m&7 ; evexr(d, m, 2, 1, n, 1, 1, 0, 0, 0) | 01000101 | 11 reg:3=d&7 rm:3=m&7 }" "asm"="vpsrlvq {m}, {n}, {d}" "dclass"="yv" "feature"="avx2" "nodecode" {
  %r = lshr <4 x i64> %n, %m
  ret <4 x i64> %r
}

define <4 x i64> @VMASK.4xi64(<4 x i1> "reg"="yv" %n) "enc"="{ vex(2, 0, n, 1, 0, d, 1, 1) | 01110011 | 11 010 rm:3=n&7 | 00111111 ; evexr(2, n, 1, 1, d, 1, 1, 0, 0, 0) | 01110011 | 11 010 rm:3=n&7 | 00111111 }" "asm"="vpsrlq $63, {n}, {d}" "dclass"="yv" "feature"="avx2" "nodecode" {
  %r = zext <4 x i1> %n to <4 x i64>
  ret <4 x i64> %r
}

define <4 x i64> @VPBLENDVB.4xi64(<4 x i1> "reg"="yv" %c, <4 x i64> "reg"="yv" %a, <4 x i64> "reg"="yv" %b) "enc"="vex(d, 0, a, 3, 0, b, 1, 1) | 01001100 | 11 reg:3=d&7 rm:3=a&7 | is4:4=c 0000" "asm"="vpblendvb {c}, {a}, {b}, {d}" "dclass"="yv" "feature"="avx2" "nodecode" {
  %r = select <4 x i1> %c, <4 x i64> %a, <4 x i64> %b
  ret <4 x i64> %r
}

define <4 x i32> @VPSLLVDx.4xi32(<4 x i32> "reg"="xv" %n, <4 x i32> "reg"="xv" %m) "enc"="{ vex(d, 0, m, 2, 0, n, 0, 1) | 01000111 | 11 reg:3=d&7 rm:3=m&7 ; evexr(d, m, 2, 0, n, 0, 1, 0, 0, 0) | 01000111 | 11 reg:3=d&7 rm:3=m&7 }" "asm"="vpsllvd {m}, {n}, {d}" "dclass"="xv" "feature"="avx2" "nodecode" {
  %r = shl <4 x i32> %n, %m
  ret <4 x i32> %r
}

define <4 x i32> @VPSRLVDx.4xi32(<4 x i32> "reg"="xv" %n, <4 x i32> "reg"="xv" %m) "enc"="{ vex(d, 0, m, 2, 0, n, 0, 1) | 01000101 | 11 reg:3=d&7 rm:3=m&7 ; evexr(d, m, 2, 0, n, 0, 1, 0, 0, 0) | 01000101 | 11 reg:3=d&7 rm:3=m&7 }" "asm"="vpsrlvd {m}, {n}, {d}" "dclass"="xv" "feature"="avx2" "nodecode" {
  %r = lshr <4 x i32> %n, %m
  ret <4 x i32> %r
}

define <4 x i32> @VPSRAVDx.4xi32(<4 x i32> "reg"="xv" %n, <4 x i32> "reg"="xv" %m) "enc"="{ vex(d, 0, m, 2, 0, n, 0, 1) | 01000110 | 11 reg:3=d&7 rm:3=m&7 ; evexr(d, m, 2, 0, n, 0, 1, 0, 0, 0) | 01000110 | 11 reg:3=d&7 rm:3=m&7 }" "asm"="vpsravd {m}, {n}, {d}" "dclass"="xv" "feature"="avx2" "nodecode" {
  %r = ashr <4 x i32> %n, %m
  ret <4 x i32> %r
}

define <2 x i64> @VPSLLVQx.2xi64(<2 x i64> "reg"="xv" %n, <2 x i64> "reg"="xv" %m) "enc"="{ vex(d, 0, m, 2, 1, n, 0, 1) | 01000111 | 11 reg:3=d&7 rm:3=m&7 ; evexr(d, m, 2, 1, n, 0, 1, 0, 0, 0) | 01000111 | 11 reg:3=d&7 rm:3=m&7 }" "asm"="vpsllvq {m}, {n}, {d}" "dclass"="xv" "feature"="avx2" "nodecode" {
  %r = shl <2 x i64> %n, %m
  ret <2 x i64> %r
}

define <2 x i64> @VPSRLVQx.2xi64(<2 x i64> "reg"="xv" %n, <2 x i64> "reg"="xv" %m) "enc"="{ vex(d, 0, m, 2, 1, n, 0, 1) | 01000101 | 11 reg:3=d&7 rm:3=m&7 ; evexr(d, m, 2, 1, n, 0, 1, 0, 0, 0) | 01000101 | 11 reg:3=d&7 rm:3=m&7 }" "asm"="vpsrlvq {m}, {n}, {d}" "dclass"="xv" "feature"="avx2" "nodecode" {
  %r = lshr <2 x i64> %n, %m
  ret <2 x i64> %r
}

define <8 x float> @VADDPS.8xf32(<8 x float> "reg"="yv" %n, <8 x float> "reg"="yv" %m) "enc"="{ if(m<8) vex(d, 0, m, 1, 0, n, 1, 0) | 01011000 | 11 reg:3=d&7 rm:3=m&7 ; if(n<8) vex(d, 0, n, 1, 0, m, 1, 0) | 01011000 | 11 reg:3=d&7 rm:3=n&7 ; vex(d, 0, m, 1, 0, n, 1, 0) | 01011000 | 11 reg:3=d&7 rm:3=m&7 ; evexr(d, m, 1, 0, n, 1, 0, 0, 0, 0) | 01011000 | 11 reg:3=d&7 rm:3=m&7 ; vex(d, 0, n, 1, 0, m, 1, 0) | 01011000 | 11 reg:3=d&7 rm:3=n&7 }" "asm"="vaddps {m}, {n}, {d}" "dclass"="yv" "feature"="avx2" "nodecode" {
  %r = fadd <8 x float> %n, %m
  ret <8 x float> %r
}

define <8 x float> @VSUBPS.8xf32(<8 x float> "reg"="yv" %n, <8 x float> "reg"="yv" %m) "enc"="{ vex(d, 0, m, 1, 0, n, 1, 0) | 01011100 | 11 reg:3=d&7 rm:3=m&7 ; evexr(d, m, 1, 0, n, 1, 0, 0, 0, 0) | 01011100 | 11 reg:3=d&7 rm:3=m&7 }" "asm"="vsubps {m}, {n}, {d}" "dclass"="yv" "feature"="avx2" "nodecode" {
  %r = fsub <8 x float> %n, %m
  ret <8 x float> %r
}

define <8 x float> @VMULPS.8xf32(<8 x float> "reg"="yv" %n, <8 x float> "reg"="yv" %m) "enc"="{ if(m<8) vex(d, 0, m, 1, 0, n, 1, 0) | 01011001 | 11 reg:3=d&7 rm:3=m&7 ; if(n<8) vex(d, 0, n, 1, 0, m, 1, 0) | 01011001 | 11 reg:3=d&7 rm:3=n&7 ; vex(d, 0, m, 1, 0, n, 1, 0) | 01011001 | 11 reg:3=d&7 rm:3=m&7 ; evexr(d, m, 1, 0, n, 1, 0, 0, 0, 0) | 01011001 | 11 reg:3=d&7 rm:3=m&7 ; vex(d, 0, n, 1, 0, m, 1, 0) | 01011001 | 11 reg:3=d&7 rm:3=n&7 }" "asm"="vmulps {m}, {n}, {d}" "dclass"="yv" "feature"="avx2" "nodecode" {
  %r = fmul <8 x float> %n, %m
  ret <8 x float> %r
}

define <8 x float> @VDIVPS.8xf32(<8 x float> "reg"="yv" %n, <8 x float> "reg"="yv" %m) "enc"="{ vex(d, 0, m, 1, 0, n, 1, 0) | 01011110 | 11 reg:3=d&7 rm:3=m&7 ; evexr(d, m, 1, 0, n, 1, 0, 0, 0, 0) | 01011110 | 11 reg:3=d&7 rm:3=m&7 }" "asm"="vdivps {m}, {n}, {d}" "dclass"="yv" "feature"="avx2" "nodecode" {
  %r = fdiv <8 x float> %n, %m
  ret <8 x float> %r
}

define <8 x float> @VSQRTPS.8xf32(<8 x float> "reg"="yv" %n) "enc"="{ vex(d, 0, n, 1, 0, 0, 1, 0) | 01010001 | 11 reg:3=d&7 rm:3=n&7 ; evexr(d, n, 1, 0, 0, 1, 0, 0, 0, 0) | 01010001 | 11 reg:3=d&7 rm:3=n&7 }" "asm"="vsqrtps {n}, {d}" "dclass"="yv" "feature"="avx2" "nodecode" {
  %r = call <8 x float> @llvm.sqrt.v8f32(<8 x float> %n)
  ret <8 x float> %r
}

declare <8 x float> @llvm.sqrt.v8f32(<8 x float>)

define <8 x i1> @VCMPOEQPS.8xf32(<8 x float> "reg"="yv" %n, <8 x float> "reg"="yv" %m) "enc"="{ if(m<8) vex(d, 0, m, 1, 0, n, 1, 0) | 11000010 | 11 reg:3=d&7 rm:3=m&7 | i1:8=0 ; if(n<8) if(((0&3)==0)|((0&3)==3)) vex(d, 0, n, 1, 0, m, 1, 0) | 11000010 | 11 reg:3=d&7 rm:3=n&7 | i1:8=0 ; vex(d, 0, m, 1, 0, n, 1, 0) | 11000010 | 11 reg:3=d&7 rm:3=m&7 | i1:8=0 ; if(((0&3)==0)|((0&3)==3)) vex(d, 0, n, 1, 0, m, 1, 0) | 11000010 | 11 reg:3=d&7 rm:3=n&7 | i1:8=0 }" "asm"="vcmpps $0, {m}, {n}, {d}" "dclass"="yv" "feature"="avx2" "nodecode" {
  %r = fcmp oeq <8 x float> %n, %m
  ret <8 x i1> %r
}

define <8 x i1> @VCMPOLTPS.8xf32(<8 x float> "reg"="yv" %n, <8 x float> "reg"="yv" %m) "enc"="{ if(m<8) vex(d, 0, m, 1, 0, n, 1, 0) | 11000010 | 11 reg:3=d&7 rm:3=m&7 | i1:8=1 ; if(n<8) if(((1&3)==0)|((1&3)==3)) vex(d, 0, n, 1, 0, m, 1, 0) | 11000010 | 11 reg:3=d&7 rm:3=n&7 | i1:8=1 ; vex(d, 0, m, 1, 0, n, 1, 0) | 11000010 | 11 reg:3=d&7 rm:3=m&7 | i1:8=1 ; if(((1&3)==0)|((1&3)==3)) vex(d, 0, n, 1, 0, m, 1, 0) | 11000010 | 11 reg:3=d&7 rm:3=n&7 | i1:8=1 }" "asm"="vcmpps $1, {m}, {n}, {d}" "dclass"="yv" "feature"="avx2" "nodecode" {
  %r = fcmp olt <8 x float> %n, %m
  ret <8 x i1> %r
}

define <8 x i1> @VCMPOLEPS.8xf32(<8 x float> "reg"="yv" %n, <8 x float> "reg"="yv" %m) "enc"="{ if(m<8) vex(d, 0, m, 1, 0, n, 1, 0) | 11000010 | 11 reg:3=d&7 rm:3=m&7 | i1:8=2 ; if(n<8) if(((2&3)==0)|((2&3)==3)) vex(d, 0, n, 1, 0, m, 1, 0) | 11000010 | 11 reg:3=d&7 rm:3=n&7 | i1:8=2 ; vex(d, 0, m, 1, 0, n, 1, 0) | 11000010 | 11 reg:3=d&7 rm:3=m&7 | i1:8=2 ; if(((2&3)==0)|((2&3)==3)) vex(d, 0, n, 1, 0, m, 1, 0) | 11000010 | 11 reg:3=d&7 rm:3=n&7 | i1:8=2 }" "asm"="vcmpps $2, {m}, {n}, {d}" "dclass"="yv" "feature"="avx2" "nodecode" {
  %r = fcmp ole <8 x float> %n, %m
  ret <8 x i1> %r
}

define <8 x i1> @VCMPUNOPS.8xf32(<8 x float> "reg"="yv" %n, <8 x float> "reg"="yv" %m) "enc"="{ if(m<8) vex(d, 0, m, 1, 0, n, 1, 0) | 11000010 | 11 reg:3=d&7 rm:3=m&7 | i1:8=3 ; if(n<8) if(((3&3)==0)|((3&3)==3)) vex(d, 0, n, 1, 0, m, 1, 0) | 11000010 | 11 reg:3=d&7 rm:3=n&7 | i1:8=3 ; vex(d, 0, m, 1, 0, n, 1, 0) | 11000010 | 11 reg:3=d&7 rm:3=m&7 | i1:8=3 ; if(((3&3)==0)|((3&3)==3)) vex(d, 0, n, 1, 0, m, 1, 0) | 11000010 | 11 reg:3=d&7 rm:3=n&7 | i1:8=3 }" "asm"="vcmpps $3, {m}, {n}, {d}" "dclass"="yv" "feature"="avx2" "nodecode" {
  %r = fcmp uno <8 x float> %n, %m
  ret <8 x i1> %r
}

define <8 x i1> @VCMPUNEPS.8xf32(<8 x float> "reg"="yv" %n, <8 x float> "reg"="yv" %m) "enc"="{ if(m<8) vex(d, 0, m, 1, 0, n, 1, 0) | 11000010 | 11 reg:3=d&7 rm:3=m&7 | i1:8=4 ; if(n<8) if(((4&3)==0)|((4&3)==3)) vex(d, 0, n, 1, 0, m, 1, 0) | 11000010 | 11 reg:3=d&7 rm:3=n&7 | i1:8=4 ; vex(d, 0, m, 1, 0, n, 1, 0) | 11000010 | 11 reg:3=d&7 rm:3=m&7 | i1:8=4 ; if(((4&3)==0)|((4&3)==3)) vex(d, 0, n, 1, 0, m, 1, 0) | 11000010 | 11 reg:3=d&7 rm:3=n&7 | i1:8=4 }" "asm"="vcmpps $4, {m}, {n}, {d}" "dclass"="yv" "feature"="avx2" "nodecode" {
  %r = fcmp une <8 x float> %n, %m
  ret <8 x i1> %r
}

define <8 x i1> @VCMPUGEPS.8xf32(<8 x float> "reg"="yv" %n, <8 x float> "reg"="yv" %m) "enc"="{ if(m<8) vex(d, 0, m, 1, 0, n, 1, 0) | 11000010 | 11 reg:3=d&7 rm:3=m&7 | i1:8=5 ; if(n<8) if(((5&3)==0)|((5&3)==3)) vex(d, 0, n, 1, 0, m, 1, 0) | 11000010 | 11 reg:3=d&7 rm:3=n&7 | i1:8=5 ; vex(d, 0, m, 1, 0, n, 1, 0) | 11000010 | 11 reg:3=d&7 rm:3=m&7 | i1:8=5 ; if(((5&3)==0)|((5&3)==3)) vex(d, 0, n, 1, 0, m, 1, 0) | 11000010 | 11 reg:3=d&7 rm:3=n&7 | i1:8=5 }" "asm"="vcmpps $5, {m}, {n}, {d}" "dclass"="yv" "feature"="avx2" "nodecode" {
  %r = fcmp uge <8 x float> %n, %m
  ret <8 x i1> %r
}

define <8 x i1> @VCMPUGTPS.8xf32(<8 x float> "reg"="yv" %n, <8 x float> "reg"="yv" %m) "enc"="{ if(m<8) vex(d, 0, m, 1, 0, n, 1, 0) | 11000010 | 11 reg:3=d&7 rm:3=m&7 | i1:8=6 ; if(n<8) if(((6&3)==0)|((6&3)==3)) vex(d, 0, n, 1, 0, m, 1, 0) | 11000010 | 11 reg:3=d&7 rm:3=n&7 | i1:8=6 ; vex(d, 0, m, 1, 0, n, 1, 0) | 11000010 | 11 reg:3=d&7 rm:3=m&7 | i1:8=6 ; if(((6&3)==0)|((6&3)==3)) vex(d, 0, n, 1, 0, m, 1, 0) | 11000010 | 11 reg:3=d&7 rm:3=n&7 | i1:8=6 }" "asm"="vcmpps $6, {m}, {n}, {d}" "dclass"="yv" "feature"="avx2" "nodecode" {
  %r = fcmp ugt <8 x float> %n, %m
  ret <8 x i1> %r
}

define <8 x i1> @VCMPORDPS.8xf32(<8 x float> "reg"="yv" %n, <8 x float> "reg"="yv" %m) "enc"="{ if(m<8) vex(d, 0, m, 1, 0, n, 1, 0) | 11000010 | 11 reg:3=d&7 rm:3=m&7 | i1:8=7 ; if(n<8) if(((7&3)==0)|((7&3)==3)) vex(d, 0, n, 1, 0, m, 1, 0) | 11000010 | 11 reg:3=d&7 rm:3=n&7 | i1:8=7 ; vex(d, 0, m, 1, 0, n, 1, 0) | 11000010 | 11 reg:3=d&7 rm:3=m&7 | i1:8=7 ; if(((7&3)==0)|((7&3)==3)) vex(d, 0, n, 1, 0, m, 1, 0) | 11000010 | 11 reg:3=d&7 rm:3=n&7 | i1:8=7 }" "asm"="vcmpps $7, {m}, {n}, {d}" "dclass"="yv" "feature"="avx2" "nodecode" {
  %r = fcmp ord <8 x float> %n, %m
  ret <8 x i1> %r
}

define <8 x i1> @VCMPUEQPS.8xf32(<8 x float> "reg"="yv" %n, <8 x float> "reg"="yv" %m) "enc"="{ if(m<8) vex(d, 0, m, 1, 0, n, 1, 0) | 11000010 | 11 reg:3=d&7 rm:3=m&7 | i1:8=8 ; if(n<8) if(((8&3)==0)|((8&3)==3)) vex(d, 0, n, 1, 0, m, 1, 0) | 11000010 | 11 reg:3=d&7 rm:3=n&7 | i1:8=8 ; vex(d, 0, m, 1, 0, n, 1, 0) | 11000010 | 11 reg:3=d&7 rm:3=m&7 | i1:8=8 ; if(((8&3)==0)|((8&3)==3)) vex(d, 0, n, 1, 0, m, 1, 0) | 11000010 | 11 reg:3=d&7 rm:3=n&7 | i1:8=8 }" "asm"="vcmpps $8, {m}, {n}, {d}" "dclass"="yv" "feature"="avx2" "nodecode" {
  %r = fcmp ueq <8 x float> %n, %m
  ret <8 x i1> %r
}

define <8 x i1> @VCMPULTPS.8xf32(<8 x float> "reg"="yv" %n, <8 x float> "reg"="yv" %m) "enc"="{ if(m<8) vex(d, 0, m, 1, 0, n, 1, 0) | 11000010 | 11 reg:3=d&7 rm:3=m&7 | i1:8=9 ; if(n<8) if(((9&3)==0)|((9&3)==3)) vex(d, 0, n, 1, 0, m, 1, 0) | 11000010 | 11 reg:3=d&7 rm:3=n&7 | i1:8=9 ; vex(d, 0, m, 1, 0, n, 1, 0) | 11000010 | 11 reg:3=d&7 rm:3=m&7 | i1:8=9 ; if(((9&3)==0)|((9&3)==3)) vex(d, 0, n, 1, 0, m, 1, 0) | 11000010 | 11 reg:3=d&7 rm:3=n&7 | i1:8=9 }" "asm"="vcmpps $9, {m}, {n}, {d}" "dclass"="yv" "feature"="avx2" "nodecode" {
  %r = fcmp ult <8 x float> %n, %m
  ret <8 x i1> %r
}

define <8 x i1> @VCMPULEPS.8xf32(<8 x float> "reg"="yv" %n, <8 x float> "reg"="yv" %m) "enc"="{ if(m<8) vex(d, 0, m, 1, 0, n, 1, 0) | 11000010 | 11 reg:3=d&7 rm:3=m&7 | i1:8=10 ; if(n<8) if(((10&3)==0)|((10&3)==3)) vex(d, 0, n, 1, 0, m, 1, 0) | 11000010 | 11 reg:3=d&7 rm:3=n&7 | i1:8=10 ; vex(d, 0, m, 1, 0, n, 1, 0) | 11000010 | 11 reg:3=d&7 rm:3=m&7 | i1:8=10 ; if(((10&3)==0)|((10&3)==3)) vex(d, 0, n, 1, 0, m, 1, 0) | 11000010 | 11 reg:3=d&7 rm:3=n&7 | i1:8=10 }" "asm"="vcmpps $10, {m}, {n}, {d}" "dclass"="yv" "feature"="avx2" "nodecode" {
  %r = fcmp ule <8 x float> %n, %m
  ret <8 x i1> %r
}

define <8 x i1> @VCMPONEPS.8xf32(<8 x float> "reg"="yv" %n, <8 x float> "reg"="yv" %m) "enc"="{ if(m<8) vex(d, 0, m, 1, 0, n, 1, 0) | 11000010 | 11 reg:3=d&7 rm:3=m&7 | i1:8=12 ; if(n<8) if(((12&3)==0)|((12&3)==3)) vex(d, 0, n, 1, 0, m, 1, 0) | 11000010 | 11 reg:3=d&7 rm:3=n&7 | i1:8=12 ; vex(d, 0, m, 1, 0, n, 1, 0) | 11000010 | 11 reg:3=d&7 rm:3=m&7 | i1:8=12 ; if(((12&3)==0)|((12&3)==3)) vex(d, 0, n, 1, 0, m, 1, 0) | 11000010 | 11 reg:3=d&7 rm:3=n&7 | i1:8=12 }" "asm"="vcmpps $12, {m}, {n}, {d}" "dclass"="yv" "feature"="avx2" "nodecode" {
  %r = fcmp one <8 x float> %n, %m
  ret <8 x i1> %r
}

define <8 x i1> @VCMPOGEPS.8xf32(<8 x float> "reg"="yv" %n, <8 x float> "reg"="yv" %m) "enc"="{ if(m<8) vex(d, 0, m, 1, 0, n, 1, 0) | 11000010 | 11 reg:3=d&7 rm:3=m&7 | i1:8=13 ; if(n<8) if(((13&3)==0)|((13&3)==3)) vex(d, 0, n, 1, 0, m, 1, 0) | 11000010 | 11 reg:3=d&7 rm:3=n&7 | i1:8=13 ; vex(d, 0, m, 1, 0, n, 1, 0) | 11000010 | 11 reg:3=d&7 rm:3=m&7 | i1:8=13 ; if(((13&3)==0)|((13&3)==3)) vex(d, 0, n, 1, 0, m, 1, 0) | 11000010 | 11 reg:3=d&7 rm:3=n&7 | i1:8=13 }" "asm"="vcmpps $13, {m}, {n}, {d}" "dclass"="yv" "feature"="avx2" "nodecode" {
  %r = fcmp oge <8 x float> %n, %m
  ret <8 x i1> %r
}

define <8 x i1> @VCMPOGTPS.8xf32(<8 x float> "reg"="yv" %n, <8 x float> "reg"="yv" %m) "enc"="{ if(m<8) vex(d, 0, m, 1, 0, n, 1, 0) | 11000010 | 11 reg:3=d&7 rm:3=m&7 | i1:8=14 ; if(n<8) if(((14&3)==0)|((14&3)==3)) vex(d, 0, n, 1, 0, m, 1, 0) | 11000010 | 11 reg:3=d&7 rm:3=n&7 | i1:8=14 ; vex(d, 0, m, 1, 0, n, 1, 0) | 11000010 | 11 reg:3=d&7 rm:3=m&7 | i1:8=14 ; if(((14&3)==0)|((14&3)==3)) vex(d, 0, n, 1, 0, m, 1, 0) | 11000010 | 11 reg:3=d&7 rm:3=n&7 | i1:8=14 }" "asm"="vcmpps $14, {m}, {n}, {d}" "dclass"="yv" "feature"="avx2" "nodecode" {
  %r = fcmp ogt <8 x float> %n, %m
  ret <8 x i1> %r
}

define <8 x float> @VBLENDVPS.8xf32(<8 x i1> "reg"="yv" %c, <8 x float> "reg"="yv" %a, <8 x float> "reg"="yv" %b) "enc"="vex(d, 0, a, 3, 0, b, 1, 1) | 01001010 | 11 reg:3=d&7 rm:3=a&7 | is4:4=c 0000" "asm"="vblendvps {c}, {a}, {b}, {d}" "dclass"="yv" "feature"="avx2" "nodecode" {
  %r = select <8 x i1> %c, <8 x float> %a, <8 x float> %b
  ret <8 x float> %r
}

define <8 x float> @VFMADD231PS.fma.8xf32(<8 x float> "reg"="yv" %n, <8 x float> "reg"="yv" %m, <8 x float> "reg"="yv" "tied" %acc) "enc"="{ vex(d, 0, m, 2, 0, n, 1, 1) | 10111000 | 11 reg:3=d&7 rm:3=m&7 ; evexr(d, m, 2, 0, n, 1, 1, 0, 0, 0) | 10111000 | 11 reg:3=d&7 rm:3=m&7 }" "asm"="vfmadd231ps {m}, {n}, {d}" "dclass"="yv" "feature"="avx2,fma" "nodecode" {
  %r = call <8 x float> @llvm.fma.v8f32(<8 x float> %n, <8 x float> %m, <8 x float> %acc)
  ret <8 x float> %r
}

declare <8 x float> @llvm.fma.v8f32(<8 x float>, <8 x float>, <8 x float>)

define <8 x float> @VFMADD231PS.fmuladd.8xf32(<8 x float> "reg"="yv" %n, <8 x float> "reg"="yv" %m, <8 x float> "reg"="yv" "tied" %acc) "enc"="{ vex(d, 0, m, 2, 0, n, 1, 1) | 10111000 | 11 reg:3=d&7 rm:3=m&7 ; evexr(d, m, 2, 0, n, 1, 1, 0, 0, 0) | 10111000 | 11 reg:3=d&7 rm:3=m&7 }" "asm"="vfmadd231ps {m}, {n}, {d}" "dclass"="yv" "feature"="avx2,fma" "nodecode" {
  %r = call <8 x float> @llvm.fmuladd.v8f32(<8 x float> %n, <8 x float> %m, <8 x float> %acc)
  ret <8 x float> %r
}

declare <8 x float> @llvm.fmuladd.v8f32(<8 x float>, <8 x float>, <8 x float>)

define <4 x double> @VADDPD.4xf64(<4 x double> "reg"="yv" %n, <4 x double> "reg"="yv" %m) "enc"="{ if(m<8) vex(d, 0, m, 1, 0, n, 1, 1) | 01011000 | 11 reg:3=d&7 rm:3=m&7 ; if(n<8) vex(d, 0, n, 1, 0, m, 1, 1) | 01011000 | 11 reg:3=d&7 rm:3=n&7 ; vex(d, 0, m, 1, 0, n, 1, 1) | 01011000 | 11 reg:3=d&7 rm:3=m&7 ; evexr(d, m, 1, 1, n, 1, 1, 0, 0, 0) | 01011000 | 11 reg:3=d&7 rm:3=m&7 ; vex(d, 0, n, 1, 0, m, 1, 1) | 01011000 | 11 reg:3=d&7 rm:3=n&7 }" "asm"="vaddpd {m}, {n}, {d}" "dclass"="yv" "feature"="avx2" "nodecode" {
  %r = fadd <4 x double> %n, %m
  ret <4 x double> %r
}

define <4 x double> @VSUBPD.4xf64(<4 x double> "reg"="yv" %n, <4 x double> "reg"="yv" %m) "enc"="{ vex(d, 0, m, 1, 0, n, 1, 1) | 01011100 | 11 reg:3=d&7 rm:3=m&7 ; evexr(d, m, 1, 1, n, 1, 1, 0, 0, 0) | 01011100 | 11 reg:3=d&7 rm:3=m&7 }" "asm"="vsubpd {m}, {n}, {d}" "dclass"="yv" "feature"="avx2" "nodecode" {
  %r = fsub <4 x double> %n, %m
  ret <4 x double> %r
}

define <4 x double> @VMULPD.4xf64(<4 x double> "reg"="yv" %n, <4 x double> "reg"="yv" %m) "enc"="{ if(m<8) vex(d, 0, m, 1, 0, n, 1, 1) | 01011001 | 11 reg:3=d&7 rm:3=m&7 ; if(n<8) vex(d, 0, n, 1, 0, m, 1, 1) | 01011001 | 11 reg:3=d&7 rm:3=n&7 ; vex(d, 0, m, 1, 0, n, 1, 1) | 01011001 | 11 reg:3=d&7 rm:3=m&7 ; evexr(d, m, 1, 1, n, 1, 1, 0, 0, 0) | 01011001 | 11 reg:3=d&7 rm:3=m&7 ; vex(d, 0, n, 1, 0, m, 1, 1) | 01011001 | 11 reg:3=d&7 rm:3=n&7 }" "asm"="vmulpd {m}, {n}, {d}" "dclass"="yv" "feature"="avx2" "nodecode" {
  %r = fmul <4 x double> %n, %m
  ret <4 x double> %r
}

define <4 x double> @VDIVPD.4xf64(<4 x double> "reg"="yv" %n, <4 x double> "reg"="yv" %m) "enc"="{ vex(d, 0, m, 1, 0, n, 1, 1) | 01011110 | 11 reg:3=d&7 rm:3=m&7 ; evexr(d, m, 1, 1, n, 1, 1, 0, 0, 0) | 01011110 | 11 reg:3=d&7 rm:3=m&7 }" "asm"="vdivpd {m}, {n}, {d}" "dclass"="yv" "feature"="avx2" "nodecode" {
  %r = fdiv <4 x double> %n, %m
  ret <4 x double> %r
}

define <4 x double> @VSQRTPD.4xf64(<4 x double> "reg"="yv" %n) "enc"="{ vex(d, 0, n, 1, 0, 0, 1, 1) | 01010001 | 11 reg:3=d&7 rm:3=n&7 ; evexr(d, n, 1, 1, 0, 1, 1, 0, 0, 0) | 01010001 | 11 reg:3=d&7 rm:3=n&7 }" "asm"="vsqrtpd {n}, {d}" "dclass"="yv" "feature"="avx2" "nodecode" {
  %r = call <4 x double> @llvm.sqrt.v4f64(<4 x double> %n)
  ret <4 x double> %r
}

declare <4 x double> @llvm.sqrt.v4f64(<4 x double>)

define <4 x i1> @VCMPOEQPD.4xf64(<4 x double> "reg"="yv" %n, <4 x double> "reg"="yv" %m) "enc"="{ if(m<8) vex(d, 0, m, 1, 0, n, 1, 1) | 11000010 | 11 reg:3=d&7 rm:3=m&7 | i1:8=0 ; if(n<8) if(((0&3)==0)|((0&3)==3)) vex(d, 0, n, 1, 0, m, 1, 1) | 11000010 | 11 reg:3=d&7 rm:3=n&7 | i1:8=0 ; vex(d, 0, m, 1, 0, n, 1, 1) | 11000010 | 11 reg:3=d&7 rm:3=m&7 | i1:8=0 ; if(((0&3)==0)|((0&3)==3)) vex(d, 0, n, 1, 0, m, 1, 1) | 11000010 | 11 reg:3=d&7 rm:3=n&7 | i1:8=0 }" "asm"="vcmppd $0, {m}, {n}, {d}" "dclass"="yv" "feature"="avx2" "nodecode" {
  %r = fcmp oeq <4 x double> %n, %m
  ret <4 x i1> %r
}

define <4 x i1> @VCMPOLTPD.4xf64(<4 x double> "reg"="yv" %n, <4 x double> "reg"="yv" %m) "enc"="{ if(m<8) vex(d, 0, m, 1, 0, n, 1, 1) | 11000010 | 11 reg:3=d&7 rm:3=m&7 | i1:8=1 ; if(n<8) if(((1&3)==0)|((1&3)==3)) vex(d, 0, n, 1, 0, m, 1, 1) | 11000010 | 11 reg:3=d&7 rm:3=n&7 | i1:8=1 ; vex(d, 0, m, 1, 0, n, 1, 1) | 11000010 | 11 reg:3=d&7 rm:3=m&7 | i1:8=1 ; if(((1&3)==0)|((1&3)==3)) vex(d, 0, n, 1, 0, m, 1, 1) | 11000010 | 11 reg:3=d&7 rm:3=n&7 | i1:8=1 }" "asm"="vcmppd $1, {m}, {n}, {d}" "dclass"="yv" "feature"="avx2" "nodecode" {
  %r = fcmp olt <4 x double> %n, %m
  ret <4 x i1> %r
}

define <4 x i1> @VCMPOLEPD.4xf64(<4 x double> "reg"="yv" %n, <4 x double> "reg"="yv" %m) "enc"="{ if(m<8) vex(d, 0, m, 1, 0, n, 1, 1) | 11000010 | 11 reg:3=d&7 rm:3=m&7 | i1:8=2 ; if(n<8) if(((2&3)==0)|((2&3)==3)) vex(d, 0, n, 1, 0, m, 1, 1) | 11000010 | 11 reg:3=d&7 rm:3=n&7 | i1:8=2 ; vex(d, 0, m, 1, 0, n, 1, 1) | 11000010 | 11 reg:3=d&7 rm:3=m&7 | i1:8=2 ; if(((2&3)==0)|((2&3)==3)) vex(d, 0, n, 1, 0, m, 1, 1) | 11000010 | 11 reg:3=d&7 rm:3=n&7 | i1:8=2 }" "asm"="vcmppd $2, {m}, {n}, {d}" "dclass"="yv" "feature"="avx2" "nodecode" {
  %r = fcmp ole <4 x double> %n, %m
  ret <4 x i1> %r
}

define <4 x i1> @VCMPUNOPD.4xf64(<4 x double> "reg"="yv" %n, <4 x double> "reg"="yv" %m) "enc"="{ if(m<8) vex(d, 0, m, 1, 0, n, 1, 1) | 11000010 | 11 reg:3=d&7 rm:3=m&7 | i1:8=3 ; if(n<8) if(((3&3)==0)|((3&3)==3)) vex(d, 0, n, 1, 0, m, 1, 1) | 11000010 | 11 reg:3=d&7 rm:3=n&7 | i1:8=3 ; vex(d, 0, m, 1, 0, n, 1, 1) | 11000010 | 11 reg:3=d&7 rm:3=m&7 | i1:8=3 ; if(((3&3)==0)|((3&3)==3)) vex(d, 0, n, 1, 0, m, 1, 1) | 11000010 | 11 reg:3=d&7 rm:3=n&7 | i1:8=3 }" "asm"="vcmppd $3, {m}, {n}, {d}" "dclass"="yv" "feature"="avx2" "nodecode" {
  %r = fcmp uno <4 x double> %n, %m
  ret <4 x i1> %r
}

define <4 x i1> @VCMPUNEPD.4xf64(<4 x double> "reg"="yv" %n, <4 x double> "reg"="yv" %m) "enc"="{ if(m<8) vex(d, 0, m, 1, 0, n, 1, 1) | 11000010 | 11 reg:3=d&7 rm:3=m&7 | i1:8=4 ; if(n<8) if(((4&3)==0)|((4&3)==3)) vex(d, 0, n, 1, 0, m, 1, 1) | 11000010 | 11 reg:3=d&7 rm:3=n&7 | i1:8=4 ; vex(d, 0, m, 1, 0, n, 1, 1) | 11000010 | 11 reg:3=d&7 rm:3=m&7 | i1:8=4 ; if(((4&3)==0)|((4&3)==3)) vex(d, 0, n, 1, 0, m, 1, 1) | 11000010 | 11 reg:3=d&7 rm:3=n&7 | i1:8=4 }" "asm"="vcmppd $4, {m}, {n}, {d}" "dclass"="yv" "feature"="avx2" "nodecode" {
  %r = fcmp une <4 x double> %n, %m
  ret <4 x i1> %r
}

define <4 x i1> @VCMPUGEPD.4xf64(<4 x double> "reg"="yv" %n, <4 x double> "reg"="yv" %m) "enc"="{ if(m<8) vex(d, 0, m, 1, 0, n, 1, 1) | 11000010 | 11 reg:3=d&7 rm:3=m&7 | i1:8=5 ; if(n<8) if(((5&3)==0)|((5&3)==3)) vex(d, 0, n, 1, 0, m, 1, 1) | 11000010 | 11 reg:3=d&7 rm:3=n&7 | i1:8=5 ; vex(d, 0, m, 1, 0, n, 1, 1) | 11000010 | 11 reg:3=d&7 rm:3=m&7 | i1:8=5 ; if(((5&3)==0)|((5&3)==3)) vex(d, 0, n, 1, 0, m, 1, 1) | 11000010 | 11 reg:3=d&7 rm:3=n&7 | i1:8=5 }" "asm"="vcmppd $5, {m}, {n}, {d}" "dclass"="yv" "feature"="avx2" "nodecode" {
  %r = fcmp uge <4 x double> %n, %m
  ret <4 x i1> %r
}

define <4 x i1> @VCMPUGTPD.4xf64(<4 x double> "reg"="yv" %n, <4 x double> "reg"="yv" %m) "enc"="{ if(m<8) vex(d, 0, m, 1, 0, n, 1, 1) | 11000010 | 11 reg:3=d&7 rm:3=m&7 | i1:8=6 ; if(n<8) if(((6&3)==0)|((6&3)==3)) vex(d, 0, n, 1, 0, m, 1, 1) | 11000010 | 11 reg:3=d&7 rm:3=n&7 | i1:8=6 ; vex(d, 0, m, 1, 0, n, 1, 1) | 11000010 | 11 reg:3=d&7 rm:3=m&7 | i1:8=6 ; if(((6&3)==0)|((6&3)==3)) vex(d, 0, n, 1, 0, m, 1, 1) | 11000010 | 11 reg:3=d&7 rm:3=n&7 | i1:8=6 }" "asm"="vcmppd $6, {m}, {n}, {d}" "dclass"="yv" "feature"="avx2" "nodecode" {
  %r = fcmp ugt <4 x double> %n, %m
  ret <4 x i1> %r
}

define <4 x i1> @VCMPORDPD.4xf64(<4 x double> "reg"="yv" %n, <4 x double> "reg"="yv" %m) "enc"="{ if(m<8) vex(d, 0, m, 1, 0, n, 1, 1) | 11000010 | 11 reg:3=d&7 rm:3=m&7 | i1:8=7 ; if(n<8) if(((7&3)==0)|((7&3)==3)) vex(d, 0, n, 1, 0, m, 1, 1) | 11000010 | 11 reg:3=d&7 rm:3=n&7 | i1:8=7 ; vex(d, 0, m, 1, 0, n, 1, 1) | 11000010 | 11 reg:3=d&7 rm:3=m&7 | i1:8=7 ; if(((7&3)==0)|((7&3)==3)) vex(d, 0, n, 1, 0, m, 1, 1) | 11000010 | 11 reg:3=d&7 rm:3=n&7 | i1:8=7 }" "asm"="vcmppd $7, {m}, {n}, {d}" "dclass"="yv" "feature"="avx2" "nodecode" {
  %r = fcmp ord <4 x double> %n, %m
  ret <4 x i1> %r
}

define <4 x i1> @VCMPUEQPD.4xf64(<4 x double> "reg"="yv" %n, <4 x double> "reg"="yv" %m) "enc"="{ if(m<8) vex(d, 0, m, 1, 0, n, 1, 1) | 11000010 | 11 reg:3=d&7 rm:3=m&7 | i1:8=8 ; if(n<8) if(((8&3)==0)|((8&3)==3)) vex(d, 0, n, 1, 0, m, 1, 1) | 11000010 | 11 reg:3=d&7 rm:3=n&7 | i1:8=8 ; vex(d, 0, m, 1, 0, n, 1, 1) | 11000010 | 11 reg:3=d&7 rm:3=m&7 | i1:8=8 ; if(((8&3)==0)|((8&3)==3)) vex(d, 0, n, 1, 0, m, 1, 1) | 11000010 | 11 reg:3=d&7 rm:3=n&7 | i1:8=8 }" "asm"="vcmppd $8, {m}, {n}, {d}" "dclass"="yv" "feature"="avx2" "nodecode" {
  %r = fcmp ueq <4 x double> %n, %m
  ret <4 x i1> %r
}

define <4 x i1> @VCMPULTPD.4xf64(<4 x double> "reg"="yv" %n, <4 x double> "reg"="yv" %m) "enc"="{ if(m<8) vex(d, 0, m, 1, 0, n, 1, 1) | 11000010 | 11 reg:3=d&7 rm:3=m&7 | i1:8=9 ; if(n<8) if(((9&3)==0)|((9&3)==3)) vex(d, 0, n, 1, 0, m, 1, 1) | 11000010 | 11 reg:3=d&7 rm:3=n&7 | i1:8=9 ; vex(d, 0, m, 1, 0, n, 1, 1) | 11000010 | 11 reg:3=d&7 rm:3=m&7 | i1:8=9 ; if(((9&3)==0)|((9&3)==3)) vex(d, 0, n, 1, 0, m, 1, 1) | 11000010 | 11 reg:3=d&7 rm:3=n&7 | i1:8=9 }" "asm"="vcmppd $9, {m}, {n}, {d}" "dclass"="yv" "feature"="avx2" "nodecode" {
  %r = fcmp ult <4 x double> %n, %m
  ret <4 x i1> %r
}

define <4 x i1> @VCMPULEPD.4xf64(<4 x double> "reg"="yv" %n, <4 x double> "reg"="yv" %m) "enc"="{ if(m<8) vex(d, 0, m, 1, 0, n, 1, 1) | 11000010 | 11 reg:3=d&7 rm:3=m&7 | i1:8=10 ; if(n<8) if(((10&3)==0)|((10&3)==3)) vex(d, 0, n, 1, 0, m, 1, 1) | 11000010 | 11 reg:3=d&7 rm:3=n&7 | i1:8=10 ; vex(d, 0, m, 1, 0, n, 1, 1) | 11000010 | 11 reg:3=d&7 rm:3=m&7 | i1:8=10 ; if(((10&3)==0)|((10&3)==3)) vex(d, 0, n, 1, 0, m, 1, 1) | 11000010 | 11 reg:3=d&7 rm:3=n&7 | i1:8=10 }" "asm"="vcmppd $10, {m}, {n}, {d}" "dclass"="yv" "feature"="avx2" "nodecode" {
  %r = fcmp ule <4 x double> %n, %m
  ret <4 x i1> %r
}

define <4 x i1> @VCMPONEPD.4xf64(<4 x double> "reg"="yv" %n, <4 x double> "reg"="yv" %m) "enc"="{ if(m<8) vex(d, 0, m, 1, 0, n, 1, 1) | 11000010 | 11 reg:3=d&7 rm:3=m&7 | i1:8=12 ; if(n<8) if(((12&3)==0)|((12&3)==3)) vex(d, 0, n, 1, 0, m, 1, 1) | 11000010 | 11 reg:3=d&7 rm:3=n&7 | i1:8=12 ; vex(d, 0, m, 1, 0, n, 1, 1) | 11000010 | 11 reg:3=d&7 rm:3=m&7 | i1:8=12 ; if(((12&3)==0)|((12&3)==3)) vex(d, 0, n, 1, 0, m, 1, 1) | 11000010 | 11 reg:3=d&7 rm:3=n&7 | i1:8=12 }" "asm"="vcmppd $12, {m}, {n}, {d}" "dclass"="yv" "feature"="avx2" "nodecode" {
  %r = fcmp one <4 x double> %n, %m
  ret <4 x i1> %r
}

define <4 x i1> @VCMPOGEPD.4xf64(<4 x double> "reg"="yv" %n, <4 x double> "reg"="yv" %m) "enc"="{ if(m<8) vex(d, 0, m, 1, 0, n, 1, 1) | 11000010 | 11 reg:3=d&7 rm:3=m&7 | i1:8=13 ; if(n<8) if(((13&3)==0)|((13&3)==3)) vex(d, 0, n, 1, 0, m, 1, 1) | 11000010 | 11 reg:3=d&7 rm:3=n&7 | i1:8=13 ; vex(d, 0, m, 1, 0, n, 1, 1) | 11000010 | 11 reg:3=d&7 rm:3=m&7 | i1:8=13 ; if(((13&3)==0)|((13&3)==3)) vex(d, 0, n, 1, 0, m, 1, 1) | 11000010 | 11 reg:3=d&7 rm:3=n&7 | i1:8=13 }" "asm"="vcmppd $13, {m}, {n}, {d}" "dclass"="yv" "feature"="avx2" "nodecode" {
  %r = fcmp oge <4 x double> %n, %m
  ret <4 x i1> %r
}

define <4 x i1> @VCMPOGTPD.4xf64(<4 x double> "reg"="yv" %n, <4 x double> "reg"="yv" %m) "enc"="{ if(m<8) vex(d, 0, m, 1, 0, n, 1, 1) | 11000010 | 11 reg:3=d&7 rm:3=m&7 | i1:8=14 ; if(n<8) if(((14&3)==0)|((14&3)==3)) vex(d, 0, n, 1, 0, m, 1, 1) | 11000010 | 11 reg:3=d&7 rm:3=n&7 | i1:8=14 ; vex(d, 0, m, 1, 0, n, 1, 1) | 11000010 | 11 reg:3=d&7 rm:3=m&7 | i1:8=14 ; if(((14&3)==0)|((14&3)==3)) vex(d, 0, n, 1, 0, m, 1, 1) | 11000010 | 11 reg:3=d&7 rm:3=n&7 | i1:8=14 }" "asm"="vcmppd $14, {m}, {n}, {d}" "dclass"="yv" "feature"="avx2" "nodecode" {
  %r = fcmp ogt <4 x double> %n, %m
  ret <4 x i1> %r
}

define <4 x double> @VBLENDVPD.4xf64(<4 x i1> "reg"="yv" %c, <4 x double> "reg"="yv" %a, <4 x double> "reg"="yv" %b) "enc"="vex(d, 0, a, 3, 0, b, 1, 1) | 01001011 | 11 reg:3=d&7 rm:3=a&7 | is4:4=c 0000" "asm"="vblendvpd {c}, {a}, {b}, {d}" "dclass"="yv" "feature"="avx2" "nodecode" {
  %r = select <4 x i1> %c, <4 x double> %a, <4 x double> %b
  ret <4 x double> %r
}

define <4 x double> @VFMADD231PD.fma.4xf64(<4 x double> "reg"="yv" %n, <4 x double> "reg"="yv" %m, <4 x double> "reg"="yv" "tied" %acc) "enc"="{ vex(d, 0, m, 2, 1, n, 1, 1) | 10111000 | 11 reg:3=d&7 rm:3=m&7 ; evexr(d, m, 2, 1, n, 1, 1, 0, 0, 0) | 10111000 | 11 reg:3=d&7 rm:3=m&7 }" "asm"="vfmadd231pd {m}, {n}, {d}" "dclass"="yv" "feature"="avx2,fma" "nodecode" {
  %r = call <4 x double> @llvm.fma.v4f64(<4 x double> %n, <4 x double> %m, <4 x double> %acc)
  ret <4 x double> %r
}

declare <4 x double> @llvm.fma.v4f64(<4 x double>, <4 x double>, <4 x double>)

define <4 x double> @VFMADD231PD.fmuladd.4xf64(<4 x double> "reg"="yv" %n, <4 x double> "reg"="yv" %m, <4 x double> "reg"="yv" "tied" %acc) "enc"="{ vex(d, 0, m, 2, 1, n, 1, 1) | 10111000 | 11 reg:3=d&7 rm:3=m&7 ; evexr(d, m, 2, 1, n, 1, 1, 0, 0, 0) | 10111000 | 11 reg:3=d&7 rm:3=m&7 }" "asm"="vfmadd231pd {m}, {n}, {d}" "dclass"="yv" "feature"="avx2,fma" "nodecode" {
  %r = call <4 x double> @llvm.fmuladd.v4f64(<4 x double> %n, <4 x double> %m, <4 x double> %acc)
  ret <4 x double> %r
}

declare <4 x double> @llvm.fmuladd.v4f64(<4 x double>, <4 x double>, <4 x double>)

define <8 x float> @VCVTDQ2PS.8xf32(<8 x i32> "reg"="yv" %n) "enc"="{ vex(d, 0, n, 1, 0, 0, 1, 0) | 01011011 | 11 reg:3=d&7 rm:3=n&7 ; evexr(d, n, 1, 0, 0, 1, 0, 0, 0, 0) | 01011011 | 11 reg:3=d&7 rm:3=n&7 }" "asm"="vcvtdq2ps {n}, {d}" "dclass"="yv" "feature"="avx2" "nodecode" {
  %r = sitofp <8 x i32> %n to <8 x float>
  ret <8 x float> %r
}

define <8 x i32> @VCVTTPS2DQ.8xi32(<8 x float> "reg"="yv" %n) "enc"="{ vex(d, 0, n, 1, 0, 0, 1, 2) | 01011011 | 11 reg:3=d&7 rm:3=n&7 ; evexr(d, n, 1, 0, 0, 1, 2, 0, 0, 0) | 01011011 | 11 reg:3=d&7 rm:3=n&7 }" "asm"="vcvttps2dq {n}, {d}" "dclass"="yv" "feature"="avx2" "nodecode" {
  %r = fptosi <8 x float> %n to <8 x i32>
  ret <8 x i32> %r
}

define <4 x double> @VCVTPS2PD.4xf64(<4 x float> "reg"="xv" %n) "enc"="{ vex(d, 0, n, 1, 0, 0, 1, 0) | 01011010 | 11 reg:3=d&7 rm:3=n&7 ; evexr(d, n, 1, 0, 0, 1, 0, 0, 0, 0) | 01011010 | 11 reg:3=d&7 rm:3=n&7 }" "asm"="vcvtps2pd {n}, {d}" "dclass"="yv" "feature"="avx2" "nodecode" {
  %r = fpext <4 x float> %n to <4 x double>
  ret <4 x double> %r
}

define <4 x double> @VCVTDQ2PD.4xf64(<4 x i32> "reg"="xv" %n) "enc"="{ vex(d, 0, n, 1, 0, 0, 1, 2) | 11100110 | 11 reg:3=d&7 rm:3=n&7 ; evexr(d, n, 1, 0, 0, 1, 2, 0, 0, 0) | 11100110 | 11 reg:3=d&7 rm:3=n&7 }" "asm"="vcvtdq2pd {n}, {d}" "dclass"="yv" "feature"="avx2" "nodecode" {
  %r = sitofp <4 x i32> %n to <4 x double>
  ret <4 x double> %r
}

define <16 x i8> @VLOW.32xi8(<32 x i8> "reg"="yv" %v) "enc"="rex(0, d, 0, v) | 00001111 | 00101000 | rr(d, v)" "asm"="movaps {v:x}, {d}" "dclass"="xv" "feature"="avx2" "nodecode" {
  %r = shufflevector <32 x i8> %v, <32 x i8> poison, <16 x i32> <i32 0, i32 1, i32 2, i32 3, i32 4, i32 5, i32 6, i32 7, i32 8, i32 9, i32 10, i32 11, i32 12, i32 13, i32 14, i32 15>
  ret <16 x i8> %r
}

define <16 x i8> @VHIGH.32xi8(<32 x i8> "reg"="yv" %v) "enc"="vex(v, 0, d, 3, 0, 0, 1, 1) | 00111001 | 11 reg:3=v&7 rm:3=d&7 | 00000001" "asm"="vextracti128 $1, {v}, {d}" "dclass"="xv" "feature"="avx2" "nodecode" {
  %r = shufflevector <32 x i8> %v, <32 x i8> poison, <16 x i32> <i32 16, i32 17, i32 18, i32 19, i32 20, i32 21, i32 22, i32 23, i32 24, i32 25, i32 26, i32 27, i32 28, i32 29, i32 30, i32 31>
  ret <16 x i8> %r
}

define <32 x i8> @VJOIN.32xi8(<16 x i8> "reg"="xv" %a, <16 x i8> "reg"="xv" %b) "enc"="vex(d, 0, b, 3, 0, a, 1, 1) | 00111000 | 11 reg:3=d&7 rm:3=b&7 | 00000001" "asm"="vinserti128 $1, {b}, {a:y}, {d}" "dclass"="yv" "feature"="avx2" "nodecode" {
  %r = shufflevector <16 x i8> %a, <16 x i8> %b, <32 x i32> <i32 0, i32 1, i32 2, i32 3, i32 4, i32 5, i32 6, i32 7, i32 8, i32 9, i32 10, i32 11, i32 12, i32 13, i32 14, i32 15, i32 16, i32 17, i32 18, i32 19, i32 20, i32 21, i32 22, i32 23, i32 24, i32 25, i32 26, i32 27, i32 28, i32 29, i32 30, i32 31>
  ret <32 x i8> %r
}

define <8 x i16> @VLOW.16xi16(<16 x i16> "reg"="yv" %v) "enc"="rex(0, d, 0, v) | 00001111 | 00101000 | rr(d, v)" "asm"="movaps {v:x}, {d}" "dclass"="xv" "feature"="avx2" "nodecode" {
  %r = shufflevector <16 x i16> %v, <16 x i16> poison, <8 x i32> <i32 0, i32 1, i32 2, i32 3, i32 4, i32 5, i32 6, i32 7>
  ret <8 x i16> %r
}

define <8 x i16> @VHIGH.16xi16(<16 x i16> "reg"="yv" %v) "enc"="vex(v, 0, d, 3, 0, 0, 1, 1) | 00111001 | 11 reg:3=v&7 rm:3=d&7 | 00000001" "asm"="vextracti128 $1, {v}, {d}" "dclass"="xv" "feature"="avx2" "nodecode" {
  %r = shufflevector <16 x i16> %v, <16 x i16> poison, <8 x i32> <i32 8, i32 9, i32 10, i32 11, i32 12, i32 13, i32 14, i32 15>
  ret <8 x i16> %r
}

define <16 x i16> @VJOIN.16xi16(<8 x i16> "reg"="xv" %a, <8 x i16> "reg"="xv" %b) "enc"="vex(d, 0, b, 3, 0, a, 1, 1) | 00111000 | 11 reg:3=d&7 rm:3=b&7 | 00000001" "asm"="vinserti128 $1, {b}, {a:y}, {d}" "dclass"="yv" "feature"="avx2" "nodecode" {
  %r = shufflevector <8 x i16> %a, <8 x i16> %b, <16 x i32> <i32 0, i32 1, i32 2, i32 3, i32 4, i32 5, i32 6, i32 7, i32 8, i32 9, i32 10, i32 11, i32 12, i32 13, i32 14, i32 15>
  ret <16 x i16> %r
}

define <4 x i32> @VLOW.8xi32(<8 x i32> "reg"="yv" %v) "enc"="rex(0, d, 0, v) | 00001111 | 00101000 | rr(d, v)" "asm"="movaps {v:x}, {d}" "dclass"="xv" "feature"="avx2" "nodecode" {
  %r = shufflevector <8 x i32> %v, <8 x i32> poison, <4 x i32> <i32 0, i32 1, i32 2, i32 3>
  ret <4 x i32> %r
}

define <4 x i32> @VHIGH.8xi32(<8 x i32> "reg"="yv" %v) "enc"="vex(v, 0, d, 3, 0, 0, 1, 1) | 00111001 | 11 reg:3=v&7 rm:3=d&7 | 00000001" "asm"="vextracti128 $1, {v}, {d}" "dclass"="xv" "feature"="avx2" "nodecode" {
  %r = shufflevector <8 x i32> %v, <8 x i32> poison, <4 x i32> <i32 4, i32 5, i32 6, i32 7>
  ret <4 x i32> %r
}

define <8 x i32> @VJOIN.8xi32(<4 x i32> "reg"="xv" %a, <4 x i32> "reg"="xv" %b) "enc"="vex(d, 0, b, 3, 0, a, 1, 1) | 00111000 | 11 reg:3=d&7 rm:3=b&7 | 00000001" "asm"="vinserti128 $1, {b}, {a:y}, {d}" "dclass"="yv" "feature"="avx2" "nodecode" {
  %r = shufflevector <4 x i32> %a, <4 x i32> %b, <8 x i32> <i32 0, i32 1, i32 2, i32 3, i32 4, i32 5, i32 6, i32 7>
  ret <8 x i32> %r
}

define <2 x i64> @VLOW.4xi64(<4 x i64> "reg"="yv" %v) "enc"="rex(0, d, 0, v) | 00001111 | 00101000 | rr(d, v)" "asm"="movaps {v:x}, {d}" "dclass"="xv" "feature"="avx2" "nodecode" {
  %r = shufflevector <4 x i64> %v, <4 x i64> poison, <2 x i32> <i32 0, i32 1>
  ret <2 x i64> %r
}

define <2 x i64> @VHIGH.4xi64(<4 x i64> "reg"="yv" %v) "enc"="vex(v, 0, d, 3, 0, 0, 1, 1) | 00111001 | 11 reg:3=v&7 rm:3=d&7 | 00000001" "asm"="vextracti128 $1, {v}, {d}" "dclass"="xv" "feature"="avx2" "nodecode" {
  %r = shufflevector <4 x i64> %v, <4 x i64> poison, <2 x i32> <i32 2, i32 3>
  ret <2 x i64> %r
}

define <4 x i64> @VJOIN.4xi64(<2 x i64> "reg"="xv" %a, <2 x i64> "reg"="xv" %b) "enc"="vex(d, 0, b, 3, 0, a, 1, 1) | 00111000 | 11 reg:3=d&7 rm:3=b&7 | 00000001" "asm"="vinserti128 $1, {b}, {a:y}, {d}" "dclass"="yv" "feature"="avx2" "nodecode" {
  %r = shufflevector <2 x i64> %a, <2 x i64> %b, <4 x i32> <i32 0, i32 1, i32 2, i32 3>
  ret <4 x i64> %r
}

define <4 x float> @VLOW.8xf32(<8 x float> "reg"="yv" %v) "enc"="rex(0, d, 0, v) | 00001111 | 00101000 | rr(d, v)" "asm"="movaps {v:x}, {d}" "dclass"="xv" "feature"="avx2" "nodecode" {
  %r = shufflevector <8 x float> %v, <8 x float> poison, <4 x i32> <i32 0, i32 1, i32 2, i32 3>
  ret <4 x float> %r
}

define <4 x float> @VHIGH.8xf32(<8 x float> "reg"="yv" %v) "enc"="vex(v, 0, d, 3, 0, 0, 1, 1) | 00111001 | 11 reg:3=v&7 rm:3=d&7 | 00000001" "asm"="vextracti128 $1, {v}, {d}" "dclass"="xv" "feature"="avx2" "nodecode" {
  %r = shufflevector <8 x float> %v, <8 x float> poison, <4 x i32> <i32 4, i32 5, i32 6, i32 7>
  ret <4 x float> %r
}

define <8 x float> @VJOIN.8xf32(<4 x float> "reg"="xv" %a, <4 x float> "reg"="xv" %b) "enc"="vex(d, 0, b, 3, 0, a, 1, 1) | 00111000 | 11 reg:3=d&7 rm:3=b&7 | 00000001" "asm"="vinserti128 $1, {b}, {a:y}, {d}" "dclass"="yv" "feature"="avx2" "nodecode" {
  %r = shufflevector <4 x float> %a, <4 x float> %b, <8 x i32> <i32 0, i32 1, i32 2, i32 3, i32 4, i32 5, i32 6, i32 7>
  ret <8 x float> %r
}

define <2 x double> @VLOW.4xf64(<4 x double> "reg"="yv" %v) "enc"="rex(0, d, 0, v) | 00001111 | 00101000 | rr(d, v)" "asm"="movaps {v:x}, {d}" "dclass"="xv" "feature"="avx2" "nodecode" {
  %r = shufflevector <4 x double> %v, <4 x double> poison, <2 x i32> <i32 0, i32 1>
  ret <2 x double> %r
}

define <2 x double> @VHIGH.4xf64(<4 x double> "reg"="yv" %v) "enc"="vex(v, 0, d, 3, 0, 0, 1, 1) | 00111001 | 11 reg:3=v&7 rm:3=d&7 | 00000001" "asm"="vextracti128 $1, {v}, {d}" "dclass"="xv" "feature"="avx2" "nodecode" {
  %r = shufflevector <4 x double> %v, <4 x double> poison, <2 x i32> <i32 2, i32 3>
  ret <2 x double> %r
}

define <4 x double> @VJOIN.4xf64(<2 x double> "reg"="xv" %a, <2 x double> "reg"="xv" %b) "enc"="vex(d, 0, b, 3, 0, a, 1, 1) | 00111000 | 11 reg:3=d&7 rm:3=b&7 | 00000001" "asm"="vinserti128 $1, {b}, {a:y}, {d}" "dclass"="yv" "feature"="avx2" "nodecode" {
  %r = shufflevector <2 x double> %a, <2 x double> %b, <4 x i32> <i32 0, i32 1, i32 2, i32 3>
  ret <4 x double> %r
}

define <16 x i16> @VPMOVSXBW(<16 x i8> "reg"="xv" %n) "enc"="{ vex(d, 0, n, 2, 0, 0, 1, 1) | 00100000 | 11 reg:3=d&7 rm:3=n&7 ; evexr(d, n, 2, 0, 0, 1, 1, 0, 0, 0) | 00100000 | 11 reg:3=d&7 rm:3=n&7 }" "asm"="vpmovsxbw {n}, {d}" "dclass"="yv" "feature"="avx2" "nodecode" {
  %r = sext <16 x i8> %n to <16 x i16>
  ret <16 x i16> %r
}

define <16 x i16> @VPMOVZXBW(<16 x i8> "reg"="xv" %n) "enc"="{ vex(d, 0, n, 2, 0, 0, 1, 1) | 00110000 | 11 reg:3=d&7 rm:3=n&7 ; evexr(d, n, 2, 0, 0, 1, 1, 0, 0, 0) | 00110000 | 11 reg:3=d&7 rm:3=n&7 }" "asm"="vpmovzxbw {n}, {d}" "dclass"="yv" "feature"="avx2" "nodecode" {
  %r = zext <16 x i8> %n to <16 x i16>
  ret <16 x i16> %r
}

define <8 x i32> @VPMOVSXWD(<8 x i16> "reg"="xv" %n) "enc"="{ vex(d, 0, n, 2, 0, 0, 1, 1) | 00100011 | 11 reg:3=d&7 rm:3=n&7 ; evexr(d, n, 2, 0, 0, 1, 1, 0, 0, 0) | 00100011 | 11 reg:3=d&7 rm:3=n&7 }" "asm"="vpmovsxwd {n}, {d}" "dclass"="yv" "feature"="avx2" "nodecode" {
  %r = sext <8 x i16> %n to <8 x i32>
  ret <8 x i32> %r
}

define <8 x i32> @VPMOVZXWD(<8 x i16> "reg"="xv" %n) "enc"="{ vex(d, 0, n, 2, 0, 0, 1, 1) | 00110011 | 11 reg:3=d&7 rm:3=n&7 ; evexr(d, n, 2, 0, 0, 1, 1, 0, 0, 0) | 00110011 | 11 reg:3=d&7 rm:3=n&7 }" "asm"="vpmovzxwd {n}, {d}" "dclass"="yv" "feature"="avx2" "nodecode" {
  %r = zext <8 x i16> %n to <8 x i32>
  ret <8 x i32> %r
}

define <4 x i64> @VPMOVSXDQ(<4 x i32> "reg"="xv" %n) "enc"="{ vex(d, 0, n, 2, 0, 0, 1, 1) | 00100101 | 11 reg:3=d&7 rm:3=n&7 ; evexr(d, n, 2, 0, 0, 1, 1, 0, 0, 0) | 00100101 | 11 reg:3=d&7 rm:3=n&7 }" "asm"="vpmovsxdq {n}, {d}" "dclass"="yv" "feature"="avx2" "nodecode" {
  %r = sext <4 x i32> %n to <4 x i64>
  ret <4 x i64> %r
}

define <4 x i64> @VPMOVZXDQ(<4 x i32> "reg"="xv" %n) "enc"="{ vex(d, 0, n, 2, 0, 0, 1, 1) | 00110101 | 11 reg:3=d&7 rm:3=n&7 ; evexr(d, n, 2, 0, 0, 1, 1, 0, 0, 0) | 00110101 | 11 reg:3=d&7 rm:3=n&7 }" "asm"="vpmovzxdq {n}, {d}" "dclass"="yv" "feature"="avx2" "nodecode" {
  %r = zext <4 x i32> %n to <4 x i64>
  ret <4 x i64> %r
}

define <32 x i8> @VBITS.32x8(<32 x i1> "reg"="yv" %m) "enc"="{ if(m<8) vex(d, 0, m, 1, 0, 0, 1, 0) | 00101000 | 11 reg:3=d&7 rm:3=m&7 ; if(d<8) vex(m, 0, d, 1, 0, 0, 1, 0) | 00101001 | 11 reg:3=m&7 rm:3=d&7 ; vex(d, 0, m, 1, 0, 0, 1, 0) | 00101000 | 11 reg:3=d&7 rm:3=m&7 ; vex(m, 0, d, 1, 0, 0, 1, 0) | 00101001 | 11 reg:3=m&7 rm:3=d&7 ; evexr(d, m, 1, 0, 0, 1, 0, 0, 0, 0) | 00101000 | 11 reg:3=d&7 rm:3=m&7 ; evexr(m, d, 1, 0, 0, 1, 0, 0, 0, 0) | 00101001 | 11 reg:3=m&7 rm:3=d&7 }" "asm"="vmovaps {m}, {d}" "dclass"="yv" "feature"="avx2" "nodecode" {
  %r = call <32 x i8> @llvm.epitome.mask.bits.v32i8(<32 x i1> %m)
  ret <32 x i8> %r
}

define <32 x i1> @VMASK.32x8(<32 x i8> "reg"="yv" %v) "enc"="{ if(v<8) vex(d, 0, v, 1, 0, 0, 1, 0) | 00101000 | 11 reg:3=d&7 rm:3=v&7 ; if(d<8) vex(v, 0, d, 1, 0, 0, 1, 0) | 00101001 | 11 reg:3=v&7 rm:3=d&7 ; vex(d, 0, v, 1, 0, 0, 1, 0) | 00101000 | 11 reg:3=d&7 rm:3=v&7 ; vex(v, 0, d, 1, 0, 0, 1, 0) | 00101001 | 11 reg:3=v&7 rm:3=d&7 ; evexr(d, v, 1, 0, 0, 1, 0, 0, 0, 0) | 00101000 | 11 reg:3=d&7 rm:3=v&7 ; evexr(v, d, 1, 0, 0, 1, 0, 0, 0, 0) | 00101001 | 11 reg:3=v&7 rm:3=d&7 }" "asm"="vmovaps {v}, {d}" "dclass"="yv" "feature"="avx2" "nodecode" {
  %r = call <32 x i1> @llvm.epitome.bits.mask.v32i8(<32 x i8> %v)
  ret <32 x i1> %r
}

declare <32 x i8> @llvm.epitome.mask.bits.v32i8(<32 x i1>)
declare <32 x i1> @llvm.epitome.bits.mask.v32i8(<32 x i8>)

define <16 x i16> @VBITS.16x16(<16 x i1> "reg"="yv" %m) "enc"="{ if(m<8) vex(d, 0, m, 1, 0, 0, 1, 0) | 00101000 | 11 reg:3=d&7 rm:3=m&7 ; if(d<8) vex(m, 0, d, 1, 0, 0, 1, 0) | 00101001 | 11 reg:3=m&7 rm:3=d&7 ; vex(d, 0, m, 1, 0, 0, 1, 0) | 00101000 | 11 reg:3=d&7 rm:3=m&7 ; vex(m, 0, d, 1, 0, 0, 1, 0) | 00101001 | 11 reg:3=m&7 rm:3=d&7 ; evexr(d, m, 1, 0, 0, 1, 0, 0, 0, 0) | 00101000 | 11 reg:3=d&7 rm:3=m&7 ; evexr(m, d, 1, 0, 0, 1, 0, 0, 0, 0) | 00101001 | 11 reg:3=m&7 rm:3=d&7 }" "asm"="vmovaps {m}, {d}" "dclass"="yv" "feature"="avx2" "nodecode" {
  %r = call <16 x i16> @llvm.epitome.mask.bits.v16i16(<16 x i1> %m)
  ret <16 x i16> %r
}

define <16 x i1> @VMASK.16x16(<16 x i16> "reg"="yv" %v) "enc"="{ if(v<8) vex(d, 0, v, 1, 0, 0, 1, 0) | 00101000 | 11 reg:3=d&7 rm:3=v&7 ; if(d<8) vex(v, 0, d, 1, 0, 0, 1, 0) | 00101001 | 11 reg:3=v&7 rm:3=d&7 ; vex(d, 0, v, 1, 0, 0, 1, 0) | 00101000 | 11 reg:3=d&7 rm:3=v&7 ; vex(v, 0, d, 1, 0, 0, 1, 0) | 00101001 | 11 reg:3=v&7 rm:3=d&7 ; evexr(d, v, 1, 0, 0, 1, 0, 0, 0, 0) | 00101000 | 11 reg:3=d&7 rm:3=v&7 ; evexr(v, d, 1, 0, 0, 1, 0, 0, 0, 0) | 00101001 | 11 reg:3=v&7 rm:3=d&7 }" "asm"="vmovaps {v}, {d}" "dclass"="yv" "feature"="avx2" "nodecode" {
  %r = call <16 x i1> @llvm.epitome.bits.mask.v16i16(<16 x i16> %v)
  ret <16 x i1> %r
}

declare <16 x i16> @llvm.epitome.mask.bits.v16i16(<16 x i1>)
declare <16 x i1> @llvm.epitome.bits.mask.v16i16(<16 x i16>)

define <8 x i32> @VBITS.8x32(<8 x i1> "reg"="yv" %m) "enc"="{ if(m<8) vex(d, 0, m, 1, 0, 0, 1, 0) | 00101000 | 11 reg:3=d&7 rm:3=m&7 ; if(d<8) vex(m, 0, d, 1, 0, 0, 1, 0) | 00101001 | 11 reg:3=m&7 rm:3=d&7 ; vex(d, 0, m, 1, 0, 0, 1, 0) | 00101000 | 11 reg:3=d&7 rm:3=m&7 ; vex(m, 0, d, 1, 0, 0, 1, 0) | 00101001 | 11 reg:3=m&7 rm:3=d&7 ; evexr(d, m, 1, 0, 0, 1, 0, 0, 0, 0) | 00101000 | 11 reg:3=d&7 rm:3=m&7 ; evexr(m, d, 1, 0, 0, 1, 0, 0, 0, 0) | 00101001 | 11 reg:3=m&7 rm:3=d&7 }" "asm"="vmovaps {m}, {d}" "dclass"="yv" "feature"="avx2" "nodecode" {
  %r = call <8 x i32> @llvm.epitome.mask.bits.v8i32(<8 x i1> %m)
  ret <8 x i32> %r
}

define <8 x i1> @VMASK.8x32(<8 x i32> "reg"="yv" %v) "enc"="{ if(v<8) vex(d, 0, v, 1, 0, 0, 1, 0) | 00101000 | 11 reg:3=d&7 rm:3=v&7 ; if(d<8) vex(v, 0, d, 1, 0, 0, 1, 0) | 00101001 | 11 reg:3=v&7 rm:3=d&7 ; vex(d, 0, v, 1, 0, 0, 1, 0) | 00101000 | 11 reg:3=d&7 rm:3=v&7 ; vex(v, 0, d, 1, 0, 0, 1, 0) | 00101001 | 11 reg:3=v&7 rm:3=d&7 ; evexr(d, v, 1, 0, 0, 1, 0, 0, 0, 0) | 00101000 | 11 reg:3=d&7 rm:3=v&7 ; evexr(v, d, 1, 0, 0, 1, 0, 0, 0, 0) | 00101001 | 11 reg:3=v&7 rm:3=d&7 }" "asm"="vmovaps {v}, {d}" "dclass"="yv" "feature"="avx2" "nodecode" {
  %r = call <8 x i1> @llvm.epitome.bits.mask.v8i32(<8 x i32> %v)
  ret <8 x i1> %r
}

declare <8 x i32> @llvm.epitome.mask.bits.v8i32(<8 x i1>)
declare <8 x i1> @llvm.epitome.bits.mask.v8i32(<8 x i32>)

define <4 x i64> @VBITS.4x64(<4 x i1> "reg"="yv" %m) "enc"="{ if(m<8) vex(d, 0, m, 1, 0, 0, 1, 0) | 00101000 | 11 reg:3=d&7 rm:3=m&7 ; if(d<8) vex(m, 0, d, 1, 0, 0, 1, 0) | 00101001 | 11 reg:3=m&7 rm:3=d&7 ; vex(d, 0, m, 1, 0, 0, 1, 0) | 00101000 | 11 reg:3=d&7 rm:3=m&7 ; vex(m, 0, d, 1, 0, 0, 1, 0) | 00101001 | 11 reg:3=m&7 rm:3=d&7 ; evexr(d, m, 1, 0, 0, 1, 0, 0, 0, 0) | 00101000 | 11 reg:3=d&7 rm:3=m&7 ; evexr(m, d, 1, 0, 0, 1, 0, 0, 0, 0) | 00101001 | 11 reg:3=m&7 rm:3=d&7 }" "asm"="vmovaps {m}, {d}" "dclass"="yv" "feature"="avx2" "nodecode" {
  %r = call <4 x i64> @llvm.epitome.mask.bits.v4i64(<4 x i1> %m)
  ret <4 x i64> %r
}

define <4 x i1> @VMASK.4x64(<4 x i64> "reg"="yv" %v) "enc"="{ if(v<8) vex(d, 0, v, 1, 0, 0, 1, 0) | 00101000 | 11 reg:3=d&7 rm:3=v&7 ; if(d<8) vex(v, 0, d, 1, 0, 0, 1, 0) | 00101001 | 11 reg:3=v&7 rm:3=d&7 ; vex(d, 0, v, 1, 0, 0, 1, 0) | 00101000 | 11 reg:3=d&7 rm:3=v&7 ; vex(v, 0, d, 1, 0, 0, 1, 0) | 00101001 | 11 reg:3=v&7 rm:3=d&7 ; evexr(d, v, 1, 0, 0, 1, 0, 0, 0, 0) | 00101000 | 11 reg:3=d&7 rm:3=v&7 ; evexr(v, d, 1, 0, 0, 1, 0, 0, 0, 0) | 00101001 | 11 reg:3=v&7 rm:3=d&7 }" "asm"="vmovaps {v}, {d}" "dclass"="yv" "feature"="avx2" "nodecode" {
  %r = call <4 x i1> @llvm.epitome.bits.mask.v4i64(<4 x i64> %v)
  ret <4 x i1> %r
}

declare <4 x i64> @llvm.epitome.mask.bits.v4i64(<4 x i1>)
declare <4 x i1> @llvm.epitome.bits.mask.v4i64(<4 x i64>)

define <8 x i32> @VPBROADCASTD(<2 x i64> "reg"="xv" %n) "enc"="{ vex(d, 0, n, 2, 0, 0, 1, 1) | 01011000 | 11 reg:3=d&7 rm:3=n&7 ; evexr(d, n, 2, 0, 0, 1, 1, 0, 0, 0) | 01011000 | 11 reg:3=d&7 rm:3=n&7 }" "asm"="vpbroadcastd {n}, {d}" "dclass"="yv" "nomatch" "feature"="avx2" "nodecode" {
  ret <8 x i32> poison
}

define <8 x i32> @vsplat.8xi32(i32 "reg"="r32" %x) "recipe"="vsplat.8xi32.expand" "feature"="avx2" {
  %i = insertelement <8 x i32> poison, i32 %x, i64 0
  %r = shufflevector <8 x i32> %i, <8 x i32> poison, <8 x i32> zeroinitializer
  ret <8 x i32> %r
}

define <8 x i32> @vsplat.8xi32.expand(i32 %x) {
  %v4 = call <4 x i32> @MOVDto(i32 %x)
  %v = bitcast <4 x i32> %v4 to <2 x i64>
  %r = call <8 x i32> @VPBROADCASTD(<2 x i64> %v)
  ret <8 x i32> %r
}

define <4 x i64> @VPBROADCASTQ(<2 x i64> "reg"="xv" %n) "enc"="{ vex(d, 0, n, 2, 0, 0, 1, 1) | 01011001 | 11 reg:3=d&7 rm:3=n&7 ; evexr(d, n, 2, 1, 0, 1, 1, 0, 0, 0) | 01011001 | 11 reg:3=d&7 rm:3=n&7 }" "asm"="vpbroadcastq {n}, {d}" "dclass"="yv" "nomatch" "feature"="avx2" "nodecode" {
  ret <4 x i64> poison
}

define <4 x i64> @vsplat.4xi64(i64 "reg"="r64" %x) "recipe"="vsplat.4xi64.expand" "feature"="avx2" {
  %i = insertelement <4 x i64> poison, i64 %x, i64 0
  %r = shufflevector <4 x i64> %i, <4 x i64> poison, <4 x i32> zeroinitializer
  ret <4 x i64> %r
}

define <4 x i64> @vsplat.4xi64.expand(i64 %x) {
  %v = call <2 x i64> @MOVQto(i64 %x)
  %r = call <4 x i64> @VPBROADCASTQ(<2 x i64> %v)
  ret <4 x i64> %r
}

define <16 x i16> @VPBROADCASTW(<2 x i64> "reg"="xv" %n) "enc"="{ vex(d, 0, n, 2, 0, 0, 1, 1) | 01111001 | 11 reg:3=d&7 rm:3=n&7 ; evexr(d, n, 2, 0, 0, 1, 1, 0, 0, 0) | 01111001 | 11 reg:3=d&7 rm:3=n&7 }" "asm"="vpbroadcastw {n}, {d}" "dclass"="yv" "nomatch" "feature"="avx2" "nodecode" {
  ret <16 x i16> poison
}

define <16 x i16> @vsplat.16xi16(i16 "reg"="r32" %x) "recipe"="vsplat.16xi16.expand" "feature"="avx2" {
  %i = insertelement <16 x i16> poison, i16 %x, i64 0
  %r = shufflevector <16 x i16> %i, <16 x i16> poison, <16 x i32> zeroinitializer
  ret <16 x i16> %r
}

define <16 x i16> @vsplat.16xi16.expand(i16 %x) {
  %w = zext i16 %x to i32
  %v4 = call <4 x i32> @MOVDto(i32 %w)
  %v = bitcast <4 x i32> %v4 to <2 x i64>
  %r = call <16 x i16> @VPBROADCASTW(<2 x i64> %v)
  ret <16 x i16> %r
}

define <32 x i8> @VPBROADCASTB(<2 x i64> "reg"="xv" %n) "enc"="{ vex(d, 0, n, 2, 0, 0, 1, 1) | 01111000 | 11 reg:3=d&7 rm:3=n&7 ; evexr(d, n, 2, 0, 0, 1, 1, 0, 0, 0) | 01111000 | 11 reg:3=d&7 rm:3=n&7 }" "asm"="vpbroadcastb {n}, {d}" "dclass"="yv" "nomatch" "feature"="avx2" "nodecode" {
  ret <32 x i8> poison
}

define <32 x i8> @vsplat.32xi8(i8 "reg"="r32" %x) "recipe"="vsplat.32xi8.expand" "feature"="avx2" {
  %i = insertelement <32 x i8> poison, i8 %x, i64 0
  %r = shufflevector <32 x i8> %i, <32 x i8> poison, <32 x i32> zeroinitializer
  ret <32 x i8> %r
}

define <32 x i8> @vsplat.32xi8.expand(i8 %x) {
  %w = zext i8 %x to i32
  %v4 = call <4 x i32> @MOVDto(i32 %w)
  %v = bitcast <4 x i32> %v4 to <2 x i64>
  %r = call <32 x i8> @VPBROADCASTB(<2 x i64> %v)
  ret <32 x i8> %r
}

define <8 x float> @VBROADCASTSS(<2 x i64> "reg"="xv" %n) "enc"="{ vex(d, 0, n, 2, 0, 0, 1, 1) | 00011000 | 11 reg:3=d&7 rm:3=n&7 ; evexr(d, n, 2, 0, 0, 1, 1, 0, 0, 0) | 00011000 | 11 reg:3=d&7 rm:3=n&7 }" "asm"="vbroadcastss {n}, {d}" "dclass"="yv" "nomatch" "feature"="avx2" "nodecode" {
  ret <8 x float> poison
}

define <8 x float> @vsplat.8xf32(float "reg"="s" %x) "recipe"="vsplat.8xf32.expand" "feature"="avx2" {
  %i = insertelement <8 x float> poison, float %x, i64 0
  %r = shufflevector <8 x float> %i, <8 x float> poison, <8 x i32> zeroinitializer
  ret <8 x float> %r
}

define <8 x float> @vsplat.8xf32.expand(float %x) {
  %v = bitcast float %x to <2 x i64>
  %r = call <8 x float> @VBROADCASTSS(<2 x i64> %v)
  ret <8 x float> %r
}

define <4 x double> @VBROADCASTSD(<2 x i64> "reg"="xv" %n) "enc"="{ vex(d, 0, n, 2, 0, 0, 1, 1) | 00011001 | 11 reg:3=d&7 rm:3=n&7 ; evexr(d, n, 2, 1, 0, 1, 1, 0, 0, 0) | 00011001 | 11 reg:3=d&7 rm:3=n&7 }" "asm"="vbroadcastsd {n}, {d}" "dclass"="yv" "nomatch" "feature"="avx2" "nodecode" {
  ret <4 x double> poison
}

define <4 x double> @vsplat.4xf64(double "reg"="d" %x) "recipe"="vsplat.4xf64.expand" "feature"="avx2" {
  %i = insertelement <4 x double> poison, double %x, i64 0
  %r = shufflevector <4 x double> %i, <4 x double> poison, <4 x i32> zeroinitializer
  ret <4 x double> %r
}

define <4 x double> @vsplat.4xf64.expand(double %x) {
  %v = bitcast double %x to <2 x i64>
  %r = call <4 x double> @VBROADCASTSD(<2 x i64> %v)
  ret <4 x double> %r
}

define <16 x float> @VLDZ.16xf32(ptr "reg"="r64" %n, i64 "imm" %off) "enc"="evexm(d, n, 4, 1, 0, 0, 2, 0, 0, 0, 0) | 00010000 | rm(d, n, 4, 1, off, 0, 64)" "asm"="vmovups {off}({n}), {d}" "dclass"="zv" "feature"="avx512f" "nodecode" {
  %a = getelementptr i8, ptr %n, i64 %off
  %r = load <16 x float>, ptr %a
  ret <16 x float> %r
}

define void @VSTZ.16xf32(<16 x float> "reg"="zv" %t, ptr "reg"="r64" %n, i64 "imm" %off) "enc"="evexm(t, n, 4, 1, 0, 0, 2, 0, 0, 0, 0) | 00010001 | rm(t, n, 4, 1, off, 0, 64)" "asm"="vmovups {t}, {off}({n})" "feature"="avx512f" "nodecode" {
  %a = getelementptr i8, ptr %n, i64 %off
  store <16 x float> %t, ptr %a
  ret void
}

define <16 x i32> @VLDZ.16xi32(ptr "reg"="r64" %n, i64 "imm" %off) "enc"="evexm(d, n, 4, 1, 0, 0, 2, 0, 0, 0, 0) | 00010000 | rm(d, n, 4, 1, off, 0, 64)" "asm"="vmovups {off}({n}), {d}" "dclass"="zv" "role"="load.zv" "feature"="avx512f" "nodecode" {
  %a = getelementptr i8, ptr %n, i64 %off
  %r = load <16 x i32>, ptr %a
  ret <16 x i32> %r
}

define void @VSTZ.16xi32(<16 x i32> "reg"="zv" %t, ptr "reg"="r64" %n, i64 "imm" %off) "enc"="evexm(t, n, 4, 1, 0, 0, 2, 0, 0, 0, 0) | 00010001 | rm(t, n, 4, 1, off, 0, 64)" "asm"="vmovups {t}, {off}({n})" "role"="store.zv" "feature"="avx512f" "nodecode" {
  %a = getelementptr i8, ptr %n, i64 %off
  store <16 x i32> %t, ptr %a
  ret void
}

define <64 x i8> @VLDZ.64xi8(ptr "reg"="r64" %n, i64 "imm" %off) "enc"="evexm(d, n, 4, 1, 0, 0, 2, 0, 0, 0, 0) | 00010000 | rm(d, n, 4, 1, off, 0, 64)" "asm"="vmovups {off}({n}), {d}" "dclass"="zv" "feature"="avx512f" "nodecode" {
  %a = getelementptr i8, ptr %n, i64 %off
  %r = load <64 x i8>, ptr %a
  ret <64 x i8> %r
}

define void @VSTZ.64xi8(<64 x i8> "reg"="zv" %t, ptr "reg"="r64" %n, i64 "imm" %off) "enc"="evexm(t, n, 4, 1, 0, 0, 2, 0, 0, 0, 0) | 00010001 | rm(t, n, 4, 1, off, 0, 64)" "asm"="vmovups {t}, {off}({n})" "feature"="avx512f" "nodecode" {
  %a = getelementptr i8, ptr %n, i64 %off
  store <64 x i8> %t, ptr %a
  ret void
}

define <32 x i16> @VLDZ.32xi16(ptr "reg"="r64" %n, i64 "imm" %off) "enc"="evexm(d, n, 4, 1, 0, 0, 2, 0, 0, 0, 0) | 00010000 | rm(d, n, 4, 1, off, 0, 64)" "asm"="vmovups {off}({n}), {d}" "dclass"="zv" "feature"="avx512f" "nodecode" {
  %a = getelementptr i8, ptr %n, i64 %off
  %r = load <32 x i16>, ptr %a
  ret <32 x i16> %r
}

define void @VSTZ.32xi16(<32 x i16> "reg"="zv" %t, ptr "reg"="r64" %n, i64 "imm" %off) "enc"="evexm(t, n, 4, 1, 0, 0, 2, 0, 0, 0, 0) | 00010001 | rm(t, n, 4, 1, off, 0, 64)" "asm"="vmovups {t}, {off}({n})" "feature"="avx512f" "nodecode" {
  %a = getelementptr i8, ptr %n, i64 %off
  store <32 x i16> %t, ptr %a
  ret void
}

define <8 x i64> @VLDZ.8xi64(ptr "reg"="r64" %n, i64 "imm" %off) "enc"="evexm(d, n, 4, 1, 0, 0, 2, 0, 0, 0, 0) | 00010000 | rm(d, n, 4, 1, off, 0, 64)" "asm"="vmovups {off}({n}), {d}" "dclass"="zv" "feature"="avx512f" "nodecode" {
  %a = getelementptr i8, ptr %n, i64 %off
  %r = load <8 x i64>, ptr %a
  ret <8 x i64> %r
}

define void @VSTZ.8xi64(<8 x i64> "reg"="zv" %t, ptr "reg"="r64" %n, i64 "imm" %off) "enc"="evexm(t, n, 4, 1, 0, 0, 2, 0, 0, 0, 0) | 00010001 | rm(t, n, 4, 1, off, 0, 64)" "asm"="vmovups {t}, {off}({n})" "feature"="avx512f" "nodecode" {
  %a = getelementptr i8, ptr %n, i64 %off
  store <8 x i64> %t, ptr %a
  ret void
}

define <8 x double> @VLDZ.8xf64(ptr "reg"="r64" %n, i64 "imm" %off) "enc"="evexm(d, n, 4, 1, 0, 0, 2, 0, 0, 0, 0) | 00010000 | rm(d, n, 4, 1, off, 0, 64)" "asm"="vmovups {off}({n}), {d}" "dclass"="zv" "feature"="avx512f" "nodecode" {
  %a = getelementptr i8, ptr %n, i64 %off
  %r = load <8 x double>, ptr %a
  ret <8 x double> %r
}

define void @VSTZ.8xf64(<8 x double> "reg"="zv" %t, ptr "reg"="r64" %n, i64 "imm" %off) "enc"="evexm(t, n, 4, 1, 0, 0, 2, 0, 0, 0, 0) | 00010001 | rm(t, n, 4, 1, off, 0, 64)" "asm"="vmovups {t}, {off}({n})" "feature"="avx512f" "nodecode" {
  %a = getelementptr i8, ptr %n, i64 %off
  store <8 x double> %t, ptr %a
  ret void
}

define <32 x half> @VLDZ.32xhalf(ptr "reg"="r64" %n, i64 "imm" %off) "enc"="evexm(d, n, 4, 1, 0, 0, 2, 0, 0, 0, 0) | 00010000 | rm(d, n, 4, 1, off, 0, 64)" "asm"="vmovups {off}({n}), {d}" "dclass"="zv" "feature"="avx512f" "nodecode" {
  %a = getelementptr i8, ptr %n, i64 %off
  %r = load <32 x half>, ptr %a
  ret <32 x half> %r
}

define void @VSTZ.32xhalf(<32 x half> "reg"="zv" %t, ptr "reg"="r64" %n, i64 "imm" %off) "enc"="evexm(t, n, 4, 1, 0, 0, 2, 0, 0, 0, 0) | 00010001 | rm(t, n, 4, 1, off, 0, 64)" "asm"="vmovups {t}, {off}({n})" "feature"="avx512f" "nodecode" {
  %a = getelementptr i8, ptr %n, i64 %off
  store <32 x half> %t, ptr %a
  ret void
}

define <32 x bfloat> @VLDZ.32xbf32(ptr "reg"="r64" %n, i64 "imm" %off) "enc"="evexm(d, n, 4, 1, 0, 0, 2, 0, 0, 0, 0) | 00010000 | rm(d, n, 4, 1, off, 0, 64)" "asm"="vmovups {off}({n}), {d}" "dclass"="zv" "feature"="avx512f" "nodecode" {
  %a = getelementptr i8, ptr %n, i64 %off
  %r = load <32 x bfloat>, ptr %a
  ret <32 x bfloat> %r
}

define void @VSTZ.32xbf32(<32 x bfloat> "reg"="zv" %t, ptr "reg"="r64" %n, i64 "imm" %off) "enc"="evexm(t, n, 4, 1, 0, 0, 2, 0, 0, 0, 0) | 00010001 | rm(t, n, 4, 1, off, 0, 64)" "asm"="vmovups {t}, {off}({n})" "feature"="avx512f" "nodecode" {
  %a = getelementptr i8, ptr %n, i64 %off
  store <32 x bfloat> %t, ptr %a
  ret void
}

define <16 x i32> @VMOVAPSzv(<16 x i32> "reg"="zv" %n) "enc"="{ evexr(d, n, 1, 0, 0, 2, 0, 0, 0, 0) | 00101000 | 11 reg:3=d&7 rm:3=n&7 ; evexr(n, d, 1, 0, 0, 2, 0, 0, 0, 0) | 00101001 | 11 reg:3=n&7 rm:3=d&7 }" "asm"="vmovaps {n}, {d}" "copy"="zv" "dclass"="zv" "feature"="avx512f" "nodecode" {
  ret <16 x i32> %n
}

define <8 x i64> @VINSERTI64X4hi(<8 x i64> "reg"="zv" "tied" %n, <4 x i64> "reg"="yv" %h) "enc"="evexr(d, h, 3, 1, n, 2, 1, 0, 0, 0) | 00111010 | 11 reg:3=d&7 rm:3=h&7 | 00000001" "asm"="vinserti64x4 $1, {h}, {d}, {d}" "dclass"="zv" "role"="vec.join.y" "nomatch" "feature"="avx512f" "nodecode" {
  ret <8 x i64> %n
}

define <16 x float> @VPXORDzero.16xf32() "enc"="evexr(d, d, 1, 0, d, 2, 1, 0, 0, 0) | 11101111 | 11 reg:3=d&7 rm:3=d&7" "asm"="vpxord {d}, {d}, {d}" "dclass"="zv" "feature"="avx512f" "nodecode" {
  ret <16 x float> zeroinitializer
}

define <16 x i32> @VPXORDzero.16xi32() "enc"="evexr(d, d, 1, 0, d, 2, 1, 0, 0, 0) | 11101111 | 11 reg:3=d&7 rm:3=d&7" "asm"="vpxord {d}, {d}, {d}" "dclass"="zv" "role"="movi.zero.zv" "feature"="avx512f" "nodecode" {
  ret <16 x i32> zeroinitializer
}

define <16 x i32> @VPBROADCASTDm.16xi32(ptr "reg"="r64" %n, i64 "imm" %off) "enc"="evexm(d, n, 4, 2, 0, 0, 2, 1, 0, 0, 0) | 01011000 | rm(d, n, 4, 1, off, 0, 4)" "asm"="vpbroadcastd {off}({n}), {d}" "dclass"="zv" "feature"="avx512f" "nodecode" {
  %a = getelementptr i8, ptr %n, i64 %off
  %x = load i32, ptr %a
  %i = insertelement <16 x i32> poison, i32 %x, i64 0
  %r = shufflevector <16 x i32> %i, <16 x i32> poison, <16 x i32> zeroinitializer
  ret <16 x i32> %r
}

define <16 x float> @VBROADCASTSSm.16xf32(ptr "reg"="r64" %n, i64 "imm" %off) "enc"="evexm(d, n, 4, 2, 0, 0, 2, 1, 0, 0, 0) | 00011000 | rm(d, n, 4, 1, off, 0, 4)" "asm"="vbroadcastss {off}({n}), {d}" "dclass"="zv" "feature"="avx512f" "nodecode" {
  %a = getelementptr i8, ptr %n, i64 %off
  %x = load float, ptr %a
  %i = insertelement <16 x float> poison, float %x, i64 0
  %r = shufflevector <16 x float> %i, <16 x float> poison, <16 x i32> zeroinitializer
  ret <16 x float> %r
}

define <16 x float> @VADDPS.16xf32(<16 x float> "reg"="zv" %n, <16 x float> "reg"="zv" %m) "enc"="evexr(d, m, 1, 0, n, 2, 0, 0, 0, 0) | 01011000 | 11 reg:3=d&7 rm:3=m&7" "asm"="vaddps {m}, {n}, {d}" "dclass"="zv" "feature"="avx512f" "nodecode" {
  %r = fadd <16 x float> %n, %m
  ret <16 x float> %r
}

define <16 x float> @VSUBPS.16xf32(<16 x float> "reg"="zv" %n, <16 x float> "reg"="zv" %m) "enc"="evexr(d, m, 1, 0, n, 2, 0, 0, 0, 0) | 01011100 | 11 reg:3=d&7 rm:3=m&7" "asm"="vsubps {m}, {n}, {d}" "dclass"="zv" "feature"="avx512f" "nodecode" {
  %r = fsub <16 x float> %n, %m
  ret <16 x float> %r
}

define <16 x float> @VMULPS.16xf32(<16 x float> "reg"="zv" %n, <16 x float> "reg"="zv" %m) "enc"="evexr(d, m, 1, 0, n, 2, 0, 0, 0, 0) | 01011001 | 11 reg:3=d&7 rm:3=m&7" "asm"="vmulps {m}, {n}, {d}" "dclass"="zv" "feature"="avx512f" "nodecode" {
  %r = fmul <16 x float> %n, %m
  ret <16 x float> %r
}

define <16 x float> @VFMADD231PS.fma.16xf32(<16 x float> "reg"="zv" %n, <16 x float> "reg"="zv" %m, <16 x float> "reg"="zv" "tied" %acc) "enc"="evexr(d, m, 2, 0, n, 2, 1, 0, 0, 0) | 10111000 | 11 reg:3=d&7 rm:3=m&7" "asm"="vfmadd231ps {m}, {n}, {d}" "dclass"="zv" "feature"="avx512f" "nodecode" {
  %r = call <16 x float> @llvm.fma.v16f32(<16 x float> %n, <16 x float> %m, <16 x float> %acc)
  ret <16 x float> %r
}

declare <16 x float> @llvm.fma.v16f32(<16 x float>, <16 x float>, <16 x float>)

define <16 x float> @VFMADD231PS.fmuladd.16xf32(<16 x float> "reg"="zv" %n, <16 x float> "reg"="zv" %m, <16 x float> "reg"="zv" "tied" %acc) "enc"="evexr(d, m, 2, 0, n, 2, 1, 0, 0, 0) | 10111000 | 11 reg:3=d&7 rm:3=m&7" "asm"="vfmadd231ps {m}, {n}, {d}" "dclass"="zv" "feature"="avx512f" "nodecode" {
  %r = call <16 x float> @llvm.fmuladd.v16f32(<16 x float> %n, <16 x float> %m, <16 x float> %acc)
  ret <16 x float> %r
}

declare <16 x float> @llvm.fmuladd.v16f32(<16 x float>, <16 x float>, <16 x float>)

define <8 x i32> @VCVTNEPS2BF16.16xf32(<16 x float> "reg"="zv" %n) "enc"="evexr(d, n, 2, 0, 0, 2, 2, 0, 0, 0) | 01110010 | 11 reg:3=d&7 rm:3=n&7" "asm"="vcvtneps2bf16 {n}, {d}" "dclass"="yv" "feature"="avx512f,avx512bf16" "nodecode" {
  %r = call <8 x i32> @llvm.epitome.x86.cvtneps2bf16.512(<16 x float> %n)
  ret <8 x i32> %r
}

declare <8 x i32> @llvm.epitome.x86.cvtneps2bf16.512(<16 x float>)

define <16 x float> @VDPBF16PS.16xf32(<16 x float> "reg"="zv" "tied" %acc, <16 x i32> "reg"="zv" %n, <16 x i32> "reg"="zv" %m) "enc"="evexr(d, m, 2, 0, n, 2, 2, 0, 0, 0) | 01010010 | 11 reg:3=d&7 rm:3=m&7" "asm"="vdpbf16ps {m}, {n}, {d}" "dclass"="zv" "feature"="avx512f,avx512bf16" "nodecode" {
  %r = call <16 x float> @llvm.epitome.x86.dpbf16ps.512(<16 x float> %acc, <16 x i32> %n, <16 x i32> %m)
  ret <16 x float> %r
}

declare <16 x float> @llvm.epitome.x86.dpbf16ps.512(<16 x float>, <16 x i32>, <16 x i32>)

define <64 x i8> @VPADDBz.64xi8(<64 x i8> "reg"="zv" %n, <64 x i8> "reg"="zv" %m) "enc"="evexr(d, m, 1, 0, n, 2, 1, 0, 0, 0) | 11111100 | 11 reg:3=d&7 rm:3=m&7" "asm"="vpaddb {m}, {n}, {d}" "dclass"="zv" "feature"="avx512f,avx512bw" "nodecode" {
  %r = add <64 x i8> %n, %m
  ret <64 x i8> %r
}

define <64 x i8> @VPSUBBz.64xi8(<64 x i8> "reg"="zv" %n, <64 x i8> "reg"="zv" %m) "enc"="evexr(d, m, 1, 0, n, 2, 1, 0, 0, 0) | 11111000 | 11 reg:3=d&7 rm:3=m&7" "asm"="vpsubb {m}, {n}, {d}" "dclass"="zv" "feature"="avx512f,avx512bw" "nodecode" {
  %r = sub <64 x i8> %n, %m
  ret <64 x i8> %r
}

define <64 x i8> @VPANDDz.64xi8(<64 x i8> "reg"="zv" %n, <64 x i8> "reg"="zv" %m) "enc"="evexr(d, m, 1, 0, n, 2, 1, 0, 0, 0) | 11011011 | 11 reg:3=d&7 rm:3=m&7" "asm"="vpandd {m}, {n}, {d}" "dclass"="zv" "feature"="avx512f" "nodecode" {
  %r = and <64 x i8> %n, %m
  ret <64 x i8> %r
}

define <64 x i8> @VPORDz.64xi8(<64 x i8> "reg"="zv" %n, <64 x i8> "reg"="zv" %m) "enc"="evexr(d, m, 1, 0, n, 2, 1, 0, 0, 0) | 11101011 | 11 reg:3=d&7 rm:3=m&7" "asm"="vpord {m}, {n}, {d}" "dclass"="zv" "feature"="avx512f" "nodecode" {
  %r = or <64 x i8> %n, %m
  ret <64 x i8> %r
}

define <64 x i8> @VPXORDz.64xi8(<64 x i8> "reg"="zv" %n, <64 x i8> "reg"="zv" %m) "enc"="evexr(d, m, 1, 0, n, 2, 1, 0, 0, 0) | 11101111 | 11 reg:3=d&7 rm:3=m&7" "asm"="vpxord {m}, {n}, {d}" "dclass"="zv" "feature"="avx512f" "nodecode" {
  %r = xor <64 x i8> %n, %m
  ret <64 x i8> %r
}

define <64 x i8> @VPANDNDz.64xi8(<64 x i8> "reg"="zv" %n, <64 x i8> "reg"="zv" %m) "enc"="evexr(d, m, 1, 0, n, 2, 1, 0, 0, 0) | 11011111 | 11 reg:3=d&7 rm:3=m&7" "asm"="vpandnd {m}, {n}, {d}" "dclass"="zv" "feature"="avx512f" "nodecode" {
  %x = xor <64 x i8> %n, splat (i8 -1)
  %r = and <64 x i8> %x, %m
  ret <64 x i8> %r
}

define <64 x i1> @VPCMPB.eq.64xi8(<64 x i8> "reg"="zv" %n, <64 x i8> "reg"="zv" %m) "enc"="evexr(d, m, 3, 0, n, 2, 1, 0, 0, 0) | 00111111 | 11 reg:3=d&7 rm:3=m&7 | 00000000" "asm"="vpcmpb $0, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f,avx512bw" "nodecode" {
  %r = icmp eq <64 x i8> %n, %m
  ret <64 x i1> %r
}

define <64 x i1> @VPCMPB.ne.64xi8(<64 x i8> "reg"="zv" %n, <64 x i8> "reg"="zv" %m) "enc"="evexr(d, m, 3, 0, n, 2, 1, 0, 0, 0) | 00111111 | 11 reg:3=d&7 rm:3=m&7 | 00000100" "asm"="vpcmpb $4, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f,avx512bw" "nodecode" {
  %r = icmp ne <64 x i8> %n, %m
  ret <64 x i1> %r
}

define <64 x i1> @VPCMPB.slt.64xi8(<64 x i8> "reg"="zv" %n, <64 x i8> "reg"="zv" %m) "enc"="evexr(d, m, 3, 0, n, 2, 1, 0, 0, 0) | 00111111 | 11 reg:3=d&7 rm:3=m&7 | 00000001" "asm"="vpcmpb $1, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f,avx512bw" "nodecode" {
  %r = icmp slt <64 x i8> %n, %m
  ret <64 x i1> %r
}

define <64 x i1> @VPCMPB.sle.64xi8(<64 x i8> "reg"="zv" %n, <64 x i8> "reg"="zv" %m) "enc"="evexr(d, m, 3, 0, n, 2, 1, 0, 0, 0) | 00111111 | 11 reg:3=d&7 rm:3=m&7 | 00000010" "asm"="vpcmpb $2, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f,avx512bw" "nodecode" {
  %r = icmp sle <64 x i8> %n, %m
  ret <64 x i1> %r
}

define <64 x i1> @VPCMPB.sge.64xi8(<64 x i8> "reg"="zv" %n, <64 x i8> "reg"="zv" %m) "enc"="evexr(d, m, 3, 0, n, 2, 1, 0, 0, 0) | 00111111 | 11 reg:3=d&7 rm:3=m&7 | 00000101" "asm"="vpcmpb $5, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f,avx512bw" "nodecode" {
  %r = icmp sge <64 x i8> %n, %m
  ret <64 x i1> %r
}

define <64 x i1> @VPCMPB.sgt.64xi8(<64 x i8> "reg"="zv" %n, <64 x i8> "reg"="zv" %m) "enc"="evexr(d, m, 3, 0, n, 2, 1, 0, 0, 0) | 00111111 | 11 reg:3=d&7 rm:3=m&7 | 00000110" "asm"="vpcmpb $6, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f,avx512bw" "nodecode" {
  %r = icmp sgt <64 x i8> %n, %m
  ret <64 x i1> %r
}

define <64 x i1> @VPCMPUB.ult.64xi8(<64 x i8> "reg"="zv" %n, <64 x i8> "reg"="zv" %m) "enc"="evexr(d, m, 3, 0, n, 2, 1, 0, 0, 0) | 00111110 | 11 reg:3=d&7 rm:3=m&7 | 00000001" "asm"="vpcmpub $1, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f,avx512bw" "nodecode" {
  %r = icmp ult <64 x i8> %n, %m
  ret <64 x i1> %r
}

define <64 x i1> @VPCMPUB.ule.64xi8(<64 x i8> "reg"="zv" %n, <64 x i8> "reg"="zv" %m) "enc"="evexr(d, m, 3, 0, n, 2, 1, 0, 0, 0) | 00111110 | 11 reg:3=d&7 rm:3=m&7 | 00000010" "asm"="vpcmpub $2, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f,avx512bw" "nodecode" {
  %r = icmp ule <64 x i8> %n, %m
  ret <64 x i1> %r
}

define <64 x i1> @VPCMPUB.uge.64xi8(<64 x i8> "reg"="zv" %n, <64 x i8> "reg"="zv" %m) "enc"="evexr(d, m, 3, 0, n, 2, 1, 0, 0, 0) | 00111110 | 11 reg:3=d&7 rm:3=m&7 | 00000101" "asm"="vpcmpub $5, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f,avx512bw" "nodecode" {
  %r = icmp uge <64 x i8> %n, %m
  ret <64 x i1> %r
}

define <64 x i1> @VPCMPUB.ugt.64xi8(<64 x i8> "reg"="zv" %n, <64 x i8> "reg"="zv" %m) "enc"="evexr(d, m, 3, 0, n, 2, 1, 0, 0, 0) | 00111110 | 11 reg:3=d&7 rm:3=m&7 | 00000110" "asm"="vpcmpub $6, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f,avx512bw" "nodecode" {
  %r = icmp ugt <64 x i8> %n, %m
  ret <64 x i1> %r
}

define <64 x i8> @VPBLENDMB.64xi8(<64 x i1> "reg"="kw" %k, <64 x i8> "reg"="zv" %a, <64 x i8> "reg"="zv" %b) "enc"="{ if(k!=0) evexr(d, a, 2, 0, b, 2, 1, 0, k, 0) | 01100110 | 11 reg:3=d&7 rm:3=a&7 }" "asm"="vpblendmb {a}, {b}, {d} {lb}{k}{rb}" "dclass"="zv" "feature"="avx512f,avx512bw" "nodecode" {
  %r = select <64 x i1> %k, <64 x i8> %a, <64 x i8> %b
  ret <64 x i8> %r
}

define <64 x i8> @VPMINSBz.64xi8(<64 x i8> "reg"="zv" %n, <64 x i8> "reg"="zv" %m) "enc"="evexr(d, m, 2, 0, n, 2, 1, 0, 0, 0) | 00111000 | 11 reg:3=d&7 rm:3=m&7" "asm"="vpminsb {m}, {n}, {d}" "dclass"="zv" "feature"="avx512f,avx512bw" "nodecode" {
  %r = call <64 x i8> @llvm.smin.v64i8(<64 x i8> %n, <64 x i8> %m)
  ret <64 x i8> %r
}

declare <64 x i8> @llvm.smin.v64i8(<64 x i8>, <64 x i8>)

define <64 x i8> @VPMAXSBz.64xi8(<64 x i8> "reg"="zv" %n, <64 x i8> "reg"="zv" %m) "enc"="evexr(d, m, 2, 0, n, 2, 1, 0, 0, 0) | 00111100 | 11 reg:3=d&7 rm:3=m&7" "asm"="vpmaxsb {m}, {n}, {d}" "dclass"="zv" "feature"="avx512f,avx512bw" "nodecode" {
  %r = call <64 x i8> @llvm.smax.v64i8(<64 x i8> %n, <64 x i8> %m)
  ret <64 x i8> %r
}

declare <64 x i8> @llvm.smax.v64i8(<64 x i8>, <64 x i8>)

define <64 x i8> @VPMINUBz.64xi8(<64 x i8> "reg"="zv" %n, <64 x i8> "reg"="zv" %m) "enc"="evexr(d, m, 1, 0, n, 2, 1, 0, 0, 0) | 11011010 | 11 reg:3=d&7 rm:3=m&7" "asm"="vpminub {m}, {n}, {d}" "dclass"="zv" "feature"="avx512f,avx512bw" "nodecode" {
  %r = call <64 x i8> @llvm.umin.v64i8(<64 x i8> %n, <64 x i8> %m)
  ret <64 x i8> %r
}

declare <64 x i8> @llvm.umin.v64i8(<64 x i8>, <64 x i8>)

define <64 x i8> @VPMAXUBz.64xi8(<64 x i8> "reg"="zv" %n, <64 x i8> "reg"="zv" %m) "enc"="evexr(d, m, 1, 0, n, 2, 1, 0, 0, 0) | 11011110 | 11 reg:3=d&7 rm:3=m&7" "asm"="vpmaxub {m}, {n}, {d}" "dclass"="zv" "feature"="avx512f,avx512bw" "nodecode" {
  %r = call <64 x i8> @llvm.umax.v64i8(<64 x i8> %n, <64 x i8> %m)
  ret <64 x i8> %r
}

declare <64 x i8> @llvm.umax.v64i8(<64 x i8>, <64 x i8>)

define <64 x i8> @VPABSBz.false.64xi8(<64 x i8> "reg"="zv" %n) "enc"="evexr(d, n, 2, 0, 0, 2, 1, 0, 0, 0) | 00011100 | 11 reg:3=d&7 rm:3=n&7" "asm"="vpabsb {n}, {d}" "dclass"="zv" "feature"="avx512f,avx512bw" "nodecode" {
  %r = call <64 x i8> @llvm.abs.v64i8(<64 x i8> %n, i1 false)
  ret <64 x i8> %r
}

define <64 x i8> @VPABSBz.true.64xi8(<64 x i8> "reg"="zv" %n) "enc"="evexr(d, n, 2, 0, 0, 2, 1, 0, 0, 0) | 00011100 | 11 reg:3=d&7 rm:3=n&7" "asm"="vpabsb {n}, {d}" "dclass"="zv" "feature"="avx512f,avx512bw" "nodecode" {
  %r = call <64 x i8> @llvm.abs.v64i8(<64 x i8> %n, i1 true)
  ret <64 x i8> %r
}

declare <64 x i8> @llvm.abs.v64i8(<64 x i8>, i1)

define <64 x i8> @VPMOVM2B.64xi8(<64 x i1> "reg"="kw" %k) "enc"="evexr(d, k, 2, 0, 0, 2, 2, 0, 0, 0) | 00101000 | 11 reg:3=d&7 rm:3=k&7" "asm"="vpmovm2b {k}, {d}" "dclass"="zv" "feature"="avx512f,avx512bw" "nodecode" {
  %r = sext <64 x i1> %k to <64 x i8>
  ret <64 x i8> %r
}

define <64 x i8> @VMASKBITS.64xi8(<64 x i1> "reg"="kw" %k) "enc"="evexr(d, k, 2, 0, 0, 2, 2, 0, 0, 0) | 00101000 | 11 reg:3=d&7 rm:3=k&7" "asm"="vpmovm2b {k}, {d}" "dclass"="zv" "feature"="avx512f,avx512bw" "nodecode" {
  %r = call <64 x i8> @llvm.epitome.mask.bits.v64i8(<64 x i1> %k)
  ret <64 x i8> %r
}

define <64 x i8> @zextk.64xi8(<64 x i1> "reg"="kw" %k) "recipe"="zextk.64xi8.expand" "feature"="avx512f,avx512bw" {
  %r = zext <64 x i1> %k to <64 x i8>
  ret <64 x i8> %r
}

define <64 x i8> @zextk.64xi8.expand(<64 x i1> %k) {
  %s = call <64 x i8> @VPMOVM2B.64xi8(<64 x i1> %k)
  %r = call <64 x i8> @VPABSBz.false.64xi8(<64 x i8> %s)
  ret <64 x i8> %r
}

define <64 x i1> @VBITSMASK.64xi8(<64 x i8> "reg"="zv" %n) "enc"="evexr(d, n, 2, 0, n, 2, 1, 0, 0, 0) | 00100110 | 11 reg:3=d&7 rm:3=n&7" "asm"="vptestmb {n}, {n}, {d}" "dclass"="kw" "feature"="avx512f,avx512bw" "nodecode" {
  %r = call <64 x i1> @llvm.epitome.bits.mask.v64i8(<64 x i8> %n)
  ret <64 x i1> %r
}

declare <64 x i8> @llvm.epitome.mask.bits.v64i8(<64 x i1>)
declare <64 x i1> @llvm.epitome.bits.mask.v64i8(<64 x i8>)

define <64 x i8> @VPBROADCASTBr.64xi8(i8 "reg"="r32" %x) "enc"="evexr(d, x, 2, 0, 0, 2, 1, 0, 0, 0) | 01111010 | 11 reg:3=d&7 rm:3=x&7" "asm"="vpbroadcastb {x}, {d}" "dclass"="zv" "role"="dup.zv.8" "feature"="avx512f,avx512bw" "nodecode" {
  %i = insertelement <64 x i8> poison, i8 %x, i64 0
  %r = shufflevector <64 x i8> %i, <64 x i8> poison, <64 x i32> zeroinitializer
  ret <64 x i8> %r
}

define <32 x i16> @VPADDWz.32xi16(<32 x i16> "reg"="zv" %n, <32 x i16> "reg"="zv" %m) "enc"="evexr(d, m, 1, 0, n, 2, 1, 0, 0, 0) | 11111101 | 11 reg:3=d&7 rm:3=m&7" "asm"="vpaddw {m}, {n}, {d}" "dclass"="zv" "feature"="avx512f,avx512bw" "nodecode" {
  %r = add <32 x i16> %n, %m
  ret <32 x i16> %r
}

define <32 x i16> @VPSUBWz.32xi16(<32 x i16> "reg"="zv" %n, <32 x i16> "reg"="zv" %m) "enc"="evexr(d, m, 1, 0, n, 2, 1, 0, 0, 0) | 11111001 | 11 reg:3=d&7 rm:3=m&7" "asm"="vpsubw {m}, {n}, {d}" "dclass"="zv" "feature"="avx512f,avx512bw" "nodecode" {
  %r = sub <32 x i16> %n, %m
  ret <32 x i16> %r
}

define <32 x i16> @VPMULLWz.32xi16(<32 x i16> "reg"="zv" %n, <32 x i16> "reg"="zv" %m) "enc"="evexr(d, m, 1, 0, n, 2, 1, 0, 0, 0) | 11010101 | 11 reg:3=d&7 rm:3=m&7" "asm"="vpmullw {m}, {n}, {d}" "dclass"="zv" "feature"="avx512f,avx512bw" "nodecode" {
  %r = mul <32 x i16> %n, %m
  ret <32 x i16> %r
}

define <32 x i16> @VPANDDz.32xi16(<32 x i16> "reg"="zv" %n, <32 x i16> "reg"="zv" %m) "enc"="evexr(d, m, 1, 0, n, 2, 1, 0, 0, 0) | 11011011 | 11 reg:3=d&7 rm:3=m&7" "asm"="vpandd {m}, {n}, {d}" "dclass"="zv" "feature"="avx512f" "nodecode" {
  %r = and <32 x i16> %n, %m
  ret <32 x i16> %r
}

define <32 x i16> @VPORDz.32xi16(<32 x i16> "reg"="zv" %n, <32 x i16> "reg"="zv" %m) "enc"="evexr(d, m, 1, 0, n, 2, 1, 0, 0, 0) | 11101011 | 11 reg:3=d&7 rm:3=m&7" "asm"="vpord {m}, {n}, {d}" "dclass"="zv" "feature"="avx512f" "nodecode" {
  %r = or <32 x i16> %n, %m
  ret <32 x i16> %r
}

define <32 x i16> @VPXORDz.32xi16(<32 x i16> "reg"="zv" %n, <32 x i16> "reg"="zv" %m) "enc"="evexr(d, m, 1, 0, n, 2, 1, 0, 0, 0) | 11101111 | 11 reg:3=d&7 rm:3=m&7" "asm"="vpxord {m}, {n}, {d}" "dclass"="zv" "feature"="avx512f" "nodecode" {
  %r = xor <32 x i16> %n, %m
  ret <32 x i16> %r
}

define <32 x i16> @VPANDNDz.32xi16(<32 x i16> "reg"="zv" %n, <32 x i16> "reg"="zv" %m) "enc"="evexr(d, m, 1, 0, n, 2, 1, 0, 0, 0) | 11011111 | 11 reg:3=d&7 rm:3=m&7" "asm"="vpandnd {m}, {n}, {d}" "dclass"="zv" "feature"="avx512f" "nodecode" {
  %x = xor <32 x i16> %n, splat (i16 -1)
  %r = and <32 x i16> %x, %m
  ret <32 x i16> %r
}

define <32 x i1> @VPCMPW.eq.32xi16(<32 x i16> "reg"="zv" %n, <32 x i16> "reg"="zv" %m) "enc"="evexr(d, m, 3, 1, n, 2, 1, 0, 0, 0) | 00111111 | 11 reg:3=d&7 rm:3=m&7 | 00000000" "asm"="vpcmpw $0, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f,avx512bw" "nodecode" {
  %r = icmp eq <32 x i16> %n, %m
  ret <32 x i1> %r
}

define <32 x i1> @VPCMPW.ne.32xi16(<32 x i16> "reg"="zv" %n, <32 x i16> "reg"="zv" %m) "enc"="evexr(d, m, 3, 1, n, 2, 1, 0, 0, 0) | 00111111 | 11 reg:3=d&7 rm:3=m&7 | 00000100" "asm"="vpcmpw $4, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f,avx512bw" "nodecode" {
  %r = icmp ne <32 x i16> %n, %m
  ret <32 x i1> %r
}

define <32 x i1> @VPCMPW.slt.32xi16(<32 x i16> "reg"="zv" %n, <32 x i16> "reg"="zv" %m) "enc"="evexr(d, m, 3, 1, n, 2, 1, 0, 0, 0) | 00111111 | 11 reg:3=d&7 rm:3=m&7 | 00000001" "asm"="vpcmpw $1, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f,avx512bw" "nodecode" {
  %r = icmp slt <32 x i16> %n, %m
  ret <32 x i1> %r
}

define <32 x i1> @VPCMPW.sle.32xi16(<32 x i16> "reg"="zv" %n, <32 x i16> "reg"="zv" %m) "enc"="evexr(d, m, 3, 1, n, 2, 1, 0, 0, 0) | 00111111 | 11 reg:3=d&7 rm:3=m&7 | 00000010" "asm"="vpcmpw $2, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f,avx512bw" "nodecode" {
  %r = icmp sle <32 x i16> %n, %m
  ret <32 x i1> %r
}

define <32 x i1> @VPCMPW.sge.32xi16(<32 x i16> "reg"="zv" %n, <32 x i16> "reg"="zv" %m) "enc"="evexr(d, m, 3, 1, n, 2, 1, 0, 0, 0) | 00111111 | 11 reg:3=d&7 rm:3=m&7 | 00000101" "asm"="vpcmpw $5, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f,avx512bw" "nodecode" {
  %r = icmp sge <32 x i16> %n, %m
  ret <32 x i1> %r
}

define <32 x i1> @VPCMPW.sgt.32xi16(<32 x i16> "reg"="zv" %n, <32 x i16> "reg"="zv" %m) "enc"="evexr(d, m, 3, 1, n, 2, 1, 0, 0, 0) | 00111111 | 11 reg:3=d&7 rm:3=m&7 | 00000110" "asm"="vpcmpw $6, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f,avx512bw" "nodecode" {
  %r = icmp sgt <32 x i16> %n, %m
  ret <32 x i1> %r
}

define <32 x i1> @VPCMPUW.ult.32xi16(<32 x i16> "reg"="zv" %n, <32 x i16> "reg"="zv" %m) "enc"="evexr(d, m, 3, 1, n, 2, 1, 0, 0, 0) | 00111110 | 11 reg:3=d&7 rm:3=m&7 | 00000001" "asm"="vpcmpuw $1, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f,avx512bw" "nodecode" {
  %r = icmp ult <32 x i16> %n, %m
  ret <32 x i1> %r
}

define <32 x i1> @VPCMPUW.ule.32xi16(<32 x i16> "reg"="zv" %n, <32 x i16> "reg"="zv" %m) "enc"="evexr(d, m, 3, 1, n, 2, 1, 0, 0, 0) | 00111110 | 11 reg:3=d&7 rm:3=m&7 | 00000010" "asm"="vpcmpuw $2, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f,avx512bw" "nodecode" {
  %r = icmp ule <32 x i16> %n, %m
  ret <32 x i1> %r
}

define <32 x i1> @VPCMPUW.uge.32xi16(<32 x i16> "reg"="zv" %n, <32 x i16> "reg"="zv" %m) "enc"="evexr(d, m, 3, 1, n, 2, 1, 0, 0, 0) | 00111110 | 11 reg:3=d&7 rm:3=m&7 | 00000101" "asm"="vpcmpuw $5, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f,avx512bw" "nodecode" {
  %r = icmp uge <32 x i16> %n, %m
  ret <32 x i1> %r
}

define <32 x i1> @VPCMPUW.ugt.32xi16(<32 x i16> "reg"="zv" %n, <32 x i16> "reg"="zv" %m) "enc"="evexr(d, m, 3, 1, n, 2, 1, 0, 0, 0) | 00111110 | 11 reg:3=d&7 rm:3=m&7 | 00000110" "asm"="vpcmpuw $6, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f,avx512bw" "nodecode" {
  %r = icmp ugt <32 x i16> %n, %m
  ret <32 x i1> %r
}

define <32 x i16> @VPBLENDMW.32xi16(<32 x i1> "reg"="kw" %k, <32 x i16> "reg"="zv" %a, <32 x i16> "reg"="zv" %b) "enc"="{ if(k!=0) evexr(d, a, 2, 1, b, 2, 1, 0, k, 0) | 01100110 | 11 reg:3=d&7 rm:3=a&7 }" "asm"="vpblendmw {a}, {b}, {d} {lb}{k}{rb}" "dclass"="zv" "feature"="avx512f,avx512bw" "nodecode" {
  %r = select <32 x i1> %k, <32 x i16> %a, <32 x i16> %b
  ret <32 x i16> %r
}

define <32 x i16> @VPSLLWiz.32xi16(<32 x i16> "reg"="zv" %n, <32 x i16> "imm" %sh) "enc"="evexr(6, n, 1, 0, d, 2, 1, 0, 0, 0) | 01110001 | 11 110 rm:3=n&7 | i1:8=sh" "asm"="vpsllw ${sh}, {n}, {d}" "dclass"="zv" "feature"="avx512f,avx512bw" "nodecode" {
  %r = shl <32 x i16> %n, %sh
  ret <32 x i16> %r
}

define <32 x i16> @VPSLLVWz.32xi16(<32 x i16> "reg"="zv" %n, <32 x i16> "reg"="zv" %m) "enc"="evexr(d, m, 2, 1, n, 2, 1, 0, 0, 0) | 00010010 | 11 reg:3=d&7 rm:3=m&7" "asm"="vpsllvw {m}, {n}, {d}" "dclass"="zv" "feature"="avx512f,avx512bw" "nodecode" {
  %r = shl <32 x i16> %n, %m
  ret <32 x i16> %r
}

define <32 x i16> @VPSRLWiz.32xi16(<32 x i16> "reg"="zv" %n, <32 x i16> "imm" %sh) "enc"="evexr(2, n, 1, 0, d, 2, 1, 0, 0, 0) | 01110001 | 11 010 rm:3=n&7 | i1:8=sh" "asm"="vpsrlw ${sh}, {n}, {d}" "dclass"="zv" "feature"="avx512f,avx512bw" "nodecode" {
  %r = lshr <32 x i16> %n, %sh
  ret <32 x i16> %r
}

define <32 x i16> @VPSRLVWz.32xi16(<32 x i16> "reg"="zv" %n, <32 x i16> "reg"="zv" %m) "enc"="evexr(d, m, 2, 1, n, 2, 1, 0, 0, 0) | 00010000 | 11 reg:3=d&7 rm:3=m&7" "asm"="vpsrlvw {m}, {n}, {d}" "dclass"="zv" "feature"="avx512f,avx512bw" "nodecode" {
  %r = lshr <32 x i16> %n, %m
  ret <32 x i16> %r
}

define <32 x i16> @VPSRAWiz.32xi16(<32 x i16> "reg"="zv" %n, <32 x i16> "imm" %sh) "enc"="evexr(4, n, 1, 0, d, 2, 1, 0, 0, 0) | 01110001 | 11 100 rm:3=n&7 | i1:8=sh" "asm"="vpsraw ${sh}, {n}, {d}" "dclass"="zv" "feature"="avx512f,avx512bw" "nodecode" {
  %r = ashr <32 x i16> %n, %sh
  ret <32 x i16> %r
}

define <32 x i16> @VPSRAVWz.32xi16(<32 x i16> "reg"="zv" %n, <32 x i16> "reg"="zv" %m) "enc"="evexr(d, m, 2, 1, n, 2, 1, 0, 0, 0) | 00010001 | 11 reg:3=d&7 rm:3=m&7" "asm"="vpsravw {m}, {n}, {d}" "dclass"="zv" "feature"="avx512f,avx512bw" "nodecode" {
  %r = ashr <32 x i16> %n, %m
  ret <32 x i16> %r
}

define <32 x i16> @VPMINSWz.32xi16(<32 x i16> "reg"="zv" %n, <32 x i16> "reg"="zv" %m) "enc"="evexr(d, m, 1, 0, n, 2, 1, 0, 0, 0) | 11101010 | 11 reg:3=d&7 rm:3=m&7" "asm"="vpminsw {m}, {n}, {d}" "dclass"="zv" "feature"="avx512f,avx512bw" "nodecode" {
  %r = call <32 x i16> @llvm.smin.v32i16(<32 x i16> %n, <32 x i16> %m)
  ret <32 x i16> %r
}

declare <32 x i16> @llvm.smin.v32i16(<32 x i16>, <32 x i16>)

define <32 x i16> @VPMAXSWz.32xi16(<32 x i16> "reg"="zv" %n, <32 x i16> "reg"="zv" %m) "enc"="evexr(d, m, 1, 0, n, 2, 1, 0, 0, 0) | 11101110 | 11 reg:3=d&7 rm:3=m&7" "asm"="vpmaxsw {m}, {n}, {d}" "dclass"="zv" "feature"="avx512f,avx512bw" "nodecode" {
  %r = call <32 x i16> @llvm.smax.v32i16(<32 x i16> %n, <32 x i16> %m)
  ret <32 x i16> %r
}

declare <32 x i16> @llvm.smax.v32i16(<32 x i16>, <32 x i16>)

define <32 x i16> @VPMINUWz.32xi16(<32 x i16> "reg"="zv" %n, <32 x i16> "reg"="zv" %m) "enc"="evexr(d, m, 2, 0, n, 2, 1, 0, 0, 0) | 00111010 | 11 reg:3=d&7 rm:3=m&7" "asm"="vpminuw {m}, {n}, {d}" "dclass"="zv" "feature"="avx512f,avx512bw" "nodecode" {
  %r = call <32 x i16> @llvm.umin.v32i16(<32 x i16> %n, <32 x i16> %m)
  ret <32 x i16> %r
}

declare <32 x i16> @llvm.umin.v32i16(<32 x i16>, <32 x i16>)

define <32 x i16> @VPMAXUWz.32xi16(<32 x i16> "reg"="zv" %n, <32 x i16> "reg"="zv" %m) "enc"="evexr(d, m, 2, 0, n, 2, 1, 0, 0, 0) | 00111110 | 11 reg:3=d&7 rm:3=m&7" "asm"="vpmaxuw {m}, {n}, {d}" "dclass"="zv" "feature"="avx512f,avx512bw" "nodecode" {
  %r = call <32 x i16> @llvm.umax.v32i16(<32 x i16> %n, <32 x i16> %m)
  ret <32 x i16> %r
}

declare <32 x i16> @llvm.umax.v32i16(<32 x i16>, <32 x i16>)

define <32 x i16> @VPABSWz.false.32xi16(<32 x i16> "reg"="zv" %n) "enc"="evexr(d, n, 2, 0, 0, 2, 1, 0, 0, 0) | 00011101 | 11 reg:3=d&7 rm:3=n&7" "asm"="vpabsw {n}, {d}" "dclass"="zv" "feature"="avx512f,avx512bw" "nodecode" {
  %r = call <32 x i16> @llvm.abs.v32i16(<32 x i16> %n, i1 false)
  ret <32 x i16> %r
}

define <32 x i16> @VPABSWz.true.32xi16(<32 x i16> "reg"="zv" %n) "enc"="evexr(d, n, 2, 0, 0, 2, 1, 0, 0, 0) | 00011101 | 11 reg:3=d&7 rm:3=n&7" "asm"="vpabsw {n}, {d}" "dclass"="zv" "feature"="avx512f,avx512bw" "nodecode" {
  %r = call <32 x i16> @llvm.abs.v32i16(<32 x i16> %n, i1 true)
  ret <32 x i16> %r
}

declare <32 x i16> @llvm.abs.v32i16(<32 x i16>, i1)

define <32 x i16> @VPMOVM2W.32xi16(<32 x i1> "reg"="kw" %k) "enc"="evexr(d, k, 2, 1, 0, 2, 2, 0, 0, 0) | 00101000 | 11 reg:3=d&7 rm:3=k&7" "asm"="vpmovm2w {k}, {d}" "dclass"="zv" "feature"="avx512f,avx512bw" "nodecode" {
  %r = sext <32 x i1> %k to <32 x i16>
  ret <32 x i16> %r
}

define <32 x i16> @VMASKBITS.32xi16(<32 x i1> "reg"="kw" %k) "enc"="evexr(d, k, 2, 1, 0, 2, 2, 0, 0, 0) | 00101000 | 11 reg:3=d&7 rm:3=k&7" "asm"="vpmovm2w {k}, {d}" "dclass"="zv" "feature"="avx512f,avx512bw" "nodecode" {
  %r = call <32 x i16> @llvm.epitome.mask.bits.v32i16(<32 x i1> %k)
  ret <32 x i16> %r
}

define <32 x i16> @zextk.32xi16(<32 x i1> "reg"="kw" %k) "recipe"="zextk.32xi16.expand" "feature"="avx512f,avx512bw" {
  %r = zext <32 x i1> %k to <32 x i16>
  ret <32 x i16> %r
}

define <32 x i16> @zextk.32xi16.expand(<32 x i1> %k) {
  %s = call <32 x i16> @VPMOVM2W.32xi16(<32 x i1> %k)
  %r = call <32 x i16> @VPABSWz.false.32xi16(<32 x i16> %s)
  ret <32 x i16> %r
}

define <32 x i1> @VBITSMASK.32xi16(<32 x i16> "reg"="zv" %n) "enc"="evexr(d, n, 2, 1, n, 2, 1, 0, 0, 0) | 00100110 | 11 reg:3=d&7 rm:3=n&7" "asm"="vptestmw {n}, {n}, {d}" "dclass"="kw" "feature"="avx512f,avx512bw" "nodecode" {
  %r = call <32 x i1> @llvm.epitome.bits.mask.v32i16(<32 x i16> %n)
  ret <32 x i1> %r
}

declare <32 x i16> @llvm.epitome.mask.bits.v32i16(<32 x i1>)
declare <32 x i1> @llvm.epitome.bits.mask.v32i16(<32 x i16>)

define <32 x i16> @VPBROADCASTWr.32xi16(i16 "reg"="r32" %x) "enc"="evexr(d, x, 2, 0, 0, 2, 1, 0, 0, 0) | 01111011 | 11 reg:3=d&7 rm:3=x&7" "asm"="vpbroadcastw {x}, {d}" "dclass"="zv" "role"="dup.zv.16" "feature"="avx512f,avx512bw" "nodecode" {
  %i = insertelement <32 x i16> poison, i16 %x, i64 0
  %r = shufflevector <32 x i16> %i, <32 x i16> poison, <32 x i32> zeroinitializer
  ret <32 x i16> %r
}

define <16 x i32> @VPADDDz.16xi32(<16 x i32> "reg"="zv" %n, <16 x i32> "reg"="zv" %m) "enc"="evexr(d, m, 1, 0, n, 2, 1, 0, 0, 0) | 11111110 | 11 reg:3=d&7 rm:3=m&7" "asm"="vpaddd {m}, {n}, {d}" "dclass"="zv" "feature"="avx512f" "nodecode" {
  %r = add <16 x i32> %n, %m
  ret <16 x i32> %r
}

; Narrowing and widening 16 lanes of 32 bits (AVX-512F): vpmovdb and
; vpmovdw truncate a zmm register into an xmm or ymm one, vpmovzxbd and
; vpmovzxwd zero-extend back.
define <16 x i8> @VPMOVDBz(<16 x i32> "reg"="zv" %n) "enc"="evexr(n, d, 2, 0, 0, 2, 2, 0, 0, 0) | 00110001 | 11 reg:3=n&7 rm:3=d&7" "asm"="vpmovdb {n}, {d}" "dclass"="xv" "feature"="avx512f" "nodecode" {
  %r = trunc <16 x i32> %n to <16 x i8>
  ret <16 x i8> %r
}

define <16 x i16> @VPMOVDWz(<16 x i32> "reg"="zv" %n) "enc"="evexr(n, d, 2, 0, 0, 2, 2, 0, 0, 0) | 00110011 | 11 reg:3=n&7 rm:3=d&7" "asm"="vpmovdw {n}, {d}" "dclass"="yv" "feature"="avx512f" "nodecode" {
  %r = trunc <16 x i32> %n to <16 x i16>
  ret <16 x i16> %r
}

define <16 x i32> @VPMOVZXBDz(<16 x i8> "reg"="xv" %n) "enc"="evexr(d, n, 2, 0, 0, 2, 1, 0, 0, 0) | 00110001 | 11 reg:3=d&7 rm:3=n&7" "asm"="vpmovzxbd {n}, {d}" "dclass"="zv" "feature"="avx512f" "nodecode" {
  %r = zext <16 x i8> %n to <16 x i32>
  ret <16 x i32> %r
}

define <16 x i32> @VPMOVZXWDz(<16 x i16> "reg"="yv" %n) "enc"="evexr(d, n, 2, 0, 0, 2, 1, 0, 0, 0) | 00110011 | 11 reg:3=d&7 rm:3=n&7" "asm"="vpmovzxwd {n}, {d}" "dclass"="zv" "feature"="avx512f" "nodecode" {
  %r = zext <16 x i16> %n to <16 x i32>
  ret <16 x i32> %r
}

; The same for 8 and 4 lanes of 32 bits: vpmovdb and vpmovdw from a ymm or
; xmm register (AVX-512VL).
define <8 x i8> @VPMOVDBy(<8 x i32> "reg"="yv" %n) "enc"="evexr(n, d, 2, 0, 0, 1, 2, 0, 0, 0) | 00110001 | 11 reg:3=n&7 rm:3=d&7" "asm"="vpmovdb {n}, {d}" "dclass"="xv" "feature"="avx512f,avx512vl" "nodecode" {
  %r = trunc <8 x i32> %n to <8 x i8>
  ret <8 x i8> %r
}

define <4 x i8> @VPMOVDBx(<4 x i32> "reg"="xv" %n) "enc"="evexr(n, d, 2, 0, 0, 0, 2, 0, 0, 0) | 00110001 | 11 reg:3=n&7 rm:3=d&7" "asm"="vpmovdb {n}, {d}" "dclass"="xv" "feature"="avx512f,avx512vl" "nodecode" {
  %r = trunc <4 x i32> %n to <4 x i8>
  ret <4 x i8> %r
}

define <8 x i16> @VPMOVDWy(<8 x i32> "reg"="yv" %n) "enc"="evexr(n, d, 2, 0, 0, 1, 2, 0, 0, 0) | 00110011 | 11 reg:3=n&7 rm:3=d&7" "asm"="vpmovdw {n}, {d}" "dclass"="xv" "feature"="avx512f,avx512vl" "nodecode" {
  %r = trunc <8 x i32> %n to <8 x i16>
  ret <8 x i16> %r
}

define <4 x i16> @VPMOVDWx(<4 x i32> "reg"="xv" %n) "enc"="evexr(n, d, 2, 0, 0, 0, 2, 0, 0, 0) | 00110011 | 11 reg:3=n&7 rm:3=d&7" "asm"="vpmovdw {n}, {d}" "dclass"="xv" "feature"="avx512f,avx512vl" "nodecode" {
  %r = trunc <4 x i32> %n to <4 x i16>
  ret <4 x i16> %r
}

define <16 x i32> @VPSUBDz.16xi32(<16 x i32> "reg"="zv" %n, <16 x i32> "reg"="zv" %m) "enc"="evexr(d, m, 1, 0, n, 2, 1, 0, 0, 0) | 11111010 | 11 reg:3=d&7 rm:3=m&7" "asm"="vpsubd {m}, {n}, {d}" "dclass"="zv" "feature"="avx512f" "nodecode" {
  %r = sub <16 x i32> %n, %m
  ret <16 x i32> %r
}

define <16 x i32> @VPMULLDz.16xi32(<16 x i32> "reg"="zv" %n, <16 x i32> "reg"="zv" %m) "enc"="evexr(d, m, 2, 0, n, 2, 1, 0, 0, 0) | 01000000 | 11 reg:3=d&7 rm:3=m&7" "asm"="vpmulld {m}, {n}, {d}" "dclass"="zv" "feature"="avx512f" "nodecode" {
  %r = mul <16 x i32> %n, %m
  ret <16 x i32> %r
}

define <16 x i32> @VPANDDz.16xi32(<16 x i32> "reg"="zv" %n, <16 x i32> "reg"="zv" %m) "enc"="evexr(d, m, 1, 0, n, 2, 1, 0, 0, 0) | 11011011 | 11 reg:3=d&7 rm:3=m&7" "asm"="vpandd {m}, {n}, {d}" "dclass"="zv" "feature"="avx512f" "nodecode" {
  %r = and <16 x i32> %n, %m
  ret <16 x i32> %r
}

define <16 x i32> @VPORDz.16xi32(<16 x i32> "reg"="zv" %n, <16 x i32> "reg"="zv" %m) "enc"="evexr(d, m, 1, 0, n, 2, 1, 0, 0, 0) | 11101011 | 11 reg:3=d&7 rm:3=m&7" "asm"="vpord {m}, {n}, {d}" "dclass"="zv" "feature"="avx512f" "nodecode" {
  %r = or <16 x i32> %n, %m
  ret <16 x i32> %r
}

define <16 x i32> @VPXORDz.16xi32(<16 x i32> "reg"="zv" %n, <16 x i32> "reg"="zv" %m) "enc"="evexr(d, m, 1, 0, n, 2, 1, 0, 0, 0) | 11101111 | 11 reg:3=d&7 rm:3=m&7" "asm"="vpxord {m}, {n}, {d}" "dclass"="zv" "feature"="avx512f" "nodecode" {
  %r = xor <16 x i32> %n, %m
  ret <16 x i32> %r
}

define <16 x i32> @VPANDNDz.16xi32(<16 x i32> "reg"="zv" %n, <16 x i32> "reg"="zv" %m) "enc"="evexr(d, m, 1, 0, n, 2, 1, 0, 0, 0) | 11011111 | 11 reg:3=d&7 rm:3=m&7" "asm"="vpandnd {m}, {n}, {d}" "dclass"="zv" "feature"="avx512f" "nodecode" {
  %x = xor <16 x i32> %n, splat (i32 -1)
  %r = and <16 x i32> %x, %m
  ret <16 x i32> %r
}

define <16 x i1> @VPCMPD.eq.16xi32(<16 x i32> "reg"="zv" %n, <16 x i32> "reg"="zv" %m) "enc"="evexr(d, m, 3, 0, n, 2, 1, 0, 0, 0) | 00011111 | 11 reg:3=d&7 rm:3=m&7 | 00000000" "asm"="vpcmpd $0, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f" "nodecode" {
  %r = icmp eq <16 x i32> %n, %m
  ret <16 x i1> %r
}

define <16 x i1> @VPCMPD.ne.16xi32(<16 x i32> "reg"="zv" %n, <16 x i32> "reg"="zv" %m) "enc"="evexr(d, m, 3, 0, n, 2, 1, 0, 0, 0) | 00011111 | 11 reg:3=d&7 rm:3=m&7 | 00000100" "asm"="vpcmpd $4, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f" "nodecode" {
  %r = icmp ne <16 x i32> %n, %m
  ret <16 x i1> %r
}

define <16 x i1> @VPCMPD.slt.16xi32(<16 x i32> "reg"="zv" %n, <16 x i32> "reg"="zv" %m) "enc"="evexr(d, m, 3, 0, n, 2, 1, 0, 0, 0) | 00011111 | 11 reg:3=d&7 rm:3=m&7 | 00000001" "asm"="vpcmpd $1, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f" "nodecode" {
  %r = icmp slt <16 x i32> %n, %m
  ret <16 x i1> %r
}

define <16 x i1> @VPCMPD.sle.16xi32(<16 x i32> "reg"="zv" %n, <16 x i32> "reg"="zv" %m) "enc"="evexr(d, m, 3, 0, n, 2, 1, 0, 0, 0) | 00011111 | 11 reg:3=d&7 rm:3=m&7 | 00000010" "asm"="vpcmpd $2, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f" "nodecode" {
  %r = icmp sle <16 x i32> %n, %m
  ret <16 x i1> %r
}

define <16 x i1> @VPCMPD.sge.16xi32(<16 x i32> "reg"="zv" %n, <16 x i32> "reg"="zv" %m) "enc"="evexr(d, m, 3, 0, n, 2, 1, 0, 0, 0) | 00011111 | 11 reg:3=d&7 rm:3=m&7 | 00000101" "asm"="vpcmpd $5, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f" "nodecode" {
  %r = icmp sge <16 x i32> %n, %m
  ret <16 x i1> %r
}

define <16 x i1> @VPCMPD.sgt.16xi32(<16 x i32> "reg"="zv" %n, <16 x i32> "reg"="zv" %m) "enc"="evexr(d, m, 3, 0, n, 2, 1, 0, 0, 0) | 00011111 | 11 reg:3=d&7 rm:3=m&7 | 00000110" "asm"="vpcmpd $6, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f" "nodecode" {
  %r = icmp sgt <16 x i32> %n, %m
  ret <16 x i1> %r
}

define <16 x i1> @VPCMPUD.ult.16xi32(<16 x i32> "reg"="zv" %n, <16 x i32> "reg"="zv" %m) "enc"="evexr(d, m, 3, 0, n, 2, 1, 0, 0, 0) | 00011110 | 11 reg:3=d&7 rm:3=m&7 | 00000001" "asm"="vpcmpud $1, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f" "nodecode" {
  %r = icmp ult <16 x i32> %n, %m
  ret <16 x i1> %r
}

define <16 x i1> @VPCMPUD.ule.16xi32(<16 x i32> "reg"="zv" %n, <16 x i32> "reg"="zv" %m) "enc"="evexr(d, m, 3, 0, n, 2, 1, 0, 0, 0) | 00011110 | 11 reg:3=d&7 rm:3=m&7 | 00000010" "asm"="vpcmpud $2, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f" "nodecode" {
  %r = icmp ule <16 x i32> %n, %m
  ret <16 x i1> %r
}

define <16 x i1> @VPCMPUD.uge.16xi32(<16 x i32> "reg"="zv" %n, <16 x i32> "reg"="zv" %m) "enc"="evexr(d, m, 3, 0, n, 2, 1, 0, 0, 0) | 00011110 | 11 reg:3=d&7 rm:3=m&7 | 00000101" "asm"="vpcmpud $5, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f" "nodecode" {
  %r = icmp uge <16 x i32> %n, %m
  ret <16 x i1> %r
}

define <16 x i1> @VPCMPUD.ugt.16xi32(<16 x i32> "reg"="zv" %n, <16 x i32> "reg"="zv" %m) "enc"="evexr(d, m, 3, 0, n, 2, 1, 0, 0, 0) | 00011110 | 11 reg:3=d&7 rm:3=m&7 | 00000110" "asm"="vpcmpud $6, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f" "nodecode" {
  %r = icmp ugt <16 x i32> %n, %m
  ret <16 x i1> %r
}

define <16 x i32> @VPBLENDMD.16xi32(<16 x i1> "reg"="kw" %k, <16 x i32> "reg"="zv" %a, <16 x i32> "reg"="zv" %b) "enc"="{ if(k!=0) evexr(d, a, 2, 0, b, 2, 1, 0, k, 0) | 01100100 | 11 reg:3=d&7 rm:3=a&7 }" "asm"="vpblendmd {a}, {b}, {d} {lb}{k}{rb}" "dclass"="zv" "feature"="avx512f" "nodecode" {
  %r = select <16 x i1> %k, <16 x i32> %a, <16 x i32> %b
  ret <16 x i32> %r
}

define <16 x i32> @VPSLLDiz.16xi32(<16 x i32> "reg"="zv" %n, <16 x i32> "imm" %sh) "enc"="evexr(6, n, 1, 0, d, 2, 1, 0, 0, 0) | 01110010 | 11 110 rm:3=n&7 | i1:8=sh" "asm"="vpslld ${sh}, {n}, {d}" "dclass"="zv" "feature"="avx512f" "nodecode" {
  %r = shl <16 x i32> %n, %sh
  ret <16 x i32> %r
}

define <16 x i32> @VPSLLVDz.16xi32(<16 x i32> "reg"="zv" %n, <16 x i32> "reg"="zv" %m) "enc"="evexr(d, m, 2, 0, n, 2, 1, 0, 0, 0) | 01000111 | 11 reg:3=d&7 rm:3=m&7" "asm"="vpsllvd {m}, {n}, {d}" "dclass"="zv" "feature"="avx512f" "nodecode" {
  %r = shl <16 x i32> %n, %m
  ret <16 x i32> %r
}

define <16 x i32> @VPSRLDiz.16xi32(<16 x i32> "reg"="zv" %n, <16 x i32> "imm" %sh) "enc"="evexr(2, n, 1, 0, d, 2, 1, 0, 0, 0) | 01110010 | 11 010 rm:3=n&7 | i1:8=sh" "asm"="vpsrld ${sh}, {n}, {d}" "dclass"="zv" "feature"="avx512f" "nodecode" {
  %r = lshr <16 x i32> %n, %sh
  ret <16 x i32> %r
}

define <16 x i32> @VPSRLVDz.16xi32(<16 x i32> "reg"="zv" %n, <16 x i32> "reg"="zv" %m) "enc"="evexr(d, m, 2, 0, n, 2, 1, 0, 0, 0) | 01000101 | 11 reg:3=d&7 rm:3=m&7" "asm"="vpsrlvd {m}, {n}, {d}" "dclass"="zv" "feature"="avx512f" "nodecode" {
  %r = lshr <16 x i32> %n, %m
  ret <16 x i32> %r
}

define <16 x i32> @VPSRADiz.16xi32(<16 x i32> "reg"="zv" %n, <16 x i32> "imm" %sh) "enc"="evexr(4, n, 1, 0, d, 2, 1, 0, 0, 0) | 01110010 | 11 100 rm:3=n&7 | i1:8=sh" "asm"="vpsrad ${sh}, {n}, {d}" "dclass"="zv" "feature"="avx512f" "nodecode" {
  %r = ashr <16 x i32> %n, %sh
  ret <16 x i32> %r
}

define <16 x i32> @VPSRAVDz.16xi32(<16 x i32> "reg"="zv" %n, <16 x i32> "reg"="zv" %m) "enc"="evexr(d, m, 2, 0, n, 2, 1, 0, 0, 0) | 01000110 | 11 reg:3=d&7 rm:3=m&7" "asm"="vpsravd {m}, {n}, {d}" "dclass"="zv" "feature"="avx512f" "nodecode" {
  %r = ashr <16 x i32> %n, %m
  ret <16 x i32> %r
}

define <16 x i32> @VPMINSDz.16xi32(<16 x i32> "reg"="zv" %n, <16 x i32> "reg"="zv" %m) "enc"="evexr(d, m, 2, 0, n, 2, 1, 0, 0, 0) | 00111001 | 11 reg:3=d&7 rm:3=m&7" "asm"="vpminsd {m}, {n}, {d}" "dclass"="zv" "feature"="avx512f" "nodecode" {
  %r = call <16 x i32> @llvm.smin.v16i32(<16 x i32> %n, <16 x i32> %m)
  ret <16 x i32> %r
}

declare <16 x i32> @llvm.smin.v16i32(<16 x i32>, <16 x i32>)

define <16 x i32> @VPMAXSDz.16xi32(<16 x i32> "reg"="zv" %n, <16 x i32> "reg"="zv" %m) "enc"="evexr(d, m, 2, 0, n, 2, 1, 0, 0, 0) | 00111101 | 11 reg:3=d&7 rm:3=m&7" "asm"="vpmaxsd {m}, {n}, {d}" "dclass"="zv" "feature"="avx512f" "nodecode" {
  %r = call <16 x i32> @llvm.smax.v16i32(<16 x i32> %n, <16 x i32> %m)
  ret <16 x i32> %r
}

declare <16 x i32> @llvm.smax.v16i32(<16 x i32>, <16 x i32>)

define <16 x i32> @VPMINUDz.16xi32(<16 x i32> "reg"="zv" %n, <16 x i32> "reg"="zv" %m) "enc"="evexr(d, m, 2, 0, n, 2, 1, 0, 0, 0) | 00111011 | 11 reg:3=d&7 rm:3=m&7" "asm"="vpminud {m}, {n}, {d}" "dclass"="zv" "feature"="avx512f" "nodecode" {
  %r = call <16 x i32> @llvm.umin.v16i32(<16 x i32> %n, <16 x i32> %m)
  ret <16 x i32> %r
}

declare <16 x i32> @llvm.umin.v16i32(<16 x i32>, <16 x i32>)

define <16 x i32> @VPMAXUDz.16xi32(<16 x i32> "reg"="zv" %n, <16 x i32> "reg"="zv" %m) "enc"="evexr(d, m, 2, 0, n, 2, 1, 0, 0, 0) | 00111111 | 11 reg:3=d&7 rm:3=m&7" "asm"="vpmaxud {m}, {n}, {d}" "dclass"="zv" "feature"="avx512f" "nodecode" {
  %r = call <16 x i32> @llvm.umax.v16i32(<16 x i32> %n, <16 x i32> %m)
  ret <16 x i32> %r
}

declare <16 x i32> @llvm.umax.v16i32(<16 x i32>, <16 x i32>)

define <16 x i32> @VPABSDz.false.16xi32(<16 x i32> "reg"="zv" %n) "enc"="evexr(d, n, 2, 0, 0, 2, 1, 0, 0, 0) | 00011110 | 11 reg:3=d&7 rm:3=n&7" "asm"="vpabsd {n}, {d}" "dclass"="zv" "feature"="avx512f" "nodecode" {
  %r = call <16 x i32> @llvm.abs.v16i32(<16 x i32> %n, i1 false)
  ret <16 x i32> %r
}

define <16 x i32> @VPABSDz.true.16xi32(<16 x i32> "reg"="zv" %n) "enc"="evexr(d, n, 2, 0, 0, 2, 1, 0, 0, 0) | 00011110 | 11 reg:3=d&7 rm:3=n&7" "asm"="vpabsd {n}, {d}" "dclass"="zv" "feature"="avx512f" "nodecode" {
  %r = call <16 x i32> @llvm.abs.v16i32(<16 x i32> %n, i1 true)
  ret <16 x i32> %r
}

declare <16 x i32> @llvm.abs.v16i32(<16 x i32>, i1)

define <16 x i32> @VPMOVM2D.16xi32(<16 x i1> "reg"="kw" %k) "enc"="{ if(k!=0) evexr(d, d, 3, 0, d, 2, 1, 0, k, 1) | 00100101 | 11 reg:3=d&7 rm:3=d&7 | 11111111 }" "asm"="vpternlogd $255, {d}, {d}, {d} {lb}{k}{rb} {lb}z{rb}" "dclass"="zv" "feature"="avx512f" "nodecode" {
  %r = sext <16 x i1> %k to <16 x i32>
  ret <16 x i32> %r
}

define <16 x i32> @VMASKBITS.16xi32(<16 x i1> "reg"="kw" %k) "enc"="{ if(k!=0) evexr(d, d, 3, 0, d, 2, 1, 0, k, 1) | 00100101 | 11 reg:3=d&7 rm:3=d&7 | 11111111 }" "asm"="vpternlogd $255, {d}, {d}, {d} {lb}{k}{rb} {lb}z{rb}" "dclass"="zv" "feature"="avx512f" "nodecode" {
  %r = call <16 x i32> @llvm.epitome.mask.bits.v16i32(<16 x i1> %k)
  ret <16 x i32> %r
}

define <16 x i32> @zextk.16xi32(<16 x i1> "reg"="kw" %k) "recipe"="zextk.16xi32.expand" "feature"="avx512f" {
  %r = zext <16 x i1> %k to <16 x i32>
  ret <16 x i32> %r
}

define <16 x i32> @zextk.16xi32.expand(<16 x i1> %k) {
  %s = call <16 x i32> @VPMOVM2D.16xi32(<16 x i1> %k)
  %r = call <16 x i32> @VPABSDz.false.16xi32(<16 x i32> %s)
  ret <16 x i32> %r
}

define <16 x i1> @VBITSMASK.16xi32(<16 x i32> "reg"="zv" %n) "enc"="evexr(d, n, 2, 0, n, 2, 1, 0, 0, 0) | 00100111 | 11 reg:3=d&7 rm:3=n&7" "asm"="vptestmd {n}, {n}, {d}" "dclass"="kw" "feature"="avx512f" "nodecode" {
  %r = call <16 x i1> @llvm.epitome.bits.mask.v16i32(<16 x i32> %n)
  ret <16 x i1> %r
}

declare <16 x i32> @llvm.epitome.mask.bits.v16i32(<16 x i1>)
declare <16 x i1> @llvm.epitome.bits.mask.v16i32(<16 x i32>)

define <16 x i32> @VPBROADCASTDr.16xi32(i32 "reg"="r32" %x) "enc"="evexr(d, x, 2, 0, 0, 2, 1, 0, 0, 0) | 01111100 | 11 reg:3=d&7 rm:3=x&7" "asm"="vpbroadcastd {x}, {d}" "dclass"="zv" "role"="dup.zv.32" "feature"="avx512f" "nodecode" {
  %i = insertelement <16 x i32> poison, i32 %x, i64 0
  %r = shufflevector <16 x i32> %i, <16 x i32> poison, <16 x i32> zeroinitializer
  ret <16 x i32> %r
}

define <8 x i64> @VPADDQz.8xi64(<8 x i64> "reg"="zv" %n, <8 x i64> "reg"="zv" %m) "enc"="evexr(d, m, 1, 1, n, 2, 1, 0, 0, 0) | 11010100 | 11 reg:3=d&7 rm:3=m&7" "asm"="vpaddq {m}, {n}, {d}" "dclass"="zv" "feature"="avx512f" "nodecode" {
  %r = add <8 x i64> %n, %m
  ret <8 x i64> %r
}

define <8 x i64> @VPSUBQz.8xi64(<8 x i64> "reg"="zv" %n, <8 x i64> "reg"="zv" %m) "enc"="evexr(d, m, 1, 1, n, 2, 1, 0, 0, 0) | 11111011 | 11 reg:3=d&7 rm:3=m&7" "asm"="vpsubq {m}, {n}, {d}" "dclass"="zv" "feature"="avx512f" "nodecode" {
  %r = sub <8 x i64> %n, %m
  ret <8 x i64> %r
}

define <8 x i64> @VPMULLQz.8xi64(<8 x i64> "reg"="zv" %n, <8 x i64> "reg"="zv" %m) "enc"="evexr(d, m, 2, 1, n, 2, 1, 0, 0, 0) | 01000000 | 11 reg:3=d&7 rm:3=m&7" "asm"="vpmullq {m}, {n}, {d}" "dclass"="zv" "feature"="avx512f,avx512dq" "nodecode" {
  %r = mul <8 x i64> %n, %m
  ret <8 x i64> %r
}
define <4 x i64> @VPMULLQ.4xi64(<4 x i64> "reg"="yv" %n, <4 x i64> "reg"="yv" %m) "enc"="evexr(d, m, 2, 1, n, 1, 1, 0, 0, 0) | 01000000 | 11 reg:3=d&7 rm:3=m&7" "asm"="vpmullq {m}, {n}, {d}" "dclass"="yv" "feature"="avx512dq,avx512vl" "nodecode" {
  %r = mul <4 x i64> %n, %m
  ret <4 x i64> %r
}
define <2 x i64> @VPMULLQ.2xi64(<2 x i64> "reg"="xv" %n, <2 x i64> "reg"="xv" %m) "enc"="evexr(d, m, 2, 1, n, 0, 1, 0, 0, 0) | 01000000 | 11 reg:3=d&7 rm:3=m&7" "asm"="vpmullq {m}, {n}, {d}" "dclass"="xv" "feature"="avx512dq,avx512vl" "nodecode" {
  %r = mul <2 x i64> %n, %m
  ret <2 x i64> %r
}

define <8 x i64> @VPANDQz.8xi64(<8 x i64> "reg"="zv" %n, <8 x i64> "reg"="zv" %m) "enc"="evexr(d, m, 1, 1, n, 2, 1, 0, 0, 0) | 11011011 | 11 reg:3=d&7 rm:3=m&7" "asm"="vpandq {m}, {n}, {d}" "dclass"="zv" "feature"="avx512f" "nodecode" {
  %r = and <8 x i64> %n, %m
  ret <8 x i64> %r
}

define <8 x i64> @VPORQz.8xi64(<8 x i64> "reg"="zv" %n, <8 x i64> "reg"="zv" %m) "enc"="evexr(d, m, 1, 1, n, 2, 1, 0, 0, 0) | 11101011 | 11 reg:3=d&7 rm:3=m&7" "asm"="vporq {m}, {n}, {d}" "dclass"="zv" "feature"="avx512f" "nodecode" {
  %r = or <8 x i64> %n, %m
  ret <8 x i64> %r
}

define <8 x i64> @VPXORQz.8xi64(<8 x i64> "reg"="zv" %n, <8 x i64> "reg"="zv" %m) "enc"="evexr(d, m, 1, 1, n, 2, 1, 0, 0, 0) | 11101111 | 11 reg:3=d&7 rm:3=m&7" "asm"="vpxorq {m}, {n}, {d}" "dclass"="zv" "feature"="avx512f" "nodecode" {
  %r = xor <8 x i64> %n, %m
  ret <8 x i64> %r
}

define <8 x i64> @VPANDNQz.8xi64(<8 x i64> "reg"="zv" %n, <8 x i64> "reg"="zv" %m) "enc"="evexr(d, m, 1, 1, n, 2, 1, 0, 0, 0) | 11011111 | 11 reg:3=d&7 rm:3=m&7" "asm"="vpandnq {m}, {n}, {d}" "dclass"="zv" "feature"="avx512f" "nodecode" {
  %x = xor <8 x i64> %n, splat (i64 -1)
  %r = and <8 x i64> %x, %m
  ret <8 x i64> %r
}

define <8 x i1> @VPCMPQ.eq.8xi64(<8 x i64> "reg"="zv" %n, <8 x i64> "reg"="zv" %m) "enc"="evexr(d, m, 3, 1, n, 2, 1, 0, 0, 0) | 00011111 | 11 reg:3=d&7 rm:3=m&7 | 00000000" "asm"="vpcmpq $0, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f" "nodecode" {
  %r = icmp eq <8 x i64> %n, %m
  ret <8 x i1> %r
}

define <8 x i1> @VPCMPQ.ne.8xi64(<8 x i64> "reg"="zv" %n, <8 x i64> "reg"="zv" %m) "enc"="evexr(d, m, 3, 1, n, 2, 1, 0, 0, 0) | 00011111 | 11 reg:3=d&7 rm:3=m&7 | 00000100" "asm"="vpcmpq $4, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f" "nodecode" {
  %r = icmp ne <8 x i64> %n, %m
  ret <8 x i1> %r
}

define <8 x i1> @VPCMPQ.slt.8xi64(<8 x i64> "reg"="zv" %n, <8 x i64> "reg"="zv" %m) "enc"="evexr(d, m, 3, 1, n, 2, 1, 0, 0, 0) | 00011111 | 11 reg:3=d&7 rm:3=m&7 | 00000001" "asm"="vpcmpq $1, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f" "nodecode" {
  %r = icmp slt <8 x i64> %n, %m
  ret <8 x i1> %r
}

define <8 x i1> @VPCMPQ.sle.8xi64(<8 x i64> "reg"="zv" %n, <8 x i64> "reg"="zv" %m) "enc"="evexr(d, m, 3, 1, n, 2, 1, 0, 0, 0) | 00011111 | 11 reg:3=d&7 rm:3=m&7 | 00000010" "asm"="vpcmpq $2, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f" "nodecode" {
  %r = icmp sle <8 x i64> %n, %m
  ret <8 x i1> %r
}

define <8 x i1> @VPCMPQ.sge.8xi64(<8 x i64> "reg"="zv" %n, <8 x i64> "reg"="zv" %m) "enc"="evexr(d, m, 3, 1, n, 2, 1, 0, 0, 0) | 00011111 | 11 reg:3=d&7 rm:3=m&7 | 00000101" "asm"="vpcmpq $5, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f" "nodecode" {
  %r = icmp sge <8 x i64> %n, %m
  ret <8 x i1> %r
}

define <8 x i1> @VPCMPQ.sgt.8xi64(<8 x i64> "reg"="zv" %n, <8 x i64> "reg"="zv" %m) "enc"="evexr(d, m, 3, 1, n, 2, 1, 0, 0, 0) | 00011111 | 11 reg:3=d&7 rm:3=m&7 | 00000110" "asm"="vpcmpq $6, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f" "nodecode" {
  %r = icmp sgt <8 x i64> %n, %m
  ret <8 x i1> %r
}

define <8 x i1> @VPCMPUQ.ult.8xi64(<8 x i64> "reg"="zv" %n, <8 x i64> "reg"="zv" %m) "enc"="evexr(d, m, 3, 1, n, 2, 1, 0, 0, 0) | 00011110 | 11 reg:3=d&7 rm:3=m&7 | 00000001" "asm"="vpcmpuq $1, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f" "nodecode" {
  %r = icmp ult <8 x i64> %n, %m
  ret <8 x i1> %r
}

define <8 x i1> @VPCMPUQ.ule.8xi64(<8 x i64> "reg"="zv" %n, <8 x i64> "reg"="zv" %m) "enc"="evexr(d, m, 3, 1, n, 2, 1, 0, 0, 0) | 00011110 | 11 reg:3=d&7 rm:3=m&7 | 00000010" "asm"="vpcmpuq $2, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f" "nodecode" {
  %r = icmp ule <8 x i64> %n, %m
  ret <8 x i1> %r
}

define <8 x i1> @VPCMPUQ.uge.8xi64(<8 x i64> "reg"="zv" %n, <8 x i64> "reg"="zv" %m) "enc"="evexr(d, m, 3, 1, n, 2, 1, 0, 0, 0) | 00011110 | 11 reg:3=d&7 rm:3=m&7 | 00000101" "asm"="vpcmpuq $5, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f" "nodecode" {
  %r = icmp uge <8 x i64> %n, %m
  ret <8 x i1> %r
}

define <8 x i1> @VPCMPUQ.ugt.8xi64(<8 x i64> "reg"="zv" %n, <8 x i64> "reg"="zv" %m) "enc"="evexr(d, m, 3, 1, n, 2, 1, 0, 0, 0) | 00011110 | 11 reg:3=d&7 rm:3=m&7 | 00000110" "asm"="vpcmpuq $6, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f" "nodecode" {
  %r = icmp ugt <8 x i64> %n, %m
  ret <8 x i1> %r
}

define <8 x i64> @VPBLENDMQ.8xi64(<8 x i1> "reg"="kw" %k, <8 x i64> "reg"="zv" %a, <8 x i64> "reg"="zv" %b) "enc"="{ if(k!=0) evexr(d, a, 2, 1, b, 2, 1, 0, k, 0) | 01100100 | 11 reg:3=d&7 rm:3=a&7 }" "asm"="vpblendmq {a}, {b}, {d} {lb}{k}{rb}" "dclass"="zv" "feature"="avx512f" "nodecode" {
  %r = select <8 x i1> %k, <8 x i64> %a, <8 x i64> %b
  ret <8 x i64> %r
}

define <8 x i64> @VPSLLQiz.8xi64(<8 x i64> "reg"="zv" %n, <8 x i64> "imm" %sh) "enc"="evexr(6, n, 1, 1, d, 2, 1, 0, 0, 0) | 01110011 | 11 110 rm:3=n&7 | i1:8=sh" "asm"="vpsllq ${sh}, {n}, {d}" "dclass"="zv" "feature"="avx512f" "nodecode" {
  %r = shl <8 x i64> %n, %sh
  ret <8 x i64> %r
}

define <8 x i64> @VPSLLVQz.8xi64(<8 x i64> "reg"="zv" %n, <8 x i64> "reg"="zv" %m) "enc"="evexr(d, m, 2, 1, n, 2, 1, 0, 0, 0) | 01000111 | 11 reg:3=d&7 rm:3=m&7" "asm"="vpsllvq {m}, {n}, {d}" "dclass"="zv" "feature"="avx512f" "nodecode" {
  %r = shl <8 x i64> %n, %m
  ret <8 x i64> %r
}

define <8 x i64> @VPSRLQiz.8xi64(<8 x i64> "reg"="zv" %n, <8 x i64> "imm" %sh) "enc"="evexr(2, n, 1, 1, d, 2, 1, 0, 0, 0) | 01110011 | 11 010 rm:3=n&7 | i1:8=sh" "asm"="vpsrlq ${sh}, {n}, {d}" "dclass"="zv" "feature"="avx512f" "nodecode" {
  %r = lshr <8 x i64> %n, %sh
  ret <8 x i64> %r
}

define <8 x i64> @VPSRLVQz.8xi64(<8 x i64> "reg"="zv" %n, <8 x i64> "reg"="zv" %m) "enc"="evexr(d, m, 2, 1, n, 2, 1, 0, 0, 0) | 01000101 | 11 reg:3=d&7 rm:3=m&7" "asm"="vpsrlvq {m}, {n}, {d}" "dclass"="zv" "feature"="avx512f" "nodecode" {
  %r = lshr <8 x i64> %n, %m
  ret <8 x i64> %r
}

define <8 x i64> @VPSRAQiz.8xi64(<8 x i64> "reg"="zv" %n, <8 x i64> "imm" %sh) "enc"="evexr(4, n, 1, 1, d, 2, 1, 0, 0, 0) | 01110010 | 11 100 rm:3=n&7 | i1:8=sh" "asm"="vpsraq ${sh}, {n}, {d}" "dclass"="zv" "feature"="avx512f" "nodecode" {
  %r = ashr <8 x i64> %n, %sh
  ret <8 x i64> %r
}

define <8 x i64> @VPSRAVQz.8xi64(<8 x i64> "reg"="zv" %n, <8 x i64> "reg"="zv" %m) "enc"="evexr(d, m, 2, 1, n, 2, 1, 0, 0, 0) | 01000110 | 11 reg:3=d&7 rm:3=m&7" "asm"="vpsravq {m}, {n}, {d}" "dclass"="zv" "feature"="avx512f" "nodecode" {
  %r = ashr <8 x i64> %n, %m
  ret <8 x i64> %r
}

define <8 x i64> @VPMINSQz.8xi64(<8 x i64> "reg"="zv" %n, <8 x i64> "reg"="zv" %m) "enc"="evexr(d, m, 2, 1, n, 2, 1, 0, 0, 0) | 00111001 | 11 reg:3=d&7 rm:3=m&7" "asm"="vpminsq {m}, {n}, {d}" "dclass"="zv" "feature"="avx512f" "nodecode" {
  %r = call <8 x i64> @llvm.smin.v8i64(<8 x i64> %n, <8 x i64> %m)
  ret <8 x i64> %r
}

declare <8 x i64> @llvm.smin.v8i64(<8 x i64>, <8 x i64>)

define <8 x i64> @VPMAXSQz.8xi64(<8 x i64> "reg"="zv" %n, <8 x i64> "reg"="zv" %m) "enc"="evexr(d, m, 2, 1, n, 2, 1, 0, 0, 0) | 00111101 | 11 reg:3=d&7 rm:3=m&7" "asm"="vpmaxsq {m}, {n}, {d}" "dclass"="zv" "feature"="avx512f" "nodecode" {
  %r = call <8 x i64> @llvm.smax.v8i64(<8 x i64> %n, <8 x i64> %m)
  ret <8 x i64> %r
}

declare <8 x i64> @llvm.smax.v8i64(<8 x i64>, <8 x i64>)

define <8 x i64> @VPMINUQz.8xi64(<8 x i64> "reg"="zv" %n, <8 x i64> "reg"="zv" %m) "enc"="evexr(d, m, 2, 1, n, 2, 1, 0, 0, 0) | 00111011 | 11 reg:3=d&7 rm:3=m&7" "asm"="vpminuq {m}, {n}, {d}" "dclass"="zv" "feature"="avx512f" "nodecode" {
  %r = call <8 x i64> @llvm.umin.v8i64(<8 x i64> %n, <8 x i64> %m)
  ret <8 x i64> %r
}

declare <8 x i64> @llvm.umin.v8i64(<8 x i64>, <8 x i64>)

define <8 x i64> @VPMAXUQz.8xi64(<8 x i64> "reg"="zv" %n, <8 x i64> "reg"="zv" %m) "enc"="evexr(d, m, 2, 1, n, 2, 1, 0, 0, 0) | 00111111 | 11 reg:3=d&7 rm:3=m&7" "asm"="vpmaxuq {m}, {n}, {d}" "dclass"="zv" "feature"="avx512f" "nodecode" {
  %r = call <8 x i64> @llvm.umax.v8i64(<8 x i64> %n, <8 x i64> %m)
  ret <8 x i64> %r
}

declare <8 x i64> @llvm.umax.v8i64(<8 x i64>, <8 x i64>)

define <8 x i64> @VPABSQz.false.8xi64(<8 x i64> "reg"="zv" %n) "enc"="evexr(d, n, 2, 1, 0, 2, 1, 0, 0, 0) | 00011111 | 11 reg:3=d&7 rm:3=n&7" "asm"="vpabsq {n}, {d}" "dclass"="zv" "feature"="avx512f" "nodecode" {
  %r = call <8 x i64> @llvm.abs.v8i64(<8 x i64> %n, i1 false)
  ret <8 x i64> %r
}

define <8 x i64> @VPABSQz.true.8xi64(<8 x i64> "reg"="zv" %n) "enc"="evexr(d, n, 2, 1, 0, 2, 1, 0, 0, 0) | 00011111 | 11 reg:3=d&7 rm:3=n&7" "asm"="vpabsq {n}, {d}" "dclass"="zv" "feature"="avx512f" "nodecode" {
  %r = call <8 x i64> @llvm.abs.v8i64(<8 x i64> %n, i1 true)
  ret <8 x i64> %r
}

declare <8 x i64> @llvm.abs.v8i64(<8 x i64>, i1)

define <8 x i64> @VPMOVM2Q.8xi64(<8 x i1> "reg"="kw" %k) "enc"="{ if(k!=0) evexr(d, d, 3, 1, d, 2, 1, 0, k, 1) | 00100101 | 11 reg:3=d&7 rm:3=d&7 | 11111111 }" "asm"="vpternlogq $255, {d}, {d}, {d} {lb}{k}{rb} {lb}z{rb}" "dclass"="zv" "feature"="avx512f" "nodecode" {
  %r = sext <8 x i1> %k to <8 x i64>
  ret <8 x i64> %r
}

define <8 x i64> @VMASKBITS.8xi64(<8 x i1> "reg"="kw" %k) "enc"="{ if(k!=0) evexr(d, d, 3, 1, d, 2, 1, 0, k, 1) | 00100101 | 11 reg:3=d&7 rm:3=d&7 | 11111111 }" "asm"="vpternlogq $255, {d}, {d}, {d} {lb}{k}{rb} {lb}z{rb}" "dclass"="zv" "feature"="avx512f" "nodecode" {
  %r = call <8 x i64> @llvm.epitome.mask.bits.v8i64(<8 x i1> %k)
  ret <8 x i64> %r
}

define <8 x i64> @zextk.8xi64(<8 x i1> "reg"="kw" %k) "recipe"="zextk.8xi64.expand" "feature"="avx512f" {
  %r = zext <8 x i1> %k to <8 x i64>
  ret <8 x i64> %r
}

define <8 x i64> @zextk.8xi64.expand(<8 x i1> %k) {
  %s = call <8 x i64> @VPMOVM2Q.8xi64(<8 x i1> %k)
  %r = call <8 x i64> @VPABSQz.false.8xi64(<8 x i64> %s)
  ret <8 x i64> %r
}

define <8 x i1> @VBITSMASK.8xi64(<8 x i64> "reg"="zv" %n) "enc"="evexr(d, n, 2, 1, n, 2, 1, 0, 0, 0) | 00100111 | 11 reg:3=d&7 rm:3=n&7" "asm"="vptestmq {n}, {n}, {d}" "dclass"="kw" "feature"="avx512f" "nodecode" {
  %r = call <8 x i1> @llvm.epitome.bits.mask.v8i64(<8 x i64> %n)
  ret <8 x i1> %r
}

declare <8 x i64> @llvm.epitome.mask.bits.v8i64(<8 x i1>)
declare <8 x i1> @llvm.epitome.bits.mask.v8i64(<8 x i64>)

define <8 x i64> @VPBROADCASTQr.8xi64(i64 "reg"="r64" %x) "enc"="evexr(d, x, 2, 1, 0, 2, 1, 0, 0, 0) | 01111100 | 11 reg:3=d&7 rm:3=x&7" "asm"="vpbroadcastq {x}, {d}" "dclass"="zv" "role"="dup.zv.64" "feature"="avx512f" "nodecode" {
  %i = insertelement <8 x i64> poison, i64 %x, i64 0
  %r = shufflevector <8 x i64> %i, <8 x i64> poison, <8 x i32> zeroinitializer
  ret <8 x i64> %r
}

define <16 x i1> @VCMPOEQPSz.16xf32(<16 x float> "reg"="zv" %n, <16 x float> "reg"="zv" %m) "enc"="{ evexr(d, m, 1, 0, n, 2, 0, 0, 0, 0) | 11000010 | 11 reg:3=d&7 rm:3=m&7 | 00000000 ; if(((0&3)==0)|((0&3)==3)) evexr(d, n, 1, 0, m, 2, 0, 0, 0, 0) | 11000010 | 11 reg:3=d&7 rm:3=n&7 | 00000000 }" "asm"="vcmpps $0, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f" "nodecode" {
  %r = fcmp oeq <16 x float> %n, %m
  ret <16 x i1> %r
}

define <16 x i1> @VCMPOLTPSz.16xf32(<16 x float> "reg"="zv" %n, <16 x float> "reg"="zv" %m) "enc"="{ evexr(d, m, 1, 0, n, 2, 0, 0, 0, 0) | 11000010 | 11 reg:3=d&7 rm:3=m&7 | 00000001 ; if(((1&3)==0)|((1&3)==3)) evexr(d, n, 1, 0, m, 2, 0, 0, 0, 0) | 11000010 | 11 reg:3=d&7 rm:3=n&7 | 00000001 }" "asm"="vcmpps $1, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f" "nodecode" {
  %r = fcmp olt <16 x float> %n, %m
  ret <16 x i1> %r
}

define <16 x i1> @VCMPOLEPSz.16xf32(<16 x float> "reg"="zv" %n, <16 x float> "reg"="zv" %m) "enc"="{ evexr(d, m, 1, 0, n, 2, 0, 0, 0, 0) | 11000010 | 11 reg:3=d&7 rm:3=m&7 | 00000010 ; if(((2&3)==0)|((2&3)==3)) evexr(d, n, 1, 0, m, 2, 0, 0, 0, 0) | 11000010 | 11 reg:3=d&7 rm:3=n&7 | 00000010 }" "asm"="vcmpps $2, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f" "nodecode" {
  %r = fcmp ole <16 x float> %n, %m
  ret <16 x i1> %r
}

define <16 x i1> @VCMPUNOPSz.16xf32(<16 x float> "reg"="zv" %n, <16 x float> "reg"="zv" %m) "enc"="{ evexr(d, m, 1, 0, n, 2, 0, 0, 0, 0) | 11000010 | 11 reg:3=d&7 rm:3=m&7 | 00000011 ; if(((3&3)==0)|((3&3)==3)) evexr(d, n, 1, 0, m, 2, 0, 0, 0, 0) | 11000010 | 11 reg:3=d&7 rm:3=n&7 | 00000011 }" "asm"="vcmpps $3, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f" "nodecode" {
  %r = fcmp uno <16 x float> %n, %m
  ret <16 x i1> %r
}

define <16 x i1> @VCMPUNEPSz.16xf32(<16 x float> "reg"="zv" %n, <16 x float> "reg"="zv" %m) "enc"="{ evexr(d, m, 1, 0, n, 2, 0, 0, 0, 0) | 11000010 | 11 reg:3=d&7 rm:3=m&7 | 00000100 ; if(((4&3)==0)|((4&3)==3)) evexr(d, n, 1, 0, m, 2, 0, 0, 0, 0) | 11000010 | 11 reg:3=d&7 rm:3=n&7 | 00000100 }" "asm"="vcmpps $4, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f" "nodecode" {
  %r = fcmp une <16 x float> %n, %m
  ret <16 x i1> %r
}

define <16 x i1> @VCMPUGEPSz.16xf32(<16 x float> "reg"="zv" %n, <16 x float> "reg"="zv" %m) "enc"="{ evexr(d, m, 1, 0, n, 2, 0, 0, 0, 0) | 11000010 | 11 reg:3=d&7 rm:3=m&7 | 00000101 ; if(((5&3)==0)|((5&3)==3)) evexr(d, n, 1, 0, m, 2, 0, 0, 0, 0) | 11000010 | 11 reg:3=d&7 rm:3=n&7 | 00000101 }" "asm"="vcmpps $5, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f" "nodecode" {
  %r = fcmp uge <16 x float> %n, %m
  ret <16 x i1> %r
}

define <16 x i1> @VCMPUGTPSz.16xf32(<16 x float> "reg"="zv" %n, <16 x float> "reg"="zv" %m) "enc"="{ evexr(d, m, 1, 0, n, 2, 0, 0, 0, 0) | 11000010 | 11 reg:3=d&7 rm:3=m&7 | 00000110 ; if(((6&3)==0)|((6&3)==3)) evexr(d, n, 1, 0, m, 2, 0, 0, 0, 0) | 11000010 | 11 reg:3=d&7 rm:3=n&7 | 00000110 }" "asm"="vcmpps $6, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f" "nodecode" {
  %r = fcmp ugt <16 x float> %n, %m
  ret <16 x i1> %r
}

define <16 x i1> @VCMPORDPSz.16xf32(<16 x float> "reg"="zv" %n, <16 x float> "reg"="zv" %m) "enc"="{ evexr(d, m, 1, 0, n, 2, 0, 0, 0, 0) | 11000010 | 11 reg:3=d&7 rm:3=m&7 | 00000111 ; if(((7&3)==0)|((7&3)==3)) evexr(d, n, 1, 0, m, 2, 0, 0, 0, 0) | 11000010 | 11 reg:3=d&7 rm:3=n&7 | 00000111 }" "asm"="vcmpps $7, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f" "nodecode" {
  %r = fcmp ord <16 x float> %n, %m
  ret <16 x i1> %r
}

define <16 x i1> @VCMPUEQPSz.16xf32(<16 x float> "reg"="zv" %n, <16 x float> "reg"="zv" %m) "enc"="{ evexr(d, m, 1, 0, n, 2, 0, 0, 0, 0) | 11000010 | 11 reg:3=d&7 rm:3=m&7 | 00001000 ; if(((8&3)==0)|((8&3)==3)) evexr(d, n, 1, 0, m, 2, 0, 0, 0, 0) | 11000010 | 11 reg:3=d&7 rm:3=n&7 | 00001000 }" "asm"="vcmpps $8, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f" "nodecode" {
  %r = fcmp ueq <16 x float> %n, %m
  ret <16 x i1> %r
}

define <16 x i1> @VCMPULTPSz.16xf32(<16 x float> "reg"="zv" %n, <16 x float> "reg"="zv" %m) "enc"="{ evexr(d, m, 1, 0, n, 2, 0, 0, 0, 0) | 11000010 | 11 reg:3=d&7 rm:3=m&7 | 00001001 ; if(((9&3)==0)|((9&3)==3)) evexr(d, n, 1, 0, m, 2, 0, 0, 0, 0) | 11000010 | 11 reg:3=d&7 rm:3=n&7 | 00001001 }" "asm"="vcmpps $9, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f" "nodecode" {
  %r = fcmp ult <16 x float> %n, %m
  ret <16 x i1> %r
}

define <16 x i1> @VCMPULEPSz.16xf32(<16 x float> "reg"="zv" %n, <16 x float> "reg"="zv" %m) "enc"="{ evexr(d, m, 1, 0, n, 2, 0, 0, 0, 0) | 11000010 | 11 reg:3=d&7 rm:3=m&7 | 00001010 ; if(((10&3)==0)|((10&3)==3)) evexr(d, n, 1, 0, m, 2, 0, 0, 0, 0) | 11000010 | 11 reg:3=d&7 rm:3=n&7 | 00001010 }" "asm"="vcmpps $10, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f" "nodecode" {
  %r = fcmp ule <16 x float> %n, %m
  ret <16 x i1> %r
}

define <16 x i1> @VCMPONEPSz.16xf32(<16 x float> "reg"="zv" %n, <16 x float> "reg"="zv" %m) "enc"="{ evexr(d, m, 1, 0, n, 2, 0, 0, 0, 0) | 11000010 | 11 reg:3=d&7 rm:3=m&7 | 00001100 ; if(((12&3)==0)|((12&3)==3)) evexr(d, n, 1, 0, m, 2, 0, 0, 0, 0) | 11000010 | 11 reg:3=d&7 rm:3=n&7 | 00001100 }" "asm"="vcmpps $12, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f" "nodecode" {
  %r = fcmp one <16 x float> %n, %m
  ret <16 x i1> %r
}

define <16 x i1> @VCMPOGEPSz.16xf32(<16 x float> "reg"="zv" %n, <16 x float> "reg"="zv" %m) "enc"="{ evexr(d, m, 1, 0, n, 2, 0, 0, 0, 0) | 11000010 | 11 reg:3=d&7 rm:3=m&7 | 00001101 ; if(((13&3)==0)|((13&3)==3)) evexr(d, n, 1, 0, m, 2, 0, 0, 0, 0) | 11000010 | 11 reg:3=d&7 rm:3=n&7 | 00001101 }" "asm"="vcmpps $13, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f" "nodecode" {
  %r = fcmp oge <16 x float> %n, %m
  ret <16 x i1> %r
}

define <16 x i1> @VCMPOGTPSz.16xf32(<16 x float> "reg"="zv" %n, <16 x float> "reg"="zv" %m) "enc"="{ evexr(d, m, 1, 0, n, 2, 0, 0, 0, 0) | 11000010 | 11 reg:3=d&7 rm:3=m&7 | 00001110 ; if(((14&3)==0)|((14&3)==3)) evexr(d, n, 1, 0, m, 2, 0, 0, 0, 0) | 11000010 | 11 reg:3=d&7 rm:3=n&7 | 00001110 }" "asm"="vcmpps $14, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f" "nodecode" {
  %r = fcmp ogt <16 x float> %n, %m
  ret <16 x i1> %r
}

define <16 x float> @VBLENDMPS.16xf32(<16 x i1> "reg"="kw" %k, <16 x float> "reg"="zv" %a, <16 x float> "reg"="zv" %b) "enc"="{ if(k!=0) evexr(d, a, 2, 0, b, 2, 1, 0, k, 0) | 01100101 | 11 reg:3=d&7 rm:3=a&7 }" "asm"="vblendmps {a}, {b}, {d} {lb}{k}{rb}" "dclass"="zv" "feature"="avx512f" "nodecode" {
  %r = select <16 x i1> %k, <16 x float> %a, <16 x float> %b
  ret <16 x float> %r
}

define <8 x i1> @VCMPOEQPDz.8xf64(<8 x double> "reg"="zv" %n, <8 x double> "reg"="zv" %m) "enc"="{ evexr(d, m, 1, 1, n, 2, 1, 0, 0, 0) | 11000010 | 11 reg:3=d&7 rm:3=m&7 | 00000000 ; if(((0&3)==0)|((0&3)==3)) evexr(d, n, 1, 1, m, 2, 1, 0, 0, 0) | 11000010 | 11 reg:3=d&7 rm:3=n&7 | 00000000 }" "asm"="vcmppd $0, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f" "nodecode" {
  %r = fcmp oeq <8 x double> %n, %m
  ret <8 x i1> %r
}

define <8 x i1> @VCMPOLTPDz.8xf64(<8 x double> "reg"="zv" %n, <8 x double> "reg"="zv" %m) "enc"="{ evexr(d, m, 1, 1, n, 2, 1, 0, 0, 0) | 11000010 | 11 reg:3=d&7 rm:3=m&7 | 00000001 ; if(((1&3)==0)|((1&3)==3)) evexr(d, n, 1, 1, m, 2, 1, 0, 0, 0) | 11000010 | 11 reg:3=d&7 rm:3=n&7 | 00000001 }" "asm"="vcmppd $1, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f" "nodecode" {
  %r = fcmp olt <8 x double> %n, %m
  ret <8 x i1> %r
}

define <8 x i1> @VCMPOLEPDz.8xf64(<8 x double> "reg"="zv" %n, <8 x double> "reg"="zv" %m) "enc"="{ evexr(d, m, 1, 1, n, 2, 1, 0, 0, 0) | 11000010 | 11 reg:3=d&7 rm:3=m&7 | 00000010 ; if(((2&3)==0)|((2&3)==3)) evexr(d, n, 1, 1, m, 2, 1, 0, 0, 0) | 11000010 | 11 reg:3=d&7 rm:3=n&7 | 00000010 }" "asm"="vcmppd $2, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f" "nodecode" {
  %r = fcmp ole <8 x double> %n, %m
  ret <8 x i1> %r
}

define <8 x i1> @VCMPUNOPDz.8xf64(<8 x double> "reg"="zv" %n, <8 x double> "reg"="zv" %m) "enc"="{ evexr(d, m, 1, 1, n, 2, 1, 0, 0, 0) | 11000010 | 11 reg:3=d&7 rm:3=m&7 | 00000011 ; if(((3&3)==0)|((3&3)==3)) evexr(d, n, 1, 1, m, 2, 1, 0, 0, 0) | 11000010 | 11 reg:3=d&7 rm:3=n&7 | 00000011 }" "asm"="vcmppd $3, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f" "nodecode" {
  %r = fcmp uno <8 x double> %n, %m
  ret <8 x i1> %r
}

define <8 x i1> @VCMPUNEPDz.8xf64(<8 x double> "reg"="zv" %n, <8 x double> "reg"="zv" %m) "enc"="{ evexr(d, m, 1, 1, n, 2, 1, 0, 0, 0) | 11000010 | 11 reg:3=d&7 rm:3=m&7 | 00000100 ; if(((4&3)==0)|((4&3)==3)) evexr(d, n, 1, 1, m, 2, 1, 0, 0, 0) | 11000010 | 11 reg:3=d&7 rm:3=n&7 | 00000100 }" "asm"="vcmppd $4, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f" "nodecode" {
  %r = fcmp une <8 x double> %n, %m
  ret <8 x i1> %r
}

define <8 x i1> @VCMPUGEPDz.8xf64(<8 x double> "reg"="zv" %n, <8 x double> "reg"="zv" %m) "enc"="{ evexr(d, m, 1, 1, n, 2, 1, 0, 0, 0) | 11000010 | 11 reg:3=d&7 rm:3=m&7 | 00000101 ; if(((5&3)==0)|((5&3)==3)) evexr(d, n, 1, 1, m, 2, 1, 0, 0, 0) | 11000010 | 11 reg:3=d&7 rm:3=n&7 | 00000101 }" "asm"="vcmppd $5, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f" "nodecode" {
  %r = fcmp uge <8 x double> %n, %m
  ret <8 x i1> %r
}

define <8 x i1> @VCMPUGTPDz.8xf64(<8 x double> "reg"="zv" %n, <8 x double> "reg"="zv" %m) "enc"="{ evexr(d, m, 1, 1, n, 2, 1, 0, 0, 0) | 11000010 | 11 reg:3=d&7 rm:3=m&7 | 00000110 ; if(((6&3)==0)|((6&3)==3)) evexr(d, n, 1, 1, m, 2, 1, 0, 0, 0) | 11000010 | 11 reg:3=d&7 rm:3=n&7 | 00000110 }" "asm"="vcmppd $6, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f" "nodecode" {
  %r = fcmp ugt <8 x double> %n, %m
  ret <8 x i1> %r
}

define <8 x i1> @VCMPORDPDz.8xf64(<8 x double> "reg"="zv" %n, <8 x double> "reg"="zv" %m) "enc"="{ evexr(d, m, 1, 1, n, 2, 1, 0, 0, 0) | 11000010 | 11 reg:3=d&7 rm:3=m&7 | 00000111 ; if(((7&3)==0)|((7&3)==3)) evexr(d, n, 1, 1, m, 2, 1, 0, 0, 0) | 11000010 | 11 reg:3=d&7 rm:3=n&7 | 00000111 }" "asm"="vcmppd $7, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f" "nodecode" {
  %r = fcmp ord <8 x double> %n, %m
  ret <8 x i1> %r
}

define <8 x i1> @VCMPUEQPDz.8xf64(<8 x double> "reg"="zv" %n, <8 x double> "reg"="zv" %m) "enc"="{ evexr(d, m, 1, 1, n, 2, 1, 0, 0, 0) | 11000010 | 11 reg:3=d&7 rm:3=m&7 | 00001000 ; if(((8&3)==0)|((8&3)==3)) evexr(d, n, 1, 1, m, 2, 1, 0, 0, 0) | 11000010 | 11 reg:3=d&7 rm:3=n&7 | 00001000 }" "asm"="vcmppd $8, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f" "nodecode" {
  %r = fcmp ueq <8 x double> %n, %m
  ret <8 x i1> %r
}

define <8 x i1> @VCMPULTPDz.8xf64(<8 x double> "reg"="zv" %n, <8 x double> "reg"="zv" %m) "enc"="{ evexr(d, m, 1, 1, n, 2, 1, 0, 0, 0) | 11000010 | 11 reg:3=d&7 rm:3=m&7 | 00001001 ; if(((9&3)==0)|((9&3)==3)) evexr(d, n, 1, 1, m, 2, 1, 0, 0, 0) | 11000010 | 11 reg:3=d&7 rm:3=n&7 | 00001001 }" "asm"="vcmppd $9, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f" "nodecode" {
  %r = fcmp ult <8 x double> %n, %m
  ret <8 x i1> %r
}

define <8 x i1> @VCMPULEPDz.8xf64(<8 x double> "reg"="zv" %n, <8 x double> "reg"="zv" %m) "enc"="{ evexr(d, m, 1, 1, n, 2, 1, 0, 0, 0) | 11000010 | 11 reg:3=d&7 rm:3=m&7 | 00001010 ; if(((10&3)==0)|((10&3)==3)) evexr(d, n, 1, 1, m, 2, 1, 0, 0, 0) | 11000010 | 11 reg:3=d&7 rm:3=n&7 | 00001010 }" "asm"="vcmppd $10, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f" "nodecode" {
  %r = fcmp ule <8 x double> %n, %m
  ret <8 x i1> %r
}

define <8 x i1> @VCMPONEPDz.8xf64(<8 x double> "reg"="zv" %n, <8 x double> "reg"="zv" %m) "enc"="{ evexr(d, m, 1, 1, n, 2, 1, 0, 0, 0) | 11000010 | 11 reg:3=d&7 rm:3=m&7 | 00001100 ; if(((12&3)==0)|((12&3)==3)) evexr(d, n, 1, 1, m, 2, 1, 0, 0, 0) | 11000010 | 11 reg:3=d&7 rm:3=n&7 | 00001100 }" "asm"="vcmppd $12, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f" "nodecode" {
  %r = fcmp one <8 x double> %n, %m
  ret <8 x i1> %r
}

define <8 x i1> @VCMPOGEPDz.8xf64(<8 x double> "reg"="zv" %n, <8 x double> "reg"="zv" %m) "enc"="{ evexr(d, m, 1, 1, n, 2, 1, 0, 0, 0) | 11000010 | 11 reg:3=d&7 rm:3=m&7 | 00001101 ; if(((13&3)==0)|((13&3)==3)) evexr(d, n, 1, 1, m, 2, 1, 0, 0, 0) | 11000010 | 11 reg:3=d&7 rm:3=n&7 | 00001101 }" "asm"="vcmppd $13, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f" "nodecode" {
  %r = fcmp oge <8 x double> %n, %m
  ret <8 x i1> %r
}

define <8 x i1> @VCMPOGTPDz.8xf64(<8 x double> "reg"="zv" %n, <8 x double> "reg"="zv" %m) "enc"="{ evexr(d, m, 1, 1, n, 2, 1, 0, 0, 0) | 11000010 | 11 reg:3=d&7 rm:3=m&7 | 00001110 ; if(((14&3)==0)|((14&3)==3)) evexr(d, n, 1, 1, m, 2, 1, 0, 0, 0) | 11000010 | 11 reg:3=d&7 rm:3=n&7 | 00001110 }" "asm"="vcmppd $14, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f" "nodecode" {
  %r = fcmp ogt <8 x double> %n, %m
  ret <8 x i1> %r
}

define <8 x double> @VBLENDMPD.8xf64(<8 x i1> "reg"="kw" %k, <8 x double> "reg"="zv" %a, <8 x double> "reg"="zv" %b) "enc"="{ if(k!=0) evexr(d, a, 2, 1, b, 2, 1, 0, k, 0) | 01100101 | 11 reg:3=d&7 rm:3=a&7 }" "asm"="vblendmpd {a}, {b}, {d} {lb}{k}{rb}" "dclass"="zv" "feature"="avx512f" "nodecode" {
  %r = select <8 x i1> %k, <8 x double> %a, <8 x double> %b
  ret <8 x double> %r
}

define <2 x i1> @KMOVWkr.2(i32 "reg"="r32" %n) "enc"="vex(d, 0, n, 1, 0, 0, 0, 0) | 10010010 | 11 reg:3=d&7 rm:3=n&7" "asm"="kmovw {n}, {d}" "dclass"="kw" "feature"="avx512f" "nodecode" {
  %r = bitcast i32 %n to <2 x i1>
  ret <2 x i1> %r
}

define i2 @KMOVWrk.2(<2 x i1> "reg"="kw" %n) "enc"="vex(d, 0, n, 1, 0, 0, 0, 0) | 10010011 | 11 reg:3=d&7 rm:3=n&7" "asm"="kmovw {n}, {d}" "dclass"="r32" "feature"="avx512f" "nodecode" {
  %r = bitcast <2 x i1> %n to i2
  ret i2 %r
}

define <2 x i1> @KANDW.2(<2 x i1> "reg"="kw" %n, <2 x i1> "reg"="kw" %m) "enc"="vex(d, 0, m, 1, 0, n, 1, 0) | 01000001 | 11 reg:3=d&7 rm:3=m&7" "asm"="kandw {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f" "nodecode" {
  %r = and <2 x i1> %n, %m
  ret <2 x i1> %r
}

define <2 x i1> @KORW.2(<2 x i1> "reg"="kw" %n, <2 x i1> "reg"="kw" %m) "enc"="vex(d, 0, m, 1, 0, n, 1, 0) | 01000101 | 11 reg:3=d&7 rm:3=m&7" "asm"="korw {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f" "nodecode" {
  %r = or <2 x i1> %n, %m
  ret <2 x i1> %r
}

define <2 x i1> @KXORW.2(<2 x i1> "reg"="kw" %n, <2 x i1> "reg"="kw" %m) "enc"="vex(d, 0, m, 1, 0, n, 1, 0) | 01000111 | 11 reg:3=d&7 rm:3=m&7" "asm"="kxorw {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f" "nodecode" {
  %r = xor <2 x i1> %n, %m
  ret <2 x i1> %r
}

define <2 x i1> @KANDNW.2(<2 x i1> "reg"="kw" %n, <2 x i1> "reg"="kw" %m) "enc"="vex(d, 0, m, 1, 0, n, 1, 0) | 01000010 | 11 reg:3=d&7 rm:3=m&7" "asm"="kandnw {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f" "nodecode" {
  %x = xor <2 x i1> %n, splat (i1 true)
  %r = and <2 x i1> %x, %m
  ret <2 x i1> %r
}

define <2 x i1> @KNOTW.2(<2 x i1> "reg"="kw" %n) "enc"="vex(d, 0, n, 1, 0, 0, 0, 0) | 01000100 | 11 reg:3=d&7 rm:3=n&7" "asm"="knotw {n}, {d}" "dclass"="kw" "feature"="avx512f" "nodecode" {
  %r = xor <2 x i1> %n, splat (i1 true)
  ret <2 x i1> %r
}

define <4 x i1> @KMOVWkr.4(i32 "reg"="r32" %n) "enc"="vex(d, 0, n, 1, 0, 0, 0, 0) | 10010010 | 11 reg:3=d&7 rm:3=n&7" "asm"="kmovw {n}, {d}" "dclass"="kw" "feature"="avx512f" "nodecode" {
  %r = bitcast i32 %n to <4 x i1>
  ret <4 x i1> %r
}

define i4 @KMOVWrk.4(<4 x i1> "reg"="kw" %n) "enc"="vex(d, 0, n, 1, 0, 0, 0, 0) | 10010011 | 11 reg:3=d&7 rm:3=n&7" "asm"="kmovw {n}, {d}" "dclass"="r32" "feature"="avx512f" "nodecode" {
  %r = bitcast <4 x i1> %n to i4
  ret i4 %r
}

define <4 x i1> @KANDW.4(<4 x i1> "reg"="kw" %n, <4 x i1> "reg"="kw" %m) "enc"="vex(d, 0, m, 1, 0, n, 1, 0) | 01000001 | 11 reg:3=d&7 rm:3=m&7" "asm"="kandw {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f" "nodecode" {
  %r = and <4 x i1> %n, %m
  ret <4 x i1> %r
}

define <4 x i1> @KORW.4(<4 x i1> "reg"="kw" %n, <4 x i1> "reg"="kw" %m) "enc"="vex(d, 0, m, 1, 0, n, 1, 0) | 01000101 | 11 reg:3=d&7 rm:3=m&7" "asm"="korw {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f" "nodecode" {
  %r = or <4 x i1> %n, %m
  ret <4 x i1> %r
}

define <4 x i1> @KXORW.4(<4 x i1> "reg"="kw" %n, <4 x i1> "reg"="kw" %m) "enc"="vex(d, 0, m, 1, 0, n, 1, 0) | 01000111 | 11 reg:3=d&7 rm:3=m&7" "asm"="kxorw {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f" "nodecode" {
  %r = xor <4 x i1> %n, %m
  ret <4 x i1> %r
}

define <4 x i1> @KANDNW.4(<4 x i1> "reg"="kw" %n, <4 x i1> "reg"="kw" %m) "enc"="vex(d, 0, m, 1, 0, n, 1, 0) | 01000010 | 11 reg:3=d&7 rm:3=m&7" "asm"="kandnw {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f" "nodecode" {
  %x = xor <4 x i1> %n, splat (i1 true)
  %r = and <4 x i1> %x, %m
  ret <4 x i1> %r
}

define <4 x i1> @KNOTW.4(<4 x i1> "reg"="kw" %n) "enc"="vex(d, 0, n, 1, 0, 0, 0, 0) | 01000100 | 11 reg:3=d&7 rm:3=n&7" "asm"="knotw {n}, {d}" "dclass"="kw" "feature"="avx512f" "nodecode" {
  %r = xor <4 x i1> %n, splat (i1 true)
  ret <4 x i1> %r
}

define <8 x i1> @KMOVWkr.8(i8 "reg"="r32" %n) "enc"="vex(d, 0, n, 1, 0, 0, 0, 0) | 10010010 | 11 reg:3=d&7 rm:3=n&7" "asm"="kmovw {n}, {d}" "dclass"="kw" "feature"="avx512f" "nodecode" {
  %r = bitcast i8 %n to <8 x i1>
  ret <8 x i1> %r
}

define i8 @KMOVWrk.8(<8 x i1> "reg"="kw" %n) "enc"="vex(d, 0, n, 1, 0, 0, 0, 0) | 10010011 | 11 reg:3=d&7 rm:3=n&7" "asm"="kmovw {n}, {d}" "dclass"="r32" "feature"="avx512f" "nodecode" {
  %r = bitcast <8 x i1> %n to i8
  ret i8 %r
}

define <8 x i1> @KANDW.8(<8 x i1> "reg"="kw" %n, <8 x i1> "reg"="kw" %m) "enc"="vex(d, 0, m, 1, 0, n, 1, 0) | 01000001 | 11 reg:3=d&7 rm:3=m&7" "asm"="kandw {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f" "nodecode" {
  %r = and <8 x i1> %n, %m
  ret <8 x i1> %r
}

define <8 x i1> @KORW.8(<8 x i1> "reg"="kw" %n, <8 x i1> "reg"="kw" %m) "enc"="vex(d, 0, m, 1, 0, n, 1, 0) | 01000101 | 11 reg:3=d&7 rm:3=m&7" "asm"="korw {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f" "nodecode" {
  %r = or <8 x i1> %n, %m
  ret <8 x i1> %r
}

define <8 x i1> @KXORW.8(<8 x i1> "reg"="kw" %n, <8 x i1> "reg"="kw" %m) "enc"="vex(d, 0, m, 1, 0, n, 1, 0) | 01000111 | 11 reg:3=d&7 rm:3=m&7" "asm"="kxorw {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f" "nodecode" {
  %r = xor <8 x i1> %n, %m
  ret <8 x i1> %r
}

define <8 x i1> @KANDNW.8(<8 x i1> "reg"="kw" %n, <8 x i1> "reg"="kw" %m) "enc"="vex(d, 0, m, 1, 0, n, 1, 0) | 01000010 | 11 reg:3=d&7 rm:3=m&7" "asm"="kandnw {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f" "nodecode" {
  %x = xor <8 x i1> %n, splat (i1 true)
  %r = and <8 x i1> %x, %m
  ret <8 x i1> %r
}

define <8 x i1> @KNOTW.8(<8 x i1> "reg"="kw" %n) "enc"="vex(d, 0, n, 1, 0, 0, 0, 0) | 01000100 | 11 reg:3=d&7 rm:3=n&7" "asm"="knotw {n}, {d}" "dclass"="kw" "feature"="avx512f" "nodecode" {
  %r = xor <8 x i1> %n, splat (i1 true)
  ret <8 x i1> %r
}

define <16 x i1> @KMOVWkr.16(i16 "reg"="r32" %n) "enc"="vex(d, 0, n, 1, 0, 0, 0, 0) | 10010010 | 11 reg:3=d&7 rm:3=n&7" "asm"="kmovw {n}, {d}" "dclass"="kw" "feature"="avx512f" "nodecode" {
  %r = bitcast i16 %n to <16 x i1>
  ret <16 x i1> %r
}

define i16 @KMOVWrk.16(<16 x i1> "reg"="kw" %n) "enc"="vex(d, 0, n, 1, 0, 0, 0, 0) | 10010011 | 11 reg:3=d&7 rm:3=n&7" "asm"="kmovw {n}, {d}" "dclass"="r32" "zext"="16" "feature"="avx512f" "nodecode" {
  %r = bitcast <16 x i1> %n to i16
  ret i16 %r
}

define <16 x i1> @KANDW.16(<16 x i1> "reg"="kw" %n, <16 x i1> "reg"="kw" %m) "enc"="vex(d, 0, m, 1, 0, n, 1, 0) | 01000001 | 11 reg:3=d&7 rm:3=m&7" "asm"="kandw {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f" "nodecode" {
  %r = and <16 x i1> %n, %m
  ret <16 x i1> %r
}

define <16 x i1> @KORW.16(<16 x i1> "reg"="kw" %n, <16 x i1> "reg"="kw" %m) "enc"="vex(d, 0, m, 1, 0, n, 1, 0) | 01000101 | 11 reg:3=d&7 rm:3=m&7" "asm"="korw {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f" "nodecode" {
  %r = or <16 x i1> %n, %m
  ret <16 x i1> %r
}

define <16 x i1> @KXORW.16(<16 x i1> "reg"="kw" %n, <16 x i1> "reg"="kw" %m) "enc"="vex(d, 0, m, 1, 0, n, 1, 0) | 01000111 | 11 reg:3=d&7 rm:3=m&7" "asm"="kxorw {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f" "nodecode" {
  %r = xor <16 x i1> %n, %m
  ret <16 x i1> %r
}

define <16 x i1> @KANDNW.16(<16 x i1> "reg"="kw" %n, <16 x i1> "reg"="kw" %m) "enc"="vex(d, 0, m, 1, 0, n, 1, 0) | 01000010 | 11 reg:3=d&7 rm:3=m&7" "asm"="kandnw {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f" "nodecode" {
  %x = xor <16 x i1> %n, splat (i1 true)
  %r = and <16 x i1> %x, %m
  ret <16 x i1> %r
}

define <16 x i1> @KNOTW.16(<16 x i1> "reg"="kw" %n) "enc"="vex(d, 0, n, 1, 0, 0, 0, 0) | 01000100 | 11 reg:3=d&7 rm:3=n&7" "asm"="knotw {n}, {d}" "dclass"="kw" "feature"="avx512f" "nodecode" {
  %r = xor <16 x i1> %n, splat (i1 true)
  ret <16 x i1> %r
}

define <32 x i1> @KMOVDkr.32(i32 "reg"="r32" %n) "enc"="vex(d, 0, n, 1, 0, 0, 0, 3) | 10010010 | 11 reg:3=d&7 rm:3=n&7" "asm"="kmovd {n}, {d}" "dclass"="kw" "feature"="avx512f,avx512bw" "nodecode" {
  %r = bitcast i32 %n to <32 x i1>
  ret <32 x i1> %r
}

define i32 @KMOVDrk.32(<32 x i1> "reg"="kw" %n) "enc"="vex(d, 0, n, 1, 0, 0, 0, 3) | 10010011 | 11 reg:3=d&7 rm:3=n&7" "asm"="kmovd {n}, {d}" "dclass"="r32" "feature"="avx512f,avx512bw" "nodecode" {
  %r = bitcast <32 x i1> %n to i32
  ret i32 %r
}

define <32 x i1> @KANDD.32(<32 x i1> "reg"="kw" %n, <32 x i1> "reg"="kw" %m) "enc"="vex(d, 0, m, 1, 1, n, 1, 1) | 01000001 | 11 reg:3=d&7 rm:3=m&7" "asm"="kandd {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f,avx512bw" "nodecode" {
  %r = and <32 x i1> %n, %m
  ret <32 x i1> %r
}

define <32 x i1> @KORD.32(<32 x i1> "reg"="kw" %n, <32 x i1> "reg"="kw" %m) "enc"="vex(d, 0, m, 1, 1, n, 1, 1) | 01000101 | 11 reg:3=d&7 rm:3=m&7" "asm"="kord {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f,avx512bw" "nodecode" {
  %r = or <32 x i1> %n, %m
  ret <32 x i1> %r
}

define <32 x i1> @KXORD.32(<32 x i1> "reg"="kw" %n, <32 x i1> "reg"="kw" %m) "enc"="vex(d, 0, m, 1, 1, n, 1, 1) | 01000111 | 11 reg:3=d&7 rm:3=m&7" "asm"="kxord {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f,avx512bw" "nodecode" {
  %r = xor <32 x i1> %n, %m
  ret <32 x i1> %r
}

define <32 x i1> @KANDND.32(<32 x i1> "reg"="kw" %n, <32 x i1> "reg"="kw" %m) "enc"="vex(d, 0, m, 1, 1, n, 1, 1) | 01000010 | 11 reg:3=d&7 rm:3=m&7" "asm"="kandnd {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f,avx512bw" "nodecode" {
  %x = xor <32 x i1> %n, splat (i1 true)
  %r = and <32 x i1> %x, %m
  ret <32 x i1> %r
}

define <32 x i1> @KNOTD.32(<32 x i1> "reg"="kw" %n) "enc"="vex(d, 0, n, 1, 1, 0, 0, 1) | 01000100 | 11 reg:3=d&7 rm:3=n&7" "asm"="knotd {n}, {d}" "dclass"="kw" "feature"="avx512f,avx512bw" "nodecode" {
  %r = xor <32 x i1> %n, splat (i1 true)
  ret <32 x i1> %r
}

define <64 x i1> @KMOVQkr.64(i64 "reg"="r64" %n) "enc"="vex(d, 0, n, 1, 1, 0, 0, 3) | 10010010 | 11 reg:3=d&7 rm:3=n&7" "asm"="kmovq {n}, {d}" "dclass"="kw" "feature"="avx512f,avx512bw" "nodecode" {
  %r = bitcast i64 %n to <64 x i1>
  ret <64 x i1> %r
}

define i64 @KMOVQrk.64(<64 x i1> "reg"="kw" %n) "enc"="vex(d, 0, n, 1, 1, 0, 0, 3) | 10010011 | 11 reg:3=d&7 rm:3=n&7" "asm"="kmovq {n}, {d}" "dclass"="r64" "feature"="avx512f,avx512bw" "nodecode" {
  %r = bitcast <64 x i1> %n to i64
  ret i64 %r
}

define <64 x i1> @KANDQ.64(<64 x i1> "reg"="kw" %n, <64 x i1> "reg"="kw" %m) "enc"="vex(d, 0, m, 1, 1, n, 1, 0) | 01000001 | 11 reg:3=d&7 rm:3=m&7" "asm"="kandq {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f,avx512bw" "nodecode" {
  %r = and <64 x i1> %n, %m
  ret <64 x i1> %r
}

define <64 x i1> @KORQ.64(<64 x i1> "reg"="kw" %n, <64 x i1> "reg"="kw" %m) "enc"="vex(d, 0, m, 1, 1, n, 1, 0) | 01000101 | 11 reg:3=d&7 rm:3=m&7" "asm"="korq {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f,avx512bw" "nodecode" {
  %r = or <64 x i1> %n, %m
  ret <64 x i1> %r
}

define <64 x i1> @KXORQ.64(<64 x i1> "reg"="kw" %n, <64 x i1> "reg"="kw" %m) "enc"="vex(d, 0, m, 1, 1, n, 1, 0) | 01000111 | 11 reg:3=d&7 rm:3=m&7" "asm"="kxorq {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f,avx512bw" "nodecode" {
  %r = xor <64 x i1> %n, %m
  ret <64 x i1> %r
}

define <64 x i1> @KANDNQ.64(<64 x i1> "reg"="kw" %n, <64 x i1> "reg"="kw" %m) "enc"="vex(d, 0, m, 1, 1, n, 1, 0) | 01000010 | 11 reg:3=d&7 rm:3=m&7" "asm"="kandnq {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f,avx512bw" "nodecode" {
  %x = xor <64 x i1> %n, splat (i1 true)
  %r = and <64 x i1> %x, %m
  ret <64 x i1> %r
}

define <64 x i1> @KNOTQ.64(<64 x i1> "reg"="kw" %n) "enc"="vex(d, 0, n, 1, 1, 0, 0, 0) | 01000100 | 11 reg:3=d&7 rm:3=n&7" "asm"="knotq {n}, {d}" "dclass"="kw" "feature"="avx512f,avx512bw" "nodecode" {
  %r = xor <64 x i1> %n, splat (i1 true)
  ret <64 x i1> %r
}

define <16 x i32> @VPLZCNTD.16xi32(<16 x i32> "reg"="zv" %n) "enc"="evexr(d, n, 2, 0, 0, 2, 1, 0, 0, 0) | 01000100 | 11 reg:3=d&7 rm:3=n&7" "asm"="vplzcntd {n}, {d}" "dclass"="zv" "feature"="avx512f,avx512cd" "nodecode" {
  %r = call <16 x i32> @llvm.ctlz.v16i32(<16 x i32> %n, i1 false)
  ret <16 x i32> %r
}

declare <16 x i32> @llvm.ctlz.v16i32(<16 x i32>, i1)

define <16 x i32> @VPOPCNTD.16xi32(<16 x i32> "reg"="zv" %n) "enc"="evexr(d, n, 2, 0, 0, 2, 1, 0, 0, 0) | 01010101 | 11 reg:3=d&7 rm:3=n&7" "asm"="vpopcntd {n}, {d}" "dclass"="zv" "feature"="avx512f,avx512vpopcntdq" "nodecode" {
  %r = call <16 x i32> @llvm.ctpop.v16i32(<16 x i32> %n)
  ret <16 x i32> %r
}

declare <16 x i32> @llvm.ctpop.v16i32(<16 x i32>)

define <8 x i32> @VPLZCNTD.8xi32(<8 x i32> "reg"="yv" %n) "enc"="evexr(d, n, 2, 0, 0, 1, 1, 0, 0, 0) | 01000100 | 11 reg:3=d&7 rm:3=n&7" "asm"="vplzcntd {n}, {d}" "dclass"="yv" "feature"="avx512f,avx512cd,avx512vl" "nodecode" {
  %r = call <8 x i32> @llvm.ctlz.v8i32(<8 x i32> %n, i1 false)
  ret <8 x i32> %r
}

declare <8 x i32> @llvm.ctlz.v8i32(<8 x i32>, i1)

define <8 x i32> @VPOPCNTD.8xi32(<8 x i32> "reg"="yv" %n) "enc"="evexr(d, n, 2, 0, 0, 1, 1, 0, 0, 0) | 01010101 | 11 reg:3=d&7 rm:3=n&7" "asm"="vpopcntd {n}, {d}" "dclass"="yv" "feature"="avx512f,avx512vpopcntdq,avx512vl" "nodecode" {
  %r = call <8 x i32> @llvm.ctpop.v8i32(<8 x i32> %n)
  ret <8 x i32> %r
}

declare <8 x i32> @llvm.ctpop.v8i32(<8 x i32>)

define <4 x i32> @VPLZCNTD.4xi32(<4 x i32> "reg"="xv" %n) "enc"="evexr(d, n, 2, 0, 0, 0, 1, 0, 0, 0) | 01000100 | 11 reg:3=d&7 rm:3=n&7" "asm"="vplzcntd {n}, {d}" "dclass"="xv" "feature"="avx512f,avx512cd,avx512vl" "nodecode" {
  %r = call <4 x i32> @llvm.ctlz.v4i32(<4 x i32> %n, i1 false)
  ret <4 x i32> %r
}

declare <4 x i32> @llvm.ctlz.v4i32(<4 x i32>, i1)

define <4 x i32> @VPOPCNTD.4xi32(<4 x i32> "reg"="xv" %n) "enc"="evexr(d, n, 2, 0, 0, 0, 1, 0, 0, 0) | 01010101 | 11 reg:3=d&7 rm:3=n&7" "asm"="vpopcntd {n}, {d}" "dclass"="xv" "feature"="avx512f,avx512vpopcntdq,avx512vl" "nodecode" {
  %r = call <4 x i32> @llvm.ctpop.v4i32(<4 x i32> %n)
  ret <4 x i32> %r
}

declare <4 x i32> @llvm.ctpop.v4i32(<4 x i32>)

define <8 x i64> @VPLZCNTQ.8xi64(<8 x i64> "reg"="zv" %n) "enc"="evexr(d, n, 2, 1, 0, 2, 1, 0, 0, 0) | 01000100 | 11 reg:3=d&7 rm:3=n&7" "asm"="vplzcntq {n}, {d}" "dclass"="zv" "feature"="avx512f,avx512cd" "nodecode" {
  %r = call <8 x i64> @llvm.ctlz.v8i64(<8 x i64> %n, i1 false)
  ret <8 x i64> %r
}

declare <8 x i64> @llvm.ctlz.v8i64(<8 x i64>, i1)

define <8 x i64> @VPOPCNTQ.8xi64(<8 x i64> "reg"="zv" %n) "enc"="evexr(d, n, 2, 1, 0, 2, 1, 0, 0, 0) | 01010101 | 11 reg:3=d&7 rm:3=n&7" "asm"="vpopcntq {n}, {d}" "dclass"="zv" "feature"="avx512f,avx512vpopcntdq" "nodecode" {
  %r = call <8 x i64> @llvm.ctpop.v8i64(<8 x i64> %n)
  ret <8 x i64> %r
}

declare <8 x i64> @llvm.ctpop.v8i64(<8 x i64>)

define <4 x i64> @VPLZCNTQ.4xi64(<4 x i64> "reg"="yv" %n) "enc"="evexr(d, n, 2, 1, 0, 1, 1, 0, 0, 0) | 01000100 | 11 reg:3=d&7 rm:3=n&7" "asm"="vplzcntq {n}, {d}" "dclass"="yv" "feature"="avx512f,avx512cd,avx512vl" "nodecode" {
  %r = call <4 x i64> @llvm.ctlz.v4i64(<4 x i64> %n, i1 false)
  ret <4 x i64> %r
}

declare <4 x i64> @llvm.ctlz.v4i64(<4 x i64>, i1)

define <4 x i64> @VPOPCNTQ.4xi64(<4 x i64> "reg"="yv" %n) "enc"="evexr(d, n, 2, 1, 0, 1, 1, 0, 0, 0) | 01010101 | 11 reg:3=d&7 rm:3=n&7" "asm"="vpopcntq {n}, {d}" "dclass"="yv" "feature"="avx512f,avx512vpopcntdq,avx512vl" "nodecode" {
  %r = call <4 x i64> @llvm.ctpop.v4i64(<4 x i64> %n)
  ret <4 x i64> %r
}

declare <4 x i64> @llvm.ctpop.v4i64(<4 x i64>)

define <2 x i64> @VPLZCNTQ.2xi64(<2 x i64> "reg"="xv" %n) "enc"="evexr(d, n, 2, 1, 0, 0, 1, 0, 0, 0) | 01000100 | 11 reg:3=d&7 rm:3=n&7" "asm"="vplzcntq {n}, {d}" "dclass"="xv" "feature"="avx512f,avx512cd,avx512vl" "nodecode" {
  %r = call <2 x i64> @llvm.ctlz.v2i64(<2 x i64> %n, i1 false)
  ret <2 x i64> %r
}

declare <2 x i64> @llvm.ctlz.v2i64(<2 x i64>, i1)

define <2 x i64> @VPOPCNTQ.2xi64(<2 x i64> "reg"="xv" %n) "enc"="evexr(d, n, 2, 1, 0, 0, 1, 0, 0, 0) | 01010101 | 11 reg:3=d&7 rm:3=n&7" "asm"="vpopcntq {n}, {d}" "dclass"="xv" "feature"="avx512f,avx512vpopcntdq,avx512vl" "nodecode" {
  %r = call <2 x i64> @llvm.ctpop.v2i64(<2 x i64> %n)
  ret <2 x i64> %r
}

declare <2 x i64> @llvm.ctpop.v2i64(<2 x i64>)

define <32 x i8> @VLOW.64xi8(<64 x i8> "reg"="zv" %v) "enc"="{ evexr(d, v, 1, 1, 0, 1, 1, 0, 0, 0) | 01101111 | 11 reg:3=d&7 rm:3=v&7 ; evexr(v, d, 1, 1, 0, 1, 1, 0, 0, 0) | 01111111 | 11 reg:3=v&7 rm:3=d&7 }" "asm"="vmovdqa64 {v:ye}, {d}" "dclass"="yv" "feature"="avx512f" "nodecode" {
  %r = shufflevector <64 x i8> %v, <64 x i8> poison, <32 x i32> <i32 0, i32 1, i32 2, i32 3, i32 4, i32 5, i32 6, i32 7, i32 8, i32 9, i32 10, i32 11, i32 12, i32 13, i32 14, i32 15, i32 16, i32 17, i32 18, i32 19, i32 20, i32 21, i32 22, i32 23, i32 24, i32 25, i32 26, i32 27, i32 28, i32 29, i32 30, i32 31>
  ret <32 x i8> %r
}

define <32 x i8> @VHIGH.64xi8(<64 x i8> "reg"="zv" %v) "enc"="evexr(v, d, 3, 1, 0, 2, 1, 0, 0, 0) | 00111011 | 11 reg:3=v&7 rm:3=d&7 | 00000001" "asm"="vextracti64x4 $1, {v}, {d}" "dclass"="yv" "feature"="avx512f" "nodecode" {
  %r = shufflevector <64 x i8> %v, <64 x i8> poison, <32 x i32> <i32 32, i32 33, i32 34, i32 35, i32 36, i32 37, i32 38, i32 39, i32 40, i32 41, i32 42, i32 43, i32 44, i32 45, i32 46, i32 47, i32 48, i32 49, i32 50, i32 51, i32 52, i32 53, i32 54, i32 55, i32 56, i32 57, i32 58, i32 59, i32 60, i32 61, i32 62, i32 63>
  ret <32 x i8> %r
}

define <64 x i8> @VJOIN.64xi8(<32 x i8> "reg"="yv" %a, <32 x i8> "reg"="yv" %b) "enc"="evexr(d, b, 3, 1, a, 2, 1, 0, 0, 0) | 00111010 | 11 reg:3=d&7 rm:3=b&7 | 00000001" "asm"="vinserti64x4 $1, {b}, {a:z}, {d}" "dclass"="zv" "feature"="avx512f" "nodecode" {
  %r = shufflevector <32 x i8> %a, <32 x i8> %b, <64 x i32> <i32 0, i32 1, i32 2, i32 3, i32 4, i32 5, i32 6, i32 7, i32 8, i32 9, i32 10, i32 11, i32 12, i32 13, i32 14, i32 15, i32 16, i32 17, i32 18, i32 19, i32 20, i32 21, i32 22, i32 23, i32 24, i32 25, i32 26, i32 27, i32 28, i32 29, i32 30, i32 31, i32 32, i32 33, i32 34, i32 35, i32 36, i32 37, i32 38, i32 39, i32 40, i32 41, i32 42, i32 43, i32 44, i32 45, i32 46, i32 47, i32 48, i32 49, i32 50, i32 51, i32 52, i32 53, i32 54, i32 55, i32 56, i32 57, i32 58, i32 59, i32 60, i32 61, i32 62, i32 63>
  ret <64 x i8> %r
}

define <16 x i16> @VLOW.32xi16(<32 x i16> "reg"="zv" %v) "enc"="{ evexr(d, v, 1, 1, 0, 1, 1, 0, 0, 0) | 01101111 | 11 reg:3=d&7 rm:3=v&7 ; evexr(v, d, 1, 1, 0, 1, 1, 0, 0, 0) | 01111111 | 11 reg:3=v&7 rm:3=d&7 }" "asm"="vmovdqa64 {v:ye}, {d}" "dclass"="yv" "feature"="avx512f" "nodecode" {
  %r = shufflevector <32 x i16> %v, <32 x i16> poison, <16 x i32> <i32 0, i32 1, i32 2, i32 3, i32 4, i32 5, i32 6, i32 7, i32 8, i32 9, i32 10, i32 11, i32 12, i32 13, i32 14, i32 15>
  ret <16 x i16> %r
}

define <16 x i16> @VHIGH.32xi16(<32 x i16> "reg"="zv" %v) "enc"="evexr(v, d, 3, 1, 0, 2, 1, 0, 0, 0) | 00111011 | 11 reg:3=v&7 rm:3=d&7 | 00000001" "asm"="vextracti64x4 $1, {v}, {d}" "dclass"="yv" "feature"="avx512f" "nodecode" {
  %r = shufflevector <32 x i16> %v, <32 x i16> poison, <16 x i32> <i32 16, i32 17, i32 18, i32 19, i32 20, i32 21, i32 22, i32 23, i32 24, i32 25, i32 26, i32 27, i32 28, i32 29, i32 30, i32 31>
  ret <16 x i16> %r
}

define <32 x i16> @VJOIN.32xi16(<16 x i16> "reg"="yv" %a, <16 x i16> "reg"="yv" %b) "enc"="evexr(d, b, 3, 1, a, 2, 1, 0, 0, 0) | 00111010 | 11 reg:3=d&7 rm:3=b&7 | 00000001" "asm"="vinserti64x4 $1, {b}, {a:z}, {d}" "dclass"="zv" "feature"="avx512f" "nodecode" {
  %r = shufflevector <16 x i16> %a, <16 x i16> %b, <32 x i32> <i32 0, i32 1, i32 2, i32 3, i32 4, i32 5, i32 6, i32 7, i32 8, i32 9, i32 10, i32 11, i32 12, i32 13, i32 14, i32 15, i32 16, i32 17, i32 18, i32 19, i32 20, i32 21, i32 22, i32 23, i32 24, i32 25, i32 26, i32 27, i32 28, i32 29, i32 30, i32 31>
  ret <32 x i16> %r
}

define <8 x i32> @VLOW.16xi32(<16 x i32> "reg"="zv" %v) "enc"="{ evexr(d, v, 1, 1, 0, 1, 1, 0, 0, 0) | 01101111 | 11 reg:3=d&7 rm:3=v&7 ; evexr(v, d, 1, 1, 0, 1, 1, 0, 0, 0) | 01111111 | 11 reg:3=v&7 rm:3=d&7 }" "asm"="vmovdqa64 {v:ye}, {d}" "dclass"="yv" "feature"="avx512f" "nodecode" {
  %r = shufflevector <16 x i32> %v, <16 x i32> poison, <8 x i32> <i32 0, i32 1, i32 2, i32 3, i32 4, i32 5, i32 6, i32 7>
  ret <8 x i32> %r
}

define <8 x i32> @VHIGH.16xi32(<16 x i32> "reg"="zv" %v) "enc"="evexr(v, d, 3, 1, 0, 2, 1, 0, 0, 0) | 00111011 | 11 reg:3=v&7 rm:3=d&7 | 00000001" "asm"="vextracti64x4 $1, {v}, {d}" "dclass"="yv" "feature"="avx512f" "nodecode" {
  %r = shufflevector <16 x i32> %v, <16 x i32> poison, <8 x i32> <i32 8, i32 9, i32 10, i32 11, i32 12, i32 13, i32 14, i32 15>
  ret <8 x i32> %r
}

define <16 x i32> @VJOIN.16xi32(<8 x i32> "reg"="yv" %a, <8 x i32> "reg"="yv" %b) "enc"="evexr(d, b, 3, 1, a, 2, 1, 0, 0, 0) | 00111010 | 11 reg:3=d&7 rm:3=b&7 | 00000001" "asm"="vinserti64x4 $1, {b}, {a:z}, {d}" "dclass"="zv" "feature"="avx512f" "nodecode" {
  %r = shufflevector <8 x i32> %a, <8 x i32> %b, <16 x i32> <i32 0, i32 1, i32 2, i32 3, i32 4, i32 5, i32 6, i32 7, i32 8, i32 9, i32 10, i32 11, i32 12, i32 13, i32 14, i32 15>
  ret <16 x i32> %r
}

define <4 x i64> @VLOW.8xi64(<8 x i64> "reg"="zv" %v) "enc"="{ evexr(d, v, 1, 1, 0, 1, 1, 0, 0, 0) | 01101111 | 11 reg:3=d&7 rm:3=v&7 ; evexr(v, d, 1, 1, 0, 1, 1, 0, 0, 0) | 01111111 | 11 reg:3=v&7 rm:3=d&7 }" "asm"="vmovdqa64 {v:ye}, {d}" "dclass"="yv" "feature"="avx512f" "nodecode" {
  %r = shufflevector <8 x i64> %v, <8 x i64> poison, <4 x i32> <i32 0, i32 1, i32 2, i32 3>
  ret <4 x i64> %r
}

define <4 x i64> @VHIGH.8xi64(<8 x i64> "reg"="zv" %v) "enc"="evexr(v, d, 3, 1, 0, 2, 1, 0, 0, 0) | 00111011 | 11 reg:3=v&7 rm:3=d&7 | 00000001" "asm"="vextracti64x4 $1, {v}, {d}" "dclass"="yv" "feature"="avx512f" "nodecode" {
  %r = shufflevector <8 x i64> %v, <8 x i64> poison, <4 x i32> <i32 4, i32 5, i32 6, i32 7>
  ret <4 x i64> %r
}

define <8 x i64> @VJOIN.8xi64(<4 x i64> "reg"="yv" %a, <4 x i64> "reg"="yv" %b) "enc"="evexr(d, b, 3, 1, a, 2, 1, 0, 0, 0) | 00111010 | 11 reg:3=d&7 rm:3=b&7 | 00000001" "asm"="vinserti64x4 $1, {b}, {a:z}, {d}" "dclass"="zv" "feature"="avx512f" "nodecode" {
  %r = shufflevector <4 x i64> %a, <4 x i64> %b, <8 x i32> <i32 0, i32 1, i32 2, i32 3, i32 4, i32 5, i32 6, i32 7>
  ret <8 x i64> %r
}

define <8 x float> @VLOW.16xf32(<16 x float> "reg"="zv" %v) "enc"="{ evexr(d, v, 1, 1, 0, 1, 1, 0, 0, 0) | 01101111 | 11 reg:3=d&7 rm:3=v&7 ; evexr(v, d, 1, 1, 0, 1, 1, 0, 0, 0) | 01111111 | 11 reg:3=v&7 rm:3=d&7 }" "asm"="vmovdqa64 {v:ye}, {d}" "dclass"="yv" "feature"="avx512f" "nodecode" {
  %r = shufflevector <16 x float> %v, <16 x float> poison, <8 x i32> <i32 0, i32 1, i32 2, i32 3, i32 4, i32 5, i32 6, i32 7>
  ret <8 x float> %r
}

define <8 x float> @VHIGH.16xf32(<16 x float> "reg"="zv" %v) "enc"="evexr(v, d, 3, 1, 0, 2, 1, 0, 0, 0) | 00111011 | 11 reg:3=v&7 rm:3=d&7 | 00000001" "asm"="vextracti64x4 $1, {v}, {d}" "dclass"="yv" "feature"="avx512f" "nodecode" {
  %r = shufflevector <16 x float> %v, <16 x float> poison, <8 x i32> <i32 8, i32 9, i32 10, i32 11, i32 12, i32 13, i32 14, i32 15>
  ret <8 x float> %r
}

define <16 x float> @VJOIN.16xf32(<8 x float> "reg"="yv" %a, <8 x float> "reg"="yv" %b) "enc"="evexr(d, b, 3, 1, a, 2, 1, 0, 0, 0) | 00111010 | 11 reg:3=d&7 rm:3=b&7 | 00000001" "asm"="vinserti64x4 $1, {b}, {a:z}, {d}" "dclass"="zv" "feature"="avx512f" "nodecode" {
  %r = shufflevector <8 x float> %a, <8 x float> %b, <16 x i32> <i32 0, i32 1, i32 2, i32 3, i32 4, i32 5, i32 6, i32 7, i32 8, i32 9, i32 10, i32 11, i32 12, i32 13, i32 14, i32 15>
  ret <16 x float> %r
}

define <4 x double> @VLOW.8xf64(<8 x double> "reg"="zv" %v) "enc"="{ evexr(d, v, 1, 1, 0, 1, 1, 0, 0, 0) | 01101111 | 11 reg:3=d&7 rm:3=v&7 ; evexr(v, d, 1, 1, 0, 1, 1, 0, 0, 0) | 01111111 | 11 reg:3=v&7 rm:3=d&7 }" "asm"="vmovdqa64 {v:ye}, {d}" "dclass"="yv" "feature"="avx512f" "nodecode" {
  %r = shufflevector <8 x double> %v, <8 x double> poison, <4 x i32> <i32 0, i32 1, i32 2, i32 3>
  ret <4 x double> %r
}

define <4 x double> @VHIGH.8xf64(<8 x double> "reg"="zv" %v) "enc"="evexr(v, d, 3, 1, 0, 2, 1, 0, 0, 0) | 00111011 | 11 reg:3=v&7 rm:3=d&7 | 00000001" "asm"="vextracti64x4 $1, {v}, {d}" "dclass"="yv" "feature"="avx512f" "nodecode" {
  %r = shufflevector <8 x double> %v, <8 x double> poison, <4 x i32> <i32 4, i32 5, i32 6, i32 7>
  ret <4 x double> %r
}

define <8 x double> @VJOIN.8xf64(<4 x double> "reg"="yv" %a, <4 x double> "reg"="yv" %b) "enc"="evexr(d, b, 3, 1, a, 2, 1, 0, 0, 0) | 00111010 | 11 reg:3=d&7 rm:3=b&7 | 00000001" "asm"="vinserti64x4 $1, {b}, {a:z}, {d}" "dclass"="zv" "feature"="avx512f" "nodecode" {
  %r = shufflevector <4 x double> %a, <4 x double> %b, <8 x i32> <i32 0, i32 1, i32 2, i32 3, i32 4, i32 5, i32 6, i32 7>
  ret <8 x double> %r
}

define <8 x i64> @VPMOVSXDQz(<8 x i32> "reg"="yv" %n) "enc"="evexr(d, n, 2, 0, 0, 2, 1, 0, 0, 0) | 00100101 | 11 reg:3=d&7 rm:3=n&7" "asm"="vpmovsxdq {n}, {d}" "dclass"="zv" "feature"="avx512f" "nodecode" {
  %r = sext <8 x i32> %n to <8 x i64>
  ret <8 x i64> %r
}

define <8 x i64> @VPMOVZXDQz(<8 x i32> "reg"="yv" %n) "enc"="evexr(d, n, 2, 0, 0, 2, 1, 0, 0, 0) | 00110101 | 11 reg:3=d&7 rm:3=n&7" "asm"="vpmovzxdq {n}, {d}" "dclass"="zv" "feature"="avx512f" "nodecode" {
  %r = zext <8 x i32> %n to <8 x i64>
  ret <8 x i64> %r
}

define <8 x i32> @VPMOVQDz(<8 x i64> "reg"="zv" %n) "enc"="evexr(n, d, 2, 0, 0, 2, 2, 0, 0, 0) | 00110101 | 11 reg:3=n&7 rm:3=d&7" "asm"="vpmovqd {n}, {d}" "dclass"="yv" "feature"="avx512f" "nodecode" {
  %r = trunc <8 x i64> %n to <8 x i32>
  ret <8 x i32> %r
}

define <32 x i1> @VPCMPB.eq.32xi8(<32 x i8> "reg"="yv" %n, <32 x i8> "reg"="yv" %m) "enc"="evexr(d, m, 3, 0, n, 1, 1, 0, 0, 0) | 00111111 | 11 reg:3=d&7 rm:3=m&7 | 00000000" "asm"="vpcmpb $0, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f,avx512vl,avx512bw" "nodecode" {
  %r = icmp eq <32 x i8> %n, %m
  ret <32 x i1> %r
}

define <32 x i1> @VPCMPB.ne.32xi8(<32 x i8> "reg"="yv" %n, <32 x i8> "reg"="yv" %m) "enc"="evexr(d, m, 3, 0, n, 1, 1, 0, 0, 0) | 00111111 | 11 reg:3=d&7 rm:3=m&7 | 00000100" "asm"="vpcmpb $4, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f,avx512vl,avx512bw" "nodecode" {
  %r = icmp ne <32 x i8> %n, %m
  ret <32 x i1> %r
}

define <32 x i1> @VPCMPB.slt.32xi8(<32 x i8> "reg"="yv" %n, <32 x i8> "reg"="yv" %m) "enc"="evexr(d, m, 3, 0, n, 1, 1, 0, 0, 0) | 00111111 | 11 reg:3=d&7 rm:3=m&7 | 00000001" "asm"="vpcmpb $1, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f,avx512vl,avx512bw" "nodecode" {
  %r = icmp slt <32 x i8> %n, %m
  ret <32 x i1> %r
}

define <32 x i1> @VPCMPB.sle.32xi8(<32 x i8> "reg"="yv" %n, <32 x i8> "reg"="yv" %m) "enc"="evexr(d, m, 3, 0, n, 1, 1, 0, 0, 0) | 00111111 | 11 reg:3=d&7 rm:3=m&7 | 00000010" "asm"="vpcmpb $2, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f,avx512vl,avx512bw" "nodecode" {
  %r = icmp sle <32 x i8> %n, %m
  ret <32 x i1> %r
}

define <32 x i1> @VPCMPB.sge.32xi8(<32 x i8> "reg"="yv" %n, <32 x i8> "reg"="yv" %m) "enc"="evexr(d, m, 3, 0, n, 1, 1, 0, 0, 0) | 00111111 | 11 reg:3=d&7 rm:3=m&7 | 00000101" "asm"="vpcmpb $5, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f,avx512vl,avx512bw" "nodecode" {
  %r = icmp sge <32 x i8> %n, %m
  ret <32 x i1> %r
}

define <32 x i1> @VPCMPB.sgt.32xi8(<32 x i8> "reg"="yv" %n, <32 x i8> "reg"="yv" %m) "enc"="evexr(d, m, 3, 0, n, 1, 1, 0, 0, 0) | 00111111 | 11 reg:3=d&7 rm:3=m&7 | 00000110" "asm"="vpcmpb $6, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f,avx512vl,avx512bw" "nodecode" {
  %r = icmp sgt <32 x i8> %n, %m
  ret <32 x i1> %r
}

define <32 x i1> @VPCMPUB.ult.32xi8(<32 x i8> "reg"="yv" %n, <32 x i8> "reg"="yv" %m) "enc"="evexr(d, m, 3, 0, n, 1, 1, 0, 0, 0) | 00111110 | 11 reg:3=d&7 rm:3=m&7 | 00000001" "asm"="vpcmpub $1, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f,avx512vl,avx512bw" "nodecode" {
  %r = icmp ult <32 x i8> %n, %m
  ret <32 x i1> %r
}

define <32 x i1> @VPCMPUB.ule.32xi8(<32 x i8> "reg"="yv" %n, <32 x i8> "reg"="yv" %m) "enc"="evexr(d, m, 3, 0, n, 1, 1, 0, 0, 0) | 00111110 | 11 reg:3=d&7 rm:3=m&7 | 00000010" "asm"="vpcmpub $2, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f,avx512vl,avx512bw" "nodecode" {
  %r = icmp ule <32 x i8> %n, %m
  ret <32 x i1> %r
}

define <32 x i1> @VPCMPUB.uge.32xi8(<32 x i8> "reg"="yv" %n, <32 x i8> "reg"="yv" %m) "enc"="evexr(d, m, 3, 0, n, 1, 1, 0, 0, 0) | 00111110 | 11 reg:3=d&7 rm:3=m&7 | 00000101" "asm"="vpcmpub $5, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f,avx512vl,avx512bw" "nodecode" {
  %r = icmp uge <32 x i8> %n, %m
  ret <32 x i1> %r
}

define <32 x i1> @VPCMPUB.ugt.32xi8(<32 x i8> "reg"="yv" %n, <32 x i8> "reg"="yv" %m) "enc"="evexr(d, m, 3, 0, n, 1, 1, 0, 0, 0) | 00111110 | 11 reg:3=d&7 rm:3=m&7 | 00000110" "asm"="vpcmpub $6, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f,avx512vl,avx512bw" "nodecode" {
  %r = icmp ugt <32 x i8> %n, %m
  ret <32 x i1> %r
}

define <32 x i8> @VPBLENDMB.32xi8(<32 x i1> "reg"="kw" %k, <32 x i8> "reg"="yv" %a, <32 x i8> "reg"="yv" %b) "enc"="{ if(k!=0) evexr(d, a, 2, 0, b, 1, 1, 0, k, 0) | 01100110 | 11 reg:3=d&7 rm:3=a&7 }" "asm"="vpblendmb {a}, {b}, {d} {lb}{k}{rb}" "dclass"="yv" "feature"="avx512f,avx512vl,avx512bw" "nodecode" {
  %r = select <32 x i1> %k, <32 x i8> %a, <32 x i8> %b
  ret <32 x i8> %r
}

define <32 x i8> @VPMOVM2B.32xi8(<32 x i1> "reg"="kw" %k) "enc"="evexr(d, k, 2, 0, 0, 1, 2, 0, 0, 0) | 00101000 | 11 reg:3=d&7 rm:3=k&7" "asm"="vpmovm2b {k}, {d}" "dclass"="yv" "feature"="avx512f,avx512vl,avx512bw" "nodecode" {
  %r = sext <32 x i1> %k to <32 x i8>
  ret <32 x i8> %r
}

define <32 x i8> @VMASKBITS.32xi8(<32 x i1> "reg"="kw" %k) "enc"="evexr(d, k, 2, 0, 0, 1, 2, 0, 0, 0) | 00101000 | 11 reg:3=d&7 rm:3=k&7" "asm"="vpmovm2b {k}, {d}" "dclass"="yv" "feature"="avx512f,avx512vl,avx512bw" "nodecode" {
  %r = call <32 x i8> @llvm.epitome.mask.bits.v32i8(<32 x i1> %k)
  ret <32 x i8> %r
}

define <32 x i8> @VPABSBk.32xi8(<32 x i8> "reg"="yv" %n) "enc"="{ vex(d, 0, n, 2, 0, 0, 1, 1) | 00011100 | 11 reg:3=d&7 rm:3=n&7 ; evexr(d, n, 2, 0, 0, 1, 1, 0, 0, 0) | 00011100 | 11 reg:3=d&7 rm:3=n&7 }" "asm"="vpabsb {n}, {d}" "dclass"="yv" "nomatch" "feature"="avx512f,avx512vl,avx512bw" "nodecode" {
  ret <32 x i8> %n
}

define <32 x i8> @zextk.32xi8(<32 x i1> "reg"="kw" %k) "recipe"="zextk.32xi8.expand" "feature"="avx512f,avx512vl,avx512bw" {
  %r = zext <32 x i1> %k to <32 x i8>
  ret <32 x i8> %r
}

define <32 x i8> @zextk.32xi8.expand(<32 x i1> %k) {
  %s = call <32 x i8> @VPMOVM2B.32xi8(<32 x i1> %k)
  %r = call <32 x i8> @VPABSBk.32xi8(<32 x i8> %s)
  ret <32 x i8> %r
}

define <32 x i1> @VBITSMASK.32xi8(<32 x i8> "reg"="yv" %n) "enc"="evexr(d, n, 2, 0, n, 1, 1, 0, 0, 0) | 00100110 | 11 reg:3=d&7 rm:3=n&7" "asm"="vptestmb {n}, {n}, {d}" "dclass"="kw" "feature"="avx512f,avx512vl,avx512bw" "nodecode" {
  %r = call <32 x i1> @llvm.epitome.bits.mask.v32i8(<32 x i8> %n)
  ret <32 x i1> %r
}

define <16 x i1> @VPCMPW.eq.16xi16(<16 x i16> "reg"="yv" %n, <16 x i16> "reg"="yv" %m) "enc"="evexr(d, m, 3, 1, n, 1, 1, 0, 0, 0) | 00111111 | 11 reg:3=d&7 rm:3=m&7 | 00000000" "asm"="vpcmpw $0, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f,avx512vl,avx512bw" "nodecode" {
  %r = icmp eq <16 x i16> %n, %m
  ret <16 x i1> %r
}

define <16 x i1> @VPCMPW.ne.16xi16(<16 x i16> "reg"="yv" %n, <16 x i16> "reg"="yv" %m) "enc"="evexr(d, m, 3, 1, n, 1, 1, 0, 0, 0) | 00111111 | 11 reg:3=d&7 rm:3=m&7 | 00000100" "asm"="vpcmpw $4, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f,avx512vl,avx512bw" "nodecode" {
  %r = icmp ne <16 x i16> %n, %m
  ret <16 x i1> %r
}

define <16 x i1> @VPCMPW.slt.16xi16(<16 x i16> "reg"="yv" %n, <16 x i16> "reg"="yv" %m) "enc"="evexr(d, m, 3, 1, n, 1, 1, 0, 0, 0) | 00111111 | 11 reg:3=d&7 rm:3=m&7 | 00000001" "asm"="vpcmpw $1, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f,avx512vl,avx512bw" "nodecode" {
  %r = icmp slt <16 x i16> %n, %m
  ret <16 x i1> %r
}

define <16 x i1> @VPCMPW.sle.16xi16(<16 x i16> "reg"="yv" %n, <16 x i16> "reg"="yv" %m) "enc"="evexr(d, m, 3, 1, n, 1, 1, 0, 0, 0) | 00111111 | 11 reg:3=d&7 rm:3=m&7 | 00000010" "asm"="vpcmpw $2, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f,avx512vl,avx512bw" "nodecode" {
  %r = icmp sle <16 x i16> %n, %m
  ret <16 x i1> %r
}

define <16 x i1> @VPCMPW.sge.16xi16(<16 x i16> "reg"="yv" %n, <16 x i16> "reg"="yv" %m) "enc"="evexr(d, m, 3, 1, n, 1, 1, 0, 0, 0) | 00111111 | 11 reg:3=d&7 rm:3=m&7 | 00000101" "asm"="vpcmpw $5, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f,avx512vl,avx512bw" "nodecode" {
  %r = icmp sge <16 x i16> %n, %m
  ret <16 x i1> %r
}

define <16 x i1> @VPCMPW.sgt.16xi16(<16 x i16> "reg"="yv" %n, <16 x i16> "reg"="yv" %m) "enc"="evexr(d, m, 3, 1, n, 1, 1, 0, 0, 0) | 00111111 | 11 reg:3=d&7 rm:3=m&7 | 00000110" "asm"="vpcmpw $6, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f,avx512vl,avx512bw" "nodecode" {
  %r = icmp sgt <16 x i16> %n, %m
  ret <16 x i1> %r
}

define <16 x i1> @VPCMPUW.ult.16xi16(<16 x i16> "reg"="yv" %n, <16 x i16> "reg"="yv" %m) "enc"="evexr(d, m, 3, 1, n, 1, 1, 0, 0, 0) | 00111110 | 11 reg:3=d&7 rm:3=m&7 | 00000001" "asm"="vpcmpuw $1, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f,avx512vl,avx512bw" "nodecode" {
  %r = icmp ult <16 x i16> %n, %m
  ret <16 x i1> %r
}

define <16 x i1> @VPCMPUW.ule.16xi16(<16 x i16> "reg"="yv" %n, <16 x i16> "reg"="yv" %m) "enc"="evexr(d, m, 3, 1, n, 1, 1, 0, 0, 0) | 00111110 | 11 reg:3=d&7 rm:3=m&7 | 00000010" "asm"="vpcmpuw $2, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f,avx512vl,avx512bw" "nodecode" {
  %r = icmp ule <16 x i16> %n, %m
  ret <16 x i1> %r
}

define <16 x i1> @VPCMPUW.uge.16xi16(<16 x i16> "reg"="yv" %n, <16 x i16> "reg"="yv" %m) "enc"="evexr(d, m, 3, 1, n, 1, 1, 0, 0, 0) | 00111110 | 11 reg:3=d&7 rm:3=m&7 | 00000101" "asm"="vpcmpuw $5, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f,avx512vl,avx512bw" "nodecode" {
  %r = icmp uge <16 x i16> %n, %m
  ret <16 x i1> %r
}

define <16 x i1> @VPCMPUW.ugt.16xi16(<16 x i16> "reg"="yv" %n, <16 x i16> "reg"="yv" %m) "enc"="evexr(d, m, 3, 1, n, 1, 1, 0, 0, 0) | 00111110 | 11 reg:3=d&7 rm:3=m&7 | 00000110" "asm"="vpcmpuw $6, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f,avx512vl,avx512bw" "nodecode" {
  %r = icmp ugt <16 x i16> %n, %m
  ret <16 x i1> %r
}

define <16 x i16> @VPBLENDMW.16xi16(<16 x i1> "reg"="kw" %k, <16 x i16> "reg"="yv" %a, <16 x i16> "reg"="yv" %b) "enc"="{ if(k!=0) evexr(d, a, 2, 1, b, 1, 1, 0, k, 0) | 01100110 | 11 reg:3=d&7 rm:3=a&7 }" "asm"="vpblendmw {a}, {b}, {d} {lb}{k}{rb}" "dclass"="yv" "feature"="avx512f,avx512vl,avx512bw" "nodecode" {
  %r = select <16 x i1> %k, <16 x i16> %a, <16 x i16> %b
  ret <16 x i16> %r
}

define <16 x i16> @VPMOVM2W.16xi16(<16 x i1> "reg"="kw" %k) "enc"="evexr(d, k, 2, 1, 0, 1, 2, 0, 0, 0) | 00101000 | 11 reg:3=d&7 rm:3=k&7" "asm"="vpmovm2w {k}, {d}" "dclass"="yv" "feature"="avx512f,avx512vl,avx512bw" "nodecode" {
  %r = sext <16 x i1> %k to <16 x i16>
  ret <16 x i16> %r
}

define <16 x i16> @VMASKBITS.16xi16(<16 x i1> "reg"="kw" %k) "enc"="evexr(d, k, 2, 1, 0, 1, 2, 0, 0, 0) | 00101000 | 11 reg:3=d&7 rm:3=k&7" "asm"="vpmovm2w {k}, {d}" "dclass"="yv" "feature"="avx512f,avx512vl,avx512bw" "nodecode" {
  %r = call <16 x i16> @llvm.epitome.mask.bits.v16i16(<16 x i1> %k)
  ret <16 x i16> %r
}

define <16 x i16> @VPABSWk.16xi16(<16 x i16> "reg"="yv" %n) "enc"="{ vex(d, 0, n, 2, 0, 0, 1, 1) | 00011101 | 11 reg:3=d&7 rm:3=n&7 ; evexr(d, n, 2, 0, 0, 1, 1, 0, 0, 0) | 00011101 | 11 reg:3=d&7 rm:3=n&7 }" "asm"="vpabsw {n}, {d}" "dclass"="yv" "nomatch" "feature"="avx512f,avx512vl,avx512bw" "nodecode" {
  ret <16 x i16> %n
}

define <16 x i16> @zextk.16xi16(<16 x i1> "reg"="kw" %k) "recipe"="zextk.16xi16.expand" "feature"="avx512f,avx512vl,avx512bw" {
  %r = zext <16 x i1> %k to <16 x i16>
  ret <16 x i16> %r
}

define <16 x i16> @zextk.16xi16.expand(<16 x i1> %k) {
  %s = call <16 x i16> @VPMOVM2W.16xi16(<16 x i1> %k)
  %r = call <16 x i16> @VPABSWk.16xi16(<16 x i16> %s)
  ret <16 x i16> %r
}

define <16 x i1> @VBITSMASK.16xi16(<16 x i16> "reg"="yv" %n) "enc"="evexr(d, n, 2, 1, n, 1, 1, 0, 0, 0) | 00100110 | 11 reg:3=d&7 rm:3=n&7" "asm"="vptestmw {n}, {n}, {d}" "dclass"="kw" "feature"="avx512f,avx512vl,avx512bw" "nodecode" {
  %r = call <16 x i1> @llvm.epitome.bits.mask.v16i16(<16 x i16> %n)
  ret <16 x i1> %r
}

define <8 x i1> @VPCMPD.eq.8xi32(<8 x i32> "reg"="yv" %n, <8 x i32> "reg"="yv" %m) "enc"="evexr(d, m, 3, 0, n, 1, 1, 0, 0, 0) | 00011111 | 11 reg:3=d&7 rm:3=m&7 | 00000000" "asm"="vpcmpd $0, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f,avx512vl" "nodecode" {
  %r = icmp eq <8 x i32> %n, %m
  ret <8 x i1> %r
}

define <8 x i1> @VPCMPD.ne.8xi32(<8 x i32> "reg"="yv" %n, <8 x i32> "reg"="yv" %m) "enc"="evexr(d, m, 3, 0, n, 1, 1, 0, 0, 0) | 00011111 | 11 reg:3=d&7 rm:3=m&7 | 00000100" "asm"="vpcmpd $4, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f,avx512vl" "nodecode" {
  %r = icmp ne <8 x i32> %n, %m
  ret <8 x i1> %r
}

define <8 x i1> @VPCMPD.slt.8xi32(<8 x i32> "reg"="yv" %n, <8 x i32> "reg"="yv" %m) "enc"="evexr(d, m, 3, 0, n, 1, 1, 0, 0, 0) | 00011111 | 11 reg:3=d&7 rm:3=m&7 | 00000001" "asm"="vpcmpd $1, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f,avx512vl" "nodecode" {
  %r = icmp slt <8 x i32> %n, %m
  ret <8 x i1> %r
}

define <8 x i1> @VPCMPD.sle.8xi32(<8 x i32> "reg"="yv" %n, <8 x i32> "reg"="yv" %m) "enc"="evexr(d, m, 3, 0, n, 1, 1, 0, 0, 0) | 00011111 | 11 reg:3=d&7 rm:3=m&7 | 00000010" "asm"="vpcmpd $2, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f,avx512vl" "nodecode" {
  %r = icmp sle <8 x i32> %n, %m
  ret <8 x i1> %r
}

define <8 x i1> @VPCMPD.sge.8xi32(<8 x i32> "reg"="yv" %n, <8 x i32> "reg"="yv" %m) "enc"="evexr(d, m, 3, 0, n, 1, 1, 0, 0, 0) | 00011111 | 11 reg:3=d&7 rm:3=m&7 | 00000101" "asm"="vpcmpd $5, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f,avx512vl" "nodecode" {
  %r = icmp sge <8 x i32> %n, %m
  ret <8 x i1> %r
}

define <8 x i1> @VPCMPD.sgt.8xi32(<8 x i32> "reg"="yv" %n, <8 x i32> "reg"="yv" %m) "enc"="evexr(d, m, 3, 0, n, 1, 1, 0, 0, 0) | 00011111 | 11 reg:3=d&7 rm:3=m&7 | 00000110" "asm"="vpcmpd $6, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f,avx512vl" "nodecode" {
  %r = icmp sgt <8 x i32> %n, %m
  ret <8 x i1> %r
}

define <8 x i1> @VPCMPUD.ult.8xi32(<8 x i32> "reg"="yv" %n, <8 x i32> "reg"="yv" %m) "enc"="evexr(d, m, 3, 0, n, 1, 1, 0, 0, 0) | 00011110 | 11 reg:3=d&7 rm:3=m&7 | 00000001" "asm"="vpcmpud $1, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f,avx512vl" "nodecode" {
  %r = icmp ult <8 x i32> %n, %m
  ret <8 x i1> %r
}

define <8 x i1> @VPCMPUD.ule.8xi32(<8 x i32> "reg"="yv" %n, <8 x i32> "reg"="yv" %m) "enc"="evexr(d, m, 3, 0, n, 1, 1, 0, 0, 0) | 00011110 | 11 reg:3=d&7 rm:3=m&7 | 00000010" "asm"="vpcmpud $2, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f,avx512vl" "nodecode" {
  %r = icmp ule <8 x i32> %n, %m
  ret <8 x i1> %r
}

define <8 x i1> @VPCMPUD.uge.8xi32(<8 x i32> "reg"="yv" %n, <8 x i32> "reg"="yv" %m) "enc"="evexr(d, m, 3, 0, n, 1, 1, 0, 0, 0) | 00011110 | 11 reg:3=d&7 rm:3=m&7 | 00000101" "asm"="vpcmpud $5, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f,avx512vl" "nodecode" {
  %r = icmp uge <8 x i32> %n, %m
  ret <8 x i1> %r
}

define <8 x i1> @VPCMPUD.ugt.8xi32(<8 x i32> "reg"="yv" %n, <8 x i32> "reg"="yv" %m) "enc"="evexr(d, m, 3, 0, n, 1, 1, 0, 0, 0) | 00011110 | 11 reg:3=d&7 rm:3=m&7 | 00000110" "asm"="vpcmpud $6, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f,avx512vl" "nodecode" {
  %r = icmp ugt <8 x i32> %n, %m
  ret <8 x i1> %r
}

define <8 x i32> @VPBLENDMD.8xi32(<8 x i1> "reg"="kw" %k, <8 x i32> "reg"="yv" %a, <8 x i32> "reg"="yv" %b) "enc"="{ if(k!=0) evexr(d, a, 2, 0, b, 1, 1, 0, k, 0) | 01100100 | 11 reg:3=d&7 rm:3=a&7 }" "asm"="vpblendmd {a}, {b}, {d} {lb}{k}{rb}" "dclass"="yv" "feature"="avx512f,avx512vl" "nodecode" {
  %r = select <8 x i1> %k, <8 x i32> %a, <8 x i32> %b
  ret <8 x i32> %r
}

define <8 x i32> @VPMOVM2D.8xi32(<8 x i1> "reg"="kw" %k) "enc"="{ if(k!=0) evexr(d, d, 3, 0, d, 1, 1, 0, k, 1) | 00100101 | 11 reg:3=d&7 rm:3=d&7 | 11111111 }" "asm"="vpternlogd $255, {d}, {d}, {d} {lb}{k}{rb} {lb}z{rb}" "dclass"="yv" "feature"="avx512f,avx512vl" "nodecode" {
  %r = sext <8 x i1> %k to <8 x i32>
  ret <8 x i32> %r
}

define <8 x i32> @VMASKBITS.8xi32(<8 x i1> "reg"="kw" %k) "enc"="{ if(k!=0) evexr(d, d, 3, 0, d, 1, 1, 0, k, 1) | 00100101 | 11 reg:3=d&7 rm:3=d&7 | 11111111 }" "asm"="vpternlogd $255, {d}, {d}, {d} {lb}{k}{rb} {lb}z{rb}" "dclass"="yv" "feature"="avx512f,avx512vl" "nodecode" {
  %r = call <8 x i32> @llvm.epitome.mask.bits.v8i32(<8 x i1> %k)
  ret <8 x i32> %r
}

define <8 x i32> @VPABSDk.8xi32(<8 x i32> "reg"="yv" %n) "enc"="{ vex(d, 0, n, 2, 0, 0, 1, 1) | 00011110 | 11 reg:3=d&7 rm:3=n&7 ; evexr(d, n, 2, 0, 0, 1, 1, 0, 0, 0) | 00011110 | 11 reg:3=d&7 rm:3=n&7 }" "asm"="vpabsd {n}, {d}" "dclass"="yv" "nomatch" "feature"="avx512f,avx512vl" "nodecode" {
  ret <8 x i32> %n
}

define <8 x i32> @zextk.8xi32(<8 x i1> "reg"="kw" %k) "recipe"="zextk.8xi32.expand" "feature"="avx512f,avx512vl" {
  %r = zext <8 x i1> %k to <8 x i32>
  ret <8 x i32> %r
}

define <8 x i32> @zextk.8xi32.expand(<8 x i1> %k) {
  %s = call <8 x i32> @VPMOVM2D.8xi32(<8 x i1> %k)
  %r = call <8 x i32> @VPABSDk.8xi32(<8 x i32> %s)
  ret <8 x i32> %r
}

define <8 x i1> @VBITSMASK.8xi32(<8 x i32> "reg"="yv" %n) "enc"="evexr(d, n, 2, 0, n, 1, 1, 0, 0, 0) | 00100111 | 11 reg:3=d&7 rm:3=n&7" "asm"="vptestmd {n}, {n}, {d}" "dclass"="kw" "feature"="avx512f,avx512vl" "nodecode" {
  %r = call <8 x i1> @llvm.epitome.bits.mask.v8i32(<8 x i32> %n)
  ret <8 x i1> %r
}

define <4 x i1> @VPCMPQ.eq.4xi64(<4 x i64> "reg"="yv" %n, <4 x i64> "reg"="yv" %m) "enc"="evexr(d, m, 3, 1, n, 1, 1, 0, 0, 0) | 00011111 | 11 reg:3=d&7 rm:3=m&7 | 00000000" "asm"="vpcmpq $0, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f,avx512vl" "nodecode" {
  %r = icmp eq <4 x i64> %n, %m
  ret <4 x i1> %r
}

define <4 x i1> @VPCMPQ.ne.4xi64(<4 x i64> "reg"="yv" %n, <4 x i64> "reg"="yv" %m) "enc"="evexr(d, m, 3, 1, n, 1, 1, 0, 0, 0) | 00011111 | 11 reg:3=d&7 rm:3=m&7 | 00000100" "asm"="vpcmpq $4, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f,avx512vl" "nodecode" {
  %r = icmp ne <4 x i64> %n, %m
  ret <4 x i1> %r
}

define <4 x i1> @VPCMPQ.slt.4xi64(<4 x i64> "reg"="yv" %n, <4 x i64> "reg"="yv" %m) "enc"="evexr(d, m, 3, 1, n, 1, 1, 0, 0, 0) | 00011111 | 11 reg:3=d&7 rm:3=m&7 | 00000001" "asm"="vpcmpq $1, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f,avx512vl" "nodecode" {
  %r = icmp slt <4 x i64> %n, %m
  ret <4 x i1> %r
}

define <4 x i1> @VPCMPQ.sle.4xi64(<4 x i64> "reg"="yv" %n, <4 x i64> "reg"="yv" %m) "enc"="evexr(d, m, 3, 1, n, 1, 1, 0, 0, 0) | 00011111 | 11 reg:3=d&7 rm:3=m&7 | 00000010" "asm"="vpcmpq $2, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f,avx512vl" "nodecode" {
  %r = icmp sle <4 x i64> %n, %m
  ret <4 x i1> %r
}

define <4 x i1> @VPCMPQ.sge.4xi64(<4 x i64> "reg"="yv" %n, <4 x i64> "reg"="yv" %m) "enc"="evexr(d, m, 3, 1, n, 1, 1, 0, 0, 0) | 00011111 | 11 reg:3=d&7 rm:3=m&7 | 00000101" "asm"="vpcmpq $5, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f,avx512vl" "nodecode" {
  %r = icmp sge <4 x i64> %n, %m
  ret <4 x i1> %r
}

define <4 x i1> @VPCMPQ.sgt.4xi64(<4 x i64> "reg"="yv" %n, <4 x i64> "reg"="yv" %m) "enc"="evexr(d, m, 3, 1, n, 1, 1, 0, 0, 0) | 00011111 | 11 reg:3=d&7 rm:3=m&7 | 00000110" "asm"="vpcmpq $6, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f,avx512vl" "nodecode" {
  %r = icmp sgt <4 x i64> %n, %m
  ret <4 x i1> %r
}

define <4 x i1> @VPCMPUQ.ult.4xi64(<4 x i64> "reg"="yv" %n, <4 x i64> "reg"="yv" %m) "enc"="evexr(d, m, 3, 1, n, 1, 1, 0, 0, 0) | 00011110 | 11 reg:3=d&7 rm:3=m&7 | 00000001" "asm"="vpcmpuq $1, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f,avx512vl" "nodecode" {
  %r = icmp ult <4 x i64> %n, %m
  ret <4 x i1> %r
}

define <4 x i1> @VPCMPUQ.ule.4xi64(<4 x i64> "reg"="yv" %n, <4 x i64> "reg"="yv" %m) "enc"="evexr(d, m, 3, 1, n, 1, 1, 0, 0, 0) | 00011110 | 11 reg:3=d&7 rm:3=m&7 | 00000010" "asm"="vpcmpuq $2, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f,avx512vl" "nodecode" {
  %r = icmp ule <4 x i64> %n, %m
  ret <4 x i1> %r
}

define <4 x i1> @VPCMPUQ.uge.4xi64(<4 x i64> "reg"="yv" %n, <4 x i64> "reg"="yv" %m) "enc"="evexr(d, m, 3, 1, n, 1, 1, 0, 0, 0) | 00011110 | 11 reg:3=d&7 rm:3=m&7 | 00000101" "asm"="vpcmpuq $5, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f,avx512vl" "nodecode" {
  %r = icmp uge <4 x i64> %n, %m
  ret <4 x i1> %r
}

define <4 x i1> @VPCMPUQ.ugt.4xi64(<4 x i64> "reg"="yv" %n, <4 x i64> "reg"="yv" %m) "enc"="evexr(d, m, 3, 1, n, 1, 1, 0, 0, 0) | 00011110 | 11 reg:3=d&7 rm:3=m&7 | 00000110" "asm"="vpcmpuq $6, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f,avx512vl" "nodecode" {
  %r = icmp ugt <4 x i64> %n, %m
  ret <4 x i1> %r
}

define <4 x i64> @VPBLENDMQ.4xi64(<4 x i1> "reg"="kw" %k, <4 x i64> "reg"="yv" %a, <4 x i64> "reg"="yv" %b) "enc"="{ if(k!=0) evexr(d, a, 2, 1, b, 1, 1, 0, k, 0) | 01100100 | 11 reg:3=d&7 rm:3=a&7 }" "asm"="vpblendmq {a}, {b}, {d} {lb}{k}{rb}" "dclass"="yv" "feature"="avx512f,avx512vl" "nodecode" {
  %r = select <4 x i1> %k, <4 x i64> %a, <4 x i64> %b
  ret <4 x i64> %r
}

define <4 x i64> @VPMOVM2Q.4xi64(<4 x i1> "reg"="kw" %k) "enc"="{ if(k!=0) evexr(d, d, 3, 1, d, 1, 1, 0, k, 1) | 00100101 | 11 reg:3=d&7 rm:3=d&7 | 11111111 }" "asm"="vpternlogq $255, {d}, {d}, {d} {lb}{k}{rb} {lb}z{rb}" "dclass"="yv" "feature"="avx512f,avx512vl" "nodecode" {
  %r = sext <4 x i1> %k to <4 x i64>
  ret <4 x i64> %r
}

define <4 x i64> @VMASKBITS.4xi64(<4 x i1> "reg"="kw" %k) "enc"="{ if(k!=0) evexr(d, d, 3, 1, d, 1, 1, 0, k, 1) | 00100101 | 11 reg:3=d&7 rm:3=d&7 | 11111111 }" "asm"="vpternlogq $255, {d}, {d}, {d} {lb}{k}{rb} {lb}z{rb}" "dclass"="yv" "feature"="avx512f,avx512vl" "nodecode" {
  %r = call <4 x i64> @llvm.epitome.mask.bits.v4i64(<4 x i1> %k)
  ret <4 x i64> %r
}

define <4 x i64> @VPABSQk.4xi64(<4 x i64> "reg"="yv" %n) "enc"="evexr(d, n, 2, 1, 0, 1, 1, 0, 0, 0) | 00011111 | 11 reg:3=d&7 rm:3=n&7" "asm"="vpabsq {n}, {d}" "dclass"="yv" "nomatch" "feature"="avx512f,avx512vl" "nodecode" {
  ret <4 x i64> %n
}

define <4 x i64> @zextk.4xi64(<4 x i1> "reg"="kw" %k) "recipe"="zextk.4xi64.expand" "feature"="avx512f,avx512vl" {
  %r = zext <4 x i1> %k to <4 x i64>
  ret <4 x i64> %r
}

define <4 x i64> @zextk.4xi64.expand(<4 x i1> %k) {
  %s = call <4 x i64> @VPMOVM2Q.4xi64(<4 x i1> %k)
  %r = call <4 x i64> @VPABSQk.4xi64(<4 x i64> %s)
  ret <4 x i64> %r
}

define <4 x i1> @VBITSMASK.4xi64(<4 x i64> "reg"="yv" %n) "enc"="evexr(d, n, 2, 1, n, 1, 1, 0, 0, 0) | 00100111 | 11 reg:3=d&7 rm:3=n&7" "asm"="vptestmq {n}, {n}, {d}" "dclass"="kw" "feature"="avx512f,avx512vl" "nodecode" {
  %r = call <4 x i1> @llvm.epitome.bits.mask.v4i64(<4 x i64> %n)
  ret <4 x i1> %r
}

define <8 x i1> @VCMPOEQPSk.8xf32(<8 x float> "reg"="yv" %n, <8 x float> "reg"="yv" %m) "enc"="{ evexr(d, m, 1, 0, n, 1, 0, 0, 0, 0) | 11000010 | 11 reg:3=d&7 rm:3=m&7 | 00000000 ; if(((0&3)==0)|((0&3)==3)) evexr(d, n, 1, 0, m, 1, 0, 0, 0, 0) | 11000010 | 11 reg:3=d&7 rm:3=n&7 | 00000000 }" "asm"="vcmpps $0, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f,avx512vl" "nodecode" {
  %r = fcmp oeq <8 x float> %n, %m
  ret <8 x i1> %r
}

define <8 x i1> @VCMPOLTPSk.8xf32(<8 x float> "reg"="yv" %n, <8 x float> "reg"="yv" %m) "enc"="{ evexr(d, m, 1, 0, n, 1, 0, 0, 0, 0) | 11000010 | 11 reg:3=d&7 rm:3=m&7 | 00000001 ; if(((1&3)==0)|((1&3)==3)) evexr(d, n, 1, 0, m, 1, 0, 0, 0, 0) | 11000010 | 11 reg:3=d&7 rm:3=n&7 | 00000001 }" "asm"="vcmpps $1, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f,avx512vl" "nodecode" {
  %r = fcmp olt <8 x float> %n, %m
  ret <8 x i1> %r
}

define <8 x i1> @VCMPOLEPSk.8xf32(<8 x float> "reg"="yv" %n, <8 x float> "reg"="yv" %m) "enc"="{ evexr(d, m, 1, 0, n, 1, 0, 0, 0, 0) | 11000010 | 11 reg:3=d&7 rm:3=m&7 | 00000010 ; if(((2&3)==0)|((2&3)==3)) evexr(d, n, 1, 0, m, 1, 0, 0, 0, 0) | 11000010 | 11 reg:3=d&7 rm:3=n&7 | 00000010 }" "asm"="vcmpps $2, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f,avx512vl" "nodecode" {
  %r = fcmp ole <8 x float> %n, %m
  ret <8 x i1> %r
}

define <8 x i1> @VCMPUNOPSk.8xf32(<8 x float> "reg"="yv" %n, <8 x float> "reg"="yv" %m) "enc"="{ evexr(d, m, 1, 0, n, 1, 0, 0, 0, 0) | 11000010 | 11 reg:3=d&7 rm:3=m&7 | 00000011 ; if(((3&3)==0)|((3&3)==3)) evexr(d, n, 1, 0, m, 1, 0, 0, 0, 0) | 11000010 | 11 reg:3=d&7 rm:3=n&7 | 00000011 }" "asm"="vcmpps $3, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f,avx512vl" "nodecode" {
  %r = fcmp uno <8 x float> %n, %m
  ret <8 x i1> %r
}

define <8 x i1> @VCMPUNEPSk.8xf32(<8 x float> "reg"="yv" %n, <8 x float> "reg"="yv" %m) "enc"="{ evexr(d, m, 1, 0, n, 1, 0, 0, 0, 0) | 11000010 | 11 reg:3=d&7 rm:3=m&7 | 00000100 ; if(((4&3)==0)|((4&3)==3)) evexr(d, n, 1, 0, m, 1, 0, 0, 0, 0) | 11000010 | 11 reg:3=d&7 rm:3=n&7 | 00000100 }" "asm"="vcmpps $4, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f,avx512vl" "nodecode" {
  %r = fcmp une <8 x float> %n, %m
  ret <8 x i1> %r
}

define <8 x i1> @VCMPUGEPSk.8xf32(<8 x float> "reg"="yv" %n, <8 x float> "reg"="yv" %m) "enc"="{ evexr(d, m, 1, 0, n, 1, 0, 0, 0, 0) | 11000010 | 11 reg:3=d&7 rm:3=m&7 | 00000101 ; if(((5&3)==0)|((5&3)==3)) evexr(d, n, 1, 0, m, 1, 0, 0, 0, 0) | 11000010 | 11 reg:3=d&7 rm:3=n&7 | 00000101 }" "asm"="vcmpps $5, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f,avx512vl" "nodecode" {
  %r = fcmp uge <8 x float> %n, %m
  ret <8 x i1> %r
}

define <8 x i1> @VCMPUGTPSk.8xf32(<8 x float> "reg"="yv" %n, <8 x float> "reg"="yv" %m) "enc"="{ evexr(d, m, 1, 0, n, 1, 0, 0, 0, 0) | 11000010 | 11 reg:3=d&7 rm:3=m&7 | 00000110 ; if(((6&3)==0)|((6&3)==3)) evexr(d, n, 1, 0, m, 1, 0, 0, 0, 0) | 11000010 | 11 reg:3=d&7 rm:3=n&7 | 00000110 }" "asm"="vcmpps $6, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f,avx512vl" "nodecode" {
  %r = fcmp ugt <8 x float> %n, %m
  ret <8 x i1> %r
}

define <8 x i1> @VCMPORDPSk.8xf32(<8 x float> "reg"="yv" %n, <8 x float> "reg"="yv" %m) "enc"="{ evexr(d, m, 1, 0, n, 1, 0, 0, 0, 0) | 11000010 | 11 reg:3=d&7 rm:3=m&7 | 00000111 ; if(((7&3)==0)|((7&3)==3)) evexr(d, n, 1, 0, m, 1, 0, 0, 0, 0) | 11000010 | 11 reg:3=d&7 rm:3=n&7 | 00000111 }" "asm"="vcmpps $7, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f,avx512vl" "nodecode" {
  %r = fcmp ord <8 x float> %n, %m
  ret <8 x i1> %r
}

define <8 x i1> @VCMPUEQPSk.8xf32(<8 x float> "reg"="yv" %n, <8 x float> "reg"="yv" %m) "enc"="{ evexr(d, m, 1, 0, n, 1, 0, 0, 0, 0) | 11000010 | 11 reg:3=d&7 rm:3=m&7 | 00001000 ; if(((8&3)==0)|((8&3)==3)) evexr(d, n, 1, 0, m, 1, 0, 0, 0, 0) | 11000010 | 11 reg:3=d&7 rm:3=n&7 | 00001000 }" "asm"="vcmpps $8, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f,avx512vl" "nodecode" {
  %r = fcmp ueq <8 x float> %n, %m
  ret <8 x i1> %r
}

define <8 x i1> @VCMPULTPSk.8xf32(<8 x float> "reg"="yv" %n, <8 x float> "reg"="yv" %m) "enc"="{ evexr(d, m, 1, 0, n, 1, 0, 0, 0, 0) | 11000010 | 11 reg:3=d&7 rm:3=m&7 | 00001001 ; if(((9&3)==0)|((9&3)==3)) evexr(d, n, 1, 0, m, 1, 0, 0, 0, 0) | 11000010 | 11 reg:3=d&7 rm:3=n&7 | 00001001 }" "asm"="vcmpps $9, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f,avx512vl" "nodecode" {
  %r = fcmp ult <8 x float> %n, %m
  ret <8 x i1> %r
}

define <8 x i1> @VCMPULEPSk.8xf32(<8 x float> "reg"="yv" %n, <8 x float> "reg"="yv" %m) "enc"="{ evexr(d, m, 1, 0, n, 1, 0, 0, 0, 0) | 11000010 | 11 reg:3=d&7 rm:3=m&7 | 00001010 ; if(((10&3)==0)|((10&3)==3)) evexr(d, n, 1, 0, m, 1, 0, 0, 0, 0) | 11000010 | 11 reg:3=d&7 rm:3=n&7 | 00001010 }" "asm"="vcmpps $10, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f,avx512vl" "nodecode" {
  %r = fcmp ule <8 x float> %n, %m
  ret <8 x i1> %r
}

define <8 x i1> @VCMPONEPSk.8xf32(<8 x float> "reg"="yv" %n, <8 x float> "reg"="yv" %m) "enc"="{ evexr(d, m, 1, 0, n, 1, 0, 0, 0, 0) | 11000010 | 11 reg:3=d&7 rm:3=m&7 | 00001100 ; if(((12&3)==0)|((12&3)==3)) evexr(d, n, 1, 0, m, 1, 0, 0, 0, 0) | 11000010 | 11 reg:3=d&7 rm:3=n&7 | 00001100 }" "asm"="vcmpps $12, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f,avx512vl" "nodecode" {
  %r = fcmp one <8 x float> %n, %m
  ret <8 x i1> %r
}

define <8 x i1> @VCMPOGEPSk.8xf32(<8 x float> "reg"="yv" %n, <8 x float> "reg"="yv" %m) "enc"="{ evexr(d, m, 1, 0, n, 1, 0, 0, 0, 0) | 11000010 | 11 reg:3=d&7 rm:3=m&7 | 00001101 ; if(((13&3)==0)|((13&3)==3)) evexr(d, n, 1, 0, m, 1, 0, 0, 0, 0) | 11000010 | 11 reg:3=d&7 rm:3=n&7 | 00001101 }" "asm"="vcmpps $13, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f,avx512vl" "nodecode" {
  %r = fcmp oge <8 x float> %n, %m
  ret <8 x i1> %r
}

define <8 x i1> @VCMPOGTPSk.8xf32(<8 x float> "reg"="yv" %n, <8 x float> "reg"="yv" %m) "enc"="{ evexr(d, m, 1, 0, n, 1, 0, 0, 0, 0) | 11000010 | 11 reg:3=d&7 rm:3=m&7 | 00001110 ; if(((14&3)==0)|((14&3)==3)) evexr(d, n, 1, 0, m, 1, 0, 0, 0, 0) | 11000010 | 11 reg:3=d&7 rm:3=n&7 | 00001110 }" "asm"="vcmpps $14, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f,avx512vl" "nodecode" {
  %r = fcmp ogt <8 x float> %n, %m
  ret <8 x i1> %r
}

define <8 x float> @VBLENDMPS.8xf32(<8 x i1> "reg"="kw" %k, <8 x float> "reg"="yv" %a, <8 x float> "reg"="yv" %b) "enc"="{ if(k!=0) evexr(d, a, 2, 0, b, 1, 1, 0, k, 0) | 01100101 | 11 reg:3=d&7 rm:3=a&7 }" "asm"="vblendmps {a}, {b}, {d} {lb}{k}{rb}" "dclass"="yv" "feature"="avx512f,avx512vl" "nodecode" {
  %r = select <8 x i1> %k, <8 x float> %a, <8 x float> %b
  ret <8 x float> %r
}

define <4 x i1> @VCMPOEQPDk.4xf64(<4 x double> "reg"="yv" %n, <4 x double> "reg"="yv" %m) "enc"="{ evexr(d, m, 1, 1, n, 1, 1, 0, 0, 0) | 11000010 | 11 reg:3=d&7 rm:3=m&7 | 00000000 ; if(((0&3)==0)|((0&3)==3)) evexr(d, n, 1, 1, m, 1, 1, 0, 0, 0) | 11000010 | 11 reg:3=d&7 rm:3=n&7 | 00000000 }" "asm"="vcmppd $0, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f,avx512vl" "nodecode" {
  %r = fcmp oeq <4 x double> %n, %m
  ret <4 x i1> %r
}

define <4 x i1> @VCMPOLTPDk.4xf64(<4 x double> "reg"="yv" %n, <4 x double> "reg"="yv" %m) "enc"="{ evexr(d, m, 1, 1, n, 1, 1, 0, 0, 0) | 11000010 | 11 reg:3=d&7 rm:3=m&7 | 00000001 ; if(((1&3)==0)|((1&3)==3)) evexr(d, n, 1, 1, m, 1, 1, 0, 0, 0) | 11000010 | 11 reg:3=d&7 rm:3=n&7 | 00000001 }" "asm"="vcmppd $1, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f,avx512vl" "nodecode" {
  %r = fcmp olt <4 x double> %n, %m
  ret <4 x i1> %r
}

define <4 x i1> @VCMPOLEPDk.4xf64(<4 x double> "reg"="yv" %n, <4 x double> "reg"="yv" %m) "enc"="{ evexr(d, m, 1, 1, n, 1, 1, 0, 0, 0) | 11000010 | 11 reg:3=d&7 rm:3=m&7 | 00000010 ; if(((2&3)==0)|((2&3)==3)) evexr(d, n, 1, 1, m, 1, 1, 0, 0, 0) | 11000010 | 11 reg:3=d&7 rm:3=n&7 | 00000010 }" "asm"="vcmppd $2, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f,avx512vl" "nodecode" {
  %r = fcmp ole <4 x double> %n, %m
  ret <4 x i1> %r
}

define <4 x i1> @VCMPUNOPDk.4xf64(<4 x double> "reg"="yv" %n, <4 x double> "reg"="yv" %m) "enc"="{ evexr(d, m, 1, 1, n, 1, 1, 0, 0, 0) | 11000010 | 11 reg:3=d&7 rm:3=m&7 | 00000011 ; if(((3&3)==0)|((3&3)==3)) evexr(d, n, 1, 1, m, 1, 1, 0, 0, 0) | 11000010 | 11 reg:3=d&7 rm:3=n&7 | 00000011 }" "asm"="vcmppd $3, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f,avx512vl" "nodecode" {
  %r = fcmp uno <4 x double> %n, %m
  ret <4 x i1> %r
}

define <4 x i1> @VCMPUNEPDk.4xf64(<4 x double> "reg"="yv" %n, <4 x double> "reg"="yv" %m) "enc"="{ evexr(d, m, 1, 1, n, 1, 1, 0, 0, 0) | 11000010 | 11 reg:3=d&7 rm:3=m&7 | 00000100 ; if(((4&3)==0)|((4&3)==3)) evexr(d, n, 1, 1, m, 1, 1, 0, 0, 0) | 11000010 | 11 reg:3=d&7 rm:3=n&7 | 00000100 }" "asm"="vcmppd $4, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f,avx512vl" "nodecode" {
  %r = fcmp une <4 x double> %n, %m
  ret <4 x i1> %r
}

define <4 x i1> @VCMPUGEPDk.4xf64(<4 x double> "reg"="yv" %n, <4 x double> "reg"="yv" %m) "enc"="{ evexr(d, m, 1, 1, n, 1, 1, 0, 0, 0) | 11000010 | 11 reg:3=d&7 rm:3=m&7 | 00000101 ; if(((5&3)==0)|((5&3)==3)) evexr(d, n, 1, 1, m, 1, 1, 0, 0, 0) | 11000010 | 11 reg:3=d&7 rm:3=n&7 | 00000101 }" "asm"="vcmppd $5, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f,avx512vl" "nodecode" {
  %r = fcmp uge <4 x double> %n, %m
  ret <4 x i1> %r
}

define <4 x i1> @VCMPUGTPDk.4xf64(<4 x double> "reg"="yv" %n, <4 x double> "reg"="yv" %m) "enc"="{ evexr(d, m, 1, 1, n, 1, 1, 0, 0, 0) | 11000010 | 11 reg:3=d&7 rm:3=m&7 | 00000110 ; if(((6&3)==0)|((6&3)==3)) evexr(d, n, 1, 1, m, 1, 1, 0, 0, 0) | 11000010 | 11 reg:3=d&7 rm:3=n&7 | 00000110 }" "asm"="vcmppd $6, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f,avx512vl" "nodecode" {
  %r = fcmp ugt <4 x double> %n, %m
  ret <4 x i1> %r
}

define <4 x i1> @VCMPORDPDk.4xf64(<4 x double> "reg"="yv" %n, <4 x double> "reg"="yv" %m) "enc"="{ evexr(d, m, 1, 1, n, 1, 1, 0, 0, 0) | 11000010 | 11 reg:3=d&7 rm:3=m&7 | 00000111 ; if(((7&3)==0)|((7&3)==3)) evexr(d, n, 1, 1, m, 1, 1, 0, 0, 0) | 11000010 | 11 reg:3=d&7 rm:3=n&7 | 00000111 }" "asm"="vcmppd $7, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f,avx512vl" "nodecode" {
  %r = fcmp ord <4 x double> %n, %m
  ret <4 x i1> %r
}

define <4 x i1> @VCMPUEQPDk.4xf64(<4 x double> "reg"="yv" %n, <4 x double> "reg"="yv" %m) "enc"="{ evexr(d, m, 1, 1, n, 1, 1, 0, 0, 0) | 11000010 | 11 reg:3=d&7 rm:3=m&7 | 00001000 ; if(((8&3)==0)|((8&3)==3)) evexr(d, n, 1, 1, m, 1, 1, 0, 0, 0) | 11000010 | 11 reg:3=d&7 rm:3=n&7 | 00001000 }" "asm"="vcmppd $8, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f,avx512vl" "nodecode" {
  %r = fcmp ueq <4 x double> %n, %m
  ret <4 x i1> %r
}

define <4 x i1> @VCMPULTPDk.4xf64(<4 x double> "reg"="yv" %n, <4 x double> "reg"="yv" %m) "enc"="{ evexr(d, m, 1, 1, n, 1, 1, 0, 0, 0) | 11000010 | 11 reg:3=d&7 rm:3=m&7 | 00001001 ; if(((9&3)==0)|((9&3)==3)) evexr(d, n, 1, 1, m, 1, 1, 0, 0, 0) | 11000010 | 11 reg:3=d&7 rm:3=n&7 | 00001001 }" "asm"="vcmppd $9, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f,avx512vl" "nodecode" {
  %r = fcmp ult <4 x double> %n, %m
  ret <4 x i1> %r
}

define <4 x i1> @VCMPULEPDk.4xf64(<4 x double> "reg"="yv" %n, <4 x double> "reg"="yv" %m) "enc"="{ evexr(d, m, 1, 1, n, 1, 1, 0, 0, 0) | 11000010 | 11 reg:3=d&7 rm:3=m&7 | 00001010 ; if(((10&3)==0)|((10&3)==3)) evexr(d, n, 1, 1, m, 1, 1, 0, 0, 0) | 11000010 | 11 reg:3=d&7 rm:3=n&7 | 00001010 }" "asm"="vcmppd $10, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f,avx512vl" "nodecode" {
  %r = fcmp ule <4 x double> %n, %m
  ret <4 x i1> %r
}

define <4 x i1> @VCMPONEPDk.4xf64(<4 x double> "reg"="yv" %n, <4 x double> "reg"="yv" %m) "enc"="{ evexr(d, m, 1, 1, n, 1, 1, 0, 0, 0) | 11000010 | 11 reg:3=d&7 rm:3=m&7 | 00001100 ; if(((12&3)==0)|((12&3)==3)) evexr(d, n, 1, 1, m, 1, 1, 0, 0, 0) | 11000010 | 11 reg:3=d&7 rm:3=n&7 | 00001100 }" "asm"="vcmppd $12, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f,avx512vl" "nodecode" {
  %r = fcmp one <4 x double> %n, %m
  ret <4 x i1> %r
}

define <4 x i1> @VCMPOGEPDk.4xf64(<4 x double> "reg"="yv" %n, <4 x double> "reg"="yv" %m) "enc"="{ evexr(d, m, 1, 1, n, 1, 1, 0, 0, 0) | 11000010 | 11 reg:3=d&7 rm:3=m&7 | 00001101 ; if(((13&3)==0)|((13&3)==3)) evexr(d, n, 1, 1, m, 1, 1, 0, 0, 0) | 11000010 | 11 reg:3=d&7 rm:3=n&7 | 00001101 }" "asm"="vcmppd $13, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f,avx512vl" "nodecode" {
  %r = fcmp oge <4 x double> %n, %m
  ret <4 x i1> %r
}

define <4 x i1> @VCMPOGTPDk.4xf64(<4 x double> "reg"="yv" %n, <4 x double> "reg"="yv" %m) "enc"="{ evexr(d, m, 1, 1, n, 1, 1, 0, 0, 0) | 11000010 | 11 reg:3=d&7 rm:3=m&7 | 00001110 ; if(((14&3)==0)|((14&3)==3)) evexr(d, n, 1, 1, m, 1, 1, 0, 0, 0) | 11000010 | 11 reg:3=d&7 rm:3=n&7 | 00001110 }" "asm"="vcmppd $14, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f,avx512vl" "nodecode" {
  %r = fcmp ogt <4 x double> %n, %m
  ret <4 x i1> %r
}

define <4 x double> @VBLENDMPD.4xf64(<4 x i1> "reg"="kw" %k, <4 x double> "reg"="yv" %a, <4 x double> "reg"="yv" %b) "enc"="{ if(k!=0) evexr(d, a, 2, 1, b, 1, 1, 0, k, 0) | 01100101 | 11 reg:3=d&7 rm:3=a&7 }" "asm"="vblendmpd {a}, {b}, {d} {lb}{k}{rb}" "dclass"="yv" "feature"="avx512f,avx512vl" "nodecode" {
  %r = select <4 x i1> %k, <4 x double> %a, <4 x double> %b
  ret <4 x double> %r
}

define <16 x i1> @VPCMPB.eq.16xi8(<16 x i8> "reg"="xv" %n, <16 x i8> "reg"="xv" %m) "enc"="evexr(d, m, 3, 0, n, 0, 1, 0, 0, 0) | 00111111 | 11 reg:3=d&7 rm:3=m&7 | 00000000" "asm"="vpcmpb $0, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f,avx512vl,avx512bw" "nodecode" {
  %r = icmp eq <16 x i8> %n, %m
  ret <16 x i1> %r
}

define <16 x i1> @VPCMPB.ne.16xi8(<16 x i8> "reg"="xv" %n, <16 x i8> "reg"="xv" %m) "enc"="evexr(d, m, 3, 0, n, 0, 1, 0, 0, 0) | 00111111 | 11 reg:3=d&7 rm:3=m&7 | 00000100" "asm"="vpcmpb $4, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f,avx512vl,avx512bw" "nodecode" {
  %r = icmp ne <16 x i8> %n, %m
  ret <16 x i1> %r
}

define <16 x i1> @VPCMPB.slt.16xi8(<16 x i8> "reg"="xv" %n, <16 x i8> "reg"="xv" %m) "enc"="evexr(d, m, 3, 0, n, 0, 1, 0, 0, 0) | 00111111 | 11 reg:3=d&7 rm:3=m&7 | 00000001" "asm"="vpcmpb $1, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f,avx512vl,avx512bw" "nodecode" {
  %r = icmp slt <16 x i8> %n, %m
  ret <16 x i1> %r
}

define <16 x i1> @VPCMPB.sle.16xi8(<16 x i8> "reg"="xv" %n, <16 x i8> "reg"="xv" %m) "enc"="evexr(d, m, 3, 0, n, 0, 1, 0, 0, 0) | 00111111 | 11 reg:3=d&7 rm:3=m&7 | 00000010" "asm"="vpcmpb $2, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f,avx512vl,avx512bw" "nodecode" {
  %r = icmp sle <16 x i8> %n, %m
  ret <16 x i1> %r
}

define <16 x i1> @VPCMPB.sge.16xi8(<16 x i8> "reg"="xv" %n, <16 x i8> "reg"="xv" %m) "enc"="evexr(d, m, 3, 0, n, 0, 1, 0, 0, 0) | 00111111 | 11 reg:3=d&7 rm:3=m&7 | 00000101" "asm"="vpcmpb $5, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f,avx512vl,avx512bw" "nodecode" {
  %r = icmp sge <16 x i8> %n, %m
  ret <16 x i1> %r
}

define <16 x i1> @VPCMPB.sgt.16xi8(<16 x i8> "reg"="xv" %n, <16 x i8> "reg"="xv" %m) "enc"="evexr(d, m, 3, 0, n, 0, 1, 0, 0, 0) | 00111111 | 11 reg:3=d&7 rm:3=m&7 | 00000110" "asm"="vpcmpb $6, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f,avx512vl,avx512bw" "nodecode" {
  %r = icmp sgt <16 x i8> %n, %m
  ret <16 x i1> %r
}

define <16 x i1> @VPCMPUB.ult.16xi8(<16 x i8> "reg"="xv" %n, <16 x i8> "reg"="xv" %m) "enc"="evexr(d, m, 3, 0, n, 0, 1, 0, 0, 0) | 00111110 | 11 reg:3=d&7 rm:3=m&7 | 00000001" "asm"="vpcmpub $1, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f,avx512vl,avx512bw" "nodecode" {
  %r = icmp ult <16 x i8> %n, %m
  ret <16 x i1> %r
}

define <16 x i1> @VPCMPUB.ule.16xi8(<16 x i8> "reg"="xv" %n, <16 x i8> "reg"="xv" %m) "enc"="evexr(d, m, 3, 0, n, 0, 1, 0, 0, 0) | 00111110 | 11 reg:3=d&7 rm:3=m&7 | 00000010" "asm"="vpcmpub $2, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f,avx512vl,avx512bw" "nodecode" {
  %r = icmp ule <16 x i8> %n, %m
  ret <16 x i1> %r
}

define <16 x i1> @VPCMPUB.uge.16xi8(<16 x i8> "reg"="xv" %n, <16 x i8> "reg"="xv" %m) "enc"="evexr(d, m, 3, 0, n, 0, 1, 0, 0, 0) | 00111110 | 11 reg:3=d&7 rm:3=m&7 | 00000101" "asm"="vpcmpub $5, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f,avx512vl,avx512bw" "nodecode" {
  %r = icmp uge <16 x i8> %n, %m
  ret <16 x i1> %r
}

define <16 x i1> @VPCMPUB.ugt.16xi8(<16 x i8> "reg"="xv" %n, <16 x i8> "reg"="xv" %m) "enc"="evexr(d, m, 3, 0, n, 0, 1, 0, 0, 0) | 00111110 | 11 reg:3=d&7 rm:3=m&7 | 00000110" "asm"="vpcmpub $6, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f,avx512vl,avx512bw" "nodecode" {
  %r = icmp ugt <16 x i8> %n, %m
  ret <16 x i1> %r
}

define <16 x i8> @VPBLENDMB.16xi8(<16 x i1> "reg"="kw" %k, <16 x i8> "reg"="xv" %a, <16 x i8> "reg"="xv" %b) "enc"="{ if(k!=0) evexr(d, a, 2, 0, b, 0, 1, 0, k, 0) | 01100110 | 11 reg:3=d&7 rm:3=a&7 }" "asm"="vpblendmb {a}, {b}, {d} {lb}{k}{rb}" "dclass"="xv" "feature"="avx512f,avx512vl,avx512bw" "nodecode" {
  %r = select <16 x i1> %k, <16 x i8> %a, <16 x i8> %b
  ret <16 x i8> %r
}

define <16 x i8> @VPMOVM2B.16xi8(<16 x i1> "reg"="kw" %k) "enc"="evexr(d, k, 2, 0, 0, 0, 2, 0, 0, 0) | 00101000 | 11 reg:3=d&7 rm:3=k&7" "asm"="vpmovm2b {k}, {d}" "dclass"="xv" "feature"="avx512f,avx512vl,avx512bw" "nodecode" {
  %r = sext <16 x i1> %k to <16 x i8>
  ret <16 x i8> %r
}

define <16 x i8> @VMASKBITS.16xi8(<16 x i1> "reg"="kw" %k) "enc"="evexr(d, k, 2, 0, 0, 0, 2, 0, 0, 0) | 00101000 | 11 reg:3=d&7 rm:3=k&7" "asm"="vpmovm2b {k}, {d}" "dclass"="xv" "feature"="avx512f,avx512vl,avx512bw" "nodecode" {
  %r = call <16 x i8> @llvm.epitome.mask.bits.v16i8(<16 x i1> %k)
  ret <16 x i8> %r
}

define <16 x i8> @VPABSBk.16xi8(<16 x i8> "reg"="xv" %n) "enc"="{ vex(d, 0, n, 2, 0, 0, 0, 1) | 00011100 | 11 reg:3=d&7 rm:3=n&7 ; evexr(d, n, 2, 0, 0, 0, 1, 0, 0, 0) | 00011100 | 11 reg:3=d&7 rm:3=n&7 }" "asm"="vpabsb {n}, {d}" "dclass"="xv" "nomatch" "feature"="avx512f,avx512vl,avx512bw" "nodecode" {
  ret <16 x i8> %n
}

define <16 x i8> @zextk.16xi8(<16 x i1> "reg"="kw" %k) "recipe"="zextk.16xi8.expand" "feature"="avx512f,avx512vl,avx512bw" {
  %r = zext <16 x i1> %k to <16 x i8>
  ret <16 x i8> %r
}

define <16 x i8> @zextk.16xi8.expand(<16 x i1> %k) {
  %s = call <16 x i8> @VPMOVM2B.16xi8(<16 x i1> %k)
  %r = call <16 x i8> @VPABSBk.16xi8(<16 x i8> %s)
  ret <16 x i8> %r
}

define <16 x i1> @VBITSMASK.16xi8(<16 x i8> "reg"="xv" %n) "enc"="evexr(d, n, 2, 0, n, 0, 1, 0, 0, 0) | 00100110 | 11 reg:3=d&7 rm:3=n&7" "asm"="vptestmb {n}, {n}, {d}" "dclass"="kw" "feature"="avx512f,avx512vl,avx512bw" "nodecode" {
  %r = call <16 x i1> @llvm.epitome.bits.mask.v16i8(<16 x i8> %n)
  ret <16 x i1> %r
}

define <8 x i1> @VPCMPW.eq.8xi16(<8 x i16> "reg"="xv" %n, <8 x i16> "reg"="xv" %m) "enc"="evexr(d, m, 3, 1, n, 0, 1, 0, 0, 0) | 00111111 | 11 reg:3=d&7 rm:3=m&7 | 00000000" "asm"="vpcmpw $0, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f,avx512vl,avx512bw" "nodecode" {
  %r = icmp eq <8 x i16> %n, %m
  ret <8 x i1> %r
}

define <8 x i1> @VPCMPW.ne.8xi16(<8 x i16> "reg"="xv" %n, <8 x i16> "reg"="xv" %m) "enc"="evexr(d, m, 3, 1, n, 0, 1, 0, 0, 0) | 00111111 | 11 reg:3=d&7 rm:3=m&7 | 00000100" "asm"="vpcmpw $4, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f,avx512vl,avx512bw" "nodecode" {
  %r = icmp ne <8 x i16> %n, %m
  ret <8 x i1> %r
}

define <8 x i1> @VPCMPW.slt.8xi16(<8 x i16> "reg"="xv" %n, <8 x i16> "reg"="xv" %m) "enc"="evexr(d, m, 3, 1, n, 0, 1, 0, 0, 0) | 00111111 | 11 reg:3=d&7 rm:3=m&7 | 00000001" "asm"="vpcmpw $1, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f,avx512vl,avx512bw" "nodecode" {
  %r = icmp slt <8 x i16> %n, %m
  ret <8 x i1> %r
}

define <8 x i1> @VPCMPW.sle.8xi16(<8 x i16> "reg"="xv" %n, <8 x i16> "reg"="xv" %m) "enc"="evexr(d, m, 3, 1, n, 0, 1, 0, 0, 0) | 00111111 | 11 reg:3=d&7 rm:3=m&7 | 00000010" "asm"="vpcmpw $2, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f,avx512vl,avx512bw" "nodecode" {
  %r = icmp sle <8 x i16> %n, %m
  ret <8 x i1> %r
}

define <8 x i1> @VPCMPW.sge.8xi16(<8 x i16> "reg"="xv" %n, <8 x i16> "reg"="xv" %m) "enc"="evexr(d, m, 3, 1, n, 0, 1, 0, 0, 0) | 00111111 | 11 reg:3=d&7 rm:3=m&7 | 00000101" "asm"="vpcmpw $5, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f,avx512vl,avx512bw" "nodecode" {
  %r = icmp sge <8 x i16> %n, %m
  ret <8 x i1> %r
}

define <8 x i1> @VPCMPW.sgt.8xi16(<8 x i16> "reg"="xv" %n, <8 x i16> "reg"="xv" %m) "enc"="evexr(d, m, 3, 1, n, 0, 1, 0, 0, 0) | 00111111 | 11 reg:3=d&7 rm:3=m&7 | 00000110" "asm"="vpcmpw $6, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f,avx512vl,avx512bw" "nodecode" {
  %r = icmp sgt <8 x i16> %n, %m
  ret <8 x i1> %r
}

define <8 x i1> @VPCMPUW.ult.8xi16(<8 x i16> "reg"="xv" %n, <8 x i16> "reg"="xv" %m) "enc"="evexr(d, m, 3, 1, n, 0, 1, 0, 0, 0) | 00111110 | 11 reg:3=d&7 rm:3=m&7 | 00000001" "asm"="vpcmpuw $1, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f,avx512vl,avx512bw" "nodecode" {
  %r = icmp ult <8 x i16> %n, %m
  ret <8 x i1> %r
}

define <8 x i1> @VPCMPUW.ule.8xi16(<8 x i16> "reg"="xv" %n, <8 x i16> "reg"="xv" %m) "enc"="evexr(d, m, 3, 1, n, 0, 1, 0, 0, 0) | 00111110 | 11 reg:3=d&7 rm:3=m&7 | 00000010" "asm"="vpcmpuw $2, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f,avx512vl,avx512bw" "nodecode" {
  %r = icmp ule <8 x i16> %n, %m
  ret <8 x i1> %r
}

define <8 x i1> @VPCMPUW.uge.8xi16(<8 x i16> "reg"="xv" %n, <8 x i16> "reg"="xv" %m) "enc"="evexr(d, m, 3, 1, n, 0, 1, 0, 0, 0) | 00111110 | 11 reg:3=d&7 rm:3=m&7 | 00000101" "asm"="vpcmpuw $5, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f,avx512vl,avx512bw" "nodecode" {
  %r = icmp uge <8 x i16> %n, %m
  ret <8 x i1> %r
}

define <8 x i1> @VPCMPUW.ugt.8xi16(<8 x i16> "reg"="xv" %n, <8 x i16> "reg"="xv" %m) "enc"="evexr(d, m, 3, 1, n, 0, 1, 0, 0, 0) | 00111110 | 11 reg:3=d&7 rm:3=m&7 | 00000110" "asm"="vpcmpuw $6, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f,avx512vl,avx512bw" "nodecode" {
  %r = icmp ugt <8 x i16> %n, %m
  ret <8 x i1> %r
}

define <8 x i16> @VPBLENDMW.8xi16(<8 x i1> "reg"="kw" %k, <8 x i16> "reg"="xv" %a, <8 x i16> "reg"="xv" %b) "enc"="{ if(k!=0) evexr(d, a, 2, 1, b, 0, 1, 0, k, 0) | 01100110 | 11 reg:3=d&7 rm:3=a&7 }" "asm"="vpblendmw {a}, {b}, {d} {lb}{k}{rb}" "dclass"="xv" "feature"="avx512f,avx512vl,avx512bw" "nodecode" {
  %r = select <8 x i1> %k, <8 x i16> %a, <8 x i16> %b
  ret <8 x i16> %r
}

define <8 x i16> @VPMOVM2W.8xi16(<8 x i1> "reg"="kw" %k) "enc"="evexr(d, k, 2, 1, 0, 0, 2, 0, 0, 0) | 00101000 | 11 reg:3=d&7 rm:3=k&7" "asm"="vpmovm2w {k}, {d}" "dclass"="xv" "feature"="avx512f,avx512vl,avx512bw" "nodecode" {
  %r = sext <8 x i1> %k to <8 x i16>
  ret <8 x i16> %r
}

define <8 x i16> @VMASKBITS.8xi16(<8 x i1> "reg"="kw" %k) "enc"="evexr(d, k, 2, 1, 0, 0, 2, 0, 0, 0) | 00101000 | 11 reg:3=d&7 rm:3=k&7" "asm"="vpmovm2w {k}, {d}" "dclass"="xv" "feature"="avx512f,avx512vl,avx512bw" "nodecode" {
  %r = call <8 x i16> @llvm.epitome.mask.bits.v8i16(<8 x i1> %k)
  ret <8 x i16> %r
}

define <8 x i16> @VPABSWk.8xi16(<8 x i16> "reg"="xv" %n) "enc"="{ vex(d, 0, n, 2, 0, 0, 0, 1) | 00011101 | 11 reg:3=d&7 rm:3=n&7 ; evexr(d, n, 2, 0, 0, 0, 1, 0, 0, 0) | 00011101 | 11 reg:3=d&7 rm:3=n&7 }" "asm"="vpabsw {n}, {d}" "dclass"="xv" "nomatch" "feature"="avx512f,avx512vl,avx512bw" "nodecode" {
  ret <8 x i16> %n
}

define <8 x i16> @zextk.8xi16(<8 x i1> "reg"="kw" %k) "recipe"="zextk.8xi16.expand" "feature"="avx512f,avx512vl,avx512bw" {
  %r = zext <8 x i1> %k to <8 x i16>
  ret <8 x i16> %r
}

define <8 x i16> @zextk.8xi16.expand(<8 x i1> %k) {
  %s = call <8 x i16> @VPMOVM2W.8xi16(<8 x i1> %k)
  %r = call <8 x i16> @VPABSWk.8xi16(<8 x i16> %s)
  ret <8 x i16> %r
}

define <8 x i1> @VBITSMASK.8xi16(<8 x i16> "reg"="xv" %n) "enc"="evexr(d, n, 2, 1, n, 0, 1, 0, 0, 0) | 00100110 | 11 reg:3=d&7 rm:3=n&7" "asm"="vptestmw {n}, {n}, {d}" "dclass"="kw" "feature"="avx512f,avx512vl,avx512bw" "nodecode" {
  %r = call <8 x i1> @llvm.epitome.bits.mask.v8i16(<8 x i16> %n)
  ret <8 x i1> %r
}

define <4 x i1> @VPCMPD.eq.4xi32(<4 x i32> "reg"="xv" %n, <4 x i32> "reg"="xv" %m) "enc"="evexr(d, m, 3, 0, n, 0, 1, 0, 0, 0) | 00011111 | 11 reg:3=d&7 rm:3=m&7 | 00000000" "asm"="vpcmpd $0, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f,avx512vl" "nodecode" {
  %r = icmp eq <4 x i32> %n, %m
  ret <4 x i1> %r
}

define <4 x i1> @VPCMPD.ne.4xi32(<4 x i32> "reg"="xv" %n, <4 x i32> "reg"="xv" %m) "enc"="evexr(d, m, 3, 0, n, 0, 1, 0, 0, 0) | 00011111 | 11 reg:3=d&7 rm:3=m&7 | 00000100" "asm"="vpcmpd $4, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f,avx512vl" "nodecode" {
  %r = icmp ne <4 x i32> %n, %m
  ret <4 x i1> %r
}

define <4 x i1> @VPCMPD.slt.4xi32(<4 x i32> "reg"="xv" %n, <4 x i32> "reg"="xv" %m) "enc"="evexr(d, m, 3, 0, n, 0, 1, 0, 0, 0) | 00011111 | 11 reg:3=d&7 rm:3=m&7 | 00000001" "asm"="vpcmpd $1, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f,avx512vl" "nodecode" {
  %r = icmp slt <4 x i32> %n, %m
  ret <4 x i1> %r
}

define <4 x i1> @VPCMPD.sle.4xi32(<4 x i32> "reg"="xv" %n, <4 x i32> "reg"="xv" %m) "enc"="evexr(d, m, 3, 0, n, 0, 1, 0, 0, 0) | 00011111 | 11 reg:3=d&7 rm:3=m&7 | 00000010" "asm"="vpcmpd $2, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f,avx512vl" "nodecode" {
  %r = icmp sle <4 x i32> %n, %m
  ret <4 x i1> %r
}

define <4 x i1> @VPCMPD.sge.4xi32(<4 x i32> "reg"="xv" %n, <4 x i32> "reg"="xv" %m) "enc"="evexr(d, m, 3, 0, n, 0, 1, 0, 0, 0) | 00011111 | 11 reg:3=d&7 rm:3=m&7 | 00000101" "asm"="vpcmpd $5, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f,avx512vl" "nodecode" {
  %r = icmp sge <4 x i32> %n, %m
  ret <4 x i1> %r
}

define <4 x i1> @VPCMPD.sgt.4xi32(<4 x i32> "reg"="xv" %n, <4 x i32> "reg"="xv" %m) "enc"="evexr(d, m, 3, 0, n, 0, 1, 0, 0, 0) | 00011111 | 11 reg:3=d&7 rm:3=m&7 | 00000110" "asm"="vpcmpd $6, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f,avx512vl" "nodecode" {
  %r = icmp sgt <4 x i32> %n, %m
  ret <4 x i1> %r
}

define <4 x i1> @VPCMPUD.ult.4xi32(<4 x i32> "reg"="xv" %n, <4 x i32> "reg"="xv" %m) "enc"="evexr(d, m, 3, 0, n, 0, 1, 0, 0, 0) | 00011110 | 11 reg:3=d&7 rm:3=m&7 | 00000001" "asm"="vpcmpud $1, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f,avx512vl" "nodecode" {
  %r = icmp ult <4 x i32> %n, %m
  ret <4 x i1> %r
}

define <4 x i1> @VPCMPUD.ule.4xi32(<4 x i32> "reg"="xv" %n, <4 x i32> "reg"="xv" %m) "enc"="evexr(d, m, 3, 0, n, 0, 1, 0, 0, 0) | 00011110 | 11 reg:3=d&7 rm:3=m&7 | 00000010" "asm"="vpcmpud $2, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f,avx512vl" "nodecode" {
  %r = icmp ule <4 x i32> %n, %m
  ret <4 x i1> %r
}

define <4 x i1> @VPCMPUD.uge.4xi32(<4 x i32> "reg"="xv" %n, <4 x i32> "reg"="xv" %m) "enc"="evexr(d, m, 3, 0, n, 0, 1, 0, 0, 0) | 00011110 | 11 reg:3=d&7 rm:3=m&7 | 00000101" "asm"="vpcmpud $5, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f,avx512vl" "nodecode" {
  %r = icmp uge <4 x i32> %n, %m
  ret <4 x i1> %r
}

define <4 x i1> @VPCMPUD.ugt.4xi32(<4 x i32> "reg"="xv" %n, <4 x i32> "reg"="xv" %m) "enc"="evexr(d, m, 3, 0, n, 0, 1, 0, 0, 0) | 00011110 | 11 reg:3=d&7 rm:3=m&7 | 00000110" "asm"="vpcmpud $6, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f,avx512vl" "nodecode" {
  %r = icmp ugt <4 x i32> %n, %m
  ret <4 x i1> %r
}

define <4 x i32> @VPBLENDMD.4xi32(<4 x i1> "reg"="kw" %k, <4 x i32> "reg"="xv" %a, <4 x i32> "reg"="xv" %b) "enc"="{ if(k!=0) evexr(d, a, 2, 0, b, 0, 1, 0, k, 0) | 01100100 | 11 reg:3=d&7 rm:3=a&7 }" "asm"="vpblendmd {a}, {b}, {d} {lb}{k}{rb}" "dclass"="xv" "feature"="avx512f,avx512vl" "nodecode" {
  %r = select <4 x i1> %k, <4 x i32> %a, <4 x i32> %b
  ret <4 x i32> %r
}

define <4 x i32> @VPMOVM2D.4xi32(<4 x i1> "reg"="kw" %k) "enc"="{ if(k!=0) evexr(d, d, 3, 0, d, 0, 1, 0, k, 1) | 00100101 | 11 reg:3=d&7 rm:3=d&7 | 11111111 }" "asm"="vpternlogd $255, {d}, {d}, {d} {lb}{k}{rb} {lb}z{rb}" "dclass"="xv" "feature"="avx512f,avx512vl" "nodecode" {
  %r = sext <4 x i1> %k to <4 x i32>
  ret <4 x i32> %r
}

define <4 x i32> @VMASKBITS.4xi32(<4 x i1> "reg"="kw" %k) "enc"="{ if(k!=0) evexr(d, d, 3, 0, d, 0, 1, 0, k, 1) | 00100101 | 11 reg:3=d&7 rm:3=d&7 | 11111111 }" "asm"="vpternlogd $255, {d}, {d}, {d} {lb}{k}{rb} {lb}z{rb}" "dclass"="xv" "feature"="avx512f,avx512vl" "nodecode" {
  %r = call <4 x i32> @llvm.epitome.mask.bits.v4i32(<4 x i1> %k)
  ret <4 x i32> %r
}

define <4 x i32> @VPABSDk.4xi32(<4 x i32> "reg"="xv" %n) "enc"="{ vex(d, 0, n, 2, 0, 0, 0, 1) | 00011110 | 11 reg:3=d&7 rm:3=n&7 ; evexr(d, n, 2, 0, 0, 0, 1, 0, 0, 0) | 00011110 | 11 reg:3=d&7 rm:3=n&7 }" "asm"="vpabsd {n}, {d}" "dclass"="xv" "nomatch" "feature"="avx512f,avx512vl" "nodecode" {
  ret <4 x i32> %n
}

define <4 x i32> @zextk.4xi32(<4 x i1> "reg"="kw" %k) "recipe"="zextk.4xi32.expand" "feature"="avx512f,avx512vl" {
  %r = zext <4 x i1> %k to <4 x i32>
  ret <4 x i32> %r
}

define <4 x i32> @zextk.4xi32.expand(<4 x i1> %k) {
  %s = call <4 x i32> @VPMOVM2D.4xi32(<4 x i1> %k)
  %r = call <4 x i32> @VPABSDk.4xi32(<4 x i32> %s)
  ret <4 x i32> %r
}

define <4 x i1> @VBITSMASK.4xi32(<4 x i32> "reg"="xv" %n) "enc"="evexr(d, n, 2, 0, n, 0, 1, 0, 0, 0) | 00100111 | 11 reg:3=d&7 rm:3=n&7" "asm"="vptestmd {n}, {n}, {d}" "dclass"="kw" "feature"="avx512f,avx512vl" "nodecode" {
  %r = call <4 x i1> @llvm.epitome.bits.mask.v4i32(<4 x i32> %n)
  ret <4 x i1> %r
}

define <2 x i1> @VPCMPQ.eq.2xi64(<2 x i64> "reg"="xv" %n, <2 x i64> "reg"="xv" %m) "enc"="evexr(d, m, 3, 1, n, 0, 1, 0, 0, 0) | 00011111 | 11 reg:3=d&7 rm:3=m&7 | 00000000" "asm"="vpcmpq $0, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f,avx512vl" "nodecode" {
  %r = icmp eq <2 x i64> %n, %m
  ret <2 x i1> %r
}

define <2 x i1> @VPCMPQ.ne.2xi64(<2 x i64> "reg"="xv" %n, <2 x i64> "reg"="xv" %m) "enc"="evexr(d, m, 3, 1, n, 0, 1, 0, 0, 0) | 00011111 | 11 reg:3=d&7 rm:3=m&7 | 00000100" "asm"="vpcmpq $4, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f,avx512vl" "nodecode" {
  %r = icmp ne <2 x i64> %n, %m
  ret <2 x i1> %r
}

define <2 x i1> @VPCMPQ.slt.2xi64(<2 x i64> "reg"="xv" %n, <2 x i64> "reg"="xv" %m) "enc"="evexr(d, m, 3, 1, n, 0, 1, 0, 0, 0) | 00011111 | 11 reg:3=d&7 rm:3=m&7 | 00000001" "asm"="vpcmpq $1, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f,avx512vl" "nodecode" {
  %r = icmp slt <2 x i64> %n, %m
  ret <2 x i1> %r
}

define <2 x i1> @VPCMPQ.sle.2xi64(<2 x i64> "reg"="xv" %n, <2 x i64> "reg"="xv" %m) "enc"="evexr(d, m, 3, 1, n, 0, 1, 0, 0, 0) | 00011111 | 11 reg:3=d&7 rm:3=m&7 | 00000010" "asm"="vpcmpq $2, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f,avx512vl" "nodecode" {
  %r = icmp sle <2 x i64> %n, %m
  ret <2 x i1> %r
}

define <2 x i1> @VPCMPQ.sge.2xi64(<2 x i64> "reg"="xv" %n, <2 x i64> "reg"="xv" %m) "enc"="evexr(d, m, 3, 1, n, 0, 1, 0, 0, 0) | 00011111 | 11 reg:3=d&7 rm:3=m&7 | 00000101" "asm"="vpcmpq $5, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f,avx512vl" "nodecode" {
  %r = icmp sge <2 x i64> %n, %m
  ret <2 x i1> %r
}

define <2 x i1> @VPCMPQ.sgt.2xi64(<2 x i64> "reg"="xv" %n, <2 x i64> "reg"="xv" %m) "enc"="evexr(d, m, 3, 1, n, 0, 1, 0, 0, 0) | 00011111 | 11 reg:3=d&7 rm:3=m&7 | 00000110" "asm"="vpcmpq $6, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f,avx512vl" "nodecode" {
  %r = icmp sgt <2 x i64> %n, %m
  ret <2 x i1> %r
}

define <2 x i1> @VPCMPUQ.ult.2xi64(<2 x i64> "reg"="xv" %n, <2 x i64> "reg"="xv" %m) "enc"="evexr(d, m, 3, 1, n, 0, 1, 0, 0, 0) | 00011110 | 11 reg:3=d&7 rm:3=m&7 | 00000001" "asm"="vpcmpuq $1, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f,avx512vl" "nodecode" {
  %r = icmp ult <2 x i64> %n, %m
  ret <2 x i1> %r
}

define <2 x i1> @VPCMPUQ.ule.2xi64(<2 x i64> "reg"="xv" %n, <2 x i64> "reg"="xv" %m) "enc"="evexr(d, m, 3, 1, n, 0, 1, 0, 0, 0) | 00011110 | 11 reg:3=d&7 rm:3=m&7 | 00000010" "asm"="vpcmpuq $2, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f,avx512vl" "nodecode" {
  %r = icmp ule <2 x i64> %n, %m
  ret <2 x i1> %r
}

define <2 x i1> @VPCMPUQ.uge.2xi64(<2 x i64> "reg"="xv" %n, <2 x i64> "reg"="xv" %m) "enc"="evexr(d, m, 3, 1, n, 0, 1, 0, 0, 0) | 00011110 | 11 reg:3=d&7 rm:3=m&7 | 00000101" "asm"="vpcmpuq $5, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f,avx512vl" "nodecode" {
  %r = icmp uge <2 x i64> %n, %m
  ret <2 x i1> %r
}

define <2 x i1> @VPCMPUQ.ugt.2xi64(<2 x i64> "reg"="xv" %n, <2 x i64> "reg"="xv" %m) "enc"="evexr(d, m, 3, 1, n, 0, 1, 0, 0, 0) | 00011110 | 11 reg:3=d&7 rm:3=m&7 | 00000110" "asm"="vpcmpuq $6, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f,avx512vl" "nodecode" {
  %r = icmp ugt <2 x i64> %n, %m
  ret <2 x i1> %r
}

define <2 x i64> @VPBLENDMQ.2xi64(<2 x i1> "reg"="kw" %k, <2 x i64> "reg"="xv" %a, <2 x i64> "reg"="xv" %b) "enc"="{ if(k!=0) evexr(d, a, 2, 1, b, 0, 1, 0, k, 0) | 01100100 | 11 reg:3=d&7 rm:3=a&7 }" "asm"="vpblendmq {a}, {b}, {d} {lb}{k}{rb}" "dclass"="xv" "feature"="avx512f,avx512vl" "nodecode" {
  %r = select <2 x i1> %k, <2 x i64> %a, <2 x i64> %b
  ret <2 x i64> %r
}

define <2 x i64> @VPMOVM2Q.2xi64(<2 x i1> "reg"="kw" %k) "enc"="{ if(k!=0) evexr(d, d, 3, 1, d, 0, 1, 0, k, 1) | 00100101 | 11 reg:3=d&7 rm:3=d&7 | 11111111 }" "asm"="vpternlogq $255, {d}, {d}, {d} {lb}{k}{rb} {lb}z{rb}" "dclass"="xv" "feature"="avx512f,avx512vl" "nodecode" {
  %r = sext <2 x i1> %k to <2 x i64>
  ret <2 x i64> %r
}

define <2 x i64> @VMASKBITS.2xi64(<2 x i1> "reg"="kw" %k) "enc"="{ if(k!=0) evexr(d, d, 3, 1, d, 0, 1, 0, k, 1) | 00100101 | 11 reg:3=d&7 rm:3=d&7 | 11111111 }" "asm"="vpternlogq $255, {d}, {d}, {d} {lb}{k}{rb} {lb}z{rb}" "dclass"="xv" "feature"="avx512f,avx512vl" "nodecode" {
  %r = call <2 x i64> @llvm.epitome.mask.bits.v2i64(<2 x i1> %k)
  ret <2 x i64> %r
}

define <2 x i64> @VPABSQk.2xi64(<2 x i64> "reg"="xv" %n) "enc"="evexr(d, n, 2, 1, 0, 0, 1, 0, 0, 0) | 00011111 | 11 reg:3=d&7 rm:3=n&7" "asm"="vpabsq {n}, {d}" "dclass"="xv" "nomatch" "feature"="avx512f,avx512vl" "nodecode" {
  ret <2 x i64> %n
}

define <2 x i64> @zextk.2xi64(<2 x i1> "reg"="kw" %k) "recipe"="zextk.2xi64.expand" "feature"="avx512f,avx512vl" {
  %r = zext <2 x i1> %k to <2 x i64>
  ret <2 x i64> %r
}

define <2 x i64> @zextk.2xi64.expand(<2 x i1> %k) {
  %s = call <2 x i64> @VPMOVM2Q.2xi64(<2 x i1> %k)
  %r = call <2 x i64> @VPABSQk.2xi64(<2 x i64> %s)
  ret <2 x i64> %r
}

define <2 x i1> @VBITSMASK.2xi64(<2 x i64> "reg"="xv" %n) "enc"="evexr(d, n, 2, 1, n, 0, 1, 0, 0, 0) | 00100111 | 11 reg:3=d&7 rm:3=n&7" "asm"="vptestmq {n}, {n}, {d}" "dclass"="kw" "feature"="avx512f,avx512vl" "nodecode" {
  %r = call <2 x i1> @llvm.epitome.bits.mask.v2i64(<2 x i64> %n)
  ret <2 x i1> %r
}

define <4 x i1> @VCMPOEQPSk.4xf32(<4 x float> "reg"="xv" %n, <4 x float> "reg"="xv" %m) "enc"="{ evexr(d, m, 1, 0, n, 0, 0, 0, 0, 0) | 11000010 | 11 reg:3=d&7 rm:3=m&7 | 00000000 ; if(((0&3)==0)|((0&3)==3)) evexr(d, n, 1, 0, m, 0, 0, 0, 0, 0) | 11000010 | 11 reg:3=d&7 rm:3=n&7 | 00000000 }" "asm"="vcmpps $0, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f,avx512vl" "nodecode" {
  %r = fcmp oeq <4 x float> %n, %m
  ret <4 x i1> %r
}

define <4 x i1> @VCMPOLTPSk.4xf32(<4 x float> "reg"="xv" %n, <4 x float> "reg"="xv" %m) "enc"="{ evexr(d, m, 1, 0, n, 0, 0, 0, 0, 0) | 11000010 | 11 reg:3=d&7 rm:3=m&7 | 00000001 ; if(((1&3)==0)|((1&3)==3)) evexr(d, n, 1, 0, m, 0, 0, 0, 0, 0) | 11000010 | 11 reg:3=d&7 rm:3=n&7 | 00000001 }" "asm"="vcmpps $1, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f,avx512vl" "nodecode" {
  %r = fcmp olt <4 x float> %n, %m
  ret <4 x i1> %r
}

define <4 x i1> @VCMPOLEPSk.4xf32(<4 x float> "reg"="xv" %n, <4 x float> "reg"="xv" %m) "enc"="{ evexr(d, m, 1, 0, n, 0, 0, 0, 0, 0) | 11000010 | 11 reg:3=d&7 rm:3=m&7 | 00000010 ; if(((2&3)==0)|((2&3)==3)) evexr(d, n, 1, 0, m, 0, 0, 0, 0, 0) | 11000010 | 11 reg:3=d&7 rm:3=n&7 | 00000010 }" "asm"="vcmpps $2, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f,avx512vl" "nodecode" {
  %r = fcmp ole <4 x float> %n, %m
  ret <4 x i1> %r
}

define <4 x i1> @VCMPUNOPSk.4xf32(<4 x float> "reg"="xv" %n, <4 x float> "reg"="xv" %m) "enc"="{ evexr(d, m, 1, 0, n, 0, 0, 0, 0, 0) | 11000010 | 11 reg:3=d&7 rm:3=m&7 | 00000011 ; if(((3&3)==0)|((3&3)==3)) evexr(d, n, 1, 0, m, 0, 0, 0, 0, 0) | 11000010 | 11 reg:3=d&7 rm:3=n&7 | 00000011 }" "asm"="vcmpps $3, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f,avx512vl" "nodecode" {
  %r = fcmp uno <4 x float> %n, %m
  ret <4 x i1> %r
}

define <4 x i1> @VCMPUNEPSk.4xf32(<4 x float> "reg"="xv" %n, <4 x float> "reg"="xv" %m) "enc"="{ evexr(d, m, 1, 0, n, 0, 0, 0, 0, 0) | 11000010 | 11 reg:3=d&7 rm:3=m&7 | 00000100 ; if(((4&3)==0)|((4&3)==3)) evexr(d, n, 1, 0, m, 0, 0, 0, 0, 0) | 11000010 | 11 reg:3=d&7 rm:3=n&7 | 00000100 }" "asm"="vcmpps $4, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f,avx512vl" "nodecode" {
  %r = fcmp une <4 x float> %n, %m
  ret <4 x i1> %r
}

define <4 x i1> @VCMPUGEPSk.4xf32(<4 x float> "reg"="xv" %n, <4 x float> "reg"="xv" %m) "enc"="{ evexr(d, m, 1, 0, n, 0, 0, 0, 0, 0) | 11000010 | 11 reg:3=d&7 rm:3=m&7 | 00000101 ; if(((5&3)==0)|((5&3)==3)) evexr(d, n, 1, 0, m, 0, 0, 0, 0, 0) | 11000010 | 11 reg:3=d&7 rm:3=n&7 | 00000101 }" "asm"="vcmpps $5, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f,avx512vl" "nodecode" {
  %r = fcmp uge <4 x float> %n, %m
  ret <4 x i1> %r
}

define <4 x i1> @VCMPUGTPSk.4xf32(<4 x float> "reg"="xv" %n, <4 x float> "reg"="xv" %m) "enc"="{ evexr(d, m, 1, 0, n, 0, 0, 0, 0, 0) | 11000010 | 11 reg:3=d&7 rm:3=m&7 | 00000110 ; if(((6&3)==0)|((6&3)==3)) evexr(d, n, 1, 0, m, 0, 0, 0, 0, 0) | 11000010 | 11 reg:3=d&7 rm:3=n&7 | 00000110 }" "asm"="vcmpps $6, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f,avx512vl" "nodecode" {
  %r = fcmp ugt <4 x float> %n, %m
  ret <4 x i1> %r
}

define <4 x i1> @VCMPORDPSk.4xf32(<4 x float> "reg"="xv" %n, <4 x float> "reg"="xv" %m) "enc"="{ evexr(d, m, 1, 0, n, 0, 0, 0, 0, 0) | 11000010 | 11 reg:3=d&7 rm:3=m&7 | 00000111 ; if(((7&3)==0)|((7&3)==3)) evexr(d, n, 1, 0, m, 0, 0, 0, 0, 0) | 11000010 | 11 reg:3=d&7 rm:3=n&7 | 00000111 }" "asm"="vcmpps $7, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f,avx512vl" "nodecode" {
  %r = fcmp ord <4 x float> %n, %m
  ret <4 x i1> %r
}

define <4 x i1> @VCMPUEQPSk.4xf32(<4 x float> "reg"="xv" %n, <4 x float> "reg"="xv" %m) "enc"="{ evexr(d, m, 1, 0, n, 0, 0, 0, 0, 0) | 11000010 | 11 reg:3=d&7 rm:3=m&7 | 00001000 ; if(((8&3)==0)|((8&3)==3)) evexr(d, n, 1, 0, m, 0, 0, 0, 0, 0) | 11000010 | 11 reg:3=d&7 rm:3=n&7 | 00001000 }" "asm"="vcmpps $8, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f,avx512vl" "nodecode" {
  %r = fcmp ueq <4 x float> %n, %m
  ret <4 x i1> %r
}

define <4 x i1> @VCMPULTPSk.4xf32(<4 x float> "reg"="xv" %n, <4 x float> "reg"="xv" %m) "enc"="{ evexr(d, m, 1, 0, n, 0, 0, 0, 0, 0) | 11000010 | 11 reg:3=d&7 rm:3=m&7 | 00001001 ; if(((9&3)==0)|((9&3)==3)) evexr(d, n, 1, 0, m, 0, 0, 0, 0, 0) | 11000010 | 11 reg:3=d&7 rm:3=n&7 | 00001001 }" "asm"="vcmpps $9, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f,avx512vl" "nodecode" {
  %r = fcmp ult <4 x float> %n, %m
  ret <4 x i1> %r
}

define <4 x i1> @VCMPULEPSk.4xf32(<4 x float> "reg"="xv" %n, <4 x float> "reg"="xv" %m) "enc"="{ evexr(d, m, 1, 0, n, 0, 0, 0, 0, 0) | 11000010 | 11 reg:3=d&7 rm:3=m&7 | 00001010 ; if(((10&3)==0)|((10&3)==3)) evexr(d, n, 1, 0, m, 0, 0, 0, 0, 0) | 11000010 | 11 reg:3=d&7 rm:3=n&7 | 00001010 }" "asm"="vcmpps $10, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f,avx512vl" "nodecode" {
  %r = fcmp ule <4 x float> %n, %m
  ret <4 x i1> %r
}

define <4 x i1> @VCMPONEPSk.4xf32(<4 x float> "reg"="xv" %n, <4 x float> "reg"="xv" %m) "enc"="{ evexr(d, m, 1, 0, n, 0, 0, 0, 0, 0) | 11000010 | 11 reg:3=d&7 rm:3=m&7 | 00001100 ; if(((12&3)==0)|((12&3)==3)) evexr(d, n, 1, 0, m, 0, 0, 0, 0, 0) | 11000010 | 11 reg:3=d&7 rm:3=n&7 | 00001100 }" "asm"="vcmpps $12, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f,avx512vl" "nodecode" {
  %r = fcmp one <4 x float> %n, %m
  ret <4 x i1> %r
}

define <4 x i1> @VCMPOGEPSk.4xf32(<4 x float> "reg"="xv" %n, <4 x float> "reg"="xv" %m) "enc"="{ evexr(d, m, 1, 0, n, 0, 0, 0, 0, 0) | 11000010 | 11 reg:3=d&7 rm:3=m&7 | 00001101 ; if(((13&3)==0)|((13&3)==3)) evexr(d, n, 1, 0, m, 0, 0, 0, 0, 0) | 11000010 | 11 reg:3=d&7 rm:3=n&7 | 00001101 }" "asm"="vcmpps $13, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f,avx512vl" "nodecode" {
  %r = fcmp oge <4 x float> %n, %m
  ret <4 x i1> %r
}

define <4 x i1> @VCMPOGTPSk.4xf32(<4 x float> "reg"="xv" %n, <4 x float> "reg"="xv" %m) "enc"="{ evexr(d, m, 1, 0, n, 0, 0, 0, 0, 0) | 11000010 | 11 reg:3=d&7 rm:3=m&7 | 00001110 ; if(((14&3)==0)|((14&3)==3)) evexr(d, n, 1, 0, m, 0, 0, 0, 0, 0) | 11000010 | 11 reg:3=d&7 rm:3=n&7 | 00001110 }" "asm"="vcmpps $14, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f,avx512vl" "nodecode" {
  %r = fcmp ogt <4 x float> %n, %m
  ret <4 x i1> %r
}

define <4 x float> @VBLENDMPS.4xf32(<4 x i1> "reg"="kw" %k, <4 x float> "reg"="xv" %a, <4 x float> "reg"="xv" %b) "enc"="{ if(k!=0) evexr(d, a, 2, 0, b, 0, 1, 0, k, 0) | 01100101 | 11 reg:3=d&7 rm:3=a&7 }" "asm"="vblendmps {a}, {b}, {d} {lb}{k}{rb}" "dclass"="xv" "feature"="avx512f,avx512vl" "nodecode" {
  %r = select <4 x i1> %k, <4 x float> %a, <4 x float> %b
  ret <4 x float> %r
}

define <2 x i1> @VCMPOEQPDk.2xf64(<2 x double> "reg"="xv" %n, <2 x double> "reg"="xv" %m) "enc"="{ evexr(d, m, 1, 1, n, 0, 1, 0, 0, 0) | 11000010 | 11 reg:3=d&7 rm:3=m&7 | 00000000 ; if(((0&3)==0)|((0&3)==3)) evexr(d, n, 1, 1, m, 0, 1, 0, 0, 0) | 11000010 | 11 reg:3=d&7 rm:3=n&7 | 00000000 }" "asm"="vcmppd $0, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f,avx512vl" "nodecode" {
  %r = fcmp oeq <2 x double> %n, %m
  ret <2 x i1> %r
}

define <2 x i1> @VCMPOLTPDk.2xf64(<2 x double> "reg"="xv" %n, <2 x double> "reg"="xv" %m) "enc"="{ evexr(d, m, 1, 1, n, 0, 1, 0, 0, 0) | 11000010 | 11 reg:3=d&7 rm:3=m&7 | 00000001 ; if(((1&3)==0)|((1&3)==3)) evexr(d, n, 1, 1, m, 0, 1, 0, 0, 0) | 11000010 | 11 reg:3=d&7 rm:3=n&7 | 00000001 }" "asm"="vcmppd $1, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f,avx512vl" "nodecode" {
  %r = fcmp olt <2 x double> %n, %m
  ret <2 x i1> %r
}

define <2 x i1> @VCMPOLEPDk.2xf64(<2 x double> "reg"="xv" %n, <2 x double> "reg"="xv" %m) "enc"="{ evexr(d, m, 1, 1, n, 0, 1, 0, 0, 0) | 11000010 | 11 reg:3=d&7 rm:3=m&7 | 00000010 ; if(((2&3)==0)|((2&3)==3)) evexr(d, n, 1, 1, m, 0, 1, 0, 0, 0) | 11000010 | 11 reg:3=d&7 rm:3=n&7 | 00000010 }" "asm"="vcmppd $2, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f,avx512vl" "nodecode" {
  %r = fcmp ole <2 x double> %n, %m
  ret <2 x i1> %r
}

define <2 x i1> @VCMPUNOPDk.2xf64(<2 x double> "reg"="xv" %n, <2 x double> "reg"="xv" %m) "enc"="{ evexr(d, m, 1, 1, n, 0, 1, 0, 0, 0) | 11000010 | 11 reg:3=d&7 rm:3=m&7 | 00000011 ; if(((3&3)==0)|((3&3)==3)) evexr(d, n, 1, 1, m, 0, 1, 0, 0, 0) | 11000010 | 11 reg:3=d&7 rm:3=n&7 | 00000011 }" "asm"="vcmppd $3, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f,avx512vl" "nodecode" {
  %r = fcmp uno <2 x double> %n, %m
  ret <2 x i1> %r
}

define <2 x i1> @VCMPUNEPDk.2xf64(<2 x double> "reg"="xv" %n, <2 x double> "reg"="xv" %m) "enc"="{ evexr(d, m, 1, 1, n, 0, 1, 0, 0, 0) | 11000010 | 11 reg:3=d&7 rm:3=m&7 | 00000100 ; if(((4&3)==0)|((4&3)==3)) evexr(d, n, 1, 1, m, 0, 1, 0, 0, 0) | 11000010 | 11 reg:3=d&7 rm:3=n&7 | 00000100 }" "asm"="vcmppd $4, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f,avx512vl" "nodecode" {
  %r = fcmp une <2 x double> %n, %m
  ret <2 x i1> %r
}

define <2 x i1> @VCMPUGEPDk.2xf64(<2 x double> "reg"="xv" %n, <2 x double> "reg"="xv" %m) "enc"="{ evexr(d, m, 1, 1, n, 0, 1, 0, 0, 0) | 11000010 | 11 reg:3=d&7 rm:3=m&7 | 00000101 ; if(((5&3)==0)|((5&3)==3)) evexr(d, n, 1, 1, m, 0, 1, 0, 0, 0) | 11000010 | 11 reg:3=d&7 rm:3=n&7 | 00000101 }" "asm"="vcmppd $5, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f,avx512vl" "nodecode" {
  %r = fcmp uge <2 x double> %n, %m
  ret <2 x i1> %r
}

define <2 x i1> @VCMPUGTPDk.2xf64(<2 x double> "reg"="xv" %n, <2 x double> "reg"="xv" %m) "enc"="{ evexr(d, m, 1, 1, n, 0, 1, 0, 0, 0) | 11000010 | 11 reg:3=d&7 rm:3=m&7 | 00000110 ; if(((6&3)==0)|((6&3)==3)) evexr(d, n, 1, 1, m, 0, 1, 0, 0, 0) | 11000010 | 11 reg:3=d&7 rm:3=n&7 | 00000110 }" "asm"="vcmppd $6, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f,avx512vl" "nodecode" {
  %r = fcmp ugt <2 x double> %n, %m
  ret <2 x i1> %r
}

define <2 x i1> @VCMPORDPDk.2xf64(<2 x double> "reg"="xv" %n, <2 x double> "reg"="xv" %m) "enc"="{ evexr(d, m, 1, 1, n, 0, 1, 0, 0, 0) | 11000010 | 11 reg:3=d&7 rm:3=m&7 | 00000111 ; if(((7&3)==0)|((7&3)==3)) evexr(d, n, 1, 1, m, 0, 1, 0, 0, 0) | 11000010 | 11 reg:3=d&7 rm:3=n&7 | 00000111 }" "asm"="vcmppd $7, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f,avx512vl" "nodecode" {
  %r = fcmp ord <2 x double> %n, %m
  ret <2 x i1> %r
}

define <2 x i1> @VCMPUEQPDk.2xf64(<2 x double> "reg"="xv" %n, <2 x double> "reg"="xv" %m) "enc"="{ evexr(d, m, 1, 1, n, 0, 1, 0, 0, 0) | 11000010 | 11 reg:3=d&7 rm:3=m&7 | 00001000 ; if(((8&3)==0)|((8&3)==3)) evexr(d, n, 1, 1, m, 0, 1, 0, 0, 0) | 11000010 | 11 reg:3=d&7 rm:3=n&7 | 00001000 }" "asm"="vcmppd $8, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f,avx512vl" "nodecode" {
  %r = fcmp ueq <2 x double> %n, %m
  ret <2 x i1> %r
}

define <2 x i1> @VCMPULTPDk.2xf64(<2 x double> "reg"="xv" %n, <2 x double> "reg"="xv" %m) "enc"="{ evexr(d, m, 1, 1, n, 0, 1, 0, 0, 0) | 11000010 | 11 reg:3=d&7 rm:3=m&7 | 00001001 ; if(((9&3)==0)|((9&3)==3)) evexr(d, n, 1, 1, m, 0, 1, 0, 0, 0) | 11000010 | 11 reg:3=d&7 rm:3=n&7 | 00001001 }" "asm"="vcmppd $9, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f,avx512vl" "nodecode" {
  %r = fcmp ult <2 x double> %n, %m
  ret <2 x i1> %r
}

define <2 x i1> @VCMPULEPDk.2xf64(<2 x double> "reg"="xv" %n, <2 x double> "reg"="xv" %m) "enc"="{ evexr(d, m, 1, 1, n, 0, 1, 0, 0, 0) | 11000010 | 11 reg:3=d&7 rm:3=m&7 | 00001010 ; if(((10&3)==0)|((10&3)==3)) evexr(d, n, 1, 1, m, 0, 1, 0, 0, 0) | 11000010 | 11 reg:3=d&7 rm:3=n&7 | 00001010 }" "asm"="vcmppd $10, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f,avx512vl" "nodecode" {
  %r = fcmp ule <2 x double> %n, %m
  ret <2 x i1> %r
}

define <2 x i1> @VCMPONEPDk.2xf64(<2 x double> "reg"="xv" %n, <2 x double> "reg"="xv" %m) "enc"="{ evexr(d, m, 1, 1, n, 0, 1, 0, 0, 0) | 11000010 | 11 reg:3=d&7 rm:3=m&7 | 00001100 ; if(((12&3)==0)|((12&3)==3)) evexr(d, n, 1, 1, m, 0, 1, 0, 0, 0) | 11000010 | 11 reg:3=d&7 rm:3=n&7 | 00001100 }" "asm"="vcmppd $12, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f,avx512vl" "nodecode" {
  %r = fcmp one <2 x double> %n, %m
  ret <2 x i1> %r
}

define <2 x i1> @VCMPOGEPDk.2xf64(<2 x double> "reg"="xv" %n, <2 x double> "reg"="xv" %m) "enc"="{ evexr(d, m, 1, 1, n, 0, 1, 0, 0, 0) | 11000010 | 11 reg:3=d&7 rm:3=m&7 | 00001101 ; if(((13&3)==0)|((13&3)==3)) evexr(d, n, 1, 1, m, 0, 1, 0, 0, 0) | 11000010 | 11 reg:3=d&7 rm:3=n&7 | 00001101 }" "asm"="vcmppd $13, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f,avx512vl" "nodecode" {
  %r = fcmp oge <2 x double> %n, %m
  ret <2 x i1> %r
}

define <2 x i1> @VCMPOGTPDk.2xf64(<2 x double> "reg"="xv" %n, <2 x double> "reg"="xv" %m) "enc"="{ evexr(d, m, 1, 1, n, 0, 1, 0, 0, 0) | 11000010 | 11 reg:3=d&7 rm:3=m&7 | 00001110 ; if(((14&3)==0)|((14&3)==3)) evexr(d, n, 1, 1, m, 0, 1, 0, 0, 0) | 11000010 | 11 reg:3=d&7 rm:3=n&7 | 00001110 }" "asm"="vcmppd $14, {m}, {n}, {d}" "dclass"="kw" "feature"="avx512f,avx512vl" "nodecode" {
  %r = fcmp ogt <2 x double> %n, %m
  ret <2 x i1> %r
}

define <2 x double> @VBLENDMPD.2xf64(<2 x i1> "reg"="kw" %k, <2 x double> "reg"="xv" %a, <2 x double> "reg"="xv" %b) "enc"="{ if(k!=0) evexr(d, a, 2, 1, b, 0, 1, 0, k, 0) | 01100101 | 11 reg:3=d&7 rm:3=a&7 }" "asm"="vblendmpd {a}, {b}, {d} {lb}{k}{rb}" "dclass"="xv" "feature"="avx512f,avx512vl" "nodecode" {
  %r = select <2 x i1> %k, <2 x double> %a, <2 x double> %b
  ret <2 x double> %r
}

define <64 x i8> @VPTERNLOGD.64xi8(i8 "imm" %t, <64 x i8> "reg"="zv" "tied" %x, <64 x i8> "reg"="zv" %y, <64 x i8> "reg"="zv" %z) "enc"="evexr(d, z, 3, 0, y, 2, 1, 0, 0, 0) | 00100101 | 11 reg:3=d&7 rm:3=z&7 | { i1:8=t ; i1:s8=t }" "asm"="vpternlogd ${t}, {z}, {y}, {d}" "dclass"="zv" "feature"="avx512f" "nodecode" {
  %r = call <64 x i8> @llvm.epitome.bitfn3.v64i8(i8 %t, <64 x i8> %x, <64 x i8> %y, <64 x i8> %z)
  ret <64 x i8> %r
}

declare <64 x i8> @llvm.epitome.bitfn3.v64i8(i8, <64 x i8>, <64 x i8>, <64 x i8>)

define <32 x i16> @VPTERNLOGD.32xi16(i8 "imm" %t, <32 x i16> "reg"="zv" "tied" %x, <32 x i16> "reg"="zv" %y, <32 x i16> "reg"="zv" %z) "enc"="evexr(d, z, 3, 0, y, 2, 1, 0, 0, 0) | 00100101 | 11 reg:3=d&7 rm:3=z&7 | { i1:8=t ; i1:s8=t }" "asm"="vpternlogd ${t}, {z}, {y}, {d}" "dclass"="zv" "feature"="avx512f" "nodecode" {
  %r = call <32 x i16> @llvm.epitome.bitfn3.v32i16(i8 %t, <32 x i16> %x, <32 x i16> %y, <32 x i16> %z)
  ret <32 x i16> %r
}

declare <32 x i16> @llvm.epitome.bitfn3.v32i16(i8, <32 x i16>, <32 x i16>, <32 x i16>)

define <16 x i32> @VPTERNLOGD.16xi32(i8 "imm" %t, <16 x i32> "reg"="zv" "tied" %x, <16 x i32> "reg"="zv" %y, <16 x i32> "reg"="zv" %z) "enc"="evexr(d, z, 3, 0, y, 2, 1, 0, 0, 0) | 00100101 | 11 reg:3=d&7 rm:3=z&7 | { i1:8=t ; i1:s8=t }" "asm"="vpternlogd ${t}, {z}, {y}, {d}" "dclass"="zv" "feature"="avx512f" "nodecode" {
  %r = call <16 x i32> @llvm.epitome.bitfn3.v16i32(i8 %t, <16 x i32> %x, <16 x i32> %y, <16 x i32> %z)
  ret <16 x i32> %r
}

declare <16 x i32> @llvm.epitome.bitfn3.v16i32(i8, <16 x i32>, <16 x i32>, <16 x i32>)

define <8 x i64> @VPTERNLOGQ.8xi64(i8 "imm" %t, <8 x i64> "reg"="zv" "tied" %x, <8 x i64> "reg"="zv" %y, <8 x i64> "reg"="zv" %z) "enc"="evexr(d, z, 3, 1, y, 2, 1, 0, 0, 0) | 00100101 | 11 reg:3=d&7 rm:3=z&7 | { i1:8=t ; i1:s8=t }" "asm"="vpternlogq ${t}, {z}, {y}, {d}" "dclass"="zv" "feature"="avx512f" "nodecode" {
  %r = call <8 x i64> @llvm.epitome.bitfn3.v8i64(i8 %t, <8 x i64> %x, <8 x i64> %y, <8 x i64> %z)
  ret <8 x i64> %r
}

declare <8 x i64> @llvm.epitome.bitfn3.v8i64(i8, <8 x i64>, <8 x i64>, <8 x i64>)

define <32 x i8> @VPTERNLOGD.32xi8(i8 "imm" %t, <32 x i8> "reg"="yv" "tied" %x, <32 x i8> "reg"="yv" %y, <32 x i8> "reg"="yv" %z) "enc"="evexr(d, z, 3, 0, y, 1, 1, 0, 0, 0) | 00100101 | 11 reg:3=d&7 rm:3=z&7 | { i1:8=t ; i1:s8=t }" "asm"="vpternlogd ${t}, {z}, {y}, {d}" "dclass"="yv" "feature"="avx512f,avx512vl" "nodecode" {
  %r = call <32 x i8> @llvm.epitome.bitfn3.v32i8(i8 %t, <32 x i8> %x, <32 x i8> %y, <32 x i8> %z)
  ret <32 x i8> %r
}

declare <32 x i8> @llvm.epitome.bitfn3.v32i8(i8, <32 x i8>, <32 x i8>, <32 x i8>)

define <16 x i16> @VPTERNLOGD.16xi16(i8 "imm" %t, <16 x i16> "reg"="yv" "tied" %x, <16 x i16> "reg"="yv" %y, <16 x i16> "reg"="yv" %z) "enc"="evexr(d, z, 3, 0, y, 1, 1, 0, 0, 0) | 00100101 | 11 reg:3=d&7 rm:3=z&7 | { i1:8=t ; i1:s8=t }" "asm"="vpternlogd ${t}, {z}, {y}, {d}" "dclass"="yv" "feature"="avx512f,avx512vl" "nodecode" {
  %r = call <16 x i16> @llvm.epitome.bitfn3.v16i16(i8 %t, <16 x i16> %x, <16 x i16> %y, <16 x i16> %z)
  ret <16 x i16> %r
}

declare <16 x i16> @llvm.epitome.bitfn3.v16i16(i8, <16 x i16>, <16 x i16>, <16 x i16>)

define <8 x i32> @VPTERNLOGD.8xi32(i8 "imm" %t, <8 x i32> "reg"="yv" "tied" %x, <8 x i32> "reg"="yv" %y, <8 x i32> "reg"="yv" %z) "enc"="evexr(d, z, 3, 0, y, 1, 1, 0, 0, 0) | 00100101 | 11 reg:3=d&7 rm:3=z&7 | { i1:8=t ; i1:s8=t }" "asm"="vpternlogd ${t}, {z}, {y}, {d}" "dclass"="yv" "feature"="avx512f,avx512vl" "nodecode" {
  %r = call <8 x i32> @llvm.epitome.bitfn3.v8i32(i8 %t, <8 x i32> %x, <8 x i32> %y, <8 x i32> %z)
  ret <8 x i32> %r
}

declare <8 x i32> @llvm.epitome.bitfn3.v8i32(i8, <8 x i32>, <8 x i32>, <8 x i32>)

define <4 x i64> @VPTERNLOGQ.4xi64(i8 "imm" %t, <4 x i64> "reg"="yv" "tied" %x, <4 x i64> "reg"="yv" %y, <4 x i64> "reg"="yv" %z) "enc"="evexr(d, z, 3, 1, y, 1, 1, 0, 0, 0) | 00100101 | 11 reg:3=d&7 rm:3=z&7 | { i1:8=t ; i1:s8=t }" "asm"="vpternlogq ${t}, {z}, {y}, {d}" "dclass"="yv" "feature"="avx512f,avx512vl" "nodecode" {
  %r = call <4 x i64> @llvm.epitome.bitfn3.v4i64(i8 %t, <4 x i64> %x, <4 x i64> %y, <4 x i64> %z)
  ret <4 x i64> %r
}

declare <4 x i64> @llvm.epitome.bitfn3.v4i64(i8, <4 x i64>, <4 x i64>, <4 x i64>)

define <16 x i8> @VPTERNLOGD.16xi8(i8 "imm" %t, <16 x i8> "reg"="xv" "tied" %x, <16 x i8> "reg"="xv" %y, <16 x i8> "reg"="xv" %z) "enc"="evexr(d, z, 3, 0, y, 0, 1, 0, 0, 0) | 00100101 | 11 reg:3=d&7 rm:3=z&7 | { i1:8=t ; i1:s8=t }" "asm"="vpternlogd ${t}, {z}, {y}, {d}" "dclass"="xv" "feature"="avx512f,avx512vl" "nodecode" {
  %r = call <16 x i8> @llvm.epitome.bitfn3.v16i8(i8 %t, <16 x i8> %x, <16 x i8> %y, <16 x i8> %z)
  ret <16 x i8> %r
}

declare <16 x i8> @llvm.epitome.bitfn3.v16i8(i8, <16 x i8>, <16 x i8>, <16 x i8>)

define <8 x i16> @VPTERNLOGD.8xi16(i8 "imm" %t, <8 x i16> "reg"="xv" "tied" %x, <8 x i16> "reg"="xv" %y, <8 x i16> "reg"="xv" %z) "enc"="evexr(d, z, 3, 0, y, 0, 1, 0, 0, 0) | 00100101 | 11 reg:3=d&7 rm:3=z&7 | { i1:8=t ; i1:s8=t }" "asm"="vpternlogd ${t}, {z}, {y}, {d}" "dclass"="xv" "feature"="avx512f,avx512vl" "nodecode" {
  %r = call <8 x i16> @llvm.epitome.bitfn3.v8i16(i8 %t, <8 x i16> %x, <8 x i16> %y, <8 x i16> %z)
  ret <8 x i16> %r
}

declare <8 x i16> @llvm.epitome.bitfn3.v8i16(i8, <8 x i16>, <8 x i16>, <8 x i16>)

define <4 x i32> @VPTERNLOGD.4xi32(i8 "imm" %t, <4 x i32> "reg"="xv" "tied" %x, <4 x i32> "reg"="xv" %y, <4 x i32> "reg"="xv" %z) "enc"="evexr(d, z, 3, 0, y, 0, 1, 0, 0, 0) | 00100101 | 11 reg:3=d&7 rm:3=z&7 | { i1:8=t ; i1:s8=t }" "asm"="vpternlogd ${t}, {z}, {y}, {d}" "dclass"="xv" "feature"="avx512f,avx512vl" "nodecode" {
  %r = call <4 x i32> @llvm.epitome.bitfn3.v4i32(i8 %t, <4 x i32> %x, <4 x i32> %y, <4 x i32> %z)
  ret <4 x i32> %r
}

declare <4 x i32> @llvm.epitome.bitfn3.v4i32(i8, <4 x i32>, <4 x i32>, <4 x i32>)

define <2 x i64> @VPTERNLOGQ.2xi64(i8 "imm" %t, <2 x i64> "reg"="xv" "tied" %x, <2 x i64> "reg"="xv" %y, <2 x i64> "reg"="xv" %z) "enc"="evexr(d, z, 3, 1, y, 0, 1, 0, 0, 0) | 00100101 | 11 reg:3=d&7 rm:3=z&7 | { i1:8=t ; i1:s8=t }" "asm"="vpternlogq ${t}, {z}, {y}, {d}" "dclass"="xv" "feature"="avx512f,avx512vl" "nodecode" {
  %r = call <2 x i64> @llvm.epitome.bitfn3.v2i64(i8 %t, <2 x i64> %x, <2 x i64> %y, <2 x i64> %z)
  ret <2 x i64> %r
}

declare <2 x i64> @llvm.epitome.bitfn3.v2i64(i8, <2 x i64>, <2 x i64>, <2 x i64>)

declare i32 @llvm.vector.reduce.add.v4i32(<4 x i32>)

define i32 @reduce.add.4xi32(<4 x i32> "reg"="xv" %v) "recipe"="reduce.add.4xi32.expand" {
  %r = call i32 @llvm.vector.reduce.add.v4i32(<4 x i32> %v)
  ret i32 %r
}

define i32 @reduce.add.4xi32.expand(<4 x i32> %v) {
  %s1 = call <4 x i32> @PSHUFD(<4 x i32> %v, i64 14)
  %a1 = call <4 x i32> @PADDD.4xi32(<4 x i32> %v, <4 x i32> %s1)
  %s2 = call <4 x i32> @PSHUFD(<4 x i32> %a1, i64 1)
  %a2 = call <4 x i32> @PADDD.4xi32(<4 x i32> %a1, <4 x i32> %s2)
  %r = call i32 @MOVDlane0.4xi32(<4 x i32> %a2)
  ret i32 %r
}

declare i64 @llvm.vector.reduce.add.v2i64(<2 x i64>)

define i64 @reduce.add.2xi64(<2 x i64> "reg"="xv" %v) "recipe"="reduce.add.2xi64.expand" {
  %r = call i64 @llvm.vector.reduce.add.v2i64(<2 x i64> %v)
  ret i64 %r
}

define i64 @reduce.add.2xi64.expand(<2 x i64> %v) {
  %s1 = call <2 x i64> @PUNPCKHQDQself(<2 x i64> %v)
  %a1 = call <2 x i64> @PADDQ.2xi64(<2 x i64> %v, <2 x i64> %s1)
  %r = call i64 @MOVQlane0(<2 x i64> %a1)
  ret i64 %r
}

declare i32 @llvm.vector.reduce.add.v8i32(<8 x i32>)

define i32 @reduce.add.8xi32(<8 x i32> "reg"="yv" %v) "recipe"="reduce.add.8xi32.expand" "feature"="avx2" {
  %r = call i32 @llvm.vector.reduce.add.v8i32(<8 x i32> %v)
  ret i32 %r
}

define i32 @reduce.add.8xi32.expand(<8 x i32> %v) {
  %l = call <4 x i32> @VLOW.8xi32(<8 x i32> %v)
  %h = call <4 x i32> @VHIGH.8xi32(<8 x i32> %v)
  %s = call <4 x i32> @PADDD.4xi32(<4 x i32> %l, <4 x i32> %h)
  %s1 = call <4 x i32> @PSHUFD(<4 x i32> %s, i64 14)
  %a1 = call <4 x i32> @PADDD.4xi32(<4 x i32> %s, <4 x i32> %s1)
  %s2 = call <4 x i32> @PSHUFD(<4 x i32> %a1, i64 1)
  %a2 = call <4 x i32> @PADDD.4xi32(<4 x i32> %a1, <4 x i32> %s2)
  %r = call i32 @MOVDlane0.4xi32(<4 x i32> %a2)
  ret i32 %r
}

declare i64 @llvm.vector.reduce.add.v4i64(<4 x i64>)

define i64 @reduce.add.4xi64(<4 x i64> "reg"="yv" %v) "recipe"="reduce.add.4xi64.expand" "feature"="avx2" {
  %r = call i64 @llvm.vector.reduce.add.v4i64(<4 x i64> %v)
  ret i64 %r
}

define i64 @reduce.add.4xi64.expand(<4 x i64> %v) {
  %l = call <2 x i64> @VLOW.4xi64(<4 x i64> %v)
  %h = call <2 x i64> @VHIGH.4xi64(<4 x i64> %v)
  %s = call <2 x i64> @PADDQ.2xi64(<2 x i64> %l, <2 x i64> %h)
  %s1 = call <2 x i64> @PUNPCKHQDQself(<2 x i64> %s)
  %a1 = call <2 x i64> @PADDQ.2xi64(<2 x i64> %s, <2 x i64> %s1)
  %r = call i64 @MOVQlane0(<2 x i64> %a1)
  ret i64 %r
}

define <2 x i64> @mul.2xi64(<2 x i64> "reg"="xv" %a, <2 x i64> "reg"="xv" %b) "recipe"="mul.2xi64.expand" {
  %r = mul <2 x i64> %a, %b
  ret <2 x i64> %r
}

define <2 x i64> @mul.2xi64.expand(<2 x i64> %a, <2 x i64> %b) {
  %h = call <2 x i64> @PSRLQi.2xi64(<2 x i64> %a, i64 32)
  %g = call <2 x i64> @PSRLQi.2xi64(<2 x i64> %b, i64 32)
  %p = call <2 x i64> @PMULUDQ(<2 x i64> %a, <2 x i64> %b)
  %q = call <2 x i64> @PMULUDQ(<2 x i64> %h, <2 x i64> %b)
  %u = call <2 x i64> @PMULUDQ(<2 x i64> %a, <2 x i64> %g)
  %s = call <2 x i64> @PADDQ.2xi64(<2 x i64> %q, <2 x i64> %u)
  %t = call <2 x i64> @PSLLQi.2xi64(<2 x i64> %s, i64 32)
  %r = call <2 x i64> @PADDQ.2xi64(<2 x i64> %p, <2 x i64> %t)
  ret <2 x i64> %r
}

define <4 x i64> @VPMULUDQ(<4 x i64> "reg"="yv" %n, <4 x i64> "reg"="yv" %m) "enc"="{ if(m<8) vex(d, 0, m, 1, 0, n, 1, 1) | 11110100 | 11 reg:3=d&7 rm:3=m&7 ; if(n<8) vex(d, 0, n, 1, 0, m, 1, 1) | 11110100 | 11 reg:3=d&7 rm:3=n&7 ; vex(d, 0, m, 1, 0, n, 1, 1) | 11110100 | 11 reg:3=d&7 rm:3=m&7 ; evexr(d, m, 1, 1, n, 1, 1, 0, 0, 0) | 11110100 | 11 reg:3=d&7 rm:3=m&7 ; vex(d, 0, n, 1, 0, m, 1, 1) | 11110100 | 11 reg:3=d&7 rm:3=n&7 }" "asm"="vpmuludq {m}, {n}, {d}" "dclass"="yv" "nomatch" "feature"="avx2" "nodecode" {
  ret <4 x i64> %n
}

define <4 x i64> @mul.4xi64(<4 x i64> "reg"="yv" %a, <4 x i64> "reg"="yv" %b) "recipe"="mul.4xi64.expand" "feature"="avx2" {
  %r = mul <4 x i64> %a, %b
  ret <4 x i64> %r
}

define <4 x i64> @mul.4xi64.expand(<4 x i64> %a, <4 x i64> %b) {
  %h = call <4 x i64> @VPSRLQi.4xi64(<4 x i64> %a, i64 32)
  %g = call <4 x i64> @VPSRLQi.4xi64(<4 x i64> %b, i64 32)
  %p = call <4 x i64> @VPMULUDQ(<4 x i64> %a, <4 x i64> %b)
  %q = call <4 x i64> @VPMULUDQ(<4 x i64> %h, <4 x i64> %b)
  %u = call <4 x i64> @VPMULUDQ(<4 x i64> %a, <4 x i64> %g)
  %s = call <4 x i64> @VPADDQ.4xi64(<4 x i64> %q, <4 x i64> %u)
  %t = call <4 x i64> @VPSLLQi.4xi64(<4 x i64> %s, i64 32)
  %r = call <4 x i64> @VPADDQ.4xi64(<4 x i64> %p, <4 x i64> %t)
  ret <4 x i64> %r
}

define <4 x i64> @VALLONES() "enc"="{ if(d<8) vex(d, 0, d, 1, 0, d, 1, 1) | 01110110 | 11 reg:3=d&7 rm:3=d&7 ; if(d<8) vex(d, 0, d, 1, 0, d, 1, 1) | 01110110 | 11 reg:3=d&7 rm:3=d&7 ; vex(d, 0, d, 1, 0, d, 1, 1) | 01110110 | 11 reg:3=d&7 rm:3=d&7 ; vex(d, 0, d, 1, 0, d, 1, 1) | 01110110 | 11 reg:3=d&7 rm:3=d&7 }" "asm"="vpcmpeqd {d}, {d}, {d}" "dclass"="yv" "nomatch" "feature"="avx2" "nodecode" {
  ret <4 x i64> splat (i64 -1)
}

define <32 x i1> @vslt.32xi8(<32 x i8> "reg"="yv" %a, <32 x i8> "reg"="yv" %b) "recipe"="vslt.32xi8.expand" "feature"="avx2" {
  %r = icmp slt <32 x i8> %a, %b
  ret <32 x i1> %r
}

define <32 x i1> @vslt.32xi8.expand(<32 x i8> %a, <32 x i8> %b) {
  %r = call <32 x i1> @VPCMPGTB.32xi8(<32 x i8> %b, <32 x i8> %a)
  ret <32 x i1> %r
}

define <32 x i1> @vne.32xi8(<32 x i8> "reg"="yv" %a, <32 x i8> "reg"="yv" %b) "recipe"="vne.32xi8.expand" "feature"="avx2" {
  %r = icmp ne <32 x i8> %a, %b
  ret <32 x i1> %r
}

define <32 x i1> @vne.32xi8.expand(<32 x i8> %a, <32 x i8> %b) {
  %c = call <32 x i1> @VPCMPEQB.32xi8(<32 x i8> %a, <32 x i8> %b)
  %xi = sext <32 x i1> %c to <32 x i8>
  %on = call <4 x i64> @VALLONES()
  %ot = bitcast <4 x i64> %on to <32 x i8>
  %ni = call <32 x i8> @VPXOR.32xi8(<32 x i8> %xi, <32 x i8> %ot)
  %r = trunc <32 x i8> %ni to <32 x i1>
  ret <32 x i1> %r
}

define <32 x i1> @vsge.32xi8(<32 x i8> "reg"="yv" %a, <32 x i8> "reg"="yv" %b) "recipe"="vsge.32xi8.expand" "feature"="avx2" {
  %r = icmp sge <32 x i8> %a, %b
  ret <32 x i1> %r
}

define <32 x i1> @vsge.32xi8.expand(<32 x i8> %a, <32 x i8> %b) {
  %c = call <32 x i1> @VPCMPGTB.32xi8(<32 x i8> %b, <32 x i8> %a)
  %xi = sext <32 x i1> %c to <32 x i8>
  %on = call <4 x i64> @VALLONES()
  %ot = bitcast <4 x i64> %on to <32 x i8>
  %ni = call <32 x i8> @VPXOR.32xi8(<32 x i8> %xi, <32 x i8> %ot)
  %r = trunc <32 x i8> %ni to <32 x i1>
  ret <32 x i1> %r
}

define <32 x i1> @vsle.32xi8(<32 x i8> "reg"="yv" %a, <32 x i8> "reg"="yv" %b) "recipe"="vsle.32xi8.expand" "feature"="avx2" {
  %r = icmp sle <32 x i8> %a, %b
  ret <32 x i1> %r
}

define <32 x i1> @vsle.32xi8.expand(<32 x i8> %a, <32 x i8> %b) {
  %c = call <32 x i1> @VPCMPGTB.32xi8(<32 x i8> %a, <32 x i8> %b)
  %xi = sext <32 x i1> %c to <32 x i8>
  %on = call <4 x i64> @VALLONES()
  %ot = bitcast <4 x i64> %on to <32 x i8>
  %ni = call <32 x i8> @VPXOR.32xi8(<32 x i8> %xi, <32 x i8> %ot)
  %r = trunc <32 x i8> %ni to <32 x i1>
  ret <32 x i1> %r
}

define <32 x i1> @vuge.32xi8(<32 x i8> "reg"="yv" %a, <32 x i8> "reg"="yv" %b) "recipe"="vuge.32xi8.expand" "feature"="avx2" {
  %r = icmp uge <32 x i8> %a, %b
  ret <32 x i1> %r
}

define <32 x i1> @vuge.32xi8.expand(<32 x i8> %a, <32 x i8> %b) {
  %m = call <32 x i8> @VPMAXUB.32xi8(<32 x i8> %a, <32 x i8> %b)
  %c = call <32 x i1> @VPCMPEQB.32xi8(<32 x i8> %m, <32 x i8> %a)
  ret <32 x i1> %c
}

define <32 x i1> @vule.32xi8(<32 x i8> "reg"="yv" %a, <32 x i8> "reg"="yv" %b) "recipe"="vule.32xi8.expand" "feature"="avx2" {
  %r = icmp ule <32 x i8> %a, %b
  ret <32 x i1> %r
}

define <32 x i1> @vule.32xi8.expand(<32 x i8> %a, <32 x i8> %b) {
  %m = call <32 x i8> @VPMAXUB.32xi8(<32 x i8> %b, <32 x i8> %a)
  %c = call <32 x i1> @VPCMPEQB.32xi8(<32 x i8> %m, <32 x i8> %b)
  ret <32 x i1> %c
}

define <32 x i1> @vult.32xi8(<32 x i8> "reg"="yv" %a, <32 x i8> "reg"="yv" %b) "recipe"="vult.32xi8.expand" "feature"="avx2" {
  %r = icmp ult <32 x i8> %a, %b
  ret <32 x i1> %r
}

define <32 x i1> @vult.32xi8.expand(<32 x i8> %a, <32 x i8> %b) {
  %m = call <32 x i8> @VPMAXUB.32xi8(<32 x i8> %a, <32 x i8> %b)
  %c = call <32 x i1> @VPCMPEQB.32xi8(<32 x i8> %m, <32 x i8> %a)
  %xi = sext <32 x i1> %c to <32 x i8>
  %on = call <4 x i64> @VALLONES()
  %ot = bitcast <4 x i64> %on to <32 x i8>
  %ni = call <32 x i8> @VPXOR.32xi8(<32 x i8> %xi, <32 x i8> %ot)
  %r = trunc <32 x i8> %ni to <32 x i1>
  ret <32 x i1> %r
}

define <32 x i1> @vugt.32xi8(<32 x i8> "reg"="yv" %a, <32 x i8> "reg"="yv" %b) "recipe"="vugt.32xi8.expand" "feature"="avx2" {
  %r = icmp ugt <32 x i8> %a, %b
  ret <32 x i1> %r
}

define <32 x i1> @vugt.32xi8.expand(<32 x i8> %a, <32 x i8> %b) {
  %m = call <32 x i8> @VPMAXUB.32xi8(<32 x i8> %b, <32 x i8> %a)
  %c = call <32 x i1> @VPCMPEQB.32xi8(<32 x i8> %m, <32 x i8> %b)
  %xi = sext <32 x i1> %c to <32 x i8>
  %on = call <4 x i64> @VALLONES()
  %ot = bitcast <4 x i64> %on to <32 x i8>
  %ni = call <32 x i8> @VPXOR.32xi8(<32 x i8> %xi, <32 x i8> %ot)
  %r = trunc <32 x i8> %ni to <32 x i1>
  ret <32 x i1> %r
}

define <32 x i8> @VPABSB.false.32xi8(<32 x i8> "reg"="yv" %n) "enc"="{ vex(d, 0, n, 2, 0, 0, 1, 1) | 00011100 | 11 reg:3=d&7 rm:3=n&7 ; evexr(d, n, 2, 0, 0, 1, 1, 0, 0, 0) | 00011100 | 11 reg:3=d&7 rm:3=n&7 }" "asm"="vpabsb {n}, {d}" "dclass"="yv" "feature"="avx2" "nodecode" {
  %r = call <32 x i8> @llvm.abs.v32i8(<32 x i8> %n, i1 false)
  ret <32 x i8> %r
}

define <32 x i8> @VPABSB.true.32xi8(<32 x i8> "reg"="yv" %n) "enc"="{ vex(d, 0, n, 2, 0, 0, 1, 1) | 00011100 | 11 reg:3=d&7 rm:3=n&7 ; evexr(d, n, 2, 0, 0, 1, 1, 0, 0, 0) | 00011100 | 11 reg:3=d&7 rm:3=n&7 }" "asm"="vpabsb {n}, {d}" "dclass"="yv" "feature"="avx2" "nodecode" {
  %r = call <32 x i8> @llvm.abs.v32i8(<32 x i8> %n, i1 true)
  ret <32 x i8> %r
}

declare <32 x i8> @llvm.abs.v32i8(<32 x i8>, i1)

define <16 x i1> @vslt.16xi16(<16 x i16> "reg"="yv" %a, <16 x i16> "reg"="yv" %b) "recipe"="vslt.16xi16.expand" "feature"="avx2" {
  %r = icmp slt <16 x i16> %a, %b
  ret <16 x i1> %r
}

define <16 x i1> @vslt.16xi16.expand(<16 x i16> %a, <16 x i16> %b) {
  %r = call <16 x i1> @VPCMPGTW.16xi16(<16 x i16> %b, <16 x i16> %a)
  ret <16 x i1> %r
}

define <16 x i1> @vne.16xi16(<16 x i16> "reg"="yv" %a, <16 x i16> "reg"="yv" %b) "recipe"="vne.16xi16.expand" "feature"="avx2" {
  %r = icmp ne <16 x i16> %a, %b
  ret <16 x i1> %r
}

define <16 x i1> @vne.16xi16.expand(<16 x i16> %a, <16 x i16> %b) {
  %c = call <16 x i1> @VPCMPEQW.16xi16(<16 x i16> %a, <16 x i16> %b)
  %xi = sext <16 x i1> %c to <16 x i16>
  %on = call <4 x i64> @VALLONES()
  %ot = bitcast <4 x i64> %on to <16 x i16>
  %ni = call <16 x i16> @VPXOR.16xi16(<16 x i16> %xi, <16 x i16> %ot)
  %r = trunc <16 x i16> %ni to <16 x i1>
  ret <16 x i1> %r
}

define <16 x i1> @vsge.16xi16(<16 x i16> "reg"="yv" %a, <16 x i16> "reg"="yv" %b) "recipe"="vsge.16xi16.expand" "feature"="avx2" {
  %r = icmp sge <16 x i16> %a, %b
  ret <16 x i1> %r
}

define <16 x i1> @vsge.16xi16.expand(<16 x i16> %a, <16 x i16> %b) {
  %c = call <16 x i1> @VPCMPGTW.16xi16(<16 x i16> %b, <16 x i16> %a)
  %xi = sext <16 x i1> %c to <16 x i16>
  %on = call <4 x i64> @VALLONES()
  %ot = bitcast <4 x i64> %on to <16 x i16>
  %ni = call <16 x i16> @VPXOR.16xi16(<16 x i16> %xi, <16 x i16> %ot)
  %r = trunc <16 x i16> %ni to <16 x i1>
  ret <16 x i1> %r
}

define <16 x i1> @vsle.16xi16(<16 x i16> "reg"="yv" %a, <16 x i16> "reg"="yv" %b) "recipe"="vsle.16xi16.expand" "feature"="avx2" {
  %r = icmp sle <16 x i16> %a, %b
  ret <16 x i1> %r
}

define <16 x i1> @vsle.16xi16.expand(<16 x i16> %a, <16 x i16> %b) {
  %c = call <16 x i1> @VPCMPGTW.16xi16(<16 x i16> %a, <16 x i16> %b)
  %xi = sext <16 x i1> %c to <16 x i16>
  %on = call <4 x i64> @VALLONES()
  %ot = bitcast <4 x i64> %on to <16 x i16>
  %ni = call <16 x i16> @VPXOR.16xi16(<16 x i16> %xi, <16 x i16> %ot)
  %r = trunc <16 x i16> %ni to <16 x i1>
  ret <16 x i1> %r
}

define <16 x i1> @vuge.16xi16(<16 x i16> "reg"="yv" %a, <16 x i16> "reg"="yv" %b) "recipe"="vuge.16xi16.expand" "feature"="avx2" {
  %r = icmp uge <16 x i16> %a, %b
  ret <16 x i1> %r
}

define <16 x i1> @vuge.16xi16.expand(<16 x i16> %a, <16 x i16> %b) {
  %m = call <16 x i16> @VPMAXUW.16xi16(<16 x i16> %a, <16 x i16> %b)
  %c = call <16 x i1> @VPCMPEQW.16xi16(<16 x i16> %m, <16 x i16> %a)
  ret <16 x i1> %c
}

define <16 x i1> @vule.16xi16(<16 x i16> "reg"="yv" %a, <16 x i16> "reg"="yv" %b) "recipe"="vule.16xi16.expand" "feature"="avx2" {
  %r = icmp ule <16 x i16> %a, %b
  ret <16 x i1> %r
}

define <16 x i1> @vule.16xi16.expand(<16 x i16> %a, <16 x i16> %b) {
  %m = call <16 x i16> @VPMAXUW.16xi16(<16 x i16> %b, <16 x i16> %a)
  %c = call <16 x i1> @VPCMPEQW.16xi16(<16 x i16> %m, <16 x i16> %b)
  ret <16 x i1> %c
}

define <16 x i1> @vult.16xi16(<16 x i16> "reg"="yv" %a, <16 x i16> "reg"="yv" %b) "recipe"="vult.16xi16.expand" "feature"="avx2" {
  %r = icmp ult <16 x i16> %a, %b
  ret <16 x i1> %r
}

define <16 x i1> @vult.16xi16.expand(<16 x i16> %a, <16 x i16> %b) {
  %m = call <16 x i16> @VPMAXUW.16xi16(<16 x i16> %a, <16 x i16> %b)
  %c = call <16 x i1> @VPCMPEQW.16xi16(<16 x i16> %m, <16 x i16> %a)
  %xi = sext <16 x i1> %c to <16 x i16>
  %on = call <4 x i64> @VALLONES()
  %ot = bitcast <4 x i64> %on to <16 x i16>
  %ni = call <16 x i16> @VPXOR.16xi16(<16 x i16> %xi, <16 x i16> %ot)
  %r = trunc <16 x i16> %ni to <16 x i1>
  ret <16 x i1> %r
}

define <16 x i1> @vugt.16xi16(<16 x i16> "reg"="yv" %a, <16 x i16> "reg"="yv" %b) "recipe"="vugt.16xi16.expand" "feature"="avx2" {
  %r = icmp ugt <16 x i16> %a, %b
  ret <16 x i1> %r
}

define <16 x i1> @vugt.16xi16.expand(<16 x i16> %a, <16 x i16> %b) {
  %m = call <16 x i16> @VPMAXUW.16xi16(<16 x i16> %b, <16 x i16> %a)
  %c = call <16 x i1> @VPCMPEQW.16xi16(<16 x i16> %m, <16 x i16> %b)
  %xi = sext <16 x i1> %c to <16 x i16>
  %on = call <4 x i64> @VALLONES()
  %ot = bitcast <4 x i64> %on to <16 x i16>
  %ni = call <16 x i16> @VPXOR.16xi16(<16 x i16> %xi, <16 x i16> %ot)
  %r = trunc <16 x i16> %ni to <16 x i1>
  ret <16 x i1> %r
}

define <16 x i16> @VPABSW.false.16xi16(<16 x i16> "reg"="yv" %n) "enc"="{ vex(d, 0, n, 2, 0, 0, 1, 1) | 00011101 | 11 reg:3=d&7 rm:3=n&7 ; evexr(d, n, 2, 0, 0, 1, 1, 0, 0, 0) | 00011101 | 11 reg:3=d&7 rm:3=n&7 }" "asm"="vpabsw {n}, {d}" "dclass"="yv" "feature"="avx2" "nodecode" {
  %r = call <16 x i16> @llvm.abs.v16i16(<16 x i16> %n, i1 false)
  ret <16 x i16> %r
}

define <16 x i16> @VPABSW.true.16xi16(<16 x i16> "reg"="yv" %n) "enc"="{ vex(d, 0, n, 2, 0, 0, 1, 1) | 00011101 | 11 reg:3=d&7 rm:3=n&7 ; evexr(d, n, 2, 0, 0, 1, 1, 0, 0, 0) | 00011101 | 11 reg:3=d&7 rm:3=n&7 }" "asm"="vpabsw {n}, {d}" "dclass"="yv" "feature"="avx2" "nodecode" {
  %r = call <16 x i16> @llvm.abs.v16i16(<16 x i16> %n, i1 true)
  ret <16 x i16> %r
}

declare <16 x i16> @llvm.abs.v16i16(<16 x i16>, i1)

define <8 x i1> @vslt.8xi32(<8 x i32> "reg"="yv" %a, <8 x i32> "reg"="yv" %b) "recipe"="vslt.8xi32.expand" "feature"="avx2" {
  %r = icmp slt <8 x i32> %a, %b
  ret <8 x i1> %r
}

define <8 x i1> @vslt.8xi32.expand(<8 x i32> %a, <8 x i32> %b) {
  %r = call <8 x i1> @VPCMPGTD.8xi32(<8 x i32> %b, <8 x i32> %a)
  ret <8 x i1> %r
}

define <8 x i1> @vne.8xi32(<8 x i32> "reg"="yv" %a, <8 x i32> "reg"="yv" %b) "recipe"="vne.8xi32.expand" "feature"="avx2" {
  %r = icmp ne <8 x i32> %a, %b
  ret <8 x i1> %r
}

define <8 x i1> @vne.8xi32.expand(<8 x i32> %a, <8 x i32> %b) {
  %c = call <8 x i1> @VPCMPEQD.8xi32(<8 x i32> %a, <8 x i32> %b)
  %xi = sext <8 x i1> %c to <8 x i32>
  %on = call <4 x i64> @VALLONES()
  %ot = bitcast <4 x i64> %on to <8 x i32>
  %ni = call <8 x i32> @VPXOR.8xi32(<8 x i32> %xi, <8 x i32> %ot)
  %r = trunc <8 x i32> %ni to <8 x i1>
  ret <8 x i1> %r
}

define <8 x i1> @vsge.8xi32(<8 x i32> "reg"="yv" %a, <8 x i32> "reg"="yv" %b) "recipe"="vsge.8xi32.expand" "feature"="avx2" {
  %r = icmp sge <8 x i32> %a, %b
  ret <8 x i1> %r
}

define <8 x i1> @vsge.8xi32.expand(<8 x i32> %a, <8 x i32> %b) {
  %c = call <8 x i1> @VPCMPGTD.8xi32(<8 x i32> %b, <8 x i32> %a)
  %xi = sext <8 x i1> %c to <8 x i32>
  %on = call <4 x i64> @VALLONES()
  %ot = bitcast <4 x i64> %on to <8 x i32>
  %ni = call <8 x i32> @VPXOR.8xi32(<8 x i32> %xi, <8 x i32> %ot)
  %r = trunc <8 x i32> %ni to <8 x i1>
  ret <8 x i1> %r
}

define <8 x i1> @vsle.8xi32(<8 x i32> "reg"="yv" %a, <8 x i32> "reg"="yv" %b) "recipe"="vsle.8xi32.expand" "feature"="avx2" {
  %r = icmp sle <8 x i32> %a, %b
  ret <8 x i1> %r
}

define <8 x i1> @vsle.8xi32.expand(<8 x i32> %a, <8 x i32> %b) {
  %c = call <8 x i1> @VPCMPGTD.8xi32(<8 x i32> %a, <8 x i32> %b)
  %xi = sext <8 x i1> %c to <8 x i32>
  %on = call <4 x i64> @VALLONES()
  %ot = bitcast <4 x i64> %on to <8 x i32>
  %ni = call <8 x i32> @VPXOR.8xi32(<8 x i32> %xi, <8 x i32> %ot)
  %r = trunc <8 x i32> %ni to <8 x i1>
  ret <8 x i1> %r
}

define <8 x i1> @vuge.8xi32(<8 x i32> "reg"="yv" %a, <8 x i32> "reg"="yv" %b) "recipe"="vuge.8xi32.expand" "feature"="avx2" {
  %r = icmp uge <8 x i32> %a, %b
  ret <8 x i1> %r
}

define <8 x i1> @vuge.8xi32.expand(<8 x i32> %a, <8 x i32> %b) {
  %m = call <8 x i32> @VPMAXUD.8xi32(<8 x i32> %a, <8 x i32> %b)
  %c = call <8 x i1> @VPCMPEQD.8xi32(<8 x i32> %m, <8 x i32> %a)
  ret <8 x i1> %c
}

define <8 x i1> @vule.8xi32(<8 x i32> "reg"="yv" %a, <8 x i32> "reg"="yv" %b) "recipe"="vule.8xi32.expand" "feature"="avx2" {
  %r = icmp ule <8 x i32> %a, %b
  ret <8 x i1> %r
}

define <8 x i1> @vule.8xi32.expand(<8 x i32> %a, <8 x i32> %b) {
  %m = call <8 x i32> @VPMAXUD.8xi32(<8 x i32> %b, <8 x i32> %a)
  %c = call <8 x i1> @VPCMPEQD.8xi32(<8 x i32> %m, <8 x i32> %b)
  ret <8 x i1> %c
}

define <8 x i1> @vult.8xi32(<8 x i32> "reg"="yv" %a, <8 x i32> "reg"="yv" %b) "recipe"="vult.8xi32.expand" "feature"="avx2" {
  %r = icmp ult <8 x i32> %a, %b
  ret <8 x i1> %r
}

define <8 x i1> @vult.8xi32.expand(<8 x i32> %a, <8 x i32> %b) {
  %m = call <8 x i32> @VPMAXUD.8xi32(<8 x i32> %a, <8 x i32> %b)
  %c = call <8 x i1> @VPCMPEQD.8xi32(<8 x i32> %m, <8 x i32> %a)
  %xi = sext <8 x i1> %c to <8 x i32>
  %on = call <4 x i64> @VALLONES()
  %ot = bitcast <4 x i64> %on to <8 x i32>
  %ni = call <8 x i32> @VPXOR.8xi32(<8 x i32> %xi, <8 x i32> %ot)
  %r = trunc <8 x i32> %ni to <8 x i1>
  ret <8 x i1> %r
}

define <8 x i1> @vugt.8xi32(<8 x i32> "reg"="yv" %a, <8 x i32> "reg"="yv" %b) "recipe"="vugt.8xi32.expand" "feature"="avx2" {
  %r = icmp ugt <8 x i32> %a, %b
  ret <8 x i1> %r
}

define <8 x i1> @vugt.8xi32.expand(<8 x i32> %a, <8 x i32> %b) {
  %m = call <8 x i32> @VPMAXUD.8xi32(<8 x i32> %b, <8 x i32> %a)
  %c = call <8 x i1> @VPCMPEQD.8xi32(<8 x i32> %m, <8 x i32> %b)
  %xi = sext <8 x i1> %c to <8 x i32>
  %on = call <4 x i64> @VALLONES()
  %ot = bitcast <4 x i64> %on to <8 x i32>
  %ni = call <8 x i32> @VPXOR.8xi32(<8 x i32> %xi, <8 x i32> %ot)
  %r = trunc <8 x i32> %ni to <8 x i1>
  ret <8 x i1> %r
}

define <8 x i32> @VPABSD.false.8xi32(<8 x i32> "reg"="yv" %n) "enc"="{ vex(d, 0, n, 2, 0, 0, 1, 1) | 00011110 | 11 reg:3=d&7 rm:3=n&7 ; evexr(d, n, 2, 0, 0, 1, 1, 0, 0, 0) | 00011110 | 11 reg:3=d&7 rm:3=n&7 }" "asm"="vpabsd {n}, {d}" "dclass"="yv" "feature"="avx2" "nodecode" {
  %r = call <8 x i32> @llvm.abs.v8i32(<8 x i32> %n, i1 false)
  ret <8 x i32> %r
}

define <8 x i32> @VPABSD.true.8xi32(<8 x i32> "reg"="yv" %n) "enc"="{ vex(d, 0, n, 2, 0, 0, 1, 1) | 00011110 | 11 reg:3=d&7 rm:3=n&7 ; evexr(d, n, 2, 0, 0, 1, 1, 0, 0, 0) | 00011110 | 11 reg:3=d&7 rm:3=n&7 }" "asm"="vpabsd {n}, {d}" "dclass"="yv" "feature"="avx2" "nodecode" {
  %r = call <8 x i32> @llvm.abs.v8i32(<8 x i32> %n, i1 true)
  ret <8 x i32> %r
}

declare <8 x i32> @llvm.abs.v8i32(<8 x i32>, i1)

define <4 x i1> @vslt.4xi64(<4 x i64> "reg"="yv" %a, <4 x i64> "reg"="yv" %b) "recipe"="vslt.4xi64.expand" "feature"="avx2" {
  %r = icmp slt <4 x i64> %a, %b
  ret <4 x i1> %r
}

define <4 x i1> @vslt.4xi64.expand(<4 x i64> %a, <4 x i64> %b) {
  %r = call <4 x i1> @VPCMPGTQ.4xi64(<4 x i64> %b, <4 x i64> %a)
  ret <4 x i1> %r
}

define <4 x i1> @vne.4xi64(<4 x i64> "reg"="yv" %a, <4 x i64> "reg"="yv" %b) "recipe"="vne.4xi64.expand" "feature"="avx2" {
  %r = icmp ne <4 x i64> %a, %b
  ret <4 x i1> %r
}

define <4 x i1> @vne.4xi64.expand(<4 x i64> %a, <4 x i64> %b) {
  %c = call <4 x i1> @VPCMPEQQ.4xi64(<4 x i64> %a, <4 x i64> %b)
  %xi = sext <4 x i1> %c to <4 x i64>
  %on = call <4 x i64> @VALLONES()
  %ot = bitcast <4 x i64> %on to <4 x i64>
  %ni = call <4 x i64> @VPXOR.4xi64(<4 x i64> %xi, <4 x i64> %ot)
  %r = trunc <4 x i64> %ni to <4 x i1>
  ret <4 x i1> %r
}

define <4 x i1> @vsge.4xi64(<4 x i64> "reg"="yv" %a, <4 x i64> "reg"="yv" %b) "recipe"="vsge.4xi64.expand" "feature"="avx2" {
  %r = icmp sge <4 x i64> %a, %b
  ret <4 x i1> %r
}

define <4 x i1> @vsge.4xi64.expand(<4 x i64> %a, <4 x i64> %b) {
  %c = call <4 x i1> @VPCMPGTQ.4xi64(<4 x i64> %b, <4 x i64> %a)
  %xi = sext <4 x i1> %c to <4 x i64>
  %on = call <4 x i64> @VALLONES()
  %ot = bitcast <4 x i64> %on to <4 x i64>
  %ni = call <4 x i64> @VPXOR.4xi64(<4 x i64> %xi, <4 x i64> %ot)
  %r = trunc <4 x i64> %ni to <4 x i1>
  ret <4 x i1> %r
}

define <4 x i1> @vsle.4xi64(<4 x i64> "reg"="yv" %a, <4 x i64> "reg"="yv" %b) "recipe"="vsle.4xi64.expand" "feature"="avx2" {
  %r = icmp sle <4 x i64> %a, %b
  ret <4 x i1> %r
}

define <4 x i1> @vsle.4xi64.expand(<4 x i64> %a, <4 x i64> %b) {
  %c = call <4 x i1> @VPCMPGTQ.4xi64(<4 x i64> %a, <4 x i64> %b)
  %xi = sext <4 x i1> %c to <4 x i64>
  %on = call <4 x i64> @VALLONES()
  %ot = bitcast <4 x i64> %on to <4 x i64>
  %ni = call <4 x i64> @VPXOR.4xi64(<4 x i64> %xi, <4 x i64> %ot)
  %r = trunc <4 x i64> %ni to <4 x i1>
  ret <4 x i1> %r
}

define <4 x i1> @vugt.4xi64(<4 x i64> "reg"="yv" %a, <4 x i64> "reg"="yv" %b) "recipe"="vugt.4xi64.expand" "feature"="avx2" {
  %r = icmp ugt <4 x i64> %a, %b
  ret <4 x i1> %r
}

define <4 x i1> @vugt.4xi64.expand(<4 x i64> %a, <4 x i64> %b) {
  %o = call <4 x i64> @VALLONES()
  %sg = call <4 x i64> @VPSLLQi.4xi64(<4 x i64> %o, i64 63)
  %fa = call <4 x i64> @VPXOR.4xi64(<4 x i64> %a, <4 x i64> %sg)
  %fb = call <4 x i64> @VPXOR.4xi64(<4 x i64> %b, <4 x i64> %sg)
  %c = call <4 x i1> @VPCMPGTQ.4xi64(<4 x i64> %fa, <4 x i64> %fb)
  ret <4 x i1> %c
}

define <4 x i1> @vult.4xi64(<4 x i64> "reg"="yv" %a, <4 x i64> "reg"="yv" %b) "recipe"="vult.4xi64.expand" "feature"="avx2" {
  %r = icmp ult <4 x i64> %a, %b
  ret <4 x i1> %r
}

define <4 x i1> @vult.4xi64.expand(<4 x i64> %a, <4 x i64> %b) {
  %o = call <4 x i64> @VALLONES()
  %sg = call <4 x i64> @VPSLLQi.4xi64(<4 x i64> %o, i64 63)
  %fa = call <4 x i64> @VPXOR.4xi64(<4 x i64> %a, <4 x i64> %sg)
  %fb = call <4 x i64> @VPXOR.4xi64(<4 x i64> %b, <4 x i64> %sg)
  %c = call <4 x i1> @VPCMPGTQ.4xi64(<4 x i64> %fb, <4 x i64> %fa)
  ret <4 x i1> %c
}

define <4 x i1> @vule.4xi64(<4 x i64> "reg"="yv" %a, <4 x i64> "reg"="yv" %b) "recipe"="vule.4xi64.expand" "feature"="avx2" {
  %r = icmp ule <4 x i64> %a, %b
  ret <4 x i1> %r
}

define <4 x i1> @vule.4xi64.expand(<4 x i64> %a, <4 x i64> %b) {
  %o = call <4 x i64> @VALLONES()
  %sg = call <4 x i64> @VPSLLQi.4xi64(<4 x i64> %o, i64 63)
  %fa = call <4 x i64> @VPXOR.4xi64(<4 x i64> %a, <4 x i64> %sg)
  %fb = call <4 x i64> @VPXOR.4xi64(<4 x i64> %b, <4 x i64> %sg)
  %c = call <4 x i1> @VPCMPGTQ.4xi64(<4 x i64> %fa, <4 x i64> %fb)
  %xi = sext <4 x i1> %c to <4 x i64>
  %on = call <4 x i64> @VALLONES()
  %ot = bitcast <4 x i64> %on to <4 x i64>
  %ni = call <4 x i64> @VPXOR.4xi64(<4 x i64> %xi, <4 x i64> %ot)
  %r = trunc <4 x i64> %ni to <4 x i1>
  ret <4 x i1> %r
}

define <4 x i1> @vuge.4xi64(<4 x i64> "reg"="yv" %a, <4 x i64> "reg"="yv" %b) "recipe"="vuge.4xi64.expand" "feature"="avx2" {
  %r = icmp uge <4 x i64> %a, %b
  ret <4 x i1> %r
}

define <4 x i1> @vuge.4xi64.expand(<4 x i64> %a, <4 x i64> %b) {
  %o = call <4 x i64> @VALLONES()
  %sg = call <4 x i64> @VPSLLQi.4xi64(<4 x i64> %o, i64 63)
  %fa = call <4 x i64> @VPXOR.4xi64(<4 x i64> %a, <4 x i64> %sg)
  %fb = call <4 x i64> @VPXOR.4xi64(<4 x i64> %b, <4 x i64> %sg)
  %c = call <4 x i1> @VPCMPGTQ.4xi64(<4 x i64> %fb, <4 x i64> %fa)
  %xi = sext <4 x i1> %c to <4 x i64>
  %on = call <4 x i64> @VALLONES()
  %ot = bitcast <4 x i64> %on to <4 x i64>
  %ni = call <4 x i64> @VPXOR.4xi64(<4 x i64> %xi, <4 x i64> %ot)
  %r = trunc <4 x i64> %ni to <4 x i1>
  ret <4 x i1> %r
}

declare <4 x i32> @llvm.abs.v4i32(<4 x i32>, i1)

define <4 x i32> @abs.false.4xi32(<4 x i32> "reg"="xv" %x) "recipe"="abs.false.4xi32.expand" {
  %r = call <4 x i32> @llvm.abs.v4i32(<4 x i32> %x, i1 false)
  ret <4 x i32> %r
}

define <4 x i32> @abs.false.4xi32.expand(<4 x i32> %x) {
  %s = call <4 x i32> @PSRADi.4xi32(<4 x i32> %x, i32 31)
  %y = call <4 x i32> @PXOR.4xi32(<4 x i32> %x, <4 x i32> %s)
  %r = call <4 x i32> @PSUBD.4xi32(<4 x i32> %y, <4 x i32> %s)
  ret <4 x i32> %r
}

define <4 x i32> @abs.true.4xi32(<4 x i32> "reg"="xv" %x) "recipe"="abs.true.4xi32.expand" {
  %r = call <4 x i32> @llvm.abs.v4i32(<4 x i32> %x, i1 true)
  ret <4 x i32> %r
}

define <4 x i32> @abs.true.4xi32.expand(<4 x i32> %x) {
  %s = call <4 x i32> @PSRADi.4xi32(<4 x i32> %x, i32 31)
  %y = call <4 x i32> @PXOR.4xi32(<4 x i32> %x, <4 x i32> %s)
  %r = call <4 x i32> @PSUBD.4xi32(<4 x i32> %y, <4 x i32> %s)
  ret <4 x i32> %r
}

declare <8 x i16> @llvm.abs.v8i16(<8 x i16>, i1)

define <8 x i16> @abs.false.8xi16(<8 x i16> "reg"="xv" %x) "recipe"="abs.false.8xi16.expand" {
  %r = call <8 x i16> @llvm.abs.v8i16(<8 x i16> %x, i1 false)
  ret <8 x i16> %r
}

define <8 x i16> @abs.false.8xi16.expand(<8 x i16> %x) {
  %s = call <8 x i16> @PSRAWi.8xi16(<8 x i16> %x, i32 15)
  %y = call <8 x i16> @PXOR.8xi16(<8 x i16> %x, <8 x i16> %s)
  %r = call <8 x i16> @PSUBW.8xi16(<8 x i16> %y, <8 x i16> %s)
  ret <8 x i16> %r
}

define <8 x i16> @abs.true.8xi16(<8 x i16> "reg"="xv" %x) "recipe"="abs.true.8xi16.expand" {
  %r = call <8 x i16> @llvm.abs.v8i16(<8 x i16> %x, i1 true)
  ret <8 x i16> %r
}

define <8 x i16> @abs.true.8xi16.expand(<8 x i16> %x) {
  %s = call <8 x i16> @PSRAWi.8xi16(<8 x i16> %x, i32 15)
  %y = call <8 x i16> @PXOR.8xi16(<8 x i16> %x, <8 x i16> %s)
  %r = call <8 x i16> @PSUBW.8xi16(<8 x i16> %y, <8 x i16> %s)
  ret <8 x i16> %r
}

declare <2 x i32> @llvm.abs.v2i32(<2 x i32>, i1)

define <2 x i32> @abs.false.2xi32(<2 x i32> "reg"="xh" %x) "recipe"="abs.false.2xi32.expand" {
  %r = call <2 x i32> @llvm.abs.v2i32(<2 x i32> %x, i1 false)
  ret <2 x i32> %r
}

define <2 x i32> @abs.false.2xi32.expand(<2 x i32> %x) {
  %s = call <2 x i32> @PSRADi.2xi32(<2 x i32> %x, i32 31)
  %y = call <2 x i32> @PXOR.2xi32(<2 x i32> %x, <2 x i32> %s)
  %r = call <2 x i32> @PSUBD.2xi32(<2 x i32> %y, <2 x i32> %s)
  ret <2 x i32> %r
}

define <2 x i32> @abs.true.2xi32(<2 x i32> "reg"="xh" %x) "recipe"="abs.true.2xi32.expand" {
  %r = call <2 x i32> @llvm.abs.v2i32(<2 x i32> %x, i1 true)
  ret <2 x i32> %r
}

define <2 x i32> @abs.true.2xi32.expand(<2 x i32> %x) {
  %s = call <2 x i32> @PSRADi.2xi32(<2 x i32> %x, i32 31)
  %y = call <2 x i32> @PXOR.2xi32(<2 x i32> %x, <2 x i32> %s)
  %r = call <2 x i32> @PSUBD.2xi32(<2 x i32> %y, <2 x i32> %s)
  ret <2 x i32> %r
}

define <4 x float> @maximum.4xf32(<4 x float> "reg"="xv" %a, <4 x float> "reg"="xv" %b) "recipe"="maximum.4xf32.expand" {
  %r = call <4 x float> @llvm.maximum.v4f32(<4 x float> %a, <4 x float> %b)
  ret <4 x float> %r
}

define <4 x float> @maximum.4xf32.expand(<4 x float> %a, <4 x float> %b) {
  %w = call <4 x i1> @CMPOLTPS.4xf32(<4 x float> %b, <4 x float> %a)
  %xc = bitcast <4 x i1> %w to <2 x i64>
  %xa = bitcast <4 x float> %a to <2 x i64>
  %xb = bitcast <4 x float> %b to <2 x i64>
  %xx = call <2 x i64> @PAND.2xi64(<2 x i64> %xc, <2 x i64> %xa)
  %xy = call <2 x i64> @PANDN.xv(<2 x i64> %xc, <2 x i64> %xb)
  %xi = call <2 x i64> @POR.2xi64(<2 x i64> %xx, <2 x i64> %xy)
  %x = bitcast <2 x i64> %xi to <4 x float>
  %e = call <4 x i1> @CMPOEQPS.4xf32(<4 x float> %a, <4 x float> %b)
  %ai = bitcast <4 x float> %a to <2 x i64>
  %bi = bitcast <4 x float> %b to <2 x i64>
  %ci = call <2 x i64> @PAND.2xi64(<2 x i64> %ai, <2 x i64> %bi)
  %c = bitcast <2 x i64> %ci to <4 x float>
  %yc = bitcast <4 x i1> %e to <2 x i64>
  %ya = bitcast <4 x float> %c to <2 x i64>
  %yb = bitcast <4 x float> %x to <2 x i64>
  %yx = call <2 x i64> @PAND.2xi64(<2 x i64> %yc, <2 x i64> %ya)
  %yy = call <2 x i64> @PANDN.xv(<2 x i64> %yc, <2 x i64> %yb)
  %yi = call <2 x i64> @POR.2xi64(<2 x i64> %yx, <2 x i64> %yy)
  %y = bitcast <2 x i64> %yi to <4 x float>
  %u = call <4 x i1> @CMPUNOPS.4xf32(<4 x float> %a, <4 x float> %b)
  %nan = call <4 x float> @FADDPS.4xf32(<4 x float> %a, <4 x float> %b)
  %rc = bitcast <4 x i1> %u to <2 x i64>
  %ra = bitcast <4 x float> %nan to <2 x i64>
  %rb = bitcast <4 x float> %y to <2 x i64>
  %rx = call <2 x i64> @PAND.2xi64(<2 x i64> %rc, <2 x i64> %ra)
  %ry = call <2 x i64> @PANDN.xv(<2 x i64> %rc, <2 x i64> %rb)
  %ri = call <2 x i64> @POR.2xi64(<2 x i64> %rx, <2 x i64> %ry)
  %r = bitcast <2 x i64> %ri to <4 x float>
  ret <4 x float> %r
}

declare <4 x float> @llvm.maximum.v4f32(<4 x float>, <4 x float>)

define <4 x float> @minimum.4xf32(<4 x float> "reg"="xv" %a, <4 x float> "reg"="xv" %b) "recipe"="minimum.4xf32.expand" {
  %r = call <4 x float> @llvm.minimum.v4f32(<4 x float> %a, <4 x float> %b)
  ret <4 x float> %r
}

define <4 x float> @minimum.4xf32.expand(<4 x float> %a, <4 x float> %b) {
  %w = call <4 x i1> @CMPOLTPS.4xf32(<4 x float> %a, <4 x float> %b)
  %xc = bitcast <4 x i1> %w to <2 x i64>
  %xa = bitcast <4 x float> %a to <2 x i64>
  %xb = bitcast <4 x float> %b to <2 x i64>
  %xx = call <2 x i64> @PAND.2xi64(<2 x i64> %xc, <2 x i64> %xa)
  %xy = call <2 x i64> @PANDN.xv(<2 x i64> %xc, <2 x i64> %xb)
  %xi = call <2 x i64> @POR.2xi64(<2 x i64> %xx, <2 x i64> %xy)
  %x = bitcast <2 x i64> %xi to <4 x float>
  %e = call <4 x i1> @CMPOEQPS.4xf32(<4 x float> %a, <4 x float> %b)
  %ai = bitcast <4 x float> %a to <2 x i64>
  %bi = bitcast <4 x float> %b to <2 x i64>
  %ci = call <2 x i64> @POR.2xi64(<2 x i64> %ai, <2 x i64> %bi)
  %c = bitcast <2 x i64> %ci to <4 x float>
  %yc = bitcast <4 x i1> %e to <2 x i64>
  %ya = bitcast <4 x float> %c to <2 x i64>
  %yb = bitcast <4 x float> %x to <2 x i64>
  %yx = call <2 x i64> @PAND.2xi64(<2 x i64> %yc, <2 x i64> %ya)
  %yy = call <2 x i64> @PANDN.xv(<2 x i64> %yc, <2 x i64> %yb)
  %yi = call <2 x i64> @POR.2xi64(<2 x i64> %yx, <2 x i64> %yy)
  %y = bitcast <2 x i64> %yi to <4 x float>
  %u = call <4 x i1> @CMPUNOPS.4xf32(<4 x float> %a, <4 x float> %b)
  %nan = call <4 x float> @FADDPS.4xf32(<4 x float> %a, <4 x float> %b)
  %rc = bitcast <4 x i1> %u to <2 x i64>
  %ra = bitcast <4 x float> %nan to <2 x i64>
  %rb = bitcast <4 x float> %y to <2 x i64>
  %rx = call <2 x i64> @PAND.2xi64(<2 x i64> %rc, <2 x i64> %ra)
  %ry = call <2 x i64> @PANDN.xv(<2 x i64> %rc, <2 x i64> %rb)
  %ri = call <2 x i64> @POR.2xi64(<2 x i64> %rx, <2 x i64> %ry)
  %r = bitcast <2 x i64> %ri to <4 x float>
  ret <4 x float> %r
}

declare <4 x float> @llvm.minimum.v4f32(<4 x float>, <4 x float>)

define <2 x double> @maximum.2xf64(<2 x double> "reg"="xv" %a, <2 x double> "reg"="xv" %b) "recipe"="maximum.2xf64.expand" {
  %r = call <2 x double> @llvm.maximum.v2f64(<2 x double> %a, <2 x double> %b)
  ret <2 x double> %r
}

define <2 x double> @maximum.2xf64.expand(<2 x double> %a, <2 x double> %b) {
  %w = call <2 x i1> @CMPOLTPD.2xf64(<2 x double> %b, <2 x double> %a)
  %xc = bitcast <2 x i1> %w to <2 x i64>
  %xa = bitcast <2 x double> %a to <2 x i64>
  %xb = bitcast <2 x double> %b to <2 x i64>
  %xx = call <2 x i64> @PAND.2xi64(<2 x i64> %xc, <2 x i64> %xa)
  %xy = call <2 x i64> @PANDN.xv(<2 x i64> %xc, <2 x i64> %xb)
  %xi = call <2 x i64> @POR.2xi64(<2 x i64> %xx, <2 x i64> %xy)
  %x = bitcast <2 x i64> %xi to <2 x double>
  %e = call <2 x i1> @CMPOEQPD.2xf64(<2 x double> %a, <2 x double> %b)
  %ai = bitcast <2 x double> %a to <2 x i64>
  %bi = bitcast <2 x double> %b to <2 x i64>
  %ci = call <2 x i64> @PAND.2xi64(<2 x i64> %ai, <2 x i64> %bi)
  %c = bitcast <2 x i64> %ci to <2 x double>
  %yc = bitcast <2 x i1> %e to <2 x i64>
  %ya = bitcast <2 x double> %c to <2 x i64>
  %yb = bitcast <2 x double> %x to <2 x i64>
  %yx = call <2 x i64> @PAND.2xi64(<2 x i64> %yc, <2 x i64> %ya)
  %yy = call <2 x i64> @PANDN.xv(<2 x i64> %yc, <2 x i64> %yb)
  %yi = call <2 x i64> @POR.2xi64(<2 x i64> %yx, <2 x i64> %yy)
  %y = bitcast <2 x i64> %yi to <2 x double>
  %u = call <2 x i1> @CMPUNOPD.2xf64(<2 x double> %a, <2 x double> %b)
  %nan = call <2 x double> @FADDPD.2xf64(<2 x double> %a, <2 x double> %b)
  %rc = bitcast <2 x i1> %u to <2 x i64>
  %ra = bitcast <2 x double> %nan to <2 x i64>
  %rb = bitcast <2 x double> %y to <2 x i64>
  %rx = call <2 x i64> @PAND.2xi64(<2 x i64> %rc, <2 x i64> %ra)
  %ry = call <2 x i64> @PANDN.xv(<2 x i64> %rc, <2 x i64> %rb)
  %ri = call <2 x i64> @POR.2xi64(<2 x i64> %rx, <2 x i64> %ry)
  %r = bitcast <2 x i64> %ri to <2 x double>
  ret <2 x double> %r
}

declare <2 x double> @llvm.maximum.v2f64(<2 x double>, <2 x double>)

define <2 x double> @minimum.2xf64(<2 x double> "reg"="xv" %a, <2 x double> "reg"="xv" %b) "recipe"="minimum.2xf64.expand" {
  %r = call <2 x double> @llvm.minimum.v2f64(<2 x double> %a, <2 x double> %b)
  ret <2 x double> %r
}

define <2 x double> @minimum.2xf64.expand(<2 x double> %a, <2 x double> %b) {
  %w = call <2 x i1> @CMPOLTPD.2xf64(<2 x double> %a, <2 x double> %b)
  %xc = bitcast <2 x i1> %w to <2 x i64>
  %xa = bitcast <2 x double> %a to <2 x i64>
  %xb = bitcast <2 x double> %b to <2 x i64>
  %xx = call <2 x i64> @PAND.2xi64(<2 x i64> %xc, <2 x i64> %xa)
  %xy = call <2 x i64> @PANDN.xv(<2 x i64> %xc, <2 x i64> %xb)
  %xi = call <2 x i64> @POR.2xi64(<2 x i64> %xx, <2 x i64> %xy)
  %x = bitcast <2 x i64> %xi to <2 x double>
  %e = call <2 x i1> @CMPOEQPD.2xf64(<2 x double> %a, <2 x double> %b)
  %ai = bitcast <2 x double> %a to <2 x i64>
  %bi = bitcast <2 x double> %b to <2 x i64>
  %ci = call <2 x i64> @POR.2xi64(<2 x i64> %ai, <2 x i64> %bi)
  %c = bitcast <2 x i64> %ci to <2 x double>
  %yc = bitcast <2 x i1> %e to <2 x i64>
  %ya = bitcast <2 x double> %c to <2 x i64>
  %yb = bitcast <2 x double> %x to <2 x i64>
  %yx = call <2 x i64> @PAND.2xi64(<2 x i64> %yc, <2 x i64> %ya)
  %yy = call <2 x i64> @PANDN.xv(<2 x i64> %yc, <2 x i64> %yb)
  %yi = call <2 x i64> @POR.2xi64(<2 x i64> %yx, <2 x i64> %yy)
  %y = bitcast <2 x i64> %yi to <2 x double>
  %u = call <2 x i1> @CMPUNOPD.2xf64(<2 x double> %a, <2 x double> %b)
  %nan = call <2 x double> @FADDPD.2xf64(<2 x double> %a, <2 x double> %b)
  %rc = bitcast <2 x i1> %u to <2 x i64>
  %ra = bitcast <2 x double> %nan to <2 x i64>
  %rb = bitcast <2 x double> %y to <2 x i64>
  %rx = call <2 x i64> @PAND.2xi64(<2 x i64> %rc, <2 x i64> %ra)
  %ry = call <2 x i64> @PANDN.xv(<2 x i64> %rc, <2 x i64> %rb)
  %ri = call <2 x i64> @POR.2xi64(<2 x i64> %rx, <2 x i64> %ry)
  %r = bitcast <2 x i64> %ri to <2 x double>
  ret <2 x double> %r
}

declare <2 x double> @llvm.minimum.v2f64(<2 x double>, <2 x double>)

define <8 x float> @maximum.8xf32(<8 x float> "reg"="yv" %a, <8 x float> "reg"="yv" %b) "recipe"="maximum.8xf32.expand" "feature"="avx2" {
  %r = call <8 x float> @llvm.maximum.v8f32(<8 x float> %a, <8 x float> %b)
  ret <8 x float> %r
}

define <8 x float> @maximum.8xf32.expand(<8 x float> %a, <8 x float> %b) {
  %w = call <8 x i1> @VCMPOGTPS.8xf32(<8 x float> %a, <8 x float> %b)
  %x = call <8 x float> @VBLENDVPS.8xf32(<8 x i1> %w, <8 x float> %a, <8 x float> %b)
  %e = call <8 x i1> @VCMPOEQPS.8xf32(<8 x float> %a, <8 x float> %b)
  %ai = bitcast <8 x float> %a to <4 x i64>
  %bi = bitcast <8 x float> %b to <4 x i64>
  %ci = call <4 x i64> @VPAND.4xi64(<4 x i64> %ai, <4 x i64> %bi)
  %c = bitcast <4 x i64> %ci to <8 x float>
  %y = call <8 x float> @VBLENDVPS.8xf32(<8 x i1> %e, <8 x float> %c, <8 x float> %x)
  %u = call <8 x i1> @VCMPUNOPS.8xf32(<8 x float> %a, <8 x float> %b)
  %nan = call <8 x float> @VADDPS.8xf32(<8 x float> %a, <8 x float> %b)
  %r = call <8 x float> @VBLENDVPS.8xf32(<8 x i1> %u, <8 x float> %nan, <8 x float> %y)
  ret <8 x float> %r
}

declare <8 x float> @llvm.maximum.v8f32(<8 x float>, <8 x float>)

define <8 x float> @minimum.8xf32(<8 x float> "reg"="yv" %a, <8 x float> "reg"="yv" %b) "recipe"="minimum.8xf32.expand" "feature"="avx2" {
  %r = call <8 x float> @llvm.minimum.v8f32(<8 x float> %a, <8 x float> %b)
  ret <8 x float> %r
}

define <8 x float> @minimum.8xf32.expand(<8 x float> %a, <8 x float> %b) {
  %w = call <8 x i1> @VCMPOLTPS.8xf32(<8 x float> %a, <8 x float> %b)
  %x = call <8 x float> @VBLENDVPS.8xf32(<8 x i1> %w, <8 x float> %a, <8 x float> %b)
  %e = call <8 x i1> @VCMPOEQPS.8xf32(<8 x float> %a, <8 x float> %b)
  %ai = bitcast <8 x float> %a to <4 x i64>
  %bi = bitcast <8 x float> %b to <4 x i64>
  %ci = call <4 x i64> @VPOR.4xi64(<4 x i64> %ai, <4 x i64> %bi)
  %c = bitcast <4 x i64> %ci to <8 x float>
  %y = call <8 x float> @VBLENDVPS.8xf32(<8 x i1> %e, <8 x float> %c, <8 x float> %x)
  %u = call <8 x i1> @VCMPUNOPS.8xf32(<8 x float> %a, <8 x float> %b)
  %nan = call <8 x float> @VADDPS.8xf32(<8 x float> %a, <8 x float> %b)
  %r = call <8 x float> @VBLENDVPS.8xf32(<8 x i1> %u, <8 x float> %nan, <8 x float> %y)
  ret <8 x float> %r
}

declare <8 x float> @llvm.minimum.v8f32(<8 x float>, <8 x float>)

define <4 x double> @maximum.4xf64(<4 x double> "reg"="yv" %a, <4 x double> "reg"="yv" %b) "recipe"="maximum.4xf64.expand" "feature"="avx2" {
  %r = call <4 x double> @llvm.maximum.v4f64(<4 x double> %a, <4 x double> %b)
  ret <4 x double> %r
}

define <4 x double> @maximum.4xf64.expand(<4 x double> %a, <4 x double> %b) {
  %w = call <4 x i1> @VCMPOGTPD.4xf64(<4 x double> %a, <4 x double> %b)
  %x = call <4 x double> @VBLENDVPD.4xf64(<4 x i1> %w, <4 x double> %a, <4 x double> %b)
  %e = call <4 x i1> @VCMPOEQPD.4xf64(<4 x double> %a, <4 x double> %b)
  %ai = bitcast <4 x double> %a to <4 x i64>
  %bi = bitcast <4 x double> %b to <4 x i64>
  %ci = call <4 x i64> @VPAND.4xi64(<4 x i64> %ai, <4 x i64> %bi)
  %c = bitcast <4 x i64> %ci to <4 x double>
  %y = call <4 x double> @VBLENDVPD.4xf64(<4 x i1> %e, <4 x double> %c, <4 x double> %x)
  %u = call <4 x i1> @VCMPUNOPD.4xf64(<4 x double> %a, <4 x double> %b)
  %nan = call <4 x double> @VADDPD.4xf64(<4 x double> %a, <4 x double> %b)
  %r = call <4 x double> @VBLENDVPD.4xf64(<4 x i1> %u, <4 x double> %nan, <4 x double> %y)
  ret <4 x double> %r
}

declare <4 x double> @llvm.maximum.v4f64(<4 x double>, <4 x double>)

define <4 x double> @minimum.4xf64(<4 x double> "reg"="yv" %a, <4 x double> "reg"="yv" %b) "recipe"="minimum.4xf64.expand" "feature"="avx2" {
  %r = call <4 x double> @llvm.minimum.v4f64(<4 x double> %a, <4 x double> %b)
  ret <4 x double> %r
}

define <4 x double> @minimum.4xf64.expand(<4 x double> %a, <4 x double> %b) {
  %w = call <4 x i1> @VCMPOLTPD.4xf64(<4 x double> %a, <4 x double> %b)
  %x = call <4 x double> @VBLENDVPD.4xf64(<4 x i1> %w, <4 x double> %a, <4 x double> %b)
  %e = call <4 x i1> @VCMPOEQPD.4xf64(<4 x double> %a, <4 x double> %b)
  %ai = bitcast <4 x double> %a to <4 x i64>
  %bi = bitcast <4 x double> %b to <4 x i64>
  %ci = call <4 x i64> @VPOR.4xi64(<4 x i64> %ai, <4 x i64> %bi)
  %c = bitcast <4 x i64> %ci to <4 x double>
  %y = call <4 x double> @VBLENDVPD.4xf64(<4 x i1> %e, <4 x double> %c, <4 x double> %x)
  %u = call <4 x i1> @VCMPUNOPD.4xf64(<4 x double> %a, <4 x double> %b)
  %nan = call <4 x double> @VADDPD.4xf64(<4 x double> %a, <4 x double> %b)
  %r = call <4 x double> @VBLENDVPD.4xf64(<4 x i1> %u, <4 x double> %nan, <4 x double> %y)
  ret <4 x double> %r
}

declare <4 x double> @llvm.minimum.v4f64(<4 x double>, <4 x double>)

define <2 x i64> @MOVAPSrr(<2 x i64> "reg"="x" %n) "enc"="rex(0, d, 0, n) | 00001111 | 00101000 | rr(d, n)" "asm"="movaps {n}, {d}" "copy"="x" "dclass"="x" "nodecode" {
  ret <2 x i64> %n
}

define <2 x i64> @VMOVAPSxe(<2 x i64> "reg"="xe" %n) "enc"="{ if(n<8) vex(d, 0, n, 1, 0, 0, 0, 0) | 00101000 | rr(d, n) ; if(d<8) vex(n, 0, d, 1, 0, 0, 0, 0) | 00101001 | rr(n, d) ; vex(d, 0, n, 1, 0, 0, 0, 0) | 00101000 | rr(d, n) ; evexr(d, n, 1, 0, 0, 0, 0, 0, 0, 0) | 00101000 | rr(d, n) }" "asm"="vmovaps {n}, {d}" "copy"="xe" "dclass"="xe" "nodecode" {
  ret <2 x i64> %n
}

define <4 x i64> @VMOVAPSye(<4 x i64> "reg"="ye" %n) "enc"="{ if(n<8) vex(d, 0, n, 1, 0, 0, 1, 0) | 00101000 | rr(d, n) ; if(d<8) vex(n, 0, d, 1, 0, 0, 1, 0) | 00101001 | rr(n, d) ; vex(d, 0, n, 1, 0, 0, 1, 0) | 00101000 | rr(d, n) ; evexr(d, n, 1, 0, 0, 1, 0, 0, 0, 0) | 00101000 | rr(d, n) }" "asm"="vmovaps {n}, {d}" "copy"="ye" "dclass"="ye" "nodecode" {
  ret <4 x i64> %n
}

define <8 x i64> @VMOVAPSz(<8 x i64> "reg"="z" %n) "enc"="evexr(d, n, 1, 0, 0, 2, 0, 0, 0, 0) | 00101000 | rr(d, n)" "asm"="vmovaps {n}, {d}" "copy"="z" "dclass"="z" "nodecode" {
  ret <8 x i64> %n
}

define i64 @KMOVQ(i64 "reg"="k" %n) "enc"="vex(d, 0, n, 1, 1, 0, 0, 0) | 10010000 | rr(d, n)" "asm"="kmovq {n}, {d}" "copy"="k" "dclass"="k" "nodecode" {
  ret i64 %n
}

define i64 @KMOVQkr(i64 "reg"="r64" %n) "enc"="vex(d, 0, n, 1, 1, 0, 0, 3) | 10010010 | 11 reg:3=d&7 rm:3=n&7" "asm"="kmovq {n}, {d}" "dclass"="k" "role"="move.k.r64" "nomatch" "nodecode" {
  ret i64 %n
}

define i64 @KMOVQrk(i64 "reg"="k" %n) "enc"="vex(d, 0, n, 1, 1, 0, 0, 3) | 10010011 | 11 reg:3=d&7 rm:3=n&7" "asm"="kmovq {n}, {d}" "dclass"="r64" "role"="move.r64.k" "nomatch" "nodecode" {
  ret i64 %n
}

define i64 @KMOVQld(ptr "reg"="r64" %n, i64 "imm" %off) "enc"="vexm(d, n, 4, 1, 1, 0, 0, 0) | 10010000 | rm(d, n, 4, 1, off, 0, 1)" "asm"="kmovq {off}({n}), {d}" "dclass"="k" "role"="load.k" "nomatch" "nodecode" {
  %a = getelementptr i8, ptr %n, i64 %off
  %r = load i64, ptr %a
  ret i64 %r
}

define void @KMOVQst(i64 "reg"="k" %t, ptr "reg"="r64" %n, i64 "imm" %off) "enc"="vexm(t, n, 4, 1, 1, 0, 0, 0) | 10010001 | rm(t, n, 4, 1, off, 0, 1)" "asm"="kmovq {t}, {off}({n})" "role"="store.k" "nomatch" "nodecode" {
  %a = getelementptr i8, ptr %n, i64 %off
  store i64 %t, ptr %a
  ret void
}

define <2 x i64> @X86.aesni.aesdec(<2 x i64> "reg"="xv" "tied" %a0, <2 x i64> "reg"="xv" %a1) "enc"="01100110 | rex(0, d, 0, a1) | 00001111 | 00111000 | 11011110 | 11 reg:3=d&7 rm:3=a1&7" "asm"="aesdec {a1}, {d}" "dclass"="xv" "feature"="aes" "nodecode" {
  %r = call <2 x i64> @llvm.x86.aesni.aesdec(<2 x i64> %a0, <2 x i64> %a1)
  ret <2 x i64> %r
}

declare <2 x i64> @llvm.x86.aesni.aesdec(<2 x i64>, <2 x i64>)

define <4 x i64> @X86.aesni.aesdec.256(<4 x i64> "reg"="yv" %a0, <4 x i64> "reg"="yv" %a1) "enc"="{ vex(d, 0, a1, 2, 0, a0, 1, 1) | 11011110 | 11 reg:3=d&7 rm:3=a1&7 ; evexr(d, a1, 2, 0, a0, 1, 1, 0, 0, 0) | 11011110 | 11 reg:3=d&7 rm:3=a1&7 }" "asm"="vaesdec {a1}, {a0}, {d}" "dclass"="yv" "feature"="vaes" "nodecode" {
  %r = call <4 x i64> @llvm.x86.aesni.aesdec.256(<4 x i64> %a0, <4 x i64> %a1)
  ret <4 x i64> %r
}

declare <4 x i64> @llvm.x86.aesni.aesdec.256(<4 x i64>, <4 x i64>)

define <8 x i64> @X86.aesni.aesdec.512(<8 x i64> "reg"="zv" %a0, <8 x i64> "reg"="zv" %a1) "enc"="evexr(d, a1, 2, 0, a0, 2, 1, 0, 0, 0) | 11011110 | 11 reg:3=d&7 rm:3=a1&7" "asm"="vaesdec {a1}, {a0}, {d}" "dclass"="zv" "feature"="vaes,avx512f" "nodecode" {
  %r = call <8 x i64> @llvm.x86.aesni.aesdec.512(<8 x i64> %a0, <8 x i64> %a1)
  ret <8 x i64> %r
}

declare <8 x i64> @llvm.x86.aesni.aesdec.512(<8 x i64>, <8 x i64>)

define <2 x i64> @X86.aesni.aesdeclast(<2 x i64> "reg"="xv" "tied" %a0, <2 x i64> "reg"="xv" %a1) "enc"="01100110 | rex(0, d, 0, a1) | 00001111 | 00111000 | 11011111 | 11 reg:3=d&7 rm:3=a1&7" "asm"="aesdeclast {a1}, {d}" "dclass"="xv" "feature"="aes" "nodecode" {
  %r = call <2 x i64> @llvm.x86.aesni.aesdeclast(<2 x i64> %a0, <2 x i64> %a1)
  ret <2 x i64> %r
}

declare <2 x i64> @llvm.x86.aesni.aesdeclast(<2 x i64>, <2 x i64>)

define <4 x i64> @X86.aesni.aesdeclast.256(<4 x i64> "reg"="yv" %a0, <4 x i64> "reg"="yv" %a1) "enc"="{ vex(d, 0, a1, 2, 0, a0, 1, 1) | 11011111 | 11 reg:3=d&7 rm:3=a1&7 ; evexr(d, a1, 2, 0, a0, 1, 1, 0, 0, 0) | 11011111 | 11 reg:3=d&7 rm:3=a1&7 }" "asm"="vaesdeclast {a1}, {a0}, {d}" "dclass"="yv" "feature"="vaes" "nodecode" {
  %r = call <4 x i64> @llvm.x86.aesni.aesdeclast.256(<4 x i64> %a0, <4 x i64> %a1)
  ret <4 x i64> %r
}

declare <4 x i64> @llvm.x86.aesni.aesdeclast.256(<4 x i64>, <4 x i64>)

define <8 x i64> @X86.aesni.aesdeclast.512(<8 x i64> "reg"="zv" %a0, <8 x i64> "reg"="zv" %a1) "enc"="evexr(d, a1, 2, 0, a0, 2, 1, 0, 0, 0) | 11011111 | 11 reg:3=d&7 rm:3=a1&7" "asm"="vaesdeclast {a1}, {a0}, {d}" "dclass"="zv" "feature"="vaes,avx512f" "nodecode" {
  %r = call <8 x i64> @llvm.x86.aesni.aesdeclast.512(<8 x i64> %a0, <8 x i64> %a1)
  ret <8 x i64> %r
}

declare <8 x i64> @llvm.x86.aesni.aesdeclast.512(<8 x i64>, <8 x i64>)

define <2 x i64> @X86.aesni.aesenc(<2 x i64> "reg"="xv" "tied" %a0, <2 x i64> "reg"="xv" %a1) "enc"="01100110 | rex(0, d, 0, a1) | 00001111 | 00111000 | 11011100 | 11 reg:3=d&7 rm:3=a1&7" "asm"="aesenc {a1}, {d}" "dclass"="xv" "feature"="aes" "nodecode" {
  %r = call <2 x i64> @llvm.x86.aesni.aesenc(<2 x i64> %a0, <2 x i64> %a1)
  ret <2 x i64> %r
}

declare <2 x i64> @llvm.x86.aesni.aesenc(<2 x i64>, <2 x i64>)

define <4 x i64> @X86.aesni.aesenc.256(<4 x i64> "reg"="yv" %a0, <4 x i64> "reg"="yv" %a1) "enc"="{ vex(d, 0, a1, 2, 0, a0, 1, 1) | 11011100 | 11 reg:3=d&7 rm:3=a1&7 ; evexr(d, a1, 2, 0, a0, 1, 1, 0, 0, 0) | 11011100 | 11 reg:3=d&7 rm:3=a1&7 }" "asm"="vaesenc {a1}, {a0}, {d}" "dclass"="yv" "feature"="vaes" "nodecode" {
  %r = call <4 x i64> @llvm.x86.aesni.aesenc.256(<4 x i64> %a0, <4 x i64> %a1)
  ret <4 x i64> %r
}

declare <4 x i64> @llvm.x86.aesni.aesenc.256(<4 x i64>, <4 x i64>)

define <8 x i64> @X86.aesni.aesenc.512(<8 x i64> "reg"="zv" %a0, <8 x i64> "reg"="zv" %a1) "enc"="evexr(d, a1, 2, 0, a0, 2, 1, 0, 0, 0) | 11011100 | 11 reg:3=d&7 rm:3=a1&7" "asm"="vaesenc {a1}, {a0}, {d}" "dclass"="zv" "feature"="vaes,avx512f" "nodecode" {
  %r = call <8 x i64> @llvm.x86.aesni.aesenc.512(<8 x i64> %a0, <8 x i64> %a1)
  ret <8 x i64> %r
}

declare <8 x i64> @llvm.x86.aesni.aesenc.512(<8 x i64>, <8 x i64>)

define <2 x i64> @X86.aesni.aesenclast(<2 x i64> "reg"="xv" "tied" %a0, <2 x i64> "reg"="xv" %a1) "enc"="01100110 | rex(0, d, 0, a1) | 00001111 | 00111000 | 11011101 | 11 reg:3=d&7 rm:3=a1&7" "asm"="aesenclast {a1}, {d}" "dclass"="xv" "feature"="aes" "nodecode" {
  %r = call <2 x i64> @llvm.x86.aesni.aesenclast(<2 x i64> %a0, <2 x i64> %a1)
  ret <2 x i64> %r
}

declare <2 x i64> @llvm.x86.aesni.aesenclast(<2 x i64>, <2 x i64>)

define <4 x i64> @X86.aesni.aesenclast.256(<4 x i64> "reg"="yv" %a0, <4 x i64> "reg"="yv" %a1) "enc"="{ vex(d, 0, a1, 2, 0, a0, 1, 1) | 11011101 | 11 reg:3=d&7 rm:3=a1&7 ; evexr(d, a1, 2, 0, a0, 1, 1, 0, 0, 0) | 11011101 | 11 reg:3=d&7 rm:3=a1&7 }" "asm"="vaesenclast {a1}, {a0}, {d}" "dclass"="yv" "feature"="vaes" "nodecode" {
  %r = call <4 x i64> @llvm.x86.aesni.aesenclast.256(<4 x i64> %a0, <4 x i64> %a1)
  ret <4 x i64> %r
}

declare <4 x i64> @llvm.x86.aesni.aesenclast.256(<4 x i64>, <4 x i64>)

define <8 x i64> @X86.aesni.aesenclast.512(<8 x i64> "reg"="zv" %a0, <8 x i64> "reg"="zv" %a1) "enc"="evexr(d, a1, 2, 0, a0, 2, 1, 0, 0, 0) | 11011101 | 11 reg:3=d&7 rm:3=a1&7" "asm"="vaesenclast {a1}, {a0}, {d}" "dclass"="zv" "feature"="vaes,avx512f" "nodecode" {
  %r = call <8 x i64> @llvm.x86.aesni.aesenclast.512(<8 x i64> %a0, <8 x i64> %a1)
  ret <8 x i64> %r
}

declare <8 x i64> @llvm.x86.aesni.aesenclast.512(<8 x i64>, <8 x i64>)

define <2 x i64> @X86.aesni.aesimc(<2 x i64> "reg"="xv" %a0) "enc"="01100110 | rex(0, d, 0, a0) | 00001111 | 00111000 | 11011011 | 11 reg:3=d&7 rm:3=a0&7" "asm"="aesimc {a0}, {d}" "dclass"="xv" "feature"="aes" "nodecode" {
  %r = call <2 x i64> @llvm.x86.aesni.aesimc(<2 x i64> %a0)
  ret <2 x i64> %r
}

declare <2 x i64> @llvm.x86.aesni.aesimc(<2 x i64>)

define <2 x i64> @X86.aesni.aeskeygenassist(<2 x i64> "reg"="xv" %a0, i8 "imm" %a1) "enc"="01100110 | rex(0, d, 0, a0) | 00001111 | 00111010 | 11011111 | 11 reg:3=d&7 rm:3=a0&7 | { i1:8=a1 ; i1:s8=a1 }" "asm"="aeskeygenassist ${a1}, {a0}, {d}" "dclass"="xv" "feature"="aes" "nodecode" {
  %r = call <2 x i64> @llvm.x86.aesni.aeskeygenassist(<2 x i64> %a0, i8 %a1)
  ret <2 x i64> %r
}

declare <2 x i64> @llvm.x86.aesni.aeskeygenassist(<2 x i64>, i8)

define <4 x double> @X86.avx.cmp.pd.256(<4 x double> "reg"="yv" %a0, <4 x double> "reg"="yv" %a1, i8 "imm" %a2) "enc"="{ if(a1<8) vex(d, 0, a1, 1, 0, a0, 1, 1) | 11000010 | 11 reg:3=d&7 rm:3=a1&7 | { i1:8=a2 ; i1:s8=a2 } ; if(a0<8) if(((a2&3)==0)|((a2&3)==3)) vex(d, 0, a0, 1, 0, a1, 1, 1) | 11000010 | 11 reg:3=d&7 rm:3=a0&7 | { i1:8=a2 ; i1:s8=a2 } ; vex(d, 0, a1, 1, 0, a0, 1, 1) | 11000010 | 11 reg:3=d&7 rm:3=a1&7 | { i1:8=a2 ; i1:s8=a2 } ; if(((a2&3)==0)|((a2&3)==3)) vex(d, 0, a0, 1, 0, a1, 1, 1) | 11000010 | 11 reg:3=d&7 rm:3=a0&7 | { i1:8=a2 ; i1:s8=a2 } }" "asm"="vcmppd ${a2}, {a1}, {a0}, {d}" "dclass"="yv" "feature"="avx" "nodecode" {
  %r = call <4 x double> @llvm.x86.avx.cmp.pd.256(<4 x double> %a0, <4 x double> %a1, i8 %a2)
  ret <4 x double> %r
}

declare <4 x double> @llvm.x86.avx.cmp.pd.256(<4 x double>, <4 x double>, i8)

define <8 x float> @X86.avx.cmp.ps.256(<8 x float> "reg"="yv" %a0, <8 x float> "reg"="yv" %a1, i8 "imm" %a2) "enc"="{ if(a1<8) vex(d, 0, a1, 1, 0, a0, 1, 0) | 11000010 | 11 reg:3=d&7 rm:3=a1&7 | { i1:8=a2 ; i1:s8=a2 } ; if(a0<8) if(((a2&3)==0)|((a2&3)==3)) vex(d, 0, a0, 1, 0, a1, 1, 0) | 11000010 | 11 reg:3=d&7 rm:3=a0&7 | { i1:8=a2 ; i1:s8=a2 } ; vex(d, 0, a1, 1, 0, a0, 1, 0) | 11000010 | 11 reg:3=d&7 rm:3=a1&7 | { i1:8=a2 ; i1:s8=a2 } ; if(((a2&3)==0)|((a2&3)==3)) vex(d, 0, a0, 1, 0, a1, 1, 0) | 11000010 | 11 reg:3=d&7 rm:3=a0&7 | { i1:8=a2 ; i1:s8=a2 } }" "asm"="vcmpps ${a2}, {a1}, {a0}, {d}" "dclass"="yv" "feature"="avx" "nodecode" {
  %r = call <8 x float> @llvm.x86.avx.cmp.ps.256(<8 x float> %a0, <8 x float> %a1, i8 %a2)
  ret <8 x float> %r
}

declare <8 x float> @llvm.x86.avx.cmp.ps.256(<8 x float>, <8 x float>, i8)

define <4 x i32> @X86.avx.cvt.pd2dq.256(<4 x double> "reg"="yv" %a0) "enc"="{ vex(d, 0, a0, 1, 0, 0, 1, 3) | 11100110 | 11 reg:3=d&7 rm:3=a0&7 ; evexr(d, a0, 1, 1, 0, 1, 3, 0, 0, 0) | 11100110 | 11 reg:3=d&7 rm:3=a0&7 }" "asm"="vcvtpd2dq {a0}, {d}" "dclass"="xv" "feature"="avx" "nodecode" {
  %r = call <4 x i32> @llvm.x86.avx.cvt.pd2dq.256(<4 x double> %a0)
  ret <4 x i32> %r
}

declare <4 x i32> @llvm.x86.avx.cvt.pd2dq.256(<4 x double>)

define <8 x i32> @X86.avx.cvt.ps2dq.256(<8 x float> "reg"="yv" %a0) "enc"="{ vex(d, 0, a0, 1, 0, 0, 1, 1) | 01011011 | 11 reg:3=d&7 rm:3=a0&7 ; evexr(d, a0, 1, 0, 0, 1, 1, 0, 0, 0) | 01011011 | 11 reg:3=d&7 rm:3=a0&7 }" "asm"="vcvtps2dq {a0}, {d}" "dclass"="yv" "feature"="avx" "nodecode" {
  %r = call <8 x i32> @llvm.x86.avx.cvt.ps2dq.256(<8 x float> %a0)
  ret <8 x i32> %r
}

declare <8 x i32> @llvm.x86.avx.cvt.ps2dq.256(<8 x float>)

define <4 x i32> @X86.avx.cvtt.pd2dq.256(<4 x double> "reg"="yv" %a0) "enc"="{ vex(d, 0, a0, 1, 0, 0, 1, 1) | 11100110 | 11 reg:3=d&7 rm:3=a0&7 ; evexr(d, a0, 1, 1, 0, 1, 1, 0, 0, 0) | 11100110 | 11 reg:3=d&7 rm:3=a0&7 }" "asm"="vcvttpd2dq {a0}, {d}" "dclass"="xv" "feature"="avx" "nodecode" {
  %r = call <4 x i32> @llvm.x86.avx.cvtt.pd2dq.256(<4 x double> %a0)
  ret <4 x i32> %r
}

declare <4 x i32> @llvm.x86.avx.cvtt.pd2dq.256(<4 x double>)

define <8 x i32> @X86.avx.cvtt.ps2dq.256(<8 x float> "reg"="yv" %a0) "enc"="{ vex(d, 0, a0, 1, 0, 0, 1, 2) | 01011011 | 11 reg:3=d&7 rm:3=a0&7 ; evexr(d, a0, 1, 0, 0, 1, 2, 0, 0, 0) | 01011011 | 11 reg:3=d&7 rm:3=a0&7 }" "asm"="vcvttps2dq {a0}, {d}" "dclass"="yv" "feature"="avx" "nodecode" {
  %r = call <8 x i32> @llvm.x86.avx.cvtt.ps2dq.256(<8 x float> %a0)
  ret <8 x i32> %r
}

declare <8 x i32> @llvm.x86.avx.cvtt.ps2dq.256(<8 x float>)

define <8 x float> @X86.avx.dp.ps.256(<8 x float> "reg"="yv" %a0, <8 x float> "reg"="yv" %a1, i8 "imm" %a2) "enc"="vex(d, 0, a1, 3, 0, a0, 1, 1) | 01000000 | 11 reg:3=d&7 rm:3=a1&7 | { i1:8=a2 ; i1:s8=a2 }" "asm"="vdpps ${a2}, {a1}, {a0}, {d}" "dclass"="yv" "feature"="avx" "nodecode" {
  %r = call <8 x float> @llvm.x86.avx.dp.ps.256(<8 x float> %a0, <8 x float> %a1, i8 %a2)
  ret <8 x float> %r
}

declare <8 x float> @llvm.x86.avx.dp.ps.256(<8 x float>, <8 x float>, i8)

define <32 x i8> @X86.avx.ldu.dq.256(ptr "reg"="r64" %a0) "enc"="vexm(d, a0, 4, 1, 0, 0, 1, 3) | 11110000 | rm(d, a0, 4, 1, 0, 0, 1)" "asm"="vlddqu 0({a0}), {d}" "dclass"="yv" "feature"="avx" "nodecode" {
  %r = call <32 x i8> @llvm.x86.avx.ldu.dq.256(ptr %a0)
  ret <32 x i8> %r
}

declare <32 x i8> @llvm.x86.avx.ldu.dq.256(ptr)

define <4 x double> @X86.avx.max.pd.256(<4 x double> "reg"="yv" %a0, <4 x double> "reg"="yv" %a1) "enc"="{ vex(d, 0, a1, 1, 0, a0, 1, 1) | 01011111 | 11 reg:3=d&7 rm:3=a1&7 ; evexr(d, a1, 1, 1, a0, 1, 1, 0, 0, 0) | 01011111 | 11 reg:3=d&7 rm:3=a1&7 }" "asm"="vmaxpd {a1}, {a0}, {d}" "dclass"="yv" "feature"="avx" "nodecode" {
  %r = call <4 x double> @llvm.x86.avx.max.pd.256(<4 x double> %a0, <4 x double> %a1)
  ret <4 x double> %r
}

declare <4 x double> @llvm.x86.avx.max.pd.256(<4 x double>, <4 x double>)

define <8 x float> @X86.avx.max.ps.256(<8 x float> "reg"="yv" %a0, <8 x float> "reg"="yv" %a1) "enc"="{ vex(d, 0, a1, 1, 0, a0, 1, 0) | 01011111 | 11 reg:3=d&7 rm:3=a1&7 ; evexr(d, a1, 1, 0, a0, 1, 0, 0, 0, 0) | 01011111 | 11 reg:3=d&7 rm:3=a1&7 }" "asm"="vmaxps {a1}, {a0}, {d}" "dclass"="yv" "feature"="avx" "nodecode" {
  %r = call <8 x float> @llvm.x86.avx.max.ps.256(<8 x float> %a0, <8 x float> %a1)
  ret <8 x float> %r
}

declare <8 x float> @llvm.x86.avx.max.ps.256(<8 x float>, <8 x float>)

define <4 x double> @X86.avx.min.pd.256(<4 x double> "reg"="yv" %a0, <4 x double> "reg"="yv" %a1) "enc"="{ vex(d, 0, a1, 1, 0, a0, 1, 1) | 01011101 | 11 reg:3=d&7 rm:3=a1&7 ; evexr(d, a1, 1, 1, a0, 1, 1, 0, 0, 0) | 01011101 | 11 reg:3=d&7 rm:3=a1&7 }" "asm"="vminpd {a1}, {a0}, {d}" "dclass"="yv" "feature"="avx" "nodecode" {
  %r = call <4 x double> @llvm.x86.avx.min.pd.256(<4 x double> %a0, <4 x double> %a1)
  ret <4 x double> %r
}

declare <4 x double> @llvm.x86.avx.min.pd.256(<4 x double>, <4 x double>)

define <8 x float> @X86.avx.min.ps.256(<8 x float> "reg"="yv" %a0, <8 x float> "reg"="yv" %a1) "enc"="{ vex(d, 0, a1, 1, 0, a0, 1, 0) | 01011101 | 11 reg:3=d&7 rm:3=a1&7 ; evexr(d, a1, 1, 0, a0, 1, 0, 0, 0, 0) | 01011101 | 11 reg:3=d&7 rm:3=a1&7 }" "asm"="vminps {a1}, {a0}, {d}" "dclass"="yv" "feature"="avx" "nodecode" {
  %r = call <8 x float> @llvm.x86.avx.min.ps.256(<8 x float> %a0, <8 x float> %a1)
  ret <8 x float> %r
}

declare <8 x float> @llvm.x86.avx.min.ps.256(<8 x float>, <8 x float>)

define <8 x float> @X86.avx.rcp.ps.256(<8 x float> "reg"="yv" %a0) "enc"="vex(d, 0, a0, 1, 0, 0, 1, 0) | 01010011 | 11 reg:3=d&7 rm:3=a0&7" "asm"="vrcpps {a0}, {d}" "dclass"="yv" "feature"="avx" "nodecode" {
  %r = call <8 x float> @llvm.x86.avx.rcp.ps.256(<8 x float> %a0)
  ret <8 x float> %r
}

declare <8 x float> @llvm.x86.avx.rcp.ps.256(<8 x float>)

define <4 x double> @X86.avx.round.pd.256(<4 x double> "reg"="yv" %a0, i32 "imm" %a1) "enc"="vex(d, 0, a0, 3, 0, 0, 1, 1) | 00001001 | 11 reg:3=d&7 rm:3=a0&7 | { i1:8=a1 ; i1:s8=a1 }" "asm"="vroundpd ${a1}, {a0}, {d}" "dclass"="yv" "feature"="avx" "nodecode" {
  %r = call <4 x double> @llvm.x86.avx.round.pd.256(<4 x double> %a0, i32 %a1)
  ret <4 x double> %r
}

declare <4 x double> @llvm.x86.avx.round.pd.256(<4 x double>, i32)

define <8 x float> @X86.avx.round.ps.256(<8 x float> "reg"="yv" %a0, i32 "imm" %a1) "enc"="vex(d, 0, a0, 3, 0, 0, 1, 1) | 00001000 | 11 reg:3=d&7 rm:3=a0&7 | { i1:8=a1 ; i1:s8=a1 }" "asm"="vroundps ${a1}, {a0}, {d}" "dclass"="yv" "feature"="avx" "nodecode" {
  %r = call <8 x float> @llvm.x86.avx.round.ps.256(<8 x float> %a0, i32 %a1)
  ret <8 x float> %r
}

declare <8 x float> @llvm.x86.avx.round.ps.256(<8 x float>, i32)

define <8 x float> @X86.avx.rsqrt.ps.256(<8 x float> "reg"="yv" %a0) "enc"="vex(d, 0, a0, 1, 0, 0, 1, 0) | 01010010 | 11 reg:3=d&7 rm:3=a0&7" "asm"="vrsqrtps {a0}, {d}" "dclass"="yv" "feature"="avx" "nodecode" {
  %r = call <8 x float> @llvm.x86.avx.rsqrt.ps.256(<8 x float> %a0)
  ret <8 x float> %r
}

declare <8 x float> @llvm.x86.avx.rsqrt.ps.256(<8 x float>)

define <2 x double> @X86.avx.vpermilvar.pd(<2 x double> "reg"="xv" %a0, <2 x i64> "reg"="xv" %a1) "enc"="{ vex(d, 0, a1, 2, 0, a0, 0, 1) | 00001101 | 11 reg:3=d&7 rm:3=a1&7 ; evexr(d, a1, 2, 1, a0, 0, 1, 0, 0, 0) | 00001101 | 11 reg:3=d&7 rm:3=a1&7 }" "asm"="vpermilpd {a1}, {a0}, {d}" "dclass"="xv" "feature"="avx" "nodecode" {
  %r = call <2 x double> @llvm.x86.avx.vpermilvar.pd(<2 x double> %a0, <2 x i64> %a1)
  ret <2 x double> %r
}

declare <2 x double> @llvm.x86.avx.vpermilvar.pd(<2 x double>, <2 x i64>)

define <4 x double> @X86.avx.vpermilvar.pd.256(<4 x double> "reg"="yv" %a0, <4 x i64> "reg"="yv" %a1) "enc"="{ vex(d, 0, a1, 2, 0, a0, 1, 1) | 00001101 | 11 reg:3=d&7 rm:3=a1&7 ; evexr(d, a1, 2, 1, a0, 1, 1, 0, 0, 0) | 00001101 | 11 reg:3=d&7 rm:3=a1&7 }" "asm"="vpermilpd {a1}, {a0}, {d}" "dclass"="yv" "feature"="avx" "nodecode" {
  %r = call <4 x double> @llvm.x86.avx.vpermilvar.pd.256(<4 x double> %a0, <4 x i64> %a1)
  ret <4 x double> %r
}

declare <4 x double> @llvm.x86.avx.vpermilvar.pd.256(<4 x double>, <4 x i64>)

define <4 x float> @X86.avx.vpermilvar.ps(<4 x float> "reg"="xv" %a0, <4 x i32> "reg"="xv" %a1) "enc"="{ vex(d, 0, a1, 2, 0, a0, 0, 1) | 00001100 | 11 reg:3=d&7 rm:3=a1&7 ; evexr(d, a1, 2, 0, a0, 0, 1, 0, 0, 0) | 00001100 | 11 reg:3=d&7 rm:3=a1&7 }" "asm"="vpermilps {a1}, {a0}, {d}" "dclass"="xv" "feature"="avx" "nodecode" {
  %r = call <4 x float> @llvm.x86.avx.vpermilvar.ps(<4 x float> %a0, <4 x i32> %a1)
  ret <4 x float> %r
}

declare <4 x float> @llvm.x86.avx.vpermilvar.ps(<4 x float>, <4 x i32>)

define <8 x float> @X86.avx.vpermilvar.ps.256(<8 x float> "reg"="yv" %a0, <8 x i32> "reg"="yv" %a1) "enc"="{ vex(d, 0, a1, 2, 0, a0, 1, 1) | 00001100 | 11 reg:3=d&7 rm:3=a1&7 ; evexr(d, a1, 2, 0, a0, 1, 1, 0, 0, 0) | 00001100 | 11 reg:3=d&7 rm:3=a1&7 }" "asm"="vpermilps {a1}, {a0}, {d}" "dclass"="yv" "feature"="avx" "nodecode" {
  %r = call <8 x float> @llvm.x86.avx.vpermilvar.ps.256(<8 x float> %a0, <8 x i32> %a1)
  ret <8 x float> %r
}

declare <8 x float> @llvm.x86.avx.vpermilvar.ps.256(<8 x float>, <8 x i32>)

define void @X86.avx.vzeroall() "enc"="vex(0, 0, 0, 1, 0, 0, 1, 0) | 01110111" "asm"="vzeroall" "feature"="avx" "nodecode" {
  call void @llvm.x86.avx.vzeroall()
  ret void
}

declare void @llvm.x86.avx.vzeroall()

define void @X86.avx.vzeroupper() "enc"="vex(0, 0, 0, 1, 0, 0, 0, 0) | 01110111" "asm"="vzeroupper" "feature"="avx" "nodecode" {
  call void @llvm.x86.avx.vzeroupper()
  ret void
}

declare void @llvm.x86.avx.vzeroupper()

define <16 x i16> @X86.avx2.packssdw(<8 x i32> "reg"="yv" %a0, <8 x i32> "reg"="yv" %a1) "enc"="{ vex(d, 0, a1, 1, 0, a0, 1, 1) | 01101011 | 11 reg:3=d&7 rm:3=a1&7 ; evexr(d, a1, 1, 0, a0, 1, 1, 0, 0, 0) | 01101011 | 11 reg:3=d&7 rm:3=a1&7 }" "asm"="vpackssdw {a1}, {a0}, {d}" "dclass"="yv" "feature"="avx2" "nodecode" {
  %r = call <16 x i16> @llvm.x86.avx2.packssdw(<8 x i32> %a0, <8 x i32> %a1)
  ret <16 x i16> %r
}

declare <16 x i16> @llvm.x86.avx2.packssdw(<8 x i32>, <8 x i32>)

define <32 x i8> @X86.avx2.packsswb(<16 x i16> "reg"="yv" %a0, <16 x i16> "reg"="yv" %a1) "enc"="{ vex(d, 0, a1, 1, 0, a0, 1, 1) | 01100011 | 11 reg:3=d&7 rm:3=a1&7 ; evexr(d, a1, 1, 0, a0, 1, 1, 0, 0, 0) | 01100011 | 11 reg:3=d&7 rm:3=a1&7 }" "asm"="vpacksswb {a1}, {a0}, {d}" "dclass"="yv" "feature"="avx2" "nodecode" {
  %r = call <32 x i8> @llvm.x86.avx2.packsswb(<16 x i16> %a0, <16 x i16> %a1)
  ret <32 x i8> %r
}

declare <32 x i8> @llvm.x86.avx2.packsswb(<16 x i16>, <16 x i16>)

define <16 x i16> @X86.avx2.packusdw(<8 x i32> "reg"="yv" %a0, <8 x i32> "reg"="yv" %a1) "enc"="{ vex(d, 0, a1, 2, 0, a0, 1, 1) | 00101011 | 11 reg:3=d&7 rm:3=a1&7 ; evexr(d, a1, 2, 0, a0, 1, 1, 0, 0, 0) | 00101011 | 11 reg:3=d&7 rm:3=a1&7 }" "asm"="vpackusdw {a1}, {a0}, {d}" "dclass"="yv" "feature"="avx2" "nodecode" {
  %r = call <16 x i16> @llvm.x86.avx2.packusdw(<8 x i32> %a0, <8 x i32> %a1)
  ret <16 x i16> %r
}

declare <16 x i16> @llvm.x86.avx2.packusdw(<8 x i32>, <8 x i32>)

define <32 x i8> @X86.avx2.packuswb(<16 x i16> "reg"="yv" %a0, <16 x i16> "reg"="yv" %a1) "enc"="{ vex(d, 0, a1, 1, 0, a0, 1, 1) | 01100111 | 11 reg:3=d&7 rm:3=a1&7 ; evexr(d, a1, 1, 0, a0, 1, 1, 0, 0, 0) | 01100111 | 11 reg:3=d&7 rm:3=a1&7 }" "asm"="vpackuswb {a1}, {a0}, {d}" "dclass"="yv" "feature"="avx2" "nodecode" {
  %r = call <32 x i8> @llvm.x86.avx2.packuswb(<16 x i16> %a0, <16 x i16> %a1)
  ret <32 x i8> %r
}

declare <32 x i8> @llvm.x86.avx2.packuswb(<16 x i16>, <16 x i16>)

define <8 x i32> @X86.avx2.permd(<8 x i32> "reg"="yv" %a0, <8 x i32> "reg"="yv" %a1) "enc"="{ vex(d, 0, a0, 2, 0, a1, 1, 1) | 00010110 | 11 reg:3=d&7 rm:3=a0&7 ; evexr(d, a0, 2, 0, a1, 1, 1, 0, 0, 0) | 00010110 | 11 reg:3=d&7 rm:3=a0&7 }" "asm"="vpermps {a0}, {a1}, {d}" "dclass"="yv" "feature"="avx2" "nodecode" {
  %r = call <8 x i32> @llvm.x86.avx2.permd(<8 x i32> %a0, <8 x i32> %a1)
  ret <8 x i32> %r
}

declare <8 x i32> @llvm.x86.avx2.permd(<8 x i32>, <8 x i32>)

define <8 x float> @X86.avx2.permps(<8 x float> "reg"="yv" %a0, <8 x i32> "reg"="yv" %a1) "enc"="{ vex(d, 0, a0, 2, 0, a1, 1, 1) | 00010110 | 11 reg:3=d&7 rm:3=a0&7 ; evexr(d, a0, 2, 0, a1, 1, 1, 0, 0, 0) | 00010110 | 11 reg:3=d&7 rm:3=a0&7 }" "asm"="vpermps {a0}, {a1}, {d}" "dclass"="yv" "feature"="avx2" "nodecode" {
  %r = call <8 x float> @llvm.x86.avx2.permps(<8 x float> %a0, <8 x i32> %a1)
  ret <8 x float> %r
}

declare <8 x float> @llvm.x86.avx2.permps(<8 x float>, <8 x i32>)

define <16 x i16> @X86.avx2.pmadd.ub.sw(<32 x i8> "reg"="yv" %a0, <32 x i8> "reg"="yv" %a1) "enc"="{ vex(d, 0, a1, 2, 0, a0, 1, 1) | 00000100 | 11 reg:3=d&7 rm:3=a1&7 ; evexr(d, a1, 2, 0, a0, 1, 1, 0, 0, 0) | 00000100 | 11 reg:3=d&7 rm:3=a1&7 }" "asm"="vpmaddubsw {a1}, {a0}, {d}" "dclass"="yv" "feature"="avx2" "nodecode" {
  %r = call <16 x i16> @llvm.x86.avx2.pmadd.ub.sw(<32 x i8> %a0, <32 x i8> %a1)
  ret <16 x i16> %r
}

declare <16 x i16> @llvm.x86.avx2.pmadd.ub.sw(<32 x i8>, <32 x i8>)

define <8 x i32> @X86.avx2.pmadd.wd(<16 x i16> "reg"="yv" %a0, <16 x i16> "reg"="yv" %a1) "enc"="{ if(a1<8) vex(d, 0, a1, 1, 0, a0, 1, 1) | 11110101 | 11 reg:3=d&7 rm:3=a1&7 ; if(a0<8) vex(d, 0, a0, 1, 0, a1, 1, 1) | 11110101 | 11 reg:3=d&7 rm:3=a0&7 ; vex(d, 0, a1, 1, 0, a0, 1, 1) | 11110101 | 11 reg:3=d&7 rm:3=a1&7 ; evexr(d, a1, 1, 0, a0, 1, 1, 0, 0, 0) | 11110101 | 11 reg:3=d&7 rm:3=a1&7 ; vex(d, 0, a0, 1, 0, a1, 1, 1) | 11110101 | 11 reg:3=d&7 rm:3=a0&7 }" "asm"="vpmaddwd {a1}, {a0}, {d}" "dclass"="yv" "feature"="avx2" "nodecode" {
  %r = call <8 x i32> @llvm.x86.avx2.pmadd.wd(<16 x i16> %a0, <16 x i16> %a1)
  ret <8 x i32> %r
}

declare <8 x i32> @llvm.x86.avx2.pmadd.wd(<16 x i16>, <16 x i16>)

define <16 x i16> @X86.avx2.pmul.hr.sw(<16 x i16> "reg"="yv" %a0, <16 x i16> "reg"="yv" %a1) "enc"="{ vex(d, 0, a1, 2, 0, a0, 1, 1) | 00001011 | 11 reg:3=d&7 rm:3=a1&7 ; evexr(d, a1, 2, 0, a0, 1, 1, 0, 0, 0) | 00001011 | 11 reg:3=d&7 rm:3=a1&7 }" "asm"="vpmulhrsw {a1}, {a0}, {d}" "dclass"="yv" "feature"="avx2" "nodecode" {
  %r = call <16 x i16> @llvm.x86.avx2.pmul.hr.sw(<16 x i16> %a0, <16 x i16> %a1)
  ret <16 x i16> %r
}

declare <16 x i16> @llvm.x86.avx2.pmul.hr.sw(<16 x i16>, <16 x i16>)

define <4 x i64> @X86.avx2.psad.bw(<32 x i8> "reg"="yv" %a0, <32 x i8> "reg"="yv" %a1) "enc"="{ if(a1<8) vex(d, 0, a1, 1, 0, a0, 1, 1) | 11110110 | 11 reg:3=d&7 rm:3=a1&7 ; if(a0<8) vex(d, 0, a0, 1, 0, a1, 1, 1) | 11110110 | 11 reg:3=d&7 rm:3=a0&7 ; vex(d, 0, a1, 1, 0, a0, 1, 1) | 11110110 | 11 reg:3=d&7 rm:3=a1&7 ; evexr(d, a1, 1, 0, a0, 1, 1, 0, 0, 0) | 11110110 | 11 reg:3=d&7 rm:3=a1&7 ; vex(d, 0, a0, 1, 0, a1, 1, 1) | 11110110 | 11 reg:3=d&7 rm:3=a0&7 }" "asm"="vpsadbw {a1}, {a0}, {d}" "dclass"="yv" "feature"="avx2" "nodecode" {
  %r = call <4 x i64> @llvm.x86.avx2.psad.bw(<32 x i8> %a0, <32 x i8> %a1)
  ret <4 x i64> %r
}

declare <4 x i64> @llvm.x86.avx2.psad.bw(<32 x i8>, <32 x i8>)

define <32 x i8> @X86.avx2.pshuf.b(<32 x i8> "reg"="yv" %a0, <32 x i8> "reg"="yv" %a1) "enc"="{ vex(d, 0, a1, 2, 0, a0, 1, 1) | 00000000 | 11 reg:3=d&7 rm:3=a1&7 ; evexr(d, a1, 2, 0, a0, 1, 1, 0, 0, 0) | 00000000 | 11 reg:3=d&7 rm:3=a1&7 }" "asm"="vpshufb {a1}, {a0}, {d}" "dclass"="yv" "feature"="avx2" "nodecode" {
  %r = call <32 x i8> @llvm.x86.avx2.pshuf.b(<32 x i8> %a0, <32 x i8> %a1)
  ret <32 x i8> %r
}

declare <32 x i8> @llvm.x86.avx2.pshuf.b(<32 x i8>, <32 x i8>)

define <32 x i8> @X86.avx2.psign.b(<32 x i8> "reg"="yv" %a0, <32 x i8> "reg"="yv" %a1) "enc"="vex(d, 0, a1, 2, 0, a0, 1, 1) | 00001000 | 11 reg:3=d&7 rm:3=a1&7" "asm"="vpsignb {a1}, {a0}, {d}" "dclass"="yv" "feature"="avx2" "nodecode" {
  %r = call <32 x i8> @llvm.x86.avx2.psign.b(<32 x i8> %a0, <32 x i8> %a1)
  ret <32 x i8> %r
}

declare <32 x i8> @llvm.x86.avx2.psign.b(<32 x i8>, <32 x i8>)

define <8 x i32> @X86.avx2.psign.d(<8 x i32> "reg"="yv" %a0, <8 x i32> "reg"="yv" %a1) "enc"="vex(d, 0, a1, 2, 0, a0, 1, 1) | 00001010 | 11 reg:3=d&7 rm:3=a1&7" "asm"="vpsignd {a1}, {a0}, {d}" "dclass"="yv" "feature"="avx2" "nodecode" {
  %r = call <8 x i32> @llvm.x86.avx2.psign.d(<8 x i32> %a0, <8 x i32> %a1)
  ret <8 x i32> %r
}

declare <8 x i32> @llvm.x86.avx2.psign.d(<8 x i32>, <8 x i32>)

define <16 x i16> @X86.avx2.psign.w(<16 x i16> "reg"="yv" %a0, <16 x i16> "reg"="yv" %a1) "enc"="vex(d, 0, a1, 2, 0, a0, 1, 1) | 00001001 | 11 reg:3=d&7 rm:3=a1&7" "asm"="vpsignw {a1}, {a0}, {d}" "dclass"="yv" "feature"="avx2" "nodecode" {
  %r = call <16 x i16> @llvm.x86.avx2.psign.w(<16 x i16> %a0, <16 x i16> %a1)
  ret <16 x i16> %r
}

declare <16 x i16> @llvm.x86.avx2.psign.w(<16 x i16>, <16 x i16>)

define <8 x i32> @X86.avx2.psll.d(<8 x i32> "reg"="yv" %a0, <4 x i32> "reg"="xv" %a1) "enc"="{ vex(d, 0, a1, 1, 0, a0, 1, 1) | 11110010 | 11 reg:3=d&7 rm:3=a1&7 ; evexr(d, a1, 1, 0, a0, 1, 1, 0, 0, 0) | 11110010 | 11 reg:3=d&7 rm:3=a1&7 }" "asm"="vpslld {a1}, {a0}, {d}" "dclass"="yv" "feature"="avx2" "nodecode" {
  %r = call <8 x i32> @llvm.x86.avx2.psll.d(<8 x i32> %a0, <4 x i32> %a1)
  ret <8 x i32> %r
}

declare <8 x i32> @llvm.x86.avx2.psll.d(<8 x i32>, <4 x i32>)

define <4 x i64> @X86.avx2.psll.q(<4 x i64> "reg"="yv" %a0, <2 x i64> "reg"="xv" %a1) "enc"="{ vex(d, 0, a1, 1, 0, a0, 1, 1) | 11110011 | 11 reg:3=d&7 rm:3=a1&7 ; evexr(d, a1, 1, 1, a0, 1, 1, 0, 0, 0) | 11110011 | 11 reg:3=d&7 rm:3=a1&7 }" "asm"="vpsllq {a1}, {a0}, {d}" "dclass"="yv" "feature"="avx2" "nodecode" {
  %r = call <4 x i64> @llvm.x86.avx2.psll.q(<4 x i64> %a0, <2 x i64> %a1)
  ret <4 x i64> %r
}

declare <4 x i64> @llvm.x86.avx2.psll.q(<4 x i64>, <2 x i64>)

define <16 x i16> @X86.avx2.psll.w(<16 x i16> "reg"="yv" %a0, <8 x i16> "reg"="xv" %a1) "enc"="{ vex(d, 0, a1, 1, 0, a0, 1, 1) | 11110001 | 11 reg:3=d&7 rm:3=a1&7 ; evexr(d, a1, 1, 0, a0, 1, 1, 0, 0, 0) | 11110001 | 11 reg:3=d&7 rm:3=a1&7 }" "asm"="vpsllw {a1}, {a0}, {d}" "dclass"="yv" "feature"="avx2" "nodecode" {
  %r = call <16 x i16> @llvm.x86.avx2.psll.w(<16 x i16> %a0, <8 x i16> %a1)
  ret <16 x i16> %r
}

declare <16 x i16> @llvm.x86.avx2.psll.w(<16 x i16>, <8 x i16>)

define <8 x i32> @X86.avx2.psra.d(<8 x i32> "reg"="yv" %a0, <4 x i32> "reg"="xv" %a1) "enc"="{ vex(d, 0, a1, 1, 0, a0, 1, 1) | 11100010 | 11 reg:3=d&7 rm:3=a1&7 ; evexr(d, a1, 1, 0, a0, 1, 1, 0, 0, 0) | 11100010 | 11 reg:3=d&7 rm:3=a1&7 }" "asm"="vpsrad {a1}, {a0}, {d}" "dclass"="yv" "feature"="avx2" "nodecode" {
  %r = call <8 x i32> @llvm.x86.avx2.psra.d(<8 x i32> %a0, <4 x i32> %a1)
  ret <8 x i32> %r
}

declare <8 x i32> @llvm.x86.avx2.psra.d(<8 x i32>, <4 x i32>)

define <16 x i16> @X86.avx2.psra.w(<16 x i16> "reg"="yv" %a0, <8 x i16> "reg"="xv" %a1) "enc"="{ vex(d, 0, a1, 1, 0, a0, 1, 1) | 11100001 | 11 reg:3=d&7 rm:3=a1&7 ; evexr(d, a1, 1, 0, a0, 1, 1, 0, 0, 0) | 11100001 | 11 reg:3=d&7 rm:3=a1&7 }" "asm"="vpsraw {a1}, {a0}, {d}" "dclass"="yv" "feature"="avx2" "nodecode" {
  %r = call <16 x i16> @llvm.x86.avx2.psra.w(<16 x i16> %a0, <8 x i16> %a1)
  ret <16 x i16> %r
}

declare <16 x i16> @llvm.x86.avx2.psra.w(<16 x i16>, <8 x i16>)

define <8 x i32> @X86.avx2.psrl.d(<8 x i32> "reg"="yv" %a0, <4 x i32> "reg"="xv" %a1) "enc"="{ vex(d, 0, a1, 1, 0, a0, 1, 1) | 11010010 | 11 reg:3=d&7 rm:3=a1&7 ; evexr(d, a1, 1, 0, a0, 1, 1, 0, 0, 0) | 11010010 | 11 reg:3=d&7 rm:3=a1&7 }" "asm"="vpsrld {a1}, {a0}, {d}" "dclass"="yv" "feature"="avx2" "nodecode" {
  %r = call <8 x i32> @llvm.x86.avx2.psrl.d(<8 x i32> %a0, <4 x i32> %a1)
  ret <8 x i32> %r
}

declare <8 x i32> @llvm.x86.avx2.psrl.d(<8 x i32>, <4 x i32>)

define <4 x i64> @X86.avx2.psrl.q(<4 x i64> "reg"="yv" %a0, <2 x i64> "reg"="xv" %a1) "enc"="{ vex(d, 0, a1, 1, 0, a0, 1, 1) | 11010011 | 11 reg:3=d&7 rm:3=a1&7 ; evexr(d, a1, 1, 1, a0, 1, 1, 0, 0, 0) | 11010011 | 11 reg:3=d&7 rm:3=a1&7 }" "asm"="vpsrlq {a1}, {a0}, {d}" "dclass"="yv" "feature"="avx2" "nodecode" {
  %r = call <4 x i64> @llvm.x86.avx2.psrl.q(<4 x i64> %a0, <2 x i64> %a1)
  ret <4 x i64> %r
}

declare <4 x i64> @llvm.x86.avx2.psrl.q(<4 x i64>, <2 x i64>)

define <16 x i16> @X86.avx2.psrl.w(<16 x i16> "reg"="yv" %a0, <8 x i16> "reg"="xv" %a1) "enc"="{ vex(d, 0, a1, 1, 0, a0, 1, 1) | 11010001 | 11 reg:3=d&7 rm:3=a1&7 ; evexr(d, a1, 1, 0, a0, 1, 1, 0, 0, 0) | 11010001 | 11 reg:3=d&7 rm:3=a1&7 }" "asm"="vpsrlw {a1}, {a0}, {d}" "dclass"="yv" "feature"="avx2" "nodecode" {
  %r = call <16 x i16> @llvm.x86.avx2.psrl.w(<16 x i16> %a0, <8 x i16> %a1)
  ret <16 x i16> %r
}

declare <16 x i16> @llvm.x86.avx2.psrl.w(<16 x i16>, <8 x i16>)

define <8 x double> @X86.avx512.add.pd.512(<8 x double> "reg"="zv" %a0, <8 x double> "reg"="zv" %a1) "enc"="evexr(d, a1, 1, 1, a0, 2, 1, 0, 0, 0) | 01011000 | 11 reg:3=d&7 rm:3=a1&7" "asm"="vaddpd {a1}, {a0}, {d}" "dclass"="zv" "feature"="avx512f" "nodecode" {
  %r = call <8 x double> @llvm.x86.avx512.add.pd.512(<8 x double> %a0, <8 x double> %a1, i32 4)
  ret <8 x double> %r
}

declare <8 x double> @llvm.x86.avx512.add.pd.512(<8 x double>, <8 x double>, i32)

define <16 x float> @X86.avx512.add.ps.512(<16 x float> "reg"="zv" %a0, <16 x float> "reg"="zv" %a1) "enc"="evexr(d, a1, 1, 0, a0, 2, 0, 0, 0, 0) | 01011000 | 11 reg:3=d&7 rm:3=a1&7" "asm"="vaddps {a1}, {a0}, {d}" "dclass"="zv" "feature"="avx512f" "nodecode" {
  %r = call <16 x float> @llvm.x86.avx512.add.ps.512(<16 x float> %a0, <16 x float> %a1, i32 4)
  ret <16 x float> %r
}

declare <16 x float> @llvm.x86.avx512.add.ps.512(<16 x float>, <16 x float>, i32)

define <4 x i32> @X86.avx512.conflict.d.128(<4 x i32> "reg"="xv" %a0) "enc"="evexr(d, a0, 2, 0, 0, 0, 1, 0, 0, 0) | 11000100 | 11 reg:3=d&7 rm:3=a0&7" "asm"="vpconflictd {a0}, {d}" "dclass"="xv" "feature"="avx512cd,avx512vl" "nodecode" {
  %r = call <4 x i32> @llvm.x86.avx512.conflict.d.128(<4 x i32> %a0)
  ret <4 x i32> %r
}

declare <4 x i32> @llvm.x86.avx512.conflict.d.128(<4 x i32>)

define <8 x i32> @X86.avx512.conflict.d.256(<8 x i32> "reg"="yv" %a0) "enc"="evexr(d, a0, 2, 0, 0, 1, 1, 0, 0, 0) | 11000100 | 11 reg:3=d&7 rm:3=a0&7" "asm"="vpconflictd {a0}, {d}" "dclass"="yv" "feature"="avx512cd,avx512vl" "nodecode" {
  %r = call <8 x i32> @llvm.x86.avx512.conflict.d.256(<8 x i32> %a0)
  ret <8 x i32> %r
}

declare <8 x i32> @llvm.x86.avx512.conflict.d.256(<8 x i32>)

define <16 x i32> @X86.avx512.conflict.d.512(<16 x i32> "reg"="zv" %a0) "enc"="evexr(d, a0, 2, 0, 0, 2, 1, 0, 0, 0) | 11000100 | 11 reg:3=d&7 rm:3=a0&7" "asm"="vpconflictd {a0}, {d}" "dclass"="zv" "feature"="avx512cd" "nodecode" {
  %r = call <16 x i32> @llvm.x86.avx512.conflict.d.512(<16 x i32> %a0)
  ret <16 x i32> %r
}

declare <16 x i32> @llvm.x86.avx512.conflict.d.512(<16 x i32>)

define <2 x i64> @X86.avx512.conflict.q.128(<2 x i64> "reg"="xv" %a0) "enc"="evexr(d, a0, 2, 1, 0, 0, 1, 0, 0, 0) | 11000100 | 11 reg:3=d&7 rm:3=a0&7" "asm"="vpconflictq {a0}, {d}" "dclass"="xv" "feature"="avx512cd,avx512vl" "nodecode" {
  %r = call <2 x i64> @llvm.x86.avx512.conflict.q.128(<2 x i64> %a0)
  ret <2 x i64> %r
}

declare <2 x i64> @llvm.x86.avx512.conflict.q.128(<2 x i64>)

define <4 x i64> @X86.avx512.conflict.q.256(<4 x i64> "reg"="yv" %a0) "enc"="evexr(d, a0, 2, 1, 0, 1, 1, 0, 0, 0) | 11000100 | 11 reg:3=d&7 rm:3=a0&7" "asm"="vpconflictq {a0}, {d}" "dclass"="yv" "feature"="avx512cd,avx512vl" "nodecode" {
  %r = call <4 x i64> @llvm.x86.avx512.conflict.q.256(<4 x i64> %a0)
  ret <4 x i64> %r
}

declare <4 x i64> @llvm.x86.avx512.conflict.q.256(<4 x i64>)

define <8 x i64> @X86.avx512.conflict.q.512(<8 x i64> "reg"="zv" %a0) "enc"="evexr(d, a0, 2, 1, 0, 2, 1, 0, 0, 0) | 11000100 | 11 reg:3=d&7 rm:3=a0&7" "asm"="vpconflictq {a0}, {d}" "dclass"="zv" "feature"="avx512cd" "nodecode" {
  %r = call <8 x i64> @llvm.x86.avx512.conflict.q.512(<8 x i64> %a0)
  ret <8 x i64> %r
}

declare <8 x i64> @llvm.x86.avx512.conflict.q.512(<8 x i64>)

define <2 x double> @X86.avx512.cvtsi2sd64(<2 x double> "reg"="xv" %a0, i64 "reg"="r64" %a1) "enc"="{ vex(d, 0, a1, 1, 1, a0, 0, 3) | 00101010 | 11 reg:3=d&7 rm:3=a1&7 ; evexr(d, a1, 1, 1, a0, 0, 3, 0, 0, 0) | 00101010 | 11 reg:3=d&7 rm:3=a1&7 }" "asm"="vcvtsi2sd {a1}, {a0}, {d}" "dclass"="xv" "feature"="avx512f" "nodecode" {
  %r = call <2 x double> @llvm.x86.avx512.cvtsi2sd64(<2 x double> %a0, i64 %a1, i32 4)
  ret <2 x double> %r
}

declare <2 x double> @llvm.x86.avx512.cvtsi2sd64(<2 x double>, i64, i32)

define <4 x float> @X86.avx512.cvtsi2ss32(<4 x float> "reg"="xv" %a0, i32 "reg"="r32" %a1) "enc"="{ vex(d, 0, a1, 1, 0, a0, 0, 2) | 00101010 | 11 reg:3=d&7 rm:3=a1&7 ; evexr(d, a1, 1, 0, a0, 0, 2, 0, 0, 0) | 00101010 | 11 reg:3=d&7 rm:3=a1&7 }" "asm"="vcvtsi2ss {a1}, {a0}, {d}" "dclass"="xv" "feature"="avx512f" "nodecode" {
  %r = call <4 x float> @llvm.x86.avx512.cvtsi2ss32(<4 x float> %a0, i32 %a1, i32 4)
  ret <4 x float> %r
}

declare <4 x float> @llvm.x86.avx512.cvtsi2ss32(<4 x float>, i32, i32)

define <4 x float> @X86.avx512.cvtsi2ss64(<4 x float> "reg"="xv" %a0, i64 "reg"="r64" %a1) "enc"="{ vex(d, 0, a1, 1, 1, a0, 0, 2) | 00101010 | 11 reg:3=d&7 rm:3=a1&7 ; evexr(d, a1, 1, 1, a0, 0, 2, 0, 0, 0) | 00101010 | 11 reg:3=d&7 rm:3=a1&7 }" "asm"="vcvtsi2ss {a1}, {a0}, {d}" "dclass"="xv" "feature"="avx512f" "nodecode" {
  %r = call <4 x float> @llvm.x86.avx512.cvtsi2ss64(<4 x float> %a0, i64 %a1, i32 4)
  ret <4 x float> %r
}

declare <4 x float> @llvm.x86.avx512.cvtsi2ss64(<4 x float>, i64, i32)

define i32 @X86.avx512.cvttsd2si(<2 x double> "reg"="xv" %a0) "enc"="{ vex(d, 0, a0, 1, 0, 0, 0, 3) | 00101100 | 11 reg:3=d&7 rm:3=a0&7 ; evexr(d, a0, 1, 0, 0, 0, 3, 0, 0, 0) | 00101100 | 11 reg:3=d&7 rm:3=a0&7 }" "asm"="vcvttsd2si {a0}, {d}" "dclass"="r32" "feature"="avx512f" "nodecode" {
  %r = call i32 @llvm.x86.avx512.cvttsd2si(<2 x double> %a0, i32 4)
  ret i32 %r
}

declare i32 @llvm.x86.avx512.cvttsd2si(<2 x double>, i32)

define i64 @X86.avx512.cvttsd2si64(<2 x double> "reg"="xv" %a0) "enc"="{ vex(d, 0, a0, 1, 1, 0, 0, 3) | 00101100 | 11 reg:3=d&7 rm:3=a0&7 ; evexr(d, a0, 1, 1, 0, 0, 3, 0, 0, 0) | 00101100 | 11 reg:3=d&7 rm:3=a0&7 }" "asm"="vcvttsd2si {a0}, {d}" "dclass"="r64" "feature"="avx512f" "nodecode" {
  %r = call i64 @llvm.x86.avx512.cvttsd2si64(<2 x double> %a0, i32 4)
  ret i64 %r
}

declare i64 @llvm.x86.avx512.cvttsd2si64(<2 x double>, i32)

define i32 @X86.avx512.cvttsd2usi(<2 x double> "reg"="xv" %a0) "enc"="evexr(d, a0, 1, 0, 0, 0, 3, 0, 0, 0) | 01111000 | 11 reg:3=d&7 rm:3=a0&7" "asm"="vcvttsd2usi {a0}, {d}" "dclass"="r32" "feature"="avx512f" "nodecode" {
  %r = call i32 @llvm.x86.avx512.cvttsd2usi(<2 x double> %a0, i32 4)
  ret i32 %r
}

declare i32 @llvm.x86.avx512.cvttsd2usi(<2 x double>, i32)

define i64 @X86.avx512.cvttsd2usi64(<2 x double> "reg"="xv" %a0) "enc"="evexr(d, a0, 1, 1, 0, 0, 3, 0, 0, 0) | 01111000 | 11 reg:3=d&7 rm:3=a0&7" "asm"="vcvttsd2usi {a0}, {d}" "dclass"="r64" "feature"="avx512f" "nodecode" {
  %r = call i64 @llvm.x86.avx512.cvttsd2usi64(<2 x double> %a0, i32 4)
  ret i64 %r
}

declare i64 @llvm.x86.avx512.cvttsd2usi64(<2 x double>, i32)

define i32 @X86.avx512.cvttss2si(<4 x float> "reg"="xv" %a0) "enc"="{ vex(d, 0, a0, 1, 0, 0, 0, 2) | 00101100 | 11 reg:3=d&7 rm:3=a0&7 ; evexr(d, a0, 1, 0, 0, 0, 2, 0, 0, 0) | 00101100 | 11 reg:3=d&7 rm:3=a0&7 }" "asm"="vcvttss2si {a0}, {d}" "dclass"="r32" "feature"="avx512f" "nodecode" {
  %r = call i32 @llvm.x86.avx512.cvttss2si(<4 x float> %a0, i32 4)
  ret i32 %r
}

declare i32 @llvm.x86.avx512.cvttss2si(<4 x float>, i32)

define i64 @X86.avx512.cvttss2si64(<4 x float> "reg"="xv" %a0) "enc"="{ vex(d, 0, a0, 1, 1, 0, 0, 2) | 00101100 | 11 reg:3=d&7 rm:3=a0&7 ; evexr(d, a0, 1, 1, 0, 0, 2, 0, 0, 0) | 00101100 | 11 reg:3=d&7 rm:3=a0&7 }" "asm"="vcvttss2si {a0}, {d}" "dclass"="r64" "feature"="avx512f" "nodecode" {
  %r = call i64 @llvm.x86.avx512.cvttss2si64(<4 x float> %a0, i32 4)
  ret i64 %r
}

declare i64 @llvm.x86.avx512.cvttss2si64(<4 x float>, i32)

define i32 @X86.avx512.cvttss2usi(<4 x float> "reg"="xv" %a0) "enc"="evexr(d, a0, 1, 0, 0, 0, 2, 0, 0, 0) | 01111000 | 11 reg:3=d&7 rm:3=a0&7" "asm"="vcvttss2usi {a0}, {d}" "dclass"="r32" "feature"="avx512f" "nodecode" {
  %r = call i32 @llvm.x86.avx512.cvttss2usi(<4 x float> %a0, i32 4)
  ret i32 %r
}

declare i32 @llvm.x86.avx512.cvttss2usi(<4 x float>, i32)

define i64 @X86.avx512.cvttss2usi64(<4 x float> "reg"="xv" %a0) "enc"="evexr(d, a0, 1, 1, 0, 0, 2, 0, 0, 0) | 01111000 | 11 reg:3=d&7 rm:3=a0&7" "asm"="vcvttss2usi {a0}, {d}" "dclass"="r64" "feature"="avx512f" "nodecode" {
  %r = call i64 @llvm.x86.avx512.cvttss2usi64(<4 x float> %a0, i32 4)
  ret i64 %r
}

declare i64 @llvm.x86.avx512.cvttss2usi64(<4 x float>, i32)

define <4 x float> @X86.avx512.cvtusi2ss(<4 x float> "reg"="xv" %a0, i32 "reg"="r32" %a1) "enc"="evexr(d, a1, 1, 0, a0, 0, 2, 0, 0, 0) | 01111011 | 11 reg:3=d&7 rm:3=a1&7" "asm"="vcvtusi2ss {a1}, {a0}, {d}" "dclass"="xv" "feature"="avx512f" "nodecode" {
  %r = call <4 x float> @llvm.x86.avx512.cvtusi2ss(<4 x float> %a0, i32 %a1, i32 4)
  ret <4 x float> %r
}

declare <4 x float> @llvm.x86.avx512.cvtusi2ss(<4 x float>, i32, i32)

define <2 x double> @X86.avx512.cvtusi642sd(<2 x double> "reg"="xv" %a0, i64 "reg"="r64" %a1) "enc"="evexr(d, a1, 1, 1, a0, 0, 3, 0, 0, 0) | 01111011 | 11 reg:3=d&7 rm:3=a1&7" "asm"="vcvtusi2sd {a1}, {a0}, {d}" "dclass"="xv" "feature"="avx512f" "nodecode" {
  %r = call <2 x double> @llvm.x86.avx512.cvtusi642sd(<2 x double> %a0, i64 %a1, i32 4)
  ret <2 x double> %r
}

declare <2 x double> @llvm.x86.avx512.cvtusi642sd(<2 x double>, i64, i32)

define <4 x float> @X86.avx512.cvtusi642ss(<4 x float> "reg"="xv" %a0, i64 "reg"="r64" %a1) "enc"="evexr(d, a1, 1, 1, a0, 0, 2, 0, 0, 0) | 01111011 | 11 reg:3=d&7 rm:3=a1&7" "asm"="vcvtusi2ss {a1}, {a0}, {d}" "dclass"="xv" "feature"="avx512f" "nodecode" {
  %r = call <4 x float> @llvm.x86.avx512.cvtusi642ss(<4 x float> %a0, i64 %a1, i32 4)
  ret <4 x float> %r
}

declare <4 x float> @llvm.x86.avx512.cvtusi642ss(<4 x float>, i64, i32)

define <8 x i16> @X86.avx512.dbpsadbw.128(<16 x i8> "reg"="xv" %a0, <16 x i8> "reg"="xv" %a1, i32 "imm" %a2) "enc"="evexr(d, a1, 3, 0, a0, 0, 1, 0, 0, 0) | 01000010 | 11 reg:3=d&7 rm:3=a1&7 | { i1:8=a2 ; i1:s8=a2 }" "asm"="vdbpsadbw ${a2}, {a1}, {a0}, {d}" "dclass"="xv" "feature"="avx512bw,avx512vl" "nodecode" {
  %r = call <8 x i16> @llvm.x86.avx512.dbpsadbw.128(<16 x i8> %a0, <16 x i8> %a1, i32 %a2)
  ret <8 x i16> %r
}

declare <8 x i16> @llvm.x86.avx512.dbpsadbw.128(<16 x i8>, <16 x i8>, i32)

define <16 x i16> @X86.avx512.dbpsadbw.256(<32 x i8> "reg"="yv" %a0, <32 x i8> "reg"="yv" %a1, i32 "imm" %a2) "enc"="evexr(d, a1, 3, 0, a0, 1, 1, 0, 0, 0) | 01000010 | 11 reg:3=d&7 rm:3=a1&7 | { i1:8=a2 ; i1:s8=a2 }" "asm"="vdbpsadbw ${a2}, {a1}, {a0}, {d}" "dclass"="yv" "feature"="avx512bw,avx512vl" "nodecode" {
  %r = call <16 x i16> @llvm.x86.avx512.dbpsadbw.256(<32 x i8> %a0, <32 x i8> %a1, i32 %a2)
  ret <16 x i16> %r
}

declare <16 x i16> @llvm.x86.avx512.dbpsadbw.256(<32 x i8>, <32 x i8>, i32)

define <32 x i16> @X86.avx512.dbpsadbw.512(<64 x i8> "reg"="zv" %a0, <64 x i8> "reg"="zv" %a1, i32 "imm" %a2) "enc"="evexr(d, a1, 3, 0, a0, 2, 1, 0, 0, 0) | 01000010 | 11 reg:3=d&7 rm:3=a1&7 | { i1:8=a2 ; i1:s8=a2 }" "asm"="vdbpsadbw ${a2}, {a1}, {a0}, {d}" "dclass"="zv" "feature"="avx512bw" "nodecode" {
  %r = call <32 x i16> @llvm.x86.avx512.dbpsadbw.512(<64 x i8> %a0, <64 x i8> %a1, i32 %a2)
  ret <32 x i16> %r
}

declare <32 x i16> @llvm.x86.avx512.dbpsadbw.512(<64 x i8>, <64 x i8>, i32)

define <8 x double> @X86.avx512.div.pd.512(<8 x double> "reg"="zv" %a0, <8 x double> "reg"="zv" %a1) "enc"="evexr(d, a1, 1, 1, a0, 2, 1, 0, 0, 0) | 01011110 | 11 reg:3=d&7 rm:3=a1&7" "asm"="vdivpd {a1}, {a0}, {d}" "dclass"="zv" "feature"="avx512f" "nodecode" {
  %r = call <8 x double> @llvm.x86.avx512.div.pd.512(<8 x double> %a0, <8 x double> %a1, i32 4)
  ret <8 x double> %r
}

declare <8 x double> @llvm.x86.avx512.div.pd.512(<8 x double>, <8 x double>, i32)

define <16 x float> @X86.avx512.div.ps.512(<16 x float> "reg"="zv" %a0, <16 x float> "reg"="zv" %a1) "enc"="evexr(d, a1, 1, 0, a0, 2, 0, 0, 0, 0) | 01011110 | 11 reg:3=d&7 rm:3=a1&7" "asm"="vdivps {a1}, {a0}, {d}" "dclass"="zv" "feature"="avx512f" "nodecode" {
  %r = call <16 x float> @llvm.x86.avx512.div.ps.512(<16 x float> %a0, <16 x float> %a1, i32 4)
  ret <16 x float> %r
}

declare <16 x float> @llvm.x86.avx512.div.ps.512(<16 x float>, <16 x float>, i32)

define <2 x double> @X86.avx512.mask.add.sd.round(<2 x double> "reg"="xv" %a0, <2 x double> "reg"="xv" %a1, <2 x double> "reg"="xv" "tied" %a2, i8 "reg"="k" %a3) "enc"="{ if(a3!=0) evexr(d, a1, 1, 1, a0, 0, 3, 0, a3, 0) | 01011000 | 11 reg:3=d&7 rm:3=a1&7 }" "asm"="vaddsd {a1}, {a0}, {d} {lb}{a3}{rb}" "dclass"="xv" "feature"="avx512f" "nodecode" {
  %r = call <2 x double> @llvm.x86.avx512.mask.add.sd.round(<2 x double> %a0, <2 x double> %a1, <2 x double> %a2, i8 %a3, i32 4)
  ret <2 x double> %r
}

declare <2 x double> @llvm.x86.avx512.mask.add.sd.round(<2 x double>, <2 x double>, <2 x double>, i8, i32)

define <4 x float> @X86.avx512.mask.add.ss.round(<4 x float> "reg"="xv" %a0, <4 x float> "reg"="xv" %a1, <4 x float> "reg"="xv" "tied" %a2, i8 "reg"="k" %a3) "enc"="{ if(a3!=0) evexr(d, a1, 1, 0, a0, 0, 2, 0, a3, 0) | 01011000 | 11 reg:3=d&7 rm:3=a1&7 }" "asm"="vaddss {a1}, {a0}, {d} {lb}{a3}{rb}" "dclass"="xv" "feature"="avx512f" "nodecode" {
  %r = call <4 x float> @llvm.x86.avx512.mask.add.ss.round(<4 x float> %a0, <4 x float> %a1, <4 x float> %a2, i8 %a3, i32 4)
  ret <4 x float> %r
}

declare <4 x float> @llvm.x86.avx512.mask.add.ss.round(<4 x float>, <4 x float>, <4 x float>, i8, i32)

define <16 x i8> @X86.avx512.mask.compress.b.128(<16 x i8> "reg"="xv" %a0, <16 x i8> "reg"="xv" "tied" %a1, i16 "reg"="k" %a2) "enc"="{ if(a2!=0) evexr(a0, d, 2, 0, 0, 0, 1, 0, a2, 0) | 01100011 | 11 reg:3=a0&7 rm:3=d&7 }" "asm"="vpcompressb {a0}, {d} {lb}{a2}{rb}" "dclass"="xv" "feature"="avx512vbmi2,avx512vl" "nodecode" {
  %r = call <16 x i8> @llvm.x86.avx512.mask.compress.b.128(<16 x i8> %a0, <16 x i8> %a1, i16 %a2)
  ret <16 x i8> %r
}

declare <16 x i8> @llvm.x86.avx512.mask.compress.b.128(<16 x i8>, <16 x i8>, i16)

define <32 x i8> @X86.avx512.mask.compress.b.256(<32 x i8> "reg"="yv" %a0, <32 x i8> "reg"="yv" "tied" %a1, i32 "reg"="k" %a2) "enc"="{ if(a2!=0) evexr(a0, d, 2, 0, 0, 1, 1, 0, a2, 0) | 01100011 | 11 reg:3=a0&7 rm:3=d&7 }" "asm"="vpcompressb {a0}, {d} {lb}{a2}{rb}" "dclass"="yv" "feature"="avx512vbmi2,avx512vl" "nodecode" {
  %r = call <32 x i8> @llvm.x86.avx512.mask.compress.b.256(<32 x i8> %a0, <32 x i8> %a1, i32 %a2)
  ret <32 x i8> %r
}

declare <32 x i8> @llvm.x86.avx512.mask.compress.b.256(<32 x i8>, <32 x i8>, i32)

define <64 x i8> @X86.avx512.mask.compress.b.512(<64 x i8> "reg"="zv" %a0, <64 x i8> "reg"="zv" "tied" %a1, i64 "reg"="k" %a2) "enc"="{ if(a2!=0) evexr(a0, d, 2, 0, 0, 2, 1, 0, a2, 0) | 01100011 | 11 reg:3=a0&7 rm:3=d&7 }" "asm"="vpcompressb {a0}, {d} {lb}{a2}{rb}" "dclass"="zv" "feature"="avx512vbmi2" "nodecode" {
  %r = call <64 x i8> @llvm.x86.avx512.mask.compress.b.512(<64 x i8> %a0, <64 x i8> %a1, i64 %a2)
  ret <64 x i8> %r
}

declare <64 x i8> @llvm.x86.avx512.mask.compress.b.512(<64 x i8>, <64 x i8>, i64)

define <4 x i32> @X86.avx512.mask.compress.d.128(<4 x i32> "reg"="xv" %a0, <4 x i32> "reg"="xv" "tied" %a1, i8 "reg"="k" %a2) "enc"="{ if(a2!=0) evexr(a0, d, 2, 0, 0, 0, 1, 0, a2, 0) | 10001011 | 11 reg:3=a0&7 rm:3=d&7 }" "asm"="vpcompressd {a0}, {d} {lb}{a2}{rb}" "dclass"="xv" "feature"="avx512f,avx512vl" "nodecode" {
  %r = call <4 x i32> @llvm.x86.avx512.mask.compress.d.128(<4 x i32> %a0, <4 x i32> %a1, i8 %a2)
  ret <4 x i32> %r
}

declare <4 x i32> @llvm.x86.avx512.mask.compress.d.128(<4 x i32>, <4 x i32>, i8)

define <8 x i32> @X86.avx512.mask.compress.d.256(<8 x i32> "reg"="yv" %a0, <8 x i32> "reg"="yv" "tied" %a1, i8 "reg"="k" %a2) "enc"="{ if(a2!=0) evexr(a0, d, 2, 0, 0, 1, 1, 0, a2, 0) | 10001011 | 11 reg:3=a0&7 rm:3=d&7 }" "asm"="vpcompressd {a0}, {d} {lb}{a2}{rb}" "dclass"="yv" "feature"="avx512f,avx512vl" "nodecode" {
  %r = call <8 x i32> @llvm.x86.avx512.mask.compress.d.256(<8 x i32> %a0, <8 x i32> %a1, i8 %a2)
  ret <8 x i32> %r
}

declare <8 x i32> @llvm.x86.avx512.mask.compress.d.256(<8 x i32>, <8 x i32>, i8)

define <16 x i32> @X86.avx512.mask.compress.d.512(<16 x i32> "reg"="zv" %a0, <16 x i32> "reg"="zv" "tied" %a1, i16 "reg"="k" %a2) "enc"="{ if(a2!=0) evexr(a0, d, 2, 0, 0, 2, 1, 0, a2, 0) | 10001011 | 11 reg:3=a0&7 rm:3=d&7 }" "asm"="vpcompressd {a0}, {d} {lb}{a2}{rb}" "dclass"="zv" "feature"="avx512f" "nodecode" {
  %r = call <16 x i32> @llvm.x86.avx512.mask.compress.d.512(<16 x i32> %a0, <16 x i32> %a1, i16 %a2)
  ret <16 x i32> %r
}

declare <16 x i32> @llvm.x86.avx512.mask.compress.d.512(<16 x i32>, <16 x i32>, i16)

define <2 x double> @X86.avx512.mask.compress.pd.128(<2 x double> "reg"="xv" %a0, <2 x double> "reg"="xv" "tied" %a1, i8 "reg"="k" %a2) "enc"="{ if(a2!=0) evexr(a0, d, 2, 1, 0, 0, 1, 0, a2, 0) | 10001010 | 11 reg:3=a0&7 rm:3=d&7 }" "asm"="vcompresspd {a0}, {d} {lb}{a2}{rb}" "dclass"="xv" "feature"="avx512f,avx512vl" "nodecode" {
  %r = call <2 x double> @llvm.x86.avx512.mask.compress.pd.128(<2 x double> %a0, <2 x double> %a1, i8 %a2)
  ret <2 x double> %r
}

declare <2 x double> @llvm.x86.avx512.mask.compress.pd.128(<2 x double>, <2 x double>, i8)

define <4 x double> @X86.avx512.mask.compress.pd.256(<4 x double> "reg"="yv" %a0, <4 x double> "reg"="yv" "tied" %a1, i8 "reg"="k" %a2) "enc"="{ if(a2!=0) evexr(a0, d, 2, 1, 0, 1, 1, 0, a2, 0) | 10001010 | 11 reg:3=a0&7 rm:3=d&7 }" "asm"="vcompresspd {a0}, {d} {lb}{a2}{rb}" "dclass"="yv" "feature"="avx512f,avx512vl" "nodecode" {
  %r = call <4 x double> @llvm.x86.avx512.mask.compress.pd.256(<4 x double> %a0, <4 x double> %a1, i8 %a2)
  ret <4 x double> %r
}

declare <4 x double> @llvm.x86.avx512.mask.compress.pd.256(<4 x double>, <4 x double>, i8)

define <8 x double> @X86.avx512.mask.compress.pd.512(<8 x double> "reg"="zv" %a0, <8 x double> "reg"="zv" "tied" %a1, i8 "reg"="k" %a2) "enc"="{ if(a2!=0) evexr(a0, d, 2, 1, 0, 2, 1, 0, a2, 0) | 10001010 | 11 reg:3=a0&7 rm:3=d&7 }" "asm"="vcompresspd {a0}, {d} {lb}{a2}{rb}" "dclass"="zv" "feature"="avx512f" "nodecode" {
  %r = call <8 x double> @llvm.x86.avx512.mask.compress.pd.512(<8 x double> %a0, <8 x double> %a1, i8 %a2)
  ret <8 x double> %r
}

declare <8 x double> @llvm.x86.avx512.mask.compress.pd.512(<8 x double>, <8 x double>, i8)

define <4 x float> @X86.avx512.mask.compress.ps.128(<4 x float> "reg"="xv" %a0, <4 x float> "reg"="xv" "tied" %a1, i8 "reg"="k" %a2) "enc"="{ if(a2!=0) evexr(a0, d, 2, 0, 0, 0, 1, 0, a2, 0) | 10001010 | 11 reg:3=a0&7 rm:3=d&7 }" "asm"="vcompressps {a0}, {d} {lb}{a2}{rb}" "dclass"="xv" "feature"="avx512f,avx512vl" "nodecode" {
  %r = call <4 x float> @llvm.x86.avx512.mask.compress.ps.128(<4 x float> %a0, <4 x float> %a1, i8 %a2)
  ret <4 x float> %r
}

declare <4 x float> @llvm.x86.avx512.mask.compress.ps.128(<4 x float>, <4 x float>, i8)

define <8 x float> @X86.avx512.mask.compress.ps.256(<8 x float> "reg"="yv" %a0, <8 x float> "reg"="yv" "tied" %a1, i8 "reg"="k" %a2) "enc"="{ if(a2!=0) evexr(a0, d, 2, 0, 0, 1, 1, 0, a2, 0) | 10001010 | 11 reg:3=a0&7 rm:3=d&7 }" "asm"="vcompressps {a0}, {d} {lb}{a2}{rb}" "dclass"="yv" "feature"="avx512f,avx512vl" "nodecode" {
  %r = call <8 x float> @llvm.x86.avx512.mask.compress.ps.256(<8 x float> %a0, <8 x float> %a1, i8 %a2)
  ret <8 x float> %r
}

declare <8 x float> @llvm.x86.avx512.mask.compress.ps.256(<8 x float>, <8 x float>, i8)

define <16 x float> @X86.avx512.mask.compress.ps.512(<16 x float> "reg"="zv" %a0, <16 x float> "reg"="zv" "tied" %a1, i16 "reg"="k" %a2) "enc"="{ if(a2!=0) evexr(a0, d, 2, 0, 0, 2, 1, 0, a2, 0) | 10001010 | 11 reg:3=a0&7 rm:3=d&7 }" "asm"="vcompressps {a0}, {d} {lb}{a2}{rb}" "dclass"="zv" "feature"="avx512f" "nodecode" {
  %r = call <16 x float> @llvm.x86.avx512.mask.compress.ps.512(<16 x float> %a0, <16 x float> %a1, i16 %a2)
  ret <16 x float> %r
}

declare <16 x float> @llvm.x86.avx512.mask.compress.ps.512(<16 x float>, <16 x float>, i16)

define <2 x i64> @X86.avx512.mask.compress.q.128(<2 x i64> "reg"="xv" %a0, <2 x i64> "reg"="xv" "tied" %a1, i8 "reg"="k" %a2) "enc"="{ if(a2!=0) evexr(a0, d, 2, 1, 0, 0, 1, 0, a2, 0) | 10001011 | 11 reg:3=a0&7 rm:3=d&7 }" "asm"="vpcompressq {a0}, {d} {lb}{a2}{rb}" "dclass"="xv" "feature"="avx512f,avx512vl" "nodecode" {
  %r = call <2 x i64> @llvm.x86.avx512.mask.compress.q.128(<2 x i64> %a0, <2 x i64> %a1, i8 %a2)
  ret <2 x i64> %r
}

declare <2 x i64> @llvm.x86.avx512.mask.compress.q.128(<2 x i64>, <2 x i64>, i8)

define <4 x i64> @X86.avx512.mask.compress.q.256(<4 x i64> "reg"="yv" %a0, <4 x i64> "reg"="yv" "tied" %a1, i8 "reg"="k" %a2) "enc"="{ if(a2!=0) evexr(a0, d, 2, 1, 0, 1, 1, 0, a2, 0) | 10001011 | 11 reg:3=a0&7 rm:3=d&7 }" "asm"="vpcompressq {a0}, {d} {lb}{a2}{rb}" "dclass"="yv" "feature"="avx512f,avx512vl" "nodecode" {
  %r = call <4 x i64> @llvm.x86.avx512.mask.compress.q.256(<4 x i64> %a0, <4 x i64> %a1, i8 %a2)
  ret <4 x i64> %r
}

declare <4 x i64> @llvm.x86.avx512.mask.compress.q.256(<4 x i64>, <4 x i64>, i8)

define <8 x i64> @X86.avx512.mask.compress.q.512(<8 x i64> "reg"="zv" %a0, <8 x i64> "reg"="zv" "tied" %a1, i8 "reg"="k" %a2) "enc"="{ if(a2!=0) evexr(a0, d, 2, 1, 0, 2, 1, 0, a2, 0) | 10001011 | 11 reg:3=a0&7 rm:3=d&7 }" "asm"="vpcompressq {a0}, {d} {lb}{a2}{rb}" "dclass"="zv" "feature"="avx512f" "nodecode" {
  %r = call <8 x i64> @llvm.x86.avx512.mask.compress.q.512(<8 x i64> %a0, <8 x i64> %a1, i8 %a2)
  ret <8 x i64> %r
}

declare <8 x i64> @llvm.x86.avx512.mask.compress.q.512(<8 x i64>, <8 x i64>, i8)

define void @X86.avx512.mask.compress.store.b.128(ptr "reg"="r64" %a0, <16 x i8> "reg"="xv" %a1, i16 "reg"="k" %a2) "enc"="{ if(a2!=0) evexm(a1, a0, 4, 2, 0, 0, 0, 1, 0, a2, 0) | 01100011 | rm(a1, a0, 4, 1, 0, 0, 1) }" "asm"="vpcompressb {a1}, 0({a0}) {lb}{a2}{rb}" "feature"="avx512vbmi2,avx512vl" "nodecode" {
  call void @llvm.x86.avx512.mask.compress.store.b.128(ptr %a0, <16 x i8> %a1, i16 %a2)
  ret void
}

declare void @llvm.x86.avx512.mask.compress.store.b.128(ptr, <16 x i8>, i16)

define void @X86.avx512.mask.compress.store.b.256(ptr "reg"="r64" %a0, <32 x i8> "reg"="yv" %a1, i32 "reg"="k" %a2) "enc"="{ if(a2!=0) evexm(a1, a0, 4, 2, 0, 0, 1, 1, 0, a2, 0) | 01100011 | rm(a1, a0, 4, 1, 0, 0, 1) }" "asm"="vpcompressb {a1}, 0({a0}) {lb}{a2}{rb}" "feature"="avx512vbmi2,avx512vl" "nodecode" {
  call void @llvm.x86.avx512.mask.compress.store.b.256(ptr %a0, <32 x i8> %a1, i32 %a2)
  ret void
}

declare void @llvm.x86.avx512.mask.compress.store.b.256(ptr, <32 x i8>, i32)

define void @X86.avx512.mask.compress.store.b.512(ptr "reg"="r64" %a0, <64 x i8> "reg"="zv" %a1, i64 "reg"="k" %a2) "enc"="{ if(a2!=0) evexm(a1, a0, 4, 2, 0, 0, 2, 1, 0, a2, 0) | 01100011 | rm(a1, a0, 4, 1, 0, 0, 1) }" "asm"="vpcompressb {a1}, 0({a0}) {lb}{a2}{rb}" "feature"="avx512vbmi2" "nodecode" {
  call void @llvm.x86.avx512.mask.compress.store.b.512(ptr %a0, <64 x i8> %a1, i64 %a2)
  ret void
}

declare void @llvm.x86.avx512.mask.compress.store.b.512(ptr, <64 x i8>, i64)

define void @X86.avx512.mask.compress.store.d.128(ptr "reg"="r64" %a0, <4 x i32> "reg"="xv" %a1, i8 "reg"="k" %a2) "enc"="{ if(a2!=0) evexm(a1, a0, 4, 2, 0, 0, 0, 1, 0, a2, 0) | 10001011 | rm(a1, a0, 4, 1, 0, 0, 4) }" "asm"="vpcompressd {a1}, 0({a0}) {lb}{a2}{rb}" "feature"="avx512f,avx512vl" "nodecode" {
  call void @llvm.x86.avx512.mask.compress.store.d.128(ptr %a0, <4 x i32> %a1, i8 %a2)
  ret void
}

declare void @llvm.x86.avx512.mask.compress.store.d.128(ptr, <4 x i32>, i8)

define void @X86.avx512.mask.compress.store.d.256(ptr "reg"="r64" %a0, <8 x i32> "reg"="yv" %a1, i8 "reg"="k" %a2) "enc"="{ if(a2!=0) evexm(a1, a0, 4, 2, 0, 0, 1, 1, 0, a2, 0) | 10001011 | rm(a1, a0, 4, 1, 0, 0, 4) }" "asm"="vpcompressd {a1}, 0({a0}) {lb}{a2}{rb}" "feature"="avx512f,avx512vl" "nodecode" {
  call void @llvm.x86.avx512.mask.compress.store.d.256(ptr %a0, <8 x i32> %a1, i8 %a2)
  ret void
}

declare void @llvm.x86.avx512.mask.compress.store.d.256(ptr, <8 x i32>, i8)

define void @X86.avx512.mask.compress.store.d.512(ptr "reg"="r64" %a0, <16 x i32> "reg"="zv" %a1, i16 "reg"="k" %a2) "enc"="{ if(a2!=0) evexm(a1, a0, 4, 2, 0, 0, 2, 1, 0, a2, 0) | 10001011 | rm(a1, a0, 4, 1, 0, 0, 4) }" "asm"="vpcompressd {a1}, 0({a0}) {lb}{a2}{rb}" "feature"="avx512f" "nodecode" {
  call void @llvm.x86.avx512.mask.compress.store.d.512(ptr %a0, <16 x i32> %a1, i16 %a2)
  ret void
}

declare void @llvm.x86.avx512.mask.compress.store.d.512(ptr, <16 x i32>, i16)

define void @X86.avx512.mask.compress.store.pd.128(ptr "reg"="r64" %a0, <2 x double> "reg"="xv" %a1, i8 "reg"="k" %a2) "enc"="{ if(a2!=0) evexm(a1, a0, 4, 2, 1, 0, 0, 1, 0, a2, 0) | 10001010 | rm(a1, a0, 4, 1, 0, 0, 8) }" "asm"="vcompresspd {a1}, 0({a0}) {lb}{a2}{rb}" "feature"="avx512f,avx512vl" "nodecode" {
  call void @llvm.x86.avx512.mask.compress.store.pd.128(ptr %a0, <2 x double> %a1, i8 %a2)
  ret void
}

declare void @llvm.x86.avx512.mask.compress.store.pd.128(ptr, <2 x double>, i8)

define void @X86.avx512.mask.compress.store.pd.256(ptr "reg"="r64" %a0, <4 x double> "reg"="yv" %a1, i8 "reg"="k" %a2) "enc"="{ if(a2!=0) evexm(a1, a0, 4, 2, 1, 0, 1, 1, 0, a2, 0) | 10001010 | rm(a1, a0, 4, 1, 0, 0, 8) }" "asm"="vcompresspd {a1}, 0({a0}) {lb}{a2}{rb}" "feature"="avx512f,avx512vl" "nodecode" {
  call void @llvm.x86.avx512.mask.compress.store.pd.256(ptr %a0, <4 x double> %a1, i8 %a2)
  ret void
}

declare void @llvm.x86.avx512.mask.compress.store.pd.256(ptr, <4 x double>, i8)

define void @X86.avx512.mask.compress.store.pd.512(ptr "reg"="r64" %a0, <8 x double> "reg"="zv" %a1, i8 "reg"="k" %a2) "enc"="{ if(a2!=0) evexm(a1, a0, 4, 2, 1, 0, 2, 1, 0, a2, 0) | 10001010 | rm(a1, a0, 4, 1, 0, 0, 8) }" "asm"="vcompresspd {a1}, 0({a0}) {lb}{a2}{rb}" "feature"="avx512f" "nodecode" {
  call void @llvm.x86.avx512.mask.compress.store.pd.512(ptr %a0, <8 x double> %a1, i8 %a2)
  ret void
}

declare void @llvm.x86.avx512.mask.compress.store.pd.512(ptr, <8 x double>, i8)

define void @X86.avx512.mask.compress.store.ps.128(ptr "reg"="r64" %a0, <4 x float> "reg"="xv" %a1, i8 "reg"="k" %a2) "enc"="{ if(a2!=0) evexm(a1, a0, 4, 2, 0, 0, 0, 1, 0, a2, 0) | 10001010 | rm(a1, a0, 4, 1, 0, 0, 4) }" "asm"="vcompressps {a1}, 0({a0}) {lb}{a2}{rb}" "feature"="avx512f,avx512vl" "nodecode" {
  call void @llvm.x86.avx512.mask.compress.store.ps.128(ptr %a0, <4 x float> %a1, i8 %a2)
  ret void
}

declare void @llvm.x86.avx512.mask.compress.store.ps.128(ptr, <4 x float>, i8)

define void @X86.avx512.mask.compress.store.ps.256(ptr "reg"="r64" %a0, <8 x float> "reg"="yv" %a1, i8 "reg"="k" %a2) "enc"="{ if(a2!=0) evexm(a1, a0, 4, 2, 0, 0, 1, 1, 0, a2, 0) | 10001010 | rm(a1, a0, 4, 1, 0, 0, 4) }" "asm"="vcompressps {a1}, 0({a0}) {lb}{a2}{rb}" "feature"="avx512f,avx512vl" "nodecode" {
  call void @llvm.x86.avx512.mask.compress.store.ps.256(ptr %a0, <8 x float> %a1, i8 %a2)
  ret void
}

declare void @llvm.x86.avx512.mask.compress.store.ps.256(ptr, <8 x float>, i8)

define void @X86.avx512.mask.compress.store.ps.512(ptr "reg"="r64" %a0, <16 x float> "reg"="zv" %a1, i16 "reg"="k" %a2) "enc"="{ if(a2!=0) evexm(a1, a0, 4, 2, 0, 0, 2, 1, 0, a2, 0) | 10001010 | rm(a1, a0, 4, 1, 0, 0, 4) }" "asm"="vcompressps {a1}, 0({a0}) {lb}{a2}{rb}" "feature"="avx512f" "nodecode" {
  call void @llvm.x86.avx512.mask.compress.store.ps.512(ptr %a0, <16 x float> %a1, i16 %a2)
  ret void
}

declare void @llvm.x86.avx512.mask.compress.store.ps.512(ptr, <16 x float>, i16)

define void @X86.avx512.mask.compress.store.q.128(ptr "reg"="r64" %a0, <2 x i64> "reg"="xv" %a1, i8 "reg"="k" %a2) "enc"="{ if(a2!=0) evexm(a1, a0, 4, 2, 1, 0, 0, 1, 0, a2, 0) | 10001011 | rm(a1, a0, 4, 1, 0, 0, 8) }" "asm"="vpcompressq {a1}, 0({a0}) {lb}{a2}{rb}" "feature"="avx512f,avx512vl" "nodecode" {
  call void @llvm.x86.avx512.mask.compress.store.q.128(ptr %a0, <2 x i64> %a1, i8 %a2)
  ret void
}

declare void @llvm.x86.avx512.mask.compress.store.q.128(ptr, <2 x i64>, i8)

define void @X86.avx512.mask.compress.store.q.256(ptr "reg"="r64" %a0, <4 x i64> "reg"="yv" %a1, i8 "reg"="k" %a2) "enc"="{ if(a2!=0) evexm(a1, a0, 4, 2, 1, 0, 1, 1, 0, a2, 0) | 10001011 | rm(a1, a0, 4, 1, 0, 0, 8) }" "asm"="vpcompressq {a1}, 0({a0}) {lb}{a2}{rb}" "feature"="avx512f,avx512vl" "nodecode" {
  call void @llvm.x86.avx512.mask.compress.store.q.256(ptr %a0, <4 x i64> %a1, i8 %a2)
  ret void
}

declare void @llvm.x86.avx512.mask.compress.store.q.256(ptr, <4 x i64>, i8)

define void @X86.avx512.mask.compress.store.q.512(ptr "reg"="r64" %a0, <8 x i64> "reg"="zv" %a1, i8 "reg"="k" %a2) "enc"="{ if(a2!=0) evexm(a1, a0, 4, 2, 1, 0, 2, 1, 0, a2, 0) | 10001011 | rm(a1, a0, 4, 1, 0, 0, 8) }" "asm"="vpcompressq {a1}, 0({a0}) {lb}{a2}{rb}" "feature"="avx512f" "nodecode" {
  call void @llvm.x86.avx512.mask.compress.store.q.512(ptr %a0, <8 x i64> %a1, i8 %a2)
  ret void
}

declare void @llvm.x86.avx512.mask.compress.store.q.512(ptr, <8 x i64>, i8)

define void @X86.avx512.mask.compress.store.w.128(ptr "reg"="r64" %a0, <8 x i16> "reg"="xv" %a1, i8 "reg"="k" %a2) "enc"="{ if(a2!=0) evexm(a1, a0, 4, 2, 1, 0, 0, 1, 0, a2, 0) | 01100011 | rm(a1, a0, 4, 1, 0, 0, 2) }" "asm"="vpcompressw {a1}, 0({a0}) {lb}{a2}{rb}" "feature"="avx512vbmi2,avx512vl" "nodecode" {
  call void @llvm.x86.avx512.mask.compress.store.w.128(ptr %a0, <8 x i16> %a1, i8 %a2)
  ret void
}

declare void @llvm.x86.avx512.mask.compress.store.w.128(ptr, <8 x i16>, i8)

define void @X86.avx512.mask.compress.store.w.256(ptr "reg"="r64" %a0, <16 x i16> "reg"="yv" %a1, i16 "reg"="k" %a2) "enc"="{ if(a2!=0) evexm(a1, a0, 4, 2, 1, 0, 1, 1, 0, a2, 0) | 01100011 | rm(a1, a0, 4, 1, 0, 0, 2) }" "asm"="vpcompressw {a1}, 0({a0}) {lb}{a2}{rb}" "feature"="avx512vbmi2,avx512vl" "nodecode" {
  call void @llvm.x86.avx512.mask.compress.store.w.256(ptr %a0, <16 x i16> %a1, i16 %a2)
  ret void
}

declare void @llvm.x86.avx512.mask.compress.store.w.256(ptr, <16 x i16>, i16)

define void @X86.avx512.mask.compress.store.w.512(ptr "reg"="r64" %a0, <32 x i16> "reg"="zv" %a1, i32 "reg"="k" %a2) "enc"="{ if(a2!=0) evexm(a1, a0, 4, 2, 1, 0, 2, 1, 0, a2, 0) | 01100011 | rm(a1, a0, 4, 1, 0, 0, 2) }" "asm"="vpcompressw {a1}, 0({a0}) {lb}{a2}{rb}" "feature"="avx512vbmi2" "nodecode" {
  call void @llvm.x86.avx512.mask.compress.store.w.512(ptr %a0, <32 x i16> %a1, i32 %a2)
  ret void
}

declare void @llvm.x86.avx512.mask.compress.store.w.512(ptr, <32 x i16>, i32)

define <8 x i16> @X86.avx512.mask.compress.w.128(<8 x i16> "reg"="xv" %a0, <8 x i16> "reg"="xv" "tied" %a1, i8 "reg"="k" %a2) "enc"="{ if(a2!=0) evexr(a0, d, 2, 1, 0, 0, 1, 0, a2, 0) | 01100011 | 11 reg:3=a0&7 rm:3=d&7 }" "asm"="vpcompressw {a0}, {d} {lb}{a2}{rb}" "dclass"="xv" "feature"="avx512vbmi2,avx512vl" "nodecode" {
  %r = call <8 x i16> @llvm.x86.avx512.mask.compress.w.128(<8 x i16> %a0, <8 x i16> %a1, i8 %a2)
  ret <8 x i16> %r
}

declare <8 x i16> @llvm.x86.avx512.mask.compress.w.128(<8 x i16>, <8 x i16>, i8)

define <16 x i16> @X86.avx512.mask.compress.w.256(<16 x i16> "reg"="yv" %a0, <16 x i16> "reg"="yv" "tied" %a1, i16 "reg"="k" %a2) "enc"="{ if(a2!=0) evexr(a0, d, 2, 1, 0, 1, 1, 0, a2, 0) | 01100011 | 11 reg:3=a0&7 rm:3=d&7 }" "asm"="vpcompressw {a0}, {d} {lb}{a2}{rb}" "dclass"="yv" "feature"="avx512vbmi2,avx512vl" "nodecode" {
  %r = call <16 x i16> @llvm.x86.avx512.mask.compress.w.256(<16 x i16> %a0, <16 x i16> %a1, i16 %a2)
  ret <16 x i16> %r
}

declare <16 x i16> @llvm.x86.avx512.mask.compress.w.256(<16 x i16>, <16 x i16>, i16)

define <32 x i16> @X86.avx512.mask.compress.w.512(<32 x i16> "reg"="zv" %a0, <32 x i16> "reg"="zv" "tied" %a1, i32 "reg"="k" %a2) "enc"="{ if(a2!=0) evexr(a0, d, 2, 1, 0, 2, 1, 0, a2, 0) | 01100011 | 11 reg:3=a0&7 rm:3=d&7 }" "asm"="vpcompressw {a0}, {d} {lb}{a2}{rb}" "dclass"="zv" "feature"="avx512vbmi2" "nodecode" {
  %r = call <32 x i16> @llvm.x86.avx512.mask.compress.w.512(<32 x i16> %a0, <32 x i16> %a1, i32 %a2)
  ret <32 x i16> %r
}

declare <32 x i16> @llvm.x86.avx512.mask.compress.w.512(<32 x i16>, <32 x i16>, i32)

define <4 x i32> @X86.avx512.mask.cvtpd2dq.128(<2 x double> "reg"="xv" %a0, <4 x i32> "reg"="xv" "tied" %a1, i8 "reg"="k" %a2) "enc"="{ if(a2!=0) evexr(d, a0, 1, 1, 0, 0, 3, 0, a2, 0) | 11100110 | 11 reg:3=d&7 rm:3=a0&7 }" "asm"="vcvtpd2dq {a0}, {d} {lb}{a2}{rb}" "dclass"="xv" "feature"="avx512f,avx512vl" "nodecode" {
  %r = call <4 x i32> @llvm.x86.avx512.mask.cvtpd2dq.128(<2 x double> %a0, <4 x i32> %a1, i8 %a2)
  ret <4 x i32> %r
}

declare <4 x i32> @llvm.x86.avx512.mask.cvtpd2dq.128(<2 x double>, <4 x i32>, i8)

define <8 x i32> @X86.avx512.mask.cvtpd2dq.512(<8 x double> "reg"="zv" %a0, <8 x i32> "reg"="yv" "tied" %a1, i8 "reg"="k" %a2) "enc"="{ if(a2!=0) evexr(d, a0, 1, 1, 0, 2, 3, 0, a2, 0) | 11100110 | 11 reg:3=d&7 rm:3=a0&7 }" "asm"="vcvtpd2dq {a0}, {d} {lb}{a2}{rb}" "dclass"="yv" "feature"="avx512f" "nodecode" {
  %r = call <8 x i32> @llvm.x86.avx512.mask.cvtpd2dq.512(<8 x double> %a0, <8 x i32> %a1, i8 %a2, i32 4)
  ret <8 x i32> %r
}

declare <8 x i32> @llvm.x86.avx512.mask.cvtpd2dq.512(<8 x double>, <8 x i32>, i8, i32)

define <4 x float> @X86.avx512.mask.cvtpd2ps(<2 x double> "reg"="xv" %a0, <4 x float> "reg"="xv" "tied" %a1, i8 "reg"="k" %a2) "enc"="{ if(a2!=0) evexr(d, a0, 1, 1, 0, 0, 1, 0, a2, 0) | 01011010 | 11 reg:3=d&7 rm:3=a0&7 }" "asm"="vcvtpd2ps {a0}, {d} {lb}{a2}{rb}" "dclass"="xv" "feature"="avx512f,avx512vl" "nodecode" {
  %r = call <4 x float> @llvm.x86.avx512.mask.cvtpd2ps(<2 x double> %a0, <4 x float> %a1, i8 %a2)
  ret <4 x float> %r
}

declare <4 x float> @llvm.x86.avx512.mask.cvtpd2ps(<2 x double>, <4 x float>, i8)

define <8 x float> @X86.avx512.mask.cvtpd2ps.512(<8 x double> "reg"="zv" %a0, <8 x float> "reg"="yv" "tied" %a1, i8 "reg"="k" %a2) "enc"="{ if(a2!=0) evexr(d, a0, 1, 1, 0, 2, 1, 0, a2, 0) | 01011010 | 11 reg:3=d&7 rm:3=a0&7 }" "asm"="vcvtpd2ps {a0}, {d} {lb}{a2}{rb}" "dclass"="yv" "feature"="avx512f" "nodecode" {
  %r = call <8 x float> @llvm.x86.avx512.mask.cvtpd2ps.512(<8 x double> %a0, <8 x float> %a1, i8 %a2, i32 4)
  ret <8 x float> %r
}

declare <8 x float> @llvm.x86.avx512.mask.cvtpd2ps.512(<8 x double>, <8 x float>, i8, i32)

define <2 x i64> @X86.avx512.mask.cvtpd2qq.128(<2 x double> "reg"="xv" %a0, <2 x i64> "reg"="xv" "tied" %a1, i8 "reg"="k" %a2) "enc"="{ if(a2!=0) evexr(d, a0, 1, 1, 0, 0, 1, 0, a2, 0) | 01111011 | 11 reg:3=d&7 rm:3=a0&7 }" "asm"="vcvtpd2qq {a0}, {d} {lb}{a2}{rb}" "dclass"="xv" "feature"="avx512dq,avx512vl" "nodecode" {
  %r = call <2 x i64> @llvm.x86.avx512.mask.cvtpd2qq.128(<2 x double> %a0, <2 x i64> %a1, i8 %a2)
  ret <2 x i64> %r
}

declare <2 x i64> @llvm.x86.avx512.mask.cvtpd2qq.128(<2 x double>, <2 x i64>, i8)

define <4 x i64> @X86.avx512.mask.cvtpd2qq.256(<4 x double> "reg"="yv" %a0, <4 x i64> "reg"="yv" "tied" %a1, i8 "reg"="k" %a2) "enc"="{ if(a2!=0) evexr(d, a0, 1, 1, 0, 1, 1, 0, a2, 0) | 01111011 | 11 reg:3=d&7 rm:3=a0&7 }" "asm"="vcvtpd2qq {a0}, {d} {lb}{a2}{rb}" "dclass"="yv" "feature"="avx512dq,avx512vl" "nodecode" {
  %r = call <4 x i64> @llvm.x86.avx512.mask.cvtpd2qq.256(<4 x double> %a0, <4 x i64> %a1, i8 %a2)
  ret <4 x i64> %r
}

declare <4 x i64> @llvm.x86.avx512.mask.cvtpd2qq.256(<4 x double>, <4 x i64>, i8)

define <8 x i64> @X86.avx512.mask.cvtpd2qq.512(<8 x double> "reg"="zv" %a0, <8 x i64> "reg"="zv" "tied" %a1, i8 "reg"="k" %a2) "enc"="{ if(a2!=0) evexr(d, a0, 1, 1, 0, 2, 1, 0, a2, 0) | 01111011 | 11 reg:3=d&7 rm:3=a0&7 }" "asm"="vcvtpd2qq {a0}, {d} {lb}{a2}{rb}" "dclass"="zv" "feature"="avx512dq" "nodecode" {
  %r = call <8 x i64> @llvm.x86.avx512.mask.cvtpd2qq.512(<8 x double> %a0, <8 x i64> %a1, i8 %a2, i32 4)
  ret <8 x i64> %r
}

declare <8 x i64> @llvm.x86.avx512.mask.cvtpd2qq.512(<8 x double>, <8 x i64>, i8, i32)

define <4 x i32> @X86.avx512.mask.cvtpd2udq.128(<2 x double> "reg"="xv" %a0, <4 x i32> "reg"="xv" "tied" %a1, i8 "reg"="k" %a2) "enc"="{ if(a2!=0) evexr(d, a0, 1, 1, 0, 0, 0, 0, a2, 0) | 01111001 | 11 reg:3=d&7 rm:3=a0&7 }" "asm"="vcvtpd2udq {a0}, {d} {lb}{a2}{rb}" "dclass"="xv" "feature"="avx512f,avx512vl" "nodecode" {
  %r = call <4 x i32> @llvm.x86.avx512.mask.cvtpd2udq.128(<2 x double> %a0, <4 x i32> %a1, i8 %a2)
  ret <4 x i32> %r
}

declare <4 x i32> @llvm.x86.avx512.mask.cvtpd2udq.128(<2 x double>, <4 x i32>, i8)

define <4 x i32> @X86.avx512.mask.cvtpd2udq.256(<4 x double> "reg"="yv" %a0, <4 x i32> "reg"="xv" "tied" %a1, i8 "reg"="k" %a2) "enc"="{ if(a2!=0) evexr(d, a0, 1, 1, 0, 1, 0, 0, a2, 0) | 01111001 | 11 reg:3=d&7 rm:3=a0&7 }" "asm"="vcvtpd2udq {a0}, {d} {lb}{a2}{rb}" "dclass"="xv" "feature"="avx512f,avx512vl" "nodecode" {
  %r = call <4 x i32> @llvm.x86.avx512.mask.cvtpd2udq.256(<4 x double> %a0, <4 x i32> %a1, i8 %a2)
  ret <4 x i32> %r
}

declare <4 x i32> @llvm.x86.avx512.mask.cvtpd2udq.256(<4 x double>, <4 x i32>, i8)

define <8 x i32> @X86.avx512.mask.cvtpd2udq.512(<8 x double> "reg"="zv" %a0, <8 x i32> "reg"="yv" "tied" %a1, i8 "reg"="k" %a2) "enc"="{ if(a2!=0) evexr(d, a0, 1, 1, 0, 2, 0, 0, a2, 0) | 01111001 | 11 reg:3=d&7 rm:3=a0&7 }" "asm"="vcvtpd2udq {a0}, {d} {lb}{a2}{rb}" "dclass"="yv" "feature"="avx512f" "nodecode" {
  %r = call <8 x i32> @llvm.x86.avx512.mask.cvtpd2udq.512(<8 x double> %a0, <8 x i32> %a1, i8 %a2, i32 4)
  ret <8 x i32> %r
}

declare <8 x i32> @llvm.x86.avx512.mask.cvtpd2udq.512(<8 x double>, <8 x i32>, i8, i32)

define <2 x i64> @X86.avx512.mask.cvtpd2uqq.128(<2 x double> "reg"="xv" %a0, <2 x i64> "reg"="xv" "tied" %a1, i8 "reg"="k" %a2) "enc"="{ if(a2!=0) evexr(d, a0, 1, 1, 0, 0, 1, 0, a2, 0) | 01111001 | 11 reg:3=d&7 rm:3=a0&7 }" "asm"="vcvtpd2uqq {a0}, {d} {lb}{a2}{rb}" "dclass"="xv" "feature"="avx512dq,avx512vl" "nodecode" {
  %r = call <2 x i64> @llvm.x86.avx512.mask.cvtpd2uqq.128(<2 x double> %a0, <2 x i64> %a1, i8 %a2)
  ret <2 x i64> %r
}

declare <2 x i64> @llvm.x86.avx512.mask.cvtpd2uqq.128(<2 x double>, <2 x i64>, i8)

define <4 x i64> @X86.avx512.mask.cvtpd2uqq.256(<4 x double> "reg"="yv" %a0, <4 x i64> "reg"="yv" "tied" %a1, i8 "reg"="k" %a2) "enc"="{ if(a2!=0) evexr(d, a0, 1, 1, 0, 1, 1, 0, a2, 0) | 01111001 | 11 reg:3=d&7 rm:3=a0&7 }" "asm"="vcvtpd2uqq {a0}, {d} {lb}{a2}{rb}" "dclass"="yv" "feature"="avx512dq,avx512vl" "nodecode" {
  %r = call <4 x i64> @llvm.x86.avx512.mask.cvtpd2uqq.256(<4 x double> %a0, <4 x i64> %a1, i8 %a2)
  ret <4 x i64> %r
}

declare <4 x i64> @llvm.x86.avx512.mask.cvtpd2uqq.256(<4 x double>, <4 x i64>, i8)

define <8 x i64> @X86.avx512.mask.cvtpd2uqq.512(<8 x double> "reg"="zv" %a0, <8 x i64> "reg"="zv" "tied" %a1, i8 "reg"="k" %a2) "enc"="{ if(a2!=0) evexr(d, a0, 1, 1, 0, 2, 1, 0, a2, 0) | 01111001 | 11 reg:3=d&7 rm:3=a0&7 }" "asm"="vcvtpd2uqq {a0}, {d} {lb}{a2}{rb}" "dclass"="zv" "feature"="avx512dq" "nodecode" {
  %r = call <8 x i64> @llvm.x86.avx512.mask.cvtpd2uqq.512(<8 x double> %a0, <8 x i64> %a1, i8 %a2, i32 4)
  ret <8 x i64> %r
}

declare <8 x i64> @llvm.x86.avx512.mask.cvtpd2uqq.512(<8 x double>, <8 x i64>, i8, i32)

define <16 x i32> @X86.avx512.mask.cvtps2dq.512(<16 x float> "reg"="zv" %a0, <16 x i32> "reg"="zv" "tied" %a1, i16 "reg"="k" %a2) "enc"="{ if(a2!=0) evexr(d, a0, 1, 0, 0, 2, 1, 0, a2, 0) | 01011011 | 11 reg:3=d&7 rm:3=a0&7 }" "asm"="vcvtps2dq {a0}, {d} {lb}{a2}{rb}" "dclass"="zv" "feature"="avx512f" "nodecode" {
  %r = call <16 x i32> @llvm.x86.avx512.mask.cvtps2dq.512(<16 x float> %a0, <16 x i32> %a1, i16 %a2, i32 4)
  ret <16 x i32> %r
}

declare <16 x i32> @llvm.x86.avx512.mask.cvtps2dq.512(<16 x float>, <16 x i32>, i16, i32)

define <8 x double> @X86.avx512.mask.cvtps2pd.512(<8 x float> "reg"="yv" %a0, <8 x double> "reg"="zv" "tied" %a1, i8 "reg"="k" %a2) "enc"="{ if(a2!=0) evexr(d, a0, 1, 0, 0, 2, 0, 0, a2, 0) | 01011010 | 11 reg:3=d&7 rm:3=a0&7 }" "asm"="vcvtps2pd {a0}, {d} {lb}{a2}{rb}" "dclass"="zv" "feature"="avx512f" "nodecode" {
  %r = call <8 x double> @llvm.x86.avx512.mask.cvtps2pd.512(<8 x float> %a0, <8 x double> %a1, i8 %a2, i32 4)
  ret <8 x double> %r
}

declare <8 x double> @llvm.x86.avx512.mask.cvtps2pd.512(<8 x float>, <8 x double>, i8, i32)

define <2 x i64> @X86.avx512.mask.cvtps2qq.128(<4 x float> "reg"="xv" %a0, <2 x i64> "reg"="xv" "tied" %a1, i8 "reg"="k" %a2) "enc"="{ if(a2!=0) evexr(d, a0, 1, 0, 0, 0, 1, 0, a2, 0) | 01111011 | 11 reg:3=d&7 rm:3=a0&7 }" "asm"="vcvtps2qq {a0}, {d} {lb}{a2}{rb}" "dclass"="xv" "feature"="avx512dq,avx512vl" "nodecode" {
  %r = call <2 x i64> @llvm.x86.avx512.mask.cvtps2qq.128(<4 x float> %a0, <2 x i64> %a1, i8 %a2)
  ret <2 x i64> %r
}

declare <2 x i64> @llvm.x86.avx512.mask.cvtps2qq.128(<4 x float>, <2 x i64>, i8)

define <4 x i64> @X86.avx512.mask.cvtps2qq.256(<4 x float> "reg"="xv" %a0, <4 x i64> "reg"="yv" "tied" %a1, i8 "reg"="k" %a2) "enc"="{ if(a2!=0) evexr(d, a0, 1, 0, 0, 1, 1, 0, a2, 0) | 01111011 | 11 reg:3=d&7 rm:3=a0&7 }" "asm"="vcvtps2qq {a0}, {d} {lb}{a2}{rb}" "dclass"="yv" "feature"="avx512dq,avx512vl" "nodecode" {
  %r = call <4 x i64> @llvm.x86.avx512.mask.cvtps2qq.256(<4 x float> %a0, <4 x i64> %a1, i8 %a2)
  ret <4 x i64> %r
}

declare <4 x i64> @llvm.x86.avx512.mask.cvtps2qq.256(<4 x float>, <4 x i64>, i8)

define <8 x i64> @X86.avx512.mask.cvtps2qq.512(<8 x float> "reg"="yv" %a0, <8 x i64> "reg"="zv" "tied" %a1, i8 "reg"="k" %a2) "enc"="{ if(a2!=0) evexr(d, a0, 1, 0, 0, 2, 1, 0, a2, 0) | 01111011 | 11 reg:3=d&7 rm:3=a0&7 }" "asm"="vcvtps2qq {a0}, {d} {lb}{a2}{rb}" "dclass"="zv" "feature"="avx512dq" "nodecode" {
  %r = call <8 x i64> @llvm.x86.avx512.mask.cvtps2qq.512(<8 x float> %a0, <8 x i64> %a1, i8 %a2, i32 4)
  ret <8 x i64> %r
}

declare <8 x i64> @llvm.x86.avx512.mask.cvtps2qq.512(<8 x float>, <8 x i64>, i8, i32)

define <4 x i32> @X86.avx512.mask.cvtps2udq.128(<4 x float> "reg"="xv" %a0, <4 x i32> "reg"="xv" "tied" %a1, i8 "reg"="k" %a2) "enc"="{ if(a2!=0) evexr(d, a0, 1, 0, 0, 0, 0, 0, a2, 0) | 01111001 | 11 reg:3=d&7 rm:3=a0&7 }" "asm"="vcvtps2udq {a0}, {d} {lb}{a2}{rb}" "dclass"="xv" "feature"="avx512f,avx512vl" "nodecode" {
  %r = call <4 x i32> @llvm.x86.avx512.mask.cvtps2udq.128(<4 x float> %a0, <4 x i32> %a1, i8 %a2)
  ret <4 x i32> %r
}

declare <4 x i32> @llvm.x86.avx512.mask.cvtps2udq.128(<4 x float>, <4 x i32>, i8)

define <8 x i32> @X86.avx512.mask.cvtps2udq.256(<8 x float> "reg"="yv" %a0, <8 x i32> "reg"="yv" "tied" %a1, i8 "reg"="k" %a2) "enc"="{ if(a2!=0) evexr(d, a0, 1, 0, 0, 1, 0, 0, a2, 0) | 01111001 | 11 reg:3=d&7 rm:3=a0&7 }" "asm"="vcvtps2udq {a0}, {d} {lb}{a2}{rb}" "dclass"="yv" "feature"="avx512f,avx512vl" "nodecode" {
  %r = call <8 x i32> @llvm.x86.avx512.mask.cvtps2udq.256(<8 x float> %a0, <8 x i32> %a1, i8 %a2)
  ret <8 x i32> %r
}

declare <8 x i32> @llvm.x86.avx512.mask.cvtps2udq.256(<8 x float>, <8 x i32>, i8)

define <16 x i32> @X86.avx512.mask.cvtps2udq.512(<16 x float> "reg"="zv" %a0, <16 x i32> "reg"="zv" "tied" %a1, i16 "reg"="k" %a2) "enc"="{ if(a2!=0) evexr(d, a0, 1, 0, 0, 2, 0, 0, a2, 0) | 01111001 | 11 reg:3=d&7 rm:3=a0&7 }" "asm"="vcvtps2udq {a0}, {d} {lb}{a2}{rb}" "dclass"="zv" "feature"="avx512f" "nodecode" {
  %r = call <16 x i32> @llvm.x86.avx512.mask.cvtps2udq.512(<16 x float> %a0, <16 x i32> %a1, i16 %a2, i32 4)
  ret <16 x i32> %r
}

declare <16 x i32> @llvm.x86.avx512.mask.cvtps2udq.512(<16 x float>, <16 x i32>, i16, i32)

define <2 x i64> @X86.avx512.mask.cvtps2uqq.128(<4 x float> "reg"="xv" %a0, <2 x i64> "reg"="xv" "tied" %a1, i8 "reg"="k" %a2) "enc"="{ if(a2!=0) evexr(d, a0, 1, 0, 0, 0, 1, 0, a2, 0) | 01111001 | 11 reg:3=d&7 rm:3=a0&7 }" "asm"="vcvtps2uqq {a0}, {d} {lb}{a2}{rb}" "dclass"="xv" "feature"="avx512dq,avx512vl" "nodecode" {
  %r = call <2 x i64> @llvm.x86.avx512.mask.cvtps2uqq.128(<4 x float> %a0, <2 x i64> %a1, i8 %a2)
  ret <2 x i64> %r
}

declare <2 x i64> @llvm.x86.avx512.mask.cvtps2uqq.128(<4 x float>, <2 x i64>, i8)

define <4 x i64> @X86.avx512.mask.cvtps2uqq.256(<4 x float> "reg"="xv" %a0, <4 x i64> "reg"="yv" "tied" %a1, i8 "reg"="k" %a2) "enc"="{ if(a2!=0) evexr(d, a0, 1, 0, 0, 1, 1, 0, a2, 0) | 01111001 | 11 reg:3=d&7 rm:3=a0&7 }" "asm"="vcvtps2uqq {a0}, {d} {lb}{a2}{rb}" "dclass"="yv" "feature"="avx512dq,avx512vl" "nodecode" {
  %r = call <4 x i64> @llvm.x86.avx512.mask.cvtps2uqq.256(<4 x float> %a0, <4 x i64> %a1, i8 %a2)
  ret <4 x i64> %r
}

declare <4 x i64> @llvm.x86.avx512.mask.cvtps2uqq.256(<4 x float>, <4 x i64>, i8)

define <8 x i64> @X86.avx512.mask.cvtps2uqq.512(<8 x float> "reg"="yv" %a0, <8 x i64> "reg"="zv" "tied" %a1, i8 "reg"="k" %a2) "enc"="{ if(a2!=0) evexr(d, a0, 1, 0, 0, 2, 1, 0, a2, 0) | 01111001 | 11 reg:3=d&7 rm:3=a0&7 }" "asm"="vcvtps2uqq {a0}, {d} {lb}{a2}{rb}" "dclass"="zv" "feature"="avx512dq" "nodecode" {
  %r = call <8 x i64> @llvm.x86.avx512.mask.cvtps2uqq.512(<8 x float> %a0, <8 x i64> %a1, i8 %a2, i32 4)
  ret <8 x i64> %r
}

declare <8 x i64> @llvm.x86.avx512.mask.cvtps2uqq.512(<8 x float>, <8 x i64>, i8, i32)

define <4 x float> @X86.avx512.mask.cvtqq2ps.128(<2 x i64> "reg"="xv" %a0, <4 x float> "reg"="xv" "tied" %a1, i8 "reg"="k" %a2) "enc"="{ if(a2!=0) evexr(d, a0, 1, 1, 0, 0, 0, 0, a2, 0) | 01011011 | 11 reg:3=d&7 rm:3=a0&7 }" "asm"="vcvtqq2ps {a0}, {d} {lb}{a2}{rb}" "dclass"="xv" "feature"="avx512dq,avx512vl" "nodecode" {
  %r = call <4 x float> @llvm.x86.avx512.mask.cvtqq2ps.128(<2 x i64> %a0, <4 x float> %a1, i8 %a2)
  ret <4 x float> %r
}

declare <4 x float> @llvm.x86.avx512.mask.cvtqq2ps.128(<2 x i64>, <4 x float>, i8)

define <4 x float> @X86.avx512.mask.cvtsd2ss.round(<4 x float> "reg"="xv" %a0, <2 x double> "reg"="xv" %a1, <4 x float> "reg"="xv" "tied" %a2, i8 "reg"="k" %a3) "enc"="{ if(a3!=0) evexr(d, a1, 1, 1, a0, 0, 3, 0, a3, 0) | 01011010 | 11 reg:3=d&7 rm:3=a1&7 }" "asm"="vcvtsd2ss {a1}, {a0}, {d} {lb}{a3}{rb}" "dclass"="xv" "feature"="avx512f" "nodecode" {
  %r = call <4 x float> @llvm.x86.avx512.mask.cvtsd2ss.round(<4 x float> %a0, <2 x double> %a1, <4 x float> %a2, i8 %a3, i32 4)
  ret <4 x float> %r
}

declare <4 x float> @llvm.x86.avx512.mask.cvtsd2ss.round(<4 x float>, <2 x double>, <4 x float>, i8, i32)

define <2 x double> @X86.avx512.mask.cvtss2sd.round(<2 x double> "reg"="xv" %a0, <4 x float> "reg"="xv" %a1, <2 x double> "reg"="xv" "tied" %a2, i8 "reg"="k" %a3) "enc"="{ if(a3!=0) evexr(d, a1, 1, 0, a0, 0, 2, 0, a3, 0) | 01011010 | 11 reg:3=d&7 rm:3=a1&7 }" "asm"="vcvtss2sd {a1}, {a0}, {d} {lb}{a3}{rb}" "dclass"="xv" "feature"="avx512f" "nodecode" {
  %r = call <2 x double> @llvm.x86.avx512.mask.cvtss2sd.round(<2 x double> %a0, <4 x float> %a1, <2 x double> %a2, i8 %a3, i32 4)
  ret <2 x double> %r
}

declare <2 x double> @llvm.x86.avx512.mask.cvtss2sd.round(<2 x double>, <4 x float>, <2 x double>, i8, i32)

define <4 x i32> @X86.avx512.mask.cvttpd2dq.128(<2 x double> "reg"="xv" %a0, <4 x i32> "reg"="xv" "tied" %a1, i8 "reg"="k" %a2) "enc"="{ if(a2!=0) evexr(d, a0, 1, 1, 0, 0, 1, 0, a2, 0) | 11100110 | 11 reg:3=d&7 rm:3=a0&7 }" "asm"="vcvttpd2dq {a0}, {d} {lb}{a2}{rb}" "dclass"="xv" "feature"="avx512f,avx512vl" "nodecode" {
  %r = call <4 x i32> @llvm.x86.avx512.mask.cvttpd2dq.128(<2 x double> %a0, <4 x i32> %a1, i8 %a2)
  ret <4 x i32> %r
}

declare <4 x i32> @llvm.x86.avx512.mask.cvttpd2dq.128(<2 x double>, <4 x i32>, i8)

define <8 x i32> @X86.avx512.mask.cvttpd2dq.512(<8 x double> "reg"="zv" %a0, <8 x i32> "reg"="yv" "tied" %a1, i8 "reg"="k" %a2) "enc"="{ if(a2!=0) evexr(d, a0, 1, 1, 0, 2, 1, 0, a2, 0) | 11100110 | 11 reg:3=d&7 rm:3=a0&7 }" "asm"="vcvttpd2dq {a0}, {d} {lb}{a2}{rb}" "dclass"="yv" "feature"="avx512f" "nodecode" {
  %r = call <8 x i32> @llvm.x86.avx512.mask.cvttpd2dq.512(<8 x double> %a0, <8 x i32> %a1, i8 %a2, i32 4)
  ret <8 x i32> %r
}

declare <8 x i32> @llvm.x86.avx512.mask.cvttpd2dq.512(<8 x double>, <8 x i32>, i8, i32)

define <2 x i64> @X86.avx512.mask.cvttpd2qq.128(<2 x double> "reg"="xv" %a0, <2 x i64> "reg"="xv" "tied" %a1, i8 "reg"="k" %a2) "enc"="{ if(a2!=0) evexr(d, a0, 1, 1, 0, 0, 1, 0, a2, 0) | 01111010 | 11 reg:3=d&7 rm:3=a0&7 }" "asm"="vcvttpd2qq {a0}, {d} {lb}{a2}{rb}" "dclass"="xv" "feature"="avx512dq,avx512vl" "nodecode" {
  %r = call <2 x i64> @llvm.x86.avx512.mask.cvttpd2qq.128(<2 x double> %a0, <2 x i64> %a1, i8 %a2)
  ret <2 x i64> %r
}

declare <2 x i64> @llvm.x86.avx512.mask.cvttpd2qq.128(<2 x double>, <2 x i64>, i8)

define <4 x i64> @X86.avx512.mask.cvttpd2qq.256(<4 x double> "reg"="yv" %a0, <4 x i64> "reg"="yv" "tied" %a1, i8 "reg"="k" %a2) "enc"="{ if(a2!=0) evexr(d, a0, 1, 1, 0, 1, 1, 0, a2, 0) | 01111010 | 11 reg:3=d&7 rm:3=a0&7 }" "asm"="vcvttpd2qq {a0}, {d} {lb}{a2}{rb}" "dclass"="yv" "feature"="avx512dq,avx512vl" "nodecode" {
  %r = call <4 x i64> @llvm.x86.avx512.mask.cvttpd2qq.256(<4 x double> %a0, <4 x i64> %a1, i8 %a2)
  ret <4 x i64> %r
}

declare <4 x i64> @llvm.x86.avx512.mask.cvttpd2qq.256(<4 x double>, <4 x i64>, i8)

define <8 x i64> @X86.avx512.mask.cvttpd2qq.512(<8 x double> "reg"="zv" %a0, <8 x i64> "reg"="zv" "tied" %a1, i8 "reg"="k" %a2) "enc"="{ if(a2!=0) evexr(d, a0, 1, 1, 0, 2, 1, 0, a2, 0) | 01111010 | 11 reg:3=d&7 rm:3=a0&7 }" "asm"="vcvttpd2qq {a0}, {d} {lb}{a2}{rb}" "dclass"="zv" "feature"="avx512dq" "nodecode" {
  %r = call <8 x i64> @llvm.x86.avx512.mask.cvttpd2qq.512(<8 x double> %a0, <8 x i64> %a1, i8 %a2, i32 4)
  ret <8 x i64> %r
}

declare <8 x i64> @llvm.x86.avx512.mask.cvttpd2qq.512(<8 x double>, <8 x i64>, i8, i32)

define <4 x i32> @X86.avx512.mask.cvttpd2udq.128(<2 x double> "reg"="xv" %a0, <4 x i32> "reg"="xv" "tied" %a1, i8 "reg"="k" %a2) "enc"="{ if(a2!=0) evexr(d, a0, 1, 1, 0, 0, 0, 0, a2, 0) | 01111000 | 11 reg:3=d&7 rm:3=a0&7 }" "asm"="vcvttpd2udq {a0}, {d} {lb}{a2}{rb}" "dclass"="xv" "feature"="avx512f,avx512vl" "nodecode" {
  %r = call <4 x i32> @llvm.x86.avx512.mask.cvttpd2udq.128(<2 x double> %a0, <4 x i32> %a1, i8 %a2)
  ret <4 x i32> %r
}

declare <4 x i32> @llvm.x86.avx512.mask.cvttpd2udq.128(<2 x double>, <4 x i32>, i8)

define <4 x i32> @X86.avx512.mask.cvttpd2udq.256(<4 x double> "reg"="yv" %a0, <4 x i32> "reg"="xv" "tied" %a1, i8 "reg"="k" %a2) "enc"="{ if(a2!=0) evexr(d, a0, 1, 1, 0, 1, 0, 0, a2, 0) | 01111000 | 11 reg:3=d&7 rm:3=a0&7 }" "asm"="vcvttpd2udq {a0}, {d} {lb}{a2}{rb}" "dclass"="xv" "feature"="avx512f,avx512vl" "nodecode" {
  %r = call <4 x i32> @llvm.x86.avx512.mask.cvttpd2udq.256(<4 x double> %a0, <4 x i32> %a1, i8 %a2)
  ret <4 x i32> %r
}

declare <4 x i32> @llvm.x86.avx512.mask.cvttpd2udq.256(<4 x double>, <4 x i32>, i8)

define <8 x i32> @X86.avx512.mask.cvttpd2udq.512(<8 x double> "reg"="zv" %a0, <8 x i32> "reg"="yv" "tied" %a1, i8 "reg"="k" %a2) "enc"="{ if(a2!=0) evexr(d, a0, 1, 1, 0, 2, 0, 0, a2, 0) | 01111000 | 11 reg:3=d&7 rm:3=a0&7 }" "asm"="vcvttpd2udq {a0}, {d} {lb}{a2}{rb}" "dclass"="yv" "feature"="avx512f" "nodecode" {
  %r = call <8 x i32> @llvm.x86.avx512.mask.cvttpd2udq.512(<8 x double> %a0, <8 x i32> %a1, i8 %a2, i32 4)
  ret <8 x i32> %r
}

declare <8 x i32> @llvm.x86.avx512.mask.cvttpd2udq.512(<8 x double>, <8 x i32>, i8, i32)

define <2 x i64> @X86.avx512.mask.cvttpd2uqq.128(<2 x double> "reg"="xv" %a0, <2 x i64> "reg"="xv" "tied" %a1, i8 "reg"="k" %a2) "enc"="{ if(a2!=0) evexr(d, a0, 1, 1, 0, 0, 1, 0, a2, 0) | 01111000 | 11 reg:3=d&7 rm:3=a0&7 }" "asm"="vcvttpd2uqq {a0}, {d} {lb}{a2}{rb}" "dclass"="xv" "feature"="avx512dq,avx512vl" "nodecode" {
  %r = call <2 x i64> @llvm.x86.avx512.mask.cvttpd2uqq.128(<2 x double> %a0, <2 x i64> %a1, i8 %a2)
  ret <2 x i64> %r
}

declare <2 x i64> @llvm.x86.avx512.mask.cvttpd2uqq.128(<2 x double>, <2 x i64>, i8)

define <4 x i64> @X86.avx512.mask.cvttpd2uqq.256(<4 x double> "reg"="yv" %a0, <4 x i64> "reg"="yv" "tied" %a1, i8 "reg"="k" %a2) "enc"="{ if(a2!=0) evexr(d, a0, 1, 1, 0, 1, 1, 0, a2, 0) | 01111000 | 11 reg:3=d&7 rm:3=a0&7 }" "asm"="vcvttpd2uqq {a0}, {d} {lb}{a2}{rb}" "dclass"="yv" "feature"="avx512dq,avx512vl" "nodecode" {
  %r = call <4 x i64> @llvm.x86.avx512.mask.cvttpd2uqq.256(<4 x double> %a0, <4 x i64> %a1, i8 %a2)
  ret <4 x i64> %r
}

declare <4 x i64> @llvm.x86.avx512.mask.cvttpd2uqq.256(<4 x double>, <4 x i64>, i8)

define <8 x i64> @X86.avx512.mask.cvttpd2uqq.512(<8 x double> "reg"="zv" %a0, <8 x i64> "reg"="zv" "tied" %a1, i8 "reg"="k" %a2) "enc"="{ if(a2!=0) evexr(d, a0, 1, 1, 0, 2, 1, 0, a2, 0) | 01111000 | 11 reg:3=d&7 rm:3=a0&7 }" "asm"="vcvttpd2uqq {a0}, {d} {lb}{a2}{rb}" "dclass"="zv" "feature"="avx512dq" "nodecode" {
  %r = call <8 x i64> @llvm.x86.avx512.mask.cvttpd2uqq.512(<8 x double> %a0, <8 x i64> %a1, i8 %a2, i32 4)
  ret <8 x i64> %r
}

declare <8 x i64> @llvm.x86.avx512.mask.cvttpd2uqq.512(<8 x double>, <8 x i64>, i8, i32)

define <16 x i32> @X86.avx512.mask.cvttps2dq.512(<16 x float> "reg"="zv" %a0, <16 x i32> "reg"="zv" "tied" %a1, i16 "reg"="k" %a2) "enc"="{ if(a2!=0) evexr(d, a0, 1, 0, 0, 2, 2, 0, a2, 0) | 01011011 | 11 reg:3=d&7 rm:3=a0&7 }" "asm"="vcvttps2dq {a0}, {d} {lb}{a2}{rb}" "dclass"="zv" "feature"="avx512f" "nodecode" {
  %r = call <16 x i32> @llvm.x86.avx512.mask.cvttps2dq.512(<16 x float> %a0, <16 x i32> %a1, i16 %a2, i32 4)
  ret <16 x i32> %r
}

declare <16 x i32> @llvm.x86.avx512.mask.cvttps2dq.512(<16 x float>, <16 x i32>, i16, i32)

define <2 x i64> @X86.avx512.mask.cvttps2qq.128(<4 x float> "reg"="xv" %a0, <2 x i64> "reg"="xv" "tied" %a1, i8 "reg"="k" %a2) "enc"="{ if(a2!=0) evexr(d, a0, 1, 0, 0, 0, 1, 0, a2, 0) | 01111010 | 11 reg:3=d&7 rm:3=a0&7 }" "asm"="vcvttps2qq {a0}, {d} {lb}{a2}{rb}" "dclass"="xv" "feature"="avx512dq,avx512vl" "nodecode" {
  %r = call <2 x i64> @llvm.x86.avx512.mask.cvttps2qq.128(<4 x float> %a0, <2 x i64> %a1, i8 %a2)
  ret <2 x i64> %r
}

declare <2 x i64> @llvm.x86.avx512.mask.cvttps2qq.128(<4 x float>, <2 x i64>, i8)

define <4 x i64> @X86.avx512.mask.cvttps2qq.256(<4 x float> "reg"="xv" %a0, <4 x i64> "reg"="yv" "tied" %a1, i8 "reg"="k" %a2) "enc"="{ if(a2!=0) evexr(d, a0, 1, 0, 0, 1, 1, 0, a2, 0) | 01111010 | 11 reg:3=d&7 rm:3=a0&7 }" "asm"="vcvttps2qq {a0}, {d} {lb}{a2}{rb}" "dclass"="yv" "feature"="avx512dq,avx512vl" "nodecode" {
  %r = call <4 x i64> @llvm.x86.avx512.mask.cvttps2qq.256(<4 x float> %a0, <4 x i64> %a1, i8 %a2)
  ret <4 x i64> %r
}

declare <4 x i64> @llvm.x86.avx512.mask.cvttps2qq.256(<4 x float>, <4 x i64>, i8)

define <8 x i64> @X86.avx512.mask.cvttps2qq.512(<8 x float> "reg"="yv" %a0, <8 x i64> "reg"="zv" "tied" %a1, i8 "reg"="k" %a2) "enc"="{ if(a2!=0) evexr(d, a0, 1, 0, 0, 2, 1, 0, a2, 0) | 01111010 | 11 reg:3=d&7 rm:3=a0&7 }" "asm"="vcvttps2qq {a0}, {d} {lb}{a2}{rb}" "dclass"="zv" "feature"="avx512dq" "nodecode" {
  %r = call <8 x i64> @llvm.x86.avx512.mask.cvttps2qq.512(<8 x float> %a0, <8 x i64> %a1, i8 %a2, i32 4)
  ret <8 x i64> %r
}

declare <8 x i64> @llvm.x86.avx512.mask.cvttps2qq.512(<8 x float>, <8 x i64>, i8, i32)

define <4 x i32> @X86.avx512.mask.cvttps2udq.128(<4 x float> "reg"="xv" %a0, <4 x i32> "reg"="xv" "tied" %a1, i8 "reg"="k" %a2) "enc"="{ if(a2!=0) evexr(d, a0, 1, 0, 0, 0, 0, 0, a2, 0) | 01111000 | 11 reg:3=d&7 rm:3=a0&7 }" "asm"="vcvttps2udq {a0}, {d} {lb}{a2}{rb}" "dclass"="xv" "feature"="avx512f,avx512vl" "nodecode" {
  %r = call <4 x i32> @llvm.x86.avx512.mask.cvttps2udq.128(<4 x float> %a0, <4 x i32> %a1, i8 %a2)
  ret <4 x i32> %r
}

declare <4 x i32> @llvm.x86.avx512.mask.cvttps2udq.128(<4 x float>, <4 x i32>, i8)

define <8 x i32> @X86.avx512.mask.cvttps2udq.256(<8 x float> "reg"="yv" %a0, <8 x i32> "reg"="yv" "tied" %a1, i8 "reg"="k" %a2) "enc"="{ if(a2!=0) evexr(d, a0, 1, 0, 0, 1, 0, 0, a2, 0) | 01111000 | 11 reg:3=d&7 rm:3=a0&7 }" "asm"="vcvttps2udq {a0}, {d} {lb}{a2}{rb}" "dclass"="yv" "feature"="avx512f,avx512vl" "nodecode" {
  %r = call <8 x i32> @llvm.x86.avx512.mask.cvttps2udq.256(<8 x float> %a0, <8 x i32> %a1, i8 %a2)
  ret <8 x i32> %r
}

declare <8 x i32> @llvm.x86.avx512.mask.cvttps2udq.256(<8 x float>, <8 x i32>, i8)

define <16 x i32> @X86.avx512.mask.cvttps2udq.512(<16 x float> "reg"="zv" %a0, <16 x i32> "reg"="zv" "tied" %a1, i16 "reg"="k" %a2) "enc"="{ if(a2!=0) evexr(d, a0, 1, 0, 0, 2, 0, 0, a2, 0) | 01111000 | 11 reg:3=d&7 rm:3=a0&7 }" "asm"="vcvttps2udq {a0}, {d} {lb}{a2}{rb}" "dclass"="zv" "feature"="avx512f" "nodecode" {
  %r = call <16 x i32> @llvm.x86.avx512.mask.cvttps2udq.512(<16 x float> %a0, <16 x i32> %a1, i16 %a2, i32 4)
  ret <16 x i32> %r
}

declare <16 x i32> @llvm.x86.avx512.mask.cvttps2udq.512(<16 x float>, <16 x i32>, i16, i32)

define <2 x i64> @X86.avx512.mask.cvttps2uqq.128(<4 x float> "reg"="xv" %a0, <2 x i64> "reg"="xv" "tied" %a1, i8 "reg"="k" %a2) "enc"="{ if(a2!=0) evexr(d, a0, 1, 0, 0, 0, 1, 0, a2, 0) | 01111000 | 11 reg:3=d&7 rm:3=a0&7 }" "asm"="vcvttps2uqq {a0}, {d} {lb}{a2}{rb}" "dclass"="xv" "feature"="avx512dq,avx512vl" "nodecode" {
  %r = call <2 x i64> @llvm.x86.avx512.mask.cvttps2uqq.128(<4 x float> %a0, <2 x i64> %a1, i8 %a2)
  ret <2 x i64> %r
}

declare <2 x i64> @llvm.x86.avx512.mask.cvttps2uqq.128(<4 x float>, <2 x i64>, i8)

define <4 x i64> @X86.avx512.mask.cvttps2uqq.256(<4 x float> "reg"="xv" %a0, <4 x i64> "reg"="yv" "tied" %a1, i8 "reg"="k" %a2) "enc"="{ if(a2!=0) evexr(d, a0, 1, 0, 0, 1, 1, 0, a2, 0) | 01111000 | 11 reg:3=d&7 rm:3=a0&7 }" "asm"="vcvttps2uqq {a0}, {d} {lb}{a2}{rb}" "dclass"="yv" "feature"="avx512dq,avx512vl" "nodecode" {
  %r = call <4 x i64> @llvm.x86.avx512.mask.cvttps2uqq.256(<4 x float> %a0, <4 x i64> %a1, i8 %a2)
  ret <4 x i64> %r
}

declare <4 x i64> @llvm.x86.avx512.mask.cvttps2uqq.256(<4 x float>, <4 x i64>, i8)

define <8 x i64> @X86.avx512.mask.cvttps2uqq.512(<8 x float> "reg"="yv" %a0, <8 x i64> "reg"="zv" "tied" %a1, i8 "reg"="k" %a2) "enc"="{ if(a2!=0) evexr(d, a0, 1, 0, 0, 2, 1, 0, a2, 0) | 01111000 | 11 reg:3=d&7 rm:3=a0&7 }" "asm"="vcvttps2uqq {a0}, {d} {lb}{a2}{rb}" "dclass"="zv" "feature"="avx512dq" "nodecode" {
  %r = call <8 x i64> @llvm.x86.avx512.mask.cvttps2uqq.512(<8 x float> %a0, <8 x i64> %a1, i8 %a2, i32 4)
  ret <8 x i64> %r
}

declare <8 x i64> @llvm.x86.avx512.mask.cvttps2uqq.512(<8 x float>, <8 x i64>, i8, i32)

define <4 x float> @X86.avx512.mask.cvtuqq2ps.128(<2 x i64> "reg"="xv" %a0, <4 x float> "reg"="xv" "tied" %a1, i8 "reg"="k" %a2) "enc"="{ if(a2!=0) evexr(d, a0, 1, 1, 0, 0, 3, 0, a2, 0) | 01111010 | 11 reg:3=d&7 rm:3=a0&7 }" "asm"="vcvtuqq2ps {a0}, {d} {lb}{a2}{rb}" "dclass"="xv" "feature"="avx512dq,avx512vl" "nodecode" {
  %r = call <4 x float> @llvm.x86.avx512.mask.cvtuqq2ps.128(<2 x i64> %a0, <4 x float> %a1, i8 %a2)
  ret <4 x float> %r
}

declare <4 x float> @llvm.x86.avx512.mask.cvtuqq2ps.128(<2 x i64>, <4 x float>, i8)

define <2 x double> @X86.avx512.mask.div.sd.round(<2 x double> "reg"="xv" %a0, <2 x double> "reg"="xv" %a1, <2 x double> "reg"="xv" "tied" %a2, i8 "reg"="k" %a3) "enc"="{ if(a3!=0) evexr(d, a1, 1, 1, a0, 0, 3, 0, a3, 0) | 01011110 | 11 reg:3=d&7 rm:3=a1&7 }" "asm"="vdivsd {a1}, {a0}, {d} {lb}{a3}{rb}" "dclass"="xv" "feature"="avx512f" "nodecode" {
  %r = call <2 x double> @llvm.x86.avx512.mask.div.sd.round(<2 x double> %a0, <2 x double> %a1, <2 x double> %a2, i8 %a3, i32 4)
  ret <2 x double> %r
}

declare <2 x double> @llvm.x86.avx512.mask.div.sd.round(<2 x double>, <2 x double>, <2 x double>, i8, i32)

define <4 x float> @X86.avx512.mask.div.ss.round(<4 x float> "reg"="xv" %a0, <4 x float> "reg"="xv" %a1, <4 x float> "reg"="xv" "tied" %a2, i8 "reg"="k" %a3) "enc"="{ if(a3!=0) evexr(d, a1, 1, 0, a0, 0, 2, 0, a3, 0) | 01011110 | 11 reg:3=d&7 rm:3=a1&7 }" "asm"="vdivss {a1}, {a0}, {d} {lb}{a3}{rb}" "dclass"="xv" "feature"="avx512f" "nodecode" {
  %r = call <4 x float> @llvm.x86.avx512.mask.div.ss.round(<4 x float> %a0, <4 x float> %a1, <4 x float> %a2, i8 %a3, i32 4)
  ret <4 x float> %r
}

declare <4 x float> @llvm.x86.avx512.mask.div.ss.round(<4 x float>, <4 x float>, <4 x float>, i8, i32)

define <16 x i8> @X86.avx512.mask.expand.b.128(<16 x i8> "reg"="xv" %a0, <16 x i8> "reg"="xv" "tied" %a1, i16 "reg"="k" %a2) "enc"="{ if(a2!=0) evexr(d, a0, 2, 0, 0, 0, 1, 0, a2, 0) | 01100010 | 11 reg:3=d&7 rm:3=a0&7 }" "asm"="vpexpandb {a0}, {d} {lb}{a2}{rb}" "dclass"="xv" "feature"="avx512vbmi2,avx512vl" "nodecode" {
  %r = call <16 x i8> @llvm.x86.avx512.mask.expand.b.128(<16 x i8> %a0, <16 x i8> %a1, i16 %a2)
  ret <16 x i8> %r
}

declare <16 x i8> @llvm.x86.avx512.mask.expand.b.128(<16 x i8>, <16 x i8>, i16)

define <32 x i8> @X86.avx512.mask.expand.b.256(<32 x i8> "reg"="yv" %a0, <32 x i8> "reg"="yv" "tied" %a1, i32 "reg"="k" %a2) "enc"="{ if(a2!=0) evexr(d, a0, 2, 0, 0, 1, 1, 0, a2, 0) | 01100010 | 11 reg:3=d&7 rm:3=a0&7 }" "asm"="vpexpandb {a0}, {d} {lb}{a2}{rb}" "dclass"="yv" "feature"="avx512vbmi2,avx512vl" "nodecode" {
  %r = call <32 x i8> @llvm.x86.avx512.mask.expand.b.256(<32 x i8> %a0, <32 x i8> %a1, i32 %a2)
  ret <32 x i8> %r
}

declare <32 x i8> @llvm.x86.avx512.mask.expand.b.256(<32 x i8>, <32 x i8>, i32)

define <64 x i8> @X86.avx512.mask.expand.b.512(<64 x i8> "reg"="zv" %a0, <64 x i8> "reg"="zv" "tied" %a1, i64 "reg"="k" %a2) "enc"="{ if(a2!=0) evexr(d, a0, 2, 0, 0, 2, 1, 0, a2, 0) | 01100010 | 11 reg:3=d&7 rm:3=a0&7 }" "asm"="vpexpandb {a0}, {d} {lb}{a2}{rb}" "dclass"="zv" "feature"="avx512vbmi2" "nodecode" {
  %r = call <64 x i8> @llvm.x86.avx512.mask.expand.b.512(<64 x i8> %a0, <64 x i8> %a1, i64 %a2)
  ret <64 x i8> %r
}

declare <64 x i8> @llvm.x86.avx512.mask.expand.b.512(<64 x i8>, <64 x i8>, i64)

define <4 x i32> @X86.avx512.mask.expand.d.128(<4 x i32> "reg"="xv" %a0, <4 x i32> "reg"="xv" "tied" %a1, i8 "reg"="k" %a2) "enc"="{ if(a2!=0) evexr(d, a0, 2, 0, 0, 0, 1, 0, a2, 0) | 10001001 | 11 reg:3=d&7 rm:3=a0&7 }" "asm"="vpexpandd {a0}, {d} {lb}{a2}{rb}" "dclass"="xv" "feature"="avx512f,avx512vl" "nodecode" {
  %r = call <4 x i32> @llvm.x86.avx512.mask.expand.d.128(<4 x i32> %a0, <4 x i32> %a1, i8 %a2)
  ret <4 x i32> %r
}

declare <4 x i32> @llvm.x86.avx512.mask.expand.d.128(<4 x i32>, <4 x i32>, i8)

define <8 x i32> @X86.avx512.mask.expand.d.256(<8 x i32> "reg"="yv" %a0, <8 x i32> "reg"="yv" "tied" %a1, i8 "reg"="k" %a2) "enc"="{ if(a2!=0) evexr(d, a0, 2, 0, 0, 1, 1, 0, a2, 0) | 10001001 | 11 reg:3=d&7 rm:3=a0&7 }" "asm"="vpexpandd {a0}, {d} {lb}{a2}{rb}" "dclass"="yv" "feature"="avx512f,avx512vl" "nodecode" {
  %r = call <8 x i32> @llvm.x86.avx512.mask.expand.d.256(<8 x i32> %a0, <8 x i32> %a1, i8 %a2)
  ret <8 x i32> %r
}

declare <8 x i32> @llvm.x86.avx512.mask.expand.d.256(<8 x i32>, <8 x i32>, i8)

define <16 x i32> @X86.avx512.mask.expand.d.512(<16 x i32> "reg"="zv" %a0, <16 x i32> "reg"="zv" "tied" %a1, i16 "reg"="k" %a2) "enc"="{ if(a2!=0) evexr(d, a0, 2, 0, 0, 2, 1, 0, a2, 0) | 10001001 | 11 reg:3=d&7 rm:3=a0&7 }" "asm"="vpexpandd {a0}, {d} {lb}{a2}{rb}" "dclass"="zv" "feature"="avx512f" "nodecode" {
  %r = call <16 x i32> @llvm.x86.avx512.mask.expand.d.512(<16 x i32> %a0, <16 x i32> %a1, i16 %a2)
  ret <16 x i32> %r
}

declare <16 x i32> @llvm.x86.avx512.mask.expand.d.512(<16 x i32>, <16 x i32>, i16)

define <16 x i8> @X86.avx512.mask.expand.load.b.128(ptr "reg"="r64" %a0, <16 x i8> "reg"="xv" "tied" %a1, i16 "reg"="k" %a2) "enc"="{ if(a2!=0) evexm(d, a0, 4, 2, 0, 0, 0, 1, 0, a2, 0) | 01100010 | rm(d, a0, 4, 1, 0, 0, 1) }" "asm"="vpexpandb 0({a0}), {d} {lb}{a2}{rb}" "dclass"="xv" "feature"="avx512vbmi2,avx512vl" "nodecode" {
  %r = call <16 x i8> @llvm.x86.avx512.mask.expand.load.b.128(ptr %a0, <16 x i8> %a1, i16 %a2)
  ret <16 x i8> %r
}

declare <16 x i8> @llvm.x86.avx512.mask.expand.load.b.128(ptr, <16 x i8>, i16)

define <32 x i8> @X86.avx512.mask.expand.load.b.256(ptr "reg"="r64" %a0, <32 x i8> "reg"="yv" "tied" %a1, i32 "reg"="k" %a2) "enc"="{ if(a2!=0) evexm(d, a0, 4, 2, 0, 0, 1, 1, 0, a2, 0) | 01100010 | rm(d, a0, 4, 1, 0, 0, 1) }" "asm"="vpexpandb 0({a0}), {d} {lb}{a2}{rb}" "dclass"="yv" "feature"="avx512vbmi2,avx512vl" "nodecode" {
  %r = call <32 x i8> @llvm.x86.avx512.mask.expand.load.b.256(ptr %a0, <32 x i8> %a1, i32 %a2)
  ret <32 x i8> %r
}

declare <32 x i8> @llvm.x86.avx512.mask.expand.load.b.256(ptr, <32 x i8>, i32)

define <64 x i8> @X86.avx512.mask.expand.load.b.512(ptr "reg"="r64" %a0, <64 x i8> "reg"="zv" "tied" %a1, i64 "reg"="k" %a2) "enc"="{ if(a2!=0) evexm(d, a0, 4, 2, 0, 0, 2, 1, 0, a2, 0) | 01100010 | rm(d, a0, 4, 1, 0, 0, 1) }" "asm"="vpexpandb 0({a0}), {d} {lb}{a2}{rb}" "dclass"="zv" "feature"="avx512vbmi2" "nodecode" {
  %r = call <64 x i8> @llvm.x86.avx512.mask.expand.load.b.512(ptr %a0, <64 x i8> %a1, i64 %a2)
  ret <64 x i8> %r
}

declare <64 x i8> @llvm.x86.avx512.mask.expand.load.b.512(ptr, <64 x i8>, i64)

define <4 x i32> @X86.avx512.mask.expand.load.d.128(ptr "reg"="r64" %a0, <4 x i32> "reg"="xv" "tied" %a1, i8 "reg"="k" %a2) "enc"="{ if(a2!=0) evexm(d, a0, 4, 2, 0, 0, 0, 1, 0, a2, 0) | 10001001 | rm(d, a0, 4, 1, 0, 0, 4) }" "asm"="vpexpandd 0({a0}), {d} {lb}{a2}{rb}" "dclass"="xv" "feature"="avx512f,avx512vl" "nodecode" {
  %r = call <4 x i32> @llvm.x86.avx512.mask.expand.load.d.128(ptr %a0, <4 x i32> %a1, i8 %a2)
  ret <4 x i32> %r
}

declare <4 x i32> @llvm.x86.avx512.mask.expand.load.d.128(ptr, <4 x i32>, i8)

define <8 x i32> @X86.avx512.mask.expand.load.d.256(ptr "reg"="r64" %a0, <8 x i32> "reg"="yv" "tied" %a1, i8 "reg"="k" %a2) "enc"="{ if(a2!=0) evexm(d, a0, 4, 2, 0, 0, 1, 1, 0, a2, 0) | 10001001 | rm(d, a0, 4, 1, 0, 0, 4) }" "asm"="vpexpandd 0({a0}), {d} {lb}{a2}{rb}" "dclass"="yv" "feature"="avx512f,avx512vl" "nodecode" {
  %r = call <8 x i32> @llvm.x86.avx512.mask.expand.load.d.256(ptr %a0, <8 x i32> %a1, i8 %a2)
  ret <8 x i32> %r
}

declare <8 x i32> @llvm.x86.avx512.mask.expand.load.d.256(ptr, <8 x i32>, i8)

define <16 x i32> @X86.avx512.mask.expand.load.d.512(ptr "reg"="r64" %a0, <16 x i32> "reg"="zv" "tied" %a1, i16 "reg"="k" %a2) "enc"="{ if(a2!=0) evexm(d, a0, 4, 2, 0, 0, 2, 1, 0, a2, 0) | 10001001 | rm(d, a0, 4, 1, 0, 0, 4) }" "asm"="vpexpandd 0({a0}), {d} {lb}{a2}{rb}" "dclass"="zv" "feature"="avx512f" "nodecode" {
  %r = call <16 x i32> @llvm.x86.avx512.mask.expand.load.d.512(ptr %a0, <16 x i32> %a1, i16 %a2)
  ret <16 x i32> %r
}

declare <16 x i32> @llvm.x86.avx512.mask.expand.load.d.512(ptr, <16 x i32>, i16)

define <2 x double> @X86.avx512.mask.expand.load.pd.128(ptr "reg"="r64" %a0, <2 x double> "reg"="xv" "tied" %a1, i8 "reg"="k" %a2) "enc"="{ if(a2!=0) evexm(d, a0, 4, 2, 1, 0, 0, 1, 0, a2, 0) | 10001000 | rm(d, a0, 4, 1, 0, 0, 8) }" "asm"="vexpandpd 0({a0}), {d} {lb}{a2}{rb}" "dclass"="xv" "feature"="avx512f,avx512vl" "nodecode" {
  %r = call <2 x double> @llvm.x86.avx512.mask.expand.load.pd.128(ptr %a0, <2 x double> %a1, i8 %a2)
  ret <2 x double> %r
}

declare <2 x double> @llvm.x86.avx512.mask.expand.load.pd.128(ptr, <2 x double>, i8)

define <4 x double> @X86.avx512.mask.expand.load.pd.256(ptr "reg"="r64" %a0, <4 x double> "reg"="yv" "tied" %a1, i8 "reg"="k" %a2) "enc"="{ if(a2!=0) evexm(d, a0, 4, 2, 1, 0, 1, 1, 0, a2, 0) | 10001000 | rm(d, a0, 4, 1, 0, 0, 8) }" "asm"="vexpandpd 0({a0}), {d} {lb}{a2}{rb}" "dclass"="yv" "feature"="avx512f,avx512vl" "nodecode" {
  %r = call <4 x double> @llvm.x86.avx512.mask.expand.load.pd.256(ptr %a0, <4 x double> %a1, i8 %a2)
  ret <4 x double> %r
}

declare <4 x double> @llvm.x86.avx512.mask.expand.load.pd.256(ptr, <4 x double>, i8)

define <8 x double> @X86.avx512.mask.expand.load.pd.512(ptr "reg"="r64" %a0, <8 x double> "reg"="zv" "tied" %a1, i8 "reg"="k" %a2) "enc"="{ if(a2!=0) evexm(d, a0, 4, 2, 1, 0, 2, 1, 0, a2, 0) | 10001000 | rm(d, a0, 4, 1, 0, 0, 8) }" "asm"="vexpandpd 0({a0}), {d} {lb}{a2}{rb}" "dclass"="zv" "feature"="avx512f" "nodecode" {
  %r = call <8 x double> @llvm.x86.avx512.mask.expand.load.pd.512(ptr %a0, <8 x double> %a1, i8 %a2)
  ret <8 x double> %r
}

declare <8 x double> @llvm.x86.avx512.mask.expand.load.pd.512(ptr, <8 x double>, i8)

define <4 x float> @X86.avx512.mask.expand.load.ps.128(ptr "reg"="r64" %a0, <4 x float> "reg"="xv" "tied" %a1, i8 "reg"="k" %a2) "enc"="{ if(a2!=0) evexm(d, a0, 4, 2, 0, 0, 0, 1, 0, a2, 0) | 10001000 | rm(d, a0, 4, 1, 0, 0, 4) }" "asm"="vexpandps 0({a0}), {d} {lb}{a2}{rb}" "dclass"="xv" "feature"="avx512f,avx512vl" "nodecode" {
  %r = call <4 x float> @llvm.x86.avx512.mask.expand.load.ps.128(ptr %a0, <4 x float> %a1, i8 %a2)
  ret <4 x float> %r
}

declare <4 x float> @llvm.x86.avx512.mask.expand.load.ps.128(ptr, <4 x float>, i8)

define <8 x float> @X86.avx512.mask.expand.load.ps.256(ptr "reg"="r64" %a0, <8 x float> "reg"="yv" "tied" %a1, i8 "reg"="k" %a2) "enc"="{ if(a2!=0) evexm(d, a0, 4, 2, 0, 0, 1, 1, 0, a2, 0) | 10001000 | rm(d, a0, 4, 1, 0, 0, 4) }" "asm"="vexpandps 0({a0}), {d} {lb}{a2}{rb}" "dclass"="yv" "feature"="avx512f,avx512vl" "nodecode" {
  %r = call <8 x float> @llvm.x86.avx512.mask.expand.load.ps.256(ptr %a0, <8 x float> %a1, i8 %a2)
  ret <8 x float> %r
}

declare <8 x float> @llvm.x86.avx512.mask.expand.load.ps.256(ptr, <8 x float>, i8)

define <16 x float> @X86.avx512.mask.expand.load.ps.512(ptr "reg"="r64" %a0, <16 x float> "reg"="zv" "tied" %a1, i16 "reg"="k" %a2) "enc"="{ if(a2!=0) evexm(d, a0, 4, 2, 0, 0, 2, 1, 0, a2, 0) | 10001000 | rm(d, a0, 4, 1, 0, 0, 4) }" "asm"="vexpandps 0({a0}), {d} {lb}{a2}{rb}" "dclass"="zv" "feature"="avx512f" "nodecode" {
  %r = call <16 x float> @llvm.x86.avx512.mask.expand.load.ps.512(ptr %a0, <16 x float> %a1, i16 %a2)
  ret <16 x float> %r
}

declare <16 x float> @llvm.x86.avx512.mask.expand.load.ps.512(ptr, <16 x float>, i16)

define <2 x i64> @X86.avx512.mask.expand.load.q.128(ptr "reg"="r64" %a0, <2 x i64> "reg"="xv" "tied" %a1, i8 "reg"="k" %a2) "enc"="{ if(a2!=0) evexm(d, a0, 4, 2, 1, 0, 0, 1, 0, a2, 0) | 10001001 | rm(d, a0, 4, 1, 0, 0, 8) }" "asm"="vpexpandq 0({a0}), {d} {lb}{a2}{rb}" "dclass"="xv" "feature"="avx512f,avx512vl" "nodecode" {
  %r = call <2 x i64> @llvm.x86.avx512.mask.expand.load.q.128(ptr %a0, <2 x i64> %a1, i8 %a2)
  ret <2 x i64> %r
}

declare <2 x i64> @llvm.x86.avx512.mask.expand.load.q.128(ptr, <2 x i64>, i8)

define <4 x i64> @X86.avx512.mask.expand.load.q.256(ptr "reg"="r64" %a0, <4 x i64> "reg"="yv" "tied" %a1, i8 "reg"="k" %a2) "enc"="{ if(a2!=0) evexm(d, a0, 4, 2, 1, 0, 1, 1, 0, a2, 0) | 10001001 | rm(d, a0, 4, 1, 0, 0, 8) }" "asm"="vpexpandq 0({a0}), {d} {lb}{a2}{rb}" "dclass"="yv" "feature"="avx512f,avx512vl" "nodecode" {
  %r = call <4 x i64> @llvm.x86.avx512.mask.expand.load.q.256(ptr %a0, <4 x i64> %a1, i8 %a2)
  ret <4 x i64> %r
}

declare <4 x i64> @llvm.x86.avx512.mask.expand.load.q.256(ptr, <4 x i64>, i8)

define <8 x i64> @X86.avx512.mask.expand.load.q.512(ptr "reg"="r64" %a0, <8 x i64> "reg"="zv" "tied" %a1, i8 "reg"="k" %a2) "enc"="{ if(a2!=0) evexm(d, a0, 4, 2, 1, 0, 2, 1, 0, a2, 0) | 10001001 | rm(d, a0, 4, 1, 0, 0, 8) }" "asm"="vpexpandq 0({a0}), {d} {lb}{a2}{rb}" "dclass"="zv" "feature"="avx512f" "nodecode" {
  %r = call <8 x i64> @llvm.x86.avx512.mask.expand.load.q.512(ptr %a0, <8 x i64> %a1, i8 %a2)
  ret <8 x i64> %r
}

declare <8 x i64> @llvm.x86.avx512.mask.expand.load.q.512(ptr, <8 x i64>, i8)

define <8 x i16> @X86.avx512.mask.expand.load.w.128(ptr "reg"="r64" %a0, <8 x i16> "reg"="xv" "tied" %a1, i8 "reg"="k" %a2) "enc"="{ if(a2!=0) evexm(d, a0, 4, 2, 1, 0, 0, 1, 0, a2, 0) | 01100010 | rm(d, a0, 4, 1, 0, 0, 2) }" "asm"="vpexpandw 0({a0}), {d} {lb}{a2}{rb}" "dclass"="xv" "feature"="avx512vbmi2,avx512vl" "nodecode" {
  %r = call <8 x i16> @llvm.x86.avx512.mask.expand.load.w.128(ptr %a0, <8 x i16> %a1, i8 %a2)
  ret <8 x i16> %r
}

declare <8 x i16> @llvm.x86.avx512.mask.expand.load.w.128(ptr, <8 x i16>, i8)

define <16 x i16> @X86.avx512.mask.expand.load.w.256(ptr "reg"="r64" %a0, <16 x i16> "reg"="yv" "tied" %a1, i16 "reg"="k" %a2) "enc"="{ if(a2!=0) evexm(d, a0, 4, 2, 1, 0, 1, 1, 0, a2, 0) | 01100010 | rm(d, a0, 4, 1, 0, 0, 2) }" "asm"="vpexpandw 0({a0}), {d} {lb}{a2}{rb}" "dclass"="yv" "feature"="avx512vbmi2,avx512vl" "nodecode" {
  %r = call <16 x i16> @llvm.x86.avx512.mask.expand.load.w.256(ptr %a0, <16 x i16> %a1, i16 %a2)
  ret <16 x i16> %r
}

declare <16 x i16> @llvm.x86.avx512.mask.expand.load.w.256(ptr, <16 x i16>, i16)

define <32 x i16> @X86.avx512.mask.expand.load.w.512(ptr "reg"="r64" %a0, <32 x i16> "reg"="zv" "tied" %a1, i32 "reg"="k" %a2) "enc"="{ if(a2!=0) evexm(d, a0, 4, 2, 1, 0, 2, 1, 0, a2, 0) | 01100010 | rm(d, a0, 4, 1, 0, 0, 2) }" "asm"="vpexpandw 0({a0}), {d} {lb}{a2}{rb}" "dclass"="zv" "feature"="avx512vbmi2" "nodecode" {
  %r = call <32 x i16> @llvm.x86.avx512.mask.expand.load.w.512(ptr %a0, <32 x i16> %a1, i32 %a2)
  ret <32 x i16> %r
}

declare <32 x i16> @llvm.x86.avx512.mask.expand.load.w.512(ptr, <32 x i16>, i32)

define <2 x double> @X86.avx512.mask.expand.pd.128(<2 x double> "reg"="xv" %a0, <2 x double> "reg"="xv" "tied" %a1, i8 "reg"="k" %a2) "enc"="{ if(a2!=0) evexr(d, a0, 2, 1, 0, 0, 1, 0, a2, 0) | 10001000 | 11 reg:3=d&7 rm:3=a0&7 }" "asm"="vexpandpd {a0}, {d} {lb}{a2}{rb}" "dclass"="xv" "feature"="avx512f,avx512vl" "nodecode" {
  %r = call <2 x double> @llvm.x86.avx512.mask.expand.pd.128(<2 x double> %a0, <2 x double> %a1, i8 %a2)
  ret <2 x double> %r
}

declare <2 x double> @llvm.x86.avx512.mask.expand.pd.128(<2 x double>, <2 x double>, i8)

define <4 x double> @X86.avx512.mask.expand.pd.256(<4 x double> "reg"="yv" %a0, <4 x double> "reg"="yv" "tied" %a1, i8 "reg"="k" %a2) "enc"="{ if(a2!=0) evexr(d, a0, 2, 1, 0, 1, 1, 0, a2, 0) | 10001000 | 11 reg:3=d&7 rm:3=a0&7 }" "asm"="vexpandpd {a0}, {d} {lb}{a2}{rb}" "dclass"="yv" "feature"="avx512f,avx512vl" "nodecode" {
  %r = call <4 x double> @llvm.x86.avx512.mask.expand.pd.256(<4 x double> %a0, <4 x double> %a1, i8 %a2)
  ret <4 x double> %r
}

declare <4 x double> @llvm.x86.avx512.mask.expand.pd.256(<4 x double>, <4 x double>, i8)

define <8 x double> @X86.avx512.mask.expand.pd.512(<8 x double> "reg"="zv" %a0, <8 x double> "reg"="zv" "tied" %a1, i8 "reg"="k" %a2) "enc"="{ if(a2!=0) evexr(d, a0, 2, 1, 0, 2, 1, 0, a2, 0) | 10001000 | 11 reg:3=d&7 rm:3=a0&7 }" "asm"="vexpandpd {a0}, {d} {lb}{a2}{rb}" "dclass"="zv" "feature"="avx512f" "nodecode" {
  %r = call <8 x double> @llvm.x86.avx512.mask.expand.pd.512(<8 x double> %a0, <8 x double> %a1, i8 %a2)
  ret <8 x double> %r
}

declare <8 x double> @llvm.x86.avx512.mask.expand.pd.512(<8 x double>, <8 x double>, i8)

define <4 x float> @X86.avx512.mask.expand.ps.128(<4 x float> "reg"="xv" %a0, <4 x float> "reg"="xv" "tied" %a1, i8 "reg"="k" %a2) "enc"="{ if(a2!=0) evexr(d, a0, 2, 0, 0, 0, 1, 0, a2, 0) | 10001000 | 11 reg:3=d&7 rm:3=a0&7 }" "asm"="vexpandps {a0}, {d} {lb}{a2}{rb}" "dclass"="xv" "feature"="avx512f,avx512vl" "nodecode" {
  %r = call <4 x float> @llvm.x86.avx512.mask.expand.ps.128(<4 x float> %a0, <4 x float> %a1, i8 %a2)
  ret <4 x float> %r
}

declare <4 x float> @llvm.x86.avx512.mask.expand.ps.128(<4 x float>, <4 x float>, i8)

define <8 x float> @X86.avx512.mask.expand.ps.256(<8 x float> "reg"="yv" %a0, <8 x float> "reg"="yv" "tied" %a1, i8 "reg"="k" %a2) "enc"="{ if(a2!=0) evexr(d, a0, 2, 0, 0, 1, 1, 0, a2, 0) | 10001000 | 11 reg:3=d&7 rm:3=a0&7 }" "asm"="vexpandps {a0}, {d} {lb}{a2}{rb}" "dclass"="yv" "feature"="avx512f,avx512vl" "nodecode" {
  %r = call <8 x float> @llvm.x86.avx512.mask.expand.ps.256(<8 x float> %a0, <8 x float> %a1, i8 %a2)
  ret <8 x float> %r
}

declare <8 x float> @llvm.x86.avx512.mask.expand.ps.256(<8 x float>, <8 x float>, i8)

define <16 x float> @X86.avx512.mask.expand.ps.512(<16 x float> "reg"="zv" %a0, <16 x float> "reg"="zv" "tied" %a1, i16 "reg"="k" %a2) "enc"="{ if(a2!=0) evexr(d, a0, 2, 0, 0, 2, 1, 0, a2, 0) | 10001000 | 11 reg:3=d&7 rm:3=a0&7 }" "asm"="vexpandps {a0}, {d} {lb}{a2}{rb}" "dclass"="zv" "feature"="avx512f" "nodecode" {
  %r = call <16 x float> @llvm.x86.avx512.mask.expand.ps.512(<16 x float> %a0, <16 x float> %a1, i16 %a2)
  ret <16 x float> %r
}

declare <16 x float> @llvm.x86.avx512.mask.expand.ps.512(<16 x float>, <16 x float>, i16)

define <2 x i64> @X86.avx512.mask.expand.q.128(<2 x i64> "reg"="xv" %a0, <2 x i64> "reg"="xv" "tied" %a1, i8 "reg"="k" %a2) "enc"="{ if(a2!=0) evexr(d, a0, 2, 1, 0, 0, 1, 0, a2, 0) | 10001001 | 11 reg:3=d&7 rm:3=a0&7 }" "asm"="vpexpandq {a0}, {d} {lb}{a2}{rb}" "dclass"="xv" "feature"="avx512f,avx512vl" "nodecode" {
  %r = call <2 x i64> @llvm.x86.avx512.mask.expand.q.128(<2 x i64> %a0, <2 x i64> %a1, i8 %a2)
  ret <2 x i64> %r
}

declare <2 x i64> @llvm.x86.avx512.mask.expand.q.128(<2 x i64>, <2 x i64>, i8)

define <4 x i64> @X86.avx512.mask.expand.q.256(<4 x i64> "reg"="yv" %a0, <4 x i64> "reg"="yv" "tied" %a1, i8 "reg"="k" %a2) "enc"="{ if(a2!=0) evexr(d, a0, 2, 1, 0, 1, 1, 0, a2, 0) | 10001001 | 11 reg:3=d&7 rm:3=a0&7 }" "asm"="vpexpandq {a0}, {d} {lb}{a2}{rb}" "dclass"="yv" "feature"="avx512f,avx512vl" "nodecode" {
  %r = call <4 x i64> @llvm.x86.avx512.mask.expand.q.256(<4 x i64> %a0, <4 x i64> %a1, i8 %a2)
  ret <4 x i64> %r
}

declare <4 x i64> @llvm.x86.avx512.mask.expand.q.256(<4 x i64>, <4 x i64>, i8)

define <8 x i64> @X86.avx512.mask.expand.q.512(<8 x i64> "reg"="zv" %a0, <8 x i64> "reg"="zv" "tied" %a1, i8 "reg"="k" %a2) "enc"="{ if(a2!=0) evexr(d, a0, 2, 1, 0, 2, 1, 0, a2, 0) | 10001001 | 11 reg:3=d&7 rm:3=a0&7 }" "asm"="vpexpandq {a0}, {d} {lb}{a2}{rb}" "dclass"="zv" "feature"="avx512f" "nodecode" {
  %r = call <8 x i64> @llvm.x86.avx512.mask.expand.q.512(<8 x i64> %a0, <8 x i64> %a1, i8 %a2)
  ret <8 x i64> %r
}

declare <8 x i64> @llvm.x86.avx512.mask.expand.q.512(<8 x i64>, <8 x i64>, i8)

define <8 x i16> @X86.avx512.mask.expand.w.128(<8 x i16> "reg"="xv" %a0, <8 x i16> "reg"="xv" "tied" %a1, i8 "reg"="k" %a2) "enc"="{ if(a2!=0) evexr(d, a0, 2, 1, 0, 0, 1, 0, a2, 0) | 01100010 | 11 reg:3=d&7 rm:3=a0&7 }" "asm"="vpexpandw {a0}, {d} {lb}{a2}{rb}" "dclass"="xv" "feature"="avx512vbmi2,avx512vl" "nodecode" {
  %r = call <8 x i16> @llvm.x86.avx512.mask.expand.w.128(<8 x i16> %a0, <8 x i16> %a1, i8 %a2)
  ret <8 x i16> %r
}

declare <8 x i16> @llvm.x86.avx512.mask.expand.w.128(<8 x i16>, <8 x i16>, i8)

define <16 x i16> @X86.avx512.mask.expand.w.256(<16 x i16> "reg"="yv" %a0, <16 x i16> "reg"="yv" "tied" %a1, i16 "reg"="k" %a2) "enc"="{ if(a2!=0) evexr(d, a0, 2, 1, 0, 1, 1, 0, a2, 0) | 01100010 | 11 reg:3=d&7 rm:3=a0&7 }" "asm"="vpexpandw {a0}, {d} {lb}{a2}{rb}" "dclass"="yv" "feature"="avx512vbmi2,avx512vl" "nodecode" {
  %r = call <16 x i16> @llvm.x86.avx512.mask.expand.w.256(<16 x i16> %a0, <16 x i16> %a1, i16 %a2)
  ret <16 x i16> %r
}

declare <16 x i16> @llvm.x86.avx512.mask.expand.w.256(<16 x i16>, <16 x i16>, i16)

define <32 x i16> @X86.avx512.mask.expand.w.512(<32 x i16> "reg"="zv" %a0, <32 x i16> "reg"="zv" "tied" %a1, i32 "reg"="k" %a2) "enc"="{ if(a2!=0) evexr(d, a0, 2, 1, 0, 2, 1, 0, a2, 0) | 01100010 | 11 reg:3=d&7 rm:3=a0&7 }" "asm"="vpexpandw {a0}, {d} {lb}{a2}{rb}" "dclass"="zv" "feature"="avx512vbmi2" "nodecode" {
  %r = call <32 x i16> @llvm.x86.avx512.mask.expand.w.512(<32 x i16> %a0, <32 x i16> %a1, i32 %a2)
  ret <32 x i16> %r
}

declare <32 x i16> @llvm.x86.avx512.mask.expand.w.512(<32 x i16>, <32 x i16>, i32)

define <2 x double> @X86.avx512.mask.fixupimm.pd.128(<2 x double> "reg"="xv" "tied" %a0, <2 x double> "reg"="xv" %a1, <2 x i64> "reg"="xv" %a2, i32 "imm" %a3, i8 "reg"="k" %a4) "enc"="{ if(a4!=0) evexr(d, a2, 3, 1, a1, 0, 1, 0, a4, 0) | 01010100 | 11 reg:3=d&7 rm:3=a2&7 | { i1:8=a3 ; i1:s8=a3 } }" "asm"="vfixupimmpd ${a3}, {a2}, {a1}, {d} {lb}{a4}{rb}" "dclass"="xv" "feature"="avx512f,avx512vl" "nodecode" {
  %r = call <2 x double> @llvm.x86.avx512.mask.fixupimm.pd.128(<2 x double> %a0, <2 x double> %a1, <2 x i64> %a2, i32 %a3, i8 %a4)
  ret <2 x double> %r
}

declare <2 x double> @llvm.x86.avx512.mask.fixupimm.pd.128(<2 x double>, <2 x double>, <2 x i64>, i32, i8)

define <4 x double> @X86.avx512.mask.fixupimm.pd.256(<4 x double> "reg"="yv" "tied" %a0, <4 x double> "reg"="yv" %a1, <4 x i64> "reg"="yv" %a2, i32 "imm" %a3, i8 "reg"="k" %a4) "enc"="{ if(a4!=0) evexr(d, a2, 3, 1, a1, 1, 1, 0, a4, 0) | 01010100 | 11 reg:3=d&7 rm:3=a2&7 | { i1:8=a3 ; i1:s8=a3 } }" "asm"="vfixupimmpd ${a3}, {a2}, {a1}, {d} {lb}{a4}{rb}" "dclass"="yv" "feature"="avx512f,avx512vl" "nodecode" {
  %r = call <4 x double> @llvm.x86.avx512.mask.fixupimm.pd.256(<4 x double> %a0, <4 x double> %a1, <4 x i64> %a2, i32 %a3, i8 %a4)
  ret <4 x double> %r
}

declare <4 x double> @llvm.x86.avx512.mask.fixupimm.pd.256(<4 x double>, <4 x double>, <4 x i64>, i32, i8)

define <8 x double> @X86.avx512.mask.fixupimm.pd.512(<8 x double> "reg"="zv" "tied" %a0, <8 x double> "reg"="zv" %a1, <8 x i64> "reg"="zv" %a2, i32 "imm" %a3, i8 "reg"="k" %a4) "enc"="{ if(a4!=0) evexr(d, a2, 3, 1, a1, 2, 1, 0, a4, 0) | 01010100 | 11 reg:3=d&7 rm:3=a2&7 | { i1:8=a3 ; i1:s8=a3 } }" "asm"="vfixupimmpd ${a3}, {a2}, {a1}, {d} {lb}{a4}{rb}" "dclass"="zv" "feature"="avx512f" "nodecode" {
  %r = call <8 x double> @llvm.x86.avx512.mask.fixupimm.pd.512(<8 x double> %a0, <8 x double> %a1, <8 x i64> %a2, i32 %a3, i8 %a4, i32 4)
  ret <8 x double> %r
}

declare <8 x double> @llvm.x86.avx512.mask.fixupimm.pd.512(<8 x double>, <8 x double>, <8 x i64>, i32, i8, i32)

define <4 x float> @X86.avx512.mask.fixupimm.ps.128(<4 x float> "reg"="xv" "tied" %a0, <4 x float> "reg"="xv" %a1, <4 x i32> "reg"="xv" %a2, i32 "imm" %a3, i8 "reg"="k" %a4) "enc"="{ if(a4!=0) evexr(d, a2, 3, 0, a1, 0, 1, 0, a4, 0) | 01010100 | 11 reg:3=d&7 rm:3=a2&7 | { i1:8=a3 ; i1:s8=a3 } }" "asm"="vfixupimmps ${a3}, {a2}, {a1}, {d} {lb}{a4}{rb}" "dclass"="xv" "feature"="avx512f,avx512vl" "nodecode" {
  %r = call <4 x float> @llvm.x86.avx512.mask.fixupimm.ps.128(<4 x float> %a0, <4 x float> %a1, <4 x i32> %a2, i32 %a3, i8 %a4)
  ret <4 x float> %r
}

declare <4 x float> @llvm.x86.avx512.mask.fixupimm.ps.128(<4 x float>, <4 x float>, <4 x i32>, i32, i8)

define <8 x float> @X86.avx512.mask.fixupimm.ps.256(<8 x float> "reg"="yv" "tied" %a0, <8 x float> "reg"="yv" %a1, <8 x i32> "reg"="yv" %a2, i32 "imm" %a3, i8 "reg"="k" %a4) "enc"="{ if(a4!=0) evexr(d, a2, 3, 0, a1, 1, 1, 0, a4, 0) | 01010100 | 11 reg:3=d&7 rm:3=a2&7 | { i1:8=a3 ; i1:s8=a3 } }" "asm"="vfixupimmps ${a3}, {a2}, {a1}, {d} {lb}{a4}{rb}" "dclass"="yv" "feature"="avx512f,avx512vl" "nodecode" {
  %r = call <8 x float> @llvm.x86.avx512.mask.fixupimm.ps.256(<8 x float> %a0, <8 x float> %a1, <8 x i32> %a2, i32 %a3, i8 %a4)
  ret <8 x float> %r
}

declare <8 x float> @llvm.x86.avx512.mask.fixupimm.ps.256(<8 x float>, <8 x float>, <8 x i32>, i32, i8)

define <16 x float> @X86.avx512.mask.fixupimm.ps.512(<16 x float> "reg"="zv" "tied" %a0, <16 x float> "reg"="zv" %a1, <16 x i32> "reg"="zv" %a2, i32 "imm" %a3, i16 "reg"="k" %a4) "enc"="{ if(a4!=0) evexr(d, a2, 3, 0, a1, 2, 1, 0, a4, 0) | 01010100 | 11 reg:3=d&7 rm:3=a2&7 | { i1:8=a3 ; i1:s8=a3 } }" "asm"="vfixupimmps ${a3}, {a2}, {a1}, {d} {lb}{a4}{rb}" "dclass"="zv" "feature"="avx512f" "nodecode" {
  %r = call <16 x float> @llvm.x86.avx512.mask.fixupimm.ps.512(<16 x float> %a0, <16 x float> %a1, <16 x i32> %a2, i32 %a3, i16 %a4, i32 4)
  ret <16 x float> %r
}

declare <16 x float> @llvm.x86.avx512.mask.fixupimm.ps.512(<16 x float>, <16 x float>, <16 x i32>, i32, i16, i32)

define <2 x double> @X86.avx512.mask.fixupimm.sd(<2 x double> "reg"="xv" "tied" %a0, <2 x double> "reg"="xv" %a1, <2 x i64> "reg"="xv" %a2, i32 "imm" %a3, i8 "reg"="k" %a4) "enc"="{ if(a4!=0) evexr(d, a2, 3, 1, a1, 0, 1, 0, a4, 0) | 01010101 | 11 reg:3=d&7 rm:3=a2&7 | { i1:8=a3 ; i1:s8=a3 } }" "asm"="vfixupimmsd ${a3}, {a2}, {a1}, {d} {lb}{a4}{rb}" "dclass"="xv" "feature"="avx512f" "nodecode" {
  %r = call <2 x double> @llvm.x86.avx512.mask.fixupimm.sd(<2 x double> %a0, <2 x double> %a1, <2 x i64> %a2, i32 %a3, i8 %a4, i32 4)
  ret <2 x double> %r
}

declare <2 x double> @llvm.x86.avx512.mask.fixupimm.sd(<2 x double>, <2 x double>, <2 x i64>, i32, i8, i32)

define <4 x float> @X86.avx512.mask.fixupimm.ss(<4 x float> "reg"="xv" "tied" %a0, <4 x float> "reg"="xv" %a1, <4 x i32> "reg"="xv" %a2, i32 "imm" %a3, i8 "reg"="k" %a4) "enc"="{ if(a4!=0) evexr(d, a2, 3, 0, a1, 0, 1, 0, a4, 0) | 01010101 | 11 reg:3=d&7 rm:3=a2&7 | { i1:8=a3 ; i1:s8=a3 } }" "asm"="vfixupimmss ${a3}, {a2}, {a1}, {d} {lb}{a4}{rb}" "dclass"="xv" "feature"="avx512f" "nodecode" {
  %r = call <4 x float> @llvm.x86.avx512.mask.fixupimm.ss(<4 x float> %a0, <4 x float> %a1, <4 x i32> %a2, i32 %a3, i8 %a4, i32 4)
  ret <4 x float> %r
}

declare <4 x float> @llvm.x86.avx512.mask.fixupimm.ss(<4 x float>, <4 x float>, <4 x i32>, i32, i8, i32)

define <2 x double> @X86.avx512.mask.getexp.pd.128(<2 x double> "reg"="xv" %a0, <2 x double> "reg"="xv" "tied" %a1, i8 "reg"="k" %a2) "enc"="{ if(a2!=0) evexr(d, a0, 2, 1, 0, 0, 1, 0, a2, 0) | 01000010 | 11 reg:3=d&7 rm:3=a0&7 }" "asm"="vgetexppd {a0}, {d} {lb}{a2}{rb}" "dclass"="xv" "feature"="avx512f,avx512vl" "nodecode" {
  %r = call <2 x double> @llvm.x86.avx512.mask.getexp.pd.128(<2 x double> %a0, <2 x double> %a1, i8 %a2)
  ret <2 x double> %r
}

declare <2 x double> @llvm.x86.avx512.mask.getexp.pd.128(<2 x double>, <2 x double>, i8)

define <4 x double> @X86.avx512.mask.getexp.pd.256(<4 x double> "reg"="yv" %a0, <4 x double> "reg"="yv" "tied" %a1, i8 "reg"="k" %a2) "enc"="{ if(a2!=0) evexr(d, a0, 2, 1, 0, 1, 1, 0, a2, 0) | 01000010 | 11 reg:3=d&7 rm:3=a0&7 }" "asm"="vgetexppd {a0}, {d} {lb}{a2}{rb}" "dclass"="yv" "feature"="avx512f,avx512vl" "nodecode" {
  %r = call <4 x double> @llvm.x86.avx512.mask.getexp.pd.256(<4 x double> %a0, <4 x double> %a1, i8 %a2)
  ret <4 x double> %r
}

declare <4 x double> @llvm.x86.avx512.mask.getexp.pd.256(<4 x double>, <4 x double>, i8)

define <8 x double> @X86.avx512.mask.getexp.pd.512(<8 x double> "reg"="zv" %a0, <8 x double> "reg"="zv" "tied" %a1, i8 "reg"="k" %a2) "enc"="{ if(a2!=0) evexr(d, a0, 2, 1, 0, 2, 1, 0, a2, 0) | 01000010 | 11 reg:3=d&7 rm:3=a0&7 }" "asm"="vgetexppd {a0}, {d} {lb}{a2}{rb}" "dclass"="zv" "feature"="avx512f" "nodecode" {
  %r = call <8 x double> @llvm.x86.avx512.mask.getexp.pd.512(<8 x double> %a0, <8 x double> %a1, i8 %a2, i32 4)
  ret <8 x double> %r
}

declare <8 x double> @llvm.x86.avx512.mask.getexp.pd.512(<8 x double>, <8 x double>, i8, i32)

define <4 x float> @X86.avx512.mask.getexp.ps.128(<4 x float> "reg"="xv" %a0, <4 x float> "reg"="xv" "tied" %a1, i8 "reg"="k" %a2) "enc"="{ if(a2!=0) evexr(d, a0, 2, 0, 0, 0, 1, 0, a2, 0) | 01000010 | 11 reg:3=d&7 rm:3=a0&7 }" "asm"="vgetexpps {a0}, {d} {lb}{a2}{rb}" "dclass"="xv" "feature"="avx512f,avx512vl" "nodecode" {
  %r = call <4 x float> @llvm.x86.avx512.mask.getexp.ps.128(<4 x float> %a0, <4 x float> %a1, i8 %a2)
  ret <4 x float> %r
}

declare <4 x float> @llvm.x86.avx512.mask.getexp.ps.128(<4 x float>, <4 x float>, i8)

define <8 x float> @X86.avx512.mask.getexp.ps.256(<8 x float> "reg"="yv" %a0, <8 x float> "reg"="yv" "tied" %a1, i8 "reg"="k" %a2) "enc"="{ if(a2!=0) evexr(d, a0, 2, 0, 0, 1, 1, 0, a2, 0) | 01000010 | 11 reg:3=d&7 rm:3=a0&7 }" "asm"="vgetexpps {a0}, {d} {lb}{a2}{rb}" "dclass"="yv" "feature"="avx512f,avx512vl" "nodecode" {
  %r = call <8 x float> @llvm.x86.avx512.mask.getexp.ps.256(<8 x float> %a0, <8 x float> %a1, i8 %a2)
  ret <8 x float> %r
}

declare <8 x float> @llvm.x86.avx512.mask.getexp.ps.256(<8 x float>, <8 x float>, i8)

define <16 x float> @X86.avx512.mask.getexp.ps.512(<16 x float> "reg"="zv" %a0, <16 x float> "reg"="zv" "tied" %a1, i16 "reg"="k" %a2) "enc"="{ if(a2!=0) evexr(d, a0, 2, 0, 0, 2, 1, 0, a2, 0) | 01000010 | 11 reg:3=d&7 rm:3=a0&7 }" "asm"="vgetexpps {a0}, {d} {lb}{a2}{rb}" "dclass"="zv" "feature"="avx512f" "nodecode" {
  %r = call <16 x float> @llvm.x86.avx512.mask.getexp.ps.512(<16 x float> %a0, <16 x float> %a1, i16 %a2, i32 4)
  ret <16 x float> %r
}

declare <16 x float> @llvm.x86.avx512.mask.getexp.ps.512(<16 x float>, <16 x float>, i16, i32)

define <2 x double> @X86.avx512.mask.getexp.sd(<2 x double> "reg"="xv" %a0, <2 x double> "reg"="xv" %a1, <2 x double> "reg"="xv" "tied" %a2, i8 "reg"="k" %a3) "enc"="{ if(a3!=0) evexr(d, a1, 2, 1, a0, 0, 1, 0, a3, 0) | 01000011 | 11 reg:3=d&7 rm:3=a1&7 }" "asm"="vgetexpsd {a1}, {a0}, {d} {lb}{a3}{rb}" "dclass"="xv" "feature"="avx512f" "nodecode" {
  %r = call <2 x double> @llvm.x86.avx512.mask.getexp.sd(<2 x double> %a0, <2 x double> %a1, <2 x double> %a2, i8 %a3, i32 4)
  ret <2 x double> %r
}

declare <2 x double> @llvm.x86.avx512.mask.getexp.sd(<2 x double>, <2 x double>, <2 x double>, i8, i32)

define <4 x float> @X86.avx512.mask.getexp.ss(<4 x float> "reg"="xv" %a0, <4 x float> "reg"="xv" %a1, <4 x float> "reg"="xv" "tied" %a2, i8 "reg"="k" %a3) "enc"="{ if(a3!=0) evexr(d, a1, 2, 0, a0, 0, 1, 0, a3, 0) | 01000011 | 11 reg:3=d&7 rm:3=a1&7 }" "asm"="vgetexpss {a1}, {a0}, {d} {lb}{a3}{rb}" "dclass"="xv" "feature"="avx512f" "nodecode" {
  %r = call <4 x float> @llvm.x86.avx512.mask.getexp.ss(<4 x float> %a0, <4 x float> %a1, <4 x float> %a2, i8 %a3, i32 4)
  ret <4 x float> %r
}

declare <4 x float> @llvm.x86.avx512.mask.getexp.ss(<4 x float>, <4 x float>, <4 x float>, i8, i32)

define <2 x double> @X86.avx512.mask.getmant.pd.128(<2 x double> "reg"="xv" %a0, i32 "imm" %a1, <2 x double> "reg"="xv" "tied" %a2, i8 "reg"="k" %a3) "enc"="{ if(a3!=0) evexr(d, a0, 3, 1, 0, 0, 1, 0, a3, 0) | 00100110 | 11 reg:3=d&7 rm:3=a0&7 | { i1:8=a1 ; i1:s8=a1 } }" "asm"="vgetmantpd ${a1}, {a0}, {d} {lb}{a3}{rb}" "dclass"="xv" "feature"="avx512f,avx512vl" "nodecode" {
  %r = call <2 x double> @llvm.x86.avx512.mask.getmant.pd.128(<2 x double> %a0, i32 %a1, <2 x double> %a2, i8 %a3)
  ret <2 x double> %r
}

declare <2 x double> @llvm.x86.avx512.mask.getmant.pd.128(<2 x double>, i32, <2 x double>, i8)

define <4 x double> @X86.avx512.mask.getmant.pd.256(<4 x double> "reg"="yv" %a0, i32 "imm" %a1, <4 x double> "reg"="yv" "tied" %a2, i8 "reg"="k" %a3) "enc"="{ if(a3!=0) evexr(d, a0, 3, 1, 0, 1, 1, 0, a3, 0) | 00100110 | 11 reg:3=d&7 rm:3=a0&7 | { i1:8=a1 ; i1:s8=a1 } }" "asm"="vgetmantpd ${a1}, {a0}, {d} {lb}{a3}{rb}" "dclass"="yv" "feature"="avx512f,avx512vl" "nodecode" {
  %r = call <4 x double> @llvm.x86.avx512.mask.getmant.pd.256(<4 x double> %a0, i32 %a1, <4 x double> %a2, i8 %a3)
  ret <4 x double> %r
}

declare <4 x double> @llvm.x86.avx512.mask.getmant.pd.256(<4 x double>, i32, <4 x double>, i8)

define <8 x double> @X86.avx512.mask.getmant.pd.512(<8 x double> "reg"="zv" %a0, i32 "imm" %a1, <8 x double> "reg"="zv" "tied" %a2, i8 "reg"="k" %a3) "enc"="{ if(a3!=0) evexr(d, a0, 3, 1, 0, 2, 1, 0, a3, 0) | 00100110 | 11 reg:3=d&7 rm:3=a0&7 | { i1:8=a1 ; i1:s8=a1 } }" "asm"="vgetmantpd ${a1}, {a0}, {d} {lb}{a3}{rb}" "dclass"="zv" "feature"="avx512f" "nodecode" {
  %r = call <8 x double> @llvm.x86.avx512.mask.getmant.pd.512(<8 x double> %a0, i32 %a1, <8 x double> %a2, i8 %a3, i32 4)
  ret <8 x double> %r
}

declare <8 x double> @llvm.x86.avx512.mask.getmant.pd.512(<8 x double>, i32, <8 x double>, i8, i32)

define <4 x float> @X86.avx512.mask.getmant.ps.128(<4 x float> "reg"="xv" %a0, i32 "imm" %a1, <4 x float> "reg"="xv" "tied" %a2, i8 "reg"="k" %a3) "enc"="{ if(a3!=0) evexr(d, a0, 3, 0, 0, 0, 1, 0, a3, 0) | 00100110 | 11 reg:3=d&7 rm:3=a0&7 | { i1:8=a1 ; i1:s8=a1 } }" "asm"="vgetmantps ${a1}, {a0}, {d} {lb}{a3}{rb}" "dclass"="xv" "feature"="avx512f,avx512vl" "nodecode" {
  %r = call <4 x float> @llvm.x86.avx512.mask.getmant.ps.128(<4 x float> %a0, i32 %a1, <4 x float> %a2, i8 %a3)
  ret <4 x float> %r
}

declare <4 x float> @llvm.x86.avx512.mask.getmant.ps.128(<4 x float>, i32, <4 x float>, i8)

define <8 x float> @X86.avx512.mask.getmant.ps.256(<8 x float> "reg"="yv" %a0, i32 "imm" %a1, <8 x float> "reg"="yv" "tied" %a2, i8 "reg"="k" %a3) "enc"="{ if(a3!=0) evexr(d, a0, 3, 0, 0, 1, 1, 0, a3, 0) | 00100110 | 11 reg:3=d&7 rm:3=a0&7 | { i1:8=a1 ; i1:s8=a1 } }" "asm"="vgetmantps ${a1}, {a0}, {d} {lb}{a3}{rb}" "dclass"="yv" "feature"="avx512f,avx512vl" "nodecode" {
  %r = call <8 x float> @llvm.x86.avx512.mask.getmant.ps.256(<8 x float> %a0, i32 %a1, <8 x float> %a2, i8 %a3)
  ret <8 x float> %r
}

declare <8 x float> @llvm.x86.avx512.mask.getmant.ps.256(<8 x float>, i32, <8 x float>, i8)

define <16 x float> @X86.avx512.mask.getmant.ps.512(<16 x float> "reg"="zv" %a0, i32 "imm" %a1, <16 x float> "reg"="zv" "tied" %a2, i16 "reg"="k" %a3) "enc"="{ if(a3!=0) evexr(d, a0, 3, 0, 0, 2, 1, 0, a3, 0) | 00100110 | 11 reg:3=d&7 rm:3=a0&7 | { i1:8=a1 ; i1:s8=a1 } }" "asm"="vgetmantps ${a1}, {a0}, {d} {lb}{a3}{rb}" "dclass"="zv" "feature"="avx512f" "nodecode" {
  %r = call <16 x float> @llvm.x86.avx512.mask.getmant.ps.512(<16 x float> %a0, i32 %a1, <16 x float> %a2, i16 %a3, i32 4)
  ret <16 x float> %r
}

declare <16 x float> @llvm.x86.avx512.mask.getmant.ps.512(<16 x float>, i32, <16 x float>, i16, i32)

define <2 x double> @X86.avx512.mask.getmant.sd(<2 x double> "reg"="xv" %a0, <2 x double> "reg"="xv" %a1, i32 "imm" %a2, <2 x double> "reg"="xv" "tied" %a3, i8 "reg"="k" %a4) "enc"="{ if(a4!=0) evexr(d, a1, 3, 1, a0, 0, 1, 0, a4, 0) | 00100111 | 11 reg:3=d&7 rm:3=a1&7 | { i1:8=a2 ; i1:s8=a2 } }" "asm"="vgetmantsd ${a2}, {a1}, {a0}, {d} {lb}{a4}{rb}" "dclass"="xv" "feature"="avx512f" "nodecode" {
  %r = call <2 x double> @llvm.x86.avx512.mask.getmant.sd(<2 x double> %a0, <2 x double> %a1, i32 %a2, <2 x double> %a3, i8 %a4, i32 4)
  ret <2 x double> %r
}

declare <2 x double> @llvm.x86.avx512.mask.getmant.sd(<2 x double>, <2 x double>, i32, <2 x double>, i8, i32)

define <4 x float> @X86.avx512.mask.getmant.ss(<4 x float> "reg"="xv" %a0, <4 x float> "reg"="xv" %a1, i32 "imm" %a2, <4 x float> "reg"="xv" "tied" %a3, i8 "reg"="k" %a4) "enc"="{ if(a4!=0) evexr(d, a1, 3, 0, a0, 0, 1, 0, a4, 0) | 00100111 | 11 reg:3=d&7 rm:3=a1&7 | { i1:8=a2 ; i1:s8=a2 } }" "asm"="vgetmantss ${a2}, {a1}, {a0}, {d} {lb}{a4}{rb}" "dclass"="xv" "feature"="avx512f" "nodecode" {
  %r = call <4 x float> @llvm.x86.avx512.mask.getmant.ss(<4 x float> %a0, <4 x float> %a1, i32 %a2, <4 x float> %a3, i8 %a4, i32 4)
  ret <4 x float> %r
}

declare <4 x float> @llvm.x86.avx512.mask.getmant.ss(<4 x float>, <4 x float>, i32, <4 x float>, i8, i32)

define <2 x double> @X86.avx512.mask.max.sd.round(<2 x double> "reg"="xv" %a0, <2 x double> "reg"="xv" %a1, <2 x double> "reg"="xv" "tied" %a2, i8 "reg"="k" %a3) "enc"="{ if(a3!=0) evexr(d, a1, 1, 1, a0, 0, 3, 0, a3, 0) | 01011111 | 11 reg:3=d&7 rm:3=a1&7 }" "asm"="vmaxsd {a1}, {a0}, {d} {lb}{a3}{rb}" "dclass"="xv" "feature"="avx512f" "nodecode" {
  %r = call <2 x double> @llvm.x86.avx512.mask.max.sd.round(<2 x double> %a0, <2 x double> %a1, <2 x double> %a2, i8 %a3, i32 4)
  ret <2 x double> %r
}

declare <2 x double> @llvm.x86.avx512.mask.max.sd.round(<2 x double>, <2 x double>, <2 x double>, i8, i32)

define <4 x float> @X86.avx512.mask.max.ss.round(<4 x float> "reg"="xv" %a0, <4 x float> "reg"="xv" %a1, <4 x float> "reg"="xv" "tied" %a2, i8 "reg"="k" %a3) "enc"="{ if(a3!=0) evexr(d, a1, 1, 0, a0, 0, 2, 0, a3, 0) | 01011111 | 11 reg:3=d&7 rm:3=a1&7 }" "asm"="vmaxss {a1}, {a0}, {d} {lb}{a3}{rb}" "dclass"="xv" "feature"="avx512f" "nodecode" {
  %r = call <4 x float> @llvm.x86.avx512.mask.max.ss.round(<4 x float> %a0, <4 x float> %a1, <4 x float> %a2, i8 %a3, i32 4)
  ret <4 x float> %r
}

declare <4 x float> @llvm.x86.avx512.mask.max.ss.round(<4 x float>, <4 x float>, <4 x float>, i8, i32)

define <2 x double> @X86.avx512.mask.min.sd.round(<2 x double> "reg"="xv" %a0, <2 x double> "reg"="xv" %a1, <2 x double> "reg"="xv" "tied" %a2, i8 "reg"="k" %a3) "enc"="{ if(a3!=0) evexr(d, a1, 1, 1, a0, 0, 3, 0, a3, 0) | 01011101 | 11 reg:3=d&7 rm:3=a1&7 }" "asm"="vminsd {a1}, {a0}, {d} {lb}{a3}{rb}" "dclass"="xv" "feature"="avx512f" "nodecode" {
  %r = call <2 x double> @llvm.x86.avx512.mask.min.sd.round(<2 x double> %a0, <2 x double> %a1, <2 x double> %a2, i8 %a3, i32 4)
  ret <2 x double> %r
}

declare <2 x double> @llvm.x86.avx512.mask.min.sd.round(<2 x double>, <2 x double>, <2 x double>, i8, i32)

define <4 x float> @X86.avx512.mask.min.ss.round(<4 x float> "reg"="xv" %a0, <4 x float> "reg"="xv" %a1, <4 x float> "reg"="xv" "tied" %a2, i8 "reg"="k" %a3) "enc"="{ if(a3!=0) evexr(d, a1, 1, 0, a0, 0, 2, 0, a3, 0) | 01011101 | 11 reg:3=d&7 rm:3=a1&7 }" "asm"="vminss {a1}, {a0}, {d} {lb}{a3}{rb}" "dclass"="xv" "feature"="avx512f" "nodecode" {
  %r = call <4 x float> @llvm.x86.avx512.mask.min.ss.round(<4 x float> %a0, <4 x float> %a1, <4 x float> %a2, i8 %a3, i32 4)
  ret <4 x float> %r
}

declare <4 x float> @llvm.x86.avx512.mask.min.ss.round(<4 x float>, <4 x float>, <4 x float>, i8, i32)

define <2 x double> @X86.avx512.mask.mul.sd.round(<2 x double> "reg"="xv" %a0, <2 x double> "reg"="xv" %a1, <2 x double> "reg"="xv" "tied" %a2, i8 "reg"="k" %a3) "enc"="{ if(a3!=0) evexr(d, a1, 1, 1, a0, 0, 3, 0, a3, 0) | 01011001 | 11 reg:3=d&7 rm:3=a1&7 }" "asm"="vmulsd {a1}, {a0}, {d} {lb}{a3}{rb}" "dclass"="xv" "feature"="avx512f" "nodecode" {
  %r = call <2 x double> @llvm.x86.avx512.mask.mul.sd.round(<2 x double> %a0, <2 x double> %a1, <2 x double> %a2, i8 %a3, i32 4)
  ret <2 x double> %r
}

declare <2 x double> @llvm.x86.avx512.mask.mul.sd.round(<2 x double>, <2 x double>, <2 x double>, i8, i32)

define <4 x float> @X86.avx512.mask.mul.ss.round(<4 x float> "reg"="xv" %a0, <4 x float> "reg"="xv" %a1, <4 x float> "reg"="xv" "tied" %a2, i8 "reg"="k" %a3) "enc"="{ if(a3!=0) evexr(d, a1, 1, 0, a0, 0, 2, 0, a3, 0) | 01011001 | 11 reg:3=d&7 rm:3=a1&7 }" "asm"="vmulss {a1}, {a0}, {d} {lb}{a3}{rb}" "dclass"="xv" "feature"="avx512f" "nodecode" {
  %r = call <4 x float> @llvm.x86.avx512.mask.mul.ss.round(<4 x float> %a0, <4 x float> %a1, <4 x float> %a2, i8 %a3, i32 4)
  ret <4 x float> %r
}

declare <4 x float> @llvm.x86.avx512.mask.mul.ss.round(<4 x float>, <4 x float>, <4 x float>, i8, i32)

define <16 x i8> @X86.avx512.mask.pmov.db.128(<4 x i32> "reg"="xv" %a0, <16 x i8> "reg"="xv" "tied" %a1, i8 "reg"="k" %a2) "enc"="{ if(a2!=0) evexr(a0, d, 2, 0, 0, 0, 2, 0, a2, 0) | 00110001 | 11 reg:3=a0&7 rm:3=d&7 }" "asm"="vpmovdb {a0}, {d} {lb}{a2}{rb}" "dclass"="xv" "feature"="avx512f,avx512vl" "nodecode" {
  %r = call <16 x i8> @llvm.x86.avx512.mask.pmov.db.128(<4 x i32> %a0, <16 x i8> %a1, i8 %a2)
  ret <16 x i8> %r
}

declare <16 x i8> @llvm.x86.avx512.mask.pmov.db.128(<4 x i32>, <16 x i8>, i8)

define <16 x i8> @X86.avx512.mask.pmov.db.256(<8 x i32> "reg"="yv" %a0, <16 x i8> "reg"="xv" "tied" %a1, i8 "reg"="k" %a2) "enc"="{ if(a2!=0) evexr(a0, d, 2, 0, 0, 1, 2, 0, a2, 0) | 00110001 | 11 reg:3=a0&7 rm:3=d&7 }" "asm"="vpmovdb {a0}, {d} {lb}{a2}{rb}" "dclass"="xv" "feature"="avx512f,avx512vl" "nodecode" {
  %r = call <16 x i8> @llvm.x86.avx512.mask.pmov.db.256(<8 x i32> %a0, <16 x i8> %a1, i8 %a2)
  ret <16 x i8> %r
}

declare <16 x i8> @llvm.x86.avx512.mask.pmov.db.256(<8 x i32>, <16 x i8>, i8)

define void @X86.avx512.mask.pmov.db.mem.128(ptr "reg"="r64" %a0, <4 x i32> "reg"="xv" %a1, i8 "reg"="k" %a2) "enc"="{ if(a2!=0) evexm(a1, a0, 4, 2, 0, 0, 0, 2, 0, a2, 0) | 00110001 | rm(a1, a0, 4, 1, 0, 0, 4) }" "asm"="vpmovdb {a1}, 0({a0}) {lb}{a2}{rb}" "feature"="avx512f,avx512vl" "nodecode" {
  call void @llvm.x86.avx512.mask.pmov.db.mem.128(ptr %a0, <4 x i32> %a1, i8 %a2)
  ret void
}

declare void @llvm.x86.avx512.mask.pmov.db.mem.128(ptr, <4 x i32>, i8)

define void @X86.avx512.mask.pmov.db.mem.256(ptr "reg"="r64" %a0, <8 x i32> "reg"="yv" %a1, i8 "reg"="k" %a2) "enc"="{ if(a2!=0) evexm(a1, a0, 4, 2, 0, 0, 1, 2, 0, a2, 0) | 00110001 | rm(a1, a0, 4, 1, 0, 0, 8) }" "asm"="vpmovdb {a1}, 0({a0}) {lb}{a2}{rb}" "feature"="avx512f,avx512vl" "nodecode" {
  call void @llvm.x86.avx512.mask.pmov.db.mem.256(ptr %a0, <8 x i32> %a1, i8 %a2)
  ret void
}

declare void @llvm.x86.avx512.mask.pmov.db.mem.256(ptr, <8 x i32>, i8)

define void @X86.avx512.mask.pmov.db.mem.512(ptr "reg"="r64" %a0, <16 x i32> "reg"="zv" %a1, i16 "reg"="k" %a2) "enc"="{ if(a2!=0) evexm(a1, a0, 4, 2, 0, 0, 2, 2, 0, a2, 0) | 00110001 | rm(a1, a0, 4, 1, 0, 0, 16) }" "asm"="vpmovdb {a1}, 0({a0}) {lb}{a2}{rb}" "feature"="avx512f" "nodecode" {
  call void @llvm.x86.avx512.mask.pmov.db.mem.512(ptr %a0, <16 x i32> %a1, i16 %a2)
  ret void
}

declare void @llvm.x86.avx512.mask.pmov.db.mem.512(ptr, <16 x i32>, i16)

define <8 x i16> @X86.avx512.mask.pmov.dw.128(<4 x i32> "reg"="xv" %a0, <8 x i16> "reg"="xv" "tied" %a1, i8 "reg"="k" %a2) "enc"="{ if(a2!=0) evexr(a0, d, 2, 0, 0, 0, 2, 0, a2, 0) | 00110011 | 11 reg:3=a0&7 rm:3=d&7 }" "asm"="vpmovdw {a0}, {d} {lb}{a2}{rb}" "dclass"="xv" "feature"="avx512f,avx512vl" "nodecode" {
  %r = call <8 x i16> @llvm.x86.avx512.mask.pmov.dw.128(<4 x i32> %a0, <8 x i16> %a1, i8 %a2)
  ret <8 x i16> %r
}

declare <8 x i16> @llvm.x86.avx512.mask.pmov.dw.128(<4 x i32>, <8 x i16>, i8)

define void @X86.avx512.mask.pmov.dw.mem.128(ptr "reg"="r64" %a0, <4 x i32> "reg"="xv" %a1, i8 "reg"="k" %a2) "enc"="{ if(a2!=0) evexm(a1, a0, 4, 2, 0, 0, 0, 2, 0, a2, 0) | 00110011 | rm(a1, a0, 4, 1, 0, 0, 8) }" "asm"="vpmovdw {a1}, 0({a0}) {lb}{a2}{rb}" "feature"="avx512f,avx512vl" "nodecode" {
  call void @llvm.x86.avx512.mask.pmov.dw.mem.128(ptr %a0, <4 x i32> %a1, i8 %a2)
  ret void
}

declare void @llvm.x86.avx512.mask.pmov.dw.mem.128(ptr, <4 x i32>, i8)

define void @X86.avx512.mask.pmov.dw.mem.256(ptr "reg"="r64" %a0, <8 x i32> "reg"="yv" %a1, i8 "reg"="k" %a2) "enc"="{ if(a2!=0) evexm(a1, a0, 4, 2, 0, 0, 1, 2, 0, a2, 0) | 00110011 | rm(a1, a0, 4, 1, 0, 0, 16) }" "asm"="vpmovdw {a1}, 0({a0}) {lb}{a2}{rb}" "feature"="avx512f,avx512vl" "nodecode" {
  call void @llvm.x86.avx512.mask.pmov.dw.mem.256(ptr %a0, <8 x i32> %a1, i8 %a2)
  ret void
}

declare void @llvm.x86.avx512.mask.pmov.dw.mem.256(ptr, <8 x i32>, i8)

define void @X86.avx512.mask.pmov.dw.mem.512(ptr "reg"="r64" %a0, <16 x i32> "reg"="zv" %a1, i16 "reg"="k" %a2) "enc"="{ if(a2!=0) evexm(a1, a0, 4, 2, 0, 0, 2, 2, 0, a2, 0) | 00110011 | rm(a1, a0, 4, 1, 0, 0, 32) }" "asm"="vpmovdw {a1}, 0({a0}) {lb}{a2}{rb}" "feature"="avx512f" "nodecode" {
  call void @llvm.x86.avx512.mask.pmov.dw.mem.512(ptr %a0, <16 x i32> %a1, i16 %a2)
  ret void
}

declare void @llvm.x86.avx512.mask.pmov.dw.mem.512(ptr, <16 x i32>, i16)

define <16 x i8> @X86.avx512.mask.pmov.qb.128(<2 x i64> "reg"="xv" %a0, <16 x i8> "reg"="xv" "tied" %a1, i8 "reg"="k" %a2) "enc"="{ if(a2!=0) evexr(a0, d, 2, 0, 0, 0, 2, 0, a2, 0) | 00110010 | 11 reg:3=a0&7 rm:3=d&7 }" "asm"="vpmovqb {a0}, {d} {lb}{a2}{rb}" "dclass"="xv" "feature"="avx512f,avx512vl" "nodecode" {
  %r = call <16 x i8> @llvm.x86.avx512.mask.pmov.qb.128(<2 x i64> %a0, <16 x i8> %a1, i8 %a2)
  ret <16 x i8> %r
}

declare <16 x i8> @llvm.x86.avx512.mask.pmov.qb.128(<2 x i64>, <16 x i8>, i8)

define <16 x i8> @X86.avx512.mask.pmov.qb.256(<4 x i64> "reg"="yv" %a0, <16 x i8> "reg"="xv" "tied" %a1, i8 "reg"="k" %a2) "enc"="{ if(a2!=0) evexr(a0, d, 2, 0, 0, 1, 2, 0, a2, 0) | 00110010 | 11 reg:3=a0&7 rm:3=d&7 }" "asm"="vpmovqb {a0}, {d} {lb}{a2}{rb}" "dclass"="xv" "feature"="avx512f,avx512vl" "nodecode" {
  %r = call <16 x i8> @llvm.x86.avx512.mask.pmov.qb.256(<4 x i64> %a0, <16 x i8> %a1, i8 %a2)
  ret <16 x i8> %r
}

declare <16 x i8> @llvm.x86.avx512.mask.pmov.qb.256(<4 x i64>, <16 x i8>, i8)

define <16 x i8> @X86.avx512.mask.pmov.qb.512(<8 x i64> "reg"="zv" %a0, <16 x i8> "reg"="xv" "tied" %a1, i8 "reg"="k" %a2) "enc"="{ if(a2!=0) evexr(a0, d, 2, 0, 0, 2, 2, 0, a2, 0) | 00110010 | 11 reg:3=a0&7 rm:3=d&7 }" "asm"="vpmovqb {a0}, {d} {lb}{a2}{rb}" "dclass"="xv" "feature"="avx512f" "nodecode" {
  %r = call <16 x i8> @llvm.x86.avx512.mask.pmov.qb.512(<8 x i64> %a0, <16 x i8> %a1, i8 %a2)
  ret <16 x i8> %r
}

declare <16 x i8> @llvm.x86.avx512.mask.pmov.qb.512(<8 x i64>, <16 x i8>, i8)

define void @X86.avx512.mask.pmov.qb.mem.128(ptr "reg"="r64" %a0, <2 x i64> "reg"="xv" %a1, i8 "reg"="k" %a2) "enc"="{ if(a2!=0) evexm(a1, a0, 4, 2, 0, 0, 0, 2, 0, a2, 0) | 00110010 | rm(a1, a0, 4, 1, 0, 0, 2) }" "asm"="vpmovqb {a1}, 0({a0}) {lb}{a2}{rb}" "feature"="avx512f,avx512vl" "nodecode" {
  call void @llvm.x86.avx512.mask.pmov.qb.mem.128(ptr %a0, <2 x i64> %a1, i8 %a2)
  ret void
}

declare void @llvm.x86.avx512.mask.pmov.qb.mem.128(ptr, <2 x i64>, i8)

define void @X86.avx512.mask.pmov.qb.mem.256(ptr "reg"="r64" %a0, <4 x i64> "reg"="yv" %a1, i8 "reg"="k" %a2) "enc"="{ if(a2!=0) evexm(a1, a0, 4, 2, 0, 0, 1, 2, 0, a2, 0) | 00110010 | rm(a1, a0, 4, 1, 0, 0, 4) }" "asm"="vpmovqb {a1}, 0({a0}) {lb}{a2}{rb}" "feature"="avx512f,avx512vl" "nodecode" {
  call void @llvm.x86.avx512.mask.pmov.qb.mem.256(ptr %a0, <4 x i64> %a1, i8 %a2)
  ret void
}

declare void @llvm.x86.avx512.mask.pmov.qb.mem.256(ptr, <4 x i64>, i8)

define void @X86.avx512.mask.pmov.qb.mem.512(ptr "reg"="r64" %a0, <8 x i64> "reg"="zv" %a1, i8 "reg"="k" %a2) "enc"="{ if(a2!=0) evexm(a1, a0, 4, 2, 0, 0, 2, 2, 0, a2, 0) | 00110010 | rm(a1, a0, 4, 1, 0, 0, 8) }" "asm"="vpmovqb {a1}, 0({a0}) {lb}{a2}{rb}" "feature"="avx512f" "nodecode" {
  call void @llvm.x86.avx512.mask.pmov.qb.mem.512(ptr %a0, <8 x i64> %a1, i8 %a2)
  ret void
}

declare void @llvm.x86.avx512.mask.pmov.qb.mem.512(ptr, <8 x i64>, i8)

define <4 x i32> @X86.avx512.mask.pmov.qd.128(<2 x i64> "reg"="xv" %a0, <4 x i32> "reg"="xv" "tied" %a1, i8 "reg"="k" %a2) "enc"="{ if(a2!=0) evexr(a0, d, 2, 0, 0, 0, 2, 0, a2, 0) | 00110101 | 11 reg:3=a0&7 rm:3=d&7 }" "asm"="vpmovqd {a0}, {d} {lb}{a2}{rb}" "dclass"="xv" "feature"="avx512f,avx512vl" "nodecode" {
  %r = call <4 x i32> @llvm.x86.avx512.mask.pmov.qd.128(<2 x i64> %a0, <4 x i32> %a1, i8 %a2)
  ret <4 x i32> %r
}

declare <4 x i32> @llvm.x86.avx512.mask.pmov.qd.128(<2 x i64>, <4 x i32>, i8)

define void @X86.avx512.mask.pmov.qd.mem.128(ptr "reg"="r64" %a0, <2 x i64> "reg"="xv" %a1, i8 "reg"="k" %a2) "enc"="{ if(a2!=0) evexm(a1, a0, 4, 2, 0, 0, 0, 2, 0, a2, 0) | 00110101 | rm(a1, a0, 4, 1, 0, 0, 8) }" "asm"="vpmovqd {a1}, 0({a0}) {lb}{a2}{rb}" "feature"="avx512f,avx512vl" "nodecode" {
  call void @llvm.x86.avx512.mask.pmov.qd.mem.128(ptr %a0, <2 x i64> %a1, i8 %a2)
  ret void
}

declare void @llvm.x86.avx512.mask.pmov.qd.mem.128(ptr, <2 x i64>, i8)

define void @X86.avx512.mask.pmov.qd.mem.256(ptr "reg"="r64" %a0, <4 x i64> "reg"="yv" %a1, i8 "reg"="k" %a2) "enc"="{ if(a2!=0) evexm(a1, a0, 4, 2, 0, 0, 1, 2, 0, a2, 0) | 00110101 | rm(a1, a0, 4, 1, 0, 0, 16) }" "asm"="vpmovqd {a1}, 0({a0}) {lb}{a2}{rb}" "feature"="avx512f,avx512vl" "nodecode" {
  call void @llvm.x86.avx512.mask.pmov.qd.mem.256(ptr %a0, <4 x i64> %a1, i8 %a2)
  ret void
}

declare void @llvm.x86.avx512.mask.pmov.qd.mem.256(ptr, <4 x i64>, i8)

define void @X86.avx512.mask.pmov.qd.mem.512(ptr "reg"="r64" %a0, <8 x i64> "reg"="zv" %a1, i8 "reg"="k" %a2) "enc"="{ if(a2!=0) evexm(a1, a0, 4, 2, 0, 0, 2, 2, 0, a2, 0) | 00110101 | rm(a1, a0, 4, 1, 0, 0, 32) }" "asm"="vpmovqd {a1}, 0({a0}) {lb}{a2}{rb}" "feature"="avx512f" "nodecode" {
  call void @llvm.x86.avx512.mask.pmov.qd.mem.512(ptr %a0, <8 x i64> %a1, i8 %a2)
  ret void
}

declare void @llvm.x86.avx512.mask.pmov.qd.mem.512(ptr, <8 x i64>, i8)

define <8 x i16> @X86.avx512.mask.pmov.qw.128(<2 x i64> "reg"="xv" %a0, <8 x i16> "reg"="xv" "tied" %a1, i8 "reg"="k" %a2) "enc"="{ if(a2!=0) evexr(a0, d, 2, 0, 0, 0, 2, 0, a2, 0) | 00110100 | 11 reg:3=a0&7 rm:3=d&7 }" "asm"="vpmovqw {a0}, {d} {lb}{a2}{rb}" "dclass"="xv" "feature"="avx512f,avx512vl" "nodecode" {
  %r = call <8 x i16> @llvm.x86.avx512.mask.pmov.qw.128(<2 x i64> %a0, <8 x i16> %a1, i8 %a2)
  ret <8 x i16> %r
}

declare <8 x i16> @llvm.x86.avx512.mask.pmov.qw.128(<2 x i64>, <8 x i16>, i8)

define <8 x i16> @X86.avx512.mask.pmov.qw.256(<4 x i64> "reg"="yv" %a0, <8 x i16> "reg"="xv" "tied" %a1, i8 "reg"="k" %a2) "enc"="{ if(a2!=0) evexr(a0, d, 2, 0, 0, 1, 2, 0, a2, 0) | 00110100 | 11 reg:3=a0&7 rm:3=d&7 }" "asm"="vpmovqw {a0}, {d} {lb}{a2}{rb}" "dclass"="xv" "feature"="avx512f,avx512vl" "nodecode" {
  %r = call <8 x i16> @llvm.x86.avx512.mask.pmov.qw.256(<4 x i64> %a0, <8 x i16> %a1, i8 %a2)
  ret <8 x i16> %r
}

declare <8 x i16> @llvm.x86.avx512.mask.pmov.qw.256(<4 x i64>, <8 x i16>, i8)

define void @X86.avx512.mask.pmov.qw.mem.128(ptr "reg"="r64" %a0, <2 x i64> "reg"="xv" %a1, i8 "reg"="k" %a2) "enc"="{ if(a2!=0) evexm(a1, a0, 4, 2, 0, 0, 0, 2, 0, a2, 0) | 00110100 | rm(a1, a0, 4, 1, 0, 0, 4) }" "asm"="vpmovqw {a1}, 0({a0}) {lb}{a2}{rb}" "feature"="avx512f,avx512vl" "nodecode" {
  call void @llvm.x86.avx512.mask.pmov.qw.mem.128(ptr %a0, <2 x i64> %a1, i8 %a2)
  ret void
}

declare void @llvm.x86.avx512.mask.pmov.qw.mem.128(ptr, <2 x i64>, i8)

define void @X86.avx512.mask.pmov.qw.mem.256(ptr "reg"="r64" %a0, <4 x i64> "reg"="yv" %a1, i8 "reg"="k" %a2) "enc"="{ if(a2!=0) evexm(a1, a0, 4, 2, 0, 0, 1, 2, 0, a2, 0) | 00110100 | rm(a1, a0, 4, 1, 0, 0, 8) }" "asm"="vpmovqw {a1}, 0({a0}) {lb}{a2}{rb}" "feature"="avx512f,avx512vl" "nodecode" {
  call void @llvm.x86.avx512.mask.pmov.qw.mem.256(ptr %a0, <4 x i64> %a1, i8 %a2)
  ret void
}

declare void @llvm.x86.avx512.mask.pmov.qw.mem.256(ptr, <4 x i64>, i8)

define void @X86.avx512.mask.pmov.qw.mem.512(ptr "reg"="r64" %a0, <8 x i64> "reg"="zv" %a1, i8 "reg"="k" %a2) "enc"="{ if(a2!=0) evexm(a1, a0, 4, 2, 0, 0, 2, 2, 0, a2, 0) | 00110100 | rm(a1, a0, 4, 1, 0, 0, 16) }" "asm"="vpmovqw {a1}, 0({a0}) {lb}{a2}{rb}" "feature"="avx512f" "nodecode" {
  call void @llvm.x86.avx512.mask.pmov.qw.mem.512(ptr %a0, <8 x i64> %a1, i8 %a2)
  ret void
}

declare void @llvm.x86.avx512.mask.pmov.qw.mem.512(ptr, <8 x i64>, i8)

define <16 x i8> @X86.avx512.mask.pmovs.db.128(<4 x i32> "reg"="xv" %a0, <16 x i8> "reg"="xv" "tied" %a1, i8 "reg"="k" %a2) "enc"="{ if(a2!=0) evexr(a0, d, 2, 0, 0, 0, 2, 0, a2, 0) | 00100001 | 11 reg:3=a0&7 rm:3=d&7 }" "asm"="vpmovsdb {a0}, {d} {lb}{a2}{rb}" "dclass"="xv" "feature"="avx512f,avx512vl" "nodecode" {
  %r = call <16 x i8> @llvm.x86.avx512.mask.pmovs.db.128(<4 x i32> %a0, <16 x i8> %a1, i8 %a2)
  ret <16 x i8> %r
}

declare <16 x i8> @llvm.x86.avx512.mask.pmovs.db.128(<4 x i32>, <16 x i8>, i8)

define <16 x i8> @X86.avx512.mask.pmovs.db.256(<8 x i32> "reg"="yv" %a0, <16 x i8> "reg"="xv" "tied" %a1, i8 "reg"="k" %a2) "enc"="{ if(a2!=0) evexr(a0, d, 2, 0, 0, 1, 2, 0, a2, 0) | 00100001 | 11 reg:3=a0&7 rm:3=d&7 }" "asm"="vpmovsdb {a0}, {d} {lb}{a2}{rb}" "dclass"="xv" "feature"="avx512f,avx512vl" "nodecode" {
  %r = call <16 x i8> @llvm.x86.avx512.mask.pmovs.db.256(<8 x i32> %a0, <16 x i8> %a1, i8 %a2)
  ret <16 x i8> %r
}

declare <16 x i8> @llvm.x86.avx512.mask.pmovs.db.256(<8 x i32>, <16 x i8>, i8)

define <16 x i8> @X86.avx512.mask.pmovs.db.512(<16 x i32> "reg"="zv" %a0, <16 x i8> "reg"="xv" "tied" %a1, i16 "reg"="k" %a2) "enc"="{ if(a2!=0) evexr(a0, d, 2, 0, 0, 2, 2, 0, a2, 0) | 00100001 | 11 reg:3=a0&7 rm:3=d&7 }" "asm"="vpmovsdb {a0}, {d} {lb}{a2}{rb}" "dclass"="xv" "feature"="avx512f" "nodecode" {
  %r = call <16 x i8> @llvm.x86.avx512.mask.pmovs.db.512(<16 x i32> %a0, <16 x i8> %a1, i16 %a2)
  ret <16 x i8> %r
}

declare <16 x i8> @llvm.x86.avx512.mask.pmovs.db.512(<16 x i32>, <16 x i8>, i16)

define void @X86.avx512.mask.pmovs.db.mem.128(ptr "reg"="r64" %a0, <4 x i32> "reg"="xv" %a1, i8 "reg"="k" %a2) "enc"="{ if(a2!=0) evexm(a1, a0, 4, 2, 0, 0, 0, 2, 0, a2, 0) | 00100001 | rm(a1, a0, 4, 1, 0, 0, 4) }" "asm"="vpmovsdb {a1}, 0({a0}) {lb}{a2}{rb}" "feature"="avx512f,avx512vl" "nodecode" {
  call void @llvm.x86.avx512.mask.pmovs.db.mem.128(ptr %a0, <4 x i32> %a1, i8 %a2)
  ret void
}

declare void @llvm.x86.avx512.mask.pmovs.db.mem.128(ptr, <4 x i32>, i8)

define void @X86.avx512.mask.pmovs.db.mem.256(ptr "reg"="r64" %a0, <8 x i32> "reg"="yv" %a1, i8 "reg"="k" %a2) "enc"="{ if(a2!=0) evexm(a1, a0, 4, 2, 0, 0, 1, 2, 0, a2, 0) | 00100001 | rm(a1, a0, 4, 1, 0, 0, 8) }" "asm"="vpmovsdb {a1}, 0({a0}) {lb}{a2}{rb}" "feature"="avx512f,avx512vl" "nodecode" {
  call void @llvm.x86.avx512.mask.pmovs.db.mem.256(ptr %a0, <8 x i32> %a1, i8 %a2)
  ret void
}

declare void @llvm.x86.avx512.mask.pmovs.db.mem.256(ptr, <8 x i32>, i8)

define void @X86.avx512.mask.pmovs.db.mem.512(ptr "reg"="r64" %a0, <16 x i32> "reg"="zv" %a1, i16 "reg"="k" %a2) "enc"="{ if(a2!=0) evexm(a1, a0, 4, 2, 0, 0, 2, 2, 0, a2, 0) | 00100001 | rm(a1, a0, 4, 1, 0, 0, 16) }" "asm"="vpmovsdb {a1}, 0({a0}) {lb}{a2}{rb}" "feature"="avx512f" "nodecode" {
  call void @llvm.x86.avx512.mask.pmovs.db.mem.512(ptr %a0, <16 x i32> %a1, i16 %a2)
  ret void
}

declare void @llvm.x86.avx512.mask.pmovs.db.mem.512(ptr, <16 x i32>, i16)

define <8 x i16> @X86.avx512.mask.pmovs.dw.128(<4 x i32> "reg"="xv" %a0, <8 x i16> "reg"="xv" "tied" %a1, i8 "reg"="k" %a2) "enc"="{ if(a2!=0) evexr(a0, d, 2, 0, 0, 0, 2, 0, a2, 0) | 00100011 | 11 reg:3=a0&7 rm:3=d&7 }" "asm"="vpmovsdw {a0}, {d} {lb}{a2}{rb}" "dclass"="xv" "feature"="avx512f,avx512vl" "nodecode" {
  %r = call <8 x i16> @llvm.x86.avx512.mask.pmovs.dw.128(<4 x i32> %a0, <8 x i16> %a1, i8 %a2)
  ret <8 x i16> %r
}

declare <8 x i16> @llvm.x86.avx512.mask.pmovs.dw.128(<4 x i32>, <8 x i16>, i8)

define <8 x i16> @X86.avx512.mask.pmovs.dw.256(<8 x i32> "reg"="yv" %a0, <8 x i16> "reg"="xv" "tied" %a1, i8 "reg"="k" %a2) "enc"="{ if(a2!=0) evexr(a0, d, 2, 0, 0, 1, 2, 0, a2, 0) | 00100011 | 11 reg:3=a0&7 rm:3=d&7 }" "asm"="vpmovsdw {a0}, {d} {lb}{a2}{rb}" "dclass"="xv" "feature"="avx512f,avx512vl" "nodecode" {
  %r = call <8 x i16> @llvm.x86.avx512.mask.pmovs.dw.256(<8 x i32> %a0, <8 x i16> %a1, i8 %a2)
  ret <8 x i16> %r
}

declare <8 x i16> @llvm.x86.avx512.mask.pmovs.dw.256(<8 x i32>, <8 x i16>, i8)

define <16 x i16> @X86.avx512.mask.pmovs.dw.512(<16 x i32> "reg"="zv" %a0, <16 x i16> "reg"="yv" "tied" %a1, i16 "reg"="k" %a2) "enc"="{ if(a2!=0) evexr(a0, d, 2, 0, 0, 2, 2, 0, a2, 0) | 00100011 | 11 reg:3=a0&7 rm:3=d&7 }" "asm"="vpmovsdw {a0}, {d} {lb}{a2}{rb}" "dclass"="yv" "feature"="avx512f" "nodecode" {
  %r = call <16 x i16> @llvm.x86.avx512.mask.pmovs.dw.512(<16 x i32> %a0, <16 x i16> %a1, i16 %a2)
  ret <16 x i16> %r
}

declare <16 x i16> @llvm.x86.avx512.mask.pmovs.dw.512(<16 x i32>, <16 x i16>, i16)

define void @X86.avx512.mask.pmovs.dw.mem.128(ptr "reg"="r64" %a0, <4 x i32> "reg"="xv" %a1, i8 "reg"="k" %a2) "enc"="{ if(a2!=0) evexm(a1, a0, 4, 2, 0, 0, 0, 2, 0, a2, 0) | 00100011 | rm(a1, a0, 4, 1, 0, 0, 8) }" "asm"="vpmovsdw {a1}, 0({a0}) {lb}{a2}{rb}" "feature"="avx512f,avx512vl" "nodecode" {
  call void @llvm.x86.avx512.mask.pmovs.dw.mem.128(ptr %a0, <4 x i32> %a1, i8 %a2)
  ret void
}

declare void @llvm.x86.avx512.mask.pmovs.dw.mem.128(ptr, <4 x i32>, i8)

define void @X86.avx512.mask.pmovs.dw.mem.256(ptr "reg"="r64" %a0, <8 x i32> "reg"="yv" %a1, i8 "reg"="k" %a2) "enc"="{ if(a2!=0) evexm(a1, a0, 4, 2, 0, 0, 1, 2, 0, a2, 0) | 00100011 | rm(a1, a0, 4, 1, 0, 0, 16) }" "asm"="vpmovsdw {a1}, 0({a0}) {lb}{a2}{rb}" "feature"="avx512f,avx512vl" "nodecode" {
  call void @llvm.x86.avx512.mask.pmovs.dw.mem.256(ptr %a0, <8 x i32> %a1, i8 %a2)
  ret void
}

declare void @llvm.x86.avx512.mask.pmovs.dw.mem.256(ptr, <8 x i32>, i8)

define void @X86.avx512.mask.pmovs.dw.mem.512(ptr "reg"="r64" %a0, <16 x i32> "reg"="zv" %a1, i16 "reg"="k" %a2) "enc"="{ if(a2!=0) evexm(a1, a0, 4, 2, 0, 0, 2, 2, 0, a2, 0) | 00100011 | rm(a1, a0, 4, 1, 0, 0, 32) }" "asm"="vpmovsdw {a1}, 0({a0}) {lb}{a2}{rb}" "feature"="avx512f" "nodecode" {
  call void @llvm.x86.avx512.mask.pmovs.dw.mem.512(ptr %a0, <16 x i32> %a1, i16 %a2)
  ret void
}

declare void @llvm.x86.avx512.mask.pmovs.dw.mem.512(ptr, <16 x i32>, i16)

define <16 x i8> @X86.avx512.mask.pmovs.qb.128(<2 x i64> "reg"="xv" %a0, <16 x i8> "reg"="xv" "tied" %a1, i8 "reg"="k" %a2) "enc"="{ if(a2!=0) evexr(a0, d, 2, 0, 0, 0, 2, 0, a2, 0) | 00100010 | 11 reg:3=a0&7 rm:3=d&7 }" "asm"="vpmovsqb {a0}, {d} {lb}{a2}{rb}" "dclass"="xv" "feature"="avx512f,avx512vl" "nodecode" {
  %r = call <16 x i8> @llvm.x86.avx512.mask.pmovs.qb.128(<2 x i64> %a0, <16 x i8> %a1, i8 %a2)
  ret <16 x i8> %r
}

declare <16 x i8> @llvm.x86.avx512.mask.pmovs.qb.128(<2 x i64>, <16 x i8>, i8)

define <16 x i8> @X86.avx512.mask.pmovs.qb.256(<4 x i64> "reg"="yv" %a0, <16 x i8> "reg"="xv" "tied" %a1, i8 "reg"="k" %a2) "enc"="{ if(a2!=0) evexr(a0, d, 2, 0, 0, 1, 2, 0, a2, 0) | 00100010 | 11 reg:3=a0&7 rm:3=d&7 }" "asm"="vpmovsqb {a0}, {d} {lb}{a2}{rb}" "dclass"="xv" "feature"="avx512f,avx512vl" "nodecode" {
  %r = call <16 x i8> @llvm.x86.avx512.mask.pmovs.qb.256(<4 x i64> %a0, <16 x i8> %a1, i8 %a2)
  ret <16 x i8> %r
}

declare <16 x i8> @llvm.x86.avx512.mask.pmovs.qb.256(<4 x i64>, <16 x i8>, i8)

define <16 x i8> @X86.avx512.mask.pmovs.qb.512(<8 x i64> "reg"="zv" %a0, <16 x i8> "reg"="xv" "tied" %a1, i8 "reg"="k" %a2) "enc"="{ if(a2!=0) evexr(a0, d, 2, 0, 0, 2, 2, 0, a2, 0) | 00100010 | 11 reg:3=a0&7 rm:3=d&7 }" "asm"="vpmovsqb {a0}, {d} {lb}{a2}{rb}" "dclass"="xv" "feature"="avx512f" "nodecode" {
  %r = call <16 x i8> @llvm.x86.avx512.mask.pmovs.qb.512(<8 x i64> %a0, <16 x i8> %a1, i8 %a2)
  ret <16 x i8> %r
}

declare <16 x i8> @llvm.x86.avx512.mask.pmovs.qb.512(<8 x i64>, <16 x i8>, i8)

define void @X86.avx512.mask.pmovs.qb.mem.128(ptr "reg"="r64" %a0, <2 x i64> "reg"="xv" %a1, i8 "reg"="k" %a2) "enc"="{ if(a2!=0) evexm(a1, a0, 4, 2, 0, 0, 0, 2, 0, a2, 0) | 00100010 | rm(a1, a0, 4, 1, 0, 0, 2) }" "asm"="vpmovsqb {a1}, 0({a0}) {lb}{a2}{rb}" "feature"="avx512f,avx512vl" "nodecode" {
  call void @llvm.x86.avx512.mask.pmovs.qb.mem.128(ptr %a0, <2 x i64> %a1, i8 %a2)
  ret void
}

declare void @llvm.x86.avx512.mask.pmovs.qb.mem.128(ptr, <2 x i64>, i8)

define void @X86.avx512.mask.pmovs.qb.mem.256(ptr "reg"="r64" %a0, <4 x i64> "reg"="yv" %a1, i8 "reg"="k" %a2) "enc"="{ if(a2!=0) evexm(a1, a0, 4, 2, 0, 0, 1, 2, 0, a2, 0) | 00100010 | rm(a1, a0, 4, 1, 0, 0, 4) }" "asm"="vpmovsqb {a1}, 0({a0}) {lb}{a2}{rb}" "feature"="avx512f,avx512vl" "nodecode" {
  call void @llvm.x86.avx512.mask.pmovs.qb.mem.256(ptr %a0, <4 x i64> %a1, i8 %a2)
  ret void
}

declare void @llvm.x86.avx512.mask.pmovs.qb.mem.256(ptr, <4 x i64>, i8)

define void @X86.avx512.mask.pmovs.qb.mem.512(ptr "reg"="r64" %a0, <8 x i64> "reg"="zv" %a1, i8 "reg"="k" %a2) "enc"="{ if(a2!=0) evexm(a1, a0, 4, 2, 0, 0, 2, 2, 0, a2, 0) | 00100010 | rm(a1, a0, 4, 1, 0, 0, 8) }" "asm"="vpmovsqb {a1}, 0({a0}) {lb}{a2}{rb}" "feature"="avx512f" "nodecode" {
  call void @llvm.x86.avx512.mask.pmovs.qb.mem.512(ptr %a0, <8 x i64> %a1, i8 %a2)
  ret void
}

declare void @llvm.x86.avx512.mask.pmovs.qb.mem.512(ptr, <8 x i64>, i8)

define <4 x i32> @X86.avx512.mask.pmovs.qd.128(<2 x i64> "reg"="xv" %a0, <4 x i32> "reg"="xv" "tied" %a1, i8 "reg"="k" %a2) "enc"="{ if(a2!=0) evexr(a0, d, 2, 0, 0, 0, 2, 0, a2, 0) | 00100101 | 11 reg:3=a0&7 rm:3=d&7 }" "asm"="vpmovsqd {a0}, {d} {lb}{a2}{rb}" "dclass"="xv" "feature"="avx512f,avx512vl" "nodecode" {
  %r = call <4 x i32> @llvm.x86.avx512.mask.pmovs.qd.128(<2 x i64> %a0, <4 x i32> %a1, i8 %a2)
  ret <4 x i32> %r
}

declare <4 x i32> @llvm.x86.avx512.mask.pmovs.qd.128(<2 x i64>, <4 x i32>, i8)

define <4 x i32> @X86.avx512.mask.pmovs.qd.256(<4 x i64> "reg"="yv" %a0, <4 x i32> "reg"="xv" "tied" %a1, i8 "reg"="k" %a2) "enc"="{ if(a2!=0) evexr(a0, d, 2, 0, 0, 1, 2, 0, a2, 0) | 00100101 | 11 reg:3=a0&7 rm:3=d&7 }" "asm"="vpmovsqd {a0}, {d} {lb}{a2}{rb}" "dclass"="xv" "feature"="avx512f,avx512vl" "nodecode" {
  %r = call <4 x i32> @llvm.x86.avx512.mask.pmovs.qd.256(<4 x i64> %a0, <4 x i32> %a1, i8 %a2)
  ret <4 x i32> %r
}

declare <4 x i32> @llvm.x86.avx512.mask.pmovs.qd.256(<4 x i64>, <4 x i32>, i8)

define <8 x i32> @X86.avx512.mask.pmovs.qd.512(<8 x i64> "reg"="zv" %a0, <8 x i32> "reg"="yv" "tied" %a1, i8 "reg"="k" %a2) "enc"="{ if(a2!=0) evexr(a0, d, 2, 0, 0, 2, 2, 0, a2, 0) | 00100101 | 11 reg:3=a0&7 rm:3=d&7 }" "asm"="vpmovsqd {a0}, {d} {lb}{a2}{rb}" "dclass"="yv" "feature"="avx512f" "nodecode" {
  %r = call <8 x i32> @llvm.x86.avx512.mask.pmovs.qd.512(<8 x i64> %a0, <8 x i32> %a1, i8 %a2)
  ret <8 x i32> %r
}

declare <8 x i32> @llvm.x86.avx512.mask.pmovs.qd.512(<8 x i64>, <8 x i32>, i8)

define void @X86.avx512.mask.pmovs.qd.mem.128(ptr "reg"="r64" %a0, <2 x i64> "reg"="xv" %a1, i8 "reg"="k" %a2) "enc"="{ if(a2!=0) evexm(a1, a0, 4, 2, 0, 0, 0, 2, 0, a2, 0) | 00100101 | rm(a1, a0, 4, 1, 0, 0, 8) }" "asm"="vpmovsqd {a1}, 0({a0}) {lb}{a2}{rb}" "feature"="avx512f,avx512vl" "nodecode" {
  call void @llvm.x86.avx512.mask.pmovs.qd.mem.128(ptr %a0, <2 x i64> %a1, i8 %a2)
  ret void
}

declare void @llvm.x86.avx512.mask.pmovs.qd.mem.128(ptr, <2 x i64>, i8)

define void @X86.avx512.mask.pmovs.qd.mem.256(ptr "reg"="r64" %a0, <4 x i64> "reg"="yv" %a1, i8 "reg"="k" %a2) "enc"="{ if(a2!=0) evexm(a1, a0, 4, 2, 0, 0, 1, 2, 0, a2, 0) | 00100101 | rm(a1, a0, 4, 1, 0, 0, 16) }" "asm"="vpmovsqd {a1}, 0({a0}) {lb}{a2}{rb}" "feature"="avx512f,avx512vl" "nodecode" {
  call void @llvm.x86.avx512.mask.pmovs.qd.mem.256(ptr %a0, <4 x i64> %a1, i8 %a2)
  ret void
}

declare void @llvm.x86.avx512.mask.pmovs.qd.mem.256(ptr, <4 x i64>, i8)

define void @X86.avx512.mask.pmovs.qd.mem.512(ptr "reg"="r64" %a0, <8 x i64> "reg"="zv" %a1, i8 "reg"="k" %a2) "enc"="{ if(a2!=0) evexm(a1, a0, 4, 2, 0, 0, 2, 2, 0, a2, 0) | 00100101 | rm(a1, a0, 4, 1, 0, 0, 32) }" "asm"="vpmovsqd {a1}, 0({a0}) {lb}{a2}{rb}" "feature"="avx512f" "nodecode" {
  call void @llvm.x86.avx512.mask.pmovs.qd.mem.512(ptr %a0, <8 x i64> %a1, i8 %a2)
  ret void
}

declare void @llvm.x86.avx512.mask.pmovs.qd.mem.512(ptr, <8 x i64>, i8)

define <8 x i16> @X86.avx512.mask.pmovs.qw.128(<2 x i64> "reg"="xv" %a0, <8 x i16> "reg"="xv" "tied" %a1, i8 "reg"="k" %a2) "enc"="{ if(a2!=0) evexr(a0, d, 2, 0, 0, 0, 2, 0, a2, 0) | 00100100 | 11 reg:3=a0&7 rm:3=d&7 }" "asm"="vpmovsqw {a0}, {d} {lb}{a2}{rb}" "dclass"="xv" "feature"="avx512f,avx512vl" "nodecode" {
  %r = call <8 x i16> @llvm.x86.avx512.mask.pmovs.qw.128(<2 x i64> %a0, <8 x i16> %a1, i8 %a2)
  ret <8 x i16> %r
}

declare <8 x i16> @llvm.x86.avx512.mask.pmovs.qw.128(<2 x i64>, <8 x i16>, i8)

define <8 x i16> @X86.avx512.mask.pmovs.qw.256(<4 x i64> "reg"="yv" %a0, <8 x i16> "reg"="xv" "tied" %a1, i8 "reg"="k" %a2) "enc"="{ if(a2!=0) evexr(a0, d, 2, 0, 0, 1, 2, 0, a2, 0) | 00100100 | 11 reg:3=a0&7 rm:3=d&7 }" "asm"="vpmovsqw {a0}, {d} {lb}{a2}{rb}" "dclass"="xv" "feature"="avx512f,avx512vl" "nodecode" {
  %r = call <8 x i16> @llvm.x86.avx512.mask.pmovs.qw.256(<4 x i64> %a0, <8 x i16> %a1, i8 %a2)
  ret <8 x i16> %r
}

declare <8 x i16> @llvm.x86.avx512.mask.pmovs.qw.256(<4 x i64>, <8 x i16>, i8)

define <8 x i16> @X86.avx512.mask.pmovs.qw.512(<8 x i64> "reg"="zv" %a0, <8 x i16> "reg"="xv" "tied" %a1, i8 "reg"="k" %a2) "enc"="{ if(a2!=0) evexr(a0, d, 2, 0, 0, 2, 2, 0, a2, 0) | 00100100 | 11 reg:3=a0&7 rm:3=d&7 }" "asm"="vpmovsqw {a0}, {d} {lb}{a2}{rb}" "dclass"="xv" "feature"="avx512f" "nodecode" {
  %r = call <8 x i16> @llvm.x86.avx512.mask.pmovs.qw.512(<8 x i64> %a0, <8 x i16> %a1, i8 %a2)
  ret <8 x i16> %r
}

declare <8 x i16> @llvm.x86.avx512.mask.pmovs.qw.512(<8 x i64>, <8 x i16>, i8)

define void @X86.avx512.mask.pmovs.qw.mem.128(ptr "reg"="r64" %a0, <2 x i64> "reg"="xv" %a1, i8 "reg"="k" %a2) "enc"="{ if(a2!=0) evexm(a1, a0, 4, 2, 0, 0, 0, 2, 0, a2, 0) | 00100100 | rm(a1, a0, 4, 1, 0, 0, 4) }" "asm"="vpmovsqw {a1}, 0({a0}) {lb}{a2}{rb}" "feature"="avx512f,avx512vl" "nodecode" {
  call void @llvm.x86.avx512.mask.pmovs.qw.mem.128(ptr %a0, <2 x i64> %a1, i8 %a2)
  ret void
}

declare void @llvm.x86.avx512.mask.pmovs.qw.mem.128(ptr, <2 x i64>, i8)

define void @X86.avx512.mask.pmovs.qw.mem.256(ptr "reg"="r64" %a0, <4 x i64> "reg"="yv" %a1, i8 "reg"="k" %a2) "enc"="{ if(a2!=0) evexm(a1, a0, 4, 2, 0, 0, 1, 2, 0, a2, 0) | 00100100 | rm(a1, a0, 4, 1, 0, 0, 8) }" "asm"="vpmovsqw {a1}, 0({a0}) {lb}{a2}{rb}" "feature"="avx512f,avx512vl" "nodecode" {
  call void @llvm.x86.avx512.mask.pmovs.qw.mem.256(ptr %a0, <4 x i64> %a1, i8 %a2)
  ret void
}

declare void @llvm.x86.avx512.mask.pmovs.qw.mem.256(ptr, <4 x i64>, i8)

define void @X86.avx512.mask.pmovs.qw.mem.512(ptr "reg"="r64" %a0, <8 x i64> "reg"="zv" %a1, i8 "reg"="k" %a2) "enc"="{ if(a2!=0) evexm(a1, a0, 4, 2, 0, 0, 2, 2, 0, a2, 0) | 00100100 | rm(a1, a0, 4, 1, 0, 0, 16) }" "asm"="vpmovsqw {a1}, 0({a0}) {lb}{a2}{rb}" "feature"="avx512f" "nodecode" {
  call void @llvm.x86.avx512.mask.pmovs.qw.mem.512(ptr %a0, <8 x i64> %a1, i8 %a2)
  ret void
}

declare void @llvm.x86.avx512.mask.pmovs.qw.mem.512(ptr, <8 x i64>, i8)

define <16 x i8> @X86.avx512.mask.pmovs.wb.128(<8 x i16> "reg"="xv" %a0, <16 x i8> "reg"="xv" "tied" %a1, i8 "reg"="k" %a2) "enc"="{ if(a2!=0) evexr(a0, d, 2, 0, 0, 0, 2, 0, a2, 0) | 00100000 | 11 reg:3=a0&7 rm:3=d&7 }" "asm"="vpmovswb {a0}, {d} {lb}{a2}{rb}" "dclass"="xv" "feature"="avx512bw,avx512vl" "nodecode" {
  %r = call <16 x i8> @llvm.x86.avx512.mask.pmovs.wb.128(<8 x i16> %a0, <16 x i8> %a1, i8 %a2)
  ret <16 x i8> %r
}

declare <16 x i8> @llvm.x86.avx512.mask.pmovs.wb.128(<8 x i16>, <16 x i8>, i8)

define void @X86.avx512.mask.pmovs.wb.mem.512(ptr "reg"="r64" %a0, <32 x i16> "reg"="zv" %a1, i32 "reg"="k" %a2) "enc"="{ if(a2!=0) evexm(a1, a0, 4, 2, 0, 0, 2, 2, 0, a2, 0) | 00100000 | rm(a1, a0, 4, 1, 0, 0, 32) }" "asm"="vpmovswb {a1}, 0({a0}) {lb}{a2}{rb}" "feature"="avx512bw" "nodecode" {
  call void @llvm.x86.avx512.mask.pmovs.wb.mem.512(ptr %a0, <32 x i16> %a1, i32 %a2)
  ret void
}

declare void @llvm.x86.avx512.mask.pmovs.wb.mem.512(ptr, <32 x i16>, i32)

define <16 x i8> @X86.avx512.mask.pmovus.db.128(<4 x i32> "reg"="xv" %a0, <16 x i8> "reg"="xv" "tied" %a1, i8 "reg"="k" %a2) "enc"="{ if(a2!=0) evexr(a0, d, 2, 0, 0, 0, 2, 0, a2, 0) | 00010001 | 11 reg:3=a0&7 rm:3=d&7 }" "asm"="vpmovusdb {a0}, {d} {lb}{a2}{rb}" "dclass"="xv" "feature"="avx512f,avx512vl" "nodecode" {
  %r = call <16 x i8> @llvm.x86.avx512.mask.pmovus.db.128(<4 x i32> %a0, <16 x i8> %a1, i8 %a2)
  ret <16 x i8> %r
}

declare <16 x i8> @llvm.x86.avx512.mask.pmovus.db.128(<4 x i32>, <16 x i8>, i8)

define <16 x i8> @X86.avx512.mask.pmovus.db.256(<8 x i32> "reg"="yv" %a0, <16 x i8> "reg"="xv" "tied" %a1, i8 "reg"="k" %a2) "enc"="{ if(a2!=0) evexr(a0, d, 2, 0, 0, 1, 2, 0, a2, 0) | 00010001 | 11 reg:3=a0&7 rm:3=d&7 }" "asm"="vpmovusdb {a0}, {d} {lb}{a2}{rb}" "dclass"="xv" "feature"="avx512f,avx512vl" "nodecode" {
  %r = call <16 x i8> @llvm.x86.avx512.mask.pmovus.db.256(<8 x i32> %a0, <16 x i8> %a1, i8 %a2)
  ret <16 x i8> %r
}

declare <16 x i8> @llvm.x86.avx512.mask.pmovus.db.256(<8 x i32>, <16 x i8>, i8)

define <16 x i8> @X86.avx512.mask.pmovus.db.512(<16 x i32> "reg"="zv" %a0, <16 x i8> "reg"="xv" "tied" %a1, i16 "reg"="k" %a2) "enc"="{ if(a2!=0) evexr(a0, d, 2, 0, 0, 2, 2, 0, a2, 0) | 00010001 | 11 reg:3=a0&7 rm:3=d&7 }" "asm"="vpmovusdb {a0}, {d} {lb}{a2}{rb}" "dclass"="xv" "feature"="avx512f" "nodecode" {
  %r = call <16 x i8> @llvm.x86.avx512.mask.pmovus.db.512(<16 x i32> %a0, <16 x i8> %a1, i16 %a2)
  ret <16 x i8> %r
}

declare <16 x i8> @llvm.x86.avx512.mask.pmovus.db.512(<16 x i32>, <16 x i8>, i16)

define void @X86.avx512.mask.pmovus.db.mem.128(ptr "reg"="r64" %a0, <4 x i32> "reg"="xv" %a1, i8 "reg"="k" %a2) "enc"="{ if(a2!=0) evexm(a1, a0, 4, 2, 0, 0, 0, 2, 0, a2, 0) | 00010001 | rm(a1, a0, 4, 1, 0, 0, 4) }" "asm"="vpmovusdb {a1}, 0({a0}) {lb}{a2}{rb}" "feature"="avx512f,avx512vl" "nodecode" {
  call void @llvm.x86.avx512.mask.pmovus.db.mem.128(ptr %a0, <4 x i32> %a1, i8 %a2)
  ret void
}

declare void @llvm.x86.avx512.mask.pmovus.db.mem.128(ptr, <4 x i32>, i8)

define void @X86.avx512.mask.pmovus.db.mem.256(ptr "reg"="r64" %a0, <8 x i32> "reg"="yv" %a1, i8 "reg"="k" %a2) "enc"="{ if(a2!=0) evexm(a1, a0, 4, 2, 0, 0, 1, 2, 0, a2, 0) | 00010001 | rm(a1, a0, 4, 1, 0, 0, 8) }" "asm"="vpmovusdb {a1}, 0({a0}) {lb}{a2}{rb}" "feature"="avx512f,avx512vl" "nodecode" {
  call void @llvm.x86.avx512.mask.pmovus.db.mem.256(ptr %a0, <8 x i32> %a1, i8 %a2)
  ret void
}

declare void @llvm.x86.avx512.mask.pmovus.db.mem.256(ptr, <8 x i32>, i8)

define void @X86.avx512.mask.pmovus.db.mem.512(ptr "reg"="r64" %a0, <16 x i32> "reg"="zv" %a1, i16 "reg"="k" %a2) "enc"="{ if(a2!=0) evexm(a1, a0, 4, 2, 0, 0, 2, 2, 0, a2, 0) | 00010001 | rm(a1, a0, 4, 1, 0, 0, 16) }" "asm"="vpmovusdb {a1}, 0({a0}) {lb}{a2}{rb}" "feature"="avx512f" "nodecode" {
  call void @llvm.x86.avx512.mask.pmovus.db.mem.512(ptr %a0, <16 x i32> %a1, i16 %a2)
  ret void
}

declare void @llvm.x86.avx512.mask.pmovus.db.mem.512(ptr, <16 x i32>, i16)

define <8 x i16> @X86.avx512.mask.pmovus.dw.128(<4 x i32> "reg"="xv" %a0, <8 x i16> "reg"="xv" "tied" %a1, i8 "reg"="k" %a2) "enc"="{ if(a2!=0) evexr(a0, d, 2, 0, 0, 0, 2, 0, a2, 0) | 00010011 | 11 reg:3=a0&7 rm:3=d&7 }" "asm"="vpmovusdw {a0}, {d} {lb}{a2}{rb}" "dclass"="xv" "feature"="avx512f,avx512vl" "nodecode" {
  %r = call <8 x i16> @llvm.x86.avx512.mask.pmovus.dw.128(<4 x i32> %a0, <8 x i16> %a1, i8 %a2)
  ret <8 x i16> %r
}

declare <8 x i16> @llvm.x86.avx512.mask.pmovus.dw.128(<4 x i32>, <8 x i16>, i8)

define <8 x i16> @X86.avx512.mask.pmovus.dw.256(<8 x i32> "reg"="yv" %a0, <8 x i16> "reg"="xv" "tied" %a1, i8 "reg"="k" %a2) "enc"="{ if(a2!=0) evexr(a0, d, 2, 0, 0, 1, 2, 0, a2, 0) | 00010011 | 11 reg:3=a0&7 rm:3=d&7 }" "asm"="vpmovusdw {a0}, {d} {lb}{a2}{rb}" "dclass"="xv" "feature"="avx512f,avx512vl" "nodecode" {
  %r = call <8 x i16> @llvm.x86.avx512.mask.pmovus.dw.256(<8 x i32> %a0, <8 x i16> %a1, i8 %a2)
  ret <8 x i16> %r
}

declare <8 x i16> @llvm.x86.avx512.mask.pmovus.dw.256(<8 x i32>, <8 x i16>, i8)

define <16 x i16> @X86.avx512.mask.pmovus.dw.512(<16 x i32> "reg"="zv" %a0, <16 x i16> "reg"="yv" "tied" %a1, i16 "reg"="k" %a2) "enc"="{ if(a2!=0) evexr(a0, d, 2, 0, 0, 2, 2, 0, a2, 0) | 00010011 | 11 reg:3=a0&7 rm:3=d&7 }" "asm"="vpmovusdw {a0}, {d} {lb}{a2}{rb}" "dclass"="yv" "feature"="avx512f" "nodecode" {
  %r = call <16 x i16> @llvm.x86.avx512.mask.pmovus.dw.512(<16 x i32> %a0, <16 x i16> %a1, i16 %a2)
  ret <16 x i16> %r
}

declare <16 x i16> @llvm.x86.avx512.mask.pmovus.dw.512(<16 x i32>, <16 x i16>, i16)

define void @X86.avx512.mask.pmovus.dw.mem.128(ptr "reg"="r64" %a0, <4 x i32> "reg"="xv" %a1, i8 "reg"="k" %a2) "enc"="{ if(a2!=0) evexm(a1, a0, 4, 2, 0, 0, 0, 2, 0, a2, 0) | 00010011 | rm(a1, a0, 4, 1, 0, 0, 8) }" "asm"="vpmovusdw {a1}, 0({a0}) {lb}{a2}{rb}" "feature"="avx512f,avx512vl" "nodecode" {
  call void @llvm.x86.avx512.mask.pmovus.dw.mem.128(ptr %a0, <4 x i32> %a1, i8 %a2)
  ret void
}

declare void @llvm.x86.avx512.mask.pmovus.dw.mem.128(ptr, <4 x i32>, i8)

define void @X86.avx512.mask.pmovus.dw.mem.256(ptr "reg"="r64" %a0, <8 x i32> "reg"="yv" %a1, i8 "reg"="k" %a2) "enc"="{ if(a2!=0) evexm(a1, a0, 4, 2, 0, 0, 1, 2, 0, a2, 0) | 00010011 | rm(a1, a0, 4, 1, 0, 0, 16) }" "asm"="vpmovusdw {a1}, 0({a0}) {lb}{a2}{rb}" "feature"="avx512f,avx512vl" "nodecode" {
  call void @llvm.x86.avx512.mask.pmovus.dw.mem.256(ptr %a0, <8 x i32> %a1, i8 %a2)
  ret void
}

declare void @llvm.x86.avx512.mask.pmovus.dw.mem.256(ptr, <8 x i32>, i8)

define void @X86.avx512.mask.pmovus.dw.mem.512(ptr "reg"="r64" %a0, <16 x i32> "reg"="zv" %a1, i16 "reg"="k" %a2) "enc"="{ if(a2!=0) evexm(a1, a0, 4, 2, 0, 0, 2, 2, 0, a2, 0) | 00010011 | rm(a1, a0, 4, 1, 0, 0, 32) }" "asm"="vpmovusdw {a1}, 0({a0}) {lb}{a2}{rb}" "feature"="avx512f" "nodecode" {
  call void @llvm.x86.avx512.mask.pmovus.dw.mem.512(ptr %a0, <16 x i32> %a1, i16 %a2)
  ret void
}

declare void @llvm.x86.avx512.mask.pmovus.dw.mem.512(ptr, <16 x i32>, i16)

define <16 x i8> @X86.avx512.mask.pmovus.qb.128(<2 x i64> "reg"="xv" %a0, <16 x i8> "reg"="xv" "tied" %a1, i8 "reg"="k" %a2) "enc"="{ if(a2!=0) evexr(a0, d, 2, 0, 0, 0, 2, 0, a2, 0) | 00010010 | 11 reg:3=a0&7 rm:3=d&7 }" "asm"="vpmovusqb {a0}, {d} {lb}{a2}{rb}" "dclass"="xv" "feature"="avx512f,avx512vl" "nodecode" {
  %r = call <16 x i8> @llvm.x86.avx512.mask.pmovus.qb.128(<2 x i64> %a0, <16 x i8> %a1, i8 %a2)
  ret <16 x i8> %r
}

declare <16 x i8> @llvm.x86.avx512.mask.pmovus.qb.128(<2 x i64>, <16 x i8>, i8)

define <16 x i8> @X86.avx512.mask.pmovus.qb.256(<4 x i64> "reg"="yv" %a0, <16 x i8> "reg"="xv" "tied" %a1, i8 "reg"="k" %a2) "enc"="{ if(a2!=0) evexr(a0, d, 2, 0, 0, 1, 2, 0, a2, 0) | 00010010 | 11 reg:3=a0&7 rm:3=d&7 }" "asm"="vpmovusqb {a0}, {d} {lb}{a2}{rb}" "dclass"="xv" "feature"="avx512f,avx512vl" "nodecode" {
  %r = call <16 x i8> @llvm.x86.avx512.mask.pmovus.qb.256(<4 x i64> %a0, <16 x i8> %a1, i8 %a2)
  ret <16 x i8> %r
}

declare <16 x i8> @llvm.x86.avx512.mask.pmovus.qb.256(<4 x i64>, <16 x i8>, i8)

define <16 x i8> @X86.avx512.mask.pmovus.qb.512(<8 x i64> "reg"="zv" %a0, <16 x i8> "reg"="xv" "tied" %a1, i8 "reg"="k" %a2) "enc"="{ if(a2!=0) evexr(a0, d, 2, 0, 0, 2, 2, 0, a2, 0) | 00010010 | 11 reg:3=a0&7 rm:3=d&7 }" "asm"="vpmovusqb {a0}, {d} {lb}{a2}{rb}" "dclass"="xv" "feature"="avx512f" "nodecode" {
  %r = call <16 x i8> @llvm.x86.avx512.mask.pmovus.qb.512(<8 x i64> %a0, <16 x i8> %a1, i8 %a2)
  ret <16 x i8> %r
}

declare <16 x i8> @llvm.x86.avx512.mask.pmovus.qb.512(<8 x i64>, <16 x i8>, i8)

define void @X86.avx512.mask.pmovus.qb.mem.128(ptr "reg"="r64" %a0, <2 x i64> "reg"="xv" %a1, i8 "reg"="k" %a2) "enc"="{ if(a2!=0) evexm(a1, a0, 4, 2, 0, 0, 0, 2, 0, a2, 0) | 00010010 | rm(a1, a0, 4, 1, 0, 0, 2) }" "asm"="vpmovusqb {a1}, 0({a0}) {lb}{a2}{rb}" "feature"="avx512f,avx512vl" "nodecode" {
  call void @llvm.x86.avx512.mask.pmovus.qb.mem.128(ptr %a0, <2 x i64> %a1, i8 %a2)
  ret void
}

declare void @llvm.x86.avx512.mask.pmovus.qb.mem.128(ptr, <2 x i64>, i8)

define void @X86.avx512.mask.pmovus.qb.mem.256(ptr "reg"="r64" %a0, <4 x i64> "reg"="yv" %a1, i8 "reg"="k" %a2) "enc"="{ if(a2!=0) evexm(a1, a0, 4, 2, 0, 0, 1, 2, 0, a2, 0) | 00010010 | rm(a1, a0, 4, 1, 0, 0, 4) }" "asm"="vpmovusqb {a1}, 0({a0}) {lb}{a2}{rb}" "feature"="avx512f,avx512vl" "nodecode" {
  call void @llvm.x86.avx512.mask.pmovus.qb.mem.256(ptr %a0, <4 x i64> %a1, i8 %a2)
  ret void
}

declare void @llvm.x86.avx512.mask.pmovus.qb.mem.256(ptr, <4 x i64>, i8)

define void @X86.avx512.mask.pmovus.qb.mem.512(ptr "reg"="r64" %a0, <8 x i64> "reg"="zv" %a1, i8 "reg"="k" %a2) "enc"="{ if(a2!=0) evexm(a1, a0, 4, 2, 0, 0, 2, 2, 0, a2, 0) | 00010010 | rm(a1, a0, 4, 1, 0, 0, 8) }" "asm"="vpmovusqb {a1}, 0({a0}) {lb}{a2}{rb}" "feature"="avx512f" "nodecode" {
  call void @llvm.x86.avx512.mask.pmovus.qb.mem.512(ptr %a0, <8 x i64> %a1, i8 %a2)
  ret void
}

declare void @llvm.x86.avx512.mask.pmovus.qb.mem.512(ptr, <8 x i64>, i8)

define <4 x i32> @X86.avx512.mask.pmovus.qd.128(<2 x i64> "reg"="xv" %a0, <4 x i32> "reg"="xv" "tied" %a1, i8 "reg"="k" %a2) "enc"="{ if(a2!=0) evexr(a0, d, 2, 0, 0, 0, 2, 0, a2, 0) | 00010101 | 11 reg:3=a0&7 rm:3=d&7 }" "asm"="vpmovusqd {a0}, {d} {lb}{a2}{rb}" "dclass"="xv" "feature"="avx512f,avx512vl" "nodecode" {
  %r = call <4 x i32> @llvm.x86.avx512.mask.pmovus.qd.128(<2 x i64> %a0, <4 x i32> %a1, i8 %a2)
  ret <4 x i32> %r
}

declare <4 x i32> @llvm.x86.avx512.mask.pmovus.qd.128(<2 x i64>, <4 x i32>, i8)

define <4 x i32> @X86.avx512.mask.pmovus.qd.256(<4 x i64> "reg"="yv" %a0, <4 x i32> "reg"="xv" "tied" %a1, i8 "reg"="k" %a2) "enc"="{ if(a2!=0) evexr(a0, d, 2, 0, 0, 1, 2, 0, a2, 0) | 00010101 | 11 reg:3=a0&7 rm:3=d&7 }" "asm"="vpmovusqd {a0}, {d} {lb}{a2}{rb}" "dclass"="xv" "feature"="avx512f,avx512vl" "nodecode" {
  %r = call <4 x i32> @llvm.x86.avx512.mask.pmovus.qd.256(<4 x i64> %a0, <4 x i32> %a1, i8 %a2)
  ret <4 x i32> %r
}

declare <4 x i32> @llvm.x86.avx512.mask.pmovus.qd.256(<4 x i64>, <4 x i32>, i8)

define <8 x i32> @X86.avx512.mask.pmovus.qd.512(<8 x i64> "reg"="zv" %a0, <8 x i32> "reg"="yv" "tied" %a1, i8 "reg"="k" %a2) "enc"="{ if(a2!=0) evexr(a0, d, 2, 0, 0, 2, 2, 0, a2, 0) | 00010101 | 11 reg:3=a0&7 rm:3=d&7 }" "asm"="vpmovusqd {a0}, {d} {lb}{a2}{rb}" "dclass"="yv" "feature"="avx512f" "nodecode" {
  %r = call <8 x i32> @llvm.x86.avx512.mask.pmovus.qd.512(<8 x i64> %a0, <8 x i32> %a1, i8 %a2)
  ret <8 x i32> %r
}

declare <8 x i32> @llvm.x86.avx512.mask.pmovus.qd.512(<8 x i64>, <8 x i32>, i8)

define void @X86.avx512.mask.pmovus.qd.mem.128(ptr "reg"="r64" %a0, <2 x i64> "reg"="xv" %a1, i8 "reg"="k" %a2) "enc"="{ if(a2!=0) evexm(a1, a0, 4, 2, 0, 0, 0, 2, 0, a2, 0) | 00010101 | rm(a1, a0, 4, 1, 0, 0, 8) }" "asm"="vpmovusqd {a1}, 0({a0}) {lb}{a2}{rb}" "feature"="avx512f,avx512vl" "nodecode" {
  call void @llvm.x86.avx512.mask.pmovus.qd.mem.128(ptr %a0, <2 x i64> %a1, i8 %a2)
  ret void
}

declare void @llvm.x86.avx512.mask.pmovus.qd.mem.128(ptr, <2 x i64>, i8)

define void @X86.avx512.mask.pmovus.qd.mem.256(ptr "reg"="r64" %a0, <4 x i64> "reg"="yv" %a1, i8 "reg"="k" %a2) "enc"="{ if(a2!=0) evexm(a1, a0, 4, 2, 0, 0, 1, 2, 0, a2, 0) | 00010101 | rm(a1, a0, 4, 1, 0, 0, 16) }" "asm"="vpmovusqd {a1}, 0({a0}) {lb}{a2}{rb}" "feature"="avx512f,avx512vl" "nodecode" {
  call void @llvm.x86.avx512.mask.pmovus.qd.mem.256(ptr %a0, <4 x i64> %a1, i8 %a2)
  ret void
}

declare void @llvm.x86.avx512.mask.pmovus.qd.mem.256(ptr, <4 x i64>, i8)

define void @X86.avx512.mask.pmovus.qd.mem.512(ptr "reg"="r64" %a0, <8 x i64> "reg"="zv" %a1, i8 "reg"="k" %a2) "enc"="{ if(a2!=0) evexm(a1, a0, 4, 2, 0, 0, 2, 2, 0, a2, 0) | 00010101 | rm(a1, a0, 4, 1, 0, 0, 32) }" "asm"="vpmovusqd {a1}, 0({a0}) {lb}{a2}{rb}" "feature"="avx512f" "nodecode" {
  call void @llvm.x86.avx512.mask.pmovus.qd.mem.512(ptr %a0, <8 x i64> %a1, i8 %a2)
  ret void
}

declare void @llvm.x86.avx512.mask.pmovus.qd.mem.512(ptr, <8 x i64>, i8)

define <8 x i16> @X86.avx512.mask.pmovus.qw.128(<2 x i64> "reg"="xv" %a0, <8 x i16> "reg"="xv" "tied" %a1, i8 "reg"="k" %a2) "enc"="{ if(a2!=0) evexr(a0, d, 2, 0, 0, 0, 2, 0, a2, 0) | 00010100 | 11 reg:3=a0&7 rm:3=d&7 }" "asm"="vpmovusqw {a0}, {d} {lb}{a2}{rb}" "dclass"="xv" "feature"="avx512f,avx512vl" "nodecode" {
  %r = call <8 x i16> @llvm.x86.avx512.mask.pmovus.qw.128(<2 x i64> %a0, <8 x i16> %a1, i8 %a2)
  ret <8 x i16> %r
}

declare <8 x i16> @llvm.x86.avx512.mask.pmovus.qw.128(<2 x i64>, <8 x i16>, i8)

define <8 x i16> @X86.avx512.mask.pmovus.qw.256(<4 x i64> "reg"="yv" %a0, <8 x i16> "reg"="xv" "tied" %a1, i8 "reg"="k" %a2) "enc"="{ if(a2!=0) evexr(a0, d, 2, 0, 0, 1, 2, 0, a2, 0) | 00010100 | 11 reg:3=a0&7 rm:3=d&7 }" "asm"="vpmovusqw {a0}, {d} {lb}{a2}{rb}" "dclass"="xv" "feature"="avx512f,avx512vl" "nodecode" {
  %r = call <8 x i16> @llvm.x86.avx512.mask.pmovus.qw.256(<4 x i64> %a0, <8 x i16> %a1, i8 %a2)
  ret <8 x i16> %r
}

declare <8 x i16> @llvm.x86.avx512.mask.pmovus.qw.256(<4 x i64>, <8 x i16>, i8)

define <8 x i16> @X86.avx512.mask.pmovus.qw.512(<8 x i64> "reg"="zv" %a0, <8 x i16> "reg"="xv" "tied" %a1, i8 "reg"="k" %a2) "enc"="{ if(a2!=0) evexr(a0, d, 2, 0, 0, 2, 2, 0, a2, 0) | 00010100 | 11 reg:3=a0&7 rm:3=d&7 }" "asm"="vpmovusqw {a0}, {d} {lb}{a2}{rb}" "dclass"="xv" "feature"="avx512f" "nodecode" {
  %r = call <8 x i16> @llvm.x86.avx512.mask.pmovus.qw.512(<8 x i64> %a0, <8 x i16> %a1, i8 %a2)
  ret <8 x i16> %r
}

declare <8 x i16> @llvm.x86.avx512.mask.pmovus.qw.512(<8 x i64>, <8 x i16>, i8)

define void @X86.avx512.mask.pmovus.qw.mem.128(ptr "reg"="r64" %a0, <2 x i64> "reg"="xv" %a1, i8 "reg"="k" %a2) "enc"="{ if(a2!=0) evexm(a1, a0, 4, 2, 0, 0, 0, 2, 0, a2, 0) | 00010100 | rm(a1, a0, 4, 1, 0, 0, 4) }" "asm"="vpmovusqw {a1}, 0({a0}) {lb}{a2}{rb}" "feature"="avx512f,avx512vl" "nodecode" {
  call void @llvm.x86.avx512.mask.pmovus.qw.mem.128(ptr %a0, <2 x i64> %a1, i8 %a2)
  ret void
}

declare void @llvm.x86.avx512.mask.pmovus.qw.mem.128(ptr, <2 x i64>, i8)

define void @X86.avx512.mask.pmovus.qw.mem.256(ptr "reg"="r64" %a0, <4 x i64> "reg"="yv" %a1, i8 "reg"="k" %a2) "enc"="{ if(a2!=0) evexm(a1, a0, 4, 2, 0, 0, 1, 2, 0, a2, 0) | 00010100 | rm(a1, a0, 4, 1, 0, 0, 8) }" "asm"="vpmovusqw {a1}, 0({a0}) {lb}{a2}{rb}" "feature"="avx512f,avx512vl" "nodecode" {
  call void @llvm.x86.avx512.mask.pmovus.qw.mem.256(ptr %a0, <4 x i64> %a1, i8 %a2)
  ret void
}

declare void @llvm.x86.avx512.mask.pmovus.qw.mem.256(ptr, <4 x i64>, i8)

define void @X86.avx512.mask.pmovus.qw.mem.512(ptr "reg"="r64" %a0, <8 x i64> "reg"="zv" %a1, i8 "reg"="k" %a2) "enc"="{ if(a2!=0) evexm(a1, a0, 4, 2, 0, 0, 2, 2, 0, a2, 0) | 00010100 | rm(a1, a0, 4, 1, 0, 0, 16) }" "asm"="vpmovusqw {a1}, 0({a0}) {lb}{a2}{rb}" "feature"="avx512f" "nodecode" {
  call void @llvm.x86.avx512.mask.pmovus.qw.mem.512(ptr %a0, <8 x i64> %a1, i8 %a2)
  ret void
}

declare void @llvm.x86.avx512.mask.pmovus.qw.mem.512(ptr, <8 x i64>, i8)

define <16 x i8> @X86.avx512.mask.pmovus.wb.128(<8 x i16> "reg"="xv" %a0, <16 x i8> "reg"="xv" "tied" %a1, i8 "reg"="k" %a2) "enc"="{ if(a2!=0) evexr(a0, d, 2, 0, 0, 0, 2, 0, a2, 0) | 00010000 | 11 reg:3=a0&7 rm:3=d&7 }" "asm"="vpmovuswb {a0}, {d} {lb}{a2}{rb}" "dclass"="xv" "feature"="avx512bw,avx512vl" "nodecode" {
  %r = call <16 x i8> @llvm.x86.avx512.mask.pmovus.wb.128(<8 x i16> %a0, <16 x i8> %a1, i8 %a2)
  ret <16 x i8> %r
}

declare <16 x i8> @llvm.x86.avx512.mask.pmovus.wb.128(<8 x i16>, <16 x i8>, i8)

define void @X86.avx512.mask.pmovus.wb.mem.512(ptr "reg"="r64" %a0, <32 x i16> "reg"="zv" %a1, i32 "reg"="k" %a2) "enc"="{ if(a2!=0) evexm(a1, a0, 4, 2, 0, 0, 2, 2, 0, a2, 0) | 00010000 | rm(a1, a0, 4, 1, 0, 0, 32) }" "asm"="vpmovuswb {a1}, 0({a0}) {lb}{a2}{rb}" "feature"="avx512bw" "nodecode" {
  call void @llvm.x86.avx512.mask.pmovus.wb.mem.512(ptr %a0, <32 x i16> %a1, i32 %a2)
  ret void
}

declare void @llvm.x86.avx512.mask.pmovus.wb.mem.512(ptr, <32 x i16>, i32)

define <2 x double> @X86.avx512.mask.range.pd.128(<2 x double> "reg"="xv" %a0, <2 x double> "reg"="xv" %a1, i32 "imm" %a2, <2 x double> "reg"="xv" "tied" %a3, i8 "reg"="k" %a4) "enc"="{ if(a4!=0) evexr(d, a1, 3, 1, a0, 0, 1, 0, a4, 0) | 01010000 | 11 reg:3=d&7 rm:3=a1&7 | { i1:8=a2 ; i1:s8=a2 } }" "asm"="vrangepd ${a2}, {a1}, {a0}, {d} {lb}{a4}{rb}" "dclass"="xv" "feature"="avx512dq,avx512vl" "nodecode" {
  %r = call <2 x double> @llvm.x86.avx512.mask.range.pd.128(<2 x double> %a0, <2 x double> %a1, i32 %a2, <2 x double> %a3, i8 %a4)
  ret <2 x double> %r
}

declare <2 x double> @llvm.x86.avx512.mask.range.pd.128(<2 x double>, <2 x double>, i32, <2 x double>, i8)

define <4 x double> @X86.avx512.mask.range.pd.256(<4 x double> "reg"="yv" %a0, <4 x double> "reg"="yv" %a1, i32 "imm" %a2, <4 x double> "reg"="yv" "tied" %a3, i8 "reg"="k" %a4) "enc"="{ if(a4!=0) evexr(d, a1, 3, 1, a0, 1, 1, 0, a4, 0) | 01010000 | 11 reg:3=d&7 rm:3=a1&7 | { i1:8=a2 ; i1:s8=a2 } }" "asm"="vrangepd ${a2}, {a1}, {a0}, {d} {lb}{a4}{rb}" "dclass"="yv" "feature"="avx512dq,avx512vl" "nodecode" {
  %r = call <4 x double> @llvm.x86.avx512.mask.range.pd.256(<4 x double> %a0, <4 x double> %a1, i32 %a2, <4 x double> %a3, i8 %a4)
  ret <4 x double> %r
}

declare <4 x double> @llvm.x86.avx512.mask.range.pd.256(<4 x double>, <4 x double>, i32, <4 x double>, i8)

define <8 x double> @X86.avx512.mask.range.pd.512(<8 x double> "reg"="zv" %a0, <8 x double> "reg"="zv" %a1, i32 "imm" %a2, <8 x double> "reg"="zv" "tied" %a3, i8 "reg"="k" %a4) "enc"="{ if(a4!=0) evexr(d, a1, 3, 1, a0, 2, 1, 0, a4, 0) | 01010000 | 11 reg:3=d&7 rm:3=a1&7 | { i1:8=a2 ; i1:s8=a2 } }" "asm"="vrangepd ${a2}, {a1}, {a0}, {d} {lb}{a4}{rb}" "dclass"="zv" "feature"="avx512dq" "nodecode" {
  %r = call <8 x double> @llvm.x86.avx512.mask.range.pd.512(<8 x double> %a0, <8 x double> %a1, i32 %a2, <8 x double> %a3, i8 %a4, i32 4)
  ret <8 x double> %r
}

declare <8 x double> @llvm.x86.avx512.mask.range.pd.512(<8 x double>, <8 x double>, i32, <8 x double>, i8, i32)

define <4 x float> @X86.avx512.mask.range.ps.128(<4 x float> "reg"="xv" %a0, <4 x float> "reg"="xv" %a1, i32 "imm" %a2, <4 x float> "reg"="xv" "tied" %a3, i8 "reg"="k" %a4) "enc"="{ if(a4!=0) evexr(d, a1, 3, 0, a0, 0, 1, 0, a4, 0) | 01010000 | 11 reg:3=d&7 rm:3=a1&7 | { i1:8=a2 ; i1:s8=a2 } }" "asm"="vrangeps ${a2}, {a1}, {a0}, {d} {lb}{a4}{rb}" "dclass"="xv" "feature"="avx512dq,avx512vl" "nodecode" {
  %r = call <4 x float> @llvm.x86.avx512.mask.range.ps.128(<4 x float> %a0, <4 x float> %a1, i32 %a2, <4 x float> %a3, i8 %a4)
  ret <4 x float> %r
}

declare <4 x float> @llvm.x86.avx512.mask.range.ps.128(<4 x float>, <4 x float>, i32, <4 x float>, i8)

define <8 x float> @X86.avx512.mask.range.ps.256(<8 x float> "reg"="yv" %a0, <8 x float> "reg"="yv" %a1, i32 "imm" %a2, <8 x float> "reg"="yv" "tied" %a3, i8 "reg"="k" %a4) "enc"="{ if(a4!=0) evexr(d, a1, 3, 0, a0, 1, 1, 0, a4, 0) | 01010000 | 11 reg:3=d&7 rm:3=a1&7 | { i1:8=a2 ; i1:s8=a2 } }" "asm"="vrangeps ${a2}, {a1}, {a0}, {d} {lb}{a4}{rb}" "dclass"="yv" "feature"="avx512dq,avx512vl" "nodecode" {
  %r = call <8 x float> @llvm.x86.avx512.mask.range.ps.256(<8 x float> %a0, <8 x float> %a1, i32 %a2, <8 x float> %a3, i8 %a4)
  ret <8 x float> %r
}

declare <8 x float> @llvm.x86.avx512.mask.range.ps.256(<8 x float>, <8 x float>, i32, <8 x float>, i8)

define <16 x float> @X86.avx512.mask.range.ps.512(<16 x float> "reg"="zv" %a0, <16 x float> "reg"="zv" %a1, i32 "imm" %a2, <16 x float> "reg"="zv" "tied" %a3, i16 "reg"="k" %a4) "enc"="{ if(a4!=0) evexr(d, a1, 3, 0, a0, 2, 1, 0, a4, 0) | 01010000 | 11 reg:3=d&7 rm:3=a1&7 | { i1:8=a2 ; i1:s8=a2 } }" "asm"="vrangeps ${a2}, {a1}, {a0}, {d} {lb}{a4}{rb}" "dclass"="zv" "feature"="avx512dq" "nodecode" {
  %r = call <16 x float> @llvm.x86.avx512.mask.range.ps.512(<16 x float> %a0, <16 x float> %a1, i32 %a2, <16 x float> %a3, i16 %a4, i32 4)
  ret <16 x float> %r
}

declare <16 x float> @llvm.x86.avx512.mask.range.ps.512(<16 x float>, <16 x float>, i32, <16 x float>, i16, i32)

define <2 x double> @X86.avx512.mask.range.sd(<2 x double> "reg"="xv" %a0, <2 x double> "reg"="xv" %a1, <2 x double> "reg"="xv" "tied" %a2, i8 "reg"="k" %a3, i32 "imm" %a4) "enc"="{ if(a3!=0) evexr(d, a1, 3, 1, a0, 0, 1, 0, a3, 0) | 01010001 | 11 reg:3=d&7 rm:3=a1&7 | { i1:8=a4 ; i1:s8=a4 } }" "asm"="vrangesd ${a4}, {a1}, {a0}, {d} {lb}{a3}{rb}" "dclass"="xv" "feature"="avx512dq" "nodecode" {
  %r = call <2 x double> @llvm.x86.avx512.mask.range.sd(<2 x double> %a0, <2 x double> %a1, <2 x double> %a2, i8 %a3, i32 %a4, i32 4)
  ret <2 x double> %r
}

declare <2 x double> @llvm.x86.avx512.mask.range.sd(<2 x double>, <2 x double>, <2 x double>, i8, i32, i32)

define <4 x float> @X86.avx512.mask.range.ss(<4 x float> "reg"="xv" %a0, <4 x float> "reg"="xv" %a1, <4 x float> "reg"="xv" "tied" %a2, i8 "reg"="k" %a3, i32 "imm" %a4) "enc"="{ if(a3!=0) evexr(d, a1, 3, 0, a0, 0, 1, 0, a3, 0) | 01010001 | 11 reg:3=d&7 rm:3=a1&7 | { i1:8=a4 ; i1:s8=a4 } }" "asm"="vrangess ${a4}, {a1}, {a0}, {d} {lb}{a3}{rb}" "dclass"="xv" "feature"="avx512dq" "nodecode" {
  %r = call <4 x float> @llvm.x86.avx512.mask.range.ss(<4 x float> %a0, <4 x float> %a1, <4 x float> %a2, i8 %a3, i32 %a4, i32 4)
  ret <4 x float> %r
}

declare <4 x float> @llvm.x86.avx512.mask.range.ss(<4 x float>, <4 x float>, <4 x float>, i8, i32, i32)

define <2 x double> @X86.avx512.mask.reduce.pd.128(<2 x double> "reg"="xv" %a0, i32 "imm" %a1, <2 x double> "reg"="xv" "tied" %a2, i8 "reg"="k" %a3) "enc"="{ if(a3!=0) evexr(d, a0, 3, 1, 0, 0, 1, 0, a3, 0) | 01010110 | 11 reg:3=d&7 rm:3=a0&7 | { i1:8=a1 ; i1:s8=a1 } }" "asm"="vreducepd ${a1}, {a0}, {d} {lb}{a3}{rb}" "dclass"="xv" "feature"="avx512dq,avx512vl" "nodecode" {
  %r = call <2 x double> @llvm.x86.avx512.mask.reduce.pd.128(<2 x double> %a0, i32 %a1, <2 x double> %a2, i8 %a3)
  ret <2 x double> %r
}

declare <2 x double> @llvm.x86.avx512.mask.reduce.pd.128(<2 x double>, i32, <2 x double>, i8)

define <4 x double> @X86.avx512.mask.reduce.pd.256(<4 x double> "reg"="yv" %a0, i32 "imm" %a1, <4 x double> "reg"="yv" "tied" %a2, i8 "reg"="k" %a3) "enc"="{ if(a3!=0) evexr(d, a0, 3, 1, 0, 1, 1, 0, a3, 0) | 01010110 | 11 reg:3=d&7 rm:3=a0&7 | { i1:8=a1 ; i1:s8=a1 } }" "asm"="vreducepd ${a1}, {a0}, {d} {lb}{a3}{rb}" "dclass"="yv" "feature"="avx512dq,avx512vl" "nodecode" {
  %r = call <4 x double> @llvm.x86.avx512.mask.reduce.pd.256(<4 x double> %a0, i32 %a1, <4 x double> %a2, i8 %a3)
  ret <4 x double> %r
}

declare <4 x double> @llvm.x86.avx512.mask.reduce.pd.256(<4 x double>, i32, <4 x double>, i8)

define <8 x double> @X86.avx512.mask.reduce.pd.512(<8 x double> "reg"="zv" %a0, i32 "imm" %a1, <8 x double> "reg"="zv" "tied" %a2, i8 "reg"="k" %a3) "enc"="{ if(a3!=0) evexr(d, a0, 3, 1, 0, 2, 1, 0, a3, 0) | 01010110 | 11 reg:3=d&7 rm:3=a0&7 | { i1:8=a1 ; i1:s8=a1 } }" "asm"="vreducepd ${a1}, {a0}, {d} {lb}{a3}{rb}" "dclass"="zv" "feature"="avx512dq" "nodecode" {
  %r = call <8 x double> @llvm.x86.avx512.mask.reduce.pd.512(<8 x double> %a0, i32 %a1, <8 x double> %a2, i8 %a3, i32 4)
  ret <8 x double> %r
}

declare <8 x double> @llvm.x86.avx512.mask.reduce.pd.512(<8 x double>, i32, <8 x double>, i8, i32)

define <4 x float> @X86.avx512.mask.reduce.ps.128(<4 x float> "reg"="xv" %a0, i32 "imm" %a1, <4 x float> "reg"="xv" "tied" %a2, i8 "reg"="k" %a3) "enc"="{ if(a3!=0) evexr(d, a0, 3, 0, 0, 0, 1, 0, a3, 0) | 01010110 | 11 reg:3=d&7 rm:3=a0&7 | { i1:8=a1 ; i1:s8=a1 } }" "asm"="vreduceps ${a1}, {a0}, {d} {lb}{a3}{rb}" "dclass"="xv" "feature"="avx512dq,avx512vl" "nodecode" {
  %r = call <4 x float> @llvm.x86.avx512.mask.reduce.ps.128(<4 x float> %a0, i32 %a1, <4 x float> %a2, i8 %a3)
  ret <4 x float> %r
}

declare <4 x float> @llvm.x86.avx512.mask.reduce.ps.128(<4 x float>, i32, <4 x float>, i8)

define <8 x float> @X86.avx512.mask.reduce.ps.256(<8 x float> "reg"="yv" %a0, i32 "imm" %a1, <8 x float> "reg"="yv" "tied" %a2, i8 "reg"="k" %a3) "enc"="{ if(a3!=0) evexr(d, a0, 3, 0, 0, 1, 1, 0, a3, 0) | 01010110 | 11 reg:3=d&7 rm:3=a0&7 | { i1:8=a1 ; i1:s8=a1 } }" "asm"="vreduceps ${a1}, {a0}, {d} {lb}{a3}{rb}" "dclass"="yv" "feature"="avx512dq,avx512vl" "nodecode" {
  %r = call <8 x float> @llvm.x86.avx512.mask.reduce.ps.256(<8 x float> %a0, i32 %a1, <8 x float> %a2, i8 %a3)
  ret <8 x float> %r
}

declare <8 x float> @llvm.x86.avx512.mask.reduce.ps.256(<8 x float>, i32, <8 x float>, i8)

define <16 x float> @X86.avx512.mask.reduce.ps.512(<16 x float> "reg"="zv" %a0, i32 "imm" %a1, <16 x float> "reg"="zv" "tied" %a2, i16 "reg"="k" %a3) "enc"="{ if(a3!=0) evexr(d, a0, 3, 0, 0, 2, 1, 0, a3, 0) | 01010110 | 11 reg:3=d&7 rm:3=a0&7 | { i1:8=a1 ; i1:s8=a1 } }" "asm"="vreduceps ${a1}, {a0}, {d} {lb}{a3}{rb}" "dclass"="zv" "feature"="avx512dq" "nodecode" {
  %r = call <16 x float> @llvm.x86.avx512.mask.reduce.ps.512(<16 x float> %a0, i32 %a1, <16 x float> %a2, i16 %a3, i32 4)
  ret <16 x float> %r
}

declare <16 x float> @llvm.x86.avx512.mask.reduce.ps.512(<16 x float>, i32, <16 x float>, i16, i32)

define <2 x double> @X86.avx512.mask.reduce.sd(<2 x double> "reg"="xv" %a0, <2 x double> "reg"="xv" %a1, <2 x double> "reg"="xv" "tied" %a2, i8 "reg"="k" %a3, i32 "imm" %a4) "enc"="{ if(a3!=0) evexr(d, a1, 3, 1, a0, 0, 1, 0, a3, 0) | 01010111 | 11 reg:3=d&7 rm:3=a1&7 | { i1:8=a4 ; i1:s8=a4 } }" "asm"="vreducesd ${a4}, {a1}, {a0}, {d} {lb}{a3}{rb}" "dclass"="xv" "feature"="avx512dq" "nodecode" {
  %r = call <2 x double> @llvm.x86.avx512.mask.reduce.sd(<2 x double> %a0, <2 x double> %a1, <2 x double> %a2, i8 %a3, i32 %a4, i32 4)
  ret <2 x double> %r
}

declare <2 x double> @llvm.x86.avx512.mask.reduce.sd(<2 x double>, <2 x double>, <2 x double>, i8, i32, i32)

define <4 x float> @X86.avx512.mask.reduce.ss(<4 x float> "reg"="xv" %a0, <4 x float> "reg"="xv" %a1, <4 x float> "reg"="xv" "tied" %a2, i8 "reg"="k" %a3, i32 "imm" %a4) "enc"="{ if(a3!=0) evexr(d, a1, 3, 0, a0, 0, 1, 0, a3, 0) | 01010111 | 11 reg:3=d&7 rm:3=a1&7 | { i1:8=a4 ; i1:s8=a4 } }" "asm"="vreducess ${a4}, {a1}, {a0}, {d} {lb}{a3}{rb}" "dclass"="xv" "feature"="avx512dq" "nodecode" {
  %r = call <4 x float> @llvm.x86.avx512.mask.reduce.ss(<4 x float> %a0, <4 x float> %a1, <4 x float> %a2, i8 %a3, i32 %a4, i32 4)
  ret <4 x float> %r
}

declare <4 x float> @llvm.x86.avx512.mask.reduce.ss(<4 x float>, <4 x float>, <4 x float>, i8, i32, i32)

define <2 x double> @X86.avx512.mask.rndscale.pd.128(<2 x double> "reg"="xv" %a0, i32 "imm" %a1, <2 x double> "reg"="xv" "tied" %a2, i8 "reg"="k" %a3) "enc"="{ if(a3!=0) evexr(d, a0, 3, 1, 0, 0, 1, 0, a3, 0) | 00001001 | 11 reg:3=d&7 rm:3=a0&7 | { i1:8=a1 ; i1:s8=a1 } }" "asm"="vrndscalepd ${a1}, {a0}, {d} {lb}{a3}{rb}" "dclass"="xv" "feature"="avx512f,avx512vl" "nodecode" {
  %r = call <2 x double> @llvm.x86.avx512.mask.rndscale.pd.128(<2 x double> %a0, i32 %a1, <2 x double> %a2, i8 %a3)
  ret <2 x double> %r
}

declare <2 x double> @llvm.x86.avx512.mask.rndscale.pd.128(<2 x double>, i32, <2 x double>, i8)

define <4 x double> @X86.avx512.mask.rndscale.pd.256(<4 x double> "reg"="yv" %a0, i32 "imm" %a1, <4 x double> "reg"="yv" "tied" %a2, i8 "reg"="k" %a3) "enc"="{ if(a3!=0) evexr(d, a0, 3, 1, 0, 1, 1, 0, a3, 0) | 00001001 | 11 reg:3=d&7 rm:3=a0&7 | { i1:8=a1 ; i1:s8=a1 } }" "asm"="vrndscalepd ${a1}, {a0}, {d} {lb}{a3}{rb}" "dclass"="yv" "feature"="avx512f,avx512vl" "nodecode" {
  %r = call <4 x double> @llvm.x86.avx512.mask.rndscale.pd.256(<4 x double> %a0, i32 %a1, <4 x double> %a2, i8 %a3)
  ret <4 x double> %r
}

declare <4 x double> @llvm.x86.avx512.mask.rndscale.pd.256(<4 x double>, i32, <4 x double>, i8)

define <8 x double> @X86.avx512.mask.rndscale.pd.512(<8 x double> "reg"="zv" %a0, i32 "imm" %a1, <8 x double> "reg"="zv" "tied" %a2, i8 "reg"="k" %a3) "enc"="{ if(a3!=0) evexr(d, a0, 3, 1, 0, 2, 1, 0, a3, 0) | 00001001 | 11 reg:3=d&7 rm:3=a0&7 | { i1:8=a1 ; i1:s8=a1 } }" "asm"="vrndscalepd ${a1}, {a0}, {d} {lb}{a3}{rb}" "dclass"="zv" "feature"="avx512f" "nodecode" {
  %r = call <8 x double> @llvm.x86.avx512.mask.rndscale.pd.512(<8 x double> %a0, i32 %a1, <8 x double> %a2, i8 %a3, i32 4)
  ret <8 x double> %r
}

declare <8 x double> @llvm.x86.avx512.mask.rndscale.pd.512(<8 x double>, i32, <8 x double>, i8, i32)

define <4 x float> @X86.avx512.mask.rndscale.ps.128(<4 x float> "reg"="xv" %a0, i32 "imm" %a1, <4 x float> "reg"="xv" "tied" %a2, i8 "reg"="k" %a3) "enc"="{ if(a3!=0) evexr(d, a0, 3, 0, 0, 0, 1, 0, a3, 0) | 00001000 | 11 reg:3=d&7 rm:3=a0&7 | { i1:8=a1 ; i1:s8=a1 } }" "asm"="vrndscaleps ${a1}, {a0}, {d} {lb}{a3}{rb}" "dclass"="xv" "feature"="avx512f,avx512vl" "nodecode" {
  %r = call <4 x float> @llvm.x86.avx512.mask.rndscale.ps.128(<4 x float> %a0, i32 %a1, <4 x float> %a2, i8 %a3)
  ret <4 x float> %r
}

declare <4 x float> @llvm.x86.avx512.mask.rndscale.ps.128(<4 x float>, i32, <4 x float>, i8)

define <8 x float> @X86.avx512.mask.rndscale.ps.256(<8 x float> "reg"="yv" %a0, i32 "imm" %a1, <8 x float> "reg"="yv" "tied" %a2, i8 "reg"="k" %a3) "enc"="{ if(a3!=0) evexr(d, a0, 3, 0, 0, 1, 1, 0, a3, 0) | 00001000 | 11 reg:3=d&7 rm:3=a0&7 | { i1:8=a1 ; i1:s8=a1 } }" "asm"="vrndscaleps ${a1}, {a0}, {d} {lb}{a3}{rb}" "dclass"="yv" "feature"="avx512f,avx512vl" "nodecode" {
  %r = call <8 x float> @llvm.x86.avx512.mask.rndscale.ps.256(<8 x float> %a0, i32 %a1, <8 x float> %a2, i8 %a3)
  ret <8 x float> %r
}

declare <8 x float> @llvm.x86.avx512.mask.rndscale.ps.256(<8 x float>, i32, <8 x float>, i8)

define <16 x float> @X86.avx512.mask.rndscale.ps.512(<16 x float> "reg"="zv" %a0, i32 "imm" %a1, <16 x float> "reg"="zv" "tied" %a2, i16 "reg"="k" %a3) "enc"="{ if(a3!=0) evexr(d, a0, 3, 0, 0, 2, 1, 0, a3, 0) | 00001000 | 11 reg:3=d&7 rm:3=a0&7 | { i1:8=a1 ; i1:s8=a1 } }" "asm"="vrndscaleps ${a1}, {a0}, {d} {lb}{a3}{rb}" "dclass"="zv" "feature"="avx512f" "nodecode" {
  %r = call <16 x float> @llvm.x86.avx512.mask.rndscale.ps.512(<16 x float> %a0, i32 %a1, <16 x float> %a2, i16 %a3, i32 4)
  ret <16 x float> %r
}

declare <16 x float> @llvm.x86.avx512.mask.rndscale.ps.512(<16 x float>, i32, <16 x float>, i16, i32)

define <2 x double> @X86.avx512.mask.rndscale.sd(<2 x double> "reg"="xv" %a0, <2 x double> "reg"="xv" %a1, <2 x double> "reg"="xv" "tied" %a2, i8 "reg"="k" %a3, i32 "imm" %a4) "enc"="{ if(a3!=0) evexr(d, a1, 3, 1, a0, 0, 1, 0, a3, 0) | 00001011 | 11 reg:3=d&7 rm:3=a1&7 | { i1:8=a4 ; i1:s8=a4 } }" "asm"="vrndscalesd ${a4}, {a1}, {a0}, {d} {lb}{a3}{rb}" "dclass"="xv" "feature"="avx512f" "nodecode" {
  %r = call <2 x double> @llvm.x86.avx512.mask.rndscale.sd(<2 x double> %a0, <2 x double> %a1, <2 x double> %a2, i8 %a3, i32 %a4, i32 4)
  ret <2 x double> %r
}

declare <2 x double> @llvm.x86.avx512.mask.rndscale.sd(<2 x double>, <2 x double>, <2 x double>, i8, i32, i32)

define <4 x float> @X86.avx512.mask.rndscale.ss(<4 x float> "reg"="xv" %a0, <4 x float> "reg"="xv" %a1, <4 x float> "reg"="xv" "tied" %a2, i8 "reg"="k" %a3, i32 "imm" %a4) "enc"="{ if(a3!=0) evexr(d, a1, 3, 0, a0, 0, 1, 0, a3, 0) | 00001010 | 11 reg:3=d&7 rm:3=a1&7 | { i1:8=a4 ; i1:s8=a4 } }" "asm"="vrndscaless ${a4}, {a1}, {a0}, {d} {lb}{a3}{rb}" "dclass"="xv" "feature"="avx512f" "nodecode" {
  %r = call <4 x float> @llvm.x86.avx512.mask.rndscale.ss(<4 x float> %a0, <4 x float> %a1, <4 x float> %a2, i8 %a3, i32 %a4, i32 4)
  ret <4 x float> %r
}

declare <4 x float> @llvm.x86.avx512.mask.rndscale.ss(<4 x float>, <4 x float>, <4 x float>, i8, i32, i32)

define <2 x double> @X86.avx512.mask.scalef.pd.128(<2 x double> "reg"="xv" %a0, <2 x double> "reg"="xv" %a1, <2 x double> "reg"="xv" "tied" %a2, i8 "reg"="k" %a3) "enc"="{ if(a3!=0) evexr(d, a1, 2, 1, a0, 0, 1, 0, a3, 0) | 00101100 | 11 reg:3=d&7 rm:3=a1&7 }" "asm"="vscalefpd {a1}, {a0}, {d} {lb}{a3}{rb}" "dclass"="xv" "feature"="avx512f,avx512vl" "nodecode" {
  %r = call <2 x double> @llvm.x86.avx512.mask.scalef.pd.128(<2 x double> %a0, <2 x double> %a1, <2 x double> %a2, i8 %a3)
  ret <2 x double> %r
}

declare <2 x double> @llvm.x86.avx512.mask.scalef.pd.128(<2 x double>, <2 x double>, <2 x double>, i8)

define <4 x double> @X86.avx512.mask.scalef.pd.256(<4 x double> "reg"="yv" %a0, <4 x double> "reg"="yv" %a1, <4 x double> "reg"="yv" "tied" %a2, i8 "reg"="k" %a3) "enc"="{ if(a3!=0) evexr(d, a1, 2, 1, a0, 1, 1, 0, a3, 0) | 00101100 | 11 reg:3=d&7 rm:3=a1&7 }" "asm"="vscalefpd {a1}, {a0}, {d} {lb}{a3}{rb}" "dclass"="yv" "feature"="avx512f,avx512vl" "nodecode" {
  %r = call <4 x double> @llvm.x86.avx512.mask.scalef.pd.256(<4 x double> %a0, <4 x double> %a1, <4 x double> %a2, i8 %a3)
  ret <4 x double> %r
}

declare <4 x double> @llvm.x86.avx512.mask.scalef.pd.256(<4 x double>, <4 x double>, <4 x double>, i8)

define <8 x double> @X86.avx512.mask.scalef.pd.512(<8 x double> "reg"="zv" %a0, <8 x double> "reg"="zv" %a1, <8 x double> "reg"="zv" "tied" %a2, i8 "reg"="k" %a3) "enc"="{ if(a3!=0) evexr(d, a1, 2, 1, a0, 2, 1, 0, a3, 0) | 00101100 | 11 reg:3=d&7 rm:3=a1&7 }" "asm"="vscalefpd {a1}, {a0}, {d} {lb}{a3}{rb}" "dclass"="zv" "feature"="avx512f" "nodecode" {
  %r = call <8 x double> @llvm.x86.avx512.mask.scalef.pd.512(<8 x double> %a0, <8 x double> %a1, <8 x double> %a2, i8 %a3, i32 4)
  ret <8 x double> %r
}

declare <8 x double> @llvm.x86.avx512.mask.scalef.pd.512(<8 x double>, <8 x double>, <8 x double>, i8, i32)

define <4 x float> @X86.avx512.mask.scalef.ps.128(<4 x float> "reg"="xv" %a0, <4 x float> "reg"="xv" %a1, <4 x float> "reg"="xv" "tied" %a2, i8 "reg"="k" %a3) "enc"="{ if(a3!=0) evexr(d, a1, 2, 0, a0, 0, 1, 0, a3, 0) | 00101100 | 11 reg:3=d&7 rm:3=a1&7 }" "asm"="vscalefps {a1}, {a0}, {d} {lb}{a3}{rb}" "dclass"="xv" "feature"="avx512f,avx512vl" "nodecode" {
  %r = call <4 x float> @llvm.x86.avx512.mask.scalef.ps.128(<4 x float> %a0, <4 x float> %a1, <4 x float> %a2, i8 %a3)
  ret <4 x float> %r
}

declare <4 x float> @llvm.x86.avx512.mask.scalef.ps.128(<4 x float>, <4 x float>, <4 x float>, i8)

define <8 x float> @X86.avx512.mask.scalef.ps.256(<8 x float> "reg"="yv" %a0, <8 x float> "reg"="yv" %a1, <8 x float> "reg"="yv" "tied" %a2, i8 "reg"="k" %a3) "enc"="{ if(a3!=0) evexr(d, a1, 2, 0, a0, 1, 1, 0, a3, 0) | 00101100 | 11 reg:3=d&7 rm:3=a1&7 }" "asm"="vscalefps {a1}, {a0}, {d} {lb}{a3}{rb}" "dclass"="yv" "feature"="avx512f,avx512vl" "nodecode" {
  %r = call <8 x float> @llvm.x86.avx512.mask.scalef.ps.256(<8 x float> %a0, <8 x float> %a1, <8 x float> %a2, i8 %a3)
  ret <8 x float> %r
}

declare <8 x float> @llvm.x86.avx512.mask.scalef.ps.256(<8 x float>, <8 x float>, <8 x float>, i8)

define <16 x float> @X86.avx512.mask.scalef.ps.512(<16 x float> "reg"="zv" %a0, <16 x float> "reg"="zv" %a1, <16 x float> "reg"="zv" "tied" %a2, i16 "reg"="k" %a3) "enc"="{ if(a3!=0) evexr(d, a1, 2, 0, a0, 2, 1, 0, a3, 0) | 00101100 | 11 reg:3=d&7 rm:3=a1&7 }" "asm"="vscalefps {a1}, {a0}, {d} {lb}{a3}{rb}" "dclass"="zv" "feature"="avx512f" "nodecode" {
  %r = call <16 x float> @llvm.x86.avx512.mask.scalef.ps.512(<16 x float> %a0, <16 x float> %a1, <16 x float> %a2, i16 %a3, i32 4)
  ret <16 x float> %r
}

declare <16 x float> @llvm.x86.avx512.mask.scalef.ps.512(<16 x float>, <16 x float>, <16 x float>, i16, i32)

define <2 x double> @X86.avx512.mask.scalef.sd(<2 x double> "reg"="xv" %a0, <2 x double> "reg"="xv" %a1, <2 x double> "reg"="xv" "tied" %a2, i8 "reg"="k" %a3) "enc"="{ if(a3!=0) evexr(d, a1, 2, 1, a0, 0, 1, 0, a3, 0) | 00101101 | 11 reg:3=d&7 rm:3=a1&7 }" "asm"="vscalefsd {a1}, {a0}, {d} {lb}{a3}{rb}" "dclass"="xv" "feature"="avx512f" "nodecode" {
  %r = call <2 x double> @llvm.x86.avx512.mask.scalef.sd(<2 x double> %a0, <2 x double> %a1, <2 x double> %a2, i8 %a3, i32 4)
  ret <2 x double> %r
}

declare <2 x double> @llvm.x86.avx512.mask.scalef.sd(<2 x double>, <2 x double>, <2 x double>, i8, i32)

define <4 x float> @X86.avx512.mask.scalef.ss(<4 x float> "reg"="xv" %a0, <4 x float> "reg"="xv" %a1, <4 x float> "reg"="xv" "tied" %a2, i8 "reg"="k" %a3) "enc"="{ if(a3!=0) evexr(d, a1, 2, 0, a0, 0, 1, 0, a3, 0) | 00101101 | 11 reg:3=d&7 rm:3=a1&7 }" "asm"="vscalefss {a1}, {a0}, {d} {lb}{a3}{rb}" "dclass"="xv" "feature"="avx512f" "nodecode" {
  %r = call <4 x float> @llvm.x86.avx512.mask.scalef.ss(<4 x float> %a0, <4 x float> %a1, <4 x float> %a2, i8 %a3, i32 4)
  ret <4 x float> %r
}

declare <4 x float> @llvm.x86.avx512.mask.scalef.ss(<4 x float>, <4 x float>, <4 x float>, i8, i32)

define <2 x double> @X86.avx512.mask.sqrt.sd(<2 x double> "reg"="xv" %a0, <2 x double> "reg"="xv" %a1, <2 x double> "reg"="xv" "tied" %a2, i8 "reg"="k" %a3) "enc"="{ if(a3!=0) evexr(d, a1, 1, 1, a0, 0, 3, 0, a3, 0) | 01010001 | 11 reg:3=d&7 rm:3=a1&7 }" "asm"="vsqrtsd {a1}, {a0}, {d} {lb}{a3}{rb}" "dclass"="xv" "feature"="avx512f" "nodecode" {
  %r = call <2 x double> @llvm.x86.avx512.mask.sqrt.sd(<2 x double> %a0, <2 x double> %a1, <2 x double> %a2, i8 %a3, i32 4)
  ret <2 x double> %r
}

declare <2 x double> @llvm.x86.avx512.mask.sqrt.sd(<2 x double>, <2 x double>, <2 x double>, i8, i32)

define <4 x float> @X86.avx512.mask.sqrt.ss(<4 x float> "reg"="xv" %a0, <4 x float> "reg"="xv" %a1, <4 x float> "reg"="xv" "tied" %a2, i8 "reg"="k" %a3) "enc"="{ if(a3!=0) evexr(d, a1, 1, 0, a0, 0, 2, 0, a3, 0) | 01010001 | 11 reg:3=d&7 rm:3=a1&7 }" "asm"="vsqrtss {a1}, {a0}, {d} {lb}{a3}{rb}" "dclass"="xv" "feature"="avx512f" "nodecode" {
  %r = call <4 x float> @llvm.x86.avx512.mask.sqrt.ss(<4 x float> %a0, <4 x float> %a1, <4 x float> %a2, i8 %a3, i32 4)
  ret <4 x float> %r
}

declare <4 x float> @llvm.x86.avx512.mask.sqrt.ss(<4 x float>, <4 x float>, <4 x float>, i8, i32)

define <2 x double> @X86.avx512.mask.sub.sd.round(<2 x double> "reg"="xv" %a0, <2 x double> "reg"="xv" %a1, <2 x double> "reg"="xv" "tied" %a2, i8 "reg"="k" %a3) "enc"="{ if(a3!=0) evexr(d, a1, 1, 1, a0, 0, 3, 0, a3, 0) | 01011100 | 11 reg:3=d&7 rm:3=a1&7 }" "asm"="vsubsd {a1}, {a0}, {d} {lb}{a3}{rb}" "dclass"="xv" "feature"="avx512f" "nodecode" {
  %r = call <2 x double> @llvm.x86.avx512.mask.sub.sd.round(<2 x double> %a0, <2 x double> %a1, <2 x double> %a2, i8 %a3, i32 4)
  ret <2 x double> %r
}

declare <2 x double> @llvm.x86.avx512.mask.sub.sd.round(<2 x double>, <2 x double>, <2 x double>, i8, i32)

define <4 x float> @X86.avx512.mask.sub.ss.round(<4 x float> "reg"="xv" %a0, <4 x float> "reg"="xv" %a1, <4 x float> "reg"="xv" "tied" %a2, i8 "reg"="k" %a3) "enc"="{ if(a3!=0) evexr(d, a1, 1, 0, a0, 0, 2, 0, a3, 0) | 01011100 | 11 reg:3=d&7 rm:3=a1&7 }" "asm"="vsubss {a1}, {a0}, {d} {lb}{a3}{rb}" "dclass"="xv" "feature"="avx512f" "nodecode" {
  %r = call <4 x float> @llvm.x86.avx512.mask.sub.ss.round(<4 x float> %a0, <4 x float> %a1, <4 x float> %a2, i8 %a3, i32 4)
  ret <4 x float> %r
}

declare <4 x float> @llvm.x86.avx512.mask.sub.ss.round(<4 x float>, <4 x float>, <4 x float>, i8, i32)

define <16 x float> @X86.avx512.mask.vcvtph2ps.512(<16 x i16> "reg"="yv" %a0, <16 x float> "reg"="zv" "tied" %a1, i16 "reg"="k" %a2) "enc"="{ if(a2!=0) evexr(d, a0, 2, 0, 0, 2, 1, 0, a2, 0) | 00010011 | 11 reg:3=d&7 rm:3=a0&7 }" "asm"="vcvtph2ps {a0}, {d} {lb}{a2}{rb}" "dclass"="zv" "feature"="avx512f" "nodecode" {
  %r = call <16 x float> @llvm.x86.avx512.mask.vcvtph2ps.512(<16 x i16> %a0, <16 x float> %a1, i16 %a2, i32 4)
  ret <16 x float> %r
}

declare <16 x float> @llvm.x86.avx512.mask.vcvtph2ps.512(<16 x i16>, <16 x float>, i16, i32)

define <8 x i16> @X86.avx512.mask.vcvtps2ph.128(<4 x float> "reg"="xv" %a0, i32 "imm" %a1, <8 x i16> "reg"="xv" "tied" %a2, i8 "reg"="k" %a3) "enc"="{ if(a3!=0) evexr(a0, d, 3, 0, 0, 0, 1, 0, a3, 0) | 00011101 | 11 reg:3=a0&7 rm:3=d&7 | { i1:8=a1 ; i1:s8=a1 } }" "asm"="vcvtps2ph ${a1}, {a0}, {d} {lb}{a3}{rb}" "dclass"="xv" "feature"="avx512f,avx512vl" "nodecode" {
  %r = call <8 x i16> @llvm.x86.avx512.mask.vcvtps2ph.128(<4 x float> %a0, i32 %a1, <8 x i16> %a2, i8 %a3)
  ret <8 x i16> %r
}

declare <8 x i16> @llvm.x86.avx512.mask.vcvtps2ph.128(<4 x float>, i32, <8 x i16>, i8)

define <8 x i16> @X86.avx512.mask.vcvtps2ph.256(<8 x float> "reg"="yv" %a0, i32 "imm" %a1, <8 x i16> "reg"="xv" "tied" %a2, i8 "reg"="k" %a3) "enc"="{ if(a3!=0) evexr(a0, d, 3, 0, 0, 1, 1, 0, a3, 0) | 00011101 | 11 reg:3=a0&7 rm:3=d&7 | { i1:8=a1 ; i1:s8=a1 } }" "asm"="vcvtps2ph ${a1}, {a0}, {d} {lb}{a3}{rb}" "dclass"="xv" "feature"="avx512f,avx512vl" "nodecode" {
  %r = call <8 x i16> @llvm.x86.avx512.mask.vcvtps2ph.256(<8 x float> %a0, i32 %a1, <8 x i16> %a2, i8 %a3)
  ret <8 x i16> %r
}

declare <8 x i16> @llvm.x86.avx512.mask.vcvtps2ph.256(<8 x float>, i32, <8 x i16>, i8)

define <16 x i16> @X86.avx512.mask.vcvtps2ph.512(<16 x float> "reg"="zv" %a0, i32 "imm" %a1, <16 x i16> "reg"="yv" "tied" %a2, i16 "reg"="k" %a3) "enc"="{ if(a3!=0) evexr(a0, d, 3, 0, 0, 2, 1, 0, a3, 0) | 00011101 | 11 reg:3=a0&7 rm:3=d&7 | { i1:8=a1 ; i1:s8=a1 } }" "asm"="vcvtps2ph ${a1}, {a0}, {d} {lb}{a3}{rb}" "dclass"="yv" "feature"="avx512f" "nodecode" {
  %r = call <16 x i16> @llvm.x86.avx512.mask.vcvtps2ph.512(<16 x float> %a0, i32 %a1, <16 x i16> %a2, i16 %a3)
  ret <16 x i16> %r
}

declare <16 x i16> @llvm.x86.avx512.mask.vcvtps2ph.512(<16 x float>, i32, <16 x i16>, i16)

define <2 x double> @X86.avx512.maskz.fixupimm.pd.128(<2 x double> "reg"="xv" "tied" %a0, <2 x double> "reg"="xv" %a1, <2 x i64> "reg"="xv" %a2, i32 "imm" %a3, i8 "reg"="k" %a4) "enc"="{ if(a4!=0) evexr(d, a2, 3, 1, a1, 0, 1, 0, a4, 1) | 01010100 | 11 reg:3=d&7 rm:3=a2&7 | { i1:8=a3 ; i1:s8=a3 } }" "asm"="vfixupimmpd ${a3}, {a2}, {a1}, {d} {lb}{a4}{rb} {lb}z{rb}" "dclass"="xv" "feature"="avx512f,avx512vl" "nodecode" {
  %r = call <2 x double> @llvm.x86.avx512.maskz.fixupimm.pd.128(<2 x double> %a0, <2 x double> %a1, <2 x i64> %a2, i32 %a3, i8 %a4)
  ret <2 x double> %r
}

declare <2 x double> @llvm.x86.avx512.maskz.fixupimm.pd.128(<2 x double>, <2 x double>, <2 x i64>, i32, i8)

define <4 x double> @X86.avx512.maskz.fixupimm.pd.256(<4 x double> "reg"="yv" "tied" %a0, <4 x double> "reg"="yv" %a1, <4 x i64> "reg"="yv" %a2, i32 "imm" %a3, i8 "reg"="k" %a4) "enc"="{ if(a4!=0) evexr(d, a2, 3, 1, a1, 1, 1, 0, a4, 1) | 01010100 | 11 reg:3=d&7 rm:3=a2&7 | { i1:8=a3 ; i1:s8=a3 } }" "asm"="vfixupimmpd ${a3}, {a2}, {a1}, {d} {lb}{a4}{rb} {lb}z{rb}" "dclass"="yv" "feature"="avx512f,avx512vl" "nodecode" {
  %r = call <4 x double> @llvm.x86.avx512.maskz.fixupimm.pd.256(<4 x double> %a0, <4 x double> %a1, <4 x i64> %a2, i32 %a3, i8 %a4)
  ret <4 x double> %r
}

declare <4 x double> @llvm.x86.avx512.maskz.fixupimm.pd.256(<4 x double>, <4 x double>, <4 x i64>, i32, i8)

define <8 x double> @X86.avx512.maskz.fixupimm.pd.512(<8 x double> "reg"="zv" "tied" %a0, <8 x double> "reg"="zv" %a1, <8 x i64> "reg"="zv" %a2, i32 "imm" %a3, i8 "reg"="k" %a4) "enc"="{ if(a4!=0) evexr(d, a2, 3, 1, a1, 2, 1, 0, a4, 1) | 01010100 | 11 reg:3=d&7 rm:3=a2&7 | { i1:8=a3 ; i1:s8=a3 } }" "asm"="vfixupimmpd ${a3}, {a2}, {a1}, {d} {lb}{a4}{rb} {lb}z{rb}" "dclass"="zv" "feature"="avx512f" "nodecode" {
  %r = call <8 x double> @llvm.x86.avx512.maskz.fixupimm.pd.512(<8 x double> %a0, <8 x double> %a1, <8 x i64> %a2, i32 %a3, i8 %a4, i32 4)
  ret <8 x double> %r
}

declare <8 x double> @llvm.x86.avx512.maskz.fixupimm.pd.512(<8 x double>, <8 x double>, <8 x i64>, i32, i8, i32)

define <4 x float> @X86.avx512.maskz.fixupimm.ps.128(<4 x float> "reg"="xv" "tied" %a0, <4 x float> "reg"="xv" %a1, <4 x i32> "reg"="xv" %a2, i32 "imm" %a3, i8 "reg"="k" %a4) "enc"="{ if(a4!=0) evexr(d, a2, 3, 0, a1, 0, 1, 0, a4, 1) | 01010100 | 11 reg:3=d&7 rm:3=a2&7 | { i1:8=a3 ; i1:s8=a3 } }" "asm"="vfixupimmps ${a3}, {a2}, {a1}, {d} {lb}{a4}{rb} {lb}z{rb}" "dclass"="xv" "feature"="avx512f,avx512vl" "nodecode" {
  %r = call <4 x float> @llvm.x86.avx512.maskz.fixupimm.ps.128(<4 x float> %a0, <4 x float> %a1, <4 x i32> %a2, i32 %a3, i8 %a4)
  ret <4 x float> %r
}

declare <4 x float> @llvm.x86.avx512.maskz.fixupimm.ps.128(<4 x float>, <4 x float>, <4 x i32>, i32, i8)

define <8 x float> @X86.avx512.maskz.fixupimm.ps.256(<8 x float> "reg"="yv" "tied" %a0, <8 x float> "reg"="yv" %a1, <8 x i32> "reg"="yv" %a2, i32 "imm" %a3, i8 "reg"="k" %a4) "enc"="{ if(a4!=0) evexr(d, a2, 3, 0, a1, 1, 1, 0, a4, 1) | 01010100 | 11 reg:3=d&7 rm:3=a2&7 | { i1:8=a3 ; i1:s8=a3 } }" "asm"="vfixupimmps ${a3}, {a2}, {a1}, {d} {lb}{a4}{rb} {lb}z{rb}" "dclass"="yv" "feature"="avx512f,avx512vl" "nodecode" {
  %r = call <8 x float> @llvm.x86.avx512.maskz.fixupimm.ps.256(<8 x float> %a0, <8 x float> %a1, <8 x i32> %a2, i32 %a3, i8 %a4)
  ret <8 x float> %r
}

declare <8 x float> @llvm.x86.avx512.maskz.fixupimm.ps.256(<8 x float>, <8 x float>, <8 x i32>, i32, i8)

define <16 x float> @X86.avx512.maskz.fixupimm.ps.512(<16 x float> "reg"="zv" "tied" %a0, <16 x float> "reg"="zv" %a1, <16 x i32> "reg"="zv" %a2, i32 "imm" %a3, i16 "reg"="k" %a4) "enc"="{ if(a4!=0) evexr(d, a2, 3, 0, a1, 2, 1, 0, a4, 1) | 01010100 | 11 reg:3=d&7 rm:3=a2&7 | { i1:8=a3 ; i1:s8=a3 } }" "asm"="vfixupimmps ${a3}, {a2}, {a1}, {d} {lb}{a4}{rb} {lb}z{rb}" "dclass"="zv" "feature"="avx512f" "nodecode" {
  %r = call <16 x float> @llvm.x86.avx512.maskz.fixupimm.ps.512(<16 x float> %a0, <16 x float> %a1, <16 x i32> %a2, i32 %a3, i16 %a4, i32 4)
  ret <16 x float> %r
}

declare <16 x float> @llvm.x86.avx512.maskz.fixupimm.ps.512(<16 x float>, <16 x float>, <16 x i32>, i32, i16, i32)

define <2 x double> @X86.avx512.maskz.fixupimm.sd(<2 x double> "reg"="xv" "tied" %a0, <2 x double> "reg"="xv" %a1, <2 x i64> "reg"="xv" %a2, i32 "imm" %a3, i8 "reg"="k" %a4) "enc"="{ if(a4!=0) evexr(d, a2, 3, 1, a1, 0, 1, 0, a4, 1) | 01010101 | 11 reg:3=d&7 rm:3=a2&7 | { i1:8=a3 ; i1:s8=a3 } }" "asm"="vfixupimmsd ${a3}, {a2}, {a1}, {d} {lb}{a4}{rb} {lb}z{rb}" "dclass"="xv" "feature"="avx512f" "nodecode" {
  %r = call <2 x double> @llvm.x86.avx512.maskz.fixupimm.sd(<2 x double> %a0, <2 x double> %a1, <2 x i64> %a2, i32 %a3, i8 %a4, i32 4)
  ret <2 x double> %r
}

declare <2 x double> @llvm.x86.avx512.maskz.fixupimm.sd(<2 x double>, <2 x double>, <2 x i64>, i32, i8, i32)

define <4 x float> @X86.avx512.maskz.fixupimm.ss(<4 x float> "reg"="xv" "tied" %a0, <4 x float> "reg"="xv" %a1, <4 x i32> "reg"="xv" %a2, i32 "imm" %a3, i8 "reg"="k" %a4) "enc"="{ if(a4!=0) evexr(d, a2, 3, 0, a1, 0, 1, 0, a4, 1) | 01010101 | 11 reg:3=d&7 rm:3=a2&7 | { i1:8=a3 ; i1:s8=a3 } }" "asm"="vfixupimmss ${a3}, {a2}, {a1}, {d} {lb}{a4}{rb} {lb}z{rb}" "dclass"="xv" "feature"="avx512f" "nodecode" {
  %r = call <4 x float> @llvm.x86.avx512.maskz.fixupimm.ss(<4 x float> %a0, <4 x float> %a1, <4 x i32> %a2, i32 %a3, i8 %a4, i32 4)
  ret <4 x float> %r
}

declare <4 x float> @llvm.x86.avx512.maskz.fixupimm.ss(<4 x float>, <4 x float>, <4 x i32>, i32, i8, i32)

define <8 x double> @X86.avx512.max.pd.512(<8 x double> "reg"="zv" %a0, <8 x double> "reg"="zv" %a1) "enc"="evexr(d, a1, 1, 1, a0, 2, 1, 0, 0, 0) | 01011111 | 11 reg:3=d&7 rm:3=a1&7" "asm"="vmaxpd {a1}, {a0}, {d}" "dclass"="zv" "feature"="avx512f" "nodecode" {
  %r = call <8 x double> @llvm.x86.avx512.max.pd.512(<8 x double> %a0, <8 x double> %a1, i32 4)
  ret <8 x double> %r
}

declare <8 x double> @llvm.x86.avx512.max.pd.512(<8 x double>, <8 x double>, i32)

define <16 x float> @X86.avx512.max.ps.512(<16 x float> "reg"="zv" %a0, <16 x float> "reg"="zv" %a1) "enc"="evexr(d, a1, 1, 0, a0, 2, 0, 0, 0, 0) | 01011111 | 11 reg:3=d&7 rm:3=a1&7" "asm"="vmaxps {a1}, {a0}, {d}" "dclass"="zv" "feature"="avx512f" "nodecode" {
  %r = call <16 x float> @llvm.x86.avx512.max.ps.512(<16 x float> %a0, <16 x float> %a1, i32 4)
  ret <16 x float> %r
}

declare <16 x float> @llvm.x86.avx512.max.ps.512(<16 x float>, <16 x float>, i32)

define <8 x double> @X86.avx512.min.pd.512(<8 x double> "reg"="zv" %a0, <8 x double> "reg"="zv" %a1) "enc"="evexr(d, a1, 1, 1, a0, 2, 1, 0, 0, 0) | 01011101 | 11 reg:3=d&7 rm:3=a1&7" "asm"="vminpd {a1}, {a0}, {d}" "dclass"="zv" "feature"="avx512f" "nodecode" {
  %r = call <8 x double> @llvm.x86.avx512.min.pd.512(<8 x double> %a0, <8 x double> %a1, i32 4)
  ret <8 x double> %r
}

declare <8 x double> @llvm.x86.avx512.min.pd.512(<8 x double>, <8 x double>, i32)

define <16 x float> @X86.avx512.min.ps.512(<16 x float> "reg"="zv" %a0, <16 x float> "reg"="zv" %a1) "enc"="evexr(d, a1, 1, 0, a0, 2, 0, 0, 0, 0) | 01011101 | 11 reg:3=d&7 rm:3=a1&7" "asm"="vminps {a1}, {a0}, {d}" "dclass"="zv" "feature"="avx512f" "nodecode" {
  %r = call <16 x float> @llvm.x86.avx512.min.ps.512(<16 x float> %a0, <16 x float> %a1, i32 4)
  ret <16 x float> %r
}

declare <16 x float> @llvm.x86.avx512.min.ps.512(<16 x float>, <16 x float>, i32)

define <8 x double> @X86.avx512.mul.pd.512(<8 x double> "reg"="zv" %a0, <8 x double> "reg"="zv" %a1) "enc"="evexr(d, a1, 1, 1, a0, 2, 1, 0, 0, 0) | 01011001 | 11 reg:3=d&7 rm:3=a1&7" "asm"="vmulpd {a1}, {a0}, {d}" "dclass"="zv" "feature"="avx512f" "nodecode" {
  %r = call <8 x double> @llvm.x86.avx512.mul.pd.512(<8 x double> %a0, <8 x double> %a1, i32 4)
  ret <8 x double> %r
}

declare <8 x double> @llvm.x86.avx512.mul.pd.512(<8 x double>, <8 x double>, i32)

define <16 x float> @X86.avx512.mul.ps.512(<16 x float> "reg"="zv" %a0, <16 x float> "reg"="zv" %a1) "enc"="evexr(d, a1, 1, 0, a0, 2, 0, 0, 0, 0) | 01011001 | 11 reg:3=d&7 rm:3=a1&7" "asm"="vmulps {a1}, {a0}, {d}" "dclass"="zv" "feature"="avx512f" "nodecode" {
  %r = call <16 x float> @llvm.x86.avx512.mul.ps.512(<16 x float> %a0, <16 x float> %a1, i32 4)
  ret <16 x float> %r
}

declare <16 x float> @llvm.x86.avx512.mul.ps.512(<16 x float>, <16 x float>, i32)

define <32 x i16> @X86.avx512.packssdw.512(<16 x i32> "reg"="zv" %a0, <16 x i32> "reg"="zv" %a1) "enc"="evexr(d, a1, 1, 0, a0, 2, 1, 0, 0, 0) | 01101011 | 11 reg:3=d&7 rm:3=a1&7" "asm"="vpackssdw {a1}, {a0}, {d}" "dclass"="zv" "feature"="avx512bw" "nodecode" {
  %r = call <32 x i16> @llvm.x86.avx512.packssdw.512(<16 x i32> %a0, <16 x i32> %a1)
  ret <32 x i16> %r
}

declare <32 x i16> @llvm.x86.avx512.packssdw.512(<16 x i32>, <16 x i32>)

define <64 x i8> @X86.avx512.packsswb.512(<32 x i16> "reg"="zv" %a0, <32 x i16> "reg"="zv" %a1) "enc"="evexr(d, a1, 1, 0, a0, 2, 1, 0, 0, 0) | 01100011 | 11 reg:3=d&7 rm:3=a1&7" "asm"="vpacksswb {a1}, {a0}, {d}" "dclass"="zv" "feature"="avx512bw" "nodecode" {
  %r = call <64 x i8> @llvm.x86.avx512.packsswb.512(<32 x i16> %a0, <32 x i16> %a1)
  ret <64 x i8> %r
}

declare <64 x i8> @llvm.x86.avx512.packsswb.512(<32 x i16>, <32 x i16>)

define <32 x i16> @X86.avx512.packusdw.512(<16 x i32> "reg"="zv" %a0, <16 x i32> "reg"="zv" %a1) "enc"="evexr(d, a1, 2, 0, a0, 2, 1, 0, 0, 0) | 00101011 | 11 reg:3=d&7 rm:3=a1&7" "asm"="vpackusdw {a1}, {a0}, {d}" "dclass"="zv" "feature"="avx512bw" "nodecode" {
  %r = call <32 x i16> @llvm.x86.avx512.packusdw.512(<16 x i32> %a0, <16 x i32> %a1)
  ret <32 x i16> %r
}

declare <32 x i16> @llvm.x86.avx512.packusdw.512(<16 x i32>, <16 x i32>)

define <64 x i8> @X86.avx512.packuswb.512(<32 x i16> "reg"="zv" %a0, <32 x i16> "reg"="zv" %a1) "enc"="evexr(d, a1, 1, 0, a0, 2, 1, 0, 0, 0) | 01100111 | 11 reg:3=d&7 rm:3=a1&7" "asm"="vpackuswb {a1}, {a0}, {d}" "dclass"="zv" "feature"="avx512bw" "nodecode" {
  %r = call <64 x i8> @llvm.x86.avx512.packuswb.512(<32 x i16> %a0, <32 x i16> %a1)
  ret <64 x i8> %r
}

declare <64 x i8> @llvm.x86.avx512.packuswb.512(<32 x i16>, <32 x i16>)

define <4 x double> @X86.avx512.permvar.df.256(<4 x double> "reg"="yv" %a0, <4 x i64> "reg"="yv" %a1) "enc"="evexr(d, a0, 2, 1, a1, 1, 1, 0, 0, 0) | 00010110 | 11 reg:3=d&7 rm:3=a0&7" "asm"="vpermpd {a0}, {a1}, {d}" "dclass"="yv" "feature"="avx512f,avx512vl" "nodecode" {
  %r = call <4 x double> @llvm.x86.avx512.permvar.df.256(<4 x double> %a0, <4 x i64> %a1)
  ret <4 x double> %r
}

declare <4 x double> @llvm.x86.avx512.permvar.df.256(<4 x double>, <4 x i64>)

define <8 x double> @X86.avx512.permvar.df.512(<8 x double> "reg"="zv" %a0, <8 x i64> "reg"="zv" %a1) "enc"="evexr(d, a0, 2, 1, a1, 2, 1, 0, 0, 0) | 00010110 | 11 reg:3=d&7 rm:3=a0&7" "asm"="vpermpd {a0}, {a1}, {d}" "dclass"="zv" "feature"="avx512f" "nodecode" {
  %r = call <8 x double> @llvm.x86.avx512.permvar.df.512(<8 x double> %a0, <8 x i64> %a1)
  ret <8 x double> %r
}

declare <8 x double> @llvm.x86.avx512.permvar.df.512(<8 x double>, <8 x i64>)

define <4 x i64> @X86.avx512.permvar.di.256(<4 x i64> "reg"="yv" %a0, <4 x i64> "reg"="yv" %a1) "enc"="evexr(d, a0, 2, 1, a1, 1, 1, 0, 0, 0) | 00010110 | 11 reg:3=d&7 rm:3=a0&7" "asm"="vpermpd {a0}, {a1}, {d}" "dclass"="yv" "feature"="avx512f,avx512vl" "nodecode" {
  %r = call <4 x i64> @llvm.x86.avx512.permvar.di.256(<4 x i64> %a0, <4 x i64> %a1)
  ret <4 x i64> %r
}

declare <4 x i64> @llvm.x86.avx512.permvar.di.256(<4 x i64>, <4 x i64>)

define <8 x i64> @X86.avx512.permvar.di.512(<8 x i64> "reg"="zv" %a0, <8 x i64> "reg"="zv" %a1) "enc"="evexr(d, a0, 2, 1, a1, 2, 1, 0, 0, 0) | 00010110 | 11 reg:3=d&7 rm:3=a0&7" "asm"="vpermpd {a0}, {a1}, {d}" "dclass"="zv" "feature"="avx512f" "nodecode" {
  %r = call <8 x i64> @llvm.x86.avx512.permvar.di.512(<8 x i64> %a0, <8 x i64> %a1)
  ret <8 x i64> %r
}

declare <8 x i64> @llvm.x86.avx512.permvar.di.512(<8 x i64>, <8 x i64>)

define <8 x i16> @X86.avx512.permvar.hi.128(<8 x i16> "reg"="xv" %a0, <8 x i16> "reg"="xv" %a1) "enc"="evexr(d, a0, 2, 1, a1, 0, 1, 0, 0, 0) | 10001101 | 11 reg:3=d&7 rm:3=a0&7" "asm"="vpermw {a0}, {a1}, {d}" "dclass"="xv" "feature"="avx512bw,avx512vl" "nodecode" {
  %r = call <8 x i16> @llvm.x86.avx512.permvar.hi.128(<8 x i16> %a0, <8 x i16> %a1)
  ret <8 x i16> %r
}

declare <8 x i16> @llvm.x86.avx512.permvar.hi.128(<8 x i16>, <8 x i16>)

define <16 x i16> @X86.avx512.permvar.hi.256(<16 x i16> "reg"="yv" %a0, <16 x i16> "reg"="yv" %a1) "enc"="evexr(d, a0, 2, 1, a1, 1, 1, 0, 0, 0) | 10001101 | 11 reg:3=d&7 rm:3=a0&7" "asm"="vpermw {a0}, {a1}, {d}" "dclass"="yv" "feature"="avx512bw,avx512vl" "nodecode" {
  %r = call <16 x i16> @llvm.x86.avx512.permvar.hi.256(<16 x i16> %a0, <16 x i16> %a1)
  ret <16 x i16> %r
}

declare <16 x i16> @llvm.x86.avx512.permvar.hi.256(<16 x i16>, <16 x i16>)

define <32 x i16> @X86.avx512.permvar.hi.512(<32 x i16> "reg"="zv" %a0, <32 x i16> "reg"="zv" %a1) "enc"="evexr(d, a0, 2, 1, a1, 2, 1, 0, 0, 0) | 10001101 | 11 reg:3=d&7 rm:3=a0&7" "asm"="vpermw {a0}, {a1}, {d}" "dclass"="zv" "feature"="avx512bw" "nodecode" {
  %r = call <32 x i16> @llvm.x86.avx512.permvar.hi.512(<32 x i16> %a0, <32 x i16> %a1)
  ret <32 x i16> %r
}

declare <32 x i16> @llvm.x86.avx512.permvar.hi.512(<32 x i16>, <32 x i16>)

define <16 x i8> @X86.avx512.permvar.qi.128(<16 x i8> "reg"="xv" %a0, <16 x i8> "reg"="xv" %a1) "enc"="evexr(d, a0, 2, 0, a1, 0, 1, 0, 0, 0) | 10001101 | 11 reg:3=d&7 rm:3=a0&7" "asm"="vpermb {a0}, {a1}, {d}" "dclass"="xv" "feature"="avx512vbmi,avx512vl" "nodecode" {
  %r = call <16 x i8> @llvm.x86.avx512.permvar.qi.128(<16 x i8> %a0, <16 x i8> %a1)
  ret <16 x i8> %r
}

declare <16 x i8> @llvm.x86.avx512.permvar.qi.128(<16 x i8>, <16 x i8>)

define <32 x i8> @X86.avx512.permvar.qi.256(<32 x i8> "reg"="yv" %a0, <32 x i8> "reg"="yv" %a1) "enc"="evexr(d, a0, 2, 0, a1, 1, 1, 0, 0, 0) | 10001101 | 11 reg:3=d&7 rm:3=a0&7" "asm"="vpermb {a0}, {a1}, {d}" "dclass"="yv" "feature"="avx512vbmi,avx512vl" "nodecode" {
  %r = call <32 x i8> @llvm.x86.avx512.permvar.qi.256(<32 x i8> %a0, <32 x i8> %a1)
  ret <32 x i8> %r
}

declare <32 x i8> @llvm.x86.avx512.permvar.qi.256(<32 x i8>, <32 x i8>)

define <64 x i8> @X86.avx512.permvar.qi.512(<64 x i8> "reg"="zv" %a0, <64 x i8> "reg"="zv" %a1) "enc"="evexr(d, a0, 2, 0, a1, 2, 1, 0, 0, 0) | 10001101 | 11 reg:3=d&7 rm:3=a0&7" "asm"="vpermb {a0}, {a1}, {d}" "dclass"="zv" "feature"="avx512vbmi" "nodecode" {
  %r = call <64 x i8> @llvm.x86.avx512.permvar.qi.512(<64 x i8> %a0, <64 x i8> %a1)
  ret <64 x i8> %r
}

declare <64 x i8> @llvm.x86.avx512.permvar.qi.512(<64 x i8>, <64 x i8>)

define <16 x float> @X86.avx512.permvar.sf.512(<16 x float> "reg"="zv" %a0, <16 x i32> "reg"="zv" %a1) "enc"="evexr(d, a0, 2, 0, a1, 2, 1, 0, 0, 0) | 00010110 | 11 reg:3=d&7 rm:3=a0&7" "asm"="vpermps {a0}, {a1}, {d}" "dclass"="zv" "feature"="avx512f" "nodecode" {
  %r = call <16 x float> @llvm.x86.avx512.permvar.sf.512(<16 x float> %a0, <16 x i32> %a1)
  ret <16 x float> %r
}

declare <16 x float> @llvm.x86.avx512.permvar.sf.512(<16 x float>, <16 x i32>)

define <16 x i32> @X86.avx512.permvar.si.512(<16 x i32> "reg"="zv" %a0, <16 x i32> "reg"="zv" %a1) "enc"="evexr(d, a0, 2, 0, a1, 2, 1, 0, 0, 0) | 00010110 | 11 reg:3=d&7 rm:3=a0&7" "asm"="vpermps {a0}, {a1}, {d}" "dclass"="zv" "feature"="avx512f" "nodecode" {
  %r = call <16 x i32> @llvm.x86.avx512.permvar.si.512(<16 x i32> %a0, <16 x i32> %a1)
  ret <16 x i32> %r
}

declare <16 x i32> @llvm.x86.avx512.permvar.si.512(<16 x i32>, <16 x i32>)

define <32 x i16> @X86.avx512.pmaddubs.w.512(<64 x i8> "reg"="zv" %a0, <64 x i8> "reg"="zv" %a1) "enc"="evexr(d, a1, 2, 0, a0, 2, 1, 0, 0, 0) | 00000100 | 11 reg:3=d&7 rm:3=a1&7" "asm"="vpmaddubsw {a1}, {a0}, {d}" "dclass"="zv" "feature"="avx512bw" "nodecode" {
  %r = call <32 x i16> @llvm.x86.avx512.pmaddubs.w.512(<64 x i8> %a0, <64 x i8> %a1)
  ret <32 x i16> %r
}

declare <32 x i16> @llvm.x86.avx512.pmaddubs.w.512(<64 x i8>, <64 x i8>)

define <16 x i32> @X86.avx512.pmaddw.d.512(<32 x i16> "reg"="zv" %a0, <32 x i16> "reg"="zv" %a1) "enc"="evexr(d, a1, 1, 0, a0, 2, 1, 0, 0, 0) | 11110101 | 11 reg:3=d&7 rm:3=a1&7" "asm"="vpmaddwd {a1}, {a0}, {d}" "dclass"="zv" "feature"="avx512bw" "nodecode" {
  %r = call <16 x i32> @llvm.x86.avx512.pmaddw.d.512(<32 x i16> %a0, <32 x i16> %a1)
  ret <16 x i32> %r
}

declare <16 x i32> @llvm.x86.avx512.pmaddw.d.512(<32 x i16>, <32 x i16>)

define <32 x i16> @X86.avx512.pmul.hr.sw.512(<32 x i16> "reg"="zv" %a0, <32 x i16> "reg"="zv" %a1) "enc"="evexr(d, a1, 2, 0, a0, 2, 1, 0, 0, 0) | 00001011 | 11 reg:3=d&7 rm:3=a1&7" "asm"="vpmulhrsw {a1}, {a0}, {d}" "dclass"="zv" "feature"="avx512bw" "nodecode" {
  %r = call <32 x i16> @llvm.x86.avx512.pmul.hr.sw.512(<32 x i16> %a0, <32 x i16> %a1)
  ret <32 x i16> %r
}

declare <32 x i16> @llvm.x86.avx512.pmul.hr.sw.512(<32 x i16>, <32 x i16>)

define <16 x i8> @X86.avx512.pmultishift.qb.128(<16 x i8> "reg"="xv" %a0, <16 x i8> "reg"="xv" %a1) "enc"="evexr(d, a1, 2, 1, a0, 0, 1, 0, 0, 0) | 10000011 | 11 reg:3=d&7 rm:3=a1&7" "asm"="vpmultishiftqb {a1}, {a0}, {d}" "dclass"="xv" "feature"="avx512vbmi,avx512vl" "nodecode" {
  %r = call <16 x i8> @llvm.x86.avx512.pmultishift.qb.128(<16 x i8> %a0, <16 x i8> %a1)
  ret <16 x i8> %r
}

declare <16 x i8> @llvm.x86.avx512.pmultishift.qb.128(<16 x i8>, <16 x i8>)

define <32 x i8> @X86.avx512.pmultishift.qb.256(<32 x i8> "reg"="yv" %a0, <32 x i8> "reg"="yv" %a1) "enc"="evexr(d, a1, 2, 1, a0, 1, 1, 0, 0, 0) | 10000011 | 11 reg:3=d&7 rm:3=a1&7" "asm"="vpmultishiftqb {a1}, {a0}, {d}" "dclass"="yv" "feature"="avx512vbmi,avx512vl" "nodecode" {
  %r = call <32 x i8> @llvm.x86.avx512.pmultishift.qb.256(<32 x i8> %a0, <32 x i8> %a1)
  ret <32 x i8> %r
}

declare <32 x i8> @llvm.x86.avx512.pmultishift.qb.256(<32 x i8>, <32 x i8>)

define <64 x i8> @X86.avx512.pmultishift.qb.512(<64 x i8> "reg"="zv" %a0, <64 x i8> "reg"="zv" %a1) "enc"="evexr(d, a1, 2, 1, a0, 2, 1, 0, 0, 0) | 10000011 | 11 reg:3=d&7 rm:3=a1&7" "asm"="vpmultishiftqb {a1}, {a0}, {d}" "dclass"="zv" "feature"="avx512vbmi" "nodecode" {
  %r = call <64 x i8> @llvm.x86.avx512.pmultishift.qb.512(<64 x i8> %a0, <64 x i8> %a1)
  ret <64 x i8> %r
}

declare <64 x i8> @llvm.x86.avx512.pmultishift.qb.512(<64 x i8>, <64 x i8>)

define <8 x i64> @X86.avx512.psad.bw.512(<64 x i8> "reg"="zv" %a0, <64 x i8> "reg"="zv" %a1) "enc"="evexr(d, a1, 1, 0, a0, 2, 1, 0, 0, 0) | 11110110 | 11 reg:3=d&7 rm:3=a1&7" "asm"="vpsadbw {a1}, {a0}, {d}" "dclass"="zv" "feature"="avx512bw" "nodecode" {
  %r = call <8 x i64> @llvm.x86.avx512.psad.bw.512(<64 x i8> %a0, <64 x i8> %a1)
  ret <8 x i64> %r
}

declare <8 x i64> @llvm.x86.avx512.psad.bw.512(<64 x i8>, <64 x i8>)

define <64 x i8> @X86.avx512.pshuf.b.512(<64 x i8> "reg"="zv" %a0, <64 x i8> "reg"="zv" %a1) "enc"="evexr(d, a1, 2, 0, a0, 2, 1, 0, 0, 0) | 00000000 | 11 reg:3=d&7 rm:3=a1&7" "asm"="vpshufb {a1}, {a0}, {d}" "dclass"="zv" "feature"="avx512bw" "nodecode" {
  %r = call <64 x i8> @llvm.x86.avx512.pshuf.b.512(<64 x i8> %a0, <64 x i8> %a1)
  ret <64 x i8> %r
}

declare <64 x i8> @llvm.x86.avx512.pshuf.b.512(<64 x i8>, <64 x i8>)

define <16 x i32> @X86.avx512.psll.d.512(<16 x i32> "reg"="zv" %a0, <4 x i32> "reg"="xv" %a1) "enc"="evexr(d, a1, 1, 0, a0, 2, 1, 0, 0, 0) | 11110010 | 11 reg:3=d&7 rm:3=a1&7" "asm"="vpslld {a1}, {a0}, {d}" "dclass"="zv" "feature"="avx512f" "nodecode" {
  %r = call <16 x i32> @llvm.x86.avx512.psll.d.512(<16 x i32> %a0, <4 x i32> %a1)
  ret <16 x i32> %r
}

declare <16 x i32> @llvm.x86.avx512.psll.d.512(<16 x i32>, <4 x i32>)

define <8 x i64> @X86.avx512.psll.q.512(<8 x i64> "reg"="zv" %a0, <2 x i64> "reg"="xv" %a1) "enc"="evexr(d, a1, 1, 1, a0, 2, 1, 0, 0, 0) | 11110011 | 11 reg:3=d&7 rm:3=a1&7" "asm"="vpsllq {a1}, {a0}, {d}" "dclass"="zv" "feature"="avx512f" "nodecode" {
  %r = call <8 x i64> @llvm.x86.avx512.psll.q.512(<8 x i64> %a0, <2 x i64> %a1)
  ret <8 x i64> %r
}

declare <8 x i64> @llvm.x86.avx512.psll.q.512(<8 x i64>, <2 x i64>)

define <32 x i16> @X86.avx512.psll.w.512(<32 x i16> "reg"="zv" %a0, <8 x i16> "reg"="xv" %a1) "enc"="evexr(d, a1, 1, 0, a0, 2, 1, 0, 0, 0) | 11110001 | 11 reg:3=d&7 rm:3=a1&7" "asm"="vpsllw {a1}, {a0}, {d}" "dclass"="zv" "feature"="avx512bw" "nodecode" {
  %r = call <32 x i16> @llvm.x86.avx512.psll.w.512(<32 x i16> %a0, <8 x i16> %a1)
  ret <32 x i16> %r
}

declare <32 x i16> @llvm.x86.avx512.psll.w.512(<32 x i16>, <8 x i16>)

define <16 x i32> @X86.avx512.psra.d.512(<16 x i32> "reg"="zv" %a0, <4 x i32> "reg"="xv" %a1) "enc"="evexr(d, a1, 1, 0, a0, 2, 1, 0, 0, 0) | 11100010 | 11 reg:3=d&7 rm:3=a1&7" "asm"="vpsrad {a1}, {a0}, {d}" "dclass"="zv" "feature"="avx512f" "nodecode" {
  %r = call <16 x i32> @llvm.x86.avx512.psra.d.512(<16 x i32> %a0, <4 x i32> %a1)
  ret <16 x i32> %r
}

declare <16 x i32> @llvm.x86.avx512.psra.d.512(<16 x i32>, <4 x i32>)

define <2 x i64> @X86.avx512.psra.q.128(<2 x i64> "reg"="xv" %a0, <2 x i64> "reg"="xv" %a1) "enc"="evexr(d, a1, 1, 1, a0, 0, 1, 0, 0, 0) | 11100010 | 11 reg:3=d&7 rm:3=a1&7" "asm"="vpsraq {a1}, {a0}, {d}" "dclass"="xv" "feature"="avx512f,avx512vl" "nodecode" {
  %r = call <2 x i64> @llvm.x86.avx512.psra.q.128(<2 x i64> %a0, <2 x i64> %a1)
  ret <2 x i64> %r
}

declare <2 x i64> @llvm.x86.avx512.psra.q.128(<2 x i64>, <2 x i64>)

define <4 x i64> @X86.avx512.psra.q.256(<4 x i64> "reg"="yv" %a0, <2 x i64> "reg"="xv" %a1) "enc"="evexr(d, a1, 1, 1, a0, 1, 1, 0, 0, 0) | 11100010 | 11 reg:3=d&7 rm:3=a1&7" "asm"="vpsraq {a1}, {a0}, {d}" "dclass"="yv" "feature"="avx512f,avx512vl" "nodecode" {
  %r = call <4 x i64> @llvm.x86.avx512.psra.q.256(<4 x i64> %a0, <2 x i64> %a1)
  ret <4 x i64> %r
}

declare <4 x i64> @llvm.x86.avx512.psra.q.256(<4 x i64>, <2 x i64>)

define <8 x i64> @X86.avx512.psra.q.512(<8 x i64> "reg"="zv" %a0, <2 x i64> "reg"="xv" %a1) "enc"="evexr(d, a1, 1, 1, a0, 2, 1, 0, 0, 0) | 11100010 | 11 reg:3=d&7 rm:3=a1&7" "asm"="vpsraq {a1}, {a0}, {d}" "dclass"="zv" "feature"="avx512f" "nodecode" {
  %r = call <8 x i64> @llvm.x86.avx512.psra.q.512(<8 x i64> %a0, <2 x i64> %a1)
  ret <8 x i64> %r
}

declare <8 x i64> @llvm.x86.avx512.psra.q.512(<8 x i64>, <2 x i64>)

define <32 x i16> @X86.avx512.psra.w.512(<32 x i16> "reg"="zv" %a0, <8 x i16> "reg"="xv" %a1) "enc"="evexr(d, a1, 1, 0, a0, 2, 1, 0, 0, 0) | 11100001 | 11 reg:3=d&7 rm:3=a1&7" "asm"="vpsraw {a1}, {a0}, {d}" "dclass"="zv" "feature"="avx512bw" "nodecode" {
  %r = call <32 x i16> @llvm.x86.avx512.psra.w.512(<32 x i16> %a0, <8 x i16> %a1)
  ret <32 x i16> %r
}

declare <32 x i16> @llvm.x86.avx512.psra.w.512(<32 x i16>, <8 x i16>)

define <16 x i32> @X86.avx512.psrl.d.512(<16 x i32> "reg"="zv" %a0, <4 x i32> "reg"="xv" %a1) "enc"="evexr(d, a1, 1, 0, a0, 2, 1, 0, 0, 0) | 11010010 | 11 reg:3=d&7 rm:3=a1&7" "asm"="vpsrld {a1}, {a0}, {d}" "dclass"="zv" "feature"="avx512f" "nodecode" {
  %r = call <16 x i32> @llvm.x86.avx512.psrl.d.512(<16 x i32> %a0, <4 x i32> %a1)
  ret <16 x i32> %r
}

declare <16 x i32> @llvm.x86.avx512.psrl.d.512(<16 x i32>, <4 x i32>)

define <8 x i64> @X86.avx512.psrl.q.512(<8 x i64> "reg"="zv" %a0, <2 x i64> "reg"="xv" %a1) "enc"="evexr(d, a1, 1, 1, a0, 2, 1, 0, 0, 0) | 11010011 | 11 reg:3=d&7 rm:3=a1&7" "asm"="vpsrlq {a1}, {a0}, {d}" "dclass"="zv" "feature"="avx512f" "nodecode" {
  %r = call <8 x i64> @llvm.x86.avx512.psrl.q.512(<8 x i64> %a0, <2 x i64> %a1)
  ret <8 x i64> %r
}

declare <8 x i64> @llvm.x86.avx512.psrl.q.512(<8 x i64>, <2 x i64>)

define <32 x i16> @X86.avx512.psrl.w.512(<32 x i16> "reg"="zv" %a0, <8 x i16> "reg"="xv" %a1) "enc"="evexr(d, a1, 1, 0, a0, 2, 1, 0, 0, 0) | 11010001 | 11 reg:3=d&7 rm:3=a1&7" "asm"="vpsrlw {a1}, {a0}, {d}" "dclass"="zv" "feature"="avx512bw" "nodecode" {
  %r = call <32 x i16> @llvm.x86.avx512.psrl.w.512(<32 x i16> %a0, <8 x i16> %a1)
  ret <32 x i16> %r
}

declare <32 x i16> @llvm.x86.avx512.psrl.w.512(<32 x i16>, <8 x i16>)

define <4 x i32> @X86.avx512.pternlog.d.128(<4 x i32> "reg"="xv" "tied" %a0, <4 x i32> "reg"="xv" %a1, <4 x i32> "reg"="xv" %a2, i32 "imm" %a3) "enc"="evexr(d, a2, 3, 0, a1, 0, 1, 0, 0, 0) | 00100101 | 11 reg:3=d&7 rm:3=a2&7 | { i1:8=a3 ; i1:s8=a3 }" "asm"="vpternlogd ${a3}, {a2}, {a1}, {d}" "dclass"="xv" "feature"="avx512f,avx512vl" "nodecode" {
  %r = call <4 x i32> @llvm.x86.avx512.pternlog.d.128(<4 x i32> %a0, <4 x i32> %a1, <4 x i32> %a2, i32 %a3)
  ret <4 x i32> %r
}

declare <4 x i32> @llvm.x86.avx512.pternlog.d.128(<4 x i32>, <4 x i32>, <4 x i32>, i32)

define <8 x i32> @X86.avx512.pternlog.d.256(<8 x i32> "reg"="yv" "tied" %a0, <8 x i32> "reg"="yv" %a1, <8 x i32> "reg"="yv" %a2, i32 "imm" %a3) "enc"="evexr(d, a2, 3, 0, a1, 1, 1, 0, 0, 0) | 00100101 | 11 reg:3=d&7 rm:3=a2&7 | { i1:8=a3 ; i1:s8=a3 }" "asm"="vpternlogd ${a3}, {a2}, {a1}, {d}" "dclass"="yv" "feature"="avx512f,avx512vl" "nodecode" {
  %r = call <8 x i32> @llvm.x86.avx512.pternlog.d.256(<8 x i32> %a0, <8 x i32> %a1, <8 x i32> %a2, i32 %a3)
  ret <8 x i32> %r
}

declare <8 x i32> @llvm.x86.avx512.pternlog.d.256(<8 x i32>, <8 x i32>, <8 x i32>, i32)

define <16 x i32> @X86.avx512.pternlog.d.512(<16 x i32> "reg"="zv" "tied" %a0, <16 x i32> "reg"="zv" %a1, <16 x i32> "reg"="zv" %a2, i32 "imm" %a3) "enc"="evexr(d, a2, 3, 0, a1, 2, 1, 0, 0, 0) | 00100101 | 11 reg:3=d&7 rm:3=a2&7 | { i1:8=a3 ; i1:s8=a3 }" "asm"="vpternlogd ${a3}, {a2}, {a1}, {d}" "dclass"="zv" "feature"="avx512f" "nodecode" {
  %r = call <16 x i32> @llvm.x86.avx512.pternlog.d.512(<16 x i32> %a0, <16 x i32> %a1, <16 x i32> %a2, i32 %a3)
  ret <16 x i32> %r
}

declare <16 x i32> @llvm.x86.avx512.pternlog.d.512(<16 x i32>, <16 x i32>, <16 x i32>, i32)

define <2 x i64> @X86.avx512.pternlog.q.128(<2 x i64> "reg"="xv" "tied" %a0, <2 x i64> "reg"="xv" %a1, <2 x i64> "reg"="xv" %a2, i32 "imm" %a3) "enc"="evexr(d, a2, 3, 1, a1, 0, 1, 0, 0, 0) | 00100101 | 11 reg:3=d&7 rm:3=a2&7 | { i1:8=a3 ; i1:s8=a3 }" "asm"="vpternlogq ${a3}, {a2}, {a1}, {d}" "dclass"="xv" "feature"="avx512f,avx512vl" "nodecode" {
  %r = call <2 x i64> @llvm.x86.avx512.pternlog.q.128(<2 x i64> %a0, <2 x i64> %a1, <2 x i64> %a2, i32 %a3)
  ret <2 x i64> %r
}

declare <2 x i64> @llvm.x86.avx512.pternlog.q.128(<2 x i64>, <2 x i64>, <2 x i64>, i32)

define <4 x i64> @X86.avx512.pternlog.q.256(<4 x i64> "reg"="yv" "tied" %a0, <4 x i64> "reg"="yv" %a1, <4 x i64> "reg"="yv" %a2, i32 "imm" %a3) "enc"="evexr(d, a2, 3, 1, a1, 1, 1, 0, 0, 0) | 00100101 | 11 reg:3=d&7 rm:3=a2&7 | { i1:8=a3 ; i1:s8=a3 }" "asm"="vpternlogq ${a3}, {a2}, {a1}, {d}" "dclass"="yv" "feature"="avx512f,avx512vl" "nodecode" {
  %r = call <4 x i64> @llvm.x86.avx512.pternlog.q.256(<4 x i64> %a0, <4 x i64> %a1, <4 x i64> %a2, i32 %a3)
  ret <4 x i64> %r
}

declare <4 x i64> @llvm.x86.avx512.pternlog.q.256(<4 x i64>, <4 x i64>, <4 x i64>, i32)

define <8 x i64> @X86.avx512.pternlog.q.512(<8 x i64> "reg"="zv" "tied" %a0, <8 x i64> "reg"="zv" %a1, <8 x i64> "reg"="zv" %a2, i32 "imm" %a3) "enc"="evexr(d, a2, 3, 1, a1, 2, 1, 0, 0, 0) | 00100101 | 11 reg:3=d&7 rm:3=a2&7 | { i1:8=a3 ; i1:s8=a3 }" "asm"="vpternlogq ${a3}, {a2}, {a1}, {d}" "dclass"="zv" "feature"="avx512f" "nodecode" {
  %r = call <8 x i64> @llvm.x86.avx512.pternlog.q.512(<8 x i64> %a0, <8 x i64> %a1, <8 x i64> %a2, i32 %a3)
  ret <8 x i64> %r
}

declare <8 x i64> @llvm.x86.avx512.pternlog.q.512(<8 x i64>, <8 x i64>, <8 x i64>, i32)

define <2 x double> @X86.avx512.rcp14.pd.128(<2 x double> "reg"="xv" %a0, <2 x double> "reg"="xv" "tied" %a1, i8 "reg"="k" %a2) "enc"="{ if(a2!=0) evexr(d, a0, 2, 1, 0, 0, 1, 0, a2, 0) | 01001100 | 11 reg:3=d&7 rm:3=a0&7 }" "asm"="vrcp14pd {a0}, {d} {lb}{a2}{rb}" "dclass"="xv" "feature"="avx512f,avx512vl" "nodecode" {
  %r = call <2 x double> @llvm.x86.avx512.rcp14.pd.128(<2 x double> %a0, <2 x double> %a1, i8 %a2)
  ret <2 x double> %r
}

declare <2 x double> @llvm.x86.avx512.rcp14.pd.128(<2 x double>, <2 x double>, i8)

define <4 x double> @X86.avx512.rcp14.pd.256(<4 x double> "reg"="yv" %a0, <4 x double> "reg"="yv" "tied" %a1, i8 "reg"="k" %a2) "enc"="{ if(a2!=0) evexr(d, a0, 2, 1, 0, 1, 1, 0, a2, 0) | 01001100 | 11 reg:3=d&7 rm:3=a0&7 }" "asm"="vrcp14pd {a0}, {d} {lb}{a2}{rb}" "dclass"="yv" "feature"="avx512f,avx512vl" "nodecode" {
  %r = call <4 x double> @llvm.x86.avx512.rcp14.pd.256(<4 x double> %a0, <4 x double> %a1, i8 %a2)
  ret <4 x double> %r
}

declare <4 x double> @llvm.x86.avx512.rcp14.pd.256(<4 x double>, <4 x double>, i8)

define <8 x double> @X86.avx512.rcp14.pd.512(<8 x double> "reg"="zv" %a0, <8 x double> "reg"="zv" "tied" %a1, i8 "reg"="k" %a2) "enc"="{ if(a2!=0) evexr(d, a0, 2, 1, 0, 2, 1, 0, a2, 0) | 01001100 | 11 reg:3=d&7 rm:3=a0&7 }" "asm"="vrcp14pd {a0}, {d} {lb}{a2}{rb}" "dclass"="zv" "feature"="avx512f" "nodecode" {
  %r = call <8 x double> @llvm.x86.avx512.rcp14.pd.512(<8 x double> %a0, <8 x double> %a1, i8 %a2)
  ret <8 x double> %r
}

declare <8 x double> @llvm.x86.avx512.rcp14.pd.512(<8 x double>, <8 x double>, i8)

define <4 x float> @X86.avx512.rcp14.ps.128(<4 x float> "reg"="xv" %a0, <4 x float> "reg"="xv" "tied" %a1, i8 "reg"="k" %a2) "enc"="{ if(a2!=0) evexr(d, a0, 2, 0, 0, 0, 1, 0, a2, 0) | 01001100 | 11 reg:3=d&7 rm:3=a0&7 }" "asm"="vrcp14ps {a0}, {d} {lb}{a2}{rb}" "dclass"="xv" "feature"="avx512f,avx512vl" "nodecode" {
  %r = call <4 x float> @llvm.x86.avx512.rcp14.ps.128(<4 x float> %a0, <4 x float> %a1, i8 %a2)
  ret <4 x float> %r
}

declare <4 x float> @llvm.x86.avx512.rcp14.ps.128(<4 x float>, <4 x float>, i8)

define <8 x float> @X86.avx512.rcp14.ps.256(<8 x float> "reg"="yv" %a0, <8 x float> "reg"="yv" "tied" %a1, i8 "reg"="k" %a2) "enc"="{ if(a2!=0) evexr(d, a0, 2, 0, 0, 1, 1, 0, a2, 0) | 01001100 | 11 reg:3=d&7 rm:3=a0&7 }" "asm"="vrcp14ps {a0}, {d} {lb}{a2}{rb}" "dclass"="yv" "feature"="avx512f,avx512vl" "nodecode" {
  %r = call <8 x float> @llvm.x86.avx512.rcp14.ps.256(<8 x float> %a0, <8 x float> %a1, i8 %a2)
  ret <8 x float> %r
}

declare <8 x float> @llvm.x86.avx512.rcp14.ps.256(<8 x float>, <8 x float>, i8)

define <16 x float> @X86.avx512.rcp14.ps.512(<16 x float> "reg"="zv" %a0, <16 x float> "reg"="zv" "tied" %a1, i16 "reg"="k" %a2) "enc"="{ if(a2!=0) evexr(d, a0, 2, 0, 0, 2, 1, 0, a2, 0) | 01001100 | 11 reg:3=d&7 rm:3=a0&7 }" "asm"="vrcp14ps {a0}, {d} {lb}{a2}{rb}" "dclass"="zv" "feature"="avx512f" "nodecode" {
  %r = call <16 x float> @llvm.x86.avx512.rcp14.ps.512(<16 x float> %a0, <16 x float> %a1, i16 %a2)
  ret <16 x float> %r
}

declare <16 x float> @llvm.x86.avx512.rcp14.ps.512(<16 x float>, <16 x float>, i16)

define <2 x double> @X86.avx512.rcp14.sd(<2 x double> "reg"="xv" %a0, <2 x double> "reg"="xv" %a1, <2 x double> "reg"="xv" "tied" %a2, i8 "reg"="k" %a3) "enc"="{ if(a3!=0) evexr(d, a1, 2, 1, a0, 0, 1, 0, a3, 0) | 01001101 | 11 reg:3=d&7 rm:3=a1&7 }" "asm"="vrcp14sd {a1}, {a0}, {d} {lb}{a3}{rb}" "dclass"="xv" "feature"="avx512f" "nodecode" {
  %r = call <2 x double> @llvm.x86.avx512.rcp14.sd(<2 x double> %a0, <2 x double> %a1, <2 x double> %a2, i8 %a3)
  ret <2 x double> %r
}

declare <2 x double> @llvm.x86.avx512.rcp14.sd(<2 x double>, <2 x double>, <2 x double>, i8)

define <4 x float> @X86.avx512.rcp14.ss(<4 x float> "reg"="xv" %a0, <4 x float> "reg"="xv" %a1, <4 x float> "reg"="xv" "tied" %a2, i8 "reg"="k" %a3) "enc"="{ if(a3!=0) evexr(d, a1, 2, 0, a0, 0, 1, 0, a3, 0) | 01001101 | 11 reg:3=d&7 rm:3=a1&7 }" "asm"="vrcp14ss {a1}, {a0}, {d} {lb}{a3}{rb}" "dclass"="xv" "feature"="avx512f" "nodecode" {
  %r = call <4 x float> @llvm.x86.avx512.rcp14.ss(<4 x float> %a0, <4 x float> %a1, <4 x float> %a2, i8 %a3)
  ret <4 x float> %r
}

declare <4 x float> @llvm.x86.avx512.rcp14.ss(<4 x float>, <4 x float>, <4 x float>, i8)

define <2 x double> @X86.avx512.rsqrt14.pd.128(<2 x double> "reg"="xv" %a0, <2 x double> "reg"="xv" "tied" %a1, i8 "reg"="k" %a2) "enc"="{ if(a2!=0) evexr(d, a0, 2, 1, 0, 0, 1, 0, a2, 0) | 01001110 | 11 reg:3=d&7 rm:3=a0&7 }" "asm"="vrsqrt14pd {a0}, {d} {lb}{a2}{rb}" "dclass"="xv" "feature"="avx512f,avx512vl" "nodecode" {
  %r = call <2 x double> @llvm.x86.avx512.rsqrt14.pd.128(<2 x double> %a0, <2 x double> %a1, i8 %a2)
  ret <2 x double> %r
}

declare <2 x double> @llvm.x86.avx512.rsqrt14.pd.128(<2 x double>, <2 x double>, i8)

define <4 x double> @X86.avx512.rsqrt14.pd.256(<4 x double> "reg"="yv" %a0, <4 x double> "reg"="yv" "tied" %a1, i8 "reg"="k" %a2) "enc"="{ if(a2!=0) evexr(d, a0, 2, 1, 0, 1, 1, 0, a2, 0) | 01001110 | 11 reg:3=d&7 rm:3=a0&7 }" "asm"="vrsqrt14pd {a0}, {d} {lb}{a2}{rb}" "dclass"="yv" "feature"="avx512f,avx512vl" "nodecode" {
  %r = call <4 x double> @llvm.x86.avx512.rsqrt14.pd.256(<4 x double> %a0, <4 x double> %a1, i8 %a2)
  ret <4 x double> %r
}

declare <4 x double> @llvm.x86.avx512.rsqrt14.pd.256(<4 x double>, <4 x double>, i8)

define <8 x double> @X86.avx512.rsqrt14.pd.512(<8 x double> "reg"="zv" %a0, <8 x double> "reg"="zv" "tied" %a1, i8 "reg"="k" %a2) "enc"="{ if(a2!=0) evexr(d, a0, 2, 1, 0, 2, 1, 0, a2, 0) | 01001110 | 11 reg:3=d&7 rm:3=a0&7 }" "asm"="vrsqrt14pd {a0}, {d} {lb}{a2}{rb}" "dclass"="zv" "feature"="avx512f" "nodecode" {
  %r = call <8 x double> @llvm.x86.avx512.rsqrt14.pd.512(<8 x double> %a0, <8 x double> %a1, i8 %a2)
  ret <8 x double> %r
}

declare <8 x double> @llvm.x86.avx512.rsqrt14.pd.512(<8 x double>, <8 x double>, i8)

define <4 x float> @X86.avx512.rsqrt14.ps.128(<4 x float> "reg"="xv" %a0, <4 x float> "reg"="xv" "tied" %a1, i8 "reg"="k" %a2) "enc"="{ if(a2!=0) evexr(d, a0, 2, 0, 0, 0, 1, 0, a2, 0) | 01001110 | 11 reg:3=d&7 rm:3=a0&7 }" "asm"="vrsqrt14ps {a0}, {d} {lb}{a2}{rb}" "dclass"="xv" "feature"="avx512f,avx512vl" "nodecode" {
  %r = call <4 x float> @llvm.x86.avx512.rsqrt14.ps.128(<4 x float> %a0, <4 x float> %a1, i8 %a2)
  ret <4 x float> %r
}

declare <4 x float> @llvm.x86.avx512.rsqrt14.ps.128(<4 x float>, <4 x float>, i8)

define <8 x float> @X86.avx512.rsqrt14.ps.256(<8 x float> "reg"="yv" %a0, <8 x float> "reg"="yv" "tied" %a1, i8 "reg"="k" %a2) "enc"="{ if(a2!=0) evexr(d, a0, 2, 0, 0, 1, 1, 0, a2, 0) | 01001110 | 11 reg:3=d&7 rm:3=a0&7 }" "asm"="vrsqrt14ps {a0}, {d} {lb}{a2}{rb}" "dclass"="yv" "feature"="avx512f,avx512vl" "nodecode" {
  %r = call <8 x float> @llvm.x86.avx512.rsqrt14.ps.256(<8 x float> %a0, <8 x float> %a1, i8 %a2)
  ret <8 x float> %r
}

declare <8 x float> @llvm.x86.avx512.rsqrt14.ps.256(<8 x float>, <8 x float>, i8)

define <16 x float> @X86.avx512.rsqrt14.ps.512(<16 x float> "reg"="zv" %a0, <16 x float> "reg"="zv" "tied" %a1, i16 "reg"="k" %a2) "enc"="{ if(a2!=0) evexr(d, a0, 2, 0, 0, 2, 1, 0, a2, 0) | 01001110 | 11 reg:3=d&7 rm:3=a0&7 }" "asm"="vrsqrt14ps {a0}, {d} {lb}{a2}{rb}" "dclass"="zv" "feature"="avx512f" "nodecode" {
  %r = call <16 x float> @llvm.x86.avx512.rsqrt14.ps.512(<16 x float> %a0, <16 x float> %a1, i16 %a2)
  ret <16 x float> %r
}

declare <16 x float> @llvm.x86.avx512.rsqrt14.ps.512(<16 x float>, <16 x float>, i16)

define <2 x double> @X86.avx512.rsqrt14.sd(<2 x double> "reg"="xv" %a0, <2 x double> "reg"="xv" %a1, <2 x double> "reg"="xv" "tied" %a2, i8 "reg"="k" %a3) "enc"="{ if(a3!=0) evexr(d, a1, 2, 1, a0, 0, 1, 0, a3, 0) | 01001111 | 11 reg:3=d&7 rm:3=a1&7 }" "asm"="vrsqrt14sd {a1}, {a0}, {d} {lb}{a3}{rb}" "dclass"="xv" "feature"="avx512f" "nodecode" {
  %r = call <2 x double> @llvm.x86.avx512.rsqrt14.sd(<2 x double> %a0, <2 x double> %a1, <2 x double> %a2, i8 %a3)
  ret <2 x double> %r
}

declare <2 x double> @llvm.x86.avx512.rsqrt14.sd(<2 x double>, <2 x double>, <2 x double>, i8)

define <4 x float> @X86.avx512.rsqrt14.ss(<4 x float> "reg"="xv" %a0, <4 x float> "reg"="xv" %a1, <4 x float> "reg"="xv" "tied" %a2, i8 "reg"="k" %a3) "enc"="{ if(a3!=0) evexr(d, a1, 2, 0, a0, 0, 1, 0, a3, 0) | 01001111 | 11 reg:3=d&7 rm:3=a1&7 }" "asm"="vrsqrt14ss {a1}, {a0}, {d} {lb}{a3}{rb}" "dclass"="xv" "feature"="avx512f" "nodecode" {
  %r = call <4 x float> @llvm.x86.avx512.rsqrt14.ss(<4 x float> %a0, <4 x float> %a1, <4 x float> %a2, i8 %a3)
  ret <4 x float> %r
}

declare <4 x float> @llvm.x86.avx512.rsqrt14.ss(<4 x float>, <4 x float>, <4 x float>, i8)

define <16 x float> @X86.avx512.sitofp.round.v16f32.v16i32(<16 x i32> "reg"="zv" %a0) "enc"="evexr(d, a0, 1, 0, 0, 2, 0, 0, 0, 0) | 01011011 | 11 reg:3=d&7 rm:3=a0&7" "asm"="vcvtdq2ps {a0}, {d}" "dclass"="zv" "feature"="avx512f" "nodecode" {
  %r = call <16 x float> @llvm.x86.avx512.sitofp.round.v16f32.v16i32(<16 x i32> %a0, i32 4)
  ret <16 x float> %r
}

declare <16 x float> @llvm.x86.avx512.sitofp.round.v16f32.v16i32(<16 x i32>, i32)

define <2 x double> @X86.avx512.sitofp.round.v2f64.v2i64(<2 x i64> "reg"="xv" %a0) "enc"="evexr(d, a0, 1, 1, 0, 0, 2, 0, 0, 0) | 11100110 | 11 reg:3=d&7 rm:3=a0&7" "asm"="vcvtqq2pd {a0}, {d}" "dclass"="xv" "feature"="avx512dq,avx512vl" "nodecode" {
  %r = call <2 x double> @llvm.x86.avx512.sitofp.round.v2f64.v2i64(<2 x i64> %a0, i32 4)
  ret <2 x double> %r
}

declare <2 x double> @llvm.x86.avx512.sitofp.round.v2f64.v2i64(<2 x i64>, i32)

define <4 x float> @X86.avx512.sitofp.round.v4f32.v4i64(<4 x i64> "reg"="yv" %a0) "enc"="evexr(d, a0, 1, 1, 0, 1, 0, 0, 0, 0) | 01011011 | 11 reg:3=d&7 rm:3=a0&7" "asm"="vcvtqq2ps {a0}, {d}" "dclass"="xv" "feature"="avx512dq,avx512vl" "nodecode" {
  %r = call <4 x float> @llvm.x86.avx512.sitofp.round.v4f32.v4i64(<4 x i64> %a0, i32 4)
  ret <4 x float> %r
}

declare <4 x float> @llvm.x86.avx512.sitofp.round.v4f32.v4i64(<4 x i64>, i32)

define <4 x double> @X86.avx512.sitofp.round.v4f64.v4i64(<4 x i64> "reg"="yv" %a0) "enc"="evexr(d, a0, 1, 1, 0, 1, 2, 0, 0, 0) | 11100110 | 11 reg:3=d&7 rm:3=a0&7" "asm"="vcvtqq2pd {a0}, {d}" "dclass"="yv" "feature"="avx512dq,avx512vl" "nodecode" {
  %r = call <4 x double> @llvm.x86.avx512.sitofp.round.v4f64.v4i64(<4 x i64> %a0, i32 4)
  ret <4 x double> %r
}

declare <4 x double> @llvm.x86.avx512.sitofp.round.v4f64.v4i64(<4 x i64>, i32)

define <8 x float> @X86.avx512.sitofp.round.v8f32.v8i64(<8 x i64> "reg"="zv" %a0) "enc"="evexr(d, a0, 1, 1, 0, 2, 0, 0, 0, 0) | 01011011 | 11 reg:3=d&7 rm:3=a0&7" "asm"="vcvtqq2ps {a0}, {d}" "dclass"="yv" "feature"="avx512dq" "nodecode" {
  %r = call <8 x float> @llvm.x86.avx512.sitofp.round.v8f32.v8i64(<8 x i64> %a0, i32 4)
  ret <8 x float> %r
}

declare <8 x float> @llvm.x86.avx512.sitofp.round.v8f32.v8i64(<8 x i64>, i32)

define <8 x double> @X86.avx512.sitofp.round.v8f64.v8i64(<8 x i64> "reg"="zv" %a0) "enc"="evexr(d, a0, 1, 1, 0, 2, 2, 0, 0, 0) | 11100110 | 11 reg:3=d&7 rm:3=a0&7" "asm"="vcvtqq2pd {a0}, {d}" "dclass"="zv" "feature"="avx512dq" "nodecode" {
  %r = call <8 x double> @llvm.x86.avx512.sitofp.round.v8f64.v8i64(<8 x i64> %a0, i32 4)
  ret <8 x double> %r
}

declare <8 x double> @llvm.x86.avx512.sitofp.round.v8f64.v8i64(<8 x i64>, i32)

define <8 x double> @X86.avx512.sqrt.pd.512(<8 x double> "reg"="zv" %a0) "enc"="evexr(d, a0, 1, 1, 0, 2, 1, 0, 0, 0) | 01010001 | 11 reg:3=d&7 rm:3=a0&7" "asm"="vsqrtpd {a0}, {d}" "dclass"="zv" "feature"="avx512f" "nodecode" {
  %r = call <8 x double> @llvm.x86.avx512.sqrt.pd.512(<8 x double> %a0, i32 4)
  ret <8 x double> %r
}

declare <8 x double> @llvm.x86.avx512.sqrt.pd.512(<8 x double>, i32)

define <16 x float> @X86.avx512.sqrt.ps.512(<16 x float> "reg"="zv" %a0) "enc"="evexr(d, a0, 1, 0, 0, 2, 0, 0, 0, 0) | 01010001 | 11 reg:3=d&7 rm:3=a0&7" "asm"="vsqrtps {a0}, {d}" "dclass"="zv" "feature"="avx512f" "nodecode" {
  %r = call <16 x float> @llvm.x86.avx512.sqrt.ps.512(<16 x float> %a0, i32 4)
  ret <16 x float> %r
}

declare <16 x float> @llvm.x86.avx512.sqrt.ps.512(<16 x float>, i32)

define <8 x double> @X86.avx512.sub.pd.512(<8 x double> "reg"="zv" %a0, <8 x double> "reg"="zv" %a1) "enc"="evexr(d, a1, 1, 1, a0, 2, 1, 0, 0, 0) | 01011100 | 11 reg:3=d&7 rm:3=a1&7" "asm"="vsubpd {a1}, {a0}, {d}" "dclass"="zv" "feature"="avx512f" "nodecode" {
  %r = call <8 x double> @llvm.x86.avx512.sub.pd.512(<8 x double> %a0, <8 x double> %a1, i32 4)
  ret <8 x double> %r
}

declare <8 x double> @llvm.x86.avx512.sub.pd.512(<8 x double>, <8 x double>, i32)

define <16 x float> @X86.avx512.sub.ps.512(<16 x float> "reg"="zv" %a0, <16 x float> "reg"="zv" %a1) "enc"="evexr(d, a1, 1, 0, a0, 2, 0, 0, 0, 0) | 01011100 | 11 reg:3=d&7 rm:3=a1&7" "asm"="vsubps {a1}, {a0}, {d}" "dclass"="zv" "feature"="avx512f" "nodecode" {
  %r = call <16 x float> @llvm.x86.avx512.sub.ps.512(<16 x float> %a0, <16 x float> %a1, i32 4)
  ret <16 x float> %r
}

declare <16 x float> @llvm.x86.avx512.sub.ps.512(<16 x float>, <16 x float>, i32)

define <16 x float> @X86.avx512.uitofp.round.v16f32.v16i32(<16 x i32> "reg"="zv" %a0) "enc"="evexr(d, a0, 1, 0, 0, 2, 3, 0, 0, 0) | 01111010 | 11 reg:3=d&7 rm:3=a0&7" "asm"="vcvtudq2ps {a0}, {d}" "dclass"="zv" "feature"="avx512f" "nodecode" {
  %r = call <16 x float> @llvm.x86.avx512.uitofp.round.v16f32.v16i32(<16 x i32> %a0, i32 4)
  ret <16 x float> %r
}

declare <16 x float> @llvm.x86.avx512.uitofp.round.v16f32.v16i32(<16 x i32>, i32)

define <2 x double> @X86.avx512.uitofp.round.v2f64.v2i64(<2 x i64> "reg"="xv" %a0) "enc"="evexr(d, a0, 1, 1, 0, 0, 2, 0, 0, 0) | 01111010 | 11 reg:3=d&7 rm:3=a0&7" "asm"="vcvtuqq2pd {a0}, {d}" "dclass"="xv" "feature"="avx512dq,avx512vl" "nodecode" {
  %r = call <2 x double> @llvm.x86.avx512.uitofp.round.v2f64.v2i64(<2 x i64> %a0, i32 4)
  ret <2 x double> %r
}

declare <2 x double> @llvm.x86.avx512.uitofp.round.v2f64.v2i64(<2 x i64>, i32)

define <4 x float> @X86.avx512.uitofp.round.v4f32.v4i64(<4 x i64> "reg"="yv" %a0) "enc"="evexr(d, a0, 1, 1, 0, 1, 3, 0, 0, 0) | 01111010 | 11 reg:3=d&7 rm:3=a0&7" "asm"="vcvtuqq2ps {a0}, {d}" "dclass"="xv" "feature"="avx512dq,avx512vl" "nodecode" {
  %r = call <4 x float> @llvm.x86.avx512.uitofp.round.v4f32.v4i64(<4 x i64> %a0, i32 4)
  ret <4 x float> %r
}

declare <4 x float> @llvm.x86.avx512.uitofp.round.v4f32.v4i64(<4 x i64>, i32)

define <4 x double> @X86.avx512.uitofp.round.v4f64.v4i64(<4 x i64> "reg"="yv" %a0) "enc"="evexr(d, a0, 1, 1, 0, 1, 2, 0, 0, 0) | 01111010 | 11 reg:3=d&7 rm:3=a0&7" "asm"="vcvtuqq2pd {a0}, {d}" "dclass"="yv" "feature"="avx512dq,avx512vl" "nodecode" {
  %r = call <4 x double> @llvm.x86.avx512.uitofp.round.v4f64.v4i64(<4 x i64> %a0, i32 4)
  ret <4 x double> %r
}

declare <4 x double> @llvm.x86.avx512.uitofp.round.v4f64.v4i64(<4 x i64>, i32)

define <8 x float> @X86.avx512.uitofp.round.v8f32.v8i64(<8 x i64> "reg"="zv" %a0) "enc"="evexr(d, a0, 1, 1, 0, 2, 3, 0, 0, 0) | 01111010 | 11 reg:3=d&7 rm:3=a0&7" "asm"="vcvtuqq2ps {a0}, {d}" "dclass"="yv" "feature"="avx512dq" "nodecode" {
  %r = call <8 x float> @llvm.x86.avx512.uitofp.round.v8f32.v8i64(<8 x i64> %a0, i32 4)
  ret <8 x float> %r
}

declare <8 x float> @llvm.x86.avx512.uitofp.round.v8f32.v8i64(<8 x i64>, i32)

define <8 x double> @X86.avx512.uitofp.round.v8f64.v8i64(<8 x i64> "reg"="zv" %a0) "enc"="evexr(d, a0, 1, 1, 0, 2, 2, 0, 0, 0) | 01111010 | 11 reg:3=d&7 rm:3=a0&7" "asm"="vcvtuqq2pd {a0}, {d}" "dclass"="zv" "feature"="avx512dq" "nodecode" {
  %r = call <8 x double> @llvm.x86.avx512.uitofp.round.v8f64.v8i64(<8 x i64> %a0, i32 4)
  ret <8 x double> %r
}

declare <8 x double> @llvm.x86.avx512.uitofp.round.v8f64.v8i64(<8 x i64>, i32)

define i32 @X86.avx512.vcvtsd2si32(<2 x double> "reg"="xv" %a0) "enc"="{ vex(d, 0, a0, 1, 0, 0, 0, 3) | 00101101 | 11 reg:3=d&7 rm:3=a0&7 ; evexr(d, a0, 1, 0, 0, 0, 3, 0, 0, 0) | 00101101 | 11 reg:3=d&7 rm:3=a0&7 }" "asm"="vcvtsd2si {a0}, {d}" "dclass"="r32" "feature"="avx512f" "nodecode" {
  %r = call i32 @llvm.x86.avx512.vcvtsd2si32(<2 x double> %a0, i32 4)
  ret i32 %r
}

declare i32 @llvm.x86.avx512.vcvtsd2si32(<2 x double>, i32)

define i64 @X86.avx512.vcvtsd2si64(<2 x double> "reg"="xv" %a0) "enc"="{ vex(d, 0, a0, 1, 1, 0, 0, 3) | 00101101 | 11 reg:3=d&7 rm:3=a0&7 ; evexr(d, a0, 1, 1, 0, 0, 3, 0, 0, 0) | 00101101 | 11 reg:3=d&7 rm:3=a0&7 }" "asm"="vcvtsd2si {a0}, {d}" "dclass"="r64" "feature"="avx512f" "nodecode" {
  %r = call i64 @llvm.x86.avx512.vcvtsd2si64(<2 x double> %a0, i32 4)
  ret i64 %r
}

declare i64 @llvm.x86.avx512.vcvtsd2si64(<2 x double>, i32)

define i32 @X86.avx512.vcvtsd2usi32(<2 x double> "reg"="xv" %a0) "enc"="evexr(d, a0, 1, 0, 0, 0, 3, 0, 0, 0) | 01111001 | 11 reg:3=d&7 rm:3=a0&7" "asm"="vcvtsd2usi {a0}, {d}" "dclass"="r32" "feature"="avx512f" "nodecode" {
  %r = call i32 @llvm.x86.avx512.vcvtsd2usi32(<2 x double> %a0, i32 4)
  ret i32 %r
}

declare i32 @llvm.x86.avx512.vcvtsd2usi32(<2 x double>, i32)

define i64 @X86.avx512.vcvtsd2usi64(<2 x double> "reg"="xv" %a0) "enc"="evexr(d, a0, 1, 1, 0, 0, 3, 0, 0, 0) | 01111001 | 11 reg:3=d&7 rm:3=a0&7" "asm"="vcvtsd2usi {a0}, {d}" "dclass"="r64" "feature"="avx512f" "nodecode" {
  %r = call i64 @llvm.x86.avx512.vcvtsd2usi64(<2 x double> %a0, i32 4)
  ret i64 %r
}

declare i64 @llvm.x86.avx512.vcvtsd2usi64(<2 x double>, i32)

define i32 @X86.avx512.vcvtss2si32(<4 x float> "reg"="xv" %a0) "enc"="{ vex(d, 0, a0, 1, 0, 0, 0, 2) | 00101101 | 11 reg:3=d&7 rm:3=a0&7 ; evexr(d, a0, 1, 0, 0, 0, 2, 0, 0, 0) | 00101101 | 11 reg:3=d&7 rm:3=a0&7 }" "asm"="vcvtss2si {a0}, {d}" "dclass"="r32" "feature"="avx512f" "nodecode" {
  %r = call i32 @llvm.x86.avx512.vcvtss2si32(<4 x float> %a0, i32 4)
  ret i32 %r
}

declare i32 @llvm.x86.avx512.vcvtss2si32(<4 x float>, i32)

define i64 @X86.avx512.vcvtss2si64(<4 x float> "reg"="xv" %a0) "enc"="{ vex(d, 0, a0, 1, 1, 0, 0, 2) | 00101101 | 11 reg:3=d&7 rm:3=a0&7 ; evexr(d, a0, 1, 1, 0, 0, 2, 0, 0, 0) | 00101101 | 11 reg:3=d&7 rm:3=a0&7 }" "asm"="vcvtss2si {a0}, {d}" "dclass"="r64" "feature"="avx512f" "nodecode" {
  %r = call i64 @llvm.x86.avx512.vcvtss2si64(<4 x float> %a0, i32 4)
  ret i64 %r
}

declare i64 @llvm.x86.avx512.vcvtss2si64(<4 x float>, i32)

define i32 @X86.avx512.vcvtss2usi32(<4 x float> "reg"="xv" %a0) "enc"="evexr(d, a0, 1, 0, 0, 0, 2, 0, 0, 0) | 01111001 | 11 reg:3=d&7 rm:3=a0&7" "asm"="vcvtss2usi {a0}, {d}" "dclass"="r32" "feature"="avx512f" "nodecode" {
  %r = call i32 @llvm.x86.avx512.vcvtss2usi32(<4 x float> %a0, i32 4)
  ret i32 %r
}

declare i32 @llvm.x86.avx512.vcvtss2usi32(<4 x float>, i32)

define i64 @X86.avx512.vcvtss2usi64(<4 x float> "reg"="xv" %a0) "enc"="evexr(d, a0, 1, 1, 0, 0, 2, 0, 0, 0) | 01111001 | 11 reg:3=d&7 rm:3=a0&7" "asm"="vcvtss2usi {a0}, {d}" "dclass"="r64" "feature"="avx512f" "nodecode" {
  %r = call i64 @llvm.x86.avx512.vcvtss2usi64(<4 x float> %a0, i32 4)
  ret i64 %r
}

declare i64 @llvm.x86.avx512.vcvtss2usi64(<4 x float>, i32)

define float @X86.avx512.vfmadd.f32(float "reg"="xv" "tied" %a0, float "reg"="xv" %a1, float "reg"="xv" %a2) "enc"="{ vex(d, 0, a2, 2, 0, a1, 0, 1) | 10101001 | 11 reg:3=d&7 rm:3=a2&7 ; evexr(d, a2, 2, 0, a1, 0, 1, 0, 0, 0) | 10101001 | 11 reg:3=d&7 rm:3=a2&7 }" "asm"="vfmadd213ss {a2}, {a1}, {d}" "dclass"="xv" "feature"="avx512f" "nodecode" {
  %r = call float @llvm.x86.avx512.vfmadd.f32(float %a0, float %a1, float %a2, i32 4)
  ret float %r
}

declare float @llvm.x86.avx512.vfmadd.f32(float, float, float, i32)

define double @X86.avx512.vfmadd.f64(double "reg"="xv" "tied" %a0, double "reg"="xv" %a1, double "reg"="xv" %a2) "enc"="{ vex(d, 0, a2, 2, 1, a1, 0, 1) | 10101001 | 11 reg:3=d&7 rm:3=a2&7 ; evexr(d, a2, 2, 1, a1, 0, 1, 0, 0, 0) | 10101001 | 11 reg:3=d&7 rm:3=a2&7 }" "asm"="vfmadd213sd {a2}, {a1}, {d}" "dclass"="xv" "feature"="avx512f" "nodecode" {
  %r = call double @llvm.x86.avx512.vfmadd.f64(double %a0, double %a1, double %a2, i32 4)
  ret double %r
}

declare double @llvm.x86.avx512.vfmadd.f64(double, double, double, i32)

define <8 x double> @X86.avx512.vfmadd.pd.512(<8 x double> "reg"="zv" "tied" %a0, <8 x double> "reg"="zv" %a1, <8 x double> "reg"="zv" %a2) "enc"="evexr(d, a2, 2, 1, a1, 2, 1, 0, 0, 0) | 10101000 | 11 reg:3=d&7 rm:3=a2&7" "asm"="vfmadd213pd {a2}, {a1}, {d}" "dclass"="zv" "feature"="avx512f" "nodecode" {
  %r = call <8 x double> @llvm.x86.avx512.vfmadd.pd.512(<8 x double> %a0, <8 x double> %a1, <8 x double> %a2, i32 4)
  ret <8 x double> %r
}

declare <8 x double> @llvm.x86.avx512.vfmadd.pd.512(<8 x double>, <8 x double>, <8 x double>, i32)

define <16 x float> @X86.avx512.vfmadd.ps.512(<16 x float> "reg"="zv" "tied" %a0, <16 x float> "reg"="zv" %a1, <16 x float> "reg"="zv" %a2) "enc"="evexr(d, a2, 2, 0, a1, 2, 1, 0, 0, 0) | 10101000 | 11 reg:3=d&7 rm:3=a2&7" "asm"="vfmadd213ps {a2}, {a1}, {d}" "dclass"="zv" "feature"="avx512f" "nodecode" {
  %r = call <16 x float> @llvm.x86.avx512.vfmadd.ps.512(<16 x float> %a0, <16 x float> %a1, <16 x float> %a2, i32 4)
  ret <16 x float> %r
}

declare <16 x float> @llvm.x86.avx512.vfmadd.ps.512(<16 x float>, <16 x float>, <16 x float>, i32)

define <8 x double> @X86.avx512.vfmaddsub.pd.512(<8 x double> "reg"="zv" "tied" %a0, <8 x double> "reg"="zv" %a1, <8 x double> "reg"="zv" %a2) "enc"="evexr(d, a2, 2, 1, a1, 2, 1, 0, 0, 0) | 10100110 | 11 reg:3=d&7 rm:3=a2&7" "asm"="vfmaddsub213pd {a2}, {a1}, {d}" "dclass"="zv" "feature"="avx512f" "nodecode" {
  %r = call <8 x double> @llvm.x86.avx512.vfmaddsub.pd.512(<8 x double> %a0, <8 x double> %a1, <8 x double> %a2, i32 4)
  ret <8 x double> %r
}

declare <8 x double> @llvm.x86.avx512.vfmaddsub.pd.512(<8 x double>, <8 x double>, <8 x double>, i32)

define <16 x float> @X86.avx512.vfmaddsub.ps.512(<16 x float> "reg"="zv" "tied" %a0, <16 x float> "reg"="zv" %a1, <16 x float> "reg"="zv" %a2) "enc"="evexr(d, a2, 2, 0, a1, 2, 1, 0, 0, 0) | 10100110 | 11 reg:3=d&7 rm:3=a2&7" "asm"="vfmaddsub213ps {a2}, {a1}, {d}" "dclass"="zv" "feature"="avx512f" "nodecode" {
  %r = call <16 x float> @llvm.x86.avx512.vfmaddsub.ps.512(<16 x float> %a0, <16 x float> %a1, <16 x float> %a2, i32 4)
  ret <16 x float> %r
}

declare <16 x float> @llvm.x86.avx512.vfmaddsub.ps.512(<16 x float>, <16 x float>, <16 x float>, i32)

define <4 x i32> @X86.avx512.vpdpbusd.128(<4 x i32> "reg"="xv" "tied" %a0, <16 x i8> "reg"="xv" %a1, <16 x i8> "reg"="xv" %a2) "enc"="evexr(d, a2, 2, 0, a1, 0, 1, 0, 0, 0) | 01010000 | 11 reg:3=d&7 rm:3=a2&7" "asm"="vpdpbusd {a2}, {a1}, {d}" "dclass"="xv" "feature"="avxvnni" "nodecode" {
  %r = call <4 x i32> @llvm.x86.avx512.vpdpbusd.128(<4 x i32> %a0, <16 x i8> %a1, <16 x i8> %a2)
  ret <4 x i32> %r
}

declare <4 x i32> @llvm.x86.avx512.vpdpbusd.128(<4 x i32>, <16 x i8>, <16 x i8>)

define <8 x i32> @X86.avx512.vpdpbusd.256(<8 x i32> "reg"="yv" "tied" %a0, <32 x i8> "reg"="yv" %a1, <32 x i8> "reg"="yv" %a2) "enc"="evexr(d, a2, 2, 0, a1, 1, 1, 0, 0, 0) | 01010000 | 11 reg:3=d&7 rm:3=a2&7" "asm"="vpdpbusd {a2}, {a1}, {d}" "dclass"="yv" "feature"="avxvnni" "nodecode" {
  %r = call <8 x i32> @llvm.x86.avx512.vpdpbusd.256(<8 x i32> %a0, <32 x i8> %a1, <32 x i8> %a2)
  ret <8 x i32> %r
}

declare <8 x i32> @llvm.x86.avx512.vpdpbusd.256(<8 x i32>, <32 x i8>, <32 x i8>)

define <16 x i32> @X86.avx512.vpdpbusd.512(<16 x i32> "reg"="zv" "tied" %a0, <64 x i8> "reg"="zv" %a1, <64 x i8> "reg"="zv" %a2) "enc"="evexr(d, a2, 2, 0, a1, 2, 1, 0, 0, 0) | 01010000 | 11 reg:3=d&7 rm:3=a2&7" "asm"="vpdpbusd {a2}, {a1}, {d}" "dclass"="zv" "feature"="avx512vnni" "nodecode" {
  %r = call <16 x i32> @llvm.x86.avx512.vpdpbusd.512(<16 x i32> %a0, <64 x i8> %a1, <64 x i8> %a2)
  ret <16 x i32> %r
}

declare <16 x i32> @llvm.x86.avx512.vpdpbusd.512(<16 x i32>, <64 x i8>, <64 x i8>)

define <4 x i32> @X86.avx512.vpdpbusds.128(<4 x i32> "reg"="xv" "tied" %a0, <16 x i8> "reg"="xv" %a1, <16 x i8> "reg"="xv" %a2) "enc"="evexr(d, a2, 2, 0, a1, 0, 1, 0, 0, 0) | 01010001 | 11 reg:3=d&7 rm:3=a2&7" "asm"="vpdpbusds {a2}, {a1}, {d}" "dclass"="xv" "feature"="avxvnni" "nodecode" {
  %r = call <4 x i32> @llvm.x86.avx512.vpdpbusds.128(<4 x i32> %a0, <16 x i8> %a1, <16 x i8> %a2)
  ret <4 x i32> %r
}

declare <4 x i32> @llvm.x86.avx512.vpdpbusds.128(<4 x i32>, <16 x i8>, <16 x i8>)

define <8 x i32> @X86.avx512.vpdpbusds.256(<8 x i32> "reg"="yv" "tied" %a0, <32 x i8> "reg"="yv" %a1, <32 x i8> "reg"="yv" %a2) "enc"="evexr(d, a2, 2, 0, a1, 1, 1, 0, 0, 0) | 01010001 | 11 reg:3=d&7 rm:3=a2&7" "asm"="vpdpbusds {a2}, {a1}, {d}" "dclass"="yv" "feature"="avxvnni" "nodecode" {
  %r = call <8 x i32> @llvm.x86.avx512.vpdpbusds.256(<8 x i32> %a0, <32 x i8> %a1, <32 x i8> %a2)
  ret <8 x i32> %r
}

declare <8 x i32> @llvm.x86.avx512.vpdpbusds.256(<8 x i32>, <32 x i8>, <32 x i8>)

define <16 x i32> @X86.avx512.vpdpbusds.512(<16 x i32> "reg"="zv" "tied" %a0, <64 x i8> "reg"="zv" %a1, <64 x i8> "reg"="zv" %a2) "enc"="evexr(d, a2, 2, 0, a1, 2, 1, 0, 0, 0) | 01010001 | 11 reg:3=d&7 rm:3=a2&7" "asm"="vpdpbusds {a2}, {a1}, {d}" "dclass"="zv" "feature"="avx512vnni" "nodecode" {
  %r = call <16 x i32> @llvm.x86.avx512.vpdpbusds.512(<16 x i32> %a0, <64 x i8> %a1, <64 x i8> %a2)
  ret <16 x i32> %r
}

declare <16 x i32> @llvm.x86.avx512.vpdpbusds.512(<16 x i32>, <64 x i8>, <64 x i8>)

define <4 x i32> @X86.avx512.vpdpwssd.128(<4 x i32> "reg"="xv" "tied" %a0, <8 x i16> "reg"="xv" %a1, <8 x i16> "reg"="xv" %a2) "enc"="evexr(d, a2, 2, 0, a1, 0, 1, 0, 0, 0) | 01010010 | 11 reg:3=d&7 rm:3=a2&7" "asm"="vpdpwssd {a2}, {a1}, {d}" "dclass"="xv" "feature"="avxvnni" "nodecode" {
  %r = call <4 x i32> @llvm.x86.avx512.vpdpwssd.128(<4 x i32> %a0, <8 x i16> %a1, <8 x i16> %a2)
  ret <4 x i32> %r
}

declare <4 x i32> @llvm.x86.avx512.vpdpwssd.128(<4 x i32>, <8 x i16>, <8 x i16>)

define <8 x i32> @X86.avx512.vpdpwssd.256(<8 x i32> "reg"="yv" "tied" %a0, <16 x i16> "reg"="yv" %a1, <16 x i16> "reg"="yv" %a2) "enc"="evexr(d, a2, 2, 0, a1, 1, 1, 0, 0, 0) | 01010010 | 11 reg:3=d&7 rm:3=a2&7" "asm"="vpdpwssd {a2}, {a1}, {d}" "dclass"="yv" "feature"="avxvnni" "nodecode" {
  %r = call <8 x i32> @llvm.x86.avx512.vpdpwssd.256(<8 x i32> %a0, <16 x i16> %a1, <16 x i16> %a2)
  ret <8 x i32> %r
}

declare <8 x i32> @llvm.x86.avx512.vpdpwssd.256(<8 x i32>, <16 x i16>, <16 x i16>)

define <16 x i32> @X86.avx512.vpdpwssd.512(<16 x i32> "reg"="zv" "tied" %a0, <32 x i16> "reg"="zv" %a1, <32 x i16> "reg"="zv" %a2) "enc"="evexr(d, a2, 2, 0, a1, 2, 1, 0, 0, 0) | 01010010 | 11 reg:3=d&7 rm:3=a2&7" "asm"="vpdpwssd {a2}, {a1}, {d}" "dclass"="zv" "feature"="avx512vnni" "nodecode" {
  %r = call <16 x i32> @llvm.x86.avx512.vpdpwssd.512(<16 x i32> %a0, <32 x i16> %a1, <32 x i16> %a2)
  ret <16 x i32> %r
}

declare <16 x i32> @llvm.x86.avx512.vpdpwssd.512(<16 x i32>, <32 x i16>, <32 x i16>)

define <4 x i32> @X86.avx512.vpdpwssds.128(<4 x i32> "reg"="xv" "tied" %a0, <8 x i16> "reg"="xv" %a1, <8 x i16> "reg"="xv" %a2) "enc"="evexr(d, a2, 2, 0, a1, 0, 1, 0, 0, 0) | 01010011 | 11 reg:3=d&7 rm:3=a2&7" "asm"="vpdpwssds {a2}, {a1}, {d}" "dclass"="xv" "feature"="avxvnni" "nodecode" {
  %r = call <4 x i32> @llvm.x86.avx512.vpdpwssds.128(<4 x i32> %a0, <8 x i16> %a1, <8 x i16> %a2)
  ret <4 x i32> %r
}

declare <4 x i32> @llvm.x86.avx512.vpdpwssds.128(<4 x i32>, <8 x i16>, <8 x i16>)

define <8 x i32> @X86.avx512.vpdpwssds.256(<8 x i32> "reg"="yv" "tied" %a0, <16 x i16> "reg"="yv" %a1, <16 x i16> "reg"="yv" %a2) "enc"="evexr(d, a2, 2, 0, a1, 1, 1, 0, 0, 0) | 01010011 | 11 reg:3=d&7 rm:3=a2&7" "asm"="vpdpwssds {a2}, {a1}, {d}" "dclass"="yv" "feature"="avxvnni" "nodecode" {
  %r = call <8 x i32> @llvm.x86.avx512.vpdpwssds.256(<8 x i32> %a0, <16 x i16> %a1, <16 x i16> %a2)
  ret <8 x i32> %r
}

declare <8 x i32> @llvm.x86.avx512.vpdpwssds.256(<8 x i32>, <16 x i16>, <16 x i16>)

define <16 x i32> @X86.avx512.vpdpwssds.512(<16 x i32> "reg"="zv" "tied" %a0, <32 x i16> "reg"="zv" %a1, <32 x i16> "reg"="zv" %a2) "enc"="evexr(d, a2, 2, 0, a1, 2, 1, 0, 0, 0) | 01010011 | 11 reg:3=d&7 rm:3=a2&7" "asm"="vpdpwssds {a2}, {a1}, {d}" "dclass"="zv" "feature"="avx512vnni" "nodecode" {
  %r = call <16 x i32> @llvm.x86.avx512.vpdpwssds.512(<16 x i32> %a0, <32 x i16> %a1, <32 x i16> %a2)
  ret <16 x i32> %r
}

declare <16 x i32> @llvm.x86.avx512.vpdpwssds.512(<16 x i32>, <32 x i16>, <32 x i16>)

define <4 x i32> @X86.avx512.vpermi2var.d.128(<4 x i32> "reg"="xv" "tied" %a0, <4 x i32> "reg"="xv" %a1, <4 x i32> "reg"="xv" %a2) "enc"="evexr(d, a2, 2, 0, a1, 0, 1, 0, 0, 0) | 01111110 | 11 reg:3=d&7 rm:3=a2&7" "asm"="vpermt2d {a2}, {a1}, {d}" "dclass"="xv" "feature"="avx512f,avx512vl" "nodecode" {
  %r = call <4 x i32> @llvm.x86.avx512.vpermi2var.d.128(<4 x i32> %a0, <4 x i32> %a1, <4 x i32> %a2)
  ret <4 x i32> %r
}

declare <4 x i32> @llvm.x86.avx512.vpermi2var.d.128(<4 x i32>, <4 x i32>, <4 x i32>)

define <8 x i32> @X86.avx512.vpermi2var.d.256(<8 x i32> "reg"="yv" "tied" %a0, <8 x i32> "reg"="yv" %a1, <8 x i32> "reg"="yv" %a2) "enc"="evexr(d, a2, 2, 0, a1, 1, 1, 0, 0, 0) | 01111110 | 11 reg:3=d&7 rm:3=a2&7" "asm"="vpermt2d {a2}, {a1}, {d}" "dclass"="yv" "feature"="avx512f,avx512vl" "nodecode" {
  %r = call <8 x i32> @llvm.x86.avx512.vpermi2var.d.256(<8 x i32> %a0, <8 x i32> %a1, <8 x i32> %a2)
  ret <8 x i32> %r
}

declare <8 x i32> @llvm.x86.avx512.vpermi2var.d.256(<8 x i32>, <8 x i32>, <8 x i32>)

define <16 x i32> @X86.avx512.vpermi2var.d.512(<16 x i32> "reg"="zv" "tied" %a0, <16 x i32> "reg"="zv" %a1, <16 x i32> "reg"="zv" %a2) "enc"="evexr(d, a2, 2, 0, a1, 2, 1, 0, 0, 0) | 01111110 | 11 reg:3=d&7 rm:3=a2&7" "asm"="vpermt2d {a2}, {a1}, {d}" "dclass"="zv" "feature"="avx512f" "nodecode" {
  %r = call <16 x i32> @llvm.x86.avx512.vpermi2var.d.512(<16 x i32> %a0, <16 x i32> %a1, <16 x i32> %a2)
  ret <16 x i32> %r
}

declare <16 x i32> @llvm.x86.avx512.vpermi2var.d.512(<16 x i32>, <16 x i32>, <16 x i32>)

define <8 x i16> @X86.avx512.vpermi2var.hi.128(<8 x i16> "reg"="xv" "tied" %a0, <8 x i16> "reg"="xv" %a1, <8 x i16> "reg"="xv" %a2) "enc"="evexr(d, a2, 2, 1, a1, 0, 1, 0, 0, 0) | 01111101 | 11 reg:3=d&7 rm:3=a2&7" "asm"="vpermt2w {a2}, {a1}, {d}" "dclass"="xv" "feature"="avx512bw,avx512vl" "nodecode" {
  %r = call <8 x i16> @llvm.x86.avx512.vpermi2var.hi.128(<8 x i16> %a0, <8 x i16> %a1, <8 x i16> %a2)
  ret <8 x i16> %r
}

declare <8 x i16> @llvm.x86.avx512.vpermi2var.hi.128(<8 x i16>, <8 x i16>, <8 x i16>)

define <16 x i16> @X86.avx512.vpermi2var.hi.256(<16 x i16> "reg"="yv" "tied" %a0, <16 x i16> "reg"="yv" %a1, <16 x i16> "reg"="yv" %a2) "enc"="evexr(d, a2, 2, 1, a1, 1, 1, 0, 0, 0) | 01111101 | 11 reg:3=d&7 rm:3=a2&7" "asm"="vpermt2w {a2}, {a1}, {d}" "dclass"="yv" "feature"="avx512bw,avx512vl" "nodecode" {
  %r = call <16 x i16> @llvm.x86.avx512.vpermi2var.hi.256(<16 x i16> %a0, <16 x i16> %a1, <16 x i16> %a2)
  ret <16 x i16> %r
}

declare <16 x i16> @llvm.x86.avx512.vpermi2var.hi.256(<16 x i16>, <16 x i16>, <16 x i16>)

define <32 x i16> @X86.avx512.vpermi2var.hi.512(<32 x i16> "reg"="zv" "tied" %a0, <32 x i16> "reg"="zv" %a1, <32 x i16> "reg"="zv" %a2) "enc"="evexr(d, a2, 2, 1, a1, 2, 1, 0, 0, 0) | 01111101 | 11 reg:3=d&7 rm:3=a2&7" "asm"="vpermt2w {a2}, {a1}, {d}" "dclass"="zv" "feature"="avx512bw" "nodecode" {
  %r = call <32 x i16> @llvm.x86.avx512.vpermi2var.hi.512(<32 x i16> %a0, <32 x i16> %a1, <32 x i16> %a2)
  ret <32 x i16> %r
}

declare <32 x i16> @llvm.x86.avx512.vpermi2var.hi.512(<32 x i16>, <32 x i16>, <32 x i16>)

define <2 x double> @X86.avx512.vpermi2var.pd.128(<2 x double> "reg"="xv" "tied" %a0, <2 x i64> "reg"="xv" %a1, <2 x double> "reg"="xv" %a2) "enc"="evexr(d, a2, 2, 1, a1, 0, 1, 0, 0, 0) | 01111111 | 11 reg:3=d&7 rm:3=a2&7" "asm"="vpermt2pd {a2}, {a1}, {d}" "dclass"="xv" "feature"="avx512f,avx512vl" "nodecode" {
  %r = call <2 x double> @llvm.x86.avx512.vpermi2var.pd.128(<2 x double> %a0, <2 x i64> %a1, <2 x double> %a2)
  ret <2 x double> %r
}

declare <2 x double> @llvm.x86.avx512.vpermi2var.pd.128(<2 x double>, <2 x i64>, <2 x double>)

define <4 x double> @X86.avx512.vpermi2var.pd.256(<4 x double> "reg"="yv" "tied" %a0, <4 x i64> "reg"="yv" %a1, <4 x double> "reg"="yv" %a2) "enc"="evexr(d, a2, 2, 1, a1, 1, 1, 0, 0, 0) | 01111111 | 11 reg:3=d&7 rm:3=a2&7" "asm"="vpermt2pd {a2}, {a1}, {d}" "dclass"="yv" "feature"="avx512f,avx512vl" "nodecode" {
  %r = call <4 x double> @llvm.x86.avx512.vpermi2var.pd.256(<4 x double> %a0, <4 x i64> %a1, <4 x double> %a2)
  ret <4 x double> %r
}

declare <4 x double> @llvm.x86.avx512.vpermi2var.pd.256(<4 x double>, <4 x i64>, <4 x double>)

define <8 x double> @X86.avx512.vpermi2var.pd.512(<8 x double> "reg"="zv" "tied" %a0, <8 x i64> "reg"="zv" %a1, <8 x double> "reg"="zv" %a2) "enc"="evexr(d, a2, 2, 1, a1, 2, 1, 0, 0, 0) | 01111111 | 11 reg:3=d&7 rm:3=a2&7" "asm"="vpermt2pd {a2}, {a1}, {d}" "dclass"="zv" "feature"="avx512f" "nodecode" {
  %r = call <8 x double> @llvm.x86.avx512.vpermi2var.pd.512(<8 x double> %a0, <8 x i64> %a1, <8 x double> %a2)
  ret <8 x double> %r
}

declare <8 x double> @llvm.x86.avx512.vpermi2var.pd.512(<8 x double>, <8 x i64>, <8 x double>)

define <4 x float> @X86.avx512.vpermi2var.ps.128(<4 x float> "reg"="xv" "tied" %a0, <4 x i32> "reg"="xv" %a1, <4 x float> "reg"="xv" %a2) "enc"="evexr(d, a2, 2, 0, a1, 0, 1, 0, 0, 0) | 01111111 | 11 reg:3=d&7 rm:3=a2&7" "asm"="vpermt2ps {a2}, {a1}, {d}" "dclass"="xv" "feature"="avx512f,avx512vl" "nodecode" {
  %r = call <4 x float> @llvm.x86.avx512.vpermi2var.ps.128(<4 x float> %a0, <4 x i32> %a1, <4 x float> %a2)
  ret <4 x float> %r
}

declare <4 x float> @llvm.x86.avx512.vpermi2var.ps.128(<4 x float>, <4 x i32>, <4 x float>)

define <8 x float> @X86.avx512.vpermi2var.ps.256(<8 x float> "reg"="yv" "tied" %a0, <8 x i32> "reg"="yv" %a1, <8 x float> "reg"="yv" %a2) "enc"="evexr(d, a2, 2, 0, a1, 1, 1, 0, 0, 0) | 01111111 | 11 reg:3=d&7 rm:3=a2&7" "asm"="vpermt2ps {a2}, {a1}, {d}" "dclass"="yv" "feature"="avx512f,avx512vl" "nodecode" {
  %r = call <8 x float> @llvm.x86.avx512.vpermi2var.ps.256(<8 x float> %a0, <8 x i32> %a1, <8 x float> %a2)
  ret <8 x float> %r
}

declare <8 x float> @llvm.x86.avx512.vpermi2var.ps.256(<8 x float>, <8 x i32>, <8 x float>)

define <16 x float> @X86.avx512.vpermi2var.ps.512(<16 x float> "reg"="zv" "tied" %a0, <16 x i32> "reg"="zv" %a1, <16 x float> "reg"="zv" %a2) "enc"="evexr(d, a2, 2, 0, a1, 2, 1, 0, 0, 0) | 01111111 | 11 reg:3=d&7 rm:3=a2&7" "asm"="vpermt2ps {a2}, {a1}, {d}" "dclass"="zv" "feature"="avx512f" "nodecode" {
  %r = call <16 x float> @llvm.x86.avx512.vpermi2var.ps.512(<16 x float> %a0, <16 x i32> %a1, <16 x float> %a2)
  ret <16 x float> %r
}

declare <16 x float> @llvm.x86.avx512.vpermi2var.ps.512(<16 x float>, <16 x i32>, <16 x float>)

define <2 x i64> @X86.avx512.vpermi2var.q.128(<2 x i64> "reg"="xv" "tied" %a0, <2 x i64> "reg"="xv" %a1, <2 x i64> "reg"="xv" %a2) "enc"="evexr(d, a2, 2, 1, a1, 0, 1, 0, 0, 0) | 01111110 | 11 reg:3=d&7 rm:3=a2&7" "asm"="vpermt2q {a2}, {a1}, {d}" "dclass"="xv" "feature"="avx512f,avx512vl" "nodecode" {
  %r = call <2 x i64> @llvm.x86.avx512.vpermi2var.q.128(<2 x i64> %a0, <2 x i64> %a1, <2 x i64> %a2)
  ret <2 x i64> %r
}

declare <2 x i64> @llvm.x86.avx512.vpermi2var.q.128(<2 x i64>, <2 x i64>, <2 x i64>)

define <4 x i64> @X86.avx512.vpermi2var.q.256(<4 x i64> "reg"="yv" "tied" %a0, <4 x i64> "reg"="yv" %a1, <4 x i64> "reg"="yv" %a2) "enc"="evexr(d, a2, 2, 1, a1, 1, 1, 0, 0, 0) | 01111110 | 11 reg:3=d&7 rm:3=a2&7" "asm"="vpermt2q {a2}, {a1}, {d}" "dclass"="yv" "feature"="avx512f,avx512vl" "nodecode" {
  %r = call <4 x i64> @llvm.x86.avx512.vpermi2var.q.256(<4 x i64> %a0, <4 x i64> %a1, <4 x i64> %a2)
  ret <4 x i64> %r
}

declare <4 x i64> @llvm.x86.avx512.vpermi2var.q.256(<4 x i64>, <4 x i64>, <4 x i64>)

define <8 x i64> @X86.avx512.vpermi2var.q.512(<8 x i64> "reg"="zv" "tied" %a0, <8 x i64> "reg"="zv" %a1, <8 x i64> "reg"="zv" %a2) "enc"="evexr(d, a2, 2, 1, a1, 2, 1, 0, 0, 0) | 01111110 | 11 reg:3=d&7 rm:3=a2&7" "asm"="vpermt2q {a2}, {a1}, {d}" "dclass"="zv" "feature"="avx512f" "nodecode" {
  %r = call <8 x i64> @llvm.x86.avx512.vpermi2var.q.512(<8 x i64> %a0, <8 x i64> %a1, <8 x i64> %a2)
  ret <8 x i64> %r
}

declare <8 x i64> @llvm.x86.avx512.vpermi2var.q.512(<8 x i64>, <8 x i64>, <8 x i64>)

define <16 x i8> @X86.avx512.vpermi2var.qi.128(<16 x i8> "reg"="xv" "tied" %a0, <16 x i8> "reg"="xv" %a1, <16 x i8> "reg"="xv" %a2) "enc"="evexr(d, a2, 2, 0, a1, 0, 1, 0, 0, 0) | 01111101 | 11 reg:3=d&7 rm:3=a2&7" "asm"="vpermt2b {a2}, {a1}, {d}" "dclass"="xv" "feature"="avx512vbmi,avx512vl" "nodecode" {
  %r = call <16 x i8> @llvm.x86.avx512.vpermi2var.qi.128(<16 x i8> %a0, <16 x i8> %a1, <16 x i8> %a2)
  ret <16 x i8> %r
}

declare <16 x i8> @llvm.x86.avx512.vpermi2var.qi.128(<16 x i8>, <16 x i8>, <16 x i8>)

define <32 x i8> @X86.avx512.vpermi2var.qi.256(<32 x i8> "reg"="yv" "tied" %a0, <32 x i8> "reg"="yv" %a1, <32 x i8> "reg"="yv" %a2) "enc"="evexr(d, a2, 2, 0, a1, 1, 1, 0, 0, 0) | 01111101 | 11 reg:3=d&7 rm:3=a2&7" "asm"="vpermt2b {a2}, {a1}, {d}" "dclass"="yv" "feature"="avx512vbmi,avx512vl" "nodecode" {
  %r = call <32 x i8> @llvm.x86.avx512.vpermi2var.qi.256(<32 x i8> %a0, <32 x i8> %a1, <32 x i8> %a2)
  ret <32 x i8> %r
}

declare <32 x i8> @llvm.x86.avx512.vpermi2var.qi.256(<32 x i8>, <32 x i8>, <32 x i8>)

define <64 x i8> @X86.avx512.vpermi2var.qi.512(<64 x i8> "reg"="zv" "tied" %a0, <64 x i8> "reg"="zv" %a1, <64 x i8> "reg"="zv" %a2) "enc"="evexr(d, a2, 2, 0, a1, 2, 1, 0, 0, 0) | 01111101 | 11 reg:3=d&7 rm:3=a2&7" "asm"="vpermt2b {a2}, {a1}, {d}" "dclass"="zv" "feature"="avx512vbmi" "nodecode" {
  %r = call <64 x i8> @llvm.x86.avx512.vpermi2var.qi.512(<64 x i8> %a0, <64 x i8> %a1, <64 x i8> %a2)
  ret <64 x i8> %r
}

declare <64 x i8> @llvm.x86.avx512.vpermi2var.qi.512(<64 x i8>, <64 x i8>, <64 x i8>)

define <8 x double> @X86.avx512.vpermilvar.pd.512(<8 x double> "reg"="zv" %a0, <8 x i64> "reg"="zv" %a1) "enc"="evexr(d, a1, 2, 1, a0, 2, 1, 0, 0, 0) | 00001101 | 11 reg:3=d&7 rm:3=a1&7" "asm"="vpermilpd {a1}, {a0}, {d}" "dclass"="zv" "feature"="avx512f" "nodecode" {
  %r = call <8 x double> @llvm.x86.avx512.vpermilvar.pd.512(<8 x double> %a0, <8 x i64> %a1)
  ret <8 x double> %r
}

declare <8 x double> @llvm.x86.avx512.vpermilvar.pd.512(<8 x double>, <8 x i64>)

define <16 x float> @X86.avx512.vpermilvar.ps.512(<16 x float> "reg"="zv" %a0, <16 x i32> "reg"="zv" %a1) "enc"="evexr(d, a1, 2, 0, a0, 2, 1, 0, 0, 0) | 00001100 | 11 reg:3=d&7 rm:3=a1&7" "asm"="vpermilps {a1}, {a0}, {d}" "dclass"="zv" "feature"="avx512f" "nodecode" {
  %r = call <16 x float> @llvm.x86.avx512.vpermilvar.ps.512(<16 x float> %a0, <16 x i32> %a1)
  ret <16 x float> %r
}

declare <16 x float> @llvm.x86.avx512.vpermilvar.ps.512(<16 x float>, <16 x i32>)

define <2 x i64> @X86.avx512.vpmadd52h.uq.128(<2 x i64> "reg"="xv" "tied" %a0, <2 x i64> "reg"="xv" %a1, <2 x i64> "reg"="xv" %a2) "enc"="evexr(d, a2, 2, 1, a1, 0, 1, 0, 0, 0) | 10110101 | 11 reg:3=d&7 rm:3=a2&7" "asm"="vpmadd52huq {a2}, {a1}, {d}" "dclass"="xv" "feature"="avxifma" "nodecode" {
  %r = call <2 x i64> @llvm.x86.avx512.vpmadd52h.uq.128(<2 x i64> %a0, <2 x i64> %a1, <2 x i64> %a2)
  ret <2 x i64> %r
}

declare <2 x i64> @llvm.x86.avx512.vpmadd52h.uq.128(<2 x i64>, <2 x i64>, <2 x i64>)

define <4 x i64> @X86.avx512.vpmadd52h.uq.256(<4 x i64> "reg"="yv" "tied" %a0, <4 x i64> "reg"="yv" %a1, <4 x i64> "reg"="yv" %a2) "enc"="evexr(d, a2, 2, 1, a1, 1, 1, 0, 0, 0) | 10110101 | 11 reg:3=d&7 rm:3=a2&7" "asm"="vpmadd52huq {a2}, {a1}, {d}" "dclass"="yv" "feature"="avxifma" "nodecode" {
  %r = call <4 x i64> @llvm.x86.avx512.vpmadd52h.uq.256(<4 x i64> %a0, <4 x i64> %a1, <4 x i64> %a2)
  ret <4 x i64> %r
}

declare <4 x i64> @llvm.x86.avx512.vpmadd52h.uq.256(<4 x i64>, <4 x i64>, <4 x i64>)

define <8 x i64> @X86.avx512.vpmadd52h.uq.512(<8 x i64> "reg"="zv" "tied" %a0, <8 x i64> "reg"="zv" %a1, <8 x i64> "reg"="zv" %a2) "enc"="evexr(d, a2, 2, 1, a1, 2, 1, 0, 0, 0) | 10110101 | 11 reg:3=d&7 rm:3=a2&7" "asm"="vpmadd52huq {a2}, {a1}, {d}" "dclass"="zv" "feature"="avx512ifma" "nodecode" {
  %r = call <8 x i64> @llvm.x86.avx512.vpmadd52h.uq.512(<8 x i64> %a0, <8 x i64> %a1, <8 x i64> %a2)
  ret <8 x i64> %r
}

declare <8 x i64> @llvm.x86.avx512.vpmadd52h.uq.512(<8 x i64>, <8 x i64>, <8 x i64>)

define <2 x i64> @X86.avx512.vpmadd52l.uq.128(<2 x i64> "reg"="xv" "tied" %a0, <2 x i64> "reg"="xv" %a1, <2 x i64> "reg"="xv" %a2) "enc"="evexr(d, a2, 2, 1, a1, 0, 1, 0, 0, 0) | 10110100 | 11 reg:3=d&7 rm:3=a2&7" "asm"="vpmadd52luq {a2}, {a1}, {d}" "dclass"="xv" "feature"="avxifma" "nodecode" {
  %r = call <2 x i64> @llvm.x86.avx512.vpmadd52l.uq.128(<2 x i64> %a0, <2 x i64> %a1, <2 x i64> %a2)
  ret <2 x i64> %r
}

declare <2 x i64> @llvm.x86.avx512.vpmadd52l.uq.128(<2 x i64>, <2 x i64>, <2 x i64>)

define <4 x i64> @X86.avx512.vpmadd52l.uq.256(<4 x i64> "reg"="yv" "tied" %a0, <4 x i64> "reg"="yv" %a1, <4 x i64> "reg"="yv" %a2) "enc"="evexr(d, a2, 2, 1, a1, 1, 1, 0, 0, 0) | 10110100 | 11 reg:3=d&7 rm:3=a2&7" "asm"="vpmadd52luq {a2}, {a1}, {d}" "dclass"="yv" "feature"="avxifma" "nodecode" {
  %r = call <4 x i64> @llvm.x86.avx512.vpmadd52l.uq.256(<4 x i64> %a0, <4 x i64> %a1, <4 x i64> %a2)
  ret <4 x i64> %r
}

declare <4 x i64> @llvm.x86.avx512.vpmadd52l.uq.256(<4 x i64>, <4 x i64>, <4 x i64>)

define <8 x i64> @X86.avx512.vpmadd52l.uq.512(<8 x i64> "reg"="zv" "tied" %a0, <8 x i64> "reg"="zv" %a1, <8 x i64> "reg"="zv" %a2) "enc"="evexr(d, a2, 2, 1, a1, 2, 1, 0, 0, 0) | 10110100 | 11 reg:3=d&7 rm:3=a2&7" "asm"="vpmadd52luq {a2}, {a1}, {d}" "dclass"="zv" "feature"="avx512ifma" "nodecode" {
  %r = call <8 x i64> @llvm.x86.avx512.vpmadd52l.uq.512(<8 x i64> %a0, <8 x i64> %a1, <8 x i64> %a2)
  ret <8 x i64> %r
}

declare <8 x i64> @llvm.x86.avx512.vpmadd52l.uq.512(<8 x i64>, <8 x i64>, <8 x i64>)

define <8 x i16> @X86.avx512bf16.cvtne2ps2bf16.128(<4 x float> "reg"="xv" %a0, <4 x float> "reg"="xv" %a1) "enc"="evexr(d, a1, 2, 0, a0, 0, 3, 0, 0, 0) | 01110010 | 11 reg:3=d&7 rm:3=a1&7" "asm"="vcvtne2ps2bf16 {a1}, {a0}, {d}" "dclass"="xv" "feature"="avx512bf16,avx512vl" "nodecode" {
  %r = call <8 x i16> @llvm.x86.avx512bf16.cvtne2ps2bf16.128(<4 x float> %a0, <4 x float> %a1)
  ret <8 x i16> %r
}

declare <8 x i16> @llvm.x86.avx512bf16.cvtne2ps2bf16.128(<4 x float>, <4 x float>)

define <16 x i16> @X86.avx512bf16.cvtne2ps2bf16.256(<8 x float> "reg"="yv" %a0, <8 x float> "reg"="yv" %a1) "enc"="evexr(d, a1, 2, 0, a0, 1, 3, 0, 0, 0) | 01110010 | 11 reg:3=d&7 rm:3=a1&7" "asm"="vcvtne2ps2bf16 {a1}, {a0}, {d}" "dclass"="yv" "feature"="avx512bf16,avx512vl" "nodecode" {
  %r = call <16 x i16> @llvm.x86.avx512bf16.cvtne2ps2bf16.256(<8 x float> %a0, <8 x float> %a1)
  ret <16 x i16> %r
}

declare <16 x i16> @llvm.x86.avx512bf16.cvtne2ps2bf16.256(<8 x float>, <8 x float>)

define <32 x i16> @X86.avx512bf16.cvtne2ps2bf16.512(<16 x float> "reg"="zv" %a0, <16 x float> "reg"="zv" %a1) "enc"="evexr(d, a1, 2, 0, a0, 2, 3, 0, 0, 0) | 01110010 | 11 reg:3=d&7 rm:3=a1&7" "asm"="vcvtne2ps2bf16 {a1}, {a0}, {d}" "dclass"="zv" "feature"="avx512bf16,avx512f" "nodecode" {
  %r = call <32 x i16> @llvm.x86.avx512bf16.cvtne2ps2bf16.512(<16 x float> %a0, <16 x float> %a1)
  ret <32 x i16> %r
}

declare <32 x i16> @llvm.x86.avx512bf16.cvtne2ps2bf16.512(<16 x float>, <16 x float>)

define <8 x i16> @X86.avx512bf16.cvtneps2bf16.256(<8 x float> "reg"="yv" %a0) "enc"="evexr(d, a0, 2, 0, 0, 1, 2, 0, 0, 0) | 01110010 | 11 reg:3=d&7 rm:3=a0&7" "asm"="vcvtneps2bf16 {a0}, {d}" "dclass"="xv" "feature"="avx512bf16,avx512vl" "nodecode" {
  %r = call <8 x i16> @llvm.x86.avx512bf16.cvtneps2bf16.256(<8 x float> %a0)
  ret <8 x i16> %r
}

declare <8 x i16> @llvm.x86.avx512bf16.cvtneps2bf16.256(<8 x float>)

define <16 x i16> @X86.avx512bf16.cvtneps2bf16.512(<16 x float> "reg"="zv" %a0) "enc"="evexr(d, a0, 2, 0, 0, 2, 2, 0, 0, 0) | 01110010 | 11 reg:3=d&7 rm:3=a0&7" "asm"="vcvtneps2bf16 {a0}, {d}" "dclass"="yv" "feature"="avx512bf16,avx512f" "nodecode" {
  %r = call <16 x i16> @llvm.x86.avx512bf16.cvtneps2bf16.512(<16 x float> %a0)
  ret <16 x i16> %r
}

declare <16 x i16> @llvm.x86.avx512bf16.cvtneps2bf16.512(<16 x float>)

define <4 x float> @X86.avx512bf16.dpbf16ps.128(<4 x float> "reg"="xv" "tied" %a0, <8 x i16> "reg"="xv" %a1, <8 x i16> "reg"="xv" %a2) "enc"="evexr(d, a2, 2, 0, a1, 0, 2, 0, 0, 0) | 01010010 | 11 reg:3=d&7 rm:3=a2&7" "asm"="vdpbf16ps {a2}, {a1}, {d}" "dclass"="xv" "feature"="avx512bf16,avx512vl" "nodecode" {
  %r = call <4 x float> @llvm.x86.avx512bf16.dpbf16ps.128(<4 x float> %a0, <8 x i16> %a1, <8 x i16> %a2)
  ret <4 x float> %r
}

declare <4 x float> @llvm.x86.avx512bf16.dpbf16ps.128(<4 x float>, <8 x i16>, <8 x i16>)

define <8 x float> @X86.avx512bf16.dpbf16ps.256(<8 x float> "reg"="yv" "tied" %a0, <16 x i16> "reg"="yv" %a1, <16 x i16> "reg"="yv" %a2) "enc"="evexr(d, a2, 2, 0, a1, 1, 2, 0, 0, 0) | 01010010 | 11 reg:3=d&7 rm:3=a2&7" "asm"="vdpbf16ps {a2}, {a1}, {d}" "dclass"="yv" "feature"="avx512bf16,avx512vl" "nodecode" {
  %r = call <8 x float> @llvm.x86.avx512bf16.dpbf16ps.256(<8 x float> %a0, <16 x i16> %a1, <16 x i16> %a2)
  ret <8 x float> %r
}

declare <8 x float> @llvm.x86.avx512bf16.dpbf16ps.256(<8 x float>, <16 x i16>, <16 x i16>)

define <16 x float> @X86.avx512bf16.dpbf16ps.512(<16 x float> "reg"="zv" "tied" %a0, <32 x i16> "reg"="zv" %a1, <32 x i16> "reg"="zv" %a2) "enc"="evexr(d, a2, 2, 0, a1, 2, 2, 0, 0, 0) | 01010010 | 11 reg:3=d&7 rm:3=a2&7" "asm"="vdpbf16ps {a2}, {a1}, {d}" "dclass"="zv" "feature"="avx512bf16,avx512f" "nodecode" {
  %r = call <16 x float> @llvm.x86.avx512bf16.dpbf16ps.512(<16 x float> %a0, <32 x i16> %a1, <32 x i16> %a2)
  ret <16 x float> %r
}

declare <16 x float> @llvm.x86.avx512bf16.dpbf16ps.512(<16 x float>, <32 x i16>, <32 x i16>)

define <4 x float> @X86.avx512fp16.mask.vfcmadd.cph.128(<4 x float> "reg"="xv" %a0, <4 x float> "reg"="xv" %a1, <4 x float> "reg"="xv" "tied" %a2, i8 "reg"="k" %a3) "enc"="{ if(a3!=0) evexr(d, a1, 6, 0, a0, 0, 3, 0, a3, 0) | 01010110 | 11 reg:3=d&7 rm:3=a1&7 }" "asm"="vfcmaddcph {a1}, {a0}, {d} {lb}{a3}{rb}" "dclass"="xv" "feature"="avx512fp16,avx512vl" "nodecode" {
  %r = call <4 x float> @llvm.x86.avx512fp16.mask.vfcmadd.cph.128(<4 x float> %a0, <4 x float> %a1, <4 x float> %a2, i8 %a3)
  ret <4 x float> %r
}

declare <4 x float> @llvm.x86.avx512fp16.mask.vfcmadd.cph.128(<4 x float>, <4 x float>, <4 x float>, i8)

define <8 x float> @X86.avx512fp16.mask.vfcmadd.cph.256(<8 x float> "reg"="yv" %a0, <8 x float> "reg"="yv" %a1, <8 x float> "reg"="yv" "tied" %a2, i8 "reg"="k" %a3) "enc"="{ if(a3!=0) evexr(d, a1, 6, 0, a0, 1, 3, 0, a3, 0) | 01010110 | 11 reg:3=d&7 rm:3=a1&7 }" "asm"="vfcmaddcph {a1}, {a0}, {d} {lb}{a3}{rb}" "dclass"="yv" "feature"="avx512fp16,avx512vl" "nodecode" {
  %r = call <8 x float> @llvm.x86.avx512fp16.mask.vfcmadd.cph.256(<8 x float> %a0, <8 x float> %a1, <8 x float> %a2, i8 %a3)
  ret <8 x float> %r
}

declare <8 x float> @llvm.x86.avx512fp16.mask.vfcmadd.cph.256(<8 x float>, <8 x float>, <8 x float>, i8)

define <16 x float> @X86.avx512fp16.mask.vfcmadd.cph.512(<16 x float> "reg"="zv" %a0, <16 x float> "reg"="zv" %a1, <16 x float> "reg"="zv" "tied" %a2, i16 "reg"="k" %a3) "enc"="{ if(a3!=0) evexr(d, a1, 6, 0, a0, 2, 3, 0, a3, 0) | 01010110 | 11 reg:3=d&7 rm:3=a1&7 }" "asm"="vfcmaddcph {a1}, {a0}, {d} {lb}{a3}{rb}" "dclass"="zv" "feature"="avx512fp16" "nodecode" {
  %r = call <16 x float> @llvm.x86.avx512fp16.mask.vfcmadd.cph.512(<16 x float> %a0, <16 x float> %a1, <16 x float> %a2, i16 %a3, i32 4)
  ret <16 x float> %r
}

declare <16 x float> @llvm.x86.avx512fp16.mask.vfcmadd.cph.512(<16 x float>, <16 x float>, <16 x float>, i16, i32)

define <4 x float> @X86.avx512fp16.mask.vfcmadd.csh(<4 x float> "reg"="xv" %a0, <4 x float> "reg"="xv" %a1, <4 x float> "reg"="xv" "tied" %a2, i8 "reg"="k" %a3) "enc"="{ if(a3!=0) evexr(d, a1, 6, 0, a0, 0, 3, 0, a3, 0) | 01010111 | 11 reg:3=d&7 rm:3=a1&7 }" "asm"="vfcmaddcsh {a1}, {a0}, {d} {lb}{a3}{rb}" "dclass"="xv" "feature"="avx512fp16" "nodecode" {
  %r = call <4 x float> @llvm.x86.avx512fp16.mask.vfcmadd.csh(<4 x float> %a0, <4 x float> %a1, <4 x float> %a2, i8 %a3, i32 4)
  ret <4 x float> %r
}

declare <4 x float> @llvm.x86.avx512fp16.mask.vfcmadd.csh(<4 x float>, <4 x float>, <4 x float>, i8, i32)

define <4 x float> @X86.avx512fp16.mask.vfcmul.cph.128(<4 x float> "reg"="xv" %a0, <4 x float> "reg"="xv" %a1, <4 x float> "reg"="xv" "tied" %a2, i8 "reg"="k" %a3) "enc"="{ if(a3!=0) evexr(d, a1, 6, 0, a0, 0, 3, 0, a3, 0) | 11010110 | 11 reg:3=d&7 rm:3=a1&7 }" "asm"="vfcmulcph {a1}, {a0}, {d} {lb}{a3}{rb}" "dclass"="xv" "feature"="avx512fp16,avx512vl" "nodecode" {
  %r = call <4 x float> @llvm.x86.avx512fp16.mask.vfcmul.cph.128(<4 x float> %a0, <4 x float> %a1, <4 x float> %a2, i8 %a3)
  ret <4 x float> %r
}

declare <4 x float> @llvm.x86.avx512fp16.mask.vfcmul.cph.128(<4 x float>, <4 x float>, <4 x float>, i8)

define <8 x float> @X86.avx512fp16.mask.vfcmul.cph.256(<8 x float> "reg"="yv" %a0, <8 x float> "reg"="yv" %a1, <8 x float> "reg"="yv" "tied" %a2, i8 "reg"="k" %a3) "enc"="{ if(a3!=0) evexr(d, a1, 6, 0, a0, 1, 3, 0, a3, 0) | 11010110 | 11 reg:3=d&7 rm:3=a1&7 }" "asm"="vfcmulcph {a1}, {a0}, {d} {lb}{a3}{rb}" "dclass"="yv" "feature"="avx512fp16,avx512vl" "nodecode" {
  %r = call <8 x float> @llvm.x86.avx512fp16.mask.vfcmul.cph.256(<8 x float> %a0, <8 x float> %a1, <8 x float> %a2, i8 %a3)
  ret <8 x float> %r
}

declare <8 x float> @llvm.x86.avx512fp16.mask.vfcmul.cph.256(<8 x float>, <8 x float>, <8 x float>, i8)

define <16 x float> @X86.avx512fp16.mask.vfcmul.cph.512(<16 x float> "reg"="zv" %a0, <16 x float> "reg"="zv" %a1, <16 x float> "reg"="zv" "tied" %a2, i16 "reg"="k" %a3) "enc"="{ if(a3!=0) evexr(d, a1, 6, 0, a0, 2, 3, 0, a3, 0) | 11010110 | 11 reg:3=d&7 rm:3=a1&7 }" "asm"="vfcmulcph {a1}, {a0}, {d} {lb}{a3}{rb}" "dclass"="zv" "feature"="avx512fp16" "nodecode" {
  %r = call <16 x float> @llvm.x86.avx512fp16.mask.vfcmul.cph.512(<16 x float> %a0, <16 x float> %a1, <16 x float> %a2, i16 %a3, i32 4)
  ret <16 x float> %r
}

declare <16 x float> @llvm.x86.avx512fp16.mask.vfcmul.cph.512(<16 x float>, <16 x float>, <16 x float>, i16, i32)

define <4 x float> @X86.avx512fp16.mask.vfcmul.csh(<4 x float> "reg"="xv" %a0, <4 x float> "reg"="xv" %a1, <4 x float> "reg"="xv" "tied" %a2, i8 "reg"="k" %a3) "enc"="{ if(a3!=0) evexr(d, a1, 6, 0, a0, 0, 3, 0, a3, 0) | 11010111 | 11 reg:3=d&7 rm:3=a1&7 }" "asm"="vfcmulcsh {a1}, {a0}, {d} {lb}{a3}{rb}" "dclass"="xv" "feature"="avx512fp16" "nodecode" {
  %r = call <4 x float> @llvm.x86.avx512fp16.mask.vfcmul.csh(<4 x float> %a0, <4 x float> %a1, <4 x float> %a2, i8 %a3, i32 4)
  ret <4 x float> %r
}

declare <4 x float> @llvm.x86.avx512fp16.mask.vfcmul.csh(<4 x float>, <4 x float>, <4 x float>, i8, i32)

define <4 x float> @X86.avx512fp16.mask.vfmadd.cph.128(<4 x float> "reg"="xv" %a0, <4 x float> "reg"="xv" %a1, <4 x float> "reg"="xv" "tied" %a2, i8 "reg"="k" %a3) "enc"="{ if(a3!=0) evexr(d, a1, 6, 0, a0, 0, 2, 0, a3, 0) | 01010110 | 11 reg:3=d&7 rm:3=a1&7 }" "asm"="vfmaddcph {a1}, {a0}, {d} {lb}{a3}{rb}" "dclass"="xv" "feature"="avx512fp16,avx512vl" "nodecode" {
  %r = call <4 x float> @llvm.x86.avx512fp16.mask.vfmadd.cph.128(<4 x float> %a0, <4 x float> %a1, <4 x float> %a2, i8 %a3)
  ret <4 x float> %r
}

declare <4 x float> @llvm.x86.avx512fp16.mask.vfmadd.cph.128(<4 x float>, <4 x float>, <4 x float>, i8)

define <8 x float> @X86.avx512fp16.mask.vfmadd.cph.256(<8 x float> "reg"="yv" %a0, <8 x float> "reg"="yv" %a1, <8 x float> "reg"="yv" "tied" %a2, i8 "reg"="k" %a3) "enc"="{ if(a3!=0) evexr(d, a1, 6, 0, a0, 1, 2, 0, a3, 0) | 01010110 | 11 reg:3=d&7 rm:3=a1&7 }" "asm"="vfmaddcph {a1}, {a0}, {d} {lb}{a3}{rb}" "dclass"="yv" "feature"="avx512fp16,avx512vl" "nodecode" {
  %r = call <8 x float> @llvm.x86.avx512fp16.mask.vfmadd.cph.256(<8 x float> %a0, <8 x float> %a1, <8 x float> %a2, i8 %a3)
  ret <8 x float> %r
}

declare <8 x float> @llvm.x86.avx512fp16.mask.vfmadd.cph.256(<8 x float>, <8 x float>, <8 x float>, i8)

define <16 x float> @X86.avx512fp16.mask.vfmadd.cph.512(<16 x float> "reg"="zv" %a0, <16 x float> "reg"="zv" %a1, <16 x float> "reg"="zv" "tied" %a2, i16 "reg"="k" %a3) "enc"="{ if(a3!=0) evexr(d, a1, 6, 0, a0, 2, 2, 0, a3, 0) | 01010110 | 11 reg:3=d&7 rm:3=a1&7 }" "asm"="vfmaddcph {a1}, {a0}, {d} {lb}{a3}{rb}" "dclass"="zv" "feature"="avx512fp16" "nodecode" {
  %r = call <16 x float> @llvm.x86.avx512fp16.mask.vfmadd.cph.512(<16 x float> %a0, <16 x float> %a1, <16 x float> %a2, i16 %a3, i32 4)
  ret <16 x float> %r
}

declare <16 x float> @llvm.x86.avx512fp16.mask.vfmadd.cph.512(<16 x float>, <16 x float>, <16 x float>, i16, i32)

define <4 x float> @X86.avx512fp16.mask.vfmadd.csh(<4 x float> "reg"="xv" %a0, <4 x float> "reg"="xv" %a1, <4 x float> "reg"="xv" "tied" %a2, i8 "reg"="k" %a3) "enc"="{ if(a3!=0) evexr(d, a1, 6, 0, a0, 0, 2, 0, a3, 0) | 01010111 | 11 reg:3=d&7 rm:3=a1&7 }" "asm"="vfmaddcsh {a1}, {a0}, {d} {lb}{a3}{rb}" "dclass"="xv" "feature"="avx512fp16" "nodecode" {
  %r = call <4 x float> @llvm.x86.avx512fp16.mask.vfmadd.csh(<4 x float> %a0, <4 x float> %a1, <4 x float> %a2, i8 %a3, i32 4)
  ret <4 x float> %r
}

declare <4 x float> @llvm.x86.avx512fp16.mask.vfmadd.csh(<4 x float>, <4 x float>, <4 x float>, i8, i32)

define <4 x float> @X86.avx512fp16.mask.vfmul.cph.128(<4 x float> "reg"="xv" %a0, <4 x float> "reg"="xv" %a1, <4 x float> "reg"="xv" "tied" %a2, i8 "reg"="k" %a3) "enc"="{ if(a3!=0) evexr(d, a1, 6, 0, a0, 0, 2, 0, a3, 0) | 11010110 | 11 reg:3=d&7 rm:3=a1&7 }" "asm"="vfmulcph {a1}, {a0}, {d} {lb}{a3}{rb}" "dclass"="xv" "feature"="avx512fp16,avx512vl" "nodecode" {
  %r = call <4 x float> @llvm.x86.avx512fp16.mask.vfmul.cph.128(<4 x float> %a0, <4 x float> %a1, <4 x float> %a2, i8 %a3)
  ret <4 x float> %r
}

declare <4 x float> @llvm.x86.avx512fp16.mask.vfmul.cph.128(<4 x float>, <4 x float>, <4 x float>, i8)

define <8 x float> @X86.avx512fp16.mask.vfmul.cph.256(<8 x float> "reg"="yv" %a0, <8 x float> "reg"="yv" %a1, <8 x float> "reg"="yv" "tied" %a2, i8 "reg"="k" %a3) "enc"="{ if(a3!=0) evexr(d, a1, 6, 0, a0, 1, 2, 0, a3, 0) | 11010110 | 11 reg:3=d&7 rm:3=a1&7 }" "asm"="vfmulcph {a1}, {a0}, {d} {lb}{a3}{rb}" "dclass"="yv" "feature"="avx512fp16,avx512vl" "nodecode" {
  %r = call <8 x float> @llvm.x86.avx512fp16.mask.vfmul.cph.256(<8 x float> %a0, <8 x float> %a1, <8 x float> %a2, i8 %a3)
  ret <8 x float> %r
}

declare <8 x float> @llvm.x86.avx512fp16.mask.vfmul.cph.256(<8 x float>, <8 x float>, <8 x float>, i8)

define <16 x float> @X86.avx512fp16.mask.vfmul.cph.512(<16 x float> "reg"="zv" %a0, <16 x float> "reg"="zv" %a1, <16 x float> "reg"="zv" "tied" %a2, i16 "reg"="k" %a3) "enc"="{ if(a3!=0) evexr(d, a1, 6, 0, a0, 2, 2, 0, a3, 0) | 11010110 | 11 reg:3=d&7 rm:3=a1&7 }" "asm"="vfmulcph {a1}, {a0}, {d} {lb}{a3}{rb}" "dclass"="zv" "feature"="avx512fp16" "nodecode" {
  %r = call <16 x float> @llvm.x86.avx512fp16.mask.vfmul.cph.512(<16 x float> %a0, <16 x float> %a1, <16 x float> %a2, i16 %a3, i32 4)
  ret <16 x float> %r
}

declare <16 x float> @llvm.x86.avx512fp16.mask.vfmul.cph.512(<16 x float>, <16 x float>, <16 x float>, i16, i32)

define <4 x float> @X86.avx512fp16.mask.vfmul.csh(<4 x float> "reg"="xv" %a0, <4 x float> "reg"="xv" %a1, <4 x float> "reg"="xv" "tied" %a2, i8 "reg"="k" %a3) "enc"="{ if(a3!=0) evexr(d, a1, 6, 0, a0, 0, 2, 0, a3, 0) | 11010111 | 11 reg:3=d&7 rm:3=a1&7 }" "asm"="vfmulcsh {a1}, {a0}, {d} {lb}{a3}{rb}" "dclass"="xv" "feature"="avx512fp16" "nodecode" {
  %r = call <4 x float> @llvm.x86.avx512fp16.mask.vfmul.csh(<4 x float> %a0, <4 x float> %a1, <4 x float> %a2, i8 %a3, i32 4)
  ret <4 x float> %r
}

declare <4 x float> @llvm.x86.avx512fp16.mask.vfmul.csh(<4 x float>, <4 x float>, <4 x float>, i8, i32)

define <4 x float> @X86.avx512fp16.maskz.vfcmadd.cph.128(<4 x float> "reg"="xv" %a0, <4 x float> "reg"="xv" %a1, <4 x float> "reg"="xv" "tied" %a2, i8 "reg"="k" %a3) "enc"="{ if(a3!=0) evexr(d, a1, 6, 0, a0, 0, 3, 0, a3, 1) | 01010110 | 11 reg:3=d&7 rm:3=a1&7 }" "asm"="vfcmaddcph {a1}, {a0}, {d} {lb}{a3}{rb} {lb}z{rb}" "dclass"="xv" "feature"="avx512fp16,avx512vl" "nodecode" {
  %r = call <4 x float> @llvm.x86.avx512fp16.maskz.vfcmadd.cph.128(<4 x float> %a0, <4 x float> %a1, <4 x float> %a2, i8 %a3)
  ret <4 x float> %r
}

declare <4 x float> @llvm.x86.avx512fp16.maskz.vfcmadd.cph.128(<4 x float>, <4 x float>, <4 x float>, i8)

define <8 x float> @X86.avx512fp16.maskz.vfcmadd.cph.256(<8 x float> "reg"="yv" %a0, <8 x float> "reg"="yv" %a1, <8 x float> "reg"="yv" "tied" %a2, i8 "reg"="k" %a3) "enc"="{ if(a3!=0) evexr(d, a1, 6, 0, a0, 1, 3, 0, a3, 1) | 01010110 | 11 reg:3=d&7 rm:3=a1&7 }" "asm"="vfcmaddcph {a1}, {a0}, {d} {lb}{a3}{rb} {lb}z{rb}" "dclass"="yv" "feature"="avx512fp16,avx512vl" "nodecode" {
  %r = call <8 x float> @llvm.x86.avx512fp16.maskz.vfcmadd.cph.256(<8 x float> %a0, <8 x float> %a1, <8 x float> %a2, i8 %a3)
  ret <8 x float> %r
}

declare <8 x float> @llvm.x86.avx512fp16.maskz.vfcmadd.cph.256(<8 x float>, <8 x float>, <8 x float>, i8)

define <16 x float> @X86.avx512fp16.maskz.vfcmadd.cph.512(<16 x float> "reg"="zv" %a0, <16 x float> "reg"="zv" %a1, <16 x float> "reg"="zv" "tied" %a2, i16 "reg"="k" %a3) "enc"="{ if(a3!=0) evexr(d, a1, 6, 0, a0, 2, 3, 0, a3, 1) | 01010110 | 11 reg:3=d&7 rm:3=a1&7 }" "asm"="vfcmaddcph {a1}, {a0}, {d} {lb}{a3}{rb} {lb}z{rb}" "dclass"="zv" "feature"="avx512fp16" "nodecode" {
  %r = call <16 x float> @llvm.x86.avx512fp16.maskz.vfcmadd.cph.512(<16 x float> %a0, <16 x float> %a1, <16 x float> %a2, i16 %a3, i32 4)
  ret <16 x float> %r
}

declare <16 x float> @llvm.x86.avx512fp16.maskz.vfcmadd.cph.512(<16 x float>, <16 x float>, <16 x float>, i16, i32)

define <4 x float> @X86.avx512fp16.maskz.vfcmadd.csh(<4 x float> "reg"="xv" %a0, <4 x float> "reg"="xv" %a1, <4 x float> "reg"="xv" "tied" %a2, i8 "reg"="k" %a3) "enc"="{ if(a3!=0) evexr(d, a1, 6, 0, a0, 0, 3, 0, a3, 1) | 01010111 | 11 reg:3=d&7 rm:3=a1&7 }" "asm"="vfcmaddcsh {a1}, {a0}, {d} {lb}{a3}{rb} {lb}z{rb}" "dclass"="xv" "feature"="avx512fp16" "nodecode" {
  %r = call <4 x float> @llvm.x86.avx512fp16.maskz.vfcmadd.csh(<4 x float> %a0, <4 x float> %a1, <4 x float> %a2, i8 %a3, i32 4)
  ret <4 x float> %r
}

declare <4 x float> @llvm.x86.avx512fp16.maskz.vfcmadd.csh(<4 x float>, <4 x float>, <4 x float>, i8, i32)

define <4 x float> @X86.avx512fp16.maskz.vfmadd.cph.128(<4 x float> "reg"="xv" %a0, <4 x float> "reg"="xv" %a1, <4 x float> "reg"="xv" "tied" %a2, i8 "reg"="k" %a3) "enc"="{ if(a3!=0) evexr(d, a1, 6, 0, a0, 0, 2, 0, a3, 1) | 01010110 | 11 reg:3=d&7 rm:3=a1&7 }" "asm"="vfmaddcph {a1}, {a0}, {d} {lb}{a3}{rb} {lb}z{rb}" "dclass"="xv" "feature"="avx512fp16,avx512vl" "nodecode" {
  %r = call <4 x float> @llvm.x86.avx512fp16.maskz.vfmadd.cph.128(<4 x float> %a0, <4 x float> %a1, <4 x float> %a2, i8 %a3)
  ret <4 x float> %r
}

declare <4 x float> @llvm.x86.avx512fp16.maskz.vfmadd.cph.128(<4 x float>, <4 x float>, <4 x float>, i8)

define <8 x float> @X86.avx512fp16.maskz.vfmadd.cph.256(<8 x float> "reg"="yv" %a0, <8 x float> "reg"="yv" %a1, <8 x float> "reg"="yv" "tied" %a2, i8 "reg"="k" %a3) "enc"="{ if(a3!=0) evexr(d, a1, 6, 0, a0, 1, 2, 0, a3, 1) | 01010110 | 11 reg:3=d&7 rm:3=a1&7 }" "asm"="vfmaddcph {a1}, {a0}, {d} {lb}{a3}{rb} {lb}z{rb}" "dclass"="yv" "feature"="avx512fp16,avx512vl" "nodecode" {
  %r = call <8 x float> @llvm.x86.avx512fp16.maskz.vfmadd.cph.256(<8 x float> %a0, <8 x float> %a1, <8 x float> %a2, i8 %a3)
  ret <8 x float> %r
}

declare <8 x float> @llvm.x86.avx512fp16.maskz.vfmadd.cph.256(<8 x float>, <8 x float>, <8 x float>, i8)

define <16 x float> @X86.avx512fp16.maskz.vfmadd.cph.512(<16 x float> "reg"="zv" %a0, <16 x float> "reg"="zv" %a1, <16 x float> "reg"="zv" "tied" %a2, i16 "reg"="k" %a3) "enc"="{ if(a3!=0) evexr(d, a1, 6, 0, a0, 2, 2, 0, a3, 1) | 01010110 | 11 reg:3=d&7 rm:3=a1&7 }" "asm"="vfmaddcph {a1}, {a0}, {d} {lb}{a3}{rb} {lb}z{rb}" "dclass"="zv" "feature"="avx512fp16" "nodecode" {
  %r = call <16 x float> @llvm.x86.avx512fp16.maskz.vfmadd.cph.512(<16 x float> %a0, <16 x float> %a1, <16 x float> %a2, i16 %a3, i32 4)
  ret <16 x float> %r
}

declare <16 x float> @llvm.x86.avx512fp16.maskz.vfmadd.cph.512(<16 x float>, <16 x float>, <16 x float>, i16, i32)

define <4 x float> @X86.avx512fp16.maskz.vfmadd.csh(<4 x float> "reg"="xv" %a0, <4 x float> "reg"="xv" %a1, <4 x float> "reg"="xv" "tied" %a2, i8 "reg"="k" %a3) "enc"="{ if(a3!=0) evexr(d, a1, 6, 0, a0, 0, 2, 0, a3, 1) | 01010111 | 11 reg:3=d&7 rm:3=a1&7 }" "asm"="vfmaddcsh {a1}, {a0}, {d} {lb}{a3}{rb} {lb}z{rb}" "dclass"="xv" "feature"="avx512fp16" "nodecode" {
  %r = call <4 x float> @llvm.x86.avx512fp16.maskz.vfmadd.csh(<4 x float> %a0, <4 x float> %a1, <4 x float> %a2, i8 %a3, i32 4)
  ret <4 x float> %r
}

declare <4 x float> @llvm.x86.avx512fp16.maskz.vfmadd.csh(<4 x float>, <4 x float>, <4 x float>, i8, i32)

define i32 @X86.bmi.bextr.32(i32 "reg"="r32" %a0, i32 "reg"="r32" %a1) "enc"="vex(d, 0, a0, 2, 0, a1, 0, 0) | 11110111 | 11 reg:3=d&7 rm:3=a0&7" "asm"="bextrl {a1}, {a0}, {d}" "dclass"="r32" "feature"="bmi" "nodecode" {
  %r = call i32 @llvm.x86.bmi.bextr.32(i32 %a0, i32 %a1)
  ret i32 %r
}

declare i32 @llvm.x86.bmi.bextr.32(i32, i32)

define i64 @X86.bmi.bextr.64(i64 "reg"="r64" %a0, i64 "reg"="r64" %a1) "enc"="vex(d, 0, a0, 2, 1, a1, 0, 0) | 11110111 | 11 reg:3=d&7 rm:3=a0&7" "asm"="bextrq {a1}, {a0}, {d}" "dclass"="r64" "feature"="bmi" "nodecode" {
  %r = call i64 @llvm.x86.bmi.bextr.64(i64 %a0, i64 %a1)
  ret i64 %r
}

declare i64 @llvm.x86.bmi.bextr.64(i64, i64)

define i32 @X86.bmi.bzhi.32(i32 "reg"="r32" %a0, i32 "reg"="r32" %a1) "enc"="vex(d, 0, a0, 2, 0, a1, 0, 0) | 11110101 | 11 reg:3=d&7 rm:3=a0&7" "asm"="bzhil {a1}, {a0}, {d}" "dclass"="r32" "feature"="bmi2" "nodecode" {
  %r = call i32 @llvm.x86.bmi.bzhi.32(i32 %a0, i32 %a1)
  ret i32 %r
}

declare i32 @llvm.x86.bmi.bzhi.32(i32, i32)

define i64 @X86.bmi.bzhi.64(i64 "reg"="r64" %a0, i64 "reg"="r64" %a1) "enc"="vex(d, 0, a0, 2, 1, a1, 0, 0) | 11110101 | 11 reg:3=d&7 rm:3=a0&7" "asm"="bzhiq {a1}, {a0}, {d}" "dclass"="r64" "feature"="bmi2" "nodecode" {
  %r = call i64 @llvm.x86.bmi.bzhi.64(i64 %a0, i64 %a1)
  ret i64 %r
}

declare i64 @llvm.x86.bmi.bzhi.64(i64, i64)

define i32 @X86.bmi.pdep.32(i32 "reg"="r32" %a0, i32 "reg"="r32" %a1) "enc"="vex(d, 0, a1, 2, 0, a0, 0, 3) | 11110101 | 11 reg:3=d&7 rm:3=a1&7" "asm"="pdepl {a1}, {a0}, {d}" "dclass"="r32" "feature"="bmi2" "nodecode" {
  %r = call i32 @llvm.x86.bmi.pdep.32(i32 %a0, i32 %a1)
  ret i32 %r
}

declare i32 @llvm.x86.bmi.pdep.32(i32, i32)

define i64 @X86.bmi.pdep.64(i64 "reg"="r64" %a0, i64 "reg"="r64" %a1) "enc"="vex(d, 0, a1, 2, 1, a0, 0, 3) | 11110101 | 11 reg:3=d&7 rm:3=a1&7" "asm"="pdepq {a1}, {a0}, {d}" "dclass"="r64" "feature"="bmi2" "nodecode" {
  %r = call i64 @llvm.x86.bmi.pdep.64(i64 %a0, i64 %a1)
  ret i64 %r
}

declare i64 @llvm.x86.bmi.pdep.64(i64, i64)

define i32 @X86.bmi.pext.32(i32 "reg"="r32" %a0, i32 "reg"="r32" %a1) "enc"="vex(d, 0, a1, 2, 0, a0, 0, 2) | 11110101 | 11 reg:3=d&7 rm:3=a1&7" "asm"="pextl {a1}, {a0}, {d}" "dclass"="r32" "feature"="bmi2" "nodecode" {
  %r = call i32 @llvm.x86.bmi.pext.32(i32 %a0, i32 %a1)
  ret i32 %r
}

declare i32 @llvm.x86.bmi.pext.32(i32, i32)

define i64 @X86.bmi.pext.64(i64 "reg"="r64" %a0, i64 "reg"="r64" %a1) "enc"="vex(d, 0, a1, 2, 1, a0, 0, 2) | 11110101 | 11 reg:3=d&7 rm:3=a1&7" "asm"="pextq {a1}, {a0}, {d}" "dclass"="r64" "feature"="bmi2" "nodecode" {
  %r = call i64 @llvm.x86.bmi.pext.64(i64 %a0, i64 %a1)
  ret i64 %r
}

declare i64 @llvm.x86.bmi.pext.64(i64, i64)

define void @X86.clflushopt(ptr "reg"="r64" %a0) "enc"="01100110 | rex(0, 0, 4, a0) | 00001111 | 10101110 | rm(7, a0, 4, 1, 0, 0, 1)" "asm"="clflushopt 0({a0})" "feature"="clflushopt" "nodecode" {
  call void @llvm.x86.clflushopt(ptr %a0)
  ret void
}

declare void @llvm.x86.clflushopt(ptr)

define void @X86.ldtilecfg(ptr "reg"="r64" %a0) "enc"="vexm(0, a0, 4, 2, 0, 0, 0, 0) | 01001001 | rm(0, a0, 4, 1, 0, 0, 1)" "asm"="ldtilecfg 0({a0})" "feature"="amx-tile" "nodecode" {
  call void @llvm.x86.ldtilecfg(ptr %a0)
  ret void
}

declare void @llvm.x86.ldtilecfg(ptr)

define void @X86.loadiwkey(<2 x i64> "reg"="xmm0" %a0, <2 x i64> "reg"="xv" %a1, <2 x i64> "reg"="xv" %a2, i32 "reg"="rax" %a3) "enc"="11110011 | rex(0, a1, 0, a2) | 00001111 | 00111000 | 11011100 | 11 reg:3=a1&7 rm:3=a2&7" "asm"="loadiwkey {a2}, {a1}" "feature"="kl" "nodecode" {
  call void @llvm.x86.loadiwkey(<2 x i64> %a0, <2 x i64> %a1, <2 x i64> %a2, i32 %a3)
  ret void
}

declare void @llvm.x86.loadiwkey(<2 x i64>, <2 x i64>, <2 x i64>, i32)

define <2 x i64> @X86.pclmulqdq(<2 x i64> "reg"="xv" "tied" %a0, <2 x i64> "reg"="xv" %a1, i8 "imm" %a2) "enc"="01100110 | rex(0, d, 0, a1) | 00001111 | 00111010 | 01000100 | 11 reg:3=d&7 rm:3=a1&7 | { i1:8=a2 ; i1:s8=a2 }" "asm"="pclmulqdq ${a2}, {a1}, {d}" "dclass"="xv" "feature"="pclmul" "nodecode" {
  %r = call <2 x i64> @llvm.x86.pclmulqdq(<2 x i64> %a0, <2 x i64> %a1, i8 %a2)
  ret <2 x i64> %r
}

declare <2 x i64> @llvm.x86.pclmulqdq(<2 x i64>, <2 x i64>, i8)

define <4 x i64> @X86.pclmulqdq.256(<4 x i64> "reg"="yv" %a0, <4 x i64> "reg"="yv" %a1, i8 "imm" %a2) "enc"="{ vex(d, 0, a1, 3, 0, a0, 1, 1) | 01000100 | 11 reg:3=d&7 rm:3=a1&7 | { i1:8=a2 ; i1:s8=a2 } ; evexr(d, a1, 3, 0, a0, 1, 1, 0, 0, 0) | 01000100 | 11 reg:3=d&7 rm:3=a1&7 | { i1:8=a2 ; i1:s8=a2 } }" "asm"="vpclmulqdq ${a2}, {a1}, {a0}, {d}" "dclass"="yv" "feature"="vpclmulqdq" "nodecode" {
  %r = call <4 x i64> @llvm.x86.pclmulqdq.256(<4 x i64> %a0, <4 x i64> %a1, i8 %a2)
  ret <4 x i64> %r
}

declare <4 x i64> @llvm.x86.pclmulqdq.256(<4 x i64>, <4 x i64>, i8)

define <8 x i64> @X86.pclmulqdq.512(<8 x i64> "reg"="zv" %a0, <8 x i64> "reg"="zv" %a1, i8 "imm" %a2) "enc"="evexr(d, a1, 3, 0, a0, 2, 1, 0, 0, 0) | 01000100 | 11 reg:3=d&7 rm:3=a1&7 | { i1:8=a2 ; i1:s8=a2 }" "asm"="vpclmulqdq ${a2}, {a1}, {a0}, {d}" "dclass"="zv" "feature"="vpclmulqdq,avx512f" "nodecode" {
  %r = call <8 x i64> @llvm.x86.pclmulqdq.512(<8 x i64> %a0, <8 x i64> %a1, i8 %a2)
  ret <8 x i64> %r
}

declare <8 x i64> @llvm.x86.pclmulqdq.512(<8 x i64>, <8 x i64>, i8)

define <4 x i32> @X86.sha1msg1(<4 x i32> "reg"="xv" "tied" %a0, <4 x i32> "reg"="xv" %a1) "enc"="rex(0, d, 0, a1) | 00001111 | 00111000 | 11001001 | 11 reg:3=d&7 rm:3=a1&7" "asm"="sha1msg1 {a1}, {d}" "dclass"="xv" "feature"="sha" "nodecode" {
  %r = call <4 x i32> @llvm.x86.sha1msg1(<4 x i32> %a0, <4 x i32> %a1)
  ret <4 x i32> %r
}

declare <4 x i32> @llvm.x86.sha1msg1(<4 x i32>, <4 x i32>)

define <4 x i32> @X86.sha1msg2(<4 x i32> "reg"="xv" "tied" %a0, <4 x i32> "reg"="xv" %a1) "enc"="rex(0, d, 0, a1) | 00001111 | 00111000 | 11001010 | 11 reg:3=d&7 rm:3=a1&7" "asm"="sha1msg2 {a1}, {d}" "dclass"="xv" "feature"="sha" "nodecode" {
  %r = call <4 x i32> @llvm.x86.sha1msg2(<4 x i32> %a0, <4 x i32> %a1)
  ret <4 x i32> %r
}

declare <4 x i32> @llvm.x86.sha1msg2(<4 x i32>, <4 x i32>)

define <4 x i32> @X86.sha1nexte(<4 x i32> "reg"="xv" "tied" %a0, <4 x i32> "reg"="xv" %a1) "enc"="rex(0, d, 0, a1) | 00001111 | 00111000 | 11001000 | 11 reg:3=d&7 rm:3=a1&7" "asm"="sha1nexte {a1}, {d}" "dclass"="xv" "feature"="sha" "nodecode" {
  %r = call <4 x i32> @llvm.x86.sha1nexte(<4 x i32> %a0, <4 x i32> %a1)
  ret <4 x i32> %r
}

declare <4 x i32> @llvm.x86.sha1nexte(<4 x i32>, <4 x i32>)

define <4 x i32> @X86.sha1rnds4(<4 x i32> "reg"="xv" "tied" %a0, <4 x i32> "reg"="xv" %a1, i8 "imm" %a2) "enc"="rex(0, d, 0, a1) | 00001111 | 00111010 | 11001100 | 11 reg:3=d&7 rm:3=a1&7 | { i1:8=a2 ; i1:s8=a2 }" "asm"="sha1rnds4 ${a2}, {a1}, {d}" "dclass"="xv" "feature"="sha" "nodecode" {
  %r = call <4 x i32> @llvm.x86.sha1rnds4(<4 x i32> %a0, <4 x i32> %a1, i8 %a2)
  ret <4 x i32> %r
}

declare <4 x i32> @llvm.x86.sha1rnds4(<4 x i32>, <4 x i32>, i8)

define <4 x i32> @X86.sha256msg1(<4 x i32> "reg"="xv" "tied" %a0, <4 x i32> "reg"="xv" %a1) "enc"="rex(0, d, 0, a1) | 00001111 | 00111000 | 11001100 | 11 reg:3=d&7 rm:3=a1&7" "asm"="sha256msg1 {a1}, {d}" "dclass"="xv" "feature"="sha" "nodecode" {
  %r = call <4 x i32> @llvm.x86.sha256msg1(<4 x i32> %a0, <4 x i32> %a1)
  ret <4 x i32> %r
}

declare <4 x i32> @llvm.x86.sha256msg1(<4 x i32>, <4 x i32>)

define <4 x i32> @X86.sha256msg2(<4 x i32> "reg"="xv" "tied" %a0, <4 x i32> "reg"="xv" %a1) "enc"="rex(0, d, 0, a1) | 00001111 | 00111000 | 11001101 | 11 reg:3=d&7 rm:3=a1&7" "asm"="sha256msg2 {a1}, {d}" "dclass"="xv" "feature"="sha" "nodecode" {
  %r = call <4 x i32> @llvm.x86.sha256msg2(<4 x i32> %a0, <4 x i32> %a1)
  ret <4 x i32> %r
}

declare <4 x i32> @llvm.x86.sha256msg2(<4 x i32>, <4 x i32>)

define <4 x i32> @X86.sha256rnds2(<4 x i32> "reg"="xv" "tied" %a0, <4 x i32> "reg"="xv" %a1, <4 x i32> "reg"="xmm0" %a2) "enc"="rex(0, d, 0, a1) | 00001111 | 00111000 | 11001011 | 11 reg:3=d&7 rm:3=a1&7" "asm"="sha256rnds2 %xmm0, {a1}, {d}" "dclass"="xv" "feature"="sha" "nodecode" {
  %r = call <4 x i32> @llvm.x86.sha256rnds2(<4 x i32> %a0, <4 x i32> %a1, <4 x i32> %a2)
  ret <4 x i32> %r
}

declare <4 x i32> @llvm.x86.sha256rnds2(<4 x i32>, <4 x i32>, <4 x i32>)

define <4 x float> @X86.sse.cmp.ps(<4 x float> "reg"="xv" "tied" %a0, <4 x float> "reg"="xv" %a1, i8 "imm" %a2) "enc"="rex(0, d, 0, a1) | 00001111 | 11000010 | 11 reg:3=d&7 rm:3=a1&7 | { i1:8=a2 ; i1:s8=a2 }" "asm"="cmpps ${a2}, {a1}, {d}" "dclass"="xv" "feature"="sse" "nodecode" {
  %r = call <4 x float> @llvm.x86.sse.cmp.ps(<4 x float> %a0, <4 x float> %a1, i8 %a2)
  ret <4 x float> %r
}

declare <4 x float> @llvm.x86.sse.cmp.ps(<4 x float>, <4 x float>, i8)

define <4 x float> @X86.sse.cmp.ss(<4 x float> "reg"="xv" "tied" %a0, <4 x float> "reg"="xv" %a1, i8 "imm" %a2) "enc"="11110011 | rex(0, d, 0, a1) | 00001111 | 11000010 | 11 reg:3=d&7 rm:3=a1&7 | { i1:8=a2 ; i1:s8=a2 }" "asm"="cmpss ${a2}, {a1}, {d}" "dclass"="xv" "feature"="sse" "nodecode" {
  %r = call <4 x float> @llvm.x86.sse.cmp.ss(<4 x float> %a0, <4 x float> %a1, i8 %a2)
  ret <4 x float> %r
}

declare <4 x float> @llvm.x86.sse.cmp.ss(<4 x float>, <4 x float>, i8)

define i32 @X86.sse.cvtss2si(<4 x float> "reg"="xv" %a0) "enc"="11110011 | rex(0, d, 0, a0) | 00001111 | 00101101 | 11 reg:3=d&7 rm:3=a0&7" "asm"="cvtss2si {a0}, {d}" "dclass"="r32" "feature"="sse" "nodecode" {
  %r = call i32 @llvm.x86.sse.cvtss2si(<4 x float> %a0)
  ret i32 %r
}

declare i32 @llvm.x86.sse.cvtss2si(<4 x float>)

define i64 @X86.sse.cvtss2si64(<4 x float> "reg"="xv" %a0) "enc"="11110011 | rex(1, d, 0, a0) | 00001111 | 00101101 | 11 reg:3=d&7 rm:3=a0&7" "asm"="cvtss2si {a0}, {d}" "dclass"="r64" "feature"="sse" "nodecode" {
  %r = call i64 @llvm.x86.sse.cvtss2si64(<4 x float> %a0)
  ret i64 %r
}

declare i64 @llvm.x86.sse.cvtss2si64(<4 x float>)

define i32 @X86.sse.cvttss2si(<4 x float> "reg"="xv" %a0) "enc"="11110011 | rex(0, d, 0, a0) | 00001111 | 00101100 | 11 reg:3=d&7 rm:3=a0&7" "asm"="cvttss2si {a0}, {d}" "dclass"="r32" "feature"="sse" "nodecode" {
  %r = call i32 @llvm.x86.sse.cvttss2si(<4 x float> %a0)
  ret i32 %r
}

declare i32 @llvm.x86.sse.cvttss2si(<4 x float>)

define i64 @X86.sse.cvttss2si64(<4 x float> "reg"="xv" %a0) "enc"="11110011 | rex(1, d, 0, a0) | 00001111 | 00101100 | 11 reg:3=d&7 rm:3=a0&7" "asm"="cvttss2si {a0}, {d}" "dclass"="r64" "feature"="sse" "nodecode" {
  %r = call i64 @llvm.x86.sse.cvttss2si64(<4 x float> %a0)
  ret i64 %r
}

declare i64 @llvm.x86.sse.cvttss2si64(<4 x float>)

define void @X86.sse.ldmxcsr(ptr "reg"="r64" %a0) "enc"="rex(0, 0, 4, a0) | 00001111 | 10101110 | rm(2, a0, 4, 1, 0, 0, 1)" "asm"="ldmxcsr 0({a0})" "feature"="sse" "nodecode" {
  call void @llvm.x86.sse.ldmxcsr(ptr %a0)
  ret void
}

declare void @llvm.x86.sse.ldmxcsr(ptr)

define <4 x float> @X86.sse.max.ps(<4 x float> "reg"="xv" "tied" %a0, <4 x float> "reg"="xv" %a1) "enc"="rex(0, d, 0, a1) | 00001111 | 01011111 | 11 reg:3=d&7 rm:3=a1&7" "asm"="maxps {a1}, {d}" "dclass"="xv" "feature"="sse" "nodecode" {
  %r = call <4 x float> @llvm.x86.sse.max.ps(<4 x float> %a0, <4 x float> %a1)
  ret <4 x float> %r
}

declare <4 x float> @llvm.x86.sse.max.ps(<4 x float>, <4 x float>)

define <4 x float> @X86.sse.max.ss(<4 x float> "reg"="xv" "tied" %a0, <4 x float> "reg"="xv" %a1) "enc"="11110011 | rex(0, d, 0, a1) | 00001111 | 01011111 | 11 reg:3=d&7 rm:3=a1&7" "asm"="maxss {a1}, {d}" "dclass"="xv" "feature"="sse" "nodecode" {
  %r = call <4 x float> @llvm.x86.sse.max.ss(<4 x float> %a0, <4 x float> %a1)
  ret <4 x float> %r
}

declare <4 x float> @llvm.x86.sse.max.ss(<4 x float>, <4 x float>)

define <4 x float> @X86.sse.min.ps(<4 x float> "reg"="xv" "tied" %a0, <4 x float> "reg"="xv" %a1) "enc"="rex(0, d, 0, a1) | 00001111 | 01011101 | 11 reg:3=d&7 rm:3=a1&7" "asm"="minps {a1}, {d}" "dclass"="xv" "feature"="sse" "nodecode" {
  %r = call <4 x float> @llvm.x86.sse.min.ps(<4 x float> %a0, <4 x float> %a1)
  ret <4 x float> %r
}

declare <4 x float> @llvm.x86.sse.min.ps(<4 x float>, <4 x float>)

define <4 x float> @X86.sse.min.ss(<4 x float> "reg"="xv" "tied" %a0, <4 x float> "reg"="xv" %a1) "enc"="11110011 | rex(0, d, 0, a1) | 00001111 | 01011101 | 11 reg:3=d&7 rm:3=a1&7" "asm"="minss {a1}, {d}" "dclass"="xv" "feature"="sse" "nodecode" {
  %r = call <4 x float> @llvm.x86.sse.min.ss(<4 x float> %a0, <4 x float> %a1)
  ret <4 x float> %r
}

declare <4 x float> @llvm.x86.sse.min.ss(<4 x float>, <4 x float>)

define <4 x float> @X86.sse.rcp.ps(<4 x float> "reg"="xv" %a0) "enc"="rex(0, d, 0, a0) | 00001111 | 01010011 | 11 reg:3=d&7 rm:3=a0&7" "asm"="rcpps {a0}, {d}" "dclass"="xv" "feature"="sse" "nodecode" {
  %r = call <4 x float> @llvm.x86.sse.rcp.ps(<4 x float> %a0)
  ret <4 x float> %r
}

declare <4 x float> @llvm.x86.sse.rcp.ps(<4 x float>)

define <4 x float> @X86.sse.rcp.ss(<4 x float> "reg"="xv" "tied" %a0) "enc"="11110011 | rex(0, d, 0, a0) | 00001111 | 01010011 | 11 reg:3=d&7 rm:3=a0&7" "asm"="rcpss {a0}, {d}" "dclass"="xv" "feature"="sse" "nodecode" {
  %r = call <4 x float> @llvm.x86.sse.rcp.ss(<4 x float> %a0)
  ret <4 x float> %r
}

declare <4 x float> @llvm.x86.sse.rcp.ss(<4 x float>)

define <4 x float> @X86.sse.rsqrt.ps(<4 x float> "reg"="xv" %a0) "enc"="rex(0, d, 0, a0) | 00001111 | 01010010 | 11 reg:3=d&7 rm:3=a0&7" "asm"="rsqrtps {a0}, {d}" "dclass"="xv" "feature"="sse" "nodecode" {
  %r = call <4 x float> @llvm.x86.sse.rsqrt.ps(<4 x float> %a0)
  ret <4 x float> %r
}

declare <4 x float> @llvm.x86.sse.rsqrt.ps(<4 x float>)

define <4 x float> @X86.sse.rsqrt.ss(<4 x float> "reg"="xv" "tied" %a0) "enc"="11110011 | rex(0, d, 0, a0) | 00001111 | 01010010 | 11 reg:3=d&7 rm:3=a0&7" "asm"="rsqrtss {a0}, {d}" "dclass"="xv" "feature"="sse" "nodecode" {
  %r = call <4 x float> @llvm.x86.sse.rsqrt.ss(<4 x float> %a0)
  ret <4 x float> %r
}

declare <4 x float> @llvm.x86.sse.rsqrt.ss(<4 x float>)

define void @X86.sse.sfence() "enc"="rex(0, 0, 0, 0) | 00001111 | 10101110 | 11 111 000" "asm"="sfence" "feature"="sse" "nodecode" {
  call void @llvm.x86.sse.sfence()
  ret void
}

declare void @llvm.x86.sse.sfence()

define void @X86.sse.stmxcsr(ptr "reg"="r64" %a0) "enc"="rex(0, 0, 4, a0) | 00001111 | 10101110 | rm(3, a0, 4, 1, 0, 0, 1)" "asm"="stmxcsr 0({a0})" "feature"="sse" "nodecode" {
  call void @llvm.x86.sse.stmxcsr(ptr %a0)
  ret void
}

declare void @llvm.x86.sse.stmxcsr(ptr)

define void @X86.sse2.clflush(ptr "reg"="r64" %a0) "enc"="rex(0, 0, 4, a0) | 00001111 | 10101110 | rm(7, a0, 4, 1, 0, 0, 1)" "asm"="clflush 0({a0})" "feature"="sse2" "nodecode" {
  call void @llvm.x86.sse2.clflush(ptr %a0)
  ret void
}

declare void @llvm.x86.sse2.clflush(ptr)

define <2 x double> @X86.sse2.cmp.pd(<2 x double> "reg"="xv" "tied" %a0, <2 x double> "reg"="xv" %a1, i8 "imm" %a2) "enc"="01100110 | rex(0, d, 0, a1) | 00001111 | 11000010 | 11 reg:3=d&7 rm:3=a1&7 | { i1:8=a2 ; i1:s8=a2 }" "asm"="cmppd ${a2}, {a1}, {d}" "dclass"="xv" "feature"="sse2" "nodecode" {
  %r = call <2 x double> @llvm.x86.sse2.cmp.pd(<2 x double> %a0, <2 x double> %a1, i8 %a2)
  ret <2 x double> %r
}

declare <2 x double> @llvm.x86.sse2.cmp.pd(<2 x double>, <2 x double>, i8)

define <2 x double> @X86.sse2.cmp.sd(<2 x double> "reg"="xv" "tied" %a0, <2 x double> "reg"="xv" %a1, i8 "imm" %a2) "enc"="11110010 | rex(0, d, 0, a1) | 00001111 | 11000010 | 11 reg:3=d&7 rm:3=a1&7 | { i1:8=a2 ; i1:s8=a2 }" "asm"="cmpsd ${a2}, {a1}, {d}" "dclass"="xv" "feature"="sse2" "nodecode" {
  %r = call <2 x double> @llvm.x86.sse2.cmp.sd(<2 x double> %a0, <2 x double> %a1, i8 %a2)
  ret <2 x double> %r
}

declare <2 x double> @llvm.x86.sse2.cmp.sd(<2 x double>, <2 x double>, i8)

define <4 x i32> @X86.sse2.cvtpd2dq(<2 x double> "reg"="xv" %a0) "enc"="11110010 | rex(0, d, 0, a0) | 00001111 | 11100110 | 11 reg:3=d&7 rm:3=a0&7" "asm"="cvtpd2dq {a0}, {d}" "dclass"="xv" "feature"="sse2" "nodecode" {
  %r = call <4 x i32> @llvm.x86.sse2.cvtpd2dq(<2 x double> %a0)
  ret <4 x i32> %r
}

declare <4 x i32> @llvm.x86.sse2.cvtpd2dq(<2 x double>)

define <4 x i32> @X86.sse2.cvtps2dq(<4 x float> "reg"="xv" %a0) "enc"="01100110 | rex(0, d, 0, a0) | 00001111 | 01011011 | 11 reg:3=d&7 rm:3=a0&7" "asm"="cvtps2dq {a0}, {d}" "dclass"="xv" "feature"="sse2" "nodecode" {
  %r = call <4 x i32> @llvm.x86.sse2.cvtps2dq(<4 x float> %a0)
  ret <4 x i32> %r
}

declare <4 x i32> @llvm.x86.sse2.cvtps2dq(<4 x float>)

define i32 @X86.sse2.cvtsd2si(<2 x double> "reg"="xv" %a0) "enc"="11110010 | rex(0, d, 0, a0) | 00001111 | 00101101 | 11 reg:3=d&7 rm:3=a0&7" "asm"="cvtsd2si {a0}, {d}" "dclass"="r32" "feature"="sse2" "nodecode" {
  %r = call i32 @llvm.x86.sse2.cvtsd2si(<2 x double> %a0)
  ret i32 %r
}

declare i32 @llvm.x86.sse2.cvtsd2si(<2 x double>)

define i64 @X86.sse2.cvtsd2si64(<2 x double> "reg"="xv" %a0) "enc"="11110010 | rex(1, d, 0, a0) | 00001111 | 00101101 | 11 reg:3=d&7 rm:3=a0&7" "asm"="cvtsd2si {a0}, {d}" "dclass"="r64" "feature"="sse2" "nodecode" {
  %r = call i64 @llvm.x86.sse2.cvtsd2si64(<2 x double> %a0)
  ret i64 %r
}

declare i64 @llvm.x86.sse2.cvtsd2si64(<2 x double>)

define <4 x float> @X86.sse2.cvtsd2ss(<4 x float> "reg"="xv" "tied" %a0, <2 x double> "reg"="xv" %a1) "enc"="11110010 | rex(0, d, 0, a1) | 00001111 | 01011010 | 11 reg:3=d&7 rm:3=a1&7" "asm"="cvtsd2ss {a1}, {d}" "dclass"="xv" "feature"="sse2" "nodecode" {
  %r = call <4 x float> @llvm.x86.sse2.cvtsd2ss(<4 x float> %a0, <2 x double> %a1)
  ret <4 x float> %r
}

declare <4 x float> @llvm.x86.sse2.cvtsd2ss(<4 x float>, <2 x double>)

define <4 x i32> @X86.sse2.cvttpd2dq(<2 x double> "reg"="xv" %a0) "enc"="01100110 | rex(0, d, 0, a0) | 00001111 | 11100110 | 11 reg:3=d&7 rm:3=a0&7" "asm"="cvttpd2dq {a0}, {d}" "dclass"="xv" "feature"="sse2" "nodecode" {
  %r = call <4 x i32> @llvm.x86.sse2.cvttpd2dq(<2 x double> %a0)
  ret <4 x i32> %r
}

declare <4 x i32> @llvm.x86.sse2.cvttpd2dq(<2 x double>)

define <4 x i32> @X86.sse2.cvttps2dq(<4 x float> "reg"="xv" %a0) "enc"="11110011 | rex(0, d, 0, a0) | 00001111 | 01011011 | 11 reg:3=d&7 rm:3=a0&7" "asm"="cvttps2dq {a0}, {d}" "dclass"="xv" "feature"="sse2" "nodecode" {
  %r = call <4 x i32> @llvm.x86.sse2.cvttps2dq(<4 x float> %a0)
  ret <4 x i32> %r
}

declare <4 x i32> @llvm.x86.sse2.cvttps2dq(<4 x float>)

define i32 @X86.sse2.cvttsd2si(<2 x double> "reg"="xv" %a0) "enc"="11110010 | rex(0, d, 0, a0) | 00001111 | 00101100 | 11 reg:3=d&7 rm:3=a0&7" "asm"="cvttsd2si {a0}, {d}" "dclass"="r32" "feature"="sse2" "nodecode" {
  %r = call i32 @llvm.x86.sse2.cvttsd2si(<2 x double> %a0)
  ret i32 %r
}

declare i32 @llvm.x86.sse2.cvttsd2si(<2 x double>)

define i64 @X86.sse2.cvttsd2si64(<2 x double> "reg"="xv" %a0) "enc"="11110010 | rex(1, d, 0, a0) | 00001111 | 00101100 | 11 reg:3=d&7 rm:3=a0&7" "asm"="cvttsd2si {a0}, {d}" "dclass"="r64" "feature"="sse2" "nodecode" {
  %r = call i64 @llvm.x86.sse2.cvttsd2si64(<2 x double> %a0)
  ret i64 %r
}

declare i64 @llvm.x86.sse2.cvttsd2si64(<2 x double>)

define void @X86.sse2.lfence() "enc"="rex(0, 0, 0, 0) | 00001111 | 10101110 | 11 101 000" "asm"="lfence" "feature"="sse2" "nodecode" {
  call void @llvm.x86.sse2.lfence()
  ret void
}

declare void @llvm.x86.sse2.lfence()

define <2 x double> @X86.sse2.max.pd(<2 x double> "reg"="xv" "tied" %a0, <2 x double> "reg"="xv" %a1) "enc"="01100110 | rex(0, d, 0, a1) | 00001111 | 01011111 | 11 reg:3=d&7 rm:3=a1&7" "asm"="maxpd {a1}, {d}" "dclass"="xv" "feature"="sse2" "nodecode" {
  %r = call <2 x double> @llvm.x86.sse2.max.pd(<2 x double> %a0, <2 x double> %a1)
  ret <2 x double> %r
}

declare <2 x double> @llvm.x86.sse2.max.pd(<2 x double>, <2 x double>)

define <2 x double> @X86.sse2.max.sd(<2 x double> "reg"="xv" "tied" %a0, <2 x double> "reg"="xv" %a1) "enc"="11110010 | rex(0, d, 0, a1) | 00001111 | 01011111 | 11 reg:3=d&7 rm:3=a1&7" "asm"="maxsd {a1}, {d}" "dclass"="xv" "feature"="sse2" "nodecode" {
  %r = call <2 x double> @llvm.x86.sse2.max.sd(<2 x double> %a0, <2 x double> %a1)
  ret <2 x double> %r
}

declare <2 x double> @llvm.x86.sse2.max.sd(<2 x double>, <2 x double>)

define void @X86.sse2.mfence() "enc"="rex(0, 0, 0, 0) | 00001111 | 10101110 | 11 110 000" "asm"="mfence" "feature"="sse2" "nodecode" {
  call void @llvm.x86.sse2.mfence()
  ret void
}

declare void @llvm.x86.sse2.mfence()

define <2 x double> @X86.sse2.min.pd(<2 x double> "reg"="xv" "tied" %a0, <2 x double> "reg"="xv" %a1) "enc"="01100110 | rex(0, d, 0, a1) | 00001111 | 01011101 | 11 reg:3=d&7 rm:3=a1&7" "asm"="minpd {a1}, {d}" "dclass"="xv" "feature"="sse2" "nodecode" {
  %r = call <2 x double> @llvm.x86.sse2.min.pd(<2 x double> %a0, <2 x double> %a1)
  ret <2 x double> %r
}

declare <2 x double> @llvm.x86.sse2.min.pd(<2 x double>, <2 x double>)

define <2 x double> @X86.sse2.min.sd(<2 x double> "reg"="xv" "tied" %a0, <2 x double> "reg"="xv" %a1) "enc"="11110010 | rex(0, d, 0, a1) | 00001111 | 01011101 | 11 reg:3=d&7 rm:3=a1&7" "asm"="minsd {a1}, {d}" "dclass"="xv" "feature"="sse2" "nodecode" {
  %r = call <2 x double> @llvm.x86.sse2.min.sd(<2 x double> %a0, <2 x double> %a1)
  ret <2 x double> %r
}

declare <2 x double> @llvm.x86.sse2.min.sd(<2 x double>, <2 x double>)

define <8 x i16> @X86.sse2.packssdw.128(<4 x i32> "reg"="xv" "tied" %a0, <4 x i32> "reg"="xv" %a1) "enc"="01100110 | rex(0, d, 0, a1) | 00001111 | 01101011 | 11 reg:3=d&7 rm:3=a1&7" "asm"="packssdw {a1}, {d}" "dclass"="xv" "feature"="sse2" "nodecode" {
  %r = call <8 x i16> @llvm.x86.sse2.packssdw.128(<4 x i32> %a0, <4 x i32> %a1)
  ret <8 x i16> %r
}

declare <8 x i16> @llvm.x86.sse2.packssdw.128(<4 x i32>, <4 x i32>)

define <16 x i8> @X86.sse2.packsswb.128(<8 x i16> "reg"="xv" "tied" %a0, <8 x i16> "reg"="xv" %a1) "enc"="01100110 | rex(0, d, 0, a1) | 00001111 | 01100011 | 11 reg:3=d&7 rm:3=a1&7" "asm"="packsswb {a1}, {d}" "dclass"="xv" "feature"="sse2" "nodecode" {
  %r = call <16 x i8> @llvm.x86.sse2.packsswb.128(<8 x i16> %a0, <8 x i16> %a1)
  ret <16 x i8> %r
}

declare <16 x i8> @llvm.x86.sse2.packsswb.128(<8 x i16>, <8 x i16>)

define <16 x i8> @X86.sse2.packuswb.128(<8 x i16> "reg"="xv" "tied" %a0, <8 x i16> "reg"="xv" %a1) "enc"="01100110 | rex(0, d, 0, a1) | 00001111 | 01100111 | 11 reg:3=d&7 rm:3=a1&7" "asm"="packuswb {a1}, {d}" "dclass"="xv" "feature"="sse2" "nodecode" {
  %r = call <16 x i8> @llvm.x86.sse2.packuswb.128(<8 x i16> %a0, <8 x i16> %a1)
  ret <16 x i8> %r
}

declare <16 x i8> @llvm.x86.sse2.packuswb.128(<8 x i16>, <8 x i16>)

define <4 x i32> @X86.sse2.pmadd.wd(<8 x i16> "reg"="xv" "tied" %a0, <8 x i16> "reg"="xv" %a1) "enc"="01100110 | rex(0, d, 0, a1) | 00001111 | 11110101 | 11 reg:3=d&7 rm:3=a1&7" "asm"="pmaddwd {a1}, {d}" "dclass"="xv" "feature"="sse2" "nodecode" {
  %r = call <4 x i32> @llvm.x86.sse2.pmadd.wd(<8 x i16> %a0, <8 x i16> %a1)
  ret <4 x i32> %r
}

declare <4 x i32> @llvm.x86.sse2.pmadd.wd(<8 x i16>, <8 x i16>)

define <2 x i64> @X86.sse2.psad.bw(<16 x i8> "reg"="xv" "tied" %a0, <16 x i8> "reg"="xv" %a1) "enc"="01100110 | rex(0, d, 0, a1) | 00001111 | 11110110 | 11 reg:3=d&7 rm:3=a1&7" "asm"="psadbw {a1}, {d}" "dclass"="xv" "feature"="sse2" "nodecode" {
  %r = call <2 x i64> @llvm.x86.sse2.psad.bw(<16 x i8> %a0, <16 x i8> %a1)
  ret <2 x i64> %r
}

declare <2 x i64> @llvm.x86.sse2.psad.bw(<16 x i8>, <16 x i8>)

define <4 x i32> @X86.sse2.psll.d(<4 x i32> "reg"="xv" "tied" %a0, <4 x i32> "reg"="xv" %a1) "enc"="01100110 | rex(0, d, 0, a1) | 00001111 | 11110010 | 11 reg:3=d&7 rm:3=a1&7" "asm"="pslld {a1}, {d}" "dclass"="xv" "feature"="sse2" "nodecode" {
  %r = call <4 x i32> @llvm.x86.sse2.psll.d(<4 x i32> %a0, <4 x i32> %a1)
  ret <4 x i32> %r
}

declare <4 x i32> @llvm.x86.sse2.psll.d(<4 x i32>, <4 x i32>)

define <2 x i64> @X86.sse2.psll.q(<2 x i64> "reg"="xv" "tied" %a0, <2 x i64> "reg"="xv" %a1) "enc"="01100110 | rex(0, d, 0, a1) | 00001111 | 11110011 | 11 reg:3=d&7 rm:3=a1&7" "asm"="psllq {a1}, {d}" "dclass"="xv" "feature"="sse2" "nodecode" {
  %r = call <2 x i64> @llvm.x86.sse2.psll.q(<2 x i64> %a0, <2 x i64> %a1)
  ret <2 x i64> %r
}

declare <2 x i64> @llvm.x86.sse2.psll.q(<2 x i64>, <2 x i64>)

define <8 x i16> @X86.sse2.psll.w(<8 x i16> "reg"="xv" "tied" %a0, <8 x i16> "reg"="xv" %a1) "enc"="01100110 | rex(0, d, 0, a1) | 00001111 | 11110001 | 11 reg:3=d&7 rm:3=a1&7" "asm"="psllw {a1}, {d}" "dclass"="xv" "feature"="sse2" "nodecode" {
  %r = call <8 x i16> @llvm.x86.sse2.psll.w(<8 x i16> %a0, <8 x i16> %a1)
  ret <8 x i16> %r
}

declare <8 x i16> @llvm.x86.sse2.psll.w(<8 x i16>, <8 x i16>)

define <4 x i32> @X86.sse2.psra.d(<4 x i32> "reg"="xv" "tied" %a0, <4 x i32> "reg"="xv" %a1) "enc"="01100110 | rex(0, d, 0, a1) | 00001111 | 11100010 | 11 reg:3=d&7 rm:3=a1&7" "asm"="psrad {a1}, {d}" "dclass"="xv" "feature"="sse2" "nodecode" {
  %r = call <4 x i32> @llvm.x86.sse2.psra.d(<4 x i32> %a0, <4 x i32> %a1)
  ret <4 x i32> %r
}

declare <4 x i32> @llvm.x86.sse2.psra.d(<4 x i32>, <4 x i32>)

define <8 x i16> @X86.sse2.psra.w(<8 x i16> "reg"="xv" "tied" %a0, <8 x i16> "reg"="xv" %a1) "enc"="01100110 | rex(0, d, 0, a1) | 00001111 | 11100001 | 11 reg:3=d&7 rm:3=a1&7" "asm"="psraw {a1}, {d}" "dclass"="xv" "feature"="sse2" "nodecode" {
  %r = call <8 x i16> @llvm.x86.sse2.psra.w(<8 x i16> %a0, <8 x i16> %a1)
  ret <8 x i16> %r
}

declare <8 x i16> @llvm.x86.sse2.psra.w(<8 x i16>, <8 x i16>)

define <4 x i32> @X86.sse2.psrl.d(<4 x i32> "reg"="xv" "tied" %a0, <4 x i32> "reg"="xv" %a1) "enc"="01100110 | rex(0, d, 0, a1) | 00001111 | 11010010 | 11 reg:3=d&7 rm:3=a1&7" "asm"="psrld {a1}, {d}" "dclass"="xv" "feature"="sse2" "nodecode" {
  %r = call <4 x i32> @llvm.x86.sse2.psrl.d(<4 x i32> %a0, <4 x i32> %a1)
  ret <4 x i32> %r
}

declare <4 x i32> @llvm.x86.sse2.psrl.d(<4 x i32>, <4 x i32>)

define <2 x i64> @X86.sse2.psrl.q(<2 x i64> "reg"="xv" "tied" %a0, <2 x i64> "reg"="xv" %a1) "enc"="01100110 | rex(0, d, 0, a1) | 00001111 | 11010011 | 11 reg:3=d&7 rm:3=a1&7" "asm"="psrlq {a1}, {d}" "dclass"="xv" "feature"="sse2" "nodecode" {
  %r = call <2 x i64> @llvm.x86.sse2.psrl.q(<2 x i64> %a0, <2 x i64> %a1)
  ret <2 x i64> %r
}

declare <2 x i64> @llvm.x86.sse2.psrl.q(<2 x i64>, <2 x i64>)

define <8 x i16> @X86.sse2.psrl.w(<8 x i16> "reg"="xv" "tied" %a0, <8 x i16> "reg"="xv" %a1) "enc"="01100110 | rex(0, d, 0, a1) | 00001111 | 11010001 | 11 reg:3=d&7 rm:3=a1&7" "asm"="psrlw {a1}, {d}" "dclass"="xv" "feature"="sse2" "nodecode" {
  %r = call <8 x i16> @llvm.x86.sse2.psrl.w(<8 x i16> %a0, <8 x i16> %a1)
  ret <8 x i16> %r
}

declare <8 x i16> @llvm.x86.sse2.psrl.w(<8 x i16>, <8 x i16>)

define <16 x i8> @X86.sse3.ldu.dq(ptr "reg"="r64" %a0) "enc"="11110010 | rex(0, d, 4, a0) | 00001111 | 11110000 | rm(d, a0, 4, 1, 0, 0, 1)" "asm"="lddqu 0({a0}), {d}" "dclass"="xv" "feature"="sse3" "nodecode" {
  %r = call <16 x i8> @llvm.x86.sse3.ldu.dq(ptr %a0)
  ret <16 x i8> %r
}

declare <16 x i8> @llvm.x86.sse3.ldu.dq(ptr)

define <2 x double> @X86.sse41.dppd(<2 x double> "reg"="xv" "tied" %a0, <2 x double> "reg"="xv" %a1, i8 "imm" %a2) "enc"="01100110 | rex(0, d, 0, a1) | 00001111 | 00111010 | 01000001 | 11 reg:3=d&7 rm:3=a1&7 | { i1:8=a2 ; i1:s8=a2 }" "asm"="dppd ${a2}, {a1}, {d}" "dclass"="xv" "feature"="sse4.1" "nodecode" {
  %r = call <2 x double> @llvm.x86.sse41.dppd(<2 x double> %a0, <2 x double> %a1, i8 %a2)
  ret <2 x double> %r
}

declare <2 x double> @llvm.x86.sse41.dppd(<2 x double>, <2 x double>, i8)

define <4 x float> @X86.sse41.dpps(<4 x float> "reg"="xv" "tied" %a0, <4 x float> "reg"="xv" %a1, i8 "imm" %a2) "enc"="01100110 | rex(0, d, 0, a1) | 00001111 | 00111010 | 01000000 | 11 reg:3=d&7 rm:3=a1&7 | { i1:8=a2 ; i1:s8=a2 }" "asm"="dpps ${a2}, {a1}, {d}" "dclass"="xv" "feature"="sse4.1" "nodecode" {
  %r = call <4 x float> @llvm.x86.sse41.dpps(<4 x float> %a0, <4 x float> %a1, i8 %a2)
  ret <4 x float> %r
}

declare <4 x float> @llvm.x86.sse41.dpps(<4 x float>, <4 x float>, i8)

define <4 x float> @X86.sse41.insertps(<4 x float> "reg"="xv" "tied" %a0, <4 x float> "reg"="xv" %a1, i8 "imm" %a2) "enc"="01100110 | rex(0, d, 0, a1) | 00001111 | 00111010 | 00100001 | 11 reg:3=d&7 rm:3=a1&7 | { i1:8=a2 ; i1:s8=a2 }" "asm"="insertps ${a2}, {a1}, {d}" "dclass"="xv" "feature"="sse4.1" "nodecode" {
  %r = call <4 x float> @llvm.x86.sse41.insertps(<4 x float> %a0, <4 x float> %a1, i8 %a2)
  ret <4 x float> %r
}

declare <4 x float> @llvm.x86.sse41.insertps(<4 x float>, <4 x float>, i8)

define <8 x i16> @X86.sse41.mpsadbw(<16 x i8> "reg"="xv" "tied" %a0, <16 x i8> "reg"="xv" %a1, i8 "imm" %a2) "enc"="01100110 | rex(0, d, 0, a1) | 00001111 | 00111010 | 01000010 | 11 reg:3=d&7 rm:3=a1&7 | { i1:8=a2 ; i1:s8=a2 }" "asm"="mpsadbw ${a2}, {a1}, {d}" "dclass"="xv" "feature"="sse4.1" "nodecode" {
  %r = call <8 x i16> @llvm.x86.sse41.mpsadbw(<16 x i8> %a0, <16 x i8> %a1, i8 %a2)
  ret <8 x i16> %r
}

declare <8 x i16> @llvm.x86.sse41.mpsadbw(<16 x i8>, <16 x i8>, i8)

define <8 x i16> @X86.sse41.packusdw(<4 x i32> "reg"="xv" "tied" %a0, <4 x i32> "reg"="xv" %a1) "enc"="01100110 | rex(0, d, 0, a1) | 00001111 | 00111000 | 00101011 | 11 reg:3=d&7 rm:3=a1&7" "asm"="packusdw {a1}, {d}" "dclass"="xv" "feature"="sse4.1" "nodecode" {
  %r = call <8 x i16> @llvm.x86.sse41.packusdw(<4 x i32> %a0, <4 x i32> %a1)
  ret <8 x i16> %r
}

declare <8 x i16> @llvm.x86.sse41.packusdw(<4 x i32>, <4 x i32>)

define <8 x i16> @X86.sse41.phminposuw(<8 x i16> "reg"="xv" %a0) "enc"="01100110 | rex(0, d, 0, a0) | 00001111 | 00111000 | 01000001 | 11 reg:3=d&7 rm:3=a0&7" "asm"="phminposuw {a0}, {d}" "dclass"="xv" "feature"="sse4.1" "nodecode" {
  %r = call <8 x i16> @llvm.x86.sse41.phminposuw(<8 x i16> %a0)
  ret <8 x i16> %r
}

declare <8 x i16> @llvm.x86.sse41.phminposuw(<8 x i16>)

define <2 x double> @X86.sse41.round.pd(<2 x double> "reg"="xv" %a0, i32 "imm" %a1) "enc"="01100110 | rex(0, d, 0, a0) | 00001111 | 00111010 | 00001001 | 11 reg:3=d&7 rm:3=a0&7 | { i1:8=a1 ; i1:s8=a1 }" "asm"="roundpd ${a1}, {a0}, {d}" "dclass"="xv" "feature"="sse4.1" "nodecode" {
  %r = call <2 x double> @llvm.x86.sse41.round.pd(<2 x double> %a0, i32 %a1)
  ret <2 x double> %r
}

declare <2 x double> @llvm.x86.sse41.round.pd(<2 x double>, i32)

define <4 x float> @X86.sse41.round.ps(<4 x float> "reg"="xv" %a0, i32 "imm" %a1) "enc"="01100110 | rex(0, d, 0, a0) | 00001111 | 00111010 | 00001000 | 11 reg:3=d&7 rm:3=a0&7 | { i1:8=a1 ; i1:s8=a1 }" "asm"="roundps ${a1}, {a0}, {d}" "dclass"="xv" "feature"="sse4.1" "nodecode" {
  %r = call <4 x float> @llvm.x86.sse41.round.ps(<4 x float> %a0, i32 %a1)
  ret <4 x float> %r
}

declare <4 x float> @llvm.x86.sse41.round.ps(<4 x float>, i32)

define <2 x double> @X86.sse41.round.sd(<2 x double> "reg"="xv" "tied" %a0, <2 x double> "reg"="xv" %a1, i32 "imm" %a2) "enc"="01100110 | rex(0, d, 0, a1) | 00001111 | 00111010 | 00001011 | 11 reg:3=d&7 rm:3=a1&7 | { i1:8=a2 ; i1:s8=a2 }" "asm"="roundsd ${a2}, {a1}, {d}" "dclass"="xv" "feature"="sse4.1" "nodecode" {
  %r = call <2 x double> @llvm.x86.sse41.round.sd(<2 x double> %a0, <2 x double> %a1, i32 %a2)
  ret <2 x double> %r
}

declare <2 x double> @llvm.x86.sse41.round.sd(<2 x double>, <2 x double>, i32)

define <4 x float> @X86.sse41.round.ss(<4 x float> "reg"="xv" "tied" %a0, <4 x float> "reg"="xv" %a1, i32 "imm" %a2) "enc"="01100110 | rex(0, d, 0, a1) | 00001111 | 00111010 | 00001010 | 11 reg:3=d&7 rm:3=a1&7 | { i1:8=a2 ; i1:s8=a2 }" "asm"="roundss ${a2}, {a1}, {d}" "dclass"="xv" "feature"="sse4.1" "nodecode" {
  %r = call <4 x float> @llvm.x86.sse41.round.ss(<4 x float> %a0, <4 x float> %a1, i32 %a2)
  ret <4 x float> %r
}

declare <4 x float> @llvm.x86.sse41.round.ss(<4 x float>, <4 x float>, i32)

define i32 @X86.sse42.pcmpestri128(<16 x i8> "reg"="xv" %a0, i32 "reg"="rax" %a1, <16 x i8> "reg"="xv" %a2, i32 "reg"="rdx" %a3, i8 "imm" %a4) "enc"="{ 01100110 | rex(0, a0, 0, a2) | 00001111 | 00111010 | 01100001 | 11 reg:3=a0&7 rm:3=a2&7 | { i1:8=a4 ; i1:s8=a4 } ; 01100110 | rex(1, a0, 0, a2) | 00001111 | 00111010 | 01100001 | 11 reg:3=a0&7 rm:3=a2&7 | { i1:8=a4 ; i1:s8=a4 } }" "asm"="pcmpestri ${a4}, {a2}, {a0}" "dclass"="rcx" "feature"="sse4.2" "nodecode" {
  %r = call i32 @llvm.x86.sse42.pcmpestri128(<16 x i8> %a0, i32 %a1, <16 x i8> %a2, i32 %a3, i8 %a4)
  ret i32 %r
}

declare i32 @llvm.x86.sse42.pcmpestri128(<16 x i8>, i32, <16 x i8>, i32, i8)

define <16 x i8> @X86.sse42.pcmpestrm128(<16 x i8> "reg"="xv" %a0, i32 "reg"="rax" %a1, <16 x i8> "reg"="xv" %a2, i32 "reg"="rdx" %a3, i8 "imm" %a4) "enc"="{ 01100110 | rex(0, a0, 0, a2) | 00001111 | 00111010 | 01100000 | 11 reg:3=a0&7 rm:3=a2&7 | { i1:8=a4 ; i1:s8=a4 } ; 01100110 | rex(1, a0, 0, a2) | 00001111 | 00111010 | 01100000 | 11 reg:3=a0&7 rm:3=a2&7 | { i1:8=a4 ; i1:s8=a4 } }" "asm"="pcmpestrm ${a4}, {a2}, {a0}" "dclass"="xmm0" "feature"="sse4.2" "nodecode" {
  %r = call <16 x i8> @llvm.x86.sse42.pcmpestrm128(<16 x i8> %a0, i32 %a1, <16 x i8> %a2, i32 %a3, i8 %a4)
  ret <16 x i8> %r
}

declare <16 x i8> @llvm.x86.sse42.pcmpestrm128(<16 x i8>, i32, <16 x i8>, i32, i8)

define i32 @X86.sse42.pcmpistri128(<16 x i8> "reg"="xv" %a0, <16 x i8> "reg"="xv" %a1, i8 "imm" %a2) "enc"="{ 01100110 | rex(0, a0, 0, a1) | 00001111 | 00111010 | 01100011 | 11 reg:3=a0&7 rm:3=a1&7 | { i1:8=a2 ; i1:s8=a2 } ; 01100110 | rex(1, a0, 0, a1) | 00001111 | 00111010 | 01100011 | 11 reg:3=a0&7 rm:3=a1&7 | { i1:8=a2 ; i1:s8=a2 } }" "asm"="pcmpistri ${a2}, {a1}, {a0}" "dclass"="rcx" "feature"="sse4.2" "nodecode" {
  %r = call i32 @llvm.x86.sse42.pcmpistri128(<16 x i8> %a0, <16 x i8> %a1, i8 %a2)
  ret i32 %r
}

declare i32 @llvm.x86.sse42.pcmpistri128(<16 x i8>, <16 x i8>, i8)

define <16 x i8> @X86.sse42.pcmpistrm128(<16 x i8> "reg"="xv" %a0, <16 x i8> "reg"="xv" %a1, i8 "imm" %a2) "enc"="01100110 | rex(0, a0, 0, a1) | 00001111 | 00111010 | 01100010 | 11 reg:3=a0&7 rm:3=a1&7 | { i1:8=a2 ; i1:s8=a2 }" "asm"="pcmpistrm ${a2}, {a1}, {a0}" "dclass"="xmm0" "feature"="sse4.2" "nodecode" {
  %r = call <16 x i8> @llvm.x86.sse42.pcmpistrm128(<16 x i8> %a0, <16 x i8> %a1, i8 %a2)
  ret <16 x i8> %r
}

declare <16 x i8> @llvm.x86.sse42.pcmpistrm128(<16 x i8>, <16 x i8>, i8)

define <8 x i16> @X86.ssse3.pmadd.ub.sw.128(<16 x i8> "reg"="xv" "tied" %a0, <16 x i8> "reg"="xv" %a1) "enc"="01100110 | rex(0, d, 0, a1) | 00001111 | 00111000 | 00000100 | 11 reg:3=d&7 rm:3=a1&7" "asm"="pmaddubsw {a1}, {d}" "dclass"="xv" "feature"="ssse3" "nodecode" {
  %r = call <8 x i16> @llvm.x86.ssse3.pmadd.ub.sw.128(<16 x i8> %a0, <16 x i8> %a1)
  ret <8 x i16> %r
}

declare <8 x i16> @llvm.x86.ssse3.pmadd.ub.sw.128(<16 x i8>, <16 x i8>)

define <8 x i16> @X86.ssse3.pmul.hr.sw.128(<8 x i16> "reg"="xv" "tied" %a0, <8 x i16> "reg"="xv" %a1) "enc"="01100110 | rex(0, d, 0, a1) | 00001111 | 00111000 | 00001011 | 11 reg:3=d&7 rm:3=a1&7" "asm"="pmulhrsw {a1}, {d}" "dclass"="xv" "feature"="ssse3" "nodecode" {
  %r = call <8 x i16> @llvm.x86.ssse3.pmul.hr.sw.128(<8 x i16> %a0, <8 x i16> %a1)
  ret <8 x i16> %r
}

declare <8 x i16> @llvm.x86.ssse3.pmul.hr.sw.128(<8 x i16>, <8 x i16>)

define <16 x i8> @X86.ssse3.pshuf.b.128(<16 x i8> "reg"="xv" "tied" %a0, <16 x i8> "reg"="xv" %a1) "enc"="01100110 | rex(0, d, 0, a1) | 00001111 | 00111000 | 00000000 | 11 reg:3=d&7 rm:3=a1&7" "asm"="pshufb {a1}, {d}" "dclass"="xv" "feature"="ssse3" "nodecode" {
  %r = call <16 x i8> @llvm.x86.ssse3.pshuf.b.128(<16 x i8> %a0, <16 x i8> %a1)
  ret <16 x i8> %r
}

declare <16 x i8> @llvm.x86.ssse3.pshuf.b.128(<16 x i8>, <16 x i8>)

define <16 x i8> @X86.ssse3.psign.b.128(<16 x i8> "reg"="xv" "tied" %a0, <16 x i8> "reg"="xv" %a1) "enc"="01100110 | rex(0, d, 0, a1) | 00001111 | 00111000 | 00001000 | 11 reg:3=d&7 rm:3=a1&7" "asm"="psignb {a1}, {d}" "dclass"="xv" "feature"="ssse3" "nodecode" {
  %r = call <16 x i8> @llvm.x86.ssse3.psign.b.128(<16 x i8> %a0, <16 x i8> %a1)
  ret <16 x i8> %r
}

declare <16 x i8> @llvm.x86.ssse3.psign.b.128(<16 x i8>, <16 x i8>)

define <4 x i32> @X86.ssse3.psign.d.128(<4 x i32> "reg"="xv" "tied" %a0, <4 x i32> "reg"="xv" %a1) "enc"="01100110 | rex(0, d, 0, a1) | 00001111 | 00111000 | 00001010 | 11 reg:3=d&7 rm:3=a1&7" "asm"="psignd {a1}, {d}" "dclass"="xv" "feature"="ssse3" "nodecode" {
  %r = call <4 x i32> @llvm.x86.ssse3.psign.d.128(<4 x i32> %a0, <4 x i32> %a1)
  ret <4 x i32> %r
}

declare <4 x i32> @llvm.x86.ssse3.psign.d.128(<4 x i32>, <4 x i32>)

define <8 x i16> @X86.ssse3.psign.w.128(<8 x i16> "reg"="xv" "tied" %a0, <8 x i16> "reg"="xv" %a1) "enc"="01100110 | rex(0, d, 0, a1) | 00001111 | 00111000 | 00001001 | 11 reg:3=d&7 rm:3=a1&7" "asm"="psignw {a1}, {d}" "dclass"="xv" "feature"="ssse3" "nodecode" {
  %r = call <8 x i16> @llvm.x86.ssse3.psign.w.128(<8 x i16> %a0, <8 x i16> %a1)
  ret <8 x i16> %r
}

declare <8 x i16> @llvm.x86.ssse3.psign.w.128(<8 x i16>, <8 x i16>)

define void @X86.sttilecfg(ptr "reg"="r64" %a0) "enc"="vexm(0, a0, 4, 2, 0, 0, 0, 1) | 01001001 | rm(0, a0, 4, 1, 0, 0, 1)" "asm"="sttilecfg 0({a0})" "feature"="amx-tile" "nodecode" {
  call void @llvm.x86.sttilecfg(ptr %a0)
  ret void
}

declare void @llvm.x86.sttilecfg(ptr)

define void @X86.tilerelease() "enc"="vex(0, 0, 0, 2, 0, 0, 0, 0) | 01001001 | 11 000 000" "asm"="tilerelease" "feature"="amx-tile" "nodecode" {
  call void @llvm.x86.tilerelease()
  ret void
}

declare void @llvm.x86.tilerelease()

define <4 x float> @X86.vcvtneebf162ps128(ptr "reg"="r64" %a0) "enc"="vexm(d, a0, 4, 2, 0, 0, 0, 2) | 10110000 | rm(d, a0, 4, 1, 0, 0, 1)" "asm"="vcvtneebf162ps 0({a0}), {d}" "dclass"="xv" "feature"="avxneconvert" "nodecode" {
  %r = call <4 x float> @llvm.x86.vcvtneebf162ps128(ptr %a0)
  ret <4 x float> %r
}

declare <4 x float> @llvm.x86.vcvtneebf162ps128(ptr)

define <8 x float> @X86.vcvtneebf162ps256(ptr "reg"="r64" %a0) "enc"="vexm(d, a0, 4, 2, 0, 0, 1, 2) | 10110000 | rm(d, a0, 4, 1, 0, 0, 1)" "asm"="vcvtneebf162ps 0({a0}), {d}" "dclass"="yv" "feature"="avxneconvert" "nodecode" {
  %r = call <8 x float> @llvm.x86.vcvtneebf162ps256(ptr %a0)
  ret <8 x float> %r
}

declare <8 x float> @llvm.x86.vcvtneebf162ps256(ptr)

define <4 x float> @X86.vcvtneeph2ps128(ptr "reg"="r64" %a0) "enc"="vexm(d, a0, 4, 2, 0, 0, 0, 1) | 10110000 | rm(d, a0, 4, 1, 0, 0, 1)" "asm"="vcvtneeph2ps 0({a0}), {d}" "dclass"="xv" "feature"="avxneconvert" "nodecode" {
  %r = call <4 x float> @llvm.x86.vcvtneeph2ps128(ptr %a0)
  ret <4 x float> %r
}

declare <4 x float> @llvm.x86.vcvtneeph2ps128(ptr)

define <8 x float> @X86.vcvtneeph2ps256(ptr "reg"="r64" %a0) "enc"="vexm(d, a0, 4, 2, 0, 0, 1, 1) | 10110000 | rm(d, a0, 4, 1, 0, 0, 1)" "asm"="vcvtneeph2ps 0({a0}), {d}" "dclass"="yv" "feature"="avxneconvert" "nodecode" {
  %r = call <8 x float> @llvm.x86.vcvtneeph2ps256(ptr %a0)
  ret <8 x float> %r
}

declare <8 x float> @llvm.x86.vcvtneeph2ps256(ptr)

define <4 x float> @X86.vcvtneobf162ps128(ptr "reg"="r64" %a0) "enc"="vexm(d, a0, 4, 2, 0, 0, 0, 3) | 10110000 | rm(d, a0, 4, 1, 0, 0, 1)" "asm"="vcvtneobf162ps 0({a0}), {d}" "dclass"="xv" "feature"="avxneconvert" "nodecode" {
  %r = call <4 x float> @llvm.x86.vcvtneobf162ps128(ptr %a0)
  ret <4 x float> %r
}

declare <4 x float> @llvm.x86.vcvtneobf162ps128(ptr)

define <8 x float> @X86.vcvtneobf162ps256(ptr "reg"="r64" %a0) "enc"="vexm(d, a0, 4, 2, 0, 0, 1, 3) | 10110000 | rm(d, a0, 4, 1, 0, 0, 1)" "asm"="vcvtneobf162ps 0({a0}), {d}" "dclass"="yv" "feature"="avxneconvert" "nodecode" {
  %r = call <8 x float> @llvm.x86.vcvtneobf162ps256(ptr %a0)
  ret <8 x float> %r
}

declare <8 x float> @llvm.x86.vcvtneobf162ps256(ptr)

define <4 x float> @X86.vcvtneoph2ps128(ptr "reg"="r64" %a0) "enc"="vexm(d, a0, 4, 2, 0, 0, 0, 0) | 10110000 | rm(d, a0, 4, 1, 0, 0, 1)" "asm"="vcvtneoph2ps 0({a0}), {d}" "dclass"="xv" "feature"="avxneconvert" "nodecode" {
  %r = call <4 x float> @llvm.x86.vcvtneoph2ps128(ptr %a0)
  ret <4 x float> %r
}

declare <4 x float> @llvm.x86.vcvtneoph2ps128(ptr)

define <8 x float> @X86.vcvtneoph2ps256(ptr "reg"="r64" %a0) "enc"="vexm(d, a0, 4, 2, 0, 0, 1, 0) | 10110000 | rm(d, a0, 4, 1, 0, 0, 1)" "asm"="vcvtneoph2ps 0({a0}), {d}" "dclass"="yv" "feature"="avxneconvert" "nodecode" {
  %r = call <8 x float> @llvm.x86.vcvtneoph2ps256(ptr %a0)
  ret <8 x float> %r
}

declare <8 x float> @llvm.x86.vcvtneoph2ps256(ptr)

define <8 x bfloat> @X86.vcvtneps2bf16128(<4 x float> "reg"="xv" %a0) "enc"="evexr(d, a0, 2, 0, 0, 0, 2, 0, 0, 0) | 01110010 | 11 reg:3=d&7 rm:3=a0&7" "asm"="vcvtneps2bf16 {a0}, {d}" "dclass"="xv" "feature"="avxneconvert" "nodecode" {
  %r = call <8 x bfloat> @llvm.x86.vcvtneps2bf16128(<4 x float> %a0)
  ret <8 x bfloat> %r
}

declare <8 x bfloat> @llvm.x86.vcvtneps2bf16128(<4 x float>)

define <8 x bfloat> @X86.vcvtneps2bf16256(<8 x float> "reg"="yv" %a0) "enc"="evexr(d, a0, 2, 0, 0, 1, 2, 0, 0, 0) | 01110010 | 11 reg:3=d&7 rm:3=a0&7" "asm"="vcvtneps2bf16 {a0}, {d}" "dclass"="xv" "feature"="avxneconvert" "nodecode" {
  %r = call <8 x bfloat> @llvm.x86.vcvtneps2bf16256(<8 x float> %a0)
  ret <8 x bfloat> %r
}

declare <8 x bfloat> @llvm.x86.vcvtneps2bf16256(<8 x float>)

define <8 x i16> @X86.vcvtps2ph.128(<4 x float> "reg"="xv" %a0, i32 "imm" %a1) "enc"="{ vex(a0, 0, d, 3, 0, 0, 0, 1) | 00011101 | 11 reg:3=a0&7 rm:3=d&7 | { i1:8=a1 ; i1:s8=a1 } ; evexr(a0, d, 3, 0, 0, 0, 1, 0, 0, 0) | 00011101 | 11 reg:3=a0&7 rm:3=d&7 | { i1:8=a1 ; i1:s8=a1 } }" "asm"="vcvtps2ph ${a1}, {a0}, {d}" "dclass"="xv" "feature"="f16c" "nodecode" {
  %r = call <8 x i16> @llvm.x86.vcvtps2ph.128(<4 x float> %a0, i32 %a1)
  ret <8 x i16> %r
}

declare <8 x i16> @llvm.x86.vcvtps2ph.128(<4 x float>, i32)

define <8 x i16> @X86.vcvtps2ph.256(<8 x float> "reg"="yv" %a0, i32 "imm" %a1) "enc"="{ vex(a0, 0, d, 3, 0, 0, 1, 1) | 00011101 | 11 reg:3=a0&7 rm:3=d&7 | { i1:8=a1 ; i1:s8=a1 } ; evexr(a0, d, 3, 0, 0, 1, 1, 0, 0, 0) | 00011101 | 11 reg:3=a0&7 rm:3=d&7 | { i1:8=a1 ; i1:s8=a1 } }" "asm"="vcvtps2ph ${a1}, {a0}, {d}" "dclass"="xv" "feature"="f16c" "nodecode" {
  %r = call <8 x i16> @llvm.x86.vcvtps2ph.256(<8 x float> %a0, i32 %a1)
  ret <8 x i16> %r
}

declare <8 x i16> @llvm.x86.vcvtps2ph.256(<8 x float>, i32)

define <16 x i8> @X86.vgf2p8affineinvqb.128(<16 x i8> "reg"="xv" "tied" %a0, <16 x i8> "reg"="xv" %a1, i8 "imm" %a2) "enc"="01100110 | rex(0, d, 0, a1) | 00001111 | 00111010 | 11001111 | 11 reg:3=d&7 rm:3=a1&7 | { i1:8=a2 ; i1:s8=a2 }" "asm"="gf2p8affineinvqb ${a2}, {a1}, {d}" "dclass"="xv" "feature"="gfni" "nodecode" {
  %r = call <16 x i8> @llvm.x86.vgf2p8affineinvqb.128(<16 x i8> %a0, <16 x i8> %a1, i8 %a2)
  ret <16 x i8> %r
}

declare <16 x i8> @llvm.x86.vgf2p8affineinvqb.128(<16 x i8>, <16 x i8>, i8)

define <32 x i8> @X86.vgf2p8affineinvqb.256(<32 x i8> "reg"="yv" %a0, <32 x i8> "reg"="yv" %a1, i8 "imm" %a2) "enc"="vex(d, 0, a1, 3, 1, a0, 1, 1) | 11001111 | 11 reg:3=d&7 rm:3=a1&7 | { i1:8=a2 ; i1:s8=a2 }" "asm"="vgf2p8affineinvqb ${a2}, {a1}, {a0}, {d}" "dclass"="yv" "feature"="gfni,avx" "nodecode" {
  %r = call <32 x i8> @llvm.x86.vgf2p8affineinvqb.256(<32 x i8> %a0, <32 x i8> %a1, i8 %a2)
  ret <32 x i8> %r
}

declare <32 x i8> @llvm.x86.vgf2p8affineinvqb.256(<32 x i8>, <32 x i8>, i8)

define <16 x i8> @X86.vgf2p8affineqb.128(<16 x i8> "reg"="xv" "tied" %a0, <16 x i8> "reg"="xv" %a1, i8 "imm" %a2) "enc"="01100110 | rex(0, d, 0, a1) | 00001111 | 00111010 | 11001110 | 11 reg:3=d&7 rm:3=a1&7 | { i1:8=a2 ; i1:s8=a2 }" "asm"="gf2p8affineqb ${a2}, {a1}, {d}" "dclass"="xv" "feature"="gfni" "nodecode" {
  %r = call <16 x i8> @llvm.x86.vgf2p8affineqb.128(<16 x i8> %a0, <16 x i8> %a1, i8 %a2)
  ret <16 x i8> %r
}

declare <16 x i8> @llvm.x86.vgf2p8affineqb.128(<16 x i8>, <16 x i8>, i8)

define <32 x i8> @X86.vgf2p8affineqb.256(<32 x i8> "reg"="yv" %a0, <32 x i8> "reg"="yv" %a1, i8 "imm" %a2) "enc"="vex(d, 0, a1, 3, 1, a0, 1, 1) | 11001110 | 11 reg:3=d&7 rm:3=a1&7 | { i1:8=a2 ; i1:s8=a2 }" "asm"="vgf2p8affineqb ${a2}, {a1}, {a0}, {d}" "dclass"="yv" "feature"="gfni,avx" "nodecode" {
  %r = call <32 x i8> @llvm.x86.vgf2p8affineqb.256(<32 x i8> %a0, <32 x i8> %a1, i8 %a2)
  ret <32 x i8> %r
}

declare <32 x i8> @llvm.x86.vgf2p8affineqb.256(<32 x i8>, <32 x i8>, i8)

define <16 x i8> @X86.vgf2p8mulb.128(<16 x i8> "reg"="xv" "tied" %a0, <16 x i8> "reg"="xv" %a1) "enc"="01100110 | rex(0, d, 0, a1) | 00001111 | 00111000 | 11001111 | 11 reg:3=d&7 rm:3=a1&7" "asm"="gf2p8mulb {a1}, {d}" "dclass"="xv" "feature"="gfni" "nodecode" {
  %r = call <16 x i8> @llvm.x86.vgf2p8mulb.128(<16 x i8> %a0, <16 x i8> %a1)
  ret <16 x i8> %r
}

declare <16 x i8> @llvm.x86.vgf2p8mulb.128(<16 x i8>, <16 x i8>)

define <32 x i8> @X86.vgf2p8mulb.256(<32 x i8> "reg"="yv" %a0, <32 x i8> "reg"="yv" %a1) "enc"="vex(d, 0, a1, 2, 0, a0, 1, 1) | 11001111 | 11 reg:3=d&7 rm:3=a1&7" "asm"="vgf2p8mulb {a1}, {a0}, {d}" "dclass"="yv" "feature"="gfni,avx" "nodecode" {
  %r = call <32 x i8> @llvm.x86.vgf2p8mulb.256(<32 x i8> %a0, <32 x i8> %a1)
  ret <32 x i8> %r
}

declare <32 x i8> @llvm.x86.vgf2p8mulb.256(<32 x i8>, <32 x i8>)

define <4 x i64> @X86.vsha512msg1(<4 x i64> "reg"="yv" "tied" %a0, <2 x i64> "reg"="xv" %a1) "enc"="vex(d, 0, a1, 2, 0, 0, 1, 3) | 11001100 | 11 reg:3=d&7 rm:3=a1&7" "asm"="vsha512msg1 {a1}, {d}" "dclass"="yv" "feature"="sha512,avx" "nodecode" {
  %r = call <4 x i64> @llvm.x86.vsha512msg1(<4 x i64> %a0, <2 x i64> %a1)
  ret <4 x i64> %r
}

declare <4 x i64> @llvm.x86.vsha512msg1(<4 x i64>, <2 x i64>)

define <4 x i64> @X86.vsha512msg2(<4 x i64> "reg"="yv" "tied" %a0, <4 x i64> "reg"="yv" %a1) "enc"="vex(d, 0, a1, 2, 0, 0, 1, 3) | 11001101 | 11 reg:3=d&7 rm:3=a1&7" "asm"="vsha512msg2 {a1}, {d}" "dclass"="yv" "feature"="sha512,avx" "nodecode" {
  %r = call <4 x i64> @llvm.x86.vsha512msg2(<4 x i64> %a0, <4 x i64> %a1)
  ret <4 x i64> %r
}

declare <4 x i64> @llvm.x86.vsha512msg2(<4 x i64>, <4 x i64>)

define <4 x i64> @X86.vsha512rnds2(<4 x i64> "reg"="yv" "tied" %a0, <4 x i64> "reg"="yv" %a1, <2 x i64> "reg"="xv" %a2) "enc"="vex(d, 0, a2, 2, 0, a1, 1, 3) | 11001011 | 11 reg:3=d&7 rm:3=a2&7" "asm"="vsha512rnds2 {a2}, {a1}, {d}" "dclass"="yv" "feature"="sha512,avx" "nodecode" {
  %r = call <4 x i64> @llvm.x86.vsha512rnds2(<4 x i64> %a0, <4 x i64> %a1, <2 x i64> %a2)
  ret <4 x i64> %r
}

declare <4 x i64> @llvm.x86.vsha512rnds2(<4 x i64>, <4 x i64>, <2 x i64>)

define <4 x i32> @X86.vsm3msg1(<4 x i32> "reg"="xv" "tied" %a0, <4 x i32> "reg"="xv" %a1, <4 x i32> "reg"="xv" %a2) "enc"="vex(d, 0, a2, 2, 0, a1, 0, 0) | 11011010 | 11 reg:3=d&7 rm:3=a2&7" "asm"="vsm3msg1 {a2}, {a1}, {d}" "dclass"="xv" "feature"="sm3,avx" "nodecode" {
  %r = call <4 x i32> @llvm.x86.vsm3msg1(<4 x i32> %a0, <4 x i32> %a1, <4 x i32> %a2)
  ret <4 x i32> %r
}

declare <4 x i32> @llvm.x86.vsm3msg1(<4 x i32>, <4 x i32>, <4 x i32>)

define <4 x i32> @X86.vsm3msg2(<4 x i32> "reg"="xv" "tied" %a0, <4 x i32> "reg"="xv" %a1, <4 x i32> "reg"="xv" %a2) "enc"="vex(d, 0, a2, 2, 0, a1, 0, 1) | 11011010 | 11 reg:3=d&7 rm:3=a2&7" "asm"="vsm3msg2 {a2}, {a1}, {d}" "dclass"="xv" "feature"="sm3,avx" "nodecode" {
  %r = call <4 x i32> @llvm.x86.vsm3msg2(<4 x i32> %a0, <4 x i32> %a1, <4 x i32> %a2)
  ret <4 x i32> %r
}

declare <4 x i32> @llvm.x86.vsm3msg2(<4 x i32>, <4 x i32>, <4 x i32>)

define <4 x i32> @X86.vsm3rnds2(<4 x i32> "reg"="xv" "tied" %a0, <4 x i32> "reg"="xv" %a1, <4 x i32> "reg"="xv" %a2, i32 "imm" %a3) "enc"="vex(d, 0, a2, 3, 0, a1, 0, 1) | 11011110 | 11 reg:3=d&7 rm:3=a2&7 | { i1:8=a3 ; i1:s8=a3 }" "asm"="vsm3rnds2 ${a3}, {a2}, {a1}, {d}" "dclass"="xv" "feature"="sm3,avx" "nodecode" {
  %r = call <4 x i32> @llvm.x86.vsm3rnds2(<4 x i32> %a0, <4 x i32> %a1, <4 x i32> %a2, i32 %a3)
  ret <4 x i32> %r
}

declare <4 x i32> @llvm.x86.vsm3rnds2(<4 x i32>, <4 x i32>, <4 x i32>, i32)

define void @X86.xend() "enc"="rex(0, 0, 0, 0) | 00001111 | 00000001 | 11 010 101" "asm"="xend" "feature"="rtm" "nodecode" {
  call void @llvm.x86.xend()
  ret void
}

declare void @llvm.x86.xend()

define void @X86.xrstor(ptr "reg"="r64" %a0, i32 "reg"="rdx" %a1, i32 "reg"="rax" %a2) "enc"="rex(0, 0, 4, a0) | 00001111 | 10101110 | rm(5, a0, 4, 1, 0, 0, 1)" "asm"="xrstor 0({a0})" "feature"="xsave" "nodecode" {
  call void @llvm.x86.xrstor(ptr %a0, i32 %a1, i32 %a2)
  ret void
}

declare void @llvm.x86.xrstor(ptr, i32, i32)

define void @X86.xrstor64(ptr "reg"="r64" %a0, i32 "reg"="rdx" %a1, i32 "reg"="rax" %a2) "enc"="rex(1, 0, 4, a0) | 00001111 | 10101110 | rm(5, a0, 4, 1, 0, 0, 1)" "asm"="xrstor64 0({a0})" "feature"="xsave" "nodecode" {
  call void @llvm.x86.xrstor64(ptr %a0, i32 %a1, i32 %a2)
  ret void
}

declare void @llvm.x86.xrstor64(ptr, i32, i32)

define void @X86.xrstors(ptr "reg"="r64" %a0, i32 "reg"="rdx" %a1, i32 "reg"="rax" %a2) "enc"="rex(0, 0, 4, a0) | 00001111 | 11000111 | rm(3, a0, 4, 1, 0, 0, 1)" "asm"="xrstors 0({a0})" "feature"="xsave,xsaves" "nodecode" {
  call void @llvm.x86.xrstors(ptr %a0, i32 %a1, i32 %a2)
  ret void
}

declare void @llvm.x86.xrstors(ptr, i32, i32)

define void @X86.xrstors64(ptr "reg"="r64" %a0, i32 "reg"="rdx" %a1, i32 "reg"="rax" %a2) "enc"="rex(1, 0, 4, a0) | 00001111 | 11000111 | rm(3, a0, 4, 1, 0, 0, 1)" "asm"="xrstors64 0({a0})" "feature"="xsave,xsaves" "nodecode" {
  call void @llvm.x86.xrstors64(ptr %a0, i32 %a1, i32 %a2)
  ret void
}

declare void @llvm.x86.xrstors64(ptr, i32, i32)

define void @X86.xsave(ptr "reg"="r64" %a0, i32 "reg"="rdx" %a1, i32 "reg"="rax" %a2) "enc"="rex(0, 0, 4, a0) | 00001111 | 10101110 | rm(4, a0, 4, 1, 0, 0, 1)" "asm"="xsave 0({a0})" "feature"="xsave" "nodecode" {
  call void @llvm.x86.xsave(ptr %a0, i32 %a1, i32 %a2)
  ret void
}

declare void @llvm.x86.xsave(ptr, i32, i32)

define void @X86.xsave64(ptr "reg"="r64" %a0, i32 "reg"="rdx" %a1, i32 "reg"="rax" %a2) "enc"="rex(1, 0, 4, a0) | 00001111 | 10101110 | rm(4, a0, 4, 1, 0, 0, 1)" "asm"="xsave64 0({a0})" "feature"="xsave" "nodecode" {
  call void @llvm.x86.xsave64(ptr %a0, i32 %a1, i32 %a2)
  ret void
}

declare void @llvm.x86.xsave64(ptr, i32, i32)

define void @X86.xsavec(ptr "reg"="r64" %a0, i32 "reg"="rdx" %a1, i32 "reg"="rax" %a2) "enc"="rex(0, 0, 4, a0) | 00001111 | 11000111 | rm(4, a0, 4, 1, 0, 0, 1)" "asm"="xsavec 0({a0})" "feature"="xsave,xsavec" "nodecode" {
  call void @llvm.x86.xsavec(ptr %a0, i32 %a1, i32 %a2)
  ret void
}

declare void @llvm.x86.xsavec(ptr, i32, i32)

define void @X86.xsavec64(ptr "reg"="r64" %a0, i32 "reg"="rdx" %a1, i32 "reg"="rax" %a2) "enc"="rex(1, 0, 4, a0) | 00001111 | 11000111 | rm(4, a0, 4, 1, 0, 0, 1)" "asm"="xsavec64 0({a0})" "feature"="xsave,xsavec" "nodecode" {
  call void @llvm.x86.xsavec64(ptr %a0, i32 %a1, i32 %a2)
  ret void
}

declare void @llvm.x86.xsavec64(ptr, i32, i32)

define void @X86.xsaveopt(ptr "reg"="r64" %a0, i32 "reg"="rdx" %a1, i32 "reg"="rax" %a2) "enc"="rex(0, 0, 4, a0) | 00001111 | 10101110 | rm(6, a0, 4, 1, 0, 0, 1)" "asm"="xsaveopt 0({a0})" "feature"="xsave,xsaveopt" "nodecode" {
  call void @llvm.x86.xsaveopt(ptr %a0, i32 %a1, i32 %a2)
  ret void
}

declare void @llvm.x86.xsaveopt(ptr, i32, i32)

define void @X86.xsaveopt64(ptr "reg"="r64" %a0, i32 "reg"="rdx" %a1, i32 "reg"="rax" %a2) "enc"="rex(1, 0, 4, a0) | 00001111 | 10101110 | rm(6, a0, 4, 1, 0, 0, 1)" "asm"="xsaveopt64 0({a0})" "feature"="xsave,xsaveopt" "nodecode" {
  call void @llvm.x86.xsaveopt64(ptr %a0, i32 %a1, i32 %a2)
  ret void
}

declare void @llvm.x86.xsaveopt64(ptr, i32, i32)

define void @X86.xsaves(ptr "reg"="r64" %a0, i32 "reg"="rdx" %a1, i32 "reg"="rax" %a2) "enc"="rex(0, 0, 4, a0) | 00001111 | 11000111 | rm(5, a0, 4, 1, 0, 0, 1)" "asm"="xsaves 0({a0})" "feature"="xsave,xsaves" "nodecode" {
  call void @llvm.x86.xsaves(ptr %a0, i32 %a1, i32 %a2)
  ret void
}

declare void @llvm.x86.xsaves(ptr, i32, i32)

define void @X86.xsaves64(ptr "reg"="r64" %a0, i32 "reg"="rdx" %a1, i32 "reg"="rax" %a2) "enc"="rex(1, 0, 4, a0) | 00001111 | 11000111 | rm(5, a0, 4, 1, 0, 0, 1)" "asm"="xsaves64 0({a0})" "feature"="xsave,xsaves" "nodecode" {
  call void @llvm.x86.xsaves64(ptr %a0, i32 %a1, i32 %a2)
  ret void
}

declare void @llvm.x86.xsaves64(ptr, i32, i32)

define void @X86.xsetbv(i32 "reg"="rcx" %a0, i32 "reg"="rdx" %a1, i32 "reg"="rax" %a2) "enc"="rex(0, 0, 0, 0) | 00001111 | 00000001 | 11 010 001" "asm"="xsetbv" "feature"="xsave" "nodecode" {
  call void @llvm.x86.xsetbv(i32 %a0, i32 %a1, i32 %a2)
  ret void
}

declare void @llvm.x86.xsetbv(i32, i32, i32)
