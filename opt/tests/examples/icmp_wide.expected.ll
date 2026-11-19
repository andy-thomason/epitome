; @from_u16: combine: (folded) x1
; @slt_negative: combine: (folded) x1
; @ugt_negative: combine: (folded) x1
; @eq_high_word: combine: (folded) x1
; @sge_three_words: combine: (folded) x1
; @main: constcall: 5 calls worked out
; @main: combine: (folded) x13


define i1 @from_u16() {
  ret i1 true
}

define i1 @slt_negative() {
  ret i1 true
}

define i1 @ugt_negative() {
  ret i1 true
}

define i1 @eq_high_word() {
  ret i1 false
}

define i1 @sge_three_words() {
  ret i1 false
}

define i32 @main() {
  ret i32 7
}
