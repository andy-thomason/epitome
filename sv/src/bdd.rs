//! Reduced ordered binary decision diagrams: a Boolean function of
//! numbered variables as a shared graph, where two functions are equal
//! exactly when their nodes are. The netlist uses them to prove what a
//! small combinational specialisation computes (`netlist::recognise`).
//!
//! Variable 0 is tested first. Nodes are never freed; a manager holds one
//! proof's worth, and gives up past `limit` nodes.

use crate::hash::FxMap;

/// A node: `FALSE`, `TRUE`, or an index into the manager's nodes.
pub type Bdd = u32;

pub const FALSE: Bdd = 0;
pub const TRUE: Bdd = 1;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct Node {
    var: u32,
    lo: Bdd,
    hi: Bdd,
}

pub struct Manager {
    nodes: Vec<Node>,
    unique: FxMap<Node, Bdd>,
    ite_cache: FxMap<(Bdd, Bdd, Bdd), Bdd>,
    limit: usize,
}

/// The diagrams grew past the manager's limit.
#[derive(Debug)]
pub struct TooBig;

impl Manager {
    pub fn new(limit: usize) -> Manager {
        // The terminals, as nodes past every variable.
        let t = Node { var: u32::MAX, lo: 0, hi: 0 };
        Manager { nodes: vec![t, t], unique: FxMap::default(), ite_cache: FxMap::default(), limit }
    }

    pub fn var(&mut self, v: u32) -> Result<Bdd, TooBig> {
        self.mk(v, FALSE, TRUE)
    }

    fn top(&self, f: Bdd) -> u32 {
        self.nodes[f as usize].var
    }

    fn mk(&mut self, var: u32, lo: Bdd, hi: Bdd) -> Result<Bdd, TooBig> {
        if lo == hi {
            return Ok(lo);
        }
        let n = Node { var, lo, hi };
        if let Some(&b) = self.unique.get(&n) {
            return Ok(b);
        }
        if self.nodes.len() >= self.limit {
            return Err(TooBig);
        }
        let b = self.nodes.len() as Bdd;
        self.nodes.push(n);
        self.unique.insert(n, b);
        Ok(b)
    }

    /// The cofactors of `f` for `var` false and true.
    fn split(&self, f: Bdd, var: u32) -> (Bdd, Bdd) {
        let n = self.nodes[f as usize];
        if f > TRUE && n.var == var { (n.lo, n.hi) } else { (f, f) }
    }

    /// `f ? g : h`.
    pub fn ite(&mut self, f: Bdd, g: Bdd, h: Bdd) -> Result<Bdd, TooBig> {
        match (f, g, h) {
            (TRUE, _, _) => return Ok(g),
            (FALSE, _, _) => return Ok(h),
            (_, TRUE, FALSE) => return Ok(f),
            _ if g == h => return Ok(g),
            _ => {}
        }
        if let Some(&r) = self.ite_cache.get(&(f, g, h)) {
            return Ok(r);
        }
        let v = self.top(f).min(self.top(g)).min(self.top(h));
        let (f0, f1) = self.split(f, v);
        let (g0, g1) = self.split(g, v);
        let (h0, h1) = self.split(h, v);
        let lo = self.ite(f0, g0, h0)?;
        let hi = self.ite(f1, g1, h1)?;
        let r = self.mk(v, lo, hi)?;
        self.ite_cache.insert((f, g, h), r);
        Ok(r)
    }

    pub fn not(&mut self, f: Bdd) -> Result<Bdd, TooBig> {
        self.ite(f, FALSE, TRUE)
    }

    pub fn and(&mut self, f: Bdd, g: Bdd) -> Result<Bdd, TooBig> {
        self.ite(f, g, FALSE)
    }

    pub fn or(&mut self, f: Bdd, g: Bdd) -> Result<Bdd, TooBig> {
        self.ite(f, TRUE, g)
    }

    pub fn xor(&mut self, f: Bdd, g: Bdd) -> Result<Bdd, TooBig> {
        let ng = self.not(g)?;
        self.ite(f, ng, g)
    }

    /// `f` with every variable false.
    pub fn at_zero(&self, mut f: Bdd) -> bool {
        while f > TRUE {
            f = self.nodes[f as usize].lo;
        }
        f == TRUE
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn equal_functions_are_the_same_node() {
        let mut m = Manager::new(1 << 16);
        let (a, b) = (m.var(0).unwrap(), m.var(1).unwrap());
        // a & b == !(!a | !b)
        let x = m.and(a, b).unwrap();
        let (na, nb) = (m.not(a).unwrap(), m.not(b).unwrap());
        let o = m.or(na, nb).unwrap();
        let y = m.not(o).unwrap();
        assert_eq!(x, y);
        let z = m.xor(a, a).unwrap();
        assert_eq!(z, FALSE);
        assert!(!m.at_zero(x));
        assert!(m.at_zero(o));
    }
}
