//! `macro_rules!`: compiling a definition's rules, matching a call's
//! input against them, and transcribing the rule that matched.
//!
//! A definition is `{ (matcher) => {transcriber}; ... }` with any
//! delimiters. A matcher is tokens, groups, metavariables (`$x:expr`) and
//! repetitions (`$( ... ) sep? op`); fragments are read by the ordinary
//! parser (`FragmentParser`), from the position the matcher has reached.
//!
//! Matching tries each rule in order, and within a rule backtracks over
//! repetition counts. The first rule that matches the whole input wins.
//!
//! Transcription copies tokens: the transcriber's own, and for each
//! metavariable the tokens it matched. An `expr` that is not a single
//! operand (`a + b`) is put in parentheses, so that it stays one
//! expression wherever it lands (`$e * 2`), as rustc keeps it one node.

use crate::ast::Kind;
use crate::lexer::{Edition, Token};
use crate::parse::{FragKind, FragmentParser};
use crate::source::Error;
use std::collections::HashMap;

/// A fragment specifier.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Frag {
    Ident,
    Lifetime,
    Literal,
    Tt,
    Parsed(FragKind),
}

fn frag_of(name: &str, edition: Edition) -> Option<Frag> {
    Some(match name {
        "ident" => Frag::Ident,
        "lifetime" => Frag::Lifetime,
        "literal" => Frag::Literal,
        "tt" => Frag::Tt,
        "expr" if edition >= Edition::E2024 => Frag::Parsed(FragKind::Expr),
        "expr" | "expr_2021" => Frag::Parsed(FragKind::Expr2021),
        "ty" => Frag::Parsed(FragKind::Ty),
        "pat" if edition >= Edition::E2021 => Frag::Parsed(FragKind::Pat),
        "pat" | "pat_param" => Frag::Parsed(FragKind::PatParam),
        "path" => Frag::Parsed(FragKind::Path),
        "block" => Frag::Parsed(FragKind::Block),
        "stmt" => Frag::Parsed(FragKind::Stmt),
        "item" => Frag::Parsed(FragKind::Item),
        "meta" => Frag::Parsed(FragKind::Meta),
        "vis" => Frag::Parsed(FragKind::Vis),
        _ => return None,
    })
}

