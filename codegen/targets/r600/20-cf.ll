; Control-flow instructions (CF_WORD0/1, CF_ALU_WORD0/1), memory exports
; (CF_ALLOC_EXPORT_WORD0_RAT/1_BUF) and vertex fetches (VTX_WORD0/1/2). Written by
; gen_r600.py.

define void @TEX(i32 "imm" %count, i32 "imm" %addr) "enc"="1 0 00000001 0 0 0000 count:6 00 00000 000 00000 000 addr:24" "asm"="TEX {count} @{addr}" "asmonly" {
  ret void
}
define void @VTX(i32 "imm" %count, i32 "imm" %addr) "enc"="1 0 00000010 0 0 0000 count:6 00 00000 000 00000 000 addr:24" "asm"="VTX {count} @{addr}" "asmonly" {
  ret void
}
define void @GDS(i32 "imm" %count, i32 "imm" %addr) "enc"="1 0 00000011 0 0 0000 count:6 00 00000 000 00000 000 addr:24" "asm"="GDS {count} @{addr}" "asmonly" {
  ret void
}
define void @LOOP_START(i32 "imm" %addr) "enc"="1 0 00000100 0 0 0000 000000 00 00000 000 00000 000 addr:24" "asm"="LOOP_START @{addr}" "asmonly" {
  ret void
}
define void @LOOP_START.pop(i32 "imm" %addr, i32 "imm" %pop) "enc"="1 0 00000100 0 0 0000 000000 00 00000 pop:3 00000 000 addr:24" "asm"="LOOP_START @{addr} POP:{pop}" "asmonly" {
  ret void
}
define void @END_LOOP(i32 "imm" %addr) "enc"="1 0 00000101 0 0 0000 000000 00 00000 000 00000 000 addr:24" "asm"="END_LOOP @{addr}" "asmonly" {
  ret void
}
define void @END_LOOP.pop(i32 "imm" %addr, i32 "imm" %pop) "enc"="1 0 00000101 0 0 0000 000000 00 00000 pop:3 00000 000 addr:24" "asm"="END_LOOP @{addr} POP:{pop}" "asmonly" {
  ret void
}
define void @LOOP_START_DX10(i32 "imm" %addr) "enc"="1 0 00000110 0 0 0000 000000 00 00000 000 00000 000 addr:24" "asm"="LOOP_START_DX10 @{addr}" "asmonly" {
  ret void
}
define void @LOOP_START_DX10.pop(i32 "imm" %addr, i32 "imm" %pop) "enc"="1 0 00000110 0 0 0000 000000 00 00000 pop:3 00000 000 addr:24" "asm"="LOOP_START_DX10 @{addr} POP:{pop}" "asmonly" {
  ret void
}
define void @LOOP_START_NO_AL(i32 "imm" %addr) "enc"="1 0 00000111 0 0 0000 000000 00 00000 000 00000 000 addr:24" "asm"="LOOP_START_NO_AL @{addr}" "asmonly" {
  ret void
}
define void @LOOP_START_NO_AL.pop(i32 "imm" %addr, i32 "imm" %pop) "enc"="1 0 00000111 0 0 0000 000000 00 00000 pop:3 00000 000 addr:24" "asm"="LOOP_START_NO_AL @{addr} POP:{pop}" "asmonly" {
  ret void
}
define void @LOOP_CONTINUE(i32 "imm" %addr) "enc"="1 0 00001000 0 0 0000 000000 00 00000 000 00000 000 addr:24" "asm"="LOOP_CONTINUE @{addr}" "asmonly" {
  ret void
}
define void @LOOP_CONTINUE.pop(i32 "imm" %addr, i32 "imm" %pop) "enc"="1 0 00001000 0 0 0000 000000 00 00000 pop:3 00000 000 addr:24" "asm"="LOOP_CONTINUE @{addr} POP:{pop}" "asmonly" {
  ret void
}
define void @LOOP_BREAK(i32 "imm" %addr) "enc"="1 0 00001001 0 0 0000 000000 00 00000 000 00000 000 addr:24" "asm"="LOOP_BREAK @{addr}" "asmonly" {
  ret void
}
define void @LOOP_BREAK.pop(i32 "imm" %addr, i32 "imm" %pop) "enc"="1 0 00001001 0 0 0000 000000 00 00000 pop:3 00000 000 addr:24" "asm"="LOOP_BREAK @{addr} POP:{pop}" "asmonly" {
  ret void
}
define void @JUMP(i32 "imm" %addr) "enc"="1 0 00001010 0 0 0000 000000 00 00000 000 00000 000 addr:24" "asm"="JUMP @{addr}" "asmonly" {
  ret void
}
define void @JUMP.pop(i32 "imm" %addr, i32 "imm" %pop) "enc"="1 0 00001010 0 0 0000 000000 00 00000 pop:3 00000 000 addr:24" "asm"="JUMP @{addr} POP:{pop}" "asmonly" {
  ret void
}
define void @PUSH(i32 "imm" %addr) "enc"="1 0 00001011 0 0 0000 000000 00 00000 000 00000 000 addr:24" "asm"="PUSH @{addr}" "asmonly" {
  ret void
}
define void @PUSH.pop(i32 "imm" %addr, i32 "imm" %pop) "enc"="1 0 00001011 0 0 0000 000000 00 00000 pop:3 00000 000 addr:24" "asm"="PUSH @{addr} POP:{pop}" "asmonly" {
  ret void
}
define void @ELSE(i32 "imm" %addr) "enc"="1 0 00001101 0 0 0000 000000 00 00000 000 00000 000 addr:24" "asm"="ELSE @{addr}" "asmonly" {
  ret void
}
define void @ELSE.pop(i32 "imm" %addr, i32 "imm" %pop) "enc"="1 0 00001101 0 0 0000 000000 00 00000 pop:3 00000 000 addr:24" "asm"="ELSE @{addr} POP:{pop}" "asmonly" {
  ret void
}
define void @POP(i32 "imm" %addr) "enc"="1 0 00001110 0 0 0000 000000 00 00000 000 00000 000 addr:24" "asm"="POP @{addr}" "asmonly" {
  ret void
}
define void @POP.pop(i32 "imm" %addr, i32 "imm" %pop) "enc"="1 0 00001110 0 0 0000 000000 00 00000 pop:3 00000 000 addr:24" "asm"="POP @{addr} POP:{pop}" "asmonly" {
  ret void
}
define void @CALL(i32 "imm" %addr) "enc"="1 0 00010010 0 0 0000 000000 00 00000 000 00000 000 addr:24" "asm"="CALL @{addr}" "asmonly" {
  ret void
}
define void @CALL.pop(i32 "imm" %addr, i32 "imm" %pop) "enc"="1 0 00010010 0 0 0000 000000 00 00000 pop:3 00000 000 addr:24" "asm"="CALL @{addr} POP:{pop}" "asmonly" {
  ret void
}
define void @RETURN(i32 "imm" %addr) "enc"="1 0 00010100 0 0 0000 000000 00 00000 000 00000 000 addr:24" "asm"="RETURN @{addr}" "asmonly" {
  ret void
}
define void @RETURN.pop(i32 "imm" %addr, i32 "imm" %pop) "enc"="1 0 00010100 0 0 0000 000000 00 00000 pop:3 00000 000 addr:24" "asm"="RETURN @{addr} POP:{pop}" "asmonly" {
  ret void
}
define void @KILL(i32 "imm" %addr) "enc"="1 0 00011000 0 0 0000 000000 00 00000 000 00000 000 addr:24" "asm"="KILL @{addr}" "asmonly" {
  ret void
}
define void @KILL.pop(i32 "imm" %addr, i32 "imm" %pop) "enc"="1 0 00011000 0 0 0000 000000 00 00000 pop:3 00000 000 addr:24" "asm"="KILL @{addr} POP:{pop}" "asmonly" {
  ret void
}
define void @WAIT_ACK(i32 "imm" %addr) "enc"="1 0 00011010 0 0 0000 000000 00 00000 000 00000 000 addr:24" "asm"="WAIT_ACK @{addr}" "asmonly" {
  ret void
}
define void @WAIT_ACK.pop(i32 "imm" %addr, i32 "imm" %pop) "enc"="1 0 00011010 0 0 0000 000000 00 00000 pop:3 00000 000 addr:24" "asm"="WAIT_ACK @{addr} POP:{pop}" "asmonly" {
  ret void
}
define void @TEX_ACK(i32 "imm" %count, i32 "imm" %addr) "enc"="1 0 00011011 0 0 0000 count:6 00 00000 000 00000 000 addr:24" "asm"="TEX_ACK {count} @{addr}" "asmonly" {
  ret void
}
define void @VTX_ACK(i32 "imm" %count, i32 "imm" %addr) "enc"="1 0 00011100 0 0 0000 count:6 00 00000 000 00000 000 addr:24" "asm"="VTX_ACK {count} @{addr}" "asmonly" {
  ret void
}

