; Generic IR of scalable vectors, as the loop vectoriser makes it:
; operations with no unpredicated instruction, compares into predicates,
; selects, splats, reductions, conversions and the vector length.
target triple = "aarch64-unknown-linux-gnu"

define <vscale x 4 x i32> @int_s(<vscale x 4 x i32> %a, <vscale x 4 x i32> %b) #0 {
  %m = mul <vscale x 4 x i32> %a, %b
  %d = sdiv <vscale x 4 x i32> %m, %b
  %u = udiv <vscale x 4 x i32> %m, %a
  %k = and <vscale x 4 x i32> %b, splat (i32 7)
  %l = shl <vscale x 4 x i32> %d, %k
  %r = ashr <vscale x 4 x i32> %u, %k
  %x = call <vscale x 4 x i32> @llvm.smax.nxv4i32(<vscale x 4 x i32> %l, <vscale x 4 x i32> %r)
  %y = call <vscale x 4 x i32> @llvm.umin.nxv4i32(<vscale x 4 x i32> %x, <vscale x 4 x i32> %m)
  %z = lshr <vscale x 4 x i32> %y, %k
  %s = add <vscale x 4 x i32> %z, %x
  ret <vscale x 4 x i32> %s
}

define <vscale x 2 x i64> @mul_d(<vscale x 2 x i64> %a, <vscale x 2 x i64> %b) #0 {
  %m = mul <vscale x 2 x i64> %a, %b
  %n = call <vscale x 2 x i64> @llvm.smin.nxv2i64(<vscale x 2 x i64> %m, <vscale x 2 x i64> %a)
  ret <vscale x 2 x i64> %n
}

define <vscale x 8 x i16> @mul_h(<vscale x 8 x i16> %a, <vscale x 8 x i16> %b) #0 {
  %m = mul <vscale x 8 x i16> %a, %b
  %n = call <vscale x 8 x i16> @llvm.umax.nxv8i16(<vscale x 8 x i16> %m, <vscale x 8 x i16> %b)
  ret <vscale x 8 x i16> %n
}

; max(a, b) and a < b unsigned ? a : -1, by compares and selects.
define <vscale x 4 x i32> @select_s(<vscale x 4 x i32> %a, <vscale x 4 x i32> %b) #0 {
  %c = icmp slt <vscale x 4 x i32> %a, %b
  %m = select <vscale x 4 x i1> %c, <vscale x 4 x i32> %b, <vscale x 4 x i32> %a
  %e = icmp ult <vscale x 4 x i32> %a, %b
  %n = select <vscale x 4 x i1> %e, <vscale x 4 x i32> %a, <vscale x 4 x i32> %m
  %q = icmp eq <vscale x 4 x i32> %n, %b
  %r = select <vscale x 4 x i1> %q, <vscale x 4 x i32> %a, <vscale x 4 x i32> %n
  ret <vscale x 4 x i32> %r
}

define <vscale x 16 x i8> @select_b(<vscale x 16 x i8> %a, <vscale x 16 x i8> %b) #0 {
  %c = icmp sge <vscale x 16 x i8> %a, %b
  %m = select <vscale x 16 x i1> %c, <vscale x 16 x i8> %a, <vscale x 16 x i8> %b
  ret <vscale x 16 x i8> %m
}

define <vscale x 2 x i64> @splat_d(i64 %x, <vscale x 2 x i64> %a) #0 {
  %i = insertelement <vscale x 2 x i64> poison, i64 %x, i64 0
  %s = shufflevector <vscale x 2 x i64> %i, <vscale x 2 x i64> poison, <vscale x 2 x i32> zeroinitializer
  %r = add <vscale x 2 x i64> %s, %a
  ret <vscale x 2 x i64> %r
}

define <vscale x 4 x i32> @splat_s(i32 %x, <vscale x 4 x i32> %a) #0 {
  %i = insertelement <vscale x 4 x i32> poison, i32 %x, i64 0
  %s = shufflevector <vscale x 4 x i32> %i, <vscale x 4 x i32> poison, <vscale x 4 x i32> zeroinitializer
  %r = mul <vscale x 4 x i32> %s, %a
  ret <vscale x 4 x i32> %r
}

define i32 @reduce_s(<vscale x 4 x i32> %a) #0 {
  %s = call i32 @llvm.vector.reduce.add.nxv4i32(<vscale x 4 x i32> %a)
  %m = call i32 @llvm.vector.reduce.smax.nxv4i32(<vscale x 4 x i32> %a)
  %n = call i32 @llvm.vector.reduce.umin.nxv4i32(<vscale x 4 x i32> %a)
  %x = call i32 @llvm.vector.reduce.xor.nxv4i32(<vscale x 4 x i32> %a)
  %t = add i32 %s, %m
  %u = xor i32 %t, %n
  %v = add i32 %u, %x
  ret i32 %v
}

define i8 @reduce_b(<vscale x 16 x i8> %a) #0 {
  %s = call i8 @llvm.vector.reduce.add.nxv16i8(<vscale x 16 x i8> %a)
  %o = call i8 @llvm.vector.reduce.or.nxv16i8(<vscale x 16 x i8> %a)
  %r = xor i8 %s, %o
  ret i8 %r
}

