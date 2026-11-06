; The gfx1151 (RDNA 3.5) description: the files, and the machine as a whole.
; Written by gen_gfx1151.py.
;
;   00-base.ll    what the machine lacks, its parameters
;   05-enums.ll   operands written by name: registers, constants, modifiers
;   10-scalar.ll  scalar ALU, scalar memory and program control
;   20-vector.ll  vector ALU
;   30-memory.ll  vector memory: LDS, buffers, flat, global, scratch, images
;
; For assembly and disassembly only: instruction selection has nothing to
; select yet. Instructions are 32 bits or more, in 32-bit units; wave32.
!target.lacks = !{!1000}
!1000 = !{!"stack", !"calls", !"globals", !"indirect", !"trap"}
; The scalar registers. Instructions name registers, and ranges of them
; (s[4:7], v[0:3]), as operands written by name (enums), so the class is
; only a name for the file.
!target.registers = !{!1050}
!1050 = !{!"s", !"s{0-105}"}
!target.classes = !{!1060}
!1060 = !{!"s", !"file", !"s"}
!target.params = !{!1100, !1101, !1102, !1103, !1104}
!1100 = !{!"instruction-unit", !"4"}
!1101 = !{!"vector-bits", !"0"}
!1102 = !{!"asm-comment", !";"}
; EM_AMDGPU, and EF_AMDGPU_MACH_AMDGCN_GFX1151.
!1103 = !{!"elf-machine", !"224"}
!1104 = !{!"elf-flags", !"74"}
!target.cpus = !{!1150}
!1150 = !{!"gfx1151"}
