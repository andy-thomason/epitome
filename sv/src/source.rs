//! Source text, and positions found from where a slice sits in it.
//!
//! Tokens and error spans are `&str` slices of a source's text. A slice's
//! byte offset is its pointer minus the text's pointer, so nothing needs
//! to store a position: line and column are worked out only when an error
//! is reported.
//!
//! A `SourceMap` holds every file and every piece of text the preprocessor
//! makes, at addresses that never change, so slices of any of them live as
//! long as the map. Which source a slice belongs to is found from its
//! address.

use std::cell::RefCell;
use std::fmt;

/// One piece of source text: a file, or text the preprocessor made.
pub struct Source {
    pub name: String,
    pub text: String,
    /// For text made by expanding a macro: the macro's name and the slice of
    /// another source where it was used.
    pub origin: Option<Origin>,
    /// `` `line `` directives: from a physical line on, the line number and
    /// file name to report instead (22.12).
    remaps: RefCell<Vec<Remap>>,
}

/// Where a macro expansion came from.
pub struct Origin {
    pub macro_name: String,
    /// The source holding the use, as an index into the `SourceMap`.
    pub source: usize,
    /// The byte offset of the use in that source.
    pub offset: usize,
}

struct Remap {
    from_line: usize,
    line: usize,
    name: String,
}

impl Source {
    pub fn new(name: impl Into<String>, text: impl Into<String>) -> Self {
        Source { name: name.into(), text: text.into(), origin: None, remaps: RefCell::new(Vec::new()) }
    }

    /// Whether `s` is a slice of this source's text. The empty slice at
    /// the end counts.
    pub fn contains(&self, s: &str) -> bool {
        let start = self.text.as_ptr() as usize;
        let p = s.as_ptr() as usize;
        p >= start && p + s.len() <= start + self.text.len()
    }

    /// The byte offset of `s` within the text.
    pub fn offset_of(&self, s: &str) -> usize {
        assert!(self.contains(s), "slice is not part of {}", self.name);
        s.as_ptr() as usize - self.text.as_ptr() as usize
    }

    /// The 1-based physical line and column (in characters) where `s`
    /// starts, ignoring `` `line ``.
    pub fn line_col(&self, s: &str) -> (usize, usize) {
        line_col(&self.text, self.offset_of(s))
    }

    /// From physical line `from_line` on, report lines as `line`, `line +
    /// 1`, ... of `name`.
    pub fn remap(&self, from_line: usize, line: usize, name: impl Into<String>) {
        self.remaps.borrow_mut().push(Remap { from_line, line, name: name.into() });
    }

    /// The file name and line number to report for physical line `line`.
    pub fn logical_line(&self, line: usize) -> (String, usize) {
        let remaps = self.remaps.borrow();
        match remaps.iter().rev().find(|r| r.from_line <= line) {
            Some(r) => (r.name.clone(), r.line + (line - r.from_line)),
            None => (self.name.clone(), line),
        }
    }

    /// `name:line:col` for `s`, after any `` `line `` directives.
    pub fn locate(&self, s: &str) -> Location {
        let (line, col) = self.line_col(s);
        let (name, line) = self.logical_line(line);
        Location { name, line, col }
    }

    /// The whole line holding `s`, without its newline.
    pub fn line_of(&self, s: &str) -> &str {
        let at = self.offset_of(s);
        let start = self.text[..at].rfind('\n').map_or(0, |i| i + 1);
        let end = self.text[at..].find('\n').map_or(self.text.len(), |i| at + i);
        self.text[start..end].trim_end_matches('\r')
    }
}

/// The 1-based line and column of byte `offset` in `text`. Columns count
/// characters, not bytes.
pub fn line_col(text: &str, offset: usize) -> (usize, usize) {
    let before = &text[..offset];
    let line = before.bytes().filter(|&b| b == b'\n').count() + 1;
    let col = before[before.rfind('\n').map_or(0, |i| i + 1)..].chars().count() + 1;
    (line, col)
}

/// Every source of a compilation, kept for the life of the map.
#[derive(Default)]
pub struct SourceMap {
    // Boxed so that a source never moves when the vector grows. Sources are
    // only added, never removed or changed (except their remaps, which are
    // behind a RefCell), until the map is dropped.
    #[allow(clippy::vec_box)]
    sources: RefCell<Vec<Box<Source>>>,
}

