; Remainders and quotients by the most negative integer, a power of two
; whose magnitude does not fit the type.
target triple = "arm64-apple-macosx14.0.0"
define i64 @rem64(i64 %x) {
  %r = srem i64 %x, -9223372036854775808
  ret i64 %r
}
define i64 @div64(i64 %x) {
  %r = sdiv i64 %x, -9223372036854775808
  ret i64 %r
}
define i32 @rem32(i32 %x) {
  %r = srem i32 %x, -2147483648
  ret i32 %r
}
define i32 @div32(i32 %x) {
  %r = sdiv i32 %x, -2147483648
  ret i32 %r
}
