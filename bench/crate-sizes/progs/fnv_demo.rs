use std::hash::Hasher;
fn main() {
    // Inputs the compilers can't fold: n is 1 at run time.
    let n = std::env::args().len();
    let mut m: fnv::FnvHashMap<String, usize> = fnv::FnvHashMap::default();
    let words = "the quick brown fox jumps over the lazy dog the end";
    for w in words[n - 1..].split(' ') {
        // (Not `entry`: epitome-rustc has no layout for hashbrown's Entry yet.)
        match m.get_mut(w) {
            Some(c) => *c += 1,
            None => {
                m.insert(w.to_string(), 1);
            }
        }
    }
    let mut v: Vec<(&String, &usize)> = m.iter().collect();
    v.sort();
    for (k, c) in v {
        println!("{} {}", k, c);
    }
    let mut s: fnv::FnvHashSet<u64> = fnv::FnvHashSet::default();
    for i in 0..1000 * n as u64 { s.insert(i * i % 997); }
    println!("set {}", s.len());
    let mut h = fnv::FnvHasher::default();
    h.write(&b"hello world"[n - 1..]);
    println!("{:x}", h.finish());
    let mut h = fnv::FnvHasher::with_key(42);
    h.write_u32(7 * n as u32);
    println!("{:x}", h.finish());
}
