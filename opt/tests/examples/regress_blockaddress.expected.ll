; @caller: inlined @small: cost 1, bonus 12, threshold 100
; @main: not inlined @caller: its blocks have their address taken
; @main: constcall: 1 calls worked out
; @small: deleted, no longer used


@tbl = global ptr blockaddress(@caller, %L)

declare i32 @printf(ptr, ...)

define i32 @caller(i32 %a) {
entry:
  br label %0

0:                                                ; preds = %entry
  %y.i = add i32 %a, 1
  br label %small.exit

small.exit:                                       ; preds = %0
  br label %L

L:                                                ; preds = %small.exit
  ret i32 %y.i
}

define i32 @main() {
  ret i32 2
}
