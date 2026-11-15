//! An iCE40 bitstream: its framing, and packing and unpacking `.asc`
//! configurations with a [`Device`]'s description in place of `icepack`.
//!
//! The framing was learnt from `icepack`'s output (experiment `header` in
//! `experiment.rs`): a bitstream is an optional comment, the preamble
//! `7e aa 99 7e`, then commands. A command is one byte, its high nibble
//! the command and its low nibble how many argument bytes follow, big
//! endian. Command 6 sets the width of the next data (the argument is
//! the width less one), 7 its height in lines and 8 the line it starts
//! at, and 1 selects a bank; command 0 with argument 1 or 3 is followed by
//! (width) x (height) bits of data, most significant bit first, line by
//! line, then two zero bytes. Reading every bitstream this way consumes
//! it exactly, which is the evidence for it. Argument 1 writes
//! configuration RAM (CRAM: the tiles' bits), 3 block RAM contents (bank
//! for bank, the `.ram_data` of experiment `bram`). Command 2 carries a
//! CRC-16 of the bytes since the command `01 05` (experiment `crc`).

use super::Device;
use super::asc::{Asc, AscRam, AscTile};

/// A command of the bitstream, with its data left out.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Cmd {
    /// `op` (high nibble the command, low the argument's length) and its
    /// argument.
    Op { op: u8, arg: u32 },
    /// `01 01`: write CRAM data into the selected bank.
    Cram,
    /// `01 03`: write block RAM data into the selected bank.
    Bram,
    /// `22`: the CRC so far.
    Crc,
}

impl Cmd {
    pub fn code(&self) -> u8 {
        match self {
            Cmd::Op { op, .. } => op >> 4,
            Cmd::Cram | Cmd::Bram => 0,
            Cmd::Crc => 2,
        }
    }
}

/// What `icepack`'s options change, as experiment `header` found.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PackOptions {
    /// `-s`: no deep-sleep command to the SPI flash after configuring.
    pub nosleep: bool,
    /// `-m0`, `-m1`, `-m2`: the SPI master clock range.
    pub freq: u8,
    /// `-n`: leave the block RAMs uninitialised.
    pub no_bram: bool,
}

impl PackOptions {
    pub fn args(&self) -> Vec<String> {
        let mut a = vec![format!("-m{}", self.freq)];
        if self.nosleep {
            a.push("-s".into());
        }
        if self.no_bram {
            a.push("-n".into());
        }
        a
    }
}

/// A data block found in a bitstream: which memory, bank, first line,
/// width and height, and where its bits start (a bit offset into the
/// bytes).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Block {
    pub bram: bool,
    pub bank: usize,
    pub line0: usize,
    pub width: usize,
    pub height: usize,
    pub bit: usize,
}

/// A bitstream read into its parts.
#[derive(Clone, Debug, Default)]
pub struct Parsed {
    pub comment: Option<Vec<u8>>,
    pub preamble_at: usize,
    /// Each command, with the byte offset it starts at.
    pub cmds: Vec<(usize, Cmd)>,
    pub blocks: Vec<Block>,
}

pub const PREAMBLE: [u8; 4] = [0x7e, 0xaa, 0x99, 0x7e];

/// Reads a bitstream's framing.
pub fn parse(b: &[u8]) -> Result<Parsed, String> {
    let at = b.windows(4).position(|w| w == PREAMBLE).ok_or("no preamble")?;
    let mut p = Parsed { preamble_at: at, ..Parsed::default() };
    if at > 0 {
        p.comment = Some(b[..at].to_vec());
    }
    let (mut width, mut height, mut line0, mut bank) = (0usize, 0usize, 0usize, 0usize);
    let mut i = at + 4;
    while i < b.len() {
        let start = i;
        let op = b[i];
        let n = (op & 15) as usize;
        if i + 1 + n > b.len() {
            return Err(format!("byte {i}: command {op:02x} runs past the end"));
        }
        let arg = b[i + 1..i + 1 + n].iter().fold(0u32, |a, &x| a << 8 | x as u32);
        i += 1 + n;
        let cmd = match (op >> 4, arg) {
            (0, 1) | (0, 3) if n == 1 => {
                let bits = width * height;
                if bits % 8 != 0 || i + bits / 8 + 2 > b.len() {
                    return Err(format!("byte {start}: a data block of {width} x {height} bits does not fit"));
                }
                p.blocks.push(Block { bram: arg == 3, bank, line0, width, height, bit: i * 8 });
                i += bits / 8;
                if b[i] != 0 || b[i + 1] != 0 {
                    return Err(format!("byte {i}: a data block is not followed by two zero bytes"));
                }
                i += 2;
                if arg == 1 { Cmd::Cram } else { Cmd::Bram }
            }
            (2, _) if n == 2 => Cmd::Crc,
            (code, _) => {
                match code {
                    1 => bank = arg as usize,
                    6 => width = arg as usize + 1,
                    7 => height = arg as usize,
                    8 => line0 = arg as usize,
                    _ => {}
                }
                Cmd::Op { op, arg }
            }
        };
        p.cmds.push((start, cmd));
    }
    Ok(p)
}

