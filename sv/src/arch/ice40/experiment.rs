//! The experiments: `.asc` configurations made to ask one question each,
//! packed by `icepack`, and the bitstreams compared. Each finding is
//! checked as it is made, and an experiment that sees anything its
//! reading cannot explain stops with an error rather than guess.
//!
//! - `grid`: an `.asc` with one logic tile (one with none makes `icepack`
//!   fail), packed and unpacked; `icepack -u` lists every tile of the die,
//!   its kind and its width, and the block RAMs.
//! - `header`: the commands of that bitstream (see `bitstream.rs` for how
//!   they are read), and of the same packed with `-s`, `-m1`, `-m2` and
//!   `-n`, to see which command each option changes.
//! - `crc`: which CRC-16, and over which bytes, gives command 2's value,
//!   searched over the two common polynomials, both common initial values
//!   and every command boundary as a start; exactly one answer must fit
//!   every bitstream tried.
//! - `comment`: a `.comment` of known lines, to see how it is framed.
//! - `cram`: where each tile bit goes. Number every tile bit of the die
//!   i = 0, 1, ...; for each bit k of i, pack the configuration with the
//!   bits whose number has bit k set, and the one with them clear. A CRAM
//!   bit that is set in exactly one of each pair holds tile bit i, read
//!   from the pairs; one set in neither holds no tile bit. Then: every
//!   tile bit lands on exactly one CRAM bit, each tile's bits lie in one
//!   bank, its rows each on one line and its columns each at one position
//!   (so a tile is a `rows` x `cols` layout at an origin), and every tile
//!   of a kind in a bank has the same layout.
//! - `bram`: the same for `.ram_data` bits and the block RAM data.
//! - `extra`: `.extra_bit BANK POS LINE` sets CRAM bit (bank, line, pos),
//!   tried in and outside tiles, and `icepack -u` gives the ones outside
//!   tiles back as `.extra_bit`s.
//! - `fingerprints`: random configurations packed, to check the packer
//!   against later.

use super::asc::{Asc, AscRam, AscTile};
use super::bitstream::{self, Cmd, PackOptions, Parsed, crc16, get_bit};
use super::icepack::Icepack;
use super::{Device, Format, Kind, Layout, RamPlace, TilePlace, fnv64};
use crate::hash::FxMap;
use crate::stimulus::Rng;

/// Packs several configurations at once.
fn pack_all(ip: &Icepack, jobs: Vec<(String, Vec<String>)>) -> Result<Vec<Vec<u8>>, String> {
    let threads = std::thread::available_parallelism().map_or(4, |n| n.get()).min(16);
    let mut out: Vec<Option<Result<Vec<u8>, String>>> = vec![None; jobs.len()];
    std::thread::scope(|s| {
        for (chunk_out, chunk) in out.chunks_mut(jobs.len().div_ceil(threads)).zip(jobs.chunks(jobs.len().div_ceil(threads))) {
            s.spawn(move || {
                for (o, (asc, args)) in chunk_out.iter_mut().zip(chunk) {
                    *o = Some(ip.pack(asc, args));
                }
            });
        }
    });
    out.into_iter().map(|o| o.unwrap()).collect()
}

fn template(p: &Parsed) -> Vec<Cmd> {
    p.cmds.iter().map(|(_, c)| c.clone()).collect()
}

fn no_args() -> Vec<String> {
    Vec::new()
}

