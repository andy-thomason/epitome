; AVX-512 kernels over 512-bit integer vectors (tests/avx512.rs): built by
; epitome-llc and by clang, linked with main.c, and run; the two must print
; the same. Each takes vectors at %a and %b and writes to %o.
target triple = "x86_64-unknown-linux-gnu"

define void @arith_v16i32(ptr %a, ptr %b, ptr %o) #0 {
  %x = load <16 x i32>, ptr %a
  %y = load <16 x i32>, ptr %b
  %r0 = add <16 x i32> %x, %y
  %p0 = getelementptr <16 x i32>, ptr %o, i64 0
  store <16 x i32> %r0, ptr %p0
  %r1 = sub <16 x i32> %x, %y
  %p1 = getelementptr <16 x i32>, ptr %o, i64 1
  store <16 x i32> %r1, ptr %p1
  %r2 = and <16 x i32> %x, %y
  %p2 = getelementptr <16 x i32>, ptr %o, i64 2
  store <16 x i32> %r2, ptr %p2
  %r3 = or <16 x i32> %x, %y
  %p3 = getelementptr <16 x i32>, ptr %o, i64 3
  store <16 x i32> %r3, ptr %p3
  %r4 = xor <16 x i32> %x, %y
  %p4 = getelementptr <16 x i32>, ptr %o, i64 4
  store <16 x i32> %r4, ptr %p4
  %r5 = mul <16 x i32> %x, %y
  %p5 = getelementptr <16 x i32>, ptr %o, i64 5
  store <16 x i32> %r5, ptr %p5
  %nx = xor <16 x i32> %x, splat (i32 -1)
  %an = and <16 x i32> %nx, %y
  %pa = getelementptr <16 x i32>, ptr %o, i64 6
  store <16 x i32> %an, ptr %pa
  ret void
}

define void @shifts_v16i32(ptr %a, ptr %b, ptr %o) #0 {
  %x = load <16 x i32>, ptr %a
  %y = load <16 x i32>, ptr %b
  %s = and <16 x i32> %y, splat (i32 31)
  %c0 = shl <16 x i32> %x, splat (i32 3)
  %v0 = shl <16 x i32> %x, %s
  %pc0 = getelementptr <16 x i32>, ptr %o, i64 0
  store <16 x i32> %c0, ptr %pc0
  %pv0 = getelementptr <16 x i32>, ptr %o, i64 1
  store <16 x i32> %v0, ptr %pv0
  %c1 = lshr <16 x i32> %x, splat (i32 3)
  %v1 = lshr <16 x i32> %x, %s
  %pc1 = getelementptr <16 x i32>, ptr %o, i64 2
  store <16 x i32> %c1, ptr %pc1
  %pv1 = getelementptr <16 x i32>, ptr %o, i64 3
  store <16 x i32> %v1, ptr %pv1
  %c2 = ashr <16 x i32> %x, splat (i32 3)
  %v2 = ashr <16 x i32> %x, %s
  %pc2 = getelementptr <16 x i32>, ptr %o, i64 4
  store <16 x i32> %c2, ptr %pc2
  %pv2 = getelementptr <16 x i32>, ptr %o, i64 5
  store <16 x i32> %v2, ptr %pv2
  ret void
}

define void @compares_v16i32(ptr %a, ptr %b, ptr %o) #0 {
  %x = load <16 x i32>, ptr %a
  %y = load <16 x i32>, ptr %b
  %c0 = icmp eq <16 x i32> %x, %y
  %m0 = bitcast <16 x i1> %c0 to i16
  %p0 = getelementptr i16, ptr %o, i64 0
  store i16 %m0, ptr %p0
  %c1 = icmp ne <16 x i32> %x, %y
  %m1 = bitcast <16 x i1> %c1 to i16
  %p1 = getelementptr i16, ptr %o, i64 1
  store i16 %m1, ptr %p1
  %c2 = icmp slt <16 x i32> %x, %y
  %m2 = bitcast <16 x i1> %c2 to i16
  %p2 = getelementptr i16, ptr %o, i64 2
  store i16 %m2, ptr %p2
  %c3 = icmp sle <16 x i32> %x, %y
  %m3 = bitcast <16 x i1> %c3 to i16
  %p3 = getelementptr i16, ptr %o, i64 3
  store i16 %m3, ptr %p3
  %c4 = icmp sgt <16 x i32> %x, %y
  %m4 = bitcast <16 x i1> %c4 to i16
  %p4 = getelementptr i16, ptr %o, i64 4
  store i16 %m4, ptr %p4
  %c5 = icmp sge <16 x i32> %x, %y
  %m5 = bitcast <16 x i1> %c5 to i16
  %p5 = getelementptr i16, ptr %o, i64 5
  store i16 %m5, ptr %p5
  %c6 = icmp ult <16 x i32> %x, %y
  %m6 = bitcast <16 x i1> %c6 to i16
  %p6 = getelementptr i16, ptr %o, i64 6
  store i16 %m6, ptr %p6
  %c7 = icmp ule <16 x i32> %x, %y
  %m7 = bitcast <16 x i1> %c7 to i16
  %p7 = getelementptr i16, ptr %o, i64 7
  store i16 %m7, ptr %p7
  %c8 = icmp ugt <16 x i32> %x, %y
  %m8 = bitcast <16 x i1> %c8 to i16
  %p8 = getelementptr i16, ptr %o, i64 8
  store i16 %m8, ptr %p8
  %c9 = icmp uge <16 x i32> %x, %y
  %m9 = bitcast <16 x i1> %c9 to i16
  %p9 = getelementptr i16, ptr %o, i64 9
  store i16 %m9, ptr %p9
  ret void
}

define void @selects_v16i32(ptr %a, ptr %b, ptr %o, i16 %bits, i32 %k) #0 {
  %x = load <16 x i32>, ptr %a
  %y = load <16 x i32>, ptr %b
  %c = icmp ugt <16 x i32> %x, %y
  %s0 = select <16 x i1> %c, <16 x i32> %x, <16 x i32> %y
  store <16 x i32> %s0, ptr %o
  %m = bitcast i16 %bits to <16 x i1>
  %s1 = select <16 x i1> %m, <16 x i32> %x, <16 x i32> %y
  %p1 = getelementptr <16 x i32>, ptr %o, i64 1
  store <16 x i32> %s1, ptr %p1
  %d = icmp slt <16 x i32> %x, zeroinitializer
  %e = and <16 x i1> %c, %d
  %f = xor <16 x i1> %e, %m
  %g = or <16 x i1> %f, %c
  %s2 = select <16 x i1> %g, <16 x i32> %x, <16 x i32> splat (i32 77)
  %p2 = getelementptr <16 x i32>, ptr %o, i64 2
  store <16 x i32> %s2, ptr %p2
  %z = zext <16 x i1> %f to <16 x i32>
  %w = sext <16 x i1> %e to <16 x i32>
  %zw = add <16 x i32> %z, %w
  %p3 = getelementptr <16 x i32>, ptr %o, i64 3
  store <16 x i32> %zw, ptr %p3
  %i = insertelement <16 x i32> poison, i32 %k, i64 0
  %sp = shufflevector <16 x i32> %i, <16 x i32> poison, <16 x i32> zeroinitializer
  %s4 = select <16 x i1> %c, <16 x i32> %sp, <16 x i32> %y
  %p4 = getelementptr <16 x i32>, ptr %o, i64 4
  store <16 x i32> %s4, ptr %p4
  %gm = bitcast <16 x i1> %g to i16
  %p5 = getelementptr <16 x i32>, ptr %o, i64 5
  store i16 %gm, ptr %p5
  ret void
}

define void @mask_memory_v16i32(ptr %a, ptr %b, ptr %o, ptr %mp) #0 {
  %x = load <16 x i32>, ptr %a
  %y = load <16 x i32>, ptr %b
  %m = load <16 x i1>, ptr %mp
  %c = icmp sle <16 x i32> %x, %y
  %e = xor <16 x i1> %c, %m
  store <16 x i1> %e, ptr %mp
  %s = select <16 x i1> %e, <16 x i32> %y, <16 x i32> %x
  store <16 x i32> %s, ptr %o
  ret void
}

declare <16 x i32> @llvm.smin.v16i32(<16 x i32>, <16 x i32>)
declare <16 x i32> @llvm.smax.v16i32(<16 x i32>, <16 x i32>)
declare <16 x i32> @llvm.umin.v16i32(<16 x i32>, <16 x i32>)
declare <16 x i32> @llvm.umax.v16i32(<16 x i32>, <16 x i32>)
declare <16 x i32> @llvm.abs.v16i32(<16 x i32>, i1)

define void @minmax_v16i32(ptr %a, ptr %b, ptr %o) #0 {
  %x = load <16 x i32>, ptr %a
  %y = load <16 x i32>, ptr %b
  %r0 = call <16 x i32> @llvm.smin.v16i32(<16 x i32> %x, <16 x i32> %y)
  %p0 = getelementptr <16 x i32>, ptr %o, i64 0
  store <16 x i32> %r0, ptr %p0
  %r1 = call <16 x i32> @llvm.smax.v16i32(<16 x i32> %x, <16 x i32> %y)
  %p1 = getelementptr <16 x i32>, ptr %o, i64 1
  store <16 x i32> %r1, ptr %p1
  %r2 = call <16 x i32> @llvm.umin.v16i32(<16 x i32> %x, <16 x i32> %y)
  %p2 = getelementptr <16 x i32>, ptr %o, i64 2
  store <16 x i32> %r2, ptr %p2
  %r3 = call <16 x i32> @llvm.umax.v16i32(<16 x i32> %x, <16 x i32> %y)
  %p3 = getelementptr <16 x i32>, ptr %o, i64 3
  store <16 x i32> %r3, ptr %p3
  %r4 = call <16 x i32> @llvm.abs.v16i32(<16 x i32> %x, i1 false)
  %p4 = getelementptr <16 x i32>, ptr %o, i64 4
  store <16 x i32> %r4, ptr %p4
  ret void
}

declare <16 x i32> @llvm.ctlz.v16i32(<16 x i32>, i1)
declare <16 x i32> @llvm.cttz.v16i32(<16 x i32>, i1)

define void @bits_v16i32(ptr %a, ptr %b, ptr %o) #0 {
  %x = load <16 x i32>, ptr %a
  %y = load <16 x i32>, ptr %b
  %l = call <16 x i32> @llvm.ctlz.v16i32(<16 x i32> %x, i1 false)
  store <16 x i32> %l, ptr %o
  %t = call <16 x i32> @llvm.cttz.v16i32(<16 x i32> %x, i1 false)
  %p1 = getelementptr <16 x i32>, ptr %o, i64 1
  store <16 x i32> %t, ptr %p1
  ret void
}

define void @arith_v8i64(ptr %a, ptr %b, ptr %o) #0 {
  %x = load <8 x i64>, ptr %a
  %y = load <8 x i64>, ptr %b
  %r0 = add <8 x i64> %x, %y
  %p0 = getelementptr <8 x i64>, ptr %o, i64 0
  store <8 x i64> %r0, ptr %p0
  %r1 = sub <8 x i64> %x, %y
  %p1 = getelementptr <8 x i64>, ptr %o, i64 1
  store <8 x i64> %r1, ptr %p1
  %r2 = and <8 x i64> %x, %y
  %p2 = getelementptr <8 x i64>, ptr %o, i64 2
  store <8 x i64> %r2, ptr %p2
  %r3 = or <8 x i64> %x, %y
  %p3 = getelementptr <8 x i64>, ptr %o, i64 3
  store <8 x i64> %r3, ptr %p3
  %r4 = xor <8 x i64> %x, %y
  %p4 = getelementptr <8 x i64>, ptr %o, i64 4
  store <8 x i64> %r4, ptr %p4
  %r5 = mul <8 x i64> %x, %y
  %p5 = getelementptr <8 x i64>, ptr %o, i64 5
  store <8 x i64> %r5, ptr %p5
  %nx = xor <8 x i64> %x, splat (i64 -1)
  %an = and <8 x i64> %nx, %y
  %pa = getelementptr <8 x i64>, ptr %o, i64 6
  store <8 x i64> %an, ptr %pa
  ret void
}

define void @shifts_v8i64(ptr %a, ptr %b, ptr %o) #0 {
  %x = load <8 x i64>, ptr %a
  %y = load <8 x i64>, ptr %b
  %s = and <8 x i64> %y, splat (i64 63)
  %c0 = shl <8 x i64> %x, splat (i64 3)
  %v0 = shl <8 x i64> %x, %s
  %pc0 = getelementptr <8 x i64>, ptr %o, i64 0
  store <8 x i64> %c0, ptr %pc0
  %pv0 = getelementptr <8 x i64>, ptr %o, i64 1
  store <8 x i64> %v0, ptr %pv0
  %c1 = lshr <8 x i64> %x, splat (i64 3)
  %v1 = lshr <8 x i64> %x, %s
  %pc1 = getelementptr <8 x i64>, ptr %o, i64 2
  store <8 x i64> %c1, ptr %pc1
  %pv1 = getelementptr <8 x i64>, ptr %o, i64 3
  store <8 x i64> %v1, ptr %pv1
  %c2 = ashr <8 x i64> %x, splat (i64 3)
  %v2 = ashr <8 x i64> %x, %s
  %pc2 = getelementptr <8 x i64>, ptr %o, i64 4
  store <8 x i64> %c2, ptr %pc2
  %pv2 = getelementptr <8 x i64>, ptr %o, i64 5
  store <8 x i64> %v2, ptr %pv2
  ret void
}

