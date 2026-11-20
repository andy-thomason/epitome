; epitome-opt: --passes=combine
; The examples from the request, and their neighbours.
define i32 @add_zero(i32 %x) {
  %r = add i32 %x, 0
  ret i32 %r
}

define i32 @add_self(i32 %x) {
  %r = add i32 %x, %x
  ret i32 %r
}

define i32 @sub_then_add(i32 %x) {
  %t = sub i32 %x, 1
  %r = add i32 %t, 1
  ret i32 %r
}

define i32 @chain(i32 %x, i32 %y) {
  %a = add i32 3, %x
  %b = add i32 %a, 4
  %c = sub i32 %b, 7
  %d = mul i32 %c, 8
  %e = xor i32 %d, %y
  %f = xor i32 %e, %y
  %g = udiv i32 %f, 16
  ret i32 %g
}

define i1 @compare(i32 %x) {
  %a = add i32 %x, 5
  %c = icmp eq i32 %a, 12
  %s = select i1 %c, i1 true, i1 false
  ret i1 %s
}

define <4 x i32> @vector(<4 x i32> %v) {
  %a = add <4 x i32> %v, zeroinitializer
  %b = mul <4 x i32> %a, splat (i32 4)
  ret <4 x i32> %b
}

define i8 @casts(i8 %x) {
  %w = zext i8 %x to i32
  %n = trunc i32 %w to i8
  ret i8 %n
}

define double @floats(double %x) {
  %a = fadd double %x, -0.0
  %b = fmul double %a, 1.0
  %c = fneg double %b
  %d = fneg double %c
  ret double %d
}

define i32 @folding() {
  %a = add i32 40, 2
  %b = shl i32 %a, 1
  %c = icmp ult i32 %b, 100
  %d = select i1 %c, i32 %b, i32 0
  ret i32 %d
}

; A pointer through an integer and back, as SROA leaves it where one slot
; is accessed as both; and zero as a pointer.
define ptr @int_ptr_round_trip(ptr %p) {
  %i = ptrtoint ptr %p to i64
  %q = inttoptr i64 %i to ptr
  ret ptr %q
}

define i64 @ptr_int_round_trip(i64 %i) {
  %p = inttoptr i64 %i to ptr
  %j = ptrtoint ptr %p to i64
  ret i64 %j
}

define i1 @null_from_zero(ptr %p) {
  %z = inttoptr i64 0 to ptr
  %c = icmp eq ptr %p, %z
  ret i1 %c
}
