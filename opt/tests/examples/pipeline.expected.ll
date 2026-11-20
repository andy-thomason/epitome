; @fact: recursion: 1 accumulator call(s) now jump back to the start, folding with mul
; @fact: combine: mul_one x1, sub_const x1
; @fact: rotate: 1 loops rotated
; @fact: simplifycfg: 0 unreachable blocks removed, 0 empty blocks bypassed, 0 selects made, 1 blocks merged, 0 edges into unreachable dropped
; @fact: indvars: loop %recur runs %n.arg - 1 times when entered
; @main: inlined @fact: cost 6, bonus 18, threshold 100
; @main: combine: (folded) x2, br_false x1
; @main: indvars: loop %recur.i runs 9 times when entered
; @fact: deleted, no longer used


define i32 @main() {
entry:
  br label %recur.i

recur.i:                                          ; preds = %recur.i, %entry
  %acc.i = phi i64 [ 1, %entry ], [ %p.i, %recur.i ]
  %n.i = phi i64 [ 10, %entry ], [ %m.i, %recur.i ]
  %m.i = add i64 %n.i, -1
  %p.i = mul i64 %acc.i, %n.i
  %base.i = icmp ule i64 %m.i, 1
  br i1 %base.i, label %one.i, label %recur.i

one.i:                                            ; preds = %recur.i
  %x = trunc i64 %p.i to i32
  ret i32 %x
}
