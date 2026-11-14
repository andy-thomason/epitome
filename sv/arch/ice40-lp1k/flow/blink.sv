// Board test (c): a 24-bit counter on the 12 MHz clock. Its top bit
// lights the LED (B6) and gates the echo: tx follows rx while it is 1
// and idles high while it is 0, so the Mac sees echoes come and go every
// 2^23 / 12 MHz = 0.70 s.
module blink (input logic clk, input logic rx, output logic tx, output logic led);
  logic [23:0] count = 0;
  always_ff @(posedge clk) count <= count + 1;
  assign led = count[23];
  assign tx = rx | ~count[23];
endmodule
