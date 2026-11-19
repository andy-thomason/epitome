; A small struct parameter changed with `insertvalue` and returned, as
; clang's optimised build of `L2 f(L2 v) { v.b++; return v; }` has it: the
; chain of `insertvalue`s starts from the parameter's registers.
target datalayout = "e-m:o-i64:64-i128:128-n32:64-S128-Fn32"
target triple = "arm64-apple-macosx14.0.0"

define [2 x i64] @bump_second([2 x i64] %0) {
  %2 = extractvalue [2 x i64] %0, 1
  %3 = add i64 %2, 1
  %4 = insertvalue [2 x i64] %0, i64 %3, 1
  ret [2 x i64] %4
}

define [2 x i64] @swap([2 x i64] %0) {
  %2 = extractvalue [2 x i64] %0, 0
  %3 = extractvalue [2 x i64] %0, 1
  %4 = insertvalue [2 x i64] %0, i64 %3, 0
  %5 = insertvalue [2 x i64] %4, i64 %2, 1
  ret [2 x i64] %5
}
