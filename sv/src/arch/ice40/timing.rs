//! Timing: a delay model of the iCE40LP1K measured on the board, and a
//! static timing analysis of a built design with it.
//!
//! Nothing here comes from a vendor's or another tool's timing data. The
//! board has one clock, 12 MHz (83.3 ns), so delays are measured by how
//! long a path must be to miss it: a flip-flop that toggles every clock
//! launches through a chain of `n` stages into one that captures it, and a
//! sticky flag, set when a capture is not what the launch sent, turns the
//! echo off ([`experiment`]). The shortest failing `n` puts the chain's
//! delay at a clock period. Chains of LUTs packed tight and spread across
//! the die give two equations in a LUT's delay and a routing switch's (the
//! switches counted on the routes the router chose); a carry chain and a
//! block RAM's read, each followed by LUTs, give the carry's delay and the
//! RAM's clock-to-output. The clock-to-output and setup of the flip-flops
//! are taken into those (they are a few hundred picoseconds against 83
//! ns).
//!
//! [`analyse`] then takes each routed signal's switches to each sink and
//! each cell's delay, and finds every flip-flop's latest arrival.

use super::Device;
use super::chipdb::Chipdb;
use super::flow::Built;
use super::icebox::Graph;
use super::map::{Chain, FfCfg, Lc, Mapped, Ram, Src};
use super::route::Sink;
use crate::hash::FxMap;
use crate::synth::Port;

/// Delays in nanoseconds.
#[derive(Clone, Debug, PartialEq)]
pub struct Model {
    /// A LUT, input to output.
    pub lut: f64,
    /// A routing switch (each step of a route).
    pub hop: f64,
    /// A carry, carry in to carry out.
    pub carry: f64,
    /// A block RAM's read, clock to data out.
    pub ram: f64,
    /// Where the numbers came from.
    pub source: String,
}

impl Model {
    /// A guess for when nothing has been measured: the same order as
    /// small FPGAs of this kind, to be replaced by the board's numbers.
    pub fn guess() -> Model {
        Model { lut: 1.0, hop: 0.5, carry: 0.2, ram: 3.0, source: "a guess: not measured".into() }
    }

    pub fn to_record(&self) -> String {
        format!(
            "# What the timing experiments on the board found (sv/src/arch/ice40/timing.rs), in ns.\n# {}\nlut {:.4}\nhop {:.4}\ncarry {:.4}\nram {:.4}\n",
            self.source, self.lut, self.hop, self.carry, self.ram
        )
    }

    pub fn from_record(text: &str) -> Result<Model, String> {
        let mut m = Model::guess();
        m.source = String::new();
        for line in text.lines() {
            if let Some(c) = line.strip_prefix("# ") {
                if !c.starts_with("What the timing") {
                    m.source = c.to_string();
                }
                continue;
            }
            let w: Vec<&str> = line.split_whitespace().collect();
            let [k, v] = w.as_slice() else { continue };
            let v: f64 = v.parse().map_err(|_| format!("timing record: bad number {v}"))?;
            match *k {
                "lut" => m.lut = v,
                "hop" => m.hop = v,
                "carry" => m.carry = v,
                "ram" => m.ram = v,
                _ => return Err(format!("timing record: unknown {k}")),
            }
        }
        Ok(m)
    }
}

/// The 12 MHz clock's period.
pub const PERIOD: f64 = 1000.0 / 12.0;

/// Which experiment.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Kind {
    /// `n` LUTs, placed by the placer.
    Luts,
    /// `n` LUTs, each in the column across the die from the one before.
    Spread,
    /// A carry chain filling a column, then `n` LUTs.
    Carry,
    /// A block RAM read, then `n` LUTs.
    Ram,
}

