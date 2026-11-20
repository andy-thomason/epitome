; @clamp: simplifycfg: 1 unreachable blocks removed, 0 empty blocks bypassed, 1 selects made, 1 blocks merged, 0 edges into unreachable dropped
; @main: constcall: 3 calls worked out
; @main: combine: (folded) x2


define i32 @clamp(i32 %x) {
entry:
  %low = icmp slt i32 %x, -50
  %high = icmp sgt i32 %x, 200
  %c = select i1 %high, i32 200, i32 %x
  %r = select i1 %low, i32 -50, i32 %c
  ret i32 %r
}

define i32 @divides(i32 %x, i32 %y) {
entry:
  %zero = icmp eq i32 %y, 0
  br i1 %zero, label %done, label %div

div:                                              ; preds = %entry
  %q = sdiv i32 %x, %y
  br label %done

done:                                             ; preds = %div, %entry
  %r = phi i32 [ 0, %entry ], [ %q, %div ]
  ret i32 %r
}

define i32 @main() {
  ret i32 150
}
