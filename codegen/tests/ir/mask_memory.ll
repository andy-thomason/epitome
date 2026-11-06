; Vectors of booleans in memory are packed bits, lane k bit k, in as many
; bytes as they fill (hashbrown stores a group's comparison as a
; `<16 x i1>`): stores of comparisons, and loads of masks that select.
target triple = "arm64-apple-macosx14.0.0"
define void @st4(ptr %a, ptr %out) {
  %v = load <4 x i32>, ptr %a, align 4
  %m = icmp slt <4 x i32> %v, zeroinitializer
  store <4 x i1> %m, ptr %out, align 1
  ret void
}
define void @st16(ptr %a, ptr %b, ptr %out) {
  %x = load <16 x i8>, ptr %a, align 1
  %y = load <16 x i8>, ptr %b, align 1
  %m = icmp eq <16 x i8> %x, %y
  store <16 x i1> %m, ptr %out, align 2
  ret void
}
define void @st32(ptr %a, ptr %out) {
  %x = load <32 x i8>, ptr %a, align 1
  %m = icmp ugt <32 x i8> %x, splat (i8 96)
  store <32 x i1> %m, ptr %out, align 4
  ret void
}
define void @st64(ptr %a, ptr %out) {
  %x = load <64 x i8>, ptr %a, align 1
  %m = icmp ne <64 x i8> %x, splat (i8 32)
  store <64 x i1> %m, ptr %out, align 8
  ret void
}
define void @ld4(ptr %m, ptr %x, ptr %out) {
  %k = load <4 x i1>, ptr %m, align 1
  %v = load <4 x i32>, ptr %x, align 4
  %e = sext <4 x i1> %k to <4 x i32>
  %s = xor <4 x i32> %e, %v
  store <4 x i32> %s, ptr %out, align 4
  ret void
}
define void @ld16(ptr %m, ptr %x, ptr %out) {
  %k = load <16 x i1>, ptr %m, align 2
  %v = load <16 x i8>, ptr %x, align 1
  %s = select <16 x i1> %k, <16 x i8> %v, <16 x i8> splat (i8 46)
  store <16 x i8> %s, ptr %out, align 1
  ret void
}
define void @ld64(ptr %m, ptr %x, ptr %out) {
  %k = load <64 x i1>, ptr %m, align 8
  %v = load <64 x i8>, ptr %x, align 1
  %s = select <64 x i1> %k, <64 x i8> %v, <64 x i8> splat (i8 46)
  store <64 x i8> %s, ptr %out, align 1
  ret void
}
