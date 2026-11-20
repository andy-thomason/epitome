fn main() {
    // Inputs the compilers can't fold: n is 1 at run time.
    let n = std::env::args().len();
    let mut buf = itoa::Buffer::new();
    let mut total = 0usize;
    for i in 0..20u32 {
        let x = (7u64 * n as u64).wrapping_pow(i) ^ (i as u64);
        let s = buf.format(x);
        total += s.len();
        println!("{}", s);
    }
    println!("{}", buf.format(-1234567i32 * n as i32));
    println!("{}", buf.format(i64::MIN / n as i64));
    println!("{}", buf.format(u128::MAX / n as u128));
    println!("{}", buf.format(i128::MIN / n as i128));
    println!("{}", buf.format(255u8 / n as u8));
    println!("{}", buf.format(-32768i16 / n as i16));
    println!("total {}", total);
}
