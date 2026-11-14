//! Placement: each logic cell of a [`Mapped`] design to a logic tile and
//! a place (0 to 7) in it, each port bit to the package pin a constraint
//! names.
//!
//! A tile's eight cells share their flip-flops' enable and set/reset nets
//! (and clock), so the flip-flops in a tile must agree on them; a carry
//! chain takes places 0 upwards in consecutive tiles of a column, its
//! carry passing from one tile's cell 7 to the next tile's cell 0.
//!
//! Cells are placed greedily, each where it adds least wirelength (the
//! half-perimeter of each net's tiles' bounding box), and then improved by
//! simulated annealing: moves and swaps of cells, and moves of chains,
//! within a window that shrinks as the temperature falls.

use super::map::{Mapped, Src};
use super::{Device, chipdb::Chipdb};
use crate::hash::FxMap;

/// Where things went.
#[derive(Clone, Debug)]
pub struct Placement {
    /// Each cell's tile and place.
    pub at: Vec<(usize, usize, usize)>,
    /// Each input port bit's I/O: tile and IOB, and the package pin.
    pub inputs: FxMap<(u32, u32), (usize, usize, usize, String)>,
    /// Each output port bit's.
    pub outputs: FxMap<(u32, u32), (usize, usize, usize, String)>,
    /// The wirelength at the end.
    pub cost: f64,
}

/// A port bit's name as constraints give it: `name` for a one-bit port,
/// `name[i]` otherwise.
pub fn bit_name(name: &str, width: u32, bit: u32) -> String {
    if width == 1 { name.to_string() } else { format!("{name}[{bit}]") }
}

/// A small xorshift, so that placements are the same each run.
pub struct Rng(u64);

impl Rng {
    pub fn new(seed: u64) -> Rng {
        Rng(seed | 1)
    }
    pub fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
    pub fn below(&mut self, n: usize) -> usize {
        (self.next() % n.max(1) as u64) as usize
    }
    pub fn unit(&mut self) -> f64 {
        (self.next() >> 11) as f64 / (1u64 << 53) as f64
    }
}

/// What a cell asks of its tile: its flip-flop's enable and set/reset.
type Ctrl = Option<(Option<Src>, Option<Src>)>;

struct State<'m> {
    m: &'m Mapped,
    /// The logic tiles, and each one's index by position.
    tiles: Vec<(usize, usize)>,
    tile_at: FxMap<(usize, usize), usize>,
    /// Each cell's tile (by index).
    tile_of: Vec<usize>,
    /// Each tile's cells that are in no chain, and how many places
    /// chains take there.
    free: Vec<Vec<u32>>,
    chained: Vec<usize>,
    /// Each chain's first tile.
    chain_at: Vec<usize>,
    chain_of: Vec<Option<(u32, u32)>>,
    ctrl: Vec<Ctrl>,
    /// The nets: each one's cells and fixed points (pins).
    nets: Vec<(Vec<u32>, Vec<(usize, usize)>)>,
    nets_of: Vec<Vec<usize>>,
    net_cost: Vec<f64>,
}

impl<'m> State<'m> {
    fn pos(&self, lc: u32) -> (usize, usize) {
        self.tiles[self.tile_of[lc as usize]]
    }

    fn hpwl(&self, n: usize) -> f64 {
        let (cells, fixed) = &self.nets[n];
        let mut b = (usize::MAX, 0, usize::MAX, 0);
        for p in cells.iter().map(|&c| self.pos(c)).chain(fixed.iter().copied()) {
            b = (b.0.min(p.0), b.1.max(p.0), b.2.min(p.1), b.3.max(p.1));
        }
        if b.0 == usize::MAX { 0.0 } else { (b.1 - b.0 + b.3 - b.2) as f64 }
    }

    /// Whether tile `t` can hold its cells: eight places, one set of
    /// flip-flop controls.
    fn legal(&self, t: usize) -> bool {
        if self.free[t].len() + self.chained[t] > 8 {
            return false;
        }
        let mut seen: Ctrl = None;
        let chain_cells = self.chain_cells(t);
        for c in self.free[t].iter().chain(&chain_cells) {
            if let Some(k) = self.ctrl[*c as usize] {
                match seen {
                    None => seen = Some(k),
                    Some(s) if s != k => return false,
                    _ => {}
                }
            }
        }
        true
    }