define void @compares_v8i64(ptr %a, ptr %b, ptr %o) #0 {
  %x = load <8 x i64>, ptr %a
  %y = load <8 x i64>, ptr %b
  %c0 = icmp eq <8 x i64> %x, %y
  %m0 = bitcast <8 x i1> %c0 to i8
  %p0 = getelementptr i8, ptr %o, i64 0
  store i8 %m0, ptr %p0
  %c1 = icmp ne <8 x i64> %x, %y
  %m1 = bitcast <8 x i1> %c1 to i8
  %p1 = getelementptr i8, ptr %o, i64 1
  store i8 %m1, ptr %p1
  %c2 = icmp slt <8 x i64> %x, %y
  %m2 = bitcast <8 x i1> %c2 to i8
  %p2 = getelementptr i8, ptr %o, i64 2
  store i8 %m2, ptr %p2
  %c3 = icmp sle <8 x i64> %x, %y
  %m3 = bitcast <8 x i1> %c3 to i8
  %p3 = getelementptr i8, ptr %o, i64 3
  store i8 %m3, ptr %p3
  %c4 = icmp sgt <8 x i64> %x, %y
  %m4 = bitcast <8 x i1> %c4 to i8
  %p4 = getelementptr i8, ptr %o, i64 4
  store i8 %m4, ptr %p4
  %c5 = icmp sge <8 x i64> %x, %y
  %m5 = bitcast <8 x i1> %c5 to i8
  %p5 = getelementptr i8, ptr %o, i64 5
  store i8 %m5, ptr %p5
  %c6 = icmp ult <8 x i64> %x, %y
  %m6 = bitcast <8 x i1> %c6 to i8
  %p6 = getelementptr i8, ptr %o, i64 6
  store i8 %m6, ptr %p6
  %c7 = icmp ule <8 x i64> %x, %y
  %m7 = bitcast <8 x i1> %c7 to i8
  %p7 = getelementptr i8, ptr %o, i64 7
  store i8 %m7, ptr %p7
  %c8 = icmp ugt <8 x i64> %x, %y
  %m8 = bitcast <8 x i1> %c8 to i8
  %p8 = getelementptr i8, ptr %o, i64 8
  store i8 %m8, ptr %p8
  %c9 = icmp uge <8 x i64> %x, %y
  %m9 = bitcast <8 x i1> %c9 to i8
  %p9 = getelementptr i8, ptr %o, i64 9
  store i8 %m9, ptr %p9
  ret void
}

define void @selects_v8i64(ptr %a, ptr %b, ptr %o, i8 %bits, i64 %k) #0 {
  %x = load <8 x i64>, ptr %a
  %y = load <8 x i64>, ptr %b
  %c = icmp ugt <8 x i64> %x, %y
  %s0 = select <8 x i1> %c, <8 x i64> %x, <8 x i64> %y
  store <8 x i64> %s0, ptr %o
  %m = bitcast i8 %bits to <8 x i1>
  %s1 = select <8 x i1> %m, <8 x i64> %x, <8 x i64> %y
  %p1 = getelementptr <8 x i64>, ptr %o, i64 1
  store <8 x i64> %s1, ptr %p1
  %d = icmp slt <8 x i64> %x, zeroinitializer
  %e = and <8 x i1> %c, %d
  %f = xor <8 x i1> %e, %m
  %g = or <8 x i1> %f, %c
  %s2 = select <8 x i1> %g, <8 x i64> %x, <8 x i64> splat (i64 77)
  %p2 = getelementptr <8 x i64>, ptr %o, i64 2
  store <8 x i64> %s2, ptr %p2
  %z = zext <8 x i1> %f to <8 x i64>
  %w = sext <8 x i1> %e to <8 x i64>
  %zw = add <8 x i64> %z, %w
  %p3 = getelementptr <8 x i64>, ptr %o, i64 3
  store <8 x i64> %zw, ptr %p3
  %i = insertelement <8 x i64> poison, i64 %k, i64 0
  %sp = shufflevector <8 x i64> %i, <8 x i64> poison, <8 x i32> zeroinitializer
  %s4 = select <8 x i1> %c, <8 x i64> %sp, <8 x i64> %y
  %p4 = getelementptr <8 x i64>, ptr %o, i64 4
  store <8 x i64> %s4, ptr %p4
  %gm = bitcast <8 x i1> %g to i8
  %p5 = getelementptr <8 x i64>, ptr %o, i64 5
  store i8 %gm, ptr %p5
  ret void
}

define void @mask_memory_v8i64(ptr %a, ptr %b, ptr %o, ptr %mp) #0 {
  %x = load <8 x i64>, ptr %a
  %y = load <8 x i64>, ptr %b
  %m = load <8 x i1>, ptr %mp
  %c = icmp sle <8 x i64> %x, %y
  %e = xor <8 x i1> %c, %m
  store <8 x i1> %e, ptr %mp
  %s = select <8 x i1> %e, <8 x i64> %y, <8 x i64> %x
  store <8 x i64> %s, ptr %o
  ret void
}

declare <8 x i64> @llvm.smin.v8i64(<8 x i64>, <8 x i64>)
declare <8 x i64> @llvm.smax.v8i64(<8 x i64>, <8 x i64>)
declare <8 x i64> @llvm.umin.v8i64(<8 x i64>, <8 x i64>)
declare <8 x i64> @llvm.umax.v8i64(<8 x i64>, <8 x i64>)
declare <8 x i64> @llvm.abs.v8i64(<8 x i64>, i1)

define void @minmax_v8i64(ptr %a, ptr %b, ptr %o) #0 {
  %x = load <8 x i64>, ptr %a
  %y = load <8 x i64>, ptr %b
  %r0 = call <8 x i64> @llvm.smin.v8i64(<8 x i64> %x, <8 x i64> %y)
  %p0 = getelementptr <8 x i64>, ptr %o, i64 0
  store <8 x i64> %r0, ptr %p0
  %r1 = call <8 x i64> @llvm.smax.v8i64(<8 x i64> %x, <8 x i64> %y)
  %p1 = getelementptr <8 x i64>, ptr %o, i64 1
  store <8 x i64> %r1, ptr %p1
  %r2 = call <8 x i64> @llvm.umin.v8i64(<8 x i64> %x, <8 x i64> %y)
  %p2 = getelementptr <8 x i64>, ptr %o, i64 2
  store <8 x i64> %r2, ptr %p2
  %r3 = call <8 x i64> @llvm.umax.v8i64(<8 x i64> %x, <8 x i64> %y)
  %p3 = getelementptr <8 x i64>, ptr %o, i64 3
  store <8 x i64> %r3, ptr %p3
  %r4 = call <8 x i64> @llvm.abs.v8i64(<8 x i64> %x, i1 false)
  %p4 = getelementptr <8 x i64>, ptr %o, i64 4
  store <8 x i64> %r4, ptr %p4
  ret void
}

declare <8 x i64> @llvm.ctlz.v8i64(<8 x i64>, i1)
declare <8 x i64> @llvm.cttz.v8i64(<8 x i64>, i1)

define void @bits_v8i64(ptr %a, ptr %b, ptr %o) #0 {
  %x = load <8 x i64>, ptr %a
  %y = load <8 x i64>, ptr %b
  %l = call <8 x i64> @llvm.ctlz.v8i64(<8 x i64> %x, i1 false)
  store <8 x i64> %l, ptr %o
  %t = call <8 x i64> @llvm.cttz.v8i64(<8 x i64> %x, i1 false)
  %p1 = getelementptr <8 x i64>, ptr %o, i64 1
  store <8 x i64> %t, ptr %p1
  ret void
}

define void @arith_v32i16(ptr %a, ptr %b, ptr %o) #0 {
  %x = load <32 x i16>, ptr %a
  %y = load <32 x i16>, ptr %b
  %r0 = add <32 x i16> %x, %y
  %p0 = getelementptr <32 x i16>, ptr %o, i64 0
  store <32 x i16> %r0, ptr %p0
  %r1 = sub <32 x i16> %x, %y
  %p1 = getelementptr <32 x i16>, ptr %o, i64 1
  store <32 x i16> %r1, ptr %p1
  %r2 = and <32 x i16> %x, %y
  %p2 = getelementptr <32 x i16>, ptr %o, i64 2
  store <32 x i16> %r2, ptr %p2
  %r3 = or <32 x i16> %x, %y
  %p3 = getelementptr <32 x i16>, ptr %o, i64 3
  store <32 x i16> %r3, ptr %p3
  %r4 = xor <32 x i16> %x, %y
  %p4 = getelementptr <32 x i16>, ptr %o, i64 4
  store <32 x i16> %r4, ptr %p4
  %r5 = mul <32 x i16> %x, %y
  %p5 = getelementptr <32 x i16>, ptr %o, i64 5
  store <32 x i16> %r5, ptr %p5
  %nx = xor <32 x i16> %x, splat (i16 -1)
  %an = and <32 x i16> %nx, %y
  %pa = getelementptr <32 x i16>, ptr %o, i64 6
  store <32 x i16> %an, ptr %pa
  ret void
}

define void @shifts_v32i16(ptr %a, ptr %b, ptr %o) #0 {
  %x = load <32 x i16>, ptr %a
  %y = load <32 x i16>, ptr %b
  %s = and <32 x i16> %y, splat (i16 15)
  %c0 = shl <32 x i16> %x, splat (i16 3)
  %v0 = shl <32 x i16> %x, %s
  %pc0 = getelementptr <32 x i16>, ptr %o, i64 0
  store <32 x i16> %c0, ptr %pc0
  %pv0 = getelementptr <32 x i16>, ptr %o, i64 1
  store <32 x i16> %v0, ptr %pv0
  %c1 = lshr <32 x i16> %x, splat (i16 3)
  %v1 = lshr <32 x i16> %x, %s
  %pc1 = getelementptr <32 x i16>, ptr %o, i64 2
  store <32 x i16> %c1, ptr %pc1
  %pv1 = getelementptr <32 x i16>, ptr %o, i64 3
  store <32 x i16> %v1, ptr %pv1
  %c2 = ashr <32 x i16> %x, splat (i16 3)
  %v2 = ashr <32 x i16> %x, %s
  %pc2 = getelementptr <32 x i16>, ptr %o, i64 4
  store <32 x i16> %c2, ptr %pc2
  %pv2 = getelementptr <32 x i16>, ptr %o, i64 5
  store <32 x i16> %v2, ptr %pv2
  ret void
}

define void @compares_v32i16(ptr %a, ptr %b, ptr %o) #0 {
  %x = load <32 x i16>, ptr %a
  %y = load <32 x i16>, ptr %b
  %c0 = icmp eq <32 x i16> %x, %y
  %m0 = bitcast <32 x i1> %c0 to i32
  %p0 = getelementptr i32, ptr %o, i64 0
  store i32 %m0, ptr %p0
  %c1 = icmp ne <32 x i16> %x, %y
  %m1 = bitcast <32 x i1> %c1 to i32
  %p1 = getelementptr i32, ptr %o, i64 1
  store i32 %m1, ptr %p1
  %c2 = icmp slt <32 x i16> %x, %y
  %m2 = bitcast <32 x i1> %c2 to i32
  %p2 = getelementptr i32, ptr %o, i64 2
  store i32 %m2, ptr %p2
  %c3 = icmp sle <32 x i16> %x, %y
  %m3 = bitcast <32 x i1> %c3 to i32
  %p3 = getelementptr i32, ptr %o, i64 3
  store i32 %m3, ptr %p3
  %c4 = icmp sgt <32 x i16> %x, %y
  %m4 = bitcast <32 x i1> %c4 to i32
  %p4 = getelementptr i32, ptr %o, i64 4
  store i32 %m4, ptr %p4
  %c5 = icmp sge <32 x i16> %x, %y
  %m5 = bitcast <32 x i1> %c5 to i32
  %p5 = getelementptr i32, ptr %o, i64 5
  store i32 %m5, ptr %p5
  %c6 = icmp ult <32 x i16> %x, %y
  %m6 = bitcast <32 x i1> %c6 to i32
  %p6 = getelementptr i32, ptr %o, i64 6
  store i32 %m6, ptr %p6
  %c7 = icmp ule <32 x i16> %x, %y
  %m7 = bitcast <32 x i1> %c7 to i32
  %p7 = getelementptr i32, ptr %o, i64 7
  store i32 %m7, ptr %p7
  %c8 = icmp ugt <32 x i16> %x, %y
  %m8 = bitcast <32 x i1> %c8 to i32
  %p8 = getelementptr i32, ptr %o, i64 8
  store i32 %m8, ptr %p8
  %c9 = icmp uge <32 x i16> %x, %y
  %m9 = bitcast <32 x i1> %c9 to i32
  %p9 = getelementptr i32, ptr %o, i64 9
  store i32 %m9, ptr %p9
  ret void
}

