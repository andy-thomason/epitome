// A UART line echo through block RAM. Bytes from the Mac (rx, 115200 baud
// at 12 MHz) are stored in a 512x8 memory; a carriage return sends the
// line back reversed, then CR LF. Line feeds are dropped, as are bytes
// that arrive while a line is being sent. The LED is on while sending.
module echo_ram (input logic clk, input logic rx, output logic tx, output logic led);
  localparam int BIT = 104;  // clocks a bit: 12 MHz / 115200, rounded
  localparam int HALF = 52;

  // Receiver: wait for a start bit, sample the middle of each bit.
  logic [1:0] sync = 2'b11;
  always_ff @(posedge clk) sync <= {sync[0], rx};
  wire r = sync[1];
  logic rbusy = 0;
  logic [6:0] rdiv = 0;
  logic [3:0] rbit = 0;
  logic [7:0] rbyte = 0;
  logic got = 0;
  always_ff @(posedge clk) begin
    got <= 0;
    if (!rbusy) begin
      if (!r) begin
        rbusy <= 1;
        rdiv <= HALF;
        rbit <= 0;
      end
    end else if (rdiv != BIT - 1) rdiv <= rdiv + 1;
    else begin
      rdiv <= 0;
      if (rbit == 0) begin
        if (r) rbusy <= 0;  // a glitch, not a start bit
        else rbit <= 1;
      end else if (rbit != 9) begin
        rbyte <= {r, rbyte[7:1]};
        rbit <= rbit + 1;
      end else begin
        rbusy <= 0;
        got <= r;  // a byte, if its stop bit is high
      end
    end
  end

  // The line.
  logic [7:0] line [512];
  logic [8:0] wptr = 0, rptr = 0;
  logic sending = 0;
  wire store = got && !sending && rbyte != 8'h0d && rbyte != 8'h0a && wptr != 511;
  always_ff @(posedge clk) if (store) line[wptr] <= rbyte;
  logic [7:0] q;
  always_ff @(posedge clk) q <= line[rptr];

  // Transmitter: the line from its end, then CR LF.
  logic [6:0] tdiv = 0;
  logic [3:0] left = 0;
  logic [9:0] frame = 0;
  logic [1:0] tail = 0;
  always_ff @(posedge clk) begin
    if (got && !sending) begin
      if (rbyte == 8'h0d) begin
        sending <= 1;
        tdiv <= 0;
        wptr <= 0;
        if (wptr == 0) tail <= 1;
        else begin
          tail <= 0;
          rptr <= wptr - 1;
        end
      end else if (store) wptr <= wptr + 1;
    end else if (sending) begin
      if (tdiv != BIT - 1) tdiv <= tdiv + 1;
      else begin
        tdiv <= 0;
        if (left == 0) begin
          case (tail)
            0: begin
              frame <= {1'b1, q, 1'b0};
              left <= 10;
              if (rptr == 0) tail <= 1;
              else rptr <= rptr - 1;
            end
            1: begin
              frame <= {1'b1, 8'h0d, 1'b0};
              left <= 10;
              tail <= 2;
            end
            2: begin
              frame <= {1'b1, 8'h0a, 1'b0};
              left <= 10;
              tail <= 3;
            end
            default: sending <= 0;
          endcase
        end else begin
          frame <= {1'b1, frame[9:1]};
          left <= left - 1;
        end
      end
    end
  end
  assign tx = left == 0 ? 1'b1 : frame[0];
  assign led = sending;
endmodule
