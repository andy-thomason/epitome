//! The FPGA flow: SystemVerilog to an iCE40 bitstream, every stage
//! Epitome's own.
//!
//! 1. [`crate::synth`]: the elaborated design flattened into gates and
//!    flip-flops.
//! 2. [`super::map`]: gates onto logic cells (LUT4s, carry chains,
//!    flip-flops).
//! 3. [`super::place`]: cells onto tiles, port bits onto pins.
//! 4. [`super::route`]: signals through the chip database's switches; the
//!    clock through a global network when its pin has one.
//! 5. The cells' and switches' bits into a [`Design`], then the `.asc`
//!    and the bitstream by Epitome's packer.
//!
//! [`check_routes`] reads the routes back from the bits: every sink must
//! be driven, through the switches set, by its signal's source.

use super::chipdb::{Bit, Chipdb};
use super::design::Design;
use super::icebox::{Graph, Semantics};
use super::map::{Mapped, Src, map};
use super::place::{Placement, place};
use super::route::{Request, Routed, Sink, drivers, pathfinder, source_of};
use super::{Device, asc::Asc};
use crate::hash::{FxMap, FxSet};
use crate::model::Sources;
use crate::synth::Netlist;

pub struct Options {
    pub package: String,
    /// Port bit (`name` or `name[i]`) to package pin.
    pub pins: Vec<(String, String)>,
    pub seed: u64,
}

/// Everything the flow made.
pub struct Built {
    pub netlist: Netlist,
    pub mapped: Mapped,
    pub placement: Placement,
    pub routed: Routed,
    pub requests: Vec<Request>,
    pub bits: Vec<(usize, usize, Bit)>,
    /// The global network the clock is on, if any.
    pub clock_glb: Option<usize>,
    pub asc: Asc,
    pub bitstream: Vec<u8>,
}

impl Built {
    pub fn report(&self) -> String {
        format!(
            "synthesis: {}\nmapping: {}\nplacement: wirelength {}\nrouting: {} signals, {} nets, {} passes{}\nbitstream: {} bytes",
            self.netlist.stats(),
            self.mapped.stats(),
            self.placement.cost,
            self.requests.len(),
            self.routed.nets_used(),
            self.routed.passes,
            self.clock_glb.map_or(String::new(), |g| format!("; clock on global network {g}")),
            self.bitstream.len()
        )
    }
}

/// A pin constraints file: lines `port pin` (or `port[i] pin`), `#`
/// comments.
pub fn parse_pins(text: &str) -> Result<Vec<(String, String)>, String> {
    let mut out = Vec::new();
    for (n, line) in text.lines().enumerate() {
        let line = line.split('#').next().unwrap().trim();
        if line.is_empty() {
            continue;
        }
        let w: Vec<&str> = line.split_whitespace().collect();
        match w.as_slice() {
            [port, pin] => out.push((port.to_string(), pin.to_string())),
            _ => return Err(format!("pins line {}: expected `port pin`", n + 1)),
        }
    }
    Ok(out)
}

/// Synthesises `top` from `src` and builds it.
pub fn build_source(
    d: &Device,
    db: &Chipdb,
    sem: &Semantics,
    src: &Sources,
    top: &str,
    opts: &Options,
) -> Result<Built, String> {
    let n = crate::synth::lower::synthesise(src, top)?;
    build(d, db, sem, n, opts)
}

