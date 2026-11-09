module decoder_top
  import slice_pkg::*;
(
    input logic [CVA6Cfg.VLEN-1:0] pc_i,
    input logic is_compressed_i,
    input logic [15:0] compressed_instr_i,
    input logic is_illegal_i,
    input logic [31:0] instruction_i,
    input branchpredict_sbe_t branch_predict_i,
    input exception_t ex_i,
    input logic [1:0] irq_i,
    input irq_ctrl_t irq_ctrl_i,
    input riscv::priv_lvl_t priv_lvl_i,
    input logic debug_req_i,
    input logic debug_mode_i,
    input riscv::xs_t fs_i,
    input logic [2:0] frm_i,
    input logic tvm_i,
    input logic tw_i,
    input logic tsr_i,
    output scoreboard_entry_t instruction_o,
    output logic [31:0] orig_instr_o,
    output logic is_control_flow_instr_o
);
  decoder #(
      .CVA6Cfg(CVA6Cfg), .branchpredict_sbe_t(branchpredict_sbe_t), .exception_t(exception_t),
      .irq_ctrl_t(irq_ctrl_t), .scoreboard_entry_t(scoreboard_entry_t), .interrupts_t(interrupts_t),
      .INTERRUPTS(INTERRUPTS)
  ) i_decoder (
      .debug_req_i, .pc_i, .is_compressed_i, .compressed_instr_i, .is_illegal_i, .instruction_i,
      .is_macro_instr_i(1'b0), .is_last_macro_instr_i(1'b0), .is_double_rd_macro_instr_i(1'b0),
      .is_zcmt_i(1'b0), .jump_address_i('0), .branch_predict_i, .ex_i, .irq_i, .irq_ctrl_i,
      .priv_lvl_i, .v_i(1'b0), .debug_mode_i, .fs_i, .vfs_i(riscv::Off), .frm_i, .vs_i(riscv::Off), .tvm_i, .tw_i,
      .vtw_i(1'b0), .tsr_i, .hu_i(1'b0), .mcbie_i(riscv::cbie_t'(0)), .scbie_i(riscv::cbie_t'(0)), .hcbie_i(riscv::cbie_t'(0)), .mcbcfe_i(1'b0),
      .scbcfe_i(1'b0), .hcbcfe_i(1'b0), .instruction_o, .orig_instr_o, .is_control_flow_instr_o,
      .sdtrig_decoder_action_i('0));
endmodule
