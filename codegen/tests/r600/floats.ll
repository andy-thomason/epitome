; Floating point: negation and absolute values, three-operand and
; transcendental operations, comparisons and selects.
declare float @llvm.fabs.f32(float)
declare float @llvm.fma.f32(float, float, float)
declare float @llvm.sqrt.f32(float)
declare float @llvm.exp2.f32(float)
declare float @llvm.log2.f32(float)
declare float @llvm.sin.f32(float)
declare float @llvm.floor.f32(float)
declare float @llvm.maxnum.f32(float, float)
define amdgpu_kernel void @fl(ptr addrspace(1) %o, float %a, float %b, float %c) {
  %na = fneg float %a
  %ab = call float @llvm.fabs.f32(float %b)
  %m = fmul float %na, %ab
  %f = call float @llvm.fma.f32(float %m, float %c, float %a)
  %s = call float @llvm.sqrt.f32(float %f)
  %e = call float @llvm.exp2.f32(float %s)
  %l = call float @llvm.log2.f32(float %e)
  %si = call float @llvm.sin.f32(float %l)
  %fl = call float @llvm.floor.f32(float %si)
  %mx = call float @llvm.maxnum.f32(float %fl, float 3.5)
  %d = fdiv float %mx, %c
  %cmp = fcmp olt float %d, %a
  %r = select i1 %cmp, float %d, float 1.0
  store float %r, ptr addrspace(1) %o
  ret void
}
