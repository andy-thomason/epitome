; Inline assembly in Intel's syntax (inteldialect), as Rust's asm! writes
; it on x86: operands in Intel's order, memory in brackets with a size,
; immediates bare, and modifiers for a register's narrower names. x86-64
; only (the AArch64 Mac skips it).
target triple = "x86_64-unknown-linux-gnu"

define i64 @lea3(i64 %a, i64 %b) {
  %r = call i64 asm inteldialect "lea $0, [$1 + 4*$2 + 8]", "=r,r,r"(i64 %a, i64 %b)
  ret i64 %r
}

define i64 @add_shift(i64 %a) {
  %r = call i64 asm inteldialect "mov $0, $1\0A\09add $0, 5\0A\09shl $0, 2", "=&r,r"(i64 %a)
  ret i64 %r
}

define i64 @load8(ptr %p) {
  %r = call i64 asm inteldialect "mov $0, qword ptr [$1 + 8]", "=r,r"(ptr %p)
  ret i64 %r
}

define i32 @zext_byte(ptr %p) {
  %r = call i32 asm inteldialect "movzx ${0:k}, byte ptr [$1]", "=r,r"(ptr %p)
  ret i32 %r
}

define i64 @mul3(i64 %a) {
  %r = call i64 asm inteldialect "imul $0, $1, -3", "=r,r"(i64 %a)
  ret i64 %r
}

define void @store_word(ptr %p, i16 %v) {
  call void asm sideeffect inteldialect "mov word ptr [$0 + 2], ${1:w}", "r,r,~{memory}"(ptr %p, i16 %v)
  ret void
}

; core::arch's cpuid: rbx (which LLVM may keep for itself) saved in another
; register around it. Leaf 0's ebx is the vendor's first four letters.
define i32 @vendor_ebx() {
  %r = call { i32, i64, i32, i32 } asm inteldialect "mov ${1:q}, rbx\0A\09cpuid\0A\09xchg ${1:q}, rbx", "={ax},=r,={cx},={dx},{ax},{cx}"(i32 0, i32 0)
  %b = extractvalue { i32, i64, i32, i32 } %r, 1
  %t = trunc i64 %b to i32
  ret i32 %t
}
