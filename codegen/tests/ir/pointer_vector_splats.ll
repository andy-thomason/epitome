; Vectors of pointers compared with splats: of null (as the loop
; vectoriser makes of Option<&T>'s), and of a global's address. The
; driver's arrays are 8-aligned, so the accesses say so (x86-64's
; aligned vector loads fault otherwise).
target triple = "arm64-apple-macosx14.0.0"
@g = global i32 0
define void @f(ptr %o, ptr %p) {
  %v = load <2 x ptr>, ptr %p, align 8
  %z = icmp eq <2 x ptr> %v, splat (ptr null)
  %e = icmp eq <2 x ptr> %v, splat (ptr @g)
  %zz = zext <2 x i1> %z to <2 x i8>
  %ee = zext <2 x i1> %e to <2 x i8>
  store <2 x i8> %zz, ptr %o, align 1
  %o2 = getelementptr i8, ptr %o, i64 2
  store <2 x i8> %ee, ptr %o2, align 1
  %s = select <2 x i1> %z, <2 x ptr> splat (ptr @g), <2 x ptr> %v
  %o4 = getelementptr i8, ptr %o, i64 8
  store <2 x ptr> %s, ptr %o4, align 8
  ret void
}
define ptr @gaddr() {
  ret ptr @g
}
