//! Renders a part's description as an SVG picture of the die: every tile
//! at its place in the grid, coloured by kind; the eight logic cells and
//! carry chain of each logic tile; the pads of each I/O tile, with their
//! package pins and, on the iCESugar-nano, what the board wires to them;
//! the block RAMs, DSP blocks and special cells; how each part is known
//! (shown on the board, found by experiment, or only named by the chip
//! database); and, optionally, a configuration laid over it: the cells,
//! flip-flops, RAMs and pads it uses.
//!
//! The SVG is plain: no scripts, no external fonts. Detail is in `<title>`
//! elements (shown on hover), so the picture stays readable zoomed out.

use super::asc::Asc;
use super::board::Results;
use super::chipdb::{Bit, Chipdb};
use super::icebox::Semantics;
use super::{Device, part};
use crate::hash::{FxMap, FxSet};
use std::fmt::Write;

/// What to draw besides the description.
#[derive(Default)]
pub struct SvgOptions<'a> {
    /// A configuration to lay over the die.
    pub overlay: Option<&'a Asc>,
    /// The board experiments, which mark what the board has shown.
    pub board: Option<&'a Results>,
    /// The picture's heading, under the part's name.
    pub caption: Option<String>,
}

/// The size of a tile, in SVG units.
const T: f64 = 48.0;
const MARGIN: f64 = 56.0;
const LEGEND: f64 = 300.0;

/// How a part of the die is known.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Known {
    Board,
    Experiment,
    Chipdb,
}

impl Known {
    fn class(self) -> &'static str {
        match self {
            Known::Board => "board",
            Known::Experiment => "exp",
            Known::Chipdb => "named",
        }
    }
    fn say(self) -> &'static str {
        match self {
            Known::Board => "shown on the board",
            Known::Experiment => "found by experiment (icepack, icebox)",
            Known::Chipdb => "named by the chip database only",
        }
    }
}

fn esc(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;")
}

/// What the iCESugar-nano wires to its FPGA's pins (its schematic).
const NANO_PINS: &[(&str, &str)] = &[
    ("A3", "UART from the iCELink (FPGA input)"),
    ("B3", "UART to the iCELink (FPGA output)"),
    ("D1", "12 MHz clock from the iCELink"),
    ("B6", "yellow LED"),
];

