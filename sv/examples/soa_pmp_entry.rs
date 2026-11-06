//! A mock of what the compiled netlist (docs/design/sv.md sections 8.1
//! and 8.3) might emit for one CVA6 specialisation, in the two-state
//! model, with every bit of every signal in an AVX-512 register of its
//! own. Written by hand; nothing generates it yet.
//!
//! The specialisation is `pmp_entry` with PLEN = 56 (cv64a6_imafdc_sv39):
//! one physical memory protection entry, saying whether an address falls
//! in a TOR (top of range) or NAPOT (naturally aligned power of two)
//! region. It is combinational, and CVA6 has 24 of them (three `pmp`
//! units of eight entries), all one specialisation.
//!
//! **Layout.** A lane is one instance in one simulation. A `Plane` is a
//! 512-bit register holding one bit of a signal for 512 lanes, and a
//! w-bit signal is `[Plane; w]`, least significant bit first. The state
//! of a group of 512 lanes is a structure of arrays of planes, and every
//! operator becomes bitwise logic on planes: `vpternlogq` computes any
//! function of three planes, so a multiplexer is one instruction for 512
//! lanes and a full adder two. Both arms of a `case` are computed in
//! every lane and the case expression selects between them; there are no
//! branches.
//!
//! **Lanes.** One CVA6 has only 24 lanes of `pmp_entry`, which would leave
//! 95% of each register idle, so the groups are filled by simulating many
//! cores at once: 64 cores give 1,536 lanes, three groups.
//!
//! **AVX-512BW.** The planes themselves need only AVX-512F (`vpternlogq`).
//! BW is for moving values in and out of planes at the edge of the
//! netlist: `vptestmb` takes one bit of each of 64 bytes into a 64-bit
//! mask register, the first step of the transpose in `to_planes`.
//!
//! **32-bit lanes.** `pmp_entry` is mostly 56-bit comparisons, which bit planes
//! do a bit at a time. So it is also mocked in section 8.3's other layout:
//! the addresses in 32-bit lanes, two pieces each, 16 lanes to a register,
//! with the mode and the match left in bit planes. Comparisons write mask
//! registers, which are pieces of the match's plane, so the two layouts
//! meet with no transpose. The per-lane trailing-zero count uses
//! AVX-512CD's `vplzcntd`.
//!
//! Run on a machine with AVX-512BW and CD:
//! `RUSTFLAGS="-C target-cpu=native" cargo run --release --example soa_pmp_entry`.
//! It checks every lane of both layouts against a scalar transcription of
//! the SystemVerilog, and times all three.

#[cfg(all(target_arch = "x86_64", target_feature = "avx512bw", target_feature = "avx512cd"))]
#[allow(unused_unsafe)]
mod soa {
    use std::arch::x86_64::*;
    use std::hint::black_box;
    use std::time::Instant;

    type Plane = __m512i;
    const LANES: usize = 512;
    const PLEN: usize = 56;
    /// The width of the configured addresses, `PLEN - 2`.
    const CW: usize = PLEN - 2;

    /// The ports of `pmp_entry` for a group of 512 lanes, a plane per bit.
    #[repr(C, align(64))]
    struct PmpEntry {
        addr_i: [Plane; PLEN],
        conf_addr_i: [Plane; CW],
        conf_addr_prev_i: [Plane; CW],
        conf_addr_mode_i: [Plane; 2],
        match_o: [Plane; 1],
    }

    // The truth tables of vpternlogq's three operands; an immediate is any
    // bitwise expression of them.
    const A: i32 = 0xF0;
    const B: i32 = 0xCC;
    const C: i32 = 0xAA;

