//! IceStorm's chip database for the UP5K, `chipdb-5k.txt`: a data file
//! that ships with the OSS CAD Suite (`share/icebox/`). The experiments
//! with `icepack` find where bits are but not what they do, and Amy agreed
//! (2026-11-14) to take that from this file. Only the data file is read:
//! never IceStorm's source or documentation. Every fact taken from it is
//! marked `"source": "chipdb"` in the description.
//!
//! The file is lines in sections, each section a line starting `.`:
//! `.device NAME W H NETS`, `.KIND_tile X Y`, `.KIND_tile_bits W H` then
//! `FUNCTION BITS...`, `.pins PACKAGE` then `PIN X Y Z`, `.gbufin` (`X Y
//! GLB`), `.gbufpin` (`X Y Z GLB`), `.iolatch` (`X Y`), `.ieren` (`X Y Z X
//! Y Z`), `.colbuf` (`X Y X Y`), `.extra_cell X Y [Z] KIND` then `KEY X Y
//! NAME`, `.extra_bits` then `FUNCTION BANK POS LINE`, `.net N` then `X Y
//! NAME` for each tile the net passes, and `.buffer X Y DST BITS...` and
//! `.routing X Y DST BITS...` then `VALUES SRC` for each source.
//!
//! The description keeps it compact ([`Routing`]): each tile's switches,
//! named by the tile's own names for their nets, fall into a few switch
//! boxes shared by many tiles; and each net is a shape (its tiles and names
//! relative to its lowest tile) placed at the lowest tile of each rectangle
//! of positions it is found at. [`Routing::expand`] gives back every net and
//! switch, which the tests compare with the file.

use crate::hash::FxMap;
use std::fmt::Write;

/// `B{r}[{c}]`.
pub type Bit = (usize, usize);

pub fn parse_bit(s: &str) -> Result<Bit, String> {
    let bad = || format!("{s}: not a tile bit");
    let (r, c) = s.strip_prefix('B').and_then(|s| s.strip_suffix(']')).and_then(|s| s.split_once('[')).ok_or_else(bad)?;
    Ok((r.parse().map_err(|_| bad())?, c.parse().map_err(|_| bad())?))
}

pub fn bit_name((r, c): Bit) -> String {
    format!("B{r}[{c}]")
}

/// A special-purpose cell: its kind, position, and each port or setting
/// (`key`) tied to a name in a tile.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Cell {
    pub kind: String,
    pub x: usize,
    pub y: usize,
    pub z: Option<usize>,
    pub ports: Vec<(String, usize, usize, String)>,
}

/// A switch in one tile: `dst` driven from one of `opts` by the setting of
/// `bits` (a buffer or a routing switch; IceStorm's names).
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Switch {
    pub buffer: bool,
    pub dst: String,
    pub bits: Vec<Bit>,
    /// The bits' values, first bit first, and the source.
    pub opts: Vec<(String, String)>,
}

/// A net's tiles and names relative to its lowest tile, and the
/// rectangles `(x0, x1, y0, y1)` of lowest tiles it is found at.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Shape {
    pub members: Vec<(isize, isize, String)>,
    pub rects: Vec<(usize, usize, usize, usize)>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Routing {
    pub shapes: Vec<Shape>,
    /// Switch boxes, each sorted.
    pub boxes: Vec<Vec<Switch>>,
    /// `(x, y, box)` for every tile with switches.
    pub tile_box: Vec<(usize, usize, usize)>,
}

/// What the description takes from the file.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Chipdb {
    pub width: usize,
    pub height: usize,
    pub tiles: Vec<(String, usize, usize)>,
    /// For each kind, its non-routing functions and their bits.
    pub tile_bits: Vec<(String, Vec<(String, Vec<Bit>)>)>,
    /// For each package, `(pin, x, y, z)`.
    pub pins: Vec<(String, Vec<(String, usize, usize, usize)>)>,
    pub gbufin: Vec<(usize, usize, usize)>,
    pub gbufpin: Vec<(usize, usize, usize, usize)>,
    pub iolatch: Vec<(usize, usize)>,
    pub ieren: Vec<[usize; 6]>,
    pub colbuf: Vec<[usize; 4]>,
    pub cells: Vec<Cell>,
    /// `(function, bank, pos, line)`.
    pub extra_bits: Vec<(String, usize, usize, usize)>,
    pub routing: Routing,
}

