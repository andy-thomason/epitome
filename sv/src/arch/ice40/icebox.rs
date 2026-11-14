//! What a logic cell's bits do, found by running IceStorm's `icebox_vlog`
//! (an `.asc` to Verilog) and `icebox_explain` (an `.asc` to IceStorm's
//! names for its settings) as black boxes, from the pinned IceStorm commit
//! `ice40_asc_check.py` fetches. Their source is not read; only what they
//! print. The chip database ([`super::chipdb`]) says which 20 bits belong
//! to each logic cell and how to route; these experiments say what each
//! bit means.
//!
//! The experiments use logic tile [`TILE`]: its cell's inputs are routed
//! from four pins, its output to a pin set up as an output, and its
//! flip-flop's clock, enable and set/reset from three more (routes found by
//! a breadth-first search of the chip database's switches, each net kept
//! clear of the routes before it). Then:
//!
//! - `lut`: each of the cell's 20 bits alone; `icebox_vlog` prints the
//!   LUT as an expression of the inputs, which is evaluated for all 16
//!   input values. A bit that makes it true for exactly one input value is
//!   that LUT index; one that leaves it 0 is not a LUT bit. Done for all
//!   eight cells.
//! - `options`: `icebox_explain` names each bit that is not a LUT bit.
//! - `ff`: the flip-flop's statement in `icebox_vlog`'s Verilog, for each
//!   setting of `Set_NoReset`, `AsyncSetReset` and the tile's `NegClk`,
//!   with `DffEnable`; and without `DffEnable`.
//! - `carry`: the carry's expression with `CarryEnable`, and with the
//!   tile's `CarryInSet` too.
//!
//! Nets are named in the statements recorded by what the tile calls them:
//! `in_0`..`in_3`, `lout` (the LUT's output), `out`, `co` (carry out),
//! `ci` (`carry_in_mux`), `clk`, `cen` and `sr`.

use super::Device;
use super::chipdb::{Bit, Chipdb, Raw};
use crate::hash::{FxMap, FxSet};
use std::fmt::Write as _;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};

/// The logic tile the experiments use, and the pins: the output, the four
/// inputs, then clock, enable and set/reset (`(x, y, z)` of IO tiles).
pub const TILE: (usize, usize) = (8, 1);
const OUT_PIN: (usize, usize, usize) = (8, 0, 0);
const IN_PINS: [(usize, usize, usize); 4] = [(9, 0, 0), (9, 0, 1), (10, 0, 0), (10, 0, 1)];
const CTRL_PINS: [(usize, usize, usize, &str); 3] =
    [(11, 0, 0, "lutff_global/clk"), (11, 0, 1, "lutff_global/cen"), (12, 0, 0, "lutff_global/s_r")];

#[derive(Clone, Debug)]
pub struct Icebox {
    pub dir: PathBuf,
}

impl Icebox {
    /// `$ICEBOX` (the directory holding `icebox_vlog.py`), or the pinned
    /// IceStorm checkout's `icebox` directory.
    pub fn find() -> Option<Icebox> {
        let dir = std::env::var_os("ICEBOX").map(PathBuf::from).or_else(|| {
            let cache = std::env::var_os("XDG_CACHE_HOME")
                .map(PathBuf::from)
                .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".cache")))?;
            Some(cache.join("epitome").join(format!("icestorm-{}", &super::icepack::ICESTORM_COMMIT[..12])).join("icebox"))
        })?;
        dir.join("icebox_vlog.py").is_file().then_some(Icebox { dir })
    }

    /// The chip database for a part, from `icebox_chipdb`.
    pub fn chipdb(&self, part: &super::Part) -> Result<String, String> {
        let args: Vec<&str> = [part.chipdb_flag].into_iter().filter(|f| !f.is_empty()).collect();
        self.run("icebox_chipdb", &args, "")
    }

    /// Runs `tool` on `.asc` text, and returns what it prints (not its
    /// errors: a traceback would show its source).
    pub fn run(&self, tool: &str, args: &[&str], asc: &str) -> Result<String, String> {
        let mut child = Command::new("python3")
            .arg(self.dir.join(format!("{tool}.py")))
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| format!("python3: {e}"))?;
        let mut stdin = child.stdin.take().unwrap();
        let mut stdout = child.stdout.take().unwrap();
        let mut out = String::new();
        std::thread::scope(|s| {
            s.spawn(move || {
                let _ = stdin.write_all(asc.as_bytes());
            });
            let _ = stdout.read_to_string(&mut out);
        });
        let status = child.wait().map_err(|e| e.to_string())?;
        if !status.success() {
            return Err(format!("{tool} failed"));
        }
        Ok(out)
    }
}

