; ranges: bounds checks that what an index can be decides, with no branch
; on it: a byte into a 256-entry table, a mask, a shift, a parameter with a
; `range` attribute, and a counter with a constant bound used through
; arithmetic. The checks against too small an array stay.
target datalayout = "e-m:o-i64:64-i128:128-n32:64-S128"

define void @panic_bounds_check(i64 %i, i64 %len) noinline noreturn {
  call void @llvm.trap()
  unreachable
}

declare void @llvm.trap()

; TABLE[x as usize] for a [u8; 256], and TABLE[(x & 15) as usize] for 16.
define i64 @lookups(ptr %t256, ptr %t16, i8 %x, i64 %h, i64 range(i64 0, 10) %d) noinline {
start:
  %i = zext i8 %x to i64
  %in = icmp ult i64 %i, 256
  br i1 %in, label %ok, label %panic

ok:
  %p = getelementptr inbounds i8, ptr %t256, i64 %i
  %a = load i8, ptr %p
  %m = and i64 %h, 15
  %in2 = icmp ult i64 %m, 16
  br i1 %in2, label %ok2, label %panic

ok2:
  %q = getelementptr inbounds i8, ptr %t16, i64 %m
  %b = load i8, ptr %q
  %top = lshr i64 %h, 60
  %in3 = icmp ult i64 %top, 16
  br i1 %in3, label %ok3, label %panic

ok3:
  %r = getelementptr inbounds i8, ptr %t16, i64 %top
  %c = load i8, ptr %r
  %in4 = icmp ult i64 %d, 10
  br i1 %in4, label %ok4, label %panic

ok4:
  %s = getelementptr inbounds i8, ptr %t16, i64 %d
  %e = load i8, ptr %s
  ; Too wide for 16 entries: stays.
  %w = and i64 %h, 31
  %in5 = icmp ult i64 %w, 16
  br i1 %in5, label %ok5, label %panic

ok5:
  %u = getelementptr inbounds i8, ptr %t16, i64 %w
  %f = load i8, ptr %u
  %a1 = zext i8 %a to i64
  %b1 = zext i8 %b to i64
  %c1 = zext i8 %c to i64
  %e1 = zext i8 %e to i64
  %f1 = zext i8 %f to i64
  %s1 = add i64 %a1, %b1
  %s2 = add i64 %s1, %c1
  %s3 = add i64 %s2, %e1
  %s4 = add i64 %s3, %f1
  ret i64 %s4

panic:
  call void @panic_bounds_check(i64 0, i64 0)
  unreachable
}

; for i in 0..8 { s += a[2 * i + 1] } with a: [u64; 16]: `i < 8` gives
; `i <= 7`, so `2 * i + 1 <= 15`. For a[2 * i + 2] the check stays.
define i64 @odd(ptr %a) noinline {
start:
  br label %for

for:
  %i = phi i64 [ 0, %start ], [ %i.next, %ok2 ]
  %s = phi i64 [ 0, %start ], [ %s.next, %ok2 ]
  %more = icmp ult i64 %i, 8
  br i1 %more, label %body, label %done

body:
  %two = shl i64 %i, 1
  %k = add i64 %two, 1
  %in = icmp ult i64 %k, 16
  br i1 %in, label %ok, label %panic

ok:
  %p = getelementptr inbounds i64, ptr %a, i64 %k
  %x = load i64, ptr %p
  %k2 = add i64 %two, 2
  %in2 = icmp ult i64 %k2, 16
  br i1 %in2, label %ok2, label %panic

ok2:
  %q = getelementptr inbounds i64, ptr %a, i64 %k2
  %y = load i64, ptr %q
  %xy = add i64 %x, %y
  %s.next = add i64 %s, %xy
  %i.next = add i64 %i, 1
  br label %for

panic:
  call void @panic_bounds_check(i64 %k, i64 16)
  unreachable

done:
  ret i64 %s
}

define i32 @main() {
  %t256 = alloca [256 x i8]
  %t16 = alloca [16 x i8]
  %a = alloca [17 x i64]
  call void @llvm.memset.p0.i64(ptr %t256, i8 3, i64 256, i1 false)
  call void @llvm.memset.p0.i64(ptr %t16, i8 5, i64 16, i1 false)
  call void @llvm.memset.p0.i64(ptr %a, i8 1, i64 136, i1 false)
  %l = call i64 @lookups(ptr %t256, ptr %t16, i8 200, i64 -1152921504606846965, i64 7)
  %o = call i64 @odd(ptr %a)
  %t = add i64 %l, %o
  %r = trunc i64 %t to i32
  ret i32 %r
}

declare void @llvm.memset.p0.i64(ptr, i8, i64, i1)
