//! Random stimulus for a top's inputs, in the order [`Io`] gives them: for
//! each cycle, for each instance, each input's 32-bit words, low first.
//! Random bits, which a caller's `pick` may then change for each port.

use crate::model::{Io, words};

/// A xorshift generator: quick, and the same sequence on every machine.
#[derive(Clone, Debug)]
pub struct Rng(pub u64);

impl Rng {
    pub fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
}

/// Stimulus for `io`'s inputs: random bits that `pick` may then change
/// (given a port's name, the cycle, the instance and its words); the bits
/// above each port's width cleared.
pub fn stimulus_at(io: &Io, cycles: usize, n: usize, rng: &mut Rng, pick: &mut dyn FnMut(&str, usize, usize, &mut Rng, &mut [u32])) -> Vec<u32> {
    let mut out = Vec::with_capacity(cycles * n * io.in_words());
    for c in 0..cycles {
        for i in 0..n {
            for p in &io.ins {
                let w = p.width;
                let mut words: Vec<u32> = (0..words(w)).map(|_| rng.next() as u32).collect();
                pick(&p.name, c, i, rng, &mut words);
                if w % 32 != 0 {
                    *words.last_mut().unwrap() &= (1u32 << (w % 32)) - 1;
                }
                out.extend(words);
            }
        }
    }
    out
}

/// [`stimulus_at`], whatever the instance.
pub fn stimulus_with(io: &Io, cycles: usize, n: usize, rng: &mut Rng, pick: &mut dyn FnMut(&str, usize, &mut Rng, &mut [u32])) -> Vec<u32> {
    stimulus_at(io, cycles, n, rng, &mut |name, c, _, rng, words| pick(name, c, rng, words))
}

/// [`stimulus_with`], where `pick` gives the low 64 bits of a port.
pub fn stimulus(io: &Io, cycles: usize, n: usize, rng: &mut Rng, pick: &mut dyn FnMut(&str, usize, &mut Rng) -> Option<u64>) -> Vec<u32> {
    stimulus_with(io, cycles, n, rng, &mut |name, c, rng, words| {
        if let Some(x) = pick(name, c, rng) {
            words[0] = x as u32;
            if words.len() > 1 {
                words[1] = (x >> 32) as u32;
            }
        }
    })
}

/// Writes `x` into bits `lo..lo + w` (`w` at most 64) of `words`.
pub fn set_bits(words: &mut [u32], lo: usize, w: usize, x: u64) {
    for i in 0..w {
        let (k, b) = ((lo + i) / 32, (lo + i) % 32);
        words[k] = words[k] & !(1 << b) | (((x >> i) & 1) as u32) << b;
    }
}

/// Stimulus made an instance at a time, for inputs that depend on each
/// other: `f` is given the cycle, the instance and a [`Frame`] of its
/// inputs, all zero to begin with.
pub fn frames(io: &Io, cycles: usize, n: usize, f: &mut dyn FnMut(usize, usize, &mut Frame)) -> Vec<u32> {
    let iw = io.in_words();
    let mut out = vec![0; cycles * n * iw];
    for c in 0..cycles {
        for i in 0..n {
            let at = (c * n + i) * iw;
            f(c, i, &mut Frame { io, words: &mut out[at..at + iw] });
        }
    }
    out
}

/// One instance's inputs in a cycle, by port name.
pub struct Frame<'a> {
    pub io: &'a Io,
    pub words: &'a mut [u32],
}

impl Frame<'_> {
    /// Sets input `port` to `v` (its low 64 bits; the rest zero).
    pub fn set(&mut self, port: &str, v: u64) {
        self.set_wide(port, &[v]);
    }

    /// Sets input `port` to `v`, 64-bit words low first.
    pub fn set_wide(&mut self, port: &str, v: &[u64]) {
        let (at, w) = self.io.input(port).unwrap_or_else(|| panic!("no input {port}"));
        for k in 0..words(w) {
            let mut x = (v.get(k / 2).copied().unwrap_or(0) >> (32 * (k % 2))) as u32;
            if k == words(w) - 1 && w % 32 != 0 {
                x &= (1u32 << (w % 32)) - 1;
            }
            self.words[at + k] = x;
        }
    }

    /// Input `port`'s words.
    pub fn get(&self, port: &str) -> &[u32] {
        input(self.io, self.words, port)
    }
}

/// Input `port`'s words in one instance's inputs, `words`.
pub fn input<'w>(io: &Io, words: &'w [u32], port: &str) -> &'w [u32] {
    let (at, w) = io.input(port).unwrap_or_else(|| panic!("no input {port}"));
    &words[at..at + crate::model::words(w)]
}

/// The low 64 bits of a value given as 32-bit words.
pub fn low64(words: &[u32]) -> u64 {
    words.iter().take(2).enumerate().map(|(k, &x)| (x as u64) << (32 * k)).sum()
}
