; epitome-opt: -O2 --print-cycles
; Branches on constants fold before the call graph is built, so the call
; behind `if cfg!(debug_assertions)` (lowered as `icmp ne i8 0, 0`) makes
; no cycle: @len and @report would be one SCC, and @report could not
; inline @len. No cycle is printed, and @len is inlined.
@out = global i32 0
define internal i32 @len(i32 %n) {
entry:
  %debug = icmp ne i8 0, 0
  br i1 %debug, label %assert, label %ok
assert:
  call void @report(i32 %n)
  br label %ok
ok:
  %r = and i32 %n, 255
  ret i32 %r
}
define internal void @report(i32 %n) {
entry:
  %l = call i32 @len(i32 %n)
  switch i8 1, label %other [
    i8 0, label %zero
    i8 1, label %one
  ]
zero:
  call void @report(i32 %l)
  br label %one
other:
  call void @report(i32 0)
  br label %one
one:
  store volatile i32 %l, ptr @out
  ret void
}
define i32 @main() {
  call void @report(i32 300)
  %v = load volatile i32, ptr @out
  ret i32 %v
}
