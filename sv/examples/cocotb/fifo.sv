// A synchronous FIFO of DEPTH words of WIDTH bits, for the cocotb example
// (`run.py`). DEPTH is a power of two, at least 2.
module fifo #(parameter int DEPTH = 4, parameter int WIDTH = 8) (
  input  logic                   clk,
  input  logic                   rst_n,
  input  logic                   push,
  input  logic [WIDTH-1:0]       din,
  input  logic                   pop,
  output logic [WIDTH-1:0]       dout,
  output logic                   empty,
  output logic                   full,
  output logic [$clog2(DEPTH):0] count
);
  localparam int AW = $clog2(DEPTH);

  logic [WIDTH-1:0] mem [DEPTH];
  logic [AW-1:0] rd, wr;
  wire do_push = push && !full;
  wire do_pop = pop && !empty;

  assign empty = count == 0;
  assign full = count == DEPTH;
  assign dout = mem[rd];

  always_ff @(posedge clk or negedge rst_n) begin
    if (!rst_n) begin
      rd <= '0;
      wr <= '0;
      count <= '0;
    end else begin
      if (do_push) begin
        mem[wr] <= din;
        wr <= wr + 1'b1;
      end
      if (do_pop)
        rd <= rd + 1'b1;
      count <= count + do_push - do_pop;
    end
  end

  // Whether each entry holds a word.
  for (genvar i = 0; i < DEPTH; i++) begin : slot
    wire [AW-1:0] offset = AW'(i) - rd;
    wire used = {1'b0, offset} < count;
  end
endmodule
