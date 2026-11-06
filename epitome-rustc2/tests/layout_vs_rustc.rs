//! The size and alignment of every type alias in `tests/layout/layouts.rs`,
//! as our layout gives them (with `core` from rust-src) and as a program
//! rustc builds from the same file prints them. Skipped where rustc or
//! rust-src is missing.

use epitome_rustc2::cfg::CfgSet;
use epitome_rustc2::layout::{LayoutCx, TargetData};
use epitome_rustc2::resolve::DefKind;
use epitome_rustc2::session::{CrateRoot, Options, Session};
use std::path::PathBuf;
use std::process::Command;

fn sysroot() -> Option<PathBuf> {
    let out = Command::new("rustc").args(["--print", "sysroot"]).output().ok()?;
    Some(PathBuf::from(String::from_utf8(out.stdout).ok()?.trim()))
}

#[test]
fn layouts_match_rustc() {
    let Some(root) = sysroot() else {
        eprintln!("no rustc: skipped");
        return;
    };
    let core = root.join("lib/rustlib/src/rust/library/core/src/lib.rs");
    if !core.exists() {
        eprintln!("no rust-src: skipped");
        return;
    }
    let file = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/layout/layouts.rs");
    let source = std::fs::read_to_string(&file).unwrap();

    // Ours.
    let session = Session::new(Options { threads: 4, cfg: Some(CfgSet::host()), ..Default::default() });
    let mut crates = session.parse_crates(vec![CrateRoot::new("core", &core), CrateRoot::new("layouts", &file)]);
    session.expand(&mut crates);
    let r = session.resolve(&crates);
    let tcx = session.collect_types(&crates, &r);
    let index = epitome_rustc2::typeck::Index::new(&tcx, &r);
    let cx = LayoutCx::new(&tcx, &r, &crates, &index, TargetData::default());
    let mut ours = Vec::new();
    for (i, def) in r.defs.iter().enumerate() {
        if def.krate != 1 || def.kind != DefKind::TypeAlias {
            continue;
        }
        let d = epitome_rustc2::resolve::DefId(i as u32);
        let t = tcx.type_of(d).expect("alias type");
        let l = cx.layout_of(t).unwrap_or_else(|e| panic!("no layout for {}: {e:?}", def.name));
        ours.push((def.name.to_string(), l.size, l.align));
    }
    assert!(ours.len() > 30, "too few aliases: {}", ours.len());

    // rustc's: the same file as a program that prints them.
    let dir = std::env::temp_dir().join(format!("epitome-rustc2-layout-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let mut program: String = source.lines().filter(|l| !l.starts_with("#![no_std]")).collect::<Vec<_>>().join("\n");
    program.push_str("\nfn main() {\n");
    for (name, _, _) in &ours {
        program.push_str(&format!(
            "    println!(\"{name} {{}} {{}}\", core::mem::size_of::<{name}>(), core::mem::align_of::<{name}>());\n"
        ));
    }
    program.push_str("}\n");
    let src = dir.join("main.rs");
    std::fs::write(&src, program).unwrap();
    let exe = dir.join("main");
    let built = Command::new("rustc")
        .args(["--edition", "2024", "-A", "warnings", "-o"])
        .arg(&exe)
        .arg(&src)
        .status()
        .expect("run rustc");
    assert!(built.success(), "rustc failed");
    let out = Command::new(&exe).output().expect("run the program");
    let _ = std::fs::remove_dir_all(&dir);
    let theirs: Vec<(String, u64, u64)> = String::from_utf8(out.stdout)
        .unwrap()
        .lines()
        .map(|l| {
            let v: Vec<&str> = l.split(' ').collect();
            (v[0].to_string(), v[1].parse().unwrap(), v[2].parse().unwrap())
        })
        .collect();

    let wrong: Vec<String> = ours
        .iter()
        .zip(&theirs)
        .filter(|(a, b)| a != b)
        .map(|(a, b)| format!("{}: ours size {} align {}, rustc's size {} align {}", a.0, a.1, a.2, b.1, b.2))
        .collect();
    assert!(wrong.is_empty(), "layouts differ:\n{}", wrong.join("\n"));
}
