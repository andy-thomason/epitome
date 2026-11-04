//! Bundles (docs/design/vliw.md, section 1), on a made-up machine
//! (`tests/bundles/toy.ll`): slots by unit and lane, registers read through
//! a port in the header, prefixes, the last bundle marked, and disassembly.

use epitome_codegen::assembler::Assembler;
use epitome_codegen::check;
use epitome_codegen::objdump::{self, Image};
use epitome_codegen::target::Target;

fn toy() -> Target {
    Target::load_path(&format!("{}/tests/bundles/toy.ll", env!("CARGO_MANIFEST_DIR"))).unwrap()
}

/// The bundles of `asm`, and the object.
fn assemble(t: &Target, asm: &str) -> Result<(Vec<u8>, Vec<u8>), String> {
    let a = Assembler::new(t).assemble(asm, None)?;
    let code = a.object.sections.iter().find(|s| s.name.as_deref() == Some(".text.toy")).ok_or("no .text.toy")?;
    Ok((code.data.clone(), a.file(t)?))
}

#[test]
fn description_passes_its_checks() {
    let report = check::check(&toy());
    let s = report.sections.iter().find(|s| s.name == "encodings").unwrap();
    assert!(s.errors.is_empty() && s.checked > 0, "{}", report.render(true));
}

#[test]
fn bundles_are_their_slots_and_header() {
    let t = toy();
    let (code, _) = assemble(
        &t,
        ".section .text.toy
         .set N, 2
         {
             add a1, a2, b3 :: add a3, a2, b3
         }
         N:L {
             mov a0, a2
         }",
    )
    .unwrap();
    assert_eq!(
        code,
        [
            // Ports: a reads a2 (bits 5-7), b reads b3 (bits 8-10). X's add
            // in slot 0, Y's in slot 1.
            0x40, 0x03, 0x00, 0x13, 0x00, 0x17, 0x00, 0x00,
            // rep, last and len 2 from the prefix (last again as the last
            // bundle); a reads a2. The mov, of either unit, in X's slot.
            0x4b, 0x00, 0x00, 0x20, 0x00, 0x00, 0x00, 0x00,
        ]
    );
}

#[test]
fn a_port_reads_one_register() {
    let t = toy();
    // b3 and bR are both read through b's port: two registers.
    let e = assemble(&t, ".section .text.toy\n{\n add a1, a2, b3\n :: add a3, a2, bR\n}").unwrap_err();
    assert!(e.contains("port[1] is both"), "{e}");
    // Three adds of unit X: its own slot and the one either unit takes.
    let e = assemble(&t, ".section .text.toy\n{\n add a1, a2, b3\n add a3, a2, b3\n add a4, a2, b3\n}").unwrap_err();
    assert!(e.contains("no free slot"), "{e}");
}

#[test]
fn bundles_disassemble_as_written() {
    let t = toy();
    let src = ".section .text.toy\nf:\n{\n add a1, aR, b3 :: add a3, aR, b3\n}\n3:L {\n mov a0, bR\n}\n";
    let (code, obj) = assemble(&t, src).unwrap();
    let text = objdump::disassemble(&t, &Image::read(&obj, &t.machine).unwrap(), false);
    assert!(text.contains("add a1, aR, b3") && text.contains(":: add a3, aR, b3"), "{text}");
    assert!(text.contains("3:L {") && text.contains("mov a0, bR"), "{text}");
    // As assembly again: the same bytes.
    let again: String = text
        .lines()
        .filter_map(|l| match l.trim().split_once(": ") {
            Some((a, rest)) if a.chars().all(|c| c.is_ascii_hexdigit()) => Some(rest.to_string()),
            _ if l.starts_with('\t') => Some(l.trim().to_string()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("\n");
    let (back, _) = assemble(&t, &format!(".section .text.toy\n{again}\n")).unwrap();
    assert_eq!(back, code, "{again}");
}
