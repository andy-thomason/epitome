; @style: combine: sub_const x1
; @widen: combine: (folded) x1, br_true x1
; @widen: simplifycfg: 1 unreachable blocks removed, 0 empty blocks bypassed, 0 selects made, 1 blocks merged
; @hook: not inlined @report: noinline
; @hook: not inlined @report: noinline
; @hook: ipo: 1 call results are the constant their callee always returns
; @hook: ipo: inlined leaf @widen
; @report: not inlined @verbose: noinline
; @report: ipo: 1 pointer parameters promoted into 1 loaded values
; @report: ipo: 1 parameters are the constant every caller passes
; @kept: not inlined @peek: noinline
; @main: not inlined @hook: callee is part of a call-graph cycle
; @main: inlined @kept: cost 8, bonus 18, threshold 100
; @kept: deleted, no longer used
; @widen: deleted, no longer used
; @verbose: deleted, no longer used


@sink = global ptr null
@count = global i32 0

define internal i8 @style(i32 %n) {
entry:
  %big = icmp sgt i32 %n, 1000
  br i1 %big, label %again, label %done

again:                                            ; preds = %entry
  %m = add i32 %n, -1
  tail call void @hook(i32 %m)
  br label %done

done:                                             ; preds = %again, %entry
  ret i8 3
}

define internal void @hook(i32 %n) {
entry:
  %s = call i8 @style(i32 %n)
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
