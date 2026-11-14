//! Technology mapping: a synthesised [`Netlist`] onto iCE40 logic cells
//! (`logic_cell.sv`: a LUT4, a carry, a flip-flop).
//!
//! - **Carry chains.** Majority nodes that follow one another (one's
//!   output is another's input) become a chain of cells, each computing
//!   `co = maj(in_1, in_2, ci)` with its carry in from the cell below. A
//!   chain starts from a constant carry in (0, or 1 with the tile's
//!   `CarryInSet`), or from a signal through a feeder cell (`in_1` the
//!   signal, carry in 1: `maj(s, 0, 1) = s`). A lone majority is ordinary
//!   logic.
//! - **LUTs.** The other gates are covered by 4-input cuts: each node's
//!   cuts are enumerated from its inputs' (at most [`CUTS`] kept), the
//!   cover is chosen for depth first and then, keeping that depth, for
//!   area flow (each leaf's area shared among its fanout).
//! - **Carries as LUT inputs.** A cell's LUT can read the carry coming in
//!   on `in_3`, so a LUT whose only carry leaf is the carry into a cell
//!   (an adder's sum) goes into that cell, its other leaves on `in_1` and
//!   `in_2` where they are the cell's carry inputs, and on `in_0`. A carry
//!   read any other way is brought out through the LUT of the cell above
//!   it (one more cell after a chain's last), and read as a signal.
//! - **Flip-flops** go into the cell of the LUT that feeds them when
//!   nothing else reads that LUT, else into a cell of their own whose LUT
//!   passes the input through. An enable and an asynchronous reset are
//!   the cell's `cen` and `sr`, which a tile's eight cells share (see
//!   [`super::place`]).

use super::super::super::synth::{F, Lit, Netlist, Node, Port, T, inv, lit, node};
use crate::hash::FxMap;

/// Most cuts kept for each node.
pub const CUTS: usize = 10;

