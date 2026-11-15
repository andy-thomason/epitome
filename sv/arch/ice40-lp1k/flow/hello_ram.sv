// Board test (f): hello.sv with its string in a block RAM, given as the
// RAM's initial contents and read a character at a time into a register
// (the RAM's own output register).
module hello_ram (input logic clk, output logic tx);
  localparam int LEN = 20;
  localparam logic [8*LEN-1:0] TEXT = {"Hello from Epitome", 8'h0d, 8'h0a};
  logic [7:0] text [32];
  initial for (int i = 0; i < LEN; i++) text[i] = TEXT[8*(LEN-1-i) +: 8];
  logic [6:0] div = 0;
  logic [3:0] left = 0;
  logic [4:0] idx = 0;
  logic [9:0] frame = 0;
  logic [7:0] ch;
  always_ff @(posedge clk) ch <= text[idx];
  always_ff @(posedge clk)
    if (div != 103) div <= div + 1;
    else begin
      div <= 0;
      if (left == 0) begin
        frame <= {1'b1, ch, 1'b0};
        left <= 10;
        idx <= idx == LEN - 1 ? 0 : idx + 1;
      end else begin
        frame <= {1'b1, frame[9:1]};
        left <= left - 1;
      end
    end
  assign tx = left == 0 ? 1'b1 : frame[0];
endmodule
