//! Assembly text before it is read as instructions: what GNU assemblers do
//! to it first.
//!
//! - **Expressions** of constants (`0x6000 & (0x6000 >> 1)`), as operands
//!   and in directives, with GNU's precedence: `* / % << >>` above
//!   `| & ^`, above `+ -`, above comparisons, above `&&`, above `||`.
//! - **Macros**: `.macro name params` ... `.endm`, used as `name args`,
//!   with `\param` (and `\()`, nothing, and `\@`, the number of the use);
//!   `.rept count` ... `.endr`; and `.if expr` / `.else` / `.endif`.
//! - **RISC-V's pseudo-instructions** of more than one instruction, where
//!   the description asks for them (`"asm-pseudos"`): `li` of any 64-bit
//!   value, and `la` and `lla`, PC-relative (`auipc`, then `addi` of
//!   `%pcrel_lo` of the `auipc`'s label).
//!
//! What comes out is statements the templates read: operands that are
//! expressions become numbers, and names of numbers (a CSR's) become the
//! numbers.
use std::collections::{HashMap, VecDeque};

use crate::machine::Machine;

/// The value of an expression of constants, if it is one.
pub fn eval(s: &str) -> Option<i64> {
    // (Quickly not one: a register, a symbol.)
    let digit_ish = |c: char| c.is_ascii_hexdigit() || matches!(c, 'x' | 'X' | '_' | '\'');
    if s.chars().any(|c| c.is_ascii_alphabetic() && !digit_ish(c)) {
        return None;
    }
    let toks = tokens(s)?;
    if toks.is_empty() {
        return None;
    }
    let mut p = Parser { toks, at: 0 };
    let v = p.binary(0)?;
    (p.at == p.toks.len()).then_some(v)
}

#[derive(Clone, Debug, PartialEq)]
enum Tok {
    Num(i64),
    Op(&'static str),
}

fn tokens(s: &str) -> Option<Vec<Tok>> {
    const OPS: [&str; 22] = [
        "<<", ">>", "<=", ">=", "<>", "==", "!=", "&&", "||", "+", "-", "*", "/", "%", "&", "|", "^", "~", "!", "<",
        ">", "(",
    ];
    let b = s.as_bytes();
    let mut out = Vec::new();
    let mut k = 0;
    while k < b.len() {
        let c = b[k];
        if c.is_ascii_whitespace() {
            k += 1;
            continue;
        }
        if c == b')' {
            out.push(Tok::Op(")"));
            k += 1;
            continue;
        }
        if c.is_ascii_digit() {
            let start = k;
            while k < b.len() && (b[k].is_ascii_alphanumeric() || b[k] == b'_') {
                k += 1;
            }
            out.push(Tok::Num(number(&s[start..k])?));
            continue;
        }
        // A character, as its code: 'a'.
        if c == b'\'' && k + 2 < b.len() && b[k + 2] == b'\'' {
            out.push(Tok::Num(b[k + 1] as i64));
            k += 3;
            continue;
        }
        let op = OPS.iter().find(|o| s[k..].starts_with(*o))?;
        out.push(Tok::Op(op));
        k += op.len();
    }
    Some(out)
}

/// A number as GNU assemblers write them: decimal, `0x` hexadecimal, `0b`
/// binary, or octal with a leading 0.
fn number(s: &str) -> Option<i64> {
    let (digits, radix) = if let Some(h) = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")) {
        (h, 16)
    } else if let Some(h) = s.strip_prefix("0b").or_else(|| s.strip_prefix("0B")) {
        (h, 2)
    } else if s.len() > 1 && s.starts_with('0') {
        (&s[1..], 8)
    } else {
        (s, 10)
    };
    u64::from_str_radix(digits, radix).ok().map(|v| v as i64)
}

struct Parser {
    toks: Vec<Tok>,
    at: usize,
}

impl Parser {
    fn peek(&self) -> Option<&Tok> {
        self.toks.get(self.at)
    }

    fn unary(&mut self) -> Option<i64> {
        let t = self.peek()?.clone();
        self.at += 1;
        match t {
            Tok::Num(v) => Some(v),
            Tok::Op("-") => Some(self.unary()?.wrapping_neg()),
            Tok::Op("+") => self.unary(),
            Tok::Op("~") => Some(!self.unary()?),
            Tok::Op("!") => Some((self.unary()? == 0) as i64),
            Tok::Op("(") => {
                let v = self.binary(0)?;
                (self.peek() == Some(&Tok::Op(")"))).then(|| self.at += 1)?;
                Some(v)
            }
            _ => None,
        }
    }

