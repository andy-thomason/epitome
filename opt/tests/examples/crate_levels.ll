; A function at "epitome-opt-level"="0" (a crate built without
; optimisation, beside optimised ones) is left as it is: nothing is
; inlined into it, its `add 0` stays, and @helper, which it calls, is kept
; (and not changed) although @main's call of it is inlined.

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
  %r = call i32 @helper(i32 41)
  %d = call i32 @debug(i32 0)
  %s = sub i32 %r, %d
  ret i32 %s
}

attributes #0 = { "epitome-opt-level"="0" }
