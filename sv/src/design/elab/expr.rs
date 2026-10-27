//! Expressions: names resolved, types worked out, and clause 11's rules
//! for widths and signedness applied once (11.6, 11.8).
//!
//! `bind` gives each node its self-determined type. Where an operator's
//! operands are context-determined, the context's type is then pushed down
//! (`ctx`): the operator takes the final type, and each operand whose own
//! type differs is wrapped in a `Convert`. `expr` is `bind` followed by
//! pushing the expression's own type down, for self-determined uses;
//! `rvalue` pushes the type an assignment gives it (10.7).

use super::{Elab, R, eval, lookup, ty};
use crate::ast::{Kind, NodeId};
use crate::design::types::{Type, TypeId, Types, UDim};
use crate::design::value::{Bits, Logic, parse_digits};
use crate::design::*;

/// A self-determined expression.
pub(crate) fn expr<'a>(e: &mut Elab<'a, '_>, sc: ScopeId, n: NodeId) -> R<'a, ExprId> {
    let x = bind(e, sc, n)?;
    Ok(selfdet(e, x))
}

/// An expression assigned to something of type `target`: its width is the
/// larger of its own and the target's, and it is then converted.
pub(crate) fn rvalue<'a>(e: &mut Elab<'a, '_>, sc: ScopeId, n: NodeId, target: TypeId) -> R<'a, ExprId> {
    // Patterns and unpacked concatenations take their shape from the target.
    match e.kind(n) {
        Kind::AssignmentPattern => return pattern(e, sc, n, Some(target)),
        Kind::Concat if e.d.types.is_unpacked_array(target) => return unpacked_concat(e, sc, n, target),
        Kind::New => return new_expr(e, sc, n, Some(target)),
        Kind::Paren => return rvalue(e, sc, e.kids(n)[0], target),
        _ => {}
    }
    let x = bind(e, sc, n)?;
    let x = context_to(e, x, target);
    super::checks::assign_compatible(e, sc, target, x, n)?;
    Ok(assign_conv(e, x, target))
}

/// Pushes an assignment's width down: the expression's own signedness,
/// the larger width.
pub(crate) fn context_to(e: &mut Elab, x: ExprId, target: TypeId) -> ExprId {
    let xt = e.ex(x).ty;
    if e.d.types.is_integral(xt) && e.d.types.is_integral(target) {
        let w = e.d.types.width(xt).max(e.d.types.width(target));
        let s = e.d.types.is_signed(xt);
        let f = e.d.types.is_four_state(xt);
        let t = if e.d.types.width(xt) == w {
            // Already as wide: an enum or struct keeps its own type.
            xt
        } else {
            e.d.types.vector(w, s, f)
        };
        ctx(e, x, t)
    } else {
        selfdet(e, x)
    }
}

pub(crate) fn selfdet(e: &mut Elab, x: ExprId) -> ExprId {
    let t = e.ex(x).ty;
    if e.d.types.is_integral(t) { ctx(e, x, t) } else { x }
}

/// Converts `x` to `ty` if it is not already of that type.
pub(crate) fn assign_conv(e: &mut Elab, x: ExprId, t: TypeId) -> ExprId {
    let xt = e.ex(x).ty;
    if xt == t || t == Types::UNKNOWN || xt == Types::UNKNOWN {
        return x;
    }
    // An unbased unsized literal fills its target (5.7.1).
    if e.d.types.is_integral(t) && is_unsized_fill(e, x) {
        return fill_literal(e, x, t);
    }
    let node = e.ex(x).node;
    e.new_expr(ExprKind::Convert, t, node, vec![x])
}

fn is_unsized_fill(e: &Elab, x: ExprId) -> bool {
    matches!(e.ex(x).kind, ExprKind::Const(_)) && e.kind(e.ex(x).node) == Kind::UnbasedUnsized
}

fn fill_literal(e: &mut Elab, x: ExprId, t: TypeId) -> ExprId {
    let node = e.ex(x).node;
    let ExprKind::Const(c) = e.ex(x).kind else { return x };
    let bit = e.d.consts[c.0 as usize].as_bits().map_or(Logic::X, |b| b.bit(0));
    let w = e.d.types.width(t);
    let signed = e.d.types.is_signed(t);
    let c2 = e.new_const(Const::Bits(Bits::fill(w, signed, bit)));
    e.new_expr(ExprKind::Const(c2), t, node, vec![])
}

/// Gives a context-determined expression the type `t` (11.8.2).
pub(crate) fn ctx(e: &mut Elab, x: ExprId, t: TypeId) -> ExprId {
    let xe = e.ex(x).clone();
    if xe.ty == Types::UNKNOWN || t == Types::UNKNOWN {
        return x;
    }
    use ExprKind as K;
    let same_shape = |e: &Elab, a: TypeId| a == t || (matches!(e.d.types.get(a), Type::Vector { .. }) && false);
    match &xe.kind {
        K::Binary(op)
            if matches!(
                op,
                BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div | BinOp::Mod | BinOp::And | BinOp::Or | BinOp::Xor | BinOp::Xnor
            ) && !e.d.types.is_real(xe.ty) =>
        {
            let a = ctx(e, xe.args[0], t);
            let b = ctx(e, xe.args[1], t);
            let ex = &mut e.d.exprs[x.0 as usize];
            ex.ty = t;
            ex.args = vec![a, b];
            x
        }
        K::Binary(op) if matches!(op, BinOp::Pow | BinOp::Shl | BinOp::Shr | BinOp::AShl | BinOp::AShr) && !e.d.types.is_real(xe.ty) => {
            let a = ctx(e, xe.args[0], t);
            let ex = &mut e.d.exprs[x.0 as usize];
            ex.ty = t;
            ex.args[0] = a;
            x
        }
        K::Unary(UnOp::Plus | UnOp::Neg | UnOp::Not) if !e.d.types.is_real(xe.ty) => {
            let a = ctx(e, xe.args[0], t);
            let ex = &mut e.d.exprs[x.0 as usize];
            ex.ty = t;
            ex.args = vec![a];
            x
        }
        K::Cond if e.d.types.is_integral(xe.ty) => {
            let a = ctx(e, xe.args[1], t);
            let b = ctx(e, xe.args[2], t);
            let ex = &mut e.d.exprs[x.0 as usize];
            ex.ty = t;
            ex.args[1] = a;
            ex.args[2] = b;
            x
        }
        _ => {
            if same_shape(e, xe.ty) {
                return x;
            }
            if !e.d.types.is_integral(xe.ty) && !e.d.types.is_real(xe.ty) {
                return x;
            }
            assign_conv(e, x, t)
        }
    }
}

/// The self-determined type of an arithmetic result over these operands.
fn arith_type(e: &mut Elab, a: TypeId, b: TypeId) -> TypeId {
    if a == Types::UNKNOWN || b == Types::UNKNOWN {
        return Types::UNKNOWN;
    }
    if e.d.types.is_real(a) || e.d.types.is_real(b) {
        return Types::REAL;
    }
    if !e.d.types.is_integral(a) || !e.d.types.is_integral(b) {
        return Types::UNKNOWN;
    }
    let w = e.d.types.width(a).max(e.d.types.width(b));
    let s = e.d.types.is_signed(a) && e.d.types.is_signed(b);
    let f = e.d.types.is_four_state(a) || e.d.types.is_four_state(b);
    e.d.types.vector(w, s, f)
}

/// The type of an integral operand, as a plain vector.
fn vector_of(e: &mut Elab, a: TypeId) -> TypeId {
    if matches!(e.d.types.get(a), Type::Vector { dims, .. } if dims.len() <= 1) {
        return a;
    }
    if !e.d.types.is_integral(a) {
        return a;
    }
    let w = e.d.types.width(a);
    let s = e.d.types.is_signed(a);
    let f = e.d.types.is_four_state(a);
    e.d.types.vector(w, s, f)
}

fn bool_type(e: &mut Elab, four: bool) -> TypeId {
    if four { Types::LOGIC } else { Types::BIT }
}

fn binop(op: &str) -> Option<BinOp> {
    use BinOp::*;
    Some(match op {
        "+" => Add,
        "-" => Sub,
        "*" => Mul,
        "/" => Div,
        "%" => Mod,
        "**" => Pow,
        "&" => And,
        "|" => Or,
        "^" => Xor,
        "~^" | "^~" => Xnor,
        "<<" => Shl,
        ">>" => Shr,
        "<<<" => AShl,
        ">>>" => AShr,
        "&&" => LogAnd,
        "||" => LogOr,
        "->" => Implies,
        "<->" => Equiv,
        "==" => Eq,
        "!=" => Ne,
        "===" => CaseEq,
        "!==" => CaseNe,
        "==?" => WildEq,
        "!=?" => WildNe,
        "<" => Lt,
        "<=" => Le,
        ">" => Gt,
        ">=" => Ge,
        _ => return None,
    })
}

/// A compound assignment's operator: `+=` is `+`.
pub(crate) fn compound_op(op: &str) -> Option<BinOp> {
    let base = op.strip_suffix('=')?;
    match base {
        "<<<" | ">>>" | "<<" | ">>" | "+" | "-" | "*" | "/" | "%" | "&" | "|" | "^" => binop(base),
        _ => None,
    }
}

/// An expression with its self-determined type; context not yet applied.
pub(crate) fn bind<'a>(e: &mut Elab<'a, '_>, sc: ScopeId, n: NodeId) -> R<'a, ExprId> {
    e.depth += 1;
    if e.depth > 2000 {
        e.depth -= 1;
        return Err(e.err(n, "the expression is nested too deeply"));
    }
    let r = bind_inner(e, sc, n);
    e.depth -= 1;
    r
}

