fn main() {
    // Inputs the compilers can't fold: n is 1 at run time.
    let n = std::env::args().len();
    let mut buf = ryu::Buffer::new();
    let xs = [0.0f64, 1.0, -1.5, 0.1, 1.0 / 3.0, 6.02214076e23, 1e-300, f64::MAX, f64::MIN_POSITIVE, 123456789.0, 5e-324];
    for &x in xs[n - 1..].iter() {
        println!("{}", buf.format(x));
    }
    let ys = [0.0f32, 0.1, 3.14159, -2.5e-20, f32::MAX, 16777216.0];
    for &y in ys[n - 1..].iter() {
        println!("{}", buf.format(y));
    }
    let mut x = n as f64;
    for _ in 0..10 {
        x = x * 1.7 + 0.3;
        println!("{}", buf.format_finite(x));
    }
}
