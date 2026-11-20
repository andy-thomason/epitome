; epitome-opt: --no-inline --recursion=stack
; Each activation has its own local array, and its address is live across
; the recursive call. The alloca moves into the loop with the old entry
; block, so every activation allocates afresh; popping a frame restores
; the stack pointer, which frees the finished activation's array too.
declare void @llvm.memset.p0.i64(ptr, i8, i64, i1)
declare void @llvm.lifetime.start.p0(i64, ptr)
declare void @llvm.lifetime.end.p0(i64, ptr)

@seed = global i32 1

; Writes the array's second element from memory the optimiser cannot know.
define void @fill(ptr %p) noinline {
  %v = load volatile i32, ptr @seed
  %at = getelementptr i32, ptr %p, i64 1
  store i32 %v, ptr %at
  ret void
}

define i32 @walk(i32 %n) {
entry:
  %buf = alloca [4 x i32]
  call void @llvm.memset.p0.i64(ptr %buf, i8 0, i64 16, i1 false)
  store i32 %n, ptr %buf
  %stop = icmp eq i32 %n, 0
  br i1 %stop, label %done, label %recur

recur:
  %m = sub i32 %n, 1
  %inner = call i32 @walk(i32 %m)
  %mine = load i32, ptr %buf
  %sum = add i32 %inner, %mine
  ret i32 %sum

done:
  ret i32 0
}

; The same with lifetime markers, and the array passed to a function, so
; it stays in memory. After the recursive call returns, the array's
; address is a reload from the frame, and where the returns meet, a phi of
; several activations' arrays: neither may be a marker's pointer (the
; LangRef allows only an alloca), so the markers that would name them go.
define i32 @walk_marked(i32 %n) {
entry:
  %buf = alloca [4 x i32]
  call void @llvm.lifetime.start.p0(i64 16, ptr %buf)
  store i32 %n, ptr %buf
  call void @fill(ptr %buf)
  %stop = icmp eq i32 %n, 0
  br i1 %stop, label %done, label %recur

recur:
  %m = sub i32 %n, 1
  %inner = call i32 @walk_marked(i32 %m)
  %mine = load i32, ptr %buf
  %sum = add i32 %inner, %mine
  call void @llvm.lifetime.end.p0(i64 16, ptr %buf)
  ret i32 %sum

done:
  call void @llvm.lifetime.end.p0(i64 16, ptr %buf)
  ret i32 0
}

define i32 @main() {
entry:
  %x = call i32 @walk(i32 10)
  %y = call i32 @walk_marked(i32 10)
  %r = add i32 %x, %y
  ret i32 %r
}
