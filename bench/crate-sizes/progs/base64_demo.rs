use base64::Engine;
use base64::engine::general_purpose::{STANDARD, URL_SAFE_NO_PAD};
fn main() {
    // Inputs the compilers can't fold: n is 1 at run time.
    let n = std::env::args().len();
    let inputs: [&[u8]; 5] = [b"", b"f", b"fo", b"foobar", b"Many hands make light work."];
    for i in inputs[n - 1..].iter() {
        let e = STANDARD.encode(i);
        let u = URL_SAFE_NO_PAD.encode(i);
        let d = STANDARD.decode(&e).unwrap();
        println!("{} {} {}", e, u, d == *i);
    }
    let data: Vec<u8> = (0..200 * n as u32).map(|i| (i * 131 + 7) as u8).collect();
    let e = STANDARD.encode(&data);
    println!("{}", e);
    println!("{}", STANDARD.decode(&e).unwrap() == data);
    println!("{:?}", STANDARD.decode(&"abc$"[n - 1..]));
    println!("{:?}", STANDARD.decode(&"abcde"[n - 1..]));
}
