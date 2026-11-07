module serdiv_top
  import slice_pkg::*;
(
    input  logic                    clk_i,
    input  logic                    rst_ni,
    input  logic [CVA6Cfg.TRANS_ID_BITS-1:0] id_i,
    input  logic [CVA6Cfg.XLEN-1:0] op_a_i,
    input  logic [CVA6Cfg.XLEN-1:0] op_b_i,
    input  logic [1:0]              opcode_i,
    input  logic                    in_vld_i,
    output logic                    in_rdy_o,
    input  logic                    flush_i,
    output logic                    out_vld_o,
    input  logic                    out_rdy_i,
    output logic [CVA6Cfg.TRANS_ID_BITS-1:0] id_o,
    output logic [CVA6Cfg.XLEN-1:0] res_o
);
  serdiv #(.CVA6Cfg(CVA6Cfg), .WIDTH(CVA6Cfg.XLEN)) i_div (.*);
endmodule
