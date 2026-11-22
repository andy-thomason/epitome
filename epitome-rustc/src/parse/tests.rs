//! Parser tests: small inputs, compared as s-expressions.

use super::{Fragment, parse_fragment, parse_str};
use crate::lexer::{Edition, Token, lex};

fn frag(src: &str, what: Fragment) -> String {
    let tokens = lex(src).unwrap();
    match parse_fragment(tokens, Token(&src[src.len()..]), Edition::E2024, what) {
        Ok(ast) => ast.sexp(ast.root),
        Err(e) => panic!("{src}: {} at {:?}", e.message, e.at),
    }
}

fn expr(src: &str) -> String {
    frag(src, Fragment::Expr)
}

fn ty(src: &str) -> String {
    frag(src, Fragment::Type)
}

fn pat(src: &str) -> String {
    frag(src, Fragment::Pattern)
}

fn item(src: &str) -> String {
    frag(src, Fragment::Item)
}

fn stmts(src: &str) -> String {
    frag(src, Fragment::Stmts)
}

fn fails(src: &str) -> String {
    match parse_str(src, Edition::E2024) {
        Ok(ast) => panic!("{src} parsed: {}", ast.sexp(ast.root)),
        Err(e) => e.message,
    }
}

/// A path expression of one segment, as an s-expression.
fn p(name: &str) -> String {
    format!("(PathExpr (Path (Segment (Name {name}))))")
}

#[test]
fn precedence() {
    assert_eq!(expr("a + b * c"), format!("(Binary {} (Op +) (Binary {} (Op *) {}))", p("a"), p("b"), p("c")));
    assert_eq!(expr("a - b - c"), format!("(Binary (Binary {} (Op -) {}) (Op -) {})", p("a"), p("b"), p("c")));
    assert_eq!(expr("a = b = c"), format!("(Binary {} (Op =) (Binary {} (Op =) {}))", p("a"), p("b"), p("c")));
    assert_eq!(expr("a << b >>= c"), format!("(Binary (Binary {} (Op <<) {}) (Op >>=) {})", p("a"), p("b"), p("c")));
    assert_eq!(expr("-a as u8"), format!("(Cast (Unary (Op -) {}) (PathType (Path (Segment (Name u8)))))", p("a")));
    assert_eq!(expr("a && b || c"), format!("(Binary (Binary {} (Op &&) {}) (Op ||) {})", p("a"), p("b"), p("c")));
    assert_eq!(expr("&&a"), format!("(Borrow (Borrow {}))", p("a")));
    assert_eq!(expr("a & &b"), format!("(Binary {} (Op &) (Borrow {}))", p("a"), p("b")));
    assert_eq!(expr("&raw const a"), format!("(Borrow (Keyword raw) (Keyword const) {})", p("a")));
}

#[test]
fn ranges() {
    assert_eq!(expr("a..b"), format!("(Range {} (Op ..) {})", p("a"), p("b")));
    assert_eq!(expr(".."), "(Range (Op ..))");
    assert_eq!(expr("..=b"), format!("(Range (Op ..=) {})", p("b")));
    assert_eq!(expr("a.."), format!("(Range {} (Op ..))", p("a")));
    assert_eq!(expr("0..n + 1"), format!("(Range (Literal 0) (Op ..) (Binary {} (Op +) (Literal 1)))", p("n")));
}

#[test]
fn postfix() {
    assert_eq!(
        expr("a.b.0.c()?"),
        format!("(Try (MethodCall (Field (Field {} (Name b)) (Literal 0)) (Segment (Name c)) (Args)))", p("a"))
    );
    assert_eq!(expr("x.0.1"), format!("(Field (Field {} (Literal 0)) (Literal 1))", p("x")));
    assert_eq!(
        expr("v.iter().map::<u8, _>(f)"),
        format!(
            "(MethodCall (MethodCall {} (Segment (Name iter)) (Args)) (Segment (Name map) (GenericArgs (PathType (Path (Segment (Name u8)))) (InferType))) (Args {}))",
            p("v"),
            p("f")
        )
    );
    assert_eq!(expr("f.await"), format!("(Await {})", p("f")));
    assert_eq!(expr("a[i](x)"), format!("(Call (Index {} {}) (Args {}))", p("a"), p("i"), p("x")));
}

