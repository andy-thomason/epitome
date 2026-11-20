use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};
fn main() {
    // Inputs the compilers can't fold: n is 1 at run time.
    let n = std::env::args().len();
    let ss = ["hello", "héllo", "日本語", "🦀 rust", "a\u{300}b", "\u{200b}zero", "ｗｉｄｅ", "tab\there"];
    for s in ss[n - 1..].iter() {
        println!("{:?} {} {}", s, s.width(), s.width_cjk());
    }
    for c in ['a', '\u{0}', '\u{1F600}', '\u{3000}', '\u{AD}', '\u{1100}'][n - 1..].iter() {
        println!("{:?} {:?} {:?}", c, c.width(), c.width_cjk());
    }
}