define <vscale x 4 x float> @float_s(<vscale x 4 x float> %a, <vscale x 4 x float> %b, <vscale x 4 x float> %c) #0 {
  %d = fdiv <vscale x 4 x float> %a, %b
  %n = fneg <vscale x 4 x float> %d
  %f = call <vscale x 4 x float> @llvm.fabs.nxv4f32(<vscale x 4 x float> %n)
  %q = call <vscale x 4 x float> @llvm.sqrt.nxv4f32(<vscale x 4 x float> %f)
  %m = call <vscale x 4 x float> @llvm.fmuladd.nxv4f32(<vscale x 4 x float> %q, <vscale x 4 x float> %b, <vscale x 4 x float> %c)
  %x = call <vscale x 4 x float> @llvm.minnum.nxv4f32(<vscale x 4 x float> %m, <vscale x 4 x float> %a)
  %l = fcmp olt <vscale x 4 x float> %x, %c
  %r = select <vscale x 4 x i1> %l, <vscale x 4 x float> %c, <vscale x 4 x float> %x
  ret <vscale x 4 x float> %r
}

define <vscale x 2 x double> @float_d(<vscale x 2 x double> %a, <vscale x 2 x double> %b, <vscale x 2 x double> %c) #0 {
  %m = call <vscale x 2 x double> @llvm.fma.nxv2f64(<vscale x 2 x double> %a, <vscale x 2 x double> %b, <vscale x 2 x double> %c)
  %x = call <vscale x 2 x double> @llvm.maxnum.nxv2f64(<vscale x 2 x double> %m, <vscale x 2 x double> %b)
  %d = fdiv <vscale x 2 x double> %x, %a
  ret <vscale x 2 x double> %d
}

define float @fsum_s(float %s, <vscale x 4 x float> %a) #0 {
  %r = call float @llvm.vector.reduce.fadd.nxv4f32(float %s, <vscale x 4 x float> %a)
  ret float %r
}

define <vscale x 4 x i32> @convert_s(<vscale x 4 x i32> %a, <vscale x 4 x float> %b) #0 {
  %f = sitofp <vscale x 4 x i32> %a to <vscale x 4 x float>
  %g = fmul <vscale x 4 x float> %f, %b
  %i = fptosi <vscale x 4 x float> %g to <vscale x 4 x i32>
  %u = uitofp <vscale x 4 x i32> %a to <vscale x 4 x float>
  %j = fptoui <vscale x 4 x float> %u to <vscale x 4 x i32>
  %r = add <vscale x 4 x i32> %i, %j
  ret <vscale x 4 x i32> %r
}

define i64 @lanes() #0 {
  %v = call i64 @llvm.vscale.i64()
  %r = shl i64 %v, 2
  ret i64 %r
}

declare <vscale x 4 x i32> @llvm.smax.nxv4i32(<vscale x 4 x i32>, <vscale x 4 x i32>)
declare <vscale x 4 x i32> @llvm.umin.nxv4i32(<vscale x 4 x i32>, <vscale x 4 x i32>)
declare <vscale x 2 x i64> @llvm.smin.nxv2i64(<vscale x 2 x i64>, <vscale x 2 x i64>)
declare <vscale x 8 x i16> @llvm.umax.nxv8i16(<vscale x 8 x i16>, <vscale x 8 x i16>)
declare i32 @llvm.vector.reduce.add.nxv4i32(<vscale x 4 x i32>)
declare i32 @llvm.vector.reduce.smax.nxv4i32(<vscale x 4 x i32>)
declare i32 @llvm.vector.reduce.umin.nxv4i32(<vscale x 4 x i32>)
declare i32 @llvm.vector.reduce.xor.nxv4i32(<vscale x 4 x i32>)
declare i8 @llvm.vector.reduce.add.nxv16i8(<vscale x 16 x i8>)
declare i8 @llvm.vector.reduce.or.nxv16i8(<vscale x 16 x i8>)
declare <vscale x 4 x float> @llvm.fabs.nxv4f32(<vscale x 4 x float>)
declare <vscale x 4 x float> @llvm.sqrt.nxv4f32(<vscale x 4 x float>)
declare <vscale x 4 x float> @llvm.fmuladd.nxv4f32(<vscale x 4 x float>, <vscale x 4 x float>, <vscale x 4 x float>)
declare <vscale x 4 x float> @llvm.minnum.nxv4f32(<vscale x 4 x float>, <vscale x 4 x float>)
declare <vscale x 2 x double> @llvm.fma.nxv2f64(<vscale x 2 x double>, <vscale x 2 x double>, <vscale x 2 x double>)
declare <vscale x 2 x double> @llvm.maxnum.nxv2f64(<vscale x 2 x double>, <vscale x 2 x double>)
declare float @llvm.vector.reduce.fadd.nxv4f32(float, <vscale x 4 x float>)
declare i64 @llvm.vscale.i64()

attributes #0 = { "target-features"="+sve" }
