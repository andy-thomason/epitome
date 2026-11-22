use semver::{Version, VersionReq};
fn main() {
    // Inputs the compilers can't fold: n is 1 at run time.
    let n = std::env::args().len();
    // (A loop, not `.iter().map(|s| Version::parse(s))`: epitome-rustc
    // doesn't deref `&&str` to `&str` in that call yet.)
    let mut vs: Vec<Version> = Vec::new();
    for s in ["1.2.3", "0.9.0", "1.10.0-alpha.1", "1.10.0", "1.10.0-beta+build.5", "2.0.0", "1.2.3+meta"][n - 1..].iter() {
        vs.push(Version::parse(s).unwrap());
    }
    vs.sort();
    for v in &vs { println!("{} major={} pre={:?}", v, v.major, v.pre.as_str()); }
    let reqs = ["^1.2", ">=1.0, <2.0", "~1.10.0", "*", "=0.9.0"];
    for r in reqs[n - 1..].iter() {
        let req = VersionReq::parse(r).unwrap();
        // (Not `vs.iter().filter(|v| req.matches(v))`: epitome-rustc can't
        // type the closure yet, "method call without a method".)
        let mut m = String::new();
        for v in &vs {
            if req.matches(v) {
                m.push(' ');
                m.push_str(&v.to_string());
            }
        }
        println!("{} ->{}", req, m);
    }
    println!("{:?}", Version::parse(&"1.2"[n - 1..]).is_err());
    match Version::parse(&"01.2.3"[n - 1..]) { Ok(v) => println!("ok {}", v), Err(e) => println!("err {}", e) }
    match VersionReq::parse(&">= >="[n - 1..]) { Ok(v) => println!("ok {}", v), Err(e) => println!("err {}", e) }
}
