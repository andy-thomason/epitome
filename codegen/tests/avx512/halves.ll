target triple = "x86_64-unknown-linux-gnu"

; 512-bit vectors cut into halves and joined, as legalisation does when it
; widens 16 lanes of 32 bits to 64 (and multiplies those), and as a
; shuffle asks.

define void @widen_mul(ptr %a, ptr %b, ptr %out) "target-features"="+avx2,+avx512f,+avx512bw,+avx512dq,+avx512vl" {
  %x = load <16 x i32>, ptr %a, align 64
  %y = load <16 x i32>, ptr %b, align 64
  %xw = zext <16 x i32> %x to <16 x i64>
  %yw = sext <16 x i32> %y to <16 x i64>
  %p = mul <16 x i64> %xw, %yw
  %hi = lshr <16 x i64> %p, splat (i64 32)
  %s = add <16 x i64> %p, %hi
  %t = trunc <16 x i64> %s to <16 x i32>
  store <16 x i32> %t, ptr %out, align 64
  ret void
}

define void @swap_halves(ptr %a, ptr %out) "target-features"="+avx2,+avx512f,+avx512bw,+avx512dq,+avx512vl" {
  %x = load <16 x i32>, ptr %a, align 64
  %lo = shufflevector <16 x i32> %x, <16 x i32> poison, <8 x i32> <i32 0, i32 1, i32 2, i32 3, i32 4, i32 5, i32 6, i32 7>
  %hi = shufflevector <16 x i32> %x, <16 x i32> poison, <8 x i32> <i32 8, i32 9, i32 10, i32 11, i32 12, i32 13, i32 14, i32 15>
  %l2 = add <8 x i32> %lo, splat (i32 1)
  %r = shufflevector <8 x i32> %hi, <8 x i32> %l2, <16 x i32> <i32 0, i32 1, i32 2, i32 3, i32 4, i32 5, i32 6, i32 7, i32 8, i32 9, i32 10, i32 11, i32 12, i32 13, i32 14, i32 15>
  store <16 x i32> %r, ptr %out, align 64
  ret void
}
