; tiny: a made-up control core, for testing partial machines
; (docs/design/partial-machines.md). It has sixteen 32-bit registers,
; arithmetic, compare-and-branch, an instruction that starts another
; unit's program and one that ends this one, and nothing else: no memory, so no stack and no globals; no
; calls; no floating point; nothing wider than 32 bits; no branch through
; a register; no trap.
;
; Instructions are 32-bit words: a 6-bit opcode at the top, then
; registers of 4 bits, then an immediate or a branch offset in words.
;
;   R  op:6 d:4 a:4 b:4 0:14       add r1, r2, r3
;   I  op:6 d:4 a:4 imm:18         addi r1, r2, -5
;   B  op:6 a:4 b:4 off:18         beq r1, r2, label
;
; A program is started with its arguments in r1-r4 and ends with `halt`,
; its result in r1.

!target.lacks = !{!1}
!1 = !{!"memory", !"calls", !"float", !"i64", !"indirect", !"trap"}

!target.registers = !{!10, !12, !13}
!10 = !{!"r", !"r{0-15}"}
!12 = !{!"r5c", !"r{0-15}"}
!13 = !{!"r6c", !"r{0-15}"}
!target.classes = !{!11, !14, !15}
!11 = !{!"r", !"file", !"r", !"bits", !"32", !"holds", !"int"}
!14 = !{!"r5c", !"file", !"r", !"bits", !"32", !"fixed", !"r5"}
!15 = !{!"r6c", !"file", !"r", !"bits", !"32", !"fixed", !"r6"}

; The launched program's entry, in words.
!target.relocs = !{!40}
!40 = !{!"name", !"launch", !"elf", !"1", !"elf-name", !"R_TINY_LAUNCH"}

!target.params = !{!20, !21, !22, !23}
!20 = !{!"boolean-branches", !"compare"}
!21 = !{!"elf-machine", !"4660"}
!22 = !{!"asm-comment", !"#"}
!23 = !{!"vector-bits", !"0"}

; Arguments in r1-r4, the result in r1. Nothing is saved, as nothing
; calls: every register is free to use.
!abi.tiny = !{!30, !31, !32}
!30 = !{!"args", !"r1", !"r2", !"r3", !"r4"}
!31 = !{!"result", !"r1"}
!32 = !{!"triples", !"riscv64", !"riscv32", !"tiny"}

; ---------------------------------------------------------------- arithmetic

