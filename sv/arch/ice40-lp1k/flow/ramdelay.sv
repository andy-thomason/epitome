// Board test (e): rx through a block RAM. Each clock writes a word made
// from rx into the next address and reads back the word written DEPTH - 1
// clocks before, so tx is rx delayed by DEPTH clocks (at most 2048, 171
// us: the Mac still sees its bytes echoed). The word alternates rx and
// ~rx, and tx is its bit 0 only while every bit is as written (else 0), so
// a wrong bit anywhere spoils the echo. WIDTH picks the RAM's mode: 16, 8,
// 4 or 2 (256x16 to 2048x2), or 1 (2048 words, mode 2048x2 with one bit).
`ifndef WIDTH
`define WIDTH 8
`endif
module ramdelay (input logic clk, input logic rx, output logic tx);
  localparam int W = `WIDTH;
  localparam int DEPTH = W >= 16 ? 256 : W == 8 ? 512 : W == 4 ? 1024 : 2048;
  localparam int A = $clog2(DEPTH);
  logic [W-1:0] mem [DEPTH];
  logic [A-1:0] wa = 0;
  logic [W-1:0] word, back, want;
  always_comb
    for (int i = 0; i < W; i++) word[i] = rx ^ i[0];
  always_ff @(posedge clk) begin
    mem[wa] <= word;
    back <= mem[wa + 1'b1];
    wa <= wa + 1'b1;
  end
  always_comb
    for (int i = 0; i < W; i++) want[i] = back[0] ^ i[0];
  assign tx = back == want ? back[0] : 1'b0;
endmodule
