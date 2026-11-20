; epitome-opt: --passes=combine,simplifycfg
; Edges into blocks that are only `unreachable` cannot be taken: a
; conditional branch to one goes to its other target, and a switch loses
; the cases that lead there (`Option::unwrap_unchecked` becomes a plain
; move). A block that calls a function that does not return first is a
; real path, and stays.
declare void @panic() noreturn

; unwrap_unchecked: no compare, no trap.
define i32 @unwrap_unchecked(i8 %tag, i32 %x) {
entry:
  %some = icmp eq i8 %tag, 1
  br i1 %some, label %ok, label %none
ok:
  ret i32 %x
none:
  unreachable
}

; The other way round, and through an empty block.
define i32 @flipped(i1 %c, i32 %x) {
entry:
  br i1 %c, label %dead, label %live
dead:
  br label %trap
trap:
  unreachable
live:
  ret i32 %x
}

; A switch whose cases and default lead to `unreachable`: the default
; becomes the first live case's target, whose cases go.
define i32 @switched(i32 %v) {
entry:
  switch i32 %v, label %bad [
    i32 0, label %a
    i32 1, label %bad
    i32 2, label %b
    i32 3, label %a
  ]
a:
  %p = phi i32 [ 10, %entry ], [ 10, %entry ]
  ret i32 %p
b:
  ret i32 20
bad:
  unreachable
}

; Every way out is unreachable, so the block is too, and so its branch.
define i32 @nowhere(i1 %c, i1 %d, i32 %x) {
entry:
  br i1 %d, label %mid, label %fine
mid:
  br i1 %c, label %u1, label %u2
u1:
  unreachable
u2:
  unreachable
fine:
  ret i32 %x
}

; A call that does not return is a real path: the branch stays.
define i32 @panics(i1 %c, i32 %x) {
entry:
  br i1 %c, label %ok, label %fail
ok:
  ret i32 %x
fail:
  call void @panic()
  unreachable
}
