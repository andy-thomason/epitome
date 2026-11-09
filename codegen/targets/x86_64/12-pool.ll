; The x86-64 description: AVX-512 operations on a constant whose lanes are
; one value, read from read-only data as a broadcast operand
; (`sym(%rip){1to16}`): no register holds the constant, and none is
; spilled for it in a loop where registers run short. The "pool" operand
; matches such a constant; selection puts its lane in read-only data
; (isel.rs, `pool_lane`). Encodings as 02-xed.ll's `_p_u_b` forms.


define <16 x i32> @VPADDDzb.16xi32(<16 x i32> "reg"="zv" %n, <16 x i32> "sym" "pool" %c) "enc"="evexm(d, 16, 4, 1, 0, n, 2, 1, 1, 0, 0) | 11111110 | rm(d, 16, 4, 1, 0, c, 4)" "asm"="vpaddd {c}(%rip){lb}1to16{rb}, {n}, {d}" "reloc"="pc32x" "dclass"="zv" "feature"="avx512f" "nodecode" {
  %r = add <16 x i32> %n, %c
  ret <16 x i32> %r
}

define <16 x i32> @VPSUBDzb.16xi32(<16 x i32> "reg"="zv" %n, <16 x i32> "sym" "pool" %c) "enc"="evexm(d, 16, 4, 1, 0, n, 2, 1, 1, 0, 0) | 11111010 | rm(d, 16, 4, 1, 0, c, 4)" "asm"="vpsubd {c}(%rip){lb}1to16{rb}, {n}, {d}" "reloc"="pc32x" "dclass"="zv" "feature"="avx512f" "nodecode" {
  %r = sub <16 x i32> %n, %c
  ret <16 x i32> %r
}

define <16 x i32> @VPMULLDzb.16xi32(<16 x i32> "reg"="zv" %n, <16 x i32> "sym" "pool" %c) "enc"="evexm(d, 16, 4, 2, 0, n, 2, 1, 1, 0, 0) | 01000000 | rm(d, 16, 4, 1, 0, c, 4)" "asm"="vpmulld {c}(%rip){lb}1to16{rb}, {n}, {d}" "reloc"="pc32x" "dclass"="zv" "feature"="avx512f" "nodecode" {
  %r = mul <16 x i32> %n, %c
  ret <16 x i32> %r
}

define <16 x i32> @VPANDDzb.16xi32(<16 x i32> "reg"="zv" %n, <16 x i32> "sym" "pool" %c) "enc"="evexm(d, 16, 4, 1, 0, n, 2, 1, 1, 0, 0) | 11011011 | rm(d, 16, 4, 1, 0, c, 4)" "asm"="vpandd {c}(%rip){lb}1to16{rb}, {n}, {d}" "reloc"="pc32x" "dclass"="zv" "feature"="avx512f" "nodecode" {
  %r = and <16 x i32> %n, %c
  ret <16 x i32> %r
}

define <16 x i32> @VPORDzb.16xi32(<16 x i32> "reg"="zv" %n, <16 x i32> "sym" "pool" %c) "enc"="evexm(d, 16, 4, 1, 0, n, 2, 1, 1, 0, 0) | 11101011 | rm(d, 16, 4, 1, 0, c, 4)" "asm"="vpord {c}(%rip){lb}1to16{rb}, {n}, {d}" "reloc"="pc32x" "dclass"="zv" "feature"="avx512f" "nodecode" {
  %r = or <16 x i32> %n, %c
  ret <16 x i32> %r
}

define <16 x i32> @VPXORDzb.16xi32(<16 x i32> "reg"="zv" %n, <16 x i32> "sym" "pool" %c) "enc"="evexm(d, 16, 4, 1, 0, n, 2, 1, 1, 0, 0) | 11101111 | rm(d, 16, 4, 1, 0, c, 4)" "asm"="vpxord {c}(%rip){lb}1to16{rb}, {n}, {d}" "reloc"="pc32x" "dclass"="zv" "feature"="avx512f" "nodecode" {
  %r = xor <16 x i32> %n, %c
  ret <16 x i32> %r
}