define void @selects_v32i16(ptr %a, ptr %b, ptr %o, i32 %bits, i16 %k) #0 {
  %x = load <32 x i16>, ptr %a
  %y = load <32 x i16>, ptr %b
  %c = icmp ugt <32 x i16> %x, %y
  %s0 = select <32 x i1> %c, <32 x i16> %x, <32 x i16> %y
  store <32 x i16> %s0, ptr %o
  %m = bitcast i32 %bits to <32 x i1>
  %s1 = select <32 x i1> %m, <32 x i16> %x, <32 x i16> %y
  %p1 = getelementptr <32 x i16>, ptr %o, i64 1
  store <32 x i16> %s1, ptr %p1
  %d = icmp slt <32 x i16> %x, zeroinitializer
  %e = and <32 x i1> %c, %d
  %f = xor <32 x i1> %e, %m
  %g = or <32 x i1> %f, %c
  %s2 = select <32 x i1> %g, <32 x i16> %x, <32 x i16> splat (i16 77)
  %p2 = getelementptr <32 x i16>, ptr %o, i64 2
  store <32 x i16> %s2, ptr %p2
  %z = zext <32 x i1> %f to <32 x i16>
  %w = sext <32 x i1> %e to <32 x i16>
  %zw = add <32 x i16> %z, %w
  %p3 = getelementptr <32 x i16>, ptr %o, i64 3
  store <32 x i16> %zw, ptr %p3
  %i = insertelement <32 x i16> poison, i16 %k, i64 0
  %sp = shufflevector <32 x i16> %i, <32 x i16> poison, <32 x i32> zeroinitializer
  %s4 = select <32 x i1> %c, <32 x i16> %sp, <32 x i16> %y
  %p4 = getelementptr <32 x i16>, ptr %o, i64 4
  store <32 x i16> %s4, ptr %p4
  %gm = bitcast <32 x i1> %g to i32
  %p5 = getelementptr <32 x i16>, ptr %o, i64 5
  store i32 %gm, ptr %p5
  ret void
}

define void @mask_memory_v32i16(ptr %a, ptr %b, ptr %o, ptr %mp) #0 {
  %x = load <32 x i16>, ptr %a
  %y = load <32 x i16>, ptr %b
  %m = load <32 x i1>, ptr %mp
  %c = icmp sle <32 x i16> %x, %y
  %e = xor <32 x i1> %c, %m
  store <32 x i1> %e, ptr %mp
  %s = select <32 x i1> %e, <32 x i16> %y, <32 x i16> %x
  store <32 x i16> %s, ptr %o
  ret void
}

declare <32 x i16> @llvm.smin.v32i16(<32 x i16>, <32 x i16>)
declare <32 x i16> @llvm.smax.v32i16(<32 x i16>, <32 x i16>)
declare <32 x i16> @llvm.umin.v32i16(<32 x i16>, <32 x i16>)
declare <32 x i16> @llvm.umax.v32i16(<32 x i16>, <32 x i16>)
declare <32 x i16> @llvm.abs.v32i16(<32 x i16>, i1)

define void @minmax_v32i16(ptr %a, ptr %b, ptr %o) #0 {
  %x = load <32 x i16>, ptr %a
  %y = load <32 x i16>, ptr %b
  %r0 = call <32 x i16> @llvm.smin.v32i16(<32 x i16> %x, <32 x i16> %y)
  %p0 = getelementptr <32 x i16>, ptr %o, i64 0
  store <32 x i16> %r0, ptr %p0
  %r1 = call <32 x i16> @llvm.smax.v32i16(<32 x i16> %x, <32 x i16> %y)
  %p1 = getelementptr <32 x i16>, ptr %o, i64 1
  store <32 x i16> %r1, ptr %p1
  %r2 = call <32 x i16> @llvm.umin.v32i16(<32 x i16> %x, <32 x i16> %y)
  %p2 = getelementptr <32 x i16>, ptr %o, i64 2
  store <32 x i16> %r2, ptr %p2
  %r3 = call <32 x i16> @llvm.umax.v32i16(<32 x i16> %x, <32 x i16> %y)
  %p3 = getelementptr <32 x i16>, ptr %o, i64 3
  store <32 x i16> %r3, ptr %p3
  %r4 = call <32 x i16> @llvm.abs.v32i16(<32 x i16> %x, i1 false)
  %p4 = getelementptr <32 x i16>, ptr %o, i64 4
  store <32 x i16> %r4, ptr %p4
  ret void
}

define void @arith_v64i8(ptr %a, ptr %b, ptr %o) #0 {
  %x = load <64 x i8>, ptr %a
  %y = load <64 x i8>, ptr %b
  %r0 = add <64 x i8> %x, %y
  %p0 = getelementptr <64 x i8>, ptr %o, i64 0
  store <64 x i8> %r0, ptr %p0
  %r1 = sub <64 x i8> %x, %y
  %p1 = getelementptr <64 x i8>, ptr %o, i64 1
  store <64 x i8> %r1, ptr %p1
  %r2 = and <64 x i8> %x, %y
  %p2 = getelementptr <64 x i8>, ptr %o, i64 2
  store <64 x i8> %r2, ptr %p2
  %r3 = or <64 x i8> %x, %y
  %p3 = getelementptr <64 x i8>, ptr %o, i64 3
  store <64 x i8> %r3, ptr %p3
  %r4 = xor <64 x i8> %x, %y
  %p4 = getelementptr <64 x i8>, ptr %o, i64 4
  store <64 x i8> %r4, ptr %p4
  %nx = xor <64 x i8> %x, splat (i8 -1)
  %an = and <64 x i8> %nx, %y
  %pa = getelementptr <64 x i8>, ptr %o, i64 5
  store <64 x i8> %an, ptr %pa
  ret void
}

define void @compares_v64i8(ptr %a, ptr %b, ptr %o) #0 {
  %x = load <64 x i8>, ptr %a
  %y = load <64 x i8>, ptr %b
  %c0 = icmp eq <64 x i8> %x, %y
  %m0 = bitcast <64 x i1> %c0 to i64
  %p0 = getelementptr i64, ptr %o, i64 0
  store i64 %m0, ptr %p0
  %c1 = icmp ne <64 x i8> %x, %y
  %m1 = bitcast <64 x i1> %c1 to i64
  %p1 = getelementptr i64, ptr %o, i64 1
  store i64 %m1, ptr %p1
  %c2 = icmp slt <64 x i8> %x, %y
  %m2 = bitcast <64 x i1> %c2 to i64
  %p2 = getelementptr i64, ptr %o, i64 2
  store i64 %m2, ptr %p2
  %c3 = icmp sle <64 x i8> %x, %y
  %m3 = bitcast <64 x i1> %c3 to i64
  %p3 = getelementptr i64, ptr %o, i64 3
  store i64 %m3, ptr %p3
  %c4 = icmp sgt <64 x i8> %x, %y
  %m4 = bitcast <64 x i1> %c4 to i64
  %p4 = getelementptr i64, ptr %o, i64 4
  store i64 %m4, ptr %p4
  %c5 = icmp sge <64 x i8> %x, %y
  %m5 = bitcast <64 x i1> %c5 to i64
  %p5 = getelementptr i64, ptr %o, i64 5
  store i64 %m5, ptr %p5
  %c6 = icmp ult <64 x i8> %x, %y
  %m6 = bitcast <64 x i1> %c6 to i64
  %p6 = getelementptr i64, ptr %o, i64 6
  store i64 %m6, ptr %p6
  %c7 = icmp ule <64 x i8> %x, %y
  %m7 = bitcast <64 x i1> %c7 to i64
  %p7 = getelementptr i64, ptr %o, i64 7
  store i64 %m7, ptr %p7
  %c8 = icmp ugt <64 x i8> %x, %y
  %m8 = bitcast <64 x i1> %c8 to i64
  %p8 = getelementptr i64, ptr %o, i64 8
  store i64 %m8, ptr %p8
  %c9 = icmp uge <64 x i8> %x, %y
  %m9 = bitcast <64 x i1> %c9 to i64
  %p9 = getelementptr i64, ptr %o, i64 9
  store i64 %m9, ptr %p9
  ret void
}

define void @selects_v64i8(ptr %a, ptr %b, ptr %o, i64 %bits, i8 %k) #0 {
  %x = load <64 x i8>, ptr %a
  %y = load <64 x i8>, ptr %b
  %c = icmp ugt <64 x i8> %x, %y
  %s0 = select <64 x i1> %c, <64 x i8> %x, <64 x i8> %y
  store <64 x i8> %s0, ptr %o
  %m = bitcast i64 %bits to <64 x i1>
  %s1 = select <64 x i1> %m, <64 x i8> %x, <64 x i8> %y
  %p1 = getelementptr <64 x i8>, ptr %o, i64 1
  store <64 x i8> %s1, ptr %p1
  %d = icmp slt <64 x i8> %x, zeroinitializer
  %e = and <64 x i1> %c, %d
  %f = xor <64 x i1> %e, %m
  %g = or <64 x i1> %f, %c
  %s2 = select <64 x i1> %g, <64 x i8> %x, <64 x i8> splat (i8 77)
  %p2 = getelementptr <64 x i8>, ptr %o, i64 2
  store <64 x i8> %s2, ptr %p2
  %z = zext <64 x i1> %f to <64 x i8>
  %w = sext <64 x i1> %e to <64 x i8>
  %zw = add <64 x i8> %z, %w
  %p3 = getelementptr <64 x i8>, ptr %o, i64 3
  store <64 x i8> %zw, ptr %p3
  %i = insertelement <64 x i8> poison, i8 %k, i64 0
  %sp = shufflevector <64 x i8> %i, <64 x i8> poison, <64 x i32> zeroinitializer
  %s4 = select <64 x i1> %c, <64 x i8> %sp, <64 x i8> %y
  %p4 = getelementptr <64 x i8>, ptr %o, i64 4
  store <64 x i8> %s4, ptr %p4
  %gm = bitcast <64 x i1> %g to i64
  %p5 = getelementptr <64 x i8>, ptr %o, i64 5
  store i64 %gm, ptr %p5
  ret void
}

define void @mask_memory_v64i8(ptr %a, ptr %b, ptr %o, ptr %mp) #0 {
  %x = load <64 x i8>, ptr %a
  %y = load <64 x i8>, ptr %b
  %m = load <64 x i1>, ptr %mp
  %c = icmp sle <64 x i8> %x, %y
  %e = xor <64 x i1> %c, %m
  store <64 x i1> %e, ptr %mp
  %s = select <64 x i1> %e, <64 x i8> %y, <64 x i8> %x
  store <64 x i8> %s, ptr %o
  ret void
}

declare <64 x i8> @llvm.smin.v64i8(<64 x i8>, <64 x i8>)
declare <64 x i8> @llvm.smax.v64i8(<64 x i8>, <64 x i8>)
declare <64 x i8> @llvm.umin.v64i8(<64 x i8>, <64 x i8>)
declare <64 x i8> @llvm.umax.v64i8(<64 x i8>, <64 x i8>)
declare <64 x i8> @llvm.abs.v64i8(<64 x i8>, i1)

define void @minmax_v64i8(ptr %a, ptr %b, ptr %o) #0 {
  %x = load <64 x i8>, ptr %a
  %y = load <64 x i8>, ptr %b
  %r0 = call <64 x i8> @llvm.smin.v64i8(<64 x i8> %x, <64 x i8> %y)
  %p0 = getelementptr <64 x i8>, ptr %o, i64 0
  store <64 x i8> %r0, ptr %p0
  %r1 = call <64 x i8> @llvm.smax.v64i8(<64 x i8> %x, <64 x i8> %y)
  %p1 = getelementptr <64 x i8>, ptr %o, i64 1
  store <64 x i8> %r1, ptr %p1
  %r2 = call <64 x i8> @llvm.umin.v64i8(<64 x i8> %x, <64 x i8> %y)
  %p2 = getelementptr <64 x i8>, ptr %o, i64 2
  store <64 x i8> %r2, ptr %p2
  %r3 = call <64 x i8> @llvm.umax.v64i8(<64 x i8> %x, <64 x i8> %y)
  %p3 = getelementptr <64 x i8>, ptr %o, i64 3
  store <64 x i8> %r3, ptr %p3
  %r4 = call <64 x i8> @llvm.abs.v64i8(<64 x i8> %x, i1 false)
  %p4 = getelementptr <64 x i8>, ptr %o, i64 4
  store <64 x i8> %r4, ptr %p4
  ret void
}

