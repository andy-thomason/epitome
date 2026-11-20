fn main() {
    // Inputs the compilers can't fold: n is 1 at run time.
    let n = std::env::args().len();
    let pairs = [("kitten", "sitting"), ("saturday", "sunday"), ("", "abc"), ("martha", "marhta"), ("ab", "ba"), ("français", "francais")];
    for (a, b) in pairs[n - 1..].iter() {
        println!("{} {} lev={} osa={} dl={} nlev={:.4} jaro={:.4} jw={:.4} dice={:.4} ham={:?}", a, b,
            strsim::levenshtein(a, b), strsim::osa_distance(a, b), strsim::damerau_levenshtein(a, b),
            strsim::normalized_levenshtein(a, b), strsim::jaro(a, b), strsim::jaro_winkler(a, b),
            strsim::sorensen_dice(a, b), strsim::hamming(a, b));
    }
}
