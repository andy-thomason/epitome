//! Lattice iCE40 parts ([`PARTS`]: the UP5K of the iCESugar board, and the
//! LP1K of the iCESugar-nano), described from experiments with IceStorm's
//! `icepack` run as a black box, and from Lattice's public datasheets. Nothing here comes from
//! IceStorm's, nextpnr's or Yosys's sources, chip databases or
//! documentation. `sv/arch/README.md` has the method and what is mapped.
//!
//! - [`experiment`] runs the experiments and records what they found, as
//!   a [`Device`]; [`Device::to_record`] and [`Device::from_record`] keep
//!   it in `sv/arch/<part>/record-icepack.txt`, so that the description
//!   can be made again where `icepack` is not installed.
//! - [`describe`] turns a [`Device`] into the machine-readable description,
//!   `sv/arch/<part>/description.json`.
//! - [`bitstream`] packs and unpacks bitstreams with a [`Device`], in place
//!   of `icepack`; the tests compare the two.

pub mod asc;
pub mod bitstream;
pub mod chipdb;
pub mod describe;
pub mod experiment;
pub mod icebox;
pub mod icepack;

use bitstream::Cmd;
use std::fmt::Write;

/// A part the generator describes: its `.asc` device name, its package
/// and board, where its description goes, the flag `icebox_chipdb` takes
/// for it, and its datasheet's resources (what, how many, and what holds
/// them, to count against).
pub struct Part {
    pub asc: &'static str,
    pub name: &'static str,
    pub package: &'static str,
    pub board: &'static str,
    pub dir: &'static str,
    pub chipdb_flag: &'static str,
    pub datasheet: &'static str,
    pub resources: &'static [(&'static str, usize, &'static str)],
}

pub const PARTS: [Part; 2] = [
    Part {
        asc: "5k",
        name: "iCE40UP5K",
        package: "SG48",
        board: "iCESugar",
        dir: "ice40-up5k",
        chipdb_flag: "-5",
        datasheet: "Lattice iCE40 UltraPlus Family Data Sheet, FPGA-DS-02008, Table 1.1 and section 2",
        resources: &[
            ("LUT4 logic cells", 5280, "logic tiles x 8"),
            ("EBR blocks (4 kbit sysMEM RAM)", 30, "ramb tiles"),
            ("SPRAM blocks (256 kbit)", 4, "SPRAM cells"),
            ("DSP blocks (MAC16)", 8, "MAC16 cells"),
            ("PLL", 1, "PLL cells"),
            ("I2C hard IP", 2, "I2C cells"),
            ("SPI hard IP", 2, "SPI cells"),
            ("HFOSC (48 MHz oscillator)", 1, "HFOSC cells"),
            ("LFOSC (10 kHz oscillator)", 1, "LFOSC cells"),
            ("RGB LED driver", 1, "RGBA_DRV cells"),
            ("I/O on the 48-pin QFN (SG48)", 39, "sg48 pins"),
        ],
    },
    Part {
        asc: "1k",
        name: "iCE40LP1K",
        package: "CM36",
        board: "iCESugar-nano",
        dir: "ice40-lp1k",
        chipdb_flag: "",
        datasheet: "Lattice iCE40 LP/HX Family Data Sheet, FPGA-DS-02029, Table 1.1 and section 2",
        resources: &[("LUT4 logic cells", 1280, "logic tiles x 8"), ("RAM4K blocks (4 kbit)", 16, "ramb tiles")],
    },
];

pub fn part(asc: &str) -> Option<&'static Part> {
    PARTS.iter().find(|p| p.asc == asc)
}

/// A kind of tile, as the `.asc` names it, and how many bits wide its
/// tiles are (all are 16 rows).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Kind {
    pub name: String,
    pub width: usize,
}

/// Where a tile's bits are in the CRAM: bit `B{r}[{c}]` of the tile at
/// `(x, y)` is in bank `bank`, on line `line0 + rows[r]`, at position
/// `pos0 + cols[c]` of the line, with `rows` and `cols` from its layout.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TilePlace {
    pub x: usize,
    pub y: usize,
    pub kind: usize,
    pub bank: usize,
    pub line0: usize,
    pub pos0: usize,
    pub layout: usize,
}

/// How the bits of a kind of tile are arranged in one bank (experiment
/// `cram`). Tiles of a kind share a few layouts.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Layout {
    pub kind: usize,
    pub bank: usize,
    pub rows: [usize; 16],
    pub cols: Vec<usize>,
}

/// A block RAM's contents in the bitstream: word w, bit k of the RAM whose
/// `.ram_data` is at `(x, y)` is in block RAM bank `bank`, line `w`,
/// position `pos0 + ram_bits[k]`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RamPlace {
    pub x: usize,
    pub y: usize,
    pub bank: usize,
    pub pos0: usize,
}

