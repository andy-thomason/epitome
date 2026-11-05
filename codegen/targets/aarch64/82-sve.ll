; SVE's instructions for selection. Written by gen_sve.py; do not edit.
; The conventions are those of 00-base.ll; docs/design/sve.md has the plan.

define <vscale x 16 x i8> @MOVz(<vscale x 16 x i8> "reg"="z" %n) "feature"="sve" "enc"="00000100011 n:5 001100 n:5 d:5" "asm"="mov {d}.d, {n}.d" "copy"="z" "nodecode" {
  ret <vscale x 16 x i8> %n
}

define <vscale x 16 x i8> @LDRzspill(ptr "reg"="xsp" %n, i64 "imm" %off) "feature"="sve" "enc"="1000010110 imm1:s6=off>>3 010 imm2:3=off&7 n:5 d:5" "asm"="ldr {d}, [{n}, #{off}, mul vl]" "role"="load.z" "nomatch" "nodecode" {
  %r = load <vscale x 16 x i8>, ptr %n
  ret <vscale x 16 x i8> %r
}

define void @STRzspill(<vscale x 16 x i8> "reg"="z" %t, ptr "reg"="xsp" %n, i64 "imm" %off) "feature"="sve" "enc"="1110010110 imm1:s6=off>>3 010 imm2:3=off&7 n:5 t:5" "asm"="str {t}, [{n}, #{off}, mul vl]" "role"="store.z" "nomatch" "nodecode" {
  store <vscale x 16 x i8> %t, ptr %n
  ret void
}

define i64 @ADDVL(i64 "reg"="xsp" %n, i64 "imm" %k) "feature"="sve" "enc"="00000100001 n:5 01010 k:s6 d:5" "asm"="addvl {d}, {n}, #{k}" "dclass"="xsp" "role"="addvl" "nomatch" "nodecode" {
  ret i64 %n
}

define <vscale x 16 x i8> @LDRz.nx16i8(ptr "reg"="xsp" %n) "feature"="sve" "enc"="1000010110000000010000 n:5 d:5" "asm"="ldr {d}, [{n}]" "nodecode" {
  %r = load <vscale x 16 x i8>, ptr %n
  ret <vscale x 16 x i8> %r
}

define void @STRz.nx16i8(<vscale x 16 x i8> "reg"="z" %t, ptr "reg"="xsp" %n) "feature"="sve" "enc"="1110010110000000010000 n:5 t:5" "asm"="str {t}, [{n}]" "nodecode" {
  store <vscale x 16 x i8> %t, ptr %n
  ret void
}

define <vscale x 8 x i16> @LDRz.nx8i16(ptr "reg"="xsp" %n) "feature"="sve" "enc"="1000010110000000010000 n:5 d:5" "asm"="ldr {d}, [{n}]" "nodecode" {
  %r = load <vscale x 8 x i16>, ptr %n
  ret <vscale x 8 x i16> %r
}

define void @STRz.nx8i16(<vscale x 8 x i16> "reg"="z" %t, ptr "reg"="xsp" %n) "feature"="sve" "enc"="1110010110000000010000 n:5 t:5" "asm"="str {t}, [{n}]" "nodecode" {
  store <vscale x 8 x i16> %t, ptr %n
  ret void
}

define <vscale x 4 x i32> @LDRz.nx4i32(ptr "reg"="xsp" %n) "feature"="sve" "enc"="1000010110000000010000 n:5 d:5" "asm"="ldr {d}, [{n}]" "nodecode" {
  %r = load <vscale x 4 x i32>, ptr %n
  ret <vscale x 4 x i32> %r
}

define void @STRz.nx4i32(<vscale x 4 x i32> "reg"="z" %t, ptr "reg"="xsp" %n) "feature"="sve" "enc"="1110010110000000010000 n:5 t:5" "asm"="str {t}, [{n}]" "nodecode" {
  store <vscale x 4 x i32> %t, ptr %n
  ret void
}