; Shifts by vectors with AVX2 alone, at 256 and 128 bits.
define void @shifts_v8i32(ptr %a, ptr %b, ptr %o) #1 {
  %x = load <8 x i32>, ptr %a
  %y = load <8 x i32>, ptr %b
  %s = and <8 x i32> %y, splat (i32 31)
  %v0 = shl <8 x i32> %x, %s
  %p0 = getelementptr <8 x i32>, ptr %o, i64 0
  store <8 x i32> %v0, ptr %p0
  %v1 = lshr <8 x i32> %x, %s
  %p1 = getelementptr <8 x i32>, ptr %o, i64 1
  store <8 x i32> %v1, ptr %p1
  %v2 = ashr <8 x i32> %x, %s
  %p2 = getelementptr <8 x i32>, ptr %o, i64 2
  store <8 x i32> %v2, ptr %p2
  ret void
}

define void @shifts_v4i32(ptr %a, ptr %b, ptr %o) #1 {
  %x = load <4 x i32>, ptr %a
  %y = load <4 x i32>, ptr %b
  %s = and <4 x i32> %y, splat (i32 31)
  %v0 = shl <4 x i32> %x, %s
  %p0 = getelementptr <4 x i32>, ptr %o, i64 0
  store <4 x i32> %v0, ptr %p0
  %v1 = lshr <4 x i32> %x, %s
  %p1 = getelementptr <4 x i32>, ptr %o, i64 1
  store <4 x i32> %v1, ptr %p1
  %v2 = ashr <4 x i32> %x, %s
  %p2 = getelementptr <4 x i32>, ptr %o, i64 2
  store <4 x i32> %v2, ptr %p2
  ret void
}

define void @shifts_v4i64(ptr %a, ptr %b, ptr %o) #1 {
  %x = load <4 x i64>, ptr %a
  %y = load <4 x i64>, ptr %b
  %s = and <4 x i64> %y, splat (i64 63)
  %v0 = shl <4 x i64> %x, %s
  %p0 = getelementptr <4 x i64>, ptr %o, i64 0
  store <4 x i64> %v0, ptr %p0
  %v1 = lshr <4 x i64> %x, %s
  %p1 = getelementptr <4 x i64>, ptr %o, i64 1
  store <4 x i64> %v1, ptr %p1
  ret void
}

define void @shifts_v2i64(ptr %a, ptr %b, ptr %o) #1 {
  %x = load <2 x i64>, ptr %a
  %y = load <2 x i64>, ptr %b
  %s = and <2 x i64> %y, splat (i64 63)
  %v0 = shl <2 x i64> %x, %s
  %p0 = getelementptr <2 x i64>, ptr %o, i64 0
  store <2 x i64> %v0, ptr %p0
  %v1 = lshr <2 x i64> %x, %s
  %p1 = getelementptr <2 x i64>, ptr %o, i64 1
  store <2 x i64> %v1, ptr %p1
  ret void
}

define void @fcompares_v16f32(ptr %a, ptr %b, ptr %o) #0 {
  %x = load <16 x float>, ptr %a
  %y = load <16 x float>, ptr %b
  %c0 = fcmp oeq <16 x float> %x, %y
  %m0 = bitcast <16 x i1> %c0 to i16
  %p0 = getelementptr i16, ptr %o, i64 0
  store i16 %m0, ptr %p0
  %c1 = fcmp olt <16 x float> %x, %y
  %m1 = bitcast <16 x i1> %c1 to i16
  %p1 = getelementptr i16, ptr %o, i64 1
  store i16 %m1, ptr %p1
  %c2 = fcmp ole <16 x float> %x, %y
  %m2 = bitcast <16 x i1> %c2 to i16
  %p2 = getelementptr i16, ptr %o, i64 2
  store i16 %m2, ptr %p2
  %c3 = fcmp uno <16 x float> %x, %y
  %m3 = bitcast <16 x i1> %c3 to i16
  %p3 = getelementptr i16, ptr %o, i64 3
  store i16 %m3, ptr %p3
  %c4 = fcmp une <16 x float> %x, %y
  %m4 = bitcast <16 x i1> %c4 to i16
  %p4 = getelementptr i16, ptr %o, i64 4
  store i16 %m4, ptr %p4
  %c5 = fcmp uge <16 x float> %x, %y
  %m5 = bitcast <16 x i1> %c5 to i16
  %p5 = getelementptr i16, ptr %o, i64 5
  store i16 %m5, ptr %p5
  %c6 = fcmp ugt <16 x float> %x, %y
  %m6 = bitcast <16 x i1> %c6 to i16
  %p6 = getelementptr i16, ptr %o, i64 6
  store i16 %m6, ptr %p6
  %c7 = fcmp ord <16 x float> %x, %y
  %m7 = bitcast <16 x i1> %c7 to i16
  %p7 = getelementptr i16, ptr %o, i64 7
  store i16 %m7, ptr %p7
  %c8 = fcmp ueq <16 x float> %x, %y
  %m8 = bitcast <16 x i1> %c8 to i16
  %p8 = getelementptr i16, ptr %o, i64 8
  store i16 %m8, ptr %p8
  %c9 = fcmp ult <16 x float> %x, %y
  %m9 = bitcast <16 x i1> %c9 to i16
  %p9 = getelementptr i16, ptr %o, i64 9
  store i16 %m9, ptr %p9
  %c10 = fcmp ule <16 x float> %x, %y
  %m10 = bitcast <16 x i1> %c10 to i16
  %p10 = getelementptr i16, ptr %o, i64 10
  store i16 %m10, ptr %p10
  %c11 = fcmp one <16 x float> %x, %y
  %m11 = bitcast <16 x i1> %c11 to i16
  %p11 = getelementptr i16, ptr %o, i64 11
  store i16 %m11, ptr %p11
  %c12 = fcmp oge <16 x float> %x, %y
  %m12 = bitcast <16 x i1> %c12 to i16
  %p12 = getelementptr i16, ptr %o, i64 12
  store i16 %m12, ptr %p12
  %c13 = fcmp ogt <16 x float> %x, %y
  %m13 = bitcast <16 x i1> %c13 to i16
  %p13 = getelementptr i16, ptr %o, i64 13
  store i16 %m13, ptr %p13
  ret void
}

define void @fcompares_v8f64(ptr %a, ptr %b, ptr %o) #0 {
  %x = load <8 x double>, ptr %a
  %y = load <8 x double>, ptr %b
  %c0 = fcmp oeq <8 x double> %x, %y
  %m0 = bitcast <8 x i1> %c0 to i8
  %p0 = getelementptr i8, ptr %o, i64 0
  store i8 %m0, ptr %p0
  %c1 = fcmp olt <8 x double> %x, %y
  %m1 = bitcast <8 x i1> %c1 to i8
  %p1 = getelementptr i8, ptr %o, i64 1
  store i8 %m1, ptr %p1
  %c2 = fcmp ole <8 x double> %x, %y
  %m2 = bitcast <8 x i1> %c2 to i8
  %p2 = getelementptr i8, ptr %o, i64 2
  store i8 %m2, ptr %p2
  %c3 = fcmp uno <8 x double> %x, %y
  %m3 = bitcast <8 x i1> %c3 to i8
  %p3 = getelementptr i8, ptr %o, i64 3
  store i8 %m3, ptr %p3
  %c4 = fcmp une <8 x double> %x, %y
  %m4 = bitcast <8 x i1> %c4 to i8
  %p4 = getelementptr i8, ptr %o, i64 4
  store i8 %m4, ptr %p4
  %c5 = fcmp uge <8 x double> %x, %y
  %m5 = bitcast <8 x i1> %c5 to i8
  %p5 = getelementptr i8, ptr %o, i64 5
  store i8 %m5, ptr %p5
  %c6 = fcmp ugt <8 x double> %x, %y
  %m6 = bitcast <8 x i1> %c6 to i8
  %p6 = getelementptr i8, ptr %o, i64 6
  store i8 %m6, ptr %p6
  %c7 = fcmp ord <8 x double> %x, %y
  %m7 = bitcast <8 x i1> %c7 to i8
  %p7 = getelementptr i8, ptr %o, i64 7
  store i8 %m7, ptr %p7
  %c8 = fcmp ueq <8 x double> %x, %y
  %m8 = bitcast <8 x i1> %c8 to i8
  %p8 = getelementptr i8, ptr %o, i64 8
  store i8 %m8, ptr %p8
  %c9 = fcmp ult <8 x double> %x, %y
  %m9 = bitcast <8 x i1> %c9 to i8
  %p9 = getelementptr i8, ptr %o, i64 9
  store i8 %m9, ptr %p9
  %c10 = fcmp ule <8 x double> %x, %y
  %m10 = bitcast <8 x i1> %c10 to i8
  %p10 = getelementptr i8, ptr %o, i64 10
  store i8 %m10, ptr %p10
  %c11 = fcmp one <8 x double> %x, %y
  %m11 = bitcast <8 x i1> %c11 to i8
  %p11 = getelementptr i8, ptr %o, i64 11
  store i8 %m11, ptr %p11
  %c12 = fcmp oge <8 x double> %x, %y
  %m12 = bitcast <8 x i1> %c12 to i8
  %p12 = getelementptr i8, ptr %o, i64 12
  store i8 %m12, ptr %p12
  %c13 = fcmp ogt <8 x double> %x, %y
  %m13 = bitcast <8 x i1> %c13 to i8
  %p13 = getelementptr i8, ptr %o, i64 13
  store i8 %m13, ptr %p13
  ret void
}

; Everything at once, as a netlist's lanes use it: add, logic, shifts by
; constants and by vectors, unsigned comparisons, selects, masks as i16 (in
; and out), and leading zeros.
define i16 @all_v16i32(ptr %a, ptr %b, ptr %o, i16 %enable) #0 {
  %x = load <16 x i32>, ptr %a
  %y = load <16 x i32>, ptr %b
  %s = add <16 x i32> %x, %y
  %m = and <16 x i32> %s, splat (i32 1048575)
  %n = xor <16 x i32> %m, %y
  %sh = and <16 x i32> %x, splat (i32 31)
  %l = shl <16 x i32> %n, splat (i32 2)
  %r = lshr <16 x i32> %l, %sh
  %lo = icmp ult <16 x i32> %r, %y
  %hi = icmp uge <16 x i32> %r, %x
  %en = bitcast i16 %enable to <16 x i1>
  %ok = and <16 x i1> %lo, %en
  %pick = select <16 x i1> %ok, <16 x i32> %r, <16 x i32> %x
  %z = call <16 x i32> @llvm.ctlz.v16i32(<16 x i32> %pick, i1 false)
  %v = select <16 x i1> %hi, <16 x i32> %z, <16 x i32> %pick
  store <16 x i32> %v, ptr %o
  %both = or <16 x i1> %ok, %hi
  %bits = bitcast <16 x i1> %both to i16
  ret i16 %bits
}

; With avx512vl (and avx512bw), the masks of 128- and 256-bit vectors are in
; k registers too: every comparison as its mask's bits, selects, and masks
; in memory, at each width; then masks of one width choosing at another.
define void @compares_v32i8(ptr %a, ptr %b, ptr %o) #0 {
  %x = load <32 x i8>, ptr %a
  %y = load <32 x i8>, ptr %b
  %c0 = icmp eq <32 x i8> %x, %y
  %m0 = bitcast <32 x i1> %c0 to i32
  %p0 = getelementptr i32, ptr %o, i64 0
  store i32 %m0, ptr %p0
  %c1 = icmp ne <32 x i8> %x, %y
  %m1 = bitcast <32 x i1> %c1 to i32
  %p1 = getelementptr i32, ptr %o, i64 1
  store i32 %m1, ptr %p1
  %c2 = icmp slt <32 x i8> %x, %y
  %m2 = bitcast <32 x i1> %c2 to i32
  %p2 = getelementptr i32, ptr %o, i64 2
  store i32 %m2, ptr %p2
  %c3 = icmp sle <32 x i8> %x, %y
  %m3 = bitcast <32 x i1> %c3 to i32
  %p3 = getelementptr i32, ptr %o, i64 3
  store i32 %m3, ptr %p3
  %c4 = icmp sgt <32 x i8> %x, %y
  %m4 = bitcast <32 x i1> %c4 to i32
  %p4 = getelementptr i32, ptr %o, i64 4
  store i32 %m4, ptr %p4
  %c5 = icmp sge <32 x i8> %x, %y
  %m5 = bitcast <32 x i1> %c5 to i32
  %p5 = getelementptr i32, ptr %o, i64 5
  store i32 %m5, ptr %p5
  %c6 = icmp ult <32 x i8> %x, %y
  %m6 = bitcast <32 x i1> %c6 to i32
  %p6 = getelementptr i32, ptr %o, i64 6
  store i32 %m6, ptr %p6
  %c7 = icmp ule <32 x i8> %x, %y
  %m7 = bitcast <32 x i1> %c7 to i32
  %p7 = getelementptr i32, ptr %o, i64 7
  store i32 %m7, ptr %p7
  %c8 = icmp ugt <32 x i8> %x, %y
  %m8 = bitcast <32 x i1> %c8 to i32
  %p8 = getelementptr i32, ptr %o, i64 8
  store i32 %m8, ptr %p8
  %c9 = icmp uge <32 x i8> %x, %y
  %m9 = bitcast <32 x i1> %c9 to i32
  %p9 = getelementptr i32, ptr %o, i64 9
  store i32 %m9, ptr %p9
  ret void
}

