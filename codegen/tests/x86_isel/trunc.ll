; Narrowing (trunc) in registers on SSE2: shuffles of the low halves
; (pshuflw, pshufhw, pshufd), or each lane sign- or zero-extended in place
; and packed (packssdw, packuswb), or shufps; never lanes through memory.
target triple = "x86_64-unknown-linux-gnu"

define void @trunc_8x32_16(ptr %p, ptr %q) {
  %a = load <8 x i32>, ptr %p, align 4
  %r = trunc <8 x i32> %a to <8 x i16>
  store <8 x i16> %r, ptr %q, align 1
  ret void
}

define void @trunc_4x32_16(ptr %p, ptr %q) {
  %a = load <4 x i32>, ptr %p, align 4
  %r = trunc <4 x i32> %a to <4 x i16>
  store <4 x i16> %r, ptr %q, align 1
  ret void
}

define void @trunc_16x16_8(ptr %p, ptr %q) {
  %a = load <16 x i16>, ptr %p, align 4
  %r = trunc <16 x i16> %a to <16 x i8>
  store <16 x i8> %r, ptr %q, align 1
  ret void
}

define void @trunc_8x16_8(ptr %p, ptr %q) {
  %a = load <8 x i16>, ptr %p, align 4
  %r = trunc <8 x i16> %a to <8 x i8>
  store <8 x i8> %r, ptr %q, align 1
  ret void
}

define void @trunc_4x64_32(ptr %p, ptr %q) {
  %a = load <4 x i64>, ptr %p, align 4
  %r = trunc <4 x i64> %a to <4 x i32>
  store <4 x i32> %r, ptr %q, align 1
  ret void
}

define void @trunc_2x64_32(ptr %p, ptr %q) {
  %a = load <2 x i64>, ptr %p, align 4
  %r = trunc <2 x i64> %a to <2 x i32>
  store <2 x i32> %r, ptr %q, align 1
  ret void
}

define void @trunc_16x32_8(ptr %p, ptr %q) {
  %a = load <16 x i32>, ptr %p, align 4
  %r = trunc <16 x i32> %a to <16 x i8>
  store <16 x i8> %r, ptr %q, align 1
  ret void
}
; With AVX2, a ymm register's lanes: vextracti128, then packed.
define void @trunc_8x32_16_avx2(ptr %p, ptr %q) #0 {
  %a = load <8 x i32>, ptr %p, align 4
  %r = trunc <8 x i32> %a to <8 x i16>
  store <8 x i16> %r, ptr %q, align 1
  ret void
}

define void @trunc_16x16_8_avx2(ptr %p, ptr %q) #0 {
  %a = load <16 x i16>, ptr %p, align 4
  %r = trunc <16 x i16> %a to <16 x i8>
  store <16 x i8> %r, ptr %q, align 1
  ret void
}

define void @trunc_4x64_32_avx2(ptr %p, ptr %q) #0 {
  %a = load <4 x i64>, ptr %p, align 4
  %r = trunc <4 x i64> %a to <4 x i32>
  store <4 x i32> %r, ptr %q, align 1
  ret void
}

attributes #0 = { nounwind "target-features"="+avx2,+avx" }
