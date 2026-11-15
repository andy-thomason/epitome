// Board tests (e) and (g): rx through block RAM. Each clock writes a word
// made from rx into the next address and reads back the word written
// DEPTH - 1 clocks before, so tx is rx delayed by DEPTH clocks (16384 are
// 1.4 ms: the Mac still sees its bytes echoed). The word alternates rx and
// ~rx, and tx is its bit 0 only while every bit is as written (else 0), so
// a wrong bit anywhere spoils the echo. WIDTH picks the RAM's mode: 16, 8,
// 4 or 2 (256x16 to 2048x2), or 1 (2048 words, mode 2048x2 with one bit).
`ifndef WIDTH
`define WIDTH 8
`endif
// DEPTH, if given, overrides the depth: deeper than a block (2048 words at
// most) the memory is split into banks, up to the whole chip's 16 blocks
// (16384 words of 4 bits).
`ifdef DEPTH
`define RAM_DEPTH `DEPTH
`else
`define RAM_DEPTH (W >= 16 ? 256 : W == 8 ? 512 : W == 4 ? 1024 : 2048)
`endif
module ramdelay (input logic clk, input logic rx, output logic tx);
  localparam int W = `WIDTH;
  localparam int DEPTH = `RAM_DEPTH;
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