/// The chip database's switches as a graph, for finding routes.
pub struct Graph {
    index: FxMap<(usize, usize, String), usize>,
    /// For each net, `(dst net, tile x, tile y, bits, values)`.
    fanout: Vec<Vec<(usize, usize, usize, Vec<Bit>, String)>>,
}

impl Graph {
    pub fn new(raw: &Raw) -> Graph {
        let mut index = FxMap::default();
        for (i, net) in raw.nets.iter().enumerate() {
            for (x, y, n) in net {
                index.insert((*x, *y, n.clone()), i);
            }
        }
        let mut fanout = vec![Vec::new(); raw.nets.len()];
        for (x, y, _, dst, bits, opts) in &raw.switches {
            for (v, src) in opts {
                fanout[*src].push((*dst, *x, *y, bits.clone(), v.clone()));
            }
        }
        Graph { index, fanout }
    }

    pub fn net(&self, x: usize, y: usize, name: &str) -> Result<usize, String> {
        self.index.get(&(x, y, name.to_string())).copied().ok_or_else(|| format!("no net {name} in tile {x} {y}"))
    }

    /// The bits to set for a route from `from` to `to` through nets not in
    /// `used`, which then holds the route's nets.
    pub fn route(&self, from: usize, to: usize, used: &mut FxSet<usize>) -> Result<Vec<(usize, usize, Bit)>, String> {
        let mut prev: FxMap<usize, (usize, usize, usize, &Vec<Bit>, &String)> = FxMap::default();
        let mut queue = std::collections::VecDeque::from([from]);
        let mut seen = FxSet::default();
        seen.insert(from);
        while let Some(n) = queue.pop_front() {
            if n == to {
                break;
            }
            for (dst, x, y, bits, v) in &self.fanout[n] {
                if (*dst == to || !used.contains(dst)) && seen.insert(*dst) {
                    prev.insert(*dst, (n, *x, *y, bits, v));
                    queue.push_back(*dst);
                }
            }
        }
        if !seen.contains(&to) {
            return Err(format!("no route from net {from} to net {to}"));
        }
        let mut out = Vec::new();
        let mut n = to;
        used.insert(to);
        while let Some(&(p, x, y, bits, v)) = prev.get(&n) {
            for (b, val) in bits.iter().zip(v.bytes()) {
                if val == b'1' {
                    out.push((x, y, *b));
                }
            }
            used.insert(p);
            n = p;
        }
        Ok(out)
    }
}

/// The full-die `.asc` text with `bits` set (`icebox` wants every tile).
pub fn full_asc(d: &Device, bits: &FxSet<(usize, usize, Bit)>) -> String {
    let mut s = String::with_capacity(720_000);
    writeln!(s, ".device {}", d.asc_device).unwrap();
    for t in &d.tiles {
        writeln!(s, ".{}_tile {} {}", d.kinds[t.kind].name, t.x, t.y).unwrap();
        for r in 0..16 {
            for c in 0..d.kinds[t.kind].width {
                s.push(if bits.contains(&(t.x, t.y, (r, c))) { '1' } else { '0' });
            }
            s.push('\n');
        }
    }
    s
}