fn bind_inner<'a>(e: &mut Elab<'a, '_>, sc: ScopeId, n: NodeId) -> R<'a, ExprId> {
    let kids = e.kids(n).to_vec();
    match e.kind(n) {
        Kind::Number => number(e, n),
        Kind::UnbasedUnsized => {
            let l = match e.text(n).as_bytes().get(1) {
                Some(b'0') => Logic::Zero,
                Some(b'1') => Logic::One,
                Some(b'x' | b'X') => Logic::X,
                _ => Logic::Z,
            };
            let c = e.new_const(Const::Bits(Bits::fill(1, false, l)));
            Ok(e.new_expr(ExprKind::Const(c), Types::LOGIC, n, vec![]))
        }
        Kind::String => {
            let bytes = string_bytes(e.text(n));
            let w = (bytes.len() as u32 * 8).max(8);
            let t = e.d.types.vector(w, false, false);
            let c = e.new_const(Const::Str(bytes));
            Ok(e.new_expr(ExprKind::Const(c), t, n, vec![]))
        }
        Kind::TimeLiteral => {
            let v = eval::time_literal(e, sc, n);
            let c = e.new_const(Const::Real(v));
            Ok(e.new_expr(ExprKind::Const(c), Types::REALTIME, n, vec![]))
        }
        Kind::Name => name(e, sc, n),
        Kind::SystemName => syscall(e, sc, n, n, &[]),
        Kind::Dollar => Ok(e.new_expr(ExprKind::Dollar, Types::INT, n, vec![])),
        Kind::Null => Ok(e.new_expr(ExprKind::Null, Types::NULL, n, vec![])),
        Kind::This => {
            let t = match lookup::enclosing(e, sc, ScopeKind::Class).and_then(|c| lookup::class_of_scope(e, c)) {
                Some(c) => e.d.types.intern(Type::Class(c)),
                None => return Err(e.err(n, "`this` can only be used inside a class (8.11)")),
            };
            super::checks::this_allowed(e, sc, n)?;
            Ok(e.new_expr(ExprKind::This, t, n, vec![]))
        }
        Kind::Super => {
            let c = lookup::enclosing(e, sc, ScopeKind::Class).and_then(|c| lookup::class_of_scope(e, c));
            let base = c.and_then(|c| e.d.classes[c.0 as usize].base);
            let t = match (c, base) {
                (None, _) => return Err(e.err(n, "`super` can only be used inside a class (8.15)")),
                (Some(_), None) => {
                    if c.is_some_and(|c| e.d.classes[c.0 as usize].node != NodeId(0)
                        && e.child(e.d.classes[c.0 as usize].node, Kind::Extends).is_none())
                    {
                        return Err(e.err(n, "`super` is used in a class that extends nothing (8.15)"));
                    }
                    Types::UNKNOWN
                }
                (Some(_), Some(b)) => e.d.types.intern(Type::Class(b)),
            };
            Ok(e.new_expr(ExprKind::This, t, n, vec![]))
        }
        Kind::Paren => bind(e, sc, kids[0]),
        Kind::MinTypMax => bind(e, sc, kids[1]),
        Kind::Binary => binary(e, sc, n, &kids),
        Kind::Unary => unary(e, sc, n, &kids),
        Kind::IncDec => {
            let (op, target) = if e.kind(kids[0]) == Kind::Op { (e.text(kids[0]), kids[1]) } else { (e.text(kids[1]), kids[0]) };
            let pre = e.kind(kids[0]) == Kind::Op;
            let x = expr(e, sc, target)?;
            super::checks::lvalue(e, sc, x, target, false)?;
            let u = match (op, pre) {
                ("++", true) => UnOp::PreInc,
                ("--", true) => UnOp::PreDec,
                ("++", false) => UnOp::PostInc,
                _ => UnOp::PostDec,
            };
            let t = e.ex(x).ty;
            Ok(e.new_expr(ExprKind::Unary(u), t, n, vec![x]))
        }
        Kind::Conditional => {
            // `(x matches p) ? a : b`: the pattern's variables are seen in a.
            let (c, asc) = if matches!(e.kind(kids[0]), Kind::CondPredicate | Kind::CondPattern)
                || (e.kind(kids[0]) == Kind::Paren && matches!(e.kind(e.kids(kids[0])[0]), Kind::CondPredicate | Kind::CondPattern))
            {
                let inner = if e.kind(kids[0]) == Kind::Paren { e.kids(kids[0])[0] } else { kids[0] };
                let scope = e.new_scope(ScopeKind::Block, None, Some(sc), inner);
                super::stmt::pattern_vars(e, scope, inner)?;
                let v = e.kids(inner).first().copied().unwrap_or(inner);
                let v = if e.kind(v) == Kind::CondPattern { e.kids(v)[0] } else { v };
                (expr(e, sc, v)?, scope)
            } else {
                (expr(e, sc, kids[0])?, sc)
            };
            let a = bind(e, asc, kids[1])?;
            let b = bind(e, sc, kids[2])?;
            let (ta, tb) = (e.ex(a).ty, e.ex(b).ty);
            let t = if e.d.types.is_integral(ta) && e.d.types.is_integral(tb) || e.d.types.is_real(ta) || e.d.types.is_real(tb) {
                arith_type(e, ta, tb)
            } else if ta == tb {
                ta
            } else if tb == Types::NULL {
                ta
            } else if ta == Types::NULL {
                tb
            } else {
                // Class handles of related classes, strings and the like.
                if ta == Types::STRING || tb == Types::STRING { Types::STRING } else { ta }
            };
            let (a, b) = if e.d.types.is_real(t) {
                (assign_conv(e, a, t), assign_conv(e, b, t))
            } else if e.d.types.is_integral(t) {
                (a, b)
            } else {
                (selfdet(e, a), selfdet(e, b))
            };
            Ok(e.new_expr(ExprKind::Cond, t, n, vec![c, a, b]))
        }
        Kind::Inside => {
            let v = expr(e, sc, kids[0])?;
            let mut args = vec![v];
            for &r in e.kids(kids[1]).to_vec().iter() {
                args.push(range_item(e, sc, r)?);
            }
            let four = e.d.types.is_four_state(e.ex(v).ty);
            let t = bool_type(e, four);
            Ok(e.new_expr(ExprKind::Inside, t, n, args))
        }
        Kind::AssignExpr => {
            let a = kids[0];
            let ak = e.kids(a).to_vec();
            let lhs = expr(e, sc, ak[0])?;
            super::checks::lvalue(e, sc, lhs, ak[0], false)?;
            let lt = e.ex(lhs).ty;
            let op = e.text(ak[1]);
            let rhs = if op == "=" {
                rvalue(e, sc, *ak.last().unwrap(), lt)?
            } else {
                let r = expr(e, sc, *ak.last().unwrap())?;
                let l2 = e.new_expr(ExprKind::Object(ObjId(0)), lt, ak[0], vec![]);
                let _ = l2;
                r
            };
            Ok(e.new_expr(ExprKind::Assign, lt, n, vec![lhs, rhs]))
        }
        Kind::Concat => concat(e, sc, n, &kids),
        Kind::Replicate => {
            let inner = concat(e, sc, kids[1], &e.kids(kids[1]).to_vec())?;
            let it = e.ex(inner).ty;
            // String literals replicated a variable number of times are
            // strings (11.4.12.2).
            let literal_strings = e.ex(inner).args.iter().all(|&a| matches!(e.ex(a).kind, ExprKind::Const(c) if matches!(e.d.consts[c.0 as usize], Const::Str(_))));
            let constant = eval::const_expr(e, sc, kids[0]).is_ok();
            if it == Types::STRING || (literal_strings && !constant) {
                // A string may be replicated a variable number of times
                // (11.4.12.2): the count is then an operand.
                if let Some(k) = eval::const_expr(e, sc, kids[0]).ok().and_then(|c| c.to_i64()) {
                    return Ok(e.new_expr(ExprKind::Replicate(k.max(0) as u32), Types::STRING, n, vec![inner]));
                }
                let c = expr(e, sc, kids[0])?;
                return Ok(e.new_expr(ExprKind::Replicate(u32::MAX), Types::STRING, n, vec![inner, c]));
            }
            let count = eval::const_expr(e, sc, kids[0])?;
            let Some(k) = count.to_i64().filter(|&k| k >= 0) else {
                return Err(e.err(kids[0], "a replication count must be a non-negative constant (11.4.12.1)"));
            };
            let w = e.d.types.width(it) * k as u32;
            let f = e.d.types.is_four_state(it);
            let t = if w == 0 { Types::UNKNOWN } else { e.d.types.vector(w, false, f) };
            Ok(e.new_expr(ExprKind::Replicate(k as u32), t, n, vec![inner]))
        }
        Kind::Streaming => {
            let mut args = Vec::new();
            let left = e.text(kids[0]) == "<<";
            let mut w = 0;
            let mut known = true;
            for &k in &kids[1..] {
                if e.kind(k) == Kind::StreamExpr {
                    let sk = e.kids(k).to_vec();
                    let x = expr(e, sc, sk[0])?;
                    let tw = e.d.types.width(e.ex(x).ty);
                    if tw == 0 {
                        known = false;
                    }
                    w += tw;
                    args.push(x);
                } else if ty::is_type(e.kind(k)) {
                    let t = ty::data_type(e, sc, k)?;
                    args.insert(0, e.new_expr(ExprKind::TypeRef(t), Types::TYPE, k, vec![]));
                } else {
                    let x = expr(e, sc, k)?;
                    args.insert(0, x);
                }
            }
            let t = if known && w > 0 { e.d.types.vector(w, false, true) } else { Types::UNKNOWN };
            Ok(e.new_expr(ExprKind::Streaming { left }, t, n, args))
        }
        Kind::AssignmentPattern => pattern(e, sc, n, None),
        Kind::Cast => cast(e, sc, n, &kids),
        Kind::Member | Kind::Index | Kind::RangeSelect | Kind::Scope | Kind::Specialize => select(e, sc, n),
        Kind::Call => call(e, sc, n, &kids),
        Kind::New => new_expr(e, sc, n, None),
        Kind::TypeExpr => {
            let t = ty::data_type(e, sc, kids[0])?;
            Ok(e.new_expr(ExprKind::TypeRef(t), Types::TYPE, n, vec![]))
        }
        k if ty::is_type(k) => {
            let t = ty::data_type(e, sc, n)?;
            Ok(e.new_expr(ExprKind::TypeRef(t), Types::TYPE, n, vec![]))
        }
        Kind::Tagged => {
            let mut args = Vec::new();
            if let Some(&v) = kids.get(1) {
                args.push(expr(e, sc, v)?);
            }
            Ok(e.new_expr(ExprKind::Tagged(0), Types::UNKNOWN, n, args))
        }
        Kind::ValueRange => range_item(e, sc, n),
        Kind::RangeList => {
            let mut args = Vec::new();
            for &k in &kids {
                args.push(range_item(e, sc, k)?);
            }
            Ok(e.new_expr(ExprKind::Pattern, Types::UNKNOWN, n, args))
        }
        Kind::Empty => Ok(e.new_expr(ExprKind::Opaque, Types::UNKNOWN, n, vec![])),
        Kind::ScopeKeyword => Ok(e.new_expr(ExprKind::Opaque, Types::UNKNOWN, n, vec![])),
        _ => Ok(e.new_expr(ExprKind::Opaque, Types::UNKNOWN, n, vec![])),
    }
}