    #[inline(always)]
    fn tern<const IMM: i32>(a: Plane, b: Plane, c: Plane) -> Plane {
        unsafe { _mm512_ternarylogic_epi64::<IMM>(a, b, c) }
    }
    #[inline(always)]
    fn zero() -> Plane {
        unsafe { _mm512_setzero_si512() }
    }
    #[inline(always)]
    fn ones() -> Plane {
        unsafe { _mm512_set1_epi64(-1) }
    }
    #[inline(always)]
    fn and(a: Plane, b: Plane) -> Plane {
        unsafe { _mm512_and_si512(a, b) }
    }
    /// `!a & b`.
    #[inline(always)]
    fn andnot(a: Plane, b: Plane) -> Plane {
        unsafe { _mm512_andnot_si512(a, b) }
    }
    #[inline(always)]
    fn xor(a: Plane, b: Plane) -> Plane {
        unsafe { _mm512_xor_si512(a, b) }
    }
    #[inline(always)]
    fn not(a: Plane) -> Plane {
        tern::<{ !A & 0xFF }>(a, a, a)
    }
    /// `s ? a : b`.
    #[inline(always)]
    fn mux(s: Plane, a: Plane, b: Plane) -> Plane {
        tern::<{ (A & B) | (!A & C & 0xFF) }>(s, a, b)
    }

    /// `a < b`, unsigned: the borrow out of `a - b`, a ripple of majority
    /// gates `borrow' = maj(!a, b, borrow)`.
    #[inline(always)]
    fn lt(w: usize, a: impl Fn(usize) -> Plane, b: impl Fn(usize) -> Plane) -> Plane {
        (0..w).fold(zero(), |borrow, i| tern::<{ (!A & B | !A & C | B & C) & 0xFF }>(a(i), b(i), borrow))
    }

    /// The trailing zeros of `x`, as `lzc` counts them with MODE = 0 (`x` is
    /// never zero here). Bit k of the count gathers the lanes whose lowest
    /// one is at a position with bit k set.
    #[inline(always)]
    fn tzc<const N: usize>(w: usize, x: impl Fn(usize) -> Plane) -> [Plane; N] {
        let mut cnt = [zero(); N];
        // The lanes with no one below position i.
        let mut none = ones();
        for i in 0..w {
            let xi = x(i);
            for (k, c) in cnt.iter_mut().enumerate() {
                if i >> k & 1 == 1 {
                    *c = tern::<{ A | (B & C) }>(*c, xi, none);
                }
            }
            none = andnot(xi, none);
        }
        cnt
    }

    /// `x + k` for a constant `k`, in `x`'s width: a ripple of full adders,
    /// two instructions each. A compiler would fold `k`'s bits away.
    #[inline(always)]
    fn add_const<const N: usize>(x: [Plane; N], k: u64) -> [Plane; N] {
        let mut carry = zero();
        std::array::from_fn(|i| {
            let ki = if k >> i & 1 == 1 { ones() } else { zero() };
            let sum = tern::<{ A ^ B ^ C }>(x[i], ki, carry);
            carry = tern::<{ (A & B) | (A & C) | (B & C) }>(x[i], ki, carry);
            sum
        })
    }

    /// `'1 << s` in W bits: a barrel shifter, one stage of multiplexers per
    /// bit of `s`, each stage in place from the top down.
    #[inline(always)]
    fn ones_shl<const W: usize>(s: &[Plane]) -> [Plane; W] {
        let mut v = [ones(); W];
        for (k, &sk) in s.iter().enumerate() {
            let d = 1 << k;
            for i in (0..W).rev() {
                v[i] = mux(sk, if i >= d { v[i - d] } else { zero() }, v[i]);
            }
        }
        v
    }

    /// One evaluation of every lane's `always_comb`.
    #[inline(never)]
    fn eval(p: &mut PmpEntry) {
        let (addr, conf, prev, mode) = (&p.addr_i, &p.conf_addr_i, &p.conf_addr_prev_i, &p.conf_addr_mode_i);

        // TOR: ({2'b0, tor_conf_addr_prev} << 2) <= addr_i < ({2'b0,
        // tor_conf_addr} << 2), where the tor_ addresses clear bit 0.
        let tor_shifted = |c: &[Plane; CW], i: usize| if i < 3 { zero() } else { c[i - 2] };
        let below = lt(PLEN, |i| addr[i], |i| tor_shifted(prev, i));
        let tor = andnot(below, lt(PLEN, |i| addr[i], |i| tor_shifted(conf, i)));

        // NAPOT: trail_ones is lzc's trailing-zero count of conf_addr_n =
        // {2'b11, ~conf_addr_i}.
        let trail_ones = tzc::<6>(PLEN, |i| if i < CW { not(conf[i]) } else { ones() });
        // size = trail_ones + 3 takes 7 bits (63 + 3); the int's other 25
        // bits are known to be zero.
        let mut size = [zero(); 7];
        size[..6].copy_from_slice(&trail_ones);
        let size = add_const(size, 3);
        let mask = ones_shl::<PLEN>(&size);
        // match = (addr_i & mask) == (({2'b0, conf_addr_i} << 2) & mask)
        let mut diff = zero();
        for i in 0..PLEN {
            let c = if i < 2 { zero() } else { conf[i - 2] };
            diff = tern::<{ A | (B & C) }>(diff, mask[i], xor(addr[i], c));
        }
        let napot = not(diff);

        // case (conf_addr_mode_i): TOR = 1 and NAPOT = 3; OFF and NA4 give 0.
        p.match_o[0] = and(mode[0], mux(mode[1], napot, tor));
    }

