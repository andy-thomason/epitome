; epitome-opt: --passes=combine
; A sign extension wider than 64 bits, folded: the sign goes on past bit
; 63 (it was zero there, making 2^64 - 5 of -5).
define i128 @sext_i8(i8 %x) {
  %b = sext i8 -5 to i128
  ret i128 %b
}

define i128 @sext_neg() {
  %a = sub i64 0, 123456789
  %b = sext i64 %a to i128
  ret i128 %b
}

define i128 @sext_pos() {
  %b = sext i32 7 to i128
  ret i128 %b
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