define <16 x i32> @VPANDNDzb.16xi32(<16 x i32> "reg"="zv" %n, <16 x i32> "sym" "pool" %c) "enc"="evexm(d, 16, 4, 1, 0, n, 2, 1, 1, 0, 0) | 11011111 | rm(d, 16, 4, 1, 0, c, 4)" "asm"="vpandnd {c}(%rip){lb}1to16{rb}, {n}, {d}" "reloc"="pc32x" "dclass"="zv" "feature"="avx512f" "nodecode" {
  %x = xor <16 x i32> %n, splat (i32 -1)
  %r = and <16 x i32> %x, %c
  ret <16 x i32> %r
}

define <16 x i1> @VPCMPDb.eq.16xi32(<16 x i32> "reg"="zv" %n, <16 x i32> "sym" "pool" %c) "enc"="evexm(d, 16, 4, 3, 0, n, 2, 1, 1, 0, 0) | 00011111 | rm(d, 16, 4, 1, 0, c, 4) | 00000000" "asm"="vpcmpd $0, {c}(%rip){lb}1to16{rb}, {n}, {d}" "reloc"="pc32x" "dclass"="kw" "feature"="avx512f" "nodecode" {
  %r = icmp eq <16 x i32> %n, %c
  ret <16 x i1> %r
}

define <16 x i1> @VPCMPDb.ne.16xi32(<16 x i32> "reg"="zv" %n, <16 x i32> "sym" "pool" %c) "enc"="evexm(d, 16, 4, 3, 0, n, 2, 1, 1, 0, 0) | 00011111 | rm(d, 16, 4, 1, 0, c, 4) | 00000100" "asm"="vpcmpd $4, {c}(%rip){lb}1to16{rb}, {n}, {d}" "reloc"="pc32x" "dclass"="kw" "feature"="avx512f" "nodecode" {
  %r = icmp ne <16 x i32> %n, %c
  ret <16 x i1> %r
}

define <16 x i1> @VPCMPDb.slt.16xi32(<16 x i32> "reg"="zv" %n, <16 x i32> "sym" "pool" %c) "enc"="evexm(d, 16, 4, 3, 0, n, 2, 1, 1, 0, 0) | 00011111 | rm(d, 16, 4, 1, 0, c, 4) | 00000001" "asm"="vpcmpd $1, {c}(%rip){lb}1to16{rb}, {n}, {d}" "reloc"="pc32x" "dclass"="kw" "feature"="avx512f" "nodecode" {
  %r = icmp slt <16 x i32> %n, %c
  ret <16 x i1> %r
}

define <16 x i1> @VPCMPDb.sle.16xi32(<16 x i32> "reg"="zv" %n, <16 x i32> "sym" "pool" %c) "enc"="evexm(d, 16, 4, 3, 0, n, 2, 1, 1, 0, 0) | 00011111 | rm(d, 16, 4, 1, 0, c, 4) | 00000010" "asm"="vpcmpd $2, {c}(%rip){lb}1to16{rb}, {n}, {d}" "reloc"="pc32x" "dclass"="kw" "feature"="avx512f" "nodecode" {
  %r = icmp sle <16 x i32> %n, %c
  ret <16 x i1> %r
}

define <16 x i1> @VPCMPDb.sge.16xi32(<16 x i32> "reg"="zv" %n, <16 x i32> "sym" "pool" %c) "enc"="evexm(d, 16, 4, 3, 0, n, 2, 1, 1, 0, 0) | 00011111 | rm(d, 16, 4, 1, 0, c, 4) | 00000101" "asm"="vpcmpd $5, {c}(%rip){lb}1to16{rb}, {n}, {d}" "reloc"="pc32x" "dclass"="kw" "feature"="avx512f" "nodecode" {
  %r = icmp sge <16 x i32> %n, %c
  ret <16 x i1> %r
}

define <16 x i1> @VPCMPDb.sgt.16xi32(<16 x i32> "reg"="zv" %n, <16 x i32> "sym" "pool" %c) "enc"="evexm(d, 16, 4, 3, 0, n, 2, 1, 1, 0, 0) | 00011111 | rm(d, 16, 4, 1, 0, c, 4) | 00000110" "asm"="vpcmpd $6, {c}(%rip){lb}1to16{rb}, {n}, {d}" "reloc"="pc32x" "dclass"="kw" "feature"="avx512f" "nodecode" {
  %r = icmp sgt <16 x i32> %n, %c
  ret <16 x i1> %r
}

