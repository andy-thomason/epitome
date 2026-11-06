//! The built-in derives: `Clone`, `Copy`, `Debug`, `Default`, `PartialEq`,
//! `Eq`, `PartialOrd`, `Ord` and `Hash`.
//!
//! Each reads the shape of the item (its name, generic parameters, `where`
//! clause, and fields or variants) and writes an `impl` as source text,
//! which expansion parses like any macro's output. The impl has the item's
//! generic parameters (without defaults), each type parameter bounded by
//! the trait, and the item's `where` clause. Paths are written from
//! `::core`, which every crate can name (`core` itself through `extern
//! crate self as core`).
//!
//! The code is ours: it does what each trait's documentation says a
//! derive does (fields in declaration order; variants compared by their
//! order, then their fields), not what rustc's derives print.

use crate::ast::{Ast, Kind, NodeId};
use crate::pretty::pretty_node;
use std::fmt::Write;

/// The derives this module writes.
pub const BUILTIN_DERIVES: &[&str] =
    &["Clone", "Copy", "Debug", "Default", "PartialEq", "Eq", "PartialOrd", "Ord", "Hash"];

/// A struct's or variant's fields.
enum Fields {
    Named(Vec<String>),
    Tuple(usize),
    Unit,
}

enum Body {
    Struct(Fields),
    /// Variants: name, fields, and whether it is `#[default]`.
    Enum(Vec<(String, Fields, bool)>),
    Union,
}

/// What a derive needs to know about an item.
struct Shape {
    name: String,
    /// Generic parameters as the impl declares them, and the type
    /// parameters' names (to bound).
    params: Vec<String>,
    type_params: Vec<String>,
    /// The arguments `Name<...>` takes.
    args: Vec<String>,
    /// The item's own `where` predicates.
    preds: Vec<String>,
    body: Body,
}

fn fields(ast: &Ast, node: NodeId) -> Fields {
    if let Some(f) = ast.child(node, Kind::NamedFields) {
        let names = ast
            .children(f)
            .iter()
            .filter_map(|&d| ast.child(d, Kind::Name).map(|n| ast.token(n).0.to_string()))
            .collect();
        Fields::Named(names)
    } else if let Some(f) = ast.child(node, Kind::TupleFields) {
        Fields::Tuple(ast.children(f).len())
    } else {
        Fields::Unit
    }
}

fn shape(ast: &Ast, item: NodeId) -> Option<Shape> {
    let name = ast.token(ast.child(item, Kind::Name)?).0.to_string();
    let (mut params, mut type_params, mut args) = (Vec::new(), Vec::new(), Vec::new());
    if let Some(g) = ast.child(item, Kind::Generics) {
        for &p in ast.children(g) {
            let kids: Vec<NodeId> = ast
                .children(p)
                .iter()
                .copied()
                .filter(|&k| !matches!(ast.kind(k), Kind::Attribute | Kind::DocComment))
                .collect();
            match ast.kind(p) {
                Kind::LifetimeParam => {
                    params.push(pretty_node(ast, p));
                    args.push(ast.text(kids[0]));
                }
                Kind::TypeParam => {
                    let n = ast.text(kids[0]);
                    let bounds = ast.child(p, Kind::Bounds).map(|b| pretty_node(ast, b));
                    params.push(match bounds {
                        Some(b) if !b.is_empty() => format!("{n}: {b}"),
                        _ => n.clone(),
                    });
                    type_params.push(n.clone());
                    args.push(n);
                }
                Kind::ConstParam => {
                    let n = ast.text(kids[0]);
                    params.push(format!("const {n}: {}", pretty_node(ast, kids[1])));
                    args.push(n);
                }
                _ => {}
            }
        }
    }
    let preds = ast
        .child(item, Kind::WhereClause)
        .map(|w| ast.children(w).iter().map(|&p| pretty_node(ast, p)).collect())
        .unwrap_or_default();
    let body = match ast.kind(item) {
        Kind::Struct => Body::Struct(fields(ast, item)),
        Kind::Union => Body::Union,
        Kind::Enum => Body::Enum(
            ast.children(item)
                .iter()
                .filter(|&&v| ast.kind(v) == Kind::Variant)
                .map(|&v| {
                    let vname = ast.token(ast.child(v, Kind::Name).unwrap()).0.to_string();
                    let is_default = ast.children(v).iter().any(|&a| {
                        ast.kind(a) == Kind::Attribute
                            && ast.child(a, Kind::Path).is_some_and(|p| ast.text(p) == "default")
                    });
                    (vname, fields(ast, v), is_default)
                })
                .collect(),
        ),
        _ => return None,
    };
    Some(Shape { name, params, type_params, args, preds, body })
}

