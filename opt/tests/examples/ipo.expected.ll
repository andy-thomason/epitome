; @style: combine: sub_const x1
; @style: implied: 0 comparisons folded, 0 selects resolved, 1 flags set
; @style: recursion: the cycle splits into pieces of 1, 2
; @style: recursion: cut 1 calls (0 cold) and 0 self-calls to order the cycle of 2
; @widen: combine: (folded) x1, br_true x1
; @widen: simplifycfg: 1 unreachable blocks removed, 0 empty blocks bypassed, 0 selects made, 1 blocks merged, 0 edges into unreachable dropped
; @hook: not inlined @report: noinline
; @hook: not inlined @report: noinline
; @hook: inlined @widen: cost 1, bonus 12, threshold 100
; @hook: not inlined @report: noinline
; @hook: not inlined @report: noinline
; @hook: combine: (dead) x1, trunc_zext_narrow x1
; @hook: inlined @style: cost 9, bonus 20, threshold 100
; @hook: not inlined @report: noinline
; @hook: not inlined @report: noinline
; @report: not inlined @verbose: noinline
; @report: ipo: 1 pointer parameters promoted into 1 loaded values
; @report: ipo: 1 parameters are the constant every caller passes
; @kept: not inlined @peek: noinline
; @main: not inlined @hook: callee is part of a call-graph cycle
; @main: inlined @kept: cost 8, bonus 18, threshold 100
; @style: deleted, no longer used
; @widen: deleted, no longer used
; @kept: deleted, no longer used
; @verbose: deleted, no longer used


@sink = global ptr null
@count = global i32 0

define internal void @hook(i32 %n) {
entry:
  %big.i = icmp sgt i32 %n, 1000
  br i1 %big.i, label %again.i, label %done.i

again.i:                                          ; preds = %entry
  %m.i = add nsw i32 %n, -1
  call void @hook(i32 %m.i)
  br label %done.i

done.i:                                           ; preds = %again.i, %entry
  %n16 = trunc i32 %n to i16
  %w32 = zext i16 %n16 to i32
  %c = load i32, ptr @count
  %c1 = add i32 %c, %w32
  store i32 %c1, ptr @count
  call void @report(i8 3)
  call void @report(i8 3)
  ret void
}

define internal void @report(i8 %0) noinline {
entry:
  %c = load i32, ptr @count
  %c1 = add i32 %c, 1
  store i32 %c1, ptr @count
  ret void
}

define internal i32 @peek(ptr %p) noinline {
  %v = load i32, ptr %p
  ret i32 %v
}

define i32 @main() {
  %x.i = alloca i32
  call void @hook(i32 5)
  store i32 7, ptr %x.i
  store ptr %x.i, ptr @sink
  %v.i = call i32 @peek(ptr %x.i)
  %c = load i32, ptr @count
  %r = add i32 %c, %v.i
  ret i32 %r
}