/// Runs every experiment, and `fingerprints` random configurations.
pub fn run(ip: &Icepack, fingerprints: u64, log: &mut dyn FnMut(&str)) -> Result<Device, String> {
    let mut d = Device { icepack: ip.commit.clone(), asc_device: "5k".into(), ..Device::default() };

    // grid
    let minimal = ".device 5k\n.logic_tile 1 1\n".to_string();
    let base = ip.pack(&minimal, &no_args())?;
    let grid = Asc::parse(&ip.unpack(&base)?)?;
    for t in &grid.tiles {
        if d.kind(&t.kind).is_none() {
            d.kinds.push(Kind { name: t.kind.clone(), width: t.width });
        }
        let k = d.kind(&t.kind).unwrap();
        if d.kinds[k].width != t.width {
            return Err(format!("grid: {} tiles of widths {} and {}", t.kind, d.kinds[k].width, t.width));
        }
        d.width = d.width.max(t.x + 1);
        d.height = d.height.max(t.y + 1);
    }
    log(&format!("grid: {} x {}, {} tiles, {} block RAMs", d.width, d.height, grid.tiles.len(), grid.ram.len()));

    // header
    let parsed = bitstream::parse(&base)?;
    if parsed.comment.is_some() || parsed.preamble_at != 0 {
        return Err("header: bytes before the preamble with no comment".into());
    }
    let commands = template(&parsed);
    let mut f = Format { commands: commands.clone(), ..Format::default() };
    for blk in &parsed.blocks {
        let banks = if blk.bram { &mut f.bram_banks } else { &mut f.cram_banks };
        if banks.len() <= blk.bank {
            banks.resize(blk.bank + 1, (0, 0));
        }
        let (lines, width) = &mut banks[blk.bank];
        if *width != 0 && *width != blk.width {
            return Err("header: a bank written with two widths".into());
        }
        *width = blk.width;
        *lines = (*lines).max(blk.line0 + blk.height);
    }
    let variant = |args: &[&str]| -> Result<Vec<Cmd>, String> {
        let b = ip.pack(&minimal, &args.iter().map(|s| s.to_string()).collect::<Vec<_>>())?;
        Ok(template(&bitstream::parse(&b)?))
    };
    // The one command an option changes: its code, and old and new arguments.
    let one_change = |other: &[Cmd], what: &str| -> Result<(u8, u32, u32), String> {
        let diffs: Vec<_> = commands.iter().zip(other).filter(|(a, b)| a != b).collect();
        match (commands.len() == other.len(), diffs.as_slice()) {
            (true, [(Cmd::Op { op, arg }, Cmd::Op { op: op2, arg: arg2 })]) if op == op2 => Ok((op >> 4, *arg, *arg2)),
            _ => Err(format!("header: {what} changes more than one command's argument")),
        }
    };
    let (code, old, new) = one_change(&variant(&["-s"])?, "-s")?;
    (f.nosleep_code, f.nosleep_mask) = (code, old ^ new);
    if variant(&["-m0"])? != commands {
        return Err("header: -m0 is not the default".into());
    }
    for m in 1..3u32 {
        let (code, _, new) = one_change(&variant(&[&format!("-m{m}")])?, "-m")?;
        if f.freq_code != 0 && f.freq_code != code || new != m {
            return Err(format!("header: -m{m} sets command {code}'s argument to {new}"));
        }
        f.freq_code = code;
    }
    let no_bram: Vec<Cmd> = commands.iter().filter(|c| **c != Cmd::Bram).cloned().collect();
    if variant(&["-n"])? != no_bram {
        return Err("header: -n does more than leave out the block RAM data".into());
    }
    log(&format!(
        "header: {} commands, CRAM banks {:?}, block RAM banks {:?}; -s flips {:#x} of command {}, -m sets command {}",
        commands.len(),
        f.cram_banks,
        f.bram_banks,
        f.nosleep_mask,
        f.nosleep_code,
        f.freq_code
    ));

    // comment
    let lines = ["ab", "cd", "", "ef"];
    let with = ip.pack(&format!(".comment\n{}\n{minimal}", lines.join("\n")), &no_args())?;
    let empty = ip.pack(&format!(".comment\n{minimal}"), &no_args())?;
    let pw = bitstream::parse(&with)?;
    let pe = bitstream::parse(&empty)?;
    let (cw, ce) = (pw.comment.unwrap_or_default(), pe.comment.unwrap_or_default());
    let at = cw.windows(2).position(|w| w == b"ab").ok_or("comment: the text is not in the bitstream")?;
    f.comment_prefix = cw[..at].to_vec();
    f.comment_end = cw[at + 2];
    let mut body = f.comment_prefix.clone();
    for l in lines {
        body.extend(l.as_bytes());
        body.push(f.comment_end);
    }
    f.comment_suffix = cw[body.len()..].to_vec();
    let mut want_empty = f.comment_prefix.clone();
    want_empty.extend(&f.comment_suffix);
    if !cw.starts_with(&body) || ce != want_empty || with[pw.preamble_at..] != base[..] {
        return Err("comment: not framed as prefix, lines each ended, suffix".into());
    }

    // crc
    let mut fits: Option<Vec<(u16, u16, (u8, u32))>> = None;
    for b in [&base, &with, &ip.pack(&minimal, &["-s".into(), "-n".into()])?] {
        let p = bitstream::parse(b)?;
        let (crc_at, _) = *p.cmds.iter().find(|(_, c)| *c == Cmd::Crc).ok_or("crc: no command 2")?;
        let want = u16::from_be_bytes([b[crc_at + 1], b[crc_at + 2]]);
        let mut here = Vec::new();
        for (i, (at, c)) in p.cmds.iter().enumerate() {
            let Cmd::Op { op, arg } = *c else { continue };
            let Some(&(next, _)) = p.cmds.get(i + 1) else { continue };
            if *at >= crc_at {
                break;
            }
            for poly in [0x1021, 0x8005] {
                for init in [0, 0xffff] {
                    if crc16(poly, init, &b[next..crc_at + 1]) == want {
                        here.push((poly, init, (op, arg)));
                    }
                }
            }
        }
        fits = Some(match fits {
            None => here,
            Some(v) => v.into_iter().filter(|x| here.contains(x)).collect(),
        });
    }
    let fits = fits.unwrap();
    let [(poly, init, reset)] = fits[..] else {
        return Err(format!("crc: {} explanations fit", fits.len()));
    };
    (f.crc_poly, f.crc_init, f.crc_reset) = (poly, init, reset);
    log(&format!("crc: CRC-16 {poly:#06x} from {init:#06x}, over the bytes after command {:02x} {:x}", reset.0, reset.1));
    d.format = f;

    cram(ip, &mut d, &grid, log)?;
    bram(ip, &mut d, &grid, log)?;
    extra(ip, &d, log)?;

    for seed in 1..=fingerprints {
        let (asc, opts) = random_config(&d, seed);
        let b = ip.pack(&asc.to_text(), &opts.args())?;
        let ours = d.pack(&asc, &opts)?;
        if ours != b {
            return Err(format!("fingerprints: the packer differs from icepack for seed {seed}"));
        }
        d.fingerprints.push((seed, b.len(), fnv64(&b)));
    }
    log(&format!("fingerprints: {fingerprints} random configurations pack as icepack packs them"));
    Ok(d)
}

