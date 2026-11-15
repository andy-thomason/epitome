// CVA6's issue/decode stage: compressed decoder, CV-X-IF compressed
// interface driver, decoder and the issue register.
module id_top
  import slice_pkg::*;
(
    input logic clk_i,
    input logic rst_ni,
    input logic flush_i,
    input logic debug_req_i,
    input fetch_entry_t fetch_entry_i,
    input logic fetch_entry_valid_i,
    output logic fetch_entry_ready_o,
    output scoreboard_entry_t issue_entry_o,
    output logic [31:0] orig_instr_o,
    output logic issue_entry_valid_o,
    output logic is_ctrl_flow_o,
    input logic issue_instr_ack_i,
    output logic rvfi_is_compressed_o,
    input riscv::priv_lvl_t priv_lvl_i,
    input riscv::xs_t fs_i,
    input logic [2:0] frm_i,
    input logic [1:0] irq_i,
    input irq_ctrl_t irq_ctrl_i,
    input logic debug_mode_i,
    input logic tvm_i,
    input logic tw_i,
    input logic tsr_i,
    input logic [CVA6Cfg.XLEN-1:0] hart_id_i,
    input logic compressed_ready_i,
    input x_compressed_resp_t compressed_resp_i,
    output logic compressed_valid_o,
    output x_compressed_req_t compressed_req_o
);
  logic [CVA6Cfg.XLEN-1:0] sdtrig[CVA6Cfg.NrIssuePorts];
  assign sdtrig[0] = '0;
  scoreboard_entry_t issue_entry_prev;
  id_stage #(
      .CVA6Cfg(CVA6Cfg), .branchpredict_sbe_t(branchpredict_sbe_t), .exception_t(exception_t),
      .fetch_entry_t(fetch_entry_t), .jvt_t(jvt_t), .irq_ctrl_t(irq_ctrl_t),
      .scoreboard_entry_t(scoreboard_entry_t), .interrupts_t(interrupts_t), .INTERRUPTS(INTERRUPTS),
      .x_compressed_req_t(x_compressed_req_t), .x_compressed_resp_t(x_compressed_resp_t)
  ) i_id (
      .clk_i, .rst_ni, .flush_i, .debug_req_i, .fetch_entry_i, .fetch_entry_valid_i, .fetch_entry_ready_o,
      .issue_entry_o, .issue_entry_o_prev(issue_entry_prev), .orig_instr_o, .issue_entry_valid_o,
      .is_ctrl_flow_o, .issue_instr_ack_i, .rvfi_is_compressed_o, .priv_lvl_i, .v_i(1'b0), .fs_i,
      .vfs_i(riscv::Off), .frm_i, .vs_i(riscv::Off), .irq_i, .irq_ctrl_i, .debug_mode_i, .tvm_i, .tw_i,
      .vtw_i(1'b0), .tsr_i, .hu_i(1'b0), .mcbie_i(riscv::cbie_t'(0)), .scbie_i(riscv::cbie_t'(0)),
      .hcbie_i(riscv::cbie_t'(0)), .mcbcfe_i(1'b0), .scbcfe_i(1'b0), .hcbcfe_i(1'b0), .hart_id_i,
      .compressed_ready_i, .jvt_i('0), .compressed_resp_i, .compressed_valid_o, .compressed_req_o,
      .sdtrig_decoder_action_i(sdtrig), .dcache_req_ports_i('0), .dcache_req_ports_o());
endmodule