impl Shape {
    /// `impl<...> path for Name<...> where ... {` and the closing `}`
    /// around `items`.
    fn imp(&self, path: &str, items: &str) -> String {
        let generics = if self.params.is_empty() { String::new() } else { format!("<{}>", self.params.join(", ")) };
        let args = if self.args.is_empty() { String::new() } else { format!("<{}>", self.args.join(", ")) };
        let mut preds = self.preds.clone();
        preds.extend(self.type_params.iter().map(|t| format!("{t}: {path}")));
        let wh = if preds.is_empty() { String::new() } else { format!(" where {}", preds.join(", ")) };
        format!("#[automatically_derived] impl{generics} {path} for {}{args}{wh} {{ {items} }}", self.name)
    }
}

/// The pattern for a struct or variant `path`, binding its fields as
/// `prefix0`, `prefix1`, ...; and the binding names.
fn bind(path: &str, f: &Fields, prefix: &str) -> (String, Vec<String>) {
    match f {
        Fields::Named(names) => {
            let vars: Vec<String> = (0..names.len()).map(|i| format!("{prefix}{i}")).collect();
            let inner: Vec<String> = names.iter().zip(&vars).map(|(n, v)| format!("{n}: {v}")).collect();
            (format!("{path} {{ {} }}", inner.join(", ")), vars)
        }
        Fields::Tuple(n) => {
            let vars: Vec<String> = (0..*n).map(|i| format!("{prefix}{i}")).collect();
            (format!("{path}({})", vars.join(", ")), vars)
        }
        Fields::Unit => (path.to_string(), Vec::new()),
    }
}

/// A struct's or variant's value at `path`, built from field expressions.
fn build(path: &str, f: &Fields, values: &[String]) -> String {
    match f {
        Fields::Named(names) => {
            let inner: Vec<String> = names.iter().zip(values).map(|(n, v)| format!("{n}: {v}")).collect();
            format!("{path} {{ {} }}", inner.join(", "))
        }
        Fields::Tuple(_) => format!("{path}({})", values.join(", ")),
        Fields::Unit => path.to_string(),
    }
}

/// `&self.a` for each field of a struct.
fn self_fields(f: &Fields, of: &str) -> Vec<String> {
    match f {
        Fields::Named(names) => names.iter().map(|n| format!("&{of}.{n}")).collect(),
        Fields::Tuple(n) => (0..*n).map(|i| format!("&{of}.{i}")).collect(),
        Fields::Unit => Vec::new(),
    }
}

/// Field names for `Debug`.
fn field_names(f: &Fields) -> Vec<Option<String>> {
    match f {
        Fields::Named(names) => names.iter().map(|n| Some(n.clone())).collect(),
        Fields::Tuple(n) => vec![None; *n],
        Fields::Unit => Vec::new(),
    }
}

/// `f.debug_struct("N").field("a", a)...finish()` and the like, for values
/// already references.
fn debug_expr(name: &str, f: &Fields, values: &[String]) -> String {
    let names = field_names(f);
    match f {
        Fields::Unit => format!("::core::fmt::Formatter::write_str(f, \"{name}\")"),
        Fields::Named(_) => {
            let mut s = format!("::core::fmt::Formatter::debug_struct(f, \"{name}\")");
            for (n, v) in names.iter().zip(values) {
                let _ = write!(s, ".field(\"{}\", {v})", n.as_deref().unwrap_or(""));
            }
            s + ".finish()"
        }
        Fields::Tuple(_) => {
            let mut s = format!("::core::fmt::Formatter::debug_tuple(f, \"{name}\")");
            for v in values {
                let _ = write!(s, ".field({v})");
            }
            s + ".finish()"
        }
    }
}

