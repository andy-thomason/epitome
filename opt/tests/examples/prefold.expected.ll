; @len: prefold: 1 branches on constants folded, 1 blocks removed
; @len: combine: (dead) x1
; @len: simplifycfg: 0 unreachable blocks removed, 0 empty blocks bypassed, 0 selects made, 1 blocks merged, 0 edges into unreachable dropped
; @report: prefold: 1 branches on constants folded, 2 blocks removed
; @report: inlined @len: cost 1, bonus 12, threshold 100
; @main: inlined @report: cost 2, bonus 15, threshold 100
; @main: combine: (folded) x1
; @len: deleted, no longer used
; @report: deleted, no longer used


@out = global i32 0

define i32 @main() {
  store volatile i32 44, ptr @out
  %v = load volatile i32, ptr @out
  ret i32 %v
}
