//! The syntax tree.
//!
//! As tokens are slices of the source, nodes are slices of the token list:
//! a node is a kind, the range of tokens it covers, and a range of the
//! child list. Nothing is copied out of the source. All nodes live in one
//! arena, `Ast::nodes`, and refer to each other by `NodeId`.
//!
//! Children are the parts of a construct that carry meaning: names,
//! literals, lifetimes, operators, the keywords that choose a form (`mut`,
//! `unsafe`, `move`, ...) and sub-constructs. Punctuation, and keywords that
//! only introduce a construct (`fn`, `struct`, `if`), are not children;
//! they are still in the node's token range. Each `Kind` says what its
//! children are.
//!
//! Later stages record what they learn about a node (what a path refers
//! to, an expression's type) in side tables indexed by `NodeId`; the tree
//! itself is never changed after parsing.

use crate::lexer::Token;
use std::fmt::Write;

/// An index into `Ast::nodes`.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, PartialOrd, Ord)]
pub struct NodeId(pub u32);

#[derive(Clone, Copy, Debug)]
pub struct Node {
    pub kind: Kind,
    /// The tokens covered, as a half-open range of indices into
    /// `Ast::tokens`. An empty node has `first == end`.
    pub first: u32,
    pub end: u32,
    child_start: u32,
    child_len: u32,
}

/// A parsed crate (or file).
pub struct Ast<'a> {
    pub tokens: Vec<Token<'a>>,
    pub nodes: Vec<Node>,
    children: Vec<NodeId>,
    pub root: NodeId,
    /// How many tokens came from the file itself. Tokens after these were
    /// added by macro expansion.
    pub original_len: u32,
    /// For each added token, the file token its expansion was called from
    /// (through any nesting): where it sits in the file's text order.
    origin: Vec<u32>,
}

impl<'a> Ast<'a> {
    pub(crate) fn new(tokens: Vec<Token<'a>>) -> Self {
        let original_len = tokens.len() as u32;
        Ast { tokens, nodes: Vec::new(), children: Vec::new(), root: NodeId(0), original_len, origin: Vec::new() }
    }

    pub(crate) fn push(&mut self, kind: Kind, first: usize, end: usize, children: &[NodeId]) -> NodeId {
        let id = NodeId(self.nodes.len() as u32);
        let child_start = self.children.len() as u32;
        self.children.extend_from_slice(children);
        self.nodes.push(Node {
            kind,
            first: first as u32,
            end: end as u32,
            child_start,
            child_len: children.len() as u32,
        });
        id
    }

    /// Gives `id` a new list of children. The old list stays in the child
    /// array, unused. Expansion (`cfg` stripping, macros) changes the tree
    /// only this way, and by adding nodes with `graft`.
    pub fn set_children(&mut self, id: NodeId, children: &[NodeId]) {
        let child_start = self.children.len() as u32;
        self.children.extend_from_slice(children);
        let n = &mut self.nodes[id.0 as usize];
        n.child_start = child_start;
        n.child_len = children.len() as u32;
    }

    /// Changes a node's kind (an attribute from `cfg_attr` in an inner
    /// attribute is itself inner).
    pub fn set_kind(&mut self, id: NodeId, kind: Kind) {
        self.nodes[id.0 as usize].kind = kind;
    }

