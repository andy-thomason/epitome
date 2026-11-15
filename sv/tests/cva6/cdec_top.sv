module cdec_top
  import slice_pkg::*;
(
    input  logic [31:0] instr_i,
    output logic [31:0] instr_o,
    output logic        illegal_instr_o,
    output logic        is_macro_instr_o,
    output logic        is_compressed_o,
    output logic        is_zcmt_instr_o
);
  compressed_decoder #(.CVA6Cfg(CVA6Cfg)) i_cdec (.*);
endmodule
