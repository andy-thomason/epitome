//! Synthesis: an elaborated design ([`crate::design::Design`]) flattened
//! and bit-blasted into a gate netlist, for an FPGA flow
//! ([`crate::arch::ice40::flow`]).
//!
//! The netlist is a graph of two-input ANDs and XORs, three-input
//! majorities, inputs and flip-flops, each node's inputs earlier nodes, and
//! each edge a literal that may be inverted (an and-inverter graph with
//! XOR and majority added). Majorities are adders' carries, kept as such so
//! that a target with carry chains can put them there; XORs are kept so
//! that sums stay small. Nodes are hashed on their inputs, so equal gates
//! are one node, and constants are folded as gates are made.
//!
//! [`lower`] makes the netlist from the design: `assign`, `always_comb`,
//! and `always_ff` (or `always`) on one clock edge with an optional
//! asynchronous reset, through the hierarchy. A flip-flop has a data input
//! and an enable (the paths where it is not assigned keep it), and an
//! asynchronous reset to a constant.
//!
//! A memory (an unpacked array) is kept whole ([`Mem`]): its write port,
//! its read ports (each read into a register on the memory's clock, the
//! register its own), and its initial contents, for a target with block
//! RAM.
//!
//! Not yet: latches, more than one clock, functions and tasks, division,
//! loops whose bounds are not constant, gates and UDPs.

pub mod check;
pub mod lower;

use crate::hash::FxMap;

/// A node's index times two, plus one if inverted.
pub type Lit = u32;
pub const F: Lit = 0;
pub const T: Lit = 1;

pub fn node(l: Lit) -> usize {
    (l >> 1) as usize
}

pub fn inv(l: Lit) -> bool {
    l & 1 != 0
}