define void @selects_v32i8(ptr %a, ptr %b, ptr %o, i64 %bits) #0 {
  %x = load <32 x i8>, ptr %a
  %y = load <32 x i8>, ptr %b
  %bt = trunc i64 %bits to i32
  %c = icmp ugt <32 x i8> %x, %y
  %m = bitcast i32 %bt to <32 x i1>
  %d = icmp slt <32 x i8> %x, zeroinitializer
  %e = and <32 x i1> %c, %d
  %f = xor <32 x i1> %e, %m
  %g = or <32 x i1> %f, %c
  %s0 = select <32 x i1> %g, <32 x i8> %x, <32 x i8> %y
  store <32 x i8> %s0, ptr %o
  %z = zext <32 x i1> %f to <32 x i8>
  %w = sext <32 x i1> %e to <32 x i8>
  %zw = add <32 x i8> %z, %w
  %p1 = getelementptr <32 x i8>, ptr %o, i64 1
  store <32 x i8> %zw, ptr %p1
  %m9 = bitcast <32 x i1> %g to i32
  %p2 = getelementptr <32 x i8>, ptr %o, i64 2
  store i32 %m9, ptr %p2
  ret void
}

define void @mask_memory_v32i8(ptr %a, ptr %b, ptr %o, ptr %mp) #0 {
  %x = load <32 x i8>, ptr %a
  %y = load <32 x i8>, ptr %b
  %m = load <32 x i1>, ptr %mp
  %c = icmp sle <32 x i8> %x, %y
  %e = xor <32 x i1> %c, %m
  store <32 x i1> %e, ptr %mp
  %s = select <32 x i1> %e, <32 x i8> %y, <32 x i8> %x
  store <32 x i8> %s, ptr %o
  ret void
}

define void @compares_v16i16(ptr %a, ptr %b, ptr %o) #0 {
  %x = load <16 x i16>, ptr %a
  %y = load <16 x i16>, ptr %b
  %c0 = icmp eq <16 x i16> %x, %y
  %m0 = bitcast <16 x i1> %c0 to i16
  %p0 = getelementptr i16, ptr %o, i64 0
  store i16 %m0, ptr %p0
  %c1 = icmp ne <16 x i16> %x, %y
  %m1 = bitcast <16 x i1> %c1 to i16
  %p1 = getelementptr i16, ptr %o, i64 1
  store i16 %m1, ptr %p1
  %c2 = icmp slt <16 x i16> %x, %y
  %m2 = bitcast <16 x i1> %c2 to i16
  %p2 = getelementptr i16, ptr %o, i64 2
  store i16 %m2, ptr %p2
  %c3 = icmp sle <16 x i16> %x, %y
  %m3 = bitcast <16 x i1> %c3 to i16
  %p3 = getelementptr i16, ptr %o, i64 3
  store i16 %m3, ptr %p3
  %c4 = icmp sgt <16 x i16> %x, %y
  %m4 = bitcast <16 x i1> %c4 to i16
  %p4 = getelementptr i16, ptr %o, i64 4
  store i16 %m4, ptr %p4
  %c5 = icmp sge <16 x i16> %x, %y
  %m5 = bitcast <16 x i1> %c5 to i16
  %p5 = getelementptr i16, ptr %o, i64 5
  store i16 %m5, ptr %p5
  %c6 = icmp ult <16 x i16> %x, %y
  %m6 = bitcast <16 x i1> %c6 to i16
  %p6 = getelementptr i16, ptr %o, i64 6
  store i16 %m6, ptr %p6
  %c7 = icmp ule <16 x i16> %x, %y
  %m7 = bitcast <16 x i1> %c7 to i16
  %p7 = getelementptr i16, ptr %o, i64 7
  store i16 %m7, ptr %p7
  %c8 = icmp ugt <16 x i16> %x, %y
  %m8 = bitcast <16 x i1> %c8 to i16
  %p8 = getelementptr i16, ptr %o, i64 8
  store i16 %m8, ptr %p8
  %c9 = icmp uge <16 x i16> %x, %y
  %m9 = bitcast <16 x i1> %c9 to i16
  %p9 = getelementptr i16, ptr %o, i64 9
  store i16 %m9, ptr %p9
  ret void
}

define void @selects_v16i16(ptr %a, ptr %b, ptr %o, i64 %bits) #0 {
  %x = load <16 x i16>, ptr %a
  %y = load <16 x i16>, ptr %b
  %bt = trunc i64 %bits to i16
  %c = icmp ugt <16 x i16> %x, %y
  %m = bitcast i16 %bt to <16 x i1>
  %d = icmp slt <16 x i16> %x, zeroinitializer
  %e = and <16 x i1> %c, %d
  %f = xor <16 x i1> %e, %m
  %g = or <16 x i1> %f, %c
  %s0 = select <16 x i1> %g, <16 x i16> %x, <16 x i16> %y
  store <16 x i16> %s0, ptr %o
  %z = zext <16 x i1> %f to <16 x i16>
  %w = sext <16 x i1> %e to <16 x i16>
  %zw = add <16 x i16> %z, %w
  %p1 = getelementptr <16 x i16>, ptr %o, i64 1
  store <16 x i16> %zw, ptr %p1
  %m9 = bitcast <16 x i1> %g to i16
  %p2 = getelementptr <16 x i16>, ptr %o, i64 2
  store i16 %m9, ptr %p2
  ret void
}

define void @mask_memory_v16i16(ptr %a, ptr %b, ptr %o, ptr %mp) #0 {
  %x = load <16 x i16>, ptr %a
  %y = load <16 x i16>, ptr %b
  %m = load <16 x i1>, ptr %mp
  %c = icmp sle <16 x i16> %x, %y
  %e = xor <16 x i1> %c, %m
  store <16 x i1> %e, ptr %mp
  %s = select <16 x i1> %e, <16 x i16> %y, <16 x i16> %x
  store <16 x i16> %s, ptr %o
  ret void
}

define void @compares_v8i32(ptr %a, ptr %b, ptr %o) #0 {
  %x = load <8 x i32>, ptr %a
  %y = load <8 x i32>, ptr %b
  %c0 = icmp eq <8 x i32> %x, %y
  %m0 = bitcast <8 x i1> %c0 to i8
  %p0 = getelementptr i8, ptr %o, i64 0
  store i8 %m0, ptr %p0
  %c1 = icmp ne <8 x i32> %x, %y
  %m1 = bitcast <8 x i1> %c1 to i8
  %p1 = getelementptr i8, ptr %o, i64 1
  store i8 %m1, ptr %p1
  %c2 = icmp slt <8 x i32> %x, %y
  %m2 = bitcast <8 x i1> %c2 to i8
  %p2 = getelementptr i8, ptr %o, i64 2
  store i8 %m2, ptr %p2
  %c3 = icmp sle <8 x i32> %x, %y
  %m3 = bitcast <8 x i1> %c3 to i8
  %p3 = getelementptr i8, ptr %o, i64 3
  store i8 %m3, ptr %p3
  %c4 = icmp sgt <8 x i32> %x, %y
  %m4 = bitcast <8 x i1> %c4 to i8
  %p4 = getelementptr i8, ptr %o, i64 4
  store i8 %m4, ptr %p4
  %c5 = icmp sge <8 x i32> %x, %y
  %m5 = bitcast <8 x i1> %c5 to i8
  %p5 = getelementptr i8, ptr %o, i64 5
  store i8 %m5, ptr %p5
  %c6 = icmp ult <8 x i32> %x, %y
  %m6 = bitcast <8 x i1> %c6 to i8
  %p6 = getelementptr i8, ptr %o, i64 6
  store i8 %m6, ptr %p6
  %c7 = icmp ule <8 x i32> %x, %y
  %m7 = bitcast <8 x i1> %c7 to i8
  %p7 = getelementptr i8, ptr %o, i64 7
  store i8 %m7, ptr %p7
  %c8 = icmp ugt <8 x i32> %x, %y
  %m8 = bitcast <8 x i1> %c8 to i8
  %p8 = getelementptr i8, ptr %o, i64 8
  store i8 %m8, ptr %p8
  %c9 = icmp uge <8 x i32> %x, %y
  %m9 = bitcast <8 x i1> %c9 to i8
  %p9 = getelementptr i8, ptr %o, i64 9
  store i8 %m9, ptr %p9
  ret void
}

define void @selects_v8i32(ptr %a, ptr %b, ptr %o, i64 %bits) #0 {
  %x = load <8 x i32>, ptr %a
  %y = load <8 x i32>, ptr %b
  %bt = trunc i64 %bits to i8
  %c = icmp ugt <8 x i32> %x, %y
  %m = bitcast i8 %bt to <8 x i1>
  %d = icmp slt <8 x i32> %x, zeroinitializer
  %e = and <8 x i1> %c, %d
  %f = xor <8 x i1> %e, %m
  %g = or <8 x i1> %f, %c
  %s0 = select <8 x i1> %g, <8 x i32> %x, <8 x i32> %y
  store <8 x i32> %s0, ptr %o
  %z = zext <8 x i1> %f to <8 x i32>
  %w = sext <8 x i1> %e to <8 x i32>
  %zw = add <8 x i32> %z, %w
  %p1 = getelementptr <8 x i32>, ptr %o, i64 1
  store <8 x i32> %zw, ptr %p1
  %m9 = bitcast <8 x i1> %g to i8
  %p2 = getelementptr <8 x i32>, ptr %o, i64 2
  store i8 %m9, ptr %p2
  ret void
}

define void @mask_memory_v8i32(ptr %a, ptr %b, ptr %o, ptr %mp) #0 {
  %x = load <8 x i32>, ptr %a
  %y = load <8 x i32>, ptr %b
  %m = load <8 x i1>, ptr %mp
  %c = icmp sle <8 x i32> %x, %y
  %e = xor <8 x i1> %c, %m
  store <8 x i1> %e, ptr %mp
  %s = select <8 x i1> %e, <8 x i32> %y, <8 x i32> %x
  store <8 x i32> %s, ptr %o
  ret void
}

define void @compares_v4i64(ptr %a, ptr %b, ptr %o) #0 {
  %x = load <4 x i64>, ptr %a
  %y = load <4 x i64>, ptr %b
  %c0 = icmp eq <4 x i64> %x, %y
  %m0 = bitcast <4 x i1> %c0 to i4
  %w0 = zext i4 %m0 to i8
  %p0 = getelementptr i8, ptr %o, i64 0
  store i8 %w0, ptr %p0
  %c1 = icmp ne <4 x i64> %x, %y
  %m1 = bitcast <4 x i1> %c1 to i4
  %w1 = zext i4 %m1 to i8
  %p1 = getelementptr i8, ptr %o, i64 1
  store i8 %w1, ptr %p1
  %c2 = icmp slt <4 x i64> %x, %y
  %m2 = bitcast <4 x i1> %c2 to i4
  %w2 = zext i4 %m2 to i8
  %p2 = getelementptr i8, ptr %o, i64 2
  store i8 %w2, ptr %p2
  %c3 = icmp sle <4 x i64> %x, %y
  %m3 = bitcast <4 x i1> %c3 to i4
  %w3 = zext i4 %m3 to i8
  %p3 = getelementptr i8, ptr %o, i64 3
  store i8 %w3, ptr %p3
  %c4 = icmp sgt <4 x i64> %x, %y
  %m4 = bitcast <4 x i1> %c4 to i4
  %w4 = zext i4 %m4 to i8
  %p4 = getelementptr i8, ptr %o, i64 4
  store i8 %w4, ptr %p4
  %c5 = icmp sge <4 x i64> %x, %y
  %m5 = bitcast <4 x i1> %c5 to i4
  %w5 = zext i4 %m5 to i8
  %p5 = getelementptr i8, ptr %o, i64 5
  store i8 %w5, ptr %p5
  %c6 = icmp ult <4 x i64> %x, %y
  %m6 = bitcast <4 x i1> %c6 to i4
  %w6 = zext i4 %m6 to i8
  %p6 = getelementptr i8, ptr %o, i64 6
  store i8 %w6, ptr %p6
  %c7 = icmp ule <4 x i64> %x, %y
  %m7 = bitcast <4 x i1> %c7 to i4
  %w7 = zext i4 %m7 to i8
  %p7 = getelementptr i8, ptr %o, i64 7
  store i8 %w7, ptr %p7
  %c8 = icmp ugt <4 x i64> %x, %y
  %m8 = bitcast <4 x i1> %c8 to i4
  %w8 = zext i4 %m8 to i8
  %p8 = getelementptr i8, ptr %o, i64 8
  store i8 %w8, ptr %p8
  %c9 = icmp uge <4 x i64> %x, %y
  %m9 = bitcast <4 x i1> %c9 to i4
  %w9 = zext i4 %m9 to i8
  %p9 = getelementptr i8, ptr %o, i64 9
  store i8 %w9, ptr %p9
  ret void
}

