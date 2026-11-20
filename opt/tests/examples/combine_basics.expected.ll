; @add_zero: combine: add_zero x1
; @add_self: combine: add_self x1
; @sub_then_add: combine: (dead) x1, add_add_const x1, add_zero x1, sub_const x1
; @chain: combine: (constant to the right) x1, (dead) x3, add_add_const x1, mul_pow2 x1, sub_add_cancel x1, udiv_pow2 x1, xor_xor_cancel x1
; @compare: combine: (dead) x1, icmp_eq_add x1, select_bool x1
; @vector: combine: add_zero x1, mul_pow2 x1
; @casts: combine: (dead) x1, trunc_zext_same x1
; @floats: combine: (dead) x1, fadd_neg_zero x1, fmul_one x1, fneg_fneg x1
; @folding: combine: (folded) x4
; @int_ptr_round_trip: combine: (dead) x1, inttoptr_ptrtoint x1
; @ptr_int_round_trip: combine: (dead) x1, ptrtoint_inttoptr x1
; @null_from_zero: combine: (folded) x1


define i32 @add_zero(i32 %x) {
  ret i32 %x
}

define i32 @add_self(i32 %x) {
  %r = shl i32 %x, 1
  ret i32 %r
}

define i32 @sub_then_add(i32 %x) {
  ret i32 %x
}

define i32 @chain(i32 %x, i32 %y) {
  %d = shl i32 %x, 3
  %g = lshr i32 %d, 4
  ret i32 %g
}

define i1 @compare(i32 %x) {
  %c = icmp eq i32 %x, 7
  ret i1 %c
}

define <4 x i32> @vector(<4 x i32> %v) {
  %b = shl <4 x i32> %v, splat (i32 2)
  ret <4 x i32> %b
}

define i8 @casts(i8 %x) {
  ret i8 %x
}

define double @floats(double %x) {
  ret double %x
}

define i32 @folding() {
  ret i32 84
}

define ptr @int_ptr_round_trip(ptr %p) {
  ret ptr %p
}

define i64 @ptr_int_round_trip(i64 %i) {
  ret i64 %i
}

define i1 @null_from_zero(ptr %p) {
  %c = icmp eq ptr %p, null
  ret i1 %c
}
