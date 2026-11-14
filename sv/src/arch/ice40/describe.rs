//! The architecture description, as JSON: everything the experiments, the
//! chip database and the datasheet say, each part marked with where it
//! came from, the checks of one source against another, and what is not
//! mapped.
//!
//! Sources:
//!
//! - `"experiment"`: [`super::experiment`], `icepack` as a black box.
//! - `"icebox"`: [`super::icebox`], `icebox_vlog` and `icebox_explain` as
//!   black boxes.
//! - `"chipdb"`: [`super::chipdb`], IceStorm's `chipdb-5k.txt` data file.
//! - `"datasheet"`: Lattice's datasheet for the part ([`super::Part`]),
//!   Table 1.1 (the family's resources) and section 2 (architecture: PLBs
//!   of eight logic cells, each a LUT4, a D flip-flop and carry logic; 4
//!   kbit block RAMs; and the UP5K's SPRAM and DSP blocks).

use super::{Device, Part};
use super::chipdb::{Bit, Chipdb};
use super::experiment::free_positions;
use super::icebox::Semantics;
use crate::hash::FxSet;
use std::fmt::Write;

/// What each tile kind is, by the datasheet's architecture section, with
/// the count from the grid to check it.
fn role(kind: &str) -> &'static str {
    match kind {
        "logic" => "PLB: eight logic cells (LUT4, flip-flop, carry) and their routing",
        "io" => "I/O tile: two PIO pads and their routing",
        "ramb" => "EBR, bottom half (each EBR spans a ramb and the ramt above it)",
        "ramt" => "EBR, top half",
        "dsp0" | "dsp1" | "dsp2" | "dsp3" => "DSP (MAC16) block, one of its four tiles",
        "ipcon" => "connections to hard IP (SPRAM, I2C, SPI, oscillators, PLL, LED driver)",
        _ => "unknown",
    }
}

fn q(s: &str) -> String {
    let mut o = String::from("\"");
    for c in s.chars() {
        match c {
            '"' => o.push_str("\\\""),
            '\\' => o.push_str("\\\\"),
            c if (c as u32) < 0x20 => write!(o, "\\u{:04x}", c as u32).unwrap(),
            c => o.push(c),
        }
    }
    o.push('"');
    o
}

fn nums(v: impl IntoIterator<Item = usize>) -> String {
    format!("[{}]", v.into_iter().map(|x| x.to_string()).collect::<Vec<_>>().join(","))
}

fn bit((r, c): Bit) -> String {
    format!("[{r},{c}]")
}

fn bits(v: &[Bit]) -> String {
    format!("[{}]", v.iter().map(|&b| bit(b)).collect::<Vec<_>>().join(","))
}

