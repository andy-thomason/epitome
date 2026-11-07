; @sext_i8: combine: (folded) x1
; @sext_neg: combine: (folded) x2
; @sext_pos: combine: (folded) x1


define i128 @sext_i8(i8 %x) {
  ret i128 -5
}

define i128 @sext_neg() {
  ret i128 -123456789
}

define i128 @sext_pos() {
  ret i128 7
}

define i32 @main() {
  %a = call i128 @sext_i8(i8 0)
  %b = call i128 @sext_neg()
  %c = call i128 @sext_pos()
  %s = add i128 %a, %b
  %t = add i128 %s, %c
  %hi = lshr i128 %t, 64
  %h = trunc i128 %hi to i32
  %l = trunc i128 %t to i32
  %ok1 = icmp eq i32 %h, -1
  %ok2 = icmp eq i32 %l, -123456787
  %ok = and i1 %ok1, %ok2
  %r = select i1 %ok, i32 0, i32 1
  ret i32 %r
}
