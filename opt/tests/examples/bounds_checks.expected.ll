; @sum: not inlined @panic_bounds_check: noinline
; @sum: combine: (dead) x2, icmp_eq_zext_one x1, icmp_ne_zext_zero x1, select_zext x1
; @sum: implied: 1 comparisons folded, 4 selects resolved, 0 flags set
; @sum: combine: (dead) x3, br_true x1
; @sum: simplifycfg: 1 unreachable blocks removed, 1 empty blocks bypassed, 0 selects made, 0 blocks merged, 0 edges into unreachable dropped
; @sum: rotate: 1 loops rotated
; @sum: combine: (constant to the right) x1, icmp_ugt_zero x1
; @sum: simplifycfg: 0 unreachable blocks removed, 0 empty blocks bypassed, 0 selects made, 1 blocks merged, 0 edges into unreachable dropped
; @sum: indvars: loop %ok runs %len times when entered
; @sum: indvars: 1 exact exit tests, 0 exit values, 0 counters widened
; @sum: lsr: 1 arrays stepped by pointer, 1 counters counting down
; @dot: not inlined @panic_bounds_check: noinline
; @dot: not inlined @panic_bounds_check: noinline
; @dot: implied: 2 comparisons folded, 7 selects resolved, 0 flags set
; @dot: combine: (dead) x4, br_true x2
; @dot: simplifycfg: 2 unreachable blocks removed, 1 empty blocks bypassed, 0 selects made, 1 blocks merged, 0 edges into unreachable dropped
; @dot: rotate: 1 loops rotated
; @dot: combine: (constant to the right) x1, icmp_ugt_zero x1
; @dot: simplifycfg: 0 unreachable blocks removed, 0 empty blocks bypassed, 0 selects made, 1 blocks merged, 0 edges into unreachable dropped
; @dot: indvars: loop %ok.a runs %n times when entered
; @dot: indvars: 1 exact exit tests, 0 exit values, 0 counters widened
; @dot: lsr: 2 arrays stepped by pointer, 1 counters counting down
; @rev: not inlined @panic_bounds_check: noinline
; @rev: not inlined @panic_bounds_check: noinline
; @rev: implied: 2 comparisons folded, 5 selects resolved, 2 flags set
; @rev: combine: (dead) x4, br_true x2
; @rev: simplifycfg: 2 unreachable blocks removed, 1 empty blocks bypassed, 0 selects made, 1 blocks merged, 0 edges into unreachable dropped
; @rev: rotate: 1 loops rotated
; @rev: combine: (constant to the right) x1, icmp_ugt_zero x1
; @rev: simplifycfg: 0 unreachable blocks removed, 0 empty blocks bypassed, 0 selects made, 1 blocks merged, 0 edges into unreachable dropped
; @rev: indvars: loop %body runs %half times when entered
; @rev: indvars: 1 exact exit tests, 0 exit values, 0 counters widened
; @kept: not inlined @panic_bounds_check: noinline
; @kept: combine: (dead) x1, (reassociated) x1
; @kept: implied: 0 comparisons folded, 0 selects resolved, 1 flags set
; @kept: rotate: 1 loops rotated
; @kept: combine: (constant to the right) x1, (dead) x1, (reassociated) x1, icmp_ugt_zero x1
; @kept: simplifycfg: 0 unreachable blocks removed, 0 empty blocks bypassed, 0 selects made, 1 blocks merged, 0 edges into unreachable dropped
; @kept: licm: 2 moved out of loops (0 loads)
; @kept: indvars: loop %body runs %n times when entered
; @kept: indvars: 1 exact exit tests, 0 exit values, 0 counters widened
; @main: not inlined @sum: noinline
; @main: not inlined @dot: noinline
; @main: not inlined @rev: noinline
; @main: not inlined @kept: noinline

target datalayout = "e-m:o-i64:64-i128:128-n32:64-S128"

define void @panic_bounds_check(i64 %i, i64 %len) noinline noreturn {
  tail call void @llvm.trap()
  unreachable
}

declare void @llvm.trap()

define i64 @sum(ptr %v, i64 %len) noinline {
start:
  %lt1 = icmp ne i64 %len, 0
  br i1 %lt1, label %ok, label %done

ok:                                               ; preds = %ok, %start
  %lsr.ptr = phi ptr [ %v, %start ], [ %lsr.ptr.next, %ok ]
  %s = phi i64 [ 0, %start ], [ %s.next, %ok ]
  %lsr.count = phi i64 [ %len, %start ], [ %lsr.count.next, %ok ]
  %x = load i64, ptr %lsr.ptr
  %s.next = add i64 %s, %x
  %lsr.count.next = add i64 %lsr.count, -1
  %lt = icmp ne i64 %lsr.count.next, 0
  %lsr.ptr.next = getelementptr i64, ptr %lsr.ptr, i64 1
  br i1 %lt, label %ok, label %done

done:                                             ; preds = %ok, %start
  %s1 = phi i64 [ 0, %start ], [ %s.next, %ok ]
  ret i64 %s1
}