/// What the experiments found.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Semantics {
    /// For each logic cell, the bit of each LUT index.
    pub lut: Vec<[Bit; 16]>,
    /// For each logic cell, its other bits by IceStorm's name.
    pub options: Vec<Vec<(String, Bit)>>,
    /// `(Set_NoReset, AsyncSetReset, NegClk, statement)`.
    pub ff: Vec<(bool, bool, bool, String)>,
    /// The cell's output without `DffEnable`.
    pub ff_off: String,
    pub carry: String,
    /// With `CarryInSet`: the statements about `ci`.
    pub carry_in_set: String,
    /// How many times `icebox_vlog` and `icebox_explain` ran.
    pub runs: usize,
}

/// `icebox_vlog`'s wires: each `nN` and the tile names in its comments.
fn wires(v: &str) -> FxMap<String, Vec<(usize, usize, String)>> {
    let mut out: FxMap<String, Vec<(usize, usize, String)>> = FxMap::default();
    let mut cur: Option<String> = None;
    for line in v.lines() {
        let line = line.trim();
        if let Some(w) = line.strip_prefix("wire ").and_then(|l| l.strip_suffix(';')) {
            cur = Some(w.to_string());
            out.entry(w.to_string()).or_default();
        } else if let (Some(c), Some(body)) = (&cur, line.strip_prefix("// (").and_then(|l| l.strip_suffix(')'))) {
            let p: Vec<&str> = body.splitn(3, ", ").collect();
            if let [x, y, name] = p[..] {
                if let (Ok(x), Ok(y)) = (x.parse(), y.parse()) {
                    out.get_mut(c).unwrap().push((x, y, name.trim_matches('\'').to_string()));
                }
            }
        } else if !line.starts_with("//") {
            cur = None;
        }
    }
    out
}

/// The role of each wire in logic cell `lc` of [`TILE`].
fn roles(v: &str, lc: usize) -> FxMap<String, String> {
    let mut out = FxMap::default();
    let names = [
        (format!("lutff_{lc}/lout"), "lout".to_string()),
        (format!("lutff_{lc}/out"), "out".to_string()),
        (format!("lutff_{lc}/cout"), "co".to_string()),
        ("carry_in_mux".to_string(), "ci".to_string()),
        ("lutff_global/clk".to_string(), "clk".to_string()),
        ("lutff_global/cen".to_string(), "cen".to_string()),
        ("lutff_global/s_r".to_string(), "sr".to_string()),
    ]
    .into_iter()
    .chain((0..4).map(|j| (format!("lutff_{lc}/in_{j}"), format!("in_{j}"))));
    let names: Vec<_> = names.collect();
    for (w, members) in wires(v) {
        for (n, role) in &names {
            if members.iter().any(|(x, y, m)| (*x, *y) == TILE && m == n) {
                out.insert(w.clone(), role.clone());
                break;
            }
        }
    }
    out
}

