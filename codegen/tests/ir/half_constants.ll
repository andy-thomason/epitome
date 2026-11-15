; Functions whose only half is a constant: returned, stored, and passed
; (Rust's `<f16 as Default>::default`).
target triple = "arm64-apple-macosx14.0.0"
define half @zero() {
  ret half 0xH0000
}
define half @one() {
  ret half 0xH3C00
}
define void @store_minus_two(ptr %p) {
  store half 0xHC000, ptr %p
  ret void
}
define half @pass(half %x) {
  ret half %x
}
define half @call_pass() {
  %r = call half @pass(half 0xH3800)
  ret half %r
}
