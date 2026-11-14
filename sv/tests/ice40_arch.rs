//! The iCE40 descriptions (`sv/arch/ice40-*`, one per part): each agrees
//! with its records, its sources agree with each other, and it predicts
//! what the tools do. The tests that need `icepack` or the `icebox` tools
//! say so and pass without them; `ICE40_REQUIRE_TOOLS=1` makes their
//! absence a failure.

use epitome_sv::arch::ice40::bitstream::{self, PackOptions};
use epitome_sv::arch::ice40::chipdb::Chipdb;
use epitome_sv::arch::ice40::experiment::random_config;
use epitome_sv::arch::ice40::icebox::{self, Graph, Icebox, Semantics, TILE};
use epitome_sv::arch::ice40::icepack::Icepack;
use epitome_sv::arch::ice40::{Device, PARTS, Part, asc::Asc, describe, experiment, fnv64};
use epitome_sv::hash::FxSet;
use epitome_sv::stimulus::Rng;
use std::path::PathBuf;

fn read(part: &Part, name: &str) -> String {
    std::fs::read_to_string(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("arch").join(part.dir).join(name)).unwrap()
}

fn records(part: &Part) -> (Device, Chipdb, Semantics) {
    (
        Device::from_record(&read(part, "record-icepack.txt")).unwrap(),
        Chipdb::from_text(&read(part, "record-chipdb.txt")).unwrap(),
        Semantics::from_text(&read(part, "record-icebox.txt")).unwrap(),
    )
}

/// Whether a test may go on without a tool.
fn skip(what: &str) -> bool {
    if std::env::var_os("ICE40_REQUIRE_TOOLS").is_some() {
        panic!("{what} not found");
    }
    eprintln!("skipped: {what} not found");
    true
}

#[test]
fn records_read_back_unchanged() {
    for part in &PARTS {
        let (d, db, sem) = records(part);
        assert_eq!(d.to_record(), read(part, "record-icepack.txt"), "{}", part.name);
        assert_eq!(db.to_text(), read(part, "record-chipdb.txt"), "{}", part.name);
        assert_eq!(sem.to_text(), read(part, "record-icebox.txt"), "{}", part.name);
    }
}

#[test]
fn description_is_up_to_date() {
    for part in &PARTS {
        let (d, db, sem) = records(part);
        let why = format!("{}: rerun the ice40_arch example with --replay", part.name);
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("arch").join(part.dir).join("record-board.txt");
        let board = std::fs::read_to_string(&path).ok().map(|t| epitome_sv::arch::ice40::board::results_from_text(&t).unwrap());
        let points = std::fs::read_to_string(path.with_file_name("record-ram-modes.txt")).map(|t| epitome_sv::arch::ice40::board::ram_points_from_text(&t)).unwrap_or_default();
        assert!(describe::to_json(&d, Some(&db), Some(&sem), board.as_ref().map(|b| (b, &points[..]))) == read(part, "description.json"), "{why}");
        assert!(describe::logic_cell_sv(part, &sem) == read(part, "logic_cell.sv"), "{why}");
    }
}

#[test]
fn sources_agree() {
    for part in &PARTS {
        let (d, db, sem) = records(part);
        let failed: Vec<_> = describe::checks(&d, Some(&db), Some(&sem)).into_iter().filter(|c| !c.1).collect();
        assert!(failed.is_empty(), "{}: {failed:?}", part.name);
    }
}

/// The packer makes the bitstreams icepack made for the recorded random
/// configurations (by length and hash), and unpacks them.
#[test]
fn packer_matches_recorded_icepack() {
    for part in &PARTS {
        let (d, _, _) = records(part);
        assert!(d.fingerprints.len() >= 16);
        for &(seed, len, hash) in &d.fingerprints {
            let (asc, opts) = random_config(&d, seed);
            let b = d.pack(&asc, &opts).unwrap();
            assert_eq!((b.len(), fnv64(&b)), (len, hash), "{} seed {seed}", part.name);
            let back = d.unpack(&b).unwrap();
            assert_eq!(d.pack(&back, &opts).unwrap(), b, "{} seed {seed}: unpacking and packing again", part.name);
        }
    }
}