/// `text` with each `nN` replaced by its role (or `x`), spaces collapsed.
fn normalise(text: &str, roles: &FxMap<String, String>) -> String {
    let mut out = String::new();
    let b = text.as_bytes();
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'n' && (i == 0 || !(b[i - 1].is_ascii_alphanumeric() || b[i - 1] == b'_')) && b.get(i + 1).is_some_and(|c| c.is_ascii_digit()) {
            let mut j = i + 1;
            while j < b.len() && b[j].is_ascii_digit() {
                j += 1;
            }
            out.push_str(roles.get(&text[i..j]).map_or("x", |s| s));
            i = j;
        } else {
            out.push(b[i] as char);
            i += 1;
        }
    }
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Evaluates a Verilog expression of `?:`, `|`, `^`, `&`, `!`, `~`,
/// parentheses, `1'b0`, `1'b1` and named one-bit values.
pub fn eval(e: &str, val: &dyn Fn(&str) -> Option<bool>) -> Result<bool, String> {
    struct P<'a> {
        t: Vec<&'a str>,
        i: usize,
    }
    fn tokens(e: &str) -> Vec<&str> {
        let mut out = Vec::new();
        let b = e.as_bytes();
        let mut i = 0;
        while i < b.len() {
            let c = b[i];
            if c.is_ascii_whitespace() {
                i += 1;
            } else if c.is_ascii_alphanumeric() || c == b'_' || c == b'\'' {
                let s = i;
                while i < b.len() && (b[i].is_ascii_alphanumeric() || b[i] == b'_' || b[i] == b'\'') {
                    i += 1;
                }
                out.push(&e[s..i]);
            } else {
                out.push(&e[i..i + 1]);
                i += 1;
            }
        }
        out
    }
    impl<'a> P<'a> {
        fn peek(&self) -> Option<&'a str> {
            self.t.get(self.i).copied()
        }
        fn eat(&mut self, s: &str) -> Result<(), String> {
            if self.peek() == Some(s) {
                self.i += 1;
                Ok(())
            } else {
                Err(format!("expected {s}"))
            }
        }
        fn cond(&mut self, v: &dyn Fn(&str) -> Option<bool>) -> Result<bool, String> {
            let c = self.bin(v, 0)?;
            if self.peek() == Some("?") {
                self.i += 1;
                let a = self.cond(v)?;
                self.eat(":")?;
                let b = self.cond(v)?;
                return Ok(if c { a } else { b });
            }
            Ok(c)
        }
        fn bin(&mut self, v: &dyn Fn(&str) -> Option<bool>, level: usize) -> Result<bool, String> {
            const OPS: [&str; 3] = ["|", "^", "&"];
            if level == OPS.len() {
                return self.unary(v);
            }
            let mut a = self.bin(v, level + 1)?;
            while self.peek() == Some(OPS[level]) {
                self.i += 1;
                let b = self.bin(v, level + 1)?;
                a = match level {
                    0 => a | b,
                    1 => a ^ b,
                    _ => a & b,
                };
            }
            Ok(a)
        }
        fn unary(&mut self, v: &dyn Fn(&str) -> Option<bool>) -> Result<bool, String> {
            match self.peek() {
                Some("!") | Some("~") => {
                    self.i += 1;
                    Ok(!self.unary(v)?)
                }
                Some("(") => {
                    self.i += 1;
                    let x = self.cond(v)?;
                    self.eat(")")?;
                    Ok(x)
                }
                Some("1'b0") => {
                    self.i += 1;
                    Ok(false)
                }
                Some("1'b1") => {
                    self.i += 1;
                    Ok(true)
                }
                Some(name) => {
                    self.i += 1;
                    v(name).ok_or_else(|| format!("unknown value {name}"))
                }
                None => Err("expression ends early".into()),
            }
        }
    }
    let mut p = P { t: tokens(e), i: 0 };
    let x = p.cond(val)?;
    if p.i != p.t.len() {
        return Err(format!("trailing tokens in {e}"));
    }
    Ok(x)
}

/// The line holding `/* WHAT x y z */` for the cell, without the comment.
fn cell_line(v: &str, what: &str, lc: usize) -> Option<String> {
    v.lines().find_map(|l| {
        let start = l.find(&format!("/* {what}"))?;
        let end = start + l[start..].find("*/")?;
        let coords: Vec<usize> = l[start + 3 + what.len()..end].split_whitespace().filter_map(|s| s.parse().ok()).collect();
        (coords == [TILE.0, TILE.1, lc]).then(|| format!("{}{}", &l[..start], &l[end + 2..]).trim().to_string())
    })
}

fn func(bits: &[(String, Vec<Bit>)], name: &str) -> Result<Vec<Bit>, String> {
    bits.iter().find(|(f, _)| f == name).map(|(_, b)| b.clone()).ok_or_else(|| format!("chipdb: no tile function {name}"))
}

