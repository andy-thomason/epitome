//! A SystemVerilog compiler, written from IEEE 1800-2023. See
//! `docs/design/sv.md` for the plan.

pub mod arch;
pub mod ast;
pub mod bdd;
pub mod design;
pub mod hash;
pub mod keywords;
pub mod lexer;
pub mod model;
pub mod netlist;
pub mod parse;
pub mod preprocess;
pub mod sim;
pub mod source;
pub mod stimulus;
pub mod synth;
pub mod verilator;