fn range_item<'a>(e: &mut Elab<'a, '_>, sc: ScopeId, r: NodeId) -> R<'a, ExprId> {
    if e.kind(r) == Kind::ValueRange {
        let rk = e.kids(r).to_vec();
        let a = expr(e, sc, rk[0])?;
        let b = expr(e, sc, *rk.last().unwrap())?;
        let t = e.ex(a).ty;
        return Ok(e.new_expr(ExprKind::Range, t, r, vec![a, b]));
    }
    expr(e, sc, r)
}

/// The bytes of a string literal, with its escapes read (5.9.1).
pub(crate) fn string_bytes(t: &str) -> Vec<u8> {
    let triple = t.starts_with("\"\"\"");
    let inner = if triple { &t[3..t.len().saturating_sub(3)] } else { &t[1..t.len().saturating_sub(1)] };
    let b = inner.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'\\' && i + 1 < b.len() {
            let c = b[i + 1];
            i += 2;
            match c {
                b'n' => out.push(b'\n'),
                b't' => out.push(b'\t'),
                b'\\' => out.push(b'\\'),
                b'"' => out.push(b'"'),
                b'v' => out.push(0x0b),
                b'f' => out.push(0x0c),
                b'a' => out.push(0x07),
                b'\n' => {}
                b'\r' => {
                    if b.get(i) == Some(&b'\n') {
                        i += 1;
                    }
                }
                b'x' => {
                    let mut v = 0u32;
                    let mut k = 0;
                    while k < 2 && i < b.len() && b[i].is_ascii_hexdigit() {
                        v = v * 16 + (b[i] as char).to_digit(16).unwrap();
                        i += 1;
                        k += 1;
                    }
                    out.push(v as u8);
                }
                b'0'..=b'7' => {
                    let mut v = (c - b'0') as u32;
                    let mut k = 1;
                    while k < 3 && i < b.len() && (b'0'..=b'7').contains(&b[i]) {
                        v = v * 8 + (b[i] - b'0') as u32;
                        i += 1;
                        k += 1;
                    }
                    out.push(v as u8);
                }
                other => out.push(other),
            }
        } else {
            out.push(b[i]);
            i += 1;
        }
    }
    out
}

fn number<'a>(e: &mut Elab<'a, '_>, n: NodeId) -> R<'a, ExprId> {
    let toks: Vec<&str> = e.ast.tokens_of(n).iter().map(|t| t.0).collect();
    let (size, base, digits) = match toks.len() {
        1 => (None, None, toks[0]),
        2 => (None, Some(toks[0]), toks[1]),
        _ => (Some(toks[0]), Some(toks[1]), toks[2]),
    };
    let Some(base) = base else {
        // A decimal number or a real.
        if digits.contains(['.', 'e', 'E']) {
            let v: f64 = digits.replace('_', "").parse().map_err(|_| e.err(n, "this real number cannot be read"))?;
            let c = e.new_const(Const::Real(v));
            return Ok(e.new_expr(ExprKind::Const(c), Types::REAL, n, vec![]));
        }
        let b = parse_digits(digits, 10, None, true).map_err(|m| e.err(n, m))?;
        // An unsized decimal is a 32-bit signed value; a larger one keeps
        // its width (5.7.1).
        let w = b.width().max(32);
        let b = b.resize(w, false).with_signed(true);
        let t = e.d.types.vector(w, true, false);
        let t = if w == 32 { Types::INT } else { t };
        let c = e.new_const(Const::Bits(b));
        return Ok(e.new_expr(ExprKind::Const(c), t, n, vec![]));
    };
    let signed = base.contains(['s', 'S']);
    let radix = match base.chars().last().unwrap().to_ascii_lowercase() {
        'b' => 2,
        'o' => 8,
        'h' => 16,
        _ => 10,
    };
    let width = match size {
        Some(s) => {
            let w: u64 = s.replace('_', "").parse().map_err(|_| e.err(n, "a literal's size must be a number"))?;
            if w == 0 || w > 16_777_215 {
                return Err(e.err(n, "a literal's size must be between 1 and 2^24 - 1 (5.7.1)"));
            }
            Some(w as u32)
        }
        None => None,
    };
    let b = parse_digits(digits, radix, width, signed).map_err(|m| e.err(n, m))?;
    let four = !b.is_known();
    let w = b.width();
    let t = e.d.types.vector(w, signed, four || size.is_none() || true);
    let c = e.new_const(Const::Bits(b));
    Ok(e.new_expr(ExprKind::Const(c), t, n, vec![]))
}

fn name<'a>(e: &mut Elab<'a, '_>, sc: ScopeId, n: NodeId) -> R<'a, ExprId> {
    let s = e.sym_of(n);
    let b = lookup::resolve(e, sc, s, Some(n))?;
    match b {
        Some(Binding::Object(o)) => object_ref(e, sc, o, n),
        Some(Binding::Subroutine(sub)) => {
            // A function called without parentheses.
            call_sub(e, sc, sub, n, &[], None)
        }
        Some(Binding::Type(t)) => Ok(e.new_expr(ExprKind::TypeRef(t), Types::TYPE, n, vec![])),
        Some(Binding::Class(c)) => {
            let t = e.d.types.intern(Type::Class(c));
            Ok(e.new_expr(ExprKind::TypeRef(t), Types::TYPE, n, vec![]))
        }
        Some(Binding::Instance(i)) if e.d.specs.get(e.d.instances[i.0 as usize].spec.0 as usize).is_some_and(|sp| sp.kind == DefKind::Interface) => {
            // An interface instance used as a value is a virtual
            // interface (25.9).
            let spec = e.d.instances[i.0 as usize].spec;
            let def = e.d.specs[spec.0 as usize].name;
            let t = e.d.types.intern(Type::Interface { spec: Some(spec), def, modport: None, virtual_: true });
            Ok(e.new_expr(ExprKind::Hier { path: vec![s], object: None }, t, n, vec![]))
        }
        Some(Binding::Instance(_) | Binding::Instances(..) | Binding::Scope(_)) => {
            Err(e.err(n, format!("`{}` is {}, not a value", e.name(s), e.describe_binding(b.unwrap()))))
        }
        Some(Binding::Other(_)) => Ok(e.new_expr(ExprKind::Opaque, Types::UNKNOWN, n, vec![])),
        None => {
            // An upward reference to a design element's instance (23.8),
            // or a top-level module.
            if let Some(x) = hier(e, sc, n)? {
                return Ok(x);
            }
            Err(e.err(n, format!("`{}` is not declared", e.name(s))))
        }
    }
}

pub(crate) fn object_ref<'a>(e: &mut Elab<'a, '_>, sc: ScopeId, o: ObjId, n: NodeId) -> R<'a, ExprId> {
    let obj = e.obj(o);
    let t = obj.ty;
    if obj.kind == ObjKind::Genvar && obj.value.is_none() {
        return Err(e.err(n, format!("genvar `{}` can only be used in a generate loop (27.4)", e.name(obj.name))));
    }
    super::checks::object_use(e, sc, o, n)?;
    Ok(e.new_expr(ExprKind::Object(o), t, n, vec![]))
}