/// One element of a matcher.
#[derive(Debug)]
enum M<'a> {
    Tok(&'a str),
    Var(&'a str, Frag),
    Group(&'a str, Vec<M<'a>>),
    Rep(Vec<M<'a>>, Vec<&'a str>, u8),
}

/// One element of a transcriber.
#[derive(Debug)]
enum T<'a> {
    Tok(Token<'a>),
    Var(&'a str, Token<'a>),
    Rep(Vec<T<'a>>, Vec<Token<'a>>),
    /// `$crate`.
    Crate,
    /// `${count(x)}`, `${count(x, depth)}`.
    Count(&'a str, usize),
    /// `${index()}`, `${index(depth)}`.
    Index(usize),
    /// `${ignore(x)}`: uses `x` (so a repetition repeats with it) and
    /// gives nothing.
    Ignore(&'a str),
    /// `${concat(a, $x, "b")}`: one identifier made of the pieces.
    Concat(Vec<Piece<'a>>),
}

/// A piece of `${concat(...)}`.
#[derive(Debug)]
enum Piece<'a> {
    /// A metavariable, `$x`.
    Var(&'a str),
    /// An identifier or literal, as written.
    Text(&'a str),
}

/// A compiled `macro_rules!` definition.
#[derive(Debug)]
pub struct Rules<'a> {
    rules: Vec<(Vec<M<'a>>, Vec<T<'a>>)>,
    edition: Edition,
}

/// What a metavariable matched.
#[derive(Clone, Debug)]
enum Binding {
    /// A range of input tokens, and for a parsed fragment the kind of node.
    Leaf(usize, usize, Frag, Option<Kind>),
    Seq(Vec<Binding>),
}

type Bindings<'a> = HashMap<&'a str, Binding>;

fn close_of(open: &str) -> &'static str {
    match open {
        "(" => ")",
        "[" => "]",
        _ => "}",
    }
}

fn is_open(s: &str) -> bool {
    matches!(s, "(" | "[" | "{")
}

fn is_close(s: &str) -> bool {
    matches!(s, ")" | "]" | "}")
}

/// For each opening delimiter in `toks`, the index of its closing one.
pub(crate) fn match_delims(toks: &[Token]) -> Vec<usize> {
    let mut close = vec![usize::MAX; toks.len()];
    let mut stack = Vec::new();
    for (i, t) in toks.iter().enumerate() {
        if is_open(t.0) {
            stack.push(i);
        } else if is_close(t.0)
            && let Some(o) = stack.pop()
        {
            close[o] = i;
        }
    }
    close
}

impl<'a> Rules<'a> {
    /// Compiles the body of `macro_rules! name { body }`: `toks` are the
    /// tokens between its delimiters.
    pub fn compile(toks: &[Token<'a>], edition: Edition) -> Result<Self, Error<'a>> {
        let close = match_delims(toks);
        let mut rules = Vec::new();
        let mut i = 0;
        while i < toks.len() {
            // Rules are separated by `;`, or by `,` in a macro 2.0.
            if toks[i].0 == ";" || toks[i].0 == "," {
                i += 1;
                continue;
            }
            if !is_open(toks[i].0) || close[i] == usize::MAX {
                return Err(Error::new(toks[i].0, "expected a macro matcher in delimiters"));
            }
            let m_end = close[i];
            let matcher = compile_matcher(toks, &close, i + 1, m_end, edition)?;
            i = m_end + 1;
            if !(toks.get(i).is_some_and(|t| t.0 == "=")
                && toks.get(i + 1).is_some_and(|t| t.0 == ">" && toks[i].touches(*t)))
            {
                return Err(Error::new(toks.get(i).map_or("", |t| t.0), "expected `=>`"));
            }
            i += 2;
            if i >= toks.len() || !is_open(toks[i].0) || close[i] == usize::MAX {
                return Err(Error::new(toks.get(i).map_or("", |t| t.0), "expected a macro transcriber in delimiters"));
            }
            let t_end = close[i];
            let transcriber = compile_transcriber(toks, &close, i + 1, t_end)?;
            rules.push((matcher, transcriber));
            i = t_end + 1;
        }
        Ok(Rules { rules, edition })
    }

    /// Compiles `macro m(args) { body }`, a macro 2.0 of one rule: `args`
    /// and `body` with their delimiters.
    pub fn compile_one(args: &[Token<'a>], body: &[Token<'a>], edition: Edition) -> Result<Self, Error<'a>> {
        let ac = match_delims(args);
        let bc = match_delims(body);
        let matcher = compile_matcher(args, &ac, 1, args.len().saturating_sub(1), edition)?;
        let transcriber = compile_transcriber(body, &bc, 1, body.len().saturating_sub(1))?;
        Ok(Rules { rules: vec![(matcher, transcriber)], edition })
    }

    /// Matches `input` (the call's tokens between its delimiters) against
    /// each rule, and transcribes the first that matches.
    pub fn expand(
        &self,
        input: &[Token<'a>],
        eof: Token<'a>,
        cx: &mut Transcribe<'_, 'a>,
    ) -> Result<Vec<Token<'a>>, String> {
        let close = match_delims(input);
        let mut parsers: HashMap<usize, FragmentParser<'a>> = HashMap::new();
        for (matcher, transcriber) in &self.rules {
            let mut st = MatchState { input, close: &close, eof, edition: self.edition, parsers: &mut parsers };
            let mut b = Bindings::new();
            if st.go(matcher, 0, 0, input.len(), (0, input.len()), true, &mut b).is_some() {
                let mut out = Vec::new();
                transcribe(transcriber, &b, input, &mut Vec::new(), cx, &mut out)?;
                return Ok(out);
            }
        }
        Err("no rule of the macro matches this input".to_string())
    }
}

fn compile_matcher<'a>(
    toks: &[Token<'a>],
    close: &[usize],
    mut i: usize,
    end: usize,
    edition: Edition,
) -> Result<Vec<M<'a>>, Error<'a>> {
    let mut out = Vec::new();
    while i < end {
        let t = toks[i];
        if t.0 == "$" && i + 1 < end {
            let n = toks[i + 1];
            if n.0 == "(" {
                let c = close[i + 1];
                let sub = compile_matcher(toks, close, i + 2, c, edition)?;
                let (sep, op, next) = separator(toks, c + 1, end)?;
                out.push(M::Rep(sub, sep.iter().map(|t| t.0).collect(), op));
                i = next;
                continue;
            }
            if n.is_ident() && toks.get(i + 2).is_some_and(|c| c.0 == ":") && i + 3 < end {
                let f = toks[i + 3];
                let Some(frag) = frag_of(f.0, edition) else {
                    return Err(Error::new(f.0, format!("unknown fragment specifier `{}`", f.0)));
                };
                out.push(M::Var(n.ident_name(), frag));
                i += 4;
                continue;
            }
            if n.0 == "crate" {
                out.push(M::Tok("$"));
                i += 1;
                continue;
            }
        }
        if is_open(t.0) && close[i] != usize::MAX {
            let c = close[i];
            out.push(M::Group(t.0, compile_matcher(toks, close, i + 1, c, edition)?));
            i = c + 1;
            continue;
        }
        out.push(M::Tok(t.0));
        i += 1;
    }
    Ok(out)
}

/// After `$( ... )`: an optional separator and `*`, `+` or `?`. Returns
/// the separator tokens, the operator, and where the repetition ends.
fn separator<'a>(toks: &[Token<'a>], i: usize, end: usize) -> Result<(Vec<Token<'a>>, u8, usize), Error<'a>> {
    let op = |t: Option<&Token>| t.and_then(|t| matches!(t.0, "*" | "+" | "?").then(|| t.0.as_bytes()[0]));
    if let Some(o) = op(toks.get(i)).filter(|_| i < end) {
        return Ok((Vec::new(), o, i + 1));
    }
    // A separator is one token, which may be a several-character operator
    // (`=>`, `&&`): its characters touch.
    let mut j = i + 1;
    while j < end
        && toks[j - 1].is_punct()
        && toks[j].is_punct()
        && toks[j - 1].touches(toks[j])
        && op(toks.get(j)).is_none()
    {
        j += 1;
    }
    match op(toks.get(j)).filter(|_| j < end) {
        Some(o) => Ok((toks[i..j].to_vec(), o, j + 1)),
        None => Err(Error::new(toks.get(i).map_or("", |t| t.0), "expected `*`, `+` or `?` after a repetition")),
    }
}

fn compile_transcriber<'a>(
    toks: &[Token<'a>],
    close: &[usize],
    mut i: usize,
    end: usize,
) -> Result<Vec<T<'a>>, Error<'a>> {
    let mut out = Vec::new();
    while i < end {
        let t = toks[i];
        if t.0 == "$" && i + 1 < end {
            let n = toks[i + 1];
            if n.0 == "(" {
                let c = close[i + 1];
                let sub = compile_transcriber(toks, close, i + 2, c)?;
                let (sep, _, next) = separator(toks, c + 1, end)?;
                out.push(T::Rep(sub, sep));
                i = next;
                continue;
            }
            if n.0 == "crate" {
                out.push(T::Crate);
                i += 2;
                continue;
            }
            if n.0 == "$" {
                out.push(T::Tok(n));
                i += 2;
                continue;
            }
            if n.0 == "{" {
                let c = close[i + 1];
                out.push(meta_expr(&toks[i + 2..c], n)?);
                i = c + 1;
                continue;
            }
            if n.is_ident() {
                out.push(T::Var(n.ident_name(), t));
                i += 2;
                continue;
            }
        }
        out.push(T::Tok(t));
        i += 1;
    }
    Ok(out)
}

/// `count(x)`, `count(x, 1)`, `index()`, `index(1)`, `ignore(x)`.
fn meta_expr<'a>(toks: &[Token<'a>], at: Token<'a>) -> Result<T<'a>, Error<'a>> {
    let bad = || Error::new(at.0, "unknown metavariable expression");
    let name = toks.first().ok_or_else(bad)?.0;
    // The arguments between the parentheses: `$x`, a depth, separated by
    // commas.
    let inner = toks.get(2..toks.len().saturating_sub(1)).unwrap_or(&[]);
    let var = inner.iter().find(|t| t.is_ident()).map(|t| t.ident_name());
    let depth = inner.iter().filter(|t| t.is_number()).find_map(|t| t.0.parse().ok()).unwrap_or(0);
    Ok(match name {
        "count" => T::Count(var.ok_or_else(bad)?, depth),
        "index" => T::Index(depth),
        "ignore" => T::Ignore(var.ok_or_else(bad)?),
        "concat" => {
            let mut pieces = Vec::new();
            let mut k = 0;
            while k < inner.len() {
                let t = inner[k];
                if t.0 == "$" && k + 1 < inner.len() {
                    pieces.push(Piece::Var(inner[k + 1].ident_name()));
                    k += 2;
                } else if t.0 == "," {
                    k += 1;
                } else {
                    pieces.push(Piece::Text(if t.is_string() { crate::cfg::string_value(t.0) } else { t.0 }));
                    k += 1;
                }
            }
            T::Concat(pieces)
        }
        _ => return Err(bad()),
    })
}

struct MatchState<'s, 't, 'a> {
    input: &'t [Token<'a>],
    close: &'t [usize],
    eof: Token<'a>,
    edition: Edition,
    /// One parser per delimited group of the input, made when first
    /// needed: `(start, end)` of the group's contents to the parser.
    parsers: &'s mut HashMap<usize, FragmentParser<'a>>,
}

