; @lookups: not inlined @panic_bounds_check: noinline
; @lookups: combine: (known bits) x3, br_true x3
; @lookups: simplifycfg: 0 unreachable blocks removed, 0 empty blocks bypassed, 0 selects made, 3 blocks merged
; @lookups: implied: 1 comparisons folded, 0 selects resolved, 4 flags set
; @lookups: combine: (dead) x1, br_true x1
; @lookups: simplifycfg: 0 unreachable blocks removed, 0 empty blocks bypassed, 0 selects made, 1 blocks merged
; @odd: not inlined @panic_bounds_check: noinline
; @odd: combine: (dead) x1, (reassociated) x1
; @odd: implied: 1 comparisons folded, 0 selects resolved, 3 flags set
; @odd: combine: (dead) x1, br_true x1
; @odd: simplifycfg: 0 unreachable blocks removed, 0 empty blocks bypassed, 0 selects made, 1 blocks merged
; @odd: rotate: 1 loops rotated
; @odd: combine: (dead) x1, (folded) x2, (reassociated) x1, br_true x1
; @odd: simplifycfg: 0 unreachable blocks removed, 0 empty blocks bypassed, 0 selects made, 1 blocks merged
; @odd: indvars: loop %body runs 8 times when entered
; @odd: indvars: 1 exact exit tests, 0 exit values, 0 counters widened
; @main: not inlined @lookups: noinline
; @main: not inlined @odd: noinline

target datalayout = "e-m:o-i64:64-i128:128-n32:64-S128"

define void @panic_bounds_check(i64 %i, i64 %len) noinline noreturn {
  tail call void @llvm.trap()
  unreachable
}

declare void @llvm.trap()

define i64 @lookups(ptr %t256, ptr %t16, i8 %x, i64 %h, i64 range(i64 0, 10) %d) noinline {
start:
  %i = zext i8 %x to i64
  %p = getelementptr inbounds i8, ptr %t256, i64 %i
  %a = load i8, ptr %p
  %m = and i64 %h, 15
  %q = getelementptr inbounds i8, ptr %t16, i64 %m
  %b = load i8, ptr %q
  %top = lshr i64 %h, 60
  %r = getelementptr inbounds i8, ptr %t16, i64 %top
  %c = load i8, ptr %r
  %s = getelementptr inbounds i8, ptr %t16, i64 %d
  %e = load i8, ptr %s
  %w = and i64 %h, 31
  %in5 = icmp ult i64 %w, 16
  br i1 %in5, label %ok5, label %panic

ok5:                                              ; preds = %start
  %u = getelementptr inbounds i8, ptr %t16, i64 %w
  %f = load i8, ptr %u
  %a1 = zext i8 %a to i64
  %b1 = zext i8 %b to i64
  %c1 = zext i8 %c to i64
  %e1 = zext i8 %e to i64
  %f1 = zext i8 %f to i64
  %s1 = add nuw nsw i64 %a1, %b1
  %s2 = add nuw nsw i64 %s1, %c1
  %s3 = add nuw nsw i64 %s2, %e1
  %s4 = add nuw nsw i64 %s3, %f1
  ret i64 %s4

panic:                                            ; preds = %start
  tail call void @panic_bounds_check(i64 0, i64 0)
  unreachable
}

define i64 @odd(ptr %a) noinline {
start:
  br label %body

body:                                             ; preds = %ok2, %start
  %s = phi i64 [ 0, %start ], [ %s.next, %ok2 ]
  %i = phi i64 [ 0, %start ], [ %i.next, %ok2 ]
  %two = shl i64 %i, 1
  %k = add nuw nsw i64 %two, 1
  %p = getelementptr inbounds i64, ptr %a, i64 %k
  %x = load i64, ptr %p
  %k2 = add nuw nsw i64 %two, 2
  %in2 = icmp ult i64 %k2, 16
  br i1 %in2, label %ok2, label %panic

ok2:                                              ; preds = %body
  %q = getelementptr inbounds i64, ptr %a, i64 %k2
  %y = load i64, ptr %q
  %0 = add i64 %x, %y
  %s.next = add i64 %0, %s
  %i.next = add nuw nsw i64 %i, 1
  %more = icmp ne i64 %i.next, 8
  br i1 %more, label %body, label %done

panic:                                            ; preds = %body
  tail call void @panic_bounds_check(i64 %k, i64 16)
  unreachable

done:                                             ; preds = %ok2
  ret i64 %s.next
}

define i32 @main() {
  %t256 = alloca [256 x i8]
  %t16 = alloca [16 x i8]
  %a = alloca [17 x i64]
  call void @llvm.memset.p0.i64(ptr %t256, i8 3, i64 256, i1 false)
  call void @llvm.memset.p0.i64(ptr %t16, i8 5, i64 16, i1 false)
  call void @llvm.memset.p0.i64(ptr %a, i8 1, i64 136, i1 false)
  %l = call i64 @lookups(ptr %t256, ptr %t16, i8 -56, i64 -1152921504606846965, i64 7)
  %o = call i64 @odd(ptr %a)
  %t = add i64 %l, %o
  %r = trunc i64 %t to i32
  ret i32 %r
}

declare void @llvm.memset.p0.i64(ptr, i8, i64, i1)
