; x86's own intrinsics written by hand: those Rust's core::arch uses that
; discover_intrinsics.py finds no one instruction of 02-xed.ll for
; (intrinsics.txt says why for each), as 10-integer.ll's generated ones
; are written. Their encodings are llc's (and Intel's manual's).
;
; The FXSAVE area (512 bytes at %a0), saved and restored: 0F AE /0 and /1,
; with REX.W for the 64-bit forms. (XED says they use the x87's control
; registers, which no operand names.)

define void @X86.fxsave(ptr "reg"="r64" %a0) "enc"="rex(0, 0, 4, a0) | 00001111 | 10101110 | rm(0, a0, 4, 1, 0, 0, 1)" "asm"="fxsave 0({a0})" "feature"="fxsr" "nodecode" {
  call void @llvm.x86.fxsave(ptr %a0)
  ret void
}

declare void @llvm.x86.fxsave(ptr)

define void @X86.fxrstor(ptr "reg"="r64" %a0) "enc"="rex(0, 0, 4, a0) | 00001111 | 10101110 | rm(1, a0, 4, 1, 0, 0, 1)" "asm"="fxrstor 0({a0})" "feature"="fxsr" "nodecode" {
  call void @llvm.x86.fxrstor(ptr %a0)
  ret void
}

declare void @llvm.x86.fxrstor(ptr)

define void @X86.fxsave64(ptr "reg"="r64" %a0) "enc"="rex(1, 0, 4, a0) | 00001111 | 10101110 | rm(0, a0, 4, 1, 0, 0, 1)" "asm"="fxsave64 0({a0})" "feature"="fxsr" "nodecode" {
  call void @llvm.x86.fxsave64(ptr %a0)
  ret void
}

declare void @llvm.x86.fxsave64(ptr)

define void @X86.fxrstor64(ptr "reg"="r64" %a0) "enc"="rex(1, 0, 4, a0) | 00001111 | 10101110 | rm(1, a0, 4, 1, 0, 0, 1)" "asm"="fxrstor64 0({a0})" "feature"="fxsr" "nodecode" {
  call void @llvm.x86.fxrstor64(ptr %a0)
  ret void
}

declare void @llvm.x86.fxrstor64(ptr)

; CRC-32C of the accumulator %a0 and %a1: F2 (REX) 0F 38 F0 /r for a byte,
; F1 for wider (66 for a word, REX.W for a quadword). A byte register 4-7
; needs a REX to be spl-dil rather than ah-bh. The feature is LLVM's
; `crc32` (which rustc turns on with `sse4.2`); llc built these only once
; told it, which discovery did not do.

define i32 @X86.crc32.32.8(i32 "reg"="r32" "tied" %a0, i8 "reg"="r32" %a1) "enc"="11110010 | { if(a1<4) rex(0, d, 0, a1) ; 0100 W:1=0 R:1=(d)>>3 X:1=0 B:1=(a1)>>3 } | 00001111 | 00111000 | 11110000 | rr(d, a1)" "asm"="crc32b {a1:r8}, {d}" "dclass"="r32" "feature"="crc32" "nodecode" {
  %r = call i32 @llvm.x86.sse42.crc32.32.8(i32 %a0, i8 %a1)
  ret i32 %r
}

declare i32 @llvm.x86.sse42.crc32.32.8(i32, i8)

define i32 @X86.crc32.32.16(i32 "reg"="r32" "tied" %a0, i16 "reg"="r32" %a1) "enc"="01100110 | 11110010 | rex(0, d, 0, a1) | 00001111 | 00111000 | 11110001 | rr(d, a1)" "asm"="crc32w {a1:r16}, {d}" "dclass"="r32" "feature"="crc32" "nodecode" {
  %r = call i32 @llvm.x86.sse42.crc32.32.16(i32 %a0, i16 %a1)
  ret i32 %r
}

declare i32 @llvm.x86.sse42.crc32.32.16(i32, i16)