    fn chain_cells(&self, t: usize) -> Vec<u32> {
        if self.chained[t] == 0 {
            return Vec::new();
        }
        (0..self.m.lcs.len() as u32).filter(|&c| self.chain_of[c as usize].is_some() && self.tile_of[c as usize] == t).collect()
    }

    /// The tiles chain `c` would take from tile `t`, if it fits there.
    fn chain_tiles(&self, c: usize, t: usize) -> Option<Vec<usize>> {
        let len = self.m.chains[c].lcs.len();
        let (x, y) = self.tiles[t];
        (0..len.div_ceil(8)).map(|k| self.tile_at.get(&(x, y + k)).copied()).collect()
    }

    fn move_chain(&mut self, c: usize, t: usize) {
        let old = self.chain_tiles(c, self.chain_at[c]).unwrap();
        let new = self.chain_tiles(c, t).unwrap();
        let len = self.m.chains[c].lcs.len();
        for (k, &ot) in old.iter().enumerate() {
            self.chained[ot] -= (len - 8 * k).min(8);
        }
        for (k, &nt) in new.iter().enumerate() {
            self.chained[nt] += (len - 8 * k).min(8);
        }
        for (k, &l) in self.m.chains[c].lcs.iter().enumerate() {
            self.tile_of[l as usize] = new[k / 8];
        }
        self.chain_at[c] = t;
    }

}

