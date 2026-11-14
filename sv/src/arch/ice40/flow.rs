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
//! 5. The cells' and switches' bits into a [`Design`], with the RAM
//!    blocks' modes and contents, then the `.asc` and the bitstream by
//!    Epitome's packer.
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
    /// The global networks clocks are on.
    pub clock_glb: Vec<usize>,
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
            self.clock_glb.iter().map(|g| format!("; a clock on global network {g}")).collect::<String>(),
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
    build_mapped(d, db, sem, netlist, mapped, opts)
}

/// Places, routes and packs cells already mapped (or made by hand, for
/// an experiment; `netlist` is then only for the report).
pub fn build_mapped(
    d: &Device,
    db: &Chipdb,
    sem: &Semantics,
    netlist: Netlist,
    mapped: Mapped,
    opts: &Options,
) -> Result<Built, String> {
    let placement = place(d, db, &mapped, &opts.package, &opts.pins, opts.seed)?;
    let raw = db.routing.raw();
    let g = Graph::new(&raw);
    let mut des = Design::new(d, db, sem, &g);
    let m = &mapped;
    let at = &placement.at;
    let net = |x: usize, y: usize, name: &str| g.net(x, y, name);

    // ---- I/O ----
    // Each clock's sinks: the clock input of every tile with its
    // flip-flops, and its RAMs' RCLK and WCLK.
    let mut clocks: Vec<(Src, Vec<(usize, usize, String)>)> = Vec::new();
    let clock_sink =
        |c: Src, sink: (usize, usize, String), clocks: &mut Vec<(Src, Vec<(usize, usize, String)>)>| match clocks
            .iter_mut()
            .find(|x| x.0 == c)
        {
            Some(x) if !x.1.contains(&sink) => x.1.push(sink),
            Some(_) => {}
            None => clocks.push((c, vec![sink])),
        };
    for (i, lc) in m.lcs.iter().enumerate() {
        if let Some(f) = &lc.ff {
            clock_sink(f.clk, (at[i].0, at[i].1, "lutff_global/clk".to_string()), &mut clocks);
        }
    }
    for (ri, r) in m.rams.iter().enumerate() {
        for &(x, y) in &placement.rams[ri] {
            clock_sink(r.clk, des.ram_port(x, y, "RCLK")?, &mut clocks);
            if r.write {
                clock_sink(r.clk, des.ram_port(x, y, "WCLK")?, &mut clocks);
            }
        }
    }
    // A clock from a pin with a global buffer goes on its global network.
    let glb_of = |s: Src| -> Option<usize> {
        let Src::Pin { port, bit } = s else { return None };
        let p = &placement.inputs[&(port, bit)];
        db.gbufpin.iter().find(|g| (g.0, g.1, g.2) == (p.0, p.1, p.2)).map(|g| g.3)
    };
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
    for r in &m.rams {
        for s in r.raddr.iter().chain(&r.waddr).chain(&r.wdata).chain([&r.re, &r.we]).flatten() {
            if let Src::Pin { port, bit } = *s {
                data_inputs.insert((port, bit));
            }
        }
    }
    for (&(port, bit), p) in &placement.inputs {
        let src = Src::Pin { port, bit };
        let via_glb = glb_of(src).is_some() && clocks.iter().any(|c| c.0 == src);
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
            if f.neg {
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

    // ---- RAM blocks: modes and contents ----
    for (ri, r) in m.rams.iter().enumerate() {
        for (b, &(x, y)) in placement.rams[ri].iter().enumerate() {
            des.ram.push(super::asc::AscRam { x, y, words: r.block_words(b as u32) });
            // icebox_vlog: CBIT_2 and 3 are the read mode's bits, CBIT_0
            // and 1 the write mode's, in the upper tile.
            for (bit, rf, wf) in
                [(1, "RamConfig.CBIT_2", "RamConfig.CBIT_0"), (2, "RamConfig.CBIT_3", "RamConfig.CBIT_1")]
            {
                if r.mode & bit != 0 {
                    des.func(x, y + 1, rf)?;
                    if r.write {
                        des.func(x, y + 1, wf)?;
                    }
                }
            }
            if r.negedge {
                des.func(x, y, "NegClk")?;
                des.func(x, y + 1, "NegClk")?;
            }
        }
    }

    // ---- clocks ----
    let mut requests: Vec<Request> = Vec::new();
    let mut clock_glb = Vec::new();
    // Clocks from no global buffer pin are routed as signals, below.
    let mut fabric_clocks: Vec<(Src, Vec<(usize, usize, String)>)> = Vec::new();
    for (c, sinks) in &clocks {
        let Some(glb) = glb_of(*c) else {
            fabric_clocks.push((*c, sinks.clone()));
            continue;
        };
        clock_glb.push(glb);
        des.extra_bit(&format!("padin_glb_netwk.{glb}"))?;
        for (x, y, name) in sinks {
            des.route((*x, *y, &format!("glb_netwk_{glb}")), (*x, *y, name))?;
            let cbs: Vec<[usize; 4]> = db.colbuf.iter().filter(|c| (c[2], c[3]) == (*x, *y)).copied().collect();
            for c in cbs {
                des.func(c[0], c[1], &format!("ColBufCtrl.glb_netwk_{glb}"))?;
            }
        }
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
            Src::Ram { ram, bit } => {
                let (b, pin) = m.rams[ram as usize].pin_of(bit);
                let (x, y) = placement.rams[ram as usize][b as usize];
                let p = des.ram_port(x, y, &format!("RDATA[{pin}]"))?;
                net(p.0, p.1, &p.2)
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
    for (c, clock_sinks) in &fabric_clocks {
        for (x, y, name) in clock_sinks {
            add(*c, Sink::Net(net(*x, *y, name)?), &mut sinks);
        }
    }
    // RAM ports: each block gets the addresses and enables, and its data
    // bits.
    for (ri, r) in m.rams.iter().enumerate() {
        for (b, &(x, y)) in placement.rams[ri].iter().enumerate() {
            let mut ports: Vec<(Src, String)> = Vec::new();
            for (k, s) in r.raddr.iter().enumerate() {
                ports.extend(s.map(|s| (s, format!("RADDR[{k}]"))));
            }
            ports.extend(r.re.map(|s| (s, "RE".to_string())));
            if r.write {
                for (k, s) in r.waddr.iter().enumerate() {
                    ports.extend(s.map(|s| (s, format!("WADDR[{k}]"))));
                }
                ports.extend(r.we.map(|s| (s, "WE".to_string())));
                for (bit, s) in r.wdata.iter().enumerate() {
                    let (blk, pin) = r.pin_of(bit as u32);
                    if blk == b as u32 {
                        ports.extend(s.map(|s| (s, format!("WDATA[{pin}]"))));
                    }
                }
            }
            for (s, port) in ports {
                let p = des.ram_port(x, y, &port)?;
                add(s, Sink::Net(net(p.0, p.1, &p.2)?), &mut sinks);
            }
        }
    }
    for s in order {
        let name = match s {
            Src::Pin { port, bit } => {
                super::place::bit_name(&m.inputs[port as usize].name, m.inputs[port as usize].width, bit)
            }
            Src::Lc(c) => m.lcs[c as usize].name.clone(),
            Src::Ram { ram, bit } => format!("{}[{bit}]", m.rams[ram as usize].name),
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
