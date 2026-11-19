; epitome-opt: --passes=combine
; Compares of constants wider than 64 bits, folded: `u16::MAX as u128 <=
; usize::MAX as u128` from `<usize as From<u16>>::from`, signed and
; unsigned orders of negative values, and words above the lowest.
define i1 @from_u16() {
  %r = icmp ule i128 65535, 18446744073709551615
  ret i1 %r
}

define i1 @slt_negative() {
  %r = icmp slt i128 -1, 1
  ret i1 %r
}

define i1 @ugt_negative() {
  %r = icmp ugt i128 -1, 1
  ret i1 %r
}

define i1 @eq_high_word() {
  %r = icmp eq i128 18446744073709551616, 0
  ret i1 %r
}

define i1 @sge_three_words() {
  %r = icmp sge i192 -3138550867693340381917894711603833208051177722232017256448, 0
  ret i1 %r
}

define i32 @main() {
  %a = call i1 @from_u16()
  %b = call i1 @slt_negative()
  %c = call i1 @ugt_negative()
  %d = call i1 @eq_high_word()
  %e = call i1 @sge_three_words()
  %a1 = zext i1 %a to i32
  %b1 = zext i1 %b to i32
  %c1 = zext i1 %c to i32
  %d1 = zext i1 %d to i32
  %e1 = zext i1 %e to i32
  %b2 = shl i32 %b1, 1
  %c2 = shl i32 %c1, 2
  %d2 = shl i32 %d1, 3
  %e2 = shl i32 %e1, 4
  %s1 = or i32 %a1, %b2
  %s2 = or i32 %s1, %c2
  %s3 = or i32 %s2, %d2
  %s4 = or i32 %s3, %e2
  ret i32 %s4
}
