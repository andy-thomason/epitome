// The design of the DPI test (tests/dpi.rs), whose imports are the C
// functions of tests/dpi/lib.rs. Each check prints a line, which the test
// compares with what it should be.

`timescale 1ns/1ps

// Built with TWO_STATE for a simulator without x and z (Verilator), whose
// output the test compares, but for the lines whose values are four-state;
// and without what Verilator does not take: open packed dimensions
// (NO_OPEN_PACKED), unpacked structs as arguments (NO_STRUCT_ARGS), and
// dynamic arrays for open arrays (NO_DYNAMIC_OPEN), and exported tasks
// that wait (NO_TIMED_EXPORTS).
`ifdef TWO_STATE
`define X 1'b0
`define Z 1'b0
`else
`define X 1'bx
`define Z 1'bz
`endif

package util_pkg;
  import "DPI-C" pure function int pkg_twice(input int x);
endpackage

typedef struct {
  int count;
  real scale;
  byte tag;
  bit [39:0] wide;
  string label;
} rec_t;

typedef struct packed {
  bit [3:0] hi;
  logic [3:0] lo;
} pk_t;

typedef enum int { RED = 1, GREEN = 2, BLUE = 7 } color_t;

// A node with exports, each instance a scope of its own.
module node #(parameter int ID = 0) ();
  export "DPI-C" function cell_id;
  export "DPI-C" task cell_wait;
  import "DPI-C" context function int ask_cell(input int k);
  import "DPI-C" context function void remember(input int v);
  import "DPI-C" context function int recall();

  function int cell_id(input int k, output int id);
    id = ID;
    return ID * 100 + k;
  endfunction

  task cell_wait(input int n, output longint at);
`ifndef NO_TIMED_EXPORTS
    #(n);
`endif
    at = $time;
  endtask

  // The imports run in this instance's scope.
  initial begin
    #(2 * ID - 1);
    $display("ask %0d: %0d", ID, ask_cell(ID + 6));
    remember(ID * 11);
    #1;
    $display("recall %0d: %0d", ID, recall());
  end
endmodule

module top;
  import util_pkg::*;

  // Small values, by value, and results (H.7.4, H.8.7, H.8.9).
  import "DPI-C" pure function byte neg_byte(input byte x);
  import "DPI-C" pure function shortint neg_short(input shortint x);
  import "DPI-C" pure function int add_int(input int a, input int b);
  import "DPI-C" pure function longint mul_long(input longint a, input longint b);
  import "DPI-C" pure function int unsigned top_bit(input int unsigned x);
  import "DPI-C" pure function byte unsigned inc_ubyte(input byte unsigned x);
  import "DPI-C" pure function real half(input real x);
  import "DPI-C" pure function shortreal scale_short(input shortreal x, input real k);
  import "DPI-C" pure function bit not_bit(input bit b);
  import "DPI-C" pure function logic pass_logic(input logic l);
  import "DPI-C" pure function string greet(input string who);
  import "DPI-C" function chandle make_counter(input int start);
  import "DPI-C" function int bump_counter(input chandle c);
  import "DPI-C" function void free_counter(input chandle c);
  import "DPI-C" function int color_next(input color_t c);
  import "DPI-C" pure function real weigh(input int a0, input int a1, input int a2, input int a3, input int a4,
                                          input int a5, input int a6, input int a7, input int a8, input int a9,
                                          input real r0, input real r1, input real r2, input real r3, input real r4,
                                          input real r5, input real r6, input real r7, input real r8, input real r9,
                                          input byte b, input shortreal s);
  // A linkage name, and a function of the C library.
  import "DPI-C" pure add_int = function int plus(input int a, input int b);
  import "DPI-C" pure function int atoi(input string s);

  // Outputs and inouts, by reference (H.8.8, H.8.10).
  import "DPI-C" function void split_long(input longint x, output int hi, output int lo);
  import "DPI-C" function void swap_reals(inout real a, inout real b);
  import "DPI-C" function void out_small(output byte b, output shortint s, output bit t, output logic l,
                                         output shortreal f, output byte unsigned u);
  import "DPI-C" function void out_string(output string s);
  import "DPI-C" function void shout(inout string s);

  // Packed values in canonical form (H.7.7).
  import "DPI-C" function void vec_ops(input bit [7:0] a, input logic [7:0] b, output bit [95:0] wide,
                                       inout logic [69:0] l);
  import "DPI-C" function int vec_popcount(input bit [127:0] v);
  import "DPI-C" function void integer_time(input integer i, input time t, output integer oi, output time ot);
  import "DPI-C" function int packed_struct(input pk_t p, output pk_t q);

  // Unpacked structs and arrays, in C layout (H.7.8, H.11.4).