/// The full-die configuration with tile bit i set when `sel(i)`.
fn tiles_text(grid: &Asc, sel: impl Fn(usize) -> bool) -> String {
    let mut s = String::with_capacity(800_000);
    s.push_str(".device 5k\n");
    let mut i = 0;
    for t in &grid.tiles {
        s.push_str(&format!(".{}_tile {} {}\n", t.kind, t.x, t.y));
        for _ in 0..16 {
            for _ in 0..t.width {
                s.push(if sel(i) { '1' } else { '0' });
                i += 1;
            }
            s.push('\n');
        }
    }
    s
}

/// Decodes encoded experiments: `pairs[k]` is the bitstream with the bits
/// numbered with bit k set, and the one with them clear. Returns, for
/// each data bit `(bram, bank, line, pos)` that holds a numbered bit, its
/// number; and how many held none.
fn decode(
    pairs: &[(Vec<u8>, Vec<u8>)],
    bram: bool,
    what: &str,
) -> Result<(FxMap<usize, (usize, usize, usize)>, FxMap<usize, usize>), String> {
    let blocks: Vec<Vec<bitstream::Block>> = pairs
        .iter()
        .flat_map(|(a, b)| [a, b])
        .map(|b| Ok(bitstream::parse(b)?.blocks.into_iter().filter(|x| x.bram == bram).collect()))
        .collect::<Result<_, String>>()?;
    if blocks.windows(2).any(|w| w[0].iter().zip(&w[1]).any(|(a, b)| (a.bank, a.line0, a.width, a.height) != (b.bank, b.line0, b.width, b.height))) {
        return Err(format!("{what}: the experiments' bitstreams are framed differently"));
    }
    let mut found = FxMap::default();
    let mut free: FxMap<usize, usize> = FxMap::default();
    for (bi, blk) in blocks[0].iter().enumerate() {
        for o in 0..blk.width * blk.height {
            let (mut code, mut agree, mut zero) = (0usize, true, true);
            for (k, (p, n)) in pairs.iter().enumerate() {
                let (pb, nb) = (
                    get_bit(p, blocks[2 * k][bi].bit + o),
                    get_bit(n, blocks[2 * k + 1][bi].bit + o),
                );
                agree &= pb != nb;
                zero &= !pb && !nb;
                code |= (pb as usize) << k;
            }
            let at = (blk.bank, blk.line0 + o / blk.width, o % blk.width);
            if zero {
                *free.entry(blk.bank).or_default() += 1;
            } else if !agree {
                return Err(format!("{what}: data bit {at:?} is not one numbered bit"));
            } else if found.insert(code, at).is_some() {
                return Err(format!("{what}: bit {code} is in two places"));
            }
        }
    }
    Ok((found, free))
}

