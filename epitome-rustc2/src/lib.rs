//! `epitome-rustc2`: a Rust compiler that does not use rustc.
//!
//! The plan is in `docs/design.md`. So far: the lexer (`lexer`), the
//! syntax tree (`ast`) and the parser (`parse`).

pub mod ast;
pub mod lexer;
pub mod parse;
pub mod source;