impl<'s, 't, 'a> MatchState<'s, 't, 'a> {
    /// Matches `pat[pi..]` from input token `i`. With `anchored`, it must
    /// end exactly at `end`; otherwise it may stop early. Returns where
    /// it ended. `group` is the enclosing group's contents, for parsing.
    #[allow(clippy::too_many_arguments)]
    fn go(
        &mut self,
        pat: &[M<'a>],
        pi: usize,
        i: usize,
        end: usize,
        group: (usize, usize),
        anchored: bool,
        b: &mut Bindings<'a>,
    ) -> Option<usize> {
        let Some(m) = pat.get(pi) else {
            return (!anchored || i == end).then_some(i);
        };
        match m {
            M::Tok(t) => {
                if i < end && self.input[i].0 == *t {
                    self.go(pat, pi + 1, i + 1, end, group, anchored, b)
                } else {
                    None
                }
            }
            M::Var(name, frag) => {
                let (j, kind) = self.fragment(*frag, i, end, group)?;
                b.insert(name, Binding::Leaf(i, j, *frag, kind));
                let r = self.go(pat, pi + 1, j, end, group, anchored, b);
                if r.is_none() {
                    b.remove(name);
                }
                r
            }
            M::Group(open, sub) => {
                if i >= end || self.input[i].0 != *open {
                    return None;
                }
                let c = self.close[i];
                if c == usize::MAX || c >= end || self.input[c].0 != close_of(open) {
                    return None;
                }
                let saved = b.clone();
                self.go(sub, 0, i + 1, c, (i + 1, c), true, b)?;
                let r = self.go(pat, pi + 1, c + 1, end, group, anchored, b);
                if r.is_none() {
                    *b = saved;
                }
                r
            }
            M::Rep(sub, sep, op) => {
                // Greedy: take as many iterations as match, then back off
                // one at a time until the rest matches.
                let mut iters: Vec<(usize, Bindings<'a>)> = Vec::new();
                let mut pos = i;
                loop {
                    if *op == b'?' && !iters.is_empty() {
                        break;
                    }
                    let mut at = pos;
                    if !iters.is_empty() && !sep.is_empty() {
                        if at + sep.len() > end || !sep.iter().enumerate().all(|(k, s)| self.input[at + k].0 == *s) {
                            break;
                        }
                        at += sep.len();
                    }
                    let mut ib = Bindings::new();
                    match self.go(sub, 0, at, end, group, false, &mut ib) {
                        // An iteration that reads nothing would repeat for
                        // ever.
                        Some(j) if j > at || sub.is_empty() => {
                            iters.push((j, ib));
                            pos = j;
                            if j == at {
                                break;
                            }
                        }
                        _ => break,
                    }
                }
                let names = var_names(sub);
                loop {
                    let n = iters.len();
                    let ok_count = !(*op == b'+' && n == 0);
                    if ok_count {
                        let after = iters.last().map_or(i, |x| x.0);
                        let mut nb = b.clone();
                        for name in &names {
                            let seq = iters
                                .iter()
                                .map(|(_, ib)| ib.get(name).cloned().unwrap_or(Binding::Seq(vec![])))
                                .collect();
                            nb.insert(name, Binding::Seq(seq));
                        }
                        if let Some(r) = self.go(pat, pi + 1, after, end, group, anchored, &mut nb) {
                            *b = nb;
                            return Some(r);
                        }
                    }
                    iters.pop()?;
                }
            }
        }
    }

    /// Reads one fragment from `i`; returns where it ends and, for parsed
    /// fragments, the kind of node.
    fn fragment(&mut self, frag: Frag, i: usize, end: usize, group: (usize, usize)) -> Option<(usize, Option<Kind>)> {
        let t = *self.input.get(i).filter(|_| i < end)?;
        match frag {
            Frag::Ident => (t.is_ident() && t.0 != "_" && !t.is_literal()).then_some((i + 1, None)),
            Frag::Lifetime => t.is_lifetime().then_some((i + 1, None)),
            Frag::Literal => {
                let lit = |t: Token| t.is_literal() || t.0 == "true" || t.0 == "false";
                if lit(t) {
                    Some((i + 1, None))
                } else if t.0 == "-" && self.input.get(i + 1).is_some_and(|n| i + 1 < end && n.is_literal()) {
                    Some((i + 2, None))
                } else {
                    None
                }
            }
            Frag::Tt => {
                if is_open(t.0) {
                    let c = self.close[i];
                    (c != usize::MAX && c < end).then_some((c + 1, None))
                } else if is_close(t.0) {
                    None
                } else {
                    Some((i + 1, None))
                }
            }
            Frag::Parsed(kind) => {
                if kind == FragKind::Vis && t.0 != "pub" {
                    return Some((i, None));
                }
                let (gs, ge) = group;
                let input = self.input;
                let eof = if ge < input.len() { Token(&input[ge].0[..0]) } else { self.eof };
                let edition = self.edition;
                let p =
                    self.parsers.entry(gs).or_insert_with(|| FragmentParser::new(input[gs..ge].to_vec(), eof, edition));
                let (j, k) = p.parse_at(i - gs, kind)?;
                Some((j + gs, Some(k)))
            }
        }
    }
}

/// The metavariables bound inside a matcher.
fn var_names<'a>(pat: &[M<'a>]) -> Vec<&'a str> {
    let mut out = Vec::new();
    for m in pat {
        match m {
            M::Var(n, _) => out.push(*n),
            M::Group(_, sub) | M::Rep(sub, _, _) => out.extend(var_names(sub)),
            M::Tok(_) => {}
        }
    }
    out
}