    /// Adds tokens made by expanding a macro called at token `call`, and
    /// returns the index of the first.
    pub fn append_tokens(&mut self, tokens: &[Token<'a>], call: usize) -> usize {
        let at = self.tokens.len();
        let origin = self.origin_of(call);
        self.tokens.extend_from_slice(tokens);
        self.origin.extend(std::iter::repeat_n(origin, tokens.len()));
        at
    }

    /// The file token that token `i` stands in for in text order: itself if
    /// it is from the file, else the call its expansion came from.
    pub fn origin_of(&self, i: usize) -> u32 {
        if i < self.original_len as usize { i as u32 } else { self.origin[i - self.original_len as usize] }
    }

    /// Whether every token of a node is from the file itself, in order.
    pub fn is_original(&self, id: NodeId) -> bool {
        self.node(id).end <= self.original_len
    }

    /// Copies the nodes of `frag`, parsed from the tokens that are this
    /// tree's from `token_offset` on, into this tree, and returns the new
    /// id of `frag`'s root.
    pub fn graft(&mut self, frag: &Ast<'a>, token_offset: usize) -> NodeId {
        let base = self.nodes.len() as u32;
        let child_base = self.children.len() as u32;
        self.children.extend(frag.children.iter().map(|c| NodeId(c.0 + base)));
        self.nodes.extend(frag.nodes.iter().map(|n| Node {
            kind: n.kind,
            first: n.first + token_offset as u32,
            end: n.end + token_offset as u32,
            child_start: n.child_start + child_base,
            child_len: n.child_len,
        }));
        NodeId(frag.root.0 + base)
    }

    /// Drops every node from `nodes` on, for backtracking.
    pub(crate) fn truncate(&mut self, nodes: usize, children: usize) {
        self.nodes.truncate(nodes);
        self.children.truncate(children);
    }

    pub(crate) fn sizes(&self) -> (usize, usize) {
        (self.nodes.len(), self.children.len())
    }

    pub fn node(&self, id: NodeId) -> &Node {
        &self.nodes[id.0 as usize]
    }

    pub fn kind(&self, id: NodeId) -> Kind {
        self.node(id).kind
    }

    pub fn children(&self, id: NodeId) -> &[NodeId] {
        let n = self.node(id);
        &self.children[n.child_start as usize..(n.child_start + n.child_len) as usize]
    }

    /// The first child of kind `kind`.
    pub fn child(&self, id: NodeId, kind: Kind) -> Option<NodeId> {
        self.children(id).iter().copied().find(|&c| self.kind(c) == kind)
    }

    /// The tokens a node covers.
    pub fn tokens_of(&self, id: NodeId) -> &[Token<'a>] {
        let n = self.node(id);
        &self.tokens[n.first as usize..n.end as usize]
    }

    /// The node's first token: for a leaf, the token itself.
    pub fn token(&self, id: NodeId) -> Token<'a> {
        self.tokens_of(id).first().copied().unwrap_or(Token(""))
    }

    /// The text of a node: its tokens, with a space between two that were
    /// not next to each other in the source.
    pub fn text(&self, id: NodeId) -> String {
        let mut out = String::new();
        let mut prev: Option<Token> = None;
        for &t in self.tokens_of(id) {
            if prev.is_some_and(|p| !p.touches(t)) {
                out.push(' ');
            }
            out += t.0;
            prev = Some(t);
        }
        out
    }

    /// The tree as nested lists, one node per line: `Kind` for a node with
    /// children, `Kind text` for a leaf.
    pub fn dump(&self, id: NodeId) -> String {
        let mut out = String::new();
        self.dump_into(id, 0, &mut out);
        out
    }

    fn dump_into(&self, id: NodeId, depth: usize, out: &mut String) {
        let _ = write!(out, "{:width$}{:?}", "", self.kind(id), width = depth * 2);
        if self.kind(id).is_leaf() {
            let _ = write!(out, " {}", self.text(id));
        }
        out.push('\n');
        for &k in self.children(id) {
            self.dump_into(k, depth + 1, out);
        }
    }

    /// The tree on one line, as s-expressions: `(Binary (PathExpr (Path
    /// (Segment (Name a)))) (Op +) (Literal 1))`. Handy in tests.
    pub fn sexp(&self, id: NodeId) -> String {
        let kids = self.children(id);
        let head = if self.kind(id).is_leaf() {
            format!("{:?} {}", self.kind(id), self.text(id))
        } else {
            format!("{:?}", self.kind(id))
        };
        if kids.is_empty() {
            return format!("({head})");
        }
        let inner: Vec<String> = kids.iter().map(|&k| self.sexp(k)).collect();
        format!("({head} {})", inner.join(" "))
    }
}