pub fn build(d: &Device, db: &Chipdb, sem: &Semantics, netlist: Netlist, opts: &Options) -> Result<Built, String> {
    let mapped = map(&netlist)?;
    let placement = place(d, db, &mapped, &opts.package, &opts.pins, opts.seed)?;
    let raw = db.routing.raw();
    let g = Graph::new(&raw);
    let mut des = Design::new(d, db, sem, &g);
    let m = &mapped;
    let at = &placement.at;
    let net = |x: usize, y: usize, name: &str| g.net(x, y, name);

    // ---- I/O ----
    let clock = m.clock;
    let clock_pin = clock.map(|(p, _)| placement.inputs[&(p, 0)].clone());
    let clock_glb =
        clock_pin.as_ref().and_then(|c| db.gbufpin.iter().find(|g| (g.0, g.1, g.2) == (c.0, c.1, c.2)).map(|g| g.3));
    let mut data_inputs: FxSet<(u32, u32)> = FxSet::default();
    for lc in &m.lcs {
        for s in lc.ins.iter().flatten().chain(lc.ff.iter().flat_map(|f| f.en.iter().chain(&f.sr))) {
            if let Src::Pin { port, bit } = *s {
                data_inputs.insert((port, bit));
            }
        }
    }
    for (_, bits) in &m.outputs {
        for s in bits {
            if let Src::Pin { port, bit } = *s {
                data_inputs.insert((port, bit));
            }
        }
    }
    for (&(port, bit), p) in &placement.inputs {
        let via_glb = clock_glb.is_some() && clock.is_some_and(|c| c.0 == port);
        if !via_glb || data_inputs.contains(&(port, bit)) {
            des.func(p.0, p.1, &format!("IOB_{}.PINTYPE_0", p.2))?;
        }
    }
    for p in placement.outputs.values() {
        des.func(p.0, p.1, &format!("IOB_{}.PINTYPE_3", p.2))?;
        des.func(p.0, p.1, &format!("IOB_{}.PINTYPE_4", p.2))?;
    }

    // ---- cells ----
    let swappable = |lc: &super::map::Lc| !lc.carry && !lc.carry_in3;
    for (i, lc) in m.lcs.iter().enumerate() {
        let (x, y, z) = at[i];
        if lc.carry {
            des.option(x, y, z, "CarryEnable")?;
        }
        if let Some(f) = &lc.ff {
            des.option(x, y, z, "DffEnable")?;
            if f.sr.is_some() {
                if f.async_sr {
                    des.option(x, y, z, "AsyncSetReset")?;
                }
                if f.set {
                    des.option(x, y, z, "Set_NoReset")?;
                }
            }
            if clock.is_some_and(|c| c.1) {
                des.func(x, y, "NegClk")?;
            }
        }
    }
    // Carry chains: each one's carry in, and the carries into `in_3`.
    for ch in &m.chains {
        let (x, y, _) = at[ch.lcs[0] as usize];
        if ch.ci_one {
            des.func(x, y, "CarryInSet")?;
        }
        for (k, &l) in ch.lcs.iter().enumerate() {
            let (x, y, z) = at[l as usize];
            if k > 0 && z == 0 {
                des.route((x, y, "carry_in"), (x, y, "carry_in_mux"))?;
            }
            if m.lcs[l as usize].carry_in3 {
                let from = if z == 0 { "carry_in_mux".to_string() } else { format!("lutff_{}/cout", z - 1) };
                des.route((x, y, &from), (x, y, &format!("lutff_{z}/in_3")))?;
            }
        }
    }

    // ---- the clock ----
    let mut ff_tiles: Vec<(usize, usize)> =
        m.lcs.iter().enumerate().filter(|(_, l)| l.ff.is_some()).map(|(i, _)| (at[i].0, at[i].1)).collect();
    ff_tiles.sort();
    ff_tiles.dedup();
    let mut requests: Vec<Request> = Vec::new();
    if let (Some(glb), Some(_)) = (clock_glb, clock) {
        des.extra_bit(&format!("padin_glb_netwk.{glb}"))?;
        for &(x, y) in &ff_tiles {
            des.route((x, y, &format!("glb_netwk_{glb}")), (x, y, "lutff_global/clk"))?;
            let cbs: Vec<[usize; 4]> = db.colbuf.iter().filter(|c| (c[2], c[3]) == (x, y)).copied().collect();
            for c in cbs {
                des.func(c[0], c[1], &format!("ColBufCtrl.glb_netwk_{glb}"))?;
            }
        }
    } else if let (Some(p), false) = (&clock_pin, ff_tiles.is_empty()) {
        let src = net(p.0, p.1, &format!("io_{}/D_IN_0", p.2))?;
        let sinks =
            ff_tiles.iter().map(|&(x, y)| net(x, y, "lutff_global/clk").map(Sink::Net)).collect::<Result<_, _>>()?;
        requests.push(Request { name: "clock".into(), src, sinks });
    }

    // ---- signals ----
    let src_net = |s: Src| -> Result<usize, String> {
        match s {
            Src::Pin { port, bit } => {
                let p = &placement.inputs[&(port, bit)];
                net(p.0, p.1, &format!("io_{}/D_IN_0", p.2))
            }
            Src::Lc(c) => {
                let (x, y, z) = at[c as usize];
                net(x, y, &format!("lutff_{z}/out"))
            }
        }
    };
    let mut sinks: FxMap<Src, Vec<Sink>> = FxMap::default();
    let mut order: Vec<Src> = Vec::new();
    let mut add = |s: Src, k: Sink, sinks: &mut FxMap<Src, Vec<Sink>>| {
        if !sinks.contains_key(&s) {
            order.push(s);
        }
        sinks.entry(s).or_default().push(k);
    };
    for (i, lc) in m.lcs.iter().enumerate() {
        let (x, y, z) = at[i];
        let pins = [0, 1, 2, 3].map(|k| net(x, y, &format!("lutff_{z}/in_{k}")));
        let pins = [pins[0].clone()?, pins[1].clone()?, pins[2].clone()?, pins[3].clone()?];
        for (k, s) in lc.ins.iter().enumerate() {
            let Some(s) = *s else { continue };
            let sink = if swappable(lc) { Sink::Lut { lut: i as u32, input: k, pins } } else { Sink::Net(pins[k]) };
            add(s, sink, &mut sinks);
        }
    }
    // The tiles' enables and set/resets, once a tile.
    let mut ctrl_done: FxSet<(usize, usize, &str)> = FxSet::default();
    for (i, lc) in m.lcs.iter().enumerate() {
        let Some(f) = &lc.ff else { continue };
        let (x, y, _) = at[i];
        for (s, name) in [(f.en, "lutff_global/cen"), (f.sr, "lutff_global/s_r")] {
            if let Some(s) = s {
                if ctrl_done.insert((x, y, name)) {
                    add(s, Sink::Net(net(x, y, name)?), &mut sinks);
                }
            }
        }
    }
    for (i, (_, bits)) in m.outputs.iter().enumerate() {
        for (b, &s) in bits.iter().enumerate() {
            let p = &placement.outputs[&(i as u32, b as u32)];
            add(s, Sink::Net(net(p.0, p.1, &format!("io_{}/D_OUT_0", p.2))?), &mut sinks);
        }
    }
    for s in order {
        let name = match s {
            Src::Pin { port, bit } => {
                super::place::bit_name(&m.inputs[port as usize].name, m.inputs[port as usize].width, bit)
            }
            Src::Lc(c) => m.lcs[c as usize].name.clone(),
        };
        requests.push(Request { name, src: src_net(s)?, sinks: sinks.remove(&s).unwrap() });
    }
    // Global networks are the clock's alone.
    let mut blocked: FxSet<usize> = FxSet::default();
    for (x, y, _) in db.tiles.iter().map(|t| (t.1, t.2, ())) {
        for k in 0..8 {
            if let Ok(n) = net(x, y, &format!("glb_netwk_{k}")) {
                blocked.insert(n);
            }
        }
    }
    let routed = pathfinder(&g, &requests, &blocked, 60)?;
    let route_bits = routed.bits(&g);
    des.bits.extend(route_bits.iter().copied());

    // ---- LUTs, permuted to the pins their inputs reached ----
    for (i, lc) in m.lcs.iter().enumerate() {
        let (x, y, z) = at[i];
        let mut pin_of = [0usize, 1, 2, 3];
        if swappable(lc) {
            for k in 0..4 {
                if lc.ins[k].is_some() {
                    pin_of[k] = routed.pins[&(i as u32, k)];
                }
            }
        }
        let mut t = 0u16;
        for mp in 0..16 {
            let mo = (0..4)
                .filter(|&k| lc.ins[k].is_some() || !swappable(lc))
                .fold(0, |a, k| a | ((mp >> pin_of[k]) & 1) << k);
            if lc.lut >> mo & 1 != 0 {
                t |= 1 << mp;
            }
        }
        des.lut(x, y, z, t);
    }
    let bits: Vec<(usize, usize, Bit)> = des.bits.iter().copied().collect();
    let asc = des.asc();
    let bitstream = des.bitstream()?;
    let built = Built { netlist, mapped, placement, routed, requests, bits, clock_glb, asc, bitstream };
    check_routes(&g, &built)?;
    Ok(built)
}

/// Reads the routes back from the configured bits: each request's sinks
/// (the pins its LUT inputs reached) must be driven by its source.
pub fn check_routes(g: &Graph, b: &Built) -> Result<(), String> {
    let bits: FxSet<(usize, usize, Bit)> = b.bits.iter().copied().collect();
    let drv = drivers(g, &bits)?;
    for (ri, r) in b.requests.iter().enumerate() {
        for s in &r.sinks {
            let n = match s {
                Sink::Net(n) => *n,
                Sink::Lut { lut, input, pins } => pins[b.routed.pins[&(*lut, *input)]],
            };
            let got = source_of(&drv, n);
            if got != r.src {
                return Err(format!(
                    "signal {} (request {ri}): sink net {n} is driven from net {got}, not {}",
                    r.name, r.src
                ));
            }
        }
    }
    Ok(())
}
