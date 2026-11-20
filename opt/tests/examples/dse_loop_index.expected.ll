; @last_depths: combine: icmp_ugt_zero x1
; @last_depths: implied: 0 comparisons folded, 0 selects resolved, 1 flags set
; @last_depths: rotate: 1 loops rotated
; @last_depths: combine: (constant to the right) x1, icmp_ugt_zero x1
; @last_depths: simplifycfg: 0 unreachable blocks removed, 0 empty blocks bypassed, 0 selects made, 1 blocks merged, 0 edges into unreachable dropped
; @last_depths: indvars: loop %body runs %n times when entered
; @last_depths: indvars: 1 exact exit tests, 0 exit values, 0 counters widened
; @main: inlined @last_depths: cost 14, bonus 26, threshold 100
; @main: combine: (folded) x2, br_true x1
; @main: indvars: loop %body.i runs 5 times when entered
; @main: indvars: 1 exact exit tests, 0 exit values, 0 counters widened


define i8 @last_depths(i64 %n) {
entry:
  %depth = alloca [8 x i8]
  %more1 = icmp ne i64 %n, 0
  br i1 %more1, label %body, label %done

body:                                             ; preds = %write, %entry
  %acc1 = phi i8 [ 0, %entry ], [ %sum, %write ]
  %len = phi i64 [ 0, %entry ], [ %next, %write ]
  %has = icmp ne i64 %len, 0
  br i1 %has, label %read, label %write

read:                                             ; preds = %body
  %prev = add nsw i64 %len, -1
  %pp = getelementptr i8, ptr %depth, i64 %prev
  %d = load i8, ptr %pp
  br label %write

write:                                            ; preds = %read, %body
  %got = phi i8 [ 0, %body ], [ %d, %read ]
  %sum = add i8 %acc1, %got
  %v = trunc i64 %len to i8
  %p = getelementptr i8, ptr %depth, i64 %len
  store i8 %v, ptr %p
  %next = add nuw i64 %len, 1
  %more = icmp ne i64 %next, %n
  br i1 %more, label %body, label %done

done:                                             ; preds = %write, %entry
  %acc2 = phi i8 [ 0, %entry ], [ %sum, %write ]
  ret i8 %acc2
}

define i32 @main() {
  %depth.i = alloca [8 x i8]
  br label %body.i

body.i:                                           ; preds = %write.i, %0
  %acc1.i = phi i8 [ 0, %0 ], [ %sum.i, %write.i ]
  %len.i = phi i64 [ 0, %0 ], [ %next.i, %write.i ]
  %has.i = icmp ne i64 %len.i, 0
  br i1 %has.i, label %read.i, label %write.i

read.i:                                           ; preds = %body.i
  %prev.i = add nsw i64 %len.i, -1
  %pp.i = getelementptr i8, ptr %depth.i, i64 %prev.i
  %d.i = load i8, ptr %pp.i
  br label %write.i

write.i:                                          ; preds = %read.i, %body.i
  %got.i = phi i8 [ 0, %body.i ], [ %d.i, %read.i ]
  %sum.i = add i8 %acc1.i, %got.i
  %v.i = trunc i64 %len.i to i8
  %p.i = getelementptr i8, ptr %depth.i, i64 %len.i
  store i8 %v.i, ptr %p.i
  %next.i = add nuw i64 %len.i, 1
  %more.i = icmp ne i64 %next.i, 5
  br i1 %more.i, label %body.i, label %done.i

done.i:                                           ; preds = %write.i
  %w = zext i8 %sum.i to i32
  ret i32 %w
}