/// The routing expands to nets each naming a tile's net once, and every
/// switch joins names its tile has.
#[test]
fn routing_is_consistent() {
    for (part, counts) in PARTS.iter().zip([(103_383, 201_460), (27_682, 53_808)]) {
        let (_, db, _) = records(part);
        let (nets, sws) = db.routing.expand();
        let mut names = FxSet::default();
        for net in &nets {
            for m in net {
                assert!(names.insert(m.clone()), "{}: {m:?} in two nets", part.name);
            }
        }
        for (x, y, sw) in &sws {
            assert!(names.contains(&(*x, *y, sw.dst.clone())), "switch to {} in {x} {y}", sw.dst);
            for (v, src) in &sw.opts {
                assert_eq!(v.len(), sw.bits.len());
                assert!(names.contains(&(*x, *y, src.clone())), "switch from {src} in {x} {y}");
            }
        }
        assert_eq!((nets.len(), sws.len()), counts, "{}", part.name);
    }
}

/// The generated logic cell models elaborate with epitome-sv.
#[test]
fn logic_cell_model_elaborates() {
    for part in &PARTS {
        let mut src = epitome_sv::model::Sources::default();
        src.text("logic_cell.sv", read(part, "logic_cell.sv"));
        epitome_sv::model::elaborate(&src, &["ice40_logic_cell"], |_| ()).unwrap();
    }
}

/// Fresh experiments find what the record says, and the packer matches
/// icepack byte for byte on fresh random configurations, with icepack's
/// unpacking giving back the configuration.
#[test]
fn fresh_icepack_experiments() {
    let Some(ip) = Icepack::find() else {
        skip("icepack");
        return;
    };
    for part in &PARTS {
        let (d, _, _) = records(part);
        let mut fresh = experiment::run(&ip, part.asc, 0, &mut |_| ()).unwrap();
        fresh.fingerprints = d.fingerprints.clone();
        assert!(fresh == d, "{}: the experiments now find something else: rerun the ice40_arch example", part.name);
        for seed in 1000..1024 {
            let (asc, opts) = random_config(&d, seed);
            let theirs = ip.pack(&asc.to_text(), &opts.args()).unwrap();
            assert!(d.pack(&asc, &opts).unwrap() == theirs, "{} seed {seed}", part.name);
            let unpacked = Asc::parse(&ip.unpack(&theirs).unwrap()).unwrap();
            let ours = d.unpack(&theirs).unwrap();
            assert_eq!(d.banks(&unpacked).unwrap().cram, d.banks(&ours).unwrap().cram, "{} seed {seed}", part.name);
            assert_eq!(bitstream::parse(&theirs).unwrap().blocks.len(), bitstream::parse(&d.pack(&ours, &opts).unwrap()).unwrap().blocks.len());
        }
        // A configuration with nothing but a comment and one tile, and every option.
        let mut asc = Asc::new(part.asc);
        asc.comment = Some(vec!["epitome".into()]);
        asc.tiles = d.unpack(&d.pack(&random_config(&d, 7).0, &PackOptions::default()).unwrap()).unwrap().tiles[..1].to_vec();
        for opts in [PackOptions { nosleep: true, freq: 2, no_bram: true }, PackOptions::default()] {
            assert!(d.pack(&asc, &opts).unwrap() == ip.pack(&asc.to_text(), &opts.args()).unwrap(), "{}", part.name);
        }
    }
}

/// The chip database made afresh is what the record holds, and the
/// compact routing gives back every net and switch it lists.
#[test]
fn fresh_chipdb() {
    let Some(ib) = Icebox::find() else {
        skip("icebox");
        return;
    };
    for part in &PARTS {
        let (db, raw) = Chipdb::parse(&ib.chipdb(part).unwrap()).unwrap();
        assert!(db == records(part).1, "{}: the chip database differs from the record", part.name);
        let (nets, sws) = db.routing.expand();
        let mut want: Vec<_> = raw
            .nets
            .iter()
            .map(|n| {
                let mut n = n.clone();
                n.sort();
                n
            })
            .collect();
        want.sort();
        assert!(nets == want, "{}", part.name);
        // Switches, by net: a net may have two names in one tile.
        let mut index = epitome_sv::hash::FxMap::default();
        for (i, net) in raw.nets.iter().enumerate() {
            for m in net {
                index.insert(m.clone(), i);
            }
        }
        let mut want_sws: Vec<_> = raw
            .switches
            .iter()
            .map(|(x, y, buffer, dst, bits, opts)| {
                let mut opts = opts.clone();
                opts.sort();
                (*x, *y, *buffer, *dst, bits.clone(), opts)
            })
            .collect();
        want_sws.sort();
        let mut got: Vec<_> = sws
            .into_iter()
            .map(|(x, y, s)| {
                let net = |n: &str| index[&(x, y, n.to_string())];
                let mut opts: Vec<_> = s.opts.iter().map(|(v, n)| (v.clone(), net(n))).collect();
                opts.sort();
                (x, y, s.buffer, net(&s.dst), s.bits.clone(), opts)
            })
            .collect();
        got.sort();
        assert!(got == want_sws, "{}", part.name);
    }
}

