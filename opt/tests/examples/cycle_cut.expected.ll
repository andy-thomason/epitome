; cycle: @item -> @fail -> @parse -> @item   (SCC of 3: @item, @fail, @parse)
; @digit: combine: sub_const x1
; @item: inlined @digit: cost 1, bonus 12, threshold 100
; @item: implied: 0 comparisons folded, 0 selects resolved, 1 flags set
; @item: recursion: cut 1 calls (1 cold) and 0 self-calls to order the cycle of 3
; @item: inlined @fail: cost 13, bonus 24, threshold 0
; @item: simplifycfg: 1 unreachable blocks removed, 0 empty blocks bypassed, 0 selects made, 0 blocks merged, 0 edges into unreachable dropped
; @parse: inlined @item: cost 16, bonus 6, threshold 100
; @parse: inlined @item: cost 16, bonus 6, threshold 100
; @parse: combine: (dead) x2, (linear) x1
; @parse: implied: 0 comparisons folded, 0 selects resolved, 2 flags set
; @main: not inlined @parse: callee is part of a call-graph cycle
; @main: constcall: 1 calls worked out
; @digit: deleted, no longer used
; @item: deleted, no longer used
; @fail: deleted, no longer used


@out = global i32 0

declare void @abort() noreturn

define i32 @parse(i32 %a, i32 %b) {
  %ok.i = icmp ult i32 %a, 58
  br i1 %ok.i, label %good.i, label %bad.i

good.i:                                           ; preds = %0
  %ok.i1 = icmp ult i32 %b, 58
  br i1 %ok.i1, label %good.i1, label %bad.i1

bad.i:                                            ; preds = %0
  %v.i.i = tail call i32 @parse(i32 0, i32 %a)
  store volatile i32 %v.i.i, ptr @out
  tail call void @abort()
  unreachable

good.i1:                                          ; preds = %good.i
  %1 = add nuw nsw i32 %a, %b
  %r = add nsw i32 %1, -96
  ret i32 %r

bad.i1:                                           ; preds = %good.i
  %v.i.i1 = tail call i32 @parse(i32 0, i32 %b)
  store volatile i32 %v.i.i1, ptr @out
  tail call void @abort()
  unreachable
}

define i32 @main() {
  ret i32 5
}