define i64 @dot(ptr %a, i64 %alen, ptr %b, i64 %blen) noinline {
start:
  %shorter = icmp ult i64 %blen, %alen
  %n = select i1 %shorter, i64 %blen, i64 %alen
  %more1 = icmp ne i64 %n, 0
  br i1 %more1, label %ok.a, label %done

ok.a:                                             ; preds = %ok.a, %start
  %lsr.ptr1 = phi ptr [ %b, %start ], [ %lsr.ptr.next1, %ok.a ]
  %lsr.ptr = phi ptr [ %a, %start ], [ %lsr.ptr.next, %ok.a ]
  %s = phi i64 [ 0, %start ], [ %s.next, %ok.a ]
  %lsr.count = phi i64 [ %n, %start ], [ %lsr.count.next, %ok.a ]
  %x = load i64, ptr %lsr.ptr
  %y = load i64, ptr %lsr.ptr1
  %xy = mul i64 %x, %y
  %s.next = add i64 %s, %xy
  %lsr.count.next = add i64 %lsr.count, -1
  %more = icmp ne i64 %lsr.count.next, 0
  %lsr.ptr.next = getelementptr i64, ptr %lsr.ptr, i64 1
  %lsr.ptr.next1 = getelementptr i64, ptr %lsr.ptr1, i64 1
  br i1 %more, label %ok.a, label %done

done:                                             ; preds = %ok.a, %start
  %s1 = phi i64 [ 0, %start ], [ %s.next, %ok.a ]
  ret i64 %s1
}

define void @rev(ptr %v, i64 %n) noinline {
start:
  %half = lshr i64 %n, 1
  %last = add i64 %n, -1
  %more1 = icmp ne i64 %half, 0
  br i1 %more1, label %body, label %done

body:                                             ; preds = %body, %start
  %i = phi i64 [ 0, %start ], [ %inc, %body ]
  %j = sub nuw i64 %last, %i
  %pi = getelementptr inbounds i8, ptr %v, i64 %i
  %pj = getelementptr inbounds i8, ptr %v, i64 %j
  %x = load i8, ptr %pi
  %y = load i8, ptr %pj
  store i8 %y, ptr %pi
  store i8 %x, ptr %pj
  %inc = add nuw i64 %i, 1
  %more = icmp ne i64 %inc, %half
  br i1 %more, label %body, label %done

done:                                             ; preds = %body, %start
  ret void
}

define i64 @kept(ptr %v, i64 %len, i64 %n, i64 %m) noinline {
start:
  %more1 = icmp ne i64 %n, 0
  br i1 %more1, label %body.preheader, label %done

body.preheader:                                   ; preds = %start
  %in2 = icmp ult i64 %m, %len
  %q = getelementptr inbounds i64, ptr %v, i64 %m
  br label %body

body:                                             ; preds = %ok2, %body.preheader
  %s = phi i64 [ %s.next, %ok2 ], [ 0, %body.preheader ]
  %i = phi i64 [ %i.next, %ok2 ], [ 0, %body.preheader ]
  %in = icmp ult i64 %i, %len
  br i1 %in, label %ok, label %panic

ok:                                               ; preds = %body
  %p = getelementptr inbounds i64, ptr %v, i64 %i
  %x = load i64, ptr %p
  br i1 %in2, label %ok2, label %panic

ok2:                                              ; preds = %ok
  %y = load i64, ptr %q
  %0 = add i64 %x, %y
  %s.next = add i64 %0, %s
  %i.next = add nuw i64 %i, 1
  %more = icmp ne i64 %i.next, %n
  br i1 %more, label %body, label %done

panic:                                            ; preds = %ok, %body
  tail call void @panic_bounds_check(i64 %i, i64 %len)
  unreachable

done:                                             ; preds = %ok2, %start
  %s1 = phi i64 [ 0, %start ], [ %s.next, %ok2 ]
  ret i64 %s1
}

define i32 @main() {
  %a = alloca [5 x i64]
  %b = alloca [3 x i64]
  %c = alloca [5 x i8]
  store i64 1, ptr %a
  %a1 = getelementptr i64, ptr %a, i64 1
  store i64 2, ptr %a1
  %a2 = getelementptr i64, ptr %a, i64 2
  store i64 3, ptr %a2
  %a3 = getelementptr i64, ptr %a, i64 3
  store i64 4, ptr %a3
  %a4 = getelementptr i64, ptr %a, i64 4
  store i64 5, ptr %a4
  store i64 7, ptr %b
  %b1 = getelementptr i64, ptr %b, i64 1
  store i64 11, ptr %b1
  %b2 = getelementptr i64, ptr %b, i64 2
  store i64 13, ptr %b2
  store i8 97, ptr %c
  %c1 = getelementptr i8, ptr %c, i64 1
  store i8 98, ptr %c1
  %c2 = getelementptr i8, ptr %c, i64 2
  store i8 99, ptr %c2
  %c3 = getelementptr i8, ptr %c, i64 3
  store i8 100, ptr %c3
  %c4 = getelementptr i8, ptr %c, i64 4
  store i8 101, ptr %c4
  %s = call i64 @sum(ptr %a, i64 5)
  %d = call i64 @dot(ptr %a, i64 5, ptr %b, i64 3)
  call void @rev(ptr %c, i64 5)
  %r0 = load i8, ptr %c
  %r0w = zext i8 %r0 to i64
  %k = call i64 @kept(ptr %a, i64 5, i64 4, i64 1)
  %t1 = add i64 %s, %d
  %t2 = add i64 %t1, %r0w
  %t3 = add i64 %t2, %k
  %r = trunc i64 %t3 to i32
  ret i32 %r
}
