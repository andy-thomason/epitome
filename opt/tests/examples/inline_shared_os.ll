; epitome-opt: -Os
; At -Os a callee with other callers is inlined only where the copy is no
; bigger than the call.
;
; - @mid is under -Os's threshold of 25, but it is called twice: inlining
;   both calls would leave two copies and the original, so it stays a call.
; - @mid_once is the same size, but internal and called once: it is
;   deleted once inlined, so the program does not grow.
; - @tiny is no bigger than a call: inlined everywhere.
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

define internal i32 @mid_once(i32 %x) {
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

define internal i32 @tiny(i32 %x) {
entry:
  %y = add i32 %x, 1
  ret i32 %y
}

define i32 @main(i32 %argc) {
entry:
  %a = call i32 @mid(i32 %argc)
  %b = call i32 @mid(i32 %a)
  %c = call i32 @mid_once(i32 %b)
  %d = call i32 @tiny(i32 %c)
  %e = call i32 @tiny(i32 %d)
  %r = and i32 %e, 7
  ret i32 %r
}