/// The checks of one source against another: `(what, passed, detail)`.
pub fn checks(d: &Device, db: Option<&Chipdb>, sem: Option<&Semantics>) -> Vec<(String, bool, String)> {
    let mut out = Vec::new();
    let part = super::part(&d.asc_device).map_or(&[][..], |p| p.resources);
    let count = |k: &str| d.tiles.iter().filter(|t| d.kinds[t.kind].name == k).count();
    let cells = |k: &str| db.map(|db| db.cells.iter().filter(|c| c.kind == k).count());
    for &(what, n, holder) in part {
        let found = match holder {
            "logic tiles x 8" => Some(count("logic") * 8),
            "ramb tiles" => Some(count("ramb")),
            h if h.ends_with(" pins") => db.and_then(|db| db.pins.iter().find(|(p, _)| p == h.trim_end_matches(" pins"))).map(|(_, p)| p.len()),
            h => cells(h.trim_end_matches(" cells")),
        };
        if let Some(found) = found {
            out.push((format!("datasheet {what}: {n}, found {found} ({holder})"), found == n, String::new()));
        }
    }
    out.push((
        "block RAM: a ramb tile, the ramt above it, and a .ram_data block, at each".into(),
        d.rams.len() == count("ramb")
            && d.rams.iter().all(|r| d.tile_at(r.x, r.y).is_some_and(|t| d.kinds[t.kind].name == "ramb") && d.tile_at(r.x, r.y + 1).is_some_and(|t| d.kinds[t.kind].name == "ramt")),
        String::new(),
    ));
    out.push((
        "block RAM: 256 words of 16 bits each, 4096 bits, as the datasheet's 4 kbit".into(),
        d.ram_bits.len() * 256 == 4096,
        String::new(),
    ));
    if let Some(db) = db {
        out.push(("chipdb's grid is the experiments' grid".into(), (db.width, db.height) == (d.width, d.height), format!("{} x {}", db.width, db.height)));
        let ours: FxSet<(String, usize, usize)> = d.tiles.iter().map(|t| (d.kinds[t.kind].name.clone(), t.x, t.y)).collect();
        let theirs: FxSet<(String, usize, usize)> = db.tiles.iter().cloned().collect();
        out.push(("chipdb's tiles and kinds are the experiments'".into(), ours == theirs, format!("{} tiles", theirs.len())));
        let mut outside = Vec::new();
        for (kind, funcs) in &db.tile_bits {
            let w = d.kind(kind).map_or(0, |k| d.kinds[k].width);
            for (f, bs) in funcs {
                if bs.iter().any(|&(r, c)| r >= 16 || c >= w) {
                    outside.push(format!("{kind} {f}"));
                }
            }
        }
        for b in &db.routing.boxes {
            for sw in b {
                if sw.bits.iter().any(|&(r, c)| r >= 16 || c >= 54) {
                    outside.push(format!("switch {}", sw.dst));
                }
            }
        }
        out.push(("chipdb's tile bits are all inside the experiments' tiles".into(), outside.is_empty(), outside.join(", ")));
        let free: FxSet<(usize, usize, usize)> = free_positions(d).into_iter().collect();
        let bad: Vec<_> = db.extra_bits.iter().filter(|(_, b, p, l)| !free.contains(&(*b, *l, *p))).map(|e| e.0.clone()).collect();
        out.push((
            format!("chipdb's {} extra bits are CRAM bits the experiments found in no tile", db.extra_bits.len()),
            bad.is_empty(),
            bad.join(", "),
        ));
        let logic = db.tile_bits.iter().find(|(k, _)| k == "logic");
        if let (Some(sem), Some((_, funcs))) = (sem, logic) {
            let mut ok = true;
            for (lc, (lut, opts)) in sem.lut.iter().zip(&sem.options).enumerate() {
                let mut ours: Vec<Bit> = lut.iter().copied().chain(opts.iter().map(|o| o.1)).collect();
                let mut theirs = funcs.iter().find(|(f, _)| *f == format!("LC_{lc}")).map_or(Vec::new(), |f| f.1.clone());
                ours.sort();
                theirs.sort();
                ok &= ours == theirs;
            }
            out.push(("icebox: each cell's 16 LUT bits and 4 option bits are chipdb's LC_k bits".into(), ok, String::new()));
        }
    }
    out
}

