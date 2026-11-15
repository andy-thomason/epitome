#include <stdint.h>
#include <stdio.h>

void narrow8(const int32_t *, uint8_t *, uint16_t *);
void narrow4(const int32_t *, uint16_t *);

int main(void) {
    static int32_t a[8] __attribute__((aligned(64)));
    static uint8_t b[8] __attribute__((aligned(64)));
    static uint16_t h[8] __attribute__((aligned(64))), q[4] __attribute__((aligned(64)));
    uint32_t s = 0x9e3779b9u;
    for (int round = 0; round < 8; round++) {
        for (int i = 0; i < 8; i++) {
            s ^= s << 13;
            s ^= s >> 17;
            s ^= s << 5;
            a[i] = (int32_t)s;
        }
        narrow8(a, b, h);
        narrow4(a, q);
        for (int i = 0; i < 8; i++) printf("%u %u ", b[i], h[i]);
        for (int i = 0; i < 4; i++) printf("%u ", q[i]);
        printf("\n");
    }
    return 0;
}