/// The cells of an experiment: input `clk` and `rx`, output `tx`, which
/// echoes `rx` until a capture goes wrong. Also how many carries it has.
pub fn experiment(kind: Kind, n: usize) -> (Mapped, usize) {
    let clk = Src::Pin { port: 0, bit: 0 };
    let rx = Src::Pin { port: 1, bit: 0 };
    let ff = |lc: &mut Lc| lc.ff = Some(FfCfg { clk, neg: false, en: None, sr: None, async_sr: false, set: false });
    let mut lcs: Vec<Lc> = Vec::new();
    let mut chains = Vec::new();
    let mut rams = Vec::new();
    // The launch: toggles every clock.
    let mut launch = Lc::new("launch".into());
    ff(&mut launch);
    launch.ins[0] = Some(Src::Lc(0));
    launch.lut = !0xaaaa;
    lcs.push(launch);
    let mut prev = Src::Lc(0);
    // Whether a right capture equals the launch (the RAM's path is a
    // clock longer).
    let mut equal = false;
    let mut carries = 0;
    match kind {
        Kind::Carry => {
            // A feeder (carry out = launch), then cells passing the carry
            // (in_1 = 1, in_2 = 0: carry out = carry in), filling a
            // column of 16 tiles, the last's carry read on in_3.
            let mut one = Lc::new("one".into());
            one.lut = 0xffff;
            lcs.push(one);
            let one = Src::Lc(1);
            let mut ids = Vec::new();
            for k in 0..127 {
                let mut lc = Lc::new(format!("carry {k}"));
                lc.carry = k < 126;
                lc.ins[1] = Some(if k == 0 { prev } else { one });
                if k == 126 {
                    lc.carry_in3 = true;
                    lc.lut = 0xff00;
                }
                ids.push(lcs.len() as u32);
                lcs.push(lc);
            }
            carries = 125;
            chains.push(Chain { lcs: ids.clone(), ci_one: true });
            prev = Src::Lc(*ids.last().unwrap());
        }
        Kind::Ram => {
            // Word 0 holds 0s, word 1 1s; the launch is the address.
            let mut init = vec![false; 256 * 16];
            for b in 0..16 {
                init[16 + b] = true;
            }
            let mut one = Lc::new("one".into());
            one.lut = 0xffff;
            lcs.push(one);
            rams.push(Ram {
                name: "ram".into(),
                mode: 0,
                width: 16,
                depth: 256,
                blocks: 1,
                raddr: vec![Some(prev), None, None, None, None, None, None, None],
                re: Some(Src::Lc(1)),
                waddr: Vec::new(),
                wdata: Vec::new(),
                we: None,
                write: false,
                init,
                clk,
                negedge: false,
            });
            prev = Src::Ram { ram: 0, bit: 0 };
            equal = true;
        }
        _ => {}
    }
    for k in 0..n {
        let mut lc = Lc::new(format!("stage {k}"));
        lc.ins[0] = Some(prev);
        lc.lut = 0xaaaa;
        if kind == Kind::Spread {
            // Columns 1 and 12, the logic columns furthest apart.
            let x = if k % 2 == 0 { 1 } else { 12 };
            lc.fixed = Some((x, 1 + (k / 2 / 8) % 16, (k / 2) % 8));
        }
        prev = Src::Lc(lcs.len() as u32);
        lcs.push(lc);
    }
    // The capture, a flag that arms a clock after the start, and the
    // sticky error.
    let cap = lcs.len() as u32;
    let mut c = Lc::new("capture".into());
    ff(&mut c);
    c.ins[0] = Some(prev);
    c.lut = 0xaaaa;
    lcs.push(c);
    // Armed 64 clocks after the start: the board showed a block RAM's
    // reads wrong for its first 16 to 32 clocks after configuration.
    let mut armed2 = 0;
    for k in 0..64 {
        let mut a = Lc::new(format!("armed {k}"));
        ff(&mut a);
        if k == 0 {
            a.lut = 0xffff;
        } else {
            a.ins[0] = Some(Src::Lc(armed2));
            a.lut = 0xaaaa;
        }
        armed2 = lcs.len() as u32;
        lcs.push(a);
    }
    let err = lcs.len() as u32;
    let mut e = Lc::new("error".into());
    ff(&mut e);
    e.ins = [Some(Src::Lc(err)), Some(Src::Lc(cap)), Some(Src::Lc(0)), Some(Src::Lc(armed2))];
    // err | armed & (capture == launch) (or !=, for the RAM).
    e.lut = (0..16).fold(0u16, |t, i| {
        let (er, cp, la, ar) = (i & 1 != 0, i & 2 != 0, i & 4 != 0, i & 8 != 0);
        let wrong = if equal { cp != la } else { cp == la };
        t | ((er || ar && wrong) as u16) << i
    });
    lcs.push(e);
    let mut tx = Lc::new("tx".into());
    tx.ins = [Some(rx), Some(Src::Lc(err)), None, None];
    tx.lut = 0x2222;
    let txi = lcs.len() as u32;
    lcs.push(tx);
    let port = |name: &str| Port { name: name.into(), width: 1, bits: Vec::new() };
    (
        Mapped {
            lcs,
            chains,
            rams,
            inputs: vec![port("clk"), port("rx")],
            outputs: vec![("tx".into(), vec![Src::Lc(txi)])],
        },
        carries,
    )
}

