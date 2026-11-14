//! Experiments on a real board: the iCESugar-nano (iCE40LP1K-CM36) on
//! Amy's desk. Designs are made from the description ([`super::design`]),
//! packed by Epitome's own packer, programmed with MuseLab's `icesprog` run
//! as a black box (over the iCELink's USB HID interface: on this Mac,
//! copying to the iCELink's USB drive hangs the FAT driver), and observed
//! through the iCELink's USB serial port.
//!
//! The board's wiring is from its schematic (iCESugar-nano v1.2) and its
//! pin constraints file, both MuseLab's board documentation:
//!
//! - A3 is driven by the iCELink's UART transmit (its PA9, USART1 TX): the
//!   FPGA must only ever read it.
//! - B3 goes to the iCELink's UART receive (PA10, USART1 RX): the FPGA may
//!   drive it, and what it drives arrives at the Mac's serial port.
//! - D1 is the iCELink's clock output (MCO, 12 MHz by default), through
//!   100 ohms.
//! - B6 lights the yellow LED, through 10 kohms to ground.
//!
//! So every experiment drives B3 alone, from a function of A3. The Mac
//! sends bytes at 115200 baud and reads what comes back: the same bytes
//! (`echo`), none (`silent`), or something else (`other`). A design whose
//! output is A3 itself echoes; one whose output is fixed, inverted or
//! never clocked does not.

use super::chipdb::Chipdb;
use super::design::Design;
use super::icebox::{Graph, Semantics};
use super::{Device, fnv64};
use std::fmt::Write as _;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::process::Command;
use std::time::{Duration, Instant};

/// The MuseLab `icesugar` commit `icesprog` is built from.
pub const ICESPROG_COMMIT: &str = "1ebe71bf448e33a1bccfa2db6730d59eafb6c390";
/// The hidapi release it is built with.
pub const HIDAPI_TAG: &str = "hidapi-0.15.0";

fn cache() -> Option<PathBuf> {
    std::env::var_os("XDG_CACHE_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".cache")))
        .map(|c| c.join("epitome"))
}

pub struct Icesprog {
    pub path: PathBuf,
}

impl Icesprog {
    /// `$ICESPROG`, or the build in the cache.
    pub fn find() -> Option<Icesprog> {
        let path = std::env::var_os("ICESPROG")
            .map(PathBuf::from)
            .or_else(|| cache().map(|c| c.join(format!("icesprog-{}", &ICESPROG_COMMIT[..12])).join("icesprog")))?;
        path.is_file().then_some(Icesprog { path })
    }

    /// Builds `icesprog` from its pinned source with hidapi's macOS
    /// backend (it includes `libusb.h` but needs nothing from it, so an
    /// empty one stands in), unless it has been already. Needs curl, tar
    /// and clang.
    pub fn install() -> Result<Icesprog, String> {
        if let Some(i) = Icesprog::find() {
            return Ok(i);
        }
        let dir = cache().ok_or("no cache directory")?.join(format!("icesprog-{}", &ICESPROG_COMMIT[..12]));
        let raw = format!("https://raw.githubusercontent.com/wuxx/icesugar/{ICESPROG_COMMIT}/tools/src/icesprog.c");
        let hidapi = format!("https://github.com/libusb/hidapi/archive/refs/tags/{HIDAPI_TAG}.tar.gz");
        let script = format!(
            "set -e; mkdir -p hidapi stub; curl -fsSL -o icesprog.c {raw}; curl -fsSL {hidapi} | tar -xz -C hidapi --strip-components 1; \
             : > stub/libusb.h; clang -O2 -w -o icesprog icesprog.c hidapi/mac/hid.c -Ihidapi/hidapi -Istub \
             -framework IOKit -framework CoreFoundation -framework AppKit"
        );
        std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        let ok = Command::new("sh").arg("-c").arg(script).current_dir(&dir).status().map_err(|e| e.to_string())?;
        if !ok.success() {
            return Err("building icesprog failed".into());
        }
        Icesprog::find().ok_or_else(|| "icesprog not built".into())
    }

    fn run(&self, args: &[&str]) -> Result<String, String> {
        let out = Command::new(&self.path).args(args).output().map_err(|e| e.to_string())?;
        let text = String::from_utf8_lossy(&out.stdout).into_owned() + &String::from_utf8_lossy(&out.stderr);
        if !out.status.success() {
            return Err(format!("icesprog {}: {}", args.join(" "), text.trim()));
        }
        Ok(text)
    }

    /// `icesprog -p`: the board and flash it reports.
    pub fn probe(&self) -> Result<(String, String), String> {
        let t = self.run(&["-p"])?;
        let field = |name: &str| {
            t.lines().find_map(|l| l.strip_prefix(name)).map(|r| r.trim().trim_start_matches('[').split(']').next().unwrap_or("").to_string())
        };
        Ok((field("board:").ok_or_else(|| format!("no board in: {t}"))?, field("flash:").unwrap_or_default()))
    }

    /// Writes a bitstream to the flash, from which the FPGA configures.
    pub fn program(&self, bin: &[u8]) -> Result<(), String> {
        let path = std::env::temp_dir().join(format!("epitome-ice40-{}.bin", std::process::id()));
        std::fs::write(&path, bin).map_err(|e| e.to_string())?;
        let res = self.run(&["-w", path.to_str().unwrap()]);
        let _ = std::fs::remove_file(&path);
        if !res?.contains("done") {
            return Err("icesprog did not finish".into());
        }
        std::thread::sleep(Duration::from_millis(500));
        Ok(())
    }
}

/// The iCELink's USB serial port, raw at 115200 baud, reads timing out.
pub struct Serial {
    file: std::fs::File,
}

impl Serial {
    /// The first `/dev/cu.usbmodem*`, or `$ICE40_SERIAL`.
    pub fn find() -> Option<PathBuf> {
        if let Some(p) = std::env::var_os("ICE40_SERIAL") {
            return Some(p.into());
        }
        let mut v: Vec<PathBuf> = std::fs::read_dir("/dev")
            .ok()?
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.file_name().and_then(|n| n.to_str()).is_some_and(|n| n.starts_with("cu.usbmodem")))
            .collect();
        v.sort();
        v.into_iter().next()
    }

    pub fn open(path: &PathBuf) -> Result<Serial, String> {
        let file = std::fs::OpenOptions::new().read(true).write(true).open(path).map_err(|e| format!("{}: {e}", path.display()))?;
        // Set the line while it is open, so the settings hold.
        let ok = Command::new("stty")
            .args(["-f", path.to_str().unwrap(), "115200", "raw", "-echo", "clocal", "-crtscts", "min", "0", "time", "1"])
            .status()
            .map_err(|e| e.to_string())?;
        if !ok.success() {
            return Err("stty failed".into());
        }
        Ok(Serial { file })
    }

    /// What arrives within `ms` milliseconds of quiet.
    pub fn read_quiet(&mut self, ms: u64) -> Vec<u8> {
        let mut out = Vec::new();
        let mut last = Instant::now();
        let mut buf = [0u8; 4096];
        while last.elapsed() < Duration::from_millis(ms) {
            match self.file.read(&mut buf) {
                Ok(0) | Err(_) => {}
                Ok(n) => {
                    out.extend(&buf[..n]);
                    last = Instant::now();
                }
            }
        }
        out
    }

    pub fn send(&mut self, data: &[u8]) -> Result<(), String> {
        self.file.write_all(data).map_err(|e| e.to_string())
    }

    /// Sends `data` and says what came back: `echo`, `silent` or `other`.
    pub fn echo(&mut self, data: &[u8]) -> Result<String, String> {
        self.read_quiet(200);
        self.file.write_all(data).map_err(|e| e.to_string())?;
        let back = self.read_quiet(400);
        Ok(if back == data {
            "echo".into()
        } else if back.is_empty() {
            "silent".into()
        } else {
            format!("other:{}", back.len())
        })
    }
}

