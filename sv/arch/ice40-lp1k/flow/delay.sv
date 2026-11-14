// Board test (b): rx delayed through a shift register of flip-flops on
// the 12 MHz clock (pin D1) before going back out on tx. At 115200 baud
// a bit lasts 104 clocks, so a delay of a few clocks still echoes.
module delay #(parameter int n = 4) (input logic clk, input logic rx, output logic tx);
  logic [n-1:0] sr = 0;
  always_ff @(posedge clk) sr <= {sr[n-2:0], rx};
  assign tx = sr[n-1];
endmodule