/// How many cells and switches the path through an experiment's stages
/// has in the routes built: LUTs, switches, carries.
pub fn path_counts(b: &Built, kind: Kind, n: usize, carries: usize) -> (usize, usize, usize) {
    let hops = hops_to_sinks(b);
    // Every request's hops to its farthest sink: a chain's stages each
    // drive one sink, the next.
    let mut total = 0;
    for (r, req) in b.requests.iter().enumerate() {
        let name = &req.name;
        let on_path = name == "launch" || name.starts_with("stage ") || name == "ram[0]" || name == "carry 126";
        if on_path {
            total += req.sinks.iter().map(|s| hops.get(&(r, sink_net(b, s))).copied().unwrap_or(0)).max().unwrap_or(0);
        }
    }
    let luts = n + 1 + if kind == Kind::Carry { 1 } else { 0 };
    (luts, total, carries)
}

fn sink_net(b: &Built, s: &Sink) -> usize {
    match s {
        Sink::Net(n) => *n,
        Sink::Lut { lut, input, pins } => pins[b.routed.pins[&(*lut, *input)]],
    }
}

/// For each request and sink net, the switches from the source.
fn hops_to_sinks(b: &Built) -> FxMap<(usize, usize), usize> {
    let mut out = FxMap::default();
    for (r, path) in b.routed.paths.iter().enumerate() {
        let parent: FxMap<usize, usize> = path.iter().map(|&(dst, src, _)| (dst, src)).collect();
        for s in &b.requests[r].sinks {
            let n = sink_net(b, s);
            let (mut k, mut h) = (n, 0);
            while let Some(&p) = parent.get(&k) {
                k = p;
                h += 1;
                if h > 10_000 {
                    break;
                }
            }
            out.insert((r, n), h);
        }
    }
    out
}

/// A path's end: its delay and what it is.
#[derive(Clone, Debug)]
pub struct Arrival {
    pub delay: f64,
    pub to: String,
    pub from: String,
}

/// What [`analyse`] found: the latest arrival at a flip-flop (or RAM
/// input), and at an output pin.
#[derive(Clone, Debug)]
pub struct Report {
    pub worst: Option<Arrival>,
    pub worst_output: Option<Arrival>,
}

impl Report {
    pub fn fmax(&self) -> Option<f64> {
        self.worst.as_ref().map(|w| 1000.0 / w.delay.max(0.001))
    }

    pub fn text(&self, m: &Model) -> String {
        let mut s = String::new();
        if let Some(w) = &self.worst {
            let src = if m.source.starts_with("measured") { "delays measured on the board" } else { "guessed delays" };
            s += &format!(
                "timing: longest path {:.1} ns ({} to {}): about {:.0} MHz at most ({src})",
                w.delay,
                w.from,
                w.to,
                self.fmax().unwrap(),
            );
        } else {
            s += "timing: no register paths";
        }
        if let Some(o) = &self.worst_output {
            s += &format!("; to an output {:.1} ns ({})", o.delay, o.to);
        }
        s
    }
}