/// The file's nets and switches as it lists them.
pub struct Raw {
    pub nets: Vec<Vec<(usize, usize, String)>>,
    /// `(x, y, buffer, dst net, bits, [(values, src net)])`.
    pub switches: Vec<(usize, usize, bool, usize, Vec<Bit>, Vec<(String, usize)>)>,
}

impl Chipdb {
    pub fn parse(text: &str) -> Result<(Chipdb, Raw), String> {
        let mut db = Chipdb::default();
        let mut raw = Raw { nets: Vec::new(), switches: Vec::new() };
        let mut section: Vec<&str> = Vec::new();
        for (n, line) in text.lines().enumerate() {
            let err = |what: &str| format!("chipdb line {}: {what}", n + 1);
            let line = line.split('#').next().unwrap().trim();
            if line.is_empty() {
                section.clear();
                continue;
            }
            let w: Vec<&str> = line.split_whitespace().collect();
            let num = |i: usize| -> Result<usize, String> { w.get(i).and_then(|s| s.parse().ok()).ok_or_else(|| err("a number expected")) };
            if w[0].starts_with('.') {
                section = w.clone();
                let head = w[0];
                if head == ".device" {
                    (db.width, db.height) = (num(2)?, num(3)?);
                } else if let Some(kind) = head.strip_prefix('.').and_then(|h| h.strip_suffix("_tile_bits")) {
                    db.tile_bits.push((kind.to_string(), Vec::new()));
                } else if let Some(kind) = head.strip_prefix('.').and_then(|h| h.strip_suffix("_tile")) {
                    db.tiles.push((kind.to_string(), num(1)?, num(2)?));
                } else if head == ".pins" {
                    db.pins.push((w[1].to_string(), Vec::new()));
                } else if head == ".extra_cell" {
                    let (z, kind) = if w.len() == 5 { (Some(num(3)?), w[4]) } else { (None, w[3]) };
                    db.cells.push(Cell { kind: kind.to_string(), x: num(1)?, y: num(2)?, z, ports: Vec::new() });
                } else if head == ".net" {
                    if num(1)? != raw.nets.len() {
                        return Err(err("nets out of order"));
                    }
                    raw.nets.push(Vec::new());
                } else if head == ".buffer" || head == ".routing" {
                    let bits = w[4..].iter().map(|b| parse_bit(b)).collect::<Result<_, _>>()?;
                    raw.switches.push((num(1)?, num(2)?, head == ".buffer", num(3)?, bits, Vec::new()));
                }
                continue;
            }
            match section.first().copied() {
                Some(h) if h.ends_with("_tile_bits") => {
                    let bits = w[1..].iter().map(|b| parse_bit(b)).collect::<Result<_, _>>()?;
                    db.tile_bits.last_mut().unwrap().1.push((w[0].to_string(), bits));
                }
                Some(".pins") => db.pins.last_mut().unwrap().1.push((w[0].to_string(), num(1)?, num(2)?, num(3)?)),
                Some(".gbufin") => db.gbufin.push((num(0)?, num(1)?, num(2)?)),
                Some(".gbufpin") => db.gbufpin.push((num(0)?, num(1)?, num(2)?, num(3)?)),
                Some(".iolatch") => db.iolatch.push((num(0)?, num(1)?)),
                Some(".ieren") => db.ieren.push([num(0)?, num(1)?, num(2)?, num(3)?, num(4)?, num(5)?]),
                Some(".colbuf") => db.colbuf.push([num(0)?, num(1)?, num(2)?, num(3)?]),
                Some(".extra_cell") => db.cells.last_mut().unwrap().ports.push((w[0].to_string(), num(1)?, num(2)?, w[3].to_string())),
                Some(".extra_bits") => db.extra_bits.push((w[0].to_string(), num(1)?, num(2)?, num(3)?)),
                Some(".net") => raw.nets.last_mut().unwrap().push((num(0)?, num(1)?, w[2].to_string())),
                Some(".buffer") | Some(".routing") => raw.switches.last_mut().unwrap().5.push((w[0].to_string(), num(1)?)),
                _ => return Err(err("a line outside any section")),
            }
        }
        db.routing = Routing::compress(&raw)?;
        Ok((db, raw))
    }
}

