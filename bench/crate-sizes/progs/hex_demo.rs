fn main() {
    // Inputs the compilers can't fold: n is 1 at run time.
    let n = std::env::args().len();
    let data: Vec<u8> = (0..40 * n as u32).map(|i| (i * 37 + 11) as u8).collect();
    let e = hex::encode(&data);
    println!("{}", e);
    println!("{}", hex::encode_upper(&b"Hello, hex!"[n - 1..]));
    let d = hex::decode(&e).unwrap();
    println!("{}", d == data);
    println!("{:?}", hex::decode(&"zz"[n - 1..]));
    println!("{:?}", hex::decode(&"abc"[n - 1..]));
    let mut buf = [0u8; 4];
    println!("{:?} {:?}", hex::decode_to_slice(&"deadbeef"[n - 1..], &mut buf), buf);
    let mut out = [0u8; 8];
    hex::encode_to_slice(&[1u8, 0x23, 0xab, 0xff][n - 1..], &mut out).unwrap();
    println!("{}", std::str::from_utf8(&out).unwrap());
}