/// The static timing analysis of a built design: each cell's output's
/// latest arrival after a clock edge (flip-flops, RAM outputs and input
/// pins start at 0), through LUTs and carries and each route's switches,
/// to every flip-flop's input, enable and set/reset, every RAM input and
/// every output pin.
pub fn analyse(b: &Built, d: &Device, db: &Chipdb, model: &Model) -> Report {
    let _ = d;
    let raw = db.routing.raw();
    let g = Graph::new(&raw);
    let m = &b.mapped;
    let at = &b.placement.at;
    let hops = hops_to_sinks(b);
    // The arrival at each sink net, given its request's source's.
    let src_of: FxMap<usize, usize> =
        b.requests.iter().enumerate().flat_map(|(r, q)| q.sinks.iter().map(move |s| (sink_net(b, s), r))).collect();
    let req_src: Vec<usize> = b.requests.iter().map(|q| q.src).collect();
    // Requests from a RAM's read data start at its clock-to-output.
    let is_ram = |name: &str| m.rams.iter().any(|r| name.strip_prefix(&r.name).is_some_and(|t| t.starts_with('[')));
    let ram_req: Vec<bool> = b.requests.iter().map(|q| is_ram(&q.name)).collect();
    // Each source net's driver.
    let mut driver: FxMap<usize, Src> = FxMap::default();
    for (i, &(x, y, z)) in at.iter().enumerate() {
        if let Ok(n) = g.net(x, y, &format!("lutff_{z}/out")) {
            driver.insert(n, Src::Lc(i as u32));
        }
    }
    let below: FxMap<u32, u32> = m.chains.iter().flat_map(|c| c.lcs.windows(2).map(|w| (w[1], w[0]))).collect();
    // Arrivals by memo: a cell's output, and its carry out.
    #[derive(Clone, Copy)]
    enum Want {
        Out(u32),
        Cout(u32),
    }
    let mut out_arr: FxMap<u32, (f64, String)> = FxMap::default();
    let mut cout_arr: FxMap<u32, (f64, String)> = FxMap::default();
    // The arrival at a cell's input net: its source's, plus the switches.
    #[allow(clippy::too_many_arguments)]
    fn at_net(
        n: usize,
        ram_req: &[bool],
        src_of: &FxMap<usize, usize>,
        req_src: &[usize],
        driver: &FxMap<usize, Src>,
        hops: &FxMap<(usize, usize), usize>,
        model: &Model,
        m: &Mapped,
        cell: &mut dyn FnMut(Want) -> (f64, String),
    ) -> (f64, String) {
        let Some(&r) = src_of.get(&n) else { return (0.0, "a pin".into()) };
        let h = hops.get(&(r, n)).copied().unwrap_or(0) as f64 * model.hop;
        if ram_req[r] {
            return (model.ram + h, "a RAM".into());
        }
        let (a, from) = match driver.get(&req_src[r]) {
            Some(Src::Lc(c)) => cell(Want::Out(*c)),
            // A RAM's output (its request named after it) or an input
            // pin.
            _ => {
                let _ = m;
                (0.0, String::new())
            }
        };
        (a + h, from)
    }
    let _ = &at_net;
    // Iterative evaluation in an order that settles: repeat until no
    // change (paths are acyclic through cells without flip-flops).
    let lc_in_net = |i: usize, k: usize| -> Option<usize> {
        let (x, y, z) = at[i];
        let lc = &m.lcs[i];
        let swappable = !lc.carry && !lc.carry_in3;
        let pin = if swappable { b.routed.pins.get(&(i as u32, k)).copied().unwrap_or(k) } else { k };
        g.net(x, y, &format!("lutff_{z}/in_{pin}")).ok()
    };
    for _ in 0..m.lcs.len() + 2 {
        let mut changed = false;
        for i in 0..m.lcs.len() {
            let lc = &m.lcs[i];
            let mut get = |w: Want| -> (f64, String) {
                match w {
                    Want::Out(c) => {
                        if m.lcs[c as usize].ff.is_some() {
                            (0.0, m.lcs[c as usize].name.clone())
                        } else {
                            out_arr.get(&c).cloned().unwrap_or((0.0, String::new()))
                        }
                    }
                    Want::Cout(c) => cout_arr.get(&c).cloned().unwrap_or((0.0, String::new())),
                }
            };
            let mut ins: Vec<(f64, String)> = Vec::new();
            for k in 0..4 {
                if lc.ins[k].is_some() {
                    if let Some(n) = lc_in_net(i, k) {
                        ins.push(at_net(n, &ram_req, &src_of, &req_src, &driver, &hops, model, m, &mut get));
                    }
                }
            }
            let cin = below.get(&(i as u32)).map(|&p| get(Want::Cout(p)));
            if lc.carry_in3 {
                ins.push(cin.clone().unwrap_or((0.0, String::new())));
            }
            let latest = |v: &[(f64, String)]| {
                v.iter().cloned().fold((0.0f64, String::new()), |a, x| if x.0 > a.0 { x } else { a })
            };
            let lut = latest(&ins);
            let o = (lut.0 + model.lut, if lut.1.is_empty() { lc.name.clone() } else { lut.1.clone() });
            if lc.carry {
                let mut v: Vec<(f64, String)> = Vec::new();
                for k in [1, 2] {
                    if lc.ins[k].is_some() {
                        if let Some(n) = lc_in_net(i, k) {
                            v.push(at_net(n, &ram_req, &src_of, &req_src, &driver, &hops, model, m, &mut get));
                        }
                    }
                }
                v.extend(cin);
                let c = latest(&v);
                let co = (c.0 + model.carry, c.1);
                if cout_arr.get(&(i as u32)).is_none_or(|x| (x.0 - co.0).abs() > 1e-9) {
                    cout_arr.insert(i as u32, co);
                    changed = true;
                }
            }
            if out_arr.get(&(i as u32)).is_none_or(|x| (x.0 - o.0).abs() > 1e-9) {
                out_arr.insert(i as u32, o);
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }
    // Ends: flip-flops' inputs (their own LUT's output), and every sink
    // net that is not a LUT input (enables, set/resets, RAM inputs,
    // output pins).
    let mut worst: Option<Arrival> = None;
    let mut worst_output: Option<Arrival> = None;
    let consider = |slot: &mut Option<Arrival>, a: Arrival| {
        if slot.as_ref().is_none_or(|w| a.delay > w.delay) {
            *slot = Some(a);
        }
    };
    for (i, lc) in m.lcs.iter().enumerate() {
        if lc.ff.is_some() {
            let (a, from) = out_arr.get(&(i as u32)).cloned().unwrap_or((0.0, String::new()));
            consider(&mut worst, Arrival { delay: a, to: lc.name.clone(), from });
        }
    }
    for (r, q) in b.requests.iter().enumerate() {
        for s in &q.sinks {
            let Sink::Net(n) = s else { continue };
            let name = g_name(&raw, *n);
            if name.contains("/in_") || name.ends_with("lutff_global/clk") || name.contains("CLK") {
                continue;
            }
            let src = match driver.get(&q.src) {
                Some(Src::Lc(c)) => {
                    if m.lcs[*c as usize].ff.is_some() {
                        (0.0, m.lcs[*c as usize].name.clone())
                    } else {
                        out_arr.get(c).cloned().unwrap_or((0.0, String::new()))
                    }
                }
                _ => (0.0, q.name.clone()),
            };
            let a = Arrival {
                delay: src.0 + hops.get(&(r, *n)).copied().unwrap_or(0) as f64 * model.hop,
                to: name.clone(),
                from: src.1,
            };
            if name.contains("D_OUT") {
                consider(&mut worst_output, a);
            } else {
                consider(&mut worst, a);
            }
        }
    }
    Report { worst, worst_output }
}

/// A net's first name, for reports.
fn g_name(raw: &super::chipdb::Raw, n: usize) -> String {
    raw.nets.get(n).and_then(|v| v.first()).map_or(format!("net {n}"), |(x, y, s)| format!("{s} at {x} {y}"))
}
