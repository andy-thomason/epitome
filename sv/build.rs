//! `epitome-vvp` and `epitome-sv` export their VPI and DPI functions, so
//! that the libraries they load can call them.

fn main() {
    let os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    if os == "linux" || os.ends_with("bsd") {
        println!("cargo:rustc-link-arg-bin=epitome-vvp=-rdynamic");
        println!("cargo:rustc-link-arg-bin=epitome-sv=-rdynamic");
    }
}
