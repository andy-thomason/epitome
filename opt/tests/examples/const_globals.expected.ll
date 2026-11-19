; @f: combine: (constant to the right) x1, (dead) x3, (folded) x10, (reassociated) x1

target datalayout = "e-m:o-i64:64-i128:128-n32:64-S128"

%pt = type { i32, i64, [2 x i16] }

@var = global i32 5
@weak = weak constant i32 6

define i64 @f() {
  %e = load i32, ptr @var
  %w = load i32, ptr @weak
  %e64 = zext i32 %e to i64
  %w64 = zext i32 %w to i64
  %1 = add i64 %e64, %w64
  %s5 = add i64 %1, 1869376649
  ret i64 %s5
}
