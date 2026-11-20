use arrayvec::{ArrayString, ArrayVec};
fn main() {
    // Inputs the compilers can't fold: n is 1 at run time.
    let n = std::env::args().len();
    let mut v: ArrayVec<i32, 8> = ArrayVec::new();
    for i in 0..6 * n as i32 { v.push(i * i - 5); }
    println!("{:?} len={} full={}", v, v.len(), v.is_full());
    v.insert(1, 99);
    println!("{:?}", v.try_push(1).is_ok());
    println!("{:?}", v.try_push(2).is_ok());
    let p = v.pop();
    v.swap_remove(0);
    v.retain(|x| *x > 0);
    println!("{:?} {:?}", p, v);
    let d: ArrayVec<i32, 8> = v.drain(1..).collect();
    println!("{:?} {:?}", v, d);
    let mut s: ArrayString<32> = ArrayString::new();
    s.push_str(&"hello"[n - 1..]);
    s.push(' ');
    s.push_str("arrayvec");
    println!("{} {} {}", s, s.len(), s.capacity());
    println!("{:?}", s.try_push_str("0123456789012345678901234567890").is_err());
    let s2 = ArrayString::<8>::from(&"short"[n - 1..]).unwrap();
    println!("{}", s2);
    let arr: [u8; 4] = ArrayVec::from([1u8, 2, 3, n as u8 * 4]).into_inner().unwrap();
    println!("{:?}", arr);
}
