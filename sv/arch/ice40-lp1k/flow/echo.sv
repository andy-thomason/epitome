// Board test (a): combinational. The UART line from the Mac (rx, pin A3)
// goes back to it (tx, pin B3): `rx & 1'b1` folds to rx itself, a route
// through the fabric alone, and echoes; with INVERT defined, tx is ~rx
// from a LUT, and the Mac hears no echo.
module echo (input logic rx, output logic tx);
`ifdef INVERT
  assign tx = ~rx;
`else
  assign tx = rx & 1'b1;
`endif
endmodule
