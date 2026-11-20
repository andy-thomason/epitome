; epitome-opt: --passes=sroa,mem2reg
; Splitting allocas into pieces mem2reg can promote.
target datalayout = "e-m:o-i64:64-i128:128-n32:64-S128"

%point = type { i32, i32 }
%mixed = type { i8, double, [2 x i16] }

declare void @use(ptr)

; A struct used one field at a time.
define i32 @fields(i32 %a, i32 %b) {
  %p = alloca %point
  %y = getelementptr inbounds %point, ptr %p, i32 0, i32 1
  store i32 %a, ptr %p
  store i32 %b, ptr %y
  %x1 = load i32, ptr %p
  %y1 = load i32, ptr %y
  %r = add i32 %x1, %y1
  ret i32 %r
}

; A struct returned in a register: stored as one i64, read as two i32s.
define i32 @returned(i64 %packed) {
  %p = alloca %point
  store i64 %packed, ptr %p
  %y = getelementptr inbounds %point, ptr %p, i32 0, i32 1
  %x1 = load i32, ptr %p
  %y1 = load i32, ptr %y
  %r = sub i32 %x1, %y1
  ret i32 %r
}

; Padding, a double, and an array with constant indices. The first field's
; bytes and the double's are separate pieces; bytes nothing uses are gone.
define double @mixed(i8 %c, double %d, i16 %h) {
  %m = alloca %mixed
  %dp = getelementptr inbounds %mixed, ptr %m, i32 0, i32 1
  %h1 = getelementptr inbounds %mixed, ptr %m, i32 0, i32 2, i32 1
  store i8 %c, ptr %m
  store double %d, ptr %dp
  store i16 %h, ptr %h1
  %d1 = load double, ptr %dp
  %h2 = load i16, ptr %h1
  %hf = sitofp i16 %h2 to double
  %r = fadd double %d1, %hf
  ret double %r
}

; The same bytes as a float and as an integer: one i32 piece, with bitcasts.
define i32 @punning(float %f) {
  %u = alloca float
  store float %f, ptr %u
  %bits = load i32, ptr %u
  ret i32 %bits
}

; A field written after a whole-struct write: the packed integer is masked
; and merged.
define i64 @update(i64 %packed, i32 %y) {
  %p = alloca %point
  store i64 %packed, ptr %p
  %yp = getelementptr inbounds %point, ptr %p, i32 0, i32 1
  store i32 %y, ptr %yp
  %r = load i64, ptr %p
  ret i64 %r
}

; Left alone: the address escapes, or an index is not constant.
define i32 @kept(i64 %i) {
  %a = alloca %point
  %b = alloca [4 x i32]
  store i32 1, ptr %a
  call void @use(ptr %a)
  %e = getelementptr inbounds [4 x i32], ptr %b, i64 0, i64 %i
  store i32 2, ptr %e
  %x = load i32, ptr %a
  %y = load i32, ptr %b
  %r = add i32 %x, %y
  ret i32 %r
}

; Copies and fills: a struct initialised from a constant, copied to
; another, and a zeroed buffer.
@init = private constant %point { i32 3, i32 4 }
declare void @llvm.memcpy.p0.p0.i64(ptr, ptr, i64, i1)
declare void @llvm.memset.p0.i64(ptr, i8, i64, i1)

define i32 @copies() {
  %a = alloca %point
  %b = alloca %point
  %z = alloca [2 x i64]
  call void @llvm.memcpy.p0.p0.i64(ptr %a, ptr @init, i64 8, i1 false)
  call void @llvm.memcpy.p0.p0.i64(ptr %b, ptr %a, i64 8, i1 false)
  call void @llvm.memset.p0.i64(ptr %z, i8 0, i64 16, i1 false)
  %by = getelementptr inbounds %point, ptr %b, i32 0, i32 1
  %x = load i32, ptr %b
  %y = load i32, ptr %by
  %z1 = getelementptr inbounds [2 x i64], ptr %z, i64 0, i64 1
  %w = load i64, ptr %z1
  %w32 = trunc i64 %w to i32
  %s = add i32 %x, %y
  %r = add i32 %s, %w32
  ret i32 %r
}

; An Option<&T>-like slot: written as an integer and read as a pointer,
; then written as a pointer and its niche tested as an integer. The piece
; is a pointer, with casts at the integer accesses.
define i1 @niche(ptr %q, i1 %c) {
  %o = alloca [8 x i8]
  store i64 0, ptr %o
  %none = load ptr, ptr %o
  %p = select i1 %c, ptr %q, ptr %none
  store ptr %p, ptr %o
  %bits = load i64, ptr %o
  %some = icmp ne i64 %bits, 0
  ret i1 %some
}

; An enum of 24 bytes: a tag byte, then a payload at 8 that only one
; variant sets. Built in one alloca, copied whole to another and on to a
; third, which is read. The copies carry bytes that are only copied (the
; payload, as two i64 pieces in each alloca) and bytes nothing writes
; (1 to 8, and the payload for the other variant), which they drop.
define i64 @enum_copies(i1 %which, i64 %a, i64 %b) {
  %e1 = alloca [24 x i8]
  %e2 = alloca [24 x i8]
  %e3 = alloca [24 x i8]
  br i1 %which, label %some, label %none

some:
  store i8 1, ptr %e1
  %pa = getelementptr i8, ptr %e1, i64 8
  store i64 %a, ptr %pa
  %pb = getelementptr i8, ptr %e1, i64 16
  store i64 %b, ptr %pb
  br label %join

none:
  store i8 0, ptr %e1
  br label %join

join:
  call void @llvm.memcpy.p0.p0.i64(ptr %e2, ptr %e1, i64 24, i1 false)
  call void @llvm.memcpy.p0.p0.i64(ptr %e3, ptr %e2, i64 24, i1 false)
  %tag = load i8, ptr %e3
  %is = icmp eq i8 %tag, 1
  %x = getelementptr i8, ptr %e3, i64 16
  %v = load i64, ptr %x
  %r = select i1 %is, i64 %v, i64 0
  ret i64 %r
}
