// Demangles Rust symbols, one per line on stdin, with rustc-demangle
// (from the Cargo registry; build.sh builds it). Prints the name without
// hashes (`{:#}`), or the line unchanged if it isn't a Rust symbol.
use std::io::{BufRead, Write};
fn main() {
    let out = std::io::stdout();
    let mut out = out.lock();
    for line in std::io::stdin().lock().lines() {
        let line = line.unwrap();
        let s = line.strip_prefix('_').filter(|s| s.starts_with("_R") || s.starts_with("_ZN")).unwrap_or(&line);
        match rustc_demangle::try_demangle(s) {
            Ok(d) => writeln!(out, "{:#}", d).unwrap(),
            Err(_) => writeln!(out, "{}", line).unwrap(),
        }
    }
}