fn cram(ip: &Icepack, d: &mut Device, grid: &Asc, log: &mut dyn FnMut(&str)) -> Result<(), String> {
    let total: usize = grid.tiles.iter().map(|t| 16 * t.width).sum();
    let nbits = usize::BITS - total.leading_zeros();
    let mut jobs = Vec::new();
    for k in 0..nbits {
        jobs.push((tiles_text(grid, |i| i >> k & 1 != 0), no_args()));
        jobs.push((tiles_text(grid, |i| i >> k & 1 == 0), no_args()));
    }
    let out = pack_all(ip, jobs)?;
    let pairs: Vec<(Vec<u8>, Vec<u8>)> = out.chunks(2).map(|p| (p[0].clone(), p[1].clone())).collect();
    let (found, free) = decode(&pairs, false, "cram")?;
    if found.len() != total {
        return Err(format!("cram: {} of {total} tile bits found", found.len()));
    }
    d.free_bits = (0..d.format.cram_banks.len()).map(|b| free.get(&b).copied().unwrap_or(0)).collect();
    let mut i = 0;
    for t in &grid.tiles {
        let at = |r: usize, c: usize| found[&(i + r * t.width + c)];
        let bank = at(0, 0).0;
        let line0 = (0..16).map(|r| at(r, 0).1).min().unwrap();
        let pos0 = (0..t.width).map(|c| at(0, c).2).min().unwrap();
        let rows: [usize; 16] = std::array::from_fn(|r| at(r, 0).1 - line0);
        let cols: Vec<usize> = (0..t.width).map(|c| at(0, c).2 - pos0).collect();
        for r in 0..16 {
            for c in 0..t.width {
                if at(r, c) != (bank, line0 + rows[r], pos0 + cols[c]) {
                    return Err(format!("cram: tile {} {} {}: B{r}[{c}] is not on its row's line and column's position", t.kind, t.x, t.y));
                }
            }
        }
        let kind = d.kind(&t.kind).unwrap();
        let layout = match d.layouts.iter().position(|l| l.kind == kind && l.bank == bank) {
            Some(l) if d.layouts[l].rows == rows && d.layouts[l].cols == cols => l,
            Some(_) => return Err(format!("cram: {} tiles in bank {bank} with different layouts", t.kind)),
            None => {
                d.layouts.push(Layout { kind, bank, rows, cols });
                d.layouts.len() - 1
            }
        };
        d.tiles.push(TilePlace { x: t.x, y: t.y, kind, bank, line0, pos0, layout });
        i += 16 * t.width;
    }
    log(&format!(
        "cram: {total} tile bits, each in one place; {} layouts; {} CRAM bits in no tile",
        d.layouts.len(),
        d.free_bits.iter().sum::<usize>()
    ));
    Ok(())
}

