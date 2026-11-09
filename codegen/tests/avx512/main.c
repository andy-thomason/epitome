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
PLAIN(bits_v16i32) PLAIN(bits_v8i64) PLAIN(consts_v16i32)
uint16_t all_v16i32(const void *, const void *, void *, uint16_t);
PLAIN(fsel_v16f32) PLAIN(mixed_v16i8_v16i32) PLAIN(fcompares_v16f32) PLAIN(fcompares_v8f64)
PLAIN(compares_v32i8) void selects_v32i8(const void *, const void *, void *, uint64_t); void mask_memory_v32i8(const void *, const void *, void *, void *);
PLAIN(compares_v16i16) void selects_v16i16(const void *, const void *, void *, uint64_t); void mask_memory_v16i16(const void *, const void *, void *, void *);
PLAIN(compares_v8i32) void selects_v8i32(const void *, const void *, void *, uint64_t); void mask_memory_v8i32(const void *, const void *, void *, void *);
PLAIN(compares_v4i64) void selects_v4i64(const void *, const void *, void *, uint64_t); void mask_memory_v4i64(const void *, const void *, void *, void *);
PLAIN(fcompares_v8f32)
PLAIN(fcompares_v4f64)
PLAIN(compares_v16i8) void selects_v16i8(const void *, const void *, void *, uint64_t); void mask_memory_v16i8(const void *, const void *, void *, void *);
PLAIN(compares_v8i16) void selects_v8i16(const void *, const void *, void *, uint64_t); void mask_memory_v8i16(const void *, const void *, void *, void *);
PLAIN(compares_v4i32) void selects_v4i32(const void *, const void *, void *, uint64_t); void mask_memory_v4i32(const void *, const void *, void *, void *);
PLAIN(compares_v2i64) void selects_v2i64(const void *, const void *, void *, uint64_t); void mask_memory_v2i64(const void *, const void *, void *, void *);
PLAIN(fcompares_v4f32)
PLAIN(fcompares_v2f64)
PLAIN(mixed_v8i32_v8i64)
PLAIN(mixed_v4i64_v4f32)
PLAIN(mixed_v16i16_v16i8)
PLAIN(mixed_v4f64_v4i32)
PLAIN(novl_selects_v8i32)
void ternlog0(const void *, const void *, const void *, void *); void ternlog1(const void *, const void *, const void *, void *); void ternlog2(const void *, const void *, const void *, void *); void ternlog3(const void *, const void *, const void *, void *); void ternlog4(const void *, const void *, const void *, void *); void ternlog5(const void *, const void *, const void *, void *); void ternlog6(const void *, const void *, const void *, void *); void ternlog7(const void *, const void *, const void *, void *); void ternlog8(const void *, const void *, const void *, void *); void ternlog9(const void *, const void *, const void *, void *); void ternlog10(const void *, const void *, const void *, void *); void ternlog11(const void *, const void *, const void *, void *); void ternlog12(const void *, const void *, const void *, void *); void ternlog13(const void *, const void *, const void *, void *); void ternlog14(const void *, const void *, const void *, void *); void ternlog15(const void *, const void *, const void *, void *); void ternlog16(const void *, const void *, const void *, void *); void ternlog17(const void *, const void *, const void *, void *); void ternlog18(const void *, const void *, const void *, void *); void ternlog19(const void *, const void *, const void *, void *); void ternlog20(const void *, const void *, const void *, void *); void ternlog21(const void *, const void *, const void *, void *); void ternlog22(const void *, const void *, const void *, void *); void ternlog23(const void *, const void *, const void *, void *); void ternlog24(const void *, const void *, const void *, void *); void ternlog25(const void *, const void *, const void *, void *); void ternlog26(const void *, const void *, const void *, void *); void ternlog27(const void *, const void *, const void *, void *); void ternlog28(const void *, const void *, const void *, void *); void ternlog29(const void *, const void *, const void *, void *); void ternlog30(const void *, const void *, const void *, void *); void ternlog31(const void *, const void *, const void *, void *); void ternlog_adder(const void *, const void *, const void *, void *);
PLAIN(shifts_v8i32) PLAIN(shifts_v4i32) PLAIN(shifts_v4i64) PLAIN(shifts_v2i64)

