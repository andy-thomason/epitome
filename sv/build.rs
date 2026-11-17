//! `epitome-vvp` exports its VPI functions, so that the libraries it loads
//! can call them.

fn main() {
    let os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    if os == "linux" || os.ends_with("bsd") {
        println!("cargo:rustc-link-arg-bin=epitome-vvp=-rdynamic");
    }
}
