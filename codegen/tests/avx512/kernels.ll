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

attributes #0 = { nounwind "target-features"="+avx512f,+avx512bw,+avx512cd,+avx512dq,+avx512vl,+avx2,+avx,+fma" }
