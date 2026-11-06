; Scalable values live across calls, which change every vector register:
; spilled to slots of a vector length each, with a stack argument (from
; the frame pointer) and fixed slots beside them.
target triple = "aarch64-unknown-linux-gnu"

declare void @clobber()
declare i64 @twice(i64)

define <vscale x 4 x i32> @keep2(<vscale x 4 x i32> %a, <vscale x 4 x i32> %b) #0 {
  call void @clobber()
  %c = add <vscale x 4 x i32> %a, %b
  call void @clobber()
  %d = add <vscale x 4 x i32> %c, %a
  ret <vscale x 4 x i32> %d
}

define <vscale x 2 x i64> @keep3(<vscale x 2 x i64> %a, <vscale x 2 x i64> %b, <vscale x 2 x i64> %c, i64 %x) #0 {
  %y = call i64 @twice(i64 %x)
  %s = add <vscale x 2 x i64> %a, %b
  %z = call i64 @twice(i64 %y)
  %t = sub <vscale x 2 x i64> %s, %c
  %u = add <vscale x 2 x i64> %t, %a
  ret <vscale x 2 x i64> %u
}

; Nine integer arguments: the ninth on the stack, found from x29.
define <vscale x 4 x i32> @stack_arg(<vscale x 4 x i32> %a, i64 %r0, i64 %r1, i64 %r2, i64 %r3, i64 %r4, i64 %r5, i64 %r6, i64 %r7, i64 %r8) #0 {
  call void @clobber()
  %v = add <vscale x 4 x i32> %a, %a
  %w = call i64 @twice(i64 %r8)
  call void @clobber()
  %x = add <vscale x 4 x i32> %v, %a
  ret <vscale x 4 x i32> %x
}

attributes #0 = { "target-features"="+sve" }