`ifndef NO_STRUCT_ARGS
  import "DPI-C" function real rec_sum(input rec_t r);
  import "DPI-C" function void rec_make(output rec_t r);
`endif
  import "DPI-C" function int arr_sum(input int a [4]);
  import "DPI-C" function void arr_rev(inout logic [7:0] m [2:0]);

  // Open arrays (H.12).
  import "DPI-C" function int open_sum(input int a []);
  import "DPI-C" function void open_fill(output logic [15:0] o []);
`ifndef NO_OPEN_PACKED
  import "DPI-C" function int open_packed(input bit [] p);
`endif
  import "DPI-C" function void open_2d(input bit [3:0] a [][], output int r [12]);
  import "DPI-C" function void open_scalars(inout logic s []);
`ifndef NO_STRUCT_ARGS
  import "DPI-C" function real open_recs(input rec_t r []);
`endif

  // The canonical-form utilities (H.11.5), and the version (H.10.1.3).
  import "DPI-C" function void selects(inout logic [63:0] v, input bit [63:0] b, output int r [6]);
  import "DPI-C" function string dpi_version();

  // Context: scopes, user data, exports (H.9, 35.5.3).
  import "DPI-C" context function int call_cell(input string path, input int k);
  import "DPI-C" context function string scope_name();
  import "DPI-C" context function int no_such_scope();
  import "DPI-C" context function int time_info(output int unit, output int prec, output longint ticks,
                                                output real scaled);

  // Imported tasks that wait in exported tasks (35.8, 35.9).
  import "DPI-C" context task run_cell(input string path, input int n, output longint at);
  import "DPI-C" context task nested(input int depth);
  import "DPI-C" context task wait_until_disabled();
  import "DPI-C" function int disabled_seen();
  import "DPI-C" task plain_task(input int x, output int y);
  export "DPI-C" task descend;
  export "DPI-C" task long_wait;
  export "DPI-C" function trace;

  node #(.ID(1)) c1();
  node #(.ID(2)) c2();

  int depth_log [$];

  function void trace(input string what, input int v);
    $display("trace %s %0d at %0t", what, v, $time);
  endfunction

  task descend(input int depth);
`ifndef NO_TIMED_EXPORTS
    #1;
`endif
    depth_log.push_back(depth);
    if (depth > 0) nested(depth - 1);
  endtask

  task long_wait();