pub fn render_svg(d: &Device, db: &Chipdb, sem: &Semantics, opts: &SvgOptions) -> String {
    let p = part(&d.asc_device);
    let (w, h) = (d.width as f64, d.height as f64);
    let width = MARGIN * 2.0 + w * T + LEGEND;
    let height = MARGIN * 2.0 + h * T + 40.0;
    let px = |x: usize| MARGIN + x as f64 * T;
    let py = |y: usize| MARGIN + 40.0 + (d.height - 1 - y) as f64 * T;
    let saw = |prefix: &str| opts.board.is_some_and(|r| r.iter().any(|x| x.0.starts_with(prefix) && x.2 == "echo"));
    let known = |kind: &str| match kind {
        "logic" if saw("lut-cell-") => Known::Board,
        "io" if saw("lut-cell-") => Known::Board,
        "ramb" | "ramt" if saw("ram-read") => Known::Board,
        "logic" | "io" | "ramb" | "ramt" => Known::Experiment,
        _ => Known::Chipdb,
    };
    let kind_at = |x: usize, y: usize| d.tile_at(x, y).map(|t| d.kinds[t.kind].name.as_str());

    // The overlay: tiles with bits, logic cells with LUT bits or a
    // flip-flop, RAMs with contents, pads with a pin type.
    let mut used_tiles: FxSet<(usize, usize)> = FxSet::default();
    let mut used_cells: FxMap<(usize, usize, usize), (bool, bool)> = FxMap::default();
    let mut used_pads: FxSet<(usize, usize, usize)> = FxSet::default();
    let mut used_rams: FxSet<(usize, usize)> = FxSet::default();
    if let Some(a) = opts.overlay {
        let io_funcs = db.tile_bits.iter().find(|(k, _)| k == "io").map(|f| &f.1);
        for t in &a.tiles {
            if t.rows.iter().all(|r| *r == 0) {
                continue;
            }
            used_tiles.insert((t.x, t.y));
            let set = |b: Bit| t.rows[b.0] >> b.1 & 1 != 0;
            if t.kind == "logic" {
                for (lc, lut) in sem.lut.iter().enumerate() {
                    let lut_used = lut.iter().any(|&b| set(b));
                    let ff = sem.options.get(lc).is_some_and(|o| o.iter().any(|(n, b)| n == "DffEnable" && set(*b)));
                    if lut_used || ff {
                        used_cells.insert((t.x, t.y, lc), (lut_used, ff));
                    }
                }
            }
            if t.kind == "io" {
                for z in 0..2 {
                    let pt = io_funcs.map_or(false, |f| f.iter().any(|(n, bs)| n.starts_with(&format!("IOB_{z}.PINTYPE_")) && bs.iter().any(|&b| set(b))));
                    if pt {
                        used_pads.insert((t.x, t.y, z));
                    }
                }
            }
        }
        for r in &a.ram {
            if r.words.iter().any(|w| *w != 0) || used_tiles.contains(&(r.x, r.y)) || used_tiles.contains(&(r.x, r.y + 1)) {
                used_rams.insert((r.x, r.y));
            }
        }
        for t in &d.tiles {
            if d.kinds[t.kind].name == "ramb" && (used_tiles.contains(&(t.x, t.y)) || used_tiles.contains(&(t.x, t.y + 1))) {
                used_rams.insert((t.x, t.y));
            }
        }
    }

    let mut s = String::new();
    writeln!(s, r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {width} {height}" width="{width}" height="{height}" font-family="Helvetica, Arial, sans-serif">"#).unwrap();
    s.push_str(
        r##"<style>
.bg{fill:#ffffff}
.tile{stroke:#94a3b8;stroke-width:0.6}
.logic{fill:#dbeafe}.io{fill:#fef3c7}.ram{fill:#fce7f3}.dsp{fill:#e0e7ff}.ipcon{fill:#dcfce7}
.board{stroke:#15803d;stroke-width:1.6}.exp{stroke:#64748b;stroke-width:0.8}.named{stroke:#94a3b8;stroke-width:0.8}
.lc{fill:#bfdbfe;stroke:#60a5fa;stroke-width:0.4}.carry{stroke:#2563eb;stroke-width:0.6;fill:none}
.pad{fill:#fde68a;stroke:#b45309;stroke-width:0.6}.pin{font-size:7px;fill:#78350f}
.boardpin{font-size:7px;fill:#15803d;font-weight:bold}
.cell{font-size:7px;fill:#3f3f46}.label{font-size:9px;fill:#334155}
.used{stroke:#dc2626;stroke-width:2;fill:none}.usedlc{fill:#f87171;stroke:#b91c1c;stroke-width:0.5}.ff{fill:#7f1d1d}
.usedpad{fill:#ef4444;stroke:#7f1d1d}
.h1{font-size:18px;fill:#0f172a;font-weight:bold}.h2{font-size:11px;fill:#475569}
.lg{font-size:10px;fill:#1e293b}
</style>
<defs>
<pattern id="hatch" width="6" height="6" patternUnits="userSpaceOnUse" patternTransform="rotate(45)"><line x1="0" y1="0" x2="0" y2="6" stroke="#64748b" stroke-width="1" opacity="0.45"/></pattern>
"##,
    );
    // A logic tile's eight cells, cell 0 at the bottom, and the carry chain
    // up through them.
    s.push_str(r#"<g id="lcs">"#);
    for lc in 0..8 {
        let cy = T - 5.0 - (lc as f64 + 1.0) * 5.0;
        write!(s, r#"<rect class="lc" x="5" y="{cy}" width="{}" height="4"/>"#, T - 16.0).unwrap();
    }
    write!(s, r#"<polyline class="carry" points="{x},{y0} {x},{y1}"/>"#, x = T - 7.0, y0 = T - 3.0, y1 = T - 47.0 + 3.0).unwrap();
    s.push_str("</g>\n</defs>\n");
    writeln!(s, r#"<rect class="bg" width="{width}" height="{height}"/>"#).unwrap();
    let name = p.map_or("iCE40".to_string(), |p| format!("{} ({}, {})", p.name, p.package, p.board));
    writeln!(s, r#"<text class="h1" x="{MARGIN}" y="30">{}</text>"#, esc(&name)).unwrap();
    let caption = opts.caption.clone().unwrap_or_else(|| format!("{} x {} tiles, from epitome-sv's description (sv/arch)", d.width, d.height));
    writeln!(s, r#"<text class="h2" x="{MARGIN}" y="46">{}</text>"#, esc(&caption)).unwrap();

    // Pins, by I/O tile and pad.
    let package = p.map(|p| p.package.to_lowercase()).unwrap_or_default();
    let mut pins: FxMap<(usize, usize, usize), String> = FxMap::default();
    if let Some((_, list)) = db.pins.iter().find(|(pk, _)| *pk == package) {
        for (pin, x, y, z) in list {
            pins.insert((*x, *y, *z), pin.clone());
        }
    }
    let nano = p.is_some_and(|p| p.board == "iCESugar-nano");

    // Tiles.
    s.push_str("<g id=\"tiles\">\n");
    for t in &d.tiles {
        let kind = d.kinds[t.kind].name.as_str();
        let (x0, y0) = (px(t.x), py(t.y));
        let k = known(kind);
        let fill = match kind {
            "logic" => "logic",
            "io" => "io",
            "ramb" | "ramt" => "ram",
            "ipcon" => "ipcon",
            _ => "dsp",
        };
        let what = match kind {
            "logic" => "8 logic cells (LUT4 + flip-flop) and a carry chain",
            "io" => "I/O tile: two pads",
            "ramb" => "block RAM, bottom half (write port)",
            "ramt" => "block RAM, top half (read port)",
            "ipcon" => "connections to hard IP",
            _ => "DSP (MAC16), one of its four tiles",
        };
        let bits = 16 * d.kinds[t.kind].width;
        writeln!(
            s,
            r#"<g class="tile {kind}" data-x="{}" data-y="{}"><title>{kind} ({}, {}): {what}; {bits} bits, CRAM bank {}; {}</title><rect class="tile {fill} {}" x="{x0}" y="{y0}" width="{T}" height="{T}"/>"#,
            t.x,
            t.y,
            t.x,
            t.y,
            t.bank,
            k.say(),
            k.class()
        )
        .unwrap();
        if k == Known::Chipdb {
            writeln!(s, r#"<rect x="{x0}" y="{y0}" width="{T}" height="{T}" fill="url(#hatch)"/>"#).unwrap();
        }
        match kind {
            "logic" => {
                writeln!(s, r##"<use href="#lcs" x="{x0}" y="{y0}"/>"##).unwrap();
                for lc in 0..8 {
                    if let Some(&(lut, ff)) = used_cells.get(&(t.x, t.y, lc)) {
                        let cy = y0 + T - 5.0 - (lc as f64 + 1.0) * 5.0;
                        let what = match (lut, ff) {
                            (true, true) => "LUT and flip-flop",
                            (true, false) => "LUT",
                            _ => "flip-flop",
                        };
                        writeln!(s, r#"<rect class="usedlc" x="{}" y="{cy}" width="{}" height="4"><title>cell {lc} of ({}, {}): {what} used</title></rect>"#, x0 + 5.0, T - 16.0, t.x, t.y).unwrap();
                        if ff {
                            writeln!(s, r#"<rect class="ff" x="{}" y="{cy}" width="4" height="4"/>"#, x0 + T - 15.0).unwrap();
                        }
                    }
                }
            }
            "io" => {
                // Pads on the edge of the die, labelled with their pins.
                let vertical = t.x == 0 || t.x == d.width - 1;
                for z in 0..2 {
                    let (cx, cy) = if vertical { (x0 + T / 2.0, y0 + T * (0.3 + 0.4 * z as f64)) } else { (x0 + T * (0.3 + 0.4 * z as f64), y0 + T / 2.0) };
                    let pin = pins.get(&(t.x, t.y, z));
                    let wired = pin.and_then(|p| NANO_PINS.iter().find(|(n, _)| nano && n == p)).map(|(_, w)| *w);
                    let tip = format!(
                        "PIO ({}, {}, {z}){}{}",
                        t.x,
                        t.y,
                        pin.map_or(String::new(), |p| format!(": package pin {p}")),
                        wired.map_or(String::new(), |w| format!("; on the board, {w}"))
                    );
                    let class = if used_pads.contains(&(t.x, t.y, z)) { "pad usedpad" } else { "pad" };
                    writeln!(s, r#"<circle class="{class}" cx="{cx}" cy="{cy}" r="4.5"><title>{}</title></circle>"#, esc(&tip)).unwrap();
                    if let Some(pin) = pin {
                        let (tx, ty) = if vertical { (cx + 6.0, cy + 2.5) } else { (cx - 5.0, cy + 13.0) };
                        let class = if wired.is_some() { "boardpin" } else { "pin" };
                        writeln!(s, r#"<text class="{class}" x="{tx}" y="{ty}">{}</text>"#, esc(pin)).unwrap();
                    }
                }
            }
            _ => {}
        }
        if used_tiles.contains(&(t.x, t.y)) {
            writeln!(s, r#"<rect class="used" x="{}" y="{}" width="{}" height="{}"/>"#, x0 + 1.0, y0 + 1.0, T - 2.0, T - 2.0).unwrap();
        }
        s.push_str("</g>\n");
    }
    s.push_str("</g>\n");

    // Block RAMs (a ramb and the ramt above it) and DSP blocks (dsp0 to
    // dsp3, upwards), outlined and labelled.
    s.push_str("<g id=\"blocks\">\n");
    for t in &d.tiles {
        let kind = d.kinds[t.kind].name.as_str();
        let (n, label) = match kind {
            "ramb" if kind_at(t.x, t.y + 1) == Some("ramt") => (2, "RAM"),
            "dsp0" => (4, "MAC16"),
            _ => continue,
        };
        let (x0, y0) = (px(t.x), py(t.y + n - 1));
        let hgt = T * n as f64;
        let used = kind == "ramb" && used_rams.contains(&(t.x, t.y));
        let tip = if kind == "ramb" { format!("block RAM at ({}, {}): 256 x 16, 512 x 8, 1024 x 4 or 2048 x 2", t.x, t.y) } else { format!("DSP block (MAC16) at ({}, {})", t.x, t.y) };
        writeln!(s, r#"<g class="block {label}"><title>{}</title><rect x="{}" y="{}" width="{}" height="{}" fill="none" stroke="{}" stroke-width="{}"/><text class="label" x="{}" y="{}" transform="rotate(-90 {} {})">{label}</text></g>"#,
            esc(&tip), x0 + 2.0, y0 + 2.0, T - 4.0, hgt - 4.0, if used { "#dc2626" } else { "#9d174d" }, if used { 2.0 } else { 0.8 },
            x0 + T / 2.0 + 3.0, y0 + hgt / 2.0 + 10.0, x0 + T / 2.0 + 3.0, y0 + hgt / 2.0 + 10.0).unwrap();
    }
    s.push_str("</g>\n");

    // Special cells, by the tile (or corner) they sit at.
    s.push_str("<g id=\"cells\">\n");
    let mut at: FxMap<(usize, usize), Vec<String>> = FxMap::default();
    for c in &db.cells {
        at.entry((c.x, c.y)).or_default().push(c.kind.clone());
    }
    let mut spots: Vec<_> = at.into_iter().collect();
    spots.sort();
    for ((x, y), kinds) in spots {
        let (x0, y0) = (px(x), py(y));
        let tip = format!("at ({x}, {y}): {}; named by the chip database only", kinds.join(", "));
        writeln!(s, r##"<g class="special"><title>{}</title><rect x="{}" y="{}" width="{}" height="{}" rx="4" fill="#fff7ed" stroke="#c2410c" stroke-width="0.8"/><rect x="{}" y="{}" width="{}" height="{}" rx="4" fill="url(#hatch)"/>"##,
            esc(&tip), x0 + 3.0, y0 + 3.0, T - 6.0, T - 6.0, x0 + 3.0, y0 + 3.0, T - 6.0, T - 6.0).unwrap();
        let mut seen: Vec<String> = Vec::new();
        for k in &kinds {
            if !seen.contains(k) {
                seen.push(k.clone());
            }
        }
        for (i, k) in seen.iter().take(4).enumerate() {
            let count = kinds.iter().filter(|x| *x == k).count();
            let text = if count > 1 { format!("{k} x{count}") } else { k.clone() };
            writeln!(s, r#"<text class="cell" x="{}" y="{}">{}</text>"#, x0 + 5.0, y0 + 12.0 + 9.0 * i as f64, esc(&text)).unwrap();
        }
        s.push_str("</g>\n");
    }
    s.push_str("</g>\n");

    // Coordinates.
    s.push_str("<g id=\"axes\" class=\"label\">\n");
    for x in 0..d.width {
        writeln!(s, r#"<text class="label" x="{}" y="{}" text-anchor="middle">{x}</text>"#, px(x) + T / 2.0, py(0) + T + 12.0).unwrap();
    }
    for y in 0..d.height {
        writeln!(s, r#"<text class="label" x="{}" y="{}" text-anchor="end">{y}</text>"#, MARGIN - 4.0, py(y) + T / 2.0 + 3.0).unwrap();
    }
    s.push_str("</g>\n");

    // Legend.
    let lx = MARGIN + w * T + 24.0;
    let mut ly = MARGIN + 46.0;
    s.push_str("<g id=\"legend\">\n");
    writeln!(s, r#"<text class="h2" x="{lx}" y="{ly}">Tiles</text>"#).unwrap();
    for (class, text) in [("logic", "logic: 8 LUT4 + flip-flop cells, carry chain"), ("io", "I/O: two pads, package pins"), ("ram", "block RAM (two tiles)"), ("dsp", "DSP, MAC16 (four tiles)"), ("ipcon", "connections to hard IP")] {
        if class == "dsp" && !d.kinds.iter().any(|k| k.name.starts_with("dsp")) || class == "ipcon" && d.kind("ipcon").is_none() {
            continue;
        }
        ly += 16.0;
        writeln!(s, r#"<rect class="tile {class}" x="{lx}" y="{}" width="12" height="12"/><text class="lg" x="{}" y="{}">{text}</text>"#, ly - 10.0, lx + 18.0, ly).unwrap();
    }
    ly += 26.0;
    writeln!(s, r#"<text class="h2" x="{lx}" y="{ly}">How it is known</text>"#).unwrap();
    for (k, text) in [(Known::Board, "shown on the board"), (Known::Experiment, "by experiment: icepack, icebox"), (Known::Chipdb, "named by the chip database only")] {
        ly += 16.0;
        writeln!(s, r##"<rect class="tile {}" x="{lx}" y="{}" width="12" height="12" fill="#f1f5f9"/>"##, k.class(), ly - 10.0).unwrap();
        if k == Known::Chipdb {
            writeln!(s, r#"<rect x="{lx}" y="{}" width="12" height="12" fill="url(#hatch)"/>"#, ly - 10.0).unwrap();
        }
        writeln!(s, r#"<text class="lg" x="{}" y="{ly}">{text}</text>"#, lx + 18.0).unwrap();
    }
    ly += 26.0;
    writeln!(s, r#"<text class="h2" x="{lx}" y="{ly}">Pins</text>"#).unwrap();
    ly += 16.0;
    writeln!(s, r#"<circle class="pad" cx="{}" cy="{}" r="4.5"/><text class="lg" x="{}" y="{ly}">pad, with its {} package pin</text>"#, lx + 6.0, ly - 4.0, lx + 18.0, esc(&package.to_uppercase())).unwrap();
    if nano {
        for (pin, what) in NANO_PINS {
            ly += 14.0;
            writeln!(s, r#"<text class="boardpin" x="{lx}" y="{ly}">{pin}</text><text class="lg" x="{}" y="{ly}">{}</text>"#, lx + 18.0, esc(what)).unwrap();
        }
    }
    if opts.overlay.is_some() {
        ly += 26.0;
        writeln!(s, r#"<text class="h2" x="{lx}" y="{ly}">The configuration</text>"#).unwrap();
        for (svg, text) in [
            (format!(r#"<rect class="used" x="{lx}" y="{}" width="12" height="12"/>"#, ly + 6.0), "a tile it sets bits in"),
            (format!(r#"<rect class="usedlc" x="{lx}" y="{}" width="12" height="5"/>"#, ly + 25.0), "a logic cell it uses (dark: flip-flop)"),
            (format!(r#"<circle class="pad usedpad" cx="{}" cy="{}" r="4.5"/>"#, lx + 6.0, ly + 44.0), "a pad it gives a pin type"),
        ] {
            ly += 18.0;
            writeln!(s, r#"{svg}<text class="lg" x="{}" y="{}">{text}</text>"#, lx + 18.0, ly + 1.0).unwrap();
        }
        let tiles = used_tiles.len();
        ly += 18.0;
        writeln!(s, r#"<text class="lg" x="{lx}" y="{ly}">{tiles} tiles, {} cells, {} RAMs, {} pads used</text>"#, used_cells.len(), used_rams.len(), used_pads.len()).unwrap();
    }
    s.push_str("</g>\n</svg>\n");
    s
}
