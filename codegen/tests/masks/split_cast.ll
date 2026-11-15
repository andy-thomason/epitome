target triple = "arm64-apple-macosx14.0.0"

; Masks of <16 x i32> comparisons, which are split across registers
; (four <4 x i1> parts with Advanced SIMD, two <8 x i1> with AVX2),
; extended to vectors of narrower or wider lanes, and combined with a mask
; made from an i16's bits.

define void @sext8(ptr %a, ptr %out) {
  %v = load <16 x i32>, ptr %a, align 4
  %c = icmp slt <16 x i32> %v, zeroinitializer
  %x = sext <16 x i1> %c to <16 x i8>
  store <16 x i8> %x, ptr %out, align 1
  ret void
}

define void @zext16(ptr %a, ptr %out) {
  %v = load <16 x i32>, ptr %a, align 4
  %m = and <16 x i32> %v, splat (i32 3)
  %c = icmp ne <16 x i32> %m, zeroinitializer
  %x = zext <16 x i1> %c to <16 x i16>
  store <16 x i16> %x, ptr %out, align 2
  ret void
}

define void @sext64(ptr %a, ptr %out) {
  %v = load <16 x i32>, ptr %a, align 4
  %c = icmp ugt <16 x i32> %v, splat (i32 1000)
  %x = sext <16 x i1> %c to <16 x i64>
  store <16 x i64> %x, ptr %out, align 8
  ret void
}

define i16 @bits(i16 %b, ptr %a) {
  %m = bitcast i16 %b to <16 x i1>
  %v = load <16 x i32>, ptr %a, align 4
  %o = and <16 x i32> %v, splat (i32 1)
  %c = icmp ne <16 x i32> %o, zeroinitializer
  %n = xor <16 x i1> %m, splat (i1 true)
  %x = or <16 x i1> %n, %c
  %y = and <16 x i1> %x, %m
  %r = bitcast <16 x i1> %y to i16
  ret i16 %r
}
