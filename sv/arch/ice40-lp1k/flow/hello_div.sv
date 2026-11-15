// Board test (i): a clock the design makes. A counter divides the 12 MHz
// clock by 8; its top bit clocks a transmitter (hello.sv's, at 13 of
// those 1.5 MHz clocks a bit: 115385 baud, 0.16% fast) sending
// "Hello from Epitome\r\n". The divided clock reaches the transmitter's
// tiles through the fabric.
module hello_div (input logic clk, output logic tx);
  logic [2:0] pre = 0;
  always_ff @(posedge clk) pre <= pre + 1;
  logic slow;
  assign slow = pre[2];
  localparam int LEN = 20;
  localparam logic [8*LEN-1:0] TEXT = {"Hello from Epitome", 8'h0d, 8'h0a};
  logic [3:0] div = 0;
  logic [3:0] left = 0;
  logic [4:0] idx = 0;
  logic [9:0] frame = 0;
  always_ff @(posedge slow)
    if (div != 12) div <= div + 1;
    else begin
      div <= 0;
      if (left == 0) begin
        frame <= {1'b1, TEXT[8*(LEN-1-idx) +: 8], 1'b0};
        left <= 10;
        idx <= idx == LEN - 1 ? 0 : idx + 1;
      end else begin
        frame <= {1'b1, frame[9:1]};
        left <= left - 1;
      end
    end
  assign tx = left == 0 ? 1'b1 : frame[0];
endmodule
