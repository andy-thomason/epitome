; @f: recursion: 1 accumulator call(s) now jump back to the start, folding with add
; @f: combine: (folded) x1, add_zero x1, sub_const x1
; @f: ipo: 1 parameters are the constant every caller passes


@fmt = private constant [4 x i8] c"%d\0A\00"

declare i32 @printf(ptr, ...)

define internal i32 @f(i32 %n.arg) {
entry.rec:
  br label %entry

entry:                                            ; preds = %rec, %entry.rec
  %n = phi i32 [ 100, %entry.rec ], [ %a, %rec ]
  %acc = phi i32 [ 0, %entry.rec ], [ %s, %rec ]
  switch i32 %n, label %rec [
    i32 0, label %done
    i32 1, label %done
  ]

rec:                                              ; preds = %entry
  %a = add i32 %n, -1
  %s = add i32 %acc, %n
  br label %entry

done:                                             ; preds = %entry
  ret i32 %acc
}

define i32 @main() {
  %v = tail call i32 @f(i32 100)
  %1 = tail call i32 (ptr, ...) @printf(ptr @fmt, i32 %v)
  ret i32 0
}
