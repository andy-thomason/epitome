//! Four-state bit vectors of any width (6.3.1) and the operators on them
//! (clause 11), for constant evaluation and the interpreter.
//!
//! A bit is two bits, one from `val` and one from `unk`: `0` is 00, `1` is
//! 10, `z` is 01 and `x` is 11 (value bit first). Bit 0 is the least
//! significant. Bits above the width are always zero in both words.

use std::fmt;

/// One four-state bit.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Logic {
    Zero,
    One,
    X,
    Z,
}

impl Logic {
    fn from_pair(v: bool, u: bool) -> Logic {
        match (v, u) {
            (false, false) => Logic::Zero,
            (true, false) => Logic::One,
            (false, true) => Logic::Z,
            (true, true) => Logic::X,
        }
    }

    fn pair(self) -> (bool, bool) {
        match self {
            Logic::Zero => (false, false),
            Logic::One => (true, false),
            Logic::Z => (false, true),
            Logic::X => (true, true),
        }
    }

    pub fn is_known(self) -> bool {
        matches!(self, Logic::Zero | Logic::One)
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Bits {
    width: u32,
    signed: bool,
    val: Words,
    unk: Words,
}

/// A vector's words: one held inline, so values up to 64 bits wide need
/// no allocation; more on the heap. Never empty.
#[derive(Clone, PartialEq, Eq, Hash)]
enum Words {
    One(u64),
    Many(Vec<u64>),
}

impl Words {
    fn filled(x: u64, n: usize) -> Words {
        if n <= 1 { Words::One(x) } else { Words::Many(vec![x; n]) }
    }

    /// Grown with zeros, or cut, to `n` words.
    fn resize(&mut self, n: usize) {
        if n == self.len() {
            return;
        }
        let mut v = self.to_vec();
        v.resize(n.max(1), 0);
        *self = Words::from(v);
    }
}

impl From<Vec<u64>> for Words {
    fn from(v: Vec<u64>) -> Words {
        if v.len() <= 1 { Words::One(v.first().copied().unwrap_or(0)) } else { Words::Many(v) }
    }
}

impl std::ops::Deref for Words {
    type Target = [u64];
    fn deref(&self) -> &[u64] {
        match self {
            Words::One(x) => std::slice::from_ref(x),
            Words::Many(v) => v,
        }
    }
}

impl std::ops::DerefMut for Words {
    fn deref_mut(&mut self) -> &mut [u64] {
        match self {
            Words::One(x) => std::slice::from_mut(x),
            Words::Many(v) => v,
        }
    }
}

/// `width` (at most 64) bits of `w` from bit `lo`; bits past the end are 0.
fn get_bits(w: &[u64], lo: u64, width: u32) -> u64 {
    let (i, off) = ((lo / 64) as usize, (lo % 64) as u32);
    let mut x = w.get(i).copied().unwrap_or(0) >> off;
    if off != 0 {
        x |= w.get(i + 1).copied().unwrap_or(0) << (64 - off);
    }
    if width < 64 { x & ((1u64 << width) - 1) } else { x }
}

/// Puts the low `width` (at most 64) bits of `x` into `w` at bit `lo`;
/// bits past the end are dropped.
fn put_bits(w: &mut [u64], lo: u64, width: u32, x: u64) {
    if width == 0 {
        return;
    }
    let m = if width < 64 { (1u64 << width) - 1 } else { u64::MAX };
    let x = x & m;
    let (i, off) = ((lo / 64) as usize, (lo % 64) as u32);
    if let Some(a) = w.get_mut(i) {
        *a = *a & !(m << off) | x << off;
    }
    if off != 0 && off + width > 64
        && let Some(a) = w.get_mut(i + 1) {
            *a = *a & !(m >> (64 - off)) | x >> (64 - off);
        }
}

fn words(width: u32) -> usize {
    (width as usize).div_ceil(64).max(1)
}

impl Bits {
    pub fn zero(width: u32, signed: bool) -> Bits {
        let n = words(width);
        Bits { width, signed, val: Words::filled(0, n), unk: Words::filled(0, n) }
    }

    pub fn from_u64(width: u32, signed: bool, v: u64) -> Bits {
        let mut b = Bits::zero(width, signed);
        b.val[0] = v;
        b.mask();
        b
    }

    /// `v` sign-extended to `width` bits.
    pub fn from_i64(width: u32, signed: bool, v: i64) -> Bits {
        let mut b = Bits::zero(width, signed);
        for w in b.val.iter_mut() {
            *w = if v < 0 { u64::MAX } else { 0 };
        }
        b.val[0] = v as u64;
        b.mask();
        b
    }

    pub fn from_bool(v: bool) -> Bits {
        Bits::from_u64(1, false, v as u64)
    }

    /// Every bit `l`.
    pub fn fill(width: u32, signed: bool, l: Logic) -> Bits {
        let (v, u) = l.pair();
        let n = words(width);
        let mut b = Bits {
            width,
            signed,
            val: Words::filled(if v { u64::MAX } else { 0 }, n),
            unk: Words::filled(if u { u64::MAX } else { 0 }, n),
        };
        b.mask();
        b
    }

    pub fn x(width: u32) -> Bits {
        Bits::fill(width, false, Logic::X)
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn signed(&self) -> bool {
        self.signed
    }

    pub fn with_signed(mut self, signed: bool) -> Bits {
        self.signed = signed;
        self
    }

    #[inline]
    fn mask(&mut self) {
        let n = words(self.width);
        if self.val.len() != n {
            self.val.resize(n);
            self.unk.resize(n);
        }
        let rem = self.width % 64;
        if rem != 0 {
            let m = (1u64 << rem) - 1;
            self.val[n - 1] &= m;
            self.unk[n - 1] &= m;
        }
        if self.width == 0 {
            self.val[0] = 0;
            self.unk[0] = 0;
        }
    }

    pub fn bit(&self, i: u32) -> Logic {
        if i >= self.width {
            return Logic::Zero;
        }
        let (w, b) = ((i / 64) as usize, i % 64);
        Logic::from_pair(self.val[w] >> b & 1 != 0, self.unk[w] >> b & 1 != 0)
    }

    pub fn set_bit(&mut self, i: u32, l: Logic) {
        if i >= self.width {
            return;
        }
        let (w, b) = ((i / 64) as usize, i % 64);
        let (v, u) = l.pair();
        self.val[w] = self.val[w] & !(1 << b) | (v as u64) << b;
        self.unk[w] = self.unk[w] & !(1 << b) | (u as u64) << b;
    }

    /// x and z bits become 0.
    pub fn known_or_zero(&self) -> Bits {
        let mut r = self.clone();
        for k in 0..r.val.len() {
            r.val[k] &= !r.unk[k];
            r.unk[k] = 0;
        }
        r
    }

    pub fn is_known(&self) -> bool {
        self.unk.iter().all(|&w| w == 0)
    }

    /// Whether any bit is `z` (as opposed to `x`).
    pub fn has_z(&self) -> bool {
        self.unk.iter().zip(self.val.iter()).any(|(&u, &v)| u & !v != 0)
    }

    pub fn is_zero(&self) -> bool {
        self.is_known() && self.val.iter().all(|&w| w == 0)
    }

    /// The value as an unsigned number, if every bit is known and it fits.
    pub fn to_u64(&self) -> Option<u64> {
        if !self.is_known() || self.val[1..].iter().any(|&w| w != 0) {
            return None;
        }
        Some(self.val[0])
    }

    /// The value as a number, read as signed if the vector is signed.
    pub fn to_i64(&self) -> Option<i64> {
        if !self.is_known() {
            return None;
        }
        if self.width <= 64 {
            let v = self.val[0];
            return Some(if self.signed && self.width > 0 && self.width < 64 {
                let sh = 64 - self.width;
                ((v << sh) as i64) >> sh
            } else if !self.signed && self.width == 64 && (v as i64) < 0 {
                return None;
            } else {
                v as i64
            });
        }
        let neg = self.signed && self.width > 0 && self.bit(self.width - 1) == Logic::One;
        let ext = self.resize(self.width.max(64) + 1, self.signed);
        let top = ext.val[1..].iter().all(|&w| w == if neg { u64::MAX } else { 0 } || ext.width <= 64);
        // Check that the bits above 63 are all copies of bit 63.
        for i in 63..ext.width {
            if (ext.bit(i) == Logic::One) != neg {
                return None;
            }
        }
        let _ = top;
        Some(ext.val[0] as i64)
    }

    /// True if any bit is 1, false if all are 0, `None` otherwise (12.4).
    pub fn truth(&self) -> Option<bool> {
        let any_one = self.val.iter().zip(self.unk.iter()).any(|(&v, &u)| v & !u != 0);
        if any_one {
            Some(true)
        } else if self.is_known() {
            Some(false)
        } else {
            None
        }
    }

    pub fn is_negative(&self) -> bool {
        self.signed && self.width > 0 && self.bit(self.width - 1) == Logic::One
    }

    /// Extended or truncated to `width`. A wider result is filled with
    /// copies of the top bit if `sign` (including `x` and `z`), otherwise
    /// with zeros. Keeps the vector's signedness.
    pub fn resize(&self, width: u32, sign: bool) -> Bits {
        if width == self.width {
            return self.clone();
        }
        if width <= 64 && self.width <= 64 {
            // One word: shift the top bit up and back down to fill.
            let fill = |x: u64| {
                if width > self.width && sign && self.width > 0 {
                    let sh = 64 - self.width;
                    (((x << sh) as i64) >> sh) as u64
                } else {
                    x
                }
            };
            let mut b = Bits { width, signed: self.signed, val: Words::One(fill(self.val[0])), unk: Words::One(fill(self.unk[0])) };
            b.mask();
            return b;
        }
        let mut b = Bits::zero(width, self.signed);
        let n = b.val.len().min(self.val.len());
        b.val[..n].copy_from_slice(&self.val[..n]);
        b.unk[..n].copy_from_slice(&self.unk[..n]);
        b.mask();
        if width > self.width && sign && self.width > 0 {
            let top = self.bit(self.width - 1);
            if top != Logic::Zero {
                b.set_slice(self.width, &Bits::fill(width - self.width, false, top));
            }
        }
        b
    }

    /// Extended as clause 11 extends an operand: by its own signedness.
    pub fn extend(&self, width: u32) -> Bits {
        self.resize(width, self.signed)
    }

    /// An unsized literal's extension: an `x` or `z` in the top bit fills
    /// the rest (5.7.1).
    pub fn extend_literal(&self, width: u32) -> Bits {
        let top = if self.width > 0 { self.bit(self.width - 1) } else { Logic::Zero };
        if width > self.width && matches!(top, Logic::X | Logic::Z) {
            let mut b = self.resize(width, false);
            b.set_slice(self.width, &Bits::fill(width - self.width, false, top));
            b
        } else {
            self.extend(width)
        }
    }

    /// `width` bits from bit `lo`; bits past the top read as `x`.
    pub fn slice(&self, lo: u32, width: u32) -> Bits {
        let mut b = Bits::zero(width, false);
        for k in 0..b.val.len() {
            let at = lo as u64 + 64 * k as u64;
            b.val[k] = get_bits(&self.val, at, 64);
            b.unk[k] = get_bits(&self.unk, at, 64);
            // Positions at or past our width are x.
            let inside = (self.width as u64).saturating_sub(at);
            if inside < 64 {
                let out = !((1u64 << inside) - 1);
                b.val[k] |= out;
                b.unk[k] |= out;
            }
        }
        b.mask();
        b
    }

    /// Overwrites bits from `lo` with `v`; bits past the top are dropped.
    pub fn set_slice(&mut self, lo: u32, v: &Bits) {
        for k in 0..v.val.len() {
            let w = (v.width - 64 * k as u32).min(64);
            let at = lo as u64 + 64 * k as u64;
            put_bits(&mut self.val, at, w, v.val[k]);
            put_bits(&mut self.unk, at, w, v.unk[k]);
        }
        self.mask();
    }

    /// Both operands extended to the wider width, and the word-wise
    /// result of `f` on (value, unknown) pairs.
    fn words2(&self, o: &Bits, f: impl Fn(u64, u64, u64, u64) -> (u64, u64)) -> Bits {
        let w = self.width.max(o.width);
        let (a, c) = (self.extend(w), o.extend(w));
        let mut r = Bits::zero(w, self.signed && o.signed);
        for k in 0..r.val.len() {
            let (v, u) = f(a.val[k], a.unk[k], c.val[k], c.unk[k]);
            r.val[k] = v;
            r.unk[k] = u;
        }
        r.mask();
        r
    }

    /// `{a, b, ...}`: the first is the most significant.
    pub fn concat(parts: &[Bits]) -> Bits {
        let width = parts.iter().map(|p| p.width).sum();
        let mut b = Bits::zero(width, false);
        let mut lo = width;
        for p in parts {
            lo -= p.width;
            b.set_slice(lo, p);
        }
        b
    }


    // ---- bitwise (11.4.8) ----

    // A known 0 is (v, u) = (0, 0), a known 1 (1, 0); an unknown result
    // bit is x, (1, 1).

    pub fn not(&self) -> Bits {
        let mut r = self.clone();
        for k in 0..r.val.len() {
            r.val[k] = !self.val[k] | self.unk[k];
        }
        r.mask();
        r
    }

    pub fn and(&self, o: &Bits) -> Bits {
        self.words2(o, |va, ua, vb, ub| {
            let zero = (!va & !ua) | (!vb & !ub);
            let one = (va & !ua) & (vb & !ub);
            let u = !(zero | one);
            (one | u, u)
        })
    }

    pub fn or(&self, o: &Bits) -> Bits {
        self.words2(o, |va, ua, vb, ub| {
            let one = (va & !ua) | (vb & !ub);
            let zero = (!va & !ua) & (!vb & !ub);
            let u = !(zero | one);
            (one | u, u)
        })
    }

    pub fn xor(&self, o: &Bits) -> Bits {
        self.words2(o, |va, ua, vb, ub| {
            let u = ua | ub;
            ((va ^ vb) | u, u)
        })
    }

    pub fn xnor(&self, o: &Bits) -> Bits {
        self.xor(o).not().with_signed(self.signed && o.signed)
    }

    // ---- reductions (11.4.9) ----

    pub fn reduce_and(&self) -> Logic {
        let mut r = Logic::One;
        for i in 0..self.width {
            r = match (r, self.bit(i)) {
                (Logic::Zero, _) | (_, Logic::Zero) => Logic::Zero,
                (Logic::One, Logic::One) => Logic::One,
                _ => Logic::X,
            };
        }
        r
    }

    pub fn reduce_or(&self) -> Logic {
        match self.truth() {
            Some(true) => Logic::One,
            Some(false) => Logic::Zero,
            None => Logic::X,
        }
    }

    pub fn reduce_xor(&self) -> Logic {
        if !self.is_known() {
            return Logic::X;
        }
        if self.val.iter().map(|w| w.count_ones()).sum::<u32>() % 2 == 1 { Logic::One } else { Logic::Zero }
    }

    pub fn of_logic(l: Logic) -> Bits {
        let mut b = Bits::zero(1, false);
        b.set_bit(0, l);
        b
    }

    // ---- arithmetic (11.4.3): any unknown bit makes the result all x ----

    fn arith(&self, o: &Bits, w: u32, f: impl Fn(&[u64], &[u64], usize) -> Option<Vec<u64>>) -> Bits {
        let signed = self.signed && o.signed;
        if !self.is_known() || !o.is_known() {
            return Bits::x(w).with_signed(signed);
        }
        let (a, b) = (self.extend(w), o.extend(w));
        match f(&a.val, &b.val, words(w)) {
            Some(val) => {
                let mut r = Bits { width: w, signed, val: Words::from(val), unk: Words::filled(0, words(w)) };
                r.mask();
                r
            }
            None => Bits::x(w).with_signed(signed),
        }
    }

    /// Known operands that fit in one word, extended to `w`.
    fn small(&self, o: &Bits, w: u32) -> Option<(u64, u64)> {
        if w > 64 || !self.is_known() || !o.is_known() {
            return None;
        }
        Some((self.extend(w).val[0], o.extend(w).val[0]))
    }

    fn small_result(&self, o: &Bits, w: u32, v: u64) -> Bits {
        Bits::from_u64(w, self.signed && o.signed, v)
    }

    pub fn add(&self, o: &Bits) -> Bits {
        let w = self.width.max(o.width);
        if let Some((a, b)) = self.small(o, w) {
            return self.small_result(o, w, a.wrapping_add(b));
        }
        self.arith(o, w, |a, b, n| {
            let mut r = vec![0; n];
            let mut carry = false;
            for i in 0..n {
                let (s1, c1) = a[i].overflowing_add(b[i]);
                let (s2, c2) = s1.overflowing_add(carry as u64);
                r[i] = s2;
                carry = c1 || c2;
            }
            Some(r)
        })
    }

    pub fn neg(&self) -> Bits {
        Bits::zero(self.width, self.signed).sub(self)
    }

    pub fn sub(&self, o: &Bits) -> Bits {
        let w = self.width.max(o.width);
        if let Some((a, b)) = self.small(o, w) {
            return self.small_result(o, w, a.wrapping_sub(b));
        }
        let mut nb = o.extend(w).not();
        nb.signed = o.signed;
        if !o.is_known() {
            return Bits::x(w).with_signed(self.signed && o.signed);
        }
        let one = Bits::from_u64(w, o.signed, 1);
        let r = self.add(&nb).add(&one.with_signed(self.signed && o.signed));
        r.with_signed(self.signed && o.signed)
    }

    pub fn mul(&self, o: &Bits) -> Bits {
        let w = self.width.max(o.width);
        if let Some((a, b)) = self.small(o, w) {
            return self.small_result(o, w, a.wrapping_mul(b));
        }
        self.arith(o, w, |a, b, n| {
            let mut r = vec![0u64; n];
            for i in 0..n {
                let mut carry = 0u128;
                for j in 0..n - i {
                    let t = r[i + j] as u128 + a[i] as u128 * b[j] as u128 + carry;
                    r[i + j] = t as u64;
                    carry = t >> 64;
                }
            }
            Some(r)
        })
    }

    /// Unsigned division of magnitudes, as (quotient, remainder).
    fn udivrem(a: &Bits, b: &Bits) -> (Bits, Bits) {
        let w = a.width;
        let mut q = Bits::zero(w, false);
        let mut r = Bits::zero(w, false);
        for i in (0..w).rev() {
            r = r.shl(1);
            r.set_bit(0, a.bit(i));
            if !r.ult(b) {
                r = r.sub(b).with_signed(false);
                q.set_bit(i, Logic::One);
            }
        }
        (q, r)
    }

    fn ult(&self, o: &Bits) -> bool {
        for i in (0..self.width.max(o.width)).rev() {
            let (a, b) = (self.bit(i) == Logic::One, o.bit(i) == Logic::One);
            if a != b {
                return b;
            }
        }
        false
    }

    fn divmod(&self, o: &Bits, want_rem: bool) -> Bits {
        let w = self.width.max(o.width);
        let signed = self.signed && o.signed;
        if !self.is_known() || !o.is_known() || o.is_zero() {
            return Bits::x(w).with_signed(signed);
        }
        let (a, b) = (self.extend(w).with_signed(signed), o.extend(w).with_signed(signed));
        let (na, nb) = (a.is_negative(), b.is_negative());
        let ma = if na { a.neg() } else { a.clone() }.with_signed(false);
        let mb = if nb { b.neg() } else { b.clone() }.with_signed(false);
        let (q, r) = Bits::udivrem(&ma, &mb);
        let res = if want_rem {
            if na { r.neg() } else { r }
        } else if na != nb {
            q.neg()
        } else {
            q
        };
        res.with_signed(signed)
    }

    pub fn div(&self, o: &Bits) -> Bits {
        self.divmod(o, false)
    }

    pub fn rem(&self, o: &Bits) -> Bits {
        self.divmod(o, true)
    }

    /// `**` (table 11-4). The result has the first operand's width.
    pub fn pow(&self, o: &Bits) -> Bits {
        let w = self.width;
        let signed = self.signed && o.signed;
        if !self.is_known() || !o.is_known() {
            return Bits::x(w).with_signed(signed);
        }
        let base = self.clone().with_signed(signed);
        if o.is_negative() {
            // A negative exponent: depends on the base (table 11-4).
            let one = Bits::from_u64(w, signed, 1);
            let minus_one = Bits::from_i64(w, signed, -1);
            return if base.is_zero() {
                Bits::x(w).with_signed(signed)
            } else if base == one {
                one
            } else if base == minus_one {
                if o.bit(0) == Logic::One { minus_one } else { one }
            } else {
                Bits::zero(w, signed)
            };
        }
        let mut result = Bits::from_u64(w, signed, 1);
        let mut b = base;
        for i in 0..o.width {
            if o.bit(i) == Logic::One {
                result = result.mul(&b).resize(w, false).with_signed(signed);
            }
            b = b.mul(&b).resize(w, false).with_signed(signed);
            if b.is_zero() && i + 1 < o.width && (i + 1..o.width).any(|j| o.bit(j) == Logic::One) {
                return Bits::zero(w, signed);
            }
        }
        result
    }

    // ---- shifts (11.4.10); the amount is unsigned ----

    pub fn shl(&self, n: u64) -> Bits {
        let mut r = Bits::zero(self.width, self.signed);
        if n < self.width as u64 {
            r.set_slice(n as u32, &self.slice(0, self.width - n as u32));
        }
        r
    }

    pub fn lshr(&self, n: u64) -> Bits {
        let mut r = Bits::zero(self.width, self.signed);
        if n < self.width as u64 {
            r.set_slice(0, &self.slice(n as u32, self.width - n as u32));
        }
        r
    }

    /// `>>>`: arithmetic only if the operand is signed.
    pub fn ashr(&self, n: u64) -> Bits {
        if !self.signed || self.width == 0 {
            return self.lshr(n);
        }
        let top = self.bit(self.width - 1);
        let mut r = self.lshr(n);
        let fill = n.min(self.width as u64) as u32;
        if fill > 0 && top != Logic::Zero {
            r.set_slice(self.width - fill, &Bits::fill(fill, false, top));
        }
        r
    }

    // ---- comparisons (11.4.4, 11.4.5): a 1-bit result ----

    fn compare(&self, o: &Bits) -> Option<std::cmp::Ordering> {
        if !self.is_known() || !o.is_known() {
            return None;
        }
        let w = self.width.max(o.width);
        let signed = self.signed && o.signed;
        let (a, b) = (self.extend(w).with_signed(signed), o.extend(w).with_signed(signed));
        let (na, nb) = (a.is_negative(), b.is_negative());
        if na != nb {
            return Some(if na { std::cmp::Ordering::Less } else { std::cmp::Ordering::Greater });
        }
        // The same sign: two's complement words compare as unsigned.
        Some(a.val.iter().rev().cmp(b.val.iter().rev()))
    }

    pub fn cmp_op(&self, o: &Bits, f: impl Fn(std::cmp::Ordering) -> bool) -> Logic {
        match self.compare(o) {
            Some(ord) => {
                if f(ord) {
                    Logic::One
                } else {
                    Logic::Zero
                }
            }
            None => Logic::X,
        }
    }

    /// `==`: x if any bit is unknown and no known bits differ.
    pub fn logic_eq(&self, o: &Bits) -> Logic {
        let w = self.width.max(o.width);
        let signed = self.signed && o.signed;
        let (a, b) = (self.extend(w).with_signed(signed), o.extend(w).with_signed(signed));
        let mut unknown = false;
        for k in 0..a.val.len() {
            let known = !(a.unk[k] | b.unk[k]);
            if (a.val[k] ^ b.val[k]) & known != 0 {
                return Logic::Zero;
            }
            unknown |= a.unk[k] | b.unk[k] != 0;
        }
        if unknown { Logic::X } else { Logic::One }
    }

    /// `===`: bit for bit, x and z included.
    pub fn case_eq(&self, o: &Bits) -> bool {
        let w = self.width.max(o.width);
        let signed = self.signed && o.signed;
        let (a, b) = (self.extend(w).with_signed(signed), o.extend(w).with_signed(signed));
        a.val == b.val && a.unk == b.unk
    }

    /// `==?`: x and z in the right operand match anything (11.4.6).
    pub fn wildcard_eq(&self, o: &Bits) -> Logic {
        let w = self.width.max(o.width);
        let signed = self.signed && o.signed;
        let (a, b) = (self.extend(w).with_signed(signed), o.extend(w).with_signed(signed));
        let mut unknown = false;
        for i in 0..w {
            let (x, y) = (a.bit(i), b.bit(i));
            if !y.is_known() {
                continue;
            }
            if !x.is_known() {
                unknown = true;
            } else if x != y {
                return Logic::Zero;
            }
        }
        if unknown { Logic::X } else { Logic::One }
    }

    /// The value read as a real number (6.12.2).
    pub fn to_f64(&self) -> f64 {
        if !self.is_known() {
            return 0.0;
        }
        let neg = self.is_negative();
        let m = if neg { self.neg().with_signed(false) } else { self.clone() };
        let mut r = 0.0;
        for i in (0..m.width).rev() {
            r = r * 2.0 + if m.bit(i) == Logic::One { 1.0 } else { 0.0 };
        }
        if neg { -r } else { r }
    }

    /// A real rounded to the nearest integer, ties away from zero (6.12.2).
    pub fn from_f64(width: u32, signed: bool, f: f64) -> Bits {
        let r = f.round();
        if !r.is_finite() {
            return Bits::x(width).with_signed(signed);
        }
        let neg = r < 0.0;
        let mut m = r.abs();
        let mut b = Bits::zero(width, signed);
        let mut i = 0;
        while m >= 1.0 && i < width {
            b.set_bit(i, if m % 2.0 >= 1.0 { Logic::One } else { Logic::Zero });
            m = (m / 2.0).floor();
            i += 1;
        }
        if neg { b.neg().with_signed(signed) } else { b }
    }

    /// The bytes of a string literal, first character most significant
    /// (5.9).
    pub fn from_str_bytes(s: &[u8]) -> Bits {
        let mut b = Bits::zero((s.len() as u32 * 8).max(8), false);
        for (k, &c) in s.iter().rev().enumerate() {
            b.set_slice(k as u32 * 8, &Bits::from_u64(8, false, c as u64));
        }
        b
    }

    /// The bytes of the value as text, skipping zero bytes (6.16).
    pub fn to_str_bytes(&self) -> Vec<u8> {
        let n = self.width.div_ceil(8);
        let mut out = Vec::new();
        for k in (0..n).rev() {
            let c = self.slice(k * 8, 8).to_u64().unwrap_or(0) as u8;
            if c != 0 {
                out.push(c);
            }
        }
        out
    }

    /// Unsigned decimal digits, or `x`/`z` when unknown.
    pub fn to_decimal(&self) -> String {
        if !self.is_known() {
            let all_z = (0..self.width).all(|i| self.bit(i) == Logic::Z);
            return if all_z { "z".into() } else { "x".into() };
        }
        let neg = self.is_negative();
        let mut m = if neg { self.neg().with_signed(false) } else { self.clone().with_signed(false) };
        if m.is_zero() {
            return "0".into();
        }
        let ten = Bits::from_u64(m.width.max(4), false, 10);
        m = m.resize(m.width.max(4), false);
        let mut digits = Vec::new();
        while !m.is_zero() {
            let (q, r) = Bits::udivrem(&m, &ten);
            digits.push(b'0' + r.to_u64().unwrap() as u8);
            m = q;
        }
        if neg {
            digits.push(b'-');
        }
        digits.reverse();
        String::from_utf8(digits).unwrap()
    }

    /// Digits in base 2, 8 or 16, most significant first; a digit with
    /// mixed unknown bits is `X` or `Z` if all are that, else `x`.
    pub fn to_radix(&self, bits_per_digit: u32) -> String {
        let n = self.width.div_ceil(bits_per_digit).max(1);
        let mut s = String::new();
        for d in (0..n).rev() {
            let lo = d * bits_per_digit;
            let hi = (lo + bits_per_digit).min(self.width);
            let ls: Vec<Logic> = (lo..hi).map(|i| self.bit(i)).collect();
            if ls.iter().all(|l| l.is_known()) {
                let v = ls.iter().enumerate().fold(0u32, |a, (i, l)| a | ((*l == Logic::One) as u32) << i);
                s.push(std::char::from_digit(v, 16).unwrap());
            } else if ls.iter().all(|&l| l == Logic::Z) {
                s.push('z');
            } else if ls.iter().all(|&l| l == Logic::X) {
                s.push('x');
            } else if ls.contains(&Logic::X) {
                s.push('X');
            } else {
                s.push('Z');
            }
        }
        s
    }
}

impl fmt::Debug for Bits {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self}")
    }
}

/// The literal form: `8'hff`, `4'sb10x1`, `32'd5`.
impl fmt::Display for Bits {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = if self.signed { "s" } else { "" };
        if self.is_known() {
            if self.width <= 64 {
                if self.is_negative() {
                    write!(f, "-{}'{s}d{}", self.width, &self.to_decimal()[1..])
                } else {
                    write!(f, "{}'{s}d{}", self.width, self.to_decimal())
                }
            } else {
                write!(f, "{}'{s}h{}", self.width, self.to_radix(4))
            }
        } else if self.width.is_multiple_of(4) && !self.to_radix(4).contains(['X', 'Z']) {
            write!(f, "{}'{s}h{}", self.width, self.to_radix(4))
        } else {
            write!(f, "{}'{s}b{}", self.width, self.to_radix(1))
        }
    }
}