define i32 @add(i32 "reg"="r" %a, i32 "reg"="r" %b) "enc"="000001 d:4 a:4 b:4 00000000000000" "asm"="add {d}, {a}, {b}" "role"="add" {
  %r = add i32 %a, %b
  ret i32 %r
}
define i32 @sub(i32 "reg"="r" %a, i32 "reg"="r" %b) "enc"="000010 d:4 a:4 b:4 00000000000000" "asm"="sub {d}, {a}, {b}" "role"="sub" {
  %r = sub i32 %a, %b
  ret i32 %r
}
define i32 @and(i32 "reg"="r" %a, i32 "reg"="r" %b) "enc"="000011 d:4 a:4 b:4 00000000000000" "asm"="and {d}, {a}, {b}" {
  %r = and i32 %a, %b
  ret i32 %r
}
define i32 @or(i32 "reg"="r" %a, i32 "reg"="r" %b) "enc"="000100 d:4 a:4 b:4 00000000000000" "asm"="or {d}, {a}, {b}" {
  %r = or i32 %a, %b
  ret i32 %r
}
define i32 @xor(i32 "reg"="r" %a, i32 "reg"="r" %b) "enc"="000101 d:4 a:4 b:4 00000000000000" "asm"="xor {d}, {a}, {b}" {
  %r = xor i32 %a, %b
  ret i32 %r
}
define i32 @shl(i32 "reg"="r" %a, i32 "reg"="r" %b) "enc"="000110 d:4 a:4 b:4 00000000000000" "asm"="shl {d}, {a}, {b}" {
  %r = shl i32 %a, %b
  ret i32 %r
}
define i32 @shr(i32 "reg"="r" %a, i32 "reg"="r" %b) "enc"="000111 d:4 a:4 b:4 00000000000000" "asm"="shr {d}, {a}, {b}" {
  %r = lshr i32 %a, %b
  ret i32 %r
}
define i32 @sar(i32 "reg"="r" %a, i32 "reg"="r" %b) "enc"="001000 d:4 a:4 b:4 00000000000000" "asm"="sar {d}, {a}, {b}" {
  %r = ashr i32 %a, %b
  ret i32 %r
}
define i32 @mul(i32 "reg"="r" %a, i32 "reg"="r" %b) "enc"="001001 d:4 a:4 b:4 00000000000000" "asm"="mul {d}, {a}, {b}" {
  %r = mul i32 %a, %b
  ret i32 %r
}
define i1 @slt(i32 "reg"="r" %a, i32 "reg"="r" %b) "enc"="001010 d:4 a:4 b:4 00000000000000" "asm"="slt {d}, {a}, {b}" "zext"="1" {
  %r = icmp slt i32 %a, %b
  ret i1 %r
}
define i1 @sltu(i32 "reg"="r" %a, i32 "reg"="r" %b) "enc"="001011 d:4 a:4 b:4 00000000000000" "asm"="sltu {d}, {a}, {b}" "zext"="1" {
  %r = icmp ult i32 %a, %b
  ret i1 %r
}
define i1 @seq(i32 "reg"="r" %a, i32 "reg"="r" %b) "enc"="001100 d:4 a:4 b:4 00000000000000" "asm"="seq {d}, {a}, {b}" "zext"="1" {
  %r = icmp eq i32 %a, %b
  ret i1 %r
}

define i32 @div(i32 "reg"="r" %a, i32 "reg"="r" %b) "enc"="001101 d:4 a:4 b:4 00000000000000" "asm"="div {d}, {a}, {b}" {
  %r = sdiv i32 %a, %b
  ret i32 %r
}
define i32 @divu(i32 "reg"="r" %a, i32 "reg"="r" %b) "enc"="001110 d:4 a:4 b:4 00000000000000" "asm"="divu {d}, {a}, {b}" {
  %r = udiv i32 %a, %b
  ret i32 %r
}
define i32 @rem(i32 "reg"="r" %a, i32 "reg"="r" %b) "enc"="001111 d:4 a:4 b:4 00000000000000" "asm"="rem {d}, {a}, {b}" {
  %r = srem i32 %a, %b
  ret i32 %r
}
define i32 @remu(i32 "reg"="r" %a, i32 "reg"="r" %b) "enc"="010111 d:4 a:4 b:4 00000000000000" "asm"="remu {d}, {a}, {b}" {
  %r = urem i32 %a, %b
  ret i32 %r
}

; ---------------------------------------------------------------- immediates

