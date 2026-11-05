; fib.rs as rustc writes it for a bare-metal RISC-V target:
;   rustc --edition 2021 --crate-type=lib --target riscv64imac-unknown-none-elf \
;         -C opt-level=2 -C panic=abort --emit=llvm-ir fib.rs
; (tests/partial.rs makes it again where rustc can).

target datalayout = "e-m:e-p:64:64-i64:64-i128:128-n32:64-S128"
target triple = "riscv64"

; __rustc::rust_begin_unwind
; Function Attrs: nofree norecurse noreturn nosync nounwind memory(none)
define dso_local void @_RNvCs5fGHjzWXRav_7___rustc17rust_begin_unwind(ptr noalias nofree noundef readonly align 8 captures(none) dereferenceable(24) %_1) unnamed_addr #0 {
start:
  br label %bb1

bb1:                                              ; preds = %bb1, %start
  br label %bb1
}

; Function Attrs: nofree norecurse nosync nounwind memory(none)
define dso_local noundef signext i32 @fib(i32 noundef signext %n) unnamed_addr #1 {
start:
  %_54.not = icmp eq i32 %n, 0
  br i1 %_54.not, label %bb3, label %bb2

bb3:                                              ; preds = %bb2, %start
  %a.sroa.0.0.lcssa = phi i32 [ 0, %start ], [ %b.sroa.0.06, %bb2 ]
  ret i32 %a.sroa.0.0.lcssa

bb2:                                              ; preds = %start, %bb2
  %a.sroa.0.07 = phi i32 [ %b.sroa.0.06, %bb2 ], [ 0, %start ]
  %b.sroa.0.06 = phi i32 [ %c, %bb2 ], [ 1, %start ]
  %iter.sroa.0.05 = phi i32 [ %_6, %bb2 ], [ 0, %start ]
  %_6 = add nuw i32 %iter.sroa.0.05, 1
  %c = add i32 %a.sroa.0.07, %b.sroa.0.06
  %exitcond.not = icmp eq i32 %_6, %n
  br i1 %exitcond.not, label %bb3, label %bb2
}

attributes #0 = { nofree norecurse noreturn nosync nounwind memory(none) "target-cpu"="generic-rv64" "target-features"="+m,+a,+zaamo,+zalrsc,+c,+zca" }
attributes #1 = { nofree norecurse nosync nounwind memory(none) "target-cpu"="generic-rv64" "target-features"="+m,+a,+zaamo,+zalrsc,+c,+zca" }

!llvm.module.flags = !{!0, !1}
!llvm.ident = !{!2}

!0 = !{i32 1, !"Code Model", i32 3}
!1 = !{i32 1, !"target-abi", !"lp64"}
!2 = !{!"rustc"}
