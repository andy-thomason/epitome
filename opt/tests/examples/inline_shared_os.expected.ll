; @main: not inlined @mid: cost 12 - bonus 6 > threshold 0 (callee is called from elsewhere too)
; @main: not inlined @mid: cost 12 - bonus 6 > threshold 0 (callee is called from elsewhere too)
; @main: inlined @mid_once: cost 12, bonus 23, threshold 25
; @main: inlined @tiny: cost 1, bonus 6, threshold 0
; @main: inlined @tiny: cost 1, bonus 6, threshold 0
; @main: combine: (dead) x1, add_add_const x1
; @mid_once: deleted, no longer used
; @tiny: deleted, no longer used


define internal i32 @mid(i32 %x) {
entry:
  %t0 = add i32 %x, 3
  %t1 = mul i32 %t0, 10
  %t2 = xor i32 %t1, 17
  %t3 = add i32 %t2, 24
  %t4 = mul i32 %t3, 31
  %t5 = xor i32 %t4, 38
  %t6 = add i32 %t5, 45
  %t7 = mul i32 %t6, 52
  %t8 = xor i32 %t7, 59
  %t9 = add i32 %t8, 66
  %t10 = mul i32 %t9, 73
  %t11 = xor i32 %t10, 80
  ret i32 %t11
}

define i32 @main(i32 %argc) {
entry:
  %a = tail call i32 @mid(i32 %argc)
  %b = tail call i32 @mid(i32 %a)
  %t0.i = add i32 %b, 3
  %t1.i = mul i32 %t0.i, 10
  %t2.i = xor i32 %t1.i, 17
  %t3.i = add i32 %t2.i, 24
  %t4.i = mul i32 %t3.i, 31
  %t5.i = xor i32 %t4.i, 38
  %t6.i = add i32 %t5.i, 45
  %t7.i = mul i32 %t6.i, 52
  %t8.i = xor i32 %t7.i, 59
  %t9.i = add i32 %t8.i, 66
  %t10.i = mul i32 %t9.i, 73
  %t11.i = xor i32 %t10.i, 80
  %y.i1 = add i32 %t11.i, 2
  %r = and i32 %y.i1, 7
  ret i32 %r
}