impl SourceMap {
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a source and returns it, borrowed for as long as the map.
    pub fn add(&self, source: Source) -> &Source {
        let boxed = Box::new(source);
        let p: *const Source = &*boxed;
        self.sources.borrow_mut().push(boxed);
        // SAFETY: the box is owned by the map and is never dropped or moved
        // out while the map lives, and the source is not mutated except
        // through its RefCell.
        unsafe { &*p }
    }

    /// The source with index `id`, in the order they were added.
    pub fn get(&self, id: usize) -> &Source {
        let p: *const Source = &*self.sources.borrow()[id];
        // SAFETY: as in `add`.
        unsafe { &*p }
    }

    pub fn len(&self) -> usize {
        self.sources.borrow().len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// The index of the source that `s` is a slice of.
    pub fn find(&self, s: &str) -> Option<usize> {
        self.sources.borrow().iter().position(|src| src.contains(s))
    }

    /// `name:line:col: error: message`, the line and a caret, then a line
    /// for each macro expansion the error is inside.
    pub fn render(&self, e: &Error) -> String {
        let Some(id) = self.find(e.at) else { return format!("error: {}\n", e.message) };
        let src = self.get(id);
        let mut out = e.render(src);
        let mut origin = src.origin.as_ref();
        while let Some(o) = origin {
            let user = self.get(o.source);
            let at = &user.text[o.offset..o.offset];
            out += &format!("{}: note: in expansion of `{}\n", user.locate(at), o.macro_name);
            origin = user.origin.as_ref();
        }
        out
    }
}

/// A position for an error message.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Location {
    pub name: String,
    pub line: usize,
    pub col: usize,
}

impl fmt::Display for Location {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}:{}", self.name, self.line, self.col)
    }
}

/// An error at a slice of some source.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Error<'a> {
    pub at: &'a str,
    pub message: String,
}

impl<'a> Error<'a> {
    pub fn new(at: &'a str, message: impl Into<String>) -> Self {
        Error { at, message: message.into() }
    }

    /// `name:line:col: error: message`, then the line and a caret.
    pub fn render(&self, src: &Source) -> String {
        let loc = src.locate(self.at);
        let line = src.line_of(self.at);
        let caret: String = line.chars().take(loc.col - 1).map(|c| if c == '\t' { '\t' } else { ' ' }).collect();
        format!("{loc}: error: {}\n{line}\n{caret}^\n", self.message)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn positions_from_pointers() {
        let src = Source::new("a.sv", "module m;\n  wire w;\nendmodule\n");
        let w = &src.text[17..18];
        assert_eq!(w, "w");
        assert_eq!(src.line_col(w), (2, 8));
        assert_eq!(src.locate(w).to_string(), "a.sv:2:8");
        assert_eq!(src.line_of(w), "  wire w;");
        let eof = &src.text[src.text.len()..];
        assert_eq!(src.line_col(eof), (4, 1));
        assert!(!src.contains("w"));
    }

    #[test]
    fn render_points_at_the_column() {
        let src = Source::new("a.sv", "x\n\tfoo bar\n");
        let e = Error::new(&src.text[7..10], "bad");
        assert_eq!(e.render(&src), "a.sv:2:6: error: bad\n\tfoo bar\n\t    ^\n");
    }

    #[test]
    fn line_directives_remap() {
        let src = Source::new("a.sv", "a\nb\nc\nd\n");
        src.remap(3, 100, "orig.sv");
        assert_eq!(src.locate(&src.text[2..3]).to_string(), "a.sv:2:1");
        assert_eq!(src.locate(&src.text[6..7]).to_string(), "orig.sv:101:1");
    }

    #[test]
    fn map_finds_sources_and_expansions() {
        let map = SourceMap::new();
        let a = map.add(Source::new("a.sv", "x = `M;\n"));
        let mut text = Source::new("`M", "1 +");
        text.origin = Some(Origin { macro_name: "M".into(), source: 0, offset: 4 });
        let b = map.add(text);
        assert_eq!(map.find(&a.text[4..6]), Some(0));
        assert_eq!(map.find(&b.text[2..3]), Some(1));
        let e = Error::new(&b.text[2..3], "expected an operand");
        assert_eq!(
            map.render(&e),
            "`M:1:3: error: expected an operand\n1 +\n  ^\na.sv:1:5: note: in expansion of `M\n"
        );
    }
}