#[test]
fn paths_and_struct_literals() {
    assert_eq!(
        expr("Vec::<Vec<u8>>::new()"),
        "(Call (PathExpr (Path (Segment (Name Vec) (GenericArgs (PathType (Path (Segment (Name Vec) (GenericArgs (PathType (Path (Segment (Name u8)))))))))) (Segment (Name new)))) (Args))"
    );
    assert_eq!(
        expr("<T as Tr>::f"),
        "(PathExpr (Path (QSelf (PathType (Path (Segment (Name T)))) (PathType (Path (Segment (Name Tr))))) (Segment (Name f))))"
    );
    assert_eq!(
        expr("S { a, b: 1, ..d }"),
        format!(
            "(StructExpr (Path (Segment (Name S))) (FieldInit (Name a)) (FieldInit (Name b) (Literal 1)) (StructBase {}))",
            p("d")
        )
    );
    assert_eq!(
        expr("::std::mem::drop"),
        "(PathExpr (Path (PathRoot) (Segment (Name std)) (Segment (Name mem)) (Segment (Name drop))))"
    );
    assert_eq!(expr("println!(\"{}\", x)"), "(MacroCall (Path (Segment (Name println))) (TokenTree (\"{}\", x)))");
}

#[test]
fn no_struct_in_conditions() {
    assert_eq!(
        expr("if x == S { 1 } else { 2 }"),
        format!("(If (Binary {} (Op ==) {}) (Block (Literal 1)) (Block (Literal 2)))", p("x"), p("S"))
    );
    assert_eq!(expr("match (S {}) {}"), "(Match (Paren (StructExpr (Path (Segment (Name S))))))");
    assert_eq!(
        expr("for x in 0..n {}"),
        format!("(For (IdentPat (Name x)) (Range (Literal 0) (Op ..) {}) (Block))", p("n"))
    );
}

#[test]
fn let_chains() {
    assert_eq!(
        expr("if let Some(x) = a && x > 1 {}"),
        format!(
            "(If (Binary (Let (TupleStructPat (Path (Segment (Name Some))) (IdentPat (Name x))) {}) (Op &&) (Binary {} (Op >) (Literal 1))) (Block))",
            p("a"),
            p("x")
        )
    );
}

#[test]
fn closures() {
    assert_eq!(expr("|| 1"), "(Closure (ClosureParams) (Literal 1))");
    assert_eq!(
        expr("move |a, b: u8| -> u8 { a }"),
        format!(
            "(Closure (Keyword move) (ClosureParams (ClosureParam (IdentPat (Name a))) (ClosureParam (IdentPat (Name b)) (PathType (Path (Segment (Name u8)))))) (RetType (PathType (Path (Segment (Name u8))))) (Block {}))",
            p("a")
        )
    );
    assert_eq!(expr("async move { 1 }"), "(Block (Keyword async) (Keyword move) (Literal 1))");
}

