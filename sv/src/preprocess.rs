//! The preprocessor (clause 22).
//!
//! The preprocessor reads tokens from a stack of frames: files, and the
//! text of macro expansions. It acts on directives and passes every other
//! token through unchanged, still a slice of the text it came from, so the
//! parser can report any token's position.
//!
//! A macro is expanded by substituting the actual arguments into the
//! macro's text, as text, then reading the result as a new frame. Macro
//! uses in the arguments and in the text are therefore expanded after
//! substitution (22.5.1). Text made this way is added to the `SourceMap`
//! with the place the macro was used, for error messages. A macro with no
//! arguments and nothing to build (no `` `" `` or ` `` `) is read straight
//! from its definition, so its tokens point into the file that defined it.
//!
//! Along with the tokens, the preprocessor records the settings the
//! directives make (`` `timescale ``, `` `default_nettype ``,
//! `` `begin_keywords ``, ...) as a list of states; each token says which
//! state it was read in.

use crate::keywords::Keywords;
use crate::lexer::{Lexer, Token};
use crate::source::{Error, Origin, Source, SourceMap};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::rc::Rc;

type Result<'a, T> = std::result::Result<T, Error<'a>>;

/// The names of compiler directives, which cannot be macro names (22.1),
/// with the informative ones of Annex E.
const DIRECTIVES: &[&str] = &[
    "__FILE__",
    "__LINE__",
    "begin_keywords",
    "celldefine",
    "default_nettype",
    "define",
    "else",
    "elsif",
    "end_keywords",
    "endcelldefine",
    "endif",
    "ifdef",
    "ifndef",
    "include",
    "line",
    "nounconnected_drive",
    "pragma",
    "resetall",
    "timescale",
    "unconnected_drive",
    "undef",
    "undefineall",
    "default_decay_time",
    "default_trireg_strength",
    "delay_mode_distributed",
    "delay_mode_path",
    "delay_mode_unit",
    "delay_mode_zero",
];

/// The coverage constants every compilation starts with (40.3.1).
const PREDEFINED: &[(&str, &str)] = &[
    ("SV_COV_START", "0"),
    ("SV_COV_STOP", "1"),
    ("SV_COV_RESET", "2"),
    ("SV_COV_CHECK", "3"),
    ("SV_COV_MODULE", "10"),
    ("SV_COV_HIER", "11"),
    ("SV_COV_ASSERTION", "20"),
    ("SV_COV_FSM_STATE", "21"),
    ("SV_COV_STATEMENT", "22"),
    ("SV_COV_TOGGLE", "23"),
    ("SV_COV_OVERFLOW", "-2"),
    ("SV_COV_ERROR", "-1"),
    ("SV_COV_NOCOV", "0"),
    ("SV_COV_OK", "1"),
    ("SV_COV_PARTIAL", "2"),
];

/// How deeply macro expansions and `` `include `` files may nest. The
/// standard asks for at least 15 levels of include.
const MAX_MACRO_DEPTH: usize = 256;
const MAX_INCLUDE_DEPTH: usize = 64;

/// What the preprocessor is told on the command line.
#[derive(Clone, Debug, Default)]
pub struct Options {
    /// Searched, in order, for `` `include "file" `` after the including
    /// file's own directory.
    pub include_dirs: Vec<PathBuf>,
    /// Searched for `` `include <file> ``: where the files the standard
    /// defines would live. Epitome has none yet.
    pub system_include_dirs: Vec<PathBuf>,
    /// Macros defined before the first file, as `(name, text)`.
    pub defines: Vec<(String, String)>,
}

/// The net type of implicit nets (22.8).
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum NetType {
    #[default]
    Wire,
    Tri,
    Tri0,
    Tri1,
    Wand,
    Triand,
    Wor,
    Trior,
    Trireg,
    Uwire,
    None,
}

impl NetType {
    fn from_name(s: &str) -> Option<NetType> {
        use NetType::*;
        Some(match s {
            "wire" => Wire,
            "tri" => Tri,
            "tri0" => Tri0,
            "tri1" => Tri1,
            "wand" => Wand,
            "triand" => Triand,
            "wor" => Wor,
            "trior" => Trior,
            "trireg" => Trireg,
            "uwire" => Uwire,
            "none" => None,
            _ => return Option::None,
        })
    }
}

/// A time unit and precision, as powers of ten of a second: `1ns / 10ps`
/// is `{ unit: -9, precision: -11 }` (22.7).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Timescale {
    pub unit: i8,
    pub precision: i8,
}

/// The settings directives make, in effect at a token.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct State {
    pub keywords: Keywords,
    pub timescale: Option<Timescale>,
    pub default_nettype: NetType,
    /// `Some(true)` after `` `unconnected_drive pull1 ``, `Some(false)` after
    /// `pull0`.
    pub unconnected_drive: Option<bool>,
    pub celldefine: bool,
}

/// A token after preprocessing, with the index of its `State`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct PpToken<'a> {
    pub token: Token<'a>,
    pub state: u32,
}

/// The preprocessed compilation unit.
#[derive(Debug)]
pub struct Output<'a> {
    pub tokens: Vec<PpToken<'a>>,
    pub states: Vec<State>,
}

impl<'a> Output<'a> {
    pub fn state(&self, t: PpToken) -> &State {
        &self.states[t.state as usize]
    }

    /// Whether a token is a keyword in the `` `begin_keywords `` set it was
    /// read under.
    pub fn is_keyword(&self, t: PpToken) -> bool {
        !t.token.is_escaped_ident() && self.state(t).keywords.contains(t.token.0)
    }

    /// The tokens as text, as `-E` prints them. Tokens that were next to
    /// each other in a source keep the white space between them; others
    /// are separated by a space.
    pub fn text(&self, map: &SourceMap) -> String {
        let mut out = String::new();
        let mut prev: Option<(&str, usize)> = None;
        for t in &self.tokens {
            let t = t.token.0;
            let Some(id) = map.find(t) else { continue };
            if let Some((p, pid)) = prev {
                let src = map.get(id);
                let gap = (pid == id).then(|| {
                    let end = src.offset_of(p) + p.len();
                    let start = src.offset_of(t);
                    if end <= start { Some(&src.text[end..start]) } else { None }
                });
                match gap.flatten() {
                    Some(g) if g.trim().is_empty() => out += g,
                    // Comments: keep a line break if there was one.
                    Some(g) if g.contains('\n') && crate::lexer::tokenize(g).is_ok_and(|t| t.len() == 1) => {
                        out.push('\n')
                    }
                    _ => out.push(' '),
                }
            }
            out += t;
            prev = Some((t, id));
        }
        out.push('\n');
        out
    }
}

/// Reads `files` (indices into `map`) as one compilation unit.
pub fn preprocess<'a>(map: &'a SourceMap, files: &[usize], options: &Options) -> Result<'a, Output<'a>> {
    let mut pp = Preprocessor::new(map, options);
    let mut prelude: String = PREDEFINED.iter().map(|(n, v)| format!("`define {n} {v}\n")).collect();
    for (name, text) in &options.defines {
        prelude += &format!("`define {name} {text}\n");
    }
    let prelude = map.add(Source::new("<command line>", prelude));
    pp.push_file(prelude, None);
    pp.run(0)?;
    for &id in files {
        let src = map.get(id);
        let dir = Path::new(&src.name).parent().map(Path::to_path_buf);
        pp.push_file(src, dir);
        pp.run(0)?;
    }
    if let Some(c) = pp.conds.first() {
        return Err(Error::new(c.at.0, format!("this `{} has no `endif", c.at.directive_name())));
    }
    Ok(Output { tokens: pp.out, states: pp.states })
}