pub fn lit(node: usize, inverted: bool) -> Lit {
    (node as Lit) << 1 | inverted as Lit
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Node {
    /// Node 0: false.
    Zero,
    /// Bit `bit` of input port `port`.
    Input {
        port: u32,
        bit: u32,
    },
    And(Lit, Lit),
    Xor(Lit, Lit),
    /// The carry of `a + b + c`: true if two or more are.
    Maj(Lit, Lit, Lit),
    /// Flip-flop `i`'s output.
    Ff(u32),
    /// Bit `bit` of read port `port` of memory `mem`: what it read at the
    /// last clock edge.
    MemOut {
        mem: u32,
        port: u32,
        bit: u32,
    },
    /// A net whose driver is not known yet ([`Netlist::drive`]), there
    /// only while the netlist is being made.
    Wire(u32),
}

/// A flip-flop: on the clock's edge, if `en`, it takes `d`; while `arst`,
/// it holds `arst_value`. It starts at `init` (false where not given).
#[derive(Clone, Debug)]
pub struct Ff {
    pub d: Lit,
    pub en: Lit,
    pub arst: Lit,
    pub arst_value: bool,
    pub init: bool,
    /// The clock: a literal of an input, and whether on the falling edge.
    pub clock: Lit,
    pub negedge: bool,
    pub name: String,
}

/// A memory: `depth` words of `width` bits, on one clock.
#[derive(Clone, Debug)]
pub struct Mem {
    pub name: String,
    pub width: u32,
    pub depth: u32,
    /// Word `w`'s bit `b` at `w * width + b`.
    pub init: Vec<bool>,
    pub write: Option<MemWrite>,
    pub reads: Vec<MemRead>,
    pub clock: Lit,
    pub negedge: bool,
    /// Every port's clock, which must be the same.
    pub clocks: Vec<(Lit, bool)>,
}

/// On the clock's edge, word `addr` takes bit `b` of `data` where
/// `en[b]` (all the same for a whole-word write; a byte write enables its
/// byte).
#[derive(Clone, Debug)]
pub struct MemWrite {
    pub addr: Vec<Lit>,
    pub data: Vec<Lit>,
    pub en: Vec<Lit>,
}

/// On the clock's edge, if `en`, the port's output takes word `addr` (as
/// it was before the edge's write).
#[derive(Clone, Debug)]
pub struct MemRead {
    pub addr: Vec<Lit>,
    pub en: Lit,
}

#[derive(Clone, Debug)]
pub struct Port {
    pub name: String,
    pub width: u32,
    /// An output's bits, least significant first.
    pub bits: Vec<Lit>,
}

#[derive(Clone, Debug, Default)]
pub struct Netlist {
    pub nodes: Vec<Node>,
    pub inputs: Vec<Port>,
    pub outputs: Vec<Port>,
    pub ffs: Vec<Ff>,
    pub mems: Vec<Mem>,
    /// For each [`Node::Wire`]: its name and its driver.
    pub wires: Vec<(String, Option<Lit>)>,
    hash: FxMap<Node, u32>,
}

impl Netlist {
    pub fn new() -> Netlist {
        let mut n = Netlist::default();
        n.nodes.push(Node::Zero);
        n
    }

    fn add(&mut self, k: Node) -> Lit {
        if let Some(&i) = self.hash.get(&k) {
            return lit(i as usize, false);
        }
        let i = self.nodes.len();
        self.nodes.push(k);
        if matches!(k, Node::And(..) | Node::Xor(..) | Node::Maj(..)) {
            self.hash.insert(k, i as u32);
        }
        lit(i, false)
    }

    pub fn input(&mut self, port: u32, bit: u32) -> Lit {
        self.add(Node::Input { port, bit })
    }

    pub fn wire(&mut self, name: String) -> Lit {
        self.wires.push((name, None));
        self.add(Node::Wire(self.wires.len() as u32 - 1))
    }

    /// Sets wire `w`'s driver; false if it has one already.
    pub fn drive(&mut self, w: Lit, to: Lit) -> bool {
        let Node::Wire(i) = self.nodes[node(w)] else { return false };
        let slot = &mut self.wires[i as usize].1;
        if slot.is_some() {
            return false;
        }
        *slot = Some(to ^ (w & 1));
        true
    }

    pub fn ff(&mut self, ff: Ff) -> Lit {
        self.ffs.push(ff);
        self.add(Node::Ff(self.ffs.len() as u32 - 1))
    }

    pub fn mem_out(&mut self, mem: u32, port: u32, bit: u32) -> Lit {
        self.add(Node::MemOut { mem, port, bit })
    }

    pub fn and(&mut self, a: Lit, b: Lit) -> Lit {
        let (a, b) = (a.min(b), a.max(b));
        if a == F || a == b ^ 1 {
            return F;
        }
        if a == T || a == b {
            return b;
        }
        self.add(Node::And(a, b))
    }

    pub fn or(&mut self, a: Lit, b: Lit) -> Lit {
        self.and(a ^ 1, b ^ 1) ^ 1
    }

    pub fn xor(&mut self, a: Lit, b: Lit) -> Lit {
        let flip = (a ^ b) & 1;
        let (a, b) = (a & !1, b & !1);
        let (a, b) = (a.min(b), a.max(b));
        if a == b {
            return flip;
        }
        if a == F {
            return b ^ flip;
        }
        self.add(Node::Xor(a, b)) ^ flip
    }

    pub fn maj(&mut self, a: Lit, b: Lit, c: Lit) -> Lit {
        let mut v = [a, b, c];
        v.sort();
        let [a, b, c] = v;
        if a == b || b == c {
            return b;
        }
        if a == c ^ 1 {
            return b;
        }
        if a == b ^ 1 {
            return c;
        }
        if b == c ^ 1 {
            return a;
        }
        // Two constants: F with T leaves the third (caught above, as F is T
        // inverted); a single constant stays, so a carry chain keeps it.
        self.add(Node::Maj(a, b, c))
    }

    /// `s ? a : b`
    pub fn mux(&mut self, s: Lit, a: Lit, b: Lit) -> Lit {
        if s == T || a == b {
            return a;
        }
        if s == F {
            return b;
        }
        if a == T {
            return self.or(s, b);
        }
        if a == F {
            return self.and(s ^ 1, b);
        }
        if b == T {
            return self.or(s ^ 1, a);
        }
        if b == F {
            return self.and(s, a);
        }
        if a == b ^ 1 {
            return self.xor(s ^ 1, a);
        }
        let x = self.and(s, a);
        let y = self.and(s ^ 1, b);
        self.or(x, y)
    }

    /// Makes a new netlist of what the outputs need, wires replaced by
    /// their drivers, constants folded through them, gates hashed again
    /// and in an order where each node's inputs come first. Errors on a
    /// combinational loop or a needed net nothing drives; flip-flops
    /// nothing reads are dropped.
    pub fn rebuild(&self) -> Result<Netlist, String> {
        let mut out = Netlist::new();
        out.inputs = self.inputs.clone();
        let mut map: Vec<Option<Lit>> = vec![None; self.nodes.len()];
        map[0] = Some(F);
        let mut ff_map: FxMap<u32, u32> = FxMap::default();
        // Flip-flops reached, to have their inputs mapped afterwards.
        let mut ff_todo: Vec<u32> = Vec::new();
        // Memories likewise, by the flip-flop map's keys plus this.
        const MEM: u32 = 1 << 31;
        let mut on_stack = vec![false; self.nodes.len()];
        let map_lit = |l: Lit, map: &Vec<Option<Lit>>| map[node(l)].map(|m| m ^ (l & 1));
        let mut visit = |root: Lit,
                         out: &mut Netlist,
                         map: &mut Vec<Option<Lit>>,
                         ff_todo: &mut Vec<u32>,
                         ff_map: &mut FxMap<u32, u32>|
         -> Result<Lit, String> {
            let mut stack = vec![(node(root), false)];
            while let Some((n, expanded)) = stack.pop() {
                if map[n].is_some() {
                    continue;
                }
                let ins: Vec<Lit> = match self.nodes[n] {
                    Node::And(a, b) | Node::Xor(a, b) => vec![a, b],
                    Node::Maj(a, b, c) => vec![a, b, c],
                    Node::Wire(w) => match self.wires[w as usize].1 {
                        Some(d) => vec![d],
                        None => return Err(format!("{} is read but nothing drives it", self.wires[w as usize].0)),
                    },
                    _ => vec![],
                };
                if !expanded {
                    if on_stack[n] {
                        let name = match self.nodes[n] {
                            Node::Wire(w) => self.wires[w as usize].0.clone(),
                            _ => format!("node {n}"),
                        };
                        return Err(format!("a combinational loop through {name}"));
                    }
                    on_stack[n] = true;
                    stack.push((n, true));
                    for &i in ins.iter().rev() {
                        if map[node(i)].is_none() {
                            if on_stack[node(i)] {
                                let name = match self.nodes[node(i)] {
                                    Node::Wire(w) => self.wires[w as usize].0.clone(),
                                    _ => format!("node {}", node(i)),
                                };
                                return Err(format!("a combinational loop through {name}"));
                            }
                            stack.push((node(i), false));
                        }
                    }
                    continue;
                }
                on_stack[n] = false;
                let m: Vec<Lit> = ins.iter().map(|&i| map_lit(i, map).unwrap()).collect();
                map[n] = Some(match self.nodes[n] {
                    Node::Zero => F,
                    Node::Input { port, bit } => out.input(port, bit),
                    Node::And(..) => out.and(m[0], m[1]),
                    Node::Xor(..) => out.xor(m[0], m[1]),
                    Node::Maj(..) => out.maj(m[0], m[1], m[2]),
                    Node::Wire(_) => m[0],
                    Node::Ff(i) => {
                        let f = &self.ffs[i as usize];
                        let l = out.ff(Ff { d: F, en: F, arst: F, ..f.clone() });
                        ff_map.insert(i, out.ffs.len() as u32 - 1);
                        ff_todo.push(i);
                        l
                    }
                    Node::MemOut { mem, port, bit } => {
                        if !ff_map.contains_key(&(mem | MEM)) {
                            let m = &self.mems[mem as usize];
                            ff_map.insert(mem | MEM, out.mems.len() as u32);
                            out.mems.push(Mem {
                                write: None,
                                reads: Vec::new(),
                                clock: F,
                                clocks: Vec::new(),
                                ..m.clone()
                            });
                            ff_todo.push(mem | MEM);
                        }
                        out.mem_out(ff_map[&(mem | MEM)], port, bit)
                    }
                });
            }
            Ok(map_lit(root, map).unwrap())
        };
        for p in &self.outputs {
            let bits = p
                .bits
                .iter()
                .map(|&b| visit(b, &mut out, &mut map, &mut ff_todo, &mut ff_map))
                .collect::<Result<Vec<_>, _>>()?;
            out.outputs.push(Port { bits, ..p.clone() });
        }
        while let Some(i) = ff_todo.pop() {
            if i & MEM != 0 {
                let m = &self.mems[(i & !MEM) as usize];
                let mut v =
                    |l: Lit,
                     out: &mut Netlist,
                     map: &mut Vec<Option<Lit>>,
                     ff_todo: &mut Vec<u32>,
                     ff_map: &mut FxMap<u32, u32>| visit(l, out, map, ff_todo, ff_map);
                let mut lits = |ls: &[Lit],
                                out: &mut Netlist,
                                map: &mut Vec<Option<Lit>>,
                                ff_todo: &mut Vec<u32>,
                                ff_map: &mut FxMap<u32, u32>|
                 -> Result<Vec<Lit>, String> {
                    ls.iter().map(|&l| v(l, out, map, ff_todo, ff_map)).collect()
                };
                let write = match &m.write {
                    Some(w) => Some(MemWrite {
                        addr: lits(&w.addr, &mut out, &mut map, &mut ff_todo, &mut ff_map)?,
                        data: lits(&w.data, &mut out, &mut map, &mut ff_todo, &mut ff_map)?,
                        en: lits(&w.en, &mut out, &mut map, &mut ff_todo, &mut ff_map)?,
                    }),
                    None => None,
                };
                let mut reads = Vec::new();
                for r in &m.reads {
                    reads.push(MemRead {
                        addr: lits(&r.addr, &mut out, &mut map, &mut ff_todo, &mut ff_map)?,
                        en: lits(&[r.en], &mut out, &mut map, &mut ff_todo, &mut ff_map)?[0],
                    });
                }
                let clock = lits(&[m.clock], &mut out, &mut map, &mut ff_todo, &mut ff_map)?[0];
                let mut clocks = Vec::new();
                for &(c, neg) in &m.clocks {
                    clocks.push((lits(&[c], &mut out, &mut map, &mut ff_todo, &mut ff_map)?[0], neg));
                }
                let g = &mut out.mems[ff_map[&i] as usize];
                (g.write, g.reads, g.clock, g.clocks) = (write, reads, clock, clocks);
                continue;
            }
            let f = &self.ffs[i as usize];
            let d = visit(f.d, &mut out, &mut map, &mut ff_todo, &mut ff_map)?;
            let en = visit(f.en, &mut out, &mut map, &mut ff_todo, &mut ff_map)?;
            let arst = visit(f.arst, &mut out, &mut map, &mut ff_todo, &mut ff_map)?;
            let clock = visit(f.clock, &mut out, &mut map, &mut ff_todo, &mut ff_map)?;
            let g = &mut out.ffs[ff_map[&i] as usize];
            (g.d, g.en, g.arst, g.clock) = (d, en, arst, clock);
        }
        Ok(out)
    }

    /// The clocks: each distinct signal flip-flops and memories take an
    /// edge of (a literal, never inverted), with the edges taken of it.
    /// A clock may be an input or a signal the design makes (a divided
    /// clock); a memory's ports must share one.
    pub fn clocks(&self) -> Result<Vec<(Lit, Vec<bool>)>, String> {
        let mut out: Vec<(Lit, Vec<bool>)> = Vec::new();
        let mut add = |l: Lit, neg: bool| {
            let (c, e) = (l & !1, neg ^ inv(l));
            match out.iter_mut().find(|x| x.0 == c) {
                Some(x) if !x.1.contains(&e) => x.1.push(e),
                Some(_) => {}
                None => out.push((c, vec![e])),
            }
        };
        for f in &self.ffs {
            add(f.clock, f.negedge);
        }
        for m in &self.mems {
            if m.clocks.iter().any(|&(c, n)| (c & !1, n ^ inv(c)) != (m.clock & !1, m.negedge ^ inv(m.clock))) {
                return Err(format!("memory {} is used on two clocks or edges", m.name));
            }
            add(m.clock, m.negedge);
        }
        Ok(out)
    }

    /// The input ports used as clocks, and the level each idles at: 1 if
    /// only falling edges are taken of it, else 0.
    pub fn clock_inputs(&self) -> Result<Vec<(usize, bool)>, String> {
        Ok(self
            .clocks()?
            .into_iter()
            .filter_map(|(l, edges)| match self.nodes[node(l)] {
                Node::Input { port, .. } => Some((port as usize, edges.iter().all(|&e| e))),
                _ => None,
            })
            .collect())
    }

    /// Splits each memory deeper than `bank(width, depth)` words into
    /// banks of that many (a power of two), on its address's upper bits:
    /// each bank written where they pick it, and each read port's result
    /// the bank its upper bits, registered with the read, picked.
    pub fn split_mems(&self, bank: &dyn Fn(u32, u32) -> u32) -> Result<Netlist, String> {
        let mut n = self.clone();
        let old = std::mem::take(&mut n.mems);
        // Each old memory's read data nodes.
        let mut outs: FxMap<(u32, u32, u32), usize> = FxMap::default();
        for (i, k) in n.nodes.iter().enumerate() {
            if let Node::MemOut { mem, port, bit } = *k {
                outs.insert((mem, port, bit), i);
            }
        }
        // Memories whose bits are written under different enables (byte
        // writes) become a memory for each enable first: their lanes.
        let mut lanes: Vec<(usize, Mem, Vec<u32>)> = Vec::new();
        for (mi, m) in old.into_iter().enumerate() {
            let groups: Vec<Lit> = match &m.write {
                Some(w) => {
                    let mut g = w.en.clone();
                    g.sort();
                    g.dedup();
                    g
                }
                None => Vec::new(),
            };
            if groups.len() <= 1 {
                let bits = (0..m.width).collect();
                lanes.push((mi, m, bits));
                continue;
            }
            let w = m.write.as_ref().unwrap();
            for (gi, &e) in groups.iter().enumerate() {
                let bits: Vec<u32> = (0..m.width).filter(|&b| w.en[b as usize] == e).collect();
                let width = bits.len() as u32;
                let mut init = Vec::with_capacity((width * m.depth) as usize);
                for a in 0..m.depth as usize {
                    for &b in &bits {
                        init.push(m.init[a * m.width as usize + b as usize]);
                    }
                }
                let write = Some(MemWrite {
                    addr: w.addr.clone(),
                    data: bits.iter().map(|&b| w.data[b as usize]).collect(),
                    en: vec![e; width as usize],
                });
                let lane = Mem { name: format!("{} lane {gi}", m.name), width, init, write, ..m.clone() };
                lanes.push((mi, lane, bits));
            }
        }
        // Each lane's read data nodes: the old memory's for its bits.
        for (mi, m, bits) in lanes {
            let lane_outs: FxMap<(u32, u32, u32), usize> = outs
                .iter()
                .filter(|(k, _)| k.0 == mi as u32 && bits.contains(&k.2))
                .map(|(k, &i)| ((k.0, k.1, bits.iter().position(|&b| b == k.2).unwrap() as u32), i))
                .collect();
            let outs = &lane_outs;
            let b = bank(m.width, m.depth);
            if b >= m.depth {
                let to = n.mems.len() as u32;
                for (&(mem, port, bit), &i) in outs {
                    if mem == mi as u32 {
                        n.nodes[i] = Node::MemOut { mem: to, port, bit };
                    }
                }
                n.mems.push(m);
                continue;
            }
            if !b.is_power_of_two() {
                return Err(format!("{}: banks of {b} words are not a power of two", m.name));
            }
            let k = b.trailing_zeros() as usize;
            let width = m.width as usize;
            let banks = m.depth.div_ceil(b) as usize;
            let bits = |d: u32| (32 - (d.max(2) - 1).leading_zeros()) as usize;
            // Whether the upper bits of `addr` are `i`.
            let picks = |n: &mut Netlist, addr: &[Lit], i: usize| -> Lit {
                let mut eq = T;
                for (j, &a) in addr.iter().enumerate().skip(k) {
                    let want = if (i >> (j - k)) & 1 != 0 { a } else { a ^ 1 };
                    eq = n.and(eq, want);
                }
                eq
            };
            let first = n.mems.len() as u32;
            for i in 0..banks {
                let depth = (m.depth - i as u32 * b).min(b);
                let ka = bits(depth).min(k);
                let lo = i * b as usize * width;
                let init = m.init[lo..lo + depth as usize * width].to_vec();
                let write = match &m.write {
                    Some(w) => {
                        let here = picks(&mut n, &w.addr, i);
                        let en = w.en.iter().map(|&e| n.and(here, e)).collect();
                        Some(MemWrite { addr: w.addr[..ka].to_vec(), data: w.data.clone(), en })
                    }
                    None => None,
                };
                let reads = m.reads.iter().map(|r| MemRead { addr: r.addr[..ka].to_vec(), en: r.en }).collect();
                n.mems.push(Mem { name: format!("{} bank {i}", m.name), depth, init, write, reads, ..m.clone() });
            }
            for (p, r) in m.reads.iter().enumerate() {
                // The upper bits, registered with the read.
                let sel: Vec<Lit> = (k..r.addr.len())
                    .map(|j| {
                        n.ff(Ff {
                            d: r.addr[j],
                            en: r.en,
                            arst: F,
                            arst_value: false,
                            init: false,
                            clock: m.clock,
                            negedge: m.negedge,
                            name: format!("{} read {p} bank select[{}]", m.name, j - k),
                        })
                    })
                    .collect();
                for bit in 0..m.width {
                    let mut v: Vec<Lit> = (0..1usize << sel.len())
                        .map(|i| if i < banks { n.mem_out(first + i as u32, p as u32, bit) } else { F })
                        .collect();
                    for &s in &sel {
                        v = v.chunks(2).map(|c| n.mux(s, c[1], c[0])).collect();
                    }
                    if let Some(&i) = outs.get(&(mi as u32, p as u32, bit)) {
                        n.wires.push((format!("{} read {p}[{bit}]", m.name), Some(v[0])));
                        n.nodes[i] = Node::Wire(n.wires.len() as u32 - 1);
                    }
                }
            }
        }
        n.rebuild()
    }

    /// The same netlist with no read of a memory on the clock edge that
    /// writes the same word, for a target whose block RAM is unreliable
    /// then (the iCE40's, the board showed). Where a BDD cannot prove a
    /// memory's reads and write never meet, its write is delayed a clock
    /// through registers, and each read port takes the delayed write's
    /// data, registered with the read, where its address matches: what a
    /// read first, write second memory gives, with no access to one word
    /// from both ports at once.
    pub fn delay_writes(&self) -> Result<Netlist, String> {
        let mut n = self.clone();
        let mut outs: FxMap<(u32, u32, u32), usize> = FxMap::default();
        for (i, k) in n.nodes.iter().enumerate() {
            if let Node::MemOut { mem, port, bit } = *k {
                outs.insert((mem, port, bit), i);
            }
        }
        let mut changed = false;
        for mi in 0..n.mems.len() {
            let m = n.mems[mi].clone();
            let Some(w) = m.write.clone() else { continue };
            let any = w.en.iter().fold(F, |a, &e| n.or(a, e));
            let mut risky = false;
            for r in &m.reads {
                let mut eq = T;
                for (&a, &b) in w.addr.iter().zip(&r.addr) {
                    let x = n.xor(a, b);
                    eq = n.and(eq, x ^ 1);
                }
                let both = n.and(any, r.en);
                risky |= !n.exclusive(both, eq);
            }
            if !risky {
                continue;
            }
            changed = true;
            let reg = |n: &mut Netlist, d: Lit, en: Lit, name: String| {
                n.ff(Ff { d, en, arst: F, arst_value: false, init: false, clock: m.clock, negedge: m.negedge, name })
            };
            let addr: Vec<Lit> = w
                .addr
                .iter()
                .enumerate()
                .map(|(i, &a)| reg(&mut n, a, T, format!("{} write address[{i}]", m.name)))
                .collect();
            let data: Vec<Lit> = w
                .data
                .iter()
                .enumerate()
                .map(|(i, &a)| reg(&mut n, a, T, format!("{} write data[{i}]", m.name)))
                .collect();
            // One register for each distinct enable, so that lanes stay
            // lanes.
            let mut en_reg: FxMap<Lit, Lit> = FxMap::default();
            let mut en: Vec<Lit> = Vec::new();
            for &a in &w.en {
                let k = en_reg.len();
                let r = match en_reg.get(&a) {
                    Some(&r) => r,
                    None => {
                        let r = reg(&mut n, a, T, format!("{} write enable {k}", m.name));
                        en_reg.insert(a, r);
                        r
                    }
                };
                en.push(r);
            }
            n.mems[mi].write = Some(MemWrite { addr: addr.clone(), data: data.clone(), en: en.clone() });
            for (p, r) in m.reads.iter().enumerate() {
                let mut hit = T;
                for (&a, &b) in addr.iter().zip(&r.addr) {
                    let x = n.xor(a, b);
                    hit = n.and(hit, x ^ 1);
                }
                for b in 0..m.width {
                    let Some(&i) = outs.get(&(mi as u32, p as u32, b)) else { continue };
                    let take = n.and(hit, en[b as usize]);
                    let sel = reg(&mut n, take, r.en, format!("{} read {p} bypass[{b}]", m.name));
                    let val = reg(&mut n, data[b as usize], r.en, format!("{} read {p} bypass data[{b}]", m.name));
                    let ram = n.mem_out(mi as u32, p as u32, b);
                    // The node just made is the RAM's output from now on;
                    // the old one becomes the mux.
                    let v = n.mux(sel, val, ram);
                    n.wires.push((format!("{} read {p}[{b}]", m.name), Some(v)));
                    n.nodes[i] = Node::Wire(n.wires.len() as u32 - 1);
                }
            }
        }
        if changed { n.rebuild() } else { Ok(n) }
    }

    /// The same netlist with every flip-flop starting at 0: one that
    /// starts at 1 holds its complement instead (its input, reset value
    /// and readers inverted), for a target whose flip-flops all start at
    /// 0.
    pub fn zero_inits(&self) -> Result<Netlist, String> {
        if !self.ffs.iter().any(|f| f.init) {
            return Ok(self.clone());
        }
        let mut n = self.clone();
        for i in 0..n.nodes.len() {
            if let Node::Ff(f) = n.nodes[i] {
                if n.ffs[f as usize].init {
                    let g = &mut n.ffs[f as usize];
                    (g.init, g.d, g.arst_value) = (false, g.d ^ 1, !g.arst_value);
                    n.nodes.push(Node::Ff(f));
                    let q = lit(n.nodes.len() - 1, true);
                    n.wires.push((g.name.clone(), Some(q)));
                    n.nodes[i] = Node::Wire(n.wires.len() as u32 - 1);
                }
            }
        }
        n.rebuild()
    }

    /// Whether `a` and `b` are never both true, as binary decision
    /// diagrams over the nodes that are not gates prove (false if they
    /// grow too big to tell).
    pub fn exclusive(&self, a: Lit, b: Lit) -> bool {
        let mut m = crate::bdd::Manager::new(200_000);
        let mut memo: FxMap<usize, crate::bdd::Bdd> = FxMap::default();
        let mut vars: FxMap<usize, u32> = FxMap::default();
        fn build(
            n: &Netlist,
            l: Lit,
            m: &mut crate::bdd::Manager,
            memo: &mut FxMap<usize, crate::bdd::Bdd>,
            vars: &mut FxMap<usize, u32>,
        ) -> Result<crate::bdd::Bdd, crate::bdd::TooBig> {
            let i = node(l);
            let f = match memo.get(&i) {
                Some(&f) => f,
                None => {
                    let f = match n.nodes[i] {
                        Node::Zero => crate::bdd::FALSE,
                        Node::And(x, y) => {
                            let (x, y) = (build(n, x, m, memo, vars)?, build(n, y, m, memo, vars)?);
                            m.and(x, y)?
                        }
                        Node::Xor(x, y) => {
                            let (x, y) = (build(n, x, m, memo, vars)?, build(n, y, m, memo, vars)?);
                            m.xor(x, y)?
                        }
                        Node::Maj(x, y, z) => {
                            let (x, y, z) =
                                (build(n, x, m, memo, vars)?, build(n, y, m, memo, vars)?, build(n, z, m, memo, vars)?);
                            let xy = m.and(x, y)?;
                            let o = m.or(x, y)?;
                            let oz = m.and(o, z)?;
                            m.or(xy, oz)?
                        }
                        Node::Wire(w) if n.wires[w as usize].1.is_some() => {
                            build(n, n.wires[w as usize].1.unwrap(), m, memo, vars)?
                        }
                        _ => {
                            let k = vars.len() as u32;
                            let v = *vars.entry(i).or_insert(k);
                            m.var(v)?
                        }
                    };
                    memo.insert(i, f);
                    f
                }
            };
            if inv(l) { m.not(f) } else { Ok(f) }
        }
        let r = (|| {
            let x = build(self, a, &mut m, &mut memo, &mut vars)?;
            let y = build(self, b, &mut m, &mut memo, &mut vars)?;
            m.and(x, y)
        })();
        r.is_ok_and(|f| f == crate::bdd::FALSE)
    }

    /// Counts of each kind of node.
    pub fn stats(&self) -> String {
        let mut c = [0usize; 4];
        for n in &self.nodes {
            match n {
                Node::And(..) => c[0] += 1,
                Node::Xor(..) => c[1] += 1,
                Node::Maj(..) => c[2] += 1,
                Node::Input { .. } => c[3] += 1,
                _ => {}
            }
        }
        let mem_bits: u64 = self.mems.iter().map(|m| m.width as u64 * m.depth as u64).sum();
        let mems = if self.mems.is_empty() {
            String::new()
        } else {
            format!(", {} memories ({mem_bits} bits)", self.mems.len())
        };
        format!("{} and, {} xor, {} maj, {} input bits, {} flip-flops{mems}", c[0], c[1], c[2], c[3], self.ffs.len())
    }
}

/// A run of the netlist over 64 independent simulations at once, a bit of
/// a `u64` each.
pub struct Sim<'n> {
    pub n: &'n Netlist,
    /// Each node's value.
    pub vals: Vec<u64>,
    /// Each flip-flop's state.
    pub state: Vec<u64>,
    /// Each input port's bits.
    pub inputs: Vec<Vec<u64>>,
    /// Each memory's contents (word `w`'s bit `b` at `w * width + b`), and
    /// each read port's output.
    pub mem: Vec<Vec<u64>>,
    pub mem_out: Vec<Vec<Vec<u64>>>,
    /// Each flip-flop's and memory's clock as last seen, to find edges.
    ff_clk: Vec<u64>,
    mem_clk: Vec<u64>,
    primed: bool,
}

