; A made-up VLIW machine, to test bundles (docs/design/vliw.md, section 1):
; a 64-bit bundle of a 16-bit header and three 16-bit slots. Units X and Y
; each have a slot, written in lanes 0 and 1 (`a :: b`); a third slot takes
; either. Two register files, a and b, are each read through one port: the
; header holds the register a bundle reads from each, and whether the file
; rotates.

!target.lacks = !{!3}
!3 = !{!"stack", !"calls", !"globals", !"indirect", !"trap"}

!target.params = !{!1, !2}
!1 = !{!"instruction-unit", !"2"}
!2 = !{!"asm-comment", !";"}

!target.registers = !{!5}
!5 = !{!"a", !"a{0-7}"}
!target.classes = !{!6}
!6 = !{!"a", !"file", !"a"}

; A source: file a (port 0) or b (port 1), its register or its rotating
; register (R): port << 4 | rotating << 3 | register.
!target.enums = !{!10, !11}
!10 = !{!"src", !"0", !"a{0-7}", !"8", !"aR", !"16", !"b{0-3}", !"24", !"bR"}
!11 = !{!"dst", !"0", !"a{0-7}"}

!target.bundles = !{!20}
!20 = !{!"name", !"toy", !"section", !".text.toy", !"bytes", !"8", !"lanes", !"::", !"end", !"last=1",
        !"field", !"port", !"0:3 3:3", !"field", !"rot", !"6:1 7:1", !"field", !"len", !"8:3",
        !"field", !"rep", !"11:1", !"field", !"last", !"12:1",
        !"slot", !"16:16", !"X", !"slot", !"32:16", !"Y", !"slot", !"48:16", !"X,Y",
        !"prefix", !"rep {len} last", !"rep=1 last=1", !"prefix", !"rep {len}", !"rep=1",
        !"body", !"len"}

define void @ADDX(i32 "enum"="dst" %d, i32 "enum"="src" %s, i32 "enum"="src" %t) "enc"="0001 d:3 t:1=t>>4 s:1=s>>4 0000000" "asm"="add {d}, {s}, {t}" "unit"="X" "lane"="0" "header"="port[s>>4]=s&7; rot[s>>4]=1 if (s>>3)&1; port[t>>4]=t&7; rot[t>>4]=1 if (t>>3)&1" "asmonly" {
  ret void
}

define void @ADDY(i32 "enum"="dst" %d, i32 "enum"="src" %s, i32 "enum"="src" %t) "enc"="0001 d:3 t:1=t>>4 s:1=s>>4 0000000" "asm"="add {d}, {s}, {t}" "unit"="Y" "lane"="1" "header"="port[s>>4]=s&7; rot[s>>4]=1 if (s>>3)&1; port[t>>4]=t&7; rot[t>>4]=1 if (t>>3)&1" "asmonly" {
  ret void
}

define void @MOV(i32 "enum"="dst" %d, i32 "enum"="src" %s) "enc"="0010 d:3 0 s:1=s>>4 0000000" "asm"="mov {d}, {s}" "unit"="X,Y" "header"="port[s>>4]=s&7; rot[s>>4]=1 if (s>>3)&1" "asmonly" {
  ret void
}
