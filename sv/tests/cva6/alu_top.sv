// CVA6's ALU, its fu_data_t as cva6.sv declares it.
module alu_top
  import ariane_pkg::*;
#(
    parameter config_pkg::cva6_cfg_t CVA6Cfg = build_config_pkg::build_config(cva6_config_pkg::cva6_cfg),
    localparam type fu_data_t = struct packed {
      fu_t                              fu;
      fu_op                             operation;
      logic [CVA6Cfg.XLEN-1:0]          operand_a;
      logic [CVA6Cfg.XLEN-1:0]          operand_b;
      logic [CVA6Cfg.XLEN-1:0]          imm;
      logic [CVA6Cfg.TRANS_ID_BITS-1:0] trans_id;
    }
) (
    input fu_op operation_i,
    input logic [CVA6Cfg.XLEN-1:0] a_i,
    input logic [CVA6Cfg.XLEN-1:0] b_i,
    output logic [CVA6Cfg.XLEN-1:0] result_o,
    output logic branch_o
);
  fu_data_t d;
  assign d.fu = ALU;
  assign d.operation = operation_i;
  assign d.operand_a = a_i;
  assign d.operand_b = b_i;
  assign d.imm = '0;
  assign d.trans_id = '0;
  alu #(.CVA6Cfg(CVA6Cfg), .fu_data_t(fu_data_t)) i_alu (
      .clk_i(1'b0), .rst_ni(1'b1), .fu_data_i(d), .fu_data_cpop_i(d), .result_o, .alu_branch_res_o(branch_o));
endmodule
