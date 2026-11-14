//! Checks a netlist against the source it came from: the same random
//! stimulus run through Epitome's interpreter ([`crate::sim`]), with a
//! testbench made here around the top module, and through the netlist;
//! each cycle's outputs must agree wherever the interpreter's are known
//! (an `x` there, as a register before its reset, matches anything).
//!
//! A cycle sets the inputs, lets the logic settle, records the outputs,
//! then gives the clock one edge.

use super::{Netlist, Sim};
use crate::model::Sources;

pub struct Stimulus {
    /// The clock's input port, and whether the flip-flops take its
    /// falling edge.
    pub clock: Option<(usize, bool)>,
    /// For each cycle, each input port's value (ports of up to 64 bits).
    pub cycles: Vec<Vec<u64>>,
}

/// A port that looks like a reset, from its name: and whether it is
/// active low.
fn reset_like(name: &str) -> Option<bool> {
    let n = name.to_ascii_lowercase();
    (n.contains("rst") || n.contains("reset"))
        .then(|| n.ends_with("_n") || n.ends_with("rstn") || n.ends_with("resetn"))
}

/// `cycles` cycles of random inputs (an xorshift of `seed`). Inputs that
/// look like resets are active one cycle in sixteen, and in the first.
pub fn random_stimulus(n: &Netlist, cycles: usize, seed: u64) -> Result<Stimulus, String> {
    let clock = n.clock()?.map(|(l, neg)| match n.nodes[super::node(l)] {
        super::Node::Input { port, .. } => (port as usize, neg),
        _ => unreachable!(),
    });
    let mut s = seed | 1;
    let mut next = move || {
        s ^= s << 13;
        s ^= s >> 7;
        s ^= s << 17;
        s
    };
    let mut out = Vec::new();
    for c in 0..cycles {
        let mut row = Vec::new();
        for (i, p) in n.inputs.iter().enumerate() {
            if p.width > 64 {
                return Err(format!("input {} is wider than 64 bits", p.name));
            }
            let mask = if p.width == 64 { !0 } else { (1u64 << p.width) - 1 };
            let v = if clock.is_some_and(|k| k.0 == i) {
                0
            } else if let Some(low) = reset_like(&p.name).filter(|_| p.width == 1) {
                let active = c == 0 || next() % 16 == 0;
                (active ^ low) as u64
            } else {
                next() & mask
            };
            row.push(v);
        }
        out.push(row);
    }
    Ok(Stimulus { clock, cycles: out })
}

/// The testbench: module `__epitome_tb` around `top`, printing the outputs
/// in binary each cycle.
pub fn testbench(n: &Netlist, top: &str, st: &Stimulus) -> String {
    let mut s = String::from("module __epitome_tb;\n");
    for p in &n.inputs {
        s += &format!("  logic [{}:0] {};\n", p.width - 1, p.name);
    }
    for p in &n.outputs {
        s += &format!("  wire [{}:0] {};\n", p.width - 1, p.name);
    }
    let conns: Vec<String> = n.inputs.iter().chain(&n.outputs).map(|p| format!(".{0}({0})", p.name)).collect();
    s += &format!("  {top} dut({});\n  initial begin\n", conns.join(", "));
    let idle = |neg: bool| if neg { 1 } else { 0 };
    // Every input starts at a known value a step before the first cycle,
    // so that the first cycle's are edges the processes see.
    for (i, p) in n.inputs.iter().enumerate() {
        let v = match st.clock {
            Some((c, neg)) if c == i => idle(neg),
            _ => 0,
        };
        s += &format!("    {} = {v};\n", p.name);
    }
    s += "    #1;\n";
    let fmt: Vec<&str> = n.outputs.iter().map(|_| "%b").collect();
    let outs: Vec<&str> = n.outputs.iter().map(|p| p.name.as_str()).collect();
    for row in &st.cycles {
        for (i, p) in n.inputs.iter().enumerate() {
            if st.clock.is_some_and(|k| k.0 == i) {
                continue;
            }
            s += &format!("    {} = {}'h{:x};\n", p.name, p.width, row[i]);
        }
        s += &format!("    #1 $display(\"{}\", {});\n", fmt.join(" "), outs.join(", "));
        if let Some((c, neg)) = st.clock {
            let name = &n.inputs[c].name;
            s += &format!("    {name} = {}; #1 {name} = {};\n", 1 - idle(neg), idle(neg));
        }
    }
    s += "    $finish;\n  end\nendmodule\n";
    s
}

/// The interpreter's lines for the stimulus.
pub fn reference(src: &Sources, top: &str, n: &Netlist, st: &Stimulus) -> Result<Vec<String>, String> {
    let mut src = src.clone();
    src.text("__epitome_tb.sv", testbench(n, top, st));
    let out = crate::model::elaborate_with_ast(&src, &["__epitome_tb"], |d, ast| {
        let mut buf = Vec::new();
        let r = crate::sim::simulate(d, ast, &mut buf, &crate::sim::Options::default());
        Ok((r, buf))
    })?;
    let (r, buf) = out;
    if r.failed || !r.unsupported.is_empty() {
        return Err(format!("the interpreter could not run the testbench: {:?}", r.unsupported));
    }
    Ok(String::from_utf8_lossy(&buf).lines().map(|l| l.to_string()).collect())
}

pub fn binary(bits: impl DoubleEndedIterator<Item = bool>) -> String {
    bits.rev().map(|b| if b { '1' } else { '0' }).collect()
}

/// The netlist's lines for the stimulus (lane 0 of [`Sim`]).
pub fn netlist_lines(n: &Netlist, st: &Stimulus) -> Vec<String> {
    let mut sim = Sim::new(n);
    let mut out = Vec::new();
    for row in &st.cycles {
        for (i, p) in n.inputs.iter().enumerate() {
            if st.clock.is_some_and(|k| k.0 == i) {
                continue;
            }
            for b in 0..p.width as usize {
                sim.inputs[i][b] = if row[i] >> b & 1 != 0 { !0 } else { 0 };
            }
        }
        sim.eval();
        let line: Vec<String> =
            (0..n.outputs.len()).map(|o| binary(sim.output(o).into_iter().map(|v| v & 1 != 0))).collect();
        out.push(line.join(" "));
        if st.clock.is_some() {
            sim.clock();
        }
    }
    out
}

/// Errors at the first cycle where `got` differs from a known bit of
/// `want`.
pub fn compare(want: &[String], got: &[String]) -> Result<(), String> {
    if want.len() != got.len() {
        return Err(format!("{} cycles from the interpreter, {} from the netlist", want.len(), got.len()));
    }
    for (c, (w, g)) in want.iter().zip(got).enumerate() {
        let ok = w.len() == g.len() && w.bytes().zip(g.bytes()).all(|(a, b)| a == b || !matches!(a, b'0' | b'1'));
        if !ok {
            return Err(format!("cycle {c}: the interpreter gives {w}, the netlist {g}"));
        }
    }
    Ok(())
}
