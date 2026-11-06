// Runs each case of a module tests/intrinsics.rs made, on inputs of
// random bits, of special values (zeros, ones, extremes, infinities,
// NaNs, halves) and of small numbers, and prints what each stored.
//
// A case is a function `void f(unsigned char *out, const unsigned char
// *in)` and two descriptions of the bytes: what `in` holds and what `out`
// gets, each a list of `KS*N@O` (N lanes of kind K, S bytes each, at
// offset O): K is `i` (integer), `n` (integer, not zero), `f` (floating
// point), `r` (floating point in the range of the integers made of it:
// finite, not too big), `p` (as `r`, but not negative), `b` (a boolean in a byte), `u` (an unsigned
// number below a limit, written `uS*N@O<L`). NaNs are printed as `nan`,
// whatever their bits: which NaN an operation gives is not fixed.
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

struct Case {
    void (*f)(unsigned char *, const unsigned char *);
    const char *name;
    const char *in;
    const char *out;
};
extern struct Case cases[];
extern int ncases;

static unsigned long long state = 0x9e3779b97f4a7c15ull;
static unsigned long long next(void) {
    state ^= state << 13;
    state ^= state >> 7;
    state ^= state << 17;
    return state;
}

static const unsigned long long special_int[] = {
    0, 1, 2, 3, 7, 8, 15, 16, 31, 32, 63, 64, 127, 128, 255, ~0ull, ~0ull - 1, 0x80ull << 56,
    0x7fffffffffffffffull, 0x8000000000000000ull, 0x55, 0xaa, 0x0123456789abcdefull,
};
static const float special_f32[] = {
    0.0f, -0.0f, 1.0f, -1.0f, 0.5f, -0.5f, 1.5f, 2.5f, -2.5f, 3.75f, 1e-40f, -1e-40f, 1e30f, -1e30f,
    3.4028235e38f, 1.17549435e-38f, 100.0f, 0.1f, 2.0f, 1e10f, -1e10f, 4294967296.0f,
};
static const double special_f64[] = {
    0.0, -0.0, 1.0, -1.0, 0.5, -0.5, 1.5, 2.5, -2.5, 3.75, 4.9e-324, -1e-310, 1e300, -1e300,
    1.7976931348623157e308, 2.2250738585072014e-308, 100.0, 0.1, 2.0, 1e10, -1e10, 9.3e18,
};
static const unsigned short special_f16[] = {
    0x0000, 0x8000, 0x3c00, 0xbc00, 0x3800, 0xb800, 0x3e00, 0x4100, 0xc100, 0x0001, 0x8001,
    0x7bff, 0xfbff, 0x0400, 0x5640, 0x2e66, 0x4000,
};
static const unsigned long long nan_inf_f32[] = {0x7f800000, 0xff800000, 0x7fc00000, 0x7f800001, 0xffc00001};
static const unsigned long long nan_inf_f64[] = {
    0x7ff0000000000000ull, 0xfff0000000000000ull, 0x7ff8000000000000ull, 0x7ff0000000000001ull,
};
static const unsigned long long nan_inf_f16[] = {0x7c00, 0xfc00, 0x7e00, 0x7c01};

#define N(a) (sizeof(a) / sizeof((a)[0]))