/// The bitstream's format beyond the tiles (experiments `header`, `crc`,
/// `comment` and `options`).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Format {
    pub comment_prefix: Vec<u8>,
    /// The byte after each line of the comment.
    pub comment_end: u8,
    pub comment_suffix: Vec<u8>,
    /// The commands `icepack` writes with its default options.
    pub commands: Vec<Cmd>,
    pub crc_poly: u16,
    pub crc_init: u16,
    /// The command after which the CRC starts.
    pub crc_reset: (u8, u32),
    /// The command whose argument `-m` sets.
    pub freq_code: u8,
    /// The command whose argument `-s` changes, and the bits it flips.
    pub nosleep_code: u8,
    pub nosleep_mask: u32,
    /// Each bank's lines and bits per line.
    pub cram_banks: Vec<(usize, usize)>,
    pub bram_banks: Vec<(usize, usize)>,
}

/// What the experiments found.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Device {
    /// The `icepack` the experiments ran (its IceStorm commit).
    pub icepack: String,
    /// The `.device` name.
    pub asc_device: String,
    pub width: usize,
    pub height: usize,
    pub kinds: Vec<Kind>,
    /// In the order `icepack -u` lists them.
    pub tiles: Vec<TilePlace>,
    pub layouts: Vec<Layout>,
    pub rams: Vec<RamPlace>,
    pub ram_bits: [usize; 16],
    pub format: Format,
    /// For each CRAM bank, the bits no tile holds (`.extra_bit`s).
    pub free_bits: Vec<usize>,
    /// For random configurations (`experiment::random_config` of the
    /// seed), the length and FNV-1a hash of the bitstream `icepack` made:
    /// the packer's check where `icepack` is not installed.
    pub fingerprints: Vec<(u64, usize, u64)>,
}

/// FNV-1a, 64 bits.
pub fn fnv64(b: &[u8]) -> u64 {
    b.iter().fold(0xcbf2_9ce4_8422_2325u64, |h, &x| (h ^ x as u64).wrapping_mul(0x100_0000_01b3))
}

fn hex(b: &[u8]) -> String {
    b.iter().map(|x| format!("{x:02x}")).collect()
}

fn unhex(s: &str) -> Result<Vec<u8>, String> {
    if s == "-" {
        return Ok(Vec::new());
    }
    (0..s.len() / 2).map(|i| u8::from_str_radix(&s[2 * i..2 * i + 2], 16).map_err(|e| e.to_string())).collect()
}

fn list(v: &[usize]) -> String {
    v.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(" ")
}

impl Device {
    pub fn tile_at(&self, x: usize, y: usize) -> Option<&TilePlace> {
        self.tiles.iter().find(|t| t.x == x && t.y == y)
    }

    pub fn kind(&self, name: &str) -> Option<usize> {
        self.kinds.iter().position(|k| k.name == name)
    }

    /// The record: one fact per line, `#` comments.
    pub fn to_record(&self) -> String {
        let mut s = String::new();
        let f = &self.format;
        s.push_str("# What sv/src/arch/ice40/experiment.rs found by running icepack; regenerate with\n");
        s.push_str("# `cargo run --release -p epitome-sv --example ice40_arch -- --rerun`.\n");
        writeln!(s, "icepack {}", self.icepack).unwrap();
        writeln!(s, "device {} {} {}", self.asc_device, self.width, self.height).unwrap();
        for k in &self.kinds {
            writeln!(s, "kind {} {}", k.name, k.width).unwrap();
        }
        for (b, &(l, w)) in f.cram_banks.iter().enumerate() {
            writeln!(s, "cram_bank {b} {l} {w} free {}", self.free_bits[b]).unwrap();
        }
        for (b, &(l, w)) in f.bram_banks.iter().enumerate() {
            writeln!(s, "bram_bank {b} {l} {w}").unwrap();
        }
        writeln!(s, "comment {} {:02x} {}", hex(&f.comment_prefix), f.comment_end, hex(&f.comment_suffix)).unwrap();
        for c in &f.commands {
            match c {
                Cmd::Op { op, arg } => writeln!(s, "cmd {op:02x} {arg:x}").unwrap(),
                Cmd::Cram => s.push_str("cmd cram\n"),
                Cmd::Bram => s.push_str("cmd bram\n"),
                Cmd::Crc => s.push_str("cmd crc\n"),
            }
        }
        writeln!(s, "crc {:04x} {:04x} {:02x} {:x}", f.crc_poly, f.crc_init, f.crc_reset.0, f.crc_reset.1).unwrap();
        writeln!(s, "option freq {:x}", f.freq_code).unwrap();
        writeln!(s, "option nosleep {:x} {:x}", f.nosleep_code, f.nosleep_mask).unwrap();
        for l in &self.layouts {
            writeln!(s, "layout {} {} rows {} cols {}", self.kinds[l.kind].name, l.bank, list(&l.rows), list(&l.cols))
                .unwrap();
        }
        for t in &self.tiles {
            writeln!(s, "tile {} {} {} {} {} {} {}", self.kinds[t.kind].name, t.x, t.y, t.bank, t.line0, t.pos0, t.layout).unwrap();
        }
        writeln!(s, "ram_bits {}", list(&self.ram_bits)).unwrap();
        for r in &self.rams {
            writeln!(s, "ram {} {} {} {}", r.x, r.y, r.bank, r.pos0).unwrap();
        }
        for (seed, len, h) in &self.fingerprints {
            writeln!(s, "fingerprint {seed} {len} {h:016x}").unwrap();
        }
        s
    }

