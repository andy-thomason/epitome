#include <stdint.h>
#include <stdio.h>
#include <string.h>

void trunc_8x32_16(const void *, void *);
void trunc_4x32_16(const void *, void *);
void trunc_16x16_8(const void *, void *);
void trunc_8x16_8(const void *, void *);
void trunc_4x64_32(const void *, void *);
void trunc_2x64_32(const void *, void *);
void trunc_16x32_8(const void *, void *);
void trunc_8x32_16_avx2(const void *, void *);
void trunc_16x16_8_avx2(const void *, void *);
void trunc_4x64_32_avx2(const void *, void *);

static uint32_t s = 0x9e3779b9u;
static uint32_t next(void) {
    s ^= s << 13;
    s ^= s >> 17;
    s ^= s << 5;
    return s;
}

static void show(const char *name, void (*f)(const void *, void *), const uint32_t *in, int bytes) {
    uint8_t out[64];
    memset(out, 0xa5, sizeof out);
    f(in, out);
    printf("%s", name);
    for (int i = 0; i < bytes + 4; i++) printf(" %02x", out[i]);
    printf("\n");
}

int main(void) {
    uint32_t in[16];
    for (int round = 0; round < 8; round++) {
        for (int i = 0; i < 16; i++) in[i] = next();
        show("8x32_16", trunc_8x32_16, in, 16);
        show("4x32_16", trunc_4x32_16, in, 8);
        show("16x16_8", trunc_16x16_8, in, 16);
        show("8x16_8", trunc_8x16_8, in, 8);
        show("4x64_32", trunc_4x64_32, in, 16);
        show("2x64_32", trunc_2x64_32, in, 8);
        show("16x32_8", trunc_16x32_8, in, 16);
        if (__builtin_cpu_supports("avx2")) {
            show("8x32_16_avx2", trunc_8x32_16_avx2, in, 16);
            show("16x16_8_avx2", trunc_16x16_8_avx2, in, 16);
            show("4x64_32_avx2", trunc_4x64_32_avx2, in, 16);
        }
    }
    return 0;
}