// One element of `kind`, `size` bytes, in the style of round `r`.
static void element(char kind, int size, unsigned long long limit, int r, unsigned char *p) {
    unsigned long long bits = next();
    int style = r % 4;
    if (kind == 'b') {
        bits &= 1;
    } else if (kind == 'u') {
        bits = limit ? bits % limit : 0;
    } else if (kind == 'i' || kind == 'n') {
        if (style == 1) bits = special_int[next() % N(special_int)];
        if (style == 2) bits = (next() % 21) - 10;
        if (kind == 'n') {
            // Not zero in its bytes; (a 1 where the low byte is zero).
            unsigned long long mask = size >= 8 ? ~0ull : (1ull << (8 * size)) - 1;
            if ((bits & mask) == 0) bits |= 1;
        }
    } else {
        int range = kind == 'r' || kind == 'p';
        int pick = (int)(next() % 32);
        if (style == 0 && !range) {
            // Random bits.
        } else if (size == 4) {
            float v = (style == 2 || range) ? (float)((long long)(next() % 2001) - 1000) / 4.0f
                                             : special_f32[pick % N(special_f32)];
            if (style == 1 && !range && pick >= 28) {
                bits = nan_inf_f32[next() % N(nan_inf_f32)];
            } else {
                unsigned u;
                memcpy(&u, &v, 4);
                bits = u;
            }
        } else if (size == 8) {
            double v = (style == 2 || range) ? (double)((long long)(next() % 2001) - 1000) / 4.0
                                              : special_f64[pick % N(special_f64)];
            if (style == 1 && !range && pick >= 28) {
                bits = nan_inf_f64[next() % N(nan_inf_f64)];
            } else {
                memcpy(&bits, &v, 8);
            }
        } else if (size == 2) {
            if (style == 1 && !range && pick >= 28) {
                bits = nan_inf_f16[next() % N(nan_inf_f16)];
            } else if (style == 2 || range) {
                // Small integers and quarters, as halves: sign, exponent,
                // a few mantissa bits.
                int m = (int)(next() % 64);
                bits = (m == 0) ? 0 : (((next() & 1) << 15) | ((14 + next() % 8) << 10) | ((next() % 16) << 6));
            } else {
                bits = special_f16[pick % N(special_f16)];
            }
        }
    }
    // (Not negative: the sign bit cleared.)
    if (kind == 'p') bits &= ~(1ull << (8 * (size < 8 ? size : 8) - 1));
    for (int k = 0; k < size; k++) p[k] = (unsigned char)(k < 8 ? bits >> (8 * k) : ((kind == 'i' || kind == 'n') && style == 1 ? (bits >> 63 ? 0xff : 0) : next()));
}

// Fills `in` as `desc` says.
static void fill(const char *desc, unsigned char *in, int r) {
    const char *p = desc;
    while (*p) {
        char kind = *p++;
        int size = (int)strtol(p, (char **)&p, 10);
        int lanes = 1, off = 0;
        unsigned long long limit = 0;
        if (*p == '*') lanes = (int)strtol(p + 1, (char **)&p, 10);
        if (*p == '@') off = (int)strtol(p + 1, (char **)&p, 10);
        if (*p == '<') limit = strtoull(p + 1, (char **)&p, 10);
        for (int l = 0; l < lanes; l++) element(kind, size, limit, r, in + off + l * size);
        while (*p == ' ') p++;
    }
}

static int is_nan(const unsigned char *b, int size) {
    unsigned long long u = 0;
    memcpy(&u, b, size);
    if (size == 2) return (u & 0x7c00) == 0x7c00 && (u & 0x3ff);
    if (size == 4) return (u & 0x7f800000) == 0x7f800000 && (u & 0x7fffff);
    if (size == 8) return (u & 0x7ff0000000000000ull) == 0x7ff0000000000000ull && (u & 0xfffffffffffffull);
    return 0;
}

// Prints `out` as `desc` says.
static void print(const char *desc, const unsigned char *out) {
    const char *p = desc;
    while (*p) {
        char kind = *p++;
        int size = (int)strtol(p, (char **)&p, 10);
        int lanes = 1, off = 0;
        if (*p == '*') lanes = (int)strtol(p + 1, (char **)&p, 10);
        if (*p == '@') off = (int)strtol(p + 1, (char **)&p, 10);
        if (*p == '<') strtoull(p + 1, (char **)&p, 10);
        for (int l = 0; l < lanes; l++) {
            const unsigned char *e = out + off + l * size;
            if ((kind == 'f' || kind == 'r' || kind == 'p') && is_nan(e, size)) {
                printf(" nan");
                continue;
            }
            printf(" ");
            for (int k = size - 1; k >= 0; k--) printf("%02x", e[k]);
        }
        while (*p == ' ') p++;
    }
}

int main(int argc, char **argv) {
    int rounds = argc > 1 ? atoi(argv[1]) : 64;
    for (int c = 0; c < ncases; c++) {
        // (Seeded by its name, so that each case's inputs stay the same
        // whatever other cases there are.)
        state = 0xcbf29ce484222325ull;
        for (const char *p = cases[c].name; *p; p++) state = (state ^ (unsigned char)*p) * 0x100000001b3ull;
        if (state == 0) state = 1;
        for (int r = 0; r < rounds; r++) {
            unsigned char in[4096] __attribute__((aligned(16)));
            unsigned char out[4096] __attribute__((aligned(16)));
            memset(in, 0, sizeof in);
            memset(out, 0, sizeof out);
            fill(cases[c].in, in, r);
            cases[c].f(out, in);
            printf("%s %d:", cases[c].name, r);
            print(cases[c].out, out);
            printf("\n");
        }
    }
    return 0;
}