fn bram(ip: &Icepack, d: &mut Device, grid: &Asc, log: &mut dyn FnMut(&str)) -> Result<(), String> {
    let total = grid.ram.len() * 4096;
    let nbits = usize::BITS - total.leading_zeros();
    let text = |sel: &dyn Fn(usize) -> bool| {
        let mut a = Asc::new("5k");
        a.tiles.push(AscTile { kind: "logic".into(), x: 1, y: 1, width: d.kinds[d.kind("logic").unwrap()].width, rows: [0; 16] });
        for (n, r) in grid.ram.iter().enumerate() {
            let words = (0..256)
                .map(|w| (0..16).fold(0u16, |v, k| v | (sel(n * 4096 + w * 16 + k) as u16) << k))
                .collect();
            a.ram.push(AscRam { x: r.x, y: r.y, words });
        }
        a.to_text()
    };
    let mut jobs = Vec::new();
    for k in 0..nbits {
        jobs.push((text(&|i| i >> k & 1 != 0), no_args()));
        jobs.push((text(&|i| i >> k & 1 == 0), no_args()));
    }
    let out = pack_all(ip, jobs)?;
    let pairs: Vec<(Vec<u8>, Vec<u8>)> = out.chunks(2).map(|p| (p[0].clone(), p[1].clone())).collect();
    let (found, free) = decode(&pairs, true, "bram")?;
    if found.len() != total || free.values().any(|&n| n != 0) {
        return Err(format!("bram: {} of {total} bits found, {} data bits hold none", found.len(), free.values().sum::<usize>()));
    }
    let r0 = |n: usize, k: usize| found[&(n * 4096 + k)];
    let pos_min = |n: usize| (0..16).map(|k| r0(n, k).2).min().unwrap();
    d.ram_bits = std::array::from_fn(|k| r0(0, k).2 - pos_min(0));
    for (n, r) in grid.ram.iter().enumerate() {
        let (bank, pos0) = (r0(n, 0).0, pos_min(n));
        for w in 0..256 {
            for k in 0..16 {
                if found[&(n * 4096 + w * 16 + k)] != (bank, w, pos0 + d.ram_bits[k]) {
                    return Err(format!("bram: ram {} {}: word {w} bit {k} is not on line {w} at its bit's position", r.x, r.y));
                }
            }
        }
        d.rams.push(RamPlace { x: r.x, y: r.y, bank, pos0 });
    }
    log(&format!("bram: {} block RAMs; word w on line w, bit k at a fixed position", d.rams.len()));
    Ok(())
}

/// The CRAM bits no tile holds, as `(bank, line, pos)`.
pub fn free_positions(d: &Device) -> Vec<(usize, usize, usize)> {
    let f = &d.format;
    let mut owned: Vec<Vec<bool>> = f.cram_banks.iter().map(|&(l, w)| vec![false; l * w]).collect();
    for t in &d.tiles {
        let l = &d.layouts[t.layout];
        let w = f.cram_banks[t.bank].1;
        for &r in &l.rows {
            for &c in &l.cols {
                owned[t.bank][(t.line0 + r) * w + t.pos0 + c] = true;
            }
        }
    }
    let mut out = Vec::new();
    for (b, o) in owned.iter().enumerate() {
        let w = f.cram_banks[b].1;
        out.extend(o.iter().enumerate().filter(|(_, x)| !**x).map(|(i, _)| (b, i / w, i % w)));
    }
    out
}

