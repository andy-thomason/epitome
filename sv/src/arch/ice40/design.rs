//! Small designs made straight from a description: tile functions set by
//! name, routes found through the chip database's switches, LUTs filled by
//! the logic cell's bit order, and the result packed by Epitome's own
//! packer. The board experiments ([`super::board`]) are built with it.

use super::asc::{Asc, AscRam, AscTile};
use super::bitstream::PackOptions;
use super::chipdb::{Bit, Chipdb, Raw};
use super::icebox::{Graph, Semantics};
use super::Device;
use crate::hash::{FxMap, FxSet};
use std::collections::BTreeSet;

impl super::chipdb::Routing {
    /// The nets and switches as the chip database lists them, numbered.
    pub fn raw(&self) -> Raw {
        let (nets, sws) = self.expand();
        let mut index: FxMap<(usize, usize, String), usize> = FxMap::default();
        for (i, net) in nets.iter().enumerate() {
            for m in net {
                index.insert(m.clone(), i);
            }
        }
        let switches = sws
            .into_iter()
            .map(|(x, y, s)| {
                let n = |name: &str| index[&(x, y, name.to_string())];
                let opts = s.opts.iter().map(|(v, src)| (v.clone(), n(src))).collect();
                (x, y, s.buffer, n(&s.dst), s.bits.clone(), opts)
            })
            .collect();
        Raw { nets, switches }
    }
}

pub struct Design<'a> {
    pub d: &'a Device,
    pub db: &'a Chipdb,
    pub sem: &'a Semantics,
    g: &'a Graph,
    used: FxSet<usize>,
    pub bits: BTreeSet<(usize, usize, Bit)>,
    pub extra: Vec<(usize, usize, usize)>,
    /// Block RAM contents, by the RAM's `ramb` tile.
    pub ram: Vec<AscRam>,
}

impl<'a> Design<'a> {
    pub fn new(d: &'a Device, db: &'a Chipdb, sem: &'a Semantics, g: &'a Graph) -> Design<'a> {
        Design { d, db, sem, g, used: FxSet::default(), bits: BTreeSet::new(), extra: Vec::new(), ram: Vec::new() }
    }

    /// Sets the bits of tile function `name` (the chip database's name) in
    /// tile `(x, y)`.
    pub fn func(&mut self, x: usize, y: usize, name: &str) -> Result<(), String> {
        let t = self.d.tile_at(x, y).ok_or_else(|| format!("no tile {x} {y}"))?;
        let kind = &self.d.kinds[t.kind].name;
        let funcs = &self.db.tile_bits.iter().find(|(k, _)| k == kind).ok_or_else(|| format!("no {kind} functions"))?.1;
        let bits = &funcs.iter().find(|(f, _)| f == name).ok_or_else(|| format!("no function {name} in {kind} tiles"))?.1;
        self.bits.extend(bits.iter().map(|&b| (x, y, b)));
        Ok(())
    }

    /// Routes from one tile's net to another's, keeping clear of earlier
    /// routes.
    pub fn route(&mut self, from: (usize, usize, &str), to: (usize, usize, &str)) -> Result<(), String> {
        let (a, b) = (self.g.net(from.0, from.1, from.2)?, self.g.net(to.0, to.1, to.2)?);
        self.bits.extend(self.g.route(a, b, &mut self.used)?);
        Ok(())
    }

    /// Fills logic cell `lc`'s LUT: bit i of `table` is the output for
    /// inputs i.
    pub fn lut(&mut self, x: usize, y: usize, lc: usize, table: u16) {
        for i in 0..16 {
            if table >> i & 1 != 0 {
                self.bits.insert((x, y, self.sem.lut[lc][i]));
            }
        }
    }

    /// Sets one of logic cell `lc`'s options (`DffEnable` and so on).
    pub fn option(&mut self, x: usize, y: usize, lc: usize, name: &str) -> Result<(), String> {
        let b = self.sem.options[lc].iter().find(|(n, _)| n == name).ok_or_else(|| format!("no option {name}"))?.1;
        self.bits.insert((x, y, b));
        Ok(())
    }

    /// The chip database's extra bit called `name`.
    pub fn extra_bit(&mut self, name: &str) -> Result<(), String> {
        let &(_, bank, pos, line) = self.db.extra_bits.iter().find(|e| e.0 == name).ok_or_else(|| format!("no extra bit {name}"))?;
        self.extra.push((bank, pos, line));
        Ok(())
    }

    /// The net of a block RAM's port bit (`RADDR[0]`, `RE`...), by the
    /// RAM's `ramb` tile, from the icebox experiments' port map.
    pub fn ram_port(&self, x: usize, y: usize, port: &str) -> Result<(usize, usize, String), String> {
        let (_, dy, name) = self.sem.ram_ports.iter().find(|p| p.0 == port).ok_or_else(|| format!("no RAM port {port}"))?;
        Ok((x, y + dy, name.clone()))
    }

    /// A package pin's I/O tile and PIO.
    pub fn pin(&self, package: &str, pin: &str) -> Result<(usize, usize, usize), String> {
        let pins = &self.db.pins.iter().find(|(p, _)| p == package).ok_or_else(|| format!("no package {package}"))?.1;
        pins.iter().find(|p| p.0 == pin).map(|p| (p.1, p.2, p.3)).ok_or_else(|| format!("no pin {pin} on {package}"))
    }

    pub fn asc(&self) -> Asc {
        let mut asc = Asc::new(&self.d.asc_device);
        let tiles: BTreeSet<(usize, usize)> = self.bits.iter().map(|&(x, y, _)| (x, y)).collect();
        for (x, y) in tiles {
            let t = self.d.tile_at(x, y).unwrap();
            let mut tile = AscTile { kind: self.d.kinds[t.kind].name.clone(), x, y, width: self.d.kinds[t.kind].width, rows: [0; 16] };
            for &(_, _, (r, c)) in self.bits.range((x, y, (0, 0))..=(x, y, (15, usize::MAX))) {
                tile.rows[r] |= 1 << c;
            }
            asc.tiles.push(tile);
        }
        asc.extra = self.extra.clone();
        asc.ram = self.ram.clone();
        asc
    }

    pub fn bitstream(&self) -> Result<Vec<u8>, String> {
        self.d.pack(&self.asc(), &PackOptions::default())
    }
}
