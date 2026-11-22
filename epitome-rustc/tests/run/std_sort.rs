//! `std`'s sorts past the insertion sort for short slices: driftsort and
//! ipnsort pick their small-sort thresholds through specialising impls
//! (`impl<T: FreezeMarker>` over `impl<T>`), and `Freeze` is an auto
//! trait a bare `T` does not have unless its bounds say so.

fn main() {
    for n in [20u64, 21, 100, 5000, 200_000] {
        let mut v: Vec<u64> = (0..n).map(|i| i.wrapping_mul(2654435761) % 1000003).collect();
        let mut w = v.clone();
        v.sort();
        w.sort_unstable();
        let r: u64 = v.iter().step_by(7).sum();
        println!("{}: {} {} {} {}", n, r, v == w, v.windows(2).all(|p| p[0] <= p[1]), v[v.len() / 2]);
    }

    let mut pairs: Vec<(u32, i16)> = (0..1000u32).map(|i| (i * 7919 % 101, (i % 13) as i16 - 6)).collect();
    pairs.sort_by_key(|&(a, b)| (b, a));
    println!("{:?} {:?}", &pairs[..3], pairs[999]);

    let mut words: Vec<String> = (0..300u32).map(|i| format!("w{}", i * 37 % 300)).collect();
    words.sort();
    words.dedup();
    println!("{} {} {}", words.len(), words[0], words[words.len() - 1]);

    let mut f: Vec<f64> = (0..500).map(|i| ((i * 131) % 97) as f64 - 48.5).collect();
    f.sort_by(|a, b| b.total_cmp(a));
    println!("{} {}", f[0], f[499]);
}