define i32 @addi(i32 "reg"="r" %a, i32 "imm" %i) "enc"="010000 d:4 a:4 i:s18" "asm"="addi {d}, {a}, {i}" "role"="addi" {
  %r = add i32 %a, %i
  ret i32 %r
}
define i32 @andi(i32 "reg"="r" %a, i32 "imm" %i) "enc"="010001 d:4 a:4 i:s18" "asm"="andi {d}, {a}, {i}" {
  %r = and i32 %a, %i
  ret i32 %r
}
define i32 @ori(i32 "reg"="r" %a, i32 "imm" %i) "enc"="010010 d:4 a:4 i:s18" "asm"="ori {d}, {a}, {i}" {
  %r = or i32 %a, %i
  ret i32 %r
}
define i32 @xori(i32 "reg"="r" %a, i32 "imm" %i) "enc"="010011 d:4 a:4 i:s18" "asm"="xori {d}, {a}, {i}" {
  %r = xor i32 %a, %i
  ret i32 %r
}
define i32 @shli(i32 "reg"="r" %a, i32 "imm" %i) "enc"="010100 d:4 a:4 0000000000000 i:5" "asm"="shli {d}, {a}, {i}" "role"="shli" {
  %r = shl i32 %a, %i
  ret i32 %r
}
define i32 @shri(i32 "reg"="r" %a, i32 "imm" %i) "enc"="010101 d:4 a:4 0000000000000 i:5" "asm"="shri {d}, {a}, {i}" "role"="shri" {
  %r = lshr i32 %a, %i
  ret i32 %r
}
define i32 @sari(i32 "reg"="r" %a, i32 "imm" %i) "enc"="010110 d:4 a:4 0000000000000 i:5" "asm"="sari {d}, {a}, {i}" "role"="sari" {
  %r = ashr i32 %a, %i
  ret i32 %r
}
define i32 @li(i32 "imm" %i) "enc"="011000 d:4 0000 i:s18" "asm"="li {d}, {i}" {
  ret i32 %i
}
; The high 18 bits of a register, the low 14 cleared: with addi, any
; 32-bit constant.
define i32 @lui(i32 "imm" %i) "enc"="011001 d:4 0000 h:s18=i>>14" "asm"="lui {d}, {=(i>>14)&262143}" {
  ret i32 %i
}
define i32 @mov(i32 "reg"="r" %a) "enc"="011111 d:4 a:4 000000000000000000" "asm"="mov {d}, {a}" "copy"="r" {
  ret i32 %a
}

; Narrow values, made whole: their own forms of andi.
define i32 @zext1(i1 "reg"="r" %a) "enc"="010001 d:4 a:4 000000000000000001" "asm"="andi {d}, {a}, 1" "zext"="1" "role"="zext.1" "nodecode" {
  %r = zext i1 %a to i32
  ret i32 %r
}
define i32 @zext8(i8 "reg"="r" %a) "enc"="010001 d:4 a:4 000000000011111111" "asm"="andi {d}, {a}, 255" "zext"="8" "role"="zext.8" "nodecode" {
  %r = zext i8 %a to i32
  ret i32 %r
}
define i32 @zext16(i16 "reg"="r" %a) "enc"="010001 d:4 a:4 001111111111111111" "asm"="andi {d}, {a}, 65535" "zext"="16" "role"="zext.16" "nodecode" {
  %r = zext i16 %a to i32
  ret i32 %r
}
define i1 @not1(i1 "reg"="r" %a) "enc"="010011 d:4 a:4 000000000000000001" "asm"="xori {d}, {a}, 1" "zext"="1" "nodecode" {
  %r = xor i1 %a, true
  ret i1 %r
}

; ---------------------------------------------------------------- branches

