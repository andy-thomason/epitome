//! Builds `tests/proc_macro/hello.rs` with rustc and expands
//! `tests/proc_macro/use_hello.rs` with it, through the toolchain's
//! proc-macro server. Skipped where rustc or the server is missing.

use epitome_rustc2::cfg::CfgSet;
use epitome_rustc2::lexer::Edition;
use epitome_rustc2::pretty::pretty;
use epitome_rustc2::proc_macro::Server;
use epitome_rustc2::session::{CrateRoot, Options, Session};
use std::path::Path;
use std::process::Command;

#[test]
fn derive_function_and_attribute_macros() {
    let Some(server) = Server::from_rustc() else { return };
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/proc_macro");
    let out = std::env::temp_dir().join(format!("epitome-rustc2-pm-{}", std::process::id()));
    std::fs::create_dir_all(&out).unwrap();
    let lib = out.join(if cfg!(target_os = "macos") { "libhello.dylib" } else { "libhello.so" });
    let status = Command::new("rustc")
        .args(["--edition", "2024", "--crate-type", "proc-macro", "-o"])
        .arg(&lib)
        .arg(dir.join("hello.rs"))
        .status()
        .unwrap();
    assert!(status.success());

    let session = Session::new(Options { threads: 2, cfg: Some(CfgSet::host()), proc_macro_server: Some(server) });
    let root = CrateRoot::new("use_hello", dir.join("use_hello.rs")).edition(Edition::E2021).proc_macro("hello", &lib);
    let mut crates = session.parse_crates(vec![root]);
    let stats = session.expand(&mut crates);
    let errors: Vec<String> = crates[0].errors.iter().map(|e| e.render_in(&session.sources)).collect();
    assert!(errors.is_empty(), "{}", errors.join(""));
    // Three proc macros, the attribute macro, and the built-in `Clone`.
    assert_eq!(stats.expanded, 6);
    let text = pretty(&crates[0].files[0].ast);
    for want in [
        "impl Hello for Point {",
        "impl ::core::clone::Clone for Point {",
        "impl Hello for Shape {",
        "(1 + 2) * 3",
        "pub fn __again_once() -> u32",
        r##"pub const SHOWN: &str = "a => b'x' + r#\"q\"#";"##,
    ] {
        assert!(text.contains(want), "no `{want}` in:\n{text}");
    }
    // `#[cfg(windows)]` fields are gone before the derive sees the item.
    assert!(!text.contains("windows_only"));
    let _ = std::fs::remove_dir_all(out);
}
