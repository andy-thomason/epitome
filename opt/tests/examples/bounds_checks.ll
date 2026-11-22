; implied: slice bounds checks under a loop's own test, in the shape
; epitome-rustc gives `for i in 0..n { ... v[i] ... }` once `Range::next`
; is inlined: the test makes an `Option` (a bool byte and a payload that is
; `undef` when there is none), and each index is checked against the
; length, with a branch to a call that does not return. The checks go, the
; loops rotate, and the exit tests become exact.
target datalayout = "e-m:o-i64:64-i128:128-n32:64-S128"

define void @panic_bounds_check(i64 %i, i64 %len) noinline noreturn {
  call void @llvm.trap()
  unreachable
}

declare void @llvm.trap()

; fn sum(v: &[u64]) -> u64 { let mut s = 0; for i in 0..v.len() { s += v[i]; } s }
define i64 @sum(ptr %v, i64 %len) noinline {
start:
  br label %for

for:
  %i = phi i64 [ 0, %start ], [ %i.next, %ok ]
  %s = phi i64 [ 0, %start ], [ %s.next, %ok ]
  %lt = icmp ult i64 %i, %len
  %lt.byte = zext i1 %lt to i8
  %more = icmp ne i8 %lt.byte, 0
  %inc = add i64 %i, 1
  %some = select i1 %more, i64 %i, i64 undef
  %tag = select i1 %more, i8 1, i8 0
  %i.next = select i1 %more, i64 %inc, i64 %i
  %is.some = icmp eq i8 %tag, 1
  br i1 %is.some, label %body, label %done

body:
  %in = icmp ult i64 %some, %len
  br i1 %in, label %ok, label %panic

ok:
  %p = getelementptr inbounds i64, ptr %v, i64 %some
  %x = load i64, ptr %p
  %s.next = add i64 %s, %x
  br label %for

panic:
  call void @panic_bounds_check(i64 %some, i64 %len)
  unreachable

done:
  ret i64 %s
}

; fn dot(a: &[i64], b: &[i64]) -> i64 {
;     let mut s = 0; for i in 0..a.len().min(b.len()) { s += a[i] * b[i]; } s }
define i64 @dot(ptr %a, i64 %alen, ptr %b, i64 %blen) noinline {
start:
  %shorter = icmp ult i64 %blen, %alen
  %n = select i1 %shorter, i64 %blen, i64 %alen
  br label %for

for:
  %i = phi i64 [ 0, %start ], [ %i.next, %ok.b ]
  %s = phi i64 [ 0, %start ], [ %s.next, %ok.b ]
  %more = icmp ult i64 %i, %n
  %inc = add i64 %i, 1
  %some = select i1 %more, i64 %i, i64 undef
  %i.next = select i1 %more, i64 %inc, i64 %i
  br i1 %more, label %body, label %done

body:
  %in.a = icmp ult i64 %some, %alen
  br i1 %in.a, label %ok.a, label %panic

ok.a:
  %pa = getelementptr inbounds i64, ptr %a, i64 %some
  %x = load i64, ptr %pa
  %in.b = icmp ult i64 %some, %blen
  br i1 %in.b, label %ok.b, label %panic.b

ok.b:
  %pb = getelementptr inbounds i64, ptr %b, i64 %some
  %y = load i64, ptr %pb
  %xy = mul i64 %x, %y
  %s.next = add i64 %s, %xy
  br label %for

panic:
  call void @panic_bounds_check(i64 %some, i64 %alen)
  unreachable

panic.b:
  call void @panic_bounds_check(i64 %some, i64 %blen)
  unreachable

done:
  ret i64 %s
}