define void @beq(i32 "reg"="r" %a, i32 "reg"="r" %b, ptr "label" %l) "enc"="100000 a:4 b:4 o:s18=l>>2" "asm"="beq {a}, {b}, {l}" "inverse"="bne" {
  %c = icmp eq i32 %a, %b
  br i1 %c, label %t, label %n
t:
  indirectbr ptr %l, []
n:
  ret void
}
define void @bne(i32 "reg"="r" %a, i32 "reg"="r" %b, ptr "label" %l) "enc"="100001 a:4 b:4 o:s18=l>>2" "asm"="bne {a}, {b}, {l}" "inverse"="beq" {
  %c = icmp ne i32 %a, %b
  br i1 %c, label %t, label %n
t:
  indirectbr ptr %l, []
n:
  ret void
}
define void @blt(i32 "reg"="r" %a, i32 "reg"="r" %b, ptr "label" %l) "enc"="100010 a:4 b:4 o:s18=l>>2" "asm"="blt {a}, {b}, {l}" "inverse"="bge" {
  %c = icmp slt i32 %a, %b
  br i1 %c, label %t, label %n
t:
  indirectbr ptr %l, []
n:
  ret void
}
define void @bge(i32 "reg"="r" %a, i32 "reg"="r" %b, ptr "label" %l) "enc"="100011 a:4 b:4 o:s18=l>>2" "asm"="bge {a}, {b}, {l}" "inverse"="blt" {
  %c = icmp sge i32 %a, %b
  br i1 %c, label %t, label %n
t:
  indirectbr ptr %l, []
n:
  ret void
}
define void @bltu(i32 "reg"="r" %a, i32 "reg"="r" %b, ptr "label" %l) "enc"="100100 a:4 b:4 o:s18=l>>2" "asm"="bltu {a}, {b}, {l}" "inverse"="bgeu" {
  %c = icmp ult i32 %a, %b
  br i1 %c, label %t, label %n
t:
  indirectbr ptr %l, []
n:
  ret void
}
define void @bgeu(i32 "reg"="r" %a, i32 "reg"="r" %b, ptr "label" %l) "enc"="100101 a:4 b:4 o:s18=l>>2" "asm"="bgeu {a}, {b}, {l}" "inverse"="bltu" {
  %c = icmp uge i32 %a, %b
  br i1 %c, label %t, label %n
t:
  indirectbr ptr %l, []
n:
  ret void
}
; Branches on one register against zero.
define void @bz(i32 "reg"="r" %a, ptr "label" %l) "enc"="100110 a:4 0000 o:s18=l>>2" "asm"="bz {a}, {l}" "inverse"="bnz" {
  %c = icmp eq i32 %a, 0
  br i1 %c, label %t, label %n
t:
  indirectbr ptr %l, []
n:
  ret void
}
define void @bnz(i32 "reg"="r" %a, ptr "label" %l) "enc"="100111 a:4 0000 o:s18=l>>2" "asm"="bnz {a}, {l}" "inverse"="bz" {
  %c = icmp ne i32 %a, 0
  br i1 %c, label %t, label %n
t:
  indirectbr ptr %l, []
n:
  ret void
}
define void @jmp(ptr "label" %l) "enc"="101000 00000000 o:s18=l>>2" "asm"="jmp {l}" "role"="jump" {
  indirectbr ptr %l, []
}

; Starts another unit's program, named by its symbol (an "epitome-opaque"
; function, compiled by something else), with its arguments in r1-r4, and
; carries on at once: the launch is not a call, and nothing returns.
define void @launch(ptr "sym" %s) "enc"="101001 0000000 e:19=s>>2" "asm"="launch {s}" "reloc"="launch" "role"="launch" "nomatch" {
  ret void
}

; The end of the program: with no calls, a return is the end.
define void @halt() "enc"="11111100000000000000000000000000" "asm"="halt" "role"="ret" {
  ret void
}

