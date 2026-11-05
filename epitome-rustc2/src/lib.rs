//! `epitome-rustc2`: a Rust compiler that does not use rustc.
//!
//! The plan is in `docs/design.md`. So far: the lexer (`lexer`), the
//! syntax tree (`ast`), the parser (`parse`) and the
//! printer that turns a tree back into source (`pretty`).

pub mod ast;
pub mod lexer;
pub mod parse;
pub mod pretty;
pub mod source;
