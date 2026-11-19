// Masks' bits and lane shuffles that one instruction does (pmovmskb,
// pshufd, pshuflw and pshufhw, pshufb, pslldq, psrldq, palignr; with AVX2
// their 256-bit forms), and bit scans (bsf, bsr), where they would
// otherwise go a lane at a time through memory.
#include <immintrin.h>
typedef unsigned char u8 __attribute__((vector_size(16)));
typedef unsigned char u8x32 __attribute__((vector_size(32)));
typedef unsigned short u16 __attribute__((vector_size(16)));
typedef unsigned u32 __attribute__((vector_size(16)));
typedef unsigned u32x8 __attribute__((vector_size(32)));

int mask16(u8 a, u8 b) { return _mm_movemask_epi8((__m128i)(a == b)); }
int any16(u8 a, u8 b) { return _mm_movemask_epi8((__m128i)(a == b)) != 0; }
__attribute__((target("avx2"))) int mask32(const u8x32 *a, const u8x32 *b) {
    return _mm256_movemask_epi8((__m256i)(*a == *b));
}
__attribute__((target("avx2"))) int any32(const u8x32 *a, const u8x32 *b) {
    return _mm256_movemask_epi8((__m256i)(*a == *b)) != 0;
}
u32 rot32(u32 a) { return __builtin_shufflevector(a, a, 1, 2, 3, 0); }
u16 swap16(u16 a) { return __builtin_shufflevector(a, a, 1, 0, 3, 2, 5, 4, 7, 6); }
u16 lohi16(u16 a) { return __builtin_shufflevector(a, a, 3, 2, 1, 0, 4, 6, 5, 7); }
__attribute__((target("ssse3"))) u8 rotb(u8 a) {
    return __builtin_shufflevector(a, a, 3, 0, 1, 2, 7, 4, 5, 6, 11, 8, 9, 10, 15, 12, 13, 14);
}
__attribute__((target("avx2"))) void rotb32(u8x32 *o, const u8x32 *a) {
    *o = __builtin_shufflevector(*a, *a, 3, 0, 1, 2, 7, 4, 5, 6, 11, 8, 9, 10, 15, 12, 13, 14, 19, 16, 17, 18, 23,
                                 20, 21, 22, 27, 24, 25, 26, 31, 28, 29, 30);
}
__attribute__((target("avx2"))) void rot32x8(u32x8 *o, const u32x8 *a) {
    *o = __builtin_shufflevector(*a, *a, 3, 0, 1, 2, 7, 4, 5, 6);
}
u8 shl4(u8 a) {
    u8 z = {0};
    return __builtin_shufflevector(z, a, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27);
}
u8 shr3(u8 a) {
    u8 z = {0};
    return __builtin_shufflevector(a, z, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18);
}
__attribute__((target("ssse3"))) u8 window5(u8 a, u8 b) {
    return __builtin_shufflevector(a, b, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20);
}
int tz(unsigned x) { return __builtin_ctz(x); }
int lz(unsigned long x) { return __builtin_clzl(x); }
int tz0(unsigned x) { return x ? __builtin_ctz(x) : 32; }
int lz0(unsigned x) { return x ? __builtin_clz(x) : 32; }
