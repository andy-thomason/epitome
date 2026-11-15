// CVA6's multiply/divide unit: the multiplier and the serial divider.
module mult_top
  import slice_pkg::*;
  import ariane_pkg::*;
#(
    localparam type fu_data_t = struct packed {
      fu_t                              fu;
      fu_op                             operation;
      logic [CVA6Cfg.XLEN-1:0]          operand_a;
      logic [CVA6Cfg.XLEN-1:0]          operand_b;
      logic [CVA6Cfg.XLEN-1:0]          imm;
      logic [CVA6Cfg.TRANS_ID_BITS-1:0] trans_id;
    }
) (
    input  logic clk_i,
    input  logic rst_ni,
    input  logic flush_i,
    input  fu_op operation_i,
    input  logic [CVA6Cfg.XLEN-1:0] a_i,
    input  logic [CVA6Cfg.XLEN-1:0] b_i,
    input  logic [CVA6Cfg.TRANS_ID_BITS-1:0] trans_id_i,
    input  logic mult_valid_i,
    output logic [CVA6Cfg.XLEN-1:0] result_o,
    output logic mult_valid_o,
    output logic mult_ready_o,
    output logic [CVA6Cfg.TRANS_ID_BITS-1:0] mult_trans_id_o
);
  fu_data_t d;
  assign d = '{fu: MULT, operation: operation_i, operand_a: a_i, operand_b: b_i, imm: '0, trans_id: trans_id_i};
  mult #(.CVA6Cfg(CVA6Cfg), .fu_data_t(fu_data_t)) i_mult (
      .clk_i, .rst_ni, .flush_i, .fu_data_i(d), .mult_valid_i, .result_o, .mult_valid_o, .mult_ready_o,
      .mult_trans_id_o);
endmodule
