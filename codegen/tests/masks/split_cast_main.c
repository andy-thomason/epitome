#include <stdint.h>
#include <stdio.h>

void sext8(const int32_t *, int8_t *);
void zext16(const int32_t *, uint16_t *);
void sext64(const int32_t *, int64_t *);
uint16_t bits(uint16_t, const int32_t *);

int main(void) {
    int32_t a[16];
    uint32_t s = 0x2545f491u;
    for (int round = 0; round < 8; round++) {
        for (int i = 0; i < 16; i++) {
            s ^= s << 13;
            s ^= s >> 17;
            s ^= s << 5;
            a[i] = (int32_t)(s >> (s % 24));
        }
        int8_t b[16];
        uint16_t h[16];
        int64_t q[16];
        sext8(a, b);
        zext16(a, h);
        sext64(a, q);
        for (int i = 0; i < 16; i++) printf("%d,%u,%lld ", b[i], h[i], (long long)q[i]);
        printf("%04x\n", bits((uint16_t)s, a));
    }
    return 0;
}