/// Where a signal comes from: an input pin, or a cell's output.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, PartialOrd, Ord)]
pub enum Src {
    Pin { port: u32, bit: u32 },
    Lc(u32),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FfCfg {
    pub en: Option<Src>,
    pub sr: Option<Src>,
    /// `AsyncSetReset`: `sr` acts at once, not on the clock.
    pub async_sr: bool,
    /// `Set_NoReset`: `sr` sets to 1.
    pub set: bool,
}

/// A logic cell.
#[derive(Clone, Debug)]
pub struct Lc {
    pub name: String,
    /// Bit `i` is the output for inputs `i` (bit `j` of `i` is `in_j`).
    pub lut: u16,
    pub ins: [Option<Src>; 4],
    /// `CarryEnable`: its carry out, `maj(in_1, in_2, ci)`, goes to the
    /// next cell of its chain.
    pub carry: bool,
    /// `in_3` is the carry in (from the cell below in the chain).
    pub carry_in3: bool,
    pub ff: Option<FfCfg>,
}

impl Lc {
    fn new(name: String) -> Lc {
        Lc { name, lut: 0, ins: [None; 4], carry: false, carry_in3: false, ff: None }
    }
}

/// A carry chain: its cells, bottom first, and its carry in.
#[derive(Clone, Debug)]
pub struct Chain {
    pub lcs: Vec<u32>,
    pub ci_one: bool,
}

#[derive(Clone, Debug)]
pub struct Mapped {
    pub lcs: Vec<Lc>,
    pub chains: Vec<Chain>,
    pub inputs: Vec<Port>,
    /// Each output port's name, width and drivers.
    pub outputs: Vec<(String, Vec<Src>)>,
    /// The clock input, and whether flip-flops take its falling edge.
    pub clock: Option<(u32, bool)>,
}

const VAR: [u16; 4] = [0xaaaa, 0xcccc, 0xf0f0, 0xff00];

/// `tt`, a function of `from`, as a function of `to` (which holds them).
fn expand(tt: u16, from: &[u32], to: &[u32]) -> u16 {
    let pos: Vec<usize> = from.iter().map(|l| to.iter().position(|x| x == l).unwrap()).collect();
    let mut out = 0u16;
    for m in 0..16 {
        let i = pos.iter().enumerate().fold(0, |i, (j, &p)| i | ((m >> p) & 1) << j);
        if tt >> i & 1 != 0 {
            out |= 1 << m;
        }
    }
    out
}

#[derive(Clone, Debug)]
struct Cut {
    leaves: Vec<u32>,
    tt: u16,
    depth: u32,
    flow: f32,
}

/// A LUT to make: what it computes (a literal), its leaves and table.
#[derive(Clone, Debug)]
struct Lut {
    id: Lit,
    leaves: Vec<u32>,
    tt: u16,
}

fn ins_of(k: &Node) -> Vec<Lit> {
    match *k {
        Node::And(a, b) | Node::Xor(a, b) => vec![a, b],
        Node::Maj(a, b, c) => vec![a, b, c],
        _ => vec![],
    }
}

fn apply(k: &Node, t: &[u16]) -> u16 {
    match k {
        Node::And(..) => t[0] & t[1],
        Node::Xor(..) => t[0] ^ t[1],
        Node::Maj(..) => t[0] & t[1] | (t[0] | t[1]) & t[2],
        _ => unreachable!(),
    }
}

pub fn map(n: &Netlist) -> Result<Mapped, String> {
    let nn = n.nodes.len();
    let clock = n.clock()?.map(|(l, neg)| match n.nodes[node(l)] {
        Node::Input { port, .. } => (port, neg),
        _ => unreachable!(),
    });
    if let Some(f) = n.ffs.iter().find(|f| f.init) {
        return Err(format!("{} starts at 1: iCE40 flip-flops start at 0", f.name));
    }

    // ---- carry chains ----
    let is_maj = |i: usize| matches!(n.nodes[i], Node::Maj(..));
    let mut pred: Vec<Option<u32>> = vec![None; nn];
    let mut succ: Vec<Option<u32>> = vec![None; nn];
    for i in 0..nn {
        if let Node::Maj(a, b, c) = n.nodes[i] {
            for l in [a, b, c] {
                if !inv(l) && is_maj(node(l)) && succ[node(l)].is_none() {
                    pred[i] = Some(node(l) as u32);
                    succ[node(l)] = Some(i as u32);
                    break;
                }
            }
        }
    }
    // Chains of majority nodes, and each one's place.
    let mut chains: Vec<Vec<u32>> = Vec::new();
    let mut in_chain: Vec<Option<(u32, u32)>> = vec![None; nn];
    for i in 0..nn {
        if is_maj(i) && pred[i].is_none() && succ[i].is_some() {
            let mut c = vec![i as u32];
            while let Some(s) = succ[*c.last().unwrap() as usize] {
                c.push(s);
            }
            for (k, &m) in c.iter().enumerate() {
                in_chain[m as usize] = Some((chains.len() as u32, k as u32));
            }
            chains.push(c);
        }
    }
    // Each chain cell's two carry inputs (literals; F for none), and each
    // chain's carry in: a constant, or a signal through a feeder.
    let mut cell_ins: FxMap<u32, [Lit; 2]> = FxMap::default();
    let mut chain_ci: Vec<(bool, Option<Lit>)> = Vec::new();
    for c in &chains {
        for (k, &m) in c.iter().enumerate() {
            let ins = ins_of(&n.nodes[m as usize]);
            if k > 0 {
                let p = lit(c[k - 1] as usize, false);
                let pos = ins.iter().position(|&l| l == p).unwrap();
                let o: Vec<Lit> = ins.iter().enumerate().filter(|&(j, _)| j != pos).map(|(_, &l)| l).collect();
                cell_ins.insert(m, [o[0], o[1]]);
            } else if let Some(pos) = ins.iter().position(|&l| l == F || l == T) {
                let o: Vec<Lit> = ins.iter().enumerate().filter(|&(j, _)| j != pos).map(|(_, &l)| l).collect();
                cell_ins.insert(m, [o[0], o[1]]);
                chain_ci.push((ins[pos] == T, None));
            } else {
                cell_ins.insert(m, [ins[0], ins[1]]);
                chain_ci.push((true, Some(ins[2])));
            }
        }
    }
    let is_gate = |i: usize| matches!(n.nodes[i], Node::And(..) | Node::Xor(..)) || is_maj(i) && in_chain[i].is_none();

    // ---- what must be a signal ----
    let mut roots: Vec<Lit> = Vec::new();
    for p in &n.outputs {
        roots.extend(p.bits.iter().copied());
    }
    for f in &n.ffs {
        roots.push(f.d);
        roots.extend([f.en, f.arst].into_iter().filter(|&l| l != T && l != F));
    }
    for ins in cell_ins.values() {
        roots.extend(ins.iter().copied());
    }
    for (_, s) in &chain_ci {
        roots.extend(s.iter().copied());
    }
    roots.retain(|&l| l != T && l != F);

    // ---- cuts ----
    let mut refs = vec![0f32; nn];
    for i in 0..nn {
        if is_gate(i) {
            for l in ins_of(&n.nodes[i]) {
                refs[node(l)] += 1.0;
            }
        }
    }
    for &r in &roots {
        refs[node(r)] += 1.0;
    }
    let mut cuts: Vec<Vec<Cut>> = vec![Vec::new(); nn];
    let mut best: Vec<usize> = vec![0; nn];
    let trivial = |i: usize| Cut { leaves: vec![i as u32], tt: VAR[0], depth: 0, flow: 0.0 };
    let leaf_cost = |cuts: &Vec<Vec<Cut>>, best: &Vec<usize>, l: u32| -> (u32, f32) {
        if is_gate(l as usize) {
            let c = &cuts[l as usize][best[l as usize]];
            (c.depth, c.flow / refs[l as usize].max(1.0))
        } else {
            (0, 0.0)
        }
    };
    for pass in 0..2 {
        let mut required = vec![u32::MAX; nn];
        if pass == 1 {
            // Required times of the depth-optimal cover.
            let mut depth = 0;
            for &r in &roots {
                if is_gate(node(r)) {
                    depth = depth.max(cuts[node(r)][best[node(r)]].depth);
                }
            }
            for &r in &roots {
                required[node(r)] = depth;
            }
            for i in (0..nn).rev() {
                if is_gate(i) && required[i] != u32::MAX {
                    let c = &cuts[i][best[i]];
                    for &l in &c.leaves {
                        required[l as usize] = required[l as usize].min(required[i].saturating_sub(1));
                    }
                }
            }
        }
        for i in 0..nn {
            if !is_gate(i) {
                cuts[i] = vec![trivial(i)];
                best[i] = 0;
                continue;
            }
            let k = &n.nodes[i];
            let fan = ins_of(k);
            let mut found: Vec<Cut> = Vec::new();
            let lists: Vec<Vec<Cut>> = fan
                .iter()
                .map(|&l| {
                    let j = node(l);
                    let mut v: Vec<Cut> = if is_gate(j) {
                        cuts[j].iter().filter(|c| c.leaves != [j as u32]).cloned().collect()
                    } else {
                        Vec::new()
                    };
                    v.push(trivial(j));
                    if inv(l) {
                        for c in &mut v {
                            c.tt = !c.tt;
                        }
                    }
                    v
                })
                .collect();
            let mut idx = vec![0usize; lists.len()];
            'combos: loop {
                let parts: Vec<&Cut> = idx.iter().enumerate().map(|(j, &x)| &lists[j][x]).collect();
                let mut leaves: Vec<u32> = parts.iter().flat_map(|c| c.leaves.iter().copied()).collect();
                leaves.sort();
                leaves.dedup();
                if leaves.len() <= 4 {
                    let tts: Vec<u16> = parts.iter().map(|c| expand(c.tt, &c.leaves, &leaves)).collect();
                    let tt = apply(k, &tts);
                    if !found.iter().any(|c| c.leaves == leaves) {
                        let (mut d, mut a) = (0, 1.0);
                        for &l in &leaves {
                            let (ld, la) = leaf_cost(&cuts, &best, l);
                            d = d.max(ld);
                            a += la;
                        }
                        found.push(Cut { leaves, tt, depth: d + 1, flow: a });
                    }
                }
                for j in 0..idx.len() {
                    idx[j] += 1;
                    if idx[j] < lists[j].len() {
                        continue 'combos;
                    }
                    idx[j] = 0;
                }
                break;
            }
            if pass == 0 || required[i] == u32::MAX {
                found.sort_by(|a, b| {
                    (a.depth, a.flow, a.leaves.len()).partial_cmp(&(b.depth, b.flow, b.leaves.len())).unwrap()
                });
            } else {
                let r = required[i];
                found.sort_by(|a, b| {
                    ((a.depth > r), a.flow, a.depth, a.leaves.len())
                        .partial_cmp(&((b.depth > r), b.flow, b.depth, b.leaves.len()))
                        .unwrap()
                });
            }
            found.truncate(CUTS);
            cuts[i] = found;
            best[i] = 0;
        }
    }

