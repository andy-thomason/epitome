//! Extensions and truncations between 16 lanes of 32 bits and of 8 or 16
//! with AVX-512: one instruction each (vpmovzxbd, vpmovzxwd, vpmovdb,
//! vpmovdw), not a chain of halving steps with lanes moved one at a time.

use epitome_codegen::target::Target;
use epitome_codegen::{Options, compile};

#[test]
fn casts_of_16_lanes_are_one_instruction() {
    let ll = r#"
target triple = "x86_64-unknown-linux-gnu"
define void @rt(ptr %a, ptr %b, ptr %out8, ptr %out16) {
  %x = load <16 x i8>, ptr %a, align 16
  %y = load <16 x i16>, ptr %b, align 32
  %xz = zext <16 x i8> %x to <16 x i32>
  %yz = zext <16 x i16> %y to <16 x i32>
  %s = add <16 x i32> %xz, %yz
  %t8 = trunc <16 x i32> %s to <16 x i8>
  %t16 = trunc <16 x i32> %s to <16 x i16>
  store <16 x i8> %t8, ptr %out8, align 16
  store <16 x i16> %t16, ptr %out16, align 32
  ret void
}
"#;
    let t = Target::x86_64();
    let mut m = epitome_ir::parse_module(ll).unwrap();
    let opts = Options { mattr: vec!["+avx2".into(), "+avx512f".into(), "+avx512bw".into(), "+avx512vl".into()], ..Default::default() };
    let asm = compile(&t, &mut m, &opts).unwrap().asm;
    for i in ["vpmovzxbd", "vpmovzxwd", "vpmovdb", "vpmovdw"] {
        assert!(asm.contains(i), "no {i}:\n{asm}");
    }
    assert!(!asm.contains("(%rsp)"), "lanes through the stack:\n{asm}");
}