    // The same specialisation with the addresses in 32-bit lanes (section
    // 8.3's other layout), 16 lanes to a register. The 56-bit address and
    // the 54-bit configured addresses are two 32-bit pieces, low piece
    // first. The mode and the match are bit planes still: a 512-bit plane
    // is also 32 16-bit masks, one per register of lanes, so a comparison's
    // mask is a piece of a plane and nothing is transposed between layouts.

    /// The registers of 32-bit lanes in a group.
    const CHUNKS: usize = LANES / 16;
    /// One 32-bit piece of a value for a group.
    type Piece = [__m512i; CHUNKS];
    /// The two pieces of one register of lanes, `[low, high]`.
    type Pair = [__m512i; 2];

    /// The ports of `pmp_entry` for a group of 512 lanes, the addresses in
    /// lanes.
    #[repr(C, align(64))]
    struct PmpEntryLanes {
        addr_i: [Piece; 2],
        conf_addr_i: [Piece; 2],
        conf_addr_prev_i: [Piece; 2],
        conf_addr_mode_i: [Plane; 2],
        match_o: [Plane; 1],
    }

    #[inline(always)]
    fn set1(x: u32) -> __m512i {
        unsafe { _mm512_set1_epi32(x as i32) }
    }

    /// `{2'b0, x} << 2` in 56 bits: a shift by a constant moves two bits
    /// from the low piece to the high one.
    #[inline(always)]
    fn shl2(x: Pair) -> Pair {
        unsafe {
            let hi = _mm512_or_si512(_mm512_slli_epi32::<2>(x[1]), _mm512_srli_epi32::<30>(x[0]));
            [_mm512_slli_epi32::<2>(x[0]), and(hi, set1(0xFF_FFFF))]
        }
    }

    /// `a < b`, unsigned: the high pieces decide unless they are equal.
    #[inline(always)]
    fn lt2(a: Pair, b: Pair) -> __mmask16 {
        unsafe {
            let eq = _mm512_cmpeq_epi32_mask(a[1], b[1]);
            _mm512_cmplt_epu32_mask(a[1], b[1]) | _mm512_mask_cmplt_epu32_mask(eq, a[0], b[0])
        }
    }

    /// The trailing zeros of each lane, 32 for a zero lane. AVX-512 has no
    /// tzcnt per lane, but `!x & (x - 1)` has exactly that many ones, at the
    /// bottom, which `vplzcntd` counts from the top.
    #[inline(always)]
    fn tz32(x: __m512i) -> __m512i {
        unsafe { _mm512_sub_epi32(set1(32), _mm512_lzcnt_epi32(andnot(x, _mm512_sub_epi32(x, set1(1))))) }
    }

    /// `'1 << s` in 56 bits, for `s` up to 63. Variable shifts of 32 or more
    /// give zero, so each piece is the OR of the shifts that can reach it.
    #[inline(always)]
    fn ones_shl2(s: __m512i) -> Pair {
        unsafe {
            let ones = set1(!0);
            let hi = _mm512_or_si512(
                _mm512_or_si512(_mm512_sllv_epi32(ones, s), _mm512_srlv_epi32(ones, _mm512_sub_epi32(set1(32), s))),
                _mm512_sllv_epi32(ones, _mm512_sub_epi32(s, set1(32))),
            );
            [_mm512_sllv_epi32(ones, s), and(hi, set1(0xFF_FFFF))]
        }
    }

