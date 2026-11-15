// pmp_entry with the cv64a6_imafdc_sv39 configuration (PLEN = 56).
module pmp_entry_top
  import cva6_config_pkg::*;
#(
    parameter config_pkg::cva6_cfg_t CVA6Cfg = build_config_pkg::build_config(cva6_config_pkg::cva6_cfg)
) (
    input logic [CVA6Cfg.PLEN-1:0] addr_i,
    input logic [CVA6Cfg.PLEN-3:0] conf_addr_i,
    input logic [CVA6Cfg.PLEN-3:0] conf_addr_prev_i,
    input riscv::pmp_addr_mode_t conf_addr_mode_i,
    output logic match_o
);
  pmp_entry #(.CVA6Cfg(CVA6Cfg)) i_entry (.*);
endmodule
