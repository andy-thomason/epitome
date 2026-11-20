; @mul: combine: (constant to the right) x3, add_zero x3
; @mul: simplifycfg: 0 unreachable blocks removed, 1 empty blocks bypassed, 0 selects made, 0 blocks merged, 0 edges into unreachable dropped
; @mul: implied: 0 comparisons folded, 0 selects resolved, 2 flags set
; @mul: rotate: 2 loops rotated
; @mul: combine: (folded) x2, br_true x2
; @mul: simplifycfg: 0 unreachable blocks removed, 0 empty blocks bypassed, 0 selects made, 2 blocks merged, 0 edges into unreachable dropped
; @mul: licm: 3 moved out of loops (0 loads)
; @mul: indvars: loop %loop.preheader runs 64 times when entered
; @mul: indvars: 2 exact exit tests, 0 exit values, 0 counters widened
; @mul: lsr: 3 arrays stepped by pointer, 1 counters counting down
; @mul: cse: 2 merged, 0 loads reused, 0 stores forwarded
; @sum: combine: (constant to the right) x6, (dead) x3, (reassociated) x3, add_zero x8
; @sum: implied: 0 comparisons folded, 0 selects resolved, 13 flags set
; @sum: rotate: 2 loops rotated
; @sum: combine: (dead) x3, (folded) x6, (reassociated) x3, br_true x2
; @sum: simplifycfg: 0 unreachable blocks removed, 0 empty blocks bypassed, 0 selects made, 2 blocks merged, 0 edges into unreachable dropped
; @sum: licm: 4 moved out of loops (0 loads)
; @sum: indvars: loop a loop runs 64 times when entered
; @sum: indvars: 2 exact exit tests, 0 exit values, 0 counters widened
; @sum: lsr: 1 arrays stepped by pointer, 1 counters counting down
; @sum: combine: (dead) x3
; @sum: cse: 1 merged, 0 loads reused, 0 stores forwarded
; @main: inlined @mul: cost 14, bonus 8, threshold 100
; @main: inlined @sum: cost 30, bonus 7, threshold 100
; @main: simplifycfg: 0 unreachable blocks removed, 2 empty blocks bypassed, 0 selects made, 0 blocks merged, 0 edges into unreachable dropped
; @main: indvars: loop %fill runs 98304 times when entered
; @main: indvars: loop %loop.preheader.i runs 64 times when entered
; @main: indvars: loop a loop runs 64 times when entered
; @main: indvars: 3 exact exit tests, 0 exit values, 0 counters widened


@a = global [98304 x float] zeroinitializer
@b = global [98304 x float] zeroinitializer
@c = global [98304 x float] zeroinitializer
@s = global [64 x float] zeroinitializer

define void @mul(ptr %0, ptr %1, ptr %2) {
  br label %loop.preheader

loop.preheader:                                   ; preds = %11, %3
  %4 = phi i64 [ 0, %3 ], [ %12, %11 ]
  %5 = mul i64 %4, 1536
  %lsr.start = getelementptr float, ptr %2, i64 %5
  %lsr.start1 = getelementptr float, ptr %1, i64 %5
  %lsr.start2 = getelementptr float, ptr %0, i64 %5
  br label %6

6:                                                ; preds = %6, %loop.preheader
  %lsr.ptr2 = phi ptr [ %lsr.start2, %loop.preheader ], [ %lsr.ptr.next2, %6 ]
  %lsr.ptr1 = phi ptr [ %lsr.start1, %loop.preheader ], [ %lsr.ptr.next1, %6 ]
  %lsr.ptr = phi ptr [ %lsr.start, %loop.preheader ], [ %lsr.ptr.next, %6 ]
  %lsr.count = phi i64 [ 1536, %loop.preheader ], [ %lsr.count.next, %6 ]
  %7 = load <4 x float>, ptr %lsr.ptr2, align 4
  %8 = load <4 x float>, ptr %lsr.ptr1, align 4
  %9 = fmul <4 x float> %7, %8
  store <4 x float> %9, ptr %lsr.ptr, align 4
  %lsr.count.next = add i64 %lsr.count, -4
  %10 = icmp ne i64 %lsr.count.next, 0
  %lsr.ptr.next = getelementptr float, ptr %lsr.ptr, i64 4
  %lsr.ptr.next1 = getelementptr float, ptr %lsr.ptr1, i64 4
  %lsr.ptr.next2 = getelementptr float, ptr %lsr.ptr2, i64 4
  br i1 %10, label %6, label %11

11:                                               ; preds = %6
  %12 = add nsw i64 %4, 1
  %13 = icmp ne i64 %12, 64
  br i1 %13, label %loop.preheader, label %14

14:                                               ; preds = %11
  ret void
}

