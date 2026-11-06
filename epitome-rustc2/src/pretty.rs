//! Printing a syntax tree back as Rust source, in a fixed style.
//!
//! The output parses to the same tree (`same_tree`), which checks the
//! parser and the printer against each other over large inputs. Parentheses
//! are in the tree as `Paren` and `ParenType` nodes, so printing never has
//! to add any.
//!
//! What is not in a node's children, the printer finds from its tokens: a
//! child is a type if the token before it is `:`, a value if it is `=`,
//! and so on. Token trees (macro input) are printed as the source text
//! they cover, so their spacing and comments are kept.

use crate::ast::{Ast, Kind, NodeId};

/// Prints the whole tree.
pub fn pretty(ast: &Ast) -> String {
    let mut p = Printer { ast, out: String::new(), indent: 0 };
    p.node(ast.root);
    if !p.out.ends_with('\n') {
        p.out.push('\n');
    }
    p.out
}

/// One node (an item, say) as source text.
pub fn pretty_node(ast: &Ast, id: NodeId) -> String {
    let mut p = Printer { ast, out: String::new(), indent: 0 };
    p.node(id);
    p.out
}

/// Whether two trees have the same kinds, shapes and leaf tokens.
pub fn same_tree(a: &Ast, ia: NodeId, b: &Ast, ib: NodeId) -> bool {
    if a.kind(ia) != b.kind(ib) {
        return false;
    }
    if a.kind(ia).is_leaf() {
        let ta = a.tokens_of(ia).iter().map(|t| t.0);
        let tb = b.tokens_of(ib).iter().map(|t| t.0);
        if !ta.eq(tb) {
            return false;
        }
    }
    let (ca, cb) = (a.children(ia), b.children(ib));
    ca.len() == cb.len() && ca.iter().zip(cb).all(|(&x, &y)| same_tree(a, x, b, y))
}

/// The first place two trees differ, as the two subtrees' s-expressions.
pub fn first_difference(a: &Ast, ia: NodeId, b: &Ast, ib: NodeId) -> Option<(String, String)> {
    if same_tree(a, ia, b, ib) {
        return None;
    }
    let (ca, cb) = (a.children(ia), b.children(ib));
    if a.kind(ia) == b.kind(ib) && ca.len() == cb.len() {
        for (&x, &y) in ca.iter().zip(cb) {
            if let Some(d) = first_difference(a, x, b, y) {
                return Some(d);
            }
        }
    }
    Some((a.sexp(ia), b.sexp(ib)))
}

fn is_type(k: Kind) -> bool {
    use Kind::*;
    matches!(
        k,
        PathType
            | RefType
            | PtrType
            | ArrayType
            | SliceType
            | TupleType
            | ParenType
            | PatternType
            | NeverType
            | InferType
            | FnPtrType
            | ImplTraitType
            | DynTraitType
    )
}

const PREC_RANGE: u8 = 2;
const PREC_CAST: u8 = 12;
const PREC_PREFIX: u8 = 13;
const PREC_POSTFIX: u8 = 14;

/// A binary operator's precedence (1, assignment, to 11, `*`), and whether
/// it groups right to left.
fn binary_prec(op: &str) -> (u8, bool) {
    match op {
        "=" | "+=" | "-=" | "*=" | "/=" | "%=" | "^=" | "&=" | "|=" | "<<=" | ">>=" => (1, true),
        "||" => (3, false),
        "&&" => (4, false),
        "==" | "!=" | "<" | ">" | "<=" | ">=" => (5, false),
        "|" => (6, false),
        "^" => (7, false),
        "&" => (8, false),
        "<<" | ">>" => (9, false),
        "+" | "-" => (10, false),
        _ => (11, false),
    }
}

/// Whether a token is not from any source text, but made by expansion
/// (the parentheses around a transcribed `$e:expr`).
fn is_synthetic(t: crate::lexer::Token) -> bool {
    crate::expand::is_synthetic(t)
}

/// A type, or a macro call standing for one.
fn is_type_like(k: Kind) -> bool {
    is_type(k) || k == Kind::TypeMacro
}

fn is_attr(k: Kind) -> bool {
    matches!(k, Kind::Attribute | Kind::DocComment)
}

struct Printer<'p, 'a> {
    ast: &'p Ast<'a>,
    out: String,
    indent: usize,
}

impl<'p, 'a> Printer<'p, 'a> {
    fn s(&mut self, s: &str) {
        self.out.push_str(s);
    }

    /// A new line at the current indent.
    fn nl(&mut self) {
        self.out.push('\n');
        for _ in 0..self.indent {
            self.out.push_str("    ");
        }
    }

    fn kind(&self, id: NodeId) -> Kind {
        self.ast.kind(id)
    }