/// Places `m` on the device: `pins` maps port bits ([`bit_name`]) to
/// package pins.
pub fn place(d: &Device, db: &Chipdb, m: &Mapped, package: &str, pins: &[(String, String)], seed: u64) -> Result<Placement, String> {
    let _ = d;
    let pkg = &db.pins.iter().find(|(p, _)| p == package).ok_or_else(|| format!("no package {package}"))?.1;
    let pin = |name: &str| -> Result<(usize, usize, usize, String), String> {
        let p = pins.iter().find(|(n, _)| n == name).ok_or_else(|| format!("no pin given for {name}"))?;
        let q = pkg.iter().find(|q| q.0 == p.1).ok_or_else(|| format!("no pin {} on {package}", p.1))?;
        Ok((q.1, q.2, q.3, q.0.clone()))
    };
    let mut inputs = FxMap::default();
    let mut outputs = FxMap::default();
    for (i, p) in m.inputs.iter().enumerate() {
        for b in 0..p.width {
            inputs.insert((i as u32, b), pin(&bit_name(&p.name, p.width, b))?);
        }
    }
    for (i, (name, bits)) in m.outputs.iter().enumerate() {
        for b in 0..bits.len() as u32 {
            outputs.insert((i as u32, b), pin(&bit_name(name, bits.len() as u32, b))?);
        }
    }
    let mut used_pins: Vec<&String> = inputs.values().chain(outputs.values()).map(|p| &p.3).collect();
    used_pins.sort();
    if let Some(w) = used_pins.windows(2).find(|w| w[0] == w[1]) {
        return Err(format!("pin {} is given to two ports", w[0]));
    }

    let mut tiles: Vec<(usize, usize)> = db.tiles.iter().filter(|t| t.0 == "logic").map(|t| (t.1, t.2)).collect();
    tiles.sort();
    let tile_at: FxMap<(usize, usize), usize> = tiles.iter().enumerate().map(|(i, &p)| (p, i)).collect();
    let n = m.lcs.len();
    if n > tiles.len() * 8 {
        return Err(format!("{n} logic cells; the device has {}", tiles.len() * 8));
    }
    // Nets: each signal's driver and readers.
    let mut net_ix: FxMap<Src, usize> = FxMap::default();
    let mut nets: Vec<(Vec<u32>, Vec<(usize, usize)>)> = Vec::new();
    let mut term = |s: Src, cell: Option<u32>, fixed: Option<(usize, usize)>, nets: &mut Vec<(Vec<u32>, Vec<(usize, usize)>)>| {
        let k = *net_ix.entry(s).or_insert_with(|| {
            nets.push((Vec::new(), Vec::new()));
            match s {
                Src::Lc(c) => nets.last_mut().unwrap().0.push(c),
                Src::Pin { port, bit } => nets.last_mut().unwrap().1.push((inputs[&(port, bit)].0, inputs[&(port, bit)].1)),
            }
            nets.len() - 1
        });
        nets[k].0.extend(cell);
        nets[k].1.extend(fixed);
    };
    let clock_port = m.clock.map(|c| c.0);
    for (i, lc) in m.lcs.iter().enumerate() {
        let ff = lc.ff.iter().flat_map(|f| [f.en, f.sr]).flatten();
        for s in lc.ins.iter().flatten().copied().chain(ff) {
            if matches!(s, Src::Pin { port, .. } if Some(port) == clock_port) {
                continue;
            }
            term(s, Some(i as u32), None, &mut nets);
        }
    }
    for (i, (_, bits)) in m.outputs.iter().enumerate() {
        for (b, &s) in bits.iter().enumerate() {
            let o = &outputs[&(i as u32, b as u32)];
            term(s, None, Some((o.0, o.1)), &mut nets);
        }
    }
    for net in &mut nets {
        net.0.sort();
        net.0.dedup();
    }
    let mut nets_of = vec![Vec::new(); n];
    for (k, net) in nets.iter().enumerate() {
        for &c in &net.0 {
            nets_of[c as usize].push(k);
        }
    }
    let chain_of = m.chain_of();
    let ctrl: Vec<Ctrl> = m.lcs.iter().map(|l| l.ff.as_ref().map(|f| (f.en, f.sr))).collect();
    let mut st = State {
        m,
        tile_at,
        tile_of: vec![usize::MAX; n],
        free: vec![Vec::new(); tiles.len()],
        chained: vec![0; tiles.len()],
        chain_at: vec![usize::MAX; m.chains.len()],
        chain_of,
        ctrl,
        nets,
        nets_of,
        net_cost: Vec::new(),
        tiles,
    };

    // ---- greedy ----
    // Chains first, longest first, near the middle of the die.
    let mut order: Vec<usize> = (0..m.chains.len()).collect();
    order.sort_by_key(|&c| std::cmp::Reverse(m.chains[c].lcs.len()));
    let (cx, cy) = {
        let xs: Vec<usize> = st.tiles.iter().map(|t| t.0).collect();
        let ys: Vec<usize> = st.tiles.iter().map(|t| t.1).collect();
        ((xs.iter().min().unwrap() + xs.iter().max().unwrap()) / 2, (ys.iter().min().unwrap() + ys.iter().max().unwrap()) / 2)
    };
    for c in order {
        let mut cands: Vec<usize> = (0..st.tiles.len()).collect();
        cands.sort_by_key(|&t| st.tiles[t].0.abs_diff(cx) * 2 + st.tiles[t].1.abs_diff(cy));
        let mut done = false;
        for t in cands {
            let Some(span) = st.chain_tiles(c, t) else { continue };
            if span.iter().any(|&s| st.chained[s] > 0 || !st.free[s].is_empty()) {
                continue;
            }
            let len = m.chains[c].lcs.len();
            for (k, &s) in span.iter().enumerate() {
                st.chained[s] += (len - 8 * k).min(8);
            }
            for (k, &l) in m.chains[c].lcs.iter().enumerate() {
                st.tile_of[l as usize] = span[k / 8];
            }
            st.chain_at[c] = t;
            done = true;
            break;
        }
        if !done {
            return Err(format!("no room for a carry chain of {} cells", m.chains[c].lcs.len()));
        }
    }
    // Then each other cell, those nearest the placed first.
    let mut placed: Vec<bool> = (0..n).map(|i| st.tile_of[i] != usize::MAX).collect();
    loop {
        let mut pick = None;
        let mut best_links = -1i64;
        for i in 0..n {
            if placed[i] {
                continue;
            }
            let links = st.nets_of[i].iter().map(|&k| st.nets[k].0.iter().filter(|&&c| placed[c as usize]).count() as i64 + st.nets[k].1.len() as i64).sum::<i64>();
            if links > best_links {
                best_links = links;
                pick = Some(i);
            }
        }
        let Some(i) = pick else { break };
        let mut best: Option<(f64, usize)> = None;
        for t in 0..st.tiles.len() {
            if st.free[t].len() + st.chained[t] >= 8 {
                continue;
            }
            st.free[t].push(i as u32);
            st.tile_of[i] = t;
            if st.legal(t) {
                // Only the placed cells count.
                let mut c = 0.0;
                for &k in &st.nets_of[i] {
                    let (cells, fixed) = &st.nets[k];
                    let pts: Vec<(usize, usize)> =
                        cells.iter().filter(|&&x| placed[x as usize] || x as usize == i).map(|&x| st.pos(x)).chain(fixed.iter().copied()).collect();
                    let (x0, x1) = (pts.iter().map(|p| p.0).min().unwrap(), pts.iter().map(|p| p.0).max().unwrap());
                    let (y0, y1) = (pts.iter().map(|p| p.1).min().unwrap(), pts.iter().map(|p| p.1).max().unwrap());
                    c += (x1 - x0 + y1 - y0) as f64;
                }
                // Prefer the middle where nothing pulls.
                let (x, y) = st.tiles[t];
                c += 0.001 * (x.abs_diff(cx) + y.abs_diff(cy)) as f64;
                if best.is_none_or(|b| c < b.0) {
                    best = Some((c, t));
                }
            }
            st.free[t].pop();
        }
        let (_, t) = best.ok_or("no legal tile for a cell: too many sets of flip-flop controls")?;
        st.free[t].push(i as u32);
        st.tile_of[i] = t;
        placed[i] = true;
    }
    st.net_cost = (0..st.nets.len()).map(|k| st.hpwl(k)).collect();

    // ---- annealing ----
    let mut rng = Rng::new(seed);
    let total = |st: &State| st.net_cost.iter().sum::<f64>();
    let free_cells: Vec<u32> = (0..n as u32).filter(|&c| st.chain_of[c as usize].is_none()).collect();
    let movable = free_cells.len() + m.chains.len();
    if movable > 0 && !st.nets.is_empty() {
        let (w, h) = (st.tiles.iter().map(|t| t.0).max().unwrap(), st.tiles.iter().map(|t| t.1).max().unwrap());
        let mut window = w.max(h) as f64;
        let moves = (10.0 * (movable as f64).powf(4.0 / 3.0)).max(200.0) as usize;
        // One move: returns (cost change, undo).
        let try_move = |st: &mut State, rng: &mut Rng, window: f64, temp: f64| -> bool {
            let pick = rng.below(movable);
            let (x0, y0);
            let mut touched: Vec<usize>;
            let mut undo: Vec<(u32, usize)> = Vec::new();
            let mut chain_undo: Option<(usize, usize)> = None;
            if pick < free_cells.len() {
                let a = free_cells[pick];
                (x0, y0) = st.pos(a);
                let wn = window.max(1.0) as isize;
                let tx = x0 as isize + rng.below(2 * wn as usize + 1) as isize - wn;
                let ty = y0 as isize + rng.below(2 * wn as usize + 1) as isize - wn;
                let Some(&t) = st.tile_at.get(&(tx.max(0) as usize, ty.max(0) as usize)) else { return false };
                let ta = st.tile_of[a as usize];
                if t == ta {
                    return false;
                }
                touched = st.nets_of[a as usize].clone();
                // Move, or swap with a cell there.
                let b = if st.free[t].len() + st.chained[t] >= 8 || rng.below(2) == 0 {
                    if st.free[t].is_empty() { None } else { Some(st.free[t][rng.below(st.free[t].len())]) }
                } else {
                    None
                };
                st.free[ta].retain(|&c| c != a);
                st.free[t].push(a);
                st.tile_of[a as usize] = t;
                undo.push((a, ta));
                if let Some(b) = b {
                    st.free[t].retain(|&c| c != b);
                    st.free[ta].push(b);
                    st.tile_of[b as usize] = ta;
                    undo.push((b, t));
                    touched.extend(st.nets_of[b as usize].iter().copied());
                }
                if !st.legal(t) || !st.legal(ta) {
                    for &(c, from) in undo.iter().rev() {
                        let now = st.tile_of[c as usize];
                        st.free[now].retain(|&x| x != c);
                        st.free[from].push(c);
                        st.tile_of[c as usize] = from;
                    }
                    return false;
                }
            } else {
                let c = pick - free_cells.len();
                let t = rng.below(st.tiles.len());
                let Some(span) = st.chain_tiles(c, t) else { return false };
                let old = st.chain_at[c];
                let old_span = st.chain_tiles(c, old).unwrap();
                if span.iter().any(|s| st.chained[*s] > 0 && !old_span.contains(s)) || t == old {
                    return false;
                }
                st.move_chain(c, t);
                if span.iter().chain(&old_span).any(|&s| !st.legal(s)) {
                    st.move_chain(c, old);
                    return false;
                }
                touched = m.chains[c].lcs.iter().flat_map(|&l| st.nets_of[l as usize].iter().copied()).collect();
                chain_undo = Some((c, old));
            }
            touched.sort();
            touched.dedup();
            let before: f64 = touched.iter().map(|&k| st.net_cost[k]).sum();
            let after_each: Vec<f64> = touched.iter().map(|&k| st.hpwl(k)).collect();
            let delta = after_each.iter().sum::<f64>() - before;
            if delta <= 0.0 || rng.unit() < (-delta / temp).exp() {
                for (k, v) in touched.iter().zip(after_each) {
                    st.net_cost[*k] = v;
                }
                return true;
            }
            if let Some((c, old)) = chain_undo {
                st.move_chain(c, old);
            }
            for &(c, from) in undo.iter().rev() {
                let now = st.tile_of[c as usize];
                st.free[now].retain(|&x| x != c);
                st.free[from].push(c);
                st.tile_of[c as usize] = from;
            }
            false
        };
        let mut temp = 1.0 + total(&st) / st.nets.len() as f64;
        for _ in 0..200 {
            let mut accepted = 0;
            for _ in 0..moves {
                accepted += try_move(&mut st, &mut rng, window, temp) as usize;
            }
            let rate = accepted as f64 / moves as f64;
            window = (window * (1.0 - 0.44 + rate)).clamp(1.0, w.max(h) as f64);
            temp *= if rate > 0.8 { 0.6 } else if rate > 0.15 { 0.9 } else { 0.7 };
            if temp < 0.005 * total(&st).max(1.0) / st.nets.len() as f64 {
                break;
            }
        }
        // A last pass of only improvements.
        for _ in 0..moves {
            try_move(&mut st, &mut rng, 2.0, 1e-9);
        }
    }

    // ---- places in tiles ----
    let mut at = vec![(0, 0, 0); n];
    let mut taken: FxMap<usize, Vec<bool>> = FxMap::default();
    for ch in &m.chains {
        for (k, &l) in ch.lcs.iter().enumerate() {
            let t = st.tile_of[l as usize];
            let (x, y) = st.tiles[t];
            at[l as usize] = (x, y, k % 8);
            taken.entry(t).or_insert_with(|| vec![false; 8])[k % 8] = true;
        }
    }
    for t in 0..st.tiles.len() {
        let slots = taken.entry(t).or_insert_with(|| vec![false; 8]);
        let (x, y) = st.tiles[t];
        let mut cells = st.free[t].clone();
        cells.sort();
        for c in cells {
            let z = slots.iter().position(|&s| !s).ok_or("a tile over full")?;
            slots[z] = true;
            at[c as usize] = (x, y, z);
        }
    }
    let p = Placement { at, inputs, outputs, cost: total(&st) };
    check(db, m, &p)?;
    Ok(p)
}