define void @selects_v4i64(ptr %a, ptr %b, ptr %o, i64 %bits) #0 {
  %x = load <4 x i64>, ptr %a
  %y = load <4 x i64>, ptr %b
  %bt = trunc i64 %bits to i4
  %c = icmp ugt <4 x i64> %x, %y
  %m = bitcast i4 %bt to <4 x i1>
  %d = icmp slt <4 x i64> %x, zeroinitializer
  %e = and <4 x i1> %c, %d
  %f = xor <4 x i1> %e, %m
  %g = or <4 x i1> %f, %c
  %s0 = select <4 x i1> %g, <4 x i64> %x, <4 x i64> %y
  store <4 x i64> %s0, ptr %o
  %z = zext <4 x i1> %f to <4 x i64>
  %w = sext <4 x i1> %e to <4 x i64>
  %zw = add <4 x i64> %z, %w
  %p1 = getelementptr <4 x i64>, ptr %o, i64 1
  store <4 x i64> %zw, ptr %p1
  %m9 = bitcast <4 x i1> %g to i4
  %w9 = zext i4 %m9 to i8
  %p2 = getelementptr <4 x i64>, ptr %o, i64 2
  store i8 %w9, ptr %p2
  ret void
}

define void @mask_memory_v4i64(ptr %a, ptr %b, ptr %o, ptr %mp) #0 {
  %x = load <4 x i64>, ptr %a
  %y = load <4 x i64>, ptr %b
  %m = load <4 x i1>, ptr %mp
  %c = icmp sle <4 x i64> %x, %y
  %e = xor <4 x i1> %c, %m
  store <4 x i1> %e, ptr %mp
  %s = select <4 x i1> %e, <4 x i64> %y, <4 x i64> %x
  store <4 x i64> %s, ptr %o
  ret void
}

define void @fcompares_v8f32(ptr %a, ptr %b, ptr %o) #0 {
  %x = load <8 x float>, ptr %a
  %y = load <8 x float>, ptr %b
  %c0 = fcmp oeq <8 x float> %x, %y
  %m0 = bitcast <8 x i1> %c0 to i8
  %p0 = getelementptr i8, ptr %o, i64 0
  store i8 %m0, ptr %p0
  %c1 = fcmp olt <8 x float> %x, %y
  %m1 = bitcast <8 x i1> %c1 to i8
  %p1 = getelementptr i8, ptr %o, i64 1
  store i8 %m1, ptr %p1
  %c2 = fcmp ole <8 x float> %x, %y
  %m2 = bitcast <8 x i1> %c2 to i8
  %p2 = getelementptr i8, ptr %o, i64 2
  store i8 %m2, ptr %p2
  %c3 = fcmp uno <8 x float> %x, %y
  %m3 = bitcast <8 x i1> %c3 to i8
  %p3 = getelementptr i8, ptr %o, i64 3
  store i8 %m3, ptr %p3
  %c4 = fcmp une <8 x float> %x, %y
  %m4 = bitcast <8 x i1> %c4 to i8
  %p4 = getelementptr i8, ptr %o, i64 4
  store i8 %m4, ptr %p4
  %c5 = fcmp uge <8 x float> %x, %y
  %m5 = bitcast <8 x i1> %c5 to i8
  %p5 = getelementptr i8, ptr %o, i64 5
  store i8 %m5, ptr %p5
  %c6 = fcmp ugt <8 x float> %x, %y
  %m6 = bitcast <8 x i1> %c6 to i8
  %p6 = getelementptr i8, ptr %o, i64 6
  store i8 %m6, ptr %p6
  %c7 = fcmp ord <8 x float> %x, %y
  %m7 = bitcast <8 x i1> %c7 to i8
  %p7 = getelementptr i8, ptr %o, i64 7
  store i8 %m7, ptr %p7
  %c8 = fcmp ueq <8 x float> %x, %y
  %m8 = bitcast <8 x i1> %c8 to i8
  %p8 = getelementptr i8, ptr %o, i64 8
  store i8 %m8, ptr %p8
  %c9 = fcmp ult <8 x float> %x, %y
  %m9 = bitcast <8 x i1> %c9 to i8
  %p9 = getelementptr i8, ptr %o, i64 9
  store i8 %m9, ptr %p9
  %c10 = fcmp ule <8 x float> %x, %y
  %m10 = bitcast <8 x i1> %c10 to i8
  %p10 = getelementptr i8, ptr %o, i64 10
  store i8 %m10, ptr %p10
  %c11 = fcmp one <8 x float> %x, %y
  %m11 = bitcast <8 x i1> %c11 to i8
  %p11 = getelementptr i8, ptr %o, i64 11
  store i8 %m11, ptr %p11
  %c12 = fcmp oge <8 x float> %x, %y
  %m12 = bitcast <8 x i1> %c12 to i8
  %p12 = getelementptr i8, ptr %o, i64 12
  store i8 %m12, ptr %p12
  %c13 = fcmp ogt <8 x float> %x, %y
  %m13 = bitcast <8 x i1> %c13 to i8
  %p13 = getelementptr i8, ptr %o, i64 13
  store i8 %m13, ptr %p13
  %s = select <8 x i1> %c1, <8 x float> %x, <8 x float> %y
  %ps = getelementptr i8, ptr %o, i64 64
  store <8 x float> %s, ptr %ps
  ret void
}

define void @fcompares_v4f64(ptr %a, ptr %b, ptr %o) #0 {
  %x = load <4 x double>, ptr %a
  %y = load <4 x double>, ptr %b
  %c0 = fcmp oeq <4 x double> %x, %y
  %m0 = bitcast <4 x i1> %c0 to i4
  %w0 = zext i4 %m0 to i8
  %p0 = getelementptr i8, ptr %o, i64 0
  store i8 %w0, ptr %p0
  %c1 = fcmp olt <4 x double> %x, %y
  %m1 = bitcast <4 x i1> %c1 to i4
  %w1 = zext i4 %m1 to i8
  %p1 = getelementptr i8, ptr %o, i64 1
  store i8 %w1, ptr %p1
  %c2 = fcmp ole <4 x double> %x, %y
  %m2 = bitcast <4 x i1> %c2 to i4
  %w2 = zext i4 %m2 to i8
  %p2 = getelementptr i8, ptr %o, i64 2
  store i8 %w2, ptr %p2
  %c3 = fcmp uno <4 x double> %x, %y
  %m3 = bitcast <4 x i1> %c3 to i4
  %w3 = zext i4 %m3 to i8
  %p3 = getelementptr i8, ptr %o, i64 3
  store i8 %w3, ptr %p3
  %c4 = fcmp une <4 x double> %x, %y
  %m4 = bitcast <4 x i1> %c4 to i4
  %w4 = zext i4 %m4 to i8
  %p4 = getelementptr i8, ptr %o, i64 4
  store i8 %w4, ptr %p4
  %c5 = fcmp uge <4 x double> %x, %y
  %m5 = bitcast <4 x i1> %c5 to i4
  %w5 = zext i4 %m5 to i8
  %p5 = getelementptr i8, ptr %o, i64 5
  store i8 %w5, ptr %p5
  %c6 = fcmp ugt <4 x double> %x, %y
  %m6 = bitcast <4 x i1> %c6 to i4
  %w6 = zext i4 %m6 to i8
  %p6 = getelementptr i8, ptr %o, i64 6
  store i8 %w6, ptr %p6
  %c7 = fcmp ord <4 x double> %x, %y
  %m7 = bitcast <4 x i1> %c7 to i4
  %w7 = zext i4 %m7 to i8
  %p7 = getelementptr i8, ptr %o, i64 7
  store i8 %w7, ptr %p7
  %c8 = fcmp ueq <4 x double> %x, %y
  %m8 = bitcast <4 x i1> %c8 to i4
  %w8 = zext i4 %m8 to i8
  %p8 = getelementptr i8, ptr %o, i64 8
  store i8 %w8, ptr %p8
  %c9 = fcmp ult <4 x double> %x, %y
  %m9 = bitcast <4 x i1> %c9 to i4
  %w9 = zext i4 %m9 to i8
  %p9 = getelementptr i8, ptr %o, i64 9
  store i8 %w9, ptr %p9
  %c10 = fcmp ule <4 x double> %x, %y
  %m10 = bitcast <4 x i1> %c10 to i4
  %w10 = zext i4 %m10 to i8
  %p10 = getelementptr i8, ptr %o, i64 10
  store i8 %w10, ptr %p10
  %c11 = fcmp one <4 x double> %x, %y
  %m11 = bitcast <4 x i1> %c11 to i4
  %w11 = zext i4 %m11 to i8
  %p11 = getelementptr i8, ptr %o, i64 11
  store i8 %w11, ptr %p11
  %c12 = fcmp oge <4 x double> %x, %y
  %m12 = bitcast <4 x i1> %c12 to i4
  %w12 = zext i4 %m12 to i8
  %p12 = getelementptr i8, ptr %o, i64 12
  store i8 %w12, ptr %p12
  %c13 = fcmp ogt <4 x double> %x, %y
  %m13 = bitcast <4 x i1> %c13 to i4
  %w13 = zext i4 %m13 to i8
  %p13 = getelementptr i8, ptr %o, i64 13
  store i8 %w13, ptr %p13
  %s = select <4 x i1> %c1, <4 x double> %x, <4 x double> %y
  %ps = getelementptr i8, ptr %o, i64 64
  store <4 x double> %s, ptr %ps
  ret void
}

define void @compares_v16i8(ptr %a, ptr %b, ptr %o) #0 {
  %x = load <16 x i8>, ptr %a
  %y = load <16 x i8>, ptr %b
  %c0 = icmp eq <16 x i8> %x, %y
  %m0 = bitcast <16 x i1> %c0 to i16
  %p0 = getelementptr i16, ptr %o, i64 0
  store i16 %m0, ptr %p0
  %c1 = icmp ne <16 x i8> %x, %y
  %m1 = bitcast <16 x i1> %c1 to i16
  %p1 = getelementptr i16, ptr %o, i64 1
  store i16 %m1, ptr %p1
  %c2 = icmp slt <16 x i8> %x, %y
  %m2 = bitcast <16 x i1> %c2 to i16
  %p2 = getelementptr i16, ptr %o, i64 2
  store i16 %m2, ptr %p2
  %c3 = icmp sle <16 x i8> %x, %y
  %m3 = bitcast <16 x i1> %c3 to i16
  %p3 = getelementptr i16, ptr %o, i64 3
  store i16 %m3, ptr %p3
  %c4 = icmp sgt <16 x i8> %x, %y
  %m4 = bitcast <16 x i1> %c4 to i16
  %p4 = getelementptr i16, ptr %o, i64 4
  store i16 %m4, ptr %p4
  %c5 = icmp sge <16 x i8> %x, %y
  %m5 = bitcast <16 x i1> %c5 to i16
  %p5 = getelementptr i16, ptr %o, i64 5
  store i16 %m5, ptr %p5
  %c6 = icmp ult <16 x i8> %x, %y
  %m6 = bitcast <16 x i1> %c6 to i16
  %p6 = getelementptr i16, ptr %o, i64 6
  store i16 %m6, ptr %p6
  %c7 = icmp ule <16 x i8> %x, %y
  %m7 = bitcast <16 x i1> %c7 to i16
  %p7 = getelementptr i16, ptr %o, i64 7
  store i16 %m7, ptr %p7
  %c8 = icmp ugt <16 x i8> %x, %y
  %m8 = bitcast <16 x i1> %c8 to i16
  %p8 = getelementptr i16, ptr %o, i64 8
  store i16 %m8, ptr %p8
  %c9 = icmp uge <16 x i8> %x, %y
  %m9 = bitcast <16 x i1> %c9 to i16
  %p9 = getelementptr i16, ptr %o, i64 9
  store i16 %m9, ptr %p9
  ret void
}

define void @selects_v16i8(ptr %a, ptr %b, ptr %o, i64 %bits) #0 {
  %x = load <16 x i8>, ptr %a
  %y = load <16 x i8>, ptr %b
  %bt = trunc i64 %bits to i16
  %c = icmp ugt <16 x i8> %x, %y
  %m = bitcast i16 %bt to <16 x i1>
  %d = icmp slt <16 x i8> %x, zeroinitializer
  %e = and <16 x i1> %c, %d
  %f = xor <16 x i1> %e, %m
  %g = or <16 x i1> %f, %c
  %s0 = select <16 x i1> %g, <16 x i8> %x, <16 x i8> %y
  store <16 x i8> %s0, ptr %o
  %z = zext <16 x i1> %f to <16 x i8>
  %w = sext <16 x i1> %e to <16 x i8>
  %zw = add <16 x i8> %z, %w
  %p1 = getelementptr <16 x i8>, ptr %o, i64 1
  store <16 x i8> %zw, ptr %p1
  %m9 = bitcast <16 x i1> %g to i16
  %p2 = getelementptr <16 x i8>, ptr %o, i64 2
  store i16 %m9, ptr %p2
  ret void
}

