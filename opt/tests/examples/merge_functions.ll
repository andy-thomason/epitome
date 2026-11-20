; epitome-opt: --passes=merge
; Identical functions are kept once. @twice_u64 and @twice_usize are the
; same up to their locals' names: the second becomes an alias of the
; first, and then unused, is deleted. @show_a and @show_b differ only in
; which of those they call, so they merge in the next round, and the
; vtables naming them after that. @ext_b is external, so it stays as an
; alias. @sig_b's address is significant (no unnamed_addr) and is
; stored, so it becomes a call of @sig_a; @local_b's is significant
; outside the module (local_unnamed_addr), so the same. @rec_a and
; @rec_b call themselves, which is the same in each. @other differs in a
; constant, and stays. The two strings are one constant.
target datalayout = "e-m:o-i64:64-i128:128-n32:64-S128"

@s1 = private unnamed_addr constant [3 x i8] c"hi\00", align 1
@s2 = private unnamed_addr constant [3 x i8] c"hi\00", align 1
@vt_a = private unnamed_addr constant [2 x ptr] [ptr @show_a, ptr @s1], align 8
@vt_b = private unnamed_addr constant [2 x ptr] [ptr @show_b, ptr @s2], align 8
@slot = global ptr @sig_b

define internal i64 @twice_u64(i64 %x) unnamed_addr noinline {
start:
  %y = shl i64 %x, 1
  %z = add i64 %y, 3
  ret i64 %z
}

define internal i64 @twice_usize(i64 %n) unnamed_addr noinline {
bb0:
  %a = shl i64 %n, 1
  %b = add i64 %a, 3
  ret i64 %b
}

define internal i64 @other(i64 %n) unnamed_addr noinline {
  %a = shl i64 %n, 1
  %b = add i64 %a, 4
  ret i64 %b
}

define internal i64 @show_a(i64 %x) unnamed_addr noinline {
  %t = call i64 @twice_u64(i64 %x)
  %u = mul i64 %t, %x
  ret i64 %u
}

define internal i64 @show_b(i64 %x) unnamed_addr noinline {
  %t = call i64 @twice_usize(i64 %x)
  %u = mul i64 %t, %x
  ret i64 %u
}

define i64 @ext_a(i64 %x) unnamed_addr noinline {
  %a = xor i64 %x, 7
  %b = mul i64 %a, %a
  ret i64 %b
}

define i64 @ext_b(i64 %x) unnamed_addr noinline {
  %a = xor i64 %x, 7
  %b = mul i64 %a, %a
  ret i64 %b
}

define internal i64 @sig_a(i64 %x) noinline {
  %a = sub i64 %x, 9
  %b = mul i64 %a, 5
  ret i64 %b
}

define internal i64 @sig_b(i64 %x) noinline {
  %a = sub i64 %x, 9
  %b = mul i64 %a, 5
  ret i64 %b
}

define i64 @local_a(i64 %x) local_unnamed_addr noinline {
  %a = or i64 %x, 1
  %b = udiv i64 %a, 3
  ret i64 %b
}

define i64 @local_b(i64 %x) local_unnamed_addr noinline {
  %a = or i64 %x, 1
  %b = udiv i64 %a, 3
  ret i64 %b
}

define internal i64 @rec_a(i64 %n) unnamed_addr noinline {
entry:
  %z = icmp eq i64 %n, 0
  br i1 %z, label %done, label %more
more:
  %m = sub i64 %n, 1
  %r = call i64 @rec_a(i64 %m)
  %s = add i64 %r, %n
  ret i64 %s
done:
  ret i64 0
}

define internal i64 @rec_b(i64 %k) unnamed_addr noinline {
entry:
  %z = icmp eq i64 %k, 0
  br i1 %z, label %done, label %more
more:
  %m = sub i64 %k, 1
  %r = call i64 @rec_b(i64 %m)
  %s = add i64 %r, %k
  ret i64 %s
done:
  ret i64 0
}

define i32 @main() {
  %f = load ptr, ptr getelementptr (i8, ptr @vt_a, i64 0)
  %g = load ptr, ptr getelementptr (i8, ptr @vt_b, i64 0)
  %h = load ptr, ptr @slot
  %v1 = call i64 %f(i64 3)
  %v2 = call i64 %g(i64 4)
  %v3 = call i64 %h(i64 20)
  %v4 = call i64 @ext_b(i64 2)
  %v5 = call i64 @local_b(i64 10)
  %v6 = call i64 @rec_b(i64 4)
  %v7a = call i64 @other(i64 1)
  %v7b = call i64 @rec_a(i64 3)
  %v7 = add i64 %v7a, %v7b
  %v8 = call i64 @sig_a(i64 11)
  %c1 = load i8, ptr @s1
  %c2 = load i8, ptr getelementptr (i8, ptr @s2, i64 1)
  %w1 = add i64 %v1, %v2
  %w2 = add i64 %w1, %v3
  %w3 = add i64 %w2, %v4
  %w4 = add i64 %w3, %v5
  %w5 = add i64 %w4, %v6
  %w6 = add i64 %w5, %v7
  %w7 = add i64 %w6, %v8
  %e1 = zext i8 %c1 to i64
  %e2 = zext i8 %c2 to i64
  %w8 = add i64 %w7, %e1
  %w9 = add i64 %w8, %e2
  %r = trunc i64 %w9 to i32
  %eq = icmp eq ptr %h, @sig_a
  %rr = select i1 %eq, i32 1, i32 %r
  ret i32 %rr
}
