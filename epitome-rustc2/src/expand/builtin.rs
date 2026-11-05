//! Built-in macros: those expansion does itself (`cfg_select!`, `cfg!`,
//! `include!`, `stringify!`, `concat!`, `line!`, `column!`, `file!`,
//! `module_path!`, `env!`, `option_env!`, `compile_error!`), and those it
//! leaves as calls for later stages (`format_args!`, `asm!`, ...).

use super::lex_text;
use crate::ast::NodeId;
use crate::cfg::string_value;
use crate::lexer::Token;
use crate::session::{ParsedFile, Session};
use crate::source::Error;

/// Built-ins expansion does itself.
const EXPANDED: &[&str] = &[
    "cfg_select",
    "cfg",
    "include",
    "stringify",
    "concat",
    "line",
    "column",
    "file",
    "module_path",
    "env",
    "option_env",
    "compile_error",
    "pattern_type",
];

/// Built-ins left as calls, for type checking or lowering to handle.
const LEFT: &[&str] = &[
    "format_args",
    "format_args_nl",
    "const_format_args",
    "asm",
    "global_asm",
    "naked_asm",
    "assert",
    "concat_idents",
    "concat_bytes",
    "include_str",
    "include_bytes",
    "trace_macros",
    "log_syntax",
    "offset_of",
    "type_ascribe",
    "deref",
    "derive",
    "derive_const",
    "test",
    "bench",
    "test_case",
    "global_allocator",
    "cfg_accessible",
    "cfg_eval",
    "alloc_error_handler",
    "core_panic",
    "std_panic",
    "unreachable_2015",
    "unreachable_2021",
    "contract_requires",
    "contract_ensures",
    "autodiff_forward",
    "autodiff_reverse",
    "define_opaque",
    "iter",
    "unsafe_binder",
];

/// Whether `name` is a built-in macro.
pub fn is_builtin(name: &str) -> bool {
    EXPANDED.contains(&name) || LEFT.contains(&name)
}

/// What a built-in may need to know about its call.
pub struct Context<'c, 's> {
    pub session: &'s Session,
    pub file: &'c ParsedFile<'s>,
    pub call: NodeId,
    pub crate_name: &'c str,
}

impl<'c, 's> Context<'c, 's> {
    /// The token in the file's own text that the call stands at (the
    /// outermost call, for one made by a macro).
    fn site(&self) -> Token<'s> {
        let ast = &self.file.ast;
        let i = ast.origin_of(ast.node(self.call).first as usize) as usize;
        ast.tokens[i]
    }

    fn error(&self, message: impl Into<String>) -> Error<'s> {
        Error::new(self.site().0, message)
    }

    fn text(&self, name: &str, text: String) -> Result<Vec<Token<'s>>, Error<'s>> {
        lex_text(self.session, name, text)
    }
}

/// Splits macro input at top-level commas.
fn args<'t, 's>(input: &'t [Token<'s>]) -> Vec<&'t [Token<'s>]> {
    let mut out = Vec::new();
    let mut depth = 0i32;
    let mut start = 0;
    for (i, t) in input.iter().enumerate() {
        match t.0 {
            "(" | "[" | "{" => depth += 1,
            ")" | "]" | "}" => depth -= 1,
            "," if depth == 0 => {
                out.push(&input[start..i]);
                start = i + 1;
            }
            _ => {}
        }
    }
    if start < input.len() {
        out.push(&input[start..]);
    }
    out
}

/// A single string literal argument, without quotes.
fn string_arg<'s>(input: &[Token<'s>]) -> Option<&'s str> {
    match args(input).as_slice() {
        [[t]] if t.is_string() && !t.0.starts_with('b') && !t.0.starts_with('c') => Some(string_value(t.0)),
        _ => None,
    }
}

/// `"text"` with `\` and `"` escaped.
fn quote(text: &str) -> String {
    let mut s = String::with_capacity(text.len() + 2);
    s.push('"');
    for c in text.chars() {
        match c {
            '"' => s.push_str("\\\""),
            '\\' => s.push_str("\\\\"),
            '\n' => s.push_str("\\n"),
            c => s.push(c),
        }
    }
    s.push('"');
    s
}

/// The tokens, with a space between any two that did not touch.
fn joined(input: &[Token]) -> String {
    let mut s = String::new();
    for (i, t) in input.iter().enumerate() {
        if i > 0 && !input[i - 1].touches(*t) {
            s.push(' ');
        }
        s.push_str(t.0);
    }
    s
}

