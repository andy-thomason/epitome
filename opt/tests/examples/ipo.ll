; epitome-opt: --recursion=off
; After the bottom-up passes, what they cannot see (ipo.rs). @hook,
; @style and @widen call each other, in a cycle left alone here as one
; too big to collapse would be, so bottom-up neither learns that @style
; always returns 3 nor inlines @widen once its check folds and it calls
; nothing. @hook passes a pointer to its frame to @report, which
; only loads from it: the pointer becomes the loaded byte, both calls pass
; 3, and @report's other arm goes, and @verbose with it. @kept's pointer
; is also stored where others can see it, so @peek keeps its pointer.

@sink = global ptr null
@count = global i32 0

define internal i8 @style(i32 %n) {
entry:
  %big = icmp sgt i32 %n, 1000
  br i1 %big, label %again, label %done

again:
  %m = sub i32 %n, 1
  call void @hook(i32 %m)
  br label %done

done:
  ret i8 3
}

define internal i64 @widen(i16 %x) {
entry:
  %ok = icmp ule i128 65535, 18446744073709551615
  br i1 %ok, label %fine, label %bad

bad:
  call void @hook(i32 0)
  unreachable

fine:
  %w = zext i16 %x to i64
  ret i64 %w
}

define internal void @hook(i32 %n) {
entry:
  %slot = alloca i8
  %s = call i8 @style(i32 %n)
  store i8 %s, ptr %slot
  %n16 = trunc i32 %n to i16
  %w = call i64 @widen(i16 %n16)
  %w32 = trunc i64 %w to i32
  %c = load i32, ptr @count
  %c1 = add i32 %c, %w32
  store i32 %c1, ptr @count
  call void @report(ptr %slot)
  call void @report(ptr %slot)
  ret void
}

define internal void @report(ptr %p) noinline {
entry:
  %v = load i8, ptr %p
  %short = icmp eq i8 %v, 3
  br i1 %short, label %quiet, label %loud

quiet:
  %c = load i32, ptr @count
  %c1 = add i32 %c, 1
  store i32 %c1, ptr @count
  ret void

loud:
  call void @verbose()
  ret void
}

define internal void @verbose() noinline {
  %c = load i32, ptr @count
  %c1 = add i32 %c, 100
  store i32 %c1, ptr @count
  ret void
}

define internal i32 @peek(ptr %p) noinline {
  %v = load i32, ptr %p
  ret i32 %v
}

define internal i32 @kept() {
  %x = alloca i32
  store i32 7, ptr %x
  store ptr %x, ptr @sink
  %v = call i32 @peek(ptr %x)
  ret i32 %v
}

define i32 @main() {
  call void @hook(i32 5)
  %k = call i32 @kept()
  %c = load i32, ptr @count
  %r = add i32 %c, %k
  ret i32 %r
}