impl Routing {
    pub fn compress(raw: &Raw) -> Result<Routing, String> {
        let mut local: FxMap<(usize, usize, usize), &str> = FxMap::default();
        for (i, net) in raw.nets.iter().enumerate() {
            for (x, y, name) in net {
                local.insert((*x, *y, i), name);
            }
        }
        let name = |x: usize, y: usize, net: usize| -> Result<String, String> {
            local.get(&(x, y, net)).map(|s| s.to_string()).ok_or_else(|| format!("chipdb: net {net} has no name in tile {x} {y}"))
        };
        let mut per_tile: std::collections::BTreeMap<(usize, usize), Vec<Switch>> = Default::default();
        for (x, y, buffer, dst, bits, opts) in &raw.switches {
            let opts = opts.iter().map(|(v, s)| Ok((v.clone(), name(*x, *y, *s)?))).collect::<Result<Vec<_>, String>>()?;
            per_tile.entry((*x, *y)).or_default().push(Switch { buffer: *buffer, dst: name(*x, *y, *dst)?, bits: bits.clone(), opts });
        }
        let mut r = Routing::default();
        let mut box_ids: FxMap<Vec<Switch>, usize> = FxMap::default();
        for ((x, y), mut sws) in per_tile {
            sws.sort();
            let next = box_ids.len();
            let id = *box_ids.entry(sws.clone()).or_insert(next);
            if id == r.boxes.len() {
                r.boxes.push(sws);
            }
            r.tile_box.push((x, y, id));
        }
        // Shapes, in order of first use; their anchors by row, then
        // merged into rectangles of equal runs in consecutive rows.
        let mut shape_ids: FxMap<Vec<(isize, isize, String)>, usize> = FxMap::default();
        let mut anchors: Vec<Vec<(usize, usize)>> = Vec::new();
        for net in &raw.nets {
            let mut m = net.clone();
            m.sort();
            let (x0, y0) = (m[0].0, m[0].1);
            let members: Vec<_> = m.iter().map(|(x, y, n)| (*x as isize - x0 as isize, *y as isize - y0 as isize, n.clone())).collect();
            let next = shape_ids.len();
            let id = *shape_ids.entry(members.clone()).or_insert(next);
            if id == r.shapes.len() {
                r.shapes.push(Shape { members, rects: Vec::new() });
                anchors.push(Vec::new());
            }
            anchors[id].push((x0, y0));
        }
        for (shape, mut an) in r.shapes.iter_mut().zip(anchors) {
            an.sort_by_key(|&(x, y)| (y, x));
            let mut rows: std::collections::BTreeMap<usize, Vec<(usize, usize)>> = Default::default();
            for (x, y) in an {
                let row = rows.entry(y).or_default();
                match row.last_mut() {
                    Some(run) if run.1 + 1 == x => run.1 = x,
                    _ => row.push((x, x)),
                }
            }
            let mut open: FxMap<(usize, usize), usize> = FxMap::default();
            for (y, runs) in rows {
                let mut next = FxMap::default();
                for (x0, x1) in runs {
                    match open.get(&(x0, x1)) {
                        Some(&i) if shape.rects[i].3 + 1 == y => {
                            shape.rects[i].3 = y;
                            next.insert((x0, x1), i);
                        }
                        _ => {
                            shape.rects.push((x0, x1, y, y));
                            next.insert((x0, x1), shape.rects.len() - 1);
                        }
                    }
                }
                open = next;
            }
        }
        Ok(r)
    }

