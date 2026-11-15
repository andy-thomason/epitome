; 256-bit vectors under AVX-512 features that do not name avx2
; (tests/avx512.rs): `+avx512f` implies AVX2, as clang's -mavx512f and
; LLVM's -mattr do, so each of these is ymm loads and stores, not "no
; instruction" (epitome-sv's netlists, compiled with
; NETLIST_MATTR=+avx512f,+avx512bw, met them).
target triple = "x86_64-unknown-linux-gnu"

define void @load_v16i16(ptr %a, ptr %o) {
  %x = load <16 x i16>, ptr %a, align 32
  %w = zext <16 x i16> %x to <16 x i32>
  %r = add <16 x i32> %w, splat (i32 1)
  %t = trunc <16 x i32> %r to <16 x i8>
  store <16 x i8> %t, ptr %o, align 16
  ret void
}

define void @store_v16i16(ptr %a, ptr %o) {
  %x = load <16 x i32>, ptr %a, align 64
  %t = trunc <16 x i32> %x to <16 x i16>
  store <16 x i16> %t, ptr %o, align 32
  ret void
}

define void @store_v8i32(ptr %a, ptr %o) {
  %x = load <8 x i32>, ptr %a, align 32
  %r = add <8 x i32> %x, %x
  store <8 x i32> %r, ptr %o, align 32
  ret void
}
