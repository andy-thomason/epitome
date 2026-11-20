; epitome-opt: -O2
; Callers that only run on the way to a panic inline as if cold.
;
; - @report is only called from %fail, which leads only to unreachable,
;   so no warm code reaches it: inlining into it uses the cold threshold,
;   and @helper stays a call there.
; - @main is warm, so @helper is inlined into it as usual.
; - @shown is reached only through @show_vtable, which @main names only in
;   %fail: it is cold too, and keeps its call to @helper.
@show_vtable = internal constant [1 x ptr] [ptr @shown]

define internal i32 @helper(i32 %x) {
entry:
  %t0 = add i32 %x, 3
  %t1 = mul i32 %t0, 10
  %t2 = xor i32 %t1, 17
  %t3 = add i32 %t2, 24
  %t4 = mul i32 %t3, 31
  %t5 = xor i32 %t4, 38
  %t6 = add i32 %t5, 45
  %t7 = mul i32 %t6, 52
  %t8 = xor i32 %t7, 59
  %t9 = add i32 %t8, 66
  ret i32 %t9
}

define internal i32 @report(i32 %x) noinline {
entry:
  %y = call i32 @helper(i32 %x)
  ret i32 %y
}

define internal i32 @shown(i32 %x) {
entry:
  %y = call i32 @helper(i32 %x)
  %z = add i32 %y, 1
  ret i32 %z
}

declare void @abort()
define void @use(ptr %p, i32 %x) {
entry:
  ret void
}

define i32 @main(i32 %argc) {
entry:
  %a = call i32 @helper(i32 %argc)
  %bad = icmp eq i32 %a, 12345
  br i1 %bad, label %fail, label %ok

fail:
  %r = call i32 @report(i32 %a)
  call void @use(ptr @show_vtable, i32 %r)
  call void @abort()
  unreachable

ok:
  %b = and i32 %a, 7
  ret i32 %b
}