    // ---- the cover ----
    let mut luts: Vec<Lut> = Vec::new();
    let mut lut_of: FxMap<u32, usize> = FxMap::default();
    let mut stack: Vec<u32> = roots.iter().map(|&r| node(r) as u32).filter(|&i| is_gate(i as usize)).collect();
    while let Some(i) = stack.pop() {
        if lut_of.contains_key(&i) {
            continue;
        }
        let c = &cuts[i as usize][best[i as usize]];
        lut_of.insert(i, luts.len());
        luts.push(Lut { id: lit(i as usize, false), leaves: c.leaves.clone(), tt: c.tt });
        stack.extend(c.leaves.iter().copied().filter(|&l| is_gate(l as usize)));
    }
    // Who reads each LUT: as a leaf, and as a root.
    let mut leaf_users = vec![0u32; nn];
    for l in &luts {
        for &x in &l.leaves {
            leaf_users[x as usize] += 1;
        }
    }
    let mut pos_roots = vec![0u32; nn];
    let mut neg_roots = vec![0u32; nn];
    for &r in &roots {
        if inv(r) { neg_roots[node(r)] += 1 } else { pos_roots[node(r)] += 1 }
    }
    // Roots of the inverse polarity, and roots on carries, inputs and
    // flip-flops that need a LUT: buffers and inverters.
    let mut sig_lut: FxMap<Lit, usize> = FxMap::default();
    for (i, l) in luts.iter_mut().enumerate() {
        let x = node(l.id);
        if pos_roots[x] == 0 && leaf_users[x] == 0 && neg_roots[x] > 0 {
            l.id ^= 1;
            l.tt = !l.tt;
        }
        sig_lut.insert(l.id, i);
    }
    let mut extra = Vec::new();
    let mut want: Vec<Lit> = roots.clone();
    want.sort();
    want.dedup();
    for &r in &want {
        let x = node(r);
        let native = matches!(n.nodes[x], Node::Input { .. } | Node::Ff(_)) && !inv(r);
        if native || sig_lut.contains_key(&r) {
            continue;
        }
        if let Some(&j) = sig_lut.get(&(r ^ 1)) {
            if is_gate(x) {
                let l = &luts[j];
                extra.push(Lut { id: r, leaves: l.leaves.clone(), tt: !l.tt });
                continue;
            }
        }
        extra.push(Lut { id: r, leaves: vec![x as u32], tt: if inv(r) { !VAR[0] } else { VAR[0] } });
    }
    for l in extra {
        for &x in &l.leaves {
            leaf_users[x as usize] += 1;
        }
        sig_lut.insert(l.id, luts.len());
        luts.push(l);
    }

