#include <stdint.h>
#include <stdio.h>

void mulhu_8(const uint16_t *, const uint16_t *, uint16_t *);
void mulhs_8(const uint16_t *, const uint16_t *, uint16_t *);
void mulhu_19(const uint16_t *, const uint16_t *, uint16_t *);
void mulhs_18(const uint16_t *, const uint16_t *, uint16_t *);
void mulhu_const(const uint16_t *, uint16_t *);
void mulhu_16(const uint16_t *, const uint16_t *, uint16_t *);
void mulhs_4(const uint16_t *, const uint16_t *, uint16_t *);
void mulhu_16_avx2(const uint16_t *, const uint16_t *, uint16_t *);

static uint32_t s = 0x9e3779b9u;
static uint16_t next(void) {
    s ^= s << 13;
    s ^= s >> 17;
    s ^= s << 5;
    return (uint16_t)(s >> 7);
}

static void show(const char *name, const uint16_t *r, int n) {
    printf("%s", name);
    for (int i = 0; i < n; i++) printf(" %u", r[i]);
    printf("\n");
}

int main(void) {
    uint16_t a[16], b[16], r[16];
    for (int round = 0; round < 8; round++) {
        for (int i = 0; i < 16; i++) {
            a[i] = next();
            b[i] = next();
        }
        // (The edges: the largest and the most negative.)
        a[0] = 0xffff, b[0] = 0xffff, a[1] = 0x8000, b[1] = 0x8000;
        mulhu_8(a, b, r), show("mulhu_8", r, 8);
        mulhs_8(a, b, r), show("mulhs_8", r, 8);
        mulhu_19(a, b, r), show("mulhu_19", r, 8);
        mulhs_18(a, b, r), show("mulhs_18", r, 8);
        mulhu_const(a, r), show("mulhu_const", r, 8);
        mulhu_16(a, b, r), show("mulhu_16", r, 16);
        mulhs_4(a, b, r), show("mulhs_4", r, 4);
        if (__builtin_cpu_supports("avx2")) mulhu_16_avx2(a, b, r), show("mulhu_16_avx2", r, 16);
    }
    return 0;
}