#[test]
fn statements() {
    assert_eq!(
        stmts("let x: u8 = 1; x"),
        format!("(Block (LetStmt (IdentPat (Name x)) (PathType (Path (Segment (Name u8)))) (Literal 1)) {})", p("x"))
    );
    // A block-like expression ends its statement.
    assert_eq!(stmts("if a {} -1"), format!("(Block (If {} (Block)) (Unary (Op -) (Literal 1)))", p("a")));
    assert_eq!(stmts("match a {}.f()"), format!("(Block (MethodCall (Match {}) (Segment (Name f)) (Args)))", p("a")));
    assert_eq!(
        stmts("let Some(x) = o else { return };"),
        format!(
            "(Block (LetStmt (TupleStructPat (Path (Segment (Name Some))) (IdentPat (Name x))) {} (Block (Return))))",
            p("o")
        )
    );
    assert_eq!(
        stmts("fn f() {} f()"),
        "(Block (Fn (Name f) (Params) (Block)) (Call (PathExpr (Path (Segment (Name f)))) (Args)))"
    );
    assert_eq!(
        stmts("m! { a } *x = 1;"),
        format!(
            "(Block (MacroCall (Path (Segment (Name m))) (TokenTree {{ a }})) (ExprStmt (Binary (Unary (Op *) {}) (Op =) (Literal 1))))",
            p("x")
        )
    );
    assert_eq!(
        stmts("m! { } x"),
        format!("(Block (MacroCall (Path (Segment (Name m))) (TokenTree {{ }})) {})", p("x"))
    );
    assert_eq!(
        stmts("'a: loop { break 'a 1; }"),
        "(Block (Loop (Lifetime 'a) (Block (ExprStmt (Break (Lifetime 'a) (Literal 1))))))"
    );
    assert_eq!(
        stmts("unsafe { f() }"),
        "(Block (Block (Keyword unsafe) (Call (PathExpr (Path (Segment (Name f)))) (Args))))"
    );
}

#[test]
fn match_arms() {
    assert_eq!(
        expr("match x { 0 | 1 => a, 2..=9 if c => {} _ => b }"),
        format!(
            "(Match {} (MatchArm (OrPat (LitPat (Literal 0)) (LitPat (Literal 1))) {}) (MatchArm (RangePat (LitPat (Literal 2)) (Op ..=) (LitPat (Literal 9))) (Guard {}) (Block)) (MatchArm (WildPat) {}))",
            p("x"),
            p("a"),
            p("c"),
            p("b")
        )
    );
}

#[test]
fn types() {
    assert_eq!(
        ty("&'a mut [u8]"),
        "(RefType (Lifetime 'a) (Keyword mut) (SliceType (PathType (Path (Segment (Name u8))))))"
    );
    assert_eq!(ty("&&T"), "(RefType (RefType (PathType (Path (Segment (Name T))))))");
    assert_eq!(ty("*const ()"), "(PtrType (Keyword const) (TupleType))");
    assert_eq!(ty("[u8; 4]"), "(ArrayType (PathType (Path (Segment (Name u8)))) (Literal 4))");
    assert_eq!(
        ty("Box<dyn Fn(u8) -> u8 + Send + 'static>"),
        "(PathType (Path (Segment (Name Box) (GenericArgs (DynTraitType (Keyword dyn) (Bounds (TraitBound (Path (Segment (Name Fn) (FnSugar (PathType (Path (Segment (Name u8)))) (RetType (PathType (Path (Segment (Name u8))))))))) (TraitBound (Path (Segment (Name Send)))) (Lifetime 'static)))))))"
    );
    assert_eq!(
        ty("impl Iterator<Item = &'a T>"),
        "(ImplTraitType (Bounds (TraitBound (Path (Segment (Name Iterator) (GenericArgs (AssocConstraint (Name Item) (RefType (Lifetime 'a) (PathType (Path (Segment (Name T))))))))))))"
    );
    assert_eq!(
        ty("for<'a> unsafe extern \"C\" fn(&'a u8, ...) -> !"),
        "(FnPtrType (ForLifetimes (LifetimeParam (Lifetime 'a))) (Keyword unsafe) (Keyword extern) (Abi \"C\") (Params (Param (RefType (Lifetime 'a) (PathType (Path (Segment (Name u8)))))) (Variadic)) (RetType (NeverType)))"
    );
    assert_eq!(ty("(u8,)"), "(TupleType (PathType (Path (Segment (Name u8)))))");
    assert_eq!(ty("(u8)"), "(ParenType (PathType (Path (Segment (Name u8)))))");
    assert_eq!(
        ty("<T as Tr>::A"),
        "(PathType (Path (QSelf (PathType (Path (Segment (Name T)))) (PathType (Path (Segment (Name Tr))))) (Segment (Name A))))"
    );
    assert!(ty("[T; { N + 1 }]").starts_with("(ArrayType"));
}