    fn kids(&self, id: NodeId) -> &'p [NodeId] {
        self.ast.children(id)
    }

    /// The text of the token just before a node.
    fn before(&self, id: NodeId) -> &'a str {
        let first = self.ast.node(id).first as usize;
        if first == 0 { "" } else { self.ast.tokens[first - 1].0 }
    }

    /// A leaf's tokens as written, joined where they touched.
    fn leaf(&mut self, id: NodeId) {
        let text = self.ast.text(id);
        self.s(&text);
    }

    /// The source text a node covers, verbatim. Tokens made by macro
    /// expansion are not one piece of text, so they are joined instead:
    /// with a space between tokens that did not touch, and a new line
    /// after a line comment.
    fn verbatim(&mut self, id: NodeId) {
        let toks = self.ast.tokens_of(id);
        if !self.ast.is_original(id) {
            let mut prev: Option<crate::lexer::Token> = None;
            // The parentheses expansion puts around a transcribed
            // expression stay in the tokens (so the expression stays one
            // when the tokens are expanded again), but are not printed, as
            // rustc does not print them.
            for &t in toks.iter().filter(|t| !is_synthetic(**t)) {
                if let Some(p) = prev {
                    if p.0.starts_with("//") {
                        self.nl();
                    } else if !p.touches(t) {
                        self.s(" ");
                    }
                }
                self.s(t.0);
                prev = Some(t);
            }
            return;
        }
        let (Some(first), Some(last)) = (toks.first(), toks.last()) else { return };
        let start = first.0.as_ptr() as usize;
        let end = last.0.as_ptr() as usize + last.0.len();
        // SAFETY: tokens of one tree are slices of one source text, in
        // order, so the bytes between the first's start and the last's end
        // are a slice of that text (on character boundaries).
        let text = unsafe { std::str::from_utf8_unchecked(std::slice::from_raw_parts(first.0.as_ptr(), end - start)) };
        self.s(text);
    }

    /// Attributes, each on a line of its own; returns the other children.
    fn attrs_lines(&mut self, kids: &'p [NodeId]) -> &'p [NodeId] {
        let n = kids.iter().take_while(|&&k| is_attr(self.kind(k))).count();
        for &k in &kids[..n] {
            self.node(k);
            self.nl();
        }
        &kids[n..]
    }

    /// Attributes before something on the same line; returns the others.
    fn attrs_inline(&mut self, kids: &'p [NodeId]) -> &'p [NodeId] {
        let n = kids.iter().take_while(|&&k| is_attr(self.kind(k))).count();
        for &k in &kids[..n] {
            self.node(k);
            if self.kind(k) == Kind::DocComment {
                self.nl();
            } else {
                self.s(" ");
            }
        }
        &kids[n..]
    }

    fn join(&mut self, kids: &[NodeId], sep: &str) {
        for (i, &k) in kids.iter().enumerate() {
            if i > 0 {
                self.s(sep);
            }
            self.node(k);
        }
    }

    /// `{`, each node on its own line, `}`.
    fn body(&mut self, kids: &[NodeId], sep: &str) {
        if kids.is_empty() {
            self.s("{}");
            return;
        }
        self.s("{");
        self.indent += 1;
        for &k in kids {
            self.nl();
            self.node(k);
            self.s(sep);
        }
        self.indent -= 1;
        self.nl();
        self.s("}");
    }

    /// Inner attributes then items or statements, in braces.
    fn item_body(&mut self, kids: &[NodeId]) {
        if kids.is_empty() {
            self.s(" {}");
            return;
        }
        self.s(" {");
        self.indent += 1;
        for &k in kids {
            self.nl();
            self.node(k);
        }
        self.indent -= 1;
        self.nl();
        self.s("}");
    }

    /// Whether a `Paren` node was made by macro expansion (to keep an
    /// expansion one expression) rather than written: either it covers no
    /// `(` of its own, or its `(` is not from any source text.
    fn is_synthetic_paren(&self, id: NodeId) -> bool {
        let first = self.ast.token(id);
        first.0 != "(" || is_synthetic(first)
    }

    /// An operand that must bind at least as tightly as `min`: a
    /// parenthesis made by expansion is printed only if it is needed.
    fn operand(&mut self, id: NodeId, min: u8) {
        if self.kind(id) == Kind::Paren && self.is_synthetic_paren(id) {
            let inner = self.kids(id)[0];
            if self.expr_prec(inner) < min {
                self.s("(");
                self.node(inner);
                self.s(")");
            } else {
                self.operand(inner, min);
            }
        } else {
            self.node(id);
        }
    }

    /// How tightly an expression binds, on the scale of `binary_prec`.
    fn expr_prec(&self, id: NodeId) -> u8 {
        use Kind::*;
        match self.kind(id) {
            Binary => binary_prec(&self.ast.text(self.kids(id)[1])).0,
            Range => PREC_RANGE,
            Cast => PREC_CAST,
            Unary | Borrow => PREC_PREFIX,
            Closure | Return | Break | Yield | Become | Yeet | Let => 0,
            Paren if self.is_synthetic_paren(id) => self.expr_prec(self.kids(id)[0]),
            _ => PREC_POSTFIX,
        }
    }

    /// A path, with `::<` before generic arguments in expressions and
    /// patterns.
    fn path(&mut self, id: NodeId, turbofish: bool) {
        let mut first = true;
        for &k in self.kids(id) {
            match self.kind(k) {
                Kind::PathRoot => self.s("::"),
                Kind::QSelf => {
                    let q = self.kids(k);
                    self.s("<");
                    self.node(q[0]);
                    if q.len() > 1 {
                        self.s(" as ");
                        self.node(q[1]);
                    }
                    self.s(">::");
                }
                _ => {
                    if !first {
                        self.s("::");
                    }
                    first = false;
                    self.segment(k, turbofish);
                }
            }
        }
    }

    fn segment(&mut self, id: NodeId, turbofish: bool) {
        for &k in self.kids(id) {
            match self.kind(k) {
                Kind::GenericArgs | Kind::FnSugar if turbofish => {
                    self.s("::");
                    self.node(k);
                }
                _ => self.node(k),
            }
        }
    }

    /// The child that is a path, printed with or without turbofish; the
    /// other children follow.
    fn path_child(&mut self, id: NodeId, turbofish: bool) -> &'p [NodeId] {
        let kids = self.kids(id);
        self.path(kids[0], turbofish);
        &kids[1..]
    }

    /// Whether the first bound of a bound list is a path starting `::`.
    fn starts_with_root(&self, bounds: NodeId) -> bool {
        let Some(&b) = self.kids(bounds).first() else { return false };
        let Some(&p) = self.kids(b).last() else { return false };
        self.kind(b) == Kind::TraitBound
            && self.kind(p) == Kind::Path
            && self.kids(p).first().is_some_and(|&r| self.kind(r) == Kind::PathRoot)
    }

    fn label_and_keywords(&mut self, kids: &'p [NodeId]) -> &'p [NodeId] {
        let mut rest = kids;
        while let Some((&k, tail)) = rest.split_first() {
            match self.kind(k) {
                Kind::Lifetime => {
                    self.leaf(k);
                    self.s(": ");
                }
                Kind::Keyword => {
                    self.leaf(k);
                    self.s(" ");
                }
                _ => break,
            }
            rest = tail;
        }
        rest
    }

    fn block(&mut self, id: NodeId) {
        let kids = self.kids(id);
        let rest = self.attrs_inline(kids);
        let rest = self.label_and_keywords(rest);
        self.item_like_body(rest);
    }

    /// `{ inner attributes, statements }` with no leading space.
    fn item_like_body(&mut self, kids: &[NodeId]) {
        if kids.is_empty() {
            self.s("{}");
            return;
        }
        self.s("{");
        self.indent += 1;
        for &k in kids {
            self.nl();
            self.node(k);
        }
        self.indent -= 1;
        self.nl();
        self.s("}");
    }

    fn node(&mut self, id: NodeId) {
        use Kind::*;
        let kids = self.kids(id);
        match self.kind(id) {
            Name | Lifetime | Literal | Op | Keyword | Abi => self.leaf(id),
            // Always followed by a new line where it is printed, as a `///`
            // comment runs to the end of its line.
            DocComment => self.leaf(id),
            TokenTree => self.verbatim(id),

            Crate => {
                for (i, &k) in kids.iter().enumerate() {
                    if i > 0 {
                        self.nl();
                        if !matches!(self.kind(k), InnerAttribute | DocComment) {
                            self.nl();
                        }
                    }
                    self.node(k);
                }
            }
            Attribute | InnerAttribute => {
                self.s(if self.kind(id) == Attribute { "#[" } else { "#![" });
                let mut rest = kids;
                let is_unsafe = self.kind(rest[0]) == Keyword;
                if is_unsafe {
                    self.s("unsafe(");
                    rest = &rest[1..];
                }
                self.path(rest[0], false);
                if let Some(&v) = rest.get(1) {
                    if self.kind(v) == TokenTree {
                        self.node(v);
                    } else {
                        self.s(" = ");
                        self.node(v);
                    }
                }
                if is_unsafe {
                    self.s(")");
                }
                self.s("]");
            }
            Visibility => {
                self.s("pub");
                if let Some(&p) = kids.first() {
                    self.s(if self.before(p) == "in" { "(in " } else { "(" });
                    self.path(p, false);
                    self.s(")");
                }
            }
            ImplRestriction => {
                self.s("impl(");
                if let Some(&p) = kids.first() {
                    if self.before(p) == "in" {
                        self.s("in ");
                    }
                    self.path(p, false);
                }
                self.s(")");
            }

            // ---- items ----
            Fn => {
                let rest = self.attrs_lines(kids);
                let mut body = false;
                for &k in rest {
                    match self.kind(k) {
                        Visibility | Keyword | Abi | ImplRestriction => {
                            self.node(k);
                            self.s(" ");
                        }
                        Name => {
                            self.s("fn ");
                            self.node(k);
                        }
                        RetType | WhereClause => {
                            self.s(" ");
                            self.node(k);
                        }
                        Block => {
                            self.s(" ");
                            self.node(k);
                            body = true;
                        }
                        _ => self.node(k),
                    }
                }
                if !body {
                    self.s(";");
                }
            }
            Struct | Union => {
                let rest = self.attrs_lines(kids);
                let mut braces = false;
                for &k in rest {
                    match self.kind(k) {
                        Visibility => {
                            self.node(k);
                            self.s(" ");
                        }
                        Name => {
                            self.s(if self.kind(id) == Struct { "struct " } else { "union " });
                            self.node(k);
                        }
                        WhereClause | NamedFields => {
                            self.s(" ");
                            self.node(k);
                            braces |= self.kind(k) == NamedFields;
                        }
                        _ => self.node(k),
                    }
                }
                if !braces {
                    self.s(";");
                }
            }
            Enum => {
                let rest = self.attrs_lines(kids);
                let mut variants = Vec::new();
                for &k in rest {
                    match self.kind(k) {
                        Visibility => {
                            self.node(k);
                            self.s(" ");
                        }
                        Name => {
                            self.s("enum ");
                            self.node(k);
                        }
                        WhereClause => {
                            self.s(" ");
                            self.node(k);
                        }
                        Variant => variants.push(k),
                        _ => self.node(k),
                    }
                }
                self.s(" ");
                self.body(&variants, ",");
            }
            Variant | FieldDef => {
                let rest = self.attrs_lines(kids);
                // A field's type, then its default; a variant's fields,
                // then its discriminant.
                let mut typed = self.kind(id) == Variant;
                for &k in rest {
                    match self.kind(k) {
                        Visibility | Keyword => {
                            self.node(k);
                            self.s(" ");
                        }
                        Name | TupleFields => self.node(k),
                        NamedFields => {
                            self.s(" ");
                            self.node(k);
                        }
                        _ if !typed => {
                            self.s(": ");
                            self.node(k);
                            typed = true;
                        }
                        _ => {
                            self.s(" = ");
                            self.node(k);
                        }
                    }
                }
            }
            NamedFields => self.body(kids, ","),
            TupleFields => {
                self.s("(");
                self.join(kids, ", ");
                self.s(")");
            }
            TupleField => {
                let rest = self.attrs_inline(kids);
                for &k in rest {
                    self.node(k);
                    if self.kind(k) == Visibility {
                        self.s(" ");
                    }
                }
            }
            Trait | TraitAlias => {
                let rest = self.attrs_lines(kids);
                let mut items = Vec::new();
                for &k in rest {
                    match self.kind(k) {
                        Visibility | Keyword | ImplRestriction => {
                            self.node(k);
                            self.s(" ");
                        }
                        Name => {
                            self.s("trait ");
                            self.node(k);
                        }
                        Bounds => {
                            self.s(if self.kind(id) == Trait { ": " } else { " = " });
                            self.node(k);
                        }
                        Generics => self.node(k),
                        WhereClause => {
                            self.s(" ");
                            self.node(k);
                        }
                        _ => items.push(k),
                    }
                }
                if self.kind(id) == Trait {
                    self.item_body(&items);
                } else {
                    self.s(";");
                }
            }
            Impl => {
                let rest = self.attrs_lines(kids);
                let mut items = Vec::new();
                let mut said_impl = false;
                let mut self_done = false;
                // Qualifiers may come before `impl` (`unsafe impl`, `const
                // impl`) or after it (`impl const`); the tokens say which.
                let start = rest.first().map_or(0, |&k| self.ast.node(k).first as usize);
                let end = self.ast.node(id).end as usize;
                let impl_at = (start..end).find(|&i| self.ast.tokens[i].0 == "impl").unwrap_or(start);
                for &k in rest {
                    let k_kind = self.kind(k);
                    let before_impl = k_kind == Keyword && (self.ast.node(k).first as usize) < impl_at;
                    if !said_impl && !before_impl {
                        self.s("impl");
                        said_impl = true;
                        if k_kind != Generics {
                            self.s(" ");
                        }
                    }
                    match k_kind {
                        Keyword => {
                            self.node(k);
                            self.s(" ");
                        }
                        Generics => {
                            self.node(k);
                            self.s(" ");
                        }
                        WhereClause => {
                            self.s(" ");
                            self.node(k);
                        }
                        ImplTrait => self.node(k),
                        // The self type is the one type in the header; what
                        // follows it (apart from `where`) is the body.
                        _ if !self_done && (is_type(k_kind) || matches!(k_kind, MacroCall | TypeMacro)) => {
                            self.node(k);
                            self_done = true;
                        }
                        _ => items.push(k),
                    }
                }
                self.item_body(&items);
            }
            Mod => {
                let rest = self.attrs_lines(kids);
                let mut items = Vec::new();
                let mut inline = false;
                for &k in rest {
                    match self.kind(k) {
                        Visibility | Keyword => {
                            self.node(k);
                            self.s(" ");
                        }
                        Name => {
                            self.s("mod ");
                            self.node(k);
                            inline = self.ast.tokens_of(id).last().is_some_and(|t| t.0 == "}");
                        }
                        _ => items.push(k),
                    }
                }
                if inline {
                    self.item_body(&items);
                } else {
                    self.s(";");
                }
            }
            Use => {
                let rest = self.attrs_lines(kids);
                for &k in rest {
                    if self.kind(k) == Visibility {
                        self.node(k);
                        self.s(" ");
                    } else {
                        self.s("use ");
                        self.node(k);
                    }
                }
                self.s(";");
            }
            UseTree => {
                let mut had_path = false;
                for &k in kids {
                    match self.kind(k) {
                        PathRoot => self.s("::"),
                        Path => {
                            self.path(k, false);
                            had_path = true;
                        }
                        UseGlob | UseGroup => {
                            if had_path {
                                self.s("::");
                            }
                            self.node(k);
                        }
                        _ => self.node(k),
                    }
                }
            }
            UseGlob => self.s("*"),
            UseGroup => {
                self.s("{");
                self.join(kids, ", ");
                self.s("}");
            }
            Rename => {
                self.s(" as ");
                self.join(kids, "");
            }
            Const | Static | TypeAlias => {
                let rest = self.attrs_lines(kids);
                let word = match self.kind(id) {
                    Const => "const ",
                    Static => "static ",
                    _ => "type ",
                };
                let mut said = false;
                let mut typed = false;
                for &k in rest {
                    let kk = self.kind(k);
                    let after_word = kk == Keyword && self.ast.text(k) == "mut";
                    if !said && (kk == Name || after_word) {
                        self.s(word);
                        said = true;
                    }
                    match kk {
                        Visibility | Keyword => {
                            self.node(k);
                            self.s(" ");
                        }
                        Name | Generics => self.node(k),
                        Bounds => {
                            self.s(": ");
                            self.node(k);
                        }
                        WhereClause => {
                            self.s(" ");
                            self.node(k);
                        }
                        // A constant's or static's type; a type alias's
                        // type is its value.
                        _ if !typed && self.kind(id) != TypeAlias && is_type_like(kk) => {
                            self.s(": ");
                            self.node(k);
                            typed = true;
                        }
                        _ => {
                            self.s(" = ");
                            self.node(k);
                        }
                    }
                }
                self.s(";");
            }
            ExternCrate => {
                let rest = self.attrs_lines(kids);
                for &k in rest {
                    match self.kind(k) {
                        Visibility => {
                            self.node(k);
                            self.s(" ");
                        }
                        Name => {
                            self.s("extern crate ");
                            self.node(k);
                        }
                        _ => self.node(k),
                    }
                }
                self.s(";");
            }
            ExternBlock => {
                let rest = self.attrs_lines(kids);
                let mut items = Vec::new();
                let mut said = false;
                for &k in rest {
                    match self.kind(k) {
                        Keyword => {
                            self.node(k);
                            self.s(" ");
                        }
                        Abi => {
                            self.s("extern ");
                            said = true;
                            self.node(k);
                        }
                        _ => items.push(k),
                    }
                }
                if !said {
                    self.s("extern");
                }
                self.item_body(&items);
            }
            MacroRules => {
                let rest = self.attrs_lines(kids);
                self.s("macro_rules! ");
                self.node(rest[0]);
                self.s(" ");
                self.node(rest[1]);
                if !self.ast.token(rest[1]).0.starts_with('{') {
                    self.s(";");
                }
            }
            MacroDef => {
                let rest = self.attrs_lines(kids);
                for &k in rest {
                    match self.kind(k) {
                        Visibility => {
                            self.node(k);
                            self.s(" ");
                        }
                        Name => {
                            self.s("macro ");
                            self.node(k);
                        }
                        _ => {
                            if self.ast.token(k).0 == "{" {
                                self.s(" ");
                            }
                            self.node(k);
                        }
                    }
                }
            }
            MacroCall | TypeMacro | PatMacro => {
                let rest = self.attrs_lines(kids);
                let at_item = self.ast.tokens_of(id).last().is_some_and(|t| t.0 == ";");
                self.path(rest[0], true);
                self.s("!");
                let tt = rest[1];
                if self.ast.token(tt).0 == "{" {
                    self.s(" ");
                }
                self.node(tt);
                if at_item {
                    self.s(";");
                }
            }

            // ---- generics ----
            Generics => {
                self.s("<");
                self.join(kids, ", ");
                self.s(">");
            }
            ForLifetimes => {
                self.s("for<");
                self.join(kids, ", ");
                self.s(">");
            }
            LifetimeParam | TypeParam | ConstParam => {
                let rest = self.attrs_inline(kids);
                if self.kind(id) == ConstParam {
                    self.s("const ");
                }
                // A const parameter's type, then any default.
                let mut typed = self.kind(id) != ConstParam;
                for &k in rest {
                    match self.kind(k) {
                        Bounds => {
                            self.s(": ");
                            self.node(k);
                        }
                        Name | Lifetime => self.node(k),
                        _ if !typed => {
                            self.s(": ");
                            self.node(k);
                            typed = true;
                        }
                        _ => {
                            self.s(" = ");
                            self.node(k);
                        }
                    }
                }
            }
            WhereClause => {
                self.s("where ");
                self.join(kids, ", ");
            }
            WherePredicate => {
                // `T: Bounds`, `'a: 'b`, or `T = U`.
                let mut first = true;
                for &k in kids {
                    match self.kind(k) {
                        ForLifetimes => {
                            self.node(k);
                            self.s(" ");
                        }
                        Bounds => {
                            self.s(": ");
                            self.node(k);
                        }
                        _ if !first => {
                            self.s(" = ");
                            self.node(k);
                        }
                        _ => {
                            self.node(k);
                            first = false;
                        }
                    }
                }
            }
            Bounds => self.join(kids, " + "),
            TraitBound => {
                for &k in kids {
                    match self.kind(k) {
                        ForLifetimes => {
                            self.node(k);
                            self.s(" ");
                        }
                        Keyword => {
                            self.node(k);
                            self.s(" ");
                        }
                        Path => self.path(k, false),
                        _ => self.node(k),
                    }
                }
            }
            UseBound => {
                self.s("use<");
                self.join(kids, ", ");
                self.s(">");
            }

            // ---- functions ----
            Params => {
                self.s("(");
                self.join(kids, ", ");
                self.s(")");
            }
            SelfParam => {
                let rest = self.attrs_inline(kids);
                for &k in rest {
                    match self.kind(k) {
                        Lifetime | Keyword => {
                            self.node(k);
                            self.s(" ");
                        }
                        Name | Op => self.node(k),
                        _ => {
                            self.s(": ");
                            self.node(k);
                        }
                    }
                }
            }
            Param | ClosureParam => {
                let rest = self.attrs_inline(kids);
                self.node(rest[0]);
                if let Some(&t) = rest.get(1) {
                    self.s(": ");
                    self.node(t);
                }
            }
            Variadic => {
                let rest = self.attrs_inline(kids);
                if let Some(&p) = rest.first() {
                    self.node(p);
                    self.s(": ");
                }
                self.s("...");
            }
            RetType => {
                self.s("-> ");
                self.join(kids, "");
            }

            // ---- paths ----
            Path => self.path(id, false),
            PathRoot => self.s("::"),
            QSelf => {}
            Segment => self.segment(id, false),
            GenericArgs => {
                self.s("<");
                self.join(kids, ", ");
                self.s(">");
            }
            FnSugar => {
                self.s("(");
                let mut ret = None;
                let mut args = Vec::new();
                for &k in kids {
                    if self.kind(k) == RetType { ret = Some(k) } else { args.push(k) }
                }
                self.join(&args, ", ");
                self.s(")");
                if let Some(r) = ret {
                    self.s(" ");
                    self.node(r);
                }
            }
            AssocConstraint => {
                for &k in kids {
                    match self.kind(k) {
                        Name | GenericArgs | FnSugar => self.node(k),
                        Bounds => {
                            self.s(": ");
                            self.node(k);
                        }
                        _ => {
                            self.s(" = ");
                            self.node(k);
                        }
                    }
                }
            }

            // ---- types ----
            PathType => self.path(kids[0], false),
            RefType => {
                self.s("&");
                for &k in kids {
                    self.node(k);
                    if matches!(self.kind(k), Lifetime | Keyword) {
                        self.s(" ");
                    }
                }
            }
            PtrType => {
                self.s("*");
                self.node(kids[0]);
                self.s(" ");
                self.node(kids[1]);
            }
            ArrayType | ArrayRepeat => {
                self.s("[");
                self.node(kids[0]);
                self.s("; ");
                self.node(kids[1]);
                self.s("]");
            }
            SliceType => {
                self.s("[");
                self.node(kids[0]);
                self.s("]");
            }
            TupleType | Tuple => {
                self.s("(");
                self.join(kids, ", ");
                if kids.len() == 1 {
                    self.s(",");
                }
                self.s(")");
            }
            Paren if self.is_synthetic_paren(id) => self.node(kids[0]),
            ParenType | Paren | ParenPat => {
                self.s("(");
                self.node(kids[0]);
                self.s(")");
            }
            PatternType => {
                self.s("(");
                self.node(kids[0]);
                self.s(" is ");
                self.node(kids[1]);
                self.s(")");
            }
            NeverType | NeverPat => self.s("!"),
            InferType | WildPat | Underscore => self.s("_"),
            FnPtrType => {
                for &k in kids {
                    match self.kind(k) {
                        ForLifetimes | Keyword | Abi => {
                            self.node(k);
                            self.s(" ");
                        }
                        Params => {
                            self.s("fn");
                            self.node(k);
                        }
                        RetType => {
                            self.s(" ");
                            self.node(k);
                        }
                        _ => self.node(k),
                    }
                }
            }
            ImplTraitType => {
                self.s("impl ");
                self.node(kids[0]);
            }
            DynTraitType => {
                for &k in kids {
                    if self.kind(k) == Keyword {
                        self.node(k);
                        self.s(" ");
                    } else if self.kids(id).len() > 1 && self.starts_with_root(k) {
                        // In 2015, `dyn ::a::Tr` is the path `dyn::a::Tr`;
                        // the bound needs its parentheses.
                        let bounds = self.kids(k);
                        self.s("(");
                        self.node(bounds[0]);
                        self.s(")");
                        for &b in &bounds[1..] {
                            self.s(" + ");
                            self.node(b);
                        }
                    } else {
                        self.node(k);
                    }
                }
            }

            // ---- patterns ----
            RestPat => self.s(".."),
            IdentPat => {
                for &k in kids {
                    match self.kind(k) {
                        Keyword => {
                            self.node(k);
                            self.s(" ");
                        }
                        Name => self.node(k),
                        _ => {
                            self.s(" @ ");
                            self.node(k);
                        }
                    }
                }
            }
            LitPat => self.node(kids[0]),
            RangePat | Range => {
                for &k in kids {
                    if self.kind(k) == Op && self.out.ends_with('.') {
                        // `1. ..2`: a float ending in `.` must not join
                        // the range operator.
                        self.s(" ");
                    }
                    if self.kind(k) == Op {
                        self.node(k);
                    } else {
                        self.operand(k, PREC_RANGE + 1);
                    }
                }
            }
            RefPat => {
                self.s("&");
                for &k in kids {
                    self.node(k);
                    if self.kind(k) == Keyword {
                        self.s(" ");
                    }
                }
            }
            TuplePat => {
                self.s("(");
                self.join(kids, ", ");
                if kids.len() == 1 && self.kind(kids[0]) != RestPat {
                    self.s(",");
                }
                self.s(")");
            }
            SlicePat | Array => {
                self.s("[");
                self.join(kids, ", ");
                self.s("]");
            }
            PathPat | PathExpr => self.path(kids[0], true),
            TupleStructPat => {
                let rest = self.path_child(id, true);
                self.s("(");
                self.join(rest, ", ");
                self.s(")");
            }
            StructPat | StructExpr => {
                let rest = self.path_child(id, true);
                if rest.is_empty() {
                    self.s(" {}");
                } else {
                    self.s(" { ");
                    self.join(rest, ", ");
                    self.s(" }");
                }
            }
            FieldPat | FieldInit => {
                let rest = self.attrs_inline(kids);
                let named = matches!(self.kind(rest[0]), Name | Literal);
                self.node(rest[0]);
                if named && let Some(&v) = rest.get(1) {
                    self.s(": ");
                    self.node(v);
                }
            }
            OrPat => self.join(kids, " | "),
            BoxPat => {
                self.s("box ");
                self.node(kids[0]);
            }

            // ---- expressions ----
            Binary => {
                let (prec, right) = binary_prec(&self.ast.text(kids[1]));
                let (l, r) = if right { (prec + 1, prec) } else { (prec, prec + 1) };
                self.operand(kids[0], l);
                self.s(" ");
                self.node(kids[1]);
                self.s(" ");
                self.operand(kids[2], r);
            }
            Unary => {
                self.node(kids[0]);
                if self.kind(kids[0]) == Keyword {
                    self.s(" ");
                }
                self.operand(kids[1], PREC_PREFIX);
            }
            Borrow => {
                self.s("&");
                for &k in kids {
                    if self.kind(k) == Keyword {
                        self.node(k);
                        self.s(" ");
                    } else {
                        self.operand(k, PREC_PREFIX);
                    }
                }
            }
            Cast => {
                self.operand(kids[0], PREC_CAST);
                self.s(" as ");
                self.node(kids[1]);
            }
            Call => {
                self.operand(kids[0], PREC_POSTFIX);
                self.node(kids[1]);
            }
            Args => {
                self.s("(");
                self.join(kids, ", ");
                self.s(")");
            }
            MethodCall => {
                self.operand(kids[0], PREC_POSTFIX);
                self.s(".");
                self.segment(kids[1], true);
                self.node(kids[2]);
            }
            Field => {
                self.operand(kids[0], PREC_POSTFIX);
                self.s(".");
                self.node(kids[1]);
            }
            Index => {
                self.operand(kids[0], PREC_POSTFIX);
                self.s("[");
                self.node(kids[1]);
                self.s("]");
            }
            Try => {
                self.operand(kids[0], PREC_POSTFIX);
                self.s("?");
            }
            Await => {
                self.operand(kids[0], PREC_POSTFIX);
                self.s(".await");
            }
            UseExpr => {
                self.operand(kids[0], PREC_POSTFIX);
                self.s(".use");
            }
            StructBase => {
                self.s("..");
                self.join(kids, "");
            }
            Block => self.block(id),
            If => {
                self.s("if ");
                self.node(kids[0]);
                self.s(" ");
                self.node(kids[1]);
                if let Some(&e) = kids.get(2) {
                    self.s(" else ");
                    self.node(e);
                }
            }
            Let => {
                self.s("let ");
                self.node(kids[0]);
                self.s(" = ");
                self.node(kids[1]);
            }
            While | Loop | For => {
                let mut rest = kids;
                if self.kind(rest[0]) == Lifetime {
                    self.node(rest[0]);
                    self.s(": ");
                    rest = &rest[1..];
                }
                match self.kind(id) {
                    Loop => self.s("loop "),
                    While => {
                        self.s("while ");
                        self.node(rest[0]);
                        self.s(" ");
                        rest = &rest[1..];
                    }
                    _ => {
                        self.s("for ");
                        if self.kind(rest[0]) == Keyword {
                            self.s("await ");
                            rest = &rest[1..];
                        }
                        self.node(rest[0]);
                        self.s(" in ");
                        self.node(rest[1]);
                        self.s(" ");
                        rest = &rest[2..];
                    }
                }
                self.node(rest[0]);
            }
            Match => {
                self.s("match ");
                self.node(kids[0]);
                self.s(" ");
                self.item_like_body(&kids[1..]);
            }
            MatchArm => {
                let rest = self.attrs_lines(kids);
                self.node(rest[0]);
                let mut rest = &rest[1..];
                if let Some(&g) = rest.first()
                    && self.kind(g) == Guard
                {
                    self.s(" if ");
                    self.node(self.kids(g)[0]);
                    rest = &rest[1..];
                }
                if let Some(&e) = rest.first() {
                    self.s(" => ");
                    self.node(e);
                }
                self.s(",");
            }
            Guard => {
                self.s("if ");
                self.node(kids[0]);
            }
            Closure => {
                for &k in kids {
                    match self.kind(k) {
                        Keyword | ForLifetimes => {
                            self.node(k);
                            self.s(" ");
                        }
                        ClosureParams => {
                            self.s("|");
                            self.join(self.kids(k), ", ");
                            self.s("| ");
                        }
                        RetType => {
                            self.node(k);
                            self.s(" ");
                        }
                        _ => self.node(k),
                    }
                }
            }
            ClosureParams => {}
            ImplTrait => {
                self.join(kids, "");
                self.s(" for ");
            }
            Break | Continue | Return | Yield | Become | Yeet => {
                self.s(match self.kind(id) {
                    Break => "break",
                    Continue => "continue",
                    Return => "return",
                    Yield => "yield",
                    Become => "become",
                    _ => "do yeet",
                });
                for &k in kids {
                    self.s(" ");
                    self.node(k);
                }
            }
            ConstBlock => {
                self.s("const ");
                self.node(kids[0]);
            }
            Builtin => {
                self.s("builtin # ");
                self.join(kids, "");
            }
            AttrExpr => {
                let rest = self.attrs_inline(kids);
                self.join(rest, "");
            }

            // ---- statements ----
            LetStmt => {
                let rest = self.attrs_lines(kids);
                // The pattern, then a type, a value and an `else` block,
                // each of which may be missing.
                let mut stage = 0;
                for &k in rest {
                    let kk = self.kind(k);
                    if kk == Keyword {
                        self.node(k);
                        self.s(" ");
                        continue;
                    }
                    match stage {
                        0 => {
                            self.s("let ");
                            self.node(k);
                            stage = 1;
                        }
                        1 if is_type_like(kk) => {
                            self.s(": ");
                            self.node(k);
                            stage = 2;
                        }
                        1 | 2 => {
                            self.s(" = ");
                            self.node(k);
                            stage = 3;
                        }
                        _ => {
                            self.s(" else ");
                            self.node(k);
                        }
                    }
                }
                self.s(";");
            }
            ExprStmt => {
                let rest = self.attrs_lines(kids);
                self.join(rest, "");
                self.s(";");
            }
            EmptyStmt => self.s(";"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Edition;
    use crate::parse::parse_str;

    /// Prints `src`, checks the output parses to the same tree, and
    /// returns it.
    fn roundtrip(src: &str) -> String {
        let a = parse_str(src, Edition::E2024).unwrap();
        let out = pretty(&a);
        let b = match parse_str(&out, Edition::E2024) {
            Ok(b) => b,
            Err(e) => panic!("{out}\ndoes not parse: {} at {:?}", e.message, e.at),
        };
        if let Some((x, y)) = first_difference(&a, a.root, &b, b.root) {
            panic!("{out}\nchanged:\n{x}\n{y}");
        }
        out
    }

    #[test]
    fn items_and_expressions() {
        let out = roundtrip(
            "#![no_std]\nuse a::{b,c::*};\n/// Docs.\n#[derive(Debug)]\npub(crate) struct S<T:Copy>{pub a:T,b:u8}\n\
             enum E{A,B(u8),C{x:u8}=3}\nimpl<T> Tr for S<T> where T:Copy{fn f(&self)->u8{let x=self.b+1;if x>2{x}else{0}}}\n\
             fn g(){match v{Some(x)if x>1=>{},None|Some(_)=>(),}for i in 0..n{v.push(i);}let c=|a,b:u8|a+b;}",
        );
        assert_eq!(
            out,
            "#![no_std]\n\nuse a::{b, c::*};\n\n/// Docs.\n#[derive(Debug)]\npub(crate) struct S<T: Copy> {\n    pub a: T,\n    b: u8,\n}\n\n\
             enum E {\n    A,\n    B(u8),\n    C {\n        x: u8,\n    } = 3,\n}\n\n\
             impl<T> Tr for S<T> where T: Copy {\n    fn f(&self) -> u8 {\n        let x = self.b + 1;\n        if x > 2 {\n            x\n        } else {\n            0\n        }\n    }\n}\n\n\
             fn g() {\n    match v {\n        Some(x) if x > 1 => {},\n        None | Some(_) => (),\n    }\n    for i in 0..n {\n        v.push(i);\n    }\n    let c = |a, b: u8| a + b;\n}\n"
        );
    }

    #[test]
    fn tricky_spacing() {
        roundtrip("fn f(){let _=1. ..2.;let _=&&x;let _=a& &b;let _=x.0.1;let v:Vec<Vec<u8>>=Vec::<u8>::new();}");
        roundtrip("fn f(){let _=|| 1;m!{a b};m![x];g(-(-1));let _=(1,);let _:(u8,)=(1,);}");
        roundtrip("fn f()->impl Fn(u8)->u8+Send{|x|x}");
        roundtrip("fn f(){if let Some(x)=a&&let Ok(y)=b{}'a:loop{break 'a;}let Some(x)=o else{return};}");
        roundtrip("macro_rules! m{($x:expr)=>{$x};}\nconst _:()=();static mut X:u8=0;type T<U>=Vec<U>where U:Copy;");
        roundtrip("unsafe extern \"C\"{safe fn f(x:u8,...);}extern crate std as s;mod m{}mod n;");
        roundtrip("fn f<'a,T:?Sized+'a,const N:usize>(x:&'a mut[T;N],y:*const T)->Box<dyn Fn(&T)->bool+'a>{todo!()}");
        roundtrip("fn f(){match x{S{a,b:1,ref c,..}=>{}[a,..,b]=>{}1..=5|-1=>{}A::B(..)=>{}}}");
    }

    #[test]
    fn parenthesised_rooted_bound_in_2015() {
        let src = "impl Tr for dyn (::a::B<u8>) {}";
        let a = parse_str(src, Edition::E2015).unwrap();
        let out = pretty(&a);
        let b = parse_str(&out, Edition::E2015).unwrap();
        assert!(same_tree(&a, a.root, &b, b.root), "{out}");
    }
}
