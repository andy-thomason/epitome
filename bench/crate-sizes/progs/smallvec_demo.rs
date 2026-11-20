use smallvec::{smallvec, SmallVec};
fn main() {
    // Inputs the compilers can't fold: n is 1 at run time.
    let n = std::env::args().len();
    let mut v: SmallVec<[u32; 4]> = SmallVec::new();
    for i in 0..3 * n as u32 { v.push(i * 10); }
    println!("{:?} spilled={}", v, v.spilled());
    v.extend(100..100 + 10 * n as u32);
    println!("{:?} spilled={} cap>={}", v, v.spilled(), v.capacity() >= 13);
    v.insert(2, 7);
    let r = v.remove(0);
    v.retain(|x| *x % 3 != 0);
    v.sort_unstable_by(|a, b| b.cmp(a));
    println!("{} {:?}", r, v);
    v.truncate(3);
    v.shrink_to_fit();
    println!("{:?} spilled={}", v, v.spilled());
    let w: SmallVec<[String; 2]> = smallvec!["a".to_string(), "bc".to_string(), "def".to_string()];
    let lens: SmallVec<[usize; 8]> = w.iter().map(|s| s.len()).collect();
    println!("{:?} {:?}", w, lens);
    let mut d: SmallVec<[u8; 16]> = SmallVec::from_slice(&b"hello"[n - 1..]);
    d.dedup();
    d.extend_from_slice(b" world");
    println!("{}", String::from_utf8_lossy(&d));
    let v2 = d.into_vec();
    println!("{}", v2.len());
}