fn binary<'a>(e: &mut Elab<'a, '_>, sc: ScopeId, n: NodeId, kids: &[NodeId]) -> R<'a, ExprId> {
    let op_node = kids[1];
    let rhs_node = *kids.last().unwrap();
    let ops = e.text(op_node);
    let Some(op) = binop(ops) else {
        return Ok(e.new_expr(ExprKind::Opaque, Types::UNKNOWN, n, vec![]));
    };
    use BinOp::*;
    match op {
        LogAnd | LogOr | Implies | Equiv => {
            let a = expr(e, sc, kids[0])?;
            let b = expr(e, sc, rhs_node)?;
            let four = e.d.types.is_four_state(e.ex(a).ty) || e.d.types.is_four_state(e.ex(b).ty);
            let t = bool_type(e, four);
            Ok(e.new_expr(ExprKind::Binary(op), t, n, vec![a, b]))
        }
        Eq | Ne | CaseEq | CaseNe | WildEq | WildNe | Lt | Le | Gt | Ge => {
            let a = bind(e, sc, kids[0])?;
            let b = bind(e, sc, rhs_node)?;
            let (ta, tb) = (e.ex(a).ty, e.ex(b).ty);
            super::checks::comparable(e, op, ta, tb, n)?;
            let (a, b, four) = if e.d.types.is_integral(ta) && e.d.types.is_integral(tb) {
                let w = e.d.types.width(ta).max(e.d.types.width(tb));
                let s = e.d.types.is_signed(ta) && e.d.types.is_signed(tb);
                let f = e.d.types.is_four_state(ta) || e.d.types.is_four_state(tb);
                let t = e.d.types.vector(w, s, f);
                (ctx(e, a, t), ctx(e, b, t), f)
            } else if e.d.types.is_real(ta) || e.d.types.is_real(tb) {
                let a = selfdet(e, a);
                let b = selfdet(e, b);
                (assign_conv(e, a, Types::REAL), assign_conv(e, b, Types::REAL), false)
            } else {
                // Strings, handles, unpacked arrays: compared as they are.
                let a = selfdet(e, a);
                let b = selfdet(e, b);
                let (ta, tb) = (e.ex(a).ty, e.ex(b).ty);
                // A string literal compared with a string is a string.
                let a = if tb == Types::STRING && e.d.types.is_integral(ta) { assign_conv(e, a, Types::STRING) } else { a };
                let b = if ta == Types::STRING && e.d.types.is_integral(tb) { assign_conv(e, b, Types::STRING) } else { b };
                (a, b, false)
            };
            let t = bool_type(e, four && !matches!(op, CaseEq | CaseNe));
            Ok(e.new_expr(ExprKind::Binary(op), t, n, vec![a, b]))
        }
        Shl | Shr | AShl | AShr | Pow => {
            let a = bind(e, sc, kids[0])?;
            let b = expr(e, sc, rhs_node)?;
            let ta = e.ex(a).ty;
            let tb = e.ex(b).ty;
            let t = if op == Pow && (e.d.types.is_real(ta) || e.d.types.is_real(tb)) {
                Types::REAL
            } else if op == Pow && e.d.types.is_integral(ta) {
                vector_of(e, ta)
            } else {
                super::checks::integral_operand(e, ta, kids[0], ops)?;
                vector_of(e, ta)
            };
            let (a, b) = if t == Types::REAL {
                let a = selfdet(e, a);
                (assign_conv(e, a, Types::REAL), assign_conv(e, b, Types::REAL))
            } else {
                (a, b)
            };
            Ok(e.new_expr(ExprKind::Binary(op), t, n, vec![a, b]))
        }
        _ => {
            let a = bind(e, sc, kids[0])?;
            let b = bind(e, sc, rhs_node)?;
            let (ta, tb) = (e.ex(a).ty, e.ex(b).ty);
            if matches!(op, And | Or | Xor | Xnor | Mod) {
                if e.d.types.is_real(ta) {
                    return Err(e.err(kids[0], format!("`{ops}` cannot take a real operand (11.4.1)")));
                }
                if e.d.types.is_real(tb) {
                    return Err(e.err(rhs_node, format!("`{ops}` cannot take a real operand (11.4.1)")));
                }
            }
            super::checks::arith_operand(e, ta, kids[0], ops)?;
            super::checks::arith_operand(e, tb, rhs_node, ops)?;
            let t = arith_type(e, ta, tb);
            let (a, b) = if t == Types::REAL {
                let a = selfdet(e, a);
                let b = selfdet(e, b);
                (assign_conv(e, a, Types::REAL), assign_conv(e, b, Types::REAL))
            } else {
                (a, b)
            };
            Ok(e.new_expr(ExprKind::Binary(op), t, n, vec![a, b]))
        }
    }
}

fn unary<'a>(e: &mut Elab<'a, '_>, sc: ScopeId, n: NodeId, kids: &[NodeId]) -> R<'a, ExprId> {
    let op = e.text(kids[0]);
    let operand = *kids.last().unwrap();
    let u = match op {
        "+" => UnOp::Plus,
        "-" => UnOp::Neg,
        "~" => UnOp::Not,
        "!" => UnOp::LogNot,
        "&" => UnOp::RedAnd,
        "~&" => UnOp::RedNand,
        "|" => UnOp::RedOr,
        "~|" => UnOp::RedNor,
        "^" => UnOp::RedXor,
        "~^" | "^~" => UnOp::RedXnor,
        _ => return Ok(e.new_expr(ExprKind::Opaque, Types::UNKNOWN, n, vec![])),
    };
    match u {
        UnOp::Plus | UnOp::Neg | UnOp::Not => {
            let a = bind(e, sc, operand)?;
            let ta = e.ex(a).ty;
            if u == UnOp::Not && e.d.types.is_real(ta) {
                return Err(e.err(operand, "`~` cannot take a real operand (11.4.1)"));
            }
            let t = if e.d.types.is_real(ta) { ta } else { vector_of(e, ta) };
            Ok(e.new_expr(ExprKind::Unary(u), t, n, vec![a]))
        }
        _ => {
            let a = expr(e, sc, operand)?;
            let ta = e.ex(a).ty;
            if u != UnOp::LogNot && e.d.types.is_real(ta) {
                return Err(e.err(operand, format!("`{op}` cannot take a real operand (11.4.1)")));
            }
            let four = e.d.types.is_four_state(ta);
            let t = bool_type(e, four);
            Ok(e.new_expr(ExprKind::Unary(u), t, n, vec![a]))
        }
    }
}

fn concat<'a>(e: &mut Elab<'a, '_>, sc: ScopeId, n: NodeId, kids: &[NodeId]) -> R<'a, ExprId> {
    let mut args = Vec::new();
    let mut w = 0u32;
    let mut four = false;
    let mut strings = false;
    let mut unknown = false;
    for &k in kids {
        let x = expr(e, sc, k)?;
        let t = e.ex(x).ty;
        if t == Types::STRING {
            strings = true;
        } else if t == Types::UNKNOWN {
            unknown = true;
        } else if !e.d.types.is_integral(t) {
            if e.d.types.is_unpacked_array(t) {
                unknown = true;
            } else {
                return Err(e.err(k, "a concatenation's operands must be integral (11.4.12)"));
            }
        } else {
            super::checks::sized_in_concat(e, k)?;
        }
        w += e.d.types.width(t);
        four |= e.d.types.is_four_state(t);
        args.push(x);
    }
    let t = if strings {
        Types::STRING
    } else if unknown || w == 0 {
        Types::UNKNOWN
    } else {
        e.d.types.vector(w, false, four)
    };
    Ok(e.new_expr(ExprKind::Concat, t, n, args))
}

/// `{a, b}` assigned to an unpacked array (10.10).
fn unpacked_concat<'a>(e: &mut Elab<'a, '_>, sc: ScopeId, n: NodeId, target: TypeId) -> R<'a, ExprId> {
    let elem = e.d.types.element(target).unwrap_or(Types::UNKNOWN);
    let mut args = Vec::new();
    for &k in e.kids(n).to_vec().iter() {
        if e.kind(k) == Kind::Concat && e.d.types.is_unpacked_array(elem) {
            args.push(unpacked_concat(e, sc, k, elem)?);
            continue;
        }
        if e.kind(k) == Kind::Concat && e.kids(k).iter().all(|&i| e.kind(i) != Kind::Concat) && e.d.types.is_integral(elem) {
            // `{1, 2}` inside: its items are elements too (10.10).
            let inner = unpacked_concat(e, sc, k, target)?;
            args.extend(e.ex(inner).args.clone());
            continue;
        }
        let x = bind(e, sc, k)?;
        let t = e.ex(x).ty;
        if t == target || e.d.types.is_unpacked_array(t) {
            args.push(selfdet(e, x));
        } else {
            let x = context_to(e, x, elem);
            args.push(assign_conv(e, x, elem));
        }
    }
    Ok(e.new_expr(ExprKind::Concat, target, n, args))
}

/// `'{...}`: elements in order, for the target's type (10.9).
pub(crate) fn pattern<'a>(e: &mut Elab<'a, '_>, sc: ScopeId, n: NodeId, target: Option<TypeId>) -> R<'a, ExprId> {
    let kids = e.kids(n).to_vec();
    // A type written before `'{` gives the pattern its type.
    let open = e.ast.tokens_of(n).iter().position(|t| t.0 == "'{").map_or(0, |p| p as u32 + e.ast.node(n).first);
    let typed = kids.first().is_some_and(|&t| e.ast.node(t).first < open);
    let (target, items) = match kids.first() {
        Some(&t) if typed => {
            let tt = match ty::maybe_type(e, sc, t)? {
                Some(tt) => tt,
                None => return Err(e.err(t, "expected a type before `'{`")),
            };
            (Some(tt), &kids[1..])
        }
        _ => (target, &kids[..]),
    };
    let t = target.unwrap_or(Types::UNKNOWN);
    let mut args = Vec::new();
    // The type each element takes, where the target says.
    let members: Vec<TypeId> = match e.d.types.get(t).clone() {
        Type::Struct { members, .. } => members.iter().map(|m| m.ty).collect(),
        _ => vec![],
    };
    let elem = e.d.types.element(t);
    for (i, &k) in items.iter().enumerate() {
        match e.kind(k) {
            Kind::PatternKey => {
                let pk = e.kids(k).to_vec();
                let v = bind(e, sc, pk[1])?;
                let v = selfdet(e, v);
                args.push(v);
            }
            Kind::Replicate => {
                // `'{n{a, b}}`: n copies of the items (10.9).
                let rk = e.kids(k).to_vec();
                let count = super::ty::const_int(e, sc, rk[0])?;
                let items: Vec<NodeId> = e.kids(rk[1]).to_vec();
                let et = elem.unwrap_or(Types::UNKNOWN);
                let mut one = Vec::new();
                for &it in &items {
                    one.push(if et == Types::UNKNOWN { expr(e, sc, it)? } else { rvalue(e, sc, it, et)? });
                }
                for _ in 0..count.max(0) {
                    args.extend(one.iter().copied());
                }
            }
            _ => {
                let et = members.get(i).copied().or(elem).unwrap_or(Types::UNKNOWN);
                let x = if et == Types::UNKNOWN { expr(e, sc, k)? } else { rvalue(e, sc, k, et)? };
                args.push(x);
            }
        }
    }
    super::checks::pattern_shape(e, sc, n, t, items)?;
    Ok(e.new_expr(ExprKind::Pattern, t, n, args))
}

fn cast<'a>(e: &mut Elab<'a, '_>, sc: ScopeId, n: NodeId, kids: &[NodeId]) -> R<'a, ExprId> {
    let (target, value) = (kids[0], kids[1]);
    let x = bind(e, sc, value)?;
    let xt = e.ex(x).ty;
    let t = if e.kind(target) == Kind::Keyword {
        match e.text(target) {
            "signed" | "unsigned" => {
                let s = e.text(target) == "signed";
                let w = e.d.types.width(xt).max(1);
                let f = e.d.types.is_four_state(xt);
                e.d.types.vector(w, s, f)
            }
            "const" => xt,
            "string" => Types::STRING,
            _ => ty::maybe_type(e, sc, target)?.unwrap_or(Types::UNKNOWN),
        }
    } else if let Some(t) = ty::maybe_type(e, sc, target)? {
        t
    } else {
        // A size: `8'(x)`.
        let w = ty::const_int(e, sc, target)?;
        if w <= 0 {
            return Err(e.err(target, "a size cast needs a positive width (6.24.1)"));
        }
        let s = e.d.types.is_signed(xt);
        let f = e.d.types.is_four_state(xt);
        e.d.types.vector(w as u32, s, f)
    };
    super::checks::cast_ok(e, t, xt, n)?;
    // The operand is evaluated at the cast's width, as if assigned (6.24.1).
    let x = if e.d.types.is_integral(t) { context_to(e, x, t) } else { selfdet(e, x) };
    if e.d.types.is_integral(t) && is_unsized_fill(e, x) {
        return Ok(fill_literal(e, x, t));
    }
    Ok(e.new_expr(ExprKind::Convert, t, n, vec![x]))
}

