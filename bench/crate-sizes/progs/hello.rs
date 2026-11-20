// The baseline sizes.py subtracts: what any program printing with std has.
fn main() {
    let n = std::env::args().len();
    println!("hello {}", n);
}
