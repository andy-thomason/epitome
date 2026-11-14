//! The iCE40UP5K description (`sv/arch/ice40-up5k`): it agrees with its
//! records, its sources agree with each other, and it predicts what the
//! tools do. The tests that need `icepack`, the chip database or the
//! `icebox` tools say so and pass without them; `ICE40_REQUIRE_TOOLS=1`
//! makes their absence a failure.

use epitome_sv::arch::ice40::bitstream::{self, PackOptions};
use epitome_sv::arch::ice40::chipdb::Chipdb;
use epitome_sv::arch::ice40::experiment::random_config;
use epitome_sv::arch::ice40::icebox::{self, Graph, Icebox, Semantics, TILE};
use epitome_sv::arch::ice40::icepack::Icepack;
use epitome_sv::arch::ice40::{Device, asc::Asc, describe, experiment, fnv64};
use epitome_sv::hash::FxSet;
use epitome_sv::stimulus::Rng;
use std::path::PathBuf;

fn dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("arch/ice40-up5k")
}

fn read(name: &str) -> String {
    std::fs::read_to_string(dir().join(name)).unwrap()
}

fn records() -> (Device, Chipdb, Semantics) {
    (
        Device::from_record(&read("record-icepack.txt")).unwrap(),
        Chipdb::from_text(&read("record-chipdb.txt")).unwrap(),
        Semantics::from_text(&read("record-icebox.txt")).unwrap(),
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

fn chipdb_path() -> Option<PathBuf> {
    std::env::var_os("CHIPDB").map(PathBuf::from).or_else(|| {
        let home = PathBuf::from(std::env::var_os("HOME")?);
        let base = home.join(".cache/epitome");
        std::fs::read_dir(&base)
            .ok()?
            .filter_map(|e| e.ok())
            .map(|e| e.path().join("oss-cad-suite/share/icebox/chipdb-5k.txt"))
            .find(|p| p.is_file())
    })
}

#[test]
fn records_read_back_unchanged() {
    let (d, db, sem) = records();
    assert_eq!(d.to_record(), read("record-icepack.txt"));
    assert_eq!(db.to_text(), read("record-chipdb.txt"));
    assert_eq!(sem.to_text(), read("record-icebox.txt"));
}

#[test]
fn description_is_up_to_date() {
    let (d, db, sem) = records();
    assert!(describe::to_json(&d, Some(&db), Some(&sem)) == read("description.json"), "rerun the ice40_arch example with --replay");
    assert!(describe::logic_cell_sv(&sem) == read("logic_cell.sv"), "rerun the ice40_arch example with --replay");
}

#[test]
fn sources_agree() {
    let (d, db, sem) = records();
    let failed: Vec<_> = describe::checks(&d, Some(&db), Some(&sem)).into_iter().filter(|c| !c.1).collect();
    assert!(failed.is_empty(), "{failed:?}");
}

/// The packer makes the bitstreams icepack made for the recorded random
/// configurations (by length and hash), and unpacks them.
#[test]
fn packer_matches_recorded_icepack() {
    let (d, _, _) = records();
    assert!(d.fingerprints.len() >= 16);
    for &(seed, len, hash) in &d.fingerprints {
        let (asc, opts) = random_config(&d, seed);
        let b = d.pack(&asc, &opts).unwrap();
        assert_eq!((b.len(), fnv64(&b)), (len, hash), "seed {seed}");
        let back = d.unpack(&b).unwrap();
        assert_eq!(d.pack(&back, &opts).unwrap(), b, "seed {seed}: unpacking and packing again");
    }
}

/// The routing expands to nets each naming a tile's net once, and every
/// switch joins names its tile has.
#[test]
fn routing_is_consistent() {
    let (_, db, _) = records();
    let (nets, sws) = db.routing.expand();
    let mut names = FxSet::default();
    for net in &nets {
        for m in net {
            assert!(names.insert(m.clone()), "{m:?} in two nets");
        }
    }
    for (x, y, sw) in &sws {
        assert!(names.contains(&(*x, *y, sw.dst.clone())), "switch to {} in {x} {y}", sw.dst);
        for (v, src) in &sw.opts {
            assert_eq!(v.len(), sw.bits.len());
            assert!(names.contains(&(*x, *y, src.clone())), "switch from {src} in {x} {y}");
        }
    }
    assert_eq!(nets.len(), 103_383);
    assert_eq!(sws.len(), 201_460);
}

/// The generated logic cell model elaborates with epitome-sv.
#[test]
fn logic_cell_model_elaborates() {
    let mut src = epitome_sv::model::Sources::default();
    src.text("logic_cell.sv", read("logic_cell.sv"));
    epitome_sv::model::elaborate(&src, &["ice40up5k_logic_cell"], |_| ()).unwrap();
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
    let (d, _, _) = records();
    let mut fresh = experiment::run(&ip, 0, &mut |_| ()).unwrap();
    fresh.fingerprints = d.fingerprints.clone();
    assert!(fresh == d, "the experiments now find something else: rerun the ice40_arch example");
    for seed in 1000..1024 {
        let (asc, opts) = random_config(&d, seed);
        let theirs = ip.pack(&asc.to_text(), &opts.args()).unwrap();
        assert!(d.pack(&asc, &opts).unwrap() == theirs, "seed {seed}");
        let unpacked = Asc::parse(&ip.unpack(&theirs).unwrap()).unwrap();
        let ours = d.unpack(&theirs).unwrap();
        assert_eq!(d.banks(&unpacked).unwrap().cram, d.banks(&ours).unwrap().cram, "seed {seed}");
        assert_eq!(bitstream::parse(&theirs).unwrap().blocks.len(), bitstream::parse(&d.pack(&ours, &opts).unwrap()).unwrap().blocks.len());
    }
    // A configuration with nothing but a comment and one tile, and every option.
    let mut asc = Asc::new("5k");
    asc.comment = Some(vec!["epitome".into()]);
    asc.tiles = d.unpack(&d.pack(&random_config(&d, 7).0, &PackOptions::default()).unwrap()).unwrap().tiles[..1].to_vec();
    for opts in [PackOptions { nosleep: true, freq: 2, no_bram: true }, PackOptions::default()] {
        assert!(d.pack(&asc, &opts).unwrap() == ip.pack(&asc.to_text(), &opts.args()).unwrap());
    }
}

/// The chip database read afresh is what the record holds, and the
/// compact routing gives back every net and switch it lists.
#[test]
fn fresh_chipdb() {
    let Some(path) = chipdb_path() else {
        skip("chipdb-5k.txt ($CHIPDB)");
        return;
    };
    let (db, raw) = Chipdb::parse(&std::fs::read_to_string(path).unwrap()).unwrap();
    assert!(db == records().1, "the chip database differs from the record");
    let (nets, sws) = db.routing.expand();
    let mut want: Vec<_> = raw.nets.iter().map(|n| {
        let mut n = n.clone();
        n.sort();
        n
    }).collect();
    want.sort();
    assert!(nets == want);
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
    assert!(got == want_sws);
}

/// Random LUT contents written by the description's LUT bits are what
/// icebox_vlog reads back, in every cell.
#[test]
fn luts_predict_icebox() {
    let (Some(ib), Some(path)) = (Icebox::find(), chipdb_path()) else {
        skip("icebox or chipdb-5k.txt");
        return;
    };
    let (d, _, sem) = records();
    let (db, raw) = Chipdb::parse(&std::fs::read_to_string(path).unwrap()).unwrap();
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
        assert_eq!(icebox::lut_table(&v, lc).unwrap(), Some(table), "cell {lc}");
    }
}