// ---- selects, members, hierarchy ----

fn select<'a>(e: &mut Elab<'a, '_>, sc: ScopeId, n: NodeId) -> R<'a, ExprId> {
    // A dotted path through the hierarchy, or `p::x`.
    if let Some(x) = hier(e, sc, n)? {
        return Ok(x);
    }
    let kids = e.kids(n).to_vec();
    match e.kind(n) {
        Kind::Scope => scoped(e, sc, n, &kids),
        Kind::Specialize => {
            let x = bind(e, sc, kids[0])?;
            Ok(x)
        }
        Kind::Member => {
            let base = bind(e, sc, kids[0])?;
            let base = selfdet(e, base);
            let m = e.sym_of(kids[1]);
            member(e, sc, base, m, n, kids[1])
        }
        Kind::Index => {
            let base = bind(e, sc, kids[0])?;
            let base = selfdet(e, base);
            let bt = e.ex(base).ty;
            let idx = expr(e, sc, kids[1])?;
            let et = match e.d.types.get(bt).clone() {
                Type::Unpacked { elem, dim } => {
                    if let UDim::Assoc(it) = dim {
                        super::checks::assoc_key(e, it, idx, kids[1])?;
                    } else if let UDim::Wildcard = dim {
                        super::checks::wildcard_key(e, idx, kids[1])?;
                    }
                    elem
                }
                Type::Unknown => Types::UNKNOWN,
                _ => match e.d.types.element(bt) {
                    Some(t) => t,
                    None => {
                        if e.d.types.is_real(bt) {
                            return Err(e.err(kids[1], "a real cannot be indexed (11.5.1)"));
                        }
                        if matches!(e.d.types.get(bt), Type::Class(_) | Type::Interface { .. } | Type::Event) {
                            return Err(e.err(kids[1], "this value cannot be indexed"));
                        }
                        Types::UNKNOWN
                    }
                },
            };
            if e.d.types.is_real(e.ex(idx).ty) && !matches!(e.d.types.get(bt), Type::Unpacked { dim: UDim::Assoc(_), .. }) {
                return Err(e.err(kids[1], "an index cannot be real (11.5.1)"));
            }
            Ok(e.new_expr(ExprKind::Index, et, n, vec![base, idx]))
        }
        Kind::RangeSelect => {
            let base = bind(e, sc, kids[0])?;
            let base = selfdet(e, base);
            let bt = e.ex(base).ty;
            let op = e.text(kids[2]);
            let kind = match op {
                "+:" => PartKind::Up,
                "-:" => PartKind::Down,
                _ => PartKind::Range,
            };
            let a = expr(e, sc, kids[1])?;
            let (width, args) = if kind == PartKind::Range {
                let l = eval::const_expr(e, sc, kids[1]);
                let r = eval::const_expr(e, sc, kids[3]);
                let b = expr(e, sc, kids[3])?;
                match (l.ok().and_then(|c| c.to_i64()), r.ok().and_then(|c| c.to_i64())) {
                    (Some(l), Some(r)) => {
                        super::checks::part_order(e, bt, l, r, n)?;
                        ((l - r).unsigned_abs() as u32 + 1, vec![base, a, b])
                    }
                    _ => {
                        if e.d.types.is_unpacked_array(bt)
                            && matches!(e.d.types.get(bt), Type::Unpacked { dim: UDim::Queue(_), .. })
                        {
                            (0, vec![base, a, b])
                        } else {
                            return Err(e.err(n, "a part select's bounds must be constant; use `+:` or `-:` (11.5.1)"));
                        }
                    }
                }
            } else {
                let w = ty::const_int(e, sc, kids[3])?;
                if w <= 0 {
                    return Err(e.err(kids[3], "a part select's width must be positive (11.5.1)"));
                }
                (w as u32, vec![base, a])
            };
            let t = match e.d.types.get(bt).clone() {
                Type::Unpacked { elem, dim } => {
                    let d = if matches!(dim, UDim::Queue(_)) { UDim::Queue(None) } else { UDim::Range(width as i64 - 1, 0) };
                    let _ = width;
                    e.d.types.intern(Type::Unpacked { elem, dim: d })
                }
                Type::Unknown => Types::UNKNOWN,
                Type::String => Types::UNKNOWN,
                _ => {
                    let f = e.d.types.is_four_state(bt);
                    // The element width: one bit, or a packed element.
                    let ew = e.d.types.element(bt).map_or(1, |el| e.d.types.width(el).max(1));
                    e.d.types.vector(width * ew, false, f)
                }
            };
            Ok(e.new_expr(ExprKind::Part { kind, width }, t, n, args))
        }
        _ => unreachable!(),
    }
}

/// `x.m` where `x` is a value: a struct member, a class property or
/// method, or a built-in method called without parentheses.
fn member<'a>(e: &mut Elab<'a, '_>, sc: ScopeId, base: ExprId, m: Sym, n: NodeId, mn: NodeId) -> R<'a, ExprId> {
    let bt = e.ex(base).ty;
    match e.d.types.get(bt).clone() {
        Type::Struct { members, .. } => match members.iter().position(|x| x.name == m) {
            Some(i) => {
                let t = members[i].ty;
                Ok(e.new_expr(ExprKind::Member(i as u32), t, n, vec![base]))
            }
            None => Err(e.err(mn, format!("the struct has no member `{}`", e.name(m)))),
        },
        Type::PackedArray { elem, .. } if matches!(e.d.types.get(elem), Type::Struct { .. }) => {
            Ok(e.new_expr(ExprKind::Opaque, Types::UNKNOWN, n, vec![base]))
        }
        Type::Class(c) => match lookup::class_member(e, c, m) {
            Some(Binding::Object(o)) => {
                super::checks::member_access(e, sc, c, o, mn)?;
                let t = e.obj(o).ty;
                Ok(e.new_expr(ExprKind::Field(o), t, n, vec![base]))
            }
            Some(Binding::Subroutine(s)) => call_sub(e, sc, s, n, &[], Some(base)),
            Some(_) => Ok(e.new_expr(ExprKind::Opaque, Types::UNKNOWN, n, vec![base])),
            None => {
                if e.d.classes[c.0 as usize].base.is_none() && !class_complete(e, c) {
                    return Ok(e.new_expr(ExprKind::Opaque, Types::UNKNOWN, n, vec![base]));
                }
                // Built-in class methods (`randomize`, `srandom`, ...).
                if matches!(e.name(m), "randomize" | "srandom" | "get_randstate" | "set_randstate" | "rand_mode" | "constraint_mode") {
                    return Ok(e.new_expr(ExprKind::Method(m), Types::INT, n, vec![base]));
                }
                Err(e.err(mn, format!("class `{}` has no member `{}`", e.name(e.d.classes[c.0 as usize].name), e.name(m))))
            }
        },
        Type::Interface { def, .. } => {
            let _ = def;
            Ok(e.new_expr(ExprKind::Opaque, Types::UNKNOWN, n, vec![base]))
        }
        _ => {
            let t = builtin_method_type(e, bt, m, &[]);
            Ok(e.new_expr(ExprKind::Method(m), t, n, vec![base]))
        }
    }
}

fn class_complete(e: &Elab, c: ClassId) -> bool {
    // A class whose base could not be resolved may have members we do not
    // know about.
    let node = e.d.classes[c.0 as usize].node;
    e.child(node, Kind::Extends).is_none()
}

/// `p::x`, `C::x`, `$unit::x`.
fn scoped<'a>(e: &mut Elab<'a, '_>, sc: ScopeId, n: NodeId, kids: &[NodeId]) -> R<'a, ExprId> {
    let s = e.sym_of(kids[1]);
    let target = ty::scope_target(e, sc, kids[0])?;
    let b = match target {
        ty::ScopeTarget::Package(p) => {
            let b = lookup::in_package(e, p, s);
            if b.is_none() {
                return Err(e.err(kids[1], format!("package `{}` has no `{}`", e.name(e.scope(p).name.unwrap()), e.name(s))));
            }
            super::checks::package_ref_order(e, sc, p, s, n)?;
            b
        }
        ty::ScopeTarget::Class(c) => {
            super::checks::class_scope_use(e, sc, c, kids[0])?;
            lookup::class_member(e, c, s)
        }
        ty::ScopeTarget::Unit => {
            let unit = e.d.unit;
            let b = e.scope(unit).names.get(&s).copied();
            super::checks::unit_ref(e, sc, s, b, n)?;
            b
        }
        ty::ScopeTarget::Unknown => None,
    };
    match b {
        Some(Binding::Object(o)) => {
            let t = e.obj(o).ty;
            Ok(e.new_expr(ExprKind::Object(o), t, n, vec![]))
        }
        Some(Binding::Subroutine(sub)) => call_sub(e, sc, sub, n, &[], None),
        Some(Binding::Type(t)) => Ok(e.new_expr(ExprKind::TypeRef(t), Types::TYPE, n, vec![])),
        Some(Binding::Class(c)) => {
            let t = e.d.types.intern(Type::Class(c));
            Ok(e.new_expr(ExprKind::TypeRef(t), Types::TYPE, n, vec![]))
        }
        _ => Ok(e.new_expr(ExprKind::Opaque, Types::UNKNOWN, n, vec![])),
    }
}

/// One step of a hierarchical name.
struct Step {
    name: Sym,
    node: NodeId,
    index: Option<NodeId>,
}

