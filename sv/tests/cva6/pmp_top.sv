// The PMP unit with the cv64a6_imafdc_sv39 configuration: 8 entries.
module pmp_top
  import ariane_pkg::*;
#(
    parameter config_pkg::cva6_cfg_t CVA6Cfg = build_config_pkg::build_config(cva6_config_pkg::cva6_cfg)
) (
    input logic [CVA6Cfg.PLEN-1:0] addr_i,
    input riscv::pmp_access_t access_type_i,
    input riscv::priv_lvl_t priv_lvl_i,
    input logic [CVA6Cfg.NrPMPEntries-1:0][CVA6Cfg.PLEN-3:0] conf_addr_i,
    input riscv::pmpcfg_t [CVA6Cfg.NrPMPEntries-1:0] conf_i,
    output logic allow_o
);
  pmp #(.CVA6Cfg(CVA6Cfg)) i_pmp (.*);
endmodule