define void @sum(ptr %0, ptr %1) {
  br label %3

3:                                                ; preds = %25, %2
  %4 = phi i64 [ 0, %2 ], [ %37, %25 ]
  %5 = getelementptr float, ptr %1, i64 %4
  %6 = load float, ptr %5
  %7 = mul nuw nsw i64 %4, 576
  %lsr.start = getelementptr float, ptr %0, i64 %7
  br label %8

8:                                                ; preds = %8, %3
  %lsr.ptr = phi ptr [ %lsr.start, %3 ], [ %lsr.ptr.next, %8 ]
  %9 = phi <4 x float> [ zeroinitializer, %3 ], [ %23, %8 ]
  %10 = phi <4 x float> [ zeroinitializer, %3 ], [ %20, %8 ]
  %11 = phi <4 x float> [ zeroinitializer, %3 ], [ %17, %8 ]
  %12 = phi <4 x float> [ zeroinitializer, %3 ], [ %14, %8 ]
  %lsr.count = phi i64 [ 576, %3 ], [ %lsr.count.next, %8 ]
  %13 = load <4 x float>, ptr %lsr.ptr, align 4
  %14 = fadd <4 x float> %13, %12
  %15 = getelementptr float, ptr %lsr.ptr, i64 4
  %16 = load <4 x float>, ptr %15, align 4
  %17 = fadd <4 x float> %16, %11
  %18 = getelementptr float, ptr %lsr.ptr, i64 8
  %19 = load <4 x float>, ptr %18, align 4
  %20 = fadd <4 x float> %19, %10
  %21 = getelementptr float, ptr %lsr.ptr, i64 12
  %22 = load <4 x float>, ptr %21, align 4
  %23 = fadd <4 x float> %22, %9
  %lsr.count.next = add i64 %lsr.count, -16
  %24 = icmp ne i64 %lsr.count.next, 0
  %lsr.ptr.next = getelementptr float, ptr %lsr.ptr, i64 16
  br i1 %24, label %8, label %25

25:                                               ; preds = %8
  %26 = fadd <4 x float> %14, %17
  %27 = fadd <4 x float> %20, %23
  %28 = fadd <4 x float> %26, %27
  %29 = extractelement <4 x float> %28, i64 0
  %30 = extractelement <4 x float> %28, i64 1
  %31 = extractelement <4 x float> %28, i64 2
  %32 = extractelement <4 x float> %28, i64 3
  %33 = fadd float %29, %30
  %34 = fadd float %31, %32
  %35 = fadd float %33, %34
  %36 = fadd float %35, %6
  store float %36, ptr %5
  %37 = add nuw nsw i64 %4, 1
  %38 = icmp ne i64 %37, 64
  br i1 %38, label %3, label %39

39:                                               ; preds = %25
  ret void
}

