// The ALU and the branch unit, connected as ex_stage connects them, with
// the types of cva6.sv.
module exec_top
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
    },
    localparam type branchpredict_sbe_t = struct packed {
      logic                    valid;
      cf_t                     cf;
      logic [CVA6Cfg.VLEN-1:0] predict_address;
    },
    localparam type exception_t = struct packed {
      logic [CVA6Cfg.XLEN-1:0] cause;
      logic [CVA6Cfg.XLEN-1:0] tval;
      logic [CVA6Cfg.GPLEN-1:0] tval2;
      logic [31:0] tinst;
      logic gva;
      logic valid;
      logic timing;
    },
    localparam type bp_resolve_t = struct packed {
      logic                    valid;
      logic [CVA6Cfg.VLEN-1:0] pc;
      logic [CVA6Cfg.VLEN-1:0] target_address;
      logic                    is_mispredict;
      logic                    is_taken;
      cf_t                     cf_type;
    }
) (
    input fu_op operation_i,
    input logic [CVA6Cfg.XLEN-1:0] a_i,
    input logic [CVA6Cfg.XLEN-1:0] b_i,
    input logic [CVA6Cfg.XLEN-1:0] imm_i,
    input logic [CVA6Cfg.VLEN-1:0] pc_i,
    input logic is_compressed_i,
    input logic branch_valid_i,
    input logic predict_valid_i,
    input cf_t predict_cf_i,
    input logic [CVA6Cfg.VLEN-1:0] predict_address_i,
    output logic [CVA6Cfg.XLEN-1:0] alu_result_o,
    output logic [CVA6Cfg.VLEN-1:0] branch_result_o,
    output bp_resolve_t resolved_branch_o,
    output logic resolve_branch_o,
    output exception_t branch_exception_o
);
  fu_data_t d;
  branchpredict_sbe_t bp;
  logic alu_branch_res;
  assign d = '{fu: ALU, operation: operation_i, operand_a: a_i, operand_b: b_i, imm: imm_i, trans_id: '0};
  assign bp = '{valid: predict_valid_i, cf: predict_cf_i, predict_address: predict_address_i};
  alu_wrapper #(.CVA6Cfg(CVA6Cfg), .fu_data_t(fu_data_t)) i_alu (
      .clk_i(1'b0), .rst_ni(1'b1), .alu_bypass_i('0), .fu_data_i(d), .result_o(alu_result_o),
      .alu_branch_res_o(alu_branch_res));
  branch_unit #(
      .CVA6Cfg(CVA6Cfg), .bp_resolve_t(bp_resolve_t), .branchpredict_sbe_t(branchpredict_sbe_t),
      .exception_t(exception_t), .fu_data_t(fu_data_t)
  ) i_branch (
      .clk_i(1'b0), .rst_ni(1'b1), .v_i(1'b0), .debug_mode_i(1'b0), .fu_data_i(d), .pc_i,
      .is_zcmt_i(1'b0), .is_compressed_instr_i(is_compressed_i), .branch_valid_i,
      .branch_comp_res_i(alu_branch_res), .branch_result_o, .branch_predict_i(bp),
      .resolved_branch_o, .resolve_branch_o, .branch_exception_o);
endmodule
