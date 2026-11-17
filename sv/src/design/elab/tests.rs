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

#[test]
fn each_specialisation_has_its_own_subroutines() {
    // A constant function in a parameter's value is the one of the
    // specialisation being elaborated, reading that one's parameters.
    let t = ok("module c #(parameter int N = 8) ();
                  function automatic int half(input int s); half = s / 2 + N; endfunction
                  localparam int L = half(N);
                endmodule
                module top; c #(.N(8)) x(); c #(.N(16)) y(); endmodule");
    assert!(t.contains("L = 32'sd12") && t.contains("L = 32'sd24"), "{t}");
}

#[test]
fn drivers_of_elements_chosen_by_a_constant_array() {
    // Outputs connected to elements picked by a parameter array's elements
    // drive distinct parts (6.5); the same element twice is two drivers.
    let src = "module s (output logic [3:0] o); assign o = 4'h5; endmodule
               module top;
                 localparam int WHERE [4] = '{0, 2, 1, 3};
                 logic [3:0][3:0] v;
                 s a (.o(v[WHERE[2]]));
                 s b (.o(v[WHERE[J]]));
               endmodule";
    ok(&src.replace("WHERE[J]", "WHERE[3]"));
    assert!(err(&src.replace("WHERE[J]", "WHERE[0] + 1")).contains("two continuous drivers"));
}

#[test]
fn processes_writing_different_bits_of_an_element() {
    // Byte lanes of a memory word, each written by its own `always_ff`,
    // never share a bit; the same lane twice does.
    let src = "module mem (input logic clk, input logic [1:0] we, input logic [2:0] a, input logic [15:0] d);
                 logic [15:0] ram [8];
                 for (genvar p = 0; p < 2; p++) begin : g
                   always_ff @(posedge clk) if (we[p]) ram[a][LANE] <= d[p*8 +: 8];
                 end
               endmodule";
    ok(&src.replace("LANE", "p*8 +: 8"));
    assert!(err(&src.replace("LANE", "7:0")).contains("9.2.2"));

fn top_parameters_from_outside() {
    let text = "module a #(parameter int W = 1, parameter real R = 0.5, localparam L = W) (); logic [W-1:0] x; endmodule
                module b #(parameter int W = 2) (); logic [W-1:0] y; endmodule";
    let map = SourceMap::new();
    map.add(Source::new("t.sv", text));
    let out = preprocess(&map, &[0], &Options::default()).unwrap();
    let ast = crate::parse::parse(&out).unwrap();
    let v = |t: &str| super::parse_param_value(t).unwrap();
    // `W` alone sets it in both tops; `b.W` only in `b`, and later wins.
    let params = [("W".to_string(), v("6")), ("b.W".to_string(), v("4'd9")), ("R".to_string(), v("2.5")), ("L".to_string(), v("3"))];
    let d = super::elaborate_with(&ast, &out, &[], &params).unwrap();
    let t = crate::design::print::print(&d);
    assert!(t.contains("var x: logic[5:0]") && t.contains("R = 2.5, L = 32'sd6"), "{t}");
    assert!(t.contains("var y: logic[8:0]"), "{t}");
    assert_eq!(super::parse_param_value("-8'sh7f"), Some(crate::design::Const::Bits(crate::design::value::Bits::from_i64(8, true, -0x7f))));
    assert_eq!(super::parse_param_value("\"hi\""), Some(crate::design::Const::Str(b"hi".to_vec())));
    assert_eq!(super::parse_param_value("1e3"), Some(crate::design::Const::Real(1000.0)));
    assert_eq!(super::parse_param_value("abc"), None);
}