define i32 @main() {
entry:
  br label %fill

fill:                                             ; preds = %fill, %entry
  %i = phi i64 [ 0, %entry ], [ %i.next, %fill ]
  %k = urem i64 %i, 7
  %kf = uitofp i64 %k to float
  %pa = getelementptr float, ptr @a, i64 %i
  store float %kf, ptr %pa
  %pb = getelementptr float, ptr @b, i64 %i
  %kb = fadd float %kf, 1.000000e+00
  store float %kb, ptr %pb
  %i.next = add nuw i64 %i, 1
  %more = icmp ne i64 %i.next, 98304
  br i1 %more, label %fill, label %loop.preheader.i

loop.preheader.i:                                 ; preds = %7, %fill
  %0 = phi i64 [ %8, %7 ], [ 0, %fill ]
  %1 = mul i64 %0, 1536
  %lsr.start.i = getelementptr float, ptr @c, i64 %1
  %lsr.start1.i = getelementptr float, ptr @b, i64 %1
  %lsr.start2.i = getelementptr float, ptr @a, i64 %1
  br label %2

2:                                                ; preds = %2, %loop.preheader.i
  %lsr.ptr2.i = phi ptr [ %lsr.start2.i, %loop.preheader.i ], [ %lsr.ptr.next2.i, %2 ]
  %lsr.ptr1.i = phi ptr [ %lsr.start1.i, %loop.preheader.i ], [ %lsr.ptr.next1.i, %2 ]
  %lsr.ptr.i = phi ptr [ %lsr.start.i, %loop.preheader.i ], [ %lsr.ptr.next.i, %2 ]
  %lsr.count.i = phi i64 [ 1536, %loop.preheader.i ], [ %lsr.count.next.i, %2 ]
  %3 = load <4 x float>, ptr %lsr.ptr2.i, align 4
  %4 = load <4 x float>, ptr %lsr.ptr1.i, align 4
  %5 = fmul <4 x float> %3, %4
  store <4 x float> %5, ptr %lsr.ptr.i, align 4
  %lsr.count.next.i = add i64 %lsr.count.i, -4
  %6 = icmp ne i64 %lsr.count.next.i, 0
  %lsr.ptr.next.i = getelementptr float, ptr %lsr.ptr.i, i64 4
  %lsr.ptr.next1.i = getelementptr float, ptr %lsr.ptr1.i, i64 4
  %lsr.ptr.next2.i = getelementptr float, ptr %lsr.ptr2.i, i64 4
  br i1 %6, label %2, label %7

7:                                                ; preds = %2
  %8 = add nsw i64 %0, 1
  %9 = icmp ne i64 %8, 64
  br i1 %9, label %loop.preheader.i, label %10

10:                                               ; preds = %32, %7
  %11 = phi i64 [ %44, %32 ], [ 0, %7 ]
  %12 = getelementptr float, ptr @s, i64 %11
  %13 = load float, ptr %12
  %14 = mul nuw nsw i64 %11, 576
  %lsr.start.i1 = getelementptr float, ptr @c, i64 %14
  br label %15

15:                                               ; preds = %15, %10
  %lsr.ptr.i1 = phi ptr [ %lsr.start.i1, %10 ], [ %lsr.ptr.next.i1, %15 ]
  %16 = phi <4 x float> [ zeroinitializer, %10 ], [ %30, %15 ]
  %17 = phi <4 x float> [ zeroinitializer, %10 ], [ %27, %15 ]
  %18 = phi <4 x float> [ zeroinitializer, %10 ], [ %24, %15 ]
  %19 = phi <4 x float> [ zeroinitializer, %10 ], [ %21, %15 ]
  %lsr.count.i1 = phi i64 [ 576, %10 ], [ %lsr.count.next.i1, %15 ]
  %20 = load <4 x float>, ptr %lsr.ptr.i1, align 4
  %21 = fadd <4 x float> %20, %19
  %22 = getelementptr float, ptr %lsr.ptr.i1, i64 4
  %23 = load <4 x float>, ptr %22, align 4
  %24 = fadd <4 x float> %23, %18
  %25 = getelementptr float, ptr %lsr.ptr.i1, i64 8
  %26 = load <4 x float>, ptr %25, align 4
  %27 = fadd <4 x float> %26, %17
  %28 = getelementptr float, ptr %lsr.ptr.i1, i64 12
  %29 = load <4 x float>, ptr %28, align 4
  %30 = fadd <4 x float> %29, %16
  %lsr.count.next.i1 = add i64 %lsr.count.i1, -16
  %31 = icmp ne i64 %lsr.count.next.i1, 0
  %lsr.ptr.next.i1 = getelementptr float, ptr %lsr.ptr.i1, i64 16
  br i1 %31, label %15, label %32

32:                                               ; preds = %15
  %33 = fadd <4 x float> %21, %24
  %34 = fadd <4 x float> %27, %30
  %35 = fadd <4 x float> %33, %34
  %36 = extractelement <4 x float> %35, i64 0
  %37 = extractelement <4 x float> %35, i64 1
  %38 = extractelement <4 x float> %35, i64 2
  %39 = extractelement <4 x float> %35, i64 3
  %40 = fadd float %36, %37
  %41 = fadd float %38, %39
  %42 = fadd float %40, %41
  %43 = fadd float %42, %13
  store float %43, ptr %12
  %44 = add nuw nsw i64 %11, 1
  %45 = icmp ne i64 %44, 64
  br i1 %45, label %10, label %46

46:                                               ; preds = %32
  %p = getelementptr float, ptr @s, i64 37
  %v = load float, ptr %p
  %r = fptosi float %v to i32
  %q = getelementptr float, ptr @c, i64 70000
  %w = load float, ptr %q
  %rw = fptosi float %w to i32
  %t = add i32 %r, %rw
  %m = urem i32 %t, 251
  ret i32 %m
}