/// One board experiment: a design, and what the description says it does.
pub struct Experiment {
    pub name: String,
    /// `echo`, `no-echo`, or `unknown` where the description says nothing.
    pub expect: &'static str,
    pub bits: Vec<u8>,
}

/// The logic tile the experiments use: next to A3 and B3's I/O tiles.
pub const TILE: (usize, usize) = (7, 16);
/// The block RAM the RAM experiments use (its `ramb` tile), and a logic
/// tile whose cells give constant ones.
pub const RAM: (usize, usize) = (3, 15);
pub const ONES: (usize, usize) = (4, 15);

/// The experiments, with their bitstreams.
pub fn experiments(d: &Device, db: &Chipdb, sem: &Semantics) -> Result<Vec<Experiment>, String> {
    let raw = db.routing.raw();
    let g = Graph::new(&raw);
    let (tx, ty) = TILE;
    let mut out = Vec::new();
    // B3 a plain output from cell `lc`, A3 a plain input to `input`
    // (a net in TILE).
    let base = |lc: usize, input: &str| -> Result<Design, String> {
        let mut des = Design::new(d, db, sem, &g);
        let (ax, ay, az) = des.pin("cm36", "A3")?;
        let (bx, by, bz) = des.pin("cm36", "B3")?;
        // icebox_vlog: PINTYPE_0 alone is a plain input, PINTYPE_3 and 4 a
        // plain output.
        des.func(ax, ay, &format!("IOB_{az}.PINTYPE_0"))?;
        des.func(bx, by, &format!("IOB_{bz}.PINTYPE_3"))?;
        des.func(bx, by, &format!("IOB_{bz}.PINTYPE_4"))?;
        des.route((ax, ay, &format!("io_{az}/D_IN_0")), (tx, ty, input))?;
        des.route((tx, ty, &format!("lutff_{lc}/out")), (bx, by, &format!("io_{bz}/D_OUT_0")))?;
        Ok(des)
    };
    // The table whose output is input j, whatever the others.
    let follows = |j: usize| (0..16).filter(|i| i >> j & 1 != 0).fold(0u16, |t, i| t | 1 << i);
    // Clocks `to` from D1: its global network (gbufpin), the pad's
    // padin bit, and the column buffers that feed `to`'s tile.
    let clock = |des: &mut Design, to: (usize, usize, &str)| -> Result<(), String> {
        let (cx, cy, cz) = des.pin("cm36", "D1")?;
        let glb = db.gbufpin.iter().find(|g| (g.0, g.1, g.2) == (cx, cy, cz)).ok_or("D1 drives no global network")?.3;
        let bit = format!("padin_glb_netwk.{glb}");
        if !des.extra.iter().any(|e| db.extra_bits.iter().any(|x| x.0 == bit && (x.1, x.2, x.3) == *e)) {
            des.extra_bit(&bit)?;
        }
        des.route((to.0, to.1, &format!("glb_netwk_{glb}")), to)?;
        for c in db.colbuf.iter().filter(|c| (c[2], c[3]) == (to.0, to.1)) {
            des.func(c[0], c[1], &format!("ColBufCtrl.glb_netwk_{glb}"))?;
        }
        Ok(())
    };
    let mut push = |name: String, expect: &'static str, des: Design| -> Result<(), String> {
        out.push(Experiment { name, expect, bits: des.bitstream()? });
        Ok(())
    };

    for lc in 0..8 {
        let mut des = base(lc, &format!("lutff_{lc}/in_0"))?;
        des.lut(tx, ty, lc, follows(0));
        push(format!("lut-cell-{lc}"), "echo", des)?;
    }
    for j in 0..4 {
        for (kind, table, expect) in [
            ("follows", follows(j), "echo"),
            // Only index 2^j: echoes if the unrouted inputs read 0, as
            // icebox_vlog shows them.
            ("single", 1u16 << (1 << j), "echo"),
            ("inverse", !follows(j), "no-echo"),
        ] {
            let mut des = base(0, &format!("lutff_0/in_{j}"))?;
            des.lut(tx, ty, 0, table);
            push(format!("lut-input-{j}-{kind}"), expect, des)?;
        }
    }
    for (ie, ren) in [(true, false), (false, true), (true, true)] {
        let mut des = base(0, "lutff_0/in_0")?;
        des.lut(tx, ty, 0, follows(0));
        let (ax, ay, az) = des.pin("cm36", "A3")?;
        if ie {
            des.func(ax, ay, &format!("IoCtrl.IE_{az}"))?;
        }
        if ren {
            des.func(ax, ay, &format!("IoCtrl.REN_{az}"))?;
        }
        push(format!("input-ie{}-ren{}", ie as u8, ren as u8), "unknown", des)?;
    }
    // The flip-flop, clocked by D1 through global network 1 (the chip
    // database's gbufpin) and, with `colbuf`, the column buffer that feeds
    // TILE (its colbuf entry); its enable and set/reset left unrouted.
    let clocked = |opts: &[&str], negclk: bool, clock: bool, colbuf: bool| -> Result<Design, String> {
        let mut des = base(0, "lutff_0/in_0")?;
        des.lut(tx, ty, 0, follows(0));
        des.option(tx, ty, 0, "DffEnable")?;
        for o in opts {
            des.option(tx, ty, 0, o)?;
        }
        if negclk {
            des.func(tx, ty, "NegClk")?;
        }
        if clock {
            let (cx, cy, cz) = des.pin("cm36", "D1")?;
            let glb = db.gbufpin.iter().find(|g| (g.0, g.1, g.2) == (cx, cy, cz)).ok_or("D1 drives no global network")?.3;
            des.extra_bit(&format!("padin_glb_netwk.{glb}"))?;
            des.route((tx, ty, &format!("glb_netwk_{glb}")), (tx, ty, "lutff_global/clk"))?;
            if colbuf {
                for c in db.colbuf.iter().filter(|c| (c[2], c[3]) == (tx, ty)) {
                    des.func(c[0], c[1], &format!("ColBufCtrl.glb_netwk_{glb}"))?;
                }
            }
        }
        Ok(des)
    };
    push("dff-unclocked".into(), "no-echo", clocked(&[], false, false, false)?)?;
    for colbuf in [false, true] {
        let tag = if colbuf { "" } else { "-no-colbuf" };
        push(format!("dff-posedge{tag}"), "unknown", clocked(&[], false, true, colbuf)?)?;
        push(format!("dff-negclk{tag}"), "unknown", clocked(&[], true, true, colbuf)?)?;
        push(format!("dff-async-sr{tag}"), "unknown", clocked(&["AsyncSetReset"], false, true, colbuf)?)?;
        push(format!("dff-async-set{tag}"), "unknown", clocked(&["AsyncSetReset", "Set_NoReset"], false, true, colbuf)?)?;
    }
    // The block RAM nearest B3 ([`RAM`]), clocked by D1 through global
    // network 1 and its column buffer. Words 0 and 1 hold `init`; `re`
    // drives RE from a cell whose LUT is all ones (unrouted, icebox_vlog
    // says it reads 0); `powerup` sets RamConfig.PowerUp (icebox_vlog then
    // shows no RAM).
    let ram_base = |init: [u16; 2], data_bit: usize, re: bool, powerup: bool| -> Result<Design, String> {
        let mut des = Design::new(d, db, sem, &g);
        let (rx, ry) = RAM;
        let (ax, ay, az) = des.pin("cm36", "A3")?;
        let (bx, by, bz) = des.pin("cm36", "B3")?;
        des.func(ax, ay, &format!("IOB_{az}.PINTYPE_0"))?;
        des.func(bx, by, &format!("IOB_{bz}.PINTYPE_3"))?;
        des.func(bx, by, &format!("IOB_{bz}.PINTYPE_4"))?;
        let mut words = vec![0u16; 256];
        words[0] = init[0];
        words[1] = init[1];
        des.ram.push(super::asc::AscRam { x: rx, y: ry, words });
        let raddr = des.ram_port(rx, ry, "RADDR[0]")?;
        des.route((ax, ay, &format!("io_{az}/D_IN_0")), (raddr.0, raddr.1, &raddr.2))?;
        let rdata = des.ram_port(rx, ry, &format!("RDATA[{data_bit}]"))?;
        des.route((rdata.0, rdata.1, &rdata.2), (bx, by, &format!("io_{bz}/D_OUT_0")))?;
        if re {
            let p = des.ram_port(rx, ry, "RE")?;
            des.lut(ONES.0, ONES.1, 7, 0xffff);
            des.route((ONES.0, ONES.1, "lutff_7/out"), (p.0, p.1, &p.2))?;
        }
        let clk = des.ram_port(rx, ry, "RCLK")?;
        clock(&mut des, (clk.0, clk.1, &clk.2))?;
        if powerup {
            des.func(rx, ry, "RamConfig.PowerUp")?;
        }
        Ok(des)
    };
    push("ram-read".into(), "echo", ram_base([0, 0xffff], 0, true, false)?)?;
    push("ram-read-powerup".into(), "no-echo", ram_base([0, 0xffff], 0, true, true)?)?;
    push("ram-read-re-unrouted".into(), "no-echo", ram_base([0, 0xffff], 0, false, false)?)?;
    for k in [0, 5, 10, 15] {
        push(format!("ram-read-bit-{k}"), "echo", ram_base([0, 1 << k], k, true, false)?)?;
        push(format!("ram-read-bit-{k}-other"), "no-echo", ram_base([0, 1 << ((k + 1) % 16)], k, true, false)?)?;
    }
    // Write A3 into word 0 bit 0 every clock, and read it back: WE from a
    // cell of ones, the write address 0 (unrouted), and MASK[0] unrouted
    // or, with `masked`, from the cell of ones.
    for masked in [false, true] {
        let mut des = Design::new(d, db, sem, &g);
        let (rx, ry) = RAM;
        des.ram.push(super::asc::AscRam { x: rx, y: ry, words: vec![0; 256] });
        let (ax, ay, az) = des.pin("cm36", "A3")?;
        let (bx, by, bz) = des.pin("cm36", "B3")?;
        des.func(ax, ay, &format!("IOB_{az}.PINTYPE_0"))?;
        des.func(bx, by, &format!("IOB_{bz}.PINTYPE_3"))?;
        des.func(bx, by, &format!("IOB_{bz}.PINTYPE_4"))?;
        let wdata = des.ram_port(rx, ry, "WDATA[0]")?;
        des.route((ax, ay, &format!("io_{az}/D_IN_0")), (wdata.0, wdata.1, &wdata.2))?;
        let rdata = des.ram_port(rx, ry, "RDATA[0]")?;
        des.route((rdata.0, rdata.1, &rdata.2), (bx, by, &format!("io_{bz}/D_OUT_0")))?;
        let mut ones = vec!["RE", "WE"];
        if masked {
            ones.push("MASK[0]");
        }
        for (i, port) in ones.iter().enumerate() {
            let p = des.ram_port(rx, ry, port)?;
            let lc = 7 - i;
            des.lut(ONES.0, ONES.1, lc, 0xffff);
            des.route((ONES.0, ONES.1, &format!("lutff_{lc}/out")), (p.0, p.1, &p.2))?;
        }
        for port in ["RCLK", "WCLK"] {
            let c = des.ram_port(rx, ry, port)?;
            clock(&mut des, (c.0, c.1, &c.2))?;
        }
        push(format!("ram-write{}", if masked { "-masked" } else { "" }), if masked { "unknown" } else { "echo" }, des)?;
    }
    // The carry: cell 0's carry out is A3 (on in_1) and its carry in;
    // cell 1 passes its in_3, the carry, to B3.
    for set in [true, false] {
        let mut des = base(1, "lutff_0/in_1")?;
        des.option(tx, ty, 0, "CarryEnable")?;
        if set {
            des.func(tx, ty, "CarryInSet")?;
        }
        des.route((tx, ty, "lutff_0/cout"), (tx, ty, "lutff_1/in_3"))?;
        des.lut(tx, ty, 1, follows(3));
        // co = (in_1 & in_2) | ((in_1 | in_2) & ci), in_2 unrouted.
        push(format!("carry-in-set-{}", set as u8), if set { "echo" } else { "unknown" }, des)?;
    }
    // The RAM's other modes, predicted by `ram_mode_bit`: reading, the
    // predicted bit alone echoes and every other bit does not; writing, A3
    // lands on the predicted bit and not on its neighbour.
    for (mode, addr, pin) in [(1, 0x33 | 1 << 8, 6), (2, 0x66 | 2 << 8, 9), (3, 0x99 | 5 << 8, 11)] {
        let p = ram_mode_bit(mode, addr, pin).ok_or("not a data pin")?;
        out.push(Experiment { name: format!("ram-mode{mode}-read"), expect: "echo", bits: ram_probe(d, db, sem, &g, mode, addr, pin, &ram_words(|q| q == p))? });
        out.push(Experiment { name: format!("ram-mode{mode}-read-others"), expect: "no-echo", bits: ram_probe(d, db, sem, &g, mode, addr, pin, &ram_words(|q| q != p))? });
        out.push(Experiment { name: format!("ram-mode{mode}-write"), expect: "echo", bits: ram_write_probe(d, db, sem, &g, mode, addr, pin, p)? });
        out.push(Experiment { name: format!("ram-mode{mode}-write-next"), expect: "no-echo", bits: ram_write_probe(d, db, sem, &g, mode, addr, pin, p ^ 1)? });
    }
    // The RAM cascade bits (icebox_vlog shows them changing nothing): each
    // alone and all four, set here or below, reading and writing, with the
    // ports of here, of below or of both driven, and the RAM here or below
    // observed.
    let mut sets: Vec<(String, Vec<(bool, usize)>)> = vec![("none".into(), vec![])];
    for below in [false, true] {
        let at = if below { "below" } else { "here" };
        for k in 4..8 {
            sets.push((format!("{at}{k}"), vec![(below, k)]));
        }
        sets.push((format!("{at}all"), (4..8).map(|k| (below, k)).collect()));
    }
    for write in [false, true] {
        for (from_below, ports) in [(false, (true, false)), (true, (true, false)), (false, (false, true)), (true, (false, true))] {
            for (name, set) in &sets {
                let label = format!(
                    "ram-cascade-{}-{}-ports-{}-{name}",
                    if write { "write" } else { "read" },
                    if from_below { "below" } else { "here" },
                    if ports.0 { "here" } else { "below" },
                );
                // With no cascade bit, a RAM reads and writes through its
                // own ports alone.
                let expect = match (set.is_empty(), from_below == ports.1, write) {
                    (true, true, false) => if from_below { "no-echo" } else { "echo" },
                    (true, true, true) => "echo",
                    (true, false, _) => "no-echo",
                    _ => "unknown",
                };
                out.push(Experiment { name: label, expect, bits: ram_cascade_probe(d, db, sem, &g, write, from_below, ports, set)? });
            }
        }
    }
    // Bits in both RAMs: CBIT_6 (which stops a RAM reading through its own
    // ports) in the RAM observed, with each other bit, or all, in the RAM
    // whose read port is driven; and CBIT_6 with each other bit in one RAM.
    for (from_below, ports) in [(true, (true, false)), (false, (false, true))] {
        let observed = from_below;
        let driven = !from_below;
        for others in [vec![4], vec![5], vec![7], vec![4, 5, 7]] {
            let mut set = vec![(observed, 6)];
            set.extend(others.iter().map(|&k| (driven, k)));
            let tag = others.iter().map(|k| k.to_string()).collect::<String>();
            let name = format!("ram-cascade-pair-read-{}-6-{}-{tag}", if observed { "below" } else { "here" }, if driven { "below" } else { "here" });
            out.push(Experiment { name, expect: "unknown", bits: ram_cascade_probe(d, db, sem, &g, false, from_below, ports, &set)? });
        }
    }
    // Each bit, watching other data pins of the RAM read through its own
    // ports.
    for pin in [7, 8, 15] {
        for k in [None, Some(4), Some(5), Some(6), Some(7)] {
            let set: Vec<(bool, usize)> = k.into_iter().map(|k| (false, k)).collect();
            let name = format!("ram-cascade-pin{pin}-{}", k.map_or("none".into(), |k| format!("here{k}")));
            let expect = if k.is_none() { "echo" } else { "unknown" };
            out.push(Experiment { name, expect, bits: ram_cascade_probe_pin(d, db, sem, &g, false, false, (true, false), &set, pin)? });
        }
    }
    for below in [false, true] {
        for k in [4, 5, 7] {
            let at = if below { "below" } else { "here" };
            let name = format!("ram-cascade-pair-read-{at}-6-{at}-{k}");
            out.push(Experiment { name, expect: "unknown", bits: ram_cascade_probe(d, db, sem, &g, false, below, (!below, below), &[(below, 6), (below, k)])? });
        }
    }
    Ok(out)
}

