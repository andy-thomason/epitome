; cycle: @is_even -> @is_odd -> @is_even
; @is_even: recursion: inlined @is_odd to break the cycle
; @is_even: recursion: 1 tail call(s) now jump back to the start
; @is_even: combine: add_add_const x1, sub_const x2
; @is_even: rotate: 1 loops rotated
; @is_even: combine: (dead) x1, icmp_eq_add x1
; @is_even: simplifycfg: 0 unreachable blocks removed, 0 empty blocks bypassed, 0 selects made, 1 blocks merged, 0 edges into unreachable dropped
; @is_even: recursion: the cycle splits into pieces of 1, 1
; @is_odd: combine: sub_const x1
; @main: constcall: 1 calls worked out
; @main: combine: (folded) x1
; @is_odd: deleted, no longer used


define i1 @is_even(i32 %n.arg) {
entry.rec:
  %zero1 = icmp eq i32 %n.arg, 0
  br i1 %zero1, label %yes, label %recur

recur:                                            ; preds = %recur.i, %entry.rec
  %n = phi i32 [ %n.arg, %entry.rec ], [ %m.i, %recur.i ]
  %zero.i = icmp eq i32 %n, 1
  br i1 %zero.i, label %no.i, label %recur.i

recur.i:                                          ; preds = %recur
  %m.i = add i32 %n, -2
  %zero = icmp eq i32 %m.i, 0
  br i1 %zero, label %yes, label %recur

no.i:                                             ; preds = %recur
  ret i1 false

yes:                                              ; preds = %recur.i, %entry.rec
  ret i1 true
}

define i32 @main() {
entry:
  ret i32 0
}
