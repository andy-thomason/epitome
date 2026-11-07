; Lists of registers (st2, st3, st4, tbl2): their members must be in
; consecutive registers, which the values do not arrive in. Members out of
; order, one value twice, values still needed after, lists in a loop, and
; more values live than registers.
target triple = "aarch64-unknown-linux-gnu"

; b then a: the arguments arrive in z0 and z1, the other way round.
define void @st2_swapped(ptr %p, <vscale x 4 x i32> %a, <vscale x 4 x i32> %b) #0 {
  %t = call <vscale x 4 x i1> @llvm.aarch64.sve.ptrue.nxv4i1(i32 31)
  call void @llvm.aarch64.sve.st2.nxv4i32(<vscale x 4 x i32> %b, <vscale x 4 x i32> %a, <vscale x 4 x i1> %t, ptr %p)
  ret void
}

; One value in both places.
define void @st2_same(ptr %p, <vscale x 4 x i32> %a) #0 {
  %t = call <vscale x 4 x i1> @llvm.aarch64.sve.ptrue.nxv4i1(i32 31)
  call void @llvm.aarch64.sve.st2.nxv4i32(<vscale x 4 x i32> %a, <vscale x 4 x i32> %a, <vscale x 4 x i1> %t, ptr %p)
  ret void
}

; Computed members, and a value read again after the list.
define <vscale x 2 x i64> @st3_mixed(ptr %p, <vscale x 2 x i64> %a, <vscale x 2 x i64> %b, <vscale x 2 x i64> %c) #0 {
  %t = call <vscale x 2 x i1> @llvm.aarch64.sve.ptrue.nxv2i1(i32 31)
  %s = add <vscale x 2 x i64> %a, %b
  call void @llvm.aarch64.sve.st3.nxv2i64(<vscale x 2 x i64> %s, <vscale x 2 x i64> %c, <vscale x 2 x i64> %a, <vscale x 2 x i1> %t, ptr %p)
  %r = mul <vscale x 2 x i64> %s, %c
  ret <vscale x 2 x i64> %r
}

; Two lists of the same four values, in opposite orders.
define void @st4_twice(ptr %p, ptr %q, <vscale x 8 x i16> %a, <vscale x 8 x i16> %b, <vscale x 8 x i16> %c, <vscale x 8 x i16> %d) #0 {
  %t = call <vscale x 8 x i1> @llvm.aarch64.sve.ptrue.nxv8i1(i32 31)
  call void @llvm.aarch64.sve.st4.nxv8i16(<vscale x 8 x i16> %a, <vscale x 8 x i16> %b, <vscale x 8 x i16> %c, <vscale x 8 x i16> %d, <vscale x 8 x i1> %t, ptr %p)
  call void @llvm.aarch64.sve.st4.nxv8i16(<vscale x 8 x i16> %d, <vscale x 8 x i16> %c, <vscale x 8 x i16> %b, <vscale x 8 x i16> %a, <vscale x 8 x i1> %t, ptr %q)
  ret void
}

; A table of two registers, looked up both ways round.
define <vscale x 16 x i8> @tbl2_both(<vscale x 16 x i8> %a, <vscale x 16 x i8> %b, <vscale x 16 x i8> %i) #0 {
  %x = call <vscale x 16 x i8> @llvm.aarch64.sve.tbl2.nxv16i8(<vscale x 16 x i8> %a, <vscale x 16 x i8> %b, <vscale x 16 x i8> %i)
  %y = call <vscale x 16 x i8> @llvm.aarch64.sve.tbl2.nxv16i8(<vscale x 16 x i8> %b, <vscale x 16 x i8> %a, <vscale x 16 x i8> %i)
  %r = xor <vscale x 16 x i8> %x, %y
  ret <vscale x 16 x i8> %r
}

; Interleaving in a loop: out[2k] = a[k] + 1, out[2k + 1] = b[k], a
; vector length at a time, for n vectors.
define void @zip_loop(ptr %out, ptr %a, ptr %b, i64 %n) #0 {
entry:
  %t = call <vscale x 4 x i1> @llvm.aarch64.sve.ptrue.nxv4i1(i32 31)
  %vl = call i64 @llvm.vscale.i64()
  %step = shl i64 %vl, 4
  %empty = icmp eq i64 %n, 0
  br i1 %empty, label %done, label %loop

