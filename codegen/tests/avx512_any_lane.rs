//! Whether any lane of a mask is set, `icmp ne (bitcast <16 x i1> %m to
//! i16), 0`, with AVX-512: the mask is in a k register, so its bits are the
//! integer (kmovw and a test, before a branch), not the lanes' maximum
//! worked out a lane at a time through the stack.

use epitome_codegen::target::Target;
use epitome_codegen::{Options, compile};

#[test]
fn any_lane_of_a_mask_register() {
    let ll = r#"
target triple = "x86_64-unknown-linux-gnu"
define void @any(ptr %a, ptr %p, ptr %out) {
entry:
  %x = load <16 x i32>, ptr %a, align 64
  %h = load i16, ptr %p, align 2
  %hm = bitcast i16 %h to <16 x i1>
  %c = icmp ne <16 x i32> %x, zeroinitializer
  %m = and <16 x i1> %c, %hm
  %b = bitcast <16 x i1> %m to i16
  %g = icmp ne i16 %b, 0
  br i1 %g, label %some, label %none
some:
  %y = mul <16 x i32> %x, %x
  br label %none
none:
  %r = phi <16 x i32> [ %y, %some ], [ %x, %entry ]
  store <16 x i32> %r, ptr %out, align 64
  ret void
}
"#;
    let t = Target::x86_64();
    let mut m = epitome_ir::parse_module(ll).unwrap();
    let opts = Options {
        mattr: ["+avx2", "+avx512f", "+avx512bw", "+avx512dq", "+avx512vl"].iter().map(|s| s.to_string()).collect(),
        ..Default::default()
    };
    let asm = compile(&t, &mut m, &opts).unwrap().asm;
    assert!(asm.contains("kmovw"), "the mask's bits not taken from its k register:\n{asm}");
    assert!(!asm.contains("(%rsp)") && !asm.contains("cmov"), "the lanes' maximum, a lane at a time:\n{asm}");
}