define i32 @X86.crc32.32.32(i32 "reg"="r32" "tied" %a0, i32 "reg"="r32" %a1) "enc"="11110010 | rex(0, d, 0, a1) | 00001111 | 00111000 | 11110001 | rr(d, a1)" "asm"="crc32l {a1}, {d}" "dclass"="r32" "feature"="crc32" "nodecode" {
  %r = call i32 @llvm.x86.sse42.crc32.32.32(i32 %a0, i32 %a1)
  ret i32 %r
}

declare i32 @llvm.x86.sse42.crc32.32.32(i32, i32)

define i64 @X86.crc32.64.64(i64 "reg"="r64" "tied" %a0, i64 "reg"="r64" %a1) "enc"="11110010 | rex(1, d, 0, a1) | 00001111 | 00111000 | 11110001 | rr(d, a1)" "asm"="crc32q {a1}, {d}" "dclass"="r64" "feature"="crc32" "nodecode" {
  %r = call i64 @llvm.x86.sse42.crc32.64.64(i64 %a0, i64 %a1)
  ret i64 %r
}

declare i64 @llvm.x86.sse42.crc32.64.64(i64, i64)

; Loads with the read-shared hint (MOVRS) and its prefetch (PREFETCHRST2):
; the plain load, and nothing (as llvm.prefetch is): the same but for the
; hint.
; (They are newer than the llvm-mc that the tests check encodings with,
; so 02-xed.ll leaves them out: rejects.txt.)

define i8 @movrsqi(ptr %a) "lower"="movrsqi.lower" {
  %r = call i8 @llvm.x86.movrsqi(ptr %a)
  ret i8 %r
}

define i8 @movrsqi.lower(ptr %a) {
  %r = load i8, ptr %a
  ret i8 %r
}

declare i8 @llvm.x86.movrsqi(ptr)

define i16 @movrshi(ptr %a) "lower"="movrshi.lower" {
  %r = call i16 @llvm.x86.movrshi(ptr %a)
  ret i16 %r
}

define i16 @movrshi.lower(ptr %a) {
  %r = load i16, ptr %a
  ret i16 %r
}

declare i16 @llvm.x86.movrshi(ptr)

define i32 @movrssi(ptr %a) "lower"="movrssi.lower" {
  %r = call i32 @llvm.x86.movrssi(ptr %a)
  ret i32 %r
}

define i32 @movrssi.lower(ptr %a) {
  %r = load i32, ptr %a
  ret i32 %r
}

declare i32 @llvm.x86.movrssi(ptr)

define i64 @movrsdi(ptr %a) "lower"="movrsdi.lower" {
  %r = call i64 @llvm.x86.movrsdi(ptr %a)
  ret i64 %r
}

define i64 @movrsdi.lower(ptr %a) {
  %r = load i64, ptr %a
  ret i64 %r
}

declare i64 @llvm.x86.movrsdi(ptr)

define void @prefetchrs(ptr %a) "lower"="prefetchrs.lower" {
  call void @llvm.x86.prefetchrs(ptr %a)
  ret void
}

define void @prefetchrs.lower(ptr %a) {
  ret void
}

declare void @llvm.x86.prefetchrs(ptr)

; Transactions (RTM). xbegin's result is -1 when the transaction starts,
; else the abort's status, which the processor puts in eax as it resumes
; at the fallback address: here the next instruction (a displacement of
; 0), so both reach the same place. xtest's is ZF clear, inside one.

define i32 @X86.xbegin() "enc"="10111000 | 11111111 | 11111111 | 11111111 | 11111111 | 11000111 | 11111000 | 00000000 | 00000000 | 00000000 | 00000000" "asm"="movl $-1, %eax\0A\09xbegin .+6" "dclass"="rax" "feature"="rtm" "nodecode" {
  %r = call i32 @llvm.x86.xbegin()
  ret i32 %r
}

declare i32 @llvm.x86.xbegin()

define i32 @X86.xtest() "enc"="00110001 | 11000000 | 00001111 | 00000001 | 11010110 | 00001111 | 10010101 | 11000000" "asm"="xorl %eax, %eax\0A\09xtest\0A\09setne %al" "dclass"="rax" "clobbers"="flags" "feature"="rtm" "nodecode" {
  %r = call i32 @llvm.x86.xtest()
  ret i32 %r
}

declare i32 @llvm.x86.xtest()
