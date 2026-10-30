use crate::preprocess::{Options, preprocess};
use crate::source::{Source, SourceMap};

/// Elaborates `text` and returns the design as `--design` prints it, or
/// the first line of the error.
fn run(text: &str) -> Result<String, String> {
    let map = SourceMap::new();
    map.add(Source::new("t.sv", text));
    let out = preprocess(&map, &[0], &Options::default()).map_err(|e| map.render(&e))?;
    let ast = crate::parse::parse(&out).map_err(|e| map.render(&e))?;
    let d = super::elaborate(&ast, &out).map_err(|e| map.render(&e).lines().next().unwrap().to_string())?;
    Ok(crate::design::print::print(&d))
}

fn ok(text: &str) -> String {
    run(text).unwrap_or_else(|e| panic!("{e}"))
}

fn err(text: &str) -> String {
    match run(text) {
        Ok(t) => panic!("expected an error, got\n{t}"),
        Err(e) => e,
    }
}

#[test]
fn widths_are_explicit() {
    // `q + 1` is 32 bits, unsigned (11.6, 11.8); the result is truncated.
    let t = ok("module m; logic [3:0] q; always_comb q = q + 1; endmodule");
    assert!(t.contains("q = trunc<4>((zext<32>(q) + cvt<logic[31:0]>(32'sd1)));"), "{t}");
    // Signed operands extend by sign; `'1` fills its context.
    let t = ok("module m; logic signed [3:0] a; logic signed [7:0] b; initial begin b = a; b = '1; end endmodule");
    assert!(t.contains("b = sext<8>(a);"), "{t}");
    assert!(t.contains("b = -8'sd1;"), "{t}");
}

#[test]
fn specialisations_are_shared() {
    let t = ok("module c #(parameter W = 1) (input [W-1:0] a); endmodule
                module top; c #(4) x(), y(); c #(8) z(); endmodule");
    assert!(t.contains("module c#1 #(W = 32'sd4) {  // 2 instances"), "{t}");
    assert!(t.contains("module c#2 #(W = 32'sd8) {  // 1 instance"), "{t}");
}

#[test]
fn generate_and_constant_functions() {
    let t = ok("module m;
                  function automatic int log2(int n); int b = 0; while ((1 << b) < n) b++; return b; endfunction
                  localparam int W = log2(10);
                  for (genvar i = 0; i < 2; i++) begin : g logic [W-1:0] v; end
                  if (W == 4) begin : yes end else begin : no end
                endmodule");
    assert!(t.contains("localparam W: int = 32'sd4"), "{t}");
    assert!(t.contains("generate g[0] {") && t.contains("generate g[1] {"), "{t}");
    assert!(t.contains("generate yes {") && !t.contains("generate no {"), "{t}");
}

#[test]
fn processes_record_reads_and_writes() {
    let t = ok("module m; logic a, b, c; always_comb begin c = a & b; end endmodule");
    assert!(t.contains("always_comb reads(a, b) writes(c) sensitive(a, b)"), "{t}");
}

#[test]
fn rules_are_checked() {
    assert!(err("module m; logic v; assign v = 1; initial v = 0; endmodule").contains("written by a procedure and by a continuous driver (6.5)"));
    assert!(err("module m; typedef enum bit [2:0] {A = 9} e_t; endmodule").contains("does not fit"));
    assert!(err("module m; initial begin int x = 1; end endmodule").contains("(6.21)"));
    assert!(err("module m; initial early = 1; int early; endmodule").contains("used before its declaration"));
    assert!(err("module m #(parameter int N); endmodule module top; m u(); endmodule").contains("has no default"));
}
