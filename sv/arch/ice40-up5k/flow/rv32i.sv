// A small RV32I computer for the iCESugar (iCE40UP5K): a processor for the
// RISC-V base integer instructions, 8 KB of memory holding its program
// (rv32i_demo.s, assembled into rv32i_demo.svh) and data, and a UART at
// 115200 baud from the 12 MHz clock. The memory map is in
// sv/src/arch/ice40/rv32i.rs, whose emulator says what the board sends.
//
// The processor takes four clocks an instruction (five for a load):
//   fetch:   the instruction is read from memory.
//   decode:  the instruction has been read; the registers it names are
//            read from the register file.
//   execute: the ALU, the branch, the store's write, and a load's read.
//   load:    the loaded word is aligned and written to its register.
// Memory and the register file are block RAM: each read gives its word a
// clock later. The register file is read twice an instruction, so it is
// two RAMs of the same contents.
module rv32i #(parameter int BIT = 104) (input logic clk, input logic rx, output logic tx, output logic led);
  localparam int WORDS = 2048;
  logic [31:0] mem [WORDS] =
`include "rv32i_demo.svh"
  ;
  logic [31:0] regs [32] = '{default: 32'h0};

  localparam logic [1:0] FETCH = 0, DECODE = 1, EXECUTE = 2, LOAD = 3;
  logic [1:0] state = FETCH;
  logic [31:0] pc = 0;
  logic [31:0] ir = 0;
  logic [31:0] q;       // the memory's read
  logic [31:0] a, b;    // the register file's reads: rs1 and rs2
  logic [31:0] ea = 0;  // a load's address, for its alignment

  // ---- decode ----
  wire [6:0] op = ir[6:0];
  wire [4:0] rd = ir[11:7];
  wire [2:0] f3 = ir[14:12];
  wire alt = ir[30];
  wire [31:0] imm_i = {{20{ir[31]}}, ir[31:20]};
  wire [31:0] imm_s = {{20{ir[31]}}, ir[31:25], ir[11:7]};
  wire [31:0] imm_b = {{19{ir[31]}}, ir[31], ir[7], ir[30:25], ir[11:8], 1'b0};
  wire [31:0] imm_u = {ir[31:12], 12'b0};
  wire [31:0] imm_j = {{11{ir[31]}}, ir[31], ir[19:12], ir[20], ir[30:21], 1'b0};
  localparam logic [6:0] OP_LUI = 7'h37, OP_AUIPC = 7'h17, OP_JAL = 7'h6f, OP_JALR = 7'h67,
    OP_BRANCH = 7'h63, OP_LOAD = 7'h03, OP_STORE = 7'h23, OP_IMM = 7'h13, OP_REG = 7'h33;

  // ---- the ALU ----
  // Its second operand is rs2 for register operations and branches, else
  // an immediate (a load's, a store's or jalr's address offset).
  wire [31:0] y = op == OP_REG || op == OP_BRANCH ? b : op == OP_STORE ? imm_s : imm_i;
  wire sub = op == OP_REG && alt;
  wire [31:0] sum = sub ? a - y : a + y;
  wire lt = $signed(a) < $signed(y);
  wire ltu = a < y;
  wire [4:0] sh = y[4:0];
  logic signed [31:0] sra;
  assign sra = $signed(a) >>> sh;
  logic [31:0] alu;
  always_comb
    case (f3)
      0: alu = sum;
      1: alu = a << sh;
      2: alu = {31'b0, lt};
      3: alu = {31'b0, ltu};
      4: alu = a ^ y;
      5: alu = alt ? sra : a >> sh;
      6: alu = a | y;
      default: alu = a & y;
    endcase
  logic take;
  always_comb
    case (f3)
      0: take = a == y;
      1: take = a != y;
      4: take = lt;
      5: take = !lt;
      6: take = ltu;
      7: take = !ltu;
      default: take = 0;
    endcase
  wire [31:0] pc4 = pc + 4;
  wire [31:0] target = pc + (op == OP_JAL ? imm_j : op == OP_AUIPC ? imm_u : imm_b);
  logic [31:0] next;
  always_comb
    case (op)
      OP_JAL: next = target;
      OP_JALR: next = {sum[31:1], 1'b0};
      OP_BRANCH: next = take ? target : pc4;
      default: next = pc4;
    endcase

  // ---- I/O: the UART and the LED ----
  wire io = sum[31];
  wire tx_busy;
  logic rx_ready = 0;
  logic [7:0] rx_data = 0;
  logic [31:0] io_q = 0;

  // ---- memory ----
  // A store writes its bytes in execute, where a load reads; otherwise
  // the read is the instruction's.
  wire [10:0] maddr = state == EXECUTE ? sum[12:2] : pc[12:2];
  wire store = state == EXECUTE && op == OP_STORE && !io;
  logic [3:0] be;
  logic [31:0] wdata;
  always_comb
    case (f3[1:0])
      0: begin
        be = 4'b0001 << sum[1:0];
        wdata = {4{b[7:0]}};
      end
      1: begin
        be = sum[1] ? 4'b1100 : 4'b0011;
        wdata = {2{b[15:0]}};
      end
      default: begin
        be = 4'b1111;
        wdata = b;
      end
    endcase
  always_ff @(posedge clk) begin
    if (store && be[0]) mem[sum[12:2]][7:0] <= wdata[7:0];
    if (store && be[1]) mem[sum[12:2]][15:8] <= wdata[15:8];
    if (store && be[2]) mem[sum[12:2]][23:16] <= wdata[23:16];
    if (store && be[3]) mem[sum[12:2]][31:24] <= wdata[31:24];
  end
  always_ff @(posedge clk) q <= mem[maddr];

  // ---- the register file ----
  // Read in decode, from the instruction just read; written in execute or,
  // for a load, in load.
  logic [31:0] loaded;
  always_comb begin
    logic [31:0] w;
    logic [15:0] h;
    logic [7:0] c;
    w = ea[31] ? io_q : q;
    h = ea[1] ? w[31:16] : w[15:0];
    case (ea[1:0])
      0: c = w[7:0];
      1: c = w[15:8];
      2: c = w[23:16];
      default: c = w[31:24];
    endcase
    case (f3)
      0: loaded = {{24{c[7]}}, c};
      1: loaded = {{16{h[15]}}, h};
      4: loaded = {24'b0, c};
      5: loaded = {16'b0, h};
      default: loaded = w;
    endcase
  end
  logic [31:0] result;
  always_comb
    case (op)
      OP_LUI: result = imm_u;
      OP_AUIPC: result = target;
      OP_JAL, OP_JALR: result = pc4;
      default: result = alu;
    endcase
  wire writes = op == OP_LUI || op == OP_AUIPC || op == OP_JAL || op == OP_JALR || op == OP_IMM || op == OP_REG;
  wire wreg = rd != 0 && (state == EXECUTE && writes || state == LOAD);
  always_ff @(posedge clk) if (wreg) regs[rd] <= state == LOAD ? loaded : result;
  always_ff @(posedge clk) a <= regs[q[19:15]];
  always_ff @(posedge clk) b <= regs[q[24:20]];

  // ---- the processor's steps ----
  always_ff @(posedge clk)
    case (state)
      FETCH: state <= DECODE;
      DECODE: begin
        ir <= q;
        state <= EXECUTE;
      end
      EXECUTE: begin
        pc <= next;
        ea <= sum;
        state <= op == OP_LOAD ? LOAD : FETCH;
      end
      default: state <= FETCH;
    endcase

  // I/O: a load's word is ready in load (io_q), as memory's is (q).
  wire io_load = state == EXECUTE && op == OP_LOAD && io;
  wire io_store = state == EXECUTE && op == OP_STORE && io;
  always_ff @(posedge clk) if (io_load) io_q <= sum[2] ? {24'b0, rx_data} : {30'b0, rx_ready, tx_busy};
  logic led_r = 0;
  always_ff @(posedge clk) if (io_store && sum[3]) led_r <= b[0];
  assign led = led_r;

  // Receiver: wait for a start bit, sample the middle of each bit. A byte
  // waits in rx_data until a load of it (the last one kept if another
  // comes first).
  logic [1:0] sync = 2'b11;
  always_ff @(posedge clk) sync <= {sync[0], rx};
  wire r = sync[1];
  logic rbusy = 0;
  logic [7:0] rdiv = 0;
  logic [3:0] rbit = 0;
  logic [7:0] rbyte = 0;
  always_ff @(posedge clk) begin
    if (io_load && sum[2]) rx_ready <= 0;
    if (!rbusy) begin
      if (!r) begin
        rbusy <= 1;
        rdiv <= BIT / 2;
        rbit <= 0;
      end
    end else if (rdiv != BIT - 1) rdiv <= rdiv + 1;
    else begin
      rdiv <= 0;
      if (rbit == 0) begin
        if (r) rbusy <= 0;  // a glitch, not a start bit
        else rbit <= 1;
      end else if (rbit != 9) begin
        rbyte <= {r, rbyte[7:1]};
        rbit <= rbit + 1;
      end else begin
        rbusy <= 0;
        if (r) begin
          rx_data <= rbyte;
          rx_ready <= 1;
        end
      end
    end
  end

  // Transmitter: a store to the UART sends a start bit, the byte and a
  // stop bit.
  logic [7:0] tdiv = 0;
  logic [3:0] left = 0;
  logic [9:0] frame = 0;
  assign tx_busy = left != 0;
  always_ff @(posedge clk)
    if (io_store && !sum[3] && !tx_busy) begin
      frame <= {1'b1, b[7:0], 1'b0};
      left <= 10;
      tdiv <= 0;
    end else if (tx_busy) begin
      if (tdiv != BIT - 1) tdiv <= tdiv + 1;
      else begin
        tdiv <= 0;
        frame <= {1'b1, frame[9:1]};
        left <= left - 1;
      end
    end
  assign tx = tx_busy ? frame[0] : 1'b1;
endmodule
