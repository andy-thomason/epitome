//! `epitome-rustc2`: a Rust compiler that does not use rustc.
//!
//! The plan is in `docs/design.md`. So far: the lexer (`lexer`), the
//! syntax tree (`ast`), the parser (`parse`) and the
//! printer that turns a tree back into source (`pretty`). `session` is the
//! library's entry point: it loads whole crates, several at once, parsing
//! their files in parallel on a pool of threads (`pool`).

pub mod ast;
pub mod cfg;
pub mod consteval;
pub mod expand;
pub mod hash;
pub mod intern;
pub mod json;
pub mod layout;
pub mod lexer;
pub mod parse;
pub mod pool;
pub mod pretty;
pub mod proc_macro;
pub mod resolve;
pub mod session;
pub mod source;
pub mod ty;
pub mod typeck;