/// What transcription needs from the caller.
pub struct Transcribe<'c, 'a> {
    /// The tokens `$crate` becomes: `crate` in the defining crate, else
    /// `::name`.
    pub dollar_crate: Vec<Token<'a>>,
    /// Makes a token from new text (a number for `${count(x)}`).
    pub make: &'c dyn Fn(String) -> Token<'a>,
    /// `(` and `)` tokens to put around an `expr` that is not one operand.
    pub parens: (Token<'a>, Token<'a>),
}

/// The binding of `name` at the current repetition indices.
fn lookup<'b>(b: &'b Bindings, name: &str, idx: &[usize]) -> Option<&'b Binding> {
    let mut cur = b.get(name)?;
    for &k in idx {
        match cur {
            Binding::Seq(v) => cur = v.get(k)?,
            Binding::Leaf(..) => return Some(cur),
        }
    }
    Some(cur)
}

/// How many times a repetition runs: the length of the sequences of the
/// metavariables used in it, which must agree.
fn rep_count(sub: &[T], b: &Bindings, idx: &[usize]) -> Result<usize, String> {
    let mut count: Option<usize> = None;
    let mut names = Vec::new();
    used_vars(sub, &mut names);
    for n in names {
        if let Some(Binding::Seq(v)) = lookup(b, n, idx) {
            match count {
                Some(c) if c != v.len() => {
                    return Err(format!("metavariable `{n}` repeats {} times, but others {c}", v.len()));
                }
                _ => count = Some(v.len()),
            }
        }
    }
    count.ok_or_else(|| "a repetition in the transcriber uses no repeating metavariable".to_string())
}