/// Whether a placement is legal: every cell in a logic tile, alone in its
/// place; the flip-flops of a tile on one enable and set/reset; each
/// chain's cells consecutive from place 0 of its first tile, upwards.
pub fn check(db: &Chipdb, m: &Mapped, p: &Placement) -> Result<(), String> {
    let logic: Vec<(usize, usize)> = db.tiles.iter().filter(|t| t.0 == "logic").map(|t| (t.1, t.2)).collect();
    let mut seen: FxMap<(usize, usize, usize), u32> = FxMap::default();
    let mut ctrl: FxMap<(usize, usize), (Option<Src>, Option<Src>)> = FxMap::default();
    for (i, &(x, y, z)) in p.at.iter().enumerate() {
        if !logic.contains(&(x, y)) || z > 7 {
            return Err(format!("{} is at {x} {y} {z}, not a logic cell", m.lcs[i].name));
        }
        if let Some(o) = seen.insert((x, y, z), i as u32) {
            return Err(format!("{} and {} are both at {x} {y} {z}", m.lcs[o as usize].name, m.lcs[i].name));
        }
        if let Some(f) = &m.lcs[i].ff {
            let k = (f.en, f.sr);
            if *ctrl.entry((x, y)).or_insert(k) != k {
                return Err(format!("tile {x} {y} has flip-flops on two enables or set/resets"));
            }
        }
    }
    for ch in &m.chains {
        let (x0, y0, _) = p.at[ch.lcs[0] as usize];
        for (k, &l) in ch.lcs.iter().enumerate() {
            if p.at[l as usize] != (x0, y0 + k / 8, k % 8) {
                return Err(format!("{} is out of its chain's order", m.lcs[l as usize].name));
            }
        }
    }
    Ok(())
}
