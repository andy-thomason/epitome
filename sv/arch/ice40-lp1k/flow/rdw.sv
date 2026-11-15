// Board test (h): a block RAM read of the word being written on the same
// clock edge. SystemVerilog's nonblocking assignments say the read sees
// the old word. The data written flips with t each clock, so the old word
// makes tx = rx (an echo) and the new word tx = ~rx. The iCE40's block RAM
// alone gives mostly the new word, and not reliably (the board showed),
// so the mapper delays the write and serves such a read from the delayed
// write's registers (Netlist::delay_writes).
`ifndef W
`define W 16
`endif
module rdw (input logic clk, input logic rx, output logic tx);
  localparam int W = `W;
  localparam int DEPTH = W >= 16 ? 256 : W == 8 ? 512 : W == 4 ? 1024 : 2048;
  logic [W-1:0] mem [DEPTH];
  logic t = 0;
  logic [W-1:0] q;
  always_ff @(posedge clk) begin
    mem[3] <= {W{rx ^ t}};
    q <= mem[3];
    t <= ~t;
  end
  assign tx = q[W-1] ^ t;
endmodule
