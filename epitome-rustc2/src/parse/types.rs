//! Paths, generics, bounds, `where` clauses and types.

use super::{PResult, Parser};
use crate::ast::Kind;

/// How a path's generic arguments are written.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum PathMode {
    /// In a type: `Vec<u8>`, `Fn(u8) -> u8`.
    Type,
    /// In an expression or pattern: only `Vec::<u8>`.
    Expr,
    /// In an attribute, `use` or visibility: no generic arguments.
    Simple,
}

impl<'a> Parser<'a> {
    /// Whether token `n` can be a path segment's name: a name, `self`,
    /// `Self`, `super`, `crate`, or `$crate`.
    pub(crate) fn segment_at(&self, n: usize) -> bool {
        self.name_at(n)
            || matches!(self.tok(n).0, "self" | "Self" | "super" | "crate")
            || self.at_n(n, "$") && self.at_n(n + 1, "crate") && self.tok(n).touches(self.tok(n + 1))
    }

    /// Whether a path starts at token `n`.
    pub(crate) fn path_start_at(&self, n: usize) -> bool {
        self.segment_at(n) || self.at_op_n(n, "::") || self.at_n(n, "<")
    }

    /// A path, optionally qualified (`<T as Tr>::x`) or rooted (`::a`).
    pub(crate) fn path(&mut self, mode: PathMode) -> PResult<'a> {
        let m = self.mark();
        if self.at_op("::") {
            self.leaf(Kind::PathRoot, 2);
        } else if mode != PathMode::Simple && self.at("<") {
            // Also `<<T as A>::B as C>::D`, as `<` is always one token.
            self.qself()?;
        }
        loop {
            self.segment(mode)?;
            if self.at_op("::") && self.segment_at(2) {
                self.bump_n(2);
                continue;
            }
            // `Vec::<u8>::new` is handled in `segment`; `a::{` and `a::*`
            // belong to `use` trees.
            break;
        }
        self.finish(m, Kind::Path);
        Ok(())
    }

    /// `<Type as Trait>::` or `<Type>::`.
    fn qself(&mut self) -> PResult<'a> {
        let m = self.mark();
        self.expect("<")?;
        self.ty()?;
        if self.eat("as") {
            self.ty()?;
        }
        self.expect_close_angle()?;
        self.finish(m, Kind::QSelf);
        self.expect_op("::")
    }

    fn segment(&mut self, mode: PathMode) -> PResult<'a> {
        let m = self.mark();
        if self.at("$") {
            self.leaf(Kind::Name, 2);
        } else if self.segment_at(0) {
            self.leaf(Kind::Name, 1);
        } else {
            return self.error("expected a path segment");
        }
        match mode {
            PathMode::Simple => {}
            PathMode::Type => {
                if self.at_op("::") && (self.at_n(2, "<") || self.at_n(2, "(")) {
                    self.bump_n(2);
                }
                if self.at("<") && !self.at_op("<=") && !self.at_op("<<=") {
                    self.generic_args()?;
                } else if self.at("(") {
                    self.fn_sugar()?;
                }
            }
            PathMode::Expr => {
                if self.at_op("::") && self.at_n(2, "<") {
                    self.bump_n(2);
                    self.generic_args()?;
                } else if self.at_op("::") && self.at_n(2, "(") && self.at_n(3, ".") {
                    // Return type notation: `method(..)`.
                    self.bump_n(2);
                    self.fn_sugar()?;
                }
            }
        }
        self.finish(m, Kind::Segment);
        Ok(())
    }

    /// `<'a, T, N, { e }, Item = T, Item: Bound>`.
    pub(crate) fn generic_args(&mut self) -> PResult<'a> {
        let m = self.mark();
        self.expect("<")?;
        while !self.at(">") {
            self.generic_arg()?;
            if !self.eat(",") {
                break;
            }
        }
        self.expect_close_angle()?;
        self.finish(m, Kind::GenericArgs);
        Ok(())
    }

    fn generic_arg(&mut self) -> PResult<'a> {
        if self.lifetime_at(0) {
            self.leaf(Kind::Lifetime, 1);
            return Ok(());
        }
        // An associated constraint, `Name (<args> | (sugar))? (= | :)`, is
        // read as a type first, then rebuilt if `=` or `:` follows. Trying
        // the constraint first and backing up would be exponential in
        // the depth of nested generic arguments.
        let m = self.mark();
        self.generic_arg_value()?;
        let eq = self.at("=") && !self.at_op("==");
        let colon = self.at(":") && !self.at_op("::");
        if (eq || colon) && self.top_is_simple_segment() {
            // PathType -> Path -> Segment -> Name, args?
            self.unwrap_top();
            self.unwrap_top();
            self.unwrap_top();
            self.bump();
            if colon {
                self.bounds(true)?;
            } else {
                self.generic_arg_value()?;
            }
            self.finish(m, Kind::AssocConstraint);
        }
        Ok(())
    }

    /// Whether the last node is a path type of one segment with a plain
    /// name and no root or qualified self: what can start an associated
    /// constraint.
    fn top_is_simple_segment(&self) -> bool {
        let id = self.top();
        if self.ast.kind(id) != Kind::PathType {
            return false;
        }
        let path = self.ast.children(id)[0];
        let segs = self.ast.children(path);
        segs.len() == 1 && self.ast.kind(segs[0]) == Kind::Segment
    }

    /// A type or a constant argument.
    fn generic_arg_value(&mut self) -> PResult<'a> {
        if self.at("{") {
            self.block_expr()
        } else if self.literal_at(0) {
            self.leaf(Kind::Literal, 1);
            Ok(())
        } else if self.at("-") && self.literal_at(1) {
            let m = self.mark();
            self.leaf(Kind::Op, 1);
            self.leaf(Kind::Literal, 1);
            self.finish(m, Kind::Unary);
            Ok(())
        } else {
            self.ty()
        }
    }

    /// `(A, B) -> C` after `Fn`, or `(..)`.
    fn fn_sugar(&mut self) -> PResult<'a> {
        let m = self.mark();
        self.expect("(")?;
        if self.at_op("..") {
            self.leaf(Kind::Op, 2);
        } else {
            while !self.at(")") {
                self.ty()?;
                if !self.eat(",") {
                    break;
                }
            }
        }
        self.expect(")")?;
        if self.at_op("->") {
            self.ret_type(false)?;
        }
        self.finish(m, Kind::FnSugar);
        Ok(())
    }

    /// `-> Type`. In `Fn() -> T + Send` and `fn() -> T` the type cannot
    /// take `+ Bound`; after a function's parameters it can (`-> impl A +
    /// B`).
    pub(crate) fn ret_type(&mut self, plus: bool) -> PResult<'a> {
        let m = self.mark();
        self.expect_op("->")?;
        self.ty_inner(plus)?;
        self.finish(m, Kind::RetType);
        Ok(())
    }

    // ---- generics ----

    /// Whether `<` at token `n` starts generic parameters rather than a
    /// qualified path (`impl <T as A>::B {}`).
    pub(crate) fn generics_at(&self, n: usize) -> bool {
        if !self.at_n(n, "<") {
            return false;
        }
        let t = self.tok(n + 1);
        t.0 == ">"
            || t.0 == "#"
            || t.0 == "const"
            || t.is_lifetime()
            || self.name_at(n + 1) && matches!(self.tok(n + 2).0, ">" | "," | ":" | "=")
    }

    /// `<'a: 'b, T: Bound = Default, const N: usize = 3>`, if present.
    pub(crate) fn generics(&mut self) -> PResult<'a> {
        if !self.at("<") {
            return Ok(());
        }
        let m = self.mark();
        self.bump();
        while !self.at(">") {
            let pm = self.mark();
            self.outer_attributes()?;
            if self.lifetime_at(0) {
                self.leaf(Kind::Lifetime, 1);
                if self.eat(":") {
                    self.lifetime_bounds()?;
                }
                self.finish(pm, Kind::LifetimeParam);
            } else if self.eat("const") {
                self.name()?;
                self.expect(":")?;
                self.ty()?;
                if self.eat("=") {
                    self.generic_arg_value()?;
                }
                self.finish(pm, Kind::ConstParam);
            } else {
                self.name()?;
                if self.eat(":") {
                    self.bounds(true)?;
                }
                if self.eat("=") {
                    self.ty()?;
                }
                self.finish(pm, Kind::TypeParam);
            }
            if !self.eat(",") {
                break;
            }
        }
        self.expect_close_angle()?;
        self.finish(m, Kind::Generics);
        Ok(())
    }

    /// The fields of `offset_of` and `field_of`, `a.b.0` or `Variant.f`,
    /// as a `TokenTree` of the tokens before the closing `)`.
    pub(crate) fn field_path(&mut self) -> PResult<'a> {
        let m = self.mark();
        while !self.at(")") && !self.tok(0).is_eof() {
            self.bump();
        }
        self.finish(m, Kind::TokenTree);
        Ok(())
    }

    /// A name, as a `Name` leaf.
    pub(crate) fn name(&mut self) -> PResult<'a> {
        if self.name_at(0) {
            self.leaf(Kind::Name, 1);
            Ok(())
        } else {
            self.error("expected a name")
        }
    }

    /// `'a + 'b`, after `'x:`.
    fn lifetime_bounds(&mut self) -> PResult<'a> {
        let m = self.mark();
        while self.lifetime_at(0) {
            self.leaf(Kind::Lifetime, 1);
            if !self.eat("+") {
                break;
            }
        }
        self.finish(m, Kind::Bounds);
        Ok(())
    }

    /// `for<'a, 'b>`, if present.
    pub(crate) fn for_lifetimes(&mut self) -> PResult<'a> {
        if !(self.at("for") && self.at_n(1, "<")) {
            return Ok(());
        }
        let m = self.mark();
        self.bump_n(2);
        while !self.at(">") {
            let pm = self.mark();
            self.outer_attributes()?;
            if self.lifetime_at(0) {
                self.leaf(Kind::Lifetime, 1);
                if self.eat(":") {
                    self.lifetime_bounds()?;
                }
                self.finish(pm, Kind::LifetimeParam);
            } else {
                // `for<T>` binders of types and constants are unstable;
                // read them as type parameters.
                self.name()?;
                if self.eat(":") {
                    self.bounds(true)?;
                }
                self.finish(pm, Kind::TypeParam);
            }
            if !self.eat(",") {
                break;
            }
        }
        self.expect_close_angle()?;
        self.finish(m, Kind::ForLifetimes);
        Ok(())
    }

    /// `where T: A, 'a: 'b,`, if present.
    pub(crate) fn where_clause(&mut self) -> PResult<'a> {
        if !self.at("where") {
            return Ok(());
        }
        let m = self.mark();
        self.bump();
        while !(self.at("{") || self.at(";") || self.at("=") || self.at_eof()) {
            let pm = self.mark();
            if self.lifetime_at(0) {
                self.leaf(Kind::Lifetime, 1);
                self.expect(":")?;
                self.lifetime_bounds()?;
            } else {
                self.for_lifetimes()?;
                self.ty()?;
                if self.eat(":") {
                    self.bounds(true)?;
                } else if self.at("=") && !self.at_op("==") {
                    // `T = U` equality predicates (unstable).
                    self.bump();
                    self.ty()?;
                }
            }
            self.finish(pm, Kind::WherePredicate);
            if !self.eat(",") {
                break;
            }
        }
        self.finish(m, Kind::WhereClause);
        Ok(())
    }

    // ---- bounds ----

    /// Whether a bound can start at token `n`.
    fn bound_at(&self, n: usize) -> bool {
        let t = self.tok(n);
        t.is_lifetime()
            || matches!(t.0, "(" | "?" | "~" | "!" | "[" | "for" | "const" | "async" | "use")
            || self.path_start_at(n)
    }

    /// Bounds joined by `+`. Without `plus`, only one.
    pub(crate) fn bounds(&mut self, plus: bool) -> PResult<'a> {
        let m = self.mark();
        while self.bound_at(0) {
            self.bound()?;
            if !plus || !self.eat("+") {
                break;
            }
        }
        self.finish(m, Kind::Bounds);
        Ok(())
    }

    fn bound(&mut self) -> PResult<'a> {
        if self.lifetime_at(0) {
            self.leaf(Kind::Lifetime, 1);
            return Ok(());
        }
        if self.at("use") && self.at_n(1, "<") {
            let m = self.mark();
            self.bump_n(2);
            while !self.at(">") {
                if self.lifetime_at(0) {
                    self.leaf(Kind::Lifetime, 1);
                } else if self.segment_at(0) {
                    self.leaf(Kind::Name, 1);
                } else {
                    return self.error("expected a lifetime or a name");
                }
                if !self.eat(",") {
                    break;
                }
            }
            self.expect_close_angle()?;
            self.finish(m, Kind::UseBound);
            return Ok(());
        }
        if self.at("(") {
            // `(?Sized)`, `(Tr + 'a)` are a parenthesised bound.
            self.bump();
            self.bound()?;
            return self.expect(")");
        }
        let m = self.mark();
        self.for_lifetimes()?;
        loop {
            if self.at("~") && self.at_n(1, "const") {
                self.leaf(Kind::Op, 1);
                self.leaf(Kind::Keyword, 1);
            } else if self.at("[") && self.at_n(1, "const") && self.at_n(2, "]") {
                self.leaf(Kind::Keyword, 3);
            } else if self.at("?") || self.at("!") {
                self.leaf(Kind::Op, 1);
            } else if self.at("const") || self.at("async") {
                self.leaf(Kind::Keyword, 1);
            } else {
                break;
            }
        }
        self.path(PathMode::Type)?;
        self.finish(m, Kind::TraitBound);
        Ok(())
    }

    // ---- types ----

    /// A type, which may be `impl A + B` or `dyn A + B`.
    pub(crate) fn ty(&mut self) -> PResult<'a> {
        self.ty_inner(true)
    }

    /// A type that cannot take `+ Bound`: after `&`, `*const`, `->` and
    /// `as`.
    pub(crate) fn ty_no_plus(&mut self) -> PResult<'a> {
        self.ty_inner(false)
    }

    fn ty_inner(&mut self, plus: bool) -> PResult<'a> {
        let m = self.mark();
        let t = self.tok(0);
        match t.0 {
            "(" => {
                self.bump();
                let mut n = 0;
                let mut comma = false;
                while !self.at(")") {
                    self.ty()?;
                    n += 1;
                    // `(u32 is 1..)`, from `pattern_type!`: the pattern as
                    // tokens.
                    if n == 1 && self.at("is") {
                        self.bump();
                        let tt = self.mark();
                        while !self.at(")") && !self.tok(0).is_eof() {
                            self.bump();
                        }
                        self.finish(tt, Kind::TokenTree);
                        self.expect(")")?;
                        self.finish(m, Kind::PatternType);
                        return Ok(());
                    }
                    comma = self.eat(",");
                    if !comma {
                        break;
                    }
                }
                self.expect(")")?;
                let kind = if n == 1 && !comma { Kind::ParenType } else { Kind::TupleType };
                self.finish(m, kind);
                // `(Tr) + Send` in old code: a bound list.
                if kind == Kind::ParenType && plus && self.at("+") {
                    return self.continue_bare_bounds(m);
                }
            }
            "!" => {
                self.bump();
                self.finish(m, Kind::NeverType);
            }
            "_" => {
                self.bump();
                self.finish(m, Kind::InferType);
            }
            "&" => {
                self.bump();
                if self.lifetime_at(0) {
                    self.leaf(Kind::Lifetime, 1);
                }
                if self.at("mut") {
                    self.leaf(Kind::Keyword, 1);
                }
                self.ty_no_plus()?;
                self.finish(m, Kind::RefType);
            }
            "*" => {
                self.bump();
                if self.at("const") || self.at("mut") {
                    self.leaf(Kind::Keyword, 1);
                } else {
                    return self.error("expected `const` or `mut`");
                }
                self.ty_no_plus()?;
                self.finish(m, Kind::PtrType);
            }
            "[" => {
                self.bump();
                self.ty()?;
                let kind = if self.eat(";") {
                    self.expr()?;
                    Kind::ArrayType
                } else {
                    Kind::SliceType
                };
                self.expect("]")?;
                self.finish(m, kind);
            }
            "impl" => {
                self.bump();
                self.bounds(plus)?;
                self.finish(m, Kind::ImplTraitType);
            }
            "dyn" if self.edition >= crate::lexer::Edition::E2018 || self.bound_at(1) && !self.at_op_n(1, "::") => {
                self.leaf(Kind::Keyword, 1);
                self.bounds(plus)?;
                self.finish(m, Kind::DynTraitType);
            }
            "unsafe" if self.at_n(1, "<") => {
                // An unsafe binder, `unsafe<'a> &'a T`.
                self.bump_n(2);
                while self.lifetime_at(0) {
                    self.leaf(Kind::Lifetime, 1);
                    if !self.eat(",") {
                        break;
                    }
                }
                self.expect_close_angle()?;
                self.ty_no_plus()?;
                self.finish(m, Kind::UnsafeBinderType);
            }
            "builtin" if self.at_n(1, "#") => {
                self.bump_n(2);
                self.name()?;
                self.expect("(")?;
                self.ty()?;
                self.expect(",")?;
                self.field_path()?;
                self.expect(")")?;
                self.finish(m, Kind::BuiltinType);
            }
            "fn" | "unsafe" | "extern" => self.fn_ptr_type(m)?,
            "for" if self.at_n(1, "<") => {
                // `for<'a> fn(&'a u8)` or a bound `for<'a> Tr<'a>`.
                let s = self.save();
                self.for_lifetimes()?;
                let fn_ptr = self.at("fn") || self.at("unsafe") || self.at("extern");
                self.restore(s);
                if fn_ptr {
                    self.fn_ptr_type(m)?;
                } else {
                    self.bounds(plus)?;
                    self.finish(m, Kind::DynTraitType);
                }
            }
            "?" => {
                // `?Sized` as a bare bound list in old code.
                self.bounds(plus)?;
                self.finish(m, Kind::DynTraitType);
            }
            _ if self.path_start_at(0) => {
                self.path(PathMode::Type)?;
                if self.at("!") && !self.at_op("!=") {
                    self.bump();
                    self.token_tree()?;
                    self.finish(m, Kind::TypeMacro);
                } else {
                    self.finish(m, Kind::PathType);
                    if plus && self.at("+") {
                        return self.continue_bare_bounds(m);
                    }
                }
            }
            _ => return self.error("expected a type"),
        }
        Ok(())
    }

    /// A path type or parenthesised bound followed by `+`: rebuilds it as
    /// the first bound of a bare trait object (`Tr + Send`, as written
    /// before `dyn`; also `T: Tr + Send` in `where` clauses).
    fn continue_bare_bounds(&mut self, m: super::Mark) -> PResult<'a> {
        // The path type becomes a trait bound: its `Path` child is kept.
        // `m` marks the stack below the path type, so it also gathers the
        // bounds.
        self.unwrap_top();
        self.finish(m, Kind::TraitBound);
        while self.eat("+") {
            if !self.bound_at(0) {
                break;
            }
            self.bound()?;
        }
        self.finish(m, Kind::Bounds);
        self.finish(m, Kind::DynTraitType);
        Ok(())
    }

    /// `for<'a> unsafe extern "C" fn(A, b: B, ...) -> R`.
    fn fn_ptr_type(&mut self, m: super::Mark) -> PResult<'a> {
        self.for_lifetimes()?;
        if self.at("unsafe") || self.at("safe") {
            self.leaf(Kind::Keyword, 1);
        }
        if self.at("extern") {
            self.leaf(Kind::Keyword, 1);
            if self.tok(0).is_string() {
                self.leaf(Kind::Abi, 1);
            }
        }
        self.expect("fn")?;
        self.params(false)?;
        if self.at_op("->") {
            self.ret_type(false)?;
        }
        self.finish(m, Kind::FnPtrType);
        Ok(())
    }
}