    pub fn from_record(text: &str) -> Result<Device, String> {
        let mut d = Device::default();
        for (n, line) in text.lines().enumerate() {
            let err = |what: &str| format!("record line {}: {what}", n + 1);
            let w: Vec<&str> = line.split_whitespace().collect();
            if w.is_empty() || w[0].starts_with('#') {
                continue;
            }
            let num = |i: usize| -> Result<usize, String> {
                w.get(i).and_then(|s| s.parse().ok()).ok_or_else(|| err("a number expected"))
            };
            let hexnum = |i: usize| -> Result<u32, String> {
                w.get(i).and_then(|s| u32::from_str_radix(s, 16).ok()).ok_or_else(|| err("a hex number expected"))
            };
            let nums = |from: usize, to: usize| -> Result<Vec<usize>, String> { (from..to).map(num).collect() };
            match w[0] {
                "icepack" => d.icepack = w.get(1).unwrap_or(&"").to_string(),
                "device" => (d.asc_device, d.width, d.height) = (w[1].to_string(), num(2)?, num(3)?),
                "kind" => d.kinds.push(Kind { name: w[1].to_string(), width: num(2)? }),
                "cram_bank" => {
                    d.format.cram_banks.push((num(2)?, num(3)?));
                    d.free_bits.push(num(5)?);
                }
                "bram_bank" => d.format.bram_banks.push((num(2)?, num(3)?)),
                "comment" => {
                    d.format.comment_prefix = unhex(w[1])?;
                    d.format.comment_end = hexnum(2)? as u8;
                    d.format.comment_suffix = unhex(w[3])?;
                }
                "cmd" => d.format.commands.push(match w[1] {
                    "cram" => Cmd::Cram,
                    "bram" => Cmd::Bram,
                    "crc" => Cmd::Crc,
                    _ => Cmd::Op { op: hexnum(1)? as u8, arg: hexnum(2)? },
                }),
                "crc" => {
                    d.format.crc_poly = hexnum(1)? as u16;
                    d.format.crc_init = hexnum(2)? as u16;
                    d.format.crc_reset = (hexnum(3)? as u8, hexnum(4)?);
                }
                "option" if w[1] == "freq" => d.format.freq_code = hexnum(2)? as u8,
                "option" if w[1] == "nosleep" => {
                    d.format.nosleep_code = hexnum(2)? as u8;
                    d.format.nosleep_mask = hexnum(3)?;
                }
                "layout" => {
                    let kind = d.kind(w[1]).ok_or_else(|| err("unknown kind"))?;
                    let cols_at = w.iter().position(|&x| x == "cols").ok_or_else(|| err("no cols"))?;
                    let rows = nums(4, cols_at)?;
                    d.layouts.push(Layout {
                        kind,
                        bank: num(2)?,
                        rows: rows.try_into().map_err(|_| err("16 rows expected"))?,
                        cols: nums(cols_at + 1, w.len())?,
                    });
                }
                "tile" => {
                    let kind = d.kind(w[1]).ok_or_else(|| err("unknown kind"))?;
                    let (bank, layout) = (num(4)?, num(7)?);
                    if d.layouts.get(layout).is_none_or(|l| (l.kind, l.bank) != (kind, bank)) {
                        return Err(err("the tile's layout is for another kind or bank"));
                    }
                    d.tiles.push(TilePlace {
                        x: num(2)?,
                        y: num(3)?,
                        kind,
                        bank,
                        line0: num(5)?,
                        pos0: num(6)?,
                        layout,
                    });
                }
                "ram_bits" => d.ram_bits = nums(1, 17)?.try_into().unwrap(),
                "ram" => d.rams.push(RamPlace { x: num(1)?, y: num(2)?, bank: num(3)?, pos0: num(4)? }),
                "fingerprint" => d.fingerprints.push((
                    num(1)? as u64,
                    num(2)?,
                    w.get(3).and_then(|s| u64::from_str_radix(s, 16).ok()).ok_or_else(|| err("a hash expected"))?,
                )),
                _ => return Err(err("unknown fact")),
            }
        }
        Ok(d)
    }
}