    // ---- carries read as LUT inputs ----
    // The cell a carry comes into: the next in its chain, or one added.
    let chain_leaves =
        |l: &Lut| -> Vec<u32> { l.leaves.iter().copied().filter(|&x| in_chain[x as usize].is_some()).collect() };
    let mut readers: FxMap<u32, Vec<usize>> = FxMap::default();
    for (i, l) in luts.iter().enumerate() {
        for c in chain_leaves(l) {
            readers.entry(c).or_default().push(i);
        }
    }
    // A LUT absorbed into the cell its carry comes into: its pins.
    let mut absorbed: FxMap<usize, (u32, [Option<u32>; 4])> = FxMap::default();
    let mut exposed: Vec<u32> = Vec::new();
    let mut carries: Vec<u32> = readers.keys().copied().collect();
    carries.sort();
    for p in carries {
        let rs = &readers[&p];
        let host = succ[p as usize];
        let fits = |l: &Lut| -> Option<[Option<u32>; 4]> {
            if chain_leaves(l).len() != 1 {
                return None;
            }
            let mut pins = [None; 4];
            pins[3] = Some(p);
            let mut free: Vec<usize> = match host {
                Some(_) => vec![0],
                None => vec![0, 1, 2],
            };
            for &x in l.leaves.iter().filter(|&&x| x != p) {
                if let Some(h) = host {
                    if let Some(j) = cell_ins[&h].iter().position(|&c| c != F && c != T && node(c) as u32 == x) {
                        pins[j + 1] = Some(x);
                        continue;
                    }
                }
                pins[free.pop()?] = Some(x);
            }
            Some(pins)
        };
        match rs.as_slice() {
            [one] if fits(&luts[*one]).is_some() => {
                absorbed.insert(*one, (p, fits(&luts[*one]).unwrap()));
            }
            _ => exposed.push(p),
        }
    }
    // An absorbed LUT whose carry is also exposed reads the exposed signal.
    absorbed.retain(|_, (p, _)| !exposed.contains(p));