#[test]
fn patterns() {
    assert_eq!(
        pat("ref mut x @ Some(_)"),
        "(IdentPat (Keyword ref) (Keyword mut) (Name x) (TupleStructPat (Path (Segment (Name Some))) (WildPat)))"
    );
    assert_eq!(pat("[a, .., b]"), "(SlicePat (IdentPat (Name a)) (RestPat) (IdentPat (Name b)))");
    assert_eq!(pat("&(a, b)"), "(RefPat (TuplePat (IdentPat (Name a)) (IdentPat (Name b))))");
    assert_eq!(pat("&&x"), "(RefPat (RefPat (IdentPat (Name x))))");
    assert_eq!(pat("(..)"), "(TuplePat (RestPat))");
    assert_eq!(pat("-1..=5"), "(RangePat (LitPat (Unary (Op -) (Literal 1))) (Op ..=) (LitPat (Literal 5)))");
    assert_eq!(
        pat("A..=B"),
        "(RangePat (PathPat (Path (Segment (Name A)))) (Op ..=) (PathPat (Path (Segment (Name B)))))"
    );
    assert_eq!(pat("..=5"), "(RangePat (Op ..=) (LitPat (Literal 5)))");
    assert_eq!(
        pat("S { a, b: 1, ref c, .. }"),
        "(StructPat (Path (Segment (Name S))) (FieldPat (IdentPat (Name a))) (FieldPat (Name b) (LitPat (Literal 1))) (FieldPat (IdentPat (Keyword ref) (Name c))) (RestPat))"
    );
    assert_eq!(pat("a::B"), "(PathPat (Path (Segment (Name a)) (Segment (Name B))))");
}

#[test]
fn items() {
    assert_eq!(
        item("pub(crate) const unsafe fn f<'a, T: Copy + 'a, const N: usize>(&mut self, x: &'a T) -> u8 where T: Tr;"),
        "(Fn (Visibility (Path (Segment (Name crate)))) (Keyword const) (Keyword unsafe) (Name f) (Generics (LifetimeParam (Lifetime 'a)) (TypeParam (Name T) (Bounds (TraitBound (Path (Segment (Name Copy)))) (Lifetime 'a))) (ConstParam (Name N) (PathType (Path (Segment (Name usize)))))) (Params (SelfParam (Op &) (Keyword mut) (Name self)) (Param (IdentPat (Name x)) (RefType (Lifetime 'a) (PathType (Path (Segment (Name T))))))) (RetType (PathType (Path (Segment (Name u8))))) (WhereClause (WherePredicate (PathType (Path (Segment (Name T)))) (Bounds (TraitBound (Path (Segment (Name Tr))))))))"
    );
    assert_eq!(
        item("struct S<T>(pub T, u8);"),
        "(Struct (Name S) (Generics (TypeParam (Name T))) (TupleFields (TupleField (Visibility) (PathType (Path (Segment (Name T))))) (TupleField (PathType (Path (Segment (Name u8)))))))"
    );
    assert_eq!(
        item("enum E { A, B(u8), C { x: u8 } = 3 }"),
        "(Enum (Name E) (Variant (Name A)) (Variant (Name B) (TupleFields (TupleField (PathType (Path (Segment (Name u8))))))) (Variant (Name C) (NamedFields (FieldDef (Name x) (PathType (Path (Segment (Name u8)))))) (Literal 3)))"
    );
    assert_eq!(
        item("impl<T> !Send for S<T> {}"),
        "(Impl (Generics (TypeParam (Name T))) (ImplTrait (Op !) (PathType (Path (Segment (Name Send))))) (PathType (Path (Segment (Name S) (GenericArgs (PathType (Path (Segment (Name T)))))))))"
    );
    assert_eq!(
        item("use a::{b, c::*, d as e, self};"),
        "(Use (UseTree (Path (Segment (Name a))) (UseGroup (UseTree (Path (Segment (Name b)))) (UseTree (Path (Segment (Name c))) (UseGlob)) (UseTree (Path (Segment (Name d))) (Rename (Name e))) (UseTree (Path (Segment (Name self)))))))"
    );
    assert_eq!(
        item("unsafe extern \"C\" { safe fn f(); }"),
        "(ExternBlock (Keyword unsafe) (Abi \"C\") (Fn (Keyword safe) (Name f) (Params)))"
    );
    assert_eq!(item("macro_rules! m { () => {} }"), "(MacroRules (Name m) (TokenTree { () => {} }))");
    assert!(
        item("#[derive(Debug)] /// doc\nunion U { a: u8 }")
            .starts_with("(Union (Attribute (Path (Segment (Name derive))) (TokenTree (Debug))) (DocComment /// doc)")
    );
    assert_eq!(item("const _: () = ();"), "(Const (Name _) (TupleType) (Tuple))");
    assert!(item("trait T: A + B { type X: C; fn f(&self) {} }").starts_with("(Trait (Name T) (Bounds"));
    assert_eq!(item("auto trait Send {}"), "(Trait (Keyword auto) (Name Send))");
    assert_eq!(item("extern crate std as s;"), "(ExternCrate (Name std) (Rename (Name s)))");
}