define <vscale x 2 x i64> @LDRz.nx2i64(ptr "reg"="xsp" %n) "feature"="sve" "enc"="1000010110000000010000 n:5 d:5" "asm"="ldr {d}, [{n}]" "nodecode" {
  %r = load <vscale x 2 x i64>, ptr %n
  ret <vscale x 2 x i64> %r
}

define void @STRz.nx2i64(<vscale x 2 x i64> "reg"="z" %t, ptr "reg"="xsp" %n) "feature"="sve" "enc"="1110010110000000010000 n:5 t:5" "asm"="str {t}, [{n}]" "nodecode" {
  store <vscale x 2 x i64> %t, ptr %n
  ret void
}

define <vscale x 8 x half> @LDRz.nx8half(ptr "reg"="xsp" %n) "feature"="sve" "enc"="1000010110000000010000 n:5 d:5" "asm"="ldr {d}, [{n}]" "nodecode" {
  %r = load <vscale x 8 x half>, ptr %n
  ret <vscale x 8 x half> %r
}

define void @STRz.nx8half(<vscale x 8 x half> "reg"="z" %t, ptr "reg"="xsp" %n) "feature"="sve" "enc"="1110010110000000010000 n:5 t:5" "asm"="str {t}, [{n}]" "nodecode" {
  store <vscale x 8 x half> %t, ptr %n
  ret void
}

define <vscale x 4 x float> @LDRz.nx4float(ptr "reg"="xsp" %n) "feature"="sve" "enc"="1000010110000000010000 n:5 d:5" "asm"="ldr {d}, [{n}]" "nodecode" {
  %r = load <vscale x 4 x float>, ptr %n
  ret <vscale x 4 x float> %r
}

define void @STRz.nx4float(<vscale x 4 x float> "reg"="z" %t, ptr "reg"="xsp" %n) "feature"="sve" "enc"="1110010110000000010000 n:5 t:5" "asm"="str {t}, [{n}]" "nodecode" {
  store <vscale x 4 x float> %t, ptr %n
  ret void
}

define <vscale x 2 x double> @LDRz.nx2double(ptr "reg"="xsp" %n) "feature"="sve" "enc"="1000010110000000010000 n:5 d:5" "asm"="ldr {d}, [{n}]" "nodecode" {
  %r = load <vscale x 2 x double>, ptr %n
  ret <vscale x 2 x double> %r
}

define void @STRz.nx2double(<vscale x 2 x double> "reg"="z" %t, ptr "reg"="xsp" %n) "feature"="sve" "enc"="1110010110000000010000 n:5 t:5" "asm"="str {t}, [{n}]" "nodecode" {
  store <vscale x 2 x double> %t, ptr %n
  ret void
}

define <vscale x 8 x bfloat> @LDRz.nx8bfloat(ptr "reg"="xsp" %n) "feature"="sve" "enc"="1000010110000000010000 n:5 d:5" "asm"="ldr {d}, [{n}]" "nodecode" {
  %r = load <vscale x 8 x bfloat>, ptr %n
  ret <vscale x 8 x bfloat> %r
}

define void @STRz.nx8bfloat(<vscale x 8 x bfloat> "reg"="z" %t, ptr "reg"="xsp" %n) "feature"="sve" "enc"="1110010110000000010000 n:5 t:5" "asm"="str {t}, [{n}]" "nodecode" {
  store <vscale x 8 x bfloat> %t, ptr %n
  ret void
}

define <vscale x 16 x i8> @ADDz.nx16i8(<vscale x 16 x i8> "reg"="z" %n, <vscale x 16 x i8> "reg"="z" %m) "feature"="sve" "enc"="00000100001 m:5 000000 n:5 d:5" "asm"="add {d}.b, {n}.b, {m}.b" "nodecode" {
  %r = add <vscale x 16 x i8> %n, %m
  ret <vscale x 16 x i8> %r
}

define <vscale x 16 x i8> @SUBz.nx16i8(<vscale x 16 x i8> "reg"="z" %n, <vscale x 16 x i8> "reg"="z" %m) "feature"="sve" "enc"="00000100001 m:5 000001 n:5 d:5" "asm"="sub {d}.b, {n}.b, {m}.b" "nodecode" {
  %r = sub <vscale x 16 x i8> %n, %m
  ret <vscale x 16 x i8> %r
}

