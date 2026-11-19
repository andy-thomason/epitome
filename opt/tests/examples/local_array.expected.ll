; @walk: sroa: 1 allocas split into 1 pieces
; @walk: mem2reg: 1 allocas promoted (1 loads, 2 stores removed)
; @walk: recursion: 1 accumulator call(s) now jump back to the start, folding with add
; @walk: combine: add_zero x1, sub_const x1
; @walk: rotate: 1 loops rotated
; @walk: simplifycfg: 0 unreachable blocks removed, 0 empty blocks bypassed, 0 selects made, 1 blocks merged, 0 edges into unreachable dropped
; @walk: indvars: loop %recur runs %n.arg times when entered
; @walk: indvars: 1 exact exit tests, 0 exit values, 0 counters widened
; @main: constcall: 1 calls worked out


define i32 @walk(i32 %n.arg) {
entry.rec:
  %stop1 = icmp eq i32 %n.arg, 0
  br i1 %stop1, label %done, label %recur

recur:                                            ; preds = %recur, %entry.rec
  %acc = phi i32 [ 0, %entry.rec ], [ %sum, %recur ]
  %n = phi i32 [ %n.arg, %entry.rec ], [ %m, %recur ]
  %m = add i32 %n, -1
  %sum = add i32 %acc, %n
  %stop = icmp eq i32 %m, 0
  br i1 %stop, label %done, label %recur

done:                                             ; preds = %recur, %entry.rec
  %acc1 = phi i32 [ 0, %entry.rec ], [ %sum, %recur ]
  ret i32 %acc1
}

define i32 @main() {
entry:
  ret i32 55
}
