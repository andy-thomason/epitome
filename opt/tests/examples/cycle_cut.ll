; epitome-opt: -O2 --print-cycles --recursion=off
; A cycle that stays a cycle: @parse -> @item -> @fail -> @parse, where
; @fail is the panic path (it calls a function that does not return, then
; reports through @parse). Both calls on the panic path are cold; the cut
; is @fail's call back to @parse, which stays a call. The rest is ordered
; callees first: @item inlines @digit and @fail, and @parse inlines @item,
; bringing the cut call along, still a call. (Recursion is off, so that
; the cycle is not collapsed first.)
@out = global i32 0
declare void @abort() noreturn
define internal i32 @digit(i32 %c) {
  %d = sub i32 %c, 48
  ret i32 %d
}
define internal i32 @item(i32 %c) {
entry:
  %ok = icmp ult i32 %c, 58
  br i1 %ok, label %good, label %bad
good:
  %d = call i32 @digit(i32 %c)
  ret i32 %d
bad:
  call void @fail(i32 %c)
  unreachable
}
define internal void @fail(i32 %c) {
  %v = call i32 @parse(i32 0, i32 %c)
  store volatile i32 %v, ptr @out
  call void @abort()
  unreachable
}
define i32 @parse(i32 %a, i32 %b) {
  %x = call i32 @item(i32 %a)
  %y = call i32 @item(i32 %b)
  %r = add i32 %x, %y
  ret i32 %r
}
define i32 @main() {
  %r = call i32 @parse(i32 50, i32 51)
  ret i32 %r
}