/// What the board did: `(name, expected, observed, bitstream length, hash)`.
pub type Results = Vec<(String, String, String, usize, u64)>;

/// Runs the experiments on the board.
pub fn run(ip: &Icesprog, serial: &PathBuf, exps: &[Experiment], log: &mut dyn FnMut(&str)) -> Result<Results, String> {
    let mut out = Vec::new();
    let probe: Vec<u8> = (0..64u8).map(|i| 0x20 + i).collect();
    for e in exps {
        ip.program(&e.bits)?;
        let mut s = Serial::open(serial)?;
        let seen = s.echo(&probe)?;
        log(&format!("{:<24} expect {:<8} saw {seen}", e.name, e.expect));
        out.push((e.name.clone(), e.expect.to_string(), seen, e.bits.len(), fnv64(&e.bits)));
    }
    Ok(out)
}

pub fn results_text(board: &str, flash: &str, r: &Results) -> String {
    let mut s = String::new();
    s.push_str("# What sv/src/arch/ice40/board.rs saw on the board: experiment, expected, observed,\n");
    s.push_str("# and the bitstream's length and hash. Regenerate with the ice40_board example.\n");
    writeln!(s, "board {board} {flash}").unwrap();
    for (n, e, o, len, h) in r {
        writeln!(s, "exp {n} {e} {o} {len} {h:016x}").unwrap();
    }
    s
}

