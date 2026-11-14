// Board test (d): a UART transmitter. It sends "Hello from Epitome\r\n"
// over and over on tx (pin B3) at 115200 baud from the 12 MHz clock
// (104 clocks a bit): a start bit, eight data bits least significant
// first, a stop bit and a bit of idle for each character.
module hello (input logic clk, output logic tx);
  localparam int LEN = 20;
  logic [6:0] div = 0;    // clocks into this bit
  logic [3:0] left = 0;   // bits of the frame still to send
  logic [4:0] idx = 0;    // the next character
  logic [9:0] frame = 0;  // shifted out from bit 0
  logic [7:0] ch;
  always_comb
    case (idx)
      5'd0: ch = "H";
      5'd1: ch = "e";
      5'd2: ch = "l";
      5'd3: ch = "l";
      5'd4: ch = "o";
      5'd5: ch = " ";
      5'd6: ch = "f";
      5'd7: ch = "r";
      5'd8: ch = "o";
      5'd9: ch = "m";
      5'd10: ch = " ";
      5'd11: ch = "E";
      5'd12: ch = "p";
      5'd13: ch = "i";
      5'd14: ch = "t";
      5'd15: ch = "o";
      5'd16: ch = "m";
      5'd17: ch = "e";
      5'd18: ch = 8'h0d;  // carriage return: SystemVerilog strings have no \r
      default: ch = "\n";
    endcase
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
