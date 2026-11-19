//! `std` paths through specialised trait methods: `Zip::fold` goes
//! through `ZipImpl`, whose most specific impl (for `TrustedRandomAccess`
//! iterators) leaves `fold` to the impl it specialises, and `<[T]>::
//! contains` through `SliceContains`, whose impl for `BytewiseEq` types
//! (`u8`, by `memchr`) specialises the one for every `PartialEq` type.

fn main() {
    let a = [1i64, 2, 3, 4];
    let b = [5i64, 6, 7, 8];
    let d: i64 = a.iter().zip(&b).map(|(x, y)| x * y).sum();
    println!("{d} {}", b"aeiou".contains(&std::hint::black_box(b'e')));

    // Zip of iterators that are not `TrustedRandomAccess`.
    let odd: i64 = (1..10i64).filter(|x| x % 2 == 1).zip(a.iter()).map(|(x, y)| x * y).sum();
    println!("{odd}");

    // `contains` on the other impls: `i32`'s own, and the `PartialEq` one.
    let ints = [3i32, 1, 4, 1, 5, 9, 2, 6];
    let words = ["fee", "fie", "foe"];
    println!(
        "{} {} {} {} {}",
        ints.contains(&std::hint::black_box(9)),
        ints.contains(&std::hint::black_box(7)),
        words.contains(&std::hint::black_box("fie")),
        words.contains(&std::hint::black_box("fum")),
        b"aeiou".contains(&std::hint::black_box(b'z')),
    );
}