/// CRC-16 with polynomial `poly`, most significant bit first, no final
/// XOR, starting from `init`.
pub fn crc16(poly: u16, init: u16, data: &[u8]) -> u16 {
    let mut c = init;
    for &byte in data {
        c ^= (byte as u16) << 8;
        for _ in 0..8 {
            c = if c & 0x8000 != 0 { c << 1 ^ poly } else { c << 1 };
        }
    }
    c
}

pub fn get_bit(b: &[u8], i: usize) -> bool {
    b[i >> 3] >> (7 - (i & 7)) & 1 != 0
}

fn set_bit(b: &mut [u8], i: usize) {
    b[i >> 3] |= 0x80 >> (i & 7);
}

/// The bits of every bank: `cram[bank][line][pos]` and likewise `bram`,
/// as flat vectors of `lines x width`.
pub struct Banks {
    pub cram: Vec<Vec<bool>>,
    pub bram: Vec<Vec<bool>>,
}

impl Device {
    /// The CRAM bits an `.asc` configuration sets, by bank.
    pub fn banks(&self, asc: &Asc) -> Result<Banks, String> {
        let f = &self.format;
        let mut cram: Vec<Vec<bool>> = f.cram_banks.iter().map(|&(l, w)| vec![false; l * w]).collect();
        let mut bram: Vec<Vec<bool>> = f.bram_banks.iter().map(|&(l, w)| vec![false; l * w]).collect();
        for t in &asc.tiles {
            let place = self.tile_at(t.x, t.y).ok_or_else(|| format!("no tile at {} {}", t.x, t.y))?;
            let layout = &self.layouts[place.layout];
            let w = f.cram_banks[place.bank].1;
            for r in 0..16 {
                for c in 0..t.width.min(layout.cols.len()) {
                    if t.rows[r] >> c & 1 != 0 {
                        cram[place.bank][(place.line0 + layout.rows[r]) * w + place.pos0 + layout.cols[c]] = true;
                    }
                }
            }
        }
        for &(bank, pos, line) in &asc.extra {
            let w = f.cram_banks.get(bank).ok_or("extra bit: no such bank")?.1;
            cram[bank][line * w + pos] = true;
        }
        for r in &asc.ram {
            let place = self.rams.iter().find(|p| p.x == r.x && p.y == r.y).ok_or("ram_data: no such block RAM")?;
            let w = f.bram_banks[place.bank].1;
            for (word, &v) in r.words.iter().enumerate() {
                for k in 0..16 {
                    if v >> k & 1 != 0 {
                        bram[place.bank][word * w + place.pos0 + self.ram_bits[k]] = true;
                    }
                }
            }
        }
        Ok(Banks { cram, bram })
    }

