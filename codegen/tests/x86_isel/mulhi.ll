; The high halves of 16-bit products: pmulhuw and pmulhw (vpmulhuw and
; vpmulhw with AVX2), then a shift for what the product is shifted by
; beyond 16 bits; their 32-bit lanes are never made.
target triple = "x86_64-unknown-linux-gnu"

define void @mulhu_8(ptr %a, ptr %b, ptr %out) {
  %x = load <8 x i16>, ptr %a, align 2
  %y = load <8 x i16>, ptr %b, align 2
  %xw = zext <8 x i16> %x to <8 x i32>
  %yw = zext <8 x i16> %y to <8 x i32>
  %p = mul nuw <8 x i32> %xw, %yw
  %h = lshr <8 x i32> %p, splat (i32 16)
  %r = trunc <8 x i32> %h to <8 x i16>
  store <8 x i16> %r, ptr %out, align 2
  ret void
}

define void @mulhs_8(ptr %a, ptr %b, ptr %out) {
  %x = load <8 x i16>, ptr %a, align 2
  %y = load <8 x i16>, ptr %b, align 2
  %xw = sext <8 x i16> %x to <8 x i32>
  %yw = sext <8 x i16> %y to <8 x i32>
  %p = mul nsw <8 x i32> %xw, %yw
  %h = ashr <8 x i32> %p, splat (i32 16)
  %r = trunc <8 x i32> %h to <8 x i16>
  store <8 x i16> %r, ptr %out, align 2
  ret void
}

; Shifted by 19: pmulhuw, then psrlw $3 (zmij's to_bcd8).
define void @mulhu_19(ptr %a, ptr %b, ptr %out) {
  %x = load <8 x i16>, ptr %a, align 2
  %y = load <8 x i16>, ptr %b, align 2
  %xw = zext <8 x i16> %x to <8 x i32>
  %yw = zext <8 x i16> %y to <8 x i32>
  %p = mul nuw <8 x i32> %yw, %xw
  %h = lshr <8 x i32> %p, splat (i32 19)
  %r = trunc nuw nsw <8 x i32> %h to <8 x i16>
  store <8 x i16> %r, ptr %out, align 2
  ret void
}

; Signed, shifted arithmetically by 18: pmulhw, then psraw $2.
define void @mulhs_18(ptr %a, ptr %b, ptr %out) {
  %x = load <8 x i16>, ptr %a, align 2
  %y = load <8 x i16>, ptr %b, align 2
  %xw = sext <8 x i16> %x to <8 x i32>
  %yw = sext <8 x i16> %y to <8 x i32>
  %p = mul nsw <8 x i32> %xw, %yw
  %h = ashr <8 x i32> %p, splat (i32 18)
  %r = trunc <8 x i32> %h to <8 x i16>
  store <8 x i16> %r, ptr %out, align 2
  ret void
}

; By a constant that fits 16 bits.
define void @mulhu_const(ptr %a, ptr %out) {
  %x = load <8 x i16>, ptr %a, align 2
  %xw = zext <8 x i16> %x to <8 x i32>
  %p = mul nuw <8 x i32> %xw, splat (i32 52429)
  %h = lshr <8 x i32> %p, splat (i32 18)
  %r = trunc <8 x i32> %h to <8 x i16>
  store <8 x i16> %r, ptr %out, align 2
  ret void
}

; Two registers' worth: one pmulhuw a half.
define void @mulhu_16(ptr %a, ptr %b, ptr %out) {
  %x = load <16 x i16>, ptr %a, align 2
  %y = load <16 x i16>, ptr %b, align 2
  %xw = zext <16 x i16> %x to <16 x i32>
  %yw = zext <16 x i16> %y to <16 x i32>
  %p = mul nuw <16 x i32> %xw, %yw
  %h = lshr <16 x i32> %p, splat (i32 16)
  %r = trunc <16 x i32> %h to <16 x i16>
  store <16 x i16> %r, ptr %out, align 2
  ret void
}

; Half a register's worth.
define void @mulhs_4(ptr %a, ptr %b, ptr %out) {
  %x = load <4 x i16>, ptr %a, align 2
  %y = load <4 x i16>, ptr %b, align 2
  %xw = sext <4 x i16> %x to <4 x i32>
  %yw = sext <4 x i16> %y to <4 x i32>
  %p = mul nsw <4 x i32> %xw, %yw
  %h = lshr <4 x i32> %p, splat (i32 16)
  %r = trunc <4 x i32> %h to <4 x i16>
  store <4 x i16> %r, ptr %out, align 2
  ret void
}

; With AVX2, a ymm register's worth.
define void @mulhu_16_avx2(ptr %a, ptr %b, ptr %out) #0 {
  %x = load <16 x i16>, ptr %a, align 2
  %y = load <16 x i16>, ptr %b, align 2
  %xw = zext <16 x i16> %x to <16 x i32>
  %yw = zext <16 x i16> %y to <16 x i32>
  %p = mul nuw <16 x i32> %xw, %yw
  %h = lshr <16 x i32> %p, splat (i32 17)
  %r = trunc <16 x i32> %h to <16 x i16>
  store <16 x i16> %r, ptr %out, align 2
  ret void
}

attributes #0 = { nounwind "target-features"="+avx2,+avx" }