/// Reads the files at `paths` into `map` and preprocesses them.
pub fn preprocess_files<'a>(
    map: &'a SourceMap,
    paths: &[PathBuf],
    options: &Options,
) -> std::result::Result<Output<'a>, String> {
    let mut ids = Vec::new();
    for path in paths {
        let text = std::fs::read_to_string(path).map_err(|e| format!("{}: error: {e}\n", path.display()))?;
        map.add(Source::new(path.display().to_string(), text));
        ids.push(map.len() - 1);
    }
    preprocess(map, &ids, options).map_err(|e| map.render(&e))
}

struct Macro<'a> {
    /// `None` for a macro defined without parentheses.
    formals: Option<Vec<Formal<'a>>>,
    body: &'a str,
}

struct Formal<'a> {
    name: &'a str,
    default: Option<&'a str>,
}

struct Frame<'a> {
    /// The source the frame's text is part of.
    src: &'a Source,
    lexer: Lexer<'a>,
    /// A file, as opposed to the text of a macro.
    file: bool,
    /// The directory of a file, searched first by `` `include ``.
    dir: Option<PathBuf>,
}

struct Cond<'a> {
    /// The `` `ifdef `` or `` `ifndef `` that opened it.
    at: Token<'a>,
    /// Whether text in the current branch is read.
    active: bool,
    /// Whether some branch has been taken, or the whole construct is inside
    /// ignored text.
    done: bool,
    seen_else: bool,
    /// How many frames were open when it started.
    frames: usize,
}

#[derive(Clone, Copy, PartialEq)]
enum Quote {
    None,
    /// Inside `` `" ... `" ``.
    Single,
    /// Inside `` `""" ... `""" ``.
    Triple,
}

struct Preprocessor<'a, 'o> {
    map: &'a SourceMap,
    options: &'o Options,
    macros: HashMap<&'a str, Rc<Macro<'a>>>,
    frames: Vec<Frame<'a>>,
    conds: Vec<Cond<'a>>,
    keyword_stack: Vec<Keywords>,
    state: State,
    states: Vec<State>,
    out: Vec<PpToken<'a>>,
    /// How many design elements (modules, interfaces, ...) the output is
    /// inside, for the directives that must be outside them.
    design_depth: usize,
    /// The base of the last token, when its digits are still to come.
    pending_base: Option<u8>,
}

impl<'a, 'o> Preprocessor<'a, 'o> {
    fn new(map: &'a SourceMap, options: &'o Options) -> Self {
        Preprocessor {
            map,
            options,
            macros: HashMap::new(),
            frames: Vec::new(),
            conds: Vec::new(),
            keyword_stack: Vec::new(),
            state: State::default(),
            states: vec![State::default()],
            out: Vec::new(),
            design_depth: 0,
            pending_base: None,
        }
    }

    fn push_file(&mut self, src: &'a Source, dir: Option<PathBuf>) {
        self.frames.push(Frame { src, lexer: Lexer::new(&src.text), file: true, dir });
    }

    fn skipping(&self) -> bool {
        self.conds.last().is_some_and(|c| !c.active)
    }

