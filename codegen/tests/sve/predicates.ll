; Predicates (scalable vectors of i1) in p registers: passed, returned,
; kept across calls, and made.
target triple = "aarch64-unknown-linux-gnu"

declare void @clobber()

define <vscale x 4 x i1> @second(<vscale x 4 x i1> %p, <vscale x 4 x i1> %q) #0 {
  ret <vscale x 4 x i1> %q
}

define <vscale x 16 x i1> @keep(<vscale x 16 x i1> %p, <vscale x 16 x i1> %q) #0 {
  call void @clobber()
  call void @clobber()
  ret <vscale x 16 x i1> %p
}

define <vscale x 2 x i1> @all_d() #0 {
  ret <vscale x 2 x i1> splat (i1 true)
}

define <vscale x 8 x i1> @none_h() #0 {
  ret <vscale x 8 x i1> zeroinitializer
}

attributes #0 = { "target-features"="+sve" }