    // ---- cells ----
    let mut lcs: Vec<Lc> = Vec::new();
    let mut out_chains: Vec<Chain> = Vec::new();
    // The cell a carry comes into, and the cell a LUT is in.
    let mut host_lc: FxMap<u32, u32> = FxMap::default();
    let mut lut_lc: Vec<Option<u32>> = vec![None; luts.len()];
    let mut feeders: Vec<(u32, Lit)> = Vec::new();
    for (ci, c) in chains.iter().enumerate() {
        let mut ids = Vec::new();
        let (one, feed) = chain_ci[ci];
        if let Some(s) = feed {
            let mut lc = Lc::new(format!("carry feeder {}", ci));
            lc.carry = true;
            ids.push(lcs.len() as u32);
            feeders.push((lcs.len() as u32, s));
            lcs.push(lc);
        }
        for (k, &m) in c.iter().enumerate() {
            let mut lc = Lc::new(format!("carry {ci}.{k}"));
            lc.carry = true;
            if k > 0 {
                host_lc.insert(c[k - 1], lcs.len() as u32);
            }
            ids.push(lcs.len() as u32);
            lcs.push(lc);
            let _ = m;
        }
        let last = *c.last().unwrap();
        if readers.contains_key(&last) {
            host_lc.insert(last, lcs.len() as u32);
            ids.push(lcs.len() as u32);
            lcs.push(Lc::new(format!("carry {ci} out")));
        }
        out_chains.push(Chain { lcs: ids, ci_one: one });
    }
    for (&i, &(p, _)) in &absorbed {
        lut_lc[i] = Some(host_lc[&p]);
    }
    for (i, l) in luts.iter().enumerate() {
        if lut_lc[i].is_none() {
            lut_lc[i] = Some(lcs.len() as u32);
            lcs.push(Lc::new(format!("lut {}", node(l.id))));
        }
    }
    let mut const_lc: [Option<u32>; 2] = [None, None];
    let mut constant = |v: bool, lcs: &mut Vec<Lc>| -> Src {
        let slot = &mut const_lc[v as usize];
        if slot.is_none() {
            let mut lc = Lc::new(format!("constant {}", v as u8));
            lc.lut = if v { 0xffff } else { 0 };
            *slot = Some(lcs.len() as u32);
            lcs.push(lc);
        }
        Src::Lc(slot.unwrap())
    };
    // Flip-flops: into the LUT's cell where only they read it.
    let mut ff_lc: Vec<u32> = Vec::new();
    let mut pass_through: Vec<(u32, Lit)> = Vec::new();
    for f in &n.ffs {
        let x = node(f.d);
        let packable = sig_lut.get(&f.d).copied().filter(|&j| {
            let users = leaf_users[x] + pos_roots[x] + neg_roots[x];
            users == 1
                && lcs[lut_lc[j].unwrap() as usize].ff.is_none()
                && !exposed.iter().any(|&p| host_lc.get(&p) == lut_lc[j].as_ref())
        });
        let at = match packable {
            Some(j) => lut_lc[j].unwrap(),
            None => {
                lcs.push(Lc::new(format!("ff {}", f.name)));
                pass_through.push((lcs.len() as u32 - 1, f.d));
                lcs.len() as u32 - 1
            }
        };
        lcs[at as usize].name = f.name.clone();
        lcs[at as usize].ff = Some(FfCfg { en: None, sr: None, async_sr: f.arst != F, set: f.arst_value });
        ff_lc.push(at);
    }