fn kind_bits<'a>(db: &'a Chipdb, kind: &str) -> Result<&'a [(String, Vec<Bit>)], String> {
    Ok(&db.tile_bits.iter().find(|(k, _)| k == kind).ok_or_else(|| format!("chipdb: no {kind} tile bits"))?.1)
}

/// The bits that wire logic cell `lc` of [`TILE`] to the pins: inputs and
/// output, and with `ctrl` the tile's clock, enable and set/reset.
pub fn wire_cell(g: &Graph, db: &Chipdb, lc: usize, ctrl: bool) -> Result<FxSet<(usize, usize, Bit)>, String> {
    let (tx, ty) = TILE;
    let mut used = FxSet::default();
    let mut bits = FxSet::default();
    let pin = |p: (usize, usize, usize), port: &str| g.net(p.0, p.1, &format!("io_{}/{port}", p.2));
    bits.extend(g.route(g.net(tx, ty, &format!("lutff_{lc}/out"))?, pin(OUT_PIN, "D_OUT_0")?, &mut used)?);
    for (j, p) in IN_PINS.iter().enumerate() {
        bits.extend(g.route(pin(*p, "D_IN_0")?, g.net(tx, ty, &format!("lutff_{lc}/in_{j}"))?, &mut used)?);
    }
    if ctrl {
        for (x, y, z, name) in CTRL_PINS {
            bits.extend(g.route(pin((x, y, z), "D_IN_0")?, g.net(tx, ty, name)?, &mut used)?);
        }
    }
    for k in 0..6 {
        for b in func(kind_bits(db, "io")?, &format!("IOB_{}.PINTYPE_{k}", OUT_PIN.2))? {
            bits.insert((OUT_PIN.0, OUT_PIN.1, b));
        }
    }
    Ok(bits)
}

/// The truth table of cell `lc`'s LUT in `icebox_vlog`'s output (bit i
/// the output for inputs i), or `None` if it prints no LUT for the cell.
pub fn lut_table(v: &str, lc: usize) -> Result<Option<u16>, String> {
    let r = roles(v, lc);
    let Some(expr) = cell_line(v, "LUT", lc) else { return Ok(None) };
    let rhs = expr.split_once('=').map(|(_, e)| e.trim().trim_end_matches(';')).ok_or("lut: no assignment")?;
    let mut table = 0u16;
    for i in 0..16usize {
        let val = |n: &str| match r.get(n).map(|s| s.as_str()) {
            Some(role) if role.starts_with("in_") => Some(i >> (role.as_bytes()[3] - b'0') & 1 != 0),
            _ => None,
        };
        if eval(rhs, &val)? {
            table |= 1 << i;
        }
    }
    Ok(Some(table))
}

