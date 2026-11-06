; Bit operations, minimum and maximum, conversions both ways, and
; selects on integers.
declare i32 @llvm.ctpop.i32(i32)
declare i32 @llvm.ctlz.i32(i32, i1)
declare i32 @llvm.cttz.i32(i32, i1)
declare i32 @llvm.bitreverse.i32(i32)
declare i32 @llvm.smax.i32(i32, i32)
declare i32 @llvm.umin.i32(i32, i32)
declare i32 @llvm.fshr.i32(i32, i32, i32)
declare float @llvm.cos.f32(float)
declare float @llvm.trunc.f32(float)
declare float @llvm.ceil.f32(float)
declare float @llvm.rint.f32(float)
declare float @llvm.minnum.f32(float, float)
define amdgpu_kernel void @bits(ptr addrspace(1) %o, i32 %a, i32 %b, i32 %c) {
  %p = call i32 @llvm.ctpop.i32(i32 %a)
  %l = call i32 @llvm.ctlz.i32(i32 %b, i1 false)
  %t = call i32 @llvm.cttz.i32(i32 %c, i1 false)
  %r = call i32 @llvm.bitreverse.i32(i32 %a)
  %mx = call i32 @llvm.smax.i32(i32 %p, i32 %l)
  %mn = call i32 @llvm.umin.i32(i32 %t, i32 %r)
  %f = call i32 @llvm.fshr.i32(i32 %mx, i32 %mn, i32 %c)
  %x = and i32 %f, 255
  %y = or i32 %x, %b
  %z = shl i32 %y, %c
  %e = lshr i32 %z, 8
  %e8 = and i32 %e, 4095
  %neg = icmp slt i32 %a, 0
  %s = select i1 %neg, i32 %e8, i32 %b
  %m24a = and i32 %a, 16777215
  %m24b = and i32 %b, 16777215
  %m24 = mul i32 %m24a, %m24b
  %sum = add i32 %s, %m24
  store i32 %sum, ptr addrspace(1) %o
  ret void
}
define amdgpu_kernel void @conv(ptr addrspace(1) %o, float %a, i32 %b, i32 %c) {
  %i = fptosi float %a to i32
  %u = fptoui float %a to i32
  %fb = uitofp i32 %b to float
  %fc = sitofp i32 %c to float
  %co = call float @llvm.cos.f32(float %fb)
  %tr = call float @llvm.trunc.f32(float %fc)
  %ce = call float @llvm.ceil.f32(float %co)
  %ri = call float @llvm.rint.f32(float %tr)
  %mn = call float @llvm.minnum.f32(float %ce, float %ri)
  %iu = add i32 %i, %u
  %fi = bitcast i32 %iu to float
  %r = fadd float %mn, %fi
  %ge = fcmp oge float %r, 0.0
  %v = select i1 %ge, float %r, float -2.0
  store float %v, ptr addrspace(1) %o
  ret void
}