`ifndef NO_TIMED_EXPORTS
    #100;
`endif
    $display("FAIL: long_wait was not disabled");
  endtask

  rec_t r, rr;
  rec_t recs [1:3];
  int a4 [4];
  int fixed_a [3:0];
  int dyn_a [];
  logic [15:0] fill [2:5];
  logic [15:0] dfill [];
  bit [3:0] grid [2:5][1:0];
  logic flags [0:3];
  logic [7:0] m [2:0];
  int res12 [12];
  int res6 [6];

  initial begin : main
    int hi, lo, ia, unit, prec;
    real ra, rb, scaled;
    byte ob;
    shortint os;
    bit ot;
    logic ol;
    shortreal of;
    byte unsigned ou;
    string s;
    bit [95:0] wide;
    logic [69:0] l70;
    integer oi;
    time otm;
    pk_t q;
    chandle h;
    longint ticks, at1, at2;
    logic [63:0] v64;

    $display("bytes %0d %0d %0d", neg_byte(5), neg_byte(-128), inc_ubyte(255));
    $display("ints %0d %0d %0d", neg_short(-300), add_int(40000, -2), plus(3, 4));
    $display("long %0d unsigned %0d", mul_long(64'd3000000000, -3), top_bit(32'h8000_0001));
    $display("reals %0.3f %0.3f", half(5.5), scale_short(1.5, 3.0));
    $display("scalars %b %b %b %b %b", not_bit(1'b0), pass_logic(1'b0), pass_logic(1'b1), pass_logic(`X), pass_logic(`Z));
    $display("string %s %0d", greet("dpi"), atoi("12345"));
    h = make_counter(10);
    void'(bump_counter(h));
    $display("chandle %0d null %0d", bump_counter(h), h == null);
    free_counter(h);
    $display("enum %0d", color_next(GREEN));
    $display("weigh %0.2f", weigh(1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 0.5, 1.5, 2.5, 3.5, 4.5, 5.5, 6.5, 7.5, 8.5, 9.5, -4, 0.25));
    $display("package %0d", pkg_twice(21));

    split_long(64'h0000_0007_ffff_fffe, hi, lo);
    ra = 1.25;
    rb = -2.0;
    swap_reals(ra, rb);
    $display("split %0d %0d swapped %0.2f %0.2f", hi, lo, ra, rb);
    out_small(ob, os, ot, ol, of, ou);
    $display("outs %0d %0d %b %b %0.2f %0d", ob, os, ot, ol, of, ou);
    out_string(s);
    $display("out string %s", s);
    s = "quiet";
    shout(s);
    $display("inout string %s", s);

    l70 = {2'b10, `X, `Z, 2'b01, 64'h0123_4567_89ab_cdef};
    vec_ops(8'ha5, {1'b1, `X, 1'b0, `Z, 4'b0101}, wide, l70);
    $display("vec wide %h l70 %b_%h", wide, l70[69:64], l70[63:0]);
    $display("popcount %0d", vec_popcount({64'hffff_0000_0000_0001, 64'h8000_0000_0000_0003}));
    integer_time(-5, 64'd1234567890123, oi, otm);
    $display("integer %0d time %0d", oi, otm);
    $display("packed struct %0d q %h", packed_struct('{hi: 4'h9, lo: {1'b1, `X, 1'b0, `Z}}, q), q.hi);

    r.count = 3;
    r.scale = 1.5;
    r.tag = -2;
    r.wide = 40'h12_3456_789a;
    r.label = "abc";
`ifndef NO_STRUCT_ARGS
    $display("rec sum %0.2f", rec_sum(r));
    rec_make(rr);
    $display("rec made %0d %0.3f %0d %h %s", rr.count, rr.scale, rr.tag, rr.wide, rr.label);
`endif
    a4 = '{1, 2, 3, 40};
    $display("arr sum %0d", arr_sum(a4));
    m = '{8'h11, 8'h22, 8'h33};
    arr_rev(m);
    $display("arr rev %h %h %h", m[2], m[1], m[0]);

    fixed_a = '{4, 3, 2, 1};
    $display("open sum %0d", open_sum(fixed_a));
    open_fill(fill);
    $display("open fill %h %h %h %h", fill[2], fill[3], fill[4], fill[5]);
`ifndef NO_DYNAMIC_OPEN
    dyn_a = new [5];
    foreach (dyn_a[i]) dyn_a[i] = i * 10;
    dfill = new [2];
    open_fill(dfill);
    $display("open dynamic %0d %h %h", open_sum(dyn_a), dfill[0], dfill[1]);
`endif
`ifndef NO_OPEN_PACKED
    $display("open packed %0d %0d", open_packed(12'b1010_0000_0111), open_packed(70'h3f_0000_0000_0000_0001));
`endif
    foreach (grid[i, j]) grid[i][j] = 4'(i * 2 + j);
    open_2d(grid, res12);
    $display("open 2d dims %0d [%0d:%0d] low %0d high %0d inc %0d size %0d packed %0d", res12[0], res12[1], res12[2], res12[3],
             res12[4], res12[5], res12[6], res12[7]);
    $display("open 2d elems %0d %0d %0d %0d", res12[8], res12[9], res12[10], res12[11]);
    flags = '{1'b0, 1'b1, `X, `Z};
    open_scalars(flags);
    $display("open scalars %b%b%b%b", flags[0], flags[1], flags[2], flags[3]);
`ifndef NO_STRUCT_ARGS
    recs[1] = r;
    recs[2] = rr;
    recs[3] = r;
    $display("open recs %0.2f", open_recs(recs));
`endif

    v64 = 64'h0000_ffff_0000_0000;
    v64[3:0] = {`X, 2'b01, `Z};
    selects(v64, 64'hdead_beef_0123_4567, res6);
    $display("selects %h_%h %h %h %h %h %h %h", v64[63:32], v64[31:0], res6[0], res6[1], res6[2], res6[3], res6[4], res6[5]);
    $display("version %s", dpi_version());

    $display("scope %s", scope_name());
    $display("cells %0d %0d %0d", call_cell("top.c1", 5), call_cell("top.c2", 6), no_such_scope());

    #6;
    ia = time_info(unit, prec, ticks, scaled);
    $display("time %0d %0d %0d %0d %0.1f", ia, unit, prec, ticks, scaled);

`ifndef NO_TIMED_EXPORTS
    fork
      run_cell("top.c1", 4, at1);
      run_cell("top.c2", 7, at2);
    join
    $display("run_cell at %0t: %0d %0d", $time, at1, at2);

    nested(3);
    $display("nested at %0t:%p", $time, depth_log);
`endif

    plain_task(6, ia);
    $display("plain task %0d", ia);

`ifndef NO_TIMED_EXPORTS
    fork
      wait_until_disabled();
    join_none
    #5 disable fork;
    #1 $display("disabled at %0t: %0d", $time, disabled_seen());
`endif
    $display("done");
    $finish;
  end
endmodule