/// Random LUT contents written by the description's LUT bits are what
/// icebox_vlog reads back, in every cell.
#[test]
fn luts_predict_icebox() {
    let Some(ib) = Icebox::find() else {
        skip("icebox");
        return;
    };
    for part in &PARTS {
        let (d, _, sem) = records(part);
        let (db, raw) = Chipdb::parse(&ib.chipdb(part).unwrap()).unwrap();
        let g = Graph::new(&raw);
        let mut rng = Rng(0x1ce40);
        for lc in 0..8 {
            let table = rng.next() as u16;
            let mut bits = icebox::wire_cell(&g, &db, lc, false).unwrap();
            for i in 0..16 {
                if table >> i & 1 != 0 {
                    bits.insert((TILE.0, TILE.1, sem.lut[lc][i]));
                }
            }
            let v = ib.run("icebox_vlog", &[], &icebox::full_asc(&d, &bits)).unwrap();
            assert_eq!(icebox::lut_table(&v, lc).unwrap(), Some(table), "{} cell {lc}", part.name);
        }
    }
}

/// The recorded board experiments (iCESugar-nano) are the bitstreams the
/// description makes now, and the board did what the description says
/// wherever it says anything.
#[test]
fn board_record_agrees() {
    use epitome_sv::arch::ice40::board;
    let part = epitome_sv::arch::ice40::part("1k").unwrap();
    let (d, db, sem) = records(part);
    let exps = board::experiments(&d, &db, &sem).unwrap();
    let res = board::results_from_text(&read(part, "record-board.txt")).unwrap();
    assert_eq!(res.len(), exps.len());
    for (e, (name, expect, seen, len, hash)) in exps.iter().zip(&res) {
        assert_eq!(&e.name, name);
        assert_eq!(e.expect, expect, "{name}");
        assert_eq!((e.bits.len(), fnv64(&e.bits)), (*len, *hash), "{name}: the design's bitstream has changed; rerun the ice40_board example");
        match e.expect {
            "echo" => assert_eq!(seen, "echo", "{name}"),
            "no-echo" => assert_ne!(seen, "echo", "{name}"),
            _ => {}
        }
    }
}

/// Reruns the board experiments; only with `ICE40_BOARD=1`, since it
/// reprograms the board.
#[test]
fn board_live() {
    use epitome_sv::arch::ice40::board::{self, Icesprog, Serial};
    if std::env::var_os("ICE40_BOARD").is_none() {
        eprintln!("skipped: set ICE40_BOARD=1 to reprogram an iCESugar-nano");
        return;
    }
    let part = epitome_sv::arch::ice40::part("1k").unwrap();
    let (d, db, sem) = records(part);
    let ip = Icesprog::find().expect("icesprog");
    assert_eq!(ip.probe().unwrap().0, "iCESugar-Nano");
    let exps = board::experiments(&d, &db, &sem).unwrap();
    let res = board::run(&ip, &Serial::find().expect("serial port"), &exps, &mut |_| ()).unwrap();
    let recorded = board::results_from_text(&read(part, "record-board.txt")).unwrap();
    for (now, then) in res.iter().zip(&recorded) {
        assert_eq!(now.2, then.2, "{}", now.0);
    }
}

/// Every point of the block RAM's read modes mapped on the board, on a pin
/// that carries data in the mode, is the bit `board::ram_mode_bit` gives.
#[test]
fn ram_modes_follow_the_formula() {
    use epitome_sv::arch::ice40::board;
    let part = epitome_sv::arch::ice40::part("1k").unwrap();
    let points = board::ram_points_from_text(&read(part, "record-ram-modes.txt"));
    let data: Vec<_> = points.iter().filter_map(|&(m, a, j, p)| board::ram_mode_bit(m, a, j).map(|q| (m, a, j, p, q))).collect();
    assert!(data.len() >= 48);
    for (m, a, j, p, q) in data {
        assert_eq!(p, Some(q), "mode {m} address {a} pin {j}");
    }
}
