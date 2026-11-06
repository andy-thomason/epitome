; An if and an else, a break out of a loop, and many arguments, so that
; constants come from more than one cache line; and a store of a vector.
define amdgpu_kernel void @branches(ptr addrspace(1) %o, i32 %a0, i32 %a1, i32 %a2, i32 %a3, i32 %a4,
    i32 %a5, i32 %a6, i32 %a7, i32 %a8, i32 %a9, i32 %a10, i32 %a11, i32 %a12, i32 %a13, i32 %a14,
    i32 %a15, i32 %a16, i32 %a17, i32 %a18, i32 %a19, i32 %a20, i32 %a21, i32 %a22, i32 %a23,
    i32 %a24, i32 %a25, i32 %a26, i32 %a27, i32 %a28, i32 %a29, i32 %a30, i32 %a31, i32 %a32,
    i32 %a33, i32 %a34, i32 %a35, i32 %a36, i32 %a37, i32 %a38, i32 %a39, i32 %a40) {
entry:
  %c = icmp eq i32 %a0, %a40
  br i1 %c, label %then, label %else
then:
  %t = mul i32 %a1, %a39
  br label %loop
else:
  %e = sub i32 %a2, %a38
  br label %loop
loop:
  %i = phi i32 [0, %then], [0, %else], [%i1, %body]
  %acc = phi i32 [%t, %then], [%e, %else], [%acc1, %body]
  %i1 = add i32 %i, 1
  %stop = icmp sgt i32 %acc, %a37
  br i1 %stop, label %done, label %body
body:
  %acc1 = add i32 %acc, %a36
  %more = icmp ult i32 %i1, %a35
  br i1 %more, label %loop, label %done
done:
  %r = phi i32 [%acc, %loop], [%acc1, %body]
  %v0 = insertelement <4 x i32> undef, i32 %r, i32 0
  %v1 = insertelement <4 x i32> %v0, i32 %a34, i32 1
  %v2 = insertelement <4 x i32> %v1, i32 %a33, i32 2
  %v3 = insertelement <4 x i32> %v2, i32 %i1, i32 3
  %p = bitcast ptr addrspace(1) %o to ptr addrspace(1)
  store <4 x i32> %v3, ptr addrspace(1) %p
  ret void
}
