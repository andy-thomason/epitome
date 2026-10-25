//! A SystemVerilog compiler, written from IEEE 1800-2023. See
//! `docs/design/sv.md` for the plan.

pub mod ast;
pub mod keywords;
pub mod lexer;
pub mod parse;
pub mod preprocess;
pub mod source;
