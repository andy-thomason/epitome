; Scalar ALU, scalar memory and program control. Written by gen_gfx1151.py.
define void @S_LOAD_B32.SMEM(i32 "enum"="e1" %sdata, i32 "enum"="e27" %sbase, i32 "enum"="e1" %soffset, i32 "enum"="e28" %cpol) "enc"="{ if((sdata<125)|(sdata>208)) if((sdata<235)|(sdata>238)) if((sdata<240)|(sdata>248)) if((sdata<253)|(sdata>254)) if((sdata<256)|(sdata>511)) if((soffset<128)|(soffset>208)) if((soffset<235)|(soffset>238)) if((soffset<240)|(soffset>248)) if((soffset<253)|(soffset>254)) if((soffset<256)|(soffset>511)) soffset_57:7=soffset 000000000000000000000000011110100000000000 glc_14:1=cpol&1 dlc_13:1=(cpol>>1)&1 sdata_6:7=sdata sbase_0:6=sbase }" "asm"="s_load_b32 {sdata}, {sbase}, {soffset}{cpol}" "asmonly" {
  ret void
}
define void @S_LOAD_B32.SMEM.zero(i32 "enum"="e1" %sdata, i32 "enum"="e27" %sbase, i32 "enum"="e28" %cpol) "enc"="{ if((sdata<125)|(sdata>208)) if((sdata<235)|(sdata>238)) if((sdata<240)|(sdata>248)) if((sdata<253)|(sdata>254)) if((sdata<256)|(sdata>511)) 1111100 000000000000000000000000011110100000000000 glc_14:1=cpol&1 dlc_13:1=(cpol>>1)&1 sdata_6:7=sdata sbase_0:6=sbase }" "asm"="s_load_b32 {sdata}, {sbase}, 0x0{cpol}" "asmonly" "nodecode" {
  ret void
}
define void @S_LOAD_B32.SMEM.offset(i32 "enum"="e1" %sdata, i32 "enum"="e27" %sbase, i32 "enum"="e29" %soffset, i32 "enum"="e28" %cpol, i64 "imm" %offset) "enc"="{ if(offset!=0) if((sdata<125)|(sdata>208)) if((sdata<235)|(sdata>238)) if((sdata<240)|(sdata>248)) if((sdata<253)|(sdata>254)) if((sdata<256)|(sdata>511)) soffset_57:7=soffset 0000 offset_32:s21=offset 11110100000000000 glc_14:1=cpol&1 dlc_13:1=(cpol>>1)&1 sdata_6:7=sdata sbase_0:6=sbase }" "asm"="s_load_b32 {sdata}, {sbase}, {soffset}{offset:sx}{cpol}" "asmonly" {
  ret void
}
define void @S_LOAD_B64.SMEM(i32 "enum"="e18" %sdata, i32 "enum"="e27" %sbase, i32 "enum"="e1" %soffset, i32 "enum"="e28" %cpol) "enc"="{ if(sdata!=126) if((sdata<128)|(sdata>208)) if((sdata<235)|(sdata>238)) if((sdata<240)|(sdata>248)) if(sdata!=253) if((sdata<256)|(sdata>510)) if((soffset<128)|(soffset>208)) if((soffset<235)|(soffset>238)) if((soffset<240)|(soffset>248)) if((soffset<253)|(soffset>254)) if((soffset<256)|(soffset>511)) soffset_57:7=soffset 000000000000000000000000011110100000001000 glc_14:1=cpol&1 dlc_13:1=(cpol>>1)&1 sdata_6:7=sdata sbase_0:6=sbase }" "asm"="s_load_b64 {sdata}, {sbase}, {soffset}{cpol}" "asmonly" {
  ret void
}
define void @S_LOAD_B64.SMEM.zero(i32 "enum"="e18" %sdata, i32 "enum"="e27" %sbase, i32 "enum"="e28" %cpol) "enc"="{ if(sdata!=126) if((sdata<128)|(sdata>208)) if((sdata<235)|(sdata>238)) if((sdata<240)|(sdata>248)) if(sdata!=253) if((sdata<256)|(sdata>510)) 1111100 000000000000000000000000011110100000001000 glc_14:1=cpol&1 dlc_13:1=(cpol>>1)&1 sdata_6:7=sdata sbase_0:6=sbase }" "asm"="s_load_b64 {sdata}, {sbase}, 0x0{cpol}" "asmonly" "nodecode" {
  ret void
}
define void @S_LOAD_B64.SMEM.offset(i32 "enum"="e18" %sdata, i32 "enum"="e27" %sbase, i32 "enum"="e29" %soffset, i32 "enum"="e28" %cpol, i64 "imm" %offset) "enc"="{ if(offset!=0) if(sdata!=126) if((sdata<128)|(sdata>208)) if((sdata<235)|(sdata>238)) if((sdata<240)|(sdata>248)) if(sdata!=253) if((sdata<256)|(sdata>510)) soffset_57:7=soffset 0000 offset_32:s21=offset 11110100000001000 glc_14:1=cpol&1 dlc_13:1=(cpol>>1)&1 sdata_6:7=sdata sbase_0:6=sbase }" "asm"="s_load_b64 {sdata}, {sbase}, {soffset}{offset:sx}{cpol}" "asmonly" {
  ret void
}
define void @S_LOAD_B128.SMEM(i32 "enum"="e30" %sdata, i32 "enum"="e27" %sbase, i32 "enum"="e1" %soffset, i32 "enum"="e28" %cpol) "enc"="{ if((soffset<128)|(soffset>208)) if((soffset<235)|(soffset>238)) if((soffset<240)|(soffset>248)) if((soffset<253)|(soffset>254)) if((soffset<256)|(soffset>511)) soffset_57:7=soffset 000000000000000000000000011110100000010000 glc_14:1=cpol&1 dlc_13:1=(cpol>>1)&1 sdata_6:7=sdata sbase_0:6=sbase }" "asm"="s_load_b128 {sdata}, {sbase}, {soffset}{cpol}" "asmonly" {
  ret void
}
define void @S_LOAD_B128.SMEM.zero(i32 "enum"="e30" %sdata, i32 "enum"="e27" %sbase, i32 "enum"="e28" %cpol) "enc"="1111100 000000000000000000000000011110100000010000 glc_14:1=cpol&1 dlc_13:1=(cpol>>1)&1 sdata_6:7=sdata sbase_0:6=sbase" "asm"="s_load_b128 {sdata}, {sbase}, 0x0{cpol}" "asmonly" "nodecode" {
  ret void
}
define void @S_LOAD_B128.SMEM.offset(i32 "enum"="e30" %sdata, i32 "enum"="e27" %sbase, i32 "enum"="e29" %soffset, i32 "enum"="e28" %cpol, i64 "imm" %offset) "enc"="{ if(offset!=0) soffset_57:7=soffset 0000 offset_32:s21=offset 11110100000010000 glc_14:1=cpol&1 dlc_13:1=(cpol>>1)&1 sdata_6:7=sdata sbase_0:6=sbase }" "asm"="s_load_b128 {sdata}, {sbase}, {soffset}{offset:sx}{cpol}" "asmonly" {
  ret void
}
define void @S_LOAD_B256.SMEM(i32 "enum"="e31" %sdata, i32 "enum"="e27" %sbase, i32 "enum"="e1" %soffset, i32 "enum"="e28" %cpol) "enc"="{ if((soffset<128)|(soffset>208)) if((soffset<235)|(soffset>238)) if((soffset<240)|(soffset>248)) if((soffset<253)|(soffset>254)) if((soffset<256)|(soffset>511)) soffset_57:7=soffset 000000000000000000000000011110100000011000 glc_14:1=cpol&1 dlc_13:1=(cpol>>1)&1 sdata_6:7=sdata sbase_0:6=sbase }" "asm"="s_load_b256 {sdata}, {sbase}, {soffset}{cpol}" "asmonly" {
  ret void
}
define void @S_LOAD_B256.SMEM.zero(i32 "enum"="e31" %sdata, i32 "enum"="e27" %sbase, i32 "enum"="e28" %cpol) "enc"="1111100 000000000000000000000000011110100000011000 glc_14:1=cpol&1 dlc_13:1=(cpol>>1)&1 sdata_6:7=sdata sbase_0:6=sbase" "asm"="s_load_b256 {sdata}, {sbase}, 0x0{cpol}" "asmonly" "nodecode" {
  ret void
}
define void @S_LOAD_B256.SMEM.offset(i32 "enum"="e31" %sdata, i32 "enum"="e27" %sbase, i32 "enum"="e29" %soffset, i32 "enum"="e28" %cpol, i64 "imm" %offset) "enc"="{ if(offset!=0) soffset_57:7=soffset 0000 offset_32:s21=offset 11110100000011000 glc_14:1=cpol&1 dlc_13:1=(cpol>>1)&1 sdata_6:7=sdata sbase_0:6=sbase }" "asm"="s_load_b256 {sdata}, {sbase}, {soffset}{offset:sx}{cpol}" "asmonly" {
  ret void
}
define void @S_LOAD_B512.SMEM(i32 "enum"="e32" %sdata, i32 "enum"="e27" %sbase, i32 "enum"="e1" %soffset, i32 "enum"="e28" %cpol) "enc"="{ if((soffset<128)|(soffset>208)) if((soffset<235)|(soffset>238)) if((soffset<240)|(soffset>248)) if((soffset<253)|(soffset>254)) if((soffset<256)|(soffset>511)) soffset_57:7=soffset 000000000000000000000000011110100000100000 glc_14:1=cpol&1 dlc_13:1=(cpol>>1)&1 sdata_6:7=sdata sbase_0:6=sbase }" "asm"="s_load_b512 {sdata}, {sbase}, {soffset}{cpol}" "asmonly" {
  ret void
}
define void @S_LOAD_B512.SMEM.zero(i32 "enum"="e32" %sdata, i32 "enum"="e27" %sbase, i32 "enum"="e28" %cpol) "enc"="1111100 000000000000000000000000011110100000100000 glc_14:1=cpol&1 dlc_13:1=(cpol>>1)&1 sdata_6:7=sdata sbase_0:6=sbase" "asm"="s_load_b512 {sdata}, {sbase}, 0x0{cpol}" "asmonly" "nodecode" {
  ret void
}
define void @S_LOAD_B512.SMEM.offset(i32 "enum"="e32" %sdata, i32 "enum"="e27" %sbase, i32 "enum"="e29" %soffset, i32 "enum"="e28" %cpol, i64 "imm" %offset) "enc"="{ if(offset!=0) soffset_57:7=soffset 0000 offset_32:s21=offset 11110100000100000 glc_14:1=cpol&1 dlc_13:1=(cpol>>1)&1 sdata_6:7=sdata sbase_0:6=sbase }" "asm"="s_load_b512 {sdata}, {sbase}, {soffset}{offset:sx}{cpol}" "asmonly" {
  ret void
}
define void @S_BUFFER_LOAD_B32.SMEM(i32 "enum"="e1" %sdata, i32 "enum"="e33" %sbase, i32 "enum"="e1" %soffset, i32 "enum"="e28" %cpol) "enc"="{ if((sdata<125)|(sdata>208)) if((sdata<235)|(sdata>238)) if((sdata<240)|(sdata>248)) if((sdata<253)|(sdata>254)) if((sdata<256)|(sdata>511)) if((soffset<128)|(soffset>208)) if((soffset<235)|(soffset>238)) if((soffset<240)|(soffset>248)) if((soffset<253)|(soffset>254)) if((soffset<256)|(soffset>511)) soffset_57:7=soffset 000000000000000000000000011110100001000000 glc_14:1=cpol&1 dlc_13:1=(cpol>>1)&1 sdata_6:7=sdata sbase_0:6=sbase }" "asm"="s_buffer_load_b32 {sdata}, {sbase}, {soffset}{cpol}" "asmonly" {
  ret void
}
define void @S_BUFFER_LOAD_B32.SMEM.zero(i32 "enum"="e1" %sdata, i32 "enum"="e33" %sbase, i32 "enum"="e28" %cpol) "enc"="{ if((sdata<125)|(sdata>208)) if((sdata<235)|(sdata>238)) if((sdata<240)|(sdata>248)) if((sdata<253)|(sdata>254)) if((sdata<256)|(sdata>511)) 1111100 000000000000000000000000011110100001000000 glc_14:1=cpol&1 dlc_13:1=(cpol>>1)&1 sdata_6:7=sdata sbase_0:6=sbase }" "asm"="s_buffer_load_b32 {sdata}, {sbase}, 0x0{cpol}" "asmonly" "nodecode" {
  ret void
}
define void @S_BUFFER_LOAD_B32.SMEM.offset(i32 "enum"="e1" %sdata, i32 "enum"="e33" %sbase, i32 "enum"="e29" %soffset, i32 "enum"="e28" %cpol, i64 "imm" %offset) "enc"="{ if(offset!=0) if((sdata<125)|(sdata>208)) if((sdata<235)|(sdata>238)) if((sdata<240)|(sdata>248)) if((sdata<253)|(sdata>254)) if((sdata<256)|(sdata>511)) soffset_57:7=soffset 0000 offset_32:s21=offset 11110100001000000 glc_14:1=cpol&1 dlc_13:1=(cpol>>1)&1 sdata_6:7=sdata sbase_0:6=sbase }" "asm"="s_buffer_load_b32 {sdata}, {sbase}, {soffset}{offset:sx}{cpol}" "asmonly" {
  ret void
}
define void @S_BUFFER_LOAD_B64.SMEM(i32 "enum"="e18" %sdata, i32 "enum"="e33" %sbase, i32 "enum"="e1" %soffset, i32 "enum"="e28" %cpol) "enc"="{ if(sdata!=126) if((sdata<128)|(sdata>208)) if((sdata<235)|(sdata>238)) if((sdata<240)|(sdata>248)) if(sdata!=253) if((sdata<256)|(sdata>510)) if(sbase!=48) if((soffset<128)|(soffset>208)) if((soffset<235)|(soffset>238)) if((soffset<240)|(soffset>248)) if((soffset<253)|(soffset>254)) if((soffset<256)|(soffset>511)) soffset_57:7=soffset 000000000000000000000000011110100001001000 glc_14:1=cpol&1 dlc_13:1=(cpol>>1)&1 sdata_6:7=sdata sbase_0:6=sbase }" "asm"="s_buffer_load_b64 {sdata}, {sbase}, {soffset}{cpol}" "asmonly" {
  ret void
}
define void @S_BUFFER_LOAD_B64.SMEM.zero(i32 "enum"="e18" %sdata, i32 "enum"="e33" %sbase, i32 "enum"="e28" %cpol) "enc"="{ if(sdata!=126) if((sdata<128)|(sdata>208)) if((sdata<235)|(sdata>238)) if((sdata<240)|(sdata>248)) if(sdata!=253) if((sdata<256)|(sdata>510)) if(sbase!=48) 1111100 000000000000000000000000011110100001001000 glc_14:1=cpol&1 dlc_13:1=(cpol>>1)&1 sdata_6:7=sdata sbase_0:6=sbase }" "asm"="s_buffer_load_b64 {sdata}, {sbase}, 0x0{cpol}" "asmonly" "nodecode" {
  ret void
}
define void @S_BUFFER_LOAD_B64.SMEM.offset(i32 "enum"="e18" %sdata, i32 "enum"="e33" %sbase, i32 "enum"="e29" %soffset, i32 "enum"="e28" %cpol, i64 "imm" %offset) "enc"="{ if(offset!=0) if(sdata!=126) if((sdata<128)|(sdata>208)) if((sdata<235)|(sdata>238)) if((sdata<240)|(sdata>248)) if(sdata!=253) if((sdata<256)|(sdata>510)) soffset_57:7=soffset 0000 offset_32:s21=offset 11110100001001000 glc_14:1=cpol&1 dlc_13:1=(cpol>>1)&1 sdata_6:7=sdata sbase_0:6=sbase }" "asm"="s_buffer_load_b64 {sdata}, {sbase}, {soffset}{offset:sx}{cpol}" "asmonly" {
  ret void
}
define void @S_BUFFER_LOAD_B128.SMEM(i32 "enum"="e30" %sdata, i32 "enum"="e33" %sbase, i32 "enum"="e1" %soffset, i32 "enum"="e28" %cpol) "enc"="{ if((soffset<128)|(soffset>208)) if((soffset<235)|(soffset>238)) if((soffset<240)|(soffset>248)) if((soffset<253)|(soffset>254)) if((soffset<256)|(soffset>511)) soffset_57:7=soffset 000000000000000000000000011110100001010000 glc_14:1=cpol&1 dlc_13:1=(cpol>>1)&1 sdata_6:7=sdata sbase_0:6=sbase }" "asm"="s_buffer_load_b128 {sdata}, {sbase}, {soffset}{cpol}" "asmonly" {
  ret void
}
define void @S_BUFFER_LOAD_B128.SMEM.zero(i32 "enum"="e30" %sdata, i32 "enum"="e33" %sbase, i32 "enum"="e28" %cpol) "enc"="1111100 000000000000000000000000011110100001010000 glc_14:1=cpol&1 dlc_13:1=(cpol>>1)&1 sdata_6:7=sdata sbase_0:6=sbase" "asm"="s_buffer_load_b128 {sdata}, {sbase}, 0x0{cpol}" "asmonly" "nodecode" {
  ret void
}
define void @S_BUFFER_LOAD_B128.SMEM.offset(i32 "enum"="e30" %sdata, i32 "enum"="e33" %sbase, i32 "enum"="e29" %soffset, i32 "enum"="e28" %cpol, i64 "imm" %offset) "enc"="{ if(offset!=0) soffset_57:7=soffset 0000 offset_32:s21=offset 11110100001010000 glc_14:1=cpol&1 dlc_13:1=(cpol>>1)&1 sdata_6:7=sdata sbase_0:6=sbase }" "asm"="s_buffer_load_b128 {sdata}, {sbase}, {soffset}{offset:sx}{cpol}" "asmonly" {
  ret void
}
define void @S_BUFFER_LOAD_B256.SMEM(i32 "enum"="e31" %sdata, i32 "enum"="e33" %sbase, i32 "enum"="e1" %soffset, i32 "enum"="e28" %cpol) "enc"="{ if((soffset<128)|(soffset>208)) if((soffset<235)|(soffset>238)) if((soffset<240)|(soffset>248)) if((soffset<253)|(soffset>254)) if((soffset<256)|(soffset>511)) soffset_57:7=soffset 000000000000000000000000011110100001011000 glc_14:1=cpol&1 dlc_13:1=(cpol>>1)&1 sdata_6:7=sdata sbase_0:6=sbase }" "asm"="s_buffer_load_b256 {sdata}, {sbase}, {soffset}{cpol}" "asmonly" {
  ret void
}
define void @S_BUFFER_LOAD_B256.SMEM.zero(i32 "enum"="e31" %sdata, i32 "enum"="e33" %sbase, i32 "enum"="e28" %cpol) "enc"="1111100 000000000000000000000000011110100001011000 glc_14:1=cpol&1 dlc_13:1=(cpol>>1)&1 sdata_6:7=sdata sbase_0:6=sbase" "asm"="s_buffer_load_b256 {sdata}, {sbase}, 0x0{cpol}" "asmonly" "nodecode" {
  ret void
}
define void @S_BUFFER_LOAD_B256.SMEM.offset(i32 "enum"="e31" %sdata, i32 "enum"="e33" %sbase, i32 "enum"="e29" %soffset, i32 "enum"="e28" %cpol, i64 "imm" %offset) "enc"="{ if(offset!=0) soffset_57:7=soffset 0000 offset_32:s21=offset 11110100001011000 glc_14:1=cpol&1 dlc_13:1=(cpol>>1)&1 sdata_6:7=sdata sbase_0:6=sbase }" "asm"="s_buffer_load_b256 {sdata}, {sbase}, {soffset}{offset:sx}{cpol}" "asmonly" {
  ret void
}
define void @S_BUFFER_LOAD_B512.SMEM(i32 "enum"="e32" %sdata, i32 "enum"="e33" %sbase, i32 "enum"="e1" %soffset, i32 "enum"="e28" %cpol) "enc"="{ if((soffset<128)|(soffset>208)) if((soffset<235)|(soffset>238)) if((soffset<240)|(soffset>248)) if((soffset<253)|(soffset>254)) if((soffset<256)|(soffset>511)) soffset_57:7=soffset 000000000000000000000000011110100001100000 glc_14:1=cpol&1 dlc_13:1=(cpol>>1)&1 sdata_6:7=sdata sbase_0:6=sbase }" "asm"="s_buffer_load_b512 {sdata}, {sbase}, {soffset}{cpol}" "asmonly" {
  ret void
}
define void @S_BUFFER_LOAD_B512.SMEM.zero(i32 "enum"="e32" %sdata, i32 "enum"="e33" %sbase, i32 "enum"="e28" %cpol) "enc"="1111100 000000000000000000000000011110100001100000 glc_14:1=cpol&1 dlc_13:1=(cpol>>1)&1 sdata_6:7=sdata sbase_0:6=sbase" "asm"="s_buffer_load_b512 {sdata}, {sbase}, 0x0{cpol}" "asmonly" "nodecode" {
  ret void
}
define void @S_BUFFER_LOAD_B512.SMEM.offset(i32 "enum"="e32" %sdata, i32 "enum"="e33" %sbase, i32 "enum"="e29" %soffset, i32 "enum"="e28" %cpol, i64 "imm" %offset) "enc"="{ if(offset!=0) soffset_57:7=soffset 0000 offset_32:s21=offset 11110100001100000 glc_14:1=cpol&1 dlc_13:1=(cpol>>1)&1 sdata_6:7=sdata sbase_0:6=sbase }" "asm"="s_buffer_load_b512 {sdata}, {sbase}, {soffset}{offset:sx}{cpol}" "asmonly" {
  ret void
}
define void @S_GL1_INV.SMEM() "enc"="0000000000000000000000000000000011110100100000000000000000000000" "asm"="s_gl1_inv" "asmonly" {
  ret void
}
define void @S_DCACHE_INV.SMEM() "enc"="0000000000000000000000000000000011110100100001000000000000000000" "asm"="s_dcache_inv" "asmonly" {
  ret void
}
define void @S_ATC_PROBE.SMEM(i32 "enum"="e34" %sdata, i32 "enum"="e27" %sbase, i32 "enum"="e1" %soffset) "enc"="{ if((soffset<128)|(soffset>208)) if((soffset<235)|(soffset>238)) if((soffset<240)|(soffset>248)) if((soffset<253)|(soffset>254)) if((soffset<256)|(soffset>511)) soffset_57:7=soffset 00000000000000000000000001111010010001000000 sdata_6:7=sdata sbase_0:6=sbase }" "asm"="s_atc_probe {sdata}, {sbase}, {soffset}" "asmonly" {
  ret void
}
define void @S_ATC_PROBE.SMEM.offset(i32 "enum"="e34" %sdata, i32 "enum"="e27" %sbase, i32 "enum"="e29" %soffset, i64 "imm" %offset) "enc"="{ if(offset!=0) soffset_57:7=soffset 0000 offset_32:s21=offset 1111010010001000000 sdata_6:7=sdata sbase_0:6=sbase }" "asm"="s_atc_probe {sdata}, {sbase}, {soffset}{offset:sx}" "asmonly" {
  ret void
}
define void @S_ATC_PROBE_BUFFER.SMEM(i32 "enum"="e34" %sdata, i32 "enum"="e33" %sbase, i32 "enum"="e1" %soffset) "enc"="{ if((soffset<128)|(soffset>208)) if((soffset<235)|(soffset>238)) if((soffset<240)|(soffset>248)) if((soffset<253)|(soffset>254)) if((soffset<256)|(soffset>511)) soffset_57:7=soffset 00000000000000000000000001111010010001100000 sdata_6:7=sdata sbase_0:6=sbase }" "asm"="s_atc_probe_buffer {sdata}, {sbase}, {soffset}" "asmonly" {
  ret void
}
define void @S_ATC_PROBE_BUFFER.SMEM.offset(i32 "enum"="e34" %sdata, i32 "enum"="e33" %sbase, i32 "enum"="e29" %soffset, i64 "imm" %offset) "enc"="{ if(offset!=0) soffset_57:7=soffset 0000 offset_32:s21=offset 1111010010001100000 sdata_6:7=sdata sbase_0:6=sbase }" "asm"="s_atc_probe_buffer {sdata}, {sbase}, {soffset}{offset:sx}" "asmonly" {
  ret void
}
define void @S_MOV_B32.SOP1.Notlit_0_Notlit_1(i32 "enum"="e1" %sdst, i32 "enum"="e3" %ssrc0) "enc"="{ if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) 101111101 sdst_16:7=sdst 00000000 ssrc0_0:8=ssrc0 }" "asm"="s_mov_b32 {sdst}, {ssrc0}" "asmonly" {
  ret void
}
define void @S_MOV_B32.SOP1.lit.lit_0(i32 "enum"="e1" %sdst, i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) literal_32:32=lit 101111101 sdst_16:7=sdst 0000000011111111 }" "asm"="s_mov_b32 {sdst}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_MOV_B64.SOP1.Notlit_0_Notlit_1(i32 "enum"="e18" %sdst, i32 "enum"="e18" %ssrc0) "enc"="{ if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if(sdst!=253) if((sdst<256)|(sdst>510)) if((ssrc0<256)|(ssrc0>510)) 101111101 sdst_16:7=sdst 00000001 ssrc0_0:8=ssrc0 }" "asm"="s_mov_b64 {sdst}, {ssrc0}" "asmonly" {
  ret void
}
define void @S_MOV_B64.SOP1.lit.lit_0(i32 "enum"="e18" %sdst, i64 "imm" %lit) "enc"="{ if(lit>64) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if(sdst!=253) if((sdst<256)|(sdst>510)) literal_32:32=lit 101111101 sdst_16:7=sdst 0000000111111111 }" "asm"="s_mov_b64 {sdst}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_CMOV_B32.SOP1.Notlit_0_Notlit_1(i32 "enum"="e1" %sdst, i32 "enum"="e3" %ssrc0) "enc"="{ if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) 101111101 sdst_16:7=sdst 00000010 ssrc0_0:8=ssrc0 }" "asm"="s_cmov_b32 {sdst}, {ssrc0}" "asmonly" {
  ret void
}
define void @S_CMOV_B32.SOP1.lit.lit_0(i32 "enum"="e1" %sdst, i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) literal_32:32=lit 101111101 sdst_16:7=sdst 0000001011111111 }" "asm"="s_cmov_b32 {sdst}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_CMOV_B64.SOP1.Notlit_0_Notlit_1(i32 "enum"="e18" %sdst, i32 "enum"="e18" %ssrc0) "enc"="{ if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if(sdst!=253) if((sdst<256)|(sdst>510)) if((ssrc0<256)|(ssrc0>510)) 101111101 sdst_16:7=sdst 00000011 ssrc0_0:8=ssrc0 }" "asm"="s_cmov_b64 {sdst}, {ssrc0}" "asmonly" {
  ret void
}
define void @S_CMOV_B64.SOP1.lit.lit_0(i32 "enum"="e18" %sdst, i64 "imm" %lit) "enc"="{ if(lit>64) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if(sdst!=253) if((sdst<256)|(sdst>510)) literal_32:32=lit 101111101 sdst_16:7=sdst 0000001111111111 }" "asm"="s_cmov_b64 {sdst}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_BREV_B32.SOP1.Notlit_0_Notlit_1(i32 "enum"="e1" %sdst, i32 "enum"="e3" %ssrc0) "enc"="{ if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if(ssrc0!=39) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) 101111101 sdst_16:7=sdst 00000100 ssrc0_0:8=ssrc0 }" "asm"="s_brev_b32 {sdst}, {ssrc0}" "asmonly" {
  ret void
}
define void @S_BREV_B32.SOP1.lit.lit_0(i32 "enum"="e1" %sdst, i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) literal_32:32=lit 101111101 sdst_16:7=sdst 0000010011111111 }" "asm"="s_brev_b32 {sdst}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_BREV_B64.SOP1.Notlit_0_Notlit_1(i32 "enum"="e18" %sdst, i32 "enum"="e18" %ssrc0) "enc"="{ if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if(sdst!=253) if((sdst<256)|(sdst>510)) if((ssrc0<256)|(ssrc0>510)) 101111101 sdst_16:7=sdst 00000101 ssrc0_0:8=ssrc0 }" "asm"="s_brev_b64 {sdst}, {ssrc0}" "asmonly" {
  ret void
}
define void @S_BREV_B64.SOP1.lit.lit_0(i32 "enum"="e18" %sdst, i64 "imm" %lit) "enc"="{ if(lit>64) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if(sdst!=253) if((sdst<256)|(sdst>510)) literal_32:32=lit 101111101 sdst_16:7=sdst 0000010111111111 }" "asm"="s_brev_b64 {sdst}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_CTZ_I32_B32.SOP1.Notlit_0_Notlit_1(i32 "enum"="e1" %sdst, i32 "enum"="e3" %ssrc0) "enc"="{ if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) 101111101 sdst_16:7=sdst 00001000 ssrc0_0:8=ssrc0 }" "asm"="s_ctz_i32_b32 {sdst}, {ssrc0}" "asmonly" {
  ret void
}
define void @S_CTZ_I32_B32.SOP1.lit.lit_0(i32 "enum"="e1" %sdst, i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) literal_32:32=lit 101111101 sdst_16:7=sdst 0000100011111111 }" "asm"="s_ctz_i32_b32 {sdst}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_CTZ_I32_B64.SOP1.Notlit_0_Notlit_1(i32 "enum"="e1" %sdst, i32 "enum"="e18" %ssrc0) "enc"="{ if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if(ssrc0!=199) if((ssrc0<256)|(ssrc0>510)) 101111101 sdst_16:7=sdst 00001001 ssrc0_0:8=ssrc0 }" "asm"="s_ctz_i32_b64 {sdst}, {ssrc0}" "asmonly" {
  ret void
}
define void @S_CTZ_I32_B64.SOP1.lit.lit_0(i32 "enum"="e1" %sdst, i64 "imm" %lit) "enc"="{ if(lit>64) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) literal_32:32=lit 101111101 sdst_16:7=sdst 0000100111111111 }" "asm"="s_ctz_i32_b64 {sdst}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_CLZ_I32_U32.SOP1.Notlit_0_Notlit_1(i32 "enum"="e1" %sdst, i32 "enum"="e3" %ssrc0) "enc"="{ if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) 101111101 sdst_16:7=sdst 00001010 ssrc0_0:8=ssrc0 }" "asm"="s_clz_i32_u32 {sdst}, {ssrc0}" "asmonly" {
  ret void
}
define void @S_CLZ_I32_U32.SOP1.lit.lit_0(i32 "enum"="e1" %sdst, i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) literal_32:32=lit 101111101 sdst_16:7=sdst 0000101011111111 }" "asm"="s_clz_i32_u32 {sdst}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_CLZ_I32_U64.SOP1.Notlit_0_Notlit_1(i32 "enum"="e1" %sdst, i32 "enum"="e18" %ssrc0) "enc"="{ if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc0<256)|(ssrc0>510)) 101111101 sdst_16:7=sdst 00001011 ssrc0_0:8=ssrc0 }" "asm"="s_clz_i32_u64 {sdst}, {ssrc0}" "asmonly" {
  ret void
}
define void @S_CLZ_I32_U64.SOP1.lit.lit_0(i32 "enum"="e1" %sdst, i64 "imm" %lit) "enc"="{ if(lit>64) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) literal_32:32=lit 101111101 sdst_16:7=sdst 0000101111111111 }" "asm"="s_clz_i32_u64 {sdst}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_CLS_I32.SOP1.Notlit_0_Notlit_1(i32 "enum"="e1" %sdst, i32 "enum"="e3" %ssrc0) "enc"="{ if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) 101111101 sdst_16:7=sdst 00001100 ssrc0_0:8=ssrc0 }" "asm"="s_cls_i32 {sdst}, {ssrc0}" "asmonly" {
  ret void
}
define void @S_CLS_I32.SOP1.lit.lit_0(i32 "enum"="e1" %sdst, i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) literal_32:32=lit 101111101 sdst_16:7=sdst 0000110011111111 }" "asm"="s_cls_i32 {sdst}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_CLS_I32_I64.SOP1.Notlit_0_Notlit_1(i32 "enum"="e1" %sdst, i32 "enum"="e18" %ssrc0) "enc"="{ if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc0<256)|(ssrc0>510)) 101111101 sdst_16:7=sdst 00001101 ssrc0_0:8=ssrc0 }" "asm"="s_cls_i32_i64 {sdst}, {ssrc0}" "asmonly" {
  ret void
}
define void @S_CLS_I32_I64.SOP1.lit.lit_0(i32 "enum"="e1" %sdst, i64 "imm" %lit) "enc"="{ if(lit>64) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) literal_32:32=lit 101111101 sdst_16:7=sdst 0000110111111111 }" "asm"="s_cls_i32_i64 {sdst}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_SEXT_I32_I8.SOP1.Notlit_0_Notlit_1(i32 "enum"="e1" %sdst, i32 "enum"="e3" %ssrc0) "enc"="{ if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) 101111101 sdst_16:7=sdst 00001110 ssrc0_0:8=ssrc0 }" "asm"="s_sext_i32_i8 {sdst}, {ssrc0}" "asmonly" {
  ret void
}
define void @S_SEXT_I32_I8.SOP1.lit.lit_0(i32 "enum"="e1" %sdst, i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) literal_32:32=lit 101111101 sdst_16:7=sdst 0000111011111111 }" "asm"="s_sext_i32_i8 {sdst}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_SEXT_I32_I16.SOP1.Notlit_0_Notlit_1(i32 "enum"="e1" %sdst, i32 "enum"="e3" %ssrc0) "enc"="{ if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) 101111101 sdst_16:7=sdst 00001111 ssrc0_0:8=ssrc0 }" "asm"="s_sext_i32_i16 {sdst}, {ssrc0}" "asmonly" {
  ret void
}
define void @S_SEXT_I32_I16.SOP1.lit.lit_0(i32 "enum"="e1" %sdst, i64 "imm" %lit) "enc"="{ if(lit<65536) if(lit>64) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) literal_32:32=lit 101111101 sdst_16:7=sdst 0000111111111111 }" "asm"="s_sext_i32_i16 {sdst}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_BITSET0_B32.SOP1.Notlit_0_Notlit_1(i32 "enum"="e1" %sdst, i32 "enum"="e3" %ssrc0) "enc"="{ if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) 101111101 sdst_16:7=sdst 00010000 ssrc0_0:8=ssrc0 }" "asm"="s_bitset0_b32 {sdst}, {ssrc0}" "asmonly" {
  ret void
}
define void @S_BITSET0_B32.SOP1.lit.lit_0(i32 "enum"="e1" %sdst, i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) literal_32:32=lit 101111101 sdst_16:7=sdst 0001000011111111 }" "asm"="s_bitset0_b32 {sdst}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_BITSET0_B64.SOP1.Notlit_0_Notlit_1(i32 "enum"="e18" %sdst, i32 "enum"="e3" %ssrc0) "enc"="{ if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if(sdst!=253) if((sdst<256)|(sdst>510)) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) 101111101 sdst_16:7=sdst 00010001 ssrc0_0:8=ssrc0 }" "asm"="s_bitset0_b64 {sdst}, {ssrc0}" "asmonly" {
  ret void
}
define void @S_BITSET0_B64.SOP1.lit.lit_0(i32 "enum"="e18" %sdst, i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if(sdst!=253) if((sdst<256)|(sdst>510)) literal_32:32=lit 101111101 sdst_16:7=sdst 0001000111111111 }" "asm"="s_bitset0_b64 {sdst}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_BITSET1_B32.SOP1.Notlit_0_Notlit_1(i32 "enum"="e1" %sdst, i32 "enum"="e3" %ssrc0) "enc"="{ if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) 101111101 sdst_16:7=sdst 00010010 ssrc0_0:8=ssrc0 }" "asm"="s_bitset1_b32 {sdst}, {ssrc0}" "asmonly" {
  ret void
}
define void @S_BITSET1_B32.SOP1.lit.lit_0(i32 "enum"="e1" %sdst, i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) literal_32:32=lit 101111101 sdst_16:7=sdst 0001001011111111 }" "asm"="s_bitset1_b32 {sdst}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_BITSET1_B64.SOP1.Notlit_0_Notlit_1(i32 "enum"="e18" %sdst, i32 "enum"="e3" %ssrc0) "enc"="{ if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if(sdst!=253) if((sdst<256)|(sdst>510)) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) 101111101 sdst_16:7=sdst 00010011 ssrc0_0:8=ssrc0 }" "asm"="s_bitset1_b64 {sdst}, {ssrc0}" "asmonly" {
  ret void
}
define void @S_BITSET1_B64.SOP1.lit.lit_0(i32 "enum"="e18" %sdst, i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if(sdst!=253) if((sdst<256)|(sdst>510)) literal_32:32=lit 101111101 sdst_16:7=sdst 0001001111111111 }" "asm"="s_bitset1_b64 {sdst}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_BITREPLICATE_B64_B32.SOP1.Notlit_0_Notlit_1(i32 "enum"="e18" %sdst, i32 "enum"="e3" %ssrc0) "enc"="{ if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if(sdst!=253) if((sdst<256)|(sdst>510)) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) 101111101 sdst_16:7=sdst 00010100 ssrc0_0:8=ssrc0 }" "asm"="s_bitreplicate_b64_b32 {sdst}, {ssrc0}" "asmonly" {
  ret void
}
define void @S_BITREPLICATE_B64_B32.SOP1.lit.lit_0(i32 "enum"="e18" %sdst, i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if(sdst!=253) if((sdst<256)|(sdst>510)) literal_32:32=lit 101111101 sdst_16:7=sdst 0001010011111111 }" "asm"="s_bitreplicate_b64_b32 {sdst}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_ABS_I32.SOP1.Notlit_0_Notlit_1(i32 "enum"="e1" %sdst, i32 "enum"="e3" %ssrc0) "enc"="{ if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) 101111101 sdst_16:7=sdst 00010101 ssrc0_0:8=ssrc0 }" "asm"="s_abs_i32 {sdst}, {ssrc0}" "asmonly" {
  ret void
}
define void @S_ABS_I32.SOP1.lit.lit_0(i32 "enum"="e1" %sdst, i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) literal_32:32=lit 101111101 sdst_16:7=sdst 0001010111111111 }" "asm"="s_abs_i32 {sdst}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_BCNT0_I32_B32.SOP1.Notlit_0_Notlit_1(i32 "enum"="e1" %sdst, i32 "enum"="e3" %ssrc0) "enc"="{ if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) 101111101 sdst_16:7=sdst 00010110 ssrc0_0:8=ssrc0 }" "asm"="s_bcnt0_i32_b32 {sdst}, {ssrc0}" "asmonly" {
  ret void
}
define void @S_BCNT0_I32_B32.SOP1.lit.lit_0(i32 "enum"="e1" %sdst, i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) literal_32:32=lit 101111101 sdst_16:7=sdst 0001011011111111 }" "asm"="s_bcnt0_i32_b32 {sdst}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_BCNT0_I32_B64.SOP1.Notlit_0_Notlit_1(i32 "enum"="e1" %sdst, i32 "enum"="e18" %ssrc0) "enc"="{ if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc0<256)|(ssrc0>510)) 101111101 sdst_16:7=sdst 00010111 ssrc0_0:8=ssrc0 }" "asm"="s_bcnt0_i32_b64 {sdst}, {ssrc0}" "asmonly" {
  ret void
}
define void @S_BCNT0_I32_B64.SOP1.lit.lit_0(i32 "enum"="e1" %sdst, i64 "imm" %lit) "enc"="{ if(lit>64) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) literal_32:32=lit 101111101 sdst_16:7=sdst 0001011111111111 }" "asm"="s_bcnt0_i32_b64 {sdst}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_BCNT1_I32_B32.SOP1.Notlit_0_Notlit_1(i32 "enum"="e1" %sdst, i32 "enum"="e3" %ssrc0) "enc"="{ if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) 101111101 sdst_16:7=sdst 00011000 ssrc0_0:8=ssrc0 }" "asm"="s_bcnt1_i32_b32 {sdst}, {ssrc0}" "asmonly" {
  ret void
}
define void @S_BCNT1_I32_B32.SOP1.lit.lit_0(i32 "enum"="e1" %sdst, i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) literal_32:32=lit 101111101 sdst_16:7=sdst 0001100011111111 }" "asm"="s_bcnt1_i32_b32 {sdst}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_BCNT1_I32_B64.SOP1.Notlit_0_Notlit_1(i32 "enum"="e1" %sdst, i32 "enum"="e18" %ssrc0) "enc"="{ if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc0<256)|(ssrc0>510)) 101111101 sdst_16:7=sdst 00011001 ssrc0_0:8=ssrc0 }" "asm"="s_bcnt1_i32_b64 {sdst}, {ssrc0}" "asmonly" {
  ret void
}
define void @S_BCNT1_I32_B64.SOP1.lit.lit_0(i32 "enum"="e1" %sdst, i64 "imm" %lit) "enc"="{ if(lit>64) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) literal_32:32=lit 101111101 sdst_16:7=sdst 0001100111111111 }" "asm"="s_bcnt1_i32_b64 {sdst}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_QUADMASK_B32.SOP1.Notlit_0_Notlit_1(i32 "enum"="e1" %sdst, i32 "enum"="e3" %ssrc0) "enc"="{ if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) 101111101 sdst_16:7=sdst 00011010 ssrc0_0:8=ssrc0 }" "asm"="s_quadmask_b32 {sdst}, {ssrc0}" "asmonly" {
  ret void
}
define void @S_QUADMASK_B32.SOP1.lit.lit_0(i32 "enum"="e1" %sdst, i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) literal_32:32=lit 101111101 sdst_16:7=sdst 0001101011111111 }" "asm"="s_quadmask_b32 {sdst}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_QUADMASK_B64.SOP1.Notlit_0_Notlit_1(i32 "enum"="e18" %sdst, i32 "enum"="e18" %ssrc0) "enc"="{ if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if(sdst!=253) if((sdst<256)|(sdst>510)) if((ssrc0<256)|(ssrc0>510)) 101111101 sdst_16:7=sdst 00011011 ssrc0_0:8=ssrc0 }" "asm"="s_quadmask_b64 {sdst}, {ssrc0}" "asmonly" {
  ret void
}
define void @S_QUADMASK_B64.SOP1.lit.lit_0(i32 "enum"="e18" %sdst, i64 "imm" %lit) "enc"="{ if(lit>64) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if(sdst!=253) if((sdst<256)|(sdst>510)) literal_32:32=lit 101111101 sdst_16:7=sdst 0001101111111111 }" "asm"="s_quadmask_b64 {sdst}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_WQM_B32.SOP1.Notlit_0_Notlit_1(i32 "enum"="e1" %sdst, i32 "enum"="e3" %ssrc0) "enc"="{ if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) 101111101 sdst_16:7=sdst 00011100 ssrc0_0:8=ssrc0 }" "asm"="s_wqm_b32 {sdst}, {ssrc0}" "asmonly" {
  ret void
}
define void @S_WQM_B32.SOP1.lit.lit_0(i32 "enum"="e1" %sdst, i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) literal_32:32=lit 101111101 sdst_16:7=sdst 0001110011111111 }" "asm"="s_wqm_b32 {sdst}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_WQM_B64.SOP1.Notlit_0_Notlit_1(i32 "enum"="e18" %sdst, i32 "enum"="e18" %ssrc0) "enc"="{ if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if(sdst!=253) if((sdst<256)|(sdst>510)) if((ssrc0<256)|(ssrc0>510)) 101111101 sdst_16:7=sdst 00011101 ssrc0_0:8=ssrc0 }" "asm"="s_wqm_b64 {sdst}, {ssrc0}" "asmonly" {
  ret void
}
define void @S_WQM_B64.SOP1.lit.lit_0(i32 "enum"="e18" %sdst, i64 "imm" %lit) "enc"="{ if(lit>64) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if(sdst!=253) if((sdst<256)|(sdst>510)) literal_32:32=lit 101111101 sdst_16:7=sdst 0001110111111111 }" "asm"="s_wqm_b64 {sdst}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_NOT_B32.SOP1.Notlit_0_Notlit_1(i32 "enum"="e1" %sdst, i32 "enum"="e3" %ssrc0) "enc"="{ if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) 101111101 sdst_16:7=sdst 00011110 ssrc0_0:8=ssrc0 }" "asm"="s_not_b32 {sdst}, {ssrc0}" "asmonly" {
  ret void
}
define void @S_NOT_B32.SOP1.lit.lit_0(i32 "enum"="e1" %sdst, i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) literal_32:32=lit 101111101 sdst_16:7=sdst 0001111011111111 }" "asm"="s_not_b32 {sdst}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_NOT_B64.SOP1.Notlit_0_Notlit_1(i32 "enum"="e18" %sdst, i32 "enum"="e18" %ssrc0) "enc"="{ if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if(sdst!=253) if((sdst<256)|(sdst>510)) if((ssrc0<256)|(ssrc0>510)) 101111101 sdst_16:7=sdst 00011111 ssrc0_0:8=ssrc0 }" "asm"="s_not_b64 {sdst}, {ssrc0}" "asmonly" {
  ret void
}
define void @S_NOT_B64.SOP1.lit.lit_0(i32 "enum"="e18" %sdst, i64 "imm" %lit) "enc"="{ if(lit>64) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if(sdst!=253) if((sdst<256)|(sdst>510)) literal_32:32=lit 101111101 sdst_16:7=sdst 0001111111111111 }" "asm"="s_not_b64 {sdst}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_AND_SAVEEXEC_B32.SOP1.Notlit_0_Notlit_1(i32 "enum"="e1" %sdst, i32 "enum"="e3" %ssrc0) "enc"="{ if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) 101111101 sdst_16:7=sdst 00100000 ssrc0_0:8=ssrc0 }" "asm"="s_and_saveexec_b32 {sdst}, {ssrc0}" "asmonly" {
  ret void
}
define void @S_AND_SAVEEXEC_B32.SOP1.lit.lit_0(i32 "enum"="e1" %sdst, i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) literal_32:32=lit 101111101 sdst_16:7=sdst 0010000011111111 }" "asm"="s_and_saveexec_b32 {sdst}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_AND_SAVEEXEC_B64.SOP1.Notlit_0_Notlit_1(i32 "enum"="e18" %sdst, i32 "enum"="e18" %ssrc0) "enc"="{ if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if(sdst!=253) if((sdst<256)|(sdst>510)) if((ssrc0<256)|(ssrc0>510)) 101111101 sdst_16:7=sdst 00100001 ssrc0_0:8=ssrc0 }" "asm"="s_and_saveexec_b64 {sdst}, {ssrc0}" "asmonly" {
  ret void
}
define void @S_AND_SAVEEXEC_B64.SOP1.lit.lit_0(i32 "enum"="e18" %sdst, i64 "imm" %lit) "enc"="{ if(lit>64) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if(sdst!=253) if((sdst<256)|(sdst>510)) literal_32:32=lit 101111101 sdst_16:7=sdst 0010000111111111 }" "asm"="s_and_saveexec_b64 {sdst}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_OR_SAVEEXEC_B32.SOP1.Notlit_0_Notlit_1(i32 "enum"="e1" %sdst, i32 "enum"="e3" %ssrc0) "enc"="{ if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) 101111101 sdst_16:7=sdst 00100010 ssrc0_0:8=ssrc0 }" "asm"="s_or_saveexec_b32 {sdst}, {ssrc0}" "asmonly" {
  ret void
}
define void @S_OR_SAVEEXEC_B32.SOP1.lit.lit_0(i32 "enum"="e1" %sdst, i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) literal_32:32=lit 101111101 sdst_16:7=sdst 0010001011111111 }" "asm"="s_or_saveexec_b32 {sdst}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_OR_SAVEEXEC_B64.SOP1.Notlit_0_Notlit_1(i32 "enum"="e18" %sdst, i32 "enum"="e18" %ssrc0) "enc"="{ if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if(sdst!=253) if((sdst<256)|(sdst>510)) if((ssrc0<256)|(ssrc0>510)) 101111101 sdst_16:7=sdst 00100011 ssrc0_0:8=ssrc0 }" "asm"="s_or_saveexec_b64 {sdst}, {ssrc0}" "asmonly" {
  ret void
}
define void @S_OR_SAVEEXEC_B64.SOP1.lit.lit_0(i32 "enum"="e18" %sdst, i64 "imm" %lit) "enc"="{ if(lit>64) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if(sdst!=253) if((sdst<256)|(sdst>510)) literal_32:32=lit 101111101 sdst_16:7=sdst 0010001111111111 }" "asm"="s_or_saveexec_b64 {sdst}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_XOR_SAVEEXEC_B32.SOP1.Notlit_0_Notlit_1(i32 "enum"="e1" %sdst, i32 "enum"="e3" %ssrc0) "enc"="{ if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) 101111101 sdst_16:7=sdst 00100100 ssrc0_0:8=ssrc0 }" "asm"="s_xor_saveexec_b32 {sdst}, {ssrc0}" "asmonly" {
  ret void
}
define void @S_XOR_SAVEEXEC_B32.SOP1.lit.lit_0(i32 "enum"="e1" %sdst, i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) literal_32:32=lit 101111101 sdst_16:7=sdst 0010010011111111 }" "asm"="s_xor_saveexec_b32 {sdst}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_XOR_SAVEEXEC_B64.SOP1.Notlit_0_Notlit_1(i32 "enum"="e18" %sdst, i32 "enum"="e18" %ssrc0) "enc"="{ if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if(sdst!=253) if((sdst<256)|(sdst>510)) if((ssrc0<256)|(ssrc0>510)) 101111101 sdst_16:7=sdst 00100101 ssrc0_0:8=ssrc0 }" "asm"="s_xor_saveexec_b64 {sdst}, {ssrc0}" "asmonly" {
  ret void
}
define void @S_XOR_SAVEEXEC_B64.SOP1.lit.lit_0(i32 "enum"="e18" %sdst, i64 "imm" %lit) "enc"="{ if(lit>64) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if(sdst!=253) if((sdst<256)|(sdst>510)) literal_32:32=lit 101111101 sdst_16:7=sdst 0010010111111111 }" "asm"="s_xor_saveexec_b64 {sdst}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_NAND_SAVEEXEC_B32.SOP1.Notlit_0_Notlit_1(i32 "enum"="e1" %sdst, i32 "enum"="e3" %ssrc0) "enc"="{ if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) 101111101 sdst_16:7=sdst 00100110 ssrc0_0:8=ssrc0 }" "asm"="s_nand_saveexec_b32 {sdst}, {ssrc0}" "asmonly" {
  ret void
}
define void @S_NAND_SAVEEXEC_B32.SOP1.lit.lit_0(i32 "enum"="e1" %sdst, i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) literal_32:32=lit 101111101 sdst_16:7=sdst 0010011011111111 }" "asm"="s_nand_saveexec_b32 {sdst}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_NAND_SAVEEXEC_B64.SOP1.Notlit_0_Notlit_1(i32 "enum"="e18" %sdst, i32 "enum"="e18" %ssrc0) "enc"="{ if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if(sdst!=253) if((sdst<256)|(sdst>510)) if((ssrc0<256)|(ssrc0>510)) 101111101 sdst_16:7=sdst 00100111 ssrc0_0:8=ssrc0 }" "asm"="s_nand_saveexec_b64 {sdst}, {ssrc0}" "asmonly" {
  ret void
}
define void @S_NAND_SAVEEXEC_B64.SOP1.lit.lit_0(i32 "enum"="e18" %sdst, i64 "imm" %lit) "enc"="{ if(lit>64) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if(sdst!=253) if((sdst<256)|(sdst>510)) literal_32:32=lit 101111101 sdst_16:7=sdst 0010011111111111 }" "asm"="s_nand_saveexec_b64 {sdst}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_NOR_SAVEEXEC_B32.SOP1.Notlit_0_Notlit_1(i32 "enum"="e1" %sdst, i32 "enum"="e3" %ssrc0) "enc"="{ if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) 101111101 sdst_16:7=sdst 00101000 ssrc0_0:8=ssrc0 }" "asm"="s_nor_saveexec_b32 {sdst}, {ssrc0}" "asmonly" {
  ret void
}
define void @S_NOR_SAVEEXEC_B32.SOP1.lit.lit_0(i32 "enum"="e1" %sdst, i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) literal_32:32=lit 101111101 sdst_16:7=sdst 0010100011111111 }" "asm"="s_nor_saveexec_b32 {sdst}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_NOR_SAVEEXEC_B64.SOP1.Notlit_0_Notlit_1(i32 "enum"="e18" %sdst, i32 "enum"="e18" %ssrc0) "enc"="{ if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if(sdst!=253) if((sdst<256)|(sdst>510)) if((ssrc0<256)|(ssrc0>510)) 101111101 sdst_16:7=sdst 00101001 ssrc0_0:8=ssrc0 }" "asm"="s_nor_saveexec_b64 {sdst}, {ssrc0}" "asmonly" {
  ret void
}
define void @S_NOR_SAVEEXEC_B64.SOP1.lit.lit_0(i32 "enum"="e18" %sdst, i64 "imm" %lit) "enc"="{ if(lit>64) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if(sdst!=253) if((sdst<256)|(sdst>510)) literal_32:32=lit 101111101 sdst_16:7=sdst 0010100111111111 }" "asm"="s_nor_saveexec_b64 {sdst}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_XNOR_SAVEEXEC_B32.SOP1.Notlit_0_Notlit_1(i32 "enum"="e1" %sdst, i32 "enum"="e3" %ssrc0) "enc"="{ if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) 101111101 sdst_16:7=sdst 00101010 ssrc0_0:8=ssrc0 }" "asm"="s_xnor_saveexec_b32 {sdst}, {ssrc0}" "asmonly" {
  ret void
}
define void @S_XNOR_SAVEEXEC_B32.SOP1.lit.lit_0(i32 "enum"="e1" %sdst, i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) literal_32:32=lit 101111101 sdst_16:7=sdst 0010101011111111 }" "asm"="s_xnor_saveexec_b32 {sdst}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_XNOR_SAVEEXEC_B64.SOP1.Notlit_0_Notlit_1(i32 "enum"="e18" %sdst, i32 "enum"="e18" %ssrc0) "enc"="{ if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if(sdst!=253) if((sdst<256)|(sdst>510)) if((ssrc0<256)|(ssrc0>510)) 101111101 sdst_16:7=sdst 00101011 ssrc0_0:8=ssrc0 }" "asm"="s_xnor_saveexec_b64 {sdst}, {ssrc0}" "asmonly" {
  ret void
}
define void @S_XNOR_SAVEEXEC_B64.SOP1.lit.lit_0(i32 "enum"="e18" %sdst, i64 "imm" %lit) "enc"="{ if(lit>64) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if(sdst!=253) if((sdst<256)|(sdst>510)) literal_32:32=lit 101111101 sdst_16:7=sdst 0010101111111111 }" "asm"="s_xnor_saveexec_b64 {sdst}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_AND_NOT0_SAVEEXEC_B32.SOP1.Notlit_0_Notlit_1(i32 "enum"="e1" %sdst, i32 "enum"="e3" %ssrc0) "enc"="{ if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) 101111101 sdst_16:7=sdst 00101100 ssrc0_0:8=ssrc0 }" "asm"="s_and_not0_saveexec_b32 {sdst}, {ssrc0}" "asmonly" {
  ret void
}
define void @S_AND_NOT0_SAVEEXEC_B32.SOP1.lit.lit_0(i32 "enum"="e1" %sdst, i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) literal_32:32=lit 101111101 sdst_16:7=sdst 0010110011111111 }" "asm"="s_and_not0_saveexec_b32 {sdst}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_AND_NOT0_SAVEEXEC_B64.SOP1.Notlit_0_Notlit_1(i32 "enum"="e18" %sdst, i32 "enum"="e18" %ssrc0) "enc"="{ if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if(sdst!=253) if((sdst<256)|(sdst>510)) if((ssrc0<256)|(ssrc0>510)) 101111101 sdst_16:7=sdst 00101101 ssrc0_0:8=ssrc0 }" "asm"="s_and_not0_saveexec_b64 {sdst}, {ssrc0}" "asmonly" {
  ret void
}
define void @S_AND_NOT0_SAVEEXEC_B64.SOP1.lit.lit_0(i32 "enum"="e18" %sdst, i64 "imm" %lit) "enc"="{ if(lit>64) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if(sdst!=253) if((sdst<256)|(sdst>510)) literal_32:32=lit 101111101 sdst_16:7=sdst 0010110111111111 }" "asm"="s_and_not0_saveexec_b64 {sdst}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_OR_NOT0_SAVEEXEC_B32.SOP1.Notlit_0_Notlit_1(i32 "enum"="e1" %sdst, i32 "enum"="e3" %ssrc0) "enc"="{ if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) 101111101 sdst_16:7=sdst 00101110 ssrc0_0:8=ssrc0 }" "asm"="s_or_not0_saveexec_b32 {sdst}, {ssrc0}" "asmonly" {
  ret void
}
define void @S_OR_NOT0_SAVEEXEC_B32.SOP1.lit.lit_0(i32 "enum"="e1" %sdst, i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) literal_32:32=lit 101111101 sdst_16:7=sdst 0010111011111111 }" "asm"="s_or_not0_saveexec_b32 {sdst}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_OR_NOT0_SAVEEXEC_B64.SOP1.Notlit_0_Notlit_1(i32 "enum"="e18" %sdst, i32 "enum"="e18" %ssrc0) "enc"="{ if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if(sdst!=253) if((sdst<256)|(sdst>510)) if((ssrc0<256)|(ssrc0>510)) 101111101 sdst_16:7=sdst 00101111 ssrc0_0:8=ssrc0 }" "asm"="s_or_not0_saveexec_b64 {sdst}, {ssrc0}" "asmonly" {
  ret void
}
define void @S_OR_NOT0_SAVEEXEC_B64.SOP1.lit.lit_0(i32 "enum"="e18" %sdst, i64 "imm" %lit) "enc"="{ if(lit>64) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if(sdst!=253) if((sdst<256)|(sdst>510)) literal_32:32=lit 101111101 sdst_16:7=sdst 0010111111111111 }" "asm"="s_or_not0_saveexec_b64 {sdst}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_AND_NOT1_SAVEEXEC_B32.SOP1.Notlit_0_Notlit_1(i32 "enum"="e1" %sdst, i32 "enum"="e3" %ssrc0) "enc"="{ if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) 101111101 sdst_16:7=sdst 00110000 ssrc0_0:8=ssrc0 }" "asm"="s_and_not1_saveexec_b32 {sdst}, {ssrc0}" "asmonly" {
  ret void
}
define void @S_AND_NOT1_SAVEEXEC_B32.SOP1.lit.lit_0(i32 "enum"="e1" %sdst, i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) literal_32:32=lit 101111101 sdst_16:7=sdst 0011000011111111 }" "asm"="s_and_not1_saveexec_b32 {sdst}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_AND_NOT1_SAVEEXEC_B64.SOP1.Notlit_0_Notlit_1(i32 "enum"="e18" %sdst, i32 "enum"="e18" %ssrc0) "enc"="{ if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if(sdst!=253) if((sdst<256)|(sdst>510)) if((ssrc0<256)|(ssrc0>510)) 101111101 sdst_16:7=sdst 00110001 ssrc0_0:8=ssrc0 }" "asm"="s_and_not1_saveexec_b64 {sdst}, {ssrc0}" "asmonly" {
  ret void
}
define void @S_AND_NOT1_SAVEEXEC_B64.SOP1.lit.lit_0(i32 "enum"="e18" %sdst, i64 "imm" %lit) "enc"="{ if(lit>64) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if(sdst!=253) if((sdst<256)|(sdst>510)) literal_32:32=lit 101111101 sdst_16:7=sdst 0011000111111111 }" "asm"="s_and_not1_saveexec_b64 {sdst}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_OR_NOT1_SAVEEXEC_B32.SOP1.Notlit_0_Notlit_1(i32 "enum"="e1" %sdst, i32 "enum"="e3" %ssrc0) "enc"="{ if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) 101111101 sdst_16:7=sdst 00110010 ssrc0_0:8=ssrc0 }" "asm"="s_or_not1_saveexec_b32 {sdst}, {ssrc0}" "asmonly" {
  ret void
}
define void @S_OR_NOT1_SAVEEXEC_B32.SOP1.lit.lit_0(i32 "enum"="e1" %sdst, i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) literal_32:32=lit 101111101 sdst_16:7=sdst 0011001011111111 }" "asm"="s_or_not1_saveexec_b32 {sdst}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_OR_NOT1_SAVEEXEC_B64.SOP1.Notlit_0_Notlit_1(i32 "enum"="e18" %sdst, i32 "enum"="e18" %ssrc0) "enc"="{ if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if(sdst!=253) if((sdst<256)|(sdst>510)) if((ssrc0<256)|(ssrc0>510)) 101111101 sdst_16:7=sdst 00110011 ssrc0_0:8=ssrc0 }" "asm"="s_or_not1_saveexec_b64 {sdst}, {ssrc0}" "asmonly" {
  ret void
}
define void @S_OR_NOT1_SAVEEXEC_B64.SOP1.lit.lit_0(i32 "enum"="e18" %sdst, i64 "imm" %lit) "enc"="{ if(lit>64) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if(sdst!=253) if((sdst<256)|(sdst>510)) literal_32:32=lit 101111101 sdst_16:7=sdst 0011001111111111 }" "asm"="s_or_not1_saveexec_b64 {sdst}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_AND_NOT0_WREXEC_B32.SOP1.Notlit_0_Notlit_1(i32 "enum"="e1" %sdst, i32 "enum"="e3" %ssrc0) "enc"="{ if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) 101111101 sdst_16:7=sdst 00110100 ssrc0_0:8=ssrc0 }" "asm"="s_and_not0_wrexec_b32 {sdst}, {ssrc0}" "asmonly" {
  ret void
}
define void @S_AND_NOT0_WREXEC_B32.SOP1.lit.lit_0(i32 "enum"="e1" %sdst, i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) literal_32:32=lit 101111101 sdst_16:7=sdst 0011010011111111 }" "asm"="s_and_not0_wrexec_b32 {sdst}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_AND_NOT0_WREXEC_B64.SOP1.Notlit_0_Notlit_1(i32 "enum"="e18" %sdst, i32 "enum"="e18" %ssrc0) "enc"="{ if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if(sdst!=253) if((sdst<256)|(sdst>510)) if((ssrc0<256)|(ssrc0>510)) 101111101 sdst_16:7=sdst 00110101 ssrc0_0:8=ssrc0 }" "asm"="s_and_not0_wrexec_b64 {sdst}, {ssrc0}" "asmonly" {
  ret void
}
define void @S_AND_NOT0_WREXEC_B64.SOP1.lit.lit_0(i32 "enum"="e18" %sdst, i64 "imm" %lit) "enc"="{ if(lit>64) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if(sdst!=253) if((sdst<256)|(sdst>510)) literal_32:32=lit 101111101 sdst_16:7=sdst 0011010111111111 }" "asm"="s_and_not0_wrexec_b64 {sdst}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_AND_NOT1_WREXEC_B32.SOP1.Notlit_0_Notlit_1(i32 "enum"="e1" %sdst, i32 "enum"="e3" %ssrc0) "enc"="{ if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if(ssrc0!=154) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) 101111101 sdst_16:7=sdst 00110110 ssrc0_0:8=ssrc0 }" "asm"="s_and_not1_wrexec_b32 {sdst}, {ssrc0}" "asmonly" {
  ret void
}
define void @S_AND_NOT1_WREXEC_B32.SOP1.lit.lit_0(i32 "enum"="e1" %sdst, i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) literal_32:32=lit 101111101 sdst_16:7=sdst 0011011011111111 }" "asm"="s_and_not1_wrexec_b32 {sdst}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_AND_NOT1_WREXEC_B64.SOP1.Notlit_0_Notlit_1(i32 "enum"="e18" %sdst, i32 "enum"="e18" %ssrc0) "enc"="{ if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if(sdst!=253) if((sdst<256)|(sdst>510)) if((ssrc0<256)|(ssrc0>510)) 101111101 sdst_16:7=sdst 00110111 ssrc0_0:8=ssrc0 }" "asm"="s_and_not1_wrexec_b64 {sdst}, {ssrc0}" "asmonly" {
  ret void
}
define void @S_AND_NOT1_WREXEC_B64.SOP1.lit.lit_0(i32 "enum"="e18" %sdst, i64 "imm" %lit) "enc"="{ if(lit>64) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if(sdst!=253) if((sdst<256)|(sdst>510)) literal_32:32=lit 101111101 sdst_16:7=sdst 0011011111111111 }" "asm"="s_and_not1_wrexec_b64 {sdst}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_MOVRELS_B32.SOP1.Notlit_0_Notlit_1(i32 "enum"="e1" %sdst, i32 "enum"="e3" %ssrc0) "enc"="{ if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc0<128)|(ssrc0>208)) if((ssrc0<235)|(ssrc0>238)) if((ssrc0<240)|(ssrc0>248)) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) 101111101 sdst_16:7=sdst 01000000 ssrc0_0:8=ssrc0 }" "asm"="s_movrels_b32 {sdst}, {ssrc0}" "asmonly" {
  ret void
}
define void @S_MOVRELS_B64.SOP1.Notlit_0_Notlit_1(i32 "enum"="e18" %sdst, i32 "enum"="e18" %ssrc0) "enc"="{ if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if(sdst!=253) if((sdst<256)|(sdst>510)) if((ssrc0<128)|(ssrc0>208)) if((ssrc0<235)|(ssrc0>238)) if((ssrc0<240)|(ssrc0>248)) if(ssrc0!=253) if((ssrc0<256)|(ssrc0>510)) 101111101 sdst_16:7=sdst 01000001 ssrc0_0:8=ssrc0 }" "asm"="s_movrels_b64 {sdst}, {ssrc0}" "asmonly" {
  ret void
}
define void @S_MOVRELD_B32.SOP1.Notlit_0_Notlit_1(i32 "enum"="e1" %sdst, i32 "enum"="e3" %ssrc0) "enc"="{ if(sdst!=25) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) 101111101 sdst_16:7=sdst 01000010 ssrc0_0:8=ssrc0 }" "asm"="s_movreld_b32 {sdst}, {ssrc0}" "asmonly" {
  ret void
}
define void @S_MOVRELD_B32.SOP1.lit.lit_0(i32 "enum"="e1" %sdst, i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) literal_32:32=lit 101111101 sdst_16:7=sdst 0100001011111111 }" "asm"="s_movreld_b32 {sdst}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_MOVRELD_B64.SOP1.Notlit_0_Notlit_1(i32 "enum"="e18" %sdst, i32 "enum"="e18" %ssrc0) "enc"="{ if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if(sdst!=253) if((sdst<256)|(sdst>510)) if((ssrc0<256)|(ssrc0>510)) 101111101 sdst_16:7=sdst 01000011 ssrc0_0:8=ssrc0 }" "asm"="s_movreld_b64 {sdst}, {ssrc0}" "asmonly" {
  ret void
}
define void @S_MOVRELD_B64.SOP1.lit.lit_0(i32 "enum"="e18" %sdst, i64 "imm" %lit) "enc"="{ if(lit>64) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if(sdst!=253) if((sdst<256)|(sdst>510)) literal_32:32=lit 101111101 sdst_16:7=sdst 0100001111111111 }" "asm"="s_movreld_b64 {sdst}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_MOVRELSD_2_B32.SOP1.Notlit_0_Notlit_1(i32 "enum"="e1" %sdst, i32 "enum"="e3" %ssrc0) "enc"="{ if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) 101111101 sdst_16:7=sdst 01000100 ssrc0_0:8=ssrc0 }" "asm"="s_movrelsd_2_b32 {sdst}, {ssrc0}" "asmonly" {
  ret void
}
define void @S_GETPC_B64.SOP1.Notlit_0_Notlit_1(i32 "enum"="e18" %sdst) "enc"="{ if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if(sdst!=253) if((sdst<256)|(sdst>510)) 101111101 sdst_16:7=sdst 0100011100000000 }" "asm"="s_getpc_b64 {sdst}" "asmonly" {
  ret void
}
define void @S_SETPC_B64.SOP1.Notlit_0_Notlit_1(i32 "enum"="e18" %ssrc0) "enc"="{ if((ssrc0<128)|(ssrc0>208)) if((ssrc0<235)|(ssrc0>238)) if((ssrc0<240)|(ssrc0>248)) if(ssrc0!=253) if((ssrc0<256)|(ssrc0>510)) 101111101000000001001000 ssrc0_0:8=ssrc0 }" "asm"="s_setpc_b64 {ssrc0}" "asmonly" {
  ret void
}
define void @S_SWAPPC_B64.SOP1.Notlit_0_Notlit_1(i32 "enum"="e18" %sdst, i32 "enum"="e18" %ssrc0) "enc"="{ if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if(sdst!=253) if((sdst<256)|(sdst>510)) if((ssrc0<256)|(ssrc0>510)) 101111101 sdst_16:7=sdst 01001001 ssrc0_0:8=ssrc0 }" "asm"="s_swappc_b64 {sdst}, {ssrc0}" "asmonly" {
  ret void
}
define void @S_RFE_B64.SOP1.Notlit_0_Notlit_1(i32 "enum"="e18" %ssrc0) "enc"="{ if((ssrc0<128)|(ssrc0>208)) if((ssrc0<235)|(ssrc0>238)) if((ssrc0<240)|(ssrc0>248)) if(ssrc0!=253) if((ssrc0<256)|(ssrc0>510)) 101111101000000001001010 ssrc0_0:8=ssrc0 }" "asm"="s_rfe_b64 {ssrc0}" "asmonly" {
  ret void
}
define void @S_SENDMSG_RTN_B32.SOP1(i32 "enum"="e1" %sdst, i32 "enum"="e35" %ssrc0) "enc"="{ if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if(ssrc0!=237) if(ssrc0!=255) 101111101 sdst_16:7=sdst 01001100 ssrc0_0:8=ssrc0 }" "asm"="s_sendmsg_rtn_b32 {sdst}, {ssrc0}" "asmonly" {
  ret void
}
define void @S_SENDMSG_RTN_B64.SOP1(i32 "enum"="e18" %sdst, i32 "enum"="e35" %ssrc0) "enc"="{ if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if(sdst!=253) if((sdst<256)|(sdst>510)) if(ssrc0!=255) 101111101 sdst_16:7=sdst 01001101 ssrc0_0:8=ssrc0 }" "asm"="s_sendmsg_rtn_b64 {sdst}, {ssrc0}" "asmonly" {
  ret void
}
define void @S_CEIL_F32.SOP1.Notlit_0_Notlit_1(i32 "enum"="e1" %sdst, i32 "enum"="e3" %ssrc0) "enc"="{ if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) 101111101 sdst_16:7=sdst 01100000 ssrc0_0:8=ssrc0 }" "asm"="s_ceil_f32 {sdst}, {ssrc0}" "asmonly" {
  ret void
}
define void @S_CEIL_F32.SOP1.lit.lit_0(i32 "enum"="e1" %sdst, i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) literal_32:32=lit 101111101 sdst_16:7=sdst 0110000011111111 }" "asm"="s_ceil_f32 {sdst}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_FLOOR_F32.SOP1.Notlit_0_Notlit_1(i32 "enum"="e1" %sdst, i32 "enum"="e3" %ssrc0) "enc"="{ if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) 101111101 sdst_16:7=sdst 01100001 ssrc0_0:8=ssrc0 }" "asm"="s_floor_f32 {sdst}, {ssrc0}" "asmonly" {
  ret void
}
define void @S_FLOOR_F32.SOP1.lit.lit_0(i32 "enum"="e1" %sdst, i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) literal_32:32=lit 101111101 sdst_16:7=sdst 0110000111111111 }" "asm"="s_floor_f32 {sdst}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_TRUNC_F32.SOP1.Notlit_0_Notlit_1(i32 "enum"="e1" %sdst, i32 "enum"="e3" %ssrc0) "enc"="{ if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) 101111101 sdst_16:7=sdst 01100010 ssrc0_0:8=ssrc0 }" "asm"="s_trunc_f32 {sdst}, {ssrc0}" "asmonly" {
  ret void
}
define void @S_TRUNC_F32.SOP1.lit.lit_0(i32 "enum"="e1" %sdst, i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) literal_32:32=lit 101111101 sdst_16:7=sdst 0110001011111111 }" "asm"="s_trunc_f32 {sdst}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_RNDNE_F32.SOP1.Notlit_0_Notlit_1(i32 "enum"="e1" %sdst, i32 "enum"="e3" %ssrc0) "enc"="{ if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) 101111101 sdst_16:7=sdst 01100011 ssrc0_0:8=ssrc0 }" "asm"="s_rndne_f32 {sdst}, {ssrc0}" "asmonly" {
  ret void
}
define void @S_RNDNE_F32.SOP1.lit.lit_0(i32 "enum"="e1" %sdst, i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) literal_32:32=lit 101111101 sdst_16:7=sdst 0110001111111111 }" "asm"="s_rndne_f32 {sdst}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_CVT_F32_I32.SOP1.Notlit_0_Notlit_1(i32 "enum"="e1" %sdst, i32 "enum"="e3" %ssrc0) "enc"="{ if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) 101111101 sdst_16:7=sdst 01100100 ssrc0_0:8=ssrc0 }" "asm"="s_cvt_f32_i32 {sdst}, {ssrc0}" "asmonly" {
  ret void
}
define void @S_CVT_F32_I32.SOP1.lit.lit_0(i32 "enum"="e1" %sdst, i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) literal_32:32=lit 101111101 sdst_16:7=sdst 0110010011111111 }" "asm"="s_cvt_f32_i32 {sdst}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_CVT_F32_U32.SOP1.Notlit_0_Notlit_1(i32 "enum"="e1" %sdst, i32 "enum"="e3" %ssrc0) "enc"="{ if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) 101111101 sdst_16:7=sdst 01100101 ssrc0_0:8=ssrc0 }" "asm"="s_cvt_f32_u32 {sdst}, {ssrc0}" "asmonly" {
  ret void
}
define void @S_CVT_F32_U32.SOP1.lit.lit_0(i32 "enum"="e1" %sdst, i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) literal_32:32=lit 101111101 sdst_16:7=sdst 0110010111111111 }" "asm"="s_cvt_f32_u32 {sdst}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_CVT_I32_F32.SOP1.Notlit_0_Notlit_1(i32 "enum"="e1" %sdst, i32 "enum"="e3" %ssrc0) "enc"="{ if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) 101111101 sdst_16:7=sdst 01100110 ssrc0_0:8=ssrc0 }" "asm"="s_cvt_i32_f32 {sdst}, {ssrc0}" "asmonly" {
  ret void
}
define void @S_CVT_I32_F32.SOP1.lit.lit_0(i32 "enum"="e1" %sdst, i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) literal_32:32=lit 101111101 sdst_16:7=sdst 0110011011111111 }" "asm"="s_cvt_i32_f32 {sdst}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_CVT_U32_F32.SOP1.Notlit_0_Notlit_1(i32 "enum"="e1" %sdst, i32 "enum"="e3" %ssrc0) "enc"="{ if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) 101111101 sdst_16:7=sdst 01100111 ssrc0_0:8=ssrc0 }" "asm"="s_cvt_u32_f32 {sdst}, {ssrc0}" "asmonly" {
  ret void
}
define void @S_CVT_U32_F32.SOP1.lit.lit_0(i32 "enum"="e1" %sdst, i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) literal_32:32=lit 101111101 sdst_16:7=sdst 0110011111111111 }" "asm"="s_cvt_u32_f32 {sdst}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_CVT_F16_F32.SOP1.Notlit_0_Notlit_1(i32 "enum"="e1" %sdst, i32 "enum"="e3" %ssrc0) "enc"="{ if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) 101111101 sdst_16:7=sdst 01101000 ssrc0_0:8=ssrc0 }" "asm"="s_cvt_f16_f32 {sdst}, {ssrc0}" "asmonly" {
  ret void
}
define void @S_CVT_F16_F32.SOP1.lit.lit_0(i32 "enum"="e1" %sdst, i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) literal_32:32=lit 101111101 sdst_16:7=sdst 0110100011111111 }" "asm"="s_cvt_f16_f32 {sdst}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_CVT_F32_F16.SOP1.Notlit_0_Notlit_1(i32 "enum"="e1" %sdst, i32 "enum"="e3" %ssrc0) "enc"="{ if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) 101111101 sdst_16:7=sdst 01101001 ssrc0_0:8=ssrc0 }" "asm"="s_cvt_f32_f16 {sdst}, {ssrc0}" "asmonly" {
  ret void
}
define void @S_CVT_F32_F16.SOP1.lit.lit_0(i32 "enum"="e1" %sdst, i64 "imm" %lit) "enc"="{ if(lit<65536) if(lit>64) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) literal_32:32=lit 101111101 sdst_16:7=sdst 0110100111111111 }" "asm"="s_cvt_f32_f16 {sdst}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_CVT_HI_F32_F16.SOP1.Notlit_0_Notlit_1(i32 "enum"="e1" %sdst, i32 "enum"="e3" %ssrc0) "enc"="{ if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) 101111101 sdst_16:7=sdst 01101010 ssrc0_0:8=ssrc0 }" "asm"="s_cvt_hi_f32_f16 {sdst}, {ssrc0}" "asmonly" {
  ret void
}
define void @S_CVT_HI_F32_F16.SOP1.lit.lit_0(i32 "enum"="e1" %sdst, i64 "imm" %lit) "enc"="{ if(lit<65536) if(lit>64) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) literal_32:32=lit 101111101 sdst_16:7=sdst 0110101011111111 }" "asm"="s_cvt_hi_f32_f16 {sdst}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_CEIL_F16.SOP1.Notlit_0_Notlit_1(i32 "enum"="e1" %sdst, i32 "enum"="e3" %ssrc0) "enc"="{ if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) 101111101 sdst_16:7=sdst 01101011 ssrc0_0:8=ssrc0 }" "asm"="s_ceil_f16 {sdst}, {ssrc0}" "asmonly" {
  ret void
}
define void @S_CEIL_F16.SOP1.lit.lit_0(i32 "enum"="e1" %sdst, i64 "imm" %lit) "enc"="{ if(lit<65536) if(lit>64) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) literal_32:32=lit 101111101 sdst_16:7=sdst 0110101111111111 }" "asm"="s_ceil_f16 {sdst}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_FLOOR_F16.SOP1.Notlit_0_Notlit_1(i32 "enum"="e1" %sdst, i32 "enum"="e3" %ssrc0) "enc"="{ if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) 101111101 sdst_16:7=sdst 01101100 ssrc0_0:8=ssrc0 }" "asm"="s_floor_f16 {sdst}, {ssrc0}" "asmonly" {
  ret void
}
define void @S_FLOOR_F16.SOP1.lit.lit_0(i32 "enum"="e1" %sdst, i64 "imm" %lit) "enc"="{ if(lit<65536) if(lit>64) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) literal_32:32=lit 101111101 sdst_16:7=sdst 0110110011111111 }" "asm"="s_floor_f16 {sdst}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_TRUNC_F16.SOP1.Notlit_0_Notlit_1(i32 "enum"="e1" %sdst, i32 "enum"="e3" %ssrc0) "enc"="{ if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) 101111101 sdst_16:7=sdst 01101101 ssrc0_0:8=ssrc0 }" "asm"="s_trunc_f16 {sdst}, {ssrc0}" "asmonly" {
  ret void
}
define void @S_TRUNC_F16.SOP1.lit.lit_0(i32 "enum"="e1" %sdst, i64 "imm" %lit) "enc"="{ if(lit<65536) if(lit>64) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) literal_32:32=lit 101111101 sdst_16:7=sdst 0110110111111111 }" "asm"="s_trunc_f16 {sdst}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_RNDNE_F16.SOP1.Notlit_0_Notlit_1(i32 "enum"="e1" %sdst, i32 "enum"="e3" %ssrc0) "enc"="{ if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) 101111101 sdst_16:7=sdst 01101110 ssrc0_0:8=ssrc0 }" "asm"="s_rndne_f16 {sdst}, {ssrc0}" "asmonly" {
  ret void
}
define void @S_RNDNE_F16.SOP1.lit.lit_0(i32 "enum"="e1" %sdst, i64 "imm" %lit) "enc"="{ if(lit<65536) if(lit>64) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) literal_32:32=lit 101111101 sdst_16:7=sdst 0110111011111111 }" "asm"="s_rndne_f16 {sdst}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_ADD_U32.SOP2(i32 "enum"="e1" %sdst, i32 "enum"="e3" %ssrc0, i32 "enum"="e3" %ssrc1) "enc"="{ if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) 100000000 sdst_16:7=sdst ssrc1_8:8=ssrc1 ssrc0_0:8=ssrc0 }" "asm"="s_add_u32 {sdst}, {ssrc0}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_ADD_U32.SOP2.lit.lit_0(i32 "enum"="e1" %sdst, i64 "imm" %lit, i32 "enum"="e3" %ssrc1) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) literal_32:32=lit 100000000 sdst_16:7=sdst ssrc1_8:8=ssrc1 11111111 }" "asm"="s_add_u32 {sdst}, {lit:h32}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_ADD_U32.SOP2.lit.lit_1(i32 "enum"="e1" %sdst, i32 "enum"="e3" %ssrc0, i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) literal_32:32=lit 100000000 sdst_16:7=sdst 11111111 ssrc0_0:8=ssrc0 }" "asm"="s_add_u32 {sdst}, {ssrc0}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_ADD_U32.SOP2.lit.lit_0_lit_1(i32 "enum"="e1" %sdst, i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) literal_32:32=lit 100000000 sdst_16:7=sdst 1111111111111111 }" "asm"="s_add_u32 {sdst}, {lit:h32}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_SUB_U32.SOP2(i32 "enum"="e1" %sdst, i32 "enum"="e3" %ssrc0, i32 "enum"="e3" %ssrc1) "enc"="{ if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) 100000001 sdst_16:7=sdst ssrc1_8:8=ssrc1 ssrc0_0:8=ssrc0 }" "asm"="s_sub_u32 {sdst}, {ssrc0}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_SUB_U32.SOP2.lit.lit_0(i32 "enum"="e1" %sdst, i64 "imm" %lit, i32 "enum"="e3" %ssrc1) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) literal_32:32=lit 100000001 sdst_16:7=sdst ssrc1_8:8=ssrc1 11111111 }" "asm"="s_sub_u32 {sdst}, {lit:h32}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_SUB_U32.SOP2.lit.lit_1(i32 "enum"="e1" %sdst, i32 "enum"="e3" %ssrc0, i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) literal_32:32=lit 100000001 sdst_16:7=sdst 11111111 ssrc0_0:8=ssrc0 }" "asm"="s_sub_u32 {sdst}, {ssrc0}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_SUB_U32.SOP2.lit.lit_0_lit_1(i32 "enum"="e1" %sdst, i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) literal_32:32=lit 100000001 sdst_16:7=sdst 1111111111111111 }" "asm"="s_sub_u32 {sdst}, {lit:h32}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_ADD_I32.SOP2(i32 "enum"="e1" %sdst, i32 "enum"="e3" %ssrc0, i32 "enum"="e3" %ssrc1) "enc"="{ if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) if(ssrc1!=101) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) 100000010 sdst_16:7=sdst ssrc1_8:8=ssrc1 ssrc0_0:8=ssrc0 }" "asm"="s_add_i32 {sdst}, {ssrc0}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_ADD_I32.SOP2.lit.lit_0(i32 "enum"="e1" %sdst, i64 "imm" %lit, i32 "enum"="e3" %ssrc1) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) literal_32:32=lit 100000010 sdst_16:7=sdst ssrc1_8:8=ssrc1 11111111 }" "asm"="s_add_i32 {sdst}, {lit:h32}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_ADD_I32.SOP2.lit.lit_1(i32 "enum"="e1" %sdst, i32 "enum"="e3" %ssrc0, i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) literal_32:32=lit 100000010 sdst_16:7=sdst 11111111 ssrc0_0:8=ssrc0 }" "asm"="s_add_i32 {sdst}, {ssrc0}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_ADD_I32.SOP2.lit.lit_0_lit_1(i32 "enum"="e1" %sdst, i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) literal_32:32=lit 100000010 sdst_16:7=sdst 1111111111111111 }" "asm"="s_add_i32 {sdst}, {lit:h32}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_SUB_I32.SOP2(i32 "enum"="e1" %sdst, i32 "enum"="e3" %ssrc0, i32 "enum"="e3" %ssrc1) "enc"="{ if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) 100000011 sdst_16:7=sdst ssrc1_8:8=ssrc1 ssrc0_0:8=ssrc0 }" "asm"="s_sub_i32 {sdst}, {ssrc0}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_SUB_I32.SOP2.lit.lit_0(i32 "enum"="e1" %sdst, i64 "imm" %lit, i32 "enum"="e3" %ssrc1) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) literal_32:32=lit 100000011 sdst_16:7=sdst ssrc1_8:8=ssrc1 11111111 }" "asm"="s_sub_i32 {sdst}, {lit:h32}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_SUB_I32.SOP2.lit.lit_1(i32 "enum"="e1" %sdst, i32 "enum"="e3" %ssrc0, i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) literal_32:32=lit 100000011 sdst_16:7=sdst 11111111 ssrc0_0:8=ssrc0 }" "asm"="s_sub_i32 {sdst}, {ssrc0}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_SUB_I32.SOP2.lit.lit_0_lit_1(i32 "enum"="e1" %sdst, i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) literal_32:32=lit 100000011 sdst_16:7=sdst 1111111111111111 }" "asm"="s_sub_i32 {sdst}, {lit:h32}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_ADDC_U32.SOP2(i32 "enum"="e1" %sdst, i32 "enum"="e3" %ssrc0, i32 "enum"="e3" %ssrc1) "enc"="{ if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) 100000100 sdst_16:7=sdst ssrc1_8:8=ssrc1 ssrc0_0:8=ssrc0 }" "asm"="s_addc_u32 {sdst}, {ssrc0}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_ADDC_U32.SOP2.lit.lit_0(i32 "enum"="e1" %sdst, i64 "imm" %lit, i32 "enum"="e3" %ssrc1) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) literal_32:32=lit 100000100 sdst_16:7=sdst ssrc1_8:8=ssrc1 11111111 }" "asm"="s_addc_u32 {sdst}, {lit:h32}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_ADDC_U32.SOP2.lit.lit_1(i32 "enum"="e1" %sdst, i32 "enum"="e3" %ssrc0, i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) literal_32:32=lit 100000100 sdst_16:7=sdst 11111111 ssrc0_0:8=ssrc0 }" "asm"="s_addc_u32 {sdst}, {ssrc0}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_ADDC_U32.SOP2.lit.lit_0_lit_1(i32 "enum"="e1" %sdst, i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) literal_32:32=lit 100000100 sdst_16:7=sdst 1111111111111111 }" "asm"="s_addc_u32 {sdst}, {lit:h32}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_SUBB_U32.SOP2(i32 "enum"="e1" %sdst, i32 "enum"="e3" %ssrc0, i32 "enum"="e3" %ssrc1) "enc"="{ if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) 100000101 sdst_16:7=sdst ssrc1_8:8=ssrc1 ssrc0_0:8=ssrc0 }" "asm"="s_subb_u32 {sdst}, {ssrc0}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_SUBB_U32.SOP2.lit.lit_0(i32 "enum"="e1" %sdst, i64 "imm" %lit, i32 "enum"="e3" %ssrc1) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) literal_32:32=lit 100000101 sdst_16:7=sdst ssrc1_8:8=ssrc1 11111111 }" "asm"="s_subb_u32 {sdst}, {lit:h32}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_SUBB_U32.SOP2.lit.lit_1(i32 "enum"="e1" %sdst, i32 "enum"="e3" %ssrc0, i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) literal_32:32=lit 100000101 sdst_16:7=sdst 11111111 ssrc0_0:8=ssrc0 }" "asm"="s_subb_u32 {sdst}, {ssrc0}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_SUBB_U32.SOP2.lit.lit_0_lit_1(i32 "enum"="e1" %sdst, i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) literal_32:32=lit 100000101 sdst_16:7=sdst 1111111111111111 }" "asm"="s_subb_u32 {sdst}, {lit:h32}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_ABSDIFF_I32.SOP2(i32 "enum"="e1" %sdst, i32 "enum"="e3" %ssrc0, i32 "enum"="e3" %ssrc1) "enc"="{ if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) 100000110 sdst_16:7=sdst ssrc1_8:8=ssrc1 ssrc0_0:8=ssrc0 }" "asm"="s_absdiff_i32 {sdst}, {ssrc0}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_ABSDIFF_I32.SOP2.lit.lit_0(i32 "enum"="e1" %sdst, i64 "imm" %lit, i32 "enum"="e3" %ssrc1) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) literal_32:32=lit 100000110 sdst_16:7=sdst ssrc1_8:8=ssrc1 11111111 }" "asm"="s_absdiff_i32 {sdst}, {lit:h32}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_ABSDIFF_I32.SOP2.lit.lit_1(i32 "enum"="e1" %sdst, i32 "enum"="e3" %ssrc0, i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) literal_32:32=lit 100000110 sdst_16:7=sdst 11111111 ssrc0_0:8=ssrc0 }" "asm"="s_absdiff_i32 {sdst}, {ssrc0}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_ABSDIFF_I32.SOP2.lit.lit_0_lit_1(i32 "enum"="e1" %sdst, i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) literal_32:32=lit 100000110 sdst_16:7=sdst 1111111111111111 }" "asm"="s_absdiff_i32 {sdst}, {lit:h32}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_LSHL_B32.SOP2(i32 "enum"="e1" %sdst, i32 "enum"="e3" %ssrc0, i32 "enum"="e3" %ssrc1) "enc"="{ if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) 100001000 sdst_16:7=sdst ssrc1_8:8=ssrc1 ssrc0_0:8=ssrc0 }" "asm"="s_lshl_b32 {sdst}, {ssrc0}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_LSHL_B32.SOP2.lit.lit_0(i32 "enum"="e1" %sdst, i64 "imm" %lit, i32 "enum"="e3" %ssrc1) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) literal_32:32=lit 100001000 sdst_16:7=sdst ssrc1_8:8=ssrc1 11111111 }" "asm"="s_lshl_b32 {sdst}, {lit:h32}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_LSHL_B32.SOP2.lit.lit_1(i32 "enum"="e1" %sdst, i32 "enum"="e3" %ssrc0, i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) literal_32:32=lit 100001000 sdst_16:7=sdst 11111111 ssrc0_0:8=ssrc0 }" "asm"="s_lshl_b32 {sdst}, {ssrc0}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_LSHL_B32.SOP2.lit.lit_0_lit_1(i32 "enum"="e1" %sdst, i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) literal_32:32=lit 100001000 sdst_16:7=sdst 1111111111111111 }" "asm"="s_lshl_b32 {sdst}, {lit:h32}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_LSHL_B64.SOP2(i32 "enum"="e18" %sdst, i32 "enum"="e18" %ssrc0, i32 "enum"="e3" %ssrc1) "enc"="{ if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if(sdst!=253) if((sdst<256)|(sdst>510)) if((ssrc0<256)|(ssrc0>510)) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) 100001001 sdst_16:7=sdst ssrc1_8:8=ssrc1 ssrc0_0:8=ssrc0 }" "asm"="s_lshl_b64 {sdst}, {ssrc0}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_LSHL_B64.SOP2.lit.lit_0(i32 "enum"="e18" %sdst, i64 "imm" %lit, i32 "enum"="e3" %ssrc1) "enc"="{ if(lit>64) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if(sdst!=253) if((sdst<256)|(sdst>510)) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) literal_32:32=lit 100001001 sdst_16:7=sdst ssrc1_8:8=ssrc1 11111111 }" "asm"="s_lshl_b64 {sdst}, {lit:h32}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_LSHL_B64.SOP2.lit.lit_1(i32 "enum"="e18" %sdst, i32 "enum"="e18" %ssrc0, i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if(sdst!=253) if((sdst<256)|(sdst>510)) if((ssrc0<256)|(ssrc0>510)) literal_32:32=lit 100001001 sdst_16:7=sdst 11111111 ssrc0_0:8=ssrc0 }" "asm"="s_lshl_b64 {sdst}, {ssrc0}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_LSHL_B64.SOP2.lit.lit_0_lit_1(i32 "enum"="e18" %sdst, i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if(sdst!=253) if((sdst<256)|(sdst>510)) literal_32:32=lit 100001001 sdst_16:7=sdst 1111111111111111 }" "asm"="s_lshl_b64 {sdst}, {lit:h32}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_LSHR_B32.SOP2(i32 "enum"="e1" %sdst, i32 "enum"="e3" %ssrc0, i32 "enum"="e3" %ssrc1) "enc"="{ if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) 100001010 sdst_16:7=sdst ssrc1_8:8=ssrc1 ssrc0_0:8=ssrc0 }" "asm"="s_lshr_b32 {sdst}, {ssrc0}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_LSHR_B32.SOP2.lit.lit_0(i32 "enum"="e1" %sdst, i64 "imm" %lit, i32 "enum"="e3" %ssrc1) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) literal_32:32=lit 100001010 sdst_16:7=sdst ssrc1_8:8=ssrc1 11111111 }" "asm"="s_lshr_b32 {sdst}, {lit:h32}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_LSHR_B32.SOP2.lit.lit_1(i32 "enum"="e1" %sdst, i32 "enum"="e3" %ssrc0, i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) literal_32:32=lit 100001010 sdst_16:7=sdst 11111111 ssrc0_0:8=ssrc0 }" "asm"="s_lshr_b32 {sdst}, {ssrc0}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_LSHR_B32.SOP2.lit.lit_0_lit_1(i32 "enum"="e1" %sdst, i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) literal_32:32=lit 100001010 sdst_16:7=sdst 1111111111111111 }" "asm"="s_lshr_b32 {sdst}, {lit:h32}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_LSHR_B64.SOP2(i32 "enum"="e18" %sdst, i32 "enum"="e18" %ssrc0, i32 "enum"="e3" %ssrc1) "enc"="{ if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if(sdst!=253) if((sdst<256)|(sdst>510)) if((ssrc0<256)|(ssrc0>510)) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) 100001011 sdst_16:7=sdst ssrc1_8:8=ssrc1 ssrc0_0:8=ssrc0 }" "asm"="s_lshr_b64 {sdst}, {ssrc0}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_LSHR_B64.SOP2.lit.lit_0(i32 "enum"="e18" %sdst, i64 "imm" %lit, i32 "enum"="e3" %ssrc1) "enc"="{ if(lit>64) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if(sdst!=253) if((sdst<256)|(sdst>510)) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) literal_32:32=lit 100001011 sdst_16:7=sdst ssrc1_8:8=ssrc1 11111111 }" "asm"="s_lshr_b64 {sdst}, {lit:h32}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_LSHR_B64.SOP2.lit.lit_1(i32 "enum"="e18" %sdst, i32 "enum"="e18" %ssrc0, i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if(sdst!=253) if((sdst<256)|(sdst>510)) if((ssrc0<256)|(ssrc0>510)) literal_32:32=lit 100001011 sdst_16:7=sdst 11111111 ssrc0_0:8=ssrc0 }" "asm"="s_lshr_b64 {sdst}, {ssrc0}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_LSHR_B64.SOP2.lit.lit_0_lit_1(i32 "enum"="e18" %sdst, i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if(sdst!=253) if((sdst<256)|(sdst>510)) literal_32:32=lit 100001011 sdst_16:7=sdst 1111111111111111 }" "asm"="s_lshr_b64 {sdst}, {lit:h32}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_ASHR_I32.SOP2(i32 "enum"="e1" %sdst, i32 "enum"="e3" %ssrc0, i32 "enum"="e3" %ssrc1) "enc"="{ if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) 100001100 sdst_16:7=sdst ssrc1_8:8=ssrc1 ssrc0_0:8=ssrc0 }" "asm"="s_ashr_i32 {sdst}, {ssrc0}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_ASHR_I32.SOP2.lit.lit_0(i32 "enum"="e1" %sdst, i64 "imm" %lit, i32 "enum"="e3" %ssrc1) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) literal_32:32=lit 100001100 sdst_16:7=sdst ssrc1_8:8=ssrc1 11111111 }" "asm"="s_ashr_i32 {sdst}, {lit:h32}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_ASHR_I32.SOP2.lit.lit_1(i32 "enum"="e1" %sdst, i32 "enum"="e3" %ssrc0, i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) literal_32:32=lit 100001100 sdst_16:7=sdst 11111111 ssrc0_0:8=ssrc0 }" "asm"="s_ashr_i32 {sdst}, {ssrc0}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_ASHR_I32.SOP2.lit.lit_0_lit_1(i32 "enum"="e1" %sdst, i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) literal_32:32=lit 100001100 sdst_16:7=sdst 1111111111111111 }" "asm"="s_ashr_i32 {sdst}, {lit:h32}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_ASHR_I64.SOP2(i32 "enum"="e18" %sdst, i32 "enum"="e18" %ssrc0, i32 "enum"="e3" %ssrc1) "enc"="{ if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if(sdst!=253) if((sdst<256)|(sdst>510)) if((ssrc0<256)|(ssrc0>510)) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) 100001101 sdst_16:7=sdst ssrc1_8:8=ssrc1 ssrc0_0:8=ssrc0 }" "asm"="s_ashr_i64 {sdst}, {ssrc0}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_ASHR_I64.SOP2.lit.lit_0(i32 "enum"="e18" %sdst, i64 "imm" %lit, i32 "enum"="e3" %ssrc1) "enc"="{ if(lit>64) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if(sdst!=253) if((sdst<256)|(sdst>510)) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) literal_32:32=lit 100001101 sdst_16:7=sdst ssrc1_8:8=ssrc1 11111111 }" "asm"="s_ashr_i64 {sdst}, {lit:h32}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_ASHR_I64.SOP2.lit.lit_1(i32 "enum"="e18" %sdst, i32 "enum"="e18" %ssrc0, i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if(sdst!=253) if((sdst<256)|(sdst>510)) if((ssrc0<256)|(ssrc0>510)) literal_32:32=lit 100001101 sdst_16:7=sdst 11111111 ssrc0_0:8=ssrc0 }" "asm"="s_ashr_i64 {sdst}, {ssrc0}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_ASHR_I64.SOP2.lit.lit_0_lit_1(i32 "enum"="e18" %sdst, i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if(sdst!=253) if((sdst<256)|(sdst>510)) literal_32:32=lit 100001101 sdst_16:7=sdst 1111111111111111 }" "asm"="s_ashr_i64 {sdst}, {lit:h32}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_LSHL1_ADD_U32.SOP2(i32 "enum"="e1" %sdst, i32 "enum"="e3" %ssrc0, i32 "enum"="e3" %ssrc1) "enc"="{ if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) 100001110 sdst_16:7=sdst ssrc1_8:8=ssrc1 ssrc0_0:8=ssrc0 }" "asm"="s_lshl1_add_u32 {sdst}, {ssrc0}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_LSHL1_ADD_U32.SOP2.lit.lit_0(i32 "enum"="e1" %sdst, i64 "imm" %lit, i32 "enum"="e3" %ssrc1) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) literal_32:32=lit 100001110 sdst_16:7=sdst ssrc1_8:8=ssrc1 11111111 }" "asm"="s_lshl1_add_u32 {sdst}, {lit:h32}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_LSHL1_ADD_U32.SOP2.lit.lit_1(i32 "enum"="e1" %sdst, i32 "enum"="e3" %ssrc0, i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) literal_32:32=lit 100001110 sdst_16:7=sdst 11111111 ssrc0_0:8=ssrc0 }" "asm"="s_lshl1_add_u32 {sdst}, {ssrc0}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_LSHL1_ADD_U32.SOP2.lit.lit_0_lit_1(i32 "enum"="e1" %sdst, i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) literal_32:32=lit 100001110 sdst_16:7=sdst 1111111111111111 }" "asm"="s_lshl1_add_u32 {sdst}, {lit:h32}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_LSHL2_ADD_U32.SOP2(i32 "enum"="e1" %sdst, i32 "enum"="e3" %ssrc0, i32 "enum"="e3" %ssrc1) "enc"="{ if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) 100001111 sdst_16:7=sdst ssrc1_8:8=ssrc1 ssrc0_0:8=ssrc0 }" "asm"="s_lshl2_add_u32 {sdst}, {ssrc0}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_LSHL2_ADD_U32.SOP2.lit.lit_0(i32 "enum"="e1" %sdst, i64 "imm" %lit, i32 "enum"="e3" %ssrc1) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) literal_32:32=lit 100001111 sdst_16:7=sdst ssrc1_8:8=ssrc1 11111111 }" "asm"="s_lshl2_add_u32 {sdst}, {lit:h32}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_LSHL2_ADD_U32.SOP2.lit.lit_1(i32 "enum"="e1" %sdst, i32 "enum"="e3" %ssrc0, i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) literal_32:32=lit 100001111 sdst_16:7=sdst 11111111 ssrc0_0:8=ssrc0 }" "asm"="s_lshl2_add_u32 {sdst}, {ssrc0}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_LSHL2_ADD_U32.SOP2.lit.lit_0_lit_1(i32 "enum"="e1" %sdst, i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) literal_32:32=lit 100001111 sdst_16:7=sdst 1111111111111111 }" "asm"="s_lshl2_add_u32 {sdst}, {lit:h32}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_LSHL3_ADD_U32.SOP2(i32 "enum"="e1" %sdst, i32 "enum"="e3" %ssrc0, i32 "enum"="e3" %ssrc1) "enc"="{ if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) 100010000 sdst_16:7=sdst ssrc1_8:8=ssrc1 ssrc0_0:8=ssrc0 }" "asm"="s_lshl3_add_u32 {sdst}, {ssrc0}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_LSHL3_ADD_U32.SOP2.lit.lit_0(i32 "enum"="e1" %sdst, i64 "imm" %lit, i32 "enum"="e3" %ssrc1) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) literal_32:32=lit 100010000 sdst_16:7=sdst ssrc1_8:8=ssrc1 11111111 }" "asm"="s_lshl3_add_u32 {sdst}, {lit:h32}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_LSHL3_ADD_U32.SOP2.lit.lit_1(i32 "enum"="e1" %sdst, i32 "enum"="e3" %ssrc0, i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) literal_32:32=lit 100010000 sdst_16:7=sdst 11111111 ssrc0_0:8=ssrc0 }" "asm"="s_lshl3_add_u32 {sdst}, {ssrc0}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_LSHL3_ADD_U32.SOP2.lit.lit_0_lit_1(i32 "enum"="e1" %sdst, i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) literal_32:32=lit 100010000 sdst_16:7=sdst 1111111111111111 }" "asm"="s_lshl3_add_u32 {sdst}, {lit:h32}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_LSHL4_ADD_U32.SOP2(i32 "enum"="e1" %sdst, i32 "enum"="e3" %ssrc0, i32 "enum"="e3" %ssrc1) "enc"="{ if(sdst!=118) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) 100010001 sdst_16:7=sdst ssrc1_8:8=ssrc1 ssrc0_0:8=ssrc0 }" "asm"="s_lshl4_add_u32 {sdst}, {ssrc0}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_LSHL4_ADD_U32.SOP2.lit.lit_0(i32 "enum"="e1" %sdst, i64 "imm" %lit, i32 "enum"="e3" %ssrc1) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) literal_32:32=lit 100010001 sdst_16:7=sdst ssrc1_8:8=ssrc1 11111111 }" "asm"="s_lshl4_add_u32 {sdst}, {lit:h32}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_LSHL4_ADD_U32.SOP2.lit.lit_1(i32 "enum"="e1" %sdst, i32 "enum"="e3" %ssrc0, i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) literal_32:32=lit 100010001 sdst_16:7=sdst 11111111 ssrc0_0:8=ssrc0 }" "asm"="s_lshl4_add_u32 {sdst}, {ssrc0}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_LSHL4_ADD_U32.SOP2.lit.lit_0_lit_1(i32 "enum"="e1" %sdst, i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) literal_32:32=lit 100010001 sdst_16:7=sdst 1111111111111111 }" "asm"="s_lshl4_add_u32 {sdst}, {lit:h32}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_MIN_I32.SOP2(i32 "enum"="e1" %sdst, i32 "enum"="e3" %ssrc0, i32 "enum"="e3" %ssrc1) "enc"="{ if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) 100010010 sdst_16:7=sdst ssrc1_8:8=ssrc1 ssrc0_0:8=ssrc0 }" "asm"="s_min_i32 {sdst}, {ssrc0}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_MIN_I32.SOP2.lit.lit_0(i32 "enum"="e1" %sdst, i64 "imm" %lit, i32 "enum"="e3" %ssrc1) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) literal_32:32=lit 100010010 sdst_16:7=sdst ssrc1_8:8=ssrc1 11111111 }" "asm"="s_min_i32 {sdst}, {lit:h32}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_MIN_I32.SOP2.lit.lit_1(i32 "enum"="e1" %sdst, i32 "enum"="e3" %ssrc0, i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) literal_32:32=lit 100010010 sdst_16:7=sdst 11111111 ssrc0_0:8=ssrc0 }" "asm"="s_min_i32 {sdst}, {ssrc0}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_MIN_I32.SOP2.lit.lit_0_lit_1(i32 "enum"="e1" %sdst, i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) literal_32:32=lit 100010010 sdst_16:7=sdst 1111111111111111 }" "asm"="s_min_i32 {sdst}, {lit:h32}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_MIN_U32.SOP2(i32 "enum"="e1" %sdst, i32 "enum"="e3" %ssrc0, i32 "enum"="e3" %ssrc1) "enc"="{ if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) 100010011 sdst_16:7=sdst ssrc1_8:8=ssrc1 ssrc0_0:8=ssrc0 }" "asm"="s_min_u32 {sdst}, {ssrc0}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_MIN_U32.SOP2.lit.lit_0(i32 "enum"="e1" %sdst, i64 "imm" %lit, i32 "enum"="e3" %ssrc1) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) literal_32:32=lit 100010011 sdst_16:7=sdst ssrc1_8:8=ssrc1 11111111 }" "asm"="s_min_u32 {sdst}, {lit:h32}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_MIN_U32.SOP2.lit.lit_1(i32 "enum"="e1" %sdst, i32 "enum"="e3" %ssrc0, i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) literal_32:32=lit 100010011 sdst_16:7=sdst 11111111 ssrc0_0:8=ssrc0 }" "asm"="s_min_u32 {sdst}, {ssrc0}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_MIN_U32.SOP2.lit.lit_0_lit_1(i32 "enum"="e1" %sdst, i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) literal_32:32=lit 100010011 sdst_16:7=sdst 1111111111111111 }" "asm"="s_min_u32 {sdst}, {lit:h32}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_MAX_I32.SOP2(i32 "enum"="e1" %sdst, i32 "enum"="e3" %ssrc0, i32 "enum"="e3" %ssrc1) "enc"="{ if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) 100010100 sdst_16:7=sdst ssrc1_8:8=ssrc1 ssrc0_0:8=ssrc0 }" "asm"="s_max_i32 {sdst}, {ssrc0}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_MAX_I32.SOP2.lit.lit_0(i32 "enum"="e1" %sdst, i64 "imm" %lit, i32 "enum"="e3" %ssrc1) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) literal_32:32=lit 100010100 sdst_16:7=sdst ssrc1_8:8=ssrc1 11111111 }" "asm"="s_max_i32 {sdst}, {lit:h32}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_MAX_I32.SOP2.lit.lit_1(i32 "enum"="e1" %sdst, i32 "enum"="e3" %ssrc0, i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) literal_32:32=lit 100010100 sdst_16:7=sdst 11111111 ssrc0_0:8=ssrc0 }" "asm"="s_max_i32 {sdst}, {ssrc0}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_MAX_I32.SOP2.lit.lit_0_lit_1(i32 "enum"="e1" %sdst, i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) literal_32:32=lit 100010100 sdst_16:7=sdst 1111111111111111 }" "asm"="s_max_i32 {sdst}, {lit:h32}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_MAX_U32.SOP2(i32 "enum"="e1" %sdst, i32 "enum"="e3" %ssrc0, i32 "enum"="e3" %ssrc1) "enc"="{ if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) 100010101 sdst_16:7=sdst ssrc1_8:8=ssrc1 ssrc0_0:8=ssrc0 }" "asm"="s_max_u32 {sdst}, {ssrc0}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_MAX_U32.SOP2.lit.lit_0(i32 "enum"="e1" %sdst, i64 "imm" %lit, i32 "enum"="e3" %ssrc1) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) literal_32:32=lit 100010101 sdst_16:7=sdst ssrc1_8:8=ssrc1 11111111 }" "asm"="s_max_u32 {sdst}, {lit:h32}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_MAX_U32.SOP2.lit.lit_1(i32 "enum"="e1" %sdst, i32 "enum"="e3" %ssrc0, i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) literal_32:32=lit 100010101 sdst_16:7=sdst 11111111 ssrc0_0:8=ssrc0 }" "asm"="s_max_u32 {sdst}, {ssrc0}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_MAX_U32.SOP2.lit.lit_0_lit_1(i32 "enum"="e1" %sdst, i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) literal_32:32=lit 100010101 sdst_16:7=sdst 1111111111111111 }" "asm"="s_max_u32 {sdst}, {lit:h32}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_AND_B32.SOP2(i32 "enum"="e1" %sdst, i32 "enum"="e3" %ssrc0, i32 "enum"="e3" %ssrc1) "enc"="{ if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) 100010110 sdst_16:7=sdst ssrc1_8:8=ssrc1 ssrc0_0:8=ssrc0 }" "asm"="s_and_b32 {sdst}, {ssrc0}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_AND_B32.SOP2.lit.lit_0(i32 "enum"="e1" %sdst, i64 "imm" %lit, i32 "enum"="e3" %ssrc1) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) literal_32:32=lit 100010110 sdst_16:7=sdst ssrc1_8:8=ssrc1 11111111 }" "asm"="s_and_b32 {sdst}, {lit:h32}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_AND_B32.SOP2.lit.lit_1(i32 "enum"="e1" %sdst, i32 "enum"="e3" %ssrc0, i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) literal_32:32=lit 100010110 sdst_16:7=sdst 11111111 ssrc0_0:8=ssrc0 }" "asm"="s_and_b32 {sdst}, {ssrc0}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_AND_B32.SOP2.lit.lit_0_lit_1(i32 "enum"="e1" %sdst, i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) literal_32:32=lit 100010110 sdst_16:7=sdst 1111111111111111 }" "asm"="s_and_b32 {sdst}, {lit:h32}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_AND_B64.SOP2.lit.lit_1(i32 "enum"="e18" %sdst, i32 "enum"="e18" %ssrc0, i64 "imm" %lit) "enc"="{ if(lit>64) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if(sdst!=253) if((sdst<256)|(sdst>510)) if((ssrc0<256)|(ssrc0>510)) literal_32:32=lit 100010111 sdst_16:7=sdst 11111111 ssrc0_0:8=ssrc0 }" "asm"="s_and_b64 {sdst}, {ssrc0}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_AND_B64.SOP2.lit.lit_0_lit_1(i32 "enum"="e18" %sdst, i64 "imm" %lit) "enc"="{ if(lit>64) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if(sdst!=253) if((sdst<256)|(sdst>510)) literal_32:32=lit 100010111 sdst_16:7=sdst 1111111111111111 }" "asm"="s_and_b64 {sdst}, {lit:h32}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_OR_B32.SOP2(i32 "enum"="e1" %sdst, i32 "enum"="e3" %ssrc0, i32 "enum"="e3" %ssrc1) "enc"="{ if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) 100011000 sdst_16:7=sdst ssrc1_8:8=ssrc1 ssrc0_0:8=ssrc0 }" "asm"="s_or_b32 {sdst}, {ssrc0}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_OR_B32.SOP2.lit.lit_0(i32 "enum"="e1" %sdst, i64 "imm" %lit, i32 "enum"="e3" %ssrc1) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) literal_32:32=lit 100011000 sdst_16:7=sdst ssrc1_8:8=ssrc1 11111111 }" "asm"="s_or_b32 {sdst}, {lit:h32}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_OR_B32.SOP2.lit.lit_1(i32 "enum"="e1" %sdst, i32 "enum"="e3" %ssrc0, i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) literal_32:32=lit 100011000 sdst_16:7=sdst 11111111 ssrc0_0:8=ssrc0 }" "asm"="s_or_b32 {sdst}, {ssrc0}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_OR_B32.SOP2.lit.lit_0_lit_1(i32 "enum"="e1" %sdst, i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) literal_32:32=lit 100011000 sdst_16:7=sdst 1111111111111111 }" "asm"="s_or_b32 {sdst}, {lit:h32}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_OR_B64.SOP2.lit.lit_1(i32 "enum"="e18" %sdst, i32 "enum"="e18" %ssrc0, i64 "imm" %lit) "enc"="{ if(lit>64) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if(sdst!=253) if((sdst<256)|(sdst>510)) if((ssrc0<256)|(ssrc0>510)) literal_32:32=lit 100011001 sdst_16:7=sdst 11111111 ssrc0_0:8=ssrc0 }" "asm"="s_or_b64 {sdst}, {ssrc0}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_OR_B64.SOP2.lit.lit_0_lit_1(i32 "enum"="e18" %sdst, i64 "imm" %lit) "enc"="{ if(lit>64) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if(sdst!=253) if((sdst<256)|(sdst>510)) literal_32:32=lit 100011001 sdst_16:7=sdst 1111111111111111 }" "asm"="s_or_b64 {sdst}, {lit:h32}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_XOR_B32.SOP2(i32 "enum"="e1" %sdst, i32 "enum"="e3" %ssrc0, i32 "enum"="e3" %ssrc1) "enc"="{ if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) 100011010 sdst_16:7=sdst ssrc1_8:8=ssrc1 ssrc0_0:8=ssrc0 }" "asm"="s_xor_b32 {sdst}, {ssrc0}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_XOR_B32.SOP2.lit.lit_0(i32 "enum"="e1" %sdst, i64 "imm" %lit, i32 "enum"="e3" %ssrc1) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) literal_32:32=lit 100011010 sdst_16:7=sdst ssrc1_8:8=ssrc1 11111111 }" "asm"="s_xor_b32 {sdst}, {lit:h32}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_XOR_B32.SOP2.lit.lit_1(i32 "enum"="e1" %sdst, i32 "enum"="e3" %ssrc0, i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) literal_32:32=lit 100011010 sdst_16:7=sdst 11111111 ssrc0_0:8=ssrc0 }" "asm"="s_xor_b32 {sdst}, {ssrc0}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_XOR_B32.SOP2.lit.lit_0_lit_1(i32 "enum"="e1" %sdst, i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) literal_32:32=lit 100011010 sdst_16:7=sdst 1111111111111111 }" "asm"="s_xor_b32 {sdst}, {lit:h32}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_XOR_B64.SOP2.lit.lit_1(i32 "enum"="e18" %sdst, i32 "enum"="e18" %ssrc0, i64 "imm" %lit) "enc"="{ if(lit>64) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if(sdst!=253) if((sdst<256)|(sdst>510)) if((ssrc0<256)|(ssrc0>510)) literal_32:32=lit 100011011 sdst_16:7=sdst 11111111 ssrc0_0:8=ssrc0 }" "asm"="s_xor_b64 {sdst}, {ssrc0}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_XOR_B64.SOP2.lit.lit_0_lit_1(i32 "enum"="e18" %sdst, i64 "imm" %lit) "enc"="{ if(lit>64) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if(sdst!=253) if((sdst<256)|(sdst>510)) literal_32:32=lit 100011011 sdst_16:7=sdst 1111111111111111 }" "asm"="s_xor_b64 {sdst}, {lit:h32}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_NAND_B32.SOP2(i32 "enum"="e1" %sdst, i32 "enum"="e3" %ssrc0, i32 "enum"="e3" %ssrc1) "enc"="{ if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) 100011100 sdst_16:7=sdst ssrc1_8:8=ssrc1 ssrc0_0:8=ssrc0 }" "asm"="s_nand_b32 {sdst}, {ssrc0}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_NAND_B32.SOP2.lit.lit_0(i32 "enum"="e1" %sdst, i64 "imm" %lit, i32 "enum"="e3" %ssrc1) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) literal_32:32=lit 100011100 sdst_16:7=sdst ssrc1_8:8=ssrc1 11111111 }" "asm"="s_nand_b32 {sdst}, {lit:h32}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_NAND_B32.SOP2.lit.lit_1(i32 "enum"="e1" %sdst, i32 "enum"="e3" %ssrc0, i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) literal_32:32=lit 100011100 sdst_16:7=sdst 11111111 ssrc0_0:8=ssrc0 }" "asm"="s_nand_b32 {sdst}, {ssrc0}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_NAND_B32.SOP2.lit.lit_0_lit_1(i32 "enum"="e1" %sdst, i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) literal_32:32=lit 100011100 sdst_16:7=sdst 1111111111111111 }" "asm"="s_nand_b32 {sdst}, {lit:h32}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_NAND_B64.SOP2.lit.lit_1(i32 "enum"="e18" %sdst, i32 "enum"="e18" %ssrc0, i64 "imm" %lit) "enc"="{ if(lit>64) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if(sdst!=253) if((sdst<256)|(sdst>510)) if((ssrc0<256)|(ssrc0>510)) literal_32:32=lit 100011101 sdst_16:7=sdst 11111111 ssrc0_0:8=ssrc0 }" "asm"="s_nand_b64 {sdst}, {ssrc0}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_NAND_B64.SOP2.lit.lit_0_lit_1(i32 "enum"="e18" %sdst, i64 "imm" %lit) "enc"="{ if(lit>64) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if(sdst!=253) if((sdst<256)|(sdst>510)) literal_32:32=lit 100011101 sdst_16:7=sdst 1111111111111111 }" "asm"="s_nand_b64 {sdst}, {lit:h32}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_NOR_B32.SOP2(i32 "enum"="e1" %sdst, i32 "enum"="e3" %ssrc0, i32 "enum"="e3" %ssrc1) "enc"="{ if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) 100011110 sdst_16:7=sdst ssrc1_8:8=ssrc1 ssrc0_0:8=ssrc0 }" "asm"="s_nor_b32 {sdst}, {ssrc0}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_NOR_B32.SOP2.lit.lit_0(i32 "enum"="e1" %sdst, i64 "imm" %lit, i32 "enum"="e3" %ssrc1) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) literal_32:32=lit 100011110 sdst_16:7=sdst ssrc1_8:8=ssrc1 11111111 }" "asm"="s_nor_b32 {sdst}, {lit:h32}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_NOR_B32.SOP2.lit.lit_1(i32 "enum"="e1" %sdst, i32 "enum"="e3" %ssrc0, i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) literal_32:32=lit 100011110 sdst_16:7=sdst 11111111 ssrc0_0:8=ssrc0 }" "asm"="s_nor_b32 {sdst}, {ssrc0}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_NOR_B32.SOP2.lit.lit_0_lit_1(i32 "enum"="e1" %sdst, i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) literal_32:32=lit 100011110 sdst_16:7=sdst 1111111111111111 }" "asm"="s_nor_b32 {sdst}, {lit:h32}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_NOR_B64.SOP2.lit.lit_1(i32 "enum"="e18" %sdst, i32 "enum"="e18" %ssrc0, i64 "imm" %lit) "enc"="{ if(lit>64) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if(sdst!=253) if((sdst<256)|(sdst>510)) if((ssrc0<256)|(ssrc0>510)) literal_32:32=lit 100011111 sdst_16:7=sdst 11111111 ssrc0_0:8=ssrc0 }" "asm"="s_nor_b64 {sdst}, {ssrc0}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_NOR_B64.SOP2.lit.lit_0_lit_1(i32 "enum"="e18" %sdst, i64 "imm" %lit) "enc"="{ if(lit>64) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if(sdst!=253) if((sdst<256)|(sdst>510)) literal_32:32=lit 100011111 sdst_16:7=sdst 1111111111111111 }" "asm"="s_nor_b64 {sdst}, {lit:h32}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_XNOR_B32.SOP2(i32 "enum"="e1" %sdst, i32 "enum"="e3" %ssrc0, i32 "enum"="e3" %ssrc1) "enc"="{ if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) 100100000 sdst_16:7=sdst ssrc1_8:8=ssrc1 ssrc0_0:8=ssrc0 }" "asm"="s_xnor_b32 {sdst}, {ssrc0}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_XNOR_B32.SOP2.lit.lit_0(i32 "enum"="e1" %sdst, i64 "imm" %lit, i32 "enum"="e3" %ssrc1) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) literal_32:32=lit 100100000 sdst_16:7=sdst ssrc1_8:8=ssrc1 11111111 }" "asm"="s_xnor_b32 {sdst}, {lit:h32}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_XNOR_B32.SOP2.lit.lit_1(i32 "enum"="e1" %sdst, i32 "enum"="e3" %ssrc0, i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) literal_32:32=lit 100100000 sdst_16:7=sdst 11111111 ssrc0_0:8=ssrc0 }" "asm"="s_xnor_b32 {sdst}, {ssrc0}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_XNOR_B32.SOP2.lit.lit_0_lit_1(i32 "enum"="e1" %sdst, i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) literal_32:32=lit 100100000 sdst_16:7=sdst 1111111111111111 }" "asm"="s_xnor_b32 {sdst}, {lit:h32}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_XNOR_B64.SOP2.lit.lit_1(i32 "enum"="e18" %sdst, i32 "enum"="e18" %ssrc0, i64 "imm" %lit) "enc"="{ if(lit>64) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if(sdst!=253) if((sdst<256)|(sdst>510)) if((ssrc0<256)|(ssrc0>510)) literal_32:32=lit 100100001 sdst_16:7=sdst 11111111 ssrc0_0:8=ssrc0 }" "asm"="s_xnor_b64 {sdst}, {ssrc0}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_XNOR_B64.SOP2.lit.lit_0_lit_1(i32 "enum"="e18" %sdst, i64 "imm" %lit) "enc"="{ if(lit>64) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if(sdst!=253) if((sdst<256)|(sdst>510)) literal_32:32=lit 100100001 sdst_16:7=sdst 1111111111111111 }" "asm"="s_xnor_b64 {sdst}, {lit:h32}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_AND_NOT1_B32.SOP2(i32 "enum"="e1" %sdst, i32 "enum"="e3" %ssrc0, i32 "enum"="e3" %ssrc1) "enc"="{ if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) if(ssrc1!=185) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) 100100010 sdst_16:7=sdst ssrc1_8:8=ssrc1 ssrc0_0:8=ssrc0 }" "asm"="s_and_not1_b32 {sdst}, {ssrc0}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_AND_NOT1_B32.SOP2.lit.lit_0(i32 "enum"="e1" %sdst, i64 "imm" %lit, i32 "enum"="e3" %ssrc1) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) literal_32:32=lit 100100010 sdst_16:7=sdst ssrc1_8:8=ssrc1 11111111 }" "asm"="s_and_not1_b32 {sdst}, {lit:h32}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_AND_NOT1_B32.SOP2.lit.lit_1(i32 "enum"="e1" %sdst, i32 "enum"="e3" %ssrc0, i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) literal_32:32=lit 100100010 sdst_16:7=sdst 11111111 ssrc0_0:8=ssrc0 }" "asm"="s_and_not1_b32 {sdst}, {ssrc0}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_AND_NOT1_B32.SOP2.lit.lit_0_lit_1(i32 "enum"="e1" %sdst, i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=65532) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) literal_32:32=lit 100100010 sdst_16:7=sdst 1111111111111111 }" "asm"="s_and_not1_b32 {sdst}, {lit:h32}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_AND_NOT1_B64.SOP2.lit.lit_1(i32 "enum"="e18" %sdst, i32 "enum"="e18" %ssrc0, i64 "imm" %lit) "enc"="{ if(lit>64) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if(sdst!=253) if((sdst<256)|(sdst>510)) if((ssrc0<256)|(ssrc0>510)) literal_32:32=lit 100100011 sdst_16:7=sdst 11111111 ssrc0_0:8=ssrc0 }" "asm"="s_and_not1_b64 {sdst}, {ssrc0}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_AND_NOT1_B64.SOP2.lit.lit_0_lit_1(i32 "enum"="e18" %sdst, i64 "imm" %lit) "enc"="{ if(lit>64) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if(sdst!=253) if((sdst<256)|(sdst>510)) literal_32:32=lit 100100011 sdst_16:7=sdst 1111111111111111 }" "asm"="s_and_not1_b64 {sdst}, {lit:h32}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_OR_NOT1_B32.SOP2(i32 "enum"="e1" %sdst, i32 "enum"="e3" %ssrc0, i32 "enum"="e3" %ssrc1) "enc"="{ if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) 100100100 sdst_16:7=sdst ssrc1_8:8=ssrc1 ssrc0_0:8=ssrc0 }" "asm"="s_or_not1_b32 {sdst}, {ssrc0}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_OR_NOT1_B32.SOP2.lit.lit_0(i32 "enum"="e1" %sdst, i64 "imm" %lit, i32 "enum"="e3" %ssrc1) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) literal_32:32=lit 100100100 sdst_16:7=sdst ssrc1_8:8=ssrc1 11111111 }" "asm"="s_or_not1_b32 {sdst}, {lit:h32}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_OR_NOT1_B32.SOP2.lit.lit_1(i32 "enum"="e1" %sdst, i32 "enum"="e3" %ssrc0, i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) literal_32:32=lit 100100100 sdst_16:7=sdst 11111111 ssrc0_0:8=ssrc0 }" "asm"="s_or_not1_b32 {sdst}, {ssrc0}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_OR_NOT1_B32.SOP2.lit.lit_0_lit_1(i32 "enum"="e1" %sdst, i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) literal_32:32=lit 100100100 sdst_16:7=sdst 1111111111111111 }" "asm"="s_or_not1_b32 {sdst}, {lit:h32}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_OR_NOT1_B64.SOP2.lit.lit_1(i32 "enum"="e18" %sdst, i32 "enum"="e18" %ssrc0, i64 "imm" %lit) "enc"="{ if(lit>64) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if(sdst!=253) if((sdst<256)|(sdst>510)) if((ssrc0<256)|(ssrc0>510)) literal_32:32=lit 100100101 sdst_16:7=sdst 11111111 ssrc0_0:8=ssrc0 }" "asm"="s_or_not1_b64 {sdst}, {ssrc0}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_OR_NOT1_B64.SOP2.lit.lit_0_lit_1(i32 "enum"="e18" %sdst, i64 "imm" %lit) "enc"="{ if(lit>64) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if(sdst!=253) if((sdst<256)|(sdst>510)) literal_32:32=lit 100100101 sdst_16:7=sdst 1111111111111111 }" "asm"="s_or_not1_b64 {sdst}, {lit:h32}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_BFE_U32.SOP2(i32 "enum"="e1" %sdst, i32 "enum"="e3" %ssrc0, i32 "enum"="e3" %ssrc1) "enc"="{ if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) if(ssrc1!=174) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) 100100110 sdst_16:7=sdst ssrc1_8:8=ssrc1 ssrc0_0:8=ssrc0 }" "asm"="s_bfe_u32 {sdst}, {ssrc0}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_BFE_U32.SOP2.lit.lit_0(i32 "enum"="e1" %sdst, i64 "imm" %lit, i32 "enum"="e3" %ssrc1) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) literal_32:32=lit 100100110 sdst_16:7=sdst ssrc1_8:8=ssrc1 11111111 }" "asm"="s_bfe_u32 {sdst}, {lit:h32}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_BFE_U32.SOP2.lit.lit_1(i32 "enum"="e1" %sdst, i32 "enum"="e3" %ssrc0, i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) literal_32:32=lit 100100110 sdst_16:7=sdst 11111111 ssrc0_0:8=ssrc0 }" "asm"="s_bfe_u32 {sdst}, {ssrc0}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_BFE_U32.SOP2.lit.lit_0_lit_1(i32 "enum"="e1" %sdst, i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) literal_32:32=lit 100100110 sdst_16:7=sdst 1111111111111111 }" "asm"="s_bfe_u32 {sdst}, {lit:h32}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_BFE_I32.SOP2(i32 "enum"="e1" %sdst, i32 "enum"="e3" %ssrc0, i32 "enum"="e3" %ssrc1) "enc"="{ if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) 100100111 sdst_16:7=sdst ssrc1_8:8=ssrc1 ssrc0_0:8=ssrc0 }" "asm"="s_bfe_i32 {sdst}, {ssrc0}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_BFE_I32.SOP2.lit.lit_0(i32 "enum"="e1" %sdst, i64 "imm" %lit, i32 "enum"="e3" %ssrc1) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) literal_32:32=lit 100100111 sdst_16:7=sdst ssrc1_8:8=ssrc1 11111111 }" "asm"="s_bfe_i32 {sdst}, {lit:h32}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_BFE_I32.SOP2.lit.lit_1(i32 "enum"="e1" %sdst, i32 "enum"="e3" %ssrc0, i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) literal_32:32=lit 100100111 sdst_16:7=sdst 11111111 ssrc0_0:8=ssrc0 }" "asm"="s_bfe_i32 {sdst}, {ssrc0}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_BFE_I32.SOP2.lit.lit_0_lit_1(i32 "enum"="e1" %sdst, i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) literal_32:32=lit 100100111 sdst_16:7=sdst 1111111111111111 }" "asm"="s_bfe_i32 {sdst}, {lit:h32}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_BFE_U64.SOP2(i32 "enum"="e18" %sdst, i32 "enum"="e18" %ssrc0, i32 "enum"="e3" %ssrc1) "enc"="{ if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if(sdst!=253) if((sdst<256)|(sdst>510)) if((ssrc0<256)|(ssrc0>510)) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) 100101000 sdst_16:7=sdst ssrc1_8:8=ssrc1 ssrc0_0:8=ssrc0 }" "asm"="s_bfe_u64 {sdst}, {ssrc0}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_BFE_U64.SOP2.lit.lit_0(i32 "enum"="e18" %sdst, i64 "imm" %lit, i32 "enum"="e3" %ssrc1) "enc"="{ if(lit>64) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if(sdst!=253) if((sdst<256)|(sdst>510)) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) literal_32:32=lit 100101000 sdst_16:7=sdst ssrc1_8:8=ssrc1 11111111 }" "asm"="s_bfe_u64 {sdst}, {lit:h32}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_BFE_U64.SOP2.lit.lit_1(i32 "enum"="e18" %sdst, i32 "enum"="e18" %ssrc0, i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if(sdst!=253) if((sdst<256)|(sdst>510)) if((ssrc0<256)|(ssrc0>510)) literal_32:32=lit 100101000 sdst_16:7=sdst 11111111 ssrc0_0:8=ssrc0 }" "asm"="s_bfe_u64 {sdst}, {ssrc0}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_BFE_U64.SOP2.lit.lit_0_lit_1(i32 "enum"="e18" %sdst, i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if(sdst!=253) if((sdst<256)|(sdst>510)) literal_32:32=lit 100101000 sdst_16:7=sdst 1111111111111111 }" "asm"="s_bfe_u64 {sdst}, {lit:h32}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_BFE_I64.SOP2(i32 "enum"="e18" %sdst, i32 "enum"="e18" %ssrc0, i32 "enum"="e3" %ssrc1) "enc"="{ if(sdst!=46) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if(sdst!=253) if((sdst<256)|(sdst>510)) if((ssrc0<256)|(ssrc0>510)) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) 100101001 sdst_16:7=sdst ssrc1_8:8=ssrc1 ssrc0_0:8=ssrc0 }" "asm"="s_bfe_i64 {sdst}, {ssrc0}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_BFE_I64.SOP2.lit.lit_0(i32 "enum"="e18" %sdst, i64 "imm" %lit, i32 "enum"="e3" %ssrc1) "enc"="{ if(lit>64) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if(sdst!=253) if((sdst<256)|(sdst>510)) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) literal_32:32=lit 100101001 sdst_16:7=sdst ssrc1_8:8=ssrc1 11111111 }" "asm"="s_bfe_i64 {sdst}, {lit:h32}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_BFE_I64.SOP2.lit.lit_1(i32 "enum"="e18" %sdst, i32 "enum"="e18" %ssrc0, i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if(sdst!=253) if((sdst<256)|(sdst>510)) if((ssrc0<256)|(ssrc0>510)) literal_32:32=lit 100101001 sdst_16:7=sdst 11111111 ssrc0_0:8=ssrc0 }" "asm"="s_bfe_i64 {sdst}, {ssrc0}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_BFE_I64.SOP2.lit.lit_0_lit_1(i32 "enum"="e18" %sdst, i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if(sdst!=253) if((sdst<256)|(sdst>510)) literal_32:32=lit 100101001 sdst_16:7=sdst 1111111111111111 }" "asm"="s_bfe_i64 {sdst}, {lit:h32}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_BFM_B32.SOP2(i32 "enum"="e1" %sdst, i32 "enum"="e3" %ssrc0, i32 "enum"="e3" %ssrc1) "enc"="{ if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) 100101010 sdst_16:7=sdst ssrc1_8:8=ssrc1 ssrc0_0:8=ssrc0 }" "asm"="s_bfm_b32 {sdst}, {ssrc0}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_BFM_B32.SOP2.lit.lit_0(i32 "enum"="e1" %sdst, i64 "imm" %lit, i32 "enum"="e3" %ssrc1) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) literal_32:32=lit 100101010 sdst_16:7=sdst ssrc1_8:8=ssrc1 11111111 }" "asm"="s_bfm_b32 {sdst}, {lit:h32}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_BFM_B32.SOP2.lit.lit_1(i32 "enum"="e1" %sdst, i32 "enum"="e3" %ssrc0, i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) literal_32:32=lit 100101010 sdst_16:7=sdst 11111111 ssrc0_0:8=ssrc0 }" "asm"="s_bfm_b32 {sdst}, {ssrc0}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_BFM_B32.SOP2.lit.lit_0_lit_1(i32 "enum"="e1" %sdst, i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) literal_32:32=lit 100101010 sdst_16:7=sdst 1111111111111111 }" "asm"="s_bfm_b32 {sdst}, {lit:h32}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_BFM_B64.SOP2(i32 "enum"="e18" %sdst, i32 "enum"="e3" %ssrc0, i32 "enum"="e3" %ssrc1) "enc"="{ if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if(sdst!=253) if((sdst<256)|(sdst>510)) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) 100101011 sdst_16:7=sdst ssrc1_8:8=ssrc1 ssrc0_0:8=ssrc0 }" "asm"="s_bfm_b64 {sdst}, {ssrc0}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_BFM_B64.SOP2.lit.lit_0(i32 "enum"="e18" %sdst, i64 "imm" %lit, i32 "enum"="e3" %ssrc1) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if(sdst!=253) if((sdst<256)|(sdst>510)) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) literal_32:32=lit 100101011 sdst_16:7=sdst ssrc1_8:8=ssrc1 11111111 }" "asm"="s_bfm_b64 {sdst}, {lit:h32}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_BFM_B64.SOP2.lit.lit_1(i32 "enum"="e18" %sdst, i32 "enum"="e3" %ssrc0, i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if(sdst!=253) if((sdst<256)|(sdst>510)) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) literal_32:32=lit 100101011 sdst_16:7=sdst 11111111 ssrc0_0:8=ssrc0 }" "asm"="s_bfm_b64 {sdst}, {ssrc0}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_BFM_B64.SOP2.lit.lit_0_lit_1(i32 "enum"="e18" %sdst, i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if(sdst!=253) if((sdst<256)|(sdst>510)) literal_32:32=lit 100101011 sdst_16:7=sdst 1111111111111111 }" "asm"="s_bfm_b64 {sdst}, {lit:h32}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_MUL_I32.SOP2(i32 "enum"="e1" %sdst, i32 "enum"="e3" %ssrc0, i32 "enum"="e3" %ssrc1) "enc"="{ if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) 100101100 sdst_16:7=sdst ssrc1_8:8=ssrc1 ssrc0_0:8=ssrc0 }" "asm"="s_mul_i32 {sdst}, {ssrc0}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_MUL_I32.SOP2.lit.lit_0(i32 "enum"="e1" %sdst, i64 "imm" %lit, i32 "enum"="e3" %ssrc1) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) literal_32:32=lit 100101100 sdst_16:7=sdst ssrc1_8:8=ssrc1 11111111 }" "asm"="s_mul_i32 {sdst}, {lit:h32}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_MUL_I32.SOP2.lit.lit_1(i32 "enum"="e1" %sdst, i32 "enum"="e3" %ssrc0, i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) literal_32:32=lit 100101100 sdst_16:7=sdst 11111111 ssrc0_0:8=ssrc0 }" "asm"="s_mul_i32 {sdst}, {ssrc0}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_MUL_I32.SOP2.lit.lit_0_lit_1(i32 "enum"="e1" %sdst, i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) literal_32:32=lit 100101100 sdst_16:7=sdst 1111111111111111 }" "asm"="s_mul_i32 {sdst}, {lit:h32}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_MUL_HI_U32.SOP2(i32 "enum"="e1" %sdst, i32 "enum"="e3" %ssrc0, i32 "enum"="e3" %ssrc1) "enc"="{ if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) 100101101 sdst_16:7=sdst ssrc1_8:8=ssrc1 ssrc0_0:8=ssrc0 }" "asm"="s_mul_hi_u32 {sdst}, {ssrc0}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_MUL_HI_U32.SOP2.lit.lit_0(i32 "enum"="e1" %sdst, i64 "imm" %lit, i32 "enum"="e3" %ssrc1) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) literal_32:32=lit 100101101 sdst_16:7=sdst ssrc1_8:8=ssrc1 11111111 }" "asm"="s_mul_hi_u32 {sdst}, {lit:h32}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_MUL_HI_U32.SOP2.lit.lit_1(i32 "enum"="e1" %sdst, i32 "enum"="e3" %ssrc0, i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) literal_32:32=lit 100101101 sdst_16:7=sdst 11111111 ssrc0_0:8=ssrc0 }" "asm"="s_mul_hi_u32 {sdst}, {ssrc0}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_MUL_HI_U32.SOP2.lit.lit_0_lit_1(i32 "enum"="e1" %sdst, i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) literal_32:32=lit 100101101 sdst_16:7=sdst 1111111111111111 }" "asm"="s_mul_hi_u32 {sdst}, {lit:h32}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_MUL_HI_I32.SOP2(i32 "enum"="e1" %sdst, i32 "enum"="e3" %ssrc0, i32 "enum"="e3" %ssrc1) "enc"="{ if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) 100101110 sdst_16:7=sdst ssrc1_8:8=ssrc1 ssrc0_0:8=ssrc0 }" "asm"="s_mul_hi_i32 {sdst}, {ssrc0}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_MUL_HI_I32.SOP2.lit.lit_0(i32 "enum"="e1" %sdst, i64 "imm" %lit, i32 "enum"="e3" %ssrc1) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) literal_32:32=lit 100101110 sdst_16:7=sdst ssrc1_8:8=ssrc1 11111111 }" "asm"="s_mul_hi_i32 {sdst}, {lit:h32}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_MUL_HI_I32.SOP2.lit.lit_1(i32 "enum"="e1" %sdst, i32 "enum"="e3" %ssrc0, i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) literal_32:32=lit 100101110 sdst_16:7=sdst 11111111 ssrc0_0:8=ssrc0 }" "asm"="s_mul_hi_i32 {sdst}, {ssrc0}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_MUL_HI_I32.SOP2.lit.lit_0_lit_1(i32 "enum"="e1" %sdst, i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) literal_32:32=lit 100101110 sdst_16:7=sdst 1111111111111111 }" "asm"="s_mul_hi_i32 {sdst}, {lit:h32}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_CSELECT_B32.SOP2(i32 "enum"="e1" %sdst, i32 "enum"="e3" %ssrc0, i32 "enum"="e3" %ssrc1) "enc"="{ if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) 100110000 sdst_16:7=sdst ssrc1_8:8=ssrc1 ssrc0_0:8=ssrc0 }" "asm"="s_cselect_b32 {sdst}, {ssrc0}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_CSELECT_B32.SOP2.lit.lit_0(i32 "enum"="e1" %sdst, i64 "imm" %lit, i32 "enum"="e3" %ssrc1) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) literal_32:32=lit 100110000 sdst_16:7=sdst ssrc1_8:8=ssrc1 11111111 }" "asm"="s_cselect_b32 {sdst}, {lit:h32}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_CSELECT_B32.SOP2.lit.lit_1(i32 "enum"="e1" %sdst, i32 "enum"="e3" %ssrc0, i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) literal_32:32=lit 100110000 sdst_16:7=sdst 11111111 ssrc0_0:8=ssrc0 }" "asm"="s_cselect_b32 {sdst}, {ssrc0}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_CSELECT_B32.SOP2.lit.lit_0_lit_1(i32 "enum"="e1" %sdst, i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) literal_32:32=lit 100110000 sdst_16:7=sdst 1111111111111111 }" "asm"="s_cselect_b32 {sdst}, {lit:h32}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_CSELECT_B64.SOP2.lit.lit_1(i32 "enum"="e18" %sdst, i32 "enum"="e18" %ssrc0, i64 "imm" %lit) "enc"="{ if(lit>64) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if(sdst!=253) if((sdst<256)|(sdst>510)) if((ssrc0<256)|(ssrc0>510)) literal_32:32=lit 100110001 sdst_16:7=sdst 11111111 ssrc0_0:8=ssrc0 }" "asm"="s_cselect_b64 {sdst}, {ssrc0}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_CSELECT_B64.SOP2.lit.lit_0_lit_1(i32 "enum"="e18" %sdst, i64 "imm" %lit) "enc"="{ if(lit>64) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if(sdst!=253) if((sdst<256)|(sdst>510)) literal_32:32=lit 100110001 sdst_16:7=sdst 1111111111111111 }" "asm"="s_cselect_b64 {sdst}, {lit:h32}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_PACK_LL_B32_B16.SOP2(i32 "enum"="e1" %sdst, i32 "enum"="e3" %ssrc0, i32 "enum"="e3" %ssrc1) "enc"="{ if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) 100110010 sdst_16:7=sdst ssrc1_8:8=ssrc1 ssrc0_0:8=ssrc0 }" "asm"="s_pack_ll_b32_b16 {sdst}, {ssrc0}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_PACK_LL_B32_B16.SOP2.lit.lit_0(i32 "enum"="e1" %sdst, i64 "imm" %lit, i32 "enum"="e3" %ssrc1) "enc"="{ if(lit<65536) if(lit>64) if(lit!=65520) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) literal_32:32=lit 100110010 sdst_16:7=sdst ssrc1_8:8=ssrc1 11111111 }" "asm"="s_pack_ll_b32_b16 {sdst}, {lit:h32}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_PACK_LL_B32_B16.SOP2.lit.lit_1(i32 "enum"="e1" %sdst, i32 "enum"="e3" %ssrc0, i64 "imm" %lit) "enc"="{ if(lit<65536) if(lit>64) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) literal_32:32=lit 100110010 sdst_16:7=sdst 11111111 ssrc0_0:8=ssrc0 }" "asm"="s_pack_ll_b32_b16 {sdst}, {ssrc0}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_PACK_LL_B32_B16.SOP2.lit.lit_0_lit_1(i32 "enum"="e1" %sdst, i64 "imm" %lit) "enc"="{ if(lit<65536) if(lit>64) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) literal_32:32=lit 100110010 sdst_16:7=sdst 1111111111111111 }" "asm"="s_pack_ll_b32_b16 {sdst}, {lit:h32}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_PACK_LH_B32_B16.SOP2(i32 "enum"="e1" %sdst, i32 "enum"="e3" %ssrc0, i32 "enum"="e3" %ssrc1) "enc"="{ if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) 100110011 sdst_16:7=sdst ssrc1_8:8=ssrc1 ssrc0_0:8=ssrc0 }" "asm"="s_pack_lh_b32_b16 {sdst}, {ssrc0}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_PACK_LH_B32_B16.SOP2.lit.lit_0(i32 "enum"="e1" %sdst, i64 "imm" %lit, i32 "enum"="e3" %ssrc1) "enc"="{ if(lit<65536) if(lit>64) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) literal_32:32=lit 100110011 sdst_16:7=sdst ssrc1_8:8=ssrc1 11111111 }" "asm"="s_pack_lh_b32_b16 {sdst}, {lit:h32}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_PACK_LH_B32_B16.SOP2.lit.lit_1(i32 "enum"="e1" %sdst, i32 "enum"="e3" %ssrc0, i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) literal_32:32=lit 100110011 sdst_16:7=sdst 11111111 ssrc0_0:8=ssrc0 }" "asm"="s_pack_lh_b32_b16 {sdst}, {ssrc0}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_PACK_LH_B32_B16.SOP2.lit.lit_0_lit_1(i32 "enum"="e1" %sdst, i64 "imm" %lit) "enc"="{ if(lit<65536) if(lit>64) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) literal_32:32=lit 100110011 sdst_16:7=sdst 1111111111111111 }" "asm"="s_pack_lh_b32_b16 {sdst}, {lit:h32}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_PACK_HH_B32_B16.SOP2(i32 "enum"="e1" %sdst, i32 "enum"="e3" %ssrc0, i32 "enum"="e3" %ssrc1) "enc"="{ if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) 100110100 sdst_16:7=sdst ssrc1_8:8=ssrc1 ssrc0_0:8=ssrc0 }" "asm"="s_pack_hh_b32_b16 {sdst}, {ssrc0}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_PACK_HH_B32_B16.SOP2.lit.lit_0(i32 "enum"="e1" %sdst, i64 "imm" %lit, i32 "enum"="e3" %ssrc1) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) literal_32:32=lit 100110100 sdst_16:7=sdst ssrc1_8:8=ssrc1 11111111 }" "asm"="s_pack_hh_b32_b16 {sdst}, {lit:h32}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_PACK_HH_B32_B16.SOP2.lit.lit_1(i32 "enum"="e1" %sdst, i32 "enum"="e3" %ssrc0, i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) literal_32:32=lit 100110100 sdst_16:7=sdst 11111111 ssrc0_0:8=ssrc0 }" "asm"="s_pack_hh_b32_b16 {sdst}, {ssrc0}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_PACK_HH_B32_B16.SOP2.lit.lit_0_lit_1(i32 "enum"="e1" %sdst, i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) literal_32:32=lit 100110100 sdst_16:7=sdst 1111111111111111 }" "asm"="s_pack_hh_b32_b16 {sdst}, {lit:h32}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_PACK_HL_B32_B16.SOP2(i32 "enum"="e1" %sdst, i32 "enum"="e3" %ssrc0, i32 "enum"="e3" %ssrc1) "enc"="{ if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) 100110101 sdst_16:7=sdst ssrc1_8:8=ssrc1 ssrc0_0:8=ssrc0 }" "asm"="s_pack_hl_b32_b16 {sdst}, {ssrc0}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_PACK_HL_B32_B16.SOP2.lit.lit_0(i32 "enum"="e1" %sdst, i64 "imm" %lit, i32 "enum"="e3" %ssrc1) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) literal_32:32=lit 100110101 sdst_16:7=sdst ssrc1_8:8=ssrc1 11111111 }" "asm"="s_pack_hl_b32_b16 {sdst}, {lit:h32}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_PACK_HL_B32_B16.SOP2.lit.lit_1(i32 "enum"="e1" %sdst, i32 "enum"="e3" %ssrc0, i64 "imm" %lit) "enc"="{ if(lit<65536) if(lit>64) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) literal_32:32=lit 100110101 sdst_16:7=sdst 11111111 ssrc0_0:8=ssrc0 }" "asm"="s_pack_hl_b32_b16 {sdst}, {ssrc0}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_PACK_HL_B32_B16.SOP2.lit.lit_0_lit_1(i32 "enum"="e1" %sdst, i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) literal_32:32=lit 100110101 sdst_16:7=sdst 1111111111111111 }" "asm"="s_pack_hl_b32_b16 {sdst}, {lit:h32}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_ADD_F32.SOP2(i32 "enum"="e1" %sdst, i32 "enum"="e3" %ssrc0, i32 "enum"="e3" %ssrc1) "enc"="{ if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) 101000000 sdst_16:7=sdst ssrc1_8:8=ssrc1 ssrc0_0:8=ssrc0 }" "asm"="s_add_f32 {sdst}, {ssrc0}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_ADD_F32.SOP2.lit.lit_0(i32 "enum"="e1" %sdst, i64 "imm" %lit, i32 "enum"="e3" %ssrc1) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) literal_32:32=lit 101000000 sdst_16:7=sdst ssrc1_8:8=ssrc1 11111111 }" "asm"="s_add_f32 {sdst}, {lit:h32}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_ADD_F32.SOP2.lit.lit_1(i32 "enum"="e1" %sdst, i32 "enum"="e3" %ssrc0, i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) literal_32:32=lit 101000000 sdst_16:7=sdst 11111111 ssrc0_0:8=ssrc0 }" "asm"="s_add_f32 {sdst}, {ssrc0}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_ADD_F32.SOP2.lit.lit_0_lit_1(i32 "enum"="e1" %sdst, i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) literal_32:32=lit 101000000 sdst_16:7=sdst 1111111111111111 }" "asm"="s_add_f32 {sdst}, {lit:h32}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_SUB_F32.SOP2(i32 "enum"="e1" %sdst, i32 "enum"="e3" %ssrc0, i32 "enum"="e3" %ssrc1) "enc"="{ if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) if(ssrc1!=166) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) 101000001 sdst_16:7=sdst ssrc1_8:8=ssrc1 ssrc0_0:8=ssrc0 }" "asm"="s_sub_f32 {sdst}, {ssrc0}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_SUB_F32.SOP2.lit.lit_0(i32 "enum"="e1" %sdst, i64 "imm" %lit, i32 "enum"="e3" %ssrc1) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) literal_32:32=lit 101000001 sdst_16:7=sdst ssrc1_8:8=ssrc1 11111111 }" "asm"="s_sub_f32 {sdst}, {lit:h32}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_SUB_F32.SOP2.lit.lit_1(i32 "enum"="e1" %sdst, i32 "enum"="e3" %ssrc0, i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) literal_32:32=lit 101000001 sdst_16:7=sdst 11111111 ssrc0_0:8=ssrc0 }" "asm"="s_sub_f32 {sdst}, {ssrc0}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_SUB_F32.SOP2.lit.lit_0_lit_1(i32 "enum"="e1" %sdst, i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) literal_32:32=lit 101000001 sdst_16:7=sdst 1111111111111111 }" "asm"="s_sub_f32 {sdst}, {lit:h32}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_MIN_F32.SOP2(i32 "enum"="e1" %sdst, i32 "enum"="e3" %ssrc0, i32 "enum"="e3" %ssrc1) "enc"="{ if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) if(ssrc1!=66) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) 101000010 sdst_16:7=sdst ssrc1_8:8=ssrc1 ssrc0_0:8=ssrc0 }" "asm"="s_min_f32 {sdst}, {ssrc0}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_MIN_F32.SOP2.lit.lit_0(i32 "enum"="e1" %sdst, i64 "imm" %lit, i32 "enum"="e3" %ssrc1) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) literal_32:32=lit 101000010 sdst_16:7=sdst ssrc1_8:8=ssrc1 11111111 }" "asm"="s_min_f32 {sdst}, {lit:h32}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_MIN_F32.SOP2.lit.lit_1(i32 "enum"="e1" %sdst, i32 "enum"="e3" %ssrc0, i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) literal_32:32=lit 101000010 sdst_16:7=sdst 11111111 ssrc0_0:8=ssrc0 }" "asm"="s_min_f32 {sdst}, {ssrc0}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_MIN_F32.SOP2.lit.lit_0_lit_1(i32 "enum"="e1" %sdst, i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) literal_32:32=lit 101000010 sdst_16:7=sdst 1111111111111111 }" "asm"="s_min_f32 {sdst}, {lit:h32}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_MAX_F32.SOP2(i32 "enum"="e1" %sdst, i32 "enum"="e3" %ssrc0, i32 "enum"="e3" %ssrc1) "enc"="{ if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) 101000011 sdst_16:7=sdst ssrc1_8:8=ssrc1 ssrc0_0:8=ssrc0 }" "asm"="s_max_f32 {sdst}, {ssrc0}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_MAX_F32.SOP2.lit.lit_0(i32 "enum"="e1" %sdst, i64 "imm" %lit, i32 "enum"="e3" %ssrc1) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) literal_32:32=lit 101000011 sdst_16:7=sdst ssrc1_8:8=ssrc1 11111111 }" "asm"="s_max_f32 {sdst}, {lit:h32}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_MAX_F32.SOP2.lit.lit_1(i32 "enum"="e1" %sdst, i32 "enum"="e3" %ssrc0, i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=65527) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) literal_32:32=lit 101000011 sdst_16:7=sdst 11111111 ssrc0_0:8=ssrc0 }" "asm"="s_max_f32 {sdst}, {ssrc0}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_MAX_F32.SOP2.lit.lit_0_lit_1(i32 "enum"="e1" %sdst, i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) literal_32:32=lit 101000011 sdst_16:7=sdst 1111111111111111 }" "asm"="s_max_f32 {sdst}, {lit:h32}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_MUL_F32.SOP2(i32 "enum"="e1" %sdst, i32 "enum"="e3" %ssrc0, i32 "enum"="e3" %ssrc1) "enc"="{ if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) 101000100 sdst_16:7=sdst ssrc1_8:8=ssrc1 ssrc0_0:8=ssrc0 }" "asm"="s_mul_f32 {sdst}, {ssrc0}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_MUL_F32.SOP2.lit.lit_0(i32 "enum"="e1" %sdst, i64 "imm" %lit, i32 "enum"="e3" %ssrc1) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) literal_32:32=lit 101000100 sdst_16:7=sdst ssrc1_8:8=ssrc1 11111111 }" "asm"="s_mul_f32 {sdst}, {lit:h32}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_MUL_F32.SOP2.lit.lit_1(i32 "enum"="e1" %sdst, i32 "enum"="e3" %ssrc0, i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) literal_32:32=lit 101000100 sdst_16:7=sdst 11111111 ssrc0_0:8=ssrc0 }" "asm"="s_mul_f32 {sdst}, {ssrc0}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_MUL_F32.SOP2.lit.lit_0_lit_1(i32 "enum"="e1" %sdst, i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) literal_32:32=lit 101000100 sdst_16:7=sdst 1111111111111111 }" "asm"="s_mul_f32 {sdst}, {lit:h32}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_FMAAK_F32.SOP2.lit(i32 "enum"="e1" %sdst, i32 "enum"="e3" %ssrc0, i32 "enum"="e3" %ssrc1, i64 "imm" %lit) "enc"="{ if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) literal_32:32=lit 101000101 sdst_16:7=sdst ssrc1_8:8=ssrc1 ssrc0_0:8=ssrc0 }" "asm"="s_fmaak_f32 {sdst}, {ssrc0}, {ssrc1}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_FMAAK_F32.SOP2.lit.lit_0(i32 "enum"="e1" %sdst, i64 "imm" %lit, i32 "enum"="e3" %ssrc1) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) literal_32:32=lit 101000101 sdst_16:7=sdst ssrc1_8:8=ssrc1 11111111 }" "asm"="s_fmaak_f32 {sdst}, {lit:h32}, {ssrc1}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_FMAAK_F32.SOP2.lit.lit_1(i32 "enum"="e1" %sdst, i32 "enum"="e3" %ssrc0, i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) literal_32:32=lit 101000101 sdst_16:7=sdst 11111111 ssrc0_0:8=ssrc0 }" "asm"="s_fmaak_f32 {sdst}, {ssrc0}, {lit:h32}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_FMAAK_F32.SOP2.lit.lit_0_lit_1(i32 "enum"="e1" %sdst, i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) literal_32:32=lit 101000101 sdst_16:7=sdst 1111111111111111 }" "asm"="s_fmaak_f32 {sdst}, {lit:h32}, {lit:h32}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_FMAMK_F32.SOP2.lit(i32 "enum"="e1" %sdst, i32 "enum"="e3" %ssrc0, i64 "imm" %lit, i32 "enum"="e3" %ssrc1) "enc"="{ if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) literal_32:32=lit 101000110 sdst_16:7=sdst ssrc1_8:8=ssrc1 ssrc0_0:8=ssrc0 }" "asm"="s_fmamk_f32 {sdst}, {ssrc0}, {lit:h32}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_FMAMK_F32.SOP2.lit.lit_0(i32 "enum"="e1" %sdst, i64 "imm" %lit, i32 "enum"="e3" %ssrc1) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) literal_32:32=lit 101000110 sdst_16:7=sdst ssrc1_8:8=ssrc1 11111111 }" "asm"="s_fmamk_f32 {sdst}, {lit:h32}, {lit:h32}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_FMAMK_F32.SOP2.lit.lit_1(i32 "enum"="e1" %sdst, i32 "enum"="e3" %ssrc0, i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) literal_32:32=lit 101000110 sdst_16:7=sdst 11111111 ssrc0_0:8=ssrc0 }" "asm"="s_fmamk_f32 {sdst}, {ssrc0}, {lit:h32}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_FMAMK_F32.SOP2.lit.lit_0_lit_1(i32 "enum"="e1" %sdst, i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) literal_32:32=lit 101000110 sdst_16:7=sdst 1111111111111111 }" "asm"="s_fmamk_f32 {sdst}, {lit:h32}, {lit:h32}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_FMAC_F32.SOP2(i32 "enum"="e1" %sdst, i32 "enum"="e3" %ssrc0, i32 "enum"="e3" %ssrc1) "enc"="{ if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) 101000111 sdst_16:7=sdst ssrc1_8:8=ssrc1 ssrc0_0:8=ssrc0 }" "asm"="s_fmac_f32 {sdst}, {ssrc0}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_FMAC_F32.SOP2.lit.lit_0(i32 "enum"="e1" %sdst, i64 "imm" %lit, i32 "enum"="e3" %ssrc1) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) literal_32:32=lit 101000111 sdst_16:7=sdst ssrc1_8:8=ssrc1 11111111 }" "asm"="s_fmac_f32 {sdst}, {lit:h32}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_FMAC_F32.SOP2.lit.lit_1(i32 "enum"="e1" %sdst, i32 "enum"="e3" %ssrc0, i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) literal_32:32=lit 101000111 sdst_16:7=sdst 11111111 ssrc0_0:8=ssrc0 }" "asm"="s_fmac_f32 {sdst}, {ssrc0}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_FMAC_F32.SOP2.lit.lit_0_lit_1(i32 "enum"="e1" %sdst, i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) literal_32:32=lit 101000111 sdst_16:7=sdst 1111111111111111 }" "asm"="s_fmac_f32 {sdst}, {lit:h32}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_CVT_PK_RTZ_F16_F32.SOP2(i32 "enum"="e1" %sdst, i32 "enum"="e3" %ssrc0, i32 "enum"="e3" %ssrc1) "enc"="{ if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) 101001000 sdst_16:7=sdst ssrc1_8:8=ssrc1 ssrc0_0:8=ssrc0 }" "asm"="s_cvt_pk_rtz_f16_f32 {sdst}, {ssrc0}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_CVT_PK_RTZ_F16_F32.SOP2.lit.lit_0(i32 "enum"="e1" %sdst, i64 "imm" %lit, i32 "enum"="e3" %ssrc1) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) literal_32:32=lit 101001000 sdst_16:7=sdst ssrc1_8:8=ssrc1 11111111 }" "asm"="s_cvt_pk_rtz_f16_f32 {sdst}, {lit:h32}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_CVT_PK_RTZ_F16_F32.SOP2.lit.lit_1(i32 "enum"="e1" %sdst, i32 "enum"="e3" %ssrc0, i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) literal_32:32=lit 101001000 sdst_16:7=sdst 11111111 ssrc0_0:8=ssrc0 }" "asm"="s_cvt_pk_rtz_f16_f32 {sdst}, {ssrc0}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_CVT_PK_RTZ_F16_F32.SOP2.lit.lit_0_lit_1(i32 "enum"="e1" %sdst, i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) literal_32:32=lit 101001000 sdst_16:7=sdst 1111111111111111 }" "asm"="s_cvt_pk_rtz_f16_f32 {sdst}, {lit:h32}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_ADD_F16.SOP2(i32 "enum"="e1" %sdst, i32 "enum"="e3" %ssrc0, i32 "enum"="e3" %ssrc1) "enc"="{ if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) 101001001 sdst_16:7=sdst ssrc1_8:8=ssrc1 ssrc0_0:8=ssrc0 }" "asm"="s_add_f16 {sdst}, {ssrc0}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_ADD_F16.SOP2.lit.lit_0(i32 "enum"="e1" %sdst, i64 "imm" %lit, i32 "enum"="e3" %ssrc1) "enc"="{ if(lit<65536) if(lit>64) if((lit<65520)|(lit>65535)) if(lit!=12568) if(lit!=14336) if(lit!=15360) if(lit!=16384) if(lit!=17408) if(lit!=47104) if(lit!=48128) if(lit!=49152) if(lit!=50176) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) literal_32:32=lit 101001001 sdst_16:7=sdst ssrc1_8:8=ssrc1 11111111 }" "asm"="s_add_f16 {sdst}, {lit:h32}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_ADD_F16.SOP2.lit.lit_1(i32 "enum"="e1" %sdst, i32 "enum"="e3" %ssrc0, i64 "imm" %lit) "enc"="{ if(lit<65536) if(lit>64) if((lit<65520)|(lit>65535)) if(lit!=12568) if(lit!=14336) if(lit!=15360) if(lit!=16384) if(lit!=17408) if(lit!=47104) if(lit!=48128) if(lit!=49152) if(lit!=50176) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) literal_32:32=lit 101001001 sdst_16:7=sdst 11111111 ssrc0_0:8=ssrc0 }" "asm"="s_add_f16 {sdst}, {ssrc0}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_ADD_F16.SOP2.lit.lit_0_lit_1(i32 "enum"="e1" %sdst, i64 "imm" %lit) "enc"="{ if(lit<65536) if(lit>64) if((lit<65520)|(lit>65535)) if(lit!=12568) if(lit!=14336) if(lit!=15360) if(lit!=16384) if(lit!=17408) if(lit!=47104) if(lit!=48128) if(lit!=49152) if(lit!=50176) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) literal_32:32=lit 101001001 sdst_16:7=sdst 1111111111111111 }" "asm"="s_add_f16 {sdst}, {lit:h32}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_SUB_F16.SOP2(i32 "enum"="e1" %sdst, i32 "enum"="e3" %ssrc0, i32 "enum"="e3" %ssrc1) "enc"="{ if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) 101001010 sdst_16:7=sdst ssrc1_8:8=ssrc1 ssrc0_0:8=ssrc0 }" "asm"="s_sub_f16 {sdst}, {ssrc0}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_SUB_F16.SOP2.lit.lit_0(i32 "enum"="e1" %sdst, i64 "imm" %lit, i32 "enum"="e3" %ssrc1) "enc"="{ if(lit<65536) if(lit>64) if((lit<65520)|(lit>65535)) if(lit!=12568) if(lit!=14336) if(lit!=15360) if(lit!=16384) if(lit!=17408) if(lit!=47104) if(lit!=48128) if(lit!=49152) if(lit!=50176) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) literal_32:32=lit 101001010 sdst_16:7=sdst ssrc1_8:8=ssrc1 11111111 }" "asm"="s_sub_f16 {sdst}, {lit:h32}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_SUB_F16.SOP2.lit.lit_1(i32 "enum"="e1" %sdst, i32 "enum"="e3" %ssrc0, i64 "imm" %lit) "enc"="{ if(lit<65536) if(lit>64) if((lit<65520)|(lit>65535)) if(lit!=12568) if(lit!=14336) if(lit!=15360) if(lit!=16384) if(lit!=17408) if(lit!=47104) if(lit!=48128) if(lit!=49152) if(lit!=50176) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) literal_32:32=lit 101001010 sdst_16:7=sdst 11111111 ssrc0_0:8=ssrc0 }" "asm"="s_sub_f16 {sdst}, {ssrc0}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_SUB_F16.SOP2.lit.lit_0_lit_1(i32 "enum"="e1" %sdst, i64 "imm" %lit) "enc"="{ if(lit<65536) if(lit>64) if((lit<65520)|(lit>65535)) if(lit!=12568) if(lit!=14336) if(lit!=15360) if(lit!=16384) if(lit!=17408) if(lit!=47104) if(lit!=48128) if(lit!=49152) if(lit!=50176) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) literal_32:32=lit 101001010 sdst_16:7=sdst 1111111111111111 }" "asm"="s_sub_f16 {sdst}, {lit:h32}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_MIN_F16.SOP2(i32 "enum"="e1" %sdst, i32 "enum"="e3" %ssrc0, i32 "enum"="e3" %ssrc1) "enc"="{ if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) 101001011 sdst_16:7=sdst ssrc1_8:8=ssrc1 ssrc0_0:8=ssrc0 }" "asm"="s_min_f16 {sdst}, {ssrc0}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_MIN_F16.SOP2.lit.lit_0(i32 "enum"="e1" %sdst, i64 "imm" %lit, i32 "enum"="e3" %ssrc1) "enc"="{ if(lit<65536) if(lit>64) if((lit<65520)|(lit>65535)) if(lit!=12568) if(lit!=14336) if(lit!=15360) if(lit!=16384) if(lit!=17408) if(lit!=47104) if(lit!=48128) if(lit!=49152) if(lit!=50176) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) literal_32:32=lit 101001011 sdst_16:7=sdst ssrc1_8:8=ssrc1 11111111 }" "asm"="s_min_f16 {sdst}, {lit:h32}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_MIN_F16.SOP2.lit.lit_1(i32 "enum"="e1" %sdst, i32 "enum"="e3" %ssrc0, i64 "imm" %lit) "enc"="{ if(lit<65536) if(lit>64) if((lit<65520)|(lit>65535)) if(lit!=12568) if(lit!=14336) if(lit!=15360) if(lit!=16384) if(lit!=17408) if(lit!=47104) if(lit!=48128) if(lit!=49152) if(lit!=50176) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) literal_32:32=lit 101001011 sdst_16:7=sdst 11111111 ssrc0_0:8=ssrc0 }" "asm"="s_min_f16 {sdst}, {ssrc0}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_MIN_F16.SOP2.lit.lit_0_lit_1(i32 "enum"="e1" %sdst, i64 "imm" %lit) "enc"="{ if(lit<65536) if(lit>64) if((lit<65520)|(lit>65535)) if(lit!=12568) if(lit!=14336) if(lit!=15360) if(lit!=16384) if(lit!=17408) if(lit!=47104) if(lit!=48128) if(lit!=49152) if(lit!=50176) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) literal_32:32=lit 101001011 sdst_16:7=sdst 1111111111111111 }" "asm"="s_min_f16 {sdst}, {lit:h32}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_MAX_F16.SOP2(i32 "enum"="e1" %sdst, i32 "enum"="e3" %ssrc0, i32 "enum"="e3" %ssrc1) "enc"="{ if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) 101001100 sdst_16:7=sdst ssrc1_8:8=ssrc1 ssrc0_0:8=ssrc0 }" "asm"="s_max_f16 {sdst}, {ssrc0}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_MAX_F16.SOP2.lit.lit_0(i32 "enum"="e1" %sdst, i64 "imm" %lit, i32 "enum"="e3" %ssrc1) "enc"="{ if(lit<65536) if(lit>64) if((lit<65520)|(lit>65535)) if(lit!=12568) if(lit!=14336) if(lit!=15360) if(lit!=16384) if(lit!=17408) if(lit!=47104) if(lit!=48128) if(lit!=49152) if(lit!=50176) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) literal_32:32=lit 101001100 sdst_16:7=sdst ssrc1_8:8=ssrc1 11111111 }" "asm"="s_max_f16 {sdst}, {lit:h32}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_MAX_F16.SOP2.lit.lit_1(i32 "enum"="e1" %sdst, i32 "enum"="e3" %ssrc0, i64 "imm" %lit) "enc"="{ if(lit<65536) if(lit>64) if((lit<65520)|(lit>65535)) if(lit!=12568) if(lit!=14336) if(lit!=15360) if(lit!=16384) if(lit!=17408) if(lit!=47104) if(lit!=48128) if(lit!=49152) if(lit!=50176) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) literal_32:32=lit 101001100 sdst_16:7=sdst 11111111 ssrc0_0:8=ssrc0 }" "asm"="s_max_f16 {sdst}, {ssrc0}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_MAX_F16.SOP2.lit.lit_0_lit_1(i32 "enum"="e1" %sdst, i64 "imm" %lit) "enc"="{ if(lit<65536) if(lit>64) if((lit<65520)|(lit>65535)) if(lit!=12568) if(lit!=14336) if(lit!=15360) if(lit!=16384) if(lit!=17408) if(lit!=47104) if(lit!=48128) if(lit!=49152) if(lit!=50176) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) literal_32:32=lit 101001100 sdst_16:7=sdst 1111111111111111 }" "asm"="s_max_f16 {sdst}, {lit:h32}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_MUL_F16.SOP2(i32 "enum"="e1" %sdst, i32 "enum"="e3" %ssrc0, i32 "enum"="e3" %ssrc1) "enc"="{ if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) 101001101 sdst_16:7=sdst ssrc1_8:8=ssrc1 ssrc0_0:8=ssrc0 }" "asm"="s_mul_f16 {sdst}, {ssrc0}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_MUL_F16.SOP2.lit.lit_0(i32 "enum"="e1" %sdst, i64 "imm" %lit, i32 "enum"="e3" %ssrc1) "enc"="{ if(lit<65536) if(lit>64) if((lit<65520)|(lit>65535)) if(lit!=12568) if(lit!=14336) if(lit!=15360) if(lit!=16384) if(lit!=17408) if(lit!=47104) if(lit!=48128) if(lit!=49152) if(lit!=50176) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) literal_32:32=lit 101001101 sdst_16:7=sdst ssrc1_8:8=ssrc1 11111111 }" "asm"="s_mul_f16 {sdst}, {lit:h32}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_MUL_F16.SOP2.lit.lit_1(i32 "enum"="e1" %sdst, i32 "enum"="e3" %ssrc0, i64 "imm" %lit) "enc"="{ if(lit<65536) if(lit>64) if((lit<65520)|(lit>65535)) if(lit!=12568) if(lit!=14336) if(lit!=15360) if(lit!=16384) if(lit!=17408) if(lit!=47104) if(lit!=48128) if(lit!=49152) if(lit!=50176) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) literal_32:32=lit 101001101 sdst_16:7=sdst 11111111 ssrc0_0:8=ssrc0 }" "asm"="s_mul_f16 {sdst}, {ssrc0}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_MUL_F16.SOP2.lit.lit_0_lit_1(i32 "enum"="e1" %sdst, i64 "imm" %lit) "enc"="{ if(lit<65536) if(lit>64) if((lit<65520)|(lit>65535)) if(lit!=12568) if(lit!=14336) if(lit!=15360) if(lit!=16384) if(lit!=17408) if(lit!=47104) if(lit!=48128) if(lit!=49152) if(lit!=50176) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) literal_32:32=lit 101001101 sdst_16:7=sdst 1111111111111111 }" "asm"="s_mul_f16 {sdst}, {lit:h32}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_FMAC_F16.SOP2(i32 "enum"="e1" %sdst, i32 "enum"="e3" %ssrc0, i32 "enum"="e3" %ssrc1) "enc"="{ if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) 101001110 sdst_16:7=sdst ssrc1_8:8=ssrc1 ssrc0_0:8=ssrc0 }" "asm"="s_fmac_f16 {sdst}, {ssrc0}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_FMAC_F16.SOP2.lit.lit_0(i32 "enum"="e1" %sdst, i64 "imm" %lit, i32 "enum"="e3" %ssrc1) "enc"="{ if(lit<65536) if(lit>64) if((lit<65520)|(lit>65535)) if(lit!=12568) if(lit!=14336) if(lit!=15360) if(lit!=16384) if(lit!=17408) if(lit!=47104) if(lit!=48128) if(lit!=49152) if(lit!=50176) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) literal_32:32=lit 101001110 sdst_16:7=sdst ssrc1_8:8=ssrc1 11111111 }" "asm"="s_fmac_f16 {sdst}, {lit:h32}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_FMAC_F16.SOP2.lit.lit_1(i32 "enum"="e1" %sdst, i32 "enum"="e3" %ssrc0, i64 "imm" %lit) "enc"="{ if(lit<65536) if(lit>64) if((lit<65520)|(lit>65535)) if(lit!=12568) if(lit!=14336) if(lit!=15360) if(lit!=16384) if(lit!=17408) if(lit!=47104) if(lit!=48128) if(lit!=49152) if(lit!=50176) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) literal_32:32=lit 101001110 sdst_16:7=sdst 11111111 ssrc0_0:8=ssrc0 }" "asm"="s_fmac_f16 {sdst}, {ssrc0}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_FMAC_F16.SOP2.lit.lit_0_lit_1(i32 "enum"="e1" %sdst, i64 "imm" %lit) "enc"="{ if(lit<65536) if(lit>64) if((lit<65520)|(lit>65535)) if(lit!=12568) if(lit!=14336) if(lit!=15360) if(lit!=16384) if(lit!=17408) if(lit!=47104) if(lit!=48128) if(lit!=49152) if(lit!=50176) if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) literal_32:32=lit 101001110 sdst_16:7=sdst 1111111111111111 }" "asm"="s_fmac_f16 {sdst}, {lit:h32}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_CMP_EQ_I32.SOPC(i32 "enum"="e3" %ssrc0, i32 "enum"="e3" %ssrc1) "enc"="{ if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) 1011111100000000 ssrc1_8:8=ssrc1 ssrc0_0:8=ssrc0 }" "asm"="s_cmp_eq_i32 {ssrc0}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_CMP_EQ_I32.SOPC.lit.lit_0(i64 "imm" %lit, i32 "enum"="e3" %ssrc1) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=65535) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) literal_32:32=lit 1011111100000000 ssrc1_8:8=ssrc1 11111111 }" "asm"="s_cmp_eq_i32 {lit:h32}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_CMP_EQ_I32.SOPC.lit.lit_1(i32 "enum"="e3" %ssrc0, i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) literal_32:32=lit 101111110000000011111111 ssrc0_0:8=ssrc0 }" "asm"="s_cmp_eq_i32 {ssrc0}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_CMP_EQ_I32.SOPC.lit.lit_0_lit_1(i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) literal_32:32=lit 10111111000000001111111111111111 }" "asm"="s_cmp_eq_i32 {lit:h32}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_CMP_LG_I32.SOPC(i32 "enum"="e3" %ssrc0, i32 "enum"="e3" %ssrc1) "enc"="{ if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) 1011111100000001 ssrc1_8:8=ssrc1 ssrc0_0:8=ssrc0 }" "asm"="s_cmp_lg_i32 {ssrc0}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_CMP_LG_I32.SOPC.lit.lit_0(i64 "imm" %lit, i32 "enum"="e3" %ssrc1) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) literal_32:32=lit 1011111100000001 ssrc1_8:8=ssrc1 11111111 }" "asm"="s_cmp_lg_i32 {lit:h32}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_CMP_LG_I32.SOPC.lit.lit_1(i32 "enum"="e3" %ssrc0, i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) literal_32:32=lit 101111110000000111111111 ssrc0_0:8=ssrc0 }" "asm"="s_cmp_lg_i32 {ssrc0}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_CMP_LG_I32.SOPC.lit.lit_0_lit_1(i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) literal_32:32=lit 10111111000000011111111111111111 }" "asm"="s_cmp_lg_i32 {lit:h32}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_CMP_GT_I32.SOPC(i32 "enum"="e3" %ssrc0, i32 "enum"="e3" %ssrc1) "enc"="{ if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) 1011111100000010 ssrc1_8:8=ssrc1 ssrc0_0:8=ssrc0 }" "asm"="s_cmp_gt_i32 {ssrc0}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_CMP_GT_I32.SOPC.lit.lit_0(i64 "imm" %lit, i32 "enum"="e3" %ssrc1) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) literal_32:32=lit 1011111100000010 ssrc1_8:8=ssrc1 11111111 }" "asm"="s_cmp_gt_i32 {lit:h32}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_CMP_GT_I32.SOPC.lit.lit_1(i32 "enum"="e3" %ssrc0, i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) literal_32:32=lit 101111110000001011111111 ssrc0_0:8=ssrc0 }" "asm"="s_cmp_gt_i32 {ssrc0}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_CMP_GT_I32.SOPC.lit.lit_0_lit_1(i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) literal_32:32=lit 10111111000000101111111111111111 }" "asm"="s_cmp_gt_i32 {lit:h32}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_CMP_GE_I32.SOPC(i32 "enum"="e3" %ssrc0, i32 "enum"="e3" %ssrc1) "enc"="{ if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) 1011111100000011 ssrc1_8:8=ssrc1 ssrc0_0:8=ssrc0 }" "asm"="s_cmp_ge_i32 {ssrc0}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_CMP_GE_I32.SOPC.lit.lit_0(i64 "imm" %lit, i32 "enum"="e3" %ssrc1) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) literal_32:32=lit 1011111100000011 ssrc1_8:8=ssrc1 11111111 }" "asm"="s_cmp_ge_i32 {lit:h32}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_CMP_GE_I32.SOPC.lit.lit_1(i32 "enum"="e3" %ssrc0, i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) literal_32:32=lit 101111110000001111111111 ssrc0_0:8=ssrc0 }" "asm"="s_cmp_ge_i32 {ssrc0}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_CMP_GE_I32.SOPC.lit.lit_0_lit_1(i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) literal_32:32=lit 10111111000000111111111111111111 }" "asm"="s_cmp_ge_i32 {lit:h32}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_CMP_LT_I32.SOPC(i32 "enum"="e3" %ssrc0, i32 "enum"="e3" %ssrc1) "enc"="{ if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) 1011111100000100 ssrc1_8:8=ssrc1 ssrc0_0:8=ssrc0 }" "asm"="s_cmp_lt_i32 {ssrc0}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_CMP_LT_I32.SOPC.lit.lit_0(i64 "imm" %lit, i32 "enum"="e3" %ssrc1) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) literal_32:32=lit 1011111100000100 ssrc1_8:8=ssrc1 11111111 }" "asm"="s_cmp_lt_i32 {lit:h32}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_CMP_LT_I32.SOPC.lit.lit_1(i32 "enum"="e3" %ssrc0, i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) literal_32:32=lit 101111110000010011111111 ssrc0_0:8=ssrc0 }" "asm"="s_cmp_lt_i32 {ssrc0}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_CMP_LT_I32.SOPC.lit.lit_0_lit_1(i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) literal_32:32=lit 10111111000001001111111111111111 }" "asm"="s_cmp_lt_i32 {lit:h32}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_CMP_LE_I32.SOPC(i32 "enum"="e3" %ssrc0, i32 "enum"="e3" %ssrc1) "enc"="{ if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) 1011111100000101 ssrc1_8:8=ssrc1 ssrc0_0:8=ssrc0 }" "asm"="s_cmp_le_i32 {ssrc0}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_CMP_LE_I32.SOPC.lit.lit_0(i64 "imm" %lit, i32 "enum"="e3" %ssrc1) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) literal_32:32=lit 1011111100000101 ssrc1_8:8=ssrc1 11111111 }" "asm"="s_cmp_le_i32 {lit:h32}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_CMP_LE_I32.SOPC.lit.lit_1(i32 "enum"="e3" %ssrc0, i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) literal_32:32=lit 101111110000010111111111 ssrc0_0:8=ssrc0 }" "asm"="s_cmp_le_i32 {ssrc0}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_CMP_LE_I32.SOPC.lit.lit_0_lit_1(i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) literal_32:32=lit 10111111000001011111111111111111 }" "asm"="s_cmp_le_i32 {lit:h32}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_CMP_EQ_U32.SOPC(i32 "enum"="e3" %ssrc0, i32 "enum"="e3" %ssrc1) "enc"="{ if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) 1011111100000110 ssrc1_8:8=ssrc1 ssrc0_0:8=ssrc0 }" "asm"="s_cmp_eq_u32 {ssrc0}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_CMP_EQ_U32.SOPC.lit.lit_0(i64 "imm" %lit, i32 "enum"="e3" %ssrc1) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) literal_32:32=lit 1011111100000110 ssrc1_8:8=ssrc1 11111111 }" "asm"="s_cmp_eq_u32 {lit:h32}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_CMP_EQ_U32.SOPC.lit.lit_1(i32 "enum"="e3" %ssrc0, i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) literal_32:32=lit 101111110000011011111111 ssrc0_0:8=ssrc0 }" "asm"="s_cmp_eq_u32 {ssrc0}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_CMP_EQ_U32.SOPC.lit.lit_0_lit_1(i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) literal_32:32=lit 10111111000001101111111111111111 }" "asm"="s_cmp_eq_u32 {lit:h32}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_CMP_LG_U32.SOPC(i32 "enum"="e3" %ssrc0, i32 "enum"="e3" %ssrc1) "enc"="{ if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) 1011111100000111 ssrc1_8:8=ssrc1 ssrc0_0:8=ssrc0 }" "asm"="s_cmp_lg_u32 {ssrc0}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_CMP_LG_U32.SOPC.lit.lit_0(i64 "imm" %lit, i32 "enum"="e3" %ssrc1) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) literal_32:32=lit 1011111100000111 ssrc1_8:8=ssrc1 11111111 }" "asm"="s_cmp_lg_u32 {lit:h32}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_CMP_LG_U32.SOPC.lit.lit_1(i32 "enum"="e3" %ssrc0, i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) literal_32:32=lit 101111110000011111111111 ssrc0_0:8=ssrc0 }" "asm"="s_cmp_lg_u32 {ssrc0}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_CMP_LG_U32.SOPC.lit.lit_0_lit_1(i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) literal_32:32=lit 10111111000001111111111111111111 }" "asm"="s_cmp_lg_u32 {lit:h32}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_CMP_GT_U32.SOPC(i32 "enum"="e3" %ssrc0, i32 "enum"="e3" %ssrc1) "enc"="{ if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) 1011111100001000 ssrc1_8:8=ssrc1 ssrc0_0:8=ssrc0 }" "asm"="s_cmp_gt_u32 {ssrc0}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_CMP_GT_U32.SOPC.lit.lit_0(i64 "imm" %lit, i32 "enum"="e3" %ssrc1) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) literal_32:32=lit 1011111100001000 ssrc1_8:8=ssrc1 11111111 }" "asm"="s_cmp_gt_u32 {lit:h32}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_CMP_GT_U32.SOPC.lit.lit_1(i32 "enum"="e3" %ssrc0, i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) literal_32:32=lit 101111110000100011111111 ssrc0_0:8=ssrc0 }" "asm"="s_cmp_gt_u32 {ssrc0}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_CMP_GT_U32.SOPC.lit.lit_0_lit_1(i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) literal_32:32=lit 10111111000010001111111111111111 }" "asm"="s_cmp_gt_u32 {lit:h32}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_CMP_GE_U32.SOPC(i32 "enum"="e3" %ssrc0, i32 "enum"="e3" %ssrc1) "enc"="{ if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) 1011111100001001 ssrc1_8:8=ssrc1 ssrc0_0:8=ssrc0 }" "asm"="s_cmp_ge_u32 {ssrc0}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_CMP_GE_U32.SOPC.lit.lit_0(i64 "imm" %lit, i32 "enum"="e3" %ssrc1) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) literal_32:32=lit 1011111100001001 ssrc1_8:8=ssrc1 11111111 }" "asm"="s_cmp_ge_u32 {lit:h32}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_CMP_GE_U32.SOPC.lit.lit_1(i32 "enum"="e3" %ssrc0, i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) literal_32:32=lit 101111110000100111111111 ssrc0_0:8=ssrc0 }" "asm"="s_cmp_ge_u32 {ssrc0}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_CMP_GE_U32.SOPC.lit.lit_0_lit_1(i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) literal_32:32=lit 10111111000010011111111111111111 }" "asm"="s_cmp_ge_u32 {lit:h32}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_CMP_LT_U32.SOPC(i32 "enum"="e3" %ssrc0, i32 "enum"="e3" %ssrc1) "enc"="{ if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) 1011111100001010 ssrc1_8:8=ssrc1 ssrc0_0:8=ssrc0 }" "asm"="s_cmp_lt_u32 {ssrc0}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_CMP_LT_U32.SOPC.lit.lit_0(i64 "imm" %lit, i32 "enum"="e3" %ssrc1) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) literal_32:32=lit 1011111100001010 ssrc1_8:8=ssrc1 11111111 }" "asm"="s_cmp_lt_u32 {lit:h32}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_CMP_LT_U32.SOPC.lit.lit_1(i32 "enum"="e3" %ssrc0, i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) literal_32:32=lit 101111110000101011111111 ssrc0_0:8=ssrc0 }" "asm"="s_cmp_lt_u32 {ssrc0}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_CMP_LT_U32.SOPC.lit.lit_0_lit_1(i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) literal_32:32=lit 10111111000010101111111111111111 }" "asm"="s_cmp_lt_u32 {lit:h32}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_CMP_LE_U32.SOPC(i32 "enum"="e3" %ssrc0, i32 "enum"="e3" %ssrc1) "enc"="{ if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) 1011111100001011 ssrc1_8:8=ssrc1 ssrc0_0:8=ssrc0 }" "asm"="s_cmp_le_u32 {ssrc0}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_CMP_LE_U32.SOPC.lit.lit_0(i64 "imm" %lit, i32 "enum"="e3" %ssrc1) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) literal_32:32=lit 1011111100001011 ssrc1_8:8=ssrc1 11111111 }" "asm"="s_cmp_le_u32 {lit:h32}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_CMP_LE_U32.SOPC.lit.lit_1(i32 "enum"="e3" %ssrc0, i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) literal_32:32=lit 101111110000101111111111 ssrc0_0:8=ssrc0 }" "asm"="s_cmp_le_u32 {ssrc0}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_CMP_LE_U32.SOPC.lit.lit_0_lit_1(i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) literal_32:32=lit 10111111000010111111111111111111 }" "asm"="s_cmp_le_u32 {lit:h32}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_BITCMP0_B32.SOPC(i32 "enum"="e3" %ssrc0, i32 "enum"="e3" %ssrc1) "enc"="{ if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) 1011111100001100 ssrc1_8:8=ssrc1 ssrc0_0:8=ssrc0 }" "asm"="s_bitcmp0_b32 {ssrc0}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_BITCMP0_B32.SOPC.lit.lit_0(i64 "imm" %lit, i32 "enum"="e3" %ssrc1) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) literal_32:32=lit 1011111100001100 ssrc1_8:8=ssrc1 11111111 }" "asm"="s_bitcmp0_b32 {lit:h32}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_BITCMP0_B32.SOPC.lit.lit_1(i32 "enum"="e3" %ssrc0, i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) literal_32:32=lit 101111110000110011111111 ssrc0_0:8=ssrc0 }" "asm"="s_bitcmp0_b32 {ssrc0}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_BITCMP0_B32.SOPC.lit.lit_0_lit_1(i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) literal_32:32=lit 10111111000011001111111111111111 }" "asm"="s_bitcmp0_b32 {lit:h32}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_BITCMP1_B32.SOPC(i32 "enum"="e3" %ssrc0, i32 "enum"="e3" %ssrc1) "enc"="{ if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) 1011111100001101 ssrc1_8:8=ssrc1 ssrc0_0:8=ssrc0 }" "asm"="s_bitcmp1_b32 {ssrc0}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_BITCMP1_B32.SOPC.lit.lit_0(i64 "imm" %lit, i32 "enum"="e3" %ssrc1) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) literal_32:32=lit 1011111100001101 ssrc1_8:8=ssrc1 11111111 }" "asm"="s_bitcmp1_b32 {lit:h32}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_BITCMP1_B32.SOPC.lit.lit_1(i32 "enum"="e3" %ssrc0, i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) literal_32:32=lit 101111110000110111111111 ssrc0_0:8=ssrc0 }" "asm"="s_bitcmp1_b32 {ssrc0}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_BITCMP1_B32.SOPC.lit.lit_0_lit_1(i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) literal_32:32=lit 10111111000011011111111111111111 }" "asm"="s_bitcmp1_b32 {lit:h32}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_BITCMP0_B64.SOPC(i32 "enum"="e18" %ssrc0, i32 "enum"="e3" %ssrc1) "enc"="{ if((ssrc0<256)|(ssrc0>510)) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) 1011111100001110 ssrc1_8:8=ssrc1 ssrc0_0:8=ssrc0 }" "asm"="s_bitcmp0_b64 {ssrc0}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_BITCMP0_B64.SOPC.lit.lit_0(i64 "imm" %lit, i32 "enum"="e3" %ssrc1) "enc"="{ if(lit>64) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) literal_32:32=lit 1011111100001110 ssrc1_8:8=ssrc1 11111111 }" "asm"="s_bitcmp0_b64 {lit:h32}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_BITCMP0_B64.SOPC.lit.lit_1(i32 "enum"="e18" %ssrc0, i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((ssrc0<256)|(ssrc0>510)) literal_32:32=lit 101111110000111011111111 ssrc0_0:8=ssrc0 }" "asm"="s_bitcmp0_b64 {ssrc0}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_BITCMP0_B64.SOPC.lit.lit_0_lit_1(i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) literal_32:32=lit 10111111000011101111111111111111 }" "asm"="s_bitcmp0_b64 {lit:h32}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_BITCMP1_B64.SOPC(i32 "enum"="e18" %ssrc0, i32 "enum"="e3" %ssrc1) "enc"="{ if((ssrc0<256)|(ssrc0>510)) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) 1011111100001111 ssrc1_8:8=ssrc1 ssrc0_0:8=ssrc0 }" "asm"="s_bitcmp1_b64 {ssrc0}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_BITCMP1_B64.SOPC.lit.lit_0(i64 "imm" %lit, i32 "enum"="e3" %ssrc1) "enc"="{ if(lit>64) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) literal_32:32=lit 1011111100001111 ssrc1_8:8=ssrc1 11111111 }" "asm"="s_bitcmp1_b64 {lit:h32}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_BITCMP1_B64.SOPC.lit.lit_1(i32 "enum"="e18" %ssrc0, i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((ssrc0<256)|(ssrc0>510)) literal_32:32=lit 101111110000111111111111 ssrc0_0:8=ssrc0 }" "asm"="s_bitcmp1_b64 {ssrc0}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_BITCMP1_B64.SOPC.lit.lit_0_lit_1(i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) literal_32:32=lit 10111111000011111111111111111111 }" "asm"="s_bitcmp1_b64 {lit:h32}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_CMP_EQ_U64.SOPC(i32 "enum"="e18" %ssrc0, i32 "enum"="e18" %ssrc1) "enc"="{ if((ssrc0<256)|(ssrc0>510)) if((ssrc1<256)|(ssrc1>510)) 1011111100010000 ssrc1_8:8=ssrc1 ssrc0_0:8=ssrc0 }" "asm"="s_cmp_eq_u64 {ssrc0}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_CMP_EQ_U64.SOPC.lit.lit_0(i64 "imm" %lit, i32 "enum"="e18" %ssrc1) "enc"="{ if(lit>64) if((ssrc1<256)|(ssrc1>510)) literal_32:32=lit 1011111100010000 ssrc1_8:8=ssrc1 11111111 }" "asm"="s_cmp_eq_u64 {lit:h32}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_CMP_EQ_U64.SOPC.lit.lit_1(i32 "enum"="e18" %ssrc0, i64 "imm" %lit) "enc"="{ if(lit>64) if((ssrc0<256)|(ssrc0>510)) literal_32:32=lit 101111110001000011111111 ssrc0_0:8=ssrc0 }" "asm"="s_cmp_eq_u64 {ssrc0}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_CMP_EQ_U64.SOPC.lit.lit_0_lit_1(i64 "imm" %lit) "enc"="{ if(lit>64) literal_32:32=lit 10111111000100001111111111111111 }" "asm"="s_cmp_eq_u64 {lit:h32}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_CMP_LG_U64.SOPC(i32 "enum"="e18" %ssrc0, i32 "enum"="e18" %ssrc1) "enc"="{ if(ssrc0!=68) if((ssrc0<256)|(ssrc0>510)) if((ssrc1<256)|(ssrc1>510)) 1011111100010001 ssrc1_8:8=ssrc1 ssrc0_0:8=ssrc0 }" "asm"="s_cmp_lg_u64 {ssrc0}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_CMP_LG_U64.SOPC.lit.lit_0(i64 "imm" %lit, i32 "enum"="e18" %ssrc1) "enc"="{ if(lit>64) if((ssrc1<256)|(ssrc1>510)) literal_32:32=lit 1011111100010001 ssrc1_8:8=ssrc1 11111111 }" "asm"="s_cmp_lg_u64 {lit:h32}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_CMP_LG_U64.SOPC.lit.lit_1(i32 "enum"="e18" %ssrc0, i64 "imm" %lit) "enc"="{ if(lit>64) if((ssrc0<256)|(ssrc0>510)) literal_32:32=lit 101111110001000111111111 ssrc0_0:8=ssrc0 }" "asm"="s_cmp_lg_u64 {ssrc0}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_CMP_LG_U64.SOPC.lit.lit_0_lit_1(i64 "imm" %lit) "enc"="{ if(lit>64) literal_32:32=lit 10111111000100011111111111111111 }" "asm"="s_cmp_lg_u64 {lit:h32}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_CMP_LT_F32.SOPC(i32 "enum"="e3" %ssrc0, i32 "enum"="e3" %ssrc1) "enc"="{ if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) 1011111101000001 ssrc1_8:8=ssrc1 ssrc0_0:8=ssrc0 }" "asm"="s_cmp_lt_f32 {ssrc0}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_CMP_LT_F32.SOPC.lit.lit_0(i64 "imm" %lit, i32 "enum"="e3" %ssrc1) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) literal_32:32=lit 1011111101000001 ssrc1_8:8=ssrc1 11111111 }" "asm"="s_cmp_lt_f32 {lit:h32}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_CMP_LT_F32.SOPC.lit.lit_1(i32 "enum"="e3" %ssrc0, i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) literal_32:32=lit 101111110100000111111111 ssrc0_0:8=ssrc0 }" "asm"="s_cmp_lt_f32 {ssrc0}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_CMP_LT_F32.SOPC.lit.lit_0_lit_1(i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) literal_32:32=lit 10111111010000011111111111111111 }" "asm"="s_cmp_lt_f32 {lit:h32}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_CMP_LT_F16.SOPC(i32 "enum"="e3" %ssrc0, i32 "enum"="e3" %ssrc1) "enc"="{ if((ssrc0<240)|(ssrc0>248)) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) if((ssrc1<240)|(ssrc1>248)) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) 1011111101010001 ssrc1_8:8=ssrc1 ssrc0_0:8=ssrc0 }" "asm"="s_cmp_lt_f16 {ssrc0}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_CMP_LT_F16.SOPC.lit.lit_0(i64 "imm" %lit, i32 "enum"="e3" %ssrc1) "enc"="{ if(lit<65536) if(lit>64) if((ssrc1<240)|(ssrc1>248)) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) literal_32:32=lit 1011111101010001 ssrc1_8:8=ssrc1 11111111 }" "asm"="s_cmp_lt_f16 {lit:h32}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_CMP_LT_F16.SOPC.lit.lit_1(i32 "enum"="e3" %ssrc0, i64 "imm" %lit) "enc"="{ if(lit<65536) if(lit>64) if((ssrc0<240)|(ssrc0>248)) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) literal_32:32=lit 101111110101000111111111 ssrc0_0:8=ssrc0 }" "asm"="s_cmp_lt_f16 {ssrc0}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_CMP_LT_F16.SOPC.lit.lit_0_lit_1(i64 "imm" %lit) "enc"="{ if(lit<65536) if(lit>64) literal_32:32=lit 10111111010100011111111111111111 }" "asm"="s_cmp_lt_f16 {lit:h32}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_CMP_EQ_F32.SOPC(i32 "enum"="e3" %ssrc0, i32 "enum"="e3" %ssrc1) "enc"="{ if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) 1011111101000010 ssrc1_8:8=ssrc1 ssrc0_0:8=ssrc0 }" "asm"="s_cmp_eq_f32 {ssrc0}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_CMP_EQ_F32.SOPC.lit.lit_0(i64 "imm" %lit, i32 "enum"="e3" %ssrc1) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) literal_32:32=lit 1011111101000010 ssrc1_8:8=ssrc1 11111111 }" "asm"="s_cmp_eq_f32 {lit:h32}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_CMP_EQ_F32.SOPC.lit.lit_1(i32 "enum"="e3" %ssrc0, i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) literal_32:32=lit 101111110100001011111111 ssrc0_0:8=ssrc0 }" "asm"="s_cmp_eq_f32 {ssrc0}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_CMP_EQ_F32.SOPC.lit.lit_0_lit_1(i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) literal_32:32=lit 10111111010000101111111111111111 }" "asm"="s_cmp_eq_f32 {lit:h32}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_CMP_EQ_F16.SOPC(i32 "enum"="e3" %ssrc0, i32 "enum"="e3" %ssrc1) "enc"="{ if((ssrc0<240)|(ssrc0>248)) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) if((ssrc1<240)|(ssrc1>248)) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) 1011111101010010 ssrc1_8:8=ssrc1 ssrc0_0:8=ssrc0 }" "asm"="s_cmp_eq_f16 {ssrc0}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_CMP_EQ_F16.SOPC.lit.lit_0(i64 "imm" %lit, i32 "enum"="e3" %ssrc1) "enc"="{ if(lit<65536) if(lit>64) if((ssrc1<240)|(ssrc1>248)) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) literal_32:32=lit 1011111101010010 ssrc1_8:8=ssrc1 11111111 }" "asm"="s_cmp_eq_f16 {lit:h32}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_CMP_EQ_F16.SOPC.lit.lit_1(i32 "enum"="e3" %ssrc0, i64 "imm" %lit) "enc"="{ if(lit<65536) if(lit>64) if((ssrc0<240)|(ssrc0>248)) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) literal_32:32=lit 101111110101001011111111 ssrc0_0:8=ssrc0 }" "asm"="s_cmp_eq_f16 {ssrc0}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_CMP_EQ_F16.SOPC.lit.lit_0_lit_1(i64 "imm" %lit) "enc"="{ if(lit<65536) if(lit>64) literal_32:32=lit 10111111010100101111111111111111 }" "asm"="s_cmp_eq_f16 {lit:h32}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_CMP_LE_F32.SOPC(i32 "enum"="e3" %ssrc0, i32 "enum"="e3" %ssrc1) "enc"="{ if(ssrc0!=11) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) 1011111101000011 ssrc1_8:8=ssrc1 ssrc0_0:8=ssrc0 }" "asm"="s_cmp_le_f32 {ssrc0}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_CMP_LE_F32.SOPC.lit.lit_0(i64 "imm" %lit, i32 "enum"="e3" %ssrc1) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) literal_32:32=lit 1011111101000011 ssrc1_8:8=ssrc1 11111111 }" "asm"="s_cmp_le_f32 {lit:h32}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_CMP_LE_F32.SOPC.lit.lit_1(i32 "enum"="e3" %ssrc0, i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) literal_32:32=lit 101111110100001111111111 ssrc0_0:8=ssrc0 }" "asm"="s_cmp_le_f32 {ssrc0}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_CMP_LE_F32.SOPC.lit.lit_0_lit_1(i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) literal_32:32=lit 10111111010000111111111111111111 }" "asm"="s_cmp_le_f32 {lit:h32}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_CMP_LE_F16.SOPC(i32 "enum"="e3" %ssrc0, i32 "enum"="e3" %ssrc1) "enc"="{ if((ssrc0<240)|(ssrc0>248)) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) if((ssrc1<240)|(ssrc1>248)) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) 1011111101010011 ssrc1_8:8=ssrc1 ssrc0_0:8=ssrc0 }" "asm"="s_cmp_le_f16 {ssrc0}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_CMP_LE_F16.SOPC.lit.lit_0(i64 "imm" %lit, i32 "enum"="e3" %ssrc1) "enc"="{ if(lit<65536) if(lit>64) if((ssrc1<240)|(ssrc1>248)) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) literal_32:32=lit 1011111101010011 ssrc1_8:8=ssrc1 11111111 }" "asm"="s_cmp_le_f16 {lit:h32}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_CMP_LE_F16.SOPC.lit.lit_1(i32 "enum"="e3" %ssrc0, i64 "imm" %lit) "enc"="{ if(lit<65536) if(lit>64) if((ssrc0<240)|(ssrc0>248)) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) literal_32:32=lit 101111110101001111111111 ssrc0_0:8=ssrc0 }" "asm"="s_cmp_le_f16 {ssrc0}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_CMP_LE_F16.SOPC.lit.lit_0_lit_1(i64 "imm" %lit) "enc"="{ if(lit<65536) if(lit>64) literal_32:32=lit 10111111010100111111111111111111 }" "asm"="s_cmp_le_f16 {lit:h32}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_CMP_GT_F32.SOPC(i32 "enum"="e3" %ssrc0, i32 "enum"="e3" %ssrc1) "enc"="{ if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) 1011111101000100 ssrc1_8:8=ssrc1 ssrc0_0:8=ssrc0 }" "asm"="s_cmp_gt_f32 {ssrc0}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_CMP_GT_F32.SOPC.lit.lit_0(i64 "imm" %lit, i32 "enum"="e3" %ssrc1) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) literal_32:32=lit 1011111101000100 ssrc1_8:8=ssrc1 11111111 }" "asm"="s_cmp_gt_f32 {lit:h32}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_CMP_GT_F32.SOPC.lit.lit_1(i32 "enum"="e3" %ssrc0, i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) literal_32:32=lit 101111110100010011111111 ssrc0_0:8=ssrc0 }" "asm"="s_cmp_gt_f32 {ssrc0}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_CMP_GT_F32.SOPC.lit.lit_0_lit_1(i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) literal_32:32=lit 10111111010001001111111111111111 }" "asm"="s_cmp_gt_f32 {lit:h32}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_CMP_GT_F16.SOPC(i32 "enum"="e3" %ssrc0, i32 "enum"="e3" %ssrc1) "enc"="{ if(ssrc0!=41) if((ssrc0<240)|(ssrc0>248)) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) if((ssrc1<240)|(ssrc1>248)) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) 1011111101010100 ssrc1_8:8=ssrc1 ssrc0_0:8=ssrc0 }" "asm"="s_cmp_gt_f16 {ssrc0}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_CMP_GT_F16.SOPC.lit.lit_0(i64 "imm" %lit, i32 "enum"="e3" %ssrc1) "enc"="{ if(lit<65536) if(lit>64) if((ssrc1<240)|(ssrc1>248)) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) literal_32:32=lit 1011111101010100 ssrc1_8:8=ssrc1 11111111 }" "asm"="s_cmp_gt_f16 {lit:h32}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_CMP_GT_F16.SOPC.lit.lit_1(i32 "enum"="e3" %ssrc0, i64 "imm" %lit) "enc"="{ if(lit<65536) if(lit>64) if((ssrc0<240)|(ssrc0>248)) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) literal_32:32=lit 101111110101010011111111 ssrc0_0:8=ssrc0 }" "asm"="s_cmp_gt_f16 {ssrc0}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_CMP_GT_F16.SOPC.lit.lit_0_lit_1(i64 "imm" %lit) "enc"="{ if(lit<65536) if(lit>64) literal_32:32=lit 10111111010101001111111111111111 }" "asm"="s_cmp_gt_f16 {lit:h32}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_CMP_LG_F32.SOPC(i32 "enum"="e3" %ssrc0, i32 "enum"="e3" %ssrc1) "enc"="{ if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) 1011111101000101 ssrc1_8:8=ssrc1 ssrc0_0:8=ssrc0 }" "asm"="s_cmp_lg_f32 {ssrc0}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_CMP_LG_F32.SOPC.lit.lit_0(i64 "imm" %lit, i32 "enum"="e3" %ssrc1) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) literal_32:32=lit 1011111101000101 ssrc1_8:8=ssrc1 11111111 }" "asm"="s_cmp_lg_f32 {lit:h32}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_CMP_LG_F32.SOPC.lit.lit_1(i32 "enum"="e3" %ssrc0, i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) literal_32:32=lit 101111110100010111111111 ssrc0_0:8=ssrc0 }" "asm"="s_cmp_lg_f32 {ssrc0}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_CMP_LG_F32.SOPC.lit.lit_0_lit_1(i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) literal_32:32=lit 10111111010001011111111111111111 }" "asm"="s_cmp_lg_f32 {lit:h32}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_CMP_LG_F16.SOPC(i32 "enum"="e3" %ssrc0, i32 "enum"="e3" %ssrc1) "enc"="{ if((ssrc0<240)|(ssrc0>248)) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) if((ssrc1<240)|(ssrc1>248)) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) 1011111101010101 ssrc1_8:8=ssrc1 ssrc0_0:8=ssrc0 }" "asm"="s_cmp_lg_f16 {ssrc0}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_CMP_LG_F16.SOPC.lit.lit_0(i64 "imm" %lit, i32 "enum"="e3" %ssrc1) "enc"="{ if(lit<65536) if(lit>64) if((ssrc1<240)|(ssrc1>248)) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) literal_32:32=lit 1011111101010101 ssrc1_8:8=ssrc1 11111111 }" "asm"="s_cmp_lg_f16 {lit:h32}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_CMP_LG_F16.SOPC.lit.lit_1(i32 "enum"="e3" %ssrc0, i64 "imm" %lit) "enc"="{ if(lit<65536) if(lit>64) if((ssrc0<240)|(ssrc0>248)) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) literal_32:32=lit 101111110101010111111111 ssrc0_0:8=ssrc0 }" "asm"="s_cmp_lg_f16 {ssrc0}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_CMP_LG_F16.SOPC.lit.lit_0_lit_1(i64 "imm" %lit) "enc"="{ if(lit<65536) if(lit>64) literal_32:32=lit 10111111010101011111111111111111 }" "asm"="s_cmp_lg_f16 {lit:h32}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_CMP_GE_F32.SOPC(i32 "enum"="e3" %ssrc0, i32 "enum"="e3" %ssrc1) "enc"="{ if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) 1011111101000110 ssrc1_8:8=ssrc1 ssrc0_0:8=ssrc0 }" "asm"="s_cmp_ge_f32 {ssrc0}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_CMP_GE_F32.SOPC.lit.lit_0(i64 "imm" %lit, i32 "enum"="e3" %ssrc1) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) literal_32:32=lit 1011111101000110 ssrc1_8:8=ssrc1 11111111 }" "asm"="s_cmp_ge_f32 {lit:h32}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_CMP_GE_F32.SOPC.lit.lit_1(i32 "enum"="e3" %ssrc0, i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) literal_32:32=lit 101111110100011011111111 ssrc0_0:8=ssrc0 }" "asm"="s_cmp_ge_f32 {ssrc0}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_CMP_GE_F32.SOPC.lit.lit_0_lit_1(i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) literal_32:32=lit 10111111010001101111111111111111 }" "asm"="s_cmp_ge_f32 {lit:h32}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_CMP_GE_F16.SOPC(i32 "enum"="e3" %ssrc0, i32 "enum"="e3" %ssrc1) "enc"="{ if((ssrc0<240)|(ssrc0>248)) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) if((ssrc1<240)|(ssrc1>248)) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) 1011111101010110 ssrc1_8:8=ssrc1 ssrc0_0:8=ssrc0 }" "asm"="s_cmp_ge_f16 {ssrc0}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_CMP_GE_F16.SOPC.lit.lit_0(i64 "imm" %lit, i32 "enum"="e3" %ssrc1) "enc"="{ if(lit<65536) if(lit>64) if((ssrc1<240)|(ssrc1>248)) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) literal_32:32=lit 1011111101010110 ssrc1_8:8=ssrc1 11111111 }" "asm"="s_cmp_ge_f16 {lit:h32}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_CMP_GE_F16.SOPC.lit.lit_1(i32 "enum"="e3" %ssrc0, i64 "imm" %lit) "enc"="{ if(lit<65536) if(lit>64) if((ssrc0<240)|(ssrc0>248)) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) literal_32:32=lit 101111110101011011111111 ssrc0_0:8=ssrc0 }" "asm"="s_cmp_ge_f16 {ssrc0}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_CMP_GE_F16.SOPC.lit.lit_0_lit_1(i64 "imm" %lit) "enc"="{ if(lit<65536) if(lit>64) literal_32:32=lit 10111111010101101111111111111111 }" "asm"="s_cmp_ge_f16 {lit:h32}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_CMP_O_F32.SOPC(i32 "enum"="e3" %ssrc0, i32 "enum"="e3" %ssrc1) "enc"="{ if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) 1011111101000111 ssrc1_8:8=ssrc1 ssrc0_0:8=ssrc0 }" "asm"="s_cmp_o_f32 {ssrc0}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_CMP_O_F32.SOPC.lit.lit_0(i64 "imm" %lit, i32 "enum"="e3" %ssrc1) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) literal_32:32=lit 1011111101000111 ssrc1_8:8=ssrc1 11111111 }" "asm"="s_cmp_o_f32 {lit:h32}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_CMP_O_F32.SOPC.lit.lit_1(i32 "enum"="e3" %ssrc0, i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) literal_32:32=lit 101111110100011111111111 ssrc0_0:8=ssrc0 }" "asm"="s_cmp_o_f32 {ssrc0}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_CMP_O_F32.SOPC.lit.lit_0_lit_1(i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) literal_32:32=lit 10111111010001111111111111111111 }" "asm"="s_cmp_o_f32 {lit:h32}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_CMP_O_F16.SOPC(i32 "enum"="e3" %ssrc0, i32 "enum"="e3" %ssrc1) "enc"="{ if(ssrc0!=161) if((ssrc0<240)|(ssrc0>248)) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) if((ssrc1<240)|(ssrc1>248)) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) 1011111101010111 ssrc1_8:8=ssrc1 ssrc0_0:8=ssrc0 }" "asm"="s_cmp_o_f16 {ssrc0}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_CMP_O_F16.SOPC.lit.lit_0(i64 "imm" %lit, i32 "enum"="e3" %ssrc1) "enc"="{ if(lit<65536) if(lit>64) if((ssrc1<240)|(ssrc1>248)) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) literal_32:32=lit 1011111101010111 ssrc1_8:8=ssrc1 11111111 }" "asm"="s_cmp_o_f16 {lit:h32}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_CMP_O_F16.SOPC.lit.lit_1(i32 "enum"="e3" %ssrc0, i64 "imm" %lit) "enc"="{ if(lit<65536) if(lit>64) if((ssrc0<240)|(ssrc0>248)) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) literal_32:32=lit 101111110101011111111111 ssrc0_0:8=ssrc0 }" "asm"="s_cmp_o_f16 {ssrc0}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_CMP_O_F16.SOPC.lit.lit_0_lit_1(i64 "imm" %lit) "enc"="{ if(lit<65536) if(lit>64) literal_32:32=lit 10111111010101111111111111111111 }" "asm"="s_cmp_o_f16 {lit:h32}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_CMP_U_F32.SOPC(i32 "enum"="e3" %ssrc0, i32 "enum"="e3" %ssrc1) "enc"="{ if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) 1011111101001000 ssrc1_8:8=ssrc1 ssrc0_0:8=ssrc0 }" "asm"="s_cmp_u_f32 {ssrc0}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_CMP_U_F32.SOPC.lit.lit_0(i64 "imm" %lit, i32 "enum"="e3" %ssrc1) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) literal_32:32=lit 1011111101001000 ssrc1_8:8=ssrc1 11111111 }" "asm"="s_cmp_u_f32 {lit:h32}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_CMP_U_F32.SOPC.lit.lit_1(i32 "enum"="e3" %ssrc0, i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) literal_32:32=lit 101111110100100011111111 ssrc0_0:8=ssrc0 }" "asm"="s_cmp_u_f32 {ssrc0}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_CMP_U_F32.SOPC.lit.lit_0_lit_1(i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) literal_32:32=lit 10111111010010001111111111111111 }" "asm"="s_cmp_u_f32 {lit:h32}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_CMP_U_F16.SOPC(i32 "enum"="e3" %ssrc0, i32 "enum"="e3" %ssrc1) "enc"="{ if((ssrc0<240)|(ssrc0>248)) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) if((ssrc1<240)|(ssrc1>248)) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) 1011111101011000 ssrc1_8:8=ssrc1 ssrc0_0:8=ssrc0 }" "asm"="s_cmp_u_f16 {ssrc0}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_CMP_U_F16.SOPC.lit.lit_0(i64 "imm" %lit, i32 "enum"="e3" %ssrc1) "enc"="{ if(lit<65536) if(lit>64) if((ssrc1<240)|(ssrc1>248)) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) literal_32:32=lit 1011111101011000 ssrc1_8:8=ssrc1 11111111 }" "asm"="s_cmp_u_f16 {lit:h32}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_CMP_U_F16.SOPC.lit.lit_1(i32 "enum"="e3" %ssrc0, i64 "imm" %lit) "enc"="{ if(lit<65536) if(lit>64) if((ssrc0<240)|(ssrc0>248)) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) literal_32:32=lit 101111110101100011111111 ssrc0_0:8=ssrc0 }" "asm"="s_cmp_u_f16 {ssrc0}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_CMP_U_F16.SOPC.lit.lit_0_lit_1(i64 "imm" %lit) "enc"="{ if(lit<65536) if(lit>64) literal_32:32=lit 10111111010110001111111111111111 }" "asm"="s_cmp_u_f16 {lit:h32}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_CMP_NGE_F32.SOPC(i32 "enum"="e3" %ssrc0, i32 "enum"="e3" %ssrc1) "enc"="{ if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) if(ssrc1!=138) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) 1011111101001001 ssrc1_8:8=ssrc1 ssrc0_0:8=ssrc0 }" "asm"="s_cmp_nge_f32 {ssrc0}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_CMP_NGE_F32.SOPC.lit.lit_0(i64 "imm" %lit, i32 "enum"="e3" %ssrc1) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) literal_32:32=lit 1011111101001001 ssrc1_8:8=ssrc1 11111111 }" "asm"="s_cmp_nge_f32 {lit:h32}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_CMP_NGE_F32.SOPC.lit.lit_1(i32 "enum"="e3" %ssrc0, i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) literal_32:32=lit 101111110100100111111111 ssrc0_0:8=ssrc0 }" "asm"="s_cmp_nge_f32 {ssrc0}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_CMP_NGE_F32.SOPC.lit.lit_0_lit_1(i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) literal_32:32=lit 10111111010010011111111111111111 }" "asm"="s_cmp_nge_f32 {lit:h32}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_CMP_NGE_F16.SOPC(i32 "enum"="e3" %ssrc0, i32 "enum"="e3" %ssrc1) "enc"="{ if((ssrc0<240)|(ssrc0>248)) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) if((ssrc1<240)|(ssrc1>248)) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) 1011111101011001 ssrc1_8:8=ssrc1 ssrc0_0:8=ssrc0 }" "asm"="s_cmp_nge_f16 {ssrc0}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_CMP_NGE_F16.SOPC.lit.lit_0(i64 "imm" %lit, i32 "enum"="e3" %ssrc1) "enc"="{ if(lit<65536) if(lit>64) if((ssrc1<240)|(ssrc1>248)) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) literal_32:32=lit 1011111101011001 ssrc1_8:8=ssrc1 11111111 }" "asm"="s_cmp_nge_f16 {lit:h32}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_CMP_NGE_F16.SOPC.lit.lit_1(i32 "enum"="e3" %ssrc0, i64 "imm" %lit) "enc"="{ if(lit<65536) if(lit>64) if((ssrc0<240)|(ssrc0>248)) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) literal_32:32=lit 101111110101100111111111 ssrc0_0:8=ssrc0 }" "asm"="s_cmp_nge_f16 {ssrc0}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_CMP_NGE_F16.SOPC.lit.lit_0_lit_1(i64 "imm" %lit) "enc"="{ if(lit<65536) if(lit>64) literal_32:32=lit 10111111010110011111111111111111 }" "asm"="s_cmp_nge_f16 {lit:h32}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_CMP_NLG_F32.SOPC(i32 "enum"="e3" %ssrc0, i32 "enum"="e3" %ssrc1) "enc"="{ if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) 1011111101001010 ssrc1_8:8=ssrc1 ssrc0_0:8=ssrc0 }" "asm"="s_cmp_nlg_f32 {ssrc0}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_CMP_NLG_F32.SOPC.lit.lit_0(i64 "imm" %lit, i32 "enum"="e3" %ssrc1) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) literal_32:32=lit 1011111101001010 ssrc1_8:8=ssrc1 11111111 }" "asm"="s_cmp_nlg_f32 {lit:h32}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_CMP_NLG_F32.SOPC.lit.lit_1(i32 "enum"="e3" %ssrc0, i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) literal_32:32=lit 101111110100101011111111 ssrc0_0:8=ssrc0 }" "asm"="s_cmp_nlg_f32 {ssrc0}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_CMP_NLG_F32.SOPC.lit.lit_0_lit_1(i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) literal_32:32=lit 10111111010010101111111111111111 }" "asm"="s_cmp_nlg_f32 {lit:h32}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_CMP_NLG_F16.SOPC(i32 "enum"="e3" %ssrc0, i32 "enum"="e3" %ssrc1) "enc"="{ if((ssrc0<240)|(ssrc0>248)) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) if((ssrc1<240)|(ssrc1>248)) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) 1011111101011010 ssrc1_8:8=ssrc1 ssrc0_0:8=ssrc0 }" "asm"="s_cmp_nlg_f16 {ssrc0}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_CMP_NLG_F16.SOPC.lit.lit_0(i64 "imm" %lit, i32 "enum"="e3" %ssrc1) "enc"="{ if(lit<65536) if(lit>64) if((ssrc1<240)|(ssrc1>248)) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) literal_32:32=lit 1011111101011010 ssrc1_8:8=ssrc1 11111111 }" "asm"="s_cmp_nlg_f16 {lit:h32}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_CMP_NLG_F16.SOPC.lit.lit_1(i32 "enum"="e3" %ssrc0, i64 "imm" %lit) "enc"="{ if(lit<65536) if(lit>64) if((ssrc0<240)|(ssrc0>248)) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) literal_32:32=lit 101111110101101011111111 ssrc0_0:8=ssrc0 }" "asm"="s_cmp_nlg_f16 {ssrc0}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_CMP_NLG_F16.SOPC.lit.lit_0_lit_1(i64 "imm" %lit) "enc"="{ if(lit<65536) if(lit>64) literal_32:32=lit 10111111010110101111111111111111 }" "asm"="s_cmp_nlg_f16 {lit:h32}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_CMP_NGT_F32.SOPC(i32 "enum"="e3" %ssrc0, i32 "enum"="e3" %ssrc1) "enc"="{ if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) if(ssrc1!=27) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) 1011111101001011 ssrc1_8:8=ssrc1 ssrc0_0:8=ssrc0 }" "asm"="s_cmp_ngt_f32 {ssrc0}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_CMP_NGT_F32.SOPC.lit.lit_0(i64 "imm" %lit, i32 "enum"="e3" %ssrc1) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=15360) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) literal_32:32=lit 1011111101001011 ssrc1_8:8=ssrc1 11111111 }" "asm"="s_cmp_ngt_f32 {lit:h32}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_CMP_NGT_F32.SOPC.lit.lit_1(i32 "enum"="e3" %ssrc0, i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) literal_32:32=lit 101111110100101111111111 ssrc0_0:8=ssrc0 }" "asm"="s_cmp_ngt_f32 {ssrc0}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_CMP_NGT_F32.SOPC.lit.lit_0_lit_1(i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) literal_32:32=lit 10111111010010111111111111111111 }" "asm"="s_cmp_ngt_f32 {lit:h32}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_CMP_NGT_F16.SOPC(i32 "enum"="e3" %ssrc0, i32 "enum"="e3" %ssrc1) "enc"="{ if((ssrc0<240)|(ssrc0>248)) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) if(ssrc1!=142) if((ssrc1<240)|(ssrc1>248)) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) 1011111101011011 ssrc1_8:8=ssrc1 ssrc0_0:8=ssrc0 }" "asm"="s_cmp_ngt_f16 {ssrc0}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_CMP_NGT_F16.SOPC.lit.lit_0(i64 "imm" %lit, i32 "enum"="e3" %ssrc1) "enc"="{ if(lit<65536) if(lit>64) if((ssrc1<240)|(ssrc1>248)) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) literal_32:32=lit 1011111101011011 ssrc1_8:8=ssrc1 11111111 }" "asm"="s_cmp_ngt_f16 {lit:h32}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_CMP_NGT_F16.SOPC.lit.lit_1(i32 "enum"="e3" %ssrc0, i64 "imm" %lit) "enc"="{ if(lit<65536) if(lit>64) if((ssrc0<240)|(ssrc0>248)) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) literal_32:32=lit 101111110101101111111111 ssrc0_0:8=ssrc0 }" "asm"="s_cmp_ngt_f16 {ssrc0}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_CMP_NGT_F16.SOPC.lit.lit_0_lit_1(i64 "imm" %lit) "enc"="{ if(lit<65536) if(lit>64) literal_32:32=lit 10111111010110111111111111111111 }" "asm"="s_cmp_ngt_f16 {lit:h32}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_CMP_NLE_F32.SOPC(i32 "enum"="e3" %ssrc0, i32 "enum"="e3" %ssrc1) "enc"="{ if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) 1011111101001100 ssrc1_8:8=ssrc1 ssrc0_0:8=ssrc0 }" "asm"="s_cmp_nle_f32 {ssrc0}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_CMP_NLE_F32.SOPC.lit.lit_0(i64 "imm" %lit, i32 "enum"="e3" %ssrc1) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) literal_32:32=lit 1011111101001100 ssrc1_8:8=ssrc1 11111111 }" "asm"="s_cmp_nle_f32 {lit:h32}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_CMP_NLE_F32.SOPC.lit.lit_1(i32 "enum"="e3" %ssrc0, i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) literal_32:32=lit 101111110100110011111111 ssrc0_0:8=ssrc0 }" "asm"="s_cmp_nle_f32 {ssrc0}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_CMP_NLE_F32.SOPC.lit.lit_0_lit_1(i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) literal_32:32=lit 10111111010011001111111111111111 }" "asm"="s_cmp_nle_f32 {lit:h32}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_CMP_NLE_F16.SOPC(i32 "enum"="e3" %ssrc0, i32 "enum"="e3" %ssrc1) "enc"="{ if((ssrc0<240)|(ssrc0>248)) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) if((ssrc1<240)|(ssrc1>248)) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) 1011111101011100 ssrc1_8:8=ssrc1 ssrc0_0:8=ssrc0 }" "asm"="s_cmp_nle_f16 {ssrc0}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_CMP_NLE_F16.SOPC.lit.lit_0(i64 "imm" %lit, i32 "enum"="e3" %ssrc1) "enc"="{ if(lit<65536) if(lit>64) if((ssrc1<240)|(ssrc1>248)) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) literal_32:32=lit 1011111101011100 ssrc1_8:8=ssrc1 11111111 }" "asm"="s_cmp_nle_f16 {lit:h32}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_CMP_NLE_F16.SOPC.lit.lit_1(i32 "enum"="e3" %ssrc0, i64 "imm" %lit) "enc"="{ if(lit<65536) if(lit>64) if((ssrc0<240)|(ssrc0>248)) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) literal_32:32=lit 101111110101110011111111 ssrc0_0:8=ssrc0 }" "asm"="s_cmp_nle_f16 {ssrc0}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_CMP_NLE_F16.SOPC.lit.lit_0_lit_1(i64 "imm" %lit) "enc"="{ if(lit<65536) if(lit>64) literal_32:32=lit 10111111010111001111111111111111 }" "asm"="s_cmp_nle_f16 {lit:h32}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_CMP_NEQ_F32.SOPC(i32 "enum"="e3" %ssrc0, i32 "enum"="e3" %ssrc1) "enc"="{ if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) 1011111101001101 ssrc1_8:8=ssrc1 ssrc0_0:8=ssrc0 }" "asm"="s_cmp_neq_f32 {ssrc0}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_CMP_NEQ_F32.SOPC.lit.lit_0(i64 "imm" %lit, i32 "enum"="e3" %ssrc1) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) literal_32:32=lit 1011111101001101 ssrc1_8:8=ssrc1 11111111 }" "asm"="s_cmp_neq_f32 {lit:h32}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_CMP_NEQ_F32.SOPC.lit.lit_1(i32 "enum"="e3" %ssrc0, i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) literal_32:32=lit 101111110100110111111111 ssrc0_0:8=ssrc0 }" "asm"="s_cmp_neq_f32 {ssrc0}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_CMP_NEQ_F32.SOPC.lit.lit_0_lit_1(i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) literal_32:32=lit 10111111010011011111111111111111 }" "asm"="s_cmp_neq_f32 {lit:h32}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_CMP_NEQ_F16.SOPC(i32 "enum"="e3" %ssrc0, i32 "enum"="e3" %ssrc1) "enc"="{ if((ssrc0<240)|(ssrc0>248)) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) if((ssrc1<240)|(ssrc1>248)) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) 1011111101011101 ssrc1_8:8=ssrc1 ssrc0_0:8=ssrc0 }" "asm"="s_cmp_neq_f16 {ssrc0}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_CMP_NEQ_F16.SOPC.lit.lit_0(i64 "imm" %lit, i32 "enum"="e3" %ssrc1) "enc"="{ if(lit<65536) if(lit>64) if((ssrc1<240)|(ssrc1>248)) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) literal_32:32=lit 1011111101011101 ssrc1_8:8=ssrc1 11111111 }" "asm"="s_cmp_neq_f16 {lit:h32}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_CMP_NEQ_F16.SOPC.lit.lit_1(i32 "enum"="e3" %ssrc0, i64 "imm" %lit) "enc"="{ if(lit<65536) if(lit>64) if((ssrc0<240)|(ssrc0>248)) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) literal_32:32=lit 101111110101110111111111 ssrc0_0:8=ssrc0 }" "asm"="s_cmp_neq_f16 {ssrc0}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_CMP_NEQ_F16.SOPC.lit.lit_0_lit_1(i64 "imm" %lit) "enc"="{ if(lit<65536) if(lit>64) literal_32:32=lit 10111111010111011111111111111111 }" "asm"="s_cmp_neq_f16 {lit:h32}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_CMP_NLT_F32.SOPC(i32 "enum"="e3" %ssrc0, i32 "enum"="e3" %ssrc1) "enc"="{ if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) 1011111101001110 ssrc1_8:8=ssrc1 ssrc0_0:8=ssrc0 }" "asm"="s_cmp_nlt_f32 {ssrc0}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_CMP_NLT_F32.SOPC.lit.lit_0(i64 "imm" %lit, i32 "enum"="e3" %ssrc1) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) literal_32:32=lit 1011111101001110 ssrc1_8:8=ssrc1 11111111 }" "asm"="s_cmp_nlt_f32 {lit:h32}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_CMP_NLT_F32.SOPC.lit.lit_1(i32 "enum"="e3" %ssrc0, i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) literal_32:32=lit 101111110100111011111111 ssrc0_0:8=ssrc0 }" "asm"="s_cmp_nlt_f32 {ssrc0}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_CMP_NLT_F32.SOPC.lit.lit_0_lit_1(i64 "imm" %lit) "enc"="{ if(lit>64) if((lit<4294967280)|(lit>4294967295)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) literal_32:32=lit 10111111010011101111111111111111 }" "asm"="s_cmp_nlt_f32 {lit:h32}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_CMP_NLT_F16.SOPC(i32 "enum"="e3" %ssrc0, i32 "enum"="e3" %ssrc1) "enc"="{ if((ssrc0<240)|(ssrc0>248)) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) if((ssrc1<240)|(ssrc1>248)) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) 1011111101011110 ssrc1_8:8=ssrc1 ssrc0_0:8=ssrc0 }" "asm"="s_cmp_nlt_f16 {ssrc0}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_CMP_NLT_F16.SOPC.lit.lit_0(i64 "imm" %lit, i32 "enum"="e3" %ssrc1) "enc"="{ if(lit<65536) if(lit>64) if((ssrc1<240)|(ssrc1>248)) if((ssrc1<256)|(ssrc1>720)) if((ssrc1<747)|(ssrc1>750)) if((ssrc1<752)|(ssrc1>760)) if(ssrc1!=765) if((ssrc1<768)|(ssrc1>1023)) literal_32:32=lit 1011111101011110 ssrc1_8:8=ssrc1 11111111 }" "asm"="s_cmp_nlt_f16 {lit:h32}, {ssrc1}" "asmonly" {
  ret void
}
define void @S_CMP_NLT_F16.SOPC.lit.lit_1(i32 "enum"="e3" %ssrc0, i64 "imm" %lit) "enc"="{ if(lit<65536) if(lit>64) if((ssrc0<240)|(ssrc0>248)) if((ssrc0<256)|(ssrc0>720)) if((ssrc0<747)|(ssrc0>750)) if((ssrc0<752)|(ssrc0>760)) if(ssrc0!=765) if((ssrc0<768)|(ssrc0>1023)) literal_32:32=lit 101111110101111011111111 ssrc0_0:8=ssrc0 }" "asm"="s_cmp_nlt_f16 {ssrc0}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_CMP_NLT_F16.SOPC.lit.lit_0_lit_1(i64 "imm" %lit) "enc"="{ if(lit<65536) if(lit>64) literal_32:32=lit 10111111010111101111111111111111 }" "asm"="s_cmp_nlt_f16 {lit:h32}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_MOVK_I32.SOPK(i32 "enum"="e1" %sdst, i64 "imm" %simm16) "enc"="{ if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) 101100000 sdst_16:7=sdst simm16_0:16=simm16 }" "asm"="s_movk_i32 {sdst}, {simm16:h32}" "asmonly" {
  ret void
}
define void @S_CMOVK_I32.SOPK(i32 "enum"="e1" %sdst, i64 "imm" %simm16) "enc"="{ if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) 101100010 sdst_16:7=sdst simm16_0:16=simm16 }" "asm"="s_cmovk_i32 {sdst}, {simm16:h32}" "asmonly" {
  ret void
}
define void @S_CMPK_EQ_I32.SOPK(i32 "enum"="e1" %sdst, i64 "imm" %simm16) "enc"="{ if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) 101100011 sdst_16:7=sdst simm16_0:16=simm16 }" "asm"="s_cmpk_eq_i32 {sdst}, {simm16:h32}" "asmonly" {
  ret void
}
define void @S_CMPK_LG_I32.SOPK(i32 "enum"="e1" %sdst, i64 "imm" %simm16) "enc"="{ if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) 101100100 sdst_16:7=sdst simm16_0:16=simm16 }" "asm"="s_cmpk_lg_i32 {sdst}, {simm16:h32}" "asmonly" {
  ret void
}
define void @S_CMPK_GT_I32.SOPK(i32 "enum"="e1" %sdst, i64 "imm" %simm16) "enc"="{ if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) 101100101 sdst_16:7=sdst simm16_0:16=simm16 }" "asm"="s_cmpk_gt_i32 {sdst}, {simm16:h32}" "asmonly" {
  ret void
}
define void @S_CMPK_GE_I32.SOPK(i32 "enum"="e1" %sdst, i64 "imm" %simm16) "enc"="{ if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) 101100110 sdst_16:7=sdst simm16_0:16=simm16 }" "asm"="s_cmpk_ge_i32 {sdst}, {simm16:h32}" "asmonly" {
  ret void
}
define void @S_CMPK_LT_I32.SOPK(i32 "enum"="e1" %sdst, i64 "imm" %simm16) "enc"="{ if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) 101100111 sdst_16:7=sdst simm16_0:16=simm16 }" "asm"="s_cmpk_lt_i32 {sdst}, {simm16:h32}" "asmonly" {
  ret void
}
define void @S_CMPK_LE_I32.SOPK(i32 "enum"="e1" %sdst, i64 "imm" %simm16) "enc"="{ if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) 101101000 sdst_16:7=sdst simm16_0:16=simm16 }" "asm"="s_cmpk_le_i32 {sdst}, {simm16:h32}" "asmonly" {
  ret void
}
define void @S_CMPK_EQ_U32.SOPK(i32 "enum"="e1" %sdst, i64 "imm" %simm16) "enc"="{ if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) 101101001 sdst_16:7=sdst simm16_0:16=simm16 }" "asm"="s_cmpk_eq_u32 {sdst}, {simm16:h32}" "asmonly" {
  ret void
}
define void @S_CMPK_LG_U32.SOPK(i32 "enum"="e1" %sdst, i64 "imm" %simm16) "enc"="{ if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) 101101010 sdst_16:7=sdst simm16_0:16=simm16 }" "asm"="s_cmpk_lg_u32 {sdst}, {simm16:h32}" "asmonly" {
  ret void
}
define void @S_CMPK_GT_U32.SOPK(i32 "enum"="e1" %sdst, i64 "imm" %simm16) "enc"="{ if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) 101101011 sdst_16:7=sdst simm16_0:16=simm16 }" "asm"="s_cmpk_gt_u32 {sdst}, {simm16:h32}" "asmonly" {
  ret void
}
define void @S_CMPK_GE_U32.SOPK(i32 "enum"="e1" %sdst, i64 "imm" %simm16) "enc"="{ if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) 101101100 sdst_16:7=sdst simm16_0:16=simm16 }" "asm"="s_cmpk_ge_u32 {sdst}, {simm16:h32}" "asmonly" {
  ret void
}
define void @S_CMPK_LT_U32.SOPK(i32 "enum"="e1" %sdst, i64 "imm" %simm16) "enc"="{ if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) 101101101 sdst_16:7=sdst simm16_0:16=simm16 }" "asm"="s_cmpk_lt_u32 {sdst}, {simm16:h32}" "asmonly" {
  ret void
}
define void @S_CMPK_LE_U32.SOPK(i32 "enum"="e1" %sdst, i64 "imm" %simm16) "enc"="{ if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) 101101110 sdst_16:7=sdst simm16_0:16=simm16 }" "asm"="s_cmpk_le_u32 {sdst}, {simm16:h32}" "asmonly" {
  ret void
}
define void @S_ADDK_I32.SOPK(i32 "enum"="e1" %sdst, i64 "imm" %simm16) "enc"="{ if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) 101101111 sdst_16:7=sdst simm16_0:16=simm16 }" "asm"="s_addk_i32 {sdst}, {simm16:h32}" "asmonly" {
  ret void
}
define void @S_MULK_I32.SOPK(i32 "enum"="e1" %sdst, i64 "imm" %simm16) "enc"="{ if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) 101110000 sdst_16:7=sdst simm16_0:16=simm16 }" "asm"="s_mulk_i32 {sdst}, {simm16:h32}" "asmonly" {
  ret void
}
define void @S_GETREG_B32.SOPK.0(i32 "enum"="e1" %sdst, i32 "enum"="e36" %hwreg) "enc"="{ if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) 101110001 sdst_16:7=sdst 1111100000 simm1606_0:6=hwreg }" "asm"="s_getreg_b32 {sdst}, {hwreg}" "asmonly" {
  ret void
}
define void @S_SETREG_B32.SOPK.0(i32 "enum"="e1" %sdst, i32 "enum"="e36" %hwreg) "enc"="{ if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if((sdst<253)|(sdst>254)) if((sdst<256)|(sdst>511)) 101110010 sdst_16:7=sdst 1111100000 simm1606_0:6=hwreg }" "asm"="s_setreg_b32 {hwreg}, {sdst}" "asmonly" {
  ret void
}
define void @S_SETREG_IMM32_B32.SOPK.lit.0.hi(i64 "imm" %lit, i32 "enum"="e36" %hwreg) "enc"="{ if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if(lit>64) literal_32:32=lit 10111001100000001111100000 simm1606_0:6=hwreg }" "asm"="s_setreg_imm32_b32 {hwreg}, {lit:h32}" "asmonly" {
  ret void
}
define void @S_SETREG_IMM32_B32.SOPK.lit.1.hi(i64 "imm" %lit, i32 "enum"="e37" %hwreg, i64 "imm" %hw_offset, i64 "imm" %hw_size) "enc"="{ if((hw_offset!=0)|(hw_size!=31)) if(lit!=1042479491) if(lit!=1056964608) if(lit!=1065353216) if(lit!=1073741824) if(lit!=1082130432) if(lit!=3204448256) if(lit!=3212836864) if(lit!=3221225472) if(lit!=3229614080) if(lit>64) literal_32:32=lit 1011100110000000 simm16115_11:5=hw_size simm1665_6:5=hw_offset simm1606_0:6=hwreg }" "asm"="s_setreg_imm32_b32 {hwreg}, {hw_offset}, {=hw_size+1}), {lit:h32}" "asmonly" {
  ret void
}
define void @S_CALL_B64.SOPK(i32 "enum"="e18" %sdst, i64 "imm" %simm16) "enc"="{ if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if(sdst!=253) if((sdst<256)|(sdst>510)) 101110100 sdst_16:7=sdst simm16_0:16=simm16 }" "asm"="s_call_b64 {sdst}, {simm16}" "asmonly" {
  ret void
}
define void @S_CALL_B64.SOPK.label(i32 "enum"="e18" %sdst, ptr "label" %dest) "enc"="{ if((sdst<128)|(sdst>208)) if((sdst<235)|(sdst>238)) if((sdst<240)|(sdst>248)) if(sdst!=253) if((sdst<256)|(sdst>510)) 101110100 sdst_16:7=sdst simm16_0:s16=(dest-end)/4 }" "asm"="s_call_b64 {sdst}, {dest}" "asmonly" "nodecode" {
  ret void
}
define void @S_WAITCNT_VSCNT.SOPK(i64 "imm" %simm16) "enc"="1011110001111100 simm16_0:16=simm16" "asm"="s_waitcnt_vscnt null, {simm16:h32}" "asmonly" {
  ret void
}
define void @S_WAITCNT_VMCNT.SOPK(i64 "imm" %simm16) "enc"="1011110011111100 simm16_0:16=simm16" "asm"="s_waitcnt_vmcnt null, {simm16:h32}" "asmonly" {
  ret void
}
define void @S_WAITCNT_EXPCNT.SOPK(i64 "imm" %simm16) "enc"="1011110101111100 simm16_0:16=simm16" "asm"="s_waitcnt_expcnt null, {simm16:h32}" "asmonly" {
  ret void
}
define void @S_WAITCNT_LGKMCNT.SOPK(i64 "imm" %simm16) "enc"="1011110111111100 simm16_0:16=simm16" "asm"="s_waitcnt_lgkmcnt null, {simm16:h32}" "asmonly" {
  ret void
}
define void @S_NOP.SOPP.lo(i64 "imm" %simm16) "enc"="{ if(simm16<=64) 1011111110000000 simm16_0:16=simm16 }" "asm"="s_nop {simm16}" "asmonly" {
  ret void
}
define void @S_NOP.SOPP.hi(i64 "imm" %simm16) "enc"="{ if(simm16>64) 1011111110000000 simm16_0:16=simm16 }" "asm"="s_nop {simm16:h32}" "asmonly" {
  ret void
}
define void @S_SETKILL.SOPP.lo(i64 "imm" %simm16) "enc"="{ if(simm16<=64) 1011111110000001 simm16_0:16=simm16 }" "asm"="s_setkill {simm16}" "asmonly" {
  ret void
}
define void @S_SETKILL.SOPP.hi(i64 "imm" %simm16) "enc"="{ if(simm16>64) 1011111110000001 simm16_0:16=simm16 }" "asm"="s_setkill {simm16:h32}" "asmonly" {
  ret void
}
define void @S_SETHALT.SOPP.lo(i64 "imm" %simm16) "enc"="{ if(simm16<=64) 1011111110000010 simm16_0:16=simm16 }" "asm"="s_sethalt {simm16}" "asmonly" {
  ret void
}
define void @S_SETHALT.SOPP.hi(i64 "imm" %simm16) "enc"="{ if(simm16>64) 1011111110000010 simm16_0:16=simm16 }" "asm"="s_sethalt {simm16:h32}" "asmonly" {
  ret void
}
define void @S_SLEEP.SOPP.lo(i64 "imm" %simm16) "enc"="{ if(simm16<=64) 1011111110000011 simm16_0:16=simm16 }" "asm"="s_sleep {simm16}" "asmonly" {
  ret void
}
define void @S_SLEEP.SOPP.hi(i64 "imm" %simm16) "enc"="{ if(simm16>64) 1011111110000011 simm16_0:16=simm16 }" "asm"="s_sleep {simm16:h32}" "asmonly" {
  ret void
}
define void @S_SET_INST_PREFETCH_DISTANCE.SOPP(i64 "imm" %simm16) "enc"="1011111110000100 simm16_0:16=simm16" "asm"="s_set_inst_prefetch_distance {simm16:h32}" "asmonly" {
  ret void
}
define void @S_CLAUSE.SOPP(i64 "imm" %simm16) "enc"="1011111110000101 simm16_0:16=simm16" "asm"="s_clause {simm16:h32}" "asmonly" {
  ret void
}
define void @S_DELAY_ALU.SOPP() "enc"="10111111100001110000000000000000" "asm"="s_delay_alu 0" "asmonly" {
  ret void
}
define void @S_DELAY_ALU.SOPP.instid1(i32 "enum"="e38" %instid1) "enc"="{ if(instid1!=0) 101111111000011100000 simm1674_7:4=instid1 0000000 }" "asm"="s_delay_alu {instid1}" "asmonly" {
  ret void
}
define void @S_DELAY_ALU.SOPP.instskip(i32 "enum"="e39" %instskip) "enc"="{ if(instskip!=0) 1011111110000111000000000 simm1643_4:3=instskip 0000 }" "asm"="s_delay_alu {instskip}" "asmonly" {
  ret void
}
define void @S_DELAY_ALU.SOPP.instskip.instid1(i32 "enum"="e39" %instskip, i32 "enum"="e38" %instid1) "enc"="{ if(instskip!=0) if(instid1!=0) 101111111000011100000 simm1674_7:4=instid1 simm1643_4:3=instskip 0000 }" "asm"="s_delay_alu {instskip} | {instid1}" "asmonly" {
  ret void
}
define void @S_DELAY_ALU.SOPP.instid0(i32 "enum"="e40" %instid0) "enc"="{ if(instid0!=0) 1011111110000111000000000000 simm1604_0:4=instid0 }" "asm"="s_delay_alu {instid0}" "asmonly" {
  ret void
}
define void @S_DELAY_ALU.SOPP.instid0.instid1(i32 "enum"="e40" %instid0, i32 "enum"="e38" %instid1) "enc"="{ if(instid0!=0) if(instid1!=0) 101111111000011100000 simm1674_7:4=instid1 000 simm1604_0:4=instid0 }" "asm"="s_delay_alu {instid0} | {instid1}" "asmonly" {
  ret void
}
define void @S_DELAY_ALU.SOPP.instid0.instskip(i32 "enum"="e40" %instid0, i32 "enum"="e39" %instskip) "enc"="{ if(instid0!=0) if(instskip!=0) 1011111110000111000000000 simm1643_4:3=instskip simm1604_0:4=instid0 }" "asm"="s_delay_alu {instid0} | {instskip}" "asmonly" {
  ret void
}
define void @S_DELAY_ALU.SOPP.instid0.instskip.instid1(i32 "enum"="e40" %instid0, i32 "enum"="e39" %instskip, i32 "enum"="e38" %instid1) "enc"="{ if(instid0!=0) if(instskip!=0) if(instid1!=0) 101111111000011100000 simm1674_7:4=instid1 simm1643_4:3=instskip simm1604_0:4=instid0 }" "asm"="s_delay_alu {instid0} | {instskip} | {instid1}" "asmonly" {
  ret void
}
define void @S_WAITCNT_DEPCTR.SOPP() "enc"="10111111100010001111111110011111" "asm"="s_waitcnt_depctr depctr_hold_cnt(1) depctr_sa_sdst(1) depctr_va_vdst(15) depctr_va_sdst(7) depctr_va_ssrc(1) depctr_va_vcc(1) depctr_vm_vsrc(7)" "asmonly" {
  ret void
}
define void @S_WAITCNT_DEPCTR.SOPP.va_vdst(i64 "imm" %va_vdst) "enc"="{ if(va_vdst!=15) 1011111110001000 simm16124_12:4=va_vdst 111110011111 }" "asm"="s_waitcnt_depctr depctr_va_vdst({va_vdst})" "asmonly" {
  ret void
}
define void @S_WAITCNT_DEPCTR.SOPP.va_sdst(i64 "imm" %va_sdst) "enc"="{ if(va_sdst!=7) 10111111100010001111 simm1693_9:3=va_sdst 110011111 }" "asm"="s_waitcnt_depctr depctr_va_sdst({va_sdst})" "asmonly" {
  ret void
}
define void @S_WAITCNT_DEPCTR.SOPP.va_sdst.va_vdst(i64 "imm" %va_sdst, i64 "imm" %va_vdst) "enc"="{ if(va_sdst!=7) if(va_vdst!=15) 1011111110001000 simm16124_12:4=va_vdst simm1693_9:3=va_sdst 110011111 }" "asm"="s_waitcnt_depctr depctr_va_vdst({va_vdst}) depctr_va_sdst({va_sdst})" "asmonly" {
  ret void
}
define void @S_WAITCNT_DEPCTR.SOPP.vm_vsrc(i64 "imm" %vm_vsrc) "enc"="{ if(vm_vsrc!=7) 101111111000100011111111100 simm1623_2:3=vm_vsrc 11 }" "asm"="s_waitcnt_depctr depctr_vm_vsrc({vm_vsrc})" "asmonly" {
  ret void
}
define void @S_WAITCNT_DEPCTR.SOPP.vm_vsrc.va_vdst(i64 "imm" %vm_vsrc, i64 "imm" %va_vdst) "enc"="{ if(vm_vsrc!=7) if(va_vdst!=15) 1011111110001000 simm16124_12:4=va_vdst 1111100 simm1623_2:3=vm_vsrc 11 }" "asm"="s_waitcnt_depctr depctr_va_vdst({va_vdst}) depctr_vm_vsrc({vm_vsrc})" "asmonly" {
  ret void
}
define void @S_WAITCNT_DEPCTR.SOPP.vm_vsrc.va_sdst(i64 "imm" %vm_vsrc, i64 "imm" %va_sdst) "enc"="{ if(vm_vsrc!=7) if(va_sdst!=7) 10111111100010001111 simm1693_9:3=va_sdst 1100 simm1623_2:3=vm_vsrc 11 }" "asm"="s_waitcnt_depctr depctr_va_sdst({va_sdst}) depctr_vm_vsrc({vm_vsrc})" "asmonly" {
  ret void
}
define void @S_WAITCNT_DEPCTR.SOPP.vm_vsrc.va_sdst.va_vdst(i64 "imm" %vm_vsrc, i64 "imm" %va_sdst, i64 "imm" %va_vdst) "enc"="{ if(vm_vsrc!=7) if(va_sdst!=7) if(va_vdst!=15) 1011111110001000 simm16124_12:4=va_vdst simm1693_9:3=va_sdst 1100 simm1623_2:3=vm_vsrc 11 }" "asm"="s_waitcnt_depctr depctr_va_vdst({va_vdst}) depctr_va_sdst({va_sdst}) depctr_vm_vsrc({vm_vsrc})" "asmonly" {
  ret void
}
define void @S_WAITCNT.SOPP() "enc"="10111111100010011111111111110111" "asm"="s_waitcnt vmcnt(63) expcnt(7) lgkmcnt(63)" "asmonly" {
  ret void
}
define void @S_WAITCNT.SOPP.vmcnt(i64 "imm" %vmcnt) "enc"="{ if(vmcnt!=63) 1011111110001001 simm16106_10:6=vmcnt 1111110111 }" "asm"="s_waitcnt vmcnt({vmcnt})" "asmonly" {
  ret void
}
define void @S_WAITCNT.SOPP.lgkmcnt(i64 "imm" %lgkmcnt) "enc"="{ if(lgkmcnt!=63) 1011111110001001111111 simm1646_4:6=lgkmcnt 0111 }" "asm"="s_waitcnt lgkmcnt({lgkmcnt})" "asmonly" {
  ret void
}
define void @S_WAITCNT.SOPP.lgkmcnt.vmcnt(i64 "imm" %lgkmcnt, i64 "imm" %vmcnt) "enc"="{ if(lgkmcnt!=63) if(vmcnt!=63) 1011111110001001 simm16106_10:6=vmcnt simm1646_4:6=lgkmcnt 0111 }" "asm"="s_waitcnt vmcnt({vmcnt}) lgkmcnt({lgkmcnt})" "asmonly" {
  ret void
}
define void @S_WAITCNT.SOPP.expcnt(i64 "imm" %expcnt) "enc"="{ if(expcnt!=7) 10111111100010011111111111110 simm1603_0:3=expcnt }" "asm"="s_waitcnt expcnt({expcnt})" "asmonly" {
  ret void
}
define void @S_WAITCNT.SOPP.expcnt.vmcnt(i64 "imm" %expcnt, i64 "imm" %vmcnt) "enc"="{ if(expcnt!=7) if(vmcnt!=63) 1011111110001001 simm16106_10:6=vmcnt 1111110 simm1603_0:3=expcnt }" "asm"="s_waitcnt vmcnt({vmcnt}) expcnt({expcnt})" "asmonly" {
  ret void
}
define void @S_WAITCNT.SOPP.expcnt.lgkmcnt(i64 "imm" %expcnt, i64 "imm" %lgkmcnt) "enc"="{ if(expcnt!=7) if(lgkmcnt!=63) 1011111110001001111111 simm1646_4:6=lgkmcnt 0 simm1603_0:3=expcnt }" "asm"="s_waitcnt expcnt({expcnt}) lgkmcnt({lgkmcnt})" "asmonly" {
  ret void
}
define void @S_WAITCNT.SOPP.expcnt.lgkmcnt.vmcnt(i64 "imm" %expcnt, i64 "imm" %lgkmcnt, i64 "imm" %vmcnt) "enc"="{ if(expcnt!=7) if(lgkmcnt!=63) if(vmcnt!=63) 1011111110001001 simm16106_10:6=vmcnt simm1646_4:6=lgkmcnt 0 simm1603_0:3=expcnt }" "asm"="s_waitcnt vmcnt({vmcnt}) expcnt({expcnt}) lgkmcnt({lgkmcnt})" "asmonly" {
  ret void
}
define void @S_WAIT_IDLE.SOPP() "enc"="10111111100010100000000000000000" "asm"="s_wait_idle" "asmonly" {
  ret void
}
define void @S_WAIT_EVENT.SOPP(i64 "imm" %simm16) "enc"="1011111110001011 simm16_0:16=simm16" "asm"="s_wait_event {simm16:h32}" "asmonly" {
  ret void
}
define void @S_TRAP.SOPP.lo(i64 "imm" %simm16) "enc"="{ if(simm16<=64) 1011111110010000 simm16_0:16=simm16 }" "asm"="s_trap {simm16}" "asmonly" {
  ret void
}
define void @S_TRAP.SOPP.hi(i64 "imm" %simm16) "enc"="{ if(simm16>64) 1011111110010000 simm16_0:16=simm16 }" "asm"="s_trap {simm16:h32}" "asmonly" {
  ret void
}
define void @S_ROUND_MODE.SOPP(i64 "imm" %simm16) "enc"="1011111110010001 simm16_0:16=simm16" "asm"="s_round_mode {simm16:h32}" "asmonly" {
  ret void
}
define void @S_DENORM_MODE.SOPP.lo(i64 "imm" %simm16) "enc"="{ if(simm16<=64) 1011111110010010 simm16_0:16=simm16 }" "asm"="s_denorm_mode {simm16}" "asmonly" {
  ret void
}
define void @S_DENORM_MODE.SOPP.hi(i64 "imm" %simm16) "enc"="{ if(simm16>64) 1011111110010010 simm16_0:16=simm16 }" "asm"="s_denorm_mode {simm16:h32}" "asmonly" {
  ret void
}
define void @S_CODE_END.SOPP() "enc"="10111111100111110000000000000000" "asm"="s_code_end" "asmonly" {
  ret void
}
define void @S_BRANCH.SOPP(i64 "imm" %simm16) "enc"="1011111110100000 simm16_0:16=simm16" "asm"="s_branch {simm16}" "asmonly" {
  ret void
}
define void @S_BRANCH.SOPP.label(ptr "label" %dest) "enc"="1011111110100000 simm16_0:s16=(dest-end)/4" "asm"="s_branch {dest}" "asmonly" "nodecode" {
  ret void
}
define void @S_CBRANCH_SCC0.SOPP(i64 "imm" %simm16) "enc"="1011111110100001 simm16_0:16=simm16" "asm"="s_cbranch_scc0 {simm16}" "asmonly" {
  ret void
}
define void @S_CBRANCH_SCC0.SOPP.label(ptr "label" %dest) "enc"="1011111110100001 simm16_0:s16=(dest-end)/4" "asm"="s_cbranch_scc0 {dest}" "asmonly" "nodecode" {
  ret void
}
define void @S_CBRANCH_SCC1.SOPP(i64 "imm" %simm16) "enc"="1011111110100010 simm16_0:16=simm16" "asm"="s_cbranch_scc1 {simm16}" "asmonly" {
  ret void
}
define void @S_CBRANCH_SCC1.SOPP.label(ptr "label" %dest) "enc"="1011111110100010 simm16_0:s16=(dest-end)/4" "asm"="s_cbranch_scc1 {dest}" "asmonly" "nodecode" {
  ret void
}
define void @S_CBRANCH_VCCZ.SOPP(i64 "imm" %simm16) "enc"="1011111110100011 simm16_0:16=simm16" "asm"="s_cbranch_vccz {simm16}" "asmonly" {
  ret void
}
define void @S_CBRANCH_VCCZ.SOPP.label(ptr "label" %dest) "enc"="1011111110100011 simm16_0:s16=(dest-end)/4" "asm"="s_cbranch_vccz {dest}" "asmonly" "nodecode" {
  ret void
}
define void @S_CBRANCH_VCCNZ.SOPP(i64 "imm" %simm16) "enc"="1011111110100100 simm16_0:16=simm16" "asm"="s_cbranch_vccnz {simm16}" "asmonly" {
  ret void
}
define void @S_CBRANCH_VCCNZ.SOPP.label(ptr "label" %dest) "enc"="1011111110100100 simm16_0:s16=(dest-end)/4" "asm"="s_cbranch_vccnz {dest}" "asmonly" "nodecode" {
  ret void
}
define void @S_CBRANCH_EXECZ.SOPP(i64 "imm" %simm16) "enc"="1011111110100101 simm16_0:16=simm16" "asm"="s_cbranch_execz {simm16}" "asmonly" {
  ret void
}
define void @S_CBRANCH_EXECZ.SOPP.label(ptr "label" %dest) "enc"="1011111110100101 simm16_0:s16=(dest-end)/4" "asm"="s_cbranch_execz {dest}" "asmonly" "nodecode" {
  ret void
}
define void @S_CBRANCH_EXECNZ.SOPP(i64 "imm" %simm16) "enc"="1011111110100110 simm16_0:16=simm16" "asm"="s_cbranch_execnz {simm16}" "asmonly" {
  ret void
}
define void @S_CBRANCH_EXECNZ.SOPP.label(ptr "label" %dest) "enc"="1011111110100110 simm16_0:s16=(dest-end)/4" "asm"="s_cbranch_execnz {dest}" "asmonly" "nodecode" {
  ret void
}
define void @S_CBRANCH_CDBGSYS.SOPP(i64 "imm" %simm16) "enc"="1011111110100111 simm16_0:16=simm16" "asm"="s_cbranch_cdbgsys {simm16}" "asmonly" {
  ret void
}
define void @S_CBRANCH_CDBGSYS.SOPP.label(ptr "label" %dest) "enc"="1011111110100111 simm16_0:s16=(dest-end)/4" "asm"="s_cbranch_cdbgsys {dest}" "asmonly" "nodecode" {
  ret void
}
define void @S_CBRANCH_CDBGUSER.SOPP(i64 "imm" %simm16) "enc"="1011111110101000 simm16_0:16=simm16" "asm"="s_cbranch_cdbguser {simm16}" "asmonly" {
  ret void
}
define void @S_CBRANCH_CDBGUSER.SOPP.label(ptr "label" %dest) "enc"="1011111110101000 simm16_0:s16=(dest-end)/4" "asm"="s_cbranch_cdbguser {dest}" "asmonly" "nodecode" {
  ret void
}
define void @S_CBRANCH_CDBGSYS_OR_USER.SOPP(i64 "imm" %simm16) "enc"="1011111110101001 simm16_0:16=simm16" "asm"="s_cbranch_cdbgsys_or_user {simm16}" "asmonly" {
  ret void
}
define void @S_CBRANCH_CDBGSYS_OR_USER.SOPP.label(ptr "label" %dest) "enc"="1011111110101001 simm16_0:s16=(dest-end)/4" "asm"="s_cbranch_cdbgsys_or_user {dest}" "asmonly" "nodecode" {
  ret void
}
define void @S_CBRANCH_CDBGSYS_AND_USER.SOPP(i64 "imm" %simm16) "enc"="1011111110101010 simm16_0:16=simm16" "asm"="s_cbranch_cdbgsys_and_user {simm16}" "asmonly" {
  ret void
}
define void @S_CBRANCH_CDBGSYS_AND_USER.SOPP.label(ptr "label" %dest) "enc"="1011111110101010 simm16_0:s16=(dest-end)/4" "asm"="s_cbranch_cdbgsys_and_user {dest}" "asmonly" "nodecode" {
  ret void
}
define void @S_ENDPGM.SOPP() "enc"="10111111101100000000000000000000" "asm"="s_endpgm" "asmonly" {
  ret void
}
define void @S_ENDPGM_SAVED.SOPP() "enc"="10111111101100010000000000000000" "asm"="s_endpgm_saved" "asmonly" {
  ret void
}
define void @S_ENDPGM_ORDERED_PS_DONE.SOPP() "enc"="10111111101100100000000000000000" "asm"="s_endpgm_ordered_ps_done" "asmonly" {
  ret void
}
define void @S_WAKEUP.SOPP() "enc"="10111111101101000000000000000000" "asm"="s_wakeup" "asmonly" {
  ret void
}
define void @S_SETPRIO.SOPP.lo(i64 "imm" %simm16) "enc"="{ if(simm16<=64) 1011111110110101 simm16_0:16=simm16 }" "asm"="s_setprio {simm16}" "asmonly" {
  ret void
}
define void @S_SETPRIO.SOPP.hi(i64 "imm" %simm16) "enc"="{ if(simm16>64) 1011111110110101 simm16_0:16=simm16 }" "asm"="s_setprio {simm16:h32}" "asmonly" {
  ret void
}
define void @S_SENDMSG.SOPP.0(i32 "enum"="e35" %msg) "enc"="101111111011011000000000 simm1608_0:8=msg" "asm"="s_sendmsg {msg}" "asmonly" {
  ret void
}
define void @S_SENDMSG.SOPP.1(i64 "imm" %simm16) "enc"="{ if(simm16>=256) if((simm16&255)!=1) if((simm16&255)!=2) if((simm16&255)!=3) if((simm16&255)!=5) if((simm16&255)!=6) if((simm16&255)!=9) if((simm16&255)!=128) if((simm16&255)!=129) if((simm16&255)!=130) if((simm16&255)!=131) if((simm16&255)!=132) if((simm16&255)!=133) if((simm16&255)!=134) 1011111110110110 simm16_0:16=simm16 }" "asm"="s_sendmsg {simm16}" "asmonly" {
  ret void
}
define void @S_SENDMSGHALT.SOPP.0(i32 "enum"="e35" %msg) "enc"="101111111011011100000000 simm1608_0:8=msg" "asm"="s_sendmsghalt {msg}" "asmonly" {
  ret void
}
define void @S_SENDMSGHALT.SOPP.1(i64 "imm" %simm16) "enc"="{ if(simm16>=256) if((simm16&255)!=1) if((simm16&255)!=2) if((simm16&255)!=3) if((simm16&255)!=5) if((simm16&255)!=6) if((simm16&255)!=9) if((simm16&255)!=128) if((simm16&255)!=129) if((simm16&255)!=130) if((simm16&255)!=131) if((simm16&255)!=132) if((simm16&255)!=133) if((simm16&255)!=134) 1011111110110111 simm16_0:16=simm16 }" "asm"="s_sendmsghalt {simm16}" "asmonly" {
  ret void
}
define void @S_INCPERFLEVEL.SOPP.lo(i64 "imm" %simm16) "enc"="{ if(simm16<=64) 1011111110111000 simm16_0:16=simm16 }" "asm"="s_incperflevel {simm16}" "asmonly" {
  ret void
}
define void @S_INCPERFLEVEL.SOPP.hi(i64 "imm" %simm16) "enc"="{ if(simm16>64) 1011111110111000 simm16_0:16=simm16 }" "asm"="s_incperflevel {simm16:h32}" "asmonly" {
  ret void
}
define void @S_DECPERFLEVEL.SOPP.lo(i64 "imm" %simm16) "enc"="{ if(simm16<=64) 1011111110111001 simm16_0:16=simm16 }" "asm"="s_decperflevel {simm16}" "asmonly" {
  ret void
}
define void @S_DECPERFLEVEL.SOPP.hi(i64 "imm" %simm16) "enc"="{ if(simm16>64) 1011111110111001 simm16_0:16=simm16 }" "asm"="s_decperflevel {simm16:h32}" "asmonly" {
  ret void
}
define void @S_TTRACEDATA.SOPP() "enc"="10111111101110100000000000000000" "asm"="s_ttracedata" "asmonly" {
  ret void
}
define void @S_TTRACEDATA_IMM.SOPP(i64 "imm" %simm16) "enc"="1011111110111011 simm16_0:16=simm16" "asm"="s_ttracedata_imm {simm16:h32}" "asmonly" {
  ret void
}
define void @S_ICACHE_INV.SOPP() "enc"="10111111101111000000000000000000" "asm"="s_icache_inv" "asmonly" {
  ret void
}
define void @S_BARRIER.SOPP() "enc"="10111111101111010000000000000000" "asm"="s_barrier" "asmonly" {
  ret void
}
