//! Items: functions, types, traits, impls, modules, `use`, constants,
//! statics, extern blocks and macro definitions.
//!
//! An item starts with qualifiers (`pub`, `const`, `async`, `unsafe`,
//! `extern "C"`, and the weak keywords `default`, `auto`, `safe`, `gen`),
//! and which item it is shows only in the keyword after them. `item_kind`
//! looks past the qualifiers to find it; each item's parser then reads the
//! qualifiers as `Keyword` children.

use super::types::PathMode;
use super::{Mark, PResult, Parser};
use crate::ast::Kind;

/// What item follows the qualifiers.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum ItemKind {
    Fn,
    Struct,
    Enum,
    Union,
    Trait,
    Impl,
    Mod,
    Use,
    Const,
    Static,
    Type,
    ExternCrate,
    ExternBlock,
    MacroRules,
    MacroDef,
    MacroCall,
}

impl<'a> Parser<'a> {
    /// Items until `close` (`}`) or the end of the input.
    pub(crate) fn items_until(&mut self, close: Option<&str>) -> PResult<'a> {
        loop {
            match close {
                Some(c) if self.at(c) => return Ok(()),
                None if self.at_eof() => return Ok(()),
                _ if self.at_eof() => return self.error(&format!("expected `{}`", close.unwrap())),
                _ => {}
            }
            if self.eat(";") {
                // A stray `;` between items.
                continue;
            }
            self.item()?;
        }
    }

    /// One item, with its attributes.
    pub(crate) fn item(&mut self) -> PResult<'a> {
        let m = self.mark();
        self.outer_attributes()?;
        self.item_from(m)
    }

    /// The rest of an item whose attributes have been read under `m`.
    pub(crate) fn item_from(&mut self, m: Mark) -> PResult<'a> {
        self.visibility()?;
        let restriction =
            matches!(self.tok(2).0, "crate" | "self" | "super") && self.at_n(3, ")") || self.at_n(2, "in");
        if self.at("impl") && self.at_n(1, "(") && restriction {
            // `impl(crate) trait`: where the trait may be implemented.
            let rm = self.mark();
            self.bump_n(2);
            if self.eat("in") || matches!(self.tok(0).0, "crate" | "self" | "super") {
                self.path(PathMode::Simple)?;
            }
            self.expect(")")?;
            self.finish(rm, Kind::ImplRestriction);
        }
        let Some(kind) = self.item_kind() else { return self.error("expected an item") };
        match kind {
            ItemKind::Fn => self.fn_item(m),
            ItemKind::Struct => self.struct_item(m),
            ItemKind::Union => self.union_item(m),
            ItemKind::Enum => self.enum_item(m),
            ItemKind::Trait => self.trait_item(m),
            ItemKind::Impl => self.impl_item(m),
            ItemKind::Mod => self.mod_item(m),
            ItemKind::Use => self.use_item(m),
            ItemKind::Const => self.const_item(m),
            ItemKind::Static => self.static_item(m),
            ItemKind::Type => self.type_item(m),
            ItemKind::ExternCrate => self.extern_crate(m),
            ItemKind::ExternBlock => self.extern_block(m),
            ItemKind::MacroRules => self.macro_rules(m),
            ItemKind::MacroDef => self.macro_def(m),
            ItemKind::MacroCall => self.item_macro_call(m),
        }
    }

    /// Whether a statement starting here (after attributes) is an item.
    pub(crate) fn item_start(&self) -> bool {
        if self.at("pub") {
            return true;
        }
        match self.tok(0).0 {
            // Blocks and closures, not items.
            "unsafe" | "const" if self.at_n(1, "{") => return false,
            "static" if matches!(self.tok(1).0, "|" | "move" | "async") => return false,
            "async" if !matches!(self.tok(1).0, "fn" | "unsafe" | "extern") => return false,
            "const" if matches!(self.tok(1).0, "|" | "move" | "async") && !self.at_n(2, "fn") => return false,
            _ => {}
        }
        // A macro call in statement position is an expression (or a
        // statement), handled there; only definitions are items.
        !matches!(self.item_kind(), None | Some(ItemKind::MacroCall))
    }

    /// The number of qualifier tokens from token `n`.
    fn qualifiers_len(&self, mut n: usize) -> usize {
        let start = n;
        loop {
            let t = self.tok(n).0;
            let next = self.tok(n + 1).0;
            let weak_ok = matches!(
                next,
                "fn" | "const"
                    | "async"
                    | "unsafe"
                    | "safe"
                    | "extern"
                    | "type"
                    | "impl"
                    | "static"
                    | "trait"
                    | "default"
            );
            match t {
                "const" | "async" | "unsafe" => n += 1,
                "extern" => {
                    n += 1;
                    if self.tok(n).is_string() {
                        n += 1;
                    }
                }
                "default" | "safe" if weak_ok => n += 1,
                "final" if weak_ok => n += 1,
                "auto" if next == "trait" => n += 1,
                "gen" if next == "fn" => n += 1,
                _ => return n - start,
            }
        }
    }

    /// Which item follows, looking past qualifiers.
    fn item_kind(&self) -> Option<ItemKind> {
        let q = self.qualifiers_len(0);
        let t = self.tok(q).0;
        let has = |s: &str| (0..q).any(|k| self.at_n(k, s));
        Some(match t {
            "fn" => ItemKind::Fn,
            "struct" => ItemKind::Struct,
            "enum" => ItemKind::Enum,
            "trait" => ItemKind::Trait,
            "impl" => ItemKind::Impl,
            "mod" => ItemKind::Mod,
            "use" => ItemKind::Use,
            "static" => ItemKind::Static,
            "type" => ItemKind::Type,
            "crate" if has("extern") => ItemKind::ExternCrate,
            "{" if has("extern") => ItemKind::ExternBlock,
            "union" if self.name_at(q + 1) => ItemKind::Union,
            "macro_rules" if self.at_n(q + 1, "!") && self.name_at(q + 2) => ItemKind::MacroRules,
            "macro" if self.name_at(q + 1) => ItemKind::MacroDef,
            _ if q > 0 && self.at_n(q - 1, "const") && (self.name_at(q) || self.at_n(q, "_")) => ItemKind::Const,
            _ if q == 0 && self.path_start_at(0) => {
                // `path! (...)` or `path! { }`.
                let mut n = 0;
                if self.at_op("::") {
                    n += 2;
                }
                while self.segment_at(n) {
                    n += if self.at_n(n, "$") { 2 } else { 1 };
                    if self.at_op_n(n, "::") {
                        n += 2;
                    } else {
                        break;
                    }
                }
                if self.at_n(n, "!") && matches!(self.tok(n + 1).0, "(" | "[" | "{") {
                    ItemKind::MacroCall
                } else {
                    return None;
                }
            }
            _ => return None,
        })
    }

    /// The qualifiers, as `Keyword` children (and `Abi` after `extern`).
    fn qualifiers(&mut self) {
        let q = self.qualifiers_len(0);
        let end = self.pos + q;
        while self.pos < end {
            if self.tok(0).is_string() {
                self.leaf(Kind::Abi, 1);
            } else {
                self.leaf(Kind::Keyword, 1);
            }
        }
    }

    /// `pub`, `pub(crate)`, `pub(self)`, `pub(super)`, `pub(in path)`.
    pub(crate) fn visibility(&mut self) -> PResult<'a> {
        if !self.at("pub") {
            return Ok(());
        }
        let m = self.mark();
        self.bump();
        if self.at("(") {
            if matches!(self.tok(1).0, "crate" | "self" | "super") && self.at_n(2, ")") {
                self.bump();
                self.path(PathMode::Simple)?;
                self.bump();
            } else if self.at_n(1, "in") {
                self.bump_n(2);
                self.path(PathMode::Simple)?;
                self.expect(")")?;
            }
        }
        self.finish(m, Kind::Visibility);
        Ok(())
    }

    // ---- functions ----

    fn fn_item(&mut self, m: Mark) -> PResult<'a> {
        self.qualifiers();
        self.expect("fn")?;
        self.name()?;
        self.generics()?;
        self.params(true)?;
        if self.at_op("->") {
            self.ret_type(true)?;
        }
        self.where_clause()?;
        if !self.eat(";") {
            self.block_expr()?;
        }
        self.finish(m, Kind::Fn);
        Ok(())
    }

    /// `(self, a: A, B, ...)`. A parameter's pattern may be left out (in
    /// a `fn` type, or a trait method in the 2015 edition).
    pub(crate) fn params(&mut self, self_allowed: bool) -> PResult<'a> {
        let m = self.mark();
        self.expect("(")?;
        let mut first = true;
        while !self.at(")") {
            let pm = self.mark();
            self.outer_attributes()?;
            if first && self_allowed && self.self_param_at() {
                self.self_param()?;
                self.finish(pm, Kind::SelfParam);
            } else if self.at_op("...") {
                self.bump_n(3);
                self.finish(pm, Kind::Variadic);
            } else {
                self.param(pm)?;
            }
            first = false;
            if !self.eat(",") {
                break;
            }
        }
        self.expect(")")?;
        self.finish(m, Kind::Params);
        Ok(())
    }

    fn self_param_at(&self) -> bool {
        let is_self = |n: usize| self.at_n(n, "self") && !self.at_op_n(n + 1, "::");
        if is_self(0) || self.at("mut") && is_self(1) {
            return true;
        }
        if !self.at("&") {
            return false;
        }
        let n = if self.lifetime_at(1) { 2 } else { 1 };
        is_self(n) || self.at_n(n, "mut") && is_self(n + 1)
    }

    fn self_param(&mut self) -> PResult<'a> {
        if self.at("&") {
            self.leaf(Kind::Op, 1);
            if self.lifetime_at(0) {
                self.leaf(Kind::Lifetime, 1);
            }
        }
        if self.at("mut") {
            self.leaf(Kind::Keyword, 1);
        }
        self.leaf(Kind::Name, 1);
        if self.eat(":") {
            self.ty()?;
        }
        Ok(())
    }

    /// `pattern: Type`, `name: ...`, or a bare `Type`.
    fn param(&mut self, pm: Mark) -> PResult<'a> {
        let s = self.save();
        if self.pattern_no_alt().is_ok() && self.at(":") && !self.at_op("::") {
            self.bump();
            if self.at_op("...") {
                self.bump_n(3);
                self.finish(pm, Kind::Variadic);
                return Ok(());
            }
            self.ty()?;
            self.finish(pm, Kind::Param);
            return Ok(());
        }
        self.restore(s);
        self.ty()?;
        self.finish(pm, Kind::Param);
        Ok(())
    }

    // ---- types ----

    fn struct_item(&mut self, m: Mark) -> PResult<'a> {
        self.expect("struct")?;
        self.name()?;
        self.generics()?;
        self.where_clause()?;
        if self.at("{") {
            self.named_fields()?;
        } else if self.at("(") {
            self.tuple_fields()?;
            self.where_clause()?;
            self.expect(";")?;
        } else {
            self.expect(";")?;
        }
        self.finish(m, Kind::Struct);
        Ok(())
    }

    fn union_item(&mut self, m: Mark) -> PResult<'a> {
        self.expect("union")?;
        self.name()?;
        self.generics()?;
        self.where_clause()?;
        self.named_fields()?;
        self.finish(m, Kind::Union);
        Ok(())
    }

    /// `{ a: A, pub b: B = default }`.
    fn named_fields(&mut self) -> PResult<'a> {
        let m = self.mark();
        self.expect("{")?;
        while !self.at("}") {
            let fm = self.mark();
            self.outer_attributes()?;
            self.visibility()?;
            if self.at("unsafe") {
                self.leaf(Kind::Keyword, 1);
            }
            self.name()?;
            self.expect(":")?;
            self.ty()?;
            if self.eat("=") {
                self.expr()?;
            }
            self.finish(fm, Kind::FieldDef);
            if !self.eat(",") {
                break;
            }
        }
        self.expect("}")?;
        self.finish(m, Kind::NamedFields);
        Ok(())
    }

    /// `(pub A, B)`.
    fn tuple_fields(&mut self) -> PResult<'a> {
        let m = self.mark();
        self.expect("(")?;
        while !self.at(")") {
            let fm = self.mark();
            self.outer_attributes()?;
            self.visibility()?;
            self.ty()?;
            self.finish(fm, Kind::TupleField);
            if !self.eat(",") {
                break;
            }
        }
        self.expect(")")?;
        self.finish(m, Kind::TupleFields);
        Ok(())
    }

    fn enum_item(&mut self, m: Mark) -> PResult<'a> {
        self.expect("enum")?;
        self.name()?;
        self.generics()?;
        self.where_clause()?;
        self.expect("{")?;
        while !self.at("}") {
            let vm = self.mark();
            self.outer_attributes()?;
            self.visibility()?;
            self.name()?;
            if self.at("{") {
                self.named_fields()?;
            } else if self.at("(") {
                self.tuple_fields()?;
            }
            if self.eat("=") {
                self.expr()?;
            }
            self.finish(vm, Kind::Variant);
            if !self.eat(",") {
                break;
            }
        }
        self.expect("}")?;
        self.finish(m, Kind::Enum);
        Ok(())
    }

    fn type_item(&mut self, m: Mark) -> PResult<'a> {
        self.qualifiers();
        self.expect("type")?;
        self.name()?;
        self.generics()?;
        if self.eat(":") {
            self.bounds(true)?;
        }
        self.where_clause()?;
        if self.eat("=") {
            self.ty()?;
        }
        self.where_clause()?;
        self.expect(";")?;
        self.finish(m, Kind::TypeAlias);
        Ok(())
    }

    // ---- traits and impls ----

    fn trait_item(&mut self, m: Mark) -> PResult<'a> {
        self.qualifiers();
        self.expect("trait")?;
        self.name()?;
        self.generics()?;
        if self.at("=") {
            self.bump();
            self.bounds(true)?;
            self.where_clause()?;
            self.expect(";")?;
            self.finish(m, Kind::TraitAlias);
            return Ok(());
        }
        if self.eat(":") {
            self.bounds(true)?;
        }
        self.where_clause()?;
        self.item_body()?;
        self.finish(m, Kind::Trait);
        Ok(())
    }

    /// `{ #![inner] items }` of a trait, impl, module or extern block.
    fn item_body(&mut self) -> PResult<'a> {
        self.expect("{")?;
        self.inner_attributes()?;
        self.items_until(Some("}"))?;
        self.expect("}")
    }

    fn impl_item(&mut self, m: Mark) -> PResult<'a> {
        self.qualifiers();
        self.expect("impl")?;
        if self.generics_at(0) {
            self.generics()?;
        }
        if self.at("const") {
            self.leaf(Kind::Keyword, 1);
        }
        // The trait, or the self type of an inherent impl.
        let tm = self.mark();
        let negative = self.at("!") && self.path_start_at(1);
        if negative {
            self.leaf(Kind::Op, 1);
        }
        self.ty()?;
        if self.eat("for") {
            self.finish(tm, Kind::ImplTrait);
            self.ty()?;
        } else if negative {
            return self.error("expected `for`");
        }
        self.where_clause()?;
        self.item_body()?;
        self.finish(m, Kind::Impl);
        Ok(())
    }

    // ---- modules and imports ----

    fn mod_item(&mut self, m: Mark) -> PResult<'a> {
        self.qualifiers();
        self.expect("mod")?;
        self.name()?;
        if !self.eat(";") {
            self.item_body()?;
        }
        self.finish(m, Kind::Mod);
        Ok(())
    }

    fn use_item(&mut self, m: Mark) -> PResult<'a> {
        self.expect("use")?;
        self.use_tree()?;
        self.expect(";")?;
        self.finish(m, Kind::Use);
        Ok(())
    }

    /// `a::b::{c, d::*}`, `a as b`, `::a`, `*`.
    fn use_tree(&mut self) -> PResult<'a> {
        let m = self.mark();
        if self.at_op("::") {
            self.leaf(Kind::PathRoot, 2);
        }
        if self.segment_at(0) {
            let pm = self.mark();
            loop {
                let sm = self.mark();
                self.leaf(Kind::Name, if self.at("$") { 2 } else { 1 });
                self.finish(sm, Kind::Segment);
                if self.at_op("::") && self.segment_at(2) {
                    self.bump_n(2);
                } else {
                    break;
                }
            }
            self.finish(pm, Kind::Path);
            if self.at("as") {
                let rm = self.mark();
                self.bump();
                if self.at("_") {
                    self.leaf(Kind::Name, 1);
                } else {
                    self.name()?;
                }
                self.finish(rm, Kind::Rename);
                self.finish(m, Kind::UseTree);
                return Ok(());
            }
            if !self.eat_op("::") {
                self.finish(m, Kind::UseTree);
                return Ok(());
            }
        }
        if self.at("*") {
            self.leaf(Kind::UseGlob, 1);
        } else if self.at("{") {
            let gm = self.mark();
            self.bump();
            while !self.at("}") {
                self.use_tree()?;
                if !self.eat(",") {
                    break;
                }
            }
            self.expect("}")?;
            self.finish(gm, Kind::UseGroup);
        } else {
            return self.error("expected a path, `*` or `{`");
        }
        self.finish(m, Kind::UseTree);
        Ok(())
    }

    fn extern_crate(&mut self, m: Mark) -> PResult<'a> {
        self.expect("extern")?;
        self.expect("crate")?;
        if self.at("self") {
            self.leaf(Kind::Name, 1);
        } else {
            self.name()?;
        }
        if self.at("as") {
            let rm = self.mark();
            self.bump();
            if self.at("_") {
                self.leaf(Kind::Name, 1);
            } else {
                self.name()?;
            }
            self.finish(rm, Kind::Rename);
        }
        self.expect(";")?;
        self.finish(m, Kind::ExternCrate);
        Ok(())
    }

    fn extern_block(&mut self, m: Mark) -> PResult<'a> {
        if self.at("unsafe") {
            self.leaf(Kind::Keyword, 1);
        }
        self.expect("extern")?;
        if self.tok(0).is_string() {
            self.leaf(Kind::Abi, 1);
        }
        self.item_body()?;
        self.finish(m, Kind::ExternBlock);
        Ok(())
    }

    // ---- constants and statics ----

    fn const_item(&mut self, m: Mark) -> PResult<'a> {
        // Any qualifiers before `const` (`default`), then `const` itself,
        // which only introduces the item.
        let end = self.pos + self.qualifiers_len(0);
        while self.pos < end && !self.at("const") {
            self.leaf(Kind::Keyword, 1);
        }
        self.expect("const")?;
        if self.at("_") {
            self.leaf(Kind::Name, 1);
        } else {
            self.name()?;
        }
        self.generics()?;
        if self.eat(":") {
            self.ty()?;
        }
        if self.eat("=") {
            self.expr()?;
        }
        self.where_clause()?;
        self.expect(";")?;
        self.finish(m, Kind::Const);
        Ok(())
    }

    fn static_item(&mut self, m: Mark) -> PResult<'a> {
        self.qualifiers();
        self.expect("static")?;
        if self.at("mut") {
            self.leaf(Kind::Keyword, 1);
        }
        self.name()?;
        self.expect(":")?;
        self.ty()?;
        if self.eat("=") {
            self.expr()?;
        }
        self.expect(";")?;
        self.finish(m, Kind::Static);
        Ok(())
    }

    // ---- macros ----

    fn macro_rules(&mut self, m: Mark) -> PResult<'a> {
        self.expect("macro_rules")?;
        self.expect("!")?;
        self.name()?;
        let braces = self.at("{");
        self.token_tree()?;
        if !braces {
            self.expect(";")?;
        }
        self.finish(m, Kind::MacroRules);
        Ok(())
    }

    fn macro_def(&mut self, m: Mark) -> PResult<'a> {
        self.expect("macro")?;
        self.name()?;
        if self.at("(") {
            self.token_tree()?;
        }
        if !self.at("{") {
            return self.error("expected `{`");
        }
        self.token_tree()?;
        self.finish(m, Kind::MacroDef);
        Ok(())
    }

    fn item_macro_call(&mut self, m: Mark) -> PResult<'a> {
        self.path(PathMode::Simple)?;
        self.expect("!")?;
        let braces = self.at("{");
        self.token_tree()?;
        if !braces {
            self.expect(";")?;
        }
        self.finish(m, Kind::MacroCall);
        Ok(())
    }
}
