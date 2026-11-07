; SME2's lists of two and four registers start at a multiple of two or
; four: values that arrive elsewhere (z1, out of order) must be moved to a
; register the encoding has. No qemu here runs SME2, so tests/sve.rs has
; clang assemble our assembly, which it refuses where a list is out of
; line, and compares the code with ours.
target triple = "aarch64-unknown-linux-gnu"

; {a, b} from z1, z2: an odd start.
define void @two_odd(i32 %w, <vscale x 4 x i32> %m, <vscale x 4 x i32> %a, <vscale x 4 x i32> %b) #0 {
  call void @llvm.aarch64.sme.add.write.single.za.vg1x2.nxv4i32(i32 %w, <vscale x 4 x i32> %a, <vscale x 4 x i32> %b, <vscale x 4 x i32> %m)
  ret void
}

; {a, b, c, d} from z1-z4, then again in another order.
define void @four_twice(i32 %w, <vscale x 4 x i32> %m, <vscale x 4 x i32> %a, <vscale x 4 x i32> %b, <vscale x 4 x i32> %c, <vscale x 4 x i32> %d) #0 {
  call void @llvm.aarch64.sme.add.write.single.za.vg1x4.nxv4i32(i32 %w, <vscale x 4 x i32> %a, <vscale x 4 x i32> %b, <vscale x 4 x i32> %c, <vscale x 4 x i32> %d, <vscale x 4 x i32> %m)
  call void @llvm.aarch64.sme.add.write.single.za.vg1x4.nxv4i32(i32 %w, <vscale x 4 x i32> %d, <vscale x 4 x i32> %a, <vscale x 4 x i32> %c, <vscale x 4 x i32> %b, <vscale x 4 x i32> %m)
  ret void
}

; Two lists in one instruction, of the same values crossed.
define void @two_lists(i32 %w, <vscale x 4 x i32> %a, <vscale x 4 x i32> %b, <vscale x 4 x i32> %c) #0 {
  call void @llvm.aarch64.sme.add.write.za.vg1x2.nxv4i32(i32 %w, <vscale x 4 x i32> %b, <vscale x 4 x i32> %c, <vscale x 4 x i32> %a, <vscale x 4 x i32> %b)
  call void @llvm.aarch64.sme.add.write.za.vg1x4.nxv4i32(i32 %w, <vscale x 4 x i32> %c, <vscale x 4 x i32> %b, <vscale x 4 x i32> %a, <vscale x 4 x i32> %c, <vscale x 4 x i32> %a, <vscale x 4 x i32> %a, <vscale x 4 x i32> %b, <vscale x 4 x i32> %c)
  ret void
}

declare void @llvm.aarch64.sme.add.write.single.za.vg1x2.nxv4i32(i32, <vscale x 4 x i32>, <vscale x 4 x i32>, <vscale x 4 x i32>)
declare void @llvm.aarch64.sme.add.write.single.za.vg1x4.nxv4i32(i32, <vscale x 4 x i32>, <vscale x 4 x i32>, <vscale x 4 x i32>, <vscale x 4 x i32>, <vscale x 4 x i32>)
declare void @llvm.aarch64.sme.add.write.za.vg1x2.nxv4i32(i32, <vscale x 4 x i32>, <vscale x 4 x i32>, <vscale x 4 x i32>, <vscale x 4 x i32>)
declare void @llvm.aarch64.sme.add.write.za.vg1x4.nxv4i32(i32, <vscale x 4 x i32>, <vscale x 4 x i32>, <vscale x 4 x i32>, <vscale x 4 x i32>, <vscale x 4 x i32>, <vscale x 4 x i32>, <vscale x 4 x i32>, <vscale x 4 x i32>)

attributes #0 = { "aarch64_pstate_sm_enabled" "aarch64_inout_za" "target-features"="+sve,+sve2,+sme,+sme2" }
