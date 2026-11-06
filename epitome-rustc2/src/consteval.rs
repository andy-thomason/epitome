//! Evaluating constant expressions: array lengths, enum discriminants,
//! `const` items of integer type, and const generic arguments.
//!
//! This is a small interpreter over the syntax tree, for what those use:
//! integer, `bool` and `char` literals, arithmetic, bit operations,
//! shifts and comparisons, `as` casts between integers, `if`, blocks with
//! a tail, paths to `const` items and const parameters, the integer
//! types' `MIN`, `MAX` and `BITS`, and `size_of::<T>()` and
//! `align_of::<T>()`. Integers are worked out in `i128`; where the type is
//! known (a cast, an array length) the value is wrapped to it. Anything
//! else is not a constant it can evaluate, and gives `None`.

use crate::ast::{Ast, Kind, NodeId};
use crate::hash::FastMap;
use crate::layout::LayoutCx;
use crate::resolve::{DefId, DefKind, Res};
use crate::ty::{Const, ConstId, IntTy, ParamId, TyKind, UintTy};

/// A constant's value.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Value {
    Int(i128),
    Bool(bool),
    Char(char),
}

impl Value {
    pub fn int(self) -> Option<i128> {
        match self {
            Value::Int(v) => Some(v),
            Value::Bool(b) => Some(b as i128),
            Value::Char(c) => Some(c as i128),
        }
    }
}

/// An integer type's width in bits and signedness (`usize` and `isize`
/// as the target's pointer).
pub fn int_bits(t: TyKind, ptr_bits: u32) -> Option<(u32, bool)> {
    Some(match t {
        TyKind::Int(i) => (
            match i {
                IntTy::I8 => 8,
                IntTy::I16 => 16,
                IntTy::I32 => 32,
                IntTy::I64 => 64,
                IntTy::I128 => 128,
                IntTy::Isize => ptr_bits,
            },
            true,
        ),
        TyKind::Uint(u) => (
            match u {
                UintTy::U8 => 8,
                UintTy::U16 => 16,
                UintTy::U32 => 32,
                UintTy::U64 => 64,
                UintTy::U128 => 128,
                UintTy::Usize => ptr_bits,
            },
            false,
        ),
        TyKind::Bool => (8, false),
        TyKind::Char => (32, false),
        _ => return None,
    })
}

/// `v` wrapped to `bits` bits, signed or not.
pub fn wrap(v: i128, bits: u32, signed: bool) -> i128 {
    if bits >= 128 {
        return v;
    }
    let m = (1i128 << bits) - 1;
    let u = v & m;
    if signed && u >> (bits - 1) & 1 == 1 { u - (1i128 << bits) } else { u }
}

/// The values of the const parameters in scope, by parameter.
pub type ConstEnv = FastMap<ParamId, i128>;

impl<'a, 's> LayoutCx<'a, 's> {
    /// The value of constant `c` (an array length, a const argument), with
    /// `env` giving const parameters' values.
    pub fn eval_const(&self, c: ConstId, env: &ConstEnv) -> Option<i128> {
        match self.tcx.const_of(c) {
            Const::Value(v) => Some(v as i128),
            Const::Param(p) => env.get(&p).copied(),
            Const::Expr(k, f, n) => self.eval(k, f, n, env)?.int(),
            Const::Error => None,
        }
    }

    /// The value of the expression `n` in crate `k`'s file `f`.
    pub fn eval(&self, k: u32, f: u32, n: NodeId, env: &ConstEnv) -> Option<Value> {
        let ast = &self.crates[k as usize].files[f as usize].ast;
        Eval { cx: self, ast, k, f, env, depth: 0 }.expr(n)
    }

    /// The value of the `const` item (or associated constant) `d`.
    pub fn eval_item(&self, d: DefId) -> Option<Value> {
        if let Some(v) = self.const_cache.lock().unwrap().get(&d) {
            return *v;
        }
        let def = &self.r.defs[d.0 as usize];
        let ast = &self.crates[def.krate as usize].files[def.file as usize].ast;
        // The initialiser: the item's last expression child.
        let init = ast.children(def.node).iter().rev().copied().find(|&c| is_value_expr(ast.kind(c)));
        // Guards against a constant defined in terms of itself.
        self.const_cache.lock().unwrap().insert(d, None);
        let v = init.and_then(|e| {
            let v = self.eval(def.krate, def.file, e, &ConstEnv::default())?;
            // Wrapped to its declared type.
            match (v, self.tcx.type_of(d).map(|t| self.tcx.kind(t))) {
                (Value::Int(i), Some(t)) => match int_bits(t, self.ptr_bits()) {
                    Some((bits, signed)) => Some(Value::Int(wrap(i, bits, signed))),
                    None => Some(v),
                },
                _ => Some(v),
            }
        });
        self.const_cache.lock().unwrap().insert(d, v);
        v
    }
}

