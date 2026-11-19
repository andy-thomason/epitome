; @known: combine: (folded) x2, br_true x1
; @known: simplifycfg: 1 unreachable blocks removed, 0 empty blocks bypassed, 0 selects made, 2 blocks merged, 0 edges into unreachable dropped
; @switched: prefold: 1 branches on constants folded, 2 blocks removed
; @switched: simplifycfg: 0 unreachable blocks removed, 0 empty blocks bypassed, 0 selects made, 1 blocks merged, 0 edges into unreachable dropped
; @max: simplifycfg: 1 unreachable blocks removed, 1 empty blocks bypassed, 1 selects made, 1 blocks merged, 0 edges into unreachable dropped
; @max: combine: (dead) x1
; @clamp_low: simplifycfg: 1 unreachable blocks removed, 0 empty blocks bypassed, 1 selects made, 1 blocks merged, 0 edges into unreachable dropped
; @count: combine: (dead) x1, br_not x1
; @count: simplifycfg: 0 unreachable blocks removed, 1 empty blocks bypassed, 0 selects made, 0 blocks merged, 0 edges into unreachable dropped


define i32 @known(i32 %x) {
entry:
  %a = add i32 %x, 1
  ret i32 %a
}

define i32 @switched(i32 %x) {
entry:
  %t = add i32 %x, 20
  ret i32 %t
}

define i32 @max(i32 %a, i32 %b) {
entry:
  %c = icmp sgt i32 %a, %b
  %m = select i1 %c, i32 %a, i32 %b
  ret i32 %m
}

define i32 @clamp_low(i32 %x) {
entry:
  %neg = icmp slt i32 %x, 0
  %r = select i1 %neg, i32 0, i32 %x
  ret i32 %r
}

define i32 @count(i32 %n) {
entry:
  br label %head

head:                                             ; preds = %body, %entry
  %i = phi i32 [ 0, %entry ], [ %next, %body ]
  %done = icmp sge i32 %i, %n
  br i1 %done, label %exit, label %body

body:                                             ; preds = %head
  %next = add i32 %i, 1
  br label %head

exit:                                             ; preds = %head
  ret i32 %i
}
