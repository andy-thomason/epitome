//! `format_args!`: its format string parsed, and the call made code that
//! builds the `core::fmt::Arguments` that `core`'s own `fmt::write` reads:
//! a template of literal pieces and placeholders (the encoding `core`
//! documents on `Arguments`), and an array of `rt::Argument`s, one for
//! each placeholder (and each width or precision taken from an argument).
//! Later stages then compile it as any other code:
//!
//! ```text
//! format_args!("{} is {:>4}", a, b)
//! match (&a, &b,) {
//!     args => unsafe {
//!         ::core::fmt::Arguments::new(
//!             b"\xC0\x04 is \xC3...",
//!             &[::core::fmt::rt::Argument::new_display(args.0),
//!               ::core::fmt::rt::Argument::new_display(args.1)],
//!         )
//!     },
//! }
//! ```
//!
//! The arguments are each evaluated once, in order, by reference, as
//! rustc's expansion does. A format string with no placeholders is
//! `Arguments::from_str`.

use super::builtin::{Context, Eager, args, eager_literal};
use crate::lexer::Token;
use crate::source::Error;

/// Which argument a placeholder (or a count) takes.
#[derive(Clone, Debug, PartialEq)]
enum ArgRef {
    /// The next positional argument.
    Next,
    Index(usize),
    Name(String),
}

/// A width or precision.
#[derive(Clone, Debug, PartialEq)]
enum Count {
    Literal(u16),
    /// From an argument (`1$`, `name$`), a `usize`.
    Arg(ArgRef),
    /// `.*`: from the next positional argument.
    Star,
}

/// The formatting trait a placeholder uses, by `rt::Argument`'s
/// constructor for it.
#[derive(Clone, Copy, Debug)]
enum Trait {
    Display,
    Debug,
    LowerHex,
    UpperHex,
    Octal,
    Binary,
    LowerExp,
    UpperExp,
    Pointer,
}

impl Trait {
    fn constructor(self) -> &'static str {
        match self {
            Trait::Display => "new_display",
            Trait::Debug => "new_debug",
            Trait::LowerHex => "new_lower_hex",
            Trait::UpperHex => "new_upper_hex",
            Trait::Octal => "new_octal",
            Trait::Binary => "new_binary",
            Trait::LowerExp => "new_lower_exp",
            Trait::UpperExp => "new_upper_exp",
            Trait::Pointer => "new_pointer",
        }
    }
}

/// `FormattingOptions`' flags (`core::fmt`'s `flags` module).
const SIGN_PLUS: u32 = 1 << 21;
const SIGN_MINUS: u32 = 1 << 22;
const ALTERNATE: u32 = 1 << 23;
const ZERO_PAD: u32 = 1 << 24;
const DEBUG_LOWER_HEX: u32 = 1 << 25;
const DEBUG_UPPER_HEX: u32 = 1 << 26;
const WIDTH: u32 = 1 << 27;
const PRECISION: u32 = 1 << 28;
const ALIGN_LEFT: u32 = 0 << 29;
const ALIGN_RIGHT: u32 = 1 << 29;
const ALIGN_CENTER: u32 = 2 << 29;
const ALIGN_UNKNOWN: u32 = 3 << 29;
/// `FormattingOptions::new()`'s: filled with spaces, no alignment.
const DEFAULT_FLAGS: u32 = ' ' as u32 | ALIGN_UNKNOWN;

/// A placeholder: `{arg:spec}`.
#[derive(Clone, Debug)]
struct Placeholder {
    arg: ArgRef,
    tr: Trait,
    /// Fill, sign, `#`, `0`, debug hex and alignment (not the width and
    /// precision bits, which `width` and `precision` say).
    flags: u32,
    width: Option<Count>,
    precision: Option<Count>,
}

enum Piece {
    Literal(String),
    Placeholder(Placeholder),
}