    // ---- signals ----
    let signal = |l: Lit| -> Option<Src> {
        let x = node(l);
        match n.nodes[x] {
            Node::Input { port, bit } if !inv(l) => Some(Src::Pin { port, bit }),
            Node::Ff(f) if !inv(l) => Some(Src::Lc(ff_lc[f as usize])),
            Node::Maj(..) if !inv(l) && exposed.contains(&(x as u32)) => Some(Src::Lc(host_lc[&(x as u32)])),
            _ => sig_lut.get(&l).map(|&j| Src::Lc(lut_lc[j].unwrap())),
        }
    };
    // A leaf's signal in either polarity: the source and whether inverted.
    let leaf = |x: u32| -> Result<(Src, bool), String> {
        let p = lit(x as usize, false);
        signal(p)
            .map(|s| (s, false))
            .or_else(|| signal(p ^ 1).map(|s| (s, true)))
            .ok_or_else(|| format!("no signal for node {x}"))
    };
    let exact = |l: Lit| -> Result<Src, String> { signal(l).ok_or_else(|| format!("no signal for literal {l}")) };

    // Fills a cell's LUT: `tt` of `leaves`, leaf j on pin `pins[j]`.
    let fill = |lc: &mut Lc, tt: u16, leaves: &[u32], pins: &[usize], carry_leaf: Option<u32>| -> Result<(), String> {
        let mut flip = vec![false; leaves.len()];
        for (j, &x) in leaves.iter().enumerate() {
            if Some(x) == carry_leaf {
                lc.carry_in3 = true;
                continue;
            }
            // A pin already set (a carry input) gives its polarity.
            flip[j] = match lc.ins[pins[j]] {
                Some(t) => match (signal(lit(x as usize, false)), signal(lit(x as usize, true))) {
                    (Some(s), _) if s == t => false,
                    (_, Some(s)) if s == t => true,
                    _ => return Err(format!("pin {} of {} wanted twice", pins[j], lc.name)),
                },
                None => {
                    let (s, f) = leaf(x)?;
                    lc.ins[pins[j]] = Some(s);
                    f
                }
            };
        }
        let mut t = 0u16;
        for m in 0..16 {
            let i = (0..leaves.len()).fold(0, |i, j| i | (((m >> pins[j]) & 1) ^ flip[j] as usize) << j);
            if tt >> i & 1 != 0 {
                t |= 1 << m;
            }
        }
        lc.lut = t;
        Ok(())
    };