pub fn results_from_text(text: &str) -> Result<Results, String> {
    let mut out = Vec::new();
    for line in text.lines() {
        let w: Vec<&str> = line.split_whitespace().collect();
        if w.first() == Some(&"exp") && w.len() == 6 {
            out.push((
                w[1].to_string(),
                w[2].to_string(),
                w[3].to_string(),
                w[4].parse().map_err(|_| "bad length")?,
                u64::from_str_radix(w[5], 16).map_err(|_| "bad hash")?,
            ));
        }
    }
    Ok(out)
}

/// What the results show, each with whether it held and the experiments
/// it rests on.
pub fn findings(r: &Results) -> Vec<(String, bool, Vec<String>)> {
    let seen = |n: &str| r.iter().find(|x| x.0 == n).map(|x| x.2.as_str());
    let all = |prefix: &str, want: &dyn Fn(&str) -> bool| -> (bool, Vec<String>) {
        let v: Vec<_> = r.iter().filter(|x| x.0.starts_with(prefix)).collect();
        (!v.is_empty() && v.iter().all(|x| want(&x.2)), v.iter().map(|x| x.0.clone()).collect())
    };
    let echo = |s: &str| s == "echo";
    let mut out = Vec::new();
    let mut add = |what: &str, (ok, names): (bool, Vec<String>)| out.push((what.to_string(), ok, names));
    add("every logic cell's LUT is where the description says: each cell passes A3 to B3", all("lut-cell-", &echo));
    let inputs: Vec<_> = r.iter().filter(|x| x.0.starts_with("lut-input-")).collect();
    let names = |v: &[&(String, String, String, usize, u64)]| v.iter().map(|x| x.0.clone()).collect::<Vec<_>>();
    add(
        "each LUT input's bits are where the description says: a table following input j passes it, the inverse table does not",
        (
            !inputs.is_empty() && inputs.iter().all(|x| if x.0.ends_with("-inverse") { x.2 != "echo" } else { x.2 == "echo" }),
            names(&inputs),
        ),
    );
    let singles: Vec<_> = inputs.iter().copied().filter(|x| x.0.ends_with("-single")).collect();
    add(
        "an unrouted LUT input reads 0: a table true only at index 2^j passes input j",
        (!singles.is_empty() && singles.iter().all(|x| x.2 == "echo"), names(&singles)),
    );
    let pair = |a: &str, b: &str| vec![a.to_string(), b.to_string()];
    add(
        "IoCtrl.IE set turns a pin's input off (A3 then reads nothing); IoCtrl.REN does not stop a driven input",
        (seen("input-ie1-ren0") == Some("silent") && seen("input-ie0-ren1") == Some("echo") && seen("lut-cell-0") == Some("echo"), pair("input-ie1-ren0", "input-ie0-ren1")),
    );
    add(
        "the global network reaches a tile only through its column buffer (ColBufCtrl in the colbuf tile)",
        (seen("dff-posedge-no-colbuf") == Some("silent") && seen("dff-posedge") == Some("echo"), pair("dff-posedge-no-colbuf", "dff-posedge")),
    );
    add(
        "the flip-flop clocks from D1 (the iCELink's 12 MHz) on either edge, with NegClk",
        (seen("dff-posedge") == Some("echo") && seen("dff-negclk") == Some("echo") && seen("dff-unclocked") != Some("echo"), vec!["dff-posedge".into(), "dff-negclk".into(), "dff-unclocked".into()]),
    );
    add(
        "an unrouted flip-flop enable reads 1 and an unrouted set/reset 0, synchronous or not, set or reset",
        (["dff-posedge", "dff-async-sr", "dff-async-set"].iter().all(|n| seen(n) == Some("echo")), vec!["dff-posedge".into(), "dff-async-sr".into(), "dff-async-set".into()]),
    );
    if seen("ram-read").is_some() {
        let bits: Vec<_> = r.iter().filter(|x| x.0.starts_with("ram-read-bit-")).collect();
        add(
            "a block RAM reads its .ram_data contents: word 1 at RADDR[0] = A3, clocked from D1, RE driven to 1",
            (seen("ram-read") == Some("echo"), vec!["ram-read".into()]),
        );
        add(
            "each RDATA bit is the word's bit of the same number, as .ram_data and the port map say",
            (!bits.is_empty() && bits.iter().all(|x| (x.2 == "echo") != x.0.ends_with("-other")), bits.iter().map(|x| x.0.clone()).collect()),
        );
        add(
            "RamConfig.PowerUp set turns the LP1K's block RAM off, and an unrouted RE reads 0 (as icebox_vlog says)",
            (seen("ram-read-powerup") == Some("silent") && seen("ram-read-re-unrouted") == Some("silent"), pair("ram-read-powerup", "ram-read-re-unrouted")),
        );
        add(
            "the write port writes: WDATA[0] = A3 into word 0 each clock, read back; MASK[0] set stops the bit being written, and unrouted it reads 0",
            (seen("ram-write") == Some("echo") && seen("ram-write-masked") == Some("silent"), pair("ram-write", "ram-write-masked")),
        );
    }
    let modes: Vec<_> = r.iter().filter(|x| x.0.starts_with("ram-mode")).collect();
    if !modes.is_empty() {
        let reads: Vec<_> = modes.iter().copied().filter(|x| x.0.contains("-read")).collect();
        let writes: Vec<_> = modes.iter().copied().filter(|x| x.0.contains("-write")).collect();
        let held = |v: &[&(String, String, String, usize, u64)]| v.iter().all(|x| (x.2 == "echo") == (x.1 == "echo"));
        add(
            "in 512x8, 1024x4 and 2048x2 a data pin reads the bit board::ram_mode_bit gives, and no other",
            (held(&reads), names(&reads)),
        );
        add(
            "in the same write modes, a data pin writes the bit board::ram_mode_bit gives, and not its neighbour",
            (held(&writes), names(&writes)),
        );
    }
    let cascade: Vec<_> = r.iter().filter(|x| x.0.starts_with("ram-cascade-")).collect();
    if !cascade.is_empty() {
        // Experiments where a RAM is read or written through its own ports.
        let own = |x: &&(String, String, String, usize, u64)| {
            let n = &x.0;
            n.starts_with("ram-cascade-pin") || n.contains("-here-ports-here-") || n.contains("-below-ports-below-")
        };
        let observed_bit6 = |x: &&(String, String, String, usize, u64)| {
            let n = &x.0;
            n.starts_with("ram-cascade-pin") && n.ends_with("here6")
                || n.contains("-read-here-ports-here-") && (n.ends_with("here6") || n.ends_with("hereall"))
                || n.contains("-read-below-ports-below-") && (n.ends_with("below6") || n.ends_with("belowall"))
        };
        let own_reads6: Vec<_> = cascade.iter().copied().filter(|x| own(x) && observed_bit6(x)).collect();
        let own_rest: Vec<_> = cascade.iter().copied().filter(|x| own(x) && !observed_bit6(x)).collect();
        let crossed: Vec<_> = cascade.iter().copied().filter(|x| !own(x)).collect();
        add(
            "RamCascade.CBIT_6 stops a block RAM reading through its own ports (every data pin), in either RAM of a pair",
            (own_reads6.iter().all(|x| x.2 == "silent"), names(&own_reads6)),
        );
        add(
            "RamCascade.CBIT_4, 5 and 7, alone or together, and CBIT_6 for writing, change nothing a RAM's own read or write shows (the RAM below reads its own, inverted, contents)",
            (own_rest.iter().all(|x| x.2 == if x.0.contains("-read-below-") { "other:64" } else { "echo" }), names(&own_rest)),
        );
        add(
            "no cascade setting tried lets one RAM read or write through the other's ports",
            (crossed.iter().all(|x| x.2 == "silent"), names(&crossed)),
        );
    }
    add(
        "the carry is (in_1 & in_2) | ((in_1 | in_2) & ci): with CarryInSet ci is 1 and the carry follows in_1; without it the first cell's ci is 0",
        (seen("carry-in-set-1") == Some("echo") && seen("carry-in-set-0") == Some("silent"), pair("carry-in-set-1", "carry-in-set-0")),
    );
    out
}