    /// One evaluation of every lane's `always_comb`, 16 lanes at a time.
    #[inline(never)]
    fn eval_lanes(p: &mut PmpEntryLanes) {
        let mode = p.conf_addr_mode_i.map(|m| unsafe { std::mem::transmute::<Plane, [u16; CHUNKS]>(m) });
        let mut out = [0u16; CHUNKS];
        for (j, out) in out.iter_mut().enumerate() {
            let pair = |x: &[Piece; 2]| [x[0][j], x[1][j]];
            let (addr, conf, prev) = (pair(&p.addr_i), pair(&p.conf_addr_i), pair(&p.conf_addr_prev_i));

            // TOR, with bit 0 of each configured address cleared.
            let clear0 = |x: Pair| [and(x[0], set1(!1)), x[1]];
            let below = lt2(addr, shl2(clear0(prev)));
            let tor = !below & lt2(addr, shl2(clear0(conf)));

            // NAPOT: the trailing zeros of conf_addr_n = {2'b11, ~conf_addr_i}.
            // conf_addr_i's high piece is zero above bit 21, so ~ supplies
            // the 2'b11 and there is always a one in the high piece.
            let (n0, n1) = (not(conf[0]), not(conf[1]));
            let t0 = tz32(n0);
            let trail_ones = unsafe { _mm512_mask_add_epi32(t0, _mm512_cmpeq_epi32_mask(n0, zero()), t0, tz32(n1)) };
            let size = unsafe { _mm512_add_epi32(trail_ones, set1(3)) };
            let mask = ones_shl2(size);
            // (addr_i & mask) == (({2'b0, conf_addr_i} << 2) & mask)
            let base = shl2(conf);
            let napot = unsafe {
                _mm512_testn_epi32_mask(xor(addr[0], base[0]), mask[0])
                    & _mm512_testn_epi32_mask(xor(addr[1], base[1]), mask[1])
            };

            // case (conf_addr_mode_i): TOR = 1 and NAPOT = 3.
            let (m0, m1) = (mode[0][j], mode[1][j]);
            *out = m0 & (m1 & napot | !m1 & tor);
        }
        p.match_o[0] = unsafe { std::mem::transmute::<[u16; CHUNKS], Plane>(out) };
    }

    /// One value per lane into two 32-bit pieces: `vpmovqd` truncates eight
    /// 64-bit values to 32 bits, after a shift for the high piece. Nothing
    /// is transposed.
    fn to_lanes(v: &[u64; LANES]) -> [Piece; 2] {
        let mut out = [[zero(); CHUNKS]; 2];
        for j in 0..CHUNKS {
            unsafe {
                let a = _mm512_loadu_si512(v[16 * j..].as_ptr().cast());
                let b = _mm512_loadu_si512(v[16 * j + 8..].as_ptr().cast());
                let join = |a: __m512i, b: __m512i| {
                    _mm512_inserti64x4::<1>(_mm512_castsi256_si512(_mm512_cvtepi64_epi32(a)), _mm512_cvtepi64_epi32(b))
                };
                out[0][j] = join(a, b);
                out[1][j] = join(_mm512_srli_epi64::<32>(a), _mm512_srli_epi64::<32>(b));
            }
        }
        out
    }

    /// The bit planes of a narrow value: `vptestmq` takes bit b of eight
    /// 64-bit values into a mask that is already in lane order.
    fn to_planes_narrow<const W: usize>(v: &[u64; LANES]) -> [Plane; W] {
        let mut words = [[0u64; LANES / 64]; W];
        for k in 0..LANES / 8 {
            let r = unsafe { _mm512_loadu_si512(v[8 * k..].as_ptr().cast()) };
            for (b, w) in words.iter_mut().enumerate() {
                let m = unsafe { _mm512_test_epi64_mask(r, _mm512_set1_epi64(1 << b)) };
                w[k / 8] |= (m as u64) << (8 * (k % 8));
            }
        }
        std::array::from_fn(|i| unsafe { _mm512_loadu_si512(words[i].as_ptr().cast()) })
    }

