// A design for the VPI test application (`app.rs`): a little of each kind
// of object, a clock the application drives, and calls of the system task
// and function it registers.
`timescale 1ns/1ps

module adder #(parameter int N = 2) (input logic clk, input logic [7:0] d, output logic [7:0] q);
  always_ff @(posedge clk) q <= d + N;
endmodule

module top #(parameter int W = 8, parameter string NAME = "top", parameter real R = 1.5);
  logic clk = 0;
  logic [W-1:0] data = 0;
  wire [W-1:0] inv = ~data;
  logic [7:0] q;
  int edges = 0;
  real ratio = 0.25;
  string label = "hello";
  logic [7:0] mem [0:3];
  bit [3:0][1:0] pairs = 8'b11_10_01_00;
  logic [3:0] nib = 4'b10zx;

  adder #(.N(3)) u (.clk(clk), .d(data), .q(q));

  for (genvar i = 0; i < 2; i++) begin : g
    logic [3:0] v;
    assign v = 4'(i + 1);
  end

  always @(posedge clk) edges <= edges + 1;

  initial begin
    mem[0] = 8'h10;
    mem[1] = 8'h11;
    mem[2] = 8'h12;
    mem[3] = 8'h13;
    $app_show("start", data, W);
    #23 $display("design: %0t mem[1]=%h inv=%h q=%h twice(21)=%0d", $time, mem[1], inv, q, $app_twice(21));
  end
endmodule