    /// Operators of precedence `min` and above, left to right.
    fn binary(&mut self, min: u8) -> Option<i64> {
        let mut a = self.unary()?;
        while let Some(Tok::Op(op)) = self.peek() {
            let op = *op;
            let prec = match op {
                "||" => 1,
                "&&" => 2,
                "==" | "!=" | "<>" | "<" | "<=" | ">" | ">=" => 3,
                "+" | "-" => 4,
                "|" | "&" | "^" => 5,
                "*" | "/" | "%" | "<<" | ">>" => 6,
                _ => return Some(a),
            };
            if prec < min {
                break;
            }
            self.at += 1;
            let b = self.binary(prec + 1)?;
            // (Comparisons are -1 when true, as GNU's are.)
            let truth = |x: bool| if x { -1 } else { 0 };
            a = match op {
                "||" => (a != 0 || b != 0) as i64,
                "&&" => (a != 0 && b != 0) as i64,
                "==" => truth(a == b),
                "!=" | "<>" => truth(a != b),
                "<" => truth(a < b),
                "<=" => truth(a <= b),
                ">" => truth(a > b),
                ">=" => truth(a >= b),
                "+" => a.wrapping_add(b),
                "-" => a.wrapping_sub(b),
                "|" => a | b,
                "&" => a & b,
                "^" => a ^ b,
                "*" => a.wrapping_mul(b),
                "/" => a.checked_div(b)?,
                "%" => a.checked_rem(b)?,
                // (Shifts beyond the width give 0; `>>` is logical.)
                "<<" => u32::try_from(b).ok().and_then(|b| (a as u64).checked_shl(b)).unwrap_or(0) as i64,
                _ => u32::try_from(b).ok().and_then(|b| (a as u64).checked_shr(b)).unwrap_or(0) as i64,
            };
        }
        Some(a)
    }
}

/// Splits operands at the commas outside brackets and quotes.
pub fn operands(s: &str) -> Vec<String> {
    let mut out = Vec::new();
    let (mut depth, mut in_str, mut start) = (0i32, false, 0);
    for (k, c) in s.char_indices() {
        match c {
            '"' => in_str = !in_str,
            '(' | '[' | '{' if !in_str => depth += 1,
            ')' | ']' | '}' if !in_str => depth -= 1,
            ',' if depth == 0 && !in_str => {
                out.push(s[start..k].trim().to_string());
                start = k + 1;
            }
            _ => {}
        }
    }
    if !s[start..].trim().is_empty() || !out.is_empty() {
        out.push(s[start..].trim().to_string());
    }
    out
}

/// The labels a statement starts with, and the rest.
fn labels(s: &str) -> (Vec<&str>, &str) {
    let mut out = Vec::new();
    let mut rest = s.trim();
    while let Some((name, after)) = rest.split_once(':')
        && !name.is_empty()
        && name.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '.' | '$'))
    {
        out.push(name);
        rest = after.trim_start();
    }
    (out, rest)
}

struct Macro {
    params: Vec<(String, Option<String>)>,
    body: Vec<String>,
}