pub fn to_json(d: &Device, db: Option<&Chipdb>, sem: Option<&Semantics>, board: Option<&super::board::Results>) -> String {
    let part: &Part = super::part(&d.asc_device).expect("a known part");
    let f = &d.format;
    let mut s = String::new();
    let count = |k: usize| d.tiles.iter().filter(|t| t.kind == k).count();
    s.push_str("{\n");
    writeln!(s, "\"device\": {{\"part\": {}, \"package\": {}, \"board\": {}, \"asc_device\": {}, \"width\": {}, \"height\": {}}},", q(part.name), q(part.package), q(part.board), q(&d.asc_device), d.width, d.height).unwrap();
    s.push_str("\"sources\": {\n");
    writeln!(s, "  \"experiment\": {},", q(&format!("icepack (IceStorm {}) run as a black box: sv/src/arch/ice40/experiment.rs", d.icepack))).unwrap();
    s.push_str("  \"icebox\": \"icebox_vlog and icebox_explain (pinned IceStorm commit) run as black boxes: sv/src/arch/ice40/icebox.rs\",\n");
    s.push_str("  \"chipdb\": \"IceStorm's chip database, a data file made by icebox_chipdb (pinned IceStorm commit) run as a black box: sv/src/arch/ice40/chipdb.rs\",\n");
    writeln!(s, "  \"datasheet\": {}\n}},", q(part.datasheet)).unwrap();

    s.push_str("\"resources\": [\n");
    let res: Vec<String> = part.resources.iter().map(|(w, n, h)| format!("  {{\"what\": {}, \"count\": {n}, \"held_by\": {}, \"source\": \"datasheet\"}}", q(w), q(h))).collect();
    s.push_str(&res.join(",\n"));
    s.push_str("\n],\n");

    s.push_str("\"kinds\": [\n");
    let kinds: Vec<String> = d
        .kinds
        .iter()
        .enumerate()
        .map(|(i, k)| format!("  {{\"name\": {}, \"rows\": 16, \"cols\": {}, \"count\": {}, \"role\": {}, \"source\": \"experiment; role from the datasheet\"}}", q(&k.name), k.width, count(i), q(role(&k.name))))
        .collect();
    s.push_str(&kinds.join(",\n"));
    s.push_str("\n],\n");

    s.push_str("\"grid\": {\"source\": \"experiment\", \"legend\": {\"L\": \"logic\", \"I\": \"io\", \"b\": \"ramb\", \"t\": \"ramt\", \"0\": \"dsp0\", \"1\": \"dsp1\", \"2\": \"dsp2\", \"3\": \"dsp3\", \"p\": \"ipcon\", \".\": \"none\"}, \"rows_top_down\": [\n");
    let mut rows = Vec::new();
    for y in (0..d.height).rev() {
        let row: String = (0..d.width)
            .map(|x| match d.tile_at(x, y).map(|t| d.kinds[t.kind].name.as_str()) {
                Some("logic") => 'L',
                Some("io") => 'I',
                Some("ramb") => 'b',
                Some("ramt") => 't',
                Some("ipcon") => 'p',
                Some(k) if k.starts_with("dsp") => k.chars().last().unwrap(),
                Some(_) => '?',
                None => '.',
            })
            .collect();
        rows.push(format!("  {}", q(&row)));
    }
    s.push_str(&rows.join(",\n"));
    s.push_str("\n]},\n");

    // bitstream
    s.push_str("\"bitstream\": {\"source\": \"experiment\",\n");
    writeln!(s, "  \"preamble\": [126,170,153,126],").unwrap();
    writeln!(s, "  \"comment\": {{\"prefix\": {}, \"line_end\": {}, \"suffix\": {}, \"before\": \"the preamble\"}},", nums(f.comment_prefix.iter().map(|&b| b as usize)), f.comment_end, nums(f.comment_suffix.iter().map(|&b| b as usize))).unwrap();
    s.push_str("  \"command_form\": \"one byte, high nibble the command, low nibble the argument's length in bytes, then the argument big-endian\",\n");
    s.push_str("  \"command_meanings\": {\"1\": \"select bank\", \"6\": \"next data's width in bits, less one\", \"7\": \"next data's height in lines\", \"8\": \"next data's first line\", \"0 1\": \"CRAM data follows, then two zero bytes\", \"0 3\": \"block RAM data follows, then two zero bytes\", \"2\": \"CRC-16 of the bytes since the CRC reset\",");
    writeln!(s, " \"{:x}\": \"SPI master clock range (icepack -m)\", \"{:x}\": \"option bits; bit mask {} is icepack -s (no deep sleep)\"}},", f.freq_code, f.nosleep_code, f.nosleep_mask).unwrap();
    let cmds: Vec<String> = f
        .commands
        .iter()
        .map(|c| match c {
            super::bitstream::Cmd::Op { op, arg } => format!("[{op},{arg}]"),
            super::bitstream::Cmd::Cram => "\"cram\"".into(),
            super::bitstream::Cmd::Bram => "\"bram\"".into(),
            super::bitstream::Cmd::Crc => "\"crc\"".into(),
        })
        .collect();
    writeln!(s, "  \"commands\": [{}],", cmds.join(",")).unwrap();
    writeln!(s, "  \"crc\": {{\"poly\": {}, \"init\": {}, \"msb_first\": true, \"reset_after\": [{},{}], \"covers\": \"bytes after the reset command, up to and including the CRC command's byte\"}},", f.crc_poly, f.crc_init, f.crc_reset.0, f.crc_reset.1).unwrap();
    writeln!(s, "  \"options\": {{\"freq_command\": {}, \"nosleep_command\": {}, \"nosleep_mask\": {}, \"no_bram\": \"icepack -n leaves out the block RAM data commands\"}},", f.freq_code, f.nosleep_code, f.nosleep_mask).unwrap();
    writeln!(s, "  \"cram_banks\": [{}],", f.cram_banks.iter().zip(&d.free_bits).map(|(&(l, w), &fr)| format!("{{\"lines\": {l}, \"bits\": {w}, \"in_no_tile\": {fr}}}")).collect::<Vec<_>>().join(", ")).unwrap();
    writeln!(s, "  \"bram_banks\": [{}],", f.bram_banks.iter().map(|&(l, w)| format!("{{\"lines\": {l}, \"bits\": {w}}}")).collect::<Vec<_>>().join(", ")).unwrap();
    s.push_str("  \"tile_bit\": \"B{r}[{c}] of a tile is CRAM bank `bank`, line line0 + layout.rows[r], position pos0 + layout.cols[c]\",\n");
    s.push_str("  \"layouts\": [\n");
    let lays: Vec<String> = d.layouts.iter().map(|l| format!("    {{\"kind\": {}, \"bank\": {}, \"rows\": {}, \"cols\": {}}}", q(&d.kinds[l.kind].name), l.bank, nums(l.rows), nums(l.cols.iter().copied()))).collect();
    s.push_str(&lays.join(",\n"));
    s.push_str("\n  ],\n  \"tiles\": [\n");
    let tiles: Vec<String> = d.tiles.iter().map(|t| format!("    {{\"x\": {}, \"y\": {}, \"kind\": {}, \"bank\": {}, \"line0\": {}, \"pos0\": {}, \"layout\": {}}}", t.x, t.y, q(&d.kinds[t.kind].name), t.bank, t.line0, t.pos0, t.layout)).collect();
    s.push_str(&tiles.join(",\n"));
    s.push_str("\n  ],\n");
    s.push_str("  \"ram_bit\": \"word w, bit k of a block RAM is block RAM bank `bank`, line w, position pos0 + ram_bits[k]; .ram_data X Y names it by its ramb tile\",\n");
    writeln!(s, "  \"ram_bits\": {},", nums(d.ram_bits)).unwrap();
    let rams: Vec<String> = d.rams.iter().map(|r| format!("{{\"x\": {}, \"y\": {}, \"bank\": {}, \"pos0\": {}}}", r.x, r.y, r.bank, r.pos0)).collect();
    writeln!(s, "  \"rams\": [{}],", rams.join(", ")).unwrap();
    s.push_str("  \"extra_bit\": \".extra_bit BANK POS LINE is CRAM bank BANK, line LINE, position POS\"\n},\n");

    // logic cell
    if let Some(sem) = sem {
        s.push_str("\"logic_cell\": {\"source\": \"icebox (bits: chipdb's LC_k)\",\n");
        s.push_str("  \"lut_index\": \"index i = in_0 + 2 in_1 + 4 in_2 + 8 in_3: the LUT's output when the inputs are those bits\",\n");
        s.push_str("  \"cells\": [\n");
        let cells: Vec<String> = sem
            .lut
            .iter()
            .zip(&sem.options)
            .enumerate()
            .map(|(lc, (lut, opts))| {
                let o: Vec<String> = opts.iter().map(|(n, b)| format!("{}: {}", q(n), bit(*b))).collect();
                format!("    {{\"lc\": {lc}, \"lut\": {}, \"options\": {{{}}}}}", bits(lut), o.join(", "))
            })
            .collect();
        s.push_str(&cells.join(",\n"));
        s.push_str("\n  ],\n");
        s.push_str("  \"flip_flop\": [\n");
        let ff: Vec<String> = sem.ff.iter().map(|(a, b, c, st)| format!("    {{\"Set_NoReset\": {a}, \"AsyncSetReset\": {b}, \"NegClk\": {c}, \"DffEnable\": true, \"verilog\": {}}}", q(st))).collect();
        s.push_str(&ff.join(",\n"));
        writeln!(s, ",\n    {{\"DffEnable\": false, \"verilog\": {}}}\n  ],", q(&sem.ff_off)).unwrap();
        writeln!(s, "  \"carry\": {{\"CarryEnable\": {}, \"with CarryInSet\": {}}},", q(&sem.carry), q(&sem.carry_in_set)).unwrap();
        s.push_str("  \"names\": \"in_0..in_3 the cell's inputs, lout the LUT's output, out the cell's output, co carry out, ci the tile's carry_in_mux (the first cell's carry in), clk/cen/sr the tile's shared clock, enable and set/reset, x a net with none of these names\",\n");
        writeln!(s, "  \"runs\": {}\n}},", sem.runs).unwrap();
    }

    if let Some(db) = db {
        s.push_str("\"tile_functions\": {\"source\": \"chipdb\", \"note\": \"non-routing settings of each kind of tile, by IceStorm's names; LC_k is a logic cell's 20 bits (see logic_cell)\",\n");
        let kinds: Vec<String> = db
            .tile_bits
            .iter()
            .map(|(k, funcs)| {
                let fs: Vec<String> = funcs.iter().map(|(f, b)| format!("    {}: {}", q(f), bits(b))).collect();
                format!("  {}: {{\n{}\n  }}", q(k), fs.join(",\n"))
            })
            .collect();
        s.push_str(&kinds.join(",\n"));
        s.push_str("\n},\n");

        s.push_str("\"pins\": {\"source\": \"chipdb\", \"note\": \"package pin: the I/O tile and its PIO (z)\",\n");
        let pk: Vec<String> = db
            .pins
            .iter()
            .map(|(p, pins)| {
                let ps: Vec<String> = pins.iter().map(|(n, x, y, z)| format!("{{\"pin\": {}, \"x\": {x}, \"y\": {y}, \"z\": {z}}}", q(n))).collect();
                format!("  {}: [{}]", q(p), ps.join(", "))
            })
            .collect();
        s.push_str(&pk.join(",\n"));
        s.push_str("\n},\n");

        s.push_str("\"globals\": {\"source\": \"chipdb\",\n");
        writeln!(s, "  \"gbufin\": {{\"note\": \"I/O tile (x, y) whose fabout drives global network g\", \"entries\": [{}]}},", db.gbufin.iter().map(|(x, y, g)| nums([*x, *y, *g])).collect::<Vec<_>>().join(",")).unwrap();
        writeln!(s, "  \"gbufpin\": {{\"note\": \"the pad (x, y, z) that can drive global network g\", \"entries\": [{}]}},", db.gbufpin.iter().map(|(x, y, z, g)| nums([*x, *y, *z, *g])).collect::<Vec<_>>().join(",")).unwrap();
        writeln!(s, "  \"padin_bits\": [{}],", db.extra_bits.iter().map(|(f, b, p, l)| format!("{{\"name\": {}, \"bank\": {b}, \"pos\": {p}, \"line\": {l}}}", q(f))).collect::<Vec<_>>().join(", ")).unwrap();
        writeln!(s, "  \"iolatch\": [{}],", db.iolatch.iter().map(|(x, y)| nums([*x, *y])).collect::<Vec<_>>().join(",")).unwrap();
        writeln!(s, "  \"colbuf\": {{\"note\": \"column buffer at (x, y) feeds tile (x, y)\", \"entries\": [{}]}},", db.colbuf.iter().map(|v| nums(*v)).collect::<Vec<_>>().join(",")).unwrap();
        writeln!(s, "  \"ieren\": {{\"note\": \"PIO (x, y, z) has its IE/REN bits in PIO (x, y, z)\", \"entries\": [{}]}}", db.ieren.iter().map(|v| nums(*v)).collect::<Vec<_>>().join(",")).unwrap();
        s.push_str("},\n");

        s.push_str("\"cells\": {\"source\": \"chipdb\", \"note\": \"special-purpose cells; each port or setting is a name in a tile (a net or a configuration bit)\", \"list\": [\n");
        let cs: Vec<String> = db
            .cells
            .iter()
            .map(|c| {
                let ps: Vec<String> = c.ports.iter().map(|(k, x, y, n)| format!("{}: [{x},{y},{}]", q(k), q(n))).collect();
                let at: Vec<String> = c.attrs.iter().map(|(k, v)| format!("{}: {}", q(k), q(v))).collect();
                format!("  {{\"kind\": {}, \"x\": {}, \"y\": {}, \"z\": {}, \"ports\": {{{}}}, \"attrs\": {{{}}}}}", q(&c.kind), c.x, c.y, c.z.map_or("null".into(), |z| z.to_string()), ps.join(", "), at.join(", "))
            })
            .collect();
        s.push_str(&cs.join(",\n"));
        s.push_str("\n]},\n");

        let r = &db.routing;
        s.push_str("\"routing\": {\"source\": \"chipdb\",\n");
        s.push_str("  \"note\": \"a switch drives dst from one source, chosen by its bits' values (first bit first); names are the tile's own names for nets. Every tile with switches has one switch box. A net shape lists its tiles and names relative to its lowest (x, then y) tile, and is placed with that tile at every position of each rectangle [x0, x1, y0, y1]\",\n");
        s.push_str("  \"switch_boxes\": [\n");
        let bx: Vec<String> = r
            .boxes
            .iter()
            .map(|b| {
                let sws: Vec<String> = b
                    .iter()
                    .map(|sw| {
                        let from: Vec<String> = sw.opts.iter().map(|(v, n)| format!("[{},{}]", q(v), q(n))).collect();
                        format!("    {{\"type\": {}, \"dst\": {}, \"bits\": {}, \"from\": [{}]}}", q(if sw.buffer { "buffer" } else { "routing" }), q(&sw.dst), bits(&sw.bits), from.join(","))
                    })
                    .collect();
                format!("   [\n{}\n   ]", sws.join(",\n"))
            })
            .collect();
        s.push_str(&bx.join(",\n"));
        s.push_str("\n  ],\n");
        writeln!(s, "  \"tile_switch_box\": [{}],", r.tile_box.iter().map(|(x, y, b)| nums([*x, *y, *b])).collect::<Vec<_>>().join(",")).unwrap();
        s.push_str("  \"net_shapes\": [\n");
        let sh: Vec<String> = r
            .shapes
            .iter()
            .map(|sh| {
                let m: Vec<String> = sh.members.iter().map(|(dx, dy, n)| format!("[{dx},{dy},{}]", q(n))).collect();
                let at: Vec<String> = sh.rects.iter().map(|(a, b, c, e)| nums([*a, *b, *c, *e])).collect();
                format!("    {{\"members\": [{}], \"at\": [{}]}}", m.join(","), at.join(","))
            })
            .collect();
        s.push_str(&sh.join(",\n"));
        s.push_str("\n  ]\n},\n");
    }

    if let Some(r) = board {
        s.push_str("\"board\": {\"source\": \"board\", \"board\": \"iCESugar-nano (iCELink firmware v1.1a)\", \"method\": \"designs made from this description and packed by Epitome, programmed with icesprog, A3 (from the iCELink's UART) passed to B3 (to its UART) through the logic under test; the Mac sends 64 bytes at 115200 baud and sees them come back (echo), nothing (silent) or something else (other:N)\",\n  \"experiments\": [\n");
        let ex: Vec<String> = r.iter().map(|(n, e, o, len, h)| format!("    {{\"name\": {}, \"expect\": {}, \"observed\": {}, \"bitstream\": {{\"bytes\": {len}, \"fnv64\": \"{h:016x}\"}}}}", q(n), q(e), q(o))).collect();
        s.push_str(&ex.join(",\n"));
        s.push_str("\n  ],\n  \"findings\": [\n");
        let fs: Vec<String> = super::board::findings(r).iter().map(|(w, ok, names)| format!("    {{\"finding\": {}, \"held\": {ok}, \"experiments\": [{}]}}", q(w), names.iter().map(|n| q(n)).collect::<Vec<_>>().join(", "))).collect();
        s.push_str(&fs.join(",\n"));
        s.push_str("\n  ]\n},\n");
    }
    s.push_str("\"checks\": [\n");
    let cs: Vec<String> = checks(d, db, sem).iter().map(|(w, ok, det)| format!("  {{\"check\": {}, \"passed\": {ok}, \"detail\": {}}}", q(w), q(det))).collect();
    s.push_str(&cs.join(",\n"));
    s.push_str("\n],\n");

    s.push_str("\"coverage\": [\n");
    let cov: Vec<String> = coverage(d, db, sem).iter().map(|(w, n)| format!("  {{\"what\": {}, \"count\": {n}}}", q(w))).collect();
    s.push_str(&cov.join(",\n"));
    s.push_str("\n]\n}\n");
    s
}