fn is_value_expr(k: Kind) -> bool {
    matches!(
        k,
        Kind::Literal | Kind::PathExpr | Kind::Binary | Kind::Unary | Kind::Cast | Kind::Paren | Kind::Block | Kind::If | Kind::Call | Kind::ConstBlock
    )
}

struct Eval<'c, 'a, 's> {
    cx: &'c LayoutCx<'a, 's>,
    ast: &'c Ast<'s>,
    k: u32,
    f: u32,
    env: &'c ConstEnv,
    depth: u32,
}

impl Eval<'_, '_, '_> {
    fn kids(&self, n: NodeId) -> Vec<NodeId> {
        self.ast.children(n).iter().copied().filter(|&c| !matches!(self.ast.kind(c), Kind::Attribute | Kind::DocComment)).collect()
    }

    fn expr(&mut self, n: NodeId) -> Option<Value> {
        self.depth += 1;
        if self.depth > 64 {
            return None;
        }
        let ast = self.ast;
        let kids = self.kids(n);
        let v = match ast.kind(n) {
            Kind::Literal => literal(ast.token(n).0),
            Kind::Paren => self.expr(*kids.last()?),
            Kind::Block | Kind::ConstBlock => {
                // A block of only a tail expression (after `let`s we do not
                // follow).
                let tail = kids.iter().rev().copied().find(|&c| ast.kind(c) != Kind::Keyword)?;
                self.expr(tail)
            }
            Kind::Unary => {
                let v = self.expr(kids[1])?;
                match (ast.token(kids[0]).0, v) {
                    ("-", Value::Int(i)) => Some(Value::Int(i.wrapping_neg())),
                    ("!", Value::Int(i)) => Some(Value::Int(!i)),
                    ("!", Value::Bool(b)) => Some(Value::Bool(!b)),
                    _ => None,
                }
            }
            Kind::Binary => self.binary(&kids),
            Kind::Cast => {
                let v = self.expr(kids[0])?.int()?;
                let t = prim_named(&ast.text(kids[1]))?;
                let (bits, signed) = int_bits(t, self.cx.ptr_bits())?;
                Some(Value::Int(wrap(v, bits, signed)))
            }
            Kind::If => {
                let c = self.expr(kids[0])?;
                match c {
                    Value::Bool(true) => self.expr(kids[1]),
                    Value::Bool(false) => self.expr(*kids.get(2)?),
                    _ => None,
                }
            }
            Kind::PathExpr => self.path(kids[0]),
            Kind::Call => self.call(&kids),
            _ => None,
        };
        self.depth -= 1;
        v
    }

    fn binary(&mut self, kids: &[NodeId]) -> Option<Value> {
        let op = self.ast.text(kids[1]);
        let a = self.expr(kids[0])?;
        if op == "&&" || op == "||" {
            let Value::Bool(x) = a else { return None };
            if (op == "&&") != x {
                return Some(Value::Bool(x));
            }
            return self.expr(kids[2]);
        }
        let b = self.expr(kids[2])?;
        if let (Value::Bool(x), Value::Bool(y)) = (a, b) {
            return Some(Value::Bool(match op.as_str() {
                "==" => x == y,
                "!=" => x != y,
                "&" => x & y,
                "|" => x | y,
                "^" => x ^ y,
                _ => return None,
            }));
        }
        let (x, y) = (a.int()?, b.int()?);
        Some(match op.as_str() {
            "+" => Value::Int(x.wrapping_add(y)),
            "-" => Value::Int(x.wrapping_sub(y)),
            "*" => Value::Int(x.wrapping_mul(y)),
            "/" => Value::Int(x.checked_div(y)?),
            "%" => Value::Int(x.checked_rem(y)?),
            "&" => Value::Int(x & y),
            "|" => Value::Int(x | y),
            "^" => Value::Int(x ^ y),
            "<<" => Value::Int(x.checked_shl(u32::try_from(y).ok()?)?),
            ">>" => Value::Int(x.checked_shr(u32::try_from(y).ok()?)?),
            "==" => Value::Bool(x == y),
            "!=" => Value::Bool(x != y),
            "<" => Value::Bool(x < y),
            ">" => Value::Bool(x > y),
            "<=" => Value::Bool(x <= y),
            ">=" => Value::Bool(x >= y),
            _ => return None,
        })
    }

    fn res(&self, path: NodeId) -> Option<&Res> {
        self.cx.r.res_of(self.k as usize, self.f as usize, path)
    }

    fn path(&mut self, path: NodeId) -> Option<Value> {
        let cx = self.cx;
        match self.res(path)?.clone() {
            Res::Def(d) => match cx.r.defs[d.0 as usize].kind {
                DefKind::Const | DefKind::AssocConst => cx.eval_item(d),
                _ => None,
            },
            Res::Generic(p) => {
                let p = *cx.tcx.param_at.get(&(self.k, self.f, p))?;
                self.env.get(&p).map(|&v| Value::Int(v))
            }
            // `u32::MAX` and the like.
            Res::Partial(base, 1) => {
                let Res::PrimTy(name) = *base else { return None };
                let t = prim_named(name)?;
                let (bits, signed) = int_bits(t, cx.ptr_bits())?;
                let last = self.ast.children(path).iter().rev().find(|&&s| self.ast.kind(s) == Kind::Segment)?;
                Some(Value::Int(match self.ast.token(*last).0 {
                    "BITS" => bits as i128,
                    "MAX" if signed => (1i128 << (bits - 1)) - 1,
                    "MAX" if bits == 128 => i128::MAX, // u128::MAX does not fit; rarely a length.
                    "MAX" => (1i128 << bits) - 1,
                    "MIN" if signed => -(1i128 << (bits - 1)),
                    "MIN" => 0,
                    _ => return None,
                }))
            }
            _ => None,
        }
    }

    /// `size_of::<T>()` and `align_of::<T>()`.
    fn call(&mut self, kids: &[NodeId]) -> Option<Value> {
        let ast = self.ast;
        if ast.kind(kids[0]) != Kind::PathExpr {
            return None;
        }
        let path = ast.children(kids[0])[0];
        let Res::Def(d) = self.res(path)?.clone() else { return None };
        let name = self.cx.r.defs[d.0 as usize].name;
        if !matches!(name, "size_of" | "align_of") {
            return None;
        }
        let seg = ast.children(path).iter().rev().copied().find(|&s| ast.kind(s) == Kind::Segment)?;
        let g = ast.child(seg, Kind::GenericArgs)?;
        let t = *ast.children(g).first()?;
        let t = self.cx.lower_written(self.k, self.f, d, t)?;
        let l = self.cx.layout_of(t).ok()?;
        Some(Value::Int(if name == "size_of" { l.size } else { l.align } as i128))
    }
}

