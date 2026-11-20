fn main() {
    // Inputs the compilers can't fold: n is 1 at run time.
    let n = std::env::args().len();
    let mut hay = Vec::new();
    for i in 0..5000 * n as u32 {
        hay.push(b"abcdefghijklmnopqrstuvwxyz "[(i * 7 % 27) as usize]);
    }
    hay.extend_from_slice(b"needle in a haystack; another needle!");
    println!("{:?}", memchr::memchr(b'!', &hay));
    println!("{:?}", memchr::memrchr(b'a', &hay));
    println!("{:?}", memchr::memchr2(b';', b'!', &hay));
    println!("{:?}", memchr::memchr3(b'X', b'Y', b';', &hay));
    println!("{}", memchr::memchr_iter(b' ', &hay).count());
    let f = memchr::memmem::Finder::new(&"needle"[n - 1..]);
    let all: Vec<usize> = f.find_iter(&hay).collect();
    println!("{:?}", all);
    println!("{:?}", memchr::memmem::rfind(&hay, &b"hay"[n - 1..]));
    println!("{:?}", memchr::memmem::find(&hay, &b"zzz"[n - 1..]));
}