/// What is mapped and what is not, with counts.
pub fn coverage(d: &Device, db: Option<&Chipdb>, sem: Option<&Semantics>) -> Vec<(String, usize)> {
    let mut out = Vec::new();
    let tile_bits: usize = d.tiles.iter().map(|t| 16 * d.kinds[t.kind].width).sum();
    out.push(("tile bits placed in the CRAM (experiment)".into(), tile_bits));
    out.push(("CRAM bits in no tile (experiment)".into(), d.free_bits.iter().sum()));
    out.push(("block RAM bits placed (experiment)".into(), d.rams.len() * 4096));
    if let Some(db) = db {
        out.push(("CRAM bits in no tile named by chipdb (extra bits)".into(), db.extra_bits.len()));
        out.push(("CRAM bits in no tile and not named".into(), d.free_bits.iter().sum::<usize>() - db.extra_bits.len()));
        // Per kind: tile bits that no function and no switch of the tile uses.
        let mut named = 0;
        let mut unnamed_by_kind: Vec<(String, usize)> = d.kinds.iter().map(|k| (k.name.clone(), 0)).collect();
        for t in &d.tiles {
            let kind = &d.kinds[t.kind].name;
            let mut used: FxSet<Bit> = FxSet::default();
            if let Some((_, funcs)) = db.tile_bits.iter().find(|(k, _)| k == kind) {
                used.extend(funcs.iter().flat_map(|(_, b)| b.iter().copied()));
            }
            if let Some(&(_, _, b)) = db.routing.tile_box.iter().find(|&&(x, y, _)| (x, y) == (t.x, t.y)) {
                used.extend(db.routing.boxes[b].iter().flat_map(|sw| sw.bits.iter().copied()));
            }
            for c in &db.cells {
                for (_, x, y, n) in &c.ports {
                    if (*x, *y) == (t.x, t.y) {
                        if let Some((_, funcs)) = db.tile_bits.iter().find(|(k, _)| k == kind) {
                            used.extend(funcs.iter().filter(|(f, _)| f.ends_with(n.as_str())).flat_map(|(_, b)| b.iter().copied()));
                        }
                    }
                }
            }
            named += used.len();
            unnamed_by_kind[t.kind].1 += 16 * d.kinds[t.kind].width - used.len();
        }
        out.push(("tile bits with a function or switch (chipdb)".into(), named));
        for (k, n) in unnamed_by_kind {
            out.push((format!("{k} tile bits with no known use (all {k} tiles)"), n));
        }
        let (nets, sws) = db.routing.expand();
        out.push(("nets (chipdb)".into(), nets.len()));
        out.push(("switches (chipdb)".into(), sws.len()));
        out.push(("switch boxes".into(), db.routing.boxes.len()));
        out.push(("net shapes".into(), db.routing.shapes.len()));
        for (p, pins) in &db.pins {
            out.push((format!("{p} package pins mapped to PIOs (chipdb)"), pins.len()));
        }
        out.push(("special-purpose cells located (chipdb)".into(), db.cells.len()));
    }
    if let Some(sem) = sem {
        out.push(("logic cell bits whose function was found by experiment (icebox)".into(), sem.lut.len() * 16 + sem.options.iter().map(|o| o.len()).sum::<usize>()));
        out.push(("flip-flop configurations observed (icebox)".into(), sem.ff.len() + 1));
    }
    out
}

