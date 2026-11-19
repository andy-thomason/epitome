#include <stdint.h>
#include <stdio.h>

void fold_aligned(const void *, void *);
void fold_unaligned(const void *, void *);

int main(void) {
    static uint16_t p[32] __attribute__((aligned(16)));
    uint16_t q[9];
    uint32_t s = 0x9e3779b9u;
    for (int round = 0; round < 8; round++) {
        for (int i = 0; i < 32; i++) {
            s ^= s << 13, s ^= s >> 17, s ^= s << 5;
            p[i] = (uint16_t)s;
        }
        for (int i = 0; i < 9; i++) q[i] = (uint16_t)(i * 4097 + round);
        fold_aligned(p, q + 1);
        for (int i = 1; i < 9; i++) printf("%u ", q[i]);
        fold_unaligned(p, q + 1);
        for (int i = 1; i < 9; i++) printf("%u ", q[i]);
        printf("\n");
    }
    return 0;
}
