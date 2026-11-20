; @unwrap_unchecked: simplifycfg: 1 unreachable blocks removed, 0 empty blocks bypassed, 0 selects made, 1 blocks merged, 1 edges into unreachable dropped
; @unwrap_unchecked: combine: (dead) x1
; @flipped: simplifycfg: 1 unreachable blocks removed, 1 empty blocks bypassed, 0 selects made, 1 blocks merged, 1 edges into unreachable dropped
; @switched: combine: (folded) x1
; @switched: simplifycfg: 1 unreachable blocks removed, 0 empty blocks bypassed, 0 selects made, 0 blocks merged, 3 edges into unreachable dropped
; @nowhere: simplifycfg: 3 unreachable blocks removed, 0 empty blocks bypassed, 0 selects made, 1 blocks merged, 3 edges into unreachable dropped


declare void @panic() noreturn

define i32 @unwrap_unchecked(i8 %tag, i32 %x) {
entry:
  ret i32 %x
}

define i32 @flipped(i1 %c, i32 %x) {
entry:
  ret i32 %x
}

define i32 @switched(i32 %v) {
entry:
  switch i32 %v, label %a [
    i32 2, label %b
  ]

a:                                                ; preds = %entry
  ret i32 10

b:                                                ; preds = %entry
  ret i32 20
}

define i32 @nowhere(i1 %c, i1 %d, i32 %x) {
entry:
  ret i32 %x
}

define i32 @panics(i1 %c, i32 %x) {
entry:
  br i1 %c, label %ok, label %fail

ok:                                               ; preds = %entry
  ret i32 %x

fail:                                             ; preds = %entry
  call void @panic()
  unreachable
}
