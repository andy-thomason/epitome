; Masks of 32 to 128 lanes: selects, reductions, bitcasts to and from
; their integers, loads and stores of them.
target triple = "arm64-apple-macosx14.0.0"
define void @sel(ptr %o, ptr %a, ptr %b) {
  %x = load <128 x i8>, ptr %a
  %y = load <128 x i8>, ptr %b
  %c = icmp ult <128 x i8> %x, %y
  %r = select <128 x i1> %c, <128 x i8> %x, <128 x i8> %y
  store <128 x i8> %r, ptr %o
  ret void
}
define i128 @bits(ptr %a, ptr %b) {
  %x = load <128 x i8>, ptr %a
  %y = load <128 x i8>, ptr %b
  %c = icmp eq <128 x i8> %x, %y
  %r = bitcast <128 x i1> %c to i128
  ret i128 %r
}
define i1 @any(ptr %a) {
  %x = load <128 x i8>, ptr %a
  %c = icmp eq <128 x i8> %x, zeroinitializer
  %r = call i1 @llvm.vector.reduce.or.v128i1(<128 x i1> %c)
  ret i1 %r
}
define void @frombits(ptr %o, ptr %a, i128 %m) {
  %c = bitcast i128 %m to <128 x i1>
  %x = load <128 x i8>, ptr %a
  %r = select <128 x i1> %c, <128 x i8> %x, <128 x i8> zeroinitializer
  store <128 x i8> %r, ptr %o
  ret void
}
define void @st(ptr %o, ptr %a) {
  %x = load <128 x i8>, ptr %a
  %c = icmp sgt <128 x i8> %x, zeroinitializer
  store <128 x i1> %c, ptr %o
  ret void
}
define i64 @cnt(ptr %a) {
  %x = load <128 x i8>, ptr %a
  %c = icmp sgt <128 x i8> %x, zeroinitializer
  %z = zext <128 x i1> %c to <128 x i8>
  %s = call i8 @llvm.vector.reduce.add.v128i8(<128 x i8> %z)
  %w = zext i8 %s to i64
  ret i64 %w
}
declare i1 @llvm.vector.reduce.or.v128i1(<128 x i1>)
declare i8 @llvm.vector.reduce.add.v128i8(<128 x i8>)
define i64 @b64(ptr %a, ptr %b) {
  %x = load <64 x i8>, ptr %a
  %y = load <64 x i8>, ptr %b
  %c = icmp eq <64 x i8> %x, %y
  %r = bitcast <64 x i1> %c to i64
  ret i64 %r
}
define i32 @b32(ptr %a, ptr %b) {
  %x = load <32 x i8>, ptr %a
  %y = load <32 x i8>, ptr %b
  %c = icmp eq <32 x i8> %x, %y
  %r = bitcast <32 x i1> %c to i32
  ret i32 %r
}
define void @st64(ptr %o, ptr %a) {
  %x = load <64 x i8>, ptr %a
  %c = icmp sgt <64 x i8> %x, zeroinitializer
  store <64 x i1> %c, ptr %o
  ret void
}
define void @f64(ptr %o, ptr %a, i64 %m) {
  %c = bitcast i64 %m to <64 x i1>
  %x = load <64 x i8>, ptr %a
  %r = select <64 x i1> %c, <64 x i8> %x, <64 x i8> zeroinitializer
  store <64 x i8> %r, ptr %o
  ret void
}
; Selects of vectors split in parts by masks that are not: made from bits,
; and loaded.
define void @selbits(ptr %o, ptr %a, ptr %b, i16 %m) {
  %c = bitcast i16 %m to <16 x i1>
  %x = load <16 x i32>, ptr %a
  %y = load <16 x i32>, ptr %b
  %r = select <16 x i1> %c, <16 x i32> %x, <16 x i32> %y
  store <16 x i32> %r, ptr %o
  ret void
}
define void @selload(ptr %o, ptr %a, ptr %b, ptr %mp) {
  %c = load <16 x i1>, ptr %mp
  %x = load <16 x i32>, ptr %a
  %y = load <16 x i32>, ptr %b
  %r = select <16 x i1> %c, <16 x i32> %x, <16 x i32> %y
  store <16 x i32> %r, ptr %o
  ret void
}