    /// Reads tokens until fewer than `base + 1` frames are left.
    fn run(&mut self, base: usize) -> Result<'a, ()> {
        while self.frames.len() > base {
            let t = self.next_token()?;
            if t.is_eof() {
                self.pop_frame()?;
            } else if t.is_line_continuation() {
                if self.frames.last().unwrap().file {
                    return Err(Error::new(t.0, "a backslash ending a line is only allowed in a macro's text"));
                }
            } else if t.is_directive() {
                self.directive(t)?;
            } else if t.0.starts_with('`') {
                if !self.skipping() {
                    return Err(Error::new(t.0, format!("{} is only allowed in a macro's text", t.0)));
                }
            } else if !self.skipping() {
                self.emit(t);
            }
        }
        Ok(())
    }

    /// The next token of the top frame.
    fn next_token(&mut self) -> Result<'a, Token<'a>> {
        let frame = self.frames.last_mut().unwrap();
        if let Some(base) = self.pending_base {
            frame.lexer.expect_digits(base);
        }
        frame.lexer.next_token()
    }

    fn pop_frame(&mut self) -> Result<'a, ()> {
        let n = self.frames.len();
        let frame = self.frames.pop().unwrap();
        if frame.file
            && let Some(c) = self.conds.iter().find(|c| c.frames >= n)
        {
            return Err(Error::new(c.at.0, format!("this `{} has no `endif in the same file", c.at.directive_name())));
        }
        Ok(())
    }

    fn emit(&mut self, t: Token<'a>) {
        self.track_design_elements(t);
        self.pending_base = t.base();
        if *self.states.last().unwrap() != self.state {
            self.states.push(self.state);
        }
        self.out.push(PpToken { token: t, state: (self.states.len() - 1) as u32 });
    }

    /// Counts how many design elements the output is inside (3.2). A
    /// keyword that starts one does not when it follows `extern` (a
    /// prototype), `virtual` (an interface type), or `(` or `,` (a generic
    /// interface port); `interface class` is a class.
    fn track_design_elements(&mut self, t: Token<'a>) {
        if t.is_escaped_ident() || !self.state.keywords.contains(t.0) {
            return;
        }
        let prev = self.out.last().map_or("", |p| p.token.0);
        match t.0 {
            "module" | "macromodule" | "interface" | "program" | "package" | "primitive" | "checker" | "config"
                if !matches!(prev, "extern" | "virtual" | "(" | ",") =>
            {
                self.design_depth += 1
            }
            "class" if prev == "interface" => self.design_depth = self.design_depth.saturating_sub(1),
            "endmodule" | "endinterface" | "endprogram" | "endpackage" | "endprimitive" | "endchecker"
            | "endconfig" => self.design_depth = self.design_depth.saturating_sub(1),
            _ => {}
        }
    }

    // ---- reading a directive's arguments -------------------------------

    /// Whether the rest of frame `i`'s line holds only white space and
    /// comments. In a macro's text the line is the whole text.
    fn at_line_end(&self, i: usize) -> bool {
        let frame = &self.frames[i];
        let text = frame.lexer.text();
        let b = text.as_bytes();
        let mut p = frame.lexer.pos();
        while p < b.len() {
            match b[p] {
                b'\n' if frame.file => return true,
                b' ' | b'\t' | b'\r' | b'\n' => p += 1,
                b'\\' if text[p + 1..].starts_with('\n') || text[p + 1..].starts_with("\r\n") => p += 2,
                b'/' if text[p..].starts_with("//") => return frame.file || !text[p..].contains('\n'),
                b'/' if text[p..].starts_with("/*") => {
                    let Some(end) = text[p + 2..].find("*/") else { return true };
                    if frame.file && text[p..p + 2 + end].contains('\n') {
                        return true;
                    }
                    p += end + 4;
                }
                _ => return false,
            }
        }
        true
    }

    /// The next token on the directive's line in frame `dir`, without
    /// expanding macros.
    fn raw_arg(&mut self, dir: usize) -> Result<'a, Option<Token<'a>>> {
        if self.frames.len() != dir + 1 || self.at_line_end(dir) {
            return Ok(None);
        }
        let t = self.frames[dir].lexer.next_token()?;
        Ok((!t.is_eof()).then_some(t))
    }

    /// The next token on the directive's line in frame `dir`, expanding
    /// macros (22.2).
    fn arg(&mut self, dir: usize) -> Result<'a, Option<Token<'a>>> {
        loop {
            let top = self.frames.len() - 1;
            if top < dir || top == dir && self.at_line_end(dir) {
                return Ok(None);
            }
            let t = self.frames[top].lexer.next_token()?;
            if t.is_eof() {
                if top == dir {
                    return Ok(None);
                }
                self.frames.pop();
            } else if t.is_line_continuation() {
            } else if t.is_directive() && self.is_macro(t.directive_name()) {
                self.expand(t)?;
            } else {
                return Ok(Some(t));
            }
        }
    }

    /// An argument that must be there.
    fn need_arg(&mut self, dir: usize, d: Token<'a>, what: &str) -> Result<'a, Token<'a>> {
        self.arg(dir)?.ok_or_else(|| Error::new(d.0, format!("expected {what} after {}", d.0)))
    }

    /// Fails unless nothing but white space and comments follows on the
    /// directive's line.
    fn line_end(&mut self, dir: usize, d: Token<'a>) -> Result<'a, ()> {
        match self.arg(dir)? {
            None => Ok(()),
            Some(t) => Err(Error::new(t.0, format!("unexpected `{}` after {}", t.0, d.0))),
        }
    }

    /// Skips the rest of a directive's line.
    fn skip_line(&mut self, dir: usize) -> Result<'a, ()> {
        while self.arg(dir)?.is_some() {}
        Ok(())
    }

    fn is_macro(&self, name: &str) -> bool {
        self.macros.contains_key(name) || name == "__FILE__" || name == "__LINE__"
    }

    // ---- directives ----------------------------------------------------

    fn directive(&mut self, d: Token<'a>) -> Result<'a, ()> {
        let dir = self.frames.len() - 1;
        let name = d.directive_name();
        match name {
            "ifdef" | "ifndef" | "elsif" | "else" | "endif" => return self.conditional(d, dir),
            // A macro's text hides any directives in it, even in ignored
            // text (22.6).
            "define" => return self.define(d, dir, !self.skipping()),
            _ if self.skipping() => return Ok(()),
            _ => {}
        }
        match name {
            "undef" => {
                let t = self.raw_arg(dir)?.filter(|t| t.is_ident());
                let t = t.ok_or_else(|| Error::new(d.0, "expected a macro name after `undef"))?;
                self.macros.remove(t.ident_name());
                Ok(())
            }
            "undefineall" => {
                self.macros.clear();
                Ok(())
            }
            "include" => self.include(d, dir),
            "resetall" => {
                self.outside_design_element(d)?;
                self.state = State { keywords: self.state.keywords, ..State::default() };
                Ok(())
            }
            "timescale" => {
                self.outside_design_element(d)?;
                let ts = self.timescale(d, dir)?;
                self.state.timescale = Some(ts);
                Ok(())
            }
            "default_nettype" => {
                self.outside_design_element(d)?;
                let t = self.need_arg(dir, d, "a net type")?;
                self.state.default_nettype = NetType::from_name(t.0)
                    .ok_or_else(|| Error::new(t.0, format!("`{}` is not a net type for `default_nettype", t.0)))?;
                Ok(())
            }
            "unconnected_drive" => {
                self.outside_design_element(d)?;
                let t = self.need_arg(dir, d, "pull0 or pull1")?;
                self.state.unconnected_drive = Some(match t.0 {
                    "pull1" => true,
                    "pull0" => false,
                    _ => return Err(Error::new(t.0, "expected pull0 or pull1")),
                });
                Ok(())
            }
            "nounconnected_drive" => {
                self.outside_design_element(d)?;
                self.state.unconnected_drive = None;
                Ok(())
            }
            "celldefine" => {
                self.state.celldefine = true;
                Ok(())
            }
            "endcelldefine" => {
                self.state.celldefine = false;
                Ok(())
            }
            "pragma" => {
                // Pragmas this implementation does not know have no effect
                // (22.11); none are known yet.
                let t = self.need_arg(dir, d, "a pragma name")?;
                if !t.is_ident() || t.is_escaped_ident() {
                    return Err(Error::new(t.0, "a pragma name must be a simple identifier"));
                }
                self.skip_line(dir)
            }
            "line" => self.line(d, dir),
            "begin_keywords" => {
                self.outside_design_element(d)?;
                let t = self.need_arg(dir, d, "a version string")?;
                let k = t
                    .is_string()
                    .then(|| Keywords::from_specifier(&t.0[1..t.0.len() - 1]))
                    .flatten()
                    .ok_or_else(|| Error::new(t.0, "unknown `begin_keywords version"))?;
                self.keyword_stack.push(self.state.keywords);
                self.state.keywords = k;
                Ok(())
            }
            "end_keywords" => {
                self.outside_design_element(d)?;
                self.state.keywords =
                    self.keyword_stack.pop().ok_or_else(|| Error::new(d.0, "`end_keywords without `begin_keywords"))?;
                Ok(())
            }
            // Annex E: informative, accepted and ignored.
            "default_decay_time" | "default_trireg_strength" => {
                self.need_arg(dir, d, "a value")?;
                self.skip_line(dir)
            }
            "delay_mode_distributed" | "delay_mode_path" | "delay_mode_unit" | "delay_mode_zero" => Ok(()),
            _ if self.is_macro(name) => self.expand(d),
            _ => Err(Error::new(d.0, format!("`{name} is not a defined macro or a compiler directive"))),
        }
    }

    fn outside_design_element(&self, d: Token<'a>) -> Result<'a, ()> {
        if self.design_depth > 0 {
            return Err(Error::new(d.0, format!("{} cannot appear inside a design element", d.0)));
        }
        Ok(())
    }

    /// `` `ifdef ``, `` `ifndef ``, `` `elsif ``, `` `else `` and `` `endif ``
    /// (22.6).
    fn conditional(&mut self, d: Token<'a>, dir: usize) -> Result<'a, ()> {
        let name = d.directive_name();
        if let "ifdef" | "ifndef" = name {
            let (active, done) = if self.skipping() {
                (false, true)
            } else {
                let v = self.condition(d, dir)? != (name == "ifndef");
                (v, v)
            };
            self.conds.push(Cond { at: d, active, done, seen_else: false, frames: self.frames.len() });
            return Ok(());
        }
        let outer_active = self.conds.len() < 2 || self.conds[self.conds.len() - 2].active;
        let Some(c) = self.conds.last_mut() else {
            return Err(Error::new(d.0, format!("{} without `ifdef or `ifndef", d.0)));
        };
        if name != "endif" && c.seen_else {
            return Err(Error::new(d.0, format!("{} after `else", d.0)));
        }
        match name {
            "elsif" => {
                if c.done || !outer_active {
                    c.active = false;
                } else {
                    let v = self.condition(d, dir)?;
                    let c = self.conds.last_mut().unwrap();
                    c.active = v;
                    c.done = v;
                }
            }
            "else" => {
                c.active = !c.done && outer_active;
                c.done = true;
                c.seen_else = true;
            }
            _ => {
                self.conds.pop();
            }
        }
        Ok(())
    }

    /// The condition after `` `ifdef `` or `` `elsif ``: a macro name, or an
    /// expression of names in parentheses.
    fn condition(&mut self, d: Token<'a>, dir: usize) -> Result<'a, bool> {
        let t = self.raw_arg(dir)?.ok_or_else(|| Error::new(d.0, format!("expected a macro name after {}", d.0)))?;
        if t.is("(") {
            let mut c = CondParser { pp: self, dir, d, peeked: None };
            let v = c.implication()?;
            c.expect(")")?;
            return Ok(v);
        }
        if !t.is_ident() {
            return Err(Error::new(t.0, format!("expected a macro name after {}", d.0)));
        }
        Ok(self.macros.contains_key(t.ident_name()))
    }

    /// `` `define `` (22.5.1). When `record` is false the definition is in
    /// ignored text and is only read past.
    fn define(&mut self, d: Token<'a>, dir: usize, record: bool) -> Result<'a, ()> {
        let frame = &mut self.frames[dir];
        let text = frame.lexer.text();
        // The name is on the same line.
        let mut p = frame.lexer.pos();
        while text[p..].starts_with([' ', '\t']) {
            p += 1;
        }
        if p == text.len() || text[p..].starts_with(['\n', '\r']) {
            return Err(Error::new(d.0, "expected a macro name after `define"));
        }
        frame.lexer.set_pos(p);
        let name = frame.lexer.next_token()?;
        if !name.is_ident() {
            return Err(Error::new(name.0, "expected a macro name after `define"));
        }
        if DIRECTIVES.contains(&name.ident_name()) {
            return Err(Error::new(
                name.0,
                format!("`{}` is a compiler directive, so it cannot be a macro name", name.0),
            ));
        }
        // Formal arguments: `(` straight after a simple name, or after the
        // one space that ends an escaped name.
        let mut p = frame.lexer.pos();
        if name.is_escaped_ident() && text[p..].starts_with([' ', '\t']) && text[p + 1..].starts_with('(') {
            p += 1;
        }
        let formals = if text[p..].starts_with('(') {
            frame.lexer.set_pos(p + 1);
            Some(self.formals(dir)?)
        } else {
            None
        };
        let frame = &mut self.frames[dir];
        let start = frame.lexer.pos();
        let end = body_end(text, start)?;
        frame.lexer.set_pos(end);
        let body = text[start..end].trim();
        if record {
            self.macros.insert(name.ident_name(), Rc::new(Macro { formals, body }));
        }
        Ok(())
    }

    /// The formal arguments of a definition, after the `(`.
    fn formals(&mut self, dir: usize) -> Result<'a, Vec<Formal<'a>>> {
        let mut formals = Vec::new();
        loop {
            let lexer = &mut self.frames[dir].lexer;
            let t = lexer.next_token()?;
            if t.is(")") && formals.is_empty() {
                return Ok(formals);
            }
            if !t.is_ident() || t.is_escaped_ident() {
                return Err(Error::new(t.0, "expected the name of a formal argument"));
            }
            let mut sep = lexer.next_token()?;
            let mut default = None;
            if sep.is("=") {
                let (text, end) = scan_arg(lexer)?;
                default = Some(text);
                sep = end;
            }
            formals.push(Formal { name: t.0, default });
            match sep.0 {
                "," => {}
                ")" => return Ok(formals),
                _ => return Err(Error::new(sep.0, "expected `,` or `)` in the formal arguments")),
            }
        }
    }

    /// `` `include "file" `` or `` `include <file> `` (22.4).
    fn include(&mut self, d: Token<'a>, dir: usize) -> Result<'a, ()> {
        let t = self.need_arg(dir, d, "a file name")?;
        let (name, system) = if t.is_string() && !t.is_triple_string() {
            (&t.0[1..t.0.len() - 1], false)
        } else if t.is("<") {
            let src = self.frames.last().unwrap().src;
            let mut end = t;
            while !end.is(">") {
                end = self.arg(dir)?.ok_or_else(|| Error::new(t.0, "expected `>` after the file name"))?;
                if !src.contains(end.0) {
                    return Err(Error::new(end.0, "the file name in `<...>` must be written out"));
                }
            }
            (&src.text[src.offset_of(t.0) + 1..src.offset_of(end.0)], true)
        } else {
            return Err(Error::new(t.0, "expected \"file\" or <file> after `include"));
        };
        self.line_end(dir, d)?;
        if self.frames.iter().filter(|f| f.file).count() > MAX_INCLUDE_DEPTH {
            return Err(Error::new(d.0, "`include files nest too deeply; does a file include itself?"));
        }
        let path = Path::new(name);
        let mut candidates = Vec::new();
        if path.is_absolute() {
            if system {
                return Err(Error::new(t.0, "an absolute path needs the \"file\" form of `include"));
            }
            candidates.push(path.to_path_buf());
        } else if system {
            candidates.extend(self.options.system_include_dirs.iter().map(|d| d.join(path)));
        } else {
            let here = self.frames.iter().rev().find(|f| f.file).and_then(|f| f.dir.clone());
            candidates.extend(here.map(|d| d.join(path)));
            candidates.extend(self.options.include_dirs.iter().map(|d| d.join(path)));
            candidates.push(path.to_path_buf());
        }
        let Some(found) = candidates.into_iter().find(|p| p.is_file()) else {
            return Err(Error::new(t.0, format!("cannot find the include file `{name}`")));
        };
        let text = std::fs::read_to_string(&found)
            .map_err(|e| Error::new(t.0, format!("cannot read `{}`: {e}", found.display())))?;
        let src = self.map.add(Source::new(found.display().to_string(), text));
        self.push_file(src, found.parent().map(Path::to_path_buf));
        Ok(())
    }

    /// The arguments of `` `timescale ``: `1ns / 1ps` or `10 us / 100 ns`.
    fn timescale(&mut self, d: Token<'a>, dir: usize) -> Result<'a, Timescale> {
        let unit = self.time_value(d, dir)?;
        let slash = self.need_arg(dir, d, "`/`")?;
        if !slash.is("/") {
            return Err(Error::new(slash.0, "expected `/` between the time unit and precision"));
        }
        let precision = self.time_value(d, dir)?;
        if precision > unit {
            return Err(Error::new(d.0, "the time precision cannot be coarser than the time unit"));
        }
        Ok(Timescale { unit, precision })
    }

    /// `1ns`, `10 us` or `100ps`, as a power of ten of a second.
    fn time_value(&mut self, d: Token<'a>, dir: usize) -> Result<'a, i8> {
        let t = self.need_arg(dir, d, "a time value")?;
        let digits = t.0.bytes().take_while(u8::is_ascii_digit).count();
        let magnitude = match &t.0[..digits] {
            "1" => 0,
            "10" => 1,
            "100" => 2,
            _ => return Err(Error::new(t.0, "a time value must be 1, 10 or 100")),
        };
        let unit_tok = if digits < t.0.len() { &t.0[digits..] } else { self.need_arg(dir, d, "a time unit")?.0 };
        let unit = match unit_tok {
            "s" => 0,
            "ms" => -3,
            "us" => -6,
            "ns" => -9,
            "ps" => -12,
            "fs" => -15,
            _ => return Err(Error::new(unit_tok, "expected a time unit: s, ms, us, ns, ps or fs")),
        };
        Ok(unit + magnitude)
    }

    /// `` `line number "file" level `` (22.12).
    fn line(&mut self, d: Token<'a>, dir: usize) -> Result<'a, ()> {
        let number = self.need_arg(dir, d, "a line number")?;
        let n: usize = number
            .0
            .parse()
            .ok()
            .filter(|&n| n > 0)
            .ok_or_else(|| Error::new(number.0, "the line number must be a positive integer"))?;
        let file = self.need_arg(dir, d, "a file name")?;
        if !file.is_string() || file.is_triple_string() {
            return Err(Error::new(file.0, "expected the file name as a string"));
        }
        let level = self.need_arg(dir, d, "a level")?;
        if !matches!(level.0, "0" | "1" | "2") {
            return Err(Error::new(level.0, "the level must be 0, 1 or 2"));
        }
        self.line_end(dir, d)?;
        let frame = &self.frames[dir];
        let (line, _) = frame.src.line_col(d.0);
        frame.src.remap(line + 1, n, unescape_string(&file.0[1..file.0.len() - 1]));
        Ok(())
    }

    // ---- macro expansion -----------------------------------------------

    /// Expands the macro used at `u`, pushing a frame with its text.
    fn expand(&mut self, u: Token<'a>) -> Result<'a, ()> {
        if self.frames.len() > MAX_MACRO_DEPTH {
            return Err(Error::new(
                u.0,
                format!("macro expansions nest too deeply; is `{} recursive?", u.directive_name()),
            ));
        }
        let name = u.directive_name();
        let text = match name {
            "__FILE__" | "__LINE__" => {
                let frame = self.frames.iter().rev().find(|f| f.file).unwrap();
                let at = &frame.lexer.text()[frame.lexer.pos()..frame.lexer.pos()];
                let (line, _) = frame.src.line_col(at);
                let (file, line) = frame.src.logical_line(line);
                if name == "__FILE__" { quote_string(&file) } else { line.to_string() }
            }
            _ => {
                let m = Rc::clone(&self.macros[name]);
                let Some(formals) = &m.formals else {
                    if !m.body.contains("`\"") && !m.body.contains("``") {
                        // Read the text where it was defined.
                        let src = self.map.get(self.map.find(m.body).expect("macro text is in the map"));
                        self.frames.push(Frame { src, lexer: Lexer::new(m.body), file: false, dir: None });
                        return Ok(());
                    }
                    let text = self.substitute(m.body, &[], &[])?;
                    return self.push_expansion(u, text);
                };
                let actuals = self.actuals(u)?;
                if actuals.len() > formals.len().max(1) {
                    return Err(Error::new(u.0, format!("too many arguments to `{name}")));
                }
                let mut args = Vec::new();
                for (i, f) in formals.iter().enumerate() {
                    let a = actuals.get(i).copied();
                    args.push(match (a, f.default) {
                        (Some(a), _) if !a.is_empty() => a,
                        (_, Some(def)) => def,
                        (Some(a), None) => a,
                        (None, None) => {
                            return Err(Error::new(
                                u.0,
                                format!("`{name} needs an argument for `{}`, which has no default", f.name),
                            ));
                        }
                    });
                }
                let names: Vec<&str> = formals.iter().map(|f| f.name).collect();
                self.substitute(m.body, &names, &args)?
            }
        };
        self.push_expansion(u, text)
    }

    fn push_expansion(&mut self, u: Token<'a>, text: String) -> Result<'a, ()> {
        let user = self.map.find(u.0).expect("macro use is in the map");
        let offset = self.map.get(user).offset_of(u.0);
        let mut src = Source::new(u.0, text);
        src.origin = Some(Origin { macro_name: u.directive_name().to_string(), source: user, offset });
        let src = self.map.add(src);
        self.frames.push(Frame { src, lexer: Lexer::new(&src.text), file: false, dir: None });
        Ok(())
    }

    /// The actual arguments of a use of a macro with formal arguments:
    /// `(a, b)`, each trimmed of white space.
    fn actuals(&mut self, u: Token<'a>) -> Result<'a, Vec<&'a str>> {
        let lexer = &mut self.frames.last_mut().unwrap().lexer;
        let open = lexer.next_token()?;
        if !open.is("(") {
            return Err(Error::new(u.0, format!("`{} takes arguments, so its use needs `(`", u.directive_name())));
        }
        let mut args = Vec::new();
        loop {
            let (arg, end) = scan_arg(lexer)?;
            args.push(arg);
            if end.is(")") {
                return Ok(args);
            }
        }
    }

    /// The text of a macro with `args` put in place of `formals` (22.5.1):
    /// comments are dropped, `` `" `` and `` `""" `` make strings in which
    /// arguments are still replaced, `` `\`" `` becomes `\"`, and ` `` `
    /// joins tokens.
    fn substitute(&mut self, body: &'a str, formals: &[&str], args: &[&'a str]) -> Result<'a, String> {
        let b = body.as_bytes();
        let mut out = String::with_capacity(body.len());
        let mut quote = Quote::None;
        let mut p = 0;
        while p < b.len() {
            let rest = &body[p..];
            let c = b[p];
            if rest.starts_with("`\"\"\"") {
                out += "\"\"\"";
                quote = if quote == Quote::Triple { Quote::None } else { Quote::Triple };
                p += 4;
            } else if rest.starts_with("`\\`\"") {
                out += "\\\"";
                p += 4;
            } else if rest.starts_with("`\"") {
                out.push('"');
                quote = if quote == Quote::Single { Quote::None } else { Quote::Single };
                p += 2;
            } else if rest.starts_with("``") {
                p += 2;
            } else if let Some(n) = continuation(rest) {
                // The newline stays, except inside `" ... `".
                if quote != Quote::Single {
                    out.push('\n');
                }
                p += n;
            } else if c == b'`' && quote != Quote::None {
                // A macro used inside `" ... `" is expanded into the string.
                let end = use_end(body, p, |n| self.macros.get(n).is_some_and(|m| m.formals.is_some()));
                let name = Token(&body[p..]).directive_name();
                let use_text = &body[p..end];
                if self.is_macro(name.split(|c: char| !c.is_ascii() || !is_ident_char(c as u8)).next().unwrap_or("")) {
                    let inner = self.substitute(use_text, formals, args)?;
                    out += &self.expand_to_string(use_text, inner)?;
                } else {
                    out += use_text;
                }
                p = end;
            } else if quote == Quote::None && rest.starts_with("//") {
                // To the end of the line, or to a backslash ending it.
                let eol = rest.find('\n').unwrap_or(rest.len());
                let line = rest[..eol].trim_end_matches('\r');
                p += if line.ends_with('\\') { line.len() - 1 } else { eol };
            } else if quote == Quote::None && rest.starts_with("/*") {
                let end = rest[2..].find("*/").map_or(rest.len(), |e| e + 4);
                out.push(' ');
                p += end;
            } else if quote == Quote::None && rest.starts_with("\"\"\"") {
                let end = string_end(body, p, true)?;
                out += &body[p..end];
                p = end;
            } else if c == b'"' {
                // A string literal, even inside `" or `""" (22.5.1): copied,
                // without its line continuations, and nothing in it replaced.
                let end = string_end(body, p, false)?;
                let mut s = &body[p..end];
                while let Some(i) = s.find("\\\n").or_else(|| s.find("\\\r\n")) {
                    out += &s[..i];
                    s = &s[i + continuation(&s[i..]).unwrap()..];
                }
                out += s;
                p = end;
            } else if c == b'\\' && quote == Quote::None {
                let end = body[p..].find(|c: char| c.is_ascii_whitespace()).map_or(b.len(), |e| p + e);
                out += &body[p..end];
                p = end;
            } else if c == b'$' || c.is_ascii_digit() {
                let end = p + 1 + body[p + 1..].bytes().take_while(|&c| is_ident_char(c)).count();
                out += &body[p..end];
                p = end;
            } else if c.is_ascii_alphabetic() || c == b'_' {
                let end = p + body[p..].bytes().take_while(|&c| is_ident_char(c)).count();
                let word = &body[p..end];
                match formals.iter().position(|f| *f == word) {
                    Some(i) if quote != Quote::None && args[i].contains('`') => {
                        out += &self.expand_to_string(args[i], args[i].to_string())?;
                    }
                    Some(i) => out += args[i],
                    None => out += word,
                }
                p = end;
            } else {
                let len = rest.chars().next().unwrap().len_utf8();
                out += &rest[..len];
                p += len;
            }
        }
        Ok(out)
    }

    /// Fully expands `text` (macro uses that appear inside a `` `" ``
    /// string) and returns the tokens as text.
    fn expand_to_string(&mut self, at: &'a str, text: String) -> Result<'a, String> {
        let saved = std::mem::take(&mut self.out);
        let base = self.frames.len();
        self.push_expansion(Token(at), text)?;
        let result = self.run(base);
        let tokens = std::mem::replace(&mut self.out, saved);
        result?;
        let out = Output { tokens, states: Vec::new() };
        Ok(out.text(self.map).trim_end().to_string())
    }
}

/// Reads an `ifdef` expression: names joined by `!`, `&&`, `||`, `->` and
/// `<->`, with parentheses. `!` binds tightest, then `&&`, then `||`, then
/// the two implications, which group to the right (11.3.2).
struct CondParser<'p, 'a, 'o> {
    pp: &'p mut Preprocessor<'a, 'o>,
    dir: usize,
    d: Token<'a>,
    peeked: Option<Token<'a>>,
}

impl<'a> CondParser<'_, 'a, '_> {
    fn peek(&mut self) -> Result<'a, Option<Token<'a>>> {
        if self.peeked.is_none() {
            self.peeked = self.pp.raw_arg(self.dir)?;
        }
        Ok(self.peeked)
    }

    fn next(&mut self) -> Result<'a, Token<'a>> {
        let t = self.peek()?.ok_or_else(|| Error::new(self.d.0, "the condition is not finished on this line"))?;
        self.peeked = None;
        Ok(t)
    }

    fn expect(&mut self, s: &str) -> Result<'a, ()> {
        let t = self.next()?;
        if !t.is(s) {
            return Err(Error::new(t.0, format!("expected `{s}`")));
        }
        Ok(())
    }

    fn implication(&mut self) -> Result<'a, bool> {
        let a = self.or()?;
        match self.peek()?.map(|t| t.0) {
            Some("->") => {
                self.next()?;
                let b = self.implication()?;
                Ok(!a || b)
            }
            Some("<->") => {
                self.next()?;
                Ok(a == self.implication()?)
            }
            _ => Ok(a),
        }
    }

    fn or(&mut self) -> Result<'a, bool> {
        let mut v = self.and()?;
        while self.peek()?.is_some_and(|t| t.is("||")) {
            self.next()?;
            v |= self.and()?;
        }
        Ok(v)
    }

    fn and(&mut self) -> Result<'a, bool> {
        let mut v = self.not()?;
        while self.peek()?.is_some_and(|t| t.is("&&")) {
            self.next()?;
            v &= self.not()?;
        }
        Ok(v)
    }

    fn not(&mut self) -> Result<'a, bool> {
        let t = self.next()?;
        if t.is("!") {
            Ok(!self.not()?)
        } else if t.is("(") {
            let v = self.implication()?;
            self.expect(")")?;
            Ok(v)
        } else if t.is_ident() {
            Ok(self.pp.macros.contains_key(t.ident_name()))
        } else {
            Err(Error::new(t.0, "expected a macro name, `!` or `(`"))
        }
    }
}

fn is_ident_char(c: u8) -> bool {
    c.is_ascii_alphanumeric() || c == b'_' || c == b'$'
}

/// The length of a backslash-newline at the start of `s`.
fn continuation(s: &str) -> Option<usize> {
    if s.starts_with("\\\n") {
        Some(2)
    } else if s.starts_with("\\\r\n") {
        Some(3)
    } else {
        None
    }
}

/// The end of the string literal starting at byte `p` of `text`.
fn string_end(text: &str, p: usize, triple: bool) -> Result<'_, usize> {
    let b = text.as_bytes();
    let mut i = p + if triple { 3 } else { 1 };
    while i < b.len() {
        match b[i] {
            b'\\' => i += 2,
            b'"' if !triple => return Ok(i + 1),
            b'"' if text[i..].starts_with("\"\"\"") => return Ok(i + 3),
            b'\n' if !triple => break,
            _ => i += 1,
        }
    }
    Err(Error::new(&text[p..p + 1], "unterminated string"))
}

/// The end of a macro's text that starts at byte `start` of `text`: the
/// first newline not preceded by a backslash and not inside a block comment
/// or a triple-quoted string (22.5.1).
fn body_end(text: &str, start: usize) -> Result<'_, usize> {
    let b = text.as_bytes();
    let mut quote = Quote::None;
    let mut p = start;
    while p < b.len() {
        let rest = &text[p..];
        if rest.starts_with("`\"\"\"") {
            quote = if quote == Quote::Triple { Quote::None } else { Quote::Triple };
            p += 4;
        } else if rest.starts_with("`\\`\"") {
            p += 4;
        } else if rest.starts_with("`\"") && quote != Quote::Triple {
            quote = if quote == Quote::Single { Quote::None } else { Quote::Single };
            p += 2;
        } else if let Some(n) = continuation(rest) {
            p += n;
        } else if b[p] == b'\n' {
            break;
        } else if quote == Quote::None && rest.starts_with("//") {
            let eol = rest.find('\n').unwrap_or(rest.len());
            let line = rest[..eol].trim_end_matches('\r');
            p += if line.ends_with('\\') { line.len() - 1 } else { eol };
        } else if quote == Quote::None && rest.starts_with("/*") {
            let Some(end) = rest[2..].find("*/") else {
                return Err(Error::new(&rest[..2], "unterminated block comment"));
            };
            p += end + 4;
        } else if quote == Quote::None && rest.starts_with("\"\"\"") {
            p = string_end(text, p, true)?;
        } else if b[p] == b'"' {
            p = string_end(text, p, false)?;
        } else if b[p] == b'\\' && quote == Quote::None {
            p += 1 + rest[1..].find(|c: char| c.is_ascii_whitespace()).unwrap_or(rest.len() - 1);
        } else {
            p += rest.chars().next().unwrap().len_utf8();
        }
    }
    Ok(p)
}

/// The end of a macro use starting at byte `p` of `text`: its name, and its
/// parenthesised arguments if `takes_args` says the macro has them.
fn use_end(text: &str, p: usize, takes_args: impl Fn(&str) -> bool) -> usize {
    let b = text.as_bytes();
    let mut i = p + 1;
    if b.get(i) == Some(&b'\\') {
        i += text[i..].find(|c: char| c.is_ascii_whitespace()).unwrap_or(text.len() - i);
    } else {
        i += text[i..].bytes().take_while(|&c| is_ident_char(c)).count();
    }
    let name = Token(&text[p..i]).directive_name();
    if !takes_args(name) {
        return i;
    }
    let mut j = i;
    while j < b.len() && b[j].is_ascii_whitespace() {
        j += 1;
    }
    if b.get(j) != Some(&b'(') {
        return i;
    }
    let mut depth = 0;
    while j < b.len() {
        match b[j] {
            b'(' | b'[' | b'{' => depth += 1,
            b')' | b']' | b'}' => {
                depth -= 1;
                if depth == 0 {
                    return j + 1;
                }
            }
            b'"' => {
                j = string_end(text, j, false).unwrap_or(b.len());
                continue;
            }
            _ => {}
        }
        j += 1;
    }
    b.len()
}

/// One actual argument or default: the text up to a `,` or `)` that is not
/// inside parentheses, brackets, braces or a string. Returns the trimmed
/// text and the token that ended it.
fn scan_arg<'a>(lexer: &mut Lexer<'a>) -> Result<'a, (&'a str, Token<'a>)> {
    let text = lexer.text();
    let start = lexer.pos();
    let mut depth = 0usize;
    loop {
        let t = lexer.next_token()?;
        if t.is_eof() {
            return Err(Error::new(&text[start..start], "unterminated macro arguments"));
        }
        match t.0 {
            "(" | "[" | "{" | "'{" => depth += 1,
            ")" | "]" | "}" if depth > 0 => depth -= 1,
            "," | ")" if depth == 0 => {
                let end = t.0.as_ptr() as usize - text.as_ptr() as usize;
                return Ok((text[start..end].trim(), t));
            }
            _ => {}
        }
    }
}

