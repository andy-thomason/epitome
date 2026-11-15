; Many masks of 16 lanes live at once with AVX2 (two ymm registers each),
; selected between: spill code must load all three operands of a blend.
; Reduced from epitome-sv's netlist of common_cells' lzc (13 bits).
target triple = "x86_64-unknown-linux-gnu"
define void @sv_eval_1(ptr %s, ptr %p0, ptr %p1, ptr %p2) {
entry:
  br label %L0
L0:                                               ; preds = %L0, %entry
  %c0 = phi i64 [ 0, %entry ], [ %c0.next, %L0 ]
  %c0.off = shl i64 %c0, 6
  %b0 = getelementptr i8, ptr %s, i64 %c0.off
  %c0.b8 = shl i64 %c0, 4
  %c0.b16 = shl i64 %c0, 5
  %t1 = getelementptr i8, ptr %p0, i64 %c0.b16
  %t2 = getelementptr i8, ptr %t1, i64 0
  %t3 = load <16 x i16>, ptr %t2, align 32
  %t4 = zext <16 x i16> %t3 to <16 x i32>
  %t6 = and <16 x i32> %t4, splat (i32 1)
  %t7 = icmp ne <16 x i32> %t6, splat (i32 0)
  %t9 = lshr <16 x i32> %t4, splat (i32 1)
  %t11 = and <16 x i32> %t9, splat (i32 1)
  %t12 = icmp ne <16 x i32> %t11, splat (i32 0)
  %t14 = lshr <16 x i32> %t4, splat (i32 2)
  %t16 = and <16 x i32> %t14, splat (i32 1)
  %t17 = icmp ne <16 x i32> %t16, splat (i32 0)
  %t19 = lshr <16 x i32> %t4, splat (i32 3)
  %t20 = and <16 x i32> %t19, splat (i32 1)
  %t21 = icmp ne <16 x i32> %t20, splat (i32 0)
  %t23 = lshr <16 x i32> %t4, splat (i32 4)
  %t24 = and <16 x i32> %t23, splat (i32 1)
  %t25 = icmp ne <16 x i32> %t24, splat (i32 0)
  %t27 = lshr <16 x i32> %t4, splat (i32 5)
  %t28 = and <16 x i32> %t27, splat (i32 1)
  %t29 = icmp ne <16 x i32> %t28, splat (i32 0)
  %t31 = lshr <16 x i32> %t4, splat (i32 6)
  %t32 = and <16 x i32> %t31, splat (i32 1)
  %t33 = icmp ne <16 x i32> %t32, splat (i32 0)
  %t35 = lshr <16 x i32> %t4, splat (i32 7)
  %t36 = and <16 x i32> %t35, splat (i32 1)
  %t37 = icmp ne <16 x i32> %t36, splat (i32 0)
  %t39 = lshr <16 x i32> %t4, splat (i32 8)
  %t40 = and <16 x i32> %t39, splat (i32 1)
  %t41 = icmp ne <16 x i32> %t40, splat (i32 0)
  %t43 = lshr <16 x i32> %t4, splat (i32 9)
  %t44 = and <16 x i32> %t43, splat (i32 1)
  %t45 = icmp ne <16 x i32> %t44, splat (i32 0)
  %t47 = lshr <16 x i32> %t4, splat (i32 10)
  %t48 = and <16 x i32> %t47, splat (i32 1)
  %t49 = icmp ne <16 x i32> %t48, splat (i32 0)
  %t51 = lshr <16 x i32> %t4, splat (i32 11)
  %t52 = and <16 x i32> %t51, splat (i32 1)
  %t53 = icmp ne <16 x i32> %t52, splat (i32 0)
  %t55 = lshr <16 x i32> %t4, splat (i32 12)
  %t56 = and <16 x i32> %t55, splat (i32 1)
  %t57 = icmp ne <16 x i32> %t56, splat (i32 0)
  %t59 = or <16 x i1> %t7, %t12
  %t62 = select <16 x i1> %t7, <16 x i32> splat (i32 0), <16 x i32> splat (i32 1)
  %t63 = or <16 x i1> %t17, %t21
  %t65 = or <16 x i1> %t59, %t63
  %t68 = select <16 x i1> %t17, <16 x i32> splat (i32 2), <16 x i32> splat (i32 3)
  %t70 = select <16 x i1> %t59, <16 x i32> %t62, <16 x i32> %t68
  %t71 = or <16 x i1> %t25, %t29
  %t74 = select <16 x i1> %t25, <16 x i32> splat (i32 4), <16 x i32> splat (i32 5)
  %t75 = or <16 x i1> %t33, %t37
  %t77 = or <16 x i1> %t71, %t75
  %t79 = or <16 x i1> %t65, %t77
  %t82 = select <16 x i1> %t33, <16 x i32> splat (i32 6), <16 x i32> splat (i32 7)
  %t84 = select <16 x i1> %t71, <16 x i32> %t74, <16 x i32> %t82
  %t86 = select <16 x i1> %t65, <16 x i32> %t70, <16 x i32> %t84
  %t87 = or <16 x i1> %t41, %t45
  %t90 = select <16 x i1> %t41, <16 x i32> splat (i32 8), <16 x i32> splat (i32 9)
  %t91 = or <16 x i1> %t49, %t53
  %t93 = or <16 x i1> %t87, %t91
  %t96 = select <16 x i1> %t49, <16 x i32> splat (i32 10), <16 x i32> splat (i32 11)
  %t98 = select <16 x i1> %t87, <16 x i32> %t90, <16 x i32> %t96
  %t104 = select <16 x i1> %t57, <16 x i32> splat (i32 12), <16 x i32> splat (i32 0)
  %t106 = select <16 x i1> %t93, <16 x i32> %t98, <16 x i32> %t104
  %t108 = select <16 x i1> %t79, <16 x i32> %t86, <16 x i32> %t106
  %t111 = getelementptr i8, ptr %p1, i64 %c0.b8
  %t112 = getelementptr i8, ptr %t111, i64 0
  %t113 = trunc <16 x i32> %t108 to <16 x i8>
  store <16 x i8> %t113, ptr %t112, align 16
  %c0.next = add i64 %c0, 1
  %c0.end = icmp eq i64 %c0.next, 32
  br i1 %c0.end, label %X0, label %L0
X0:                                               ; preds = %L0
  ret void
}
define void @sv_eval_0(ptr %s) {
entry:
  br label %L0
L0:                                               ; preds = %L0, %entry
  %lsr.count = phi i64 [ 32, %entry ], [ %lsr.count.next, %L0 ]
  %lsr.count.next = add i64 %lsr.count, -1
  %c0.end = icmp eq i64 %lsr.count.next, 0
  br i1 %c0.end, label %X0, label %L0
X0:                                               ; preds = %L0
  %t1 = getelementptr i8, ptr %s, i64 2048
  %t2 = getelementptr i8, ptr %s, i64 0
  %t3 = getelementptr i8, ptr %s, i64 1024
  %t4 = getelementptr i8, ptr %s, i64 1536
  tail call void @sv_eval_1(ptr %t1, ptr %t2, ptr %t3, ptr %t4)
  br label %L1
L1:                                               ; preds = %L1, %X0
  %lsr.count1 = phi i64 [ 32, %X0 ], [ %lsr.count.next1, %L1 ]
  %lsr.count.next1 = add i64 %lsr.count1, -1
  %c1.end = icmp eq i64 %lsr.count.next1, 0
  br i1 %c1.end, label %X1, label %L1
X1:                                               ; preds = %L1
  ret void
}