loop:
  %k = phi i64 [ 0, %entry ], [ %k1, %loop ]
  %off = phi i64 [ 0, %entry ], [ %off1, %loop ]
  %pa = getelementptr i8, ptr %a, i64 %off
  %pb = getelementptr i8, ptr %b, i64 %off
  %va = load <vscale x 4 x i32>, ptr %pa
  %vb = load <vscale x 4 x i32>, ptr %pb
  %va1 = add <vscale x 4 x i32> %va, splat (i32 1)
  %o2 = shl i64 %off, 1
  %po = getelementptr i8, ptr %out, i64 %o2
  call void @llvm.aarch64.sve.st2.nxv4i32(<vscale x 4 x i32> %va1, <vscale x 4 x i32> %vb, <vscale x 4 x i1> %t, ptr %po)
  %k1 = add i64 %k, 1
  %off1 = add i64 %off, %step
  %more = icmp ult i64 %k1, %n
  br i1 %more, label %loop, label %done

done:
  ret void
}

; Forty values live at once, then stored four at a time: more than the
; registers, so some are spilled and loaded back into a list.
define void @st4_pressure(ptr %p, <vscale x 4 x i32> %x) #0 {
  %t = call <vscale x 4 x i1> @llvm.aarch64.sve.ptrue.nxv4i1(i32 31)
  %v0 = add <vscale x 4 x i32> %x, splat (i32 1)
  %v1 = mul <vscale x 4 x i32> %v0, %x
  %v2 = add <vscale x 4 x i32> %v1, splat (i32 3)
  %v3 = mul <vscale x 4 x i32> %v2, %v0
  %v4 = add <vscale x 4 x i32> %v3, splat (i32 5)
  %v5 = mul <vscale x 4 x i32> %v4, %v1
  %v6 = add <vscale x 4 x i32> %v5, splat (i32 7)
  %v7 = mul <vscale x 4 x i32> %v6, %v2
  %v8 = add <vscale x 4 x i32> %v7, splat (i32 9)
  %v9 = mul <vscale x 4 x i32> %v8, %v3
  %v10 = add <vscale x 4 x i32> %v9, splat (i32 11)
  %v11 = mul <vscale x 4 x i32> %v10, %v4
  %v12 = add <vscale x 4 x i32> %v11, splat (i32 13)
  %v13 = mul <vscale x 4 x i32> %v12, %v5
  %v14 = add <vscale x 4 x i32> %v13, splat (i32 15)
  %v15 = mul <vscale x 4 x i32> %v14, %v6
  %v16 = add <vscale x 4 x i32> %v15, splat (i32 17)
  %v17 = mul <vscale x 4 x i32> %v16, %v7
  %v18 = add <vscale x 4 x i32> %v17, splat (i32 19)
  %v19 = mul <vscale x 4 x i32> %v18, %v8
  %v20 = add <vscale x 4 x i32> %v19, splat (i32 21)
  %v21 = mul <vscale x 4 x i32> %v20, %v9
  %v22 = add <vscale x 4 x i32> %v21, splat (i32 23)
  %v23 = mul <vscale x 4 x i32> %v22, %v10
  %v24 = add <vscale x 4 x i32> %v23, splat (i32 25)
  %v25 = mul <vscale x 4 x i32> %v24, %v11
  %v26 = add <vscale x 4 x i32> %v25, splat (i32 27)
  %v27 = mul <vscale x 4 x i32> %v26, %v12
  %v28 = add <vscale x 4 x i32> %v27, splat (i32 29)
  %v29 = mul <vscale x 4 x i32> %v28, %v13
  %v30 = add <vscale x 4 x i32> %v29, splat (i32 31)
  %v31 = mul <vscale x 4 x i32> %v30, %v14
  %v32 = add <vscale x 4 x i32> %v31, splat (i32 33)
  %v33 = mul <vscale x 4 x i32> %v32, %v15
  %v34 = add <vscale x 4 x i32> %v33, splat (i32 35)
  %v35 = mul <vscale x 4 x i32> %v34, %v16
  %v36 = add <vscale x 4 x i32> %v35, splat (i32 37)
  %v37 = mul <vscale x 4 x i32> %v36, %v17
  %v38 = add <vscale x 4 x i32> %v37, splat (i32 39)
  %v39 = mul <vscale x 4 x i32> %v38, %v18
  %vl = call i64 @llvm.vscale.i64()
  %w = shl i64 %vl, 6
  %p1 = getelementptr i8, ptr %p, i64 %w
  %p2 = getelementptr i8, ptr %p1, i64 %w
  %p3 = getelementptr i8, ptr %p2, i64 %w
  %p4 = getelementptr i8, ptr %p3, i64 %w
  %p5 = getelementptr i8, ptr %p4, i64 %w
  %p6 = getelementptr i8, ptr %p5, i64 %w
  %p7 = getelementptr i8, ptr %p6, i64 %w
  %p8 = getelementptr i8, ptr %p7, i64 %w
  %p9 = getelementptr i8, ptr %p8, i64 %w
  call void @llvm.aarch64.sve.st4.nxv4i32(<vscale x 4 x i32> %v39, <vscale x 4 x i32> %v0, <vscale x 4 x i32> %v20, <vscale x 4 x i32> %v10, <vscale x 4 x i1> %t, ptr %p)
  call void @llvm.aarch64.sve.st4.nxv4i32(<vscale x 4 x i32> %v1, <vscale x 4 x i32> %v38, <vscale x 4 x i32> %v11, <vscale x 4 x i32> %v21, <vscale x 4 x i1> %t, ptr %p1)
  call void @llvm.aarch64.sve.st4.nxv4i32(<vscale x 4 x i32> %v2, <vscale x 4 x i32> %v12, <vscale x 4 x i32> %v37, <vscale x 4 x i32> %v22, <vscale x 4 x i1> %t, ptr %p2)
  call void @llvm.aarch64.sve.st4.nxv4i32(<vscale x 4 x i32> %v3, <vscale x 4 x i32> %v13, <vscale x 4 x i32> %v23, <vscale x 4 x i32> %v36, <vscale x 4 x i1> %t, ptr %p3)
  call void @llvm.aarch64.sve.st4.nxv4i32(<vscale x 4 x i32> %v4, <vscale x 4 x i32> %v14, <vscale x 4 x i32> %v24, <vscale x 4 x i32> %v35, <vscale x 4 x i1> %t, ptr %p4)
  call void @llvm.aarch64.sve.st4.nxv4i32(<vscale x 4 x i32> %v5, <vscale x 4 x i32> %v15, <vscale x 4 x i32> %v25, <vscale x 4 x i32> %v34, <vscale x 4 x i1> %t, ptr %p5)
  call void @llvm.aarch64.sve.st4.nxv4i32(<vscale x 4 x i32> %v6, <vscale x 4 x i32> %v16, <vscale x 4 x i32> %v26, <vscale x 4 x i32> %v33, <vscale x 4 x i1> %t, ptr %p6)
  call void @llvm.aarch64.sve.st4.nxv4i32(<vscale x 4 x i32> %v7, <vscale x 4 x i32> %v17, <vscale x 4 x i32> %v27, <vscale x 4 x i32> %v32, <vscale x 4 x i1> %t, ptr %p7)
  call void @llvm.aarch64.sve.st4.nxv4i32(<vscale x 4 x i32> %v8, <vscale x 4 x i32> %v18, <vscale x 4 x i32> %v28, <vscale x 4 x i32> %v31, <vscale x 4 x i1> %t, ptr %p8)
  call void @llvm.aarch64.sve.st4.nxv4i32(<vscale x 4 x i32> %v9, <vscale x 4 x i32> %v19, <vscale x 4 x i32> %v29, <vscale x 4 x i32> %v30, <vscale x 4 x i1> %t, ptr %p9)
  ret void
}