/// Runs the experiments.
pub fn run(ib: &Icebox, d: &Device, db: &Chipdb, raw: &Raw, log: &mut dyn FnMut(&str)) -> Result<Semantics, String> {
    let g = Graph::new(raw);
    let logic_bits = kind_bits(db, "logic")?;
    let (tx, ty) = TILE;
    let mut sem = Semantics::default();
    let wiring = |lc: usize, ctrl: bool| wire_cell(&g, db, lc, ctrl);

    // lut and options
    let mut jobs: Vec<(usize, Bit, String)> = Vec::new();
    for lc in 0..8 {
        let base = wiring(lc, false)?;
        for &b in &func(logic_bits, &format!("LC_{lc}"))? {
            let mut bits = base.clone();
            bits.insert((tx, ty, b));
            jobs.push((lc, b, full_asc(d, &bits)));
        }
    }
    let outs = parallel(jobs.iter().map(|(_, _, a)| ("icebox_vlog", a.as_str())).collect(), ib)?;
    sem.runs += outs.len();
    let mut lut: Vec<[Option<Bit>; 16]> = vec![[None; 16]; 8];
    let mut others: Vec<Vec<Bit>> = vec![Vec::new(); 8];
    for ((lc, b, _), v) in jobs.iter().zip(&outs) {
        match lut_table(v, *lc)? {
            None | Some(0) => others[*lc].push(*b),
            Some(t) if t.count_ones() == 1 && lut[*lc][t.trailing_zeros() as usize].is_none() => lut[*lc][t.trailing_zeros() as usize] = Some(*b),
            Some(t) => return Err(format!("lut: cell {lc} bit {b:?} gives truth table {t:#06x}")),
        }
    }
    for lc in 0..8 {
        sem.lut.push(lut[lc].map(|b| b.ok_or(format!("lut: cell {lc} has no bit for some index"))).into_iter().collect::<Result<Vec<_>, _>>()?.try_into().unwrap());
    }
    let mut named = Vec::new();
    for lc in 0..8 {
        let mut opts = Vec::new();
        for &b in &others[lc] {
            let mut bits = FxSet::default();
            bits.insert((tx, ty, b));
            let e = ib.run("icebox_explain", &["-t", &format!("{tx} {ty}")], &full_asc(d, &bits))?;
            sem.runs += 1;
            let line = e.lines().find(|l| l.starts_with(&format!("LC_{lc} "))).ok_or("options: no LC line")?;
            let words: Vec<&str> = line.split_whitespace().skip(3).collect();
            let [name] = words[..] else { return Err(format!("options: cell {lc} bit {b:?}: {line}")) };
            opts.push((name.to_string(), b));
        }
        opts.sort();
        named.push(opts);
    }
    sem.options = named;
    log(&format!("icebox: LUT and option bits of 8 cells ({} runs)", sem.runs));

    // ff
    let opt = |name: &str| -> Result<Bit, String> {
        sem.options[0].iter().find(|(n, _)| n == name).map(|(_, b)| *b).ok_or_else(|| format!("no option {name}"))
    };
    let ctrl = wiring(0, true)?;
    let neg = func(logic_bits, "NegClk")?;
    let carry_in_set = func(logic_bits, "CarryInSet")?;
    let mut jobs = Vec::new();
    for k in 0..8 {
        let (set, asy, negclk) = (k & 1 != 0, k & 2 != 0, k & 4 != 0);
        let mut bits = ctrl.clone();
        bits.insert((tx, ty, opt("DffEnable")?));
        if set {
            bits.insert((tx, ty, opt("Set_NoReset")?));
        }
        if asy {
            bits.insert((tx, ty, opt("AsyncSetReset")?));
        }
        if negclk {
            bits.extend(neg.iter().map(|&b| (tx, ty, b)));
        }
        jobs.push(((set, asy, negclk), full_asc(d, &bits)));
    }
    let mut off = ctrl.clone();
    off.insert((tx, ty, sem.lut[0][0]));
    jobs.push(((false, false, false), full_asc(d, &off)));
    let mut carry = wiring(0, false)?;
    carry.insert((tx, ty, opt("CarryEnable")?));
    jobs.push(((false, false, false), full_asc(d, &carry)));
    carry.extend(carry_in_set.iter().map(|&b| (tx, ty, b)));
    jobs.push(((false, false, false), full_asc(d, &carry)));
    let outs = parallel(jobs.iter().map(|(_, a)| ("icebox_vlog", a.as_str())).collect(), ib)?;
    sem.runs += outs.len();
    for (i, ((key, _), v)) in jobs.iter().zip(&outs).enumerate() {
        let r = roles(v, 0);
        if i < 8 {
            let line = cell_line(v, "FF", 0).ok_or("ff: no flip-flop in icebox_vlog's output")?;
            sem.ff.push((key.0, key.1, key.2, normalise(&line, &r)));
        } else if i == 8 {
            sem.ff_off = normalise(&cell_line(v, "FF", 0).ok_or("ff: no cell output")?, &r);
        } else {
            let line = normalise(&cell_line(v, "CARRY", 0).ok_or("carry: no carry in icebox_vlog's output")?, &r);
            if i == 9 {
                sem.carry = line;
            } else {
                let ci: Vec<String> = v.lines().map(|l| normalise(l, &r)).filter(|l| l.starts_with("assign ci ")).collect();
                sem.carry_in_set = format!("{line} {}", ci.join(" "));
            }
        }
    }
    log(&format!("icebox: flip-flop and carry statements ({} runs in all)", sem.runs));
    Ok(sem)
}