    /// Packs an `.asc` configuration into a bitstream, as `icepack` does.
    pub fn pack(&self, asc: &Asc, opts: &PackOptions) -> Result<Vec<u8>, String> {
        let f = &self.format;
        let banks = self.banks(asc)?;
        let mut out = Vec::new();
        if let Some(comment) = &asc.comment {
            out.extend(&f.comment_prefix);
            for line in comment {
                out.extend(line.as_bytes());
                out.push(f.comment_end);
            }
            out.extend(&f.comment_suffix);
        }
        out.extend(PREAMBLE);
        let (mut width, mut height, mut line0, mut bank) = (0usize, 0usize, 0usize, 0usize);
        let mut crc_from = out.len();
        for cmd in &f.commands {
            match *cmd {
                Cmd::Op { op, mut arg } => {
                    let code = op >> 4;
                    if code == f.freq_code {
                        arg = opts.freq as u32;
                    }
                    if code == f.nosleep_code && opts.nosleep {
                        arg ^= f.nosleep_mask;
                    }
                    match code {
                        1 => bank = arg as usize,
                        6 => width = arg as usize + 1,
                        7 => height = arg as usize,
                        8 => line0 = arg as usize,
                        _ => {}
                    }
                    out.push(op);
                    let n = (op & 15) as usize;
                    for k in (0..n).rev() {
                        out.push((arg >> (8 * k)) as u8);
                    }
                    if (op, arg) == f.crc_reset {
                        crc_from = out.len();
                    }
                }
                Cmd::Cram | Cmd::Bram => {
                    let is_bram = *cmd == Cmd::Bram;
                    if is_bram && opts.no_bram {
                        continue;
                    }
                    out.extend([0x01, if is_bram { 3 } else { 1 }]);
                    let (bits, bank_width) = if is_bram {
                        (&banks.bram[bank], f.bram_banks[bank].1)
                    } else {
                        (&banks.cram[bank], f.cram_banks[bank].1)
                    };
                    if width != bank_width {
                        return Err(format!("a data block {width} wide for a bank {bank_width} wide"));
                    }
                    let start = out.len();
                    out.resize(start + width * height / 8, 0);
                    let src = &bits[line0 * width..(line0 + height) * width];
                    for (i, &b) in src.iter().enumerate() {
                        if b {
                            set_bit(&mut out[start..], i);
                        }
                    }
                    out.extend([0, 0]);
                }
                Cmd::Crc => {
                    out.push(0x22);
                    let c = crc16(f.crc_poly, f.crc_init, &out[crc_from..]);
                    out.extend(c.to_be_bytes());
                }
            }
        }
        Ok(out)
    }

    /// Reads a bitstream back into an `.asc` configuration: every tile, all
    /// its bits, the block RAMs written, and the CRAM bits outside every
    /// tile as extra bits.
    pub fn unpack(&self, b: &[u8]) -> Result<Asc, String> {
        let f = &self.format;
        let p = parse(b)?;
        let mut cram: Vec<Vec<bool>> = f.cram_banks.iter().map(|&(l, w)| vec![false; l * w]).collect();
        let mut bram: Vec<Vec<bool>> = f.bram_banks.iter().map(|&(l, w)| vec![false; l * w]).collect();
        let mut bram_seen = vec![false; f.bram_banks.len()];
        for blk in &p.blocks {
            let dst = if blk.bram { &mut bram[blk.bank] } else { &mut cram[blk.bank] };
            for i in 0..blk.width * blk.height {
                if get_bit(b, blk.bit + i) {
                    dst[blk.line0 * blk.width + i] = true;
                }
            }
            if blk.bram {
                bram_seen[blk.bank] = true;
            }
        }
        let mut asc = Asc::new(&self.asc_device);
        if let Some(c) = p.comment {
            let body = c
                .strip_prefix(&f.comment_prefix[..])
                .and_then(|c| c.strip_suffix(&f.comment_suffix[..]))
                .ok_or("a comment not framed as expected")?;
            let mut lines: Vec<String> =
                body.split(|&x| x == f.comment_end).map(|l| String::from_utf8_lossy(l).into_owned()).collect();
            lines.pop();
            asc.comment = Some(lines);
        }
        let mut owned: Vec<Vec<bool>> = f.cram_banks.iter().map(|&(l, w)| vec![false; l * w]).collect();
        for place in &self.tiles {
            let layout = &self.layouts[place.layout];
            let w = f.cram_banks[place.bank].1;
            let mut tile = AscTile {
                kind: self.kinds[layout.kind].name.clone(),
                x: place.x,
                y: place.y,
                width: layout.cols.len(),
                rows: [0; 16],
            };
            for r in 0..16 {
                for (c, &col) in layout.cols.iter().enumerate() {
                    let i = (place.line0 + layout.rows[r]) * w + place.pos0 + col;
                    owned[place.bank][i] = true;
                    if cram[place.bank][i] {
                        tile.rows[r] |= 1 << c;
                    }
                }
            }
            asc.tiles.push(tile);
        }
        for (bank, bits) in cram.iter().enumerate() {
            let w = f.cram_banks[bank].1;
            for (i, &b) in bits.iter().enumerate() {
                if b && !owned[bank][i] {
                    asc.extra.push((bank, i % w, i / w));
                }
            }
        }
        for place in &self.rams {
            if !bram_seen[place.bank] {
                continue;
            }
            let w = f.bram_banks[place.bank].1;
            let words = (0..256)
                .map(|word| {
                    (0..16).fold(0u16, |v, k| {
                        v | (bram[place.bank][word * w + place.pos0 + self.ram_bits[k]] as u16) << k
                    })
                })
                .collect();
            asc.ram.push(AscRam { x: place.x, y: place.y, words });
        }
        Ok(asc)
    }
}
