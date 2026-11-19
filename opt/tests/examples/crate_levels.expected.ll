; @main: inlined @helper: cost 1, bonus 8, threshold 100
; @main: combine: (folded) x1


define internal i32 @helper(i32 %x) {
  %y = add i32 %x, 1
  ret i32 %y
}

define i32 @debug(i32 %x) #0 {
  %a = add i32 %x, 0
  %r = call i32 @helper(i32 %a)
  ret i32 %r
}

define i32 @main() {
  %d = tail call i32 @debug(i32 0)
  %s = sub i32 42, %d
  ret i32 %s
}

attributes #0 = { "epitome-opt-level"="0" }