define <vscale x 16 x i8> @ANDz.nx16i8(<vscale x 16 x i8> "reg"="z" %n, <vscale x 16 x i8> "reg"="z" %m) "feature"="sve" "enc"="00000100001 m:5 001100 n:5 d:5" "asm"="and {d}.d, {n}.d, {m}.d" "nodecode" {
  %r = and <vscale x 16 x i8> %n, %m
  ret <vscale x 16 x i8> %r
}

define <vscale x 16 x i8> @ORRz.nx16i8(<vscale x 16 x i8> "reg"="z" %n, <vscale x 16 x i8> "reg"="z" %m) "feature"="sve" "enc"="00000100011 m:5 001100 n:5 d:5" "asm"="orr {d}.d, {n}.d, {m}.d" "nodecode" {
  %r = or <vscale x 16 x i8> %n, %m
  ret <vscale x 16 x i8> %r
}

define <vscale x 16 x i8> @EORz.nx16i8(<vscale x 16 x i8> "reg"="z" %n, <vscale x 16 x i8> "reg"="z" %m) "feature"="sve" "enc"="00000100101 m:5 001100 n:5 d:5" "asm"="eor {d}.d, {n}.d, {m}.d" "nodecode" {
  %r = xor <vscale x 16 x i8> %n, %m
  ret <vscale x 16 x i8> %r
}

define <vscale x 8 x i16> @ADDz.nx8i16(<vscale x 8 x i16> "reg"="z" %n, <vscale x 8 x i16> "reg"="z" %m) "feature"="sve" "enc"="00000100011 m:5 000000 n:5 d:5" "asm"="add {d}.h, {n}.h, {m}.h" "nodecode" {
  %r = add <vscale x 8 x i16> %n, %m
  ret <vscale x 8 x i16> %r
}

define <vscale x 8 x i16> @SUBz.nx8i16(<vscale x 8 x i16> "reg"="z" %n, <vscale x 8 x i16> "reg"="z" %m) "feature"="sve" "enc"="00000100011 m:5 000001 n:5 d:5" "asm"="sub {d}.h, {n}.h, {m}.h" "nodecode" {
  %r = sub <vscale x 8 x i16> %n, %m
  ret <vscale x 8 x i16> %r
}

define <vscale x 8 x i16> @ANDz.nx8i16(<vscale x 8 x i16> "reg"="z" %n, <vscale x 8 x i16> "reg"="z" %m) "feature"="sve" "enc"="00000100001 m:5 001100 n:5 d:5" "asm"="and {d}.d, {n}.d, {m}.d" "nodecode" {
  %r = and <vscale x 8 x i16> %n, %m
  ret <vscale x 8 x i16> %r
}

define <vscale x 8 x i16> @ORRz.nx8i16(<vscale x 8 x i16> "reg"="z" %n, <vscale x 8 x i16> "reg"="z" %m) "feature"="sve" "enc"="00000100011 m:5 001100 n:5 d:5" "asm"="orr {d}.d, {n}.d, {m}.d" "nodecode" {
  %r = or <vscale x 8 x i16> %n, %m
  ret <vscale x 8 x i16> %r
}

define <vscale x 8 x i16> @EORz.nx8i16(<vscale x 8 x i16> "reg"="z" %n, <vscale x 8 x i16> "reg"="z" %m) "feature"="sve" "enc"="00000100101 m:5 001100 n:5 d:5" "asm"="eor {d}.d, {n}.d, {m}.d" "nodecode" {
  %r = xor <vscale x 8 x i16> %n, %m
  ret <vscale x 8 x i16> %r
}

define <vscale x 4 x i32> @ADDz.nx4i32(<vscale x 4 x i32> "reg"="z" %n, <vscale x 4 x i32> "reg"="z" %m) "feature"="sve" "enc"="00000100101 m:5 000000 n:5 d:5" "asm"="add {d}.s, {n}.s, {m}.s" "nodecode" {
  %r = add <vscale x 4 x i32> %n, %m
  ret <vscale x 4 x i32> %r
}