declare <vscale x 2 x i1> @llvm.aarch64.sve.ptrue.nxv2i1(i32)
declare <vscale x 4 x i1> @llvm.aarch64.sve.ptrue.nxv4i1(i32)
declare <vscale x 8 x i1> @llvm.aarch64.sve.ptrue.nxv8i1(i32)
declare void @llvm.aarch64.sve.st2.nxv4i32(<vscale x 4 x i32>, <vscale x 4 x i32>, <vscale x 4 x i1>, ptr)
declare void @llvm.aarch64.sve.st3.nxv2i64(<vscale x 2 x i64>, <vscale x 2 x i64>, <vscale x 2 x i64>, <vscale x 2 x i1>, ptr)
declare void @llvm.aarch64.sve.st4.nxv8i16(<vscale x 8 x i16>, <vscale x 8 x i16>, <vscale x 8 x i16>, <vscale x 8 x i16>, <vscale x 8 x i1>, ptr)
declare void @llvm.aarch64.sve.st4.nxv4i32(<vscale x 4 x i32>, <vscale x 4 x i32>, <vscale x 4 x i32>, <vscale x 4 x i32>, <vscale x 4 x i1>, ptr)
declare <vscale x 16 x i8> @llvm.aarch64.sve.tbl2.nxv16i8(<vscale x 16 x i8>, <vscale x 16 x i8>, <vscale x 16 x i8>)
declare i64 @llvm.vscale.i64()

attributes #0 = { "target-features"="+sve,+sve2" }