impl Kind {
    /// Whether nodes of this kind stand for their tokens rather than for
    /// their children. A dump prints a leaf's text.
    pub fn is_leaf(self) -> bool {
        use Kind::*;
        matches!(self, Name | Lifetime | Literal | Op | Keyword | DocComment | TokenTree | Abi)
    }
}

/// What a node is. The comment on each kind lists its children, always in
/// source order; `?` marks an optional child, `*` a repeated one, and `|`
/// a choice. `attrs` is `(Attribute | DocComment)*`; `vis` is
/// `Visibility?`.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Kind {
    // ---- leaves ----
    /// An identifier, raw or not; also `self`, `Self`, `super`, `crate` and
    /// `$crate` in paths, and `_` where it names nothing.
    Name,
    /// `'a`, `'static`, `'_`; also a label.
    Lifetime,
    /// A literal, with its suffix: `1u8`, `"s"`, `b'x'`, `true`, `false`.
    /// A negative literal in a pattern or const argument is a `Unary`.
    Literal,
    /// An operator, which may be several touching tokens: `+`, `<<=`,
    /// `..=`, `?` (in `?Sized`), `!` (in `impl !Send`).
    Op,
    /// A keyword or weak keyword that chooses a form: `mut`, `ref`,
    /// `unsafe`, `const`, `async`, `move`, `static`, `default`, `auto`,
    /// `safe`, `raw`, `dyn`, `impl`, `extern`, `gen`, `box`, `try`.
    Keyword,
    /// `/// text` or `/** text */` (an outer `doc` attribute), or `//!` /
    /// `/*! */` (an inner one).
    DocComment,
    /// A delimited group of tokens, kept unparsed: a macro's input or
    /// definition, or an attribute's arguments.
    TokenTree,
    /// The string after `extern`: `"C"`.
    Abi,

    // ---- the crate and attributes ----
    /// The whole file: `InnerAttribute* Item*`.
    Crate,
    /// `#[path args]` or `#[path = value]`: `Keyword(unsafe)? Path
    /// (TokenTree | Expr)?`.
    Attribute,
    /// `#![...]`: as `Attribute`.
    InnerAttribute,
    /// `pub`, `pub(crate)`, `pub(self)`, `pub(super)`, `pub(in path)`:
    /// `Path?`.
    Visibility,
    /// `impl(crate)`, `impl(in path)` before a trait: where it may be
    /// implemented. `Path`.
    ImplRestriction,

    // ---- items ----
    /// `attrs vis Keyword(const|async|gen|unsafe|safe|default|final|extern)*
    /// Abi? Name Generics? Params RetType? WhereClause? Block?`. Without a
    /// body it ends in `;`.
    Fn,
    /// `attrs vis Name Generics? WhereClause? (NamedFields | TupleFields)?`;
    /// a unit struct has no fields node.
    Struct,
    /// `attrs vis Name Generics? WhereClause? NamedFields`.
    Union,
    /// `attrs vis Name Generics? WhereClause? Variant*`.
    Enum,
    /// `attrs vis Name (NamedFields | TupleFields)? Expr?`; the expression
    /// is the discriminant.
    Variant,
    /// `{ FieldDef* }`.
    NamedFields,
    /// `attrs vis Keyword(unsafe)? Name Type Expr?`; the expression is a
    /// default value.
    FieldDef,
    /// `( TupleField* )`.
    TupleFields,
    /// `attrs vis Type`.
    TupleField,
    /// `attrs vis ImplRestriction? Keyword(unsafe|auto|const)* Name Generics? Bounds?
    /// WhereClause? InnerAttribute* Item*`.
    Trait,
    /// `trait A<T> = Bounds where ...;`: `attrs vis Name Generics? Bounds?
    /// WhereClause?`.
    TraitAlias,
    /// `attrs Keyword(default|unsafe)* Generics? Keyword(const)?
    /// ImplTrait? Type WhereClause? InnerAttribute* Item*`. `ImplTrait`
    /// is the trait for a trait impl; the `Type` is the self type.
    Impl,
    /// `Op(!)? Type`: the trait an impl implements.
    ImplTrait,
    /// `attrs vis Keyword(unsafe)? Name (InnerAttribute* Item*)?`; a
    /// module in another file has no items here.
    Mod,
    /// `attrs vis UseTree`.
    Use,
    /// `Path? (UseGlob | UseGroup | Rename)?`; `PathRoot` for a leading
    /// `::`.
    UseTree,
    /// `*`, at the end of a use path.
    UseGlob,
    /// `{ UseTree* }`.
    UseGroup,
    /// `as name` or `as _`: `Name`.
    Rename,
    /// `attrs vis Keyword(default)? Name Generics? Type? WhereClause?
    /// Expr?`. The name may be `_`.
    Const,
    /// `attrs vis Keyword(safe|unsafe|mut)* Name Type Expr?`.
    Static,
    /// `attrs vis Keyword(default)? Name Generics? Bounds? WhereClause?
    /// Type? WhereClause?`.
    TypeAlias,
    /// `attrs vis Name Rename?`.
    ExternCrate,
    /// `attrs Keyword(unsafe)? Abi? InnerAttribute* Item*`.
    ExternBlock,
    /// `macro_rules! name { ... }`: `attrs Name TokenTree`.
    MacroRules,
    /// `macro name(...) { ... }` or `macro name { ... }`: `attrs vis Name
    /// TokenTree TokenTree?`.
    MacroDef,
    /// `path!(...)`, `path![...]` or `path!{...}` as an item, statement or
    /// expression: `attrs Path TokenTree`.
    MacroCall,
    /// A macro call in a type: `Path TokenTree`.
    TypeMacro,
    /// A macro call in a pattern: `Path TokenTree`.
    PatMacro,

    // ---- generics ----
    /// `<GenericParam*>`: `(LifetimeParam | TypeParam | ConstParam)*`.
    Generics,
    /// `attrs Lifetime Bounds?`.
    LifetimeParam,
    /// `attrs Name Bounds? Type?`; the type is the default.
    TypeParam,
    /// `attrs Name Type Expr?`; the expression is the default.
    ConstParam,
    /// `WherePredicate*`.
    WhereClause,
    /// `ForLifetimes? Type Bounds?` or `Lifetime Bounds?`.
    WherePredicate,
    /// `for<'a, 'b>`: `LifetimeParam*`.
    ForLifetimes,
    /// `(TraitBound | Lifetime | UseBound)*`, joined by `+`.
    Bounds,
    /// `ForLifetimes? (Op | Keyword)* Path`: `?Sized`, `~const Fn()`,
    /// `[const] Tr`, `async Fn()`, `!Unpin`, `for<'a> Fn(&'a u8)`. A
    /// bound in parentheses is the same node.
    TraitBound,
    /// `use<'a, T>`: `(Lifetime | Name)*`.
    UseBound,

    // ---- functions ----
    /// `(SelfParam | Param | Variadic)*`.
    Params,
    /// `attrs Op(&)? Lifetime? Keyword(mut)? Name(self) Type?`.
    SelfParam,
    /// `attrs Pattern? Type`; in a function without a body (or a `fn`
    /// type) the pattern may be missing.
    Param,
    /// `...`, possibly named: `attrs Pattern?`.
    Variadic,
    /// `-> Type`.
    RetType,

    // ---- paths ----
    /// `PathRoot? QSelf? Segment*`.
    Path,
    /// A leading `::`.
    PathRoot,
    /// `<Type as Trait>::` or `<Type>::`: `Type Type?`.
    QSelf,
    /// `Name (GenericArgs | FnSugar)?`.
    Segment,
    /// `<...>` or `::<...>`: `(Lifetime | Type | Expr | Block |
    /// AssocConstraint)*`.
    GenericArgs,
    /// `Fn(A, B) -> C`: `Type* RetType?`; `Fn(..)` (return type notation)
    /// has an `Op(..)` child.
    FnSugar,
    /// `Item = Type`, `Item: Bounds`, `N = 3`, `Item<'a> = T`: `Name
    /// (GenericArgs | FnSugar)? (Type | Expr | Block | Bounds)`.
    AssocConstraint,

    // ---- types ----
    /// `Path`.
    PathType,
    /// `&'a mut T`: `Lifetime? Keyword(mut)? Type`.
    RefType,
    /// `*const T`, `*mut T`: `Keyword Type`.
    PtrType,
    /// `[T; N]`: `Type Expr`.
    ArrayType,
    /// `[T]`: `Type`.
    SliceType,
    /// `(A, B)`, `()`: `Type*`.
    TupleType,
    /// `(T)`: `Type`.
    ParenType,
    /// `!`.
    NeverType,
    /// `_`.
    InferType,
    /// `ForLifetimes? Keyword(unsafe|extern)* Abi? Params RetType?`.
    FnPtrType,
    /// `impl Bounds`: `Bounds`.
    ImplTraitType,
    /// `dyn Bounds`, or a bare bound list in old editions: `Keyword(dyn)?
    /// Bounds`.
    DynTraitType,

    // ---- patterns ----
    /// `_`.
    WildPat,
    /// `..`.
    RestPat,
    /// `ref mut x @ sub`: `Keyword(ref)? Keyword(mut)? Name Pattern?`.
    IdentPat,
    /// `1`, `-1`, `"s"`, `b'x'`: `Literal | Unary`.
    LitPat,
    /// `a..=b`, `a..`, `..=b`: `Pattern? Op Pattern?`.
    RangePat,
    /// `&mut p`: `Keyword(mut)? Pattern`.
    RefPat,
    /// `(a, b)`: `Pattern*`.
    TuplePat,
    /// `(p)`: `Pattern`.
    ParenPat,
    /// `[a, .., b]`: `Pattern*`.
    SlicePat,
    /// `Path`, as a constant, unit struct or variant.
    PathPat,
    /// `Path(a, b)`: `Path Pattern*`.
    TupleStructPat,
    /// `Path { a, b: p, .. }`: `Path FieldPat* RestPat?`.
    StructPat,
    /// `attrs (Name | Literal)? Pattern`: `x: p`, `0: p`, or the shorthand
    /// `ref mut x` (an `IdentPat` alone).
    FieldPat,
    /// `a | b`: `Pattern*`.
    OrPat,
    /// `box p`: `Pattern`.
    BoxPat,
    /// `!`, the never pattern. Macro calls in patterns are `PatMacro`,
    /// and `const { }` is a `ConstBlock`.
    NeverPat,

    // ---- expressions ----
    /// `Path`.
    PathExpr,
    /// `a + b`, `a = b`, `a += b`, `a && b`: `Expr Op Expr`.
    Binary,
    /// `-a`, `!a`, `*a`: `Op Expr`.
    Unary,
    /// `&a`, `&mut a`, `&raw const a`: `Keyword(raw)? Keyword(mut|const)?
    /// Expr`.
    Borrow,
    /// `a as T`: `Expr Type`.
    Cast,
    /// `a..b`, `..`, `a..=b`: `Expr? Op Expr?`.
    Range,
    /// `f(a, b)`: `Expr Args`.
    Call,
    /// `a.f::<T>(b)`: `Expr Segment Args`.
    MethodCall,
    /// `(Expr*)`, the arguments of a call.
    Args,
    /// `a.b`, `a.0`: `Expr (Name | Literal)`.
    Field,
    /// `a[b]`: `Expr Expr`.
    Index,
    /// `a?`: `Expr`.
    Try,
    /// `a.await`: `Expr`.
    Await,
    /// `a.use`: `Expr`.
    UseExpr,
    /// `(a, b)`, `()`: `Expr*`.
    Tuple,
    /// `(a)`: `Expr`.
    Paren,
    /// `[a, b]`: `Expr*`.
    Array,
    /// `[a; n]`: `Expr Expr`.
    ArrayRepeat,
    /// `Path { a, b: e, ..base }`: `Path FieldInit* StructBase?`.
    StructExpr,
    /// `attrs (Name | Literal) Expr?`; shorthand `a` has no expression.
    FieldInit,
    /// `..base`, or `..` alone (default field values): `Expr?`.
    StructBase,
    /// `{ ... }`, with any qualifier: `attrs Lifetime? Keyword(unsafe|async
    /// |move|const|try|gen)* InnerAttribute* Stmt*`. A block's last
    /// expression without `;` is an expression child, not an `ExprStmt`.
    Block,
    /// `if cond { } else ...`: `Expr Block (Block | If)?`. A `let` in the
    /// condition is a `Let`, possibly inside `&&`.
    If,
    /// `let p = e` inside a condition: `Pattern Expr`.
    Let,
    /// `Lifetime? Expr Block`.
    While,
    /// `Lifetime? Block`.
    Loop,
    /// `Lifetime? Pattern Expr Block`; `for await` adds `Keyword(await)`.
    For,
    /// `match e { arms }`: `Expr InnerAttribute* MatchArm*`.
    Match,
    /// `attrs Pattern Guard? Expr?`; a never pattern needs no expression.
    MatchArm,
    /// `if e` after an arm's pattern: `Expr`.
    Guard,
    /// `Keyword(static|async|gen|move|use)* ForLifetimes? ClosureParams
    /// RetType? Expr`.
    Closure,
    /// `|a, b: T|`: `ClosureParam*`.
    ClosureParams,
    /// `attrs Pattern Type?`.
    ClosureParam,
    /// `break 'a e`: `Lifetime? Expr?`.
    Break,
    /// `continue 'a`: `Lifetime?`.
    Continue,
    /// `return e`: `Expr?`.
    Return,
    /// `yield e`: `Expr?`.
    Yield,
    /// `become f()`: `Expr`.
    Become,
    /// `do yeet e`: `Expr?`.
    Yeet,
    /// `_` as an expression (the left of a destructuring assignment).
    Underscore,
    /// `const { }`: `Block`.
    ConstBlock,
    /// `builtin # name(args)`: `Name TokenTree`.
    Builtin,
    /// An expression with outer attributes: `attrs Expr`.
    AttrExpr,

    // ---- statements ----
    /// `attrs (Keyword(super))? Pattern Type? Expr? Block?`; the block is
    /// the `else` of a `let ... else`.
    LetStmt,
    /// An expression and its `;`: `Expr`.
    ExprStmt,
    /// A lone `;`.
    EmptyStmt,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dump_and_sexp() {
        let src = "a + 1";
        let toks = crate::lexer::lex(src).unwrap();
        let mut ast = Ast::new(toks);
        let a = ast.push(Kind::Name, 0, 1, &[]);
        let op = ast.push(Kind::Op, 1, 2, &[]);
        let one = ast.push(Kind::Literal, 2, 3, &[]);
        let b = ast.push(Kind::Binary, 0, 3, &[a, op, one]);
        assert_eq!(ast.sexp(b), "(Binary (Name a) (Op +) (Literal 1))");
        assert_eq!(ast.dump(b), "Binary\n  Name a\n  Op +\n  Literal 1\n");
        assert_eq!(ast.text(b), "a + 1");
    }
}
