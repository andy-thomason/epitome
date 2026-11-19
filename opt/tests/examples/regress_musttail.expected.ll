; @f: recursion: not removed: musttail call to another function
; @f: combine: sub_const x1
; @f: implied: 0 comparisons folded, 0 selects resolved, 1 flags set


@fmt = private constant [4 x i8] c"%d\0A\00"

declare i32 @printf(ptr, ...)

define internal i32 @g(i32 %n) noinline {
  %r = add i32 %n, 1000
  ret i32 %r
}

define internal i32 @f(i32 %n) noinline {
entry:
  %c = icmp sle i32 %n, 0
  br i1 %c, label %base, label %rec

base:                                             ; preds = %entry
  %t = musttail call i32 @g(i32 %n)
  ret i32 %t

rec:                                              ; preds = %entry
  %a = add nsw i32 %n, -1
  %r = tail call i32 @f(i32 %a)
  %s = add i32 %r, %n
  ret i32 %s
}

define i32 @main() {
  %v = tail call i32 @f(i32 10)
  %1 = tail call i32 (ptr, ...) @printf(ptr @fmt, i32 %v)
  ret i32 0
}
