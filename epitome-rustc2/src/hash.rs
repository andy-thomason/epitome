//! A fast hasher for the compiler's own maps, in place of the standard
//! library's default (SipHash, which resists deliberate collisions and is
//! slow; our keys are integers and names from the source, not attacks).
//!
//! Each word is mixed in with a rotate, an xor and a multiply by an odd
//! constant; `finish` folds the high bits down so that tables that use the
//! low bits see all of them. Prefer `Vec`s indexed by ids to maps; use these
//! where a map is needed.

use std::hash::{BuildHasherDefault, Hasher};

/// An odd constant with well-spread bits (from the golden ratio).
const K: u64 = 0x9e37_79b9_7f4a_7c15;

#[derive(Default, Clone, Copy)]
pub struct FastHasher {
    h: u64,
}

impl FastHasher {
    #[inline]
    fn mix(&mut self, w: u64) {
        self.h = (self.h.rotate_left(26) ^ w).wrapping_mul(K);
    }
}

impl Hasher for FastHasher {
    #[inline]
    fn write(&mut self, bytes: &[u8]) {
        let (words, rest) = bytes.as_chunks::<8>();
        for w in words {
            self.mix(u64::from_le_bytes(*w));
        }
        if !rest.is_empty() {
            let mut buf = [0u8; 8];
            buf[..rest.len()].copy_from_slice(rest);
            self.mix(u64::from_le_bytes(buf) ^ (rest.len() as u64) << 59);
        }
    }

    #[inline]
    fn write_u8(&mut self, n: u8) {
        self.mix(n as u64);
    }

    #[inline]
    fn write_u16(&mut self, n: u16) {
        self.mix(n as u64);
    }

    #[inline]
    fn write_u32(&mut self, n: u32) {
        self.mix(n as u64);
    }

    #[inline]
    fn write_u64(&mut self, n: u64) {
        self.mix(n);
    }

    #[inline]
    fn write_usize(&mut self, n: usize) {
        self.mix(n as u64);
    }

    #[inline]
    fn finish(&self) -> u64 {
        self.h ^ (self.h >> 29)
    }
}

pub type FastBuild = BuildHasherDefault<FastHasher>;
pub type FastMap<K, V> = std::collections::HashMap<K, V, FastBuild>;
pub type FastSet<K> = std::collections::HashSet<K, FastBuild>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spreads_small_integers() {
        // Consecutive ids land in different buckets of a small table.
        let mut buckets = [0u32; 64];
        for i in 0u32..4096 {
            let mut h = FastHasher::default();
            h.write_u32(i);
            buckets[(h.finish() & 63) as usize] += 1;
        }
        assert!(buckets.iter().all(|&n| (32..=96).contains(&n)), "{buckets:?}");
        let mut m: FastMap<&str, u32> = FastMap::default();
        m.insert("a", 1);
        m.insert("ab", 2);
        assert_eq!(m["ab"], 2);
    }
}
