target triple = "x86_64-unknown-linux-gnu"

; 8 and 4 lanes of 32 bits narrowed to bytes and halves, as a netlist of
; few lanes stores its narrow values (AVX-512VL's vpmovdb and vpmovdw).

define void @narrow8(ptr %a, ptr %b, ptr %h) "target-features"="+avx2,+avx512f,+avx512bw,+avx512dq,+avx512vl" {
  %x = load <8 x i32>, ptr %a, align 32
  %y = add <8 x i32> %x, splat (i32 3)
  %t = trunc <8 x i32> %y to <8 x i8>
  store <8 x i8> %t, ptr %b, align 8
  %u = trunc <8 x i32> %y to <8 x i16>
  store <8 x i16> %u, ptr %h, align 16
  ret void
}

define void @narrow4(ptr %a, ptr %h) "target-features"="+avx2,+avx512f,+avx512bw,+avx512dq,+avx512vl" {
  %x = load <4 x i32>, ptr %a, align 16
  %y = mul <4 x i32> %x, splat (i32 5)
  %u = trunc <4 x i32> %y to <4 x i16>
  store <4 x i16> %u, ptr %h, align 8
  ret void
}
