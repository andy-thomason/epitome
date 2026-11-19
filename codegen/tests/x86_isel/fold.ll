; Loads folded into SSE2's operands where they are 16-aligned, as legacy
; SSE's memory operands must be; a load less aligned is a movups first.
target triple = "x86_64-unknown-linux-gnu"

define void @fold_aligned(ptr %p, ptr %q) {
  %a = load <8 x i16>, ptr %q, align 1
  %b16 = getelementptr i8, ptr %p, i64 16
  %b = load <8 x i16>, ptr %b16, align 16
  %s = add <8 x i16> %a, %b
  %c32 = getelementptr i8, ptr %p, i64 32
  %c = load <8 x i16>, ptr %c32, align 16
  %m = mul <8 x i16> %s, %c
  %f48 = getelementptr i8, ptr %p, i64 48
  %f = load <8 x i16>, ptr %f48, align 16
  %g = xor <8 x i16> %m, %f
  store <8 x i16> %g, ptr %q, align 1
  ret void
}

define void @fold_unaligned(ptr %p, ptr %q) {
  %a = load <8 x i16>, ptr %q, align 1
  %b16 = getelementptr i8, ptr %p, i64 16
  %b = load <8 x i16>, ptr %b16, align 8
  %s = add <8 x i16> %a, %b
  store <8 x i16> %s, ptr %q, align 1
  ret void
}
