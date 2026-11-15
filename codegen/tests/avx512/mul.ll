target triple = "x86_64-unknown-linux-gnu"

; Products in 64-bit lanes of a ymm and an xmm (AVX512DQ+VL's vpmullq), as a
; netlist of 4 or 2 instances with 64-bit lanes multiplies, and 32-bit
; lanes of an xmm (SSE4.1's pmulld).

define void @mul4x64(ptr %a, ptr %b, ptr %r) "target-features"="+avx2,+avx512f,+avx512bw,+avx512dq,+avx512vl" {
  %x = load <4 x i64>, ptr %a, align 32
  %y = load <4 x i64>, ptr %b, align 32
  %p = mul <4 x i64> %x, %y
  store <4 x i64> %p, ptr %r, align 32
  ret void
}

define void @mul2x64(ptr %a, ptr %b, ptr %r) "target-features"="+avx2,+avx512f,+avx512bw,+avx512dq,+avx512vl" {
  %x = load <2 x i64>, ptr %a, align 16
  %y = load <2 x i64>, ptr %b, align 16
  %p = mul <2 x i64> %x, %y
  store <2 x i64> %p, ptr %r, align 16
  ret void
}

define void @mul4x32(ptr %a, ptr %b, ptr %r) "target-features"="+sse4.1" {
  %x = load <4 x i32>, ptr %a, align 16
  %y = load <4 x i32>, ptr %b, align 16
  %p = mul <4 x i32> %x, %y
  store <4 x i32> %p, ptr %r, align 16
  ret void
}