    for (ci, c) in chains.iter().enumerate() {
        let ids = &out_chains[ci].lcs;
        let off = chain_ci[ci].1.is_some() as usize;
        for (k, &m) in c.iter().enumerate() {
            let at = ids[k + off] as usize;
            for (j, &l) in cell_ins[&m].iter().enumerate() {
                lcs[at].ins[j + 1] = match l {
                    F => None,
                    T => Some(constant(true, &mut lcs)),
                    _ => Some(exact(l)?),
                };
            }
        }
    }
    for &(at, s) in &feeders {
        lcs[at as usize].ins[1] = Some(exact(s)?);
    }
    for &p in &exposed {
        let at = host_lc[&p] as usize;
        lcs[at].carry_in3 = true;
        lcs[at].lut = VAR[3];
    }
    for (i, l) in luts.iter().enumerate() {
        let at = lut_lc[i].unwrap() as usize;
        match absorbed.get(&i) {
            Some(&(p, pins)) => {
                let pin_of: Vec<usize> =
                    l.leaves.iter().map(|x| pins.iter().position(|q| *q == Some(*x)).unwrap()).collect();
                fill(&mut lcs[at], l.tt, &l.leaves, &pin_of, Some(p))?;
            }
            None => {
                let pins: Vec<usize> = (0..l.leaves.len()).collect();
                fill(&mut lcs[at], l.tt, &l.leaves, &pins, None)?;
            }
        }
    }
    for &(at, d) in &pass_through {
        match d {
            F | T => lcs[at as usize].lut = if d == T { 0xffff } else { 0 },
            _ => {
                let tt = if inv(d) { !VAR[0] } else { VAR[0] };
                fill(&mut lcs[at as usize], tt, &[node(d) as u32], &[0], None)?;
            }
        }
    }
    for (fi, f) in n.ffs.iter().enumerate() {
        let at = ff_lc[fi] as usize;
        let en = if f.en == T {
            None
        } else if f.en == F {
            Some(constant(false, &mut lcs))
        } else {
            Some(exact(f.en)?)
        };
        let sr = if f.arst == F { None } else { Some(exact(f.arst)?) };
        let cfg = lcs[at].ff.as_mut().unwrap();
        cfg.en = en;
        cfg.sr = sr;
    }
    let mut outputs = Vec::new();
    for p in &n.outputs {
        let mut v = Vec::new();
        for &b in &p.bits {
            v.push(match b {
                F | T => constant(b == T, &mut lcs),
                _ => exact(b)?,
            });
        }
        outputs.push((p.name.clone(), v));
    }
    Ok(Mapped { lcs, chains: out_chains, inputs: n.inputs.clone(), outputs, clock })
}

impl Mapped {
    pub fn stats(&self) -> String {
        let luts = self.lcs.iter().filter(|l| l.ins.iter().any(|i| i.is_some()) || l.carry_in3 || l.lut != 0).count();
        let carry = self.lcs.iter().filter(|l| l.carry).count();
        let ffs = self.lcs.iter().filter(|l| l.ff.is_some()).count();
        format!(
            "{} logic cells: {luts} LUTs, {carry} carries in {} chains, {ffs} flip-flops",
            self.lcs.len(),
            self.chains.len()
        )
    }

    /// The chain a cell is in, and its place there.
    pub fn chain_of(&self) -> Vec<Option<(u32, u32)>> {
        let mut v = vec![None; self.lcs.len()];
        for (c, ch) in self.chains.iter().enumerate() {
            for (k, &l) in ch.lcs.iter().enumerate() {
                v[l as usize] = Some((c as u32, k as u32));
            }
        }
        v
    }
}

/// A run of the mapped cells, as `logic_cell.sv` describes them, 64
/// simulations at once.
pub struct LcSim<'m> {
    pub m: &'m Mapped,
    pub out: Vec<u64>,
    pub cout: Vec<u64>,
    pub state: Vec<u64>,
    pub inputs: Vec<Vec<u64>>,
    /// The cell below each one in its chain, and each chain's first
    /// cell's carry in.
    below: Vec<Option<u32>>,
    ci: Vec<u64>,
}