/// Comparison of `a[i]` with `b[i]` in turn, stopping at the first that is
/// not equal: for `PartialOrd` (`partial`) or `Ord`.
fn compare_chain(a: &[String], b: &[String], partial: bool) -> String {
    let (call, equal, done) = if partial {
        (
            "::core::cmp::PartialOrd::partial_cmp",
            "::core::option::Option::Some(::core::cmp::Ordering::Equal)",
            "::core::option::Option::Some(::core::cmp::Ordering::Equal)",
        )
    } else {
        ("::core::cmp::Ord::cmp", "::core::cmp::Ordering::Equal", "::core::cmp::Ordering::Equal")
    };
    let mut out = done.to_string();
    for (x, y) in a.iter().zip(b).rev() {
        out = format!("match {call}({x}, {y}) {{ {equal} => {out}, cmp => cmp, }}");
    }
    out
}

/// `match self { Self::A(..) => 0isize, ... }`: the variant's position.
fn variant_index(variants: &[(String, Fields, bool)], of: &str) -> String {
    if variants.is_empty() {
        return format!("match *{of} {{}}");
    }
    let arms: Vec<String> = variants
        .iter()
        .enumerate()
        .map(|(i, (v, f, _))| {
            let pat = match f {
                Fields::Named(_) => format!("Self::{v} {{ .. }}"),
                Fields::Tuple(_) => format!("Self::{v}(..)"),
                Fields::Unit => format!("Self::{v}"),
            };
            format!("{pat} => {i}isize,")
        })
        .collect();
    format!("match {of} {{ {} }}", arms.join(" "))
}