/// A primitive type by name.
pub fn prim_named(name: &str) -> Option<TyKind> {
    Some(match name.trim() {
        "u8" => TyKind::Uint(UintTy::U8),
        "u16" => TyKind::Uint(UintTy::U16),
        "u32" => TyKind::Uint(UintTy::U32),
        "u64" => TyKind::Uint(UintTy::U64),
        "u128" => TyKind::Uint(UintTy::U128),
        "usize" => TyKind::Uint(UintTy::Usize),
        "i8" => TyKind::Int(IntTy::I8),
        "i16" => TyKind::Int(IntTy::I16),
        "i32" => TyKind::Int(IntTy::I32),
        "i64" => TyKind::Int(IntTy::I64),
        "i128" => TyKind::Int(IntTy::I128),
        "isize" => TyKind::Int(IntTy::Isize),
        "bool" => TyKind::Bool,
        "char" => TyKind::Char,
        _ => return None,
    })
}

/// An integer, `bool` or `char` literal (a suffix is ignored).
fn literal(t: &str) -> Option<Value> {
    match t {
        "true" => return Some(Value::Bool(true)),
        "false" => return Some(Value::Bool(false)),
        _ => {}
    }
    if let Some(c) = t.strip_prefix('\'').and_then(|s| s.strip_suffix('\'')) {
        return unescape_char(c).map(Value::Char);
    }
    if let Some(c) = t.strip_prefix("b'").and_then(|s| s.strip_suffix('\'')) {
        return unescape_char(c).map(|c| Value::Int(c as i128));
    }
    let t = t.replace('_', "");
    let (radix, body) = match t.get(..2) {
        Some("0x") => (16, &t[2..]),
        Some("0o") => (8, &t[2..]),
        Some("0b") => (2, &t[2..]),
        _ => (10, &t[..]),
    };
    let end = body.find(|c: char| !c.is_digit(radix)).unwrap_or(body.len());
    if body[end..].starts_with('.') || (radix == 10 && body[end..].starts_with(['e', 'E'])) {
        return None;
    }
    u128::from_str_radix(&body[..end], radix).ok().map(|v| Value::Int(v as i128))
}

fn unescape_char(s: &str) -> Option<char> {
    let mut it = s.chars();
    let c = it.next()?;
    if c != '\\' {
        return Some(c);
    }
    Some(match it.next()? {
        'n' => '\n',
        't' => '\t',
        'r' => '\r',
        '0' => '\0',
        '\\' => '\\',
        '\'' => '\'',
        '"' => '"',
        'x' => char::from_u32(u32::from_str_radix(&s[2..4], 16).ok()?)?,
        'u' => char::from_u32(u32::from_str_radix(s[2..].trim_matches(['{', '}']), 16).ok()?)?,
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn literals_and_wrapping() {
        assert_eq!(literal("0x1F_u8"), Some(Value::Int(31)));
        assert_eq!(literal("1_000"), Some(Value::Int(1000)));
        assert_eq!(literal("'a'"), Some(Value::Char('a')));
        assert_eq!(literal("b'\\n'"), Some(Value::Int(10)));
        assert_eq!(literal("1.5"), None);
        assert_eq!(wrap(300, 8, false), 44);
        assert_eq!(wrap(255, 8, true), -1);
        assert_eq!(wrap(-1, 64, false), u64::MAX as i128);
    }
}