/// Parses a format string (its value, escapes undone) into pieces.
fn parse(s: &str) -> Result<Vec<Piece>, String> {
    let mut out = Vec::new();
    let mut lit = String::new();
    let cs: Vec<char> = s.chars().collect();
    let mut i = 0;
    while i < cs.len() {
        match cs[i] {
            '{' if cs.get(i + 1) == Some(&'{') => {
                lit.push('{');
                i += 2;
            }
            '}' if cs.get(i + 1) == Some(&'}') => {
                lit.push('}');
                i += 2;
            }
            '}' => return Err("unmatched `}` in format string".into()),
            '{' => {
                let close = cs[i..].iter().position(|&c| c == '}').ok_or("unmatched `{` in format string")? + i;
                let inner: String = cs[i + 1..close].iter().collect();
                if !lit.is_empty() {
                    out.push(Piece::Literal(std::mem::take(&mut lit)));
                }
                out.push(Piece::Placeholder(placeholder(&inner)?));
                i = close + 1;
            }
            c => {
                lit.push(c);
                i += 1;
            }
        }
    }
    if !lit.is_empty() {
        out.push(Piece::Literal(lit));
    }
    Ok(out)
}

fn is_ident_start(c: char) -> bool {
    c == '_' || c.is_alphabetic()
}

fn is_ident_char(c: char) -> bool {
    c == '_' || c.is_alphanumeric()
}

/// An argument at the start of `s`: digits or an identifier.
fn arg_ref(s: &[char]) -> (Option<ArgRef>, usize) {
    if s.first().is_some_and(|c| c.is_ascii_digit()) {
        let n = s.iter().take_while(|c| c.is_ascii_digit()).count();
        let v: String = s[..n].iter().collect();
        return (v.parse().ok().map(ArgRef::Index), n);
    }
    if s.first().is_some_and(|&c| is_ident_start(c)) {
        let n = s.iter().take_while(|&&c| is_ident_char(c)).count();
        return (Some(ArgRef::Name(s[..n].iter().collect())), n);
    }
    (None, 0)
}

/// A count (a width or precision) at the start of `s`: a number, or an
/// argument followed by `$`.
fn count(s: &[char]) -> Result<(Option<Count>, usize), String> {
    let (a, n) = arg_ref(s);
    match a {
        Some(a) if s.get(n) == Some(&'$') => Ok((Some(Count::Arg(a)), n + 1)),
        Some(ArgRef::Index(v)) => {
            let v = u16::try_from(v).map_err(|_| "a width or precision over 65535 in a format string")?;
            Ok((Some(Count::Literal(v)), n))
        }
        _ => Ok((None, 0)),
    }
}

/// What is between a placeholder's braces: `arg:spec`.
fn placeholder(inner: &str) -> Result<Placeholder, String> {
    let cs: Vec<char> = inner.trim_end().chars().collect();
    let (arg, mut i) = arg_ref(&cs);
    let mut p = Placeholder {
        arg: arg.unwrap_or(ArgRef::Next),
        tr: Trait::Display,
        flags: DEFAULT_FLAGS,
        width: None,
        precision: None,
    };
    if i == cs.len() {
        return Ok(p);
    }
    if cs[i] != ':' {
        return Err(format!("bad placeholder `{{{inner}}}` in format string"));
    }
    i += 1;
    let align = |c: char| match c {
        '<' => Some(ALIGN_LEFT),
        '>' => Some(ALIGN_RIGHT),
        '^' => Some(ALIGN_CENTER),
        _ => None,
    };
    // [[fill]align]
    let mut fill = ' ';
    let mut al = ALIGN_UNKNOWN;
    if let Some(a) = cs.get(i + 1).and_then(|&c| align(c)) {
        fill = cs[i];
        al = a;
        i += 2;
    } else if let Some(a) = cs.get(i).and_then(|&c| align(c)) {
        al = a;
        i += 1;
    }
    let mut flags = fill as u32 | al;
    // [sign]['#']['0']
    match cs.get(i) {
        Some('+') => {
            flags |= SIGN_PLUS;
            i += 1;
        }
        Some('-') => {
            flags |= SIGN_MINUS;
            i += 1;
        }
        _ => {}
    }
    if cs.get(i) == Some(&'#') {
        flags |= ALTERNATE;
        i += 1;
    }
    // (`0$` is a width from argument 0, not the flag.)
    if cs.get(i) == Some(&'0') && cs.get(i + 1) != Some(&'$') {
        flags |= ZERO_PAD;
        i += 1;
    }
    // [width]
    let (w, n) = count(&cs[i..])?;
    p.width = w;
    i += n;
    // ['.' precision]
    if cs.get(i) == Some(&'.') {
        i += 1;
        if cs.get(i) == Some(&'*') {
            p.precision = Some(Count::Star);
            i += 1;
        } else {
            let (c, n) = count(&cs[i..])?;
            p.precision = Some(c.ok_or_else(|| format!("bad precision in `{{{inner}}}`"))?);
            i += n;
        }
    }
    // type
    let ty: String = cs[i..].iter().collect();
    p.tr = match ty.as_str() {
        "" => Trait::Display,
        "?" => Trait::Debug,
        "x?" => {
            flags |= DEBUG_LOWER_HEX;
            Trait::Debug
        }
        "X?" => {
            flags |= DEBUG_UPPER_HEX;
            Trait::Debug
        }
        "x" => Trait::LowerHex,
        "X" => Trait::UpperHex,
        "o" => Trait::Octal,
        "b" => Trait::Binary,
        "e" => Trait::LowerExp,
        "E" => Trait::UpperExp,
        "p" => Trait::Pointer,
        _ => return Err(format!("unknown format trait `{ty}`")),
    };
    p.flags = flags;
    Ok(p)
}