define <vscale x 4 x i32> @SUBz.nx4i32(<vscale x 4 x i32> "reg"="z" %n, <vscale x 4 x i32> "reg"="z" %m) "feature"="sve" "enc"="00000100101 m:5 000001 n:5 d:5" "asm"="sub {d}.s, {n}.s, {m}.s" "nodecode" {
  %r = sub <vscale x 4 x i32> %n, %m
  ret <vscale x 4 x i32> %r
}

define <vscale x 4 x i32> @ANDz.nx4i32(<vscale x 4 x i32> "reg"="z" %n, <vscale x 4 x i32> "reg"="z" %m) "feature"="sve" "enc"="00000100001 m:5 001100 n:5 d:5" "asm"="and {d}.d, {n}.d, {m}.d" "nodecode" {
  %r = and <vscale x 4 x i32> %n, %m
  ret <vscale x 4 x i32> %r
}

define <vscale x 4 x i32> @ORRz.nx4i32(<vscale x 4 x i32> "reg"="z" %n, <vscale x 4 x i32> "reg"="z" %m) "feature"="sve" "enc"="00000100011 m:5 001100 n:5 d:5" "asm"="orr {d}.d, {n}.d, {m}.d" "nodecode" {
  %r = or <vscale x 4 x i32> %n, %m
  ret <vscale x 4 x i32> %r
}

define <vscale x 4 x i32> @EORz.nx4i32(<vscale x 4 x i32> "reg"="z" %n, <vscale x 4 x i32> "reg"="z" %m) "feature"="sve" "enc"="00000100101 m:5 001100 n:5 d:5" "asm"="eor {d}.d, {n}.d, {m}.d" "nodecode" {
  %r = xor <vscale x 4 x i32> %n, %m
  ret <vscale x 4 x i32> %r
}

define <vscale x 2 x i64> @ADDz.nx2i64(<vscale x 2 x i64> "reg"="z" %n, <vscale x 2 x i64> "reg"="z" %m) "feature"="sve" "enc"="00000100111 m:5 000000 n:5 d:5" "asm"="add {d}.d, {n}.d, {m}.d" "nodecode" {
  %r = add <vscale x 2 x i64> %n, %m
  ret <vscale x 2 x i64> %r
}

define <vscale x 2 x i64> @SUBz.nx2i64(<vscale x 2 x i64> "reg"="z" %n, <vscale x 2 x i64> "reg"="z" %m) "feature"="sve" "enc"="00000100111 m:5 000001 n:5 d:5" "asm"="sub {d}.d, {n}.d, {m}.d" "nodecode" {
  %r = sub <vscale x 2 x i64> %n, %m
  ret <vscale x 2 x i64> %r
}

define <vscale x 2 x i64> @ANDz.nx2i64(<vscale x 2 x i64> "reg"="z" %n, <vscale x 2 x i64> "reg"="z" %m) "feature"="sve" "enc"="00000100001 m:5 001100 n:5 d:5" "asm"="and {d}.d, {n}.d, {m}.d" "nodecode" {
  %r = and <vscale x 2 x i64> %n, %m
  ret <vscale x 2 x i64> %r
}

define <vscale x 2 x i64> @ORRz.nx2i64(<vscale x 2 x i64> "reg"="z" %n, <vscale x 2 x i64> "reg"="z" %m) "feature"="sve" "enc"="00000100011 m:5 001100 n:5 d:5" "asm"="orr {d}.d, {n}.d, {m}.d" "nodecode" {
  %r = or <vscale x 2 x i64> %n, %m
  ret <vscale x 2 x i64> %r
}

define <vscale x 2 x i64> @EORz.nx2i64(<vscale x 2 x i64> "reg"="z" %n, <vscale x 2 x i64> "reg"="z" %m) "feature"="sve" "enc"="00000100101 m:5 001100 n:5 d:5" "asm"="eor {d}.d, {n}.d, {m}.d" "nodecode" {
  %r = xor <vscale x 2 x i64> %n, %m
  ret <vscale x 2 x i64> %r
}