/// A SystemVerilog model of a logic cell, built from the statements
/// `icebox_vlog` printed (see [`super::icebox`]): the LUT, the carry, and
/// the flip-flop for each setting of its options. `ci`, `clk`, `cen` and
/// `sr` are the tile's carry in and shared clock, enable and set/reset as
/// routed; on the board an unrouted enable reads 1 and an unrouted
/// set/reset 0 (`board::findings`).
pub fn logic_cell_sv(part: &Part, sem: &Semantics) -> String {
    let mut s = String::new();
    writeln!(s, "// The {} logic cell, generated by sv/examples/ice40_arch.rs from", part.name).unwrap();
    writeln!(s, "// sv/arch/{}/record-icebox.txt: the carry and flip-flop statements are", part.dir).unwrap();
    s.push_str("// icebox_vlog's own (run as a black box), with its nets named by role.\n");
    s.push_str("module ice40_logic_cell #(\n");
    s.push_str("  parameter logic [15:0] LUT_INIT = 16'h0000,\n");
    s.push_str("  parameter bit DFF_ENABLE = 1'b0,\n  parameter bit SET_NORESET = 1'b0,\n");
    s.push_str("  parameter bit ASYNC_SR = 1'b0,\n  parameter bit NEG_CLK = 1'b0\n");
    s.push_str(") (\n  input logic in_0, in_1, in_2, in_3, ci, clk, cen, sr,\n  output logic out, co\n);\n");
    s.push_str("  logic lout, q;\n");
    s.push_str("  // LUT index in_0 + 2 in_1 + 4 in_2 + 8 in_3 (logic_cell.lut_index).\n");
    s.push_str("  assign lout = LUT_INIT[{in_3, in_2, in_1, in_0}];\n");
    writeln!(s, "  // CarryEnable\n  {}", sem.carry).unwrap();
    s.push_str("  generate\n");
    for (i, (set, asy, neg, st)) in sem.ff.iter().enumerate() {
        let cond = format!("SET_NORESET == 1'b{} && ASYNC_SR == 1'b{} && NEG_CLK == 1'b{}", *set as u8, *asy as u8, *neg as u8);
        let kw = if i == 0 { "if" } else { "else if" };
        writeln!(s, "    {kw} ({cond}) begin : ff_{}{}{}\n      {}\n    end", *set as u8, *asy as u8, *neg as u8, st.replace("x <=", "q <=")).unwrap();
    }
    s.push_str("  endgenerate\n");
    writeln!(s, "  // DffEnable, and without it: {}", sem.ff_off).unwrap();
    s.push_str("  assign out = DFF_ENABLE ? q : lout;\nendmodule\n");
    s
}