    /// Every net, as its sorted `(x, y, name)`s, sorted; and every switch
    /// with its tile, sorted.
    pub fn expand(&self) -> (Vec<Vec<(usize, usize, String)>>, Vec<(usize, usize, Switch)>) {
        let mut nets = Vec::new();
        for s in &self.shapes {
            for &(x0, x1, y0, y1) in &s.rects {
                for y in y0..=y1 {
                    for x in x0..=x1 {
                        nets.push(
                            s.members.iter().map(|(dx, dy, n)| ((x as isize + dx) as usize, (y as isize + dy) as usize, n.clone())).collect(),
                        );
                    }
                }
            }
        }
        nets.sort();
        let mut sws: Vec<_> = self.tile_box.iter().flat_map(|&(x, y, b)| self.boxes[b].iter().map(move |s| (x, y, s.clone()))).collect();
        sws.sort();
        (nets, sws)
    }

    pub fn to_text(&self) -> String {
        let mut s = String::new();
        for (i, b) in self.boxes.iter().enumerate() {
            writeln!(s, "box {i} {}", b.len()).unwrap();
            for sw in b {
                let bits: Vec<_> = sw.bits.iter().map(|&b| bit_name(b)).collect();
                let opts: Vec<_> = sw.opts.iter().map(|(v, n)| format!("{v} {n}")).collect();
                writeln!(s, "sw {} {} {} : {}", if sw.buffer { "b" } else { "r" }, sw.dst, bits.join(" "), opts.join(" ")).unwrap();
            }
        }
        for (x, y, b) in &self.tile_box {
            writeln!(s, "tilebox {x} {y} {b}").unwrap();
        }
        for sh in &self.shapes {
            let m: Vec<_> = sh.members.iter().map(|(dx, dy, n)| format!("{dx} {dy} {n}")).collect();
            let r: Vec<_> = sh.rects.iter().map(|(a, b, c, d)| format!("{a} {b} {c} {d}")).collect();
            writeln!(s, "shape {} : {}", m.join(" "), r.join(" ")).unwrap();
        }
        s
    }
}

impl Chipdb {
    /// The facts in a compact text form of the record's kind.
    pub fn to_text(&self) -> String {
        let mut s = String::new();
        s.push_str("# Facts taken from IceStorm's chipdb-5k.txt (a data file; see sv/src/arch/ice40/chipdb.rs),\n");
        s.push_str("# in a compact form. Regenerate with the ice40_arch example's --chipdb option.\n");
        writeln!(s, "device {} {}", self.width, self.height).unwrap();
        for (k, x, y) in &self.tiles {
            writeln!(s, "tile {k} {x} {y}").unwrap();
        }
        for (kind, funcs) in &self.tile_bits {
            for (f, bits) in funcs {
                let b: Vec<_> = bits.iter().map(|&b| bit_name(b)).collect();
                writeln!(s, "tile_bits {kind} {f} {}", b.join(" ")).unwrap();
            }
        }
        for (pkg, pins) in &self.pins {
            for (p, x, y, z) in pins {
                writeln!(s, "pin {pkg} {p} {x} {y} {z}").unwrap();
            }
        }
        for (x, y, g) in &self.gbufin {
            writeln!(s, "gbufin {x} {y} {g}").unwrap();
        }
        for (x, y, z, g) in &self.gbufpin {
            writeln!(s, "gbufpin {x} {y} {z} {g}").unwrap();
        }
        for (x, y) in &self.iolatch {
            writeln!(s, "iolatch {x} {y}").unwrap();
        }
        for v in &self.ieren {
            writeln!(s, "ieren {}", v.map(|x| x.to_string()).join(" ")).unwrap();
        }
        for v in &self.colbuf {
            writeln!(s, "colbuf {}", v.map(|x| x.to_string()).join(" ")).unwrap();
        }
        for c in &self.cells {
            let z = c.z.map_or("-".to_string(), |z| z.to_string());
            writeln!(s, "cell {} {} {} {}", c.kind, c.x, c.y, z).unwrap();
            for (k, x, y, n) in &c.ports {
                writeln!(s, "port {k} {x} {y} {n}").unwrap();
            }
        }
        for (f, b, p, l) in &self.extra_bits {
            writeln!(s, "extra_bit {f} {b} {p} {l}").unwrap();
        }
        s.push_str(&self.routing.to_text());
        s
    }