/// Clocks `to` from D1: its global network (the chip database's gbufpin),
/// the pad's padin bit, and the column buffers that feed `to`'s tile.
fn clock_from_d1(des: &mut Design, to: (usize, usize, &str)) -> Result<(), String> {
    let (cx, cy, cz) = des.pin("cm36", "D1")?;
    let glb = des.db.gbufpin.iter().find(|g| (g.0, g.1, g.2) == (cx, cy, cz)).ok_or("D1 drives no global network")?.3;
    let bit = format!("padin_glb_netwk.{glb}");
    let e = des.db.extra_bits.iter().find(|x| x.0 == bit).map(|x| (x.1, x.2, x.3)).ok_or("no padin bit")?;
    if !des.extra.contains(&e) {
        des.extra.push(e);
    }
    des.route((to.0, to.1, &format!("glb_netwk_{glb}")), to)?;
    let cbs: Vec<[usize; 4]> = des.db.colbuf.iter().filter(|c| (c[2], c[3]) == (to.0, to.1)).copied().collect();
    for c in cbs {
        des.func(c[0], c[1], &format!("ColBufCtrl.glb_netwk_{glb}"))?;
    }
    Ok(())
}

/// The cells that give constant ones: the cells of [`ONES`] and the tile
/// below it, each a LUT of all ones.
const ONE_CELLS: [(usize, usize, usize); 16] = {
    let mut v = [(0, 0, 0); 16];
    let mut i = 0;
    while i < 16 {
        v[i] = (ONES.0, ONES.1 - i / 8, i % 8);
        i += 1;
    }
    v
};