/// The statements with macros, `.rept` and `.if` done.
pub fn expand(stmts: Vec<(usize, String)>) -> Result<Vec<(usize, String)>, (usize, String)> {
    // (Most text has none of these.)
    let block = |s: &str| {
        let w = labels(s).1.split_whitespace().next().unwrap_or("");
        matches!(w, ".macro" | ".rept" | ".if" | ".ifne" | ".ifeq")
    };
    if !stmts.iter().any(|(_, s)| block(s)) {
        return Ok(stmts);
    }
    let mut macros: HashMap<String, Macro> = HashMap::new();
    let mut out = Vec::new();
    let mut queue: VecDeque<(usize, String, u32)> = stmts.into_iter().map(|(n, s)| (n, s, 0)).collect();
    // Each open `.if`: whether its statements are kept, and whether a branch
    // of it has been.
    let mut conds: Vec<(bool, bool)> = Vec::new();
    let mut uses = 0;
    // A block's statements, to its end (`.endm`, `.endr`), nested ones in it.
    let block = |queue: &mut VecDeque<(usize, String, u32)>, open: &[&str], close: &str, n: usize| {
        let mut body = Vec::new();
        let mut depth = 0;
        loop {
            let Some((_, s, _)) = queue.pop_front() else {
                return Err((n, format!("no {close}")));
            };
            let w = s.split_whitespace().next().unwrap_or("");
            if open.contains(&w) {
                depth += 1;
            } else if w == close {
                if depth == 0 {
                    return Ok(body);
                }
                depth -= 1;
            }
            body.push(s);
        }
    };
    while let Some((n, s, depth)) = queue.pop_front() {
        if depth > 64 {
            return Err((n, "macros used within themselves".into()));
        }
        let (labs, rest) = labels(&s);
        let (word, args) = rest.split_once(char::is_whitespace).unwrap_or((rest, ""));
        let keep = conds.iter().all(|c| c.0);
        match word {
            ".if" | ".ifne" | ".ifeq" => {
                let v = if keep { eval(args).ok_or((n, format!("a condition not a constant: {args}")))? } else { 0 };
                let yes = if word == ".ifeq" { v == 0 } else { v != 0 };
                conds.push((keep && yes, yes));
                continue;
            }
            ".else" => {
                let (_, taken) = conds.pop().ok_or((n, ".else without .if".to_string()))?;
                let outer = conds.iter().all(|c| c.0);
                conds.push((outer && !taken, true));
                continue;
            }
            ".endif" => {
                conds.pop().ok_or((n, ".endif without .if".to_string()))?;
                continue;
            }
            _ if !keep => continue,
            _ => {}
        }
        for l in &labs {
            out.push((n, format!("{l}:")));
        }
        match word {
            ".macro" => {
                let mut ps = args.split(|c: char| c == ',' || c.is_whitespace()).filter(|p| !p.is_empty());
                let name = ps.next().ok_or((n, "a .macro without a name".to_string()))?.to_string();
                let params = ps
                    .map(|p| {
                        let p = p.split(':').next().unwrap_or(p);
                        match p.split_once('=') {
                            Some((a, d)) => (a.to_string(), Some(d.to_string())),
                            None => (p.to_string(), None),
                        }
                    })
                    .collect();
                let body = block(&mut queue, &[".macro"], ".endm", n)?;
                macros.insert(name, Macro { params, body });
            }
            ".rept" => {
                let count = eval(args).ok_or((n, format!("a .rept count not a constant: {args}")))?;
                let body = block(&mut queue, &[".rept", ".irp"], ".endr", n)?;
                for _ in 0..count.max(0) {
                    for s in body.iter().rev() {
                        queue.push_front((n, s.clone(), depth + 1));
                    }
                }
            }
            ".endm" | ".endr" => return Err((n, format!("{word} without its start"))),
            _ if let Some(m) = macros.get(word) => {
                uses += 1;
                let mut given = operands(args);
                if given.len() == 1 && m.params.len() > 1 {
                    given = args.split_whitespace().map(str::to_string).collect();
                }
                let mut values: Vec<(String, String)> = Vec::new();
                for (k, (p, default)) in m.params.iter().enumerate() {
                    let v = given.get(k).filter(|v| !v.is_empty()).cloned().or_else(|| default.clone());
                    values.push((p.clone(), v.unwrap_or_default()));
                }
                // (Longer names first: `\ab` is not `\a` then `b`.)
                values.sort_by_key(|(p, _)| std::cmp::Reverse(p.len()));
                for s in m.body.iter().rev() {
                    let mut t = s.replace("\\@", &uses.to_string());
                    for (p, v) in &values {
                        t = t.replace(&format!("\\{p}"), v);
                    }
                    queue.push_front((n, t.replace("\\()", ""), depth + 1));
                }
            }
            _ => out.push((n, rest.to_string())),
        }
    }
    if !conds.is_empty() {
        return Err((out.last().map_or(0, |x| x.0), "no .endif".into()));
    }
    Ok(out)
}