/// The impl for derive `which` on `item`, as source text, or why there is
/// none.
pub fn derive(ast: &Ast, item: NodeId, which: &str) -> Result<String, String> {
    let s = shape(ast, item).ok_or("`derive` applies only to structs, enums and unions")?;
    let text = match which {
        "Copy" => s.imp("::core::marker::Copy", ""),
        "Eq" => s.imp("::core::cmp::Eq", ""),
        "Clone" => {
            let body = match &s.body {
                Body::Struct(f) => {
                    let vals: Vec<String> =
                        self_fields(f, "self").iter().map(|r| format!("::core::clone::Clone::clone({r})")).collect();
                    build("Self", f, &vals)
                }
                Body::Union => "*self".to_string(),
                Body::Enum(vs) if vs.is_empty() => "match *self {}".to_string(),
                Body::Enum(vs) => {
                    let arms: Vec<String> = vs
                        .iter()
                        .map(|(v, f, _)| {
                            let (pat, vars) = bind(&format!("Self::{v}"), f, "__self_");
                            let vals: Vec<String> =
                                vars.iter().map(|x| format!("::core::clone::Clone::clone({x})")).collect();
                            format!("{pat} => {},", build(&format!("Self::{v}"), f, &vals))
                        })
                        .collect();
                    format!("match self {{ {} }}", arms.join(" "))
                }
            };
            s.imp("::core::clone::Clone", &format!("#[inline] fn clone(&self) -> Self {{ {body} }}"))
        }
        "Debug" => {
            let body = match &s.body {
                Body::Struct(f) => debug_expr(&s.name, f, &self_fields(f, "self")),
                Body::Union => return Err("`Debug` cannot be derived for a union".into()),
                Body::Enum(vs) if vs.is_empty() => "match *self {}".to_string(),
                Body::Enum(vs) => {
                    let arms: Vec<String> = vs
                        .iter()
                        .map(|(v, f, _)| {
                            let (pat, vars) = bind(&format!("Self::{v}"), f, "__self_");
                            format!("{pat} => {},", debug_expr(v, f, &vars))
                        })
                        .collect();
                    format!("match self {{ {} }}", arms.join(" "))
                }
            };
            s.imp(
                "::core::fmt::Debug",
                &format!("fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {{ {body} }}"),
            )
        }
        "Default" => {
            let default = "::core::default::Default::default()";
            let body = match &s.body {
                Body::Struct(f) => {
                    let n = match f {
                        Fields::Named(v) => v.len(),
                        Fields::Tuple(n) => *n,
                        Fields::Unit => 0,
                    };
                    build("Self", f, &vec![default.to_string(); n])
                }
                Body::Union => return Err("`Default` cannot be derived for a union".into()),
                Body::Enum(vs) => {
                    let Some((v, f, _)) = vs.iter().find(|v| v.2) else {
                        return Err("`#[derive(Default)]` on an enum needs a `#[default]` variant".into());
                    };
                    if !matches!(f, Fields::Unit) {
                        return Err("the `#[default]` variant must be a unit variant".into());
                    }
                    format!("Self::{v}")
                }
            };
            s.imp("::core::default::Default", &format!("#[inline] fn default() -> Self {{ {body} }}"))
        }
        "PartialEq" => {
            let body = match &s.body {
                Body::Struct(f) => {
                    let a = self_fields(f, "self");
                    let b = self_fields(f, "other");
                    let parts: Vec<String> = a.iter().zip(&b).map(|(x, y)| format!("*{x} == *{y}")).collect();
                    if parts.is_empty() { "true".to_string() } else { parts.join(" && ") }
                }
                Body::Union => return Err("`PartialEq` cannot be derived for a union".into()),
                Body::Enum(vs) if vs.is_empty() => "match *self {}".to_string(),
                Body::Enum(vs) => {
                    let mut arms: Vec<String> = vs
                        .iter()
                        .map(|(v, f, _)| {
                            let (pa, va) = bind(&format!("Self::{v}"), f, "__self_");
                            let (pb, vb) = bind(&format!("Self::{v}"), f, "__arg1_");
                            let parts: Vec<String> = va.iter().zip(&vb).map(|(x, y)| format!("*{x} == *{y}")).collect();
                            let e = if parts.is_empty() { "true".to_string() } else { parts.join(" && ") };
                            format!("({pa}, {pb}) => {e},")
                        })
                        .collect();
                    if vs.len() > 1 {
                        arms.push("_ => false,".into());
                    }
                    format!("match (self, other) {{ {} }}", arms.join(" "))
                }
            };
            s.imp("::core::cmp::PartialEq", &format!("#[inline] fn eq(&self, other: &Self) -> bool {{ {body} }}"))
        }
        "PartialOrd" | "Ord" => {
            let partial = which == "PartialOrd";
            let body = match &s.body {
                Body::Struct(f) => compare_chain(&self_fields(f, "self"), &self_fields(f, "other"), partial),
                Body::Union => return Err(format!("`{which}` cannot be derived for a union")),
                Body::Enum(vs) if vs.is_empty() => "match *self {}".to_string(),
                Body::Enum(vs) => {
                    let mut arms: Vec<String> = vs
                        .iter()
                        .map(|(v, f, _)| {
                            let (pa, va) = bind(&format!("Self::{v}"), f, "__self_");
                            let (pb, vb) = bind(&format!("Self::{v}"), f, "__arg1_");
                            format!("({pa}, {pb}) => {},", compare_chain(&va, &vb, partial))
                        })
                        .collect();
                    let call = if partial { "::core::cmp::PartialOrd::partial_cmp" } else { "::core::cmp::Ord::cmp" };
                    if vs.len() > 1 {
                        arms.push(format!("_ => {call}(&__self_discr, &__arg1_discr),"));
                    }
                    format!(
                        "let __self_discr = {}; let __arg1_discr = {}; match (self, other) {{ {} }}",
                        variant_index(vs, "self"),
                        variant_index(vs, "other"),
                        arms.join(" ")
                    )
                }
            };
            if partial {
                s.imp(
                    "::core::cmp::PartialOrd",
                    &format!(
                        "#[inline] fn partial_cmp(&self, other: &Self) -> ::core::option::Option<::core::cmp::Ordering> {{ {body} }}"
                    ),
                )
            } else {
                s.imp(
                    "::core::cmp::Ord",
                    &format!("#[inline] fn cmp(&self, other: &Self) -> ::core::cmp::Ordering {{ {body} }}"),
                )
            }
        }
        "Hash" => {
            let hash = |x: &str| format!("::core::hash::Hash::hash({x}, state);");
            let body = match &s.body {
                Body::Struct(f) => self_fields(f, "self").iter().map(|x| hash(x)).collect::<Vec<_>>().join(" "),
                Body::Union => return Err("`Hash` cannot be derived for a union".into()),
                Body::Enum(vs) if vs.is_empty() => "match *self {}".to_string(),
                Body::Enum(vs) => {
                    let arms: Vec<String> = vs
                        .iter()
                        .map(|(v, f, _)| {
                            let (pat, vars) = bind(&format!("Self::{v}"), f, "__self_");
                            let stmts: Vec<String> = vars.iter().map(|x| hash(x)).collect();
                            format!("{pat} => {{ {} }}", stmts.join(" "))
                        })
                        .collect();
                    format!(
                        "let __self_discr = {}; ::core::hash::Hash::hash(&__self_discr, state); match self {{ {} }}",
                        variant_index(vs, "self"),
                        arms.join(" ")
                    )
                }
            };
            s.imp(
                "::core::hash::Hash",
                &format!("fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) {{ {body} }}"),
            )
        }
        _ => return Err(format!("`{which}` is not a built-in derive")),
    };
    Ok(text)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Edition;
    use crate::parse::parse_str;

    fn derive_all(src: &str) -> Vec<String> {
        let ast = parse_str(src, Edition::E2024).unwrap();
        let item = ast.children(ast.root)[0];
        BUILTIN_DERIVES.iter().filter_map(|d| derive(&ast, item, d).ok()).collect()
    }

    /// Every impl parses as an item.
    fn parses(text: &str) {
        if let Err(e) = parse_str(text, Edition::E2024) {
            panic!("{text}\ndoes not parse: {} at {:?}", e.message, e.at);
        }
    }

    #[test]
    fn structs() {
        let impls = derive_all("struct P<'a, T: Copy = u8, const N: usize> where T: Send { x: &'a T, y: [u8; N] }");
        assert_eq!(impls.len(), 9);
        for i in &impls {
            parses(i);
        }
        assert!(impls[0].starts_with(
            "#[automatically_derived] impl<'a, T: Copy, const N: usize> ::core::clone::Clone for P<'a, T, N> where T: Send, T: ::core::clone::Clone {"
        ));
        assert!(
            impls[0]
                .contains("Self { x: ::core::clone::Clone::clone(&self.x), y: ::core::clone::Clone::clone(&self.y) }")
        );
        let t = derive_all("struct T(u8, u16);");
        assert!(t[2].contains("::core::fmt::Formatter::debug_tuple(f, \"T\").field(&self.0).field(&self.1).finish()"));
        let u = derive_all("struct U;");
        assert!(u[4].contains("fn eq(&self, other: &Self) -> bool { true }"));
    }

    #[test]
    fn enums() {
        let src = "enum E { A, B(u8), #[default] C, D { x: u8 } }";
        let impls = derive_all(src);
        assert_eq!(impls.len(), 9);
        for i in &impls {
            parses(i);
        }
        assert!(impls[3].contains("fn default() -> Self { Self::C }"));
        assert!(impls[4].contains("(Self::B(__self_0), Self::B(__arg1_0)) => *__self_0 == *__arg1_0,"));
        assert!(impls[6].contains("let __self_discr = match self { Self::A => 0isize, Self::B(..) => 1isize,"));
        // An enum without `#[default]` has no `Default`.
        assert_eq!(derive_all("enum F { A }").len(), 8);
    }
}
