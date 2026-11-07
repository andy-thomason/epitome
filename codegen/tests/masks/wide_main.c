// Drives tests/masks/wide.ll on random bytes, and prints what comes back.
#include <stdio.h>
#include <string.h>
#include <stdint.h>
void sel(uint8_t *, const uint8_t *, const uint8_t *); __int128 bits(const uint8_t *, const uint8_t *);
_Bool any(const uint8_t *); void frombits(uint8_t *, const uint8_t *, __int128); void st(uint8_t *, const uint8_t *);
long cnt(const uint8_t *);
uint64_t b64(const uint8_t *, const uint8_t *); uint32_t b32(const uint8_t *, const uint8_t *); void st64(uint8_t *, const uint8_t *);
void f64(uint8_t *, const uint8_t *, uint64_t);
void selbits(uint8_t *, const uint8_t *, const uint8_t *, uint16_t); void selload(uint8_t *, const uint8_t *, const uint8_t *, const uint8_t *);
static uint32_t seed = 12345; static uint8_t rnd(void) { seed = seed * 1103515245 + 12345; return seed >> 16; }
static void dump(const uint8_t *p, int n) { unsigned h = 0; for (int i = 0; i < n; i++) h = h * 31 + p[i]; printf("%08x ", h); }
int main(void) {
    for (int t = 0; t < 20; t++) {
        uint8_t a[128], b[128], o[128];
        for (int i = 0; i < 128; i++) { a[i] = rnd() % 5; b[i] = (t & 1) ? a[i] : rnd() % 5; if (t == 3) a[i] = 1; }
        sel(o, a, b); dump(o, 128);
        __int128 m = bits(a, b); printf("%016llx%016llx ", (unsigned long long)(m >> 64), (unsigned long long)m);
        printf("%d ", any(a));
        frombits(o, a, ((__int128)0x123456789abcdef0ULL << 64 | 0xfedcba9876543210ULL) ^ m); dump(o, 128);
        memset(o, 0xaa, 128); st(o, a); dump(o, 20);
        printf("%ld ", cnt(a));
        printf("%016llx %08x ", (unsigned long long)b64(a, b), b32(a, b));
        memset(o, 0x55, 128); st64(o, a); dump(o, 12);
        f64(o, a, 0x0f0f00ff12345678ULL ^ t); dump(o, 64);
        selbits(o, a, b, (uint16_t)(0x9a3c * (t + 1))); dump(o, 64);
        uint8_t mb[2] = { (uint8_t)(0x5f ^ t), (uint8_t)(0x17 * t) }; selload(o, a, b, mb); dump(o, 64);
        printf("\n");
    }
    return 0;
}