/// The address each of 64 simulations gives, from its bits' values.
pub fn lane_addrs(bits: &[u64]) -> [usize; 64] {
    let mut a = [0usize; 64];
    for (k, &b) in bits.iter().enumerate().take(usize::BITS as usize - 1) {
        for (l, x) in a.iter_mut().enumerate() {
            *x |= ((b >> l) & 1) as usize * (1 << k);
        }
    }
    a
}

/// A memory's clock edge for 64 simulations: each read port whose enable
/// is set reads (before the write), then the write. Words past `depth`
/// read 0 and are not written.
pub fn mem_edge(
    width: usize,
    depth: usize,
    contents: &mut [u64],
    reads: &[(Vec<u64>, u64)],
    outs: &mut [Vec<u64>],
    write: Option<(Vec<u64>, Vec<u64>, Vec<u64>)>,
) {
    for (p, (addr, en)) in reads.iter().enumerate() {
        let a = lane_addrs(addr);
        for (b, o) in outs[p].iter_mut().enumerate() {
            let mut v = *o & !en;
            for l in 0..64 {
                if en >> l & 1 != 0 && a[l] < depth {
                    v |= contents[a[l] * width + b] & 1 << l;
                }
            }
            *o = v;
        }
    }
    if let Some((addr, data, en)) = write {
        let a = lane_addrs(&addr);
        for l in 0..64 {
            if a[l] < depth {
                for (b, d) in data.iter().enumerate() {
                    if en[b] >> l & 1 != 0 {
                        let c = &mut contents[a[l] * width + b];
                        *c = *c & !(1 << l) | d & 1 << l;
                    }
                }
            }
        }
    }
}