/// A probe of the RAM's read port in read mode `mode` (RamConfig.CBIT_2 is
/// its low bit, CBIT_3 its high, as icebox_vlog shows): read address `addr`
/// fixed (its ones from cells of ones, its zeros unrouted), RE from a cell
/// of ones, and B3 = A3 AND RDATA[pin], so the board echoes when the RAM
/// gives 1 there. `words` are the contents, as `.ram_data` holds them.
pub fn ram_probe(d: &Device, db: &Chipdb, sem: &Semantics, g: &Graph, mode: usize, addr: usize, pin: usize, words: &[u16]) -> Result<Vec<u8>, String> {
    let mut des = Design::new(d, db, sem, g);
    let (rx, ry) = RAM;
    let (tx, ty) = TILE;
    let (ax, ay, az) = des.pin("cm36", "A3")?;
    let (bx, by, bz) = des.pin("cm36", "B3")?;
    des.func(ax, ay, &format!("IOB_{az}.PINTYPE_0"))?;
    des.func(bx, by, &format!("IOB_{bz}.PINTYPE_3"))?;
    des.func(bx, by, &format!("IOB_{bz}.PINTYPE_4"))?;
    des.ram.push(super::asc::AscRam { x: rx, y: ry, words: words.to_vec() });
    for (bit, f) in [(1, "RamConfig.CBIT_2"), (2, "RamConfig.CBIT_3")] {
        if mode & bit != 0 {
            des.func(rx, ry + 1, f)?;
        }
    }
    let mut ones = ONE_CELLS.iter();
    let mut one = |des: &mut Design, port: &str| -> Result<(), String> {
        let &(x, y, lc) = ones.next().ok_or("out of cells of ones")?;
        let p = des.ram_port(rx, ry, port)?;
        des.lut(x, y, lc, 0xffff);
        des.route((x, y, &format!("lutff_{lc}/out")), (p.0, p.1, &p.2))
    };
    one(&mut des, "RE")?;
    for k in 0..11 {
        if addr >> k & 1 != 0 {
            one(&mut des, &format!("RADDR[{k}]"))?;
        }
    }
    let c = des.ram_port(rx, ry, "RCLK")?;
    clock_from_d1(&mut des, (c.0, c.1, &c.2))?;
    let r = des.ram_port(rx, ry, &format!("RDATA[{pin}]"))?;
    des.route((ax, ay, &format!("io_{az}/D_IN_0")), (tx, ty, "lutff_0/in_0"))?;
    des.route((r.0, r.1, &r.2), (tx, ty, "lutff_0/in_1"))?;
    des.lut(tx, ty, 0, 0x8888);
    des.route((tx, ty, "lutff_0/out"), (bx, by, &format!("io_{bz}/D_OUT_0")))?;
    des.bitstream()
}