impl<'m> LcSim<'m> {
    pub fn new(m: &'m Mapped) -> LcSim<'m> {
        let mut below = vec![None; m.lcs.len()];
        let mut ci = vec![0; m.lcs.len()];
        for ch in &m.chains {
            for (k, &l) in ch.lcs.iter().enumerate() {
                if k > 0 {
                    below[l as usize] = Some(ch.lcs[k - 1]);
                } else if ch.ci_one {
                    ci[l as usize] = !0;
                }
            }
        }
        let n = m.lcs.len();
        LcSim {
            m,
            out: vec![0; n],
            cout: vec![0; n],
            state: vec![0; n],
            inputs: m.inputs.iter().map(|p| vec![0; p.width as usize]).collect(),
            below,
            ci,
        }
    }

    pub fn get(&self, s: Src) -> u64 {
        match s {
            Src::Pin { port, bit } => self.inputs[port as usize][bit as usize],
            Src::Lc(i) => self.out[i as usize],
        }
    }

    fn lut(&self, i: usize) -> (u64, u64) {
        let lc = &self.m.lcs[i];
        let cin = match self.below[i] {
            Some(b) => self.cout[b as usize],
            None => self.ci[i],
        };
        let v: Vec<u64> =
            (0..4).map(|k| if k == 3 && lc.carry_in3 { cin } else { lc.ins[k].map_or(0, |s| self.get(s)) }).collect();
        let mut o = 0u64;
        for m in 0..16 {
            if lc.lut >> m & 1 != 0 {
                let mut sel = !0u64;
                for (k, &x) in v.iter().enumerate() {
                    sel &= if m >> k & 1 != 0 { x } else { !x };
                }
                o |= sel;
            }
        }
        let co = if lc.carry { v[1] & v[2] | (v[1] | v[2]) & cin } else { 0 };
        (o, co)
    }

    /// Settles the cells (asynchronous resets first).
    pub fn eval(&mut self) {
        for _ in 0..2 {
            for _ in 0..=self.m.lcs.len() {
                let mut changed = false;
                for i in 0..self.m.lcs.len() {
                    let (o, co) = self.lut(i);
                    let o = if self.m.lcs[i].ff.is_some() { self.state[i] } else { o };
                    changed |= o != self.out[i] || co != self.cout[i];
                    self.out[i] = o;
                    self.cout[i] = co;
                }
                if !changed {
                    break;
                }
            }
            let mut again = false;
            for (i, lc) in self.m.lcs.iter().enumerate() {
                if let Some(FfCfg { sr: Some(s), async_sr: true, set, .. }) = &lc.ff {
                    let r = self.get(*s);
                    let v = self.state[i] & !r | if *set { r } else { 0 };
                    again |= v != self.state[i];
                    self.state[i] = v;
                }
            }
            if !again {
                break;
            }
        }
    }

    pub fn clock(&mut self) {
        let next: Vec<u64> = (0..self.m.lcs.len())
            .map(|i| match &self.m.lcs[i].ff {
                None => 0,
                Some(f) => {
                    let d = self.lut(i).0;
                    let en = f.en.map_or(!0, |s| self.get(s));
                    let r = f.sr.map_or(0, |s| self.get(s));
                    let set = if f.set { !0 } else { 0 };
                    let v = if f.async_sr { d } else { d & !r | set & r };
                    let v = self.state[i] & !en | v & en;
                    if f.async_sr { v & !r | set & r } else { v }
                }
            })
            .collect();
        self.state = next;
        self.eval();
    }
}

/// The mapped cells' lines for the stimulus, as
/// [`crate::synth::check::netlist_lines`] gives the netlist's.
pub fn mapped_lines(m: &Mapped, st: &crate::synth::check::Stimulus) -> Vec<String> {
    let mut sim = LcSim::new(m);
    let mut out = Vec::new();
    for row in &st.cycles {
        for (i, p) in m.inputs.iter().enumerate() {
            if st.clock.is_some_and(|k| k.0 == i) {
                continue;
            }
            for b in 0..p.width as usize {
                sim.inputs[i][b] = if row[i] >> b & 1 != 0 { !0 } else { 0 };
            }
        }
        sim.eval();
        let line: Vec<String> = m
            .outputs
            .iter()
            .map(|(_, bits)| crate::synth::check::binary(bits.iter().map(|&s| sim.get(s) & 1 != 0)))
            .collect();
        out.push(line.join(" "));
        if st.clock.is_some() {
            sim.clock();
        }
    }
    out
}
