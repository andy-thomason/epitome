//! Routing by negotiated congestion (PathFinder: McMurchie and Ebeling,
//! 1995) over the chip database's switches ([`Graph`]).
//!
//! Each signal is a tree from its source net to its sinks, grown one sink
//! at a time by the cheapest path from any net already in the tree
//! ([`Graph::search`]). A net costs `(1 + history) * (1 + present * use)`,
//! where `use` counts the other signals on it: at first signals share nets
//! freely, then each pass rips every signal up and routes it again with
//! sharing dearer (`present` grows) and nets that were fought over dearer
//! for good (`history`), until no net carries two signals.
//!
//! A sink may be any of a LUT's inputs not taken by another signal: the
//! LUT's table is then permuted to match ([`Routed::pins`]).

use super::chipdb::Bit;
use super::icebox::Graph;
use crate::hash::{FxMap, FxSet};

#[derive(Clone, Debug)]
pub enum Sink {
    Net(usize),
    /// Input `input` of a LUT, on any of the nets `pins` (`in_0` to
    /// `in_3`) that is free; `lut` names the LUT.
    Lut { lut: u32, input: usize, pins: [usize; 4] },
}

#[derive(Clone, Debug)]
pub struct Request {
    pub name: String,
    pub src: usize,
    pub sinks: Vec<Sink>,
}

#[derive(Clone, Debug, Default)]
pub struct Routed {
    /// Each request's steps, `(dst, src, index in src's fanout)`.
    pub paths: Vec<Vec<(usize, usize, usize)>>,
    /// For each LUT input routed to one of several pins: the pin (0..4).
    pub pins: FxMap<(u32, usize), usize>,
    pub passes: usize,
}

impl Routed {
    pub fn bits(&self, g: &Graph) -> Vec<(usize, usize, Bit)> {
        self.paths.iter().flatten().flat_map(|&e| g.edge_bits(e)).collect()
    }

    /// Nets used, all requests together.
    pub fn nets_used(&self) -> usize {
        self.paths.iter().map(|p| p.len()).sum()
    }
}

/// Routes `reqs`, never entering `blocked`.
pub fn pathfinder(g: &Graph, reqs: &[Request], blocked: &FxSet<usize>, max_passes: usize) -> Result<Routed, String> {
    let mut occ: FxMap<usize, u32> = FxMap::default();
    let mut hist: FxMap<usize, u32> = FxMap::default();
    let mut paths: Vec<Vec<(usize, usize, usize)>> = vec![Vec::new(); reqs.len()];
    let mut claims: FxMap<usize, usize> = FxMap::default();
    let mut pins: FxMap<(u32, usize), usize> = FxMap::default();
    let mut present = 0.5f64;
    // Requests with more sinks first.
    let mut order: Vec<usize> = (0..reqs.len()).collect();
    order.sort_by_key(|&i| std::cmp::Reverse(reqs[i].sinks.len()));
    for pass in 1..=max_passes {
        for &i in &order {
            let r = &reqs[i];
            // Rip up.
            for &(dst, ..) in &paths[i] {
                *occ.get_mut(&dst).unwrap() -= 1;
            }
            paths[i].clear();
            claims.retain(|_, &mut owner| owner != i);
            let mut tree: Vec<usize> = vec![r.src];
            let mut in_tree: FxSet<usize> = FxSet::default();
            in_tree.insert(r.src);
            for sink in &r.sinks {
                let targets: Vec<usize> = match sink {
                    Sink::Net(n) => vec![*n],
                    Sink::Lut { pins, .. } => pins.iter().copied().filter(|p| !claims.contains_key(p) && !in_tree.contains(p)).collect(),
                };
                if let Sink::Net(n) = sink {
                    if in_tree.contains(n) {
                        continue;
                    }
                }
                let cost = |n: usize| -> Option<u64> {
                    if blocked.contains(&n) {
                        return None;
                    }
                    let o = occ.get(&n).copied().unwrap_or(0) as f64;
                    let h = hist.get(&n).copied().unwrap_or(0) as f64;
                    Some(((1.0 + h) * (1.0 + present * o) * 100.0) as u64)
                };
                let path = g
                    .search(&tree, &|n| targets.contains(&n), &cost)
                    .ok_or_else(|| format!("{}: no route to a sink (net {:?})", r.name, sink))?;
                for &(dst, ..) in &path {
                    *occ.entry(dst).or_default() += 1;
                    in_tree.insert(dst);
                    tree.push(dst);
                }
                let end = path.last().map_or(r.src, |e| e.0);
                if let Sink::Lut { lut, input, pins: p } = sink {
                    claims.insert(end, i);
                    pins.insert((*lut, *input), p.iter().position(|&x| x == end).unwrap());
                }
                paths[i].extend(path);
            }
        }
        let over: Vec<usize> = occ.iter().filter(|&(_, &o)| o > 1).map(|(&n, _)| n).collect();
        if over.is_empty() {
            return Ok(Routed { paths, pins, passes: pass });
        }
        for n in over {
            *hist.entry(n).or_default() += occ[&n] - 1;
        }
        present *= 1.8;
    }
    Err(format!("routing did not converge in {max_passes} passes"))
}

/// What the configured bits connect: for each net, the net that drives it
/// through a switch whose bits are set to that source's values.
pub fn drivers(g: &Graph, bits: &FxSet<(usize, usize, Bit)>) -> Result<FxMap<usize, usize>, String> {
    let mut out: FxMap<usize, usize> = FxMap::default();
    for src in 0..g.nets() {
        for (dst, x, y, sb, v) in g.fanout(src) {
            let on = sb.iter().zip(v.bytes()).all(|(b, c)| (c == b'1') == bits.contains(&(*x, *y, *b)));
            if on && v.contains('1') {
                if let Some(&o) = out.get(dst) {
                    if o != src {
                        return Err(format!("net {dst} driven from both {o} and {src}"));
                    }
                }
                out.insert(*dst, src);
            }
        }
    }
    Ok(out)
}

/// Follows drivers back from `n` to the net nothing drives.
pub fn source_of(drv: &FxMap<usize, usize>, mut n: usize) -> usize {
    let mut steps = 0;
    while let Some(&p) = drv.get(&n) {
        n = p;
        steps += 1;
        if steps > 10_000 {
            break;
        }
    }
    n
}
