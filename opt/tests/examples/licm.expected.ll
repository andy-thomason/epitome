; @sum: licm: 2 moved out of loops (1 loads)
; @sum: indvars: loop %loop runs %n times when entered
; @sum: indvars: 1 exact exit tests, 0 exit values, 0 counters widened
; @sum: lsr: 2 arrays stepped by pointer, 1 counters counting down

target datalayout = "e-m:o-i64:64-i128:128-n32:64-S128"

@scale = global i32 3

define i32 @sum(ptr %a, ptr noalias %out, i64 %n, i32 %k) {
entry:
  %e0 = icmp sgt i64 %n, 0
  br i1 %e0, label %loop.preheader, label %done

loop.preheader:                                   ; preds = %entry
  %s = load i32, ptr @scale
  %m = mul i32 %s, %k
  br label %loop

loop:                                             ; preds = %loop, %loop.preheader
  %lsr.ptr1 = phi ptr [ %out, %loop.preheader ], [ %lsr.ptr.next1, %loop ]
  %lsr.ptr = phi ptr [ %a, %loop.preheader ], [ %lsr.ptr.next, %loop ]
  %lsr.count = phi i64 [ %lsr.count.next, %loop ], [ %n, %loop.preheader ]
  %acc = phi i32 [ %acc1, %loop ], [ 0, %loop.preheader ]
  %x = load i32, ptr %lsr.ptr
  %y = mul i32 %x, %m
  %acc1 = add i32 %acc, %y
  store i32 %y, ptr %lsr.ptr1
  %lsr.count.next = add i64 %lsr.count, -1
  %c = icmp ne i64 %lsr.count.next, 0
  %lsr.ptr.next = getelementptr i32, ptr %lsr.ptr, i64 1
  %lsr.ptr.next1 = getelementptr i32, ptr %lsr.ptr1, i64 1
  br i1 %c, label %loop, label %done

done:                                             ; preds = %loop, %entry
  %r = phi i32 [ 0, %entry ], [ %acc1, %loop ]
  ret i32 %r
}