/// Reads a literal's digits in base `base` (2, 8, 10 or 16) into a vector
/// of `width` bits, or of as many bits as the digits need when `width` is
/// `None`. Underscores are skipped; `x`, `z` and `?` fill a digit.
pub fn parse_digits(digits: &str, base: u32, width: Option<u32>, signed: bool) -> Result<Bits, String> {
    let ds: Vec<u8> = digits.bytes().filter(|&c| c != b'_').collect();
    if base == 10 {
        if ds.len() == 1 && matches!(ds[0], b'x' | b'X' | b'z' | b'Z' | b'?') {
            let l = if matches!(ds[0], b'x' | b'X') { Logic::X } else { Logic::Z };
            return Ok(Bits::fill(width.unwrap_or(32), signed, l));
        }
        let mut v = Bits::zero(width.unwrap_or(32).max(1), false);
        // Accumulate in a wide vector, then check it fits.
        let wide = (ds.len() as u32 * 4 + 4).max(v.width());
        let mut acc = Bits::zero(wide, false);
        let ten = Bits::from_u64(wide, false, 10);
        for &c in &ds {
            if !c.is_ascii_digit() {
                return Err(format!("`{}` is not a decimal digit", c as char));
            }
            acc = acc.mul(&ten).add(&Bits::from_u64(wide, false, (c - b'0') as u64));
        }
        let needed = (0..wide).rev().find(|&i| acc.bit(i) == Logic::One).map_or(1, |i| i + 1);
        let w = width.unwrap_or(32.max(needed));
        if width.is_some() && needed > w {
            // 5.7.1: a sized literal's extra bits are truncated (a warning).
        }
        v = acc.resize(w, false);
        return Ok(v.with_signed(signed));
    }
    let per = match base {
        2 => 1,
        8 => 3,
        _ => 4,
    };
    let natural = ds.len() as u32 * per;
    let mut b = Bits::zero(natural.max(1), false);
    for (k, &c) in ds.iter().rev().enumerate() {
        let lo = k as u32 * per;
        match c {
            b'x' | b'X' => (0..per).for_each(|i| b.set_bit(lo + i, Logic::X)),
            b'z' | b'Z' | b'?' => (0..per).for_each(|i| b.set_bit(lo + i, Logic::Z)),
            _ => {
                let d = (c as char).to_digit(16).ok_or_else(|| format!("`{}` is not a digit", c as char))?;
                for i in 0..per {
                    b.set_bit(lo + i, if d >> i & 1 != 0 { Logic::One } else { Logic::Zero });
                }
            }
        }
    }
    let w = width.unwrap_or(natural.max(32));
    Ok(b.extend_literal(w).with_signed(signed))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arithmetic() {
        let a = Bits::from_u64(8, false, 200);
        let b = Bits::from_u64(8, false, 100);
        assert_eq!(a.add(&b).to_u64(), Some(44));
        assert_eq!(b.sub(&a).to_u64(), Some(156));
        assert_eq!(a.mul(&b).to_u64(), Some((200 * 100) % 256));
        assert_eq!(a.div(&b).to_u64(), Some(2));
        let m7 = Bits::from_i64(8, true, -7);
        let two = Bits::from_i64(8, true, 2);
        assert_eq!(m7.div(&two).to_i64(), Some(-3));
        assert_eq!(m7.rem(&two).to_i64(), Some(-1));
        assert_eq!(Bits::from_u64(32, false, 3).pow(&Bits::from_u64(32, false, 4)).to_u64(), Some(81));
        assert!(!a.add(&Bits::x(8)).is_known());
        let wide = Bits::from_u64(100, false, u64::MAX).add(&Bits::from_u64(100, false, 1));
        assert_eq!(wide.to_radix(4), "0000000010000000000000000");
        assert_eq!(wide.slice(64, 1).to_u64(), Some(1));
    }

    #[test]
    fn literals() {
        assert_eq!(parse_digits("ff", 16, Some(8), false).unwrap().to_u64(), Some(255));
        assert_eq!(parse_digits("1x", 2, Some(4), false).unwrap().to_string(), "4'b001x");
        assert_eq!(parse_digits("z", 16, None, false).unwrap().to_string(), "32'hzzzzzzzz");
        assert_eq!(parse_digits("12", 10, None, false).unwrap().to_string(), "32'd12");
        assert_eq!(Bits::from_i64(8, true, -3).to_string(), "-8'sd3");
        assert_eq!(Bits::from_str_bytes(b"AB").to_u64(), Some(0x4142));
    }

    #[test]
    fn logic() {
        let a = parse_digits("10xz", 2, Some(4), false).unwrap();
        let b = parse_digits("1100", 2, Some(4), false).unwrap();
        assert_eq!(a.and(&b).to_radix(1), "1000");
        assert_eq!(a.or(&b).to_radix(1), "11xx");
        assert_eq!(a.logic_eq(&b), Logic::Zero);
        assert_eq!(b.wildcard_eq(&parse_digits("1x0z", 2, Some(4), false).unwrap()), Logic::One);
        assert_eq!(Bits::from_i64(4, true, -2).ashr(1).to_i64(), Some(-1));
    }

    /// A bit at a time, as the reference for the word-wise operators.
    fn per_bit(a: &Bits, o: &Bits, f: impl Fn(Logic, Logic) -> Logic) -> Bits {
        let w = a.width.max(o.width);
        let (x, y) = (a.extend(w), o.extend(w));
        let mut r = Bits::zero(w, a.signed && o.signed);
        for i in 0..w {
            r.set_bit(i, f(x.bit(i), y.bit(i)));
        }
        r
    }

    fn known(l: Logic) -> Logic {
        if l.is_known() { l } else { Logic::X }
    }

    /// Word-wise operators agree with a bit at a time, across word
    /// boundaries and with x and z.
    #[test]
    fn word_ops() {
        let mut seed = 0x1234_5678_9abc_def0u64;
        let mut rnd = move || {
            seed ^= seed << 13;
            seed ^= seed >> 7;
            seed ^= seed << 17;
            seed
        };
        let all = [Logic::Zero, Logic::One, Logic::X, Logic::Z];
        for w in [1u32, 3, 63, 64, 65, 100, 130] {
            for _ in 0..20 {
                let make = |rnd: &mut dyn FnMut() -> u64, unknowns: bool| {
                    let mut b = Bits::zero(w, rnd() & 1 == 0);
                    for i in 0..w {
                        let r = rnd();
                        let l = if unknowns { all[(r % 4) as usize] } else { all[(r % 2) as usize] };
                        b.set_bit(i, l);
                    }
                    b
                };
                let (a, b) = (make(&mut rnd, true), make(&mut rnd, true));
                let and = per_bit(&a, &b, |x, y| match (x, y) {
                    (Logic::Zero, _) | (_, Logic::Zero) => Logic::Zero,
                    (Logic::One, Logic::One) => Logic::One,
                    _ => Logic::X,
                });
                assert!(a.and(&b).case_eq(&and), "and {w}");
                let or = per_bit(&a, &b, |x, y| match (x, y) {
                    (Logic::One, _) | (_, Logic::One) => Logic::One,
                    (Logic::Zero, Logic::Zero) => Logic::Zero,
                    _ => Logic::X,
                });
                assert!(a.or(&b).case_eq(&or), "or {w}");
                let xor = per_bit(&a, &b, |x, y| match (known(x), known(y)) {
                    (Logic::X, _) | (_, Logic::X) => Logic::X,
                    (p, q) if p == q => Logic::Zero,
                    _ => Logic::One,
                });
                assert!(a.xor(&b).case_eq(&xor), "xor {w}");
                let not = per_bit(&a, &a, |x, _| match x {
                    Logic::Zero => Logic::One,
                    Logic::One => Logic::Zero,
                    _ => Logic::X,
                });
                assert!(a.not().case_eq(&not.with_signed(a.signed)), "not {w}");
                let lo = (rnd() % (w as u64 + 10)) as u32;
                let n = (rnd() % 70) as u32 + 1;
                let sl = a.slice(lo, n);
                for i in 0..n {
                    let want = if lo + i < w { a.bit(lo + i) } else { Logic::X };
                    assert_eq!(sl.bit(i), want, "slice {w} {lo} {n} bit {i}");
                }
                let mut c = a.clone();
                c.set_slice(lo, &b);
                for i in 0..w {
                    let want = if i >= lo && i - lo < b.width { b.bit(i - lo) } else { a.bit(i) };
                    assert_eq!(c.bit(i), want, "set_slice {w} {lo} bit {i}");
                }
                let sh = rnd() % (w as u64 + 2);
                let (l, r) = (a.shl(sh), a.lshr(sh));
                for i in 0..w as u64 {
                    let wl = if i >= sh { a.bit((i - sh) as u32) } else { Logic::Zero };
                    let wr = if i + sh < w as u64 { a.bit((i + sh) as u32) } else { Logic::Zero };
                    assert_eq!(l.bit(i as u32), wl, "shl {w} {sh}");
                    assert_eq!(r.bit(i as u32), wr, "lshr {w} {sh}");
                }
                let (p, q) = (make(&mut rnd, false), make(&mut rnd, false));
                let ord = |x: &Bits, y: &Bits| {
                    let s = x.signed && y.signed;
                    let m = x.width.max(y.width);
                    let (x, y) = (x.extend(m).with_signed(s), y.extend(m).with_signed(s));
                    let (nx, ny) = (x.is_negative(), y.is_negative());
                    if nx != ny {
                        return if nx { std::cmp::Ordering::Less } else { std::cmp::Ordering::Greater };
                    }
                    (0..m).rev().map(|i| (x.bit(i) == Logic::One).cmp(&(y.bit(i) == Logic::One))).find(|o| o.is_ne()).unwrap_or(std::cmp::Ordering::Equal)
                };
                assert_eq!(p.compare(&q), Some(ord(&p, &q)), "compare {w}");
                if w <= 64 {
                    let m = if w == 64 { u64::MAX } else { (1u64 << w) - 1 };
                    let (x, y) = (p.to_u64().unwrap(), q.to_u64().unwrap());
                    assert_eq!(p.add(&q).to_u64(), Some(x.wrapping_add(y) & m), "add {w}");
                    assert_eq!(p.sub(&q).to_u64(), Some(x.wrapping_sub(y) & m), "sub {w}");
                    assert_eq!(p.mul(&q).to_u64(), Some(x.wrapping_mul(y) & m), "mul {w}");
                }
            }
        }
    }
}