    pub fn from_text(text: &str) -> Result<Chipdb, String> {
        let mut db = Chipdb::default();
        for (n, line) in text.lines().enumerate() {
            let err = |what: &str| format!("chipdb facts line {}: {what}", n + 1);
            let w: Vec<&str> = line.split_whitespace().collect();
            if w.is_empty() || w[0].starts_with('#') {
                continue;
            }
            let num = |i: usize| -> Result<usize, String> { w.get(i).and_then(|s| s.parse().ok()).ok_or_else(|| err("a number expected")) };
            let inum = |i: usize| -> Result<isize, String> { w.get(i).and_then(|s| s.parse().ok()).ok_or_else(|| err("a number expected")) };
            let colon = w.iter().position(|&x| x == ":");
            match w[0] {
                "device" => (db.width, db.height) = (num(1)?, num(2)?),
                "tile" => db.tiles.push((w[1].to_string(), num(2)?, num(3)?)),
                "tile_bits" => {
                    let bits = w[3..].iter().map(|b| parse_bit(b)).collect::<Result<_, _>>()?;
                    match db.tile_bits.last_mut() {
                        Some((k, v)) if k == w[1] => v.push((w[2].to_string(), bits)),
                        _ => db.tile_bits.push((w[1].to_string(), vec![(w[2].to_string(), bits)])),
                    }
                }
                "pin" => {
                    let p = (w[2].to_string(), num(3)?, num(4)?, num(5)?);
                    match db.pins.last_mut() {
                        Some((k, v)) if k == w[1] => v.push(p),
                        _ => db.pins.push((w[1].to_string(), vec![p])),
                    }
                }
                "gbufin" => db.gbufin.push((num(1)?, num(2)?, num(3)?)),
                "gbufpin" => db.gbufpin.push((num(1)?, num(2)?, num(3)?, num(4)?)),
                "iolatch" => db.iolatch.push((num(1)?, num(2)?)),
                "ieren" => db.ieren.push([num(1)?, num(2)?, num(3)?, num(4)?, num(5)?, num(6)?]),
                "colbuf" => db.colbuf.push([num(1)?, num(2)?, num(3)?, num(4)?]),
                "cell" => db.cells.push(Cell {
                    kind: w[1].to_string(),
                    x: num(2)?,
                    y: num(3)?,
                    z: if w[4] == "-" { None } else { Some(num(4)?) },
                    ports: Vec::new(),
                }),
                "port" => db.cells.last_mut().ok_or_else(|| err("a port before any cell"))?.ports.push((w[1].to_string(), num(2)?, num(3)?, w[4].to_string())),
                "extra_bit" => db.extra_bits.push((w[1].to_string(), num(2)?, num(3)?, num(4)?)),
                "box" => db.routing.boxes.push(Vec::new()),
                "sw" => {
                    let c = colon.ok_or_else(|| err("no :"))?;
                    let bits = w[3..c].iter().map(|b| parse_bit(b)).collect::<Result<_, _>>()?;
                    let opts = w[c + 1..].chunks(2).map(|p| (p[0].to_string(), p[1].to_string())).collect();
                    db.routing.boxes.last_mut().ok_or_else(|| err("a switch before any box"))?.push(Switch {
                        buffer: w[1] == "b",
                        dst: w[2].to_string(),
                        bits,
                        opts,
                    });
                }
                "tilebox" => db.routing.tile_box.push((num(1)?, num(2)?, num(3)?)),
                "shape" => {
                    let c = colon.ok_or_else(|| err("no :"))?;
                    let members = (1..c).step_by(3).map(|i| Ok((inum(i)?, inum(i + 1)?, w[i + 2].to_string()))).collect::<Result<_, String>>()?;
                    let rects = (c + 1..w.len()).step_by(4).map(|i| Ok((num(i)?, num(i + 1)?, num(i + 2)?, num(i + 3)?))).collect::<Result<_, String>>()?;
                    db.routing.shapes.push(Shape { members, rects });
                }
                _ => return Err(err("unknown fact")),
            }
        }
        Ok(db)
    }
}