/// If `n` is a reference through the hierarchy (`u1.x`, `blk[2].y`,
/// `top.a`, `$root.top.a`), the reference; `None` if it is something
/// else, such as a struct member.
pub(crate) fn hier<'a>(e: &mut Elab<'a, '_>, sc: ScopeId, n: NodeId) -> R<'a, Option<ExprId>> {
    // Flatten the chain of members and indices.
    let mut steps: Vec<Step> = Vec::new();
    let mut cur = n;
    let mut trailing: Vec<NodeId> = Vec::new();
    loop {
        match e.kind(cur) {
            Kind::Member => {
                let k = e.kids(cur).to_vec();
                let s = e.sym_of(k[1]);
                steps.push(Step { name: s, node: k[1], index: None });
                cur = k[0];
            }
            Kind::Index => {
                let k = e.kids(cur).to_vec();
                trailing.push(k[1]);
                // An index applies to the step written before it.
                cur = k[0];
                if matches!(e.kind(cur), Kind::Name | Kind::Member) {
                    let idx = trailing.pop();
                    let (name, node) = if e.kind(cur) == Kind::Name {
                        (e.sym_of(cur), cur)
                    } else {
                        let k2 = e.kids(cur).to_vec();
                        (e.sym_of(k2[1]), k2[1])
                    };
                    steps.push(Step { name, node, index: idx });
                    if e.kind(cur) == Kind::Name {
                        break;
                    }
                    cur = e.kids(cur)[0];
                    continue;
                }
                return Ok(None);
            }
            Kind::Name => {
                let s = e.sym_of(cur);
                steps.push(Step { name: s, node: cur, index: None });
                break;
            }
            Kind::ScopeKeyword if e.text(cur) == "$root" => {
                let s = e.sym("$root");
                steps.push(Step { name: s, node: cur, index: None });
                break;
            }
            _ => return Ok(None),
        }
    }
    if !trailing.is_empty() {
        return Ok(None);
    }
    steps.reverse();
    // A name with no dots that is not declared: only an upward module name
    // could make it a reference, and that needs a dot.
    if steps.len() == 1 {
        return Ok(None);
    }
    let head = &steps[0];
    let mut at: Location;
    let mut rest = 1;
    let root = e.name(head.name) == "$root";
    if root {
        let Some(top) = steps.get(1) else { return Ok(None) };
        let t = e.d.tops.iter().copied().find(|&i| e.d.instances[i.0 as usize].name == top.name);
        let in_progress = e.inst_path.first().copied().filter(|&i| e.d.instances[i.0 as usize].name == top.name);
        match t.or(in_progress) {
            Some(i) => at = Location::Instance(i),
            None => {
                let unit = e.d.unit;
                if e.scope(unit).names.contains_key(&top.name) {
                    return Err(e.err(top.node, "compilation-unit items are not part of the `$root` hierarchy (3.12.1)"));
                }
                return Err(e.err(top.node, format!("there is no top-level instance `{}`", e.name(top.name))));
            }
        }
        e.upward = true;
        rest = 2;
    } else {
        match lookup::lookup(e, sc, head.name) {
            Some(Binding::Instance(i)) => at = Location::Instance(i),
            Some(Binding::Instances(first, count)) => {
                let Some(ix) = head.index else { return Ok(None) };
                at = Location::Instance(pick_instance(e, sc, first, count, ix)?);
            }
            Some(Binding::Scope(s)) => {
                at = Location::Scope(s);
                if let Some(ix) = head.index {
                    at = Location::Scope(pick_block(e, sc, s, ix)?);
                }
            }
            Some(Binding::Other(m)) if e.kind(m) == Kind::AnsiPort || e.kind(m) == Kind::InterfacePortDecl => {
                // An interface port: resolve in the interface.
                at = Location::InterfacePort(m);
            }
            Some(Binding::Object(o)) if matches!(e.d.types.get(e.obj(o).ty), Type::Interface { .. }) => {
                return Ok(Some(e.new_expr(ExprKind::Opaque, Types::UNKNOWN, n, vec![])));
            }
            Some(_) => return Ok(None),
            None => {
                // Upward: an instance or module name above (23.8).
                match upward(e, head.name) {
                    Some(i) => {
                        e.upward = true;
                        at = Location::Instance(i);
                    }
                    None => {
                        if e.d.tops.iter().any(|&i| e.d.instances[i.0 as usize].name == head.name) {
                            let i = e.d.tops.iter().copied().find(|&i| e.d.instances[i.0 as usize].name == head.name).unwrap();
                            e.upward = true;
                            at = Location::Instance(i);
                        } else if e.defs.contains_key(&head.name) {
                            // A design element not yet elaborated, named
                            // from below or before it: kept unresolved.
                            return Ok(Some(e.new_expr(
                                ExprKind::Hier { path: steps.iter().map(|s| s.name).collect(), object: None },
                                Types::UNKNOWN,
                                n,
                                vec![],
                            )));
                        } else {
                            return Ok(None);
                        }
                    }
                }
            }
        }
    }
    let path: Vec<Sym> = steps.iter().map(|s| s.name).collect();
    for i in rest..steps.len() {
        let step = &steps[i];
        let scope = match at {
            Location::Instance(inst) => {
                let spec = e.d.instances[inst.0 as usize].spec;
                if spec.0 == u32::MAX {
                    // Still being elaborated (an upward reference from
                    // inside it): find the scope by its node.
                    let node = e.d.instances[inst.0 as usize].node;
                    let _ = node;
                    return Ok(Some(e.new_expr(ExprKind::Hier { path, object: None }, Types::UNKNOWN, n, vec![])));
                }
                e.d.specs[spec.0 as usize].scope
            }
            Location::Scope(s) => s,
            Location::InterfacePort(_) => {
                return Ok(Some(e.new_expr(ExprKind::Hier { path, object: None }, Types::UNKNOWN, n, vec![])));
            }
        };
        let b = e.scope(scope).names.get(&step.name).copied();
        let imported = e.scope(scope).items.iter().any(|it| matches!(it, Item::Import(_, Some(x)) if *x == step.name))
            || e.imported_uses.contains_key(&(scope, step.name));
        if imported {
            return Err(e.err(
                step.node,
                format!("`{}` is imported, and imported names cannot be reached through the hierarchy (26.3)", e.name(step.name)),
            ));
        }
        match b {
            Some(Binding::Instance(inst)) => at = Location::Instance(inst),
            Some(Binding::Instances(first, count)) => {
                let Some(ix) = step.index else {
                    return Err(e.err(step.node, "an array of instances needs an index here"));
                };
                at = Location::Instance(pick_instance(e, sc, first, count, ix)?);
            }
            Some(Binding::Scope(s)) => {
                at = Location::Scope(s);
                if let Some(ix) = step.index {
                    at = Location::Scope(pick_block(e, sc, s, ix)?);
                }
            }
            Some(Binding::Object(o)) => {
                let t = e.obj(o).ty;
                let mut x = e.new_expr(ExprKind::Hier { path: path[..=i].to_vec(), object: Some(o) }, t, n, vec![]);
                if let Some(ix) = step.index {
                    let idx = expr(e, sc, ix)?;
                    let et = e.d.types.element(t).unwrap_or(Types::UNKNOWN);
                    x = e.new_expr(ExprKind::Index, et, n, vec![x, idx]);
                }
                // What follows is member access on the value.
                for s2 in &steps[i + 1..] {
                    x = member(e, sc, x, s2.name, n, s2.node)?;
                }
                return Ok(Some(x));
            }
            Some(Binding::Subroutine(sub)) => {
                let t = e.d.subroutines[sub.0 as usize].ret;
                return Ok(Some(e.new_expr(ExprKind::Hier { path, object: None }, t, n, vec![])));
            }
            Some(_) => {
                return Ok(Some(e.new_expr(ExprKind::Hier { path, object: None }, Types::UNKNOWN, n, vec![])));
            }
            None => {
                // `bind` adds instances to other scopes (23.11), which is not
                // modelled yet: the path is kept unresolved.
                if e.has_bind {
                    return Ok(Some(e.new_expr(ExprKind::Hier { path, object: None }, Types::UNKNOWN, n, vec![])));
                }
                return Err(e.err(step.node, format!("there is no `{}` there", e.name(step.name))));
            }
        }
    }
    // The path names a scope or instance, not a value.
    Ok(Some(e.new_expr(ExprKind::Hier { path, object: None }, Types::UNKNOWN, n, vec![])))
}

#[derive(Clone, Copy)]
enum Location {
    Instance(InstId),
    Scope(ScopeId),
    InterfacePort(NodeId),
}

/// A task or function declared in an instance above (23.8).
fn upward_sub(e: &Elab, s: Sym) -> Option<SubId> {
    for &i in e.inst_path.iter().rev().skip(1) {
        let spec = e.d.instances[i.0 as usize].spec;
        let Some(sp) = e.d.specs.get(spec.0 as usize) else { continue };
        if let Some(Binding::Subroutine(sub)) = e.scope(sp.scope).names.get(&s) {
            return Some(*sub);
        }
    }
    None
}

/// An instance above in the hierarchy, by its instance or module name
/// (23.8).
fn upward(e: &Elab, s: Sym) -> Option<InstId> {
    for &i in e.inst_path.iter().rev() {
        let inst = &e.d.instances[i.0 as usize];
        if inst.name == s {
            return Some(i);
        }
        let def = inst.node;
        let _ = def;
        if inst.spec.0 != u32::MAX && e.d.specs[inst.spec.0 as usize].name == s {
            return Some(i);
        }
    }
    None
}

fn pick_instance<'a>(e: &mut Elab<'a, '_>, sc: ScopeId, first: InstId, count: u32, ix: NodeId) -> R<'a, InstId> {
    let idx = ty::const_int(e, sc, ix)?;
    for k in 0..count {
        let id = InstId(first.0 + k);
        if e.d.instances[id.0 as usize].index == Some(idx) {
            return Ok(id);
        }
    }
    Err(e.err(ix, format!("there is no instance with index {idx}")))
}

