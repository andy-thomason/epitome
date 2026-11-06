; The AArch64 description: calling conventions and platforms. The
; conventions are in 00-base.ll.

; ===========================================================================
; Calling conventions and platforms
;
; Each is a list of properties, each a tuple of strings: the property's
; name, then its values. Registers are named as in assembly; "x0-x7" is a
; range.
;
;   args          registers for integer and pointer arguments, in order
;   result        the register an integer or pointer result comes back in
;   clobbers      registers a call may change
;   saved         registers a function must preserve, saving them if used
;   reserved      registers never allocated (the platform's)
;   scratch       registers kept free for spill code
;   frame         the frame pointer and the link register
;   stack-args    how arguments past the registers are laid out on the
;                 stack: "natural" (each at its own size and alignment,
;                 packed) or "slots" (each in an 8-byte slot)
;   variadic      where the variable arguments of a variadic call go:
;                 "stack" (always, in 8-byte slots) or "named" (as named
;                 arguments are)
;   syntax        the assembler's dialect: "macho" or "elf"
;   triples       substrings of a module's target triple that select it
;   sret          the register for the address a result is returned at
;                 (an sret argument); without it, the first argument's
;   base          a saved register that addresses the frame in a function
;                 whose stack pointer moves (dynamic allocas)
;   wide-pairs    "even": an integer wider than a register (i128) starts in
;                 an even register, and at a 16-aligned stack slot
;   va-list       "aapcs64": a variadic function's va_list is AAPCS64's
;                 record of the saved argument registers and the stack
;                 (__stack, __gr_top, __vr_top, __gr_offs, __vr_offs);
;                 "sysv": System V x86-64's (gp_offset, fp_offset,
;                 overflow_arg_area, reg_save_area); without it, a pointer
;                 to the next argument
;   stack-align   the stack pointer's alignment at calls, in bytes
;   hfa           "yes": homogeneous floating-point aggregates (up to four
;                 floats, doubles or 64- or 128-bit vectors, as [3 x float]
;                 or a struct of them) go in consecutive floating-point
;                 registers, all or none, and come back in them
;   far-scratch   a register never allocated, for the address of a stack
;                 slot too far for an instruction's offset
;   fp-args, fp-result, fp-clobbers, fp-saved, fp-scratch
;                 the same, for floating-point values in d0-d31 (only
;                 the low 64 bits of d8-d15 are preserved; SVE's z0-z31,
;                 these registers, are passed and returned as they are)
;   pred-args, pred-result, pred-clobbers, pred-scratch
;                 the same, for SVE's predicates (p0-p3 for arguments, p0
;                 for a result; none preserved but under the SVE
;                 convention, which is not this one)

; Apple's arm64 ABI: AAPCS64 with x18 reserved, stack arguments packed,
; variadic arguments on the stack.
!abi.darwin = !{!100, !101, !102, !103, !104, !105, !106, !107, !108, !109, !110, !111, !112, !113, !114, !115, !116, !117, !118, !119, !120, !121, !122, !123, !124}
!100 = !{!"args", !"x0-x7"}
!101 = !{!"result", !"x0"}
!102 = !{!"clobbers", !"x0-x18", !"x30"}
!103 = !{!"saved", !"x19-x28"}
!104 = !{!"reserved", !"x18"}
!105 = !{!"scratch", !"x15-x17"}
!106 = !{!"frame", !"x29", !"x30"}
!107 = !{!"stack-args", !"natural"}
!108 = !{!"variadic", !"stack"}
!109 = !{!"syntax", !"macho"}
!110 = !{!"triples", !"apple", !"darwin", !"macos", !"ios"}
!111 = !{!"fp-args", !"d0-d7"}
!112 = !{!"fp-result", !"d0"}
!113 = !{!"fp-clobbers", !"d0-d7", !"d16-d31"}
!114 = !{!"fp-saved", !"d8-d15"}
!115 = !{!"fp-scratch", !"d29-d31"}
!116 = !{!"sret", !"x8"}
!117 = !{!"base", !"x19"}
!118 = !{!"stack-align", !"16"}
!119 = !{!"hfa", !"yes"}
!120 = !{!"far-scratch", !"x14"}
!121 = !{!"pred-args", !"p0-p3"}
!122 = !{!"pred-result", !"p0"}
!123 = !{!"pred-clobbers", !"p0-p14"}
!124 = !{!"pred-scratch", !"p15"}

; The Procedure Call Standard for the Arm 64-bit Architecture, as used by
; Linux: x18 is an ordinary temporary, stack arguments take 8-byte slots,
; and variadic arguments go where named ones would.
!abi.aapcs64 = !{!200, !201, !202, !203, !204, !205, !206, !207, !208, !209, !210, !211, !212, !213, !214, !215, !216, !217, !218, !219, !220, !221, !222, !223, !224, !225, !226}
!200 = !{!"args", !"x0-x7"}
!201 = !{!"result", !"x0"}
!202 = !{!"clobbers", !"x0-x18", !"x30"}
!203 = !{!"saved", !"x19-x28"}
!204 = !{!"reserved"}
!205 = !{!"scratch", !"x15-x17"}
!206 = !{!"frame", !"x29", !"x30"}
!207 = !{!"stack-args", !"slots"}
!208 = !{!"variadic", !"named"}
!209 = !{!"syntax", !"elf"}
!210 = !{!"triples", !"linux", !"elf", !"none"}
!211 = !{!"fp-args", !"d0-d7"}
!212 = !{!"fp-result", !"d0"}
!213 = !{!"fp-clobbers", !"d0-d7", !"d16-d31"}
!214 = !{!"fp-saved", !"d8-d15"}
!215 = !{!"fp-scratch", !"d29-d31"}
!216 = !{!"sret", !"x8"}
!217 = !{!"base", !"x19"}
!218 = !{!"wide-pairs", !"even"}
!219 = !{!"va-list", !"aapcs64"}
!220 = !{!"stack-align", !"16"}
!221 = !{!"hfa", !"yes"}
!222 = !{!"far-scratch", !"x14"}
!223 = !{!"pred-args", !"p0-p3"}
!224 = !{!"pred-result", !"p0"}
!225 = !{!"pred-clobbers", !"p0-p14"}
!226 = !{!"pred-scratch", !"p15"}
