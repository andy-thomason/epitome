; @base_b: merge: the same as @base_a, now an alias of it

target datalayout = "e-m:o-i64:64-i128:128-n32:64-S128"

@k1 = private unnamed_addr constant i32 42
@k3 = private constant i32 42
@ba = global ptr blockaddress(@blocks, %next)

define internal i32 @base_a(i32 %x) unnamed_addr noinline {
  %a = mul i32 %x, 3
  %b = add i32 %a, 1
  ret i32 %b
}

define weak i32 @weak(i32 %x) unnamed_addr noinline {
  %a = mul i32 %x, 3
  %b = add i32 %a, 1
  ret i32 %b
}

define i32 @blocks(i32 %x) unnamed_addr noinline {
  br label %next

next:                                             ; preds = %0
  %a = mul i32 %x, 3
  %b = add i32 %a, 1
  ret i32 %b
}

define internal i32 @blocks2(i32 %x) unnamed_addr noinline {
  br label %next

next:                                             ; preds = %0
  %a = mul i32 %x, 3
  %b = add i32 %a, 1
  ret i32 %b
}

define internal i32 @unoptimised(i32 %x) unnamed_addr noinline "epitome-opt-level"="0" {
  %a = mul i32 %x, 3
  %b = add i32 %a, 1
  ret i32 %b
}

define internal i32 @opaque(i32 %x) unnamed_addr noinline "epitome-opaque" {
  %a = mul i32 %x, 3
  %b = add i32 %a, 1
  ret i32 %b
}

define internal i32 @cold(i32 %x) unnamed_addr noinline cold {
  %a = mul i32 %x, 3
  %b = add i32 %a, 1
  ret i32 %b
}

define i32 @main() {
  %v1 = call i32 @base_a(i32 1)
  %v2 = call i32 @base_a(i32 2)
  %v3 = call i32 @weak(i32 3)
  %v5 = call i32 @blocks(i32 5)
  %v5b = call i32 @blocks2(i32 5)
  %v6a = call i32 @unoptimised(i32 6)
  %v6 = add i32 %v6a, %v5b
  %v7 = call i32 @opaque(i32 7)
  %v8 = call i32 @cold(i32 8)
  %k1 = load i32, ptr @k1
  %k2 = load i32, ptr @k1
  %k3 = load i32, ptr @k3
  %s1 = add i32 %v1, %v2
  %s2 = add i32 %s1, %v3
  %s3 = add i32 %s2, 4
  %s4 = add i32 %s3, %v5
  %s5 = add i32 %s4, %v6
  %s6 = add i32 %s5, %v7
  %s7 = add i32 %s6, %v8
  %s8 = add i32 %s7, %k1
  %s9 = add i32 %s8, %k2
  %s10 = add i32 %s9, %k3
  %r = and i32 %s10, 255
  ret i32 %r
}
