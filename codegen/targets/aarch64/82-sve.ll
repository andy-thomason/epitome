; SVE for selection: entries with bodies for scalable vectors, in z0-z31
; (class z, which holds them: its 128 bits are the least vector length,
; the machine's a multiple of it). Their encodings are 81-scalable.ll's.
; Functions need the feature sve.

; Copies: mov, which is orr of a register with itself.
define <vscale x 16 x i8> @MOVz(<vscale x 16 x i8> "reg"="z" %n) "feature"="sve" "copy"="z" "enc"="00000100011 n:5 001100 n:5 d:5" "asm"="mov {d}.d, {n}.d" "nodecode" {
  ret <vscale x 16 x i8> %n
}

; Integer arithmetic, unpredicated.
define <vscale x 16 x i8> @ADDzb(<vscale x 16 x i8> "reg"="z" %n, <vscale x 16 x i8> "reg"="z" %m) "feature"="sve" "enc"="00000100001 m:5 000000 n:5 d:5" "asm"="add {d}.b, {n}.b, {m}.b" "nodecode" {
  %r = add <vscale x 16 x i8> %n, %m
  ret <vscale x 16 x i8> %r
}
define <vscale x 8 x i16> @ADDzh(<vscale x 8 x i16> "reg"="z" %n, <vscale x 8 x i16> "reg"="z" %m) "feature"="sve" "enc"="00000100011 m:5 000000 n:5 d:5" "asm"="add {d}.h, {n}.h, {m}.h" "nodecode" {
  %r = add <vscale x 8 x i16> %n, %m
  ret <vscale x 8 x i16> %r
}
define <vscale x 4 x i32> @ADDzs(<vscale x 4 x i32> "reg"="z" %n, <vscale x 4 x i32> "reg"="z" %m) "feature"="sve" "enc"="00000100101 m:5 000000 n:5 d:5" "asm"="add {d}.s, {n}.s, {m}.s" "nodecode" {
  %r = add <vscale x 4 x i32> %n, %m
  ret <vscale x 4 x i32> %r
}
define <vscale x 2 x i64> @ADDzd(<vscale x 2 x i64> "reg"="z" %n, <vscale x 2 x i64> "reg"="z" %m) "feature"="sve" "enc"="00000100111 m:5 000000 n:5 d:5" "asm"="add {d}.d, {n}.d, {m}.d" "nodecode" {
  %r = add <vscale x 2 x i64> %n, %m
  ret <vscale x 2 x i64> %r
}
