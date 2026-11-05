//! Source text, and positions found from where a slice sits in it.
//!
//! Tokens and error spans are `&str` slices of a source's text. A slice's
//! byte offset is its pointer minus the text's pointer, so nothing stores a
//! position: line and column are worked out only when an error is reported.
//!
//! A `SourceMap` holds every file and every piece of text made later (by
//! macros), at addresses that never change, so slices of any of them live
//! as long as the map. Which source a slice belongs to is found from its
//! address.

use std::cell::RefCell;
use std::fmt;

/// One piece of source text: a file, or text made during the compilation.
pub struct Source {
    pub name: String,
    pub text: String,
}

impl Source {
    pub fn new(name: impl Into<String>, text: impl Into<String>) -> Self {
        Source { name: name.into(), text: text.into() }
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

    /// `name:line:col` for `s`.
    pub fn locate(&self, s: &str) -> Location {
        let (line, col) = line_col(&self.text, self.offset_of(s));
        Location { name: self.name.clone(), line, col }
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
    // Boxed so that a source never moves when the vector grows. Sources
    // are only added, never removed or changed, until the map is dropped.
    #[allow(clippy::vec_box)]
    sources: RefCell<Vec<Box<Source>>>,
}

impl SourceMap {
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a source and returns it, borrowed for the life of the map.
    pub fn add(&self, source: Source) -> &Source {
        let boxed = Box::new(source);
        let ptr: *const Source = &*boxed;
        self.sources.borrow_mut().push(boxed);
        // SAFETY: the box is never dropped or moved out of while the map
        // lives, and the source is never changed after it is added.
        unsafe { &*ptr }
    }

    /// The source holding the slice `s`, if any.
    pub fn find(&self, s: &str) -> Option<&Source> {
        let sources = self.sources.borrow();
        let ptr: *const Source = &**sources.iter().find(|src| src.contains(s))?;
        // SAFETY: as in `add`.
        Some(unsafe { &*ptr })
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Location {
    pub name: String,
    pub line: usize,
    pub col: usize,
}

impl fmt::Display for Location {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}:{}:{}", self.name, self.line, self.col)
    }
}

/// An error at a slice of some source.
#[derive(Clone, Debug)]
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

    /// As `render`, finding the source in a map.
    pub fn render_in(&self, map: &SourceMap) -> String {
        match map.find(self.at) {
            Some(src) => self.render(src),
            None => format!("error: {}\n", self.message),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn positions_from_pointers() {
        let map = SourceMap::new();
        let src = map.add(Source::new("a.rs", "fn main() {\n    let x = 1;\n}\n"));
        let x = &src.text[20..21];
        assert_eq!(x, "x");
        assert_eq!(src.locate(x).to_string(), "a.rs:2:9");
        assert_eq!(src.line_of(x), "    let x = 1;");
        assert!(std::ptr::eq(map.find(x).unwrap(), src));
        let e = Error::new(x, "oops").render(src);
        assert_eq!(e, "a.rs:2:9: error: oops\n    let x = 1;\n        ^\n");
    }
}
