//! The arguments of `asm!`, as tokens: its template strings, its operands
//! and its options. Expansion uses this to find what to parse (each
//! operand's expressions, which it adds to the call as children, after the
//! templates it works out); lowering uses it to know what each child is.

use crate::lexer::Token;

/// What an operand does.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Dir {
    In,
    Out,
    LateOut,
    InOut,
    InLateOut,
    Const,
    Sym,
}

#[derive(Clone, Debug)]
pub struct Operand<'t, 's> {
    pub name: Option<&'s str>,
    pub dir: Dir,
    /// What is inside the parentheses: a register class (`reg`) or a
    /// register (`"eax"`). Empty for `const` and `sym`.
    pub spec: &'t [Token<'s>],
    /// Its expressions, in order: one, or for `inout(..) a => b` two; `_`
    /// is none (an output nothing keeps).
    pub exprs: Vec<Option<&'t [Token<'s>]>>,
}

#[derive(Clone, Debug, Default)]
pub struct Asm<'t, 's> {
    /// The template arguments, joined by newlines.
    pub templates: Vec<&'t [Token<'s>]>,
    pub operands: Vec<Operand<'t, 's>>,
    pub options: Vec<&'s str>,
    pub clobber_abis: Vec<&'t [Token<'s>]>,
}

impl Asm<'_, '_> {
    /// Every operand expression present, in order: the order expansion
    /// adds them in.
    pub fn exprs(&self) -> impl Iterator<Item = (usize, usize)> + '_ {
        self.operands.iter().enumerate().flat_map(|(i, o)| o.exprs.iter().enumerate().filter(|e| e.1.is_some()).map(move |(j, _)| (i, j)))
    }
}

/// Splits at commas outside brackets.
fn split<'t, 's>(input: &'t [Token<'s>], sep: &str) -> Vec<&'t [Token<'s>]> {
    let mut out = Vec::new();
    let mut depth = 0i32;
    let mut start = 0;
    let mut i = 0;
    while i < input.len() {
        match input[i].0 {
            "(" | "[" | "{" => depth += 1,
            ")" | "]" | "}" => depth -= 1,
            t if depth == 0 && t == sep => {
                out.push(&input[start..i]);
                start = i + 1;
            }
            // `=>` as two tokens.
            "=" if depth == 0 && sep == "=>" && input.get(i + 1).is_some_and(|n| n.0 == ">" && input[i].touches(*n)) => {
                out.push(&input[start..i]);
                start = i + 2;
                i += 1;
            }
            _ => {}
        }
        i += 1;
    }
    if start < input.len() {
        out.push(&input[start..]);
    }
    out
}

/// The tokens inside `word(...)` at the start of `arg`, and the rest.
fn call<'t, 's>(arg: &'t [Token<'s>], word: &str) -> Option<(&'t [Token<'s>], &'t [Token<'s>])> {
    if arg.first()?.0 != word || arg.get(1)?.0 != "(" {
        return None;
    }
    let mut depth = 0;
    for (i, t) in arg.iter().enumerate().skip(1) {
        match t.0 {
            "(" | "[" | "{" => depth += 1,
            ")" | "]" | "}" => {
                depth -= 1;
                if depth == 0 {
                    return Some((&arg[2..i], &arg[i + 1..]));
                }
            }
            _ => {}
        }
    }
    None
}

/// Reads `asm!`'s arguments; `None` if they are not its form.
pub fn parse<'t, 's>(input: &'t [Token<'s>]) -> Option<Asm<'t, 's>> {
    let mut asm = Asm::default();
    for arg in split(input, ",") {
        if arg.is_empty() {
            continue;
        }
        if let Some((words, rest)) = call(arg, "options") {
            if !rest.is_empty() {
                return None;
            }
            asm.options.extend(words.iter().filter(|t| t.0 != ",").map(|t| t.0));
            continue;
        }
        if let Some((abis, _)) = call(arg, "clobber_abi") {
            asm.clobber_abis.push(abis);
            continue;
        }
        // `name = operand`.
        let (name, op) = if arg.len() > 2 && arg[0].is_ident() && arg[1].0 == "=" && arg[2].0 != ">" {
            (Some(arg[0].0), &arg[2..])
        } else {
            (None, arg)
        };
        let dir = match op.first()?.0 {
            "in" => Some(Dir::In),
            "out" => Some(Dir::Out),
            "lateout" => Some(Dir::LateOut),
            "inout" => Some(Dir::InOut),
            "inlateout" => Some(Dir::InLateOut),
            "const" => Some(Dir::Const),
            "sym" => Some(Dir::Sym),
            _ => None,
        };
        let Some(dir) = dir.filter(|d| matches!(d, Dir::Const | Dir::Sym) || op.get(1).is_some_and(|t| t.0 == "(")) else {
            // A template, before any operand.
            if !asm.operands.is_empty() || name.is_some() {
                return None;
            }
            asm.templates.push(arg);
            continue;
        };
        let (spec, rest) = match dir {
            Dir::Const | Dir::Sym => (&op[..0], &op[1..]),
            _ => {
                let w = op[0].0;
                call(op, w)?
            }
        };
        let expr = |t: &'t [Token<'s>]| (!(t.len() == 1 && t[0].0 == "_")).then_some(t);
        let exprs = match dir {
            Dir::InOut | Dir::InLateOut => {
                let parts = split(rest, "=>");
                match parts.as_slice() {
                    [a] => vec![expr(a)],
                    [a, b] => vec![expr(a), expr(b)],
                    _ => return None,
                }
            }
            _ => vec![expr(rest)],
        };
        asm.operands.push(Operand { name, dir, spec, exprs });
    }
    Some(asm)
}