define <vscale x 8 x half> @FADDz.nx8half(<vscale x 8 x half> "reg"="z" %n, <vscale x 8 x half> "reg"="z" %m) "feature"="sve" "enc"="01100101010 m:5 000000 n:5 d:5" "asm"="fadd {d}.h, {n}.h, {m}.h" "nodecode" {
  %r = fadd <vscale x 8 x half> %n, %m
  ret <vscale x 8 x half> %r
}

define <vscale x 8 x half> @FSUBz.nx8half(<vscale x 8 x half> "reg"="z" %n, <vscale x 8 x half> "reg"="z" %m) "feature"="sve" "enc"="01100101010 m:5 000001 n:5 d:5" "asm"="fsub {d}.h, {n}.h, {m}.h" "nodecode" {
  %r = fsub <vscale x 8 x half> %n, %m
  ret <vscale x 8 x half> %r
}

define <vscale x 8 x half> @FMULz.nx8half(<vscale x 8 x half> "reg"="z" %n, <vscale x 8 x half> "reg"="z" %m) "feature"="sve" "enc"="01100101010 m:5 000010 n:5 d:5" "asm"="fmul {d}.h, {n}.h, {m}.h" "nodecode" {
  %r = fmul <vscale x 8 x half> %n, %m
  ret <vscale x 8 x half> %r
}

define <vscale x 4 x float> @FADDz.nx4float(<vscale x 4 x float> "reg"="z" %n, <vscale x 4 x float> "reg"="z" %m) "feature"="sve" "enc"="01100101100 m:5 000000 n:5 d:5" "asm"="fadd {d}.s, {n}.s, {m}.s" "nodecode" {
  %r = fadd <vscale x 4 x float> %n, %m
  ret <vscale x 4 x float> %r
}

define <vscale x 4 x float> @FSUBz.nx4float(<vscale x 4 x float> "reg"="z" %n, <vscale x 4 x float> "reg"="z" %m) "feature"="sve" "enc"="01100101100 m:5 000001 n:5 d:5" "asm"="fsub {d}.s, {n}.s, {m}.s" "nodecode" {
  %r = fsub <vscale x 4 x float> %n, %m
  ret <vscale x 4 x float> %r
}

define <vscale x 4 x float> @FMULz.nx4float(<vscale x 4 x float> "reg"="z" %n, <vscale x 4 x float> "reg"="z" %m) "feature"="sve" "enc"="01100101100 m:5 000010 n:5 d:5" "asm"="fmul {d}.s, {n}.s, {m}.s" "nodecode" {
  %r = fmul <vscale x 4 x float> %n, %m
  ret <vscale x 4 x float> %r
}

define <vscale x 2 x double> @FADDz.nx2double(<vscale x 2 x double> "reg"="z" %n, <vscale x 2 x double> "reg"="z" %m) "feature"="sve" "enc"="01100101110 m:5 000000 n:5 d:5" "asm"="fadd {d}.d, {n}.d, {m}.d" "nodecode" {
  %r = fadd <vscale x 2 x double> %n, %m
  ret <vscale x 2 x double> %r
}

define <vscale x 2 x double> @FSUBz.nx2double(<vscale x 2 x double> "reg"="z" %n, <vscale x 2 x double> "reg"="z" %m) "feature"="sve" "enc"="01100101110 m:5 000001 n:5 d:5" "asm"="fsub {d}.d, {n}.d, {m}.d" "nodecode" {
  %r = fsub <vscale x 2 x double> %n, %m
  ret <vscale x 2 x double> %r
}

define <vscale x 2 x double> @FMULz.nx2double(<vscale x 2 x double> "reg"="z" %n, <vscale x 2 x double> "reg"="z" %m) "feature"="sve" "enc"="01100101110 m:5 000010 n:5 d:5" "asm"="fmul {d}.d, {n}.d, {m}.d" "nodecode" {
  %r = fmul <vscale x 2 x double> %n, %m
  ret <vscale x 2 x double> %r
}