/// An instruction's operands as numbers where they are expressions of
/// constants or names of numbers, and its pseudo-instructions of more than
/// one instruction done: the statements it is. `fresh` names a label.
pub fn instruction(m: &Machine, stmt: &str, fresh: &mut dyn FnMut() -> String) -> Vec<String> {
    let (labs, rest) = labels(stmt);
    let (mnem, args) = rest.split_once(char::is_whitespace).unwrap_or((rest, ""));
    let given = operands(args);
    let ops: Vec<String> = given.iter().map(|op| fold(m, mnem, op)).collect();
    let head: String = labs.iter().map(|l| format!("{l}: ")).collect();
    let body = match (m.asm_pseudos.as_str(), mnem, ops.as_slice()) {
        ("riscv", "li", [d, v]) if let Ok(v) = v.parse::<i64>() => li(d, v),
        // (The address of a number is the number.)
        ("riscv", "la" | "lla", [d, v]) if let Ok(v) = v.parse::<i64>() => li(d, v),
        ("riscv", "la" | "lla", [d, s]) if !s.starts_with('%') => {
            let l = fresh();
            vec![format!("{l}: auipc {d}, %pcrel_hi({s})"), format!("addi {d}, {d}, %pcrel_lo({l})")]
        }
        // A load from a symbol, through its destination (or for
        // floating-point, a register given), and a store to one, through
        // the register given.
        ("riscv", m, [d, s, rest @ ..]) if symbolic(m, s) && rest.len() <= 1 => {
            let store = STORES.contains(&m);
            let via = match rest {
                [t] => t.clone(),
                _ if store || m.starts_with('f') => return vec![stmt.to_string()],
                _ => d.clone(),
            };
            let l = fresh();
            vec![format!("{l}: auipc {via}, %pcrel_hi({s})"), format!("{m} {d}, %pcrel_lo({l})({via})")]
        }
        // As written, where nothing is folded (an operand may have a comma
        // of its own: R600's `ExecMask,PredicateBit (MASKED)`).
        _ if ops == given => vec![rest.to_string()],
        _ if ops.is_empty() => vec![mnem.to_string()],
        _ => vec![format!("{mnem} {}", ops.join(", "))],
    };
    let mut out = body;
    out[0] = head + &out[0];
    out
}

const LOADS: [&str; 11] = ["lb", "lh", "lw", "ld", "lbu", "lhu", "lwu", "flh", "flw", "fld", "flq"];
const STORES: [&str; 8] = ["sb", "sh", "sw", "sd", "fsh", "fsw", "fsd", "fsq"];

/// Whether a load's or store's address is a symbol (`tohost`,
/// `tohost + 4`), not an offset from a register.
fn symbolic(mnem: &str, s: &str) -> bool {
    (LOADS.contains(&mnem) || STORES.contains(&mnem))
        && !s.contains('(')
        && s.starts_with(|c: char| c.is_ascii_alphabetic() || c == '_' || c == '.')
}

/// An operand with expressions of constants as their values, and names of
/// numbers the description gives for this mnemonic as the numbers.
fn fold(m: &Machine, mnem: &str, op: &str) -> String {
    if let Some((_, _, v)) = m.asm_names.iter().find(|(p, name, _)| mnem.starts_with(p.as_str()) && name == op) {
        return v.to_string();
    }
    // A register by its other name, alone or as a base (`8(x2)`).
    if !m.asm_register_names.is_empty() {
        let renamed = rename_registers(m, op);
        if renamed != op {
            return fold(m, mnem, &renamed);
        }
    }
    let plain = |s: &str| s.parse::<i64>().is_ok_and(|v| v.to_string() == s);
    if !op.bytes().any(|c| c.is_ascii_digit()) || plain(op) {
        return op.to_string();
    }
    if let Some(v) = eval(op) {
        return if plain(op) { op.to_string() } else { v.to_string() };
    }
    // An offset from a register: `(8 + 8)(a0)`.
    if let Some(inner) = op.strip_suffix(')') {
        let mut depth = 0;
        for (k, c) in inner.char_indices().rev() {
            match c {
                ')' => depth += 1,
                '(' if depth > 0 => depth -= 1,
                '(' => {
                    let (pre, reg) = (&op[..k], &inner[k + 1..]);
                    if eval(reg).is_none()
                        && !pre.trim().is_empty()
                        && let Some(v) = eval(pre)
                    {
                        return format!("{v}({reg})");
                    }
                    break;
                }
                _ => {}
            }
        }
    }
    op.to_string()
}

