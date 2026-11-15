; Quad precision's IEEE 754-2019 minimum and maximum, and minimumNumber
; and maximumNumber, which x86-64 does by the C library's fminimumf128 and
; the rest. llc cannot build this (it fails to select fp128's minimum and
; maximum on x86-64), so the driver checks each against clang's build of
; the same operation on doubles instead.
; clang cannot build this
target triple = "x86_64-unknown-linux-gnu"
define fp128 @minimum128(fp128 %a, fp128 %b) {
  %r = call fp128 @llvm.minimum.f128(fp128 %a, fp128 %b)
  ret fp128 %r
}
define fp128 @maximum128(fp128 %a, fp128 %b) {
  %r = call fp128 @llvm.maximum.f128(fp128 %a, fp128 %b)
  ret fp128 %r
}
define fp128 @minimumnum128(fp128 %a, fp128 %b) {
  %r = call fp128 @llvm.minimumnum.f128(fp128 %a, fp128 %b)
  ret fp128 %r
}
define fp128 @maximumnum128(fp128 %a, fp128 %b) {
  %r = call fp128 @llvm.maximumnum.f128(fp128 %a, fp128 %b)
  ret fp128 %r
}
declare fp128 @llvm.minimum.f128(fp128, fp128)
declare fp128 @llvm.maximum.f128(fp128, fp128)
declare fp128 @llvm.minimumnum.f128(fp128, fp128)
declare fp128 @llvm.maximumnum.f128(fp128, fp128)
