; @report: not inlined @helper: cost 10 - bonus 6 > threshold 0 (caller only runs on the way to a panic)
; @shown: not inlined @helper: cost 10 - bonus 6 > threshold 0 (caller only runs on the way to a panic)
; @main: inlined @helper: cost 10, bonus 6, threshold 100
; @main: not inlined @report: noinline
; @main: inlined @use: cost 0, bonus 7, threshold 0
; @shown: deleted, no longer used


define internal i32 @helper(i32 %x) {
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
  ret i32 %t9
}

define internal i32 @report(i32 %x) noinline {
entry:
  %y = tail call i32 @helper(i32 %x)
  ret i32 %y
}

declare void @abort()

define void @use(ptr %p, i32 %x) {
entry:
  ret void
}

define i32 @main(i32 %argc) {
entry:
  %t0.i = add i32 %argc, 3
  %t1.i = mul i32 %t0.i, 10
  %t2.i = xor i32 %t1.i, 17
  %t3.i = add i32 %t2.i, 24
  %t4.i = mul i32 %t3.i, 31
  %t5.i = xor i32 %t4.i, 38
  %t6.i = add i32 %t5.i, 45
  %t7.i = mul i32 %t6.i, 52
  %t8.i = xor i32 %t7.i, 59
  %t9.i = add i32 %t8.i, 66
  %bad = icmp eq i32 %t9.i, 12345
  br i1 %bad, label %fail, label %ok

fail:                                             ; preds = %entry
  %r = tail call i32 @report(i32 %t9.i)
  tail call void @abort()
  unreachable

ok:                                               ; preds = %entry
  %b = and i32 %t9.i, 7
  ret i32 %b
}
