; Scalable vectors to and from memory, a whole register at a time (ldr,
; str), of each lane type.
target triple = "aarch64-unknown-linux-gnu"

define void @add_s(ptr %d, ptr %a, ptr %b) #0 {
  %x = load <vscale x 4 x i32>, ptr %a
  %y = load <vscale x 4 x i32>, ptr %b
  %z = add <vscale x 4 x i32> %x, %y
  store <vscale x 4 x i32> %z, ptr %d
  ret void
}

define void @fma_f(ptr %d, ptr %a, ptr %b) #0 {
  %x = load <vscale x 4 x float>, ptr %a
  %y = load <vscale x 4 x float>, ptr %b
  %p = fmul <vscale x 4 x float> %x, %y
  %q = fadd <vscale x 4 x float> %p, %x
  %r = fsub <vscale x 4 x float> %q, %y
  store <vscale x 4 x float> %r, ptr %d
  ret void
}

define void @logic_b(ptr %d, ptr %a, ptr %b) #0 {
  %x = load <vscale x 16 x i8>, ptr %a
  %y = load <vscale x 16 x i8>, ptr %b
  %p = and <vscale x 16 x i8> %x, %y
  %q = xor <vscale x 16 x i8> %p, %y
  %r = or <vscale x 16 x i8> %q, %x
  store <vscale x 16 x i8> %r, ptr %d
  ret void
}

define void @copy_d(ptr %d, ptr %a) #0 {
  %x = load <vscale x 2 x double>, ptr %a
  store <vscale x 2 x double> %x, ptr %d
  ret void
}

attributes #0 = { "target-features"="+sve" }
