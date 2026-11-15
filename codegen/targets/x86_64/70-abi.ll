; x86-64's calling convention: the System V x86-64 psABI's. The properties
; are those of codegen/targets/aarch64/70-abi.ll.
;
; Integers and pointers in rdi, rsi, rdx, rcx, r8, r9, and back in rax and
; rdx; floating-point values in xmm0-xmm7, back in xmm0 and xmm1. The call
; pushes the return address ("return-address"="stack"): there is no link
; register, and frames need no frame pointer (unwinding uses the tables),
; so rbp is saved like rbx and r12-r15. r10 and r11 are kept for spill
; code, as are xmm14 and xmm15 and AVX-512's k6 and k7; rbx addresses
; frames whose stack pointer moves.

!abi.sysv = !{!100, !101, !102, !103, !104, !105, !106, !107, !108, !109, !110, !111, !112, !113, !114, !115, !116, !117, !118, !119, !120, !121, !122, !123, !124, !125}
!100 = !{!"args", !"%rdi", !"%rsi", !"%rdx", !"%rcx", !"%r8", !"%r9"}
!101 = !{!"result", !"%rax"}
!102 = !{!"results", !"%rax", !"%rdx"}
; AVX-512's masks are not kept across calls either (k0, which as a mask
; means none, is not allocated).
!103 = !{!"clobbers", !"%rax", !"%rcx", !"%rdx", !"%rsi", !"%rdi", !"%r8", !"%r9", !"%r10", !"%r11", !"%k1-%k7"}
!104 = !{!"saved", !"%rbx", !"%rbp", !"%r12", !"%r13", !"%r14", !"%r15"}
!105 = !{!"reserved"}
!106 = !{!"scratch", !"%r11", !"%r10"}
!107 = !{!"frame", !"%rbp"}
!108 = !{!"return-address", !"stack"}
!109 = !{!"stack-args", !"slots"}
!110 = !{!"variadic", !"named"}
!111 = !{!"syntax", !"elf"}
!112 = !{!"triples", !"x86_64", !"linux", !"elf"}
!113 = !{!"stack-align", !"16"}
!114 = !{!"base", !"%rbx"}
!115 = !{!"eh-regs", !"%rax", !"%rdx"}
!116 = !{!"fp-args", !"%xmm0-%xmm7"}
!117 = !{!"fp-result", !"%xmm0"}
!118 = !{!"fp-clobbers", !"%xmm0-%xmm31"}
!119 = !{!"fp-saved"}
; Three, as an instruction reads up to three vector registers (a blend, an
; FMA, vpternlogd), each of which spill code may have to load.
!120 = !{!"fp-scratch", !"%xmm15", !"%xmm14", !"%xmm13"}
!121 = !{!"fp-results", !"%xmm0", !"%xmm1"}
; __int128 on the stack is 16-aligned.
!122 = !{!"wide-pairs", !"stack16"}
; A variadic call says in al how many vector registers its arguments take.
!123 = !{!"variadic-fp-count", !"%rax"}
; A variadic function's va_list: { gp_offset, fp_offset, overflow_arg_area,
; reg_save_area }.
!124 = !{!"va-list", !"sysv"}
; A spilled mask is reloaded into these.
!125 = !{!"pred-scratch", !"%k7", !"%k6"}