/// `s` as a string literal.
fn quote_string(s: &str) -> String {
    let mut out = String::from("\"");
    for c in s.chars() {
        if c == '"' || c == '\\' {
            out.push('\\');
        }
        out.push(c);
    }
    out.push('"');
    out
}

/// The contents of a simple string literal with `\"` and `\\` decoded.
fn unescape_string(s: &str) -> String {
    let mut out = String::new();
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c == '\\'
            && let Some(n) = chars.next()
        {
            out.push(n);
        } else {
            out.push(c);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The output tokens, the states, and each token's state.
    type Run = (Vec<String>, Vec<State>, Vec<u32>);

    fn run(text: &str) -> std::result::Result<Run, String> {
        let map = SourceMap::new();
        map.add(Source::new("t.sv", text));
        let out = preprocess(&map, &[0], &Options::default()).map_err(|e| map.render(&e))?;
        let toks = out.tokens.iter().map(|t| t.token.0.to_string()).collect();
        let states = out.tokens.iter().map(|t| t.state).collect();
        Ok((toks, out.states, states))
    }

    /// The output tokens joined by spaces.
    fn pp(text: &str) -> String {
        match run(text) {
            Ok((toks, _, _)) => toks.join(" "),
            Err(e) => panic!("{e}"),
        }
    }

    /// The first line of the error message.
    fn err(text: &str) -> String {
        match run(text) {
            Ok((toks, _, _)) => panic!("expected an error, got {toks:?}"),
            Err(e) => e.lines().next().unwrap().to_string(),
        }
    }

    #[test]
    fn simple_macros() {
        assert_eq!(pp("`define W 16\nlogic [`W-1:0] x;"), "logic [ 16 - 1 : 0 ] x ;");
        assert_eq!(pp("`define W 1\n`define W 2\n`W"), "2");
        assert_eq!(pp("`define E\na `E b"), "a b");
        assert_eq!(pp("`define W 3\nW `W"), "W 3");
        assert_eq!(
            err("`define A 1\n`undef A\n`A"),
            "t.sv:3:1: error: `A is not a defined macro or a compiler directive"
        );
        assert_eq!(
            err("`define A 1\n`define B 2\n`undefineall\n`B"),
            "t.sv:4:1: error: `B is not a defined macro or a compiler directive"
        );
        assert_eq!(pp("`SV_COV_TOGGLE `SV_COV_ERROR"), "23 - 1");
    }

    #[test]
    fn text_spans_lines_and_drops_comments() {
        assert_eq!(pp("`define L a + /* x\n y */ b // c \\\n + c\nd `L"), "d a + b + c");
        assert_eq!(pp("`define L(x) x + /* x */ 1 // x\n`L(y)"), "y + 1");
        assert_eq!(pp("`define L 1 \\\n + 2\n`L"), "1 + 2");
        assert_eq!(pp("`define T \"\"\"a\nb\"\"\" c\n`T"), "\"\"\"a\nb\"\"\" c");
    }

    #[test]
    fn arguments_and_defaults() {
        let def = "`define F(a, b = 7, c = ) {a, b, c}\n";
        assert_eq!(pp(&format!("{def}`F(1, 2, 3)")), "{ 1 , 2 , 3 }");
        assert_eq!(pp(&format!("{def}`F(1)")), "{ 1 , 7 , }");
        assert_eq!(pp(&format!("{def}`F( 1 , , 3)")), "{ 1 , 7 , 3 }");
        assert_eq!(pp(&format!("{def}`F (x[1,2], {{3, 4}}, f(5, 6))")), "{ x [ 1 , 2 ] , { 3 , 4 } , f ( 5 , 6 ) }");
        assert_eq!(pp(&format!("{def}`F(\"a,b\")")), "{ \"a,b\" , 7 , }");
        assert_eq!(err(&format!("{def}`F(1, 2, 3, 4)")), "t.sv:2:1: error: too many arguments to `F");
        assert_eq!(
            err("`define G(p, q) p\n`G(1)"),
            "t.sv:2:1: error: `G needs an argument for `q`, which has no default"
        );
        assert_eq!(err("`define G(p) p\n`G;"), "t.sv:2:1: error: `G takes arguments, so its use needs `(`");
        assert_eq!(pp("`define G(p, q) p q\n`G(,)"), "");
    }

    #[test]
    fn arguments_expand_after_substitution() {
        assert_eq!(pp("`define M(a, b) a - b\n`M(`M(x, 1), `M(2, y))"), "x - 1 - 2 - y");
        assert_eq!(pp("`define ONE 1\n`define ID(v) v\n`ID(`ONE)"), "1");
        assert_eq!(err("`define R 1 + `R\n`R"), "t.sv:1:15: error: macro expansions nest too deeply; is `R recursive?");
    }

    #[test]
    fn strings_pasting_and_quotes() {
        assert_eq!(pp("`define Q(v) \"v\"\n`Q(z)"), "\"v\"");
        assert_eq!(pp("`define N 5\n\"`N\""), "\"`N\"");
        assert_eq!(pp("`define S(v) `\"v is v`\"\n`S(k)"), "\"k is k\"");
        assert_eq!(pp("`define S(v) `\"say `\\`\"v`\\`\"`\"\n`S(hi)"), "\"say \\\"hi\\\"\"");
        assert_eq!(pp("`define J(a, b) a``b\nint `J(cnt, _q);"), "int cnt_q ;");
        assert_eq!(pp("`define T(v) `\"\"\"<v \"v\">`\"\"\"\n`T(w)"), "\"\"\"<w \"v\">\"\"\"");
        assert_eq!(pp("`define N 5\n`define S `\"n=`N`\"\n`S"), "\"n=5\"");
        assert_eq!(pp("`define N 5\n`define S(v) `\"v!`\"\n`S(`N)"), "\"5!\"");
        assert_eq!(pp("`define P(name) `\"dir/name.svh`\"\n`P(cfg)"), "\"dir/cfg.svh\"");
    }

    #[test]
    fn escaped_macro_names() {
        assert_eq!(pp("`define \\m+n (a = 4) a * 2\n`\\m+n (3) `\\m+n ()"), "3 * 2 4 * 2");
        assert_eq!(pp("`define \\cpu3 9\n`cpu3"), "9");
    }

    #[test]
    fn literal_parts_from_macros() {
        let toks = run("`define SZ 12\n`define B 'h\n`define D fA\nx = `SZ`B`D;").unwrap().0;
        assert_eq!(toks, ["x", "=", "12", "'h", "fA", ";"]);
    }

    #[test]
    fn bad_definitions() {
        assert_eq!(
            err("`define include 3"),
            "t.sv:1:9: error: `include` is a compiler directive, so it cannot be a macro name"
        );
        assert_eq!(err("`define\nX 1"), "t.sv:1:1: error: expected a macro name after `define");
        assert_eq!(err("`define H \"abc\n`H def\""), "t.sv:1:11: error: unterminated string");
        assert_eq!(err("x `\" y"), "t.sv:1:3: error: `\" is only allowed in a macro's text");
    }

    #[test]
    fn conditionals() {
        let src = "`define A\n`define B\n\
            `ifdef C c `elsif B b1 `ifndef A na `else a `endif `elsif A no `else no2 `endif\n\
            `ifdef C `ifdef A no3 `else no4 `endif `else e `endif";
        assert_eq!(pp(src), "b1 a e");
        assert_eq!(pp("`define A\n`ifdef (A && !C) y `endif `ifndef (A || C) n `endif"), "y");
        assert_eq!(pp("`define A\n`ifdef (C -> D) i `endif `ifdef (A <-> C) n `else q `endif"), "i q");
        assert_eq!(pp("`define A\n`ifdef ((A || C) && !(C || D)) p `endif"), "p");
        assert_eq!(pp("`ifdef define x `else y `endif"), "y");
        // Directives hidden in comments, strings and macro text.
        assert_eq!(pp("`ifdef X // `else\n a `endif b"), "b");
        assert_eq!(pp("`ifdef X \"`else\" `define M `endif\n `endif c"), "c");
        assert_eq!(err("`ifdef X\n`define M \"half\n`endif"), "t.sv:2:11: error: unterminated string");
    }

    #[test]
    fn ignored_text_is_still_lexed() {
        assert_eq!(err("`ifdef X\n s = \"open;\n`endif"), "t.sv:2:6: error: unterminated string");
        assert_eq!(err("`ifdef X\n`else\n`else\n`endif"), "t.sv:3:1: error: `else after `else");
        assert_eq!(err("`endif"), "t.sv:1:1: error: `endif without `ifdef or `ifndef");
        assert_eq!(err("`ifdef X\n"), "t.sv:1:1: error: this `ifdef has no `endif in the same file");
    }

    #[test]
    fn file_and_line() {
        assert_eq!(pp("a\n`__LINE__\n`__FILE__"), "a 2 \"t.sv\"");
        assert_eq!(pp("`define L `__LINE__\n\n`L"), "3");
        assert_eq!(pp("`line 40 \"x\\\\y.sv\" 0\n`__LINE__ `__FILE__\n`__LINE__"), "40 \"x\\\\y.sv\" 41");
        assert_eq!(err("`line 0 \"a\" 0"), "t.sv:1:7: error: the line number must be a positive integer");
        assert_eq!(err("`line 3 \"a\" 7"), "t.sv:1:13: error: the level must be 0, 1 or 2");
        assert_eq!(
            err("`line 5 \"a.sv\" 0\n\n`nope"),
            "a.sv:6:1: error: `nope is not a defined macro or a compiler directive"
        );
    }

    #[test]
    fn settings() {
        let (toks, states, idx) =
            run("`timescale 10ns / 1 ps\n`default_nettype none\n`unconnected_drive pull0\n`celldefine\nm\n\
                 `resetall\nn\n`timescale 1 s/100ms o")
            .unwrap();
        assert_eq!(toks, ["m", "n", "o"]);
        let m = states[idx[0] as usize];
        assert_eq!(m.timescale, Some(Timescale { unit: -8, precision: -12 }));
        assert_eq!((m.default_nettype, m.unconnected_drive, m.celldefine), (NetType::None, Some(false), true));
        assert_eq!(states[idx[1] as usize], State::default());
        assert_eq!(states[idx[2] as usize].timescale, Some(Timescale { unit: 0, precision: -1 }));
        assert_eq!(
            err("`timescale 1ns / 10ns"),
            "t.sv:1:1: error: the time precision cannot be coarser than the time unit"
        );
        assert_eq!(err("`timescale 5ns / 1ns"), "t.sv:1:12: error: a time value must be 1, 10 or 100");
        assert_eq!(err("`default_nettype logic"), "t.sv:1:18: error: `logic` is not a net type for `default_nettype");
        assert_eq!(err("`unconnected_drive weak"), "t.sv:1:20: error: expected pull0 or pull1");
    }

    #[test]
    fn directives_outside_design_elements() {
        assert_eq!(
            err("module m;\n`timescale 1ns/1ns\nendmodule"),
            "t.sv:2:1: error: `timescale cannot appear inside a design element"
        );
        assert_eq!(
            err("interface i;\n`resetall\nendinterface"),
            "t.sv:2:1: error: `resetall cannot appear inside a design element"
        );
        assert_eq!(pp("module m; endmodule\n`resetall\n`default_nettype wire"), "module m ; endmodule");
        assert_eq!(pp("interface class c; endclass\n`resetall"), "interface class c ; endclass");
        assert_eq!(pp("extern module e;\n`resetall"), "extern module e ;");
    }

    #[test]
    fn begin_keywords() {
        let (toks, states, idx) = run("`begin_keywords \"1364-2005\"\nlogic\n`begin_keywords \"1800-2009\"\nlogic\n`end_keywords\nlogic\n`end_keywords\nlogic").unwrap();
        assert_eq!(toks, ["logic"; 4]);
        let kw: Vec<bool> = idx.iter().map(|&i| states[i as usize].keywords.contains("logic")).collect();
        assert_eq!(kw, [false, true, false, true]);
        assert_eq!(err("`begin_keywords \"2099\""), "t.sv:1:17: error: unknown `begin_keywords version");
        assert_eq!(err("`end_keywords"), "t.sv:1:1: error: `end_keywords without `begin_keywords");
    }

    #[test]
    fn pragmas_and_annex_e() {
        assert_eq!(
            pp("`pragma tool_x opt = (a, 2), \"s\"\n`pragma reset tool_x\n`delay_mode_zero\n`default_decay_time 5\nx"),
            "x"
        );
        assert_eq!(err("`pragma"), "t.sv:1:1: error: expected a pragma name after `pragma");
    }

    #[test]
    fn includes() {
        let dir = std::env::temp_dir().join(format!("epitome-sv-pp-{}", std::process::id()));
        std::fs::create_dir_all(dir.join("sub")).unwrap();
        std::fs::write(dir.join("sub/inner.svh"), "inner `__FILE__\n").unwrap();
        std::fs::write(dir.join("outer.svh"), "`define FROM_OUTER 1\n`include \"inner.svh\"\nouter `__LINE__\n")
            .unwrap();
        std::fs::write(
            dir.join("top.sv"),
            "`define NAME(f) `\"f`\"\n`include `NAME(outer.svh) // comment\ntop `FROM_OUTER `__LINE__\n",
        )
        .unwrap();
        let map = SourceMap::new();
        let options = Options { include_dirs: vec![dir.join("sub")], ..Options::default() };
        let out = preprocess_files(&map, &[dir.join("top.sv")], &options).unwrap();
        let toks: Vec<&str> = out.tokens.iter().map(|t| t.token.0).collect();
        let inner = format!("\"{}\"", dir.join("sub/inner.svh").display());
        assert_eq!(toks, ["inner", inner.as_str(), "outer", "3", "top", "1", "3"]);

        std::fs::write(dir.join("bad.sv"), "`include \"missing.svh\"\n").unwrap();
        let e = preprocess_files(&map, &[dir.join("bad.sv")], &options).unwrap_err();
        assert!(e.contains("bad.sv:1:10: error: cannot find the include file `missing.svh`"), "{e}");
        std::fs::write(dir.join("self.sv"), "`include \"self.sv\"\n").unwrap();
        let e = preprocess_files(&map, &[dir.join("self.sv")], &options).unwrap_err();
        assert!(e.contains("nest too deeply"), "{e}");
        std::fs::write(dir.join("trail.sv"), "`include \"outer.svh\" x\n").unwrap();
        let e = preprocess_files(&map, &[dir.join("trail.sv")], &options).unwrap_err();
        assert!(e.contains("trail.sv:1:22: error: unexpected `x` after `include"), "{e}");
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn errors_in_expansions_name_the_use() {
        let map = SourceMap::new();
        map.add(Source::new("t.sv", "`define S(v) `\"v\n\nx = `S(1);"));
        let e = preprocess(&map, &[0], &Options::default()).unwrap_err();
        assert_eq!(map.render(&e), "`S:1:1: error: unterminated string\n\"1\n^\nt.sv:3:5: note: in expansion of `S\n");
    }

    #[test]
    fn print_keeps_layout() {
        let map = SourceMap::new();
        map.add(Source::new("t.sv", "`define W 8\nmodule m;\n  logic [`W:0] x; // c\nendmodule\n"));
        let out = preprocess(&map, &[0], &Options::default()).unwrap();
        assert_eq!(out.text(&map), "module m;\n  logic [ 8 :0] x;\nendmodule\n");
    }
}