define void @CF_END() "enc"="1 0 00000000 1 0 0000 000000 00 00000 000 00000 000 000000000000000000000000" "asm"="CF_END" "asmonly" {
  ret void
}

define void @PAD() "enc"="0000000000000000000000000000000000000000000000000000000000000000" "asm"="PAD" "asmonly" {
  ret void
}

define void @ALU.00(i32 "imm" %count, i32 "imm" %addr) "enc"="1 0 1000 0 count:7 00000000 00000000 00 00 0000 0000 addr:22" "asm"="ALU {count}, @{addr}, KC0[], KC1[]" "asmonly" {
  ret void
}
define void @ALU.02(i32 "imm" %count, i32 "imm" %addr, i32 "imm" %kb1, i32 "imm" %ka1) "enc"="1 0 1000 0 count:7 ka1:8 00000000 10 00 kb1:4 0000 addr:22" "asm"="ALU {count}, @{addr}, KC0[], KC1[CB{kb1}:{=ka1<<4}-{=(ka1<<4)+32}]" "asmonly" {
  ret void
}
define void @ALU.20(i32 "imm" %count, i32 "imm" %addr, i32 "imm" %kb0, i32 "imm" %ka0) "enc"="1 0 1000 0 count:7 00000000 ka0:8 00 10 0000 kb0:4 addr:22" "asm"="ALU {count}, @{addr}, KC0[CB{kb0}:{=ka0<<4}-{=(ka0<<4)+32}], KC1[]" "asmonly" {
  ret void
}
define void @ALU.22(i32 "imm" %count, i32 "imm" %addr, i32 "imm" %kb0, i32 "imm" %ka0, i32 "imm" %kb1, i32 "imm" %ka1) "enc"="1 0 1000 0 count:7 ka1:8 ka0:8 10 10 kb1:4 kb0:4 addr:22" "asm"="ALU {count}, @{addr}, KC0[CB{kb0}:{=ka0<<4}-{=(ka0<<4)+32}], KC1[CB{kb1}:{=ka1<<4}-{=(ka1<<4)+32}]" "asmonly" {
  ret void
}
define void @ALU_PUSH_BEFORE.00(i32 "imm" %count, i32 "imm" %addr) "enc"="1 0 1001 0 count:7 00000000 00000000 00 00 0000 0000 addr:22" "asm"="ALU_PUSH_BEFORE {count}, @{addr}, KC0[], KC1[]" "asmonly" {
  ret void
}
define void @ALU_PUSH_BEFORE.02(i32 "imm" %count, i32 "imm" %addr, i32 "imm" %kb1, i32 "imm" %ka1) "enc"="1 0 1001 0 count:7 ka1:8 00000000 10 00 kb1:4 0000 addr:22" "asm"="ALU_PUSH_BEFORE {count}, @{addr}, KC0[], KC1[CB{kb1}:{=ka1<<4}-{=(ka1<<4)+32}]" "asmonly" {
  ret void
}
define void @ALU_PUSH_BEFORE.20(i32 "imm" %count, i32 "imm" %addr, i32 "imm" %kb0, i32 "imm" %ka0) "enc"="1 0 1001 0 count:7 00000000 ka0:8 00 10 0000 kb0:4 addr:22" "asm"="ALU_PUSH_BEFORE {count}, @{addr}, KC0[CB{kb0}:{=ka0<<4}-{=(ka0<<4)+32}], KC1[]" "asmonly" {
  ret void
}
define void @ALU_PUSH_BEFORE.22(i32 "imm" %count, i32 "imm" %addr, i32 "imm" %kb0, i32 "imm" %ka0, i32 "imm" %kb1, i32 "imm" %ka1) "enc"="1 0 1001 0 count:7 ka1:8 ka0:8 10 10 kb1:4 kb0:4 addr:22" "asm"="ALU_PUSH_BEFORE {count}, @{addr}, KC0[CB{kb0}:{=ka0<<4}-{=(ka0<<4)+32}], KC1[CB{kb1}:{=ka1<<4}-{=(ka1<<4)+32}]" "asmonly" {
  ret void
}
define void @ALU_POP_AFTER.00(i32 "imm" %count, i32 "imm" %addr) "enc"="1 0 1010 0 count:7 00000000 00000000 00 00 0000 0000 addr:22" "asm"="ALU_POP_AFTER {count}, @{addr}, KC0[], KC1[]" "asmonly" {
  ret void
}
define void @ALU_POP_AFTER.02(i32 "imm" %count, i32 "imm" %addr, i32 "imm" %kb1, i32 "imm" %ka1) "enc"="1 0 1010 0 count:7 ka1:8 00000000 10 00 kb1:4 0000 addr:22" "asm"="ALU_POP_AFTER {count}, @{addr}, KC0[], KC1[CB{kb1}:{=ka1<<4}-{=(ka1<<4)+32}]" "asmonly" {
  ret void
}
define void @ALU_POP_AFTER.20(i32 "imm" %count, i32 "imm" %addr, i32 "imm" %kb0, i32 "imm" %ka0) "enc"="1 0 1010 0 count:7 00000000 ka0:8 00 10 0000 kb0:4 addr:22" "asm"="ALU_POP_AFTER {count}, @{addr}, KC0[CB{kb0}:{=ka0<<4}-{=(ka0<<4)+32}], KC1[]" "asmonly" {
  ret void
}
define void @ALU_POP_AFTER.22(i32 "imm" %count, i32 "imm" %addr, i32 "imm" %kb0, i32 "imm" %ka0, i32 "imm" %kb1, i32 "imm" %ka1) "enc"="1 0 1010 0 count:7 ka1:8 ka0:8 10 10 kb1:4 kb0:4 addr:22" "asm"="ALU_POP_AFTER {count}, @{addr}, KC0[CB{kb0}:{=ka0<<4}-{=(ka0<<4)+32}], KC1[CB{kb1}:{=ka1<<4}-{=(ka1<<4)+32}]" "asmonly" {
  ret void
}
define void @ALU_POP2_AFTER.00(i32 "imm" %count, i32 "imm" %addr) "enc"="1 0 1011 0 count:7 00000000 00000000 00 00 0000 0000 addr:22" "asm"="ALU_POP2_AFTER {count}, @{addr}, KC0[], KC1[]" "asmonly" {
  ret void
}
define void @ALU_POP2_AFTER.02(i32 "imm" %count, i32 "imm" %addr, i32 "imm" %kb1, i32 "imm" %ka1) "enc"="1 0 1011 0 count:7 ka1:8 00000000 10 00 kb1:4 0000 addr:22" "asm"="ALU_POP2_AFTER {count}, @{addr}, KC0[], KC1[CB{kb1}:{=ka1<<4}-{=(ka1<<4)+32}]" "asmonly" {
  ret void
}
define void @ALU_POP2_AFTER.20(i32 "imm" %count, i32 "imm" %addr, i32 "imm" %kb0, i32 "imm" %ka0) "enc"="1 0 1011 0 count:7 00000000 ka0:8 00 10 0000 kb0:4 addr:22" "asm"="ALU_POP2_AFTER {count}, @{addr}, KC0[CB{kb0}:{=ka0<<4}-{=(ka0<<4)+32}], KC1[]" "asmonly" {
  ret void
}
define void @ALU_POP2_AFTER.22(i32 "imm" %count, i32 "imm" %addr, i32 "imm" %kb0, i32 "imm" %ka0, i32 "imm" %kb1, i32 "imm" %ka1) "enc"="1 0 1011 0 count:7 ka1:8 ka0:8 10 10 kb1:4 kb0:4 addr:22" "asm"="ALU_POP2_AFTER {count}, @{addr}, KC0[CB{kb0}:{=ka0<<4}-{=(ka0<<4)+32}], KC1[CB{kb1}:{=ka1<<4}-{=(ka1<<4)+32}]" "asmonly" {
  ret void
}
define void @ALU_CONTINUE.00(i32 "imm" %count, i32 "imm" %addr) "enc"="1 0 1101 0 count:7 00000000 00000000 00 00 0000 0000 addr:22" "asm"="ALU_CONTINUE {count}, @{addr}, KC0[], KC1[]" "asmonly" {
  ret void
}
define void @ALU_CONTINUE.02(i32 "imm" %count, i32 "imm" %addr, i32 "imm" %kb1, i32 "imm" %ka1) "enc"="1 0 1101 0 count:7 ka1:8 00000000 10 00 kb1:4 0000 addr:22" "asm"="ALU_CONTINUE {count}, @{addr}, KC0[], KC1[CB{kb1}:{=ka1<<4}-{=(ka1<<4)+32}]" "asmonly" {
  ret void
}
define void @ALU_CONTINUE.20(i32 "imm" %count, i32 "imm" %addr, i32 "imm" %kb0, i32 "imm" %ka0) "enc"="1 0 1101 0 count:7 00000000 ka0:8 00 10 0000 kb0:4 addr:22" "asm"="ALU_CONTINUE {count}, @{addr}, KC0[CB{kb0}:{=ka0<<4}-{=(ka0<<4)+32}], KC1[]" "asmonly" {
  ret void
}
define void @ALU_CONTINUE.22(i32 "imm" %count, i32 "imm" %addr, i32 "imm" %kb0, i32 "imm" %ka0, i32 "imm" %kb1, i32 "imm" %ka1) "enc"="1 0 1101 0 count:7 ka1:8 ka0:8 10 10 kb1:4 kb0:4 addr:22" "asm"="ALU_CONTINUE {count}, @{addr}, KC0[CB{kb0}:{=ka0<<4}-{=(ka0<<4)+32}], KC1[CB{kb1}:{=ka1<<4}-{=(ka1<<4)+32}]" "asmonly" {
  ret void
}
define void @ALU_BREAK.00(i32 "imm" %count, i32 "imm" %addr) "enc"="1 0 1110 0 count:7 00000000 00000000 00 00 0000 0000 addr:22" "asm"="ALU_BREAK {count}, @{addr}, KC0[], KC1[]" "asmonly" {
  ret void
}
define void @ALU_BREAK.02(i32 "imm" %count, i32 "imm" %addr, i32 "imm" %kb1, i32 "imm" %ka1) "enc"="1 0 1110 0 count:7 ka1:8 00000000 10 00 kb1:4 0000 addr:22" "asm"="ALU_BREAK {count}, @{addr}, KC0[], KC1[CB{kb1}:{=ka1<<4}-{=(ka1<<4)+32}]" "asmonly" {
  ret void
}
define void @ALU_BREAK.20(i32 "imm" %count, i32 "imm" %addr, i32 "imm" %kb0, i32 "imm" %ka0) "enc"="1 0 1110 0 count:7 00000000 ka0:8 00 10 0000 kb0:4 addr:22" "asm"="ALU_BREAK {count}, @{addr}, KC0[CB{kb0}:{=ka0<<4}-{=(ka0<<4)+32}], KC1[]" "asmonly" {
  ret void
}
define void @ALU_BREAK.22(i32 "imm" %count, i32 "imm" %addr, i32 "imm" %kb0, i32 "imm" %ka0, i32 "imm" %kb1, i32 "imm" %ka1) "enc"="1 0 1110 0 count:7 ka1:8 ka0:8 10 10 kb1:4 kb0:4 addr:22" "asm"="ALU_BREAK {count}, @{addr}, KC0[CB{kb0}:{=ka0<<4}-{=(ka0<<4)+32}], KC1[CB{kb1}:{=ka1<<4}-{=(ka1<<4)+32}]" "asmonly" {
  ret void
}
define void @ALU_ELSE_AFTER.00(i32 "imm" %count, i32 "imm" %addr) "enc"="1 0 1111 0 count:7 00000000 00000000 00 00 0000 0000 addr:22" "asm"="ALU_ELSE_AFTER {count}, @{addr}, KC0[], KC1[]" "asmonly" {
  ret void
}
define void @ALU_ELSE_AFTER.02(i32 "imm" %count, i32 "imm" %addr, i32 "imm" %kb1, i32 "imm" %ka1) "enc"="1 0 1111 0 count:7 ka1:8 00000000 10 00 kb1:4 0000 addr:22" "asm"="ALU_ELSE_AFTER {count}, @{addr}, KC0[], KC1[CB{kb1}:{=ka1<<4}-{=(ka1<<4)+32}]" "asmonly" {
  ret void
}
define void @ALU_ELSE_AFTER.20(i32 "imm" %count, i32 "imm" %addr, i32 "imm" %kb0, i32 "imm" %ka0) "enc"="1 0 1111 0 count:7 00000000 ka0:8 00 10 0000 kb0:4 addr:22" "asm"="ALU_ELSE_AFTER {count}, @{addr}, KC0[CB{kb0}:{=ka0<<4}-{=(ka0<<4)+32}], KC1[]" "asmonly" {
  ret void
}
define void @ALU_ELSE_AFTER.22(i32 "imm" %count, i32 "imm" %addr, i32 "imm" %kb0, i32 "imm" %ka0, i32 "imm" %kb1, i32 "imm" %ka1) "enc"="1 0 1111 0 count:7 ka1:8 ka0:8 10 10 kb1:4 kb0:4 addr:22" "asm"="ALU_ELSE_AFTER {count}, @{addr}, KC0[CB{kb0}:{=ka0<<4}-{=(ka0<<4)+32}], KC1[CB{kb1}:{=ka1<<4}-{=(ka1<<4)+32}]" "asmonly" {
  ret void
}
define void @MEM_RAT_CACHELESS_STORE_RAW(i32 "enum"="gprmask" %rw, i32 "enum"="gprx" %ix, i32 "imm" %eop) "enc"="1 0 01010111 eop:1 0 0000 cm:4=rw&15 000000000000 00 ix:7 0 rg:7=rw>>4 01 00 0 000010 0000" "asm"="MEM_RAT_CACHELESS STORE_RAW {rw}, {ix}, {eop}" "asmonly" {
  ret void
}
define void @VTX_READ_32(i32 "enum"="gprx" %vd, i32 "enum"="dst" %vs, i32 "imm" %off, i32 "imm" %buf) "enc"="0 0 01 001101 0 111 111 111 000 0 0 vd:7 000100 sc:2=vs&3 0 sg:7=vs>>2 buf:8 0 10 00000 | 00000000000000000000000000000000 000000000000 1 000 off:16" "asm"="VTX_READ_32 {vd}, {vs}, {off}, #{buf}" "asmonly" {
  ret void
}
