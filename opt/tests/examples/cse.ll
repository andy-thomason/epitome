; epitome-opt: --passes=cse
; Common-subexpression elimination. Each function shows one rule, and
; the last ones show what must not be merged.

declare void @write(ptr)
declare i32 @pure(i32) memory(none)

; The same computation twice, and through a chain: once %b is %a, %d is %c.
define i32 @repeat(i32 %x, i32 %y) {
  %a = mul i32 %x, %y
  %b = mul i32 %x, %y
  %c = add i32 %a, 1
  %d = add i32 %b, 1
  %r = xor i32 %c, %d
  ret i32 %r
}

; Commutative operands and swapped comparisons have the same key.
define i1 @canonical(i32 %x, i32 %y) {
  %a = add i32 %x, %y
  %b = add i32 %y, %x
  %c = icmp slt i32 %a, %x
  %d = icmp sgt i32 %x, %b
  %r = and i1 %c, %d
  ret i1 %r
}

; The instruction kept loses flags the other did not have.
define i32 @flags(i32 %x, i32 %y) {
  %a = add nsw nuw i32 %x, %y
  %b = add nuw i32 %x, %y
  %r = sub i32 %a, %b
  ret i32 %r
}

; A dominating computation is reused in a later block; one in a sibling
; branch is not visible.
define i32 @dominance(i32 %x, i1 %c) {
entry:
  %a = shl i32 %x, 2
  br i1 %c, label %then, label %else
then:
  %b = shl i32 %x, 2
  %t = mul i32 %x, 3
  br label %join
else:
  %e = mul i32 %x, 3
  br label %join
join:
  %p = phi i32 [ %t, %then ], [ %e, %else ]
  %q = phi i32 [ %t, %then ], [ %e, %else ]
  %s = add i32 %p, %q
  %r = add i32 %s, %b
  ret i32 %r
}

; Loads: a second load of the same address is the first; a load after a
; store gets the stored value; a call that may write ends both.
define i32 @memory(ptr %p, i32 %v) {
  %a = load i32, ptr %p
  %b = load i32, ptr %p
  store i32 %v, ptr %p
  %c = load i32, ptr %p
  call void @write(ptr %p)
  %d = load i32, ptr %p
  %e = call i32 @pure(i32 %d)
  %f = load i32, ptr %p
  %s1 = add i32 %a, %b
  %s2 = add i32 %c, %d
  %s3 = add i32 %e, %f
  %s4 = add i32 %s1, %s2
  %r = add i32 %s4, %s3
  ret i32 %r
}

; A loop: the header has another way in, so loads before the loop are not
; reused inside it.
define i32 @loop(ptr %p, i32 %n) {
entry:
  %before = load i32, ptr %p
  br label %head
head:
  %i = phi i32 [ 0, %entry ], [ %next, %head ]
  %inside = load i32, ptr %p
  store i32 %i, ptr %p
  %next = add i32 %i, 1
  %more = icmp slt i32 %next, %n
  br i1 %more, label %head, label %done
done:
  %r = add i32 %before, %inside
  ret i32 %r
}

; Never merged: two freezes may give different values, and volatile
; loads must each happen.
define i32 @not_merged(i32 %x, ptr %p) {
  %a = freeze i32 %x
  %b = freeze i32 %x
  %c = load volatile i32, ptr %p
  %d = load volatile i32, ptr %p
  %s = add i32 %a, %b
  %t = add i32 %c, %d
  %r = add i32 %s, %t
  ret i32 %r
}

; With alias analysis: a store to a different object, or a call that cannot
; reach a local whose address never escapes, does not end what is known.
@counter = global i32 0

define i32 @aliasing(ptr %p, i32 %v) {
  %local = alloca i32
  %pair = alloca [2 x i32]
  %second = getelementptr inbounds [2 x i32], ptr %pair, i64 0, i64 1
  store i32 %v, ptr %local
  store i32 1, ptr %pair
  store i32 2, ptr %second
  store i32 3, ptr @counter
  call void @write(ptr %p)
  %a = load i32, ptr %local
  %b = load i32, ptr %pair
  %c = load i32, ptr %second
  %d = load i32, ptr @counter
  %s1 = add i32 %a, %b
  %s2 = add i32 %c, %d
  %r = add i32 %s1, %s2
  ret i32 %r
}

; A load of the bytes a store just wrote as another type: a pointer read
; as an integer, an integer as a pointer, and the low byte of an i64 (on
; this little-endian machine), each a cast of the stored value.
define i64 @forward_cast(ptr %slot, ptr %p, i64 %n) {
  store ptr %p, ptr %slot
  %bits = load i64, ptr %slot
  %other = getelementptr i8, ptr %slot, i64 8
  store i64 %n, ptr %other
  %low = load i8, ptr %other
  %back = load ptr, ptr %other
  %w = zext i8 %low to i64
  %b = ptrtoint ptr %back to i64
  %s = add i64 %bits, %w
  %r = add i64 %s, %b
  ret i64 %r
}