/// The words of `op` that are registers' other names as the templates'.
fn rename_registers(m: &Machine, op: &str) -> String {
    let mut out = String::new();
    let mut word = String::new();
    let flush = |word: &mut String, out: &mut String| {
        match m.asm_register_names.iter().find(|(a, _)| a == word) {
            Some((_, b)) => out.push_str(b),
            None => out.push_str(word),
        }
        word.clear();
    };
    for c in op.chars() {
        if c.is_ascii_alphanumeric() || c == '_' || c == '.' {
            word.push(c);
        } else {
            flush(&mut word, &mut out);
            out.push(c);
        }
    }
    flush(&mut word, &mut out);
    out
}

/// RISC-V's `li rd, v`: as one instruction where it fits 12 bits; else the
/// upper bits first (`lui`, and `addiw` of the low 12), then for 64-bit
/// values, shifted up and the next 12 bits added, as many times as it
/// takes.
fn li(d: &str, v: i64) -> Vec<String> {
    let sext12 = |x: i64| ((x & 0xfff) ^ 0x800) - 0x800;
    if (-2048..2048).contains(&v) {
        return vec![format!("li {d}, {v}")];
    }
    if i32::try_from(v).is_ok() {
        let lo = sext12(v);
        let hi = v.wrapping_sub(lo) >> 12;
        let mut out = vec![format!("lui {d}, {}", hi & 0xfffff)];
        if lo != 0 {
            out.push(format!("addiw {d}, {d}, {lo}"));
        }
        return out;
    }
    let lo = sext12(v);
    let hi = v.wrapping_sub(lo) >> 12;
    let tz = hi.trailing_zeros();
    let mut out = li(d, hi >> tz);
    out.push(format!("slli {d}, {d}, {}", 12 + tz));
    if lo != 0 {
        out.push(format!("addi {d}, {d}, {lo}"));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expressions() {
        assert_eq!(eval("0x6000 & (0x6000 >> 1)"), Some(0x2000));
        assert_eq!(eval("1 + 2 * 3"), Some(7));
        assert_eq!(eval("1 + 2 & 3"), Some(3));
        assert_eq!(eval("-(1 << 63) >> 63"), Some(1));
        assert_eq!(eval("010"), Some(8));
        assert_eq!(eval("~0"), Some(-1));
        assert_eq!(eval("a0"), None);
        assert_eq!(eval("1.0"), None);
    }

    #[test]
    fn li_sequences() {
        // Each sequence, run, gives its value.
        for v in [
            0,
            2047,
            -2048,
            2048,
            0x7fff_f800,
            -0x8000_0000,
            0x8000_0000,
            0x1234_5678_9abc_def0,
            i64::MIN,
            -1,
            0xffff_ffff,
            0x8000_0000_0000_0001u64 as i64,
        ] {
            let mut x = 0i64;
            for s in li("a0", v) {
                let (m, args) = s.split_once(' ').unwrap();
                let imm: i64 = args.rsplit(", ").next().unwrap().parse().unwrap();
                x = match m {
                    "li" => imm,
                    "lui" => ((imm << 12) as i32) as i64,
                    "addiw" => (x.wrapping_add(imm) as i32) as i64,
                    "slli" => x << imm,
                    "addi" => x.wrapping_add(imm),
                    _ => panic!("{s}"),
                };
            }
            assert_eq!(x, v, "{:?}", li("a0", v));
        }
    }

    #[test]
    fn macros() {
        let stmts =
            [".macro add2 r, v=1", "addi \\r, \\r, \\v", "addi \\r, \\r, \\v", ".endm", "add2 a0", "add2 a1, 3"]
                .iter()
                .map(|s| (0, s.to_string()))
                .collect();
        let out: Vec<String> = expand(stmts).unwrap().into_iter().map(|x| x.1).collect();
        assert_eq!(out, ["addi a0, a0, 1", "addi a0, a0, 1", "addi a1, a1, 3", "addi a1, a1, 3"]);
        let stmts = [".rept 2", "nop", ".endr", ".if 1 - 1", "x", ".else", "y", ".endif"]
            .iter()
            .map(|s| (0, s.to_string()))
            .collect();
        let out: Vec<String> = expand(stmts).unwrap().into_iter().map(|x| x.1).collect();
        assert_eq!(out, ["nop", "nop", "y"]);
    }
}