/// Expands built-in `name`. `Ok(None)` leaves the call as it is.
pub fn expand<'s>(cx: &Context<'_, 's>, name: &str, input: &[Token<'s>]) -> Result<Option<Vec<Token<'s>>>, Error<'s>> {
    match name {
        "cfg_select" => cfg_select(cx, input),
        "cfg" => {
            let Some(cfg) = &cx.session.options.cfg else { return Ok(None) };
            let v = cfg.eval(input, cx.site())?;
            Ok(Some(vec![Token(if v { "true" } else { "false" })]))
        }
        "stringify" => cx.text("<stringify!>", quote(&joined(input))).map(Some),
        "concat" => concat(cx, input),
        "line" | "column" => {
            let site = cx.site();
            let Some(src) = cx.session.sources.find(site.0) else { return Ok(None) };
            let loc = src.locate(site.0);
            let n = if name == "line" { loc.line } else { loc.col };
            cx.text("<line!>", format!("{n}u32")).map(Some)
        }
        "file" => cx.text("<file!>", quote(&cx.file.source.name)).map(Some),
        "module_path" => {
            let mut path = vec![cx.crate_name.to_string()];
            path.extend(cx.file.module.iter().cloned());
            cx.text("<module_path!>", quote(&path.join("::"))).map(Some)
        }
        "env" | "option_env" => {
            let Some(var) = args(input).first().and_then(|a| string_arg(a)) else { return Ok(None) };
            let value = std::env::var(var).ok();
            let text = match (name, value) {
                ("env", Some(v)) => quote(&v),
                ("env", None) => return Err(cx.error(format!("environment variable `{var}` is not set"))),
                (_, Some(v)) => format!("::core::option::Option::Some({})", quote(&v)),
                (_, None) => "::core::option::Option::None::<&'static str>".to_string(),
            };
            cx.text("<env!>", text).map(Some)
        }
        "compile_error" => {
            let msg = string_arg(input).unwrap_or("compile_error!");
            Err(cx.error(msg.to_string()))
        }
        "include" => include(cx, input),
        // `pattern_type!(u32 is 1..)`: the type, without the pattern that
        // narrows it (which only gives the layout a niche).
        "pattern_type" => {
            let end = input.iter().position(|t| t.0 == "is").unwrap_or(input.len());
            Ok(Some(input[..end].to_vec()))
        }
        _ => Ok(None),
    }
}

/// `cfg_select! { pred => { tokens } ... _ => { tokens } }`: the tokens of
/// the first arm whose predicate holds. An arm may also be `pred =>
/// expression,`.
fn cfg_select<'s>(cx: &Context<'_, 's>, input: &[Token<'s>]) -> Result<Option<Vec<Token<'s>>>, Error<'s>> {
    let Some(cfg) = &cx.session.options.cfg else { return Ok(None) };
    let close = super::mbe_delims(input);
    let mut i = 0;
    while i < input.len() {
        // The predicate runs to `=>` at the top level.
        let start = i;
        while i < input.len() && !(input[i].0 == "=" && input.get(i + 1).is_some_and(|t| t.0 == ">")) {
            if close[i] != usize::MAX {
                i = close[i];
            }
            i += 1;
        }
        if i >= input.len() {
            return Err(cx.error("expected `=>` in `cfg_select!`"));
        }
        let pred = &input[start..i];
        i += 2;
        let (body, next) = if input.get(i).is_some_and(|t| t.0 == "{") && close[i] != usize::MAX {
            (&input[i + 1..close[i]], close[i] + 1)
        } else {
            let s = i;
            while i < input.len() && input[i].0 != "," {
                if close[i] != usize::MAX {
                    i = close[i];
                }
                i += 1;
            }
            (&input[s..i], i)
        };
        i = next;
        if input.get(i).is_some_and(|t| t.0 == ",") {
            i += 1;
        }
        let holds = match pred {
            [t] if t.0 == "_" => true,
            _ => cfg.eval(pred, cx.site())?,
        };
        if holds {
            return Ok(Some(body.to_vec()));
        }
    }
    Err(cx.error("no arm of `cfg_select!` matches"))
}

/// `concat!("a", 1, 'c', true)`: one string literal. Arguments that are not
/// literals (yet) leave the call.
fn concat<'s>(cx: &Context<'_, 's>, input: &[Token<'s>]) -> Result<Option<Vec<Token<'s>>>, Error<'s>> {
    let mut out = String::new();
    for arg in args(input) {
        match arg {
            [t] if t.is_string() && !t.0.starts_with('b') && !t.0.starts_with('c') => {
                if t.0.starts_with('r') {
                    // A raw string's text has no escapes; escape it.
                    let q = quote(string_value(t.0));
                    out.push_str(&q[1..q.len() - 1]);
                } else {
                    out.push_str(string_value(t.0));
                }
            }
            [t] if t.0.starts_with('\'') => out.push_str(&t.0[1..t.0.len() - 1]),
            [t] if t.is_number() => out.push_str(t.0.trim_end_matches(|c: char| c.is_ascii_alphabetic() && c != 'e')),
            [t] if t.0 == "true" || t.0 == "false" => out.push_str(t.0),
            [m, t] if m.0 == "-" && t.is_number() => {
                out.push('-');
                out.push_str(t.0);
            }
            _ => return Ok(None),
        }
    }
    cx.text("<concat!>", format!("\"{out}\"")).map(Some)
}

/// `include!("file")`: the file's tokens, relative to the including file.
fn include<'s>(cx: &Context<'_, 's>, input: &[Token<'s>]) -> Result<Option<Vec<Token<'s>>>, Error<'s>> {
    let Some(rel) = string_arg(input) else { return Ok(None) };
    let dir = cx.file.path.parent().unwrap_or(std::path::Path::new(""));
    let path = dir.join(rel);
    let text =
        std::fs::read_to_string(&path).map_err(|e| cx.error(format!("cannot read `{}`: {e}", path.display())))?;
    cx.session.included.lock().unwrap().push(path.clone());
    cx.text(&path.display().to_string(), text).map(Some)
}