/// Contents with the bits numbered `p` (word p / 16, bit p % 16) set where
/// `set(p)`.
pub fn ram_words(set: impl Fn(usize) -> bool) -> Vec<u16> {
    (0..256).map(|w| (0..16).fold(0u16, |v, k| v | (set(w * 16 + k) as u16) << k)).collect()
}

/// Finds which stored bit the RAM gives at `addr` on RDATA[pin] in read
/// mode `mode`: twelve probes, with the bits whose number has bit t set,
/// read off the number; then a probe with that bit alone, which must echo,
/// and one with every other bit, which must not. `None` if the pin gives
/// no stored bit there.
pub fn ram_locate(probe: &mut dyn FnMut(&[u16]) -> Result<bool, String>) -> Result<Option<usize>, String> {
    let mut p = 0;
    for t in 0..12 {
        if probe(&ram_words(|q| q >> t & 1 != 0))? {
            p |= 1 << t;
        }
    }
    let alone = probe(&ram_words(|q| q == p))?;
    let others = probe(&ram_words(|q| q != p))?;
    Ok((alone && !others).then_some(p))
}

/// A probe of the RAM's write port in write mode `mode` (RamConfig.CBIT_0
/// its low bit, CBIT_1 its high): A3 written each clock at `waddr` on
/// WDATA[wpin], the contents all zeros, and the stored bit `p` (word p /
/// 16, bit p % 16) read back in 256x16 onto B3. The board echoes when that
/// write lands on `p`.
pub fn ram_write_probe(d: &Device, db: &Chipdb, sem: &Semantics, g: &Graph, mode: usize, waddr: usize, wpin: usize, p: usize) -> Result<Vec<u8>, String> {
    let mut des = Design::new(d, db, sem, g);
    let (rx, ry) = RAM;
    let (ax, ay, az) = des.pin("cm36", "A3")?;
    let (bx, by, bz) = des.pin("cm36", "B3")?;
    des.func(ax, ay, &format!("IOB_{az}.PINTYPE_0"))?;
    des.func(bx, by, &format!("IOB_{bz}.PINTYPE_3"))?;
    des.func(bx, by, &format!("IOB_{bz}.PINTYPE_4"))?;
    des.ram.push(super::asc::AscRam { x: rx, y: ry, words: vec![0; 256] });
    for (bit, f) in [(1, "RamConfig.CBIT_0"), (2, "RamConfig.CBIT_1")] {
        if mode & bit != 0 {
            des.func(rx, ry + 1, f)?;
        }
    }
    let mut ones = ONE_CELLS.iter().cycle();
    let mut one = |des: &mut Design, port: &str| -> Result<(), String> {
        let &(x, y, lc) = ones.next().unwrap();
        let q = des.ram_port(rx, ry, port)?;
        des.lut(x, y, lc, 0xffff);
        des.route((x, y, &format!("lutff_{lc}/out")), (q.0, q.1, &q.2))
    };
    one(&mut des, "RE")?;
    one(&mut des, "WE")?;
    for k in 0..11 {
        if waddr >> k & 1 != 0 {
            one(&mut des, &format!("WADDR[{k}]"))?;
        }
    }
    for k in 0..8 {
        if (p / 16) >> k & 1 != 0 {
            one(&mut des, &format!("RADDR[{k}]"))?;
        }
    }
    for port in ["RCLK", "WCLK"] {
        let c = des.ram_port(rx, ry, port)?;
        clock_from_d1(&mut des, (c.0, c.1, &c.2))?;
    }
    let w = des.ram_port(rx, ry, &format!("WDATA[{wpin}]"))?;
    des.route((ax, ay, &format!("io_{az}/D_IN_0")), (w.0, w.1, &w.2))?;
    let r = des.ram_port(rx, ry, &format!("RDATA[{}]", p % 16))?;
    des.route((r.0, r.1, &r.2), (bx, by, &format!("io_{bz}/D_OUT_0")))?;
    des.bitstream()
}

/// [`ram_locate`] within word `w`: four probes with the word's bits whose
/// number has bit t set, then the same two checks (the found bit alone
/// echoes, every other stored bit does not).
pub fn ram_locate_in_word(w: usize, probe: &mut dyn FnMut(&[u16]) -> Result<bool, String>) -> Result<Option<usize>, String> {
    let mut k = 0;
    for t in 0..4 {
        if probe(&ram_words(|q| q / 16 == w && (q % 16) >> t & 1 != 0))? {
            k |= 1 << t;
        }
    }
    let p = w * 16 + k;
    let alone = probe(&ram_words(|q| q == p))?;
    let others = probe(&ram_words(|q| q != p))?;
    Ok((alone && !others).then_some(p))
}