impl<'n> Sim<'n> {
    pub fn new(n: &'n Netlist) -> Sim<'n> {
        let state = n.ffs.iter().map(|f| if f.init { !0 } else { 0 }).collect();
        let inputs = n.inputs.iter().map(|p| vec![0; p.width as usize]).collect();
        let mem = n.mems.iter().map(|m| m.init.iter().map(|&b| if b { !0 } else { 0 }).collect()).collect();
        let mem_out = n.mems.iter().map(|m| vec![vec![0; m.width as usize]; m.reads.len()]).collect();
        Sim {
            n,
            vals: vec![0; n.nodes.len()],
            state,
            inputs,
            mem,
            mem_out,
            ff_clk: vec![0; n.ffs.len()],
            mem_clk: vec![0; n.mems.len()],
            primed: false,
        }
    }

    pub fn get(&self, l: Lit) -> u64 {
        self.vals[node(l)] ^ if inv(l) { !0 } else { 0 }
    }

    fn eval_once(&mut self) {
        for i in 0..self.n.nodes.len() {
            self.vals[i] = match self.n.nodes[i] {
                Node::Zero => 0,
                Node::Input { port, bit } => self.inputs[port as usize][bit as usize],
                Node::And(a, b) => self.get(a) & self.get(b),
                Node::Xor(a, b) => self.get(a) ^ self.get(b),
                Node::Maj(a, b, c) => {
                    let (a, b, c) = (self.get(a), self.get(b), self.get(c));
                    a & b | (a | b) & c
                }
                Node::Ff(f) => self.state[f as usize],
                Node::MemOut { mem, port, bit } => self.mem_out[mem as usize][port as usize][bit as usize],
                Node::Wire(_) => panic!("a wire in a built netlist"),
            };
        }
    }

    /// Settles the logic: the asynchronous resets act, then the gates.
    pub fn eval(&mut self) {
        self.eval_once();
        let mut again = false;
        for (i, f) in self.n.ffs.iter().enumerate() {
            let r = self.get(f.arst);
            if r != 0 {
                let v = if f.arst_value { !0 } else { 0 };
                let s = self.state[i] & !r | v & r;
                again |= s != self.state[i];
                self.state[i] = s;
            }
        }
        if again {
            self.eval_once();
        }
    }

    /// Settles after the inputs change: the logic settles, then every
    /// flip-flop and memory whose clock had its edge takes its input, all
    /// at once, and so on while edges come (a clock the design makes ticks
    /// after the one that makes it, as in SystemVerilog's deltas). The
    /// first settle only notes the clocks' levels.
    pub fn settle(&mut self) {
        for _ in 0..1000 {
            self.eval();
            let edge = |prev: u64, now: u64, neg: bool| if neg { prev & !now } else { !prev & now };
            let ff_edges: Vec<u64> = self
                .n
                .ffs
                .iter()
                .enumerate()
                .map(|(i, f)| if self.primed { edge(self.ff_clk[i], self.get(f.clock), f.negedge) } else { 0 })
                .collect();
            let mem_edges: Vec<u64> = self
                .n
                .mems
                .iter()
                .enumerate()
                .map(|(i, m)| if self.primed { edge(self.mem_clk[i], self.get(m.clock), m.negedge) } else { 0 })
                .collect();
            self.ff_clk = self.n.ffs.iter().map(|f| self.get(f.clock)).collect();
            self.mem_clk = self.n.mems.iter().map(|m| self.get(m.clock)).collect();
            self.primed = true;
            if ff_edges.iter().chain(&mem_edges).all(|&e| e == 0) {
                return;
            }
            let next: Vec<u64> = self
                .n
                .ffs
                .iter()
                .enumerate()
                .map(|(i, f)| {
                    let (en, r) = (self.get(f.en) & ff_edges[i], self.get(f.arst));
                    let v = self.state[i] & !en | self.get(f.d) & en;
                    v & !r | if f.arst_value { r } else { 0 }
                })
                .collect();
            for (i, m) in self.n.mems.iter().enumerate() {
                let e = mem_edges[i];
                if e == 0 {
                    continue;
                }
                let reads: Vec<(Vec<u64>, u64)> = m
                    .reads
                    .iter()
                    .map(|r| (r.addr.iter().map(|&a| self.get(a)).collect(), self.get(r.en) & e))
                    .collect();
                let write = m.write.as_ref().map(|w| {
                    (
                        w.addr.iter().map(|&a| self.get(a)).collect(),
                        w.data.iter().map(|&a| self.get(a)).collect(),
                        w.en.iter().map(|&a| self.get(a) & e).collect(),
                    )
                });
                mem_edge(m.width as usize, m.depth as usize, &mut self.mem[i], &reads, &mut self.mem_out[i], write);
            }
            self.state = next;
        }
        panic!("clocks keep ticking: a clock loop");
    }

    pub fn output(&self, port: usize) -> Vec<u64> {
        self.n.outputs[port].bits.iter().map(|&b| self.get(b)).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn folding() {
        let mut n = Netlist::new();
        let a = n.input(0, 0);
        let b = n.input(0, 1);
        assert_eq!(n.and(a, F), F);
        assert_eq!(n.and(a, T), a);
        assert_eq!(n.and(a, a ^ 1), F);
        assert_eq!(n.xor(a, a), F);
        assert_eq!(n.xor(a ^ 1, b), n.xor(a, b) ^ 1);
        assert_eq!(n.and(a, b), n.and(b, a));
        assert_eq!(n.maj(a, T, F), a);
        assert_eq!(n.maj(a, b, b), b);
        assert_ne!(n.maj(a, b, F), n.and(a, b));
        assert_eq!(n.mux(a, b, b), b);
    }
}