    /// Transposes an 8x8 bit matrix held a row per byte.
    fn transpose8(mut x: u64) -> u64 {
        let t = (x ^ (x >> 7)) & 0x00AA_00AA_00AA_00AA;
        x ^= t ^ (t << 7);
        let t = (x ^ (x >> 14)) & 0x0000_CCCC_0000_CCCC;
        x ^= t ^ (t << 14);
        let t = (x ^ (x >> 28)) & 0x0000_0000_F0F0_F0F0;
        x ^= t ^ (t << 28);
        x
    }

    /// The bit planes of one value per lane: plane i holds bit i of every
    /// lane. A register holds eight values; `vptestmb` takes bit t of each
    /// of its 64 bytes into a mask, which an 8x8 transpose sorts by byte.
    fn to_planes<const W: usize>(v: &[u64; LANES]) -> [Plane; W] {
        let mut words = [[0u64; LANES / 64]; 64];
        for k in 0..LANES / 8 {
            let r = unsafe { _mm512_loadu_si512(v[8 * k..].as_ptr().cast()) };
            for t in 0..8 {
                // Bit 8j + b: bit 8b + t of value 8k + j.
                let m = unsafe { _mm512_test_epi8_mask(r, _mm512_set1_epi8((1u8 << t) as i8)) };
                let m = transpose8(m);
                for b in 0..8 {
                    words[8 * b + t][k / 8] |= (m >> (8 * b) & 0xFF) << (8 * (k % 8));
                }
            }
        }
        std::array::from_fn(|i| unsafe { _mm512_loadu_si512(words[i].as_ptr().cast()) })
    }

    fn from_plane(p: Plane) -> [u64; LANES / 64] {
        let mut w = [0u64; LANES / 64];
        unsafe { _mm512_storeu_si512(w.as_mut_ptr().cast(), p) };
        w
    }

    /// `pmp_entry`'s `always_comb` for one lane, transcribed from the source.
    fn reference(addr: u64, conf: u64, prev: u64, mode: u64) -> bool {
        let m56 = (1u64 << PLEN) - 1;
        match mode {
            1 => {
                let lo = ((prev & !1) << 2) & m56;
                let hi = ((conf & !1) << 2) & m56;
                addr >= lo && addr < hi
            }
            3 => {
                let conf_n = (0b11 << CW) | (!conf & ((1 << CW) - 1));
                let size = conf_n.trailing_zeros() + 3;
                let mask = (m56 << size) & m56;
                (addr & mask) == ((conf << 2) & mask)
            }
            _ => false,
        }
    }

    struct Rng(u64);
    impl Rng {
        fn next(&mut self) -> u64 {
            self.0 ^= self.0 << 13;
            self.0 ^= self.0 >> 7;
            self.0 ^= self.0 << 17;
            self.0
        }
    }

    struct Stimulus {
        addr: [u64; LANES],
        conf: [u64; LANES],
        prev: [u64; LANES],
        mode: [u64; LANES],
    }

    /// Random configurations, with addresses often inside the region.
    fn stimulus(rng: &mut Rng) -> Stimulus {
        let (m56, mcw) = ((1u64 << PLEN) - 1, (1u64 << CW) - 1);
        let mut s = Stimulus { addr: [0; LANES], conf: [0; LANES], prev: [0; LANES], mode: [0; LANES] };
        for n in 0..LANES {
            let mode = rng.next() % 4;
            let t = rng.next() % (CW as u64 + 1);
            let conf = if mode == 3 { (rng.next() << (t + 1) | ((1 << t) - 1)) & mcw } else { rng.next() & mcw };
            let prev = rng.next() & mcw;
            let (lo, hi) = (((prev & !1) << 2) & m56, ((conf & !1) << 2) & m56);
            s.addr[n] = match rng.next() % 3 {
                0 => rng.next() & m56,
                1 if mode == 3 => ((conf << 2) & m56) ^ (rng.next() & ((1u64 << (t + 3).min(56)) - 1)),
                _ if hi > lo => lo + rng.next() % (hi - lo),
                _ => rng.next() & m56,
            };
            (s.conf[n], s.prev[n], s.mode[n]) = (conf, prev, mode);
        }
        s
    }

