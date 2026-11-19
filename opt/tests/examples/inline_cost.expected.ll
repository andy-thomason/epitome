; @big: combine: (dead) x27, add_add_const x27, sub_const x27
; @big_once: combine: (dead) x27, add_add_const x27, sub_const x27
; @main: inlined @tiny: cost 1, bonus 6, threshold 100
; @main: inlined @tiny: cost 1, bonus 6, threshold 100
; @main: inlined @mode_op: cost 5, bonus 18, threshold 100
; @main: inlined @mode_op: cost 5, bonus 8, threshold 100
; @main: inlined @big: cost 83, bonus 6, threshold 100
; @main: inlined @big: cost 83, bonus 6, threshold 100
; @main: inlined @big_once: cost 83, bonus 94, threshold 100
; @main: not inlined @never: noinline
; @main: not inlined @big: cost 83 - bonus 6 > threshold 0
; @main: combine: (dead) x2, (folded) x2, (reassociated) x1, (switch folded) x1, add_add_const x1, sub_add_add x2
; @main: simplifycfg: 2 unreachable blocks removed, 1 empty blocks bypassed, 0 selects made, 2 blocks merged, 0 edges into unreachable dropped
; @main: implied: 0 comparisons folded, 0 selects resolved, 1 flags set
; @main: cse: 1 merged, 0 loads reused, 0 stores forwarded
; @tiny: deleted, no longer used
; @mode_op: deleted, no longer used
; @big_once: deleted, no longer used


define internal i32 @big(i32 %x) {
entry:
  %t0 = add i32 %x, 3
  %t1 = mul i32 %t0, 10
  %t2 = xor i32 %t1, 17
  %t4 = add i32 %t2, 7
  %t5 = mul i32 %t4, 38
  %t6 = xor i32 %t5, 45
  %t8 = add i32 %t6, 7
  %t9 = mul i32 %t8, 66
  %t10 = xor i32 %t9, 73
  %t12 = add i32 %t10, 7
  %t13 = mul i32 %t12, 94
  %t14 = xor i32 %t13, 101
  %t16 = add i32 %t14, 7
  %t17 = mul i32 %t16, 122
  %t18 = xor i32 %t17, 129
  %t20 = add i32 %t18, 7
  %t21 = mul i32 %t20, 150
  %t22 = xor i32 %t21, 157
  %t24 = add i32 %t22, 7
  %t25 = mul i32 %t24, 178
  %t26 = xor i32 %t25, 185
  %t28 = add i32 %t26, 7
  %t29 = mul i32 %t28, 206
  %t30 = xor i32 %t29, 213
  %t32 = add i32 %t30, 7
  %t33 = mul i32 %t32, 234
  %t34 = xor i32 %t33, 241
  %t36 = add i32 %t34, 7
  %t37 = mul i32 %t36, 262
  %t38 = xor i32 %t37, 269
  %t40 = add i32 %t38, 7
  %t41 = mul i32 %t40, 290
  %t42 = xor i32 %t41, 297
  %t44 = add i32 %t42, 7
  %t45 = mul i32 %t44, 318
  %t46 = xor i32 %t45, 325
  %t48 = add i32 %t46, 7
  %t49 = mul i32 %t48, 346
  %t50 = xor i32 %t49, 353
  %t52 = add i32 %t50, 7
  %t53 = mul i32 %t52, 374
  %t54 = xor i32 %t53, 381
  %t56 = add i32 %t54, 7
  %t57 = mul i32 %t56, 402
  %t58 = xor i32 %t57, 409
  %t60 = add i32 %t58, 7
  %t61 = mul i32 %t60, 430
  %t62 = xor i32 %t61, 437
  %t64 = add i32 %t62, 7
  %t65 = mul i32 %t64, 458
  %t66 = xor i32 %t65, 465
  %t68 = add i32 %t66, 7
  %t69 = mul i32 %t68, 486
  %t70 = xor i32 %t69, 493
  %t72 = add i32 %t70, 7
  %t73 = mul i32 %t72, 514
  %t74 = xor i32 %t73, 521
  %t76 = add i32 %t74, 7
  %t77 = mul i32 %t76, 542
  %t78 = xor i32 %t77, 549
  %t80 = add i32 %t78, 7
  %t81 = mul i32 %t80, 570
  %t82 = xor i32 %t81, 577
  %t84 = add i32 %t82, 7
  %t85 = mul i32 %t84, 598
  %t86 = xor i32 %t85, 605
  %t88 = add i32 %t86, 7
  %t89 = mul i32 %t88, 626
  %t90 = xor i32 %t89, 633
  %t92 = add i32 %t90, 7
  %t93 = mul i32 %t92, 654
  %t94 = xor i32 %t93, 661
  %t96 = add i32 %t94, 7
  %t97 = mul i32 %t96, 682
  %t98 = xor i32 %t97, 689
  %t100 = add i32 %t98, 7
  %t101 = mul i32 %t100, 710
  %t102 = xor i32 %t101, 717
  %t104 = add i32 %t102, 7
  %t105 = mul i32 %t104, 738
  %t106 = xor i32 %t105, 745
  %t108 = add i32 %t106, 7
  %t109 = mul i32 %t108, 766
  ret i32 %t109
}