define <16 x i1> @VPCMPUDb.ult.16xi32(<16 x i32> "reg"="zv" %n, <16 x i32> "sym" "pool" %c) "enc"="evexm(d, 16, 4, 3, 0, n, 2, 1, 1, 0, 0) | 00011110 | rm(d, 16, 4, 1, 0, c, 4) | 00000001" "asm"="vpcmpud $1, {c}(%rip){lb}1to16{rb}, {n}, {d}" "reloc"="pc32x" "dclass"="kw" "feature"="avx512f" "nodecode" {
  %r = icmp ult <16 x i32> %n, %c
  ret <16 x i1> %r
}

define <16 x i1> @VPCMPUDb.ule.16xi32(<16 x i32> "reg"="zv" %n, <16 x i32> "sym" "pool" %c) "enc"="evexm(d, 16, 4, 3, 0, n, 2, 1, 1, 0, 0) | 00011110 | rm(d, 16, 4, 1, 0, c, 4) | 00000010" "asm"="vpcmpud $2, {c}(%rip){lb}1to16{rb}, {n}, {d}" "reloc"="pc32x" "dclass"="kw" "feature"="avx512f" "nodecode" {
  %r = icmp ule <16 x i32> %n, %c
  ret <16 x i1> %r
}

define <16 x i1> @VPCMPUDb.uge.16xi32(<16 x i32> "reg"="zv" %n, <16 x i32> "sym" "pool" %c) "enc"="evexm(d, 16, 4, 3, 0, n, 2, 1, 1, 0, 0) | 00011110 | rm(d, 16, 4, 1, 0, c, 4) | 00000101" "asm"="vpcmpud $5, {c}(%rip){lb}1to16{rb}, {n}, {d}" "reloc"="pc32x" "dclass"="kw" "feature"="avx512f" "nodecode" {
  %r = icmp uge <16 x i32> %n, %c
  ret <16 x i1> %r
}

define <16 x i1> @VPCMPUDb.ugt.16xi32(<16 x i32> "reg"="zv" %n, <16 x i32> "sym" "pool" %c) "enc"="evexm(d, 16, 4, 3, 0, n, 2, 1, 1, 0, 0) | 00011110 | rm(d, 16, 4, 1, 0, c, 4) | 00000110" "asm"="vpcmpud $6, {c}(%rip){lb}1to16{rb}, {n}, {d}" "reloc"="pc32x" "dclass"="kw" "feature"="avx512f" "nodecode" {
  %r = icmp ugt <16 x i32> %n, %c
  ret <16 x i1> %r
}

; The constant where the mask is set; the register where it is not.
define <16 x i32> @VPBLENDMDb.16xi32(<16 x i1> "reg"="kw" %k, <16 x i32> "sym" "pool" %c, <16 x i32> "reg"="zv" %b) "enc"="{ if(k!=0) evexm(d, 16, 4, 2, 0, b, 2, 1, 1, k, 0) | 01100100 | rm(d, 16, 4, 1, 0, c, 4) }" "asm"="vpblendmd {c}(%rip){lb}1to16{rb}, {b}, {d} {lb}{k}{rb}" "reloc"="pc32x" "dclass"="zv" "feature"="avx512f" "nodecode" {
  %r = select <16 x i1> %k, <16 x i32> %c, <16 x i32> %b
  ret <16 x i32> %r
}

; A three-input bit function whose last input is the constant (bitfn3.rs
; puts a constant last).
define <16 x i32> @VPTERNLOGDb.16xi32(i8 "imm" %t, <16 x i32> "reg"="zv" "tied" %x, <16 x i32> "reg"="zv" %y, <16 x i32> "sym" "pool" %c) "enc"="evexm(d, 16, 4, 3, 0, y, 2, 1, 1, 0, 0) | 00100101 | rm(d, 16, 4, 1, 0, c, 4) | { i1:8=t ; i1:s8=t }" "asm"="vpternlogd ${t}, {c}(%rip){lb}1to16{rb}, {y}, {d}" "reloc"="pc32x" "dclass"="zv" "feature"="avx512f" "nodecode" {
  %r = call <16 x i32> @llvm.epitome.bitfn3.v16i32(i8 %t, <16 x i32> %x, <16 x i32> %y, <16 x i32> %c)
  ret <16 x i32> %r
}
