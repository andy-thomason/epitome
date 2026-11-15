// A model of the iCE40's block RAM primitive, SB_RAM40_4K, as icebox_vlog
// prints it for a configured RAM: for simulating that Verilog against a
// design's source (sv/tests/ice40_flow.rs). Ports and parameters are as
// Lattice documents the primitive; the data pins each mode uses and where
// their bits sit in the 256 words of 16 bits are what the board showed
// (sv/src/arch/ice40/board.rs, ram_mode_bit):
//   mode 0 (256x16): all 16 pins, MASK bits set to keep bits from a write;
//   mode m (1 512x8, 2 1024x4, 3 2048x2): word ADDR[7:0], and pin p with
//   p % 2^m = 2^(m-1) - 1 holding bit (p - p % 2^m) + ADDR[10:8].
// INIT_k holds words 16k to 16k + 15, word w at bits 16(w % 16) up. RDATA
// starts at 0, as Epitome's cell simulation has it.
module SB_RAM40_4K #(
  parameter int READ_MODE = 0,
  parameter int WRITE_MODE = 0,
  parameter logic [255:0] INIT_0 = 0, INIT_1 = 0, INIT_2 = 0, INIT_3 = 0,
  parameter logic [255:0] INIT_4 = 0, INIT_5 = 0, INIT_6 = 0, INIT_7 = 0,
  parameter logic [255:0] INIT_8 = 0, INIT_9 = 0, INIT_A = 0, INIT_B = 0,
  parameter logic [255:0] INIT_C = 0, INIT_D = 0, INIT_E = 0, INIT_F = 0
) (
  output logic [15:0] RDATA,
  input logic [10:0] RADDR,
  input logic [10:0] WADDR,
  input logic [15:0] MASK,
  input logic [15:0] WDATA,
  input logic RCLKE, RCLK, RE,
  input logic WCLKE, WCLK, WE
);
  localparam logic [4095:0] INIT = {INIT_F, INIT_E, INIT_D, INIT_C, INIT_B, INIT_A, INIT_9, INIT_8,
                                    INIT_7, INIT_6, INIT_5, INIT_4, INIT_3, INIT_2, INIT_1, INIT_0};
  logic [15:0] mem [256];
  initial for (int w = 0; w < 256; w++) mem[w] = INIT[16*w +: 16];
  initial RDATA = 0;

  // In mode m, pin p carries data if m is 0 or p % 2^m = 2^(m-1) - 1, for
  // bit p of the word, or bit p - p % 2^m + ADDR[10:8].
  localparam int RM = 1 << READ_MODE, WM = 1 << WRITE_MODE;

  always @(posedge RCLK)
    if (RCLKE && RE)
      for (int p = 0; p < 16; p++)
        RDATA[p] <= RM == 1 ? mem[RADDR[7:0]][p]
                  : p % RM == RM / 2 - 1 ? mem[RADDR[7:0]][p - p % RM + (RADDR[10:8] & (RM - 1))] : 1'b0;

  always @(posedge WCLK)
    if (WCLKE && WE)
      for (int p = 0; p < 16; p++)
        if (WM == 1 ? !MASK[p] : p % WM == WM / 2 - 1)
          mem[WADDR[7:0]][WM == 1 ? p : p - p % WM + (WADDR[10:8] & (WM - 1))] <= WDATA[p];
endmodule
