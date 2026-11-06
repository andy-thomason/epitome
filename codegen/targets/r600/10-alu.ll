; ALU instructions: ALU_WORD0 and ALU_WORD1_OP2 or ALU_WORD1_OP3, a 64-bit
; word each. Written by gen_r600.py.

define void @ADD.00(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000000000 00 1 0 0 0 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="ADD {last}{dst}, {n0}{s0}, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @ADD.10(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000000000 00 1 0 0 0 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="ADD {last}{dst}, {n0}|{s0}|, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @ADD.01(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000000000 00 1 0 0 1 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="ADD {last}{dst}, {n0}{s0}, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @ADD.11(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000000000 00 1 0 0 1 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="ADD {last}{dst}, {n0}|{s0}|, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @MUL.00(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000000001 00 1 0 0 0 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="MUL {last}{dst}, {n0}{s0}, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @MUL.10(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000000001 00 1 0 0 0 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="MUL {last}{dst}, {n0}|{s0}|, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @MUL.01(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000000001 00 1 0 0 1 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="MUL {last}{dst}, {n0}{s0}, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @MUL.11(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000000001 00 1 0 0 1 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="MUL {last}{dst}, {n0}|{s0}|, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @MUL_IEEE.00(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000000010 00 1 0 0 0 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="MUL_IEEE {last}{dst}, {n0}{s0}, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @MUL_IEEE.10(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000000010 00 1 0 0 0 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="MUL_IEEE {last}{dst}, {n0}|{s0}|, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @MUL_IEEE.01(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000000010 00 1 0 0 1 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="MUL_IEEE {last}{dst}, {n0}{s0}, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @MUL_IEEE.11(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000000010 00 1 0 0 1 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="MUL_IEEE {last}{dst}, {n0}|{s0}|, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @MAX.00(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000000011 00 1 0 0 0 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="MAX {last}{dst}, {n0}{s0}, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @MAX.10(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000000011 00 1 0 0 0 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="MAX {last}{dst}, {n0}|{s0}|, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @MAX.01(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000000011 00 1 0 0 1 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="MAX {last}{dst}, {n0}{s0}, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @MAX.11(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000000011 00 1 0 0 1 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="MAX {last}{dst}, {n0}|{s0}|, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @MIN.00(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000000100 00 1 0 0 0 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="MIN {last}{dst}, {n0}{s0}, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @MIN.10(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000000100 00 1 0 0 0 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="MIN {last}{dst}, {n0}|{s0}|, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @MIN.01(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000000100 00 1 0 0 1 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="MIN {last}{dst}, {n0}{s0}, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @MIN.11(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000000100 00 1 0 0 1 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="MIN {last}{dst}, {n0}|{s0}|, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @MAX_DX10.00(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000000101 00 1 0 0 0 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="MAX_DX10 {last}{dst}, {n0}{s0}, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @MAX_DX10.10(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000000101 00 1 0 0 0 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="MAX_DX10 {last}{dst}, {n0}|{s0}|, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @MAX_DX10.01(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000000101 00 1 0 0 1 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="MAX_DX10 {last}{dst}, {n0}{s0}, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @MAX_DX10.11(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000000101 00 1 0 0 1 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="MAX_DX10 {last}{dst}, {n0}|{s0}|, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @MIN_DX10.00(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000000110 00 1 0 0 0 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="MIN_DX10 {last}{dst}, {n0}{s0}, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @MIN_DX10.10(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000000110 00 1 0 0 0 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="MIN_DX10 {last}{dst}, {n0}|{s0}|, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @MIN_DX10.01(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000000110 00 1 0 0 1 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="MIN_DX10 {last}{dst}, {n0}{s0}, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @MIN_DX10.11(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000000110 00 1 0 0 1 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="MIN_DX10 {last}{dst}, {n0}|{s0}|, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @SETE.00(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000001000 00 1 0 0 0 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="SETE {last}{dst}, {n0}{s0}, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @SETE.10(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000001000 00 1 0 0 0 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="SETE {last}{dst}, {n0}|{s0}|, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @SETE.01(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000001000 00 1 0 0 1 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="SETE {last}{dst}, {n0}{s0}, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @SETE.11(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000001000 00 1 0 0 1 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="SETE {last}{dst}, {n0}|{s0}|, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @SETGT.00(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000001001 00 1 0 0 0 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="SETGT {last}{dst}, {n0}{s0}, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @SETGT.10(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000001001 00 1 0 0 0 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="SETGT {last}{dst}, {n0}|{s0}|, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @SETGT.01(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000001001 00 1 0 0 1 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="SETGT {last}{dst}, {n0}{s0}, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @SETGT.11(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000001001 00 1 0 0 1 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="SETGT {last}{dst}, {n0}|{s0}|, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @SETGE.00(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000001010 00 1 0 0 0 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="SETGE {last}{dst}, {n0}{s0}, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @SETGE.10(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000001010 00 1 0 0 0 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="SETGE {last}{dst}, {n0}|{s0}|, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @SETGE.01(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000001010 00 1 0 0 1 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="SETGE {last}{dst}, {n0}{s0}, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @SETGE.11(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000001010 00 1 0 0 1 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="SETGE {last}{dst}, {n0}|{s0}|, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @SETNE.00(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000001011 00 1 0 0 0 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="SETNE {last}{dst}, {n0}{s0}, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @SETNE.10(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000001011 00 1 0 0 0 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="SETNE {last}{dst}, {n0}|{s0}|, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @SETNE.01(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000001011 00 1 0 0 1 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="SETNE {last}{dst}, {n0}{s0}, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @SETNE.11(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000001011 00 1 0 0 1 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="SETNE {last}{dst}, {n0}|{s0}|, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @SETE_DX10.00(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000001100 00 1 0 0 0 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="SETE_DX10 {last}{dst}, {n0}{s0}, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @SETE_DX10.10(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000001100 00 1 0 0 0 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="SETE_DX10 {last}{dst}, {n0}|{s0}|, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @SETE_DX10.01(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000001100 00 1 0 0 1 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="SETE_DX10 {last}{dst}, {n0}{s0}, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @SETE_DX10.11(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000001100 00 1 0 0 1 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="SETE_DX10 {last}{dst}, {n0}|{s0}|, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @SETGT_DX10.00(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000001101 00 1 0 0 0 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="SETGT_DX10 {last}{dst}, {n0}{s0}, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @SETGT_DX10.10(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000001101 00 1 0 0 0 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="SETGT_DX10 {last}{dst}, {n0}|{s0}|, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @SETGT_DX10.01(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000001101 00 1 0 0 1 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="SETGT_DX10 {last}{dst}, {n0}{s0}, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @SETGT_DX10.11(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000001101 00 1 0 0 1 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="SETGT_DX10 {last}{dst}, {n0}|{s0}|, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @SETGE_DX10.00(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000001110 00 1 0 0 0 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="SETGE_DX10 {last}{dst}, {n0}{s0}, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @SETGE_DX10.10(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000001110 00 1 0 0 0 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="SETGE_DX10 {last}{dst}, {n0}|{s0}|, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @SETGE_DX10.01(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000001110 00 1 0 0 1 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="SETGE_DX10 {last}{dst}, {n0}{s0}, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @SETGE_DX10.11(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000001110 00 1 0 0 1 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="SETGE_DX10 {last}{dst}, {n0}|{s0}|, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @SETNE_DX10.00(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000001111 00 1 0 0 0 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="SETNE_DX10 {last}{dst}, {n0}{s0}, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @SETNE_DX10.10(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000001111 00 1 0 0 0 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="SETNE_DX10 {last}{dst}, {n0}|{s0}|, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @SETNE_DX10.01(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000001111 00 1 0 0 1 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="SETNE_DX10 {last}{dst}, {n0}{s0}, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @SETNE_DX10.11(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000001111 00 1 0 0 1 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="SETNE_DX10 {last}{dst}, {n0}|{s0}|, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @FRACT.00(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000010000 00 1 0 0 0 0 last:1 ps:2 000 0000000000000 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="FRACT {last}{dst}, {n0}{s0},{ps}{bs}" "asmonly" {
  ret void
}
define void @FRACT.10(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000010000 00 1 0 0 0 1 last:1 ps:2 000 0000000000000 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="FRACT {last}{dst}, {n0}|{s0}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @TRUNC.00(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000010001 00 1 0 0 0 0 last:1 ps:2 000 0000000000000 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="TRUNC {last}{dst}, {n0}{s0},{ps}{bs}" "asmonly" {
  ret void
}
define void @TRUNC.10(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000010001 00 1 0 0 0 1 last:1 ps:2 000 0000000000000 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="TRUNC {last}{dst}, {n0}|{s0}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @CEIL.00(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000010010 00 1 0 0 0 0 last:1 ps:2 000 0000000000000 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="CEIL {last}{dst}, {n0}{s0},{ps}{bs}" "asmonly" {
  ret void
}
define void @CEIL.10(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000010010 00 1 0 0 0 1 last:1 ps:2 000 0000000000000 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="CEIL {last}{dst}, {n0}|{s0}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @RNDNE.00(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000010011 00 1 0 0 0 0 last:1 ps:2 000 0000000000000 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="RNDNE {last}{dst}, {n0}{s0},{ps}{bs}" "asmonly" {
  ret void
}
define void @RNDNE.10(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000010011 00 1 0 0 0 1 last:1 ps:2 000 0000000000000 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="RNDNE {last}{dst}, {n0}|{s0}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @FLOOR.00(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000010100 00 1 0 0 0 0 last:1 ps:2 000 0000000000000 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="FLOOR {last}{dst}, {n0}{s0},{ps}{bs}" "asmonly" {
  ret void
}
define void @FLOOR.10(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000010100 00 1 0 0 0 1 last:1 ps:2 000 0000000000000 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="FLOOR {last}{dst}, {n0}|{s0}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @ASHR_INT.00(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000010101 00 1 0 0 0 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="ASHR_INT {last}{dst}, {n0}{s0}, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @ASHR_INT.10(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000010101 00 1 0 0 0 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="ASHR_INT {last}{dst}, {n0}|{s0}|, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @ASHR_INT.01(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000010101 00 1 0 0 1 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="ASHR_INT {last}{dst}, {n0}{s0}, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @ASHR_INT.11(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000010101 00 1 0 0 1 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="ASHR_INT {last}{dst}, {n0}|{s0}|, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @LSHR_INT.00(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000010110 00 1 0 0 0 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="LSHR_INT {last}{dst}, {n0}{s0}, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @LSHR_INT.10(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000010110 00 1 0 0 0 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="LSHR_INT {last}{dst}, {n0}|{s0}|, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @LSHR_INT.01(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000010110 00 1 0 0 1 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="LSHR_INT {last}{dst}, {n0}{s0}, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @LSHR_INT.11(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000010110 00 1 0 0 1 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="LSHR_INT {last}{dst}, {n0}|{s0}|, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @LSHL_INT.00(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000010111 00 1 0 0 0 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="LSHL_INT {last}{dst}, {n0}{s0}, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @LSHL_INT.10(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000010111 00 1 0 0 0 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="LSHL_INT {last}{dst}, {n0}|{s0}|, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @LSHL_INT.01(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000010111 00 1 0 0 1 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="LSHL_INT {last}{dst}, {n0}{s0}, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @LSHL_INT.11(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000010111 00 1 0 0 1 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="LSHL_INT {last}{dst}, {n0}|{s0}|, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @MOV.00(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000011001 00 1 0 0 0 0 last:1 ps:2 000 0000000000000 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="MOV {last}{dst}, {n0}{s0},{ps}{bs}" "asmonly" {
  ret void
}
define void @MOV.10(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000011001 00 1 0 0 0 1 last:1 ps:2 000 0000000000000 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="MOV {last}{dst}, {n0}|{s0}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @NOP.00(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000011010 00 1 0 0 0 0 last:1 ps:2 000 0000000000000 0000000000000" "asm"="NOP {last}{dst},{ps}{bs}" "asmonly" {
  ret void
}
define void @MUL_64.00(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000011011 00 1 0 0 0 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="MUL_64 {last}{dst}, {n0}{s0}, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @MUL_64.10(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000011011 00 1 0 0 0 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="MUL_64 {last}{dst}, {n0}|{s0}|, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @MUL_64.01(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000011011 00 1 0 0 1 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="MUL_64 {last}{dst}, {n0}{s0}, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @MUL_64.11(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000011011 00 1 0 0 1 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="MUL_64 {last}{dst}, {n0}|{s0}|, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @FLT64_TO_FLT32.00(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000011100 00 1 0 0 0 0 last:1 ps:2 000 0000000000000 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="FLT64_TO_FLT32 {last}{dst}, {n0}{s0},{ps}{bs}" "asmonly" {
  ret void
}
define void @FLT64_TO_FLT32.10(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000011100 00 1 0 0 0 1 last:1 ps:2 000 0000000000000 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="FLT64_TO_FLT32 {last}{dst}, {n0}|{s0}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @FLT32_TO_FLT64.00(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000011101 00 1 0 0 0 0 last:1 ps:2 000 0000000000000 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="FLT32_TO_FLT64 {last}{dst}, {n0}{s0},{ps}{bs}" "asmonly" {
  ret void
}
define void @FLT32_TO_FLT64.10(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000011101 00 1 0 0 0 1 last:1 ps:2 000 0000000000000 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="FLT32_TO_FLT64 {last}{dst}, {n0}|{s0}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SETGT_UINT.00(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000011110 00 1 0 0 0 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SETGT_UINT {last}{dst}, {n0}{s0}, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SETGT_UINT.pred.00(i32 "enum"="last" %last, i32 "enum"="pdst" %pdst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 00 0 0000000 bs:3 00000011110 00 0 up:1=pdst>>1 uem:1=pdst&1 0 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SETGT_UINT {last}{pdst}, {n0}{s0}, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SETGT_UINT.10(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000011110 00 1 0 0 0 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SETGT_UINT {last}{dst}, {n0}|{s0}|, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SETGT_UINT.pred.10(i32 "enum"="last" %last, i32 "enum"="pdst" %pdst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 00 0 0000000 bs:3 00000011110 00 0 up:1=pdst>>1 uem:1=pdst&1 0 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SETGT_UINT {last}{pdst}, {n0}|{s0}|, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SETGT_UINT.01(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000011110 00 1 0 0 1 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SETGT_UINT {last}{dst}, {n0}{s0}, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SETGT_UINT.pred.01(i32 "enum"="last" %last, i32 "enum"="pdst" %pdst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 00 0 0000000 bs:3 00000011110 00 0 up:1=pdst>>1 uem:1=pdst&1 1 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SETGT_UINT {last}{pdst}, {n0}{s0}, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SETGT_UINT.11(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000011110 00 1 0 0 1 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SETGT_UINT {last}{dst}, {n0}|{s0}|, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SETGT_UINT.pred.11(i32 "enum"="last" %last, i32 "enum"="pdst" %pdst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 00 0 0000000 bs:3 00000011110 00 0 up:1=pdst>>1 uem:1=pdst&1 1 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SETGT_UINT {last}{pdst}, {n0}|{s0}|, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SETGE_UINT.00(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000011111 00 1 0 0 0 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SETGE_UINT {last}{dst}, {n0}{s0}, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SETGE_UINT.pred.00(i32 "enum"="last" %last, i32 "enum"="pdst" %pdst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 00 0 0000000 bs:3 00000011111 00 0 up:1=pdst>>1 uem:1=pdst&1 0 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SETGE_UINT {last}{pdst}, {n0}{s0}, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SETGE_UINT.10(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000011111 00 1 0 0 0 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SETGE_UINT {last}{dst}, {n0}|{s0}|, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SETGE_UINT.pred.10(i32 "enum"="last" %last, i32 "enum"="pdst" %pdst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 00 0 0000000 bs:3 00000011111 00 0 up:1=pdst>>1 uem:1=pdst&1 0 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SETGE_UINT {last}{pdst}, {n0}|{s0}|, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SETGE_UINT.01(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000011111 00 1 0 0 1 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SETGE_UINT {last}{dst}, {n0}{s0}, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SETGE_UINT.pred.01(i32 "enum"="last" %last, i32 "enum"="pdst" %pdst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 00 0 0000000 bs:3 00000011111 00 0 up:1=pdst>>1 uem:1=pdst&1 1 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SETGE_UINT {last}{pdst}, {n0}{s0}, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SETGE_UINT.11(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000011111 00 1 0 0 1 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SETGE_UINT {last}{dst}, {n0}|{s0}|, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SETGE_UINT.pred.11(i32 "enum"="last" %last, i32 "enum"="pdst" %pdst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 00 0 0000000 bs:3 00000011111 00 0 up:1=pdst>>1 uem:1=pdst&1 1 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SETGE_UINT {last}{pdst}, {n0}|{s0}|, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SETE.00(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000100000 00 1 0 0 0 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SETE {last}{dst}, {n0}{s0}, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SETE.pred.00(i32 "enum"="last" %last, i32 "enum"="pdst" %pdst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 00 0 0000000 bs:3 00000100000 00 0 up:1=pdst>>1 uem:1=pdst&1 0 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SETE {last}{pdst}, {n0}{s0}, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SETE.10(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000100000 00 1 0 0 0 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SETE {last}{dst}, {n0}|{s0}|, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SETE.pred.10(i32 "enum"="last" %last, i32 "enum"="pdst" %pdst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 00 0 0000000 bs:3 00000100000 00 0 up:1=pdst>>1 uem:1=pdst&1 0 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SETE {last}{pdst}, {n0}|{s0}|, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SETE.01(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000100000 00 1 0 0 1 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SETE {last}{dst}, {n0}{s0}, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SETE.pred.01(i32 "enum"="last" %last, i32 "enum"="pdst" %pdst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 00 0 0000000 bs:3 00000100000 00 0 up:1=pdst>>1 uem:1=pdst&1 1 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SETE {last}{pdst}, {n0}{s0}, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SETE.11(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000100000 00 1 0 0 1 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SETE {last}{dst}, {n0}|{s0}|, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SETE.pred.11(i32 "enum"="last" %last, i32 "enum"="pdst" %pdst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 00 0 0000000 bs:3 00000100000 00 0 up:1=pdst>>1 uem:1=pdst&1 1 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SETE {last}{pdst}, {n0}|{s0}|, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SETGT.00(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000100001 00 1 0 0 0 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SETGT {last}{dst}, {n0}{s0}, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SETGT.pred.00(i32 "enum"="last" %last, i32 "enum"="pdst" %pdst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 00 0 0000000 bs:3 00000100001 00 0 up:1=pdst>>1 uem:1=pdst&1 0 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SETGT {last}{pdst}, {n0}{s0}, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SETGT.10(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000100001 00 1 0 0 0 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SETGT {last}{dst}, {n0}|{s0}|, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SETGT.pred.10(i32 "enum"="last" %last, i32 "enum"="pdst" %pdst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 00 0 0000000 bs:3 00000100001 00 0 up:1=pdst>>1 uem:1=pdst&1 0 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SETGT {last}{pdst}, {n0}|{s0}|, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SETGT.01(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000100001 00 1 0 0 1 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SETGT {last}{dst}, {n0}{s0}, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SETGT.pred.01(i32 "enum"="last" %last, i32 "enum"="pdst" %pdst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 00 0 0000000 bs:3 00000100001 00 0 up:1=pdst>>1 uem:1=pdst&1 1 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SETGT {last}{pdst}, {n0}{s0}, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SETGT.11(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000100001 00 1 0 0 1 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SETGT {last}{dst}, {n0}|{s0}|, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SETGT.pred.11(i32 "enum"="last" %last, i32 "enum"="pdst" %pdst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 00 0 0000000 bs:3 00000100001 00 0 up:1=pdst>>1 uem:1=pdst&1 1 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SETGT {last}{pdst}, {n0}|{s0}|, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SETGE.00(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000100010 00 1 0 0 0 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SETGE {last}{dst}, {n0}{s0}, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SETGE.pred.00(i32 "enum"="last" %last, i32 "enum"="pdst" %pdst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 00 0 0000000 bs:3 00000100010 00 0 up:1=pdst>>1 uem:1=pdst&1 0 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SETGE {last}{pdst}, {n0}{s0}, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SETGE.10(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000100010 00 1 0 0 0 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SETGE {last}{dst}, {n0}|{s0}|, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SETGE.pred.10(i32 "enum"="last" %last, i32 "enum"="pdst" %pdst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 00 0 0000000 bs:3 00000100010 00 0 up:1=pdst>>1 uem:1=pdst&1 0 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SETGE {last}{pdst}, {n0}|{s0}|, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SETGE.01(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000100010 00 1 0 0 1 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SETGE {last}{dst}, {n0}{s0}, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SETGE.pred.01(i32 "enum"="last" %last, i32 "enum"="pdst" %pdst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 00 0 0000000 bs:3 00000100010 00 0 up:1=pdst>>1 uem:1=pdst&1 1 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SETGE {last}{pdst}, {n0}{s0}, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SETGE.11(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000100010 00 1 0 0 1 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SETGE {last}{dst}, {n0}|{s0}|, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SETGE.pred.11(i32 "enum"="last" %last, i32 "enum"="pdst" %pdst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 00 0 0000000 bs:3 00000100010 00 0 up:1=pdst>>1 uem:1=pdst&1 1 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SETGE {last}{pdst}, {n0}|{s0}|, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SETNE.00(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000100011 00 1 0 0 0 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SETNE {last}{dst}, {n0}{s0}, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SETNE.pred.00(i32 "enum"="last" %last, i32 "enum"="pdst" %pdst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 00 0 0000000 bs:3 00000100011 00 0 up:1=pdst>>1 uem:1=pdst&1 0 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SETNE {last}{pdst}, {n0}{s0}, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SETNE.10(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000100011 00 1 0 0 0 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SETNE {last}{dst}, {n0}|{s0}|, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SETNE.pred.10(i32 "enum"="last" %last, i32 "enum"="pdst" %pdst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 00 0 0000000 bs:3 00000100011 00 0 up:1=pdst>>1 uem:1=pdst&1 0 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SETNE {last}{pdst}, {n0}|{s0}|, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SETNE.01(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000100011 00 1 0 0 1 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SETNE {last}{dst}, {n0}{s0}, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SETNE.pred.01(i32 "enum"="last" %last, i32 "enum"="pdst" %pdst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 00 0 0000000 bs:3 00000100011 00 0 up:1=pdst>>1 uem:1=pdst&1 1 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SETNE {last}{pdst}, {n0}{s0}, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SETNE.11(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000100011 00 1 0 0 1 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SETNE {last}{dst}, {n0}|{s0}|, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SETNE.pred.11(i32 "enum"="last" %last, i32 "enum"="pdst" %pdst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 00 0 0000000 bs:3 00000100011 00 0 up:1=pdst>>1 uem:1=pdst&1 1 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SETNE {last}{pdst}, {n0}|{s0}|, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SET_INV.00(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000100100 00 1 0 0 0 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SET_INV {last}{dst}, {n0}{s0}, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SET_INV.pred.00(i32 "enum"="last" %last, i32 "enum"="pdst" %pdst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 00 0 0000000 bs:3 00000100100 00 0 up:1=pdst>>1 uem:1=pdst&1 0 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SET_INV {last}{pdst}, {n0}{s0}, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SET_INV.10(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000100100 00 1 0 0 0 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SET_INV {last}{dst}, {n0}|{s0}|, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SET_INV.pred.10(i32 "enum"="last" %last, i32 "enum"="pdst" %pdst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 00 0 0000000 bs:3 00000100100 00 0 up:1=pdst>>1 uem:1=pdst&1 0 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SET_INV {last}{pdst}, {n0}|{s0}|, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SET_INV.01(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000100100 00 1 0 0 1 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SET_INV {last}{dst}, {n0}{s0}, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SET_INV.pred.01(i32 "enum"="last" %last, i32 "enum"="pdst" %pdst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 00 0 0000000 bs:3 00000100100 00 0 up:1=pdst>>1 uem:1=pdst&1 1 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SET_INV {last}{pdst}, {n0}{s0}, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SET_INV.11(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000100100 00 1 0 0 1 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SET_INV {last}{dst}, {n0}|{s0}|, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SET_INV.pred.11(i32 "enum"="last" %last, i32 "enum"="pdst" %pdst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 00 0 0000000 bs:3 00000100100 00 0 up:1=pdst>>1 uem:1=pdst&1 1 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SET_INV {last}{pdst}, {n0}|{s0}|, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SET_POP.00(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000100101 00 1 0 0 0 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SET_POP {last}{dst}, {n0}{s0}, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SET_POP.pred.00(i32 "enum"="last" %last, i32 "enum"="pdst" %pdst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 00 0 0000000 bs:3 00000100101 00 0 up:1=pdst>>1 uem:1=pdst&1 0 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SET_POP {last}{pdst}, {n0}{s0}, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SET_POP.10(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000100101 00 1 0 0 0 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SET_POP {last}{dst}, {n0}|{s0}|, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SET_POP.pred.10(i32 "enum"="last" %last, i32 "enum"="pdst" %pdst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 00 0 0000000 bs:3 00000100101 00 0 up:1=pdst>>1 uem:1=pdst&1 0 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SET_POP {last}{pdst}, {n0}|{s0}|, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SET_POP.01(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000100101 00 1 0 0 1 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SET_POP {last}{dst}, {n0}{s0}, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SET_POP.pred.01(i32 "enum"="last" %last, i32 "enum"="pdst" %pdst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 00 0 0000000 bs:3 00000100101 00 0 up:1=pdst>>1 uem:1=pdst&1 1 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SET_POP {last}{pdst}, {n0}{s0}, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SET_POP.11(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000100101 00 1 0 0 1 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SET_POP {last}{dst}, {n0}|{s0}|, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SET_POP.pred.11(i32 "enum"="last" %last, i32 "enum"="pdst" %pdst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 00 0 0000000 bs:3 00000100101 00 0 up:1=pdst>>1 uem:1=pdst&1 1 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SET_POP {last}{pdst}, {n0}|{s0}|, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SET_CLR.00(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000100110 00 1 0 0 0 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SET_CLR {last}{dst}, {n0}{s0}, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SET_CLR.pred.00(i32 "enum"="last" %last, i32 "enum"="pdst" %pdst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 00 0 0000000 bs:3 00000100110 00 0 up:1=pdst>>1 uem:1=pdst&1 0 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SET_CLR {last}{pdst}, {n0}{s0}, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SET_CLR.10(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000100110 00 1 0 0 0 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SET_CLR {last}{dst}, {n0}|{s0}|, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SET_CLR.pred.10(i32 "enum"="last" %last, i32 "enum"="pdst" %pdst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 00 0 0000000 bs:3 00000100110 00 0 up:1=pdst>>1 uem:1=pdst&1 0 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SET_CLR {last}{pdst}, {n0}|{s0}|, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SET_CLR.01(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000100110 00 1 0 0 1 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SET_CLR {last}{dst}, {n0}{s0}, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SET_CLR.pred.01(i32 "enum"="last" %last, i32 "enum"="pdst" %pdst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 00 0 0000000 bs:3 00000100110 00 0 up:1=pdst>>1 uem:1=pdst&1 1 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SET_CLR {last}{pdst}, {n0}{s0}, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SET_CLR.11(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000100110 00 1 0 0 1 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SET_CLR {last}{dst}, {n0}|{s0}|, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SET_CLR.pred.11(i32 "enum"="last" %last, i32 "enum"="pdst" %pdst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 00 0 0000000 bs:3 00000100110 00 0 up:1=pdst>>1 uem:1=pdst&1 1 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SET_CLR {last}{pdst}, {n0}|{s0}|, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SET_RESTORE.00(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000100111 00 1 0 0 0 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SET_RESTORE {last}{dst}, {n0}{s0}, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SET_RESTORE.pred.00(i32 "enum"="last" %last, i32 "enum"="pdst" %pdst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 00 0 0000000 bs:3 00000100111 00 0 up:1=pdst>>1 uem:1=pdst&1 0 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SET_RESTORE {last}{pdst}, {n0}{s0}, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SET_RESTORE.10(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000100111 00 1 0 0 0 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SET_RESTORE {last}{dst}, {n0}|{s0}|, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SET_RESTORE.pred.10(i32 "enum"="last" %last, i32 "enum"="pdst" %pdst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 00 0 0000000 bs:3 00000100111 00 0 up:1=pdst>>1 uem:1=pdst&1 0 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SET_RESTORE {last}{pdst}, {n0}|{s0}|, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SET_RESTORE.01(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000100111 00 1 0 0 1 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SET_RESTORE {last}{dst}, {n0}{s0}, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SET_RESTORE.pred.01(i32 "enum"="last" %last, i32 "enum"="pdst" %pdst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 00 0 0000000 bs:3 00000100111 00 0 up:1=pdst>>1 uem:1=pdst&1 1 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SET_RESTORE {last}{pdst}, {n0}{s0}, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SET_RESTORE.11(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000100111 00 1 0 0 1 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SET_RESTORE {last}{dst}, {n0}|{s0}|, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SET_RESTORE.pred.11(i32 "enum"="last" %last, i32 "enum"="pdst" %pdst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 00 0 0000000 bs:3 00000100111 00 0 up:1=pdst>>1 uem:1=pdst&1 1 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SET_RESTORE {last}{pdst}, {n0}|{s0}|, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SETE_PUSH.00(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000101000 00 1 0 0 0 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SETE_PUSH {last}{dst}, {n0}{s0}, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SETE_PUSH.pred.00(i32 "enum"="last" %last, i32 "enum"="pdst" %pdst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 00 0 0000000 bs:3 00000101000 00 0 up:1=pdst>>1 uem:1=pdst&1 0 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SETE_PUSH {last}{pdst}, {n0}{s0}, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SETE_PUSH.10(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000101000 00 1 0 0 0 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SETE_PUSH {last}{dst}, {n0}|{s0}|, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SETE_PUSH.pred.10(i32 "enum"="last" %last, i32 "enum"="pdst" %pdst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 00 0 0000000 bs:3 00000101000 00 0 up:1=pdst>>1 uem:1=pdst&1 0 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SETE_PUSH {last}{pdst}, {n0}|{s0}|, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SETE_PUSH.01(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000101000 00 1 0 0 1 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SETE_PUSH {last}{dst}, {n0}{s0}, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SETE_PUSH.pred.01(i32 "enum"="last" %last, i32 "enum"="pdst" %pdst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 00 0 0000000 bs:3 00000101000 00 0 up:1=pdst>>1 uem:1=pdst&1 1 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SETE_PUSH {last}{pdst}, {n0}{s0}, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SETE_PUSH.11(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000101000 00 1 0 0 1 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SETE_PUSH {last}{dst}, {n0}|{s0}|, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SETE_PUSH.pred.11(i32 "enum"="last" %last, i32 "enum"="pdst" %pdst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 00 0 0000000 bs:3 00000101000 00 0 up:1=pdst>>1 uem:1=pdst&1 1 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SETE_PUSH {last}{pdst}, {n0}|{s0}|, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SETGT_PUSH.00(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000101001 00 1 0 0 0 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SETGT_PUSH {last}{dst}, {n0}{s0}, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SETGT_PUSH.pred.00(i32 "enum"="last" %last, i32 "enum"="pdst" %pdst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 00 0 0000000 bs:3 00000101001 00 0 up:1=pdst>>1 uem:1=pdst&1 0 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SETGT_PUSH {last}{pdst}, {n0}{s0}, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SETGT_PUSH.10(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000101001 00 1 0 0 0 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SETGT_PUSH {last}{dst}, {n0}|{s0}|, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SETGT_PUSH.pred.10(i32 "enum"="last" %last, i32 "enum"="pdst" %pdst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 00 0 0000000 bs:3 00000101001 00 0 up:1=pdst>>1 uem:1=pdst&1 0 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SETGT_PUSH {last}{pdst}, {n0}|{s0}|, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SETGT_PUSH.01(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000101001 00 1 0 0 1 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SETGT_PUSH {last}{dst}, {n0}{s0}, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SETGT_PUSH.pred.01(i32 "enum"="last" %last, i32 "enum"="pdst" %pdst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 00 0 0000000 bs:3 00000101001 00 0 up:1=pdst>>1 uem:1=pdst&1 1 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SETGT_PUSH {last}{pdst}, {n0}{s0}, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SETGT_PUSH.11(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000101001 00 1 0 0 1 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SETGT_PUSH {last}{dst}, {n0}|{s0}|, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SETGT_PUSH.pred.11(i32 "enum"="last" %last, i32 "enum"="pdst" %pdst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 00 0 0000000 bs:3 00000101001 00 0 up:1=pdst>>1 uem:1=pdst&1 1 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SETGT_PUSH {last}{pdst}, {n0}|{s0}|, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SETGE_PUSH.00(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000101010 00 1 0 0 0 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SETGE_PUSH {last}{dst}, {n0}{s0}, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SETGE_PUSH.pred.00(i32 "enum"="last" %last, i32 "enum"="pdst" %pdst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 00 0 0000000 bs:3 00000101010 00 0 up:1=pdst>>1 uem:1=pdst&1 0 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SETGE_PUSH {last}{pdst}, {n0}{s0}, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SETGE_PUSH.10(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000101010 00 1 0 0 0 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SETGE_PUSH {last}{dst}, {n0}|{s0}|, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SETGE_PUSH.pred.10(i32 "enum"="last" %last, i32 "enum"="pdst" %pdst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 00 0 0000000 bs:3 00000101010 00 0 up:1=pdst>>1 uem:1=pdst&1 0 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SETGE_PUSH {last}{pdst}, {n0}|{s0}|, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SETGE_PUSH.01(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000101010 00 1 0 0 1 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SETGE_PUSH {last}{dst}, {n0}{s0}, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SETGE_PUSH.pred.01(i32 "enum"="last" %last, i32 "enum"="pdst" %pdst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 00 0 0000000 bs:3 00000101010 00 0 up:1=pdst>>1 uem:1=pdst&1 1 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SETGE_PUSH {last}{pdst}, {n0}{s0}, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SETGE_PUSH.11(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000101010 00 1 0 0 1 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SETGE_PUSH {last}{dst}, {n0}|{s0}|, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SETGE_PUSH.pred.11(i32 "enum"="last" %last, i32 "enum"="pdst" %pdst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 00 0 0000000 bs:3 00000101010 00 0 up:1=pdst>>1 uem:1=pdst&1 1 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SETGE_PUSH {last}{pdst}, {n0}|{s0}|, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SETNE_PUSH.00(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000101011 00 1 0 0 0 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SETNE_PUSH {last}{dst}, {n0}{s0}, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SETNE_PUSH.pred.00(i32 "enum"="last" %last, i32 "enum"="pdst" %pdst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 00 0 0000000 bs:3 00000101011 00 0 up:1=pdst>>1 uem:1=pdst&1 0 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SETNE_PUSH {last}{pdst}, {n0}{s0}, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SETNE_PUSH.10(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000101011 00 1 0 0 0 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SETNE_PUSH {last}{dst}, {n0}|{s0}|, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SETNE_PUSH.pred.10(i32 "enum"="last" %last, i32 "enum"="pdst" %pdst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 00 0 0000000 bs:3 00000101011 00 0 up:1=pdst>>1 uem:1=pdst&1 0 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SETNE_PUSH {last}{pdst}, {n0}|{s0}|, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SETNE_PUSH.01(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000101011 00 1 0 0 1 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SETNE_PUSH {last}{dst}, {n0}{s0}, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SETNE_PUSH.pred.01(i32 "enum"="last" %last, i32 "enum"="pdst" %pdst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 00 0 0000000 bs:3 00000101011 00 0 up:1=pdst>>1 uem:1=pdst&1 1 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SETNE_PUSH {last}{pdst}, {n0}{s0}, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SETNE_PUSH.11(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000101011 00 1 0 0 1 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SETNE_PUSH {last}{dst}, {n0}|{s0}|, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SETNE_PUSH.pred.11(i32 "enum"="last" %last, i32 "enum"="pdst" %pdst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 00 0 0000000 bs:3 00000101011 00 0 up:1=pdst>>1 uem:1=pdst&1 1 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SETNE_PUSH {last}{pdst}, {n0}|{s0}|, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @KILLE.00(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000101100 00 1 0 0 0 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="KILLE {last}{dst}, {n0}{s0}, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @KILLE.10(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000101100 00 1 0 0 0 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="KILLE {last}{dst}, {n0}|{s0}|, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @KILLE.01(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000101100 00 1 0 0 1 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="KILLE {last}{dst}, {n0}{s0}, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @KILLE.11(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000101100 00 1 0 0 1 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="KILLE {last}{dst}, {n0}|{s0}|, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @KILLGT.00(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000101101 00 1 0 0 0 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="KILLGT {last}{dst}, {n0}{s0}, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @KILLGT.10(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000101101 00 1 0 0 0 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="KILLGT {last}{dst}, {n0}|{s0}|, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @KILLGT.01(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000101101 00 1 0 0 1 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="KILLGT {last}{dst}, {n0}{s0}, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @KILLGT.11(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000101101 00 1 0 0 1 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="KILLGT {last}{dst}, {n0}|{s0}|, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @KILLGE.00(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000101110 00 1 0 0 0 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="KILLGE {last}{dst}, {n0}{s0}, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @KILLGE.10(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000101110 00 1 0 0 0 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="KILLGE {last}{dst}, {n0}|{s0}|, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @KILLGE.01(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000101110 00 1 0 0 1 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="KILLGE {last}{dst}, {n0}{s0}, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @KILLGE.11(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000101110 00 1 0 0 1 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="KILLGE {last}{dst}, {n0}|{s0}|, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @KILLNE.00(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000101111 00 1 0 0 0 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="KILLNE {last}{dst}, {n0}{s0}, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @KILLNE.10(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000101111 00 1 0 0 0 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="KILLNE {last}{dst}, {n0}|{s0}|, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @KILLNE.01(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000101111 00 1 0 0 1 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="KILLNE {last}{dst}, {n0}{s0}, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @KILLNE.11(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000101111 00 1 0 0 1 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="KILLNE {last}{dst}, {n0}|{s0}|, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @AND_INT.00(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000110000 00 1 0 0 0 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="AND_INT {last}{dst}, {n0}{s0}, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @AND_INT.10(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000110000 00 1 0 0 0 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="AND_INT {last}{dst}, {n0}|{s0}|, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @AND_INT.01(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000110000 00 1 0 0 1 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="AND_INT {last}{dst}, {n0}{s0}, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @AND_INT.11(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000110000 00 1 0 0 1 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="AND_INT {last}{dst}, {n0}|{s0}|, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @OR_INT.00(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000110001 00 1 0 0 0 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="OR_INT {last}{dst}, {n0}{s0}, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @OR_INT.10(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000110001 00 1 0 0 0 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="OR_INT {last}{dst}, {n0}|{s0}|, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @OR_INT.01(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000110001 00 1 0 0 1 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="OR_INT {last}{dst}, {n0}{s0}, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @OR_INT.11(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000110001 00 1 0 0 1 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="OR_INT {last}{dst}, {n0}|{s0}|, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @XOR_INT.00(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000110010 00 1 0 0 0 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="XOR_INT {last}{dst}, {n0}{s0}, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @XOR_INT.10(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000110010 00 1 0 0 0 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="XOR_INT {last}{dst}, {n0}|{s0}|, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @XOR_INT.01(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000110010 00 1 0 0 1 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="XOR_INT {last}{dst}, {n0}{s0}, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @XOR_INT.11(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000110010 00 1 0 0 1 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="XOR_INT {last}{dst}, {n0}|{s0}|, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @NOT_INT.00(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000110011 00 1 0 0 0 0 last:1 ps:2 000 0000000000000 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="NOT_INT {last}{dst}, {n0}{s0},{ps}{bs}" "asmonly" {
  ret void
}
define void @NOT_INT.10(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000110011 00 1 0 0 0 1 last:1 ps:2 000 0000000000000 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="NOT_INT {last}{dst}, {n0}|{s0}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @ADD_INT.00(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000110100 00 1 0 0 0 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="ADD_INT {last}{dst}, {n0}{s0}, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @ADD_INT.10(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000110100 00 1 0 0 0 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="ADD_INT {last}{dst}, {n0}|{s0}|, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @ADD_INT.01(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000110100 00 1 0 0 1 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="ADD_INT {last}{dst}, {n0}{s0}, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @ADD_INT.11(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000110100 00 1 0 0 1 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="ADD_INT {last}{dst}, {n0}|{s0}|, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @SUB_INT.00(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000110101 00 1 0 0 0 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="SUB_INT {last}{dst}, {n0}{s0}, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @SUB_INT.10(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000110101 00 1 0 0 0 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="SUB_INT {last}{dst}, {n0}|{s0}|, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @SUB_INT.01(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000110101 00 1 0 0 1 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="SUB_INT {last}{dst}, {n0}{s0}, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @SUB_INT.11(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000110101 00 1 0 0 1 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="SUB_INT {last}{dst}, {n0}|{s0}|, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @MAX_INT.00(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000110110 00 1 0 0 0 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="MAX_INT {last}{dst}, {n0}{s0}, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @MAX_INT.10(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000110110 00 1 0 0 0 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="MAX_INT {last}{dst}, {n0}|{s0}|, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @MAX_INT.01(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000110110 00 1 0 0 1 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="MAX_INT {last}{dst}, {n0}{s0}, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @MAX_INT.11(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000110110 00 1 0 0 1 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="MAX_INT {last}{dst}, {n0}|{s0}|, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @MIN_INT.00(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000110111 00 1 0 0 0 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="MIN_INT {last}{dst}, {n0}{s0}, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @MIN_INT.10(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000110111 00 1 0 0 0 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="MIN_INT {last}{dst}, {n0}|{s0}|, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @MIN_INT.01(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000110111 00 1 0 0 1 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="MIN_INT {last}{dst}, {n0}{s0}, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @MIN_INT.11(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000110111 00 1 0 0 1 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="MIN_INT {last}{dst}, {n0}|{s0}|, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @MAX_UINT.00(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000111000 00 1 0 0 0 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="MAX_UINT {last}{dst}, {n0}{s0}, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @MAX_UINT.10(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000111000 00 1 0 0 0 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="MAX_UINT {last}{dst}, {n0}|{s0}|, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @MAX_UINT.01(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000111000 00 1 0 0 1 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="MAX_UINT {last}{dst}, {n0}{s0}, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @MAX_UINT.11(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000111000 00 1 0 0 1 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="MAX_UINT {last}{dst}, {n0}|{s0}|, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @MIN_UINT.00(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000111001 00 1 0 0 0 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="MIN_UINT {last}{dst}, {n0}{s0}, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @MIN_UINT.10(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000111001 00 1 0 0 0 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="MIN_UINT {last}{dst}, {n0}|{s0}|, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @MIN_UINT.01(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000111001 00 1 0 0 1 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="MIN_UINT {last}{dst}, {n0}{s0}, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @MIN_UINT.11(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000111001 00 1 0 0 1 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="MIN_UINT {last}{dst}, {n0}|{s0}|, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @SETE_INT.00(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000111010 00 1 0 0 0 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="SETE_INT {last}{dst}, {n0}{s0}, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @SETE_INT.10(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000111010 00 1 0 0 0 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="SETE_INT {last}{dst}, {n0}|{s0}|, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @SETE_INT.01(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000111010 00 1 0 0 1 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="SETE_INT {last}{dst}, {n0}{s0}, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @SETE_INT.11(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000111010 00 1 0 0 1 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="SETE_INT {last}{dst}, {n0}|{s0}|, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @SETGT_INT.00(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000111011 00 1 0 0 0 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="SETGT_INT {last}{dst}, {n0}{s0}, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @SETGT_INT.10(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000111011 00 1 0 0 0 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="SETGT_INT {last}{dst}, {n0}|{s0}|, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @SETGT_INT.01(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000111011 00 1 0 0 1 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="SETGT_INT {last}{dst}, {n0}{s0}, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @SETGT_INT.11(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000111011 00 1 0 0 1 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="SETGT_INT {last}{dst}, {n0}|{s0}|, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @SETGE_INT.00(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000111100 00 1 0 0 0 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="SETGE_INT {last}{dst}, {n0}{s0}, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @SETGE_INT.10(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000111100 00 1 0 0 0 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="SETGE_INT {last}{dst}, {n0}|{s0}|, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @SETGE_INT.01(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000111100 00 1 0 0 1 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="SETGE_INT {last}{dst}, {n0}{s0}, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @SETGE_INT.11(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000111100 00 1 0 0 1 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="SETGE_INT {last}{dst}, {n0}|{s0}|, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @SETNE_INT.00(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000111101 00 1 0 0 0 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="SETNE_INT {last}{dst}, {n0}{s0}, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @SETNE_INT.10(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000111101 00 1 0 0 0 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="SETNE_INT {last}{dst}, {n0}|{s0}|, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @SETNE_INT.01(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000111101 00 1 0 0 1 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="SETNE_INT {last}{dst}, {n0}{s0}, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @SETNE_INT.11(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000111101 00 1 0 0 1 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="SETNE_INT {last}{dst}, {n0}|{s0}|, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @SETGT_UINT.00(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000111110 00 1 0 0 0 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="SETGT_UINT {last}{dst}, {n0}{s0}, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @SETGT_UINT.10(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000111110 00 1 0 0 0 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="SETGT_UINT {last}{dst}, {n0}|{s0}|, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @SETGT_UINT.01(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000111110 00 1 0 0 1 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="SETGT_UINT {last}{dst}, {n0}{s0}, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @SETGT_UINT.11(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000111110 00 1 0 0 1 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="SETGT_UINT {last}{dst}, {n0}|{s0}|, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @SETGE_UINT.00(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000111111 00 1 0 0 0 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="SETGE_UINT {last}{dst}, {n0}{s0}, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @SETGE_UINT.10(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000111111 00 1 0 0 0 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="SETGE_UINT {last}{dst}, {n0}|{s0}|, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @SETGE_UINT.01(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000111111 00 1 0 0 1 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="SETGE_UINT {last}{dst}, {n0}{s0}, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @SETGE_UINT.11(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00000111111 00 1 0 0 1 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="SETGE_UINT {last}{dst}, {n0}|{s0}|, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @KILLGT_UINT.00(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00001000000 00 1 0 0 0 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="KILLGT_UINT {last}{dst}, {n0}{s0}, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @KILLGT_UINT.10(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00001000000 00 1 0 0 0 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="KILLGT_UINT {last}{dst}, {n0}|{s0}|, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @KILLGT_UINT.01(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00001000000 00 1 0 0 1 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="KILLGT_UINT {last}{dst}, {n0}{s0}, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @KILLGT_UINT.11(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00001000000 00 1 0 0 1 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="KILLGT_UINT {last}{dst}, {n0}|{s0}|, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @KILLGE_UINT.00(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00001000001 00 1 0 0 0 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="KILLGE_UINT {last}{dst}, {n0}{s0}, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @KILLGE_UINT.10(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00001000001 00 1 0 0 0 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="KILLGE_UINT {last}{dst}, {n0}|{s0}|, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @KILLGE_UINT.01(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00001000001 00 1 0 0 1 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="KILLGE_UINT {last}{dst}, {n0}{s0}, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @KILLGE_UINT.11(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00001000001 00 1 0 0 1 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="KILLGE_UINT {last}{dst}, {n0}|{s0}|, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SETE_INT.00(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00001000010 00 1 0 0 0 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SETE_INT {last}{dst}, {n0}{s0}, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SETE_INT.pred.00(i32 "enum"="last" %last, i32 "enum"="pdst" %pdst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 00 0 0000000 bs:3 00001000010 00 0 up:1=pdst>>1 uem:1=pdst&1 0 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SETE_INT {last}{pdst}, {n0}{s0}, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SETE_INT.10(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00001000010 00 1 0 0 0 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SETE_INT {last}{dst}, {n0}|{s0}|, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SETE_INT.pred.10(i32 "enum"="last" %last, i32 "enum"="pdst" %pdst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 00 0 0000000 bs:3 00001000010 00 0 up:1=pdst>>1 uem:1=pdst&1 0 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SETE_INT {last}{pdst}, {n0}|{s0}|, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SETE_INT.01(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00001000010 00 1 0 0 1 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SETE_INT {last}{dst}, {n0}{s0}, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SETE_INT.pred.01(i32 "enum"="last" %last, i32 "enum"="pdst" %pdst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 00 0 0000000 bs:3 00001000010 00 0 up:1=pdst>>1 uem:1=pdst&1 1 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SETE_INT {last}{pdst}, {n0}{s0}, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SETE_INT.11(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00001000010 00 1 0 0 1 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SETE_INT {last}{dst}, {n0}|{s0}|, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SETE_INT.pred.11(i32 "enum"="last" %last, i32 "enum"="pdst" %pdst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 00 0 0000000 bs:3 00001000010 00 0 up:1=pdst>>1 uem:1=pdst&1 1 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SETE_INT {last}{pdst}, {n0}|{s0}|, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SETGT_INT.00(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00001000011 00 1 0 0 0 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SETGT_INT {last}{dst}, {n0}{s0}, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SETGT_INT.pred.00(i32 "enum"="last" %last, i32 "enum"="pdst" %pdst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 00 0 0000000 bs:3 00001000011 00 0 up:1=pdst>>1 uem:1=pdst&1 0 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SETGT_INT {last}{pdst}, {n0}{s0}, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SETGT_INT.10(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00001000011 00 1 0 0 0 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SETGT_INT {last}{dst}, {n0}|{s0}|, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SETGT_INT.pred.10(i32 "enum"="last" %last, i32 "enum"="pdst" %pdst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 00 0 0000000 bs:3 00001000011 00 0 up:1=pdst>>1 uem:1=pdst&1 0 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SETGT_INT {last}{pdst}, {n0}|{s0}|, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SETGT_INT.01(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00001000011 00 1 0 0 1 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SETGT_INT {last}{dst}, {n0}{s0}, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SETGT_INT.pred.01(i32 "enum"="last" %last, i32 "enum"="pdst" %pdst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 00 0 0000000 bs:3 00001000011 00 0 up:1=pdst>>1 uem:1=pdst&1 1 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SETGT_INT {last}{pdst}, {n0}{s0}, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SETGT_INT.11(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00001000011 00 1 0 0 1 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SETGT_INT {last}{dst}, {n0}|{s0}|, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SETGT_INT.pred.11(i32 "enum"="last" %last, i32 "enum"="pdst" %pdst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 00 0 0000000 bs:3 00001000011 00 0 up:1=pdst>>1 uem:1=pdst&1 1 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SETGT_INT {last}{pdst}, {n0}|{s0}|, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SETGE_INT.00(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00001000100 00 1 0 0 0 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SETGE_INT {last}{dst}, {n0}{s0}, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SETGE_INT.pred.00(i32 "enum"="last" %last, i32 "enum"="pdst" %pdst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 00 0 0000000 bs:3 00001000100 00 0 up:1=pdst>>1 uem:1=pdst&1 0 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SETGE_INT {last}{pdst}, {n0}{s0}, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SETGE_INT.10(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00001000100 00 1 0 0 0 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SETGE_INT {last}{dst}, {n0}|{s0}|, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SETGE_INT.pred.10(i32 "enum"="last" %last, i32 "enum"="pdst" %pdst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 00 0 0000000 bs:3 00001000100 00 0 up:1=pdst>>1 uem:1=pdst&1 0 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SETGE_INT {last}{pdst}, {n0}|{s0}|, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SETGE_INT.01(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00001000100 00 1 0 0 1 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SETGE_INT {last}{dst}, {n0}{s0}, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SETGE_INT.pred.01(i32 "enum"="last" %last, i32 "enum"="pdst" %pdst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 00 0 0000000 bs:3 00001000100 00 0 up:1=pdst>>1 uem:1=pdst&1 1 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SETGE_INT {last}{pdst}, {n0}{s0}, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SETGE_INT.11(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00001000100 00 1 0 0 1 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SETGE_INT {last}{dst}, {n0}|{s0}|, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SETGE_INT.pred.11(i32 "enum"="last" %last, i32 "enum"="pdst" %pdst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 00 0 0000000 bs:3 00001000100 00 0 up:1=pdst>>1 uem:1=pdst&1 1 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SETGE_INT {last}{pdst}, {n0}|{s0}|, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SETNE_INT.00(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00001000101 00 1 0 0 0 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SETNE_INT {last}{dst}, {n0}{s0}, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SETNE_INT.pred.00(i32 "enum"="last" %last, i32 "enum"="pdst" %pdst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 00 0 0000000 bs:3 00001000101 00 0 up:1=pdst>>1 uem:1=pdst&1 0 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SETNE_INT {last}{pdst}, {n0}{s0}, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SETNE_INT.10(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00001000101 00 1 0 0 0 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SETNE_INT {last}{dst}, {n0}|{s0}|, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SETNE_INT.pred.10(i32 "enum"="last" %last, i32 "enum"="pdst" %pdst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 00 0 0000000 bs:3 00001000101 00 0 up:1=pdst>>1 uem:1=pdst&1 0 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SETNE_INT {last}{pdst}, {n0}|{s0}|, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SETNE_INT.01(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00001000101 00 1 0 0 1 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SETNE_INT {last}{dst}, {n0}{s0}, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SETNE_INT.pred.01(i32 "enum"="last" %last, i32 "enum"="pdst" %pdst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 00 0 0000000 bs:3 00001000101 00 0 up:1=pdst>>1 uem:1=pdst&1 1 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SETNE_INT {last}{pdst}, {n0}{s0}, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SETNE_INT.11(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00001000101 00 1 0 0 1 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SETNE_INT {last}{dst}, {n0}|{s0}|, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SETNE_INT.pred.11(i32 "enum"="last" %last, i32 "enum"="pdst" %pdst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 00 0 0000000 bs:3 00001000101 00 0 up:1=pdst>>1 uem:1=pdst&1 1 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SETNE_INT {last}{pdst}, {n0}|{s0}|, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @KILLE_INT.00(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00001000110 00 1 0 0 0 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="KILLE_INT {last}{dst}, {n0}{s0}, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @KILLE_INT.10(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00001000110 00 1 0 0 0 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="KILLE_INT {last}{dst}, {n0}|{s0}|, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @KILLE_INT.01(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00001000110 00 1 0 0 1 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="KILLE_INT {last}{dst}, {n0}{s0}, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @KILLE_INT.11(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00001000110 00 1 0 0 1 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="KILLE_INT {last}{dst}, {n0}|{s0}|, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @KILLGT_INT.00(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00001000111 00 1 0 0 0 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="KILLGT_INT {last}{dst}, {n0}{s0}, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @KILLGT_INT.10(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00001000111 00 1 0 0 0 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="KILLGT_INT {last}{dst}, {n0}|{s0}|, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @KILLGT_INT.01(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00001000111 00 1 0 0 1 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="KILLGT_INT {last}{dst}, {n0}{s0}, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @KILLGT_INT.11(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00001000111 00 1 0 0 1 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="KILLGT_INT {last}{dst}, {n0}|{s0}|, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @KILLGE_INT.00(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00001001000 00 1 0 0 0 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="KILLGE_INT {last}{dst}, {n0}{s0}, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @KILLGE_INT.10(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00001001000 00 1 0 0 0 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="KILLGE_INT {last}{dst}, {n0}|{s0}|, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @KILLGE_INT.01(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00001001000 00 1 0 0 1 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="KILLGE_INT {last}{dst}, {n0}{s0}, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @KILLGE_INT.11(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00001001000 00 1 0 0 1 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="KILLGE_INT {last}{dst}, {n0}|{s0}|, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @KILLNE_INT.00(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00001001001 00 1 0 0 0 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="KILLNE_INT {last}{dst}, {n0}{s0}, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @KILLNE_INT.10(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00001001001 00 1 0 0 0 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="KILLNE_INT {last}{dst}, {n0}|{s0}|, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @KILLNE_INT.01(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00001001001 00 1 0 0 1 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="KILLNE_INT {last}{dst}, {n0}{s0}, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @KILLNE_INT.11(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00001001001 00 1 0 0 1 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="KILLNE_INT {last}{dst}, {n0}|{s0}|, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SETE_PUSH_INT.00(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00001001010 00 1 0 0 0 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SETE_PUSH_INT {last}{dst}, {n0}{s0}, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SETE_PUSH_INT.pred.00(i32 "enum"="last" %last, i32 "enum"="pdst" %pdst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 00 0 0000000 bs:3 00001001010 00 0 up:1=pdst>>1 uem:1=pdst&1 0 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SETE_PUSH_INT {last}{pdst}, {n0}{s0}, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SETE_PUSH_INT.10(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00001001010 00 1 0 0 0 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SETE_PUSH_INT {last}{dst}, {n0}|{s0}|, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SETE_PUSH_INT.pred.10(i32 "enum"="last" %last, i32 "enum"="pdst" %pdst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 00 0 0000000 bs:3 00001001010 00 0 up:1=pdst>>1 uem:1=pdst&1 0 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SETE_PUSH_INT {last}{pdst}, {n0}|{s0}|, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SETE_PUSH_INT.01(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00001001010 00 1 0 0 1 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SETE_PUSH_INT {last}{dst}, {n0}{s0}, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SETE_PUSH_INT.pred.01(i32 "enum"="last" %last, i32 "enum"="pdst" %pdst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 00 0 0000000 bs:3 00001001010 00 0 up:1=pdst>>1 uem:1=pdst&1 1 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SETE_PUSH_INT {last}{pdst}, {n0}{s0}, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SETE_PUSH_INT.11(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00001001010 00 1 0 0 1 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SETE_PUSH_INT {last}{dst}, {n0}|{s0}|, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SETE_PUSH_INT.pred.11(i32 "enum"="last" %last, i32 "enum"="pdst" %pdst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 00 0 0000000 bs:3 00001001010 00 0 up:1=pdst>>1 uem:1=pdst&1 1 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SETE_PUSH_INT {last}{pdst}, {n0}|{s0}|, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SETGT_PUSH_INT.00(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00001001011 00 1 0 0 0 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SETGT_PUSH_INT {last}{dst}, {n0}{s0}, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SETGT_PUSH_INT.pred.00(i32 "enum"="last" %last, i32 "enum"="pdst" %pdst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 00 0 0000000 bs:3 00001001011 00 0 up:1=pdst>>1 uem:1=pdst&1 0 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SETGT_PUSH_INT {last}{pdst}, {n0}{s0}, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SETGT_PUSH_INT.10(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00001001011 00 1 0 0 0 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SETGT_PUSH_INT {last}{dst}, {n0}|{s0}|, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SETGT_PUSH_INT.pred.10(i32 "enum"="last" %last, i32 "enum"="pdst" %pdst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 00 0 0000000 bs:3 00001001011 00 0 up:1=pdst>>1 uem:1=pdst&1 0 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SETGT_PUSH_INT {last}{pdst}, {n0}|{s0}|, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SETGT_PUSH_INT.01(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00001001011 00 1 0 0 1 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SETGT_PUSH_INT {last}{dst}, {n0}{s0}, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SETGT_PUSH_INT.pred.01(i32 "enum"="last" %last, i32 "enum"="pdst" %pdst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 00 0 0000000 bs:3 00001001011 00 0 up:1=pdst>>1 uem:1=pdst&1 1 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SETGT_PUSH_INT {last}{pdst}, {n0}{s0}, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SETGT_PUSH_INT.11(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00001001011 00 1 0 0 1 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SETGT_PUSH_INT {last}{dst}, {n0}|{s0}|, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SETGT_PUSH_INT.pred.11(i32 "enum"="last" %last, i32 "enum"="pdst" %pdst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 00 0 0000000 bs:3 00001001011 00 0 up:1=pdst>>1 uem:1=pdst&1 1 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SETGT_PUSH_INT {last}{pdst}, {n0}|{s0}|, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SETGE_PUSH_INT.00(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00001001100 00 1 0 0 0 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SETGE_PUSH_INT {last}{dst}, {n0}{s0}, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SETGE_PUSH_INT.pred.00(i32 "enum"="last" %last, i32 "enum"="pdst" %pdst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 00 0 0000000 bs:3 00001001100 00 0 up:1=pdst>>1 uem:1=pdst&1 0 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SETGE_PUSH_INT {last}{pdst}, {n0}{s0}, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SETGE_PUSH_INT.10(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00001001100 00 1 0 0 0 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SETGE_PUSH_INT {last}{dst}, {n0}|{s0}|, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SETGE_PUSH_INT.pred.10(i32 "enum"="last" %last, i32 "enum"="pdst" %pdst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 00 0 0000000 bs:3 00001001100 00 0 up:1=pdst>>1 uem:1=pdst&1 0 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SETGE_PUSH_INT {last}{pdst}, {n0}|{s0}|, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SETGE_PUSH_INT.01(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00001001100 00 1 0 0 1 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SETGE_PUSH_INT {last}{dst}, {n0}{s0}, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SETGE_PUSH_INT.pred.01(i32 "enum"="last" %last, i32 "enum"="pdst" %pdst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 00 0 0000000 bs:3 00001001100 00 0 up:1=pdst>>1 uem:1=pdst&1 1 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SETGE_PUSH_INT {last}{pdst}, {n0}{s0}, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SETGE_PUSH_INT.11(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00001001100 00 1 0 0 1 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SETGE_PUSH_INT {last}{dst}, {n0}|{s0}|, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SETGE_PUSH_INT.pred.11(i32 "enum"="last" %last, i32 "enum"="pdst" %pdst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 00 0 0000000 bs:3 00001001100 00 0 up:1=pdst>>1 uem:1=pdst&1 1 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SETGE_PUSH_INT {last}{pdst}, {n0}|{s0}|, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SETNE_PUSH_INT.00(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00001001101 00 1 0 0 0 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SETNE_PUSH_INT {last}{dst}, {n0}{s0}, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SETNE_PUSH_INT.pred.00(i32 "enum"="last" %last, i32 "enum"="pdst" %pdst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 00 0 0000000 bs:3 00001001101 00 0 up:1=pdst>>1 uem:1=pdst&1 0 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SETNE_PUSH_INT {last}{pdst}, {n0}{s0}, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SETNE_PUSH_INT.10(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00001001101 00 1 0 0 0 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SETNE_PUSH_INT {last}{dst}, {n0}|{s0}|, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SETNE_PUSH_INT.pred.10(i32 "enum"="last" %last, i32 "enum"="pdst" %pdst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 00 0 0000000 bs:3 00001001101 00 0 up:1=pdst>>1 uem:1=pdst&1 0 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SETNE_PUSH_INT {last}{pdst}, {n0}|{s0}|, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SETNE_PUSH_INT.01(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00001001101 00 1 0 0 1 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SETNE_PUSH_INT {last}{dst}, {n0}{s0}, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SETNE_PUSH_INT.pred.01(i32 "enum"="last" %last, i32 "enum"="pdst" %pdst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 00 0 0000000 bs:3 00001001101 00 0 up:1=pdst>>1 uem:1=pdst&1 1 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SETNE_PUSH_INT {last}{pdst}, {n0}{s0}, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SETNE_PUSH_INT.11(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00001001101 00 1 0 0 1 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SETNE_PUSH_INT {last}{dst}, {n0}|{s0}|, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SETNE_PUSH_INT.pred.11(i32 "enum"="last" %last, i32 "enum"="pdst" %pdst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 00 0 0000000 bs:3 00001001101 00 0 up:1=pdst>>1 uem:1=pdst&1 1 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SETNE_PUSH_INT {last}{pdst}, {n0}|{s0}|, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SETLT_PUSH_INT.00(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00001001110 00 1 0 0 0 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SETLT_PUSH_INT {last}{dst}, {n0}{s0}, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SETLT_PUSH_INT.pred.00(i32 "enum"="last" %last, i32 "enum"="pdst" %pdst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 00 0 0000000 bs:3 00001001110 00 0 up:1=pdst>>1 uem:1=pdst&1 0 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SETLT_PUSH_INT {last}{pdst}, {n0}{s0}, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SETLT_PUSH_INT.10(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00001001110 00 1 0 0 0 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SETLT_PUSH_INT {last}{dst}, {n0}|{s0}|, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SETLT_PUSH_INT.pred.10(i32 "enum"="last" %last, i32 "enum"="pdst" %pdst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 00 0 0000000 bs:3 00001001110 00 0 up:1=pdst>>1 uem:1=pdst&1 0 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SETLT_PUSH_INT {last}{pdst}, {n0}|{s0}|, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SETLT_PUSH_INT.01(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00001001110 00 1 0 0 1 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SETLT_PUSH_INT {last}{dst}, {n0}{s0}, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SETLT_PUSH_INT.pred.01(i32 "enum"="last" %last, i32 "enum"="pdst" %pdst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 00 0 0000000 bs:3 00001001110 00 0 up:1=pdst>>1 uem:1=pdst&1 1 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SETLT_PUSH_INT {last}{pdst}, {n0}{s0}, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SETLT_PUSH_INT.11(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00001001110 00 1 0 0 1 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SETLT_PUSH_INT {last}{dst}, {n0}|{s0}|, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SETLT_PUSH_INT.pred.11(i32 "enum"="last" %last, i32 "enum"="pdst" %pdst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 00 0 0000000 bs:3 00001001110 00 0 up:1=pdst>>1 uem:1=pdst&1 1 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SETLT_PUSH_INT {last}{pdst}, {n0}|{s0}|, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SETLE_PUSH_INT.00(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00001001111 00 1 0 0 0 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SETLE_PUSH_INT {last}{dst}, {n0}{s0}, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SETLE_PUSH_INT.pred.00(i32 "enum"="last" %last, i32 "enum"="pdst" %pdst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 00 0 0000000 bs:3 00001001111 00 0 up:1=pdst>>1 uem:1=pdst&1 0 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SETLE_PUSH_INT {last}{pdst}, {n0}{s0}, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SETLE_PUSH_INT.10(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00001001111 00 1 0 0 0 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SETLE_PUSH_INT {last}{dst}, {n0}|{s0}|, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SETLE_PUSH_INT.pred.10(i32 "enum"="last" %last, i32 "enum"="pdst" %pdst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 00 0 0000000 bs:3 00001001111 00 0 up:1=pdst>>1 uem:1=pdst&1 0 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SETLE_PUSH_INT {last}{pdst}, {n0}|{s0}|, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SETLE_PUSH_INT.01(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00001001111 00 1 0 0 1 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SETLE_PUSH_INT {last}{dst}, {n0}{s0}, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SETLE_PUSH_INT.pred.01(i32 "enum"="last" %last, i32 "enum"="pdst" %pdst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 00 0 0000000 bs:3 00001001111 00 0 up:1=pdst>>1 uem:1=pdst&1 1 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SETLE_PUSH_INT {last}{pdst}, {n0}{s0}, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SETLE_PUSH_INT.11(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00001001111 00 1 0 0 1 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SETLE_PUSH_INT {last}{dst}, {n0}|{s0}|, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SETLE_PUSH_INT.pred.11(i32 "enum"="last" %last, i32 "enum"="pdst" %pdst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 00 0 0000000 bs:3 00001001111 00 0 up:1=pdst>>1 uem:1=pdst&1 1 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SETLE_PUSH_INT {last}{pdst}, {n0}|{s0}|, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @FLT_TO_INT.00(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00001010000 00 1 0 0 0 0 last:1 ps:2 000 0000000000000 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="FLT_TO_INT {last}{dst}, {n0}{s0},{ps}{bs}" "asmonly" {
  ret void
}
define void @FLT_TO_INT.10(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00001010000 00 1 0 0 0 1 last:1 ps:2 000 0000000000000 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="FLT_TO_INT {last}{dst}, {n0}|{s0}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @BFREV_INT.00(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00001010001 00 1 0 0 0 0 last:1 ps:2 000 0000000000000 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="BFREV_INT {last}{dst}, {n0}{s0},{ps}{bs}" "asmonly" {
  ret void
}
define void @BFREV_INT.10(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00001010001 00 1 0 0 0 1 last:1 ps:2 000 0000000000000 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="BFREV_INT {last}{dst}, {n0}|{s0}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @ADDC_UINT.00(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00001010010 00 1 0 0 0 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="ADDC_UINT {last}{dst}, {n0}{s0}, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @ADDC_UINT.10(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00001010010 00 1 0 0 0 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="ADDC_UINT {last}{dst}, {n0}|{s0}|, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @ADDC_UINT.01(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00001010010 00 1 0 0 1 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="ADDC_UINT {last}{dst}, {n0}{s0}, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @ADDC_UINT.11(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00001010010 00 1 0 0 1 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="ADDC_UINT {last}{dst}, {n0}|{s0}|, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @SUBB_UINT.00(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00001010011 00 1 0 0 0 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="SUBB_UINT {last}{dst}, {n0}{s0}, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @SUBB_UINT.10(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00001010011 00 1 0 0 0 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="SUBB_UINT {last}{dst}, {n0}|{s0}|, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @SUBB_UINT.01(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00001010011 00 1 0 0 1 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="SUBB_UINT {last}{dst}, {n0}{s0}, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @SUBB_UINT.11(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00001010011 00 1 0 0 1 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="SUBB_UINT {last}{dst}, {n0}|{s0}|, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @GROUP_BARRIER.00(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00001010100 00 1 0 0 0 0 last:1 ps:2 000 0000000000000 0000000000000" "asm"="GROUP_BARRIER {last}{dst},{ps}{bs}" "asmonly" {
  ret void
}
define void @GROUP_SEQ_BEGIN.00(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00001010101 00 1 0 0 0 0 last:1 ps:2 000 0000000000000 0000000000000" "asm"="GROUP_SEQ_BEGIN {last}{dst},{ps}{bs}" "asmonly" {
  ret void
}
define void @GROUP_SEQ_END.00(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00001010110 00 1 0 0 0 0 last:1 ps:2 000 0000000000000 0000000000000" "asm"="GROUP_SEQ_END {last}{dst},{ps}{bs}" "asmonly" {
  ret void
}
define void @SET_MODE.00(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00001010111 00 1 0 0 0 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="SET_MODE {last}{dst}, {n0}{s0}, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @SET_MODE.10(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00001010111 00 1 0 0 0 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="SET_MODE {last}{dst}, {n0}|{s0}|, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @SET_MODE.01(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00001010111 00 1 0 0 1 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="SET_MODE {last}{dst}, {n0}{s0}, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @SET_MODE.11(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00001010111 00 1 0 0 1 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="SET_MODE {last}{dst}, {n0}|{s0}|, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @SET_CF_IDX0.00(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00001011000 00 1 0 0 0 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="SET_CF_IDX0 {last}{dst}, {n0}{s0}, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @SET_CF_IDX0.10(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00001011000 00 1 0 0 0 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="SET_CF_IDX0 {last}{dst}, {n0}|{s0}|, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @SET_CF_IDX0.01(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00001011000 00 1 0 0 1 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="SET_CF_IDX0 {last}{dst}, {n0}{s0}, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @SET_CF_IDX0.11(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00001011000 00 1 0 0 1 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="SET_CF_IDX0 {last}{dst}, {n0}|{s0}|, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @SET_CF_IDX1.00(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00001011001 00 1 0 0 0 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="SET_CF_IDX1 {last}{dst}, {n0}{s0}, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @SET_CF_IDX1.10(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00001011001 00 1 0 0 0 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="SET_CF_IDX1 {last}{dst}, {n0}|{s0}|, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @SET_CF_IDX1.01(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00001011001 00 1 0 0 1 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="SET_CF_IDX1 {last}{dst}, {n0}{s0}, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @SET_CF_IDX1.11(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00001011001 00 1 0 0 1 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="SET_CF_IDX1 {last}{dst}, {n0}|{s0}|, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @SET_LDS_SIZE.00(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00001011010 00 1 0 0 0 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="SET_LDS_SIZE {last}{dst}, {n0}{s0}, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @SET_LDS_SIZE.10(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00001011010 00 1 0 0 0 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="SET_LDS_SIZE {last}{dst}, {n0}|{s0}|, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @SET_LDS_SIZE.01(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00001011010 00 1 0 0 1 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="SET_LDS_SIZE {last}{dst}, {n0}{s0}, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @SET_LDS_SIZE.11(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00001011010 00 1 0 0 1 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="SET_LDS_SIZE {last}{dst}, {n0}|{s0}|, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @EXP_IEEE.00(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00010000001 00 1 0 0 0 0 last:1 ps:2 000 0000000000000 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="EXP_IEEE {last}{dst}, {n0}{s0},{ps}{bs}" "asmonly" {
  ret void
}
define void @EXP_IEEE.10(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00010000001 00 1 0 0 0 1 last:1 ps:2 000 0000000000000 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="EXP_IEEE {last}{dst}, {n0}|{s0}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @LOG_CLAMPED.00(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00010000010 00 1 0 0 0 0 last:1 ps:2 000 0000000000000 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="LOG_CLAMPED {last}{dst}, {n0}{s0},{ps}{bs}" "asmonly" {
  ret void
}
define void @LOG_CLAMPED.10(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00010000010 00 1 0 0 0 1 last:1 ps:2 000 0000000000000 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="LOG_CLAMPED {last}{dst}, {n0}|{s0}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @LOG_IEEE.00(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00010000011 00 1 0 0 0 0 last:1 ps:2 000 0000000000000 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="LOG_IEEE {last}{dst}, {n0}{s0},{ps}{bs}" "asmonly" {
  ret void
}
define void @LOG_IEEE.10(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00010000011 00 1 0 0 0 1 last:1 ps:2 000 0000000000000 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="LOG_IEEE {last}{dst}, {n0}|{s0}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @RECIP_CLAMPED.00(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00010000100 00 1 0 0 0 0 last:1 ps:2 000 0000000000000 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="RECIP_CLAMPED {last}{dst}, {n0}{s0},{ps}{bs}" "asmonly" {
  ret void
}
define void @RECIP_CLAMPED.10(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00010000100 00 1 0 0 0 1 last:1 ps:2 000 0000000000000 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="RECIP_CLAMPED {last}{dst}, {n0}|{s0}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @RECIP_FF.00(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00010000101 00 1 0 0 0 0 last:1 ps:2 000 0000000000000 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="RECIP_FF {last}{dst}, {n0}{s0},{ps}{bs}" "asmonly" {
  ret void
}
define void @RECIP_FF.10(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00010000101 00 1 0 0 0 1 last:1 ps:2 000 0000000000000 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="RECIP_FF {last}{dst}, {n0}|{s0}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @RECIP_IEEE.00(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00010000110 00 1 0 0 0 0 last:1 ps:2 000 0000000000000 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="RECIP_IEEE {last}{dst}, {n0}{s0},{ps}{bs}" "asmonly" {
  ret void
}
define void @RECIP_IEEE.10(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00010000110 00 1 0 0 0 1 last:1 ps:2 000 0000000000000 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="RECIP_IEEE {last}{dst}, {n0}|{s0}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @RECIPSQRT_CLAMPED.00(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00010000111 00 1 0 0 0 0 last:1 ps:2 000 0000000000000 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="RECIPSQRT_CLAMPED {last}{dst}, {n0}{s0},{ps}{bs}" "asmonly" {
  ret void
}
define void @RECIPSQRT_CLAMPED.10(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00010000111 00 1 0 0 0 1 last:1 ps:2 000 0000000000000 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="RECIPSQRT_CLAMPED {last}{dst}, {n0}|{s0}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @RECIPSQRT_FF.00(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00010001000 00 1 0 0 0 0 last:1 ps:2 000 0000000000000 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="RECIPSQRT_FF {last}{dst}, {n0}{s0},{ps}{bs}" "asmonly" {
  ret void
}
define void @RECIPSQRT_FF.10(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00010001000 00 1 0 0 0 1 last:1 ps:2 000 0000000000000 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="RECIPSQRT_FF {last}{dst}, {n0}|{s0}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @RECIPSQRT_IEEE.00(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00010001001 00 1 0 0 0 0 last:1 ps:2 000 0000000000000 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="RECIPSQRT_IEEE {last}{dst}, {n0}{s0},{ps}{bs}" "asmonly" {
  ret void
}
define void @RECIPSQRT_IEEE.10(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00010001001 00 1 0 0 0 1 last:1 ps:2 000 0000000000000 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="RECIPSQRT_IEEE {last}{dst}, {n0}|{s0}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @SQRT_IEEE.00(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00010001010 00 1 0 0 0 0 last:1 ps:2 000 0000000000000 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="SQRT_IEEE {last}{dst}, {n0}{s0},{ps}{bs}" "asmonly" {
  ret void
}
define void @SQRT_IEEE.10(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00010001010 00 1 0 0 0 1 last:1 ps:2 000 0000000000000 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="SQRT_IEEE {last}{dst}, {n0}|{s0}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @SIN.00(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00010001101 00 1 0 0 0 0 last:1 ps:2 000 0000000000000 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="SIN {last}{dst}, {n0}{s0},{ps}{bs}" "asmonly" {
  ret void
}
define void @SIN.10(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00010001101 00 1 0 0 0 1 last:1 ps:2 000 0000000000000 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="SIN {last}{dst}, {n0}|{s0}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @COS.00(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00010001110 00 1 0 0 0 0 last:1 ps:2 000 0000000000000 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="COS {last}{dst}, {n0}{s0},{ps}{bs}" "asmonly" {
  ret void
}
define void @COS.10(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00010001110 00 1 0 0 0 1 last:1 ps:2 000 0000000000000 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="COS {last}{dst}, {n0}|{s0}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @MULLO_INT.00(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00010001111 00 1 0 0 0 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="MULLO_INT {last}{dst}, {n0}{s0}, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @MULLO_INT.10(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00010001111 00 1 0 0 0 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="MULLO_INT {last}{dst}, {n0}|{s0}|, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @MULLO_INT.01(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00010001111 00 1 0 0 1 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="MULLO_INT {last}{dst}, {n0}{s0}, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @MULLO_INT.11(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00010001111 00 1 0 0 1 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="MULLO_INT {last}{dst}, {n0}|{s0}|, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @MULHI_INT.00(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00010010000 00 1 0 0 0 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="MULHI_INT {last}{dst}, {n0}{s0}, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @MULHI_INT.10(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00010010000 00 1 0 0 0 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="MULHI_INT {last}{dst}, {n0}|{s0}|, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @MULHI_INT.01(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00010010000 00 1 0 0 1 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="MULHI_INT {last}{dst}, {n0}{s0}, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @MULHI_INT.11(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00010010000 00 1 0 0 1 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="MULHI_INT {last}{dst}, {n0}|{s0}|, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @MULLO_UINT.00(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00010010001 00 1 0 0 0 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="MULLO_UINT {last}{dst}, {n0}{s0}, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @MULLO_UINT.10(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00010010001 00 1 0 0 0 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="MULLO_UINT {last}{dst}, {n0}|{s0}|, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @MULLO_UINT.01(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00010010001 00 1 0 0 1 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="MULLO_UINT {last}{dst}, {n0}{s0}, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @MULLO_UINT.11(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00010010001 00 1 0 0 1 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="MULLO_UINT {last}{dst}, {n0}|{s0}|, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @MULHI_UINT.00(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00010010010 00 1 0 0 0 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="MULHI_UINT {last}{dst}, {n0}{s0}, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @MULHI_UINT.10(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00010010010 00 1 0 0 0 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="MULHI_UINT {last}{dst}, {n0}|{s0}|, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @MULHI_UINT.01(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00010010010 00 1 0 0 1 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="MULHI_UINT {last}{dst}, {n0}{s0}, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @MULHI_UINT.11(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00010010010 00 1 0 0 1 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="MULHI_UINT {last}{dst}, {n0}|{s0}|, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @RECIP_INT.00(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00010010011 00 1 0 0 0 0 last:1 ps:2 000 0000000000000 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="RECIP_INT {last}{dst}, {n0}{s0},{ps}{bs}" "asmonly" {
  ret void
}
define void @RECIP_INT.10(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00010010011 00 1 0 0 0 1 last:1 ps:2 000 0000000000000 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="RECIP_INT {last}{dst}, {n0}|{s0}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @RECIP_UINT.00(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00010010100 00 1 0 0 0 0 last:1 ps:2 000 0000000000000 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="RECIP_UINT {last}{dst}, {n0}{s0},{ps}{bs}" "asmonly" {
  ret void
}
define void @RECIP_UINT.10(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00010010100 00 1 0 0 0 1 last:1 ps:2 000 0000000000000 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="RECIP_UINT {last}{dst}, {n0}|{s0}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @RECIP_64.00(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00010010101 00 1 0 0 0 0 last:1 ps:2 000 0000000000000 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="RECIP_64 {last}{dst}, {n0}{s0},{ps}{bs}" "asmonly" {
  ret void
}
define void @RECIP_64.10(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00010010101 00 1 0 0 0 1 last:1 ps:2 000 0000000000000 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="RECIP_64 {last}{dst}, {n0}|{s0}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @RECIP_CLAMPED_64.00(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00010010110 00 1 0 0 0 0 last:1 ps:2 000 0000000000000 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="RECIP_CLAMPED_64 {last}{dst}, {n0}{s0},{ps}{bs}" "asmonly" {
  ret void
}
define void @RECIP_CLAMPED_64.10(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00010010110 00 1 0 0 0 1 last:1 ps:2 000 0000000000000 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="RECIP_CLAMPED_64 {last}{dst}, {n0}|{s0}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @RECIPSQRT_64.00(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00010010111 00 1 0 0 0 0 last:1 ps:2 000 0000000000000 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="RECIPSQRT_64 {last}{dst}, {n0}{s0},{ps}{bs}" "asmonly" {
  ret void
}
define void @RECIPSQRT_64.10(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00010010111 00 1 0 0 0 1 last:1 ps:2 000 0000000000000 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="RECIPSQRT_64 {last}{dst}, {n0}|{s0}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @RECIPSQRT_CLAMPED_64.00(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00010011000 00 1 0 0 0 0 last:1 ps:2 000 0000000000000 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="RECIPSQRT_CLAMPED_64 {last}{dst}, {n0}{s0},{ps}{bs}" "asmonly" {
  ret void
}
define void @RECIPSQRT_CLAMPED_64.10(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00010011000 00 1 0 0 0 1 last:1 ps:2 000 0000000000000 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="RECIPSQRT_CLAMPED_64 {last}{dst}, {n0}|{s0}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @SQRT_64.00(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00010011001 00 1 0 0 0 0 last:1 ps:2 000 0000000000000 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="SQRT_64 {last}{dst}, {n0}{s0},{ps}{bs}" "asmonly" {
  ret void
}
define void @SQRT_64.10(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00010011001 00 1 0 0 0 1 last:1 ps:2 000 0000000000000 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="SQRT_64 {last}{dst}, {n0}|{s0}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @FLT_TO_UINT.00(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00010011010 00 1 0 0 0 0 last:1 ps:2 000 0000000000000 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="FLT_TO_UINT {last}{dst}, {n0}{s0},{ps}{bs}" "asmonly" {
  ret void
}
define void @FLT_TO_UINT.10(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00010011010 00 1 0 0 0 1 last:1 ps:2 000 0000000000000 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="FLT_TO_UINT {last}{dst}, {n0}|{s0}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @INT_TO_FLT.00(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00010011011 00 1 0 0 0 0 last:1 ps:2 000 0000000000000 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="INT_TO_FLT {last}{dst}, {n0}{s0},{ps}{bs}" "asmonly" {
  ret void
}
define void @INT_TO_FLT.10(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00010011011 00 1 0 0 0 1 last:1 ps:2 000 0000000000000 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="INT_TO_FLT {last}{dst}, {n0}|{s0}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @UINT_TO_FLT.00(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00010011100 00 1 0 0 0 0 last:1 ps:2 000 0000000000000 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="UINT_TO_FLT {last}{dst}, {n0}{s0},{ps}{bs}" "asmonly" {
  ret void
}
define void @UINT_TO_FLT.10(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00010011100 00 1 0 0 0 1 last:1 ps:2 000 0000000000000 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="UINT_TO_FLT {last}{dst}, {n0}|{s0}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @BFM_INT.00(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00010100000 00 1 0 0 0 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="BFM_INT {last}{dst}, {n0}{s0}, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @BFM_INT.10(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00010100000 00 1 0 0 0 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="BFM_INT {last}{dst}, {n0}|{s0}|, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @BFM_INT.01(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00010100000 00 1 0 0 1 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="BFM_INT {last}{dst}, {n0}{s0}, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @BFM_INT.11(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00010100000 00 1 0 0 1 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="BFM_INT {last}{dst}, {n0}|{s0}|, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @FLT32_TO_FLT16.00(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00010100010 00 1 0 0 0 0 last:1 ps:2 000 0000000000000 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="FLT32_TO_FLT16 {last}{dst}, {n0}{s0},{ps}{bs}" "asmonly" {
  ret void
}
define void @FLT32_TO_FLT16.10(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00010100010 00 1 0 0 0 1 last:1 ps:2 000 0000000000000 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="FLT32_TO_FLT16 {last}{dst}, {n0}|{s0}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @FLT16_TO_FLT32.00(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00010100011 00 1 0 0 0 0 last:1 ps:2 000 0000000000000 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="FLT16_TO_FLT32 {last}{dst}, {n0}{s0},{ps}{bs}" "asmonly" {
  ret void
}
define void @FLT16_TO_FLT32.10(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00010100011 00 1 0 0 0 1 last:1 ps:2 000 0000000000000 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="FLT16_TO_FLT32 {last}{dst}, {n0}|{s0}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @UBYTE0_FLT.00(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00010100100 00 1 0 0 0 0 last:1 ps:2 000 0000000000000 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="UBYTE0_FLT {last}{dst}, {n0}{s0},{ps}{bs}" "asmonly" {
  ret void
}
define void @UBYTE0_FLT.10(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00010100100 00 1 0 0 0 1 last:1 ps:2 000 0000000000000 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="UBYTE0_FLT {last}{dst}, {n0}|{s0}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @UBYTE1_FLT.00(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00010100101 00 1 0 0 0 0 last:1 ps:2 000 0000000000000 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="UBYTE1_FLT {last}{dst}, {n0}{s0},{ps}{bs}" "asmonly" {
  ret void
}
define void @UBYTE1_FLT.10(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00010100101 00 1 0 0 0 1 last:1 ps:2 000 0000000000000 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="UBYTE1_FLT {last}{dst}, {n0}|{s0}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @UBYTE2_FLT.00(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00010100110 00 1 0 0 0 0 last:1 ps:2 000 0000000000000 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="UBYTE2_FLT {last}{dst}, {n0}{s0},{ps}{bs}" "asmonly" {
  ret void
}
define void @UBYTE2_FLT.10(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00010100110 00 1 0 0 0 1 last:1 ps:2 000 0000000000000 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="UBYTE2_FLT {last}{dst}, {n0}|{s0}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @UBYTE3_FLT.00(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00010100111 00 1 0 0 0 0 last:1 ps:2 000 0000000000000 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="UBYTE3_FLT {last}{dst}, {n0}{s0},{ps}{bs}" "asmonly" {
  ret void
}
define void @UBYTE3_FLT.10(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00010100111 00 1 0 0 0 1 last:1 ps:2 000 0000000000000 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="UBYTE3_FLT {last}{dst}, {n0}|{s0}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @BCNT_INT.00(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00010101010 00 1 0 0 0 0 last:1 ps:2 000 0000000000000 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="BCNT_INT {last}{dst}, {n0}{s0},{ps}{bs}" "asmonly" {
  ret void
}
define void @BCNT_INT.10(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00010101010 00 1 0 0 0 1 last:1 ps:2 000 0000000000000 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="BCNT_INT {last}{dst}, {n0}|{s0}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @FFBH_UINT.00(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00010101011 00 1 0 0 0 0 last:1 ps:2 000 0000000000000 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="FFBH_UINT {last}{dst}, {n0}{s0},{ps}{bs}" "asmonly" {
  ret void
}
define void @FFBH_UINT.10(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00010101011 00 1 0 0 0 1 last:1 ps:2 000 0000000000000 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="FFBH_UINT {last}{dst}, {n0}|{s0}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @FFBL_INT.00(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00010101100 00 1 0 0 0 0 last:1 ps:2 000 0000000000000 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="FFBL_INT {last}{dst}, {n0}{s0},{ps}{bs}" "asmonly" {
  ret void
}
define void @FFBL_INT.10(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00010101100 00 1 0 0 0 1 last:1 ps:2 000 0000000000000 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="FFBL_INT {last}{dst}, {n0}|{s0}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @FFBH_INT.00(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00010101101 00 1 0 0 0 0 last:1 ps:2 000 0000000000000 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="FFBH_INT {last}{dst}, {n0}{s0},{ps}{bs}" "asmonly" {
  ret void
}
define void @FFBH_INT.10(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00010101101 00 1 0 0 0 1 last:1 ps:2 000 0000000000000 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="FFBH_INT {last}{dst}, {n0}|{s0}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @FLT_TO_UINT4.00(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00010101110 00 1 0 0 0 0 last:1 ps:2 000 0000000000000 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="FLT_TO_UINT4 {last}{dst}, {n0}{s0},{ps}{bs}" "asmonly" {
  ret void
}
define void @FLT_TO_UINT4.10(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00010101110 00 1 0 0 0 1 last:1 ps:2 000 0000000000000 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="FLT_TO_UINT4 {last}{dst}, {n0}|{s0}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @DOT_IEEE.00(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00010101111 00 1 0 0 0 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="DOT_IEEE {last}{dst}, {n0}{s0}, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @DOT_IEEE.10(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00010101111 00 1 0 0 0 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="DOT_IEEE {last}{dst}, {n0}|{s0}|, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @DOT_IEEE.01(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00010101111 00 1 0 0 1 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="DOT_IEEE {last}{dst}, {n0}{s0}, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @DOT_IEEE.11(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00010101111 00 1 0 0 1 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="DOT_IEEE {last}{dst}, {n0}|{s0}|, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @FLT_TO_INT_RPI.00(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00010110000 00 1 0 0 0 0 last:1 ps:2 000 0000000000000 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="FLT_TO_INT_RPI {last}{dst}, {n0}{s0},{ps}{bs}" "asmonly" {
  ret void
}
define void @FLT_TO_INT_RPI.10(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00010110000 00 1 0 0 0 1 last:1 ps:2 000 0000000000000 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="FLT_TO_INT_RPI {last}{dst}, {n0}|{s0}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @FLT_TO_INT_FLOOR.00(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00010110001 00 1 0 0 0 0 last:1 ps:2 000 0000000000000 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="FLT_TO_INT_FLOOR {last}{dst}, {n0}{s0},{ps}{bs}" "asmonly" {
  ret void
}
define void @FLT_TO_INT_FLOOR.10(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00010110001 00 1 0 0 0 1 last:1 ps:2 000 0000000000000 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="FLT_TO_INT_FLOOR {last}{dst}, {n0}|{s0}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @MULHI_UINT24.00(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00010110010 00 1 0 0 0 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="MULHI_UINT24 {last}{dst}, {n0}{s0}, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @MULHI_UINT24.10(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00010110010 00 1 0 0 0 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="MULHI_UINT24 {last}{dst}, {n0}|{s0}|, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @MULHI_UINT24.01(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00010110010 00 1 0 0 1 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="MULHI_UINT24 {last}{dst}, {n0}{s0}, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @MULHI_UINT24.11(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00010110010 00 1 0 0 1 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="MULHI_UINT24 {last}{dst}, {n0}|{s0}|, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @MBCNT_32HI_INT.00(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00010110011 00 1 0 0 0 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="MBCNT_32HI_INT {last}{dst}, {n0}{s0}, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @MBCNT_32HI_INT.10(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00010110011 00 1 0 0 0 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="MBCNT_32HI_INT {last}{dst}, {n0}|{s0}|, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @MBCNT_32HI_INT.01(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00010110011 00 1 0 0 1 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="MBCNT_32HI_INT {last}{dst}, {n0}{s0}, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @MBCNT_32HI_INT.11(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00010110011 00 1 0 0 1 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="MBCNT_32HI_INT {last}{dst}, {n0}|{s0}|, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @OFFSET_TO_FLT.00(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00010110100 00 1 0 0 0 0 last:1 ps:2 000 0000000000000 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="OFFSET_TO_FLT {last}{dst}, {n0}{s0},{ps}{bs}" "asmonly" {
  ret void
}
define void @OFFSET_TO_FLT.10(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00010110100 00 1 0 0 0 1 last:1 ps:2 000 0000000000000 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="OFFSET_TO_FLT {last}{dst}, {n0}|{s0}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @MUL_UINT24.00(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00010110101 00 1 0 0 0 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="MUL_UINT24 {last}{dst}, {n0}{s0}, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @MUL_UINT24.10(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00010110101 00 1 0 0 0 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="MUL_UINT24 {last}{dst}, {n0}|{s0}|, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @MUL_UINT24.01(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00010110101 00 1 0 0 1 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="MUL_UINT24 {last}{dst}, {n0}{s0}, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @MUL_UINT24.11(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00010110101 00 1 0 0 1 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="MUL_UINT24 {last}{dst}, {n0}|{s0}|, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @BCNT_ACCUM_PREV_INT.00(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00010110110 00 1 0 0 0 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="BCNT_ACCUM_PREV_INT {last}{dst}, {n0}{s0}, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @BCNT_ACCUM_PREV_INT.10(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00010110110 00 1 0 0 0 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="BCNT_ACCUM_PREV_INT {last}{dst}, {n0}|{s0}|, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @BCNT_ACCUM_PREV_INT.01(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00010110110 00 1 0 0 1 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="BCNT_ACCUM_PREV_INT {last}{dst}, {n0}{s0}, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @BCNT_ACCUM_PREV_INT.11(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00010110110 00 1 0 0 1 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="BCNT_ACCUM_PREV_INT {last}{dst}, {n0}|{s0}|, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @MBCNT_32LO_ACCUM_PREV_INT.00(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00010110111 00 1 0 0 0 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="MBCNT_32LO_ACCUM_PREV_INT {last}{dst}, {n0}{s0}, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @MBCNT_32LO_ACCUM_PREV_INT.10(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00010110111 00 1 0 0 0 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="MBCNT_32LO_ACCUM_PREV_INT {last}{dst}, {n0}|{s0}|, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @MBCNT_32LO_ACCUM_PREV_INT.01(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00010110111 00 1 0 0 1 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="MBCNT_32LO_ACCUM_PREV_INT {last}{dst}, {n0}{s0}, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @MBCNT_32LO_ACCUM_PREV_INT.11(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00010110111 00 1 0 0 1 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="MBCNT_32LO_ACCUM_PREV_INT {last}{dst}, {n0}|{s0}|, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @SETE_64.00(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00010111000 00 1 0 0 0 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="SETE_64 {last}{dst}, {n0}{s0}, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @SETE_64.10(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00010111000 00 1 0 0 0 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="SETE_64 {last}{dst}, {n0}|{s0}|, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @SETE_64.01(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00010111000 00 1 0 0 1 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="SETE_64 {last}{dst}, {n0}{s0}, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @SETE_64.11(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00010111000 00 1 0 0 1 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="SETE_64 {last}{dst}, {n0}|{s0}|, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @SETNE_64.00(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00010111001 00 1 0 0 0 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="SETNE_64 {last}{dst}, {n0}{s0}, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @SETNE_64.10(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00010111001 00 1 0 0 0 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="SETNE_64 {last}{dst}, {n0}|{s0}|, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @SETNE_64.01(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00010111001 00 1 0 0 1 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="SETNE_64 {last}{dst}, {n0}{s0}, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @SETNE_64.11(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00010111001 00 1 0 0 1 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="SETNE_64 {last}{dst}, {n0}|{s0}|, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @SETGT_64.00(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00010111010 00 1 0 0 0 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="SETGT_64 {last}{dst}, {n0}{s0}, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @SETGT_64.10(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00010111010 00 1 0 0 0 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="SETGT_64 {last}{dst}, {n0}|{s0}|, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @SETGT_64.01(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00010111010 00 1 0 0 1 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="SETGT_64 {last}{dst}, {n0}{s0}, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @SETGT_64.11(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00010111010 00 1 0 0 1 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="SETGT_64 {last}{dst}, {n0}|{s0}|, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @SETGE_64.00(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00010111011 00 1 0 0 0 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="SETGE_64 {last}{dst}, {n0}{s0}, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @SETGE_64.10(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00010111011 00 1 0 0 0 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="SETGE_64 {last}{dst}, {n0}|{s0}|, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @SETGE_64.01(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00010111011 00 1 0 0 1 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="SETGE_64 {last}{dst}, {n0}{s0}, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @SETGE_64.11(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00010111011 00 1 0 0 1 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="SETGE_64 {last}{dst}, {n0}|{s0}|, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @MIN_64.00(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00010111100 00 1 0 0 0 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="MIN_64 {last}{dst}, {n0}{s0}, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @MIN_64.10(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00010111100 00 1 0 0 0 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="MIN_64 {last}{dst}, {n0}|{s0}|, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @MIN_64.01(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00010111100 00 1 0 0 1 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="MIN_64 {last}{dst}, {n0}{s0}, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @MIN_64.11(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00010111100 00 1 0 0 1 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="MIN_64 {last}{dst}, {n0}|{s0}|, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @MAX_64.00(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00010111101 00 1 0 0 0 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="MAX_64 {last}{dst}, {n0}{s0}, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @MAX_64.10(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00010111101 00 1 0 0 0 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="MAX_64 {last}{dst}, {n0}|{s0}|, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @MAX_64.01(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00010111101 00 1 0 0 1 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="MAX_64 {last}{dst}, {n0}{s0}, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @MAX_64.11(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00010111101 00 1 0 0 1 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="MAX_64 {last}{dst}, {n0}|{s0}|, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @DOT4.00(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00010111110 00 1 0 0 0 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="DOT4 {last}{dst}, {n0}{s0}, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @DOT4.10(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00010111110 00 1 0 0 0 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="DOT4 {last}{dst}, {n0}|{s0}|, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @DOT4.01(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00010111110 00 1 0 0 1 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="DOT4 {last}{dst}, {n0}{s0}, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @DOT4.11(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00010111110 00 1 0 0 1 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="DOT4 {last}{dst}, {n0}|{s0}|, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @DOT4_IEEE.00(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00010111111 00 1 0 0 0 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="DOT4_IEEE {last}{dst}, {n0}{s0}, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @DOT4_IEEE.10(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00010111111 00 1 0 0 0 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="DOT4_IEEE {last}{dst}, {n0}|{s0}|, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @DOT4_IEEE.01(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00010111111 00 1 0 0 1 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="DOT4_IEEE {last}{dst}, {n0}{s0}, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @DOT4_IEEE.11(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00010111111 00 1 0 0 1 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="DOT4_IEEE {last}{dst}, {n0}|{s0}|, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @CUBE.00(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00011000000 00 1 0 0 0 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="CUBE {last}{dst}, {n0}{s0}, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @CUBE.10(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00011000000 00 1 0 0 0 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="CUBE {last}{dst}, {n0}|{s0}|, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @CUBE.01(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00011000000 00 1 0 0 1 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="CUBE {last}{dst}, {n0}{s0}, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @CUBE.11(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00011000000 00 1 0 0 1 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="CUBE {last}{dst}, {n0}|{s0}|, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @MAX4.00(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00011000001 00 1 0 0 0 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="MAX4 {last}{dst}, {n0}{s0}, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @MAX4.10(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00011000001 00 1 0 0 0 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="MAX4 {last}{dst}, {n0}|{s0}|, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @MAX4.01(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00011000001 00 1 0 0 1 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="MAX4 {last}{dst}, {n0}{s0}, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @MAX4.11(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00011000001 00 1 0 0 1 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="MAX4 {last}{dst}, {n0}|{s0}|, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @FREXP_64.00(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00011000100 00 1 0 0 0 0 last:1 ps:2 000 0000000000000 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="FREXP_64 {last}{dst}, {n0}{s0},{ps}{bs}" "asmonly" {
  ret void
}
define void @FREXP_64.10(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00011000100 00 1 0 0 0 1 last:1 ps:2 000 0000000000000 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="FREXP_64 {last}{dst}, {n0}|{s0}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @LDEXP_64.00(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00011000101 00 1 0 0 0 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="LDEXP_64 {last}{dst}, {n0}{s0}, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @LDEXP_64.10(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00011000101 00 1 0 0 0 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="LDEXP_64 {last}{dst}, {n0}|{s0}|, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @LDEXP_64.01(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00011000101 00 1 0 0 1 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="LDEXP_64 {last}{dst}, {n0}{s0}, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @LDEXP_64.11(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00011000101 00 1 0 0 1 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="LDEXP_64 {last}{dst}, {n0}|{s0}|, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @FRACT_64.00(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00011000110 00 1 0 0 0 0 last:1 ps:2 000 0000000000000 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="FRACT_64 {last}{dst}, {n0}{s0},{ps}{bs}" "asmonly" {
  ret void
}
define void @FRACT_64.10(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00011000110 00 1 0 0 0 1 last:1 ps:2 000 0000000000000 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="FRACT_64 {last}{dst}, {n0}|{s0}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SETGT_64.00(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00011000111 00 1 0 0 0 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SETGT_64 {last}{dst}, {n0}{s0}, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SETGT_64.pred.00(i32 "enum"="last" %last, i32 "enum"="pdst" %pdst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 00 0 0000000 bs:3 00011000111 00 0 up:1=pdst>>1 uem:1=pdst&1 0 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SETGT_64 {last}{pdst}, {n0}{s0}, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SETGT_64.10(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00011000111 00 1 0 0 0 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SETGT_64 {last}{dst}, {n0}|{s0}|, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SETGT_64.pred.10(i32 "enum"="last" %last, i32 "enum"="pdst" %pdst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 00 0 0000000 bs:3 00011000111 00 0 up:1=pdst>>1 uem:1=pdst&1 0 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SETGT_64 {last}{pdst}, {n0}|{s0}|, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SETGT_64.01(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00011000111 00 1 0 0 1 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SETGT_64 {last}{dst}, {n0}{s0}, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SETGT_64.pred.01(i32 "enum"="last" %last, i32 "enum"="pdst" %pdst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 00 0 0000000 bs:3 00011000111 00 0 up:1=pdst>>1 uem:1=pdst&1 1 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SETGT_64 {last}{pdst}, {n0}{s0}, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SETGT_64.11(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00011000111 00 1 0 0 1 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SETGT_64 {last}{dst}, {n0}|{s0}|, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SETGT_64.pred.11(i32 "enum"="last" %last, i32 "enum"="pdst" %pdst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 00 0 0000000 bs:3 00011000111 00 0 up:1=pdst>>1 uem:1=pdst&1 1 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SETGT_64 {last}{pdst}, {n0}|{s0}|, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SETE_64.00(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00011001000 00 1 0 0 0 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SETE_64 {last}{dst}, {n0}{s0}, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SETE_64.pred.00(i32 "enum"="last" %last, i32 "enum"="pdst" %pdst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 00 0 0000000 bs:3 00011001000 00 0 up:1=pdst>>1 uem:1=pdst&1 0 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SETE_64 {last}{pdst}, {n0}{s0}, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SETE_64.10(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00011001000 00 1 0 0 0 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SETE_64 {last}{dst}, {n0}|{s0}|, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SETE_64.pred.10(i32 "enum"="last" %last, i32 "enum"="pdst" %pdst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 00 0 0000000 bs:3 00011001000 00 0 up:1=pdst>>1 uem:1=pdst&1 0 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SETE_64 {last}{pdst}, {n0}|{s0}|, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SETE_64.01(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00011001000 00 1 0 0 1 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SETE_64 {last}{dst}, {n0}{s0}, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SETE_64.pred.01(i32 "enum"="last" %last, i32 "enum"="pdst" %pdst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 00 0 0000000 bs:3 00011001000 00 0 up:1=pdst>>1 uem:1=pdst&1 1 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SETE_64 {last}{pdst}, {n0}{s0}, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SETE_64.11(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00011001000 00 1 0 0 1 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SETE_64 {last}{dst}, {n0}|{s0}|, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SETE_64.pred.11(i32 "enum"="last" %last, i32 "enum"="pdst" %pdst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 00 0 0000000 bs:3 00011001000 00 0 up:1=pdst>>1 uem:1=pdst&1 1 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SETE_64 {last}{pdst}, {n0}|{s0}|, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SETGE_64.00(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00011001001 00 1 0 0 0 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SETGE_64 {last}{dst}, {n0}{s0}, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SETGE_64.pred.00(i32 "enum"="last" %last, i32 "enum"="pdst" %pdst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 00 0 0000000 bs:3 00011001001 00 0 up:1=pdst>>1 uem:1=pdst&1 0 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SETGE_64 {last}{pdst}, {n0}{s0}, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SETGE_64.10(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00011001001 00 1 0 0 0 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SETGE_64 {last}{dst}, {n0}|{s0}|, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SETGE_64.pred.10(i32 "enum"="last" %last, i32 "enum"="pdst" %pdst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 00 0 0000000 bs:3 00011001001 00 0 up:1=pdst>>1 uem:1=pdst&1 0 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SETGE_64 {last}{pdst}, {n0}|{s0}|, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SETGE_64.01(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00011001001 00 1 0 0 1 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SETGE_64 {last}{dst}, {n0}{s0}, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SETGE_64.pred.01(i32 "enum"="last" %last, i32 "enum"="pdst" %pdst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 00 0 0000000 bs:3 00011001001 00 0 up:1=pdst>>1 uem:1=pdst&1 1 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SETGE_64 {last}{pdst}, {n0}{s0}, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SETGE_64.11(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00011001001 00 1 0 0 1 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SETGE_64 {last}{dst}, {n0}|{s0}|, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @PRED_SETGE_64.pred.11(i32 "enum"="last" %last, i32 "enum"="pdst" %pdst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 00 0 0000000 bs:3 00011001001 00 0 up:1=pdst>>1 uem:1=pdst&1 1 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="PRED_SETGE_64 {last}{pdst}, {n0}|{s0}|, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @ADD_64.00(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00011001011 00 1 0 0 0 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="ADD_64 {last}{dst}, {n0}{s0}, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @ADD_64.10(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00011001011 00 1 0 0 0 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="ADD_64 {last}{dst}, {n0}|{s0}|, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @ADD_64.01(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00011001011 00 1 0 0 1 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="ADD_64 {last}{dst}, {n0}{s0}, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @ADD_64.11(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00011001011 00 1 0 0 1 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="ADD_64 {last}{dst}, {n0}|{s0}|, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @MOVA_INT.00(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00011001100 00 1 0 0 0 0 last:1 ps:2 000 0000000000000 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="MOVA_INT {last}{dst}, {n0}{s0},{ps}{bs}" "asmonly" {
  ret void
}
define void @MOVA_INT.10(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00011001100 00 1 0 0 0 1 last:1 ps:2 000 0000000000000 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="MOVA_INT {last}{dst}, {n0}|{s0}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @SAD_ACCUM_PREV_UINT.00(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00011001111 00 1 0 0 0 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="SAD_ACCUM_PREV_UINT {last}{dst}, {n0}{s0}, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @SAD_ACCUM_PREV_UINT.10(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00011001111 00 1 0 0 0 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="SAD_ACCUM_PREV_UINT {last}{dst}, {n0}|{s0}|, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @SAD_ACCUM_PREV_UINT.01(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00011001111 00 1 0 0 1 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="SAD_ACCUM_PREV_UINT {last}{dst}, {n0}{s0}, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @SAD_ACCUM_PREV_UINT.11(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00011001111 00 1 0 0 1 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="SAD_ACCUM_PREV_UINT {last}{dst}, {n0}|{s0}|, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @DOT.00(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00011010000 00 1 0 0 0 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="DOT {last}{dst}, {n0}{s0}, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @DOT.10(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00011010000 00 1 0 0 0 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="DOT {last}{dst}, {n0}|{s0}|, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @DOT.01(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00011010000 00 1 0 0 1 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="DOT {last}{dst}, {n0}{s0}, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @DOT.11(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00011010000 00 1 0 0 1 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="DOT {last}{dst}, {n0}|{s0}|, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @MUL_PREV.00(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00011010001 00 1 0 0 0 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="MUL_PREV {last}{dst}, {n0}{s0}, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @MUL_PREV.10(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00011010001 00 1 0 0 0 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="MUL_PREV {last}{dst}, {n0}|{s0}|, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @MUL_PREV.01(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00011010001 00 1 0 0 1 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="MUL_PREV {last}{dst}, {n0}{s0}, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @MUL_PREV.11(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00011010001 00 1 0 0 1 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="MUL_PREV {last}{dst}, {n0}|{s0}|, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @MUL_IEEE_PREV.00(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00011010010 00 1 0 0 0 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="MUL_IEEE_PREV {last}{dst}, {n0}{s0}, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @MUL_IEEE_PREV.10(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00011010010 00 1 0 0 0 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="MUL_IEEE_PREV {last}{dst}, {n0}|{s0}|, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @MUL_IEEE_PREV.01(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00011010010 00 1 0 0 1 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="MUL_IEEE_PREV {last}{dst}, {n0}{s0}, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @MUL_IEEE_PREV.11(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00011010010 00 1 0 0 1 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="MUL_IEEE_PREV {last}{dst}, {n0}|{s0}|, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @ADD_PREV.00(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00011010011 00 1 0 0 0 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="ADD_PREV {last}{dst}, {n0}{s0}, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @ADD_PREV.10(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00011010011 00 1 0 0 0 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="ADD_PREV {last}{dst}, {n0}|{s0}|, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @ADD_PREV.01(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00011010011 00 1 0 0 1 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="ADD_PREV {last}{dst}, {n0}{s0}, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @ADD_PREV.11(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00011010011 00 1 0 0 1 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="ADD_PREV {last}{dst}, {n0}|{s0}|, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @MULADD_PREV.00(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00011010100 00 1 0 0 0 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="MULADD_PREV {last}{dst}, {n0}{s0}, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @MULADD_PREV.10(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00011010100 00 1 0 0 0 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="MULADD_PREV {last}{dst}, {n0}|{s0}|, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @MULADD_PREV.01(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00011010100 00 1 0 0 1 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="MULADD_PREV {last}{dst}, {n0}{s0}, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @MULADD_PREV.11(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00011010100 00 1 0 0 1 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="MULADD_PREV {last}{dst}, {n0}|{s0}|, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @MULADD_IEEE_PREV.00(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00011010101 00 1 0 0 0 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="MULADD_IEEE_PREV {last}{dst}, {n0}{s0}, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @MULADD_IEEE_PREV.10(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00011010101 00 1 0 0 0 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="MULADD_IEEE_PREV {last}{dst}, {n0}|{s0}|, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @MULADD_IEEE_PREV.01(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00011010101 00 1 0 0 1 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="MULADD_IEEE_PREV {last}{dst}, {n0}{s0}, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @MULADD_IEEE_PREV.11(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00011010101 00 1 0 0 1 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="MULADD_IEEE_PREV {last}{dst}, {n0}|{s0}|, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @INTERP_XY.00(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00011010110 00 1 0 0 0 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="INTERP_XY {last}{dst}, {n0}{s0}, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @INTERP_XY.10(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00011010110 00 1 0 0 0 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="INTERP_XY {last}{dst}, {n0}|{s0}|, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @INTERP_XY.01(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00011010110 00 1 0 0 1 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="INTERP_XY {last}{dst}, {n0}{s0}, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @INTERP_XY.11(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00011010110 00 1 0 0 1 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="INTERP_XY {last}{dst}, {n0}|{s0}|, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @INTERP_ZW.00(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00011010111 00 1 0 0 0 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="INTERP_ZW {last}{dst}, {n0}{s0}, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @INTERP_ZW.10(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00011010111 00 1 0 0 0 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="INTERP_ZW {last}{dst}, {n0}|{s0}|, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @INTERP_ZW.01(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00011010111 00 1 0 0 1 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="INTERP_ZW {last}{dst}, {n0}{s0}, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @INTERP_ZW.11(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00011010111 00 1 0 0 1 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="INTERP_ZW {last}{dst}, {n0}|{s0}|, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @INTERP_X.00(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00011011000 00 1 0 0 0 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="INTERP_X {last}{dst}, {n0}{s0}, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @INTERP_X.10(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00011011000 00 1 0 0 0 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="INTERP_X {last}{dst}, {n0}|{s0}|, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @INTERP_X.01(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00011011000 00 1 0 0 1 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="INTERP_X {last}{dst}, {n0}{s0}, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @INTERP_X.11(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00011011000 00 1 0 0 1 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="INTERP_X {last}{dst}, {n0}|{s0}|, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @INTERP_Z.00(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00011011001 00 1 0 0 0 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="INTERP_Z {last}{dst}, {n0}{s0}, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @INTERP_Z.10(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00011011001 00 1 0 0 0 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="INTERP_Z {last}{dst}, {n0}|{s0}|, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @INTERP_Z.01(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00011011001 00 1 0 0 1 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="INTERP_Z {last}{dst}, {n0}{s0}, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @INTERP_Z.11(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00011011001 00 1 0 0 1 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="INTERP_Z {last}{dst}, {n0}|{s0}|, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @STORE_FLAGS.00(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00011011010 00 1 0 0 0 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="STORE_FLAGS {last}{dst}, {n0}{s0}, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @STORE_FLAGS.10(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00011011010 00 1 0 0 0 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="STORE_FLAGS {last}{dst}, {n0}|{s0}|, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @STORE_FLAGS.01(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00011011010 00 1 0 0 1 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="STORE_FLAGS {last}{dst}, {n0}{s0}, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @STORE_FLAGS.11(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00011011010 00 1 0 0 1 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="STORE_FLAGS {last}{dst}, {n0}|{s0}|, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @LOAD_STORE_FLAGS.00(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00011011011 00 1 0 0 0 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="LOAD_STORE_FLAGS {last}{dst}, {n0}{s0}, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @LOAD_STORE_FLAGS.10(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00011011011 00 1 0 0 0 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="LOAD_STORE_FLAGS {last}{dst}, {n0}|{s0}|, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @LOAD_STORE_FLAGS.01(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00011011011 00 1 0 0 1 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="LOAD_STORE_FLAGS {last}{dst}, {n0}{s0}, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @LOAD_STORE_FLAGS.11(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00011011011 00 1 0 0 1 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="LOAD_STORE_FLAGS {last}{dst}, {n0}|{s0}|, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @INTERP_LOAD_P0.00(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00011100000 00 1 0 0 0 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="INTERP_LOAD_P0 {last}{dst}, {n0}{s0}, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @INTERP_LOAD_P0.10(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00011100000 00 1 0 0 0 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="INTERP_LOAD_P0 {last}{dst}, {n0}|{s0}|, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @INTERP_LOAD_P0.01(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00011100000 00 1 0 0 1 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="INTERP_LOAD_P0 {last}{dst}, {n0}{s0}, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @INTERP_LOAD_P0.11(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00011100000 00 1 0 0 1 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="INTERP_LOAD_P0 {last}{dst}, {n0}|{s0}|, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @INTERP_LOAD_P10.00(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00011100001 00 1 0 0 0 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="INTERP_LOAD_P10 {last}{dst}, {n0}{s0}, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @INTERP_LOAD_P10.10(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00011100001 00 1 0 0 0 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="INTERP_LOAD_P10 {last}{dst}, {n0}|{s0}|, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @INTERP_LOAD_P10.01(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00011100001 00 1 0 0 1 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="INTERP_LOAD_P10 {last}{dst}, {n0}{s0}, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @INTERP_LOAD_P10.11(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00011100001 00 1 0 0 1 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="INTERP_LOAD_P10 {last}{dst}, {n0}|{s0}|, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @INTERP_LOAD_P20.00(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00011100010 00 1 0 0 0 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="INTERP_LOAD_P20 {last}{dst}, {n0}{s0}, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @INTERP_LOAD_P20.10(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00011100010 00 1 0 0 0 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="INTERP_LOAD_P20 {last}{dst}, {n0}|{s0}|, {n1}{s1},{ps}{bs}" "asmonly" {
  ret void
}
define void @INTERP_LOAD_P20.01(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00011100010 00 1 0 0 1 0 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="INTERP_LOAD_P20 {last}{dst}, {n0}{s0}, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @INTERP_LOAD_P20.11(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00011100010 00 1 0 0 1 1 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="INTERP_LOAD_P20 {last}{dst}, {n0}|{s0}|, {n1}|{s1}|,{ps}{bs}" "asmonly" {
  ret void
}
define void @BFE_UINT(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="neg" %n2, i32 "enum"="src" %s2, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00100 n2:1 c2:2=s2&3 0 sel2:9=s2>>2 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="BFE_UINT {last}{dst}, {n0}{s0}, {n1}{s1}, {n2}{s2},{ps}{bs}" "asmonly" {
  ret void
}
define void @BFE_INT(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="neg" %n2, i32 "enum"="src" %s2, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00101 n2:1 c2:2=s2&3 0 sel2:9=s2>>2 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="BFE_INT {last}{dst}, {n0}{s0}, {n1}{s1}, {n2}{s2},{ps}{bs}" "asmonly" {
  ret void
}
define void @BFI_INT(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="neg" %n2, i32 "enum"="src" %s2, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00110 n2:1 c2:2=s2&3 0 sel2:9=s2>>2 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="BFI_INT {last}{dst}, {n0}{s0}, {n1}{s1}, {n2}{s2},{ps}{bs}" "asmonly" {
  ret void
}
define void @FMA(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="neg" %n2, i32 "enum"="src" %s2, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 00111 n2:1 c2:2=s2&3 0 sel2:9=s2>>2 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="FMA {last}{dst}, {n0}{s0}, {n1}{s1}, {n2}{s2},{ps}{bs}" "asmonly" {
  ret void
}
define void @CNDNE_64(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="neg" %n2, i32 "enum"="src" %s2, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 01001 n2:1 c2:2=s2&3 0 sel2:9=s2>>2 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="CNDNE_64 {last}{dst}, {n0}{s0}, {n1}{s1}, {n2}{s2},{ps}{bs}" "asmonly" {
  ret void
}
define void @FMA_64(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="neg" %n2, i32 "enum"="src" %s2, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 01010 n2:1 c2:2=s2&3 0 sel2:9=s2>>2 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="FMA_64 {last}{dst}, {n0}{s0}, {n1}{s1}, {n2}{s2},{ps}{bs}" "asmonly" {
  ret void
}
define void @LERP_UINT(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="neg" %n2, i32 "enum"="src" %s2, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 01011 n2:1 c2:2=s2&3 0 sel2:9=s2>>2 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="LERP_UINT {last}{dst}, {n0}{s0}, {n1}{s1}, {n2}{s2},{ps}{bs}" "asmonly" {
  ret void
}
define void @BIT_ALIGN_INT(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="neg" %n2, i32 "enum"="src" %s2, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 01100 n2:1 c2:2=s2&3 0 sel2:9=s2>>2 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="BIT_ALIGN_INT {last}{dst}, {n0}{s0}, {n1}{s1}, {n2}{s2},{ps}{bs}" "asmonly" {
  ret void
}
define void @BYTE_ALIGN_INT(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="neg" %n2, i32 "enum"="src" %s2, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 01101 n2:1 c2:2=s2&3 0 sel2:9=s2>>2 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="BYTE_ALIGN_INT {last}{dst}, {n0}{s0}, {n1}{s1}, {n2}{s2},{ps}{bs}" "asmonly" {
  ret void
}
define void @SAD_ACCUM_UINT(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="neg" %n2, i32 "enum"="src" %s2, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 01110 n2:1 c2:2=s2&3 0 sel2:9=s2>>2 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="SAD_ACCUM_UINT {last}{dst}, {n0}{s0}, {n1}{s1}, {n2}{s2},{ps}{bs}" "asmonly" {
  ret void
}
define void @SAD_ACCUM_HI_UINT(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="neg" %n2, i32 "enum"="src" %s2, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 01111 n2:1 c2:2=s2&3 0 sel2:9=s2>>2 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="SAD_ACCUM_HI_UINT {last}{dst}, {n0}{s0}, {n1}{s1}, {n2}{s2},{ps}{bs}" "asmonly" {
  ret void
}
define void @MULADD_UINT24(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="neg" %n2, i32 "enum"="src" %s2, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 10000 n2:1 c2:2=s2&3 0 sel2:9=s2>>2 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="MULADD_UINT24 {last}{dst}, {n0}{s0}, {n1}{s1}, {n2}{s2},{ps}{bs}" "asmonly" {
  ret void
}
define void @MULADD(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="neg" %n2, i32 "enum"="src" %s2, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 10100 n2:1 c2:2=s2&3 0 sel2:9=s2>>2 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="MULADD {last}{dst}, {n0}{s0}, {n1}{s1}, {n2}{s2},{ps}{bs}" "asmonly" {
  ret void
}
define void @MULADD_M2(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="neg" %n2, i32 "enum"="src" %s2, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 10101 n2:1 c2:2=s2&3 0 sel2:9=s2>>2 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="MULADD_M2 {last}{dst}, {n0}{s0}, {n1}{s1}, {n2}{s2},{ps}{bs}" "asmonly" {
  ret void
}
define void @MULADD_M4(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="neg" %n2, i32 "enum"="src" %s2, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 10110 n2:1 c2:2=s2&3 0 sel2:9=s2>>2 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="MULADD_M4 {last}{dst}, {n0}{s0}, {n1}{s1}, {n2}{s2},{ps}{bs}" "asmonly" {
  ret void
}
define void @MULADD_D2(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="neg" %n2, i32 "enum"="src" %s2, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 10111 n2:1 c2:2=s2&3 0 sel2:9=s2>>2 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="MULADD_D2 {last}{dst}, {n0}{s0}, {n1}{s1}, {n2}{s2},{ps}{bs}" "asmonly" {
  ret void
}
define void @MULADD_IEEE(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="neg" %n2, i32 "enum"="src" %s2, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 11000 n2:1 c2:2=s2&3 0 sel2:9=s2>>2 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="MULADD_IEEE {last}{dst}, {n0}{s0}, {n1}{s1}, {n2}{s2},{ps}{bs}" "asmonly" {
  ret void
}
define void @CNDE(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="neg" %n2, i32 "enum"="src" %s2, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 11001 n2:1 c2:2=s2&3 0 sel2:9=s2>>2 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="CNDE {last}{dst}, {n0}{s0}, {n1}{s1}, {n2}{s2},{ps}{bs}" "asmonly" {
  ret void
}
define void @CNDGT(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="neg" %n2, i32 "enum"="src" %s2, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 11010 n2:1 c2:2=s2&3 0 sel2:9=s2>>2 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="CNDGT {last}{dst}, {n0}{s0}, {n1}{s1}, {n2}{s2},{ps}{bs}" "asmonly" {
  ret void
}
define void @CNDGE(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="neg" %n2, i32 "enum"="src" %s2, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 11011 n2:1 c2:2=s2&3 0 sel2:9=s2>>2 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="CNDGE {last}{dst}, {n0}{s0}, {n1}{s1}, {n2}{s2},{ps}{bs}" "asmonly" {
  ret void
}
define void @CNDE_INT(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="neg" %n2, i32 "enum"="src" %s2, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 11100 n2:1 c2:2=s2&3 0 sel2:9=s2>>2 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="CNDE_INT {last}{dst}, {n0}{s0}, {n1}{s1}, {n2}{s2},{ps}{bs}" "asmonly" {
  ret void
}
define void @CNDGT_INT(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="neg" %n2, i32 "enum"="src" %s2, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 11101 n2:1 c2:2=s2&3 0 sel2:9=s2>>2 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="CNDGT_INT {last}{dst}, {n0}{s0}, {n1}{s1}, {n2}{s2},{ps}{bs}" "asmonly" {
  ret void
}
define void @CNDGE_INT(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="neg" %n2, i32 "enum"="src" %s2, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 11110 n2:1 c2:2=s2&3 0 sel2:9=s2>>2 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="CNDGE_INT {last}{dst}, {n0}{s0}, {n1}{s1}, {n2}{s2},{ps}{bs}" "asmonly" {
  ret void
}
define void @MUL_LIT(i32 "enum"="last" %last, i32 "enum"="dst" %dst, i32 "enum"="neg" %n0, i32 "enum"="src" %s0, i32 "enum"="neg" %n1, i32 "enum"="src" %s1, i32 "enum"="neg" %n2, i32 "enum"="src" %s2, i32 "enum"="ps" %ps, i32 "enum"="bs" %bs) "enc"="0 dc:2=dst&3 0 dg:7=dst>>2 bs:3 11111 n2:1 c2:2=s2&3 0 sel2:9=s2>>2 last:1 ps:2 000 n1:1 c1:2=s1&3 0 sel1:9=s1>>2 n0:1 c0:2=s0&3 0 sel0:9=s0>>2" "asm"="MUL_LIT {last}{dst}, {n0}{s0}, {n1}{s1}, {n2}{s2},{ps}{bs}" "asmonly" {
  ret void
}

define void @LITERALS(i32 "imm" %x, i32 "imm" %y) "enc"="y:s32 x:s32" "asm"="LITERALS {x}, {y}" "asmonly" {
  ret void
}