; fn rev(v: &mut [u8]) { let n = v.len(); for i in 0..n / 2 { v.swap(i, n - 1 - i); } }
; `n - 1 - i` is below `n`: `i < n / 2` gives `i < n`, so `n != 0`, and
; `i <= n - 1`.
define void @rev(ptr %v, i64 %n) noinline {
start:
  %half = lshr i64 %n, 1
  %last = add i64 %n, -1
  br label %for

for:
  %i = phi i64 [ 0, %start ], [ %i.next, %ok.j ]
  %more = icmp ult i64 %i, %half
  %inc = add i64 %i, 1
  %some = select i1 %more, i64 %i, i64 undef
  %i.next = select i1 %more, i64 %inc, i64 %i
  br i1 %more, label %body, label %done

body:
  %j = sub i64 %last, %some
  %in.i = icmp ult i64 %some, %n
  br i1 %in.i, label %ok.i, label %panic.i

ok.i:
  %in.j = icmp ult i64 %j, %n
  br i1 %in.j, label %ok.j, label %panic.j

ok.j:
  %pi = getelementptr inbounds i8, ptr %v, i64 %some
  %pj = getelementptr inbounds i8, ptr %v, i64 %j
  %x = load i8, ptr %pi
  %y = load i8, ptr %pj
  store i8 %y, ptr %pi
  store i8 %x, ptr %pj
  br label %for

panic.i:
  call void @panic_bounds_check(i64 %some, i64 %n)
  unreachable

panic.j:
  call void @panic_bounds_check(i64 %j, i64 %n)
  unreachable

done:
  ret void
}

; Checks that stay: the loop runs to `%n`, not the length, and `%m` is
; not tested at all. `i + 1` still gets `nuw`: `i` is below something.
define i64 @kept(ptr %v, i64 %len, i64 %n, i64 %m) noinline {
start:
  br label %for

for:
  %i = phi i64 [ 0, %start ], [ %i.next, %ok2 ]
  %s = phi i64 [ 0, %start ], [ %s.next, %ok2 ]
  %more = icmp ult i64 %i, %n
  br i1 %more, label %body, label %done

body:
  %in = icmp ult i64 %i, %len
  br i1 %in, label %ok, label %panic

ok:
  %p = getelementptr inbounds i64, ptr %v, i64 %i
  %x = load i64, ptr %p
  %in2 = icmp ult i64 %m, %len
  br i1 %in2, label %ok2, label %panic

ok2:
  %q = getelementptr inbounds i64, ptr %v, i64 %m
  %y = load i64, ptr %q
  %xy = add i64 %x, %y
  %s.next = add i64 %s, %xy
  %i.next = add i64 %i, 1
  br label %for

panic:
  call void @panic_bounds_check(i64 %i, i64 %len)
  unreachable

done:
  ret i64 %s
}

define i32 @main() {
  %a = alloca [5 x i64]
  %b = alloca [3 x i64]
  %c = alloca [5 x i8]
  store i64 1, ptr %a
  %a1 = getelementptr i64, ptr %a, i64 1
  store i64 2, ptr %a1
  %a2 = getelementptr i64, ptr %a, i64 2
  store i64 3, ptr %a2
  %a3 = getelementptr i64, ptr %a, i64 3
  store i64 4, ptr %a3
  %a4 = getelementptr i64, ptr %a, i64 4
  store i64 5, ptr %a4
  store i64 7, ptr %b
  %b1 = getelementptr i64, ptr %b, i64 1
  store i64 11, ptr %b1
  %b2 = getelementptr i64, ptr %b, i64 2
  store i64 13, ptr %b2
  store i8 97, ptr %c
  %c1 = getelementptr i8, ptr %c, i64 1
  store i8 98, ptr %c1
  %c2 = getelementptr i8, ptr %c, i64 2
  store i8 99, ptr %c2
  %c3 = getelementptr i8, ptr %c, i64 3
  store i8 100, ptr %c3
  %c4 = getelementptr i8, ptr %c, i64 4
  store i8 101, ptr %c4
  %s = call i64 @sum(ptr %a, i64 5)
  %d = call i64 @dot(ptr %a, i64 5, ptr %b, i64 3)
  call void @rev(ptr %c, i64 5)
  %r0 = load i8, ptr %c
  %r0w = zext i8 %r0 to i64
  %k = call i64 @kept(ptr %a, i64 5, i64 4, i64 1)
  %t1 = add i64 %s, %d
  %t2 = add i64 %t1, %r0w
  %t3 = add i64 %t2, %k
  %r = trunc i64 %t3 to i32
  ret i32 %r
}
