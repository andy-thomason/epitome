//! Borrow checking.
//!
//! Work in progress: the plan, and what each milestone covers, is in
//! `docs/borrowck.md`. So far there is the interface the checker plugs into
//! (`Session::borrowck` and `Diagnostic`), the `--borrowck` flag of the
//! driver, and the expected-errors runner of `tests/ui_check_pass.rs`
//! (`EPITOME_UI_WANT=borrowck`), which measures it. The checker itself
//! finds nothing yet.
//!
//! The rule that matters most: it must never reject what rustc accepts.
//! Every program of the check-pass and run-pass UI tests, and `core`,
//! `alloc` and `std`, are checked and must have no diagnostics.

use crate::ast::{Ast, NodeId};
use crate::resolve::Resolver;
use crate::session::{ParsedCrate, Session};
use crate::ty::TyCtxt;
use crate::typeck::TypeckResults;

/// A borrow-checking error, where rustc would report it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Diagnostic {
    /// The crate it is in.
    pub krate: u32,
    /// The file name, line and column (1-based) of its primary span.
    pub file: String,
    pub line: usize,
    pub col: usize,
    /// rustc's error code: `E0382`.
    pub code: &'static str,
    pub message: String,
}

impl std::fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}:{}: error[{}]: {}", self.file, self.line, self.col, self.code, self.message)
    }
}

/// The error codes this checker is to give: those of rustc's borrow
/// checker (and its mutability and move checks). A UI test is of it when
/// all the errors it expects have one of these.
pub const CODES: &[&str] = &[
    "E0381", "E0382", "E0384", "E0499", "E0500", "E0501", "E0502", "E0503", "E0505", "E0506", "E0507", "E0508",
    "E0509", "E0510", "E0515", "E0516", "E0521", "E0594", "E0596", "E0597", "E0712", "E0713", "E0716", "E0373",
];

impl Session {
    /// Borrow checks the bodies of the crates `only` names (the others are
    /// not looked at): rustc's errors, or fewer. Nothing yet.
    pub fn borrowck<'s>(
        &'s self,
        crates: &[ParsedCrate<'s>],
        r: &Resolver<'s>,
        tcx: &TyCtxt<'s>,
        res: &TypeckResults,
        only: &[u32],
    ) -> Vec<Diagnostic> {
        let _ = (crates, r, tcx, res, only);
        Vec::new()
    }

    /// A diagnostic for `node` of `ast` (in crate `krate`).
    pub fn diagnostic_at(&self, ast: &Ast<'_>, node: NodeId, krate: u32, code: &'static str, message: impl Into<String>) -> Diagnostic {
        let at = self.sources.find(ast.token(node).0).map(|s| s.locate(ast.token(node).0));
        let (file, line, col) = at.map_or((String::new(), 0, 0), |l| (l.name, l.line, l.col));
        Diagnostic { krate, file, line, col, code, message: message.into() }
    }
}