fn extra(ip: &Icepack, d: &Device, log: &mut dyn FnMut(&str)) -> Result<(), String> {
    let free = free_positions(d);
    let mut rng = Rng(0x9e37_79b9_7f4a_7c15);
    let mut tried = Vec::new();
    for _ in 0..8 {
        tried.push((free[rng.next() as usize % free.len()], true));
        let t = &d.tiles[rng.next() as usize % d.tiles.len()];
        let l = &d.layouts[t.layout];
        let (r, c) = (rng.next() as usize % 16, rng.next() as usize % l.cols.len());
        tried.push(((t.bank, t.line0 + l.rows[r], t.pos0 + l.cols[c]), false));
    }
    for ((bank, line, pos), outside) in tried {
        let text = format!(".device 5k\n.logic_tile 1 1\n.extra_bit {bank} {pos} {line}\n");
        let b = ip.pack(&text, &no_args())?;
        let p = bitstream::parse(&b)?;
        let set: Vec<_> = p
            .blocks
            .iter()
            .filter(|x| !x.bram)
            .flat_map(|x| (0..x.width * x.height).filter(|&o| get_bit(&b, x.bit + o)).map(|o| (x.bank, x.line0 + o / x.width, o % x.width)))
            .collect();
        if set != [(bank, line, pos)] {
            return Err(format!("extra: .extra_bit {bank} {pos} {line} set {set:?}"));
        }
        let back = Asc::parse(&ip.unpack(&b)?)?;
        if outside != (back.extra == [(bank, pos, line)]) {
            return Err(format!("extra: .extra_bit {bank} {pos} {line} unpacks as {:?}", back.extra));
        }
    }
    log("extra: .extra_bit BANK POS LINE is CRAM bit (bank, line, pos)");
    Ok(())
}

/// A random configuration: random bits in some tiles, some block RAMs and
/// extra bits, perhaps a comment, and random options.
pub fn random_config(d: &Device, seed: u64) -> (Asc, PackOptions) {
    let mut rng = Rng(seed.wrapping_mul(0x9e37_79b9_7f4a_7c15) | 1);
    for _ in 0..4 {
        rng.next();
    }
    let mut asc = Asc::new(&d.asc_device);
    if rng.next() % 2 == 0 {
        asc.comment = Some((0..rng.next() % 3).map(|i| format!("random {seed} {i}")).collect());
    }
    let density = 1 + rng.next() % 6;
    for t in &d.tiles {
        if rng.next() % 8 != 0 && !asc.tiles.is_empty() {
            continue;
        }
        let width = d.kinds[t.kind].width;
        let rows = std::array::from_fn(|_| {
            let mut v = u64::MAX;
            for _ in 0..density {
                v &= rng.next();
            }
            v & ((1 << width) - 1)
        });
        asc.tiles.push(AscTile { kind: d.kinds[t.kind].name.clone(), x: t.x, y: t.y, width, rows });
    }
    for r in &d.rams {
        if rng.next() % 4 == 0 {
            let words = (0..256).map(|_| rng.next() as u16 & rng.next() as u16).collect();
            asc.ram.push(AscRam { x: r.x, y: r.y, words });
        }
    }
    let free = free_positions(d);
    for _ in 0..rng.next() % 4 {
        let (b, l, p) = free[rng.next() as usize % free.len()];
        if !asc.extra.contains(&(b, p, l)) {
            asc.extra.push((b, p, l));
        }
    }
    let opts = PackOptions { nosleep: rng.next() % 2 == 0, freq: (rng.next() % 3) as u8, no_bram: rng.next() % 4 == 0 };
    (asc, opts)
}