/// Appends literal piece `s` to `template`: each run of at most 127 bytes
/// as its length and bytes, longer ones as 0x80 and a 16-bit length (cut
/// at characters' boundaries).
fn push_literal(template: &mut Vec<u8>, s: &str) {
    let mut rest = s;
    while !rest.is_empty() {
        let mut n = rest.len().min(u16::MAX as usize);
        while !rest.is_char_boundary(n) {
            n -= 1;
        }
        let (part, tail) = rest.split_at(n);
        if part.len() < 0x80 {
            template.push(part.len() as u8);
        } else {
            template.push(0x80);
            template.extend_from_slice(&(part.len() as u16).to_le_bytes());
        }
        template.extend_from_slice(part.as_bytes());
        rest = tail;
    }
}

/// `b"..."` of `bytes`.
fn byte_string(bytes: &[u8]) -> String {
    let mut s = String::from("b\"");
    for &b in bytes {
        match b {
            b'"' | b'\\' => {
                s.push('\\');
                s.push(b as char);
            }
            0x20..=0x7e => s.push(b as char),
            _ => s.push_str(&format!("\\x{b:02X}")),
        }
    }
    s.push('"');
    s
}

/// Expands `format_args!` (`format_args_nl!` with `newline`). `Ok(None)`
/// leaves the call as it is, where the format string is not a literal
/// expansion can see.
pub fn format_args<'s>(
    cx: &Context<'_, 's>,
    input: &[Token<'s>],
    newline: bool,
) -> Result<Option<Vec<Token<'s>>>, Error<'s>> {
    let mut parts = args(input);
    if parts.last().is_some_and(|p| p.is_empty()) {
        parts.pop();
    }
    let Some((&first, rest)) = parts.split_first() else {
        return Err(cx.error("`format_args!` without a format string"));
    };
    let expanded;
    let first = match eager_literal(cx, first)? {
        Eager::Literal(t) => {
            expanded = t;
            &expanded[..]
        }
        Eager::NotACall => first,
        Eager::Left => return Ok(None),
    };
    let text = match first {
        [t] if t.is_string() && !t.0.starts_with('b') && !t.0.starts_with('c') => crate::lower::parse_str(t.0)
            .and_then(|b| String::from_utf8(b).ok())
            .ok_or_else(|| cx.error("bad format string"))?,
        _ => return Ok(None),
    };
    let mut text = text;
    if newline {
        text.push('\n');
    }
    let pieces = parse(&text).map_err(|e| cx.error(e))?;

    // The arguments: positional, then named (`name = expr`), then the
    // names the string captures (`{x}`), each a slot of the tuple.
    let mut positional: Vec<&[Token<'s>]> = Vec::new();
    let mut named: Vec<(String, Vec<Token<'s>>)> = Vec::new();
    for &p in rest {
        match p {
            [n, eq, e @ ..] if n.is_ident() && eq.0 == "=" && !e.is_empty() => {
                named.push((n.0.to_string(), e.to_vec()))
            }
            _ if named.is_empty() => positional.push(p),
            _ => return Err(cx.error("a positional argument after named ones in `format_args!`")),
        }
    }
    let capture = |name: &str, named: &mut Vec<(String, Vec<Token<'s>>)>| -> Result<usize, Error<'s>> {
        if let Some(k) = named.iter().position(|(n, _)| n == name) {
            return Ok(k);
        }
        // (Written where the call is: it resolves there.)
        let toks = cx.text("<format_args! capture>", name.to_string())?;
        named.push((name.to_string(), toks));
        Ok(named.len() - 1)
    };
    let mut next = 0usize;
    let slot = |a: &ArgRef, next: &mut usize, named: &mut Vec<(String, Vec<Token<'s>>)>| -> Result<usize, Error<'s>> {
        match a {
            ArgRef::Next => {
                *next += 1;
                Ok(*next - 1)
            }
            ArgRef::Index(i) => Ok(*i),
            ArgRef::Name(n) => Ok(positional.len() + capture(n, named)?),
        }
    };

    // The template and the `rt::Argument`s: (slot, constructor).
    let mut template = Vec::new();
    let mut entries: Vec<(usize, &'static str)> = Vec::new();
    let mut has_placeholder = false;
    // Which entry `write` takes next, by itself.
    let mut implicit = 0usize;
    for piece in &pieces {
        match piece {
            Piece::Literal(s) => push_literal(&mut template, s),
            Piece::Placeholder(p) => {
                has_placeholder = true;
                let mut head = 0xC0u8;
                let mut fields = Vec::new();
                let mut flags = p.flags;
                // `.*` takes the next argument before the value does.
                let mut count_entry = |c: &Count,
                                       next: &mut usize,
                                       named: &mut Vec<(String, Vec<Token<'s>>)>|
                 -> Result<(u16, bool), Error<'s>> {
                    match c {
                        Count::Literal(v) => Ok((*v, false)),
                        Count::Arg(a) => {
                            let s = slot(a, next, named)?;
                            entries.push((s, "from_usize"));
                            Ok(((entries.len() - 1) as u16, true))
                        }
                        Count::Star => {
                            let s = slot(&ArgRef::Next, next, named)?;
                            entries.push((s, "from_usize"));
                            Ok(((entries.len() - 1) as u16, true))
                        }
                    }
                };
                let precision = match &p.precision {
                    Some(c) => Some(count_entry(c, &mut next, &mut named)?),
                    None => None,
                };
                let width = match &p.width {
                    Some(c) => Some(count_entry(c, &mut next, &mut named)?),
                    None => None,
                };
                let value = slot(&p.arg, &mut next, &mut named)?;
                entries.push((value, p.tr.constructor()));
                let index = entries.len() - 1;
                if width.is_some() {
                    flags |= WIDTH;
                }
                if precision.is_some() {
                    flags |= PRECISION;
                }
                if flags != DEFAULT_FLAGS {
                    head |= 1;
                    fields.extend_from_slice(&flags.to_le_bytes());
                }
                if let Some((w, indirect)) = width {
                    head |= 2 | if indirect { 16 } else { 0 };
                    fields.extend_from_slice(&w.to_le_bytes());
                }
                if let Some((v, indirect)) = precision {
                    head |= 4 | if indirect { 32 } else { 0 };
                    fields.extend_from_slice(&v.to_le_bytes());
                }
                if index != implicit {
                    head |= 8;
                    fields.extend_from_slice(&(index as u16).to_le_bytes());
                }
                implicit = index + 1;
                template.push(head);
                template.extend_from_slice(&fields);
            }
        }
    }
    template.push(0);

    // The code.
    let mut out: Vec<Token<'s>> = Vec::new();
    if !has_placeholder {
        let s: String = pieces
            .iter()
            .map(|p| match p {
                Piece::Literal(s) => s.as_str(),
                Piece::Placeholder(_) => "",
            })
            .collect();
        // (The arguments are still evaluated, as rustc does.)
        let lead = if positional.is_empty() && named.is_empty() { "" } else { "match (" };
        out.extend(code(cx, lead.to_string())?);
        if !lead.is_empty() {
            push_args(cx, &mut out, &positional, &named)?;
            out.extend(code(cx, ") { _ => ".to_string())?);
        }
        out.extend(code(cx, format!("::core::fmt::Arguments::from_str({})", super::builtin::quote_text(&s)))?);
        if !lead.is_empty() {
            out.extend(code(cx, " }".to_string())?);
        }
        return Ok(Some(out));
    }
    let nslots = positional.len() + named.len();
    if let Some(&(s, _)) = entries.iter().find(|(s, _)| *s >= nslots) {
        return Err(cx.error(format!("`format_args!` refers to argument {s}, but has {nslots}")));
    }
    out.extend(code(cx, "match (".to_string())?);
    push_args(cx, &mut out, &positional, &named)?;
    let mut body = format!(") {{ args => unsafe {{ ::core::fmt::Arguments::new({}, &[", byte_string(&template));
    for (s, ctor) in &entries {
        body.push_str(&format!("::core::fmt::rt::Argument::{ctor}(args.{s}), "));
    }
    body.push_str("]) } }");
    out.extend(code(cx, body)?);
    Ok(Some(out))
}

/// Tokens of `text`, code that resolves in `core::fmt`
/// (`Session::format_texts`).
fn code<'s>(cx: &Context<'_, 's>, text: String) -> Result<Vec<Token<'s>>, Error<'s>> {
    crate::lexer::lex(cx.session.add_format_text(text))
}

/// `&a, &b, ...,` for the arguments, each keeping its own tokens.
fn push_args<'s>(
    cx: &Context<'_, 's>,
    out: &mut Vec<Token<'s>>,
    positional: &[&[Token<'s>]],
    named: &[(String, Vec<Token<'s>>)],
) -> Result<(), Error<'s>> {
    for e in positional.iter().copied().chain(named.iter().map(|(_, t)| &t[..])) {
        out.extend(code(cx, "&(".to_string())?);
        out.extend_from_slice(e);
        out.extend(code(cx, "),".to_string())?);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn template(s: &str) -> Vec<u8> {
        let mut t = Vec::new();
        for p in parse(s).unwrap() {
            match p {
                Piece::Literal(s) => push_literal(&mut t, &s),
                Piece::Placeholder(_) => t.push(0xC0),
            }
        }
        t
    }

    #[test]
    fn parses_pieces_and_specs() {
        let mut want = vec![5u8];
        want.extend_from_slice(b"a{b}c");
        assert_eq!(template("a{{b}}c"), want);
        let Piece::Placeholder(p) = &parse("{0:*^+#012.3x?}").unwrap()[0] else { panic!() };
        assert_eq!(p.arg, ArgRef::Index(0));
        assert_eq!(p.flags, '*' as u32 | ALIGN_CENTER | SIGN_PLUS | ALTERNATE | ZERO_PAD | DEBUG_LOWER_HEX);
        assert_eq!(p.width, Some(Count::Literal(12)));
        assert_eq!(p.precision, Some(Count::Literal(3)));
        let Piece::Placeholder(p) = &parse("{x:>w$.*}").unwrap()[0] else { panic!() };
        assert_eq!(p.arg, ArgRef::Name("x".into()));
        assert_eq!(p.flags, ' ' as u32 | ALIGN_RIGHT);
        assert_eq!(p.width, Some(Count::Arg(ArgRef::Name("w".into()))));
        assert_eq!(p.precision, Some(Count::Star));
        let Piece::Placeholder(p) = &parse("{:0$}").unwrap()[0] else { panic!() };
        assert_eq!(p.flags, DEFAULT_FLAGS);
        assert_eq!(p.width, Some(Count::Arg(ArgRef::Index(0))));
        assert!(parse("{").is_err() && parse("}").is_err());
    }

    #[test]
    fn long_pieces_split() {
        let mut t = Vec::new();
        push_literal(&mut t, &"x".repeat(200));
        assert_eq!(&t[..3], &[0x80, 200, 0]);
        assert_eq!(t.len(), 203);
    }
}
