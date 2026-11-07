; Quad precision to and from the integers narrower than the runtime's
; functions (i8 and i16), which x86-64 does by way of i32.
target triple = "x86_64-unknown-linux-gnu"
define i8 @fptosi_i8(fp128 %a) {
  %r = fptosi fp128 %a to i8
  ret i8 %r
}
define i8 @fptoui_i8(fp128 %a) {
  %r = fptoui fp128 %a to i8
  ret i8 %r
}
define fp128 @sitofp_i8(i8 %a) {
  %r = sitofp i8 %a to fp128
  ret fp128 %r
}
define fp128 @uitofp_i8(i8 %a) {
  %r = uitofp i8 %a to fp128
  ret fp128 %r
}
define i16 @fptosi_i16(fp128 %a) {
  %r = fptosi fp128 %a to i16
  ret i16 %r
}
define i16 @fptoui_i16(fp128 %a) {
  %r = fptoui fp128 %a to i16
  ret i16 %r
}
define fp128 @sitofp_i16(i16 %a) {
  %r = sitofp i16 %a to fp128
  ret fp128 %r
}
define fp128 @uitofp_i16(i16 %a) {
  %r = uitofp i16 %a to fp128
  ret fp128 %r
}
