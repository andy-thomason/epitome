//! Functions marked `"epitome-opaque"` are for another tool (Epitome's
//! `docs/design/partial-machines.md`): the optimiser leaves their bodies as
//! they are, and keeps the calls to them, which it would otherwise inline
//! or work out.

use epitome_opt::pipeline::{self, Options};

#[test]
fn opaque_functions_are_left_alone() {
    let src = r#"
define internal i32 @kernel(i32 %a) "epitome-opaque" {
  %b = add i32 %a, 0
  %c = mul i32 %b, 1
  ret i32 %c
}
define i32 @start(i32 %x) {
  %r = call i32 @kernel(i32 3)
  %s = call i32 @kernel(i32 %x)
  %t = add i32 %r, %s
  ret i32 %t
}
"#;
    let mut m = epitome_ir::parse_module(src).unwrap();
    pipeline::run(&mut m, &Options::default());
    let out = m.to_string();
    // Its body, as written.
    for line in ["%b = add i32 %a, 0", "%c = mul i32 %b, 1", "ret i32 %c"] {
        assert!(out.contains(line), "no `{line}` in:\n{out}");
    }
    // Both calls, neither inlined nor worked out from the constant.
    assert!(out.contains("call i32 @kernel(i32 3)"), "{out}");
    assert!(out.contains("call i32 @kernel(i32 %x)"), "{out}");
}
