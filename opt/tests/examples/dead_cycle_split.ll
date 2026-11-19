; epitome-opt: -O2 --print-cycles --recursion=off
; A cycle closed only by a call that a constant makes dead, the way
; `unreachable_unchecked`'s `ub_checks()` call to `precondition_check`
; closes std's formatting and panic machinery into one SCC:
; @write -> @unwrap -> @hint -> @fail -> @write. The call graph is built
; before @checks is known to return 0, so the four are one cycle and
; nothing among them is inlined at first. Once @hint is simplified, the
; call to @fail is gone, the cycle splits, and each piece is finished in
; turn: @unwrap inlines @hint (only `unreachable`, so the `none` arm
; goes), and @write inlines @unwrap. (Recursion is off, so that the cycle
; is not collapsed into @write first.)
@out = global i32 0
define internal i8 @checks() {
  ret i8 0
}
define internal void @fail() {
  %v = call i32 @write(i32 1, i1 false)
  store volatile i32 %v, ptr @out
  ret void
}
define internal void @hint() {
entry:
  %c = call i8 @checks()
  %b = icmp ne i8 %c, 0
  br i1 %b, label %check, label %done
check:
  call void @fail()
  br label %done
done:
  unreachable
}
define internal i32 @unwrap(i32 %x, i1 %some) {
entry:
  br i1 %some, label %ok, label %none
ok:
  ret i32 %x
none:
  call void @hint()
  unreachable
}
define i32 @write(i32 %x, i1 %s) {
  %a = call i32 @unwrap(i32 %x, i1 %s)
  %b = call i32 @unwrap(i32 %a, i1 true)
  %r = add i32 %a, %b
  ret i32 %r
}
define i32 @main() {
  %r = call i32 @write(i32 5, i1 true)
  ret i32 %r
}