; ---------------------------------------------------------------- recipes
; What has no instruction of its own, from those that do.
define i1 @icmp.ne(i32 "reg"="r" %a, i32 "reg"="r" %b) "recipe"="icmp.ne.expand" {
  %r = icmp ne i32 %a, %b
  ret i1 %r
}
define i1 @icmp.ne.expand(i32 %a, i32 %b) {
  %x = call i32 @xor(i32 %a, i32 %b)
  %z = call i32 @li(i32 0)
  %r = call i1 @sltu(i32 %z, i32 %x)
  ret i1 %r
}
define i1 @icmp.sgt(i32 "reg"="r" %a, i32 "reg"="r" %b) "recipe"="icmp.sgt.expand" {
  %r = icmp sgt i32 %a, %b
  ret i1 %r
}
define i1 @icmp.sgt.expand(i32 %a, i32 %b) {
  %l = call i1 @slt(i32 %b, i32 %a)
  ret i1 %l
}
define i1 @icmp.ugt(i32 "reg"="r" %a, i32 "reg"="r" %b) "recipe"="icmp.ugt.expand" {
  %r = icmp ugt i32 %a, %b
  ret i1 %r
}
define i1 @icmp.ugt.expand(i32 %a, i32 %b) {
  %l = call i1 @sltu(i32 %b, i32 %a)
  ret i1 %l
}
define i1 @icmp.sge(i32 "reg"="r" %a, i32 "reg"="r" %b) "recipe"="icmp.sge.expand" {
  %r = icmp sge i32 %a, %b
  ret i1 %r
}
define i1 @icmp.sge.expand(i32 %a, i32 %b) {
  %l = call i1 @slt(i32 %a, i32 %b)
  %r = call i1 @not1(i1 %l)
  ret i1 %r
}
define i1 @icmp.uge(i32 "reg"="r" %a, i32 "reg"="r" %b) "recipe"="icmp.uge.expand" {
  %r = icmp uge i32 %a, %b
  ret i1 %r
}
define i1 @icmp.uge.expand(i32 %a, i32 %b) {
  %l = call i1 @sltu(i32 %a, i32 %b)
  %r = call i1 @not1(i1 %l)
  ret i1 %r
}
define i1 @icmp.sle(i32 "reg"="r" %a, i32 "reg"="r" %b) "recipe"="icmp.sle.expand" {
  %r = icmp sle i32 %a, %b
  ret i1 %r
}
define i1 @icmp.sle.expand(i32 %a, i32 %b) {
  %l = call i1 @slt(i32 %b, i32 %a)
  %r = call i1 @not1(i1 %l)
  ret i1 %r
}
define i1 @icmp.ule(i32 "reg"="r" %a, i32 "reg"="r" %b) "recipe"="icmp.ule.expand" {
  %r = icmp ule i32 %a, %b
  ret i1 %r
}
define i1 @icmp.ule.expand(i32 %a, i32 %b) {
  %l = call i1 @sltu(i32 %b, i32 %a)
  %r = call i1 @not1(i1 %l)
  ret i1 %r
}
define i32 @select(i1 "reg"="r" %c, i32 "reg"="r" %a, i32 "reg"="r" %b) "recipe"="select.expand" {
  %r = select i1 %c, i32 %a, i32 %b
  ret i32 %r
}
define i32 @select.expand(i1 %c, i32 %a, i32 %b) {
  %c1 = call i32 @zext1(i1 %c)
  %z = call i32 @li(i32 0)
  %m = call i32 @sub(i32 %z, i32 %c1)
  %x = call i32 @xor(i32 %a, i32 %b)
  %y = call i32 @and(i32 %x, i32 %m)
  %r = call i32 @xor(i32 %b, i32 %y)
  ret i32 %r
}
define i32 @smin(i32 "reg"="r" %a, i32 "reg"="r" %b) "recipe"="smin.expand" {
  %r = call i32 @llvm.smin.i32(i32 %a, i32 %b)
  ret i32 %r
}
define i32 @smin.expand(i32 %a, i32 %b) {
  %l = call i1 @slt(i32 %a, i32 %b)
  %c1 = call i32 @zext1(i1 %l)
  %z = call i32 @li(i32 0)
  %m = call i32 @sub(i32 %z, i32 %c1)
  %x = call i32 @xor(i32 %a, i32 %b)
  %y = call i32 @and(i32 %x, i32 %m)
  %r = call i32 @xor(i32 %b, i32 %y)
  ret i32 %r
}
define i32 @smax(i32 "reg"="r" %a, i32 "reg"="r" %b) "recipe"="smax.expand" {
  %r = call i32 @llvm.smax.i32(i32 %a, i32 %b)
  ret i32 %r
}
define i32 @smax.expand(i32 %a, i32 %b) {
  %l = call i1 @slt(i32 %b, i32 %a)
  %c1 = call i32 @zext1(i1 %l)
  %z = call i32 @li(i32 0)
  %m = call i32 @sub(i32 %z, i32 %c1)
  %x = call i32 @xor(i32 %a, i32 %b)
  %y = call i32 @and(i32 %x, i32 %m)
  %r = call i32 @xor(i32 %b, i32 %y)
  ret i32 %r
}
define i32 @umin(i32 "reg"="r" %a, i32 "reg"="r" %b) "recipe"="umin.expand" {
  %r = call i32 @llvm.umin.i32(i32 %a, i32 %b)
  ret i32 %r
}
define i32 @umin.expand(i32 %a, i32 %b) {
  %l = call i1 @sltu(i32 %a, i32 %b)
  %c1 = call i32 @zext1(i1 %l)
  %z = call i32 @li(i32 0)
  %m = call i32 @sub(i32 %z, i32 %c1)
  %x = call i32 @xor(i32 %a, i32 %b)
  %y = call i32 @and(i32 %x, i32 %m)
  %r = call i32 @xor(i32 %b, i32 %y)
  ret i32 %r
}
define i32 @umax(i32 "reg"="r" %a, i32 "reg"="r" %b) "recipe"="umax.expand" {
  %r = call i32 @llvm.umax.i32(i32 %a, i32 %b)
  ret i32 %r
}
define i32 @umax.expand(i32 %a, i32 %b) {
  %l = call i1 @sltu(i32 %b, i32 %a)
  %c1 = call i32 @zext1(i1 %l)
  %z = call i32 @li(i32 0)
  %m = call i32 @sub(i32 %z, i32 %c1)
  %x = call i32 @xor(i32 %a, i32 %b)
  %y = call i32 @and(i32 %x, i32 %m)
  %r = call i32 @xor(i32 %b, i32 %y)
  ret i32 %r
}
define i32 @abs(i32 "reg"="r" %a) "recipe"="abs.expand" {
  %r = call i32 @llvm.abs.i32(i32 %a, i1 false)
  ret i32 %r
}
define i32 @abs.expand(i32 %a) {
  %s = call i32 @sari(i32 %a, i32 31)
  %x = call i32 @xor(i32 %a, i32 %s)
  %r = call i32 @sub(i32 %x, i32 %s)
  ret i32 %r
}
define i32 @sext.i8(i8 "reg"="r" %a) "recipe"="sext.i8.expand" {
  %r = sext i8 %a to i32
  ret i32 %r
}
define i32 @sext.i8.expand(i8 %a) {
  %w = zext i8 %a to i32
  %u = call i32 @shli(i32 %w, i32 24)
  %r = call i32 @sari(i32 %u, i32 24)
  ret i32 %r
}
define i32 @sext.i16(i16 "reg"="r" %a) "recipe"="sext.i16.expand" {
  %r = sext i16 %a to i32
  ret i32 %r
}
define i32 @sext.i16.expand(i16 %a) {
  %w = zext i16 %a to i32
  %u = call i32 @shli(i32 %w, i32 16)
  %r = call i32 @sari(i32 %u, i32 16)
  ret i32 %r
}
define i32 @sext.i1(i1 "reg"="r" %a) "recipe"="sext.i1.expand" {
  %r = sext i1 %a to i32
  ret i32 %r
}
define i32 @sext.i1.expand(i1 %a) {
  %c1 = call i32 @zext1(i1 %a)
  %z = call i32 @li(i32 0)
  %r = call i32 @sub(i32 %z, i32 %c1)
  ret i32 %r
}

