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
//! Not yet: memories, latches, more than one clock, functions and tasks,
//! division, loops whose bounds are not constant, gates and UDPs.

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

    /// The clock all flip-flops share: an input bit's literal, and
    /// whether on the falling edge. Errors if they do not share one.
    pub fn clock(&self) -> Result<Option<(Lit, bool)>, String> {
        let mut clock = None;
        for f in &self.ffs {
            if !matches!(self.nodes[node(f.clock)], Node::Input { .. }) {
                return Err(format!("{}'s clock is not an input of the top module", f.name));
            }
            let c = (f.clock & !1, f.negedge ^ inv(f.clock));
            match clock {
                None => clock = Some(c),
                Some(k) if k == c => {}
                Some(_) => {
                    return Err(format!(
                        "{} is on another clock or edge than the flip-flops before it: one clock is supported",
                        f.name
                    ));
                }
            }
        }
        Ok(clock)
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
        format!("{} and, {} xor, {} maj, {} input bits, {} flip-flops", c[0], c[1], c[2], c[3], self.ffs.len())
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
}

impl<'n> Sim<'n> {
    pub fn new(n: &'n Netlist) -> Sim<'n> {
        let state = n.ffs.iter().map(|f| if f.init { !0 } else { 0 }).collect();
        let inputs = n.inputs.iter().map(|p| vec![0; p.width as usize]).collect();
        Sim { n, vals: vec![0; n.nodes.len()], state, inputs }
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

    /// A clock edge, after [`Sim::eval`]: each flip-flop takes its input
    /// where enabled and not reset. Then settles again.
    pub fn clock(&mut self) {
        let next: Vec<u64> = self
            .n
            .ffs
            .iter()
            .enumerate()
            .map(|(i, f)| {
                let (en, r) = (self.get(f.en), self.get(f.arst));
                let v = self.state[i] & !en | self.get(f.d) & en;
                v & !r | if f.arst_value { r } else { 0 }
            })
            .collect();
        self.state = next;
        self.eval();
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
