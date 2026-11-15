#include <stdint.h>
#include <stdio.h>

void widen_mul(const int32_t *, const int32_t *, int32_t *);
void swap_halves(const int32_t *, int32_t *);

int main(void) {
    static int32_t a[16] __attribute__((aligned(64))), b[16] __attribute__((aligned(64))), r[16] __attribute__((aligned(64)));
    uint32_t s = 0x9e3779b9u;
    for (int round = 0; round < 8; round++) {
        for (int i = 0; i < 16; i++) {
            s ^= s << 13;
            s ^= s >> 17;
            s ^= s << 5;
            a[i] = (int32_t)s;
            b[i] = (int32_t)(s * 2654435761u) >> (i % 7);
        }
        widen_mul(a, b, r);
        for (int i = 0; i < 16; i++) printf("%d ", r[i]);
        swap_halves(a, r);
        for (int i = 0; i < 16; i++) printf("%d ", r[i]);
        printf("\n");
    }
    return 0;
}