fn pick_block<'a>(e: &mut Elab<'a, '_>, sc: ScopeId, s: ScopeId, ix: NodeId) -> R<'a, ScopeId> {
    let idx = ty::const_int(e, sc, ix)?;
    for item in e.scope(s).items.clone() {
        if let Item::Scope(c) = item
            && e.scope(c).index == Some(idx)
        {
            return Ok(c);
        }
    }
    Err(e.err(ix, format!("the generate loop has no block with index {idx}")))
}

// ---- calls ----

fn call<'a>(e: &mut Elab<'a, '_>, sc: ScopeId, n: NodeId, kids: &[NodeId]) -> R<'a, ExprId> {
    let callee = kids[0];
    let args: Vec<NodeId> = match kids.iter().copied().find(|&k| e.kind(k) == Kind::Args) {
        Some(a) => e.kids(a).to_vec(),
        None => vec![],
    };
    let with = kids.iter().copied().find(|&k| e.kind(k) == Kind::With);
    match e.kind(callee) {
        Kind::SystemName => syscall(e, sc, n, callee, &args),
        Kind::Name => {
            let s = e.sym_of(callee);
            match lookup::resolve(e, sc, s, Some(callee))? {
                Some(Binding::Subroutine(sub)) => call_sub(e, sc, sub, n, &args, None),
                // Inside a function its name is its result; called, it is
                // the function (13.4.1).
                Some(Binding::Object(o)) if e.obj(o).qualifiers.iter().any(|&q| e.name(q) == "result") => {
                    let osc = e.obj(o).scope;
                    let sub = e.d.subroutines.iter().position(|x| x.scope == osc).map(|i| SubId(i as u32));
                    match sub {
                        Some(sub) => call_sub(e, sc, sub, n, &args, None),
                        None => Err(e.err(callee, format!("`{}` is not a task or function", e.name(s)))),
                    }
                }
                Some(Binding::Other(_)) => {
                    let mut xs = Vec::new();
                    for &a in &args {
                        xs.push(arg_expr(e, sc, a)?);
                    }
                    Ok(e.new_expr(ExprKind::Opaque, Types::UNKNOWN, n, xs))
                }
                Some(b) => Err(e.err(callee, format!("`{}` is {}, not a task or function", e.name(s), e.describe_binding(b)))),
                None => {
                    if e.name(s) == "randomize" {
                        return Ok(e.new_expr(ExprKind::Method(s), Types::INT, n, vec![]));
                    }
                    // A task or function in an enclosing instance (23.8).
                    if let Some(sub) = upward_sub(e, s) {
                        e.upward = true;
                        return call_sub(e, sc, sub, n, &args, None);
                    }
                    Err(e.err(callee, format!("there is no task or function `{}`", e.name(s))))
                }
            }
        }
        Kind::Member => {
            // `obj.m(args)` or a hierarchical call `u1.f(args)`.
            if let Some(h) = hier(e, sc, callee)? {
                let mut xs = vec![];
                for &a in &args {
                    xs.push(arg_expr(e, sc, a)?);
                }
                let t = e.ex(h).ty;
                let mut all = vec![h];
                all.extend(xs);
                return Ok(e.new_expr(ExprKind::Opaque, t, n, all));
            }
            let ck = e.kids(callee).to_vec();
            let base = bind(e, sc, ck[0])?;
            let base = selfdet(e, base);
            let m = e.sym_of(ck[1]);
            let bt = e.ex(base).ty;
            // `obj.randomize(a, b) with {...}`: the arguments name the
            // object's members (18.11), which are kept as syntax.
            if e.name(m) == "randomize" {
                return Ok(e.new_expr(ExprKind::Method(m), Types::INT, n, vec![base]));
            }
            if let Type::Class(c) = e.d.types.get(bt).clone() {
                match lookup::class_member(e, c, m) {
                    Some(Binding::Subroutine(sub)) => return call_sub(e, sc, sub, n, &args, Some(base)),
                    Some(Binding::Object(_)) => {
                        return Err(e.err(ck[1], format!("`{}` is a property, not a method", e.name(m))));
                    }
                    _ => {}
                }
            }
            let mut xs = vec![base];
            for &a in &args {
                xs.push(arg_expr(e, sc, a)?);
            }
            super::checks::builtin_method(e, bt, m, &xs[1..], with, ck[1])?;
            let t = builtin_method_type(e, bt, m, &xs[1..]);
            Ok(e.new_expr(ExprKind::Method(m), t, n, xs))
        }
        Kind::Scope => {
            let ck = e.kids(callee).to_vec();
            let s = e.sym_of(ck[1]);
            let target = ty::scope_target(e, sc, ck[0])?;
            let b = match target {
                ty::ScopeTarget::Package(p) => lookup::in_package(e, p, s),
                ty::ScopeTarget::Class(c) => {
                    super::checks::class_scope_use(e, sc, c, ck[0])?;
                    lookup::class_member(e, c, s)
                }
                ty::ScopeTarget::Unit => e.scope(e.d.unit).names.get(&s).copied(),
                ty::ScopeTarget::Unknown => None,
            };
            match b {
                Some(Binding::Subroutine(sub)) => call_sub(e, sc, sub, n, &args, None),
                _ => {
                    let mut xs = vec![];
                    for &a in &args {
                        xs.push(arg_expr(e, sc, a)?);
                    }
                    Ok(e.new_expr(ExprKind::Opaque, Types::UNKNOWN, n, xs))
                }
            }
        }
        _ => {
            let mut xs = vec![];
            for &a in &args {
                xs.push(arg_expr(e, sc, a)?);
            }
            Ok(e.new_expr(ExprKind::Opaque, Types::UNKNOWN, n, xs))
        }
    }
}

fn arg_expr<'a>(e: &mut Elab<'a, '_>, sc: ScopeId, a: NodeId) -> R<'a, ExprId> {
    match e.kind(a) {
        Kind::NamedArg => match e.kids(a).get(1) {
            Some(&v) => expr(e, sc, v),
            None => Ok(e.new_expr(ExprKind::Opaque, Types::UNKNOWN, a, vec![])),
        },
        Kind::Empty => Ok(e.new_expr(ExprKind::Opaque, Types::UNKNOWN, a, vec![])),
        k if ty::is_type(k) || k == Kind::TypeExpr => {
            let t = ty::maybe_type(e, sc, a)?.unwrap_or(Types::UNKNOWN);
            Ok(e.new_expr(ExprKind::TypeRef(t), Types::TYPE, a, vec![]))
        }
        _ => {
            if let Some(t) = ty::maybe_type(e, sc, a)? {
                return Ok(e.new_expr(ExprKind::TypeRef(t), Types::TYPE, a, vec![]));
            }
            expr(e, sc, a)
        }
    }
}

/// A call of a user subroutine: arguments matched to the formals by
/// position or name, defaults filled in (13.5).
pub(crate) fn call_sub<'a>(
    e: &mut Elab<'a, '_>,
    sc: ScopeId,
    sub: SubId,
    n: NodeId,
    args: &[NodeId],
    this: Option<ExprId>,
) -> R<'a, ExprId> {
    super::stmt::sub_signature(e, sub)?;
    let formals = e.d.subroutines[sub.0 as usize].args.clone();
    let defaults = e.d.subroutines[sub.0 as usize].defaults.clone();
    let name = e.d.subroutines[sub.0 as usize].name;
    let mut given: Vec<Option<NodeId>> = vec![None; formals.len()];
    let mut named = false;
    let mut pos = 0;
    for &a in args {
        if e.kind(a) == Kind::NamedArg {
            named = true;
            let k = e.kids(a).to_vec();
            let s = e.sym_of(k[0]);
            let Some(i) = formals.iter().position(|&f| e.obj(f).name == s) else {
                return Err(e.err(k[0], format!("`{}` has no argument `{}`", e.name(name), e.name(s))));
            };
            if given[i].is_some() {
                return Err(e.err(k[0], format!("argument `{}` is given twice", e.name(s))));
            }
            given[i] = Some(k.get(1).copied().unwrap_or(k[0]));
        } else {
            if named {
                return Err(e.err(a, "a positional argument cannot follow a named one (13.5.4)"));
            }
            if pos >= formals.len() {
                if formals.is_empty() && args.len() == 1 && e.kind(a) == Kind::Empty {
                    break;
                }
                return Err(e.err(a, format!("`{}` takes {} arguments, but more are given", e.name(name), formals.len())));
            }
            if e.kind(a) != Kind::Empty {
                given[pos] = Some(a);
            }
            pos += 1;
        }
    }
    let mut xs = Vec::new();
    if let Some(t) = this {
        xs.push(t);
    }
    for (i, &f) in formals.iter().enumerate() {
        let ft = e.obj(f).ty;
        let dir = e.obj(f).dir.unwrap_or(Dir::Input);
        let x = match given[i] {
            Some(a) => match dir {
                Dir::Input | Dir::ConstRef => rvalue(e, sc, a, ft)?,
                _ => {
                    let x = expr(e, sc, a)?;
                    super::checks::output_arg(e, sc, sub, f, x, a)?;
                    x
                }
            },
            None => match defaults.get(i).copied().flatten() {
                Some(d) => {
                    let fsc = e.d.subroutines[sub.0 as usize].scope;
                    rvalue(e, fsc, d, ft)?
                }
                None => {
                    return Err(e.err(
                        n,
                        format!("argument `{}` of `{}` has no default, so it must be given (13.5.3)", e.name(e.obj(f).name), e.name(name)),
                    ));
                }
            },
        };
        xs.push(x);
    }
    super::checks::call_allowed(e, sc, sub, n)?;
    if let Some(cur) = e.current_sub {
        e.calls.entry(cur).or_default().push(sub);
    }
    let t = e.d.subroutines[sub.0 as usize].ret;
    Ok(e.new_expr(ExprKind::Call(sub), t, n, xs))
}