fn used_vars<'a>(t: &[T<'a>], out: &mut Vec<&'a str>) {
    for x in t {
        match x {
            T::Var(n, _) | T::Count(n, _) | T::Ignore(n) => out.push(n),
            T::Concat(pieces) => out.extend(pieces.iter().filter_map(|p| match p {
                Piece::Var(n) => Some(*n),
                Piece::Text(_) => None,
            })),
            T::Rep(sub, _) => used_vars(sub, out),
            _ => {}
        }
    }
}

/// Whether an expression of this kind is one operand, which needs no
/// parentheses wherever it is put.
pub(crate) fn is_operand(k: Kind) -> bool {
    use Kind::*;
    matches!(
        k,
        Literal
            | PathExpr
            | Call
            | MethodCall
            | Field
            | Index
            | Try
            | Await
            | Tuple
            | Paren
            | Array
            | ArrayRepeat
            | StructExpr
            | Block
            | MacroCall
            | If
            | Match
            | Loop
            | While
            | For
            | ConstBlock
            | Underscore
    )
}

fn transcribe<'a>(
    t: &[T<'a>],
    b: &Bindings<'a>,
    input: &[Token<'a>],
    idx: &mut Vec<usize>,
    cx: &mut Transcribe<'_, 'a>,
    out: &mut Vec<Token<'a>>,
) -> Result<(), String> {
    for x in t {
        match x {
            T::Tok(tok) => out.push(*tok),
            T::Var(name, dollar) => match lookup(b, name, idx) {
                Some(Binding::Leaf(s, e, frag, kind)) => {
                    let wrap = *frag == Frag::Parsed(FragKind::Expr) || *frag == Frag::Parsed(FragKind::Expr2021);
                    let wrap = wrap && kind.is_some_and(|k| !is_operand(k));
                    if wrap {
                        out.push(cx.parens.0);
                    }
                    out.extend_from_slice(&input[*s..*e]);
                    if wrap {
                        out.push(cx.parens.1);
                    }
                }
                Some(Binding::Seq(_)) => return Err(format!("metavariable `{name}` is still repeating here")),
                // Not a metavariable: the `$` and the name are kept.
                None => {
                    out.push(*dollar);
                    out.push((cx.make)(name.to_string()));
                }
            },
            T::Rep(sub, sep) => {
                let n = rep_count(sub, b, idx)?;
                for k in 0..n {
                    if k > 0 {
                        out.extend_from_slice(sep);
                    }
                    idx.push(k);
                    transcribe(sub, b, input, idx, cx, out)?;
                    idx.pop();
                }
            }
            T::Crate => out.extend_from_slice(&cx.dollar_crate),
            T::Count(name, depth) => {
                let n = count(lookup(b, name, idx), *depth);
                out.push((cx.make)(n.to_string()));
            }
            T::Index(depth) => {
                let k = idx.len().checked_sub(1 + depth).map(|d| idx[d]).ok_or("`${index()}` outside a repetition")?;
                out.push((cx.make)(k.to_string()));
            }
            T::Ignore(_) => {}
            T::Concat(pieces) => {
                let mut name = String::new();
                for p in pieces {
                    match p {
                        Piece::Text(t) => name.push_str(t),
                        Piece::Var(v) => match lookup(b, v, idx) {
                            Some(Binding::Leaf(s, e, _, _)) => {
                                for t in &input[*s..*e] {
                                    let t = t.ident_name();
                                    name.push_str(crate::cfg::string_value(t));
                                }
                            }
                            _ => return Err(format!("`${{concat}}` needs `{v}` to be one token here")),
                        },
                    }
                }
                out.push((cx.make)(name));
            }
        }
    }
    Ok(())
}

/// `${count(x)}`: how many times `x` repeats in all, through every level
/// of repetition below here; `${count(x, d)}`: how many repetitions there
/// are `d` levels down.
fn count(b: Option<&Binding>, depth: usize) -> usize {
    match b {
        Some(Binding::Seq(v)) if depth == 0 => v.iter().map(leaves).sum(),
        Some(Binding::Seq(v)) if depth == 1 => v.len(),
        Some(Binding::Seq(v)) => v.iter().map(|x| count(Some(x), depth - 1)).sum(),
        _ => 1,
    }
}

fn leaves(b: &Binding) -> usize {
    match b {
        Binding::Seq(v) => v.iter().map(leaves).sum(),
        Binding::Leaf(..) => 1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::lex;

    fn expand(def: &str, input: &str) -> Result<String, String> {
        let dt = lex(def).unwrap();
        let rules = Rules::compile(&dt, Edition::E2024).map_err(|e| e.message)?;
        let it = lex(input).unwrap();
        let make = |s: String| Token(Box::leak(s.into_boxed_str()));
        let mut cx = Transcribe { dollar_crate: vec![Token("crate")], make: &make, parens: (Token("("), Token(")")) };
        let out = rules.expand(&it, Token(""), &mut cx)?;
        Ok(out.iter().map(|t| t.0).collect::<Vec<_>>().join(" "))
    }

    #[test]
    fn simple_rules() {
        let def = "() => { 0 }; ($x:expr) => { $x * 2 }; ($a:ident, $b:ident) => { $a + $b }";
        assert_eq!(expand(def, "").unwrap(), "0");
        assert_eq!(expand(def, "a + b").unwrap(), "( a + b ) * 2");
        assert_eq!(expand(def, "f(x)").unwrap(), "f ( x ) * 2");
        assert_eq!(expand(def, "p, q").unwrap(), "p + q");
    }

    #[test]
    fn repetitions() {
        let def = "($($k:ident => $v:expr),* $(,)?) => { [$(($k, $v)),*] }";
        assert_eq!(expand(def, "a => 1, b => 2,").unwrap(), "[ ( a , 1 ) , ( b , 2 ) ]");
        assert_eq!(expand(def, "").unwrap(), "[ ]");
        let nested = "($($name:ident: $($t:ty),+);*) => { $(fn $name() { $(let _: $t;)+ })* }";
        assert_eq!(
            expand(nested, "f: u8, u16; g: bool").unwrap(),
            "fn f ( ) { let _ : u8 ; let _ : u16 ; } fn g ( ) { let _ : bool ; }"
        );
    }

    #[test]
    fn tt_munchers_and_groups() {
        let def = "(@acc [$($a:tt)*]) => { $($a)* }; (@acc [$($a:tt)*] $x:tt $($rest:tt)*) => { m!(@acc [$x $($a)*] $($rest)*) }";
        assert_eq!(expand(def, "@acc [] a b").unwrap(), "m ! ( @ acc [ a ] b )");
        assert_eq!(expand(def, "@acc [b a]").unwrap(), "b a");
    }

    #[test]
    fn metavariable_expressions_and_crate() {
        let def = "($($x:ident),*) => { [$(${ignore($x)} ${index()}),*] ${count($x)} $crate::f }";
        assert_eq!(expand(def, "a, b, c").unwrap(), "[ 0 , 1 , 2 ] 3 crate : : f");
    }

    #[test]
    fn no_match() {
        assert!(expand("(a) => {}", "b").is_err());
        assert!(expand("($x:expr) => {}", "").is_err());
    }
}
