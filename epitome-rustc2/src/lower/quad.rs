//! Decimal text to IEEE binary128 (`f128`), rounded correctly (to nearest,
//! ties to even), with exact big-integer arithmetic: an `f128` literal has
//! more digits than an `f64` holds (`core`'s `PI` has 64).
//!
//! The value is `D × 10^E` for the digits `D`. For `E ≥ 0` that is an
//! integer, rounded to the precision directly. For `E < 0`, `D × 2^k`
//! divided by `10^-E` gives a quotient of enough bits (the precision and
//! two more), and whether the remainder is zero says whether anything was
//! lost below them. Division is by shifting and subtracting, which is slow
//! for literals with huge exponents, but literals are few.

/// The bits of the binary128 nearest the decimal number `text` (digits,
/// an optional `.`, an optional exponent; `_` allowed; no sign, no
/// suffix). `None` if it is not one.
pub(crate) fn decimal_to_quad(text: &str) -> Option<u128> {
    let text: String = text.chars().filter(|&c| c != '_').collect();
    let (mant, exp) = match text.find(['e', 'E']) {
        Some(i) => (&text[..i], text[i + 1..].parse::<i64>().ok()?),
        None => (&text[..], 0),
    };
    let (int, frac) = mant.split_once('.').unwrap_or((mant, ""));
    if int.is_empty() && frac.is_empty() || !int.bytes().chain(frac.bytes()).all(|b| b.is_ascii_digit()) {
        return None;
    }
    let mut d = Big::default();
    for b in int.bytes().chain(frac.bytes()) {
        d.mul_add(10, (b - b'0') as u32);
    }
    if d.is_zero() {
        return Some(0);
    }
    let e10 = exp.checked_sub(frac.len() as i64)?;
    // Far out of range either way: infinity or zero.
    if e10 > 5000 {
        return Some(INF);
    }
    if e10 < -5100 && d.bits() < 400 {
        return Some(0);
    }
    // The value is q × 2^s, and `sticky` if anything non-zero was lost
    // below q's last bit.
    let (q, s, sticky) = if e10 >= 0 {
        for _ in 0..e10 {
            d.mul_add(10, 0);
        }
        (d, 0i64, false)
    } else {
        let mut den = Big::from(1);
        for _ in 0..-e10 {
            den.mul_add(10, 0);
        }
        let k = (PRECISION as i64 + 3 + den.bits() as i64 - d.bits() as i64).max(0);
        d.shl(k as u64);
        let (q, r) = d.div_rem(&den);
        (q, -k, !r.is_zero())
    };
    Some(round(q, s, sticky))
}

/// Bits of precision of a normal binary128, the leading one included.
const PRECISION: u32 = 113;
/// The least exponent of a normal number.
const EMIN: i64 = -16382;
const EMAX: i64 = 16383;
const INF: u128 = 0x7fff << 112;

/// `q × 2^s` (plus something below, if `sticky`), rounded to a binary128.
fn round(mut q: Big, s: i64, sticky: bool) -> u128 {
    let lead = q.bits() as i64 - 1 + s;
    if lead > EMAX {
        return INF;
    }
    // The bits kept: all the precision for a normal number; below the
    // normal range, fewer, the last always at 2^(EMIN - 112).
    let p = if lead >= EMIN { PRECISION as i64 } else { PRECISION as i64 - (EMIN - lead) };
    if p < 0 {
        return 0;
    }
    let shift = q.bits() as i64 - p;
    let (mut m, lsb) = if shift > 0 {
        let half = q.bit(shift as u64 - 1);
        let below = sticky || q.any_below(shift as u64 - 1);
        q.shr(shift as u64);
        let m = q.low_u128();
        (if half && (below || m & 1 == 1) { m + 1 } else { m }, s + shift)
    } else {
        (q.low_u128() << -shift, s + shift)
    };
    if m == 0 {
        return 0;
    }
    // Rounding up may have carried into one more bit.
    let lead = 127 - m.leading_zeros() as i64 + lsb;
    if lead > EMAX {
        return INF;
    }
    if lead < EMIN {
        // Subnormal: the last bit is at 2^(EMIN - 112).
        return m;
    }
    if m >> PRECISION != 0 {
        m >>= 1;
    }
    let biased = (lead + EMAX) as u128;
    (biased << 112) | (m & ((1u128 << 112) - 1))
}

/// A big unsigned integer, 32-bit words, least significant first.
#[derive(Clone, Default, Debug)]
struct Big(Vec<u32>);

impl Big {
    fn from(v: u32) -> Big {
        Big(vec![v])
    }

    fn is_zero(&self) -> bool {
        self.0.iter().all(|&w| w == 0)
    }

    fn trim(&mut self) {
        while self.0.last() == Some(&0) {
            self.0.pop();
        }
    }

    fn bits(&self) -> u64 {
        match self.0.iter().rposition(|&w| w != 0) {
            Some(i) => i as u64 * 32 + (32 - self.0[i].leading_zeros() as u64),
            None => 0,
        }
    }

    fn bit(&self, i: u64) -> bool {
        self.0.get((i / 32) as usize).is_some_and(|w| w >> (i % 32) & 1 == 1)
    }

    /// Whether any bit below `i` is set.
    fn any_below(&self, i: u64) -> bool {
        let (w, b) = ((i / 32) as usize, i % 32);
        self.0.iter().take(w).any(|&x| x != 0) || self.0.get(w).is_some_and(|&x| x & ((1u32 << b) - 1) != 0)
    }