declare i32 @llvm.smin.i32(i32, i32)
declare i32 @llvm.smax.i32(i32, i32)
declare i32 @llvm.umin.i32(i32, i32)
declare i32 @llvm.umax.i32(i32, i32)
declare i32 @llvm.abs.i32(i32, i1)

; ---------------------------------------------------------------- its own
; What the IR has no operation for: calls to the machine's intrinsics,
; declarations marked "intrinsic", which these entries match.

declare i32 @tiny.cycles() "intrinsic"
declare void @tiny.signal(i32, i32) "intrinsic"
declare void @tiny.put2(i32, i32) "intrinsic"

define i32 @cycles() "enc"="101010 d:4 0000000000000000000000" "asm"="cycles {d}" {
  %r = call i32 @tiny.cycles()
  ret i32 %r
}
define void @signal(i32 "reg"="r" %a, i32 "imm" %k) "enc"="101011 0000 a:4 k:s18" "asm"="signal {a}, {k}" {
  call void @tiny.signal(i32 %a, i32 %k)
  ret void
}
; A pair of registers named by the first: r5 and r6, fixed classes
; (r5c, r6c), so selection copies the values there just before it.
define void @put2(i32 "reg"="r5c" %a, i32 "reg"="r6c" %b) "enc"="101100 0101 0000000000000000000000" "asm"="put2 r5" {
  call void @tiny.put2(i32 %a, i32 %b)
  ret void
}