define internal i32 @never(i32 %x) noinline {
entry:
  %y = mul i32 %x, %x
  ret i32 %y
}

declare void @abort()

define i32 @main(i32 %argc) {
entry:
  %y.i = add i32 %argc, 1
  %y.i1 = add i32 %argc, 2
  %s.i = add i32 %y.i, %y.i1
  switch i32 %argc, label %other.i1 [
    i32 0, label %add.i1
    i32 1, label %mode_op.exit1
  ]

add.i1:                                           ; preds = %entry
  br label %mode_op.exit1

other.i1:                                         ; preds = %entry
  %m.i1 = mul i32 %y.i, %y.i1
  br label %mode_op.exit1

mode_op.exit1:                                    ; preds = %other.i1, %add.i1, %entry
  %d1 = phi i32 [ %s.i, %add.i1 ], [ %m.i1, %other.i1 ], [ -1, %entry ]
  %t0.i = add i32 %s.i, 3
  %t1.i = mul i32 %t0.i, 10
  %t2.i = xor i32 %t1.i, 17
  %t4.i = add i32 %t2.i, 7
  %t5.i = mul i32 %t4.i, 38
  %t6.i = xor i32 %t5.i, 45
  %t8.i = add i32 %t6.i, 7
  %t9.i = mul i32 %t8.i, 66
  %t10.i = xor i32 %t9.i, 73
  %t12.i = add i32 %t10.i, 7
  %t13.i = mul i32 %t12.i, 94
  %t14.i = xor i32 %t13.i, 101
  %t16.i = add i32 %t14.i, 7
  %t17.i = mul i32 %t16.i, 122
  %t18.i = xor i32 %t17.i, 129
  %t20.i = add i32 %t18.i, 7
  %t21.i = mul i32 %t20.i, 150
  %t22.i = xor i32 %t21.i, 157
  %t24.i = add i32 %t22.i, 7
  %t25.i = mul i32 %t24.i, 178
  %t26.i = xor i32 %t25.i, 185
  %t28.i = add i32 %t26.i, 7
  %t29.i = mul i32 %t28.i, 206
  %t30.i = xor i32 %t29.i, 213
  %t32.i = add i32 %t30.i, 7
  %t33.i = mul i32 %t32.i, 234
  %t34.i = xor i32 %t33.i, 241
  %t36.i = add i32 %t34.i, 7
  %t37.i = mul i32 %t36.i, 262
  %t38.i = xor i32 %t37.i, 269
  %t40.i = add i32 %t38.i, 7
  %t41.i = mul i32 %t40.i, 290
  %t42.i = xor i32 %t41.i, 297
  %t44.i = add i32 %t42.i, 7
  %t45.i = mul i32 %t44.i, 318
  %t46.i = xor i32 %t45.i, 325
  %t48.i = add i32 %t46.i, 7
  %t49.i = mul i32 %t48.i, 346
  %t50.i = xor i32 %t49.i, 353
  %t52.i = add i32 %t50.i, 7
  %t53.i = mul i32 %t52.i, 374
  %t54.i = xor i32 %t53.i, 381
  %t56.i = add i32 %t54.i, 7
  %t57.i = mul i32 %t56.i, 402
  %t58.i = xor i32 %t57.i, 409
  %t60.i = add i32 %t58.i, 7
  %t61.i = mul i32 %t60.i, 430
  %t62.i = xor i32 %t61.i, 437
  %t64.i = add i32 %t62.i, 7
  %t65.i = mul i32 %t64.i, 458
  %t66.i = xor i32 %t65.i, 465
  %t68.i = add i32 %t66.i, 7
  %t69.i = mul i32 %t68.i, 486
  %t70.i = xor i32 %t69.i, 493
  %t72.i = add i32 %t70.i, 7
  %t73.i = mul i32 %t72.i, 514
  %t74.i = xor i32 %t73.i, 521
  %t76.i = add i32 %t74.i, 7
  %t77.i = mul i32 %t76.i, 542
  %t78.i = xor i32 %t77.i, 549
  %t80.i = add i32 %t78.i, 7
  %t81.i = mul i32 %t80.i, 570
  %t82.i = xor i32 %t81.i, 577
  %t84.i = add i32 %t82.i, 7
  %t85.i = mul i32 %t84.i, 598
  %t86.i = xor i32 %t85.i, 605
  %t88.i = add i32 %t86.i, 7
  %t89.i = mul i32 %t88.i, 626
  %t90.i = xor i32 %t89.i, 633
  %t92.i = add i32 %t90.i, 7
  %t93.i = mul i32 %t92.i, 654
  %t94.i = xor i32 %t93.i, 661
  %t96.i = add i32 %t94.i, 7
  %t97.i = mul i32 %t96.i, 682
  %t98.i = xor i32 %t97.i, 689
  %t100.i = add i32 %t98.i, 7
  %t101.i = mul i32 %t100.i, 710
  %t102.i = xor i32 %t101.i, 717
  %t104.i = add i32 %t102.i, 7
  %t105.i = mul i32 %t104.i, 738
  %t106.i = xor i32 %t105.i, 745
  %t108.i = add i32 %t106.i, 7
  %t109.i = mul i32 %t108.i, 766
  %t0.i1 = add i32 %d1, 3
  %t1.i1 = mul i32 %t0.i1, 10
  %t2.i1 = xor i32 %t1.i1, 17
  %t4.i1 = add i32 %t2.i1, 7
  %t5.i1 = mul i32 %t4.i1, 38
  %t6.i1 = xor i32 %t5.i1, 45
  %t8.i1 = add i32 %t6.i1, 7
  %t9.i1 = mul i32 %t8.i1, 66
  %t10.i1 = xor i32 %t9.i1, 73
  %t12.i1 = add i32 %t10.i1, 7
  %t13.i1 = mul i32 %t12.i1, 94
  %t14.i1 = xor i32 %t13.i1, 101
  %t16.i1 = add i32 %t14.i1, 7
  %t17.i1 = mul i32 %t16.i1, 122
  %t18.i1 = xor i32 %t17.i1, 129
  %t20.i1 = add i32 %t18.i1, 7
  %t21.i1 = mul i32 %t20.i1, 150
  %t22.i1 = xor i32 %t21.i1, 157
  %t24.i1 = add i32 %t22.i1, 7
  %t25.i1 = mul i32 %t24.i1, 178
  %t26.i1 = xor i32 %t25.i1, 185
  %t28.i1 = add i32 %t26.i1, 7
  %t29.i1 = mul i32 %t28.i1, 206
  %t30.i1 = xor i32 %t29.i1, 213
  %t32.i1 = add i32 %t30.i1, 7
  %t33.i1 = mul i32 %t32.i1, 234
  %t34.i1 = xor i32 %t33.i1, 241
  %t36.i1 = add i32 %t34.i1, 7
  %t37.i1 = mul i32 %t36.i1, 262
  %t38.i1 = xor i32 %t37.i1, 269
  %t40.i1 = add i32 %t38.i1, 7
  %t41.i1 = mul i32 %t40.i1, 290
  %t42.i1 = xor i32 %t41.i1, 297
  %t44.i1 = add i32 %t42.i1, 7
  %t45.i1 = mul i32 %t44.i1, 318
  %t46.i1 = xor i32 %t45.i1, 325
  %t48.i1 = add i32 %t46.i1, 7
  %t49.i1 = mul i32 %t48.i1, 346
  %t50.i1 = xor i32 %t49.i1, 353
  %t52.i1 = add i32 %t50.i1, 7
  %t53.i1 = mul i32 %t52.i1, 374
  %t54.i1 = xor i32 %t53.i1, 381
  %t56.i1 = add i32 %t54.i1, 7
  %t57.i1 = mul i32 %t56.i1, 402
  %t58.i1 = xor i32 %t57.i1, 409
  %t60.i1 = add i32 %t58.i1, 7
  %t61.i1 = mul i32 %t60.i1, 430
  %t62.i1 = xor i32 %t61.i1, 437
  %t64.i1 = add i32 %t62.i1, 7
  %t65.i1 = mul i32 %t64.i1, 458
  %t66.i1 = xor i32 %t65.i1, 465
  %t68.i1 = add i32 %t66.i1, 7
  %t69.i1 = mul i32 %t68.i1, 486
  %t70.i1 = xor i32 %t69.i1, 493
  %t72.i1 = add i32 %t70.i1, 7
  %t73.i1 = mul i32 %t72.i1, 514
  %t74.i1 = xor i32 %t73.i1, 521
  %t76.i1 = add i32 %t74.i1, 7
  %t77.i1 = mul i32 %t76.i1, 542
  %t78.i1 = xor i32 %t77.i1, 549
  %t80.i1 = add i32 %t78.i1, 7
  %t81.i1 = mul i32 %t80.i1, 570
  %t82.i1 = xor i32 %t81.i1, 577
  %t84.i1 = add i32 %t82.i1, 7
  %t85.i1 = mul i32 %t84.i1, 598
  %t86.i1 = xor i32 %t85.i1, 605
  %t88.i1 = add i32 %t86.i1, 7
  %t89.i1 = mul i32 %t88.i1, 626
  %t90.i1 = xor i32 %t89.i1, 633
  %t92.i1 = add i32 %t90.i1, 7
  %t93.i1 = mul i32 %t92.i1, 654
  %t94.i1 = xor i32 %t93.i1, 661
  %t96.i1 = add i32 %t94.i1, 7
  %t97.i1 = mul i32 %t96.i1, 682
  %t98.i1 = xor i32 %t97.i1, 689
  %t100.i1 = add i32 %t98.i1, 7
  %t101.i1 = mul i32 %t100.i1, 710
  %t102.i1 = xor i32 %t101.i1, 717
  %t104.i1 = add i32 %t102.i1, 7
  %t105.i1 = mul i32 %t104.i1, 738
  %t106.i1 = xor i32 %t105.i1, 745
  %t108.i1 = add i32 %t106.i1, 7
  %t109.i1 = mul i32 %t108.i1, 766
  %t0.i2 = add i32 %t109.i, 3
  %t1.i2 = mul i32 %t0.i2, 10
  %t2.i2 = xor i32 %t1.i2, 17
  %t4.i2 = add i32 %t2.i2, 7
  %t5.i2 = mul i32 %t4.i2, 38
  %t6.i2 = xor i32 %t5.i2, 45
  %t8.i2 = add i32 %t6.i2, 7
  %t9.i2 = mul i32 %t8.i2, 66
  %t10.i2 = xor i32 %t9.i2, 73
  %t12.i2 = add i32 %t10.i2, 7
  %t13.i2 = mul i32 %t12.i2, 94
  %t14.i2 = xor i32 %t13.i2, 101
  %t16.i2 = add i32 %t14.i2, 7
  %t17.i2 = mul i32 %t16.i2, 122
  %t18.i2 = xor i32 %t17.i2, 129
  %t20.i2 = add i32 %t18.i2, 7
  %t21.i2 = mul i32 %t20.i2, 150
  %t22.i2 = xor i32 %t21.i2, 157
  %t24.i2 = add i32 %t22.i2, 7
  %t25.i2 = mul i32 %t24.i2, 178
  %t26.i2 = xor i32 %t25.i2, 185
  %t28.i2 = add i32 %t26.i2, 7
  %t29.i2 = mul i32 %t28.i2, 206
  %t30.i2 = xor i32 %t29.i2, 213
  %t32.i2 = add i32 %t30.i2, 7
  %t33.i2 = mul i32 %t32.i2, 234
  %t34.i2 = xor i32 %t33.i2, 241
  %t36.i2 = add i32 %t34.i2, 7
  %t37.i2 = mul i32 %t36.i2, 262
  %t38.i2 = xor i32 %t37.i2, 269
  %t40.i2 = add i32 %t38.i2, 7
  %t41.i2 = mul i32 %t40.i2, 290
  %t42.i2 = xor i32 %t41.i2, 297
  %t44.i2 = add i32 %t42.i2, 7
  %t45.i2 = mul i32 %t44.i2, 318
  %t46.i2 = xor i32 %t45.i2, 325
  %t48.i2 = add i32 %t46.i2, 7
  %t49.i2 = mul i32 %t48.i2, 346
  %t50.i2 = xor i32 %t49.i2, 353
  %t52.i2 = add i32 %t50.i2, 7
  %t53.i2 = mul i32 %t52.i2, 374
  %t54.i2 = xor i32 %t53.i2, 381
  %t56.i2 = add i32 %t54.i2, 7
  %t57.i2 = mul i32 %t56.i2, 402
  %t58.i2 = xor i32 %t57.i2, 409
  %t60.i2 = add i32 %t58.i2, 7
  %t61.i2 = mul i32 %t60.i2, 430
  %t62.i2 = xor i32 %t61.i2, 437
  %t64.i2 = add i32 %t62.i2, 7
  %t65.i2 = mul i32 %t64.i2, 458
  %t66.i2 = xor i32 %t65.i2, 465
  %t68.i2 = add i32 %t66.i2, 7
  %t69.i2 = mul i32 %t68.i2, 486
  %t70.i2 = xor i32 %t69.i2, 493
  %t72.i2 = add i32 %t70.i2, 7
  %t73.i2 = mul i32 %t72.i2, 514
  %t74.i2 = xor i32 %t73.i2, 521
  %t76.i2 = add i32 %t74.i2, 7
  %t77.i2 = mul i32 %t76.i2, 542
  %t78.i2 = xor i32 %t77.i2, 549
  %t80.i2 = add i32 %t78.i2, 7
  %t81.i2 = mul i32 %t80.i2, 570
  %t82.i2 = xor i32 %t81.i2, 577
  %t84.i2 = add i32 %t82.i2, 7
  %t85.i2 = mul i32 %t84.i2, 598
  %t86.i2 = xor i32 %t85.i2, 605
  %t88.i2 = add i32 %t86.i2, 7
  %t89.i2 = mul i32 %t88.i2, 626
  %t90.i2 = xor i32 %t89.i2, 633
  %t92.i2 = add i32 %t90.i2, 7
  %t93.i2 = mul i32 %t92.i2, 654
  %t94.i2 = xor i32 %t93.i2, 661
  %t96.i2 = add i32 %t94.i2, 7
  %t97.i2 = mul i32 %t96.i2, 682
  %t98.i2 = xor i32 %t97.i2, 689
  %t100.i2 = add i32 %t98.i2, 7
  %t101.i2 = mul i32 %t100.i2, 710
  %t102.i2 = xor i32 %t101.i2, 717
  %t104.i2 = add i32 %t102.i2, 7
  %t105.i2 = mul i32 %t104.i2, 738
  %t106.i2 = xor i32 %t105.i2, 745
  %t108.i2 = add i32 %t106.i2, 7
  %t109.i2 = mul i32 %t108.i2, 766
  %h = tail call i32 @never(i32 %t109.i1)
  %0 = add i32 %h, %t109.i
  %1 = add i32 %0, %t109.i1
  %sum = add i32 %1, %t109.i2
  %bad = icmp eq i32 %sum, 12345
  br i1 %bad, label %fail, label %ok

fail:                                             ; preds = %mode_op.exit1
  %z = tail call i32 @big(i32 %sum)
  tail call void @abort()
  unreachable

ok:                                               ; preds = %mode_op.exit1
  ret i32 %sum
}
