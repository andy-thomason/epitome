//! LLVM IR for Epitome: the in-memory representation, plus a parser and
//! printer for the text format (`.ll`).
//!
//! Written from LLVM's published documentation only; see `docs/llvm-ir`.
//!
//! ```
//! let src = "define i32 @add(i32 %a, i32 %b) {\n  %s = add i32 %a, %b\n  ret i32 %s\n}\n";
//! let module = epitome_ir::parse_module(src).unwrap();
//! assert!(module.to_string().contains("%s = add i32 %a, %b"));
//! ```

pub mod apint;
pub mod build;
pub mod chop;
pub mod edit;
pub mod fits;
pub mod float;
pub mod io;
pub mod layout;
pub mod lexer;
pub mod memlimit;
pub mod metadata;
pub mod module;
pub mod parser;
pub mod printer;
pub mod types;

pub use module::Module;
pub use parser::{ParseError, parse_module};
pub use printer::print_module;
