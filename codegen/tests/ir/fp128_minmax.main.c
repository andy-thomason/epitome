// Each fp128 operation of fp128_minmax.ll on every pair of the values
// below (each exact in a double), against clang's build of the same
// operation on doubles (LLVM's llvm.minimum.f64 and the rest). It prints
// each pair that differs, and fails if any does.
#include <math.h>
#include <stdint.h>
#include <stdio.h>
#include <string.h>

__float128 minimum128(__float128, __float128);
__float128 maximum128(__float128, __float128);
__float128 minimumnum128(__float128, __float128);
__float128 maximumnum128(__float128, __float128);

static double minimum64(double a, double b) { return __builtin_elementwise_minimum(a, b); }
static double maximum64(double a, double b) { return __builtin_elementwise_maximum(a, b); }
static double minimumnum64(double a, double b) { return __builtin_elementwise_minimumnum(a, b); }
static double maximumnum64(double a, double b) { return __builtin_elementwise_maximumnum(a, b); }

static uint64_t bits(double x) {
    uint64_t u;
    memcpy(&u, &x, 8);
    return u;
}

int main(void) {
    volatile double v[] = {0.0, -0.0, 1.0, -1.0, 2.5, -2.5, 0x1p-1074, -0x1p-1074, INFINITY, -INFINITY, NAN, -NAN};
    struct {
        const char *name;
        __float128 (*q)(__float128, __float128);
        double (*d)(double, double);
    } ops[] = {
        {"minimum", minimum128, minimum64},
        {"maximum", maximum128, maximum64},
        {"minimumnum", minimumnum128, minimumnum64},
        {"maximumnum", maximumnum128, maximumnum64},
    };
    int n = sizeof v / sizeof v[0], bad = 0;
    for (int o = 0; o < 4; o++) {
        for (int i = 0; i < n; i++) {
            for (int j = 0; j < n; j++) {
                double want = ops[o].d(v[i], v[j]);
                double got = (double)ops[o].q((__float128)v[i], (__float128)v[j]);
                if (isnan(want) ? !isnan(got) : bits(want) != bits(got)) {
                    printf("%s(%a, %a): %a, not %a\n", ops[o].name, v[i], v[j], got, want);
                    bad = 1;
                }
            }
        }
    }
    printf("%s\n", bad ? "differs" : "same");
    return bad;
}
