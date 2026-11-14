//! The `.asc` text form of an iCE40 configuration, the form IceStorm's
//! `icepack` reads and `icepack -u` writes. What is known of it here was
//! learnt from `icepack` as a program (see `experiment.rs`), and from
//! what `sv/tools/ice40_asc_check.py` already relied on:
//!
//! - `.comment`, then lines of text up to the next line starting `.`;
//! - `.device NAME` (`5k` for the UP5K: the die, not the package);
//! - `.KIND_tile X Y`, then 16 rows of `0`s and `1`s, as wide as the kind's
//!   tiles: row r, column c is tile bit `B{r}[{c}]`;
//! - `.ram_data X Y`, then 16 lines of 64 hex digits: a block RAM's 4096
//!   bits, each line a 256-bit number, sixteen 16-bit words, word 0 the
//!   last four digits;
//! - `.extra_bit BANK POS LINE`: a CRAM bit addressed directly, for the
//!   bits outside every tile (experiment `extra`);
//! - `.sym` lines, which `icepack` ignores (packing with and without them
//!   gives the same bitstream).

use std::fmt::Write;

/// One tile's bits: `rows[r]` bit c is `B{r}[{c}]`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AscTile {
    pub kind: String,
    pub x: usize,
    pub y: usize,
    pub width: usize,
    pub rows: [u64; 16],
}

/// A block RAM's initial contents, 256 words of 16 bits.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AscRam {
    pub x: usize,
    pub y: usize,
    pub words: Vec<u16>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Asc {
    pub comment: Option<Vec<String>>,
    pub device: String,
    pub tiles: Vec<AscTile>,
    pub ram: Vec<AscRam>,
    /// `(bank, pos, line)`.
    pub extra: Vec<(usize, usize, usize)>,
}

impl Asc {
    pub fn new(device: &str) -> Asc {
        Asc { device: device.to_string(), ..Asc::default() }
    }

    /// Reads an `.asc` file's text. Tiles keep the width of their longest
    /// row; an unknown directive is an error.
    pub fn parse(text: &str) -> Result<Asc, String> {
        let mut asc = Asc::default();
        let lines: Vec<&str> = text.lines().collect();
        let mut i = 0;
        let num = |s: Option<&str>, at: usize| -> Result<usize, String> {
            s.and_then(|s| s.parse().ok()).ok_or_else(|| format!("line {}: a number expected", at + 1))
        };
        while i < lines.len() {
            let line = lines[i].trim();
            let at = i;
            i += 1;
            if line.is_empty() {
                continue;
            }
            let mut words = line.split_whitespace();
            let cmd = words.next().unwrap();
            if cmd == ".comment" {
                let mut body = Vec::new();
                while i < lines.len() && !lines[i].starts_with('.') {
                    body.push(lines[i].to_string());
                    i += 1;
                }
                asc.comment = Some(body);
            } else if cmd == ".device" {
                asc.device = words.next().unwrap_or("").to_string();
            } else if let Some(kind) = cmd.strip_prefix('.').and_then(|c| c.strip_suffix("_tile")) {
                let (x, y) = (num(words.next(), at)?, num(words.next(), at)?);
                let mut tile = AscTile { kind: kind.to_string(), x, y, width: 0, rows: [0; 16] };
                for r in 0..16 {
                    let Some(row) = lines.get(i).map(|l| l.trim()) else { break };
                    if row.starts_with('.') {
                        break;
                    }
                    i += 1;
                    tile.width = tile.width.max(row.len());
                    for (c, ch) in row.bytes().enumerate() {
                        if ch == b'1' {
                            tile.rows[r] |= 1 << c;
                        }
                    }
                }
                asc.tiles.push(tile);
            } else if cmd == ".ram_data" {
                let (x, y) = (num(words.next(), at)?, num(words.next(), at)?);
                let mut words16 = vec![0u16; 256];
                for l in 0..16 {
                    let Some(hex) = lines.get(i).map(|l| l.trim()) else { break };
                    if hex.starts_with('.') {
                        break;
                    }
                    i += 1;
                    let hex = hex.as_bytes();
                    for w in 0..16 {
                        // Word w is the four digits w from the right.
                        let end = hex.len().saturating_sub(4 * w);
                        let start = end.saturating_sub(4);
                        let digits = std::str::from_utf8(&hex[start..end]).unwrap_or("0");
                        words16[l * 16 + w] =
                            u16::from_str_radix(digits, 16).map_err(|_| format!("line {}: bad hex", i))?;
                    }
                }
                asc.ram.push(AscRam { x, y, words: words16 });
            } else if cmd == ".extra_bit" {
                asc.extra.push((num(words.next(), at)?, num(words.next(), at)?, num(words.next(), at)?));
            } else if cmd == ".sym" {
            } else {
                return Err(format!("line {}: unknown directive {cmd}", at + 1));
            }
        }
        Ok(asc)
    }

    pub fn to_text(&self) -> String {
        let mut s = String::new();
        if let Some(comment) = &self.comment {
            s.push_str(".comment\n");
            for line in comment {
                s.push_str(line);
                s.push('\n');
            }
        }
        writeln!(s, ".device {}", self.device).unwrap();
        for t in &self.tiles {
            writeln!(s, ".{}_tile {} {}", t.kind, t.x, t.y).unwrap();
            for row in &t.rows {
                for c in 0..t.width {
                    s.push(if row >> c & 1 != 0 { '1' } else { '0' });
                }
                s.push('\n');
            }
        }
        for r in &self.ram {
            writeln!(s, ".ram_data {} {}", r.x, r.y).unwrap();
            for l in 0..16 {
                for w in (0..16).rev() {
                    write!(s, "{:04x}", r.words[l * 16 + w]).unwrap();
                }
                s.push('\n');
            }
        }
        for (bank, pos, line) in &self.extra {
            writeln!(s, ".extra_bit {bank} {pos} {line}").unwrap();
        }
        s
    }
}