static unsigned char a[64] __attribute__((aligned(64))), b[64] __attribute__((aligned(64)));
static unsigned char c3[64] __attribute__((aligned(64)));
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
        for (int k = 0; k < 64; k++) c3[k] = (unsigned char)next();
        uint64_t pattern = next() * 0x9a3c5f0e17d2b649ull;
        unsigned char m[8];
        RUN(arith_v16i32, 7 * 64) RUN(arith_v8i64, 7 * 64) RUN(arith_v32i16, 7 * 64) RUN(arith_v64i8, 6 * 64)
        RUN(shifts_v16i32, 6 * 64) RUN(shifts_v8i64, 6 * 64) RUN(shifts_v32i16, 6 * 64)
        RUN(compares_v16i32, 20) RUN(compares_v8i64, 10) RUN(compares_v32i16, 40) RUN(compares_v64i8, 80)
        RUN(minmax_v16i32, 5 * 64) RUN(minmax_v8i64, 5 * 64) RUN(minmax_v32i16, 5 * 64) RUN(minmax_v64i8, 5 * 64)
        RUN(bits_v16i32, 2 * 64) RUN(bits_v8i64, 2 * 64) RUN(consts_v16i32, 10 * 64)
        RUN(mixed_v16i8_v16i32, 64)
        memset(out, 0, sizeof out);
        { uint16_t r = all_v16i32(a, b, out, (uint16_t)pattern); memcpy(out + 64, &r, 2); }
        show("all_v16i32", set, 66);
        RUN(fsel_v16f32, 64) RUN(fcompares_v16f32, 28) RUN(fcompares_v8f64, 14)
        RUN(shifts_v8i32, 3 * 32) RUN(shifts_v4i32, 3 * 16) RUN(shifts_v4i64, 2 * 32) RUN(shifts_v2i64, 2 * 16)
        SELECTS(v16i32, (uint16_t)pattern, (int32_t)next())
        SELECTS(v8i64, (uint8_t)pattern, (int64_t)(next() << 20))
        SELECTS(v32i16, (uint32_t)pattern, (int16_t)next())
        SELECTS(v64i8, pattern, (int8_t)next())
        RUN(compares_v32i8, 40)
        memset(out, 0, sizeof out); selects_v32i8(a, b, out, pattern); show("selects_v32i8", set, 68);
        memset(out, 0, sizeof out); memcpy(m, &pattern, 8); mask_memory_v32i8(a, b, out, m); memcpy(out + 64, m, 8); show("mask_memory_v32i8", set, 72);
        RUN(compares_v16i16, 20)
        memset(out, 0, sizeof out); selects_v16i16(a, b, out, pattern); show("selects_v16i16", set, 66);
        memset(out, 0, sizeof out); memcpy(m, &pattern, 8); mask_memory_v16i16(a, b, out, m); memcpy(out + 64, m, 8); show("mask_memory_v16i16", set, 72);
        RUN(compares_v8i32, 10)
        memset(out, 0, sizeof out); selects_v8i32(a, b, out, pattern); show("selects_v8i32", set, 65);
        memset(out, 0, sizeof out); memcpy(m, &pattern, 8); mask_memory_v8i32(a, b, out, m); memcpy(out + 64, m, 8); show("mask_memory_v8i32", set, 72);
        RUN(compares_v4i64, 10)
        memset(out, 0, sizeof out); selects_v4i64(a, b, out, pattern); show("selects_v4i64", set, 65);
        memset(out, 0, sizeof out); memcpy(m, &pattern, 8); mask_memory_v4i64(a, b, out, m); memcpy(out + 64, m, 8); show("mask_memory_v4i64", set, 72);
        RUN(fcompares_v8f32, 96)
        RUN(fcompares_v4f64, 96)
        RUN(compares_v16i8, 20)
        memset(out, 0, sizeof out); selects_v16i8(a, b, out, pattern); show("selects_v16i8", set, 34);
        memset(out, 0, sizeof out); memcpy(m, &pattern, 8); mask_memory_v16i8(a, b, out, m); memcpy(out + 64, m, 8); show("mask_memory_v16i8", set, 72);
        RUN(compares_v8i16, 10)
        memset(out, 0, sizeof out); selects_v8i16(a, b, out, pattern); show("selects_v8i16", set, 33);
        memset(out, 0, sizeof out); memcpy(m, &pattern, 8); mask_memory_v8i16(a, b, out, m); memcpy(out + 64, m, 8); show("mask_memory_v8i16", set, 72);
        RUN(compares_v4i32, 10)
        memset(out, 0, sizeof out); selects_v4i32(a, b, out, pattern); show("selects_v4i32", set, 33);
        memset(out, 0, sizeof out); memcpy(m, &pattern, 8); mask_memory_v4i32(a, b, out, m); memcpy(out + 64, m, 8); show("mask_memory_v4i32", set, 72);
        RUN(compares_v2i64, 10)
        memset(out, 0, sizeof out); selects_v2i64(a, b, out, pattern); show("selects_v2i64", set, 33);
        memset(out, 0, sizeof out); memcpy(m, &pattern, 8); mask_memory_v2i64(a, b, out, m); memcpy(out + 64, m, 8); show("mask_memory_v2i64", set, 72);
        RUN(fcompares_v4f32, 80)
        RUN(fcompares_v2f64, 80)
        RUN(mixed_v8i32_v8i64, 64)
        RUN(mixed_v4i64_v4f32, 64)
        RUN(mixed_v16i16_v16i8, 64)
        RUN(mixed_v4f64_v4i32, 64)
        RUN(novl_selects_v8i32, 32)
        memset(out, 0, sizeof out); ternlog0(a, b, c3, out); show("ternlog0", set, 128);
        memset(out, 0, sizeof out); ternlog1(a, b, c3, out); show("ternlog1", set, 128);
        memset(out, 0, sizeof out); ternlog2(a, b, c3, out); show("ternlog2", set, 128);
        memset(out, 0, sizeof out); ternlog3(a, b, c3, out); show("ternlog3", set, 128);
        memset(out, 0, sizeof out); ternlog4(a, b, c3, out); show("ternlog4", set, 128);
        memset(out, 0, sizeof out); ternlog5(a, b, c3, out); show("ternlog5", set, 128);
        memset(out, 0, sizeof out); ternlog6(a, b, c3, out); show("ternlog6", set, 128);
        memset(out, 0, sizeof out); ternlog7(a, b, c3, out); show("ternlog7", set, 128);
        memset(out, 0, sizeof out); ternlog8(a, b, c3, out); show("ternlog8", set, 128);
        memset(out, 0, sizeof out); ternlog9(a, b, c3, out); show("ternlog9", set, 128);
        memset(out, 0, sizeof out); ternlog10(a, b, c3, out); show("ternlog10", set, 128);
        memset(out, 0, sizeof out); ternlog11(a, b, c3, out); show("ternlog11", set, 128);
        memset(out, 0, sizeof out); ternlog12(a, b, c3, out); show("ternlog12", set, 128);
        memset(out, 0, sizeof out); ternlog13(a, b, c3, out); show("ternlog13", set, 128);
        memset(out, 0, sizeof out); ternlog14(a, b, c3, out); show("ternlog14", set, 128);
        memset(out, 0, sizeof out); ternlog15(a, b, c3, out); show("ternlog15", set, 128);
        memset(out, 0, sizeof out); ternlog16(a, b, c3, out); show("ternlog16", set, 128);
        memset(out, 0, sizeof out); ternlog17(a, b, c3, out); show("ternlog17", set, 128);
        memset(out, 0, sizeof out); ternlog18(a, b, c3, out); show("ternlog18", set, 128);
        memset(out, 0, sizeof out); ternlog19(a, b, c3, out); show("ternlog19", set, 128);
        memset(out, 0, sizeof out); ternlog20(a, b, c3, out); show("ternlog20", set, 128);
        memset(out, 0, sizeof out); ternlog21(a, b, c3, out); show("ternlog21", set, 128);
        memset(out, 0, sizeof out); ternlog22(a, b, c3, out); show("ternlog22", set, 128);
        memset(out, 0, sizeof out); ternlog23(a, b, c3, out); show("ternlog23", set, 128);
        memset(out, 0, sizeof out); ternlog24(a, b, c3, out); show("ternlog24", set, 128);
        memset(out, 0, sizeof out); ternlog25(a, b, c3, out); show("ternlog25", set, 128);
        memset(out, 0, sizeof out); ternlog26(a, b, c3, out); show("ternlog26", set, 128);
        memset(out, 0, sizeof out); ternlog27(a, b, c3, out); show("ternlog27", set, 128);
        memset(out, 0, sizeof out); ternlog28(a, b, c3, out); show("ternlog28", set, 128);
        memset(out, 0, sizeof out); ternlog29(a, b, c3, out); show("ternlog29", set, 128);
        memset(out, 0, sizeof out); ternlog30(a, b, c3, out); show("ternlog30", set, 128);
        memset(out, 0, sizeof out); ternlog31(a, b, c3, out); show("ternlog31", set, 128);
        memset(out, 0, sizeof out); ternlog_adder(a, b, c3, out); show("ternlog_adder", set, 128);
        MASKS(v16i32) MASKS(v8i64) MASKS(v32i16) MASKS(v64i8)
    }
    return 0;
}
