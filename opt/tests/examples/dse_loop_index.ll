; A store to a[i] in a loop, read the next time round as a[i - 1]. The
; same SSA value names both indices, but the phi that makes it moves it on
; each time round, so the load reads what the store wrote: the store must
; stay. (Driftsort's run stack, `desired_depths[stack_len - 1]`.)
define i8 @last_depths(i64 %n) {
entry:
  %depth = alloca [8 x i8]
  br label %loop

loop:
  %len = phi i64 [ 0, %entry ], [ %next, %write ]
  %acc = phi i8 [ 0, %entry ], [ %sum, %write ]
  %more = icmp ult i64 %len, %n
  br i1 %more, label %body, label %done

body:
  %has = icmp ugt i64 %len, 0
  br i1 %has, label %read, label %write

read:
  %prev = add nsw i64 %len, -1
  %pp = getelementptr i8, ptr %depth, i64 %prev
  %d = load i8, ptr %pp
  br label %write

write:
  %got = phi i8 [ 0, %body ], [ %d, %read ]
  %sum = add i8 %acc, %got
  %v = trunc i64 %len to i8
  %p = getelementptr i8, ptr %depth, i64 %len
  store i8 %v, ptr %p
  %next = add i64 %len, 1
  br label %loop

done:
  ret i8 %acc
}

define i32 @main() {
  %r = call i8 @last_depths(i64 5)
  %w = zext i8 %r to i32
  ret i32 %w
}