define void @mask_memory_v16i8(ptr %a, ptr %b, ptr %o, ptr %mp) #0 {
  %x = load <16 x i8>, ptr %a
  %y = load <16 x i8>, ptr %b
  %m = load <16 x i1>, ptr %mp
  %c = icmp sle <16 x i8> %x, %y
  %e = xor <16 x i1> %c, %m
  store <16 x i1> %e, ptr %mp
  %s = select <16 x i1> %e, <16 x i8> %y, <16 x i8> %x
  store <16 x i8> %s, ptr %o
  ret void
}

define void @compares_v8i16(ptr %a, ptr %b, ptr %o) #0 {
  %x = load <8 x i16>, ptr %a
  %y = load <8 x i16>, ptr %b
  %c0 = icmp eq <8 x i16> %x, %y
  %m0 = bitcast <8 x i1> %c0 to i8
  %p0 = getelementptr i8, ptr %o, i64 0
  store i8 %m0, ptr %p0
  %c1 = icmp ne <8 x i16> %x, %y
  %m1 = bitcast <8 x i1> %c1 to i8
  %p1 = getelementptr i8, ptr %o, i64 1
  store i8 %m1, ptr %p1
  %c2 = icmp slt <8 x i16> %x, %y
  %m2 = bitcast <8 x i1> %c2 to i8
  %p2 = getelementptr i8, ptr %o, i64 2
  store i8 %m2, ptr %p2
  %c3 = icmp sle <8 x i16> %x, %y
  %m3 = bitcast <8 x i1> %c3 to i8
  %p3 = getelementptr i8, ptr %o, i64 3
  store i8 %m3, ptr %p3
  %c4 = icmp sgt <8 x i16> %x, %y
  %m4 = bitcast <8 x i1> %c4 to i8
  %p4 = getelementptr i8, ptr %o, i64 4
  store i8 %m4, ptr %p4
  %c5 = icmp sge <8 x i16> %x, %y
  %m5 = bitcast <8 x i1> %c5 to i8
  %p5 = getelementptr i8, ptr %o, i64 5
  store i8 %m5, ptr %p5
  %c6 = icmp ult <8 x i16> %x, %y
  %m6 = bitcast <8 x i1> %c6 to i8
  %p6 = getelementptr i8, ptr %o, i64 6
  store i8 %m6, ptr %p6
  %c7 = icmp ule <8 x i16> %x, %y
  %m7 = bitcast <8 x i1> %c7 to i8
  %p7 = getelementptr i8, ptr %o, i64 7
  store i8 %m7, ptr %p7
  %c8 = icmp ugt <8 x i16> %x, %y
  %m8 = bitcast <8 x i1> %c8 to i8
  %p8 = getelementptr i8, ptr %o, i64 8
  store i8 %m8, ptr %p8
  %c9 = icmp uge <8 x i16> %x, %y
  %m9 = bitcast <8 x i1> %c9 to i8
  %p9 = getelementptr i8, ptr %o, i64 9
  store i8 %m9, ptr %p9
  ret void
}

define void @selects_v8i16(ptr %a, ptr %b, ptr %o, i64 %bits) #0 {
  %x = load <8 x i16>, ptr %a
  %y = load <8 x i16>, ptr %b
  %bt = trunc i64 %bits to i8
  %c = icmp ugt <8 x i16> %x, %y
  %m = bitcast i8 %bt to <8 x i1>
  %d = icmp slt <8 x i16> %x, zeroinitializer
  %e = and <8 x i1> %c, %d
  %f = xor <8 x i1> %e, %m
  %g = or <8 x i1> %f, %c
  %s0 = select <8 x i1> %g, <8 x i16> %x, <8 x i16> %y
  store <8 x i16> %s0, ptr %o
  %z = zext <8 x i1> %f to <8 x i16>
  %w = sext <8 x i1> %e to <8 x i16>
  %zw = add <8 x i16> %z, %w
  %p1 = getelementptr <8 x i16>, ptr %o, i64 1
  store <8 x i16> %zw, ptr %p1
  %m9 = bitcast <8 x i1> %g to i8
  %p2 = getelementptr <8 x i16>, ptr %o, i64 2
  store i8 %m9, ptr %p2
  ret void
}

define void @mask_memory_v8i16(ptr %a, ptr %b, ptr %o, ptr %mp) #0 {
  %x = load <8 x i16>, ptr %a
  %y = load <8 x i16>, ptr %b
  %m = load <8 x i1>, ptr %mp
  %c = icmp sle <8 x i16> %x, %y
  %e = xor <8 x i1> %c, %m
  store <8 x i1> %e, ptr %mp
  %s = select <8 x i1> %e, <8 x i16> %y, <8 x i16> %x
  store <8 x i16> %s, ptr %o
  ret void
}

define void @compares_v4i32(ptr %a, ptr %b, ptr %o) #0 {
  %x = load <4 x i32>, ptr %a
  %y = load <4 x i32>, ptr %b
  %c0 = icmp eq <4 x i32> %x, %y
  %m0 = bitcast <4 x i1> %c0 to i4
  %w0 = zext i4 %m0 to i8
  %p0 = getelementptr i8, ptr %o, i64 0
  store i8 %w0, ptr %p0
  %c1 = icmp ne <4 x i32> %x, %y
  %m1 = bitcast <4 x i1> %c1 to i4
  %w1 = zext i4 %m1 to i8
  %p1 = getelementptr i8, ptr %o, i64 1
  store i8 %w1, ptr %p1
  %c2 = icmp slt <4 x i32> %x, %y
  %m2 = bitcast <4 x i1> %c2 to i4
  %w2 = zext i4 %m2 to i8
  %p2 = getelementptr i8, ptr %o, i64 2
  store i8 %w2, ptr %p2
  %c3 = icmp sle <4 x i32> %x, %y
  %m3 = bitcast <4 x i1> %c3 to i4
  %w3 = zext i4 %m3 to i8
  %p3 = getelementptr i8, ptr %o, i64 3
  store i8 %w3, ptr %p3
  %c4 = icmp sgt <4 x i32> %x, %y
  %m4 = bitcast <4 x i1> %c4 to i4
  %w4 = zext i4 %m4 to i8
  %p4 = getelementptr i8, ptr %o, i64 4
  store i8 %w4, ptr %p4
  %c5 = icmp sge <4 x i32> %x, %y
  %m5 = bitcast <4 x i1> %c5 to i4
  %w5 = zext i4 %m5 to i8
  %p5 = getelementptr i8, ptr %o, i64 5
  store i8 %w5, ptr %p5
  %c6 = icmp ult <4 x i32> %x, %y
  %m6 = bitcast <4 x i1> %c6 to i4
  %w6 = zext i4 %m6 to i8
  %p6 = getelementptr i8, ptr %o, i64 6
  store i8 %w6, ptr %p6
  %c7 = icmp ule <4 x i32> %x, %y
  %m7 = bitcast <4 x i1> %c7 to i4
  %w7 = zext i4 %m7 to i8
  %p7 = getelementptr i8, ptr %o, i64 7
  store i8 %w7, ptr %p7
  %c8 = icmp ugt <4 x i32> %x, %y
  %m8 = bitcast <4 x i1> %c8 to i4
  %w8 = zext i4 %m8 to i8
  %p8 = getelementptr i8, ptr %o, i64 8
  store i8 %w8, ptr %p8
  %c9 = icmp uge <4 x i32> %x, %y
  %m9 = bitcast <4 x i1> %c9 to i4
  %w9 = zext i4 %m9 to i8
  %p9 = getelementptr i8, ptr %o, i64 9
  store i8 %w9, ptr %p9
  ret void
}

define void @selects_v4i32(ptr %a, ptr %b, ptr %o, i64 %bits) #0 {
  %x = load <4 x i32>, ptr %a
  %y = load <4 x i32>, ptr %b
  %bt = trunc i64 %bits to i4
  %c = icmp ugt <4 x i32> %x, %y
  %m = bitcast i4 %bt to <4 x i1>
  %d = icmp slt <4 x i32> %x, zeroinitializer
  %e = and <4 x i1> %c, %d
  %f = xor <4 x i1> %e, %m
  %g = or <4 x i1> %f, %c
  %s0 = select <4 x i1> %g, <4 x i32> %x, <4 x i32> %y
  store <4 x i32> %s0, ptr %o
  %z = zext <4 x i1> %f to <4 x i32>
  %w = sext <4 x i1> %e to <4 x i32>
  %zw = add <4 x i32> %z, %w
  %p1 = getelementptr <4 x i32>, ptr %o, i64 1
  store <4 x i32> %zw, ptr %p1
  %m9 = bitcast <4 x i1> %g to i4
  %w9 = zext i4 %m9 to i8
  %p2 = getelementptr <4 x i32>, ptr %o, i64 2
  store i8 %w9, ptr %p2
  ret void
}

define void @mask_memory_v4i32(ptr %a, ptr %b, ptr %o, ptr %mp) #0 {
  %x = load <4 x i32>, ptr %a
  %y = load <4 x i32>, ptr %b
  %m = load <4 x i1>, ptr %mp
  %c = icmp sle <4 x i32> %x, %y
  %e = xor <4 x i1> %c, %m
  store <4 x i1> %e, ptr %mp
  %s = select <4 x i1> %e, <4 x i32> %y, <4 x i32> %x
  store <4 x i32> %s, ptr %o
  ret void
}

define void @compares_v2i64(ptr %a, ptr %b, ptr %o) #0 {
  %x = load <2 x i64>, ptr %a
  %y = load <2 x i64>, ptr %b
  %c0 = icmp eq <2 x i64> %x, %y
  %m0 = bitcast <2 x i1> %c0 to i2
  %w0 = zext i2 %m0 to i8
  %p0 = getelementptr i8, ptr %o, i64 0
  store i8 %w0, ptr %p0
  %c1 = icmp ne <2 x i64> %x, %y
  %m1 = bitcast <2 x i1> %c1 to i2
  %w1 = zext i2 %m1 to i8
  %p1 = getelementptr i8, ptr %o, i64 1
  store i8 %w1, ptr %p1
  %c2 = icmp slt <2 x i64> %x, %y
  %m2 = bitcast <2 x i1> %c2 to i2
  %w2 = zext i2 %m2 to i8
  %p2 = getelementptr i8, ptr %o, i64 2
  store i8 %w2, ptr %p2
  %c3 = icmp sle <2 x i64> %x, %y
  %m3 = bitcast <2 x i1> %c3 to i2
  %w3 = zext i2 %m3 to i8
  %p3 = getelementptr i8, ptr %o, i64 3
  store i8 %w3, ptr %p3
  %c4 = icmp sgt <2 x i64> %x, %y
  %m4 = bitcast <2 x i1> %c4 to i2
  %w4 = zext i2 %m4 to i8
  %p4 = getelementptr i8, ptr %o, i64 4
  store i8 %w4, ptr %p4
  %c5 = icmp sge <2 x i64> %x, %y
  %m5 = bitcast <2 x i1> %c5 to i2
  %w5 = zext i2 %m5 to i8
  %p5 = getelementptr i8, ptr %o, i64 5
  store i8 %w5, ptr %p5
  %c6 = icmp ult <2 x i64> %x, %y
  %m6 = bitcast <2 x i1> %c6 to i2
  %w6 = zext i2 %m6 to i8
  %p6 = getelementptr i8, ptr %o, i64 6
  store i8 %w6, ptr %p6
  %c7 = icmp ule <2 x i64> %x, %y
  %m7 = bitcast <2 x i1> %c7 to i2
  %w7 = zext i2 %m7 to i8
  %p7 = getelementptr i8, ptr %o, i64 7
  store i8 %w7, ptr %p7
  %c8 = icmp ugt <2 x i64> %x, %y
  %m8 = bitcast <2 x i1> %c8 to i2
  %w8 = zext i2 %m8 to i8
  %p8 = getelementptr i8, ptr %o, i64 8
  store i8 %w8, ptr %p8
  %c9 = icmp uge <2 x i64> %x, %y
  %m9 = bitcast <2 x i1> %c9 to i2
  %w9 = zext i2 %m9 to i8
  %p9 = getelementptr i8, ptr %o, i64 9
  store i8 %w9, ptr %p9
  ret void
}

define void @selects_v2i64(ptr %a, ptr %b, ptr %o, i64 %bits) #0 {
  %x = load <2 x i64>, ptr %a
  %y = load <2 x i64>, ptr %b
  %bt = trunc i64 %bits to i2
  %c = icmp ugt <2 x i64> %x, %y
  %m = bitcast i2 %bt to <2 x i1>
  %d = icmp slt <2 x i64> %x, zeroinitializer
  %e = and <2 x i1> %c, %d
  %f = xor <2 x i1> %e, %m
  %g = or <2 x i1> %f, %c
  %s0 = select <2 x i1> %g, <2 x i64> %x, <2 x i64> %y
  store <2 x i64> %s0, ptr %o
  %z = zext <2 x i1> %f to <2 x i64>
  %w = sext <2 x i1> %e to <2 x i64>
  %zw = add <2 x i64> %z, %w
  %p1 = getelementptr <2 x i64>, ptr %o, i64 1
  store <2 x i64> %zw, ptr %p1
  %m9 = bitcast <2 x i1> %g to i2
  %w9 = zext i2 %m9 to i8
  %p2 = getelementptr <2 x i64>, ptr %o, i64 2
  store i8 %w9, ptr %p2
  ret void
}