/// The stored bit (word p / 16, bit p % 16) a block RAM's data pin `pin`
/// reads at `addr` in read mode `mode` (0 256x16, 1 512x8, 2 1024x4, 3
/// 2048x2), as the board showed (`record-ram-modes.txt`): the word is the
/// address's low eight bits; in mode m > 0 the data pins are those with
/// pin % 2^m = 2^(m-1) - 1 (the even pins, then 1, 5, 9, 13, then 3, 11),
/// each reading bit (pin with its low m bits clear) + (addr >> 8). `None`
/// for a pin that carries no data in the mode. The write modes are taken to
/// be the same, which the `ram-mode*-write` experiments check.
pub fn ram_mode_bit(mode: usize, addr: usize, pin: usize) -> Option<usize> {
    let word = addr & 255;
    if mode == 0 {
        return Some(word * 16 + pin);
    }
    let n = 1 << mode;
    (pin % n == n / 2 - 1).then(|| word * 16 + (pin & !(n - 1)) + ((addr >> 8) & (n - 1)))
}

/// A mapped point of a block RAM read mode: `(mode, addr, pin, stored bit)`.
pub type RamPoint = (usize, usize, usize, Option<usize>);

/// The `bit` lines of `record-ram-modes.txt` (the ice40_board example's
/// `--ram-modes`); `pin` lines (the screen with every bit one) are left out.
pub fn ram_points_from_text(text: &str) -> Vec<RamPoint> {
    text.lines()
        .filter_map(|l| {
            let w: Vec<&str> = l.split_whitespace().collect();
            if w.first() != Some(&"bit") || w.len() != 5 {
                return None;
            }
            Some((w[1].parse().ok()?, w[2].parse().ok()?, w[3].parse().ok()?, w[4].parse().ok()))
        })
        .collect()
}

/// The block RAM below [`RAM`] in its column, for the cascade experiments.
pub const RAM_BELOW: (usize, usize) = (RAM.0, RAM.1 - 2);

/// A probe of the RAM cascade bits, with [`RAM`] ("here") and the RAM
/// below it in its column.
///
/// Reading (`write` false): here holds 0 in word 0 and ones in word 1, the
/// RAM below the opposite; the read ports of the RAMs in `ports` (here,
/// below) have RADDR[0] = A3, RE from a cell of ones and RCLK from D1, and
/// B3 is RDATA[0] of here or, with `from_below`, the RAM below. B3 follows
/// A3 (echo) if here's contents are read at A3, and is A3 inverted (other)
/// if the RAM below's are.
///
/// Writing (`write` true): both RAMs hold zeros; the write ports of the
/// RAMs in `ports` write A3 into word 0 bit 0 each clock (WDATA[0] = A3, WE
/// from a cell of ones, WCLK from D1), and the read port of the RAM read
/// reads word 0 onto B3: echo if A3 is being written into it.
///
/// `cascade` lists `(below, k)`: RamCascade.CBIT_k set in here or below.
pub fn ram_cascade_probe(
    d: &Device,
    db: &Chipdb,
    sem: &Semantics,
    g: &Graph,
    write: bool,
    from_below: bool,
    ports: (bool, bool),
    cascade: &[(bool, usize)],
) -> Result<Vec<u8>, String> {
    ram_cascade_probe_pin(d, db, sem, g, write, from_below, ports, cascade, 0)
}

/// [`ram_cascade_probe`] observing RDATA[pin] (the contents' words are all
/// zeros or all ones, so any pin reads alike).
#[allow(clippy::too_many_arguments)]
pub fn ram_cascade_probe_pin(
    d: &Device,
    db: &Chipdb,
    sem: &Semantics,
    g: &Graph,
    write: bool,
    from_below: bool,
    ports: (bool, bool),
    cascade: &[(bool, usize)],
    pin: usize,
) -> Result<Vec<u8>, String> {
    let mut des = Design::new(d, db, sem, g);
    let rams = [RAM, RAM_BELOW];
    let (ax, ay, az) = des.pin("cm36", "A3")?;
    let (bx, by, bz) = des.pin("cm36", "B3")?;
    des.func(ax, ay, &format!("IOB_{az}.PINTYPE_0"))?;
    des.func(bx, by, &format!("IOB_{bz}.PINTYPE_3"))?;
    des.func(bx, by, &format!("IOB_{bz}.PINTYPE_4"))?;
    for (i, &(x, y)) in rams.iter().enumerate() {
        let mut v = vec![0u16; 256];
        if !write {
            (v[0], v[1]) = if i == 0 { (0, 0xffff) } else { (0xffff, 0) };
        }
        des.ram.push(super::asc::AscRam { x, y, words: v });
    }
    for &(below, k) in cascade {
        let (x, y) = rams[below as usize];
        des.func(x, y + 1, &format!("RamCascade.CBIT_{k}"))?;
    }
    let mut ones = ONE_CELLS.iter();
    let mut one = |des: &mut Design, x: usize, y: usize, port: &str| -> Result<(), String> {
        let &(ox, oy, lc) = ones.next().ok_or("out of cells of ones")?;
        let q = des.ram_port(x, y, port)?;
        des.lut(ox, oy, lc, 0xffff);
        des.route((ox, oy, &format!("lutff_{lc}/out")), (q.0, q.1, &q.2))
    };
    let a3 = (ax, ay, format!("io_{az}/D_IN_0"));
    for (i, &(x, y)) in rams.iter().enumerate() {
        if !(if i == 0 { ports.0 } else { ports.1 }) {
            continue;
        }
        let (data, en, clk) = if write { ("WDATA[0]", "WE", "WCLK") } else { ("RADDR[0]", "RE", "RCLK") };
        let q = des.ram_port(x, y, data)?;
        des.route((a3.0, a3.1, &a3.2), (q.0, q.1, &q.2))?;
        one(&mut des, x, y, en)?;
        let c = des.ram_port(x, y, clk)?;
        clock_from_d1(&mut des, (c.0, c.1, &c.2))?;
    }
    let (sx, sy) = rams[from_below as usize];
    if write {
        one(&mut des, sx, sy, "RE")?;
        let c = des.ram_port(sx, sy, "RCLK")?;
        clock_from_d1(&mut des, (c.0, c.1, &c.2))?;
    }
    let r = des.ram_port(sx, sy, &format!("RDATA[{pin}]"))?;
    des.route((r.0, r.1, &r.2), (bx, by, &format!("io_{bz}/D_OUT_0")))?;
    des.bitstream()
}
