; Integers in a counted loop, a load and a store, and a conversion.
define amdgpu_kernel void @in(ptr addrspace(1) %o, i32 %a, i32 %b, i32 %n) {
entry:
  br label %loop
loop:
  %i = phi i32 [0, %entry], [%i1, %loop]
  %acc = phi i32 [%a, %entry], [%acc1, %loop]
  %x = mul i32 %acc, %b
  %y = xor i32 %x, %i
  %z = ashr i32 %y, 3
  %w = add i32 %z, 12345678
  %q = udiv i32 %w, 7
  %acc1 = sub i32 %q, %a
  %i1 = add i32 %i, 1
  %c = icmp slt i32 %i1, %n
  br i1 %c, label %loop, label %done
done:
  %p = getelementptr i32, ptr addrspace(1) %o, i32 %i
  store i32 %acc1, ptr addrspace(1) %p
  %v = load i32, ptr addrspace(1) %o
  %cv = sitofp i32 %v to float
  %pf = getelementptr float, ptr addrspace(1) %o, i32 1
  store float %cv, ptr addrspace(1) %pf
  ret void
}
