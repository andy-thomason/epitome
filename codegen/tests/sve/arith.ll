; Unpredicated integer arithmetic of scalable vectors, in and out in z
; registers.
target triple = "aarch64-unknown-linux-gnu"

define <vscale x 4 x i32> @add_s(<vscale x 4 x i32> %a, <vscale x 4 x i32> %b) #0 {
  %c = add <vscale x 4 x i32> %a, %b
  ret <vscale x 4 x i32> %c
}

define <vscale x 16 x i8> @add_b(<vscale x 16 x i8> %a, <vscale x 16 x i8> %b) #0 {
  %c = add <vscale x 16 x i8> %b, %a
  %d = add <vscale x 16 x i8> %c, %c
  ret <vscale x 16 x i8> %d
}

define <vscale x 8 x i16> @add_h(<vscale x 8 x i16> %a, <vscale x 8 x i16> %b) #0 {
  %c = add <vscale x 8 x i16> %a, %b
  ret <vscale x 8 x i16> %c
}

define <vscale x 2 x i64> @add_d3(<vscale x 2 x i64> %a, <vscale x 2 x i64> %b, <vscale x 2 x i64> %c) #0 {
  %s = add <vscale x 2 x i64> %a, %b
  %t = add <vscale x 2 x i64> %s, %c
  ret <vscale x 2 x i64> %t
}

attributes #0 = { "target-features"="+sve" }