    pub fn run() {
        // 64 cores of 24 entries.
        const GROUPS: usize = 3;
        let mut rng = Rng(0x9E37_79B9_7F4A_7C15);
        let inputs: Vec<Stimulus> = (0..GROUPS).map(|_| stimulus(&mut rng)).collect();
        let t = Instant::now();
        let mut groups: Vec<Box<PmpEntry>> = inputs
            .iter()
            .map(|s| {
                Box::new(PmpEntry {
                    addr_i: to_planes(&s.addr),
                    conf_addr_i: to_planes(&s.conf),
                    conf_addr_prev_i: to_planes(&s.prev),
                    conf_addr_mode_i: to_planes(&s.mode),
                    match_o: [zero()],
                })
            })
            .collect();
        let transpose = t.elapsed().as_secs_f64() / (GROUPS * LANES) as f64;

        let t = Instant::now();
        let mut lane_groups: Vec<Box<PmpEntryLanes>> = inputs
            .iter()
            .map(|s| {
                Box::new(PmpEntryLanes {
                    addr_i: to_lanes(&s.addr),
                    conf_addr_i: to_lanes(&s.conf),
                    conf_addr_prev_i: to_lanes(&s.prev),
                    conf_addr_mode_i: to_planes_narrow(&s.mode),
                    match_o: [zero()],
                })
            })
            .collect();
        let to_lanes_time = t.elapsed().as_secs_f64() / (GROUPS * LANES) as f64;

        let mut matches = 0;
        for ((g, l), s) in groups.iter_mut().zip(lane_groups.iter_mut()).zip(&inputs) {
            eval(g);
            eval_lanes(l);
            let (out, out_lanes) = (from_plane(g.match_o[0]), from_plane(l.match_o[0]));
            for n in 0..LANES {
                let want = reference(s.addr[n], s.conf[n], s.prev[n], s.mode[n]);
                for (layout, out) in [("bit planes", out), ("lanes", out_lanes)] {
                    let got = out[n / 64] >> (n % 64) & 1 == 1;
                    assert_eq!(
                        got, want,
                        "{layout}, lane {n}: addr {:x} conf {:x} prev {:x} mode {}",
                        s.addr[n], s.conf[n], s.prev[n], s.mode[n]
                    );
                }
                matches += want as usize;
            }
        }
        println!(
            "{} lanes agree with the scalar model in both layouts ({matches} of them match)",
            GROUPS * LANES
        );

        const ITERS: usize = 100_000;
        let t = Instant::now();
        for _ in 0..ITERS {
            for g in groups.iter_mut() {
                eval(black_box(&mut **g));
            }
        }
        let soa = t.elapsed().as_secs_f64() / (ITERS * GROUPS * LANES) as f64;
        let t = Instant::now();
        for _ in 0..ITERS {
            for l in lane_groups.iter_mut() {
                eval_lanes(black_box(&mut **l));
            }
        }
        let lanes = t.elapsed().as_secs_f64() / (ITERS * GROUPS * LANES) as f64;
        let t = Instant::now();
        let mut hits = 0u64;
        for _ in 0..ITERS {
            for s in &inputs {
                for n in 0..LANES {
                    hits += reference(black_box(s.addr[n]), black_box(s.conf[n]), s.prev[n], s.mode[n]) as u64;
                }
            }
        }
        black_box(hits);
        let scalar = t.elapsed().as_secs_f64() / (ITERS * GROUPS * LANES) as f64;
        println!(
            "eval per lane: bit planes {:.3} ns, 32-bit lanes {:.3} ns, scalar {:.3} ns",
            soa * 1e9,
            lanes * 1e9,
            scalar * 1e9
        );
        println!(
            "moving inputs in, per lane: bit planes {:.1} ns, 32-bit lanes {:.1} ns",
            transpose * 1e9,
            to_lanes_time * 1e9
        );
    }
}

#[cfg(all(target_arch = "x86_64", target_feature = "avx512bw", target_feature = "avx512cd"))]
fn main() {
    soa::run();
}

#[cfg(not(all(target_arch = "x86_64", target_feature = "avx512bw", target_feature = "avx512cd")))]
fn main() {
    eprintln!(
        "soa_pmp_entry needs AVX-512BW and CD: build it with RUSTFLAGS=\"-C target-cpu=native\" on a machine that has it"
    );
}
