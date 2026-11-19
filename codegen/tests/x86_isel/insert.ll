; A scalar into a vector whose other lanes are zero (or poison): one movq or
; movd, which clear the rest, not a stack slot.
target triple = "x86_64-unknown-linux-gnu"

define void @insert_q(i64 %x, ptr %q) {
  %v = insertelement <2 x i64> <i64 poison, i64 0>, i64 %x, i64 0
  store <2 x i64> %v, ptr %q, align 1
  ret void
}

define void @insert_d(i32 %x, ptr %q) {
  %v = insertelement <4 x i32> <i32 poison, i32 0, i32 poison, i32 0>, i32 %x, i64 0
  store <4 x i32> %v, ptr %q, align 1
  ret void
}
