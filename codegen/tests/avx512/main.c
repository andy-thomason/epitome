/* Runs tests/avx512/kernels.ll's kernels on several sets of inputs and
   prints what each writes, in hex, a line per kernel and set. */
#include <stdint.h>
#include <stdio.h>
#include <string.h>

#define PLAIN(n) void n(const void *, const void *, void *);
#define TYPE(t, it, et)                                                                      \
    PLAIN(arith_##t) PLAIN(compares_##t) PLAIN(minmax_##t)                                   \
    void selects_##t(const void *, const void *, void *, it, et);                            \
    void mask_memory_##t(const void *, const void *, void *, void *);
TYPE(v16i32, uint16_t, int32_t)
TYPE(v8i64, uint8_t, int64_t)
TYPE(v32i16, uint32_t, int16_t)
TYPE(v64i8, uint64_t, int8_t)
PLAIN(shifts_v16i32) PLAIN(shifts_v8i64) PLAIN(shifts_v32i16)
PLAIN(bits_v16i32) PLAIN(bits_v8i64)
PLAIN(lanes_fsel) PLAIN(lanes_mixed)
PLAIN(shifts_v8i32) PLAIN(shifts_v4i32) PLAIN(shifts_v4i64) PLAIN(shifts_v2i64)

static unsigned char a[64] __attribute__((aligned(64))), b[64] __attribute__((aligned(64)));
static unsigned char out[1024] __attribute__((aligned(64)));

static void show(const char *name, int set, size_t n) {
    printf("%s %d ", name, set);
    for (size_t k = 0; k < n; k++) printf("%02x", out[k]);
    printf("\n");
}

static uint64_t state;
static uint64_t next(void) {
    state = state * 6364136223846793005ull + 1442695040888963407ull;
    return state >> 24;
}

#define RUN(n, len)               \
    memset(out, 0, sizeof out); \
    n(a, b, out);               \
    show(#n, set, len);
#define SELECTS(t, bits, k)                \
    memset(out, 0, sizeof out);            \
    selects_##t(a, b, out, bits, k);       \
    show("selects_" #t, set, 6 * 64);
#define MASKS(t)                                \
    memset(out, 0, sizeof out);                 \
    memcpy(m, &pattern, 8);                     \
    mask_memory_##t(a, b, out, m);              \
    memcpy(out + 64, m, 8);                     \
    show("mask_memory_" #t, set, 72);

int main(void) {
    for (int set = 0; set < 6; set++) {
        state = 0x9e3779b97f4a7c15ull * (set + 1);
        for (int k = 0; k < 64; k++) {
            a[k] = (unsigned char)next();
            /* Some lanes equal, some edge values. */
            b[k] = (k % 7 == 0) ? a[k] : (unsigned char)next();
        }
        if (set == 1) { memset(a, 0x80, 16); memset(b, 0x7f, 16); }
        if (set == 2) { memset(a, 0, 32); memset(b + 32, 0xff, 32); }
        if (set == 3) { memset(a, 0xff, 8); a[8] = 1; memset(b, 0, 8); }
        uint64_t pattern = next() * 0x9a3c5f0e17d2b649ull;
        unsigned char m[8];
        RUN(arith_v16i32, 7 * 64) RUN(arith_v8i64, 7 * 64) RUN(arith_v32i16, 7 * 64) RUN(arith_v64i8, 6 * 64)
        RUN(shifts_v16i32, 6 * 64) RUN(shifts_v8i64, 6 * 64) RUN(shifts_v32i16, 6 * 64)
        RUN(compares_v16i32, 20) RUN(compares_v8i64, 10) RUN(compares_v32i16, 40) RUN(compares_v64i8, 80)
        RUN(minmax_v16i32, 5 * 64) RUN(minmax_v8i64, 5 * 64) RUN(minmax_v32i16, 5 * 64) RUN(minmax_v64i8, 5 * 64)
        RUN(bits_v16i32, 2 * 64) RUN(bits_v8i64, 2 * 64)
        RUN(lanes_mixed, 64)
        RUN(lanes_fsel, 64)
        RUN(shifts_v8i32, 3 * 32) RUN(shifts_v4i32, 3 * 16) RUN(shifts_v4i64, 2 * 32) RUN(shifts_v2i64, 2 * 16)
        SELECTS(v16i32, (uint16_t)pattern, (int32_t)next())
        SELECTS(v8i64, (uint8_t)pattern, (int64_t)(next() << 20))
        SELECTS(v32i16, (uint32_t)pattern, (int16_t)next())
        SELECTS(v64i8, pattern, (int8_t)next())
        MASKS(v16i32) MASKS(v8i64) MASKS(v32i16) MASKS(v64i8)
    }
    return 0;
}