fn parallel(jobs: Vec<(&str, &str)>, ib: &Icebox) -> Result<Vec<String>, String> {
    let threads = std::thread::available_parallelism().map_or(4, |n| n.get()).min(12);
    let mut out: Vec<Option<Result<String, String>>> = vec![None; jobs.len()];
    let per = jobs.len().div_ceil(threads).max(1);
    std::thread::scope(|s| {
        for (o, j) in out.chunks_mut(per).zip(jobs.chunks(per)) {
            s.spawn(move || {
                for (o, (tool, asc)) in o.iter_mut().zip(j) {
                    *o = Some(ib.run(tool, &[], asc));
                }
            });
        }
    });
    out.into_iter().map(|o| o.unwrap()).collect()
}

impl Semantics {
    pub fn to_text(&self) -> String {
        let mut s = String::new();
        s.push_str("# What sv/src/arch/ice40/icebox.rs found by running icebox_vlog and icebox_explain as black boxes.\n");
        writeln!(s, "runs {}", self.runs).unwrap();
        for (lc, bits) in self.lut.iter().enumerate() {
            for (i, (r, c)) in bits.iter().enumerate() {
                writeln!(s, "lut {lc} {i} {r} {c}").unwrap();
            }
        }
        for (lc, opts) in self.options.iter().enumerate() {
            for (n, (r, c)) in opts {
                writeln!(s, "option {lc} {n} {r} {c}").unwrap();
            }
        }
        for (a, b, c, st) in &self.ff {
            writeln!(s, "ff {} {} {} {st}", *a as u8, *b as u8, *c as u8).unwrap();
        }
        writeln!(s, "ff_off {}", self.ff_off).unwrap();
        writeln!(s, "carry {}", self.carry).unwrap();
        writeln!(s, "carry_in_set {}", self.carry_in_set).unwrap();
        s
    }

    pub fn from_text(text: &str) -> Result<Semantics, String> {
        let mut sem = Semantics::default();
        for (n, line) in text.lines().enumerate() {
            let err = || format!("icebox record line {}: bad", n + 1);
            let w: Vec<&str> = line.split_whitespace().collect();
            if w.is_empty() || w[0].starts_with('#') {
                continue;
            }
            let num = |i: usize| -> Result<usize, String> { w.get(i).and_then(|s| s.parse().ok()).ok_or_else(err) };
            let rest = |i: usize| w[i..].join(" ");
            match w[0] {
                "runs" => sem.runs = num(1)?,
                "lut" => {
                    let lc = num(1)?;
                    if sem.lut.len() <= lc {
                        sem.lut.resize(lc + 1, [(0, 0); 16]);
                    }
                    sem.lut[lc][num(2)?] = (num(3)?, num(4)?);
                }
                "option" => {
                    let lc = num(1)?;
                    if sem.options.len() <= lc {
                        sem.options.resize(lc + 1, Vec::new());
                    }
                    sem.options[lc].push((w[2].to_string(), (num(3)?, num(4)?)));
                }
                "ff" => sem.ff.push((num(1)? != 0, num(2)? != 0, num(3)? != 0, rest(4))),
                "ff_off" => sem.ff_off = rest(1),
                "carry" => sem.carry = rest(1),
                "carry_in_set" => sem.carry_in_set = rest(1),
                _ => return Err(err()),
            }
        }
        Ok(sem)
    }
}