define void @mask_memory_v2i64(ptr %a, ptr %b, ptr %o, ptr %mp) #0 {
  %x = load <2 x i64>, ptr %a
  %y = load <2 x i64>, ptr %b
  %m = load <2 x i1>, ptr %mp
  %c = icmp sle <2 x i64> %x, %y
  %e = xor <2 x i1> %c, %m
  store <2 x i1> %e, ptr %mp
  %s = select <2 x i1> %e, <2 x i64> %y, <2 x i64> %x
  store <2 x i64> %s, ptr %o
  ret void
}

define void @fcompares_v4f32(ptr %a, ptr %b, ptr %o) #0 {
  %x = load <4 x float>, ptr %a
  %y = load <4 x float>, ptr %b
  %c0 = fcmp oeq <4 x float> %x, %y
  %m0 = bitcast <4 x i1> %c0 to i4
  %w0 = zext i4 %m0 to i8
  %p0 = getelementptr i8, ptr %o, i64 0
  store i8 %w0, ptr %p0
  %c1 = fcmp olt <4 x float> %x, %y
  %m1 = bitcast <4 x i1> %c1 to i4
  %w1 = zext i4 %m1 to i8
  %p1 = getelementptr i8, ptr %o, i64 1
  store i8 %w1, ptr %p1
  %c2 = fcmp ole <4 x float> %x, %y
  %m2 = bitcast <4 x i1> %c2 to i4
  %w2 = zext i4 %m2 to i8
  %p2 = getelementptr i8, ptr %o, i64 2
  store i8 %w2, ptr %p2
  %c3 = fcmp uno <4 x float> %x, %y
  %m3 = bitcast <4 x i1> %c3 to i4
  %w3 = zext i4 %m3 to i8
  %p3 = getelementptr i8, ptr %o, i64 3
  store i8 %w3, ptr %p3
  %c4 = fcmp une <4 x float> %x, %y
  %m4 = bitcast <4 x i1> %c4 to i4
  %w4 = zext i4 %m4 to i8
  %p4 = getelementptr i8, ptr %o, i64 4
  store i8 %w4, ptr %p4
  %c5 = fcmp uge <4 x float> %x, %y
  %m5 = bitcast <4 x i1> %c5 to i4
  %w5 = zext i4 %m5 to i8
  %p5 = getelementptr i8, ptr %o, i64 5
  store i8 %w5, ptr %p5
  %c6 = fcmp ugt <4 x float> %x, %y
  %m6 = bitcast <4 x i1> %c6 to i4
  %w6 = zext i4 %m6 to i8
  %p6 = getelementptr i8, ptr %o, i64 6
  store i8 %w6, ptr %p6
  %c7 = fcmp ord <4 x float> %x, %y
  %m7 = bitcast <4 x i1> %c7 to i4
  %w7 = zext i4 %m7 to i8
  %p7 = getelementptr i8, ptr %o, i64 7
  store i8 %w7, ptr %p7
  %c8 = fcmp ueq <4 x float> %x, %y
  %m8 = bitcast <4 x i1> %c8 to i4
  %w8 = zext i4 %m8 to i8
  %p8 = getelementptr i8, ptr %o, i64 8
  store i8 %w8, ptr %p8
  %c9 = fcmp ult <4 x float> %x, %y
  %m9 = bitcast <4 x i1> %c9 to i4
  %w9 = zext i4 %m9 to i8
  %p9 = getelementptr i8, ptr %o, i64 9
  store i8 %w9, ptr %p9
  %c10 = fcmp ule <4 x float> %x, %y
  %m10 = bitcast <4 x i1> %c10 to i4
  %w10 = zext i4 %m10 to i8
  %p10 = getelementptr i8, ptr %o, i64 10
  store i8 %w10, ptr %p10
  %c11 = fcmp one <4 x float> %x, %y
  %m11 = bitcast <4 x i1> %c11 to i4
  %w11 = zext i4 %m11 to i8
  %p11 = getelementptr i8, ptr %o, i64 11
  store i8 %w11, ptr %p11
  %c12 = fcmp oge <4 x float> %x, %y
  %m12 = bitcast <4 x i1> %c12 to i4
  %w12 = zext i4 %m12 to i8
  %p12 = getelementptr i8, ptr %o, i64 12
  store i8 %w12, ptr %p12
  %c13 = fcmp ogt <4 x float> %x, %y
  %m13 = bitcast <4 x i1> %c13 to i4
  %w13 = zext i4 %m13 to i8
  %p13 = getelementptr i8, ptr %o, i64 13
  store i8 %w13, ptr %p13
  %s = select <4 x i1> %c1, <4 x float> %x, <4 x float> %y
  %ps = getelementptr i8, ptr %o, i64 64
  store <4 x float> %s, ptr %ps
  ret void
}

define void @fcompares_v2f64(ptr %a, ptr %b, ptr %o) #0 {
  %x = load <2 x double>, ptr %a
  %y = load <2 x double>, ptr %b
  %c0 = fcmp oeq <2 x double> %x, %y
  %m0 = bitcast <2 x i1> %c0 to i2
  %w0 = zext i2 %m0 to i8
  %p0 = getelementptr i8, ptr %o, i64 0
  store i8 %w0, ptr %p0
  %c1 = fcmp olt <2 x double> %x, %y
  %m1 = bitcast <2 x i1> %c1 to i2
  %w1 = zext i2 %m1 to i8
  %p1 = getelementptr i8, ptr %o, i64 1
  store i8 %w1, ptr %p1
  %c2 = fcmp ole <2 x double> %x, %y
  %m2 = bitcast <2 x i1> %c2 to i2
  %w2 = zext i2 %m2 to i8
  %p2 = getelementptr i8, ptr %o, i64 2
  store i8 %w2, ptr %p2
  %c3 = fcmp uno <2 x double> %x, %y
  %m3 = bitcast <2 x i1> %c3 to i2
  %w3 = zext i2 %m3 to i8
  %p3 = getelementptr i8, ptr %o, i64 3
  store i8 %w3, ptr %p3
  %c4 = fcmp une <2 x double> %x, %y
  %m4 = bitcast <2 x i1> %c4 to i2
  %w4 = zext i2 %m4 to i8
  %p4 = getelementptr i8, ptr %o, i64 4
  store i8 %w4, ptr %p4
  %c5 = fcmp uge <2 x double> %x, %y
  %m5 = bitcast <2 x i1> %c5 to i2
  %w5 = zext i2 %m5 to i8
  %p5 = getelementptr i8, ptr %o, i64 5
  store i8 %w5, ptr %p5
  %c6 = fcmp ugt <2 x double> %x, %y
  %m6 = bitcast <2 x i1> %c6 to i2
  %w6 = zext i2 %m6 to i8
  %p6 = getelementptr i8, ptr %o, i64 6
  store i8 %w6, ptr %p6
  %c7 = fcmp ord <2 x double> %x, %y
  %m7 = bitcast <2 x i1> %c7 to i2
  %w7 = zext i2 %m7 to i8
  %p7 = getelementptr i8, ptr %o, i64 7
  store i8 %w7, ptr %p7
  %c8 = fcmp ueq <2 x double> %x, %y
  %m8 = bitcast <2 x i1> %c8 to i2
  %w8 = zext i2 %m8 to i8
  %p8 = getelementptr i8, ptr %o, i64 8
  store i8 %w8, ptr %p8
  %c9 = fcmp ult <2 x double> %x, %y
  %m9 = bitcast <2 x i1> %c9 to i2
  %w9 = zext i2 %m9 to i8
  %p9 = getelementptr i8, ptr %o, i64 9
  store i8 %w9, ptr %p9
  %c10 = fcmp ule <2 x double> %x, %y
  %m10 = bitcast <2 x i1> %c10 to i2
  %w10 = zext i2 %m10 to i8
  %p10 = getelementptr i8, ptr %o, i64 10
  store i8 %w10, ptr %p10
  %c11 = fcmp one <2 x double> %x, %y
  %m11 = bitcast <2 x i1> %c11 to i2
  %w11 = zext i2 %m11 to i8
  %p11 = getelementptr i8, ptr %o, i64 11
  store i8 %w11, ptr %p11
  %c12 = fcmp oge <2 x double> %x, %y
  %m12 = bitcast <2 x i1> %c12 to i2
  %w12 = zext i2 %m12 to i8
  %p12 = getelementptr i8, ptr %o, i64 12
  store i8 %w12, ptr %p12
  %c13 = fcmp ogt <2 x double> %x, %y
  %m13 = bitcast <2 x i1> %c13 to i2
  %w13 = zext i2 %m13 to i8
  %p13 = getelementptr i8, ptr %o, i64 13
  store i8 %w13, ptr %p13
  %s = select <2 x i1> %c1, <2 x double> %x, <2 x double> %y
  %ps = getelementptr i8, ptr %o, i64 64
  store <2 x double> %s, ptr %ps
  ret void
}

define void @mixed_v8i32_v8i64(ptr %a, ptr %b, ptr %o) #0 {
  %x = load <8 x i32>, ptr %a
  %y = load <8 x i32>, ptr %b
  %c = icmp slt <8 x i32> %x, %y
  %p = load <8 x i64>, ptr %a
  %q = load <8 x i64>, ptr %b
  %s = select <8 x i1> %c, <8 x i64> %p, <8 x i64> %q
  store <8 x i64> %s, ptr %o
  ret void
}

define void @mixed_v4i64_v4f32(ptr %a, ptr %b, ptr %o) #0 {
  %x = load <4 x i64>, ptr %a
  %y = load <4 x i64>, ptr %b
  %c = icmp slt <4 x i64> %x, %y
  %p = load <4 x float>, ptr %a
  %q = load <4 x float>, ptr %b
  %s = select <4 x i1> %c, <4 x float> %p, <4 x float> %q
  store <4 x float> %s, ptr %o
  ret void
}

define void @mixed_v16i16_v16i8(ptr %a, ptr %b, ptr %o) #0 {
  %x = load <16 x i16>, ptr %a
  %y = load <16 x i16>, ptr %b
  %c = icmp slt <16 x i16> %x, %y
  %p = load <16 x i8>, ptr %a
  %q = load <16 x i8>, ptr %b
  %s = select <16 x i1> %c, <16 x i8> %p, <16 x i8> %q
  store <16 x i8> %s, ptr %o
  ret void
}

define void @mixed_v4f64_v4i32(ptr %a, ptr %b, ptr %o) #0 {
  %x = load <4 x double>, ptr %a
  %y = load <4 x double>, ptr %b
  %c = fcmp olt <4 x double> %x, %y
  %p = load <4 x i32>, ptr %a
  %q = load <4 x i32>, ptr %b
  %s = select <4 x i1> %c, <4 x i32> %p, <4 x i32> %q
  store <4 x i32> %s, ptr %o
  ret void
}

; With avx512f but not avx512vl: the masks of 256-bit vectors stay in ymm
; registers, as AVX2's.
define void @novl_selects_v8i32(ptr %a, ptr %b, ptr %o) #2 {
  %x = load <8 x i32>, ptr %a
  %y = load <8 x i32>, ptr %b
  %c = icmp sgt <8 x i32> %x, %y
  %s = select <8 x i1> %c, <8 x i32> %x, <8 x i32> %y
  store <8 x i32> %s, ptr %o
  ret void
}

; Floats: compared, and chosen between.
define void @fsel_v16f32(ptr %a, ptr %b, ptr %o) #0 {
  %x = load <16 x float>, ptr %a
  %y = load <16 x float>, ptr %b
  %c = fcmp olt <16 x float> %x, %y
  %s = select <16 x i1> %c, <16 x float> %x, <16 x float> %y
  store <16 x float> %s, ptr %o
  ret void
}

; A mask of byte lanes choosing between 512-bit vectors.
define void @mixed_v16i8_v16i32(ptr %a, ptr %b, ptr %o) #0 {
  %x = load <16 x i8>, ptr %a
  %y = load <16 x i8>, ptr %b
  %c = icmp ult <16 x i8> %x, %y
  %p = load <16 x i32>, ptr %a
  %q = load <16 x i32>, ptr %b
  %s = select <16 x i1> %c, <16 x i32> %p, <16 x i32> %q
  store <16 x i32> %s, ptr %o
  ret void
}

attributes #0 = { nounwind "target-features"="+avx512f,+avx512bw,+avx512cd,+avx512dq,+avx512vl,+avx2,+avx,+fma" }
attributes #1 = { nounwind "target-features"="+avx2,+avx" }
attributes #2 = { nounwind "target-features"="+avx512f,+avx2,+avx" }