/// A system task or function (20, 21) and the type it returns.
fn syscall<'a>(e: &mut Elab<'a, '_>, sc: ScopeId, n: NodeId, name_node: NodeId, args: &[NodeId]) -> R<'a, ExprId> {
    let name = e.text(name_node);
    let s = e.sym_of(name_node);
    let mut xs = Vec::new();
    for &a in args {
        // Some tasks take a scope: `$dumpvars(0, top)`, `$printtimescale(u)`.
        if let Some(x) = scope_arg(e, sc, a)? {
            xs.push(x);
            continue;
        }
        xs.push(arg_expr(e, sc, a)?);
    }
    super::checks::syscall(e, sc, name, &xs, args, n)?;
    let arg_ty = |e: &Elab, i: usize| xs.get(i).map(|&x| e.ex(x).ty);
    let t = match name {
        "$bits" | "$size" | "$left" | "$right" | "$low" | "$high" | "$increment" | "$dimensions"
        | "$unpacked_dimensions" | "$clog2" | "$countones" | "$countbits" | "$cast" | "$fopen" | "$feof" | "$fgetc"
        | "$fgets" | "$fscanf" | "$sscanf" | "$fread" | "$ftell" | "$fseek" | "$rewind" | "$ungetc" | "$ferror"
        | "$test$plusargs" | "$value$plusargs" | "$random" | "$dist_uniform" | "$dist_normal" | "$dist_exponential"
        | "$dist_poisson" | "$dist_chi_square" | "$dist_t" | "$dist_erlang" | "$system" | "$get_coverage"
        | "$coverage_control" | "$coverage_get_max" | "$coverage_get" | "$coverage_merge" | "$coverage_save"
        | "$q_full" | "$q_exam" | "$async$and$array" | "$stacktrace" => Types::INT,
        "$urandom" | "$urandom_range" | "$stime" => e.d.types.vector(32, false, false),
        "$onehot" | "$onehot0" | "$isunknown" | "$rose" | "$fell" | "$stable" | "$changed" | "$sampled_bool" => Types::BIT,
        "$time" => Types::TIME,
        "$realtime" => Types::REALTIME,
        "$itor" | "$bitstoreal" | "$sqrt" | "$ln" | "$log10" | "$exp" | "$pow" | "$floor" | "$ceil" | "$sin" | "$cos"
        | "$tan" | "$asin" | "$acos" | "$atan" | "$atan2" | "$hypot" | "$sinh" | "$cosh" | "$tanh" | "$asinh"
        | "$acosh" | "$atanh" | "$get_coverage_real" => Types::REAL,
        "$shortrealtobits" => e.d.types.vector(32, false, false),
        "$bitstoshortreal" => Types::SHORTREAL,
        "$rtoi" => Types::INT,
        "$realtobits" => e.d.types.vector(64, false, false),
        "$signed" | "$unsigned" => match arg_ty(e, 0) {
            Some(t) if e.d.types.is_integral(t) => {
                let w = e.d.types.width(t);
                let f = e.d.types.is_four_state(t);
                e.d.types.vector(w, name == "$signed", f)
            }
            _ => Types::UNKNOWN,
        },
        "$sformatf" | "$psprintf" | "$typename" => Types::STRING,
        "$past" | "$sampled" => arg_ty(e, 0).unwrap_or(Types::UNKNOWN),
        "$display" | "$displayb" | "$displayo" | "$displayh" | "$write" | "$writeb" | "$writeo" | "$writeh"
        | "$strobe" | "$strobeb" | "$strobeo" | "$strobeh" | "$monitor" | "$monitorb" | "$monitoro" | "$monitorh"
        | "$monitoron" | "$monitoroff" | "$finish" | "$stop" | "$exit" | "$fatal" | "$error" | "$warning" | "$info"
        | "$fdisplay" | "$fwrite" | "$fstrobe" | "$fmonitor" | "$fclose" | "$fflush" | "$readmemh" | "$readmemb"
        | "$writememh" | "$writememb" | "$sformat" | "$swrite" | "$swriteb" | "$swriteo" | "$swriteh" | "$dumpfile"
        | "$dumpvars" | "$dumpon" | "$dumpoff" | "$dumpall" | "$dumpflush" | "$dumplimit" | "$dumpports"
        | "$printtimescale" | "$timeformat" | "$asserton" | "$assertoff" | "$assertkill" | "$assertcontrol"
        | "$assertpasson" | "$assertpassoff" | "$assertfailon" | "$assertfailoff" | "$assertnonvacuouson"
        | "$assertvacuousoff" | "$srandom" | "$q_initialize" | "$q_add" | "$q_remove" | "$set_coverage_db_name"
        | "$load_coverage_db" => Types::VOID,
        _ => Types::UNKNOWN,
    };
    Ok(e.new_expr(ExprKind::SysCall(s), t, n, xs))
}

/// An argument naming an instance, a block or a design element, kept as a
/// path.
fn scope_arg<'a>(e: &mut Elab<'a, '_>, sc: ScopeId, a: NodeId) -> R<'a, Option<ExprId>> {
    match e.kind(a) {
        Kind::Name => {
            let s = e.sym_of(a);
            let scope_like = match lookup::lookup(e, sc, s) {
                Some(Binding::Instance(_) | Binding::Instances(..) | Binding::Scope(_)) => true,
                None => e.defs.contains_key(&s) || e.d.tops.iter().any(|&i| e.d.instances[i.0 as usize].name == s),
                _ => false,
            };
            Ok(scope_like.then(|| e.new_expr(ExprKind::Hier { path: vec![s], object: None }, Types::UNKNOWN, a, vec![])))
        }
        Kind::Member => match hier(e, sc, a)? {
            Some(x) if matches!(e.ex(x).kind, ExprKind::Hier { object: None, .. }) => Ok(Some(x)),
            _ => Ok(None),
        },
        _ => Ok(None),
    }
}

/// The type returned by a built-in method of an array, queue, string or
/// enum (6.16, 6.19.5, 7.5–7.12).
pub(crate) fn builtin_method_type(e: &mut Elab, bt: TypeId, m: Sym, args: &[ExprId]) -> TypeId {
    let name = e.name(m);
    let _ = args;
    match e.d.types.get(bt).clone() {
        Type::String => match name {
            "len" | "compare" | "icompare" | "atoi" | "atohex" | "atooct" | "atobin" => Types::INT,
            "getc" => e.d.types.intern(Type::Vector { atom: crate::design::types::Atom::Byte, signed: true, dims: vec![] }),
            "toupper" | "tolower" | "substr" => Types::STRING,
            "atoreal" => Types::REAL,
            "putc" | "itoa" | "hextoa" | "octtoa" | "bintoa" | "realtoa" => Types::VOID,
            _ => Types::UNKNOWN,
        },
        Type::Enum { .. } => match name {
            "first" | "last" | "next" | "prev" => bt,
            "num" => Types::INT,
            "name" => Types::STRING,
            _ => Types::UNKNOWN,
        },
        Type::Unpacked { elem, dim } => match name {
            "size" | "num" => Types::INT,
            "exists" => Types::INT,
            "delete" | "push_back" | "push_front" | "insert" | "reverse" | "sort" | "rsort" | "shuffle" => Types::VOID,
            "pop_front" | "pop_back" => elem,
            "first" | "last" | "next" | "prev" if matches!(dim, UDim::Assoc(_) | UDim::Wildcard) => Types::INT,
            "find" | "find_first" | "find_last" | "unique" | "min" | "max" => {
                e.d.types.intern(Type::Unpacked { elem, dim: UDim::Queue(None) })
            }
            "find_index" | "find_first_index" | "find_last_index" | "unique_index" => {
                let it = match dim {
                    UDim::Assoc(t) => t,
                    _ => Types::INT,
                };
                e.d.types.intern(Type::Unpacked { elem: it, dim: UDim::Queue(None) })
            }
            "sum" | "product" | "and" | "or" | "xor" => elem,
            "map" => Types::UNKNOWN,
            _ => Types::UNKNOWN,
        },
        Type::Event => match name {
            "triggered" => Types::BIT,
            _ => Types::UNKNOWN,
        },
        _ => Types::UNKNOWN,
    }
}

/// `new`, `new(args)`, `new[n]`, `new[n](init)`, `new obj`.
fn new_expr<'a>(e: &mut Elab<'a, '_>, sc: ScopeId, n: NodeId, target: Option<TypeId>) -> R<'a, ExprId> {
    let kids = e.kids(n).to_vec();
    let t = target.unwrap_or(Types::UNKNOWN);
    let is_array = e.ast.tokens_of(n).get(1).is_some_and(|t| t.0 == "[");
    let mut xs = Vec::new();
    if is_array {
        super::checks::new_array(e, t, n)?;
        for &k in &kids {
            let x = if xs.len() == 1 && target.is_some() { rvalue(e, sc, k, t)? } else { expr(e, sc, k)? };
            xs.push(x);
        }
        return Ok(e.new_expr(ExprKind::New, t, n, xs));
    }
    // A class: check the constructor's arguments.
    let class = match e.d.types.get(t) {
        Type::Class(c) => Some(*c),
        _ => None,
    };
    if let Some(c) = class {
        super::checks::new_class(e, sc, c, n)?;
        let args: Vec<NodeId> = match kids.iter().copied().find(|&k| e.kind(k) == Kind::Args) {
            Some(a) => e.kids(a).to_vec(),
            None => vec![],
        };
        let ctor_name = e.sym("new");
        let own = e.scope(e.d.classes[c.0 as usize].scope).names.get(&ctor_name).copied();
        if let Some(Binding::Subroutine(ctor)) = own {
            let call = call_sub(e, sc, ctor, n, &args, None)?;
            xs.push(call);
        } else if args.iter().any(|&a| e.kind(a) != Kind::Empty) {
            return Err(e.err(n, "the class has no constructor that takes arguments (8.7)"));
        }
    } else {
        for &k in &kids {
            if e.kind(k) == Kind::Args {
                for &a in e.kids(k).to_vec().iter() {
                    xs.push(arg_expr(e, sc, a)?);
                }
            } else {
                xs.push(expr(e, sc, k)?);
            }
        }
    }
    Ok(e.new_expr(ExprKind::New, t, n, xs))
}

/// The bits of a constant integral expression, for checks.
#[allow(dead_code)]
pub(crate) fn const_bits(e: &Elab, x: ExprId) -> Option<Bits> {
    match &e.ex(x).kind {
        ExprKind::Const(c) => e.d.consts[c.0 as usize].as_bits().cloned(),
        _ => None,
    }
}