    /// `self × m + a`.
    fn mul_add(&mut self, m: u32, a: u32) {
        let mut carry = a as u64;
        for w in &mut self.0 {
            let v = *w as u64 * m as u64 + carry;
            *w = v as u32;
            carry = v >> 32;
        }
        if carry != 0 {
            self.0.push(carry as u32);
        }
    }

    fn shl(&mut self, n: u64) {
        let (words, bits) = ((n / 32) as usize, n % 32);
        let mut out = vec![0u32; words];
        let mut carry = 0u32;
        for &w in &self.0 {
            if bits == 0 {
                out.push(w);
            } else {
                out.push(w << bits | carry);
                carry = w >> (32 - bits);
            }
        }
        if carry != 0 {
            out.push(carry);
        }
        self.0 = out;
    }

    fn shr(&mut self, n: u64) {
        let (words, bits) = ((n / 32) as usize, n % 32);
        if words >= self.0.len() {
            self.0.clear();
            return;
        }
        let src = &self.0[words..];
        let mut out = Vec::with_capacity(src.len());
        for i in 0..src.len() {
            let lo = src[i] >> bits;
            let hi = if bits == 0 { 0 } else { src.get(i + 1).map_or(0, |&h| h << (32 - bits)) };
            out.push(lo | hi);
        }
        self.0 = out;
        self.trim();
    }

    fn low_u128(&self) -> u128 {
        self.0.iter().take(4).enumerate().fold(0u128, |a, (i, &w)| a | (w as u128) << (32 * i))
    }

    fn cmp(&self, o: &Big) -> std::cmp::Ordering {
        let (a, b) = (self.bits(), o.bits());
        if a != b {
            return a.cmp(&b);
        }
        for i in (0..self.0.len().max(o.0.len())).rev() {
            let (x, y) = (self.0.get(i).copied().unwrap_or(0), o.0.get(i).copied().unwrap_or(0));
            if x != y {
                return x.cmp(&y);
            }
        }
        std::cmp::Ordering::Equal
    }

    /// `self -= o`, where `o ≤ self`.
    fn sub(&mut self, o: &Big) {
        let mut borrow = 0i64;
        for i in 0..self.0.len() {
            let v = self.0[i] as i64 - o.0.get(i).copied().unwrap_or(0) as i64 - borrow;
            borrow = (v < 0) as i64;
            self.0[i] = v.rem_euclid(1 << 32) as u32;
        }
        self.trim();
    }

    /// Quotient and remainder, by shifting and subtracting.
    fn div_rem(&self, den: &Big) -> (Big, Big) {
        let mut q = Big::default();
        let mut r = Big::default();
        for i in (0..self.bits()).rev() {
            r.shl(1);
            if self.bit(i) {
                if r.0.is_empty() {
                    r.0.push(1);
                } else {
                    r.0[0] |= 1;
                }
            }
            if r.cmp(den) != std::cmp::Ordering::Less {
                r.sub(den);
                let w = (i / 32) as usize;
                if q.0.len() <= w {
                    q.0.resize(w + 1, 0);
                }
                q.0[w] |= 1 << (i % 32);
            }
        }
        q.trim();
        (q, r)
    }
}

#[cfg(test)]
mod tests {
    use super::decimal_to_quad;

    #[test]
    fn quads() {
        // Known binary128 encodings.
        assert_eq!(decimal_to_quad("1"), Some(0x3fff_0000_0000_0000_0000_0000_0000_0000));
        assert_eq!(decimal_to_quad("1.0"), Some(0x3fff_0000_0000_0000_0000_0000_0000_0000));
        assert_eq!(decimal_to_quad("2.5"), Some(0x4000_4000_0000_0000_0000_0000_0000_0000));
        assert_eq!(decimal_to_quad("0.1"), Some(0x3ffb_9999_9999_9999_9999_9999_9999_999a));
        assert_eq!(decimal_to_quad("0"), Some(0));
        // core's f128 constants.
        assert_eq!(
            decimal_to_quad("3.14159265358979323846264338327950288419716939937510582097494459"),
            Some(0x4000_921f_b544_42d1_8469_898c_c517_01b8)
        );
        assert_eq!(decimal_to_quad("1.18973149535723176508575932662800702e4932"), Some(0x7ffe_ffff_ffff_ffff_ffff_ffff_ffff_ffff));
        assert_eq!(decimal_to_quad("3.36210314311209350626267781732175260e-4932"), Some(0x0001_0000_0000_0000_0000_0000_0000_0000));
        assert_eq!(decimal_to_quad("1.92592994438723585305597794258492732e-34"), Some(0x3f8f_0000_0000_0000_0000_0000_0000_0000));
        // The least subnormal, and half of it (a tie, to even: zero).
        assert_eq!(decimal_to_quad("6.47517511943802511092443895822764655e-4966"), Some(1));
        assert_eq!(decimal_to_quad("3.2e-4966"), Some(0));
        // Too big: infinity.
        assert_eq!(decimal_to_quad("1.2e4932"), Some(0x7fff_0000_0000_0000_0000_0000_0000_0000));
        // Agrees with f64 where f64 is exact.
        for (t, v) in [("1024", 1024.0f64), ("0.75", 0.75), ("123456789", 123456789.0), ("1e10", 1e10)] {
            let q = decimal_to_quad(t).unwrap();
            let e = ((q >> 112) & 0x7fff) as i64 - 16383;
            let m = (q & ((1u128 << 112) - 1)) | (1u128 << 112);
            let back = m as f64 * 2f64.powi(e as i32 - 112);
            assert_eq!(back, v, "{t}");
        }
    }
}
