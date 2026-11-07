// The types cva6.sv declares, for the cv64a6_imafdc_sv39 configuration,
// in a package that the tests' wrappers import.
package slice_pkg;
  import ariane_pkg::*;
  localparam config_pkg::cva6_cfg_t CVA6Cfg = build_config_pkg::build_config(cva6_config_pkg::cva6_cfg);
  typedef struct packed {
    logic [CVA6Cfg.XLEN-1:0] cause;
    logic [CVA6Cfg.XLEN-1:0] tval;
    logic [CVA6Cfg.GPLEN-1:0] tval2;
    logic [31:0] tinst;
    logic gva;
    logic valid;
    logic timing;
  } exception_t;
  typedef struct packed {
    logic                    valid;
    cf_t                     cf;
    logic [CVA6Cfg.VLEN-1:0] predict_address;
  } branchpredict_sbe_t;
  typedef struct packed {
    logic [CVA6Cfg.VLEN-1:0] address;
    logic [31:0] instruction;
    branchpredict_sbe_t branch_predict;
    exception_t ex;
  } fetch_entry_t;
  typedef struct packed {
    logic [CVA6Cfg.XLEN-7:0] base;
    logic [5:0] mode;
  } jvt_t;
  typedef struct packed {
    logic [CVA6Cfg.VLEN-1:0] pc;
    logic [CVA6Cfg.TRANS_ID_BITS-1:0] trans_id;
    fu_t fu;
    fu_op op;
    logic [REG_ADDR_SIZE-1:0] rs1;
    logic [REG_ADDR_SIZE-1:0] rs2;
    logic [REG_ADDR_SIZE-1:0] rd;
    logic [CVA6Cfg.XLEN-1:0] result;
    logic valid;
    logic use_imm;
    logic use_zimm;
    logic use_pc;
    exception_t ex;
    branchpredict_sbe_t bp;
    logic is_compressed;
    logic is_macro_instr;
    logic is_last_macro_instr;
    logic is_double_rd_macro_instr;
    logic vfp;
    logic is_zcmt;
  } scoreboard_entry_t;
  typedef struct packed {
    logic [CVA6Cfg.XLEN-1:0] mie;
    logic [CVA6Cfg.XLEN-1:0] mip;
    logic [CVA6Cfg.XLEN-1:0] mideleg;
    logic [CVA6Cfg.XLEN-1:0] hideleg;
    logic                    sie;
    logic                    global_enable;
  } irq_ctrl_t;
  typedef struct packed {
    logic [CVA6Cfg.XLEN-1:0] S_SW;
    logic [CVA6Cfg.XLEN-1:0] VS_SW;
    logic [CVA6Cfg.XLEN-1:0] M_SW;
    logic [CVA6Cfg.XLEN-1:0] S_TIMER;
    logic [CVA6Cfg.XLEN-1:0] VS_TIMER;
    logic [CVA6Cfg.XLEN-1:0] M_TIMER;
    logic [CVA6Cfg.XLEN-1:0] S_EXT;
    logic [CVA6Cfg.XLEN-1:0] VS_EXT;
    logic [CVA6Cfg.XLEN-1:0] M_EXT;
    logic [CVA6Cfg.XLEN-1:0] HS_EXT;
  } interrupts_t;
  localparam interrupts_t INTERRUPTS = '{
      S_SW: (CVA6Cfg.XLEN'(1) << (CVA6Cfg.XLEN - 1)) | CVA6Cfg.XLEN'(riscv::IRQ_S_SOFT),
      VS_SW: (CVA6Cfg.XLEN'(1) << (CVA6Cfg.XLEN - 1)) | CVA6Cfg.XLEN'(riscv::IRQ_VS_SOFT),
      M_SW: (CVA6Cfg.XLEN'(1) << (CVA6Cfg.XLEN - 1)) | CVA6Cfg.XLEN'(riscv::IRQ_M_SOFT),
      S_TIMER: (CVA6Cfg.XLEN'(1) << (CVA6Cfg.XLEN - 1)) | CVA6Cfg.XLEN'(riscv::IRQ_S_TIMER),
      VS_TIMER: (CVA6Cfg.XLEN'(1) << (CVA6Cfg.XLEN - 1)) | CVA6Cfg.XLEN'(riscv::IRQ_VS_TIMER),
      M_TIMER: (CVA6Cfg.XLEN'(1) << (CVA6Cfg.XLEN - 1)) | CVA6Cfg.XLEN'(riscv::IRQ_M_TIMER),
      S_EXT: (CVA6Cfg.XLEN'(1) << (CVA6Cfg.XLEN - 1)) | CVA6Cfg.XLEN'(riscv::IRQ_S_EXT),
      VS_EXT: (CVA6Cfg.XLEN'(1) << (CVA6Cfg.XLEN - 1)) | CVA6Cfg.XLEN'(riscv::IRQ_VS_EXT),
      M_EXT: (CVA6Cfg.XLEN'(1) << (CVA6Cfg.XLEN - 1)) | CVA6Cfg.XLEN'(riscv::IRQ_M_EXT),
      HS_EXT: (CVA6Cfg.XLEN'(1) << (CVA6Cfg.XLEN - 1)) | CVA6Cfg.XLEN'(riscv::IRQ_HS_EXT)
  };
  typedef logic [CVA6Cfg.XLEN-1:0] hartid_t;
  typedef struct packed {
    logic [15:0] instr;
    hartid_t hartid;
  } x_compressed_req_t;
  typedef struct packed {
    logic [31:0] instr;
    logic accept;
  } x_compressed_resp_t;
endpackage