#[test]
fn whole_file() {
    let src = "#![no_std]\n//! Crate docs.\nmod m;\nfn main() { let v: Vec<Vec<u8>>= vec![]; }\n";
    let ast = parse_str(src, Edition::E2024).unwrap();
    assert!(ast.sexp(ast.root).starts_with(
        "(Crate (InnerAttribute (Path (Segment (Name no_std)))) (DocComment //! Crate docs.) (Mod (Name m))"
    ));
}

#[test]
fn errors() {
    assert_eq!(fails("fn f( {}"), "expected a type, found `{`");
    assert_eq!(fails("fn f() { let x = ; }"), "expected an expression, found `;`");
    assert_eq!(fails("fn f() { a b }"), "expected `;`, found `b`");
    assert_eq!(fails("fn f() { ( ] }"), "expected an expression, found `]`");
}

#[test]
fn deep_generics_are_linear() {
    // Backing up at each level would take 2^200 steps.
    let depth = 200;
    let src = format!("{}u8{}", "A<".repeat(depth), ">".repeat(depth));
    let t = std::time::Instant::now();
    assert!(ty(&src).starts_with("(PathType"));
    assert!(t.elapsed().as_secs_f64() < 1.0);
}

#[test]
fn builtin_forms() {
    assert_eq!(
        expr("builtin # offset_of(Outer<T>, inner.c.0)"),
        "(Builtin (Name offset_of) (PathType (Path (Segment (Name Outer) (GenericArgs (PathType (Path (Segment (Name T)))))))) (TokenTree inner.c.0))"
    );
    assert_eq!(
        expr("builtin # type_ascribe(a + 1, u64)"),
        "(Builtin (Name type_ascribe) (Binary (PathExpr (Path (Segment (Name a)))) (Op +) (Literal 1)) (PathType (Path (Segment (Name u64)))))"
    );
    assert_eq!(
        expr("builtin # unwrap_binder(b, &u8)"),
        "(Builtin (Name unwrap_binder) (PathExpr (Path (Segment (Name b)))) (RefType (PathType (Path (Segment (Name u8))))))"
    );
    assert_eq!(expr("builtin # wrap_binder(b)"), "(Builtin (Name wrap_binder) (PathExpr (Path (Segment (Name b)))))");
    // A form not known: its arguments as tokens.
    assert_eq!(expr("builtin # other(a b c)"), "(Builtin (Name other) (TokenTree (a b c)))");
    assert_eq!(
        ty("unsafe<'a, 'b> &'a &'b u8"),
        "(UnsafeBinderType (Lifetime 'a) (Lifetime 'b) (RefType (Lifetime 'a) (RefType (Lifetime 'b) (PathType (Path (Segment (Name u8)))))))"
    );
    assert_eq!(ty("builtin # field_of(E, B.q)"), "(BuiltinType (Name field_of) (PathType (Path (Segment (Name E)))) (TokenTree B.q))");
    assert_eq!(
        pat("builtin # deref(Some(x))"),
        "(BuiltinPat (Name deref) (TupleStructPat (Path (Segment (Name Some))) (IdentPat (Name x))))"
    );
}
